//! # Mega-Scale Hidden Dimension (1M+) & Dyadic Geometric Attention Cascade
//!
//! Implements the generalized architecture:
//! - **Hidden size**: $1\text{M}+$ ($D \ge 1,048,576$) sparse-latent representation.
//! - **Layered attention progression**:
//!   $$1 \to 3 \to 7 \to 11 \to 21 \dots \to N = C - [\text{MAX}n - 3] \to \infty/pq$$
//! - Shannon entropy bottleneck & rational $p/q$ channel-capacity bound.

use crate::felix_asm::{
    felix_binary_gcd_u64, felix_normalize64_f32, felix_quant_tern_pq_gemm32_i32,
    felix_quant_tern_pq_pack32,
};
use serde::{Deserialize, Serialize};

/// Canonical million-plus hidden dimension ($2^{20} = 1,048,576$).
pub const MEGA_HIDDEN_DIM_1M: usize = 1_048_576;

/// Rational p/q Channel Capacity specification.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct RationalPqChannel {
    pub p: u64,
    pub q: u64,
    pub capacity_bits: f64,
    pub shannon_entropy: f64,
    pub compression_ratio: f64,
}

impl RationalPqChannel {
    pub fn new(p: u64, q: u64, snr_linear: f64) -> Self {
        let q_raw = q.max(1);
        let g = felix_binary_gcd_u64(p, q_raw).max(1);
        let p_red = p / g;
        let q_safe = (q_raw / g).max(1);
        let ratio = (p_red as f64) / (q_safe as f64);
        // Shannon-Hartley capacity C = B * log2(1 + S/N)
        let capacity_bits = ratio * (1.0 + snr_linear.max(0.0)).log2();
        let p_prob = (ratio / (1.0 + ratio)).clamp(1e-12, 1.0 - 1e-12);
        let shannon_entropy = -p_prob * p_prob.log2() - (1.0 - p_prob) * (1.0 - p_prob).log2();
        let compression_ratio =
            (MEGA_HIDDEN_DIM_1M as f64) / ((p_red.max(1) as f64) * ratio.max(1e-6));

        Self {
            p: p_red,
            q: q_safe,
            capacity_bits,
            shannon_entropy,
            compression_ratio,
        }
    }
}

/// A stage in the layered geometric attention progression:
/// $1 \to 3 \to 7 \to 11 \to 21 \dots \to N = C - [\text{MAX}n - 3] \to \infty/pq$
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct AttentionCascadeStage {
    pub stage_index: usize,
    pub step_stride: usize,
    pub effective_subspace_dim: usize,
    pub active_heads: usize,
    pub shannon_divergence: f64,
}

/// The multi-tier Attention Cascade Engine supporting $1\text{M}+$ hidden size.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MegaHiddenCascade {
    pub hidden_size: usize,
    pub stages: Vec<AttentionCascadeStage>,
    pub channel_pq: RationalPqChannel,
    pub terminal_n_layer: usize,
}

impl MegaHiddenCascade {
    /// Constructs the layered attention cascade from 1 to $N = C - [\text{MAX}n - 3]$.
    pub fn new(
        hidden_size: usize,
        channel_capacity_c: usize,
        max_n: usize,
        p: u64,
        q: u64,
    ) -> Self {
        let actual_hidden = hidden_size.max(MEGA_HIDDEN_DIM_1M);
        let terminal_n = channel_capacity_c.saturating_sub(max_n.saturating_sub(3));
        let bound_n = terminal_n.max(1);

        // Sequence: 1, 3, 7, 11, 21 ... up to bound_n
        let mut strides: Vec<usize> = [1, 3, 7, 11, 21]
            .into_iter()
            .filter(|&s| s <= bound_n)
            .collect();
        if strides.is_empty() {
            strides.push(1);
        }
        let mut curr = *strides.last().unwrap_or(&1);
        while curr < bound_n {
            curr = curr * 2 + 1; // generalized dyadic odd sequence
            strides.push(curr.min(bound_n));
            if curr >= bound_n {
                break;
            }
        }

        let channel_pq = RationalPqChannel::new(p, q, 31.62); // 15 dB SNR
        let mut stages = Vec::with_capacity(strides.len());

        for (idx, &stride) in strides.iter().enumerate() {
            let active_heads = stride.max(1);
            let effective_subspace_dim = (actual_hidden / active_heads).max(64);
            let p_step = (idx + 1) as f64 / (strides.len() as f64 + 1.0);
            let shannon_divergence = -p_step * p_step.log2();

            stages.push(AttentionCascadeStage {
                stage_index: idx,
                step_stride: stride,
                effective_subspace_dim,
                active_heads,
                shannon_divergence,
            });
        }

        Self {
            hidden_size: actual_hidden,
            stages,
            channel_pq,
            terminal_n_layer: terminal_n,
        }
    }

    /// Projects a sparse $1\text{M}+$ hidden activation across the cascade into $p/q$ channel equilibrium.
    pub fn project_sparse_cascade(&self, nonzero_coords: &[(usize, f32)]) -> Vec<f32> {
        let out_dim = 64;
        let mut out = vec![0.0f32; out_dim];
        if nonzero_coords.is_empty() {
            return out;
        }

        let scale = 1.0 / (nonzero_coords.len() as f32).sqrt();
        let pq_mod = (self.channel_pq.p ^ self.channel_pq.q).max(1) as usize;

        for &(idx, val) in nonzero_coords {
            let target_bucket = (idx ^ pq_mod) % out_dim;
            out[target_bucket] += val * scale;
        }

        // Apply Shannon channel compression normalization
        for x in &mut out {
            *x = (*x * (self.channel_pq.shannon_entropy as f32)).tanh();
        }

        out
    }

    /// Projects a sparse $1\text{M}+$ hidden activation into a hardware-normalized 64-lane wave
    /// and packs it into two 7-byte `quantTernP/Q` Block32 packets (`14 bytes` total for 64 trits),
    /// returning `(wave64, [wire_lo, wire_hi], resonance_dot_i32)`.
    pub fn project_sparse_cascade_quant_tern_pq(
        &self,
        nonzero_coords: &[(usize, f32)],
    ) -> ([f32; 64], [[u8; 7]; 2], i32) {
        let projected = self.project_sparse_cascade(nonzero_coords);
        let mut wave64 = [0.0f32; 64];
        wave64.copy_from_slice(&projected[..64]);
        let _norm = felix_normalize64_f32(&mut wave64);

        let mut trits_lo = [0i8; 32];
        let mut trits_hi = [0i8; 32];
        let mut acts_lo = [0i16; 32];
        let mut acts_hi = [0i16; 32];

        for i in 0..32 {
            let v_lo = wave64[i];
            let v_hi = wave64[i + 32];
            acts_lo[i] = (v_lo * 1000.0).round().clamp(-32000.0, 32000.0) as i16;
            acts_hi[i] = (v_hi * 1000.0).round().clamp(-32000.0, 32000.0) as i16;
            trits_lo[i] = if v_lo > 0.05 {
                1
            } else if v_lo < -0.05 {
                -1
            } else {
                0
            };
            trits_hi[i] = if v_hi > 0.05 {
                1
            } else if v_hi < -0.05 {
                -1
            } else {
                0
            };
        }

        let q_nibble = ((self.channel_pq.p ^ self.channel_pq.q) & 0x0F) as u8;
        let wire_lo = felix_quant_tern_pq_pack32(&trits_lo, q_nibble);
        let wire_hi = felix_quant_tern_pq_pack32(&trits_hi, q_nibble);
        let (dot_lo, _) = felix_quant_tern_pq_gemm32_i32(&acts_lo, &wire_lo);
        let (dot_hi, _) = felix_quant_tern_pq_gemm32_i32(&acts_hi, &wire_hi);
        (wave64, [wire_lo, wire_hi], dot_lo.wrapping_add(dot_hi))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mega_hidden_1m_cascade_progression() {
        let cascade = MegaHiddenCascade::new(1_048_576, 128, 43, 65537, 4096);
        assert!(cascade.hidden_size >= 1_048_576);
        assert_eq!(cascade.stages[0].step_stride, 1);
        assert_eq!(cascade.stages[1].step_stride, 3);
        assert_eq!(cascade.stages[2].step_stride, 7);
        assert_eq!(cascade.stages[3].step_stride, 11);
        assert_eq!(cascade.stages[4].step_stride, 21);
        assert!(cascade.terminal_n_layer > 0);
        assert!(cascade.channel_pq.capacity_bits > 0.0);
    }

    #[test]
    fn test_sparse_cascade_projection_bounded() {
        let cascade = MegaHiddenCascade::new(1_048_576, 128, 43, 65536, 4096);
        // Verify GCD reduction: 65536 / 4096 -> 16 / 1
        assert_eq!(cascade.channel_pq.p, 16);
        assert_eq!(cascade.channel_pq.q, 1);
        let sparse_coords = vec![(0, 1.5), (1024, -2.0), (1_000_000, 3.2)];
        let projected = cascade.project_sparse_cascade(&sparse_coords);
        assert_eq!(projected.len(), 64);
        for &val in &projected {
            assert!(val.abs() <= 1.0);
        }
        let (wave64, wires, dot) = cascade.project_sparse_cascade_quant_tern_pq(&sparse_coords);
        assert_eq!(wave64.len(), 64);
        assert_eq!(wires[0].len(), 7);
        assert_eq!(wires[1].len(), 7);
        assert!(dot > 0);
    }
}
