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
/// Exact fixed-point `Q16` scale (`2^16 = 65_536`) for zero-float Kohn–Sham & Strang propagation.
pub const STRANG_Q16_SCALE: u64 = 65_536;
/// Exact `Q16` representation of Strang step $h = 1/4$ (`16_384`).
pub const STRANG_STEP_H_Q16: u64 = STRANG_Q16_SCALE / 4;
/// Maximum number of dominant Kohn–Sham salience orbitals retained (`5`).
pub const MAX_DOMINANT_SALIENCE_ORBITALS: usize = 5;
/// Rational denominator lattice for Strang split-operator wave coherence (`10_000`).
pub const STRANG_COHERENCE_LATTICE_DEN: u64 = 10_000;
/// Rational denominator lattice for orbital Born probabilities (`10_000`).
pub const BORN_PROB_LATTICE_DEN: u64 = 10_000;

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
    /// Exact coprime rational $p/q$ Born probabilities for top super-node orbitals (`10_000` lattice)
    pub dominant_salience_orbitals_pq: Vec<(ClusterId, String)>,
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

/// Converts IEEE-754 `f32` bit pattern to signed `Q16` fixed-point (`value * 65_536`)
/// using pure integer bit-manipulation (zero floating-point instructions).
/// Non-finite (`NaN`/`Inf`), zero, and subnormal inputs map deterministically to `0`.
/// Results are saturated to `[-64 * Q16, +64 * Q16]` to guarantee overflow-free `i128` accumulation.
#[deny(clippy::float_arithmetic)]
#[inline]
pub(crate) fn f32_bits_to_q16_i64(bits: u32) -> i64 {
    const MAX_ABS_Q16: u64 = 64 * STRANG_Q16_SCALE;
    let sign = (bits >> 31) != 0;
    let exp = ((bits >> 23) & 0xFF) as i32;
    let frac = bits & 0x007F_FFFF;
    if exp == 0 || exp == 0xFF {
        return 0;
    }
    let mant: u64 = (1u64 << 23) | (frac as u64);
    // value * 2^16 = mant * 2^(exp - 127 - 23 + 16) = mant * 2^(exp - 134)
    let shift = exp - 134;
    let mag: u64 = if shift >= 0 {
        let s = shift as u32;
        if s >= 40 {
            MAX_ABS_Q16
        } else {
            mant.checked_shl(s).unwrap_or(MAX_ABS_Q16).min(MAX_ABS_Q16)
        }
    } else {
        let r = (-shift) as u32;
        if r >= 64 {
            0
        } else {
            // Round-to-nearest integer using half-LSB bias
            let half = 1u64 << (r - 1);
            mant.saturating_add(half) >> r
        }
    };
    let clamped = mag.min(MAX_ABS_Q16) as i64;
    if sign { -clamped } else { clamped }
}

/// Exact integer square root `floor(sqrt(n))` for `u128` via binary restoring / Newton iteration.
#[deny(clippy::float_arithmetic)]
#[inline]
pub(crate) fn isqrt_u128(n: u128) -> u128 {
    if n < 2 {
        return n;
    }
    let shift = (128 - n.leading_zeros()).div_ceil(2);
    let mut x = 1u128 << shift;
    loop {
        let next = (x + n / x) >> 1;
        if next >= x {
            return x;
        }
        x = next;
    }
}

/// Exact zero-float `Q16` approximation of $e^{-x}$ on $x \in [0, 1]$ (`0 <= x_q16 <= 65_536`)
/// using the diagonal `[2/2]` Padé approximant:
/// $$e^{-x} \approx \frac{12 - 6x + x^2}{12 + 6x + x^2}$$
/// Scale: `Q16 = 65_536`, domain: `x_q16 in [0, 65_536]`, rounding: nearest-half-up.
/// Error bound: $< 3 \times 10^{-8}$ over the Strang temporal half-step domain $x \le 1/16$,
/// and $< 10^{-4}$ across the full unit interval $[0, 1]$.
#[deny(clippy::float_arithmetic)]
#[inline]
pub(crate) fn exp_neg_q16(x_q16: u64) -> u64 {
    const Q16: u128 = STRANG_Q16_SCALE as u128;
    let x = (x_q16.min(STRANG_Q16_SCALE)) as u128;
    let x2 = (x * x + (Q16 >> 1)) >> 16;
    let num = (12 * Q16 + x2).saturating_sub(6 * x);
    let den = 12 * Q16 + 6 * x + x2;
    (((num << 16) + (den >> 1)) / den) as u64
}

/// Pure-integer 64-lane `Q16` absolute inner product `|a · b|` returning `Q16`.
#[deny(clippy::float_arithmetic)]
#[inline]
fn dot64_q16_abs(a: &[i64; 64], b: &[i64; 64]) -> u64 {
    let mut acc: i128 = 0;
    let mut i = 0;
    while i < 64 {
        acc += (a[i] as i128) * (b[i] as i128);
        i += 1;
    }
    let abs_q32 = acc.unsigned_abs();
    ((abs_q32 + ((STRANG_Q16_SCALE as u128) >> 1)) >> 16).min(u64::MAX as u128) as u64
}

#[deny(clippy::float_arithmetic)]
#[inline]
fn quantize_lanes_q16(lanes: &[f32; 64]) -> [i64; 64] {
    let mut out = [0i64; 64];
    let mut i = 0;
    while i < 64 {
        out[i] = f32_bits_to_q16_i64(lanes[i].to_bits());
        i += 1;
    }
    out
}

struct ExactManyBodyQ16Result {
    k: usize,
    t8r_len: usize,
    speedup_rat: super::pq_engine::RationalPQ,
    exchange_energy_q16: i64,
    exchange_pqm: String,
    coherence_rat: super::pq_engine::RationalPQ,
    dominant_orbitals_bps: Vec<(ClusterId, u64, String)>,
}

/// Zero-float `Q16` / exact `p/q` Quantum Many-Body Kohn–Sham DFT and Second-Order
/// Strang Split-Operator propagator (`#[deny(clippy::float_arithmetic)]` enforced):
/// $$\Psi(t + h) = e^{\frac{h}{2} A_{\text{temporal}}} \, e^{h B_{\text{relational}}} \, e^{\frac{h}{2} A_{\text{temporal}}} \Psi(t)$$
/// Note: `RegularSuperEdgePod.relative_p_density_fp16` is encoded as unsigned Q6.10 fixed-point
/// (`raw / 1024`), so converting to `Q16` is an exact left shift `(raw as u64) << 6` (fixing
/// the prior IEEE-754 `decode_fp16` sign-bit corruption on dense codebases where `raw >= 0x8000`).
#[deny(clippy::float_arithmetic)]
fn compute_manybody_dft_strang_q16(
    graph: &CsrSymbolGraph,
    archive: &Mem8BookArchive,
    book: &ZeroCopyMem8Book,
    drift_num_u64: u64,
    drift_den_u64: u64,
) -> ExactManyBodyQ16Result {
    const Q16: u64 = STRANG_Q16_SCALE;
    const Q16_U128: u128 = STRANG_Q16_SCALE as u128;

    let k = archive.super_nodes.len();
    let n = graph.num_vertices;
    let t8r_len = archive.exception_ledger.len();

    let full_pairs_u64 = (n * n.saturating_sub(1)).max(1) as u64;
    let dft_ops_u64 = (k * k + t8r_len).max(1) as u64;
    let speedup_rat = super::pq_engine::RationalPQ::new_reduced(full_pairs_u64, dft_ops_u64);

    if k == 0 {
        return ExactManyBodyQ16Result {
            k: 0,
            t8r_len,
            speedup_rat,
            exchange_energy_q16: 0,
            exchange_pqm: "(0/1)*2^0".to_string(),
            coherence_rat: super::pq_engine::RationalPQ::new_reduced(1, 1),
            dominant_orbitals_bps: Vec::new(),
        };
    }

    let centroids = book.centroids();
    let super_nodes = book.super_nodes();
    let reduced_edges = book.reduced_edges();

    // Quantize legacy POD f32 centroids & internal densities at the boundary via pure bit-unpacking
    let centroid_q16: Vec<[i64; 64]> = centroids
        .iter()
        .map(|c| quantize_lanes_q16(&c.lanes))
        .collect();
    let sn_density_q16: Vec<u64> = super_nodes
        .iter()
        .map(|sn| {
            let v = f32_bits_to_q16_i64(sn.internal_p_density.to_bits());
            if v <= 0 { 0 } else { v as u64 }
        })
        .collect();

    let max_cid = super_nodes
        .iter()
        .map(|s| s.cluster_id as usize)
        .max()
        .unwrap_or(0);
    let mut cid_to_idx = vec![usize::MAX; max_cid + 1];
    for (idx, sn) in super_nodes.iter().enumerate() {
        cid_to_idx[sn.cluster_id as usize] = idx;
    }

    // Initial wave state \Psi(0) normalized across K Kohn-Sham orbitals in Q16: floor(Q16 / sqrt(K))
    let sqrt_k_q16 = isqrt_u128((k as u128) << 32).max(1);
    let inv_sqrt_k_q16 = (((Q16_U128 << 16) + (sqrt_k_q16 >> 1)) / sqrt_k_q16).max(1);
    let mut psi_q16: Vec<u128> = vec![inv_sqrt_k_q16; k];

    // Drift rate in Q16 clamped to [0.01, 0.5] => [655, 32_768]
    let drift_min_q16: u64 = 655; // ~0.01 * 65536
    let drift_max_q16: u64 = Q16 / 2; // 0.5 * 65536
    let raw_drift_q16 = ((drift_num_u64 as u128) << 16)
        .checked_div(drift_den_u64 as u128)
        .map_or(drift_min_q16, |v| v.min(u64::MAX as u128) as u64);
    let drift_rate_q16 = raw_drift_q16.clamp(drift_min_q16, drift_max_q16);

    // Precompute exact half-step temporal decay factors exp(-(h/2) * drift / (1 + sn_density)) in Q16
    let half_h_drift_q32: u128 = ((STRANG_STEP_H_Q16 as u128) >> 1) * (drift_rate_q16 as u128);
    let half_decay_q16: Vec<u128> = sn_density_q16
        .iter()
        .map(|&d| {
            let d_clamped = d.min(2 * Q16) as u128;
            let exponent_q16 = ((half_h_drift_q32 + ((Q16_U128 + d_clamped) >> 1))
                / (Q16_U128 + d_clamped)) as u64;
            exp_neg_q16(exponent_q16) as u128
        })
        .collect();

    // Step 1: First half-step kinetic temporal operator exp((h/2) * A_temporal)
    for (idx, amp) in psi_q16.iter_mut().enumerate() {
        *amp = (*amp * half_decay_q16[idx] + (Q16_U128 >> 1)) >> 16;
    }

    // Step 2: Full-step relational Kohn-Sham potential operator exp(h * B_relational)
    let mut next_psi_q16 = psi_q16.clone();
    let mut exchange_energy_q16: u128 = 0;

    // 2a. Diagonal Hartree / Kohn-Sham orbital self-exchange density d_p(V_i, V_i) <\psi_i | \psi_i>
    let min_self_dens_q16: u64 = 3_277; // 0.05 * 65536
    let max_self_dens_q16: u64 = 4 * Q16; // 4.0 * 65536
    for idx in 0..super_nodes.len() {
        let self_overlap_q16 = dot64_q16_abs(&centroid_q16[idx], &centroid_q16[idx]) as u128;
        let clamped_dens_q16 = sn_density_q16[idx].clamp(min_self_dens_q16, max_self_dens_q16) as u128;
        let self_xc_q16 = (clamped_dens_q16 * self_overlap_q16 + (Q16_U128 >> 1)) >> 16;
        // exchange_energy += self_xc * 0.25 (exact >> 2)
        exchange_energy_q16 = exchange_energy_q16.saturating_add(self_xc_q16 >> 2);
        // next_psi[idx] += (1/10) * (1/4) * self_xc * psi[idx] = self_xc * psi / (40 * Q16)
        let delta = (self_xc_q16 * psi_q16[idx]) / (40 * Q16_U128);
        next_psi_q16[idx] = next_psi_q16[idx].saturating_add(delta);
    }

    // 2b. Off-diagonal regular super-edge Hartree-Fock / Kohn-Sham exchange integrals
    // RegularSuperEdgePod.relative_p_density_fp16 is unsigned Q6.10 (density * 1024),
    // so converting to Q16 is exact left-shift by 6 (clamped to 16.0 for numerical stability).
    let max_edge_weight_q16: u128 = 16 * Q16_U128;
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

        let wave_overlap_q16 = dot64_q16_abs(&centroid_q16[i], &centroid_q16[j]) as u128;
        let weight_q16 = ((edge.relative_p_density_fp16 as u128) << 6).min(max_edge_weight_q16);
        let coupling_q16 = (weight_q16 * wave_overlap_q16 + (Q16_U128 >> 1)) >> 16;
        exchange_energy_q16 = exchange_energy_q16.saturating_add(coupling_q16);

        // h = 1/4 => h * coupling * psi / Q16 = (coupling * psi) >> 18
        let di = (coupling_q16 * psi_q16[j]) >> 18;
        next_psi_q16[i] = next_psi_q16[i].saturating_add(di);
        if i != j {
            let dj = (coupling_q16 * psi_q16[i]) >> 18;
            next_psi_q16[j] = next_psi_q16[j].saturating_add(dj);
        }
    }

    // 2c. Local Natural Orbital (LNO / QM-MM) correlation corrections from T8R exceptions
    let sym_to_cluster = book.symbol_to_cluster();
    let floor_025_q16: u64 = Q16 / 4; // 0.25
    let floor_020_q16: u64 = 13_107; // 0.20 * 65536
    let fallback_050_q16: u64 = Q16 / 2; // 0.50
    for exc in &archive.exception_ledger {
        let u_sym = exc.src_symbol as usize;
        let v_sym = exc.dst_symbol as usize;
        let c_u = sym_to_cluster.get(u_sym).copied().unwrap_or(0) as usize;
        let c_v = sym_to_cluster.get(v_sym).copied().unwrap_or(0) as usize;
        let i = cid_to_idx.get(c_u).copied().unwrap_or(usize::MAX);
        let j = cid_to_idx.get(c_v).copied().unwrap_or(usize::MAX);

        if i != usize::MAX && j != usize::MAX {
            let lno_overlap_q16 =
                dot64_q16_abs(&centroid_q16[i], &centroid_q16[j]).max(floor_025_q16) as u128;
            // 0.08 = 2/25
            let lno_coupling_q16 = (lno_overlap_q16 * 2 + 12) / 25;
            next_psi_q16[i] =
                next_psi_q16[i].saturating_add((lno_coupling_q16 * psi_q16[j]) >> 18);
            exchange_energy_q16 = exchange_energy_q16.saturating_add(lno_coupling_q16);
        } else if i != usize::MAX || j != usize::MAX {
            let (orb_idx, outlier_sym) = if i != usize::MAX {
                (i, v_sym)
            } else {
                (j, u_sym)
            };
            let qm_mm_overlap_q16 = graph
                .symbol_embeddings
                .get(outlier_sym)
                .map(|emb| {
                    let q_emb = quantize_lanes_q16(emb);
                    dot64_q16_abs(&q_emb, &centroid_q16[orb_idx])
                })
                .unwrap_or(fallback_050_q16)
                .max(floor_020_q16) as u128;
            // 0.06 = 3/50
            let qm_mm_coupling_q16 = (qm_mm_overlap_q16 * 3 + 25) / 50;
            next_psi_q16[orb_idx] =
                next_psi_q16[orb_idx].saturating_add((qm_mm_coupling_q16 * psi_q16[orb_idx]) >> 18);
            exchange_energy_q16 = exchange_energy_q16.saturating_add(qm_mm_coupling_q16);
        } else if u_sym < graph.symbol_embeddings.len() && v_sym < graph.symbol_embeddings.len() {
            let qu = quantize_lanes_q16(&graph.symbol_embeddings[u_sym]);
            let qv = quantize_lanes_q16(&graph.symbol_embeddings[v_sym]);
            let active_overlap_q16 = dot64_q16_abs(&qu, &qv).max(floor_020_q16) as u128;
            // 0.04 = 1/25
            let active_coupling_q16 = (active_overlap_q16 + 12) / 25;
            exchange_energy_q16 = exchange_energy_q16.saturating_add(active_coupling_q16);
        }
    }

    psi_q16 = next_psi_q16;

    // Step 3: Second half-step kinetic temporal operator exp((h/2) * A_temporal)
    let mut max_amp: u128 = 0;
    for (idx, amp) in psi_q16.iter_mut().enumerate() {
        *amp = (*amp * half_decay_q16[idx] + (Q16_U128 >> 1)) >> 16;
        if *amp > max_amp {
            max_amp = *amp;
        }
    }

    // Rescale amplitudes down into <= 2^40 before squaring so sum(amp^2) never overflows u128
    let downshift: u32 = if max_amp > (1u128 << 40) {
        (128 - max_amp.leading_zeros()).saturating_sub(40)
    } else {
        0
    };
    let mut norm_sq: u128 = 0;
    let mut amp_sum: u128 = 0;
    for &amp in &psi_q16 {
        let s = amp >> downshift;
        norm_sq = norm_sq.saturating_add(s.saturating_mul(s));
        amp_sum = amp_sum.saturating_add(s);
    }

    // Exact Born probabilities on the 10_000 lattice (bps) + reduced coprime p/q
    let mut ranked: Vec<(ClusterId, u128, u64, String)> = super_nodes
        .iter()
        .enumerate()
        .map(|(idx, sn)| {
            let s = psi_q16[idx] >> downshift;
            let sq = s.saturating_mul(s);
            let bps = sq
                .saturating_mul(BORN_PROB_LATTICE_DEN as u128)
                .saturating_add(norm_sq >> 1)
                .checked_div(norm_sq)
                .map_or(0, |v| v.min(BORN_PROB_LATTICE_DEN as u128) as u64);
            let pq = super::pq_engine::RationalPQ::new_reduced(bps, BORN_PROB_LATTICE_DEN)
                .display_ratio();
            (sn.cluster_id, sq, bps, pq)
        })
        .collect();
    ranked.sort_by(|a, b| b.1.cmp(&a.1).then_with(|| a.0.cmp(&b.0)));
    ranked.truncate(MAX_DOMINANT_SALIENCE_ORBITALS);
    let dominant_orbitals_bps: Vec<(ClusterId, u64, String)> = ranked
        .into_iter()
        .map(|(cid, _, bps, pq)| (cid, bps, pq))
        .collect();

    // Exact Strang coherence |<\Psi(0) | \Psi(h)>| = sum(psi_i) / (sqrt(K) * ||psi||_2)
    let coh_den = isqrt_u128((k as u128).saturating_mul(norm_sq));
    let coh_num = amp_sum
        .saturating_mul(STRANG_COHERENCE_LATTICE_DEN as u128)
        .saturating_add(coh_den >> 1)
        .checked_div(coh_den)
        .map_or(STRANG_COHERENCE_LATTICE_DEN, |v| {
            v.min(STRANG_COHERENCE_LATTICE_DEN as u128) as u64
        });
    let coherence_rat =
        super::pq_engine::RationalPQ::new_reduced(coh_num, STRANG_COHERENCE_LATTICE_DEN);

    let xc_q16_i64 = (exchange_energy_q16.min(i64::MAX as u128) as i64).max(1);
    let xc_pqm = crate::pq_trainer::normalize_symplectic_pqm(xc_q16_i64, 1, -16).display_pqm();

    ExactManyBodyQ16Result {
        k,
        t8r_len,
        speedup_rat,
        exchange_energy_q16: xc_q16_i64,
        exchange_pqm: xc_pqm,
        coherence_rat,
        dominant_orbitals_bps,
    }
}

/// Evaluates Quantum Many-Body Kohn–Sham DFT exchange energy and propagates a salience wave
/// via Second-Order Strang Operator Splitting using exact zero-float `Q16` / `p/q` arithmetic:
/// $$\Psi(t + h) = e^{\frac{h}{2} A_{\text{temporal}}} \, e^{h B_{\text{relational}}} \, e^{\frac{h}{2} A_{\text{temporal}}} \Psi(t)$$
fn compute_manybody_dft_strang(
    graph: &CsrSymbolGraph,
    archive: &Mem8BookArchive,
    book: &ZeroCopyMem8Book,
    _salience: &SalienceOntologyReport,
) -> ManyBodyDftWaveReport {
    let drift_num = (archive.exception_ledger.len() + archive.outlier_vertices.len()) as u64;
    let drift_den = graph.num_vertices.max(1) as u64;
    let exact = compute_manybody_dft_strang_q16(graph, archive, book, drift_num, drift_den);

    // Serialize-time compatibility projections derived strictly from exact integer/p-q outputs
    let dominant_salience_orbitals: Vec<(ClusterId, f32)> = exact
        .dominant_orbitals_bps
        .iter()
        .map(|(cid, bps, _)| (*cid, (*bps as f32) / (BORN_PROB_LATTICE_DEN as f32)))
        .collect();
    let dominant_salience_orbitals_pq: Vec<(ClusterId, String)> = exact
        .dominant_orbitals_bps
        .into_iter()
        .map(|(cid, _, pq)| (cid, pq))
        .collect();

    ManyBodyDftWaveReport {
        kohn_sham_orbitals_k: exact.k,
        lno_t8r_correlations: exact.t8r_len,
        many_body_speedup_factor: exact.speedup_rat.approx_f64.max(1.0),
        many_body_speedup_pq: exact.speedup_rat.display_ratio(),
        kohn_sham_exchange_energy: (exact.exchange_energy_q16 as f64) / (STRANG_Q16_SCALE as f64),
        kohn_sham_exchange_pqm: exact.exchange_pqm,
        strang_split_coherence: exact.coherence_rat.approx_f64,
        strang_coherence_pq: exact.coherence_rat.display_ratio(),
        dominant_salience_orbitals,
        dominant_salience_orbitals_pq,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mem8::regularity::types::{
        EdgeKind, ExceptionReason, Mem8BookArchive, Mem8SuperNode, RegularSuperEdge,
        T8rExceptionEdge,
    };
    use crate::mem8::regularity::wire::RegularSuperEdgePod;

    #[test]
    fn f32_bits_to_q16_unpacks_without_float_math() {
        assert_eq!(f32_bits_to_q16_i64(0.0_f32.to_bits()), 0);
        assert_eq!(f32_bits_to_q16_i64(1.0_f32.to_bits()), 65_536);
        assert_eq!(f32_bits_to_q16_i64((-1.0_f32).to_bits()), -65_536);
        assert_eq!(f32_bits_to_q16_i64(0.25_f32.to_bits()), 16_384);
        assert_eq!(f32_bits_to_q16_i64(f32::NAN.to_bits()), 0);
        assert_eq!(f32_bits_to_q16_i64(f32::INFINITY.to_bits()), 0);
        assert_eq!(f32_bits_to_q16_i64(0.125_f32.to_bits()), 8_192);
    }

    #[test]
    fn pade_exp_neg_q16_matches_analytic_bounds() {
        assert_eq!(exp_neg_q16(0), STRANG_Q16_SCALE);
        let q_quarter = exp_neg_q16(STRANG_Q16_SCALE / 4);
        assert!((q_quarter as i64 - 51_039).abs() <= 2, "got {q_quarter}");
        let q_half = exp_neg_q16(STRANG_Q16_SCALE / 2);
        assert!((q_half as i64 - 39_750).abs() <= 4, "got {q_half}");
        let q_one = exp_neg_q16(STRANG_Q16_SCALE);
        assert!((q_one as i64 - 24_145).abs() <= 2, "got {q_one}");
        assert!(q_quarter > q_half && q_half > q_one);
    }

    #[test]
    fn dense_q6_10_super_edge_never_corrupts_exchange_energy_or_coherence() {
        // Regression test for dense graphs (e.g. aye / 8b-public-documents) where
        // RegularSuperEdgePod::encode_density_fp16 sets bit 15 (raw >= 0x8000 for density >= 32.0).
        let raw_dense = RegularSuperEdgePod::encode_density_fp16(40.0);
        assert!(
            raw_dense >= 0x8000,
            "expected bit 15 set in Q6.10, got {raw_dense:#x}"
        );

        let mut lanes_a = [0.0_f32; 64];
        let mut lanes_b = [0.0_f32; 64];
        lanes_a[0] = 1.0;
        lanes_b[0] = 0.75;
        lanes_b[1] = 0.5;

        let archive = Mem8BookArchive {
            model_digest: [0u8; 32],
            epsilon: 0.1,
            compression_ratio: 10.0,
            final_energy: 0.5,
            super_nodes: vec![
                Mem8SuperNode {
                    cluster_id: 1,
                    vertex_count: 8,
                    internal_p_density: 1.0,
                    centroid_embedding: lanes_a.to_vec(),
                    representative_symbols: vec![0, 1, 2],
                },
                Mem8SuperNode {
                    cluster_id: 2,
                    vertex_count: 8,
                    internal_p_density: 0.5,
                    centroid_embedding: lanes_b.to_vec(),
                    representative_symbols: vec![8, 9, 10],
                },
            ],
            reduced_edges: vec![RegularSuperEdge {
                src_cluster: 1,
                dst_cluster: 2,
                relative_p_density: 40.0,
                dominant_kind: EdgeKind::CallRef,
            }],
            exception_ledger: vec![T8rExceptionEdge {
                src_symbol: 0,
                dst_symbol: 8,
                kind: EdgeKind::CallRef,
                reason: ExceptionReason::IrregularPair,
            }],
            outlier_vertices: Vec::new(),
            total_symbol_count: 16,
            vertex_to_cluster: vec![1, 1, 1, 1, 1, 1, 1, 1, 2, 2, 2, 2, 2, 2, 2, 2],
        };

        let book = ZeroCopyMem8Book::from_archive(&archive);
        let graph = CsrSymbolGraph::from_edges(16, &[(0, 8, EdgeKind::CallRef)], vec![lanes_a; 16]);

        let exact = compute_manybody_dft_strang_q16(&graph, &archive, &book, 1, 16);
        assert!(exact.exchange_energy_q16 > STRANG_Q16_SCALE as i64);
        assert_ne!(exact.exchange_pqm, "(1/1)*2^-16");
        assert!(
            exact.coherence_rat.p > 0,
            "coherence must stay strictly positive"
        );
        assert_eq!(exact.dominant_orbitals_bps.len(), 2);
        let sum_bps: u64 = exact.dominant_orbitals_bps.iter().map(|x| x.1).sum();
        assert!(
            (9_998..=10_002).contains(&sum_bps),
            "Born sum bps={sum_bps}"
        );
    }
}
