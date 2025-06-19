pub mod hex;
pub mod classic;
pub mod json;
pub mod ai;
pub mod stats;
pub mod csv;
pub mod tsv;

use crate::scanner::{FileNode, TreeStats};
use anyhow::Result;
use std::io::Write;

pub trait Formatter {
    fn format(
        &self,
        writer: &mut dyn Write,
        nodes: &[FileNode],
        stats: &TreeStats,
        root_path: &std::path::Path,
    ) -> Result<()>;
}