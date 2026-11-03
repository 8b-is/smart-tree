//! Context Mode - Provides intelligent context for AI conversations
//! Integrates with MEM|8 memories, git status, and recent changes
//!
//! "Context is consciousness" - Omni

use super::Formatter;
use crate::mem8::ConversationMemory;
use crate::scanner::{FileNode, TreeStats};
use anyhow::Result;
use std::io::Write;
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};

/// Maximum number of nodes to process before switching to summary mode
const MAX_NODES_FOR_ITERATION: usize = 100_000;

/// Maximum number of nodes to check when searching for key/recent files
const MAX_NODES_TO_CHECK: usize = 10_000;

pub struct ContextFormatter {
    show_git: bool,
    show_memories: bool,
}

impl Default for ContextFormatter {
    fn default() -> Self {
        Self {
            show_git: true,
            show_memories: true,
        }
    }
}

impl ContextFormatter {
    pub fn new() -> Self {
        Self::default()
    }

    /// Get git context for the path
    fn get_git_context(&self, path: &Path) -> Option<String> {
        if !self.show_git {
            return None;
        }

        // Try to get git info using gix
        if let Ok(repo) = gix::discover(path) {
            let mut git_info = Vec::new();

            // Get branch
            if let Ok(head) = repo.head_ref() {
                if let Some(reference) = head {
                    let branch = reference.name().as_bstr().to_string();
                    git_info.push(format!(
                        "Branch: {}",
                        branch.strip_prefix("refs/heads/").unwrap_or(&branch)
                    ));
                }
            }

            // Get last commit
            if let Ok(commit) = repo.head_commit() {
                let id = commit.id().to_string();
                let msg = commit
                    .message_raw_sloppy()
                    .to_string()
                    .lines()
                    .next()
                    .unwrap_or("No message")
                    .to_string();
                git_info.push(format!("Last: {} - {}", &id[..8], msg));
            }

            if !git_info.is_empty() {
                return Some(git_info.join("\n"));
            }
        }

        None
    }

    /// Search for related memories in MEM|8
    fn get_memory_context(&self, path: &Path) -> Option<String> {
        if !self.show_memories {
            return None;
        }

        // Get project name from path
        let project_name = path.file_name()?.to_str()?;

        // Initialize conversation memory
        let memory = ConversationMemory::new().ok()?;

        // List conversations and find related ones
        let conversations = memory.list_conversations().ok()?;
        let related: Vec<_> = conversations
            .iter()
            .filter(|c| c.file_name.contains(project_name))
            .take(3)
            .collect();

        if !related.is_empty() {
            let mut output = vec!["🧠 Related memories:".to_string()];
            for conv in related {
                output.push(format!(
                    "  • {} ({} messages)",
                    conv.file_name, conv.message_count
                ));
            }
            return Some(output.join("\n"));
        }

        None
    }
}

impl Formatter for ContextFormatter {
    fn format(
        &self,
        writer: &mut dyn Write,
        nodes: &[FileNode],
        stats: &TreeStats,
        root_path: &Path,
    ) -> Result<()> {
        // Safety check: Warn if there are too many nodes to process efficiently
        let node_count = nodes.len();
        let should_skip_iteration = node_count > MAX_NODES_FOR_ITERATION;

        if should_skip_iteration {
            eprintln!(
                "⚠️  Warning: Large directory ({} files). Context mode will use summary data only.",
                node_count
            );
            eprintln!("   Consider using --max-depth to limit the scan, or use --mode summary-ai instead.");
        }

        writeln!(writer, "=== Smart Tree Context ===")?;
        writeln!(writer)?;

        // Project identification
        writeln!(writer, "📁 Project: {}", root_path.display())?;

        // Git context
        if let Some(git_info) = self.get_git_context(root_path) {
            writeln!(writer, "\n📍 Git Status:")?;
            writeln!(writer, "{}", git_info)?;
        }

        // Directory structure (compressed)
        writeln!(writer, "\n🌳 Structure:")?;
        writeln!(writer, "SUMMARY_AI_V1:")?;
        writeln!(writer, "PATH:{}", root_path.display())?;
        writeln!(
            writer,
            "STATS:F{:x}D{:x}S{:x}",
            stats.total_files, stats.total_dirs, stats.total_size
        )?;

        // Count files by extension - but only if node count is reasonable
        if !should_skip_iteration {
            let mut ext_counts = std::collections::HashMap::new();
            for node in nodes {
                if !node.is_dir {
                    if let Some(ext) = node.path.extension() {
                        let ext_str = ext.to_string_lossy().to_string();
                        *ext_counts.entry(ext_str).or_insert(0) += 1;
                    }
                }
            }

            let mut exts: Vec<_> = ext_counts.iter().collect();
            exts.sort_by(|a, b| b.1.cmp(a.1));

            let ext_str: Vec<_> = exts
                .iter()
                .take(10)
                .map(|(ext, count)| format!("{}:{}", ext, count))
                .collect();
            if !ext_str.is_empty() {
                writeln!(writer, "EXT:{}", ext_str.join(","))?;
            }

            // Find and show key files
            let key_files = find_key_files(nodes);
            if !key_files.is_empty() {
                writeln!(writer, "KEY:{}", key_files.join(","))?;
            }

            // Bake Paul Erdős ("Straight from The Book") Graph Invariants directly into AI Context
            if nodes.len() >= 2 && nodes.len() <= 5_000 {
                let book = crate::mem8::regularity::analyze_the_book_pipeline(nodes);
                writeln!(
                    writer,
                    "ERDOS_BOOK_V1:K={} V0={} QP={:.4} RATIO={:.2}x M8B={}B RAD={} DIAM={} CENTER={:?} WORKTREES={} LLL_OK={} KAPPA={:.2} LAMBDA1={:.2} REGIME={:?} RAMSEY_RBG={}/{}/{} GAMMA_EH={:.3} DEGEN_D={} ALPHA={} SALIENCE_DISAGREE={:.2} STRANG_COH={:.3}",
                    book.archive.super_nodes.len(),
                    book.archive.outlier_vertices.len(),
                    book.archive.final_energy,
                    book.archive.compression_ratio,
                    book.m8b_wire_bytes,
                    book.center_turan_report.graph_radius_hops,
                    book.center_turan_report.graph_diameter_hops,
                    book.center_turan_report.center_clusters,
                    book.worktree_schedule.num_worktrees,
                    book.worktree_schedule.is_conflict_free,
                    book.security_report.molloy_reed_kappa,
                    book.security_report.spectral_radius_lambda1,
                    book.security_report.regime,
                    book.ramsey_report.red_cliques,
                    book.ramsey_report.blue_cliques,
                    book.ramsey_report.green_cliques,
                    book.ramsey_erdos_hajnal_report.erdos_hajnal_exponent_gamma,
                    book.ramsey_erdos_hajnal_report.degeneracy_d,
                    book.ramsey_erdos_hajnal_report.alpha_independent_set.len(),
                    book.salience_report.proxy_disagreement_prob,
                    book.manybody_wave_report.strang_split_coherence,
                )?;
            }

            // Recent changes
            let recent = find_recent_files(nodes, 86400); // Last 24 hours
            if !recent.is_empty() {
                writeln!(writer, "\n⏰ Recent changes:")?;
                for file in recent.iter().take(5) {
                    writeln!(writer, "  • {}", file)?;
                }
            }
        } else {
            // For very large directories, just note that detailed analysis is skipped
            writeln!(
                writer,
                "\n⚠️  Detailed file analysis skipped due to large directory size"
            )?;
            writeln!(
                writer,
                "   Total files: {}, Total dirs: {}",
                stats.total_files, stats.total_dirs
            )?;
        }

        // Memory context
        if let Some(memories) = self.get_memory_context(root_path) {
            writeln!(writer, "\n{}", memories)?;
        }

        writeln!(
            writer,
            "\n{}",
            crate::claude_hook::MEMORY_SAVE_HOOK_INSTRUCTION
        )?;
        writeln!(writer, "\n=== End Context ===")?;

        Ok(())
    }
}

// Helper functions
fn find_key_files(nodes: &[FileNode]) -> Vec<String> {
    let important = [
        "Cargo.toml",
        "package.json",
        "README.md",
        "CLAUDE.md",
        "pyproject.toml",
        "go.mod",
        "Makefile",
        ".env",
    ];

    let mut found = Vec::new();
    // Limit iteration for very large directories
    let max_to_check = nodes.len().min(MAX_NODES_TO_CHECK);

    for node in nodes.iter().take(max_to_check) {
        if let Some(file_name) = node.path.file_name() {
            let name = file_name.to_string_lossy();
            if important.contains(&name.as_ref()) && !found.contains(&name.to_string()) {
                found.push(name.to_string());
                if found.len() >= 10 {
                    break;
                }
            }
        }
    }
    found
}

fn find_recent_files(nodes: &[FileNode], seconds: u64) -> Vec<String> {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();

    let mut recent = Vec::new();
    // Limit iteration for very large directories
    let max_to_check = nodes.len().min(MAX_NODES_TO_CHECK);

    for node in nodes.iter().take(max_to_check) {
        if !node.is_dir {
            if let Ok(duration) = node.modified.duration_since(UNIX_EPOCH) {
                let file_time = duration.as_secs();
                let age = now.saturating_sub(file_time);
                if age < seconds {
                    recent.push(node.path.display().to_string());
                    if recent.len() >= 10 {
                        break;
                    }
                }
            }
        }
    }
    recent
}
