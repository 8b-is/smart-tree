use crate::mcp_combinatorics_llm::latent_moe_simd::{
    fused_unpack_unbend_gemv_row_simd, latent_moe_gemv_par_simd, LatentMoePackedMatrix,
};
use rayon::prelude::*;
use serde::{Deserialize, Serialize};
use std::cmp::Ordering;

/// Total routed experts per ERNIE 5.5 Flash MoE layer (layers 2..43).
pub const ERNIE55_ROUTED_EXPERTS: usize = 512;
/// Top-k routed experts activated per token.
pub const ERNIE55_TOP_K_EXPERTS: usize = 10;
/// Number of independent sigmoid routers in Multi-Router gating.
pub const DEFAULT_MULTI_ROUTERS: usize = 4;
/// Number of parallel residual streams in Manifold-Constrained Hyper-Connections (mHC).
pub const MHC_STREAMS: usize = 4;

// ============================================================================
// 1. Dynamic Symmetric INT8 Activation Quantizer (`f32 -> i8` in [-127, 127])
// ============================================================================

/// Dynamically quantizes an `f32` activation slice into signed `i8` in `[-127, 127]`
/// so `i8::MIN (-128)` is avoided and returns `(quantized_i8, dequant_scale)`.
pub fn quantize_activations_sym_i8(input: &[f32]) -> (Vec<i8>, f32) {
    let max_abs = input.iter().copied().map(f32::abs).fold(0.0f32, f32::max);

    if max_abs <= 1e-12 {
        return (vec![0i8; input.len()], 0.0);
    }

    let dequant_scale = max_abs / 127.0;
    let inv_scale = 127.0 / max_abs;

    let quant: Vec<i8> = input
        .iter()
        .map(|&x| (x * inv_scale).round().clamp(-127.0, 127.0) as i8)
        .collect();

    (quant, dequant_scale)
}

// ============================================================================
// 2. SiTU-GLU Activation & Pre-Up-Projection Stable RMSNorm (2048-d)
// ============================================================================

/// Evaluates the SiTU (Sigmoid-Tanh Linear Unit) activation:
/// `SiTU(u) = u * sigmoid(u) * tanh(softplus(u))`.
#[inline(always)]
pub fn situ_activation(u: f32) -> f32 {
    let u_clamped = u.clamp(-20.0, 20.0);
    let sig = 1.0 / (1.0 + (-u_clamped).exp());
    let softplus = u_clamped.max(0.0) + (-u_clamped.abs()).exp().ln_1p();
    u * sig * softplus.tanh()
}

/// Applies elementwise `SiTU-GLU(gate, up) = SiTU(gate[i]) * up[i]` across the 2048-d latent vector.
pub fn apply_situ_glu_latent(gate: &[f32], up: &[f32]) -> Vec<f32> {
    assert_eq!(gate.len(), up.len());
    gate.iter()
        .zip(up.iter())
        .map(|(&g, &u)| situ_activation(g) * u)
        .collect()
}

/// Applies ERNIE 5.5 Stable LatentMoE RMSNorm across the combined 2048-d latent output
/// immediately before the shared 2048 -> 4096 up-projection.
pub fn rms_norm_latent_in_place(latent: &mut [f32], gamma: &[f32], eps: f32) -> f32 {
    assert_eq!(latent.len(), gamma.len());
    let n = latent.len().max(1) as f32;
    let mean_sq: f32 = latent.iter().map(|&x| x * x).sum::<f32>() / n;
    let inv_rms = 1.0 / (mean_sq + eps).sqrt();

    for (x, &g) in latent.iter_mut().zip(gamma.iter()) {
        *x = (*x) * inv_rms * g;
    }
    inv_rms
}

// ============================================================================
// 3. Multi-Router Joint Sigmoid Gate + Aux-Loss-Free Bias Load Balancer
// ============================================================================

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MultiRouterSelection {
    pub selected_experts: Vec<usize>,
    pub expert_weights: Vec<f32>,
    pub joint_sigmoid_scores: Vec<f32>,
    pub per_router_scores: Vec<Vec<f32>>,
}

#[derive(Clone, Debug)]
pub struct MultiRouterGate {
    pub num_routers: usize,
    pub num_experts: usize,
    pub top_k: usize,
    pub router_projections: Vec<LatentMoePackedMatrix>,
    pub load_balance_bias: Vec<f32>,
    pub expert_learnable_scalars: Vec<f32>,
    pub expert_selection_counts: Vec<u64>,
    pub total_tokens_routed: u64,
}

impl MultiRouterGate {
    pub fn new(
        router_projections: Vec<LatentMoePackedMatrix>,
        expert_learnable_scalars: Vec<f32>,
        top_k: usize,
    ) -> Self {
        let num_routers = router_projections.len().max(1);
        let num_experts = router_projections
            .first()
            .map(|m| m.rows)
            .unwrap_or(ERNIE55_ROUTED_EXPERTS);

        assert_eq!(expert_learnable_scalars.len(), num_experts);

        Self {
            num_routers,
            num_experts,
            top_k: top_k.min(num_experts).max(1),
            router_projections,
            load_balance_bias: vec![0.0; num_experts],
            expert_learnable_scalars,
            expert_selection_counts: vec![0; num_experts],
            total_tokens_routed: 0,
        }
    }

    /// Computes joint sigmoid scores across all `R` routers using the fused 5-trit SIMD GEMV kernel,
    /// selects top-K experts via `s_e + b_e`, and returns the scalar-scaled weights `g_e`.
    pub fn route_token(&self, x_quant: &[i8], x_scale: f32) -> MultiRouterSelection {
        let router_probs: Vec<Vec<f32>> = self
            .router_projections
            .par_iter()
            .map(|proj| {
                let gemv = latent_moe_gemv_par_simd(proj, x_quant, x_scale);
                gemv.scaled_outputs
                    .into_iter()
                    .map(|logit| {
                        let z = logit.clamp(-15.0, 15.0);
                        1.0 / (1.0 + (-z).exp())
                    })
                    .collect()
            })
            .collect();

        let inv_r = 1.0 / (self.num_routers as f32);
        let mut joint_scores = vec![0.0f32; self.num_experts];
        for r_vec in &router_probs {
            for (e, &p) in r_vec.iter().enumerate() {
                joint_scores[e] += p * inv_r;
            }
        }

        let mut ranked: Vec<(usize, f32)> = (0..self.num_experts)
            .map(|e| (e, joint_scores[e] + self.load_balance_bias[e]))
            .collect();

        ranked.sort_unstable_by(|a, b| {
            b.1.partial_cmp(&a.1)
                .unwrap_or(Ordering::Equal)
                .then_with(|| a.0.cmp(&b.0))
        });
        ranked.truncate(self.top_k);

        let selected_experts: Vec<usize> = ranked.iter().map(|&(e, _)| e).collect();
        let selected_unbiased: Vec<f32> =
            selected_experts.iter().map(|&e| joint_scores[e]).collect();

        let sum_unbiased: f32 = selected_unbiased.iter().sum::<f32>().max(1e-12);
        let expert_weights: Vec<f32> = selected_experts
            .iter()
            .zip(selected_unbiased.iter())
            .map(|(&e, &s)| self.expert_learnable_scalars[e] * (s / sum_unbiased))
            .collect();

        let per_router_scores: Vec<Vec<f32>> = router_probs
            .into_iter()
            .map(|r_vec| selected_experts.iter().map(|&e| r_vec[e]).collect())
            .collect();

        MultiRouterSelection {
            selected_experts,
            expert_weights,
            joint_sigmoid_scores: selected_unbiased,
            per_router_scores,
        }
    }

    /// Updates the auxiliary-loss-free bias vector `b_e` based on observed expert load vs. target load.
    pub fn update_load_balance_biases(&mut self, selected_experts: &[usize], bias_step: f32) {
        self.total_tokens_routed += 1;
        for &e in selected_experts {
            if e < self.num_experts {
                self.expert_selection_counts[e] += 1;
            }
        }

        let target_count =
            (self.total_tokens_routed as f32) * (self.top_k as f32 / (self.num_experts as f32));

        for (b, &cnt) in self
            .load_balance_bias
            .iter_mut()
            .zip(self.expert_selection_counts.iter())
        {
            let diff = target_count - (cnt as f32);
            if diff > 0.5 {
                *b += bias_step;
            } else if diff < -0.5 {
                *b -= bias_step;
            }
        }
    }
}

// ============================================================================
// 4. Single 2048-d Latent Expert (Gate + Up + SiTU-GLU + Down in 5-Trit SIMD)
// ============================================================================

#[derive(Clone, Debug)]
pub struct LatentExpert5Trit {
    pub expert_id: usize,
    pub gate_proj: LatentMoePackedMatrix,
    pub up_proj: LatentMoePackedMatrix,
    pub down_proj: LatentMoePackedMatrix,
}

impl LatentExpert5Trit {
    /// Evaluates the expert entirely inside the latent space (`d_latent -> d_latent`)
    /// using `fused_unpack_unbend_gemv_row_simd` without materializing unpacked weights.
    pub fn forward_latent_from_prequantized(
        &self,
        h_lat_quant: &[i8],
        h_lat_scale: f32,
    ) -> Vec<f32> {
        let d_lat = self.gate_proj.rows;
        assert_eq!(h_lat_quant.len(), self.gate_proj.cols);

        let gate_step =
            0.5 * ((self.gate_proj.centroids[2] as f32) - (self.gate_proj.centroids[0] as f32));
        let gate_scale = self.gate_proj.routing_scalar_alpha * gate_step * h_lat_scale;

        let up_step =
            0.5 * ((self.up_proj.centroids[2] as f32) - (self.up_proj.centroids[0] as f32));
        let up_scale = self.up_proj.routing_scalar_alpha * up_step * h_lat_scale;

        let mut intermediate = vec![0.0f32; d_lat];
        let stride = self.gate_proj.packed_row_stride_bytes;

        for r in 0..d_lat {
            let row_slice = r * stride..(r + 1) * stride;
            let k_gate = if self.gate_proj.bend_k == 0 {
                0
            } else {
                ((self.gate_proj.bend_k as usize + r) % 3) as u8
            };
            let k_up = if self.up_proj.bend_k == 0 {
                0
            } else {
                ((self.up_proj.bend_k as usize + r) % 3) as u8
            };

            let dot_g = fused_unpack_unbend_gemv_row_simd(
                &self.gate_proj.packed_data[row_slice.clone()],
                h_lat_quant,
                k_gate,
            );
            let dot_u = fused_unpack_unbend_gemv_row_simd(
                &self.up_proj.packed_data[row_slice],
                h_lat_quant,
                k_up,
            );

            let g_val = (dot_g as f32) * gate_scale;
            let u_val = (dot_u as f32) * up_scale;
            intermediate[r] = situ_activation(g_val) * u_val;
        }

        let (inter_quant, inter_scale) = quantize_activations_sym_i8(&intermediate);
        let down_step =
            0.5 * ((self.down_proj.centroids[2] as f32) - (self.down_proj.centroids[0] as f32));
        let down_scale = self.down_proj.routing_scalar_alpha * down_step * inter_scale;

        let mut out_latent = vec![0.0f32; self.down_proj.rows];
        for (r, out_val) in out_latent.iter_mut().enumerate() {
            let row_slice = r * stride..(r + 1) * stride;
            let k_down = if self.down_proj.bend_k == 0 {
                0
            } else {
                ((self.down_proj.bend_k as usize + r) % 3) as u8
            };
            let dot_d = fused_unpack_unbend_gemv_row_simd(
                &self.down_proj.packed_data[row_slice],
                &inter_quant,
                k_down,
            );
            *out_val = (dot_d as f32) * down_scale;
        }

        out_latent
    }

    pub fn packed_bytes(&self) -> usize {
        self.gate_proj.packed_data.len()
            + self.up_proj.packed_data.len()
            + self.down_proj.packed_data.len()
    }
}

// ============================================================================
// 5. Complete ERNIE 5.5 Multi-Router Stable LatentMoE + 4-Stream mHC Block
// ============================================================================

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct LatentMoeForwardTelemetry {
    pub hidden_dim: usize,
    pub latent_dim: usize,
    pub total_routed_experts: usize,
    pub active_routed_experts: usize,
    pub shared_experts: usize,
    pub routing: MultiRouterSelection,
    pub active_packed_bytes_read: usize,
    pub total_layer_packed_bytes: usize,
    pub active_memory_fraction: f64,
    pub pre_up_proj_inv_rms: f32,
    pub mhc_l1_norm_before: f64,
    pub mhc_l1_norm_after_mixing: f64,
}

pub struct Ernie55LatentMoeLayer {
    pub hidden_dim: usize,
    pub latent_dim: usize,
    pub router: MultiRouterGate,
    pub shared_down_proj: LatentMoePackedMatrix,
    pub shared_expert: LatentExpert5Trit,
    pub routed_experts: Vec<LatentExpert5Trit>,
    pub pre_up_rmsnorm_gamma: Vec<f32>,
    pub rmsnorm_eps: f32,
    pub shared_up_proj: LatentMoePackedMatrix,
    pub mhc_mixing_matrix: [[f32; MHC_STREAMS]; MHC_STREAMS],
    pub mhc_read_weights: [f32; MHC_STREAMS],
    pub mhc_write_weights: [f32; MHC_STREAMS],
}

impl Ernie55LatentMoeLayer {
    /// Executes the full ERNIE 5.5 Multi-Router Stable LatentMoE forward pass on a single
    /// `hidden_dim` (`4096-d`) hidden state vector `x_in`, returning `(y_out, telemetry)`.
    pub fn forward_hidden(&self, x_in: &[f32]) -> (Vec<f32>, LatentMoeForwardTelemetry) {
        assert_eq!(x_in.len(), self.hidden_dim);

        let (x_quant, x_scale) = quantize_activations_sym_i8(x_in);

        let routing = self.router.route_token(&x_quant, x_scale);

        let down_gemv = latent_moe_gemv_par_simd(&self.shared_down_proj, &x_quant, x_scale);
        let h_latent = down_gemv.scaled_outputs;
        assert_eq!(h_latent.len(), self.latent_dim);

        let (h_lat_quant, h_lat_scale) = quantize_activations_sym_i8(&h_latent);

        let (shared_out, routed_outputs): (Vec<f32>, Vec<Vec<f32>>) = rayon::join(
            || {
                self.shared_expert
                    .forward_latent_from_prequantized(&h_lat_quant, h_lat_scale)
            },
            || {
                routing
                    .selected_experts
                    .par_iter()
                    .map(|&e_idx| {
                        self.routed_experts[e_idx]
                            .forward_latent_from_prequantized(&h_lat_quant, h_lat_scale)
                    })
                    .collect()
            },
        );

        let mut combined_latent = shared_out;
        for (expert_vec, &weight) in routed_outputs.iter().zip(routing.expert_weights.iter()) {
            for (dst, &val) in combined_latent.iter_mut().zip(expert_vec.iter()) {
                *dst += weight * val;
            }
        }

        let inv_rms = rms_norm_latent_in_place(
            &mut combined_latent,
            &self.pre_up_rmsnorm_gamma,
            self.rmsnorm_eps,
        );

        let (norm_lat_quant, norm_lat_scale) = quantize_activations_sym_i8(&combined_latent);
        let up_gemv =
            latent_moe_gemv_par_simd(&self.shared_up_proj, &norm_lat_quant, norm_lat_scale);
        let y_out = up_gemv.scaled_outputs;

        let router_bytes: usize = self
            .router
            .router_projections
            .iter()
            .map(|m| m.packed_data.len())
            .sum();
        let down_up_bytes =
            self.shared_down_proj.packed_data.len() + self.shared_up_proj.packed_data.len();
        let shared_exp_bytes = self.shared_expert.packed_bytes();
        let active_routed_bytes: usize = routing
            .selected_experts
            .iter()
            .map(|&e| self.routed_experts[e].packed_bytes())
            .sum();
        let all_routed_bytes: usize = self.routed_experts.iter().map(|e| e.packed_bytes()).sum();

        let active_bytes = router_bytes + down_up_bytes + shared_exp_bytes + active_routed_bytes;
        let total_bytes = router_bytes + down_up_bytes + shared_exp_bytes + all_routed_bytes;

        let telemetry = LatentMoeForwardTelemetry {
            hidden_dim: self.hidden_dim,
            latent_dim: self.latent_dim,
            total_routed_experts: self.routed_experts.len(),
            active_routed_experts: routing.selected_experts.len(),
            shared_experts: 1,
            routing,
            active_packed_bytes_read: active_bytes,
            total_layer_packed_bytes: total_bytes,
            active_memory_fraction: (active_bytes as f64) / (total_bytes.max(1) as f64),
            pre_up_proj_inv_rms: inv_rms,
            mhc_l1_norm_before: 0.0,
            mhc_l1_norm_after_mixing: 0.0,
        };

        (y_out, telemetry)
    }

    /// Executes the full layer over the 4 parallel Manifold-Constrained Hyper-Connection (`mHC`)
    /// residual streams `streams: &mut [[f32; d_model]; 4]`.
    pub fn forward_mhc_streams(
        &self,
        streams: &mut [Vec<f32>; MHC_STREAMS],
    ) -> LatentMoeForwardTelemetry {
        for s in streams.iter() {
            assert_eq!(s.len(), self.hidden_dim);
        }

        let mut l1_before = 0.0f64;
        for d in 0..self.hidden_dim {
            let col_sum: f32 = (0..MHC_STREAMS).map(|s| streams[s][d]).sum();
            l1_before += col_sum.abs() as f64;
        }

        let mut mixed_streams: [Vec<f32>; MHC_STREAMS] = [
            vec![0.0; self.hidden_dim],
            vec![0.0; self.hidden_dim],
            vec![0.0; self.hidden_dim],
            vec![0.0; self.hidden_dim],
        ];
        let mut x_in = vec![0.0f32; self.hidden_dim];

        for d in 0..self.hidden_dim {
            let s_col = [streams[0][d], streams[1][d], streams[2][d], streams[3][d]];
            for i in 0..MHC_STREAMS {
                let mut m_val = 0.0f32;
                for j in 0..MHC_STREAMS {
                    m_val += self.mhc_mixing_matrix[i][j] * s_col[j];
                }
                mixed_streams[i][d] = m_val;
                x_in[d] += self.mhc_read_weights[i] * m_val;
            }
        }

        let mut l1_after_mix = 0.0f64;
        for d in 0..self.hidden_dim {
            let col_sum: f32 = (0..MHC_STREAMS).map(|s| mixed_streams[s][d]).sum();
            l1_after_mix += col_sum.abs() as f64;
        }

        let (moe_delta, mut telemetry) = self.forward_hidden(&x_in);
        telemetry.mhc_l1_norm_before = l1_before;
        telemetry.mhc_l1_norm_after_mixing = l1_after_mix;

        for i in 0..MHC_STREAMS {
            let beta = self.mhc_write_weights[i];
            for d in 0..self.hidden_dim {
                streams[i][d] = mixed_streams[i][d] + beta * moe_delta[d];
            }
        }

        telemetry
    }
}
