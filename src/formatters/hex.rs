use super::{Formatter, PathDisplayMode, StreamingFormatter};
use crate::scanner::{FileNode, FileType, FilesystemType, TreeStats};
use anyhow::Result;
use std::io::Write;
use std::path::Path;

pub struct HexFormatter {
    pub use_color: bool,
    pub no_emoji: bool,
    pub show_ignored: bool,
    pub path_mode: PathDisplayMode,
    pub show_filesystems: bool,
}

impl HexFormatter {
    pub fn new(
        use_color: bool,
        no_emoji: bool,
        show_ignored: bool,
        path_mode: PathDisplayMode,
        show_filesystems: bool,
    ) -> Self {
        Self {
            use_color,
            no_emoji,
            show_ignored,
            path_mode,
            show_filesystems,
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
        let time_hex = format!(
            "{:08x}",
            node.modified
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs()
        );

        let emoji = self.get_file_emoji(node.file_type);

        // Add filesystem indicator if enabled
        let fs_indicator = if self.show_filesystems && node.filesystem_type.should_show_by_default()
        {
            format!("{} ", node.filesystem_type.to_char())
        } else {
            String::new()
        };

        // Get name based on path mode
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

        // Add brackets for permission denied or ignored
        let display_name = if node.permission_denied {
            format!("[{}]", name)
        } else if node.is_ignored {
            format!("[{}]", name)
        } else {
            name
        };

        // Add search matches if present
        let display_name_with_search = if let Some(matches) = &node.search_matches {
            if matches.total_count > 0 {
                // Show first match position and total count
                let (line, col) = matches.first_match;
                let truncated_indicator = if matches.truncated { ",TRUNCATED" } else { "" };
                
                if matches.total_count > 1 {
                    format!("{} [SEARCH:L{}:C{},{}x{}]", display_name, line, col, matches.total_count, truncated_indicator)
                } else {
                    format!("{} [SEARCH:L{}:C{}]", display_name, line, col)
                }
            } else {
                display_name
            }
        } else {
            display_name
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
                "{}{}{} {}{}{} {}{} {}{} {}{}{} {}{}{} {}{} {}",
                CYAN,
                depth_hex,
                RESET,
                YELLOW,
                perms_hex,
                RESET,
                MAGENTA,
                uid_hex,
                gid_hex,
                RESET,
                GREEN,
                size_hex,
                RESET,
                BLUE,
                time_hex,
                RESET,
                fs_indicator,
                emoji,
                display_name_with_search
            )
        } else {
            format!(
                "{} {} {} {} {} {} {}{} {}",
                depth_hex,
                perms_hex,
                uid_hex,
                gid_hex,
                size_hex,
                time_hex,
                fs_indicator,
                emoji,
                display_name_with_search
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

impl StreamingFormatter for HexFormatter {
    fn start_stream(&self, _writer: &mut dyn Write, _root_path: &Path) -> Result<()> {
        // No header needed for hex format
        Ok(())
    }

    fn format_node(&self, writer: &mut dyn Write, node: &FileNode, root_path: &Path) -> Result<()> {
        writeln!(writer, "{}", self.format_node(node, root_path))?;
        writer.flush()?; // Ensure immediate output
        Ok(())
    }

    fn end_stream(
        &self,
        _writer: &mut dyn Write,
        _stats: &TreeStats,
        _root_path: &Path,
    ) -> Result<()> {
        // No footer needed for hex format
        Ok(())
    }
}
