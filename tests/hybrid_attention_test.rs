use st::mcp_combinatorics_llm::{
    hybrid_attention::{
        AttentionLayerKind, Ernie55HybridAttentionLayer, HcaCompressedKvCache, HybridKvStore,
        LowRankQueryGate, MlaDsaKvCache, VirtualHeadAttentionMixer, DEFAULT_DSA_INDEX_DIM,
        DEFAULT_VHA_RANK, ERNIE55_NUM_HEADS, ERNIE55_QUERY_LATENT_DIM, ERNIE55_TOTAL_LAYERS,
        HCA_COMPRESSION_BLOCK_SIZE, HCA_HEAD_DIM, MLA_HEAD_DIM, PACKED_QUERY_LATENT_ROW_BYTES,
    },
    latent_moe_layer::MHC_STREAMS,
    latent_moe_simd::{
        pack_latent_moe_matrix_par_simd, ERNIE55_HIDDEN_DIM, PACKED_HIDDEN_ROW_BYTES,
    },
};

#[test]
fn test_ernie55_hybrid_mla_256_dsa_and_hca_128x_512_attention_module() {
    let mla_count = (0..ERNIE55_TOTAL_LAYERS)
        .filter(|&l| AttentionLayerKind::for_layer(l) == AttentionLayerKind::MlaDsaSparse256)
        .count();
    let hca_count = (0..ERNIE55_TOTAL_LAYERS)
        .filter(|&l| AttentionLayerKind::for_layer(l) == AttentionLayerKind::Hca128xCompressed512)
        .count();

    assert_eq!(mla_count, 6);
    assert_eq!(hca_count, 37);
    assert_eq!(
        AttentionLayerKind::Hca128xCompressed512.head_dim(),
        HCA_HEAD_DIM
    );
    assert_eq!(AttentionLayerKind::MlaDsaSparse256.head_dim(), MLA_HEAD_DIM);

    assert_eq!(PACKED_QUERY_LATENT_ROW_BYTES, 205);
    assert_eq!(PACKED_HIDDEN_ROW_BYTES, 820);
    assert_eq!(PACKED_QUERY_LATENT_ROW_BYTES * 4, PACKED_HIDDEN_ROW_BYTES);

    let thresholds = (75u8, 180u8);
    let centroids = [20u8, 128u8, 236u8];
    let build_mat = |rows: usize, cols: usize, seed: usize| {
        let raw: Vec<u8> = (0..(rows * cols))
            .map(|i| ((i.wrapping_mul(43 + seed) + 17) & 0xFF) as u8)
            .collect();
        pack_latent_moe_matrix_par_simd(&raw, rows, cols, thresholds, centroids, 1, 0.01)
    };

    let hca_layer = Ernie55HybridAttentionLayer {
        layer_idx: 0,
        kind: AttentionLayerKind::Hca128xCompressed512,
        hidden_dim: ERNIE55_HIDDEN_DIM,
        num_heads: ERNIE55_NUM_HEADS,
        head_dim: HCA_HEAD_DIM,
        low_rank_query_gate: LowRankQueryGate {
            w_q_down: build_mat(ERNIE55_QUERY_LATENT_DIM, ERNIE55_HIDDEN_DIM, 1),
            w_gate_from_latent: build_mat(HCA_HEAD_DIM, ERNIE55_QUERY_LATENT_DIM, 2),
        },
        w_q_up: build_mat(HCA_HEAD_DIM, ERNIE55_QUERY_LATENT_DIM, 3),
        w_kv_down: build_mat(HCA_HEAD_DIM, ERNIE55_HIDDEN_DIM, 4),
        w_dsa_indexer: None,
        vha_mixer: VirtualHeadAttentionMixer::new_identity_plus_low_rank(
            ERNIE55_NUM_HEADS,
            DEFAULT_VHA_RANK,
            0.05,
        ),
        w_out_proj: build_mat(ERNIE55_HIDDEN_DIM, HCA_HEAD_DIM, 5),
        mhc_mixing_matrix: [
            [0.55, 0.15, 0.15, 0.15],
            [0.15, 0.55, 0.15, 0.15],
            [0.15, 0.15, 0.55, 0.15],
            [0.15, 0.15, 0.15, 0.55],
        ],
        mhc_read_weights: [0.25, 0.25, 0.25, 0.25],
        mhc_write_weights: [0.25, 0.25, 0.25, 0.25],
    };

    let mut hca_cache = HcaCompressedKvCache::new(HCA_COMPRESSION_BLOCK_SIZE, 16, HCA_HEAD_DIM);
    for t in 0..511 {
        let kv_vec: Vec<f32> = (0..HCA_HEAD_DIM)
            .map(|d| ((t + d) as f32 * 0.03).sin())
            .collect();
        hca_cache.push_token_kv(kv_vec);
    }
    assert_eq!(hca_cache.compressed_blocks.len(), 3);

    let mut hca_store = HybridKvStore::Hca(hca_cache);
    let mut streams: [Vec<f32>; MHC_STREAMS] = [
        (0..ERNIE55_HIDDEN_DIM)
            .map(|i| ((i as f32) * 0.01).cos())
            .collect(),
        (0..ERNIE55_HIDDEN_DIM)
            .map(|i| ((i as f32) * 0.02).sin())
            .collect(),
        (0..ERNIE55_HIDDEN_DIM)
            .map(|i| ((i as f32) * 0.03).cos())
            .collect(),
        (0..ERNIE55_HIDDEN_DIM)
            .map(|i| ((i as f32) * 0.04).sin())
            .collect(),
    ];

    let hca_tel = hca_layer.forward_mhc_step(&mut streams, &mut hca_store);
    assert_eq!(hca_tel.head_dim, 512);
    assert_eq!(hca_tel.total_tokens_in_history, 512);
    assert_eq!(hca_tel.attended_kv_entries, 4 + 16);
    assert!(
        (hca_tel.low_rank_gate_telemetry.weight_reduction_ratio_pct - 75.0).abs() < 1e-6,
        "Low-rank query gate must achieve exact 75.0% weight reduction"
    );
    assert!((hca_tel.mhc_l1_norm_before - hca_tel.mhc_l1_norm_after_mixing).abs() < 1e-2);

    let mla_layer = Ernie55HybridAttentionLayer {
        layer_idx: 6,
        kind: AttentionLayerKind::MlaDsaSparse256,
        hidden_dim: ERNIE55_HIDDEN_DIM,
        num_heads: ERNIE55_NUM_HEADS,
        head_dim: MLA_HEAD_DIM,
        low_rank_query_gate: LowRankQueryGate {
            w_q_down: build_mat(ERNIE55_QUERY_LATENT_DIM, ERNIE55_HIDDEN_DIM, 11),
            w_gate_from_latent: build_mat(MLA_HEAD_DIM, ERNIE55_QUERY_LATENT_DIM, 12),
        },
        w_q_up: build_mat(MLA_HEAD_DIM, ERNIE55_QUERY_LATENT_DIM, 13),
        w_kv_down: build_mat(MLA_HEAD_DIM, ERNIE55_HIDDEN_DIM, 14),
        w_dsa_indexer: Some(build_mat(
            DEFAULT_DSA_INDEX_DIM,
            ERNIE55_QUERY_LATENT_DIM,
            15,
        )),
        vha_mixer: VirtualHeadAttentionMixer::new_identity_plus_low_rank(
            ERNIE55_NUM_HEADS,
            DEFAULT_VHA_RANK,
            0.05,
        ),
        w_out_proj: build_mat(ERNIE55_HIDDEN_DIM, MLA_HEAD_DIM, 16),
        mhc_mixing_matrix: hca_layer.mhc_mixing_matrix,
        mhc_read_weights: [0.25, 0.25, 0.25, 0.25],
        mhc_write_weights: [0.25, 0.25, 0.25, 0.25],
    };

    let mut mla_cache = MlaDsaKvCache::new(DEFAULT_DSA_INDEX_DIM, MLA_HEAD_DIM, 16);
    for t in 0..255 {
        let idx_k: Vec<f32> = (0..DEFAULT_DSA_INDEX_DIM)
            .map(|d| ((t * 3 + d) as f32 * 0.05).cos())
            .collect();
        let kv_v: Vec<f32> = (0..MLA_HEAD_DIM)
            .map(|d| ((t + d) as f32 * 0.02).sin())
            .collect();
        mla_cache.push_token(idx_k, kv_v);
    }

    let mut mla_store = HybridKvStore::MlaDsa(mla_cache);
    let mla_tel = mla_layer.forward_mhc_step(&mut streams, &mut mla_store);

    assert_eq!(mla_tel.head_dim, 256);
    assert_eq!(mla_tel.total_tokens_in_history, 256);
    assert_eq!(mla_tel.attended_kv_entries, 16);
    assert_eq!(mla_tel.dsa_selected_positions.len(), 16);
    assert!((mla_tel.kv_compression_or_sparsity_ratio - 16.0).abs() < 1e-6);
}
