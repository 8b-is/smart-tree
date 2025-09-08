//! Session negotiation handlers for MCP

use super::session::*;
use super::McpContext;
use anyhow::Result;
use serde_json::{json, Value};
use std::sync::Arc;

/// Handle session negotiation request
pub async fn handle_negotiate_session(
    params: Option<Value>,
    context: Arc<McpContext>,
) -> Result<Value> {
    // Extract session ID if provided
    let session_id = params
        .as_ref()
        .and_then(|p| p.get("session_id"))
        .and_then(|s| s.as_str())
        .map(String::from);

    // Get or create session
    let mut session = context.sessions.get_or_create(session_id).await;

    // Check for client preferences
    let client_prefs = params
        .as_ref()
        .and_then(|p| p.get("session_prefs"))
        .and_then(|sp| serde_json::from_value::<SessionPreferences>(sp.clone()).ok());

    // Negotiate with client
    let response = session.negotiate(client_prefs);

    // Update session in manager
    context.sessions.update(session).await;

    // Return negotiation response
    Ok(json!({
        "session_id": response.session_id,
        "accepted": response.accepted,
        "compression": {
            "format": response.format,
            "available_formats": ["none", "light", "standard", "quantum", "quantum_semantic"],
        },
        "project_path": response.project_path.to_string_lossy(),
        "tools_available": response.tools_available,
        "needs_preferences": !response.accepted,
    }))
}

/// Handle initialize with session awareness
pub async fn handle_session_aware_initialize(
    params: Option<Value>,
    context: Arc<McpContext>,
) -> Result<Value> {
    // Check for compression hints in environment
    let compression_mode = CompressionMode::from_env();

    // Create initial session
    let session = McpSession::from_context(None);
    context.sessions.update(session.clone()).await;

    // Determine which tools to advertise based on mode
    let tools_to_advertise = match std::env::var("ST_TOOL_MODE").as_deref() {
        Ok("all") => ToolAdvertisement::All,
        Ok("minimal") => ToolAdvertisement::Minimal,
        Ok("context") => ToolAdvertisement::ContextAware,
        _ => ToolAdvertisement::Lazy,
    };

    Ok(json!({
        "protocolVersion": "2024-11-05",
        "serverInfo": {
            "name": "smart-tree",
            "version": env!("CARGO_PKG_VERSION"),
        },
        "capabilities": {
            "tools": {},
            "resources": {
                "list": true,
                "read": true,
            },
            "prompts": {
                "list": true,
            },
            "session": {
                "negotiation": true,
                "compression": compression_mode.to_output_mode(),
                "session_id": session.id,
                "project_context": session.project_path.to_string_lossy(),
            },
        },
        "instructions": "Smart Tree MCP server with session-aware compression. Call 'negotiate_session' to configure compression preferences.",
    }))
}

/// Apply session context to tool calls
pub async fn apply_session_context(
    tool_name: &str,
    params: &mut Value,
    context: Arc<McpContext>,
) -> Result<()> {
    // Extract session ID from params if present
    let session_id = params
        .get("session_id")
        .and_then(|s| s.as_str())
        .map(String::from);

    // Get session
    let session = context.sessions.get_or_create(session_id).await;

    // Apply context
    session.apply_context(tool_name, params);

    Ok(())
}
