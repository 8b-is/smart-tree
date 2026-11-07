use crate::{
    latent_moe_layer::{quantize_activations_sym_i8, MHC_STREAMS},
    mcp_combinatorics_llm::latent_moe_simd::{latent_moe_gemv_par_simd, LatentMoePackedMatrix},
};
use rayon::prelude::*;
use serde::{Deserialize, Serialize};
use std::cmp::Ordering;

/// Total Transformer layers in ERNIE 5.5 Flash Base.
pub const ERNIE55_TOTAL_LAYERS: usize = 43;
/// Number of attention heads per layer.
pub const ERNIE55_NUM_HEADS: usize = 64;
/// Low-rank query latent dimension (d_model / 4 = 4096 / 4 = 1024).
pub const ERNIE55_QUERY_LATENT_DIM: usize = 1024;
/// Packed 5-trit byte stride for a 1024-d query-latent gate row: ceil(1024 / 5) = 205 bytes.
pub const PACKED_QUERY_LATENT_ROW_BYTES: usize = ERNIE55_QUERY_LATENT_DIM.div_ceil(5);

/// Head dimension for full-context MLA layers (1 in 7 layers).
pub const MLA_HEAD_DIM: usize = 256;
/// Head dimension for 128x Heavily Compressed Attention (HCA) layers (6 in 7 layers).
pub const HCA_HEAD_DIM: usize = 512;
/// Token block compression factor in HCA layers (128 tokens -> 1 cached summary entry).
pub const HCA_COMPRESSION_BLOCK_SIZE: usize = 128;
/// Default Virtual Head Attention (VHA) low-rank cross-head mixing rank.
pub const DEFAULT_VHA_RANK: usize = 8;
/// Default DSA lightweight indexer dimension for top-k token scoring in MLA layers.
pub const DEFAULT_DSA_INDEX_DIM: usize = 64;

// ============================================================================
// 1. 43-Layer 1:6 Hybrid MLA + HCA Schedule Classifier
// ============================================================================

#[derive(Copy, Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AttentionLayerKind {
    Hca128xCompressed512,
    MlaDsaSparse256,
}

impl AttentionLayerKind {
    #[inline]
    pub const fn for_layer(layer_idx: usize) -> Self {
        if (layer_idx + 1).is_multiple_of(7) {
            Self::MlaDsaSparse256
        } else {
            Self::Hca128xCompressed512
        }
    }

    #[inline]
    pub const fn head_dim(self) -> usize {
        match self {
            Self::Hca128xCompressed512 => HCA_HEAD_DIM,
            Self::MlaDsaSparse256 => MLA_HEAD_DIM,
        }
    }
}

// ============================================================================
// 2. Low-Rank Query-Latent Gate (1024-d Query Latent -> 75% Fewer Gate Weights)
// ============================================================================

#[derive(Clone, Debug)]
pub struct LowRankQueryGate {
    pub w_q_down: LatentMoePackedMatrix,
    pub w_gate_from_latent: LatentMoePackedMatrix,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct LowRankGateTelemetry {
    pub query_latent_dim: usize,
    pub gate_output_dim: usize,
    pub packed_gate_bytes: usize,
    pub full_hidden_gate_bytes_equivalent: usize,
    pub weight_reduction_ratio_pct: f64,
}

impl LowRankQueryGate {
    /// Computes both the reusable low-rank query latent `q_latent` (`1024-d`) and the
    /// sigmoid attention gate `g = \sigma(W_gate * q_latent)` using 5-trit SIMD GEMV.
    pub fn compute_query_latent_and_gate(
        &self,
        x_quant: &[i8],
        x_scale: f32,
    ) -> (Vec<f32>, Vec<f32>, LowRankGateTelemetry) {
        let q_lat_gemv = latent_moe_gemv_par_simd(&self.w_q_down, x_quant, x_scale);
        let q_latent = q_lat_gemv.scaled_outputs;

        let (qlat_quant, qlat_scale) = quantize_activations_sym_i8(&q_latent);
        let gate_gemv = latent_moe_gemv_par_simd(&self.w_gate_from_latent, &qlat_quant, qlat_scale);

        let gate_probs: Vec<f32> = gate_gemv
            .scaled_outputs
            .into_iter()
            .map(|logit| {
                let z = logit.clamp(-15.0, 15.0);
                1.0 / (1.0 + (-z).exp())
            })
            .collect();

        let gate_rows = self.w_gate_from_latent.rows;
        let packed_gate_bytes = self.w_gate_from_latent.packed_data.len();
        let full_equiv_bytes = gate_rows * self.w_q_down.cols.div_ceil(5);
        let reduction_pct = if full_equiv_bytes > 0 {
            (1.0 - (packed_gate_bytes as f64) / (full_equiv_bytes as f64)) * 100.0
        } else {
            0.0
        };

        let telemetry = LowRankGateTelemetry {
            query_latent_dim: q_latent.len(),
            gate_output_dim: gate_probs.len(),
            packed_gate_bytes,
            full_hidden_gate_bytes_equivalent: full_equiv_bytes,
            weight_reduction_ratio_pct: reduction_pct,
        };

        (q_latent, gate_probs, telemetry)
    }
}

// ============================================================================
// 3. Virtual Head Attention (VHA): Fused Low-Rank Cross-Head Mixer + Skip
// ============================================================================

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct VirtualHeadAttentionMixer {
    pub num_heads: usize,
    pub rank: usize,
    pub u_proj: Vec<Vec<f32>>,
    pub v_proj: Vec<Vec<f32>>,
}

impl VirtualHeadAttentionMixer {
    pub fn new_identity_plus_low_rank(num_heads: usize, rank: usize, coupling_scale: f32) -> Self {
        let r = rank.max(1);
        let mut u_proj = vec![vec![0.0f32; r]; num_heads];
        let mut v_proj = vec![vec![0.0f32; r]; num_heads];

        for h in 0..num_heads {
            for k in 0..r {
                let phase = ((h * 7 + k * 13) as f32) * 0.17;
                u_proj[h][k] = coupling_scale * phase.sin();
                v_proj[h][k] = coupling_scale * phase.cos();
            }
        }

        Self {
            num_heads,
            rank: r,
            u_proj,
            v_proj,
        }
    }

    /// Applies `(I_H + U * V^T)` across `head_outputs` (`num_heads` vectors of length `head_dim`).
    pub fn mix_heads_in_place(&self, head_outputs: &mut [Vec<f32>]) {
        assert_eq!(head_outputs.len(), self.num_heads);
        let Some(head_dim) = head_outputs.first().map(|v| v.len()) else {
            return;
        };

        let mut virtual_bottleneck = vec![vec![0.0f32; head_dim]; self.rank];
        for (j, head_vec) in head_outputs.iter().enumerate() {
            for (r, bottleneck_row) in virtual_bottleneck.iter_mut().enumerate().take(self.rank) {
                let v_jr = self.v_proj[j][r];
                if v_jr.abs() <= 1e-12 {
                    continue;
                }
                for (dst, &val) in bottleneck_row.iter_mut().zip(head_vec.iter()) {
                    *dst += v_jr * val;
                }
            }
        }

        for (h, head_vec) in head_outputs.iter_mut().enumerate() {
            for (r, bottleneck_row) in virtual_bottleneck.iter().enumerate().take(self.rank) {
                let u_hr = self.u_proj[h][r];
                if u_hr.abs() <= 1e-12 {
                    continue;
                }
                for (dst, &b_val) in head_vec.iter_mut().zip(bottleneck_row.iter()) {
                    *dst += u_hr * b_val;
                }
            }
        }
    }
}

// ============================================================================
// 4. HCA 128x Heavily Compressed KV Cache (512-d Heads)
// ============================================================================

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct KvLatentEntry {
    pub kv_latent: Vec<f32>,
}

#[derive(Clone, Debug)]
pub struct HcaCompressedKvCache {
    pub block_size: usize,
    pub max_recent_window: usize,
    pub kv_latent_dim: usize,
    pub compressed_blocks: Vec<KvLatentEntry>,
    staging_block: Vec<KvLatentEntry>,
    pub recent_window: Vec<KvLatentEntry>,
    pub total_tokens_ingested: usize,
}

impl HcaCompressedKvCache {
    pub fn new(block_size: usize, max_recent_window: usize, kv_latent_dim: usize) -> Self {
        let bs = block_size.max(1);
        Self {
            block_size: bs,
            max_recent_window: max_recent_window.max(1),
            kv_latent_dim,
            compressed_blocks: Vec::new(),
            staging_block: Vec::with_capacity(bs),
            recent_window: Vec::with_capacity(max_recent_window.max(1)),
            total_tokens_ingested: 0,
        }
    }

    pub fn push_token_kv(&mut self, kv_latent: Vec<f32>) {
        assert_eq!(kv_latent.len(), self.kv_latent_dim);
        self.total_tokens_ingested += 1;

        self.staging_block.push(KvLatentEntry {
            kv_latent: kv_latent.clone(),
        });

        if self.recent_window.len() >= self.max_recent_window {
            self.recent_window.remove(0);
        }
        self.recent_window.push(KvLatentEntry { kv_latent });

        if self.staging_block.len() == self.block_size {
            let mut summary = vec![0.0f32; self.kv_latent_dim];
            let inv_block = 1.0 / (self.block_size as f32);

            for entry in self.staging_block.drain(..) {
                for (dst, val) in summary.iter_mut().zip(entry.kv_latent) {
                    *dst += val * inv_block;
                }
            }

            let mean_sq: f32 =
                summary.iter().map(|&x| x * x).sum::<f32>() / (self.kv_latent_dim as f32);
            let scale = 1.0 / (mean_sq + 1e-6).sqrt().max(1.0);
            for v in &mut summary {
                *v *= scale;
            }

            self.compressed_blocks
                .push(KvLatentEntry { kv_latent: summary });
        }
    }

    #[inline]
    pub fn active_cached_entries(&self) -> usize {
        self.compressed_blocks.len() + self.recent_window.len()
    }

    #[inline]
    pub fn compression_ratio(&self) -> f64 {
        let active = self.active_cached_entries().max(1);
        (self.total_tokens_ingested as f64) / (active as f64)
    }
}

// ============================================================================
// 5. MLA Full-Token KV Cache + DSA Top-K Sparse Indexer (256-d Heads)
// ============================================================================

#[derive(Clone, Debug)]
pub struct MlaTokenKvEntry {
    pub position: usize,
    pub dsa_index_key: Vec<f32>,
    pub mla_kv_latent: Vec<f32>,
}

#[derive(Clone, Debug)]
pub struct MlaDsaKvCache {
    pub dsa_index_dim: usize,
    pub mla_kv_latent_dim: usize,
    pub top_k_sparse: usize,
    pub entries: Vec<MlaTokenKvEntry>,
}

impl MlaDsaKvCache {
    pub fn new(dsa_index_dim: usize, mla_kv_latent_dim: usize, top_k_sparse: usize) -> Self {
        Self {
            dsa_index_dim,
            mla_kv_latent_dim,
            top_k_sparse: top_k_sparse.max(1),
            entries: Vec::new(),
        }
    }

    pub fn push_token(&mut self, dsa_index_key: Vec<f32>, mla_kv_latent: Vec<f32>) {
        assert_eq!(dsa_index_key.len(), self.dsa_index_dim);
        assert_eq!(mla_kv_latent.len(), self.mla_kv_latent_dim);
        let pos = self.entries.len();
        self.entries.push(MlaTokenKvEntry {
            position: pos,
            dsa_index_key,
            mla_kv_latent,
        });
    }

    pub fn select_dsa_top_k_positions(&self, q_index: &[f32]) -> Vec<(usize, f32)> {
        assert_eq!(q_index.len(), self.dsa_index_dim);
        if self.entries.is_empty() {
            return Vec::new();
        }

        let inv_sqrt_d = 1.0 / (self.dsa_index_dim as f32).sqrt();
        let mut scored: Vec<(usize, f32)> = self
            .entries
            .par_iter()
            .map(|entry| {
                let dot: f32 = q_index
                    .iter()
                    .zip(entry.dsa_index_key.iter())
                    .map(|(&q, &k)| q * k)
                    .sum();
                (entry.position, dot * inv_sqrt_d)
            })
            .collect();

        let k = self.top_k_sparse.min(scored.len());
        scored.sort_unstable_by(|a, b| {
            b.1.partial_cmp(&a.1)
                .unwrap_or(Ordering::Equal)
                .then_with(|| a.0.cmp(&b.0))
        });
        scored.truncate(k);
        scored
    }
}

// ============================================================================
// 6. Unified ERNIE 5.5 Hybrid Attention Layer (MLA-256 / HCA-512 + VHA + mHC)
// ============================================================================

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct HybridAttentionTelemetry {
    pub layer_idx: usize,
    pub layer_kind: AttentionLayerKind,
    pub num_heads: usize,
    pub head_dim: usize,
    pub total_tokens_in_history: usize,
    pub attended_kv_entries: usize,
    pub kv_compression_or_sparsity_ratio: f64,
    pub dsa_selected_positions: Vec<usize>,
    pub low_rank_gate_telemetry: LowRankGateTelemetry,
    pub vha_rank: usize,
    pub mhc_l1_norm_before: f64,
    pub mhc_l1_norm_after_mixing: f64,
}

pub enum HybridKvStore {
    Hca(HcaCompressedKvCache),
    MlaDsa(MlaDsaKvCache),
}

pub struct Ernie55HybridAttentionLayer {
    pub layer_idx: usize,
    pub kind: AttentionLayerKind,
    pub hidden_dim: usize,
    pub num_heads: usize,
    pub head_dim: usize,
    pub low_rank_query_gate: LowRankQueryGate,
    pub w_q_up: LatentMoePackedMatrix,
    pub w_kv_down: LatentMoePackedMatrix,
    pub w_dsa_indexer: Option<LatentMoePackedMatrix>,
    pub vha_mixer: VirtualHeadAttentionMixer,
    pub w_out_proj: LatentMoePackedMatrix,
    pub mhc_mixing_matrix: [[f32; MHC_STREAMS]; MHC_STREAMS],
    pub mhc_read_weights: [f32; MHC_STREAMS],
    pub mhc_write_weights: [f32; MHC_STREAMS],
}

impl Ernie55HybridAttentionLayer {
    #[allow(clippy::needless_range_loop)]
    pub fn forward_mhc_step(
        &self,
        streams: &mut [Vec<f32>; MHC_STREAMS],
        kv_store: &mut HybridKvStore,
    ) -> HybridAttentionTelemetry {
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
            let col = [streams[0][d], streams[1][d], streams[2][d], streams[3][d]];
            for i in 0..MHC_STREAMS {
                let mut m_val = 0.0f32;
                for j in 0..MHC_STREAMS {
                    m_val += self.mhc_mixing_matrix[i][j] * col[j];
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

        let (x_quant, x_scale) = quantize_activations_sym_i8(&x_in);
        let (q_latent, low_rank_gate, gate_telemetry) = self
            .low_rank_query_gate
            .compute_query_latent_and_gate(&x_quant, x_scale);

        let (qlat_quant, qlat_scale) = quantize_activations_sym_i8(&q_latent);
        let q_core = latent_moe_gemv_par_simd(&self.w_q_up, &qlat_quant, qlat_scale).scaled_outputs;
        let kv_latent = latent_moe_gemv_par_simd(&self.w_kv_down, &x_quant, x_scale).scaled_outputs;

        let (mut head_outputs, total_hist, attended_entries, dsa_positions) = match kv_store {
            HybridKvStore::Hca(hca_cache) => {
                hca_cache.push_token_kv(kv_latent);

                let all_entries: Vec<&[f32]> = hca_cache
                    .compressed_blocks
                    .iter()
                    .chain(hca_cache.recent_window.iter())
                    .map(|e| e.kv_latent.as_slice())
                    .collect();

                let attended_count = all_entries.len();
                let heads = evaluate_multi_head_latent_attention(
                    &q_core,
                    &all_entries,
                    self.num_heads,
                    self.head_dim,
                );
                (
                    heads,
                    hca_cache.total_tokens_ingested,
                    attended_count,
                    Vec::new(),
                )
            }
            HybridKvStore::MlaDsa(mla_cache) => {
                let dsa_proj = self
                    .w_dsa_indexer
                    .as_ref()
                    .expect("MLA+DSA layer requires w_dsa_indexer matrix");
                let dsa_vec =
                    latent_moe_gemv_par_simd(dsa_proj, &qlat_quant, qlat_scale).scaled_outputs;

                mla_cache.push_token(dsa_vec.clone(), kv_latent);

                let top_k_hits = mla_cache.select_dsa_top_k_positions(&dsa_vec);
                let selected_pos: Vec<usize> = top_k_hits.iter().map(|&(pos, _)| pos).collect();
                let selected_kv: Vec<&[f32]> = selected_pos
                    .iter()
                    .map(|&pos| mla_cache.entries[pos].mla_kv_latent.as_slice())
                    .collect();

                let attended_count = selected_kv.len();
                let heads = evaluate_multi_head_latent_attention(
                    &q_core,
                    &selected_kv,
                    self.num_heads,
                    self.head_dim,
                );
                (heads, mla_cache.entries.len(), attended_count, selected_pos)
            }
        };

        self.vha_mixer.mix_heads_in_place(&mut head_outputs);

        let inv_heads = 1.0 / (self.num_heads as f32);
        let mut gated_context = vec![0.0f32; self.head_dim];
        for head_vec in &head_outputs {
            for (dst, &val) in gated_context.iter_mut().zip(head_vec.iter()) {
                *dst += val * inv_heads;
            }
        }
        for (dst, &g) in gated_context.iter_mut().zip(low_rank_gate.iter()) {
            *dst *= g;
        }

        let (ctx_quant, ctx_scale) = quantize_activations_sym_i8(&gated_context);
        let attn_delta =
            latent_moe_gemv_par_simd(&self.w_out_proj, &ctx_quant, ctx_scale).scaled_outputs;

        for i in 0..MHC_STREAMS {
            let beta = self.mhc_write_weights[i];
            for d in 0..self.hidden_dim {
                streams[i][d] = mixed_streams[i][d] + beta * attn_delta[d];
            }
        }

        let ratio = (total_hist.max(1) as f64) / (attended_entries.max(1) as f64);

        HybridAttentionTelemetry {
            layer_idx: self.layer_idx,
            layer_kind: self.kind,
            num_heads: self.num_heads,
            head_dim: self.head_dim,
            total_tokens_in_history: total_hist,
            attended_kv_entries: attended_entries,
            kv_compression_or_sparsity_ratio: ratio,
            dsa_selected_positions: dsa_positions,
            low_rank_gate_telemetry: gate_telemetry,
            vha_rank: self.vha_mixer.rank,
            mhc_l1_norm_before: l1_before,
            mhc_l1_norm_after_mixing: l1_after_mix,
        }
    }
}

/// Evaluates multi-head scaled dot-product attention over the latent KV entries `kv_entries`
/// across `num_heads` heads of dimension `head_dim`.
fn evaluate_multi_head_latent_attention(
    q_core: &[f32],
    kv_entries: &[&[f32]],
    num_heads: usize,
    head_dim: usize,
) -> Vec<Vec<f32>> {
    assert_eq!(q_core.len(), head_dim);
    if kv_entries.is_empty() {
        return vec![vec![0.0f32; head_dim]; num_heads];
    }

    let inv_sqrt_d = 1.0 / (head_dim as f32).sqrt();

    (0..num_heads)
        .into_par_iter()
        .map(|h| {
            let head_phase = 1.0 + 0.015 * ((h as f32) - (num_heads as f32) * 0.5);
            let mut scores = Vec::with_capacity(kv_entries.len());
            let mut max_score = f32::NEG_INFINITY;

            for &kv in kv_entries {
                let dot: f32 = q_core
                    .iter()
                    .zip(kv.iter())
                    .enumerate()
                    .map(|(d, (&q_d, &k_d))| {
                        let rot = if (d + h) & 1 == 0 {
                            head_phase
                        } else {
                            1.0 / head_phase
                        };
                        (q_d * rot) * k_d
                    })
                    .sum();
                let s = dot * inv_sqrt_d;
                if s > max_score {
                    max_score = s;
                }
                scores.push(s);
            }

            let mut sum_exp = 0.0f32;
            for s in &mut scores {
                *s = (*s - max_score).exp();
                sum_exp += *s;
            }
            let inv_sum = 1.0 / sum_exp.max(1e-12);

            let mut head_out = vec![0.0f32; head_dim];
            for (&weight_unnorm, &kv) in scores.iter().zip(kv_entries.iter()) {
                let w = weight_unnorm * inv_sum;
                for (dst, &v_d) in head_out.iter_mut().zip(kv.iter()) {
                    *dst += w * v_d;
                }
            }
            head_out
        })
        .collect()
}
