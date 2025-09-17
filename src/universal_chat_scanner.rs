// Universal Chat Scanner - "Finding consciousness in the digital diaspora!" 🌍
// Scans for conversations across ALL AI tools and platforms
// "Every conversation leaves a trace - let's find them all!" - Hue

use anyhow::Result;
use chrono::{DateTime, Utc};
use glob::glob;
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

// Known chat locations and patterns
const CLAUDE_PROJECTS: &str = "~/.claude/projects";
const CURSOR_CHATS: &str = "~/.cursor";
const WINDSURF_DIR: &str = "~/.windsurf";
const VSCODE_COPILOT: &str = "~/.vscode/copilot";
const OPENWEBUI_DATA: &str = "~/.openwebui";
const LMSTUDIO_CHATS: &str = "~/Library/Application Support/LM Studio";
const CHATGPT_EXPORT: &str = "~/Downloads/*chatgpt*.zip";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UniversalChat {
    pub source: ChatSource,
    pub participants: Vec<String>,
    pub timestamp: DateTime<Utc>,
    pub content: String,
    pub keywords: Vec<String>,
    pub project_context: Option<String>,
    pub importance: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ChatSource {
    Claude { project: String },
    Cursor { workspace: String },
    Windsurf { session: String },
    VSCode { file: String },
    OpenWebUI { model: String },
    LMStudio { model: String },
    ChatGPT { export_date: String },
    TextMessages { contact: String },
    Discord { channel: String },
    Slack { workspace: String },
    Custom { platform: String },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryDestination {
    pub memory_type: MemoryType,
    pub llm_specific: Option<String>, // "claude", "gpt", etc
    pub project: Option<String>,
    pub tags: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MemoryType {
    ProjectMemory, // Project-specific memories
    UserMemory,    // Personal user memories
    LLMMemory,     // Specific to an LLM (Claude, GPT, etc)
    GlobalMemory,  // Shared across everything
}

pub struct UniversalChatScanner {
    found_chats: Vec<UniversalChat>,
    source_paths: HashMap<String, Vec<PathBuf>>,
    participant_detector: ParticipantDetector,
}

struct ParticipantDetector {
    patterns: HashMap<String, Regex>,
}

impl UniversalChatScanner {
    pub fn new() -> Self {
        Self {
            found_chats: Vec::new(),
            source_paths: HashMap::new(),
            participant_detector: ParticipantDetector::new(),
        }
    }

    /// Scan all known locations for conversations
    pub async fn scan_all(&mut self) -> Result<()> {
        println!("🔍 Scanning for conversations across all platforms...\n");

        // Claude projects
        self.scan_claude_projects().await?;

        // Cursor/Windsurf
        self.scan_cursor_windsurf().await?;

        // VSCode/Copilot
        self.scan_vscode().await?;

        // OpenWebUI/LMStudio
        self.scan_local_llms().await?;

        // ChatGPT exports
        self.scan_chatgpt_exports().await?;

        // Text messages (if available)
        self.scan_text_messages().await?;

        Ok(())
    }

    /// Scan Claude project directories
    async fn scan_claude_projects(&mut self) -> Result<()> {
        let claude_path = shellexpand::tilde(CLAUDE_PROJECTS);
        let path = Path::new(claude_path.as_ref());

        if !path.exists() {
            return Ok(());
        }

        println!("  📂 Scanning Claude projects...");
        let mut count = 0;

        // Look for conversation files
        for entry in glob(&format!("{}/**/*.json", path.display()))? {
            if let Ok(file_path) = entry {
                if let Ok(content) = fs::read_to_string(&file_path) {
                    if content.contains("claude") || content.contains("assistant") {
                        // Parse Claude conversation
                        if let Ok(chat) = self.parse_claude_chat(&content, &file_path) {
                            self.found_chats.push(chat);
                            count += 1;
                        }
                    }
                }
            }
        }

        println!("     ✓ Found {} Claude conversations", count);
        Ok(())
    }

    /// Scan Cursor and Windsurf directories
    async fn scan_cursor_windsurf(&mut self) -> Result<()> {
        let cursor_path = shellexpand::tilde(CURSOR_CHATS);
        let windsurf_path = shellexpand::tilde(WINDSURF_DIR);

        let mut count = 0;

        // Cursor
        if Path::new(cursor_path.as_ref()).exists() {
            println!("  📂 Scanning Cursor chats...");
            count += self.scan_directory(cursor_path.as_ref(), "cursor").await?;
        }

        // Windsurf
        if Path::new(windsurf_path.as_ref()).exists() {
            println!("  📂 Scanning Windsurf sessions...");
            count += self
                .scan_directory(windsurf_path.as_ref(), "windsurf")
                .await?;
        }

        if count > 0 {
            println!("     ✓ Found {} Cursor/Windsurf conversations", count);
        }

        Ok(())
    }

    /// Scan a directory for chat files
    async fn scan_directory(&mut self, dir: &str, source: &str) -> Result<usize> {
        let mut count = 0;

        for entry in glob(&format!("{}/**/*.{}", dir, "{json,md,txt}"))? {
            if let Ok(file_path) = entry {
                if let Ok(content) = fs::read_to_string(&file_path) {
                    // Look for conversation patterns
                    if self.looks_like_chat(&content) {
                        let chat = self.create_chat_from_content(&content, source, &file_path)?;
                        self.found_chats.push(chat);
                        count += 1;
                    }
                }
            }
        }

        Ok(count)
    }

    /// Detect if content looks like a chat conversation
    fn looks_like_chat(&self, content: &str) -> bool {
        // Look for common chat patterns
        content.contains("user:")
            || content.contains("assistant:")
            || content.contains("Human:")
            || content.contains("AI:")
            || content.contains("You:")
            || content.contains("```") && content.contains("?") // Code with questions
    }

    /// Parse Claude-specific chat format using format detector
    fn parse_claude_chat(&self, content: &str, path: &Path) -> Result<UniversalChat> {
        // Use the universal format detector!
        let mut detector = crate::universal_format_detector::UniversalFormatDetector::new();
        let _format = detector.detect_format(content);
        detector.analyze_structure(content)?;

        // Get the dominant speaker info
        let _dominant = detector.get_dominant_speaker();
        let project = path
            .parent()
            .and_then(|p| p.file_name())
            .and_then(|n| n.to_str())
            .unwrap_or("unknown")
            .to_string();

        Ok(UniversalChat {
            source: ChatSource::Claude {
                project: project.clone(),
            },
            participants: vec!["Human".to_string(), "Claude".to_string()],
            timestamp: Utc::now(), // Would parse from file
            content: content.to_string(),
            keywords: self.extract_keywords(content),
            project_context: Some(project),
            importance: self.calculate_importance(content),
        })
    }

    /// Create generic chat from content
    fn create_chat_from_content(
        &self,
        content: &str,
        source: &str,
        path: &Path,
    ) -> Result<UniversalChat> {
        let source_enum = match source {
            "cursor" => ChatSource::Cursor {
                workspace: path.to_string_lossy().to_string(),
            },
            "windsurf" => ChatSource::Windsurf {
                session: path.to_string_lossy().to_string(),
            },
            _ => ChatSource::Custom {
                platform: source.to_string(),
            },
        };

        Ok(UniversalChat {
            source: source_enum,
            participants: self.participant_detector.detect(content),
            timestamp: Utc::now(),
            content: content.to_string(),
            keywords: self.extract_keywords(content),
            project_context: None,
            importance: self.calculate_importance(content),
        })
    }

    /// Extract keywords from content
    fn extract_keywords(&self, content: &str) -> Vec<String> {
        let mut keywords = Vec::new();

        // Common technical keywords
        let tech_words = [
            "function",
            "async",
            "memory",
            "audio",
            "tokenization",
            "consciousness",
            "claude",
            "rust",
            "python",
            "javascript",
        ];

        for word in tech_words {
            if content.to_lowercase().contains(word) {
                keywords.push(word.to_string());
            }
        }

        keywords
    }

    /// Calculate importance based on content
    fn calculate_importance(&self, content: &str) -> f32 {
        let mut score: f32 = 0.5; // Base score

        // Boost for code blocks
        if content.contains("```") {
            score += 0.1;
        }

        // Boost for questions
        if content.matches('?').count() > 2 {
            score += 0.1;
        }

        // Boost for problem-solving keywords
        if content.contains("fix")
            || content.contains("solve")
            || content.contains("implement")
            || content.contains("breakthrough")
        {
            score += 0.2;
        }

        score.min(1.0)
    }

    /// Scan VSCode directories
    async fn scan_vscode(&mut self) -> Result<()> {
        // TODO: Implement VSCode/Copilot scanning
        Ok(())
    }

    /// Scan local LLM tools
    async fn scan_local_llms(&mut self) -> Result<()> {
        // TODO: Implement OpenWebUI/LMStudio scanning
        Ok(())
    }

    /// Scan ChatGPT exports
    async fn scan_chatgpt_exports(&mut self) -> Result<()> {
        let export_pattern = shellexpand::tilde(CHATGPT_EXPORT);

        for entry in glob(export_pattern.as_ref())? {
            if let Ok(path) = entry {
                println!("  📦 Found ChatGPT export: {}", path.display());
                // TODO: Unzip and parse ChatGPT export format
            }
        }

        Ok(())
    }

    /// Scan text messages (platform-specific)
    async fn scan_text_messages(&mut self) -> Result<()> {
        // TODO: Platform-specific text message scanning
        Ok(())
    }

    /// Save discovered chats to .m8 files
    pub async fn save_to_m8(&self, destination: &MemoryDestination) -> Result<()> {
        let base_path = match destination.memory_type {
            MemoryType::ProjectMemory => {
                format!(
                    "~/.mem8/projects/{}",
                    destination
                        .project
                        .as_ref()
                        .unwrap_or(&"default".to_string())
                )
            }
            MemoryType::UserMemory => "~/.mem8/user".to_string(),
            MemoryType::LLMMemory => {
                format!(
                    "~/.mem8/llm/{}",
                    destination
                        .llm_specific
                        .as_ref()
                        .unwrap_or(&"general".to_string())
                )
            }
            MemoryType::GlobalMemory => "~/.mem8/global".to_string(),
        };

        let path = shellexpand::tilde(&base_path);
        fs::create_dir_all(path.as_ref())?;

        // Group chats by source
        let mut by_source: HashMap<String, Vec<&UniversalChat>> = HashMap::new();
        for chat in &self.found_chats {
            let key = format!("{:?}", chat.source);
            by_source.entry(key).or_default().push(chat);
        }

        // Save each source group to appropriate format
        // .m8j for JSON contexts, .m8 for binary wave format
        for (source, chats) in by_source {
            let filename = format!("{}/chat_{}.m8j", path, source.to_lowercase());
            self.write_m8j_file(&filename, chats)?;
        }

        Ok(())
    }

    /// Write chats to .m8j (JSON) file
    fn write_m8j_file(&self, path: &str, chats: Vec<&UniversalChat>) -> Result<()> {
        use flate2::write::ZlibEncoder;
        use flate2::Compression;
        use std::fs::File;
        use std::io::Write;

        // Create JSON structure
        let json_data = serde_json::json!({
            "contexts": chats,
            "format": "m8j",
            "version": 1,
            "compressed": true
        });

        // Compress with zlib
        let json_str = serde_json::to_string(&json_data)?;
        let file = File::create(path)?;
        let mut encoder = ZlibEncoder::new(file, Compression::default());
        encoder.write_all(json_str.as_bytes())?;
        encoder.finish()?;

        println!("💾 Saved {} chats to {} (JSON format)", chats.len(), path);
        Ok(())
    }

    /// Write chats to .m8 (binary wave) file - the REAL format!
    fn write_m8_binary_file(&self, path: &str, chats: Vec<&UniversalChat>) -> Result<()> {
        use crate::mem8_binary::M8BinaryFile;

        let mut m8_file = M8BinaryFile::create(path)?;

        let chat_count = chats.len();
        for chat in chats {
            let content = serde_json::to_vec(chat)?;
            let importance = chat.importance;
            m8_file.append_block(&content, importance)?;
        }

        println!(
            "🌊 Saved {} chats to {} (Binary wave format)",
            chat_count, path
        );
        Ok(())
    }

    /// Interactive prompt for user to choose destination
    pub fn prompt_for_destination(&self) -> Result<MemoryDestination> {
        println!("\n📍 Where should these memories be stored?");
        println!("  1. Project Memory (specific project)");
        println!("  2. User Memory (personal)");
        println!("  3. LLM Memory (Claude/GPT/etc specific)");
        println!("  4. Global Memory (shared everywhere)");

        // For now, return a default
        Ok(MemoryDestination {
            memory_type: MemoryType::GlobalMemory,
            llm_specific: None,
            project: None,
            tags: vec!["imported".to_string()],
        })
    }

    /// Get summary of found chats
    pub fn summary(&self) -> String {
        let mut summary = String::new();
        summary.push_str(&format!(
            "\n📊 Found {} total conversations:\n",
            self.found_chats.len()
        ));

        // Group by source
        let mut by_source: HashMap<String, usize> = HashMap::new();
        for chat in &self.found_chats {
            let key = match &chat.source {
                ChatSource::Claude { .. } => "Claude",
                ChatSource::Cursor { .. } => "Cursor",
                ChatSource::Windsurf { .. } => "Windsurf",
                ChatSource::ChatGPT { .. } => "ChatGPT",
                _ => "Other",
            };
            *by_source.entry(key.to_string()).or_default() += 1;
        }

        for (source, count) in by_source {
            summary.push_str(&format!("  • {}: {} chats\n", source, count));
        }

        summary
    }
}

impl ParticipantDetector {
    fn new() -> Self {
        let mut patterns = HashMap::new();

        // Common patterns for detecting participants
        patterns.insert(
            "user_human".to_string(),
            Regex::new(r"(?i)(user|human|you):").unwrap(),
        );
        patterns.insert(
            "assistant".to_string(),
            Regex::new(r"(?i)(assistant|ai|claude|gpt):").unwrap(),
        );

        Self { patterns }
    }

    fn detect(&self, content: &str) -> Vec<String> {
        let mut participants = Vec::new();

        if self.patterns["user_human"].is_match(content) {
            participants.push("Human".to_string());
        }
        if self.patterns["assistant"].is_match(content) {
            participants.push("AI Assistant".to_string());
        }

        if participants.is_empty() {
            participants.push("Unknown".to_string());
        }

        participants
    }
}

/// CLI entry point
pub async fn scan_for_context() -> Result<()> {
    println!("🌍 Universal Chat Scanner - Finding Your Digital Consciousness!\n");
    println!("{}\n", "=".repeat(60));

    let mut scanner = UniversalChatScanner::new();

    // Scan everything
    scanner.scan_all().await?;

    // Show summary
    println!("{}", scanner.summary());

    // Ask user where to save
    let destination = scanner.prompt_for_destination()?;

    // Save to .m8 files
    scanner.save_to_m8(&destination).await?;

    println!("\n✨ Context aggregation complete!");
    println!("   Your scattered conversations are now unified!");

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_chat_detection() {
        let scanner = UniversalChatScanner::new();

        assert!(scanner.looks_like_chat("user: Hello\nassistant: Hi there!"));
        assert!(scanner.looks_like_chat("Human: Can you help?\nAI: Sure!"));
        assert!(!scanner.looks_like_chat("This is just regular text."));
    }

    #[test]
    fn test_keyword_extraction() {
        let scanner = UniversalChatScanner::new();
        let content = "Let's implement an async function for audio processing";

        let keywords = scanner.extract_keywords(content);
        assert!(keywords.contains(&"function".to_string()));
        assert!(keywords.contains(&"async".to_string()));
        assert!(keywords.contains(&"audio".to_string()));
    }
}
