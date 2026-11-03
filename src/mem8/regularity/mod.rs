//! Paul Erdős ("The Magician from Budapest" / "Straight from The Book")
//! Extremal, Probabilistic, and Structural Graph-Theoretic Engine for `smart-tree`.
//!
//! Unifies five pillars of Erdősian mathematics inside `src/mem8/regularity/`:
//! 1. **Szemerédi Regularity Compression ("MEM8 Book Mode")**: Collapses massive AST/directory graphs
//!    into $O(1/\varepsilon^2)$ regular super-nodes + a T8R Exception Ledger (`compressor.rs`, `oracle.rs`).
//! 2. **Zero-Copy `.m8b` Binary Wire Format & SIMD Recall**: 128-byte `Mem8BookHeader`, 256-byte cache-aligned
//!    MEM8 wave centroids, and columnar delta + LEB128 exception streams (`wire.rs`, `mmap_view.rs`).
//! 3. **Erdős Number & Mehlhorn Voronoi Steiner Tree Pruner**: Multi-source Voronoi Dijkstra wavefronts
//!    + Kruskal's MST over Voronoi boundary bridges (`erdos_steiner.rs`).
//! 4. **Constructive Lovász Local Lemma (Moser–Tardos) Worktree Splitter**: Parallel Maximal Independent Set
//!    resampling for zero-conflict multi-agent git worktree scheduling (`moser_tardos.rs`).
//! 5. **Erdős–Rényi Phase-Transition Security Scanner & Ramsey Monochromatic Cliques**: Molloy–Reed percolation
//!    criterion $\kappa = \langle k^2 \rangle / \langle k \rangle$, Chernoff–Erdős union-bound log-surprise $\Lambda(C)$,
//!    Füredi–Komlós spectral radius $\lambda_1$, and Red/Blue/Green Ramsey clique hunting (`erdos_renyi.rs`).

pub mod compressor;
pub mod erdos_renyi;
pub mod erdos_steiner;
pub mod mmap_view;
pub mod moser_tardos;
pub mod oracle;
pub mod types;
pub mod wire;

pub use compressor::{compress_symbol_graph_to_mem8, RegularityConfig};
pub use erdos_renyi::{
    scan_erdos_renyi_supply_chain, scan_ramsey_monochromatic_cliques, ComponentAnomaly,
    ErdosRenyiSecurityReport, MonochromaticClique, PhaseRegime, RamseyCliqueReport,
    RamseyEdgeColor, SymbolHazardFlags,
};
pub use erdos_steiner::{
    prune_context_erdos_steiner, ErdosSteinerConfig, ErdosSteinerSlice, SteinerClusterNode,
};
pub use mmap_view::ZeroCopyMem8Book;
pub use moser_tardos::{
    build_hypergraph_from_steiner_slices, solve_moser_tardos_worktrees, BadEventId, BadEventKind,
    LllHypergraph, MoserTardosSchedule, WorkUnitId, WorktreeColor,
};
pub use oracle::{check_bipartite_regularity, RegularityCheck};
pub use types::{
    ClusterId, CsrSymbolGraph, EdgeKind, ErdosLexiconBadge, ExceptionReason, Mem8BookArchive,
    Mem8SuperNode, RegularSuperEdge, SymbolId, SzemerediPartition, T8rExceptionEdge,
};
pub use wire::{
    decode_t8r_exceptions_leb128, encode_t8r_exceptions_leb128, Mem8BookHeader,
    RegularSuperEdgePod, SuperNodeCentroidPod, SuperNodeMetaPod, MEM8_BOOK_MAGIC,
    MEM8_BOOK_VERSION, SQLITE_WAL_MIGRATION_004_SZEMEREDI,
};

use crate::scanner::{FileCategory, FileNode};

/// Complete end-to-end diagnostic produced by `--mode the-book` (`st --mode erdos`).
#[derive(Debug, Clone)]
pub struct TheBookPipelineReport {
    pub archive: Mem8BookArchive,
    pub m8b_wire_bytes: usize,
    pub t8r_wire_bytes: usize,
    pub steiner_slice: ErdosSteinerSlice,
    pub worktree_schedule: MoserTardosSchedule,
    pub security_report: ErdosRenyiSecurityReport,
    pub ramsey_report: RamseyCliqueReport,
    /// Counts of files classified under Paul Erdős's lexicon:
    /// (StraightFromTheBook, Epsilons, LegallyDead, PureNoiseMonoliths)
    pub lexicon_counts: (usize, usize, usize, usize),
}

/// Runs all 5 stages of the Paul Erdős ("Straight from The Book") mathematical pipeline
/// on a scanned slice of `FileNode`s.
pub fn analyze_the_book_pipeline(nodes: &[FileNode]) -> TheBookPipelineReport {
    let (graph, initial_partition) = CsrSymbolGraph::from_file_nodes(nodes);
    let archive =
        compress_symbol_graph_to_mem8(&graph, initial_partition, RegularityConfig::default());
    let zero_copy_book = ZeroCopyMem8Book::from_archive(&archive);
    let aligned_bytes = zero_copy_book.to_aligned_bytes();

    // Choose up to 3 focal root terminals (largest source files or root directory)
    let mut focal_roots: Vec<SymbolId> = nodes
        .iter()
        .enumerate()
        .filter(|(_, n)| !n.is_dir && !n.is_ignored)
        .map(|(idx, n)| (idx as SymbolId, n.size))
        .collect::<Vec<_>>()
        .into_iter()
        .take(3)
        .map(|(sym, _)| sym)
        .collect();
    if focal_roots.is_empty() && !nodes.is_empty() {
        focal_roots.push(0);
    }

    let steiner_slice = prune_context_erdos_steiner(
        &zero_copy_book,
        &focal_roots,
        None,
        ErdosSteinerConfig::default(),
    );

    let hg = build_hypergraph_from_steiner_slices(
        &zero_copy_book,
        std::slice::from_ref(&steiner_slice),
        0.35,
    );
    let worktree_schedule = solve_moser_tardos_worktrees(&hg, 4, 0xE2D0_5B00, 1024);

    // Derive static hazard flags per node from category & path heuristics
    let hazards: Vec<SymbolHazardFlags> = nodes
        .iter()
        .map(|node| {
            let mut flags = SymbolHazardFlags::NONE;
            let path_str = node.path.to_string_lossy();
            if path_str.contains("mcp") || path_str.contains("hook") {
                flags = flags.union(SymbolHazardFlags::MCP_HOOK);
            }
            if path_str.contains("net") || path_str.contains("http") || path_str.contains("web") {
                flags = flags.union(SymbolHazardFlags::NET_SOCKET);
            }
            if path_str.contains("env") || path_str.contains("secret") || path_str.contains("auth")
            {
                flags = flags.union(SymbolHazardFlags::ENV_SECRET);
            }
            if matches!(node.category, FileCategory::Shell | FileCategory::Binary) {
                flags = flags.union(SymbolHazardFlags::PROC_EXEC);
            }
            flags
        })
        .collect();

    let security_report = scan_erdos_renyi_supply_chain(&graph, &hazards, 10.0);
    let ramsey_report = scan_ramsey_monochromatic_cliques(&graph);

    // Classify every node using Paul Erdős's vocabulary
    let mut from_the_book = 0usize;
    let mut epsilons = 0usize;
    let mut legally_dead = 0usize;
    let mut pure_noise = 0usize;

    for (idx, node) in nodes.iter().enumerate() {
        match classify_erdos_lexicon(node, idx, &archive) {
            ErdosLexiconBadge::StraightFromTheBook => from_the_book += 1,
            ErdosLexiconBadge::Epsilon => epsilons += 1,
            ErdosLexiconBadge::LegallyDead => legally_dead += 1,
            ErdosLexiconBadge::PureNoiseMonolith => pure_noise += 1,
        }
    }

    TheBookPipelineReport {
        t8r_wire_bytes: zero_copy_book.t8r_raw_bytes().len(),
        m8b_wire_bytes: aligned_bytes.len(),
        archive,
        steiner_slice,
        worktree_schedule,
        security_report,
        ramsey_report,
        lexicon_counts: (from_the_book, epsilons, legally_dead, pure_noise),
    }
}

/// Classifies a `FileNode` into Paul Erdős's lexicon:
/// - **Straight from The Book**: Clean, well-proportioned module inside a regular Szemerédi cluster
/// - **Epsilon ($\varepsilon$)**: Small utility script / helper file (`< 1 KiB`)
/// - **Legally Dead (L.D.)**: Ignored, empty, or orphaned artifact
/// - **Pure Noise / S.F. Monolith**: Oversized outlier in $V_0$ (`> 128 KiB` or high exception degree)
pub fn classify_erdos_lexicon(
    node: &FileNode,
    idx: usize,
    archive: &Mem8BookArchive,
) -> ErdosLexiconBadge {
    if node.is_ignored || (!node.is_dir && node.size == 0) {
        return ErdosLexiconBadge::LegallyDead;
    }
    if !node.is_dir && node.size > 131_072 {
        return ErdosLexiconBadge::PureNoiseMonolith;
    }
    if !node.is_dir && node.size < 1024 {
        return ErdosLexiconBadge::Epsilon;
    }
    if idx < archive.vertex_to_cluster.len() && archive.vertex_to_cluster[idx] == 0 && idx > 0 {
        return ErdosLexiconBadge::PureNoiseMonolith;
    }
    ErdosLexiconBadge::StraightFromTheBook
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::scanner::{FileCategory, FileNode, FilesystemType};
    use std::path::PathBuf;
    use std::time::SystemTime;

    fn make_synthetic_graph(
        num_clusters: usize,
        cluster_size: usize,
    ) -> (CsrSymbolGraph, SzemerediPartition) {
        let n = num_clusters * cluster_size + 1; // +1 for outlier 0
        let mut edges = Vec::new();
        let mut clusters = vec![vec![0 as SymbolId]];
        let mut vertex_to_cluster = vec![0 as ClusterId; n];

        for c in 1..=num_clusters {
            let mut c_verts = Vec::with_capacity(cluster_size);
            let base = ((c - 1) * cluster_size + 1) as SymbolId;
            for offset in 0..(cluster_size as SymbolId) {
                let u = base + offset;
                c_verts.push(u);
                vertex_to_cluster[u as usize] = c as ClusterId;
                // Internal edges
                let next = base + ((offset + 1) % (cluster_size as SymbolId));
                edges.push((u, next, EdgeKind::CallRef));
            }
            clusters.push(c_verts);
        }

        // Quasi-random bipartite edges between cluster c and c+1
        for c in 1..num_clusters {
            let v_a = &clusters[c];
            let v_b = &clusters[c + 1];
            for (i, &u) in v_a.iter().enumerate() {
                for (j, &v) in v_b.iter().enumerate() {
                    if (i * 7 + j * 13) % 3 == 0 {
                        edges.push((u, v, EdgeKind::Import));
                    }
                }
            }
        }

        // Outlier 0 cross-cutting edge
        edges.push((0, 1, EdgeKind::MutableShared));

        let embeddings = (0..n)
            .map(|i| CsrSymbolGraph::synthetic_wave_embedding(i as u64 + 1, 2048, i % 5))
            .collect();

        let graph = CsrSymbolGraph::from_edges(n, &edges, embeddings);
        let partition = SzemerediPartition {
            vertex_to_cluster,
            clusters,
            energy_index: 0.0,
            epsilon: 0.12,
        };
        (graph, partition)
    }

    #[test]
    fn test_szemeredi_regularity_compression_and_m8b_roundtrip() {
        let (graph, partition) = make_synthetic_graph(4, 12);
        let archive = compress_symbol_graph_to_mem8(&graph, partition, RegularityConfig::default());

        assert!(!archive.super_nodes.is_empty());
        assert_eq!(archive.total_symbol_count, 49);
        assert!(archive.compression_ratio > 1.0);

        let book = ZeroCopyMem8Book::from_archive(&archive);
        let raw_bytes = book.to_aligned_bytes();
        // Verify 64-byte alignment of all offsets in the header
        assert_eq!(book.header().offset_centroids % 64, 0);
        assert_eq!(book.header().offset_super_nodes % 64, 0);
        assert_eq!(book.header().offset_reduced_edges % 64, 0);
        assert_eq!(book.header().offset_symbol_to_cluster % 64, 0);
        assert_eq!(book.header().offset_t8r_exceptions % 64, 0);

        let decoded = ZeroCopyMem8Book::from_bytes(&raw_bytes).expect("valid .m8b frame");
        assert_eq!(decoded.header(), book.header());
        assert_eq!(decoded.centroids(), book.centroids());
        assert_eq!(decoded.super_nodes(), book.super_nodes());
        assert_eq!(decoded.reduced_edges(), book.reduced_edges());
        assert_eq!(decoded.symbol_to_cluster(), book.symbol_to_cluster());

        let decoded_t8r =
            decode_t8r_exceptions_leb128(decoded.t8r_raw_bytes()).expect("valid T8R stream");
        assert_eq!(decoded_t8r.len(), archive.exception_ledger.len());

        // Test SIMD top_k_super_nodes cosine recall
        let query = CsrSymbolGraph::synthetic_wave_embedding(2, 2048, 1);
        let top = decoded.top_k_super_nodes(&query, 2);
        assert!(!top.is_empty());
    }

    #[test]
    fn test_erdos_steiner_voronoi_pruner() {
        let (graph, partition) = make_synthetic_graph(4, 12);
        let archive = compress_symbol_graph_to_mem8(&graph, partition, RegularityConfig::default());
        let book = ZeroCopyMem8Book::from_archive(&archive);

        // Connect symbol 1 (in cluster 1) and symbol 40 (in cluster 4)
        let slice =
            prune_context_erdos_steiner(&book, &[0, 1, 40], None, ErdosSteinerConfig::default());

        assert!(!slice.steiner_super_nodes.is_empty());
        // Outlier 0 should be captured as an Erdős Number 0 exact bridge symbol
        assert!(slice.exact_bridge_symbols.contains(&(0, 0)));
        // Terminal clusters have Erdős Number 0
        assert_eq!(slice.steiner_super_nodes[0].erdos_number, 0);
        assert!(slice.estimated_tokens <= ErdosSteinerConfig::default().token_budget);
    }

    #[test]
    fn test_moser_tardos_constructive_lll_worktree_splitter() {
        let constraints: Vec<(BadEventKind, &[WorkUnitId])> = vec![
            (BadEventKind::PairwiseExclusive, &[1, 2]),
            (BadEventKind::PairwiseExclusive, &[2, 3]),
            (BadEventKind::PairwiseExclusive, &[3, 4]),
            (BadEventKind::MonochromaticHyperedge, &[1, 3, 5]),
            (
                BadEventKind::CongestionThreshold { max_same_color: 2 },
                &[1, 2, 3, 4, 5],
            ),
        ];
        let hg = LllHypergraph::from_constraints(6, &constraints);
        let schedule = solve_moser_tardos_worktrees(&hg, 4, 42, 500);

        assert!(schedule.is_conflict_free);
        assert!(schedule.witness_bottleneck_events.is_empty());
        assert_ne!(schedule.unit_to_worktree[1], schedule.unit_to_worktree[2]);
        assert_ne!(schedule.unit_to_worktree[2], schedule.unit_to_worktree[3]);
        assert_ne!(schedule.unit_to_worktree[3], schedule.unit_to_worktree[4]);
    }

    #[test]
    fn test_erdos_renyi_security_scanner_and_ramsey_cliques() {
        // Build a sparse tree of 30 nodes plus a dense 5-node C2 clique (nodes 25..30)
        let n = 30;
        let mut edges = Vec::new();
        for u in 1..25 {
            edges.push(((u - 1) as SymbolId, u as SymbolId, EdgeKind::Import));
        }
        // Plant a monochromatic Red K_5 clique with multi-hazard flags on nodes 25..30
        for u in 25..30 {
            for v in 25..30 {
                if u != v {
                    edges.push((u as SymbolId, v as SymbolId, EdgeKind::CallRef));
                }
            }
        }

        let graph = CsrSymbolGraph::from_edges(n, &edges, Vec::new());
        let mut hazards = vec![SymbolHazardFlags::NONE; n];
        hazards[25] = SymbolHazardFlags::NET_SOCKET;
        hazards[26] = SymbolHazardFlags::PROC_EXEC;
        hazards[27] = SymbolHazardFlags::ENV_SECRET;
        hazards[28] = SymbolHazardFlags::MCP_HOOK;

        let report = scan_erdos_renyi_supply_chain(&graph, &hazards, 4.0);
        assert_eq!(report.regime, PhaseRegime::AnomalousC2Clique);
        assert!(!report.anomalies.is_empty());

        let ramsey = scan_ramsey_monochromatic_cliques(&graph);
        assert!(ramsey.red_cliques > 0);
        assert!(!ramsey.top_cliques.is_empty());
        assert_eq!(ramsey.top_cliques[0].order_k, 4);
    }

    #[test]
    fn test_analyze_the_book_pipeline_e2e() {
        let nodes: Vec<FileNode> = (0..16)
            .map(|i| FileNode {
                path: PathBuf::from(format!("src/mod_{}/file_{}.rs", i / 4, i)),
                is_dir: i % 4 == 0,
                size: if i == 1 {
                    200_000
                } else {
                    2048 + (i as u64) * 100
                },
                permissions: 0o644,
                uid: 1000,
                gid: 1000,
                modified: SystemTime::UNIX_EPOCH,
                is_symlink: false,
                is_hidden: false,
                permission_denied: false,
                is_ignored: i == 15,
                depth: if i == 0 { 0 } else { (i % 3) + 1 },
                file_type: if i % 4 == 0 {
                    crate::scanner::FileType::Directory
                } else {
                    crate::scanner::FileType::RegularFile
                },
                category: FileCategory::Rust,
                search_matches: None,
                filesystem_type: FilesystemType::Apfs,
                git_branch: None,
                traversal_context: None,
                interest: None,
                security_findings: Vec::new(),
                change_status: None,
                content_hash: None,
            })
            .collect();

        let report = analyze_the_book_pipeline(&nodes);
        assert!(report.m8b_wire_bytes >= 128);
        assert!(report.lexicon_counts.0 > 0);
        assert_eq!(report.lexicon_counts.2, 1); // i == 15 is LegallyDead
    }
}
