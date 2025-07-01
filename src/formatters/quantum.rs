use super::{Formatter, StreamingFormatter};
use crate::scanner::{FileNode, TreeStats};
use anyhow::Result;
use std::collections::HashMap;
use std::io::Write;
use std::path::Path;

/// MEM|8 Quantum Format - The Ultimate Compression
/// 
/// Header byte (8 bits):
/// 7 6 5 4 3 2 1 0
/// | | | | | | | └─ Size present (always 1 for now)
/// | | | | | | └─── Permissions differ from parent
/// | | | | | └──── Time differs from parent  
/// | | | | └───── Owner/Group differ from parent
/// | | | └────── Is directory
/// | | └─────── Is symlink
/// | └──────── Has extended attributes
/// └───────── Reserved for summary
pub struct QuantumFormatter {
    // Context for delta encoding
    parent_perms: u16,
    parent_uid: u32,
    parent_gid: u32,
    parent_time: u64,
    
    // Token dictionary for common patterns
    tokens: HashMap<String, u8>,
    next_token: u8,
}

// Bit positions in header
const SIZE_BIT: u8 = 0b00000001;
const PERMS_BIT: u8 = 0b00000010;
const TIME_BIT: u8 = 0b00000100;
const OWNER_BIT: u8 = 0b00001000;
const DIR_BIT: u8 = 0b00010000;
const LINK_BIT: u8 = 0b00100000;
const XATTR_BIT: u8 = 0b01000000;
const SUMMARY_BIT: u8 = 0b10000000;

// ASCII control codes for tree traversal
const TRAVERSE_SAME: char = '\x0B';     // Vertical Tab
const TRAVERSE_DEEPER: char = '\x0E';   // Shift Out
const TRAVERSE_BACK: char = '\x0F';     // Shift In

impl QuantumFormatter {
    pub fn new() -> Self {
        let mut tokens = HashMap::new();
        
        // Pre-populate common tokens
        tokens.insert("node_modules".to_string(), 0x80);
        tokens.insert(".git".to_string(), 0x81);
        tokens.insert("src".to_string(), 0x82);
        tokens.insert("target".to_string(), 0x83);
        tokens.insert("dist".to_string(), 0x84);
        tokens.insert(".js".to_string(), 0x90);
        tokens.insert(".rs".to_string(), 0x91);
        tokens.insert(".json".to_string(), 0x92);
        tokens.insert(".md".to_string(), 0x93);
        tokens.insert("index".to_string(), 0x94);
        tokens.insert("README".to_string(), 0x95);
        
        Self {
            parent_perms: 0o755,
            parent_uid: 1000,
            parent_gid: 1000,
            parent_time: 0,
            tokens,
            next_token: 0xA0,
        }
    }
    
    /// Encode size using variable-length encoding
    fn encode_size(size: u64) -> Vec<u8> {
        match size {
            0..=255 => vec![0x00, size as u8],
            256..=65535 => {
                let bytes = (size as u16).to_le_bytes();
                vec![0x01, bytes[0], bytes[1]]
            }
            65536..=4294967295 => {
                let bytes = (size as u32).to_le_bytes();
                vec![0x02, bytes[0], bytes[1], bytes[2], bytes[3]]
            }
            _ => {
                let bytes = size.to_le_bytes();
                let mut result = vec![0x03];
                result.extend_from_slice(&bytes);
                result
            }
        }
    }
    
    /// Encode permissions as delta from parent
    fn encode_perms_delta(&self, perms: u32) -> Vec<u8> {
        let perms16 = (perms & 0o777) as u16;
        if perms16 == self.parent_perms {
            vec![]
        } else {
            // Just store the different bits
            let delta = perms16 ^ self.parent_perms;
            vec![(delta >> 8) as u8, delta as u8]
        }
    }
    
    /// Tokenize filename components
    fn tokenize_name(&mut self, name: &str) -> Vec<u8> {
        let mut result = Vec::new();
        
        // Check for exact token match
        if let Some(&token) = self.tokens.get(name) {
            result.push(token);
            return result;
        }
        
        // Check for extension tokens
        if let Some(dot_pos) = name.rfind('.') {
            let ext = &name[dot_pos..];
            if let Some(&token) = self.tokens.get(ext) {
                result.extend_from_slice(name[..dot_pos].as_bytes());
                result.push(token);
                return result;
            }
        }
        
        // Check for prefix tokens
        for (pattern, &token) in &self.tokens {
            if name.starts_with(pattern) && pattern.len() > 3 {
                result.push(token);
                result.extend_from_slice(name[pattern.len()..].as_bytes());
                return result;
            }
        }
        
        // No token found, use raw name
        result.extend_from_slice(name.as_bytes());
        result
    }
    
    fn encode_entry(&mut self, node: &FileNode) -> Vec<u8> {
        let mut header = 0u8;
        let mut data = Vec::new();
        
        // Always include size (for now)
        header |= SIZE_BIT;
        let size_bytes = Self::encode_size(node.size);
        data.extend(size_bytes);
        
        // Check what differs from parent context
        if (node.permissions & 0o777) as u16 != self.parent_perms {
            header |= PERMS_BIT;
            data.extend(self.encode_perms_delta(node.permissions));
        }
        
        // For directories, update context
        if node.is_dir {
            header |= DIR_BIT;
            // Update parent context for children
            self.parent_perms = (node.permissions & 0o777) as u16;
            self.parent_uid = node.uid;
            self.parent_gid = node.gid;
            self.parent_time = node.modified
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs();
        }
        
        // Add header
        let mut result = vec![header];
        result.extend(data);
        
        // Add tokenized name with null terminator
        let name = node.path.file_name()
            .unwrap_or(node.path.as_os_str())
            .to_string_lossy();
        let tokenized = self.tokenize_name(&name);
        result.extend(tokenized);
        result.push(0); // Add null terminator for name
        
        result
    }
    
}

impl Formatter for QuantumFormatter {
    fn format(
        &self,
        writer: &mut dyn Write,
        nodes: &[FileNode],
        _stats: &TreeStats,
        _root_path: &Path,
    ) -> Result<()> {
        
        let mut formatter = QuantumFormatter::new();
        
        // Write header
        writeln!(writer, "MEM8_QUANTUM_V1:")?;
        writeln!(writer, "KEY:HSSSSS...")?; // Header + variable size
        writeln!(writer, "TOKENS:80=node_modules,81=.git,82=src,90=.js,91=.rs")?;
        writeln!(writer, "---BEGIN_DATA---")?; // Clear marker for binary data start
        
        // Process nodes with depth tracking
        let mut current_depth = 0;
        
        for (i, node) in nodes.iter().enumerate() {
            // Handle depth changes
            if current_depth > node.depth {
                // Going back up one or more levels
                for _ in 0..(current_depth - node.depth) {
                    write!(writer, "{}", TRAVERSE_BACK)?;
                }
                current_depth = node.depth;
            }
            
            // Encode entry
            let encoded = formatter.encode_entry(node);
            writer.write_all(&encoded)?;
            
            // Add traversal code
            let is_last = i + 1 >= nodes.len() || 
                         (i + 1 < nodes.len() && nodes[i + 1].depth < node.depth);
            
            if node.is_dir && i + 1 < nodes.len() && nodes[i + 1].depth > node.depth {
                write!(writer, "{}", TRAVERSE_DEEPER)?;
                current_depth = node.depth + 1;
            } else if is_last && node.depth > 0 {
                write!(writer, "{}", TRAVERSE_BACK)?;
                current_depth = node.depth - 1;
            } else {
                write!(writer, "{}", TRAVERSE_SAME)?;
            }
        }
        
        // Close any remaining directories
        while current_depth > 0 {
            write!(writer, "{}", TRAVERSE_BACK)?;
            current_depth -= 1;
        }
        
        writeln!(writer)?; // Final newline
        writeln!(writer, "---END_DATA---")?; // Clear marker for binary data end
        
        Ok(())
    }
}

impl StreamingFormatter for QuantumFormatter {
    fn start_stream(&self, writer: &mut dyn Write, _root_path: &Path) -> Result<()> {
        writeln!(writer, "MEM8_QUANTUM_V1_STREAM:")?;
        writeln!(writer, "KEY:HSSSSS...")?;
        writeln!(writer, "BASE_CONTEXT:perms=755,uid=1000,gid=1000")?;
        Ok(())
    }
    
    fn format_node(
        &self,
        writer: &mut dyn Write,
        node: &FileNode,
        _root_path: &Path,
    ) -> Result<()> {
        let mut formatter = QuantumFormatter::new();
        let encoded = formatter.encode_entry(node);
        writer.write_all(&encoded)?;
        write!(writer, "{}", TRAVERSE_SAME)?;
        Ok(())
    }
    
    fn end_stream(
        &self,
        writer: &mut dyn Write,
        stats: &TreeStats,
        _root_path: &Path,
    ) -> Result<()> {
        // Compact summary
        writeln!(writer, "\nQUANTUM_STATS:")?;
        writeln!(writer, "F:{:x} D:{:x} S:{:x}", 
                 stats.total_files, 
                 stats.total_dirs, 
                 stats.total_size)?;
        Ok(())
    }
}