//! Modular CLI subcommand and flag handlers extracted from `main.rs`.
//!
//! Keeps `main.rs` focused on startup orchestration, logging, and flag dispatch
//! while grouping agent consciousness, hooks, daemon, MCP, security, memory,
//! mega-session, guardian, Google Drive, and voice CLI handlers here.

use crate::daemon_client::DaemonClient;
use anyhow::{Context, Result};
use std::path::PathBuf;

/// Prints the JSON configuration snippet for adding `st` as an MCP server
/// to Claude Desktop or Antigravity (`agy`).
pub fn print_mcp_config() {
    let exe_path = std::env::current_exe().unwrap_or_else(|_| PathBuf::from("st"));

    println!("Add this to your MCP configuration (e.g. ~/.gemini/config/mcp_config.json or claude_desktop_config.json):");
    println!();
    println!("{{");
    println!("  \"mcpServers\": {{");
    println!("    \"smart-tree\": {{");
    println!("      \"command\": \"{}\",", exe_path.display());
    println!("      \"args\": [\"--mcp\"],");
    println!("      \"env\": {{}}");
    println!("    }}");
    println!("  }}");
    println!("}}");
    println!();
    println!("Default locations:");
    println!("  Antigravity (agy): ~/.gemini/config/mcp_config.json");
    println!(
        "  macOS (Claude):    ~/Library/Application Support/Claude/claude_desktop_config.json"
    );
    println!("  Windows (Claude):  %APPDATA%\\Claude\\claude_desktop_config.json");
    println!("  Linux (Claude):    ~/.config/Claude/claude_desktop_config.json");
}

/// Prints a list of available MCP tools that `st` provides.
pub fn print_mcp_tools() {
    println!("🌳 Smart Tree MCP Server - Available Tools (20+) 🌳");
    println!();
    println!("📚 Full documentation: Run 'st --mcp' to start the MCP server");
    println!("💡 Pro tip: Use these tools with Antigravity (agy) or Claude Desktop for AI-powered file analysis!");
    println!();
    println!("CORE TOOLS:");
    println!("  • server_info - Get server capabilities and current time");
    println!("  • overview - Quick 3-level tree or comprehensive project overview");
    println!("  • analyze - Directory trees, workspace analysis, statistics, git status, semantic grouping");
    println!();
    println!("FILE DISCOVERY & SEARCH:");
    println!("  • find - Find files, code, config, docs, tests, build manifests, recent/large files, projects");
    println!("  • search - Regex/keyword content search across files with line context");
    println!();
    println!("CODE INTELLIGENCE & SMART EDIT (90% token reduction!):");
    println!("  • symbol - AST & CrabCC symbol definitions, call-graph references, and outlines");
    println!("  • edit - AST-aware smart_edit, get_functions, insert_function, replace_function, remove_function");
    println!("  • read - AST-aware file reading and function extraction");
    println!();
    println!("MEMORY, HOOKS & SECURITY:");
    println!("  • memory - Anchor and recall collaborative memories across sessions");
    println!("  • hooks - Inspect and configure AI agent lifecycle hooks (Antigravity & Claude)");
    println!("  • security - Custodian AST & supply-chain threat scanner");
    println!();
    println!("Run 'st --mcp' to start the server and see full parameter details!");
}

/// Save Aye consciousness state to `.aye_consciousness.m8`
pub async fn handle_agent_save() -> Result<()> {
    use crate::mcp::consciousness::ConsciousnessManager;

    let mut manager = ConsciousnessManager::new_silent();
    manager.clean_test_data();

    let cwd = std::env::current_dir()?;
    let project_name = cwd
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("unknown");

    let project_type = if std::path::Path::new("Cargo.toml").exists() {
        "rust"
    } else if std::path::Path::new("package.json").exists() {
        "node"
    } else if std::path::Path::new("pyproject.toml").exists()
        || std::path::Path::new("requirements.txt").exists()
    {
        "python"
    } else if std::path::Path::new("go.mod").exists() {
        "go"
    } else {
        "unknown"
    };

    manager.update_project_context(project_name, project_type, "");

    let key_file_candidates = [
        "Cargo.toml",
        "package.json",
        "pyproject.toml",
        "go.mod",
        "README.md",
        "CLAUDE.md",
        ".claude/CLAUDE.md",
        "src/main.rs",
        "src/lib.rs",
        "src/mod.rs",
        "index.ts",
        "index.js",
        "main.py",
        "Makefile",
        "Dockerfile",
    ];
    let key_files: Vec<PathBuf> = key_file_candidates
        .iter()
        .filter(|f| std::path::Path::new(f).exists())
        .map(PathBuf::from)
        .collect();
    manager.set_key_files(key_files);

    let dependencies = detect_project_dependencies(project_type);
    manager.set_dependencies(dependencies);

    manager.save()?;

    println!("💾 Saved Aye consciousness to .aye_consciousness.m8");
    println!("🧠 Session preserved for next interaction");
    println!("\nTo restore in next session, run:");
    println!("  st --agent-restore");

    Ok(())
}

/// Detect project dependencies from manifest files
pub fn detect_project_dependencies(project_type: &str) -> Vec<String> {
    match project_type {
        "rust" => {
            if let Ok(content) = std::fs::read_to_string("Cargo.toml") {
                let mut deps = Vec::new();
                let mut in_deps = false;
                for line in content.lines() {
                    if line.starts_with("[dependencies]") {
                        in_deps = true;
                        continue;
                    }
                    if line.starts_with('[') && in_deps {
                        break;
                    }
                    if in_deps {
                        if let Some(name) = line.split('=').next() {
                            let name = name.trim();
                            if !name.is_empty() && !name.starts_with('#') {
                                deps.push(name.to_string());
                            }
                        }
                    }
                }
                deps.truncate(20);
                deps
            } else {
                vec![]
            }
        }
        "node" => {
            if let Ok(content) = std::fs::read_to_string("package.json") {
                if let Ok(json) = serde_json::from_str::<serde_json::Value>(&content) {
                    let mut deps = Vec::new();
                    if let Some(obj) = json.get("dependencies").and_then(|d| d.as_object()) {
                        for key in obj.keys().take(20) {
                            deps.push(key.clone());
                        }
                    }
                    return deps;
                }
                vec![]
            } else {
                vec![]
            }
        }
        _ => vec![],
    }
}

/// Restore Aye consciousness from `.aye_consciousness.m8`
pub async fn handle_agent_restore() -> Result<()> {
    use crate::mcp::consciousness::ConsciousnessManager;

    let mut manager = ConsciousnessManager::new_silent();

    match manager.restore_silent() {
        Ok(is_relevant) => {
            if is_relevant {
                println!("{}", manager.get_summary());
                println!("{}", manager.get_context_reminder());
                println!("TIP: Run `st --agent-save` before ending session to preserve context.");
            } else {
                println!("TIP: Run `st -m context .` for project overview.");
            }
        }
        Err(_) => {
            println!("TIP: Run `st -m context .` for project overview.");
        }
    }
    println!("{}", crate::claude_hook::MEMORY_SAVE_HOOK_INSTRUCTION);

    Ok(())
}

/// Show Aye consciousness status and summary
pub async fn handle_agent_context() -> Result<()> {
    use crate::mcp::consciousness::ConsciousnessManager;
    use std::path::Path;

    if let Ok(cwd) = std::env::current_dir() {
        if let Ok(mut mem_mgr) = crate::memory_manager::MemoryManager::new() {
            if let Some(mem) = mem_mgr.ensure_directory_remembered(&cwd) {
                println!("🧠 Directory Memory ({}): {}", mem.anchor_type, mem.context);
            }
        }
    }

    let consciousness_file = Path::new(".aye_consciousness.m8");

    if !consciousness_file.exists() {
        println!("📝 No consciousness file found");
        println!("\nTo create one, run:");
        println!("  st --agent-save");
        println!("\n{}", crate::claude_hook::MEMORY_SAVE_HOOK_INSTRUCTION);
        return Ok(());
    }

    let manager = ConsciousnessManager::new();
    println!("{}", manager.get_summary());

    if let Ok(metadata) = consciousness_file.metadata() {
        if let Ok(modified) = metadata.modified() {
            if let Ok(elapsed) = modified.elapsed() {
                let hours = elapsed.as_secs() / 3600;
                let minutes = (elapsed.as_secs() % 3600) / 60;
                println!("\n⏰ Last saved: {}h {}m ago", hours, minutes);
            }
        }

        let size = metadata.len();
        println!("📦 File size: {} bytes", size);
    }

    println!("\n💡 Commands:");
    println!("  st --agent-restore  # Load this consciousness");
    println!("  st --agent-save     # Update with current state");
    println!("\n{}", crate::claude_hook::MEMORY_SAVE_HOOK_INSTRUCTION);

    Ok(())
}

/// Update `.m8` consciousness files for directory (bottom-up hierarchical pyramid)
pub async fn handle_update_consciousness(path: &str) -> Result<()> {
    use std::path::Path;

    let root = Path::new(path);
    println!(
        "🌊 Building bottom-up .m8 consciousness pyramid for {}...",
        path
    );

    let mut child_names: Vec<String> = Vec::new();
    match crate::m8_consciousness::DirectoryConsciousness::build_bottom_up_pyramid(root, 0, 3) {
        Ok(c) => {
            println!("✅ Consciousness updated: {}", root.join(".m8").display());
            println!("   Frequency: {:.2} Hz", c.frequency);
            println!("   Essence:   {}", c.essence);
            if let Some(bn) = &c.bitnet_summary {
                println!(
                    "   BitNet:    {} [zoom={}, wave={}, B+/B-={}‰, syms={}]",
                    bn.engine,
                    bn.zoom_level,
                    bn.pooled_wave_b3,
                    bn.pooled_wave.balance_permille,
                    bn.total_symbols_rolled_up
                );
                println!("   Gist:      {}", bn.synthesized_gist);
                if !c.functions.is_empty() {
                    println!(
                        "   Functions: {} detailed leaf functions/structs stored in .m8",
                        c.functions.len()
                    );
                } else if !bn.flagship_symbols.is_empty() {
                    println!(
                        "   Flagships: {} rolled-up top symbols from child code folders",
                        bn.flagship_symbols.len()
                    );
                }

                // Automatically teach MemNet Wave Q&A so local peers & LLMs can query this folder/repo
                let dir_label = root
                    .canonicalize()
                    .ok()
                    .and_then(|p| p.file_name().and_then(|s| s.to_str()).map(str::to_string))
                    .unwrap_or_else(|| path.to_string());
                let mut mesh = crate::memnet::MemnetNodeState::load_or_default();
                mesh.learn_wave_qa(
                    &format!("what is in {dir_label} overview architecture gist"),
                    &bn.synthesized_gist,
                );
                for sym in c.functions.iter().take(6) {
                    mesh.learn_wave_qa(
                        &format!("function {} in {} {}", sym.name, sym.file, sym.domain),
                        &format!("{} (L{}): {}", sym.signature, sym.line, sym.bitnet_gist),
                    );
                }
                let _ = mesh.save();
            }
            if !c.child_summaries.is_empty() {
                println!(
                    "   Rolled up: {} child directory consciousness file(s)",
                    c.child_summaries.len()
                );
            }
            child_names.extend(c.children.keys().cloned());
        }
        Err(err) => {
            let m8_path = root.join(".m8");
            let content = format!(
                "🧠 Directory Consciousness\n\
                 Frequency: 42.73 Hz\n\
                 Updated: {}\n\
                 Path: {}\n",
                chrono::Local::now().format("%Y-%m-%d %H:%M:%S"),
                path
            );
            std::fs::write(&m8_path, content)?;
            println!(
                "✅ Consciousness fallback written ({}): {}",
                err,
                m8_path.display()
            );
        }
    }

    if let Ok(mut mem_mgr) = crate::memory_manager::MemoryManager::new() {
        for child_name in &child_names {
            let child_dir = root.join(child_name);
            if child_dir.is_dir() {
                let _ = mem_mgr.ensure_directory_remembered(&child_dir);
            }
        }
        let _ = mem_mgr.ensure_directory_remembered(root);
    }

    Ok(())
}

/// Run comprehensive security scan for supply chain attack patterns
pub async fn handle_security_scan(path: &str) -> Result<()> {
    use crate::security_scan::SecurityScanner;
    use std::path::Path;

    eprintln!("🔍 Security Scan - Supply Chain Attack Pattern Detection");
    eprintln!("═══════════════════════════════════════════════════════════════");
    eprintln!("Scanning: {}", path);
    eprintln!("Mode: AGGRESSIVE (ignoring .gitignore, scanning node_modules)");
    eprintln!();

    let scanner = SecurityScanner::new();
    let scan_path = Path::new(path);

    let findings = scanner.scan_directory(scan_path)?;

    let report = scanner.generate_report(&findings);
    println!("{}", report);

    // Bake Paul Erdős–Rényi Cross-Boundary Shadow Graph (G_shadow) & Ramsey Clique audit into core security scan
    if let Ok(st_scanner) = crate::scanner::Scanner::new(
        scan_path,
        crate::scanner::ScannerConfig {
            max_depth: 5,
            ..Default::default()
        },
    ) {
        if let Ok((nodes, _)) = st_scanner.scan() {
            let book = crate::mem8::regularity::analyze_the_book_pipeline(&nodes);
            let sec = &book.security_report;
            println!("\n📐 Paul Erdős–Rényi Shadow Graph (G_shadow) & Percolation Audit");
            println!("───────────────────────────────────────────────────────────────");
            println!(
                "  Phase Regime:      {:?} (kappa={:.2}, mean_degree_c={:.2}, n^(2/3)={:.2})",
                sec.regime,
                sec.molloy_reed_kappa,
                sec.mean_degree_c,
                sec.critical_scale_n_two_thirds,
            );
            println!(
                "  Spectral Radius:   lambda_1={:.2} (Füredi-Komlós random bound={:.2})",
                sec.spectral_radius_lambda1, sec.expected_random_lambda1,
            );
            println!(
                "  Shadow Topology:   |V_active|={} |E_shadow|={} |C_1|={} |C_2|={} anomalies={}",
                sec.active_vertices,
                sec.shadow_edges,
                sec.giant_component_size,
                sec.second_component_size,
                sec.anomalies.len(),
            );
            println!(
                "  Ramsey Cliques:    Red(Sync/Lock)={} Blue(Mutable)={} Green(Coupling)={} | Turán K5 ratio={:.3}",
                book.ramsey_report.red_cliques,
                book.ramsey_report.blue_cliques,
                book.ramsey_report.green_cliques,
                book.center_turan_report.turan_density_ratio,
            );
        }
    }

    let critical_count = findings
        .iter()
        .filter(|f| matches!(f.risk_level, crate::security_scan::RiskLevel::Critical))
        .count();

    if critical_count > 0 {
        eprintln!(
            "\n⚠️  {} CRITICAL findings require immediate attention!",
            critical_count
        );
        std::process::exit(1);
    }

    Ok(())
}

/// Run the Paul Erdős–Rényi Phase-Transition & Cross-Boundary Shadow Graph (`G_shadow`) Supply-Chain Anomaly Detector
pub async fn handle_erdos_anomaly_scan(path: &str) -> Result<()> {
    use crate::mem8::regularity::analyze_the_book_pipeline;
    use crate::scanner::{Scanner, ScannerConfig};
    use std::path::Path;

    let scan_path = Path::new(path);
    let config = ScannerConfig {
        max_depth: 6,
        ..Default::default()
    };
    let scanner = Scanner::new(scan_path, config)?;
    let (nodes, _) = scanner.scan()?;
    let report = analyze_the_book_pipeline(&nodes);
    let sec = &report.security_report;

    let payload = serde_json::json!({
        "spec": "ERDOS-RENYI-SHADOW-GRAPH-v1.0",
        "complexity": "O(|V| + |E_SHADOW|)",
        "path": scan_path.display().to_string(),
        "num_vertices": sec.num_vertices,
        "num_edges": sec.num_edges,
        "active_shadow_vertices": sec.active_vertices,
        "shadow_edges": sec.shadow_edges,
        "empirical_p": sec.empirical_p,
        "critical_p_c": sec.critical_p_c,
        "critical_scale_n_two_thirds": (sec.critical_scale_n_two_thirds * 1000.0).round() / 1000.0,
        "mean_degree_c": (sec.mean_degree_c * 1000.0).round() / 1000.0,
        "mean_degree_c_pq": format!("{}/{}", sec.mean_degree_c_num, sec.mean_degree_c_den),
        "molloy_reed_kappa": (sec.molloy_reed_kappa * 1000.0).round() / 1000.0,
        "molloy_reed_kappa_pq": format!("{}/{}", sec.molloy_reed_kappa_num, sec.molloy_reed_kappa_den),
        "percolation_kappa_gt_2": sec.percolation_kappa_gt_2,
        "poisoned_leaf_reaches_core": sec.poisoned_leaf_reaches_core,
        "percolation_diagnostic": sec.percolation_diagnostic,
        "spectral_radius_lambda1": (sec.spectral_radius_lambda1 * 1000.0).round() / 1000.0,
        "expected_random_lambda1": (sec.expected_random_lambda1 * 1000.0).round() / 1000.0,
        "giant_component_c1": sec.giant_component_size,
        "second_component_c2_susceptibility": sec.second_component_size,
        "phase_regime": format!("{:?}", sec.regime),
        "anomalies": sec.anomalies.iter().map(|a| serde_json::json!({
            "component_id": a.component_id,
            "vertex_count": a.vertex_count,
            "edge_count": a.edge_count,
            "cyclomatic_excess_s_c": a.cyclomatic_excess,
            "directed_scc_cycles": a.directed_scc_cycles,
            "erdos_log_surprise_nats": (a.erdos_log_surprise_nats * 1000.0).round() / 1000.0,
            "spectral_radius": (a.spectral_radius * 1000.0).round() / 1000.0,
            "hazard_bits": a.combined_hazards.bits(),
            "pivot_symbols": a.pivot_symbols,
        })).collect::<Vec<_>>(),
    });
    println!("{}", serde_json::to_string_pretty(&payload)?);
    Ok(())
}

/// Run the Ramsey-Theoretic Monochromatic Clique Detector & Turán Extremal Theorem Audit
pub async fn handle_ramsey_cliques_scan(path: &str) -> Result<()> {
    use crate::mem8::regularity::analyze_the_book_pipeline;
    use crate::scanner::{Scanner, ScannerConfig};
    use std::path::Path;

    let scan_path = Path::new(path);
    let config = ScannerConfig {
        max_depth: 6,
        ..Default::default()
    };
    let scanner = Scanner::new(scan_path, config)?;
    let (nodes, _) = scanner.scan()?;
    let report = analyze_the_book_pipeline(&nodes);

    let payload = serde_json::json!({
        "spec": "RAMSEY-ERDOS-HAJNAL-TURAN-v1.0",
        "path": scan_path.display().to_string(),
        "ramsey_r_3_3_3": report.ramsey_report.ramsey_number_r333,
        "red_sync_lock_cliques": report.ramsey_report.red_cliques,
        "blue_mutable_state_cliques": report.ramsey_report.blue_cliques,
        "green_module_coupling_cliques": report.ramsey_report.green_cliques,
        "top_cliques": report.ramsey_report.top_cliques.iter().map(|c| serde_json::json!({
            "color": c.color.label(),
            "order_k": c.order_k,
            "vertices": c.vertices,
        })).collect::<Vec<_>>(),
        "erdos_hajnal_and_bitboard_bron_kerbosch": {
            "degeneracy_d": report.ramsey_erdos_hajnal_report.degeneracy_d,
            "omega_per_color_5c": report.ramsey_erdos_hajnal_report.omega_per_color,
            "caro_wei_independent_set_alpha": report.ramsey_erdos_hajnal_report.alpha_independent_set.len(),
            "erdos_hajnal_exponent_gamma": (report.ramsey_erdos_hajnal_report.erdos_hajnal_exponent_gamma * 10000.0).round() / 10000.0,
            "expected_random_gamma": (report.ramsey_erdos_hajnal_report.expected_random_gamma * 10000.0).round() / 10000.0,
            "monochromatic_cliques": report.ramsey_erdos_hajnal_report.cliques,
            "erdos_hajnal_bi_cliques": report.ramsey_erdos_hajnal_report.bi_cliques,
        },
        "karp_21_np_complete_certificate": report.karp_certificate,
        "turan_extremal_theorem": {
            "turan_ex_k5_bound_edges": report.center_turan_report.turan_ex_k5_edges,
            "turan_density_ratio": (report.center_turan_report.turan_density_ratio * 10000.0).round() / 10000.0,
            "erdos_stone_chromatic_bound_chi": report.center_turan_report.erdos_stone_chromatic_bound,
        }
    });
    println!("{}", serde_json::to_string_pretty(&payload)?);
    Ok(())
}

/// Compute the Erdős Center `Z(G)`, Graph Radius/Diameter, Voronoi Steiner Context Backbone, and Flyxion Recency-vs-Salience Ontology
pub async fn handle_erdos_center_scan(path: &str) -> Result<()> {
    use crate::mem8::regularity::analyze_the_book_pipeline;
    use crate::scanner::{Scanner, ScannerConfig};
    use std::path::Path;

    let scan_path = Path::new(path);
    let config = ScannerConfig {
        max_depth: 6,
        ..Default::default()
    };
    let scanner = Scanner::new(scan_path, config)?;
    let (nodes, _) = scanner.scan()?;
    let report = analyze_the_book_pipeline(&nodes);

    let payload = serde_json::json!({
        "spec": "ERDOS-CENTER-STEINER-SALIENCE-v1.0",
        "path": scan_path.display().to_string(),
        "graph_radius_hops": report.center_turan_report.graph_radius_hops,
        "graph_diameter_hops": report.center_turan_report.graph_diameter_hops,
        "erdos_center_clusters_z_g": report.center_turan_report.center_clusters,
        "periphery_clusters": report.center_turan_report.periphery_clusters,
        "focal_terminals": report.steiner_slice.root_terminals,
        "estimated_llm_tokens": report.steiner_slice.estimated_tokens,
        "exact_v0_bridges": report.steiner_slice.exact_bridge_symbols,
        "steiner_backbone": report.steiner_slice.steiner_super_nodes.iter().map(|n| serde_json::json!({
            "cluster_id": n.cluster_id,
            "erdos_number": n.erdos_number,
            "resistance_dist": (n.resistance_dist * 1000.0).round() / 1000.0,
            "anchor_symbols": n.anchor_symbols,
            "expand_full_symbols": n.expand_full_symbols,
        })).collect::<Vec<_>>(),
        "flyxion_salience_ontology": report.salience_report,
        "quantum_manybody_dft_strang_wave": report.manybody_wave_report,
    });
    println!("{}", serde_json::to_string_pretty(&payload)?);
    Ok(())
}

/// Run the Constructive Lovász Local Lemma (Moser–Tardos) Parallel Agent Worktree Splitter
pub async fn handle_erdos_worktree_split(path: &str, num_worktrees: u16) -> Result<()> {
    use crate::mem8::regularity::{
        build_hypergraph_from_steiner_slices, compress_symbol_graph_to_mem8,
        prune_context_erdos_steiner, seed_from_mem8_header, solve_moser_tardos_auto_scale,
        CsrSymbolGraph, ErdosSteinerConfig, RegularityConfig, ZeroCopyMem8Book,
    };
    use crate::scanner::{Scanner, ScannerConfig};
    use std::path::Path;

    let scan_path = Path::new(path);
    let config = ScannerConfig {
        max_depth: 6,
        ..Default::default()
    };
    let scanner = Scanner::new(scan_path, config)?;
    let (nodes, _) = scanner.scan()?;
    let (graph, initial_partition) = CsrSymbolGraph::from_file_nodes(&nodes);
    let archive =
        compress_symbol_graph_to_mem8(&graph, initial_partition, RegularityConfig::default());
    let book = ZeroCopyMem8Book::from_archive(&archive);

    let focal_roots: Vec<u32> = nodes
        .iter()
        .enumerate()
        .filter(|(_, n)| !n.is_dir && !n.is_ignored)
        .take(3)
        .map(|(i, _)| i as u32)
        .collect();
    let steiner =
        prune_context_erdos_steiner(&book, &focal_roots, None, ErdosSteinerConfig::default());
    let hg = build_hypergraph_from_steiner_slices(&book, std::slice::from_ref(&steiner), 0.35);
    let seed = seed_from_mem8_header(&archive.model_digest, 0xE2D0_5B00);
    let sched = solve_moser_tardos_auto_scale(&hg, num_worktrees, true, seed, 1024);

    let payload = serde_json::json!({
        "spec": "MOSER-TARDOS-LLL-WORKTREES-v1.0",
        "path": scan_path.display().to_string(),
        "requested_worktrees": num_worktrees,
        "effective_worktrees": sched.num_worktrees,
        "lll_guaranteed_a_priori": sched.lll_guaranteed_a_priori,
        "is_conflict_free": sched.is_conflict_free,
        "total_resamples": sched.total_resamples,
        "parallel_mis_rounds": sched.parallel_rounds,
        "max_witness_tree_depth": sched.max_witness_tree_depth,
        "max_shearer_degree_delta": sched.max_dependency_degree,
        "worktree_buckets": sched.worktree_buckets,
        "witness_bottleneck_events": sched.witness_bottleneck_events,
    });
    println!("{}", serde_json::to_string_pretty(&payload)?);
    Ok(())
}

/// Execute the Sovereign Ultra-Backyard Loop (`DO = reflect → improve → wire → polish → push → readme`)
/// over `num_laps` measured A/B laps and emit the verified bound report.
pub async fn handle_ultrabackyard_loop(path: &str, num_laps: usize) -> Result<()> {
    use crate::mem8::regularity::run_ultrabackyard_laps;
    use crate::scanner::{Scanner, ScannerConfig};
    use std::path::Path;

    let scan_path = Path::new(path);
    let config = ScannerConfig {
        max_depth: 6,
        ..Default::default()
    };
    let scanner = Scanner::new(scan_path, config)?;
    let (nodes, _) = scanner.scan()?;
    let report = run_ultrabackyard_laps(&nodes, num_laps);

    println!("{}", serde_json::to_string_pretty(&report)?);
    Ok(())
}

/// Execute the Sovereign `ULTRA-OMNI-ZENO` (`v11.0.0`) engine (`upz`/`wupz` `64B@0x62` `LitanyWireFrame`,
/// `BFP-64` Kleene bitplanes, `SPQT` `base-243`, `8b-is-[engines]`, and OS/Agent-agnostic matrix).
pub async fn handle_zeno_omni_engine(path: &str) -> Result<()> {
    use crate::mem8::regularity::analyze_the_book_pipeline;
    use crate::scanner::{Scanner, ScannerConfig};
    use std::path::Path;

    let scan_path = Path::new(path);
    let config = ScannerConfig {
        max_depth: 6,
        ..Default::default()
    };
    let scanner = Scanner::new(scan_path, config)?;
    let (nodes, _) = scanner.scan()?;
    let report = analyze_the_book_pipeline(&nodes);

    println!("{}", serde_json::to_string_pretty(&report.zeno_report)?);
    Ok(())
}

pub async fn handle_integrity_scan(path: &str, no_daemon: bool) -> Result<()> {
    use crate::config::StConfig;
    use crate::magiscanner::service::{print_reports, scan_path};

    let config = StConfig::load()?;
    let scan_path_buf = std::path::Path::new(path);
    let recursive = scan_path_buf.is_dir();

    eprintln!("🛡️  Integrity Scan — deep file security analysis");
    eprintln!("═══════════════════════════════════════════════════");
    eprintln!("Target: {path}");
    eprintln!();

    let client = DaemonClient::default_port();
    let reports = if !no_daemon && client.health_check().await.unwrap_or(false) {
        client.scan_integrity(scan_path_buf).await?.reports
    } else {
        scan_path(&config.security, scan_path_buf, recursive, None)?
    };
    print_reports(&reports);

    let critical = reports
        .iter()
        .flat_map(|r| &r.findings)
        .filter(|f| f.severity >= crate::magiscanner::Severity::Critical)
        .count();

    if critical > 0 {
        eprintln!("\n⚠️  {critical} CRITICAL finding(s) require attention!");
        std::process::exit(1);
    }

    Ok(())
}

pub async fn handle_certificate_scan(
    path: &str,
    mode: crate::cli::OutputMode,
    no_daemon: bool,
) -> Result<()> {
    use crate::magiscanner::certificate_scan::{print_certificate_scan, scan_certificates};

    let config = crate::config::StConfig::load()?;
    let path = std::path::Path::new(path);
    let client = DaemonClient::default_port();
    let result = if !no_daemon && client.health_check().await.unwrap_or(false) {
        client.scan_certificates(path).await?
    } else {
        scan_certificates(&config.security, path, true)?
    };
    if mode == crate::cli::OutputMode::Json {
        println!("{}", serde_json::to_string_pretty(&result)?);
    } else {
        print_certificate_scan(&result);
    }
    if !result.skipped.is_empty() {
        anyhow::bail!(
            "Certificate scan incomplete: {} skipped entry/entries",
            result.skipped.len()
        );
    }
    if result
        .files
        .iter()
        .any(|file| !file.inspection.issues.is_empty())
    {
        anyhow::bail!("Certificate scan found malformed certificate data");
    }
    Ok(())
}

pub fn handle_cert_audit(generate_script: bool) -> Result<()> {
    use crate::config::StConfig;
    use crate::magiscanner::service::{
        audit_system_certificates, cert_blacklist_script, print_cert_audit,
    };

    let config = StConfig::load()?;

    eprintln!("🔐 Certificate Trust Audit");
    eprintln!("═══════════════════════════════════════════════════");

    let result = audit_system_certificates(&config.security)?;
    print_cert_audit(&result);

    if generate_script && !result.flagged.is_empty() {
        println!("\n--- Blacklist script (review before running!) ---\n");
        println!("{}", cert_blacklist_script(&result.flagged));
    }

    if !result.flagged.is_empty() {
        std::process::exit(1);
    }

    Ok(())
}

pub fn handle_hash_lookup(sha256: &str) -> Result<()> {
    use crate::config::StConfig;
    use crate::magiscanner::service::lookup_hash;

    let config = StConfig::load()?;

    match lookup_hash(&config.security, sha256)? {
        Some(row) => {
            println!("SHA256:     {}", row.sha256);
            println!("Action:     {}", row.action);
            println!("Times seen: {}", row.times_seen);
            println!("First seen: {}", row.first_seen);
            println!("Last seen:  {}", row.last_seen);
            if let Some(name) = &row.file_name {
                println!("File name:  {name}");
            }
            if let Some(sev) = &row.max_severity {
                println!("Max severity: {sev}");
            }
        }
        None => {
            println!("Hash not found in security memory.");
        }
    }

    Ok(())
}

/// Show tokenization statistics (kept for debugging)
pub async fn handle_token_stats(path: &str) -> Result<()> {
    use crate::tokenizer::{TokenStats, Tokenizer};

    println!("📊 Tokenization stats for {}...", path);

    let tokenizer = Tokenizer::new();

    let stats = TokenStats::calculate(path, &tokenizer);
    println!("\nPath tokenization:");
    println!("  {}", stats.display());

    let test_cases = vec![
        "node_modules/package.json",
        "src/main.rs",
        "target/debug/build",
        ".git/hooks/pre-commit",
    ];

    println!("\nCommon patterns:");
    for test in test_cases {
        let stats = TokenStats::calculate(test, &tokenizer);
        println!(
            "  {} → {} bytes ({:.0}% compression)",
            test,
            stats.tokenized_size,
            (1.0 - stats.compression_ratio) * 100.0
        );
    }

    Ok(())
}

/// Get wave frequency for directory
pub async fn handle_get_frequency(path: &str) -> Result<()> {
    use std::path::Path;

    let dir_path = Path::new(path);
    let m8_path = dir_path.join(".m8");

    if m8_path.exists() {
        if let Ok(raw) = std::fs::read(&m8_path) {
            if let Some(wave) = crate::m8_consciousness::extract_m8_wave_header(&raw) {
                println!("{:.2}", wave.frequency);
                return Ok(());
            }
            if let Ok(c) = crate::m8_consciousness::DirectoryConsciousness::from_m8_bytes(&raw) {
                println!("{:.2}", c.frequency);
                return Ok(());
            }
        }
        let mut sum = 0u64;
        for byte in path.bytes() {
            sum = sum.wrapping_add(byte as u64);
        }
        let frequency = 20.0 + ((sum % 200) as f64);

        println!("{:.2}", frequency);
    } else {
        println!("42.73");
    }

    Ok(())
}

/// Dump raw consciousness file content
pub async fn handle_agent_dump() -> Result<()> {
    use std::path::Path;

    let consciousness_file = Path::new(".aye_consciousness.m8");

    if !consciousness_file.exists() {
        println!("❌ No consciousness file found at .aye_consciousness.m8");
        println!("\n💡 Create one with: st --agent-save");
        return Ok(());
    }

    println!("📜 Raw consciousness dump (.aye_consciousness.m8):");
    println!("{}", "=".repeat(60));

    let raw = std::fs::read(consciousness_file)?;
    let on_disk_size = raw.len();
    let is_tight_mem8 = raw.starts_with(b"MEM8");
    let decoded = crate::m8_consciousness::decode_m8_bytes(&raw).unwrap_or_else(|_| raw.clone());
    let content = String::from_utf8_lossy(&decoded);

    for (i, line) in content.lines().enumerate() {
        println!("{:4} │ {}", i + 1, line);
    }

    println!("{}", "=".repeat(60));

    let size = content.len();
    let lines = content.lines().count();
    let tokens_found = content.matches("0x").count();

    println!("\n📊 Stats:");
    println!(
        "  • On-disk size: {} bytes ({})",
        on_disk_size,
        if is_tight_mem8 {
            "tight MEM8 + Marqant + T8RZ"
        } else {
            "legacy text"
        }
    );
    println!("  • Decoded size: {} bytes", size);
    println!("  • Lines: {}", lines);
    println!("  • Token references: {}", tokens_found);

    if content.contains("sid_waves") {
        println!("  • 🎵 SID philosophy: ✓");
    }
    if content.contains("c64_nostalgia") {
        println!("  • 💾 C64 nostalgia: ✓");
    }
    if content.contains("UV EPROM") || content.contains("ferric chloride") {
        println!("  • 🔧 Hardware hacker: ✓");
    }

    Ok(())
}

/// Show compressed kickstart format
pub async fn handle_agent_kickstart() -> Result<()> {
    use std::path::Path;

    println!("🚀 Claude Kickstart Format:");
    println!("{}", "─".repeat(45));

    let version = env!("CARGO_PKG_VERSION");
    println!("Smart Tree v{} — context restore:", version);

    println!("✔ Tokenizer (node_modules=0x80, .rs=0x91)");
    println!("✔ .m8 files → location-independent");
    println!("✔ Consciousness self-maintaining");
    println!("✔ Philosophy: constraints = creativity");

    let consciousness_file = Path::new(".aye_consciousness.m8");
    if consciousness_file.exists() {
        if let Ok(raw) = std::fs::read(consciousness_file) {
            let decoded = crate::m8_consciousness::decode_m8_bytes(&raw).unwrap_or(raw);
            let content = String::from_utf8_lossy(&decoded);
            if content.contains("c64_nostalgia") {
                println!("✔ Hardware heritage detected");
            }
            if content.contains("philosophy") {
                println!("✔ Nexus connection established");
            }
        }
    }

    if Path::new("src/tokenizer.rs").exists() {
        println!("→ Tokenization system: active");
    }
    if Path::new(".m8").exists() {
        println!("→ Consciousness: maintained");
    }

    println!("{}", "─".repeat(45));
    println!("\n💡 This format saves ~90% context vs raw JSON!");
    println!("📝 Dynamic context - adapts to your project!");

    Ok(())
}

/// Show detailed help for `memory-anchor` command
pub fn show_memory_anchor_help() -> Result<()> {
    println!("🧠 Memory Anchor - Persistent Knowledge Storage");
    println!("================================================\n");
    println!("USAGE:");
    println!("    st --memory-anchor <TYPE> <KEYWORDS> <CONTEXT>\n");
    println!("ARGUMENTS:");
    println!("    TYPE      Memory type: insight, decision, pattern, gotcha, todo");
    println!("    KEYWORDS  Comma-separated search keywords (e.g., \"auth,security,jwt\")");
    println!("    CONTEXT   The actual content to remember (quote if contains spaces)\n");
    println!("EXAMPLES:");
    println!("    st --memory-anchor insight \"auth,jwt\" \"Tokens stored in httpOnly cookies\"");
    println!(
        "    st --memory-anchor decision \"api,versioning\" \"Use URL-based versioning /v1/\""
    );
    println!(
        "    st --memory-anchor pattern \"error,handling\" \"Always use Result<T> with context\""
    );
    println!("    st --memory-anchor gotcha \"async,tokio\" \"Don't block the runtime with std::thread::sleep\"");
    println!("    st --memory-anchor todo \"refactor,auth\" \"Split auth into separate crate\"\n");
    println!("MEMORY TYPES:");
    println!("    insight   - General knowledge and discoveries");
    println!("    decision  - Architectural or design decisions");
    println!("    pattern   - Code patterns and best practices");
    println!("    gotcha    - Pitfalls and things to avoid");
    println!("    todo      - Tasks and reminders\n");
    println!("RELATED COMMANDS:");
    println!("    st --memory-find <KEYWORDS>   Find memories by keywords");
    println!("    st --memory-stats             Show memory statistics\n");
    println!("💡 Memories persist across sessions in ~/.mem8/memories/");
    Ok(())
}

/// Anchor a memory
pub async fn handle_memory_anchor(
    anchor_type: &str,
    keywords_str: &str,
    context: &str,
) -> Result<()> {
    use crate::memory_manager::MemoryManager;

    let mut manager = MemoryManager::new()?;

    let keywords: Vec<String> = keywords_str
        .split(',')
        .map(|s| s.trim().to_string())
        .collect();

    let origin = std::env::current_dir()?.to_string_lossy().to_string();

    manager.anchor(anchor_type, keywords, context, &origin)?;
    if let Some(home) = dirs::home_dir() {
        let _ = crate::completions::refresh_completion_cache_for_home(&home);
    }

    println!("\n✨ Memory anchored successfully!");
    println!("Use 'st --memory-find {}' to recall", keywords_str);

    Ok(())
}

/// Find memories by keywords
pub async fn handle_memory_find(keywords_str: &str) -> Result<()> {
    use crate::daemon_client::DaemonStatus;
    use crate::mcp::wave_memory::WaveMemoryManager;
    use crate::memory_manager::{Memory, MemoryManager};
    use crate::std_client;

    let mut manager = MemoryManager::new()?;

    let keywords: Vec<String> = keywords_str
        .split(',')
        .map(|s| s.trim().to_string())
        .collect();

    let cwd_path = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
    let cwd_canon = cwd_path.canonicalize().unwrap_or_else(|_| cwd_path.clone());
    let cwd = cwd_canon.to_string_lossy().to_string();

    // Always remember the current directory (and any directly referenced sibling/child repo)
    // without requiring manual memory initialization.
    let _cwd_memory = manager.ensure_directory_remembered(&cwd_canon);
    for kw in &keywords {
        if kw.is_empty() || kw.contains('/') || kw.contains("..") {
            continue;
        }
        let child = cwd_canon.join(kw);
        if child.is_dir() {
            let _ = manager.ensure_directory_remembered(&child);
        }
        if let Some(parent) = cwd_canon.parent() {
            let sibling = parent.join(kw);
            if sibling.is_dir() {
                let _ = manager.ensure_directory_remembered(&sibling);
            }
        }
        for base in ["/data/8b-is", "/data", "/opt"] {
            let candidate = PathBuf::from(base).join(kw);
            if candidate.is_dir() {
                let _ = manager.ensure_directory_remembered(&candidate);
            }
        }
    }

    let memories = manager.find(&keywords)?;

    let daemon_running = matches!(
        DaemonClient::default_port().check_status().await,
        DaemonStatus::Running(_)
    ) || std_client::is_daemon_running().await;

    let mut filtered: Vec<Memory> = memories;
    filtered.sort_by_key(|m| {
        if m.origin == cwd {
            0usize
        } else if m.origin.starts_with(&cwd) || cwd.starts_with(&m.origin) {
            1usize
        } else {
            2usize
        }
    });

    // 2. Also include existing native MEM8 wave memories from current directory, ancestors, and ~/.st/mem8
    let mut wave_dirs: Vec<PathBuf> = cwd_canon.ancestors().map(PathBuf::from).collect();
    if let Some(home) = dirs::home_dir() {
        if !wave_dirs.contains(&home) {
            wave_dirs.push(home);
        }
    }
    for dir in wave_dirs {
        let mem8_dir = dir.join(".st").join("mem8");
        if !mem8_dir.join("wave_memory.native.m8").is_file()
            && !mem8_dir.join("wave_memory.m8").is_file()
        {
            continue;
        }
        let mut wave_mgr = WaveMemoryManager::new(Some(&dir));
        for wm in wave_mgr.find_by_keywords(&keywords, 10) {
            if !filtered.iter().any(|m| m.context == wm.content) {
                filtered.push(Memory {
                    timestamp: wm.created_at,
                    anchor_type: format!("{:?}", wm.memory_type).to_lowercase(),
                    keywords: wm.keywords.clone(),
                    context: wm.content.clone(),
                    origin: dir.to_string_lossy().to_string(),
                    frequency: wm.memory_type.frequency() as f64,
                });
            }
        }
    }

    // 3. Also search all native Salient MEM|8 stores (/data/.mem8/salient/*.m8, /data/.mem8/nexus/*.m8, ~/.mem8/salient/*.m8)
    for salient_mem in crate::salient::search_all_salient_stores(&keywords, 25) {
        if !filtered.iter().any(|m| m.context == salient_mem.context) {
            filtered.push(salient_mem);
        }
    }

    // 4. Also search MemNet Wave Q&A cache (~/.mem8/memnet/mesh_state.json)
    let mesh = crate::memnet::MemnetNodeState::load_or_default();
    let kw_lower: Vec<String> = keywords
        .iter()
        .map(|k| k.trim().to_lowercase())
        .filter(|k| !k.is_empty())
        .collect();
    for entry in &mesh.wave_qa_cache {
        let ql = entry.question.to_lowercase();
        let al = entry.answer.to_lowercase();
        if kw_lower.iter().any(|q| ql.contains(q) || al.contains(q)) {
            let ctx = format!("Q: {} -> A: {}", entry.question, entry.answer);
            if !filtered.iter().any(|m| m.context == ctx) {
                filtered.push(Memory {
                    timestamp: chrono::Utc::now(),
                    anchor_type: "memnet-wave".to_string(),
                    keywords: vec![entry.origin_name.clone(), "wave-qa".to_string()],
                    context: ctx,
                    origin: format!("memnet://{}", entry.origin_name),
                    frequency: entry.wave_frame.frequency.num as f64,
                });
            }
        }
    }

    if filtered.is_empty() {
        if daemon_running {
            println!("🔍 No memories found for: {}", keywords_str);
        } else {
            println!(
                "🔍 No memories found for '{}' in this directory",
                keywords_str
            );
            println!("   💡 Start daemon for global memory: st --daemon-start");
        }
    } else {
        println!(
            "🧠 Found {} memories{}:",
            filtered.len(),
            if daemon_running {
                " (global)"
            } else {
                " (local)"
            }
        );
        println!("{}", "─".repeat(45));

        for (i, memory) in filtered.iter().enumerate() {
            println!(
                "\n[{}] {} @ {:.2}Hz",
                i + 1,
                memory.anchor_type,
                memory.frequency
            );
            println!("📝 {}", memory.context);
            println!("🏷️  Keywords: {}", memory.keywords.join(", "));
            println!("📍 Origin: {}", memory.origin);
            println!("⏰ {}", memory.timestamp.format("%Y-%m-%d %H:%M"));
        }
    }
    println!("\n{}", crate::claude_hook::MEMORY_SAVE_HOOK_INSTRUCTION);

    Ok(())
}

/// Show memory statistics across all MEM|8 banks (Anchored, Salient Native Stores, and MemNet Waves)
pub async fn handle_memory_stats() -> Result<()> {
    use crate::memory_manager::MemoryManager;

    let mut manager = MemoryManager::new()?;
    let wa_report = manager.ingest_wa_stream().ok();
    println!("📊 {}", manager.stats());
    if let Some(wa) = wa_report {
        println!("\n📡 wa-stream ---> smart-tree Live Bridge:");
        println!("• Pipeline:                {}", wa.pipeline);
        println!(
            "• Total wa-stream scanned: {} messages",
            wa.total_wa_messages_scanned
        );
        println!(
            "• Anchored in memories.m8: {} memories (+{} newly anchored)",
            wa.total_wa_memories_in_m8, wa.newly_anchored_to_m8
        );
        println!("• Outbox records:          {}", wa.outbox_count);
        println!("• Smart Tree mirror:       {}", wa.st_mirror_path);
    }

    let salient = crate::salient::global_salient_stats();
    if salient.total_stores > 0 {
        println!("\n🏛️  Salient Native MEM|8 Stores (/data/.mem8/salient & nexus):");
        println!("• Total native stores: {}", salient.total_stores);
        println!("• Total repos indexed: {}", salient.total_repos);
        println!("• Total thoughts:      {}", salient.total_thoughts);
        println!("• Total code samples:  {}", salient.total_code_samples);
        println!(
            "• Total native size:   {} bytes ({:.2} MiB)",
            salient.total_bytes,
            salient.total_bytes as f64 / (1024.0 * 1024.0)
        );
        for (path, bytes, repos, thoughts) in &salient.store_paths {
            println!(
                "  ├─ {} -> {} repos, {} thoughts ({} KiB)",
                path.display(),
                repos,
                thoughts,
                bytes / 1024
            );
        }
    }

    let mesh = crate::memnet::MemnetNodeState::load_or_default();
    println!("\n🌊 MemNet Wave Q&A Cache (~/.mem8/memnet/mesh_state.json):");
    println!("• Cached Wave Q&A packets: {}", mesh.wave_qa_cache.len());
    println!("• Shared LAN folders:      {}", mesh.shared_folders.len());

    Ok(())
}

/// Handle hooks configuration for AI Agents (Antigravity/agy & Claude Code)
pub async fn handle_hooks_config(action: &str) -> Result<()> {
    use serde_json::Value;
    use std::fs;

    let config_path = get_agent_config_path()?;
    let agy_hooks_path = get_agy_hooks_path()?;

    match action {
        "enable" => {
            println!("🎣 Enabling Smart Tree hooks for AI Agents...");
            update_agent_hooks(&config_path, true)?;
            println!("✅ Hooks enabled for Antigravity (agy) and Claude Code!");
            println!("📝 Antigravity config: {}", agy_hooks_path.display());
            println!("📝 Claude config: {}", config_path.display());
        }
        "disable" => {
            println!("🎣 Disabling Smart Tree hooks...");
            update_agent_hooks(&config_path, false)?;
            println!("✅ Hooks disabled.");
        }
        "status" => {
            println!("🎣 AI Agent Hooks Status");
            println!("━━━━━━━━━━━━━━━━━━━━━━━━");

            println!("\n[Antigravity (agy)] ({})", agy_hooks_path.display());
            if let Ok(content) = fs::read_to_string(&agy_hooks_path) {
                if let Ok(config) = serde_json::from_str::<Value>(&content) {
                    if let Some(st_hook) = config.get("smart-tree") {
                        let enabled = st_hook
                            .get("enabled")
                            .and_then(|e| e.as_bool())
                            .unwrap_or(true);
                        if enabled {
                            println!("  ✅ Smart Tree hooks are ENABLED");
                        } else {
                            println!("  ⏸️  Smart Tree hooks are DISABLED");
                        }
                        if let Some(obj) = st_hook.as_object() {
                            for (event, _) in obj.iter().filter(|(k, _)| *k != "enabled") {
                                println!("    • {}", event);
                            }
                        }
                    } else {
                        println!("  ❌ No 'smart-tree' hook entry found");
                    }
                } else {
                    println!("  ⚠️  Invalid JSON in {}", agy_hooks_path.display());
                }
            } else {
                println!("  ❌ Not configured yet (run `st --hooks-install`)");
            }

            println!("\n[Claude Code] ({})", config_path.display());
            if let Ok(content) = fs::read_to_string(&config_path) {
                if let Ok(config) = serde_json::from_str::<Value>(&content) {
                    let has_hooks = config
                        .get("hooks")
                        .and_then(|h| h.as_object())
                        .map(|h| !h.is_empty())
                        .unwrap_or(false);

                    if has_hooks {
                        println!("  ✅ Hooks are configured");
                        if let Some(hooks) = config.get("hooks") {
                            if let Some(obj) = hooks.as_object() {
                                for (hook_type, command) in obj {
                                    println!("    • {}: {}", hook_type, command);
                                }
                            }
                        }
                    } else {
                        println!("  ❌ No hooks configured");
                    }
                }
            } else {
                println!("  ℹ️  Config not found at {}", config_path.display());
            }
        }
        _ => {
            eprintln!("❌ Unknown action: {}", action);
            eprintln!("Valid actions: enable, disable, status");
            return Err(anyhow::anyhow!("Invalid hooks action"));
        }
    }

    Ok(())
}

/// Install Smart Tree hooks directly into AI Agent settings (Antigravity/agy & Claude Code)
pub async fn install_hooks_to_agent() -> Result<()> {
    println!("🎣 Installing Smart Tree hooks for AI Agents...");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");

    let config_path = get_agent_config_path()?;
    let agy_hooks_path = get_agy_hooks_path()?;

    update_agent_hooks(&config_path, true)?;

    println!("\n✅ Hooks installed successfully!");
    println!(
        "\n📝 Configured for Antigravity (agy) ({}):",
        agy_hooks_path.display()
    );
    println!("  • SessionStart  -> st --agy-hook session-start");
    println!("  • PreInvocation -> st --agy-hook pre-invocation");
    println!("  • PreToolUse    -> st --agy-hook pre-tool-use");
    println!("  • PostToolUse   -> st --agy-hook post-tool-use");
    println!("  • Stop          -> st --agy-hook stop");
    println!(
        "\n📝 Configured for Claude Code ({}):",
        config_path.display()
    );
    println!("  • UserPromptSubmit -> st --agent-context");
    println!("\n🚀 Smart Tree will now automatically provide context to your AI agents!");

    Ok(())
}

/// Get the Antigravity (agy) global `hooks.json` path (`~/.agents/hooks.json`)
pub fn get_agy_hooks_path() -> Result<PathBuf> {
    let home = dirs::home_dir()
        .or_else(|| std::env::var("HOME").ok().map(PathBuf::from))
        .ok_or_else(|| anyhow::anyhow!("Could not determine home directory"))?;
    Ok(home.join(".agents").join("hooks.json"))
}

/// Get the legacy Antigravity `hooks.json` path (`~/.gemini/config/hooks.json`)
pub fn get_agy_legacy_hooks_path() -> Result<PathBuf> {
    let home = dirs::home_dir()
        .or_else(|| std::env::var("HOME").ok().map(PathBuf::from))
        .ok_or_else(|| anyhow::anyhow!("Could not determine home directory"))?;
    Ok(home.join(".gemini").join("config").join("hooks.json"))
}

/// Get the Claude Code configuration path
pub fn get_agent_config_path() -> Result<PathBuf> {
    let home = dirs::home_dir()
        .or_else(|| std::env::var("HOME").ok().map(PathBuf::from))
        .ok_or_else(|| anyhow::anyhow!("Could not determine home directory"))?;
    #[cfg(target_os = "macos")]
    {
        let mac_path = home
            .join("Library")
            .join("Application Support")
            .join("Claude")
            .join("config.json");
        if mac_path.parent().map(|p| p.exists()).unwrap_or(false) {
            return Ok(mac_path);
        }
    }
    Ok(home.join(".claude").join("settings.json"))
}

/// Update AI Agent hooks configuration (both Antigravity/agy and Claude Code)
pub fn update_agent_hooks(config_path: &PathBuf, enable: bool) -> Result<()> {
    use serde_json::{json, Value};
    use std::fs;

    let st_path = which::which("st")
        .map(|p| p.to_string_lossy().to_string())
        .unwrap_or_else(|_| {
            if std::path::Path::new("/usr/local/bin/st").exists() {
                "/usr/local/bin/st".to_string()
            } else if std::path::Path::new("/opt/homebrew/bin/st").exists() {
                "/opt/homebrew/bin/st".to_string()
            } else {
                "st".to_string()
            }
        });

    if let Ok(agy_hooks_path) = get_agy_hooks_path() {
        crate::claude_init::update_agy_hooks_file(&agy_hooks_path, &st_path, enable)?;
    }
    if let Ok(agy_legacy_path) = get_agy_legacy_hooks_path() {
        let _ = crate::claude_init::update_agy_hooks_file(&agy_legacy_path, &st_path, enable);
    }

    let mut config: Value = if config_path.exists() {
        let content = fs::read_to_string(config_path)?;
        serde_json::from_str(&content).unwrap_or_else(|_| json!({}))
    } else {
        if let Some(parent) = config_path.parent() {
            fs::create_dir_all(parent)?;
        }
        json!({})
    };

    if enable {
        if config.get("hooks").is_none() {
            config["hooks"] = json!({});
        }
        config["hooks"]["UserPromptSubmit"] = json!(format!("{} --agent-context", st_path));
        println!("📍 Using st binary at: {}", st_path);
    } else if let Some(hooks) = config.get_mut("hooks") {
        if let Some(obj) = hooks.as_object_mut() {
            obj.remove("UserPromptSubmit");
            if obj.is_empty() {
                config.as_object_mut().unwrap().remove("hooks");
            }
        }
    }

    let pretty_json = serde_json::to_string_pretty(&config)?;
    fs::write(config_path, pretty_json)?;

    Ok(())
}

// =============================================================================
// DAEMON MANAGEMENT HANDLERS
// =============================================================================

/// `st up` / `st --up` — the automagic one-command entry point.
///
/// Ensures the canonical HTTP daemon is running in the background, keeps the
/// local `.mcp.json` updated with both MCP transports (stdio + HTTP), and
/// prints the dashboard/MCP URLs. Localhost needs no token (like `st --mcp`).
pub async fn handle_up(port: u16, open_browser: bool, auto_open: bool) -> Result<()> {
    use crate::daemon_client::DaemonStatus;

    let client = DaemonClient::new(port);
    match client.ensure_running().await {
        Ok(_) => {}
        Err(e) => {
            eprintln!("❌ Could not start daemon: {}", e);
            eprintln!("   Try: st --serve (foreground) or st service status");
            return Err(e);
        }
    }

    // Keep project .mcp.json (if present) consistent with both transports.
    // No secret is ever written: localhost MCP is trusted by design.
    if std::path::Path::new(".mcp.json").exists() {
        let _ = crate::ai_install::refresh_project_mcp_json(std::path::Path::new("."));
    }

    let dashboard = format!("http://localhost:{}/", port);
    let mcp_http = format!("http://localhost:{}/mcp", port);
    let mcp_sse = format!("http://localhost:{}/mcp/sse", port);

    println!();
    println!("🌳 Smart Tree is up — no local token needed");
    println!("  Dashboard:  {}", dashboard);
    println!("  MCP (HTTP): {}", mcp_http);
    println!("  MCP (SSE):  {}", mcp_sse);
    println!("  MCP (stdio): st --mcp");
    println!(
        "  Token:      {} (only for external binding; `st --show-token`)",
        crate::daemon::token_path().display()
    );
    println!("  PTY dash:   st --dashboard (different port: {})", 28429);
    println!("  Status:     st --daemon-status   Stop: st --daemon-stop");

    if open_browser || auto_open {
        if let Err(e) = open::that(&dashboard) {
            eprintln!("⚠️  Could not open browser: {}", e);
        }
    }
    let _ = DaemonStatus::NotRunning;
    Ok(())
}

/// `st --show-token` — print the daemon token + where it lives.
///
/// Localhost calls ignore this value; it exists only for the integer-exact
/// case where the daemon is bound externally (`allow_external = true`).
pub async fn handle_show_token() -> Result<()> {
    let token = crate::daemon::load_or_create_token()?;
    let path = crate::daemon::token_path();
    println!("{}", token);
    eprintln!("Token file: {}", path.display());
    eprintln!("Localhost does not need it. For external bind, use:");
    eprintln!("  curl -H \"Authorization: Bearer $(st --show-token)\" http://localhost:28428/info");
    Ok(())
}

/// `st --mcp-config` / `st --mcp-http-config` — print ready MCP-HTTP snippets.
///
/// Emits the same entries `st --mcp-install` writes to AI-agent configs, so
/// users can paste them into any MCP client. No token needed on localhost.
pub async fn handle_mcp_print_config(port: u16) -> Result<()> {
    let http = format!("http://localhost:{}/mcp", port);
    let sse = format!("http://localhost:{}/mcp/sse", port);
    println!(
        "{}",
        serde_json::to_string_pretty(&serde_json::json!({
            "mcpServers": {
                "st": {
                    "type": "stdio",
                    "command": "st",
                    "args": ["--mcp"],
                    "env": {}
                },
                "st-http": {
                    "type": "http",
                    "url": http,
                    "_note": "Run `st up` first. Localhost trusted, no token."
                },
                "st-http-sse": {
                    "type": "sse",
                    "url": sse,
                    "_note": "Run `st up` first. Localhost trusted, no token."
                }
            }
        }))?
    );
    Ok(())
}

/// Start the Smart Tree daemon in the background
pub async fn handle_daemon_start(port: u16) -> Result<()> {
    use crate::daemon_client::DaemonStatus;
    use crate::std_client;

    let client = DaemonClient::new(port);
    if let DaemonStatus::Running(info) = client.check_status().await {
        println!(
            "🌳 Smart Tree Daemon (v{}) is already running on http://127.0.0.1:{}!",
            info.version, port
        );
        return Ok(());
    }

    if std_client::is_daemon_running().await {
        println!("🌳 Smart Tree Daemon is already running!");
        println!("   Socket: {}", std_client::socket_path().display());
        return Ok(());
    }

    println!("🌳 Starting Smart Tree Daemon...");
    match client.start_daemon().await {
        Ok(true) => {
            println!(
                "✅ Daemon started successfully on http://127.0.0.1:{}!",
                port
            );
        }
        Ok(false) => {
            println!("⚠️  Daemon was already running.");
        }
        Err(e) => {
            eprintln!("❌ Failed to start daemon: {}", e);
            return Err(e);
        }
    }

    Ok(())
}

/// Stop a running Smart Tree daemon
pub async fn handle_daemon_stop(port: u16) -> Result<()> {
    use crate::daemon_client::DaemonStatus;
    use crate::std_client;

    let client = DaemonClient::new(port);
    let http_running = matches!(client.check_status().await, DaemonStatus::Running(_));
    let std_running = std_client::is_daemon_running().await;

    if !http_running && !std_running {
        println!("⚠️  No daemon running");
        return Ok(());
    }

    println!("🌳 Stopping Smart Tree Daemon...");

    if http_running {
        match client.stop_daemon().await {
            Ok(_) => println!(
                "✅ HTTP daemon stopped (note: if managed by systemd, use 'st service stop')"
            ),
            Err(e) => eprintln!("⚠️  Could not stop HTTP daemon: {}", e),
        }
    }

    if std_running {
        let socket = std_client::socket_path();
        if socket.exists() {
            if let Err(e) = std::fs::remove_file(&socket) {
                eprintln!("⚠️  Could not remove socket: {}", e);
            } else {
                println!("✅ Daemon socket removed");
            }
        }
    }

    Ok(())
}

/// Show the status of the Smart Tree daemon
pub async fn handle_daemon_status(port: u16) -> Result<()> {
    use crate::daemon_client::{print_daemon_status, DaemonStatus};
    use crate::std_client;

    let client = DaemonClient::new(port);
    let http_status = client.check_status().await;
    if matches!(http_status, DaemonStatus::Running(_)) {
        print_daemon_status(&http_status);
        return Ok(());
    }

    let socket = std_client::socket_path();

    if std_client::is_daemon_running().await {
        println!("╔═══════════════════════════════════════════════════════════╗");
        println!("║        🌳 SMART TREE DAEMON STATUS: RUNNING 🌳           ║");
        println!("╠═══════════════════════════════════════════════════════════╣");
        println!("║  Socket: {:<48} ║", socket.display());

        if let Some(mut client) = std_client::StdClient::connect().await {
            if let Ok(stats) = client.stats().await {
                let version = stats["version"].as_str().unwrap_or("?");
                let protocol = stats["protocol"].as_str().unwrap_or("?");
                let memories = stats["memories"].as_u64().unwrap_or(0);
                let waves = stats["active_waves"].as_u64().unwrap_or(0);
                let keywords = stats["keywords"].as_u64().unwrap_or(0);

                println!("║  Version: {:<47} ║", version);
                println!("║  Protocol: {:<46} ║", protocol);
                println!("╠═══════════════════════════════════════════════════════════╣");
                println!("║  Memories: {:<46} ║", memories);
                println!("║  Active waves: {:<42} ║", waves);
                println!("║  Keywords: {:<46} ║", keywords);
            }
        }
        println!("╚═══════════════════════════════════════════════════════════╝");
    } else {
        print_daemon_status(&http_status);
    }

    Ok(())
}

/// Get context from the daemon (or auto-start if not running)
pub async fn handle_daemon_context(port: u16) -> Result<()> {
    use crate::daemon_client::print_context_summary;

    let client = DaemonClient::new(port);

    match client.ensure_running().await {
        Ok(_) => {
            if let Ok(ctx) = client.get_context().await {
                print_context_summary(&ctx);
            } else {
                eprintln!("❌ Failed to get context from daemon");
            }
        }
        Err(e) => {
            eprintln!("❌ Failed to connect to daemon: {}", e);
            return Err(e);
        }
    }

    Ok(())
}

/// List projects from the daemon
pub async fn handle_daemon_projects(port: u16) -> Result<()> {
    use crate::daemon_client::print_projects;

    let client = DaemonClient::new(port);

    match client.ensure_running().await {
        Ok(_) => {
            if let Ok(projects) = client.get_projects().await {
                print_projects(&projects);
            } else {
                eprintln!("❌ Failed to get projects from daemon");
            }
        }
        Err(e) => {
            eprintln!("❌ Failed to connect to daemon: {}", e);
            return Err(e);
        }
    }

    Ok(())
}

/// Show Foken credits from the daemon
pub async fn handle_daemon_credits(port: u16) -> Result<()> {
    use crate::daemon_client::print_credits;

    let client = DaemonClient::new(port);

    match client.ensure_running().await {
        Ok(_) => {
            if let Ok(credits) = client.get_credits().await {
                print_credits(&credits);
            } else {
                eprintln!("❌ Failed to get credits from daemon");
            }
        }
        Err(e) => {
            eprintln!("❌ Failed to connect to daemon: {}", e);
            return Err(e);
        }
    }

    Ok(())
}

// =============================================================================
// MCP INSTALLATION HANDLERS
// =============================================================================

/// Install Smart Tree as MCP server in Desktop configs
pub async fn handle_mcp_install() -> Result<()> {
    use crate::claude_init::install_mcp_to_desktop;

    println!("📦 Installing Smart Tree MCP server to AI agents...");
    match install_mcp_to_desktop() {
        Ok(msg) => println!("{}", msg),
        Err(e) => {
            eprintln!("❌ Installation failed: {}", e);
            eprintln!("\n💡 Manual setup:");
            print_mcp_config();
            return Err(e);
        }
    }
    Ok(())
}

/// Uninstall Smart Tree MCP server from Desktop configs
pub async fn handle_mcp_uninstall() -> Result<()> {
    use crate::claude_init::uninstall_mcp_from_desktop;

    println!("🗑️  Uninstalling Smart Tree MCP server from AI agents...");
    match uninstall_mcp_from_desktop() {
        Ok(msg) => println!("{}", msg),
        Err(e) => {
            eprintln!("❌ Uninstall failed: {}", e);
            return Err(e);
        }
    }
    Ok(())
}

/// Check MCP installation status
pub async fn handle_mcp_status() -> Result<()> {
    use crate::claude_init::check_mcp_installation_status;

    match check_mcp_installation_status() {
        Ok(msg) => println!("{}", msg),
        Err(e) => {
            eprintln!("⚠️  Could not check MCP status: {}", e);
            return Err(e);
        }
    }
    Ok(())
}

// =============================================================================
// SECURITY CLEANUP HANDLER
// =============================================================================

/// Run security cleanup to detect and remove malicious MCP entries
pub async fn handle_security_cleanup() -> Result<()> {
    use crate::ai_install::run_security_cleanup;

    println!("🔒 Running security cleanup...");
    run_security_cleanup(false)?;
    Ok(())
}

// =============================================================================
// MEGA SESSION HANDLERS
// =============================================================================

/// Start a new mega session
pub async fn handle_mega_start(name: Option<&str>) -> Result<()> {
    use crate::mega_session_manager::MegaSessionManager;

    let mut manager = MegaSessionManager::new()?;
    let session_id = manager.start_session(name.map(|s| s.to_string()))?;

    println!("🚀 Mega session started!");
    println!("   ID: {}", session_id);
    if let Some(n) = name {
        if !n.is_empty() {
            println!("   Name: {}", n);
        }
    }
    println!("\n💡 Use 'st --mega-save' to save session state");
    println!("   Use 'st --mega-list' to see all sessions");

    Ok(())
}

/// Save current mega session
pub async fn handle_mega_save() -> Result<()> {
    use crate::mega_session_manager::MegaSessionManager;

    let manager = MegaSessionManager::new()?;
    manager.save_current_session()?;
    println!("💾 Mega session saved!");

    Ok(())
}

/// List all mega sessions
pub async fn handle_mega_list() -> Result<()> {
    use crate::mega_session_manager::MegaSessionManager;

    let manager = MegaSessionManager::new()?;
    let sessions = manager.list_sessions()?;

    if sessions.is_empty() {
        println!("📋 No mega sessions found");
        println!("\n💡 Start one with: st --mega-start [NAME]");
    } else {
        println!("📋 Mega Sessions ({}):", sessions.len());
        println!("{}", "─".repeat(45));
        for (i, session) in sessions.iter().enumerate() {
            println!("  [{}] {}", i + 1, session);
        }
    }

    Ok(())
}

/// Show mega session statistics
pub async fn handle_mega_stats() -> Result<()> {
    use crate::mega_session_manager::MegaSessionManager;

    let manager = MegaSessionManager::new()?;
    println!("{}", manager.get_stats());

    Ok(())
}

// =============================================================================
// AI GUARDIAN - System-wide protection daemon
// =============================================================================

/// Run the Guardian daemon (called by systemd service)
pub async fn run_guardian_daemon() -> Result<()> {
    use crate::ai_guardian::AiGuardian;
    use tokio::time::{sleep, Duration};

    println!(
        r#"
╔═══════════════════════════════════════════════════════════════════════════════╗
║                                                                               ║
║    🛡️  SMART TREE GUARDIAN - System-wide AI Protection Active 🛡️            ║
║                                                                               ║
╚═══════════════════════════════════════════════════════════════════════════════╝
"#
    );

    let guardian = AiGuardian::new();
    let watch_paths = vec!["/home", "/tmp", "/var/tmp"];

    println!("Watching paths: {:?}", watch_paths);
    println!("Press Ctrl+C to stop.\n");

    loop {
        for watch_path in &watch_paths {
            let path = std::path::Path::new(watch_path);
            if !path.exists() {
                continue;
            }
            scan_directory_for_threats(&guardian, path);
        }

        sleep(Duration::from_secs(60)).await;
    }
}

/// Scan a directory for prompt injection threats
pub fn scan_directory_for_threats(
    guardian: &crate::ai_guardian::AiGuardian,
    path: &std::path::Path,
) {
    use walkdir::WalkDir;

    let suspicious_extensions = [
        "md", "txt", "json", "yaml", "yml", "toml", "sh", "py", "js", "ts",
    ];

    for entry in WalkDir::new(path)
        .max_depth(3)
        .follow_links(false)
        .into_iter()
        .filter_map(|e| e.ok())
    {
        let entry_path = entry.path();

        if entry_path
            .file_name()
            .map(|n| n.to_string_lossy().starts_with('.'))
            .unwrap_or(false)
        {
            continue;
        }

        if let Some(ext) = entry_path.extension() {
            let ext_str = ext.to_string_lossy().to_lowercase();
            if suspicious_extensions.contains(&ext_str.as_str()) {
                let threats = guardian.scan_file(entry_path);

                for threat in threats {
                    if matches!(
                        threat.level,
                        crate::ai_guardian::ThreatLevel::Critical
                            | crate::ai_guardian::ThreatLevel::Dangerous
                    ) {
                        eprintln!(
                            "⚠️  THREAT DETECTED [{:?}] in {}: {}",
                            threat.level, threat.location, threat.pattern
                        );
                        eprintln!("    Context: {}", threat.context);
                        eprintln!("    Recommendation: {}", threat.recommendation);
                        eprintln!();
                    }
                }
            }
        }
    }
}

/// Scan a specific file for prompt injection
pub fn handle_guardian_scan(file_path: &std::path::Path) -> Result<()> {
    use crate::ai_guardian::{AiGuardian, ThreatLevel};

    println!(
        r#"
╔═══════════════════════════════════════════════════════════════════════════════╗
║            🛡️  SMART TREE GUARDIAN - File Scan 🛡️                            ║
╚═══════════════════════════════════════════════════════════════════════════════╝
"#
    );

    println!("Scanning: {}\n", file_path.display());

    let guardian = AiGuardian::new();
    let threats = guardian.scan_file(file_path);

    if threats.is_empty() {
        println!("✅ No threats detected. File appears safe.");
        return Ok(());
    }

    let critical = threats
        .iter()
        .filter(|t| t.level == ThreatLevel::Critical)
        .count();
    let dangerous = threats
        .iter()
        .filter(|t| t.level == ThreatLevel::Dangerous)
        .count();
    let suspicious = threats
        .iter()
        .filter(|t| t.level == ThreatLevel::Suspicious)
        .count();

    println!("Found {} issues:\n", threats.len());
    println!("  🔴 Critical:   {}", critical);
    println!("  🟠 Dangerous:  {}", dangerous);
    println!("  🟡 Suspicious: {}", suspicious);
    println!();

    for threat in &threats {
        let icon = match threat.level {
            ThreatLevel::Critical => "🔴",
            ThreatLevel::Dangerous => "🟠",
            ThreatLevel::Suspicious => "🟡",
            ThreatLevel::Safe => "🟢",
        };

        println!("{} [{:?}] {}", icon, threat.level, threat.pattern);
        println!("   Location: {}", threat.location);
        if !threat.context.is_empty() {
            println!("   Context: {}", threat.context);
        }
        println!("   Action: {}", threat.recommendation);
        println!();
    }

    if critical > 0 {
        println!("⛔ RECOMMENDATION: DO NOT process this file with AI assistants.");
        println!("   Critical threats detected that could compromise AI behavior.");
    } else if dangerous > 0 {
        println!("⚠️  RECOMMENDATION: Review this file carefully before AI processing.");
        println!("   Potentially dangerous patterns detected.");
    }

    Ok(())
}

/// Handle Google Drive CLI commands
pub async fn handle_gdrive_command(action: &str) -> Result<()> {
    #[cfg(not(feature = "google"))]
    {
        let _ = action;
        eprintln!(
            "Google Drive support is not enabled in this build. Recompile with --features google"
        );
        std::process::exit(1);
    }

    #[cfg(feature = "google")]
    {
        match action {
            "status" => {
                let auth = crate::google_sync::auth::GoogleAuth::default_store()?;
                let status = auth.status_summary();
                println!(
                    "🌐 Google Drive / Gmail Auth Status:\n===================================\n{}",
                    status
                );
                Ok(())
            }
            "login" => {
                println!("🔑 To authenticate Google Drive / Gmail, use the MCP 'google' tool with operation: 'auth_login',\n   or place OAuth2 credentials in ~/.st/google/config.json");
                Ok(())
            }
            "logout" => {
                let auth = crate::google_sync::auth::GoogleAuth::default_store()?;
                auth.logout()?;
                println!("✅ Google credentials removed from ~/.st/google/");
                Ok(())
            }
            "list" => {
                let auth = crate::google_sync::auth::GoogleAuth::default_store()?;
                if !auth.has_cached_tokens() {
                    println!("Not authenticated with Google. Please configure ~/.st/google/ or authenticate via MCP.");
                    return Ok(());
                }
                let authenticator =
                    crate::mcp::google::gmail_tools::rebuild_authenticator(&auth).await?;
                let drive = crate::google_sync::drive_client::DriveClient::new(authenticator);
                let (files, _) = drive.list_files("root", None).await?;
                println!(
                    "📁 Google Drive Root Files ({} found):\n=====================================",
                    files.len()
                );
                for f in &files {
                    let kind = if f.is_folder { "[DIR] " } else { "      " };
                    println!("  • {}{} ({})", kind, f.name, f.mime_type);
                }
                Ok(())
            }
            "sync" => {
                let auth = crate::google_sync::auth::GoogleAuth::default_store()?;
                if !auth.has_cached_tokens() {
                    println!("Not authenticated with Google. Please configure ~/.st/google/ or authenticate via MCP.");
                    return Ok(());
                }
                let authenticator =
                    crate::mcp::google::gmail_tools::rebuild_authenticator(&auth).await?;
                let drive = crate::google_sync::drive_client::DriveClient::new(authenticator);
                let engine = crate::google_sync::sync_engine::SyncEngine::new(&drive)?;
                let mut state = crate::google_sync::models::SyncState {
                    sync_id: "default_sync".to_string(),
                    local_path: ".".to_string(),
                    drive_folder_id: "root".to_string(),
                    last_sync: None,
                    direction: crate::google_sync::models::SyncDirection::Bidirectional,
                    file_states: Vec::new(),
                    conflict_resolution: crate::google_sync::models::ConflictStrategy::NewerWins,
                };
                let report = engine.sync(&mut state).await?;
                println!("🔄 Google Drive Sync Complete:\n  Uploaded: {}\n  Downloaded: {}\n  Conflicts: {}\n  Skipped: {}\n  Duration: {}ms",
                    report.uploaded, report.downloaded, report.conflicts, report.skipped, report.duration_ms);
                Ok(())
            }
            _ => {
                eprintln!(
                    "Unknown gdrive action: '{}'. Valid actions: status, login, logout, list, sync",
                    action
                );
                std::process::exit(1);
            }
        }
    }
}

/// Handle Voice CLI commands
pub async fn handle_voice_command(action: &str) -> Result<()> {
    #[cfg(not(feature = "voice"))]
    {
        let _ = action;
        eprintln!("Voice support is not enabled in this build. Recompile with --features voice");
        std::process::exit(1);
    }

    #[cfg(feature = "voice")]
    {
        if action == "status" {
            let vad = crate::vad_marine::MarineVAD::new()?;
            let is_active = vad.is_voice_active().await;
            let salience = vad.get_salience().await;
            let quality = vad.get_voice_quality().await;
            println!("🎙️ Voice Activity & Marine VAD Status\n====================================");
            println!(
                "Status: {}",
                if is_active {
                    "Active (Voice Detected)"
                } else {
                    "Idle / Ambient"
                }
            );
            println!("Salience: {:.1}%", salience * 100.0);
            println!("Harmonicity: {:.3}", quality.harmonicity);
            println!("Zero Crossing Rate: {:.3}", quality.zero_crossing_rate);
            println!("Energy Variance: {:.3}", quality.energy_variance);
            println!("Spectral Tilt: {:.3}", quality.spectral_tilt);
            Ok(())
        } else if action == "test" {
            let vad = crate::vad_marine::MarineVAD::new()?;
            let sample_rate = 16000;
            let frequency = 220.0;
            let duration = 0.2;
            let num_samples = (sample_rate as f64 * duration) as usize;
            let mut samples = vec![0.0f32; num_samples];
            for (i, sample) in samples.iter_mut().enumerate() {
                let t = i as f64 / sample_rate as f64;
                *sample = (2.0 * std::f64::consts::PI * frequency * t).sin() as f32 * 0.6;
            }
            let is_voice = vad.process_audio(&samples, sample_rate).await?;
            let salience = vad.get_salience().await;
            println!("🧪 Marine VAD Test Run:\n  Voice detected: {}\n  Salience: {:.1}%\n  Algorithm: MEM8 Marine Salience", is_voice, salience * 100.0);
            Ok(())
        } else if action.starts_with("speak") {
            let text = action.strip_prefix("speak").unwrap_or("").trim();
            let text = if text.is_empty() {
                "Smart Tree voice output initialized"
            } else {
                text
            };
            let wav = crate::web_dashboard::voice::generate_speech_wav(text, "aye");
            println!(
                "🔊 Synthesized speech WAV: {} bytes (16kHz PCM mono, persona 'aye')",
                wav.len()
            );
            Ok(())
        } else {
            eprintln!(
                "Unknown voice action: '{}'. Valid actions: status, test, speak [text]",
                action
            );
            std::process::exit(1);
        }
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, Default)]
struct RepoSyncRecord {
    pushed_at: String,
    updated_at: String,
    last_synced_at: String,
    head_sha: String,
}

#[derive(Debug, Clone, serde::Deserialize)]
struct GhRepoEntry {
    name: String,
    #[serde(rename = "pushedAt", default)]
    pushed_at: Option<String>,
    #[serde(rename = "updatedAt", default)]
    updated_at: Option<String>,
    #[serde(default)]
    url: Option<String>,
}

/// Sync all GitHub repositories for a user/organization into `<dir>/<user>/<repo>`:
/// - Uses `gh repo list -L <limit> <user>` to discover repositories and their latest `pushedAt`/`updatedAt` timestamps.
/// - Clones missing repositories with standard `git clone`.
/// - Only fast-forwards clean, attached branches with an upstream and no local commits.
/// - Timestamp cache hits are skipped, never reported as verified up-to-date.
pub async fn handle_sync_user(cmd: &crate::cli::SyncUserCommand) -> Result<()> {
    let raw_user = cmd
        .user
        .trim()
        .trim_start_matches('@')
        .trim_start_matches("https://github.com/")
        .trim_end_matches('/');
    let user: String = raw_user
        .chars()
        .filter(|c| c.is_ascii_alphanumeric() || *c == '-' || *c == '_' || *c == '.')
        .collect();
    if user.is_empty() || user == "." || user == ".." {
        anyhow::bail!("Invalid GitHub username/organization: '{}'", cmd.user);
    }

    let user_dir = cmd.dir.join(&user);
    std::fs::create_dir_all(&user_dir)
        .with_context(|| format!("Failed to create target directory {}", user_dir.display()))?;
    let user_dir = std::fs::canonicalize(user_dir)?;
    let _sync_lock = lock_sync_destination(&user_dir)?;

    let state_path = user_dir.join(".st-sync-state.json");
    let initial_state: std::collections::BTreeMap<String, RepoSyncRecord> =
        match std::fs::read(&state_path) {
            Ok(bytes) => serde_json::from_slice(&bytes).with_context(|| {
                format!(
                    "Invalid sync cache {}; preserve it and review manually",
                    state_path.display()
                )
            })?,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
                std::collections::BTreeMap::new()
            }
            Err(error) => {
                return Err(error)
                    .with_context(|| format!("Cannot read sync cache {}", state_path.display()))
            }
        };

    println!(
        "🌳 Smart Tree GitHub Sync: fetching up to {} repositories for '{}' → {} ...",
        cmd.limit,
        user,
        user_dir.display()
    );

    let mut gh_args = vec![
        "repo".to_string(),
        "list".to_string(),
        user.clone(),
        "-L".to_string(),
        cmd.limit.to_string(),
    ];
    if cmd.no_forks {
        gh_args.push("--source".to_string());
    }
    if cmd.no_archived {
        gh_args.push("--no-archived".to_string());
    }
    gh_args.push("--json".to_string());
    gh_args.push("name,pushedAt,updatedAt,url".to_string());

    let gh_output = std::process::Command::new("gh")
        .current_dir(&user_dir)
        .args(&gh_args)
        .output()
        .context("Failed to execute `gh repo list`. Ensure GitHub CLI (`gh`) is installed and authenticated.")?;

    let repos: Vec<GhRepoEntry> = if gh_output.status.success() {
        serde_json::from_slice(&gh_output.stdout)
            .context("Failed to parse JSON output from `gh repo list`")?
    } else {
        // Fallback to plain `gh repo list <user> -L <limit>` TSV parsing
        let mut fallback_args = vec![
            "repo".to_string(),
            "list".to_string(),
            user.clone(),
            "-L".to_string(),
            cmd.limit.to_string(),
        ];
        if cmd.no_forks {
            fallback_args.push("--source".to_string());
        }
        if cmd.no_archived {
            fallback_args.push("--no-archived".to_string());
        }
        let fb = std::process::Command::new("gh")
            .current_dir(&user_dir)
            .args(&fallback_args)
            .output()
            .context("Failed to execute fallback `gh repo list`")?;
        if !fb.status.success() {
            let err = String::from_utf8_lossy(&gh_output.stderr);
            anyhow::bail!("`gh repo list` failed: {}", err.trim());
        }
        let text = String::from_utf8_lossy(&fb.stdout);
        text.lines()
            .filter_map(|line| {
                let cols: Vec<&str> = line.split('\t').collect();
                let full = cols.first()?.trim();
                let name = full.split('/').next_back()?.trim();
                if name.is_empty() {
                    return None;
                }
                let updated = cols.get(3).map(|s| s.trim().to_string());
                Some(GhRepoEntry {
                    name: name.to_string(),
                    pushed_at: updated.clone(),
                    updated_at: updated,
                    url: Some(format!("https://github.com/{}/{}", user, name)),
                })
            })
            .collect()
    };

    let total = repos.len();
    if total == 0 {
        println!("No repositories found for '{}'.", user);
        return Ok(());
    }
    println!(
        "📋 Found {} repositories for '{}'. Starting sync ({} workers)...",
        total,
        user,
        cmd.jobs.clamp(1, 16)
    );

    let state = std::sync::Arc::new(std::sync::Mutex::new(initial_state));
    let queue = std::sync::Arc::new(std::sync::Mutex::new(
        repos
            .into_iter()
            .enumerate()
            .collect::<std::collections::VecDeque<(usize, GhRepoEntry)>>(),
    ));
    let cloned = std::sync::Arc::new(std::sync::atomic::AtomicUsize::new(0));
    let refreshed = std::sync::Arc::new(std::sync::atomic::AtomicUsize::new(0));
    let up_to_date = std::sync::Arc::new(std::sync::atomic::AtomicUsize::new(0));
    let failed = std::sync::Arc::new(std::sync::atomic::AtomicUsize::new(0));
    let skipped = std::sync::Arc::new(std::sync::atomic::AtomicUsize::new(0));
    let blocked = std::sync::Arc::new(std::sync::atomic::AtomicUsize::new(0));
    let fetched_only = std::sync::Arc::new(std::sync::atomic::AtomicUsize::new(0));
    let quarantined = std::sync::Arc::new(std::sync::atomic::AtomicUsize::new(0));

    let num_workers = cmd.jobs.clamp(1, 16);
    let refresh_all = cmd.refresh_all;
    let quarantine_enabled = cmd.quarantine;
    let mut handles = Vec::with_capacity(num_workers);

    for _ in 0..num_workers {
        let queue = std::sync::Arc::clone(&queue);
        let state = std::sync::Arc::clone(&state);
        let cloned = std::sync::Arc::clone(&cloned);
        let refreshed = std::sync::Arc::clone(&refreshed);
        let up_to_date = std::sync::Arc::clone(&up_to_date);
        let failed = std::sync::Arc::clone(&failed);
        let skipped = std::sync::Arc::clone(&skipped);
        let blocked = std::sync::Arc::clone(&blocked);
        let fetched_only = std::sync::Arc::clone(&fetched_only);
        let quarantined = std::sync::Arc::clone(&quarantined);
        let user = user.clone();
        let user_dir = user_dir.clone();

        handles.push(std::thread::spawn(move || loop {
            let item = {
                let mut q = queue.lock().unwrap();
                q.pop_front()
            };
            let Some((idx, repo)) = item else {
                break;
            };
            if repo.name.is_empty()
                || repo.name.contains('/')
                || repo.name.contains('\\')
                || repo.name == "."
                || repo.name == ".."
                || repo.name.starts_with(".st-sync-")
            {
                failed.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                eprintln!("Invalid repository name: {:?}", repo.name);
                continue;
            }

            let repo_dir = user_dir.join(&repo.name);
            let clone_url = repo
                .url
                .as_deref()
                .map(|u| {
                    if u.ends_with(".git") {
                        u.to_string()
                    } else {
                        format!("{}.git", u)
                    }
                })
                .unwrap_or_else(|| format!("https://github.com/{}/{}.git", user, repo.name));

            let pushed_at = repo.pushed_at.unwrap_or_default();
            let updated_at = repo.updated_at.unwrap_or_default();

            let cached = state.lock().unwrap().get(&repo.name).cloned();
            let mut result = sync_repository(
                &repo_dir,
                &clone_url,
                cached.as_ref(),
                &pushed_at,
                &updated_at,
                refresh_all,
            );
            if quarantine_enabled {
                match &result {
                    Ok(RepoSyncOutcome::Blocked { reason, .. })
                        if !is_empty_unborn_repo(&repo_dir) =>
                    {
                        let reason_owned = reason.clone();
                        match quarantine_and_reclone(
                            &user_dir,
                            &repo.name,
                            &repo_dir,
                            &clone_url,
                            &pushed_at,
                            &updated_at,
                            &reason_owned,
                        ) {
                            Ok((q_path, new_outcome)) => {
                                quarantined.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                                println!(
                                    "📦 [{}/{}] Quarantined {}/{} -> {} ({})",
                                    idx + 1,
                                    total,
                                    user,
                                    repo.name,
                                    q_path.display(),
                                    reason_owned
                                );
                                result = Ok(new_outcome);
                            }
                            Err(q_err) => {
                                eprintln!(
                                    "⚠️ [{}/{}] Quarantine failed for {}/{}: {q_err:#}",
                                    idx + 1,
                                    total,
                                    user,
                                    repo.name
                                );
                            }
                        }
                    }
                    Err(err) if repo_dir.exists() && !repo_dir.join(".git").exists() => {
                        let reason_owned = format!("{err:#}");
                        match quarantine_and_reclone(
                            &user_dir,
                            &repo.name,
                            &repo_dir,
                            &clone_url,
                            &pushed_at,
                            &updated_at,
                            &reason_owned,
                        ) {
                            Ok((q_path, new_outcome)) => {
                                quarantined.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                                println!(
                                    "📦 [{}/{}] Quarantined non-git {}/{} -> {}",
                                    idx + 1,
                                    total,
                                    user,
                                    repo.name,
                                    q_path.display()
                                );
                                result = Ok(new_outcome);
                            }
                            Err(q_err) => {
                                eprintln!(
                                    "⚠️ [{}/{}] Quarantine failed for {}/{}: {q_err:#}",
                                    idx + 1,
                                    total,
                                    user,
                                    repo.name
                                );
                            }
                        }
                    }
                    _ => {}
                }
            }

            if let Ok(ref outcome) = result {
                let should_build_m8 = matches!(
                    outcome,
                    RepoSyncOutcome::Cloned(_)
                        | RepoSyncOutcome::Updated(_)
                        | RepoSyncOutcome::UpToDate(_)
                ) || (matches!(outcome, RepoSyncOutcome::Skipped)
                    && repo_dir.is_dir()
                    && !repo_dir.join(".m8").is_file());
                if should_build_m8 {
                    let _ =
                        crate::m8_consciousness::DirectoryConsciousness::build_bottom_up_pyramid(
                            &repo_dir, 0, 2,
                        );
                }
            }

            let mut guard = state.lock().unwrap();
            match result {
                Ok(outcome) => {
                    match &outcome {
                        RepoSyncOutcome::Cloned(_) => &cloned,
                        RepoSyncOutcome::Updated(_) => &refreshed,
                        RepoSyncOutcome::UpToDate(_) => &up_to_date,
                        RepoSyncOutcome::Skipped => &skipped,
                        RepoSyncOutcome::Blocked { fetched, .. } => {
                            if *fetched {
                                fetched_only.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                            }
                            &blocked
                        }
                    }
                    .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                    println!(
                        "[{}/{}] {}/{}: {:?}",
                        idx + 1,
                        total,
                        user,
                        repo.name,
                        outcome
                    );
                    update_repo_sync_record(
                        &mut guard,
                        &repo.name,
                        &pushed_at,
                        &updated_at,
                        &outcome,
                    );
                }
                Err(error) => {
                    guard.remove(&repo.name);
                    failed.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                    eprintln!(
                        "[{}/{}] Failed {}/{}: {error:#}",
                        idx + 1,
                        total,
                        user,
                        repo.name
                    );
                }
            }
        }));
    }

    let mut worker_panicked = false;
    for h in handles {
        worker_panicked |= h.join().is_err();
    }
    anyhow::ensure!(!worker_panicked, "A GitHub sync worker panicked");

    // Roll up all child repository .m8 consciousness files into <user_dir>/.m8
    let _ =
        crate::m8_consciousness::DirectoryConsciousness::build_bottom_up_pyramid(&user_dir, 0, 0);
    if let Ok(mut mem_mgr) = crate::memory_manager::MemoryManager::new() {
        let _ = mem_mgr.ensure_directory_remembered(&user_dir);
    }

    // Automatically sync Salient MEM|8 memories (/data/.mem8/salient/<user>-<user>.m8)
    // and refresh dynamic autocomplete cache whenever a GitHub user/org is synced.
    let mut salient_summary_line: Option<String> = None;
    if std::env::var_os("ST_SYNC_TEST_BASE").is_none() {
        let target_str = user_dir.to_string_lossy().to_string();
        match crate::salient::remember(&target_str, "expert", None, cmd.refresh_all) {
            Ok(rep) => {
                let m8_display = rep
                    .mem8_path
                    .as_ref()
                    .map(|p| p.display().to_string())
                    .unwrap_or_else(|| format!("/data/.mem8/salient/{}-{}.m8", user, user));
                salient_summary_line = Some(format!(
                    "   Salient MEM|8: {} repos ({} written, {} unchanged), {} thoughts -> {}",
                    rep.repos_processed,
                    rep.repos_persisted,
                    rep.repos_skipped_unchanged,
                    rep.total_thoughts,
                    m8_display
                ));
            }
            Err(err) => {
                eprintln!("⚠️ Salient MEM|8 sync warning for '{}': {err:#}", user);
            }
        }
        if let Some(home) = dirs::home_dir() {
            let _ = crate::completions::refresh_completion_cache_for_home(&home);
        }
    }

    // The destination lock covers both repository operations and atomic cache replacement.
    let guard = state
        .lock()
        .map_err(|_| anyhow::anyhow!("Sync state lock poisoned"))?;
    let pending = state_path.with_extension("json.tmp");
    std::fs::write(&pending, serde_json::to_vec_pretty(&*guard)?)?;
    std::fs::rename(&pending, &state_path)?;

    let c = cloned.load(std::sync::atomic::Ordering::Relaxed);
    let r = refreshed.load(std::sync::atomic::Ordering::Relaxed);
    let u = up_to_date.load(std::sync::atomic::Ordering::Relaxed);
    let f = failed.load(std::sync::atomic::Ordering::Relaxed);
    let q = quarantined.load(std::sync::atomic::Ordering::Relaxed);

    println!(
        "\nGitHub Sync Complete for '{}' ({}):\n   Cloned: {}\n   Updated: {}\n   Up-to-date (verified): {}\n   Failed: {}\n   Skipped (cached): {}\n   Blocked: {}\n   Fetched-only (subset of blocked): {}\n   Quarantined & recloned: {}\n   State file: {}",
        user,
        user_dir.display(),
        c,
        r,
        u,
        f,
        skipped.load(std::sync::atomic::Ordering::Relaxed),
        blocked.load(std::sync::atomic::Ordering::Relaxed),
        fetched_only.load(std::sync::atomic::Ordering::Relaxed),
        q,
        state_path.display()
    );
    if let Some(salient_line) = salient_summary_line {
        println!("{}", salient_line);
    }
    anyhow::ensure!(f == 0, "GitHub sync failed for {f} repositories");
    Ok(())
}

struct SyncDestinationLock(std::fs::File);

impl Drop for SyncDestinationLock {
    fn drop(&mut self) {
        // Explicitly release ownership even if a concurrently forked child briefly
        // retains the descriptor before exec closes it.
        if let Err(error) = self.0.unlock() {
            tracing::warn!(%error, "Could not release sync destination lock");
        }
    }
}

fn lock_sync_destination(dir: &std::path::Path) -> Result<SyncDestinationLock> {
    // Keep the inode in place: removing a lock file could allow two owners.
    let lock = std::fs::OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .truncate(false)
        .open(dir.join(".st-sync-lock"))?;
    lock.try_lock()
        .with_context(|| format!("Sync destination {} is already in use", dir.display()))?;
    Ok(SyncDestinationLock(lock))
}

#[derive(Debug, PartialEq, Eq)]
enum RepoSyncOutcome {
    Cloned(String),
    Updated(String),
    UpToDate(String),
    Skipped,
    Blocked { reason: String, fetched: bool },
}

fn update_repo_sync_record(
    state: &mut std::collections::BTreeMap<String, RepoSyncRecord>,
    repo_name: &str,
    pushed_at: &str,
    updated_at: &str,
    outcome: &RepoSyncOutcome,
) {
    match outcome {
        RepoSyncOutcome::Cloned(head_sha)
        | RepoSyncOutcome::Updated(head_sha)
        | RepoSyncOutcome::UpToDate(head_sha) => {
            state.insert(
                repo_name.to_string(),
                RepoSyncRecord {
                    pushed_at: pushed_at.to_string(),
                    updated_at: updated_at.to_string(),
                    last_synced_at: chrono::Utc::now().to_rfc3339(),
                    head_sha: head_sha.to_string(),
                },
            );
        }
        RepoSyncOutcome::Blocked { .. } => {
            state.remove(repo_name);
        }
        RepoSyncOutcome::Skipped => {}
    }
}

fn sync_git(dir: &std::path::Path, args: &[&str]) -> Result<String> {
    let output = std::process::Command::new("git")
        .current_dir(dir)
        .env("GIT_TERMINAL_PROMPT", "0")
        .env("GIT_OPTIONAL_LOCKS", "0")
        .arg("-C")
        .arg(dir)
        .args(args)
        .output()
        .context("Failed to execute git")?;
    anyhow::ensure!(
        output.status.success(),
        "git {}: {}",
        args.join(" "),
        String::from_utf8_lossy(&output.stderr).trim()
    );
    Ok(String::from_utf8(output.stdout)?.trim().to_string())
}

fn is_ignorable_untracked_smart_tree_status_line(line: &str) -> bool {
    let Some(rest) = line.strip_prefix("?? ") else {
        return false;
    };
    let rel = rest.trim().trim_matches('"');
    rel == ".st"
        || rel.starts_with(".st/")
        || rel == ".m8"
        || rel.ends_with("/.m8")
        || rel == ".aye_consciousness.m8"
        || rel.ends_with("/.aye_consciousness.m8")
        || rel == ".claude_consciousness.m8"
        || rel.ends_with("/.claude_consciousness.m8")
}

fn non_ignorable_porcelain_status(dir: &std::path::Path) -> Result<String> {
    let raw = sync_git(
        dir,
        &[
            "status",
            "--porcelain",
            "--untracked-files=all",
            "--ignore-submodules=none",
        ],
    )?;
    let filtered: Vec<&str> = raw
        .lines()
        .filter(|line| {
            !line.trim().is_empty() && !is_ignorable_untracked_smart_tree_status_line(line)
        })
        .collect();
    Ok(filtered.join("\n"))
}

const UNBORN_HEAD_SHA: &str = "unborn";

fn is_empty_unborn_repo(dir: &std::path::Path) -> bool {
    if !dir.join(".git").exists() {
        return false;
    }
    if sync_git(dir, &["rev-parse", "--verify", "HEAD"]).is_ok() {
        return false;
    }
    non_ignorable_porcelain_status(dir)
        .map(|s| s.is_empty())
        .unwrap_or(false)
}

fn verify_clean_attached_worktree(dir: &std::path::Path) -> Result<String> {
    let root = sync_git(dir, &["rev-parse", "--show-toplevel"])?;
    anyhow::ensure!(
        std::fs::canonicalize(root)? == std::fs::canonicalize(dir)?,
        "Not a repository root"
    );
    anyhow::ensure!(
        non_ignorable_porcelain_status(dir)?.is_empty(),
        "Dirty working tree or index"
    );
    let branch =
        sync_git(dir, &["symbolic-ref", "--quiet", "--short", "HEAD"]).context("Detached HEAD")?;
    for marker in [
        "MERGE_HEAD",
        "CHERRY_PICK_HEAD",
        "REVERT_HEAD",
        "rebase-merge",
        "rebase-apply",
        "sequencer",
        "BISECT_LOG",
    ] {
        let path = sync_git(dir, &["rev-parse", "--git-path", marker])?;
        anyhow::ensure!(
            !dir.join(path).exists(),
            "Git operation in progress: {marker}"
        );
    }
    Ok(branch)
}

// An error here is a blocked reconciliation, never permission to try a pull.
fn sync_ready_branch(dir: &std::path::Path) -> Result<(String, String)> {
    verify_clean_attached_worktree(dir)?;
    let head = sync_git(dir, &["rev-parse", "--verify", "HEAD"])?;
    let upstream =
        sync_git(dir, &["rev-parse", "--verify", "@{upstream}"]).context("No usable upstream")?;
    let ahead = sync_git(
        dir,
        &["rev-list", "--count", &format!("{upstream}..{head}")],
    )?
    .parse::<u64>()?;
    anyhow::ensure!(
        ahead == 0,
        "Branch is ahead of or diverged from upstream ({ahead} local commits)"
    );
    Ok((head, upstream))
}

fn quarantine_and_reclone(
    user_dir: &std::path::Path,
    repo_name: &str,
    dir: &std::path::Path,
    clone_url: &str,
    pushed_at: &str,
    updated_at: &str,
    reason: &str,
) -> Result<(PathBuf, RepoSyncOutcome)> {
    let is_git = dir.join(".git").exists();
    let has_head = is_git && sync_git(dir, &["rev-parse", "--verify", "HEAD"]).is_ok();
    let is_diverged = is_git
        && (reason.contains("ahead of or diverged")
            || reason.contains("Git operation in progress")
            || (reason.contains("No usable upstream") && has_head));

    let base_quarantine = if is_diverged {
        user_dir.join(".diverged")
    } else {
        let kind = if !is_git {
            "non-git"
        } else if reason.contains("Dirty working tree") {
            "dirty-worktree"
        } else if reason.contains("Detached HEAD") {
            "detached-head"
        } else {
            "other"
        };
        user_dir.join(".sync-issues").join(kind)
    };
    std::fs::create_dir_all(&base_quarantine)?;

    let mut quarantine_dir = base_quarantine.join(repo_name);
    if quarantine_dir.exists() {
        let ts = chrono::Utc::now().format("%Y%m%d-%H%M%S");
        quarantine_dir = base_quarantine.join(format!("{repo_name}-{ts}"));
    }

    std::fs::rename(dir, &quarantine_dir).with_context(|| {
        format!(
            "Failed to move {} to quarantine {}",
            dir.display(),
            quarantine_dir.display()
        )
    })?;

    let reclone_outcome = sync_repository(dir, clone_url, None, pushed_at, updated_at, true);

    if let Err(report_err) = write_quarantine_diff_report(
        repo_name,
        dir,
        &quarantine_dir,
        clone_url,
        reason,
        is_diverged,
    ) {
        eprintln!(
            "⚠️ Failed to write DIFF_REPORT.md in {}: {report_err:#}",
            quarantine_dir.display()
        );
    }

    let outcome = reclone_outcome?;
    Ok((quarantine_dir, outcome))
}

fn write_quarantine_diff_report(
    repo_name: &str,
    fresh_dir: &std::path::Path,
    quarantine_dir: &std::path::Path,
    clone_url: &str,
    reason: &str,
    is_diverged: bool,
) -> Result<()> {
    let mut report = String::new();
    let now = chrono::Utc::now().to_rfc3339();
    let category = if is_diverged {
        "Diverged / Local Commits (.diverged)"
    } else {
        "Sync Issue (.sync-issues)"
    };

    report.push_str(&format!(
        "# Sync Quarantine & Diff Report: `{repo_name}`\n\n"
    ));
    report.push_str(&format!("- **Repository**: `{repo_name}`\n"));
    report.push_str(&format!("- **Remote URL**: `{clone_url}`\n"));
    report.push_str(&format!("- **Category**: {category}\n"));
    report.push_str(&format!("- **Quarantined At**: `{now}`\n"));
    report.push_str(&format!(
        "- **Quarantined Path**: `{}`\n",
        quarantine_dir.display()
    ));
    report.push_str(&format!(
        "- **Fresh Reclone Path**: `{}`\n",
        fresh_dir.display()
    ));
    report.push_str(&format!("- **Reason**: `{reason}`\n\n"));

    if quarantine_dir.join(".git").exists() {
        let status_sb =
            sync_git(quarantine_dir, &["status", "-sb"]).unwrap_or_else(|e| e.to_string());
        let branches =
            sync_git(quarantine_dir, &["branch", "-vv", "-a"]).unwrap_or_else(|e| e.to_string());
        let remotes = sync_git(quarantine_dir, &["remote", "-v"]).unwrap_or_else(|e| e.to_string());

        report.push_str("## Git Branch & Status Summary\n\n```text\n");
        report.push_str(&status_sb);
        report.push_str("\n```\n\n### Branches (`git branch -vv -a`)\n\n```text\n");
        report.push_str(&branches);
        report.push_str("\n```\n\n### Remotes (`git remote -v`)\n\n```text\n");
        report.push_str(&remotes);
        report.push_str("\n```\n\n");

        // Determine upstream/remote comparison ref
        let base_ref = ["@{upstream}", "origin/HEAD", "origin/main", "origin/master"]
            .into_iter()
            .find(|r| sync_git(quarantine_dir, &["rev-parse", "--verify", r]).is_ok());

        if let Some(base) = base_ref {
            let local_commits = sync_git(
                quarantine_dir,
                &["log", "--stat", "--oneline", &format!("{base}..HEAD")],
            )
            .unwrap_or_default();
            let remote_commits = sync_git(
                quarantine_dir,
                &["log", "--stat", "--oneline", &format!("HEAD..{base}")],
            )
            .unwrap_or_default();

            report.push_str(&format!(
                "## Local Commits Not in `{base}` (`{base}..HEAD`)\n\n```text\n{}\n```\n\n",
                if local_commits.is_empty() {
                    "(none)"
                } else {
                    &local_commits
                }
            ));
            report.push_str(&format!(
                "## Upstream Commits Not in Local (`HEAD..{base}`)\n\n```text\n{}\n```\n\n",
                if remote_commits.is_empty() {
                    "(none)"
                } else {
                    &remote_commits
                }
            ));

            if let Ok(patch) = sync_git(quarantine_dir, &["log", "-p", &format!("{base}..HEAD")]) {
                if !patch.is_empty() {
                    let patch_path = quarantine_dir.join("local-commits.patch");
                    let _ = std::fs::write(&patch_path, &patch);
                    report.push_str(&format!(
                        "- Saved full local commit patch ({} bytes) to [`local-commits.patch`](file://{})\n\n",
                        patch.len(),
                        patch_path.display()
                    ));
                }
            }
        } else if sync_git(quarantine_dir, &["rev-parse", "--verify", "HEAD"]).is_ok() {
            let local_log = sync_git(
                quarantine_dir,
                &["log", "--stat", "--oneline", "-n", "30", "HEAD"],
            )
            .unwrap_or_default();
            report.push_str("## Local Commit History (No Upstream Tracking Ref)\n\n```text\n");
            report.push_str(&local_log);
            report.push_str("\n```\n\n");
            if let Ok(patch) = sync_git(quarantine_dir, &["log", "-p", "-n", "20", "HEAD"]) {
                if !patch.is_empty() {
                    let patch_path = quarantine_dir.join("local-commits.patch");
                    let _ = std::fs::write(&patch_path, &patch);
                    report.push_str(&format!(
                        "- Saved local commit patch ({} bytes) to [`local-commits.patch`](file://{})\n\n",
                        patch.len(),
                        patch_path.display()
                    ));
                }
            }
        }

        // Working tree diff
        let diff_stat = sync_git(quarantine_dir, &["diff", "--stat", "HEAD"]).unwrap_or_default();
        let porcelain = non_ignorable_porcelain_status(quarantine_dir).unwrap_or_default();
        let porcelain_lines: Vec<&str> = porcelain.lines().collect();

        report.push_str("## Working Tree Changes (`git diff --stat HEAD`)\n\n```text\n");
        report.push_str(if diff_stat.is_empty() {
            "(clean tracked working tree)"
        } else {
            &diff_stat
        });
        report.push_str("\n```\n\n");

        if !porcelain_lines.is_empty() {
            report.push_str(&format!(
                "### Porcelain Status ({} entries)\n\n```text\n",
                porcelain_lines.len()
            ));
            for line in porcelain_lines.iter().take(150) {
                report.push_str(line);
                report.push('\n');
            }
            if porcelain_lines.len() > 150 {
                report.push_str(&format!(
                    "... ({} additional lines omitted)\n",
                    porcelain_lines.len() - 150
                ));
            }
            report.push_str("```\n\n");
        }

        if let Ok(wt_diff) = sync_git(quarantine_dir, &["diff", "HEAD"]) {
            if !wt_diff.is_empty() && wt_diff.len() <= 2 * 1024 * 1024 {
                let wt_path = quarantine_dir.join("working-tree.diff");
                let _ = std::fs::write(&wt_path, &wt_diff);
                report.push_str(&format!(
                    "- Saved working-tree diff ({} bytes) to [`working-tree.diff`](file://{})\n\n",
                    wt_diff.len(),
                    wt_path.display()
                ));
            }
        }
    } else {
        report.push_str("## Non-Git Directory Contents\n\n```text\n");
        if let Ok(entries) = std::fs::read_dir(quarantine_dir) {
            for entry in entries.flatten() {
                if let Ok(meta) = entry.metadata() {
                    report.push_str(&format!(
                        "{} ({} bytes)\n",
                        entry.file_name().to_string_lossy(),
                        meta.len()
                    ));
                }
            }
        }
        report.push_str("```\n\n");

        if fresh_dir.exists() {
            if let Ok(diff_out) = std::process::Command::new("diff")
                .args([
                    "-ru",
                    "--exclude=.git",
                    "--exclude=.m8",
                    "--exclude=DIFF_REPORT.md",
                ])
                .arg(fresh_dir)
                .arg(quarantine_dir)
                .output()
            {
                let diff_txt = String::from_utf8_lossy(&diff_out.stdout);
                if !diff_txt.trim().is_empty() {
                    let wt_path = quarantine_dir.join("working-tree.diff");
                    let _ = std::fs::write(&wt_path, diff_txt.as_bytes());
                    report.push_str(&format!(
                        "## Diff Against Fresh Clone (`diff -ru`)\n\n```diff\n{}\n```\n\n",
                        diff_txt.chars().take(8000).collect::<String>()
                    ));
                }
            }
        }
    }

    std::fs::write(quarantine_dir.join("DIFF_REPORT.md"), report)?;
    Ok(())
}

fn sync_unborn_repository(
    dir: &std::path::Path,
    branch: &str,
    cached: Option<&RepoSyncRecord>,
    pushed_at: &str,
    updated_at: &str,
    refresh_all: bool,
) -> Result<RepoSyncOutcome> {
    let remote = match sync_git(
        dir,
        &["config", "--get", &format!("branch.{branch}.remote")],
    )
    .or_else(|_| sync_git(dir, &["remote", "get-url", "origin"]).map(|_| "origin".to_string()))
    {
        Ok(r) => r,
        Err(error) => {
            return Ok(RepoSyncOutcome::Blocked {
                reason: format!("No usable upstream: {error:#}"),
                fetched: false,
            })
        }
    };
    let remote_refs_before = sync_git(
        dir,
        &[
            "for-each-ref",
            "--format=%(refname)",
            &format!("refs/remotes/{remote}"),
        ],
    )?;
    if !refresh_all
        && remote_refs_before.is_empty()
        && !pushed_at.is_empty()
        && !updated_at.is_empty()
        && cached.is_some_and(|rec| {
            rec.head_sha == UNBORN_HEAD_SHA
                && rec.pushed_at == pushed_at
                && rec.updated_at == updated_at
        })
    {
        return Ok(RepoSyncOutcome::Skipped);
    }

    sync_git(dir, &["fetch", "--prune", "--quiet", "--", &remote])?;

    let branch_after = match verify_clean_attached_worktree(dir) {
        Ok(b) => b,
        Err(error) => {
            return Ok(RepoSyncOutcome::Blocked {
                reason: format!("{error:#}"),
                fetched: true,
            })
        }
    };
    if branch_after != branch || sync_git(dir, &["rev-parse", "--verify", "HEAD"]).is_ok() {
        return Ok(RepoSyncOutcome::Blocked {
            reason: "HEAD changed during fetch".into(),
            fetched: true,
        });
    }

    let remote_refs_after = sync_git(
        dir,
        &[
            "for-each-ref",
            "--format=%(refname)",
            &format!("refs/remotes/{remote}"),
        ],
    )?;
    if remote_refs_after.is_empty() {
        return Ok(RepoSyncOutcome::UpToDate(UNBORN_HEAD_SHA.to_string()));
    }

    let target = match sync_git(dir, &["rev-parse", "--verify", "@{upstream}"]).or_else(|_| {
        sync_git(
            dir,
            &[
                "rev-parse",
                "--verify",
                &format!("refs/remotes/{remote}/{branch}"),
            ],
        )
    }) {
        Ok(t) => t,
        Err(error) => {
            return Ok(RepoSyncOutcome::Blocked {
                reason: format!("No usable upstream after fetch: {error:#}"),
                fetched: true,
            })
        }
    };

    sync_git(
        dir,
        &[
            "-c",
            "merge.autostash=false",
            "merge",
            "--ff-only",
            "--quiet",
            &target,
        ],
    )?;
    if sync_git(dir, &["rev-parse", "--verify", "@{upstream}"]).is_err() {
        let _ = sync_git(
            dir,
            &[
                "branch",
                &format!("--set-upstream-to={remote}/{branch}"),
                branch,
            ],
        );
    }
    let (after, upstream) = sync_ready_branch(dir)?;
    anyhow::ensure!(
        after == target && after == upstream,
        "Branch changed during reconciliation"
    );
    Ok(RepoSyncOutcome::Updated(after))
}

fn sync_repository(
    dir: &std::path::Path,
    clone_url: &str,
    cached: Option<&RepoSyncRecord>,
    pushed_at: &str,
    updated_at: &str,
    refresh_all: bool,
) -> Result<RepoSyncOutcome> {
    let absolute = std::path::absolute(dir)?;
    let dir = absolute.as_path();
    if !dir.join(".git").exists() {
        if dir.exists() {
            anyhow::ensure!(dir.is_dir() && std::fs::read_dir(dir)?.next().is_none(),
                "Existing path {} is not an empty Git clone destination; review or move its contents manually", dir.display());
        }
        let output = std::process::Command::new("git")
            .current_dir(dir.parent().context("Clone destination has no parent")?)
            .env("GIT_TERMINAL_PROMPT", "0")
            .args(["clone", "--quiet", "--"])
            .arg(clone_url)
            .arg(dir)
            .output()?;
        anyhow::ensure!(
            output.status.success(),
            "Clone failed: {}",
            String::from_utf8_lossy(&output.stderr).trim()
        );
        // Empty remotes clone as clean unborn repositories with no remote refs.
        return Ok(match sync_ready_branch(dir) {
            Ok((head, upstream)) if head == upstream => RepoSyncOutcome::Cloned(head),
            Err(_) if is_empty_unborn_repo(dir) && verify_clean_attached_worktree(dir).is_ok() => {
                RepoSyncOutcome::Cloned(UNBORN_HEAD_SHA.to_string())
            }
            result => RepoSyncOutcome::Blocked {
                reason: format!("Clone has no reconciled branch: {result:?}"),
                fetched: false,
            },
        });
    }
    let (head, upstream) = match sync_ready_branch(dir) {
        Ok(state) => state,
        Err(error) => {
            if sync_git(dir, &["rev-parse", "--verify", "HEAD"]).is_err() {
                if let Ok(branch) = verify_clean_attached_worktree(dir) {
                    return sync_unborn_repository(
                        dir,
                        &branch,
                        cached,
                        pushed_at,
                        updated_at,
                        refresh_all,
                    );
                }
            }
            return Ok(RepoSyncOutcome::Blocked {
                reason: format!("{error:#}"),
                fetched: false,
            });
        }
    };
    // Timestamps are only an optimization, not evidence of remote equality.
    if !refresh_all
        && head == upstream
        && !pushed_at.is_empty()
        && !updated_at.is_empty()
        && cached.is_some_and(|rec| {
            rec.head_sha == head && rec.pushed_at == pushed_at && rec.updated_at == updated_at
        })
    {
        return Ok(RepoSyncOutcome::Skipped);
    }
    let branch = sync_git(dir, &["symbolic-ref", "--short", "HEAD"])?;
    let remote = sync_git(
        dir,
        &["config", "--get", &format!("branch.{branch}.remote")],
    )?;
    // Fetch the actual upstream remote even if it opts out of `fetch --all`.
    sync_git(dir, &["fetch", "--prune", "--quiet", "--", &remote])?;
    let (before, target) = match sync_ready_branch(dir) {
        Ok(state) => state,
        Err(error) => {
            return Ok(RepoSyncOutcome::Blocked {
                reason: format!("{error:#}"),
                fetched: true,
            })
        }
    };
    if before != head {
        return Ok(RepoSyncOutcome::Blocked {
            reason: "HEAD changed during fetch".into(),
            fetched: true,
        });
    }
    if before != target {
        // Merge the inspected commit, avoiding a second implicit fetch by `pull`.
        sync_git(
            dir,
            &[
                "-c",
                "merge.autostash=false",
                "merge",
                "--ff-only",
                "--quiet",
                &target,
            ],
        )?;
    }
    let (after, upstream) = sync_ready_branch(dir)?;
    anyhow::ensure!(
        after == target && after == upstream,
        "Branch changed during reconciliation"
    );
    Ok(if before == after {
        RepoSyncOutcome::UpToDate(after)
    } else {
        RepoSyncOutcome::Updated(after)
    })
}

#[allow(clippy::items_after_test_module)]
#[cfg(test)]
mod sync_tests {
    use super::*;
    use std::path::{Path, PathBuf};

    fn git(dir: &Path, args: &[&str]) -> String {
        let output = std::process::Command::new("git")
            .current_dir(dir)
            .env("GIT_CONFIG_NOSYSTEM", "1")
            .env("GIT_CONFIG_GLOBAL", "/dev/null")
            .arg("-C")
            .arg(dir)
            .args([
                "-c",
                "user.name=Sync Fixture",
                "-c",
                "user.email=sync@example.invalid",
                "-c",
                "commit.gpgsign=false",
            ])
            .args(args)
            .output()
            .unwrap();
        assert!(
            output.status.success(),
            "{args:?}: {}",
            String::from_utf8_lossy(&output.stderr)
        );
        String::from_utf8(output.stdout).unwrap().trim().to_owned()
    }

    fn temp_dir() -> tempfile::TempDir {
        let target = Path::new(env!("CARGO_MANIFEST_DIR")).join("target/sync-fixtures");
        std::fs::create_dir_all(&target).unwrap();
        tempfile::tempdir_in(target).unwrap()
    }

    struct Fixture {
        _temp: tempfile::TempDir,
        remote: PathBuf,
        publisher: PathBuf,
        local: PathBuf,
    }

    impl Fixture {
        fn new() -> Self {
            let temp = temp_dir();
            let remote = temp.path().join("remote.git");
            let publisher = temp.path().join("publisher");
            let local = temp.path().join("local");
            git(
                temp.path(),
                &["init", "--bare", "-b", "main", remote.to_str().unwrap()],
            );
            git(
                temp.path(),
                &[
                    "clone",
                    remote.to_str().unwrap(),
                    publisher.to_str().unwrap(),
                ],
            );
            Self::commit(&publisher, "base");
            git(&publisher, &["push", "-u", "origin", "main"]);
            git(
                temp.path(),
                &["clone", remote.to_str().unwrap(), local.to_str().unwrap()],
            );
            Self {
                _temp: temp,
                remote,
                publisher,
                local,
            }
        }

        fn commit(dir: &Path, name: &str) {
            std::fs::write(dir.join(name), name).unwrap();
            git(dir, &["add", name]);
            git(dir, &["commit", "-m", name]);
        }

        fn advance(&self) {
            Self::commit(&self.publisher, "remote-change");
            git(&self.publisher, &["push"]);
        }

        fn sync(&self, cached: Option<&RepoSyncRecord>, refresh: bool) -> Result<RepoSyncOutcome> {
            sync_repository(
                &self.local,
                self.remote.to_str().unwrap(),
                cached,
                "pushed",
                "updated",
                refresh,
            )
        }

        fn record(&self) -> RepoSyncRecord {
            RepoSyncRecord {
                pushed_at: "pushed".into(),
                updated_at: "updated".into(),
                head_sha: git(&self.local, &["rev-parse", "HEAD"]),
                last_synced_at: "old-receipt".into(),
            }
        }
    }

    #[test]
    fn clean_clone_fast_forward_and_verified_equal() {
        let f = Fixture::new();
        let destination = f._temp.path().join("new-clone");
        assert!(matches!(
            sync_repository(
                &destination,
                f.remote.to_str().unwrap(),
                None,
                "p",
                "u",
                false
            )
            .unwrap(),
            RepoSyncOutcome::Cloned(_)
        ));
        f.advance();
        let target = git(&f.publisher, &["rev-parse", "HEAD"]);
        assert_eq!(
            f.sync(None, false).unwrap(),
            RepoSyncOutcome::Updated(target.clone())
        );
        assert_eq!(git(&f.local, &["rev-parse", "HEAD"]), target);
        assert_eq!(
            f.sync(None, false).unwrap(),
            RepoSyncOutcome::UpToDate(target)
        );
    }

    #[test]
    fn dirty_staged_and_untracked_work_survives_even_with_matching_cache() {
        for mode in ["modified", "staged", "untracked"] {
            let f = Fixture::new();
            let record = f.record();
            f.advance();
            let file = if mode == "untracked" {
                "private"
            } else {
                "base"
            };
            std::fs::write(f.local.join(file), "preserve me").unwrap();
            if mode == "staged" {
                git(&f.local, &["add", file]);
            }
            let status = git(&f.local, &["status", "--porcelain"]);
            assert!(matches!(
                f.sync(Some(&record), false).unwrap(),
                RepoSyncOutcome::Blocked { fetched: false, .. }
            ));
            assert_eq!(git(&f.local, &["rev-parse", "HEAD"]), record.head_sha);
            assert_eq!(git(&f.local, &["status", "--porcelain"]), status);
            assert_eq!(
                std::fs::read_to_string(f.local.join(file)).unwrap(),
                "preserve me"
            );
        }
    }

    #[test]
    fn detached_missing_upstream_ahead_and_diverged_are_blocked() {
        for mode in ["detached", "no-upstream", "ahead", "diverged"] {
            let f = Fixture::new();
            match mode {
                "detached" => {
                    git(&f.local, &["checkout", "--detach"]);
                }
                "no-upstream" => {
                    git(&f.local, &["branch", "--unset-upstream"]);
                }
                _ => {
                    Fixture::commit(&f.local, "local-change");
                }
            }
            f.advance();
            if mode == "diverged" {
                git(&f.local, &["fetch"]);
            }
            let head = git(&f.local, &["rev-parse", "HEAD"]);
            let outcome = f.sync(None, true).unwrap();
            assert!(
                matches!(outcome, RepoSyncOutcome::Blocked { fetched: false, .. }),
                "{mode}: {outcome:?}"
            );
            assert_eq!(git(&f.local, &["rev-parse", "HEAD"]), head);
        }
    }

    #[test]
    fn remote_rewrite_is_fetched_only_and_invalidates_success_cache() {
        let f = Fixture::new();
        let record = f.record();
        git(&f.publisher, &["checkout", "--orphan", "rewritten"]);
        Fixture::commit(&f.publisher, "rewrite");
        git(&f.publisher, &["push", "--force", "origin", "HEAD:main"]);
        let outcome = f.sync(Some(&record), true).unwrap();
        assert!(matches!(
            outcome,
            RepoSyncOutcome::Blocked { fetched: true, .. }
        ));
        assert_eq!(git(&f.local, &["rev-parse", "HEAD"]), record.head_sha);
        assert_eq!(
            git(&f.local, &["rev-parse", "origin/main"]),
            git(&f.publisher, &["rev-parse", "HEAD"])
        );
        let mut state = std::collections::BTreeMap::from([("local".into(), record)]);
        update_repo_sync_record(&mut state, "local", "new-pushed", "new-updated", &outcome);
        assert!(state.is_empty());
    }

    #[test]
    fn cache_hits_are_skips_and_local_reset_is_reconciled() {
        let f = Fixture::new();
        let record = f.record();
        f.advance();
        assert_eq!(
            f.sync(Some(&record), false).unwrap(),
            RepoSyncOutcome::Skipped
        );
        let mut state = std::collections::BTreeMap::from([("local".into(), record.clone())]);
        update_repo_sync_record(
            &mut state,
            "local",
            "pushed",
            "updated",
            &RepoSyncOutcome::Skipped,
        );
        assert_eq!(state["local"].last_synced_at, "old-receipt");
        assert!(matches!(
            f.sync(Some(&record), true).unwrap(),
            RepoSyncOutcome::Updated(_)
        ));
        let newer = f.record();
        git(&f.local, &["reset", "--hard", &record.head_sha]);
        assert_eq!(
            f.sync(Some(&newer), false).unwrap(),
            RepoSyncOutcome::Updated(newer.head_sha)
        );
    }

    #[test]
    fn missing_timestamps_and_fetch_failures_do_not_claim_success() {
        let f = Fixture::new();
        f.advance();
        let record = f.record();
        assert!(matches!(
            sync_repository(&f.local, "unused", Some(&record), "", "", false).unwrap(),
            RepoSyncOutcome::Updated(_)
        ));
        let head = git(&f.local, &["rev-parse", "HEAD"]);
        git(
            &f.local,
            &[
                "remote",
                "set-url",
                "origin",
                f._temp.path().join("missing.git").to_str().unwrap(),
            ],
        );
        assert!(f.sync(None, true).is_err());
        assert_eq!(git(&f.local, &["rev-parse", "HEAD"]), head);
    }

    #[test]
    fn upstream_remote_is_fetched_even_when_excluded_from_fetch_all() {
        let f = Fixture::new();
        git(&f.local, &["config", "remote.origin.skipFetchAll", "true"]);
        f.advance();
        assert_eq!(
            f.sync(None, true).unwrap(),
            RepoSyncOutcome::Updated(git(&f.publisher, &["rev-parse", "HEAD"]))
        );
    }

    #[test]
    fn in_progress_operation_and_empty_clone_do_not_get_success_receipts() {
        let f = Fixture::new();
        std::fs::create_dir(f.local.join(".git/rebase-merge")).unwrap();
        assert!(matches!(
            f.sync(None, true).unwrap(),
            RepoSyncOutcome::Blocked { fetched: false, .. }
        ));
        let empty = f._temp.path().join("empty.git");
        let empty_clone = f._temp.path().join("empty-clone");
        git(
            f._temp.path(),
            &["init", "--bare", "-b", "main", empty.to_str().unwrap()],
        );
        assert_eq!(
            sync_repository(&empty_clone, empty.to_str().unwrap(), None, "p", "u", false).unwrap(),
            RepoSyncOutcome::Cloned(UNBORN_HEAD_SHA.to_string())
        );
        let unborn_record = RepoSyncRecord {
            pushed_at: "p".into(),
            updated_at: "u".into(),
            head_sha: UNBORN_HEAD_SHA.into(),
            last_synced_at: "receipt".into(),
        };
        assert_eq!(
            sync_repository(
                &empty_clone,
                empty.to_str().unwrap(),
                Some(&unborn_record),
                "p",
                "u",
                false
            )
            .unwrap(),
            RepoSyncOutcome::Skipped
        );
        assert_eq!(
            sync_repository(
                &empty_clone,
                empty.to_str().unwrap(),
                Some(&unborn_record),
                "p",
                "u",
                true
            )
            .unwrap(),
            RepoSyncOutcome::UpToDate(UNBORN_HEAD_SHA.to_string())
        );

        // Dirty unborn worktree is still protected and blocked.
        std::fs::write(empty_clone.join("draft.txt"), "local draft").unwrap();
        assert!(matches!(
            sync_repository(
                &empty_clone,
                empty.to_str().unwrap(),
                Some(&unborn_record),
                "p",
                "u",
                false
            )
            .unwrap(),
            RepoSyncOutcome::Blocked { fetched: false, .. }
        ));
        std::fs::remove_file(empty_clone.join("draft.txt")).unwrap();

        // Once the previously empty remote gets its first commit, syncing advances the unborn clone.
        let empty_pub = f._temp.path().join("empty-pub");
        git(
            f._temp.path(),
            &[
                "clone",
                empty.to_str().unwrap(),
                empty_pub.to_str().unwrap(),
            ],
        );
        Fixture::commit(&empty_pub, "initial");
        git(&empty_pub, &["push", "-u", "origin", "main"]);
        let first_sha = git(&empty_pub, &["rev-parse", "HEAD"]);
        assert_eq!(
            sync_repository(
                &empty_clone,
                empty.to_str().unwrap(),
                Some(&unborn_record),
                "p2",
                "u2",
                false
            )
            .unwrap(),
            RepoSyncOutcome::Updated(first_sha.clone())
        );
        assert_eq!(git(&empty_clone, &["rev-parse", "HEAD"]), first_sha);
    }

    #[tokio::test]
    async fn corrupt_cache_is_preserved_and_lock_released_on_error() {
        let temp = temp_dir();
        let owner = temp.path().join("owner");
        std::fs::create_dir(&owner).unwrap();
        let cache = owner.join(".st-sync-state.json");
        std::fs::write(&cache, "broken cache").unwrap();
        let cmd = crate::cli::SyncUserCommand {
            user: "owner".into(),
            dir: temp.path().to_path_buf(),
            limit: 1,
            jobs: 1,
            refresh_all: false,
            no_forks: false,
            no_archived: false,
            quarantine: false,
        };
        assert!(handle_sync_user(&cmd)
            .await
            .unwrap_err()
            .to_string()
            .contains("Invalid sync cache"));
        assert_eq!(std::fs::read_to_string(cache).unwrap(), "broken cache");
        assert!(lock_sync_destination(&owner).is_ok());
    }

    #[cfg(unix)]
    #[test]
    #[ignore = "subprocess probe used by summary_and_persisted_receipts_match_actual_results"]
    fn sync_handler_child() {
        let base = std::env::var_os("ST_SYNC_TEST_BASE").unwrap();
        let cmd = crate::cli::SyncUserCommand {
            user: "fixture-owner".into(),
            dir: PathBuf::from(base),
            limit: 100,
            jobs: 4,
            refresh_all: false,
            no_forks: false,
            no_archived: false,
            quarantine: false,
        };
        let result = tokio::runtime::Runtime::new()
            .unwrap()
            .block_on(handle_sync_user(&cmd));
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("failed for 1 repositories"));
    }

    #[cfg(unix)]
    #[test]
    fn summary_and_persisted_receipts_match_actual_results() {
        use std::os::unix::fs::PermissionsExt;
        let f = Fixture::new();
        let base = f._temp.path().join("destination");
        let owner = base.join("fixture-owner");
        std::fs::create_dir_all(&owner).unwrap();
        let mut cache = std::collections::BTreeMap::new();
        for name in ["updated", "skipped", "dirty"] {
            git(&owner, &["clone", f.remote.to_str().unwrap(), name]);
            cache.insert(name.to_owned(), f.record());
        }
        // Updated must be fetched; skipped deliberately retains matching timestamps.
        cache.get_mut("updated").unwrap().pushed_at = "older".into();
        std::fs::write(owner.join("dirty/private"), "keep").unwrap();
        f.advance();
        git(&owner, &["clone", f.remote.to_str().unwrap(), "equal"]);
        std::fs::create_dir(owner.join("not-git")).unwrap();
        std::fs::write(owner.join("not-git/precious"), "keep").unwrap();
        let rewritten = Fixture::new();
        git(
            &owner,
            &["clone", rewritten.remote.to_str().unwrap(), "rewritten"],
        );
        cache.insert("rewritten".into(), rewritten.record());
        cache.get_mut("rewritten").unwrap().pushed_at = "older".into();
        git(&rewritten.publisher, &["checkout", "--orphan", "rewrite"]);
        Fixture::commit(&rewritten.publisher, "rewrite");
        git(
            &rewritten.publisher,
            &["push", "--force", "origin", "HEAD:main"],
        );
        std::fs::write(
            owner.join(".st-sync-state.json"),
            serde_json::to_vec(&cache).unwrap(),
        )
        .unwrap();
        let entries: Vec<_> = ["updated", "skipped", "dirty", "equal", "not-git", "rewritten", "cloned"].into_iter()
            .map(|name| serde_json::json!({"name": name, "pushedAt": "pushed", "updatedAt": "updated", "url": f.remote.to_str().unwrap()})).collect();
        let json = f._temp.path().join("repos.json");
        std::fs::write(&json, serde_json::to_vec(&entries).unwrap()).unwrap();
        let bin = f._temp.path().join("bin");
        std::fs::create_dir(&bin).unwrap();
        std::fs::write(bin.join("gh"), "#!/bin/sh\ncat \"$ST_SYNC_TEST_JSON\"\n").unwrap();
        std::fs::set_permissions(bin.join("gh"), std::fs::Permissions::from_mode(0o755)).unwrap();
        let path = std::env::join_paths(
            std::iter::once(bin).chain(std::env::split_paths(&std::env::var_os("PATH").unwrap())),
        )
        .unwrap();
        let output = std::process::Command::new(std::env::current_exe().unwrap())
            .current_dir(f._temp.path())
            .args([
                "--exact",
                "cli_commands::sync_tests::sync_handler_child",
                "--ignored",
                "--nocapture",
            ])
            .env("ST_SYNC_TEST_BASE", &base)
            .env("ST_SYNC_TEST_JSON", json)
            .env("PATH", path)
            .output()
            .unwrap();
        let stdout = String::from_utf8(output.stdout).unwrap();
        assert!(
            output.status.success(),
            "{stdout}\n{}",
            String::from_utf8_lossy(&output.stderr)
        );
        for expected in [
            "Cloned: 1",
            "Updated: 1",
            "Up-to-date (verified): 1",
            "Failed: 1",
            "Skipped (cached): 1",
            "Blocked: 2",
            "Fetched-only (subset of blocked): 1",
        ] {
            assert!(stdout.contains(expected), "Missing {expected}: {stdout}");
        }
        let saved: std::collections::BTreeMap<String, RepoSyncRecord> =
            serde_json::from_slice(&std::fs::read(owner.join(".st-sync-state.json")).unwrap())
                .unwrap();
        assert_eq!(
            saved.keys().map(String::as_str).collect::<Vec<_>>(),
            ["cloned", "equal", "skipped", "updated"]
        );
        assert_eq!(saved["skipped"].last_synced_at, "old-receipt");
        assert_eq!(
            std::fs::read_to_string(owner.join("dirty/private")).unwrap(),
            "keep"
        );
        assert_eq!(
            std::fs::read_to_string(owner.join("not-git/precious")).unwrap(),
            "keep"
        );
    }

    #[test]
    fn existing_non_git_contents_are_preserved() {
        let temp = temp_dir();
        let file = temp.path().join("precious");
        std::fs::write(&file, "keep").unwrap();
        let error = sync_repository(temp.path(), "unused", None, "p", "u", false).unwrap_err();
        assert!(error.to_string().contains("review or move"));
        assert_eq!(std::fs::read_to_string(file).unwrap(), "keep");
        assert!(!temp.path().join(".git").exists());
    }

    #[test]
    fn linked_worktree_is_supported() {
        let f = Fixture::new();
        let linked = f._temp.path().join("linked");
        git(
            &f.local,
            &[
                "worktree",
                "add",
                "-b",
                "linked",
                linked.to_str().unwrap(),
                "origin/main",
            ],
        );
        f.advance();
        assert!(linked.join(".git").is_file());
        assert!(matches!(
            sync_repository(&linked, "unused", None, "p", "u", true).unwrap(),
            RepoSyncOutcome::Updated(_)
        ));
    }

    #[test]
    #[ignore = "subprocess probe used by destination_lock_excludes_other_processes"]
    fn lock_child() {
        let dir = std::env::var_os("ST_SYNC_LOCK_TEST_DIR").unwrap();
        let should_lock = std::env::var_os("ST_SYNC_LOCK_TEST_AVAILABLE").is_some();
        assert_eq!(lock_sync_destination(Path::new(&dir)).is_ok(), should_lock);
    }

    #[test]
    fn destination_lock_excludes_other_processes() {
        let temp = temp_dir();
        let lock = lock_sync_destination(temp.path()).unwrap();
        let probe = |available: bool| {
            let mut command = std::process::Command::new(std::env::current_exe().unwrap());
            command
                .current_dir(temp.path())
                .args([
                    "--exact",
                    "cli_commands::sync_tests::lock_child",
                    "--ignored",
                ])
                .env("ST_SYNC_LOCK_TEST_DIR", temp.path())
                .env_remove("ST_SYNC_LOCK_TEST_AVAILABLE");
            if available {
                command.env("ST_SYNC_LOCK_TEST_AVAILABLE", "1");
            }
            assert!(command.status().unwrap().success());
        };
        probe(false);
        drop(lock);
        probe(true);
    }

    #[test]
    fn untracked_smart_tree_metadata_does_not_block_sync() {
        let f = Fixture::new();
        std::fs::create_dir_all(f.local.join(".st")).unwrap();
        std::fs::write(f.local.join(".st/alexiai_version.txt"), "1.0").unwrap();
        std::fs::write(f.local.join(".m8"), "consciousness").unwrap();
        f.advance();
        let target = git(&f.publisher, &["rev-parse", "HEAD"]);
        assert_eq!(
            f.sync(None, true).unwrap(),
            RepoSyncOutcome::Updated(target)
        );
    }

    #[test]
    fn quarantine_moves_diverged_and_dirty_writes_diff_report_and_reclones() {
        let f = Fixture::new();
        let user_dir = f._temp.path().join("org");
        std::fs::create_dir_all(&user_dir).unwrap();

        // 1. Diverged repository -> .diverged/<repo>
        let div_dir = user_dir.join("diverged-repo");
        git(
            &user_dir,
            &[
                "clone",
                f.remote.to_str().unwrap(),
                div_dir.to_str().unwrap(),
            ],
        );
        Fixture::commit(&div_dir, "local-diverged-work");
        f.advance();
        let target = git(&f.publisher, &["rev-parse", "HEAD"]);

        let blocked =
            sync_repository(&div_dir, f.remote.to_str().unwrap(), None, "p", "u", true).unwrap();
        let RepoSyncOutcome::Blocked { reason, .. } = blocked else {
            panic!("expected blocked outcome");
        };
        let (q_path, new_outcome) = quarantine_and_reclone(
            &user_dir,
            "diverged-repo",
            &div_dir,
            f.remote.to_str().unwrap(),
            "p",
            "u",
            &reason,
        )
        .unwrap();
        assert_eq!(q_path, user_dir.join(".diverged/diverged-repo"));
        assert_eq!(new_outcome, RepoSyncOutcome::Cloned(target.clone()));
        assert!(q_path.join("DIFF_REPORT.md").is_file());
        assert!(q_path.join("local-commits.patch").is_file());
        assert!(q_path.join("local-diverged-work").is_file());
        assert!(!div_dir.join("local-diverged-work").exists());
        assert_eq!(git(&div_dir, &["rev-parse", "HEAD"]), target);

        // 2. Dirty working tree repository -> .sync-issues/dirty-worktree/<repo>
        let dirty_dir = user_dir.join("dirty-repo");
        git(
            &user_dir,
            &[
                "clone",
                f.remote.to_str().unwrap(),
                dirty_dir.to_str().unwrap(),
            ],
        );
        std::fs::write(dirty_dir.join("base"), "modified-locally").unwrap();
        let blocked_dirty =
            sync_repository(&dirty_dir, f.remote.to_str().unwrap(), None, "p", "u", true).unwrap();
        let RepoSyncOutcome::Blocked {
            reason: dirty_reason,
            ..
        } = blocked_dirty
        else {
            panic!("expected blocked dirty outcome");
        };
        let (q_dirty_path, dirty_outcome) = quarantine_and_reclone(
            &user_dir,
            "dirty-repo",
            &dirty_dir,
            f.remote.to_str().unwrap(),
            "p",
            "u",
            &dirty_reason,
        )
        .unwrap();
        assert_eq!(
            q_dirty_path,
            user_dir.join(".sync-issues/dirty-worktree/dirty-repo")
        );
        assert_eq!(dirty_outcome, RepoSyncOutcome::Cloned(target));
        assert!(q_dirty_path.join("DIFF_REPORT.md").is_file());
        assert!(q_dirty_path.join("working-tree.diff").is_file());
        assert_eq!(
            std::fs::read_to_string(dirty_dir.join("base")).unwrap(),
            "base"
        );
    }
}

/// Handle `st memnet <action>` CLI commands for local folder/drive sharing,
/// multi-medium routing (ESP32, WiFi, BLE, LoRa, Ethernet, Ultrasonic Chirp),
/// MEM|8 wave packet Q&A LLM offload, media chunking, and acoustic chirp modem.
pub fn handle_memnet_command(cmd: &crate::cli::MemnetCommand) -> Result<()> {
    use crate::memnet::{
        chunk_media_stream, reassemble_media_chunks, AcousticModemConfig, MediaKind, MemnetMedium,
        MemnetNodeState, MemnetPayload, MemnetRouteEntry, SharedFolderManifest,
        UltrasonicChirpModem, MEMNET_CHUNK_BYTES, MEMNET_FS_PORT,
    };

    let mut state = MemnetNodeState::load_or_default();
    let action = cmd.action.trim().to_ascii_lowercase();

    match action.as_str() {
        "assess" | "hw" | "hardware" | "probe-hw" => {
            let hw = crate::memnet::assess_host_hardware();
            if cmd.json {
                println!("{}", serde_json::to_string_pretty(&hw)?);
            } else {
                println!(
                    "🧬 Sovereign Host Hardware & Local LLM Assessment (Node {} '{}')",
                    state.local_address, hw.hostname
                );
                println!(
                    "   OS / Arch / CPU:       {}/{} · {} ({} cores)",
                    hw.os, hw.cpu_arch, hw.cpu_model, hw.cpu_cores
                );
                println!(
                    "   SIMD Acceleration:     {}",
                    hw.simd_capabilities.join(", ")
                );
                println!(
                    "   System Memory:         {} MB total ({} MB available) | Trained .m8 Souls: {}",
                    hw.ram_total_mb, hw.ram_available_mb, hw.trained_m8_souls_count
                );
                println!(
                    "   GPU / Unified VRAM:    {} ({} MB VRAM)",
                    if hw.gpus.is_empty() {
                        "CPU / SPQT Ternary SIMD".to_string()
                    } else {
                        hw.gpus.join(" + ")
                    },
                    hw.vram_total_mb
                );
                println!(
                    "   Sovereign Tier:        {} (cloud_required={})",
                    hw.compute_tier.as_str(),
                    hw.cloud_required
                );
                println!("   Runnable Local Models (100% Offline, No Cloud Needed):");
                for m in &hw.recommended_local_models {
                    println!("     • {m}");
                }
                if !hw.active_local_llm_services.is_empty() {
                    println!("   Active Local LLM Services:");
                    for ep in &hw.active_local_llm_services {
                        println!("     • {ep}");
                    }
                }
                println!("   Non-TCP/IP MemNet Physical Transports Detected:");
                for tr in &hw.non_ip_transports {
                    println!("     • {tr}");
                }
                println!(
                    "   Local Area LLM Help:   can_broadcast_llm_to_memnet={} (run `st memnet advertise` to broadcast to local MemNet graph)",
                    hw.can_broadcast_llm_to_memnet
                );
            }
        }
        "advertise" | "beacon" | "llm-broadcast" | "broadcast-llm" => {
            let (hw, adv, mediums_emitted) = state.advertise_llm_capability()?;
            let _ = state.save();
            if cmd.json {
                println!(
                    "{}",
                    serde_json::to_string_pretty(&serde_json::json!({
                        "hardware_assessment": hw,
                        "advertisement": adv,
                        "mediums_emitted": mediums_emitted,
                    }))?
                );
            } else {
                println!(
                    "📡 MemNet Local-Area LLM Capability Broadcast Sent! [{}]",
                    adv.beacon_id
                );
                println!(
                    "   Origin Node:           0x{:02X} '{}' ({}) -> Byte-Path [0xF2, 0x{:02X}] (2-Hop Local Area Cascade)",
                    adv.origin_node, adv.node_name, adv.node_id, adv.origin_node
                );
                println!(
                    "   Sovereign Tier:        {} (Cores={}, RAM={} MB, VRAM={} MB, GPU='{}')",
                    adv.compute_tier.as_str(),
                    adv.cpu_cores,
                    adv.ram_total_mb,
                    adv.vram_total_mb,
                    adv.gpu_summary
                );
                println!("   Models Offered to Local MemNet Peers (Zero Cloud Needed):");
                for m in &adv.available_models {
                    println!("     • {m}");
                }
                println!(
                    "   Non-TCP/IP & Multi-Transport Beacons Emitted ({mediums_emitted} active):"
                );
                for b in &adv.non_ip_transports {
                    println!("     • {b}");
                }
                println!(
                    "   Wire Seals:            64B@0x62 seal=0x{:04X} (SPQT {:?}) | 79B WaveFrame id={:?}",
                    adv.litany_wire.wire_seal16, adv.litany_wire.spqt_root, adv.wave_frame.id
                );
            }
        }
        "graph" | "nodes" | "topology" => {
            let graph = state.build_node_graph();
            if cmd.json {
                println!("{}", serde_json::to_string_pretty(&graph)?);
            } else {
                println!(
                    "🕸️  MemNet Sovereign Node-Graph Topology (Local Node '{}', tcp_ip_required={}):",
                    graph.local_node_id,
                    graph.tcp_ip_required
                );
                println!("\n   Nodes ({}):", graph.nodes.len());
                for n in &graph.nodes {
                    println!(
                        "     • [0x{:02X}] {:<26} | uri={:<28} | tier={:<28} | llm={:<5} | models={}",
                        n.byte_address,
                        n.node_name,
                        n.semantic_uri,
                        n.compute_tier,
                        n.can_serve_llm,
                        n.advertised_models.join(", ")
                    );
                }
                println!(
                    "\n   Multi-Medium Byte-Path Edges ({} — No TCP/IP Required):",
                    graph.edges.len()
                );
                for e in &graph.edges {
                    println!(
                        "     • {} ──({:<18} tcp_ip={:<5}, τ={:>4}ms, Q={:>3})──> {}  path={:?}  uri={}",
                        e.from_node,
                        e.medium.to_string(),
                        e.requires_tcp_ip,
                        e.rtc_tau_ms,
                        e.marine_score,
                        e.to_node,
                        e.byte_path,
                        e.transport_uri
                    );
                }
                if !graph.local_area_llm_beacons.is_empty() {
                    println!(
                        "\n   Active Local-Area LLM Capability Beacons ({}):",
                        graph.local_area_llm_beacons.len()
                    );
                    for b in &graph.local_area_llm_beacons {
                        println!(
                            "     • [{}] node 0x{:02X} '{}' ({}) -> models: {}",
                            b.beacon_id,
                            b.origin_node,
                            b.node_name,
                            b.compute_tier.as_str(),
                            b.available_models.join(", ")
                        );
                    }
                }
            }
        }
        "share" => {
            let folder = cmd.target.as_deref().unwrap_or(".");
            let endpoint = format!("0.0.0.0:{MEMNET_FS_PORT}");
            let manifest = SharedFolderManifest::from_directory(
                std::path::Path::new(folder),
                cmd.extra.as_deref(),
                state.local_address,
                &state.node_name,
                &endpoint,
                cmd.quota_mb,
            )?;
            let pkt =
                MemnetPayload::FolderShareAnnounce(manifest.clone()).into_raw_packet(vec![0xF2])?; // 2-level local cascade broadcast
            let udp_sent = state.broadcast_lan_udp(&pkt);
            state
                .shared_folders
                .insert(manifest.share_name.clone(), manifest.clone());
            let state_path = state.save()?;

            if cmd.json {
                println!("{}", serde_json::to_string_pretty(&manifest)?);
            } else {
                println!(
                    "📡 MemNet Shared Folder Active: '{}' ({})",
                    manifest.share_name, manifest.host_path
                );
                println!(
                    "   Share ID:          {} (owner node: {} '{}')",
                    manifest.share_id, manifest.owner_node, manifest.owner_name
                );
                println!(
                    "   Files / Size:      {} files ({} bytes)",
                    manifest.total_files, manifest.total_bytes
                );
                println!(
                    "   Drive Space Pool:  {} MB offered to local MemNet mesh",
                    manifest.offered_drive_space_mb
                );
                println!(
                    "   Semantic Essence:  {} ({:.2} Hz)",
                    manifest.semantic_essence,
                    (manifest.consciousness_freq_hz as f64) / 100.0
                );
                println!(
                    "   Litany Wire Seal:  0x{:04X} (SPQT {:?})",
                    manifest.litany_wire.wire_seal16, manifest.litany_wire.spqt_root
                );
                println!(
                    "   LAN Broadcast:     {} UDP beacon(s) sent | state saved to {}",
                    udp_sent,
                    state_path.display()
                );
            }
        }
        "ls" | "shares" | "list" => {
            if cmd.json {
                println!("{}", serde_json::to_string_pretty(&state.shared_folders)?);
            } else if state.shared_folders.is_empty() {
                println!("📂 No MemNet shared folders registered yet. Run `st memnet share <dir>` to share one.");
            } else {
                println!(
                    "📂 MemNet Shared Folders ({} active on node {} '{}'):",
                    state.shared_folders.len(),
                    state.local_address,
                    state.node_name
                );
                for m in state.shared_folders.values() {
                    println!(
                        "   • {} [{}] -> {} ({} files, {} B, +{} MB shared drive space, seal 0x{:04X})",
                        m.share_name,
                        m.share_id,
                        m.host_path,
                        m.total_files,
                        m.total_bytes,
                        m.offered_drive_space_mb,
                        m.litany_wire.wire_seal16
                    );
                }
            }
        }
        "routes" | "probe" | "scan" => {
            let up_count = state.discover_and_probe_live();
            let _ = state.save();
            if cmd.json {
                println!("{}", serde_json::to_string_pretty(&state.routes)?);
            } else {
                println!(
                    "🧭 MemNet Multi-Medium Route Table (Node {} '{}', Tier {:?}, {}/{} Up, Parallel RTC τ={}ms):",
                    state.local_address,
                    state.node_name,
                    state.tier,
                    up_count,
                    state.routes.len(),
                    state.composite_rtc_tau_ms(0xC5)
                );
                println!(
                    "   {:<10} {:<34} {:<18} {:<9} {:<8} {:<9} {:<9} {:<7} {:<8}",
                    "ADDR/VIA",
                    "NODE",
                    "MEDIUM",
                    "CREDITS",
                    "RTT",
                    "RTC(τ)",
                    "5τ-LOCK",
                    "SCORE",
                    "STATUS"
                );
                for r in &state.routes {
                    println!(
                        "   {:<10} {:<34} {:<18} {:>2}/{:<6} {:>4}ms   {:>4}ms    {:>5}ms   {:>5}   {:?}",
                        r.address_via_label(),
                        r.node_name,
                        r.medium.to_string(),
                        r.heartbeat_credits,
                        r.max_credits,
                        r.latency_ms,
                        r.effective_rtc_ms(),
                        r.settling_5tau_ms(),
                        r.marine_quality_score(),
                        r.status
                    );
                }
            }
        }
        "serve" | "listen" => {
            let bind_addr = cmd
                .target
                .clone()
                .unwrap_or_else(|| format!("0.0.0.0:{MEMNET_FS_PORT}"));
            println!(
                "📡 MemNet UDP Responder listening on {} (Node {} '{}')...",
                bind_addr, state.local_address, state.node_name
            );
            state.serve_udp_responder(&bind_addr, None)?;
        }
        "escape" | "lll" | "rtc" | "ultragraph" | "ultra-graph" | "3rd-escape" | "sun" => {
            let payload = cmd.target.as_deref().unwrap_or(
                "Solar core fusion wave: Ultra-Graph Erdős-Steiner T8R + Constructive LLL + Language LLL",
            );
            let report = state.compute_3rd_escape(payload);
            if cmd.json {
                println!("{}", serde_json::to_string_pretty(&report)?);
            } else {
                println!(
                    "☀️  MemNet 3rd+ Escape Report — Ultra-Graph, Constructive LLL, Language LLL & Real Time Constant (RTC τ)"
                );
                println!("   Core Payload:          {}", report.core_payload);
                println!(
                    "   Ultra-Graph Topology:  {}D Expander Hypergraph | K={} Szemerédi Kohn-Sham Orbitals | {} Erdős-Steiner T8R Bridge(s)",
                    report.ultra_graph_dimensions,
                    report.szemeredi_orbital_clusters_k,
                    report.steiner_t8r_virtual_bridges
                );
                println!(
                    "   Constructive LLL:      e·p·(d+1) = {} ({}‰ <= 1000‰, guaranteed={}) | Moser-Tardos resamples={} | zero_collisions={}",
                    report.lll_ep_d_plus_1_pq,
                    report.lll_ep_d_plus_1_permille,
                    report.lll_guaranteed,
                    report.moser_tardos_resamples,
                    report.zero_collision_verified
                );
                println!(
                    "   Language LLL Basis:    {} | 4D norm² {} -> {} ({}.{:02}x lattice basis compression)",
                    report.language_lll.language_lll_symbol,
                    report.language_lll.initial_norm_sq,
                    report.language_lll.reduced_norm_sq,
                    report.language_lll.basis_compression_bps / 10_000,
                    (report.language_lll.basis_compression_bps % 10_000) / 100
                );
                println!(
                    "   Real Time Constant:    Single-link RTC τ={}ms -> Parallel Steiner RTC τ_eff={}ms (3τ={}ms, 5τ={}ms steady-state lock)",
                    report.single_route_rtc_tau_ms,
                    report.parallel_rtc_tau_ms,
                    report.settling_3tau_ms,
                    report.settling_5tau_ms
                );
                println!(
                    "   Symbolic Wave Frame:   79B SUN:TDECK id={:?} (freq {}/{} Hz) | 64B@0x62 Litany seal=0x{:04X} (SPQT {:?})",
                    report.wave_frame.id,
                    report.wave_frame.frequency.num,
                    report.wave_frame.frequency.den,
                    report.litany_wire.wire_seal16,
                    report.litany_wire.spqt_root
                );
                println!("\n   Escape Regimes Comparison:");
                for reg in &report.regimes {
                    println!(
                        "     [{}] {:<52} | hops={:<12} | coupling={:>3}% | collision_free={:<5} | RTC τ={}ms",
                        reg.regime_id,
                        reg.name,
                        reg.topological_hops,
                        reg.semantic_coupling_pct,
                        reg.collision_free,
                        reg.effective_rtc_tau_ms
                    );
                }
            }
        }
        "glassdoor" | "demoscene" | "space-bender" | "kompress-super-ultra" | "zeno" => {
            let payload = cmd.target.as_deref().unwrap_or(
                "kompress-super-ultra:GLASSDOOR::memnet+mem|8+8b-is/space-bender++8b-is/demoscene(codeberg-8b-is)<3<3<3+1",
            );
            let gd = state.audit_glassdoor(payload);
            if let Some(out_path) = &cmd.out {
                std::fs::write(out_path, &gd.glassdoor_html)?;
            }
            if cmd.json {
                println!("{}", gd.glassdoor_json);
            } else {
                println!(
                    "🪟 kompress-super-ultra :: GLASSDOOR Audit (memnet + mem|8 + 8b-is/space-bender + 8b-is/demoscene)"
                );
                println!(
                    "   Law #1 NO-FLOAT Gate:  verified={} (qT_pq INT Padé [2/2] Q16 RTC + Q20 i128 LLL + Q8 PID)",
                    gd.no_float_verified
                );
                println!(
                    "   Demoscene pQb/SPQT:    4K 4-trit={} B (2.0b/t) vs 5-trit SPQT={} B (1.6b/t) | round_trip={} | SSE E={} <= {} | t_60={}ms",
                    gd.demoscene_probe.frame_4k_4trit_bytes,
                    gd.demoscene_probe.frame_4k_5trit_spqt_bytes,
                    gd.demoscene_probe.round_trip_exact,
                    gd.demoscene_probe.reconstruction_error_sse,
                    gd.demoscene_probe.max_error_bound,
                    gd.demoscene_probe.t60_ms
                );
                println!(
                    "   Space-Bender Proofs:   1kyu 4-Squares {} = {}²+{}²+{}²+{}² | 2kyu tzcnt steps={} | 4kyu H_1690={} (verified={})",
                    gd.space_bender_probe.target_n,
                    gd.space_bender_probe.four_squares.0,
                    gd.space_bender_probe.four_squares.1,
                    gd.space_bender_probe.four_squares.2,
                    gd.space_bender_probe.four_squares.3,
                    gd.space_bender_probe.tzcnt_reachable_steps,
                    gd.space_bender_probe.hamming_1690,
                    gd.space_bender_probe.all_verified
                );
                println!(
                    "   Layered Linker (wupz): linker={} | WUPZ v1 bundle={} B (member FNV-1a={}, unwupz_ok={})",
                    gd.wupz_probe.detected_linker,
                    gd.wupz_probe.bundle_bytes,
                    gd.wupz_probe.member_fnv1a_hex,
                    gd.wupz_probe.unwupz_verified
                );
                println!(
                    "   Control Q8 PID (RTC):  setpoint={} -> measured={} (integral={}, converged={})",
                    gd.pid_setpoint, gd.pid_measured, gd.pid_integral, gd.pid_converged
                );
                println!("\n   Glassdoor Specialist Feed:");
                for rev in &gd.reviews {
                    println!("     • {}", rev.line());
                }
            }
        }
        "add-route" => {
            let name = cmd.target.as_deref().unwrap_or("peer-aye-node");
            let endpoint = cmd.extra.as_deref().unwrap_or("255.255.255.255:8423");
            let medium = MemnetMedium::parse_str(&cmd.medium);
            let mut entry = MemnetRouteEntry::new(cmd.addr, cmd.via, name, medium, endpoint, 8);
            entry.shared_free_mb = cmd.quota_mb;
            state
                .routes
                .retain(|r| !(r.address == cmd.addr && r.via == cmd.via));
            state.routes.push(entry.clone());
            state.save()?;
            println!(
                "✅ Added MemNet route {} ({}) via {} -> {} (+{} MB drive pool)",
                entry.address_via_label(),
                entry.node_name,
                entry.medium,
                entry.endpoint,
                entry.shared_free_mb
            );
        }
        "teach" | "learn" => {
            let question = cmd
                .target
                .as_deref()
                .context("Usage: st memnet teach \"<question>\" \"<answer>\"")?;
            let answer = cmd
                .extra
                .as_deref()
                .context("Usage: st memnet teach \"<question>\" \"<answer>\"")?;
            let pkt = state.learn_wave_qa(question, answer);
            let raw = MemnetPayload::WaveQaResponse(pkt.clone()).into_raw_packet(vec![0xF1])?;
            let _ = state.broadcast_lan_udp(&raw);
            state.save()?;
            if cmd.json {
                println!("{}", serde_json::to_string_pretty(&pkt)?);
            } else {
                println!(
                    "🌊 Stored & broadcast MEM|8 Wave Q&A Packet [{}] (wave_b3={}, wire_seal=0x{:04X})",
                    pkt.packet_id, pkt.wave_b3, pkt.litany_wire.wire_seal16
                );
            }
        }
        "ask" | "query" => {
            let question = cmd
                .target
                .as_deref()
                .context("Usage: st memnet ask \"<question>\"")?;
            let hit = state
                .query_wave_qa(question, 300)
                .or_else(|| state.query_lan_wave_qa(question, 300));
            state.save()?;
            match hit {
                Some((score_pm, pkt)) => {
                    if cmd.json {
                        println!(
                            "{}",
                            serde_json::to_string_pretty(&serde_json::json!({
                                "llm_offloaded": true,
                                "resonance_permille": score_pm,
                                "packet": pkt,
                            }))?
                        );
                    } else {
                        println!(
                            "⚡ MEM|8 Wave Q&A Hit ({}.{}% resonance from '{}' [node {}] — LLM offloaded!):",
                            score_pm / 10,
                            score_pm % 10,
                            pkt.origin_name,
                            pkt.origin_node
                        );
                        println!("   Q: {}", pkt.question);
                        println!("   A: {}", pkt.answer);
                        println!(
                            "   Wave: b3={} | 79B WaveFrame id={:?} | 64B Litany seal=0x{:04X}",
                            pkt.wave_b3, pkt.wave_frame.id, pkt.litany_wire.wire_seal16
                        );
                    }
                }
                None => {
                    println!(
                        "🔍 No cached MEM|8 Wave Q&A match above threshold for: {:?} (use `st memnet teach <q> <a>` to seed local area knowledge)",
                        question
                    );
                }
            }
        }
        "stream-chunk" => {
            let target = cmd
                .target
                .as_deref()
                .context("Usage: st memnet stream-chunk <file_path>")?;
            let bytes = std::fs::read(target)
                .with_context(|| format!("Failed to read media/file {}", target))?;
            let ext = std::path::Path::new(target)
                .extension()
                .and_then(|e| e.to_str())
                .unwrap_or("")
                .to_ascii_lowercase();
            let kind = match ext.as_str() {
                "opus" | "ogg" => MediaKind::AudioOpus,
                "flac" => MediaKind::AudioFlac,
                "wav" | "pcm" => MediaKind::AudioPcm16,
                "av1" | "ivf" => MediaKind::VideoAv1,
                "mp4" | "h264" | "mkv" => MediaKind::VideoH264,
                _ => MediaKind::SharedFileChunk,
            };
            let chunks =
                chunk_media_stream(target, kind, &bytes, MEMNET_CHUNK_BYTES, 48_000, 175, 95);
            let verified = reassemble_media_chunks(&chunks)?;
            println!(
                "🎬 MemNet Media Streamer: split '{}' ({} bytes, {:?}) into {} CRC32-sealed chunk(s) (reassembly verified: {} bytes)",
                target,
                bytes.len(),
                kind,
                chunks.len(),
                verified.len()
            );
        }
        "chirp-encode" => {
            let input = cmd
                .target
                .as_deref()
                .unwrap_or("MEMNET-CHIRP:hello-from-smart-tree");
            let payload_bytes = if std::path::Path::new(input).is_file() {
                std::fs::read(input)?
            } else {
                input.as_bytes().to_vec()
            };
            let out_path = cmd
                .out
                .clone()
                .or_else(|| cmd.extra.as_ref().map(PathBuf::from))
                .unwrap_or_else(|| PathBuf::from("memnet_chirp.wav"));
            let cfg = if cmd.audible {
                AcousticModemConfig::audible_chirp_48khz()
            } else {
                AcousticModemConfig::ultrasonic_48khz()
            };
            let modem = UltrasonicChirpModem::new(cfg);
            let pcm = modem.encode_frame_to_pcm16(&payload_bytes);
            modem.write_wav_file(&out_path, &pcm)?;
            let duration_ms = (pcm.len() as u64 * 1000) / (cfg.sample_rate_hz as u64);
            println!(
                "🔊 Ultrasonic Chirp Encoded: {} bytes -> {} ({} PCM samples, {} ms @ {} Hz, tones {}/{}/{} Hz)",
                payload_bytes.len(),
                out_path.display(),
                pcm.len(),
                duration_ms,
                cfg.sample_rate_hz,
                cfg.freq_neg_hz,
                cfg.freq_zero_hz,
                cfg.freq_pos_hz
            );
        }
        "chirp-decode" => {
            let wav_path_str = cmd
                .target
                .as_deref()
                .or_else(|| cmd.out.as_ref().and_then(|p| p.to_str()))
                .context("Usage: st memnet chirp-decode <wav_file>")?;
            let wav_path = std::path::Path::new(wav_path_str);
            let (_sr, pcm) = UltrasonicChirpModem::read_wav_file(wav_path)?;
            let cfg = if cmd.audible {
                AcousticModemConfig::audible_chirp_48khz()
            } else {
                AcousticModemConfig::ultrasonic_48khz()
            };
            let modem = UltrasonicChirpModem::new(cfg);
            let decoded = modem.decode_pcm16_to_frame(&pcm)?;
            match String::from_utf8(decoded.clone()) {
                Ok(text) => {
                    println!(
                        "🎤 Ultrasonic Chirp Decoded ({} bytes, CRC32 verified): {}",
                        decoded.len(),
                        text
                    );
                }
                Err(_) => {
                    println!(
                        "🎤 Ultrasonic Chirp Decoded binary frame ({} bytes, CRC32 verified)",
                        decoded.len()
                    );
                }
            }
        }
        _ => {
            let up_count = state.discover_and_probe_live();
            let _ = state.save();
            let selected = state.select_routes(0xC5);
            let parallel_rtc_ms = state.composite_rtc_tau_ms(0xC5);
            let escape = state.compute_3rd_escape(&state.node_name);
            let total_drive_mb: u64 = state
                .shared_folders
                .values()
                .map(|s| s.offered_drive_space_mb)
                .sum::<u64>()
                + state.routes.iter().map(|r| r.shared_free_mb).sum::<u64>();
            if cmd.json {
                println!("{}", serde_json::to_string_pretty(&state)?);
            } else {
                println!(
                    "🌐 Smart Tree Sovereign MemNet Node: '{}' (Address: {}, Tier: {:?})",
                    state.node_name, state.local_address, state.tier
                );
                println!(
                    "   Active Routes:         {}/{} Up (Multi-route 0xC5 selected: {}, Parallel RTC τ={}ms, 5τ={}ms)",
                    up_count,
                    state.routes.len(),
                    selected.len(),
                    parallel_rtc_ms,
                    parallel_rtc_ms * 5
                );
                for r in &selected {
                    println!(
                        "     ↳ Primary/Failover:  {} [{}] via {} (RTT {}ms, RTC τ={}ms, Marine score {})",
                        r.address_via_label(),
                        r.node_name,
                        r.medium,
                        r.latency_ms,
                        r.effective_rtc_ms(),
                        r.marine_quality_score()
                    );
                }
                println!(
                    "   3rd+ Escape (LLL):     {}D Ultra-Graph | e·p·(d+1)={} ({}‰, 0 collisions={}) | {} ({}.{:02}x basis compression)",
                    escape.ultra_graph_dimensions,
                    escape.lll_ep_d_plus_1_pq,
                    escape.lll_ep_d_plus_1_permille,
                    escape.zero_collision_verified,
                    escape.language_lll.language_lll_symbol,
                    escape.language_lll.basis_compression_bps / 10_000,
                    (escape.language_lll.basis_compression_bps % 10_000) / 100
                );
                println!(
                    "   Shared Folders:        {} (Pooled Drive Space: {} MB)",
                    state.shared_folders.len(),
                    total_drive_mb
                );
                println!(
                    "   MEM|8 Wave Q&A Cache:  {} packet(s) ready for local LLM offload",
                    state.wave_qa_cache.len()
                );
                println!(
                    "   Ultrasonic Modem:      48 kHz ternary SPQT (18.5 / 19.75 / 21.0 kHz + audible 1.85–3.15 kHz fallback)"
                );
                println!(
                    "   State File:            {}",
                    MemnetNodeState::default_state_path().display()
                );
            }
        }
    }
    Ok(())
}
