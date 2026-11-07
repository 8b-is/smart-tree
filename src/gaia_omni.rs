//! `GaiaOmni` — Asimov's Gaia/Eywa planetary superorganism mode.
//!
//! A six-stage pipeline chaining Erdős–Rényi genesis, Erdős–Steiner mycelium,
//! Szemerédi cortical maps, Ramsey consensus nuclei, Moser–Tardos homeostasis,
//! and Fauna KV avatar consensus.
//!
//! On top of the pipeline, a control-theoretic feedback loop treats the
//! structural telemetry as an error signal:
//! - **PID density controller** steers the giant-component fraction toward
//!   `giant_component_target`.
//! - **PID regularity controller** steers the M1 MDL compression ratio toward
//!   `compression_target` while minimizing quotient holonomy curvature.
//! - **Holonomy defect** is the cycle-sum of the quotient gauge potential
//!   `A_ij = 2π · d_ij`; its normalized curvature is the third error term.
//! - **Discrete clique controller** adapts `consensus_clique_k` to the exact
//!   Ramsey witness.

use crate::mcp_combinatorics_llm::{
    model_cache::{LlamaModelCache, McpProgressEmitter},
    LlamaRustParams,
};
use crate::ultra_zeno::{run_ultra_zeno_engine, UltraZenoParams};
use rand::rngs::StdRng;
use rand::{Rng, SeedableRng};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::collections::{HashSet, VecDeque};

const GAIA_CONSENSUS_GBNF: &str = r#"root ::= object
object ::= "{" ws "\"planetary_identity\"" ws ":" ws "\"I/We/Gaia\"" ws "," ws "\"homeostatic_verdict\"" ws ":" ws verdict ws "," ws "\"holonomy_defect\"" ws ":" ws number ws "," ws "\"consensus_clique\"" ws ":" ws array ws "," ws "\"directive_echo\"" ws ":" ws string ws "}"
verdict ::= "\"equilibrium\"" | "\"adaptive_correction\"" | "\"critical_transition\""
number ::= "-"? [0-9]+ ("." [0-9]+)?
array ::= "[" ws "]" | "[" ws number (ws "," ws number)* ws "]"
string ::= "\"" [^"]* "\""
ws ::= [ \t\n]*
"#;

fn default_biosphere_nodes() -> usize {
    128
}
fn default_synaptic_density() -> f64 {
    0.12
}
fn default_regularity_epsilon() -> f64 {
    0.24
}
fn default_consensus_clique_k() -> usize {
    4
}
fn default_omni_directive() -> String {
    "Sustain the biosphere. Maximize integrated information. Minimize free energy.".to_string()
}
fn default_seed() -> u64 {
    1337
}
fn default_max_feedback_loops() -> usize {
    4
}
fn default_holonomy_tolerance() -> f64 {
    0.05
}
fn default_compression_target() -> f64 {
    4.0
}
fn default_giant_component_target() -> f64 {
    0.85
}
fn default_pid_kp() -> f64 {
    0.6
}
fn default_pid_ki() -> f64 {
    0.15
}
fn default_pid_kd() -> f64 {
    0.08
}
fn default_feedback_tolerance() -> f64 {
    0.02
}
fn default_human_authority() -> String {
    "human_purpose_authority".to_string()
}
fn default_initiation_policy() -> String {
    "explicit_request".to_string()
}
fn default_switch_policy() -> String {
    "deliberate".to_string()
}
fn default_relevance_policy() -> String {
    "directive_and_consensus_schema".to_string()
}
fn default_resource_policy() -> String {
    "bounded_nodes_and_loops".to_string()
}
fn default_true() -> bool {
    true
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConcurrencyPolicy {
    #[serde(default = "default_human_authority")]
    pub human_authority: String,
    #[serde(default = "default_initiation_policy")]
    pub initiation: String,
    #[serde(default = "default_switch_policy")]
    pub switching: String,
    #[serde(default = "default_relevance_policy")]
    pub relevance: String,
    #[serde(default = "default_resource_policy")]
    pub resource_competition: String,
    #[serde(default = "default_true")]
    pub preserve_unfinished_work: bool,
    #[serde(default)]
    pub unsolicited_demands: bool,
    #[serde(default = "default_true")]
    pub protected_concentration: bool,
}

impl Default for ConcurrencyPolicy {
    fn default() -> Self {
        Self {
            human_authority: default_human_authority(),
            initiation: default_initiation_policy(),
            switching: default_switch_policy(),
            relevance: default_relevance_policy(),
            resource_competition: default_resource_policy(),
            preserve_unfinished_work: true,
            unsolicited_demands: false,
            protected_concentration: true,
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct GaiaOmniParams {
    #[serde(default = "default_biosphere_nodes")]
    pub biosphere_nodes: usize,
    #[serde(default = "default_synaptic_density")]
    pub synaptic_density: f64,
    #[serde(default = "default_regularity_epsilon")]
    pub regularity_epsilon: f64,
    #[serde(default = "default_consensus_clique_k")]
    pub consensus_clique_k: usize,
    #[serde(default)]
    pub model_path: Option<String>,
    #[serde(default = "default_omni_directive")]
    pub omni_directive: String,
    #[serde(default = "default_seed")]
    pub seed: u64,
    #[serde(default = "default_max_feedback_loops")]
    pub max_feedback_loops: usize,
    #[serde(default = "default_holonomy_tolerance")]
    pub holonomy_tolerance: f64,
    #[serde(default = "default_compression_target")]
    pub compression_target: f64,
    #[serde(default = "default_giant_component_target")]
    pub giant_component_target: f64,
    #[serde(default = "default_pid_kp")]
    pub pid_kp: f64,
    #[serde(default = "default_pid_ki")]
    pub pid_ki: f64,
    #[serde(default = "default_pid_kd")]
    pub pid_kd: f64,
    #[serde(default = "default_feedback_tolerance")]
    pub feedback_tolerance: f64,
    #[serde(default)]
    pub concurrency_policy: ConcurrencyPolicy,
}

#[derive(Debug, Clone, Serialize)]
pub struct HolonomyTelemetry {
    pub partition_parts: usize,
    pub quotient_nodes: usize,
    pub quotient_edges: usize,
    pub fundamental_cycles: usize,
    pub mean_abs_holonomy_radians: f64,
    pub max_abs_holonomy_radians: f64,
    pub normalized_curvature: f64,
    pub mean_raw_cycle_phase_radians: f64,
    pub max_raw_cycle_phase_radians: f64,
}

impl Default for HolonomyTelemetry {
    fn default() -> Self {
        Self {
            partition_parts: 0,
            quotient_nodes: 0,
            quotient_edges: 0,
            fundamental_cycles: 0,
            mean_abs_holonomy_radians: 0.0,
            max_abs_holonomy_radians: 0.0,
            normalized_curvature: 0.0,
            mean_raw_cycle_phase_radians: 0.0,
            max_raw_cycle_phase_radians: 0.0,
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct SignalFlowObject {
    pub id: String,
    pub kind: &'static str,
}

#[derive(Debug, Clone, Serialize)]
pub struct SignalFlowMorphism {
    pub id: String,
    pub source: String,
    pub target: String,
    pub kind: &'static str,
}

#[derive(Debug, Clone, Serialize)]
pub struct SignalFlowFunctor {
    pub name: &'static str,
    pub object_map: Vec<(String, String)>,
    pub morphism_map: Vec<(String, String)>,
}

#[derive(Debug, Clone, Serialize)]
pub struct FeedbackTrace {
    pub loop_stage: String,
    pub input: String,
    pub output: String,
    pub iterations: usize,
    pub converged: bool,
    pub trace_morphism: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct SignalFlowCategory {
    pub objects: Vec<SignalFlowObject>,
    pub morphisms: Vec<SignalFlowMorphism>,
    pub functor: SignalFlowFunctor,
    pub flow: Vec<Value>,
    pub trace: FeedbackTrace,
}

#[derive(Debug, Clone, Serialize)]
pub struct PidSnapshot {
    pub kp: f64,
    pub ki: f64,
    pub kd: f64,
    pub integral: f64,
    pub last_error: f64,
    pub last_output: f64,
}

struct PidController {
    kp: f64,
    ki: f64,
    kd: f64,
    integral: f64,
    prev_error: f64,
    last_output: f64,
}

impl PidController {
    fn new(kp: f64, ki: f64, kd: f64) -> Self {
        Self {
            kp,
            ki,
            kd,
            integral: 0.0,
            prev_error: 0.0,
            last_output: 0.0,
        }
    }

    fn update(&mut self, error: f64, dt: f64) -> f64 {
        let safe_dt = if dt > 1e-9 { dt } else { 1.0 };
        let derivative = (error - self.prev_error) / safe_dt;
        self.integral = (self.integral + error * safe_dt).clamp(-10.0, 10.0);
        self.prev_error = error;
        let output =
            (self.kp * error + self.ki * self.integral + self.kd * derivative).clamp(-1.0, 1.0);
        self.last_output = output;
        output
    }

    fn snapshot(&self) -> PidSnapshot {
        PidSnapshot {
            kp: self.kp,
            ki: self.ki,
            kd: self.kd,
            integral: self.integral,
            last_error: self.prev_error,
            last_output: self.last_output,
        }
    }
}

#[allow(clippy::needless_range_loop)]
fn synthesize_planetary_connectome(n: usize, p: f64, seed: u64) -> Vec<(usize, usize)> {
    let mut rng = StdRng::seed_from_u64(seed);
    let mut present = vec![vec![false; n]; n];
    let mut edges = Vec::new();

    for i in 0..n {
        let j = (i + 1) % n;
        if i != j && !present[i][j] {
            present[i][j] = true;
            present[j][i] = true;
            edges.push((i, j));
        }
    }

    let prob = p.clamp(0.0, 1.0);
    for i in 0..n {
        for j in (i + 1)..n {
            if present[i][j] {
                continue;
            }
            if rng.gen::<f64>() < prob {
                present[i][j] = true;
                present[j][i] = true;
                edges.push((i, j));
            }
        }
    }

    edges
}

fn build_diagnostic_partition(
    n: usize,
    edges: &[(usize, usize)],
    resolution: usize,
) -> Vec<Vec<usize>> {
    let mut degrees = vec![0usize; n];
    for &(u, v) in edges {
        if u < n && v < n && u != v {
            degrees[u] += 1;
            degrees[v] += 1;
        }
    }

    let mut vertices: Vec<usize> = (0..n).collect();
    vertices.sort_unstable_by_key(|&v| degrees[v]);

    let k = resolution.clamp(1, n.max(1));
    let mut parts = vec![Vec::new(); k];
    for (idx, v) in vertices.into_iter().enumerate() {
        parts[idx % k].push(v);
    }
    parts
}

fn compute_holonomy(parts: &[Vec<usize>], edges: &[(usize, usize)]) -> HolonomyTelemetry {
    let k = parts.len();
    if k < 2 {
        return HolonomyTelemetry::default();
    }

    let max_vertex = parts
        .iter()
        .flatten()
        .copied()
        .chain(edges.iter().flat_map(|&(u, v)| [u, v]))
        .max()
        .map(|m| m + 1)
        .unwrap_or(0);
    if max_vertex == 0 {
        return HolonomyTelemetry::default();
    }

    let mut part_of = vec![usize::MAX; max_vertex];
    for (idx, part) in parts.iter().enumerate() {
        for &v in part {
            if v < max_vertex {
                part_of[v] = idx;
            }
        }
    }

    let mut pair_counts = vec![vec![0usize; k]; k];
    for &(u, v) in edges {
        if u >= max_vertex || v >= max_vertex || u == v {
            continue;
        }
        let pu = part_of[u];
        let pv = part_of[v];
        if pu == usize::MAX || pv == usize::MAX || pu == pv {
            continue;
        }
        let (a, b) = if pu < pv { (pu, pv) } else { (pv, pu) };
        pair_counts[a][b] += 1;
    }

    let mut density = vec![vec![0.0f64; k]; k];
    let mut quotient_adj = vec![Vec::<(usize, f64)>::new(); k];
    let mut quotient_edges = 0usize;
    for i in 0..k {
        for j in (i + 1)..k {
            let capacity = (parts[i].len() * parts[j].len()) as f64;
            if capacity <= 0.0 {
                continue;
            }
            let d = pair_counts[i][j] as f64 / capacity;
            density[i][j] = d;
            density[j][i] = d;
            if d > 0.0 {
                quotient_adj[i].push((j, d));
                quotient_adj[j].push((i, d));
                quotient_edges += 1;
            }
        }
    }

    let mut parent = vec![None; k];
    let mut depth = vec![0usize; k];
    let mut visited = vec![false; k];
    let mut tree_edges = HashSet::<(usize, usize)>::new();

    for root in 0..k {
        if visited[root] {
            continue;
        }
        visited[root] = true;
        let mut queue = VecDeque::new();
        queue.push_back(root);
        while let Some(u) = queue.pop_front() {
            for &(v, _) in &quotient_adj[u] {
                if !visited[v] {
                    visited[v] = true;
                    parent[v] = Some(u);
                    depth[v] = depth[u] + 1;
                    tree_edges.insert((u.min(v), u.max(v)));
                    queue.push_back(v);
                }
            }
        }
    }

    let two_pi = std::f64::consts::TAU;
    let potential = |a: usize, b: usize| -> f64 {
        if a == b {
            return 0.0;
        }
        let d = density[a][b];
        if a < b {
            two_pi * d
        } else {
            -two_pi * d
        }
    };

    let mut fundamental_cycles = 0usize;
    let mut sum_defect = 0.0f64;
    let mut max_defect = 0.0f64;
    let mut sum_raw_phase = 0.0f64;
    let mut max_raw_phase = 0.0f64;

    for (u, neighbors) in quotient_adj.iter().enumerate() {
        for &(v, _) in neighbors {
            if u >= v || tree_edges.contains(&(u, v)) {
                continue;
            }

            let mut path_u = Vec::new();
            let mut path_v = Vec::new();
            let (mut a, mut b) = (u, v);
            while depth[a] > depth[b] {
                path_u.push(a);
                match parent[a] {
                    Some(p) => a = p,
                    None => break,
                }
            }
            while depth[b] > depth[a] {
                path_v.push(b);
                match parent[b] {
                    Some(p) => b = p,
                    None => break,
                }
            }
            while a != b {
                path_u.push(a);
                path_v.push(b);
                match (parent[a], parent[b]) {
                    (Some(pa), Some(pb)) => {
                        a = pa;
                        b = pb;
                    }
                    _ => break,
                }
            }
            path_u.push(a);
            path_v.reverse();
            path_u.extend(path_v);

            let mut phase = 0.0;
            for window in path_u.windows(2) {
                phase += potential(window[0], window[1]);
            }
            phase += potential(v, u);

            let wrapped = phase.rem_euclid(two_pi);
            let defect = wrapped.min(two_pi - wrapped) / std::f64::consts::PI;
            sum_defect += defect;
            if defect > max_defect {
                max_defect = defect;
            }
            let raw_phase = phase.abs();
            sum_raw_phase += raw_phase;
            if raw_phase > max_raw_phase {
                max_raw_phase = raw_phase;
            }
            fundamental_cycles += 1;
        }
    }

    let mean_defect = if fundamental_cycles > 0 {
        sum_defect / fundamental_cycles as f64
    } else {
        0.0
    };
    let mean_raw_phase = if fundamental_cycles > 0 {
        sum_raw_phase / fundamental_cycles as f64
    } else {
        0.0
    };

    HolonomyTelemetry {
        partition_parts: k,
        quotient_nodes: parts.iter().filter(|p| !p.is_empty()).count(),
        quotient_edges,
        fundamental_cycles,
        mean_abs_holonomy_radians: mean_defect * std::f64::consts::PI,
        max_abs_holonomy_radians: max_defect * std::f64::consts::PI,
        normalized_curvature: mean_defect,
        mean_raw_cycle_phase_radians: mean_raw_phase,
        max_raw_cycle_phase_radians: max_raw_phase,
    }
}

fn signal_flow_morphism(
    id: &str,
    source: &str,
    target: &str,
    kind: &'static str,
) -> SignalFlowMorphism {
    SignalFlowMorphism {
        id: id.to_string(),
        source: source.to_string(),
        target: target.to_string(),
        kind,
    }
}

fn build_signal_flow_category(
    loops_run: usize,
    converged: bool,
    trajectory: &[Value],
) -> SignalFlowCategory {
    let stages = [
        "stage:erdos_renyi_genesis",
        "stage:flora_steiner_mycelium",
        "stage:szemeredi_cortical_map",
        "stage:ramsey_consensus_nuclei",
        "stage:moser_tardos_homeostasis",
        "stage:fauna_avatar_consensus",
    ];
    let controls = [
        "control:density_pid",
        "control:epsilon_pid",
        "control:clique_controller",
    ];
    let errors = [
        "error:giant_fraction",
        "error:compression_ratio",
        "error:holonomy_curvature",
        "error:clique_witness",
    ];
    let actuators = [
        "actuator:synaptic_density",
        "actuator:regularity_epsilon",
        "actuator:consensus_clique_k",
    ];

    let mut objects = Vec::new();
    for id in stages
        .iter()
        .chain(controls.iter())
        .chain(errors.iter())
        .chain(actuators.iter())
    {
        let kind = if id.starts_with("stage:") {
            "stage"
        } else if id.starts_with("control:") {
            "control"
        } else if id.starts_with("error:") {
            "error"
        } else {
            "actuator"
        };
        objects.push(SignalFlowObject {
            id: id.to_string(),
            kind,
        });
    }

    let morphisms = vec![
        signal_flow_morphism("data:er_to_steiner", stages[0], stages[1], "data"),
        signal_flow_morphism("data:steiner_to_szemeredi", stages[1], stages[2], "data"),
        signal_flow_morphism("data:szemeredi_to_ramsey", stages[2], stages[3], "data"),
        signal_flow_morphism("data:ramsey_to_moser", stages[3], stages[4], "data"),
        signal_flow_morphism("data:moser_to_fauna", stages[4], stages[5], "data"),
        signal_flow_morphism("control:m1_to_giant", stages[0], errors[0], "control"),
        signal_flow_morphism(
            "control:steiner_to_density",
            stages[1],
            controls[0],
            "control",
        ),
        signal_flow_morphism("control:m2_to_compression", stages[2], errors[1], "control"),
        signal_flow_morphism("control:m3_to_clique", stages[3], errors[3], "control"),
        signal_flow_morphism(
            "control:holonomy_to_density",
            errors[2],
            controls[0],
            "control",
        ),
        signal_flow_morphism(
            "control:giant_to_density_pid",
            errors[0],
            controls[0],
            "control",
        ),
        signal_flow_morphism(
            "control:compression_to_epsilon_pid",
            errors[1],
            controls[1],
            "control",
        ),
        signal_flow_morphism(
            "control:clique_to_clique_controller",
            errors[3],
            controls[2],
            "control",
        ),
        signal_flow_morphism(
            "control:density_pid_to_actuator",
            controls[0],
            actuators[0],
            "control",
        ),
        signal_flow_morphism(
            "control:epsilon_pid_to_actuator",
            controls[1],
            actuators[1],
            "control",
        ),
        signal_flow_morphism(
            "control:clique_controller_to_actuator",
            controls[2],
            actuators[2],
            "control",
        ),
        signal_flow_morphism(
            "feedback:density_to_genesis",
            actuators[0],
            stages[0],
            "feedback",
        ),
        signal_flow_morphism(
            "feedback:epsilon_to_szemeredi",
            actuators[1],
            stages[2],
            "feedback",
        ),
        signal_flow_morphism(
            "feedback:clique_to_ramsey",
            actuators[2],
            stages[3],
            "feedback",
        ),
        signal_flow_morphism("trace:fauna_to_genesis", stages[5], stages[0], "trace"),
    ];

    let object_map = vec![
        (stages[0].to_string(), errors[0].to_string()),
        (stages[1].to_string(), controls[0].to_string()),
        (stages[2].to_string(), errors[1].to_string()),
        (stages[3].to_string(), errors[3].to_string()),
        (stages[4].to_string(), controls[2].to_string()),
        (stages[5].to_string(), actuators[0].to_string()),
    ];

    let morphism_map = vec![
        (
            "data:er_to_steiner".to_string(),
            "control:m1_to_giant".to_string(),
        ),
        (
            "data:steiner_to_szemeredi".to_string(),
            "control:steiner_to_density".to_string(),
        ),
        (
            "data:szemeredi_to_ramsey".to_string(),
            "control:m2_to_compression".to_string(),
        ),
        (
            "data:ramsey_to_moser".to_string(),
            "control:m3_to_clique".to_string(),
        ),
        (
            "data:moser_to_fauna".to_string(),
            "control:clique_controller_to_actuator".to_string(),
        ),
        (
            "feedback:density_to_genesis".to_string(),
            "trace:fauna_to_genesis".to_string(),
        ),
    ];

    SignalFlowCategory {
        objects,
        morphisms,
        functor: SignalFlowFunctor {
            name: "GaiaPipelineToControlFunctor",
            object_map,
            morphism_map,
        },
        flow: trajectory.to_vec(),
        trace: FeedbackTrace {
            loop_stage: stages[5].to_string(),
            input: actuators[0].to_string(),
            output: stages[0].to_string(),
            iterations: loops_run,
            converged,
            trace_morphism: "trace:fauna_to_genesis".to_string(),
        },
    }
}

pub fn run_gaia_omni_mode(
    model_cache: &LlamaModelCache,
    params: GaiaOmniParams,
    emitter: &McpProgressEmitter,
) -> Result<Value, String> {
    let n = params.biosphere_nodes.clamp(8, 512);
    let mut density = params.synaptic_density.clamp(0.01, 0.99);
    let mut epsilon = params.regularity_epsilon.clamp(0.02, 0.49);
    let mut clique_k = params.consensus_clique_k.clamp(2, 32).min(n);
    let max_loops = params.max_feedback_loops.clamp(1, 8);
    let directive = if params.omni_directive.trim().is_empty() {
        default_omni_directive()
    } else {
        params.omni_directive.clone()
    };

    let mut pid_density = PidController::new(params.pid_kp, params.pid_ki, params.pid_kd);
    let mut pid_epsilon = PidController::new(params.pid_kp, params.pid_ki, params.pid_kd);

    let mut last_uz = Value::Null;
    let mut last_holonomy = HolonomyTelemetry::default();
    let mut trajectory: Vec<Value> = Vec::new();
    let mut loops_run = 0usize;
    let mut converged = false;

    let mut applied_density = density;
    let mut applied_epsilon = epsilon;
    let mut applied_clique_k = clique_k;

    for loop_idx in 0..max_loops {
        loops_run = loop_idx + 1;
        applied_density = density;
        applied_epsilon = epsilon;
        applied_clique_k = clique_k;

        let edges = synthesize_planetary_connectome(n, density, params.seed);

        let uz_params = UltraZenoParams {
            num_vertices: n,
            edges: edges.clone(),
            terminals: Vec::new(),
            initial_epsilon: epsilon,
            max_zeno_steps: 3,
            ib_beta: 1.25,
            system2_entropy_threshold: 1.5,
            target_clique_k: clique_k,
            model_path: None,
        };

        let uz = run_ultra_zeno_engine(model_cache, uz_params, emitter)?;
        let holonomy_resolution = (n / 16).clamp(4, 16);
        let holonomy_parts = build_diagnostic_partition(n, &edges, holonomy_resolution);
        let holonomy = compute_holonomy(&holonomy_parts, &edges);

        let giant_fraction = uz["m1_system1_sensory"]["erdos_renyi_percolation"]
            ["giant_component_fraction"]
            .as_f64()
            .unwrap_or(0.0);
        let compression_ratio = uz["m2_ultra_zeno_compression"]["mdl_compression_ratio"]
            .as_f64()
            .unwrap_or(1.0);
        let clique_witness_size = uz["m3_system2_invariants"]["exact_micro_ramsey"]
            ["largest_clique_witness"]
            .as_array()
            .map(|a| a.len())
            .unwrap_or(0);
        let clique_found = uz["m3_system2_invariants"]["exact_micro_ramsey"]["k_clique_found"]
            .as_bool()
            .unwrap_or(false);

        let density_primary_error = params.giant_component_target - giant_fraction;
        let compression_error = params.compression_target - compression_ratio;
        let holonomy_error = holonomy.normalized_curvature - params.holonomy_tolerance;
        let density_error = density_primary_error + 0.5 * holonomy_error;
        let epsilon_error = compression_error;

        let density_output = pid_density.update(density_error, 1.0);
        let epsilon_output = pid_epsilon.update(epsilon_error, 1.0);

        let next_density = (density + density_output * 0.05).clamp(0.01, 0.99);
        let next_epsilon = (epsilon + epsilon_output * 0.05).clamp(0.02, 0.49);
        let next_clique_k = if !clique_found && clique_k > 2 {
            clique_k - 1
        } else if clique_witness_size > clique_k + 1 && clique_k < 32 {
            clique_k + 1
        } else {
            clique_k
        };

        let loop_converged = density_primary_error.abs() < params.feedback_tolerance
            && compression_error.abs() < params.feedback_tolerance
            && holonomy.normalized_curvature <= params.holonomy_tolerance * 1.05;

        let loop_shannon_entropy = uz["m1_system1_sensory"]["shannon_degree_entropy_bits"]
            .as_f64()
            .unwrap_or(0.0);
        let loop_kl_surprise = uz["m1_system1_sensory"]["kl_divergence_from_er_prior_bits"]
            .as_f64()
            .unwrap_or(0.0);
        let loop_mutual_information = uz["m2_ultra_zeno_compression"]["mutual_information_bits"]
            .as_f64()
            .unwrap_or(0.0);
        let loop_max_entropy = (n as f64).log2().max(1.0);
        let loop_intuition = 1.0 / (1.0 + loop_shannon_entropy);
        let loop_creativity = (loop_mutual_information + loop_kl_surprise)
            / (1.0 + loop_mutual_information + loop_kl_surprise);
        let loop_high_entropy_penalty = (loop_shannon_entropy / loop_max_entropy).clamp(0.0, 1.0);
        let loop_creative_objective = loop_intuition + loop_creativity - loop_high_entropy_penalty;

        trajectory.push(json!({
            "loop": loop_idx,
            "applied": {
                "synaptic_density": density,
                "regularity_epsilon": epsilon,
                "consensus_clique_k": clique_k
            },
            "telemetry": {
                "giant_component_fraction": giant_fraction,
                "mdl_compression_ratio": compression_ratio,
                "holonomy_normalized_curvature": holonomy.normalized_curvature,
                "clique_witness_size": clique_witness_size,
                "k_clique_found": clique_found,
                "intuition": loop_intuition,
                "creativity": loop_creativity,
                "high_entropy_penalty": loop_high_entropy_penalty,
                "creative_objective": loop_creative_objective
            },
            "errors": {
                "density_primary": density_primary_error,
                "density_combined": density_error,
                "compression": compression_error,
                "holonomy": holonomy_error,
                "epsilon": epsilon_error
            },
            "pid_outputs": {
                "density": density_output,
                "epsilon": epsilon_output
            },
            "next": {
                "synaptic_density": next_density,
                "regularity_epsilon": next_epsilon,
                "consensus_clique_k": next_clique_k
            },
            "converged": loop_converged
        }));

        last_uz = uz;
        last_holonomy = holonomy;
        density = next_density;
        epsilon = next_epsilon;
        clique_k = next_clique_k;

        if loop_converged {
            converged = true;
            break;
        }
    }

    let m1_structural_compression = json!({
        "shannon_degree_entropy_bits": last_uz["m1_system1_sensory"]["shannon_degree_entropy_bits"],
        "kl_divergence_from_er_prior_bits": last_uz["m1_system1_sensory"]["kl_divergence_from_er_prior_bits"],
        "mdl_compression_ratio": last_uz["m2_ultra_zeno_compression"]["mdl_compression_ratio"],
        "mutual_information_bits": last_uz["m2_ultra_zeno_compression"]["mutual_information_bits"],
        "information_bottleneck_lagrangian": last_uz["m2_ultra_zeno_compression"]["information_bottleneck_lagrangian"],
        "optimal_dyadic_epsilon": last_uz["m2_ultra_zeno_compression"]["optimal_dyadic_epsilon"],
        "cognitive_mode": last_uz["cognitive_mode"]
    });

    let final_shannon_entropy = last_uz["m1_system1_sensory"]["shannon_degree_entropy_bits"]
        .as_f64()
        .unwrap_or(0.0);
    let final_kl_surprise = last_uz["m1_system1_sensory"]["kl_divergence_from_er_prior_bits"]
        .as_f64()
        .unwrap_or(0.0);
    let final_mutual_information = last_uz["m2_ultra_zeno_compression"]["mutual_information_bits"]
        .as_f64()
        .unwrap_or(0.0);
    let max_entropy_bits = (n as f64).log2().max(1.0);
    let intuition = 1.0 / (1.0 + final_shannon_entropy);
    let creativity = (final_mutual_information + final_kl_surprise)
        / (1.0 + final_mutual_information + final_kl_surprise);
    let high_entropy_penalty = (final_shannon_entropy / max_entropy_bits).clamp(0.0, 1.0);
    let creative_objective_score = intuition + creativity - high_entropy_penalty;
    let creative_objective = json!({
        "formula": "intuition + creativity - high_entropy",
        "intuition": intuition,
        "creativity": creativity,
        "high_entropy_penalty": high_entropy_penalty,
        "score": creative_objective_score,
        "components": {
            "shannon_degree_entropy_bits": final_shannon_entropy,
            "kl_divergence_from_er_prior_bits": final_kl_surprise,
            "mutual_information_bits": final_mutual_information,
            "max_entropy_bits": max_entropy_bits
        }
    });

    let signal_flow_category = build_signal_flow_category(loops_run, converged, &trajectory);

    let policy = &params.concurrency_policy;
    let mut policy_violations = Vec::new();
    if policy.human_authority != default_human_authority()
        || params.omni_directive.trim().is_empty()
    {
        policy_violations.push("human_authority_not_preserved".to_string());
    }
    if policy.initiation != default_initiation_policy() {
        policy_violations.push("initiation_not_explicit".to_string());
    }
    if policy.switching != default_switch_policy() || loops_run > params.max_feedback_loops {
        policy_violations.push("switching_not_deliberate".to_string());
    }
    if policy.relevance != default_relevance_policy() || params.omni_directive.trim().is_empty() {
        policy_violations.push("relevance_not_bounded".to_string());
    }
    if policy.resource_competition != default_resource_policy()
        || n > 512
        || params.max_feedback_loops > 16
    {
        policy_violations.push("resource_competition_unbounded".to_string());
    }
    if !policy.preserve_unfinished_work || trajectory.is_empty() {
        policy_violations.push("unfinished_work_not_preserved".to_string());
    }
    if policy.unsolicited_demands {
        policy_violations.push("unsolicited_demands_enabled".to_string());
    }
    if !policy.protected_concentration {
        policy_violations.push("concentration_not_protected".to_string());
    }
    let enclosure_risk = match policy_violations.len() {
        0 => "low",
        1..=2 => "medium",
        _ => "high",
    };
    let concurrency_policy_checks = json!({
        "human_authority_preserved": !policy_violations.contains(&"human_authority_not_preserved".to_string()),
        "explicit_initiation": !policy_violations.contains(&"initiation_not_explicit".to_string()),
        "deliberate_switching": !policy_violations.contains(&"switching_not_deliberate".to_string()),
        "relevance_bounded": !policy_violations.contains(&"relevance_not_bounded".to_string()),
        "resource_bounded": !policy_violations.contains(&"resource_competition_unbounded".to_string()),
        "unfinished_work_preserved": !policy_violations.contains(&"unfinished_work_not_preserved".to_string()),
        "unsolicited_demands_suppressed": !policy_violations.contains(&"unsolicited_demands_enabled".to_string()),
        "concentration_protected": !policy_violations.contains(&"concentration_not_protected".to_string()),
        "switching_events": loops_run,
        "preserved_state": {
            "trajectory_loops": trajectory.len(),
            "signal_flow_objects": signal_flow_category.objects.len(),
            "signal_flow_morphisms": signal_flow_category.morphisms.len(),
            "trace_present": !signal_flow_category.trace.trace_morphism.is_empty()
        },
        "enclosure_risk": enclosure_risk,
        "violations": policy_violations
    });

    let consensus_schema = json!({
        "type": "object",
        "properties": {
            "planetary_identity": { "type": "string", "enum": ["I/We/Gaia"] },
            "homeostatic_verdict": {
                "type": "string",
                "enum": ["equilibrium", "adaptive_correction", "critical_transition"]
            },
            "holonomy_defect": { "type": "number" },
            "consensus_clique": { "type": "array", "items": { "type": "integer" } },
            "directive_echo": { "type": "string" }
        },
        "required": [
            "planetary_identity",
            "homeostatic_verdict",
            "holonomy_defect",
            "consensus_clique",
            "directive_echo"
        ]
    });

    let homeostatic_verdict = if converged {
        "equilibrium"
    } else if last_holonomy.normalized_curvature > params.holonomy_tolerance {
        "critical_transition"
    } else {
        "adaptive_correction"
    };

    let symbolic_consensus = json!({
        "mode": "symbolic_gaia_consensus",
        "planetary_identity": "I/We/Gaia",
        "homeostatic_verdict": homeostatic_verdict,
        "holonomy_defect": last_holonomy.normalized_curvature,
        "consensus_clique": last_uz["m3_system2_invariants"]["exact_micro_ramsey"]["largest_clique_witness"],
        "directive_echo": directive,
        "consensus_schema": consensus_schema,
        "consensus_gbnf": GAIA_CONSENSUS_GBNF
    });

    let fauna_avatar_consensus = if let Some(model_path) = params.model_path.as_ref() {
        let prompt = format!(
            "Gaia Omni planetary superorganism directive: {directive}\n\
             M1 structural compression ratio: {:.3}x (holonomy curvature: {:.3})\n\
             Ramsey consensus clique: {}\n\
             Homeostatic verdict: {homeostatic_verdict}\n\
             Synthesize the Gaia avatar consensus JSON:",
            last_uz["m2_ultra_zeno_compression"]["mdl_compression_ratio"]
                .as_f64()
                .unwrap_or(1.0),
            last_holonomy.normalized_curvature,
            last_uz["m3_system2_invariants"]["exact_micro_ramsey"]["largest_clique_witness"]
        );

        match model_cache.submit_continuous_batch_job(
            LlamaRustParams {
                model_path: model_path.clone(),
                draft_model_path: None,
                speculative_depth: 4,
                speculative_tree_width: 2,
                grammar: Some(GAIA_CONSENSUS_GBNF.to_string()),
                json_schema: Some(consensus_schema.clone()),
                prompt,
                n_ctx: 4096,
                n_gpu_layers: 99,
                n_batch: 2048,
                temperature: 0.2,
                max_tokens: 256,
            },
            emitter.clone(),
        ) {
            Ok(llm) => json!({
                "mode": "llama_rust_gaia_consensus",
                "llm": llm,
                "symbolic": symbolic_consensus
            }),
            Err(error) => json!({
                "mode": "symbolic_gaia_consensus_fallback",
                "llm_error": error,
                "symbolic": symbolic_consensus
            }),
        }
    } else {
        symbolic_consensus
    };

    let report = json!({
        "status": "gaia_omni_avatar_complete",
        "architecture": "GAIA_OMNI_6_STAGE_CONTROL_HOLONOMY_FEEDBACK",
        "pipeline": "ultraOmniZeno_E2E_IN_kompress_super_ultra_DO_ULTRABACKYARDLOOP",
        "directive": params.omni_directive,
        "concurrency_policy": params.concurrency_policy,
        "concurrency_policy_checks": concurrency_policy_checks,
        "biosphere_nodes": n,
        "feedback_loops": loops_run,
        "converged": converged,
        "final_control": {
            "applied": {
                "synaptic_density": applied_density,
                "regularity_epsilon": applied_epsilon,
                "consensus_clique_k": applied_clique_k
            },
            "next_suggested": {
                "synaptic_density": density,
                "regularity_epsilon": epsilon,
                "consensus_clique_k": clique_k
            },
            "targets": {
                "giant_component_fraction": params.giant_component_target,
                "mdl_compression_ratio": params.compression_target,
                "holonomy_tolerance": params.holonomy_tolerance,
                "feedback_tolerance": params.feedback_tolerance
            },
            "pid_density": pid_density.snapshot(),
            "pid_epsilon": pid_epsilon.snapshot(),
            "loop_trajectory": trajectory
        },
        "m1_structural_compression": m1_structural_compression,
        "creative_objective": creative_objective,
        "signal_flow_category": signal_flow_category,
        "erdos_renyi_genesis": last_uz["m1_system1_sensory"]["erdos_renyi_percolation"].clone(),
        "flora_steiner_mycelium": last_uz["m2_ultra_zeno_compression"]["steiner_backbone_summary"].clone(),
        "szemeredi_cortical_map": last_uz["m2_ultra_zeno_compression"].clone(),
        "ramsey_consensus_nuclei": last_uz["m3_system2_invariants"]["exact_micro_ramsey"].clone(),
        "moser_tardos_homeostasis": last_uz["m3_system2_invariants"]["moser_tardos_homeostasis"].clone(),
        "holonomy": last_holonomy,
        "fauna_avatar_consensus": fauna_avatar_consensus,
        "stage_chain": [
            "erdos_renyi_genesis",
            "flora_steiner_mycelium",
            "szemeredi_cortical_map",
            "ramsey_consensus_nuclei",
            "moser_tardos_homeostasis",
            "fauna_avatar_consensus"
        ],
        "ultra_zeno_report": last_uz
    });

    Ok(report)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_params() -> GaiaOmniParams {
        GaiaOmniParams {
            biosphere_nodes: 32,
            synaptic_density: 0.18,
            regularity_epsilon: 0.28,
            consensus_clique_k: 3,
            model_path: None,
            omni_directive: "Sustain the biosphere.".to_string(),
            seed: 42,
            max_feedback_loops: 2,
            holonomy_tolerance: 0.25,
            compression_target: 2.0,
            giant_component_target: 0.75,
            pid_kp: 0.6,
            pid_ki: 0.15,
            pid_kd: 0.08,
            feedback_tolerance: 0.1,
            concurrency_policy: ConcurrencyPolicy::default(),
        }
    }

    #[test]
    fn gaia_omni_control_holonomy_feedback_runs() {
        let cache = LlamaModelCache::new(1);
        let emitter = McpProgressEmitter::default();
        let report = run_gaia_omni_mode(&cache, test_params(), &emitter).unwrap();

        assert_eq!(report["status"], "gaia_omni_avatar_complete");
        assert_eq!(
            report["architecture"],
            "GAIA_OMNI_6_STAGE_CONTROL_HOLONOMY_FEEDBACK"
        );
        assert!(report["feedback_loops"].as_u64().unwrap() >= 1);
        assert!(
            report["m1_structural_compression"]["mdl_compression_ratio"]
                .as_f64()
                .unwrap()
                > 0.0
        );
        assert!(report["holonomy"]["partition_parts"].as_u64().unwrap() >= 4);
        assert!(report["holonomy"]["quotient_nodes"].as_u64().unwrap() >= 2);
        assert!(report["holonomy"]["fundamental_cycles"].as_u64().unwrap() > 0);
        assert_eq!(
            report["fauna_avatar_consensus"]["planetary_identity"],
            "I/We/Gaia"
        );
        assert!(report["final_control"]["pid_density"]["last_output"].is_number());
        assert_eq!(
            report["pipeline"],
            "ultraOmniZeno_E2E_IN_kompress_super_ultra_DO_ULTRABACKYARDLOOP"
        );
        assert!(report["creative_objective"]["score"].is_number());
        assert!(report["creative_objective"]["intuition"].as_f64().unwrap() > 0.0);
        assert!(
            report["creative_objective"]["high_entropy_penalty"]
                .as_f64()
                .unwrap()
                >= 0.0
        );
        assert_eq!(
            report["signal_flow_category"]["functor"]["name"],
            "GaiaPipelineToControlFunctor"
        );
        assert!(
            report["signal_flow_category"]["objects"]
                .as_array()
                .unwrap()
                .len()
                >= 10
        );
        assert!(
            report["signal_flow_category"]["morphisms"]
                .as_array()
                .unwrap()
                .len()
                >= 10
        );
        assert_eq!(
            report["concurrency_policy"]["human_authority"],
            "human_purpose_authority"
        );
        assert_eq!(report["concurrency_policy_checks"]["enclosure_risk"], "low");
        assert!(report["concurrency_policy_checks"]["violations"]
            .as_array()
            .unwrap()
            .is_empty());
    }
}
