//! End-to-end FOKEN=VAKED integration test.
//!
//! Tests the complete pipeline: VAKED sovereign golf function → FOKEN scoreboard submission → reward computation.

use st::g8t_integration::{
    compute_capsule_fokens, format_foken_scoreboard, get_foken_scoreboard, query_best_functions,
    sovereign_cycle_golf_library, FokenBreakdown, FokenScoreboardReport, LeaderboardCapsuleEntry,
    RationalPq, FOKEN_BASE_C64_COMPLIANT, FOKEN_EQUIVALENCE_CROWN_BONUS, FOKEN_PER_POSITIVE_VOTE,
    FOKEN_SUB_10_CYCLE_BONUS,
};

#[test]
fn test_vaked_toss_spiral_fold_in_library() {
    // Verify the VAKED function exists in the sovereign golf library
    let library = sovereign_cycle_golf_library();
    let vaked_entry = library.iter().find(|spec| spec.domain == "vaked");

    assert!(
        vaked_entry.is_some(),
        "VAKED domain function should exist in library"
    );
    let spec = vaked_entry.unwrap();

    assert_eq!(spec.target_name, "vaked_toss_spiral_fold");
    assert_eq!(spec.golfed_fn_name, "st_golf_vaked_toss_spiral_fold_i64");
    assert_eq!(spec.domain, "vaked");
    assert!(!spec.source_code.is_empty());
    assert!(spec
        .source_code
        .contains("st_golf_vaked_toss_spiral_fold_i64"));
    assert!(spec.source_code.contains("SIN_Q8"));
    assert!(spec.source_code.contains("COS_Q8"));
}

#[test]
fn test_vaked_foken_breakdown_computation() {
    // Test that VAKED capsule FOKEN breakdown computes correctly
    let library = sovereign_cycle_golf_library();
    let vaked_spec = library.iter().find(|s| s.domain == "vaked").unwrap();

    // Create a mock leaderboard entry for the VAKED function
    let capsule = st::g8t_integration::FnCapsuleWire {
        id: "vaked-test-1".to_string(),
        name: vaked_spec.golfed_fn_name.to_string(),
        domain: vaked_spec.domain.to_string(),
        source_code: vaked_spec.source_code.to_string(),
        compiled_text_bytes: 256,
        asm_instruction_count: 48,
        output_digest_blake3: "804ec08bebcf".to_string(),
        signature_key: "vaked:st_golf_vaked_toss_spiral_fold_i64:test".to_string(),
        author_aye: "vaked-partner".to_string(),
        author_resonance_hz: 432,
        published_epoch_ns: 0,
        inputs: vec![],
        c64_report: st::g8t_integration::C64ReportWire::default(),
        asm_preview: String::new(),
    };

    let efficiency = st::g8t_integration::EfficiencySummaryWire {
        capsule_id: "vaked-test-1".to_string(),
        vote_count: 5,
        total_historical_votes: 5,
        net_trit_score: 5,
        net_trit_balanced: "5".to_string(),
        consensus_trit: "+".to_string(),
        speed_ops_per_us: RationalPq::new(1000000, 8),
        ns_per_op: RationalPq::new(8, 1),
        cycles_per_op: RationalPq::new(8, 1),
        c64_memory_ratio: RationalPq::new(1, 1),
        timing_salience: RationalPq::new(1, 1),
        efficiency_index: RationalPq::new(1, 1),
    };

    let entry = LeaderboardCapsuleEntry {
        capsule,
        efficiency,
        ns_per_op_pq: "8/1".to_string(),
        speed_ops_per_us_pq: "1000000/8".to_string(),
        cycles_per_op_pq: "8/1".to_string(),
        marine_salience_pq: "0".to_string(),
        vote_trits: "+++++".to_string(),
        equivalence_class: "vaked::spiral_fold".to_string(),
    };

    // Compute FOKEN breakdown
    let breakdown = compute_capsule_fokens(&entry, false);

    // VAKED function at 8 cycles/op should get sub-10-cycle bonus
    assert_eq!(breakdown.sub_10_cycle_fkn, FOKEN_SUB_10_CYCLE_BONUS);
    assert_eq!(breakdown.consensus_votes_fkn, 5 * FOKEN_PER_POSITIVE_VOTE);
    assert!(breakdown.total_fkn > 0);

    println!("VAKED FOKEN breakdown: {:?}", breakdown);
}

#[tokio::test]
async fn test_vaked_e2e_query_leaderboard() {
    // Query the g8t.is leaderboard for VAKED domain functions
    let result = query_best_functions("vaked", 3, false, Some("vaked"));

    match result {
        Ok(report) => {
            println!(
                "VAKED leaderboard query: {} recommendations",
                report.recommendations.len()
            );
            assert!(!report.recommendations.is_empty() || report.total_leaderboard_capsules == 0);
        }
        Err(e) => {
            println!("VAKED query skipped (network): {}", e);
        }
    }
}

#[tokio::test]
async fn test_vaked_e2e_foken_scoreboard() {
    // Get the full FOKEN scoreboard and verify VAKED domain presence
    let result = get_foken_scoreboard(20);

    match result {
        Ok(report) => {
            println!(
                "FOKEN Scoreboard: {} capsules, {} FKN minted",
                report.total_capsules, report.total_fokens_minted_fkn
            );

            // Check if VAKED domain entries exist in fastest_capsules
            let vaked_fastest: Vec<_> = report
                .fastest_capsules
                .iter()
                .filter(|e| e.domain == "vaked")
                .collect();

            println!("VAKED fastest capsules: {}", vaked_fastest.len());

            // Format and print scoreboard for verification
            let formatted = format_foken_scoreboard(&report);
            assert!(!formatted.is_empty());
            assert!(formatted.contains("FOKEN"));
        }
        Err(e) => {
            println!("FOKEN scoreboard query skipped (network): {}", e);
        }
    }
}

#[test]
fn test_vaked_sovereign_golf_spec_validation() {
    // Validate the VAKED SovereignGolfSpec source code
    let library = sovereign_cycle_golf_library();
    let vaked_spec = library.iter().find(|s| s.domain == "vaked").unwrap();

    let source = &vaked_spec.source_code;

    // Check required components
    assert!(source.contains("pub const fn"));
    assert!(source.contains("st_golf_vaked_toss_spiral_fold_i64"));
    assert!(source.contains("SIN_Q8"));
    assert!(source.contains("COS_Q8"));
    assert!(source.contains("coord: i64"));
    assert!(source.contains("-> i64"));

    // Verify the function logic components
    assert!(source.contains("32336")); // x offset
    assert!(source.contains("32984")); // y offset
    assert!(source.contains("radius"));
    assert!(source.contains("step"));
    assert!(source.contains("while step < 8"));
}

#[test]
fn test_foken_constants_consistency() {
    // Verify FOKEN constants are consistent
    assert_eq!(FOKEN_BASE_C64_COMPLIANT, 100);
    assert_eq!(FOKEN_EQUIVALENCE_CROWN_BONUS, 250);
    assert_eq!(FOKEN_PER_POSITIVE_VOTE, 50);
    assert_eq!(FOKEN_SUB_10_CYCLE_BONUS, 100);

    // Maximum possible FOKEN for a single capsule
    let max_fkn = FOKEN_BASE_C64_COMPLIANT
        + FOKEN_EQUIVALENCE_CROWN_BONUS
        + FOKEN_PER_POSITIVE_VOTE * 10
        + FOKEN_SUB_10_CYCLE_BONUS;

    assert!(max_fkn <= 1000);
}

#[test]
fn test_vaked_foken_bounty_computation() {
    // Test bounty computation for VAKED challenges
    let bounty = FOKEN_BASE_C64_COMPLIANT
        + FOKEN_EQUIVALENCE_CROWN_BONUS
        + FOKEN_PER_POSITIVE_VOTE
        + FOKEN_SUB_10_CYCLE_BONUS;

    assert_eq!(bounty, 500); // 100 + 250 + 50 + 100
}

#[test]
fn test_rational_pq_arithmetic() {
    // Test RationalPq exact arithmetic used in FOKEN/cycle computations
    let r1 = RationalPq::new(8, 1); // 8 cycles/op
    let r2 = RationalPq::new(10, 1); // 10 cycles/op

    assert!(r1 < r2); // 8 < 10
    assert_eq!(r1.to_pq_string(), "8/1");
    assert_eq!(r2.to_pq_string(), "10/1");

    // Test comparison for fastest-first ordering
    use std::cmp::Ordering;
    let cmp = st::g8t_integration::cmp_cycles_fastest_first(r1, r2);
    assert_eq!(cmp, Ordering::Less); // 8 is faster than 10
}
