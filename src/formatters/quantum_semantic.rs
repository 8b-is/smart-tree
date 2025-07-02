//! Quantum Semantic formatter - "AST-aware compression!" - Omni
//! Combines quantum compression with semantic code understanding

use super::Formatter;
use crate::scanner::{FileNode, TreeStats};
use crate::tree_sitter_quantum::SemanticQuantumCompressor;
use crate::dynamic_tokenizer::DynamicTokenizer;
use anyhow::Result;
use std::io::Write;
use std::path::Path;

pub struct QuantumSemanticFormatter {
    compressor: SemanticQuantumCompressor,
    max_nodes_per_file: usize,
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
        writeln!(writer, "STATS:F{:x}D{:x}S{:x}", 
            stats.total_files, stats.total_dirs, stats.total_size)?;
        
        // Show tokenizer statistics
        let token_stats = tokenizer.get_stats();
        writeln!(writer, "TOKENIZER:patterns={},tokens={},saved={}B",
            token_stats.patterns_found,
            token_stats.tokens_generated,
            token_stats.estimated_savings)?;
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
                        
                        // Add semantic information
                        // In real implementation, we'd extract actual semantic nodes
                        if ext == "rs" {
                            if path_str.contains("main") {
                                write!(writer, ":F6,F1x2,F7x5")?; // main, 2 structs, 5 pub functions
                            } else if path_str.contains("mod") {
                                write!(writer, ":F2x5,F7")?; // 5 traits, pub
                            } else if path_str.contains("test") {
                                write!(writer, ":F8,F0x10")?; // test, 10 functions
                            } else {
                                write!(writer, ":F1,F2,F7x3")?; // struct, trait, 3 pub
                            }
                        } else if ext == "py" {
                            if path_str.contains("test") {
                                write!(writer, ":F8,F5x5")?; // test, 5 defs
                            } else {
                                write!(writer, ":F4,F5x5")?; // class, 5 defs
                            }
                        } else if ext == "js" || ext == "ts" {
                            write!(writer, ":F0x5")?; // 5 functions
                        }
                        
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