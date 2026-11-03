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

/// Compile-time IEEE 802.3 CRC32 lookup table (`256 x u32` = `1,024` bytes).
pub const CRC32_LUT: [u32; 256] = build_crc32_lut();

const fn build_crc32_lut() -> [u32; 256] {
    let mut table = [0u32; 256];
    let mut i = 0usize;
    while i < 256 {
        let mut crc = i as u32;
        let mut bit = 0;
        while bit < 8 {
            let mask = (crc & 1).wrapping_neg();
            crc = (crc >> 1) ^ (0xEDB8_8320 & mask);
            bit += 1;
        }
        table[i] = crc;
        i += 1;
    }
    table
}

/// Branchless 1-lookup-per-byte IEEE 802.3 CRC32 computation.
#[inline]
pub fn crc32_branchless(bytes: &[u8]) -> u32 {
    let mut crc = 0xFFFF_FFFFu32;
    for &b in bytes {
        let idx = ((crc ^ (b as u32)) & 0xFF) as usize;
        crc = (crc >> 8) ^ CRC32_LUT[idx];
    }
    !crc
}

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
    #[inline]
    fn prefix_bytes(&self) -> [u8; 26] {
        let mut prefix = [0u8; 26];
        prefix[0..4].copy_from_slice(&self.magic);
        prefix[4] = self.version;
        prefix[5] = self.flags;
        prefix[6..10].copy_from_slice(&self.block_count.to_le_bytes());
        prefix[10..18].copy_from_slice(&self.identity_freq.to_le_bytes());
        prefix[18..26].copy_from_slice(&self.temporal_phase.to_le_bytes());
        prefix
    }

    #[inline]
    fn compute_crc32(&self) -> u32 {
        crc32_branchless(&self.prefix_bytes())
    }

    #[inline]
    fn to_bytes(&self) -> [u8; 30] {
        let mut bytes = [0u8; 30];
        let prefix = self.prefix_bytes();
        bytes[0..26].copy_from_slice(&prefix);
        let crc = crc32_branchless(&prefix);
        bytes[26..30].copy_from_slice(&crc.to_le_bytes());
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

    /// SHA-256 chain digest of the most recently written/read block (`Dharma-mudrā` continuity)
    last_block_hash: [u8; 32],
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

        let mut header = M8Header {
            magic: *M8_MAGIC,
            version: M8_VERSION,
            flags: 0,
            block_count: 0,
            identity_freq: 440.0, // A440 Hz by default
            temporal_phase: 0.0,
            crc32: 0,
        };
        header.crc32 = header.compute_crc32();

        // Write header
        file.write_all(&header.to_bytes())?;

        Ok(Self {
            path,
            file,
            header,
            tokens: TokenMap::new(),
            read_position: std::mem::size_of::<M8Header>() as u64,
            importance_index: Vec::new(),
            last_block_hash: [0u8; 32],
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

        // Read header (stack-allocated 30-byte buffer)
        let mut header_bytes = [0u8; 30];
        file.read_exact(&mut header_bytes)?;

        let header = M8Header::from_bytes(&header_bytes)?;

        // Verify magic
        if header.magic != *M8_MAGIC {
            anyhow::bail!("Invalid .m8 file (bad magic)");
        }
        // Verify CRC32 if non-zero (backwards-compatible with legacy 0 CRC headers)
        if header.crc32 != 0 && header.crc32 != header.compute_crc32() {
            anyhow::bail!("Invalid .m8 file (header CRC32 mismatch)");
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
            last_block_hash: [0u8; 32],
        };

        // Build importance index
        m8.build_importance_index()?;
        // Restore read_position to end of file so read_backwards starts at the latest block
        m8.read_position = file_size;

        Ok(m8)
    }

    /// Append memory block (never modifies existing blocks)
    pub fn append_block(&mut self, content: &[u8], importance: f32) -> Result<()> {
        use sha2::{Digest, Sha256};

        let timestamp = SystemTime::now().duration_since(UNIX_EPOCH)?.as_micros() as u64;
        let wave_sig = self.generate_wave_signature(content);

        // Extract primary token for dynamic tokenization
        let primary_word = std::str::from_utf8(content)
            .ok()
            .and_then(|s| s.split_whitespace().next())
            .unwrap_or("binary_capsule");
        let token_id = self.tokens.get_token(primary_word);
        let max_payload = BLOCK_SIZE - 72;
        let clamped_len = content.len().min(max_payload);

        let block = M8Block {
            index: self.header.block_count as u64,
            wave_signature: wave_sig,
            timestamp,
            importance: (importance.clamp(0.0, 1.0) * 65535.0) as u16,
            token_id,
            prev_hash: self.last_block_hash,
            content_len: clamped_len as u32,
            content: content[..clamped_len].to_vec(),
        };

        // Compute new chained block hash
        let mut hasher = Sha256::new();
        hasher.update(block.index.to_le_bytes());
        hasher.update(block.wave_signature);
        hasher.update(block.timestamp.to_le_bytes());
        hasher.update(block.prev_hash);
        hasher.update(&block.content);
        let digest = hasher.finalize();
        self.last_block_hash.copy_from_slice(&digest);

        // Seek to end and record true block offset BEFORE update_header seeks back to 0
        let block_offset = self.file.seek(SeekFrom::End(0))?;

        // Write block
        self.write_block(&block)?;

        // Update header (computes fresh CRC32)
        self.header.block_count += 1;
        self.header.crc32 = self.header.compute_crc32();
        self.update_header()?;

        // Update importance index with the block's true offset
        self.importance_index.push((block_offset, importance));
        self.read_position = block_offset + (BLOCK_SIZE as u64);

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

    /// Write block to file (zero heap allocation via stack-resident 4KB frame)
    fn write_block(&mut self, block: &M8Block) -> Result<()> {
        let mut buffer = [0u8; BLOCK_SIZE];
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

    /// Read block from file (zero-alloc 4KB stack buffer)
    fn read_block(&mut self) -> Result<Option<M8Block>> {
        let mut buffer = [0u8; BLOCK_SIZE];

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

        let safe_len = (content_len as usize).min(BLOCK_SIZE - cursor);
        let content = buffer[cursor..cursor + safe_len].to_vec();

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

// ============================================================================
// 8B-META-WP-v2.1: LOUDS BIT-PACKED SUFFICIENT DIRECTORY & AST PROJECTION
// ============================================================================

/// Maximum nodes packed into a single L1-cache-resident `LoudsSmartTree` capsule (`64` nodes).
pub const MAX_LOUDS_NODES: usize = 64;

/// Magic bytes (`"SMTR"`) for a 64-byte `MEMNET` `:9876` `smart-tree` `.m8` capsule.
pub const SMTR_CAPSULE_MAGIC: [u8; 4] = *b"SMTR";

/// Current wire version of the `8B-META-WP-v2.1` `SMTR` 64-byte capsule.
pub const SMTR_CAPSULE_VERSION: u8 = 2;

/// Semantic AST / Directory Node Ontology (`3-bit` tag, `0..=7`).
#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum LoudsNodeKind {
    Dir = 0,
    RustModule = 1,
    FnItem = 2,
    StructType = 3,
    TraitCap = 4,
    TestSpec = 5,
    ConfigToml = 6,
    IgnoredNoise = 7,
}

impl LoudsNodeKind {
    #[inline]
    pub const fn from_u8(b: u8) -> Self {
        match b & 0x07 {
            0 => Self::Dir,
            1 => Self::RustModule,
            2 => Self::FnItem,
            3 => Self::StructType,
            4 => Self::TraitCap,
            5 => Self::TestSpec,
            6 => Self::ConfigToml,
            _ => Self::IgnoredNoise,
        }
    }

    #[inline]
    pub const fn is_policy_relevant(self, include_tests: bool) -> bool {
        match self {
            Self::IgnoredNoise => false,
            Self::TestSpec => include_tests,
            _ => true,
        }
    }
}

/// Cache-line-aligned (`64B`) LOUDS Bit-Packed Smart Tree (`8B-META-WP-v2.1`).
///
/// Replaces verbose JSON/ASCII tree walks with a 256-bit LOUDS bitvector (`[u64; 4]`),
/// hardware `POPCNT` (`u64::count_ones()`) rank/select navigation, **Indra's Net** recursive
/// SHA-256 Merkle digests, and **Upāya-kauśalya** sufficient projection (`project_sufficient`).
#[repr(C, align(64))]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LoudsSmartTree {
    pub louds_bits: [u64; 4],
    pub node_kinds: [u8; MAX_LOUDS_NODES],
    pub parent_idx: [u8; MAX_LOUDS_NODES],
    pub name_hashes: [u64; MAX_LOUDS_NODES],
    pub content_digests: [[u8; 8]; MAX_LOUDS_NODES],
    pub node_count: usize,
}

impl Default for LoudsSmartTree {
    fn default() -> Self {
        Self::new()
    }
}

impl LoudsSmartTree {
    pub const fn new() -> Self {
        Self {
            louds_bits: [0u64; 4],
            node_kinds: [0u8; MAX_LOUDS_NODES],
            parent_idx: [0u8; MAX_LOUDS_NODES],
            name_hashes: [0u64; MAX_LOUDS_NODES],
            content_digests: [[0u8; 8]; MAX_LOUDS_NODES],
            node_count: 0,
        }
    }

    fn keyed_sha256(key: &[u8; 32], data: &[u8]) -> [u8; 32] {
        use sha2::{Digest, Sha256};
        let mut h = Sha256::new();
        h.update(key);
        h.update(b"|8b-is::louds-smart-tree::v2.1|");
        h.update(data);
        let res = h.finalize();
        let mut out = [0u8; 32];
        out.copy_from_slice(&res);
        out
    }

    #[inline]
    fn push_node_raw(
        &mut self,
        parent: u8,
        kind: LoudsNodeKind,
        name: &str,
        content_sig: &[u8],
        key: &[u8; 32],
    ) -> usize {
        if self.node_count >= MAX_LOUDS_NODES {
            return MAX_LOUDS_NODES - 1;
        }
        let idx = self.node_count;
        self.node_kinds[idx] = kind as u8;
        self.parent_idx[idx] = if idx == 0 { 0 } else { parent };

        let name_mac = Self::keyed_sha256(key, name.as_bytes());
        self.name_hashes[idx] = u64::from_le_bytes(name_mac[0..8].try_into().unwrap());
        let content_mac = Self::keyed_sha256(key, content_sig);
        self.content_digests[idx].copy_from_slice(&content_mac[0..8]);

        self.node_count += 1;
        idx
    }

    pub fn insert_node(
        &mut self,
        parent: u8,
        kind: LoudsNodeKind,
        name: &str,
        content_sig: &[u8],
        key: &[u8; 32],
    ) -> usize {
        let idx = self.push_node_raw(parent, kind, name, content_sig, key);
        self.rebuild_louds_bitmap();
        idx
    }

    /// Builds a `LoudsSmartTree` directly from a slice of scanned `smart-tree` `FileNode`s
    /// in a single $O(N)$ pass, reconstructing directory hierarchy from `FileNode::depth`.
    pub fn from_file_nodes(nodes: &[crate::scanner::FileNode], key: &[u8; 32]) -> Self {
        let mut tree = Self::new();
        let mut depth_parent = [0u8; MAX_LOUDS_NODES];

        for node in nodes.iter().take(MAX_LOUDS_NODES) {
            let name = node
                .path
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("root");
            let kind = if node.is_dir {
                if node.is_ignored
                    || matches!(name, "target" | "node_modules" | ".git" | ".DS_Store")
                {
                    LoudsNodeKind::IgnoredNoise
                } else {
                    LoudsNodeKind::Dir
                }
            } else if node.is_ignored || name == ".DS_Store" {
                LoudsNodeKind::IgnoredNoise
            } else if name == "Cargo.toml" || name == "flake.nix" || name.ends_with(".toml") {
                LoudsNodeKind::ConfigToml
            } else if name.ends_with("_test.rs") || name.contains("test") {
                LoudsNodeKind::TestSpec
            } else if name.ends_with(".rs") {
                LoudsNodeKind::RustModule
            } else {
                LoudsNodeKind::StructType
            };

            let depth = node.depth.min(MAX_LOUDS_NODES - 1);
            let parent = if tree.node_count == 0 || depth == 0 {
                0u8
            } else {
                depth_parent[depth - 1]
            };

            let mut sig = [0u8; 12];
            sig[0..8].copy_from_slice(&node.size.to_le_bytes());
            sig[8..12].copy_from_slice(&node.permissions.to_le_bytes());
            let idx = tree.push_node_raw(parent, kind, name, &sig, key);
            if node.is_dir {
                depth_parent[depth] = idx as u8;
            }
        }
        tree.rebuild_louds_bitmap();
        tree
    }

    /// Single-pass $O(N)$ LOUDS bitvector synthesis using stack-resident out-degree counts.
    fn rebuild_louds_bitmap(&mut self) {
        self.louds_bits = [0u64; 4];
        if self.node_count == 0 {
            return;
        }
        let mut child_counts = [0u8; MAX_LOUDS_NODES];
        for candidate in 1..self.node_count {
            let p = self.parent_idx[candidate] as usize;
            if p < self.node_count {
                child_counts[p] = child_counts[p].saturating_add(1);
            }
        }

        let mut bit_cursor = 0usize;
        Self::set_bit(&mut self.louds_bits, bit_cursor);
        bit_cursor += 2; // super-root "10"

        for &deg in &child_counts[..self.node_count] {
            for _ in 0..deg {
                if bit_cursor < 256 {
                    Self::set_bit(&mut self.louds_bits, bit_cursor);
                }
                bit_cursor += 1;
            }
            bit_cursor += 1; // terminating '0' bit for this node's child run
        }
    }

    /// Computes node out-degrees using Wild Linker's zero-cost in-place
    /// `Vec<u32>` <-> `Vec<AtomicU32>` conversion (`wild_perf::parallel_count_by_index`).
    pub fn atomic_out_degrees(&self) -> Vec<u32> {
        if self.node_count == 0 {
            return Vec::new();
        }
        let parents: Vec<u32> = self.parent_idx[1..self.node_count]
            .iter()
            .map(|&p| p as u32)
            .collect();
        crate::wild_perf::parallel_count_by_index(self.node_count, &parents)
    }

    #[inline]
    fn set_bit(words: &mut [u64; 4], bit_pos: usize) {
        let w = bit_pos >> 6;
        let b = bit_pos & 63;
        if w < 4 {
            words[w] |= 1u64 << b;
        }
    }

    /// Hardware `POPCNT` (`felix_asm::felix_popcnt_u64`) $O(1)$ rank-1 query over `louds_bits[0..=bit_pos]`.
    #[inline]
    pub fn rank1(&self, bit_pos: usize) -> u32 {
        let clamped = bit_pos.min(255);
        let word_idx = clamped >> 6;
        let bit_idx = clamped & 63;
        let mut total = 0u32;
        for w in 0..word_idx {
            total += crate::felix_asm::felix_popcnt_u64(self.louds_bits[w]);
        }
        let mask = if bit_idx == 63 {
            u64::MAX
        } else {
            (1u64 << (bit_idx + 1)) - 1
        };
        total + crate::felix_asm::felix_popcnt_u64(self.louds_bits[word_idx] & mask)
    }

    /// Complement $O(1)$ rank-0 query over `louds_bits[0..=bit_pos]`.
    #[inline]
    pub fn rank0(&self, bit_pos: usize) -> u32 {
        let clamped = bit_pos.min(255);
        ((clamped + 1) as u32).saturating_sub(self.rank1(clamped))
    }

    /// Hardware `POPCNT` + `TZCNT` (`felix_asm::felix_tzcnt_u64`) $O(1)$ select-1 query.
    /// Returns the bit index (0..256) of the `target_rank`-th (1-based) set bit.
    #[inline]
    pub fn select1(&self, target_rank: u32) -> Option<usize> {
        if target_rank == 0 {
            return None;
        }
        let mut remaining = target_rank;
        for (w_idx, &word) in self.louds_bits.iter().enumerate() {
            let ones = crate::felix_asm::felix_popcnt_u64(word);
            if remaining <= ones {
                let mut cur = word;
                for _ in 1..remaining {
                    cur &= cur - 1; // clear lowest set bit (BLSR)
                }
                return Some((w_idx << 6) + (crate::felix_asm::felix_tzcnt_u64(cur) as usize));
            }
            remaining -= ones;
        }
        None
    }

    /// Out-degree (number of direct children) of `node_idx`.
    #[inline]
    pub fn degree(&self, node_idx: usize) -> usize {
        if node_idx >= self.node_count {
            return 0;
        }
        let mut deg = 0usize;
        for candidate in 1..self.node_count {
            if (self.parent_idx[candidate] as usize) == node_idx {
                deg += 1;
            }
        }
        deg
    }

    /// Recursive **Indra's Net** Merkle digest of the subtree rooted at `root_idx`.
    pub fn subtree_digest(&self, root_idx: usize, key: &[u8; 32]) -> [u8; 32] {
        if root_idx >= self.node_count {
            return [0u8; 32];
        }
        let mut buf = Vec::with_capacity(64);
        buf.push(self.node_kinds[root_idx]);
        buf.extend_from_slice(&self.name_hashes[root_idx].to_le_bytes());
        buf.extend_from_slice(&self.content_digests[root_idx]);
        for child in 1..self.node_count {
            if (self.parent_idx[child] as usize) == root_idx {
                let child_hash = self.subtree_digest(child, key);
                buf.extend_from_slice(&child_hash);
            }
        }
        Self::keyed_sha256(key, &buf)
    }

    /// **Upāya-kauśalya (`Sufficient Projections`, Guimond 2026)**:
    /// Prunes `IgnoredNoise` (and optionally `TestSpec`) while preserving topological closure.
    pub fn project_sufficient(&self, include_tests: bool) -> Self {
        let mut projected = Self::new();
        if self.node_count == 0 {
            return projected;
        }
        let mut old_to_new: [Option<u8>; MAX_LOUDS_NODES] = [None; MAX_LOUDS_NODES];
        for old_idx in 0..self.node_count {
            let kind = LoudsNodeKind::from_u8(self.node_kinds[old_idx]);
            if !kind.is_policy_relevant(include_tests) {
                continue;
            }
            let new_parent = if old_idx == 0 {
                0u8
            } else {
                match old_to_new[self.parent_idx[old_idx] as usize] {
                    Some(p) => p,
                    None => continue,
                }
            };
            let new_idx = projected.node_count;
            old_to_new[old_idx] = Some(new_idx as u8);
            projected.node_kinds[new_idx] = self.node_kinds[old_idx];
            projected.parent_idx[new_idx] = new_parent;
            projected.name_hashes[new_idx] = self.name_hashes[old_idx];
            projected.content_digests[new_idx] = self.content_digests[old_idx];
            projected.node_count += 1;
        }
        projected.rebuild_louds_bitmap();
        projected
    }

    /// Packs the tree into a 64-byte `SMTR` `.m8` structural capsule.
    pub fn pack_m8_capsule(&self, key: &[u8; 32]) -> [u8; 64] {
        let mut out = [0u8; 64];
        out[0..4].copy_from_slice(&SMTR_CAPSULE_MAGIC);
        out[4] = SMTR_CAPSULE_VERSION;
        out[5] = self.node_count as u8;

        let mut louds_bytes = [0u8; 32];
        for (i, &w) in self.louds_bits.iter().enumerate() {
            louds_bytes[i * 8..(i + 1) * 8].copy_from_slice(&w.to_le_bytes());
        }
        let crc16 = (crc32_branchless(&louds_bytes) & 0xFFFF) as u16;
        out[6..8].copy_from_slice(&crc16.to_le_bytes());
        out[8..16].copy_from_slice(&self.louds_bits[0].to_le_bytes());

        let root_digest = self.subtree_digest(0, key);
        out[16..48].copy_from_slice(&root_digest);

        let mac = Self::keyed_sha256(key, &out[0..48]);
        out[48..64].copy_from_slice(&mac[0..16]);
        out
    }

    /// Constant-time verification of a 64-byte `SMTR` `.m8` capsule.
    pub fn verify_m8_capsule(capsule: &[u8; 64], key: &[u8; 32]) -> bool {
        if capsule[0..4] != SMTR_CAPSULE_MAGIC || capsule[4] != SMTR_CAPSULE_VERSION {
            return false;
        }
        let expected = Self::keyed_sha256(key, &capsule[0..48]);
        let mut diff = 0u8;
        for i in 0..16 {
            diff |= capsule[48 + i] ^ expected[i];
        }
        diff == 0
    }
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

    #[test]
    fn test_louds_smart_tree_and_m8_block_hash_chain() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("chained.m8");

        let mut m8 = M8BinaryFile::create(&path).unwrap();
        m8.append_block(b".rs module first", 0.9).unwrap();
        m8.append_block(b"Claude context second", 0.95).unwrap();

        let second = m8.read_backwards().unwrap().unwrap();
        let first = m8.read_backwards().unwrap().unwrap();
        assert_eq!(
            first.prev_hash, [0u8; 32],
            "genesis block has zero prev_hash"
        );
        assert_ne!(
            second.prev_hash, [0u8; 32],
            "second block chains non-zero SHA-256 digest of first block"
        );
        assert_eq!(first.token_id, 0x90, ".rs reserved token");
        assert_eq!(second.token_id, 0xFFFE, "Claude reserved token");

        // Verify LoudsSmartTree POPCNT rank, Indra's Net digest, and 64B SMTR capsule
        let key = [0x77u8; 32];
        let mut tree = LoudsSmartTree::new();
        let r = tree.insert_node(0, LoudsNodeKind::Dir, "smart-tree", b"", &key);
        let s = tree.insert_node(r as u8, LoudsNodeKind::Dir, "src", b"", &key);
        let _n = tree.insert_node(r as u8, LoudsNodeKind::IgnoredNoise, "target", b"", &key);
        let _m = tree.insert_node(
            s as u8,
            LoudsNodeKind::RustModule,
            "mem8_binary.rs",
            b"v2.1",
            &key,
        );

        assert_eq!(tree.node_count, 4);
        assert_eq!(
            tree.degree(r),
            2,
            "root has 2 direct children (src + target)"
        );
        assert_eq!(tree.degree(s), 1, "src has 1 direct child (mem8_binary.rs)");
        let proj = tree.project_sufficient(false);
        assert_eq!(
            proj.node_count, 3,
            "IgnoredNoise pruned by Upaya-kausalya projection"
        );
        assert_eq!(proj.degree(0), 1, "projected root has 1 direct child (src)");
        assert_eq!(proj.select1(1), Some(0), "super-root 1-bit is at bit 0");
        assert_eq!(
            proj.rank1(16) + proj.rank0(16),
            17,
            "rank1 + rank0 equals bit prefix length"
        );
        let cap = proj.pack_m8_capsule(&key);
        assert!(LoudsSmartTree::verify_m8_capsule(&cap, &key));
    }
}
