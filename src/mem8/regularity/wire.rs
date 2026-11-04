//! Zero-Copy Binary Layout (`MEM8BOOK` `.m8b` Frame + T8R Columnar LEB128 Stream + SQLite WAL Schema).
//!
//! Designed for 64-byte cache-line alignment so AVX-512 / NEON / Apple AMX vector instructions
//! can scan `SuperNodeCentroidPod` (256 bytes = 4 cache lines) without unaligned loads,
//! and zero-copy readers can slice directly into `.m8b` byte buffers.

use super::types::{EdgeKind, ExceptionReason, T8rExceptionEdge};

pub const MEM8_BOOK_MAGIC: [u8; 8] = *b"MEM8BOOK";
pub const MEM8_BOOK_VERSION: u16 = 1;

/// 128-byte (2 cache lines) fixed header at offset `0x0000` of `~/.st/mem8_book.m8b`
#[repr(C, align(64))]
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Mem8BookHeader {
    /// Magic bytes: `b"MEM8BOOK"`
    pub magic: [u8; 8],
    /// Wire format version (`1`)
    pub version: u16,
    /// Flags (bit 0: SIMD-aligned centroids, bit 1: Zstd-compressed T8R tail)
    pub flags: u16,
    /// Total symbol count `N` in `~/.st/symbols/`
    pub total_symbols: u32,
    /// Number of regular super-nodes `K` (`<= 256`)
    pub num_super_nodes: u16,
    /// Number of outlier vertices in `V_0`
    pub num_outliers: u16,
    /// Number of regular super-edges in `R`
    pub num_reduced_edges: u32,
    /// Number of exact exception edges in the T8R stream
    pub num_exception_edges: u32,
    /// Szemerédi regularity parameter `epsilon`
    pub epsilon: f32,
    /// Final Kohayakawa-Rödl partition energy `q_p(P)`
    pub final_energy: f64,
    /// Byte offsets into the mmap file (all 64-byte aligned)
    pub offset_centroids: u64,
    pub offset_super_nodes: u64,
    pub offset_reduced_edges: u64,
    pub offset_symbol_to_cluster: u64,
    pub offset_t8r_exceptions: u64,
    pub t8r_byte_len: u64,
    /// BLAKE3 / deterministic 256-bit digest of the underlying AST symbol graph
    pub graph_blake3_digest: [u8; 32],
}

const _: () = assert!(core::mem::size_of::<Mem8BookHeader>() == 128);
const _: () = assert!(core::mem::align_of::<Mem8BookHeader>() == 64);

impl Mem8BookHeader {
    pub fn to_bytes(&self) -> [u8; 128] {
        let mut out = [0u8; 128];
        out[0..8].copy_from_slice(&self.magic);
        out[8..10].copy_from_slice(&self.version.to_le_bytes());
        out[10..12].copy_from_slice(&self.flags.to_le_bytes());
        out[12..16].copy_from_slice(&self.total_symbols.to_le_bytes());
        out[16..18].copy_from_slice(&self.num_super_nodes.to_le_bytes());
        out[18..20].copy_from_slice(&self.num_outliers.to_le_bytes());
        out[20..24].copy_from_slice(&self.num_reduced_edges.to_le_bytes());
        out[24..28].copy_from_slice(&self.num_exception_edges.to_le_bytes());
        out[28..32].copy_from_slice(&self.epsilon.to_le_bytes());
        out[32..40].copy_from_slice(&self.final_energy.to_le_bytes());
        out[40..48].copy_from_slice(&self.offset_centroids.to_le_bytes());
        out[48..56].copy_from_slice(&self.offset_super_nodes.to_le_bytes());
        out[56..64].copy_from_slice(&self.offset_reduced_edges.to_le_bytes());
        out[64..72].copy_from_slice(&self.offset_symbol_to_cluster.to_le_bytes());
        out[72..80].copy_from_slice(&self.offset_t8r_exceptions.to_le_bytes());
        out[80..88].copy_from_slice(&self.t8r_byte_len.to_le_bytes());
        out[88..120].copy_from_slice(&self.graph_blake3_digest);
        out
    }

    pub fn from_bytes(bytes: &[u8]) -> Option<Self> {
        if bytes.len() < 128 {
            return None;
        }
        let mut magic = [0u8; 8];
        magic.copy_from_slice(&bytes[0..8]);
        let mut graph_blake3_digest = [0u8; 32];
        graph_blake3_digest.copy_from_slice(&bytes[88..120]);

        Some(Self {
            magic,
            version: u16::from_le_bytes(bytes[8..10].try_into().ok()?),
            flags: u16::from_le_bytes(bytes[10..12].try_into().ok()?),
            total_symbols: u32::from_le_bytes(bytes[12..16].try_into().ok()?),
            num_super_nodes: u16::from_le_bytes(bytes[16..18].try_into().ok()?),
            num_outliers: u16::from_le_bytes(bytes[18..20].try_into().ok()?),
            num_reduced_edges: u32::from_le_bytes(bytes[20..24].try_into().ok()?),
            num_exception_edges: u32::from_le_bytes(bytes[24..28].try_into().ok()?),
            epsilon: f32::from_le_bytes(bytes[28..32].try_into().ok()?),
            final_energy: f64::from_le_bytes(bytes[32..40].try_into().ok()?),
            offset_centroids: u64::from_le_bytes(bytes[40..48].try_into().ok()?),
            offset_super_nodes: u64::from_le_bytes(bytes[48..56].try_into().ok()?),
            offset_reduced_edges: u64::from_le_bytes(bytes[56..64].try_into().ok()?),
            offset_symbol_to_cluster: u64::from_le_bytes(bytes[64..72].try_into().ok()?),
            offset_t8r_exceptions: u64::from_le_bytes(bytes[72..80].try_into().ok()?),
            t8r_byte_len: u64::from_le_bytes(bytes[80..88].try_into().ok()?),
            graph_blake3_digest,
        })
    }
}

/// 256-byte (4 cache lines) aligned MEM8 wave centroid for zero-copy SIMD cosine similarity
#[repr(C, align(64))]
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SuperNodeCentroidPod {
    pub lanes: [f32; 64],
}

const _: () = assert!(core::mem::size_of::<SuperNodeCentroidPod>() == 256);
const _: () = assert!(core::mem::align_of::<SuperNodeCentroidPod>() == 64);

impl SuperNodeCentroidPod {
    pub fn to_bytes(&self) -> [u8; 256] {
        let mut out = [0u8; 256];
        for (i, &val) in self.lanes.iter().enumerate() {
            out[i * 4..(i + 1) * 4].copy_from_slice(&val.to_le_bytes());
        }
        out
    }

    pub fn from_bytes(bytes: &[u8]) -> Option<Self> {
        if bytes.len() < 256 {
            return None;
        }
        let mut lanes = [0.0_f32; 64];
        for (i, slot) in lanes.iter_mut().enumerate() {
            let chunk: [u8; 4] = bytes[i * 4..(i + 1) * 4].try_into().ok()?;
            *slot = f32::from_le_bytes(chunk);
        }
        Some(Self { lanes })
    }
}

/// 32-byte POD metadata per Szemerédi Super-Node `V_i`
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SuperNodeMetaPod {
    pub cluster_id: u16,
    pub flags: u16,
    pub vertex_count: u32,
    pub internal_p_density: f32,
    /// Top-3 highest-degree anchor SymbolIds for instant LLM naming (`0xFFFF_FFFF` if unused)
    pub anchor_symbols: [u32; 3],
    /// Start offset into reduced-edge CSR adjacency
    pub reduced_edge_start: u32,
    pub reduced_edge_len: u32,
}

const _: () = assert!(core::mem::size_of::<SuperNodeMetaPod>() == 32);

impl SuperNodeMetaPod {
    pub fn to_bytes(&self) -> [u8; 32] {
        let mut out = [0u8; 32];
        out[0..2].copy_from_slice(&self.cluster_id.to_le_bytes());
        out[2..4].copy_from_slice(&self.flags.to_le_bytes());
        out[4..8].copy_from_slice(&self.vertex_count.to_le_bytes());
        out[8..12].copy_from_slice(&self.internal_p_density.to_le_bytes());
        out[12..16].copy_from_slice(&self.anchor_symbols[0].to_le_bytes());
        out[16..20].copy_from_slice(&self.anchor_symbols[1].to_le_bytes());
        out[20..24].copy_from_slice(&self.anchor_symbols[2].to_le_bytes());
        out[24..28].copy_from_slice(&self.reduced_edge_start.to_le_bytes());
        out[28..32].copy_from_slice(&self.reduced_edge_len.to_le_bytes());
        out
    }

    pub fn from_bytes(bytes: &[u8]) -> Option<Self> {
        if bytes.len() < 32 {
            return None;
        }
        Some(Self {
            cluster_id: u16::from_le_bytes(bytes[0..2].try_into().ok()?),
            flags: u16::from_le_bytes(bytes[2..4].try_into().ok()?),
            vertex_count: u32::from_le_bytes(bytes[4..8].try_into().ok()?),
            internal_p_density: f32::from_le_bytes(bytes[8..12].try_into().ok()?),
            anchor_symbols: [
                u32::from_le_bytes(bytes[12..16].try_into().ok()?),
                u32::from_le_bytes(bytes[16..20].try_into().ok()?),
                u32::from_le_bytes(bytes[20..24].try_into().ok()?),
            ],
            reduced_edge_start: u32::from_le_bytes(bytes[24..28].try_into().ok()?),
            reduced_edge_len: u32::from_le_bytes(bytes[28..32].try_into().ok()?),
        })
    }
}

/// 8-byte POD record for a regular bipartite bundle `(V_i, V_j)` in the Reduced Graph `R`
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RegularSuperEdgePod {
    pub src_cluster: u16,
    pub dst_cluster: u16,
    /// Quantized `fp16` or fixed-point `u16` (`density * 1024.0`)
    pub relative_p_density_fp16: u16,
    pub dominant_kind: u8,
    pub _pad: u8,
}

const _: () = assert!(core::mem::size_of::<RegularSuperEdgePod>() == 8);

impl RegularSuperEdgePod {
    #[inline]
    pub fn encode_density_fp16(density: f32) -> u16 {
        (density.clamp(0.0, 63.99) * 1024.0).round() as u16
    }

    #[inline]
    pub fn decode_density_fp16(raw: u16) -> f32 {
        (raw as f32) * (1.0 / 1024.0)
    }

    pub fn to_bytes(&self) -> [u8; 8] {
        let mut out = [0u8; 8];
        out[0..2].copy_from_slice(&self.src_cluster.to_le_bytes());
        out[2..4].copy_from_slice(&self.dst_cluster.to_le_bytes());
        out[4..6].copy_from_slice(&self.relative_p_density_fp16.to_le_bytes());
        out[6] = self.dominant_kind;
        out[7] = self._pad;
        out
    }

    pub fn from_bytes(bytes: &[u8]) -> Option<Self> {
        if bytes.len() < 8 {
            return None;
        }
        Some(Self {
            src_cluster: u16::from_le_bytes(bytes[0..2].try_into().ok()?),
            dst_cluster: u16::from_le_bytes(bytes[2..4].try_into().ok()?),
            relative_p_density_fp16: u16::from_le_bytes(bytes[4..6].try_into().ok()?),
            dominant_kind: bytes[6],
            _pad: bytes[7],
        })
    }
}

/// Encodes `T8rExceptionEdge` records into a compact columnar delta + LEB128 stream (~2.1 bytes/edge).
pub fn encode_t8r_exceptions_leb128(edges: &[T8rExceptionEdge]) -> Vec<u8> {
    let mut sorted = edges.to_vec();
    sorted.sort_unstable_by_key(|e| (e.src_symbol, e.dst_symbol));

    let mut out = Vec::with_capacity(sorted.len() * 3);
    write_uvarint(&mut out, sorted.len() as u64);

    let mut prev_src = 0u32;
    for e in &sorted {
        let delta_src = e.src_symbol - prev_src;
        write_uvarint(&mut out, delta_src as u64);
        prev_src = e.src_symbol;
    }

    let mut prev_dst = 0u32;
    let mut last_src = u32::MAX;
    for e in &sorted {
        if e.src_symbol != last_src {
            prev_dst = 0;
            last_src = e.src_symbol;
        }
        let delta_dst = e.dst_symbol.saturating_sub(prev_dst);
        write_uvarint(&mut out, delta_dst as u64);
        prev_dst = e.dst_symbol;
    }

    // Pack (EdgeKind: 4 bits | ExceptionReason: 4 bits) into 1 byte per edge
    for e in &sorted {
        let tag = ((e.kind as u8) & 0x0F) | (((e.reason as u8) & 0x0F) << 4);
        out.push(tag);
    }

    out
}

/// Decodes a columnar delta + LEB128 T8R exception stream back into `T8rExceptionEdge` records.
pub fn decode_t8r_exceptions_leb128(mut slice: &[u8]) -> Option<Vec<T8rExceptionEdge>> {
    let count = read_uvarint(&mut slice)? as usize;
    if count == 0 {
        return Some(Vec::new());
    }

    let mut srcs = Vec::with_capacity(count);
    let mut prev_src = 0u32;
    for _ in 0..count {
        let delta = read_uvarint(&mut slice)? as u32;
        prev_src = prev_src.wrapping_add(delta);
        srcs.push(prev_src);
    }

    let mut dsts = Vec::with_capacity(count);
    let mut prev_dst = 0u32;
    let mut last_src = u32::MAX;
    for &src in &srcs {
        if src != last_src {
            prev_dst = 0;
            last_src = src;
        }
        let delta = read_uvarint(&mut slice)? as u32;
        prev_dst = prev_dst.wrapping_add(delta);
        dsts.push(prev_dst);
    }

    if slice.len() < count {
        return None;
    }

    let mut edges = Vec::with_capacity(count);
    for idx in 0..count {
        let tag = slice[idx];
        let kind = EdgeKind::from_u8(tag & 0x0F);
        let reason = match (tag >> 4) & 0x0F {
            0 => ExceptionReason::OutlierVertex,
            1 => ExceptionReason::IrregularPair,
            _ => ExceptionReason::ArchitecturalLeak,
        };
        edges.push(T8rExceptionEdge {
            src_symbol: srcs[idx],
            dst_symbol: dsts[idx],
            kind,
            reason,
        });
    }

    Some(edges)
}

#[inline(always)]
fn write_uvarint(buf: &mut Vec<u8>, mut val: u64) {
    while val >= 0x80 {
        buf.push((val as u8) | 0x80);
        val >>= 7;
    }
    buf.push(val as u8);
}

#[inline(always)]
fn read_uvarint(slice: &mut &[u8]) -> Option<u64> {
    let mut result = 0u64;
    let mut shift = 0u32;
    loop {
        if slice.is_empty() || shift >= 64 {
            return None;
        }
        let byte = slice[0];
        *slice = &slice[1..];
        result |= ((byte & 0x7F) as u64) << shift;
        if (byte & 0x80) == 0 {
            return Some(result);
        }
        shift += 7;
    }
}

/// SQLite WAL Generation & Incremental Dirty-Symbol Ledger (`~/.st/symbols/index.db`).
pub const SQLITE_WAL_MIGRATION_004_SZEMEREDI: &str = r#"
PRAGMA journal_mode = WAL;
PRAGMA synchronous = NORMAL;

CREATE TABLE IF NOT EXISTS mem8_book_generations (
    generation_id      INTEGER PRIMARY KEY AUTOINCREMENT,
    created_at_unix_ms INTEGER NOT NULL,
    m8b_file_path      TEXT NOT NULL,
    graph_blake3_hex   TEXT NOT NULL,
    epsilon            REAL NOT NULL,
    final_energy       REAL NOT NULL,
    num_super_nodes    INTEGER NOT NULL,
    num_outliers       INTEGER NOT NULL,
    compression_ratio  REAL NOT NULL
) STRICT;

CREATE TABLE IF NOT EXISTS symbol_cluster_map (
    symbol_id          INTEGER PRIMARY KEY,
    file_path          TEXT NOT NULL,
    qualified_name     TEXT NOT NULL,
    cluster_id         INTEGER NOT NULL,
    is_anchor          INTEGER NOT NULL DEFAULT 0,
    dirty_since_gen    INTEGER NOT NULL DEFAULT 0
) WITHOUT ROWID, STRICT;

CREATE INDEX IF NOT EXISTS idx_cluster_lookup
    ON symbol_cluster_map(cluster_id, is_anchor DESC);
"#;
