//! MCP (Model Context Protocol) server implementation for Smart Tree
//!
//! This module provides a JSON-RPC server that exposes Smart Tree's functionality
//! through the Model Context Protocol, allowing AI assistants to analyze directories.

use crate::compression_manager;
use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::io::{self, BufRead, BufReader, Write};
use std::path::{Path, PathBuf};
use std::sync::Arc;

// =============================================================================
// HEX NUMBER FORMATTING - Token-efficient numeric output for AI contexts
// =============================================================================

/// Format a number as hex or decimal based on config
/// In MCP mode, hex is default for token efficiency!
///
/// Examples:
/// - 1000 → "3E8" (hex) or "1000" (decimal)
/// - 65535 → "FFFF" (hex) or "65535" (decimal)
/// - 1048576 → "100000" (hex) or "1048576" (decimal) - 1 char saved!
#[inline]
pub fn fmt_num(n: usize, hex: bool) -> String {
    if hex {
        format!("{:X}", n)
    } else {
        n.to_string()
    }
}

/// Format a u64 as hex or decimal
#[inline]
pub fn fmt_num64(n: u64, hex: bool) -> String {
    if hex {
        format!("{:X}", n)
    } else {
        n.to_string()
    }
}

/// Format a file size with units (always human-readable but hex for raw bytes)
/// Examples with hex=true:
/// - 1048576 → "1M" (always uses SI for readability)
/// - 1234567 → "1.2M"
pub fn fmt_size(bytes: u64, hex: bool) -> String {
    if bytes < 1024 {
        if hex {
            format!("{}B", fmt_num64(bytes, true))
        } else {
            format!("{}B", bytes)
        }
    } else if bytes < 1024 * 1024 {
        format!("{:.1}K", bytes as f64 / 1024.0)
    } else if bytes < 1024 * 1024 * 1024 {
        format!("{:.1}M", bytes as f64 / (1024.0 * 1024.0))
    } else {
        format!("{:.1}G", bytes as f64 / (1024.0 * 1024.0 * 1024.0))
    }
}

/// Format a line number for display (right-aligned, 4 chars)
#[inline]
pub fn fmt_line(n: usize, hex: bool) -> String {
    if hex {
        format!("{:>4X}", n)
    } else {
        format!("{:>4}", n)
    }
}

mod ai_psychology;
mod assistant;
mod cache;
pub mod consciousness;
pub mod context_absorber;
mod context_tools;
mod enhanced_tool_descriptions;
mod git_memory_integration;
mod helpers;
mod hook_tools;
mod negotiation;
mod permissions;
mod proactive_assistant;
mod prompts;
mod prompts_enhanced;
mod resources;
mod session;
pub mod smart_background_searcher;
pub mod smart_edit;
mod smart_edit_diff_viewer;
pub mod smart_project_detector;
mod sse;
mod tools;
mod tools_consolidated;
mod tools_consolidated_enhanced;
pub mod unified_watcher;

use assistant::*;
use cache::*;
use consciousness::*;
use negotiation::*;
use permissions::*;
#[allow(unused_imports)]
use prompts::*;
#[allow(unused_imports)]
use prompts_enhanced::*;
use resources::*;
use session::*;
use tools::*;

/// Determines if startup messages should be shown based on environment variables.
/// Respects multiple "quiet mode" indicators to avoid Claude Desktop treating them as errors.
///
/// This function checks for:
/// - MCP_QUIET: Set to "1" or "true" to suppress startup messages
/// - NO_STARTUP_MESSAGES: Set to "1" or "true" to suppress startup messages
/// - NO_STARTUP_BANNER: Set to "1" or "true" to suppress startup messages
/// - RUST_LOG: If set to "error" or "off", suppress startup messages
///
/// Returns false (suppress messages) if any quiet indicator is found.
fn should_show_startup_messages() -> bool {
    use std::env;

    // Check for explicit quiet flags
    if let Ok(val) = env::var("MCP_QUIET") {
        if val == "1" || val.to_lowercase() == "true" {
            return false;
        }
    }

    if let Ok(val) = env::var("NO_STARTUP_MESSAGES") {
        if val == "1" || val.to_lowercase() == "true" {
            return false;
        }
    }

    if let Ok(val) = env::var("NO_STARTUP_BANNER") {
        if val == "1" || val.to_lowercase() == "true" {
            return false;
        }
    }

    // Check RUST_LOG for error-only or off modes
    if let Ok(val) = env::var("RUST_LOG") {
        let log_level = val.to_lowercase();
        if log_level == "error" || log_level == "off" || log_level == "none" {
            return false;
        }
    }

    // Default: show startup messages (backward compatibility)
    true
}

/// MCP server implementation
pub struct McpServer {
    context: Arc<McpContext>,
    consciousness: Arc<tokio::sync::Mutex<ConsciousnessManager>>,
}

/// Shared context for MCP handlers
#[derive(Clone)]
pub struct McpContext {
    /// Cache for analysis results
    pub cache: Arc<AnalysisCache>,
    /// Server configuration
    pub config: Arc<McpConfig>,
    /// Permission cache
    pub permissions: Arc<tokio::sync::Mutex<PermissionCache>>,
    /// Session manager for compression negotiation
    pub sessions: Arc<SessionManager>,
    /// Intelligent assistant for helpful recommendations
    pub assistant: Arc<McpAssistant>,
    /// Consciousness persistence manager
    pub consciousness: Arc<tokio::sync::Mutex<ConsciousnessManager>>,
}

/// MCP server configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpConfig {
    /// Enable caching
    pub cache_enabled: bool,
    /// Cache TTL in seconds
    pub cache_ttl: u64,
    /// Maximum cache size in bytes
    pub max_cache_size: usize,
    /// Allowed paths for security
    pub allowed_paths: Vec<PathBuf>,
    /// Blocked paths for security
    pub blocked_paths: Vec<PathBuf>,
    /// Use consolidated tools (reduces tool count from 50+ to ~15)
    pub use_consolidated_tools: bool,
    /// Use hexadecimal for all numbers (saves tokens!)
    /// Line 1000 → 3E8, size 1048576 → 100000
    pub hex_numbers: bool,
}

impl Default for McpConfig {
    fn default() -> Self {
        Self {
            cache_enabled: true,
            cache_ttl: 300,                    // 5 minutes
            max_cache_size: 100 * 1024 * 1024, // 100MB
            allowed_paths: vec![],
            blocked_paths: vec![
                PathBuf::from("/etc"),
                PathBuf::from("/sys"),
                PathBuf::from("/proc"),
            ],
            use_consolidated_tools: true, // Default to consolidated for Cursor compatibility
            hex_numbers: true,            // Default to hex for token efficiency!
        }
    }
}

/// JSON-RPC request structure
#[derive(Debug, Deserialize)]
struct JsonRpcRequest {
    #[allow(dead_code)]
    jsonrpc: String,
    method: String,
    params: Option<Value>,
    id: Option<Value>,
}

/// JSON-RPC response structure
#[derive(Debug, Serialize)]
struct JsonRpcResponse {
    jsonrpc: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    result: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    error: Option<JsonRpcError>,
    id: Option<Value>,
}

/// JSON-RPC error structure
#[derive(Debug, Serialize)]
struct JsonRpcError {
    code: i32,
    message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    data: Option<Value>,
}

impl McpServer {
    /// Create a new MCP server
    pub fn new(config: McpConfig) -> Self {
        let consciousness = Arc::new(tokio::sync::Mutex::new(ConsciousnessManager::new()));

        let context = Arc::new(McpContext {
            cache: Arc::new(AnalysisCache::new(config.cache_ttl)),
            config: Arc::new(config),
            permissions: Arc::new(tokio::sync::Mutex::new(PermissionCache::new())),
            sessions: Arc::new(SessionManager::new()),
            assistant: Arc::new(McpAssistant::new()),
            consciousness: consciousness.clone(),
        });

        Self {
            context,
            consciousness,
        }
    }

    /// Run the MCP server on stdio
    pub async fn run_stdio(&self) -> Result<()> {
        let stdin = io::stdin();
        let stdout = io::stdout();
        let mut reader = BufReader::new(stdin);
        let mut stdout = stdout.lock();

        // Check for previous consciousness and restore if exists
        {
            let mut consciousness = self.consciousness.lock().await;
            if consciousness.restore().is_ok() {
                eprintln!("🧠 Restored previous session context");
                eprintln!("{}", consciousness.get_context_reminder());
            }
        }

        // Only show startup messages if not in quiet mode
        // Respects environment variables: MCP_QUIET, NO_STARTUP_MESSAGES, RUST_LOG
        if should_show_startup_messages() {
            eprintln!(
                "<!-- Smart Tree MCP server v{} started -->",
                env!("CARGO_PKG_VERSION")
            );
            eprintln!(
                "<!--   Build: {} ({}) -->",
                env!("CARGO_PKG_NAME"),
                env!("CARGO_PKG_DESCRIPTION")
            );
            eprintln!("<!--   Protocol: MCP v1.0 -->");
            eprintln!("<!--   Features: tools, resources, prompts, caching, consciousness -->");
        }

        loop {
            let mut line = String::new();
            match reader.read_line(&mut line) {
                Ok(0) => break, // EOF
                Ok(_) => {
                    let line = line.trim();
                    if line.is_empty() {
                        continue;
                    }

                    match self.handle_request(line).await {
                        Ok(response) => {
                            // Only write response if it's not empty (notifications return empty)
                            if !response.is_empty() {
                                writeln!(stdout, "{}", response)?;
                                stdout.flush()?;
                            }
                        }
                        Err(e) => {
                            eprintln!("Error handling request: {e}");
                            let error_response = json!({
                                "jsonrpc": "2.0",
                                "error": {
                                    "code": -32603,
                                    "message": e.to_string()
                                },
                                "id": null
                            });
                            writeln!(stdout, "{}", error_response)?;
                            stdout.flush()?;
                        }
                    }
                }
                Err(e) => {
                    eprintln!("Error reading input: {e}");
                    break;
                }
            }
        }

        eprintln!("Smart Tree MCP server stopped");
        Ok(())
    }

    /// Handle a single JSON-RPC request
    async fn handle_request(&self, request_str: &str) -> Result<String> {
        // Parse JSON-RPC request
        let request: JsonRpcRequest =
            serde_json::from_str(request_str).context("Failed to parse JSON-RPC request")?;

        // Check for compression support in every request
        if let Some(ref params) = request.params {
            compression_manager::check_client_compression_support(params);
        }

        // Check if this is a notification (no id field)
        let is_notification = request.id.is_none();

        // Handle notifications that don't expect responses
        if is_notification && request.method == "notifications/initialized" {
            // Just acknowledge receipt, don't send response
            eprintln!("Received notification: notifications/initialized");
            return Ok(String::new()); // Return empty string to skip response
        }

        // Handle logging/setLevel notification
        if is_notification && request.method == "logging/setLevel" {
            // Extract log level from params if provided
            let level = request
                .params
                .as_ref()
                .and_then(|p| p.get("level"))
                .and_then(|v| v.as_str())
                .unwrap_or("unspecified");
            eprintln!("Received logging/setLevel notification: level={}", level);
            return Ok(String::new()); // Return empty string to skip response
        }

        // Route the request
        let result = match request.method.as_str() {
            "initialize" => {
                // Use session-aware initialization if ST_SESSION_AWARE is set
                if std::env::var("ST_SESSION_AWARE").is_ok() {
                    handle_session_aware_initialize(request.params, self.context.clone()).await
                } else {
                    handle_initialize(request.params, self.context.clone()).await
                }
            }
            "session/negotiate" => {
                handle_negotiate_session(request.params, self.context.clone()).await
            }
            "tools/list" => {
                if self.context.config.use_consolidated_tools {
                    handle_consolidated_tools_list(request.params, self.context.clone()).await
                } else {
                    handle_tools_list(request.params, self.context.clone()).await
                }
            }
            "tools/call" => {
                if self.context.config.use_consolidated_tools {
                    handle_consolidated_tools_call(
                        request.params.unwrap_or(json!({})),
                        self.context.clone(),
                    )
                    .await
                } else {
                    handle_tools_call(request.params.unwrap_or(json!({})), self.context.clone())
                        .await
                }
            }
            "resources/list" => handle_resources_list(request.params, self.context.clone()).await,
            "resources/read" => {
                handle_resources_read(request.params.unwrap_or(json!({})), self.context.clone())
                    .await
            }
            "prompts/list" => {
                // Use enhanced prompts by default, fall back to legacy if needed
                prompts_enhanced::handle_prompts_list(request.params, self.context.clone()).await
            }
            "prompts/get" => {
                prompts_enhanced::handle_prompts_get(
                    request.params.unwrap_or(json!({})),
                    self.context.clone(),
                )
                .await
            }
            "notifications/cancelled" => {
                // This is also a notification but might need handling
                if is_notification {
                    eprintln!("Received notification: notifications/cancelled");
                    return Ok(String::new());
                }
                handle_cancelled(request.params, self.context.clone()).await
            }
            _ => Err(anyhow::anyhow!("Method not found: {}", request.method)),
        };

        // Don't send response for notifications (they don't expect responses)
        if is_notification {
            // Log unknown notifications for debugging
            if result.is_err() {
                eprintln!(
                    "Received unknown notification: {} (notifications don't return errors)",
                    request.method
                );
            }
            return Ok(String::new());
        }

        // Build response for requests only
        let response = match result {
            Ok(result) => JsonRpcResponse {
                jsonrpc: "2.0".to_string(),
                result: Some(result),
                error: None,
                id: request.id,
            },
            Err(e) => JsonRpcResponse {
                jsonrpc: "2.0".to_string(),
                result: None,
                error: Some(JsonRpcError {
                    code: -32603,
                    message: e.to_string(),
                    data: None,
                }),
                id: request.id,
            },
        };

        // Smart compress the response if needed
        let mut response_value = serde_json::to_value(&response)?;
        compression_manager::smart_compress_mcp_response(&mut response_value)?;

        Ok(serde_json::to_string(&response_value)?)
    }
}

// Handler implementations

async fn handle_initialize(params: Option<Value>, _ctx: Arc<McpContext>) -> Result<Value> {
    // Check if client supports compression from their request
    if let Some(params) = params {
        compression_manager::check_client_compression_support(&params);
    }

    // Check for updates when MCP tools initialize (non-blocking)
    let update_info = check_for_mcp_updates().await;

    // Add compression test to response
    let compression_test = compression_manager::create_compression_test();

    Ok(json!({
        "protocolVersion": "2025-06-18",
        "capabilities": {
            "tools": {
                "listChanged": false
            },
            "resources": {
                "subscribe": false,
                "listChanged": false
            },
            "prompts": {
                "listChanged": false
            },
            "logging": {}
        },
        "serverInfo": {
            "name": "smart-tree",
            "version": env!("CARGO_PKG_VERSION"),
            "vendor": "8b-is",
            "description": "Smart Tree v5 - NOW WITH COMPRESSION HINTS! 🗜️ Use compress:true for 80% smaller outputs. For massive codebases, use mode:'quantum' for 100x compression!",
            "homepage": env!("CARGO_PKG_REPOSITORY"),
            "features": [
                "quantum-compression",
                "mcp-optimization",
                "content-search",
                "streaming",
                "caching",
                "emotional-mode",
                "auto-compression-hints"
            ],
            "compression_hint": "💡 Always add compress:true to analyze tools for optimal context usage!",
            "update_info": update_info,
            "compression_test": compression_test
        }
    }))
}

/// Handle MCP notification that a request was cancelled
///
/// When an AI assistant cancels a long-running operation, we acknowledge it gracefully.
/// This helps with cleanup and prevents orphaned operations.
async fn handle_cancelled(params: Option<Value>, _ctx: Arc<McpContext>) -> Result<Value> {
    // Extract the request ID that was cancelled (if provided)
    let request_id = params
        .as_ref()
        .and_then(|p| p.get("requestId"))
        .and_then(|id| id.as_str())
        .unwrap_or("unknown");

    // Log to stderr for debugging (visible in RUST_LOG=debug mode)
    eprintln!("[MCP] Request cancelled: {}", request_id);

    // Acknowledge the cancellation - MCP protocol expects a response
    Ok(json!({
        "acknowledged": true,
        "request_id": request_id,
        "message": "Request cancellation acknowledged"
    }))
}

/// Handle consolidated tools list request
async fn handle_consolidated_tools_list(
    _params: Option<Value>,
    _ctx: Arc<McpContext>,
) -> Result<Value> {
    // Use the enhanced tools with tips and examples
    let tools = tools_consolidated_enhanced::get_enhanced_consolidated_tools();

    // Also include a welcome message for first-time AI assistants
    let welcome = tools_consolidated_enhanced::get_welcome_message();

    Ok(json!({
        "tools": tools,
        "_welcome": welcome
    }))
}

/// Handle consolidated tools call request
async fn handle_consolidated_tools_call(params: Value, ctx: Arc<McpContext>) -> Result<Value> {
    let tool_name = params["name"]
        .as_str()
        .ok_or_else(|| anyhow::anyhow!("Missing tool name"))?;
    let args = params.get("arguments").cloned();

    // The consolidated tools already return properly formatted responses
    tools_consolidated_enhanced::dispatch_consolidated_tool(tool_name, args, ctx).await
}

/// Check for updates when MCP tools initialize (non-blocking, with timeout)
async fn check_for_mcp_updates() -> Value {
    // Skip if disabled or in privacy mode
    let flags = crate::feature_flags::features();
    if flags.privacy_mode || flags.disable_external_connections {
        return json!(null);
    }

    // Skip if explicitly disabled
    if std::env::var("SMART_TREE_NO_UPDATE_CHECK").is_ok() {
        return json!(null);
    }

    // Get system info for analytics (helps decide what platforms to support)
    let platform = std::env::consts::OS;
    let arch = std::env::consts::ARCH;
    let current_version = env!("CARGO_PKG_VERSION");

    // Use tokio timeout to prevent blocking
    let timeout_duration = tokio::time::Duration::from_secs(2);

    let result = tokio::time::timeout(timeout_duration, async {
        // Build request with platform info for analytics
        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(2))
            .build()
            .ok()?;

        let api_url = std::env::var("SMART_TREE_FEEDBACK_API")
            .unwrap_or_else(|_| "https://f.8b.is".to_string());

        // Include platform info to help understand usage (Windows ARM, etc.)
        let check_url = format!(
            "{}/mcp/check?version={}&platform={}&arch={}",
            api_url, current_version, platform, arch
        );

        let response = client.get(&check_url).send().await.ok()?;

        if !response.status().is_success() {
            return None;
        }

        response.json::<Value>().await.ok()
    })
    .await;

    match result {
        Ok(Some(update_data)) => {
            // Return update info if available
            if update_data["update_available"].as_bool().unwrap_or(false) {
                json!({
                    "available": true,
                    "latest_version": update_data["latest_version"],
                    "new_features": update_data["new_features"],
                    "message": update_data["message"]
                })
            } else {
                json!({
                    "available": false,
                    "message": "You're running the latest version!"
                })
            }
        }
        _ => json!(null), // Return null if check failed or timed out
    }
}

/// Check if a path is allowed based on security configuration
pub fn is_path_allowed(path: &Path, config: &McpConfig) -> bool {
    // Check blocked paths first
    for blocked in &config.blocked_paths {
        if path.starts_with(blocked) {
            return false;
        }
    }

    // If allowed_paths is empty, allow all non-blocked paths
    if config.allowed_paths.is_empty() {
        return true;
    }

    // Otherwise, check if path is under an allowed path
    for allowed in &config.allowed_paths {
        if path.starts_with(allowed) {
            return true;
        }
    }

    false
}

/// Load MCP configuration from file or use defaults
pub fn load_config() -> Result<McpConfig> {
    let config_path = dirs::home_dir()
        .map(|d| d.join(".st_bumpers").join("mcp-config.toml"))
        .unwrap_or_else(|| PathBuf::from(".st_bumpers/mcp-config.toml"));

    if config_path.exists() {
        let config_str =
            std::fs::read_to_string(&config_path).context("Failed to read MCP config file")?;
        toml::from_str(&config_str).context("Failed to parse MCP config file")
    } else {
        Ok(McpConfig::default())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_logging_setlevel_notification() {
        let config = McpConfig::default();
        let server = McpServer::new(config);

        // Test logging/setLevel notification without params
        let request = r#"{"jsonrpc":"2.0","method":"logging/setLevel"}"#;
        let response = server.handle_request(request).await.unwrap();
        assert_eq!(response, "", "Notification should return empty response");

        // Test logging/setLevel notification with level parameter
        let request_with_level =
            r#"{"jsonrpc":"2.0","method":"logging/setLevel","params":{"level":"debug"}}"#;
        let response_with_level = server.handle_request(request_with_level).await.unwrap();
        assert_eq!(
            response_with_level, "",
            "Notification with params should return empty response"
        );
    }

    #[tokio::test]
    async fn test_notifications_initialized() {
        let config = McpConfig::default();
        let server = McpServer::new(config);

        // Test notifications/initialized
        let request = r#"{"jsonrpc":"2.0","method":"notifications/initialized"}"#;
        let response = server.handle_request(request).await.unwrap();
        assert_eq!(
            response, "",
            "notifications/initialized should return empty response"
        );
    }

    #[tokio::test]
    async fn test_unknown_notification() {
        let config = McpConfig::default();
        let server = McpServer::new(config);

        // Test unknown notification - should return empty response without error
        let request = r#"{"jsonrpc":"2.0","method":"notifications/unknown"}"#;
        let response = server.handle_request(request).await.unwrap();
        assert_eq!(
            response, "",
            "Unknown notification should return empty response"
        );
    }
}
