//! Summary formatter - "Intelligent defaults for humans!" - Omni
//! Provides an intelligent summary based on directory content

use super::Formatter;
use crate::content_detector::{ContentDetector, DirectoryType, Language};
use crate::scanner::{FileNode, TreeStats};
use anyhow::Result;
use colored::Colorize;
use std::collections::HashMap;
use std::io::Write;
use std::path::Path;

pub struct SummaryFormatter {
    use_color: bool,
    max_examples: usize,
}

impl SummaryFormatter {
    pub fn new(use_color: bool) -> Self {
        Self {
            use_color,
            max_examples: 5,
        }
    }

    fn colorize(&self, text: &str, color: &str) -> String {
        if self.use_color {
            match color {
                "blue" => text.blue().to_string(),
                "green" => text.green().to_string(),
                "yellow" => text.yellow().to_string(),
                "red" => text.red().to_string(),
                "cyan" => text.cyan().to_string(),
                "magenta" => text.magenta().to_string(),
                "bold" => text.bold().to_string(),
                _ => text.to_string(),
            }
        } else {
            text.to_string()
        }
    }

    fn is_high_level_directory(&self, nodes: &[FileNode], _stats: &TreeStats) -> bool {
        // Heuristics for detecting high-level directories:
        // 1. More than 20 subdirectories in root
        // 2. Has typical home directory folders (Documents, Downloads, etc.)
        // 3. Has multiple project-like directories

        // Count directories at root level (relative to scanned path)
        let mut root_dir_count = 0;
        let mut seen_paths = std::collections::HashSet::new();

        for node in nodes {
            if node.is_dir {
                // Get the depth relative to the first node's parent
                if let Some(first) = nodes.first() {
                    if let Some(base) = first.path.parent() {
                        if let Ok(relative) = node.path.strip_prefix(base) {
                            if relative.components().count() == 1
                                && seen_paths.insert(node.path.clone())
                            {
                                root_dir_count += 1;
                            }
                        }
                    }
                }
            }
        }

        if root_dir_count > 20 {
            return true;
        }

        // Check for home directory patterns
        let home_folders = [
            "Documents",
            "Downloads",
            "Desktop",
            "Pictures",
            "Music",
            "Videos",
        ];
        let mut home_folder_count = 0;

        for node in nodes {
            if node.is_dir {
                if let Some(name) = node.path.file_name().and_then(|f| f.to_str()) {
                    if home_folders.contains(&name) {
                        home_folder_count += 1;
                    }
                }
            }
        }

        if home_folder_count >= 3 {
            return true;
        }

        // Check for multiple project-like directories
        let project_indicators = [
            "Cargo.toml",
            "package.json",
            "pom.xml",
            ".git",
            "requirements.txt",
        ];
        let mut project_dirs = std::collections::HashSet::new();

        for node in nodes {
            if let Some(name) = node.path.file_name().and_then(|f| f.to_str()) {
                if project_indicators.contains(&name) {
                    if let Some(parent) = node.path.parent() {
                        project_dirs.insert(parent);
                    }
                }
            }
        }

        project_dirs.len() > 5
    }

    fn format_high_level_summary(
        &self,
        writer: &mut dyn Write,
        nodes: &[FileNode],
        stats: &TreeStats,
        root_path: &Path,
    ) -> Result<()> {
        // Header
        writeln!(writer, "{}", self.colorize("📊 Directory Overview", "bold"))?;
        writeln!(writer, "{}", self.colorize("─".repeat(50).as_str(), "blue"))?;
        writeln!(writer)?;

        // Path and basic stats
        writeln!(
            writer,
            "📁 {}: {}",
            self.colorize("Path", "cyan"),
            root_path.display()
        )?;
        writeln!(
            writer,
            "📈 {}: {} files, {} directories, {}",
            self.colorize("Total", "cyan"),
            self.colorize(&stats.total_files.to_string(), "green"),
            self.colorize(&stats.total_dirs.to_string(), "green"),
            self.colorize(&format_size(stats.total_size), "green")
        )?;
        writeln!(writer)?;

        // Analyze subdirectories (skip root-level files)
        let mut subdirs: HashMap<String, (usize, usize, u64)> = HashMap::new();
        let mut actual_dirs: std::collections::HashSet<String> = std::collections::HashSet::new();

        for node in nodes {
            if let Ok(relative) = node.path.strip_prefix(root_path) {
                let components: Vec<_> = relative.components().collect();
                if let Some(first) = components.first() {
                    if let Some(name) = first.as_os_str().to_str() {
                        // Only track as directory if:
                        // 1. The node is a directory at depth 1, OR
                        // 2. There are more components (meaning this is a parent dir)
                        if node.is_dir && components.len() == 1 {
                            actual_dirs.insert(name.to_string());
                        }
                        if components.len() > 1 {
                            actual_dirs.insert(name.to_string());
                        }

                        let entry = subdirs.entry(name.to_string()).or_insert((0, 0, 0));
                        if node.is_dir {
                            entry.1 += 1;
                        } else {
                            entry.0 += 1;
                            entry.2 += node.size;
                        }
                    }
                }
            }
        }

        // Filter to only actual directories and sort by size
        let mut sorted_dirs: Vec<_> = subdirs
            .into_iter()
            .filter(|(name, _)| actual_dirs.contains(name))
            .collect();
        sorted_dirs.sort_by(|a, b| b.1 .2.cmp(&a.1 .2));

        // Show top directories
        writeln!(
            writer,
            "{}",
            self.colorize("Top Directories by Size:", "yellow")
        )?;
        writeln!(writer)?;

        for (name, (files, dirs, size)) in sorted_dirs.iter().take(10) {
            let size_str = format_size(*size);
            let size_bar = self.make_size_bar(*size, stats.total_size);

            writeln!(
                writer,
                "  {} {} {}",
                self.colorize(&format!("{:20}", name), "cyan"),
                self.colorize(&format!("{:>10}", size_str), "green"),
                size_bar
            )?;
            writeln!(
                writer,
                "  {:20} {} files, {} dirs",
                "",
                self.colorize(&files.to_string(), "blue"),
                self.colorize(&dirs.to_string(), "blue")
            )?;
            writeln!(writer)?;
        }

        // Detect projects
        let projects = self.detect_projects(nodes, root_path);
        if !projects.is_empty() {
            writeln!(writer, "{}", self.colorize("Detected Projects:", "yellow"))?;
            writeln!(writer)?;

            for (path, project_type) in projects.iter().take(10) {
                writeln!(
                    writer,
                    "  • {} {}",
                    self.colorize(path, "cyan"),
                    self.colorize(&format!("({})", project_type), "magenta")
                )?;
            }
            writeln!(writer)?;
        }

        // Footer
        writeln!(writer, "{}", self.colorize("─".repeat(50).as_str(), "blue"))?;
        writeln!(
            writer,
            "💡 {}: Use {} to analyze a specific directory",
            self.colorize("Tip", "yellow"),
            self.colorize("st <directory>", "cyan")
        )?;

        Ok(())
    }

    fn make_size_bar(&self, size: u64, total: u64) -> String {
        if total == 0 {
            return String::new();
        }

        let percentage = (size as f64 / total as f64) * 100.0;
        let bar_width = 20;
        let filled = ((percentage / 100.0) * bar_width as f64) as usize;

        let bar = "█".repeat(filled) + &"░".repeat(bar_width - filled);

        format!("{} {:5.1}%", self.colorize(&bar, "blue"), percentage)
    }

    fn detect_projects(&self, nodes: &[FileNode], root_path: &Path) -> Vec<(String, String)> {
        let mut projects = Vec::new();
        let mut checked_dirs = std::collections::HashSet::new();

        for node in nodes {
            if let Some(parent) = node.path.parent() {
                if checked_dirs.contains(parent) {
                    continue;
                }
                checked_dirs.insert(parent);

                let name = node.path.file_name().and_then(|n| n.to_str()).unwrap_or("");

                let project_type = match name {
                    "Cargo.toml" => Some("Rust"),
                    "package.json" => Some("Node.js"),
                    "requirements.txt" | "setup.py" | "pyproject.toml" => Some("Python"),
                    "go.mod" => Some("Go"),
                    "pom.xml" => Some("Java/Maven"),
                    "build.gradle" | "build.gradle.kts" => Some("Java/Gradle"),
                    "Gemfile" => Some("Ruby"),
                    ".git" if node.is_dir => Some("Git Repository"),
                    _ => None,
                };

                if let Some(ptype) = project_type {
                    if let Ok(relative) = parent.strip_prefix(root_path) {
                        projects.push((relative.display().to_string(), ptype.to_string()));
                    }
                }
            }
        }

        projects.sort();
        projects.dedup();
        projects
    }
}

impl Formatter for SummaryFormatter {
    fn format(
        &self,
        writer: &mut dyn Write,
        nodes: &[FileNode],
        stats: &TreeStats,
        root_path: &Path,
    ) -> Result<()> {
        // Check if this looks like a high-level directory (home, root, etc)
        let is_high_level = self.is_high_level_directory(nodes, stats);

        if is_high_level {
            return self.format_high_level_summary(writer, nodes, stats, root_path);
        }

        // Detect directory type for project-level analysis
        let dir_type = ContentDetector::detect(nodes, root_path);

        // Header
        writeln!(writer, "{}", self.colorize("📊 Directory Summary", "bold"))?;
        writeln!(writer, "{}", self.colorize("─".repeat(50).as_str(), "blue"))?;
        writeln!(writer)?;

        // Path and basic stats
        writeln!(
            writer,
            "📁 {}: {}",
            self.colorize("Path", "cyan"),
            root_path.display()
        )?;
        writeln!(
            writer,
            "📈 {}: {} files, {} directories, {}",
            self.colorize("Stats", "cyan"),
            self.colorize(&stats.total_files.to_string(), "green"),
            self.colorize(&stats.total_dirs.to_string(), "green"),
            self.colorize(&format_size(stats.total_size), "green")
        )?;
        writeln!(writer)?;

        // Content-specific analysis
        match &dir_type {
            DirectoryType::CodeProject {
                language,
                framework,
                has_tests,
                has_docs,
            } => {
                writeln!(
                    writer,
                    "🔧 {}: {} Project",
                    self.colorize("Type", "yellow"),
                    self.colorize(&format!("{:?}", language), "magenta")
                )?;

                if let Some(fw) = framework {
                    writeln!(
                        writer,
                        "🚀 {}: {:?}",
                        self.colorize("Framework", "yellow"),
                        fw
                    )?;
                }

                writeln!(
                    writer,
                    "✅ Tests: {} | 📚 Docs: {}",
                    if *has_tests {
                        self.colorize("Yes", "green")
                    } else {
                        self.colorize("No", "red")
                    },
                    if *has_docs {
                        self.colorize("Yes", "green")
                    } else {
                        self.colorize("No", "red")
                    }
                )?;

                // Show main files
                writeln!(writer)?;
                writeln!(writer, "{}", self.colorize("Key Files:", "cyan"))?;

                // Find and display important files
                let important_files = find_important_code_files(nodes, language);
                for file in important_files.iter().take(self.max_examples) {
                    writeln!(writer, "  • {}", file)?;
                }

                // Language-specific tips
                writeln!(writer)?;
                writeln!(writer, "{}", self.colorize("Quick Commands:", "cyan"))?;
                match language {
                    Language::Rust => {
                        writeln!(writer, "  • cargo build --release")?;
                        writeln!(writer, "  • cargo test")?;
                        writeln!(writer, "  • cargo run")?;
                    }
                    Language::Python => {
                        writeln!(writer, "  • python -m venv venv")?;
                        writeln!(writer, "  • pip install -r requirements.txt")?;
                        writeln!(writer, "  • python main.py")?;
                    }
                    Language::JavaScript | Language::TypeScript => {
                        writeln!(writer, "  • npm install")?;
                        writeln!(writer, "  • npm test")?;
                        writeln!(writer, "  • npm start")?;
                    }
                    _ => {
                        writeln!(writer, "  • Check README for build instructions")?;
                    }
                }
            }

            DirectoryType::PhotoCollection {
                image_count,
                date_range,
                cameras,
            } => {
                writeln!(
                    writer,
                    "📷 {}: Photo Collection",
                    self.colorize("Type", "yellow")
                )?;
                writeln!(
                    writer,
                    "🖼️  {}: {} images",
                    self.colorize("Count", "cyan"),
                    self.colorize(&image_count.to_string(), "green")
                )?;

                if let Some((start, end)) = date_range {
                    writeln!(
                        writer,
                        "📅 {}: {} to {}",
                        self.colorize("Date Range", "cyan"),
                        start,
                        end
                    )?;
                }

                if !cameras.is_empty() {
                    writeln!(
                        writer,
                        "📸 {}: {}",
                        self.colorize("Cameras", "cyan"),
                        cameras.join(", ")
                    )?;
                }

                // Show file type breakdown
                let mut type_counts: HashMap<&str, usize> = HashMap::new();
                for node in nodes {
                    if !node.is_dir {
                        if let Some(ext) = node.path.extension().and_then(|e| e.to_str()) {
                            *type_counts.entry(ext).or_insert(0) += 1;
                        }
                    }
                }

                writeln!(writer)?;
                writeln!(writer, "{}", self.colorize("File Types:", "cyan"))?;
                for (ext, count) in type_counts.iter() {
                    writeln!(writer, "  • .{}: {}", ext, count)?;
                }
            }

            DirectoryType::DocumentArchive {
                categories,
                total_docs,
            } => {
                writeln!(
                    writer,
                    "📚 {}: Document Archive",
                    self.colorize("Type", "yellow")
                )?;
                writeln!(
                    writer,
                    "📄 {}: {} documents",
                    self.colorize("Count", "cyan"),
                    self.colorize(&total_docs.to_string(), "green")
                )?;

                if !categories.is_empty() {
                    writeln!(writer)?;
                    writeln!(writer, "{}", self.colorize("Categories:", "cyan"))?;
                    for (category, count) in categories.iter() {
                        writeln!(writer, "  • {}: {}", category, count)?;
                    }
                }
            }

            DirectoryType::MediaLibrary {
                video_count,
                audio_count,
                total_duration,
                quality,
            } => {
                writeln!(
                    writer,
                    "🎬 {}: Media Library",
                    self.colorize("Type", "yellow")
                )?;
                writeln!(
                    writer,
                    "🎥 Videos: {} | 🎵 Audio: {}",
                    self.colorize(&video_count.to_string(), "green"),
                    self.colorize(&audio_count.to_string(), "green")
                )?;

                if let Some(duration) = total_duration {
                    writeln!(
                        writer,
                        "⏱️  {}: {}",
                        self.colorize("Total Duration", "cyan"),
                        duration
                    )?;
                }

                if !quality.is_empty() {
                    writeln!(
                        writer,
                        "📺 {}: {}",
                        self.colorize("Quality", "cyan"),
                        quality.join(", ")
                    )?;
                }
            }

            DirectoryType::DataScience {
                notebooks,
                datasets,
                languages,
            } => {
                writeln!(
                    writer,
                    "🔬 {}: Data Science Workspace",
                    self.colorize("Type", "yellow")
                )?;
                writeln!(
                    writer,
                    "📓 Notebooks: {} | 📊 Datasets: {}",
                    self.colorize(&notebooks.to_string(), "green"),
                    self.colorize(&datasets.to_string(), "green")
                )?;

                if !languages.is_empty() {
                    writeln!(
                        writer,
                        "🐍 {}: {}",
                        self.colorize("Languages", "cyan"),
                        languages.join(", ")
                    )?;
                }

                writeln!(writer)?;
                writeln!(writer, "{}", self.colorize("Quick Commands:", "cyan"))?;
                writeln!(writer, "  • jupyter notebook")?;
                writeln!(writer, "  • jupyter lab")?;
                writeln!(writer, "  • python -m notebook")?;
            }

            DirectoryType::MixedContent {
                dominant_type,
                file_types,
                total_files,
            } => {
                writeln!(
                    writer,
                    "📦 {}: Mixed Content",
                    self.colorize("Type", "yellow")
                )?;

                if let Some(dominant) = dominant_type {
                    writeln!(
                        writer,
                        "🎯 {}: {}",
                        self.colorize("Dominant Type", "cyan"),
                        dominant
                    )?;
                }

                writeln!(
                    writer,
                    "📊 {}: {}",
                    self.colorize("Total Files", "cyan"),
                    self.colorize(&total_files.to_string(), "green")
                )?;

                // Show top file types
                let mut types: Vec<_> = file_types.iter().collect();
                types.sort_by(|a, b| b.1.cmp(a.1));

                writeln!(writer)?;
                writeln!(writer, "{}", self.colorize("Top File Types:", "cyan"))?;
                for (ext, count) in types.iter().take(self.max_examples) {
                    writeln!(writer, "  • .{}: {}", ext, count)?;
                }
            }
        }

        // Footer with suggestions
        writeln!(writer)?;
        writeln!(writer, "{}", self.colorize("─".repeat(50).as_str(), "blue"))?;
        writeln!(
            writer,
            "💡 {}: Use {} for detailed analysis",
            self.colorize("Tip", "yellow"),
            self.colorize("st --mode relations", "cyan")
        )?;

        Ok(())
    }
}

fn format_size(bytes: u64) -> String {
    const UNITS: &[&str] = &["B", "KB", "MB", "GB", "TB"];
    let mut size = bytes as f64;
    let mut unit_index = 0;

    while size >= 1024.0 && unit_index < UNITS.len() - 1 {
        size /= 1024.0;
        unit_index += 1;
    }

    if unit_index == 0 {
        format!("{} {}", size as u64, UNITS[unit_index])
    } else {
        format!("{:.2} {}", size, UNITS[unit_index])
    }
}

fn find_important_code_files(nodes: &[FileNode], language: &Language) -> Vec<String> {
    let mut important = Vec::new();

    for node in nodes {
        if node.is_dir {
            continue;
        }

        let name = node.path.file_name().and_then(|n| n.to_str()).unwrap_or("");

        let is_important = match language {
            Language::Rust => {
                matches!(name, "main.rs" | "lib.rs" | "Cargo.toml" | "build.rs")
            }
            Language::Python => {
                matches!(
                    name,
                    "main.py" | "__init__.py" | "setup.py" | "requirements.txt" | "pyproject.toml"
                )
            }
            Language::JavaScript | Language::TypeScript => {
                matches!(
                    name,
                    "index.js"
                        | "index.ts"
                        | "package.json"
                        | "tsconfig.json"
                        | "webpack.config.js"
                )
            }
            Language::Go => {
                matches!(name, "main.go" | "go.mod" | "go.sum")
            }
            Language::Java => {
                matches!(name, "Main.java" | "pom.xml" | "build.gradle")
            }
            _ => false,
        };

        if is_important {
            important.push(node.path.display().to_string());
        }
    }

    important
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::scanner::FileNode;
    use std::path::PathBuf;

    fn create_test_nodes() -> Vec<FileNode> {
        use crate::scanner::{FileCategory, FileType, FilesystemType};
        vec![
            FileNode {
                path: PathBuf::from("/test/src/main.rs"),
                is_dir: false,
                size: 1000,
                permissions: 0o644,
                uid: 1000,
                gid: 1000,
                modified: std::time::SystemTime::now(),
                is_symlink: false,
                is_hidden: false,
                permission_denied: false,
                is_ignored: false,
                depth: 2,
                file_type: FileType::RegularFile,
                category: FileCategory::Rust,
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
                path: PathBuf::from("/test/Cargo.toml"),
                is_dir: false,
                size: 500,
                permissions: 0o644,
                uid: 1000,
                gid: 1000,
                modified: std::time::SystemTime::now(),
                is_symlink: false,
                is_hidden: false,
                permission_denied: false,
                is_ignored: false,
                depth: 1,
                file_type: FileType::RegularFile,
                category: FileCategory::Toml,
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
                path: PathBuf::from("/test/src"),
                is_dir: true,
                size: 0,
                permissions: 0o755,
                uid: 1000,
                gid: 1000,
                modified: std::time::SystemTime::now(),
                is_symlink: false,
                is_hidden: false,
                permission_denied: false,
                is_ignored: false,
                depth: 1,
                file_type: FileType::Directory,
                category: FileCategory::Unknown,
                search_matches: None,
                filesystem_type: FilesystemType::Ext4,
                git_branch: None,
                traversal_context: None,
                interest: None,
                security_findings: Vec::new(),
                change_status: None,
                content_hash: None,
            },
        ]
    }

    #[test]
    fn test_summary_formatter_rust_project() {
        let formatter = SummaryFormatter::new(false);
        let nodes = create_test_nodes();
        let stats = TreeStats {
            total_files: 2,
            total_dirs: 1,
            total_size: 1500,
            file_types: HashMap::new(),
            largest_files: vec![],
            newest_files: vec![],
            oldest_files: vec![],
        };

        let mut output = Vec::new();
        let result = formatter.format(&mut output, &nodes, &stats, &PathBuf::from("/test"));

        assert!(result.is_ok());
        let output_str = String::from_utf8(output).unwrap();
        assert!(output_str.contains("Rust Project"));
        assert!(output_str.contains("cargo build"));
    }

    #[test]
    fn test_high_level_directory_detection() {
        let formatter = SummaryFormatter::new(false);

        // Create many directories to trigger high-level detection
        use crate::scanner::{FileCategory, FileType, FilesystemType};
        let mut nodes = vec![];
        for i in 0..25 {
            nodes.push(FileNode {
                path: PathBuf::from(format!("/home/user/dir{}", i)),
                is_dir: true,
                size: 0,
                permissions: 0o755,
                uid: 1000,
                gid: 1000,
                modified: std::time::SystemTime::now(),
                is_symlink: false,
                is_hidden: false,
                permission_denied: false,
                is_ignored: false,
                depth: 1,
                file_type: FileType::Directory,
                category: FileCategory::Unknown,
                search_matches: None,
                filesystem_type: FilesystemType::Ext4,
                git_branch: None,
                traversal_context: None,
                interest: None,
                security_findings: Vec::new(),
                change_status: None,
                content_hash: None,
            });
        }

        let stats = TreeStats {
            total_files: 0,
            total_dirs: 25,
            total_size: 0,
            file_types: HashMap::new(),
            largest_files: vec![],
            newest_files: vec![],
            oldest_files: vec![],
        };

        let is_high_level = formatter.is_high_level_directory(&nodes, &stats);
        assert!(is_high_level);
    }
}
