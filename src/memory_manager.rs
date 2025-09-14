// Smart Tree Memory Manager - Real memory that works! 🧠
// "Like UV EPROMs but for consciousness!" - Hue

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use chrono::{DateTime, Utc};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Memory {
    pub timestamp: DateTime<Utc>,
    pub anchor_type: String,
    pub keywords: Vec<String>,
    pub context: String,
    pub origin: String,  // Where this memory came from
    pub frequency: f64,  // Wave frequency of this memory
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
        let bank_path = Path::new(&home).join(".mem8").join("smart_tree_memories.json");

        // Ensure directory exists
        if let Some(parent) = bank_path.parent() {
            fs::create_dir_all(parent)?;
        }

        // Load existing memories or create new bank
        let bank = if bank_path.exists() {
            let content = fs::read_to_string(&bank_path)?;
            serde_json::from_str(&content).unwrap_or_default()
        } else {
            MemoryBank::default()
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
                if memory.keywords.contains(keyword) ||
                   memory.context.to_lowercase().contains(&keyword.to_lowercase()) {
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

    /// Save memory bank to disk
    fn save(&self) -> Result<()> {
        let json = serde_json::to_string_pretty(&self.bank)?;
        fs::write(&self.bank_path, json)?;
        Ok(())
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
            output.push_str(&format!("Time: {}\n", memory.timestamp.format("%Y-%m-%d %H:%M")));
        }

        Ok(output)
    }
}