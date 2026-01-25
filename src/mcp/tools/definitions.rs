//! Shared type definitions for MCP tools
//!
//! Contains ToolDefinition, ToolLane, default functions, and shared argument structs.

use serde::{Deserialize, Serialize};
use serde_json::Value;

/// Tool lanes for AI escalation path - Omni's three-lane design
#[derive(Debug, Clone, Serialize)]
pub enum ToolLane {
    #[allow(dead_code)]
    Explore, // Discovery and overview
    #[allow(dead_code)]
    Analyze, // Deep analysis and search
    #[allow(dead_code)]
    Act, // Modifications and writes
}

impl ToolLane {
    #[allow(dead_code)]
    pub fn emoji(&self) -> &str {
        match self {
            ToolLane::Explore => "🔍",
            ToolLane::Analyze => "🧪",
            ToolLane::Act => "⚡",
        }
    }

    #[allow(dead_code)]
    pub fn name(&self) -> &str {
        match self {
            ToolLane::Explore => "EXPLORE",
            ToolLane::Analyze => "ANALYZE",
            ToolLane::Act => "ACT",
        }
    }
}

/// Tool definition structure for MCP protocol
#[derive(Debug, Serialize, Deserialize)]
pub struct ToolDefinition {
    pub name: String,
    pub description: String,
    #[serde(rename = "inputSchema")]
    pub input_schema: Value,
}

// =============================================================================
// Default functions for serde deserialization
// =============================================================================

pub fn default_path() -> String {
    ".".to_string()
}

pub fn default_mode() -> String {
    "ai".to_string()
}

pub fn default_max_depth() -> usize {
    10
}

pub fn default_path_mode() -> String {
    "off".to_string()
}

pub fn default_context_depth() -> usize {
    5
}

pub fn default_max_files() -> usize {
    100
}

pub fn default_compression() -> String {
    "auto".to_string()
}

pub fn default_token_budget() -> usize {
    10000
}

pub fn default_true() -> bool {
    true
}

pub fn default_one() -> usize {
    1
}

pub fn default_agent() -> String {
    "claude".to_string()
}

pub fn default_sse_format() -> String {
    "ai".to_string()
}

pub fn default_heartbeat_interval() -> u64 {
    30
}

pub fn default_stats_interval() -> u64 {
    60
}

// =============================================================================
// Shared argument structs
// =============================================================================

/// Arguments for analyze_directory tool
#[derive(Debug, Deserialize)]
pub struct AnalyzeDirectoryArgs {
    #[serde(default = "default_path")]
    pub path: String,
    #[serde(default = "default_mode")]
    pub mode: String,
    #[serde(default = "default_max_depth")]
    pub max_depth: usize,
    #[serde(default)]
    pub show_hidden: bool,
    #[serde(default)]
    pub show_ignored: bool,
    #[serde(default = "default_path_mode")]
    pub path_mode: String,
    #[serde(default)]
    pub compress: Option<bool>,
}

/// Arguments for project_context_dump tool
#[derive(Debug, Deserialize)]
pub struct ProjectContextDumpArgs {
    pub path: String,
    #[serde(default = "default_context_depth")]
    pub max_depth: usize,
    #[serde(default = "default_max_files")]
    pub max_files: usize,
    #[serde(default)]
    pub include_content: bool,
    #[serde(default = "default_compression")]
    pub compression: String,
    #[serde(default = "default_token_budget")]
    pub token_budget: usize,
    #[serde(default = "default_true")]
    pub include_git: bool,
    #[serde(default)]
    pub key_files_only: bool,
}

/// Arguments for find_files tool
#[derive(Debug, Deserialize)]
pub struct FindFilesArgs {
    #[serde(default = "default_path")]
    pub path: String,
    pub pattern: Option<String>,
    pub file_type: Option<String>,
    pub entry_type: Option<String>,
    pub min_size: Option<String>,
    pub max_size: Option<String>,
    pub newer_than: Option<String>,
    pub older_than: Option<String>,
    #[serde(default = "default_max_depth")]
    pub max_depth: usize,
}

/// Arguments for verify_permissions tool
#[derive(Debug, Deserialize)]
pub struct VerifyPermissionsArgs {
    #[serde(default = "default_path")]
    pub path: String,
}

/// Arguments for watch_directory_sse tool
#[derive(Debug, Deserialize)]
pub struct WatchDirectorySseArgs {
    #[serde(default = "default_path")]
    pub path: String,
    #[serde(default = "default_sse_format")]
    pub format: String,
    #[serde(default = "default_heartbeat_interval")]
    pub heartbeat_interval: u64,
    #[serde(default = "default_stats_interval")]
    pub stats_interval: u64,
    #[serde(default)]
    pub include_content: bool,
    pub max_depth: Option<usize>,
    #[serde(default)]
    pub include_patterns: Vec<String>,
    #[serde(default)]
    pub exclude_patterns: Vec<String>,
}

/// Arguments for track_file_operation tool
#[derive(Debug, Deserialize)]
pub struct TrackFileOperationArgs {
    pub file_path: String,
    #[serde(default)]
    pub operation: Option<String>,
    pub old_content: Option<String>,
    pub new_content: Option<String>,
    #[serde(default = "default_agent")]
    pub agent: String,
    pub session_id: Option<String>,
}

/// Arguments for get_file_history tool
#[derive(Debug, Deserialize)]
pub struct GetFileHistoryArgs {
    pub file_path: String,
}

/// Arguments for get_project_history_summary tool
#[derive(Debug, Deserialize)]
pub struct GetProjectHistorySummaryArgs {
    pub project_path: String,
}

/// Arguments for smart_read tool
#[derive(Debug, Deserialize)]
pub struct SmartReadArgs {
    pub file_path: String,
    #[serde(default = "default_true")]
    pub compress: bool,
    #[serde(default)]
    pub expand_functions: Vec<String>,
    #[serde(default)]
    pub expand_context: Vec<String>,
    #[serde(default)]
    pub expand_all: bool,
    #[serde(default)]
    pub max_lines: usize,
    #[serde(default = "default_one")]
    pub offset: usize,
    #[serde(default = "default_true")]
    pub show_line_numbers: bool,
    /// Use hex line numbers. If not specified, uses MCP config default (true for AI mode)
    #[serde(default)]
    pub hex_line_numbers: Option<bool>,
}
