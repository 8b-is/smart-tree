use super::Formatter;
use crate::scanner::{FileNode, FileType, TreeStats};
use anyhow::Result;
use std::io::Write;
use std::path::Path;

pub struct HexFormatter {
    pub use_color: bool,
    pub no_emoji: bool,
    pub show_ignored: bool,
}

impl HexFormatter {
    pub fn new(use_color: bool, no_emoji: bool, show_ignored: bool) -> Self {
        Self {
            use_color,
            no_emoji,
            show_ignored,
        }
    }

    fn get_file_emoji(&self, file_type: FileType) -> &'static str {
        if self.no_emoji {
            match file_type {
                FileType::Directory => "d",
                FileType::Symlink => "l",
                FileType::Executable => "x",
                FileType::Socket => "s",
                FileType::Pipe => "p",
                FileType::BlockDevice => "b",
                FileType::CharDevice => "c",
                FileType::RegularFile => "f",
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

    fn format_node(&self, node: &FileNode, root_path: &Path) -> String {
        let depth_hex = format!("{:x}", node.depth);
        let perms_hex = format!("{:03x}", node.permissions);
        let uid_hex = format!("{:04x}", node.uid);
        let gid_hex = format!("{:04x}", node.gid);
        let size_hex = if node.is_dir {
            format!("{:08x}", 0)
        } else {
            format!("{:08x}", node.size)
        };
        let time_hex = format!("{:08x}", node.modified.duration_since(std::time::UNIX_EPOCH).unwrap_or_default().as_secs());
        
        let emoji = self.get_file_emoji(node.file_type);
        
        // Get relative path or just the name
        let name = if node.path == root_path {
            node.path.file_name()
                .unwrap_or(node.path.as_os_str())
                .to_string_lossy()
                .to_string()
        } else if let Ok(rel_path) = node.path.strip_prefix(root_path) {
            rel_path.to_string_lossy().to_string()
        } else {
            node.path.file_name()
                .unwrap_or(node.path.as_os_str())
                .to_string_lossy()
                .to_string()
        };

        // Add brackets for permission denied
        let display_name = if node.permission_denied {
            format!("[{}]", name)
        } else {
            name
        };

        if self.use_color {
            // ANSI color codes
            const CYAN: &str = "\x1b[36m";
            const YELLOW: &str = "\x1b[33m";
            const MAGENTA: &str = "\x1b[35m";
            const GREEN: &str = "\x1b[32m";
            const BLUE: &str = "\x1b[34m";
            const RESET: &str = "\x1b[0m";

            format!(
                "{}{}{} {}{}{} {}{} {}{} {}{}{} {}{}{} {} {}",
                CYAN, depth_hex, RESET,
                YELLOW, perms_hex, RESET,
                MAGENTA, uid_hex, gid_hex, RESET,
                GREEN, size_hex, RESET,
                BLUE, time_hex, RESET,
                emoji, display_name
            )
        } else {
            format!(
                "{} {} {} {} {} {} {} {}",
                depth_hex, perms_hex, uid_hex, gid_hex, size_hex, time_hex, emoji, display_name
            )
        }
    }
}

impl Formatter for HexFormatter {
    fn format(
        &self,
        writer: &mut dyn Write,
        nodes: &[FileNode],
        _stats: &TreeStats,
        root_path: &Path,
    ) -> Result<()> {
        // Sort nodes by path to ensure proper tree order
        let mut sorted_nodes = nodes.to_vec();
        sorted_nodes.sort_by(|a, b| a.path.cmp(&b.path));

        for node in &sorted_nodes {
            writeln!(writer, "{}", self.format_node(node, root_path))?;
        }

        Ok(())
    }
}