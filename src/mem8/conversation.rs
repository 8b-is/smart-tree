//! Conversation Memory System for Smart Tree
//! Intelligently detects, parses, and stores conversation data in MEM|8 format
//!
//! "Every conversation is a wave pattern waiting to be preserved" - Omni

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};
use std::fs;
use std::path::PathBuf;
use std::time::SystemTime;

use super::wave::{MemoryWave, WaveGrid};

/// Conversation memory manager
pub struct ConversationMemory {
    /// Base directory for storing conversations (~/.mem8/conversations/)
    base_path: PathBuf,
    /// Wave grid for memory storage (lazy-initialized to avoid 34GB allocation!)
    wave_grid: Option<WaveGrid>,
    /// Smart structure analyzer
    analyzer: ConversationAnalyzer,
}

impl ConversationMemory {
    /// Create a new conversation memory system
    pub fn new() -> Result<Self> {
        let home_dir = dirs::home_dir().context("Failed to get home directory")?;

        let base_path = home_dir.join(".mem8").join("conversations");

        // Create directory if it doesn't exist
        fs::create_dir_all(&base_path)?;

        Ok(Self {
            base_path,
            wave_grid: None, // Don't allocate 34GB until actually needed!
            analyzer: ConversationAnalyzer::new(),
        })
    }
    
    /// Ensure wave grid is initialized (lazy initialization)
    fn ensure_wave_grid(&mut self) {
        if self.wave_grid.is_none() {
            self.wave_grid = Some(WaveGrid::new());
        }
    }

    /// Intelligently detect and save conversation from JSON
    pub fn save_conversation(
        &mut self,
        json_data: &Value,
        source: Option<&str>,
    ) -> Result<PathBuf> {
        // Ensure wave grid is initialized before using it
        self.ensure_wave_grid();
        
        // Analyze the JSON structure to understand conversation format
        let analysis = self.analyzer.analyze(json_data)?;

        // Generate a unique filename based on content
        let timestamp = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)?
            .as_secs();

        let filename = format!(
            "conv_{}_{}_{}.m8",
            analysis.conversation_type.as_str(),
            source.unwrap_or("unknown"),
            timestamp
        );

        let file_path = self.base_path.join(&filename);

        // Convert conversation to wave patterns
        let waves = self.conversation_to_waves(&analysis)?;

        // Store in wave grid
        if let Some(ref mut grid) = self.wave_grid {
            for (idx, wave) in waves.iter().enumerate() {
                let x = (idx % 256) as u8;
                let y = ((idx / 256) % 256) as u8;
                let z = (idx / (256 * 256)) as u16;
                grid.store(x, y, z, wave.clone());
            }
        }

        // For now, save directly as JSON until M8Writer is properly implemented
        // TODO: Implement proper M8Writer integration

        // Also save a JSON companion file for easy retrieval
        let json_path = file_path.with_extension("json");
        fs::write(&json_path, serde_json::to_string_pretty(json_data)?)?;

        println!("🧠 Conversation saved to MEM|8: {}", filename);
        println!("   Type: {:?}", analysis.conversation_type);
        println!("   Messages: {}", analysis.message_count);
        println!("   Participants: {}", analysis.participants.join(", "));

        Ok(file_path)
    }

    /// Convert conversation analysis to wave patterns
    fn conversation_to_waves(&self, analysis: &ConversationAnalysis) -> Result<Vec<MemoryWave>> {
        let mut waves = Vec::new();

        for message in &analysis.messages {
            // Map message emotion to frequency
            let frequency = match message.emotion.as_str() {
                "happy" | "excited" => 100.0,    // High energy
                "sad" | "worried" => 20.0,       // Low energy
                "angry" | "frustrated" => 150.0, // Intense
                "neutral" | "thinking" => 50.0,  // Balanced
                _ => 44.1,                       // Default to audio baseline
            };

            // Create wave with message characteristics
            let mut wave = MemoryWave::new(frequency, message.importance as f32);
            wave.phase = message.timestamp as f32;
            wave.valence = match message.emotion.as_str() {
                "happy" | "excited" => 0.8,
                "sad" | "worried" => -0.5,
                "angry" | "frustrated" => -0.8,
                _ => 0.0,
            };
            wave.arousal = message.importance as f32 / 10.0;

            waves.push(wave);
        }

        Ok(waves)
    }

    /// List all saved conversations
    pub fn list_conversations(&self) -> Result<Vec<ConversationSummary>> {
        let mut summaries = Vec::new();

        for entry in fs::read_dir(&self.base_path)? {
            let entry = entry?;
            let path = entry.path();

            if path.extension() == Some(std::ffi::OsStr::new("m8")) {
                // Read the companion JSON for quick summary
                let json_path = path.with_extension("json");
                if json_path.exists() {
                    let json_str = fs::read_to_string(&json_path)?;
                    let json_data: Value = serde_json::from_str(&json_str)?;

                    let analysis = self.analyzer.analyze(&json_data)?;
                    summaries.push(ConversationSummary {
                        file_name: path.file_name().unwrap().to_string_lossy().to_string(),
                        conversation_type: analysis.conversation_type,
                        message_count: analysis.message_count,
                        participants: analysis.participants,
                        timestamp: entry.metadata()?.modified()?,
                    });
                }
            }
        }

        Ok(summaries)
    }
}

/// Smart conversation structure analyzer
pub struct ConversationAnalyzer {
    /// Known conversation patterns
    patterns: Vec<ConversationPattern>,
}

impl Default for ConversationAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}

impl ConversationAnalyzer {
    pub fn new() -> Self {
        Self {
            patterns: Self::default_patterns(),
        }
    }

    /// Analyze JSON to understand conversation structure
    pub fn analyze(&self, json_data: &Value) -> Result<ConversationAnalysis> {
        // Try to detect the conversation format
        let conversation_type = self.detect_type(json_data);

        // Extract messages based on detected type
        let messages = self.extract_messages(json_data, &conversation_type)?;

        // Identify participants
        let participants = self.extract_participants(&messages);

        // Get message count before moving messages
        let message_count = messages.len();

        // Build metadata
        let mut metadata = Map::new();
        metadata.insert(
            "type".to_string(),
            Value::String(conversation_type.to_string()),
        );
        metadata.insert("version".to_string(), Value::String("1.0".to_string()));

        Ok(ConversationAnalysis {
            conversation_type,
            messages,
            participants,
            message_count,
            metadata,
        })
    }

    /// Detect conversation type from JSON structure
    fn detect_type(&self, json_data: &Value) -> ConversationType {
        // Check for common conversation patterns
        if json_data.get("messages").is_some() {
            ConversationType::ChatGPT
        } else if json_data.get("conversation").is_some() {
            ConversationType::Claude
        } else if json_data.get("history").is_some() {
            ConversationType::Generic
        } else if json_data.is_array() {
            ConversationType::MessageArray
        } else {
            ConversationType::Unknown
        }
    }

    /// Extract messages from JSON based on type
    fn extract_messages(
        &self,
        json_data: &Value,
        conv_type: &ConversationType,
    ) -> Result<Vec<Message>> {
        let mut messages = Vec::new();

        match conv_type {
            ConversationType::ChatGPT => {
                if let Some(msgs) = json_data.get("messages").and_then(|m| m.as_array()) {
                    for (idx, msg) in msgs.iter().enumerate() {
                        messages.push(Message {
                            content: msg
                                .get("content")
                                .and_then(|c| c.as_str())
                                .unwrap_or("")
                                .to_string(),
                            role: msg
                                .get("role")
                                .and_then(|r| r.as_str())
                                .unwrap_or("unknown")
                                .to_string(),
                            timestamp: idx as u64,
                            emotion: self.detect_emotion(msg),
                            importance: self.calculate_importance(msg),
                        });
                    }
                }
            }
            ConversationType::MessageArray => {
                if let Some(msgs) = json_data.as_array() {
                    for (idx, msg) in msgs.iter().enumerate() {
                        messages.push(Message {
                            content: msg
                                .get("text")
                                .or_else(|| msg.get("content"))
                                .and_then(|c| c.as_str())
                                .unwrap_or("")
                                .to_string(),
                            role: msg
                                .get("sender")
                                .or_else(|| msg.get("role"))
                                .and_then(|r| r.as_str())
                                .unwrap_or("unknown")
                                .to_string(),
                            timestamp: idx as u64,
                            emotion: self.detect_emotion(msg),
                            importance: self.calculate_importance(msg),
                        });
                    }
                }
            }
            _ => {
                // Try to extract any text content
                self.extract_generic_messages(json_data, &mut messages, 0);
            }
        }

        Ok(messages)
    }

    /// Recursively extract text from generic JSON
    fn extract_generic_messages(&self, value: &Value, messages: &mut Vec<Message>, depth: usize) {
        if depth > 10 {
            return; // Prevent infinite recursion
        }

        match value {
            Value::String(s) if s.len() > 20 => {
                messages.push(Message {
                    content: s.clone(),
                    role: "extracted".to_string(),
                    timestamp: messages.len() as u64,
                    emotion: "neutral".to_string(),
                    importance: 5,
                });
            }
            Value::Object(map) => {
                for (_key, val) in map {
                    self.extract_generic_messages(val, messages, depth + 1);
                }
            }
            Value::Array(arr) => {
                for val in arr {
                    self.extract_generic_messages(val, messages, depth + 1);
                }
            }
            _ => {}
        }
    }

    /// Extract unique participants from messages
    fn extract_participants(&self, messages: &[Message]) -> Vec<String> {
        let mut participants = Vec::new();
        for msg in messages {
            if !participants.contains(&msg.role) {
                participants.push(msg.role.clone());
            }
        }
        participants
    }

    /// Detect emotion from message (simple heuristic)
    fn detect_emotion(&self, _msg: &Value) -> String {
        // TODO: Implement actual emotion detection
        "neutral".to_string()
    }

    /// Calculate message importance (1-10)
    fn calculate_importance(&self, msg: &Value) -> u8 {
        // Simple heuristic based on length
        if let Some(content) = msg.get("content").and_then(|c| c.as_str()) {
            let len = content.len();
            if len > 500 {
                8
            } else if len > 200 {
                6
            } else if len > 50 {
                5
            } else {
                3
            }
        } else {
            5
        }
    }

    /// Default conversation patterns
    fn default_patterns() -> Vec<ConversationPattern> {
        vec![
            ConversationPattern {
                name: "OpenAI".to_string(),
                message_path: vec!["messages".to_string()],
                content_field: "content".to_string(),
                role_field: "role".to_string(),
            },
            ConversationPattern {
                name: "Claude".to_string(),
                message_path: vec!["conversation".to_string()],
                content_field: "text".to_string(),
                role_field: "sender".to_string(),
            },
        ]
    }
}

/// Conversation type enumeration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ConversationType {
    ChatGPT,
    Claude,
    Generic,
    MessageArray,
    Unknown,
}

impl ConversationType {
    fn as_str(&self) -> &str {
        match self {
            Self::ChatGPT => "chatgpt",
            Self::Claude => "claude",
            Self::Generic => "generic",
            Self::MessageArray => "array",
            Self::Unknown => "unknown",
        }
    }
}

impl std::fmt::Display for ConversationType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

/// Analyzed conversation structure
#[derive(Debug)]
pub struct ConversationAnalysis {
    pub conversation_type: ConversationType,
    pub messages: Vec<Message>,
    pub participants: Vec<String>,
    pub message_count: usize,
    pub metadata: Map<String, Value>,
}

/// Individual message in a conversation
#[derive(Debug, Clone)]
pub struct Message {
    pub content: String,
    pub role: String,
    pub timestamp: u64,
    pub emotion: String,
    pub importance: u8,
}

/// Known conversation pattern
#[derive(Debug, Clone)]
pub struct ConversationPattern {
    pub name: String,
    pub message_path: Vec<String>,
    pub content_field: String,
    pub role_field: String,
}

/// Conversation summary for listing
#[derive(Debug, Serialize)]
pub struct ConversationSummary {
    pub file_name: String,
    pub conversation_type: ConversationType,
    pub message_count: usize,
    pub participants: Vec<String>,
    pub timestamp: std::time::SystemTime,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_conversation_detection() {
        let analyzer = ConversationAnalyzer::new();

        // Test ChatGPT format
        let chatgpt_json = serde_json::json!({
            "messages": [
                {"role": "user", "content": "Hello"},
                {"role": "assistant", "content": "Hi there!"}
            ]
        });

        let analysis = analyzer.analyze(&chatgpt_json).unwrap();
        assert!(matches!(
            analysis.conversation_type,
            ConversationType::ChatGPT
        ));
        assert_eq!(analysis.message_count, 2);

        // Test array format
        let array_json = serde_json::json!([
            {"text": "Hello", "sender": "user"},
            {"text": "Hi!", "sender": "bot"}
        ]);

        let analysis = analyzer.analyze(&array_json).unwrap();
        assert!(matches!(
            analysis.conversation_type,
            ConversationType::MessageArray
        ));
    }
}
