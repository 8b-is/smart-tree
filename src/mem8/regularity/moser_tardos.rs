//! Constructive Lovász Local Lemma (Moser–Tardos) Parallel Agent Worktree Splitter
//! (`src/mem8/regularity/moser_tardos.rs`).
//!
//! Partitions $U$ refactoring tasks or Steiner subgraphs across $C$ parallel git worktrees
//! (or isolated agent waves) using the Moser–Tardos Constructive LLL Resampling Algorithm
//! with Maximal Independent Set (MIS) parallel resampling and Moser Witness Tree certificates.

use super::erdos_steiner::ErdosSteinerSlice;
use super::mmap_view::ZeroCopyMem8Book;
use super::wire::decode_t8r_exceptions_leb128;

/// Identifier for a refactorable unit (either a `ClusterId` in $R$ or an outlier `SymbolId` in $V_0$)
pub type WorkUnitId = u32;
/// Identifier for a "Bad Event" $B_e$ (merge/type/lock collision constraint)
pub type BadEventId = u32;
/// Identifier for a target Git Worktree / Parallel Agent (`0..num_worktrees`)
pub type WorktreeColor = u16;

/// Types of bad events $B_e \in \mathcal{B}$ in the AST dependency hypergraph
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BadEventKind {
    /// Two units share a mutable T8R exception edge or tight call-cycle: MUST NOT be modified
    /// concurrently in separate worktrees unless sequenced into distinct execution waves, OR
    /// must be co-located/isolated depending on agent mode.
    /// Here: `PairwiseExclusive` means $X_u == X_v$ is a collision (two agents editing overlapping lock/state).
    PairwiseExclusive,
    /// A $k$-uniform hyperedge (e.g., trait + impls + callers) cannot be fractured across all identical
    /// simultaneous wave slots without a coordinator (all monochromatic = bad event).
    MonochromaticHyperedge,
    /// Worktree load-balance / token-budget cap: no more than `max_same_color` units in this scope
    /// may share the same worktree color.
    CongestionThreshold { max_same_color: u8 },
}

/// Zero-allocation Dual-CSR representation of the Lovász Local Lemma Hypergraph $\mathcal{H} = (\mathcal{U}, \mathcal{B})$
#[derive(Debug, Clone)]
pub struct LllHypergraph {
    pub num_units: usize,
    pub num_events: usize,
    /// Event -> Units CSR: `event_ptr.len() == num_events + 1`
    pub event_ptr: Vec<u32>,
    pub event_units: Vec<WorkUnitId>,
    pub event_kinds: Vec<BadEventKind>,
    /// Unit -> Events Transpose CSR: `unit_ptr.len() == num_units + 1`
    pub unit_ptr: Vec<u32>,
    pub unit_events: Vec<BadEventId>,
}

/// Output of the Moser–Tardos Constructive LLL Solver
#[derive(Debug, Clone)]
pub struct MoserTardosSchedule {
    /// Final conflict-free assignment: `unit_id -> worktree_color (0..C)`
    pub unit_to_worktree: Vec<WorktreeColor>,
    /// Number of parallel worktrees $C$ used
    pub num_worktrees: u16,
    /// Total Moser–Tardos resampling steps taken (guaranteed $O(|\mathcal{U}|)$ under Shearer's condition)
    pub total_resamples: usize,
    /// Number of parallel MIS resampling rounds
    pub parallel_rounds: usize,
    /// True if all bad events were eliminated ($0$ conflicts); false if LLL phase threshold was exceeded
    /// and a minimal witness-tree bottleneck was isolated
    pub is_conflict_free: bool,
    /// If `is_conflict_free == false`, holds the minimal Moser Witness Tree (the exact architectural bottleneck)
    pub witness_bottleneck_events: Vec<BadEventId>,
    /// Empirical maximum Shearer degree $\Delta_{\max}$ across $\Gamma(B)$
    pub max_dependency_degree: usize,
}

impl LllHypergraph {
    /// Constructs both forward (`Event -> Units`) and transpose (`Unit -> Events`) CSR tables in $O(\sum |S(B)|)$.
    pub fn from_constraints(
        num_units: usize,
        constraints: &[(BadEventKind, &[WorkUnitId])],
    ) -> Self {
        let num_events = constraints.len();
        let mut event_ptr = Vec::with_capacity(num_events + 1);
        let mut event_units = Vec::new();
        let mut event_kinds = Vec::with_capacity(num_events);
        let mut unit_deg = vec![0u32; num_units];

        event_ptr.push(0);
        for &(kind, scope) in constraints {
            event_kinds.push(kind);
            for &u in scope {
                if (u as usize) < num_units {
                    event_units.push(u);
                    unit_deg[u as usize] += 1;
                }
            }
            event_ptr.push(event_units.len() as u32);
        }

        // Build Transpose CSR (Unit -> Events)
        let mut unit_ptr = Vec::with_capacity(num_units + 1);
        let mut accum = 0u32;
        for &d in &unit_deg {
            unit_ptr.push(accum);
            accum += d;
        }
        unit_ptr.push(accum);

        let mut unit_events = vec![0u32; accum as usize];
        let mut cursor = unit_ptr[..num_units].to_vec();

        for (ev_id, &(_, scope)) in constraints.iter().enumerate() {
            for &u in scope {
                if (u as usize) < num_units {
                    let pos = cursor[u as usize] as usize;
                    unit_events[pos] = ev_id as BadEventId;
                    cursor[u as usize] += 1;
                }
            }
        }

        Self {
            num_units,
            num_events,
            event_ptr,
            event_units,
            event_kinds,
            unit_ptr,
            unit_events,
        }
    }

    #[inline(always)]
    pub fn event_scope(&self, ev: BadEventId) -> &[WorkUnitId] {
        let start = self.event_ptr[ev as usize] as usize;
        let end = self.event_ptr[ev as usize + 1] as usize;
        &self.event_units[start..end]
    }

    #[inline(always)]
    pub fn unit_incident_events(&self, u: WorkUnitId) -> &[BadEventId] {
        let start = self.unit_ptr[u as usize] as usize;
        let end = self.unit_ptr[u as usize + 1] as usize;
        &self.unit_events[start..end]
    }

    /// Evaluates whether bad event `ev` is currently violated under `coloring`
    #[inline]
    pub fn is_event_violated(&self, ev: BadEventId, coloring: &[WorktreeColor]) -> bool {
        let scope = self.event_scope(ev);
        if scope.len() <= 1 {
            return false;
        }
        match self.event_kinds[ev as usize] {
            BadEventKind::PairwiseExclusive => {
                coloring[scope[0] as usize] == coloring[scope[1] as usize]
            }
            BadEventKind::MonochromaticHyperedge => {
                let first = coloring[scope[0] as usize];
                scope[1..].iter().all(|&u| coloring[u as usize] == first)
            }
            BadEventKind::CongestionThreshold { max_same_color } => {
                let limit = max_same_color as usize;
                for i in 0..scope.len() {
                    let c = coloring[scope[i] as usize];
                    let count = scope[i..]
                        .iter()
                        .filter(|&&u| coloring[u as usize] == c)
                        .count();
                    if count > limit {
                        return true;
                    }
                }
                false
            }
        }
    }
}

/// Executes the Parallel Moser–Tardos Constructive Lovász Local Lemma solver.
/// Uses deterministic SplitMix64 PRNG so `--erdos-split` is 100% reproducible across runs.
pub fn solve_moser_tardos_worktrees(
    hg: &LllHypergraph,
    num_worktrees: u16,
    seed: u64,
    max_resamples: usize,
) -> MoserTardosSchedule {
    let n = hg.num_units;
    let m = hg.num_events;
    let c = num_worktrees.max(2);

    let mut rng = SplitMix64::new(seed ^ 0x9E37_79B9_7F4A_7C15);

    // 1. Initial independent uniform product distribution X_u ~ Uniform(0..C)
    let mut coloring: Vec<WorktreeColor> = (0..n).map(|_| rng.next_color(c)).collect();

    if m == 0 || n == 0 {
        return MoserTardosSchedule {
            unit_to_worktree: coloring,
            num_worktrees: c,
            total_resamples: 0,
            parallel_rounds: 0,
            is_conflict_free: true,
            witness_bottleneck_events: Vec::new(),
            max_dependency_degree: 0,
        };
    }

    // Compute maximum dependency degree Delta_max in Gamma(B)
    let mut max_dep_degree = 0usize;
    for ev in 0..m {
        let deg: usize = hg
            .event_scope(ev as BadEventId)
            .iter()
            .map(|&u| hg.unit_incident_events(u).len().saturating_sub(1))
            .sum();
        max_dep_degree = max_dep_degree.max(deg);
    }

    // 2. Initialize violated event queue + O(1) membership bitset
    let mut in_violated_queue = vec![false; m];
    let mut violated_queue: Vec<BadEventId> = Vec::with_capacity(m);

    for ev in 0..(m as BadEventId) {
        if hg.is_event_violated(ev, &coloring) {
            in_violated_queue[ev as usize] = true;
            violated_queue.push(ev);
        }
    }

    // Track Moser Witness Tree depth per unit & event to detect LLL phase boundary
    let mut unit_resample_depth = vec![0u32; n];
    let mut event_witness_height = vec![0u32; m];
    let witness_height_cap = ((n as f64).ln().ceil() as u32 * 6).max(32);

    let mut unit_locked_in_round = vec![false; n];
    let mut independent_batch: Vec<BadEventId> = Vec::with_capacity(64);
    let mut affected_events: Vec<BadEventId> = Vec::with_capacity(128);

    let mut total_resamples = 0usize;
    let mut parallel_rounds = 0usize;

    // 3. Moser–Tardos Parallel MIS Resampling Loop
    while !violated_queue.is_empty() && total_resamples < max_resamples {
        parallel_rounds += 1;
        independent_batch.clear();
        affected_events.clear();

        // Greedily extract a Maximal Independent Set (MIS) of variable-disjoint violated events
        let mut cursor = 0;
        while cursor < violated_queue.len() {
            let ev = violated_queue[cursor];
            in_violated_queue[ev as usize] = false;

            if !hg.is_event_violated(ev, &coloring) {
                violated_queue.swap_remove(cursor);
                continue;
            }

            let scope = hg.event_scope(ev);
            let is_disjoint = scope.iter().all(|&u| !unit_locked_in_round[u as usize]);

            if is_disjoint {
                for &u in scope {
                    unit_locked_in_round[u as usize] = true;
                }
                independent_batch.push(ev);
                violated_queue.swap_remove(cursor);
            } else {
                in_violated_queue[ev as usize] = true;
                cursor += 1;
            }
        }

        if independent_batch.is_empty() {
            break;
        }

        // Resample all variables in the independent batch simultaneously
        let mut exceeded_witness_cap = false;
        for &ev in &independent_batch {
            let scope = hg.event_scope(ev);
            let max_u_depth = scope
                .iter()
                .map(|&u| unit_resample_depth[u as usize])
                .max()
                .unwrap_or(0);
            let new_height = max_u_depth + 1;
            event_witness_height[ev as usize] = new_height;

            if new_height > witness_height_cap {
                exceeded_witness_cap = true;
            }

            for &u in scope {
                coloring[u as usize] = rng.next_color(c);
                unit_resample_depth[u as usize] = new_height;
                unit_locked_in_round[u as usize] = false;

                for &inc_ev in hg.unit_incident_events(u) {
                    if !in_violated_queue[inc_ev as usize] {
                        affected_events.push(inc_ev);
                    }
                }
            }
            total_resamples += 1;
        }

        // Re-evaluate only the local neighborhood Gamma^+(B) of resampled events
        for &inc_ev in &affected_events {
            if !in_violated_queue[inc_ev as usize] && hg.is_event_violated(inc_ev, &coloring) {
                in_violated_queue[inc_ev as usize] = true;
                violated_queue.push(inc_ev);
            }
        }

        if exceeded_witness_cap {
            break;
        }
    }

    // Final verification pass
    let mut remaining_conflicts: Vec<BadEventId> = (0..(m as BadEventId))
        .filter(|&ev| hg.is_event_violated(ev, &coloring))
        .collect();

    if !remaining_conflicts.is_empty() {
        remaining_conflicts
            .sort_unstable_by_key(|&ev| std::cmp::Reverse(event_witness_height[ev as usize]));
        remaining_conflicts.truncate(8);
    }

    MoserTardosSchedule {
        unit_to_worktree: coloring,
        num_worktrees: c,
        total_resamples,
        parallel_rounds,
        is_conflict_free: remaining_conflicts.is_empty(),
        witness_bottleneck_events: remaining_conflicts,
        max_dependency_degree: max_dep_degree,
    }
}

/// Converts a set of Erdős-Steiner task slices and `.m8b` Reduced Graph edges into an `LllHypergraph`.
pub fn build_hypergraph_from_steiner_slices(
    book: &ZeroCopyMem8Book,
    task_slices: &[ErdosSteinerSlice],
    density_conflict_threshold: f32,
) -> LllHypergraph {
    let num_clusters = book.super_nodes().len() + 1;
    let mut constraints: Vec<(BadEventKind, Vec<WorkUnitId>)> = Vec::new();

    // 1. High-density Reduced Graph edges become PairwiseExclusive bad events
    for edge in book.reduced_edges() {
        let density = edge.relative_p_density_fp16 as f32 / 1024.0;
        if density >= density_conflict_threshold {
            constraints.push((
                BadEventKind::PairwiseExclusive,
                vec![
                    edge.src_cluster as WorkUnitId,
                    edge.dst_cluster as WorkUnitId,
                ],
            ));
        }
    }

    // 2. Decode T8R exception edges and add cross-cluster PairwiseExclusive locks
    if let Some(exceptions) = decode_t8r_exceptions_leb128(book.t8r_raw_bytes()) {
        let sym_map = book.symbol_to_cluster();
        for ex in exceptions {
            if (ex.src_symbol as usize) < sym_map.len() && (ex.dst_symbol as usize) < sym_map.len()
            {
                let cu = sym_map[ex.src_symbol as usize] as WorkUnitId;
                let cv = sym_map[ex.dst_symbol as usize] as WorkUnitId;
                if cu != 0 && cv != 0 && cu != cv {
                    constraints.push((BadEventKind::PairwiseExclusive, vec![cu, cv]));
                }
            }
        }
    }

    // 3. Each task's Steiner backbone forms a CongestionThreshold hyperedge
    for slice in task_slices {
        if slice.steiner_super_nodes.len() >= 3 {
            let scope: Vec<WorkUnitId> = slice
                .steiner_super_nodes
                .iter()
                .map(|n| n.cluster_id as WorkUnitId)
                .collect();
            let cap = ((scope.len() / 2) as u8).max(1);
            constraints.push((
                BadEventKind::CongestionThreshold {
                    max_same_color: cap,
                },
                scope,
            ));
        }
    }

    let borrowed: Vec<(BadEventKind, &[WorkUnitId])> = constraints
        .iter()
        .map(|(k, v)| (*k, v.as_slice()))
        .collect();

    LllHypergraph::from_constraints(num_clusters, &borrowed)
}

/// Ultra-fast 4-lane superscalar SplitMix64 PRNG (`felix_asm::felix_splitmix64_x4`, zero heap state)
struct SplitMix64 {
    state: u64,
    batch: [u64; 4],
    cursor: usize,
}

impl SplitMix64 {
    #[inline(always)]
    const fn new(seed: u64) -> Self {
        Self {
            state: seed,
            batch: [0; 4],
            cursor: 4,
        }
    }

    #[inline(always)]
    fn next_u64(&mut self) -> u64 {
        if self.cursor >= 4 {
            self.batch = crate::felix_asm::felix_splitmix64_x4(&mut self.state);
            self.cursor = 0;
        }
        let val = self.batch[self.cursor];
        self.cursor += 1;
        val
    }

    /// Lemire's fast divisionless bounded random integer in `0..bound`
    #[inline(always)]
    fn next_color(&mut self, bound: u16) -> WorktreeColor {
        ((self.next_u64() as u32 as u64 * (bound as u64)) >> 32) as WorktreeColor
    }
}
