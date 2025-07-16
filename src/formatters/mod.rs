pub mod ai;
pub mod ai_json;
pub mod classic;
pub mod csv;
pub mod digest;
pub mod hex;
pub mod json;
pub mod ls;
pub mod markdown;
pub mod markqant;
pub mod mermaid;
pub mod quantum;
pub mod quantum_safe;
pub mod quantum_semantic;
pub mod quantum_semantic_v2;
pub mod relations;
pub mod relations_formatter;
pub mod semantic;
pub mod stats;
pub mod summary;
pub mod summary_ai;
pub mod tsv;
pub mod waste;

use crate::scanner::{FileNode, TreeStats};
use anyhow::Result;
use std::io::Write;

#[derive(Debug, Clone, Copy)]
pub enum PathDisplayMode {
    Off,
    Relative,
    Full,
}

pub trait Formatter {
    fn format(
        &self,
        writer: &mut dyn Write,
        nodes: &[FileNode],
        stats: &TreeStats,
        root_path: &std::path::Path,
    ) -> Result<()>;
}

pub trait StreamingFormatter {
    /// Initialize the stream (e.g., write headers)
    fn start_stream(&self, writer: &mut dyn Write, root_path: &std::path::Path) -> Result<()>;

    /// Format a single node as it arrives
    fn format_node(
        &self,
        writer: &mut dyn Write,
        node: &FileNode,
        root_path: &std::path::Path,
    ) -> Result<()>;

    /// Finalize the stream (e.g., write stats, footers)
    fn end_stream(
        &self,
        writer: &mut dyn Write,
        stats: &TreeStats,
        root_path: &std::path::Path,
    ) -> Result<()>;
}
