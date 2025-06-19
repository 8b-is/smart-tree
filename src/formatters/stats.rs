use super::Formatter;
use crate::scanner::{FileNode, TreeStats};
use anyhow::Result;
use chrono::{DateTime, Local};
use humansize::{format_size, BINARY};
use std::io::Write;
use std::path::Path;

pub struct StatsFormatter;

impl StatsFormatter {
    pub fn new() -> Self {
        Self
    }
}

impl Formatter for StatsFormatter {
    fn format(
        &self,
        writer: &mut dyn Write,
        _nodes: &[FileNode],
        stats: &TreeStats,
        root_path: &Path,
    ) -> Result<()> {
        writeln!(writer, "{}", "=".repeat(60))?;
        writeln!(writer, "Directory Statistics for: {}", root_path.display())?;
        writeln!(writer, "{}", "=".repeat(60))?;
        writeln!(writer, "Total Files: {} ({:x} hex)", stats.total_files, stats.total_files)?;
        writeln!(writer, "Total Directories: {} ({:x} hex)", stats.total_dirs, stats.total_dirs)?;
        writeln!(
            writer,
            "Total Size: {} bytes ({:x} hex) ({})",
            stats.total_size,
            stats.total_size,
            format_size(stats.total_size, BINARY)
        )?;
        writeln!(writer)?;
        
        // File types by count
        if !stats.file_types.is_empty() {
            writeln!(writer, "File Types (by count):")?;
            let mut types: Vec<_> = stats.file_types.iter().collect();
            types.sort_by(|a, b| b.1.cmp(a.1));
            
            for (ext, count) in types.iter().take(20) {
                writeln!(writer, "  .{}: {}", ext, count)?;
            }
            writeln!(writer)?;
        }
        
        // Largest files
        if !stats.largest_files.is_empty() {
            writeln!(writer, "Largest Files:")?;
            for (size, path) in stats.largest_files.iter().take(10) {
                let rel_path = path.strip_prefix(root_path).unwrap_or(path);
                writeln!(
                    writer,
                    "  {:>12} bytes ({:>10x} hex) {:>8}  {}",
                    size,
                    size,
                    format_size(*size, BINARY),
                    rel_path.display()
                )?;
            }
            writeln!(writer)?;
        }
        
        // Newest files
        if !stats.newest_files.is_empty() {
            writeln!(writer, "Newest Files:")?;
            for (mtime, path) in stats.newest_files.iter().take(5) {
                let datetime = DateTime::<Local>::from(*mtime);
                let rel_path = path.strip_prefix(root_path).unwrap_or(path);
                writeln!(
                    writer,
                    "  {}  {}",
                    datetime.format("%Y-%m-%d %H:%M"),
                    rel_path.display()
                )?;
            }
            writeln!(writer)?;
        }
        
        // Oldest files
        if !stats.oldest_files.is_empty() {
            writeln!(writer, "Oldest Files:")?;
            for (mtime, path) in stats.oldest_files.iter().take(5) {
                let datetime = DateTime::<Local>::from(*mtime);
                let rel_path = path.strip_prefix(root_path).unwrap_or(path);
                writeln!(
                    writer,
                    "  {}  {}",
                    datetime.format("%Y-%m-%d %H:%M"),
                    rel_path.display()
                )?;
            }
        }
        
        Ok(())
    }
}