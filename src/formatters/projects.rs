// Projects Discovery Mode - Fast project scanner for AI context
// Finds all README.md files and creates condensed project summaries

use anyhow::Result;
use rayon::prelude::*;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use walkdir::{DirEntry, WalkDir};

use crate::formatters::Formatter;
use crate::scanner::{FileNode, TreeStats};

#[derive(Debug, Clone)]
pub struct ProjectInfo {
    pub path: PathBuf,
    pub name: String,
    pub project_type: ProjectType,
    pub summary: String, // Condensed summary from README
    pub size: u64,       // Total project size
    pub file_count: usize,
    pub created: u64, // Creation timestamp
    pub last_modified: u64,
    pub last_accessed: u64,        // Last access time
    pub dependencies: Vec<String>, // Key dependencies detected
    pub hex_signature: String,     // HEX mode-like signature
    pub git_info: Option<GitInfo>, // Git repository information
}

#[derive(Debug, Clone)]
pub struct GitInfo {
    pub branch: String,
    pub commit: String,         // Short commit hash
    pub commit_message: String, // First line of commit message
    pub is_dirty: bool,         // Has uncommitted changes
    pub ahead: usize,           // Commits ahead of upstream
    pub behind: usize,          // Commits behind upstream
    pub last_commit_date: u64,  // Timestamp of last commit
}

#[derive(Debug, Clone, PartialEq)]
pub enum ProjectType {
    Rust,       // Cargo.toml
    NodeJs,     // package.json
    Python,     // requirements.txt, pyproject.toml, setup.py
    Go,         // go.mod
    Java,       // pom.xml, build.gradle
    DotNet,     // *.csproj, *.sln
    Ruby,       // Gemfile
    Docker,     // Dockerfile
    Kubernetes, // k8s yaml files
    Monorepo,   // Multiple project types
    Unknown,
}

pub struct ProjectsFormatter {
    max_depth: Option<usize>,
    min_project_size: u64, // Skip tiny projects
    show_dependencies: bool,
    condensed_mode: bool, // Ultra-condensed for AI
}

impl Default for ProjectsFormatter {
    fn default() -> Self {
        Self::new()
    }
}

impl ProjectsFormatter {
    pub fn new() -> Self {
        Self {
            max_depth: Some(8),     // Don't go too deep by default
            min_project_size: 1024, // Skip projects < 1KB
            show_dependencies: true,
            condensed_mode: true,
        }
    }

    /// Scan directory for all projects
    pub fn scan_projects(&self, root: &Path) -> Result<Vec<ProjectInfo>> {
        let projects = Arc::new(Mutex::new(Vec::new()));
        let seen_paths = Arc::new(Mutex::new(std::collections::HashSet::new()));
        let root = root.canonicalize().unwrap_or_else(|_| root.to_path_buf());

        // First pass: Find projects with README.md
        let walker = WalkDir::new(&root)
            .max_depth(self.max_depth.unwrap_or(10))
            .follow_links(false)
            .into_iter()
            .filter_map(|e| e.ok())
            .collect::<Vec<_>>();

        walker.par_iter().for_each(|entry| {
            // Check for README.md files
            if self.is_readme(entry) {
                let project_path = entry.path().parent().unwrap();
                if seen_paths
                    .lock()
                    .unwrap()
                    .insert(project_path.to_path_buf())
                {
                    if let Ok(project) = self.analyze_project(project_path) {
                        if project.size >= self.min_project_size {
                            projects.lock().unwrap().push(project);
                        }
                    }
                }
            }

            // Also check for project marker files (even without README)
            if entry.file_type().is_file() {
                let filename = entry.file_name().to_str().unwrap_or("");
                let is_project_marker = matches!(
                    filename,
                    "Cargo.toml"
                        | "package.json"
                        | "go.mod"
                        | "pom.xml"
                        | "build.gradle"
                        | "Gemfile"
                        | "requirements.txt"
                        | "pyproject.toml"
                        | "Dockerfile"
                        | ".gitmodules"
                        | "setup.py"
                        | "Makefile"
                        | "CMakeLists.txt"
                        | "configure.ac"
                        | "Rakefile"
                        | "build.xml"
                        | "build.gradle.kts"
                        | "build.sbt"
                        | "build.sh"
                        | "build.ps1"
                        | "build.bat"
                        | "CMakeCache.txt"
                        | "CMakeLists.txt.user"
                        | "CMakeLists.txt.in"
                        | "CMakeLists.txt.cmake"
                        | ".gitignore"
                        | ".dockerignore"
                        | "docker-compose.yml"
                        | "kustomization.yaml"
                        | "config.yaml"
                        | "CLAUDE.md"
                        


                    );

                if is_project_marker {
                    let project_path = entry.path().parent().unwrap();

                    // Check if we haven't seen this project yet
                    if seen_paths
                        .lock()
                        .unwrap()
                        .insert(project_path.to_path_buf())
                    {
                        // Look for README in parent directory if not found
                        let readme_path = if !project_path.join("README.md").exists() {
                            // Check parent directory
                            project_path
                                .parent()
                                .and_then(|p| {
                                    if p.join("README.md").exists() {
                                        Some(p)
                                    } else {
                                        None
                                    }
                                })
                                .unwrap_or(project_path)
                        } else {
                            project_path
                        };

                        if let Ok(project) =
                            self.analyze_project_with_readme_path(project_path, readme_path)
                        {
                            if project.size >= self.min_project_size {
                                projects.lock().unwrap().push(project);
                            }
                        }
                    }
                }
            }
        });

        let mut result = projects.lock().unwrap().clone();
        result.sort_by(|a, b| b.last_modified.cmp(&a.last_modified)); // Most recent first
        Ok(result)
    }

    fn is_readme(&self, entry: &DirEntry) -> bool {
        entry.file_type().is_file()
            && entry
                .file_name()
                .to_str()
                .map(|s| s.eq_ignore_ascii_case("README.md"))
                .unwrap_or(false)
    }

    /// Analyze a project directory
    fn analyze_project(&self, project_path: &Path) -> Result<ProjectInfo> {
        self.analyze_project_with_readme_path(project_path, project_path)
    }

    /// Analyze project with potentially different README location
    fn analyze_project_with_readme_path(
        &self,
        project_path: &Path,
        readme_path: &Path,
    ) -> Result<ProjectInfo> {
        let mut name = project_path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("unknown")
            .to_string();

        // Check if this is a git submodule
        let is_submodule = project_path.join(".git").exists()
            && project_path
                .parent()
                .map(|p| p.join(".gitmodules").exists())
                .unwrap_or(false);

        if is_submodule {
            name = format!("📎{}", name); // Indicate submodule
        }

        let mut project_type = self.detect_project_type(project_path);

        // If no type detected, check parent directory
        if project_type == ProjectType::Unknown && project_path != readme_path {
            project_type = self.detect_project_type(readme_path);
        }

        let summary = self.extract_summary(readme_path)?;
        let (size, file_count, created, last_modified, last_accessed) =
            self.get_project_stats(project_path)?;
        let mut dependencies = self.detect_dependencies(project_path, &project_type);

        // Check for git submodules as dependencies
        if let Ok(submodules) = self.detect_git_submodules(project_path) {
            for submodule in submodules {
                dependencies.push(format!("📎{}", submodule));
            }
        }

        // Get git information if available
        let git_info = self.get_git_info(project_path);

        // Generate HEX-like signature
        let hex_signature = self.generate_hex_signature(&name, &project_type, size);

        Ok(ProjectInfo {
            path: project_path.to_path_buf(),
            name,
            project_type,
            summary,
            size,
            file_count,
            created,
            last_modified,
            last_accessed,
            dependencies,
            hex_signature,
            git_info,
        })
    }

    /// Detect git submodules in project
    fn detect_git_submodules(&self, path: &Path) -> Result<Vec<String>> {
        let gitmodules_path = path.join(".gitmodules");
        let mut submodules = Vec::new();

        if gitmodules_path.exists() {
            let content = fs::read_to_string(&gitmodules_path)?;
            for line in content.lines() {
                if line.trim().starts_with("path = ") {
                    if let Some(path) = line.split('=').nth(1) {
                        submodules.push(path.trim().to_string());
                    }
                }
            }
        }

        Ok(submodules)
    }

    /// Detect project type based on marker files
    fn detect_project_type(&self, path: &Path) -> ProjectType {
        let markers = vec![
            ("Cargo.toml", ProjectType::Rust),
            ("package.json", ProjectType::NodeJs),
            ("requirements.txt", ProjectType::Python),
            ("pyproject.toml", ProjectType::Python),
            ("setup.py", ProjectType::Python),
            ("go.mod", ProjectType::Go),
            ("pom.xml", ProjectType::Java),
            ("build.gradle", ProjectType::Java),
            ("Gemfile", ProjectType::Ruby),
            ("Dockerfile", ProjectType::Docker),
        ];

        let mut detected_types = Vec::new();

        for (marker, proj_type) in markers {
            if path.join(marker).exists() {
                detected_types.push(proj_type);
            }
        }

        // Check for .csproj or .sln files
        if let Ok(entries) = fs::read_dir(path) {
            for entry in entries.flatten() {
                if let Some(ext) = entry.path().extension() {
                    if ext == "csproj" || ext == "sln" {
                        detected_types.push(ProjectType::DotNet);
                        break;
                    }
                }
            }
        }

        match detected_types.len() {
            0 => ProjectType::Unknown,
            1 => detected_types[0].clone(),
            _ => ProjectType::Monorepo,
        }
    }

    /// Extract and condense summary from README.md
    fn extract_summary(&self, project_path: &Path) -> Result<String> {
        let readme_path = project_path.join("README.md");
        if !readme_path.exists() {
            return Ok(String::new());
        }

        let content = fs::read_to_string(&readme_path)?;

        // Get first paragraph or description
        let summary = self.extract_description(&content);

        if self.condensed_mode {
            Ok(self.condense_text(&summary))
        } else {
            Ok(summary)
        }
    }

    /// Extract description from README content
    fn extract_description(&self, content: &str) -> String {
        let lines: Vec<&str> = content.lines().collect();
        let mut description = String::new();
        let mut found_header = false;
        let mut in_code_block = false;
        let mut consecutive_content_lines = 0;

        for line in lines.iter().take(50) {
            // Look through more lines
            let trimmed = line.trim();

            // Track code blocks
            if trimmed.starts_with("```") {
                in_code_block = !in_code_block;
                continue;
            }

            if in_code_block {
                continue;
            }

            // Skip various non-content patterns
            if trimmed.is_empty() {
                // Empty line - if we have content, check if we should stop
                if consecutive_content_lines >= 2 {
                    break; // We've found enough content
                }
                continue;
            }

            // Skip headers but note we've seen them
            if trimmed.starts_with('#') {
                found_header = true;
                consecutive_content_lines = 0;
                continue;
            }

            // Skip all the badge/shield/image/link-only lines
            if trimmed.starts_with("![") ||       // Images
               trimmed.starts_with("[![") ||      // Clickable badges
               trimmed.starts_with("<!--") ||     // HTML comments
               trimmed.starts_with("<") ||        // HTML tags
               trimmed.starts_with(">") ||        // Blockquotes (often used for notes)
               trimmed.starts_with("[!") ||       // GitHub alerts
               trimmed.starts_with("- [") ||      // TOC entries
               trimmed.starts_with("* [") ||      // TOC entries
               trimmed.starts_with("+ [") ||      // TOC entries
               trimmed.starts_with("|") ||        // Table rows
               trimmed.starts_with("---") ||     // Horizontal rules
               trimmed.starts_with("===") ||     // Alternative headers
               (trimmed.starts_with("[") && trimmed.ends_with(")") &&
                (trimmed.contains("shields.io") || trimmed.contains("badge")))
            {
                continue;
            }

            // Check if this line is mostly links (common in READMEs)
            let link_count = trimmed.matches("](").count();
            let word_count = trimmed.split_whitespace().count();
            if word_count > 0 && link_count > 0 {
                let link_ratio = link_count as f32 / word_count as f32;
                if link_ratio > 0.4 {
                    // More than 40% links
                    continue;
                }
            }

            // Skip very short lines that might just be fragments
            if trimmed.len() < 15 && !found_header {
                continue;
            }

            // Skip lines that are just URLs
            if trimmed.starts_with("http://") || trimmed.starts_with("https://") {
                continue;
            }

            // This looks like real content!
            if !description.is_empty() {
                description.push(' ');
            }

            // Clean up any markdown formatting as we add it
            let cleaned = trimmed
                .replace("**", "") // Bold
                .replace("__", "") // Bold alt
                .replace("~~", "") // Strikethrough
                .replace("`", ""); // Inline code

            description.push_str(&cleaned);
            consecutive_content_lines += 1;

            // Stop after we have enough content
            if description.len() > 200 || consecutive_content_lines >= 3 {
                break;
            }
        }

        // Final cleanup - remove any leftover markdown artifacts
        let mut final_desc = description.trim().to_string();

        // Remove emoji shortcodes that might remain
        if final_desc.contains(':') {
            final_desc = final_desc
                .split_whitespace()
                .filter(|word| !word.starts_with(':') || !word.ends_with(':'))
                .collect::<Vec<_>>()
                .join(" ");
        }

        // Truncate if still too long
        if final_desc.len() > 250 {
            let mut truncated = String::new();
            for (char_count, ch) in final_desc.chars().enumerate() {
                if char_count >= 247 {
                    break;
                }
                truncated.push(ch);
            }

            format!("{}...", truncated)
        } else {
            final_desc
        }
    }

    /// Condense text by removing vowels, stopwords, spaces
    fn condense_text(&self, text: &str) -> String {
        // Common stopwords to remove
        let stopwords = vec![
            "the", "a", "an", "and", "or", "but", "in", "on", "at", "to", "for", "of", "with",
            "by", "from", "up", "about", "into", "through", "during", "is", "are", "was", "were",
            "be", "been", "being", "have", "has", "had", "do", "does", "did", "will", "would",
            "should", "could", "may", "might", "this", "that", "these", "those", "it", "its",
            "which", "what",
        ];

        let words: Vec<&str> = text.split_whitespace().collect();
        let mut condensed = Vec::new();

        for word in words {
            let lower = word.to_lowercase();

            // Skip stopwords
            if stopwords.contains(&lower.as_str()) {
                continue;
            }

            // Remove vowels from words > 3 chars
            let condensed_word = if word.len() > 3 {
                word.chars()
                    .filter(|c| !"aeiouAEIOU".contains(*c))
                    .collect::<String>()
            } else {
                word.to_string()
            };

            if !condensed_word.is_empty() {
                condensed.push(condensed_word);
            }
        }

        // Join with minimal separator
        condensed.join(".")
    }

    /// Get project statistics with timestamps
    fn get_project_stats(&self, path: &Path) -> Result<(u64, usize, u64, u64, u64)> {
        let mut total_size = 0u64;
        let mut file_count = 0usize;
        let mut last_modified = 0u64;
        let mut created = u64::MAX;
        let mut last_accessed = 0u64;

        // Quick scan - don't recurse into node_modules, target, etc.
        let ignored_dirs = [
            "node_modules",
            "target",
            ".git",
            "dist",
            "build",
            "__pycache__",
        ];

        // Get project directory metadata for creation time
        if let Ok(dir_metadata) = fs::metadata(path) {
            #[cfg(unix)]
            {
                use std::os::unix::fs::MetadataExt;
                created = dir_metadata.ctime() as u64;
                last_accessed = dir_metadata.atime() as u64;
            }
            #[cfg(not(unix))]
            {
                if let Ok(created_time) = dir_metadata.created() {
                    if let Ok(duration) = created_time.duration_since(std::time::UNIX_EPOCH) {
                        created = duration.as_secs();
                    }
                }
                if let Ok(accessed_time) = dir_metadata.accessed() {
                    if let Ok(duration) = accessed_time.duration_since(std::time::UNIX_EPOCH) {
                        last_accessed = duration.as_secs();
                    }
                }
            }
        }

        for entry in WalkDir::new(path)
            .max_depth(3) // Don't go too deep for stats
            .into_iter()
            .filter_entry(|e| {
                !e.file_name()
                    .to_str()
                    .map(|s| ignored_dirs.contains(&s))
                    .unwrap_or(false)
            })
            .filter_map(|e| e.ok())
        {
            if entry.file_type().is_file() {
                file_count += 1;
                if let Ok(metadata) = entry.metadata() {
                    total_size += metadata.len();
                    if let Ok(modified) = metadata.modified() {
                        if let Ok(duration) = modified.duration_since(std::time::UNIX_EPOCH) {
                            last_modified = last_modified.max(duration.as_secs());
                        }
                    }
                }
            }
        }

        Ok((
            total_size,
            file_count,
            created,
            last_modified,
            last_accessed,
        ))
    }

    /// Get git information using command-line git
    fn get_git_info(&self, path: &Path) -> Option<GitInfo> {
        use std::process::Command;

        // Check if this is a git repository
        if !path.join(".git").exists() {
            return None;
        }

        // Get current branch
        let branch = Command::new("git")
            .arg("branch")
            .arg("--show-current")
            .current_dir(path)
            .output()
            .ok()
            .and_then(|output| String::from_utf8(output.stdout).ok())
            .map(|s| s.trim().to_string())
            .unwrap_or_else(|| "unknown".to_string());

        // Get current commit hash (short)
        let commit = Command::new("git")
            .arg("rev-parse")
            .arg("--short")
            .arg("HEAD")
            .current_dir(path)
            .output()
            .ok()
            .and_then(|output| String::from_utf8(output.stdout).ok())
            .map(|s| s.trim().to_string())
            .unwrap_or_else(|| "unknown".to_string());

        // Get last commit message (first line)
        let commit_message = Command::new("git")
            .arg("log")
            .arg("-1")
            .arg("--pretty=%s")
            .current_dir(path)
            .output()
            .ok()
            .and_then(|output| String::from_utf8(output.stdout).ok())
            .map(|s| s.trim().to_string())
            .unwrap_or_default();

        // Check if repository is dirty
        let is_dirty = Command::new("git")
            .arg("status")
            .arg("--porcelain")
            .current_dir(path)
            .output()
            .ok()
            .map(|output| !output.stdout.is_empty())
            .unwrap_or(false);

        // Get commits ahead/behind (if tracking upstream)
        let (ahead, behind) = Command::new("git")
            .arg("rev-list")
            .arg("--left-right")
            .arg("--count")
            .arg("HEAD...@{upstream}")
            .current_dir(path)
            .output()
            .ok()
            .and_then(|output| String::from_utf8(output.stdout).ok())
            .and_then(|s| {
                let parts: Vec<&str> = s.trim().split('\t').collect();
                if parts.len() == 2 {
                    Some((parts[0].parse().unwrap_or(0), parts[1].parse().unwrap_or(0)))
                } else {
                    None
                }
            })
            .unwrap_or((0, 0));

        // Get last commit timestamp
        let last_commit_date = Command::new("git")
            .arg("log")
            .arg("-1")
            .arg("--pretty=%ct")
            .current_dir(path)
            .output()
            .ok()
            .and_then(|output| String::from_utf8(output.stdout).ok())
            .and_then(|s| s.trim().parse::<u64>().ok())
            .unwrap_or(0);

        Some(GitInfo {
            branch,
            commit,
            commit_message,
            is_dirty,
            ahead,
            behind,
            last_commit_date,
        })
    }

    /// Detect key dependencies
    fn detect_dependencies(&self, path: &Path, project_type: &ProjectType) -> Vec<String> {
        let mut deps = Vec::new();

        match project_type {
            ProjectType::Rust => {
                if let Ok(content) = fs::read_to_string(path.join("Cargo.toml")) {
                    // Extract key dependencies
                    for line in content.lines() {
                        if line.contains("=") && !line.starts_with('[') {
                            if let Some(dep) = line.split('=').next() {
                                let dep = dep.trim().replace('"', "");
                                if !dep.is_empty() && deps.len() < 5 {
                                    deps.push(dep);
                                }
                            }
                        }
                    }
                }
            }
            ProjectType::NodeJs => {
                if let Ok(content) = fs::read_to_string(path.join("package.json")) {
                    if let Ok(json) = serde_json::from_str::<serde_json::Value>(&content) {
                        if let Some(dependencies) = json["dependencies"].as_object() {
                            for (key, _) in dependencies.iter().take(5) {
                                deps.push(key.to_string());
                            }
                        }
                    }
                }
            }
            _ => {}
        }

        deps
    }

    /// Generate HEX-like signature for project
    fn generate_hex_signature(&self, name: &str, project_type: &ProjectType, size: u64) -> String {
        let type_byte = match project_type {
            ProjectType::Rust => 0x52,       // R
            ProjectType::NodeJs => 0x4E,     // N
            ProjectType::Python => 0x50,     // P
            ProjectType::Go => 0x47,         // G
            ProjectType::Java => 0x4A,       // J
            ProjectType::DotNet => 0x44,     // D
            ProjectType::Ruby => 0x52,       // R
            ProjectType::Docker => 0x43,     // C (Container)
            ProjectType::Kubernetes => 0x4B, // K
            ProjectType::Monorepo => 0x4D,   // M
            ProjectType::Unknown => 0x55,    // U
            
        };

        // Simple hash from name
        let name_hash = name.bytes().fold(0u16, |acc, b| acc.wrapping_add(b as u16));

        // Size category (0-F)
        let size_cat = match size {
            0..=1024 => 0x1,
            1025..=10240 => 0x2,
            10241..=102400 => 0x4,
            102401..=1048576 => 0x8,
            _ => 0xF,
        };

        format!("{:02X}{:04X}{:X}", type_byte, name_hash, size_cat)
    }
}

impl Formatter for ProjectsFormatter {
    fn format(
        &self,
        writer: &mut dyn std::io::Write,
        _nodes: &[FileNode],
        _stats: &TreeStats,
        root_path: &Path,
    ) -> Result<()> {
        let projects = self.scan_projects(root_path)?;

        let mut output = String::new();

        // Header
        output.push_str(&format!(
            "🔍 Project Discovery: {} projects found\n",
            projects.len()
        ));
        output.push_str("═".repeat(60).as_str());
        output.push('\n');

        for project in projects {
            // HEX signature and type indicator
            let type_icon = match project.project_type {
                ProjectType::Rust => "🦀",
                ProjectType::NodeJs => "📦",
                ProjectType::Python => "🐍",
                ProjectType::Go => "🐹",
                ProjectType::Java => "☕",
                ProjectType::DotNet => "🔷",
                ProjectType::Ruby => "💎",
                ProjectType::Docker => "🐳",
                ProjectType::Kubernetes => "☸️",
                ProjectType::Monorepo => "📚",
                ProjectType::Unknown => "📁",
            };

            // Format project entry with name and type
            let project_name = if let Some(ref git) = project.git_info {
                if git.is_dirty {
                    format!("{} *", project.name) // Asterisk for dirty repos
                } else {
                    project.name.clone()
                }
            } else {
                project.name.clone()
            };

            output.push_str(&format!(
                "[{}] {} {}\n",
                project.hex_signature, type_icon, project_name
            ));

            // Condensed summary
            if !project.summary.is_empty() {
                output.push_str(&format!("  └─ {}\n", project.summary));
            }

            // Path (relative if possible)
            let display_path = if let Ok(cwd) = std::env::current_dir() {
                project
                    .path
                    .strip_prefix(&cwd)
                    .unwrap_or(&project.path)
                    .display()
                    .to_string()
            } else {
                project.path.display().to_string()
            };
            output.push_str(&format!("     📍 {}\n", display_path));

            // Git information
            if let Some(ref git) = project.git_info {
                let git_status = if git.ahead > 0 && git.behind > 0 {
                    format!("↑{}↓{}", git.ahead, git.behind)
                } else if git.ahead > 0 {
                    format!("↑{}", git.ahead)
                } else if git.behind > 0 {
                    format!("↓{}", git.behind)
                } else {
                    String::new()
                };

                output.push_str(&format!(
                    "     🔀 {} @ {} {}\n",
                    git.branch, git.commit, git_status
                ));

                if !git.commit_message.is_empty() {
                    let msg = if git.commit_message.chars().count() > 50 {
                        // Use char boundary-safe truncation for Unicode
                        let mut truncated = String::new();
                        for (char_count, ch) in git.commit_message.chars().enumerate() {
                            if char_count >= 47 {
                                break;
                            }
                            truncated.push(ch);
                        }
                        format!("{}...", truncated)
                    } else {
                        git.commit_message.clone()
                    };
                    output.push_str(&format!("        \"{}\"\n", msg));
                }
            }

            // Timestamps
            let created_str = format_timestamp(project.created);
            let modified_str = format_timestamp(project.last_modified);
            output.push_str(&format!(
                "     📅 Created: {}, Modified: {}\n",
                created_str, modified_str
            ));

            // Quick stats
            output.push_str(&format!(
                "     📊 {} files, {}\n",
                project.file_count,
                format_size(project.size)
            ));

            // Dependencies (if any)
            if self.show_dependencies && !project.dependencies.is_empty() {
                let deps_str = if project.dependencies.len() > 3 {
                    format!(
                        "{}, +{} more",
                        project.dependencies[..3].join(", "),
                        project.dependencies.len() - 3
                    )
                } else {
                    project.dependencies.join(", ")
                };
                output.push_str(&format!("     📦 {}\n", deps_str));
            }

            output.push('\n');
        }

        writer.write_all(output.as_bytes())?;
        Ok(())
    }
}

fn format_size(size: u64) -> String {
    const UNITS: &[&str] = &["B", "KB", "MB", "GB"];
    let mut size = size as f64;
    let mut unit_index = 0;

    while size >= 1024.0 && unit_index < UNITS.len() - 1 {
        size /= 1024.0;
        unit_index += 1;
    }

    if unit_index == 0 {
        format!("{} {}", size as u64, UNITS[unit_index])
    } else {
        format!("{:.1} {}", size, UNITS[unit_index])
    }
}

fn format_timestamp(timestamp: u64) -> String {
    use chrono::{Local, TimeZone};

    if timestamp == 0 || timestamp == u64::MAX {
        return "unknown".to_string();
    }

    let dt = Local.timestamp_opt(timestamp as i64, 0).single();
    if let Some(dt) = dt {
        let now = Local::now();
        let duration = now.signed_duration_since(dt);

        if duration.num_days() == 0 {
            return "today".to_string();
        } else if duration.num_days() == 1 {
            return "yesterday".to_string();
        } else if duration.num_days() < 7 {
            return format!("{} days ago", duration.num_days());
        } else if duration.num_days() < 30 {
            return format!("{} weeks ago", duration.num_weeks());
        } else if duration.num_days() < 365 {
            return format!("{} months ago", duration.num_days() / 30);
        } else {
            return format!("{} years ago", duration.num_days() / 365);
        }
    }

    "unknown".to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_condense_text() {
        let formatter = ProjectsFormatter::new();

        let text = "This is a test project for machine learning and data analysis";
        let condensed = formatter.condense_text(text);

        // Should remove vowels and stopwords
        assert!(!condensed.contains("This"));
        assert!(!condensed.contains("is"));
        assert!(!condensed.contains("a"));
        assert!(condensed.contains("tst")); // "test" without vowels
        assert!(condensed.contains("prjct")); // "project" without vowels
    }

    #[test]
    fn test_project_type_detection() {
        let formatter = ProjectsFormatter::new();
        let temp_dir = tempfile::tempdir().unwrap();

        // Create Rust project
        std::fs::write(temp_dir.path().join("Cargo.toml"), "").unwrap();
        assert_eq!(
            formatter.detect_project_type(temp_dir.path()),
            ProjectType::Rust
        );

        // Add package.json - should become Monorepo
        std::fs::write(temp_dir.path().join("package.json"), "{}").unwrap();
        assert_eq!(
            formatter.detect_project_type(temp_dir.path()),
            ProjectType::Monorepo
        );
    }

    // #[test]
    // fn test_hex_signature() {
    //     let formatter = ProjectsFormatter::new();

    //     let sig = formatter.generate_hex_signature("test-project", &ProjectType::Rust, 10240);
    //     assert_eq!(sig.len(), 8); // 2 + 4 + 1 hex chars
    //     assert!(sig.starts_with("52")); // Rust = 0x52
    // }
}
