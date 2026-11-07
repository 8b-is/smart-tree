//! Erdős–Rényi Phase-Transition & Giant-Component Supply-Chain Security Scanner
//! + Ramsey-Theoretic Monochromatic Clique Detector (`src/mem8/regularity/erdos_renyi.rs`).
//!
//! Computes over the Cross-Boundary Shadow Graph $G_{\text{shadow}} = (V, E_{\text{shadow}})$:
//! 1. The Molloy–Reed percolation parameter $\kappa = \langle k^2 \rangle / \langle k \rangle$
//! 2. Critical percolation scale $n^{2/3}$ and component excess $s(C) = |E(C)| - |V(C)| + 1$ via Union-Find
//! 3. Chernoff–Erdős union-bound log-surprise $\Lambda(C)$ in nats
//! 4. Füredi–Komlós spectral radius $\lambda_1(A_{\text{shadow}})$ vs. random bound $\max(c, \sqrt{\Delta_{\max}})$
//!    + Top-5 Eigenvector Centrality `pivot_symbols` per anomalous component
//! 5. Iterative Tarjan SCC feedback loop extraction (`directed_scc_cycles`)
//! 6. Ramsey-theoretic monochromatic cliques ($K_3$ / $K_4$ colored Red/Blue/Green)

use super::types::{CsrSymbolGraph, EdgeKind, SymbolId};

/// Bitflags representing static AST capability hazards per symbol (`0..N`)
#[repr(transparent)]
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct SymbolHazardFlags(pub u8);

impl SymbolHazardFlags {
    pub const NONE: Self = Self(0);
    /// Network / HTTP / TCP / IPFS / IPNS gateway egress
    pub const NET_SOCKET: Self = Self(1 << 0);
    pub const NET_IPFS_EGRESS: Self = Self(1 << 0);
    /// Process spawning (`child_process`, `std::process::Command`)
    pub const PROC_EXEC: Self = Self(1 << 1);
    /// Secret / `.env` / credential / SSH key reads
    pub const ENV_SECRET: Self = Self(1 << 2);
    pub const SECRET_ENV_READ: Self = Self(1 << 2);
    /// Filesystem mutation (`fs::write`, `chmod`, `unlink`)
    pub const FS_MUTATE: Self = Self(1 << 3);
    /// Dynamic `eval()`, `Function()`, `dlopen`, or `proc_macro` injection
    pub const EVAL_DYN: Self = Self(1 << 4);
    pub const DYNAMIC_EVAL: Self = Self(1 << 4);
    /// MCP tool registration or git/agent lifecycle hook mutation
    pub const MCP_HOOK: Self = Self(1 << 5);
    pub const MCP_HOOK_MUTATE: Self = Self(1 << 5);
    /// High-entropy obfuscated identifier or payload literal
    pub const HIGH_ENTROPY_OBFUSCATED: Self = Self(1 << 6);

    #[inline(always)]
    pub const fn union(self, other: Self) -> Self {
        Self(self.0 | other.0)
    }

    #[inline(always)]
    pub const fn is_hazardous(self) -> bool {
        self.0 != 0
    }

    #[inline(always)]
    pub const fn bits(self) -> u8 {
        self.0
    }

    #[inline(always)]
    pub const fn distinct_capabilities(self) -> u32 {
        self.0.count_ones()
    }
}

/// Phase regime classification according to Erdős–Rényi & Molloy–Reed percolation theory
/// (`O(|V| + |E_SHADOW|)`):
/// - `*1`: Because dependency graphs have skewed degree distributions, we supplement
///   `*2` ($\langle k \rangle = c = \frac{2|E_{\text{SHADOW}}|}{|V|}$) with
///   `*3` ($\kappa = \frac{\langle k^2 \rangle}{\langle k \rangle} = \frac{\sum_{v \in V} \deg(v)^2}{\sum_{v \in V} \deg(v)}$).
/// - `WHEN \kappa > 2`: a giant transitive reachability component forms across package boundaries
///   allowing a single poisoned leaf dependency to reach core application state.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PhaseRegime {
    /// $\kappa < 2$ ($\sum \deg(v)^2 < 2 \sum \deg(v)$): Subcritical forest hierarchy
    SubcriticalForest,
    /// $\kappa = 2$ or critical percolation boundary $|C_1| \approx \Theta(n^{2/3})$
    CriticalWindow,
    /// $\kappa > 2$ ($\sum \deg(v)^2 > 2 \sum \deg(v)$): Giant transitive reachability component across package boundaries
    SupercriticalGiant,
    /// Localized subgraph violates Erdős–Rényi Chernoff bound ($\Lambda(C) > \tau$) or forms a multi-hazard SCC
    AnomalousC2Clique,
}

/// Detailed mathematical forensic report for a suspicious subgraph or SCC
#[derive(Debug, Clone)]
pub struct ComponentAnomaly {
    pub component_id: u32,
    pub vertex_count: usize,
    pub vertices: Vec<SymbolId>,
    pub edge_count: usize,
    /// Cyclomatic surplus $s(C) = |E(C)| - |V(C)| + 1$
    pub cyclomatic_excess: isize,
    /// Number of non-trivial directed Strongly Connected Component cycles inside this component
    pub directed_scc_cycles: usize,
    /// Chernoff–Erdős union-bound log-surprise $\Lambda(C)$ in nats (`> 15.0` $\implies P < 3 \times 10^{-7}$)
    pub erdos_log_surprise_nats: f64,
    /// Top eigenvalue $\lambda_1(A_C)$ (Füredi–Komlós spectral radius)
    pub spectral_radius: f64,
    /// Combined capability bitmask across all symbols in this component
    pub combined_hazards: SymbolHazardFlags,
    /// True if this component contains or is a directed Strongly Connected Component (feedback/hook loop)
    pub is_directed_scc: bool,
    /// Top-5 symbols inside this component ranked by shadow eigenvector centrality $\mathbf{v}_1$
    pub pivot_symbols: Vec<(SymbolId, f32)>,
}

/// Global Supply-Chain Topological Security Report emitted by `st --security-scan` & `st --mode the-book`
/// Computed in $O(|V| + |E_{\text{SHADOW}}|)$.
#[derive(Debug, Clone)]
pub struct ErdosRenyiSecurityReport {
    pub num_vertices: usize,
    pub num_edges: usize,
    /// Active vertices participating in the cross-boundary shadow graph $G_{\text{shadow}}$
    pub active_vertices: usize,
    /// Cross-boundary or hazard-bearing shadow edges $|E_{\text{shadow}}|$
    pub shadow_edges: usize,
    /// Empirical Erdős–Rényi edge probability $\hat{p} = 2|E| / (N(N-1))$
    pub empirical_p: f64,
    /// Critical threshold $p_c = 1 / N$
    pub critical_p_c: f64,
    /// Mean degree $c = \langle k \rangle = 2|E_{\text{shadow}}|/|V|$ (`*2`)
    pub mean_degree_c: f64,
    /// Exact coprime numerator $p_c$ of $\langle k \rangle = c = \frac{2|E_{\text{SHADOW}}|}{|V|}$ (`*2`)
    pub mean_degree_c_num: u64,
    /// Exact coprime denominator $q_c$ of $\langle k \rangle = c = \frac{2|E_{\text{SHADOW}}|}{|V|}$ (`*2`)
    pub mean_degree_c_den: u64,
    /// Molloy–Reed phase parameter $\kappa = \langle k^2 \rangle / \langle k \rangle = \frac{\sum_{v \in V} \deg(v)^2}{\sum_{v \in V} \deg(v)}$ (`*3`)
    pub molloy_reed_kappa: f64,
    /// Exact coprime numerator $p_\kappa$ of $\kappa = \frac{\sum_{v \in V} \deg(v)^2}{\sum_{v \in V} \deg(v)}$ (`*3`)
    pub molloy_reed_kappa_num: u64,
    /// Exact coprime denominator $q_\kappa$ of $\kappa = \frac{\sum_{v \in V} \deg(v)^2}{\sum_{v \in V} \deg(v)}$ (`*3`)
    pub molloy_reed_kappa_den: u64,
    /// Exact integer threshold check: $\kappa > 2 \iff \sum_{v \in V} \deg(v)^2 > 2 \sum_{v \in V} \deg(v)$
    /// (`WHEN \kappa > 2 -> giant transitive reachability component across package boundaries`)
    pub percolation_kappa_gt_2: bool,
    /// True when $\kappa > 2$ and a poisoned/hazardous leaf dependency can transitively reach core application state
    pub poisoned_leaf_reaches_core: bool,
    /// Human-readable percolation theorem diagnostic (`*1`, `*2`, `*3`)
    pub percolation_diagnostic: String,
    /// Size of the largest connected component $|C_1|$
    pub giant_component_size: usize,
    /// Size of the second-largest component $|C_2|$ (Susceptibility indicator $\chi$)
    pub second_component_size: usize,
    /// Erdős–Rényi critical percolation window scale $n^{2/3}$
    pub critical_scale_n_two_thirds: f64,
    /// Largest eigenvalue $\lambda_1(A_{\text{shadow}})$ via 12-iteration power method
    pub spectral_radius_lambda1: f64,
    /// Füredi–Komlós expected random spectral radius $\max(c, \sqrt{\Delta_{\max}})$
    pub expected_random_lambda1: f64,
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

/// Executes the Erdős–Rényi Phase-Transition & Subgraph Discrepancy Security Scanner in $O(|V| + |E|)$
/// when no explicit Szemerédi cluster LUT is provided (treats all edges as candidates).
pub fn scan_erdos_renyi_supply_chain(
    graph: &CsrSymbolGraph,
    hazards: &[SymbolHazardFlags],
    surprise_threshold_nats: f64,
) -> ErdosRenyiSecurityReport {
    scan_erdos_renyi_shadow_graph(graph, &[], hazards, surprise_threshold_nats)
}

/// Executes the Erdős–Rényi Phase-Transition & Cross-Boundary Shadow Graph ($G_{\text{shadow}}$)
/// Supply-Chain Anomaly Detector in $O(|V| + |E_{\text{shadow}}|)$.
///
/// Filters `graph` down to cross-boundary edges (`symbol_to_cluster[u] != symbol_to_cluster[v]`
/// or `V_0` outlier cluster `0`) or edges touching capability hazards (`hazards[u]` / `hazards[v]`),
/// tracks component excess $s(C) = |E(C)| - |V(C)| + 1$ via Path-Compressing Union-Find,
/// and computes the Füredi–Komlós spectral radius $\lambda_1(A_{\text{shadow}})$ and Top-5
/// eigenvector centrality `pivot_symbols`.
pub fn scan_erdos_renyi_shadow_graph(
    graph: &CsrSymbolGraph,
    symbol_to_cluster: &[u16],
    hazards: &[SymbolHazardFlags],
    surprise_threshold_nats: f64,
) -> ErdosRenyiSecurityReport {
    let n = graph.num_vertices;
    let m = graph.num_edges;
    if n < 2 {
        return ErdosRenyiSecurityReport {
            num_vertices: n,
            num_edges: m,
            active_vertices: n,
            shadow_edges: 0,
            empirical_p: 0.0,
            critical_p_c: 1.0,
            mean_degree_c: 0.0,
            mean_degree_c_num: 0,
            mean_degree_c_den: 1,
            molloy_reed_kappa: 0.0,
            molloy_reed_kappa_num: 0,
            molloy_reed_kappa_den: 1,
            percolation_kappa_gt_2: false,
            poisoned_leaf_reaches_core: false,
            percolation_diagnostic: "Subcritical (< 2 vertices)".to_string(),
            giant_component_size: n,
            second_component_size: 0,
            critical_scale_n_two_thirds: (n as f64).powf(2.0 / 3.0),
            spectral_radius_lambda1: 0.0,
            expected_random_lambda1: 0.0,
            regime: PhaseRegime::SubcriticalForest,
            anomalies: Vec::new(),
        };
    }

    // 1. Construct Cross-Boundary Shadow Graph G_shadow & Union-Find with Excess Tracking
    let mut uf_parent: Vec<u32> = (0..n as u32).collect();
    let mut uf_rank: Vec<u8> = vec![0; n];
    let mut comp_v: Vec<usize> = vec![1; n];
    let mut comp_e: Vec<usize> = vec![0; n];
    let mut comp_haz: Vec<SymbolHazardFlags> = (0..n)
        .map(|i| hazards.get(i).copied().unwrap_or(SymbolHazardFlags::NONE))
        .collect();

    let mut shadow_deg: Vec<u32> = vec![0; n];
    let mut shadow_edges_undirected: Vec<(u32, u32)> = Vec::new();
    let mut shadow_edges_directed: Vec<(u32, u32)> = Vec::new();

    for u in 0..n {
        let cu = symbol_to_cluster.get(u).copied().unwrap_or(u as u16 + 1);
        let hu = hazards.get(u).copied().unwrap_or(SymbolHazardFlags::NONE);

        let start = graph.row_ptr[u] as usize;
        let end = graph.row_ptr[u + 1] as usize;
        for &v_sym in &graph.col_idx[start..end] {
            let v = v_sym as usize;
            if u == v || v >= n {
                continue;
            }
            let cv = symbol_to_cluster.get(v).copied().unwrap_or(v as u16 + 1);
            let hv = hazards.get(v).copied().unwrap_or(SymbolHazardFlags::NONE);

            // Filter to cross-boundary shadow edges, V_0 outliers, or capability-hazard edges
            let is_cross_boundary = symbol_to_cluster.is_empty() || cu != cv || cu == 0 || cv == 0;
            if is_cross_boundary || hu.is_hazardous() || hv.is_hazardous() {
                shadow_edges_directed.push((u as u32, v as u32));
                shadow_deg[u] += 1;
                shadow_deg[v] += 1;
                shadow_edges_undirected.push((u as u32, v as u32));
                uf_union_with_excess(
                    u as u32,
                    v as u32,
                    &mut uf_parent,
                    &mut uf_rank,
                    &mut comp_v,
                    &mut comp_e,
                    &mut comp_haz,
                );
            }
        }
    }

    // 2. Compute Erdős–Rényi & Molloy–Reed Moment Invariants over Active Shadow Graph
    let active_n = shadow_deg
        .iter()
        .enumerate()
        .filter(|&(i, &d)| {
            d > 0
                || hazards
                    .get(i)
                    .copied()
                    .unwrap_or(SymbolHazardFlags::NONE)
                    .is_hazardous()
        })
        .count()
        .max(2);

    let m_shadow = shadow_edges_undirected.len();
    let mut sum_k: u64 = 0;
    let mut sum_k2: u64 = 0;
    let mut max_deg: u32 = 0;

    for &d in &shadow_deg {
        sum_k += d as u64;
        sum_k2 += (d as u64) * (d as u64);
        if d > max_deg {
            max_deg = d;
        }
    }

    // *2: <k> = c =: 2 |E_SHADOW| / |V| reduced to coprime (mean_degree_c_num, mean_degree_c_den)
    let raw_c_num = (2 * m_shadow) as u64;
    let raw_c_den = (active_n as u64).max(1);
    let gcd_c = crate::felix_asm::felix_binary_gcd_u64(raw_c_num, raw_c_den).max(1);
    let mean_degree_c_num = if raw_c_num == 0 { 0 } else { raw_c_num / gcd_c };
    let mean_degree_c_den = if raw_c_num == 0 { 1 } else { raw_c_den / gcd_c };

    // *3: Molloy-Reed percolation ratio kappa = <k^2>/<k> = sum_{v in V} deg(v)^2 / sum_{v in V} deg(v)
    //     (*1: supplements <k> = c = 2|E_SHADOW|/|V| because dependency graphs have skewed degree distributions)
    let gcd_k = crate::felix_asm::felix_binary_gcd_u64(sum_k2, sum_k.max(1)).max(1);
    let molloy_reed_kappa_num = if sum_k == 0 { 0 } else { sum_k2 / gcd_k };
    let molloy_reed_kappa_den = if sum_k == 0 { 1 } else { sum_k / gcd_k };

    // Exact integer threshold: WHEN kappa > 2 <=> sum_{v in V} deg(v)^2 > 2 * sum_{v in V} deg(v)
    let percolation_kappa_gt_2 = sum_k > 0 && sum_k2 > 2 * sum_k;

    let mean_degree_c = (sum_k as f64) / (active_n as f64);
    let molloy_reed_kappa = if sum_k > 0 {
        (sum_k2 as f64) / (sum_k as f64)
    } else {
        0.0
    };

    let total_pairs = (n as f64) * ((n - 1) as f64);
    let empirical_p = ((2 * m_shadow) as f64 / total_pairs).clamp(1e-12, 1.0 - 1e-12);
    let critical_p_c = 1.0 / (active_n as f64);
    let critical_scale = (active_n as f64).powf(2.0 / 3.0);

    // 3. Füredi–Komlós Spectral Radius & Eigenvector Centrality via 12-Step Power Iteration
    let (spectral_radius_lambda1, eigen_centrality) =
        compute_shadow_spectral_radius(n, &shadow_edges_undirected, 12);
    let expected_random_lambda1 = mean_degree_c.max((max_deg as f64).sqrt());

    // 4. Iterative Tarjan SCC to count directed feedback cycles per Union-Find component root
    let scc_cycles_by_root =
        count_nontrivial_sccs_per_root(n, &shadow_edges_directed, &mut uf_parent);

    // Group vertices by their Union-Find component root
    let mut root_vertices: Vec<Vec<SymbolId>> = vec![Vec::new(); n];
    for v in 0..n {
        let r = uf_find(v as u32, &mut uf_parent) as usize;
        root_vertices[r].push(v as SymbolId);
    }

    // 5. Evaluate Component Excess s(C) & Chernoff–Erdős Log-Surprise Lambda(C)
    let mut comp_sizes: Vec<usize> = Vec::new();
    let mut anomalies: Vec<ComponentAnomaly> = Vec::new();
    let mut poisoned_leaf_reaches_core = false;

    for r in 0..n {
        if uf_parent[r] as usize != r {
            continue;
        }
        let k = comp_v[r];
        let e = comp_e[r];
        if k == 1 && e == 0 {
            continue;
        }
        comp_sizes.push(k);

        let cyclomatic_excess = (e as isize) - (k as isize) + 1;
        let scc_cycles = scc_cycles_by_root[r];
        let surprise = compute_erdos_chernoff_surprise(n, k, e, empirical_p);
        let haz = comp_haz[r];

        // Check if this component spans package/cluster boundaries and connects a leaf/hazardous
        // dependency to core application state (cluster <= 1 or root vertex 0) when kappa > 2
        if percolation_kappa_gt_2 && k >= 3 {
            let mut has_core = false;
            let mut has_leaf_or_hazard = haz.is_hazardous();
            let mut first_cluster: Option<u16> = None;
            let mut spans_package_boundaries = symbol_to_cluster.is_empty();
            for &sym in &root_vertices[r] {
                let idx = sym as usize;
                let c_id = symbol_to_cluster.get(idx).copied().unwrap_or(1);
                if let Some(fc) = first_cluster {
                    if fc != c_id {
                        spans_package_boundaries = true;
                    }
                } else {
                    first_cluster = Some(c_id);
                }
                if idx == 0 || c_id <= 1 {
                    has_core = true;
                }
                if shadow_deg.get(idx).copied().unwrap_or(0) == 1 {
                    has_leaf_or_hazard = true;
                }
            }
            if spans_package_boundaries && has_core && has_leaf_or_hazard {
                poisoned_leaf_reaches_core = true;
            }
        }

        let is_anomalous = surprise >= surprise_threshold_nats
            || cyclomatic_excess >= 2
            || scc_cycles > 0
            || (k >= 3 && haz.distinct_capabilities() >= 2);

        if is_anomalous && k >= 2 {
            let mut pivots: Vec<(SymbolId, f32)> = root_vertices[r]
                .iter()
                .map(|&sym| (sym, eigen_centrality[sym as usize]))
                .collect();
            pivots.sort_unstable_by(|a, b| b.1.total_cmp(&a.1));
            pivots.truncate(5);

            let comp_spec_rad = estimate_component_spectral_radius(graph, &root_vertices[r]);

            anomalies.push(ComponentAnomaly {
                component_id: r as u32,
                vertex_count: k,
                vertices: root_vertices[r].clone(),
                edge_count: e,
                cyclomatic_excess,
                directed_scc_cycles: scc_cycles,
                erdos_log_surprise_nats: surprise,
                spectral_radius: comp_spec_rad,
                combined_hazards: haz,
                is_directed_scc: scc_cycles > 0,
                pivot_symbols: pivots,
            });
        }
    }

    comp_sizes.sort_unstable_by_key(|&sz| std::cmp::Reverse(sz));
    let giant_component_size = comp_sizes.first().copied().unwrap_or(0);
    let second_component_size = comp_sizes.get(1).copied().unwrap_or(0);

    // Also run full-graph Tarjan SCC to catch tight intra-cluster cyclic hook injectors
    let mut next_comp_id = (n as u32).saturating_add(1);
    detect_tarjan_scc_anomalies(
        graph,
        hazards,
        &eigen_centrality,
        empirical_p,
        surprise_threshold_nats,
        &mut next_comp_id,
        &mut anomalies,
    );

    // Sort anomalies by highest Erdős log-surprise + hazard multiplier + cyclomatic excess
    anomalies.sort_unstable_by(|a, b| {
        let score_a = (a.erdos_log_surprise_nats + a.cyclomatic_excess.max(0) as f64)
            * (1.0 + a.combined_hazards.distinct_capabilities() as f64);
        let score_b = (b.erdos_log_surprise_nats + b.cyclomatic_excess.max(0) as f64)
            * (1.0 + b.combined_hazards.distinct_capabilities() as f64);
        score_b.total_cmp(&score_a)
    });

    // 6. Classify Global Erdős–Rényi Phase Regime
    // Whiteboard theorem:
    // - *1: Because dependency graphs have skewed degree distributions, we supplement
    //   *2 (<k> = c = 2|E_SHADOW|/|V|) with *3 (kappa = <k^2>/<k> = sum(deg(v)^2)/sum(deg(v))).
    // - WHEN kappa > 2 (sum_k2 > 2 * sum_k): a giant transitive reachability component forms
    //   across package boundaries allowing a single poisoned leaf dependency to reach core application state.
    let has_c2_anomaly = anomalies.iter().any(|a| {
        a.erdos_log_surprise_nats > surprise_threshold_nats * 1.25
            || a.combined_hazards.distinct_capabilities() >= 3
            || (a.directed_scc_cycles > 0 && a.combined_hazards.is_hazardous())
    }) || (spectral_radius_lambda1 > 2.2 * expected_random_lambda1.max(1.5)
        && !anomalies.is_empty());

    let regime = if has_c2_anomaly {
        PhaseRegime::AnomalousC2Clique
    } else if percolation_kappa_gt_2 {
        PhaseRegime::SupercriticalGiant
    } else if (sum_k > 0 && sum_k2 == 2 * sum_k) || raw_c_num >= raw_c_den {
        PhaseRegime::CriticalWindow
    } else {
        PhaseRegime::SubcriticalForest
    };

    let percolation_diagnostic = if percolation_kappa_gt_2 {
        format!(
            "WHEN kappa = {}/{} > 2 (supplementing <k> = c = 2|E_SHADOW|/|V| = {}/{} for skewed degree distribution): a giant transitive reachability component forms across package boundaries allowing a single poisoned leaf dependency to reach core application state (|C_1|={}, reachable={})",
            molloy_reed_kappa_num,
            molloy_reed_kappa_den,
            mean_degree_c_num,
            mean_degree_c_den,
            giant_component_size,
            poisoned_leaf_reaches_core
        )
    } else {
        format!(
            "Subcritical/bounded percolation: kappa = {}/{} <= 2 (<k> = c = 2|E_SHADOW|/|V| = {}/{}, |C_1|={}) — cross-package reachability remains fragmented",
            molloy_reed_kappa_num,
            molloy_reed_kappa_den,
            mean_degree_c_num,
            mean_degree_c_den,
            giant_component_size
        )
    };

    ErdosRenyiSecurityReport {
        num_vertices: n,
        num_edges: m,
        active_vertices: active_n,
        shadow_edges: m_shadow,
        empirical_p,
        critical_p_c,
        mean_degree_c,
        mean_degree_c_num,
        mean_degree_c_den,
        molloy_reed_kappa,
        molloy_reed_kappa_num,
        molloy_reed_kappa_den,
        percolation_kappa_gt_2,
        poisoned_leaf_reaches_core,
        percolation_diagnostic,
        giant_component_size,
        second_component_size,
        critical_scale_n_two_thirds: critical_scale,
        spectral_radius_lambda1,
        expected_random_lambda1,
        regime,
        anomalies,
    }
}

#[inline(always)]
fn uf_find(mut x: u32, parent: &mut [u32]) -> u32 {
    while parent[x as usize] != x {
        let gp = parent[parent[x as usize] as usize];
        parent[x as usize] = gp;
        x = gp;
    }
    x
}

#[inline]
fn uf_union_with_excess(
    u: u32,
    v: u32,
    parent: &mut [u32],
    rank: &mut [u8],
    comp_v: &mut [usize],
    comp_e: &mut [usize],
    comp_haz: &mut [SymbolHazardFlags],
) {
    let ru = uf_find(u, parent) as usize;
    let rv = uf_find(v, parent) as usize;
    if ru == rv {
        // Edge connects two vertices already in the same component -> increments cyclomatic surplus s(C)!
        comp_e[ru] += 1;
        return;
    }
    let (root, child) = if rank[ru] >= rank[rv] {
        (ru, rv)
    } else {
        (rv, ru)
    };
    parent[child] = root as u32;
    if rank[ru] == rank[rv] {
        rank[root] = rank[root].saturating_add(1);
    }
    comp_v[root] += comp_v[child];
    comp_e[root] += comp_e[child] + 1;
    comp_haz[root] = comp_haz[root].union(comp_haz[child]);
}

/// 12-iteration Power Method to compute $\lambda_1(A_{\text{shadow}})$ and eigenvector centrality $\mathbf{v}_1$
fn compute_shadow_spectral_radius(
    n: usize,
    edges: &[(u32, u32)],
    iterations: usize,
) -> (f64, Vec<f32>) {
    if n == 0 || edges.is_empty() {
        return (0.0, vec![0.0; n]);
    }
    let inv_sqrt_n = 1.0 / (n as f32).sqrt();
    let mut x = vec![inv_sqrt_n; n];
    let mut next_x = vec![0.0_f32; n];
    let mut rayleigh = 0.0_f64;

    for _ in 0..iterations {
        next_x.fill(0.0);
        for &(u, v) in edges {
            let iu = u as usize;
            let iv = v as usize;
            next_x[iu] += x[iv];
            next_x[iv] += x[iu];
        }

        let mut dot_num = 0.0_f64;
        let mut norm_sq = 0.0_f64;
        for i in 0..n {
            dot_num += (x[i] as f64) * (next_x[i] as f64);
            norm_sq += (next_x[i] as f64) * (next_x[i] as f64);
        }
        rayleigh = dot_num;
        let norm = norm_sq.sqrt() as f32;
        if norm <= 1e-12 {
            break;
        }
        let inv_norm = 1.0 / norm;
        for i in 0..n {
            x[i] = next_x[i] * inv_norm;
        }
    }

    (rayleigh, x)
}

/// Iterative Tarjan SCC algorithm to count non-trivial directed cycles ($\ge 2$ vertices) per Union-Find root
fn count_nontrivial_sccs_per_root(
    n: usize,
    directed_edges: &[(u32, u32)],
    uf_parent: &mut [u32],
) -> Vec<usize> {
    let mut head = vec![u32::MAX; n];
    let mut next_edge = vec![u32::MAX; directed_edges.len()];
    let mut to_vert = vec![0u32; directed_edges.len()];

    for (idx, &(u, v)) in directed_edges.iter().enumerate() {
        to_vert[idx] = v;
        next_edge[idx] = head[u as usize];
        head[u as usize] = idx as u32;
    }

    let mut index_counter = 1u32;
    let mut disc = vec![0u32; n];
    let mut low = vec![0u32; n];
    let mut on_stack = vec![false; n];
    let mut scc_stack: Vec<u32> = Vec::new();
    let mut call_stack: Vec<(u32, u32)> = Vec::new();
    let mut scc_counts = vec![0usize; n];

    for start in 0..n {
        if disc[start] != 0 || head[start] == u32::MAX {
            continue;
        }
        disc[start] = index_counter;
        low[start] = index_counter;
        index_counter += 1;
        on_stack[start] = true;
        scc_stack.push(start as u32);
        call_stack.push((start as u32, head[start]));

        while let Some((u, e_idx)) = call_stack.pop() {
            let iu = u as usize;
            if e_idx != u32::MAX {
                let nxt = next_edge[e_idx as usize];
                call_stack.push((u, nxt));

                let v = to_vert[e_idx as usize];
                let iv = v as usize;
                if disc[iv] == 0 {
                    disc[iv] = index_counter;
                    low[iv] = index_counter;
                    index_counter += 1;
                    on_stack[iv] = true;
                    scc_stack.push(v);
                    call_stack.push((v, head[iv]));
                } else if on_stack[iv] {
                    low[iu] = low[iu].min(disc[iv]);
                }
            } else {
                if let Some(&(parent_u, _)) = call_stack.last() {
                    low[parent_u as usize] = low[parent_u as usize].min(low[iu]);
                }
                if low[iu] == disc[iu] {
                    let mut sz = 0usize;
                    while let Some(w) = scc_stack.pop() {
                        on_stack[w as usize] = false;
                        sz += 1;
                        if w == u {
                            break;
                        }
                    }
                    if sz >= 2 {
                        let root = uf_find(u, uf_parent) as usize;
                        scc_counts[root] += 1;
                    }
                }
            }
        }
    }

    scc_counts
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
                let mut common_w = Vec::new();
                for &w in u_neighbors {
                    if w > v && v_neighbors.binary_search(&w).is_ok() {
                        common_w.push(w);
                    }
                }
                for (idx_w, &w) in common_w.iter().enumerate() {
                    *counter += 1;
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
    let m_s = (s * (s - 1)) as f64;
    let q = ((edges as f64) / m_s).clamp(1e-12, 1.0 - 1e-12);
    if q <= p {
        return 0.0;
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
    eigen_centrality: &[f32],
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
    let mut dfs_stack: Vec<(usize, usize)> = Vec::new();

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
                            let mut pivots: Vec<(SymbolId, f32)> = scc_verts
                                .iter()
                                .map(|&sym| {
                                    let c =
                                        eigen_centrality.get(sym as usize).copied().unwrap_or(0.0);
                                    (sym, c)
                                })
                                .collect();
                            pivots.sort_unstable_by(|a, b| b.1.total_cmp(&a.1));
                            pivots.truncate(5);

                            out_anomalies.push(ComponentAnomaly {
                                component_id: *comp_id_counter,
                                vertex_count: s,
                                vertices: scc_verts,
                                edge_count: internal_edges,
                                cyclomatic_excess: cyc,
                                directed_scc_cycles: 1,
                                erdos_log_surprise_nats: surprise,
                                spectral_radius: spec_rad,
                                combined_hazards: combined_haz,
                                is_directed_scc: true,
                                pivot_symbols: pivots,
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
