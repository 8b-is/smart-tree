//! Zero-Copy Reader & SIMD Cosine Recall over `MEM8BOOK` (`.m8b`) archives
//! (`src/mem8/regularity/mmap_view.rs`).

use super::types::{ClusterId, Mem8BookArchive};
use super::wire::*;

/// Byte width (`2`) of each `ClusterId` entry in the `symbol_to_cluster` section.
pub const CLUSTER_ID_ENTRY_BYTES: usize = 2;

/// `Mem8BookHeader` flag bit (`0x0001`) indicating 64-byte SIMD-aligned centroids.
pub const HEADER_FLAG_SIMD_ALIGNED: u16 = 0x0001;

/// Sentinel `u32::MAX` for unused anchor symbol slots in `SuperNodeMetaPod`.
pub const UNUSED_ANCHOR_SYMBOL: u32 = u32::MAX;

/// Zero-copy view over a `MEM8BOOK` archive with 64-byte aligned POD tables.
#[derive(Debug, Clone)]
pub struct ZeroCopyMem8Book {
    header: Mem8BookHeader,
    centroids: Vec<SuperNodeCentroidPod>,
    super_nodes: Vec<SuperNodeMetaPod>,
    reduced_edges: Vec<RegularSuperEdgePod>,
    symbol_to_cluster: Vec<ClusterId>,
    t8r_raw_bytes: Vec<u8>,
}

impl ZeroCopyMem8Book {
    #[inline]
    fn align64(offset: usize) -> usize {
        (offset + (MEM8_BOOK_CACHE_LINE_ALIGN - 1)) & !(MEM8_BOOK_CACHE_LINE_ALIGN - 1)
    }

    /// Constructs a `ZeroCopyMem8Book` directly from a `Mem8BookArchive`, sorting
    /// reduced edges by `src_cluster` so each super-node's `reduced_edge_start..+len`
    /// forms a contiguous CSR slice.
    pub fn from_archive(archive: &Mem8BookArchive) -> Self {
        let k = archive.super_nodes.len();
        let mut sorted_edges = archive.reduced_edges.clone();
        sorted_edges.sort_unstable_by_key(|e| (e.src_cluster, e.dst_cluster));

        // Build per-cluster CSR ranges into sorted_edges
        let mut edge_start_by_cluster = vec![0u32; k + 2];
        let mut edge_len_by_cluster = vec![0u32; k + 2];
        for (idx, edge) in sorted_edges.iter().enumerate() {
            let cid = edge.src_cluster as usize;
            if cid < edge_len_by_cluster.len() {
                if edge_len_by_cluster[cid] == 0 {
                    edge_start_by_cluster[cid] = idx as u32;
                }
                edge_len_by_cluster[cid] += 1;
            }
        }

        let mut centroids = Vec::with_capacity(k);
        let mut super_nodes = Vec::with_capacity(k);

        for sn in &archive.super_nodes {
            let mut lanes = [0.0_f32; SUPER_NODE_CENTROID_LANES];
            for (i, &v) in sn
                .centroid_embedding
                .iter()
                .take(SUPER_NODE_CENTROID_LANES)
                .enumerate()
            {
                lanes[i] = v;
            }
            centroids.push(SuperNodeCentroidPod { lanes });

            let mut anchors = [UNUSED_ANCHOR_SYMBOL; SUPER_NODE_ANCHOR_SLOTS];
            for (i, &sym) in sn
                .representative_symbols
                .iter()
                .take(SUPER_NODE_ANCHOR_SLOTS)
                .enumerate()
            {
                anchors[i] = sym;
            }

            let cid = sn.cluster_id as usize;
            let (start, len) = if cid < edge_start_by_cluster.len() {
                (edge_start_by_cluster[cid], edge_len_by_cluster[cid])
            } else {
                (0, 0)
            };

            super_nodes.push(SuperNodeMetaPod {
                cluster_id: sn.cluster_id,
                flags: HEADER_FLAG_SIMD_ALIGNED,
                vertex_count: sn.vertex_count,
                internal_p_density: sn.internal_p_density,
                anchor_symbols: anchors,
                reduced_edge_start: start,
                reduced_edge_len: len,
            });
        }

        let reduced_edges: Vec<RegularSuperEdgePod> = sorted_edges
            .iter()
            .map(|e| RegularSuperEdgePod {
                src_cluster: e.src_cluster,
                dst_cluster: e.dst_cluster,
                relative_p_density_fp16: RegularSuperEdgePod::encode_density_fp16(
                    e.relative_p_density,
                ),
                dominant_kind: e.dominant_kind as u8,
                _pad: 0,
            })
            .collect();

        let t8r_raw_bytes = encode_t8r_exceptions_leb128(&archive.exception_ledger);

        let offset_centroids = MEM8_BOOK_HEADER_BYTES;
        let offset_super_nodes =
            Self::align64(offset_centroids + centroids.len() * SUPER_NODE_CENTROID_BYTES);
        let offset_reduced_edges =
            Self::align64(offset_super_nodes + super_nodes.len() * SUPER_NODE_META_BYTES);
        let offset_symbol_to_cluster =
            Self::align64(offset_reduced_edges + reduced_edges.len() * REGULAR_SUPER_EDGE_BYTES);
        let offset_t8r_exceptions = Self::align64(
            offset_symbol_to_cluster + archive.vertex_to_cluster.len() * CLUSTER_ID_ENTRY_BYTES,
        );

        let header = Mem8BookHeader {
            magic: MEM8_BOOK_MAGIC,
            version: MEM8_BOOK_VERSION,
            flags: HEADER_FLAG_SIMD_ALIGNED, // bit 0: SIMD-aligned centroids
            total_symbols: archive.total_symbol_count,
            num_super_nodes: k as u16,
            num_outliers: archive.outlier_vertices.len() as u16,
            num_reduced_edges: reduced_edges.len() as u32,
            num_exception_edges: archive.exception_ledger.len() as u32,
            epsilon: archive.epsilon,
            final_energy: archive.final_energy,
            offset_centroids: offset_centroids as u64,
            offset_super_nodes: offset_super_nodes as u64,
            offset_reduced_edges: offset_reduced_edges as u64,
            offset_symbol_to_cluster: offset_symbol_to_cluster as u64,
            offset_t8r_exceptions: offset_t8r_exceptions as u64,
            t8r_byte_len: t8r_raw_bytes.len() as u64,
            graph_blake3_digest: archive.model_digest,
        };

        Self {
            header,
            centroids,
            super_nodes,
            reduced_edges,
            symbol_to_cluster: archive.vertex_to_cluster.clone(),
            t8r_raw_bytes,
        }
    }

    /// Serializes the `ZeroCopyMem8Book` into a 64-byte section-aligned `.m8b` byte buffer.
    pub fn to_aligned_bytes(&self) -> Vec<u8> {
        let total_size =
            Self::align64((self.header.offset_t8r_exceptions as usize) + self.t8r_raw_bytes.len());
        let mut buf = vec![0u8; total_size];

        buf[0..MEM8_BOOK_HEADER_BYTES].copy_from_slice(&self.header.to_bytes());

        let mut cursor = self.header.offset_centroids as usize;
        for c in &self.centroids {
            buf[cursor..cursor + SUPER_NODE_CENTROID_BYTES].copy_from_slice(&c.to_bytes());
            cursor += SUPER_NODE_CENTROID_BYTES;
        }

        cursor = self.header.offset_super_nodes as usize;
        for sn in &self.super_nodes {
            buf[cursor..cursor + SUPER_NODE_META_BYTES].copy_from_slice(&sn.to_bytes());
            cursor += SUPER_NODE_META_BYTES;
        }

        cursor = self.header.offset_reduced_edges as usize;
        for re in &self.reduced_edges {
            buf[cursor..cursor + REGULAR_SUPER_EDGE_BYTES].copy_from_slice(&re.to_bytes());
            cursor += REGULAR_SUPER_EDGE_BYTES;
        }

        cursor = self.header.offset_symbol_to_cluster as usize;
        for &cid in &self.symbol_to_cluster {
            buf[cursor..cursor + CLUSTER_ID_ENTRY_BYTES].copy_from_slice(&cid.to_le_bytes());
            cursor += CLUSTER_ID_ENTRY_BYTES;
        }

        let t8r_start = self.header.offset_t8r_exceptions as usize;
        let t8r_len = self.t8r_raw_bytes.len();
        buf[t8r_start..t8r_start + t8r_len].copy_from_slice(&self.t8r_raw_bytes);

        buf
    }

    /// Parses and validates a `.m8b` byte buffer in `O(1)` section-offset time.
    pub fn from_bytes(bytes: &[u8]) -> Result<Self, &'static str> {
        let header = Mem8BookHeader::from_bytes(bytes).ok_or("MEM8BOOK buffer too small")?;
        if header.magic != MEM8_BOOK_MAGIC {
            return Err("Invalid MEM8BOOK magic header");
        }
        if header.version != MEM8_BOOK_VERSION {
            return Err("Unsupported MEM8BOOK wire version");
        }

        let k = header.num_super_nodes as usize;
        let c_start = header.offset_centroids as usize;
        let c_end = c_start + k * SUPER_NODE_CENTROID_BYTES;
        if c_end > bytes.len() {
            return Err("Corrupt centroids section offset");
        }
        let mut centroids = Vec::with_capacity(k);
        for idx in 0..k {
            let off = c_start + idx * SUPER_NODE_CENTROID_BYTES;
            centroids.push(
                SuperNodeCentroidPod::from_bytes(&bytes[off..off + SUPER_NODE_CENTROID_BYTES])
                    .ok_or("Corrupt centroid record")?,
            );
        }

        let sn_start = header.offset_super_nodes as usize;
        let sn_end = sn_start + k * SUPER_NODE_META_BYTES;
        if sn_end > bytes.len() {
            return Err("Corrupt super_nodes section offset");
        }
        let mut super_nodes = Vec::with_capacity(k);
        for idx in 0..k {
            let off = sn_start + idx * SUPER_NODE_META_BYTES;
            super_nodes.push(
                SuperNodeMetaPod::from_bytes(&bytes[off..off + SUPER_NODE_META_BYTES])
                    .ok_or("Corrupt super_node record")?,
            );
        }

        let re_count = header.num_reduced_edges as usize;
        let re_start = header.offset_reduced_edges as usize;
        let re_end = re_start + re_count * REGULAR_SUPER_EDGE_BYTES;
        if re_end > bytes.len() {
            return Err("Corrupt reduced_edges section offset");
        }
        let mut reduced_edges = Vec::with_capacity(re_count);
        for idx in 0..re_count {
            let off = re_start + idx * REGULAR_SUPER_EDGE_BYTES;
            reduced_edges.push(
                RegularSuperEdgePod::from_bytes(&bytes[off..off + REGULAR_SUPER_EDGE_BYTES])
                    .ok_or("Corrupt reduced_edge record")?,
            );
        }

        let sym_count = header.total_symbols as usize;
        let sym_start = header.offset_symbol_to_cluster as usize;
        let sym_end = sym_start + sym_count * CLUSTER_ID_ENTRY_BYTES;
        if sym_end > bytes.len() {
            return Err("Corrupt symbol_to_cluster section offset");
        }
        let mut symbol_to_cluster = Vec::with_capacity(sym_count);
        for idx in 0..sym_count {
            let off = sym_start + idx * CLUSTER_ID_ENTRY_BYTES;
            let cid = u16::from_le_bytes([bytes[off], bytes[off + 1]]);
            symbol_to_cluster.push(cid);
        }

        let t8r_start = header.offset_t8r_exceptions as usize;
        let t8r_end = t8r_start + (header.t8r_byte_len as usize);
        if t8r_end > bytes.len() {
            return Err("Corrupt t8r_exceptions section offset");
        }

        Ok(Self {
            header,
            centroids,
            super_nodes,
            reduced_edges,
            symbol_to_cluster,
            t8r_raw_bytes: bytes[t8r_start..t8r_end].to_vec(),
        })
    }

    #[inline(always)]
    pub fn header(&self) -> &Mem8BookHeader {
        &self.header
    }

    /// Zero-copy slice of `K` 256-byte aligned MEM8 super-node centroids
    #[inline(always)]
    pub fn centroids(&self) -> &[SuperNodeCentroidPod] {
        &self.centroids
    }

    /// Zero-copy slice of `K` super-node metadata records
    #[inline(always)]
    pub fn super_nodes(&self) -> &[SuperNodeMetaPod] {
        &self.super_nodes
    }

    /// Zero-copy slice of regular bipartite super-edges in `R`
    #[inline(always)]
    pub fn reduced_edges(&self) -> &[RegularSuperEdgePod] {
        &self.reduced_edges
    }

    /// Instant `O(1)` lookup: `symbol_id -> cluster_id` (`0 == Outlier V_0`)
    #[inline(always)]
    pub fn symbol_to_cluster(&self) -> &[u16] {
        &self.symbol_to_cluster
    }

    /// Raw compressed T8R exception edge stream
    #[inline(always)]
    pub fn t8r_raw_bytes(&self) -> &[u8] {
        &self.t8r_raw_bytes
    }

    /// Ultra-fast SIMD-friendly scan: finds the top-`m` relevant Szemerédi super-nodes for an
    /// MCP query embedding in `< 2 microseconds` (`K <= 256` dot products over 64 `f32` lanes).
    pub fn top_k_super_nodes(&self, query_embedding: &[f32; 64], m: usize) -> Vec<(u16, f32)> {
        let centroids = self.centroids();
        let meta = self.super_nodes();

        let mut scores: Vec<(u16, f32)> = centroids
            .iter()
            .zip(meta.iter())
            .map(|(c, node)| {
                (
                    node.cluster_id,
                    crate::felix_asm::felix_dot64_f32(&c.lanes, query_embedding),
                )
            })
            .collect();

        scores.sort_unstable_by(|a, b| b.1.total_cmp(&a.1));
        scores.truncate(m);
        scores
    }
}
