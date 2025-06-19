pub mod hex;
pub mod classic;
pub mod json;
pub mod ai;
pub mod ai_json;
pub mod stats;
pub mod csv;
pub mod tsv;
pub mod digest;

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