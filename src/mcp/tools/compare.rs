//! Directory comparison tools
//!
//! Contains compare_directories and analyze_workspace handlers.

use super::directory::{analyze_directory, project_overview};
use super::search::{find_build_files, find_config_files};
use crate::mcp::helpers::validate_and_convert_path;
use crate::mcp::McpContext;
use anyhow::Result;
use serde_json::{json, Value};
use std::sync::Arc;

/// Compare two directory structures
pub async fn compare_directories(args: Value, ctx: Arc<McpContext>) -> Result<Value> {
    let path1_str = args["path1"]
        .as_str()
        .ok_or_else(|| anyhow::anyhow!("Missing path1"))?;
    let path2_str = args["path2"]
        .as_str()
        .ok_or_else(|| anyhow::anyhow!("Missing path2"))?;

    let path1 = validate_and_convert_path(path1_str, &ctx)?;
    let path2 = validate_and_convert_path(path2_str, &ctx)?;

    // Get directory structures
    let tree1 = analyze_directory(
        json!({
            "path": path1.display().to_string(),
            "mode": "json",
            "max_depth": 10
        }),
        ctx.clone(),
    )
    .await?;

    let tree2 = analyze_directory(
        json!({
            "path": path2.display().to_string(),
            "mode": "json",
            "max_depth": 10
        }),
        ctx.clone(),
    )
    .await?;

    // Compare and format differences
    Ok(json!({
        "content": [{
            "type": "text",
            "text": format!(
                "DIRECTORY COMPARISON\n\nPath 1: {}\n{}\n\nPath 2: {}\n{}\n\nNote: Use the JSON structures to identify specific differences.",
                path1.display(),
                tree1["content"][0]["text"].as_str().unwrap_or(""),
                path2.display(),
                tree2["content"][0]["text"].as_str().unwrap_or("")
            )
        }]
    }))
}

/// Comprehensive development workspace analysis
pub async fn analyze_workspace(args: Value, ctx: Arc<McpContext>) -> Result<Value> {
    let path = args["path"]
        .as_str()
        .ok_or_else(|| anyhow::anyhow!("Missing path"))?;

    // Get project overview
    let overview = project_overview(json!({ "path": path }), ctx.clone()).await?;

    // Find build files
    let build_files = find_build_files(json!({ "path": path }), ctx.clone()).await?;

    // Find config files
    let config_files = find_config_files(json!({ "path": path }), ctx.clone()).await?;

    Ok(json!({
        "content": [{
            "type": "text",
            "text": format!(
                "WORKSPACE ANALYSIS\n\n{}\n\nBUILD FILES:\n{}\n\nCONFIG FILES:\n{}",
                overview["content"][0]["text"].as_str().unwrap_or(""),
                build_files["content"][0]["text"].as_str().unwrap_or(""),
                config_files["content"][0]["text"].as_str().unwrap_or("")
            )
        }]
    }))
}
