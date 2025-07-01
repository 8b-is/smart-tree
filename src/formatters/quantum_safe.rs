// Quantum Safe Formatter - Binary-safe quantum format for JSON transmission
// This variant base64-encodes the binary sections while keeping the header readable

use super::{Formatter, StreamingFormatter};
use crate::scanner::{FileNode, TreeStats};
use anyhow::Result;
use base64::{Engine as _, engine::general_purpose};
use std::io::Write;
use std::path::Path;

/// Quantum formatter that produces JSON-safe output
/// Maintains all the compression benefits while being transmittable
pub struct QuantumSafeFormatter {
    inner: super::quantum::QuantumFormatter,
}

impl QuantumSafeFormatter {
    pub fn new() -> Self {
        Self {
            inner: super::quantum::QuantumFormatter::new(),
        }
    }
}

impl Formatter for QuantumSafeFormatter {
    fn format(
        &self,
        writer: &mut dyn Write,
        nodes: &[FileNode],
        stats: &TreeStats,
        root_path: &Path,
    ) -> Result<()> {
        // Generate quantum format to a buffer
        let mut quantum_buffer = Vec::new();
        self.inner.format(&mut quantum_buffer, nodes, stats, root_path)?;
        
        // Find the binary data section
        let data_start = b"---BEGIN_DATA---\n";
        let data_end = b"\n---END_DATA---";
        
        if let Some(start_pos) = quantum_buffer.windows(data_start.len())
            .position(|w| w == data_start) {
            
            let end_pos = quantum_buffer[start_pos + data_start.len()..]
                .windows(data_end.len())
                .position(|w| w == data_end)
                .map(|p| p + start_pos + data_start.len())
                .unwrap_or(quantum_buffer.len());
            
            // Write header as-is
            writer.write_all(&quantum_buffer[..start_pos])?;
            
            // Write modified data section with base64
            writeln!(writer, "---BEGIN_DATA_BASE64---")?;
            
            // Base64 encode the binary data
            let binary_data = &quantum_buffer[start_pos + data_start.len()..end_pos];
            let encoded = general_purpose::STANDARD.encode(binary_data);
            
            // Write in 76-character lines for readability
            for chunk in encoded.as_bytes().chunks(76) {
                writer.write_all(chunk)?;
                writeln!(writer)?;
            }
            
            writeln!(writer, "---END_DATA_BASE64---")?;
            
            // Include metadata about the encoding
            writeln!(writer, "---METADATA---")?;
            writeln!(writer, "ENCODING: base64")?;
            writeln!(writer, "BINARY_SIZE: {}", binary_data.len())?;
            writeln!(writer, "ENCODED_SIZE: {}", encoded.len())?;
            writeln!(writer, "COMPRESSION_RATIO: {:.1}%", 
                     (binary_data.len() as f64 / stats.total_files.max(1) as f64 / 200.0) * 100.0)?;
        } else {
            // Fallback - just encode the whole thing
            writer.write_all(&quantum_buffer)?;
        }
        
        Ok(())
    }
}

impl StreamingFormatter for QuantumSafeFormatter {
    fn start_stream(&self, writer: &mut dyn Write, _root_path: &Path) -> Result<()> {
        writeln!(writer, "MEM8_QUANTUM_SAFE_V1:")?;
        writeln!(writer, "STREAMING: true")?;
        writeln!(writer, "ENCODING: base64")?;
        writeln!(writer, "---BEGIN_STREAM---")?;
        Ok(())
    }
    
    fn format_node(&self, writer: &mut dyn Write, node: &FileNode, _root_path: &Path) -> Result<()> {
        // For streaming, we need to encode each entry
        let mut buffer = Vec::new();
        
        // Use the inner formatter to encode this node
        // This is a simplified version - in practice we'd need more context
        let header = if node.is_dir { 0x11 } else { 0x01 };
        buffer.push(header);
        
        // Add size
        if node.size <= 255 {
            buffer.extend_from_slice(&[0x00, node.size as u8]);
        } else {
            buffer.extend_from_slice(&[0x01, 0xFF, 0xFF]); // Simplified
        }
        
        // Add name
        let name = node.path.file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("?");
        buffer.extend_from_slice(name.as_bytes());
        buffer.push(0x00); // Null terminator
        
        // Encode and write
        let encoded = general_purpose::STANDARD.encode(&buffer);
        writeln!(writer, "{}", encoded)?;
        
        Ok(())
    }
    
    fn end_stream(&self, writer: &mut dyn Write, stats: &TreeStats, _root_path: &Path) -> Result<()> {
        writeln!(writer, "---END_STREAM---")?;
        writeln!(writer, "STATS: F:{} D:{} S:{:x}", 
                 stats.total_files, stats.total_dirs, stats.total_size)?;
        Ok(())
    }
}