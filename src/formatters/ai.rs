use super::{Formatter, hex::HexFormatter};
use crate::scanner::{FileNode, TreeStats};
use anyhow::Result;
use std::io::Write;
use std::path::Path;

pub struct AiFormatter {
    hex_formatter: HexFormatter,
}

impl AiFormatter {
    pub fn new(no_emoji: bool) -> Self {
        Self {
            hex_formatter: HexFormatter::new(false, no_emoji, true),
        }
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
        
        // Use hex formatter for the tree
        self.hex_formatter.format(writer, nodes, stats, root_path)?;
        
        // Then print compact statistics
        writeln!(writer, "\nSTATS:")?;
        writeln!(
            writer,
            "F:{} D:{} S:{:x} ({:.1}MB)",
            stats.total_files,
            stats.total_dirs,
            stats.total_size,
            stats.total_size as f64 / (1024.0 * 1024.0)
        )?;
        
        // File type summary (top 10)
        if !stats.file_types.is_empty() {
            let mut types: Vec<_> = stats.file_types.iter().collect();
            types.sort_by(|a, b| b.1.cmp(a.1));
            
            let types_str: Vec<String> = types
                .iter()
                .take(10)
                .map(|(ext, count)| format!("{}:{}", ext, count))
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