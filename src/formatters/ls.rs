// -----------------------------------------------------------------------------
// 🗂️ LS MODE - The Classic Unix Experience with Smart-Tree Magic!
// -----------------------------------------------------------------------------
// This formatter replicates the beloved `ls -Alh` command that every Unix user
// knows and loves. We take that familiar format and supercharge it with
// smart-tree's intelligence and beautiful formatting.
//
// Output format matches: drwxrwxr-x 1 hue hue 1.2K Jul  9 14:56 filename
// - Permissions (like drwxrwxr-x)
// - Link count
// - Owner and group
// - Human-readable file size (1.2K, 45M, 2.3G)
// - Last modified date and time
// - Filename with proper coloring and emojis (optional)
//
// Hue gets the comfort of familiar ls output, Trish gets beautiful formatting,
// and Aye gets to show off some Rust file system wizardry! 🎭
// -----------------------------------------------------------------------------

use super::Formatter;
use crate::emoji_mapper;
use crate::scanner::{FileNode, TreeStats};
use anyhow::Result;
use chrono::{DateTime, Local};
use std::fs;
use std::io::Write;
use std::path::Path;

#[cfg(unix)]
use std::os::unix::fs::{MetadataExt, PermissionsExt};

/// LS Formatter - Unix ls -Alh output with smart-tree enhancements
///
/// This formatter provides the classic Unix `ls -Alh` experience:
/// - Long format with detailed file information
/// - Human-readable file sizes  
/// - All files including hidden ones
/// - Familiar permissions display
/// - Proper date/time formatting
///
/// Perfect for users who want smart-tree's power with familiar ls output!
pub struct LsFormatter {
    /// Whether to show emojis alongside filenames (default: true)
    show_emojis: bool,
    /// Whether to use colors in output (default: true)  
    use_colors: bool,
}

impl Default for LsFormatter {
    fn default() -> Self {
        Self::new(true, true)
    }
}

impl LsFormatter {
    /// Create a new LS formatter
    ///
    /// # Arguments
    /// * `show_emojis` - Whether to include emojis in the output (Trish loves these!)
    /// * `use_colors` - Whether to colorize the output for better readability
    pub fn new(show_emojis: bool, use_colors: bool) -> Self {
        Self {
            show_emojis,
            use_colors,
        }
    }

    /// Format file permissions in the classic Unix style (e.g., drwxrwxr-x)
    ///
    /// This creates the familiar 10-character permission string that every
    /// Unix user recognizes. First character is file type, then 3 groups of
    /// 3 characters each for owner, group, and other permissions.
    /// On Windows, we show a simplified version.
    fn format_permissions(&self, node: &FileNode) -> String {
        let metadata = match fs::metadata(&node.path) {
            Ok(meta) => meta,
            Err(_) => return "?---------".to_string(), // Permission denied or file missing
        };

        let file_type = if metadata.is_dir() {
            'd'
        } else if metadata.is_symlink() {
            'l'
        } else {
            '-'
        };

        #[cfg(unix)]
        {
            let mode = metadata.permissions().mode();

            // Extract permission bits (owner, group, other)
            let owner_perms = format!(
                "{}{}{}",
                if mode & 0o400 != 0 { 'r' } else { '-' },
                if mode & 0o200 != 0 { 'w' } else { '-' },
                if mode & 0o100 != 0 { 'x' } else { '-' }
            );

            let group_perms = format!(
                "{}{}{}",
                if mode & 0o040 != 0 { 'r' } else { '-' },
                if mode & 0o020 != 0 { 'w' } else { '-' },
                if mode & 0o010 != 0 { 'x' } else { '-' }
            );

            let other_perms = format!(
                "{}{}{}",
                if mode & 0o004 != 0 { 'r' } else { '-' },
                if mode & 0o002 != 0 { 'w' } else { '-' },
                if mode & 0o001 != 0 { 'x' } else { '-' }
            );

            format!("{}{}{}{}", file_type, owner_perms, group_perms, other_perms)
        }

        #[cfg(windows)]
        {
            // On Windows, show simplified permissions
            let readonly = metadata.permissions().readonly();
            if readonly {
                format!("{}r--r--r--", file_type)
            } else {
                format!("{}rw-rw-rw-", file_type)
            }
        }

        #[cfg(not(any(unix, windows)))]
        {
            // For other platforms, show a generic format
            format!("{}rwxrwxrwx", file_type)
        }
    }

    /// Format file size in human-readable format (like ls -h)
    ///
    /// Converts bytes to human-readable units (B, K, M, G, T)
    /// Uses binary units (1024) like traditional ls command
    fn format_size(&self, size: u64) -> String {
        const UNITS: &[&str] = &["B", "K", "M", "G", "T"];

        if size == 0 {
            return "0".to_string();
        }

        let mut size_f = size as f64;
        let mut unit_index = 0;

        while size_f >= 1024.0 && unit_index < UNITS.len() - 1 {
            size_f /= 1024.0;
            unit_index += 1;
        }

        if unit_index == 0 {
            format!("{}", size)
        } else if size_f >= 10.0 {
            format!("{:.0}{}", size_f, UNITS[unit_index])
        } else {
            format!("{:.1}{}", size_f, UNITS[unit_index])
        }
    }

    /// Get the appropriate emoji for a file node
    ///
    /// This adds visual flair to the output, making it easier to quickly
    /// identify file types. Uses the centralized emoji mapper for rich file type representation!
    fn get_emoji(&self, node: &FileNode) -> &'static str {
        if !self.show_emojis {
            return "";
        }

        emoji_mapper::get_file_emoji(node, false)
    }

    /// Format the filename with optional emoji and coloring
    /// Ensures consistent spacing by padding emoji field to 2 characters
    fn format_filename(&self, node: &FileNode) -> String {
        let emoji = self.get_emoji(node);
        let filename = node
            .path
            .file_name()
            .unwrap_or_else(|| node.path.as_os_str())
            .to_string_lossy();

        // Format emoji with consistent spacing
        let emoji_field = if emoji.is_empty() {
            String::new()
        } else {
            // Always add a space after emoji for consistent alignment
            format!("{} ", emoji)
        };

        if self.use_colors {
            if node.is_dir {
                // Blue color for directories (ANSI color code 34)
                format!("{}\x1b[34m{}\x1b[0m", emoji_field, filename)
            } else if node.path.extension().and_then(|s| s.to_str()) == Some("rs") {
                // Orange color for Rust files (Hue's favorite!)
                format!("{}\x1b[38;5;208m{}\x1b[0m", emoji_field, filename)
            } else {
                // Default color for regular files
                format!("{}{}", emoji_field, filename)
            }
        } else {
            if emoji_field.is_empty() {
                filename.to_string()
            } else {
                format!("{}{}", emoji_field, filename)
            }
        }
    }

    /// Get owner and group information
    ///
    /// On Unix systems, this attempts to resolve uid/gid to actual names.
    /// Falls back to numeric IDs if resolution fails.
    fn get_owner_group(&self, node: &FileNode) -> (String, String) {
        #[cfg(unix)]
        {
            use std::ffi::CStr;

            // Get username from uid
            let owner = unsafe {
                let passwd = libc::getpwuid(node.uid);
                if passwd.is_null() {
                    // User not found, use numeric ID
                    node.uid.to_string()
                } else {
                    // Convert username to String
                    CStr::from_ptr((*passwd).pw_name)
                        .to_string_lossy()
                        .to_string()
                }
            };

            // Get group name from gid
            let group = unsafe {
                let grp = libc::getgrgid(node.gid);
                if grp.is_null() {
                    // Group not found, use numeric ID
                    node.gid.to_string()
                } else {
                    // Convert group name to String
                    CStr::from_ptr((*grp).gr_name).to_string_lossy().to_string()
                }
            };

            (owner, group)
        }

        #[cfg(not(unix))]
        {
            // On non-Unix systems, just show the numeric IDs
            (node.uid.to_string(), node.gid.to_string())
        }
    }

    /// Get hard link count (simplified)
    fn get_link_count(&self, node: &FileNode) -> u64 {
        #[cfg(unix)]
        {
            match fs::metadata(&node.path) {
                Ok(meta) => meta.nlink(),
                Err(_) => 1, // Default to 1 if we can't read metadata
            }
        }

        #[cfg(not(unix))]
        {
            // On non-Unix systems, always return 1 for files, 2 for directories
            // This is a reasonable approximation
            if node.is_dir {
                2
            } else {
                1
            }
        }
    }
}

impl Formatter for LsFormatter {
    fn format(
        &self,
        writer: &mut dyn Write,
        nodes: &[FileNode],
        _stats: &TreeStats,
        root_path: &Path,
    ) -> Result<()> {
        // Check if this appears to be a filtered result set (from --find or other filters)
        // Heuristic: if nodes don't include all direct children of root, it's likely filtered
        let direct_child_count = nodes
            .iter()
            .filter(|n| n.path != root_path && n.path.parent() == Some(root_path))
            .count();
        let total_non_root = nodes.iter().filter(|n| n.path != root_path).count();
        let is_filtered = total_non_root > 0
            && (direct_child_count == 0 || total_non_root > direct_child_count * 2);

        let display_nodes: Vec<&FileNode> = if is_filtered {
            // For filtered results, show all matching nodes with full paths
            nodes
                .iter()
                .filter(|node| node.path != root_path) // Still exclude the root
                .collect()
        } else {
            // Normal ls behavior: only show direct children of root_path
            nodes
                .iter()
                .filter(|node| {
                    if node.path == root_path {
                        return false; // Don't show the root directory itself
                    }
                    // Only show direct children (depth 1 from root)
                    node.path.parent() == Some(root_path)
                })
                .collect()
        };

        // If no files/directories to display, show a message
        if display_nodes.is_empty() {
            writeln!(writer, "No matching files or directories found")?;
            if is_filtered {
                writeln!(writer, "")?;
                writeln!(
                    writer,
                    "💡 Tip: Try using --everything to search in ignored directories like .cache"
                )?;
                writeln!(
                    writer,
                    "💡 Tip: Use -d 10 or higher to search deeper (default is 5 levels)"
                )?;
                writeln!(
                    writer,
                    "💡 Tip: Hidden directories need -a flag, ignored ones need --everything"
                )?;
            }
            return Ok(());
        }

        // Note: Nodes are already sorted by the scanner based on user's --sort preference
        // We don't re-sort here to preserve the requested sort order

        // Format each file/directory in ls -Alh style
        for node in display_nodes {
            let permissions = self.format_permissions(node);
            let link_count = self.get_link_count(node);
            let (owner, group) = self.get_owner_group(node);
            let size = self.format_size(node.size);

            // Format the modification time
            let modified_time = match fs::metadata(&node.path) {
                Ok(meta) => match meta.modified() {
                    Ok(time) => {
                        let datetime: DateTime<Local> = time.into();
                        datetime.format("%b %d %H:%M").to_string()
                    }
                    Err(_) => "??? ?? ??:??".to_string(),
                },
                Err(_) => "??? ?? ??:??".to_string(),
            };

            // Determine filename display strategy:
            // - When filtering results (search/pattern match): Show relative path for context
            // - Otherwise: Show only the filename for cleaner output
            let filename = if is_filtered {
                // Format with relative path to help identify match locations
                let emoji = self.get_emoji(node);
                // Format emoji with consistent spacing
                let emoji_field = if emoji.is_empty() {
                    String::new()
                } else {
                    // Always add a space after emoji for consistent alignment
                    format!("{} ", emoji)
                };

                // Get relative path from root_path
                let relative_path = node
                    .path
                    .strip_prefix(root_path)
                    .unwrap_or(&node.path)
                    .display();

                // Apply directory coloring if colors are enabled
                if self.use_colors && node.is_dir {
                    // Blue color (ANSI 34) for directories
                    format!("{}\x1b[34m{}\x1b[0m", emoji_field, relative_path)
                } else {
                    // Default formatting for files or when colors are disabled
                    format!("{}{}", emoji_field, relative_path)
                }
            } else {
                self.format_filename(node)
            };

            // Write the ls -Alh formatted line
            writeln!(
                writer,
                "{:<10} {:>1} {:<4} {:<4} {:>6} {} {}",
                permissions, link_count, owner, group, size, modified_time, filename
            )?;
        }

        Ok(())
    }
}

// -----------------------------------------------------------------------------
// 🎭 Tests - Because Trish insists on quality assurance!
// -----------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::scanner::{FileCategory, FileType, FilesystemType};
    use std::path::PathBuf;
    use std::time::SystemTime;

    #[test]
    fn test_format_size() {
        let formatter = LsFormatter::new(false, false);

        assert_eq!(formatter.format_size(0), "0");
        assert_eq!(formatter.format_size(500), "500");
        assert_eq!(formatter.format_size(1024), "1.0K");
        assert_eq!(formatter.format_size(1536), "1.5K");
        assert_eq!(formatter.format_size(1048576), "1.0M");
        assert_eq!(formatter.format_size(1073741824), "1.0G");
    }

    #[test]
    fn test_emoji_selection() {
        let formatter = LsFormatter::new(true, false);

        // Test directory emojis
        let empty_dir = FileNode {
            path: PathBuf::from("/test"),
            file_type: FileType::Directory,
            size: 0,
            is_dir: true,
            depth: 0,
            permissions: 0o755,
            modified: SystemTime::now(),
            uid: 1000,
            gid: 1000,
            is_symlink: false,
            is_hidden: false,
            permission_denied: false,
            is_ignored: false,
            category: FileCategory::Unknown,
            search_matches: None,
            filesystem_type: FilesystemType::Unknown,
        };
        assert_eq!(formatter.get_emoji(&empty_dir), "📂");

        // Test file emojis
        let empty_file = FileNode {
            path: PathBuf::from("/test.txt"),
            file_type: FileType::RegularFile,
            size: 0,
            is_dir: false,
            depth: 0,
            permissions: 0o644,
            modified: SystemTime::now(),
            uid: 1000,
            gid: 1000,
            is_symlink: false,
            is_hidden: false,
            permission_denied: false,
            is_ignored: false,
            category: FileCategory::Unknown,
            search_matches: None,
            filesystem_type: FilesystemType::Unknown,
        };
        assert_eq!(formatter.get_emoji(&empty_file), "🪹");
    }

    #[test]
    fn test_permissions_format() {
        let formatter = LsFormatter::new(false, false);

        // This is a basic test - in real usage, format_permissions
        // reads actual file metadata
        let test_node = FileNode {
            path: PathBuf::from("/test"),
            file_type: FileType::Directory,
            size: 0,
            is_dir: true,
            depth: 0,
            permissions: 0o755,
            modified: SystemTime::now(),
            uid: 1000,
            gid: 1000,
            is_symlink: false,
            is_hidden: false,
            permission_denied: false,
            is_ignored: false,
            category: FileCategory::Unknown,
            search_matches: None,
            filesystem_type: FilesystemType::Unknown,
        };

        let perms = formatter.format_permissions(&test_node);
        // Should start with 'd' for directory or '?' if we can't read it
        assert!(perms.starts_with('d') || perms.starts_with('?'));
        assert_eq!(perms.len(), 10); // Always 10 characters
    }
}
