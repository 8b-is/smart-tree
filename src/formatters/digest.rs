use super::Formatter;
use crate::scanner::{FileNode, TreeStats};
use anyhow::Result;
use sha2::{Sha256, Digest};
use std::io::Write;
use std::path::Path;

pub struct DigestFormatter;

impl DigestFormatter {
    pub fn new() -> Self {
        Self
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

impl Formatter for DigestFormatter {
    fn format(
        &self,
        writer: &mut dyn Write,
        nodes: &[FileNode],
        stats: &TreeStats,
        _root_path: &Path,
    ) -> Result<()> {
        // Calculate SHA256 hash of the tree structure
        let tree_hash = self.calculate_tree_hash(nodes);
        
        // First line: Hash and basic stats
        write!(
            writer,
            "HASH: {} F:{} D:{} S:{:x}",
            tree_hash,
            stats.total_files,
            stats.total_dirs,
            stats.total_size,
        )?;
        
        // File type summary (top 5) - super compact
        if !stats.file_types.is_empty() {
            let mut types: Vec<_> = stats.file_types.iter().collect();
            types.sort_by(|a, b| b.1.cmp(a.1));
            
            let types_str: Vec<String> = types
                .iter()
                .take(5)
                .map(|(ext, count)| format!("{}:{}", ext, count))
                .collect();
            
            write!(writer, " TYPES: {}", types_str.join(" "))?;
        }
        
        // Add newline at the end
        writeln!(writer)?;
        
        Ok(())
    }
}