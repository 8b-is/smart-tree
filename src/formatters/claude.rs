// Claude Formatter - Optimized format for sending to Anthropic's API
// Combines quantum compression with API-friendly encoding

use super::{Formatter, StreamingFormatter};
use crate::scanner::{FileNode, TreeStats};
use anyhow::Result;
use base64::{Engine as _, engine::general_purpose};
use std::io::Write;
use std::path::Path;

/// Format specifically designed for Claude API transmission
/// Uses quantum compression with base64 encoding for binary safety
pub struct ClaudeFormatter {
    quantum_formatter: super::quantum::QuantumFormatter,
    include_context: bool,
}

impl ClaudeFormatter {
    pub fn new(include_context: bool) -> Self {
        Self {
            quantum_formatter: super::quantum::QuantumFormatter::new(),
            include_context,
        }
    }
    
    fn write_api_wrapper(&self, writer: &mut dyn Write, 
                        quantum_data: &[u8], 
                        stats: &TreeStats,
                        root_path: &Path) -> Result<()> {
        // Find binary data section
        let begin_marker = b"---BEGIN_DATA---\n";
        let end_marker = b"\n---END_DATA---";
        
        let begin_pos = quantum_data.windows(begin_marker.len())
            .position(|w| w == begin_marker)
            .unwrap_or(quantum_data.len());
            
        let end_pos = begin_pos + begin_marker.len() + 
            quantum_data[begin_pos + begin_marker.len()..]
                .windows(end_marker.len())
                .position(|w| w == end_marker)
                .unwrap_or(0);
        
        // Extract header and binary sections
        let header = std::str::from_utf8(&quantum_data[..begin_pos])?;
        let binary = if begin_pos < quantum_data.len() && end_pos > begin_pos + begin_marker.len() {
            &quantum_data[begin_pos + begin_marker.len()..end_pos]
        } else {
            b""
        };
        
        // Base64 encode the binary data
        let binary_b64 = general_purpose::STANDARD.encode(binary);
        
        // Calculate compression metrics
        let original_estimate = stats.total_files * 200 + stats.total_dirs * 100; // Rough estimate
        let compression_ratio = if original_estimate > 0 {
            (binary.len() as f64 / original_estimate as f64) * 100.0
        } else {
            100.0
        };
        
        // Write JSON structure optimized for Claude
        writeln!(writer, "{{")?;
        writeln!(writer, r#"  "format": "smart-tree-quantum-v1","#)?;
        writeln!(writer, r#"  "api_version": "1.0","#)?;
        writeln!(writer, r#"  "root_path": "{}","#, root_path.display())?;
        
        if self.include_context {
            writeln!(writer, r#"  "context": {{"#)?;
            writeln!(writer, r#"    "description": "Ultra-compressed directory structure using Smart Tree quantum format","#)?;
            writeln!(writer, r#"    "features": ["#)?;
            writeln!(writer, r#"      "Bitfield headers for efficient metadata encoding","#)?;
            writeln!(writer, r#"      "Token substitution (e.g., 0x80=node_modules, 0x91=.rs)","#)?;
            writeln!(writer, r#"      "ASCII control codes for tree traversal","#)?;
            writeln!(writer, r#"      "Delta encoding for permissions","#)?;
            writeln!(writer, r#"      "Variable-length size encoding""#)?;
            writeln!(writer, r#"    ],"#)?;
            writeln!(writer, r#"    "benefits": {{"#)?;
            writeln!(writer, r#"      "compression_ratio": "{:.1}%","#, compression_ratio)?;
            writeln!(writer, r#"      "original_size_estimate": {},"#, original_estimate)?;
            writeln!(writer, r#"      "compressed_size": {},"#, binary.len())?;
            writeln!(writer, r#"      "tokens_saved": {}"#, original_estimate.saturating_sub(binary.len() as u64))?;
            writeln!(writer, r#"    }}"#)?;
            writeln!(writer, r#"  }},"#)?;
        }
        
        writeln!(writer, r#"  "header": {},"#, serde_json::to_string(header)?)?;
        writeln!(writer, r#"  "data_base64": "{}","#, binary_b64)?;
        writeln!(writer, r#"  "data_size": {},"#, binary.len())?;
        
        writeln!(writer, r#"  "statistics": {{"#)?;
        writeln!(writer, r#"    "total_files": {},"#, stats.total_files)?;
        writeln!(writer, r#"    "total_dirs": {},"#, stats.total_dirs)?;
        writeln!(writer, r#"    "total_size": {},"#, stats.total_size)?;
        // max_depth is not available in TreeStats, would need to calculate during scan
        writeln!(writer, r#"    "compression_ratio": "{:.1}%""#, compression_ratio)?;
        writeln!(writer, r#"  }},"#)?;
        
        writeln!(writer, r#"  "usage_hints": ["#)?;
        writeln!(writer, r#"    "The data_base64 field contains the binary quantum format encoded in base64","#)?;
        writeln!(writer, r#"    "Use the header information to understand token mappings","#)?;
        writeln!(writer, r#"    "ASCII codes: 0x0E=enter dir, 0x0F=exit dir, 0x0B=same level","#)?;
        writeln!(writer, r#"    "Permission deltas are XOR differences from parent (e.g., 0x0049 = 755^644)""#)?;
        writeln!(writer, r#"  ]"#)?;
        
        writeln!(writer, "}}")?;
        
        Ok(())
    }
}

impl Formatter for ClaudeFormatter {
    fn format(
        &self,
        writer: &mut dyn Write,
        nodes: &[FileNode],
        stats: &TreeStats,
        root_path: &Path,
    ) -> Result<()> {
        // First, generate quantum format
        let mut quantum_buffer = Vec::new();
        self.quantum_formatter.format(&mut quantum_buffer, nodes, stats, root_path)?;
        
        // Then wrap it for API
        self.write_api_wrapper(writer, &quantum_buffer, stats, root_path)?;
        
        Ok(())
    }
}

impl StreamingFormatter for ClaudeFormatter {
    fn start_stream(&self, writer: &mut dyn Write, root_path: &Path) -> Result<()> {
        writeln!(writer, "{{")?;
        writeln!(writer, r#"  "format": "smart-tree-quantum-stream-v1","#)?;
        writeln!(writer, r#"  "root_path": "{}","#, root_path.display())?;
        writeln!(writer, r#"  "streaming": true,"#)?;
        writeln!(writer, r#"  "chunks": ["#)?;
        Ok(())
    }
    
    fn format_node(
        &self,
        writer: &mut dyn Write,
        node: &FileNode,
        _root_path: &Path,
    ) -> Result<()> {
        // For streaming, we could send chunks of quantum data
        // For now, we'll just indicate streaming is not fully implemented
        writeln!(writer, r#"    {{"node": "{}"}}, "#, node.path.display())?;
        Ok(())
    }
    
    fn end_stream(
        &self,
        writer: &mut dyn Write,
        stats: &TreeStats,
        _root_path: &Path,
    ) -> Result<()> {
        writeln!(writer, r#"  ],"#)?;
        writeln!(writer, r#"  "statistics": {{"#)?;
        writeln!(writer, r#"    "total_files": {},"#, stats.total_files)?;
        writeln!(writer, r#"    "total_dirs": {},"#, stats.total_dirs)?;
        writeln!(writer, r#"    "total_size": {}"#, stats.total_size)?;
        writeln!(writer, r#"  }}"#)?;
        writeln!(writer, "}}")?;
        Ok(())
    }
}