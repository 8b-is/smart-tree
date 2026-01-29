//! Statistics tools
//!
//! Contains get_statistics, get_digest, and directory_size_breakdown handlers.

use crate::formatters::{digest::DigestFormatter, stats::StatsFormatter, Formatter};
use crate::mcp::helpers::{scan_with_config, should_use_default_ignores, validate_and_convert_path, ScannerConfigBuilder};
use crate::mcp::{fmt_num64, fmt_size, McpContext};
use anyhow::Result;
use serde_json::{json, Value};
use std::sync::Arc;

/// Get comprehensive statistics about a directory
pub async fn get_statistics(args: Value, ctx: Arc<McpContext>) -> Result<Value> {
    let path_str = args["path"]
        .as_str()
        .ok_or_else(|| anyhow::anyhow!("Missing path"))?;
    let show_hidden = args["show_hidden"].as_bool().unwrap_or(false);
    let path = validate_and_convert_path(path_str, &ctx)?;

    // Build scanner configuration using builder
    let config = ScannerConfigBuilder::for_stats(&path)
        .show_hidden(show_hidden)
        .build();

    // Scan directory
    let (_nodes, stats) = scan_with_config(&path, config)?;

    // Use stats formatter
    let formatter = StatsFormatter::new();
    let mut output = Vec::new();
    formatter.format(&mut output, &[], &stats, &path)?;

    Ok(json!({
        "content": [{
            "type": "text",
            "text": String::from_utf8_lossy(&output).to_string()
        }]
    }))
}

/// Get SHA256 digest of directory structure
pub async fn get_digest(args: Value, ctx: Arc<McpContext>) -> Result<Value> {
    let path_str = args["path"]
        .as_str()
        .ok_or_else(|| anyhow::anyhow!("Missing path"))?;
    let path = validate_and_convert_path(path_str, &ctx)?;

    // Build scanner configuration using builder
    let config = ScannerConfigBuilder::new()
        .use_default_ignores(should_use_default_ignores(&path))
        .build();

    // Scan directory
    let (nodes, stats) = scan_with_config(&path, config)?;

    // Use digest formatter
    let formatter = DigestFormatter::new();
    let mut output = Vec::new();
    formatter.format(&mut output, &nodes, &stats, &path)?;

    Ok(json!({
        "content": [{
            "type": "text",
            "text": String::from_utf8_lossy(&output).to_string()
        }]
    }))
}

/// Get size analysis of immediate subdirectories
pub async fn directory_size_breakdown(args: Value, ctx: Arc<McpContext>) -> Result<Value> {
    let path_str = args["path"]
        .as_str()
        .ok_or_else(|| anyhow::anyhow!("Missing path"))?;
    let path = validate_and_convert_path(path_str, &ctx)?;

    // Get immediate subdirectories
    let config = ScannerConfigBuilder::new()
        .max_depth(1)
        .respect_gitignore(false)
        .show_hidden(true)
        .show_ignored(true)
        .use_default_ignores(false)
        .build();

    let (nodes, _) = scan_with_config(&path, config)?;

    // Calculate size for each subdirectory
    let mut dir_sizes = Vec::new();
    for node in &nodes {
        if node.is_dir && node.path != path {
            // Get size of this directory
            let subconfig = ScannerConfigBuilder::new()
                .respect_gitignore(false)
                .show_hidden(true)
                .show_ignored(true)
                .use_default_ignores(false)
                .build();
            let (_, substats) = scan_with_config(&node.path, subconfig)?;

            // Store raw data for sorting, then convert to hex later
            dir_sizes.push((
                node.path
                    .file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or("")
                    .to_string(),
                node.path.display().to_string(),
                substats.total_size,
                substats.total_files,
            ));
        }
    }

    // Sort by size (raw u64) descending
    dir_sizes.sort_by_key(|(_, _, size, _)| std::cmp::Reverse(*size));

    // Convert to hex-formatted JSON
    let use_hex = ctx.config.hex_numbers;
    let formatted_dirs: Vec<Value> = dir_sizes
        .into_iter()
        .map(|(name, path, size, files)| {
            json!({
                "dir": name,
                "path": path,
                "size": fmt_num64(size, use_hex),
                "sz": fmt_size(size, use_hex),  // Human-readable with hex
                "files": fmt_num64(files, use_hex)
            })
        })
        .collect();

    Ok(json!({
        "content": [{
            "type": "text",
            "text": serde_json::to_string_pretty(&json!({
                "directory": path.display().to_string(),
                "subdirs": formatted_dirs
            }))?
        }]
    }))
}
