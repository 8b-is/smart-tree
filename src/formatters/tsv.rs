use super::Formatter;
use crate::scanner::{FileNode, TreeStats};
use anyhow::Result;
use chrono::{DateTime, Local};
use std::io::Write;
use std::path::Path;

pub struct TsvFormatter;

impl Default for TsvFormatter {
    fn default() -> Self {
        Self::new()
    }
}

impl TsvFormatter {
    pub fn new() -> Self {
        Self
    }
}

impl Formatter for TsvFormatter {
    fn format(
        &self,
        writer: &mut dyn Write,
        nodes: &[FileNode],
        _stats: &TreeStats,
        root_path: &Path,
    ) -> Result<()> {
        // Write header
        writeln!(
            writer,
            "path\ttype\tsize\tpermissions\tuid\tgid\tmodified\tdepth"
        )?;

        // Sort nodes by path
        let mut sorted_nodes = nodes.to_vec();
        sorted_nodes.sort_by(|a, b| a.path.cmp(&b.path));

        for node in &sorted_nodes {
            let rel_path = if node.path == root_path {
                ".".to_string()
            } else {
                node.path
                    .strip_prefix(root_path)
                    .unwrap_or(&node.path)
                    .to_string_lossy()
                    .to_string()
            };

            let file_type = if node.is_dir { "d" } else { "f" };
            let datetime = DateTime::<Local>::from(node.modified);

            writeln!(
                writer,
                "{}\t{}\t{}\t{:o}\t{}\t{}\t{}\t{}",
                rel_path,
                file_type,
                node.size,
                node.permissions,
                node.uid,
                node.gid,
                datetime.to_rfc3339(),
                node.depth
            )?;
        }

        Ok(())
    }
}
