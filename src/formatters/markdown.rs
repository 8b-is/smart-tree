// -----------------------------------------------------------------------------
// MARKDOWN FORMATTER - The Ultimate Documentation Generator! 📝✨
//
// This formatter creates beautiful markdown reports with:
// - Mermaid diagrams (flowchart, pie charts)
// - Tables with statistics
// - File type breakdowns
// - Size analysis
// - Everything you need for instant documentation!
//
// "Making documentation so beautiful, even Trisha cries tears of joy!"
// - Trisha from Accounting
//
// Brought to you by The Cheet, turning directory trees into visual masterpieces! 🎨
// -----------------------------------------------------------------------------

use super::{Formatter, PathDisplayMode};
use crate::scanner::{FileNode, TreeStats};
use anyhow::Result;
use chrono::Local;
use std::collections::HashMap;
use std::io::Write;
use std::path::Path;

pub struct MarkdownFormatter {
    no_emoji: bool,
    include_mermaid: bool,
    include_tables: bool,
    include_pie_charts: bool,
    max_pie_slices: usize,
}

impl MarkdownFormatter {
    pub fn new(
        _path_mode: PathDisplayMode,
        no_emoji: bool,
        include_mermaid: bool,
        include_tables: bool,
        include_pie_charts: bool,
    ) -> Self {
        Self {
            no_emoji,
            include_mermaid,
            include_tables,
            include_pie_charts,
            max_pie_slices: 10, // Limit pie chart slices for readability
        }
    }

    fn escape_mermaid(text: &str) -> String {
        text.replace('|', "&#124;")
            .replace('<', "&lt;")
            .replace('>', "&gt;")
            .replace('"', "&quot;")
            .replace('\'', "&#39;")
            .replace('[', "&#91;")
            .replace(']', "&#93;")
            .replace('{', "&#123;")
            .replace('}', "&#125;")
            .replace('(', "&#40;")
            .replace(')', "&#41;")
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

    fn escape_table_cell(text: &str) -> String {
        text.chars()
            .map(|ch| match ch {
                '&' => "&amp;".to_string(),
                '<' => "&lt;".to_string(),
                '>' => "&gt;".to_string(),
                '|' => "&#124;".to_string(),
                '`' => "&#96;".to_string(),
                '\\' => "&#92;".to_string(),
                '*' => "&#42;".to_string(),
                '_' => "&#95;".to_string(),
                '[' => "&#91;".to_string(),
                ']' => "&#93;".to_string(),
                '\n' => "<br>".to_string(),
                ch if ch.is_control() => " ".to_string(),
                ch => ch.to_string(),
            })
            .collect()
    }

    fn write_concerning_files(
        &self,
        writer: &mut dyn Write,
        nodes: &[FileNode],
        root_path: &Path,
    ) -> Result<()> {
        writeln!(
            writer,
            "## {}Concerning Files\n",
            if self.no_emoji { "" } else { "⚠️ " }
        )?;
        let mut findings: Vec<_> = nodes
            .iter()
            .flat_map(|node| &node.security_findings)
            .collect();
        findings.sort_by(|a, b| {
            b.risk_level
                .cmp(&a.risk_level)
                .then_with(|| a.file_path.cmp(&b.file_path))
                .then_with(|| a.line_number.cmp(&b.line_number))
                .then_with(|| a.pattern_name.cmp(&b.pattern_name))
        });
        if findings.is_empty() {
            writeln!(
                writer,
                "No security findings were reported for the displayed entries.\n"
            )?;
        } else {
            writeln!(
                writer,
                "{} finding(s) in the displayed entries, highest risk first.\n",
                findings.len()
            )?;
            writeln!(writer, "| Risk | File | Line | Pattern | Concern |")?;
            writeln!(writer, "|------|------|------|---------|---------|")?;
            for finding in findings {
                let path = finding
                    .file_path
                    .strip_prefix(root_path)
                    .unwrap_or(&finding.file_path);
                writeln!(
                    writer,
                    "| {} | {} | {} | {} | {} |",
                    finding.risk_level,
                    Self::escape_table_cell(&path.to_string_lossy()),
                    finding.line_number,
                    Self::escape_table_cell(&finding.pattern_name),
                    Self::escape_table_cell(&finding.description),
                )?;
            }
            writeln!(writer)?;
        }
        writeln!(writer, "Coverage follows the tree scan's filters and security settings. Use `st --integrity-scan PATH` for deeper analysis or `st --cert-scan PATH` to inspect embedded certificates.\n")?;
        Ok(())
    }

    fn get_file_emoji(&self, path: &Path, is_dir: bool) -> &'static str {
        if self.no_emoji {
            return "";
        }

        if is_dir {
            "📁"
        } else {
            match path.extension().and_then(|e| e.to_str()) {
                Some("rs") => "🦀",
                Some("py") => "🐍",
                Some("js") | Some("ts") => "📜",
                Some("md") => "📝",
                Some("toml") | Some("yaml") | Some("yml") | Some("json") => "⚙️",
                Some("png") | Some("jpg") | Some("jpeg") | Some("gif") => "🖼️",
                Some("pdf") => "📕",
                Some("zip") | Some("tar") | Some("gz") => "📦",
                Some("mp3") | Some("wav") | Some("flac") => "🎵",
                Some("mp4") | Some("avi") | Some("mov") => "🎬",
                _ => "📄",
            }
        }
    }

    fn write_header(
        &self,
        writer: &mut dyn Write,
        root_path: &Path,
        stats: &TreeStats,
    ) -> Result<()> {
        let timestamp = Local::now().format("%Y-%m-%d %H:%M:%S");
        let root_name = root_path
            .file_name()
            .unwrap_or(root_path.as_os_str())
            .to_string_lossy();

        writeln!(writer, "# 📊 Directory Analysis Report")?;
        writeln!(writer)?;
        writeln!(writer, "**Generated by Smart Tree** | {}", timestamp)?;
        writeln!(writer)?;
        writeln!(writer, "## 📁 Overview")?;
        writeln!(writer)?;
        writeln!(writer, "- **Directory**: `{}`", root_name)?;
        writeln!(writer, "- **Total Files**: {}", stats.total_files)?;
        writeln!(writer, "- **Total Directories**: {}", stats.total_dirs)?;
        writeln!(
            writer,
            "- **Total Size**: {}",
            Self::format_size(stats.total_size)
        )?;
        writeln!(writer)?;

        Ok(())
    }

    fn write_mermaid_diagram(
        &self,
        writer: &mut dyn Write,
        nodes: &[FileNode],
        root_path: &Path,
    ) -> Result<()> {
        writeln!(writer, "## 🌳 Directory Structure")?;
        writeln!(writer)?;
        writeln!(writer, "```mermaid")?;
        writeln!(writer, "flowchart LR")?; // Use Left-Right for better readability
        writeln!(writer, "    %% Smart Tree Directory Visualization")?;

        // Limit nodes for readability
        let max_nodes = 40; // Reduced for cleaner diagrams
        let display_nodes: Vec<&FileNode> = nodes.iter().take(max_nodes).collect();

        // Write root node with better styling
        let root_name = root_path
            .file_name()
            .unwrap_or(root_path.as_os_str())
            .to_string_lossy();
        writeln!(
            writer,
            "    root{{\"📁 {}\"}}",
            Self::escape_mermaid(&root_name)
        )?;
        writeln!(
            writer,
            "    style root fill:#ff9800,stroke:#e65100,stroke-width:4px,color:#fff"
        )?;
        writeln!(writer)?;

        // Group nodes by parent directory for cleaner visualization
        let mut dir_groups: HashMap<std::path::PathBuf, Vec<&FileNode>> = HashMap::new();
        let mut all_dirs: Vec<std::path::PathBuf> = Vec::new();

        for node in &display_nodes {
            // Skip the root directory itself
            if node.path == *root_path {
                continue;
            }

            if let Some(parent) = node.path.parent() {
                dir_groups
                    .entry(parent.to_path_buf())
                    .or_default()
                    .push(node);
                if node.is_dir && !all_dirs.contains(&node.path) {
                    all_dirs.push(node.path.clone());
                }
            }
        }

        // Write subgraphs for each directory with children
        let mut subgraph_count = 0;
        for (parent, children) in &dir_groups {
            if children.len() > 1 && parent != root_path {
                subgraph_count += 1;
                let parent_name = parent.file_name().and_then(|n| n.to_str()).unwrap_or("?");
                writeln!(
                    writer,
                    "    subgraph sub{} [\"{}\" ]",
                    subgraph_count,
                    Self::escape_mermaid(parent_name)
                )?;
                writeln!(writer, "        direction TB")?;

                for child in children {
                    let node_id = format!(
                        "node_{}",
                        display_nodes
                            .iter()
                            .position(|n| n.path == child.path)
                            .unwrap_or(0)
                    );
                    let name = child
                        .path
                        .file_name()
                        .and_then(|n| n.to_str())
                        .unwrap_or("?");
                    let emoji = self.get_file_emoji(&child.path, child.is_dir);

                    if child.is_dir {
                        writeln!(
                            writer,
                            "        {}[\"📁 {}\"]",
                            node_id,
                            Self::escape_mermaid(name)
                        )?;
                    } else {
                        let size_str = Self::format_size(child.size);
                        writeln!(
                            writer,
                            "        {}[\"{}{}\\n{}\"]",
                            node_id,
                            emoji,
                            Self::escape_mermaid(name),
                            size_str
                        )?;
                    }
                }
                writeln!(writer, "    end")?;
                writeln!(writer)?;
            }
        }

        // Write remaining nodes (not in subgraphs)
        for (i, node) in display_nodes.iter().enumerate() {
            // Skip the root directory itself
            if node.path == *root_path {
                continue;
            }

            let parent = node.path.parent();
            let in_subgraph = parent
                .map(|p| dir_groups.get(p).map(|c| c.len() > 1).unwrap_or(false))
                .unwrap_or(false);

            if !in_subgraph || parent == Some(root_path) {
                let node_id = format!("node_{}", i);
                let name = node
                    .path
                    .file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or("?");
                let emoji = self.get_file_emoji(&node.path, node.is_dir);

                if node.is_dir {
                    writeln!(
                        writer,
                        "    {}[\"📁 {}\"]",
                        node_id,
                        Self::escape_mermaid(name)
                    )?;
                    writeln!(
                        writer,
                        "    style {} fill:#e3f2fd,stroke:#1976d2,stroke-width:2px",
                        node_id
                    )?;
                } else {
                    let size_str = Self::format_size(node.size);
                    writeln!(
                        writer,
                        "    {}[\"{}{}\\n{}\"]",
                        node_id,
                        emoji,
                        Self::escape_mermaid(name),
                        size_str
                    )?;

                    // Style based on file type
                    match node.path.extension().and_then(|e| e.to_str()) {
                        Some("rs") => {
                            writeln!(writer, "    style {} fill:#dcedc8,stroke:#689f38", node_id)?
                        }
                        Some("md") => {
                            writeln!(writer, "    style {} fill:#fff9c4,stroke:#f9a825", node_id)?
                        }
                        Some("json") | Some("toml") | Some("yaml") => {
                            writeln!(writer, "    style {} fill:#f3e5f5,stroke:#7b1fa2", node_id)?
                        }
                        _ => writeln!(writer, "    style {} fill:#f5f5f5,stroke:#616161", node_id)?,
                    }
                }
            }
        }

        // Simplified connections
        writeln!(writer)?;
        writeln!(writer, "    %% Connections")?;

        // Connect root to immediate children
        for node in &display_nodes {
            if let Some(parent) = node.path.parent() {
                if parent == root_path {
                    let node_id = format!(
                        "node_{}",
                        display_nodes
                            .iter()
                            .position(|n| n.path == node.path)
                            .unwrap_or(0)
                    );
                    writeln!(writer, "    root --> {}", node_id)?;
                }
            }
        }

        // Connect directories to their subgraphs
        let mut connected_subgraphs = std::collections::HashSet::new();
        let mut subgraph_map = HashMap::new();
        let mut current_sub = 0;

        // Build a map of directories to subgraph numbers
        for (parent, children) in &dir_groups {
            if children.len() > 1 && parent != root_path {
                current_sub += 1;
                subgraph_map.insert(parent.clone(), current_sub);
            }
        }

        // Connect parent directories to their subgraphs
        for node in &display_nodes {
            if node.is_dir {
                if let Some(&sub_num) = subgraph_map.get(&node.path) {
                    if !connected_subgraphs.contains(&sub_num) {
                        // Find parent of this directory
                        if let Some(parent) = node.path.parent() {
                            if parent == root_path {
                                writeln!(writer, "    root --> sub{}", sub_num)?;
                            } else {
                                // Find parent node id
                                if let Some(parent_idx) =
                                    display_nodes.iter().position(|n| n.path == *parent)
                                {
                                    writeln!(writer, "    node_{} --> sub{}", parent_idx, sub_num)?;
                                }
                            }
                        }
                        connected_subgraphs.insert(sub_num);
                    }
                }
            }
        }

        if nodes.len() > max_nodes {
            writeln!(writer)?;
            writeln!(
                writer,
                "    more[\"... and {} more items\"]",
                nodes.len() - max_nodes
            )?;
            writeln!(
                writer,
                "    style more fill:#ffecb3,stroke:#ff6f00,stroke-dasharray: 5 5"
            )?;
        }

        writeln!(writer, "```")?;
        writeln!(writer)?;

        // Add a simple text tree as fallback
        writeln!(writer, "### 📂 Simple Tree View")?;
        writeln!(writer)?;
        writeln!(writer, "```")?;

        let root_name = root_path
            .file_name()
            .unwrap_or(root_path.as_os_str())
            .to_string_lossy();
        writeln!(
            writer,
            "{} {}/",
            if !self.no_emoji { "📁" } else { "" },
            root_name
        )?;

        // Sort nodes by path for consistent output
        let mut sorted_nodes = display_nodes.clone();
        sorted_nodes.sort_by_key(|n| &n.path);

        for (i, node) in sorted_nodes.iter().enumerate() {
            if node.path == *root_path {
                continue;
            }

            let depth = node.path.components().count() - root_path.components().count();
            if depth > 2 {
                continue; // Only show 2 levels in simple view
            }

            let is_last = i == sorted_nodes.len() - 1
                || sorted_nodes
                    .get(i + 1)
                    .map(|next| {
                        let next_depth =
                            next.path.components().count() - root_path.components().count();
                        next_depth < depth
                    })
                    .unwrap_or(true);

            let indent = if depth > 0 {
                "│   ".repeat(depth - 1)
            } else {
                String::new()
            };

            let prefix = if is_last { "└── " } else { "├── " };
            let emoji = self.get_file_emoji(&node.path, node.is_dir);
            let name = node
                .path
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("?");

            if node.is_dir {
                writeln!(writer, "{}{}{} {}/", indent, prefix, emoji, name)?;
            } else {
                writeln!(
                    writer,
                    "{}{}{} {} ({})",
                    indent,
                    prefix,
                    emoji,
                    name,
                    Self::format_size(node.size)
                )?;
            }
        }

        if nodes.len() > max_nodes {
            writeln!(
                writer,
                "│   └── ... and {} more items",
                nodes.len() - max_nodes
            )?;
        }

        writeln!(writer, "```")?;
        writeln!(writer)?;

        Ok(())
    }

    fn write_file_type_table(&self, writer: &mut dyn Write, stats: &TreeStats) -> Result<()> {
        writeln!(writer, "## 📋 File Types Breakdown")?;
        writeln!(writer)?;
        writeln!(writer, "| Extension | Count | Percentage | Total Size |")?;
        writeln!(writer, "|-----------|-------|------------|------------|")?;

        let total_files = stats.total_files as f64;

        for (ext, count) in stats.file_types.iter().take(20) {
            let percentage = (*count as f64 / total_files) * 100.0;
            let emoji = match ext.as_str() {
                "rs" => "🦀",
                "py" => "🐍",
                "js" | "ts" => "📜",
                "md" => "📝",
                "json" | "yaml" | "yml" | "toml" => "⚙️",
                _ => "📄",
            };

            writeln!(
                writer,
                "| {} .{} | {} | {:.1}% | - |",
                if self.no_emoji { "" } else { emoji },
                ext,
                count,
                percentage
            )?;
        }

        writeln!(writer)?;
        Ok(())
    }

    fn write_size_distribution_pie(
        &self,
        writer: &mut dyn Write,
        nodes: &[FileNode],
    ) -> Result<()> {
        writeln!(writer, "## 📊 Size Distribution")?;
        writeln!(writer)?;

        let mut counts = [0usize; 5];
        for node in nodes.iter().filter(|node| !node.is_dir) {
            let bucket = match node.size {
                0..=1023 => 0,
                1024..=10239 => 1,
                10240..=102399 => 2,
                102400..=1048575 => 3,
                _ => 4,
            };
            counts[bucket] += 1;
        }
        writeln!(writer, "```mermaid")?;
        writeln!(writer, "pie title File Size Distribution")?;
        for (label, count) in ["< 1 KB", "1-10 KB", "10-100 KB", "100 KB - 1 MB", ">= 1 MB"]
            .iter()
            .zip(counts)
        {
            writeln!(writer, "    \"{}\" : {}", label, count)?;
        }
        writeln!(writer, "```")?;
        writeln!(writer)?;

        Ok(())
    }

    fn write_file_type_pie(&self, writer: &mut dyn Write, stats: &TreeStats) -> Result<()> {
        writeln!(writer, "## 🍰 File Type Distribution")?;
        writeln!(writer)?;
        writeln!(writer, "```mermaid")?;
        writeln!(writer, "pie title Files by Type")?;

        let mut other_count = 0;
        let mut shown_types = 0;

        for (ext, count) in &stats.file_types {
            if shown_types < self.max_pie_slices {
                writeln!(writer, "    \"{}\" : {}", ext, count)?;
                shown_types += 1;
            } else {
                other_count += count;
            }
        }

        if other_count > 0 {
            writeln!(writer, "    \"Other\" : {}", other_count)?;
        }

        writeln!(writer, "```")?;
        writeln!(writer)?;

        Ok(())
    }

    fn write_largest_files_table(&self, writer: &mut dyn Write, stats: &TreeStats) -> Result<()> {
        writeln!(writer, "## 🏆 Largest Files")?;
        writeln!(writer)?;
        writeln!(writer, "| Rank | File | Size |")?;
        writeln!(writer, "|------|------|------|")?;

        for (i, (size, path)) in stats.largest_files.iter().enumerate() {
            let name = path.file_name().and_then(|n| n.to_str()).unwrap_or("?");
            let emoji = self.get_file_emoji(path, false);

            writeln!(
                writer,
                "| {} | {} {} | {} |",
                match i {
                    0 => "🥇",
                    1 => "🥈",
                    2 => "🥉",
                    _ => "📄",
                },
                emoji,
                name,
                Self::format_size(*size)
            )?;
        }

        writeln!(writer)?;
        Ok(())
    }

    fn write_recent_files_table(&self, writer: &mut dyn Write, stats: &TreeStats) -> Result<()> {
        if stats.newest_files.is_empty() {
            return Ok(());
        }

        writeln!(writer, "## 🕐 Recent Activity")?;
        writeln!(writer)?;
        writeln!(writer, "| File | Last Modified |")?;
        writeln!(writer, "|------|---------------|")?;

        for (timestamp, path) in stats.newest_files.iter().take(10) {
            let name = path.file_name().and_then(|n| n.to_str()).unwrap_or("?");
            let emoji = self.get_file_emoji(path, false);

            if let Ok(duration) = std::time::SystemTime::now().duration_since(*timestamp) {
                let days = duration.as_secs() / 86400;
                let time_str = if days == 0 {
                    "Today".to_string()
                } else if days == 1 {
                    "Yesterday".to_string()
                } else if days < 7 {
                    format!("{} days ago", days)
                } else if days < 30 {
                    format!("{} weeks ago", days / 7)
                } else {
                    format!("{} months ago", days / 30)
                };

                writeln!(writer, "| {} {} | {} |", emoji, name, time_str)?;
            }
        }

        writeln!(writer)?;
        Ok(())
    }

    fn write_summary(&self, writer: &mut dyn Write, _stats: &TreeStats) -> Result<()> {
        writeln!(writer, "## 📈 Summary")?;
        writeln!(writer)?;

        if !self.no_emoji {
            writeln!(writer, "This analysis brought to you by **Smart Tree** 🌳")?;
            writeln!(
                writer,
                "Where directories become beautiful documentation! ✨"
            )?;
        } else {
            writeln!(writer, "This analysis brought to you by **Smart Tree**")?;
            writeln!(writer, "Where directories become beautiful documentation!")?;
        }

        writeln!(writer)?;
        writeln!(writer, "---")?;
        writeln!(writer)?;
        writeln!(writer, "**Generated with [Smart Tree](https://github.com/8b-is/smart-tree/) - Making directory visualization intelligent, fast, and beautiful!** ")?;

        Ok(())
    }
}

impl Formatter for MarkdownFormatter {
    fn format(
        &self,
        writer: &mut dyn Write,
        nodes: &[FileNode],
        stats: &TreeStats,
        root_path: &Path,
    ) -> Result<()> {
        // Header with overview
        self.write_header(writer, root_path, stats)?;
        self.write_concerning_files(writer, nodes, root_path)?;

        // Mermaid directory diagram
        if self.include_mermaid {
            self.write_mermaid_diagram(writer, nodes, root_path)?;
        }

        // File types table
        if self.include_tables && !stats.file_types.is_empty() {
            self.write_file_type_table(writer, stats)?;
        }

        // Pie charts
        if self.include_pie_charts {
            if !stats.file_types.is_empty() {
                self.write_file_type_pie(writer, stats)?;
            }
            self.write_size_distribution_pie(writer, nodes)?;
        }

        // Largest files
        if self.include_tables && !stats.largest_files.is_empty() {
            self.write_largest_files_table(writer, stats)?;
        }

        // Recent activity
        if self.include_tables && !stats.newest_files.is_empty() {
            self.write_recent_files_table(writer, stats)?;
        }

        // Summary
        self.write_summary(writer, stats)?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::scanner::{FileCategory, FileNode, FileType, FilesystemType, TreeStats};
    use std::path::PathBuf;
    use std::time::SystemTime;

    fn test_node(path: &str) -> FileNode {
        FileNode {
            path: PathBuf::from(path),
            is_dir: false,
            size: 0,
            permissions: 0o644,
            uid: 0,
            gid: 0,
            modified: SystemTime::UNIX_EPOCH,
            is_symlink: false,
            is_hidden: false,
            permission_denied: false,
            is_ignored: false,
            depth: 1,
            file_type: FileType::RegularFile,
            category: FileCategory::Unknown,
            search_matches: None,
            filesystem_type: FilesystemType::Unknown,
            git_branch: None,
            traversal_context: None,
            interest: None,
            security_findings: vec![],
            change_status: None,
            content_hash: None,
        }
    }

    #[test]
    fn concerning_files_are_sorted_escaped_and_omit_source_content() {
        use crate::security_scan::{RiskLevel, SecurityFinding};
        let mut node = test_node("/project/odd|`<script>\n.rs");
        for risk in [RiskLevel::Low, RiskLevel::Critical] {
            node.security_findings.push(SecurityFinding {
                file_path: node.path.clone(),
                line_number: 7,
                pattern_name: "[pattern]|name".to_string(),
                matched_text: "secret-source-content".to_string(),
                risk_level: risk,
                description: "Review <code> & config".to_string(),
            });
        }
        let formatter = MarkdownFormatter::new(PathDisplayMode::Off, true, false, false, false);
        let mut output = Vec::new();
        formatter
            .format(
                &mut output,
                &[node],
                &TreeStats::default(),
                Path::new("/project"),
            )
            .unwrap();
        let output = String::from_utf8(output).unwrap();
        assert!(output.contains("## Concerning Files"));
        assert!(output.find("| CRITICAL |").unwrap() < output.find("| LOW |").unwrap());
        assert!(output.contains("odd&#124;&#96;&lt;script&gt;<br>.rs"));
        assert!(output.contains("&#91;pattern&#93;&#124;name"));
        assert!(!output.contains("secret-source-content"));
        assert!(!output.contains("/project/"));
    }

    #[test]
    fn empty_concerns_describe_scan_coverage_and_offer_certificate_scan() {
        let formatter = MarkdownFormatter::new(PathDisplayMode::Off, true, false, true, false);
        let mut output = Vec::new();
        formatter
            .write_concerning_files(&mut output, &[], Path::new("."))
            .unwrap();
        let output = String::from_utf8(output).unwrap();
        assert!(output.contains("No security findings were reported"));
        assert!(output.contains("filters and security settings"));
        assert!(output.contains("st --cert-scan PATH"));
    }

    #[test]
    fn size_chart_counts_scanned_files_at_bucket_boundaries() {
        let mut nodes: Vec<_> = [
            0, 1023, 1024, 10239, 10240, 102399, 102400, 1048575, 1048576,
        ]
        .iter()
        .map(|&size| {
            let mut node = test_node("file");
            node.size = size;
            node
        })
        .collect();
        let mut directory = test_node("dir");
        directory.is_dir = true;
        nodes.push(directory);
        let formatter = MarkdownFormatter::new(PathDisplayMode::Off, true, false, true, true);
        let mut output = Vec::new();
        formatter
            .write_size_distribution_pie(&mut output, &nodes)
            .unwrap();
        let output = String::from_utf8(output).unwrap();
        assert!(output.contains("\"< 1 KB\" : 2"));
        assert!(output.contains("\"1-10 KB\" : 2"));
        assert!(output.contains("\"10-100 KB\" : 2"));
        assert!(output.contains("\"100 KB - 1 MB\" : 2"));
        assert!(output.contains("\">= 1 MB\" : 1"));
    }

    #[test]
    fn test_markdown_formatter() {
        let formatter = MarkdownFormatter::new(
            PathDisplayMode::Off,
            false,
            true, // include_mermaid
            true, // include_tables
            true, // include_pie_charts
        );

        let nodes = vec![FileNode {
            path: PathBuf::from("src"),
            is_dir: true,
            size: 0,
            permissions: 0o755,
            uid: 1000,
            gid: 1000,
            modified: SystemTime::now(),
            is_symlink: false,
            is_ignored: false,
            search_matches: None,
            // ---- Fields added to fix compilation ----
            is_hidden: false,
            permission_denied: false,
            depth: 1,
            file_type: FileType::Directory,
            category: FileCategory::Unknown,
            filesystem_type: FilesystemType::Unknown,
            git_branch: None,
            traversal_context: None,
            interest: None,
            security_findings: Vec::new(),
            change_status: None,
            content_hash: None,
        }];

        let mut stats = TreeStats::default();
        stats.update_file(&nodes[0]);

        let mut output = Vec::new();
        let result = formatter.format(&mut output, &nodes, &stats, &PathBuf::from("."));
        assert!(result.is_ok());

        let output_str = String::from_utf8(output).unwrap();
        assert!(output_str.contains("# 📊 Directory Analysis Report"));
        assert!(output_str.contains("mermaid"));
    }
}
