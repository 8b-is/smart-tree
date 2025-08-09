//! MCP (Model Context Protocol) server implementation for Smart Tree
//!
//! This module provides a JSON-RPC server that exposes Smart Tree's functionality
//! through the Model Context Protocol, allowing AI assistants to analyze directories.

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::io::{self, BufRead, BufReader, Write};
use std::path::{Path, PathBuf};
use std::sync::Arc;

mod cache;
mod context_tools;
mod permissions;
mod prompts;
mod resources;
pub mod smart_edit;
mod smart_edit_diff_viewer;
mod sse;
mod tools;
mod tools_consolidated;
mod tools_consolidated_enhanced;

use cache::*;
use permissions::*;
use prompts::*;
use resources::*;
use tools::*;

/// MCP server implementation
pub struct McpServer {
    context: Arc<McpContext>,
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
        let context = Arc::new(McpContext {
            cache: Arc::new(AnalysisCache::new(config.cache_ttl)),
            config: Arc::new(config),
            permissions: Arc::new(tokio::sync::Mutex::new(PermissionCache::new())),
        });

        Self { context }
    }

    /// Run the MCP server on stdio
    pub async fn run_stdio(&self) -> Result<()> {
        let stdin = io::stdin();
        let stdout = io::stdout();
        let mut reader = BufReader::new(stdin);
        let mut stdout = stdout.lock();

        eprintln!(
            "Smart Tree MCP server v{} started",
            env!("CARGO_PKG_VERSION")
        );
        eprintln!(
            "  Build: {} ({})",
            env!("CARGO_PKG_NAME"),
            env!("CARGO_PKG_DESCRIPTION")
        );
        eprintln!("  Protocol: MCP v1.0");
        eprintln!("  Features: tools, resources, prompts, caching");

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

        // Check if this is a notification (no id field)
        let is_notification = request.id.is_none();

        // Handle notifications that don't expect responses
        if is_notification && request.method == "notifications/initialized" {
            // Just acknowledge receipt, don't send response
            eprintln!("Received notification: notifications/initialized");
            return Ok(String::new()); // Return empty string to skip response
        }

        // Route the request
        let result = match request.method.as_str() {
            "initialize" => handle_initialize(request.params, self.context.clone()).await,
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
            "prompts/list" => handle_prompts_list(request.params, self.context.clone()).await,
            "prompts/get" => {
                handle_prompts_get(request.params.unwrap_or(json!({})), self.context.clone()).await
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

        // Don't send response for notifications
        if is_notification {
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

        Ok(serde_json::to_string(&response)?)
    }
}

// Handler implementations

async fn handle_initialize(_params: Option<Value>, _ctx: Arc<McpContext>) -> Result<Value> {
    Ok(json!({
        "protocolVersion": "2024-11-05",
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
            "description": "Smart Tree - AI-optimized directory visualization with quantum compression",
            "homepage": env!("CARGO_PKG_REPOSITORY"),
            "features": [
                "quantum-compression",
                "mcp-optimization",
                "content-search",
                "streaming",
                "caching"
            ]
        }
    }))
}

async fn handle_cancelled(_params: Option<Value>, _ctx: Arc<McpContext>) -> Result<Value> {
    // TODO: Implement cancellation logic
    Ok(json!({}))
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

    let result = tools_consolidated_enhanced::dispatch_consolidated_tool(tool_name, args, ctx).await?;

    Ok(json!({
        "content": [{
            "type": "text",
            "text": serde_json::to_string_pretty(&result)?
        }]
    }))
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
        .map(|d| d.join(".st").join("mcp-config.toml"))
        .unwrap_or_else(|| PathBuf::from(".st/mcp-config.toml"));

    if config_path.exists() {
        let config_str =
            std::fs::read_to_string(&config_path).context("Failed to read MCP config file")?;
        toml::from_str(&config_str).context("Failed to parse MCP config file")
    } else {
        Ok(McpConfig::default())
    }
}
