use super::{Formatter, PathDisplayMode};
use crate::emoji_mapper;
use crate::scanner::{FileCategory, FileNode, TreeStats};
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
    pub sort_field: Option<String>,
}

impl ClassicFormatter {
    pub fn new(no_emoji: bool, use_color: bool, path_mode: PathDisplayMode) -> Self {
        Self {
            no_emoji,
            use_color,
            path_mode,
            sort_field: None,
        }
    }

    pub fn with_sort(mut self, sort_field: Option<String>) -> Self {
        self.sort_field = sort_field;
        self
    }

    /// Calculate visual weight based on directory size and depth
    /// Larger directories and shallower depths get higher visual weight (thicker lines)
    #[allow(dead_code)]
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
        let base_char = if is_last { "└── " } else { "├── " };

        if self.no_emoji || !self.use_color {
            // No color/emoji mode - just return plain characters
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

        if self.no_emoji || !self.use_color {
            // No color/emoji mode - just return plain characters
            return base_char.to_string();
        }

        // Calculate gradient intensity based on file size
        let intensity = self.calculate_gradient_intensity(file_size);

        // Create gradient background that fades from left to right
        self.apply_gradient_background(base_char, intensity)
    }

    /// Calculate gradient intensity (0-100) based on file size
    /// Enhanced thresholds with 50% darker colors for better visibility
    fn calculate_gradient_intensity(&self, file_size: u64) -> u8 {
        // Enhanced size thresholds - make gradients much more visible!
        const MB_100: u64 = 100 * 1024 * 1024; // 100MB
        const MB_200: u64 = 200 * 1024 * 1024; // 200MB
        const MB_500: u64 = 500 * 1024 * 1024; // 500MB
        const GB_1: u64 = 1024 * 1024 * 1024; // 1GB
        const GB_4: u64 = 4 * 1024 * 1024 * 1024; // 4GB

        match file_size {
            0..=1024 => 50,           // 0-1KB: 50% darker base
            1025..=10240 => 55,       // 1-10KB: Slightly more
            10241..=102400 => 60,     // 10-100KB: Visible
            102401..=1048576 => 65,   // 100KB-1MB: More visible
            1048577..=10485760 => 70, // 1-10MB: Quite visible
            10485761..MB_100 => 75,   // 10-100MB: Strong
            MB_100..MB_200 => 80,     // 100-200MB: 60% intensity
            MB_200..MB_500 => 85,     // 200-500MB: 70% intensity
            MB_500..GB_1 => 90,       // 500MB-1GB: 80% intensity
            GB_1..GB_4 => 95,         // 1-4GB: 90% intensity
            _ => 100,                 // 4GB+: 100% maximum intensity!
        }
    }

    /// Apply solid gradient background blocks based on file size intensity
    /// Creates stunning visual hierarchy with darker, more visible colors!
    fn apply_gradient_background(&self, text: &str, intensity: u8) -> String {
        let chars: Vec<char> = text.chars().collect();
        let mut result = String::new();

        // Choose solid background color based on intensity level
        let bg_color = match intensity {
            50..=60 => "\x1b[48;5;17m",  // Small files: Dark blue block
            61..=70 => "\x1b[48;5;22m",  // Medium files: Dark green block
            71..=75 => "\x1b[48;5;58m",  // Large files: Dark yellow/brown block
            76..=80 => "\x1b[48;5;94m",  // 100-200MB: Dark orange block (60%)
            81..=85 => "\x1b[48;5;130m", // 200-500MB: Darker orange block (70%)
            86..=90 => "\x1b[48;5;124m", // 500MB-1GB: Dark red block (80%)
            91..=95 => "\x1b[48;5;88m",  // 1-4GB: Very dark red block (90%)
            96..=100 => "\x1b[48;5;52m", // 4GB+: Maximum dark red block (100%)
            _ => "\x1b[48;5;236m",       // Fallback: Dark gray
        };

        // Apply solid background to entire tree structure as a block
        for &ch in chars.iter() {
            result.push_str(&format!("{}{}", bg_color, ch));
        }

        // Add reset at the end to prevent color bleeding
        result.push_str("\x1b[0m");
        result
    }

    /// Get context-aware emoji based on file type and node properties
    /// Now uses the centralized emoji mapper for consistent, rich file type representation
    fn get_file_emoji(&self, node: &FileNode) -> &'static str {
        emoji_mapper::get_file_emoji(node, self.no_emoji)
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

        // Sort children based on sort field
        for children in children_map.values_mut() {
            let sort_field = self.sort_field.clone();
            children.sort_by(|&a, &b| {
                let node_a = &sorted_nodes[a];
                let node_b = &sorted_nodes[b];

                // Apply custom sorting if specified
                match sort_field.as_deref() {
                    Some("size") | Some("largest") => node_b.size.cmp(&node_a.size),
                    Some("smallest") => node_a.size.cmp(&node_b.size),
                    Some("date") | Some("newest") => node_b.modified.cmp(&node_a.modified),
                    Some("oldest") => node_a.modified.cmp(&node_b.modified),
                    Some("name") | Some("a-to-z") => {
                        let name_a = node_a
                            .path
                            .file_name()
                            .unwrap_or_default()
                            .to_string_lossy();
                        let name_b = node_b
                            .path
                            .file_name()
                            .unwrap_or_default()
                            .to_string_lossy();
                        name_a.cmp(&name_b)
                    }
                    Some("z-to-a") => {
                        let name_a = node_a
                            .path
                            .file_name()
                            .unwrap_or_default()
                            .to_string_lossy();
                        let name_b = node_b
                            .path
                            .file_name()
                            .unwrap_or_default()
                            .to_string_lossy();
                        name_b.cmp(&name_a)
                    }
                    Some("type") => {
                        // Sort by extension, then by name
                        let ext_a = node_a
                            .path
                            .extension()
                            .unwrap_or_default()
                            .to_string_lossy();
                        let ext_b = node_b
                            .path
                            .extension()
                            .unwrap_or_default()
                            .to_string_lossy();
                        match ext_a.cmp(&ext_b) {
                            std::cmp::Ordering::Equal => {
                                let name_a = node_a
                                    .path
                                    .file_name()
                                    .unwrap_or_default()
                                    .to_string_lossy();
                                let name_b = node_b
                                    .path
                                    .file_name()
                                    .unwrap_or_default()
                                    .to_string_lossy();
                                name_a.cmp(&name_b)
                            }
                            other => other,
                        }
                    }
                    _ => {
                        // Default: directories first, then by name
                        match (node_a.is_dir, node_b.is_dir) {
                            (true, false) => std::cmp::Ordering::Less,
                            (false, true) => std::cmp::Ordering::Greater,
                            _ => node_a.path.file_name().cmp(&node_b.path.file_name()),
                        }
                    }
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

            // Database
            FileCategory::Database => Some(Color::TrueColor {
                r: 139,
                g: 90,
                b: 43,
            }), // Brown

            // Office & Documents
            FileCategory::Office => Some(Color::TrueColor {
                r: 21,
                g: 101,
                b: 192,
            }), // Word blue
            FileCategory::Spreadsheet => Some(Color::TrueColor {
                r: 30,
                g: 123,
                b: 64,
            }), // Excel green
            FileCategory::PowerPoint => Some(Color::TrueColor {
                r: 209,
                g: 69,
                b: 36,
            }), // PowerPoint red
            FileCategory::Pdf => Some(Color::TrueColor {
                r: 215,
                g: 41,
                b: 42,
            }), // PDF red
            FileCategory::Ebook => Some(Color::TrueColor {
                r: 65,
                g: 105,
                b: 225,
            }), // Royal blue

            // Text Variants
            FileCategory::Log => Some(Color::TrueColor {
                r: 128,
                g: 128,
                b: 128,
            }), // Gray
            FileCategory::Config => Some(Color::TrueColor {
                r: 100,
                g: 149,
                b: 237,
            }), // Cornflower blue
            FileCategory::License => Some(Color::TrueColor {
                r: 255,
                g: 193,
                b: 7,
            }), // Amber
            FileCategory::Readme => Some(Color::TrueColor {
                r: 0,
                g: 176,
                b: 255,
            }), // Deep sky blue
            FileCategory::Txt => Some(Color::TrueColor {
                r: 160,
                g: 160,
                b: 160,
            }), // Light gray
            FileCategory::Rtf => Some(Color::TrueColor {
                r: 0,
                g: 128,
                b: 128,
            }), // Teal
            FileCategory::Csv => Some(Color::TrueColor {
                r: 46,
                g: 125,
                b: 50,
            }), // Green

            // Security & Crypto
            FileCategory::Certificate => Some(Color::TrueColor {
                r: 255,
                g: 87,
                b: 34,
            }), // Deep orange
            FileCategory::Encrypted => Some(Color::TrueColor {
                r: 156,
                g: 39,
                b: 176,
            }), // Purple

            // Fonts
            FileCategory::Font => Some(Color::TrueColor {
                r: 103,
                g: 58,
                b: 183,
            }), // Deep purple

            // Disk Images
            FileCategory::DiskImage => Some(Color::TrueColor {
                r: 33,
                g: 33,
                b: 33,
            }), // Very dark gray

            // 3D & CAD
            FileCategory::Model3D => Some(Color::TrueColor {
                r: 255,
                g: 152,
                b: 0,
            }), // Orange

            // Scientific & Data
            FileCategory::Jupyter => Some(Color::TrueColor {
                r: 255,
                g: 109,
                b: 0,
            }), // Deep orange
            FileCategory::RData => Some(Color::TrueColor {
                r: 39,
                g: 99,
                b: 165,
            }), // R blue
            FileCategory::Matlab => Some(Color::TrueColor {
                r: 237,
                g: 119,
                b: 3,
            }), // MATLAB orange

            // Web Assets
            FileCategory::WebAsset => Some(Color::TrueColor {
                r: 96,
                g: 125,
                b: 139,
            }), // Blue grey

            // Package & Dependencies
            FileCategory::Package => Some(Color::TrueColor {
                r: 203,
                g: 31,
                b: 26,
            }), // NPM red
            FileCategory::Lock => Some(Color::TrueColor {
                r: 171,
                g: 71,
                b: 188,
            }), // Medium purple

            // Testing
            FileCategory::Test => Some(Color::TrueColor {
                r: 67,
                g: 160,
                b: 71,
            }), // Test green

            // Memory Files (MEM|8!)
            FileCategory::Memory => Some(Color::TrueColor {
                r: 147,
                g: 112,
                b: 219,
            }), // Medium purple (brain-like)

            // Others
            FileCategory::Backup => Some(Color::TrueColor {
                r: 189,
                g: 189,
                b: 189,
            }), // Silver
            FileCategory::Temp => Some(Color::TrueColor {
                r: 117,
                g: 117,
                b: 117,
            }), // Gray

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
