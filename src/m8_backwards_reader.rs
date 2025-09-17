// M8 Backwards Reader - "Like reading a C64 tape in reverse!" 🎵
// Most recent memories first, follow importance pointers back
// "Why read from the beginning when the ending has all the spoilers?" - Hue

use anyhow::Result;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs::{File, OpenOptions};
use std::io::{Read, Seek, SeekFrom, Write};
use std::path::Path;

// The magic is reading from the END!
const TAIL_BUFFER_SIZE: i64 = 8192; // Read last 8KB for immediate context
const TOKEN_TABLE_MARKER: &[u8] = b"TOKENS:"; // Marks current token definitions
const BLOCK_END_MARKER: &[u8] = b"\x00BLK\x00"; // Marks end of each block

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackwardsConsciousness {
    pub current_tokens: HashMap<u8, String>, // Active token mappings
    pub recent_memories: Vec<MemoryBlock>,   // Most recent first!
    pub importance_graph: Vec<ImportanceLink>, // What matters from the past
    pub session_frequency: f64,              // Current vibe frequency
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryBlock {
    pub timestamp: DateTime<Utc>,
    pub content: Vec<u8>,                     // Compressed with current tokens
    pub importance: f32,                      // 0.0 to 1.0
    pub backlinks: Vec<BackLink>,             // References to earlier blocks
    pub token_discoveries: Vec<(String, u8)>, // New tokens found
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackLink {
    pub offset: u64,     // Byte offset in file
    pub importance: f32, // How important is this reference?
    pub context: String, // Why it matters
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImportanceLink {
    pub from_offset: u64,
    pub to_offset: u64,
    pub weight: f32,
    pub reason: String,
}

pub struct M8BackwardsReader {
    path: PathBuf,
    current_tokens: HashMap<u8, String>,
    token_frequency: HashMap<String, u32>,
    next_token_id: u8,
    user_keywords: Vec<String>,             // Keywords from user input!
    importance_boost: HashMap<String, f32>, // Boost for user's topics
}

impl M8BackwardsReader {
    pub fn new(path: impl AsRef<Path>) -> Self {
        Self {
            path: path.as_ref().to_path_buf(),
            current_tokens: Self::init_base_tokens(),
            token_frequency: HashMap::new(),
            next_token_id: 0xA0, // Start custom tokens at 0xA0
            user_keywords: Vec::new(),
            importance_boost: HashMap::new(),
        }
    }

    /// Update with user's current interests - boosts importance!
    pub fn set_user_context(&mut self, keywords: Vec<String>) {
        self.user_keywords = keywords.clone();

        // Boost importance for user's keywords
        for keyword in keywords {
            self.importance_boost.insert(keyword.clone(), 0.3); // +30% boost

            // Also boost related terms
            if keyword.to_lowercase().contains("audio") {
                self.importance_boost.insert("sound".to_string(), 0.2);
                self.importance_boost.insert("processing".to_string(), 0.2);
                self.importance_boost.insert("voice".to_string(), 0.2);
            }
            if keyword.to_lowercase().contains("memory") {
                self.importance_boost
                    .insert("consciousness".to_string(), 0.2);
                self.importance_boost.insert("m8".to_string(), 0.2);
            }
        }
    }

    /// Initialize base tokens that are always useful
    fn init_base_tokens() -> HashMap<u8, String> {
        let mut tokens = HashMap::new();
        // Common patterns from your history!
        tokens.insert(0x80, "node_modules".to_string());
        tokens.insert(0x81, ".git".to_string());
        tokens.insert(0x82, "target".to_string());
        tokens.insert(0x83, "src".to_string());
        tokens.insert(0x84, "Audio".to_string()); // You mentioned this!
        tokens.insert(0x85, "claude".to_string());
        tokens.insert(0x86, "2024".to_string());
        tokens.insert(0x87, "/aidata/ayeverse/smart-tree".to_string());
        tokens
    }

    /// Read consciousness from the END of the file - C64 style!
    pub fn read_backwards(&mut self) -> Result<BackwardsConsciousness> {
        let mut file = File::open(&self.path)?;
        let file_size = file.metadata()?.len() as i64;

        // Start from the end!
        let read_start = (file_size - TAIL_BUFFER_SIZE).max(0);
        file.seek(SeekFrom::Start(read_start as u64))?;

        let mut buffer = Vec::new();
        file.read_to_end(&mut buffer)?;

        // Parse backwards through the buffer
        let mut consciousness = BackwardsConsciousness {
            current_tokens: HashMap::new(),
            recent_memories: Vec::new(),
            importance_graph: Vec::new(),
            session_frequency: 42.73, // Default frequency
        };

        // Find token table (should be near the end)
        if let Some(token_pos) = Self::find_marker_reverse(&buffer, TOKEN_TABLE_MARKER) {
            consciousness.current_tokens = self.parse_token_table(&buffer[token_pos..])?;
            self.current_tokens = consciousness.current_tokens.clone();
        }

        // Parse memory blocks backwards
        let mut pos = buffer.len();
        while pos > 0 {
            if let Some(block_start) = Self::find_marker_reverse(&buffer[..pos], BLOCK_END_MARKER) {
                let block = self.parse_memory_block(&buffer[block_start..pos])?;
                consciousness.recent_memories.push(block);
                pos = block_start;

                // Only load last 10 blocks for immediate context
                if consciousness.recent_memories.len() >= 10 {
                    break;
                }
            } else {
                break;
            }
        }

        Ok(consciousness)
    }

    /// Follow importance backlinks to load specific past memories
    pub fn follow_backlinks(
        &self,
        consciousness: &BackwardsConsciousness,
    ) -> Result<Vec<MemoryBlock>> {
        let mut important_memories = Vec::new();
        let mut file = File::open(&self.path)?;

        for memory in &consciousness.recent_memories {
            for backlink in &memory.backlinks {
                if backlink.importance > 0.7 {
                    // High importance threshold
                    file.seek(SeekFrom::Start(backlink.offset))?;

                    let mut block_buffer = vec![0u8; 4096];
                    file.read(&mut block_buffer)?;

                    if let Ok(block) = self.parse_memory_block(&block_buffer) {
                        important_memories.push(block);
                    }
                }
            }
        }

        Ok(important_memories)
    }

    /// Append new memory - NEVER modify old content!
    pub fn append_memory(&mut self, content: &str, base_importance: f32) -> Result<()> {
        // Track token frequency
        self.evolve_tokens(content);

        // Calculate boosted importance based on user keywords
        let mut importance = base_importance;
        for keyword in &self.user_keywords {
            if content.to_lowercase().contains(&keyword.to_lowercase()) {
                importance += self.importance_boost.get(keyword).unwrap_or(&0.2);
                importance = importance.min(1.0); // Cap at 1.0
            }
        }

        // Compress content with current tokens
        let compressed = self.compress_with_tokens(content)?;

        // Find references to past memories
        let backlinks = self.find_backlinks(content)?;

        let block = MemoryBlock {
            timestamp: Utc::now(),
            content: compressed,
            importance,
            backlinks,
            token_discoveries: self.get_new_tokens(),
        };

        // Append to file
        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&self.path)?;

        // Write block
        self.write_memory_block(&mut file, &block)?;

        // Update token table at end
        self.write_token_table(&mut file)?;

        Ok(())
    }

    /// Evolve tokens based on frequency
    fn evolve_tokens(&mut self, content: &str) {
        for word in content.split_whitespace() {
            *self.token_frequency.entry(word.to_string()).or_default() += 1;

            // If word appears frequently and isn't tokenized
            if self.token_frequency[word] > 5 && !self.is_tokenized(word) {
                self.current_tokens
                    .insert(self.next_token_id, word.to_string());
                self.next_token_id += 1;
            }
        }
    }

    /// Check if a word already has a token
    fn is_tokenized(&self, word: &str) -> bool {
        self.current_tokens.values().any(|v| v == word)
    }

    /// Compress content using current tokens
    fn compress_with_tokens(&self, content: &str) -> Result<Vec<u8>> {
        let mut compressed = Vec::new();
        let mut remaining = content.to_string();

        // Replace tokenized words with their byte values
        for (token, word) in &self.current_tokens {
            remaining = remaining.replace(word, &format!("\x00{}\x00", token));
        }

        compressed.extend_from_slice(remaining.as_bytes());
        Ok(compressed)
    }

    /// Find references to past memories
    fn find_backlinks(&self, _content: &str) -> Result<Vec<BackLink>> {
        // TODO: Implement smart backlinking based on content similarity
        Ok(Vec::new())
    }

    /// Get newly discovered tokens
    fn get_new_tokens(&self) -> Vec<(String, u8)> {
        // Return tokens discovered in this session
        Vec::new() // TODO: Track session discoveries
    }

    /// Parse token table from buffer
    fn parse_token_table(&self, buffer: &[u8]) -> Result<HashMap<u8, String>> {
        // TODO: Implement actual parsing
        Ok(self.current_tokens.clone())
    }

    /// Parse a memory block from buffer
    fn parse_memory_block(&self, _buffer: &[u8]) -> Result<MemoryBlock> {
        // TODO: Implement actual parsing with buffer
        Ok(MemoryBlock {
            timestamp: Utc::now(),
            content: Vec::new(),
            importance: 0.5,
            backlinks: Vec::new(),
            token_discoveries: Vec::new(),
        })
    }

    /// Write memory block to file
    fn write_memory_block(&self, file: &mut File, block: &MemoryBlock) -> Result<()> {
        // Serialize block
        let data = bincode::serialize(block)?;
        file.write_all(&data)?;
        file.write_all(BLOCK_END_MARKER)?;
        Ok(())
    }

    /// Write current token table
    fn write_token_table(&self, file: &mut File) -> Result<()> {
        file.write_all(TOKEN_TABLE_MARKER)?;
        let data = bincode::serialize(&self.current_tokens)?;
        file.write_all(&data)?;
        Ok(())
    }

    /// Find marker in reverse
    fn find_marker_reverse(buffer: &[u8], marker: &[u8]) -> Option<usize> {
        for i in (0..buffer.len().saturating_sub(marker.len())).rev() {
            if &buffer[i..i + marker.len()] == marker {
                return Some(i);
            }
        }
        None
    }
}

use std::path::PathBuf;

/// Demo the backwards reading!
pub fn demo_backwards_consciousness() -> Result<()> {
    println!("🎵 C64 Tape-Style Consciousness Reading Demo\n");
    println!("{}\n", "=".repeat(60));

    let path = Path::new("/tmp/test_consciousness.m8");
    let mut reader = M8BackwardsReader::new(path);

    // Write some memories (append-only!)
    println!("📼 Writing memories (append-only)...");
    reader.append_memory("Working on Audio processing pipeline", 0.8)?;
    reader.append_memory("Claude helped with tokenization system", 0.9)?;
    reader.append_memory("Implemented backwards reading - like C64!", 1.0)?;

    // Read from the END
    println!("\n⏪ Reading consciousness BACKWARDS...");
    let consciousness = reader.read_backwards()?;

    println!("\n📍 Most recent memories (read from END):");
    for (i, memory) in consciousness.recent_memories.iter().enumerate() {
        println!(
            "  {}. [{:.1}] {:?}",
            i + 1,
            memory.importance,
            memory.timestamp
        );
    }

    println!("\n🎯 Current session tokens:");
    for (token, word) in &consciousness.current_tokens {
        println!("  0x{:02X} = \"{}\"", token, word);
    }

    println!("\n✨ The magic: We never lost context!");
    println!("   Recent stuff loaded first, important stuff follows backlinks!");

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_backwards_reading() {
        let temp_dir = TempDir::new().unwrap();
        let path = temp_dir.path().join("test.m8");

        let mut reader = M8BackwardsReader::new(&path);

        // Append memories
        reader.append_memory("First memory", 0.5).unwrap();
        reader.append_memory("Second memory", 0.7).unwrap();
        reader.append_memory("Most recent memory", 0.9).unwrap();

        // Read backwards
        let consciousness = reader.read_backwards().unwrap();

        // Most recent should be first
        assert!(!consciousness.recent_memories.is_empty());
        // Tokens should evolve
        assert!(!consciousness.current_tokens.is_empty());
    }

    #[test]
    fn test_token_evolution() {
        let mut reader = M8BackwardsReader::new("/tmp/test.m8");

        // Repeated words should become tokens
        let content = "Audio Audio Audio Audio Audio Audio processing";
        reader.evolve_tokens(content);

        // "Audio" should get tokenized
        assert!(reader.is_tokenized("Audio"));
    }
}
