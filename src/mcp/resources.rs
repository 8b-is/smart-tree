//! MCP resources implementation for Smart Tree

use super::McpContext;
use anyhow::Result;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::sync::Arc;

#[derive(Debug, Serialize, Deserialize)]
struct ResourceDefinition {
    uri: String,
    name: String,
    description: String,
    #[serde(rename = "mimeType")]
    mime_type: String,
}

pub async fn handle_resources_list(_params: Option<Value>, _ctx: Arc<McpContext>) -> Result<Value> {
    let resources = vec![
        ResourceDefinition {
            uri: "cache://directory_cache".to_string(),
            name: "Directory Cache".to_string(),
            description: "Cached directory analysis results".to_string(),
            mime_type: "application/json".to_string(),
        },
        ResourceDefinition {
            uri: "config://ignore_patterns".to_string(),
            name: "Ignore Patterns".to_string(),
            description: "Active gitignore and default ignore patterns".to_string(),
            mime_type: "text/plain".to_string(),
        },
        ResourceDefinition {
            uri: "config://mcp_settings".to_string(),
            name: "MCP Settings".to_string(),
            description: "Current MCP server configuration".to_string(),
            mime_type: "application/json".to_string(),
        },
    ];

    Ok(json!({
        "resources": resources
    }))
}

pub async fn handle_resources_read(params: Value, ctx: Arc<McpContext>) -> Result<Value> {
    let uri = params["uri"]
        .as_str()
        .ok_or_else(|| anyhow::anyhow!("Missing resource URI"))?;

    match uri {
        "cache://directory_cache" => read_directory_cache(ctx).await,
        "config://ignore_patterns" => read_ignore_patterns(ctx).await,
        "config://mcp_settings" => read_mcp_settings(ctx).await,
        _ => Err(anyhow::anyhow!("Unknown resource: {}", uri)),
    }
}

async fn read_directory_cache(ctx: Arc<McpContext>) -> Result<Value> {
    // Get cache statistics
    let cache_size = ctx.cache.len();

    // Clean up expired entries
    ctx.cache.cleanup().await;
    let active_size = ctx.cache.len();

    let content = json!({
        "total_entries": cache_size,
        "active_entries": active_size,
        "expired_entries": cache_size - active_size,
        "cache_ttl_seconds": ctx.config.cache_ttl,
        "cache_enabled": ctx.config.cache_enabled,
    });

    Ok(json!({
        "contents": [{
            "uri": "cache://directory_cache",
            "mimeType": "application/json",
            "text": serde_json::to_string_pretty(&content)?
        }]
    }))
}

async fn read_ignore_patterns(_ctx: Arc<McpContext>) -> Result<Value> {
    // Default ignore patterns used by Smart Tree
    let patterns = vec![
        "# Git",
        ".git/",
        ".gitignore",
        "",
        "# Development",
        "node_modules/",
        "__pycache__/",
        "*.pyc",
        ".pytest_cache/",
        ".mypy_cache/",
        ".tox/",
        ".coverage",
        ".coverage.*",
        "htmlcov/",
        ".hypothesis/",
        "",
        "# Build outputs",
        "target/",
        "build/",
        "dist/",
        "*.egg-info/",
        "*.egg",
        "",
        "# IDE",
        ".vscode/",
        ".idea/",
        "*.swp",
        "*.swo",
        "*~",
        ".DS_Store",
        "",
        "# Environment",
        ".env",
        ".venv/",
        "env/",
        "venv/",
        "ENV/",
        "",
        "# Logs",
        "*.log",
        "logs/",
        "",
        "# Dependencies",
        "vendor/",
        "bower_components/",
    ];

    Ok(json!({
        "contents": [{
            "uri": "config://ignore_patterns",
            "mimeType": "text/plain",
            "text": patterns.join("\n")
        }]
    }))
}

async fn read_mcp_settings(ctx: Arc<McpContext>) -> Result<Value> {
    let settings = json!({
        "cache": {
            "enabled": ctx.config.cache_enabled,
            "ttl_seconds": ctx.config.cache_ttl,
            "max_size_bytes": ctx.config.max_cache_size,
        },
        "security": {
            "allowed_paths": ctx.config.allowed_paths,
            "blocked_paths": ctx.config.blocked_paths,
        },
        "server": {
            "version": env!("CARGO_PKG_VERSION"),
            "name": "smart-tree",
        }
    });

    Ok(json!({
        "contents": [{
            "uri": "config://mcp_settings",
            "mimeType": "application/json",
            "text": serde_json::to_string_pretty(&settings)?
        }]
    }))
}
