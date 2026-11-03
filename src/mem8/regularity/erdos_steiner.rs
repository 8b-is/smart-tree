//! Erdős Number & Mehlhorn Voronoi Steiner Tree Context Pruner
//! (`src/mem8/regularity/erdos_steiner.rs`).
//!
//! Computes exact Erdős Numbers $E_{\mathcal{T}}(v) = \min_{t \in \mathcal{T}} \text{dist}(t, v)$
//! and the 2-Approximation Minimum Steiner Tree $T_{\text{Steiner}}$ connecting focal symbols
//! directly over the zero-copy `ZeroCopyMem8Book` slices.

use super::mmap_view::ZeroCopyMem8Book;
use super::types::{ClusterId, SymbolId};
use std::cmp::Ordering;
use std::collections::BinaryHeap;

/// Configuration for `--erdos-root` / `--erdos-radius` Steiner context pruning
#[derive(Debug, Clone, Copy)]
pub struct ErdosSteinerConfig {
    /// Maximum Erdős distance radius $R_{\max}$ around the Steiner backbone (typically `1` or `2`)
    pub max_erdos_radius: u32,
    /// Hard LLM token budget $B_{\text{tokens}}$ (e.g., `8_000` tokens)
    pub token_budget: usize,
    /// Estimated tokens per full AST symbol body (default `~120`), vs `~18` tokens for a SuperNode summary
    pub tokens_per_symbol: usize,
    pub tokens_per_supernode: usize,
}

impl Default for ErdosSteinerConfig {
    fn default() -> Self {
        Self {
            max_erdos_radius: 2,
            token_budget: 8_000,
            tokens_per_symbol: 120,
            tokens_per_supernode: 18,
        }
    }
}

/// Output of the Erdős-Steiner Context Pruner fed directly into MCP / Claude context
#[derive(Debug, Clone)]
pub struct ErdosSteinerSlice {
    /// Terminal symbols $\mathcal{T}$ (Erdős Number 0 — "Paul Erdős" focal roots)
    pub root_terminals: Vec<SymbolId>,
    /// Ordered Steiner backbone of Szemerédi Super-Nodes connecting all terminals
    pub steiner_super_nodes: Vec<SteinerClusterNode>,
    /// Exact outlier/bridge symbols from $V_0$ or T8R exceptions required for compilation/flow
    pub exact_bridge_symbols: Vec<(SymbolId, u32)>, // (symbol_id, erdos_number)
    /// Total estimated LLM tokens consumed by this slice
    pub estimated_tokens: usize,
}

#[derive(Debug, Clone)]
pub struct SteinerClusterNode {
    pub cluster_id: ClusterId,
    /// Combinatorial Erdős Number (0 = contains a root terminal, 1 = direct Steiner bridge, >=2 = shell)
    pub erdos_number: u32,
    /// Geodesic resistance distance from the nearest terminal $t \in \mathcal{T}$
    pub resistance_dist: f32,
    /// Top-3 anchor symbols representing this cluster
    pub anchor_symbols: [SymbolId; 3],
    /// Whether this cluster should be expanded to full AST symbols (Erdős #0) or kept as a MEM8 summary
    pub expand_full_symbols: bool,
}

#[derive(Copy, Clone, PartialEq)]
struct DijkstraState {
    cost: f32,
    hops: u32,
    cluster: ClusterId,
    voronoi_root: ClusterId,
}

impl Eq for DijkstraState {}
impl PartialOrd for DijkstraState {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}
impl Ord for DijkstraState {
    fn cmp(&self, other: &Self) -> Ordering {
        other
            .cost
            .partial_cmp(&self.cost)
            .unwrap_or(Ordering::Equal)
    }
}

/// Runs Mehlhorn's 2-Approximation Minimum Steiner Tree + Erdős Number shell expansion
/// directly on the zero-copy `.m8b` mmap view.
pub fn prune_context_erdos_steiner(
    book: &ZeroCopyMem8Book,
    root_terminals: &[SymbolId],
    query_embedding: Option<&[f32; 64]>,
    cfg: ErdosSteinerConfig,
) -> ErdosSteinerSlice {
    let super_nodes = book.super_nodes();
    let reduced_edges = book.reduced_edges();
    let sym_to_cluster = book.symbol_to_cluster();
    let centroids = book.centroids();
    let num_clusters = super_nodes.len() + 1; // 1-indexed clusters 1..=K

    // 1. Map terminal SymbolIds -> Terminal ClusterIds T_R (and record V_0 outlier terminals)
    let mut terminal_clusters: Vec<ClusterId> = Vec::new();
    let mut exact_bridge_symbols: Vec<(SymbolId, u32)> = Vec::new();

    for &sym in root_terminals {
        if (sym as usize) < sym_to_cluster.len() {
            let cid = sym_to_cluster[sym as usize];
            if cid == 0 {
                // Terminal is in Outlier V_0 ("God Object") -> Erdős Number 0 exact symbol
                exact_bridge_symbols.push((sym, 0));
            } else if !terminal_clusters.contains(&cid) {
                terminal_clusters.push(cid);
            }
        }
    }

    // Fallback: if all terminals were in V_0 or empty, seed with top-1 semantic centroid
    if terminal_clusters.is_empty() {
        if let Some(q_emb) = query_embedding {
            if let Some(&(best_cid, _)) = book.top_k_super_nodes(q_emb, 1).first() {
                terminal_clusters.push(best_cid);
            }
        } else if let Some(first_sn) = super_nodes.first() {
            terminal_clusters.push(first_sn.cluster_id);
        }
    }

    // Build O(1) lookup from ClusterId -> index in `super_nodes` slice
    let max_cid = super_nodes
        .iter()
        .map(|s| s.cluster_id as usize)
        .max()
        .unwrap_or(0)
        .max(num_clusters);
    let mut cid_to_idx = vec![usize::MAX; max_cid + 1];
    for (idx, sn) in super_nodes.iter().enumerate() {
        cid_to_idx[sn.cluster_id as usize] = idx;
    }

    // Build undirected adjacency list over Reduced Graph R so Steiner paths can traverse
    // both caller -> callee and callee -> caller directions
    let mut adj: Vec<Vec<(ClusterId, f32)>> = vec![Vec::new(); max_cid + 1];
    for edge in reduced_edges {
        let u = edge.src_cluster as usize;
        let v = edge.dst_cluster as usize;
        if u <= max_cid
            && v <= max_cid
            && cid_to_idx[u] != usize::MAX
            && cid_to_idx[v] != usize::MAX
        {
            let density = (edge.relative_p_density_fp16 as f32 / 1024.0).max(0.01);
            let mut weight = 1.0 / (1.0 + density.ln_1p());

            if let Some(q_emb) = query_embedding {
                let dst_idx = cid_to_idx[v];
                let dot = crate::felix_asm::felix_dot64_f32(&centroids[dst_idx].lanes, q_emb);
                weight *= 1.0 - 0.35 * dot.max(0.0);
            }

            adj[u].push((edge.dst_cluster, weight));
            adj[v].push((edge.src_cluster, weight * 1.05));
        }
    }

    // 2. Multi-Source Voronoi Dijkstra from all terminal clusters simultaneously
    let mut dist = vec![f32::INFINITY; max_cid + 1];
    let mut erdos_hops = vec![u32::MAX; max_cid + 1];
    let mut voronoi_source = vec![0 as ClusterId; max_cid + 1];
    let mut parent_cluster = vec![0 as ClusterId; max_cid + 1];
    let mut heap = BinaryHeap::new();

    for &t_cid in &terminal_clusters {
        if (t_cid as usize) <= max_cid && cid_to_idx[t_cid as usize] != usize::MAX {
            dist[t_cid as usize] = 0.0;
            erdos_hops[t_cid as usize] = 0;
            voronoi_source[t_cid as usize] = t_cid;
            heap.push(DijkstraState {
                cost: 0.0,
                hops: 0,
                cluster: t_cid,
                voronoi_root: t_cid,
            });
        }
    }

    while let Some(DijkstraState {
        cost,
        hops,
        cluster: u,
        voronoi_root,
    }) = heap.pop()
    {
        if cost > dist[u as usize] {
            continue;
        }
        for &(v, weight) in &adj[u as usize] {
            let next_cost = cost + weight;
            if next_cost < dist[v as usize] {
                dist[v as usize] = next_cost;
                erdos_hops[v as usize] = hops + 1;
                voronoi_source[v as usize] = voronoi_root;
                parent_cluster[v as usize] = u;
                heap.push(DijkstraState {
                    cost: next_cost,
                    hops: hops + 1,
                    cluster: v,
                    voronoi_root,
                });
            }
        }
    }

    // 3. Mehlhorn Voronoi Boundary Bridges & Kruskal MST over Terminal Regions
    #[derive(Clone, Copy)]
    struct VoronoiBridge {
        reg_a: ClusterId,
        reg_b: ClusterId,
        u: ClusterId,
        v: ClusterId,
        total_cost: f32,
    }

    let mut bridges: Vec<VoronoiBridge> = Vec::new();
    for u_cid in 1..=max_cid {
        let reg_u = voronoi_source[u_cid];
        if reg_u == 0 {
            continue;
        }
        for &(v_cid, w) in &adj[u_cid] {
            let reg_v = voronoi_source[v_cid as usize];
            if reg_v != 0 && reg_u < reg_v {
                let total_cost = dist[u_cid] + w + dist[v_cid as usize];
                bridges.push(VoronoiBridge {
                    reg_a: reg_u,
                    reg_b: reg_v,
                    u: u_cid as ClusterId,
                    v: v_cid,
                    total_cost,
                });
            }
        }
    }

    bridges.sort_unstable_by(|a, b| {
        a.total_cost
            .partial_cmp(&b.total_cost)
            .unwrap_or(Ordering::Equal)
    });

    // Disjoint-Set (Union-Find) over Voronoi regions to extract Steiner MST
    let mut uf_parent: Vec<ClusterId> = (0..=(max_cid as ClusterId)).collect();
    fn uf_find(parent: &mut [ClusterId], mut x: ClusterId) -> ClusterId {
        while parent[x as usize] != x {
            parent[x as usize] = parent[parent[x as usize] as usize];
            x = parent[x as usize];
        }
        x
    }

    let mut in_steiner_tree = vec![false; max_cid + 1];
    for &t_cid in &terminal_clusters {
        if (t_cid as usize) <= max_cid {
            in_steiner_tree[t_cid as usize] = true;
        }
    }

    for bridge in bridges {
        let root_a = uf_find(&mut uf_parent, bridge.reg_a);
        let root_b = uf_find(&mut uf_parent, bridge.reg_b);
        if root_a != root_b {
            uf_parent[root_a as usize] = root_b;
            // Trace shortest paths back from bridge.u -> reg_a and bridge.v -> reg_b
            for start_node in [bridge.u, bridge.v] {
                let mut curr = start_node;
                while curr != 0 && !in_steiner_tree[curr as usize] {
                    in_steiner_tree[curr as usize] = true;
                    curr = parent_cluster[curr as usize];
                }
            }
        }
    }

    // 4. Budget-Aware Erdős Shell Expansion (Erdős #0 -> #1 Steiner -> #2 Radius Shell)
    let mut candidates: Vec<(ClusterId, u32, f32, bool)> = Vec::new();
    for sn in super_nodes {
        let cid = sn.cluster_id as usize;
        let is_terminal = terminal_clusters.contains(&sn.cluster_id);
        let is_backbone = in_steiner_tree[cid];
        let hops = erdos_hops[cid];

        let effective_erdos = if is_terminal {
            0
        } else if is_backbone {
            1
        } else {
            hops
        };

        if effective_erdos <= cfg.max_erdos_radius {
            candidates.push((sn.cluster_id, effective_erdos, dist[cid], is_terminal));
        }
    }

    // Sort strictly by (Erdős Number ASC, Resistance Distance ASC)
    candidates.sort_unstable_by(|a, b| {
        a.1.cmp(&b.1)
            .then_with(|| a.2.partial_cmp(&b.2).unwrap_or(Ordering::Equal))
    });

    let mut selected_nodes = Vec::new();
    let mut tokens_used = exact_bridge_symbols.len() * cfg.tokens_per_symbol;

    for (cid, erdos_num, res_dist, is_terminal) in candidates {
        let sn = &super_nodes[cid_to_idx[cid as usize]];
        let cost = if is_terminal {
            (sn.vertex_count.min(8) as usize) * cfg.tokens_per_symbol
        } else {
            cfg.tokens_per_supernode
        };

        if tokens_used + cost > cfg.token_budget && !selected_nodes.is_empty() {
            break;
        }
        tokens_used += cost;

        selected_nodes.push(SteinerClusterNode {
            cluster_id: cid,
            erdos_number: erdos_num,
            resistance_dist: res_dist,
            anchor_symbols: sn.anchor_symbols,
            expand_full_symbols: is_terminal,
        });
    }

    ErdosSteinerSlice {
        root_terminals: root_terminals.to_vec(),
        steiner_super_nodes: selected_nodes,
        exact_bridge_symbols,
        estimated_tokens: tokens_used,
    }
}
