//! Core data structures for Szemerédi Regularity Compression ("MEM8 Book Mode")
//! and Paul Erdős Extremal / Probabilistic Graph Analysis.

use crate::scanner::{FileCategory, FileNode};
use serde::{Deserialize, Serialize};

/// Number of `f32` lanes (`64`) in a native MEM8 semantic wave embedding.
pub const WAVE_EMBEDDING_LANES: usize = 64;

/// Floor (`1e-9`) on global baseline graph density $p = |E| / |V|^2$.
pub const AMBIENT_DENSITY_FLOOR_P: f64 = 1e-9;

/// SplitMix64 golden ratio constant (`0x9E37_79B9_7F4A_7C15`) for deterministic wave seeding.
pub const SPLITMIX64_SEED_MUL: u64 = 0x9E37_79B9_7F4A_7C15;

/// FNV-1a 64-bit offset basis (`0xCBF2_9CE4_8422_2325`).
pub const FNV1A_64_OFFSET_BASIS: u64 = 0xCBF2_9CE4_8422_2325;

/// FNV-1a 64-bit prime (`0x0100_0000_01B3`).
pub const FNV1A_64_PRIME: u64 = 0x0100_0000_01B3;

/// Maximum directory tree depth slots (`128`) tracked during `from_file_nodes`.
pub const MAX_DIRECTORY_DEPTH_SLOTS: usize = 128;

/// Default regularity $\varepsilon$ (`0.10`) for an empty `SzemerediPartition`.
pub const DEFAULT_EMPTY_PARTITION_EPSILON: f64 = 0.10;

/// Default initial regularity $\varepsilon$ (`0.12`) for directory-seeded partitions.
pub const DEFAULT_INITIAL_PARTITION_EPSILON: f64 = 0.12;

/// Byte length (`32`) of the `Mem8BookArchive` model digest.
pub const ARCHIVE_DIGEST_BYTES: usize = 32;

/// Compact 32-bit identifier for symbols/files in `~/.st/symbols/`
pub type SymbolId = u32;
/// Identifier for a Szemerédi partition cell `V_i` (`0` is reserved for the outlier set `V_0`)
pub type ClusterId = u16;

/// Edge categories extracted by `smart-tree`'s AST and directory graph engine
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum EdgeKind {
    Import = 0,
    CallRef = 1,
    TraitImpl = 2,
    TypeBound = 3,
    MutableShared = 4,
}

impl EdgeKind {
    #[inline]
    pub const fn from_u8(v: u8) -> Self {
        match v {
            0 => Self::Import,
            1 => Self::CallRef,
            2 => Self::TraitImpl,
            3 => Self::TypeBound,
            _ => Self::MutableShared,
        }
    }
}

/// Paul Erdős lexicon classification for codebase nodes (`--mode the-book`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ErdosLexiconBadge {
    /// Clean, acyclic, well-balanced module ("Straight from The Book")
    StraightFromTheBook,
    /// Small utility helper script/file ("Epsilon")
    Epsilon,
    /// Orphaned / ignored / empty dead artifact ("Legally Dead (L.D.)")
    LegallyDead,
    /// High-discrepancy god-object or deeply nested monolith ("Pure Noise / S.F.")
    PureNoiseMonolith,
}

/// Zero-allocation CSR (Compressed Sparse Row) representation of the AST Symbol Graph.
/// Keeps both forward (out-edges) and transpose (in-edges) for `O(e(V_i, V_j))` power iteration.
#[derive(Debug, Clone)]
pub struct CsrSymbolGraph {
    pub num_vertices: usize,
    pub num_edges: usize,
    /// Global baseline density `p = |E| / |V|^2`
    pub ambient_density_p: f64,
    /// Forward CSR offsets: `length = num_vertices + 1`
    pub row_ptr: Vec<u32>,
    /// Forward CSR column targets: `length = num_edges`
    pub col_idx: Vec<SymbolId>,
    /// Edge weights / kinds
    pub edge_kind: Vec<EdgeKind>,
    /// Transpose CSR offsets (for `M^T * u` in singular value iteration)
    pub transpose_row_ptr: Vec<u32>,
    pub transpose_col_idx: Vec<SymbolId>,
    /// Native MEM8 semantic vectors per symbol (L2-normalized 64-lane wave embeddings)
    pub symbol_embeddings: Vec<[f32; 64]>,
}

impl CsrSymbolGraph {
    /// Builds a forward + transpose CSR graph from a list of directed `(u, v, EdgeKind)` edges.
    pub fn from_edges(
        num_vertices: usize,
        edges: &[(SymbolId, SymbolId, EdgeKind)],
        symbol_embeddings: Vec<[f32; 64]>,
    ) -> Self {
        let mut out_deg = vec![0u32; num_vertices];
        let mut in_deg = vec![0u32; num_vertices];
        let mut valid_edges = Vec::with_capacity(edges.len());

        for &(u, v, kind) in edges {
            if (u as usize) < num_vertices && (v as usize) < num_vertices && u != v {
                out_deg[u as usize] += 1;
                in_deg[v as usize] += 1;
                valid_edges.push((u, v, kind));
            }
        }

        let num_edges = valid_edges.len();
        let mut row_ptr = Vec::with_capacity(num_vertices + 1);
        row_ptr.push(0);
        for &d in &out_deg {
            let last = *row_ptr.last().unwrap_or(&0);
            row_ptr.push(last + d);
        }

        let mut transpose_row_ptr = Vec::with_capacity(num_vertices + 1);
        transpose_row_ptr.push(0);
        for &d in &in_deg {
            let last = *transpose_row_ptr.last().unwrap_or(&0);
            transpose_row_ptr.push(last + d);
        }

        let mut col_idx = vec![0u32; num_edges];
        let mut edge_kind = vec![EdgeKind::Import; num_edges];
        let mut out_cursor = row_ptr[..num_vertices].to_vec();

        let mut transpose_col_idx = vec![0u32; num_edges];
        let mut in_cursor = transpose_row_ptr[..num_vertices].to_vec();

        for &(u, v, kind) in &valid_edges {
            let pos = out_cursor[u as usize] as usize;
            col_idx[pos] = v;
            edge_kind[pos] = kind;
            out_cursor[u as usize] += 1;

            let t_pos = in_cursor[v as usize] as usize;
            transpose_col_idx[t_pos] = u;
            in_cursor[v as usize] += 1;
        }

        let denom = (num_vertices.max(1) * num_vertices.max(1)) as f64;
        let ambient_density_p = ((num_edges as f64) / denom).max(AMBIENT_DENSITY_FLOOR_P);

        let embeddings = if symbol_embeddings.len() == num_vertices {
            symbol_embeddings
        } else {
            (0..num_vertices)
                .map(|i| Self::synthetic_wave_embedding(i as u64, 0, 0))
                .collect()
        };

        Self {
            num_vertices,
            num_edges,
            ambient_density_p,
            row_ptr,
            col_idx,
            edge_kind,
            transpose_row_ptr,
            transpose_col_idx,
            symbol_embeddings: embeddings,
        }
    }

    /// Synthesizes a deterministic L2-normalized 64-lane MEM8 wave embedding for a node.
    pub fn synthetic_wave_embedding(
        seed: u64,
        size: u64,
        depth: usize,
    ) -> [f32; WAVE_EMBEDDING_LANES] {
        let mut vec = [0.0_f32; WAVE_EMBEDDING_LANES];
        let mut norm_sq = 0.0_f32;
        let base_freq = ((seed.wrapping_mul(SPLITMIX64_SEED_MUL) >> 48) as f32) * 0.01 + 1.0;
        let size_phase = ((size.saturating_add(1) as f32).ln()) * 0.25;
        let depth_mod = (depth as f32) * 0.15;

        for (lane, slot) in vec.iter_mut().enumerate() {
            let t = (lane as f32) / (WAVE_EMBEDDING_LANES as f32);
            let val = (base_freq * t * core::f32::consts::TAU + size_phase).sin()
                + 0.5 * (depth_mod * (lane as f32)).cos();
            *slot = val;
            norm_sq += val * val;
        }

        let inv_norm = 1.0 / norm_sq.sqrt().max(1e-6);
        for slot in &mut vec {
            *slot *= inv_norm;
        }
        vec
    }

    /// Constructs a `CsrSymbolGraph` and initial directory-seeded `SzemerediPartition`
    /// from a slice of scanned `FileNode`s.
    pub fn from_file_nodes(nodes: &[FileNode]) -> (Self, SzemerediPartition) {
        let n = nodes.len();
        if n == 0 {
            let empty_graph = Self::from_edges(0, &[], Vec::new());
            let empty_part = SzemerediPartition {
                vertex_to_cluster: Vec::new(),
                clusters: vec![Vec::new()],
                energy_index: 0.0,
                epsilon: DEFAULT_EMPTY_PARTITION_EPSILON,
            };
            return (empty_graph, empty_part);
        }

        let mut edges: Vec<(SymbolId, SymbolId, EdgeKind)> = Vec::with_capacity(n * 3);
        let mut embeddings = Vec::with_capacity(n);
        let mut depth_parent = [0u32; MAX_DIRECTORY_DEPTH_SLOTS];

        // Map top-level directories to initial Szemerédi clusters V_1 .. V_k
        let mut dir_to_cluster = std::collections::HashMap::<String, ClusterId>::new();
        let mut clusters: Vec<Vec<SymbolId>> = vec![Vec::new()]; // clusters[0] is Outlier V_0
        let mut vertex_to_cluster = vec![0u16; n];

        for (idx, node) in nodes.iter().enumerate() {
            let sym = idx as SymbolId;
            let name = node
                .path
                .file_name()
                .and_then(|s| s.to_str())
                .unwrap_or("root");
            let mut hash_seed = FNV1A_64_OFFSET_BASIS;
            for &b in name.as_bytes() {
                hash_seed ^= b as u64;
                hash_seed = hash_seed.wrapping_mul(FNV1A_64_PRIME);
            }
            embeddings.push(Self::synthetic_wave_embedding(
                hash_seed, node.size, node.depth,
            ));

            let d = node.depth.min(MAX_DIRECTORY_DEPTH_SLOTS - 1);
            if idx > 0 {
                let p = if d == 0 { 0 } else { depth_parent[d - 1] };
                edges.push((p, sym, EdgeKind::Import));
                edges.push((sym, p, EdgeKind::CallRef));
            }
            if node.is_dir {
                depth_parent[d] = sym;
            }

            // Connect sibling files in the same directory or matching category
            if idx >= 1 && nodes[idx - 1].depth == node.depth {
                let prev = (idx - 1) as SymbolId;
                let kind = match node.category {
                    FileCategory::Rust | FileCategory::Python | FileCategory::TypeScript => {
                        EdgeKind::TraitImpl
                    }
                    FileCategory::Config | FileCategory::Json | FileCategory::Yaml => {
                        EdgeKind::MutableShared
                    }
                    _ => EdgeKind::TypeBound,
                };
                edges.push((prev, sym, kind));
            }

            // Assign initial cluster based on parent directory (or V_0 for root/ignored)
            if idx == 0 || node.is_ignored {
                clusters[0].push(sym);
                vertex_to_cluster[idx] = 0;
            } else {
                let group_key = node
                    .path
                    .parent()
                    .and_then(|p| p.file_name())
                    .and_then(|s| s.to_str())
                    .unwrap_or("root_cluster")
                    .to_string();

                let cid = *dir_to_cluster.entry(group_key).or_insert_with(|| {
                    let next_id = clusters.len() as ClusterId;
                    clusters.push(Vec::new());
                    next_id
                });
                clusters[cid as usize].push(sym);
                vertex_to_cluster[idx] = cid;
            }
        }

        // Ensure at least 2 regular clusters exist when n >= 4 so bipartite regularity operates
        if clusters.len() == 2 && clusters[1].len() >= 4 {
            let full = std::mem::take(&mut clusters[1]);
            let mid = full.len() / 2;
            let (left, right) = full.split_at(mid);
            clusters[1] = left.to_vec();
            clusters.push(right.to_vec());
            for &sym in &clusters[2] {
                vertex_to_cluster[sym as usize] = 2;
            }
        }

        let graph = Self::from_edges(n, &edges, embeddings);
        let partition = SzemerediPartition {
            vertex_to_cluster,
            clusters,
            energy_index: 0.0,
            epsilon: DEFAULT_INITIAL_PARTITION_EPSILON,
        };
        (graph, partition)
    }
}

/// Represents a current vertex partition `P = {V_0, V_1, ..., V_k}`
#[derive(Debug, Clone)]
pub struct SzemerediPartition {
    /// Maps each `SymbolId -> ClusterId` (`0 == Exceptional Set V_0`)
    pub vertex_to_cluster: Vec<ClusterId>,
    /// Explicit vertex lists per cluster: `clusters[0]` is `V_0`
    pub clusters: Vec<Vec<SymbolId>>,
    /// Current Kohayakawa-Rödl mean-square energy `q_p(P)`
    pub energy_index: f64,
    /// Regularity parameter `epsilon` (e.g., `0.08`)
    pub epsilon: f64,
}

/// A super-node in the Szemerédi Reduced Graph `R`, stored in MEM8 format
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Mem8SuperNode {
    pub cluster_id: ClusterId,
    pub vertex_count: u32,
    /// Internal p-density `d_p(V_i, V_i)` (measures module cohesion)
    pub internal_p_density: f32,
    /// Centroid of the member symbols' MEM8 embeddings
    pub centroid_embedding: Vec<f32>,
    /// Principal symbol IDs (highest degree within `V_i`) for LLM anchoring
    pub representative_symbols: Vec<SymbolId>,
}

/// A regular bipartite super-edge between `V_i` and `V_j` in the Reduced Graph `R`
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct RegularSuperEdge {
    pub src_cluster: ClusterId,
    pub dst_cluster: ClusterId,
    /// Normalized density `d_p(V_i, V_j)`
    pub relative_p_density: f32,
    /// Dominant AST relationship in this bipartite bundle
    pub dominant_kind: EdgeKind,
}

/// Exact irregular or cross-cutting edge preserved for T8R token records
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct T8rExceptionEdge {
    pub src_symbol: SymbolId,
    pub dst_symbol: SymbolId,
    pub kind: EdgeKind,
    pub reason: ExceptionReason,
}

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ExceptionReason {
    /// Incident to the outlier bucket `V_0` ("god objects" / global orchestrators)
    OutlierVertex = 0,
    /// Belongs to a pair `(V_i, V_j)` that remained irregular at termination
    IrregularPair = 1,
    /// Architectural boundary violation (sparse bridge below `d_min` between otherwise unlinked clusters)
    ArchitecturalLeak = 2,
}

/// Final compressed artifact written to `.m8` / `8s.is` hub
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Mem8BookArchive {
    pub model_digest: [u8; 32],
    pub epsilon: f32,
    pub compression_ratio: f32,
    pub final_energy: f64,
    /// `O(1/epsilon^2)` regular super-nodes
    pub super_nodes: Vec<Mem8SuperNode>,
    /// Dense-like regular bipartite bundles
    pub reduced_edges: Vec<RegularSuperEdge>,
    /// Outlier vertices `V_0` requiring exact T8R attention
    pub outlier_vertices: Vec<SymbolId>,
    /// Explicit architectural exceptions (stored in T8R stream)
    pub exception_ledger: Vec<T8rExceptionEdge>,
    /// Total symbol count `N`
    pub total_symbol_count: u32,
    /// Mapping `SymbolId -> ClusterId`
    pub vertex_to_cluster: Vec<ClusterId>,
}
