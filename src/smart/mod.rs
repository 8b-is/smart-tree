//! 🧠 Smart Tools - Revolutionary AI-Driven Context-Aware Features
//!
//! This module contains the next-generation Smart Tools that provide
//! context-aware, AI-optimized functionality with 70-90% token reduction.
//!
//! ## Smart Tools Overview:
//! - **SmartRead**: Context-aware file reading focusing on relevant sections
//! - **SemanticEdit**: Intent-based code modifications understanding code intent
//! - **SmartLS**: Task-aware directory listings prioritizing relevant files
//! - **Unified Search**: Natural language queries replacing multiple tool calls
//!
//! These tools work together to drastically improve AI-human collaboration
//! by reducing typical 10+ tool workflows to just 3 smart calls while
//! improving accuracy and reducing token usage.

use serde::{Deserialize, Serialize};

// 📦 Smart Tools Modules
pub mod context; // Context analysis engine
pub mod git_relay; // 🔄 Smart Git CLI integration with compression
pub mod nlp; // Natural language processing
pub mod relevance; // Advanced relevance scoring
pub mod smart_ls; // Task-aware directory listings
pub mod smart_read; // Context-aware file reading
pub mod unified_search; // Natural language search engine

// Re-export key types for convenience
pub use context::ContextAnalyzer;
pub use git_relay::{GitOperation, GitRelay, GitRelayResponse, GitResult};
pub use nlp::{ParsedQuery, QueryParser, SearchIntent};
pub use relevance::{ProjectContext, ProjectType, RelevanceEngine};
pub use smart_ls::{SmartDirEntry, SmartLS, SmartLSResponse};
pub use smart_read::{FileSection, SmartReadResponse, SmartReader};
pub use unified_search::{SearchResult, SearchResultType, UnifiedSearch, UnifiedSearchResponse};

/// 🎯 Core context analysis for understanding user intent and task focus
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskContext {
    /// Primary task description (e.g., "debugging authentication issues")
    pub task: String,
    /// Focus areas for this task
    pub focus_areas: Vec<FocusArea>,
    /// Relevance threshold (0.0-1.0)
    pub relevance_threshold: f32,
    /// Maximum results to return
    pub max_results: Option<usize>,
}

/// 🔍 Areas of focus for context-aware analysis
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum FocusArea {
    Authentication,
    API,
    Database,
    Frontend,
    Backend,
    Testing,
    Configuration,
    Security,
    Performance,
    Documentation,
    ErrorHandling,
    Logging,
    Deployment,
    Dependencies,
    Custom(String),
}

/// 📊 Relevance scoring for files/sections
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RelevanceScore {
    pub score: f32,
    pub reasons: Vec<String>,
    pub focus_matches: Vec<FocusArea>,
}

/// 🎭 Smart response with context awareness
#[derive(Debug, Serialize, Deserialize)]
pub struct SmartResponse<T> {
    /// Primary results (high relevance)
    pub primary: Vec<T>,
    /// Secondary results (medium relevance)  
    pub secondary: Vec<T>,
    /// Context summary
    pub context_summary: String,
    /// Token savings achieved
    pub token_savings: TokenSavings,
    /// Proactive suggestions
    pub suggestions: Vec<String>,
}

/// 💰 Token efficiency metrics
#[derive(Debug, Serialize, Deserialize)]
pub struct TokenSavings {
    /// Original token count (estimated)
    pub original_tokens: usize,
    /// Compressed token count (estimated)
    pub compressed_tokens: usize,
    /// Percentage saved
    pub percentage_saved: f32,
    /// Compression method used
    pub method: String,
}

impl TokenSavings {
    pub fn new(original: usize, compressed: usize, method: &str) -> Self {
        let percentage = if original > 0 {
            ((original - compressed) as f32 / original as f32) * 100.0
        } else {
            0.0
        };

        Self {
            original_tokens: original,
            compressed_tokens: compressed,
            percentage_saved: percentage,
            method: method.to_string(),
        }
    }
}

/// 🌟 Default implementations for common use cases
impl Default for TaskContext {
    fn default() -> Self {
        Self {
            task: "General development".to_string(),
            focus_areas: vec![FocusArea::API, FocusArea::Configuration],
            relevance_threshold: 0.6,
            max_results: Some(50),
        }
    }
}

impl FocusArea {
    /// Parse focus area from string
    pub fn from_str(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "auth" | "authentication" => FocusArea::Authentication,
            "api" => FocusArea::API,
            "db" | "database" => FocusArea::Database,
            "frontend" | "ui" => FocusArea::Frontend,
            "backend" | "server" => FocusArea::Backend,
            "test" | "testing" => FocusArea::Testing,
            "config" | "configuration" => FocusArea::Configuration,
            "security" | "sec" => FocusArea::Security,
            "performance" | "perf" => FocusArea::Performance,
            "docs" | "documentation" => FocusArea::Documentation,
            "error" | "errors" | "error_handling" => FocusArea::ErrorHandling,
            "logging" | "logs" => FocusArea::Logging,
            "deploy" | "deployment" => FocusArea::Deployment,
            "deps" | "dependencies" => FocusArea::Dependencies,
            _ => FocusArea::Custom(s.to_string()),
        }
    }

    /// Get keywords associated with this focus area
    pub fn keywords(&self) -> Vec<&'static str> {
        match self {
            FocusArea::Authentication => vec![
                "auth", "login", "password", "token", "session", "jwt", "oauth",
            ],
            FocusArea::API => vec![
                "api", "endpoint", "route", "handler", "request", "response", "http",
            ],
            FocusArea::Database => vec![
                "db",
                "database",
                "sql",
                "query",
                "table",
                "schema",
                "migration",
            ],
            FocusArea::Frontend => {
                vec!["ui", "component", "react", "vue", "angular", "html", "css"]
            }
            FocusArea::Backend => vec![
                "server",
                "service",
                "controller",
                "model",
                "business",
                "logic",
            ],
            FocusArea::Testing => vec![
                "test",
                "spec",
                "mock",
                "assert",
                "expect",
                "unit",
                "integration",
            ],
            FocusArea::Configuration => vec![
                "config",
                "env",
                "settings",
                "properties",
                "yaml",
                "json",
                "toml",
            ],
            FocusArea::Security => vec![
                "security",
                "vulnerability",
                "sanitize",
                "validate",
                "encrypt",
                "hash",
            ],
            FocusArea::Performance => vec![
                "performance",
                "optimize",
                "cache",
                "memory",
                "cpu",
                "benchmark",
            ],
            FocusArea::Documentation => vec![
                "doc",
                "readme",
                "comment",
                "documentation",
                "guide",
                "manual",
            ],
            FocusArea::ErrorHandling => vec![
                "error",
                "exception",
                "try",
                "catch",
                "panic",
                "result",
                "option",
            ],
            FocusArea::Logging => vec!["log", "logger", "debug", "info", "warn", "error", "trace"],
            FocusArea::Deployment => vec![
                "deploy",
                "docker",
                "kubernetes",
                "ci",
                "cd",
                "pipeline",
                "build",
            ],
            FocusArea::Dependencies => vec![
                "dependency",
                "import",
                "require",
                "package",
                "module",
                "crate",
            ],
            FocusArea::Custom(_s) => vec![], // Custom focus areas don't have predefined keywords
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_focus_area_parsing() {
        assert_eq!(FocusArea::from_str("auth"), FocusArea::Authentication);
        assert_eq!(FocusArea::from_str("API"), FocusArea::API);
        assert_eq!(
            FocusArea::from_str("custom_thing"),
            FocusArea::Custom("custom_thing".to_string())
        );
    }

    #[test]
    fn test_token_savings() {
        let savings = TokenSavings::new(1000, 300, "quantum-semantic");
        assert_eq!(savings.percentage_saved, 70.0);
    }

    #[test]
    fn test_focus_area_keywords() {
        let auth_keywords = FocusArea::Authentication.keywords();
        assert!(auth_keywords.contains(&"auth"));
        assert!(auth_keywords.contains(&"login"));
    }
}
