//! Erdős–Rényi Phase-Transition & Giant-Component Supply-Chain Security Scanner
//! + Ramsey-Theoretic Monochromatic Clique Detector (`src/mem8/regularity/erdos_renyi.rs`).
//!
//! Computes:
//! 1. The Molloy–Reed percolation parameter $\kappa = \langle k^2 \rangle / \langle k \rangle$
//! 2. Cyclomatic excess $s(C) = |E(C)| - |V(C)| + 1$
//! 3. Chernoff–Erdős union-bound log-surprise $\Lambda(C)$ in nats
//! 4. Füredi–Komlós spectral radius $\lambda_1(A_C)$
//! 5. Iterative Tarjan SCC feedback loop extraction
//! 6. Ramsey-theoretic monochromatic cliques ($K_3$ / $K_4$ colored Red/Blue/Green)

use super::types::{CsrSymbolGraph, EdgeKind, SymbolId};

/// Bitflags representing static AST capability hazards per symbol (`0..N`)
#[repr(transparent)]
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct SymbolHazardFlags(pub u8);

impl SymbolHazardFlags {
    pub const NONE: Self = Self(0);
    pub const NET_SOCKET: Self = Self(1 << 0); // HTTP / TCP / IPFS / WebSocket
    pub const PROC_EXEC: Self = Self(1 << 1); // child_process / std::process::Command
    pub const ENV_SECRET: Self = Self(1 << 2); // std::env::var / process.env / ~/.ssh
    pub const FS_MUTATE: Self = Self(1 << 3); // fs::write / unlink / chmod
    pub const EVAL_DYN: Self = Self(1 << 4); // eval() / Function() / dlopen / proc_macro
    pub const MCP_HOOK: Self = Self(1 << 5); // MCP tool registration / pre-commit hook

    #[inline(always)]
    pub const fn union(self, other: Self) -> Self {
        Self(self.0 | other.0)
    }

    #[inline(always)]
    pub const fn distinct_capabilities(self) -> u32 {
        self.0.count_ones()
    }
}

/// Phase regime classification according to Erdős–Rényi & Molloy–Reed percolation theory
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PhaseRegime {
    /// $\kappa < 2.0$ and $s(C) \le 1$: Clean tree/forest hierarchy
    SubcriticalForest,
    /// $2.0 \le \kappa \le 3.5$ and moderate tree-like core: Critical percolation window
    CriticalWindow,
    /// $\kappa > 3.5$ with $O(N)$ Giant Component: Expected in dense monorepos, inspect local cliques
    SupercriticalGiant,
    /// Localized subgraph violates Erdős–Rényi Chernoff bound ($\Lambda(C) > \tau$) or forms a multi-hazard SCC
    AnomalousC2Clique,
}

/// Detailed mathematical forensic report for a suspicious subgraph or SCC
#[derive(Debug, Clone)]
pub struct ComponentAnomaly {
    pub component_id: u32,
    pub vertices: Vec<SymbolId>,
    pub edge_count: usize,
    /// Cyclomatic surplus $s(C) = |E(C)| - |V(C)| + 1$
    pub cyclomatic_excess: isize,
    /// Chernoff–Erdős union-bound log-surprise $\Lambda(C)$ in nats (`> 15.0` $\implies P < 3 \times 10^{-7}$)
    pub erdos_log_surprise_nats: f64,
    /// Top eigenvalue $\lambda_1(A_C)$ (Füredi–Komlós spectral radius)
    pub spectral_radius: f64,
    /// Combined capability bitmask across all symbols in this component
    pub combined_hazards: SymbolHazardFlags,
    /// True if this component is a directed Strongly Connected Component (feedback/hook loop)
    pub is_directed_scc: bool,
}

/// Global Supply-Chain Topological Security Report emitted by `st --security-scan`
#[derive(Debug, Clone)]
pub struct ErdosRenyiSecurityReport {
    pub num_vertices: usize,
    pub num_edges: usize,
    /// Empirical Erdős–Rényi edge probability $\hat{p} = 2|E| / (N(N-1))$
    pub empirical_p: f64,
    /// Critical threshold $p_c = 1 / N$
    pub critical_p_c: f64,
    /// Mean degree $c = \langle k \rangle = 2|E|/N$
    pub mean_degree_c: f64,
    /// Molloy–Reed phase parameter $\kappa = \langle k^2 \rangle / \langle k \rangle$ (Giant component at $\kappa > 2$)
    pub molloy_reed_kappa: f64,
    /// Size of the largest connected component $|C_1|$
    pub giant_component_size: usize,
    /// Size of the second-largest component $|C_2|$ (Susceptibility indicator $\chi$)
    pub second_component_size: usize,
    /// Overall phase regime of the scanned dependency graph
    pub regime: PhaseRegime,
    /// Ranked list of anomalous dense cliques, C2 bridges, or cyclic hook injectors
    pub anomalies: Vec<ComponentAnomaly>,
}

/// Ramsey edge colors for `st graph --ramsey` monochromatic clique hunting
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum RamseyEdgeColor {
    /// Synchronous blocking calls / mutex locks (`EdgeKind::CallRef` / `EdgeKind::Import`)
    RedSyncLock,
    /// Shared mutable state / env / config reads (`EdgeKind::MutableShared`)
    BlueSharedState,
    /// Trait / type / circular module coupling (`EdgeKind::TraitImpl` / `EdgeKind::TypeBound`)
    GreenModuleCoupling,
}

impl RamseyEdgeColor {
    #[inline]
    pub const fn from_edge_kind(kind: EdgeKind) -> Self {
        match kind {
            EdgeKind::Import | EdgeKind::CallRef => Self::RedSyncLock,
            EdgeKind::MutableShared => Self::BlueSharedState,
            EdgeKind::TraitImpl | EdgeKind::TypeBound => Self::GreenModuleCoupling,
        }
    }

    pub const fn label(self) -> &'static str {
        match self {
            Self::RedSyncLock => "Red (Sync/Lock)",
            Self::BlueSharedState => "Blue (Mutable State)",
            Self::GreenModuleCoupling => "Green (Module Coupling)",
        }
    }
}

/// A monochromatic clique ($K_3$ or $K_4$) discovered via Ramsey Theory
#[derive(Debug, Clone)]
pub struct MonochromaticClique {
    pub color: RamseyEdgeColor,
    pub order_k: usize,
    pub vertices: Vec<SymbolId>,
}

/// Ramsey-Theoretic Monochromatic Clique Report (`st graph --ramsey`)
#[derive(Debug, Clone)]
pub struct RamseyCliqueReport {
    /// Classical Ramsey threshold $R(3, 3) = 6$, $R(3, 3, 3) = 17$
    pub ramsey_number_r333: usize,
    pub red_cliques: usize,
    pub blue_cliques: usize,
    pub green_cliques: usize,
    pub top_cliques: Vec<MonochromaticClique>,
}

/// Executes the Erdős–Rényi Phase-Transition & Subgraph Discrepancy Security Scanner in $O(|V| + |E|)$.
pub fn scan_erdos_renyi_supply_chain(
    graph: &CsrSymbolGraph,
    hazards: &[SymbolHazardFlags],
    surprise_threshold_nats: f64,
) -> ErdosRenyiSecurityReport {
    let n = graph.num_vertices;
    let m = graph.num_edges;
    if n < 2 {
        return ErdosRenyiSecurityReport {
            num_vertices: n,
            num_edges: m,
            empirical_p: 0.0,
            critical_p_c: 1.0,
            mean_degree_c: 0.0,
            molloy_reed_kappa: 0.0,
            giant_component_size: n,
            second_component_size: 0,
            regime: PhaseRegime::SubcriticalForest,
            anomalies: Vec::new(),
        };
    }

    // 1. Compute degree moments <k> and <k^2> for the Molloy–Reed criterion
    let mut sum_k = 0.0_f64;
    let mut sum_k2 = 0.0_f64;
    for u in 0..n {
        let out_deg = (graph.row_ptr[u + 1] - graph.row_ptr[u]) as usize;
        let in_deg = (graph.transpose_row_ptr[u + 1] - graph.transpose_row_ptr[u]) as usize;
        let k_u = (out_deg + in_deg) as f64;
        sum_k += k_u;
        sum_k2 += k_u * k_u;
    }

    let mean_k = sum_k / (n as f64);
    let mean_k2 = sum_k2 / (n as f64);
    let molloy_reed_kappa = if mean_k > 1e-9 { mean_k2 / mean_k } else { 0.0 };

    let total_pairs = (n as f64) * ((n - 1) as f64);
    let empirical_p = ((2 * m) as f64 / total_pairs).clamp(1e-12, 1.0 - 1e-12);
    let critical_p_c = 1.0 / (n as f64);

    // 2. Weakly Connected Components (WCC) via linear-time BFS to measure |C_1|, |C_2|, and s(C)
    let mut visited = vec![false; n];
    let mut comp_sizes: Vec<usize> = Vec::new();
    let mut anomalies: Vec<ComponentAnomaly> = Vec::new();
    let mut bfs_queue: Vec<SymbolId> = Vec::with_capacity(256);
    let mut comp_id_counter = 0u32;

    for start_v in 0..n {
        if visited[start_v] {
            continue;
        }
        visited[start_v] = true;
        bfs_queue.clear();
        bfs_queue.push(start_v as SymbolId);

        let mut head = 0;
        let mut directed_edges = 0usize;
        let mut combined_haz = SymbolHazardFlags::NONE;

        while head < bfs_queue.len() {
            let u = bfs_queue[head] as usize;
            head += 1;

            if u < hazards.len() {
                combined_haz = combined_haz.union(hazards[u]);
            }

            let out_s = graph.row_ptr[u] as usize;
            let out_e = graph.row_ptr[u + 1] as usize;
            directed_edges += out_e - out_s;

            for &v in &graph.col_idx[out_s..out_e] {
                if !visited[v as usize] {
                    visited[v as usize] = true;
                    bfs_queue.push(v);
                }
            }

            let in_s = graph.transpose_row_ptr[u] as usize;
            let in_e = graph.transpose_row_ptr[u + 1] as usize;
            for &v in &graph.transpose_col_idx[in_s..in_e] {
                if !visited[v as usize] {
                    visited[v as usize] = true;
                    bfs_queue.push(v);
                }
            }
        }

        let s = bfs_queue.len();
        comp_sizes.push(s);

        if s >= 3 {
            let cyclomatic_excess = (directed_edges as isize) - (s as isize) + 1;
            let surprise = compute_erdos_chernoff_surprise(n, s, directed_edges, empirical_p);

            if surprise >= surprise_threshold_nats
                || (cyclomatic_excess >= 3 && combined_haz.distinct_capabilities() >= 2)
            {
                let spec_rad = estimate_component_spectral_radius(graph, &bfs_queue);
                anomalies.push(ComponentAnomaly {
                    component_id: comp_id_counter,
                    vertices: bfs_queue.clone(),
                    edge_count: directed_edges,
                    cyclomatic_excess,
                    erdos_log_surprise_nats: surprise,
                    spectral_radius: spec_rad,
                    combined_hazards: combined_haz,
                    is_directed_scc: false,
                });
                comp_id_counter += 1;
            }
        }
    }

    comp_sizes.sort_unstable_by(|a, b| b.cmp(a));
    let giant_component_size = comp_sizes.first().copied().unwrap_or(0);
    let second_component_size = comp_sizes.get(1).copied().unwrap_or(0);

    // 3. Tarjan's Strongly Connected Components (SCC) to catch cyclic hook injectors inside the Giant Component
    detect_tarjan_scc_anomalies(
        graph,
        hazards,
        empirical_p,
        surprise_threshold_nats,
        &mut comp_id_counter,
        &mut anomalies,
    );

    // Sort anomalies by highest Erdős log-surprise + hazard multiplier
    anomalies.sort_unstable_by(|a, b| {
        let score_a =
            a.erdos_log_surprise_nats * (1.0 + a.combined_hazards.distinct_capabilities() as f64);
        let score_b =
            b.erdos_log_surprise_nats * (1.0 + b.combined_hazards.distinct_capabilities() as f64);
        score_b
            .partial_cmp(&score_a)
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    let regime = if !anomalies.is_empty()
        && anomalies.iter().any(|a| {
            a.erdos_log_surprise_nats > surprise_threshold_nats * 1.5
                || a.combined_hazards.distinct_capabilities() >= 3
        }) {
        PhaseRegime::AnomalousC2Clique
    } else if molloy_reed_kappa < 2.0 {
        PhaseRegime::SubcriticalForest
    } else if molloy_reed_kappa <= 3.5 {
        PhaseRegime::CriticalWindow
    } else {
        PhaseRegime::SupercriticalGiant
    };

    ErdosRenyiSecurityReport {
        num_vertices: n,
        num_edges: m,
        empirical_p,
        critical_p_c,
        mean_degree_c: mean_k,
        molloy_reed_kappa,
        giant_component_size,
        second_component_size,
        regime,
        anomalies,
    }
}

/// Scans the symbol graph for Ramsey-theoretic monochromatic cliques ($K_3$ and $K_4$)
/// colored by `RamseyEdgeColor` (Red = Sync/Lock, Blue = Mutable State, Green = Module Coupling).
pub fn scan_ramsey_monochromatic_cliques(graph: &CsrSymbolGraph) -> RamseyCliqueReport {
    let n = graph.num_vertices;
    let mut red_cliques = 0usize;
    let mut blue_cliques = 0usize;
    let mut green_cliques = 0usize;
    let mut top_cliques: Vec<MonochromaticClique> = Vec::new();

    // Build per-vertex sorted undirected neighbor lists by color for fast triangle/K4 intersection
    let mut red_adj: Vec<Vec<SymbolId>> = vec![Vec::new(); n];
    let mut blue_adj: Vec<Vec<SymbolId>> = vec![Vec::new(); n];
    let mut green_adj: Vec<Vec<SymbolId>> = vec![Vec::new(); n];

    for u in 0..n {
        let start = graph.row_ptr[u] as usize;
        let end = graph.row_ptr[u + 1] as usize;
        for idx in start..end {
            let v = graph.col_idx[idx] as usize;
            if u == v {
                continue;
            }
            let (lo, hi) = if u < v {
                (u, v as SymbolId)
            } else {
                (v, u as SymbolId)
            };
            match RamseyEdgeColor::from_edge_kind(graph.edge_kind[idx]) {
                RamseyEdgeColor::RedSyncLock => red_adj[lo].push(hi),
                RamseyEdgeColor::BlueSharedState => blue_adj[lo].push(hi),
                RamseyEdgeColor::GreenModuleCoupling => green_adj[lo].push(hi),
            }
        }
    }

    for u in 0..n {
        red_adj[u].sort_unstable();
        red_adj[u].dedup();
        blue_adj[u].sort_unstable();
        blue_adj[u].dedup();
        green_adj[u].sort_unstable();
        green_adj[u].dedup();
    }

    for (color, adj, counter) in [
        (RamseyEdgeColor::RedSyncLock, &red_adj, &mut red_cliques),
        (
            RamseyEdgeColor::BlueSharedState,
            &blue_adj,
            &mut blue_cliques,
        ),
        (
            RamseyEdgeColor::GreenModuleCoupling,
            &green_adj,
            &mut green_cliques,
        ),
    ] {
        for (u, u_neighbors) in adj.iter().enumerate() {
            for &v in u_neighbors {
                let v_neighbors = &adj[v as usize];
                // Intersect sorted neighbor lists u_neighbors and v_neighbors for common w > v
                let mut common_w = Vec::new();
                for &w in u_neighbors {
                    if w > v && v_neighbors.binary_search(&w).is_ok() {
                        common_w.push(w);
                    }
                }
                for (idx_w, &w) in common_w.iter().enumerate() {
                    *counter += 1;
                    // Check if any z in common_w forms a K_4
                    let w_neighbors = &adj[w as usize];
                    let mut found_k4 = false;
                    for &z in &common_w[(idx_w + 1)..] {
                        if w_neighbors.binary_search(&z).is_ok() {
                            *counter += 1;
                            found_k4 = true;
                            if top_cliques.len() < 12 {
                                top_cliques.push(MonochromaticClique {
                                    color,
                                    order_k: 4,
                                    vertices: vec![u as SymbolId, v, w, z],
                                });
                            }
                        }
                    }
                    if !found_k4 && top_cliques.len() < 12 {
                        top_cliques.push(MonochromaticClique {
                            color,
                            order_k: 3,
                            vertices: vec![u as SymbolId, v, w],
                        });
                    }
                }
            }
        }
    }

    top_cliques.sort_unstable_by_key(|a| std::cmp::Reverse(a.order_k));

    RamseyCliqueReport {
        ramsey_number_r333: 17,
        red_cliques,
        blue_cliques,
        green_cliques,
        top_cliques,
    }
}

/// Computes the Erdős–Rényi Chernoff–Kullback–Leibler union-bound log-surprise:
/// $\Lambda(C) = M_s \cdot D_{KL}(\hat{q}_C \parallel \hat{p}) - [s \ln(e N / s)]$
#[inline]
fn compute_erdos_chernoff_surprise(n: usize, s: usize, edges: usize, p: f64) -> f64 {
    if s < 2 {
        return 0.0;
    }
    let m_s = (s * (s - 1)) as f64; // Directed pairs
    let q = ((edges as f64) / m_s).clamp(1e-12, 1.0 - 1e-12);
    if q <= p {
        return 0.0; // Not denser than random baseline
    }

    let kl_div = q * (q / p).ln() + (1.0 - q) * ((1.0 - q) / (1.0 - p)).ln();
    let stirling_union_penalty =
        (s as f64) * ((std::f64::consts::E * (n as f64)) / (s as f64)).ln();

    (m_s * kl_div - stirling_union_penalty).max(0.0)
}

/// Non-recursive (iterative) Tarjan Strongly Connected Components (SCC) detector
fn detect_tarjan_scc_anomalies(
    graph: &CsrSymbolGraph,
    hazards: &[SymbolHazardFlags],
    empirical_p: f64,
    surprise_threshold_nats: f64,
    comp_id_counter: &mut u32,
    out_anomalies: &mut Vec<ComponentAnomaly>,
) {
    let n = graph.num_vertices;
    let mut index = 0u32;
    let mut indices = vec![u32::MAX; n];
    let mut lowlink = vec![u32::MAX; n];
    let mut on_stack = vec![false; n];
    let mut scc_stack: Vec<SymbolId> = Vec::new();
    let mut dfs_stack: Vec<(usize, usize)> = Vec::new(); // (u, current_edge_idx)

    for start_node in 0..n {
        if indices[start_node] != u32::MAX {
            continue;
        }

        indices[start_node] = index;
        lowlink[start_node] = index;
        index += 1;
        scc_stack.push(start_node as SymbolId);
        on_stack[start_node] = true;
        dfs_stack.push((start_node, graph.row_ptr[start_node] as usize));

        while let Some((u, edge_idx)) = dfs_stack.last_mut() {
            let u_val = *u;
            let end_edge = graph.row_ptr[u_val + 1] as usize;

            if *edge_idx < end_edge {
                let v = graph.col_idx[*edge_idx] as usize;
                *edge_idx += 1;

                if indices[v] == u32::MAX {
                    indices[v] = index;
                    lowlink[v] = index;
                    index += 1;
                    scc_stack.push(v as SymbolId);
                    on_stack[v] = true;
                    dfs_stack.push((v, graph.row_ptr[v] as usize));
                } else if on_stack[v] {
                    lowlink[u_val] = lowlink[u_val].min(indices[v]);
                }
            } else {
                dfs_stack.pop();
                if let Some(&(parent, _)) = dfs_stack.last() {
                    lowlink[parent] = lowlink[parent].min(lowlink[u_val]);
                }

                // Root of an SCC found
                if lowlink[u_val] == indices[u_val] {
                    let mut scc_verts = Vec::new();
                    while let Some(w) = scc_stack.pop() {
                        on_stack[w as usize] = false;
                        scc_verts.push(w);
                        if w as usize == u_val {
                            break;
                        }
                    }

                    if scc_verts.len() >= 2 {
                        let s = scc_verts.len();
                        let mut internal_edges = 0usize;
                        let mut combined_haz = SymbolHazardFlags::NONE;

                        for &x in &scc_verts {
                            if (x as usize) < hazards.len() {
                                combined_haz = combined_haz.union(hazards[x as usize]);
                            }
                            let es = graph.row_ptr[x as usize] as usize;
                            let ee = graph.row_ptr[x as usize + 1] as usize;
                            for &y in &graph.col_idx[es..ee] {
                                if scc_verts.contains(&y) {
                                    internal_edges += 1;
                                }
                            }
                        }

                        let surprise =
                            compute_erdos_chernoff_surprise(n, s, internal_edges, empirical_p);
                        let cyc = (internal_edges as isize) - (s as isize) + 1;

                        if surprise >= surprise_threshold_nats * 0.5
                            || combined_haz.distinct_capabilities() >= 2
                        {
                            let spec_rad = estimate_component_spectral_radius(graph, &scc_verts);
                            out_anomalies.push(ComponentAnomaly {
                                component_id: *comp_id_counter,
                                vertices: scc_verts,
                                edge_count: internal_edges,
                                cyclomatic_excess: cyc,
                                erdos_log_surprise_nats: surprise,
                                spectral_radius: spec_rad,
                                combined_hazards: combined_haz,
                                is_directed_scc: true,
                            });
                            *comp_id_counter += 1;
                        }
                    }
                }
            }
        }
    }
}

/// 5-step power iteration to compute $\lambda_1(A_C)$ (Füredi–Komlós spectral radius)
fn estimate_component_spectral_radius(graph: &CsrSymbolGraph, verts: &[SymbolId]) -> f64 {
    let s = verts.len();
    if s <= 1 {
        return 0.0;
    }
    let inv_sqrt = 1.0 / (s as f64).sqrt();
    let mut vec_x = vec![inv_sqrt; s];
    let mut vec_y = vec![0.0_f64; s];
    let mut lambda = 0.0_f64;

    for _ in 0..5 {
        for (i, &u) in verts.iter().enumerate() {
            let es = graph.row_ptr[u as usize] as usize;
            let ee = graph.row_ptr[u as usize + 1] as usize;
            let mut sum = 0.0_f64;
            for &v in &graph.col_idx[es..ee] {
                if let Some(j) = verts.iter().position(|&x| x == v) {
                    sum += vec_x[j];
                }
            }
            vec_y[i] = sum;
        }
        lambda = vec_y.iter().map(|v| v * v).sum::<f64>().sqrt();
        if lambda < 1e-12 {
            return 0.0;
        }
        for i in 0..s {
            vec_x[i] = vec_y[i] / lambda;
        }
    }
    lambda
}
