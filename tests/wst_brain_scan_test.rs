use wst_brain_scan::{
    scan_ernie55_weight_space_time, verify_five_layer_bent_ternary, BalTrit, OneHotQubitPair,
};

#[test]
fn test_five_layer_bent_ternary_diff() {
    let diff = verify_five_layer_bent_ternary(19952026, 2, 14);

    assert_eq!(diff.agreed, 14);
    assert_eq!(diff.disagreed, 0);
    assert_eq!(diff.executable_layers, 4);
    assert_eq!(diff.declared_layers, 6);
    assert_eq!(diff.named_skips.len(), 4);
    assert!(diff.named_skips.iter().any(|skip| skip.layer == "ada_2022"));
    assert_eq!(diff.ada_2022_lane.standard, "Ada 2022 (ISO/IEC 8652:2023)");
    assert!(!diff.ada_2022_lane.executed);
    assert_eq!(diff.ada_2022_lane.expected_stage_b_bent_trits.len(), 14);
}

#[test]
fn test_bal_trit_and_one_hot_qubit_pair() {
    assert_eq!(BalTrit::Neg.rotate_cyclic(1), BalTrit::Zero);
    assert_eq!(BalTrit::Neg.rotate_cyclic(2), BalTrit::Pos);
    assert_eq!(BalTrit::Neg.ternary_nand(BalTrit::Pos), BalTrit::Pos);
    assert_eq!(BalTrit::Pos.ternary_nand(BalTrit::Pos), BalTrit::Neg);

    let pair = OneHotQubitPair::apply_bent_circuit(BalTrit::Neg, 2);
    assert_eq!(pair.measure_z_basis(), BalTrit::Pos);
}

#[test]
fn test_ernie55_wst_scan() {
    let scan = scan_ernie55_weight_space_time();

    assert_eq!(scan.model, "ERNIE-5.5-Flash-Base");
    assert_eq!(scan.base, "8b-is-demoscene");
    assert_eq!(scan.mla_full_token_layers, 6);
    assert_eq!(scan.hca_128x_compressed_layers, 37);
    assert_eq!(scan.ternary_packed_expert_row_bytes, 410);
    assert!(scan.kv_cache_compression_ratio > 6.0);
    assert!(scan.mhc_stream_energy_conservation);
    assert!(scan.critical_path_43_layer_us < scan.sequential_sum_43_layer_us);
    assert_eq!(scan.joint_scores.len(), 16);
    assert_eq!(scan.mhc_stream_out.len(), 4);
}
