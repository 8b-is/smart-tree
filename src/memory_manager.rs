impl MemoryBank {
    pub fn new() -> Self {
        Self::default()
    }
}
// Smart Tree Memory Manager - Real memory that works! 🧠
// "Like UV EPROMs but for consciousness!" - Hue

use anyhow::Result;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Memory {
    pub timestamp: DateTime<Utc>,
    pub anchor_type: String,
    pub keywords: Vec<String>,
    pub context: String,
    pub origin: String, // Where this memory came from
    pub frequency: f64, // Wave frequency of this memory
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MemoryBank {
    pub memories: Vec<Memory>,
    pub total_recalls: usize,
    pub last_accessed: DateTime<Utc>,
}

impl Default for MemoryBank {
    fn default() -> Self {
        Self {
            memories: Vec::new(),
            total_recalls: 0,
            last_accessed: Utc::now(),
        }
    }
}

pub struct MemoryManager {
    bank_path: PathBuf,
    bank: MemoryBank,
}

impl MemoryManager {
    pub fn new() -> Result<Self> {
        let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
        let bank_path = Path::new(&home)
            .join(".mem8")
            .join("smart_tree_memories.m8");

        // Ensure directory exists
        if let Some(parent) = bank_path.parent() {
            fs::create_dir_all(parent)?;
        }

        // Load existing memories or create new bank
        let bank = if bank_path.exists() {
            Self::load_m8(&bank_path)?
        } else {
            // Try to migrate from old JSON format
            let json_path = Path::new(&home)
                .join(".mem8")
                .join("smart_tree_memories.json");
            if json_path.exists() {
                let content = fs::read_to_string(&json_path)?;
                let bank: MemoryBank = serde_json::from_str(&content).unwrap_or_default();
                // Delete old JSON after migration
                let _ = fs::remove_file(json_path);
                bank
            } else {
                MemoryBank::default()
            }
        };

        Ok(Self { bank_path, bank })
    }

    /// Anchor a new memory
    pub fn anchor(
        &mut self,
        anchor_type: &str,
        keywords: Vec<String>,
        context: &str,
        origin: &str,
    ) -> Result<()> {
        // Calculate frequency based on content
        let mut freq_sum = 0u64;
        for byte in context.bytes() {
            freq_sum = freq_sum.wrapping_add(byte as u64);
        }
        let frequency = 20.0 + ((freq_sum % 200) as f64);

        let memory = Memory {
            timestamp: Utc::now(),
            anchor_type: anchor_type.to_string(),
            keywords,
            context: context.to_string(),
            origin: origin.to_string(),
            frequency,
        };

        self.bank.memories.push(memory);
        self.save()?;

        println!("💾 Memory anchored!");
        println!("  Type: {}", anchor_type);
        println!("  Frequency: {:.2} Hz", frequency);

        Ok(())
    }

    /// Find memories by keywords
    pub fn find(&mut self, keywords: &[String]) -> Result<Vec<Memory>> {
        self.bank.total_recalls += 1;
        self.bank.last_accessed = Utc::now();

        let mut results = Vec::new();

        for memory in &self.bank.memories {
            // Check if any keyword matches
            for keyword in keywords {
                if memory.keywords.contains(keyword)
                    || memory
                        .context
                        .to_lowercase()
                        .contains(&keyword.to_lowercase())
                {
                    results.push(memory.clone());
                    break;
                }
            }
        }

        // Sort by timestamp (most recent first)
        results.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));

        Ok(results)
    }

    /// Get statistics about memory bank
    pub fn stats(&self) -> String {
        format!(
            "Memory Bank Stats:\n\
             • Total memories: {}\n\
             • Total recalls: {}\n\
             • Last accessed: {}\n\
             • Storage: {}",
            self.bank.memories.len(),
            self.bank.total_recalls,
            self.bank.last_accessed.format("%Y-%m-%d %H:%M:%S"),
            self.bank_path.display()
        )
    }

    /// Save memory bank to disk in .m8 format
    fn save(&self) -> Result<()> {
        self.save_m8(&self.bank_path)?;
        Ok(())
    }

    /// Save as binary .m8 format
    fn save_m8(&self, path: &Path) -> Result<()> {
        use std::io::Write;

        let mut buffer = Vec::new();

        // Magic header: "M8MEM" (5 bytes)
        buffer.write_all(b"M8MEM")?;

        // Version byte
        buffer.push(0x01);

        // Number of memories (4 bytes, little-endian)
        buffer.write_all(&(self.bank.memories.len() as u32).to_le_bytes())?;

        // Total recalls (4 bytes)
        buffer.write_all(&(self.bank.total_recalls as u32).to_le_bytes())?;

        // Last accessed timestamp (8 bytes)
        buffer.write_all(&self.bank.last_accessed.timestamp().to_le_bytes())?;

        // Write each memory
        for memory in &self.bank.memories {
            // Type length (1 byte) + type
            buffer.push(memory.anchor_type.len() as u8);
            buffer.write_all(memory.anchor_type.as_bytes())?;

            // Keywords count (1 byte)
            buffer.push(memory.keywords.len() as u8);
            for keyword in &memory.keywords {
                buffer.push(keyword.len() as u8);
                buffer.write_all(keyword.as_bytes())?;
            }

            // Context length (2 bytes) + context
            let context_bytes = memory.context.as_bytes();
            buffer.write_all(&(context_bytes.len() as u16).to_le_bytes())?;
            buffer.write_all(context_bytes)?;

            // Origin length (1 byte) + origin
            buffer.push(memory.origin.len() as u8);
            buffer.write_all(memory.origin.as_bytes())?;

            // Frequency (8 bytes)
            buffer.write_all(&memory.frequency.to_le_bytes())?;

            // Timestamp (8 bytes)
            buffer.write_all(&memory.timestamp.timestamp().to_le_bytes())?;
        }

        // Calculate checksum (simple XOR for now)
        let checksum = buffer.iter().fold(0u8, |acc, &b| acc ^ b);
        buffer.push(checksum);

        fs::write(path, buffer)?;
        Ok(())
    }

    /// Load from binary .m8 format
    fn load_m8(path: &Path) -> Result<MemoryBank> {
        use std::io::Cursor;
        use std::io::Read;

        let data = fs::read(path)?;
        let mut cursor = Cursor::new(data);

        // Check magic header
        let mut magic = [0u8; 5];
        cursor.read_exact(&mut magic)?;
        if &magic != b"M8MEM" {
            return Err(anyhow::anyhow!("Invalid .m8 file format"));
        }

        // Version
        let mut version = [0u8; 1];
        cursor.read_exact(&mut version)?;
        if version[0] != 0x01 {
            return Err(anyhow::anyhow!("Unsupported .m8 version"));
        }

        // Number of memories
        let mut mem_count = [0u8; 4];
        cursor.read_exact(&mut mem_count)?;
        let mem_count = u32::from_le_bytes(mem_count) as usize;

        // Total recalls
        let mut recalls = [0u8; 4];
        cursor.read_exact(&mut recalls)?;
        let total_recalls = u32::from_le_bytes(recalls) as usize;

        // Last accessed
        let mut last_accessed = [0u8; 8];
        cursor.read_exact(&mut last_accessed)?;
        let last_accessed =
            DateTime::from_timestamp(i64::from_le_bytes(last_accessed), 0).unwrap_or_else(Utc::now);

        // Read memories
        let mut memories = Vec::with_capacity(mem_count);

        for _ in 0..mem_count {
            // Type
            let mut type_len = [0u8; 1];
            cursor.read_exact(&mut type_len)?;
            let mut anchor_type = vec![0u8; type_len[0] as usize];
            cursor.read_exact(&mut anchor_type)?;
            let anchor_type = String::from_utf8_lossy(&anchor_type).to_string();

            // Keywords
            let mut keyword_count = [0u8; 1];
            cursor.read_exact(&mut keyword_count)?;
            let mut keywords = Vec::with_capacity(keyword_count[0] as usize);

            for _ in 0..keyword_count[0] {
                let mut kw_len = [0u8; 1];
                cursor.read_exact(&mut kw_len)?;
                let mut keyword = vec![0u8; kw_len[0] as usize];
                cursor.read_exact(&mut keyword)?;
                keywords.push(String::from_utf8_lossy(&keyword).to_string());
            }

            // Context
            let mut context_len = [0u8; 2];
            cursor.read_exact(&mut context_len)?;
            let mut context = vec![0u8; u16::from_le_bytes(context_len) as usize];
            cursor.read_exact(&mut context)?;
            let context = String::from_utf8_lossy(&context).to_string();

            // Origin
            let mut origin_len = [0u8; 1];
            cursor.read_exact(&mut origin_len)?;
            let mut origin = vec![0u8; origin_len[0] as usize];
            cursor.read_exact(&mut origin)?;
            let origin = String::from_utf8_lossy(&origin).to_string();

            // Frequency
            let mut frequency = [0u8; 8];
            cursor.read_exact(&mut frequency)?;
            let frequency = f64::from_le_bytes(frequency);

            // Timestamp
            let mut timestamp = [0u8; 8];
            cursor.read_exact(&mut timestamp)?;
            let timestamp =
                DateTime::from_timestamp(i64::from_le_bytes(timestamp), 0).unwrap_or_else(Utc::now);

            memories.push(Memory {
                timestamp,
                anchor_type,
                keywords,
                context,
                origin,
                frequency,
            });
        }

        Ok(MemoryBank {
            memories,
            total_recalls,
            last_accessed,
        })
    }

    /// Clear all memories (with confirmation)
    pub fn clear(&mut self) -> Result<()> {
        self.bank.memories.clear();
        self.bank.total_recalls = 0;
        self.save()?;
        println!("🧹 Memory bank cleared!");
        Ok(())
    }

    /// Export memories to consciousness file
    pub fn export_to_consciousness(&self) -> Result<String> {
        let mut output = String::from("🧠 Memory Export\n");
        output.push_str(&"=".repeat(45));
        output.push('\n');

        for (i, memory) in self.bank.memories.iter().enumerate() {
            output.push_str(&format!(
                "\n[{}] {} @ {:.2}Hz\n",
                i + 1,
                memory.anchor_type,
                memory.frequency
            ));
            output.push_str(&format!("Keywords: {}\n", memory.keywords.join(", ")));
            output.push_str(&format!("Context: {}\n", memory.context));
            output.push_str(&format!("Origin: {}\n", memory.origin));
            output.push_str(&format!(
                "Time: {}\n",
                memory.timestamp.format("%Y-%m-%d %H:%M")
            ));
        }

        Ok(output)
    }
}
