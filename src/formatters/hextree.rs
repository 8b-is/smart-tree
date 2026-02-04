//! HexTree Formatter - Quantum compression meets readable tree structure
//!
//! Combines the token efficiency of quantum format with human/AI readable output.
//! Uses ULTRA_V2 traversal codes rendered as visible Unicode symbols.
//!
//! Format:
//! ```
//! HEXTREE_V1:
//! KEY: ↓=enter ·=same ↑=exit │=tree @=rust #=py $=js
//! TOK: 80=src 81=tests 82=mod.rs 83=lib.rs
//! ---
//! │project↓
//! │ 80↓
//! │  main.rs·4k2
//! │  lib.rs·1k8
//! │  82↑F3S6k
//! │ 81↓
//! │  test_main.rs↑F1S512
//! ↑F4D2S6k5
//! ```

use super::Formatter;
use crate::scanner::{FileNode, TreeStats};
use anyhow::Result;
use std::collections::HashMap;
use std::io::Write;
use std::path::Path;

/// HexTree - The readable quantum format
pub struct HexTreeFormatter {
    /// Dynamic tokens learned from the tree
    tokens: HashMap<String, u8>,
    /// Next available token ID
    next_token: u8,
    /// Minimum occurrences to earn a token
    min_occurrences: usize,
}

impl Default for HexTreeFormatter {
    fn default() -> Self {
        Self::new()
    }
}

impl HexTreeFormatter {
    pub fn new() -> Self {
        Self {
            tokens: HashMap::new(),
            next_token: 0x80,
            min_occurrences: 2,
        }
    }

    /// Learn tokens from analyzing the tree
    fn learn_tokens(&mut self, nodes: &[FileNode]) {
        let mut occurrences: HashMap<String, usize> = HashMap::new();

        // Count directory names and common extensions
        for node in nodes {
            if let Some(name) = node.path.file_name() {
                let name_str = name.to_string_lossy().to_string();

                // Count directory names
                if node.is_dir {
                    *occurrences.entry(name_str.clone()).or_insert(0) += 1;
                }

                // Count file stems for common patterns
                if let Some(stem) = node.path.file_stem() {
                    let stem_str = stem.to_string_lossy().to_string();
                    if stem_str == "mod"
                        || stem_str == "lib"
                        || stem_str == "main"
                        || stem_str == "index"
                        || stem_str == "test"
                    {
                        *occurrences.entry(format!("{}.rs", stem_str)).or_insert(0) += 1;
                        *occurrences.entry(format!("{}.py", stem_str)).or_insert(0) += 1;
                        *occurrences.entry(format!("{}.js", stem_str)).or_insert(0) += 1;
                    }
                }
            }
        }

        // Assign tokens to frequently occurring names
        let mut sorted: Vec<_> = occurrences.into_iter().collect();
        sorted.sort_by(|a, b| b.1.cmp(&a.1));

        for (name, count) in sorted {
            if count >= self.min_occurrences && self.next_token < 0xFF {
                self.tokens.insert(name, self.next_token);
                self.next_token += 1;
            }
            if self.tokens.len() >= 32 {
                break; // Limit token count for readability
            }
        }
    }

    /// Format a size in compact hex notation
    fn format_size(size: u64) -> String {
        if size == 0 {
            return "0".to_string();
        }

        // Use suffixes: k=1024, m=1048576, g=1073741824
        if size >= 1073741824 {
            format!("{:x}g", size / 1073741824)
        } else if size >= 1048576 {
            format!("{:x}m", size / 1048576)
        } else if size >= 1024 {
            format!("{:x}k", size / 1024)
        } else {
            format!("{:x}", size)
        }
    }

    /// Tokenize a name if possible
    fn tokenize(&self, name: &str) -> String {
        if let Some(&token) = self.tokens.get(name) {
            format!("{:X}", token)
        } else {
            name.to_string()
        }
    }

    /// Get language marker for file extension
    fn lang_marker(ext: Option<&str>) -> &'static str {
        match ext {
            Some("rs") => "@",
            Some("py") => "#",
            Some("js" | "jsx" | "ts" | "tsx") => "$",
            Some("md") => "%",
            Some("toml" | "yaml" | "yml" | "json") => "&",
            _ => "",
        }
    }
}

impl Formatter for HexTreeFormatter {
    fn format(
        &self,
        writer: &mut dyn Write,
        nodes: &[FileNode],
        stats: &TreeStats,
        root_path: &Path,
    ) -> Result<()> {
        let mut formatter = HexTreeFormatter::new();

        // Learn tokens from the tree
        formatter.learn_tokens(nodes);

        // Write header
        writeln!(writer, "HEXTREE_V1:")?;
        writeln!(writer, "KEY: ↓=enter ·=same ↑=exit")?;
        writeln!(writer, "EXT: @=rs #=py $=js %=md &=cfg")?;

        // Write token definitions if any
        if !formatter.tokens.is_empty() {
            write!(writer, "TOK:")?;
            let mut token_list: Vec<_> = formatter.tokens.iter().collect();
            token_list.sort_by_key(|(_, &v)| v);
            for (name, &id) in token_list.iter().take(16) {
                write!(writer, " {:X}={}", id, name)?;
            }
            writeln!(writer)?;
        }

        writeln!(writer, "ROOT:{}", root_path.display())?;
        writeln!(writer, "---")?;

        // Track directory state for summaries
        struct DirState {
            depth: usize,
            file_count: usize,
            total_size: u64,
        }
        let mut dir_stack: Vec<DirState> = vec![];
        let mut prev_depth = 0;

        for node in nodes {
            let depth = node.depth;

            // Handle depth changes - close directories
            while prev_depth > depth {
                if let Some(state) = dir_stack.pop() {
                    // Write directory summary on exit
                    let indent = "  ".repeat(state.depth);
                    writeln!(
                        writer,
                        "{}↑F{}S{}",
                        indent,
                        state.file_count,
                        Self::format_size(state.total_size)
                    )?;
                }
                prev_depth -= 1;
            }

            let indent = "  ".repeat(depth);
            let name = node
                .path
                .file_name()
                .map(|n| n.to_string_lossy().to_string())
                .unwrap_or_else(|| node.path.to_string_lossy().to_string());

            let tokenized = formatter.tokenize(&name);

            if node.is_dir {
                // Directory - going deeper
                writeln!(writer, "{}{}↓", indent, tokenized)?;
                dir_stack.push(DirState {
                    depth,
                    file_count: 0,
                    total_size: 0,
                });
                prev_depth = depth + 1;
            } else {
                // File - same level
                let ext = node.path.extension().and_then(|e| e.to_str());
                let lang = Self::lang_marker(ext);
                let size = Self::format_size(node.size);
                writeln!(writer, "{}{}{}·{}", indent, lang, tokenized, size)?;

                // Update parent directory stats
                if let Some(parent) = dir_stack.last_mut() {
                    parent.file_count += 1;
                    parent.total_size += node.size;
                }
            }
        }

        // Close remaining directories
        while let Some(state) = dir_stack.pop() {
            let indent = "  ".repeat(state.depth);
            writeln!(
                writer,
                "{}↑F{}S{}",
                indent,
                state.file_count,
                Self::format_size(state.total_size)
            )?;
        }

        // Final stats
        writeln!(writer, "---")?;
        writeln!(
            writer,
            "TOTAL:F{:x}D{:x}S{}",
            stats.total_files,
            stats.total_dirs,
            Self::format_size(stats.total_size)
        )?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::scanner::{FileCategory, FileType, FilesystemType};
    use std::path::PathBuf;
    use std::time::SystemTime;

    fn make_node(path: &str, is_dir: bool, size: u64, depth: usize) -> FileNode {
        FileNode {
            path: PathBuf::from(path),
            is_dir,
            size,
            depth,
            permissions: 0o644,
            uid: 1000,
            gid: 1000,
            modified: SystemTime::now(),
            is_symlink: false,
            is_hidden: false,
            permission_denied: false,
            is_ignored: false,
            file_type: FileType::RegularFile,
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
    fn test_hextree_basic() {
        let nodes = vec![
            make_node("project", true, 0, 0),
            make_node("project/src", true, 0, 1),
            make_node("project/src/main.rs", false, 4096, 2),
            make_node("project/src/lib.rs", false, 2048, 2),
        ];

        let stats = TreeStats {
            total_files: 2,
            total_dirs: 2,
            total_size: 6144,
            ..Default::default()
        };

        let formatter = HexTreeFormatter::new();
        let mut output = Vec::new();
        formatter
            .format(&mut output, &nodes, &stats, Path::new("project"))
            .unwrap();

        let result = String::from_utf8(output).unwrap();
        assert!(result.contains("HEXTREE_V1:"));
        assert!(result.contains("↓")); // Directory marker
        assert!(result.contains("·")); // File marker
    }
}
