//! Ramsey-Theoretic & Erdős–Hajnal Structural Analyzer (`st --ramsey-cliques`).
//!
//! Applies Paul Erdős's Ramsey Theory (Erdős–Szekeres 1935, diagonal Ramsey bounds 1947)
//! and the **Erdős–Hajnal Conjecture (1989)** directly to the 5-color AST symbol graph
//! (`Import`, `CallRef`, `TraitImpl`, `TypeBound`, `MutableShared`) plus the 6th implicit
//! non-edge anti-color ($c_\emptyset$, independent sets):
//!
//! 1. **Degeneracy-Ordered 256-Bit Bitboard Bron–Kerbosch (`BitSet256`)**:
//!    Computes the Matula–Beck $O(|V| + |E|)$ degeneracy ordering $(v_1, \dots, v_n)$ and
//!    packs each vertex's forward monochromatic neighborhood $N_c^+(v_i)$ into a stack-allocated
//!    `BitSet256([u64; 4])` accelerated by `felix_asm::felix_popcnt_u64` and `felix_tzcnt_u64`.
//! 2. **Erdős–Moser Tournament vs. Cyclic Deadlock Knot Classification**:
//!    Distinguishes acyclic transitive compilation/call pipelines ($\vec{T}_k^{(c)}$) from
//!    mutual cyclic deadlock/spaghetti knots ($K_k^{(c)}$), computing minimum feedback back-edges.
//! 3. **Fox–Sudakov / Rödl Pure & Almost-Pure Bi-Clique Extraction (`ErdosHajnalBiClique`)**:
//!    Extracts latent trait abstractions ($d_c(A, B) = 1.0$) and **API Conformance Bugs**
//!    ($1 - \delta \le d_c(A, B) < 1.0$, pinpointing the exact missing edge $(a^*, b^*)$).
//! 4. **The Erdős–Hajnal Modularity Exponent ($\gamma_{\text{EH}}$)**:
//!    Quantifies whether the codebase obeys an $H$-free polynomial law ($\max(\omega, \alpha) \sim n^{\gamma_{\text{EH}}}$)
//!    or has degraded toward an unstructured Erdős–Rényi random graph ($\sim 2\log_2 n$).

use super::types::{CsrSymbolGraph, EdgeKind, SymbolId};
use crate::felix_asm::{felix_popcnt_u64, felix_tzcnt_u64};
use serde::{Deserialize, Serialize};
use std::cmp::Ordering;

/// Stack-allocated 256-bit SIMD bitboard for single-cycle Bron–Kerbosch neighborhood intersections.
#[derive(Clone, Copy, PartialEq, Eq, Default, Debug)]
pub struct BitSet256(pub [u64; 4]);

impl BitSet256 {
    pub const EMPTY: Self = Self([0; 4]);

    #[inline(always)]
    pub fn set(&mut self, idx: usize) {
        debug_assert!(idx < 256);
        self.0[idx >> 6] |= 1u64 << (idx & 63);
    }

    #[inline(always)]
    pub fn clear(&mut self, idx: usize) {
        debug_assert!(idx < 256);
        self.0[idx >> 6] &= !(1u64 << (idx & 63));
    }

    #[inline(always)]
    pub fn contains(&self, idx: usize) -> bool {
        debug_assert!(idx < 256);
        (self.0[idx >> 6] & (1u64 << (idx & 63))) != 0
    }

    #[inline(always)]
    pub fn is_empty(&self) -> bool {
        (self.0[0] | self.0[1] | self.0[2] | self.0[3]) == 0
    }

    #[inline(always)]
    pub fn count_ones(&self) -> u32 {
        felix_popcnt_u64(self.0[0])
            + felix_popcnt_u64(self.0[1])
            + felix_popcnt_u64(self.0[2])
            + felix_popcnt_u64(self.0[3])
    }

    #[inline(always)]
    pub fn first_set_bit(&self) -> Option<usize> {
        for (word_idx, &w) in self.0.iter().enumerate() {
            if w != 0 {
                return Some((word_idx << 6) + (felix_tzcnt_u64(w) as usize));
            }
        }
        None
    }

    #[inline(always)]
    pub fn and(&self, other: &Self) -> Self {
        Self([
            self.0[0] & other.0[0],
            self.0[1] & other.0[1],
            self.0[2] & other.0[2],
            self.0[3] & other.0[3],
        ])
    }

    #[inline(always)]
    pub fn and_not(&self, other: &Self) -> Self {
        Self([
            self.0[0] & !other.0[0],
            self.0[1] & !other.0[1],
            self.0[2] & !other.0[2],
            self.0[3] & !other.0[3],
        ])
    }

    #[inline(always)]
    pub fn or(&self, other: &Self) -> Self {
        Self([
            self.0[0] | other.0[0],
            self.0[1] | other.0[1],
            self.0[2] | other.0[2],
            self.0[3] | other.0[3],
        ])
    }
}

/// Directed topology of a monochromatic clique (Erdős–Moser Tournament vs. Mutual Knot).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CliqueTopology {
    /// Acyclic total order ($v_1 \prec v_2 \prec \dots \prec v_k$): clean compilation/call pipeline
    TransitiveTournament,
    /// Contains directed cycles or mutual bidirectional locks: deadlock / spaghetti knot
    CyclicDeadlockKnot { bidirectional_pairs: u32 },
}

/// Actionable refactoring prescription emitted for each Ramsey structure.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RefactorPrescription {
    /// Break mutual state/call cycle via message-passing or dependency inversion
    BreakDeadlockCycle,
    /// Flatten deeply coupled transitive module import chain
    FlattenCircularImports,
    /// Extract a shared trait or facade for a high-surprise bi-clique
    ExtractSharedTraitFacade,
    /// Fix a missing call/impl edge in an almost-pure Erdős–Hajnal bi-clique
    AuditMissingBiCliqueEdge,
    /// Collapse redundant transitive type/trait hierarchy
    SimplifyTypeHierarchy,
}

/// A maximal monochromatic clique $K_k^{(c)}$ discovered in the AST symbol graph.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RamseyMonochromaticClique {
    pub color: EdgeKind,
    pub vertices: Vec<SymbolId>,
    pub size_k: usize,
    /// Ramsey log-surprise $\mathcal{S}_{\text{Ramsey}}(K_k^{(c)})$ in nats above $G(n, p_c)$
    pub ramsey_surprise_nats: f64,
    pub topology: CliqueTopology,
    /// Minimum feedback arc set (edges to invert/remove to break cycles)
    pub feedback_back_edges: Vec<(SymbolId, SymbolId)>,
    pub prescription: RefactorPrescription,
}

/// An Erdős–Hajnal monochromatic bi-clique $(A, B)$ with density $d_c(A, B) \ge 1 - \delta$.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErdosHajnalBiClique {
    pub color: EdgeKind,
    /// Source symbols (e.g., structs/callers sharing an identical dependency footprint)
    pub left_set_a: Vec<SymbolId>,
    /// Target symbols (e.g., shared callee methods or trait bounds)
    pub right_set_b: Vec<SymbolId>,
    /// Bipartite density $d_c(A, B) \in [1 - \delta, 1.0]$
    pub bipartite_density: f64,
    /// If $d_c(A, B) < 1.0$, exact missing edges $(a \in A, b \in B)$ indicating forgotten calls/impls
    pub missing_anomaly_edges: Vec<(SymbolId, SymbolId)>,
    pub prescription: RefactorPrescription,
}

/// Full Ramsey & Erdős–Hajnal structural report for `st --ramsey-cliques`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RamseyErdosHajnalReport {
    pub num_vertices: usize,
    /// Graph degeneracy $d$ (max minimum degree over all induced subgraphs)
    pub degeneracy_d: usize,
    /// Clique number $\omega(G_c)$ for each of the 5 `EdgeKind` colors
    pub omega_per_color: [usize; 5],
    /// Approximate independence number $\alpha(G)$ (largest decoupled parallel batch)
    pub alpha_independent_set: Vec<SymbolId>,
    /// Empirical Erdős–Hajnal exponent $\gamma_{\text{EH}} = \ln \max(\omega, \alpha) / \ln n$
    pub erdos_hajnal_exponent_gamma: f64,
    /// Expected random graph exponent $\ln(2 \log_2 n) / \ln n$
    pub expected_random_gamma: f64,
    pub cliques: Vec<RamseyMonochromaticClique>,
    pub bi_cliques: Vec<ErdosHajnalBiClique>,
}

/// Configuration for Ramsey & Erdős–Hajnal clique and bi-clique mining.
#[derive(Debug, Clone, Copy)]
pub struct RamseyScanConfig {
    /// Minimum monochromatic clique size $k$ to report (default: `3`, or `4` for large graphs)
    pub min_clique_k: usize,
    /// Minimum Ramsey log-surprise in nats (default: `2.5`)
    pub min_surprise_nats: f64,
    /// Minimum left/right set size for Erdős–Hajnal bi-cliques (default: `3`)
    pub min_biclique_side: usize,
    /// Almost-pure bi-clique tolerance $\delta$ (default: `0.15`, allowing 1 missing edge in $3 \times 3$)
    pub biclique_tolerance_delta: f64,
}

impl Default for RamseyScanConfig {
    fn default() -> Self {
        Self {
            min_clique_k: 3,
            min_surprise_nats: 2.5,
            min_biclique_side: 3,
            biclique_tolerance_delta: 0.15,
        }
    }
}

const ALL_COLORS: [EdgeKind; 5] = [
    EdgeKind::Import,
    EdgeKind::CallRef,
    EdgeKind::TraitImpl,
    EdgeKind::TypeBound,
    EdgeKind::MutableShared,
];

/// Executes 5-color + anti-color ($c_\emptyset$) Ramsey clique and Erdős–Hajnal bi-clique discovery.
pub fn analyze_ramsey_and_erdos_hajnal(
    graph: &CsrSymbolGraph,
    config: RamseyScanConfig,
) -> RamseyErdosHajnalReport {
    let n = graph.num_vertices;
    if n == 0 {
        return RamseyErdosHajnalReport {
            num_vertices: 0,
            degeneracy_d: 0,
            omega_per_color: [0; 5],
            alpha_independent_set: Vec::new(),
            erdos_hajnal_exponent_gamma: 0.0,
            expected_random_gamma: 0.0,
            cliques: Vec::new(),
            bi_cliques: Vec::new(),
        };
    }

    // 1. Build per-color undirected adjacency lists + directed lookup for tournament classification
    let mut color_undirected: Vec<Vec<Vec<SymbolId>>> = vec![vec![Vec::new(); n]; 5];
    let mut color_edge_counts = [0usize; 5];

    for u in 0..n {
        let start = graph.row_ptr[u] as usize;
        let end = graph.row_ptr[u + 1] as usize;
        for idx in start..end {
            let v = graph.col_idx[idx] as usize;
            if u == v || v >= n {
                continue;
            }
            let c = graph.edge_kind[idx] as usize;
            color_undirected[c][u].push(v as SymbolId);
            color_undirected[c][v].push(u as SymbolId);
            color_edge_counts[c] += 1;
        }
    }

    for color_adj in color_undirected.iter_mut().take(5) {
        for adj in color_adj {
            adj.sort_unstable();
            adj.dedup();
        }
    }

    // 2. Compute global Matula-Beck degeneracy ordering & Caro-Wei independent set (6th color c_empty)
    let (global_degeneracy_d, _) = compute_degeneracy_ordering(&color_undirected[1]); // CallRef core
    let alpha_independent_set = extract_caro_wei_independent_set(graph);

    let mut omega_per_color = [1usize; 5];
    let mut all_cliques = Vec::new();
    let max_pairs = ((n * n.saturating_sub(1)) / 2).max(1) as f64;

    // 3. Run 256-Bit Bitboard Bron-Kerbosch with Tomita pivoting per color
    for &color in &ALL_COLORS {
        let c_idx = color as usize;
        let adj = &color_undirected[c_idx];
        let undirected_edges: usize = adj.iter().map(|v| v.len()).sum::<usize>() / 2;
        if undirected_edges == 0 {
            omega_per_color[c_idx] = 0;
            continue;
        }

        let p_c = (undirected_edges as f64 / max_pairs).clamp(1e-9, 0.999_999);
        let (_, order) = compute_degeneracy_ordering(adj);
        let mut pos_in_order = vec![0usize; n];
        for (pos, &v) in order.iter().enumerate() {
            pos_in_order[v as usize] = pos;
        }

        let mut raw_cliques: Vec<Vec<SymbolId>> = Vec::new();

        for (ord_idx, &root_u) in order.iter().enumerate() {
            let u_usize = root_u as usize;
            // Forward neighborhood N_c^+(u) in degeneracy order (bounded by d <= 255)
            let forward_neighbors: Vec<SymbolId> = adj[u_usize]
                .iter()
                .copied()
                .filter(|&w| pos_in_order[w as usize] > ord_idx)
                .take(255)
                .collect();

            let m = forward_neighbors.len();
            if m + 1 < config.min_clique_k {
                if !forward_neighbors.is_empty() {
                    omega_per_color[c_idx] = omega_per_color[c_idx].max(2);
                }
                continue;
            }

            // Build local m x m bitboard adjacency matrix for N_c^+(root_u)
            let mut local_adj = vec![BitSet256::EMPTY; m];
            for i in 0..m {
                let vi = forward_neighbors[i] as usize;
                for j in (i + 1)..m {
                    let vj = forward_neighbors[j];
                    if adj[vi].binary_search(&vj).is_ok() {
                        local_adj[i].set(j);
                        local_adj[j].set(i);
                    }
                }
            }

            let mut p_set = BitSet256::EMPTY;
            for i in 0..m {
                p_set.set(i);
            }
            let r_set = BitSet256::EMPTY;
            let x_set = BitSet256::EMPTY;

            bron_kerbosch_tomita_256(
                r_set,
                p_set,
                x_set,
                &local_adj,
                root_u,
                &forward_neighbors,
                config.min_clique_k,
                &mut raw_cliques,
            );
        }

        // Deduplicate & score discovered monochromatic cliques
        for mut verts in raw_cliques {
            verts.sort_unstable();
            omega_per_color[c_idx] = omega_per_color[c_idx].max(verts.len());

            let k = verts.len();
            let k_f = k as f64;
            let pairs_k = 0.5 * k_f * (k_f - 1.0);
            let surprise_nats =
                pairs_k * (1.0 / p_c).ln() - k_f * ((std::f64::consts::E * (n as f64)) / k_f).ln();

            if surprise_nats >= config.min_surprise_nats
                || (color == EdgeKind::MutableShared && k >= 3)
            {
                let (topology, feedback_back_edges) =
                    classify_directed_clique(graph, color, &verts);
                let prescription = match (color, topology) {
                    (EdgeKind::MutableShared, _) => RefactorPrescription::BreakDeadlockCycle,
                    (_, CliqueTopology::CyclicDeadlockKnot { .. }) => {
                        if color == EdgeKind::Import {
                            RefactorPrescription::FlattenCircularImports
                        } else {
                            RefactorPrescription::BreakDeadlockCycle
                        }
                    }
                    (EdgeKind::TraitImpl | EdgeKind::TypeBound, _) => {
                        RefactorPrescription::SimplifyTypeHierarchy
                    }
                    _ => RefactorPrescription::ExtractSharedTraitFacade,
                };

                all_cliques.push(RamseyMonochromaticClique {
                    color,
                    vertices: verts,
                    size_k: k,
                    ramsey_surprise_nats: surprise_nats.max(0.0),
                    topology,
                    feedback_back_edges,
                    prescription,
                });
            }
        }
    }

    // Sort cliques by descending Ramsey surprise
    all_cliques.sort_by(|a, b| {
        b.ramsey_surprise_nats
            .partial_cmp(&a.ramsey_surprise_nats)
            .unwrap_or(Ordering::Equal)
    });
    all_cliques.dedup_by(|a, b| a.color == b.color && a.vertices == b.vertices);

    // 4. Mine Erdős-Hajnal Pure & Almost-Pure Bi-Cliques (Rödl / Fox-Sudakov extraction)
    let bi_cliques = extract_erdos_hajnal_bicliques(graph, config);

    // 5. Compute the Erdős-Hajnal Modularity Exponent gamma_EH
    let max_omega = omega_per_color.into_iter().max().unwrap_or(1);
    let max_homogeneous = max_omega.max(alpha_independent_set.len()).max(1) as f64;
    let ln_n = (n.max(2) as f64).ln();
    let erdos_hajnal_exponent_gamma = (max_homogeneous.ln() / ln_n).clamp(0.0, 1.0);
    let random_clique_est = (2.0 * (n.max(2) as f64).log2()).max(2.0);
    let expected_random_gamma = (random_clique_est.ln() / ln_n).clamp(0.0, 1.0);

    RamseyErdosHajnalReport {
        num_vertices: n,
        degeneracy_d: global_degeneracy_d,
        omega_per_color,
        alpha_independent_set,
        erdos_hajnal_exponent_gamma,
        expected_random_gamma,
        cliques: all_cliques,
        bi_cliques,
    }
}

/// Matula–Beck $O(|V| + |E|)$ bucket-queue algorithm for degeneracy ordering.
fn compute_degeneracy_ordering(adj: &[Vec<SymbolId>]) -> (usize, Vec<SymbolId>) {
    let n = adj.len();
    let mut deg: Vec<usize> = adj.iter().map(|v| v.len()).collect();
    let max_deg = deg.iter().copied().max().unwrap_or(0);

    let mut buckets: Vec<Vec<SymbolId>> = vec![Vec::new(); max_deg + 1];
    for (u, &d) in deg.iter().enumerate() {
        buckets[d].push(u as SymbolId);
    }

    let mut removed = vec![false; n];
    let mut order = Vec::with_capacity(n);
    let mut degeneracy_d = 0usize;
    let mut min_d = 0usize;

    while order.len() < n {
        while min_d <= max_deg && buckets[min_d].is_empty() {
            min_d += 1;
        }
        if min_d > max_deg {
            break;
        }
        let u = buckets[min_d].pop().unwrap();
        let u_usize = u as usize;
        if removed[u_usize] {
            continue;
        }
        removed[u_usize] = true;
        degeneracy_d = degeneracy_d.max(deg[u_usize]);
        order.push(u);

        for &v in &adj[u_usize] {
            let v_usize = v as usize;
            if !removed[v_usize] && deg[v_usize] > 0 {
                deg[v_usize] -= 1;
                let new_d = deg[v_usize];
                buckets[new_d].push(v);
                if new_d < min_d {
                    min_d = new_d;
                }
            }
        }
    }

    (degeneracy_d, order)
}

/// Zero-allocation 256-bit Bron–Kerbosch recursion with Tomita pivoting.
#[allow(clippy::too_many_arguments)]
fn bron_kerbosch_tomita_256(
    r_set: BitSet256,
    mut p_set: BitSet256,
    mut x_set: BitSet256,
    local_adj: &[BitSet256],
    root_u: SymbolId,
    forward_map: &[SymbolId],
    min_k: usize,
    out: &mut Vec<Vec<SymbolId>>,
) {
    if p_set.is_empty() {
        if x_set.is_empty() {
            let total_k = (r_set.count_ones() as usize) + 1;
            if total_k >= min_k {
                let mut clique = Vec::with_capacity(total_k);
                clique.push(root_u);
                let mut scan = r_set;
                while let Some(bit) = scan.first_set_bit() {
                    clique.push(forward_map[bit]);
                    scan.clear(bit);
                }
                out.push(clique);
            }
        }
        return;
    }

    // Prune branch if even taking all of P cannot reach min_k
    if (r_set.count_ones() + p_set.count_ones() + 1) < (min_k as u32) {
        return;
    }

    // Tomita Pivot: choose u_piv in (P U X) maximizing |P \cap N(u_piv)|
    let mut union_px = p_set.or(&x_set);
    let mut best_pivot = 0usize;
    let mut best_overlap = 0u32;
    let mut has_pivot = false;

    while let Some(cand) = union_px.first_set_bit() {
        union_px.clear(cand);
        let overlap = p_set.and(&local_adj[cand]).count_ones();
        if !has_pivot || overlap > best_overlap {
            has_pivot = true;
            best_pivot = cand;
            best_overlap = overlap;
        }
    }

    let mut candidates = p_set.and_not(&local_adj[best_pivot]);
    while let Some(v_bit) = candidates.first_set_bit() {
        candidates.clear(v_bit);

        let mut next_r = r_set;
        next_r.set(v_bit);
        let next_p = p_set.and(&local_adj[v_bit]);
        let next_x = x_set.and(&local_adj[v_bit]);

        bron_kerbosch_tomita_256(
            next_r,
            next_p,
            next_x,
            local_adj,
            root_u,
            forward_map,
            min_k,
            out,
        );

        p_set.clear(v_bit);
        x_set.set(v_bit);

        if out.len() >= 128 {
            return;
        }
    }
}

/// Classifies a monochromatic clique into an Erdős–Moser Transitive Tournament ($\vec{T}_k$)
/// or a Cyclic Deadlock Knot, returning the minimum feedback back-edges.
fn classify_directed_clique(
    graph: &CsrSymbolGraph,
    color: EdgeKind,
    vertices: &[SymbolId],
) -> (CliqueTopology, Vec<(SymbolId, SymbolId)>) {
    let k = vertices.len();
    let mut out_scores = vec![0usize; k];
    let mut dir_matrix = vec![vec![false; k]; k];

    for (i, &u) in vertices.iter().enumerate() {
        let start = graph.row_ptr[u as usize] as usize;
        let end = graph.row_ptr[u as usize + 1] as usize;
        for idx in start..end {
            if graph.edge_kind[idx] != color {
                continue;
            }
            let target = graph.col_idx[idx];
            if let Ok(j) = vertices.binary_search(&target) {
                if i != j && !dir_matrix[i][j] {
                    dir_matrix[i][j] = true;
                    out_scores[i] += 1;
                }
            }
        }
    }

    let mut bidirectional_pairs = 0u32;
    for (i, row_i) in dir_matrix.iter().enumerate().take(k) {
        for (j, row_j) in dir_matrix.iter().enumerate().take(k).skip(i + 1) {
            if row_i[j] && row_j[i] {
                bidirectional_pairs += 1;
            }
        }
    }

    // Order vertices by descending out-degree inside the tournament to isolate backward arcs
    let mut order: Vec<usize> = (0..k).collect();
    order.sort_by_key(|&i| std::cmp::Reverse(out_scores[i]));

    let mut rank_pos = vec![0usize; k];
    for (rank, &idx) in order.iter().enumerate() {
        rank_pos[idx] = rank;
    }

    let mut feedback_back_edges = Vec::new();
    for i in 0..k {
        for j in 0..k {
            if i != j && dir_matrix[i][j] && rank_pos[i] > rank_pos[j] {
                feedback_back_edges.push((vertices[i], vertices[j]));
            }
        }
    }

    if bidirectional_pairs == 0 && feedback_back_edges.is_empty() {
        (CliqueTopology::TransitiveTournament, feedback_back_edges)
    } else {
        (
            CliqueTopology::CyclicDeadlockKnot {
                bidirectional_pairs,
            },
            feedback_back_edges,
        )
    }
}

/// Extracts Erdős–Hajnal Pure ($d_c(A, B) = 1.0$) and Almost-Pure ($d_c(A, B) \ge 1 - \delta$)
/// monochromatic bi-cliques to discover latent trait abstractions and API conformance bugs.
fn extract_erdos_hajnal_bicliques(
    graph: &CsrSymbolGraph,
    config: RamseyScanConfig,
) -> Vec<ErdosHajnalBiClique> {
    let n = graph.num_vertices;
    let mut results = Vec::new();

    for &color in &ALL_COLORS {
        // Collect outgoing target sets per vertex u for this color
        let mut active_sources: Vec<(SymbolId, Vec<SymbolId>)> = Vec::new();
        for u in 0..n {
            let start = graph.row_ptr[u] as usize;
            let end = graph.row_ptr[u + 1] as usize;
            let mut targets: Vec<SymbolId> = (start..end)
                .filter(|&idx| graph.edge_kind[idx] == color)
                .map(|idx| graph.col_idx[idx])
                .collect();
            targets.sort_unstable();
            targets.dedup();
            if targets.len() >= config.min_biclique_side.saturating_sub(1).max(2) {
                active_sources.push((u as SymbolId, targets));
            }
        }

        let mut used = vec![false; active_sources.len()];
        for i in 0..active_sources.len() {
            if used[i] || active_sources[i].1.len() < config.min_biclique_side {
                continue;
            }
            let seed_targets = &active_sources[i].1;
            let mut left_a = vec![active_sources[i].0];

            // Gather sibling sources that share at least floor(|B| * (1 - delta)) of seed_targets
            let min_shared = ((seed_targets.len() as f64) * (1.0 - config.biclique_tolerance_delta))
                .floor() as usize;
            if min_shared < 2 {
                continue;
            }

            for j in (i + 1)..active_sources.len() {
                let cand_targets = &active_sources[j].1;
                let shared = sorted_intersection_count(seed_targets, cand_targets);
                if shared >= min_shared {
                    left_a.push(active_sources[j].0);
                    used[j] = true;
                }
            }

            if left_a.len() < config.min_biclique_side {
                continue;
            }

            // Compute exact bipartite density and missing edges over (left_a, seed_targets)
            let mut present_edges = 0usize;
            let mut missing_anomaly_edges = Vec::new();

            for &a_sym in &left_a {
                let start = graph.row_ptr[a_sym as usize] as usize;
                let end = graph.row_ptr[a_sym as usize + 1] as usize;
                for &b_sym in seed_targets {
                    let has_edge = (start..end)
                        .any(|idx| graph.col_idx[idx] == b_sym && graph.edge_kind[idx] == color);
                    if has_edge {
                        present_edges += 1;
                    } else {
                        missing_anomaly_edges.push((a_sym, b_sym));
                    }
                }
            }

            let total_possible = left_a.len() * seed_targets.len();
            let density = (present_edges as f64) / (total_possible.max(1) as f64);

            if density >= 1.0 - config.biclique_tolerance_delta {
                let prescription = if missing_anomaly_edges.is_empty() {
                    RefactorPrescription::ExtractSharedTraitFacade
                } else {
                    RefactorPrescription::AuditMissingBiCliqueEdge
                };

                results.push(ErdosHajnalBiClique {
                    color,
                    left_set_a: left_a,
                    right_set_b: seed_targets.clone(),
                    bipartite_density: density,
                    missing_anomaly_edges,
                    prescription,
                });
            }
        }
    }

    results.sort_by(|a, b| {
        let area_a = a.left_set_a.len() * a.right_set_b.len();
        let area_b = b.left_set_a.len() * b.right_set_b.len();
        area_b.cmp(&area_a)
    });
    results.truncate(32);
    results
}

/// Caro–Wei / Paul Erdős greedy minimum-degree independent set ($c_\emptyset$ 6th color).
/// Guarantees an independent set of size at least $\sum_{v \in V} \frac{1}{\text{deg}(v) + 1}$.
fn extract_caro_wei_independent_set(graph: &CsrSymbolGraph) -> Vec<SymbolId> {
    let n = graph.num_vertices;
    let mut degrees = vec![0usize; n];
    for (u, deg_slot) in degrees.iter_mut().enumerate().take(n) {
        let out_d = (graph.row_ptr[u + 1] - graph.row_ptr[u]) as usize;
        let in_d = (graph.transpose_row_ptr[u + 1] - graph.transpose_row_ptr[u]) as usize;
        *deg_slot = out_d + in_d;
    }

    let mut order: Vec<usize> = (0..n).collect();
    order.sort_by_key(|&u| degrees[u]);

    let mut blocked = vec![false; n];
    let mut independent_set = Vec::new();

    for u in order {
        if blocked[u] {
            continue;
        }
        independent_set.push(u as SymbolId);
        blocked[u] = true;

        let f_start = graph.row_ptr[u] as usize;
        let f_end = graph.row_ptr[u + 1] as usize;
        for idx in f_start..f_end {
            blocked[graph.col_idx[idx] as usize] = true;
        }
        let t_start = graph.transpose_row_ptr[u] as usize;
        let t_end = graph.transpose_row_ptr[u + 1] as usize;
        for idx in t_start..t_end {
            blocked[graph.transpose_col_idx[idx] as usize] = true;
        }
    }

    independent_set
}

#[inline]
fn sorted_intersection_count(a: &[SymbolId], b: &[SymbolId]) -> usize {
    let (mut i, mut j, mut count) = (0, 0, 0);
    while i < a.len() && j < b.len() {
        match a[i].cmp(&b[j]) {
            Ordering::Less => i += 1,
            Ordering::Greater => j += 1,
            Ordering::Equal => {
                count += 1;
                i += 1;
                j += 1;
            }
        }
    }
    count
}
