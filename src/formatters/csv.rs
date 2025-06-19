use super::Formatter;
use crate::scanner::{FileNode, TreeStats};
use anyhow::Result;
use chrono::{DateTime, Local};
use csv::Writer;
use std::io::Write;
use std::path::Path;

pub struct CsvFormatter;

impl CsvFormatter {
    pub fn new() -> Self {
        Self
    }
}

impl Formatter for CsvFormatter {
    fn format(
        &self,
        writer: &mut dyn Write,
        nodes: &[FileNode],
        _stats: &TreeStats,
        root_path: &Path,
    ) -> Result<()> {
        let mut csv_writer = Writer::from_writer(writer);
        
        // Write header
        csv_writer.write_record(&[
            "path",
            "type",
            "size",
            "permissions",
            "uid",
            "gid",
            "modified",
            "depth",
        ])?;
        
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
            
            csv_writer.write_record(&[
                rel_path,
                file_type.to_string(),
                node.size.to_string(),
                format!("{:o}", node.permissions),
                node.uid.to_string(),
                node.gid.to_string(),
                datetime.to_rfc3339(),
                node.depth.to_string(),
            ])?;
        }
        
        csv_writer.flush()?;
        Ok(())
    }
}