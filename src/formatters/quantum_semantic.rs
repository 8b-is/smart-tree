//! Quantum Semantic formatter - "AST-aware compression!" - Omni
//! Combines quantum compression with semantic code understanding

use super::Formatter;
use crate::dynamic_tokenizer::DynamicTokenizer;
use crate::scanner::{FileNode, TreeStats};
use crate::tree_sitter_quantum::SemanticQuantumCompressor;
use anyhow::Result;
use std::io::Write;
use std::path::Path;

pub struct QuantumSemanticFormatter {
    #[allow(dead_code)]
    compressor: SemanticQuantumCompressor,
    #[allow(dead_code)]
    max_nodes_per_file: usize,
}

impl Default for QuantumSemanticFormatter {
    fn default() -> Self {
        Self::new()
    }
}

impl QuantumSemanticFormatter {
    pub fn new() -> Self {
        Self {
            compressor: SemanticQuantumCompressor::new(),
            max_nodes_per_file: 10, // Extract top 10 semantic nodes per file
        }
    }
}

impl Formatter for QuantumSemanticFormatter {
    fn format(
        &self,
        writer: &mut dyn Write,
        nodes: &[FileNode],
        stats: &TreeStats,
        root_path: &Path,
    ) -> Result<()> {
        // First pass: analyze the project to learn patterns
        let mut tokenizer = DynamicTokenizer::new();
        tokenizer.analyze(nodes);

        // Header with dynamically generated token definitions
        writeln!(writer, "QUANTUM_SEMANTIC_V2:")?;
        write!(writer, "{}", tokenizer.get_token_header())?;

        // Add semantic tokens (these are still hardcoded as they're language-specific)
        writeln!(writer, "  F0=fn")?;
        writeln!(writer, "  F1=struct")?;
        writeln!(writer, "  F2=trait")?;
        writeln!(writer, "  F3=impl")?;
        writeln!(writer, "  F4=class")?;
        writeln!(writer, "  F5=def")?;
        writeln!(writer, "  F6=main[1.0]")?;
        writeln!(writer, "  F7=pub[0.9]")?;
        writeln!(writer, "  F8=test[0.3]")?;
        writeln!(writer)?;

        // Stats
        writeln!(writer, "ROOT:{}", root_path.display())?;
        writeln!(
            writer,
            "STATS:F{:x}D{:x}S{:x}",
            stats.total_files, stats.total_dirs, stats.total_size
        )?;

        // Show tokenizer statistics
        let token_stats = tokenizer.get_stats();
        writeln!(
            writer,
            "TOKENIZER:patterns={},tokens={},saved={}B",
            token_stats.patterns_found, token_stats.tokens_generated, token_stats.estimated_savings
        )?;
        writeln!(writer)?;
        writeln!(writer, "DATA:")?;

        // Process source code files with dynamic token compression
        for node in nodes {
            if !node.is_dir {
                // Get relative path
                let relative = node.path.strip_prefix(root_path).unwrap_or(&node.path);
                let path_str = relative.to_string_lossy().to_string();

                // Compress the path using learned tokens
                let compressed_path = tokenizer.compress_path(&path_str);

                if let Some(ext) = node.path.extension().and_then(|e| e.to_str()) {
                    // Determine language marker
                    let lang_marker = match ext {
                        "rs" => "@",
                        "py" => "#",
                        "js" => "$",
                        "ts" => "%",
                        _ => "",
                    };

                    if !lang_marker.is_empty() {
                        write!(writer, "{}", lang_marker)?;
                        write!(writer, "{}", compressed_path)?;

                        // Extract and write semantic nodes
                        // For now, using placeholder semantic info
                        // TODO: Integrate actual tree-sitter parsing

                        writeln!(writer)?;
                    }
                }
            }
        }

        writeln!(writer)?;
        writeln!(writer, "END_QS")?;

        Ok(())
    }
}
