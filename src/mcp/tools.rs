//! MCP tools implementation for Smart Tree

use super::{is_path_allowed, McpContext};
use crate::{
    formatters::{
        ai::AiFormatter, classic::ClassicFormatter, csv::CsvFormatter, digest::DigestFormatter,
        hex::HexFormatter, json::JsonFormatter, stats::StatsFormatter, tsv::TsvFormatter,
        Formatter, PathDisplayMode,
    },
    parse_size, Scanner, ScannerConfig,
};
use anyhow::Result;
use regex::Regex;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::path::PathBuf;
use std::sync::Arc;
use std::time::SystemTime;

#[derive(Debug, Serialize, Deserialize)]
struct ToolDefinition {
    name: String,
    description: String,
    #[serde(rename = "inputSchema")]
    input_schema: Value,
}

pub async fn handle_tools_list(
    _params: Option<Value>,
    _ctx: Arc<McpContext>,
) -> Result<Value> {
    let tools = vec![
        ToolDefinition {
            name: "analyze_directory".to_string(),
            description: "Analyze a directory and return its structure in various formats".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "path": {
                        "type": "string",
                        "description": "Path to the directory to analyze"
                    },
                    "mode": {
                        "type": "string",
                        "enum": ["classic", "hex", "json", "ai", "stats", "csv", "tsv", "digest"],
                        "description": "Output format mode",
                        "default": "ai"
                    },
                    "max_depth": {
                        "type": "integer",
                        "description": "Maximum depth to traverse",
                        "default": 10
                    },
                    "show_hidden": {
                        "type": "boolean",
                        "description": "Show hidden files and directories",
                        "default": false
                    },
                    "show_ignored": {
                        "type": "boolean",
                        "description": "Show ignored directories in brackets",
                        "default": false
                    },
                    "no_emoji": {
                        "type": "boolean",
                        "description": "Disable emoji in output",
                        "default": false
                    },
                    "compress": {
                        "type": "boolean",
                        "description": "Compress output with zlib",
                        "default": false
                    },
                    "path_mode": {
                        "type": "string",
                        "enum": ["off", "relative", "full"],
                        "description": "Path display mode",
                        "default": "off"
                    }
                },
                "required": ["path"]
            }),
        },
        ToolDefinition {
            name: "find_files".to_string(),
            description: "Find files matching specific criteria".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "path": {
                        "type": "string",
                        "description": "Path to search in"
                    },
                    "pattern": {
                        "type": "string",
                        "description": "Regex pattern to match file/directory names"
                    },
                    "file_type": {
                        "type": "string",
                        "description": "Filter by file extension (e.g., 'rs', 'py')"
                    },
                    "min_size": {
                        "type": "string",
                        "description": "Minimum file size (e.g., '1M', '500K')"
                    },
                    "max_size": {
                        "type": "string",
                        "description": "Maximum file size"
                    },
                    "newer_than": {
                        "type": "string",
                        "description": "Show files newer than date (YYYY-MM-DD)"
                    },
                    "older_than": {
                        "type": "string",
                        "description": "Show files older than date (YYYY-MM-DD)"
                    },
                    "max_depth": {
                        "type": "integer",
                        "description": "Maximum depth to traverse",
                        "default": 10
                    }
                },
                "required": ["path"]
            }),
        },
        ToolDefinition {
            name: "get_statistics".to_string(),
            description: "Get detailed statistics about a directory".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "path": {
                        "type": "string",
                        "description": "Path to analyze"
                    },
                    "show_hidden": {
                        "type": "boolean",
                        "description": "Include hidden files in statistics",
                        "default": false
                    }
                },
                "required": ["path"]
            }),
        },
        ToolDefinition {
            name: "get_digest".to_string(),
            description: "Get SHA256 digest of directory structure".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "path": {
                        "type": "string",
                        "description": "Path to analyze"
                    }
                },
                "required": ["path"]
            }),
        },
    ];

    Ok(json!({
        "tools": tools
    }))
}

pub async fn handle_tools_call(
    params: Value,
    ctx: Arc<McpContext>,
) -> Result<Value> {
    let tool_name = params["name"]
        .as_str()
        .ok_or_else(|| anyhow::anyhow!("Missing tool name"))?;
    let args = params["arguments"].clone();

    match tool_name {
        "analyze_directory" => analyze_directory(args, ctx).await,
        "find_files" => find_files(args, ctx).await,
        "get_statistics" => get_statistics(args, ctx).await,
        "get_digest" => get_digest(args, ctx).await,
        _ => Err(anyhow::anyhow!("Unknown tool: {}", tool_name)),
    }
}

#[derive(Debug, Deserialize)]
struct AnalyzeDirectoryArgs {
    path: String,
    #[serde(default = "default_mode")]
    mode: String,
    #[serde(default = "default_max_depth")]
    max_depth: usize,
    #[serde(default)]
    show_hidden: bool,
    #[serde(default)]
    show_ignored: bool,
    #[serde(default)]
    no_emoji: bool,
    #[serde(default)]
    compress: bool,
    #[serde(default = "default_path_mode")]
    path_mode: String,
}

fn default_mode() -> String {
    "ai".to_string()
}

fn default_max_depth() -> usize {
    10
}

fn default_path_mode() -> String {
    "off".to_string()
}

async fn analyze_directory(args: Value, ctx: Arc<McpContext>) -> Result<Value> {
    let args: AnalyzeDirectoryArgs = serde_json::from_value(args)?;
    let path = PathBuf::from(&args.path);

    // Security check
    if !is_path_allowed(&path, &ctx.config) {
        return Err(anyhow::anyhow!("Access denied: path not allowed"));
    }

    // Check cache if enabled
    let cache_key = format!(
        "{}:{}:{}:{}:{}:{}",
        path.display(),
        args.mode,
        args.max_depth,
        args.show_hidden,
        args.show_ignored,
        args.path_mode
    );

    if ctx.config.cache_enabled {
        if let Some(cached) = ctx.cache.get(&cache_key).await {
            return Ok(json!({
                "content": [{
                    "type": "text",
                    "text": cached
                }]
            }));
        }
    }

    // Build scanner configuration
    let config = ScannerConfig {
        max_depth: args.max_depth,
        follow_symlinks: false,
        respect_gitignore: true,
        show_hidden: args.show_hidden,
        show_ignored: args.show_ignored || args.mode == "ai",
        find_pattern: None,
        file_type_filter: None,
        min_size: None,
        max_size: None,
        newer_than: None,
        older_than: None,
        use_default_ignores: true,
        search_keyword: None,
        show_filesystems: false,
    };

    // Scan directory
    let scanner = Scanner::new(&path, config)?;
    let (nodes, stats) = scanner.scan()?;

    // Convert path mode
    let path_display_mode = match args.path_mode.as_str() {
        "relative" => PathDisplayMode::Relative,
        "full" => PathDisplayMode::Full,
        _ => PathDisplayMode::Off,
    };

    // Create formatter
    let formatter: Box<dyn Formatter> = match args.mode.as_str() {
        "classic" => Box::new(ClassicFormatter::new(args.no_emoji, true, path_display_mode)),
        "hex" => Box::new(HexFormatter::new(true, args.no_emoji, args.show_ignored, path_display_mode, false)),
        "json" => Box::new(JsonFormatter::new(false)),
        "ai" => Box::new(AiFormatter::new(args.no_emoji, path_display_mode)),
        "stats" => Box::new(StatsFormatter::new()),
        "csv" => Box::new(CsvFormatter::new()),
        "tsv" => Box::new(TsvFormatter::new()),
        "digest" => Box::new(DigestFormatter::new()),
        _ => return Err(anyhow::anyhow!("Invalid mode: {}", args.mode)),
    };

    // Format output
    let mut output = Vec::new();
    formatter.format(&mut output, &nodes, &stats, &path)?;

    let output_str = String::from_utf8(output)?;

    // Handle compression
    let final_output = if args.compress {
        use flate2::write::ZlibEncoder;
        use flate2::Compression;
        use std::io::Write;

        let mut encoder = ZlibEncoder::new(Vec::new(), Compression::default());
        encoder.write_all(output_str.as_bytes())?;
        let compressed = encoder.finish()?;
        format!("COMPRESSED_V1:{}", hex::encode(&compressed))
    } else {
        output_str
    };

    // Cache result if enabled
    if ctx.config.cache_enabled {
        ctx.cache.set(cache_key, final_output.clone()).await;
    }

    Ok(json!({
        "content": [{
            "type": "text",
            "text": final_output
        }]
    }))
}

#[derive(Debug, Deserialize)]
struct FindFilesArgs {
    path: String,
    pattern: Option<String>,
    file_type: Option<String>,
    min_size: Option<String>,
    max_size: Option<String>,
    newer_than: Option<String>,
    older_than: Option<String>,
    #[serde(default = "default_max_depth")]
    max_depth: usize,
}

async fn find_files(args: Value, ctx: Arc<McpContext>) -> Result<Value> {
    let args: FindFilesArgs = serde_json::from_value(args)?;
    let path = PathBuf::from(&args.path);

    // Security check
    if !is_path_allowed(&path, &ctx.config) {
        return Err(anyhow::anyhow!("Access denied: path not allowed"));
    }

    // Parse dates
    let parse_date = |date_str: &str| -> Result<SystemTime> {
        use chrono::NaiveDate;
        let date = NaiveDate::parse_from_str(date_str, "%Y-%m-%d")?;
        let datetime = date.and_hms_opt(0, 0, 0).unwrap();
        Ok(SystemTime::UNIX_EPOCH + std::time::Duration::from_secs(datetime.and_utc().timestamp() as u64))
    };

    // Build scanner configuration
    let config = ScannerConfig {
        max_depth: args.max_depth,
        follow_symlinks: false,
        respect_gitignore: true,
        show_hidden: true,
        show_ignored: false,
        find_pattern: args.pattern.as_ref().map(|p| Regex::new(p)).transpose()?,
        file_type_filter: args.file_type,
        min_size: args.min_size.as_ref().map(|s| parse_size(s)).transpose()?,
        max_size: args.max_size.as_ref().map(|s| parse_size(s)).transpose()?,
        newer_than: args.newer_than.as_ref().map(|d| parse_date(d)).transpose()?,
        older_than: args.older_than.as_ref().map(|d| parse_date(d)).transpose()?,
        use_default_ignores: true,
        search_keyword: None,
        show_filesystems: false,
    };

    // Scan directory
    let scanner = Scanner::new(&path, config)?;
    let (nodes, _stats) = scanner.scan()?;

    // Format results as JSON list
    let mut results = Vec::new();
    for node in &nodes {
        if !node.is_dir {
            results.push(json!({
                "path": node.path.display().to_string(),
                "name": node.path.file_name().and_then(|n| n.to_str()).unwrap_or(""),
                "size": node.size,
                "modified": node.modified.duration_since(SystemTime::UNIX_EPOCH)?.as_secs(),
                "permissions": format!("{:o}", node.permissions),
            }));
        }
    }

    Ok(json!({
        "content": [{
            "type": "text",
            "text": serde_json::to_string_pretty(&json!({
                "found": results.len(),
                "files": results
            }))?
        }]
    }))
}

async fn get_statistics(args: Value, ctx: Arc<McpContext>) -> Result<Value> {
    let path = args["path"]
        .as_str()
        .ok_or_else(|| anyhow::anyhow!("Missing path"))?;
    let show_hidden = args["show_hidden"].as_bool().unwrap_or(false);
    let path = PathBuf::from(path);

    // Security check
    if !is_path_allowed(&path, &ctx.config) {
        return Err(anyhow::anyhow!("Access denied: path not allowed"));
    }

    // Build scanner configuration
    let config = ScannerConfig {
        max_depth: 100,
        follow_symlinks: false,
        respect_gitignore: true,
        show_hidden,
        show_ignored: false,
        find_pattern: None,
        file_type_filter: None,
        min_size: None,
        max_size: None,
        newer_than: None,
        older_than: None,
        use_default_ignores: true,
        search_keyword: None,
        show_filesystems: false,
    };

    // Scan directory
    let scanner = Scanner::new(&path, config)?;
    let (_nodes, stats) = scanner.scan()?;

    // Use stats formatter
    let formatter = StatsFormatter::new();
    let mut output = Vec::new();
    formatter.format(&mut output, &[], &stats, &path)?;

    Ok(json!({
        "content": [{
            "type": "text",
            "text": String::from_utf8(output)?
        }]
    }))
}

async fn get_digest(args: Value, ctx: Arc<McpContext>) -> Result<Value> {
    let path = args["path"]
        .as_str()
        .ok_or_else(|| anyhow::anyhow!("Missing path"))?;
    let path = PathBuf::from(path);

    // Security check
    if !is_path_allowed(&path, &ctx.config) {
        return Err(anyhow::anyhow!("Access denied: path not allowed"));
    }

    // Build scanner configuration
    let config = ScannerConfig {
        max_depth: 100,
        follow_symlinks: false,
        respect_gitignore: true,
        show_hidden: false,
        show_ignored: false,
        find_pattern: None,
        file_type_filter: None,
        min_size: None,
        max_size: None,
        newer_than: None,
        older_than: None,
        use_default_ignores: true,
        search_keyword: None,
        show_filesystems: false,
    };

    // Scan directory
    let scanner = Scanner::new(&path, config)?;
    let (nodes, stats) = scanner.scan()?;

    // Use digest formatter
    let formatter = DigestFormatter::new();
    let mut output = Vec::new();
    formatter.format(&mut output, &nodes, &stats, &path)?;

    Ok(json!({
        "content": [{
            "type": "text",
            "text": String::from_utf8(output)?
        }]
    }))
}