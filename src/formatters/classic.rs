use super::Formatter;
use crate::scanner::{FileNode, FileType, TreeStats};
use anyhow::Result;
use humansize::{format_size, BINARY};
use std::collections::HashMap;
use std::io::Write;
use std::path::{Path, PathBuf};

pub struct ClassicFormatter {
    pub no_emoji: bool,
    pub use_color: bool,
}

impl ClassicFormatter {
    pub fn new(no_emoji: bool, use_color: bool) -> Self {
        Self { no_emoji, use_color }
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

    fn build_tree_structure(&self, nodes: &[FileNode], root_path: &Path) -> Vec<(FileNode, Vec<bool>)> {
        let mut result = Vec::new();
        
        // Build a map of parent -> children
        let mut children_map: HashMap<PathBuf, Vec<&FileNode>> = HashMap::new();
        for node in nodes {
            if let Some(parent) = node.path.parent() {
                children_map.entry(parent.to_path_buf()).or_default().push(node);
            }
        }
        
        // Sort children by directory first, then name
        for children in children_map.values_mut() {
            children.sort_by(|a, b| {
                match (a.is_dir, b.is_dir) {
                    (true, false) => std::cmp::Ordering::Less,
                    (false, true) => std::cmp::Ordering::Greater,
                    _ => a.path.file_name().cmp(&b.path.file_name()),
                }
            });
        }
        
        // Recursive function to build tree
        fn add_to_tree(
            node: &FileNode,
            result: &mut Vec<(FileNode, Vec<bool>)>,
            children_map: &HashMap<PathBuf, Vec<&FileNode>>,
            is_last: Vec<bool>,
        ) {
            result.push((node.clone(), is_last.clone()));
            
            if let Some(children) = children_map.get(&node.path) {
                for (i, child) in children.iter().enumerate() {
                    let mut child_is_last = is_last.clone();
                    child_is_last.push(i == children.len() - 1);
                    add_to_tree(child, result, children_map, child_is_last);
                }
            }
        }
        
        // Find root node
        if let Some(root_node) = nodes.iter().find(|n| n.path == root_path) {
            add_to_tree(root_node, &mut result, &children_map, vec![]);
        }
        
        result
    }

    fn format_node(&self, node: &FileNode, is_last: &[bool]) -> String {
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
        let name = node.path.file_name()
            .unwrap_or(node.path.as_os_str())
            .to_string_lossy();
        
        let size_str = if node.is_dir {
            String::new()
        } else {
            format!(" ({})", format_size(node.size, BINARY))
        };
        
        let perm_indicator = if node.permission_denied { " [*]" } else { "" };
        
        if is_last.is_empty() {
            // Root node
            format!("{} {}{}{}", emoji, name, size_str, perm_indicator)
        } else {
            format!("{}{} {}{}{}", prefix, emoji, name, size_str, perm_indicator)
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
            writeln!(writer, "{}", self.format_node(&node, &is_last))?;
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
        
        Ok(())
    }
}