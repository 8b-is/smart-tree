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

    /// Calculate visual weight based on directory size and depth
    /// Larger directories and shallower depths get higher visual weight (thicker lines)
    fn calculate_visual_weight(&self, node: &FileNode) -> u8 {
        if !node.is_dir {
            return 1; // Files get standard weight
        }
        
        // Base weight starts higher for directories
        let mut weight = 2;
        
        // Size-based scaling (logarithmic to avoid extreme values)
        // Directories with more content get thicker lines
        if node.size > 0 {
            let size_factor = (node.size as f64).log10().max(1.0) as u8;
            weight += (size_factor / 2).min(2); // Cap the size contribution
        }
        
        // Depth-based scaling - shallower directories get thicker lines
        // Root level (depth 0) gets maximum thickness
        let depth_bonus = if node.depth == 0 {
            3 // Root gets the thickest lines
        } else if node.depth == 1 {
            2 // First level gets thick lines
        } else if node.depth <= 3 {
            1 // Moderate depth gets medium lines
        } else {
            0 // Deep levels get standard lines
        };
        
        weight += depth_bonus;
        
        // Cap the maximum weight to avoid going beyond our character sets
        weight.min(5)
    }
    
    /// Get terminal characters with gradient background based on file size
    /// Returns formatted string with gradient background that fades to the right
    fn get_terminal_chars(&self, file_size: u64, is_last: bool) -> String {
        let base_char = if is_last { "└──" } else { "├── " };
        
        if self.no_emoji {
            // No color mode - just return plain characters
            return base_char.to_string();
        }
        
        // Calculate gradient intensity based on file size (0-100)
        let intensity = self.calculate_gradient_intensity(file_size);
        
        // Create gradient background that fades from left to right
        self.apply_gradient_background(base_char, intensity)
    }
    
    /// Get continuation characters with gradient background
    /// Returns formatted string with gradient background for vertical lines
    fn get_continuation_chars(&self, file_size: u64, is_vertical: bool) -> String {
        let base_char = if is_vertical { "│   " } else { "    " };
        
        if self.no_emoji {
            // No color mode - just return plain characters
            return base_char.to_string();
        }
        
        // Calculate gradient intensity based on file size
        let intensity = self.calculate_gradient_intensity(file_size);
        
        // Create gradient background that fades from left to right
        self.apply_gradient_background(base_char, intensity)
    }
    
    /// Calculate gradient intensity (0-100) based on file size
    /// Larger files get more intense gradients
    fn calculate_gradient_intensity(&self, file_size: u64) -> u8 {
        // Define size thresholds for gradient intensity
        match file_size {
            0..=1024 => 10,                    // 0-1KB: Very light
            1025..=10240 => 25,                // 1-10KB: Light
            10241..=102400 => 40,              // 10-100KB: Medium-light
            102401..=1048576 => 55,            // 100KB-1MB: Medium
            1048577..=10485760 => 70,          // 1-10MB: Medium-heavy
            10485761..=104857600 => 85,        // 10-100MB: Heavy
            _ => 100,                          // >100MB: Maximum intensity
        }
    }
    
    /// Apply gradient background that fades from left to right
    /// Creates beautiful visual hierarchy based on file size
    fn apply_gradient_background(&self, text: &str, intensity: u8) -> String {
        let chars: Vec<char> = text.chars().collect();
        let mut result = String::new();
        
        // Apply gradient from left to right based on file size intensity
        if intensity < 15 {
            // Very small files - subtle blue gradient
            for (i, &ch) in chars.iter().enumerate() {
                let bg_color = match i {
                    0 => "\x1b[48;5;17m",      // Dark blue
                    1 => "\x1b[48;5;18m",      // Medium blue
                    2 => "\x1b[48;5;19m",      // Light blue
                    _ => "\x1b[48;5;20m",      // Very light blue
                };
                result.push_str(&format!("{}{}", bg_color, ch));
            }
        } else if intensity < 35 {
            // Small files - green gradient
            for (i, &ch) in chars.iter().enumerate() {
                let bg_color = match i {
                    0 => "\x1b[48;5;22m",      // Dark green
                    1 => "\x1b[48;5;28m",      // Medium green
                    2 => "\x1b[48;5;34m",      // Light green
                    _ => "\x1b[48;5;40m",      // Very light green
                };
                result.push_str(&format!("{}{}", bg_color, ch));
            }
        } else if intensity < 55 {
            // Medium files - yellow gradient
            for (i, &ch) in chars.iter().enumerate() {
                let bg_color = match i {
                    0 => "\x1b[48;5;3m",       // Dark yellow
                    1 => "\x1b[48;5;11m",      // Medium yellow
                    2 => "\x1b[48;5;227m",     // Light yellow
                    _ => "\x1b[48;5;228m",     // Very light yellow
                };
                result.push_str(&format!("{}{}", bg_color, ch));
            }
        } else if intensity < 75 {
            // Large files - orange gradient
            for (i, &ch) in chars.iter().enumerate() {
                let bg_color = match i {
                    0 => "\x1b[48;5;202m",     // Dark orange
                    1 => "\x1b[48;5;208m",     // Medium orange
                    2 => "\x1b[48;5;214m",     // Light orange
                    _ => "\x1b[48;5;220m",     // Very light orange
                };
                result.push_str(&format!("{}{}", bg_color, ch));
            }
        } else {
            // Huge files - red gradient
            for (i, &ch) in chars.iter().enumerate() {
                let bg_color = match i {
                    0 => "\x1b[48;5;196m",     // Dark red
                    1 => "\x1b[48;5;202m",     // Medium red
                    2 => "\x1b[48;5;208m",     // Light red
                    _ => "\x1b[48;5;214m",     // Very light red
                };
                result.push_str(&format!("{}{}", bg_color, ch));
            }
        }
        
        // Reset color at the end
        result.push_str("\x1b[0m");
        result
    }

    /// Get context-aware emoji based on file type and node properties
    /// Returns different emojis for empty files, empty directories, and locked directories
    fn get_file_emoji(&self, node: &FileNode) -> &'static str {
        // Handle permission denied directories with lock emoji
        if node.permission_denied {
            return if self.no_emoji { "[LOCK]" } else { "🔒" };
        }
        
        if self.no_emoji {
            match node.file_type {
                FileType::Directory => {
                    if node.size == 0 {
                        "[EMPTY_D]" // Empty directory
                    } else {
                        "[D]" // Regular directory
                    }
                },
                FileType::Symlink => "[L]",
                FileType::Executable => "[X]",
                FileType::Socket => "[S]",
                FileType::Pipe => "[P]",
                FileType::BlockDevice => "[B]",
                FileType::CharDevice => "[C]",
                FileType::RegularFile => {
                    if node.size == 0 {
                        "[EMPTY_F]" // Empty file
                    } else {
                        "[F]" // Regular file
                    }
                },
            }
        } else {
            match node.file_type {
                FileType::Directory => {
                    if node.size == 0 {
                        "📂" // Empty directory (open folder)
                    } else {
                        "📁" // Regular directory (closed folder)
                    }
                },
                FileType::Symlink => "🔗",
                FileType::Executable => "⚙️",
                FileType::Socket => "🔌",
                FileType::Pipe => "📝",
                FileType::BlockDevice => "💾",
                FileType::CharDevice => "📺",
                FileType::RegularFile => {
                    if node.size == 0 {
                        "📋" // Empty file (clipboard/empty document)
                    } else {
                        "📄" // Regular file
                    }
                },
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

        // Build tree prefix with gradient backgrounds based on file size
        // Larger files get more intense gradient backgrounds that fade to the right
        
        for (i, &last) in is_last.iter().enumerate() {
            if i == is_last.len() - 1 {
                // Terminal connectors (last level) - with gradient background
                let terminal_chars = self.get_terminal_chars(node.size, last);
                prefix.push_str(&terminal_chars);
            } else {
                // Continuation lines (intermediate levels) - with gradient background
                let continuation_chars = self.get_continuation_chars(node.size, !last);
                prefix.push_str(&continuation_chars);
            }
        }

        let emoji = self.get_file_emoji(node);

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
                    format!(
                        " [FOUND:L{}:C{},{}x{}]",
                        line, col, matches.total_count, truncated
                    )
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
            format!(
                "{} {}{}{}{}",
                emoji, colored_name, size_str, indicator, search_indicator
            )
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
