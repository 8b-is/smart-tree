use super::Formatter;
use crate::mem8::regularity::{analyze_the_book_pipeline, run_ultrabackyard_laps, PhaseRegime};
use crate::mem8_binary::LoudsSmartTree;
use crate::scanner::{FileNode, TreeStats};
use anyhow::Result;
use sha2::{Digest, Sha256};
use std::io::Write;
use std::path::Path;

pub struct DigestFormatter;
pub struct SmtrCapsuleFormatter;
pub struct TheBookFormatter;
pub struct PqModeFormatter;
pub struct UltraBackyardFormatter;
pub struct ZenoModeFormatter;

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
            PhaseRegime::SubcriticalForest => "SubcriticalForest (kappa < 2)",
            PhaseRegime::CriticalWindow => "CriticalWindow (kappa == 2)",
            PhaseRegime::SupercriticalGiant => {
                "SupercriticalGiant (kappa > 2: Giant Transitive Reachability Across Package Boundaries)"
            }
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
                "complexity": "O(|V| + |E_SHADOW|)",
                "active_shadow_vertices": report.security_report.active_vertices,
                "shadow_edges": report.security_report.shadow_edges,
                "empirical_p": report.security_report.empirical_p,
                "critical_p_c": report.security_report.critical_p_c,
                "critical_scale_n_two_thirds": (report.security_report.critical_scale_n_two_thirds * 1000.0).round() / 1000.0,
                "mean_degree_c_pq": format!("{}/{}", report.security_report.mean_degree_c_num, report.security_report.mean_degree_c_den),
                "mean_degree_c": (report.security_report.mean_degree_c * 1000.0).round() / 1000.0,
                "molloy_reed_kappa_pq": format!("{}/{}", report.security_report.molloy_reed_kappa_num, report.security_report.molloy_reed_kappa_den),
                "molloy_reed_kappa": (report.security_report.molloy_reed_kappa * 1000.0).round() / 1000.0,
                "percolation_kappa_gt_2": report.security_report.percolation_kappa_gt_2,
                "poisoned_leaf_reaches_core": report.security_report.poisoned_leaf_reaches_core,
                "percolation_diagnostic": report.security_report.percolation_diagnostic,
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
            "pq_rational_symplectic_mode": {
                "ambient_density_pq": report.pq_report.ambient_density_pq.display_ratio(),
                "molloy_reed_kappa_pq": report.pq_report.molloy_reed_kappa_pq.display_ratio(),
                "turan_k5_ratio_pq": report.pq_report.turan_k5_ratio_pq.display_ratio(),
                "kohayakawa_rodl_energy_pq": report.pq_report.kohayakawa_rodl_energy_pq.display_ratio(),
                "holder_conjugate_pair": format!("(L_{}, L_{})", report.pq_report.holder_p_ratio, report.pq_report.holder_q_ratio),
                "holder_3_2_to_3_norm": (report.pq_report.holder_3_2_to_3_norm * 10000.0).round() / 10000.0,
                "hilbert_l2_norm": (report.pq_report.hilbert_l2_norm * 10000.0).round() / 10000.0,
                "holder_hub_concentration_ratio": (report.pq_report.holder_hub_concentration_ratio * 10000.0).round() / 10000.0,
                "zarankiewicz_z_2_2_bound": report.pq_report.zarankiewicz_z_2_2_bound,
                "zarankiewicz_z_3_3_bound": report.pq_report.zarankiewicz_z_3_3_bound,
                "zarankiewicz_k22_guaranteed": report.pq_report.zarankiewicz_k22_guaranteed,
                "zarankiewicz_k33_guaranteed": report.pq_report.zarankiewicz_k33_guaranteed,
                "symplectic_initial_h0": (report.pq_report.symplectic_initial_h0 * 10000.0).round() / 10000.0,
                "symplectic_final_h": (report.pq_report.symplectic_final_h * 10000.0).round() / 10000.0,
                "symplectic_energy_drift": report.pq_report.symplectic_energy_drift,
                "liouville_jacobian_det": report.pq_report.liouville_jacobian_det,
                "phase_space_orbitals": report.pq_report.phase_space_orbitals,
            },
            "ultra_omni_zeno_v11": {
                "spec": report.zeno_report.spec,
                "litany_wire_seal16": report.zeno_report.upz_wupz.litany_wire_seal16_hex,
                "litany_wire_verified": report.zeno_report.upz_wupz.litany_wire_verified,
                "qat_single_edit_reuse_pq": report.zeno_report.upz_wupz.qat_single_edit_reuse_pq,
                "gaia_b158_compression_pq": report.zeno_report.upz_wupz.gaia_b158_compression_pq,
                "ternary_gemm_i32": report.zeno_report.eight_b_is_engines.ternary_gemm_i32_accumulator,
                "spherepop_value": report.zeno_report.eight_b_is_engines.spherepop_innermost_first_value,
                "admissibility_ct000_verified": report.zeno_report.eight_b_is_engines.admissibility_ct000_epistemic_overlap_verified,
                "cpu_isa_profile": report.zeno_report.os_agent_matrix.cpu_isa_profile,
            },
            "semantic_world_engine": &report.world_field_report,
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

impl Default for PqModeFormatter {
    fn default() -> Self {
        Self::new()
    }
}

impl PqModeFormatter {
    pub fn new() -> Self {
        Self
    }
}

impl Formatter for PqModeFormatter {
    fn format(
        &self,
        writer: &mut dyn Write,
        nodes: &[FileNode],
        stats: &TreeStats,
        root_path: &Path,
    ) -> Result<()> {
        let report = analyze_the_book_pipeline(nodes);
        let pq = &report.pq_report;
        let payload = serde_json::json!({
            "spec": "SOVEREIGN-PQ-MODE-v1.0",
            "mode": "p/q (Exact Coprime Rational + Hölder L_p/L_q Dual + Zarankiewicz z(n;p,q) + Symplectic (p,q) Leapfrog)",
            "root_path": root_path.display().to_string(),
            "total_files": stats.total_files,
            "total_dirs": stats.total_dirs,
            "exact_coprime_rationals_pq": {
                "ambient_density": {
                    "ratio": pq.ambient_density_pq.display_ratio(),
                    "p": pq.ambient_density_pq.p,
                    "q": pq.ambient_density_pq.q,
                    "q32_fixed": pq.ambient_density_pq.q32_fixed,
                },
                "molloy_reed_kappa": {
                    "ratio": pq.molloy_reed_kappa_pq.display_ratio(),
                    "p": pq.molloy_reed_kappa_pq.p,
                    "q": pq.molloy_reed_kappa_pq.q,
                    "q32_fixed": pq.molloy_reed_kappa_pq.q32_fixed,
                },
                "turan_k5_density_ratio": {
                    "ratio": pq.turan_k5_ratio_pq.display_ratio(),
                    "p": pq.turan_k5_ratio_pq.p,
                    "q": pq.turan_k5_ratio_pq.q,
                },
                "kohayakawa_rodl_energy_q_p": {
                    "ratio": pq.kohayakawa_rodl_energy_pq.display_ratio(),
                    "p": pq.kohayakawa_rodl_energy_pq.p,
                    "q": pq.kohayakawa_rodl_energy_pq.q,
                },
            },
            "holder_conjugate_dual_spectral_oracle": {
                "identity": "1/p + 1/q = 1",
                "p_exponent": pq.holder_p_ratio,
                "q_exponent": pq.holder_q_ratio,
                "bollobas_riordan_3_2_to_3_norm": (pq.holder_3_2_to_3_norm * 10000.0).round() / 10000.0,
                "hilbert_2_to_2_norm": (pq.hilbert_l2_norm * 10000.0).round() / 10000.0,
                "hub_concentration_ratio": (pq.holder_hub_concentration_ratio * 10000.0).round() / 10000.0,
            },
            "kovari_sos_turan_zarankiewicz_bounds": {
                "z_n_2_2_c4_free_bound": pq.zarankiewicz_z_2_2_bound,
                "z_n_3_3_k33_free_bound": pq.zarankiewicz_z_3_3_bound,
                "k22_biclique_guaranteed": pq.zarankiewicz_k22_guaranteed,
                "k33_biclique_guaranteed": pq.zarankiewicz_k33_guaranteed,
            },
            "hamiltonian_symplectic_phase_space": {
                "integrator": "Störmer–Verlet 2nd-Order Symplectic Leapfrog (S=16, dt=0.025)",
                "initial_hamiltonian_h0": (pq.symplectic_initial_h0 * 10000.0).round() / 10000.0,
                "final_hamiltonian_h": (pq.symplectic_final_h * 10000.0).round() / 10000.0,
                "relative_energy_drift": pq.symplectic_energy_drift,
                "liouville_jacobian_det": pq.liouville_jacobian_det,
                "phase_space_orbitals": pq.phase_space_orbitals,
            },
        });
        writeln!(writer, "{}", serde_json::to_string_pretty(&payload)?)?;
        Ok(())
    }
}

impl Default for UltraBackyardFormatter {
    fn default() -> Self {
        Self::new()
    }
}

impl UltraBackyardFormatter {
    pub fn new() -> Self {
        Self
    }
}

impl Formatter for UltraBackyardFormatter {
    fn format(
        &self,
        writer: &mut dyn Write,
        nodes: &[FileNode],
        stats: &TreeStats,
        root_path: &Path,
    ) -> Result<()> {
        let report = run_ultrabackyard_laps(nodes, 6);
        let payload = serde_json::json!({
            "spec": report.spec,
            "motto": report.motto,
            "root_path": root_path.display().to_string(),
            "total_files": stats.total_files,
            "total_dirs": stats.total_dirs,
            "do_stages": report.do_stages,
            "total_laps": report.total_laps,
            "total_symbols": report.total_symbols,
            "total_edges": report.total_edges,
            "best_lap": report.best_lap,
            "best_symbols_per_sec": report.best_symbols_per_sec,
            "best_epsilon": report.best_epsilon,
            "best_bound_worktrees_c": report.best_bound_worktrees_c,
            "constellation_wiring": report.wiring,
            "laps": report.laps,
        });
        writeln!(writer, "{}", serde_json::to_string_pretty(&payload)?)?;
        Ok(())
    }
}

impl Default for ZenoModeFormatter {
    fn default() -> Self {
        Self::new()
    }
}

impl ZenoModeFormatter {
    pub fn new() -> Self {
        Self
    }
}

impl Formatter for ZenoModeFormatter {
    fn format(
        &self,
        writer: &mut dyn Write,
        nodes: &[FileNode],
        stats: &TreeStats,
        root_path: &Path,
    ) -> Result<()> {
        let book_report = analyze_the_book_pipeline(nodes);
        let zeno = &book_report.zeno_report;
        let payload = serde_json::json!({
            "spec": zeno.spec,
            "version": zeno.version,
            "header_banner": zeno.header_banner,
            "root_path": root_path.display().to_string(),
            "total_files": stats.total_files,
            "total_dirs": stats.total_dirs,
            "upz_wupz_core": zeno.upz_wupz,
            "eight_b_is_engines": zeno.eight_b_is_engines,
            "os_agent_agnostic_matrix": zeno.os_agent_matrix,
            "semantic_world_engine": &book_report.world_field_report,
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

        let mut pq_out = Vec::new();
        PqModeFormatter::new()
            .format(&mut pq_out, &nodes, &stats, Path::new("."))
            .unwrap();
        let pq_val: serde_json::Value = serde_json::from_slice(&pq_out).unwrap();
        assert_eq!(pq_val["spec"], "SOVEREIGN-PQ-MODE-v1.0");
        assert_eq!(
            pq_val["hamiltonian_symplectic_phase_space"]["liouville_jacobian_det"],
            1.0
        );
    }
}
