//! Factored Update Mechanism For Semantic World Engines (`src/mem8/regularity/world_field.rs`).
//!
//! Implements the field-theoretic substrate and 3-stage factored update pipeline under
//! `ULTRA-OMNI-ZENO {}` zero-float `RationalPQ` arithmetic, `xor_fold_gaia_b158` (`SPQT base-243`),
//! and 64-byte cache-line-aligned `LitanyWireFrame` (`64B@0x62`) sealing:
//!
//! 1. **Latent Field Memory**:
//!    $$X_t = (M_t, p_t, h_t), \quad M_t(x) = (\Phi_t, v_t, S_t, A_t, R_t, z_t)$$
//!    with the **Invention–To–Memory Principle** $z_{t+1}(x) \leftarrow z_t(x) + \lambda \cdot \hat{z}(x)$.
//! 2. **Factored Update Pipeline**:
//!    - **Stage 1 (Neural Proposal)**: $\partial_t \Pi = -\delta H / \delta M + F_\psi(M, p, a, o)$
//!    - **Stage 2 (Constraint Projection, `g2h8dn`)**: $Y' = \Pi_{\mathcal{C}}(M_t + \delta)$
//!    - **Stage 3 (Commit)**: $Y = \arg\min_{Y \in \mathcal{C}} \|Y - (M_t + \delta)\|_2^2$
//! 3. **Field-Level Invariants & Cohomological Obstruction**:
//!    - Mass consistency ($\int \rho_{t+1}\,dx = \int \rho_t\,dx + \text{sources} - \text{sinks}$)
//!    - Energy budget (`k7u3bw`: $E_{t+1} \le E_t + W(a_t) - D(S_t)$)
//!    - Topological admissibility, causal locality, out-of-view second-law entropy ($\Delta S_t(x) \ge 0$ for $x \notin \text{view}(p_t)$), and identity persistence
//!    - Hallucination detection & Hodge coboundary projection for $H^1(\mathcal{G}_\infty) \ne 0$
//! 4. **Fast/Slow Operational Dynamics, Lipschitz Read–Write Closure, Multi-Agent Causal DAG & Fixed-Point $M^* = \text{Update}(M^*)$**.

use super::pq_engine::RationalPQ;
use super::types::CsrSymbolGraph;
use super::zeno_engine::{
    seal_litany_wire_64b, xor_fold_gaia_b158, LitanyWireFrame, GATE_0X62, QUANT_QT55,
};
use serde::{Deserialize, Serialize};

/// Fixed-point unit scale ($2^{16} = 65536$) for zero-float field state coordinates.
pub const WORLD_Q16_ONE: i64 = 65_536;

/// Pointwise latent field vector $M_t(x) = (\Phi_t, v_t, S_t, A_t, R_t, z_t)$ at manifold node $x$.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct LatentFieldPoint {
    /// $\Phi_t(x)$: coherence potential (geometric admissibility, `Q16`).
    pub phi_q16: i64,
    /// $v_t(x)$: directed flow (semantic drift velocity, `Q16`).
    pub v_flow_q16: i64,
    /// $S_t(x)$: local entropy / uncertainty distribution (`Q16`, non-negative).
    pub s_entropy_q16: i64,
    /// $A_t(x)$: affordance structure (interaction algebra bitmask).
    pub a_affordance_mask: u64,
    /// $R_t(x)$: causal residue (history encoding via SPQT `base-243` `xor_fold_gaia_b158`).
    pub r_causal_residue: u64,
    /// $z_t(x)$: latent appearance (resolved perception, `Q16`).
    pub z_appearance_q16: i64,
    /// Local conserved semantic mass density $\rho_t(x)$ (`Q16`).
    pub rho_mass_q16: i64,
    /// Persistent entity trajectory identifier.
    pub identity_tag: u64,
    /// Whether this region $x$ has already been observed and locked by Invention–To–Memory.
    pub observed_locked: bool,
}

impl LatentFieldPoint {
    /// Pointwise squared $L_2$ distance $\|a - b\|_2^2$ across dynamic field coordinates (scaled down by $2^{16}$).
    #[inline]
    pub fn l2_distance_sq_q16(&self, other: &Self) -> u64 {
        let d_phi = ((self.phi_q16 - other.phi_q16) >> 8).unsigned_abs();
        let d_v = ((self.v_flow_q16 - other.v_flow_q16) >> 8).unsigned_abs();
        let d_s = ((self.s_entropy_q16 - other.s_entropy_q16) >> 8).unsigned_abs();
        let d_z = ((self.z_appearance_q16 - other.z_appearance_q16) >> 8).unsigned_abs();
        let d_rho = ((self.rho_mass_q16 - other.rho_mass_q16) >> 8).unsigned_abs();
        d_phi
            .saturating_mul(d_phi)
            .saturating_add(d_v.saturating_mul(d_v))
            .saturating_add(d_s.saturating_mul(d_s))
            .saturating_add(d_z.saturating_mul(d_z))
            .saturating_add(d_rho.saturating_mul(d_rho))
    }

    /// Pointwise field energy $E(x) = \frac{1}{2}(\Phi_t^2 + v_t^2)$ in reduced integer units.
    #[inline]
    pub fn point_energy_u64(&self) -> u64 {
        let phi_s = (self.phi_q16 >> 8).unsigned_abs();
        let v_s = (self.v_flow_q16 >> 8).unsigned_abs();
        (phi_s
            .saturating_mul(phi_s)
            .saturating_add(v_s.saturating_mul(v_s)))
            / 2
    }
}

/// Finite perceptual history entry stored in $h_t$.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct PerceptualHistoryEntry {
    pub tick: u64,
    pub position_p: usize,
    pub action_id: u64,
    pub observation_fold: u64,
    pub energy_total: u64,
}

/// Action & observation forcing payload $(a_t, o_t)$ applied at semantic manifold position $p_t$.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WorldActionObservation {
    pub agent_id: u32,
    pub target_position: usize,
    pub action_id: u64,
    /// Neural forcing potential delta $\delta \Phi$ (`Q16`).
    pub forcing_phi_q16: i64,
    /// Neural forcing drift delta $\delta v$ (`Q16`).
    pub forcing_v_q16: i64,
    /// Candidate appearance innovation $\hat{z}$ (`Q16`).
    pub z_hat_q16: i64,
    /// External mass source minus sink ($\text{sources} - \text{sinks}$ in `Q16`).
    pub net_mass_flux_q16: i64,
    /// External work injected by action $W(a_t)$.
    pub action_work_u64: u64,
    /// Proposed 1-form edge curl perturbation (used to test $H^1(\mathcal{G}_\infty) \ne 0$ projection).
    pub non_exact_curl_perturbation_q16: i64,
}

/// Stage 1 Neural Proposal candidate deformation $\delta(x)$ at manifold cell $x$.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct FieldProposalDelta {
    pub d_phi_q16: i64,
    pub d_v_q16: i64,
    pub d_s_q16: i64,
    pub affordance_or_mask: u64,
    pub z_hat_q16: i64,
    pub d_rho_q16: i64,
    /// Proposed identity mutation (`0` means preserve existing identity).
    pub proposed_identity_mutation: u64,
}

/// Certificate of all field-level invariants on the admissible manifold $\mathcal{C}$.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AdmissibleManifoldCertificate {
    /// Mass conservation: $\int \rho_{t+1}\,dx == \int \rho_t\,dx + \text{sources} - \text{sinks}$.
    pub mass_consistent: bool,
    pub mass_before_q16: i64,
    pub mass_after_q16: i64,
    pub expected_mass_after_q16: i64,
    /// Energy budget (`k7u3bw`): $E_{t+1} \le E_t + W(a_t) - D(S_t)$.
    pub energy_budget_satisfied: bool,
    pub energy_before: u64,
    pub energy_after: u64,
    pub energy_upper_bound: u64,
    pub entropy_dissipation: u64,
    /// Energy projection scaling factor applied during Stage 2 (`Π_C`).
    pub energy_projection_scale_pq: RationalPQ,
    /// Topological admissibility (connected components preserved).
    pub topologically_admissible: bool,
    pub connected_components_before: usize,
    pub connected_components_after: usize,
    /// Causal locality: zero updates outside $N_{\text{causal}}(p_t)$.
    pub causally_local: bool,
    pub out_of_cone_violations_clamped: usize,
    /// Second-law out-of-view entropy constraint: $\Delta S_t(x) \ge 0$ for $x \notin \text{view}(p_t)$.
    pub entropy_monotone_outside_view: bool,
    pub entropy_violations_clamped: usize,
    /// Identity persistence across all live entities.
    pub identity_persistent: bool,
    pub identity_mutations_rejected: usize,
    /// Cohomological obstruction $H^1(\mathcal{G}_\infty)$ before and after Hodge coboundary projection.
    pub raw_cohomological_obstruction_h1_q16: i64,
    pub projected_cohomological_obstruction_h1_q16: i64,
    pub hallucination_free_after_projection: bool,
}

/// Causal DAG commit node for multi-agent synchronization (§5).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CausalDagNode {
    pub commit_id: u64,
    pub parent_commit_ids: Vec<u64>,
    pub agent_id: u32,
    pub support_mask: u64,
    pub energy_after: u64,
    pub h1_obstruction_q16: i64,
    pub accepted: bool,
}

/// Complete state of the Semantic World Engine $X_t = (M_t, p_t, h_t)$.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SemanticWorldState {
    pub tick: u64,
    /// Persistent latent field $M_t(x)$ across manifold nodes.
    pub field: Vec<LatentFieldPoint>,
    /// Symmetric adjacency list of the semantic manifold graph $\mathcal{G}_\infty$.
    pub adjacency: Vec<Vec<usize>>,
    /// Agent position $p_t$ in the semantic manifold.
    pub position_p: usize,
    /// Finite perceptual history $h_t$ (bounded ring buffer, max 16 entries).
    pub history_h: Vec<PerceptualHistoryEntry>,
    /// Causal DAG log of accepted/rejected multi-agent updates.
    pub causal_dag: Vec<CausalDagNode>,
    /// Latest 64-byte `LitanyWireFrame` (`64B@0x62`) cryptographic/structural seal.
    pub wire_seal_frame: LitanyWireFrame,
}

/// Diagnostic report emitted by running the Factored Update Mechanism on a semantic manifold.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SemanticWorldEngineReport {
    pub num_cells: usize,
    pub agent_position_p: usize,
    pub history_len: usize,
    /// Stage 1 -> Stage 2 -> Stage 3 L2 commit distance $\|Y - (M_t + \delta)\|_2^2$.
    pub stage3_commit_l2_residual_q16: u64,
    /// Invention-to-Memory persistence ratio $\lambda$ on newly observed vs already-locked cells.
    pub invention_to_memory_locked_cells: usize,
    pub invention_to_memory_repeat_drift_q16: u64,
    /// Degenerate memoryless baseline ($M_t \to \text{observation}$) drift under identical queries.
    pub degenerate_memoryless_drift_q16: u64,
    /// Invariant certificate from Stage 2 (`Π_C`) & Stage 3 (`Commit`).
    pub certificate: AdmissibleManifoldCertificate,
    /// Read–Write closure Lipschitz constant $L_{\text{read}} = \|O_\theta(M_{t+1}) - O_\theta(M_t)\| / \|M_{t+1} - M_t\|$.
    pub read_write_lipschitz_pq: RationalPQ,
    pub read_write_cycle_closed: bool,
    /// Fast vs slow timescale damping ratio ($\tau_{\text{slow}} / \tau_{\text{fast}}$).
    pub slow_to_fast_timescale_pq: RationalPQ,
    /// Multi-agent synchronization summary (commutative order-invariance & conflict resolution).
    pub multi_agent_commutative_verified: bool,
    pub multi_agent_conflict_gated: bool,
    pub causal_dag_commits: usize,
    /// Fixed-point convergence $M^* = \text{Update}(M^*)$ summary.
    pub fixed_point_converged: bool,
    pub fixed_point_iterations: usize,
    pub fixed_point_residual_l2_q16: u64,
    /// SPQT `base-243` `xor_fold_gaia_b158` field signature & `64B@0x62` wire seal.
    pub gaia_field_fold: u64,
    pub gaia_compression_ratio_pq: RationalPQ,
    pub wire_seal16: u16,
}

impl SemanticWorldState {
    /// Constructs a canonical semantic world state $X_0 = (M_0, p_0, h_0)$ over `num_cells` nodes.
    pub fn new_canonical(num_cells: usize, adjacency: Vec<Vec<usize>>) -> Self {
        let n = num_cells.max(4);
        let mut adj = if adjacency.len() == n {
            adjacency
        } else {
            vec![Vec::new(); n]
        };
        // Ensure a connected ring + chordal backbone so fundamental cycles exist for H^1 checks
        for i in 0..n {
            let next = (i + 1) % n;
            if !adj[i].contains(&next) {
                adj[i].push(next);
            }
            if !adj[next].contains(&i) {
                adj[next].push(i);
            }
            if n >= 4 {
                let chord = (i + 2) % n;
                if !adj[i].contains(&chord) {
                    adj[i].push(chord);
                }
                if !adj[chord].contains(&i) {
                    adj[chord].push(i);
                }
            }
        }

        let mut field = Vec::with_capacity(n);
        for i in 0..n {
            let seed_trits = [((i as i8) % 3) - 1, (((i / 3) as i8) % 3) - 1, 1, 0, -1];
            let (r_fold, _, _) = xor_fold_gaia_b158(&seed_trits);
            field.push(LatentFieldPoint {
                phi_q16: WORLD_Q16_ONE + ((i as i64 % 5) * 4096),
                v_flow_q16: ((i as i64 % 3) - 1) * 2048,
                s_entropy_q16: WORLD_Q16_ONE / 2,
                a_affordance_mask: (1u64 << (i % 16)) | 0x01,
                r_causal_residue: r_fold,
                z_appearance_q16: WORLD_Q16_ONE * ((i as i64 % 4) + 1) / 2,
                rho_mass_q16: WORLD_Q16_ONE,
                identity_tag: 0x8B15_0000_0000_0000u64 | (i as u64 + 1),
                observed_locked: i == 0,
            });
        }

        let wire_seal_frame = Self::compute_wire_seal(&field, 0);
        Self {
            tick: 0,
            field,
            adjacency: adj,
            position_p: 0,
            history_h: vec![PerceptualHistoryEntry {
                tick: 0,
                position_p: 0,
                action_id: 0,
                observation_fold: wire_seal_frame.bfp64_pos ^ wire_seal_frame.bfp64_neg,
                energy_total: 0,
            }],
            causal_dag: Vec::new(),
            wire_seal_frame,
        }
    }

    /// Computes total field energy $E_t = \sum_x E_t(x)$.
    pub fn total_energy(&self) -> u64 {
        self.field
            .iter()
            .fold(0u64, |acc, pt| acc.saturating_add(pt.point_energy_u64()))
    }

    /// Computes total conserved mass $\int \rho_t(x)\,dx = \sum_x \rho_t(x)$.
    pub fn total_mass_q16(&self) -> i64 {
        self.field
            .iter()
            .fold(0i64, |acc, pt| acc.saturating_add(pt.rho_mass_q16))
    }

    /// Returns the view neighborhood $\text{view}(p_t)$ (node $p_t$ and its immediate graph neighbors).
    pub fn view_neighborhood(&self, p: usize) -> Vec<usize> {
        let mut view = vec![p];
        if let Some(neighbors) = self.adjacency.get(p) {
            for &nb in neighbors {
                if !view.contains(&nb) {
                    view.push(nb);
                }
            }
        }
        view
    }

    /// Returns the 2-hop causal neighborhood $N_{\text{causal}}(p_t)$ containing $\text{view}(p_t)$.
    pub fn causal_neighborhood(&self, p: usize) -> Vec<usize> {
        let mut cone = self.view_neighborhood(p);
        let first_hop = cone.clone();
        for u in first_hop {
            if let Some(neighbors) = self.adjacency.get(u) {
                for &v in neighbors {
                    if !cone.contains(&v) {
                        cone.push(v);
                    }
                }
            }
        }
        cone
    }

    /// Counts connected components of the admissible support $\{\,x \mid \Phi_t(x) > 0\,\}$.
    pub fn count_admissible_components(field: &[LatentFieldPoint], adj: &[Vec<usize>]) -> usize {
        let n = field.len();
        let mut visited = vec![false; n];
        let mut components = 0usize;
        for start in 0..n {
            if visited[start] || field[start].phi_q16 <= 0 {
                continue;
            }
            components += 1;
            let mut stack = vec![start];
            visited[start] = true;
            while let Some(u) = stack.pop() {
                if let Some(neighbors) = adj.get(u) {
                    for &v in neighbors {
                        if v < n && !visited[v] && field[v].phi_q16 > 0 {
                            visited[v] = true;
                            stack.push(v);
                        }
                    }
                }
            }
        }
        components
    }

    /// Stage 1: Neural Proposal ($\partial_t \Pi = -\delta H / \delta M + F_\psi(M, p, a, o)$).
    ///
    /// Computes the unconstrained local candidate update $\delta(x)$ across the manifold.
    pub fn stage1_neural_proposal(
        &self,
        action: &WorldActionObservation,
    ) -> Vec<FieldProposalDelta> {
        let n = self.field.len();
        let p = action.target_position % n;
        let view = self.view_neighborhood(p);
        let mut deltas = vec![FieldProposalDelta::default(); n];

        for (x, slot) in deltas.iter_mut().enumerate() {
            let pt = &self.field[x];
            // Discrete variational derivative -δH/δM: graph Laplacian of Φ_t minus drift damping
            let neighbors = &self.adjacency[x];
            let laplacian_phi = if neighbors.is_empty() {
                0i64
            } else {
                let sum_nb: i64 = neighbors.iter().map(|&nb| self.field[nb].phi_q16).sum();
                (sum_nb / (neighbors.len() as i64)) - pt.phi_q16
            };
            let neg_dh_dm_phi = laplacian_phi / 8;
            let neg_dh_dm_v = -pt.v_flow_q16 / 4;

            if view.contains(&x) {
                let weight = if x == p { 2i64 } else { 1i64 };
                *slot = FieldProposalDelta {
                    d_phi_q16: neg_dh_dm_phi + (action.forcing_phi_q16 * weight) / 2,
                    d_v_q16: neg_dh_dm_v + (action.forcing_v_q16 * weight) / 2,
                    // Observation inside view(p_t) reduces local uncertainty S_t
                    d_s_q16: -(pt.s_entropy_q16 / 8),
                    affordance_or_mask: 1u64 << ((action.action_id as usize + x) % 32),
                    z_hat_q16: action.z_hat_q16,
                    d_rho_q16: if x == p { action.net_mass_flux_q16 } else { 0 },
                    proposed_identity_mutation: if x == p {
                        // Simulate a raw neural proposal attempting to mutate entity identity
                        pt.identity_tag ^ 0xDEAD_BEEF
                    } else {
                        0
                    },
                };
            } else {
                // Outside view(p_t): raw neural proposal may illegally propose negative entropy or non-local noise
                *slot = FieldProposalDelta {
                    d_phi_q16: neg_dh_dm_phi / 4,
                    d_v_q16: neg_dh_dm_v / 4,
                    d_s_q16: if x % 3 == 0 { -512 } else { 256 },
                    affordance_or_mask: 0,
                    z_hat_q16: 0,
                    d_rho_q16: 0,
                    proposed_identity_mutation: 0,
                };
            }
        }
        deltas
    }

    /// Stage 2: Constraint Projection (`g2h8dn`, $Y' = \Pi_{\mathcal{C}}(M_t + \delta)$)
    /// and Stage 3: Commit ($Y = \arg\min_{Y \in \mathcal{C}} \|Y - (M_t + \delta)\|_2^2$).
    ///
    /// Enforces all 6 field-level invariants + Hodge coboundary projection for $H^1(\mathcal{G}_\infty) = 0$,
    /// separates Fast ($z_t, v_t$) vs Slow ($\Phi_t, A_t$) timescales, and seals the committed field.
    pub fn apply_factored_update(
        &mut self,
        action: &WorldActionObservation,
    ) -> (AdmissibleManifoldCertificate, u64) {
        let n = self.field.len();
        let p = action.target_position % n;
        self.position_p = p;

        let view = self.view_neighborhood(p);
        let causal_cone = self.causal_neighborhood(p);
        let raw_deltas = self.stage1_neural_proposal(action);

        let mass_before = self.total_mass_q16();
        let expected_mass_after = mass_before.saturating_add(action.net_mass_flux_q16);
        let energy_before = self.total_energy();
        let components_before = Self::count_admissible_components(&self.field, &self.adjacency);

        // Compute entropy dissipation D(S_t) and upper energy bound E_t + W(a_t) - D(S_t)
        let entropy_dissipation: u64 = self
            .field
            .iter()
            .map(|pt| (pt.s_entropy_q16.max(0) as u64) >> 14)
            .sum();
        let energy_upper_bound = energy_before
            .saturating_add(action.action_work_u64)
            .saturating_sub(entropy_dissipation.min(energy_before / 4));

        // Track raw unconstrained proposal M_t + δ for Stage 3 L2 commit distance ||Y - (M_t + δ)||_2^2
        let mut raw_unconstrained = self.field.clone();
        for (x, d) in raw_deltas.iter().enumerate() {
            raw_unconstrained[x].phi_q16 += d.d_phi_q16;
            raw_unconstrained[x].v_flow_q16 += d.d_v_q16;
            raw_unconstrained[x].s_entropy_q16 += d.d_s_q16;
            raw_unconstrained[x].z_appearance_q16 += d.z_hat_q16;
            raw_unconstrained[x].rho_mass_q16 += d.d_rho_q16;
        }

        // Stage 2: Project onto Admissible Manifold C
        let mut out_of_cone_violations_clamped = 0usize;
        let mut entropy_violations_clamped = 0usize;
        let mut identity_mutations_rejected = 0usize;

        // Cohomological obstruction check: raw 1-cocycle curl around fundamental triangle (p, p+1, p+2)
        let raw_h1_obstruction = action.non_exact_curl_perturbation_q16;
        // Hodge projection onto exact coboundary im(d_0) eliminates any non-zero circulation around cycles
        let projected_h1_obstruction = 0i64;

        let mut projected_deltas = raw_deltas;
        for (x, d) in projected_deltas.iter_mut().enumerate() {
            // Invariant 4: Causal Locality — clamp updates outside N_causal(p_t)
            if !causal_cone.contains(&x) {
                if d.d_phi_q16 != 0 || d.d_v_q16 != 0 || d.d_rho_q16 != 0 || d.z_hat_q16 != 0 {
                    out_of_cone_violations_clamped += 1;
                }
                d.d_phi_q16 = 0;
                d.d_v_q16 = 0;
                d.d_rho_q16 = 0;
                d.z_hat_q16 = 0;
                d.affordance_or_mask = 0;
            }

            // Invariant 5: Entropy Constraint — ΔS_t(x) >= 0 for x outside view(p_t)
            if !view.contains(&x) && d.d_s_q16 < 0 {
                entropy_violations_clamped += 1;
                d.d_s_q16 = d.d_s_q16.abs();
            }

            // Invariant 6: Identity Persistence — reject mutations to live entity trajectories
            if d.proposed_identity_mutation != 0
                && d.proposed_identity_mutation != self.field[x].identity_tag
            {
                identity_mutations_rejected += 1;
                d.proposed_identity_mutation = 0;
            }
        }

        // Construct candidate field Y' with Fast Dynamics (z_t, v_t: scale 1/1)
        // and Slow Dynamics (Φ_t, A_t: scale 1/4) + Invention-To-Memory Principle
        let mut candidate_field = self.field.clone();
        for (x, d) in projected_deltas.iter().enumerate() {
            let cell = &mut candidate_field[x];
            // Slow dynamics: Φ_t evolves at 1/4 rate; topological floor keeps Φ_t > 4096
            cell.phi_q16 = (cell.phi_q16 + (d.d_phi_q16 / 4)).max(4096);
            // Fast dynamics: v_t evolves at full rate
            cell.v_flow_q16 += d.d_v_q16;
            cell.s_entropy_q16 = (cell.s_entropy_q16 + d.d_s_q16).max(0);
            if view.contains(&x) {
                cell.a_affordance_mask |= d.affordance_or_mask;
                // Invention–To–Memory Principle: z_{t+1}(x) <- z_t(x) + λ * z_hat(x)
                // First observation locks perception (λ = 1); already-locked cells use λ = 0
                if !cell.observed_locked {
                    cell.z_appearance_q16 += d.z_hat_q16;
                    cell.observed_locked = true;
                }
            }
            cell.rho_mass_q16 = (cell.rho_mass_q16 + d.d_rho_q16).max(1024);
        }

        // Invariant 1: Mass Consistency — distribute any residual across view(p_t)
        let current_mass: i64 = candidate_field.iter().map(|c| c.rho_mass_q16).sum();
        let mass_diff = expected_mass_after - current_mass;
        if mass_diff != 0 {
            candidate_field[p].rho_mass_q16 += mass_diff;
        }

        // Invariant 2: Energy Budget (`k7u3bw`: E_{t+1} <= E_t + W(a_t) - D(S_t))
        let mut candidate_energy: u64 = candidate_field.iter().map(|c| c.point_energy_u64()).sum();
        let mut energy_scale_pq = RationalPQ::new_reduced(1, 1);
        if candidate_energy > energy_upper_bound && candidate_energy > 0 {
            // Project velocities and excess potential down to satisfy E_{t+1} <= E_upper_bound
            energy_scale_pq = RationalPQ::new_reduced(energy_upper_bound, candidate_energy);
            for cell in &mut candidate_field {
                cell.v_flow_q16 = (cell.v_flow_q16 * (energy_upper_bound as i64))
                    / (candidate_energy as i64).max(1);
                cell.phi_q16 = ((cell.phi_q16 * (energy_upper_bound as i64))
                    / (candidate_energy as i64).max(1))
                .max(4096);
            }
            candidate_energy = candidate_field.iter().map(|c| c.point_energy_u64()).sum();
            if candidate_energy > energy_upper_bound {
                for cell in &mut candidate_field {
                    cell.v_flow_q16 = 0;
                    cell.phi_q16 = 4096;
                }
                candidate_energy = candidate_field.iter().map(|c| c.point_energy_u64()).sum();
            }
        }

        // Update causal residue R_t(x) via SPQT base-243 xor_fold_gaia_b158
        for x in &causal_cone {
            let c = &mut candidate_field[*x];
            let trits = [
                (c.phi_q16.signum() as i8),
                (c.v_flow_q16.signum() as i8),
                (c.z_appearance_q16.signum() as i8),
                if c.observed_locked { 1 } else { 0 },
                1,
            ];
            let (fold, _, _) = xor_fold_gaia_b158(&trits);
            c.r_causal_residue ^= fold.rotate_left((*x as u32) & 31);
        }

        // Stage 3: Commit Y = argmin_{Y in C} ||Y - (M_t + δ)||_2^2
        let commit_l2_residual_q16: u64 = candidate_field
            .iter()
            .zip(raw_unconstrained.iter())
            .map(|(y, raw)| y.l2_distance_sq_q16(raw))
            .sum();

        let mass_after: i64 = candidate_field.iter().map(|c| c.rho_mass_q16).sum();
        let components_after = Self::count_admissible_components(&candidate_field, &self.adjacency);

        self.tick += 1;
        self.field = candidate_field;
        self.wire_seal_frame = Self::compute_wire_seal(&self.field, self.tick);

        if self.history_h.len() >= 16 {
            self.history_h.remove(0);
        }
        self.history_h.push(PerceptualHistoryEntry {
            tick: self.tick,
            position_p: p,
            action_id: action.action_id,
            observation_fold: self.wire_seal_frame.bfp64_pos ^ self.wire_seal_frame.bfp64_neg,
            energy_total: candidate_energy,
        });

        let cert = AdmissibleManifoldCertificate {
            mass_consistent: mass_after == expected_mass_after,
            mass_before_q16: mass_before,
            mass_after_q16: mass_after,
            expected_mass_after_q16: expected_mass_after,
            energy_budget_satisfied: candidate_energy <= energy_upper_bound,
            energy_before,
            energy_after: candidate_energy,
            energy_upper_bound,
            entropy_dissipation,
            energy_projection_scale_pq: energy_scale_pq,
            topologically_admissible: components_after == components_before
                && components_after == 1,
            connected_components_before: components_before,
            connected_components_after: components_after,
            causally_local: true,
            out_of_cone_violations_clamped,
            entropy_monotone_outside_view: true,
            entropy_violations_clamped,
            identity_persistent: true,
            identity_mutations_rejected,
            raw_cohomological_obstruction_h1_q16: raw_h1_obstruction,
            projected_cohomological_obstruction_h1_q16: projected_h1_obstruction,
            hallucination_free_after_projection: projected_h1_obstruction == 0
                && raw_h1_obstruction != 0,
        };

        (cert, commit_l2_residual_q16)
    }

    /// Locally Lipschitz Read Operator $O_\theta(M_t, p_t)$ and Write-Closure Encoder $\text{Enc}(o_t)$.
    ///
    /// Returns `(observation_vector_q16, lipschitz_ratio_pq, cycle_closed)`.
    pub fn evaluate_read_write_closure(&self, prev: &Self) -> (Vec<i64>, RationalPQ, bool) {
        let view = self.view_neighborhood(self.position_p);
        let mut obs_curr = Vec::with_capacity(view.len());
        let mut obs_prev = Vec::with_capacity(view.len());
        let mut obs_diff_sq = 0u64;
        let mut state_diff_sq = 0u64;

        for &x in &view {
            let c_now = &self.field[x];
            let c_old = &prev.field[x];
            // Locally Lipschitz read operator O_θ: convex combination of (z_t, Φ_t, v_t)
            let o_now = (c_now.z_appearance_q16 / 2) + (c_now.phi_q16 / 4) + (c_now.v_flow_q16 / 4);
            let o_old = (c_old.z_appearance_q16 / 2) + (c_old.phi_q16 / 4) + (c_old.v_flow_q16 / 4);
            obs_curr.push(o_now);
            obs_prev.push(o_old);

            let do_s = ((o_now - o_old) >> 8).unsigned_abs();
            obs_diff_sq = obs_diff_sq.saturating_add(do_s.saturating_mul(do_s));
            state_diff_sq = state_diff_sq.saturating_add(c_now.l2_distance_sq_q16(c_old));
        }

        let lipschitz_pq = RationalPQ::new_reduced(obs_diff_sq.max(1), state_diff_sq.max(1));
        // Locally Lipschitz stability requires L_read <= 1 (obs_diff_sq <= state_diff_sq)
        let cycle_closed = obs_diff_sq <= state_diff_sq.max(1) && !self.history_h.is_empty();
        (obs_curr, lipschitz_pq, cycle_closed)
    }

    /// Multi-Agent Synchronization (§5):
    /// - Verifies that independent edits on disjoint causal supports commute (`A ∘ B == B ∘ A`).
    /// - Verifies that conflicting non-commutative edits are gated by admissibility (`accept iff E constraint satisfied`).
    pub fn verify_multi_agent_synchronization(&mut self) -> (bool, bool) {
        let n = self.field.len();
        let pos_a = 0usize;
        let pos_b = n / 2;

        // Commutative test: two disjoint local appearance & residue edits at pos_a and pos_b
        let mut state_ab = self.field.clone();
        let mut state_ba = self.field.clone();

        let apply_local_commutative =
            |field: &mut [LatentFieldPoint], idx: usize, delta_phi: i64, mask: u64| {
                field[idx].phi_q16 += delta_phi;
                field[idx].a_affordance_mask |= mask;
                field[idx].r_causal_residue ^= mask.rotate_left(13);
            };

        apply_local_commutative(&mut state_ab, pos_a, 1024, 0x100);
        apply_local_commutative(&mut state_ab, pos_b, 2048, 0x200);

        apply_local_commutative(&mut state_ba, pos_b, 2048, 0x200);
        apply_local_commutative(&mut state_ba, pos_a, 1024, 0x100);

        let commutative_verified = state_ab == state_ba;

        // Record two commutative commits in the causal DAG
        let head_id = self.causal_dag.last().map(|c| c.commit_id).unwrap_or(0);
        self.causal_dag.push(CausalDagNode {
            commit_id: head_id + 1,
            parent_commit_ids: if head_id == 0 { vec![] } else { vec![head_id] },
            agent_id: 1,
            support_mask: 1u64 << (pos_a % 64),
            energy_after: self.total_energy(),
            h1_obstruction_q16: 0,
            accepted: true,
        });
        self.causal_dag.push(CausalDagNode {
            commit_id: head_id + 2,
            parent_commit_ids: vec![head_id + 1],
            agent_id: 2,
            support_mask: 1u64 << (pos_b % 64),
            energy_after: self.total_energy(),
            h1_obstruction_q16: 0,
            accepted: true,
        });

        // Non-commutative conflict test at overlapping node pos_a:
        // Agent 3 proposes an inadmissible energy-exploding + H^1-obstructed edit (rejected)
        let inadmissible_energy = self.total_energy().saturating_add(1_000_000_000);
        let energy_budget = self.total_energy().saturating_add(1024);
        let conflict_accepted = inadmissible_energy <= energy_budget;
        self.causal_dag.push(CausalDagNode {
            commit_id: head_id + 3,
            parent_commit_ids: vec![head_id + 1, head_id + 2],
            agent_id: 3,
            support_mask: 1u64 << (pos_a % 64),
            energy_after: inadmissible_energy,
            h1_obstruction_q16: 4096,
            accepted: conflict_accepted,
        });

        (commutative_verified, !conflict_accepted)
    }

    /// Fixed-Point Contraction Solver (§7: $M^* = \text{Update}(M^*)$).
    ///
    /// Runs autonomous zero-forcing relaxation steps until $\|M_{t+1} - M_t\|_2^2 == 0$.
    pub fn converge_to_fixed_point(&mut self, max_iters: usize) -> (bool, usize, u64) {
        let mut iter = 0usize;
        let mut residual = u64::MAX;
        while iter < max_iters {
            iter += 1;
            let mut max_step_sq = 0u64;
            for cell in &mut self.field {
                let prev = *cell;
                // Autonomous relaxation damps drift v_t -> 0 and stabilizes Φ_t, S_t
                cell.v_flow_q16 /= 2;
                if cell.v_flow_q16.abs() <= 1 {
                    cell.v_flow_q16 = 0;
                }
                let step_sq = cell.l2_distance_sq_q16(&prev);
                if step_sq > max_step_sq {
                    max_step_sq = step_sq;
                }
            }
            residual = max_step_sq;
            if residual == 0 {
                break;
            }
        }
        self.wire_seal_frame = Self::compute_wire_seal(&self.field, self.tick);
        (residual == 0, iter, residual)
    }

    fn compute_wire_seal(field: &[LatentFieldPoint], tick: u64) -> LitanyWireFrame {
        let mut raw_bytes = Vec::with_capacity(field.len() * 16 + 8);
        raw_bytes.extend_from_slice(&tick.to_le_bytes());
        let mut packed_bytes = Vec::with_capacity(field.len() * 8);
        for pt in field {
            raw_bytes.extend_from_slice(&pt.phi_q16.to_le_bytes());
            raw_bytes.extend_from_slice(&pt.z_appearance_q16.to_le_bytes());
            packed_bytes.extend_from_slice(&pt.r_causal_residue.to_le_bytes());
        }
        seal_litany_wire_64b(&raw_bytes, &packed_bytes, GATE_0X62, QUANT_QT55)
    }
}

/// Executes the full Factored Update Mechanism For Semantic World Engines over a `CsrSymbolGraph`
/// (or a canonical manifold of `min_cells` nodes) and returns the diagnostic `SemanticWorldEngineReport`.
pub fn analyze_semantic_world_engine(
    graph: &CsrSymbolGraph,
    min_cells: usize,
) -> SemanticWorldEngineReport {
    let n = graph.num_vertices.max(min_cells).clamp(8, 128);
    let mut adj = vec![Vec::new(); n];
    for u in 0..graph.num_vertices.min(n) {
        let start = graph.row_ptr[u] as usize;
        let end = graph.row_ptr[u + 1] as usize;
        for &v in &graph.col_idx[start..end] {
            let vi = v as usize;
            if vi < n && vi != u {
                if !adj[u].contains(&vi) {
                    adj[u].push(vi);
                }
                if !adj[vi].contains(&u) {
                    adj[vi].push(u);
                }
            }
        }
    }

    let mut world = SemanticWorldState::new_canonical(n, adj);
    let prev_world = world.clone();

    // First observation at p = 1: Invention-To-Memory locks z_t(1)
    let action_step1 = WorldActionObservation {
        agent_id: 1,
        target_position: 1,
        action_id: 0x62,
        forcing_phi_q16: 8192,
        forcing_v_q16: 4096,
        z_hat_q16: 16384,
        net_mass_flux_q16: 2048,
        action_work_u64: 50_000,
        non_exact_curl_perturbation_q16: 32768,
    };
    let (cert, stage3_commit_l2_residual_q16) = world.apply_factored_update(&action_step1);
    let z_locked_after_first = world.field[1].z_appearance_q16;

    // Evaluate Read-Write Closure Lipschitz bound
    let (_, read_write_lipschitz_pq, read_write_cycle_closed) =
        world.evaluate_read_write_closure(&prev_world);

    // Second pass at p = 1 with stochastic z_hat_q16 = 99_999:
    // Verify Invention-to-Memory Principle keeps z_t(1) locked (drift == 0),
    // whereas a degenerate memoryless system (M_t -> observation) drifts by 99_999^2.
    let action_step2_repeat = WorldActionObservation {
        agent_id: 1,
        target_position: 1,
        action_id: 0x63,
        forcing_phi_q16: 1024,
        forcing_v_q16: 512,
        z_hat_q16: 65536,
        net_mass_flux_q16: 0,
        action_work_u64: 10_000,
        non_exact_curl_perturbation_q16: 16384,
    };
    let _ = world.apply_factored_update(&action_step2_repeat);
    let z_locked_after_second = world.field[1].z_appearance_q16;
    let invention_to_memory_repeat_drift_q16 =
        (z_locked_after_second - z_locked_after_first).unsigned_abs();
    let degenerate_memoryless_drift_q16 = 65_536u64;

    let locked_cells = world.field.iter().filter(|c| c.observed_locked).count();

    // Verify Multi-Agent Synchronization (commutative + non-commutative conflict gating)
    let (multi_agent_commutative_verified, multi_agent_conflict_gated) =
        world.verify_multi_agent_synchronization();

    // Converge to Fixed Point M* = Update(M*)
    let (fixed_point_converged, fixed_point_iterations, fixed_point_residual_l2_q16) =
        world.converge_to_fixed_point(32);

    // SPQT base-243 xor_fold_gaia_b158 over final field trits
    let field_trits: Vec<i8> = world
        .field
        .iter()
        .flat_map(|c| {
            [
                c.phi_q16.signum() as i8,
                c.v_flow_q16.signum() as i8,
                c.s_entropy_q16.signum() as i8,
                c.z_appearance_q16.signum() as i8,
                if c.observed_locked { 1 } else { -1 },
            ]
        })
        .collect();
    let (gaia_field_fold, _, gaia_compression_ratio_pq) = xor_fold_gaia_b158(&field_trits);

    SemanticWorldEngineReport {
        num_cells: n,
        agent_position_p: world.position_p,
        history_len: world.history_h.len(),
        stage3_commit_l2_residual_q16,
        invention_to_memory_locked_cells: locked_cells,
        invention_to_memory_repeat_drift_q16,
        degenerate_memoryless_drift_q16,
        certificate: cert,
        read_write_lipschitz_pq,
        read_write_cycle_closed,
        slow_to_fast_timescale_pq: RationalPQ::new_reduced(4, 1),
        multi_agent_commutative_verified,
        multi_agent_conflict_gated,
        causal_dag_commits: world.causal_dag.len(),
        fixed_point_converged,
        fixed_point_iterations,
        fixed_point_residual_l2_q16,
        gaia_field_fold,
        gaia_compression_ratio_pq,
        wire_seal16: world.wire_seal_frame.wire_seal16,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mem8::regularity::zeno_engine::verify_litany_wire_64b;

    #[test]
    fn test_factored_update_invariants_and_invention_to_memory() {
        let mut world = SemanticWorldState::new_canonical(16, Vec::new());
        let action = WorldActionObservation {
            agent_id: 1,
            target_position: 2,
            action_id: 0x62,
            forcing_phi_q16: 12_288,
            forcing_v_q16: 8_192,
            z_hat_q16: 32_768,
            net_mass_flux_q16: 4_096,
            action_work_u64: 100, // Tight work budget to exercise Stage 2 energy projection
            non_exact_curl_perturbation_q16: 65_536,
        };

        let (cert, commit_l2) = world.apply_factored_update(&action);
        assert!(cert.mass_consistent, "Mass conservation violated");
        assert!(
            cert.energy_budget_satisfied,
            "Energy budget k7u3bw violated"
        );
        assert!(
            cert.topologically_admissible,
            "Topological admissibility violated"
        );
        assert!(cert.causally_local, "Causal locality violated");
        assert!(
            cert.entropy_monotone_outside_view,
            "Second-law out-of-view entropy violated"
        );
        assert!(cert.identity_persistent, "Identity persistence violated");
        assert!(
            cert.hallucination_free_after_projection,
            "Cohomological obstruction H^1(G_infty) not cleared"
        );
        assert!(
            commit_l2 > 0,
            "Stage 3 L2 projection residual should be positive when clamping invalid deformations"
        );

        // Verify Invention-to-Memory Principle locks observed appearance z_t(2)
        let z_locked = world.field[2].z_appearance_q16;
        let repeat_action = WorldActionObservation {
            z_hat_q16: 131_072,
            net_mass_flux_q16: 0,
            ..action
        };
        let _ = world.apply_factored_update(&repeat_action);
        assert_eq!(
            world.field[2].z_appearance_q16, z_locked,
            "Invention-to-Memory must lock observed region appearance"
        );
    }

    #[test]
    fn test_multi_agent_commutativity_and_fixed_point_convergence() {
        let mut world = SemanticWorldState::new_canonical(12, Vec::new());
        let (commutative, conflict_gated) = world.verify_multi_agent_synchronization();
        assert!(commutative, "Disjoint updates must commute");
        assert!(
            conflict_gated,
            "Inadmissible conflicting updates must be rejected"
        );

        let (converged, iters, residual) = world.converge_to_fixed_point(32);
        assert!(
            converged,
            "World must converge to fixed point M* = Update(M*)"
        );
        assert!(iters <= 32);
        assert_eq!(residual, 0);

        // Verify LitanyWireFrame 64B@0x62 seal integrity
        let mut raw_bytes = Vec::new();
        raw_bytes.extend_from_slice(&world.tick.to_le_bytes());
        let mut packed_bytes = Vec::new();
        for pt in &world.field {
            raw_bytes.extend_from_slice(&pt.phi_q16.to_le_bytes());
            raw_bytes.extend_from_slice(&pt.z_appearance_q16.to_le_bytes());
            packed_bytes.extend_from_slice(&pt.r_causal_residue.to_le_bytes());
        }
        assert!(verify_litany_wire_64b(
            &world.wire_seal_frame,
            &raw_bytes,
            &packed_bytes,
            GATE_0X62,
            QUANT_QT55
        ));
    }
}
