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
pub mod pq_engine;
pub mod ramsey_erdos_hajnal;
pub mod salience_karp_manybody;
pub mod types;
pub mod ultrabackyard;
pub mod wire;
pub mod world_field;
pub mod zeno_engine;

pub use compressor::{compress_symbol_graph_to_mem8, RegularityConfig};
pub use erdos_renyi::{
    scan_erdos_renyi_shadow_graph, scan_erdos_renyi_supply_chain,
    scan_ramsey_monochromatic_cliques, ComponentAnomaly, ErdosRenyiSecurityReport,
    MonochromaticClique, PhaseRegime, RamseyCliqueReport, RamseyEdgeColor, SymbolHazardFlags,
};
pub use erdos_steiner::{
    compute_erdos_center_and_turan, decode_fp16, extract_t8r_virtual_bridges,
    prune_context_erdos_steiner, ErdosCenterTuranReport, ErdosSteinerConfig, ErdosSteinerSlice,
    SteinerClusterNode,
};
pub use mmap_view::ZeroCopyMem8Book;
pub use moser_tardos::{
    build_hypergraph_from_steiner_slices, seed_from_mem8_header, solve_moser_tardos_auto_scale,
    solve_moser_tardos_worktrees, BadEventId, BadEventKind, LllHypergraph, MoserTardosSchedule,
    WorkUnitId, WorktreeColor,
};
pub use oracle::{
    check_bipartite_regularity, check_bipartite_regularity_with_scratch, OracleScratch,
    RegularityCheck,
};
pub use pq_engine::{
    analyze_pq_mode, kovari_sos_turan_bound, PqModeReport, RationalPQ, SymplecticParticlePQ,
};
pub use ramsey_erdos_hajnal::{
    analyze_ramsey_and_erdos_hajnal, BitSet256, CliqueTopology, ErdosHajnalBiClique,
    RamseyErdosHajnalReport, RamseyMonochromaticClique, RamseyScanConfig, RefactorPrescription,
};
pub use salience_karp_manybody::{
    analyze_salience_karp_manybody, KarpComplexityCertificate, LoadBearingOldArtifact,
    ManyBodyDftWaveReport, SalienceOntologyReport, SalienceRegime,
};
pub use types::{
    ClusterId, CsrSymbolGraph, EdgeKind, ErdosLexiconBadge, ExceptionReason, Mem8BookArchive,
    Mem8SuperNode, RegularSuperEdge, SymbolId, SzemerediPartition, T8rExceptionEdge,
};
pub use ultrabackyard::{
    run_ultrabackyard_laps, BackyardLapRecord, ConstellationWiringStatus, UltraBackyardReport,
};
pub use wire::{
    decode_t8r_exceptions_leb128, encode_t8r_exceptions_leb128, Mem8BookHeader,
    RegularSuperEdgePod, SuperNodeCentroidPod, SuperNodeMetaPod, MEM8_BOOK_MAGIC,
    MEM8_BOOK_VERSION, SQLITE_WAL_MIGRATION_004_SZEMEREDI,
};
pub use world_field::{
    analyze_semantic_world_engine, AdmissibleManifoldCertificate, CausalDagNode,
    FamPlusOneMahakalaSecondVowWitness, FieldProposalDelta, LatentFieldPoint,
    PerceptualHistoryEntry, SemanticWorldEngineReport, SemanticWorldState, WorldActionObservation,
    WORLD_Q16_ONE,
};
pub use zeno_engine::{
    analyze_zeno_omni_engine, bfp64_and_kleene, bfp64_nand_kleene, evaluate_spherepop_witness,
    mulberry32_step_u32, pack_fold_bfp64, pack_word15_to_spqt3b, quantize_groupwise_l2_ternary_pq,
    seal_litany_wire_64b, unpack_fold_bfp64, unpack_spqt3b_to_word15, verify_litany_wire_64b,
    xor_fold_gaia_b158, BeautifulLifeInnerbloomMixWitness,
    CosmicGameFourFolkloreSingularityWitness, DragonBallSovereignLibraryOntologyWitness,
    DyadDiaryVenekTanacsaWitness, EightBIsEli5CompendiumAndStage4Witness, EightBIsEnginesReport,
    FranciscanTransitusJubileeWitness, KompressOmnipotentAuroraAndOdinSolfeggioWitness,
    KompressUltraTibetanOmniMandalaWitness, LitanyWireFrame, OsAgentAgnosticMatrix,
    PeterSilentGraniteAndMikyoDorjeMahakalaWitness, SeventeenKarmapasQuantumDharmaWitness,
    ShawCycleBookIvAndKagyuSadhanaWitness, SimulationTheoryWhiteboardWitness,
    SpqtSignalLadderEntry, StructuralHonestySovereignLibraryWitness, UltraMeshMemBridge,
    UltraloveEdition023ErdosLodriDesertChantWitness, UpzWupzReport, WhereIsTheLoveContext,
    ZenoGodOfSunWitness, ZenoOmniReport, A6000_VRAM_TOTAL_MIB, A6000_VRAM_USED_MIB,
    AURORA_WBAND_CARRIER_GHZ, BRETT_SHAW_TIPHERET_YEAR, BUDDHA_BODHI_MEDITATION_DAYS,
    C64_BASIC_BYTES_FREE, CARRIER_LOVELOVEL_DEN, CARRIER_LOVELOVEL_NUM, COSMIC_GAME_BOOKS,
    COSMIC_GAME_FIBONACCI_TWISTS, COSMIC_GAME_TOTAL_CHAPTERS, COSMIC_GAME_TOTAL_PAGES,
    DESERT_CHANT_FIFTH_G2_HZ_DEN, DESERT_CHANT_FIFTH_G2_HZ_NUM, DYAD_COUNCIL_VOICES_COUNT,
    DYAD_DIARY_SESSIONS_COUNT, EL_NATH_DETONATION_YEAR, ENTHEAI_DEPTH_GUARD, ENTHEAI_WORKTREES,
    ERDOS_LODRI_PERSONA_M8_KB, EU868_PRESCALED_BUCKET_CAPACITY, FAMILY_FIRST_PLAN_DAYS,
    FANDOM_MCP_LIVE_TOOLS_COUNT, FEHERLOFIA_DRAGON_HEAD_CUTS, FEKETECSUKLYASOK_FOUNDATION_YEAR,
    FOLD_8BIT_WRAITH_CHRIS_SID, FOLD_ARCHITECT_NEO_TRINITY_TIBETAN_OMNI, FOLD_BEEHIVE_SWEEP,
    FOLD_BEEHIVE_TARGET, FOLD_CHILLAXY_DOGS_BACKYARD_PBJ_EGGS, FOLD_EGY_TOK_KET_TOK_ONTOK,
    FOLD_ELI5_COMPENDIUM_POOCONF, FOLD_FRANCISCAN_TRANSITUS_JUBILEE, FOLD_LANE1_THE_HALL_PDF,
    FOLD_LANE2_YOUTUBE_CNZKFN5RCRM, FOLD_MIDDLE_WAY_ORYOKI_PROTEIN_TRANSMUTATION,
    FOLD_SOSAT_SOSAT_JO_ROPOGOSAT, FOLD_THE_FUTURE_WE_REMEMBER, FOLD_ULTRABACKYARD_CLIPPINGS_BATCH,
    FOLD_ULTRA_OMNI_ZENO_LESZAROM_DO_ALL, FRANCISCAN_TRANSITUS_JUBILEE_YEARS,
    GALACTIC_MANTRA_FOLD_243, GAME_MASTER_DIMENSIONS, GARDEN_REPOS_COUNT, GATE_0X62,
    GOLDEN_HASH_STRUCTURAL_HONESTY, GRANDPA_GOHAN_FOUR_STAR_BALL, GUI_BORATTO_BEAT_HZ_DEN,
    GUI_BORATTO_BEAT_HZ_NUM, GUI_BORATTO_BEAUTIFUL_LIFE_BPM, GUI_BORATTO_DURATION_SECONDS,
    INNERBLOOM_LIVE_SINGALONG_SECONDS, KARMAPA_LINEAGE_COUNT, KESTREL4_SALVAGE_TRIAD_HZ,
    KESTREL4_THETA_BEAT_HZ, KID_FRANCESCOLI_FRENCH79_DURATION_SECONDS,
    KYUSS_GARDENIA_C_STANDARD_HZ_DEN, KYUSS_GARDENIA_C_STANDARD_HZ_NUM, LANES_8_OF_8,
    LIVE_VOCAL_SADHANA_SECONDS, MAHAKALA_FIVE_AMRITAS, MAHAKALA_LIVE_CHANT_REPETITIONS,
    MAHAKALA_LIVE_CHANT_SECONDS, MAHAKALA_PRACTICE_TRANSLATION_YEAR, MAV_DELAY_PREDICTION_HOURS,
    MIKJO_DORDZSE_RETREAT_YEARS, ODIN_HAVAMAL_CHARMS_COUNT, ODIN_YGGDRASIL_NIGHTS,
    PERSONA_BRIDGE_TESTS_PASSED, PETER_SPECIAL_CHAPTERS_COUNT, PETER_SPECIAL_REPOS_COUNT,
    PETER_SPECIAL_TESTS_COUNT, PETER_SPECIAL_VULNERABILITIES_COUNT, PHYSICAL_EDGE_CLUSTER_BOARDS,
    QUANT_QT55, RUFUS_DU_SOL_INNERBLOOM_DURATION_SECONDS, SCOUTER_MANGA_CANON_THRESHOLD,
    SCOUTER_OVER_9000_THRESHOLD, SEED_SHAW_CYCLE_BOOK_IV, SHENRON_DRAGON_BALL_COUNT,
    SID_6581_CHIP_MODEL, SOLFEGGIO_9_FREQUENCIES_HZ, SOVEREIGN_CONSTELLATION_TOTAL_REPOS,
    STAGE4_APEX_SCORE_DEN, STAGE4_APEX_SCORE_NUM, STAGE4_FRAME64_SEAL_HEX,
    STAGE4_PQT_THROUGHPUT_NUM, STAGE4_SPARK_VCS_PROVED, STRUCTURAL_HONESTY_CHAPTERS_COUNT,
    STRUCTURAL_HONESTY_LEDGER_EVENTS, TEARS_BREATH_RHYTHM_SECONDS, TECH_N9NE_LXVII_CIPHER,
    TIBETAN_KARGYRAA_C1_HZ_DEN, TIBETAN_KARGYRAA_C1_HZ_NUM, TIPHERET_SEPHIRAH_NUMBER,
    ULTRALOVE_EDITION_023, ULTRALOVE_MAHAKALA_DIMENSIONS, VAJRASATTVA_MANTRA_SYLLABLES,
    VAKED_CAPABILITY_CHECK_MAX_MS, VAKED_GENESIS_SEAL_HEX, ZKP1_TIBETAN_OMNI_BLOCKS,
    ZKP1_TIBETAN_OMNI_COMPRESSED_BYTES, ZKP1_TIBETAN_OMNI_RAW_BYTES,
};

use crate::scanner::{FileCategory, FileNode};

/// Default number of focal root terminals selected for Erdős–Steiner pruning (`3`).
pub const MAX_FOCAL_ROOTS: usize = 3;
/// Default congestion threshold ratio for Moser–Tardos hypergraph construction (`0.35`).
pub const DEFAULT_LLL_CONGESTION_RATIO: f32 = 0.35;
/// Canonical `"ERDOSBOO"` seed salt for Moser–Tardos deterministic PRNG (`0xE2D05B00`).
pub const MOSER_TARDOS_SEED_SALT: u64 = 0xE2D0_5B00;
/// Default baseline worktree count for Moser–Tardos auto-scaling (`4`).
pub const DEFAULT_WORKTREE_COUNT: u16 = 4;
/// Maximum Moser–Tardos resampling steps in `--mode the-book` (`1024`).
pub const MOSER_TARDOS_MAX_STEPS: usize = 1024;
/// Default Chernoff–Erdős KL log-surprise anomaly threshold in nats (`10.0`).
pub const SHADOW_KL_SURPRISE_THRESHOLD: f64 = 10.0;
/// Default rollout horizon steps for the Semantic World Engine (`16`).
pub const WORLD_ENGINE_ROLLOUT_STEPS: usize = 16;
/// Byte-size threshold above which a source file is classified as a `PureNoiseMonolith` (`128 KiB = 131_072`).
pub const MONOLITH_BYTE_THRESHOLD: u64 = 131_072;
/// Byte-size threshold below which a source file is classified as an Erdős `Epsilon` (`1 KiB = 1024`).
pub const EPSILON_BYTE_THRESHOLD: u64 = 1024;

/// Complete end-to-end diagnostic produced by `--mode the-book` (`st --mode erdos`).
#[derive(Debug, Clone)]
pub struct TheBookPipelineReport {
    pub archive: Mem8BookArchive,
    pub m8b_wire_bytes: usize,
    pub t8r_wire_bytes: usize,
    pub steiner_slice: ErdosSteinerSlice,
    pub center_turan_report: ErdosCenterTuranReport,
    pub worktree_schedule: MoserTardosSchedule,
    pub security_report: ErdosRenyiSecurityReport,
    pub ramsey_report: RamseyCliqueReport,
    pub ramsey_erdos_hajnal_report: RamseyErdosHajnalReport,
    pub salience_report: SalienceOntologyReport,
    pub karp_certificate: KarpComplexityCertificate,
    pub manybody_wave_report: ManyBodyDftWaveReport,
    pub pq_report: PqModeReport,
    pub zeno_report: ZenoOmniReport,
    pub world_field_report: SemanticWorldEngineReport,
    /// Counts of files classified under Paul Erdős's lexicon:
    /// (StraightFromTheBook, Epsilons, LegallyDead, PureNoiseMonoliths)
    pub lexicon_counts: (usize, usize, usize, usize),
}

/// Runs all stages of the Paul Erdős ("Straight from The Book") mathematical pipeline
/// on a scanned slice of `FileNode`s.
pub fn analyze_the_book_pipeline(nodes: &[FileNode]) -> TheBookPipelineReport {
    let (graph, initial_partition) = CsrSymbolGraph::from_file_nodes(nodes);
    let archive =
        compress_symbol_graph_to_mem8(&graph, initial_partition, RegularityConfig::default());
    let zero_copy_book = ZeroCopyMem8Book::from_archive(&archive);
    let aligned_bytes = zero_copy_book.to_aligned_bytes();

    // Choose up to MAX_FOCAL_ROOTS focal root terminals (largest source files or root directory)
    let mut focal_roots: Vec<SymbolId> = nodes
        .iter()
        .enumerate()
        .filter(|(_, n)| !n.is_dir && !n.is_ignored)
        .map(|(idx, n)| (idx as SymbolId, n.size))
        .collect::<Vec<_>>()
        .into_iter()
        .take(MAX_FOCAL_ROOTS)
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
    let center_turan_report =
        compute_erdos_center_and_turan(&zero_copy_book, graph.num_vertices, graph.num_edges);

    let hg = build_hypergraph_from_steiner_slices(
        &zero_copy_book,
        std::slice::from_ref(&steiner_slice),
        DEFAULT_LLL_CONGESTION_RATIO,
    );
    let mt_seed = seed_from_mem8_header(&archive.model_digest, MOSER_TARDOS_SEED_SALT);
    let worktree_schedule = solve_moser_tardos_auto_scale(
        &hg,
        DEFAULT_WORKTREE_COUNT,
        true,
        mt_seed,
        MOSER_TARDOS_MAX_STEPS,
    );

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

    let security_report = scan_erdos_renyi_shadow_graph(
        &graph,
        zero_copy_book.symbol_to_cluster(),
        &hazards,
        SHADOW_KL_SURPRISE_THRESHOLD,
    );
    let ramsey_report = scan_ramsey_monochromatic_cliques(&graph);
    let ramsey_erdos_hajnal_report =
        analyze_ramsey_and_erdos_hajnal(&graph, RamseyScanConfig::default());
    let (salience_report, karp_certificate, manybody_wave_report) = analyze_salience_karp_manybody(
        nodes,
        &graph,
        &archive,
        &zero_copy_book,
        &steiner_slice,
        &center_turan_report,
        &worktree_schedule,
        &ramsey_erdos_hajnal_report,
    );
    let pq_report = analyze_pq_mode(&graph, &archive, &salience_report, &center_turan_report);
    let zeno_report = analyze_zeno_omni_engine(&graph, &archive, &center_turan_report, &pq_report);
    let world_field_report = analyze_semantic_world_engine(&graph, WORLD_ENGINE_ROLLOUT_STEPS);

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
        center_turan_report,
        worktree_schedule,
        security_report,
        ramsey_report,
        ramsey_erdos_hajnal_report,
        salience_report,
        karp_certificate,
        manybody_wave_report,
        pq_report,
        zeno_report,
        world_field_report,
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
    if !node.is_dir && node.size > MONOLITH_BYTE_THRESHOLD {
        return ErdosLexiconBadge::PureNoiseMonolith;
    }
    if !node.is_dir && node.size < EPSILON_BYTE_THRESHOLD {
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
        let schedule = solve_moser_tardos_auto_scale(&hg, 2, true, 42, 500);

        assert!(schedule.is_conflict_free);
        assert!(schedule.num_worktrees >= 2);
        assert_eq!(
            schedule.worktree_buckets.len(),
            schedule.num_worktrees as usize
        );
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
        hazards[25] = SymbolHazardFlags::NET_IPFS_EGRESS;
        hazards[26] = SymbolHazardFlags::PROC_EXEC;
        hazards[27] = SymbolHazardFlags::SECRET_ENV_READ;
        hazards[28] = SymbolHazardFlags::MCP_HOOK_MUTATE;
        hazards[29] = SymbolHazardFlags::HIGH_ENTROPY_OBFUSCATED;

        let report = scan_erdos_renyi_supply_chain(&graph, &hazards, 4.0);
        assert_eq!(report.regime, PhaseRegime::AnomalousC2Clique);
        assert!(report.critical_scale_n_two_thirds > 1.0);
        assert!(report.spectral_radius_lambda1 > 0.0);
        assert!(report.mean_degree_c_den >= 1);
        assert!(report.molloy_reed_kappa_den >= 1);
        // 25-node path (deg 1,2,...,2,1) + 5-clique on nodes 25..30 (deg 8 each):
        // Σ deg(v)² = 2*1 + 23*4 + 5*64 = 414, Σ deg(v) = 2*1 + 23*2 + 5*8 = 88 -> κ = 414/88 = 207/44 > 2
        assert_eq!(
            (report.molloy_reed_kappa_num, report.molloy_reed_kappa_den),
            (207, 44)
        );
        assert!(report.percolation_kappa_gt_2);
        assert!(report.poisoned_leaf_reaches_core);
        assert!(report
            .percolation_diagnostic
            .contains("giant transitive reachability component forms across package boundaries"));
        assert!(!report.anomalies.is_empty());
        assert!(!report.anomalies[0].pivot_symbols.is_empty());
        assert!(report.anomalies[0].directed_scc_cycles >= 1);

        // Verify IEEE-754 fp16 software decoder
        let one_fp16 = decode_fp16(0x3C00);
        assert!((one_fp16 - 1.0).abs() < 1e-5);

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
        assert!(report.manybody_wave_report.kohn_sham_orbitals_k > 0);
        assert!(report.manybody_wave_report.strang_split_coherence > 0.0);
        assert!(report.karp_certificate.independent_set_alpha > 0);
        assert!(report.pq_report.ambient_density_pq.q >= 1);
        assert!(report.pq_report.holder_3_2_to_3_norm >= 0.0);
        assert!((report.pq_report.liouville_jacobian_det - 1.0).abs() < 1e-12);
        assert!(report.world_field_report.certificate.mass_consistent);
        assert!(
            report
                .world_field_report
                .certificate
                .energy_budget_satisfied
        );
        assert!(
            report
                .world_field_report
                .certificate
                .hallucination_free_after_projection
        );
        assert!(report.world_field_report.fixed_point_converged);
        assert_eq!(
            report
                .world_field_report
                .invention_to_memory_repeat_drift_q16,
            0
        );
    }

    #[test]
    fn test_ramsey_erdos_hajnal_bitboard_bron_kerbosch_and_biclique() {
        // 40 vertices:
        // - Sparse background tree on 0..25
        // - Mutual CyclicDeadlockKnot K_5 in MutableShared on 25..30
        // - Acyclic Erdős-Moser TransitiveTournament T_4 in TraitImpl on 30..34
        // - Almost-Pure Erdős-Hajnal Bi-Clique A={10,11,12} -> B={20,21,22} in CallRef missing (12, 22)
        let n = 40;
        let mut edges = Vec::new();
        for u in 1..25 {
            edges.push(((u - 1) as SymbolId, u as SymbolId, EdgeKind::Import));
        }

        // Plant mutual K_5 in MutableShared (25..30)
        for u in 25..30 {
            for v in 25..30 {
                if u != v {
                    edges.push((u as SymbolId, v as SymbolId, EdgeKind::MutableShared));
                }
            }
        }

        // Plant acyclic Erdős-Moser TransitiveTournament T_4 in TraitImpl (30..34)
        for u in 30..34 {
            for v in (u + 1)..34 {
                edges.push((u as SymbolId, v as SymbolId, EdgeKind::TraitImpl));
            }
        }

        // Plant Almost-Pure Bi-Clique A={10,11,12} -> B={20,21,22} in CallRef missing (12, 22)
        for &a in &[10u32, 11, 12] {
            for &b in &[20u32, 21, 22] {
                if !(a == 12 && b == 22) {
                    edges.push((a, b, EdgeKind::CallRef));
                }
            }
        }

        let graph = CsrSymbolGraph::from_edges(n, &edges, Vec::new());
        let eh = analyze_ramsey_and_erdos_hajnal(&graph, RamseyScanConfig::default());

        assert_eq!(eh.omega_per_color[EdgeKind::MutableShared as usize], 5);
        assert_eq!(eh.omega_per_color[EdgeKind::TraitImpl as usize], 4);
        assert!(!eh.alpha_independent_set.is_empty());
        assert!(eh.erdos_hajnal_exponent_gamma > 0.0);

        // Verify both CyclicDeadlockKnot and TransitiveTournament are classified
        assert!(eh.cliques.iter().any(|c| {
            c.color == EdgeKind::MutableShared
                && c.size_k == 5
                && matches!(c.topology, CliqueTopology::CyclicDeadlockKnot { .. })
                && c.prescription == RefactorPrescription::BreakDeadlockCycle
        }));
        assert!(eh.cliques.iter().any(|c| {
            c.color == EdgeKind::TraitImpl
                && c.size_k == 4
                && c.topology == CliqueTopology::TransitiveTournament
        }));

        // Verify Almost-Pure Bi-Clique detects the missing API conformance edge (12, 22)
        assert!(eh.bi_cliques.iter().any(|bc| {
            bc.color == EdgeKind::CallRef
                && bc.missing_anomaly_edges == vec![(12, 22)]
                && bc.prescription == RefactorPrescription::AuditMissingBiCliqueEdge
        }));
    }
}
