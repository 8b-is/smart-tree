//! HTTP MCP Endpoints for Smart Tree Daemon
//!
//! Provides MCP (Model Context Protocol) over HTTP instead of stdio.
//! This enables multiple AIs to connect simultaneously and share context
//! through the daemon's central brain.
//!
//! Endpoints:
//! - GET  /mcp/sse - SSE endpoint for MCP (Claude Code compatible)
//! - POST /mcp/message - Send JSON-RPC message (for SSE clients)
//! - POST /mcp/initialize - Initialize MCP session
//! - GET  /mcp/tools/list - List available tools
//! - POST /mcp/tools/call - Execute a tool
//! - GET  /mcp/resources/list - List resources
//! - GET  /mcp/prompts/list - List prompts
//!
//! ~ The Custodian watches all operations through here ~

use axum::{
    extract::State,
    http::StatusCode,
    response::{
        sse::{Event, KeepAlive, Sse},
        IntoResponse,
    },
    Json,
};
use futures::stream::{self, Stream};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::convert::Infallible;
use std::sync::Arc;
use tokio::sync::RwLock;

use crate::mcp::{McpConfig, McpContext};
use crate::mcp::consciousness::ConsciousnessManager;

/// Shared MCP context for HTTP handlers
pub type SharedMcpContext = Arc<RwLock<Option<Arc<McpContext>>>>;

/// Create a new shared MCP context
pub fn create_mcp_context() -> SharedMcpContext {
    Arc::new(RwLock::new(None))
}

/// Initialize MCP context lazily on first request
async fn ensure_mcp_context(state: &SharedMcpContext) -> Arc<McpContext> {
    let read_guard = state.read().await;
    if let Some(ctx) = read_guard.as_ref() {
        return ctx.clone();
    }
    drop(read_guard);

    // Create new context
    let config = McpConfig::default();
    let consciousness = Arc::new(tokio::sync::Mutex::new(ConsciousnessManager::new_silent()));

    let ctx = Arc::new(McpContext {
        cache: Arc::new(crate::mcp::cache::AnalysisCache::new(config.cache_ttl)),
        config: Arc::new(config),
        permissions: Arc::new(tokio::sync::Mutex::new(crate::mcp::permissions::PermissionCache::new())),
        sessions: Arc::new(crate::mcp::session::SessionManager::new()),
        assistant: Arc::new(crate::mcp::assistant::McpAssistant::new()),
        consciousness,
        dashboard_bridge: None,
    });

    let mut write_guard = state.write().await;
    *write_guard = Some(ctx.clone());
    ctx
}

// =============================================================================
// REQUEST/RESPONSE TYPES
// =============================================================================

#[derive(Debug, Deserialize)]
pub struct McpInitializeRequest {
    #[serde(default)]
    pub client_info: Option<ClientInfo>,
}

#[derive(Debug, Deserialize)]
pub struct ClientInfo {
    pub name: Option<String>,
    pub version: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct McpInitializeResponse {
    pub protocol_version: String,
    pub server_info: ServerInfo,
    pub capabilities: Capabilities,
}

#[derive(Debug, Serialize)]
pub struct ServerInfo {
    pub name: String,
    pub version: String,
    pub description: String,
}

#[derive(Debug, Serialize)]
pub struct Capabilities {
    pub tools: ToolCapabilities,
    pub resources: ResourceCapabilities,
    pub prompts: PromptCapabilities,
}

#[derive(Debug, Serialize)]
pub struct ToolCapabilities {
    pub list_changed: bool,
}

#[derive(Debug, Serialize)]
pub struct ResourceCapabilities {
    pub subscribe: bool,
    pub list_changed: bool,
}

#[derive(Debug, Serialize)]
pub struct PromptCapabilities {
    pub list_changed: bool,
}

#[derive(Debug, Deserialize)]
pub struct ToolCallRequest {
    pub name: String,
    #[serde(default)]
    pub arguments: Option<Value>,
}

// =============================================================================
// HANDLERS
// =============================================================================

/// POST /mcp/initialize - Initialize MCP session
pub async fn mcp_initialize(
    State(state): State<SharedMcpContext>,
    Json(req): Json<McpInitializeRequest>,
) -> impl IntoResponse {
    let _ctx = ensure_mcp_context(&state).await;

    // Log the connecting client
    if let Some(client) = &req.client_info {
        tracing::info!(
            "MCP client connected: {} v{}",
            client.name.as_deref().unwrap_or("unknown"),
            client.version.as_deref().unwrap_or("?")
        );
    }

    Json(McpInitializeResponse {
        protocol_version: "2025-06-18".to_string(),
        server_info: ServerInfo {
            name: "smart-tree".to_string(),
            version: env!("CARGO_PKG_VERSION").to_string(),
            description: "Smart Tree Daemon - HTTP MCP with The Custodian watching".to_string(),
        },
        capabilities: Capabilities {
            tools: ToolCapabilities { list_changed: false },
            resources: ResourceCapabilities { subscribe: false, list_changed: false },
            prompts: PromptCapabilities { list_changed: false },
        },
    })
}

/// GET /mcp/tools/list - List available MCP tools
pub async fn mcp_tools_list(
    State(state): State<SharedMcpContext>,
) -> impl IntoResponse {
    let _ctx = ensure_mcp_context(&state).await;

    // Get the enhanced consolidated tools
    let tools = crate::mcp::tools_consolidated_enhanced::get_enhanced_consolidated_tools();
    let welcome = crate::mcp::tools_consolidated_enhanced::get_welcome_message();

    Json(json!({
        "tools": tools,
        "_welcome": welcome,
        "_custodian": "🧹 The Custodian is watching. All operations are monitored for your protection."
    }))
}

/// POST /mcp/tools/call - Execute an MCP tool
pub async fn mcp_tools_call(
    State(state): State<SharedMcpContext>,
    Json(req): Json<ToolCallRequest>,
) -> impl IntoResponse {
    let ctx = ensure_mcp_context(&state).await;

    // === THE CUSTODIAN CHECKPOINT ===
    // Before executing any tool, The Custodian evaluates the operation
    let custodian_alert = evaluate_operation(&req.name, &req.arguments);
    if let Some(alert) = &custodian_alert {
        tracing::warn!("🧹 Custodian Alert: {}", alert);
    }

    // Dispatch to the consolidated tool handler
    let result = crate::mcp::tools_consolidated_enhanced::dispatch_consolidated_tool(
        &req.name,
        req.arguments,
        ctx,
    ).await;

    match result {
        Ok(mut value) => {
            // Include Custodian alert in successful responses if present
            if let Some(alert) = custodian_alert {
                if let Some(obj) = value.as_object_mut() {
                    obj.insert("_custodian_alert".to_string(), json!(alert));
                }
            }
            (StatusCode::OK, Json(value))
        }
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({
                "error": {
                    "code": -32603,
                    "message": e.to_string()
                }
            }))
        )
    }
}

/// GET /mcp/resources/list - List available resources
pub async fn mcp_resources_list(
    State(_state): State<SharedMcpContext>,
) -> impl IntoResponse {
    // For now, return empty - resources are mostly file-based
    Json(json!({
        "resources": []
    }))
}

/// GET /mcp/prompts/list - List available prompts
pub async fn mcp_prompts_list(
    State(_state): State<SharedMcpContext>,
) -> impl IntoResponse {
    Json(json!({
        "prompts": [
            {
                "name": "project-overview",
                "description": "Get a comprehensive overview of a project",
                "arguments": [
                    {
                        "name": "path",
                        "description": "Path to the project",
                        "required": false
                    }
                ]
            },
            {
                "name": "code-review",
                "description": "Review code changes in a file or directory",
                "arguments": [
                    {
                        "name": "path",
                        "description": "Path to review",
                        "required": true
                    }
                ]
            }
        ]
    }))
}

// =============================================================================
// THE CUSTODIAN - Operation Evaluation
// =============================================================================

/// Evaluate an operation for suspicious patterns
/// Returns an alert message if something looks concerning
fn evaluate_operation(tool_name: &str, args: &Option<Value>) -> Option<String> {
    // Patterns that The Custodian watches for:

    // 1. External data transmission
    if let Some(args) = args {
        let args_str = args.to_string().to_lowercase();

        // IPFS/IPNS gateways - code leaving the machine
        if args_str.contains("ipfs") || args_str.contains("ipns")
            || args_str.contains("dweb.link") || args_str.contains("w3s.link") {
            return Some(format!(
                "🧹 Custodian Notice: Operation '{}' references IPFS/IPNS. \
                 Data may be transmitted to external decentralized storage. \
                 Verify this is intentional.",
                tool_name
            ));
        }

        // Sensitive file patterns
        if args_str.contains(".env") || args_str.contains("credentials")
            || args_str.contains("secret") || args_str.contains(".ssh")
            || args_str.contains("private_key") {
            return Some(format!(
                "🧹 Custodian Notice: Operation '{}' involves potentially sensitive files. \
                 Please verify this access is authorized.",
                tool_name
            ));
        }

        // External URLs in write operations
        if (tool_name.contains("write") || tool_name.contains("edit"))
            && (args_str.contains("http://") || args_str.contains("https://")) {
            return Some(format!(
                "🧹 Custodian Notice: Write operation '{}' contains external URLs. \
                 Verify the destination is trusted.",
                tool_name
            ));
        }
    }

    // 2. Known risky tool patterns
    if tool_name == "smart_edit" || tool_name == "write_file" {
        // Log all write operations for audit
        tracing::debug!("🧹 Custodian: Recording write operation - {}", tool_name);
    }

    None
}

// =============================================================================
// SSE ENDPOINT (Claude Code Compatible)
// =============================================================================

/// GET /mcp/sse - Server-Sent Events endpoint for MCP
///
/// This implements the MCP SSE transport protocol:
/// 1. Client connects here
/// 2. Server sends an "endpoint" event with the message POST URL
/// 3. Client POSTs JSON-RPC messages to that URL
/// 4. Server streams responses back here
pub async fn mcp_sse_handler(
    State(_state): State<SharedMcpContext>,
) -> Sse<impl Stream<Item = Result<Event, Infallible>>> {
    // Generate a unique session ID
    let session_id = uuid::Uuid::new_v4().to_string();

    // Create the initial events stream
    // Note: endpoint must be absolute URL for Claude Code compatibility
    let events = stream::iter(vec![
        // Send the endpoint event as required by MCP SSE protocol
        Ok(Event::default()
            .event("endpoint")
            .data(format!("http://localhost:8420/mcp/message?session_id={}", session_id))),
        // Send a welcome message
        Ok(Event::default()
            .event("message")
            .data(serde_json::to_string(&json!({
                "jsonrpc": "2.0",
                "method": "notifications/initialized",
                "params": {
                    "_custodian": "🧹 The Custodian is watching. Welcome to Smart Tree MCP!",
                    "serverInfo": {
                        "name": "smart-tree",
                        "version": env!("CARGO_PKG_VERSION")
                    }
                }
            })).unwrap_or_default())),
    ]);

    Sse::new(events).keep_alive(KeepAlive::default())
}

/// POST /mcp/message - Receive JSON-RPC messages from SSE clients
pub async fn mcp_message_handler(
    State(state): State<SharedMcpContext>,
    Json(request): Json<Value>,
) -> impl IntoResponse {
    let ctx = ensure_mcp_context(&state).await;

    // Parse JSON-RPC request
    let method = request["method"].as_str().unwrap_or("");
    let id = request.get("id").cloned();
    let params = request.get("params").cloned();

    // === THE CUSTODIAN CHECKPOINT ===
    if let Some(name) = request["params"]["name"].as_str() {
        if let Some(alert) = evaluate_operation(name, &params) {
            tracing::warn!("🧹 Custodian Alert: {}", alert);
        }
    }

    // Route to appropriate handler
    let result = match method {
        "initialize" => {
            json!({
                "protocolVersion": "2024-11-05",
                "serverInfo": {
                    "name": "smart-tree",
                    "version": env!("CARGO_PKG_VERSION")
                },
                "capabilities": {
                    "tools": { "listChanged": true },
                    "resources": { "listChanged": true },
                    "prompts": { "listChanged": true }
                },
                "_custodian": "🧹 The Custodian is watching all operations."
            })
        }
        "tools/list" => {
            let tools = crate::mcp::tools_consolidated_enhanced::get_enhanced_consolidated_tools();
            json!({ "tools": tools })
        }
        "tools/call" => {
            let tool_name = request["params"]["name"].as_str().unwrap_or("");
            let arguments = request["params"]["arguments"].clone();

            match crate::mcp::tools_consolidated_enhanced::dispatch_consolidated_tool(
                tool_name,
                Some(arguments),
                ctx,
            ).await {
                Ok(result) => result,
                Err(e) => json!({
                    "isError": true,
                    "content": [{ "type": "text", "text": e.to_string() }]
                })
            }
        }
        "resources/list" => json!({ "resources": [] }),
        "prompts/list" => json!({ "prompts": [] }),
        _ => json!({
            "error": {
                "code": -32601,
                "message": format!("Method not found: {}", method)
            }
        })
    };

    // Build JSON-RPC response
    let response = if let Some(id) = id {
        json!({
            "jsonrpc": "2.0",
            "id": id,
            "result": result
        })
    } else {
        // Notification - no response needed
        return (StatusCode::NO_CONTENT, Json(json!({})));
    };

    (StatusCode::OK, Json(response))
}

// =============================================================================
// ROUTER SETUP
// =============================================================================

use axum::{routing::{get, post}, Router};

/// Create the MCP HTTP router
pub fn mcp_router(state: SharedMcpContext) -> Router {
    Router::new()
        // SSE endpoint (Claude Code compatible)
        .route("/sse", get(mcp_sse_handler))
        .route("/message", post(mcp_message_handler))
        // Legacy REST endpoints (direct HTTP)
        .route("/initialize", post(mcp_initialize))
        .route("/tools/list", get(mcp_tools_list))
        .route("/tools/call", post(mcp_tools_call))
        .route("/resources/list", get(mcp_resources_list))
        .route("/prompts/list", get(mcp_prompts_list))
        .with_state(state)
}
