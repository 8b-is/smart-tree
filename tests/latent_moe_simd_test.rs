use st::mcp_combinatorics_llm::latent_moe_simd::{
    pack_latent_moe_matrix_par_simd, quantize_bend_pack_row_scalar, quantize_bend_pack_row_simd,
    unpack_and_unbend_row, ERNIE55_HIDDEN_DIM, ERNIE55_LATENT_DIM, PACKED_HIDDEN_ROW_BYTES,
    PACKED_LATENT_ROW_BYTES,
};

#[test]
fn test_ernie55_latent_moe_4096_to_2048_simd_bent_5trit_kernel() {
    assert_eq!(PACKED_LATENT_ROW_BYTES, 410); // ceil(2048 / 5) = 410 bytes
    assert_eq!(PACKED_HIDDEN_ROW_BYTES, 820); // ceil(4096 / 5) = 820 bytes

    let thresholds = (72u8, 184u8);
    let centroids = [16u8, 128u8, 240u8];

    // 1. Verify SIMD vs Scalar bit-exact parity across 2048-d (410 B) and 4096-d (820 B) rows
    //    and all cyclic bend multipliers k in {0, 1, 2}
    for &dim in &[ERNIE55_LATENT_DIM, ERNIE55_HIDDEN_DIM, 2053usize] {
        let weights: Vec<u8> = (0..dim).map(|i| ((i * 41 + 17) & 0xFF) as u8).collect();
        let packed_len = (dim + 4) / 5;

        for bend_k in 0u8..=2u8 {
            let mut out_scalar = vec![0u8; packed_len];
            let mut out_simd = vec![0u8; packed_len];

            let sse_scalar = quantize_bend_pack_row_scalar(
                &weights,
                thresholds,
                centroids,
                bend_k,
                &mut out_scalar,
            );
            let sse_simd =
                quantize_bend_pack_row_simd(&weights, thresholds, centroids, bend_k, &mut out_simd);

            assert_eq!(
                sse_simd, sse_scalar,
                "64-bit SSE mismatch at dim={dim}, bend_k={bend_k}"
            );
            assert_eq!(
                out_simd, out_scalar,
                "Packed 5-trit bytes diverged at dim={dim}, bend_k={bend_k}"
            );

            // 2. Verify lossless inverse unbending back to balanced trits {-1, 0, +1} (disagreed == 0)
            let mut recovered_bal = vec![0i8; dim];
            unpack_and_unbend_row(&out_simd, bend_k, &mut recovered_bal);

            for (i, &w) in weights.iter().enumerate() {
                let expected_bal: i8 = if w < thresholds.0 {
                    -1
                } else if w < thresholds.1 {
                    0
                } else {
                    1
                };
                assert_eq!(
                    recovered_bal[i], expected_bal,
                    "Unbent balanced trit mismatch at index {i} (dim={dim}, bend_k={bend_k})"
                );
            }
        }
    }

    // 3. Verify full 4096 x 2048 LatentMoE matrix parallel compression (8,388,608 weights -> 1,679,360 B)
    let total_weights = ERNIE55_HIDDEN_DIM * ERNIE55_LATENT_DIM;
    let matrix_weights: Vec<u8> = (0..total_weights)
        .map(|i| ((i * 29 + 7) & 0xFF) as u8)
        .collect();

    let packed_mat = pack_latent_moe_matrix_par_simd(
        &matrix_weights,
        ERNIE55_HIDDEN_DIM,
        ERNIE55_LATENT_DIM,
        thresholds,
        centroids,
        2,
        1.0,
    );

    assert_eq!(packed_mat.packed_row_stride_bytes, 410);
    assert_eq!(packed_mat.packed_data.len(), 4096 * 410); // 1,679,360 bytes
    assert!(packed_mat.total_reconstruction_sse_u64 > 0);
}

#[test]
fn test_fused_unpack_unbend_gemv_410b_latent_moe_row_and_i8_min_safety() {
    use st::mcp_combinatorics_llm::latent_moe_simd::{
        fused_unpack_unbend_gemv_row_scalar, fused_unpack_unbend_gemv_row_simd,
        latent_moe_gemv_par_simd, pack_latent_moe_matrix_par_simd, quantize_bend_pack_row_simd,
        unpack_and_unbend_row, ERNIE55_HIDDEN_DIM, ERNIE55_LATENT_DIM, PACKED_LATENT_ROW_BYTES,
    };

    let thresholds = (70u8, 185u8);
    let centroids = [16u8, 128u8, 240u8];

    for &dim in &[ERNIE55_LATENT_DIM, ERNIE55_HIDDEN_DIM, 2053usize] {
        let raw_weights: Vec<u8> = (0..dim).map(|i| ((i * 53 + 19) & 0xFF) as u8).collect();
        let activations: Vec<i8> = (0..dim)
            .map(|i| ((i * 97 + 31) & 0xFF) as u8 as i8)
            .collect();

        let packed_len = (dim + 4) / 5;
        if dim == ERNIE55_LATENT_DIM {
            assert_eq!(packed_len, PACKED_LATENT_ROW_BYTES); // 410 bytes
        }

        for bend_k in 0u8..=2u8 {
            let mut packed_row = vec![0u8; packed_len];
            quantize_bend_pack_row_simd(
                &raw_weights,
                thresholds,
                centroids,
                bend_k,
                &mut packed_row,
            );

            let mut unpacked_bal = vec![0i8; dim];
            unpack_and_unbend_row(&packed_row, bend_k, &mut unpacked_bal);
            let expected_two_pass_dot: i32 = unpacked_bal
                .iter()
                .zip(activations.iter())
                .map(|(&t, &x)| (t as i32) * (x as i32))
                .sum();

            let dot_scalar = fused_unpack_unbend_gemv_row_scalar(&packed_row, &activations, bend_k);
            let dot_simd = fused_unpack_unbend_gemv_row_simd(&packed_row, &activations, bend_k);

            assert_eq!(
                dot_scalar, expected_two_pass_dot,
                "Scalar fused GEMV diverged at dim={dim}, bend_k={bend_k}"
            );
            assert_eq!(
                dot_simd, expected_two_pass_dot,
                "SIMD fused GEMV diverged from two-pass reference at dim={dim}, bend_k={bend_k}"
            );
        }
    }

    let all_neg_weights = vec![0u8; ERNIE55_LATENT_DIM];
    let all_min_acts = vec![i8::MIN; ERNIE55_LATENT_DIM];
    let mut packed_neg = vec![0u8; PACKED_LATENT_ROW_BYTES];
    quantize_bend_pack_row_simd(&all_neg_weights, thresholds, centroids, 2, &mut packed_neg);

    let min_dot = fused_unpack_unbend_gemv_row_simd(&packed_neg, &all_min_acts, 2);
    assert_eq!(
        min_dot,
        (ERNIE55_LATENT_DIM as i32) * 128,
        "Fused SIMD GEMV must handle x_i == -128 without 8-bit sign-negation wraparound"
    );

    let rows = 64usize;
    let cols = ERNIE55_LATENT_DIM;
    let mat_weights: Vec<u8> = (0..(rows * cols))
        .map(|i| ((i * 37 + 11) & 0xFF) as u8)
        .collect();
    let vec_acts: Vec<i8> = (0..cols).map(|i| ((i * 17) as u8) as i8).collect();

    let packed_mat =
        pack_latent_moe_matrix_par_simd(&mat_weights, rows, cols, thresholds, centroids, 1, 0.75);
    let gemv_out = latent_moe_gemv_par_simd(&packed_mat, &vec_acts, 1.0 / 127.0);
    assert_eq!(gemv_out.raw_integer_dots.len(), rows);
    assert_eq!(gemv_out.scaled_outputs.len(), rows);
}
