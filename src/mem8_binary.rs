// MEM8 Binary Format - "The REAL wave-based memory!" 🌊
// Proper binary .m8 format with wave interference and temporal encoding
// "No more JSON pretenders!" - Hue

use anyhow::{Context, Result};
use std::collections::HashMap;
use std::fs::{File, OpenOptions};
use std::io::{Read, Seek, SeekFrom, Write};
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

/// Magic bytes for .m8 files
const M8_MAGIC: &[u8; 4] = b"MEM8";

/// Version of the .m8 format
const M8_VERSION: u8 = 1;

/// Block size (4KB for efficiency)
const BLOCK_SIZE: usize = 4096;

/// Header for .m8 files
#[repr(C, packed)]
pub struct M8Header {
    magic: [u8; 4],
    version: u8,
    flags: u8,
    block_count: u32,
    identity_freq: f64,
    temporal_phase: f64,
    crc32: u32,
}

impl M8Header {
    fn to_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::with_capacity(std::mem::size_of::<Self>());
        bytes.extend_from_slice(&self.magic);
        bytes.push(self.version);
        bytes.push(self.flags);
        bytes.extend_from_slice(&self.block_count.to_le_bytes());
        bytes.extend_from_slice(&self.identity_freq.to_le_bytes());
        bytes.extend_from_slice(&self.temporal_phase.to_le_bytes());
        bytes.extend_from_slice(&self.crc32.to_le_bytes());
        bytes
    }

    fn from_bytes(bytes: &[u8]) -> Result<Self> {
        if bytes.len() < std::mem::size_of::<Self>() {
            anyhow::bail!("Invalid header size");
        }

        let mut cursor = 0;
        let mut magic = [0u8; 4];
        magic.copy_from_slice(&bytes[cursor..cursor + 4]);
        cursor += 4;

        let version = bytes[cursor];
        cursor += 1;

        let flags = bytes[cursor];
        cursor += 1;

        let block_count = u32::from_le_bytes(bytes[cursor..cursor + 4].try_into()?);
        cursor += 4;

        let identity_freq = f64::from_le_bytes(bytes[cursor..cursor + 8].try_into()?);
        cursor += 8;

        let temporal_phase = f64::from_le_bytes(bytes[cursor..cursor + 8].try_into()?);
        cursor += 8;

        let crc32 = u32::from_le_bytes(bytes[cursor..cursor + 4].try_into()?);

        Ok(Self {
            magic,
            version,
            flags,
            block_count,
            identity_freq,
            temporal_phase,
            crc32,
        })
    }
}

/// Memory block in .m8 format
#[repr(C)]
#[derive(Debug, Clone)]
pub struct M8Block {
    /// Block index (even = consciousness, odd = context)
    pub index: u64,

    /// Wave signature (16 bytes)
    pub wave_signature: [u8; 16],

    /// Temporal timestamp (microseconds since epoch)
    pub timestamp: u64,

    /// Importance score (0.0 to 1.0 encoded as u16)
    pub importance: u16,

    /// Token ID (dynamic tokenization)
    pub token_id: u16,

    /// Previous block hash (for chaining)
    pub prev_hash: [u8; 32],

    /// Content length
    pub content_len: u32,

    /// Content (variable length, padded to BLOCK_SIZE)
    pub content: Vec<u8>,
}

/// Token mapping for dynamic tokenization
pub struct TokenMap {
    /// String to token ID
    str_to_token: HashMap<String, u16>,

    /// Token ID to string
    token_to_str: HashMap<u16, String>,

    /// Next available token ID
    next_id: u16,

    /// Reserved tokens (0x80 = node_modules, etc)
    reserved: HashMap<u16, String>,
}

impl Default for TokenMap {
    fn default() -> Self {
        Self::new()
    }
}

impl TokenMap {
    pub fn new() -> Self {
        let mut reserved = HashMap::new();
        reserved.insert(0x80, "node_modules".to_string());
        reserved.insert(0x81, ".git".to_string());
        reserved.insert(0x82, "target".to_string());
        reserved.insert(0x83, "dist".to_string());
        reserved.insert(0x84, "build".to_string());
        reserved.insert(0x90, ".rs".to_string());
        reserved.insert(0x91, ".py".to_string());
        reserved.insert(0x92, ".js".to_string());
        reserved.insert(0x93, ".ts".to_string());
        reserved.insert(0xFFFE, "Claude".to_string());
        reserved.insert(0xFFFF, "Hue".to_string());

        let mut str_to_token = HashMap::new();
        let mut token_to_str = HashMap::new();

        for (&id, value) in &reserved {
            str_to_token.insert(value.clone(), id);
            token_to_str.insert(id, value.clone());
        }

        Self {
            str_to_token,
            token_to_str,
            next_id: 0x100, // Start after reserved range
            reserved,
        }
    }

    /// Get or create token for string
    pub fn get_token(&mut self, s: &str) -> u16 {
        if let Some(&token) = self.str_to_token.get(s) {
            return token;
        }

        let token = self.next_id;
        self.next_id += 1;

        self.str_to_token.insert(s.to_string(), token);
        self.token_to_str.insert(token, s.to_string());

        token
    }

    /// Decode token to string
    pub fn decode_token(&self, token: u16) -> Option<&str> {
        self.token_to_str.get(&token).map(|s| s.as_str())
    }
}

/// MEM8 Binary File - append-only wave-based memory
pub struct M8BinaryFile {
    path: PathBuf,
    file: File,
    header: M8Header,
    tokens: TokenMap,

    /// Current position for backwards reading
    read_position: u64,

    /// Cache of important blocks for quick access
    importance_index: Vec<(u64, f32)>, // (block_offset, importance)
}

impl M8BinaryFile {
    /// Create new .m8 file
    pub fn create(path: impl AsRef<Path>) -> Result<Self> {
        let path = path.as_ref().to_path_buf();

        let mut file = OpenOptions::new()
            .create(true)
            .write(true)
            .read(true)
            .truncate(true)
            .open(&path)
            .context("Failed to create .m8 file")?;

        let header = M8Header {
            magic: *M8_MAGIC,
            version: M8_VERSION,
            flags: 0,
            block_count: 0,
            identity_freq: 440.0, // A440 Hz by default
            temporal_phase: 0.0,
            crc32: 0,
        };

        // Write header
        file.write_all(&header.to_bytes())?;

        Ok(Self {
            path,
            file,
            header,
            tokens: TokenMap::new(),
            read_position: std::mem::size_of::<M8Header>() as u64,
            importance_index: Vec::new(),
        })
    }

    /// Open existing .m8 file
    pub fn open(path: impl AsRef<Path>) -> Result<Self> {
        let path = path.as_ref().to_path_buf();

        let mut file = OpenOptions::new()
            .read(true)
            .write(true)
            .open(&path)
            .context("Failed to open .m8 file")?;

        // Read header
        let mut header_bytes = vec![0u8; std::mem::size_of::<M8Header>()];
        file.read_exact(&mut header_bytes)?;

        let header = M8Header::from_bytes(&header_bytes)?;

        // Verify magic
        if header.magic != *M8_MAGIC {
            anyhow::bail!("Invalid .m8 file (bad magic)");
        }

        // Seek to end for backwards reading
        let file_size = file.seek(SeekFrom::End(0))?;

        let mut m8 = Self {
            path,
            file,
            header,
            tokens: TokenMap::new(),
            read_position: file_size,
            importance_index: Vec::new(),
        };

        // Build importance index
        m8.build_importance_index()?;

        Ok(m8)
    }

    /// Append memory block (never modifies existing blocks)
    pub fn append_block(&mut self, content: &[u8], importance: f32) -> Result<()> {
        let timestamp = SystemTime::now().duration_since(UNIX_EPOCH)?.as_micros() as u64;

        let wave_sig = self.generate_wave_signature(content);

        let block = M8Block {
            index: self.header.block_count as u64,
            wave_signature: wave_sig,
            timestamp,
            importance: (importance * 65535.0) as u16,
            token_id: 0,        // TODO: tokenize content
            prev_hash: [0; 32], // TODO: compute hash
            content_len: content.len() as u32,
            content: content.to_vec(),
        };

        // Seek to end
        self.file.seek(SeekFrom::End(0))?;

        // Write block
        self.write_block(&block)?;

        // Update header
        self.header.block_count += 1;
        self.update_header()?;

        // Update importance index
        let offset = self.file.stream_position()?;
        self.importance_index.push((offset, importance));

        Ok(())
    }

    /// Read backwards from end (most recent first)
    pub fn read_backwards(&mut self) -> Result<Option<M8Block>> {
        if self.read_position <= std::mem::size_of::<M8Header>() as u64 {
            return Ok(None);
        }

        // Seek to previous block
        self.read_position -= BLOCK_SIZE as u64;
        self.file.seek(SeekFrom::Start(self.read_position))?;

        // Read block
        self.read_block()
    }

    /// Read blocks by importance (user keywords boost importance)
    pub fn read_by_importance(&mut self, keywords: &[String]) -> Result<Vec<M8Block>> {
        let mut blocks = Vec::new();

        // Sort by importance
        self.importance_index
            .sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());

        // Extract offsets to avoid holding any borrow of self while reading blocks
        let offsets: Vec<u64> = self.importance_index.iter().map(|(off, _)| *off).collect();

        for offset in offsets {
            self.file.seek(SeekFrom::Start(offset))?;

            if let Some(block) = self.read_block()? {
                // Check if block contains keywords
                let content_str = String::from_utf8_lossy(&block.content);
                let has_keyword = keywords.iter().any(|kw| content_str.contains(kw));

                if has_keyword || blocks.len() < 10 {
                    blocks.push(block);
                }

                if blocks.len() >= 20 {
                    break;
                }
            }
        }

        Ok(blocks)
    }

    /// Generate wave signature from content
    fn generate_wave_signature(&self, content: &[u8]) -> [u8; 16] {
        use sha2::{Digest, Sha256};

        let mut hasher = Sha256::new();
        hasher.update(content);
        hasher.update(self.header.identity_freq.to_le_bytes());

        let hash = hasher.finalize();
        let mut signature = [0u8; 16];
        signature.copy_from_slice(&hash[..16]);

        signature
    }

    /// Write block to file
    fn write_block(&mut self, block: &M8Block) -> Result<()> {
        // Create fixed-size block buffer
        let mut buffer = vec![0u8; BLOCK_SIZE];
        let mut cursor = 0;

        // Write block fields
        buffer[cursor..cursor + 8].copy_from_slice(&block.index.to_le_bytes());
        cursor += 8;

        buffer[cursor..cursor + 16].copy_from_slice(&block.wave_signature);
        cursor += 16;

        buffer[cursor..cursor + 8].copy_from_slice(&block.timestamp.to_le_bytes());
        cursor += 8;

        buffer[cursor..cursor + 2].copy_from_slice(&block.importance.to_le_bytes());
        cursor += 2;

        buffer[cursor..cursor + 2].copy_from_slice(&block.token_id.to_le_bytes());
        cursor += 2;

        buffer[cursor..cursor + 32].copy_from_slice(&block.prev_hash);
        cursor += 32;

        buffer[cursor..cursor + 4].copy_from_slice(&block.content_len.to_le_bytes());
        cursor += 4;

        // Copy content (up to remaining space)
        let content_space = BLOCK_SIZE - cursor;
        let content_to_copy = block.content.len().min(content_space);
        buffer[cursor..cursor + content_to_copy].copy_from_slice(&block.content[..content_to_copy]);

        self.file.write_all(&buffer)?;

        Ok(())
    }

    /// Read block from file
    fn read_block(&mut self) -> Result<Option<M8Block>> {
        let mut buffer = vec![0u8; BLOCK_SIZE];

        match self.file.read_exact(&mut buffer) {
            Ok(_) => {}
            Err(e) if e.kind() == std::io::ErrorKind::UnexpectedEof => return Ok(None),
            Err(e) => return Err(e.into()),
        }

        let mut cursor = 0;

        let index = u64::from_le_bytes(buffer[cursor..cursor + 8].try_into()?);
        cursor += 8;

        let mut wave_signature = [0u8; 16];
        wave_signature.copy_from_slice(&buffer[cursor..cursor + 16]);
        cursor += 16;

        let timestamp = u64::from_le_bytes(buffer[cursor..cursor + 8].try_into()?);
        cursor += 8;

        let importance = u16::from_le_bytes(buffer[cursor..cursor + 2].try_into()?);
        cursor += 2;

        let token_id = u16::from_le_bytes(buffer[cursor..cursor + 2].try_into()?);
        cursor += 2;

        let mut prev_hash = [0u8; 32];
        prev_hash.copy_from_slice(&buffer[cursor..cursor + 32]);
        cursor += 32;

        let content_len = u32::from_le_bytes(buffer[cursor..cursor + 4].try_into()?);
        cursor += 4;

        let content = buffer[cursor..cursor + content_len as usize].to_vec();

        Ok(Some(M8Block {
            index,
            wave_signature,
            timestamp,
            importance,
            token_id,
            prev_hash,
            content_len,
            content,
        }))
    }

    /// Update header in file
    fn update_header(&mut self) -> Result<()> {
        self.file.seek(SeekFrom::Start(0))?;
        self.file.write_all(&self.header.to_bytes())?;
        self.file.flush()?;
        Ok(())
    }

    /// Build importance index from file
    fn build_importance_index(&mut self) -> Result<()> {
        self.file
            .seek(SeekFrom::Start(std::mem::size_of::<M8Header>() as u64))?;

        loop {
            let offset = self.file.stream_position()?;

            match self.read_block()? {
                Some(block) => {
                    let importance = block.importance as f32 / 65535.0;
                    self.importance_index.push((offset, importance));
                }
                None => break,
            }
        }

        Ok(())
    }
}

/// Convert existing JSON .m8 files to proper binary format
pub fn convert_json_to_binary(json_path: &Path, binary_path: &Path) -> Result<()> {
    use std::fs;

    // Read and decompress if needed
    let data = fs::read(json_path)?;

    let json_str = if data.starts_with(b"\x78\x9c") || data.starts_with(b"\x78\xda") {
        // zlib compressed
        use flate2::read::ZlibDecoder;
        let mut decoder = ZlibDecoder::new(&data[..]);
        let mut decompressed = String::new();
        decoder.read_to_string(&mut decompressed)?;
        decompressed
    } else {
        String::from_utf8(data)?
    };

    // Parse JSON
    let contexts: serde_json::Value = serde_json::from_str(&json_str)?;

    // Create binary .m8 file
    let mut m8_file = M8BinaryFile::create(binary_path)?;

    // Convert contexts to blocks
    if let Some(contexts_array) = contexts.get("contexts").and_then(|c| c.as_array()) {
        for context in contexts_array {
            let content = serde_json::to_vec(context)?;
            let importance = context.get("score").and_then(|s| s.as_f64()).unwrap_or(0.5) as f32;

            m8_file.append_block(&content, importance)?;
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_m8_binary_format() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("test.m8");

        // Create file
        let mut m8 = M8BinaryFile::create(&path).unwrap();

        // Append some blocks
        m8.append_block(b"First memory", 0.8).unwrap();
        m8.append_block(b"Second memory", 0.5).unwrap();
        m8.append_block(b"Important memory", 1.0).unwrap();

        // Reopen and read backwards
        let mut m8 = M8BinaryFile::open(&path).unwrap();

        // Should read "Important memory" first (most recent)
        let block = m8.read_backwards().unwrap().unwrap();
        assert_eq!(&block.content, b"Important memory");

        // Read by importance
        let important_blocks = m8.read_by_importance(&["Important".to_string()]).unwrap();
        assert!(!important_blocks.is_empty());
    }

    #[test]
    fn test_tokenization() {
        let mut tokens = TokenMap::new();

        // Reserved tokens
        assert_eq!(tokens.get_token("node_modules"), 0x80);
        assert_eq!(tokens.get_token(".rs"), 0x90);
        assert_eq!(tokens.get_token("Claude"), 0xFFFE);

        // Dynamic tokens
        let token1 = tokens.get_token("custom_string");
        let token2 = tokens.get_token("another_string");
        assert!(token1 >= 0x100);
        assert!(token2 > token1);

        // Decode
        assert_eq!(tokens.decode_token(0x80), Some("node_modules"));
        assert_eq!(tokens.decode_token(token1), Some("custom_string"));
    }
}
