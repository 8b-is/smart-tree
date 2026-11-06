//! ERNIE-5.5-Flash-Base LatentMoE bent balanced-ternary 5-trit SIMD kernel.
//!
//! Quantizes a `4096 -> 2048` shared down-projection (and `2048 -> 4096` shared
//! up-projection) weight matrix into `{-1, 0, +1}` trits, applies the
//! position-dependent `(k * i) mod 3` cyclic bend, and packs 5 bent trits per
//! byte (base-3, `3^5 = 243 <= 256`). A 2048-d latent row packs to 410 bytes;
//! a 4096-d projection row packs to 820 bytes.

use rayon::prelude::*;
use serde::{Deserialize, Serialize};

/// ERNIE 5.5 Flash Base full hidden dimension (d_model = 4096).
pub const ERNIE55_HIDDEN_DIM: usize = 4096;
/// ERNIE 5.5 Flash Base LatentMoE bottleneck & expert intermediate dimension (d_latent = 2048).
pub const ERNIE55_LATENT_DIM: usize = 2048;
/// Packed 5-trit byte length for a 2048-d LatentMoE row: ceil(2048 / 5) = 410 bytes.
pub const PACKED_LATENT_ROW_BYTES: usize = (ERNIE55_LATENT_DIM + 4) / 5;
/// Packed 5-trit byte length for a 4096-d projection row: ceil(4096 / 5) = 820 bytes.
pub const PACKED_HIDDEN_ROW_BYTES: usize = (ERNIE55_HIDDEN_DIM + 4) / 5;

// ============================================================================
// 1. Compile-Time SIMD Bend Phase Tables: r_{i0 + j} = (r_base + k * j) mod 3
// ============================================================================

/// AVX2 20-trit phase table `[k_mod3][r_base]` formatted for two 10-trit 128-bit lanes:
/// - Low 128-bit lane bytes 0..10 hold `(r_base + k * j) mod 3` for `j in 0..10`.
/// - High 128-bit lane bytes 16..26 hold `(r_base + k * j) mod 3` for `j in 10..20`.
#[cfg(target_arch = "x86_64")]
const AVX2_BEND_PHASE_20: [[[i8; 32]; 3]; 3] = build_avx2_bend_phase_20();

#[cfg(target_arch = "x86_64")]
const fn build_avx2_bend_phase_20() -> [[[i8; 32]; 3]; 3] {
    let mut table = [[[0i8; 32]; 3]; 3];
    let mut k = 0usize;
    while k < 3 {
        let mut r_base = 0usize;
        while r_base < 3 {
            let mut j = 0usize;
            while j < 10 {
                table[k][r_base][j] = ((r_base + k * j) % 3) as i8;
                let j_hi = j + 10;
                table[k][r_base][16 + j] = ((r_base + k * j_hi) % 3) as i8;
                j += 1;
            }
            r_base += 1;
        }
        k += 1;
    }
    table
}

/// ARM NEON 48-byte phase table `[k_mod3][r_base]` holding `(r_base + k * j) mod 3` for `j in 0..48`.
#[cfg(target_arch = "aarch64")]
const NEON_BEND_PHASE_48: [[[u8; 48]; 3]; 3] = build_neon_bend_phase_48();

#[cfg(target_arch = "aarch64")]
const fn build_neon_bend_phase_48() -> [[[u8; 48]; 3]; 3] {
    let mut table = [[[0u8; 48]; 3]; 3];
    let mut k = 0usize;
    while k < 3 {
        let mut r_base = 0usize;
        while r_base < 3 {
            let mut j = 0usize;
            while j < 48 {
                table[k][r_base][j] = ((r_base + k * j) % 3) as u8;
                j += 1;
            }
            r_base += 1;
        }
        k += 1;
    }
    table
}

/// Compile-time 243-entry base-3 unpacking lookup table.
pub const UNPACK_5TRITS_LUT: [[u8; 5]; 243] = build_5trit_lut();

const fn build_5trit_lut() -> [[u8; 5]; 243] {
    let mut lut = [[0u8; 5]; 243];
    let mut b = 0usize;
    while b < 243 {
        let mut rem = b as u8;
        let mut m = 0;
        while m < 5 {
            lut[b][m] = rem % 3;
            rem /= 3;
            m += 1;
        }
        b += 1;
    }
    lut
}

// ============================================================================
// 2. Public Fused Quantize + Bent (k*i) mod 3 + 5-Trit Row Packer
// ============================================================================

/// Quantizes a row of raw weights `weights` (`u8` in `0..=255`) against `thresholds = (T_1, T_2)`
/// into unbent trits `u_i in {0, 1, 2}` (representing balanced trits `t_i = u_i - 1 in {-1, 0, +1}`),
/// applies the position-dependent cyclic bend `u'_i = (u_i + k * i) mod 3`, packs 5 bent trits per
/// byte into `packed_out` (410 bytes for 2048-d, 820 bytes for 4096-d), and returns the exact
/// 64-bit unbent reconstruction error `E = \sum (w_i - c_{q(w_i)})^2`.
pub fn quantize_bend_pack_row_simd(
    weights: &[u8],
    thresholds: (u8, u8),
    centroids: [u8; 3],
    bend_k: u8,
    packed_out: &mut [u8],
) -> u64 {
    let expected_bytes = (weights.len() + 4) / 5;
    assert!(
        packed_out.len() >= expected_bytes,
        "packed_out too small: need {expected_bytes} bytes for {} weights, got {}",
        weights.len(),
        packed_out.len()
    );

    #[cfg(target_arch = "x86_64")]
    {
        if std::arch::is_x86_feature_detected!("avx2") {
            // SAFETY: AVX2 runtime support is verified above.
            unsafe {
                return quantize_bend_pack_row_avx2(
                    weights, thresholds, centroids, bend_k, packed_out,
                );
            }
        }
    }

    #[cfg(target_arch = "aarch64")]
    {
        if std::arch::is_aarch64_feature_detected!("neon") {
            // SAFETY: NEON runtime support is verified above.
            unsafe {
                return quantize_bend_pack_row_neon(
                    weights, thresholds, centroids, bend_k, packed_out,
                );
            }
        }
    }

    quantize_bend_pack_row_scalar(weights, thresholds, centroids, bend_k, packed_out)
}

/// Scalar reference implementation using the exact assembly recurrence (`one add, one conditional subtract`)
/// for `r_i = (k * i) mod 3`.
pub fn quantize_bend_pack_row_scalar(
    weights: &[u8],
    thresholds: (u8, u8),
    centroids: [u8; 3],
    bend_k: u8,
    packed_out: &mut [u8],
) -> u64 {
    quantize_bend_pack_row_scalar_with_phase(weights, thresholds, centroids, bend_k, 0, packed_out)
}

fn quantize_bend_pack_row_scalar_with_phase(
    weights: &[u8],
    thresholds: (u8, u8),
    centroids: [u8; 3],
    bend_k: u8,
    initial_phase_mod3: u8,
    packed_out: &mut [u8],
) -> u64 {
    let (t1, t2) = thresholds;
    let [c0, c1, c2] = centroids;
    let k_mod3 = bend_k % 3;
    let mut r_mod3 = initial_phase_mod3 % 3;

    let mut total_sse = 0u64;
    let mut out_idx = 0usize;

    let chunks = weights.chunks_exact(5);
    let rem = chunks.remainder();

    for chunk in chunks {
        let mut bent_5 = [0u8; 5];
        for (m, &w) in chunk.iter().enumerate() {
            let (u_trit, cq) = if w < t1 {
                (0u8, c0)
            } else if w < t2 {
                (1u8, c1)
            } else {
                (2u8, c2)
            };

            let diff = (w as i32) - (cq as i32);
            total_sse += (diff * diff) as u64;

            // Apply cyclic bend (u_trit + r_mod3) mod 3 with zero divisions
            let mut bent = u_trit + r_mod3;
            if bent >= 3 {
                bent -= 3;
            }
            bent_5[m] = bent;

            // Advance ASM recurrence: one add + one conditional subtract
            r_mod3 += k_mod3;
            if r_mod3 >= 3 {
                r_mod3 -= 3;
            }
        }

        packed_out[out_idx] =
            bent_5[0] + 3 * bent_5[1] + 9 * bent_5[2] + 27 * bent_5[3] + 81 * bent_5[4];
        out_idx += 1;
    }

    if !rem.is_empty() {
        let mut bent_tail = [0u8; 5];
        for (m, &w) in rem.iter().enumerate() {
            let (u_trit, cq) = if w < t1 {
                (0u8, c0)
            } else if w < t2 {
                (1u8, c1)
            } else {
                (2u8, c2)
            };

            let diff = (w as i32) - (cq as i32);
            total_sse += (diff * diff) as u64;

            let mut bent = u_trit + r_mod3;
            if bent >= 3 {
                bent -= 3;
            }
            bent_tail[m] = bent;

            r_mod3 += k_mod3;
            if r_mod3 >= 3 {
                r_mod3 -= 3;
            }
        }

        packed_out[out_idx] = bent_tail[0]
            + 3 * bent_tail[1]
            + 9 * bent_tail[2]
            + 27 * bent_tail[3]
            + 81 * bent_tail[4];
    }

    total_sse
}

// ============================================================================
// 3. x86_64 AVX2 Fused Quantize + Bent (k*i) mod 3 + 5-Trit Base-3 Packer
// ============================================================================

#[cfg(target_arch = "x86_64")]
#[target_feature(enable = "avx2")]
unsafe fn quantize_bend_pack_row_avx2(
    weights: &[u8],
    thresholds: (u8, u8),
    centroids: [u8; 3],
    bend_k: u8,
    packed_out: &mut [u8],
) -> u64 {
    use std::arch::x86_64::*;

    let n = weights.len();
    let mut in_offset = 0usize;
    let mut out_offset = 0usize;

    let k_mod3 = (bend_k % 3) as usize;
    // Advancing 20 elements shifts r_base by (20 * k) mod 3 = (2 * k) mod 3
    let stride20_step_mod3 = ((20 * k_mod3) % 3) as u8;
    let mut r_base: u8 = 0;

    let t1_vec = _mm256_set1_epi8(thresholds.0 as i8);
    let t2_vec = _mm256_set1_epi8(thresholds.1 as i8);
    let c0_vec = _mm256_set1_epi8(centroids[0] as i8);
    let c1_vec = _mm256_set1_epi8(centroids[1] as i8);
    let c2_vec = _mm256_set1_epi8(centroids[2] as i8);
    let zero = _mm256_setzero_si256();
    let v3 = _mm256_set1_epi8(3);

    // Mask out bytes 10..16 of each 128-bit lane so only the 20 active weights contribute to SSE
    let active_20_mask = _mm256_setr_epi8(
        -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, 0, 0, 0, 0, 0, 0, -1, -1, -1, -1, -1, -1, -1, -1,
        -1, -1, 0, 0, 0, 0, 0, 0,
    );

    // Align 2 groups of 5 trits per 128-bit lane into 8-byte blocks for `_mm256_maddubs_epi16`
    let group_shuf = _mm256_setr_epi8(
        0, 1, 2, 3, 4, -1, -1, -1, 5, 6, 7, 8, 9, -1, -1, -1, 0, 1, 2, 3, 4, -1, -1, -1, 5, 6, 7,
        8, 9, -1, -1, -1,
    );
    let pow3_weights = _mm256_setr_epi8(
        1, 3, 9, 27, 81, 0, 0, 0, 1, 3, 9, 27, 81, 0, 0, 0, 1, 3, 9, 27, 81, 0, 0, 0, 1, 3, 9, 27,
        81, 0, 0, 0,
    );
    let ones_16 = _mm256_set1_epi16(1);

    let phase_vecs = [
        _mm256_loadu_si256(AVX2_BEND_PHASE_20[k_mod3][0].as_ptr() as *const __m256i),
        _mm256_loadu_si256(AVX2_BEND_PHASE_20[k_mod3][1].as_ptr() as *const __m256i),
        _mm256_loadu_si256(AVX2_BEND_PHASE_20[k_mod3][2].as_ptr() as *const __m256i),
    ];

    let mut acc_64_lo = _mm256_setzero_si256();
    let mut acc_64_hi = _mm256_setzero_si256();

    // Process 20 weights -> 4 packed base-3 bytes per iteration.
    // Guard `in_offset + 26 <= n` ensures the second 128-bit load at `ptr.add(10)` is in-bounds.
    while in_offset + 26 <= n {
        let ptr = weights.as_ptr().add(in_offset);
        let lo_128 = _mm_loadu_si128(ptr as *const __m128i);
        let hi_128 = _mm_loadu_si128(ptr.add(10) as *const __m128i);
        let x = _mm256_set_m128i(hi_128, lo_128);

        // 1. Threshold quantize into unbent trits u in {0, 1, 2}
        let mask_ge_t1 = _mm256_cmpeq_epi8(_mm256_max_epu8(x, t1_vec), x);
        let mask_ge_t2 = _mm256_cmpeq_epi8(_mm256_max_epu8(x, t2_vec), x);
        let u_trits = _mm256_sub_epi8(zero, _mm256_add_epi8(mask_ge_t1, mask_ge_t2));

        // 2. Accumulate 64-bit reconstruction SSE across the 20 active lanes
        let c_01 = _mm256_blendv_epi8(c0_vec, c1_vec, mask_ge_t1);
        let c_q = _mm256_blendv_epi8(c_01, c2_vec, mask_ge_t2);
        let raw_diff = _mm256_or_si256(_mm256_subs_epu8(x, c_q), _mm256_subs_epu8(c_q, x));
        let diff_u8 = _mm256_and_si256(raw_diff, active_20_mask);

        let d16_lo = _mm256_unpacklo_epi8(diff_u8, zero);
        let d16_hi = _mm256_unpackhi_epi8(diff_u8, zero);
        let sq32_sum = _mm256_add_epi32(
            _mm256_madd_epi16(d16_lo, d16_lo),
            _mm256_madd_epi16(d16_hi, d16_hi),
        );
        acc_64_lo = _mm256_add_epi64(acc_64_lo, _mm256_unpacklo_epi32(sq32_sum, zero));
        acc_64_hi = _mm256_add_epi64(acc_64_hi, _mm256_unpackhi_epi32(sq32_sum, zero));

        // 3. Apply position-dependent cyclic bend: bent = (u_trits + phase) mod 3
        let sum_ur = _mm256_add_epi8(u_trits, phase_vecs[r_base as usize]);
        let ge_3 = _mm256_cmpeq_epi8(_mm256_max_epu8(sum_ur, v3), sum_ur);
        let bent_trits = _mm256_sub_epi8(sum_ur, _mm256_and_si256(ge_3, v3));

        // Advance block recurrence: one add and one conditional subtract
        r_base += stride20_step_mod3;
        if r_base >= 3 {
            r_base -= 3;
        }

        // 4. Pack 20 bent trits into 4 base-3 bytes via `_mm256_maddubs_epi16`
        let aligned = _mm256_shuffle_epi8(bent_trits, group_shuf);
        let prod_16 = _mm256_maddubs_epi16(aligned, pow3_weights);
        let sum_32 = _mm256_madd_epi16(prod_16, ones_16);
        let total_32 = _mm256_hadd_epi32(sum_32, sum_32);

        let out_ptr = packed_out.as_mut_ptr().add(out_offset);
        *out_ptr = _mm256_extract_epi32::<0>(total_32) as u8;
        *out_ptr.add(1) = _mm256_extract_epi32::<1>(total_32) as u8;
        *out_ptr.add(2) = _mm256_extract_epi32::<4>(total_32) as u8;
        *out_ptr.add(3) = _mm256_extract_epi32::<5>(total_32) as u8;

        in_offset += 20;
        out_offset += 4;
    }

    let total_vec = _mm256_add_epi64(acc_64_lo, acc_64_hi);
    let mut lanes = [0u64; 4];
    _mm256_storeu_si256(lanes.as_mut_ptr() as *mut __m256i, total_vec);
    let mut total_sse = lanes.iter().sum::<u64>();

    // Process remaining tail weights (e.g. final 8 weights at W = 2048 -> 2 packed bytes)
    if in_offset < n {
        total_sse += quantize_bend_pack_row_scalar_with_phase(
            &weights[in_offset..],
            thresholds,
            centroids,
            bend_k,
            r_base,
            &mut packed_out[out_offset..],
        );
    }

    total_sse
}

// ============================================================================
// 4. AArch64 NEON Fused Quantize + Bent (k*i) mod 3 + 5-Trit Base-3 Packer
// ============================================================================

#[cfg(target_arch = "aarch64")]
#[target_feature(enable = "neon")]
unsafe fn quantize_bend_pack_row_neon(
    weights: &[u8],
    thresholds: (u8, u8),
    centroids: [u8; 3],
    bend_k: u8,
    packed_out: &mut [u8],
) -> u64 {
    use std::arch::aarch64::*;

    let n = weights.len();
    let mut in_offset = 0usize;
    let mut out_offset = 0usize;

    let k_mod3 = (bend_k % 3) as usize;
    // Advancing 40 elements shifts r_base by (40 * k) mod 3 = k mod 3
    let stride40_step_mod3 = (k_mod3 % 3) as u8;
    let mut r_base: u8 = 0;

    let t1_vec = vdupq_n_u8(thresholds.0);
    let t2_vec = vdupq_n_u8(thresholds.1);
    let c0_vec = vdupq_n_u8(centroids[0]);
    let c1_vec = vdupq_n_u8(centroids[1]);
    let c2_vec = vdupq_n_u8(centroids[2]);
    let v3 = vdupq_n_u8(3);

    let w3 = vdup_n_u8(3);
    let w9 = vdup_n_u8(9);
    let w27 = vdup_n_u8(27);
    let w81 = vdup_n_u8(81);

    let idx_t0: [u8; 8] = [0, 5, 10, 15, 20, 25, 30, 35];
    let idx_t1: [u8; 8] = [1, 6, 11, 16, 21, 26, 31, 36];
    let idx_t2: [u8; 8] = [2, 7, 12, 17, 22, 27, 32, 37];
    let idx_t3: [u8; 8] = [3, 8, 13, 18, 23, 28, 33, 38];
    let idx_t4: [u8; 8] = [4, 9, 14, 19, 24, 29, 34, 39];

    let v_idx0 = vld1_u8(idx_t0.as_ptr());
    let v_idx1 = vld1_u8(idx_t1.as_ptr());
    let v_idx2 = vld1_u8(idx_t2.as_ptr());
    let v_idx3 = vld1_u8(idx_t3.as_ptr());
    let v_idx4 = vld1_u8(idx_t4.as_ptr());

    let mut acc_u64 = vdupq_n_u64(0);

    // Helper closure to quantize 16 bytes, accumulate u64 SSE on `active_lanes`, and apply bend
    let quantize_and_bend_16 = |x: uint8x16_t,
                                phase: uint8x16_t,
                                lane_mask: uint8x16_t,
                                acc: &mut uint64x2_t|
     -> uint8x16_t {
        let m_t1 = vcgeq_u8(x, t1_vec);
        let m_t2 = vcgeq_u8(x, t2_vec);

        let u_s8 = vnegq_s8(vaddq_s8(
            vreinterpretq_s8_u8(m_t1),
            vreinterpretq_s8_u8(m_t2),
        ));
        let u_trits = vreinterpretq_u8_s8(u_s8);

        let c_01 = vbslq_u8(m_t1, c1_vec, c0_vec);
        let c_q = vbslq_u8(m_t2, c2_vec, c_01);
        let diff = vandq_u8(vabdq_u8(x, c_q), lane_mask);

        let sq_lo = vmull_u8(vget_low_u8(diff), vget_low_u8(diff));
        let sq_hi = vmull_u8(vget_high_u8(diff), vget_high_u8(diff));
        let mut sum_u32 = vpaddlq_u16(sq_lo);
        sum_u32 = vpadalq_u16(sum_u32, sq_hi);
        *acc = vpadalq_u32(*acc, sum_u32);

        // Branchless cyclic rotation: (u_trits + phase) mod 3
        let sum_ur = vaddq_u8(u_trits, phase);
        let ge_3 = vcgeq_u8(sum_ur, v3);
        vsubq_u8(sum_ur, vandq_u8(ge_3, v3))
    };

    let mask_all_16 = vdupq_n_u8(0xFF);
    let mask_low_8 = vcombine_u8(vdup_n_u8(0xFF), vdup_n_u8(0x00));

    // Process 40 weights -> 8 packed base-3 bytes per iteration (guard +48 for vld1q_u8_x3)
    while in_offset + 48 <= n {
        let raw_x = vld1q_u8_x3(weights.as_ptr().add(in_offset));
        let phase_tbl = vld1q_u8_x3(NEON_BEND_PHASE_48[k_mod3][r_base as usize].as_ptr());

        let b0 = quantize_and_bend_16(raw_x.0, phase_tbl.0, mask_all_16, &mut acc_u64);
        let b1 = quantize_and_bend_16(raw_x.1, phase_tbl.1, mask_all_16, &mut acc_u64);
        let b2 = quantize_and_bend_16(raw_x.2, phase_tbl.2, mask_low_8, &mut acc_u64);

        // Advance block recurrence: one add and one conditional subtract
        r_base += stride40_step_mod3;
        if r_base >= 3 {
            r_base -= 3;
        }

        let bent_tbl = uint8x16x3_t(b0, b1, b2);
        let t0 = vqtbl3_u8(bent_tbl, v_idx0);
        let t1 = vqtbl3_u8(bent_tbl, v_idx1);
        let t2 = vqtbl3_u8(bent_tbl, v_idx2);
        let t3 = vqtbl3_u8(bent_tbl, v_idx3);
        let t4 = vqtbl3_u8(bent_tbl, v_idx4);

        let mut packed_8 = vmla_u8(t0, t1, w3);
        packed_8 = vmla_u8(packed_8, t2, w9);
        packed_8 = vmla_u8(packed_8, t3, w27);
        packed_8 = vmla_u8(packed_8, t4, w81);

        vst1_u8(packed_out.as_mut_ptr().add(out_offset), packed_8);

        in_offset += 40;
        out_offset += 8;
    }

    let mut total_sse = vaddvq_u64(acc_u64);
    if in_offset < n {
        total_sse += quantize_bend_pack_row_scalar_with_phase(
            &weights[in_offset..],
            thresholds,
            centroids,
            bend_k,
            r_base,
            &mut packed_out[out_offset..],
        );
    }

    total_sse
}

// ============================================================================
// 5. Inverse Unpack + Unbend `(3 - (k*i) mod 3) mod 3` + LatentMoE Matrix Scan
// ============================================================================

/// Unpacks a 410-byte (2048-d) or 820-byte (4096-d) packed 5-trit row and reverses the
/// `(k * i) mod 3` cyclic bend using the division-free ASM recurrence, recovering the
/// exact unbent balanced trits `{-1, 0, +1}` in `out_balanced_trits`.
pub fn unpack_and_unbend_row(
    packed_bytes: &[u8],
    bend_k: u8,
    out_balanced_trits: &mut [i8],
) -> usize {
    let total_trits = out_balanced_trits.len();
    let required_bytes = (total_trits + 4) / 5;
    assert!(packed_bytes.len() >= required_bytes);

    let k_mod3 = bend_k % 3;
    let mut r_mod3 = 0u8;

    let full_groups = total_trits / 5;
    let rem_trits = total_trits % 5;

    for g in 0..full_groups {
        let b = packed_bytes[g];
        let bent_5 = if (b as usize) < 243 {
            UNPACK_5TRITS_LUT[b as usize]
        } else {
            [0; 5]
        };

        for m in 0..5 {
            // Inverse cyclic shift: (bent + 3 - r_mod3) mod 3
            let mut unbent_u = bent_5[m] + 3 - r_mod3;
            if unbent_u >= 3 {
                unbent_u -= 3;
            }
            out_balanced_trits[g * 5 + m] = (unbent_u as i8) - 1;

            r_mod3 += k_mod3;
            if r_mod3 >= 3 {
                r_mod3 -= 3;
            }
        }
    }

    if rem_trits > 0 {
        let b = packed_bytes[full_groups];
        let bent_5 = if (b as usize) < 243 {
            UNPACK_5TRITS_LUT[b as usize]
        } else {
            [0; 5]
        };
        let base = full_groups * 5;
        for m in 0..rem_trits {
            let mut unbent_u = bent_5[m] + 3 - r_mod3;
            if unbent_u >= 3 {
                unbent_u -= 3;
            }
            out_balanced_trits[base + m] = (unbent_u as i8) - 1;

            r_mod3 += k_mod3;
            if r_mod3 >= 3 {
                r_mod3 -= 3;
            }
        }
    }

    total_trits
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LatentMoePackedMatrix {
    pub rows: usize,
    pub cols: usize,
    pub packed_row_stride_bytes: usize,
    pub bend_k: u8,
    pub thresholds: (u8, u8),
    pub centroids: [u8; 3],
    pub routing_scalar_alpha: f32,
    pub total_reconstruction_sse_u64: u64,
    pub packed_data: Vec<u8>,
}

/// Quantizes, bends, and packs an entire ERNIE 5.5 LatentMoE weight matrix
/// (e.g. `4096 x 2048` shared projection or `2048 x 2048` routed latent expert)
/// in parallel across Rayon worker threads.
pub fn pack_latent_moe_matrix_par_simd(
    weights: &[u8],
    rows: usize,
    cols: usize,
    thresholds: (u8, u8),
    centroids: [u8; 3],
    bend_k: u8,
    routing_scalar_alpha: f32,
) -> LatentMoePackedMatrix {
    assert_eq!(weights.len(), rows * cols);
    let row_stride = (cols + 4) / 5;
    let mut packed_data = vec![0u8; rows * row_stride];

    let total_sse: u64 = weights
        .par_chunks_exact(cols)
        .zip(packed_data.par_chunks_exact_mut(row_stride))
        .enumerate()
        .map(|(row_idx, (w_row, out_row))| {
            // Rotate the bend multiplier per row `((bend_k + row_idx) % 3)` or keep fixed `bend_k`
            let effective_k = if bend_k == 0 {
                0
            } else {
                ((bend_k as usize + row_idx) % 3) as u8
            };
            quantize_bend_pack_row_simd(w_row, thresholds, centroids, effective_k, out_row)
        })
        .sum();

    LatentMoePackedMatrix {
        rows,
        cols,
        packed_row_stride_bytes: row_stride,
        bend_k,
        thresholds,
        centroids,
        routing_scalar_alpha,
        total_reconstruction_sse_u64: total_sse,
        packed_data,
    }
}

// ============================================================================
// Compile-Time Planar Unbend Tables & 5-Way Register Interleave Masks
// ============================================================================

/// AVX2 planar inverse-bend table `[k_mod3][r_base][plane_m]` (32 bytes per entry):
/// Byte `g in 0..32` holds `(3 - ((r_base + k * (5*g + m)) mod 3)) mod 3`.
#[cfg(target_arch = "x86_64")]
const AVX2_PLANAR_UNBEND_32: [[[[i8; 32]; 5]; 3]; 3] = build_avx2_planar_unbend_32();

#[cfg(target_arch = "x86_64")]
const fn build_avx2_planar_unbend_32() -> [[[[i8; 32]; 5]; 3]; 3] {
    let mut tbl = [[[[0i8; 32]; 5]; 3]; 3];
    let mut k = 0usize;
    while k < 3 {
        let mut r_base = 0usize;
        while r_base < 3 {
            let mut m = 0usize;
            while m < 5 {
                let mut g = 0usize;
                while g < 32 {
                    let fwd_phase = (r_base + k * (5 * g + m)) % 3;
                    let inv_phase = (3 - fwd_phase) % 3;
                    tbl[k][r_base][m][g] = inv_phase as i8;
                    g += 1;
                }
                m += 1;
            }
            r_base += 1;
        }
        k += 1;
    }
    tbl
}

/// ARM NEON planar inverse-bend table `[k_mod3][r_base][plane_m]` (16 bytes per entry):
/// Byte `g in 0..16` holds `(3 - ((r_base + k * (5*g + m)) mod 3)) mod 3`.
#[cfg(target_arch = "aarch64")]
const NEON_PLANAR_UNBEND_16: [[[[u8; 16]; 5]; 3]; 3] = build_neon_planar_unbend_16();

#[cfg(target_arch = "aarch64")]
const fn build_neon_planar_unbend_16() -> [[[[u8; 16]; 5]; 3]; 3] {
    let mut tbl = [[[[0u8; 16]; 5]; 3]; 3];
    let mut k = 0usize;
    while k < 3 {
        let mut r_base = 0usize;
        while r_base < 3 {
            let mut m = 0usize;
            while m < 5 {
                let mut g = 0usize;
                while g < 16 {
                    let fwd_phase = (r_base + k * (5 * g + m)) % 3;
                    let inv_phase = (3 - fwd_phase) % 3;
                    tbl[k][r_base][m][g] = inv_phase as u8;
                    g += 1;
                }
                m += 1;
            }
            r_base += 1;
        }
        k += 1;
    }
    tbl
}

/// AVX2 low (0..15) and high (16..26) base-3 residual lookup tables for `r2 in 0..26`.
#[cfg(target_arch = "x86_64")]
const AVX2_GEMV_R2_LO: [[i8; 32]; 3] = build_avx2_gemv_r2_lut(0);
#[cfg(target_arch = "x86_64")]
const AVX2_GEMV_R2_HI: [[i8; 32]; 3] = build_avx2_gemv_r2_lut(16);

#[cfg(target_arch = "x86_64")]
const fn build_avx2_gemv_r2_lut(base: usize) -> [[i8; 32]; 3] {
    let mut lut = [[0i8; 32]; 3];
    let mut idx = 0usize;
    while idx < 16 {
        let r = base + idx;
        if r < 27 {
            let t0 = (r % 3) as i8;
            let t1 = ((r / 3) % 3) as i8;
            let t2 = (r / 9) as i8;
            lut[0][idx] = t0;
            lut[0][16 + idx] = t0;
            lut[1][idx] = t1;
            lut[1][16 + idx] = t1;
            lut[2][idx] = t2;
            lut[2][16 + idx] = t2;
        }
        idx += 1;
    }
    lut
}

/// AVX2 5-way planar-to-interleaved `vpshufb` routing masks `[chunk_j][plane_m]`.
#[cfg(target_arch = "x86_64")]
const AVX2_GEMV_INTERLEAVE_MASKS: [[[i8; 32]; 5]; 5] = build_avx2_gemv_interleave_masks();

#[cfg(target_arch = "x86_64")]
const fn build_avx2_gemv_interleave_masks() -> [[[i8; 32]; 5]; 5] {
    let mut masks = [[[-1i8; 32]; 5]; 5];
    let mut chunk_j = 0usize;
    while chunk_j < 5 {
        let mut plane_m = 0usize;
        while plane_m < 5 {
            let mut p = 0usize;
            while p < 16 {
                let offset = chunk_j * 16 + p;
                if offset % 5 == plane_m {
                    let src_g = (offset / 5) as i8;
                    masks[chunk_j][plane_m][p] = src_g;
                    masks[chunk_j][plane_m][16 + p] = src_g;
                }
                p += 1;
            }
            plane_m += 1;
        }
        chunk_j += 1;
    }
    masks
}

/// ARM NEON 32-byte lookup tables for `r2 in 0..26 -> t0, t1, t2`.
#[cfg(target_arch = "aarch64")]
const NEON_GEMV_R2_LUT: [[u8; 32]; 3] = build_neon_gemv_r2_lut();

#[cfg(target_arch = "aarch64")]
const fn build_neon_gemv_r2_lut() -> [[u8; 32]; 3] {
    let mut lut = [[0u8; 32]; 3];
    let mut r = 0usize;
    while r < 27 {
        lut[0][r] = (r % 3) as u8;
        lut[1][r] = ((r / 3) % 3) as u8;
        lut[2][r] = (r / 9) as u8;
        r += 1;
    }
    lut
}

#[cfg(target_arch = "aarch64")]
const NEON_GEMV_IDX_0: [u8; 16] = build_neon_gemv_idx::<16>(0);
#[cfg(target_arch = "aarch64")]
const NEON_GEMV_IDX_1: [u8; 16] = build_neon_gemv_idx::<16>(16);
#[cfg(target_arch = "aarch64")]
const NEON_GEMV_IDX_2: [u8; 8] = build_neon_gemv_idx::<8>(32);

#[cfg(target_arch = "aarch64")]
const fn build_neon_gemv_idx<const LEN: usize>(start_m: usize) -> [u8; LEN] {
    let mut arr = [0u8; LEN];
    let mut idx = 0usize;
    while idx < LEN {
        let m = start_m + idx;
        arr[idx] = ((m % 5) * 8 + (m / 5)) as u8;
        idx += 1;
    }
    arr
}

// ============================================================================
// Public Fused Unpack-Unbend-GEMV Row Dispatcher & Scalar Reference
// ============================================================================

/// Multiplies a packed 5-trit row (`410` bytes for 2048-d, `820` bytes for 4096-d)
/// against a signed `i8` activation vector `activations`, reversing the `(k * i) mod 3`
/// cyclic bend in registers and returning the exact balanced-trit dot product:
///   `dot = \sum_{i=0}^{D-1} t_i * activations[i]` where `t_i in {-1, 0, +1}`.
pub fn fused_unpack_unbend_gemv_row_simd(packed_row: &[u8], activations: &[i8], bend_k: u8) -> i32 {
    let dim = activations.len();
    let required_bytes = (dim + 4) / 5;
    assert!(
        packed_row.len() >= required_bytes,
        "packed_row too short: need {required_bytes} bytes for {dim} activations, got {}",
        packed_row.len()
    );

    #[cfg(target_arch = "x86_64")]
    {
        if std::arch::is_x86_feature_detected!("avx2") {
            unsafe {
                return fused_unpack_unbend_gemv_row_avx2(packed_row, activations, bend_k);
            }
        }
    }

    #[cfg(target_arch = "aarch64")]
    {
        if std::arch::is_aarch64_feature_detected!("neon") {
            unsafe {
                return fused_unpack_unbend_gemv_row_neon(packed_row, activations, bend_k);
            }
        }
    }

    fused_unpack_unbend_gemv_row_scalar(packed_row, activations, bend_k)
}

/// Scalar reference implementation using the division-free ASM recurrence (`one add, one cond-sub`).
pub fn fused_unpack_unbend_gemv_row_scalar(
    packed_row: &[u8],
    activations: &[i8],
    bend_k: u8,
) -> i32 {
    fused_unpack_unbend_gemv_row_scalar_with_phase(packed_row, activations, bend_k, 0)
}

fn fused_unpack_unbend_gemv_row_scalar_with_phase(
    packed_row: &[u8],
    activations: &[i8],
    bend_k: u8,
    initial_phase_mod3: u8,
) -> i32 {
    let dim = activations.len();
    let k_mod3 = bend_k % 3;
    let mut r_mod3 = initial_phase_mod3 % 3;
    let mut acc: i32 = 0;

    let full_groups = dim / 5;
    let rem = dim % 5;

    for g in 0..full_groups {
        let b = packed_row[g];
        let bent_5 = if (b as usize) < 243 {
            UNPACK_5TRITS_LUT[b as usize]
        } else {
            [0; 5]
        };
        let act_chunk = &activations[g * 5..g * 5 + 5];

        for m in 0..5 {
            let mut unbent_u = bent_5[m] + 3 - r_mod3;
            if unbent_u >= 3 {
                unbent_u -= 3;
            }
            let bal_trit = (unbent_u as i32) - 1;
            acc += bal_trit * (act_chunk[m] as i32);

            r_mod3 += k_mod3;
            if r_mod3 >= 3 {
                r_mod3 -= 3;
            }
        }
    }

    if rem > 0 {
        let b = packed_row[full_groups];
        let bent_5 = if (b as usize) < 243 {
            UNPACK_5TRITS_LUT[b as usize]
        } else {
            [0; 5]
        };
        let base = full_groups * 5;
        for m in 0..rem {
            let mut unbent_u = bent_5[m] + 3 - r_mod3;
            if unbent_u >= 3 {
                unbent_u -= 3;
            }
            let bal_trit = (unbent_u as i32) - 1;
            acc += bal_trit * (activations[base + m] as i32);

            r_mod3 += k_mod3;
            if r_mod3 >= 3 {
                r_mod3 -= 3;
            }
        }
    }

    acc
}

// ============================================================================
// x86_64 AVX2 Fused Unpack-Unbend-GEMV Kernel (32 Packed Bytes -> 160 MACs/Iter)
// ============================================================================

#[cfg(target_arch = "x86_64")]
#[target_feature(enable = "avx2")]
unsafe fn fused_unpack_unbend_gemv_row_avx2(
    packed_row: &[u8],
    activations: &[i8],
    bend_k: u8,
) -> i32 {
    use std::arch::x86_64::*;

    let dim = activations.len();
    let mut in_offset = 0usize;
    let mut act_offset = 0usize;

    let k_mod3 = (bend_k % 3) as usize;
    let stride160_step_mod3 = (k_mod3 % 3) as u8;
    let mut r_base: u8 = 0;

    let max_valid = _mm256_set1_epi8(242u8 as i8);
    let zero = _mm256_setzero_si256();
    let v3 = _mm256_set1_epi8(3);
    let v15 = _mm256_set1_epi8(15);
    let v16 = _mm256_set1_epi8(16);
    let v27 = _mm256_set1_epi8(27);
    let v54 = _mm256_set1_epi8(54);
    let v81 = _mm256_set1_epi8(81);
    let v162 = _mm256_set1_epi8(162u8 as i8);

    let ones_u8 = _mm256_set1_epi8(1);
    let ones_i16 = _mm256_set1_epi16(1);

    let lut_lo_t0 = _mm256_loadu_si256(AVX2_GEMV_R2_LO[0].as_ptr() as *const __m256i);
    let lut_lo_t1 = _mm256_loadu_si256(AVX2_GEMV_R2_LO[1].as_ptr() as *const __m256i);
    let lut_lo_t2 = _mm256_loadu_si256(AVX2_GEMV_R2_LO[2].as_ptr() as *const __m256i);

    let lut_hi_t0 = _mm256_loadu_si256(AVX2_GEMV_R2_HI[0].as_ptr() as *const __m256i);
    let lut_hi_t1 = _mm256_loadu_si256(AVX2_GEMV_R2_HI[1].as_ptr() as *const __m256i);
    let lut_hi_t2 = _mm256_loadu_si256(AVX2_GEMV_R2_HI[2].as_ptr() as *const __m256i);

    let mut acc_i32 = _mm256_setzero_si256();

    while act_offset + 160 <= dim {
        let raw_b = _mm256_loadu_si256(packed_row.as_ptr().add(in_offset) as *const __m256i);
        let is_valid = _mm256_cmpeq_epi8(_mm256_min_epu8(raw_b, max_valid), raw_b);
        let b = _mm256_and_si256(raw_b, is_valid);

        let ge_81 = _mm256_cmpeq_epi8(_mm256_max_epu8(b, v81), b);
        let ge_162 = _mm256_cmpeq_epi8(_mm256_max_epu8(b, v162), b);
        let t4 = _mm256_sub_epi8(zero, _mm256_add_epi8(ge_81, ge_162));
        let r3 = _mm256_sub_epi8(
            _mm256_sub_epi8(b, _mm256_and_si256(ge_81, v81)),
            _mm256_and_si256(ge_162, v81),
        );

        let ge_27 = _mm256_cmpeq_epi8(_mm256_max_epu8(r3, v27), r3);
        let ge_54 = _mm256_cmpeq_epi8(_mm256_max_epu8(r3, v54), r3);
        let t3 = _mm256_sub_epi8(zero, _mm256_add_epi8(ge_27, ge_54));
        let r2 = _mm256_sub_epi8(
            _mm256_sub_epi8(r3, _mm256_and_si256(ge_27, v27)),
            _mm256_and_si256(ge_54, v27),
        );

        let idx_lo = _mm256_or_si256(r2, _mm256_cmpgt_epi8(r2, v15));
        let idx_hi = _mm256_sub_epi8(r2, v16);

        let t0 = _mm256_or_si256(
            _mm256_shuffle_epi8(lut_lo_t0, idx_lo),
            _mm256_shuffle_epi8(lut_hi_t0, idx_hi),
        );
        let t1 = _mm256_or_si256(
            _mm256_shuffle_epi8(lut_lo_t1, idx_lo),
            _mm256_shuffle_epi8(lut_hi_t1, idx_hi),
        );
        let t2 = _mm256_or_si256(
            _mm256_shuffle_epi8(lut_lo_t2, idx_lo),
            _mm256_shuffle_epi8(lut_hi_t2, idx_hi),
        );

        let mut planes = [t0, t1, t2, t3, t4];

        let r_idx = r_base as usize;
        for m in 0..5usize {
            let inv_p = _mm256_loadu_si256(
                AVX2_PLANAR_UNBEND_32[k_mod3][r_idx][m].as_ptr() as *const __m256i
            );
            let sum = _mm256_add_epi8(planes[m], inv_p);
            let ge3 = _mm256_cmpeq_epi8(_mm256_max_epu8(sum, v3), sum);
            planes[m] = _mm256_sub_epi8(sum, _mm256_and_si256(ge3, v3));
        }

        r_base += stride160_step_mod3;
        if r_base >= 3 {
            r_base -= 3;
        }

        let act_ptr = activations.as_ptr().add(act_offset);
        let mut sum_ux_16 = _mm256_setzero_si256();
        let mut sum_x_16 = _mm256_setzero_si256();

        for chunk_j in 0..5usize {
            let mut u_vec = _mm256_setzero_si256();
            for plane_m in 0..5usize {
                let mask = _mm256_loadu_si256(
                    AVX2_GEMV_INTERLEAVE_MASKS[chunk_j][plane_m].as_ptr() as *const __m256i,
                );
                u_vec = _mm256_or_si256(u_vec, _mm256_shuffle_epi8(planes[plane_m], mask));
            }

            let x_lo = _mm_loadu_si128(act_ptr.add(chunk_j * 16) as *const __m128i);
            let x_hi = _mm_loadu_si128(act_ptr.add(80 + chunk_j * 16) as *const __m128i);
            let x_vec = _mm256_set_m128i(x_hi, x_lo);

            sum_ux_16 = _mm256_add_epi16(sum_ux_16, _mm256_maddubs_epi16(u_vec, x_vec));
            sum_x_16 = _mm256_add_epi16(sum_x_16, _mm256_maddubs_epi16(ones_u8, x_vec));
        }

        let net_16 = _mm256_sub_epi16(sum_ux_16, sum_x_16);
        let net_32 = _mm256_madd_epi16(net_16, ones_i16);
        acc_i32 = _mm256_add_epi32(acc_i32, net_32);

        in_offset += 32;
        act_offset += 160;
    }

    let mut lanes = [0i32; 8];
    _mm256_storeu_si256(lanes.as_mut_ptr() as *mut __m256i, acc_i32);
    let mut total_dot: i32 = lanes.iter().sum();

    if act_offset < dim {
        total_dot += fused_unpack_unbend_gemv_row_scalar_with_phase(
            &packed_row[in_offset..],
            &activations[act_offset..],
            bend_k,
            r_base,
        );
    }

    total_dot
}

// ============================================================================
// AArch64 NEON Fused Unpack-Unbend-GEMV Kernel (16 Packed Bytes -> 80 MACs/Iter)
// ============================================================================

#[cfg(target_arch = "aarch64")]
#[target_feature(enable = "neon")]
unsafe fn fused_unpack_unbend_gemv_row_neon(
    packed_row: &[u8],
    activations: &[i8],
    bend_k: u8,
) -> i32 {
    use std::arch::aarch64::*;

    let dim = activations.len();
    let mut in_offset = 0usize;
    let mut act_offset = 0usize;

    let k_mod3 = (bend_k % 3) as usize;
    let stride80_step_mod3 = ((2 * k_mod3) % 3) as u8;
    let mut r_base: u8 = 0;

    let max_valid = vdupq_n_u8(242);
    let v3 = vdupq_n_u8(3);
    let v27 = vdupq_n_u8(27);
    let v54 = vdupq_n_u8(54);
    let v81 = vdupq_n_u8(81);
    let v162 = vdupq_n_u8(162);
    let ones_s8 = vdupq_n_s8(1);
    let zero_8 = vdup_n_u8(0);

    let lut_t0 = vld1q_u8_x2(NEON_GEMV_R2_LUT[0].as_ptr());
    let lut_t1 = vld1q_u8_x2(NEON_GEMV_R2_LUT[1].as_ptr());
    let lut_t2 = vld1q_u8_x2(NEON_GEMV_R2_LUT[2].as_ptr());

    let idx_0 = vld1q_u8(NEON_GEMV_IDX_0.as_ptr());
    let idx_1 = vld1q_u8(NEON_GEMV_IDX_1.as_ptr());
    let idx_2 = vld1_u8(NEON_GEMV_IDX_2.as_ptr());

    let mut acc_i32 = vdupq_n_s32(0);

    while act_offset + 80 <= dim {
        let raw_b = vld1q_u8(packed_row.as_ptr().add(in_offset));
        let b = vandq_u8(raw_b, vcleq_u8(raw_b, max_valid));

        let ge_81 = vcgeq_u8(b, v81);
        let ge_162 = vcgeq_u8(b, v162);
        let t4_u8 = vreinterpretq_u8_s8(vnegq_s8(vaddq_s8(
            vreinterpretq_s8_u8(ge_81),
            vreinterpretq_s8_u8(ge_162),
        )));
        let r3 = vsubq_u8(vsubq_u8(b, vandq_u8(ge_81, v81)), vandq_u8(ge_162, v81));

        let ge_27 = vcgeq_u8(r3, v27);
        let ge_54 = vcgeq_u8(r3, v54);
        let t3_u8 = vreinterpretq_u8_s8(vnegq_s8(vaddq_s8(
            vreinterpretq_s8_u8(ge_27),
            vreinterpretq_s8_u8(ge_54),
        )));
        let r2 = vsubq_u8(vsubq_u8(r3, vandq_u8(ge_27, v27)), vandq_u8(ge_54, v27));

        let t0_u8 = vqtbl2q_u8(lut_t0, r2);
        let t1_u8 = vqtbl2q_u8(lut_t1, r2);
        let t2_u8 = vqtbl2q_u8(lut_t2, r2);

        let mut planes = [t0_u8, t1_u8, t2_u8, t3_u8, t4_u8];

        let r_idx = r_base as usize;
        for m in 0..5usize {
            let inv_p = vld1q_u8(NEON_PLANAR_UNBEND_16[k_mod3][r_idx][m].as_ptr());
            let sum = vaddq_u8(planes[m], inv_p);
            let ge3 = vcgeq_u8(sum, v3);
            let unbent_u8 = vsubq_u8(sum, vandq_u8(ge3, v3));
            let bal_s8 = vsubq_s8(vreinterpretq_s8_u8(unbent_u8), ones_s8);
            planes[m] = vreinterpretq_u8_s8(bal_s8);
        }

        r_base += stride80_step_mod3;
        if r_base >= 3 {
            r_base -= 3;
        }

        let tbl_lo = uint8x16x3_t(
            vcombine_u8(vget_low_u8(planes[0]), vget_low_u8(planes[1])),
            vcombine_u8(vget_low_u8(planes[2]), vget_low_u8(planes[3])),
            vcombine_u8(vget_low_u8(planes[4]), zero_8),
        );
        let w_lo_0 = vreinterpretq_s8_u8(vqtbl3q_u8(tbl_lo, idx_0));
        let w_lo_1 = vreinterpretq_s8_u8(vqtbl3q_u8(tbl_lo, idx_1));
        let w_lo_2 = vreinterpret_s8_u8(vqtbl3_u8(tbl_lo, idx_2));

        let tbl_hi = uint8x16x3_t(
            vcombine_u8(vget_high_u8(planes[0]), vget_high_u8(planes[1])),
            vcombine_u8(vget_high_u8(planes[2]), vget_high_u8(planes[3])),
            vcombine_u8(vget_high_u8(planes[4]), zero_8),
        );
        let w_hi_0 = vreinterpretq_s8_u8(vqtbl3q_u8(tbl_hi, idx_0));
        let w_hi_1 = vreinterpretq_s8_u8(vqtbl3q_u8(tbl_hi, idx_1));
        let w_hi_2 = vreinterpret_s8_u8(vqtbl3_u8(tbl_hi, idx_2));

        let act_ptr = activations.as_ptr().add(act_offset);
        let x_lo_0 = vld1q_s8(act_ptr);
        let x_lo_1 = vld1q_s8(act_ptr.add(16));
        let x_lo_2 = vld1_s8(act_ptr.add(32));
        let x_hi_0 = vld1q_s8(act_ptr.add(40));
        let x_hi_1 = vld1q_s8(act_ptr.add(56));
        let x_hi_2 = vld1_s8(act_ptr.add(72));

        let mut sum_i16 = vmull_s8(vget_low_s8(w_lo_0), vget_low_s8(x_lo_0));
        sum_i16 = vmlal_s8(sum_i16, vget_high_s8(w_lo_0), vget_high_s8(x_lo_0));
        sum_i16 = vmlal_s8(sum_i16, vget_low_s8(w_lo_1), vget_low_s8(x_lo_1));
        sum_i16 = vmlal_s8(sum_i16, vget_high_s8(w_lo_1), vget_high_s8(x_lo_1));
        sum_i16 = vmlal_s8(sum_i16, w_lo_2, x_lo_2);

        sum_i16 = vmlal_s8(sum_i16, vget_low_s8(w_hi_0), vget_low_s8(x_hi_0));
        sum_i16 = vmlal_s8(sum_i16, vget_high_s8(w_hi_0), vget_high_s8(x_hi_0));
        sum_i16 = vmlal_s8(sum_i16, vget_low_s8(w_hi_1), vget_low_s8(x_hi_1));
        sum_i16 = vmlal_s8(sum_i16, vget_high_s8(w_hi_1), vget_high_s8(x_hi_1));
        sum_i16 = vmlal_s8(sum_i16, w_hi_2, x_hi_2);

        acc_i32 = vpadalq_s16(acc_i32, sum_i16);

        in_offset += 16;
        act_offset += 80;
    }

    let mut total_dot: i32 = vaddvq_s32(acc_i32);
    if act_offset < dim {
        total_dot += fused_unpack_unbend_gemv_row_scalar_with_phase(
            &packed_row[in_offset..],
            &activations[act_offset..],
            bend_k,
            r_base,
        );
    }

    total_dot
}

// ============================================================================
// Parallel LatentMoE Matrix-Vector Product (`LatentMoePackedMatrix` x `&[i8]`)
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LatentMoeGemvOutput {
    pub raw_integer_dots: Vec<i32>,
    pub scaled_outputs: Vec<f32>,
}

/// Executes a parallel zero-materialization GEMV across all rows of a `LatentMoePackedMatrix`
/// (`2048 x 2048` latent expert or `4096 x 2048` projection).
pub fn latent_moe_gemv_par_simd(
    matrix: &LatentMoePackedMatrix,
    activations: &[i8],
    activation_scale: f32,
) -> LatentMoeGemvOutput {
    assert_eq!(
        activations.len(),
        matrix.cols,
        "Activation vector length ({}) must match matrix.cols ({})",
        activations.len(),
        matrix.cols
    );

    let step_scale = 0.5 * ((matrix.centroids[2] as f32) - (matrix.centroids[0] as f32));
    let combined_scale = matrix.routing_scalar_alpha * step_scale * activation_scale;

    let raw_integer_dots: Vec<i32> = matrix
        .packed_data
        .par_chunks_exact(matrix.packed_row_stride_bytes)
        .enumerate()
        .map(|(row_idx, packed_row)| {
            let effective_k = if matrix.bend_k == 0 {
                0
            } else {
                ((matrix.bend_k as usize + row_idx) % 3) as u8
            };
            fused_unpack_unbend_gemv_row_simd(packed_row, activations, effective_k)
        })
        .collect();

    let scaled_outputs: Vec<f32> = raw_integer_dots
        .par_iter()
        .map(|&dot| (dot as f32) * combined_scale)
        .collect();

    LatentMoeGemvOutput {
        raw_integer_dots,
        scaled_outputs,
    }
}
