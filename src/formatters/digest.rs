use super::Formatter;
use crate::mem8::regularity::{analyze_the_book_pipeline, PhaseRegime};
use crate::mem8_binary::LoudsSmartTree;
use crate::scanner::{FileNode, TreeStats};
use anyhow::Result;
use sha2::{Digest, Sha256};
use std::io::Write;
use std::path::Path;

pub struct DigestFormatter;
pub struct SmtrCapsuleFormatter;
pub struct TheBookFormatter;

impl Default for DigestFormatter {
    fn default() -> Self {
        Self::new()
    }
}

impl DigestFormatter {
    pub fn new() -> Self {
        Self
    }

    /// Calculate a SHA256 hash of the tree structure for consistency verification
    fn calculate_tree_hash(&self, nodes: &[FileNode]) -> String {
        let mut hasher = Sha256::new();

        // Hash each node's key properties in a deterministic way
        for node in nodes {
            // Hash: depth, name, type (dir/file), size, permissions
            hasher.update(node.depth.to_le_bytes());
            hasher.update(
                node.path
                    .file_name()
                    .unwrap_or_default()
                    .to_string_lossy()
                    .as_bytes(),
            );
            hasher.update([if node.is_dir { 1 } else { 0 }]);
            hasher.update(node.size.to_le_bytes());
            hasher.update(node.permissions.to_le_bytes());
        }

        // Return first 16 chars of hex for brevity
        let result = hasher.finalize();
        hex::encode(&result[..8])
    }
}

impl Formatter for DigestFormatter {
    fn format(
        &self,
        writer: &mut dyn Write,
        nodes: &[FileNode],
        stats: &TreeStats,
        _root_path: &Path,
    ) -> Result<()> {
        // Calculate SHA256 hash of the tree structure
        let tree_hash = self.calculate_tree_hash(nodes);

        // Compute 8B-META-WP-v2.1 LoudsSmartTree sufficient projection & 64B SMTR capsule
        let key = *b"8b-is::smart-tree::smtr-key::v21";
        let raw_tree = LoudsSmartTree::from_file_nodes(nodes, &key);
        let proj_tree = raw_tree.project_sufficient(false);
        let capsule = proj_tree.pack_m8_capsule(&key);
        let book = analyze_the_book_pipeline(nodes);

        // First line: Hash, basic stats, LOUDS/SMTR capsule, and baked-in Erdős Book invariants
        write!(
            writer,
            "HASH: {} F:{} D:{} S:{:x} LOUDS:{}/{} SMTR_V2:{} ERDOS_K:{}/{} KAPPA:{:.2}",
            tree_hash,
            stats.total_files,
            stats.total_dirs,
            stats.total_size,
            proj_tree.node_count,
            raw_tree.node_count,
            hex::encode(capsule),
            book.archive.super_nodes.len(),
            book.archive.outlier_vertices.len(),
            book.security_report.molloy_reed_kappa,
        )?;

        // File type summary (top 5) - super compact
        if !stats.file_types.is_empty() {
            let mut types: Vec<_> = stats.file_types.iter().collect();
            types.sort_by(|a, b| b.1.cmp(a.1));

            let types_str: Vec<String> = types
                .iter()
                .take(5)
                .map(|(ext, count)| format!("{}:{}", ext, count))
                .collect();

            write!(writer, " TYPES: {}", types_str.join(" "))?;
        }

        // Add newline at the end
        writeln!(writer)?;

        Ok(())
    }
}

impl Default for SmtrCapsuleFormatter {
    fn default() -> Self {
        Self::new()
    }
}

impl SmtrCapsuleFormatter {
    pub fn new() -> Self {
        Self
    }
}

impl Formatter for SmtrCapsuleFormatter {
    fn format(
        &self,
        writer: &mut dyn Write,
        nodes: &[FileNode],
        stats: &TreeStats,
        root_path: &Path,
    ) -> Result<()> {
        let key = *b"8b-is::smart-tree::smtr-key::v21";
        let raw_tree = LoudsSmartTree::from_file_nodes(nodes, &key);
        let proj_tree = raw_tree.project_sufficient(false);
        let capsule = proj_tree.pack_m8_capsule(&key);
        let indra_root = proj_tree.subtree_digest(0, &key);

        let payload = serde_json::json!({
            "spec": "8B-META-WP-v2.1",
            "root_path": root_path.display().to_string(),
            "total_files": stats.total_files,
            "total_dirs": stats.total_dirs,
            "louds_raw_nodes": raw_tree.node_count,
            "louds_sufficient_nodes": proj_tree.node_count,
            "louds_popcnt_edges": proj_tree.rank1(255),
            "louds_bits_w0_hex": format!("{:016x}", proj_tree.louds_bits[0]),
            "indra_net_root_hex": hex::encode(indra_root),
            "smtr_m8_capsule_64b_hex": hex::encode(capsule),
            "dharma_mudra_verified": LoudsSmartTree::verify_m8_capsule(&capsule, &key),
        });
        writeln!(writer, "{}", serde_json::to_string_pretty(&payload)?)?;
        Ok(())
    }
}

impl Default for TheBookFormatter {
    fn default() -> Self {
        Self::new()
    }
}

impl TheBookFormatter {
    pub fn new() -> Self {
        Self
    }
}

impl Formatter for TheBookFormatter {
    fn format(
        &self,
        writer: &mut dyn Write,
        nodes: &[FileNode],
        stats: &TreeStats,
        root_path: &Path,
    ) -> Result<()> {
        let report = analyze_the_book_pipeline(nodes);
        let regime_str = match report.security_report.regime {
            PhaseRegime::SubcriticalForest => "SubcriticalForest (kappa < 2.0)",
            PhaseRegime::CriticalWindow => "CriticalWindow (2.0 <= kappa <= 3.5)",
            PhaseRegime::SupercriticalGiant => "SupercriticalGiant (kappa > 3.5)",
            PhaseRegime::AnomalousC2Clique => "AnomalousC2Clique (Chernoff / Multi-Hazard Alert)",
        };

        let steiner_backbone: Vec<serde_json::Value> = report
            .steiner_slice
            .steiner_super_nodes
            .iter()
            .map(|n| {
                serde_json::json!({
                    "cluster_id": n.cluster_id,
                    "erdos_number": n.erdos_number,
                    "resistance_dist": (n.resistance_dist * 1000.0).round() / 1000.0,
                    "anchor_symbols": n.anchor_symbols,
                    "expand_full_symbols": n.expand_full_symbols,
                })
            })
            .collect();

        let payload = serde_json::json!({
            "spec": "ERDOS-THE-BOOK-v1.0",
            "mathematician": "Paul Erdős (1913–1996) — The Magician from Budapest",
            "motto": "Another roof, another proof. Straight from The Book.",
            "root_path": root_path.display().to_string(),
            "total_files": stats.total_files,
            "total_dirs": stats.total_dirs,
            "szemeredi_regularity": {
                "epsilon": report.archive.epsilon,
                "kohayakawa_rodl_energy_q_p": (report.archive.final_energy * 10_000.0).round() / 10_000.0,
                "compression_ratio": (report.archive.compression_ratio * 100.0).round() / 100.0,
                "super_nodes_k": report.archive.super_nodes.len(),
                "regular_super_edges": report.archive.reduced_edges.len(),
                "outlier_v0_vertices": report.archive.outlier_vertices.len(),
                "t8r_exception_edges": report.archive.exception_ledger.len(),
                "m8b_zero_copy_bytes": report.m8b_wire_bytes,
                "t8r_leb128_bytes": report.t8r_wire_bytes,
                "digest_hex": hex::encode(report.archive.model_digest),
            },
            "erdos_steiner_context": {
                "focal_terminals": report.steiner_slice.root_terminals,
                "estimated_llm_tokens": report.steiner_slice.estimated_tokens,
                "exact_v0_bridges": report.steiner_slice.exact_bridge_symbols,
                "steiner_backbone": steiner_backbone,
                "graph_radius_hops": report.center_turan_report.graph_radius_hops,
                "graph_diameter_hops": report.center_turan_report.graph_diameter_hops,
                "erdos_center_clusters_z_g": report.center_turan_report.center_clusters,
                "periphery_clusters": report.center_turan_report.periphery_clusters,
            },
            "turan_extremal_theorem": {
                "turan_ex_k5_bound_edges": report.center_turan_report.turan_ex_k5_edges,
                "turan_density_ratio": (report.center_turan_report.turan_density_ratio * 10000.0).round() / 10000.0,
                "erdos_stone_chromatic_bound_chi": report.center_turan_report.erdos_stone_chromatic_bound,
            },
            "moser_tardos_lll_worktrees": {
                "num_worktrees": report.worktree_schedule.num_worktrees,
                "is_conflict_free": report.worktree_schedule.is_conflict_free,
                "lll_guaranteed_a_priori": report.worktree_schedule.lll_guaranteed_a_priori,
                "total_resamples": report.worktree_schedule.total_resamples,
                "parallel_mis_rounds": report.worktree_schedule.parallel_rounds,
                "max_witness_tree_depth": report.worktree_schedule.max_witness_tree_depth,
                "max_shearer_degree_delta": report.worktree_schedule.max_dependency_degree,
                "unit_to_worktree": report.worktree_schedule.unit_to_worktree,
                "worktree_buckets": report.worktree_schedule.worktree_buckets,
            },
            "erdos_renyi_percolation": {
                "active_shadow_vertices": report.security_report.active_vertices,
                "shadow_edges": report.security_report.shadow_edges,
                "empirical_p": report.security_report.empirical_p,
                "critical_p_c": report.security_report.critical_p_c,
                "critical_scale_n_two_thirds": (report.security_report.critical_scale_n_two_thirds * 1000.0).round() / 1000.0,
                "mean_degree_c": (report.security_report.mean_degree_c * 1000.0).round() / 1000.0,
                "molloy_reed_kappa": (report.security_report.molloy_reed_kappa * 1000.0).round() / 1000.0,
                "spectral_radius_lambda1": (report.security_report.spectral_radius_lambda1 * 1000.0).round() / 1000.0,
                "expected_random_lambda1": (report.security_report.expected_random_lambda1 * 1000.0).round() / 1000.0,
                "giant_component_c1": report.security_report.giant_component_size,
                "second_component_c2_susceptibility": report.security_report.second_component_size,
                "phase_regime": regime_str,
                "anomalous_components": report.security_report.anomalies.len(),
            },
            "ramsey_monochromatic_cliques": {
                "ramsey_r_3_3_3": report.ramsey_report.ramsey_number_r333,
                "red_sync_lock_cliques": report.ramsey_report.red_cliques,
                "blue_mutable_state_cliques": report.ramsey_report.blue_cliques,
                "green_module_coupling_cliques": report.ramsey_report.green_cliques,
                "degeneracy_d": report.ramsey_erdos_hajnal_report.degeneracy_d,
                "omega_per_color": report.ramsey_erdos_hajnal_report.omega_per_color,
                "alpha_independent_set_size": report.ramsey_erdos_hajnal_report.alpha_independent_set.len(),
                "erdos_hajnal_exponent_gamma": (report.ramsey_erdos_hajnal_report.erdos_hajnal_exponent_gamma * 10000.0).round() / 10000.0,
                "expected_random_gamma": (report.ramsey_erdos_hajnal_report.expected_random_gamma * 10000.0).round() / 10000.0,
                "bitboard_cliques": report.ramsey_erdos_hajnal_report.cliques,
                "erdos_hajnal_bi_cliques": report.ramsey_erdos_hajnal_report.bi_cliques,
            },
            "flyxion_salience_ontology": {
                "regime": format!("{:?}", report.salience_report.regime),
                "proxy_disagreement_prob": (report.salience_report.proxy_disagreement_prob * 1000.0).round() / 1000.0,
                "admissibility_change_ratio": (report.salience_report.admissibility_change_ratio * 1000.0).round() / 1000.0,
                "argmin_temporal_symbol": report.salience_report.argmin_temporal_symbol,
                "argmin_relational_symbol": report.salience_report.argmin_relational_symbol,
                "load_bearing_old_corrugated_iron": report.salience_report.load_bearing_old_artifacts,
            },
            "karp_21_np_complete_certificate": {
                "fpt_degeneracy_d": report.karp_certificate.fpt_degeneracy_d,
                "max_clique_omega": report.karp_certificate.max_clique_omega,
                "independent_set_alpha": report.karp_certificate.independent_set_alpha,
                "vertex_cover_2approx_size": report.karp_certificate.vertex_cover_2approx_size,
                "feedback_arc_set_size": report.karp_certificate.feedback_arc_set_size,
                "chromatic_number_lll": report.karp_certificate.chromatic_number_lll,
                "clique_cover_clusters_k": report.karp_certificate.clique_cover_clusters_k,
                "steiner_tree_2approx_clusters": report.karp_certificate.steiner_tree_2approx_clusters,
                "knapsack_budget_utilization": (report.karp_certificate.knapsack_budget_utilization * 1000.0).round() / 1000.0,
            },
            "quantum_manybody_dft_strang_wave": {
                "kohn_sham_orbitals_k": report.manybody_wave_report.kohn_sham_orbitals_k,
                "lno_t8r_correlations": report.manybody_wave_report.lno_t8r_correlations,
                "many_body_speedup_factor": (report.manybody_wave_report.many_body_speedup_factor * 100.0).round() / 100.0,
                "kohn_sham_exchange_energy": (report.manybody_wave_report.kohn_sham_exchange_energy * 1000.0).round() / 1000.0,
                "strang_split_coherence": (report.manybody_wave_report.strang_split_coherence * 10000.0).round() / 10000.0,
                "dominant_salience_orbitals": report.manybody_wave_report.dominant_salience_orbitals,
            },
            "erdos_lexicon": {
                "straight_from_the_book": report.lexicon_counts.0,
                "epsilons_utility_helpers": report.lexicon_counts.1,
                "legally_dead_orphans": report.lexicon_counts.2,
                "pure_noise_sf_monoliths": report.lexicon_counts.3,
            }
        });
        writeln!(writer, "{}", serde_json::to_string_pretty(&payload)?)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::scanner::{FileCategory, FileNode, FileType, FilesystemType, TreeStats};
    use std::path::PathBuf;
    use std::time::SystemTime;

    fn sample_node(name: &str, is_dir: bool, is_ignored: bool) -> FileNode {
        FileNode {
            path: PathBuf::from(name),
            is_dir,
            size: 128,
            permissions: 0o644,
            uid: 501,
            gid: 20,
            modified: SystemTime::UNIX_EPOCH,
            is_symlink: false,
            is_hidden: false,
            permission_denied: false,
            is_ignored,
            depth: if is_dir { 0 } else { 1 },
            file_type: if is_dir {
                FileType::Directory
            } else {
                FileType::RegularFile
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
        }
    }

    #[test]
    fn test_digest_and_smtr_capsule_formatter_emit_verified_capsule() {
        let nodes = vec![
            sample_node("smart-tree", true, false),
            sample_node("lib.rs", false, false),
            sample_node("target", true, true),
        ];
        let stats = TreeStats::default();

        let mut digest_out = Vec::new();
        DigestFormatter::new()
            .format(&mut digest_out, &nodes, &stats, Path::new("."))
            .unwrap();
        let digest_str = String::from_utf8(digest_out).unwrap();
        assert!(digest_str.contains("LOUDS:2/3"));
        assert!(digest_str.contains("SMTR_V2:534d545202"));

        let mut smtr_out = Vec::new();
        SmtrCapsuleFormatter::new()
            .format(&mut smtr_out, &nodes, &stats, Path::new("."))
            .unwrap();
        let val: serde_json::Value = serde_json::from_slice(&smtr_out).unwrap();
        assert_eq!(val["spec"], "8B-META-WP-v2.1");
        assert_eq!(val["louds_raw_nodes"], 3);
        assert_eq!(val["louds_sufficient_nodes"], 2);
        assert_eq!(val["dharma_mudra_verified"], true);
    }
}
