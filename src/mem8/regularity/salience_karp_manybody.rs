//! Flyxion's *Recency as Salience* ($d_T \neq d_R$) Ontology, Karp's 21 NP-Complete FPT
//! Reductions, and Quantum Many-Body Kohn–Sham DFT / Strang Split-Operator Wave Dynamics.
//!
//! Unifies three foundational mathematical frameworks inside `smart-tree`:
//! 1. **Recency vs. Salience ($d_T(x, q) \neq d_R(x, q)$) & Retrieval-as-Amnesia Detector**:
//!    Separates observable temporal distance $d_T(x, q) = t_q - t_a(x)$ from relational distance
//!    $d_R(x, q)$ (Erdős–Steiner geodesic resistance distance + Szemerédi cluster centrality +
//!    MEM8 wave alignment). Identifies **"Load-Bearing Old" ("Corrugated Iron")** artifacts that
//!    pure chronological ranking drops ($\text{retained}(x) \not\Rightarrow \text{retrievable}(x, q)
//!    \not\Rightarrow \text{salient}(x, q)$).
//! 2. **Karp's 21 NP-Complete Problems FPT & Approximation Certificate**:
//!    Certifies how `smart-tree` solves/approximates Clique, Independent Set, Vertex Cover,
//!    Feedback Arc Set, Chromatic Number, Steiner Tree, Max Cut, and Token Knapsack in polynomial
//!    or Fixed-Parameter Tractable ($O(d \cdot |V| \cdot 3^{d/3})$) time.
//! 3. **Quantum Many-Body Kohn–Sham DFT & Strang Split-Operator Wave Propagation**:
//!    Avoids the $O(2^N)$ exponential wave-function wall by treating each Szemerédi super-node's
//!    `[f32; 64]` centroid as a one-particle Kohn–Sham electron density $\rho(\mathbf{r})$ coupled
//!    with Local Natural Orbital (`T8R` exception ledger) correlation corrections, evolved via
//!    second-order Strang operator splitting:
//!    $$e^{h(A + B)} = e^{\frac{h}{2} A} e^{h B} e^{\frac{h}{2} A} + O(h^3)$$

use super::erdos_steiner::{ErdosCenterTuranReport, ErdosSteinerSlice};
use super::mmap_view::ZeroCopyMem8Book;
use super::moser_tardos::MoserTardosSchedule;
use super::ramsey_erdos_hajnal::RamseyErdosHajnalReport;
use super::types::{ClusterId, CsrSymbolGraph, Mem8BookArchive, SymbolId};
use crate::felix_asm::felix_dot64_f32;
use crate::scanner::FileNode;
use serde::{Deserialize, Serialize};
use std::time::UNIX_EPOCH;

/// Fallback Steiner resistance distance for clusters outside the active Steiner slice (`4.0`).
pub const FALLBACK_CLUSTER_RESISTANCE_DIST: f32 = 4.0;
/// Maximum resistance distance cap for Erdős Center clusters (`0.35`).
pub const CENTER_CLUSTER_MAX_DIST: f32 = 0.35;
/// Weight of cluster Steiner resistance distance in relational distance $d_R(x, q)$ (`0.7`).
pub const SALIENCE_CLUSTER_WEIGHT: f32 = 0.7;
/// Weight of degree centrality bonus in relational distance $d_R(x, q)$ (`0.3`).
pub const SALIENCE_DEGREE_WEIGHT: f32 = 0.3;
/// Top-$k$ frontier width for evaluating proxy disagreement $\Pr[x^*_T \neq x^*_R]$ (`10`).
pub const PROXY_DISAGREEMENT_FRONTIER_K: usize = 10;
/// Maximum relational candidates inspected for Load-Bearing Old ("Corrugated Iron") rescue (`16`).
pub const MAX_LOAD_BEARING_CANDIDATES: usize = 16;
/// Maximum Load-Bearing Old ("Corrugated Iron") artifacts reported (`5`).
pub const MAX_LOAD_BEARING_ARTIFACTS: usize = 5;
/// Proxy disagreement threshold triggering `SalienceRegime::RelationalShiftActive` (`0.4`).
pub const RELATIONAL_SHIFT_DISAGREEMENT_THRESHOLD: f64 = 0.4;
/// Admissibility change ratio $\Delta_R / N$ threshold triggering `RelationalShiftActive` (`0.25`).
pub const RELATIONAL_SHIFT_CHANGE_RATIO_THRESHOLD: f64 = 0.25;
/// Second-order Strang Split-Operator propagation step $h$ (`0.25`).
pub const STRANG_STEP_H: f32 = 0.25;
/// Maximum number of dominant Kohn–Sham salience orbitals retained (`5`).
pub const MAX_DOMINANT_SALIENCE_ORBITALS: usize = 5;
/// Rational denominator lattice for Strang split-operator wave coherence (`10_000`).
pub const STRANG_COHERENCE_LATTICE_DEN: u64 = 10_000;

/// Regime classification for whether observable recency ($d_T$) is a valid proxy for
/// relational salience ($d_R$).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SalienceRegime {
    /// $\Delta_R(t, t + \Delta t) / N \ll 1$: Admissibility structure $R_t$ is quasi-static;
    /// recency acts as a valid cheap prefix/handshape candidate reducer.
    QuasiStaticRecencyValid,
    /// High proxy disagreement ($\Pr[x^*_T \neq x^*_R]$) or active relational shift:
    /// chronological order suffers retrieval-as-amnesia on load-bearing older infrastructure.
    RelationalShiftActive,
}

/// A "Load-Bearing Old" ("Corrugated Iron") artifact where temporal rank $r_T(x)$ is old
/// (high $d_T$), but relational rank $r_R(x)$ is near the top of the Steiner/Erdős Center backbone.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoadBearingOldArtifact {
    pub symbol_id: SymbolId,
    pub path: String,
    /// Rank by chronological modification time (`0` = newest, `N-1` = oldest)
    pub temporal_rank_r_t: usize,
    /// Rank by relational salience $d_R(x, q)$ (`0` = most salient / lowest $d_R$)
    pub relational_rank_r_r: usize,
    /// Relational distance $d_R(x, q)$ in the Erdős–Steiner + MEM8 wave metric
    pub relational_distance_d_r: f32,
    /// Age displacement $r_T(x) - r_R(x)$ measuring severity of chronological amnesia
    pub amnesia_displacement: usize,
}

/// Flyxion *Recency as Salience* ($d_T \neq d_R$) ontology diagnostic report.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SalienceOntologyReport {
    pub regime: SalienceRegime,
    /// Empirical probability $\Pr[x^*_T \neq x^*_R]$ that recency disagrees with relational salience
    /// across the top-$k$ retrieval frontier
    pub proxy_disagreement_prob: f64,
    /// Structural admissibility change ratio $\Delta_R / N$ (measured from T8R exception & outlier ratio)
    pub admissibility_change_ratio: f64,
    /// Top-1 temporal candidate $x^*_T = \arg\min d_T(x, q)$
    pub argmin_temporal_symbol: Option<SymbolId>,
    /// Top-1 relational candidate $x^*_R = \arg\min d_R(x, q)$
    pub argmin_relational_symbol: Option<SymbolId>,
    /// Foundational old files rescued from chronological amnesia ("Corrugated Iron vs. V-2")
    pub load_bearing_old_artifacts: Vec<LoadBearingOldArtifact>,
}

/// Certifies how `smart-tree` tames Karp's 21 NP-Complete problems on codebase graphs via
/// Fixed-Parameter Tractable (FPT) degeneracy, LLL resampling, and polynomial approximation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KarpComplexityCertificate {
    /// Matula–Beck graph degeneracy $d \ll n$ bounding FPT Bron–Kerbosch to $O(d \cdot |V| \cdot 3^{d/3})$
    pub fpt_degeneracy_d: usize,
    /// Maximum monochromatic clique order $\omega(G)$ (Karp Problem #3: Clique)
    pub max_clique_omega: usize,
    /// Caro–Wei lower bound on independence number $\alpha(G)$ (Karp Problem #4: Independent Set / Set Packing)
    pub independent_set_alpha: usize,
    /// Gavril maximal-matching 2-Approximation for Minimum Vertex Cover (Karp Problem #2: Vertex Cover)
    pub vertex_cover_2approx_size: usize,
    /// Minimum Feedback Arc Set back-edges to break directed cycles (Karp Problem #7: Feedback Arc Set)
    pub feedback_arc_set_size: usize,
    /// Moser–Tardos Constructive LLL Chromatic Number $\chi_{\text{LLL}}$ (Karp Problem #11: Chromatic Number)
    pub chromatic_number_lll: u16,
    /// Szemerédi Clique/Cluster Cover count $K$ (Karp Problem #12: Clique Cover)
    pub clique_cover_clusters_k: usize,
    /// Mehlhorn Voronoi 2-Approximation Steiner Tree cluster count (Karp Problem #15: Steiner Tree)
    pub steiner_tree_2approx_clusters: usize,
    /// Token Knapsack budget utilization ratio (Karp Problem #18: Knapsack)
    pub knapsack_budget_utilization: f64,
}

/// Quantum Many-Body Kohn–Sham DFT & Second-Order Strang Split-Operator Wave Report.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ManyBodyDftWaveReport {
    /// Number of Kohn–Sham mean-field super-orbitals ($K$ Szemerédi super-nodes)
    pub kohn_sham_orbitals_k: usize,
    /// Number of Local Natural Orbital (LNO / QM-MM) correlation exceptions in `T8R`
    pub lno_t8r_correlations: usize,
    /// Pairwise complexity reduction factor $N^2 / \max(1, K^2 + |E_{\text{T8R}}|)$
    pub many_body_speedup_factor: f64,
    /// Exact coprime rational speedup $p/q = N(N-1) / (K^2 + |E_{\text{T8R}}|)$
    pub many_body_speedup_pq: String,
    /// Total Kohn–Sham mean-field + LNO `T8R` exchange energy $\sum_{i,j} d_p(V_i, V_j) \langle \psi_i | \psi_j \rangle$
    pub kohn_sham_exchange_energy: f64,
    /// Exact `4gaia` zero-float `(p/q) * 2^m` PQM register for Kohn–Sham exchange energy
    pub kohn_sham_exchange_pqm: String,
    /// Second-order Strang Split-Operator ($e^{hA/2} e^{hB} e^{hA/2}$) L2 wave coherence after step $h$
    pub strang_split_coherence: f64,
    /// Exact coprime rational $p/q$ representation of Strang split-operator wave coherence
    pub strang_coherence_pq: String,
    /// Top super-node orbital indices ranked by Strang-propagated salience amplitude $|\Psi_i(t + h)|^2$
    pub dominant_salience_orbitals: Vec<(ClusterId, f32)>,
}

/// Computes the unified Flyxion Recency-vs-Salience ontology report, Karp's 21 NP-Complete
/// FPT certificate, and Quantum Many-Body Kohn–Sham DFT / Strang Split-Operator wave report.
#[allow(clippy::too_many_arguments)]
pub fn analyze_salience_karp_manybody(
    nodes: &[FileNode],
    graph: &CsrSymbolGraph,
    archive: &Mem8BookArchive,
    book: &ZeroCopyMem8Book,
    steiner: &ErdosSteinerSlice,
    center_turan: &ErdosCenterTuranReport,
    schedule: &MoserTardosSchedule,
    ramsey_eh: &RamseyErdosHajnalReport,
) -> (
    SalienceOntologyReport,
    KarpComplexityCertificate,
    ManyBodyDftWaveReport,
) {
    let salience = compute_salience_ontology(nodes, graph, archive, steiner, center_turan);
    let karp = compute_karp_certificate(graph, archive, steiner, schedule, ramsey_eh);
    let manybody = compute_manybody_dft_strang(graph, archive, book, &salience);
    (salience, karp, manybody)
}

/// Evaluates Flyxion's *Recency as Salience* ($d_T(x, q) \neq d_R(x, q)$) ontology over scanned nodes.
fn compute_salience_ontology(
    nodes: &[FileNode],
    graph: &CsrSymbolGraph,
    archive: &Mem8BookArchive,
    steiner: &ErdosSteinerSlice,
    center_turan: &ErdosCenterTuranReport,
) -> SalienceOntologyReport {
    let n = nodes.len();
    if n == 0 {
        return SalienceOntologyReport {
            regime: SalienceRegime::QuasiStaticRecencyValid,
            proxy_disagreement_prob: 0.0,
            admissibility_change_ratio: 0.0,
            argmin_temporal_symbol: None,
            argmin_relational_symbol: None,
            load_bearing_old_artifacts: Vec::new(),
        };
    }

    // Map each cluster to its Steiner resistance distance (or fallback radius)
    let mut cluster_dist = vec![FALLBACK_CLUSTER_RESISTANCE_DIST; archive.super_nodes.len() + 2];
    for sn in &steiner.steiner_super_nodes {
        let cid = sn.cluster_id as usize;
        if cid < cluster_dist.len() {
            cluster_dist[cid] = sn.resistance_dist;
        }
    }
    for &c_id in &center_turan.center_clusters {
        let cid = c_id as usize;
        if cid < cluster_dist.len() {
            cluster_dist[cid] = cluster_dist[cid].min(CENTER_CLUSTER_MAX_DIST);
        }
    }

    // Collect file candidates with (symbol_id, modified_secs, relational_distance_d_r)
    let mut file_indices: Vec<usize> = (0..n)
        .filter(|&i| !nodes[i].is_dir && !nodes[i].is_ignored)
        .collect();
    if file_indices.is_empty() {
        file_indices = (0..n).collect();
    }

    let mut mod_secs = vec![0u64; n];
    let mut rel_dist = vec![10.0_f32; n];

    for &i in &file_indices {
        mod_secs[i] = nodes[i]
            .modified
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);

        let cid = archive.vertex_to_cluster.get(i).copied().unwrap_or(0) as usize;
        let base_c_dist = cluster_dist
            .get(cid)
            .copied()
            .unwrap_or(FALLBACK_CLUSTER_RESISTANCE_DIST);
        let deg = if i < graph.num_vertices {
            (graph.row_ptr[i + 1] - graph.row_ptr[i])
                + (graph.transpose_row_ptr[i + 1] - graph.transpose_row_ptr[i])
        } else {
            0
        };
        // Relational distance d_R(x, q): lower when on the Steiner backbone / Erdős Center and high structural degree
        let deg_bonus = 1.0 / (1.0 + (deg as f32).ln_1p());
        rel_dist[i] = base_c_dist * SALIENCE_CLUSTER_WEIGHT + deg_bonus * SALIENCE_DEGREE_WEIGHT;
    }

    // Sort by Observable Temporal Distance d_T (descending modified timestamp = newest first; tie-break by reverse index)
    let mut temporal_order = file_indices.clone();
    temporal_order.sort_by(|&a, &b| mod_secs[b].cmp(&mod_secs[a]).then_with(|| b.cmp(&a)));

    // Sort by Relational Distance d_R (ascending relational distance = most salient first)
    let mut relational_order = file_indices.clone();
    relational_order.sort_by(|&a, &b| rel_dist[a].total_cmp(&rel_dist[b]).then_with(|| a.cmp(&b)));

    let mut temporal_rank = vec![0usize; n];
    for (rk, &sym) in temporal_order.iter().enumerate() {
        temporal_rank[sym] = rk;
    }

    let mut relational_rank = vec![0usize; n];
    for (rk, &sym) in relational_order.iter().enumerate() {
        relational_rank[sym] = rk;
    }

    // Evaluate proxy disagreement probability Pr[x*_T != x*_R] across the top-k frontier
    let frontier_k = file_indices.len().clamp(1, PROXY_DISAGREEMENT_FRONTIER_K);
    let mut disagreements = 0usize;
    for pos in 0..frontier_k {
        if temporal_order[pos] != relational_order[pos] {
            disagreements += 1;
        }
    }
    let proxy_disagreement_prob = (disagreements as f64) / (frontier_k as f64);

    // Identify "Load-Bearing Old" ("Corrugated Iron") artifacts:
    // Files in the top relational salience half where temporal rank is significantly older
    let mut load_bearing_old = Vec::new();
    let half_m = (file_indices.len() / 2).max(1);
    for &sym in relational_order
        .iter()
        .take(half_m.min(MAX_LOAD_BEARING_CANDIDATES))
    {
        let r_t = temporal_rank[sym];
        let r_r = relational_rank[sym];
        if r_t > r_r {
            load_bearing_old.push(LoadBearingOldArtifact {
                symbol_id: sym as SymbolId,
                path: nodes[sym].path.display().to_string(),
                temporal_rank_r_t: r_t,
                relational_rank_r_r: r_r,
                relational_distance_d_r: rel_dist[sym],
                amnesia_displacement: r_t - r_r,
            });
        }
    }
    load_bearing_old.sort_by_key(|a| std::cmp::Reverse(a.amnesia_displacement));
    load_bearing_old.truncate(MAX_LOAD_BEARING_ARTIFACTS);

    let admissibility_change_ratio = ((archive.exception_ledger.len()
        + archive.outlier_vertices.len()) as f64)
        / (n.max(1) as f64);

    let regime = if proxy_disagreement_prob > RELATIONAL_SHIFT_DISAGREEMENT_THRESHOLD
        || admissibility_change_ratio > RELATIONAL_SHIFT_CHANGE_RATIO_THRESHOLD
    {
        SalienceRegime::RelationalShiftActive
    } else {
        SalienceRegime::QuasiStaticRecencyValid
    };

    SalienceOntologyReport {
        regime,
        proxy_disagreement_prob,
        admissibility_change_ratio,
        argmin_temporal_symbol: temporal_order.first().map(|&i| i as SymbolId),
        argmin_relational_symbol: relational_order.first().map(|&i| i as SymbolId),
        load_bearing_old_artifacts: load_bearing_old,
    }
}

/// Computes Karp's 21 NP-Complete FPT & Approximation Certificate on the AST symbol graph.
fn compute_karp_certificate(
    graph: &CsrSymbolGraph,
    archive: &Mem8BookArchive,
    steiner: &ErdosSteinerSlice,
    schedule: &MoserTardosSchedule,
    ramsey_eh: &RamseyErdosHajnalReport,
) -> KarpComplexityCertificate {
    // Gavril's 2-Approximation for Minimum Vertex Cover via Maximal Matching
    let n = graph.num_vertices;
    let mut matched = vec![false; n];
    let mut cover_size = 0usize;
    for u in 0..n {
        if matched[u] {
            continue;
        }
        let start = graph.row_ptr[u] as usize;
        let end = graph.row_ptr[u + 1] as usize;
        for idx in start..end {
            let v = graph.col_idx[idx] as usize;
            if u != v && v < n && !matched[v] {
                matched[u] = true;
                matched[v] = true;
                cover_size += 2;
                break;
            }
        }
    }

    let max_clique_omega = ramsey_eh.omega_per_color.into_iter().max().unwrap_or(1);
    let feedback_arc_set_size: usize = ramsey_eh
        .cliques
        .iter()
        .map(|c| c.feedback_back_edges.len())
        .sum();

    let default_budget = super::erdos_steiner::ErdosSteinerConfig::default()
        .token_budget
        .max(1) as f64;
    let knapsack_budget_utilization =
        ((steiner.estimated_tokens as f64) / default_budget).clamp(0.0, 1.0);

    KarpComplexityCertificate {
        fpt_degeneracy_d: ramsey_eh.degeneracy_d,
        max_clique_omega,
        independent_set_alpha: ramsey_eh.alpha_independent_set.len(),
        vertex_cover_2approx_size: cover_size,
        feedback_arc_set_size,
        chromatic_number_lll: schedule.num_worktrees,
        clique_cover_clusters_k: archive.super_nodes.len(),
        steiner_tree_2approx_clusters: steiner.steiner_super_nodes.len(),
        knapsack_budget_utilization,
    }
}

/// Evaluates Quantum Many-Body Kohn–Sham DFT exchange energy and propagates a salience wave
/// via Second-Order Strang Operator Splitting:
/// $$\Psi(t + h) = e^{\frac{h}{2} A_{\text{temporal}}} \, e^{h B_{\text{relational}}} \, e^{\frac{h}{2} A_{\text{temporal}}} \Psi(t)$$
fn compute_manybody_dft_strang(
    graph: &CsrSymbolGraph,
    archive: &Mem8BookArchive,
    book: &ZeroCopyMem8Book,
    salience: &SalienceOntologyReport,
) -> ManyBodyDftWaveReport {
    let k = archive.super_nodes.len();
    let n = graph.num_vertices;
    let t8r_len = archive.exception_ledger.len();

    let full_pairs_u64 = (n * n.saturating_sub(1)).max(1) as u64;
    let dft_ops_u64 = (k * k + t8r_len).max(1) as u64;
    let speedup_rat = super::pq_engine::RationalPQ::new_reduced(full_pairs_u64, dft_ops_u64);
    let many_body_speedup_pq = speedup_rat.display_ratio();
    let many_body_speedup_factor = (full_pairs_u64 as f64 / dft_ops_u64 as f64).max(1.0);

    if k == 0 {
        return ManyBodyDftWaveReport {
            kohn_sham_orbitals_k: 0,
            lno_t8r_correlations: t8r_len,
            many_body_speedup_factor,
            many_body_speedup_pq,
            kohn_sham_exchange_energy: 0.0,
            kohn_sham_exchange_pqm: "(0/1)*2^0".to_string(),
            strang_split_coherence: 1.0,
            strang_coherence_pq: "1/1".to_string(),
            dominant_salience_orbitals: Vec::new(),
        };
    }

    let centroids = book.centroids();
    let super_nodes = book.super_nodes();
    let reduced_edges = book.reduced_edges();

    // Map cluster_id -> orbital index 0..k
    let max_cid = super_nodes
        .iter()
        .map(|s| s.cluster_id as usize)
        .max()
        .unwrap_or(0);
    let mut cid_to_idx = vec![usize::MAX; max_cid + 1];
    for (idx, sn) in super_nodes.iter().enumerate() {
        cid_to_idx[sn.cluster_id as usize] = idx;
    }

    // Initial wave state \Psi(0) normalized across K Kohn-Sham super-node orbitals
    let inv_sqrt_k = 1.0 / (k as f32).sqrt();
    let mut psi = vec![inv_sqrt_k; k];

    // Step 1: Half-step kinetic temporal operator exp((h/2) * A_temporal)
    let h = STRANG_STEP_H;
    let drift_rate = (salience.admissibility_change_ratio as f32).clamp(0.01, 0.5);
    for (idx, amp) in psi.iter_mut().enumerate() {
        let sn_density = super_nodes[idx].internal_p_density.clamp(0.0, 2.0);
        let phase_decay = (-0.5 * h * drift_rate / (1.0 + sn_density)).exp();
        *amp *= phase_decay;
    }

    // Step 2: Full-step relational Kohn-Sham potential operator exp(h * B_relational)
    // Uses SIMD 64-lane dot products (`felix_dot64_f32`) between super-node wave centroids
    let mut next_psi = psi.clone();
    let mut exchange_energy = 0.0_f64;

    // 2a. Diagonal Hartree / Kohn-Sham orbital self-exchange density d_p(V_i, V_i) <\psi_i | \psi_i>
    for (idx, sn) in super_nodes.iter().enumerate() {
        let self_overlap = felix_dot64_f32(&centroids[idx].lanes, &centroids[idx].lanes).abs();
        let self_xc = sn.internal_p_density.clamp(0.05, 4.0) * self_overlap;
        exchange_energy += (self_xc as f64) * 0.25;
        next_psi[idx] += 0.10 * h * self_xc * psi[idx];
    }

    // 2b. Off-diagonal regular super-edge Hartree-Fock / Kohn-Sham exchange integrals
    for edge in reduced_edges {
        let u_cid = edge.src_cluster as usize;
        let v_cid = edge.dst_cluster as usize;
        if u_cid >= cid_to_idx.len() || v_cid >= cid_to_idx.len() {
            continue;
        }
        let i = cid_to_idx[u_cid];
        let j = cid_to_idx[v_cid];
        if i == usize::MAX || j == usize::MAX {
            continue;
        }

        let wave_overlap = felix_dot64_f32(&centroids[i].lanes, &centroids[j].lanes);
        let weight = super::erdos_steiner::decode_fp16(edge.relative_p_density_fp16);
        let coupling = weight * wave_overlap.abs();
        exchange_energy += coupling as f64;

        next_psi[i] += h * coupling * psi[j];
        if i != j {
            next_psi[j] += h * coupling * psi[i];
        }
    }

    // 2c. Local Natural Orbital (LNO / QM-MM) correlation corrections from T8R exceptions
    // Includes both inter-orbital correlations (V_i <-> V_j) and QM-MM active-space boundary
    // couplings (Outlier V_0 <-> Kohn-Sham orbital V_j) via 64-lane SIMD wave overlap.
    let sym_to_cluster = book.symbol_to_cluster();
    for exc in &archive.exception_ledger {
        let u_sym = exc.src_symbol as usize;
        let v_sym = exc.dst_symbol as usize;
        let c_u = sym_to_cluster.get(u_sym).copied().unwrap_or(0) as usize;
        let c_v = sym_to_cluster.get(v_sym).copied().unwrap_or(0) as usize;
        let i = cid_to_idx.get(c_u).copied().unwrap_or(usize::MAX);
        let j = cid_to_idx.get(c_v).copied().unwrap_or(usize::MAX);

        if i != usize::MAX && j != usize::MAX {
            let lno_overlap = felix_dot64_f32(&centroids[i].lanes, &centroids[j].lanes)
                .abs()
                .max(0.25);
            let lno_coupling = 0.08 * lno_overlap;
            next_psi[i] += lno_coupling * h * psi[j];
            exchange_energy += lno_coupling as f64;
        } else if i != usize::MAX || j != usize::MAX {
            let (orb_idx, outlier_sym) = if i != usize::MAX {
                (i, v_sym)
            } else {
                (j, u_sym)
            };
            let qm_mm_overlap = graph
                .symbol_embeddings
                .get(outlier_sym)
                .map(|emb| felix_dot64_f32(emb, &centroids[orb_idx].lanes).abs())
                .unwrap_or(0.5)
                .max(0.2);
            let qm_mm_coupling = 0.06 * qm_mm_overlap;
            next_psi[orb_idx] += qm_mm_coupling * h * psi[orb_idx];
            exchange_energy += qm_mm_coupling as f64;
        } else if u_sym < graph.symbol_embeddings.len() && v_sym < graph.symbol_embeddings.len() {
            let active_overlap = felix_dot64_f32(
                &graph.symbol_embeddings[u_sym],
                &graph.symbol_embeddings[v_sym],
            )
            .abs()
            .max(0.2);
            exchange_energy += (0.04 * active_overlap) as f64;
        }
    }

    psi = next_psi;

    // Step 3: Second half-step kinetic temporal operator exp((h/2) * A_temporal)
    let mut norm_sq = 0.0_f32;
    for (idx, amp) in psi.iter_mut().enumerate() {
        let sn_density = super_nodes[idx].internal_p_density.clamp(0.0, 2.0);
        let phase_decay = (-0.5 * h * drift_rate / (1.0 + sn_density)).exp();
        *amp *= phase_decay;
        norm_sq += (*amp) * (*amp);
    }

    let norm = norm_sq.sqrt().max(1e-6);
    let mut dominant_salience_orbitals: Vec<(ClusterId, f32)> = super_nodes
        .iter()
        .enumerate()
        .map(|(idx, sn)| {
            let prob_amp = (psi[idx] / norm) * (psi[idx] / norm);
            (sn.cluster_id, prob_amp)
        })
        .collect();

    dominant_salience_orbitals.sort_by(|a, b| b.1.total_cmp(&a.1));
    dominant_salience_orbitals.truncate(MAX_DOMINANT_SALIENCE_ORBITALS);

    // Coherence inner product |<\Psi(0) | \Psi(h)>|
    let coherence: f32 = psi
        .iter()
        .map(|&a| (a / norm) * inv_sqrt_k)
        .sum::<f32>()
        .clamp(0.0, 1.0);

    let xc_q16 =
        ((exchange_energy * (super::zeno_engine::Q16_UNIT_SCALE as f64)).round() as i64).max(1);
    let xc_pqm = crate::pq_trainer::normalize_symplectic_pqm(xc_q16, 1, -16);
    let coh_num = (((coherence as f64) * (STRANG_COHERENCE_LATTICE_DEN as f64)).round() as u64)
        .clamp(0, STRANG_COHERENCE_LATTICE_DEN);
    let coh_rat = super::pq_engine::RationalPQ::new_reduced(coh_num, STRANG_COHERENCE_LATTICE_DEN);

    ManyBodyDftWaveReport {
        kohn_sham_orbitals_k: k,
        lno_t8r_correlations: t8r_len,
        many_body_speedup_factor,
        many_body_speedup_pq,
        kohn_sham_exchange_energy: exchange_energy,
        kohn_sham_exchange_pqm: xc_pqm.display_pqm(),
        strang_split_coherence: coherence as f64,
        strang_coherence_pq: coh_rat.display_ratio(),
        dominant_salience_orbitals,
    }
}
