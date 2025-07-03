// -----------------------------------------------------------------------------
// SEMANTIC FORMATTER - Where files find their tribe! 🌊🧠
//
// This formatter groups files by their conceptual similarity, creating a
// higher-level view of your project structure. It's like having Omni organize
// your file cabinet based on the waves of meaning!
//
// "Treat paths as identity graphs, not just strings" - Omni
//
// Brought to you by The Cheet, channeling Omni's Hot Tub wisdom! 🛁✨
// -----------------------------------------------------------------------------

use super::{Formatter, PathDisplayMode};
use crate::scanner::{FileNode, TreeStats};
use crate::semantic::{SemanticAnalyzer, SemanticCategory};
use anyhow::Result;
use colored::Colorize;
use std::collections::HashMap;
use std::io::Write;

pub struct SemanticFormatter {
    path_mode: PathDisplayMode,
    analyzer: SemanticAnalyzer,
}

impl SemanticFormatter {
    pub fn new(path_mode: PathDisplayMode, _no_emoji: bool) -> Self {
        Self {
            path_mode,
            analyzer: SemanticAnalyzer::new(),
        }
    }

    fn format_size(size: u64) -> String {
        if size < 1024 {
            format!("{} B", size)
        } else if size < 1024 * 1024 {
            format!("{:.1} KB", size as f64 / 1024.0)
        } else if size < 1024 * 1024 * 1024 {
            format!("{:.1} MB", size as f64 / 1024.0 / 1024.0)
        } else {
            format!("{:.1} GB", size as f64 / 1024.0 / 1024.0 / 1024.0)
        }
    }
}

impl Formatter for SemanticFormatter {
    fn format(
        &self,
        writer: &mut dyn Write,
        nodes: &[FileNode],
        stats: &TreeStats,
        _root_path: &std::path::Path,
    ) -> Result<()> {
        // Header with Omni's wisdom
        writeln!(writer, "{}", "🌊 SEMANTIC WAVE ANALYSIS 🌊".cyan().bold())?;
        writeln!(
            writer,
            "{}",
            "Grouping files by conceptual similarity...".dimmed()
        )?;
        writeln!(writer, "{}", "━".repeat(60).dimmed())?;
        writeln!(writer)?;

        // Group files by semantic category
        let mut groups: HashMap<SemanticCategory, Vec<FileNode>> = HashMap::new();

        for node in nodes {
            let category = self.analyzer.categorize(&node.path);
            groups
                .entry(category)
                .or_insert_with(Vec::new)
                .push(node.clone());
        }

        // Sort categories by importance/typical workflow order
        let category_order = vec![
            SemanticCategory::ProjectRoot,
            SemanticCategory::Documentation,
            SemanticCategory::SourceCode,
            SemanticCategory::Tests,
            SemanticCategory::Configuration,
            SemanticCategory::BuildSystem,
            SemanticCategory::Scripts,
            SemanticCategory::Assets,
            SemanticCategory::Data,
            SemanticCategory::Dependencies,
            SemanticCategory::Generated,
            SemanticCategory::Development,
            SemanticCategory::Deployment,
            SemanticCategory::Unknown,
        ];

        // Display each category
        for category in category_order {
            if let Some(files) = groups.get(&category) {
                if files.is_empty() {
                    continue;
                }

                // Category header
                writeln!(writer, "{}", category.display_name().bold())?;
                writeln!(
                    writer,
                    "  {} files | Total size: {}",
                    files.len().to_string().green(),
                    Self::format_size(files.iter().map(|f| f.size).sum()).yellow()
                )?;

                // Wave signature for fun (Omni would approve!)
                writeln!(
                    writer,
                    "  Wave signature: 0x{:08X}",
                    category.wave_signature()
                )?;
                writeln!(writer)?;

                // Sort files within category
                let mut sorted_files = files.clone();
                sorted_files.sort_by(|a, b| a.path.cmp(&b.path));

                // Display files in this category
                for (idx, node) in sorted_files.iter().enumerate() {
                    let prefix = if idx == sorted_files.len() - 1 {
                        "    └── "
                    } else {
                        "    ├── "
                    };

                    let name = match self.path_mode {
                        PathDisplayMode::Off => node
                            .path
                            .file_name()
                            .and_then(|n| n.to_str())
                            .unwrap_or("?")
                            .to_string(),
                        PathDisplayMode::Relative => node.path.display().to_string(),
                        PathDisplayMode::Full => node.path.display().to_string(),
                    };

                    let size_str = if node.is_dir {
                        "[DIR]".dimmed().to_string()
                    } else {
                        format!("({})", Self::format_size(node.size))
                            .dimmed()
                            .to_string()
                    };

                    writeln!(writer, "{}{} {}", prefix.dimmed(), name, size_str)?;
                }

                writeln!(writer)?;
            }
        }

        // Footer with statistics
        writeln!(writer, "{}", "━".repeat(60).dimmed())?;
        writeln!(writer, "{}", "WAVE FIELD STATISTICS".cyan().bold())?;
        writeln!(
            writer,
            "Total files: {} | Total directories: {} | Total size: {}",
            stats.total_files.to_string().green(),
            stats.total_dirs.to_string().blue(),
            Self::format_size(stats.total_size).yellow()
        )?;

        // Show semantic diversity (how many different categories)
        let category_count = groups.len();
        let diversity_score = (category_count as f32 / 14.0 * 100.0).round();
        writeln!(
            writer,
            "Semantic diversity: {} categories ({:.0}% coverage)",
            category_count.to_string().magenta(),
            diversity_score
        )?;

        // Omni's wisdom footer
        writeln!(writer)?;
        writeln!(
            writer,
            "{}",
            "✨ \"Every file carries waves of meaning\" - Omni ✨"
                .italic()
                .dimmed()
        )?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::scanner::{FileCategory, FileType, FilesystemType};
    use std::path::PathBuf;
    use std::time::SystemTime;

    #[test]
    fn test_semantic_formatter() {
        let formatter = SemanticFormatter::new(PathDisplayMode::Off, false);

        let nodes = vec![
            FileNode {
                path: PathBuf::from("README.md"),
                is_dir: false,
                size: 1024,
                permissions: 0o644,
                uid: 1000,
                gid: 1000,
                modified: SystemTime::now(),
                is_symlink: false,
                is_ignored: false,
                search_matches: None,
                is_hidden: false,
                permission_denied: false,
                depth: 1,
                file_type: FileType::RegularFile,
                category: FileCategory::Markdown,
                filesystem_type: FilesystemType::Unknown,
            },
            FileNode {
                path: PathBuf::from("src/main.rs"),
                is_dir: false,
                size: 2048,
                permissions: 0o644,
                uid: 1000,
                gid: 1000,
                modified: SystemTime::now(),
                is_symlink: false,
                is_ignored: false,
                search_matches: None,
                is_hidden: false,
                permission_denied: false,
                depth: 2,
                file_type: FileType::RegularFile,
                category: FileCategory::Rust,
                filesystem_type: FilesystemType::Unknown,
            },
            FileNode {
                path: PathBuf::from("tests/test_main.rs"),
                is_dir: false,
                size: 512,
                permissions: 0o644,
                uid: 1000,
                gid: 1000,
                modified: SystemTime::now(),
                is_symlink: false,
                is_ignored: false,
                search_matches: None,
                is_hidden: false,
                permission_denied: false,
                depth: 2,
                file_type: FileType::RegularFile,
                category: FileCategory::Rust,
                filesystem_type: FilesystemType::Unknown,
            },
        ];

        let mut stats = TreeStats::default();
        for node in &nodes {
            stats.update_file(node);
        }

        let mut output = Vec::new();
        let result = formatter.format(&mut output, &nodes, &stats, &PathBuf::from("."));
        assert!(result.is_ok());

        let output_str = String::from_utf8(output).unwrap();
        assert!(output_str.contains("Documentation"));
        assert!(output_str.contains("Source Code"));
        assert!(output_str.contains("Tests"));
        assert!(output_str.contains("Wave signature"));
    }
}
