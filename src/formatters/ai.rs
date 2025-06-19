use super::{Formatter, hex::HexFormatter, PathDisplayMode};
use crate::scanner::{FileNode, TreeStats};
use crate::context::detect_project_context;
use anyhow::Result;
use sha2::{Sha256, Digest};
use std::io::Write;
use std::path::Path;

pub struct AiFormatter {
    hex_formatter: HexFormatter,
}

impl AiFormatter {
    pub fn new(no_emoji: bool, _path_mode: PathDisplayMode) -> Self {
        Self {
            // AI format should always use PathDisplayMode::Off for maximum compactness
            // unless explicitly requested otherwise
            hex_formatter: HexFormatter::new(false, no_emoji, true, PathDisplayMode::Off),
        }
    }
    
    /// Calculate a SHA256 hash of the tree structure for consistency verification
    fn calculate_tree_hash(&self, nodes: &[FileNode]) -> String {
        let mut hasher = Sha256::new();
        
        // Hash each node's key properties in a deterministic way
        for node in nodes {
            // Hash: depth, name, type (dir/file), size, permissions
            hasher.update(node.depth.to_le_bytes());
            hasher.update(node.path.file_name().unwrap_or_default().to_string_lossy().as_bytes());
            hasher.update(&[if node.is_dir { 1 } else { 0 }]);
            hasher.update(node.size.to_le_bytes());
            hasher.update(node.permissions.to_le_bytes());
        }
        
        // Return first 16 chars of hex for brevity
        let result = hasher.finalize();
        hex::encode(&result[..8])
    }
}

impl Formatter for AiFormatter {
    fn format(
        &self,
        writer: &mut dyn Write,
        nodes: &[FileNode],
        stats: &TreeStats,
        root_path: &Path,
    ) -> Result<()> {
        // First print the hex tree header
        writeln!(writer, "TREE_HEX_V1:")?;
        
        // Optionally add project context if detected
        if let Some(context) = detect_project_context(root_path) {
            writeln!(writer, "CONTEXT: {}", context)?;
        }
        
        // Calculate SHA256 hash of the tree structure
        let tree_hash = self.calculate_tree_hash(nodes);
        writeln!(writer, "HASH: {}", tree_hash)?;
        
        // Use hex formatter for the tree
        self.hex_formatter.format(writer, nodes, stats, root_path)?;
        
        // Then print compact statistics - all in hex for consistency
        writeln!(writer, "\nSTATS:")?;
        writeln!(
            writer,
            "F:{:x} D:{:x} S:{:x} ({:.1}MB)",
            stats.total_files,
            stats.total_dirs,
            stats.total_size,
            stats.total_size as f64 / (1024.0 * 1024.0)
        )?;
        
        // File type summary (top 10) - counts in hex
        if !stats.file_types.is_empty() {
            let mut types: Vec<_> = stats.file_types.iter().collect();
            types.sort_by(|a, b| b.1.cmp(a.1));
            
            let types_str: Vec<String> = types
                .iter()
                .take(10)
                .map(|(ext, count)| format!("{}:{:x}", ext, count))
                .collect();
            
            writeln!(writer, "TYPES: {}", types_str.join(" "))?;
        }
        
        // Largest files (top 5)
        if !stats.largest_files.is_empty() {
            let large_str: Vec<String> = stats.largest_files
                .iter()
                .take(5)
                .map(|(size, path)| {
                    let name = path.file_name()
                        .unwrap_or(path.as_os_str())
                        .to_string_lossy();
                    format!("{}:{:x}", name, size)
                })
                .collect();
            
            writeln!(writer, "LARGE: {}", large_str.join(" "))?;
        }
        
        // Date range
        if !stats.oldest_files.is_empty() && !stats.newest_files.is_empty() {
            let oldest = stats.oldest_files[0].0
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs();
            let newest = stats.newest_files[0].0
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs();
            
            writeln!(writer, "DATES: {:x}-{:x}", oldest, newest)?;
        }
        
        writeln!(writer, "END_AI")?;
        
        Ok(())
    }
}