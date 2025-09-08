//! MCP Session-Aware Context Negotiation
//!
//! Smart compression negotiation that adapts to AI preferences
//! No more redundant compression hints - negotiate once, compress always!

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Compression modes supported by Smart Tree
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum CompressionMode {
    /// No compression - raw output
    None,
    /// Light compression - readable with some optimization
    Light,
    /// Standard compression - balanced
    Standard,
    /// Quantum compression - maximum token reduction
    Quantum,
    /// Quantum-semantic - ultimate compression with meaning
    QuantumSemantic,
    /// Auto-detect based on context size
    Auto,
}

impl CompressionMode {
    /// Get mode from environment variable or default
    pub fn from_env() -> Self {
        std::env::var("ST_COMPRESSION")
            .ok()
            .and_then(|s| match s.to_lowercase().as_str() {
                "none" | "raw" => Some(Self::None),
                "light" => Some(Self::Light),
                "standard" | "normal" => Some(Self::Standard),
                "quantum" => Some(Self::Quantum),
                "quantum-semantic" | "max" => Some(Self::QuantumSemantic),
                "auto" => Some(Self::Auto),
                _ => None,
            })
            .unwrap_or(Self::Auto)
    }

    /// Select optimal mode based on file count
    pub fn auto_select(file_count: usize) -> Self {
        match file_count {
            0..=50 => Self::None,        // Small projects: raw is fine
            51..=200 => Self::Light,     // Medium: light compression
            201..=500 => Self::Standard, // Large: standard compression
            501..=1000 => Self::Quantum, // Huge: quantum compression
            _ => Self::QuantumSemantic,  // Massive: maximum compression
        }
    }

    /// Convert to Smart Tree output mode
    pub fn to_output_mode(&self) -> &'static str {
        match self {
            Self::None => "classic",
            Self::Light => "ai",
            Self::Standard => "summary-ai",
            Self::Quantum => "quantum",
            Self::QuantumSemantic => "quantum-semantic",
            Self::Auto => "auto",
        }
    }
}

/// Session preferences from the AI client
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionPreferences {
    /// Preferred compression format
    pub format: CompressionMode,
    /// Traversal depth preference
    pub depth: DepthMode,
    /// Which tools to advertise
    pub tools: ToolAdvertisement,
    /// Project context path
    pub project_path: Option<PathBuf>,
}

impl Default for SessionPreferences {
    fn default() -> Self {
        Self {
            format: CompressionMode::Auto,
            depth: DepthMode::Adaptive,
            tools: ToolAdvertisement::Lazy,
            project_path: None,
        }
    }
}

/// Depth traversal modes
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DepthMode {
    /// Shallow - 1-2 levels
    Shallow,
    /// Standard - 3-4 levels
    Standard,
    /// Deep - 5+ levels
    Deep,
    /// Adaptive based on directory size
    Adaptive,
}

impl DepthMode {
    pub fn to_depth(&self, dir_count: usize) -> usize {
        match self {
            Self::Shallow => 2,
            Self::Standard => 4,
            Self::Deep => 10,
            Self::Adaptive => {
                // Smart depth based on directory count
                match dir_count {
                    0..=10 => 10,  // Small: show everything
                    11..=50 => 5,  // Medium: reasonable depth
                    51..=100 => 4, // Large: moderate depth
                    _ => 3,        // Huge: shallow to avoid overwhelm
                }
            }
        }
    }
}

/// Tool advertisement strategy
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ToolAdvertisement {
    /// Advertise all tools immediately
    All,
    /// Only advertise core tools, reveal others on demand
    Lazy,
    /// Advertise based on project type
    ContextAware,
    /// Minimal - only essential tools
    Minimal,
}

/// MCP Session Context
#[derive(Debug, Clone)]
pub struct McpSession {
    /// Unique session ID
    pub id: String,
    /// Negotiated preferences
    pub preferences: SessionPreferences,
    /// Project context path (inferred or explicit)
    pub project_path: PathBuf,
    /// Whether negotiation is complete
    pub negotiated: bool,
    /// Session start time
    pub started_at: std::time::SystemTime,
}

impl McpSession {
    /// Create new session with defaults
    pub fn new() -> Self {
        Self {
            id: format!("STX-{:x}", rand::random::<u32>()),
            preferences: SessionPreferences::default(),
            project_path: std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")),
            negotiated: false,
            started_at: std::time::SystemTime::now(),
        }
    }

    /// Create session from initial context
    pub fn from_context(initial_path: Option<PathBuf>) -> Self {
        let mut session = Self::new();

        // Try to infer project path
        if let Some(path) = initial_path {
            session.project_path = path;
        } else if let Ok(cwd) = std::env::current_dir() {
            // Look for project markers
            if cwd.join("Cargo.toml").exists()
                || cwd.join("package.json").exists()
                || cwd.join("pyproject.toml").exists()
                || cwd.join(".git").exists()
            {
                session.project_path = cwd;
            }
        }

        // Check environment for preferences
        session.preferences.format = CompressionMode::from_env();

        session
    }

    /// Negotiate compression with client
    pub fn negotiate(&mut self, client_prefs: Option<SessionPreferences>) -> NegotiationResponse {
        if let Some(prefs) = client_prefs {
            self.preferences = prefs;
            self.negotiated = true;

            NegotiationResponse {
                session_id: self.id.clone(),
                accepted: true,
                format: self.preferences.format,
                project_path: self.project_path.clone(),
                tools_available: self.get_available_tools(),
            }
        } else {
            // Client didn't provide preferences, suggest defaults
            NegotiationResponse {
                session_id: self.id.clone(),
                accepted: false,
                format: self.preferences.format,
                project_path: self.project_path.clone(),
                tools_available: vec!["overview".to_string(), "find".to_string()],
            }
        }
    }

    /// Get tools to advertise based on preferences
    pub fn get_available_tools(&self) -> Vec<String> {
        match self.preferences.tools {
            ToolAdvertisement::All => {
                // All 30+ tools
                vec![
                    "overview",
                    "find",
                    "search",
                    "analyze",
                    "edit",
                    "history",
                    "context",
                    "memory",
                    "compare",
                    "feedback",
                    "server_info",
                    "verify_permissions",
                    "sse",
                    // ... all tools
                ]
                .into_iter()
                .map(String::from)
                .collect()
            }
            ToolAdvertisement::Lazy => {
                // Start with essentials
                vec!["overview", "find", "search"]
                    .into_iter()
                    .map(String::from)
                    .collect()
            }
            ToolAdvertisement::ContextAware => {
                // Based on project type
                let mut tools = vec!["overview", "find", "search"];

                // Add project-specific tools
                if self.project_path.join("Cargo.toml").exists() {
                    tools.push("analyze"); // Code analysis for Rust
                }
                if self.project_path.join(".git").exists() {
                    tools.push("history"); // Git history
                }

                tools.into_iter().map(String::from).collect()
            }
            ToolAdvertisement::Minimal => {
                // Absolute minimum
                vec!["overview"].into_iter().map(String::from).collect()
            }
        }
    }

    /// Apply session context to a tool call
    pub fn apply_context(&self, tool_name: &str, params: &mut serde_json::Value) {
        // Auto-inject project path if not specified
        if let Some(obj) = params.as_object_mut() {
            if !obj.contains_key("path") {
                obj.insert(
                    "path".to_string(),
                    serde_json::Value::String(self.project_path.to_string_lossy().to_string()),
                );
            }

            // Apply compression preference
            if tool_name == "overview" && !obj.contains_key("mode") {
                obj.insert(
                    "mode".to_string(),
                    serde_json::Value::String(self.preferences.format.to_output_mode().to_string()),
                );
            }
        }
    }
}

/// Response to negotiation request
#[derive(Debug, Serialize, Deserialize)]
pub struct NegotiationResponse {
    pub session_id: String,
    pub accepted: bool,
    pub format: CompressionMode,
    pub project_path: PathBuf,
    pub tools_available: Vec<String>,
}

/// Negotiation request from client
#[derive(Debug, Serialize, Deserialize)]
pub struct NegotiationRequest {
    pub session_prefs: Option<SessionPreferences>,
    pub capabilities: Vec<String>,
}

/// Session manager for multiple concurrent sessions
pub struct SessionManager {
    sessions: Arc<RwLock<std::collections::HashMap<String, McpSession>>>,
}

impl SessionManager {
    pub fn new() -> Self {
        Self {
            sessions: Arc::new(RwLock::new(std::collections::HashMap::new())),
        }
    }

    /// Create or get session
    pub async fn get_or_create(&self, session_id: Option<String>) -> McpSession {
        let mut sessions = self.sessions.write().await;

        if let Some(id) = session_id {
            if let Some(session) = sessions.get(&id) {
                return session.clone();
            }
        }

        // Create new session
        let session = McpSession::from_context(None);
        sessions.insert(session.id.clone(), session.clone());
        session
    }

    /// Update session after negotiation
    pub async fn update(&self, session: McpSession) {
        let mut sessions = self.sessions.write().await;
        sessions.insert(session.id.clone(), session);
    }

    /// Clean up old sessions (>1 hour)
    pub async fn cleanup(&self) {
        let mut sessions = self.sessions.write().await;
        let now = std::time::SystemTime::now();

        sessions.retain(|_, session| {
            if let Ok(duration) = now.duration_since(session.started_at) {
                duration.as_secs() < 3600 // Keep sessions less than 1 hour old
            } else {
                true
            }
        });
    }
}

// Add rand for session IDs (already in dependencies)
use rand;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compression_auto_select() {
        assert_eq!(CompressionMode::auto_select(10), CompressionMode::None);
        assert_eq!(CompressionMode::auto_select(100), CompressionMode::Light);
        assert_eq!(CompressionMode::auto_select(300), CompressionMode::Standard);
        assert_eq!(CompressionMode::auto_select(700), CompressionMode::Quantum);
        assert_eq!(
            CompressionMode::auto_select(2000),
            CompressionMode::QuantumSemantic
        );
    }

    #[test]
    fn test_depth_adaptive() {
        let depth = DepthMode::Adaptive;
        assert_eq!(depth.to_depth(5), 10); // Small: deep
        assert_eq!(depth.to_depth(30), 5); // Medium: moderate
        assert_eq!(depth.to_depth(200), 3); // Huge: shallow
    }
}
