//! Context Gathering System for Smart Tree
//!
//! This module searches across AI tool directories (~/.claude, ~/.windsurf, ~/.cursor, etc.)
//! to gather project-related context and convert it into M8 format for processing.

pub mod collab_session;
pub mod cross_session;
pub mod partnership;
pub mod temporal;

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;
// TODO: Use proper M8 wave format when available
// use crate::mem8::wave::{MemoryWave, WaveGrid, FrequencyBand, SensorType};
// use crate::mem8::format::M8Writer;

/// AI tool directories to search for context
pub const AI_TOOL_DIRS: &[&str] = &[
    ".claude",
    ".windsurf",
    ".cursor",
    ".continue",
    ".github/copilot",
    ".vscode",
    ".idea",
    ".zed",
];

/// File extensions that contain context information
pub const CONTEXT_EXTENSIONS: &[&str] = &[
    "json", "jsonl", "xml", "yaml", "yml", "toml", "md", "txt", "log", "conf", "config", "env",
    "settings",
];

/// Context gathering configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GatherConfig {
    /// Directories to search (relative to home)
    pub search_dirs: Vec<String>,
    /// Additional custom directories
    pub custom_dirs: Vec<PathBuf>,
    /// File extensions to include
    pub extensions: Vec<String>,
    /// Project identifiers to look for
    pub project_identifiers: Vec<String>,
    /// Maximum file size to process (in bytes)
    pub max_file_size: usize,
    /// Enable recursive search in subdirectories
    pub recursive: bool,
    /// Privacy mode - redact sensitive information
    pub privacy_mode: bool,
}

impl Default for GatherConfig {
    fn default() -> Self {
        Self {
            search_dirs: AI_TOOL_DIRS.iter().map(|s| s.to_string()).collect(),
            custom_dirs: vec![],
            extensions: CONTEXT_EXTENSIONS.iter().map(|s| s.to_string()).collect(),
            project_identifiers: vec![],
            max_file_size: 10 * 1024 * 1024, // 10MB
            recursive: true,
            privacy_mode: true,
        }
    }
}

/// Represents gathered context from various sources
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GatheredContext {
    pub source_path: PathBuf,
    pub ai_tool: String,
    pub content_type: ContextType,
    pub content: ContextContent,
    pub metadata: HashMap<String, String>,
    pub relevance_score: f32,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum ContextType {
    ChatHistory,
    ProjectSettings,
    CodeSnippets,
    Documentation,
    Configuration,
    SearchHistory,
    Bookmarks,
    CustomPrompts,
    ModelPreferences,
    WorkspaceState,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ContextContent {
    Text(String),
    Json(serde_json::Value),
    Xml(String),
    Binary(Vec<u8>),
}

/// Main context gatherer
pub struct ContextGatherer {
    config: GatherConfig,
    project_path: PathBuf,
    gathered_contexts: Vec<GatheredContext>,
    session_tracker: collab_session::CollaborativeSessionTracker,
    cross_session_bridge: cross_session::CrossSessionBridge,
}

impl ContextGatherer {
    pub fn new(project_path: PathBuf, config: GatherConfig) -> Self {
        Self {
            config,
            project_path,
            gathered_contexts: Vec::new(),
            session_tracker: collab_session::CollaborativeSessionTracker::new(),
            cross_session_bridge: cross_session::CrossSessionBridge::new(),
        }
    }

    /// Gather context from all configured sources
    pub fn gather_all(&mut self) -> Result<()> {
        let home_dir = dirs::home_dir().context("Failed to get home directory")?;

        // Clone to avoid borrow issues
        let search_dirs = self.config.search_dirs.clone();
        let custom_dirs = self.config.custom_dirs.clone();

        // Search AI tool directories
        for dir_name in search_dirs {
            let search_path = home_dir.join(&dir_name);
            if search_path.exists() {
                println!("🔍 Scanning {}", search_path.display());
                self.scan_directory(&search_path, &dir_name)?;
            }
        }

        // Search custom directories
        for custom_dir in custom_dirs {
            if custom_dir.exists() {
                println!("🔍 Scanning custom: {}", custom_dir.display());
                let tool_name = custom_dir
                    .file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or("custom");
                self.scan_directory(&custom_dir, tool_name)?;
            }
        }

        // Post-process and score relevance
        self.score_relevance();

        // Analyze for cross-domain patterns
        let patterns = self
            .cross_session_bridge
            .analyze_for_patterns(&self.gathered_contexts);
        if !patterns.is_empty() {
            println!("🔗 Found {} cross-domain patterns", patterns.len());
        }

        // Generate insights if we have enough data
        let insights = self.cross_session_bridge.generate_insights(0.3);
        if !insights.is_empty() {
            println!("💡 Generated {} cross-session insights", insights.len());
        }

        Ok(())
    }

    /// Scan a directory for context files
    fn scan_directory(&mut self, path: &Path, ai_tool: &str) -> Result<()> {
        let walker = if self.config.recursive {
            WalkDir::new(path).max_depth(5)
        } else {
            WalkDir::new(path).max_depth(1)
        };

        for entry in walker.into_iter().filter_map(|e| e.ok()) {
            let path = entry.path();

            // Skip if not a file
            if !path.is_file() {
                continue;
            }

            // Check file extension
            if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
                if !self.config.extensions.contains(&ext.to_string()) {
                    continue;
                }

                // Check file size
                if let Ok(metadata) = fs::metadata(path) {
                    if metadata.len() as usize > self.config.max_file_size {
                        continue;
                    }
                }

                // Process the file
                if let Ok(context) = self.process_file(path, ai_tool) {
                    if self.is_relevant(&context) {
                        // Track collaborative sessions
                        let _ = self.session_tracker.process_context(&context);
                        self.gathered_contexts.push(context);
                    }
                }
            }
        }

        Ok(())
    }

    /// Process a single file and extract context
    fn process_file(&self, path: &Path, ai_tool: &str) -> Result<GatheredContext> {
        let content = fs::read_to_string(path).context("Failed to read file")?;

        let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("");

        let (content_type, content) = match ext {
            "json" => self.parse_json(&content, path)?,
            "jsonl" => self.parse_jsonl(&content, path)?,
            "xml" => self.parse_xml(&content)?,
            "yaml" | "yml" => self.parse_yaml(&content)?,
            "md" => (ContextType::Documentation, ContextContent::Text(content)),
            _ => (ContextType::Configuration, ContextContent::Text(content)),
        };

        Ok(GatheredContext {
            source_path: path.to_path_buf(),
            ai_tool: ai_tool.to_string(),
            content_type,
            content,
            metadata: self.extract_metadata(path),
            relevance_score: 0.0, // Will be calculated later
            timestamp: chrono::Utc::now(),
        })
    }

    /// Parse JSON content and determine its type
    fn parse_json(&self, content: &str, path: &Path) -> Result<(ContextType, ContextContent)> {
        let json: serde_json::Value = serde_json::from_str(content)?;

        // Detect content type based on structure and filename
        let content_type = if path.to_str().unwrap_or("").contains("chat") {
            ContextType::ChatHistory
        } else if path.to_str().unwrap_or("").contains("settings") {
            ContextType::ProjectSettings
        } else if json.get("messages").is_some() {
            ContextType::ChatHistory
        } else if json.get("workspace").is_some() {
            ContextType::WorkspaceState
        } else {
            ContextType::Configuration
        };

        // Apply privacy redaction if needed
        let json = if self.config.privacy_mode {
            self.redact_sensitive_json(json)
        } else {
            json
        };

        Ok((content_type, ContextContent::Json(json)))
    }

    /// Parse JSONL (JSON Lines) format
    fn parse_jsonl(&self, content: &str, path: &Path) -> Result<(ContextType, ContextContent)> {
        let mut lines = Vec::new();

        for line in content.lines() {
            if let Ok(json) = serde_json::from_str::<serde_json::Value>(line) {
                lines.push(json);
            }
        }

        let content_type = if path.to_str().unwrap_or("").contains("history") {
            ContextType::SearchHistory
        } else {
            ContextType::ChatHistory
        };

        Ok((
            content_type,
            ContextContent::Json(serde_json::Value::Array(lines)),
        ))
    }

    /// Parse XML content
    fn parse_xml(&self, content: &str) -> Result<(ContextType, ContextContent)> {
        // For now, store as text - could add proper XML parsing later
        Ok((
            ContextType::Configuration,
            ContextContent::Xml(content.to_string()),
        ))
    }

    /// Parse YAML content
    fn parse_yaml(&self, content: &str) -> Result<(ContextType, ContextContent)> {
        let yaml: serde_yaml::Value = serde_yaml::from_str(content)?;
        let json = serde_json::to_value(yaml)?;
        Ok((ContextType::Configuration, ContextContent::Json(json)))
    }

    /// Check if context is relevant to the current project
    fn is_relevant(&self, context: &GatheredContext) -> bool {
        let project_name = self
            .project_path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("");

        // Check content for project references
        let content_str = match &context.content {
            ContextContent::Text(t) => t.clone(),
            ContextContent::Json(j) => j.to_string(),
            ContextContent::Xml(x) => x.clone(),
            ContextContent::Binary(_) => return false,
        };

        // Look for project name or identifiers
        if content_str
            .to_lowercase()
            .contains(&project_name.to_lowercase())
        {
            return true;
        }

        for identifier in &self.config.project_identifiers {
            if content_str.contains(identifier) {
                return true;
            }
        }

        // Check path references
        let project_path_str = self.project_path.to_string_lossy();
        if content_str.contains(&project_path_str.as_ref()) {
            return true;
        }

        false
    }

    /// Score relevance of gathered contexts
    fn score_relevance(&mut self) {
        let project_name = self
            .project_path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("")
            .to_lowercase();

        for context in &mut self.gathered_contexts {
            let mut score = 0.0;

            // Score based on content type
            score += match context.content_type {
                ContextType::ChatHistory => 0.8,
                ContextType::ProjectSettings => 0.9,
                ContextType::CodeSnippets => 0.7,
                ContextType::Documentation => 0.6,
                ContextType::CustomPrompts => 0.8,
                _ => 0.5,
            };

            // Score based on recency
            let age_days = (chrono::Utc::now() - context.timestamp).num_days();
            if age_days < 7 {
                score += 0.3;
            } else if age_days < 30 {
                score += 0.2;
            } else if age_days < 90 {
                score += 0.1;
            }

            // Score based on project name mentions
            let content_str = match &context.content {
                ContextContent::Text(t) => t.clone(),
                ContextContent::Json(j) => j.to_string(),
                ContextContent::Xml(x) => x.clone(),
                ContextContent::Binary(_) => String::new(),
            };

            let mentions = content_str.to_lowercase().matches(&project_name).count();
            score += (mentions as f32 * 0.1).min(0.5);

            context.relevance_score = score.min(1.0);
        }

        // Sort by relevance
        self.gathered_contexts
            .sort_by(|a, b| b.relevance_score.partial_cmp(&a.relevance_score).unwrap());
    }

    /// Extract metadata from file path
    fn extract_metadata(&self, path: &Path) -> HashMap<String, String> {
        let mut metadata = HashMap::new();

        if let Some(parent) = path.parent() {
            metadata.insert(
                "parent_dir".to_string(),
                parent.to_string_lossy().to_string(),
            );
        }

        if let Ok(file_metadata) = fs::metadata(path) {
            if let Ok(modified) = file_metadata.modified() {
                metadata.insert(
                    "modified".to_string(),
                    chrono::DateTime::<chrono::Utc>::from(modified).to_rfc3339(),
                );
            }
            metadata.insert("size".to_string(), file_metadata.len().to_string());
        }

        metadata
    }

    /// Redact sensitive information from JSON
    fn redact_sensitive_json(&self, mut json: serde_json::Value) -> serde_json::Value {
        if let Some(obj) = json.as_object_mut() {
            for (key, value) in obj.iter_mut() {
                if key.contains("key")
                    || key.contains("token")
                    || key.contains("secret")
                    || key.contains("password")
                {
                    *value = serde_json::Value::String("[REDACTED]".to_string());
                } else if value.is_object() || value.is_array() {
                    *value = self.redact_sensitive_json(value.clone());
                }
            }
        } else if let Some(arr) = json.as_array_mut() {
            for value in arr.iter_mut() {
                *value = self.redact_sensitive_json(value.clone());
            }
        }

        json
    }

    /// Convert gathered contexts to M8 format
    pub fn to_m8(&self) -> Result<Vec<u8>> {
        // For now, create a simple JSON representation
        // TODO: Implement proper M8 wave-based format
        let m8_data = serde_json::json!({
            "version": "1.0",
            "type": "context_gather",
            "metadata": {
                "project_path": self.project_path,
                "total_contexts": self.gathered_contexts.len(),
                "timestamp": chrono::Utc::now().to_rfc3339(),
                "top_sources": self.get_top_sources(),
            },
            "contexts": self.gathered_contexts.iter().map(|c| {
                serde_json::json!({
                    "path": c.source_path.to_string_lossy(),
                    "tool": c.ai_tool,
                    "type": format!("{:?}", c.content_type),
                    "score": c.relevance_score,
                    "preview": match &c.content {
                        ContextContent::Text(t) => t.chars().take(100).collect::<String>(),
                        ContextContent::Json(j) => j.to_string().chars().take(100).collect::<String>(),
                        ContextContent::Xml(x) => x.chars().take(100).collect::<String>(),
                        ContextContent::Binary(b) => format!("[Binary: {} bytes]", b.len()),
                    }
                })
            }).collect::<Vec<_>>()
        });

        // Compress with zlib for efficiency
        use flate2::write::ZlibEncoder;
        use flate2::Compression;
        use std::io::Write;

        let json_bytes = serde_json::to_vec(&m8_data)?;
        let mut encoder = ZlibEncoder::new(Vec::new(), Compression::default());
        encoder.write_all(&json_bytes)?;
        encoder.finish().map_err(Into::into)
    }

    /// Get summary of top context sources
    fn get_top_sources(&self) -> HashMap<String, usize> {
        let mut sources = HashMap::new();

        for context in &self.gathered_contexts {
            *sources.entry(context.ai_tool.clone()).or_insert(0) += 1;
        }

        sources
    }

    /// Get gathered contexts
    pub fn contexts(&self) -> &[GatheredContext] {
        &self.gathered_contexts
    }

    /// Save gathered contexts to JSON file
    pub fn save_json(&self, path: &Path) -> Result<()> {
        let json = serde_json::to_string_pretty(&self.gathered_contexts)?;
        fs::write(path, json)?;
        Ok(())
    }

    /// Perform temporal analysis on gathered contexts
    pub fn analyze_temporal(
        &self,
        resolution: temporal::TemporalResolution,
    ) -> temporal::TemporalPatterns {
        let analyzer =
            temporal::TemporalContextAnalyzer::new(self.gathered_contexts.clone(), resolution);
        analyzer.detect_patterns()
    }

    /// Get temporal wave grid representation
    pub fn create_temporal_waves(
        &self,
        resolution: temporal::TemporalResolution,
    ) -> temporal::TemporalWaveGrid {
        let analyzer =
            temporal::TemporalContextAnalyzer::new(self.gathered_contexts.clone(), resolution);
        analyzer.create_temporal_waves()
    }

    /// Apply temporal decay to relevance scores
    pub fn apply_temporal_decay(&mut self, half_life_days: f32) {
        let mut analyzer = temporal::TemporalContextAnalyzer::new(
            self.gathered_contexts.clone(),
            temporal::TemporalResolution::Day,
        );
        analyzer.apply_temporal_decay(half_life_days);

        // Update our contexts with decayed scores
        self.gathered_contexts = analyzer.contexts;
    }

    /// Analyze AI-human partnership patterns
    pub fn analyze_partnership(&self) -> partnership::PartnershipAnalysis {
        let analyzer = partnership::PartnershipAnalyzer::new(self.gathered_contexts.clone());
        analyzer.analyze_partnership()
    }

    /// Get collaborative session tracker
    pub fn session_tracker(&self) -> &collab_session::CollaborativeSessionTracker {
        &self.session_tracker
    }

    /// Get mutable collaborative session tracker
    pub fn session_tracker_mut(&mut self) -> &mut collab_session::CollaborativeSessionTracker {
        &mut self.session_tracker
    }

    /// Anchor an important collaborative memory
    pub fn anchor_memory(
        &mut self,
        origin: collab_session::CollaborativeOrigin,
        anchor_type: collab_session::AnchorType,
        context: String,
        keywords: Vec<String>,
    ) -> Result<String> {
        self.session_tracker
            .anchor_memory(origin, anchor_type, context, keywords)
    }

    /// Find memories related to keywords
    pub fn find_relevant_memories(&self, keywords: &[String]) -> Vec<String> {
        self.session_tracker
            .find_relevant_anchors(keywords)
            .into_iter()
            .map(|anchor| {
                format!(
                    "[{}] {}: {} (keywords: {})",
                    anchor.timestamp.format("%Y-%m-%d"),
                    match &anchor.anchor_type {
                        collab_session::AnchorType::PatternInsight => "Pattern",
                        collab_session::AnchorType::Solution => "Solution",
                        collab_session::AnchorType::Breakthrough => "Breakthrough",
                        collab_session::AnchorType::LearningMoment => "Learning",
                        collab_session::AnchorType::SharedJoke => "Joke",
                        collab_session::AnchorType::TechnicalPattern => "Tech Pattern",
                        collab_session::AnchorType::ProcessImprovement => "Process",
                    },
                    anchor.context,
                    anchor.keywords.join(", ")
                )
            })
            .collect()
    }

    /// Get co-engagement heatmap
    pub fn get_co_engagement_heatmap(&self) -> collab_session::CoEngagementHeatmap {
        let sessions: Vec<_> = self
            .session_tracker
            .session_history
            .iter()
            .cloned()
            .collect();
        collab_session::CoEngagementHeatmap::from_sessions(&sessions)
    }

    /// Get cross-domain patterns
    pub fn get_cross_domain_patterns(&self) -> Vec<&cross_session::CrossDomainPattern> {
        self.cross_session_bridge.get_patterns()
    }

    /// Get relevant insights for current project
    pub fn get_relevant_insights(
        &self,
        keywords: &[String],
    ) -> Vec<cross_session::CrossSessionInsight> {
        self.cross_session_bridge
            .suggest_relevant_insights(&self.project_path, keywords)
    }

    /// Invite a persona for consultation
    pub fn invite_persona(
        &self,
        context: &str,
        duration: u32,
    ) -> Option<cross_session::PersonaInvitation> {
        self.cross_session_bridge.invite_persona(context, duration)
    }
}

// TODO: Implement M8Writer extension when M8 format is available
