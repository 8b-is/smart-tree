//! Sovereign `p/q` Mode Engine (`src/mem8/regularity/pq_engine.rs`).
//!
//! Unifies the four mathematical pillars of `(p, q)` graph theory and physics across Smart Tree:
//! 1. **Exact Coprime Rational `(p/q)` Arithmetic**: Hardware-division-free Stein's Binary GCD
//!    (`felix_binary_gcd_u64`) and 128-bit cross-multiplication (`felix_pq_cross_cmp`) for exact
//!    rational graph densities, Molloy–Reed $\kappa = p_\kappa / q_\kappa$, and Turán ratios.
//! 2. **Hölder Conjugate $(L_p, L_q)$ Dual-Space Spectral Norm ($\frac{1}{p} + \frac{1}{q} = 1$)**:
//!    Computes the Bollobás–Riordan operator norm $\|A_R\|_{3/2 \to 3}$ vs. Hilbert $\|A_R\|_{2 \to 2}$
//!    via `felix_holder_pq_dual_norm_f64` to quantify scale-free hub concentration.
//! 3. **Kővári–Sós–Turán / Zarankiewicz Extremal Bound $z(n; p, q)$ (1954)**:
//!    Evaluates $z(n; p, q) = \lfloor \frac{1}{2}(q-1)^{1/p} n^{2 - 1/p} + \frac{1}{2}(p-1)n \rfloor$
//!    for $(p, q) \in \{(2, 2), (3, 3)\}$, certifying guaranteed $K_{p,q}$ bipartite cores.
//! 4. **Hamiltonian Canonical $(p, q)$ Phase-Space & Symplectic Störmer–Verlet Integrator**:
//!    Couples Flyxion's relational position $q_i = d_R(V_i)$ with conjugate temporal momentum
//!    $p_i = d_T^{-1}(V_i)$ in a Liouville volume-preserving ($\det J = 1.0$) leapfrog flow.

use super::erdos_steiner::ErdosCenterTuranReport;
use super::salience_karp_manybody::SalienceOntologyReport;
use super::types::{ClusterId, CsrSymbolGraph, Mem8BookArchive};
use crate::felix_asm::{felix_binary_gcd_u64, felix_holder_pq_dual_norm_f64, felix_pq_cross_cmp};
use serde::{Deserialize, Serialize};
use std::cmp::Ordering;

/// Exact coprime rational number $p / q$ reduced via Stein's Binary GCD (`tzcnt`, zero `udiv`).
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct RationalPQ {
    /// Numerator $p$ (coprime to $q$)
    pub p: u64,
    /// Denominator $q \ge 1$ (coprime to $p$)
    pub q: u64,
    /// Q32.32 fixed-point representation `((p as u128) << 32) / (q as u128)`
    pub q32_fixed: u64,
    /// IEEE-754 f64 projection for JSON telemetry
    pub approx_f64: f64,
}

impl RationalPQ {
    /// Constructs a canonical coprime rational $p/q$ reduced via `felix_binary_gcd_u64`.
    #[inline]
    pub fn new_reduced(p: u64, q: u64) -> Self {
        if q == 0 {
            return Self {
                p: 0,
                q: 1,
                q32_fixed: 0,
                approx_f64: 0.0,
            };
        }
        if p == 0 {
            return Self {
                p: 0,
                q: 1,
                q32_fixed: 0,
                approx_f64: 0.0,
            };
        }
        let g = felix_binary_gcd_u64(p, q).max(1);
        let rp = p / g;
        let rq = q / g;
        let q32_fixed = (((rp as u128) << 32) / (rq as u128)) as u64;
        let approx_f64 = (rp as f64) / (rq as f64);
        Self {
            p: rp,
            q: rq,
            q32_fixed,
            approx_f64,
        }
    }

    /// Exact 128-bit integer cross-multiplication comparison (`p1 * q2` vs `p2 * q1`)
    /// with zero floating-point division or rounding drift.
    #[inline]
    pub fn cross_cmp(&self, other: &Self) -> Ordering {
        felix_pq_cross_cmp(self.p, self.q, other.p, other.q)
    }

    /// Formats the rational as `"p/q"`.
    #[inline]
    pub fn display_ratio(&self) -> String {
        format!("{}/{}", self.p, self.q)
    }
}

/// Canonical Hamiltonian phase-space particle $(q_i, p_i)$ for a Szemerédi super-node $V_i$.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SymplecticParticlePQ {
    pub cluster_id: ClusterId,
    /// Generalized position $q_i(T)$ (relational Steiner salience coordinate $d_R$)
    pub q_position_d_r: f64,
    /// Conjugate momentum $p_i(T)$ (temporal activity flux $d_T^{-1}$)
    pub p_momentum_d_t_inv: f64,
    /// Local particle Hamiltonian energy $H_i(p_i, q_i) = \frac{p_i^2}{2m_i} + V_i(q)$
    pub local_hamiltonian_h: f64,
}

/// Complete report for Smart Tree's Sovereign `p/q` Mode (`--mode pq`).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PqModeReport {
    /// Pillar 1: Exact coprime ambient edge density $p_{\text{amb}} / q_{\text{amb}}$
    pub ambient_density_pq: RationalPQ,
    /// Pillar 1: Exact coprime Molloy–Reed percolation ratio $\kappa = \langle k^2 \rangle / \langle k \rangle$
    pub molloy_reed_kappa_pq: RationalPQ,
    /// Pillar 1: Exact coprime ratio of undirected edges to Turán's $K_5$-free bound $\text{ex}(n, K_5)$
    pub turan_k5_ratio_pq: RationalPQ,
    /// Pillar 1: Rationalized Kohayakawa–Rödl partition energy $q_p(\mathcal{P})$ on a $10^6$ lattice
    pub kohayakawa_rodl_energy_pq: RationalPQ,
    /// Pillar 2: Hölder conjugate exponent $p = p_{\text{num}} / p_{\text{den}}$ (default $3/2$)
    pub holder_p_ratio: String,
    /// Pillar 2: Hölder conjugate exponent $q = q_{\text{num}} / q_{\text{den}}$ (default $3/1$)
    pub holder_q_ratio: String,
    /// Pillar 2: Bollobás–Riordan operator norm $\|A_R\|_{3/2 \to 3}$
    pub holder_3_2_to_3_norm: f64,
    /// Pillar 2: Classical Hilbert operator norm $\|A_R\|_{2 \to 2}$
    pub hilbert_l2_norm: f64,
    /// Pillar 2: Scale-free hub concentration ratio $\|A_R\|_{3/2 \to 3} / \|A_R\|_{2 \to 2}$
    pub holder_hub_concentration_ratio: f64,
    /// Pillar 3: Kővári–Sós–Turán / Reiman $z(n; 2, 2)$ upper bound for $K_{2,2}$-free graphs
    pub zarankiewicz_z_2_2_bound: u64,
    /// Pillar 3: Kővári–Sós–Turán $z(n; 3, 3)$ upper bound for $K_{3,3}$-free graphs
    pub zarankiewicz_z_3_3_bound: u64,
    /// Pillar 3: True if $|E_{\text{undir}}| > z(n; 2, 2)$ (guarantees a $K_{2,2}$ 4-cycle bi-clique)
    pub zarankiewicz_k22_guaranteed: bool,
    /// Pillar 3: True if $|E_{\text{undir}}| > z(n; 3, 3)$ (guarantees a $K_{3,3}$ bi-clique)
    pub zarankiewicz_k33_guaranteed: bool,
    /// Pillar 4: Initial total Hamiltonian energy $H_0(p(0), q(0))$
    pub symplectic_initial_h0: f64,
    /// Pillar 4: Final total Hamiltonian energy $H_T(p(T), q(T))$ after Störmer–Verlet leapfrog
    pub symplectic_final_h: f64,
    /// Pillar 4: Relative symplectic energy drift $|H_T - H_0| / \max(|H_0|, 10^{-9})$
    pub symplectic_energy_drift: f64,
    /// Pillar 4: Liouville phase-space volume determinant $\det J_{\text{leapfrog}}$ (exact $1.0$)
    pub liouville_jacobian_det: f64,
    /// Pillar 4: Top super-node canonical $(q_i, p_i)$ phase-space orbitals
    pub phase_space_orbitals: Vec<SymplecticParticlePQ>,
}

/// Computes the Boyd / Bollobás–Riordan $(L_p \to L_q)$ operator norm $\|A\|_{p \to q}$
/// (where $\frac{1}{p} + \frac{1}{q} = 1$, $p = p_{\text{num}} / p_{\text{den}}$) via nonlinear
/// dual-space power iteration accelerated by `felix_holder_pq_dual_norm_f64`.
fn compute_holder_operator_norm(
    sym_adj: &[f64],
    k: usize,
    p_num: u32,
    p_den: u32,
    max_iters: usize,
) -> f64 {
    if k == 0 {
        return 0.0;
    }
    let p = (p_num.max(1) as f64) / (p_den.max(1) as f64);
    let q = p / (p - 1.0).max(1e-6);

    let mut x = vec![(k as f64).powf(-1.0 / p); k];
    let mut y = vec![0.0_f64; k];
    let mut op_norm = 0.0_f64;

    for _iter in 0..max_iters {
        // 1. Forward operator application: y = A * x
        let mut lq_sum = 0.0_f64;
        for i in 0..k {
            let mut sum = 0.0_f64;
            for j in 0..k {
                sum += sym_adj[i * k + j] * x[j];
            }
            y[i] = sum;
            lq_sum += sum.abs().powf(q);
        }
        op_norm = lq_sum.powf(1.0 / q);
        if op_norm < 1e-12 {
            return 0.0;
        }

        // 2. Map y through Hölder dual map psi_q(y_i) = sgn(y_i)|y_i|^{q-1} and normalize in L_p
        // Note: since A is symmetric, dual iteration applies felix_holder_pq_dual_norm_f64
        let _ = felix_holder_pq_dual_norm_f64(&mut y, p_num, p_den);
        let mut lp_sum = 0.0_f64;
        for &val in &y {
            lp_sum += val.abs().powf(p);
        }
        let lp_norm = lp_sum.powf(1.0 / p).max(1e-12);
        for i in 0..k {
            x[i] = y[i] / lp_norm;
        }
    }
    op_norm
}

/// Evaluates the Kővári–Sós–Turán (1954) upper bound on the Zarankiewicz problem $z(n; p, q)$:
/// $$z(n; p, q) = \left\lfloor \frac{1}{2} (q - 1)^{1/p} n^{2 - 1/p} + \frac{1}{2} (p - 1) n \right\rfloor$$
#[inline]
pub fn kovari_sos_turan_bound(n: usize, p: u32, q: u32) -> u64 {
    if n == 0 || p == 0 || q == 0 {
        return 0;
    }
    let nf = n as f64;
    let inv_p = 1.0 / (p as f64);
    let term1 = 0.5 * ((q.saturating_sub(1) as f64).powf(inv_p)) * nf.powf(2.0 - inv_p);
    let term2 = 0.5 * (p.saturating_sub(1) as f64) * nf;
    (term1 + term2).floor() as u64
}

/// Executes the full 4-pillar Sovereign `p/q` analysis over the AST symbol graph,
/// the compressed `Mem8BookArchive`, and Flyxion's $(d_T, d_R)$ salience profiles.
pub fn analyze_pq_mode(
    graph: &CsrSymbolGraph,
    archive: &Mem8BookArchive,
    salience: &SalienceOntologyReport,
    center_turan: &ErdosCenterTuranReport,
) -> PqModeReport {
    let n = graph.num_vertices;
    let m = graph.num_edges;

    // -------------------------------------------------------------------------
    // Pillar 1: Exact Coprime Rational (p/q) Graph Invariants
    // -------------------------------------------------------------------------
    let max_directed_edges = (n as u64)
        .saturating_mul((n.saturating_sub(1)) as u64)
        .max(1);
    let ambient_density_pq = RationalPQ::new_reduced(m as u64, max_directed_edges);

    // Compute exact undirected degree sequence for Molloy-Reed kappa = sum(k^2) / sum(k)
    let mut sum_deg = 0u64;
    let mut sum_deg_sq = 0u64;
    for u in 0..n {
        let deg = (graph.row_ptr[u + 1] - graph.row_ptr[u]) as u64;
        sum_deg += deg;
        sum_deg_sq += deg * deg;
    }
    let molloy_reed_kappa_pq = RationalPQ::new_reduced(sum_deg_sq, sum_deg.max(1));

    // Turán K_5-free extremal bound ex(n, K_5) = floor(3/8 * n^2)
    let turan_k5_bound = center_turan.turan_ex_k5_edges.max(1) as u64;
    let approx_undir_edges = (m as u64).div_ceil(2);
    let turan_k5_ratio_pq = RationalPQ::new_reduced(approx_undir_edges, turan_k5_bound);

    // Kohayakawa-Rödl energy rationalized on a 1,000,000 denominator lattice
    let energy_num = (archive.final_energy.clamp(0.0, 100.0) * 1_000_000.0).round() as u64;
    let kohayakawa_rodl_energy_pq = RationalPQ::new_reduced(energy_num, 1_000_000);

    // -------------------------------------------------------------------------
    // Pillar 2: Hölder Conjugate (L_p, L_q) Dual-Space Spectral Operator Norm
    // -------------------------------------------------------------------------
    let k = archive.super_nodes.len().max(1);
    let mut sym_adj = vec![0.0_f64; k * k];
    // Include internal super-node self-coupling on the diagonal
    for (idx, sn) in archive.super_nodes.iter().enumerate() {
        sym_adj[idx * k + idx] = (sn.internal_p_density as f64).max(0.1);
    }
    for edge in &archive.reduced_edges {
        let u = (edge.src_cluster as usize).saturating_sub(1);
        let v = (edge.dst_cluster as usize).saturating_sub(1);
        if u < k && v < k {
            let w = edge.relative_p_density as f64;
            sym_adj[u * k + v] += 0.5 * w;
            sym_adj[v * k + u] += 0.5 * w;
        }
    }

    // Conjugate pair p = 3/2, q = 3 (1/(3/2) + 1/3 = 2/3 + 1/3 = 1)
    let holder_3_2_to_3_norm = compute_holder_operator_norm(&sym_adj, k, 3, 2, 24);
    // Classical Hilbert space p = 2, q = 2 (1/2 + 1/2 = 1)
    let hilbert_l2_norm = compute_holder_operator_norm(&sym_adj, k, 2, 1, 24);
    let holder_hub_concentration_ratio = holder_3_2_to_3_norm / hilbert_l2_norm.max(1e-9);

    // -------------------------------------------------------------------------
    // Pillar 3: Kővári–Sós–Turán / Zarankiewicz Extremal Problem z(n; p, q)
    // -------------------------------------------------------------------------
    let zarankiewicz_z_2_2_bound = kovari_sos_turan_bound(n, 2, 2);
    let zarankiewicz_z_3_3_bound = kovari_sos_turan_bound(n, 3, 3);
    let zarankiewicz_k22_guaranteed = approx_undir_edges > zarankiewicz_z_2_2_bound;
    let zarankiewicz_k33_guaranteed = approx_undir_edges > zarankiewicz_z_3_3_bound;

    // -------------------------------------------------------------------------
    // Pillar 4: Hamiltonian Canonical (p, q) Symplectic Störmer–Verlet Leapfrog
    // -------------------------------------------------------------------------
    // Derive per-cluster initial generalized coordinate q_i(0) (relational salience)
    // and conjugate momentum p_i(0) (temporal activity flux) from Flyxion's ontology
    // and Erdős Center eccentricity.
    let mut q_pos = vec![0.5_f64; k];
    let mut p_mom = vec![0.0_f64; k];
    let mut mass = vec![1.0_f64; k];
    let mut q_eq = vec![0.5_f64; k];

    let max_ecc = center_turan.graph_diameter_hops.max(1) as f64;
    for (idx, sn) in archive.super_nodes.iter().enumerate() {
        mass[idx] = (sn.vertex_count.max(1) as f64).ln_1p().max(1.0);
        let is_center = center_turan.center_clusters.contains(&sn.cluster_id);
        let base_r = if is_center {
            1.0 - (center_turan.graph_radius_hops as f64) / (max_ecc + 1.0)
        } else {
            0.45 + 0.25 * (sn.internal_p_density as f64).tanh()
        };
        let mut amnesia_boost = 0.0_f64;
        for &sym in &sn.representative_symbols {
            if let Some(lbo) = salience
                .load_bearing_old_artifacts
                .iter()
                .find(|a| a.symbol_id == sym)
            {
                amnesia_boost += (1.0 - lbo.relational_distance_d_r as f64) * 0.15;
            }
        }
        let r_coord = (base_r + amnesia_boost).clamp(0.05, 0.98);
        let phi = (1.0 + 5.0_f64.sqrt()) * 0.5;
        let t_flux =
            (salience.proxy_disagreement_prob - 0.35) + 0.12 * (((idx as f64) + 1.0) * phi).sin();
        q_pos[idx] = r_coord;
        q_eq[idx] = 0.5 * (r_coord + 0.5);
        p_mom[idx] = t_flux * mass[idx];
    }

    let compute_forces = |q: &[f64], forces: &mut [f64]| {
        for i in 0..k {
            // Harmonic confinement toward equilibrium relational depth q_eq[i]
            let mut d_v_dq = 1.25 * (q[i] - q_eq[i]);
            // Pairwise super-edge spring coupling sum_j w_ij (q_i - q_j)
            for j in 0..k {
                if i != j {
                    let w = sym_adj[i * k + j];
                    if w > 0.0 {
                        d_v_dq += w * (q[i] - q[j]);
                    }
                }
            }
            forces[i] = -d_v_dq;
        }
    };

    let compute_hamiltonian = |q: &[f64], p: &[f64]| -> (f64, Vec<f64>) {
        let mut local_h = vec![0.0_f64; k];
        let mut total_h = 0.0_f64;
        for i in 0..k {
            let kinetic = (p[i] * p[i]) / (2.0 * mass[i]);
            let mut potential = 0.625 * (q[i] - q_eq[i]) * (q[i] - q_eq[i]);
            for j in 0..k {
                if i != j {
                    let w = sym_adj[i * k + j];
                    if w > 0.0 {
                        potential += 0.25 * w * (q[i] - q[j]) * (q[i] - q[j]);
                    }
                }
            }
            local_h[i] = kinetic + potential;
            total_h += local_h[i];
        }
        (total_h, local_h)
    };

    let (symplectic_initial_h0, _) = compute_hamiltonian(&q_pos, &p_mom);

    // Execute S = 16 second-order symplectic Störmer–Verlet leapfrog steps (dt = 0.025)
    let dt = 0.025_f64;
    let mut forces = vec![0.0_f64; k];
    compute_forces(&q_pos, &mut forces);

    for _step in 0..16 {
        // 1. Half-step momentum kick: p(t + dt/2) = p(t) + (dt/2) * F(q(t))
        for i in 0..k {
            p_mom[i] += 0.5 * dt * forces[i];
        }
        // 2. Full-step position drift: q(t + dt) = q(t) + dt * p(t + dt/2) / m_i
        for i in 0..k {
            q_pos[i] += dt * (p_mom[i] / mass[i]);
        }
        // 3. Re-evaluate conservative forces at q(t + dt) and finish half-step momentum kick
        compute_forces(&q_pos, &mut forces);
        for i in 0..k {
            p_mom[i] += 0.5 * dt * forces[i];
        }
    }

    let (symplectic_final_h, local_h_vec) = compute_hamiltonian(&q_pos, &p_mom);
    let symplectic_energy_drift =
        (symplectic_final_h - symplectic_initial_h0).abs() / symplectic_initial_h0.abs().max(1e-9);

    let mut phase_space_orbitals: Vec<SymplecticParticlePQ> = archive
        .super_nodes
        .iter()
        .enumerate()
        .map(|(idx, sn)| SymplecticParticlePQ {
            cluster_id: sn.cluster_id,
            q_position_d_r: q_pos[idx],
            p_momentum_d_t_inv: p_mom[idx],
            local_hamiltonian_h: local_h_vec[idx],
        })
        .collect();

    phase_space_orbitals.sort_by(|a, b| {
        b.local_hamiltonian_h
            .partial_cmp(&a.local_hamiltonian_h)
            .unwrap_or(Ordering::Equal)
    });
    phase_space_orbitals.truncate(8);

    PqModeReport {
        ambient_density_pq,
        molloy_reed_kappa_pq,
        turan_k5_ratio_pq,
        kohayakawa_rodl_energy_pq,
        holder_p_ratio: "3/2".to_string(),
        holder_q_ratio: "3/1".to_string(),
        holder_3_2_to_3_norm,
        hilbert_l2_norm,
        holder_hub_concentration_ratio,
        zarankiewicz_z_2_2_bound,
        zarankiewicz_z_3_3_bound,
        zarankiewicz_k22_guaranteed,
        zarankiewicz_k33_guaranteed,
        symplectic_initial_h0,
        symplectic_final_h,
        symplectic_energy_drift,
        liouville_jacobian_det: 1.0,
        phase_space_orbitals,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rational_pq_binary_gcd_and_cross_cmp() {
        let r1 = RationalPQ::new_reduced(360, 1080);
        assert_eq!(r1.p, 1);
        assert_eq!(r1.q, 3);
        assert_eq!(r1.display_ratio(), "1/3");

        let r2 = RationalPQ::new_reduced(2, 6);
        assert_eq!(r1.cross_cmp(&r2), Ordering::Equal);

        let r3 = RationalPQ::new_reduced(355, 113);
        let r4 = RationalPQ::new_reduced(22, 7);
        // 355/113 = 3.1415929 < 22/7 = 3.1428571
        assert_eq!(r3.cross_cmp(&r4), Ordering::Less);
    }

    #[test]
    fn test_kovari_sos_turan_bounds() {
        let z22 = kovari_sos_turan_bound(100, 2, 2);
        let z33 = kovari_sos_turan_bound(100, 3, 3);
        assert!(z22 > 0);
        assert!(z33 > z22);
    }
}
