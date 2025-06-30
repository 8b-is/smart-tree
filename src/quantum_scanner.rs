// Quantum Scanner - The native tree walker that speaks in quantum format
// This is where the magic happens - no intermediate representation, just pure quantum output

#![allow(dead_code)] // Many constants and fields are reserved for future use

use std::fs;
use std::io::Write;
use std::path::Path;
use std::collections::HashMap;
use std::time::SystemTime;
use anyhow::Result;

// Token ranges as suggested
const TOKEN_RESERVED_START: u16 = 0x0000;
const TOKEN_RESERVED_END: u16 = 0x00FF;
const TOKEN_USER_START: u16 = 0x0100;

// Pre-defined tokens for common filesystem terms
const TOKEN_DIR: u16 = 0x0001;
const TOKEN_FILE: u16 = 0x0002;
const TOKEN_LINK: u16 = 0x0003;
const TOKEN_PERM_755: u16 = 0x0010;
const TOKEN_PERM_644: u16 = 0x0011;
const TOKEN_PERM_777: u16 = 0x0012;
const TOKEN_PERM_600: u16 = 0x0013;

// Common extensions (0x20-0x7F)
const TOKEN_EXT_JS: u16 = 0x0020;
const TOKEN_EXT_RS: u16 = 0x0021;
const TOKEN_EXT_PY: u16 = 0x0022;
const TOKEN_EXT_GO: u16 = 0x0023;
const TOKEN_EXT_MD: u16 = 0x0024;
const TOKEN_EXT_JSON: u16 = 0x0025;
const TOKEN_EXT_YAML: u16 = 0x0026;
const TOKEN_EXT_TXT: u16 = 0x0027;

// Common directory names (0x80-0xFF)
const TOKEN_NODE_MODULES: u16 = 0x0080;
const TOKEN_GIT: u16 = 0x0081;
const TOKEN_SRC: u16 = 0x0082;
const TOKEN_TARGET: u16 = 0x0083;
const TOKEN_BUILD: u16 = 0x0084;
const TOKEN_DIST: u16 = 0x0085;
const TOKEN_DOCS: u16 = 0x0086;
const TOKEN_TESTS: u16 = 0x0087;

// Size tokens for common ranges
const TOKEN_SIZE_ZERO: u16 = 0x00A0;
const TOKEN_SIZE_TINY: u16 = 0x00A1;  // 1-1KB
const TOKEN_SIZE_SMALL: u16 = 0x00A2; // 1KB-100KB
const TOKEN_SIZE_MEDIUM: u16 = 0x00A3; // 100KB-10MB
const TOKEN_SIZE_LARGE: u16 = 0x00A4; // 10MB+

// ASCII control codes for tree traversal
const TRAVERSE_SAME: u8 = 0x0B;     // Vertical Tab
const TRAVERSE_DEEPER: u8 = 0x0E;   // Shift Out
const TRAVERSE_BACK: u8 = 0x0F;     // Shift In
const TRAVERSE_SUMMARY: u8 = 0x0C;  // Form Feed

// Header bit flags
const HDR_HAS_SIZE: u8 = 0b00000001;
const HDR_HAS_PERMS: u8 = 0b00000010;
const HDR_HAS_TIME: u8 = 0b00000100;
const HDR_HAS_OWNER: u8 = 0b00001000;
const HDR_IS_DIR: u8 = 0b00010000;
const HDR_IS_LINK: u8 = 0b00100000;
const HDR_HAS_XATTR: u8 = 0b01000000;
const HDR_TOKENIZED: u8 = 0b10000000;

pub struct QuantumScanner<W: Write> {
    writer: W,
    token_map: HashMap<String, u16>,
    #[allow(dead_code)]
    next_dynamic_token: u16,
    
    // Context for delta encoding
    parent_perms: u32,
    #[allow(dead_code)]
    parent_uid: u32,
    #[allow(dead_code)]
    parent_gid: u32,
    #[allow(dead_code)]
    parent_time: SystemTime,
    
    // Stats tracking
    total_files: u64,
    total_dirs: u64,
    total_size: u64,
}

impl<W: Write> QuantumScanner<W> {
    // Cross-platform permission handling
    #[cfg(unix)]
    fn get_permissions(metadata: &fs::Metadata) -> u16 {
        use std::os::unix::fs::PermissionsExt;
        (metadata.permissions().mode() & 0o777) as u16
    }
    
    #[cfg(not(unix))]
    fn get_permissions(_metadata: &fs::Metadata) -> u16 {
        0o755 // Default permissions for non-Unix
    }
    
    pub fn new(writer: W) -> Self {
        let mut token_map = HashMap::new();
        
        // Initialize with predefined tokens
        token_map.insert("node_modules".to_string(), TOKEN_NODE_MODULES);
        token_map.insert(".git".to_string(), TOKEN_GIT);
        token_map.insert("src".to_string(), TOKEN_SRC);
        token_map.insert("target".to_string(), TOKEN_TARGET);
        token_map.insert("build".to_string(), TOKEN_BUILD);
        token_map.insert("dist".to_string(), TOKEN_DIST);
        token_map.insert("docs".to_string(), TOKEN_DOCS);
        token_map.insert("tests".to_string(), TOKEN_TESTS);
        
        // Extension tokens
        token_map.insert(".js".to_string(), TOKEN_EXT_JS);
        token_map.insert(".rs".to_string(), TOKEN_EXT_RS);
        token_map.insert(".py".to_string(), TOKEN_EXT_PY);
        token_map.insert(".go".to_string(), TOKEN_EXT_GO);
        token_map.insert(".md".to_string(), TOKEN_EXT_MD);
        token_map.insert(".json".to_string(), TOKEN_EXT_JSON);
        token_map.insert(".yaml".to_string(), TOKEN_EXT_YAML);
        token_map.insert(".txt".to_string(), TOKEN_EXT_TXT);
        
        Self {
            writer,
            token_map,
            next_dynamic_token: TOKEN_USER_START,
            parent_perms: 0o755,
            parent_uid: 1000,
            parent_gid: 1000,
            parent_time: SystemTime::UNIX_EPOCH,
            total_files: 0,
            total_dirs: 0,
            total_size: 0,
        }
    }
    
    /// Write the quantum format header
    pub fn write_header(&mut self) -> Result<()> {
        writeln!(self.writer, "QUANTUM_NATIVE_V1:")?;
        writeln!(self.writer, "TOKENS:")?;
        
        // Write token map in sorted order
        let mut tokens: Vec<_> = self.token_map.iter().collect();
        tokens.sort_by_key(|(_, &token)| token);
        
        for (name, token) in tokens {
            writeln!(self.writer, "  {:04X}={}", token, name)?;
        }
        
        writeln!(self.writer, "DATA:")?;
        Ok(())
    }
    
    /// Scan a path and emit quantum format directly
    pub fn scan(&mut self, path: &Path) -> Result<()> {
        self.write_header()?;
        self.scan_recursive(path, 0)?;
        self.write_summary()?;
        Ok(())
    }
    
    fn scan_recursive(&mut self, path: &Path, depth: usize) -> Result<()> {
        let metadata = fs::metadata(path)?;
        
        // Emit quantum entry
        if metadata.is_dir() {
            self.emit_directory(path, &metadata, depth)?;
            
            // Update parent context
            let old_perms = self.parent_perms;
            self.parent_perms = Self::get_permissions(&metadata);
            
            // Scan children
            let mut entries: Vec<_> = fs::read_dir(path)?
                .filter_map(|e| e.ok())
                .collect();
            
            // Sort for consistent output
            entries.sort_by_key(|e| e.file_name());
            
            for (i, entry) in entries.iter().enumerate() {
                let child_path = entry.path();
                self.scan_recursive(&child_path, depth + 1)?;
                
                // Emit traversal code
                if i < entries.len() - 1 {
                    self.writer.write_all(&[TRAVERSE_SAME])?;
                }
            }
            
            // Restore parent context
            self.parent_perms = old_perms;
            
            // Emit back traversal if not at root
            if depth > 0 {
                self.writer.write_all(&[TRAVERSE_BACK])?;
            }
            
            self.total_dirs += 1;
        } else {
            self.emit_file(path, &metadata)?;
            self.total_files += 1;
            self.total_size += metadata.len();
        }
        
        Ok(())
    }
    
    fn emit_directory(&mut self, path: &Path, metadata: &fs::Metadata, depth: usize) -> Result<()> {
        let mut header = HDR_IS_DIR;
        let mut data = Vec::new();
        
        // Size (for directories, this is the entry size)
        header |= HDR_HAS_SIZE;
        data.extend(&self.encode_size(metadata.len()));
        
        // Permissions if different
        let perms = Self::get_permissions(metadata);
        if perms != self.parent_perms {
            header |= HDR_HAS_PERMS;
            let delta = perms ^ self.parent_perms;
            data.push((delta >> 8) as u8);
            data.push(delta as u8);
        }
        
        // Emit header and data
        self.writer.write_all(&[header])?;
        self.writer.write_all(&data)?;
        
        // Emit name (tokenized if possible)
        self.emit_name(path)?;
        
        // Emit traversal code
        if depth == 0 {
            // Root directory
            self.writer.write_all(&[TRAVERSE_DEEPER])?;
        }
        
        Ok(())
    }
    
    fn emit_file(&mut self, path: &Path, metadata: &fs::Metadata) -> Result<()> {
        let mut header = 0u8;
        let mut data = Vec::new();
        
        // Size
        header |= HDR_HAS_SIZE;
        data.extend(&self.encode_size(metadata.len()));
        
        // Permissions if different
        let perms = Self::get_permissions(metadata);
        if perms != self.parent_perms {
            header |= HDR_HAS_PERMS;
            let delta = perms ^ self.parent_perms;
            data.push((delta >> 8) as u8);
            data.push(delta as u8);
        }
        
        // Emit header and data
        self.writer.write_all(&[header])?;
        self.writer.write_all(&data)?;
        
        // Emit name
        self.emit_name(path)?;
        
        Ok(())
    }
    
    fn emit_name(&mut self, path: &Path) -> Result<()> {
        let name = path.file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("");
        
        // Check for exact token match
        if let Some(&token) = self.token_map.get(name) {
            self.writer.write_all(&token.to_le_bytes())?;
            return Ok(());
        }
        
        // Check for extension token
        if let Some(dot_pos) = name.rfind('.') {
            let ext = &name[dot_pos..];
            if let Some(&token) = self.token_map.get(ext) {
                // Write base name + extension token
                self.writer.write_all(name[..dot_pos].as_bytes())?;
                self.writer.write_all(&token.to_le_bytes())?;
                return Ok(());
            }
        }
        
        // No token found - consider adding dynamically for frequently seen patterns
        // For now, just write the raw name
        self.writer.write_all(name.as_bytes())?;
        Ok(())
    }
    
    fn encode_size(&self, size: u64) -> Vec<u8> {
        // Size-based tokenization
        match size {
            0 => vec![TOKEN_SIZE_ZERO as u8, (TOKEN_SIZE_ZERO >> 8) as u8],
            1..=1024 => vec![TOKEN_SIZE_TINY as u8, (TOKEN_SIZE_TINY >> 8) as u8, size as u8],
            1025..=102400 => {
                let kb = (size / 1024) as u16;
                vec![TOKEN_SIZE_SMALL as u8, (TOKEN_SIZE_SMALL >> 8) as u8, 
                     kb as u8, (kb >> 8) as u8]
            }
            _ => {
                // For larger sizes, use standard encoding
                match size {
                    0..=255 => vec![0x00, size as u8],
                    256..=65535 => {
                        let bytes = (size as u16).to_le_bytes();
                        vec![0x01, bytes[0], bytes[1]]
                    }
                    _ => {
                        let bytes = (size as u32).to_le_bytes();
                        vec![0x02, bytes[0], bytes[1], bytes[2], bytes[3]]
                    }
                }
            }
        }
    }
    
    fn write_summary(&mut self) -> Result<()> {
        writeln!(self.writer, "\nSUMMARY:")?;
        writeln!(self.writer, "FILES: {}", self.total_files)?;
        writeln!(self.writer, "DIRS: {}", self.total_dirs)?;
        writeln!(self.writer, "SIZE: {}", self.total_size)?;
        Ok(())
    }
}

#[cfg(not(target_os = "windows"))]
#[cfg(unix)]
use std::os::unix::fs::PermissionsExt;