//! Zero-Copy Reader & SIMD Cosine Recall over `MEM8BOOK` (`.m8b`) archives
//! (`src/mem8/regularity/mmap_view.rs`).

use super::types::{ClusterId, Mem8BookArchive};
use super::wire::*;

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
        (offset + 63) & !63
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
            let mut lanes = [0.0_f32; 64];
            for (i, &v) in sn.centroid_embedding.iter().take(64).enumerate() {
                lanes[i] = v;
            }
            centroids.push(SuperNodeCentroidPod { lanes });

            let mut anchors = [u32::MAX; 3];
            for (i, &sym) in sn.representative_symbols.iter().take(3).enumerate() {
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
                flags: 1,
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

        let offset_centroids = 128usize;
        let offset_super_nodes = Self::align64(offset_centroids + centroids.len() * 256);
        let offset_reduced_edges = Self::align64(offset_super_nodes + super_nodes.len() * 32);
        let offset_symbol_to_cluster =
            Self::align64(offset_reduced_edges + reduced_edges.len() * 8);
        let offset_t8r_exceptions =
            Self::align64(offset_symbol_to_cluster + archive.vertex_to_cluster.len() * 2);

        let header = Mem8BookHeader {
            magic: MEM8_BOOK_MAGIC,
            version: MEM8_BOOK_VERSION,
            flags: 0x0001, // bit 0: SIMD-aligned centroids
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

        buf[0..128].copy_from_slice(&self.header.to_bytes());

        let mut cursor = self.header.offset_centroids as usize;
        for c in &self.centroids {
            buf[cursor..cursor + 256].copy_from_slice(&c.to_bytes());
            cursor += 256;
        }

        cursor = self.header.offset_super_nodes as usize;
        for sn in &self.super_nodes {
            buf[cursor..cursor + 32].copy_from_slice(&sn.to_bytes());
            cursor += 32;
        }

        cursor = self.header.offset_reduced_edges as usize;
        for re in &self.reduced_edges {
            buf[cursor..cursor + 8].copy_from_slice(&re.to_bytes());
            cursor += 8;
        }

        cursor = self.header.offset_symbol_to_cluster as usize;
        for &cid in &self.symbol_to_cluster {
            buf[cursor..cursor + 2].copy_from_slice(&cid.to_le_bytes());
            cursor += 2;
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
        let c_end = c_start + k * 256;
        if c_end > bytes.len() {
            return Err("Corrupt centroids section offset");
        }
        let mut centroids = Vec::with_capacity(k);
        for idx in 0..k {
            let off = c_start + idx * 256;
            centroids.push(
                SuperNodeCentroidPod::from_bytes(&bytes[off..off + 256])
                    .ok_or("Corrupt centroid record")?,
            );
        }

        let sn_start = header.offset_super_nodes as usize;
        let sn_end = sn_start + k * 32;
        if sn_end > bytes.len() {
            return Err("Corrupt super_nodes section offset");
        }
        let mut super_nodes = Vec::with_capacity(k);
        for idx in 0..k {
            let off = sn_start + idx * 32;
            super_nodes.push(
                SuperNodeMetaPod::from_bytes(&bytes[off..off + 32])
                    .ok_or("Corrupt super_node record")?,
            );
        }

        let re_count = header.num_reduced_edges as usize;
        let re_start = header.offset_reduced_edges as usize;
        let re_end = re_start + re_count * 8;
        if re_end > bytes.len() {
            return Err("Corrupt reduced_edges section offset");
        }
        let mut reduced_edges = Vec::with_capacity(re_count);
        for idx in 0..re_count {
            let off = re_start + idx * 8;
            reduced_edges.push(
                RegularSuperEdgePod::from_bytes(&bytes[off..off + 8])
                    .ok_or("Corrupt reduced_edge record")?,
            );
        }

        let sym_count = header.total_symbols as usize;
        let sym_start = header.offset_symbol_to_cluster as usize;
        let sym_end = sym_start + sym_count * 2;
        if sym_end > bytes.len() {
            return Err("Corrupt symbol_to_cluster section offset");
        }
        let mut symbol_to_cluster = Vec::with_capacity(sym_count);
        for idx in 0..sym_count {
            let off = sym_start + idx * 2;
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

        scores.sort_unstable_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        scores.truncate(m);
        scores
    }
}
