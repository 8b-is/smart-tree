use super::{Formatter, PathDisplayMode};
use crate::scanner::{FileCategory, FileNode, FileType, TreeStats};
use anyhow::Result;
use colored::*;
use humansize::{format_size, BINARY};
use std::collections::{HashMap, HashSet};
use std::io::Write;
use std::path::{Path, PathBuf};

pub struct ClassicFormatter {
    pub no_emoji: bool,
    pub use_color: bool,
    pub path_mode: PathDisplayMode,
}

impl ClassicFormatter {
    pub fn new(no_emoji: bool, use_color: bool, path_mode: PathDisplayMode) -> Self {
        Self {
            no_emoji,
            use_color,
            path_mode,
        }
    }

    fn get_file_emoji(&self, file_type: FileType) -> &'static str {
        if self.no_emoji {
            match file_type {
                FileType::Directory => "[D]",
                FileType::Symlink => "[L]",
                FileType::Executable => "[X]",
                FileType::Socket => "[S]",
                FileType::Pipe => "[P]",
                FileType::BlockDevice => "[B]",
                FileType::CharDevice => "[C]",
                FileType::RegularFile => "[F]",
            }
        } else {
            match file_type {
                FileType::Directory => "📁",
                FileType::Symlink => "🔗",
                FileType::Executable => "⚙️",
                FileType::Socket => "🔌",
                FileType::Pipe => "📝",
                FileType::BlockDevice => "💾",
                FileType::CharDevice => "📺",
                FileType::RegularFile => "📄",
            }
        }
    }

    fn build_tree_structure(
        &self,
        nodes: &[FileNode],
        root_path: &Path,
    ) -> Vec<(FileNode, Vec<bool>)> {
        let mut result = Vec::new();

        if nodes.is_empty() {
            return result;
        }

        // Sort all nodes by path to ensure proper tree order
        let mut sorted_nodes = nodes.to_vec();
        sorted_nodes.sort_by(|a, b| a.path.cmp(&b.path));

        // Remove duplicates based on path
        let mut seen = HashSet::new();
        sorted_nodes.retain(|node| seen.insert(node.path.clone()));

        // Build parent-child relationships
        let mut children_map: HashMap<PathBuf, Vec<usize>> = HashMap::new();
        let mut parent_indices: Vec<Option<usize>> = vec![None; sorted_nodes.len()];

        // Create a path-to-index map for O(1) parent lookups
        let path_to_index: HashMap<PathBuf, usize> = sorted_nodes
            .iter()
            .enumerate()
            .map(|(i, node)| (node.path.clone(), i))
            .collect();

        for (i, node) in sorted_nodes.iter().enumerate() {
            if let Some(parent_path) = node.path.parent() {
                // Find parent node index using HashMap lookup (O(1) instead of O(n))
                if let Some(&parent_idx) = path_to_index.get(parent_path) {
                    parent_indices[i] = Some(parent_idx);
                    children_map
                        .entry(parent_path.to_path_buf())
                        .or_default()
                        .push(i);
                }
            }
        }

        // Sort children by directory first, then name
        for children in children_map.values_mut() {
            children.sort_by(|&a, &b| {
                let node_a = &sorted_nodes[a];
                let node_b = &sorted_nodes[b];
                match (node_a.is_dir, node_b.is_dir) {
                    (true, false) => std::cmp::Ordering::Less,
                    (false, true) => std::cmp::Ordering::Greater,
                    _ => node_a.path.file_name().cmp(&node_b.path.file_name()),
                }
            });
        }

        // Build the tree structure recursively
        fn add_node_to_result(
            node_idx: usize,
            nodes: &[FileNode],
            children_map: &HashMap<PathBuf, Vec<usize>>,
            result: &mut Vec<(FileNode, Vec<bool>)>,
            is_last_stack: Vec<bool>,
        ) {
            let node = &nodes[node_idx];
            result.push((node.clone(), is_last_stack.clone()));

            if let Some(children) = children_map.get(&node.path) {
                for (i, &child_idx) in children.iter().enumerate() {
                    let is_last = i == children.len() - 1;
                    let mut new_stack = is_last_stack.clone();
                    new_stack.push(is_last);
                    add_node_to_result(child_idx, nodes, children_map, result, new_stack);
                }
            }
        }

        // Find root node (should only be the scan root)
        for (i, node) in sorted_nodes.iter().enumerate() {
            if node.path == root_path {
                add_node_to_result(i, &sorted_nodes, &children_map, &mut result, vec![]);
                break;
            }
        }

        result
    }

    fn get_color_for_category(&self, category: FileCategory) -> Option<Color> {
        if !self.use_color {
            return None;
        }

        match category {
            // Programming languages
            FileCategory::Rust => Some(Color::TrueColor {
                r: 255,
                g: 65,
                b: 54,
            }), // Rust orange
            FileCategory::Python => Some(Color::TrueColor {
                r: 55,
                g: 118,
                b: 171,
            }), // Python blue
            FileCategory::JavaScript => Some(Color::TrueColor {
                r: 240,
                g: 219,
                b: 79,
            }), // JS yellow
            FileCategory::TypeScript => Some(Color::TrueColor {
                r: 0,
                g: 122,
                b: 204,
            }), // TS blue
            FileCategory::Java => Some(Color::TrueColor {
                r: 244,
                g: 67,
                b: 54,
            }), // Java red
            FileCategory::C => Some(Color::TrueColor {
                r: 0,
                g: 89,
                b: 157,
            }), // C blue
            FileCategory::Cpp => Some(Color::TrueColor {
                r: 0,
                g: 89,
                b: 157,
            }), // C++ blue
            FileCategory::Go => Some(Color::TrueColor {
                r: 0,
                g: 173,
                b: 216,
            }), // Go cyan
            FileCategory::Ruby => Some(Color::TrueColor {
                r: 204,
                g: 52,
                b: 45,
            }), // Ruby red
            FileCategory::PHP => Some(Color::TrueColor {
                r: 119,
                g: 123,
                b: 180,
            }), // PHP purple
            FileCategory::Shell => Some(Color::Green),

            // Markup/Data
            FileCategory::Markdown => Some(Color::TrueColor {
                r: 76,
                g: 202,
                b: 240,
            }), // Light blue
            FileCategory::Html => Some(Color::TrueColor {
                r: 228,
                g: 77,
                b: 38,
            }), // HTML orange
            FileCategory::Css => Some(Color::TrueColor {
                r: 33,
                g: 150,
                b: 243,
            }), // CSS blue
            FileCategory::Json => Some(Color::TrueColor {
                r: 0,
                g: 150,
                b: 136,
            }), // Changed to teal
            FileCategory::Yaml => Some(Color::TrueColor {
                r: 203,
                g: 71,
                b: 119,
            }), // YAML pink
            FileCategory::Xml => Some(Color::TrueColor {
                r: 255,
                g: 111,
                b: 0,
            }), // XML orange
            FileCategory::Toml => Some(Color::TrueColor {
                r: 150,
                g: 111,
                b: 214,
            }), // TOML purple

            // Build/Config
            FileCategory::Makefile => Some(Color::TrueColor {
                r: 66,
                g: 165,
                b: 245,
            }), // Make blue
            FileCategory::Dockerfile => Some(Color::TrueColor {
                r: 33,
                g: 150,
                b: 243,
            }), // Docker blue
            FileCategory::GitConfig => Some(Color::TrueColor {
                r: 241,
                g: 80,
                b: 47,
            }), // Git orange

            // Archives
            FileCategory::Archive => Some(Color::TrueColor {
                r: 121,
                g: 134,
                b: 203,
            }), // Archive purple

            // Media
            FileCategory::Image => Some(Color::Magenta),
            FileCategory::Video => Some(Color::TrueColor {
                r: 255,
                g: 87,
                b: 34,
            }), // Video orange
            FileCategory::Audio => Some(Color::TrueColor {
                r: 0,
                g: 188,
                b: 212,
            }), // Audio cyan

            // System
            FileCategory::SystemFile => Some(Color::TrueColor {
                r: 96,
                g: 96,
                b: 96,
            }), // Dark grey
            FileCategory::Binary => Some(Color::TrueColor {
                r: 158,
                g: 158,
                b: 158,
            }), // Light grey

            // Default
            FileCategory::Unknown => None,
        }
    }

    fn format_node(&self, node: &FileNode, is_last: &[bool], root_path: &Path) -> String {
        let mut prefix = String::new();

        // Build tree prefix
        for (i, &last) in is_last.iter().enumerate() {
            if i == is_last.len() - 1 {
                prefix.push_str(if last { "└── " } else { "├── " });
            } else {
                prefix.push_str(if last { "    " } else { "│   " });
            }
        }

        let emoji = self.get_file_emoji(node.file_type);

        // Determine what name to show based on path mode
        let name = match self.path_mode {
            PathDisplayMode::Off => node
                .path
                .file_name()
                .unwrap_or(node.path.as_os_str())
                .to_string_lossy()
                .to_string(),
            PathDisplayMode::Relative => {
                if node.path == root_path {
                    node.path
                        .file_name()
                        .unwrap_or(node.path.as_os_str())
                        .to_string_lossy()
                        .to_string()
                } else {
                    node.path
                        .strip_prefix(root_path)
                        .unwrap_or(&node.path)
                        .to_string_lossy()
                        .to_string()
                }
            }
            PathDisplayMode::Full => node.path.display().to_string(),
        };

        let size_str = if node.is_dir {
            String::new()
        } else {
            format!(" ({})", format_size(node.size, BINARY))
        };

        let indicator = if node.permission_denied {
            " [*]"
        } else if node.is_ignored {
            " [ignored]"
        } else {
            ""
        };

        // Add search match indicator
        let search_indicator = if let Some(matches) = &node.search_matches {
            if matches.total_count > 0 {
                let (line, col) = matches.first_match;
                let truncated = if matches.truncated { ",TRUNCATED" } else { "" };
                if matches.total_count > 1 {
                    format!(" [FOUND:L{}:C{},{}x{}]", line, col, matches.total_count, truncated)
                } else {
                    format!(" [FOUND:L{}:C{}]", line, col)
                }
            } else {
                String::new()
            }
        } else {
            String::new()
        };

        // Apply color to the name based on file category
        let colored_name = if node.is_dir {
            // Directories get bright yellow and bold
            if self.use_color {
                name.bright_yellow().bold().to_string()
            } else {
                name
            }
        } else if let Some(color) = self.get_color_for_category(node.category) {
            name.color(color).to_string()
        } else {
            name
        };

        if is_last.is_empty() {
            // Root node
            format!("{} {}{}{}{}", emoji, colored_name, size_str, indicator, search_indicator)
        } else {
            format!(
                "{}{} {}{}{}{}",
                prefix, emoji, colored_name, size_str, indicator, search_indicator
            )
        }
    }
}

impl Formatter for ClassicFormatter {
    fn format(
        &self,
        writer: &mut dyn Write,
        nodes: &[FileNode],
        stats: &TreeStats,
        root_path: &Path,
    ) -> Result<()> {
        let tree_structure = self.build_tree_structure(nodes, root_path);

        for (node, is_last) in tree_structure {
            writeln!(writer, "{}", self.format_node(&node, &is_last, root_path))?;
        }

        // Print summary
        writeln!(writer)?;
        writeln!(
            writer,
            "{} directories, {} files, {} total",
            stats.total_dirs,
            stats.total_files,
            format_size(stats.total_size, BINARY)
        )?;

        // Check if any nodes had permission denied
        if nodes.iter().any(|n| n.permission_denied) {
            writeln!(writer)?;
            writeln!(writer, "[*] Permission denied")?;
        }

        Ok(())
    }
}
