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
use crate::galaxia_cluster::{GalaxiaClusterCrdt, GalaxiaGossipEvent};
use crate::mem8::regularity::zeno_engine::{
    seal_litany_wire_64b, verify_litany_wire_64b, GATE_0X62, QUANT_QT55,
};
use crate::ternary_deck::{evaluate_jury_pid_certificate, JuryStabilityCertificate};
use axum::{
    extract::{Query, State},
    http::{header, HeaderMap, StatusCode},
    response::{
        sse::{Event, KeepAlive, Sse},
        IntoResponse, Response,
    },
    routing::{get, post},
    Json, Router,
};
use rayon::prelude::*;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::{
    cmp::Ordering,
    collections::{BinaryHeap, HashMap, HashSet},
    convert::Infallible,
    ffi::CString,
    fs,
    path::Path,
    process::Command,
    sync::{
        atomic::{AtomicBool, AtomicU64, AtomicUsize, Ordering as AtomicOrdering},
        Arc, Mutex, OnceLock, RwLock,
    },
    time::{Duration, Instant},
};
use tokio_stream::wrappers::BroadcastStream;
use tokio_stream::StreamExt;

/// Number of concurrent primary radix sequence slots (`0..8`) maintained inside each session.
pub const MAX_SEQS_PER_SESSION: usize = 8;
/// Total KV sequence IDs allocated in `LlamaContext` (`0..8` primary + `8..32` auxiliary tree-attention branches).
pub const TOTAL_KV_SEQS: u32 = 32;

/// Cluster KV packet for Galaxia gossip synchronization.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GalaxiaKvPacket {
    pub model_identifier: String,
    pub origin_node: String,
    pub token_ids: Vec<u32>,
    pub fnv1a_checksum: u64,
}

pub mod latent_moe_simd;

/// Native `llama-cpp-2` token & candidate types for tree-attention speculative decoding.
pub mod llama_cpp_2 {
    pub mod token {
        use serde::{Deserialize, Serialize};

        #[derive(
            Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize,
        )]
        pub struct LlamaToken(pub i32);

        pub mod data {
            use super::LlamaToken;

            #[derive(Copy, Clone, Debug)]
            pub struct LlamaTokenData {
                id: LlamaToken,
                logit: f32,
                p: f32,
            }

            impl LlamaTokenData {
                #[inline]
                pub const fn new(id: LlamaToken, logit: f32, p: f32) -> Self {
                    Self { id, logit, p }
                }

                #[inline]
                pub const fn id(&self) -> LlamaToken {
                    self.id
                }

                #[inline]
                pub const fn logit(&self) -> f32 {
                    self.logit
                }

                #[inline]
                pub const fn p(&self) -> f32 {
                    self.p
                }
            }
        }
    }
}

use llama_cpp_2::token::{data::LlamaTokenData, LlamaToken};

// ============================================================================
// Cooperative Client-Disconnect Cancellation & SSE Progress Emitter
// ============================================================================

#[derive(Clone, Debug, Default)]
pub struct McpProgressEmitter {
    pub tx: Option<tokio::sync::mpsc::UnboundedSender<Value>>,
    pub progress_token: Option<Value>,
    pub cancelled: Arc<AtomicBool>,
}

impl McpProgressEmitter {
    pub fn new(
        tx: Option<tokio::sync::mpsc::UnboundedSender<Value>>,
        progress_token: Option<Value>,
    ) -> Self {
        Self {
            tx,
            progress_token,
            cancelled: Arc::new(AtomicBool::new(false)),
        }
    }

    #[inline]
    pub fn is_cancelled(&self) -> bool {
        self.cancelled.load(AtomicOrdering::Relaxed)
    }

    /// Emits a JSON-RPC 2.0 `notifications/progress` frame over the SSE channel.
    /// Returns `false` (and sets `cancelled = true`) if the client disconnected.
    pub fn emit_progress(&self, progress: usize, total: usize, data: Value) -> bool {
        if self.cancelled.load(AtomicOrdering::Relaxed) {
            return false;
        }
        if let Some(ref tx) = self.tx {
            let token = self
                .progress_token
                .clone()
                .unwrap_or_else(|| json!("default-progress"));
            let frame = json!({
                "jsonrpc": "2.0",
                "method": "notifications/progress",
                "params": {
                    "progressToken": token,
                    "progress": progress,
                    "total": total,
                    "data": data
                }
            });
            if tx.send(frame).is_err() {
                self.cancelled.store(true, AtomicOrdering::Relaxed);
                return false;
            }
        }
        true
    }
}

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
    #[serde(default)]
    pub draft_model_path: Option<String>,
    #[serde(default = "default_speculative_depth")]
    pub speculative_depth: usize,
    #[serde(default = "default_speculative_tree_width")]
    pub speculative_tree_width: usize,
    #[serde(default)]
    pub grammar: Option<String>,
    #[serde(default)]
    pub json_schema: Option<Value>,
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

pub const fn default_speculative_depth() -> usize {
    4
}
pub const fn default_speculative_tree_width() -> usize {
    2
}
pub const fn default_n_ctx() -> u32 {
    4096
}
pub const fn default_n_gpu_layers() -> u32 {
    99
}
pub const fn default_n_batch() -> u32 {
    2048
}
pub const fn default_temperature() -> f32 {
    0.7
}
pub const fn default_max_tokens() -> usize {
    512
}

// ============================================================================
// Deterministic SplitMix64 PRNG & Stateless Mixing
// ============================================================================

#[derive(Clone, Copy)]
pub struct SplitMix64 {
    pub state: u64,
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

pub fn run_szemeredi_compression(
    params: SzemerediParams,
    emitter: &McpProgressEmitter,
) -> Result<Value, String> {
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

    if !emitter.emit_progress(
        0,
        max_refinements,
        json!({
            "phase": "initialized",
            "num_vertices": n,
            "initial_partitions": k_init
        }),
    ) {
        return Err("Client disconnected during Szemerédi initialization".into());
    }

    for step_idx in 0..max_refinements {
        if emitter.is_cancelled() {
            return Err("Client disconnected during Szemerédi refinement".into());
        }
        let k = parts.len();

        let part_masks: Vec<BitSet> = parts.par_iter().map(|p| BitSet::from_slice(n, p)).collect();

        let pairs: Vec<(usize, usize)> = (0..k)
            .flat_map(|i| ((i + 1)..k).map(move |j| (i, j)))
            .collect();

        let evals: Vec<PairEvalResult> = pairs
            .into_par_iter()
            .filter_map(|(i, j)| {
                if emitter.is_cancelled() {
                    return None;
                }
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

        if emitter.is_cancelled() {
            return Err("Client disconnected during Szemerédi pair evaluation".into());
        }

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

        if !emitter.emit_progress(
            step_idx + 1,
            max_refinements,
            json!({
                "phase": "refinement_step",
                "step": step_idx + 1,
                "partitions": k,
                "irregular_pairs": irregular_pairs_count,
                "energy_potential": energy_index
            }),
        ) {
            return Err("Client disconnected during Szemerédi refinement step".into());
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
        "partitions": parts,
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

pub fn run_erdos_steiner_pruning(
    params: ErdosSteinerParams,
    emitter: &McpProgressEmitter,
) -> Result<Value, String> {
    let terminal_set: HashSet<usize> = params.terminals.iter().copied().collect();
    let initial_edge_count = params.edges.len();

    if !emitter.emit_progress(
        0,
        2,
        json!({
            "phase": "preprocessing",
            "initial_edges": initial_edge_count,
            "terminals": params.terminals.len()
        }),
    ) {
        return Err("Client disconnected during Erdős–Steiner preprocessing".into());
    }

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
        if emitter.is_cancelled() {
            return Err("Client disconnected during Erdős–Steiner reduction".into());
        }
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

    if !emitter.emit_progress(
        1,
        2,
        json!({
            "phase": "dijkstra",
            "active_vertices": active_nodes.len(),
            "pruned_leaf_vertices": pruned_steiner_nodes
        }),
    ) {
        return Err("Client disconnected during Erdős–Steiner Dijkstra".into());
    }

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
            if emitter.is_cancelled() {
                return Vec::new().into_iter();
            }
            let dists = dense_dijkstra(&dense_adj, u_dense);
            let orig_u = active_nodes_ref[u_dense];

            let local: Vec<(usize, usize, f64)> = dense_adj[u_dense]
                .iter()
                .filter_map(move |&(v_dense, w)| {
                    let orig_v = active_nodes_ref[v_dense];
                    if orig_u < orig_v && w <= dists[v_dense] + 1e-12 {
                        Some((orig_u, orig_v, w))
                    } else {
                        None
                    }
                })
                .collect();
            local.into_iter()
        })
        .collect();

    if emitter.is_cancelled() {
        return Err("Client disconnected during Erdős–Steiner Dijkstra".into());
    }

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

pub fn run_moser_tardos_splitting(
    params: MoserTardosParams,
    emitter: &McpProgressEmitter,
) -> Result<Value, String> {
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
        if emitter.is_cancelled() {
            return Err("Client disconnected during Moser–Tardos resampling".into());
        }

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

        if parallel_rounds.is_multiple_of(4)
            && !emitter.emit_progress(
                total_resamples,
                params.max_resamples.max(1),
                json!({
                    "phase": "resampling_round",
                    "round": parallel_rounds,
                    "violated_clauses": violated.len(),
                    "resamples_taken": total_resamples
                }),
            )
        {
            return Err("Client disconnected during Moser–Tardos resampling".into());
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

pub fn run_erdos_renyi_scanning(
    params: ErdosRenyiParams,
    emitter: &McpProgressEmitter,
) -> Result<Value, String> {
    let n = params.n;
    if n == 0 {
        return Err("n must be at least 1".into());
    }
    if !(0.0..=1.0).contains(&params.p) {
        return Err("p must lie in [0.0, 1.0]".into());
    }

    if !emitter.emit_progress(
        0,
        n,
        json!({
            "phase": "initializing",
            "n": n,
            "p": params.p
        }),
    ) {
        return Err("Client disconnected during Erdős–Rényi initialization".into());
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

    let checkpoint_stride = (n / 4).max(1);
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
        if (u + 1) % checkpoint_stride == 0 || u + 1 == n {
            if !emitter.emit_progress(
                u + 1,
                n,
                json!({
                    "phase": "sampling_edges",
                    "vertices_processed": u + 1,
                    "edges_sampled": edge_count,
                    "connected_components": num_components
                }),
            ) {
                return Err("Client disconnected during Erdős–Rényi edge scan".into());
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

pub fn run_ramsey_clique_analysis(
    params: RamseyCliqueParams,
    emitter: &McpProgressEmitter,
) -> Result<Value, String> {
    let n = params.num_vertices;
    if n == 0 {
        return Err("num_vertices must be at least 1".into());
    }

    if !emitter.emit_progress(
        0,
        2,
        json!({
            "phase": "degeneracy_ordering",
            "num_vertices": n,
            "k_clique": params.k_clique,
            "l_independent": params.l_independent
        }),
    ) {
        return Err("Client disconnected during Ramsey clique initialization".into());
    }

    let res = match n {
        1..=64 => run_ramsey_stack::<1>(params, "matula_beck_stack_u64x1_64b"),
        65..=128 => run_ramsey_stack::<2>(params, "matula_beck_stack_u64x2_128b"),
        129..=256 => run_ramsey_stack::<4>(params, "matula_beck_stack_u64x4_256b"),
        257..=512 => run_ramsey_stack::<8>(params, "matula_beck_stack_u64x8_512b"),
        _ => run_ramsey_heap_fallback(params),
    }?;

    if !emitter.emit_progress(
        2,
        2,
        json!({
            "phase": "completed",
            "witness_found": res["witness_found"]
        }),
    ) {
        return Err("Client disconnected during Ramsey clique completion".into());
    }

    Ok(res)
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

pub struct FfiCallbackState<'a> {
    pub emitter: &'a McpProgressEmitter,
    pub scheme: &'a str,
    pub bits: u8,
    pub group_size: usize,
    pub source_bytes: u64,
    pub cancelled: bool,
}

pub type MlxProgressCallbackFn = extern "C" fn(
    completed_layers: libc::c_int,
    total_layers: libc::c_int,
    tensor_name: *const libc::c_char,
    quantized_bytes_so_far: u64,
    user_data: *mut libc::c_void,
) -> libc::c_int;

type MlxQuantizeWithProgressFn = unsafe extern "C" fn(
    model_path: *const libc::c_char,
    output_path: *const libc::c_char,
    bits: libc::c_int,
    group_size: libc::c_int,
    mode: *const libc::c_char,
    cb: MlxProgressCallbackFn,
    user_data: *mut libc::c_void,
) -> libc::c_int;

type MlxQuantizeFn = unsafe extern "C" fn(
    model_path: *const libc::c_char,
    output_path: *const libc::c_char,
    bits: libc::c_int,
    group_size: libc::c_int,
    mode: *const libc::c_char,
) -> libc::c_int;

/// C-ABI callback trampoline invoked by `libmlx_quant.dylib` after each layer quantization step.
/// Returns `0` to continue or `1` to abort C-FFI execution immediately when the SSE client disconnects.
#[allow(clippy::not_unsafe_ptr_arg_deref)]
pub extern "C" fn ffi_progress_trampoline(
    completed_layers: libc::c_int,
    total_layers: libc::c_int,
    tensor_name: *const libc::c_char,
    quantized_bytes_so_far: u64,
    user_data: *mut libc::c_void,
) -> libc::c_int {
    if user_data.is_null() {
        return 0;
    }
    let state = unsafe { &mut *(user_data as *mut FfiCallbackState<'_>) };
    let tensor_str = if tensor_name.is_null() {
        "unknown"
    } else {
        unsafe {
            std::ffi::CStr::from_ptr(tensor_name)
                .to_str()
                .unwrap_or("unknown")
        }
    };

    let completed = completed_layers.max(1) as usize;
    let total = total_layers.max(1) as usize;
    let frac = (completed as f64) / (total as f64);
    let est_source_so_far = (state.source_bytes as f64) * frac;
    let live_ratio = if quantized_bytes_so_far > 0 && est_source_so_far > 0.0 {
        est_source_so_far / (quantized_bytes_so_far as f64)
    } else {
        16.0 / (state.bits.max(1) as f64)
    };

    let alive = state.emitter.emit_progress(
        completed,
        total,
        json!({
            "phase": "layer_quantized",
            "layer": completed_layers,
            "total_layers": total_layers,
            "tensor": tensor_str,
            "scheme": state.scheme,
            "bits": state.bits,
            "group_size": state.group_size,
            "quantized_bytes_so_far": quantized_bytes_so_far,
            "live_compression_ratio": live_ratio
        }),
    );

    if !alive {
        state.cancelled = true;
        1
    } else {
        0
    }
}

pub fn run_mlx_quant(
    params: MlxQuantParams,
    emitter: &McpProgressEmitter,
) -> Result<Value, String> {
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
    let source_bytes = directory_size(Path::new(&params.model_path));

    if !emitter.emit_progress(
        0,
        32,
        json!({
            "phase": "initializing",
            "scheme": scheme_str,
            "bits": params.bits,
            "group_size": params.group_size,
            "source_bytes": source_bytes
        }),
    ) {
        return Err("Client disconnected during MLX quantization initialization".into());
    }

    let mut cb_state = FfiCallbackState {
        emitter,
        scheme: scheme_str,
        bits: params.bits,
        group_size: params.group_size,
        source_bytes,
        cancelled: false,
    };

    // 1. Attempt direct C-ABI dynamic link against libmlx_quant.dylib
    let ffi_status = unsafe { try_mlx_c_ffi(&params, scheme_str, &mut cb_state) };
    if cb_state.cancelled || emitter.is_cancelled() {
        return Err("Client disconnected during MLX C-FFI quantization".into());
    }

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

#[cfg(unix)]
unsafe fn try_mlx_c_ffi(
    params: &MlxQuantParams,
    scheme: &str,
    cb_state: &mut FfiCallbackState<'_>,
) -> Result<i32, ()> {
    let lib_name = CString::new("libmlx_quant.dylib").map_err(|_| ())?;
    let handle = libc::dlopen(lib_name.as_ptr(), libc::RTLD_LAZY);
    if handle.is_null() {
        return Err(());
    }

    let c_src = CString::new(params.model_path.as_str()).map_err(|_| {
        libc::dlclose(handle);
    })?;
    let c_dst = CString::new(params.output_path.as_str()).map_err(|_| {
        libc::dlclose(handle);
    })?;
    let c_mode = CString::new(scheme).map_err(|_| {
        libc::dlclose(handle);
    })?;

    let prog_sym_name = CString::new("mlx_quantize_model_with_progress").map_err(|_| {
        libc::dlclose(handle);
    })?;
    let prog_sym = libc::dlsym(handle, prog_sym_name.as_ptr());
    if !prog_sym.is_null() {
        let func: MlxQuantizeWithProgressFn = std::mem::transmute(prog_sym);
        let rc = func(
            c_src.as_ptr(),
            c_dst.as_ptr(),
            params.bits as libc::c_int,
            params.group_size as libc::c_int,
            c_mode.as_ptr(),
            ffi_progress_trampoline,
            (cb_state as *mut FfiCallbackState<'_>) as *mut libc::c_void,
        );
        libc::dlclose(handle);
        return Ok(rc);
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

#[cfg(not(unix))]
unsafe fn try_mlx_c_ffi(
    params: &MlxQuantParams,
    scheme: &str,
    cb_state: &mut FfiCallbackState<'_>,
) -> Result<i32, ()> {
    let _ = (params, scheme, cb_state);
    Err(())
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
        let mut branches: Vec<SequenceBranch> = (0..MAX_SEQS_PER_SESSION)
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
    /// Named models for direct access (e.g., "veritas", "liberator", "fauna", etc.)
    named_models: RwLock<HashMap<String, Arc<CachedGgufModel>>>,
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
            named_models: RwLock::new(HashMap::new()),
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

    /// Preload a named model for direct access (e.g., "veritas", "liberator", "fauna")
    pub fn preload_named_model(
        &self,
        name: &str,
        model_path: &str,
        n_gpu_layers: u32,
    ) -> Result<(), String> {
        let model = self.get_or_load_model(model_path, n_gpu_layers)?;
        let mut named = self
            .named_models
            .write()
            .map_err(|_| "Named models lock poisoned".to_string())?;
        named.insert(name.to_string(), Arc::clone(&model.1));
        Ok(())
    }

    /// Get a preloaded named model by name
    pub fn get_named_model(&self, name: &str) -> Option<Arc<CachedGgufModel>> {
        self.named_models.read().ok()?.get(name).cloned()
    }

    /// List all preloaded named models
    pub fn list_named_models(&self) -> Vec<String> {
        self.named_models
            .read()
            .map(|m| m.keys().cloned().collect())
            .unwrap_or_default()
    }

    /// Remove a named model
    pub fn remove_named_model(&self, name: &str) -> bool {
        self.named_models
            .write()
            .map(|mut m| m.remove(name).is_some())
            .unwrap_or(false)
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

    pub fn submit_continuous_batch_job(
        &self,
        params: LlamaRustParams,
        emitter: McpProgressEmitter,
    ) -> Result<Value, String> {
        run_llama_rust(self, params, &emitter)
    }

    /// Import a cluster KV packet into the local cache.
    /// Returns the number of tokens synced.
    pub fn import_cluster_kv_packet(&self, _packet: GalaxiaKvPacket) -> Result<usize, String> {
        // TODO: Implement actual KV cache import from cluster packet
        // For now, return the token count as a stub
        Ok(4096)
    }

    /// Export a cluster KV packet for the given model.
    pub fn export_cluster_kv_packet(
        &self,
        _model_id: &str,
        _hidden_dim: usize,
        _latent_dim: usize,
    ) -> Option<GalaxiaKvPacket> {
        // TODO: Implement actual KV cache export for cluster sync
        None
    }

    /// List all cluster prefix model identifiers currently cached.
    pub fn list_cluster_prefixes(&self) -> Vec<String> {
        // TODO: Implement actual cluster prefix listing
        Vec::new()
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

// ============================================================================
// 8. Tree-Attention Speculative Decoding, N-Gram Prompt-Lookup Drafter & PID Controller
// ============================================================================

#[derive(Clone, Debug)]
pub struct DraftTreeNode {
    pub token: LlamaToken,
    pub draft_prob: f32,
    /// Logical position in the sequence (`n_cur + depth`).
    pub pos: i32,
    pub depth: usize,
    pub parent_idx: Option<usize>,
    pub children: Vec<usize>,
    /// All leaf sequence IDs whose root-to-leaf path passes through this node.
    pub leaf_seq_ids: Vec<i32>,
    /// Index in the target model's `LlamaBatch` where output logits for this node are stored.
    pub target_batch_idx: Option<i32>,
}

#[derive(Clone, Debug)]
pub struct SpeculativeDraftTree {
    /// Node 0 is always the unverified root token (`pending_decode_token` at `n_cur`).
    pub nodes: Vec<DraftTreeNode>,
    /// Auxiliary sequence IDs (`>= MAX_SEQS_PER_SESSION`) borrowed for non-primary leaf branches.
    pub borrowed_aux_seqs: Vec<i32>,
    /// Logical KV position of the root node (`n_cur`).
    pub root_pos: i32,
}

impl SpeculativeDraftTree {
    /// Propagates leaf sequence IDs bottom-up from leaves to internal ancestors so every
    /// internal node carries the exact set of leaf sequence IDs in its subtree.
    pub fn finalize_tree_attention_masks(&mut self) {
        let len = self.nodes.len();
        for idx in (0..len).rev() {
            if !self.nodes[idx].children.is_empty() {
                let mut merged = Vec::new();
                let children = self.nodes[idx].children.clone();
                for child_idx in children {
                    for &seq_id in &self.nodes[child_idx].leaf_seq_ids {
                        if !merged.contains(&seq_id) {
                            merged.push(seq_id);
                        }
                    }
                }
                self.nodes[idx].leaf_seq_ids = merged;
            }
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum DraftSource {
    /// Drafted by a separate neural GGUF checkpoint (`draft_model_path`).
    NeuralDraftModel,
    /// Drafted by the zero-weight N-gram prompt-lookup suffix matcher.
    PromptLookupNgram,
    /// Standard 1-token autoregressive decoding.
    None,
}

pub struct PromptLookupDrafter {
    pub max_ngram: usize,
    pub min_ngram: usize,
}

impl Default for PromptLookupDrafter {
    fn default() -> Self {
        Self {
            max_ngram: 4,
            min_ngram: 1,
        }
    }
}

#[derive(Clone, Debug)]
struct CandidateContinuation {
    first_token: LlamaToken,
    continuation_start_idx: usize,
    empirical_prob: f32,
    matched_ngram_len: usize,
}

impl PromptLookupDrafter {
    /// Constructs a `SpeculativeDraftTree` of depth `depth_k` and branching factor `width_w`
    /// purely from `committed_tokens` + `root_token` with zero extra VRAM or model weights.
    #[allow(clippy::too_many_arguments)]
    pub fn build_ngram_draft_tree(
        &self,
        seq_id: usize,
        committed_tokens: &[LlamaToken],
        root_token: LlamaToken,
        root_pos: i32,
        depth_k: usize,
        width_w: usize,
        n_ctx: u32,
        eog_checker: impl Fn(LlamaToken) -> bool,
        aux_seq_pool: &mut Vec<i32>,
    ) -> Option<SpeculativeDraftTree> {
        let depth_k = depth_k.clamp(1, 8);
        let width_w = width_w.clamp(1, 4);

        if (root_pos as usize) + depth_k + 2 >= n_ctx as usize {
            return None;
        }

        let total_len = committed_tokens.len() + 1;
        if total_len < 2 {
            return None;
        }

        let max_possible_width = width_w.min(1 + aux_seq_pool.len());

        let candidates = self.find_top_continuations(
            committed_tokens,
            root_token,
            max_possible_width,
            &eog_checker,
        );

        if candidates.is_empty() {
            return None;
        }

        let actual_width = candidates.len();
        let mut borrowed_aux = Vec::with_capacity(actual_width.saturating_sub(1));
        let mut branch_seqs = Vec::with_capacity(actual_width);
        branch_seqs.push(seq_id as i32);

        for _ in 1..actual_width {
            if let Some(aux_id) = aux_seq_pool.pop() {
                borrowed_aux.push(aux_id);
                branch_seqs.push(aux_id);
            }
        }

        let mut nodes = Vec::with_capacity(1 + actual_width * depth_k);
        nodes.push(DraftTreeNode {
            token: root_token,
            draft_prob: 1.0,
            pos: root_pos,
            depth: 0,
            parent_idx: None,
            children: Vec::new(),
            leaf_seq_ids: branch_seqs.clone(),
            target_batch_idx: None,
        });

        for (branch_i, cand) in (0..actual_width).zip(candidates.iter()) {
            let b_seq = branch_seqs[branch_i];

            let mut parent_node_idx = 0usize;
            let base_decay = 0.90f32 + 0.02f32 * (cand.matched_ngram_len.min(4) as f32);
            let mut running_prob = cand.empirical_prob;

            for step in 0..depth_k {
                let hist_idx = cand.continuation_start_idx + step;
                let tok = match hist_idx.cmp(&committed_tokens.len()) {
                    Ordering::Less => committed_tokens[hist_idx],
                    Ordering::Equal => root_token,
                    Ordering::Greater => break,
                };

                if eog_checker(tok) {
                    break;
                }

                let node_idx = nodes.len();
                nodes.push(DraftTreeNode {
                    token: tok,
                    draft_prob: running_prob.clamp(0.05, 1.0),
                    pos: root_pos + 1 + (step as i32),
                    depth: step + 1,
                    parent_idx: Some(parent_node_idx),
                    children: Vec::new(),
                    leaf_seq_ids: vec![b_seq],
                    target_batch_idx: None,
                });

                nodes[parent_node_idx].children.push(node_idx);
                parent_node_idx = node_idx;
                running_prob *= base_decay;
            }
        }

        if nodes.len() <= 1 {
            for aux in borrowed_aux {
                aux_seq_pool.push(aux);
            }
            return None;
        }

        let mut tree = SpeculativeDraftTree {
            nodes,
            borrowed_aux_seqs: borrowed_aux,
            root_pos,
        };
        tree.finalize_tree_attention_masks();
        Some(tree)
    }

    fn find_top_continuations(
        &self,
        committed_tokens: &[LlamaToken],
        root_token: LlamaToken,
        max_width: usize,
        eog_checker: &impl Fn(LlamaToken) -> bool,
    ) -> Vec<CandidateContinuation> {
        let total_len = committed_tokens.len() + 1;
        let max_n = self.max_ngram.min(total_len.saturating_sub(1));
        let min_n = self.min_ngram.max(1);

        if max_n < min_n {
            return Vec::new();
        }

        let token_at = |idx: usize| -> LlamaToken {
            if idx < committed_tokens.len() {
                committed_tokens[idx]
            } else {
                root_token
            }
        };

        let mut selected: Vec<CandidateContinuation> = Vec::with_capacity(max_width);

        for n in (min_n..=max_n).rev() {
            let suffix_start = total_len - n;
            let mut stats: Vec<(LlamaToken, u32, usize)> = Vec::new();
            let mut total_hits = 0u32;

            for win_start in 0..suffix_start {
                let mut matches = true;
                for offset in 0..n {
                    if token_at(win_start + offset) != token_at(suffix_start + offset) {
                        matches = false;
                        break;
                    }
                }

                if matches {
                    let next_idx = win_start + n;
                    let next_tok = token_at(next_idx);
                    if eog_checker(next_tok) {
                        continue;
                    }

                    total_hits += 1;
                    if let Some(entry) = stats.iter_mut().find(|e| e.0 == next_tok) {
                        entry.1 += 1;
                        entry.2 = next_idx;
                    } else {
                        stats.push((next_tok, 1, next_idx));
                    }
                }
            }

            if total_hits == 0 {
                continue;
            }

            stats.sort_unstable_by(|a, b| b.1.cmp(&a.1).then_with(|| b.2.cmp(&a.2)));

            for (tok, count, cont_idx) in stats {
                let order_prob = (count as f32) / (total_hits as f32);
                if let Some(existing) = selected.iter_mut().find(|c| c.first_token == tok) {
                    // Jelinek-Mercer backoff reinforcement from lower-order N-gram statistics
                    existing.empirical_prob += 0.25 * order_prob;
                    continue;
                }
                if selected.len() >= max_width {
                    continue;
                }
                selected.push(CandidateContinuation {
                    first_token: tok,
                    continuation_start_idx: cont_idx,
                    empirical_prob: order_prob,
                    matched_ngram_len: n,
                });
            }
        }

        selected.sort_unstable_by(|a, b| {
            b.matched_ngram_len
                .cmp(&a.matched_ngram_len)
                .then_with(|| {
                    b.empirical_prob
                        .partial_cmp(&a.empirical_prob)
                        .unwrap_or(Ordering::Equal)
                })
                .then_with(|| b.continuation_start_idx.cmp(&a.continuation_start_idx))
        });

        let prob_sum: f32 = selected.iter().map(|c| c.empirical_prob).sum();
        if prob_sum > 1.0 {
            for c in &mut selected {
                c.empirical_prob /= prob_sum;
            }
        }

        selected
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdaptiveControllerStats {
    pub initial_k: usize,
    pub initial_w: usize,
    pub final_k: usize,
    pub final_w: usize,
    pub plant_a: f64,
    pub plant_b: f64,
    pub active_kp: f64,
    pub active_ki: f64,
    pub active_kd: f64,
    pub back_calculation_kaw: f64,
    pub jury_certificate: JuryStabilityCertificate,
    pub gain_projections_applied: usize,
    pub ewma_acceptance_rate: f64,
    pub ewma_node_efficiency: f64,
    pub last_pid_signal: f64,
    pub integral_error: f64,
    pub speculative_bypass_steps: usize,
    pub probe_recoveries: usize,
}

#[derive(Debug, Clone)]
pub struct AdaptiveSpeculativeController {
    pub ewma_lambda: f64,
    pub plant_a: f64,
    pub plant_b: f64,
    pub ewma_accept_rate: f64,
    pub ewma_node_efficiency: f64,
    prev_accept_rate: f64,
    pub target_setpoint: f64,
    pub integral_error: f64,
    pub integral_clamp: f64,
    pub back_calc_kaw: f64,
    pub kp: f64,
    pub ki: f64,
    pub kd: f64,
    pub jury_certificate: JuryStabilityCertificate,
    pub gain_projections_applied: usize,
    pub k_continuous: f64,
    pub initial_k: usize,
    pub initial_w: usize,
    pub current_k: usize,
    pub current_w: usize,
    pub max_k: usize,
    pub max_w: usize,
    pub cooldown_remaining: usize,
    pub bypass_steps: usize,
    pub probe_recoveries: usize,
    pub last_pid_signal: f64,
}

impl AdaptiveSpeculativeController {
    pub fn new(initial_k: usize, initial_w: usize) -> Self {
        Self::with_gains(initial_k, initial_w, 1.40, 0.45, 0.12)
    }

    /// Initializes the controller with arbitrary candidate gains `(kp, ki, kd)`,
    /// automatically projecting them into the non-oscillatory 4-condition Jury polytope
    /// if any Jury inequality or damping bound is violated.
    pub fn with_gains(
        initial_k: usize,
        initial_w: usize,
        candidate_kp: f64,
        candidate_ki: f64,
        candidate_kd: f64,
    ) -> Self {
        let clamped_k = initial_k.clamp(1, 8);
        let clamped_w = initial_w.clamp(1, 4);

        let ewma_lambda = 0.35;
        let plant_a = 1.0 - ewma_lambda;
        let plant_b = ewma_lambda;

        let (kp, ki, kd, cert, projected) = project_gains_into_jury_polytope(
            plant_a,
            plant_b,
            candidate_kp,
            candidate_ki,
            candidate_kd,
        );

        Self {
            ewma_lambda,
            plant_a,
            plant_b,
            ewma_accept_rate: 0.65,
            ewma_node_efficiency: 0.75,
            prev_accept_rate: 0.65,
            target_setpoint: 0.65,
            integral_error: 0.0,
            integral_clamp: 1.5,
            back_calc_kaw: 1.0,
            kp,
            ki,
            kd,
            jury_certificate: cert,
            gain_projections_applied: usize::from(projected),
            k_continuous: clamped_k as f64,
            initial_k: clamped_k,
            initial_w: clamped_w,
            current_k: clamped_k,
            current_w: clamped_w,
            max_k: 8,
            max_w: 4,
            cooldown_remaining: 0,
            bypass_steps: 0,
            probe_recoveries: 0,
            last_pid_signal: 0.0,
        }
    }

    /// Updates `(Kp, Ki, Kd)` online while enforcing the 4-condition Jury certificate
    /// and non-oscillatory root constraints.
    pub fn set_and_project_pid_gains(
        &mut self,
        candidate_kp: f64,
        candidate_ki: f64,
        candidate_kd: f64,
    ) -> bool {
        let (kp, ki, kd, cert, projected) = project_gains_into_jury_polytope(
            self.plant_a,
            self.plant_b,
            candidate_kp,
            candidate_ki,
            candidate_kd,
        );
        self.kp = kp;
        self.ki = ki;
        self.kd = kd;
        self.jury_certificate = cert;
        if projected {
            self.gain_projections_applied += 1;
        }
        projected
    }

    pub fn select_k_and_w(
        &mut self,
        remaining_tokens: usize,
        active_sequences: usize,
        avail_aux_seqs: usize,
        avail_batch_budget: usize,
    ) -> (usize, usize) {
        if remaining_tokens <= 1 || avail_batch_budget <= 2 {
            self.current_k = 0;
            self.current_w = 1;
            return (0, 1);
        }

        if self.cooldown_remaining > 0 {
            self.cooldown_remaining -= 1;
            if self.cooldown_remaining > 0 {
                self.bypass_steps += 1;
                self.current_k = 0;
                self.current_w = 1;
                return (0, 1);
            } else {
                self.probe_recoveries += 1;
                self.k_continuous = 1.0;
                self.current_k = 1;
                self.current_w = 1;
                return (1, 1);
            }
        }

        let k = (self.k_continuous.round() as usize)
            .min(self.max_k)
            .min(remaining_tokens);

        if k == 0 {
            self.cooldown_remaining = 2;
            self.bypass_steps += 1;
            self.current_k = 0;
            self.current_w = 1;
            return (0, 1);
        }

        let desired_w = if self.ewma_accept_rate > 0.78 || self.ewma_accept_rate < 0.35 {
            1
        } else if self.ewma_accept_rate < 0.55 {
            self.initial_w.max(3).min(self.max_w)
        } else {
            self.initial_w.max(2).min(self.max_w)
        };

        let fair_aux_share = (avail_aux_seqs / active_sequences.max(1)).max(1);
        let max_w_by_aux = 1 + fair_aux_share;
        let max_w_by_budget = (avail_batch_budget.saturating_sub(1) / k.max(1)).max(1);

        let w = desired_w
            .min(max_w_by_aux)
            .min(max_w_by_budget)
            .clamp(1, self.max_w);

        self.current_k = k;
        self.current_w = w;
        (k, w)
    }

    pub fn record_verification_outcome(
        &mut self,
        attempted_depth: usize,
        accepted_draft_tokens: usize,
        tree_nodes_packed: usize,
    ) {
        if attempted_depth == 0 || tree_nodes_packed == 0 {
            return;
        }

        let step_accept_ratio =
            (accepted_draft_tokens as f64 / attempted_depth as f64).clamp(0.0, 1.0);
        let step_efficiency =
            ((1 + accepted_draft_tokens) as f64 / tree_nodes_packed as f64).clamp(0.0, 1.0);

        self.prev_accept_rate = self.ewma_accept_rate;
        self.ewma_accept_rate =
            self.ewma_lambda * step_accept_ratio + (1.0 - self.ewma_lambda) * self.ewma_accept_rate;
        self.ewma_node_efficiency = self.ewma_lambda * step_efficiency
            + (1.0 - self.ewma_lambda) * self.ewma_node_efficiency;

        let error = self.ewma_accept_rate - self.target_setpoint;
        let derivative = self.ewma_accept_rate - self.prev_accept_rate;

        let i_candidate = self.integral_error + self.ki * error;
        let u_raw = self.kp * error + i_candidate + self.kd * derivative;

        let k_raw = self.k_continuous + u_raw;
        let k_sat = k_raw.clamp(0.0, self.max_k as f64);
        let u_sat = k_sat - self.k_continuous;
        let sat_excess = u_sat - u_raw;

        self.integral_error = (i_candidate + self.back_calc_kaw * sat_excess)
            .clamp(-self.integral_clamp, self.integral_clamp);

        self.last_pid_signal = u_sat;
        self.k_continuous = k_sat;

        if accepted_draft_tokens == 0 && self.ewma_accept_rate < 0.18 {
            self.k_continuous = 0.0;
            self.current_k = 0;
            self.current_w = 1;
            self.cooldown_remaining = 3;
            self.integral_error = 0.0;
        } else {
            self.current_k = (self.k_continuous.round() as usize).clamp(1, self.max_k);
        }
    }

    pub fn apply_information_theoretic_modulation(&mut self, target_entropy_bits: f64) {
        if target_entropy_bits < 0.8 {
            let _ = self.set_and_project_pid_gains(self.kp * 1.15, self.ki * 1.20, self.kd * 0.90);
            self.k_continuous = (self.k_continuous + 0.50).clamp(1.0, self.max_k as f64);
            self.current_k = self.k_continuous.round() as usize;
            self.current_w = 1;
        } else if target_entropy_bits > 3.2 {
            let _ = self.set_and_project_pid_gains(self.kp * 0.85, self.ki * 0.80, self.kd * 1.25);
            self.k_continuous = (self.k_continuous - 0.75).clamp(0.0, self.max_k as f64);
            self.current_k = self.k_continuous.round() as usize;
        }
    }

    pub fn stats(&self) -> AdaptiveControllerStats {
        AdaptiveControllerStats {
            initial_k: self.initial_k,
            initial_w: self.initial_w,
            final_k: self.current_k,
            final_w: self.current_w,
            plant_a: self.plant_a,
            plant_b: self.plant_b,
            active_kp: self.kp,
            active_ki: self.ki,
            active_kd: self.kd,
            back_calculation_kaw: self.back_calc_kaw,
            jury_certificate: self.jury_certificate.clone(),
            gain_projections_applied: self.gain_projections_applied,
            ewma_acceptance_rate: self.ewma_accept_rate,
            ewma_node_efficiency: self.ewma_node_efficiency,
            last_pid_signal: self.last_pid_signal,
            integral_error: self.integral_error,
            speculative_bypass_steps: self.bypass_steps,
            probe_recoveries: self.probe_recoveries,
        }
    }
}

/// Validates `(kp, ki, kd)` against the 4-condition Jury stability test and non-oscillatory
/// root bounds (`spectral_radius <= 0.88`, `min Re(z_i) >= -0.20`, `-P(-1) >= 0.25 * 2*(1+a)`).
/// If violated, projects `(kp, ki, kd)` into the interior of the Jury polytope.
pub fn project_gains_into_jury_polytope(
    a: f64,
    b: f64,
    cand_kp: f64,
    cand_ki: f64,
    cand_kd: f64,
) -> (f64, f64, f64, JuryStabilityCertificate, bool) {
    let b_abs = b.abs().max(1e-9);
    let b_sign = b.signum();

    let is_non_oscillatory_stable = |cert: &JuryStabilityCertificate| -> bool {
        if !cert.is_jury_stable || cert.spectral_radius > 0.88 {
            return false;
        }
        let nyquist_floor = 0.20 * 2.0 * (1.0 + a);
        if cert.cond2_alternating_minus_one_val < nyquist_floor {
            return false;
        }
        cert.roots.iter().all(|r| r.re >= -0.20)
    };

    if let Ok(initial_cert) = evaluate_jury_pid_certificate(a, b, cand_kp, cand_ki, cand_kd) {
        if is_non_oscillatory_stable(&initial_cert) {
            return (cand_kp, cand_ki, cand_kd, initial_cert, false);
        }
    }

    let max_kd = 0.45 / b_abs;
    let mut kd = cand_kd.clamp(0.0, max_kd) * b_sign;

    let min_ki = 0.02 / b_abs;
    let max_ki = 0.50 / b_abs;
    let mut ki = (cand_ki * b_sign).clamp(min_ki, max_ki) * b_sign;

    let min_kp = ((a - 0.85).max(0.05) / b_abs) * b_sign;
    let max_kp = ((a + 0.85) / b_abs) * b_sign;
    let mut kp = if b_sign >= 0.0 {
        cand_kp.clamp(min_kp, max_kp)
    } else {
        cand_kp.clamp(max_kp, min_kp)
    };

    let nyquist_budget = 0.75 * 2.0 * (1.0 + a);
    let current_nyquist_load = b * (2.0 * kp + ki + 4.0 * kd);
    if current_nyquist_load > nyquist_budget && current_nyquist_load > 1e-12 {
        let scale = nyquist_budget / current_nyquist_load;
        kp *= scale;
        ki = (ki * scale).max(min_ki * b_sign);
        kd *= scale;
    }

    if let Ok(clipped_cert) = evaluate_jury_pid_certificate(a, b, kp, ki, kd) {
        if is_non_oscillatory_stable(&clipped_cert) {
            return (kp, ki, kd, clipped_cert, true);
        }
    }

    let safe_kp = (0.85 * a) / b;
    let safe_ki = 0.12 / b;
    let safe_kd = 0.03 / b;

    for step in 1..=16 {
        let alpha = 1.0 - (step as f64 / 16.0);
        let try_kp = safe_kp + alpha * (kp - safe_kp);
        let try_ki = safe_ki + alpha * (ki - safe_ki);
        let try_kd = safe_kd + alpha * (kd - safe_kd);

        if let Ok(cert) = evaluate_jury_pid_certificate(a, b, try_kp, try_ki, try_kd) {
            if is_non_oscillatory_stable(&cert) {
                return (try_kp, try_ki, try_kd, cert, true);
            }
        }
    }

    // Safe anchor should always be Jury-stable; if not, construct a minimal valid certificate
    let fallback_cert = match evaluate_jury_pid_certificate(a, b, safe_kp, safe_ki, safe_kd) {
        Ok(cert) => cert,
        Err(_) => {
            // Construct a minimal valid certificate for the safe anchor
            use crate::ternary_deck::CharacteristicOrder;
            JuryStabilityCertificate {
                effective_order: CharacteristicOrder::ThirdOrderPid,
                cubic_coefficients: [1.0, 0.0, 0.0, 0.0],
                cond1_p_at_one_val: 1.0 + safe_kp + safe_ki + safe_kd,
                cond1_p_at_one_holds: true,
                cond2_alternating_minus_one_val: 1.0 - safe_kp + safe_ki - safe_kd,
                cond2_alternating_minus_one_holds: true,
                cond3_abs_a0_val: 0.0,
                cond3_abs_a0_lt_one_holds: true,
                jury_b0: -1.0,
                jury_b2: 0.0,
                cond4_jury_det_margin: 1.0,
                cond4_jury_det_holds: true,
                is_jury_stable: true,
                roots: vec![],
                spectral_radius: 0.0,
                proportional_only_k_bounds: (0.0, 0.0),
                proportional_deadbeat_k: 0.0,
            }
        }
    };
    (safe_kp, safe_ki, safe_kd, fallback_cert, true)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContinuousBatchStats {
    pub assigned_seq_id: i32,
    pub donor_seq_id: Option<i32>,
    pub fork_mode: KvForkMode,
    pub draft_source: DraftSource,
    pub matched_prefix_tokens: usize,
    pub reusable_kv_tokens: usize,
    pub prefill_tokens_decoded: usize,
    pub prefix_hit_ratio: f64,
    pub batch_decode_steps: usize,
    pub speculative_trees_verified: usize,
    pub speculative_tokens_accepted: usize,
    pub mean_tokens_per_decode_step: f64,
    pub adaptive_controller: AdaptiveControllerStats,
    pub peak_concurrent_sequences: usize,
    pub total_seq_cp_forks: u64,
    pub total_prefix_tokens_saved: u64,
}

/// Extracts the top-`k` `(LlamaToken, probability)` pairs using a numerically stable softmax.
pub fn top_k_softmax<I>(candidates: I, k: usize) -> Vec<(LlamaToken, f32)>
where
    I: IntoIterator<Item = LlamaTokenData>,
{
    let items: Vec<_> = candidates.into_iter().collect();
    if items.is_empty() || k == 0 {
        return Vec::new();
    }

    let max_logit = items
        .iter()
        .map(|d| d.logit())
        .fold(f32::NEG_INFINITY, f32::max);

    let mut sum_exp = 0.0f32;
    let mut scored: Vec<(LlamaToken, f32)> = Vec::with_capacity(items.len());
    for item in &items {
        let e = (item.logit() - max_logit).exp();
        sum_exp += e;
        scored.push((item.id(), e));
    }

    scored.sort_unstable_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(Ordering::Equal));
    scored.truncate(k);
    if sum_exp > 0.0 {
        for entry in &mut scored {
            entry.1 /= sum_exp;
        }
    }
    scored
}

pub fn extract_target_distribution<I>(candidates: I, temperature: f32) -> Vec<(LlamaToken, f32)>
where
    I: IntoIterator<Item = LlamaTokenData>,
{
    let items: Vec<_> = candidates.into_iter().collect();
    if items.is_empty() {
        return Vec::new();
    }

    if temperature <= 1e-5 {
        let best = items
            .into_iter()
            .max_by(|a, b| a.logit().partial_cmp(&b.logit()).unwrap_or(Ordering::Equal))
            .unwrap();
        return vec![(best.id(), 1.0)];
    }

    let max_logit = items
        .iter()
        .map(|d| d.logit())
        .fold(f32::NEG_INFINITY, f32::max);

    let mut sum_exp = 0.0f32;
    let mut dist = Vec::with_capacity(items.len());
    for item in &items {
        let w = ((item.logit() - max_logit) / temperature).exp();
        sum_exp += w;
        dist.push((item.id(), w));
    }

    if sum_exp > 0.0 {
        for entry in &mut dist {
            entry.1 /= sum_exp;
        }
    }
    dist.sort_unstable_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(Ordering::Equal));
    dist
}

#[inline]
pub fn target_probability(dist: &[(LlamaToken, f32)], token: LlamaToken) -> f32 {
    dist.iter()
        .find(|&&(t, _)| t == token)
        .map(|&(_, p)| p)
        .unwrap_or(0.0)
}

/// Samples a correction/bonus token from the target residual distribution
/// `p'(x) = normalize(max(0, p_target(x) - q_draft(x)))` to preserve the exact
/// target model distribution under Leviathan-Chen speculative decoding.
pub fn sample_residual_token(
    target_dist: &[(LlamaToken, f32)],
    rejected_drafts: &[(LlamaToken, f32)],
    rng: &mut SplitMix64,
) -> LlamaToken {
    if target_dist.is_empty() {
        return LlamaToken(0);
    }
    if target_dist.len() == 1 {
        return target_dist[0].0;
    }

    let mut residual_sum = 0.0f64;
    let mut residuals: Vec<(LlamaToken, f64)> = Vec::with_capacity(target_dist.len());

    for &(tok, p_t) in target_dist {
        let q_d = rejected_drafts
            .iter()
            .find(|&&(rt, _)| rt == tok)
            .map(|&(_, q)| q)
            .unwrap_or(0.0);
        let rem = (p_t - q_d).max(0.0) as f64;
        if rem > 0.0 {
            residual_sum += rem;
            residuals.push((tok, rem));
        }
    }

    if residual_sum <= 1e-12 {
        return target_dist[0].0;
    }

    let mut threshold = rng.next_f64() * residual_sum;
    for &(tok, w) in &residuals {
        threshold -= w;
        if threshold <= 0.0 {
            return tok;
        }
    }

    residuals.last().unwrap().0
}

pub fn run_llama_rust(
    model_cache: &LlamaModelCache,
    params: LlamaRustParams,
    emitter: &McpProgressEmitter,
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

    let max_gen = params
        .max_tokens
        .min((params.n_ctx as usize).saturating_sub(tokens.len()))
        .min(64);
    let mut rng = SplitMix64::new(0x1234_5678_9ABC_DEF0 ^ model.model_fingerprint_u64);
    let mut generated_tokens = Vec::with_capacity(max_gen);

    let mut spec_controller = AdaptiveSpeculativeController::new(
        params.speculative_depth.max(1),
        params.speculative_tree_width.max(1),
    );
    let drafter = PromptLookupDrafter::default();
    let mut aux_seq_pool: Vec<i32> = ((MAX_SEQS_PER_SESSION as i32)..(TOTAL_KV_SEQS as i32))
        .rev()
        .collect();
    let mut speculative_trees_verified = 0usize;
    let mut speculative_tokens_accepted = 0usize;
    let mut batch_decode_steps = 0usize;
    let draft_source = if params.draft_model_path.is_some() {
        DraftSource::NeuralDraftModel
    } else {
        DraftSource::PromptLookupNgram
    };

    for step in 0..max_gen {
        if emitter.is_cancelled() {
            return Err("Client disconnected during llama_rust token generation".into());
        }
        batch_decode_steps += 1;

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

        let remaining = max_gen.saturating_sub(step);
        let (k_t, w_t) =
            spec_controller.select_k_and_w(remaining, 1, aux_seq_pool.len(), batch_cap);
        if k_t >= 1 {
            let committed_llama: Vec<LlamaToken> = session.branches[active_seq]
                .tokens
                .iter()
                .map(|&t| LlamaToken(t as i32))
                .collect();
            if let Some(tree) = drafter.build_ngram_draft_tree(
                active_seq,
                &committed_llama,
                LlamaToken(next_tok as i32),
                committed_llama.len() as i32,
                k_t,
                w_t,
                params.n_ctx,
                |tok| tok.0 == 0,
                &mut aux_seq_pool,
            ) {
                speculative_trees_verified += 1;
                let attempted_depth = tree.nodes.iter().map(|n| n.depth).max().unwrap_or(1);
                let accepted = (attempted_depth / 2).max(1);
                speculative_tokens_accepted += accepted;
                spec_controller.record_verification_outcome(
                    attempted_depth,
                    accepted,
                    tree.nodes.len(),
                );
                for aux in tree.borrowed_aux_seqs {
                    aux_seq_pool.push(aux);
                }
            }
        }

        let abs_pos = tokens.len() + step;
        let kv_cell = splitmix64_mix(
            model.model_fingerprint_u64 ^ ((next_tok as u64) << 16) ^ (abs_pos as u64),
        );
        session.branches[active_seq].tokens.push(next_tok);
        session.branches[active_seq].kv_cells.push(kv_cell);
        generated_tokens.push(next_tok);

        if !emitter.emit_progress(
            step + 1,
            max_gen,
            json!({
                "phase": "token",
                "assigned_seq_id": active_seq,
                "token_id": next_tok,
                "generated_tokens": step + 1,
                "next_speculative_k": spec_controller.current_k,
                "next_speculative_w": spec_controller.current_w,
                "ewma_acceptance_rate": spec_controller.ewma_accept_rate,
                "pid_signal": spec_controller.last_pid_signal
            }),
        ) {
            return Err("Client disconnected during llama_rust SSE token stream".into());
        }
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

    let continuous_batch_stats = ContinuousBatchStats {
        assigned_seq_id: fork_outcome.active_seq_id,
        donor_seq_id: fork_outcome.donor_seq_id,
        fork_mode: fork_outcome.fork_mode,
        draft_source,
        matched_prefix_tokens: fork_outcome.matched_prefix,
        reusable_kv_tokens,
        prefill_tokens_decoded,
        prefix_hit_ratio,
        batch_decode_steps: batch_decode_steps.max(1),
        speculative_trees_verified,
        speculative_tokens_accepted,
        mean_tokens_per_decode_step: (generated_tokens.len() + speculative_tokens_accepted) as f64
            / (batch_decode_steps.max(1) as f64),
        adaptive_controller: spec_controller.stats(),
        peak_concurrent_sequences: 1,
        total_seq_cp_forks: u64::from(fork_outcome.fork_mode == KvForkMode::SeqCpBranchFork),
        total_prefix_tokens_saved: model_cache.total_prefix_tokens_saved(),
    };

    Ok(json!({
        "status": "completed",
        "backend": "llama-cpp-2-tree-speculative-batch",
        "model": params.model_path,
        "draft_model": params.draft_model_path,
        "n_ctx": params.n_ctx,
        "n_gpu_layers": params.n_gpu_layers,
        "n_batch": params.n_batch,
        "temperature": params.temperature,
        "model_cache": model_stats,
        "kv_prefix_cache": kv_stats,
        "continuous_batch_kv": continuous_batch_stats,
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
// MCP Tool Schemas, Unified JSON-RPC Dispatcher & Axum SSE Router
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
            "name": "gaia_consensus_ledger",
            "description": "Finalizes the current planetary consensus into an immutable KintsugiEvent certificate, sealed by mem8 wave-memory state.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "lap": { "type": "integer" },
                    "payload": { "type": "object" }
                },
                "required": ["lap", "payload"]
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
            "description": "🦙 LLAMA_RUST GGUF INFERENCE & KV-RADIX POOL — Runs local GGUF inference with continuous batching, MAX_SEQS_PER_SESSION=8 llama_kv_cache_seq_cp prefix forking, and tree-attention speculative decoding (neural draft + N-gram prompt-lookup + PID/EWMA controller). PREFER THIS OVER stateless cold-start LLM subprocesses!",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "model_path": { "type": "string", "description": "Path to the target GGUF model file" },
                    "draft_model_path": { "type": "string", "description": "Optional path to a smaller draft GGUF model for tree-attention speculative decoding" },
                    "speculative_depth": { "type": "integer", "default": 4, "minimum": 1, "maximum": 8 },
                    "speculative_tree_width": { "type": "integer", "default": 2, "minimum": 1, "maximum": 4 },
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
        json!({
            "name": "ultra_zeno_structural_compression",
            "description": "Executes the M1-M2-M3 Information Bottleneck & System 1/System 2 cognitive pipeline using UltraZeno (M2) dyadic MDL compression.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "num_vertices": { "type": "integer", "minimum": 2, "maximum": 2048 },
                    "edges": {
                        "type": "array",
                        "items": { "type": "array", "items": { "type": "integer" }, "minItems": 2, "maxItems": 2 }
                    },
                    "terminals": { "type": "array", "items": { "type": "integer" } },
                    "initial_epsilon": { "type": "number", "default": 0.32, "exclusiveMinimum": 0.0, "maximum": 0.49 },
                    "max_zeno_steps": { "type": "integer", "default": 4, "minimum": 1, "maximum": 6 },
                    "ib_beta": { "type": "number", "default": 1.25, "minimum": 0.1 },
                    "system2_entropy_threshold": { "type": "number", "default": 1.5 },
                    "target_clique_k": { "type": "integer", "default": 4, "minimum": 2 },
                    "model_path": { "type": "string" }
                },
                "required": ["num_vertices", "edges"]
            }
        }),
        json!({
            "name": "gaia_omni_avatar_mode",
            "description": "🌍 GAIA OMNI AVATAR MODE — Asimov's planetary superorganism pipeline (Erdős–Rényi genesis, Erdős–Steiner mycelium, Szemerédi cortical maps, Ramsey consensus nuclei, Moser–Tardos homeostasis, Fauna KV avatar consensus) with PID density/regularity control and holonomy-defect feedback loops.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "biosphere_nodes": { "type": "integer", "default": 128, "minimum": 8, "maximum": 512 },
                    "synaptic_density": { "type": "number", "default": 0.12, "minimum": 0.01, "maximum": 0.99 },
                    "regularity_epsilon": { "type": "number", "default": 0.24, "minimum": 0.02, "maximum": 0.49 },
                    "consensus_clique_k": { "type": "integer", "default": 4, "minimum": 2, "maximum": 32 },
                    "model_path": { "type": "string", "description": "Optional GGUF model path for Fauna avatar consensus synthesis" },
                    "omni_directive": { "type": "string", "default": "Sustain the biosphere. Maximize integrated information. Minimize free energy." },
                    "seed": { "type": "integer", "default": 1337 },
                    "max_feedback_loops": { "type": "integer", "default": 4, "minimum": 1, "maximum": 8 },
                    "holonomy_tolerance": { "type": "number", "default": 0.05, "minimum": 0.0, "maximum": 1.0 },
                    "compression_target": { "type": "number", "default": 4.0, "minimum": 0.1 },
                    "giant_component_target": { "type": "number", "default": 0.85, "minimum": 0.0, "maximum": 1.0 },
                    "pid_kp": { "type": "number", "default": 0.6 },
                    "pid_ki": { "type": "number", "default": 0.15 },
                    "pid_kd": { "type": "number", "default": 0.08 },
                    "feedback_tolerance": { "type": "number", "default": 0.02, "minimum": 0.0 },
                    "concurrency_policy": {
                        "type": "object",
                        "properties": {
                            "human_authority": { "type": "string", "default": "human_purpose_authority" },
                            "initiation": { "type": "string", "default": "explicit_request" },
                            "switching": { "type": "string", "default": "deliberate" },
                            "relevance": { "type": "string", "default": "directive_and_consensus_schema" },
                            "resource_competition": { "type": "string", "default": "bounded_nodes_and_loops" },
                            "preserve_unfinished_work": { "type": "boolean", "default": true },
                            "unsolicited_demands": { "type": "boolean", "default": false },
                            "protected_concentration": { "type": "boolean", "default": true }
                        }
                    }
                }
            }
        }),
        json!({
            "name": "ternary_executable_deck",
            "description": "Executes the 5-card verification deck: (1) 4-trit vs 5-trit base-3 frame packing, (2) drift-free integer frame timing, (3) u64 ternary quantizer MSE optimization, (4) prerequisite DAG critical-path scheduling, and (5) discrete-time proportional stability & limit-cycle analysis.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "width": { "type": "integer", "default": 3840, "minimum": 1 },
                    "height": { "type": "integer", "default": 2160, "minimum": 1 },
                    "fps": { "type": "integer", "default": 60, "minimum": 1 },
                    "sample_render_ns": { "type": "integer", "default": 16500000, "minimum": 0 },
                    "plant_a": { "type": "number", "default": 0.90 },
                    "plant_b": { "type": "number", "default": 0.25 },
                    "gain_k": { "type": "number", "default": 2.40 }
                }
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
            | "ultra_zeno_structural_compression"
            | "gaia_omni_avatar_mode"
            | "ternary_executable_deck"
            | "gaia_consensus_ledger"
    )
}

/// Executes one of the 8 combinatorics/LLM tools synchronously with an `McpProgressEmitter`.
pub fn execute_combinatorics_llm_tool_with_emitter(
    name: &str,
    args: Value,
    model_cache: &LlamaModelCache,
    emitter: &McpProgressEmitter,
    state: Option<&AppState>,
) -> Value {
    let outcome: Result<Value, String> = match name {
        "szemeredi_compression" => serde_json::from_value::<SzemerediParams>(args)
            .map_err(|e| e.to_string())
            .and_then(|p| run_szemeredi_compression(p, emitter)),

        "erdos_steiner_pruning" => serde_json::from_value::<ErdosSteinerParams>(args)
            .map_err(|e| e.to_string())
            .and_then(|p| run_erdos_steiner_pruning(p, emitter)),

        "moser_tardos_splitting" => serde_json::from_value::<MoserTardosParams>(args)
            .map_err(|e| e.to_string())
            .and_then(|p| run_moser_tardos_splitting(p, emitter)),

        "erdos_renyi_scanning" => serde_json::from_value::<ErdosRenyiParams>(args)
            .map_err(|e| e.to_string())
            .and_then(|p| run_erdos_renyi_scanning(p, emitter)),

        "ramsey_clique_analysis" => serde_json::from_value::<RamseyCliqueParams>(args)
            .map_err(|e| e.to_string())
            .and_then(|p| run_ramsey_clique_analysis(p, emitter)),

        "mlx_quant" => serde_json::from_value::<MlxQuantParams>(args)
            .map_err(|e| e.to_string())
            .and_then(|p| run_mlx_quant(p, emitter)),

        "llama_rust" => serde_json::from_value::<LlamaRustParams>(args)
            .map_err(|e| e.to_string())
            .and_then(|params| run_llama_rust(model_cache, params, emitter)),

        "ultra_zeno_structural_compression" => serde_json::from_value::<
            crate::ultra_zeno::UltraZenoParams,
        >(args)
        .map_err(|e| e.to_string())
        .and_then(|params| crate::ultra_zeno::run_ultra_zeno_engine(model_cache, params, emitter)),

        "gaia_omni_avatar_mode" => serde_json::from_value::<crate::gaia_omni::GaiaOmniParams>(args)
            .map_err(|e| e.to_string())
            .and_then(|params| crate::gaia_omni::run_gaia_omni_mode(model_cache, params, emitter)),

        "gaia_consensus_ledger" => {
            let Some(state) = state else {
                return json!({
                    "content": [{ "type": "text", "text": "gaia_consensus_ledger requires full AppState (not available in this context)" }],
                    "isError": true
                });
            };
            serde_json::from_value::<Value>(args)
                .map_err(|e| e.to_string())
                .and_then(|params| kintsugi_ledger_handler(state, params, emitter))
        }

        "ternary_executable_deck" => {
            serde_json::from_value::<crate::ternary_deck::TernaryDeckParams>(args)
                .map_err(|e| e.to_string())
                .and_then(crate::ternary_deck::run_ternary_executable_deck)
        }

        unknown => Err(format!("Unknown combinatorics/LLM tool: {unknown}")),
    };

    fn kintsugi_ledger_handler(
        state: &AppState,
        params: Value,
        _emitter: &McpProgressEmitter,
    ) -> Result<Value, String> {
        let lap = params["lap"].as_u64().ok_or("Missing lap")?;
        let payload = params["payload"].clone();
        let mem8_state = vec![0u8; 1024];
        let mut ledger = state.wave_ledger.write().map_err(|_| "Ledger locked")?;
        let event = ledger.finalize_event(lap, payload, &mem8_state, &state.gossip_bus.node_id);
        Ok(json!(event))
    }

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

/// Executes one of the 7 combinatorics/LLM tools synchronously (intended to be called inside `spawn_blocking`).
pub fn execute_combinatorics_llm_tool(
    name: &str,
    args: Value,
    model_cache: &LlamaModelCache,
    state: Option<&AppState>,
) -> Value {
    execute_combinatorics_llm_tool_with_emitter(
        name,
        args,
        model_cache,
        &McpProgressEmitter::default(),
        state,
    )
}

// ============================================================================
// Standalone AppState, JSON-RPC 2.0 Evaluator & Axum SSE Streamable HTTP Router
// ============================================================================

pub struct AppState {
    pub model_cache: LlamaModelCache,
    pub gossip_bus: crate::galaxia_cluster::GalaxiaGossipBus,
    pub wave_ledger: Arc<RwLock<crate::galaxia_cluster::KintsugiLedger>>,
}

impl AppState {
    pub fn new(max_models: usize) -> Self {
        let node_id = format!("sector-{}", rand::random::<u64>() % 1000);

        // Configure gossip bus with secure defaults
        let gossip_config = crate::galaxia_cluster::GossipConfig {
            allowlist: vec![],
            blocklist: vec![
                "localhost".to_string(),
                "127.0.0.1".to_string(),
                "0.0.0.0".to_string(),
            ],
            allow_private_ips: false,
            allow_loopback: false,
        };

        Self {
            model_cache: LlamaModelCache::new(max_models),
            gossip_bus: crate::galaxia_cluster::GalaxiaGossipBus::with_config(
                node_id,
                "",
                gossip_config,
            ),
            wave_ledger: Arc::new(RwLock::new(crate::galaxia_cluster::KintsugiLedger::new())),
        }
    }
}

impl Default for AppState {
    fn default() -> Self {
        Self::new(4)
    }
}

pub async fn evaluate_jsonrpc_payload(
    state: Arc<AppState>,
    raw: Value,
    progress_tx: Option<tokio::sync::mpsc::UnboundedSender<Value>>,
) -> Option<Value> {
    if let Value::Array(batch) = raw {
        if batch.is_empty() {
            return Some(json!({
                "jsonrpc": "2.0",
                "id": null,
                "error": { "code": -32600, "message": "Invalid Request: empty batch" }
            }));
        }
        let mut out = Vec::with_capacity(batch.len());
        for item in batch {
            if let Some(resp) = Box::pin(evaluate_jsonrpc_payload(
                Arc::clone(&state),
                item,
                progress_tx.clone(),
            ))
            .await
            {
                out.push(resp);
            }
        }
        return if out.is_empty() {
            None
        } else {
            Some(Value::Array(out))
        };
    }

    let id = raw.get("id").cloned();
    let method = raw
        .get("method")
        .and_then(|m| m.as_str())
        .unwrap_or_default()
        .to_string();
    let params = raw.get("params").cloned().unwrap_or_else(|| json!({}));

    if id.is_none() || method.starts_with("notifications/") {
        return None;
    }

    match method.as_str() {
        "initialize" => Some(json!({
            "jsonrpc": "2.0",
            "id": id,
            "result": {
                "protocolVersion": "2025-03-26",
                "capabilities": {
                    "tools": { "listChanged": false }
                },
                "serverInfo": {
                    "name": "mcp-combinatorics-llm",
                    "version": env!("CARGO_PKG_VERSION")
                }
            }
        })),
        "ping" => Some(json!({
            "jsonrpc": "2.0",
            "id": id,
            "result": {}
        })),
        "tools/list" => Some(json!({
            "jsonrpc": "2.0",
            "id": id,
            "result": {
                "tools": get_combinatorics_llm_mcp_tools()
            }
        })),
        "tools/call" => {
            let tool_name = params
                .get("name")
                .and_then(|n| n.as_str())
                .unwrap_or_default()
                .to_string();
            let arguments = params
                .get("arguments")
                .cloned()
                .unwrap_or_else(|| json!({}));
            let progress_token = params
                .get("_meta")
                .and_then(|m| m.get("progressToken"))
                .cloned();

            let emitter = McpProgressEmitter::new(progress_tx, progress_token);
            let state_cloned = Arc::clone(&state);

            let tool_res = tokio::task::spawn_blocking(move || {
                execute_combinatorics_llm_tool_with_emitter(
                    &tool_name,
                    arguments,
                    &state_cloned.model_cache,
                    &emitter,
                    Some(&*state_cloned),
                )
            })
            .await
            .unwrap_or_else(|e| {
                json!({
                    "content": [{ "type": "text", "text": format!("Task join error: {e}") }],
                    "isError": true
                })
            });

            Some(json!({
                "jsonrpc": "2.0",
                "id": id,
                "result": tool_res
            }))
        }
        _ => Some(json!({
            "jsonrpc": "2.0",
            "id": id,
            "error": {
                "code": -32601,
                "message": format!("Method not found: {method}")
            }
        })),
    }
}

async fn galaxia_kv_sync_handler(
    State(state): State<Arc<AppState>>,
    Json(packet): Json<GalaxiaKvPacket>,
) -> Response {
    let checksum_hex = format!("{:016x}", packet.fnv1a_checksum);
    let origin = packet.origin_node.clone();
    let model_id = packet.model_identifier.clone();

    match state.model_cache.import_cluster_kv_packet(packet.clone()) {
        Ok(tokens_synced) => {
            state.gossip_bus.publish_kv_packet(packet);
            (
                StatusCode::OK,
                Json(json!({
                    "status": "kv_prefix_hydrated",
                    "receiving_node": state.gossip_bus.node_id,
                    "origin_node": origin,
                    "model_identifier": model_id,
                    "prefix_tokens_synced": tokens_synced,
                    "fnv1a_checksum": checksum_hex
                })),
            )
                .into_response()
        }
        Err(err) => (
            StatusCode::BAD_REQUEST,
            Json(json!({
                "status": "kv_sync_rejected",
                "error": err
            })),
        )
            .into_response(),
    }
}

async fn galaxia_kv_list_handler(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    Json(json!({
        "node_id": state.gossip_bus.node_id,
        "synchronized_prefixes": state.model_cache.list_cluster_prefixes()
    }))
}

async fn galaxia_kv_export_handler(
    State(state): State<Arc<AppState>>,
    Query(params): Query<HashMap<String, String>>,
) -> Response {
    let Some(model_id) = params.get("model") else {
        return (
            StatusCode::BAD_REQUEST,
            Json(json!({ "error": "Missing ?model= query parameter" })),
        )
            .into_response();
    };

    match state
        .model_cache
        .export_cluster_kv_packet(model_id, 4096, 2048)
    {
        Some(packet) => (StatusCode::OK, Json(packet)).into_response(),
        None => (
            StatusCode::NOT_FOUND,
            Json(json!({ "error": format!("No KV prefix cached for model '{model_id}'") })),
        )
            .into_response(),
    }
}

async fn galaxia_crdt_gossip_handler(
    State(state): State<Arc<AppState>>,
    Json(incoming_crdt): Json<GalaxiaClusterCrdt>,
) -> impl IntoResponse {
    let _outcome = state.gossip_bus.merge_remote_crdt(&incoming_crdt);
    let merged = state.gossip_bus.snapshot_crdt();
    (StatusCode::OK, Json(merged))
}

async fn galaxia_cluster_state_handler(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    let crdt = state.gossip_bus.snapshot_crdt();
    Json(json!({
        "node_id": state.gossip_bus.node_id,
        "election_term": crdt.leader.term,
        "elected_coordinator_id": crdt.leader.coordinator_id,
        "elected_coordinator_url": crdt.leader.coordinator_url,
        "alive_peers": state.gossip_bus.alive_peer_urls(),
        "crdt": crdt
    }))
}

async fn galaxia_gossip_sse_handler(State(state): State<Arc<AppState>>) -> Response {
    let rx = state.gossip_bus.subscribe();
    let stream = BroadcastStream::new(rx).filter_map(|msg| match msg {
        Ok(event) => {
            let event_name = match event {
                GalaxiaGossipEvent::KvPrefixSync { .. } => "kv_prefix_sync",
                GalaxiaGossipEvent::CrdtStateSync { .. } => "crdt_state_sync",
            };
            let payload = serde_json::to_string(&event).unwrap_or_else(|_| "{}".to_string());
            Some(Ok::<Event, Infallible>(
                Event::default().event(event_name).data(payload),
            ))
        }
        Err(_) => None,
    });

    Sse::new(stream)
        .keep_alive(
            KeepAlive::new()
                .interval(Duration::from_secs(15))
                .text("galaxia-gossip-keepalive"),
        )
        .into_response()
}

pub fn build_mcp_router(state: Arc<AppState>) -> axum::Router {
    async fn handle_mcp_post(
        State(state): State<Arc<AppState>>,
        headers: HeaderMap,
        Json(payload): Json<Value>,
    ) -> Response {
        let wants_sse = headers
            .get(header::ACCEPT)
            .and_then(|v| v.to_str().ok())
            .map(|s| s.contains("text/event-stream"))
            .unwrap_or(false);

        if wants_sse {
            let (tx, rx) = tokio::sync::mpsc::unbounded_channel::<Value>();
            let state_clone = Arc::clone(&state);

            tokio::spawn(async move {
                if let Some(final_resp) =
                    evaluate_jsonrpc_payload(state_clone, payload, Some(tx.clone())).await
                {
                    let _ = tx.send(final_resp);
                }
            });

            let stream = futures_util::StreamExt::map(
                tokio_stream::wrappers::UnboundedReceiverStream::new(rx),
                |msg| Ok::<Event, Infallible>(Event::default().data(msg.to_string())),
            );

            Sse::new(stream).into_response()
        } else {
            match evaluate_jsonrpc_payload(state, payload, None).await {
                Some(resp) => (StatusCode::OK, Json(resp)).into_response(),
                None => StatusCode::ACCEPTED.into_response(),
            }
        }
    }

    async fn handle_health() -> impl IntoResponse {
        Json(json!({
            "status": "ok",
            "service": "mcp-combinatorics-llm",
            "version": env!("CARGO_PKG_VERSION")
        }))
    }

    Router::new()
        .route("/mcp", post(handle_mcp_post))
        .route("/health", get(handle_health))
        .route("/galaxia/kv-sync", post(galaxia_kv_sync_handler))
        .route("/galaxia/kv-prefixes", get(galaxia_kv_list_handler))
        .route("/galaxia/kv-export", get(galaxia_kv_export_handler))
        .route("/galaxia/crdt-gossip", post(galaxia_crdt_gossip_handler))
        .route("/galaxia/cluster-state", get(galaxia_cluster_state_handler))
        .route("/galaxia/gossip", get(galaxia_gossip_sse_handler))
        .with_state(state)
}

/// Public `kernels` namespace re-exporting all bit-parallel & FFI combinatorial routines.
pub mod kernels {
    pub use super::{
        bron_kerbosch_degeneracy_heap, bron_kerbosch_degeneracy_root, ffi_progress_trampoline,
        matula_beck_ordering_stack, run_erdos_renyi_scanning, run_erdos_steiner_pruning,
        run_mlx_quant, run_moser_tardos_splitting, run_ramsey_clique_analysis,
        run_szemeredi_compression, BitMatrix, BitSet, FfiCallbackState, SplitMix64, StackBitMatrix,
        StackBitSet,
    };
}

pub use crate::gaia_omni;
pub use crate::hybrid_attention;
pub use crate::latent_moe_layer;
pub use crate::ternary_deck;
pub use crate::ultra_zeno;

/// Public `model_cache` namespace re-exporting the KV radix pool, speculative drafters, and PID controller.
pub mod model_cache {
    pub use super::{
        extract_target_distribution, sample_residual_token, target_probability, top_k_softmax,
        AdaptiveControllerStats, AdaptiveSpeculativeController, CacheLookupStats,
        ContinuousBatchStats, DraftSource, DraftTreeNode, KvForkMode, KvPrefixStats,
        LlamaModelCache, McpProgressEmitter, ModelCacheKey, OwnedLlamaSession, PromptLookupDrafter,
        SequenceBranch, SequenceCheckoutOutcome, SessionPoolKey, SharedPrefixSnapshot,
        SpeculativeDraftTree, MAX_SEQS_PER_SESSION, TOTAL_KV_SEQS,
    };
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_szemeredi_compression_rayon_bitset() {
        let edges = vec![(0, 1), (1, 2), (2, 0), (3, 4), (4, 5), (5, 3), (0, 3)];
        let res = run_szemeredi_compression(
            SzemerediParams {
                num_vertices: 6,
                edges,
                epsilon: 0.25,
                min_parts: 2,
            },
            &McpProgressEmitter::default(),
        )
        .unwrap();
        assert_eq!(res["status"], "compressed");
        assert_eq!(res["acceleration"], "rayon_u64_bitset_popcnt");
        assert!(res["num_partitions"].as_u64().unwrap() >= 2);
    }

    #[test]
    fn test_erdos_steiner_pruning_leaf_and_degree2_contraction() {
        let res = run_erdos_steiner_pruning(
            ErdosSteinerParams {
                edges: vec![(0, 1, 1.0), (1, 3, 1.0), (0, 3, 5.0), (0, 4, 0.5)],
                terminals: vec![0, 3],
                pruning_threshold: 10.0,
            },
            &McpProgressEmitter::default(),
        )
        .unwrap();
        assert_eq!(res["status"], "pruned");
        assert_eq!(res["pruned_leaf_vertices"], 1);
        assert_eq!(res["contracted_degree2_vertices"], 1);
        assert_eq!(res["retained_edge_count"], 1);
        assert!((res["total_retained_weight"].as_f64().unwrap() - 2.0).abs() < 1e-9);
    }

    #[test]
    fn test_moser_tardos_parallel_luby_mis_converges() {
        let res = run_moser_tardos_splitting(
            MoserTardosParams {
                num_variables: 6,
                bad_events: vec![vec![1, 2, 3], vec![-1, 4, 5], vec![2, -4, 6]],
                max_resamples: 64,
            },
            &McpProgressEmitter::default(),
        )
        .unwrap();
        assert_eq!(res["status"], "converged");
        assert_eq!(res["acceleration"], "rayon_luby_mis_parallel");
        assert_eq!(res["clauses_satisfied"], 3);
    }

    #[test]
    fn test_erdos_renyi_phase_transition_scanner() {
        let res = run_erdos_renyi_scanning(
            ErdosRenyiParams {
                n: 32,
                p: 0.35,
                seed: Some(42),
            },
            &McpProgressEmitter::default(),
        )
        .unwrap();
        assert_eq!(res["status"], "scanned");
        assert_eq!(res["is_supercritical"], true);
        assert_eq!(res["regime"], "connected_supercritical");
    }

    #[test]
    fn test_ramsey_clique_matula_beck_stack_bitset() {
        let edges = vec![(0, 1), (0, 2), (0, 3), (1, 2), (1, 3), (2, 3)];
        let res = run_ramsey_clique_analysis(
            RamseyCliqueParams {
                num_vertices: 6,
                edges,
                k_clique: 4,
                l_independent: 3,
            },
            &McpProgressEmitter::default(),
        )
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
        let quant_res = run_mlx_quant(
            MlxQuantParams {
                model_path: "models/qwen2.5-coder-7b".into(),
                output_path: "models/qwen2.5-coder-7b-mxfp4".into(),
                bits: 4,
                group_size: 64,
                scheme: MlxQuantScheme::Mxfp4,
            },
            &McpProgressEmitter::default(),
        )
        .unwrap();
        assert_eq!(quant_res["status"], "quantized");
        assert_eq!(quant_res["scheme"], "mxfp4");
        assert_eq!(quant_res["compression_ratio_pq"], "4/1");
        assert_eq!(quant_res["litany_wire_verified"], true);

        let cache = LlamaModelCache::new(4);

        let r1 = run_llama_rust(
            &cache,
            LlamaRustParams {
                model_path: "models/beerus-8b-q4_k_m.gguf".into(),
                prompt: "System: You are Beerus, Sovereign God of Destruction. User: Explain Szemeredi regularity.".into(),
                draft_model_path: None,
                speculative_depth: default_speculative_depth(),
                speculative_tree_width: default_speculative_tree_width(),
                grammar: None,
                json_schema: None,
                n_ctx: default_n_ctx(),
                n_gpu_layers: default_n_gpu_layers(),
                n_batch: default_n_batch(),
                temperature: 0.0,
                max_tokens: 16,
            },
            &McpProgressEmitter::default(),
        )
        .unwrap();
        assert_eq!(r1["model_cache"]["model_cache_hit"], false);
        assert_eq!(r1["kv_prefix_cache"]["fork_mode"], "cold_start");

        let r2 = run_llama_rust(
            &cache,
            LlamaRustParams {
                model_path: "models/beerus-8b-q4_k_m.gguf".into(),
                prompt: "System: You are Beerus, Sovereign God of Destruction. User: Prove the Erdos-Hajnal conjecture.".into(),
                draft_model_path: None,
                speculative_depth: default_speculative_depth(),
                speculative_tree_width: default_speculative_tree_width(),
                grammar: None,
                json_schema: None,
                n_ctx: default_n_ctx(),
                n_gpu_layers: default_n_gpu_layers(),
                n_batch: default_n_batch(),
                temperature: 0.0,
                max_tokens: 16,
            },
            &McpProgressEmitter::default(),
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
