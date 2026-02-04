//
// -----------------------------------------------------------------------------
// 🗑️ WASTE DETECTION FORMATTER - The Marie Kondo of Code! ✨
//
// Hey there, Hue! This is our brilliant waste detection system that you dreamed up!
// It analyzes projects for duplicate files, bloated dependencies, forgotten build
// artifacts, and suggests optimizations that would make Trisha in Accounting
// do a happy dance! 💃
//
// This formatter is like having a personal organizer for your codebase - it finds
// all the clutter and tells you exactly how to clean it up. Elvis would be proud
// of this rock-solid optimization! 🎸
//
// Brought to you by Hue & Aye - making codebases lean and mean! 🚀
// -----------------------------------------------------------------------------

use super::Formatter;
use crate::scanner::{FileNode, TreeStats};
use anyhow::Result;
use humansize::{format_size, BINARY};
use std::collections::HashMap;
use std::io::Write;
use std::path::Path;

/// The WasteFormatter - Your personal codebase cleanup consultant! 🧹
pub struct WasteFormatter {
    /// Show detailed suggestions for cleanup
    pub show_suggestions: bool,
    /// Minimum file size to consider for large file analysis (default: 10MB)
    pub large_file_threshold: u64,
    /// Maximum number of duplicates to show per group
    pub max_duplicates_shown: usize,
}

impl Default for WasteFormatter {
    fn default() -> Self {
        Self::new()
    }
}

impl WasteFormatter {
    pub fn new() -> Self {
        Self {
            show_suggestions: true,
            large_file_threshold: 10 * 1024 * 1024, // 10MB
            max_duplicates_shown: 5,
        }
    }

    pub fn with_threshold(mut self, threshold: u64) -> Self {
        self.large_file_threshold = threshold;
        self
    }

    pub fn with_suggestions(mut self, show: bool) -> Self {
        self.show_suggestions = show;
        self
    }

    /// Analyze files for potential duplicates based on size and name patterns
    fn analyze_duplicates<'a>(&self, nodes: &'a [FileNode]) -> HashMap<u64, Vec<&'a FileNode>> {
        let mut size_groups: HashMap<u64, Vec<&FileNode>> = HashMap::new();

        for node in nodes {
            if !node.is_dir && node.size > 0 && !node.permission_denied {
                size_groups.entry(node.size).or_default().push(node);
            }
        }

        // Only keep groups with multiple files of the same size
        size_groups.retain(|_, files| files.len() > 1);
        size_groups
    }

    /// Detect common build artifacts and temporary files
    fn analyze_build_artifacts<'a>(&self, nodes: &'a [FileNode]) -> Vec<&'a FileNode> {
        let build_patterns = [
            "node_modules",
            "target",
            "build",
            "dist",
            ".next",
            ".nuxt",
            ".svelte-kit",
            "__pycache__",
            ".pytest_cache",
            "coverage",
            ".coverage",
            ".nyc_output",
            "logs",
            "*.log",
            ".DS_Store",
            "Thumbs.db",
            "*.tmp",
            "*.temp",
            ".cache",
            ".parcel-cache",
        ];

        nodes
            .iter()
            .filter(|node| {
                let path_str = node.path.to_string_lossy().to_lowercase();
                build_patterns.iter().any(|pattern| {
                    if pattern.contains('*') {
                        // Simple wildcard matching
                        let pattern = pattern.replace('*', "");
                        path_str.contains(&pattern)
                    } else {
                        path_str.contains(pattern)
                    }
                })
            })
            .collect()
    }

    /// Find large files that might be candidates for optimization
    fn analyze_large_files<'a>(&self, nodes: &'a [FileNode]) -> Vec<&'a FileNode> {
        let mut large_files: Vec<&FileNode> = nodes
            .iter()
            .filter(|node| !node.is_dir && node.size >= self.large_file_threshold)
            .collect();

        large_files.sort_by(|a, b| b.size.cmp(&a.size));
        large_files
    }

    /// Detect dependency-related waste (package managers)
    fn analyze_dependency_waste<'a>(
        &self,
        nodes: &'a [FileNode],
    ) -> HashMap<String, Vec<&'a FileNode>> {
        let mut dependency_groups: HashMap<String, Vec<&FileNode>> = HashMap::new();

        for node in nodes {
            let path_str = node.path.to_string_lossy();

            // Node.js dependencies
            if path_str.contains("node_modules") {
                dependency_groups
                    .entry("node_modules".to_string())
                    .or_default()
                    .push(node);
            }
            // Rust dependencies
            else if path_str.contains("target/debug") || path_str.contains("target/release") {
                dependency_groups
                    .entry("rust_target".to_string())
                    .or_default()
                    .push(node);
            }
            // Python cache
            else if path_str.contains("__pycache__") || path_str.contains(".pyc") {
                dependency_groups
                    .entry("python_cache".to_string())
                    .or_default()
                    .push(node);
            }
            // Go modules
            else if path_str.contains("go/pkg/mod") {
                dependency_groups
                    .entry("go_modules".to_string())
                    .or_default()
                    .push(node);
            }
        }

        dependency_groups
    }

    /// Calculate potential space savings
    fn calculate_savings(
        &self,
        duplicates: &HashMap<u64, Vec<&FileNode>>,
        build_artifacts: &[&FileNode],
        _large_files: &[&FileNode],
    ) -> u64 {
        let mut total_savings = 0u64;

        // Savings from duplicate removal (keep one, remove others)
        for (size, files) in duplicates {
            if files.len() > 1 {
                total_savings += size * (files.len() - 1) as u64;
            }
        }

        // Savings from build artifact cleanup (conservative estimate: 70%)
        let artifact_size: u64 = build_artifacts.iter().map(|n| n.size).sum();
        total_savings += (artifact_size as f64 * 0.7) as u64;

        total_savings
    }

    /// Generate cleanup suggestions
    fn generate_suggestions(
        &self,
        duplicates: &HashMap<u64, Vec<&FileNode>>,
        build_artifacts: &[&FileNode],
        dependency_waste: &HashMap<String, Vec<&FileNode>>,
        _root_path: &Path,
    ) -> Vec<String> {
        let mut suggestions = Vec::new();

        // Duplicate file suggestions
        if !duplicates.is_empty() {
            suggestions.push("🔄 DUPLICATE FILE CLEANUP:".to_string());
            suggestions.push(
                "   Consider using symbolic links or git submodules for identical files"
                    .to_string(),
            );
            suggestions.push("   Review and consolidate duplicate configuration files".to_string());
            suggestions.push("".to_string());
        }

        // Build artifact suggestions
        if !build_artifacts.is_empty() {
            suggestions.push("🧹 BUILD ARTIFACT CLEANUP:".to_string());
            suggestions.push("   rm -rf */node_modules  # Clean Node.js dependencies".to_string());
            suggestions.push("   rm -rf */target        # Clean Rust build artifacts".to_string());
            suggestions.push("   find . -name '__pycache__' -type d -exec rm -rf {} +".to_string());
            suggestions.push("   Add build directories to .gitignore".to_string());
            suggestions.push("".to_string());
        }

        // Dependency optimization suggestions
        if dependency_waste.contains_key("node_modules") {
            suggestions.push("📦 DEPENDENCY OPTIMIZATION:".to_string());
            suggestions.push("   Consider using pnpm for 60-80% space savings".to_string());
            suggestions.push("   Use yarn workspaces for monorepos".to_string());
            suggestions.push("   Run 'npm dedupe' to remove duplicate packages".to_string());
            suggestions.push("".to_string());
        }

        // General optimization tips
        suggestions.push("💡 OPTIMIZATION TIPS:".to_string());
        suggestions.push("   Use .gitignore to prevent committing build artifacts".to_string());
        suggestions.push("   Consider using Docker multi-stage builds".to_string());
        suggestions.push("   Implement automated cleanup scripts".to_string());

        suggestions
    }
}

impl Formatter for WasteFormatter {
    fn format(
        &self,
        writer: &mut dyn Write,
        nodes: &[FileNode],
        stats: &TreeStats,
        root_path: &Path,
    ) -> Result<()> {
        // Header with Elvis-worthy style! 🎸
        writeln!(writer, "{}", "═".repeat(80))?;
        writeln!(
            writer,
            "🗑️  SMART TREE WASTE ANALYSIS - Marie Kondo Mode Activated! ✨"
        )?;
        writeln!(writer, "   Project: {}", root_path.display())?;
        writeln!(
            writer,
            "   Analyzed: {} files, {} directories",
            stats.total_files, stats.total_dirs
        )?;
        writeln!(writer, "{}", "═".repeat(80))?;
        writeln!(writer)?;

        // Analyze different types of waste
        let duplicates = self.analyze_duplicates(nodes);
        let build_artifacts = self.analyze_build_artifacts(nodes);
        let large_files = self.analyze_large_files(nodes);
        let dependency_waste = self.analyze_dependency_waste(nodes);

        // Calculate total waste and potential savings
        let total_waste_size: u64 = duplicates
            .values()
            .flat_map(|files| files.iter())
            .map(|node| node.size)
            .sum::<u64>()
            + build_artifacts.iter().map(|node| node.size).sum::<u64>();

        let potential_savings = self.calculate_savings(&duplicates, &build_artifacts, &large_files);

        // Summary section - The executive summary for Trisha! 📊
        writeln!(writer, "📊 WASTE SUMMARY:")?;
        writeln!(
            writer,
            "├── Total Project Size: {}",
            format_size(stats.total_size, BINARY)
        )?;
        writeln!(
            writer,
            "├── Potential Waste: {} ({:.1}% of project)",
            format_size(total_waste_size, BINARY),
            (total_waste_size as f64 / stats.total_size as f64) * 100.0
        )?;
        writeln!(writer, "├── Duplicate Groups: {}", duplicates.len())?;
        writeln!(writer, "├── Build Artifacts: {}", build_artifacts.len())?;
        writeln!(
            writer,
            "├── Large Files (>{}): {}",
            format_size(self.large_file_threshold, BINARY),
            large_files.len()
        )?;
        writeln!(
            writer,
            "└── Potential Savings: {} ({:.1}% reduction possible)",
            format_size(potential_savings, BINARY),
            (potential_savings as f64 / stats.total_size as f64) * 100.0
        )?;
        writeln!(writer)?;

        // Duplicate files analysis
        if !duplicates.is_empty() {
            writeln!(writer, "🔄 DUPLICATE FILES DETECTED:")?;
            let mut sorted_duplicates: Vec<_> = duplicates.iter().collect();
            sorted_duplicates
                .sort_by(|a, b| (b.1.len() * *b.0 as usize).cmp(&(a.1.len() * *a.0 as usize)));

            for (size, files) in sorted_duplicates.iter().take(10) {
                writeln!(
                    writer,
                    "├── {} files of size {} each:",
                    files.len(),
                    format_size(**size, BINARY)
                )?;
                for (i, file) in files.iter().take(self.max_duplicates_shown).enumerate() {
                    let rel_path = file.path.strip_prefix(root_path).unwrap_or(&file.path);
                    let prefix = if i == files.len() - 1 || i == self.max_duplicates_shown - 1 {
                        "└──"
                    } else {
                        "├──"
                    };
                    writeln!(writer, "│   {} {}", prefix, rel_path.display())?;
                }
                if files.len() > self.max_duplicates_shown {
                    writeln!(
                        writer,
                        "│   └── ... and {} more",
                        files.len() - self.max_duplicates_shown
                    )?;
                }
            }
            writeln!(writer)?;
        }

        // Build artifacts analysis
        if !build_artifacts.is_empty() {
            writeln!(writer, "🧹 BUILD ARTIFACTS & TEMPORARY FILES:")?;
            let artifact_size: u64 = build_artifacts.iter().map(|n| n.size).sum();
            writeln!(
                writer,
                "├── Total Size: {}",
                format_size(artifact_size, BINARY)
            )?;

            let mut artifact_types: HashMap<String, (usize, u64)> = HashMap::new();
            for artifact in &build_artifacts {
                let path_str = artifact.path.to_string_lossy();
                let artifact_type = if path_str.contains("node_modules") {
                    "node_modules"
                } else if path_str.contains("target") {
                    "rust_target"
                } else if path_str.contains("__pycache__") {
                    "python_cache"
                } else if path_str.contains(".svelte-kit") {
                    "svelte_build"
                } else {
                    "other"
                };

                let entry = artifact_types
                    .entry(artifact_type.to_string())
                    .or_insert((0, 0));
                entry.0 += 1;
                entry.1 += artifact.size;
            }

            for (artifact_type, (count, size)) in artifact_types {
                writeln!(
                    writer,
                    "├── {}: {} files ({})",
                    artifact_type,
                    count,
                    format_size(size, BINARY)
                )?;
            }
            writeln!(writer)?;
        }

        // Large files analysis
        if !large_files.is_empty() {
            writeln!(writer, "📦 LARGE FILES (Potential Optimization Targets):")?;
            for (i, file) in large_files.iter().take(10).enumerate() {
                let rel_path = file.path.strip_prefix(root_path).unwrap_or(&file.path);
                let prefix = if i == large_files.len().min(10) - 1 {
                    "└──"
                } else {
                    "├──"
                };
                writeln!(
                    writer,
                    "{} {} ({})",
                    prefix,
                    rel_path.display(),
                    format_size(file.size, BINARY)
                )?;
            }
            if large_files.len() > 10 {
                writeln!(
                    writer,
                    "└── ... and {} more large files",
                    large_files.len() - 10
                )?;
            }
            writeln!(writer)?;
        }

        // Dependency waste analysis
        if !dependency_waste.is_empty() {
            writeln!(writer, "📚 DEPENDENCY ANALYSIS:")?;
            for (dep_type, files) in &dependency_waste {
                let total_size: u64 = files.iter().map(|f| f.size).sum();
                writeln!(
                    writer,
                    "├── {}: {} files ({})",
                    dep_type,
                    files.len(),
                    format_size(total_size, BINARY)
                )?;
            }
            writeln!(writer)?;
        }

        // Suggestions section - The action plan! 🎯
        if self.show_suggestions {
            let suggestions = self.generate_suggestions(
                &duplicates,
                &build_artifacts,
                &dependency_waste,
                root_path,
            );
            if !suggestions.is_empty() {
                writeln!(writer, "💡 OPTIMIZATION SUGGESTIONS:")?;
                for suggestion in suggestions {
                    if suggestion.is_empty() {
                        writeln!(writer)?;
                    } else {
                        writeln!(writer, "{}", suggestion)?;
                    }
                }
                writeln!(writer)?;
            }
        }

        // Footer with encouragement from Trisha! 💪
        writeln!(writer, "{}", "═".repeat(80))?;
        writeln!(
            writer,
            "🎉 Analysis Complete! Trisha from Accounting is proud of this optimization mindset!"
        )?;
        writeln!(
            writer,
            "   Remember: A clean codebase is a happy codebase! Keep it lean, keep it mean! 🚀"
        )?;
        writeln!(
            writer,
            "   Pro Tip: Run this analysis regularly to keep your projects in tip-top shape!"
        )?;
        writeln!(writer, "{}", "═".repeat(80))?;

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
    fn test_waste_formatter_creation() {
        let formatter = WasteFormatter::new();
        assert_eq!(formatter.large_file_threshold, 10 * 1024 * 1024);
        assert!(formatter.show_suggestions);
    }

    #[test]
    fn test_duplicate_detection() {
        let formatter = WasteFormatter::new();

        // Create test nodes with same size
        let nodes = vec![
            FileNode {
                path: PathBuf::from("/test/file1.txt"),
                is_dir: false,
                size: 1024,
                permissions: 644,
                uid: 1000,
                gid: 1000,
                modified: SystemTime::now(),
                is_symlink: false,
                is_hidden: false,
                permission_denied: false,
                is_ignored: false,
                depth: 1,
                file_type: FileType::RegularFile,
                category: FileCategory::Markdown,
                search_matches: None,
                filesystem_type: FilesystemType::Ext4,
                git_branch: None,
                traversal_context: None,
                interest: None,
                security_findings: Vec::new(),
                change_status: None,
                content_hash: None,
            },
            FileNode {
                path: PathBuf::from("/test/file2.txt"),
                is_dir: false,
                size: 1024, // Same size as file1
                permissions: 644,
                uid: 1000,
                gid: 1000,
                modified: SystemTime::now(),
                is_symlink: false,
                is_hidden: false,
                permission_denied: false,
                is_ignored: false,
                depth: 1,
                file_type: FileType::RegularFile,
                category: FileCategory::Markdown,
                search_matches: None,
                filesystem_type: FilesystemType::Ext4,
                git_branch: None,
                traversal_context: None,
                interest: None,
                security_findings: Vec::new(),
                change_status: None,
                content_hash: None,
            },
        ];

        let duplicates = formatter.analyze_duplicates(&nodes);
        assert_eq!(duplicates.len(), 1);
        assert_eq!(duplicates.get(&1024).unwrap().len(), 2);
    }

    #[test]
    fn test_build_artifact_detection() {
        let formatter = WasteFormatter::new();

        let nodes = vec![FileNode {
            path: PathBuf::from("/test/node_modules/package/index.js"),
            is_dir: false,
            size: 1024,
            permissions: 644,
            uid: 1000,
            gid: 1000,
            modified: SystemTime::now(),
            is_symlink: false,
            is_hidden: false,
            permission_denied: false,
            is_ignored: false,
            depth: 2,
            file_type: FileType::RegularFile,
            category: FileCategory::JavaScript,
            search_matches: None,
            filesystem_type: FilesystemType::Ext4,
            git_branch: None,
            traversal_context: None,
            interest: None,
            security_findings: Vec::new(),
            change_status: None,
            content_hash: None,
        }];

        let artifacts = formatter.analyze_build_artifacts(&nodes);
        assert_eq!(artifacts.len(), 1);
    }
}
