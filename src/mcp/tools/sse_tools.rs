//! SSE (Server-Sent Events) tools
//!
//! Contains watch_directory_sse handler.

use super::definitions::WatchDirectorySseArgs;
use crate::mcp::{is_path_allowed, McpContext};
use anyhow::Result;
use serde_json::{json, Value};
use std::path::PathBuf;
use std::sync::Arc;

/// Watch a directory for real-time changes via SSE
pub async fn watch_directory_sse(args: Value, ctx: Arc<McpContext>) -> Result<Value> {
    let args: WatchDirectorySseArgs =
        serde_json::from_value(args).map_err(|e| anyhow::anyhow!("Invalid arguments: {}", e))?;

    let path = PathBuf::from(&args.path);

    // Validate path
    if !is_path_allowed(&path, &ctx.config) {
        return Err(anyhow::anyhow!(
            "Path not allowed by security policy: {}",
            args.path
        ));
    }

    if !path.exists() {
        return Err(anyhow::anyhow!("Path does not exist: {}", args.path));
    }

    // Parse output format
    let format = match args.format.as_str() {
        "hex" => crate::mcp::sse::OutputFormat::Hex,
        "ai" => crate::mcp::sse::OutputFormat::Ai,
        "quantum" => crate::mcp::sse::OutputFormat::Quantum,
        "quantum_semantic" => crate::mcp::sse::OutputFormat::QuantumSemantic,
        "json" => crate::mcp::sse::OutputFormat::Json,
        "summary" => crate::mcp::sse::OutputFormat::Summary,
        _ => crate::mcp::sse::OutputFormat::Ai,
    };

    let sse_config = crate::mcp::sse::SseConfig {
        path: path.clone(),
        format,
        heartbeat_interval: args.heartbeat_interval,
        stats_interval: args.stats_interval,
        include_content: args.include_content,
        max_depth: args.max_depth,
        include_patterns: args.include_patterns,
        exclude_patterns: args.exclude_patterns,
    };

    // Note: In a real implementation, this would start an SSE endpoint
    // For MCP, we'll return instructions on how to use SSE
    let sse_info = format!(
        "🔄 SSE Directory Watch Configuration Created!\n\n\
        Path: {}\n\
        Format: {:?}\n\
        Heartbeat: {}s\n\
        Stats Update: {}s\n\n\
        To start receiving events, connect to the SSE endpoint:\n\
        ```javascript\n\
        const source = new EventSource('/mcp/sse/watch');\n\
        source.addEventListener('message', (e) => {{\n\
        const event = JSON.parse(e.data);\n\
        console.log('Event:', event);\n\
        }});\n\
        ```\n\n\
        Event Types:\n\
        - scan_complete: Initial scan finished\n\
        - created: File/directory created\n\
        - modified: File/directory modified\n\
        - deleted: File/directory deleted\n\
        - analysis: Periodic analysis update\n\
        - stats: Statistics update\n\
        - heartbeat: Keep-alive signal",
        args.path, args.format, args.heartbeat_interval, args.stats_interval
    );

    // Store the SSE config in cache for later retrieval
    let cache_key = format!("sse_watch_{}", args.path);
    let _ = ctx
        .cache
        .set(cache_key.clone(), serde_json::to_string(&sse_config)?)
        .await;

    Ok(json!({
        "content": [{
            "type": "text",
            "text": sse_info
        }],
        "metadata": {
            "sse_config_id": cache_key,
            "config": sse_config
        }
    }))
}
