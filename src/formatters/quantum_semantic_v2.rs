//! Quantum Semantic V2 - Ultra-compressed with binary tokens
//! "Maximum semantic density!" - Omni

use super::Formatter;
use crate::scanner::{FileNode, TreeStats};
use anyhow::Result;
use std::io::Write;
use std::path::Path;

pub struct QuantumSemanticV2Formatter;

impl QuantumSemanticV2Formatter {
    pub fn new() -> Self {
        Self
    }
}

impl Formatter for QuantumSemanticV2Formatter {
    fn format(
        &self,
        writer: &mut dyn Write,
        nodes: &[FileNode],
        stats: &TreeStats,
        root_path: &Path,
    ) -> Result<()> {
        // Ultra-compressed header
        write!(writer, "QS2:")?;
        
        // Stats in minimal hex
        write!(writer, "{:x},{:x},{:x};", 
            stats.total_files, stats.total_dirs, stats.total_size)?;
        
        // Legend for semantic tokens (single byte)
        // Languages: @ = Rust, # = Python, $ = JS, % = TS
        // Elements: F = fn, S = struct, T = trait, I = impl, C = class, D = def
        // Importance: ! = 1.0, + = 0.9, ~ = 0.6, - = 0.3
        
        let mut last_lang = '\0';
        
        for node in nodes {
            if !node.is_dir {
                if let Some(ext) = node.path.extension().and_then(|e| e.to_str()) {
                    let lang = match ext {
                        "rs" => '@',
                        "py" => '#',
                        "js" => '$',
                        "ts" => '%',
                        _ => '\0',
                    };
                    
                    if lang != '\0' {
                        // Language marker only if changed
                        if lang != last_lang {
                            write!(writer, "{}", lang)?;
                            last_lang = lang;
                        }
                        
                        // File path compressed
                        let relative = node.path.strip_prefix(root_path).unwrap_or(&node.path);
                        let path_str = relative.to_string_lossy();
                        
                        // Remove common parts
                        let compressed_path = path_str
                            .replace("formatters/", "f/")
                            .replace("src/", "")
                            .replace(".rs", "")
                            .replace(".py", "");
                        
                        write!(writer, "{}:", compressed_path)?;
                        
                        // Semantic tokens (simulated)
                        if path_str.contains("main") {
                            write!(writer, "F!")?; // main function, importance 1.0
                        }
                        
                        if ext == "rs" {
                            write!(writer, "S+T+F+F+F~")?; // struct, trait, 3 pub fns, 1 private
                        } else if ext == "py" {
                            write!(writer, "C+D+D+D~D-")?; // class, __init__, methods, test
                        }
                        
                        write!(writer, ";")?;
                    }
                }
            }
        }
        
        write!(writer, ".")?; // End marker
        Ok(())
    }
}