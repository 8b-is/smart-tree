//! Matrix-Free Spectral Witness Oracle (`src/mem8/regularity/oracle.rs`).
//!
//! Tests whether a bipartite pair `(V_i, V_j)` is `(epsilon, p)`-regular in
//! `O(e(V_i, V_j) + |V_i| + |V_j|)` time using rank-1 shifted power iteration on
//! the Frieze-Kannan / Alon-Naor Discrepancy Operator `M = A_{V_i, V_j} - d_bar * 1 * 1^T`.
//!
//! Polished for zero inner-loop heap allocations via thread-local `OracleScratch` reuse
//! and single-pass dominant `EdgeKind` histogramming.

use super::types::{ClusterId, CsrSymbolGraph, EdgeKind, SymbolId};

/// Initial capacity cap (`512`) for thread-local `OracleScratch` working vectors.
pub const SCRATCH_INITIAL_CAPACITY_CAP: usize = 512;

/// Number of histogram bins (`5`) covering all `EdgeKind` variants (`0..=4`).
pub const EDGE_KIND_HIST_BINS: usize = 5;

/// Default fallback `EdgeKind` index (`1` = `EdgeKind::CallRef`) when a pair has zero edges.
pub const DEFAULT_DOMINANT_KIND_INDEX: usize = 1;

/// Floor on ambient density $p$ to prevent division-by-zero (`1e-12`).
pub const ORACLE_AMBIENT_DENSITY_FLOOR: f64 = 1e-12;

/// Minimum L2 norm threshold (`1e-12`) during rank-1 shifted power iteration.
pub const SPECTRAL_NORM_EPSILON: f64 = 1e-12;

/// Number of matrix-free rank-1 shifted power iterations (`6`) per bipartite check.
pub const POWER_ITERATION_STEPS: usize = 6;

/// Sentinel scratch marker (`u32::MAX - 1`) for $O(1)$ membership checks in `measure_witness_deviation`.
pub const WITNESS_SCRATCH_MARKER: u32 = u32::MAX - 1;

/// Reusable thread-local scratch arena eliminating per-pair `Vec` allocations during
/// parallel Kohayakawa–Rödl regularity refinement.
#[derive(Debug, Clone)]
pub struct OracleScratch {
    pub local_index_map: Vec<u32>,
    pub vec_u: Vec<f64>,
    pub vec_v: Vec<f64>,
    pub indexed: Vec<(usize, f64)>,
}

impl OracleScratch {
    pub fn with_vertices(num_vertices: usize) -> Self {
        let cap = num_vertices.min(SCRATCH_INITIAL_CAPACITY_CAP);
        Self {
            local_index_map: vec![0u32; num_vertices],
            vec_u: Vec::with_capacity(cap),
            vec_v: Vec::with_capacity(cap),
            indexed: Vec::with_capacity(cap),
        }
    }
}

/// Result of checking `(epsilon, p)`-regularity on the pair `(V_i, V_j)`
#[derive(Debug, Clone)]
pub enum RegularityCheck {
    /// Pair is `(epsilon, p)`-regular with relative density `d_p` and dominant AST edge kind
    Regular {
        relative_p_density: f64,
        dominant_kind: EdgeKind,
    },
    /// Pair is irregular; includes witness sets `A subset V_i` and `B subset V_j`
    Irregular {
        witness_a: Vec<SymbolId>,
        witness_b: Vec<SymbolId>,
        discrepancy_energy: f64,
    },
}

/// Evaluates `(epsilon, p)`-regularity of `(V_i, V_j)` using matrix-free rank-1 shifted
/// power iteration on the Discrepancy Operator `M = A_{V_i, V_j} - d_bar * 1 * 1^T`.
#[allow(clippy::too_many_arguments)]
pub fn check_bipartite_regularity(
    graph: &CsrSymbolGraph,
    vertex_to_cluster: &[ClusterId],
    cluster_i: ClusterId,
    cluster_j: ClusterId,
    v_i: &[SymbolId],
    v_j: &[SymbolId],
    epsilon: f64,
    local_index_map: &mut [u32],
) -> RegularityCheck {
    let mut vec_u = Vec::new();
    let mut vec_v = Vec::new();
    let mut indexed = Vec::new();
    check_bipartite_regularity_buffers(
        graph,
        vertex_to_cluster,
        cluster_i,
        cluster_j,
        v_i,
        v_j,
        epsilon,
        local_index_map,
        &mut vec_u,
        &mut vec_v,
        &mut indexed,
    )
}

/// Zero-allocation entry point using a thread-local `OracleScratch` arena.
#[inline]
#[allow(clippy::too_many_arguments)]
pub fn check_bipartite_regularity_with_scratch(
    graph: &CsrSymbolGraph,
    vertex_to_cluster: &[ClusterId],
    cluster_i: ClusterId,
    cluster_j: ClusterId,
    v_i: &[SymbolId],
    v_j: &[SymbolId],
    epsilon: f64,
    scratch: &mut OracleScratch,
) -> RegularityCheck {
    check_bipartite_regularity_buffers(
        graph,
        vertex_to_cluster,
        cluster_i,
        cluster_j,
        v_i,
        v_j,
        epsilon,
        &mut scratch.local_index_map,
        &mut scratch.vec_u,
        &mut scratch.vec_v,
        &mut scratch.indexed,
    )
}

#[allow(clippy::too_many_arguments)]
fn check_bipartite_regularity_buffers(
    graph: &CsrSymbolGraph,
    vertex_to_cluster: &[ClusterId],
    cluster_i: ClusterId,
    cluster_j: ClusterId,
    v_i: &[SymbolId],
    v_j: &[SymbolId],
    epsilon: f64,
    local_index_map: &mut [u32],
    vec_u: &mut Vec<f64>,
    vec_v: &mut Vec<f64>,
    indexed: &mut Vec<(usize, f64)>,
) -> RegularityCheck {
    let n_i = v_i.len();
    let n_j = v_j.len();
    if n_i == 0 || n_j == 0 {
        return RegularityCheck::Regular {
            relative_p_density: 0.0,
            dominant_kind: EdgeKind::CallRef,
        };
    }

    // Map global SymbolIds in V_i and V_j to local dense indices 0..n_i and 0..n_j
    for (local_idx, &sym) in v_i.iter().enumerate() {
        local_index_map[sym as usize] = local_idx as u32;
    }
    for (local_idx, &sym) in v_j.iter().enumerate() {
        local_index_map[sym as usize] = local_idx as u32;
    }

    // Count edges e(V_i, V_j) + dominant EdgeKind in a single CSR pass O(|V_i| + deg(V_i))
    let mut edge_count: usize = 0;
    let mut kind_hist = [0u32; EDGE_KIND_HIST_BINS];
    for &u in v_i {
        let start = graph.row_ptr[u as usize] as usize;
        let end = graph.row_ptr[u as usize + 1] as usize;
        for idx in start..end {
            let v = graph.col_idx[idx];
            if vertex_to_cluster[v as usize] == cluster_j {
                edge_count += 1;
                kind_hist[graph.edge_kind[idx] as usize] += 1;
            }
        }
    }

    let mut best_k = DEFAULT_DOMINANT_KIND_INDEX; // default CallRef
    let mut best_cnt = 0u32;
    for (k_idx, &cnt) in kind_hist.iter().enumerate() {
        if cnt > best_cnt {
            best_cnt = cnt;
            best_k = k_idx;
        }
    }
    let dominant_kind = EdgeKind::from_u8(best_k as u8);

    let pair_area = (n_i * n_j) as f64;
    let raw_density = (edge_count as f64) / pair_area;
    let relative_p_density =
        raw_density / graph.ambient_density_p.max(ORACLE_AMBIENT_DENSITY_FLOOR);

    // Trivial sparsity short-circuit: if total edges cannot even violate an epsilon*V_i*V_j box
    if relative_p_density < epsilon * epsilon {
        return RegularityCheck::Regular {
            relative_p_density,
            dominant_kind,
        };
    }

    // Reuse vec_u and vec_v without heap allocation
    vec_u.clear();
    vec_u.resize(n_i, 0.0);

    let inv_sqrt_nj = 1.0 / (n_j as f64).sqrt();
    vec_v.clear();
    vec_v.reserve(n_j);
    for idx in 0..n_j {
        vec_v.push(if idx & 1 == 0 {
            inv_sqrt_nj
        } else {
            -inv_sqrt_nj
        });
    }

    let power_steps = POWER_ITERATION_STEPS;
    let mut sigma = 0.0_f64;

    for _ in 0..power_steps {
        // Step 1: vec_u = M * vec_v = A_{i,j} * vec_v - raw_density * sum(vec_v) * 1
        let sum_v: f64 = vec_v.iter().sum();
        let rank1_shift_u = raw_density * sum_v;

        for (local_u, &u) in v_i.iter().enumerate() {
            let start = graph.row_ptr[u as usize] as usize;
            let end = graph.row_ptr[u as usize + 1] as usize;
            let mut dot = 0.0_f64;
            for &v in &graph.col_idx[start..end] {
                if vertex_to_cluster[v as usize] == cluster_j {
                    let local_v = local_index_map[v as usize] as usize;
                    dot += vec_v[local_v];
                }
            }
            vec_u[local_u] = dot - rank1_shift_u;
        }

        // Normalize vec_u via FELIX hardware SIMD norm & reciprocal scale
        let norm_u = crate::felix_asm::felix_normalize_f64_slice(vec_u);
        if norm_u < SPECTRAL_NORM_EPSILON {
            return RegularityCheck::Regular {
                relative_p_density,
                dominant_kind,
            };
        }

        // Step 2: vec_v = M^T * vec_u = A_{i,j}^T * vec_u - raw_density * sum(vec_u) * 1
        let sum_u: f64 = vec_u.iter().sum();
        let rank1_shift_v = raw_density * sum_u;

        for (local_v, &v) in v_j.iter().enumerate() {
            let start = graph.transpose_row_ptr[v as usize] as usize;
            let end = graph.transpose_row_ptr[v as usize + 1] as usize;
            let mut dot = 0.0_f64;
            for &u in &graph.transpose_col_idx[start..end] {
                if vertex_to_cluster[u as usize] == cluster_i {
                    let local_u = local_index_map[u as usize] as usize;
                    dot += vec_u[local_u];
                }
            }
            vec_v[local_v] = dot - rank1_shift_v;
        }

        sigma = crate::felix_asm::felix_normalize_f64_slice(vec_v);
        if sigma < SPECTRAL_NORM_EPSILON {
            return RegularityCheck::Regular {
                relative_p_density,
                dominant_kind,
            };
        }
    }

    // Spectral threshold for (epsilon, p)-regularity
    let spectral_threshold = epsilon * graph.ambient_density_p * pair_area.sqrt();
    if sigma <= spectral_threshold {
        return RegularityCheck::Regular {
            relative_p_density,
            dominant_kind,
        };
    }

    // Extract combinatorial witness sets A subset V_i and B subset V_j
    let min_a = ((epsilon * (n_i as f64)).ceil() as usize).max(1);
    let min_b = ((epsilon * (n_j as f64)).ceil() as usize).max(1);

    let witness_a = extract_top_witness(v_i, vec_u, min_a, indexed);
    let witness_b = extract_top_witness(v_j, vec_v, min_b, indexed);

    // Verify actual combinatorial density deviation |d_p(A, B) - d_p(V_i, V_j)|
    let dev = measure_witness_deviation(
        graph,
        &witness_a,
        &witness_b,
        relative_p_density,
        local_index_map,
    );

    if dev > epsilon {
        RegularityCheck::Irregular {
            witness_a,
            witness_b,
            discrepancy_energy: dev * dev,
        }
    } else {
        RegularityCheck::Regular {
            relative_p_density,
            dominant_kind,
        }
    }
}

fn extract_top_witness(
    vertices: &[SymbolId],
    scores: &[f64],
    min_len: usize,
    indexed: &mut Vec<(usize, f64)>,
) -> Vec<SymbolId> {
    let pos_count = scores.iter().filter(|&&s| s > 0.0).count();
    let use_positive = pos_count >= min_len;

    indexed.clear();
    indexed.reserve(scores.len());
    for (i, &s) in scores.iter().enumerate() {
        indexed.push((i, if use_positive { s } else { -s }));
    }

    indexed.sort_unstable_by(|a, b| b.1.total_cmp(&a.1));

    let take_count = indexed
        .iter()
        .take_while(|&&(_, s)| s > 0.0)
        .count()
        .max(min_len)
        .min(vertices.len());

    indexed[..take_count]
        .iter()
        .map(|&(idx, _)| vertices[idx])
        .collect()
}

fn measure_witness_deviation(
    graph: &CsrSymbolGraph,
    witness_a: &[SymbolId],
    witness_b: &[SymbolId],
    parent_p_density: f64,
    scratch: &mut [u32],
) -> f64 {
    for &b in witness_b {
        scratch[b as usize] = WITNESS_SCRATCH_MARKER;
    }

    let mut sub_edges = 0usize;
    for &a in witness_a {
        let start = graph.row_ptr[a as usize] as usize;
        let end = graph.row_ptr[a as usize + 1] as usize;
        for &v in &graph.col_idx[start..end] {
            if scratch[v as usize] == WITNESS_SCRATCH_MARKER {
                sub_edges += 1;
            }
        }
    }

    for &b in witness_b {
        scratch[b as usize] = 0;
    }

    let sub_area = (witness_a.len() * witness_b.len()) as f64;
    let sub_p_density =
        (sub_edges as f64) / (sub_area * graph.ambient_density_p.max(ORACLE_AMBIENT_DENSITY_FLOOR));
    (sub_p_density - parent_p_density).abs()
}
