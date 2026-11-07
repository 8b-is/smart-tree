//! `UltraZeno` M1–M2–M3 Structural Compression & Dual-Process Information Bottleneck Engine (`src/ultra_zeno.rs`).
//!
//! Combines **Shannon / KL / Minimum Description Length (MDL) Information Theory**,
//! **Kahneman's System 1 / System 2 dual-process routing**, and the **M1–M2–M3 memory hierarchy**:
//! - **M1 (System 1 Fast Reflex):** Measures Shannon degree entropy $H(X)$, Kullback–Leibler
//!   surprise $D_{\text{KL}}(P \parallel Q)$ against the Erdős–Rényi prior, and percolation regime.
//! - **M2 (`UltraZeno` Manifold):** Applies Zeno's Dichotomy ($\varepsilon_t = \varepsilon_0 \cdot 2^{-t}$)
//!   over Erdős–Steiner metric pruning and Szemerédi $\varepsilon_t$-regularity partitions to minimize
//!   the Information Bottleneck Lagrangian $\mathcal{L}_{\text{IB}} = L_{\text{MDL}}(G, \mathcal{P}_t) - \beta \, I(G; M_2)$.
//! - **M3 (System 2 Deep Deliberation):** Extracts exact Matula–Beck `StackBitSet` Ramsey clique
//!   witnesses and Luby-MIS Moser–Tardos Lovász Local Lemma fixed points over the compressed quotient.

use crate::mcp_combinatorics_llm::{
    kernels::{
        run_erdos_renyi_scanning, run_erdos_steiner_pruning, run_moser_tardos_splitting,
        run_ramsey_clique_analysis, run_szemeredi_compression, BitMatrix, BitSet,
    },
    model_cache::{LlamaModelCache, McpProgressEmitter},
    ErdosRenyiParams, ErdosSteinerParams, LlamaRustParams, MoserTardosParams, RamseyCliqueParams,
    SzemerediParams,
};
use rayon::prelude::*;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

// ============================================================================
// 1. Information Theory Primitives (Shannon, KL Divergence, Structural MDL)
// ============================================================================

/// Binary Shannon entropy H_2(p) = -p log2(p) - (1-p) log2(1-p) in bits.
#[inline]
pub fn binary_entropy_bits(p: f64) -> f64 {
    let clamped = p.clamp(0.0, 1.0);
    if clamped <= 1e-12 || clamped >= 1.0 - 1e-12 {
        return 0.0;
    }
    -clamped * clamped.log2() - (1.0 - clamped) * (1.0 - clamped).log2()
}

/// Shannon entropy H(P) = -\sum p_i log2(p_i) in bits for a discrete distribution.
pub fn shannon_entropy_bits(probs: &[f64]) -> f64 {
    let sum: f64 = probs.iter().copied().filter(|&p| p > 0.0).sum();
    if sum <= 0.0 {
        return 0.0;
    }
    probs
        .iter()
        .copied()
        .filter(|&p| p > 1e-15)
        .map(|p| {
            let norm_p = p / sum;
            -norm_p * norm_p.log2()
        })
        .sum()
}

/// Kullback-Leibler divergence D_KL(P || Q) = \sum p_i log2(p_i / q_i) in bits.
/// Quantifies the information gain (surprise) when updating from System 1 prior Q
/// to System 2 posterior P.
pub fn kl_divergence_bits(p_target: &[f64], q_draft: &[f64]) -> f64 {
    let p_sum: f64 = p_target.iter().copied().filter(|&x| x > 0.0).sum();
    let q_sum: f64 = q_draft.iter().copied().filter(|&x| x > 0.0).sum();
    if p_sum <= 0.0 || q_sum <= 0.0 {
        return 0.0;
    }

    p_target
        .iter()
        .zip(q_draft.iter())
        .map(|(&p_raw, &q_raw)| {
            let p = p_raw / p_sum;
            if p <= 1e-15 {
                0.0
            } else {
                let q = (q_raw / q_sum).max(1e-12);
                p * (p / q).log2()
            }
        })
        .sum()
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MdlCompressionTelemetry {
    pub zeno_step: usize,
    pub dyadic_epsilon: f64,
    pub num_partitions: usize,
    pub naive_graph_bits: f64,
    pub model_header_bits: f64,
    pub residual_entropy_bits: f64,
    pub total_mdl_bits: f64,
    pub compression_ratio: f64,
    pub mutual_information_bits: f64,
}

/// Computes the Minimum Description Length (MDL) of a graph under a partition `parts`:
///   L(G, P) = L_model(P) + L_residual(G | P)
pub fn compute_structural_mdl(
    num_vertices: usize,
    adj: &BitMatrix,
    parts: &[Vec<usize>],
    zeno_step: usize,
    dyadic_epsilon: f64,
) -> (MdlCompressionTelemetry, Vec<(usize, usize)>) {
    let n = num_vertices;
    let k = parts.len().max(1);
    let naive_graph_bits = (n * n.saturating_sub(1) / 2) as f64;

    // 1. Model description cost L(P): vertex-to-partition assignment + 16-bit density table
    let k_log2 = (k as f64).log2().max(1.0);
    let num_pairs = k * k.saturating_sub(1) / 2;
    let model_header_bits = (n as f64) * k_log2 + (num_pairs as f64) * 16.0;

    let masks: Vec<BitSet> = parts.par_iter().map(|p| BitSet::from_slice(n, p)).collect();

    let mut residual_entropy_bits = 0.0f64;
    let mut total_edges = 0usize;
    let mut quotient_edges: Vec<(usize, usize)> = Vec::new();

    // Intra-part entropy + inter-part Szemerédi bipartite entropy
    for i in 0..k {
        let vi = &parts[i];
        if vi.len() > 1 {
            let intra_edges: usize = vi
                .iter()
                .map(|&u| masks[i].intersect_count_slice(adj.row(u)))
                .sum::<usize>()
                / 2;
            total_edges += intra_edges;
            let max_intra = vi.len() * (vi.len() - 1) / 2;
            let d_ii = intra_edges as f64 / (max_intra as f64);
            residual_entropy_bits += (max_intra as f64) * binary_entropy_bits(d_ii);
        }

        for j in (i + 1)..k {
            let vj = &parts[j];
            if vi.is_empty() || vj.is_empty() {
                continue;
            }
            let bip_edges: usize = vi
                .iter()
                .map(|&u| masks[j].intersect_count_slice(adj.row(u)))
                .sum();
            total_edges += bip_edges;

            let pair_cap = (vi.len() * vj.len()) as f64;
            let d_ij = (bip_edges as f64) / pair_cap;
            residual_entropy_bits += pair_cap * binary_entropy_bits(d_ij);

            // Retain regular super-edges in the M2 quotient skeleton
            if d_ij >= dyadic_epsilon {
                quotient_edges.push((i, j));
            }
        }
    }

    let global_density = if naive_graph_bits > 0.0 {
        (total_edges as f64) / naive_graph_bits
    } else {
        0.0
    };
    let prior_er_bits = naive_graph_bits * binary_entropy_bits(global_density);
    let total_mdl_bits = model_header_bits + residual_entropy_bits;

    // Mutual information I(G; M2) = H(G) - H(G | M2)
    let mutual_info = (prior_er_bits - residual_entropy_bits).max(0.0);
    let compression_ratio = if total_mdl_bits > 0.0 {
        naive_graph_bits / total_mdl_bits
    } else {
        1.0
    };

    (
        MdlCompressionTelemetry {
            zeno_step,
            dyadic_epsilon,
            num_partitions: k,
            naive_graph_bits,
            model_header_bits,
            residual_entropy_bits,
            total_mdl_bits,
            compression_ratio,
            mutual_information_bits: mutual_info,
        },
        quotient_edges,
    )
}

// ============================================================================
// 2. Dual-Process (System 1 / System 2) & M1-M2-M3 Data Structures
// ============================================================================

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum CognitiveMode {
    /// Low-entropy regime: System 1 resolves directly from M1 reflexes (N-gram / KV prefix).
    System1FastReflex,
    /// Moderate-entropy regime: M2 UltraZeno compresses M1 into a quotient graph for rapid M3 check.
    System1To2UltraZenoBridge,
    /// High-entropy / high-surprise regime: Full dyadic M2 compression + deep System 2 M3 solvers.
    System2DeepDeliberation,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UltraZenoParams {
    pub num_vertices: usize,
    pub edges: Vec<(usize, usize)>,
    #[serde(default)]
    pub terminals: Vec<usize>,
    /// Initial regularity epsilon_0 before Zeno dyadic halving (epsilon_t = epsilon_0 * 2^-t).
    #[serde(default = "default_initial_epsilon")]
    pub initial_epsilon: f64,
    /// Maximum Zeno dyadic halving steps (t = 0 .. max_zeno_steps).
    #[serde(default = "default_max_zeno_steps")]
    pub max_zeno_steps: usize,
    /// Information Bottleneck trade-off beta: higher beta favors finer M2 structure over compression.
    #[serde(default = "default_ib_beta")]
    pub ib_beta: f64,
    /// System 1 -> System 2 Shannon degree entropy threshold (in bits).
    #[serde(default = "default_system2_entropy_threshold")]
    pub system2_entropy_threshold: f64,
    /// Target clique size for M3 invariant extraction.
    #[serde(default = "default_target_clique_k")]
    pub target_clique_k: usize,
    /// Optional GGUF model path for M3 grammar-constrained synthesis.
    #[serde(default)]
    pub model_path: Option<String>,
}

fn default_initial_epsilon() -> f64 {
    0.32
}
fn default_max_zeno_steps() -> usize {
    4
}
fn default_ib_beta() -> f64 {
    1.25
}
fn default_system2_entropy_threshold() -> f64 {
    1.50
}
fn default_target_clique_k() -> usize {
    4
}

#[derive(Debug, Serialize)]
pub struct UltraZenoReport {
    pub architecture: &'static str,
    pub cognitive_mode: CognitiveMode,
    pub m1_system1_sensory: Value,
    pub m2_ultra_zeno_compression: Value,
    pub m3_system2_invariants: Value,
}

// ============================================================================
// 3. M1 -> M2 (UltraZeno) -> M3 Execution Pipeline
// ============================================================================

pub fn run_ultra_zeno_engine(
    model_cache: &LlamaModelCache,
    params: UltraZenoParams,
    emitter: &McpProgressEmitter,
) -> Result<Value, String> {
    let n = params.num_vertices;
    if n == 0 {
        return Err("num_vertices must be at least 1".into());
    }

    // ========================================================================
    // TIER M1 (System 1): Fast Sensory Intake, Degree Entropy & KL Surprise
    // ========================================================================
    let mut adj = BitMatrix::new(n);
    let mut degrees = vec![0usize; n];
    for &(u, v) in &params.edges {
        if u < n && v < n && u != v {
            adj.add_undirected_edge(u, v);
        }
    }
    for (u, degree) in degrees.iter_mut().enumerate() {
        *degree = adj.row(u).iter().map(|w| w.count_ones() as usize).sum();
    }

    let deg_probs: Vec<f64> = degrees.iter().map(|&d| (d + 1) as f64).collect();
    let m1_degree_entropy_bits = shannon_entropy_bits(&deg_probs);

    // Compare empirical degree distribution P against uniform Erdős-Rényi prior Q via D_KL(P || Q)
    let uniform_q = vec![1.0 / (n as f64); n];
    let m1_kl_surprise_bits = kl_divergence_bits(&deg_probs, &uniform_q);

    let max_edges = (n * n.saturating_sub(1) / 2).max(1);
    let empirical_p = (params.edges.len() as f64 / max_edges as f64).clamp(0.0, 1.0);
    let er_scan = run_erdos_renyi_scanning(
        ErdosRenyiParams {
            n,
            p: empirical_p,
            seed: Some(0x5151_5252_4D31_4D32),
        },
        &McpProgressEmitter::default(),
    )?;

    // Dual-Process Gating: decide whether System 1 suffices or System 2 deliberation is required
    let cognitive_mode = if m1_degree_entropy_bits < params.system2_entropy_threshold * 0.5
        && m1_kl_surprise_bits < 0.05
    {
        CognitiveMode::System1FastReflex
    } else if m1_kl_surprise_bits < 0.25 {
        CognitiveMode::System1To2UltraZenoBridge
    } else {
        CognitiveMode::System2DeepDeliberation
    };

    if !emitter.emit_progress(
        1,
        3,
        json!({
            "solver": "ultra_zeno_structural_compression",
            "tier": "M1_System1_Sensory",
            "cognitive_mode": cognitive_mode,
            "shannon_degree_entropy_bits": m1_degree_entropy_bits,
            "kl_divergence_from_er_bits": m1_kl_surprise_bits,
            "percolation_regime": er_scan["regime"]
        }),
    ) {
        return Err("Client disconnected at M1 Sensory stage".into());
    }

    // ========================================================================
    // TIER M2 (`UltraZeno`): Dyadic Structural Compression (epsilon_t = eps_0 * 2^-t)
    // ========================================================================
    // Step 2A: Prune zero-information non-terminal branches via Erdős-Steiner
    let terminals = if params.terminals.is_empty() {
        (0..n).step_by((n / 8).max(1)).collect()
    } else {
        params.terminals.clone()
    };
    let weighted_edges: Vec<(usize, usize, f64)> =
        params.edges.iter().map(|&(u, v)| (u, v, 1.0)).collect();

    let steiner_pruned = run_erdos_steiner_pruning(
        ErdosSteinerParams {
            edges: weighted_edges,
            terminals,
            pruning_threshold: 2.0,
        },
        &McpProgressEmitter::default(),
    )?;

    // Step 2B: Zeno Dyadic Halving Loop (epsilon_t = initial_epsilon * 2^-t)
    // Evaluates Szemerédi regularity partitions at progressively finer dyadic scales
    // and selects the fixed point minimizing the Information Bottleneck Lagrangian:
    //   J(P_t) = L_MDL(G, P_t) - beta * I(G; M2_t)
    let max_steps = params.max_zeno_steps.clamp(1, 6);
    let mut zeno_trajectory: Vec<MdlCompressionTelemetry> = Vec::with_capacity(max_steps);
    let mut best_ib_score = f64::INFINITY;
    let mut optimal_quotient_edges: Vec<(usize, usize)> = Vec::new();
    let mut optimal_parts: Vec<Vec<usize>> = Vec::new();
    let mut finest_szemeredi_partition: Vec<Vec<usize>> = Vec::new();
    let mut optimal_num_parts = 2usize;
    let mut optimal_zeno_step = 0usize;

    for step in 0..max_steps {
        let dyadic_scale = 0.5f64.powi(step as i32);
        let eps_t = (params.initial_epsilon * dyadic_scale).clamp(0.02, 0.49);
        let min_parts = (1usize << (step + 1)).min(n);

        let sz = run_szemeredi_compression(
            SzemerediParams {
                num_vertices: n,
                edges: params.edges.clone(),
                epsilon: eps_t,
                min_parts,
            },
            &McpProgressEmitter::default(),
        )?;
        if let Ok(parts) = serde_json::from_value::<Vec<Vec<usize>>>(sz["partitions"].clone()) {
            if !parts.is_empty() {
                finest_szemeredi_partition = parts;
            }
        }

        // Construct degree-sorted equitable partition of size `min_parts` for MDL evaluation
        let mut sorted_vertices: Vec<usize> = (0..n).collect();
        sorted_vertices.sort_unstable_by_key(|&u| degrees[u]);
        let mut parts: Vec<Vec<usize>> = (0..min_parts).map(|_| Vec::new()).collect();
        for (idx, v) in sorted_vertices.into_iter().enumerate() {
            parts[idx % min_parts].push(v);
        }

        let (telemetry, q_edges) = compute_structural_mdl(n, &adj, &parts, step, eps_t);
        let ib_lagrangian =
            telemetry.total_mdl_bits - params.ib_beta * telemetry.mutual_information_bits;

        if ib_lagrangian < best_ib_score {
            best_ib_score = ib_lagrangian;
            optimal_quotient_edges = q_edges;
            optimal_parts = parts.clone();
            optimal_num_parts = telemetry.num_partitions;
            optimal_zeno_step = step;
        }

        zeno_trajectory.push(telemetry);

        // Early Zeno convergence if System 1 Fast Reflex mode is active
        if cognitive_mode == CognitiveMode::System1FastReflex {
            break;
        }
    }

    let best_mdl = &zeno_trajectory[optimal_zeno_step];

    if !emitter.emit_progress(
        2,
        3,
        json!({
            "solver": "ultra_zeno_structural_compression",
            "tier": "M2_UltraZeno_Manifold",
            "optimal_zeno_step": optimal_zeno_step,
            "dyadic_epsilon": best_mdl.dyadic_epsilon,
            "quotient_supernodes": optimal_num_parts,
            "mdl_compression_ratio": best_mdl.compression_ratio,
            "mutual_information_bits": best_mdl.mutual_information_bits
        }),
    ) {
        return Err("Client disconnected at M2 UltraZeno stage".into());
    }

    // ========================================================================
    // TIER M3 (System 2): Deep Invariant Extraction & Homeostatic Equilibrium
    // ========================================================================
    // Run Ramsey clique extraction on BOTH the M2 compressed quotient skeleton
    // (fast macro-structure check) and the core graph (exact micro-structure witness).
    let m2_quotient_ramsey = run_ramsey_clique_analysis(
        RamseyCliqueParams {
            num_vertices: optimal_num_parts.max(2),
            edges: optimal_quotient_edges.clone(),
            k_clique: params.target_clique_k.min(optimal_num_parts.max(2)),
            l_independent: params.target_clique_k.min(optimal_num_parts.max(2)),
        },
        &McpProgressEmitter::default(),
    )?;

    let m3_exact_ramsey = run_ramsey_clique_analysis(
        RamseyCliqueParams {
            num_vertices: n,
            edges: params.edges.clone(),
            k_clique: params.target_clique_k,
            l_independent: params.target_clique_k,
        },
        &McpProgressEmitter::default(),
    )?;

    // Constructive Moser-Tardos LLL over the M2 quotient super-nodes to lock in M3 homeostasis
    let bad_events: Vec<Vec<i32>> = optimal_quotient_edges
        .iter()
        .enumerate()
        .map(|(idx, &(u, v))| {
            let lit1 = (u + 1) as i32;
            let lit2 = -((v + 1) as i32);
            let lit3 = (((idx + 1) % optimal_num_parts.max(1)) + 1) as i32;
            vec![lit1, lit2, lit3]
        })
        .collect();

    let m3_lll_equilibrium = run_moser_tardos_splitting(
        MoserTardosParams {
            num_variables: optimal_num_parts.max(2),
            bad_events,
            max_resamples: 500,
        },
        &McpProgressEmitter::default(),
    )?;

    let m3_llm_synthesis = if let Some(ref model_path) = params.model_path {
        let prompt = format!(
            "M1 Entropy: {:.3} bits (KL Surprise: {:.3} bits)\n\
             M2 UltraZeno Compression Ratio: {:.2}x ({} super-nodes at dyadic step {})\n\
             M3 Exact Clique Witness: {}\n\
             Synthesize structural invariant:",
            m1_degree_entropy_bits,
            m1_kl_surprise_bits,
            best_mdl.compression_ratio,
            optimal_num_parts,
            optimal_zeno_step,
            m3_exact_ramsey["largest_clique_witness"]
        );
        model_cache.submit_continuous_batch_job(
            LlamaRustParams {
                model_path: model_path.clone(),
                draft_model_path: None,
                speculative_depth: 4,
                speculative_tree_width: 2,
                grammar: None,
                json_schema: None,
                prompt,
                n_ctx: 4096,
                n_gpu_layers: 99,
                n_batch: 2048,
                temperature: 0.2,
                max_tokens: 128,
            },
            emitter.clone(),
        )?
    } else {
        json!({
            "mode": "symbolic_m3_fixed_point",
            "information_bottleneck_lagrangian": best_ib_score
        })
    };

    let _ = emitter.emit_progress(
        3,
        3,
        json!({
            "solver": "ultra_zeno_structural_compression",
            "tier": "M3_System2_Invariants",
            "quotient_clique_witness": m2_quotient_ramsey["largest_clique_witness"],
            "exact_clique_witness": m3_exact_ramsey["largest_clique_witness"],
            "lll_status": m3_lll_equilibrium["status"]
        }),
    );

    let report = UltraZenoReport {
        architecture: "M1_M2_M3_DUAL_PROCESS_ULTRAZENO_BOTTLENECK",
        cognitive_mode,
        m1_system1_sensory: json!({
            "shannon_degree_entropy_bits": m1_degree_entropy_bits,
            "kl_divergence_from_er_prior_bits": m1_kl_surprise_bits,
            "erdos_renyi_percolation": er_scan
        }),
        m2_ultra_zeno_compression: json!({
            "optimal_zeno_step": optimal_zeno_step,
            "optimal_dyadic_epsilon": best_mdl.dyadic_epsilon,
            "quotient_supernodes": optimal_num_parts,
            "quotient_superedges": optimal_quotient_edges.len(),
            "quotient_superedge_list": optimal_quotient_edges.clone(),
            "optimal_partition": optimal_parts,
            "finest_szemeredi_partition": finest_szemeredi_partition,
            "mdl_compression_ratio": best_mdl.compression_ratio,
            "mutual_information_bits": best_mdl.mutual_information_bits,
            "information_bottleneck_lagrangian": best_ib_score,
            "steiner_backbone_summary": {
                "retained_edges": steiner_pruned["retained_edge_count"],
                "pruned_leaves": steiner_pruned["pruned_leaf_vertices"],
                "contracted_series": steiner_pruned["contracted_degree2_vertices"]
            },
            "zeno_dyadic_trajectory": zeno_trajectory
        }),
        m3_system2_invariants: json!({
            "quotient_macro_ramsey": m2_quotient_ramsey,
            "exact_micro_ramsey": m3_exact_ramsey,
            "moser_tardos_homeostasis": m3_lll_equilibrium,
            "synthesis": m3_llm_synthesis
        }),
    };

    serde_json::to_value(report).map_err(|e| e.to_string())
}
