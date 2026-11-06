use st::mcp_combinatorics_llm::{
    latent_moe_layer::{
        Ernie55LatentMoeLayer, LatentExpert5Trit, MultiRouterGate, DEFAULT_MULTI_ROUTERS,
        ERNIE55_TOP_K_EXPERTS, MHC_STREAMS,
    },
    latent_moe_simd::{
        pack_latent_moe_matrix_par_simd, ERNIE55_HIDDEN_DIM, ERNIE55_LATENT_DIM,
        PACKED_HIDDEN_ROW_BYTES, PACKED_LATENT_ROW_BYTES,
    },
};

#[test]
fn test_ernie55_multi_router_stable_latent_moe_full_4096_2048_4096_forward() {
    let d_model = ERNIE55_HIDDEN_DIM; // 4096
    let d_latent = ERNIE55_LATENT_DIM; // 2048
    let num_routed_experts = 16usize;
    let top_k = ERNIE55_TOP_K_EXPERTS; // 10
    let thresholds = (75u8, 180u8);
    let centroids = [20u8, 128u8, 236u8];

    let build_packed_mat = |rows: usize, cols: usize, seed: usize, bend_k: u8, alpha: f32| {
        let raw: Vec<u8> = (0..(rows * cols))
            .map(|i| ((i.wrapping_mul(37 + seed) + 19) & 0xFF) as u8)
            .collect();
        pack_latent_moe_matrix_par_simd(&raw, rows, cols, thresholds, centroids, bend_k, alpha)
    };

    let router_projs: Vec<_> = (0..DEFAULT_MULTI_ROUTERS)
        .map(|r| build_packed_mat(num_routed_experts, d_model, r * 11 + 1, (r % 3) as u8, 0.01))
        .collect();
    let learnable_scalars: Vec<f32> = (0..num_routed_experts)
        .map(|e| 0.85 + 0.02 * (e as f32))
        .collect();
    let mut router = MultiRouterGate::new(router_projs, learnable_scalars, top_k);

    let shared_down_proj = build_packed_mat(d_latent, d_model, 101, 1, 0.005);
    assert_eq!(
        shared_down_proj.packed_row_stride_bytes,
        PACKED_HIDDEN_ROW_BYTES
    );
    assert_eq!(shared_down_proj.packed_data.len(), 2048 * 820);

    let shared_up_proj = build_packed_mat(d_model, d_latent, 103, 2, 0.02);
    assert_eq!(
        shared_up_proj.packed_row_stride_bytes,
        PACKED_LATENT_ROW_BYTES
    );
    assert_eq!(shared_up_proj.packed_data.len(), 4096 * 410);

    let build_expert = |e_id: usize| -> LatentExpert5Trit {
        let base_mat = build_packed_mat(
            d_latent,
            d_latent,
            200 + e_id * 7,
            ((e_id + 1) % 3) as u8,
            0.008,
        );
        assert_eq!(base_mat.packed_data.len(), 2048 * 410);
        LatentExpert5Trit {
            expert_id: e_id,
            gate_proj: base_mat.clone(),
            up_proj: base_mat.clone(),
            down_proj: base_mat,
        }
    };

    let shared_expert = build_expert(0);
    assert_eq!(shared_expert.packed_bytes(), 3 * 2048 * 410);

    let routed_experts: Vec<LatentExpert5Trit> =
        (0..num_routed_experts).map(build_expert).collect();

    let sample_x: Vec<f32> = (0..d_model).map(|i| ((i as f32) * 0.013).sin()).collect();
    let (x_q, x_s) =
        st::mcp_combinatorics_llm::latent_moe_layer::quantize_activations_sym_i8(&sample_x);
    let sel_before = router.route_token(&x_q, x_s);
    assert_eq!(sel_before.selected_experts.len(), top_k);

    for _ in 0..10 {
        router.update_load_balance_biases(&sel_before.selected_experts, 0.25);
    }
    let top_expert_0 = sel_before.selected_experts[0];
    assert!(
        router.load_balance_bias[top_expert_0] < 0.0,
        "Over-selected expert must receive negative load-balancing bias without auxiliary loss"
    );

    let layer = Ernie55LatentMoeLayer {
        hidden_dim: d_model,
        latent_dim: d_latent,
        router,
        shared_down_proj,
        shared_expert,
        routed_experts,
        pre_up_rmsnorm_gamma: vec![1.0; d_latent],
        rmsnorm_eps: 1e-5,
        shared_up_proj,
        mhc_mixing_matrix: [
            [0.55, 0.15, 0.15, 0.15],
            [0.15, 0.55, 0.15, 0.15],
            [0.15, 0.15, 0.55, 0.15],
            [0.15, 0.15, 0.15, 0.55],
        ],
        mhc_read_weights: [0.25, 0.25, 0.25, 0.25],
        mhc_write_weights: [0.25, 0.25, 0.25, 0.25],
    };

    let mut streams: [Vec<f32>; MHC_STREAMS] = [
        (0..d_model).map(|i| ((i as f32) * 0.01).cos()).collect(),
        (0..d_model).map(|i| ((i as f32) * 0.02).sin()).collect(),
        (0..d_model).map(|i| ((i as f32) * 0.03).cos()).collect(),
        (0..d_model).map(|i| ((i as f32) * 0.04).sin()).collect(),
    ];

    let telemetry = layer.forward_mhc_streams(&mut streams);

    assert_eq!(telemetry.hidden_dim, 4096);
    assert_eq!(telemetry.latent_dim, 2048);
    assert_eq!(telemetry.active_routed_experts, 10);
    assert_eq!(telemetry.shared_experts, 1);
    assert_eq!(telemetry.routing.per_router_scores.len(), 4);
    assert!(telemetry.pre_up_proj_inv_rms.is_finite() && telemetry.pre_up_proj_inv_rms > 0.0);

    assert!(
        (telemetry.mhc_l1_norm_before - telemetry.mhc_l1_norm_after_mixing).abs() < 1e-2,
        "4-stream mHC doubly-stochastic mixing must preserve column-sum L1 norm"
    );

    for s in &streams {
        assert_eq!(s.len(), 4096);
        assert!(s.iter().all(|v| v.is_finite()));
    }
}
