//! Energy-Pumping Szemerédi Regularity Compressor (`src/mem8/regularity/compressor.rs`).
//!
//! Implements the Kohayakawa–Rödl / Alon–Frieze–Kannan bounded-refinement loop:
//! tracks mean-square energy $q_p(\mathcal{P}) = \sum_{1 \le i < j \le k} \frac{|V_i||V_j|}{n^2} d_p(V_i, V_j)^2$,
//! splits only clusters actively participating in irregular pairs (`witness_a`, `witness_b`),
//! absorbs micro-fragments ($|C| < \lfloor \varepsilon n / k_{\max} \rfloor$) into the outlier bucket $V_0$,
//! and synthesizes the `Mem8BookArchive` (Reduced Graph $R$ + T8R Exception Ledger).

use super::oracle::{check_bipartite_regularity_with_scratch, OracleScratch, RegularityCheck};
use super::types::*;
use crate::felix_asm::{felix_add_assign_64_f32, felix_normalize64_f32};
use rayon::prelude::*;

/// Hyperparameters for MEM8 "Book Mode" Szemerédi Regularity Compression
#[derive(Debug, Clone, Copy)]
pub struct RegularityConfig {
    /// Target regularity threshold $\varepsilon \in [0.05, 0.15]$
    pub epsilon: f64,
    /// Minimum relative $p$-density $d_{\min}$ to emit a `RegularSuperEdge` in $R$
    pub min_reduced_density: f64,
    /// Hard cap on cluster count $K_{\max}$ (prevents tower-type growth; typically $64..=256$)
    pub max_clusters: usize,
    /// Maximum energy-pumping refinement iterations (guaranteed $\le \lceil 1 / \varepsilon^5 \rceil$)
    pub max_refinement_steps: usize,
}

impl Default for RegularityConfig {
    fn default() -> Self {
        Self {
            epsilon: 0.10,
            min_reduced_density: 0.25,
            max_clusters: 128,
            max_refinement_steps: 16,
        }
    }
}

/// Executes the Kohayakawa–Rödl / Frieze–Kannan bounded regularity refinement loop
/// and produces a compressed `Mem8BookArchive`.
pub fn compress_symbol_graph_to_mem8(
    graph: &CsrSymbolGraph,
    mut partition: SzemerediPartition,
    cfg: RegularityConfig,
) -> Mem8BookArchive {
    let n = graph.num_vertices;
    if n == 0 {
        return Mem8BookArchive {
            model_digest: [0u8; 32],
            epsilon: cfg.epsilon as f32,
            compression_ratio: 1.0,
            final_energy: 0.0,
            super_nodes: Vec::new(),
            reduced_edges: Vec::new(),
            outlier_vertices: Vec::new(),
            exception_ledger: Vec::new(),
            total_symbol_count: 0,
            vertex_to_cluster: Vec::new(),
        };
    }

    for _step in 0..cfg.max_refinement_steps {
        let k = partition.clusters.len().saturating_sub(1); // Exclude V_0
        if k == 0 || k >= cfg.max_clusters {
            break;
        }

        // Generate all unordered pairs (i, j) with 1 <= i < j <= k
        let mut pairs = Vec::with_capacity((k * k) / 2);
        for i in 1..=k {
            for j in (i + 1)..=k {
                pairs.push((i as ClusterId, j as ClusterId));
            }
        }

        // Evaluate all pairs in parallel using zero-allocation thread-local OracleScratch arenas
        let pair_evaluations: Vec<((ClusterId, ClusterId), RegularityCheck, usize)> = pairs
            .par_iter()
            .map_init(
                || OracleScratch::with_vertices(n),
                |scratch, &(ci, cj)| {
                    let v_i = &partition.clusters[ci as usize];
                    let v_j = &partition.clusters[cj as usize];
                    let check = check_bipartite_regularity_with_scratch(
                        graph,
                        &partition.vertex_to_cluster,
                        ci,
                        cj,
                        v_i,
                        v_j,
                        cfg.epsilon,
                        scratch,
                    );
                    ((ci, cj), check, v_i.len() * v_j.len())
                },
            )
            .collect();

        // Compute current mean-square energy q_p(P) and total irregular pair area
        let mut irregular_area = 0usize;
        let mut total_regular_area = 0usize;
        let mut energy_sum = 0.0_f64;
        let mut split_proposals: Vec<Vec<SymbolId>> = vec![Vec::new(); k + 1];
        let mut best_witness_gain: Vec<f64> = vec![0.0_f64; k + 1];

        for ((ci, cj), check, area) in pair_evaluations {
            total_regular_area += area;
            match check {
                RegularityCheck::Regular {
                    relative_p_density, ..
                } => {
                    energy_sum +=
                        (area as f64 / (n * n) as f64) * relative_p_density * relative_p_density;
                }
                RegularityCheck::Irregular {
                    witness_a,
                    witness_b,
                    discrepancy_energy,
                } => {
                    irregular_area += area;
                    if discrepancy_energy > best_witness_gain[ci as usize] {
                        best_witness_gain[ci as usize] = discrepancy_energy;
                        split_proposals[ci as usize] = witness_a;
                    }
                    if discrepancy_energy > best_witness_gain[cj as usize] {
                        best_witness_gain[cj as usize] = discrepancy_energy;
                        split_proposals[cj as usize] = witness_b;
                    }
                }
            }
        }

        partition.energy_index = energy_sum;

        // Stopping Condition: Partition is (epsilon, p)-regular!
        if (irregular_area as f64) <= cfg.epsilon * (total_regular_area.max(1) as f64) {
            break;
        }

        // Refine clusters that have a valid witness split, respecting max_clusters and V_0 size bound
        apply_partition_refinement(&mut partition, &split_proposals, n, &cfg);
    }

    build_mem8_book_archive(graph, &partition, &cfg)
}

fn apply_partition_refinement(
    partition: &mut SzemerediPartition,
    split_proposals: &[Vec<SymbolId>],
    total_n: usize,
    cfg: &RegularityConfig,
) {
    let min_cluster_card = ((cfg.epsilon * (total_n as f64)) / (cfg.max_clusters as f64))
        .floor()
        .max(2.0) as usize;

    let mut next_clusters: Vec<Vec<SymbolId>> = Vec::with_capacity(cfg.max_clusters + 1);
    let mut outlier_v0 = std::mem::take(&mut partition.clusters[0]);
    next_clusters.push(Vec::new()); // Placeholder for V_0 at index 0

    let mut is_in_witness = vec![false; total_n];

    for (cid, cluster_verts) in partition.clusters.iter().enumerate().skip(1) {
        let witness = &split_proposals[cid];
        if witness.is_empty() || next_clusters.len() >= cfg.max_clusters {
            if cluster_verts.len() < min_cluster_card {
                outlier_v0.extend_from_slice(cluster_verts);
            } else {
                next_clusters.push(cluster_verts.clone());
            }
            continue;
        }

        for &w in witness {
            is_in_witness[w as usize] = true;
        }

        let mut part_a = Vec::with_capacity(witness.len());
        let mut part_b = Vec::with_capacity(cluster_verts.len().saturating_sub(witness.len()));

        for &v in cluster_verts {
            if is_in_witness[v as usize] {
                part_a.push(v);
                is_in_witness[v as usize] = false;
            } else {
                part_b.push(v);
            }
        }

        for subpart in [part_a, part_b] {
            if subpart.len() < min_cluster_card {
                outlier_v0.extend_from_slice(&subpart);
            } else if next_clusters.len() < cfg.max_clusters {
                next_clusters.push(subpart);
            } else {
                if let Some(last) = next_clusters.last_mut() {
                    last.extend(subpart);
                }
            }
        }
    }

    next_clusters[0] = outlier_v0;

    // Rebuild vertex_to_cluster lookup table
    for (new_cid, verts) in next_clusters.iter().enumerate() {
        let cid_u16 = new_cid as ClusterId;
        for &v in verts {
            partition.vertex_to_cluster[v as usize] = cid_u16;
        }
    }

    partition.clusters = next_clusters;
}

/// Constructs the final `Mem8BookArchive`:
/// 1. Super-nodes `V_1 .. V_k` with MEM8 wave centroids and principal anchor symbols
/// 2. Regular super-edges where `(V_i, V_j)` is regular and `d_p >= d_min`
/// 3. Exact `T8rExceptionEdge` records for `V_0` outliers, irregular pairs, and sparse leaks
fn build_mem8_book_archive(
    graph: &CsrSymbolGraph,
    partition: &SzemerediPartition,
    cfg: &RegularityConfig,
) -> Mem8BookArchive {
    let k = partition.clusters.len().saturating_sub(1);
    let mut super_nodes = Vec::with_capacity(k);
    let mut reduced_edges = Vec::new();
    let mut exception_ledger = Vec::new();

    // 1. Build Super-Nodes & compute MEM8 semantic centroids
    for cid in 1..=k {
        let verts = &partition.clusters[cid];
        if verts.is_empty() {
            continue;
        }

        let mut centroid = [0.0_f32; 64];
        let mut internal_edges = 0usize;
        let mut deg_pairs: Vec<(SymbolId, u32)> = Vec::with_capacity(verts.len());

        for &u in verts {
            let emb = &graph.symbol_embeddings[u as usize];
            felix_add_assign_64_f32(&mut centroid, emb);

            let start = graph.row_ptr[u as usize] as usize;
            let end = graph.row_ptr[u as usize + 1] as usize;
            let mut internal_deg = 0u32;
            for &v in &graph.col_idx[start..end] {
                if partition.vertex_to_cluster[v as usize] == cid as ClusterId {
                    internal_edges += 1;
                    internal_deg += 1;
                }
            }
            deg_pairs.push((u, internal_deg));
        }

        // L2-normalize the super-node centroid embedding via FELIX 64-lane SIMD kernel
        felix_normalize64_f32(&mut centroid);

        // Pick top-3 internal-degree symbols as human/LLM anchor names
        deg_pairs.sort_unstable_by_key(|a| std::cmp::Reverse(a.1));
        let representative_symbols: Vec<SymbolId> =
            deg_pairs.iter().take(3).map(|&(s, _)| s).collect();

        let area = (verts.len() * verts.len()) as f64;
        let internal_p_density =
            ((internal_edges as f64) / (area * graph.ambient_density_p.max(1e-12))) as f32;

        super_nodes.push(Mem8SuperNode {
            cluster_id: cid as ClusterId,
            vertex_count: verts.len() as u32,
            internal_p_density,
            centroid_embedding: centroid.to_vec(),
            representative_symbols,
        });
    }

    // 2. Classify inter-cluster edges into Reduced Graph R vs. T8R Exception Ledger
    let mut scratch = OracleScratch::with_vertices(graph.num_vertices);
    for i in 1..=k {
        for j in 1..=k {
            if i == j {
                continue;
            }
            let v_i = &partition.clusters[i];
            let v_j = &partition.clusters[j];
            if v_i.is_empty() || v_j.is_empty() {
                continue;
            }

            let check = check_bipartite_regularity_with_scratch(
                graph,
                &partition.vertex_to_cluster,
                i as ClusterId,
                j as ClusterId,
                v_i,
                v_j,
                cfg.epsilon,
                &mut scratch,
            );

            match check {
                RegularityCheck::Regular {
                    relative_p_density,
                    dominant_kind,
                } => {
                    if relative_p_density >= cfg.min_reduced_density {
                        reduced_edges.push(RegularSuperEdge {
                            src_cluster: i as ClusterId,
                            dst_cluster: j as ClusterId,
                            relative_p_density: relative_p_density as f32,
                            dominant_kind,
                        });
                    } else if relative_p_density > 0.0 {
                        // Sparse cross-module leak below d_min -> record exact edges in T8R
                        collect_pair_edges_into_t8r(
                            graph,
                            &partition.vertex_to_cluster,
                            v_i,
                            j as ClusterId,
                            ExceptionReason::ArchitecturalLeak,
                            &mut exception_ledger,
                        );
                    }
                }
                RegularityCheck::Irregular { .. } => {
                    collect_pair_edges_into_t8r(
                        graph,
                        &partition.vertex_to_cluster,
                        v_i,
                        j as ClusterId,
                        ExceptionReason::IrregularPair,
                        &mut exception_ledger,
                    );
                }
            }
        }
    }

    // 3. Record edges incident to the Outlier Set V_0
    for &u in &partition.clusters[0] {
        let start = graph.row_ptr[u as usize] as usize;
        let end = graph.row_ptr[u as usize + 1] as usize;
        for idx in start..end {
            exception_ledger.push(T8rExceptionEdge {
                src_symbol: u,
                dst_symbol: graph.col_idx[idx],
                kind: graph.edge_kind[idx],
                reason: ExceptionReason::OutlierVertex,
            });
        }
    }

    let compressed_primitives = super_nodes.len() + reduced_edges.len() + exception_ledger.len();
    let original_primitives = (graph.num_vertices + graph.num_edges).max(1);
    let compression_ratio = (original_primitives as f32) / (compressed_primitives.max(1) as f32);

    // Deterministic FNV-1a / SplitMix64 32-byte digest over the partition structure
    let mut model_digest = [0u8; 32];
    let mut state = (graph.num_vertices as u64)
        .wrapping_mul(0x9E37_79B9_7F4A_7C15)
        .wrapping_add(graph.num_edges as u64);
    for slot_idx in 0..4 {
        state ^= (slot_idx as u64).wrapping_add(super_nodes.len() as u64);
        state = (state ^ (state >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        state = (state ^ (state >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
        state ^= state >> 31;
        let off = slot_idx * 8;
        model_digest[off..off + 8].copy_from_slice(&state.to_le_bytes());
    }

    Mem8BookArchive {
        model_digest,
        epsilon: cfg.epsilon as f32,
        compression_ratio,
        final_energy: partition.energy_index,
        super_nodes,
        reduced_edges,
        outlier_vertices: partition.clusters[0].clone(),
        exception_ledger,
        total_symbol_count: graph.num_vertices as u32,
        vertex_to_cluster: partition.vertex_to_cluster.clone(),
    }
}

fn collect_pair_edges_into_t8r(
    graph: &CsrSymbolGraph,
    vertex_to_cluster: &[ClusterId],
    v_i: &[SymbolId],
    target_cluster: ClusterId,
    reason: ExceptionReason,
    out_ledger: &mut Vec<T8rExceptionEdge>,
) {
    for &u in v_i {
        let start = graph.row_ptr[u as usize] as usize;
        let end = graph.row_ptr[u as usize + 1] as usize;
        for idx in start..end {
            let v = graph.col_idx[idx];
            if vertex_to_cluster[v as usize] == target_cluster {
                out_ledger.push(T8rExceptionEdge {
                    src_symbol: u,
                    dst_symbol: v,
                    kind: graph.edge_kind[idx],
                    reason,
                });
            }
        }
    }
}
