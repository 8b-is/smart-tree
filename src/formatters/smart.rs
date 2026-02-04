//
// -----------------------------------------------------------------------------
//  SMART FORMATTER: Surface What Matters
//
//  This formatter transforms a wall of files into actionable intelligence.
//  Instead of "here's everything", it says "here's what you need to know."
//
//  Output is grouped by interest level:
//  - SECURITY: Critical findings that need immediate attention
//  - IMPORTANT: Key files, recent changes, high-interest items
//  - CHANGES: What's different since last scan
//  - NOTABLE: Worth knowing but not urgent
//  - BACKGROUND: Summary count (not listed individually)
//
//  "Don't list everything. Surface what matters." - The Smart Tree Philosophy
// -----------------------------------------------------------------------------
//

use crate::formatters::{Formatter, PathDisplayMode};
use crate::scanner::{FileNode, TreeStats};
use crate::scanner_interest::InterestLevel;
use crate::security_scan::RiskLevel;
use anyhow::Result;
use std::collections::HashMap;
use std::io::Write;
use std::path::Path;
use std::time::SystemTime;

/// Smart Formatter - Groups output by interest level
pub struct SmartFormatter {
    /// Use colors in output
    use_color: bool,
    /// Use emoji in output
    use_emoji: bool,
    /// Show background files individually (default: just count)
    show_background: bool,
    /// Path display mode
    path_mode: PathDisplayMode,
    /// Minimum interest level to show individually
    min_level: InterestLevel,
}

impl SmartFormatter {
    pub fn new(use_color: bool, use_emoji: bool) -> Self {
        Self {
            use_color,
            use_emoji,
            show_background: false,
            path_mode: PathDisplayMode::Relative,
            min_level: InterestLevel::Background,
        }
    }

    /// Set whether to show background files individually
    pub fn with_show_background(mut self, show: bool) -> Self {
        self.show_background = show;
        self
    }

    /// Set path display mode
    pub fn with_path_mode(mut self, mode: PathDisplayMode) -> Self {
        self.path_mode = mode;
        self
    }

    /// Set minimum interest level to display
    pub fn with_min_level(mut self, level: InterestLevel) -> Self {
        self.min_level = level;
        self
    }

    /// Format a path for display
    fn format_path<'a>(&self, path: &'a Path, root: &Path) -> std::borrow::Cow<'a, str> {
        match self.path_mode {
            PathDisplayMode::Off => path
                .file_name()
                .map(|n| n.to_string_lossy())
                .unwrap_or_else(|| path.to_string_lossy()),
            PathDisplayMode::Relative => path
                .strip_prefix(root)
                .map(|p| p.to_string_lossy())
                .unwrap_or_else(|_| path.to_string_lossy()),
            PathDisplayMode::Full => path.to_string_lossy(),
        }
    }

    /// Get color code for interest level
    fn level_color(&self, level: InterestLevel) -> &'static str {
        if !self.use_color {
            return "";
        }
        match level {
            InterestLevel::Critical => "\x1b[1;31m", // Bold red
            InterestLevel::Important => "\x1b[1;33m", // Bold yellow
            InterestLevel::Notable => "\x1b[36m",    // Cyan
            InterestLevel::Background => "\x1b[90m", // Gray
            InterestLevel::Boring => "\x1b[90m",     // Gray
        }
    }

    /// Get color code for risk level
    fn risk_color(&self, risk: RiskLevel) -> &'static str {
        if !self.use_color {
            return "";
        }
        match risk {
            RiskLevel::Critical => "\x1b[1;31m", // Bold red
            RiskLevel::High => "\x1b[31m",       // Red
            RiskLevel::Medium => "\x1b[33m",     // Yellow
            RiskLevel::Low => "\x1b[36m",        // Cyan
        }
    }

    /// Reset color
    fn reset(&self) -> &'static str {
        if self.use_color {
            "\x1b[0m"
        } else {
            ""
        }
    }

    /// Get emoji for section
    fn section_emoji(&self, section: &str) -> &'static str {
        if !self.use_emoji {
            return "";
        }
        match section {
            "security" => "⚠️  ",
            "important" => "🔥 ",
            "changes" => "📝 ",
            "notable" => "📌 ",
            "background" => "📦 ",
            "project" => "🌳 ",
            _ => "",
        }
    }

    /// Format time ago string
    fn time_ago(&self, modified: SystemTime) -> String {
        let now = SystemTime::now();
        let duration = now.duration_since(modified).unwrap_or_default();
        let secs = duration.as_secs();

        if secs < 60 {
            "just now".to_string()
        } else if secs < 3600 {
            format!("{}m ago", secs / 60)
        } else if secs < 86400 {
            format!("{}h ago", secs / 3600)
        } else if secs < 604800 {
            format!("{}d ago", secs / 86400)
        } else {
            format!("{}w ago", secs / 604800)
        }
    }

    /// Detect project type from nodes
    fn detect_project_type(&self, nodes: &[FileNode]) -> Option<(&'static str, Option<String>)> {
        for node in nodes {
            if node.is_dir {
                continue;
            }
            let name = node.path.file_name()?.to_str()?;
            match name.to_lowercase().as_str() {
                "cargo.toml" => return Some(("Rust", node.git_branch.clone())),
                "package.json" => return Some(("Node.js", node.git_branch.clone())),
                "pyproject.toml" | "setup.py" => return Some(("Python", node.git_branch.clone())),
                "go.mod" => return Some(("Go", node.git_branch.clone())),
                "gemfile" => return Some(("Ruby", node.git_branch.clone())),
                "pom.xml" | "build.gradle" => return Some(("Java", node.git_branch.clone())),
                _ => continue,
            }
        }
        // Check for git branch in any node
        for node in nodes {
            if let Some(ref branch) = node.git_branch {
                return Some(("Project", Some(branch.clone())));
            }
        }
        None
    }

    /// Group nodes by interest level
    fn group_by_interest<'a>(&self, nodes: &'a [FileNode]) -> HashMap<InterestLevel, Vec<&'a FileNode>> {
        let mut groups: HashMap<InterestLevel, Vec<&'a FileNode>> = HashMap::new();

        for node in nodes {
            let level = node
                .interest
                .as_ref()
                .map(|i| i.level)
                .unwrap_or(InterestLevel::Background);

            groups.entry(level).or_default().push(node);
        }

        groups
    }

    /// Collect all security findings from nodes
    fn collect_security_findings<'a>(&self, nodes: &'a [FileNode]) -> Vec<(&'a FileNode, &'a crate::security_scan::SecurityFinding)> {
        let mut findings = Vec::new();
        for node in nodes {
            for finding in &node.security_findings {
                findings.push((node, finding));
            }
        }
        // Sort by risk level (critical first)
        findings.sort_by(|a, b| b.1.risk_level.cmp(&a.1.risk_level));
        findings
    }

    /// Collect changed files
    fn collect_changes<'a>(&self, nodes: &'a [FileNode]) -> (Vec<&'a FileNode>, Vec<&'a FileNode>, Vec<&'a FileNode>) {
        let mut added = Vec::new();
        let mut modified = Vec::new();
        let mut deleted = Vec::new(); // We won't have deleted in nodes, but for completeness

        for node in nodes {
            if let Some(ref change) = node.change_status {
                match change {
                    crate::scanner_interest::ChangeType::Added => added.push(node),
                    crate::scanner_interest::ChangeType::Modified
                    | crate::scanner_interest::ChangeType::PermissionChanged
                    | crate::scanner_interest::ChangeType::TypeChanged
                    | crate::scanner_interest::ChangeType::Renamed => modified.push(node),
                    crate::scanner_interest::ChangeType::Deleted => deleted.push(node),
                }
            }
        }

        (added, modified, deleted)
    }
}

impl Formatter for SmartFormatter {
    fn format(
        &self,
        writer: &mut dyn Write,
        nodes: &[FileNode],
        stats: &TreeStats,
        root_path: &Path,
    ) -> Result<()> {
        // === HEADER: Project info ===
        let project_name = root_path
            .file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_else(|| ".".to_string());

        let (project_type, git_branch) = self.detect_project_type(nodes).unwrap_or(("", None));

        write!(writer, "{}", self.section_emoji("project"))?;
        write!(writer, "{}{}{}", self.level_color(InterestLevel::Important), project_name, self.reset())?;

        if !project_type.is_empty() {
            write!(writer, " ({})", project_type)?;
        }
        if let Some(branch) = git_branch {
            write!(writer, " [{}]", branch)?;
        }
        writeln!(writer)?;
        writeln!(writer)?;

        // === SECURITY SECTION ===
        let security_findings = self.collect_security_findings(nodes);
        if !security_findings.is_empty() {
            writeln!(
                writer,
                "{}{}SECURITY ({} finding{}){}",
                self.section_emoji("security"),
                self.level_color(InterestLevel::Critical),
                security_findings.len(),
                if security_findings.len() == 1 { "" } else { "s" },
                self.reset()
            )?;

            for (node, finding) in security_findings.iter().take(10) {
                let path = self.format_path(&node.path, root_path);
                writeln!(
                    writer,
                    "  {}{}: {}{}",
                    self.risk_color(finding.risk_level),
                    path,
                    finding.description,
                    self.reset()
                )?;
            }

            if security_findings.len() > 10 {
                writeln!(
                    writer,
                    "  {}... and {} more{}",
                    self.level_color(InterestLevel::Background),
                    security_findings.len() - 10,
                    self.reset()
                )?;
            }
            writeln!(writer)?;
        }

        // === CHANGES SECTION ===
        let (added, modified, _deleted) = self.collect_changes(nodes);
        if !added.is_empty() || !modified.is_empty() {
            writeln!(
                writer,
                "{}{}CHANGES{}",
                self.section_emoji("changes"),
                self.level_color(InterestLevel::Notable),
                self.reset()
            )?;

            // Show added files
            for node in added.iter().take(5) {
                let path = self.format_path(&node.path, root_path);
                writeln!(
                    writer,
                    "  {}+ {}{}",
                    self.level_color(InterestLevel::Notable),
                    path,
                    self.reset()
                )?;
            }

            // Show modified files
            for node in modified.iter().take(5) {
                let path = self.format_path(&node.path, root_path);
                let time = self.time_ago(node.modified);
                writeln!(
                    writer,
                    "  {}~ {} [{}]{}",
                    self.level_color(InterestLevel::Notable),
                    path,
                    time,
                    self.reset()
                )?;
            }

            let total_changes = added.len() + modified.len();
            if total_changes > 10 {
                writeln!(
                    writer,
                    "  {}... and {} more changes{}",
                    self.level_color(InterestLevel::Background),
                    total_changes - 10,
                    self.reset()
                )?;
            }
            writeln!(writer)?;
        }

        // === IMPORTANT SECTION ===
        let groups = self.group_by_interest(nodes);

        if let Some(critical) = groups.get(&InterestLevel::Critical) {
            if !critical.is_empty() {
                writeln!(
                    writer,
                    "{}{}CRITICAL{}",
                    self.section_emoji("important"),
                    self.level_color(InterestLevel::Critical),
                    self.reset()
                )?;

                for node in critical.iter().take(10) {
                    let path = self.format_path(&node.path, root_path);
                    let time = self.time_ago(node.modified);
                    let score = node.interest.as_ref().map(|i| i.score).unwrap_or(0.0);
                    writeln!(
                        writer,
                        "  {} [{}] {:.0}%",
                        path,
                        time,
                        score * 100.0
                    )?;
                }
                writeln!(writer)?;
            }
        }

        if let Some(important) = groups.get(&InterestLevel::Important) {
            if !important.is_empty() {
                writeln!(
                    writer,
                    "{}{}IMPORTANT{}",
                    self.section_emoji("important"),
                    self.level_color(InterestLevel::Important),
                    self.reset()
                )?;

                for node in important.iter().take(10) {
                    let path = self.format_path(&node.path, root_path);
                    let time = self.time_ago(node.modified);
                    writeln!(writer, "  {} [{}]", path, time)?;
                }

                if important.len() > 10 {
                    writeln!(
                        writer,
                        "  {}... and {} more{}",
                        self.level_color(InterestLevel::Background),
                        important.len() - 10,
                        self.reset()
                    )?;
                }
                writeln!(writer)?;
            }
        }

        // === NOTABLE SECTION ===
        if let Some(notable) = groups.get(&InterestLevel::Notable) {
            if !notable.is_empty() && self.min_level <= InterestLevel::Notable {
                writeln!(
                    writer,
                    "{}{}NOTABLE ({}){}",
                    self.section_emoji("notable"),
                    self.level_color(InterestLevel::Notable),
                    notable.len(),
                    self.reset()
                )?;

                for node in notable.iter().take(5) {
                    let path = self.format_path(&node.path, root_path);
                    writeln!(writer, "  {}", path)?;
                }

                if notable.len() > 5 {
                    writeln!(
                        writer,
                        "  {}... and {} more{}",
                        self.level_color(InterestLevel::Background),
                        notable.len() - 5,
                        self.reset()
                    )?;
                }
                writeln!(writer)?;
            }
        }

        // === BACKGROUND SUMMARY ===
        let background_count = groups
            .get(&InterestLevel::Background)
            .map(|v| v.len())
            .unwrap_or(0)
            + groups
                .get(&InterestLevel::Boring)
                .map(|v| v.len())
                .unwrap_or(0);

        if background_count > 0 {
            writeln!(
                writer,
                "{}{}BACKGROUND: {} files, {} dirs ({}){}",
                self.section_emoji("background"),
                self.level_color(InterestLevel::Background),
                stats.total_files,
                stats.total_dirs,
                humansize::format_size(stats.total_size, humansize::BINARY),
                self.reset()
            )?;
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::scanner::{FileCategory, FileType, FilesystemType};
    use std::path::PathBuf;

    fn make_test_node(path: &str, is_dir: bool) -> FileNode {
        FileNode {
            path: PathBuf::from(path),
            is_dir,
            size: 1000,
            permissions: 0o644,
            uid: 1000,
            gid: 1000,
            modified: SystemTime::now(),
            is_symlink: false,
            is_hidden: false,
            permission_denied: false,
            is_ignored: false,
            depth: path.matches('/').count(),
            file_type: if is_dir {
                FileType::Directory
            } else {
                FileType::RegularFile
            },
            category: FileCategory::Unknown,
            search_matches: None,
            filesystem_type: FilesystemType::Unknown,
            git_branch: None,
            traversal_context: None,
            interest: None,
            security_findings: Vec::new(),
            change_status: None,
            content_hash: None,
        }
    }

    #[test]
    fn test_smart_formatter_basic() {
        let formatter = SmartFormatter::new(false, false);
        let nodes = vec![
            make_test_node("src", true),
            make_test_node("src/main.rs", false),
            make_test_node("Cargo.toml", false),
        ];

        let stats = TreeStats {
            total_files: 2,
            total_dirs: 1,
            total_size: 2000,
            file_types: std::collections::HashMap::new(),
            largest_files: vec![],
            newest_files: vec![],
            oldest_files: vec![],
        };

        let mut output = Vec::new();
        formatter
            .format(&mut output, &nodes, &stats, Path::new("/project"))
            .unwrap();

        let output_str = String::from_utf8(output).unwrap();
        assert!(output_str.contains("BACKGROUND"));
    }

    #[test]
    fn test_time_ago() {
        let formatter = SmartFormatter::new(false, false);

        let now = SystemTime::now();
        assert_eq!(formatter.time_ago(now), "just now");

        let hour_ago = now - std::time::Duration::from_secs(3600);
        assert_eq!(formatter.time_ago(hour_ago), "1h ago");

        let day_ago = now - std::time::Duration::from_secs(86400);
        assert_eq!(formatter.time_ago(day_ago), "1d ago");
    }
}
