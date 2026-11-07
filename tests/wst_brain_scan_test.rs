use wst_brain_scan::{scan_ernie55_weight_space_time, verify_five_layer_bent_ternary, BalTrit};

#[test]
fn test_kompress_super_ultra_ernie55_and_demoscene_5layer_diff() {
    // 1. Verify 5-implementation bent balanced-ternary diff (C <-> ASM <-> Rust <-> Felix <-> Qiskit)
    let diff = verify_five_layer_bent_ternary(1995_2026, 2, 14);
    assert_eq!(diff.agreed, 14);
    assert_eq!(diff.disagreed, 0);
    assert_eq!(diff.named_skips.len(), 4);
    assert_eq!(diff.executable_layers, 4);
    assert_eq!(diff.declared_layers, 6);

    // Verify deterministic 2-qubit one-hot Z-basis measurement never loses -1 (BalTrit::Neg)
    for (&expected_bent, &q_pair) in diff
        .stage_b_bent_trits
        .iter()
        .zip(diff.qiskit_one_hot_readout.iter())
    {
        assert_eq!(q_pair.measure_z_basis() as i8, expected_bent);
    }

    // Ada 2022 lane: declared, source-verified, execution is an explicit named skip
    assert_eq!(diff.ada_2022_lane.standard, "Ada 2022 (ISO/IEC 8652:2023)");
    assert!(!diff.ada_2022_lane.executed);
    assert!(diff.named_skips.iter().any(|skip| skip.layer == "ada_2022"));
    assert_eq!(
        diff.ada_2022_lane.expected_stage_b_bent_trits,
        diff.stage_b_bent_trits
    );

    // 2. Verify universal Ternary NAND completeness (-min(a, b))
    assert_eq!(BalTrit::Pos.ternary_nand(BalTrit::Pos), BalTrit::Neg);
    assert_eq!(BalTrit::Neg.ternary_nand(BalTrit::Pos), BalTrit::Pos);
    assert_eq!(BalTrit::Zero.ternary_nand(BalTrit::Zero), BalTrit::Zero);

    // 3. Verify ERNIE 5.5 Flash Base Weight-Space-Time (W-S-T) scan at 128K context
    let scan = scan_ernie55_weight_space_time(131_072, 512, 4);
    assert_eq!(scan.total_layers, 43);
    assert_eq!(scan.hidden_size, 4096);
    assert_eq!(scan.mhc_parallel_streams, 4);
    assert_eq!(scan.mla_full_token_layers, 6);
    assert_eq!(scan.hca_128x_compressed_layers, 37);
    assert_eq!(scan.latent_moe_active_experts, 11);
    assert_eq!(scan.latent_moe_total_experts, 513);
    assert_eq!(scan.ternary_packed_expert_row_bytes, 410); // ceil(2048 / 5) = 410 bytes
    assert!(
        scan.kv_cache_compression_ratio > 6.0,
        "1:6 MLA + 128x HCA must compress 128K KV cache entries by > 6x, got {}",
        scan.kv_cache_compression_ratio
    );
    assert!(scan.mhc_stream_energy_conservation);
    assert!(scan.critical_path_43_layer_us < scan.sequential_sum_43_layer_us);
}
