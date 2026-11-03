//! Erdős Number & Mehlhorn Voronoi Steiner Tree Context Pruner
//! (`src/mem8/regularity/erdos_steiner.rs`).
//!
//! Computes exact Erdős Numbers $E_{\mathcal{T}}(v) = \min_{t \in \mathcal{T}} \text{dist}(t, v)$
//! and the 2-Approximation Minimum Steiner Tree $T_{\text{Steiner}}$ connecting focal symbols
//! directly over the zero-copy `ZeroCopyMem8Book` slices + T8R virtual bridges.

use super::mmap_view::ZeroCopyMem8Book;
use super::types::{ClusterId, SymbolId};
use super::wire::decode_t8r_exceptions_leb128;
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

    // Inject virtual T8R exception bridges (Outlier V_0 & Architectural Leaks) into R
    let t8r_bridges = extract_t8r_virtual_bridges(book, max_cid);
    for &(cu, cv, bridge_sym) in &t8r_bridges {
        if (cu as usize) <= max_cid
            && (cv as usize) <= max_cid
            && cid_to_idx[cu as usize] != usize::MAX
            && cid_to_idx[cv as usize] != usize::MAX
        {
            adj[cu as usize].push((cv, 0.85));
            adj[cv as usize].push((cu, 0.85));
            let _ = bridge_sym;
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
            for start_node in [bridge.u, bridge.v] {
                let mut curr = start_node;
                while curr != 0 && !in_steiner_tree[curr as usize] {
                    in_steiner_tree[curr as usize] = true;
                    curr = parent_cluster[curr as usize];
                }
            }
        }
    }

    // Record exact T8R bridge symbols that connect Steiner backbone clusters
    for &(cu, cv, bridge_sym) in &t8r_bridges {
        if (cu as usize) <= max_cid
            && (cv as usize) <= max_cid
            && in_steiner_tree[cu as usize]
            && in_steiner_tree[cv as usize]
            && !exact_bridge_symbols.iter().any(|&(s, _)| s == bridge_sym)
        {
            exact_bridge_symbols.push((bridge_sym, 1));
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

/// Scans the LEB128-compressed T8R exception stream to discover virtual bridges between
/// regular clusters `(ClusterA, ClusterB, BridgeSymbolId)` via $V_0$ outliers or architectural leaks.
pub fn extract_t8r_virtual_bridges(
    book: &ZeroCopyMem8Book,
    max_cid: usize,
) -> Vec<(ClusterId, ClusterId, SymbolId)> {
    let lut = book.symbol_to_cluster();
    let Some(exceptions) = decode_t8r_exceptions_leb128(book.t8r_raw_bytes()) else {
        return Vec::new();
    };

    let mut bridges = Vec::new();
    let mut outlier_neighbors: std::collections::HashMap<SymbolId, Vec<ClusterId>> =
        std::collections::HashMap::new();

    for edge in exceptions {
        let su = edge.src_symbol as usize;
        let sv = edge.dst_symbol as usize;
        if su >= lut.len() || sv >= lut.len() {
            continue;
        }
        let cu = lut[su];
        let cv = lut[sv];

        if cu > 0 && cv > 0 && cu != cv && (cu as usize) <= max_cid && (cv as usize) <= max_cid {
            bridges.push((cu, cv, edge.src_symbol));
        } else if cu == 0 && cv > 0 && (cv as usize) <= max_cid {
            outlier_neighbors
                .entry(edge.src_symbol)
                .or_default()
                .push(cv);
        } else if cv == 0 && cu > 0 && (cu as usize) <= max_cid {
            outlier_neighbors
                .entry(edge.dst_symbol)
                .or_default()
                .push(cu);
        }
    }

    // Connect clusters that share a common V_0 outlier hub (capped to avoid quadratic blowup)
    for (outlier_sym, mut clusters) in outlier_neighbors {
        clusters.sort_unstable();
        clusters.dedup();
        if clusters.len() >= 2 && clusters.len() <= 8 {
            for i in 0..clusters.len() {
                for j in (i + 1)..clusters.len() {
                    bridges.push((clusters[i], clusters[j], outlier_sym));
                }
            }
        }
    }

    bridges
}

/// Software IEEE-754 half-precision (`fp16`) to `f32` decoder (`O(1)` bit manipulation).
#[inline(always)]
pub fn decode_fp16(bits: u16) -> f32 {
    let sign = ((bits & 0x8000) as u32) << 16;
    let exp = ((bits >> 10) & 0x1F) as u32;
    let mant = (bits & 0x03FF) as u32;

    if exp == 0 {
        if mant == 0 {
            return f32::from_bits(sign);
        }
        // Subnormal fp16
        return (if sign != 0 { -1.0 } else { 1.0 }) * (mant as f32) * 5.960_464_5e-8;
    } else if exp == 0x1F {
        return f32::from_bits(sign | 0x7F80_0000 | (mant << 13));
    }

    let f32_exp = (exp + 112) << 23;
    let f32_mant = mant << 13;
    f32::from_bits(sign | f32_exp | f32_mant)
}

/// Erdős Center $Z(G)$, Graph Radius/Diameter, and Turán Extremal Density Report
#[derive(Debug, Clone)]
pub struct ErdosCenterTuranReport {
    /// Graph radius $\text{rad}(R) = \min_{u \in R} e(u)$ (in hops over the Reduced Graph $R$)
    pub graph_radius_hops: u32,
    /// Graph diameter $\text{diam}(R) = \max_{u \in R} e(u)$
    pub graph_diameter_hops: u32,
    /// Erdős Center $Z(R) = \{ C_i \mid e(C_i) = \text{rad}(R) \}$ (minimum-eccentricity core clusters)
    pub center_clusters: Vec<ClusterId>,
    /// Graph Periphery $\{ C_i \mid e(C_i) = \text{diam}(R) \}$ (maximum-eccentricity leaf clusters)
    pub periphery_clusters: Vec<ClusterId>,
    /// Turán extremal edge bound $\text{ex}(n; K_{r+1}) = (1 - 1/r) \cdot n^2 / 2$ for $r = 4$ architectural layers
    pub turan_ex_k5_edges: usize,
    /// Ratio of actual edges to Turán's $K_5$-free extremal bound ($> 1.0$ guarantees $K_5$ entanglement)
    pub turan_density_ratio: f64,
    /// Erdős–Stone–Simonovits effective chromatic number $\hat{\chi}(G)$
    pub erdos_stone_chromatic_bound: usize,
}

/// Computes the Erdős Center $Z(R)$, Graph Radius/Diameter via All-Pairs BFS over the Reduced Graph $R$,
/// and Turán's Extremal Theorem ($T(n, r)$) invariant for $r = 4$ canonical architectural layers.
pub fn compute_erdos_center_and_turan(
    book: &ZeroCopyMem8Book,
    total_vertices: usize,
    total_edges: usize,
) -> ErdosCenterTuranReport {
    let super_nodes = book.super_nodes();
    let reduced_edges = book.reduced_edges();
    let k = super_nodes.len();

    // Turán extremal bound ex(n; K_{r+1}) for r = 4 architectural layers: (1 - 1/4) * n^2 / 2 = 3/8 * n^2
    let r_layers = 4.0_f64;
    let turan_ex_k5 =
        (((1.0 - 1.0 / r_layers) * (total_vertices as f64) * (total_vertices as f64) * 0.5).ceil()
            as usize)
            .max(1);
    let turan_ratio = (total_edges as f64) / (turan_ex_k5 as f64);

    // Erdős–Stone–Simonovits effective chromatic number
    let max_possible = (total_vertices * total_vertices.saturating_sub(1)).max(1) as f64 * 0.5;
    let density = ((total_edges as f64) / max_possible).clamp(0.0, 0.99);
    let erdos_stone_chi = ((1.0 / (1.0 - density)).ceil() as usize).clamp(2, 16);

    if k == 0 {
        return ErdosCenterTuranReport {
            graph_radius_hops: 0,
            graph_diameter_hops: 0,
            center_clusters: Vec::new(),
            periphery_clusters: Vec::new(),
            turan_ex_k5_edges: turan_ex_k5,
            turan_density_ratio: turan_ratio,
            erdos_stone_chromatic_bound: erdos_stone_chi,
        };
    }

    let max_cid = super_nodes
        .iter()
        .map(|s| s.cluster_id as usize)
        .max()
        .unwrap_or(0)
        .max(k);
    let mut adj: Vec<Vec<ClusterId>> = vec![Vec::new(); max_cid + 1];
    for e in reduced_edges {
        if (e.src_cluster as usize) <= max_cid && (e.dst_cluster as usize) <= max_cid {
            adj[e.src_cluster as usize].push(e.dst_cluster);
            adj[e.dst_cluster as usize].push(e.src_cluster);
        }
    }

    let mut eccentricities: Vec<(ClusterId, u32)> = Vec::with_capacity(k);
    let mut bfs_dist = vec![u32::MAX; max_cid + 1];
    let mut queue = Vec::with_capacity(k);

    for sn in super_nodes {
        let start = sn.cluster_id;
        bfs_dist.fill(u32::MAX);
        queue.clear();
        bfs_dist[start as usize] = 0;
        queue.push(start);

        let mut head = 0;
        let mut max_d = 0u32;
        while head < queue.len() {
            let u = queue[head];
            head += 1;
            let d = bfs_dist[u as usize];
            if d > max_d {
                max_d = d;
            }
            for &v in &adj[u as usize] {
                if bfs_dist[v as usize] == u32::MAX {
                    bfs_dist[v as usize] = d + 1;
                    queue.push(v);
                }
            }
        }
        eccentricities.push((start, max_d));
    }

    let positive_ecc: Vec<u32> = eccentricities
        .iter()
        .map(|&(_, e)| e)
        .filter(|&e| e > 0)
        .collect();
    let radius = positive_ecc
        .iter()
        .copied()
        .min()
        .unwrap_or_else(|| eccentricities.iter().map(|&(_, e)| e).min().unwrap_or(0));
    let diameter = eccentricities.iter().map(|&(_, e)| e).max().unwrap_or(0);

    let center_clusters: Vec<ClusterId> = eccentricities
        .iter()
        .filter(|&&(_, e)| e == radius)
        .map(|&(cid, _)| cid)
        .collect();
    let periphery_clusters: Vec<ClusterId> = eccentricities
        .iter()
        .filter(|&&(_, e)| e == diameter)
        .map(|&(cid, _)| cid)
        .collect();

    ErdosCenterTuranReport {
        graph_radius_hops: radius,
        graph_diameter_hops: diameter,
        center_clusters,
        periphery_clusters,
        turan_ex_k5_edges: turan_ex_k5,
        turan_density_ratio: turan_ratio,
        erdos_stone_chromatic_bound: erdos_stone_chi,
    }
}
