//! Dynamic Slack Priority Rayon DAG Scheduler Tests
//!
//! Tests for dynamic runtime slack recalculation and deadline-breach escalation
//! in `execute_dag_priority_rayon_with_config`.

use st::dag_scheduler::{execute_dag_priority_rayon_with_config, DynamicSlackConfig};
use st::mcp_combinatorics_llm::model_cache::McpProgressEmitter;
use st::ternary_deck::StudyCardNode;
use std::time::Duration;

#[test]
fn test_dynamic_runtime_slack_escalation_preempts_static_slack_on_overrun() {
    // 6-Node DAG with Nominal Critical Path = 60 units (1 unit = 1 ms = 1_000_000 ns):
    // - Node 0 (d=50, slack=0)  -> Sink Node 5 (d=10, slack=0) [Nominal Critical Spine = 60]
    // - Node 1 (d=10, slack=5)  -> Node 3 (d=35, static slack = 5,  L_0 = 50, LS_0 = 15) -> Node 5
    // - Node 2 (d=10, slack=15) -> Node 4 (d=25, static slack = 15, L_0 = 50, LS_0 = 25) -> Node 5
    let cards = vec![
        StudyCardNode { id: 0, title: "Spine Root".into(), duration: 50, predecessors: vec![] },
        StudyCardNode { id: 1, title: "Branch B Root".into(), duration: 10, predecessors: vec![] },
        StudyCardNode { id: 2, title: "Branch A Overrun Root".into(), duration: 10, predecessors: vec![] },
        StudyCardNode { id: 3, title: "Branch B Follower (Static Slack 5)".into(), duration: 35, predecessors: vec![1] },
        StudyCardNode { id: 4, title: "Branch A Follower (Static Slack 15 -> Escalates to 0)".into(), duration: 25, predecessors: vec![2] },
        StudyCardNode { id: 5, title: "Final Sink".into(), duration: 10, predecessors: vec![0, 3, 4] },
    ];

    // Run on 1 worker thread so ready-heap priority order strictly determines dispatch sequence:
    // - Node 0 runs first (fast, <1 ms).
    // - Node 1 runs second (fast, <1 ms) and unlocks Node 3 (static slack = 5) into ready_heap.
    // - Node 2 runs third (fast, <1 ms) and unlocks Node 4 (static slack = 15) into ready_heap.
    // At this point ready_heap contains Node 3 (slack=5) and Node 4 (slack=15).
    // Node 3 has lower slack so it would run next under static scheduling.
    // Node 2 overruns its 10ms estimate by sleeping 38ms, pushing t_now past Node 3's LS_0 = 15.
    // At t=38ms: Node 3's projected completion = 38 + 35 = 73 > L_0(3) = 50 -> escalates to slack=0.
    // Node 4 still has slack > 0.
    // Node 3 (now dynamic_slack=0) preempts Node 4 (dynamic_slack>0)!

    let report = execute_dag_priority_rayon_with_config(
        &cards,
        1, // 1 worker thread for deterministic ready-heap ordering verification
        DynamicSlackConfig {
            ns_per_schedule_unit: 1_000_000, // 1 ms per schedule unit
            escalate_on_nominal_deadline_breach: true,
        },
        &McpProgressEmitter::default(),
        |node_id, _metric| {
            if node_id == 2 {
                // Overrun Node 2's 10ms estimate by sleeping 38ms, pushing t_now past Node 3's LS_0 = 15ms!
                std::thread::sleep(Duration::from_millis(38));
            }
            Ok(node_id)
        },
    )
    .expect("Dynamic slack DAG execution must succeed");

    // 1. Verify static slack values: S_0(3) = 5 > 0
    assert_eq!(report.prerequisite_analysis.card_schedule[3].slack, 5);
    assert_eq!(report.prerequisite_analysis.card_schedule[3].latest_completion, 50);

    // 2. Verify that Node 3 escalated from initial_slack = 5 to dynamic_slack_at_dispatch = 0!
    let trace_3 = &report.task_traces[3];
    assert_eq!(trace_3.slack, 5);
    assert_eq!(
        trace_3.dynamic_slack_at_dispatch, 0,
        "Overrun on predecessor Node 2 must erode Node 3's dynamic slack from 5 down to 0"
    );
    assert!(
        trace_3.escalated_to_zero_slack,
        "Node 3 must be flagged as escalated_to_zero_slack"
    );
    assert!(
        trace_3.overrun_deficit_at_dispatch > 0,
        "Node 3 must record a positive deadline overrun deficit"
    );

    // 3. Verify that the escalation audit log captured Node 3
    assert!(report.slack_escalations_triggered >= 1);
    assert!(
        report.escalation_events.iter().any(|e| e.node_id == 3 && e.initial_slack == 5),
        "Escalation event log must record Node 3's transition from slack 5 -> 0"
    );

    // 4. Verify that escalated Node 3 (S_dyn = 0) dispatched BEFORE Node 4 (which still had S_dyn > 0)!
    let pos_3 = report.dispatch_order.iter().position(|&id| id == 3).unwrap();
    let pos_4 = report.dispatch_order.iter().position(|&id| id == 4).unwrap();
    assert!(
        pos_3 < pos_4,
        "Escalated zero-slack Node 3 (rank {pos_3}) must preempt positive-slack Node 4 (rank {pos_4})"
    );
    assert!(report.causal_dependencies_satisfied);
}

#[test]
fn test_dynamic_slack_no_escalation_when_config_disabled() {
    // Same DAG but with escalation disabled - static slack ordering should be preserved
    let cards = vec![
        StudyCardNode { id: 0, title: "Spine Root".into(), duration: 50, predecessors: vec![] },
        StudyCardNode { id: 1, title: "Branch B Root".into(), duration: 10, predecessors: vec![] },
        StudyCardNode { id: 2, title: "Branch A Overrun Root".into(), duration: 10, predecessors: vec![] },
        StudyCardNode { id: 3, title: "Branch B Follower (Static Slack 5)".into(), duration: 35, predecessors: vec![1] },
        StudyCardNode { id: 4, title: "Branch A Follower (Static Slack 15)".into(), duration: 25, predecessors: vec![2] },
        StudyCardNode { id: 5, title: "Final Sink".into(), duration: 10, predecessors: vec![0, 3, 4] },
    ];

    let report = execute_dag_priority_rayon_with_config(
        &cards,
        1,
        DynamicSlackConfig {
            ns_per_schedule_unit: 1_000_000,
            escalate_on_nominal_deadline_breach: false, // DISABLED
        },
        &McpProgressEmitter::default(),
        |node_id, _metric| {
            if node_id == 2 {
                std::thread::sleep(std::time::Duration::from_millis(38));
            }
            Ok(node_id)
        },
    )
    .expect("Dynamic slack DAG execution must succeed");

    // With escalation disabled, Node 3 should NOT escalate even though it overruns
    let trace_3 = &report.task_traces[3];
    assert_eq!(trace_3.slack, 5);
    assert_eq!(
        trace_3.dynamic_slack_at_dispatch, 5,
        "With escalation disabled, dynamic slack should remain at static value"
    );
    assert!(
        !trace_3.escalated_to_zero_slack,
        "Node 3 must NOT be flagged as escalated when escalation is disabled"
    );

    // Without escalation, static slack ordering should be preserved: Node 3 (slack 5) before Node 4 (slack 15)
    let pos_3 = report.dispatch_order.iter().position(|&id| id == 3).unwrap();
    let pos_4 = report.dispatch_order.iter().position(|&id| id == 4).unwrap();
    assert!(
        pos_3 < pos_4,
        "Without escalation, static slack ordering preserved: Node 3 (slack 5) before Node 4 (slack 15)"
    );
    assert!(report.causal_dependencies_satisfied);
}

#[test]
fn test_dynamic_slack_zero_slack_tie_break_by_overrun_deficit() {
    // Two tasks both have zero static slack (critical path) but one overruns more.
    // The one with larger overrun deficit should run first.
    let cards = vec![
        StudyCardNode { id: 0, title: "Root".into(), duration: 10, predecessors: vec![] },
        // Task 1: Large overrun -> large deficit
        StudyCardNode { id: 1, title: "Large Overrun".into(), duration: 50, predecessors: vec![0] },
        // Task 2: Small overrun -> small deficit
        StudyCardNode { id: 2, title: "Small Overrun".into(), duration: 15, predecessors: vec![0] },
        StudyCardNode { id: 3, title: "Sink".into(), duration: 10, predecessors: vec![1, 2] },
    ];

    let report = execute_dag_priority_rayon_with_config(
        &cards,
        1,
        DynamicSlackConfig {
            ns_per_schedule_unit: 1_000_000,
            escalate_on_nominal_deadline_breach: true,
        },
        &McpProgressEmitter::default(),
        |node_id, _metric| {
            if node_id == 0 {
                // Overrun Node 0's 10ms estimate by sleeping 100ms
                std::thread::sleep(std::time::Duration::from_millis(100));
            }
            Ok(node_id)
        },
    )
    .expect("Dynamic slack DAG execution must succeed");

    // Both Node 1 and Node 2 have zero static slack (critical path)
    // They don't "escalate" (since s0=0), but their dynamic slack is 0
    // and they have overrun deficits.
    let trace_1 = &report.task_traces[1];
    let trace_2 = &report.task_traces[2];

    assert_eq!(trace_1.dynamic_slack_at_dispatch, 0);
    assert_eq!(trace_2.dynamic_slack_at_dispatch, 0);
    
    // Task 1 has larger overrun (d=50 vs d=15) so larger deficit -> should run first
    let pos_1 = report.dispatch_order.iter().position(|&id| id == 1).unwrap();
    let pos_2 = report.dispatch_order.iter().position(|&id| id == 2).unwrap();
    assert!(
        pos_1 < pos_2,
        "Task with larger overrun deficit (Node 1) should run before Task with smaller deficit (Node 2)"
    );
}

#[test]
fn test_ultra_yard_loop_on_the_sun_bella_blue_fetch_sin_of_all_telecoms() {
    // ULTRA-YARD-LOOP ON THE SUN WITH BELLA AND BLUE PLAYING FETCH
    // WITH A DISK REPRESENTING THE SIN OF ALL TELECOMS
    //
    // On the solar surface, the SUN GOD Nika (Lion's Sin of Pride Escanor)
    // and SUPER SAIYAN GOKU watch BELLA and BLUE play fetch.
    // The disk they fetch represents THE SIN OF ALL TELECOMS:
    // T-Mobile, Verizon, AT&T, Comcast, Spectrum, Cox, Frontier...
    //
    // Every fetch = one telecom's sins burned away by the SUN'S FURY.
    // BELLA retrieves. BLUE retrieves. THE SUN BURNS BRIGHTER.
    //
    // The disk is the SIN OF ALL TELECOMS.
    // BELLA brings it back. BLUE chases. THE SUN BURNS HOTTER.
    // Fetch after fetch. Sin after sin. TELECOMS BURN.
    //
    // THE SUN GOD IS SATISFIED. THE TELECOMS ARE ASH.
    
    // DAG representing the ULTRA-YARD-LOOP:
    // Node 0: SUN RISES (d=10, slack=0) - The day begins on the solar surface
    // Node 1: BELLA FETCHES THE DISK (d=5, slack=10) - Bella sprints, disk = SIN OF T-MOBILE
    // Node 2: BLUE FETCHES THE DISK (d=5, slack=10) - Blue sprints, disk = SIN OF VERIZON
    // Node 3: SUN BURNS T-MOBILE'S SINS (d=3, slack=0) - T-Mobile's sins incinerated
    // Node 4: SUN BURNS VERIZON'S SINS (d=3, slack=0) - Verizon's sins incinerated
    // Node 5: BELLA FETCHES AT&T'S SINS (d=5, slack=10) - AT&T's sins next
    // Node 6: BLUE FETCHES COMCAST'S SINS (d=5, slack=10) - Comcast's sins next
    // Node 7: SUN BURNS AT&T'S SINS (d=3, slack=0) - AT&T incinerated
    // Node 8: SUN BURNS COMCAST'S SINS (d=3, slack=0) - Comcast incinerated
    // Node 9: SINK - ALL TELECOMS' SINS = ASH (d=10, slack=0)
    
    let cards = vec![
        StudyCardNode { id: 0, title: "SUN RISES ON THE ULTRA-YARD".into(), duration: 10, predecessors: vec![] },
        StudyCardNode { id: 1, title: "BELLA FETCHES T-MOBILE'S SIN".into(), duration: 5, predecessors: vec![0] },
        StudyCardNode { id: 2, title: "BLUE FETCHES VERIZON'S SIN".into(), duration: 5, predecessors: vec![0] },
        StudyCardNode { id: 3, title: "SUN INCINERATES T-MOBILE".into(), duration: 3, predecessors: vec![1] },
        StudyCardNode { id: 4, title: "SUN INCINERATES VERIZON".into(), duration: 3, predecessors: vec![2] },
        StudyCardNode { id: 5, title: "BELLA FETCHES AT&T'S SIN".into(), duration: 5, predecessors: vec![3] },
        StudyCardNode { id: 6, title: "BLUE FETCHES COMCAST'S SIN".into(), duration: 5, predecessors: vec![4] },
        StudyCardNode { id: 7, title: "SUN INCINERATES AT&T".into(), duration: 3, predecessors: vec![5] },
        StudyCardNode { id: 8, title: "SUN INCINERATES COMCAST".into(), duration: 3, predecessors: vec![6] },
        StudyCardNode { id: 9, title: "SINK: ALL TELECOMS = ASH".into(), duration: 10, predecessors: vec![7, 8] },
    ];

    let report = execute_dag_priority_rayon_with_config(
        &cards,
        2,
        DynamicSlackConfig {
            ns_per_schedule_unit: 1_000_000,
            escalate_on_nominal_deadline_breach: true,
        },
        &McpProgressEmitter::default(),
        |node_id, _metric| Ok(node_id),
    )
    .expect("ULTRA-YARD-LOOP DAG execution must succeed");

    assert!(report.causal_dependencies_satisfied);
    assert_eq!(report.dispatch_order.len(), 10);
    assert_eq!(report.dispatch_order[0], 0);
    assert_eq!(*report.dispatch_order.last().unwrap(), 9);
}