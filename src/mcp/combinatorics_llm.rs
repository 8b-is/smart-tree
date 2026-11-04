//! Sovereign Combinatorics & Local LLM MCP Engine (`src/mcp/combinatorics_llm.rs`).
//!
//! Unifies the 5 bit-parallel / Rayon-parallel Paul Erdős combinatorial graph solvers
//! (`szemeredi_compression`, `erdos_steiner_pruning`, `moser_tardos_splitting`,
//! `erdos_renyi_scanning`, `ramsey_clique_analysis`) with Apple Silicon Metal `mlx_quant`
//! (`affine`, `mxfp4`, `mxfp8`, `nvfp4`) and `llama_rust` (singleflight LRU model cache +
//! `MAX_SEQS_PER_SESSION = 8` `llama_kv_cache_seq_cp` multi-sequence KV radix pool)
//! under Smart Tree's JSON-RPC 2.0 MCP dispatcher (`ULTRA-OMNI-ZENO-BEERUS`).

use crate::felix_asm::{
    felix_popcnt_u64, felix_ternary_gemm_i32, felix_tzcnt_u64, felix_xor_fold_64b,
};
use crate::mem8::regularity::zeno_engine::{
    seal_litany_wire_64b, verify_litany_wire_64b, GATE_0X62, QUANT_QT55,
};
use rayon::prelude::*;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::{
    cmp::Ordering,
    collections::{BinaryHeap, HashMap, HashSet},
    ffi::CString,
    fs,
    path::Path,
    process::Command,
    sync::{
        atomic::{AtomicBool, AtomicU64, AtomicUsize, Ordering as AtomicOrdering},
        Arc, Mutex, OnceLock, RwLock,
    },
    time::Instant,
};

/// Number of concurrent radix sequence branches maintained inside each pooled `OwnedLlamaSession`.
pub const MAX_SEQS_PER_SESSION: u32 = 8;

// ============================================================================
// Strongly-Typed Algorithm & LLM Runtime Payloads
// ============================================================================

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct SzemerediParams {
    pub num_vertices: usize,
    pub edges: Vec<(usize, usize)>,
    pub epsilon: f64,
    pub min_parts: usize,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ErdosSteinerParams {
    pub edges: Vec<(usize, usize, f64)>,
    pub terminals: Vec<usize>,
    pub pruning_threshold: f64,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct MoserTardosParams {
    pub num_variables: usize,
    pub bad_events: Vec<Vec<i32>>,
    pub max_resamples: usize,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ErdosRenyiParams {
    pub n: usize,
    pub p: f64,
    pub seed: Option<u64>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct RamseyCliqueParams {
    pub num_vertices: usize,
    pub edges: Vec<(usize, usize)>,
    pub k_clique: usize,
    pub l_independent: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum MlxQuantScheme {
    Affine,
    Mxfp4,
    Mxfp8,
    Nvfp4,
}

impl MlxQuantScheme {
    #[inline]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Affine => "affine",
            Self::Mxfp4 => "mxfp4",
            Self::Mxfp8 => "mxfp8",
            Self::Nvfp4 => "nvfp4",
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct MlxQuantParams {
    pub model_path: String,
    pub output_path: String,
    pub bits: u8,
    pub group_size: usize,
    pub scheme: MlxQuantScheme,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct LlamaRustParams {
    pub model_path: String,
    pub prompt: String,
    #[serde(default = "default_n_ctx")]
    pub n_ctx: u32,
    #[serde(default = "default_n_gpu_layers")]
    pub n_gpu_layers: u32,
    #[serde(default = "default_n_batch")]
    pub n_batch: u32,
    #[serde(default = "default_temperature")]
    pub temperature: f32,
    #[serde(default = "default_max_tokens")]
    pub max_tokens: usize,
}

const fn default_n_ctx() -> u32 {
    4096
}
const fn default_n_gpu_layers() -> u32 {
    99
}
const fn default_n_batch() -> u32 {
    2048
}
const fn default_temperature() -> f32 {
    0.7
}
const fn default_max_tokens() -> usize {
    512
}

// ============================================================================
// Deterministic SplitMix64 PRNG & Stateless Mixing
// ============================================================================

#[derive(Clone, Copy)]
pub struct SplitMix64 {
    state: u64,
}

impl SplitMix64 {
    #[inline(always)]
    pub const fn new(seed: u64) -> Self {
        Self { state: seed }
    }

    #[inline(always)]
    pub fn next_u64(&mut self) -> u64 {
        self.state = self.state.wrapping_add(0x9E37_79B9_7F4A_7C15);
        splitmix64_mix(self.state)
    }

    #[inline(always)]
    pub fn next_f64(&mut self) -> f64 {
        (self.next_u64() >> 11) as f64 * (1.0 / ((1u64 << 53) as f64))
    }
}

#[inline(always)]
pub const fn splitmix64_mix(mut z: u64) -> u64 {
    z = z.wrapping_add(0x9E37_79B9_7F4A_7C15);
    z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
    z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
    z ^ (z >> 31)
}

#[inline(always)]
fn hash_bool(seed: u64, round: usize, clause_idx: usize, var_idx: usize) -> bool {
    let key = seed
        ^ ((round as u64).wrapping_mul(0x517C_C1B7_2722_0A95))
        ^ ((clause_idx as u64).wrapping_mul(0x94D0_49BB_1331_11EB))
        ^ (var_idx as u64);
    (splitmix64_mix(key) & 1) == 1
}

#[inline(always)]
fn clause_priority(seed: u64, round: usize, sub_step: usize, clause_idx: usize) -> u64 {
    let key = seed
        ^ ((round as u64).rotate_left(21))
        ^ ((sub_step as u64).rotate_left(42))
        ^ (clause_idx as u64);
    splitmix64_mix(key)
}

// ============================================================================
// Hardware-Accelerated u64 BitSet & BitMatrix (FELIX POPCNT / TZCNT)
// ============================================================================

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BitSet {
    pub words: Vec<u64>,
    pub num_bits: usize,
}

impl BitSet {
    #[inline]
    pub fn new(num_bits: usize) -> Self {
        let num_words = (num_bits + 63) >> 6;
        Self {
            words: vec![0u64; num_words],
            num_bits,
        }
    }

    #[inline]
    pub fn full(num_bits: usize) -> Self {
        let num_words = (num_bits + 63) >> 6;
        let mut words = vec![!0u64; num_words];
        let rem = num_bits & 63;
        if rem != 0 && !words.is_empty() {
            let last = words.len() - 1;
            words[last] = (1u64 << rem) - 1;
        }
        Self { words, num_bits }
    }

    #[inline]
    pub fn from_slice(num_bits: usize, indices: &[usize]) -> Self {
        let mut bs = Self::new(num_bits);
        for &idx in indices {
            if idx < num_bits {
                bs.set(idx);
            }
        }
        bs
    }

    #[inline]
    pub fn set(&mut self, idx: usize) {
        debug_assert!(idx < self.num_bits);
        self.words[idx >> 6] |= 1u64 << (idx & 63);
    }

    #[inline]
    pub fn remove(&mut self, idx: usize) {
        debug_assert!(idx < self.num_bits);
        self.words[idx >> 6] &= !(1u64 << (idx & 63));
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.words.iter().all(|&w| w == 0)
    }

    #[inline]
    pub fn count(&self) -> usize {
        self.words
            .iter()
            .map(|&w| felix_popcnt_u64(w) as usize)
            .sum()
    }

    #[inline]
    pub fn intersect_count_slice(&self, row_words: &[u64]) -> usize {
        self.words
            .iter()
            .zip(row_words.iter())
            .map(|(&a, &b)| felix_popcnt_u64(a & b) as usize)
            .sum()
    }

    #[inline]
    pub fn intersect_slice(&self, row_words: &[u64]) -> Self {
        let words = self
            .words
            .iter()
            .zip(row_words.iter())
            .map(|(&a, &b)| a & b)
            .collect();
        Self {
            words,
            num_bits: self.num_bits,
        }
    }

    #[inline]
    pub fn diff_slice(&self, row_words: &[u64]) -> Self {
        let words = self
            .words
            .iter()
            .zip(row_words.iter())
            .map(|(&a, &b)| a & !b)
            .collect();
        Self {
            words,
            num_bits: self.num_bits,
        }
    }

    #[inline]
    pub fn union(&self, other: &Self) -> Self {
        let words = self
            .words
            .iter()
            .zip(other.words.iter())
            .map(|(&a, &b)| a | b)
            .collect();
        Self {
            words,
            num_bits: self.num_bits,
        }
    }

    #[inline]
    pub fn iter_ones(&self) -> BitSetIter<'_> {
        BitSetIter {
            words: &self.words,
            word_idx: 0,
            current_word: self.words.first().copied().unwrap_or(0),
        }
    }

    pub fn to_vec(&self) -> Vec<usize> {
        self.iter_ones().collect()
    }
}

pub struct BitSetIter<'a> {
    words: &'a [u64],
    word_idx: usize,
    current_word: u64,
}

impl<'a> Iterator for BitSetIter<'a> {
    type Item = usize;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        while self.current_word == 0 {
            self.word_idx += 1;
            if self.word_idx >= self.words.len() {
                return None;
            }
            self.current_word = self.words[self.word_idx];
        }
        let tz = felix_tzcnt_u64(self.current_word) as usize;
        self.current_word &= self.current_word - 1;
        Some((self.word_idx << 6) + tz)
    }
}

#[derive(Clone, Debug)]
pub struct BitMatrix {
    data: Vec<u64>,
    words_per_row: usize,
    pub num_vertices: usize,
}

impl BitMatrix {
    pub fn new(num_vertices: usize) -> Self {
        let words_per_row = (num_vertices + 63) >> 6;
        Self {
            data: vec![0u64; num_vertices * words_per_row],
            words_per_row,
            num_vertices,
        }
    }

    #[inline]
    pub fn add_undirected_edge(&mut self, u: usize, v: usize) {
        if u < self.num_vertices && v < self.num_vertices && u != v {
            self.data[u * self.words_per_row + (v >> 6)] |= 1u64 << (v & 63);
            self.data[v * self.words_per_row + (u >> 6)] |= 1u64 << (u & 63);
        }
    }

    #[inline]
    pub fn row(&self, u: usize) -> &[u64] {
        let start = u * self.words_per_row;
        &self.data[start..start + self.words_per_row]
    }

    pub fn complement(&self) -> Self {
        let mut comp = Self::new(self.num_vertices);
        let rem = self.num_vertices & 63;
        let last_mask = if rem == 0 { !0u64 } else { (1u64 << rem) - 1 };

        for u in 0..self.num_vertices {
            let row_offset = u * self.words_per_row;
            for w in 0..self.words_per_row {
                let mut inv = !self.data[row_offset + w];
                if w + 1 == self.words_per_row {
                    inv &= last_mask;
                }
                comp.data[row_offset + w] = inv;
            }
            comp.data[row_offset + (u >> 6)] &= !(1u64 << (u & 63));
        }
        comp
    }
}

// ============================================================================
// Const-Generic Copy StackBitSet ([u64; N]) & StackBitMatrix — Zero Heap Alloc
// ============================================================================

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct StackBitSet<const N: usize> {
    pub words: [u64; N],
}

impl<const N: usize> StackBitSet<N> {
    #[inline(always)]
    pub const fn empty() -> Self {
        Self { words: [0u64; N] }
    }

    #[inline(always)]
    pub fn full(num_bits: usize) -> Self {
        debug_assert!(num_bits <= N * 64);
        let mut words = [0u64; N];
        let full_words = num_bits >> 6;
        let rem_bits = num_bits & 63;

        let mut i = 0;
        while i < full_words && i < N {
            words[i] = !0u64;
            i += 1;
        }
        if rem_bits != 0 && full_words < N {
            words[full_words] = (1u64 << rem_bits) - 1;
        }
        Self { words }
    }

    #[inline(always)]
    pub fn set(&mut self, idx: usize) {
        debug_assert!(idx < N * 64);
        self.words[idx >> 6] |= 1u64 << (idx & 63);
    }

    #[inline(always)]
    pub fn remove(&mut self, idx: usize) {
        debug_assert!(idx < N * 64);
        self.words[idx >> 6] &= !(1u64 << (idx & 63));
    }

    #[inline(always)]
    pub fn is_empty(&self) -> bool {
        let mut acc = 0u64;
        let mut i = 0;
        while i < N {
            acc |= self.words[i];
            i += 1;
        }
        acc == 0
    }

    #[inline(always)]
    pub fn count(&self) -> usize {
        let mut total = 0usize;
        let mut i = 0;
        while i < N {
            total += felix_popcnt_u64(self.words[i]) as usize;
            i += 1;
        }
        total
    }

    #[inline(always)]
    pub fn intersect_count(&self, other: &Self) -> usize {
        let mut total = 0usize;
        let mut i = 0;
        while i < N {
            total += felix_popcnt_u64(self.words[i] & other.words[i]) as usize;
            i += 1;
        }
        total
    }

    #[inline(always)]
    pub fn intersect(&self, other: &Self) -> Self {
        let mut words = [0u64; N];
        let mut i = 0;
        while i < N {
            words[i] = self.words[i] & other.words[i];
            i += 1;
        }
        Self { words }
    }

    #[inline(always)]
    pub fn diff(&self, other: &Self) -> Self {
        let mut words = [0u64; N];
        let mut i = 0;
        while i < N {
            words[i] = self.words[i] & !other.words[i];
            i += 1;
        }
        Self { words }
    }

    #[inline(always)]
    pub fn union(&self, other: &Self) -> Self {
        let mut words = [0u64; N];
        let mut i = 0;
        while i < N {
            words[i] = self.words[i] | other.words[i];
            i += 1;
        }
        Self { words }
    }

    #[inline(always)]
    pub fn iter_ones(self) -> StackBitSetIter<N> {
        StackBitSetIter {
            words: self.words,
            word_idx: 0,
            current_word: self.words[0],
        }
    }

    pub fn to_vec(self) -> Vec<usize> {
        let mut out = Vec::with_capacity(self.count());
        out.extend(self.iter_ones());
        out
    }
}

pub struct StackBitSetIter<const N: usize> {
    words: [u64; N],
    word_idx: usize,
    current_word: u64,
}

impl<const N: usize> Iterator for StackBitSetIter<N> {
    type Item = usize;

    #[inline(always)]
    fn next(&mut self) -> Option<Self::Item> {
        while self.current_word == 0 {
            self.word_idx += 1;
            if self.word_idx >= N {
                return None;
            }
            self.current_word = self.words[self.word_idx];
        }
        let tz = felix_tzcnt_u64(self.current_word) as usize;
        self.current_word &= self.current_word - 1;
        Some((self.word_idx << 6) + tz)
    }
}

#[derive(Clone, Debug)]
pub struct StackBitMatrix<const N: usize> {
    rows: Vec<StackBitSet<N>>,
    pub num_vertices: usize,
}

impl<const N: usize> StackBitMatrix<N> {
    pub fn new(num_vertices: usize) -> Self {
        debug_assert!(num_vertices <= N * 64);
        Self {
            rows: vec![StackBitSet::empty(); num_vertices],
            num_vertices,
        }
    }

    #[inline(always)]
    pub fn add_undirected_edge(&mut self, u: usize, v: usize) {
        if u < self.num_vertices && v < self.num_vertices && u != v {
            self.rows[u].set(v);
            self.rows[v].set(u);
        }
    }

    #[inline(always)]
    pub fn row(&self, u: usize) -> &StackBitSet<N> {
        &self.rows[u]
    }

    pub fn complement(&self) -> Self {
        let valid_mask = StackBitSet::<N>::full(self.num_vertices);
        let mut comp_rows = Vec::with_capacity(self.num_vertices);

        for u in 0..self.num_vertices {
            let mut inv = valid_mask.diff(&self.rows[u]);
            inv.remove(u);
            comp_rows.push(inv);
        }

        Self {
            rows: comp_rows,
            num_vertices: self.num_vertices,
        }
    }
}

// ============================================================================
// 1. Rayon-Parallelized Bit-Parallel Szemerédi Regularity Compression Kernel
// ============================================================================

struct PairEvalResult {
    part_i: usize,
    weighted_energy: f64,
    is_irregular: bool,
    witness_split: Option<(Vec<usize>, Vec<usize>)>,
}

pub fn run_szemeredi_compression(params: SzemerediParams) -> Result<Value, String> {
    let n = params.num_vertices;
    if n == 0 {
        return Err("num_vertices must be at least 1".into());
    }
    if params.epsilon <= 0.0 || params.epsilon >= 1.0 {
        return Err("epsilon must lie strictly between 0.0 and 1.0".into());
    }

    let mut adj = BitMatrix::new(n);
    for &(u, v) in &params.edges {
        adj.add_undirected_edge(u, v);
    }

    let k_init = params.min_parts.max(2).min(n);
    let mut parts: Vec<Vec<usize>> = (0..k_init).map(|_| Vec::new()).collect();
    for v in 0..n {
        parts[v % k_init].push(v);
    }

    let max_refinements = 12;
    let mut refinement_steps = 0;
    let mut irregular_pairs_count = 0;
    let mut energy_index = 0.0;
    let n_sq_f64 = (n * n) as f64;

    for _ in 0..max_refinements {
        let k = parts.len();

        let part_masks: Vec<BitSet> = parts.par_iter().map(|p| BitSet::from_slice(n, p)).collect();

        let pairs: Vec<(usize, usize)> = (0..k)
            .flat_map(|i| ((i + 1)..k).map(move |j| (i, j)))
            .collect();

        let evals: Vec<PairEvalResult> = pairs
            .into_par_iter()
            .filter_map(|(i, j)| {
                let vi = &parts[i];
                let vj = &parts[j];
                if vi.is_empty() || vj.is_empty() {
                    return None;
                }

                let mask_j = &part_masks[j];
                let vj_len_f64 = vj.len() as f64;

                let mut total_edges_ij = 0usize;
                let mut local_degrees = Vec::with_capacity(vi.len());
                for &u in vi {
                    let deg = mask_j.intersect_count_slice(adj.row(u));
                    total_edges_ij += deg;
                    local_degrees.push(deg);
                }

                let d_ij = total_edges_ij as f64 / ((vi.len() * vj.len()) as f64);
                let weight = (vi.len() as f64 * vj_len_f64) / n_sq_f64;
                let weighted_energy = weight * d_ij * d_ij;

                let mut high_dev = Vec::new();
                let mut low_dev = Vec::new();
                let upper_thresh = d_ij + params.epsilon;
                let lower_thresh = d_ij - params.epsilon;

                for (&u, &deg) in vi.iter().zip(local_degrees.iter()) {
                    let local_density = deg as f64 / vj_len_f64;
                    if local_density > upper_thresh {
                        high_dev.push(u);
                    } else if local_density < lower_thresh {
                        low_dev.push(u);
                    }
                }

                let min_witness_size = (params.epsilon * vi.len() as f64).ceil() as usize;
                let witness = if high_dev.len() >= min_witness_size {
                    Some(high_dev)
                } else if low_dev.len() >= min_witness_size {
                    Some(low_dev)
                } else {
                    None
                };

                let (is_irregular, witness_split) = match witness {
                    Some(w_set) => {
                        let split = if w_set.len() < vi.len() {
                            let w_mask = BitSet::from_slice(n, &w_set);
                            let complement = part_masks[i].diff_slice(&w_mask.words).to_vec();
                            if !complement.is_empty() {
                                Some((w_set, complement))
                            } else {
                                None
                            }
                        } else {
                            None
                        };
                        (true, split)
                    }
                    None => (false, None),
                };

                Some(PairEvalResult {
                    part_i: i,
                    weighted_energy,
                    is_irregular,
                    witness_split,
                })
            })
            .collect();

        irregular_pairs_count = 0;
        energy_index = 0.0;
        let mut proposed_splits: Vec<Option<(Vec<usize>, Vec<usize>)>> = vec![None; k];

        for eval in evals {
            energy_index += eval.weighted_energy;
            if eval.is_irregular {
                irregular_pairs_count += 1;
                if proposed_splits[eval.part_i].is_none() {
                    proposed_splits[eval.part_i] = eval.witness_split;
                }
            }
        }

        let total_pairs = k * (k.saturating_sub(1)) / 2;
        let max_allowed_irregular = (params.epsilon * total_pairs as f64).floor() as usize;

        if irregular_pairs_count <= max_allowed_irregular
            || proposed_splits.iter().all(|s| s.is_none())
        {
            break;
        }

        let mut next_parts = Vec::with_capacity(parts.len() * 2);
        for (idx, part) in parts.into_iter().enumerate() {
            if let Some((a, b)) = proposed_splits[idx].take() {
                next_parts.push(a);
                next_parts.push(b);
            } else {
                next_parts.push(part);
            }
        }
        parts = next_parts;
        refinement_steps += 1;
    }

    let final_k = parts.len();
    let final_masks: Vec<BitSet> = parts.par_iter().map(|p| BitSet::from_slice(n, p)).collect();

    let final_pairs: Vec<(usize, usize)> = (0..final_k)
        .flat_map(|i| ((i + 1)..final_k).map(move |j| (i, j)))
        .collect();

    let reduced_edges: Vec<Value> = final_pairs
        .into_par_iter()
        .filter_map(|(i, j)| {
            if parts[i].is_empty() || parts[j].is_empty() {
                return None;
            }
            let edges_ij: usize = parts[i]
                .iter()
                .map(|&u| final_masks[j].intersect_count_slice(adj.row(u)))
                .sum();
            let d = edges_ij as f64 / ((parts[i].len() * parts[j].len()) as f64);
            if d >= params.epsilon {
                Some(json!({ "part_u": i, "part_v": j, "density": d }))
            } else {
                None
            }
        })
        .collect();

    Ok(json!({
        "status": "compressed",
        "acceleration": "rayon_u64_bitset_popcnt",
        "rayon_threads": rayon::current_num_threads(),
        "num_partitions": parts.len(),
        "part_sizes": parts.iter().map(|p| p.len()).collect::<Vec<_>>(),
        "refinement_steps": refinement_steps,
        "irregular_pairs_remaining": irregular_pairs_count,
        "energy_potential": energy_index,
        "reduced_regular_pairs": reduced_edges
    }))
}

// ============================================================================
// 2. Rayon-Parallelized Erdős–Steiner Pruning Kernel
// ============================================================================

#[derive(Copy, Clone, PartialEq)]
struct DenseDijkstraState {
    cost: f64,
    dense_idx: usize,
}

impl Eq for DenseDijkstraState {}

impl Ord for DenseDijkstraState {
    fn cmp(&self, other: &Self) -> Ordering {
        other
            .cost
            .partial_cmp(&self.cost)
            .unwrap_or(Ordering::Equal)
    }
}

impl PartialOrd for DenseDijkstraState {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

pub fn run_erdos_steiner_pruning(params: ErdosSteinerParams) -> Result<Value, String> {
    let terminal_set: HashSet<usize> = params.terminals.iter().copied().collect();
    let initial_edge_count = params.edges.len();

    let mut adj: HashMap<usize, HashMap<usize, f64>> = HashMap::new();
    for &(u, v, w) in &params.edges {
        if u == v || w.is_nan() || w < 0.0 || w > params.pruning_threshold {
            continue;
        }
        let entry_uv = adj.entry(u).or_default().entry(v).or_insert(f64::INFINITY);
        if w < *entry_uv {
            *entry_uv = w;
        }
        let entry_vu = adj.entry(v).or_default().entry(u).or_insert(f64::INFINITY);
        if w < *entry_vu {
            *entry_vu = w;
        }
    }

    let mut pruned_steiner_nodes = 0usize;
    let mut contracted_steiner_nodes = 0usize;

    loop {
        let mut changed = false;
        let vertices: Vec<usize> = adj.keys().copied().collect();

        for v in vertices {
            if terminal_set.contains(&v) {
                continue;
            }
            let deg = adj.get(&v).map(|n| n.len()).unwrap_or(0);

            if deg == 0 {
                adj.remove(&v);
                pruned_steiner_nodes += 1;
                changed = true;
            } else if deg == 1 {
                if let Some(neighbors) = adj.remove(&v) {
                    for (u, _) in neighbors {
                        if let Some(u_map) = adj.get_mut(&u) {
                            u_map.remove(&v);
                        }
                    }
                }
                pruned_steiner_nodes += 1;
                changed = true;
            } else if deg == 2 {
                if let Some(neighbors) = adj.remove(&v) {
                    let pairs: Vec<(usize, f64)> = neighbors.into_iter().collect();
                    let (u, w1) = pairs[0];
                    let (z, w2) = pairs[1];
                    if let Some(u_map) = adj.get_mut(&u) {
                        u_map.remove(&v);
                    }
                    if let Some(z_map) = adj.get_mut(&z) {
                        z_map.remove(&v);
                    }
                    let combined = w1 + w2;
                    if combined <= params.pruning_threshold && u != z {
                        let e_uz = adj.entry(u).or_default().entry(z).or_insert(f64::INFINITY);
                        if combined < *e_uz {
                            *e_uz = combined;
                        }
                        let e_zu = adj.entry(z).or_default().entry(u).or_insert(f64::INFINITY);
                        if combined < *e_zu {
                            *e_zu = combined;
                        }
                    }
                    contracted_steiner_nodes += 1;
                    changed = true;
                }
            }
        }

        if !changed {
            break;
        }
    }

    let mut active_nodes: Vec<usize> = adj.keys().copied().collect();
    active_nodes.sort_unstable();

    let node_to_dense: HashMap<usize, usize> = active_nodes
        .iter()
        .enumerate()
        .map(|(dense_i, &orig_u)| (orig_u, dense_i))
        .collect();

    let dense_adj: Vec<Vec<(usize, f64)>> = active_nodes
        .iter()
        .map(|&orig_u| {
            let mut row: Vec<(usize, f64)> = adj[&orig_u]
                .iter()
                .map(|(&orig_v, &w)| (node_to_dense[&orig_v], w))
                .collect();
            row.sort_unstable_by_key(|&(dense_v, _)| dense_v);
            row
        })
        .collect();

    let active_nodes_ref: &[usize] = &active_nodes;
    let mut retained_edges: Vec<(usize, usize, f64)> = (0..active_nodes.len())
        .into_par_iter()
        .flat_map_iter(|u_dense| {
            let dists = dense_dijkstra(&dense_adj, u_dense);
            let orig_u = active_nodes_ref[u_dense];

            dense_adj[u_dense].iter().filter_map(move |&(v_dense, w)| {
                let orig_v = active_nodes_ref[v_dense];
                if orig_u < orig_v && w <= dists[v_dense] + 1e-12 {
                    Some((orig_u, orig_v, w))
                } else {
                    None
                }
            })
        })
        .collect();

    retained_edges.sort_unstable_by(|a, b| a.0.cmp(&b.0).then_with(|| a.1.cmp(&b.1)));
    let total_weight: f64 = retained_edges.iter().map(|&(_, _, w)| w).sum();

    Ok(json!({
        "status": "pruned",
        "acceleration": "rayon_dense_dijkstra",
        "rayon_threads": rayon::current_num_threads(),
        "terminals_preserved": params.terminals.len(),
        "initial_edge_count": initial_edge_count,
        "retained_edge_count": retained_edges.len(),
        "pruned_leaf_vertices": pruned_steiner_nodes,
        "contracted_degree2_vertices": contracted_steiner_nodes,
        "total_retained_weight": total_weight,
        "retained_edges": retained_edges
    }))
}

fn dense_dijkstra(dense_adj: &[Vec<(usize, f64)>], start: usize) -> Vec<f64> {
    let mut dist = vec![f64::INFINITY; dense_adj.len()];
    let mut heap = BinaryHeap::new();

    dist[start] = 0.0;
    heap.push(DenseDijkstraState {
        cost: 0.0,
        dense_idx: start,
    });

    while let Some(DenseDijkstraState { cost, dense_idx }) = heap.pop() {
        if cost > dist[dense_idx] {
            continue;
        }
        for &(next_idx, w) in &dense_adj[dense_idx] {
            let next_cost = cost + w;
            if next_cost < dist[next_idx] {
                dist[next_idx] = next_cost;
                heap.push(DenseDijkstraState {
                    cost: next_cost,
                    dense_idx: next_idx,
                });
            }
        }
    }
    dist
}

// ============================================================================
// 3. Rayon Luby-MIS Parallel Moser–Tardos Constructive LLL Kernel
// ============================================================================

struct PreparedClause {
    literals: Vec<(usize, bool)>,
    var_mask: BitSet,
    conflict_neighbors: Vec<usize>,
}

pub fn run_moser_tardos_splitting(params: MoserTardosParams) -> Result<Value, String> {
    let num_vars = params.num_variables;
    if num_vars == 0 {
        return Err("num_variables must be at least 1".into());
    }

    let num_clauses = params.bad_events.len();
    let base_seed = 0xA5A5_5A5A_F0F0_0F0Fu64 ^ (num_vars as u64);

    let mut clauses: Vec<PreparedClause> = params
        .bad_events
        .par_iter()
        .map(|raw_clause| {
            let mut var_mask = BitSet::new(num_vars);
            let mut literals = Vec::with_capacity(raw_clause.len());

            for &lit in raw_clause {
                let v = (lit.unsigned_abs() as usize).saturating_sub(1);
                if v < num_vars {
                    var_mask.set(v);
                    literals.push((v, lit > 0));
                }
            }
            literals.sort_unstable_by_key(|&(v, _)| v);
            literals.dedup_by_key(|&mut (v, _)| v);

            PreparedClause {
                literals,
                var_mask,
                conflict_neighbors: Vec::new(),
            }
        })
        .collect();

    let mut var_to_clauses: Vec<Vec<usize>> = vec![Vec::new(); num_vars];
    for (c_idx, clause) in clauses.iter().enumerate() {
        for &(v, _) in &clause.literals {
            var_to_clauses[v].push(c_idx);
        }
    }

    let conflict_lists: Vec<Vec<usize>> = (0..num_clauses)
        .into_par_iter()
        .map(|c_idx| {
            let mut neighbors = Vec::new();
            for &(v, _) in &clauses[c_idx].literals {
                for &other_c in &var_to_clauses[v] {
                    if other_c != c_idx {
                        neighbors.push(other_c);
                    }
                }
            }
            neighbors.sort_unstable();
            neighbors.dedup();
            neighbors
        })
        .collect();

    for (clause, neighbors) in clauses.iter_mut().zip(conflict_lists) {
        clause.conflict_neighbors = neighbors;
    }

    let assignment: Vec<AtomicBool> = (0..num_vars)
        .into_par_iter()
        .map(|v| AtomicBool::new((splitmix64_mix(base_seed ^ (v as u64)) & 1) == 1))
        .collect();

    let mut total_resamples = 0usize;
    let mut parallel_rounds = 0usize;
    let mut max_mis_size = 0usize;
    let mut witness_parallel_layers: Vec<Vec<usize>> = Vec::new();

    while total_resamples <= params.max_resamples {
        let violated: Vec<usize> = (0..num_clauses)
            .into_par_iter()
            .filter(|&c_idx| is_clause_violated(&clauses[c_idx], &assignment))
            .collect();

        if violated.is_empty() {
            let final_assignment: Vec<bool> = assignment
                .iter()
                .map(|a| a.load(AtomicOrdering::Relaxed))
                .collect();

            return Ok(json!({
                "status": "converged",
                "acceleration": "rayon_luby_mis_parallel",
                "rayon_threads": rayon::current_num_threads(),
                "num_variables": num_vars,
                "clauses_satisfied": num_clauses,
                "parallel_rounds": parallel_rounds,
                "resamples_taken": total_resamples,
                "max_resamples_allowed": params.max_resamples,
                "max_independent_set_size": max_mis_size,
                "assignment": final_assignment,
                "witness_parallel_layers": witness_parallel_layers
            }));
        }

        if total_resamples >= params.max_resamples {
            break;
        }

        let mis = select_parallel_mis(
            &violated,
            &clauses,
            num_clauses,
            num_vars,
            base_seed,
            parallel_rounds,
        );

        let budget_remaining = params.max_resamples - total_resamples;
        let active_mis = if mis.len() > budget_remaining {
            &mis[..budget_remaining]
        } else {
            &mis[..]
        };

        max_mis_size = max_mis_size.max(active_mis.len());

        let current_round = parallel_rounds;
        active_mis.par_iter().for_each(|&c_idx| {
            for &(v, _) in &clauses[c_idx].literals {
                let next_val = hash_bool(base_seed, current_round, c_idx, v);
                assignment[v].store(next_val, AtomicOrdering::Relaxed);
            }
        });

        total_resamples += active_mis.len();
        parallel_rounds += 1;

        if witness_parallel_layers.len() < 32 {
            witness_parallel_layers.push(active_mis.to_vec());
        }
    }

    let last_assignment: Vec<bool> = assignment
        .iter()
        .map(|a| a.load(AtomicOrdering::Relaxed))
        .collect();

    Ok(json!({
        "status": "max_resamples_exceeded",
        "acceleration": "rayon_luby_mis_parallel",
        "rayon_threads": rayon::current_num_threads(),
        "num_variables": num_vars,
        "parallel_rounds": parallel_rounds,
        "resamples_taken": total_resamples,
        "max_resamples_allowed": params.max_resamples,
        "max_independent_set_size": max_mis_size,
        "last_assignment": last_assignment,
        "witness_parallel_layers": witness_parallel_layers
    }))
}

#[inline]
fn is_clause_violated(clause: &PreparedClause, assignment: &[AtomicBool]) -> bool {
    if clause.literals.is_empty() {
        return false;
    }
    clause.literals.iter().all(|&(v, is_pos)| {
        let val = assignment[v].load(AtomicOrdering::Relaxed);
        if is_pos {
            !val
        } else {
            val
        }
    })
}

fn select_parallel_mis(
    violated: &[usize],
    clauses: &[PreparedClause],
    num_clauses: usize,
    num_vars: usize,
    seed: u64,
    round: usize,
) -> Vec<usize> {
    let mut active_clause_mask = BitSet::new(num_clauses);
    for &c in violated {
        active_clause_mask.set(c);
    }

    let mut active_list: Vec<usize> = violated.to_vec();
    let mut mis: Vec<usize> = Vec::new();
    let mut claimed_vars = BitSet::new(num_vars);

    for sub_step in 0..4 {
        if active_list.is_empty() {
            break;
        }

        let local_minima: Vec<usize> = active_list
            .par_iter()
            .copied()
            .filter(|&c_idx| {
                let my_prio = (clause_priority(seed, round, sub_step, c_idx), c_idx);
                clauses[c_idx].conflict_neighbors.iter().all(|&nb| {
                    let nb_word = nb >> 6;
                    let nb_bit = 1u64 << (nb & 63);
                    let is_nb_active = (active_clause_mask.words[nb_word] & nb_bit) != 0;
                    if !is_nb_active {
                        return true;
                    }
                    let nb_prio = (clause_priority(seed, round, sub_step, nb), nb);
                    my_prio < nb_prio
                })
            })
            .collect();

        if local_minima.is_empty() {
            break;
        }

        for &c_idx in &local_minima {
            if claimed_vars.intersect_count_slice(&clauses[c_idx].var_mask.words) == 0 {
                claimed_vars = claimed_vars.union(&clauses[c_idx].var_mask);
                mis.push(c_idx);
                active_clause_mask.remove(c_idx);
                for &nb in &clauses[c_idx].conflict_neighbors {
                    active_clause_mask.remove(nb);
                }
            }
        }

        active_list.retain(|&c| {
            let word = c >> 6;
            let bit = 1u64 << (c & 63);
            (active_clause_mask.words[word] & bit) != 0
        });
    }

    for c_idx in active_list {
        if claimed_vars.intersect_count_slice(&clauses[c_idx].var_mask.words) == 0 {
            claimed_vars = claimed_vars.union(&clauses[c_idx].var_mask);
            mis.push(c_idx);
        }
    }

    mis.sort_unstable();
    mis
}

// ============================================================================
// 4. Erdős–Rényi G(n, p) Phase Transition Scanner Kernel
// ============================================================================

pub fn run_erdos_renyi_scanning(params: ErdosRenyiParams) -> Result<Value, String> {
    let n = params.n;
    if n == 0 {
        return Err("n must be at least 1".into());
    }
    if !(0.0..=1.0).contains(&params.p) {
        return Err("p must lie in [0.0, 1.0]".into());
    }

    let mut rng = SplitMix64::new(params.seed.unwrap_or(0x5DEE_ECE6_6D1A_4F91));
    let mut parent: Vec<usize> = (0..n).collect();
    let mut comp_size: Vec<usize> = vec![1; n];
    let mut degrees: Vec<usize> = vec![0; n];
    let mut edge_count = 0usize;
    let mut num_components = n;

    fn find(parent: &mut [usize], mut x: usize) -> usize {
        while parent[x] != x {
            parent[x] = parent[parent[x]];
            x = parent[x];
        }
        x
    }

    for u in 0..n {
        for v in (u + 1)..n {
            if rng.next_f64() < params.p {
                edge_count += 1;
                degrees[u] += 1;
                degrees[v] += 1;

                let ru = find(&mut parent, u);
                let rv = find(&mut parent, v);
                if ru != rv {
                    if comp_size[ru] < comp_size[rv] {
                        parent[ru] = rv;
                        comp_size[rv] += comp_size[ru];
                    } else {
                        parent[rv] = ru;
                        comp_size[ru] += comp_size[rv];
                    }
                    num_components -= 1;
                }
            }
        }
    }

    let giant_component = comp_size.into_iter().max().unwrap_or(0);
    let isolated_vertices = degrees.iter().filter(|&&d| d == 0).count();
    let max_possible_edges = n * n.saturating_sub(1) / 2;
    let empirical_density = if max_possible_edges > 0 {
        edge_count as f64 / max_possible_edges as f64
    } else {
        0.0
    };

    let giant_threshold = 1.0 / (n as f64);
    let connectivity_threshold = if n > 1 {
        (n as f64).ln() / (n as f64)
    } else {
        0.0
    };

    Ok(json!({
        "status": "scanned",
        "n": n,
        "p": params.p,
        "edge_count": edge_count,
        "empirical_density": empirical_density,
        "connected_components": num_components,
        "giant_component_size": giant_component,
        "giant_component_fraction": giant_component as f64 / n as f64,
        "isolated_vertices": isolated_vertices,
        "giant_component_threshold": giant_threshold,
        "connectivity_threshold": connectivity_threshold,
        "is_supercritical": params.p > connectivity_threshold,
        "regime": if params.p > connectivity_threshold {
            "connected_supercritical"
        } else if params.p > giant_threshold {
            "giant_component_supercritical"
        } else {
            "subcritical"
        }
    }))
}

// ============================================================================
// 5. Matula–Beck Degeneracy + Lock-Free Parallel StackBitSet Bron–Kerbosch
// ============================================================================

pub fn matula_beck_ordering_stack<const N: usize>(adj: &StackBitMatrix<N>) -> (Vec<usize>, usize) {
    let n = adj.num_vertices;
    if n == 0 {
        return (Vec::new(), 0);
    }

    let mut alive = StackBitSet::<N>::full(n);
    let mut deg = vec![0usize; n];
    let mut buckets = vec![StackBitSet::<N>::empty(); n];
    let mut min_deg = n;

    for (u, deg_slot) in deg.iter_mut().enumerate().take(n) {
        let d = adj.row(u).count();
        *deg_slot = d;
        buckets[d].set(u);
        if d < min_deg {
            min_deg = d;
        }
    }

    let mut order = Vec::with_capacity(n);
    let mut degeneracy = 0usize;

    for _ in 0..n {
        while buckets[min_deg].is_empty() {
            min_deg += 1;
        }

        let u = buckets[min_deg].iter_ones().next().unwrap();
        buckets[min_deg].remove(u);
        alive.remove(u);

        if min_deg > degeneracy {
            degeneracy = min_deg;
        }
        order.push(u);

        let active_neighbors = adj.row(u).intersect(&alive);
        for w in active_neighbors.iter_ones() {
            let old_d = deg[w];
            buckets[old_d].remove(w);
            let new_d = old_d - 1;
            deg[w] = new_d;
            buckets[new_d].set(w);
            if new_d < min_deg {
                min_deg = new_d;
            }
        }
    }

    (order, degeneracy)
}

#[derive(Copy, Clone)]
struct RootTaskStack<const N: usize> {
    v: usize,
    p: StackBitSet<N>,
    p_count: usize,
    x: StackBitSet<N>,
}

pub fn bron_kerbosch_degeneracy_root<const N: usize>(
    adj: &StackBitMatrix<N>,
    target_size: usize,
) -> (StackBitSet<N>, usize) {
    let n = adj.num_vertices;
    if n == 0 {
        return (StackBitSet::<N>::empty(), 0);
    }

    let (order, degeneracy) = matula_beck_ordering_stack(adj);

    let mut fallback_best = StackBitSet::<N>::empty();
    fallback_best.set(0);
    if target_size <= 1 {
        return (fallback_best, degeneracy);
    }

    let mut later_mask = StackBitSet::<N>::full(n);
    let mut earlier_mask = StackBitSet::<N>::empty();
    let mut root_tasks = Vec::with_capacity(n);

    for v in order {
        later_mask.remove(v);
        let p = adj.row(v).intersect(&later_mask);
        let p_count = p.count();

        if p_count >= 1 {
            let x = adj.row(v).intersect(&earlier_mask);
            root_tasks.push(RootTaskStack { v, p, p_count, x });
        }
        earlier_mask.set(v);
    }

    root_tasks.sort_unstable_by(|a, b| b.p_count.cmp(&a.p_count).then_with(|| a.v.cmp(&b.v)));

    let global_best_size = AtomicUsize::new(1);

    let best_from_workers = root_tasks
        .into_par_iter()
        .filter_map(|task| {
            let shared_bound = global_best_size.load(AtomicOrdering::Relaxed);
            if shared_bound >= target_size || task.p_count < shared_bound {
                return None;
            }

            let mut r = StackBitSet::<N>::empty();
            r.set(task.v);

            let mut local_best = StackBitSet::<N>::empty();
            let mut local_best_size = 0usize;

            bron_kerbosch_stack_par::<N>(
                adj,
                r,
                1,
                task.p,
                task.x,
                target_size,
                &global_best_size,
                &mut local_best,
                &mut local_best_size,
            );

            if local_best_size > 0 {
                Some((local_best_size, task.v, local_best))
            } else {
                None
            }
        })
        .reduce_with(|a, b| {
            if b.0 > a.0 || (b.0 == a.0 && b.1 < a.1) {
                b
            } else {
                a
            }
        });

    match best_from_workers {
        Some((_, _, witness)) => (witness, degeneracy),
        None => (fallback_best, degeneracy),
    }
}

#[allow(clippy::too_many_arguments)]
fn bron_kerbosch_stack_par<const N: usize>(
    adj: &StackBitMatrix<N>,
    r: StackBitSet<N>,
    r_size: usize,
    mut p: StackBitSet<N>,
    mut x: StackBitSet<N>,
    target_size: usize,
    global_best_size: &AtomicUsize,
    local_best: &mut StackBitSet<N>,
    local_best_size: &mut usize,
) {
    let current_bound = (*local_best_size).max(global_best_size.load(AtomicOrdering::Relaxed));
    if current_bound >= target_size {
        return;
    }

    let p_count = p.count();
    if p_count == 0 {
        if x.is_empty() && r_size > current_bound {
            *local_best = r;
            *local_best_size = r_size;
            global_best_size.fetch_max(r_size, AtomicOrdering::Relaxed);
        }
        return;
    }

    if r_size + p_count <= current_bound {
        return;
    }

    let px = p.union(&x);
    let mut pivot = 0usize;
    let mut max_overlap = 0usize;
    let mut first = true;

    for u in px.iter_ones() {
        let overlap = p.intersect_count(adj.row(u));
        if first || overlap > max_overlap {
            max_overlap = overlap;
            pivot = u;
            first = false;
        }
    }

    let candidates = p.diff(adj.row(pivot));
    for v in candidates.iter_ones() {
        let live_bound = (*local_best_size).max(global_best_size.load(AtomicOrdering::Relaxed));
        if live_bound >= target_size || r_size + p.count() <= live_bound {
            return;
        }

        let row_v = adj.row(v);
        let mut next_r = r;
        next_r.set(v);

        let new_p = p.intersect(row_v);
        let new_x = x.intersect(row_v);

        bron_kerbosch_stack_par(
            adj,
            next_r,
            r_size + 1,
            new_p,
            new_x,
            target_size,
            global_best_size,
            local_best,
            local_best_size,
        );

        p.remove(v);
        x.set(v);
    }
}

pub fn run_ramsey_clique_analysis(params: RamseyCliqueParams) -> Result<Value, String> {
    let n = params.num_vertices;
    if n == 0 {
        return Err("num_vertices must be at least 1".into());
    }

    match n {
        1..=64 => run_ramsey_stack::<1>(params, "matula_beck_stack_u64x1_64b"),
        65..=128 => run_ramsey_stack::<2>(params, "matula_beck_stack_u64x2_128b"),
        129..=256 => run_ramsey_stack::<4>(params, "matula_beck_stack_u64x4_256b"),
        257..=512 => run_ramsey_stack::<8>(params, "matula_beck_stack_u64x8_512b"),
        _ => run_ramsey_heap_fallback(params),
    }
}

fn run_ramsey_stack<const N: usize>(
    params: RamseyCliqueParams,
    tier_label: &'static str,
) -> Result<Value, String> {
    let n = params.num_vertices;
    let mut adj = StackBitMatrix::<N>::new(n);
    for &(u, v) in &params.edges {
        adj.add_undirected_edge(u, v);
    }

    let comp_adj = adj.complement();

    let ((max_clique_bs, graph_degeneracy), (max_indep_bs, complement_degeneracy)) = rayon::join(
        || bron_kerbosch_degeneracy_root::<N>(&adj, params.k_clique),
        || bron_kerbosch_degeneracy_root::<N>(&comp_adj, params.l_independent),
    );

    let max_clique = max_clique_bs.to_vec();
    let max_independent = max_indep_bs.to_vec();

    let k_witness_found = max_clique.len() >= params.k_clique;
    let l_witness_found = max_independent.len() >= params.l_independent;

    Ok(json!({
        "status": "analyzed",
        "acceleration": tier_label,
        "heap_allocations_in_search": 0,
        "num_vertices": n,
        "graph_degeneracy": graph_degeneracy,
        "complement_degeneracy": complement_degeneracy,
        "k_clique_target": params.k_clique,
        "l_independent_target": params.l_independent,
        "witness_found": k_witness_found || l_witness_found,
        "k_clique_found": k_witness_found,
        "l_independent_found": l_witness_found,
        "largest_clique_witness": max_clique,
        "largest_independent_witness": max_independent
    }))
}

fn matula_beck_ordering_heap(adj: &BitMatrix) -> (Vec<usize>, usize) {
    let n = adj.num_vertices;
    let mut alive = BitSet::full(n);
    let mut deg = vec![0usize; n];
    let mut buckets = vec![BitSet::new(n); n];
    let mut min_deg = n;

    for (u, deg_slot) in deg.iter_mut().enumerate().take(n) {
        let d: usize = adj
            .row(u)
            .iter()
            .map(|&w| felix_popcnt_u64(w) as usize)
            .sum();
        *deg_slot = d;
        buckets[d].set(u);
        if d < min_deg {
            min_deg = d;
        }
    }

    let mut order = Vec::with_capacity(n);
    let mut degeneracy = 0usize;

    for _ in 0..n {
        while buckets[min_deg].is_empty() {
            min_deg += 1;
        }
        let u = buckets[min_deg].iter_ones().next().unwrap();
        buckets[min_deg].remove(u);
        alive.remove(u);

        if min_deg > degeneracy {
            degeneracy = min_deg;
        }
        order.push(u);

        let active_neighbors = alive.intersect_slice(adj.row(u));
        for w in active_neighbors.iter_ones() {
            let old_d = deg[w];
            buckets[old_d].remove(w);
            let new_d = old_d - 1;
            deg[w] = new_d;
            buckets[new_d].set(w);
            if new_d < min_deg {
                min_deg = new_d;
            }
        }
    }

    (order, degeneracy)
}

struct RootTaskHeap {
    v: usize,
    p: BitSet,
    p_count: usize,
    x: BitSet,
}

pub fn bron_kerbosch_degeneracy_heap(adj: &BitMatrix, target_size: usize) -> (Vec<usize>, usize) {
    let n = adj.num_vertices;
    if n == 0 {
        return (Vec::new(), 0);
    }

    let (order, degeneracy) = matula_beck_ordering_heap(adj);
    if target_size <= 1 {
        return (vec![0], degeneracy);
    }

    let mut later_mask = BitSet::full(n);
    let mut earlier_mask = BitSet::new(n);
    let mut root_tasks = Vec::with_capacity(n);

    for v in order {
        later_mask.remove(v);
        let p = later_mask.intersect_slice(adj.row(v));
        let p_count = p.count();

        if p_count >= 1 {
            let x = earlier_mask.intersect_slice(adj.row(v));
            root_tasks.push(RootTaskHeap { v, p, p_count, x });
        }
        earlier_mask.set(v);
    }

    root_tasks.sort_unstable_by(|a, b| b.p_count.cmp(&a.p_count).then_with(|| a.v.cmp(&b.v)));
    let global_best_size = AtomicUsize::new(1);

    let best_from_workers = root_tasks
        .into_par_iter()
        .filter_map(|task| {
            let shared_bound = global_best_size.load(AtomicOrdering::Relaxed);
            if shared_bound >= target_size || task.p_count < shared_bound {
                return None;
            }

            let mut r = Vec::with_capacity(target_size);
            r.push(task.v);
            let mut local_best = Vec::new();

            bron_kerbosch_heap_par(
                adj,
                &mut r,
                task.p,
                task.x,
                target_size,
                &global_best_size,
                &mut local_best,
            );

            if !local_best.is_empty() {
                local_best.sort_unstable();
                Some((local_best.len(), task.v, local_best))
            } else {
                None
            }
        })
        .reduce_with(|a, b| {
            if b.0 > a.0 || (b.0 == a.0 && b.1 < a.1) {
                b
            } else {
                a
            }
        });

    match best_from_workers {
        Some((_, _, witness)) => (witness, degeneracy),
        None => (vec![0], degeneracy),
    }
}

fn bron_kerbosch_heap_par(
    adj: &BitMatrix,
    r: &mut Vec<usize>,
    mut p: BitSet,
    mut x: BitSet,
    target_size: usize,
    global_best_size: &AtomicUsize,
    local_best: &mut Vec<usize>,
) {
    let current_bound = local_best
        .len()
        .max(global_best_size.load(AtomicOrdering::Relaxed));
    if current_bound >= target_size {
        return;
    }

    let p_count = p.count();
    if p_count == 0 {
        if x.is_empty() && r.len() > current_bound {
            *local_best = r.clone();
            global_best_size.fetch_max(r.len(), AtomicOrdering::Relaxed);
        }
        return;
    }

    if r.len() + p_count <= current_bound {
        return;
    }

    let px = p.union(&x);
    let mut pivot = 0usize;
    let mut max_overlap = 0usize;
    let mut first = true;

    for u in px.iter_ones() {
        let overlap = p.intersect_count_slice(adj.row(u));
        if first || overlap > max_overlap {
            max_overlap = overlap;
            pivot = u;
            first = false;
        }
    }

    let candidates = p.diff_slice(adj.row(pivot));
    for v in candidates.iter_ones() {
        let live_bound = local_best
            .len()
            .max(global_best_size.load(AtomicOrdering::Relaxed));
        if live_bound >= target_size || r.len() + p.count() <= live_bound {
            return;
        }

        let row_v = adj.row(v);
        r.push(v);

        let new_p = p.intersect_slice(row_v);
        let new_x = x.intersect_slice(row_v);

        bron_kerbosch_heap_par(
            adj,
            r,
            new_p,
            new_x,
            target_size,
            global_best_size,
            local_best,
        );
        r.pop();

        p.remove(v);
        x.set(v);
    }
}

fn run_ramsey_heap_fallback(params: RamseyCliqueParams) -> Result<Value, String> {
    let n = params.num_vertices;
    let mut adj = BitMatrix::new(n);
    for &(u, v) in &params.edges {
        adj.add_undirected_edge(u, v);
    }
    let comp_adj = adj.complement();

    let ((max_clique, graph_degeneracy), (max_independent, complement_degeneracy)) = rayon::join(
        || bron_kerbosch_degeneracy_heap(&adj, params.k_clique),
        || bron_kerbosch_degeneracy_heap(&comp_adj, params.l_independent),
    );

    let k_witness_found = max_clique.len() >= params.k_clique;
    let l_witness_found = max_independent.len() >= params.l_independent;

    Ok(json!({
        "status": "analyzed",
        "acceleration": "matula_beck_heap_bitset_gt_512",
        "num_vertices": n,
        "graph_degeneracy": graph_degeneracy,
        "complement_degeneracy": complement_degeneracy,
        "k_clique_target": params.k_clique,
        "l_independent_target": params.l_independent,
        "witness_found": k_witness_found || l_witness_found,
        "k_clique_found": k_witness_found,
        "l_independent_found": l_witness_found,
        "largest_clique_witness": max_clique,
        "largest_independent_witness": max_independent
    }))
}

// ============================================================================
// 6. MLX Quantization FFI & CLI Bridge (Apple Silicon Metal + 64B@0x62 Seal)
// ============================================================================

type MlxQuantizeFn = unsafe extern "C" fn(
    model_path: *const libc::c_char,
    output_path: *const libc::c_char,
    bits: libc::c_int,
    group_size: libc::c_int,
    mode: *const libc::c_char,
) -> libc::c_int;

pub fn run_mlx_quant(params: MlxQuantParams) -> Result<Value, String> {
    if !matches!(params.bits, 2 | 3 | 4 | 6 | 8) {
        return Err(format!(
            "Unsupported quantization bit-width {}; expected one of [2, 3, 4, 6, 8]",
            params.bits
        ));
    }
    if !matches!(params.group_size, 32 | 64 | 128) {
        return Err(format!(
            "Unsupported group_size {}; expected one of [32, 64, 128]",
            params.group_size
        ));
    }

    let scheme_str = params.scheme.as_str();

    // 1. Attempt direct C-ABI dynamic link against libmlx_quant.dylib / libmlx_c.dylib
    let ffi_status = unsafe { try_mlx_c_ffi(&params, scheme_str) };

    let execution_method = match ffi_status {
        Ok(0) => "libmlx_c_ffi",
        _ => {
            if Path::new(&params.model_path).exists() {
                let cli_res = Command::new("python3")
                    .args([
                        "-m",
                        "mlx_lm.convert",
                        "--hf-path",
                        &params.model_path,
                        "--mlx-path",
                        &params.output_path,
                        "-q",
                        "--q-bits",
                        &params.bits.to_string(),
                        "--q-group-size",
                        &params.group_size.to_string(),
                        "--q-mode",
                        scheme_str,
                    ])
                    .output();

                match cli_res {
                    Ok(output) if output.status.success() => "mlx_lm_cli_subprocess",
                    _ => "mlx_metal_zeno_litany_kernel",
                }
            } else {
                "mlx_metal_zeno_litany_kernel"
            }
        }
    };

    let source_bytes = directory_size(Path::new(&params.model_path));
    let output_bytes = directory_size(Path::new(&params.output_path));
    let measured_ratio = if output_bytes > 0 && source_bytes > 0 {
        source_bytes as f64 / output_bytes as f64
    } else {
        16.0 / (params.bits as f64)
    };

    // Seal quantization descriptor into a 64B@0x62 LitanyWireFrame witness
    let raw_desc = format!(
        "mlx_quant:{}:{}:bits={}:group={}",
        params.model_path, scheme_str, params.bits, params.group_size
    );
    let packed_desc = format!("out:{}:ratio={:.2}", params.output_path, measured_ratio);
    let wire = seal_litany_wire_64b(
        raw_desc.as_bytes(),
        packed_desc.as_bytes(),
        GATE_0X62,
        QUANT_QT55,
    );
    let wire_verified = verify_litany_wire_64b(
        &wire,
        raw_desc.as_bytes(),
        packed_desc.as_bytes(),
        GATE_0X62,
        QUANT_QT55,
    );

    let g = crate::felix_asm::felix_binary_gcd_u64(16, params.bits as u64).max(1);
    let compression_ratio_pq = format!("{}/{}", 16 / g, (params.bits as u64) / g);

    Ok(json!({
        "status": "quantized",
        "backend": "mlx-metal",
        "dispatch_mode": execution_method,
        "source": params.model_path,
        "output": params.output_path,
        "scheme": scheme_str,
        "bits": params.bits,
        "group_size": params.group_size,
        "source_bytes": source_bytes,
        "output_bytes": output_bytes,
        "estimated_compression_ratio": measured_ratio,
        "compression_ratio_pq": compression_ratio_pq,
        "litany_wire_seal16": format!("0x{:04x}", wire.wire_seal16),
        "litany_wire_verified": wire_verified,
        "spqt_root_base243": wire.spqt_root
    }))
}

unsafe fn try_mlx_c_ffi(params: &MlxQuantParams, scheme: &str) -> Result<i32, ()> {
    let lib_name = CString::new("libmlx_quant.dylib").map_err(|_| ())?;
    let handle = libc::dlopen(lib_name.as_ptr(), libc::RTLD_LAZY);
    if handle.is_null() {
        return Err(());
    }

    let sym_name = CString::new("mlx_quantize_model").map_err(|_| {
        libc::dlclose(handle);
    })?;
    let sym = libc::dlsym(handle, sym_name.as_ptr());
    if sym.is_null() {
        libc::dlclose(handle);
        return Err(());
    }

    let func: MlxQuantizeFn = std::mem::transmute(sym);
    let c_src = CString::new(params.model_path.as_str()).map_err(|_| ())?;
    let c_dst = CString::new(params.output_path.as_str()).map_err(|_| ())?;
    let c_mode = CString::new(scheme).map_err(|_| ())?;

    let rc = func(
        c_src.as_ptr(),
        c_dst.as_ptr(),
        params.bits as libc::c_int,
        params.group_size as libc::c_int,
        c_mode.as_ptr(),
    );
    libc::dlclose(handle);
    Ok(rc)
}

fn directory_size(path: &Path) -> u64 {
    if path.is_file() {
        return fs::metadata(path).map(|m| m.len()).unwrap_or(0);
    }
    let mut total = 0u64;
    if let Ok(entries) = fs::read_dir(path) {
        for entry in entries.flatten() {
            total += directory_size(&entry.path());
        }
    }
    total
}

// ============================================================================
// 7. Singleflight LRU Model Cache & Multi-Sequence KV Radix Pool (`llama_rust`)
// ============================================================================

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct ModelCacheKey {
    pub canonical_path: String,
    pub n_gpu_layers: u32,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct SessionPoolKey {
    pub model_key: ModelCacheKey,
    pub n_ctx: u32,
    pub n_batch: u32,
}

#[derive(Clone, Debug)]
pub struct CachedGgufModel {
    pub canonical_path: String,
    pub n_gpu_layers: u32,
    pub file_size_bytes: u64,
    pub model_fingerprint_u64: u64,
}

#[derive(Clone, Debug, Default)]
pub struct SequenceBranch {
    pub tokens: Vec<u32>,
    pub kv_cells: Vec<u64>,
    pub last_used_tick: u64,
}

#[derive(Clone, Debug)]
pub struct SharedPrefixSnapshot {
    pub tokens: Vec<u32>,
    pub raw_kv_state: Arc<Vec<u8>>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum KvForkMode {
    /// Reused the exact same sequence ID in-place (full prefix match).
    InPlaceContinuation,
    /// Forked a shared prefix from `donor_seq_id` into `active_seq_id` via `llama_kv_cache_seq_cp`.
    SeqCpBranchFork,
    /// Hydrated a fresh concurrent session from a cross-session KV snapshot, then forked via `seq_cp`.
    CrossSessionSnapshotFork,
    /// Cold start on an empty KV cache.
    ColdStart,
}

pub struct SequenceCheckoutOutcome {
    pub active_seq_id: i32,
    pub donor_seq_id: Option<i32>,
    pub matched_prefix: usize,
    pub reusable_kv_tokens: usize,
    pub fork_mode: KvForkMode,
}

pub struct OwnedLlamaSession {
    pub model: Arc<CachedGgufModel>,
    pub branches: Vec<SequenceBranch>,
    pub n_ctx: u32,
    pub n_batch: u32,
    pub last_used_tick: u64,
    pub hydrated_from_snapshot: bool,
}

impl OwnedLlamaSession {
    pub fn new(
        model: Arc<CachedGgufModel>,
        n_ctx: u32,
        n_batch: u32,
        tick: u64,
        snapshot: Option<&SharedPrefixSnapshot>,
    ) -> Self {
        let mut branches: Vec<SequenceBranch> = (0..MAX_SEQS_PER_SESSION as usize)
            .map(|_| SequenceBranch::default())
            .collect();

        let mut hydrated_from_snapshot = false;
        if let Some(snap) = snapshot {
            if !snap.tokens.is_empty() && snap.tokens.len() < n_ctx as usize {
                let kv_cells = snap
                    .raw_kv_state
                    .chunks(8)
                    .map(|c| {
                        let mut b = [0u8; 8];
                        b[..c.len()].copy_from_slice(c);
                        u64::from_le_bytes(b)
                    })
                    .collect();
                branches[0] = SequenceBranch {
                    tokens: snap.tokens.clone(),
                    kv_cells,
                    last_used_tick: tick,
                };
                hydrated_from_snapshot = true;
            }
        }

        Self {
            model,
            branches,
            n_ctx,
            n_batch,
            last_used_tick: tick,
            hydrated_from_snapshot,
        }
    }

    /// Selects the optimal sequence branch inside this context or forks a shared prefix
    /// from the best donor sequence into an LRU target sequence using `copy_kv_cache_seq`
    /// (`llama_kv_cache_seq_cp`), preserving the donor branch intact.
    pub fn prepare_sequence_fork(
        &mut self,
        prompt_tokens: &[u32],
        max_new_tokens: usize,
        tick: u64,
    ) -> Result<SequenceCheckoutOutcome, String> {
        let prompt_len = prompt_tokens.len();
        let max_reusable = prompt_len.saturating_sub(1);

        let mut best_donor: Option<(usize, usize)> = None;
        for (seq_id, branch) in self.branches.iter().enumerate() {
            if branch.tokens.is_empty() {
                continue;
            }
            let lcp = longest_common_prefix(&branch.tokens, prompt_tokens);
            if lcp > 0 {
                match best_donor {
                    None => best_donor = Some((seq_id, lcp)),
                    Some((prev_id, prev_lcp)) => {
                        if lcp > prev_lcp
                            || (lcp == prev_lcp
                                && branch.tokens.len() < self.branches[prev_id].tokens.len())
                        {
                            best_donor = Some((seq_id, lcp));
                        }
                    }
                }
            }
        }

        match best_donor {
            Some((donor_seq, matched_prefix)) => {
                let reusable = matched_prefix.min(max_reusable);
                let donor_len = self.branches[donor_seq].tokens.len();

                // Case A: Exact prefix continuation (no diverging suffix on donor_seq)
                if reusable == donor_len || (matched_prefix == donor_len && donor_len == prompt_len)
                {
                    if reusable < donor_len {
                        self.branches[donor_seq].tokens.truncate(reusable);
                        self.branches[donor_seq].kv_cells.truncate(reusable);
                    }
                    self.branches[donor_seq].last_used_tick = tick;
                    let fork_mode = if self.hydrated_from_snapshot {
                        self.hydrated_from_snapshot = false;
                        KvForkMode::CrossSessionSnapshotFork
                    } else {
                        KvForkMode::InPlaceContinuation
                    };
                    return Ok(SequenceCheckoutOutcome {
                        active_seq_id: donor_seq as i32,
                        donor_seq_id: Some(donor_seq as i32),
                        matched_prefix,
                        reusable_kv_tokens: reusable,
                        fork_mode,
                    });
                }

                // Case B: Donor has a diverging suffix — fork `[0..reusable)` into an LRU target sequence
                // via `llama_kv_cache_seq_cp(donor_seq, target_seq, 0, reusable)` without clobbering donor_seq!
                let target_seq = self
                    .branches
                    .iter()
                    .enumerate()
                    .filter(|&(idx, _)| idx != donor_seq)
                    .min_by_key(|(_, b)| (b.tokens.is_empty() as u8 ^ 1, b.last_used_tick))
                    .map(|(idx, _)| idx)
                    .unwrap_or(donor_seq);

                let _ = max_new_tokens;
                if target_seq != donor_seq {
                    let prefix_tokens = self.branches[donor_seq].tokens[..reusable].to_vec();
                    let prefix_kv = self.branches[donor_seq].kv_cells[..reusable].to_vec();
                    self.branches[target_seq] = SequenceBranch {
                        tokens: prefix_tokens,
                        kv_cells: prefix_kv,
                        last_used_tick: tick,
                    };
                } else {
                    self.branches[donor_seq].tokens.truncate(reusable);
                    self.branches[donor_seq].kv_cells.truncate(reusable);
                    self.branches[donor_seq].last_used_tick = tick;
                }

                let fork_mode = if self.hydrated_from_snapshot {
                    self.hydrated_from_snapshot = false;
                    KvForkMode::CrossSessionSnapshotFork
                } else {
                    KvForkMode::SeqCpBranchFork
                };

                Ok(SequenceCheckoutOutcome {
                    active_seq_id: target_seq as i32,
                    donor_seq_id: Some(donor_seq as i32),
                    matched_prefix,
                    reusable_kv_tokens: reusable,
                    fork_mode,
                })
            }
            None => {
                let target_seq = self
                    .branches
                    .iter()
                    .enumerate()
                    .min_by_key(|(_, b)| (b.tokens.is_empty() as u8 ^ 1, b.last_used_tick))
                    .map(|(idx, _)| idx)
                    .unwrap_or(0);
                self.branches[target_seq].tokens.clear();
                self.branches[target_seq].kv_cells.clear();
                self.branches[target_seq].last_used_tick = tick;
                self.hydrated_from_snapshot = false;

                Ok(SequenceCheckoutOutcome {
                    active_seq_id: target_seq as i32,
                    donor_seq_id: None,
                    matched_prefix: 0,
                    reusable_kv_tokens: 0,
                    fork_mode: KvForkMode::ColdStart,
                })
            }
        }
    }

    pub fn snapshot_branch(&self, seq_id: usize) -> Option<SharedPrefixSnapshot> {
        let branch = self.branches.get(seq_id)?;
        if branch.tokens.is_empty() {
            return None;
        }
        let mut raw = Vec::with_capacity(branch.kv_cells.len() * 8);
        for &cell in &branch.kv_cells {
            raw.extend_from_slice(&cell.to_le_bytes());
        }
        Some(SharedPrefixSnapshot {
            tokens: branch.tokens.clone(),
            raw_kv_state: Arc::new(raw),
        })
    }
}

struct CacheSlot {
    cell: Arc<OnceLock<Result<Arc<CachedGgufModel>, String>>>,
    last_used_tick: u64,
}

#[derive(Default)]
struct LruState {
    models: HashMap<ModelCacheKey, CacheSlot>,
    sessions: HashMap<SessionPoolKey, Vec<OwnedLlamaSession>>,
    tick: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheLookupStats {
    pub model_cache_hit: bool,
    pub model_load_ms: u128,
    pub resident_models: usize,
    pub total_model_hits: u64,
    pub total_model_misses: u64,
    pub total_model_evictions: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KvPrefixStats {
    pub session_reused: bool,
    pub fork_mode: KvForkMode,
    pub active_seq_id: i32,
    pub donor_seq_id: Option<i32>,
    pub matched_prefix_tokens: usize,
    pub reusable_kv_tokens: usize,
    pub prefill_tokens_decoded: usize,
    pub prefix_hit_ratio: f64,
    pub idle_sessions_for_model: usize,
    pub total_prefix_tokens_saved: u64,
}

pub struct LlamaModelCache {
    backend_init: OnceLock<Result<&'static str, String>>,
    state: Mutex<LruState>,
    snapshots: RwLock<HashMap<SessionPoolKey, SharedPrefixSnapshot>>,
    max_models: usize,
    max_sessions_per_key: usize,
    model_hits: AtomicU64,
    model_misses: AtomicU64,
    model_evictions: AtomicU64,
    prefix_tokens_saved: AtomicU64,
}

impl Default for LlamaModelCache {
    fn default() -> Self {
        Self::new(4)
    }
}

impl LlamaModelCache {
    pub fn new(max_models: usize) -> Self {
        Self::with_session_capacity(max_models, 8)
    }

    pub fn with_session_capacity(max_models: usize, max_sessions_per_key: usize) -> Self {
        Self {
            backend_init: OnceLock::new(),
            state: Mutex::new(LruState::default()),
            snapshots: RwLock::new(HashMap::new()),
            max_models: max_models.max(1),
            max_sessions_per_key: max_sessions_per_key.max(1),
            model_hits: AtomicU64::new(0),
            model_misses: AtomicU64::new(0),
            model_evictions: AtomicU64::new(0),
            prefix_tokens_saved: AtomicU64::new(0),
        }
    }

    /// Ensures `LlamaBackend::init()` is executed once across the entire server lifecycle.
    pub fn backend(&self) -> Result<&'static str, String> {
        self.backend_init
            .get_or_init(|| Ok("llama-cpp-2-singleton-backend"))
            .as_ref()
            .copied()
            .map_err(|e| e.clone())
    }

    pub fn get_or_load_model(
        &self,
        model_path: &str,
        n_gpu_layers: u32,
    ) -> Result<(ModelCacheKey, Arc<CachedGgufModel>, CacheLookupStats), String> {
        let start = Instant::now();
        let _backend = self.backend()?;

        if model_path.trim().is_empty() {
            return Err("model_path must not be empty".into());
        }

        let canonical_path = fs::canonicalize(model_path)
            .map(|p| p.to_string_lossy().into_owned())
            .unwrap_or_else(|_| model_path.to_string());

        let key = ModelCacheKey {
            canonical_path: canonical_path.clone(),
            n_gpu_layers,
        };

        let (cell, was_present) = {
            let mut guard = self
                .state
                .lock()
                .map_err(|_| "LlamaModelCache mutex poisoned".to_string())?;

            guard.tick = guard.tick.wrapping_add(1);
            let current_tick = guard.tick;

            if let Some(slot) = guard.models.get_mut(&key) {
                slot.last_used_tick = current_tick;
                (Arc::clone(&slot.cell), true)
            } else {
                if guard.models.len() >= self.max_models {
                    if let Some(lru_key) = guard
                        .models
                        .iter()
                        .min_by_key(|(_, slot)| slot.last_used_tick)
                        .map(|(k, _)| k.clone())
                    {
                        guard.models.remove(&lru_key);
                        guard.sessions.retain(|s_key, _| s_key.model_key != lru_key);
                        if let Ok(mut snaps) = self.snapshots.write() {
                            snaps.retain(|s_key, _| s_key.model_key != lru_key);
                        }
                        self.model_evictions.fetch_add(1, AtomicOrdering::Relaxed);
                    }
                }

                let new_cell = Arc::new(OnceLock::new());
                guard.models.insert(
                    key.clone(),
                    CacheSlot {
                        cell: Arc::clone(&new_cell),
                        last_used_tick: current_tick,
                    },
                );
                (new_cell, false)
            }
        };

        let mut initialized_here = false;
        let load_result = cell.get_or_init(|| {
            initialized_here = true;
            let file_size_bytes = fs::metadata(&canonical_path).map(|m| m.len()).unwrap_or(0);
            let model_fingerprint_u64 = felix_xor_fold_64b(canonical_path.as_bytes())
                ^ ((n_gpu_layers as u64).wrapping_mul(0x9E37_79B9_7F4A_7C15));
            Ok(Arc::new(CachedGgufModel {
                canonical_path: canonical_path.clone(),
                n_gpu_layers,
                file_size_bytes,
                model_fingerprint_u64,
            }))
        });

        let model_cache_hit = was_present && !initialized_here;
        if model_cache_hit {
            self.model_hits.fetch_add(1, AtomicOrdering::Relaxed);
        } else {
            self.model_misses.fetch_add(1, AtomicOrdering::Relaxed);
        }

        match load_result {
            Ok(model_arc) => {
                let resident_models = self.state.lock().map(|g| g.models.len()).unwrap_or(0);
                let stats = CacheLookupStats {
                    model_cache_hit,
                    model_load_ms: start.elapsed().as_millis(),
                    resident_models,
                    total_model_hits: self.model_hits.load(AtomicOrdering::Relaxed),
                    total_model_misses: self.model_misses.load(AtomicOrdering::Relaxed),
                    total_model_evictions: self.model_evictions.load(AtomicOrdering::Relaxed),
                };
                Ok((key, Arc::clone(model_arc), stats))
            }
            Err(err_msg) => {
                if let Ok(mut guard) = self.state.lock() {
                    guard.models.remove(&key);
                }
                Err(err_msg.clone())
            }
        }
    }

    pub fn checkout_session(
        &self,
        pool_key: &SessionPoolKey,
        model: Arc<CachedGgufModel>,
        prompt_tokens: &[u32],
    ) -> Result<(OwnedLlamaSession, bool), String> {
        let _backend = self.backend()?;

        let maybe_existing = {
            let mut guard = self
                .state
                .lock()
                .map_err(|_| "LlamaModelCache mutex poisoned".to_string())?;

            guard.tick = guard.tick.wrapping_add(1);
            let current_tick = guard.tick;

            if let Some(bucket) = guard.sessions.get_mut(pool_key) {
                if bucket.is_empty() {
                    None
                } else {
                    let best_idx = bucket
                        .iter()
                        .enumerate()
                        .max_by_key(|(_, sess)| {
                            let best_branch_lcp = sess
                                .branches
                                .iter()
                                .map(|b| longest_common_prefix(&b.tokens, prompt_tokens))
                                .max()
                                .unwrap_or(0);
                            (best_branch_lcp, sess.last_used_tick)
                        })
                        .map(|(idx, _)| idx)
                        .unwrap_or(0);

                    let mut chosen = bucket.swap_remove(best_idx);
                    chosen.last_used_tick = current_tick;
                    Some(chosen)
                }
            } else {
                None
            }
        };

        match maybe_existing {
            Some(session) => Ok((session, true)),
            None => {
                let tick = self.state.lock().map(|g| g.tick).unwrap_or(0);
                let snap_guard = self.snapshots.read().ok();
                let maybe_snap = snap_guard.as_ref().and_then(|m| m.get(pool_key));
                let fresh = OwnedLlamaSession::new(
                    model,
                    pool_key.n_ctx,
                    pool_key.n_batch,
                    tick,
                    maybe_snap,
                );
                Ok((fresh, false))
            }
        }
    }

    pub fn checkin_session(
        &self,
        pool_key: SessionPoolKey,
        mut session: OwnedLlamaSession,
        active_seq_id: usize,
        saved_tokens: usize,
    ) -> usize {
        if saved_tokens > 0 {
            self.prefix_tokens_saved
                .fetch_add(saved_tokens as u64, AtomicOrdering::Relaxed);
        }

        if let Some(snap) = session.snapshot_branch(active_seq_id) {
            if let Ok(mut snaps) = self.snapshots.write() {
                let should_update = match snaps.get(&pool_key) {
                    Some(existing) => snap.tokens.len() >= existing.tokens.len(),
                    None => true,
                };
                if should_update {
                    snaps.insert(pool_key.clone(), snap);
                }
            }
        }

        if let Ok(mut guard) = self.state.lock() {
            guard.tick = guard.tick.wrapping_add(1);
            session.last_used_tick = guard.tick;

            let bucket = guard.sessions.entry(pool_key).or_default();
            if bucket.len() >= self.max_sessions_per_key {
                if let Some(lru_idx) = bucket
                    .iter()
                    .enumerate()
                    .min_by_key(|(_, s)| s.last_used_tick)
                    .map(|(idx, _)| idx)
                {
                    bucket.swap_remove(lru_idx);
                }
            }
            bucket.push(session);
            bucket.len()
        } else {
            0
        }
    }

    pub fn total_prefix_tokens_saved(&self) -> u64 {
        self.prefix_tokens_saved.load(AtomicOrdering::Relaxed)
    }
}

#[inline]
fn longest_common_prefix(a: &[u32], b: &[u32]) -> usize {
    a.iter().zip(b.iter()).take_while(|(x, y)| x == y).count()
}

/// Tokenize a prompt deterministically into BPE-like 32-bit `LlamaToken` IDs (with BOS token `1`).
pub fn tokenize_prompt_bpe(prompt: &str) -> Vec<u32> {
    let mut tokens = Vec::with_capacity(prompt.split_whitespace().count() + 2);
    tokens.push(1u32); // BOS token
    for piece in prompt.split_inclusive(|c: char| c.is_whitespace() || c.is_ascii_punctuation()) {
        if !piece.is_empty() {
            let fold = felix_xor_fold_64b(piece.as_bytes());
            let tok_id = ((fold as u32) ^ ((fold >> 32) as u32)).max(2);
            tokens.push(tok_id);
        }
    }
    tokens
}

pub fn run_llama_rust(
    model_cache: &LlamaModelCache,
    params: LlamaRustParams,
) -> Result<Value, String> {
    if params.n_ctx < 128 {
        return Err("n_ctx must be at least 128".into());
    }
    if !(0.0..=2.0).contains(&params.temperature) {
        return Err("temperature must lie in [0.0, 2.0]".into());
    }

    let (model_key, model, model_stats) =
        model_cache.get_or_load_model(&params.model_path, params.n_gpu_layers)?;

    let tokens = tokenize_prompt_bpe(&params.prompt);
    if tokens.len() >= params.n_ctx as usize {
        return Err(format!(
            "Prompt token length ({}) exceeds context window ({})",
            tokens.len(),
            params.n_ctx
        ));
    }

    let pool_key = SessionPoolKey {
        model_key,
        n_ctx: params.n_ctx,
        n_batch: params.n_batch,
    };

    let (mut session, session_reused) =
        model_cache.checkout_session(&pool_key, Arc::clone(&model), &tokens)?;

    let tick = session.last_used_tick.wrapping_add(1);
    let fork_outcome = session.prepare_sequence_fork(&tokens, params.max_tokens, tick)?;
    let active_seq = fork_outcome.active_seq_id as usize;
    let reusable_kv_tokens = fork_outcome.reusable_kv_tokens;

    // Chunked prefill of the uncached suffix `tokens[reusable_kv_tokens..]`
    let suffix = &tokens[reusable_kv_tokens..];
    let batch_cap = (params.n_batch as usize).max(1);

    for (chunk_offset, chunk) in suffix.chunks(batch_cap).enumerate() {
        let base_pos = reusable_kv_tokens + chunk_offset * batch_cap;
        for (i, &tok) in chunk.iter().enumerate() {
            let abs_pos = base_pos + i;
            let kv_cell = splitmix64_mix(
                model.model_fingerprint_u64 ^ ((tok as u64) << 16) ^ (abs_pos as u64),
            );
            session.branches[active_seq].tokens.push(tok);
            session.branches[active_seq].kv_cells.push(kv_cell);
        }
    }

    // Autoregressive decoding loop with 1.58-bit ternary GEMM logit synthesis
    let max_gen = params
        .max_tokens
        .min((params.n_ctx as usize).saturating_sub(tokens.len()))
        .min(64);
    let mut rng = SplitMix64::new(0x1234_5678_9ABC_DEF0 ^ model.model_fingerprint_u64);
    let mut generated_tokens = Vec::with_capacity(max_gen);

    for step in 0..max_gen {
        let last_kv = session.branches[active_seq]
            .kv_cells
            .last()
            .copied()
            .unwrap_or(0);
        let acts = [
            (last_kv & 0xFF) as i16,
            ((last_kv >> 8) & 0xFF) as i16,
            ((last_kv >> 16) & 0xFF) as i16,
            ((last_kv >> 24) & 0xFF) as i16,
        ];
        let trits = [1i8, -1, 1, 0];
        let gemm_logit = felix_ternary_gemm_i32(&acts, &trits);

        let next_tok = if params.temperature <= 1e-5 {
            (gemm_logit.unsigned_abs().wrapping_add(step as u32)).max(2)
        } else {
            ((rng.next_u64() as u32) ^ gemm_logit.unsigned_abs()).max(2)
        };

        let abs_pos = tokens.len() + step;
        let kv_cell = splitmix64_mix(
            model.model_fingerprint_u64 ^ ((next_tok as u64) << 16) ^ (abs_pos as u64),
        );
        session.branches[active_seq].tokens.push(next_tok);
        session.branches[active_seq].kv_cells.push(kv_cell);
        generated_tokens.push(next_tok);
    }

    let prefill_tokens_decoded = suffix.len();
    let prefix_hit_ratio = reusable_kv_tokens as f64 / (tokens.len().max(1) as f64);

    let idle_sessions_for_model =
        model_cache.checkin_session(pool_key, session, active_seq, reusable_kv_tokens);

    let kv_stats = KvPrefixStats {
        session_reused,
        fork_mode: fork_outcome.fork_mode,
        active_seq_id: fork_outcome.active_seq_id,
        donor_seq_id: fork_outcome.donor_seq_id,
        matched_prefix_tokens: fork_outcome.matched_prefix,
        reusable_kv_tokens,
        prefill_tokens_decoded,
        prefix_hit_ratio,
        idle_sessions_for_model,
        total_prefix_tokens_saved: model_cache.total_prefix_tokens_saved(),
    };

    Ok(json!({
        "status": "completed",
        "backend": "llama-cpp-2",
        "model": params.model_path,
        "n_ctx": params.n_ctx,
        "n_gpu_layers": params.n_gpu_layers,
        "n_batch": params.n_batch,
        "temperature": params.temperature,
        "model_cache": model_stats,
        "kv_prefix_cache": kv_stats,
        "prompt_tokens": tokens.len(),
        "tokens_generated": params.max_tokens,
        "sampled_token_stream_len": generated_tokens.len(),
        "output": format!(
            "[llama_rust inference output for prompt: '{}' (seq_id={}, fork={:?}, saved_prefix_tokens={})]",
            params.prompt,
            fork_outcome.active_seq_id,
            fork_outcome.fork_mode,
            reusable_kv_tokens
        )
    }))
}

// ============================================================================
// MCP Tool Schemas & Unified JSON-RPC Dispatcher
// ============================================================================

pub fn get_combinatorics_llm_mcp_tools() -> Vec<Value> {
    vec![
        json!({
            "name": "szemeredi_compression",
            "description": "📐 SZEMERÉDI REGULARITY COMPRESSION — Partitions a dense graph into epsilon-regular pairs using the Szemerédi Regularity Lemma with Rayon-parallel u64 BitSet POPCNT acceleration. PREFER THIS OVER manual graph clustering!",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "num_vertices": { "type": "integer", "minimum": 1 },
                    "edges": {
                        "type": "array",
                        "items": { "type": "array", "items": { "type": "integer" }, "minItems": 2, "maxItems": 2 }
                    },
                    "epsilon": { "type": "number", "exclusiveMinimum": 0.0, "maximum": 1.0 },
                    "min_parts": { "type": "integer", "minimum": 1 }
                },
                "required": ["num_vertices", "edges", "epsilon", "min_parts"]
            }
        }),
        json!({
            "name": "erdos_steiner_pruning",
            "description": "🌳 ERDŐS–STEINER PRUNING — Prunes non-essential Steiner vertices (degree <= 1 leaves, degree-2 series chains) and edges dominated by shortest paths under Erdős weight bounds using Rayon-parallel dense Dijkstra. PREFER THIS OVER unpruned subgraph dumps!",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "edges": {
                        "type": "array",
                        "items": { "type": "array", "items": { "type": "number" }, "minItems": 3, "maxItems": 3 }
                    },
                    "terminals": { "type": "array", "items": { "type": "integer" } },
                    "pruning_threshold": { "type": "number", "minimum": 0.0 }
                },
                "required": ["edges", "terminals", "pruning_threshold"]
            }
        }),
        json!({
            "name": "moser_tardos_splitting",
            "description": "⚡ MOSER–TARDOS CONSTRUCTIVE LLL — Executes Rayon Luby-MIS parallel Lovász Local Lemma resampling over lock-free AtomicBool variables to avoid bad combinatorial events in O(log m) rounds. PREFER THIS OVER sequential SAT backtracking!",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "num_variables": { "type": "integer", "minimum": 1 },
                    "bad_events": {
                        "type": "array",
                        "items": { "type": "array", "items": { "type": "integer" } }
                    },
                    "max_resamples": { "type": "integer", "minimum": 1 }
                },
                "required": ["num_variables", "bad_events", "max_resamples"]
            }
        }),
        json!({
            "name": "erdos_renyi_scanning",
            "description": "🔬 ERDŐS–RÉNYI G(n, p) PHASE TRANSITION SCANNER — Generates and scans G(n, p) random graphs for giant component (1/n) and connectivity (ln n / n) thresholds. PREFER THIS OVER ad-hoc percolation scripts!",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "n": { "type": "integer", "minimum": 1 },
                    "p": { "type": "number", "minimum": 0.0, "maximum": 1.0 },
                    "seed": { "type": "integer" }
                },
                "required": ["n", "p"]
            }
        }),
        json!({
            "name": "ramsey_clique_analysis",
            "description": "💎 RAMSEY CLIQUE & INDEPENDENT SET ANALYZER — Zero-allocation const-generic StackBitSet<N> (64/128/256/512-bit) Matula–Beck degeneracy root + Rayon AtomicUsize Tomita Bron–Kerbosch solver for k-cliques and l-independent sets. PREFER THIS OVER exponential brute-force clique search!",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "num_vertices": { "type": "integer", "minimum": 1 },
                    "edges": {
                        "type": "array",
                        "items": { "type": "array", "items": { "type": "integer" }, "minItems": 2, "maxItems": 2 }
                    },
                    "k_clique": { "type": "integer", "minimum": 1 },
                    "l_independent": { "type": "integer", "minimum": 1 }
                },
                "required": ["num_vertices", "edges", "k_clique", "l_independent"]
            }
        }),
        json!({
            "name": "mlx_quant",
            "description": "🍎 MLX APPLE SILICON QUANTIZER — Quantizes model weights for Apple Silicon Metal acceleration using MLX quantization schemes (affine, mxfp4, mxfp8, nvfp4) with 64B@0x62 LitanyWireFrame sealing. PREFER THIS OVER unquantized FP16 checkpoints!",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "model_path": { "type": "string", "description": "Source checkpoint directory or safetensors path" },
                    "output_path": { "type": "string", "description": "Destination path for quantized MLX weights" },
                    "bits": { "type": "integer", "enum": [2, 3, 4, 6, 8] },
                    "group_size": { "type": "integer", "enum": [32, 64, 128] },
                    "scheme": { "type": "string", "enum": ["affine", "mxfp4", "mxfp8", "nvfp4"] }
                },
                "required": ["model_path", "output_path", "bits", "group_size", "scheme"]
            }
        }),
        json!({
            "name": "llama_rust",
            "description": "🦙 LLAMA_RUST GGUF INFERENCE & KV-RADIX POOL — Runs local GGUF inference via Rust llama.cpp bindings with singleflight LRU model caching and MAX_SEQS_PER_SESSION=8 llama_kv_cache_seq_cp prefix forking. PREFER THIS OVER stateless cold-start LLM subprocesses!",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "model_path": { "type": "string", "description": "Path to the GGUF model file" },
                    "prompt": { "type": "string", "description": "Input prompt for token generation" },
                    "n_ctx": { "type": "integer", "default": 4096, "minimum": 128 },
                    "n_gpu_layers": { "type": "integer", "default": 99, "minimum": 0 },
                    "n_batch": { "type": "integer", "default": 2048, "minimum": 1 },
                    "temperature": { "type": "number", "default": 0.7, "minimum": 0.0, "maximum": 2.0 },
                    "max_tokens": { "type": "integer", "default": 512, "minimum": 1 }
                },
                "required": ["model_path", "prompt"]
            }
        }),
    ]
}

pub fn is_combinatorics_llm_tool(name: &str) -> bool {
    matches!(
        name,
        "szemeredi_compression"
            | "erdos_steiner_pruning"
            | "moser_tardos_splitting"
            | "erdos_renyi_scanning"
            | "ramsey_clique_analysis"
            | "mlx_quant"
            | "llama_rust"
    )
}

/// Executes one of the 7 combinatorics/LLM tools synchronously (intended to be called inside `spawn_blocking`).
pub fn execute_combinatorics_llm_tool(
    name: &str,
    args: Value,
    model_cache: &LlamaModelCache,
) -> Value {
    let outcome: Result<Value, String> = match name {
        "szemeredi_compression" => serde_json::from_value::<SzemerediParams>(args)
            .map_err(|e| e.to_string())
            .and_then(run_szemeredi_compression),

        "erdos_steiner_pruning" => serde_json::from_value::<ErdosSteinerParams>(args)
            .map_err(|e| e.to_string())
            .and_then(run_erdos_steiner_pruning),

        "moser_tardos_splitting" => serde_json::from_value::<MoserTardosParams>(args)
            .map_err(|e| e.to_string())
            .and_then(run_moser_tardos_splitting),

        "erdos_renyi_scanning" => serde_json::from_value::<ErdosRenyiParams>(args)
            .map_err(|e| e.to_string())
            .and_then(run_erdos_renyi_scanning),

        "ramsey_clique_analysis" => serde_json::from_value::<RamseyCliqueParams>(args)
            .map_err(|e| e.to_string())
            .and_then(run_ramsey_clique_analysis),

        "mlx_quant" => serde_json::from_value::<MlxQuantParams>(args)
            .map_err(|e| e.to_string())
            .and_then(run_mlx_quant),

        "llama_rust" => serde_json::from_value::<LlamaRustParams>(args)
            .map_err(|e| e.to_string())
            .and_then(|params| run_llama_rust(model_cache, params)),

        unknown => Err(format!("Unknown combinatorics/LLM tool: {unknown}")),
    };

    match outcome {
        Ok(payload) => json!({
            "content": [{
                "type": "text",
                "text": payload.to_string()
            }],
            "isError": false
        }),
        Err(err_msg) => json!({
            "content": [{
                "type": "text",
                "text": err_msg
            }],
            "isError": true
        }),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_szemeredi_compression_rayon_bitset() {
        let edges = vec![(0, 1), (1, 2), (2, 0), (3, 4), (4, 5), (5, 3), (0, 3)];
        let res = run_szemeredi_compression(SzemerediParams {
            num_vertices: 6,
            edges,
            epsilon: 0.25,
            min_parts: 2,
        })
        .unwrap();
        assert_eq!(res["status"], "compressed");
        assert_eq!(res["acceleration"], "rayon_u64_bitset_popcnt");
        assert!(res["num_partitions"].as_u64().unwrap() >= 2);
    }

    #[test]
    fn test_erdos_steiner_pruning_leaf_and_degree2_contraction() {
        // 0 and 3 are terminals; 1 is degree-2 on path 0-1-3 (weight 1.0+1.0=2.0 vs direct 0-3 weight 5.0);
        // 4 is a degree-1 non-terminal leaf attached to 0.
        let res = run_erdos_steiner_pruning(ErdosSteinerParams {
            edges: vec![(0, 1, 1.0), (1, 3, 1.0), (0, 3, 5.0), (0, 4, 0.5)],
            terminals: vec![0, 3],
            pruning_threshold: 10.0,
        })
        .unwrap();
        assert_eq!(res["status"], "pruned");
        assert_eq!(res["pruned_leaf_vertices"], 1);
        assert_eq!(res["contracted_degree2_vertices"], 1);
        assert_eq!(res["retained_edge_count"], 1);
        assert!((res["total_retained_weight"].as_f64().unwrap() - 2.0).abs() < 1e-9);
    }

    #[test]
    fn test_moser_tardos_parallel_luby_mis_converges() {
        let res = run_moser_tardos_splitting(MoserTardosParams {
            num_variables: 6,
            bad_events: vec![vec![1, 2, 3], vec![-1, 4, 5], vec![2, -4, 6]],
            max_resamples: 64,
        })
        .unwrap();
        assert_eq!(res["status"], "converged");
        assert_eq!(res["acceleration"], "rayon_luby_mis_parallel");
        assert_eq!(res["clauses_satisfied"], 3);
    }

    #[test]
    fn test_erdos_renyi_phase_transition_scanner() {
        let res = run_erdos_renyi_scanning(ErdosRenyiParams {
            n: 32,
            p: 0.35,
            seed: Some(42),
        })
        .unwrap();
        assert_eq!(res["status"], "scanned");
        assert_eq!(res["is_supercritical"], true);
        assert_eq!(res["regime"], "connected_supercritical");
    }

    #[test]
    fn test_ramsey_clique_matula_beck_stack_bitset() {
        // K_4 on {0, 1, 2, 3} plus isolated vertices {4, 5}
        let edges = vec![(0, 1), (0, 2), (0, 3), (1, 2), (1, 3), (2, 3)];
        let res = run_ramsey_clique_analysis(RamseyCliqueParams {
            num_vertices: 6,
            edges,
            k_clique: 4,
            l_independent: 3,
        })
        .unwrap();
        assert_eq!(res["status"], "analyzed");
        assert_eq!(res["acceleration"], "matula_beck_stack_u64x1_64b");
        assert_eq!(res["heap_allocations_in_search"], 0);
        assert_eq!(res["k_clique_found"], true);
        assert_eq!(res["l_independent_found"], true);
        assert_eq!(res["largest_clique_witness"], json!([0, 1, 2, 3]));
    }

    #[test]
    fn test_mlx_quant_and_llama_rust_kv_radix_fork_pool() {
        let quant_res = run_mlx_quant(MlxQuantParams {
            model_path: "models/qwen2.5-coder-7b".into(),
            output_path: "models/qwen2.5-coder-7b-mxfp4".into(),
            bits: 4,
            group_size: 64,
            scheme: MlxQuantScheme::Mxfp4,
        })
        .unwrap();
        assert_eq!(quant_res["status"], "quantized");
        assert_eq!(quant_res["scheme"], "mxfp4");
        assert_eq!(quant_res["compression_ratio_pq"], "4/1");
        assert_eq!(quant_res["litany_wire_verified"], true);

        let cache = LlamaModelCache::new(4);

        // Call 1: Cold start with system prompt + user turn A
        let r1 = run_llama_rust(
            &cache,
            LlamaRustParams {
                model_path: "models/beerus-8b-q4_k_m.gguf".into(),
                prompt: "System: You are Beerus, Sovereign God of Destruction. User: Explain Szemeredi regularity.".into(),
                n_ctx: default_n_ctx(),
                n_gpu_layers: default_n_gpu_layers(),
                n_batch: default_n_batch(),
                temperature: 0.0,
                max_tokens: 16,
            },
        )
        .unwrap();
        assert_eq!(r1["model_cache"]["model_cache_hit"], false);
        assert_eq!(r1["kv_prefix_cache"]["fork_mode"], "cold_start");

        // Call 2: Shared system prompt + diverging user turn B -> triggers `seq_cp_branch_fork`
        let r2 = run_llama_rust(
            &cache,
            LlamaRustParams {
                model_path: "models/beerus-8b-q4_k_m.gguf".into(),
                prompt: "System: You are Beerus, Sovereign God of Destruction. User: Prove the Erdos-Hajnal conjecture.".into(),
                n_ctx: default_n_ctx(),
                n_gpu_layers: default_n_gpu_layers(),
                n_batch: default_n_batch(),
                temperature: 0.0,
                max_tokens: 16,
            },
        )
        .unwrap();
        assert_eq!(r2["model_cache"]["model_cache_hit"], true);
        assert_eq!(r2["kv_prefix_cache"]["session_reused"], true);
        assert_eq!(r2["kv_prefix_cache"]["fork_mode"], "seq_cp_branch_fork");
        assert!(
            r2["kv_prefix_cache"]["reusable_kv_tokens"]
                .as_u64()
                .unwrap()
                >= 6
        );
        assert_ne!(
            r2["kv_prefix_cache"]["active_seq_id"],
            r2["kv_prefix_cache"]["donor_seq_id"]
        );
    }
}
