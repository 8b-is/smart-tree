//! Dynamic Slack Priority Rayon DAG Scheduler
//!
//! This module provides a Rayon-based DAG executor with dynamic runtime slack
//   recalculation and deadline-breach escalation for adaptive task dispatch.
//!
//! Key features:
//! - Live forward/backward slack recalculation on every task completion
//! - Zero-slack escalation when side branches threaten the critical path
//! - Overrun deficit tie-breaking among zero-slack tasks
//! - Escalation audit logging with `SlackEscalationEvent` records
//! - Full execution trace with `TaskExecutionTrace` records
//! - Causal dependency verification

use crate::{
    g8t_integration::RationalPq,
    ultra_zeno::{UltraZenoParams, run_ultra_zeno_engine},
    mcp::combinatorics_llm::{
        LlamaModelCache, McpProgressEmitter, ErdosRenyiParams, ErdosSteinerParams,
        MoserTardosParams, RamseyCliqueParams, SzemerediParams,
        run_erdos_renyi_scanning, run_erdos_steiner_pruning, run_moser_tardos_splitting,
        run_ramsey_clique_analysis, run_szemeredi_compression, run_llama_rust,
        LlamaRustParams, MlxQuantParams,
    },
    ternary_deck::{evaluate_prerequisite_deck, evaluate_ternary_lane, StudyCardNode, CardCompletionMetrics, PrerequisiteDeckReport},
};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::{
    cmp::Ordering,
    collections::{BinaryHeap, VecDeque},
    sync::{
        atomic::{AtomicUsize, Ordering as AtomicOrdering},
        Condvar, Mutex,
    },
    time::{Duration, Instant},
};

/// Default conversion scale from wall-clock nanoseconds to abstract schedule duration units:
/// `1_000_000 ns = 1 ms` per schedule unit `d(v)`.
pub const DEFAULT_NS_PER_SCHEDULE_UNIT: u64 = 1_000_000;

// ============================================================================
// 1. Dynamic Slack Configuration & Priority Ready-Queue Key
// ============================================================================

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub struct DynamicSlackConfig {
    /// Nanoseconds corresponding to `1` abstract duration unit `d(v)`.
    pub ns_per_schedule_unit: u64,
    /// When true, any side branch whose projected completion `\hat{C}(v)` reaches or exceeds
    /// its static latest completion `L_0(v)` immediately escalates to `dynamic_slack = 0`.
    pub escalate_on_nominal_deadline_breach: bool,
}

impl Default for DynamicSlackConfig {
    fn default() -> Self {
        Self {
            ns_per_schedule_unit: DEFAULT_NS_PER_SCHEDULE_UNIT,
            escalate_on_nominal_deadline_breach: true,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct PrioritizedReadyTask {
    pub id: usize,
    pub initial_slack: u64,
    pub dynamic_slack: u64,
    pub overrun_deficit: u64,
    pub earliest_start: u64,
    pub dynamic_earliest_start: u64,
    pub estimated_duration: u64,
    pub on_critical_path: bool,
    pub escalated_to_zero_slack: bool,
}

impl Ord for PrioritizedReadyTask {
    fn cmp(&self, other: &Self) -> Ordering {
        // 1. Lowest dynamic slack first (0 = critical spine or escalated overrun branch)
        other
            .dynamic_slack
            .cmp(&self.dynamic_slack)
            // 2. Highest deadline overrun deficit first among zero-slack tasks
            .then_with(|| self.overrun_deficit.cmp(&other.overrun_deficit))
            // 3. Earliest nominal topological wave first
            .then_with(|| other.earliest_start.cmp(&self.earliest_start))
            // 4. Longest Processing Time (LPT) first on ties
            .then_with(|| self.estimated_duration.cmp(&other.estimated_duration))
            // 5. Deterministic lowest node ID tie-breaker
            .then_with(|| other.id.cmp(&self.id))
    }
}

impl PartialOrd for PrioritizedReadyTask {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

// ============================================================================
// 2. Dynamic Slack Escalation Audit Events & Execution Telemetry
// ============================================================================

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SlackEscalationEvent {
    pub node_id: usize,
    pub title: String,
    pub initial_slack: u64,
    pub trigger_wall_units: u64,
    pub dynamic_earliest_start_units: u64,
    pub projected_completion_units: u64,
    pub nominal_latest_completion_l0: u64,
    pub overrun_deficit_units: u64,
}

#[derive(Clone, Debug, Serialize)]
pub struct TaskExecutionTrace {
    pub id: usize,
    pub title: String,
    /// Static slack S_0(v) from the initial prerequisite analysis.
    pub slack: u64,
    /// Live dynamic slack S_dyn(v, t_dispatch) at the moment a Rayon worker popped this task.
    pub dynamic_slack_at_dispatch: u64,
    /// Deadline overrun deficit max(0, \hat{C}(v) - L_0(v)) at dispatch time.
    pub overrun_deficit_at_dispatch: u64,
    pub earliest_start: u64,
    pub dynamic_earliest_start: u64,
    pub earliest_completion_c: u64,
    pub on_critical_path: bool,
    /// True if S_0(v) > 0 initially, but wall-clock overrun/delay eroded S_dyn(v) to 0.
    pub escalated_to_zero_slack: bool,
    pub dispatch_order_rank: usize,
    pub rayon_worker_index: Option<usize>,
    pub start_timestamp_ns: u64,
    pub finish_timestamp_ns: u64,
    pub measured_duration_ns: u64,
}

#[derive(Clone, Debug, Serialize)]
pub struct DagScheduleExecutionReport<T> {
    pub prerequisite_analysis: PrerequisiteDeckReport,
    pub dispatch_order: Vec<usize>,
    pub total_wall_time_ns: u64,
    pub sum_task_durations_ns: u64,
    pub empirical_parallel_speedup: f64,
    pub dynamic_critical_path_duration: u64,
    pub slack_escalations_triggered: usize,
    pub escalation_events: Vec<SlackEscalationEvent>,
    pub causal_dependencies_satisfied: bool,
    pub task_traces: Vec<TaskExecutionTrace>,
    pub outputs: Vec<T>,
}

// ============================================================================
// 3. Internal Dynamic Forward/Backward Graph State & Re-Heapifier
// ============================================================================

#[derive(Clone, Copy, Debug)]
enum NodeLifecycle {
    Pending,
    Running {
        start_units: u64,
    },
    Completed {
        start_units: u64,
        finish_units: u64,
    },
}

struct SharedQueueState {
    ready_heap: BinaryHeap<PrioritizedReadyTask>,
    node_lifecycle: Vec<NodeLifecycle>,
    dynamic_earliest_start: Vec<u64>,
    dynamic_earliest_completion: Vec<u64>,
    dynamic_latest_completion: Vec<u64>,
    dynamic_slack: Vec<u64>,
    overrun_deficit: Vec<u64>,
    escalated_flags: Vec<bool>,
    escalation_events: Vec<SlackEscalationEvent>,
    dynamic_makespan_units: u64,
    last_recalc_units: u64,
    completed_count: usize,
    aborted_error: Option<String>,
}

impl SharedQueueState {
    /// Recomputes live earliest start `\hat{E}(v)`, projected completion `\hat{C}(v)`,
    /// dynamic latest completion `\hat{L}(v)`, effective dynamic slack `S_dyn(v)`,
    /// and deadline overrun deficit `\delta(v)` across the DAG at wall-clock time `now_units`,
    /// then rebuilds `self.ready_heap` so newly escalated zero-slack branches preempt waiting tasks.
    fn recalculate_dynamic_slack_and_reheapify(
        &mut self,
        now_units: u64,
        topo_order: &[usize],
        cards: &[StudyCardNode],
        successors: &[Vec<usize>],
        static_schedule: &[CardCompletionMetrics],
        nominal_critical_path: u64,
        config: &DynamicSlackConfig,
    ) {
        self.last_recalc_units = now_units;

        // 1. Forward Topological Pass: propagate actual completions and wall-clock `now_units`
        for &u in topo_order {
            let d_u = static_schedule[u].duration;
            match self.node_lifecycle[u] {
                NodeLifecycle::Completed {
                    start_units,
                    finish_units,
                } => {
                    self.dynamic_earliest_start[u] = start_units;
                    self.dynamic_earliest_completion[u] = finish_units;
                }
                NodeLifecycle::Running { start_units } => {
                    self.dynamic_earliest_start[u] = start_units;
                    self.dynamic_earliest_completion[u] =
                        now_units.max(start_units.saturating_add(d_u));
                }
                NodeLifecycle::Pending => {
                    let pred_max = cards[u]
                        .predecessors
                        .iter()
                        .map(|&p| self.dynamic_earliest_completion[p])
                        .max()
                        .unwrap_or(0);
                    // An unstarted task cannot begin earlier than `now_units` or its slowest predecessor
                    let e_dyn = now_units.max(pred_max);
                    self.dynamic_earliest_start[u] = e_dyn;
                    self.dynamic_earliest_completion[u] = e_dyn.saturating_add(d_u);
                }
            }
        }

        // 2. Dynamic Makespan & Reverse Topological Pass for `\hat{L}(u)`
        let projected_max_c = self
            .dynamic_earliest_completion
            .iter()
            .copied()
            .max()
            .unwrap_or(nominal_critical_path);
        let dyn_makespan = nominal_critical_path.max(projected_max_c);
        self.dynamic_makespan_units = dyn_makespan;

        self.dynamic_latest_completion.fill(dyn_makespan);
        for &u in topo_order.iter().rev() {
            if !successors[u].is_empty() {
                let min_succ_latest_start = successors[u]
                    .iter()
                    .map(|&w| {
                        let rem_d = self.dynamic_earliest_completion[w]
                            .saturating_sub(self.dynamic_earliest_start[w]);
                        self.dynamic_latest_completion[w].saturating_sub(rem_d)
                    })
                    .min()
                    .unwrap_or(dyn_makespan);
                self.dynamic_latest_completion[u] = min_succ_latest_start;
            }
        }

        // 3. Evaluate Effective Dynamic Slack & Record Zero-Slack Escalations
        for &v in topo_order {
            if !matches!(self.node_lifecycle[v], NodeLifecycle::Pending) {
                continue;
            }

            let c_hat = self.dynamic_earliest_completion[v];
            let l0 = static_schedule[v].latest_completion;
            let l_hat = self.dynamic_latest_completion[v];

            let slack_vs_Shift = l_hat.saturating_sub(c_hat);
            let slack_vs_nominal = l0.saturating_sub(c_hat);

            let eff_slack = if config.escalate_on_nominal_deadline_breach {
                slack_vs_nominal.min(slack_vs_Shift)
            } else {
                // When escalation is disabled, keep dynamic slack at static value
                static_schedule[v].slack
            };

            let deficit = c_hat.saturating_sub(l0);
            self.dynamic_slack[v] = eff_slack;
            self.overrun_deficit[v] = deficit;

            let s0 = static_schedule[v].slack;
            if config.escalate_on_nominal_deadline_breach && s0 > 0 && eff_slack == 0 && !self.escalated_flags[v] {
                self.escalated_flags[v] = true;
                self.escalation_events.push(SlackEscalationEvent {
                    node_id: v,
                    title: static_schedule[v].title.clone(),
                    initial_slack: s0,
                    trigger_wall_units: now_units,
                    dynamic_earliest_start_units: self.dynamic_earliest_start[v],
                    projected_completion_units: c_hat,
                    nominal_latest_completion_l0: l0,
                    overrun_deficit_units: deficit,
                });
            }
        }

        // 4. Rebuild `self.ready_heap` in O(K) so escalated tasks jump to the front
        if self.ready_heap.len() >= 1 {
            let mut updated_tasks = Vec::with_capacity(self.ready_heap.len());
            for mut task in self.ready_heap.drain() {
                let v = task.id;
                task.dynamic_slack = self.dynamic_slack[v];
                task.overrun_deficit = self.overrun_deficit[v];
                task.dynamic_earliest_start = self.dynamic_earliest_start[v];
                task.escalated_to_zero_slack = self.escalated_flags[v];
                updated_tasks.push(task);
            }
            self.ready_heap = BinaryHeap::from(updated_tasks);
        }
    }
}

// ============================================================================
// 4. Public Priority-Weighted Rayon DAG Executors
// ============================================================================

/// Executes a DAG of `cards` across `max_workers` Rayon threads using default dynamic slack
/// recalculation (`1 schedule unit = 1 ms` wall-clock time).
pub fn execute_dag_priority_rayon<T, F>(
    cards: &[StudyCardNode],
    max_workers: usize,
    emitter: &McpProgressEmitter,
    run_node: F,
) -> Result<DagScheduleExecutionReport<T>, String>
where
    T: Send + Clone + Default,
    F: Fn(usize, &CardCompletionMetrics) -> Result<T, String> + Sync,
{
    execute_dag_priority_rayon_with_config(
        cards,
        max_workers,
        DynamicSlackConfig::default(),
        emitter,
        run_node,
    )
}

/// Executes a DAG of `cards` across `max_workers` Rayon threads with configurable wall-clock
/// schedule unit scaling (`DynamicSlackConfig`) and automatic zero-slack escalation when
/// delayed side branches threaten the critical path.
pub fn execute_dag_priority_rayon_with_config<T, F>(
    cards: &[StudyCardNode],
    max_workers: usize,
    config: DynamicSlackConfig,
    emitter: &McpProgressEmitter,
    run_node: F,
) -> Result<DagScheduleExecutionReport<T>, String>
where
    T: Send + Clone + Default,
    F: Fn(usize, &CardCompletionMetrics) -> Result<T, String> + Sync,
{
    let deck_report = evaluate_prerequisite_deck(cards)?;
    let n = cards.len();
    if n == 0 {
        return Ok(DagScheduleExecutionReport {
            prerequisite_analysis: deck_report,
            dispatch_order: Vec::new(),
            total_wall_time_ns: 0,
            sum_task_durations_ns: 0,
            empirical_parallel_speedup: 1.0,
            dynamic_critical_path_duration: 0,
            slack_escalations_triggered: 0,
            escalation_events: Vec::new(),
            causal_dependencies_satisfied: true,
            task_traces: Vec::new(),
            outputs: Vec::new(),
        });
    }

    let mut successors: Vec<Vec<usize>> = vec![Vec::new(); n];
    let mut in_deg = vec![0usize; n];
    let remaining_preds: Vec<AtomicUsize> = (0..n)
        .map(|v| AtomicUsize::new(cards[v].predecessors.len()))
        .collect();

    for card in cards {
        in_deg[card.id] = card.predecessors.len();
        for &u in &card.predecessors {
            successors[u].push(card.id);
        }
    }

    // Compute topological order for O(V + E) dynamic slack forward/backward sweeps
    let mut topo_queue: VecDeque<usize> = (0..n).filter(|&v| in_deg[v] == 0).collect();
    let mut topo_order = Vec::with_capacity(n);
    while let Some(u) = topo_queue.pop_front() {
        topo_order.push(u);
        for &w in &successors[u] {
            in_deg[w] -= 1;
            if in_deg[w] == 0 {
                topo_queue.push_back(w);
            }
        }
    }

    let mut initial_heap = BinaryHeap::new();
    for metric in &deck_report.card_schedule {
        if cards[metric.id].predecessors.is_empty() {
            initial_heap.push(PrioritizedReadyTask {
                id: metric.id,
                initial_slack: metric.slack,
                dynamic_slack: metric.slack,
                overrun_deficit: 0,
                earliest_start: metric.earliest_start,
                dynamic_earliest_start: metric.earliest_start,
                estimated_duration: metric.duration,
                on_critical_path: metric.on_critical_path,
                escalated_to_zero_slack: false,
            });
        }
    }

    let nominal_crit = deck_report.critical_path_duration;
    let queue_mutex = Mutex::new(SharedQueueState {
        ready_heap: initial_heap,
        node_lifecycle: vec![NodeLifecycle::Pending; n],
        dynamic_earliest_start: deck_report
            .card_schedule
            .iter()
            .map(|m| m.earliest_start)
            .collect(),
        dynamic_earliest_completion: deck_report
            .card_schedule
            .iter()
            .map(|m| m.earliest_completion_c)
            .collect(),
        dynamic_latest_completion: deck_report
            .card_schedule
            .iter()
            .map(|m| m.latest_completion)
            .collect(),
        dynamic_slack: deck_report.card_schedule.iter().map(|m| m.slack).collect(),
        overrun_deficit: vec![0; n],
        escalated_flags: vec![false; n],
        escalation_events: Vec::new(),
        dynamic_makespan_units: nominal_crit,
        last_recalc_units: 0,
        completed_count: 0,
        aborted_error: None,
    });
    let queue_cv = Condvar::new();

    let dispatch_counter = AtomicUsize::new(0);
    let results_mutex: Mutex<Vec<Option<T>>> = Mutex::new(vec![None; n]);
    let traces_mutex: Mutex<Vec<Option<TaskExecutionTrace>>> = Mutex::new(vec![None; n]);
    let dispatch_order_mutex: Mutex<Vec<usize>> = Mutex::new(Vec::with_capacity(n));

    let num_workers = max_workers
        .max(1)
        .min(n)
        .min(rayon::current_num_threads().max(1));
    let unit_ns = config.ns_per_schedule_unit.max(1);
    let epoch = Instant::now();

    rayon::scope(|s| {
        for _worker_slot in 0..num_workers {
            s.spawn(|_| loop {
                // 1. Refresh dynamic slack if wall-clock units advanced, then pop highest-priority task
                let (next_task, start_ns, start_units) = {
                    let mut guard = queue_mutex.lock().unwrap();
                    loop {
                        if guard.aborted_error.is_some() || guard.completed_count >= n {
                            return;
                        }
                        if !guard.ready_heap.is_empty() {
                            let now_ns = epoch.elapsed().as_nanos() as u64;
                            let now_units = now_ns / unit_ns;
                            if now_units > guard.last_recalc_units {
                                guard.recalculate_dynamic_slack_and_reheapify(
                                    now_units,
                                    &topo_order,
                                    cards,
                                    &successors,
                                    &deck_report.card_schedule,
                                    nominal_crit,
                                    &config,
                                );
                            }
                            if let Some(task) = guard.ready_heap.pop() {
                                guard.node_lifecycle[task.id] =
                                    NodeLifecycle::Running { start_units: now_units };
                                break (task, now_ns, now_units);
                            }
                        }
                        let (next_guard, _) = queue_cv
                            .wait_timeout(guard, Duration::from_millis(2))
                            .unwrap();
                        guard = next_guard;
                    }
                };

                let v = next_task.id;
                let static_metric = &deck_report.card_schedule[v];
                // Construct live view of `CardCompletionMetrics` reflecting dynamic escalation
                let live_metric = CardCompletionMetrics {
                    id: v,
                    title: static_metric.title.clone(),
                    duration: static_metric.duration,
                    earliest_start: next_task.dynamic_earliest_start,
                    earliest_completion_c: next_task
                        .dynamic_earliest_start
                        .saturating_add(static_metric.duration),
                    latest_completion: static_metric.latest_completion,
                    slack: next_task.dynamic_slack,
                    on_critical_path: static_metric.on_critical_path
                        || next_task.escalated_to_zero_slack,
                };

                let rank = dispatch_counter.fetch_add(1, AtomicOrdering::SeqCst);
                if let Ok(mut ord) = dispatch_order_mutex.lock() {
                    ord.push(v);
                }

                let worker_idx = rayon::current_thread_index();

                // 2. Run node compute closure outside the mutex
                let node_res = run_node(v, &live_metric);
                let finish_ns = epoch.elapsed().as_nanos() as u64;
                let finish_units = (finish_ns / unit_ns).max(start_units);

                match node_res {
                    Ok(output_val) => {
                        if let Ok(mut res_guard) = results_mutex.lock() {
                            res_guard[v] = Some(output_val);
                        }
                        if let Ok(mut tr_guard) = traces_mutex.lock() {
                            tr_guard[v] = Some(TaskExecutionTrace {
                                id: v,
                                title: static_metric.title.clone(),
                                slack: next_task.initial_slack,
                                dynamic_slack_at_dispatch: next_task.dynamic_slack,
                                overrun_deficit_at_dispatch: next_task.overrun_deficit,
                                earliest_start: static_metric.earliest_start,
                                dynamic_earliest_start: next_task.dynamic_earliest_start,
                                earliest_completion_c: static_metric.earliest_completion_c,
                                on_critical_path: static_metric.on_critical_path,
                                escalated_to_zero_slack: next_task.escalated_to_zero_slack,
                                dispatch_order_rank: rank,
                                rayon_worker_index: worker_idx,
                                start_timestamp_ns: start_ns,
                                finish_timestamp_ns: finish_ns,
                                measured_duration_ns: finish_ns.saturating_sub(start_ns),
                            });
                        }

                        // 3. Unlock successors whose predecessor counter reaches 0
                        let mut newly_ready = Vec::new();
                        for &succ in &successors[v] {
                            let prev =
                                remaining_preds[succ].fetch_sub(1, AtomicOrdering::AcqRel);
                            if prev == 1 {
                                let sm = &deck_report.card_schedule[succ];
                                newly_ready.push(PrioritizedReadyTask {
                                    id: succ,
                                    initial_slack: sm.slack,
                                    dynamic_slack: sm.slack,
                                    overrun_deficit: 0,
                                    earliest_start: sm.earliest_start,
                                    dynamic_earliest_start: sm.earliest_start,
                                    estimated_duration: sm.duration,
                                    on_critical_path: sm.on_critical_path,
                                    escalated_to_zero_slack: false,
                                });
                            }
                        }

                        // 4. Commit completion, push unlocked tasks, recalculate dynamic slack, and notify
                        let (completed_now, escalations_so_far) = {
                            let mut guard = queue_mutex.lock().unwrap();
                            guard.node_lifecycle[v] = NodeLifecycle::Completed {
                                start_units,
                                finish_units,
                            };
                            for item in newly_ready {
                                guard.ready_heap.push(item);
                            }
                            guard.recalculate_dynamic_slack_and_reheapify(
                                finish_units,
                                &topo_order,
                                cards,
                                &successors,
                                &deck_report.card_schedule,
                                nominal_crit,
                                &config,
                            );
                            guard.completed_count += 1;
                            let done = guard.completed_count;
                            let esc = guard.escalation_events.len();
                            queue_cv.notify_all();
                            (done, esc)
                        };

                        let _ = emitter.emit_progress(
                            completed_now,
                            n,
                            json!({
                                "scheduler": "dynamic_slack_priority_rayon_dag",
                                "completed_node_id": v,
                                "title": static_metric.title,
                                "initial_slack": next_task.initial_slack,
                                "dynamic_slack_at_dispatch": next_task.dynamic_slack,
                                "escalated_to_zero_slack": next_task.escalated_to_zero_slack,
                                "total_escalations": escalations_so_far,
                                "dispatch_rank": rank
                            }),
                        );
                    }
                    Err(err_msg) => {
                        let mut guard = queue_mutex.lock().unwrap();
                        if guard.aborted_error.is_none() {
                            guard.aborted_error = Some(err_msg);
                        }
                        queue_cv.notify_all();
                        return;
                    }
                }
            });
        }
    });

    let total_wall_ns = epoch.elapsed().as_nanos() as u64;
    let final_state = queue_mutex.into_inner().unwrap();

    if let Some(err) = final_state.aborted_error {
        return Err(err);
    }

    let outputs: Vec<T> = results_mutex
        .into_inner()
        .unwrap()
        .into_iter()
        .map(|opt| opt.unwrap_or_default())
        .collect();

    let traces: Vec<TaskExecutionTrace> = traces_mutex
        .into_inner()
        .unwrap()
        .into_iter()
        .enumerate()
        .map(|(idx, opt)| {
            opt.unwrap_or(TaskExecutionTrace {
                id: idx,
                title: cards[idx].title.clone(),
                slack: 0,
                dynamic_slack_at_dispatch: 0,
                overrun_deficit_at_dispatch: 0,
                earliest_start: 0,
                dynamic_earliest_start: 0,
                earliest_completion_c: 0,
                on_critical_path: false,
                escalated_to_zero_slack: false,
                dispatch_order_rank: idx,
                rayon_worker_index: None,
                start_timestamp_ns: 0,
                finish_timestamp_ns: 0,
                measured_duration_ns: 0,
            })
        })
        .collect();

    let dispatch_order = dispatch_order_mutex.into_inner().unwrap();

    let mut causal_ok = true;
    for card in cards {
        let v_start = traces[card.id].start_timestamp_ns;
        for &u in &card.predecessors {
            if traces[u].finish_timestamp_ns > v_start {
                causal_ok = false;
            }
        }
    }

    let sum_task_ns: u64 = traces.iter().map(|t| t.measured_duration_ns).sum();
    let speedup = if total_wall_ns > 0 {
        (sum_task_ns as f64) / (total_wall_ns as f64)
    } else {
        1.0
    };

    let num_escalations = final_state.escalation_events.len();

    Ok(DagScheduleExecutionReport {
        prerequisite_analysis: deck_report,
        dispatch_order,
        total_wall_time_ns: total_wall_ns,
        sum_task_durations_ns: sum_task_ns,
        empirical_parallel_speedup: speedup,
        dynamic_critical_path_duration: final_state.dynamic_makespan_units,
        slack_escalations_triggered: num_escalations,
        escalation_events: final_state.escalation_events,
        causal_dependencies_satisfied: causal_ok,
        task_traces: traces,
        outputs,
    })
}

// ============================================================================
// 5. Multi-Stage MCP Pipeline Orchestrator Driven by Dynamic Slack
// ============================================================================

#[derive(Debug, Clone, Deserialize)]
pub struct PriorityDagPipelineParams {
    #[serde(default = "default_graph_vertices")]
    pub num_vertices: usize,
    #[serde(default = "default_edge_prob")]
    pub edge_probability: f64,
    #[serde(default = "default_max_workers")]
    pub max_workers: usize,
    #[serde(default = "default_ns_per_unit")]
    pub ns_per_schedule_unit: u64,
    #[serde(default = "default_seed")]
    pub seed: u64,
}

fn default_graph_vertices() -> usize { 96 }
fn default_edge_prob() -> f64 { 0.14 }
fn default_max_workers() -> usize { 4 }
fn default_ns_per_unit() -> u64 { DEFAULT_NS_PER_SCHEDULE_UNIT }
fn default_seed() -> u64 { 0x7C24_2080_DA60_0001 }

pub fn run_priority_dag_mcp_pipeline(
    params: PriorityDagPipelineParams,
    emitter: &McpProgressEmitter,
) -> Result<Value, String> {
    let n = params.num_vertices.clamp(16, 256);

    let cards = vec![
        StudyCardNode {
            id: 0,
            title: "Stage 0 [M1]: Erdős-Rényi Percolation Genesis".into(),
            duration: 15,
            predecessors: vec![],
        },
        StudyCardNode {
            id: 1,
            title: "Stage 1 [SIMD]: 4K Ternary 4-Trit & 5-Trit Lane Verification".into(),
            duration: 10,
            predecessors: vec![],
        },
        StudyCardNode {
            id: 2,
            title: "Stage 2 [Ada/SPARK]: EmbeddedQuantPeter 2048-d Fused GEMV".into(),
            duration: 12,
            predecessors: vec![],
        },
        StudyCardNode {
            id: 3,
            title: "Stage 3 [M2 Critical]: Erdős-Steiner Mycorrhizal Backbone Pruning".into(),
            duration: 30,
            predecessors: vec![0],
        },
        StudyCardNode {
            id: 4,
            title: "Stage 4 [M2 Branch]: Szemerédi Regularity Cortical Partition".into(),
            duration: 14,
            predecessors: vec![0],
        },
        StudyCardNode {
            id: 5,
            title: "Stage 5 [M3 Critical]: Ramsey Clique & Moser-Tardos LLL Equilibrium".into(),
            duration: 25,
            predecessors: vec![3, 4],
        },
        StudyCardNode {
            id: 6,
            title: "Stage 6 [Sink]: ERNIE 5.5 + Jury PID Speculative Consensus".into(),
            duration: 20,
            predecessors: vec![1, 2, 5],
        },
    ];

    let mut undirected = Vec::new();
    let mut weighted = Vec::new();
    let mut s = params.seed;
    for u in 0..n {
        for v in (u + 1)..n {
            s = s.wrapping_add(0x9E37_79B9_7F4A_7C15);
            let mut z = s;
            z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
            z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
            z ^= z >> 31;
            let r01 = (z >> 11) as f64 * (1.0 / ((1u64 << 53) as f64));
            if r01 < params.edge_probability {
                undirected.push((u, v));
                weighted.push((u, v, 0.25 + 0.75 * r01));
            }
        }
    }

    let schedule_report = execute_dag_priority_rayon_with_config(
        &cards,
        params.max_workers,
        DynamicSlackConfig {
            ns_per_schedule_unit: params.ns_per_schedule_unit.max(1),
            escalate_on_nominal_deadline_breach: true,
        },
        emitter,
        |node_id, metric| -> Result<Value, String> {
            let stage_out = match node_id {
                0 => run_erdos_renyi_scanning(
                    ErdosRenyiParams {
                        n,
                        p: params.edge_probability,
                        seed: Some(params.seed),
                    },
                    &McpProgressEmitter::default(),
                )?,
                1 => serde_json::to_value(evaluate_ternary_lane(3840, 2160))
                    .map_err(|e| e.to_string())?,
                2 => {
                    // Mock Ada verification result for the pipeline
                    serde_json::to_value(json!({
                        "verified": true,
                        "seed": params.seed as i64,
                        "lanes": 2
                    }))
                    .map_err(|e| e.to_string())?
                }
                3 => {
                    let terminals: Vec<usize> = (0..n).step_by((n / 8).max(1)).collect();
                    run_erdos_steiner_pruning(
                        ErdosSteinerParams {
                            edges: weighted.clone(),
                            terminals,
                            pruning_threshold: 1.0,
                        },
                        &McpProgressEmitter::default(),
                    )?
                }
                4 => run_szemeredi_compression(
                    SzemerediParams {
                        num_vertices: n,
                        edges: undirected.clone(),
                        epsilon: 0.16,
                        min_parts: 4,
                    },
                    &McpProgressEmitter::default(),
                )?,
                5 => {
                    let ramsey = run_ramsey_clique_analysis(
                        RamseyCliqueParams {
                            num_vertices: n,
                            edges: undirected.clone(),
                            k_clique: 4,
                            l_independent: 4,
                        },
                        &McpProgressEmitter::default(),
                    )?;
                    let clauses: Vec<Vec<i32>> = (0..(n / 2).max(4))
                        .map(|i| {
                            vec![
                                ((i * 3) % n + 1) as i32,
                                -(((i * 5 + 1) % n + 1) as i32),
                                ((i * 7 + 2) % n + 1) as i32,
                            ]
                        })
                        .collect();
                    let lll = run_moser_tardos_splitting(
                        MoserTardosParams {
                            num_variables: n,
                            bad_events: clauses,
                            max_resamples: 500,
                        },
                        &McpProgressEmitter::default(),
                    )?;
                    json!({ "ramsey": ramsey, "moser_tardos": lll })
                }
                6 => {
                    let mut pid = crate::mcp::combinatorics_llm::AdaptiveSpeculativeController::new(4, 2);
                    pid.record_verification_outcome(4, 3, 5);
                    serde_json::to_value(pid.stats()).map_err(|e| e.to_string())?
                }
                _ => json!({}),
            };

            Ok(json!({
                "node_id": node_id,
                "dynamic_slack": metric.slack,
                "dynamic_earliest_start": metric.earliest_start,
                "on_critical_path": metric.on_critical_path,
                "payload": stage_out
            }))
        },
    )?;

    serde_json::to_value(schedule_report).map_err(|e| e.to_string())
}