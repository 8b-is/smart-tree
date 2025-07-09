// -----------------------------------------------------------------------------
// MERMAID FORMATTER - Making directory trees documentation-ready! 🧜‍♀️
//
// This formatter outputs directory structures as Mermaid diagrams, perfect for
// embedding in markdown documentation, GitHub READMEs, and wikis!
//
// "Every diagram tells a story" - Trisha from Accounting
//
// Brought to you by The Cheet, making documentation as beautiful as it is useful! 📊✨
// -----------------------------------------------------------------------------

use super::{Formatter, PathDisplayMode};
use crate::scanner::{FileNode, TreeStats};
use anyhow::Result;
use std::collections::HashMap;
use std::io::Write;

pub struct MermaidFormatter {
    style: MermaidStyle,
    no_emoji: bool,
    path_mode: PathDisplayMode,
    max_label_length: usize,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum MermaidStyle {
    Flowchart, // Traditional flowchart style (TD/LR)
    Mindmap,   // Mind map style (great for overviews)
    GitGraph,  // Git-like graph (good for showing relationships)
    Treemap,   // Treemap style (perfect for showing sizes!)
}

impl MermaidFormatter {
    pub fn new(style: MermaidStyle, no_emoji: bool, path_mode: PathDisplayMode) -> Self {
        Self {
            style,
            no_emoji,
            path_mode,
            max_label_length: 50, // Prevent overly long labels
        }
    }

    fn sanitize_node_id(path: &std::path::Path) -> String {
        // Create safe node IDs for Mermaid
        let path_str = path.to_string_lossy();
        // Replace problematic characters
        path_str.replace(
            [
                '/', '\\', '.', ' ', '-', '(', ')', '[', ']', '{', '}', ':', ';', ',', '\'', '"',
                '`', '~', '!', '@', '#', '$', '%', '^', '&', '*', '=', '+', '|', '<', '>', '?',
            ],
            "_",
        )
    }

    fn escape_label(text: &str) -> String {
        // Escape special characters that might break Mermaid syntax
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

    fn format_label(&self, node: &FileNode) -> String {
        let name = match self.path_mode {
            PathDisplayMode::Off => node
                .path
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("?")
                .to_string(),
            PathDisplayMode::Relative | PathDisplayMode::Full => {
                node.path.to_string_lossy().to_string()
            }
        };

        // Add emoji if enabled
        let emoji = if !self.no_emoji {
            if node.is_dir {
                "📁 "
            } else {
                match node.path.extension().and_then(|e| e.to_str()) {
                    Some("rs") => "🦀 ",
                    Some("py") => "🐍 ",
                    Some("js") | Some("ts") => "📜 ",
                    Some("md") => "📝 ",
                    Some("toml") | Some("yaml") | Some("yml") | Some("json") => "⚙️ ",
                    Some("png") | Some("jpg") | Some("jpeg") | Some("gif") => "🖼️ ",
                    _ => "📄 ",
                }
            }
        } else {
            ""
        };

        // Escape the name for Mermaid
        let escaped_name = Self::escape_label(&name);

        // Truncate if too long
        let mut label = format!("{}{}", emoji, escaped_name);
        if label.len() > self.max_label_length {
            label.truncate(self.max_label_length - 3);
            label.push_str("...");
        }

        // Add size for files
        if !node.is_dir && node.size > 0 {
            label.push_str(&format!("<br/>{}", format_size(node.size)));
        }

        label
    }

    fn write_flowchart(
        &self,
        writer: &mut dyn Write,
        nodes: &[FileNode],
        root_path: &std::path::Path,
    ) -> Result<()> {
        writeln!(writer, "```mermaid")?;
        writeln!(writer, "graph TD")?;
        writeln!(writer, "    %% Smart Tree Directory Structure")?;
        writeln!(writer)?;

        // Build parent-child relationships
        let mut parent_map: HashMap<String, Vec<&FileNode>> = HashMap::new();
        let root_id = Self::sanitize_node_id(root_path);

        for node in nodes {
            let _node_id = Self::sanitize_node_id(&node.path);

            // Find parent
            if let Some(parent_path) = node.path.parent() {
                let parent_id = if parent_path == root_path {
                    root_id.clone()
                } else {
                    Self::sanitize_node_id(parent_path)
                };

                parent_map.entry(parent_id).or_default().push(node);
            }
        }

        // Write root node with emoji handling
        let root_emoji = if !self.no_emoji { "📁 " } else { "" };
        let root_name = root_path
            .file_name()
            .unwrap_or(root_path.as_os_str())
            .to_string_lossy();
        let escaped_root_name = Self::escape_label(&root_name);
        writeln!(
            writer,
            "    {}[\"{}{}\"]",
            root_id, root_emoji, escaped_root_name
        )?;

        // Write all nodes and connections
        for node in nodes {
            let node_id = Self::sanitize_node_id(&node.path);
            let label = self.format_label(node);

            // Determine node shape based on type
            let (open_shape, close_shape) = if node.is_dir {
                ("[\"", "\"]") // Rectangle for directories - use quotes to handle emojis
            } else {
                match node.path.extension().and_then(|e| e.to_str()) {
                    Some("md") | Some("txt") | Some("rst") => ("([\"", "\"])"), // Stadium for docs
                    Some("rs") | Some("py") | Some("js") | Some("ts") => ("{{\"", "\"}}"), // Hexagon for code
                    Some("toml") | Some("yaml") | Some("yml") | Some("json") => ("[\"", "\"]"), // Rectangle for config (simpler than cylinder)
                    _ => ("[\"", "\"]"), // Rectangle for other files (safer than circles)
                }
            };

            writeln!(
                writer,
                "    {}{}{}{}",
                node_id, open_shape, label, close_shape
            )?;

            // Connect to parent
            if let Some(parent_path) = node.path.parent() {
                let parent_id = if parent_path == root_path {
                    root_id.clone()
                } else {
                    Self::sanitize_node_id(parent_path)
                };

                writeln!(writer, "    {} --> {}", parent_id, node_id)?;
            }
        }

        // Add styling
        writeln!(writer)?;
        writeln!(writer, "    %% Styling")?;
        writeln!(
            writer,
            "    classDef dirStyle fill:#e1f5fe,stroke:#01579b,stroke-width:2px"
        )?;
        writeln!(
            writer,
            "    classDef codeStyle fill:#e8f5e9,stroke:#1b5e20,stroke-width:2px"
        )?;
        writeln!(
            writer,
            "    classDef docStyle fill:#fff3e0,stroke:#e65100,stroke-width:2px"
        )?;
        writeln!(
            writer,
            "    classDef configStyle fill:#fce4ec,stroke:#880e4f,stroke-width:2px"
        )?;

        // Apply styles
        for node in nodes {
            let node_id = Self::sanitize_node_id(&node.path);
            if node.is_dir {
                writeln!(writer, "    class {} dirStyle", node_id)?;
            } else {
                match node.path.extension().and_then(|e| e.to_str()) {
                    Some("rs") | Some("py") | Some("js") | Some("ts") => {
                        writeln!(writer, "    class {} codeStyle", node_id)?;
                    }
                    Some("md") | Some("txt") | Some("rst") => {
                        writeln!(writer, "    class {} docStyle", node_id)?;
                    }
                    Some("toml") | Some("yaml") | Some("yml") | Some("json") => {
                        writeln!(writer, "    class {} configStyle", node_id)?;
                    }
                    _ => {}
                }
            }
        }

        writeln!(writer, "```")?;
        Ok(())
    }

    fn write_mindmap(
        &self,
        writer: &mut dyn Write,
        nodes: &[FileNode],
        root_path: &std::path::Path,
    ) -> Result<()> {
        writeln!(writer, "```mermaid")?;
        writeln!(writer, "mindmap")?;
        let root_name = root_path
            .file_name()
            .unwrap_or(root_path.as_os_str())
            .to_string_lossy();
        let escaped_root_name = Self::escape_label(&root_name);
        let root_emoji = if !self.no_emoji { "📁 " } else { "" };
        writeln!(writer, "  root(({}{}))", root_emoji, escaped_root_name)?;

        // Build tree structure
        let _current_depth = 0;
        let _depth_stack = [root_path.to_path_buf()];

        for node in nodes {
            // Calculate depth
            let depth = node.path.components().count() - root_path.components().count();

            // Adjust indentation
            let indent = "    ".repeat(depth + 1);
            let label = self.format_label(node);

            writeln!(writer, "{}{}", indent, label)?;
        }

        writeln!(writer, "```")?;
        Ok(())
    }

    fn write_gitgraph(
        &self,
        writer: &mut dyn Write,
        nodes: &[FileNode],
        _root_path: &std::path::Path,
    ) -> Result<()> {
        writeln!(writer, "```mermaid")?;
        writeln!(writer, "gitGraph")?;
        writeln!(writer, "    commit id: \"Project Root\"")?;

        // Group by directory
        let _current_branch = "main";
        let mut branch_count = 0;

        for (i, node) in nodes.iter().enumerate() {
            if node.is_dir {
                branch_count += 1;
                let branch_name = format!("dir{}", branch_count);
                writeln!(writer, "    branch {}", branch_name)?;
                writeln!(writer, "    checkout {}", branch_name)?;
                let dir_name = node.path.file_name().unwrap_or_default().to_string_lossy();
                let escaped_dir_name = Self::escape_label(&dir_name);
                writeln!(writer, "    commit id: \"{}\"", escaped_dir_name)?;
                // current_branch = &branch_name; // This was unused, so we comment it out.
            } else if i < 20 {
                // Limit to prevent overly complex graphs
                let file_name = node.path.file_name().unwrap_or_default().to_string_lossy();
                let escaped_file_name = Self::escape_label(&file_name);
                writeln!(writer, "    commit id: \"{}\"", escaped_file_name)?;
            }
        }

        writeln!(writer, "```")?;
        Ok(())
    }

    fn write_treemap(
        &self,
        writer: &mut dyn Write,
        nodes: &[FileNode],
        root_path: &std::path::Path,
    ) -> Result<()> {
        writeln!(writer, "```mermaid")?;
        writeln!(writer, "%%{{init: {{'theme':'dark'}}}}%%")?; // Dark theme looks better
        writeln!(writer, "treemap-beta")?; // Treemap is a Mermaid Beta feature.        
        // Build directory tree with sizes
        let root_name = root_path
            .file_name()
            .unwrap_or(root_path.as_os_str())
            .to_string_lossy();
        let escaped_root_name = Self::escape_label(&root_name);
        let root_emoji = if !self.no_emoji { "📁 " } else { "" };
        
        // Write in hierarchical order based on path components
        let mut current_path = vec![root_path.to_path_buf()];
        let mut current_depth = 0;
        let indent_base = "    ";
        
        writeln!(writer, "{}\"{}{}\"", indent_base, root_emoji, escaped_root_name)?;
        
        // Sort nodes by path for consistent hierarchical output
        let mut sorted_nodes = nodes.to_vec();
        sorted_nodes.sort_by_key(|n| n.path.clone());
        
        for node in &sorted_nodes {
            // Skip the root itself
            if node.path == *root_path {
                continue;
            }
            
            // Calculate the depth of this node
            let node_depth = node.path.components().count() - root_path.components().count();
            
            // Adjust current path to match this node's parent path
            while current_depth >= node_depth {
                current_path.pop();
                current_depth -= 1;
            }
            
            // Determine indent level
            let indent = indent_base.repeat(node_depth + 1);
            
            let name = node.path.file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("?");
            let escaped_name = Self::escape_label(name);
            
            if node.is_dir {
                let dir_emoji = if !self.no_emoji { "📁 " } else { "" };
                writeln!(writer, "{}\"{}{}\"", indent, dir_emoji, escaped_name)?;
                current_path.push(node.path.clone());
                current_depth = node_depth;
            } else {
                let emoji = if !self.no_emoji {
                    match node.path.extension().and_then(|e| e.to_str()) {
                        Some("rs") => "🦀 ",
                        Some("py") => "🐍 ",
                        Some("js") | Some("ts") => "📜 ",
                        Some("md") => "📝 ",
                        Some("toml") | Some("yaml") | Some("yml") | Some("json") => "⚙️ ",
                        _ => "📄 ",
                    }
                } else {
                    ""
                };
                
                // Convert size to KB for better readability in treemap
                let size_kb = (node.size as f64 / 1024.0).max(1.0) as u64;
                writeln!(writer, "{}\"{}{}\": {}", indent, emoji, escaped_name, size_kb)?;
            }
        }
        
        writeln!(writer, "```")?;
        Ok(())
    }
}

impl Formatter for MermaidFormatter {
    fn format(
        &self,
        writer: &mut dyn Write,
        nodes: &[FileNode],
        stats: &TreeStats,
        root_path: &std::path::Path,
    ) -> Result<()> {
        // Header
        writeln!(writer, "# Directory Structure Diagram")?;
        writeln!(writer)?;
        writeln!(
            writer,
            "Generated by Smart Tree - {} files, {} directories, {}",
            stats.total_files,
            stats.total_dirs,
            format_size(stats.total_size)
        )?;
        writeln!(writer)?;

        // Choose format based on style
        match self.style {
            MermaidStyle::Flowchart => self.write_flowchart(writer, nodes, root_path)?,
            MermaidStyle::Mindmap => self.write_mindmap(writer, nodes, root_path)?,
            MermaidStyle::GitGraph => self.write_gitgraph(writer, nodes, root_path)?,
            MermaidStyle::Treemap => self.write_treemap(writer, nodes, root_path)?,
        }

        // Footer with copy instructions
        writeln!(writer)?;
        writeln!(
            writer,
            "<!-- Copy the mermaid code block above into your markdown file -->"
        )?;
        writeln!(
            writer,
            "<!-- GitHub, GitLab, and many other platforms will render it automatically! -->"
        )?;

        Ok(())
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::scanner::{FileCategory, FileType, FilesystemType};
    use std::path::PathBuf;
    use std::time::SystemTime;

    #[test]
    fn test_sanitize_node_id() {
        let path = PathBuf::from("/home/user/my-project/src/main.rs");
        let id = MermaidFormatter::sanitize_node_id(&path);
        assert!(!id.contains('/'));
        assert!(!id.contains('.'));
        assert!(!id.contains('-'));
    }

    #[test]
    fn test_mermaid_flowchart() {
        let formatter = MermaidFormatter::new(MermaidStyle::Flowchart, false, PathDisplayMode::Off);

        let nodes = vec![
            FileNode {
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
                is_hidden: false,
                permission_denied: false,
                depth: 1,
                file_type: FileType::Directory,
                category: FileCategory::Unknown,
                filesystem_type: FilesystemType::Unknown,
            },
            FileNode {
                path: PathBuf::from("src/main.rs"),
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
        assert!(output_str.contains("```mermaid"));
        assert!(output_str.contains("graph TD"));
        assert!(output_str.contains("src"));
        assert!(output_str.contains("main.rs"));
    }
}
