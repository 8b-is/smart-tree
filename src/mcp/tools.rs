//! MCP tools implementation for Smart Tree

use super::{is_path_allowed, McpContext};
use crate::{
    formatters::{
        ai::AiFormatter, classic::ClassicFormatter, claude::ClaudeFormatter, csv::CsvFormatter,
        digest::DigestFormatter, hex::HexFormatter, json::JsonFormatter, quantum::QuantumFormatter,
        semantic::SemanticFormatter, stats::StatsFormatter, tsv::TsvFormatter, Formatter,
        PathDisplayMode,
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

pub async fn handle_tools_list(_params: Option<Value>, _ctx: Arc<McpContext>) -> Result<Value> {
    let tools = vec![
        ToolDefinition {
            name: "server_info".to_string(),
            description: "Get information about the Smart Tree MCP server".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {},
                "required": []
            }),
        },
        ToolDefinition {
            name: "analyze_directory".to_string(),
            description: "Analyze a directory with smart compression. Use mode='claude' for MAXIMUM compression (10x reduction!), mode='ai' (default) for balanced output, mode='classic' for visual trees. For large directories, 'claude' mode is HIGHLY RECOMMENDED!"
                .to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "path": {
                        "type": "string",
                        "description": "Path to the directory to analyze"
                    },
                    "mode": {
                        "type": "string",
                        "enum": ["classic", "hex", "json", "ai", "stats", "csv", "tsv", "digest", "quantum", "claude", "semantic"],
                        "description": "Output format mode",
                        "default": "claude"
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
        ToolDefinition {
            name: "quick_tree".to_string(),
            description: "START HERE! Quick 3-level overview using CLAUDE mode (10x compression). Perfect for initial exploration before using analyze_directory for details. Automatically optimized for AI token efficiency!".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "path": {
                        "type": "string",
                        "description": "Path to the directory"
                    },
                    "depth": {
                        "type": "integer",
                        "description": "Maximum depth (default: 3 for quick overview)",
                        "default": 3
                    }
                },
                "required": ["path"]
            }),
        },
        ToolDefinition {
            name: "project_overview".to_string(),
            description: "Get a comprehensive project overview using CLAUDE mode compression. Provides context, structure, and key files with maximum token efficiency (10x reduction!)".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "path": {
                        "type": "string",
                        "description": "Path to the project root"
                    }
                },
                "required": ["path"]
            }),
        },
        ToolDefinition {
            name: "find_code_files".to_string(),
            description: "Find all code files in a project by common programming languages".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "path": {
                        "type": "string",
                        "description": "Path to search in"
                    },
                    "languages": {
                        "type": "array",
                        "items": {
                            "type": "string",
                            "enum": ["python", "javascript", "typescript", "rust", "go", "java", "cpp", "c", "ruby", "php", "swift", "kotlin", "scala", "r", "julia", "all"]
                        },
                        "description": "Programming languages to search for",
                        "default": ["all"]
                    }
                },
                "required": ["path"]
            }),
        },
        ToolDefinition {
            name: "find_config_files".to_string(),
            description: "Find all configuration files (json, yaml, toml, ini, env, etc.)".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "path": {
                        "type": "string",
                        "description": "Path to search in"
                    }
                },
                "required": ["path"]
            }),
        },
        ToolDefinition {
            name: "find_documentation".to_string(),
            description: "Find all documentation files (README, markdown, rst, txt docs)".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "path": {
                        "type": "string",
                        "description": "Path to search in"
                    }
                },
                "required": ["path"]
            }),
        },
        ToolDefinition {
            name: "search_in_files".to_string(),
            description: "Search for content within files (like grep but AI-friendly output)".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "path": {
                        "type": "string",
                        "description": "Path to search in"
                    },
                    "keyword": {
                        "type": "string",
                        "description": "Keyword or phrase to search for"
                    },
                    "file_type": {
                        "type": "string",
                        "description": "Limit search to specific file types"
                    },
                    "case_sensitive": {
                        "type": "boolean",
                        "description": "Case sensitive search",
                        "default": false
                    }
                },
                "required": ["path", "keyword"]
            }),
        },
        ToolDefinition {
            name: "find_large_files".to_string(),
            description: "Find files larger than a specified size".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "path": {
                        "type": "string",
                        "description": "Path to search in"
                    },
                    "min_size": {
                        "type": "string",
                        "description": "Minimum size (e.g., '10M', '1G')",
                        "default": "10M"
                    }
                },
                "required": ["path"]
            }),
        },
        ToolDefinition {
            name: "find_recent_changes".to_string(),
            description: "Find recently modified files".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "path": {
                        "type": "string",
                        "description": "Path to search in"
                    },
                    "days": {
                        "type": "integer",
                        "description": "Files modified within last N days",
                        "default": 7
                    }
                },
                "required": ["path"]
            }),
        },
        ToolDefinition {
            name: "compare_directories".to_string(),
            description: "Compare two directories and show differences".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "path1": {
                        "type": "string",
                        "description": "First directory path"
                    },
                    "path2": {
                        "type": "string",
                        "description": "Second directory path"
                    }
                },
                "required": ["path1", "path2"]
            }),
        },
        ToolDefinition {
            name: "get_git_status".to_string(),
            description: "Get git repository status and structure (if directory is a git repo)".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "path": {
                        "type": "string",
                        "description": "Path to the git repository"
                    }
                },
                "required": ["path"]
            }),
        },
        ToolDefinition {
            name: "find_duplicates".to_string(),
            description: "Find duplicate files based on size and name patterns".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "path": {
                        "type": "string",
                        "description": "Path to search in"
                    }
                },
                "required": ["path"]
            }),
        },
        ToolDefinition {
            name: "analyze_workspace".to_string(),
            description: "Analyze a development workspace and identify project structure, build files, dependencies".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "path": {
                        "type": "string",
                        "description": "Path to the workspace"
                    }
                },
                "required": ["path"]
            }),
        },
        ToolDefinition {
            name: "find_tests".to_string(),
            description: "Find all test files in a project".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "path": {
                        "type": "string",
                        "description": "Path to search in"
                    }
                },
                "required": ["path"]
            }),
        },
        ToolDefinition {
            name: "find_build_files".to_string(),
            description: "Find build configuration files (Makefile, CMake, Cargo.toml, package.json, etc.)".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "path": {
                        "type": "string",
                        "description": "Path to search in"
                    }
                },
                "required": ["path"]
            }),
        },
        ToolDefinition {
            name: "directory_size_breakdown".to_string(),
            description: "Get size breakdown of subdirectories to identify space usage".to_string(),
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
        ToolDefinition {
            name: "find_empty_directories".to_string(),
            description: "Find all empty directories".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "path": {
                        "type": "string",
                        "description": "Path to search in"
                    }
                },
                "required": ["path"]
            }),
        },
        ToolDefinition {
            name: "semantic_analysis".to_string(),
            description: "Group files by semantic similarity (inspired by Omni!). Uses wave-based analysis to categorize files by their conceptual purpose: Documentation, Source Code, Tests, Configuration, etc.".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "path": {
                        "type": "string",
                        "description": "Path to analyze"
                    },
                    "max_depth": {
                        "type": "integer",
                        "description": "Maximum depth to traverse",
                        "default": 10
                    },
                    "show_wave_signatures": {
                        "type": "boolean",
                        "description": "Show wave signatures for each category",
                        "default": true
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

pub async fn handle_tools_call(params: Value, ctx: Arc<McpContext>) -> Result<Value> {
    let tool_name = params["name"]
        .as_str()
        .ok_or_else(|| anyhow::anyhow!("Missing tool name"))?;
    let args = params["arguments"].clone();

    match tool_name {
        "server_info" => server_info(args, ctx).await,
        "analyze_directory" => analyze_directory(args, ctx).await,
        "find_files" => find_files(args, ctx).await,
        "get_statistics" => get_statistics(args, ctx).await,
        "get_digest" => get_digest(args, ctx).await,
        "quick_tree" => quick_tree(args, ctx).await,
        "project_overview" => project_overview(args, ctx).await,
        "find_code_files" => find_code_files(args, ctx).await,
        "find_config_files" => find_config_files(args, ctx).await,
        "find_documentation" => find_documentation(args, ctx).await,
        "search_in_files" => search_in_files(args, ctx).await,
        "find_large_files" => find_large_files(args, ctx).await,
        "find_recent_changes" => find_recent_changes(args, ctx).await,
        "compare_directories" => compare_directories(args, ctx).await,
        "get_git_status" => get_git_status(args, ctx).await,
        "find_duplicates" => find_duplicates(args, ctx).await,
        "analyze_workspace" => analyze_workspace(args, ctx).await,
        "find_tests" => find_tests(args, ctx).await,
        "find_build_files" => find_build_files(args, ctx).await,
        "directory_size_breakdown" => directory_size_breakdown(args, ctx).await,
        "find_empty_directories" => find_empty_directories(args, ctx).await,
        "semantic_analysis" => semantic_analysis(args, ctx).await,
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
    "claude".to_string()
}

fn default_max_depth() -> usize {
    10
}

fn default_path_mode() -> String {
    "off".to_string()
}

async fn server_info(_args: Value, ctx: Arc<McpContext>) -> Result<Value> {
    let cache_stats = ctx.cache.stats().await;

    let info = json!({
        "server": {
            "name": "Smart Tree MCP Server",
            "version": env!("CARGO_PKG_VERSION"),
            "build": {
                "name": env!("CARGO_PKG_NAME"),
                "description": env!("CARGO_PKG_DESCRIPTION"),
                "authors": env!("CARGO_PKG_AUTHORS"),
                "repository": env!("CARGO_PKG_REPOSITORY"),
                "rust_version": env!("CARGO_PKG_RUST_VERSION"),
            },
            "protocol": {
                "version": "1.0",
                "features": ["tools", "resources", "prompts", "notifications"],
            },
        },
        "capabilities": {
            "output_formats": [
                "classic", "hex", "json", "ai", "stats", "csv", "tsv", "digest",
                "quantum", "claude", "semantic"
            ],
            "compression": {
                "supported": true,
                "formats": ["zlib", "quantum", "base64"],
            },
            "streaming": {
                "supported": true,
                "formats": ["hex", "ai", "quantum", "claude"],
            },
            "search": {
                "content_search": true,
                "pattern_matching": true,
                "regex_support": true,
            },
        },
        "configuration": {
            "cache": {
                "enabled": ctx.config.cache_enabled,
                "ttl_seconds": ctx.config.cache_ttl,
                "max_size_bytes": ctx.config.max_cache_size,
                "current_entries": cache_stats.entries,
                "current_size_bytes": cache_stats.size,
                "hit_rate": format!("{:.1}%", cache_stats.hit_rate * 100.0),
            },
            "security": {
                "allowed_paths": ctx.config.allowed_paths.len(),
                "blocked_paths": ctx.config.blocked_paths.len(),
                "default_blocks": ["/etc", "/sys", "/proc"],
            },
        },
        "features": {
            "quantum_compression": {
                "description": "Ultra-compressed binary format with 90%+ compression",
                "status": "active",
                "notes": "Base64-encoded for JSON transport in MCP",
            },
            "claude_format": {
                "description": "API-optimized format with quantum compression",
                "status": "active",
                "recommended_for": ["LLM APIs", "Claude", "GPT-4"],
            },
            "tokenization": {
                "description": "Semantic tokenization for common patterns",
                "tokens": {
                    "directories": ["node_modules=0x80", ".git=0x81", "src=0x82"],
                    "extensions": [".js=0x90", ".rs=0x91", ".py=0x92"],
                },
            },
        },
        "statistics": {
            "uptime_seconds": 0, // Would need to track this
            "requests_handled": 0, // Would need to track this
            "cache_hits": cache_stats.hits,
            "cache_misses": cache_stats.misses,
        },
        "tips": [
            "Use 'claude' format for optimal LLM API transmission",
            "Enable compression with compress=true for large directories",
            "Use 'quantum' format for maximum compression (90%+ reduction)",
            "Stream mode available for very large directories",
            "Content search supported with 'search_in_files' tool",
        ],
    });

    // Convert to pretty JSON string
    let json_string = serde_json::to_string_pretty(&info)?;

    // Return in MCP content format
    Ok(json!({
        "content": [{
            "type": "text",
            "text": json_string
        }]
    }))
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
        "classic" => Box::new(ClassicFormatter::new(
            args.no_emoji,
            true,
            path_display_mode,
        )),
        "hex" => Box::new(HexFormatter::new(
            true,
            args.no_emoji,
            args.show_ignored,
            path_display_mode,
            false,
        )),
        "json" => Box::new(JsonFormatter::new(false)),
        "ai" => Box::new(AiFormatter::new(args.no_emoji, path_display_mode)),
        "stats" => Box::new(StatsFormatter::new()),
        "csv" => Box::new(CsvFormatter::new()),
        "tsv" => Box::new(TsvFormatter::new()),
        "digest" => Box::new(DigestFormatter::new()),
        "quantum" => Box::new(QuantumFormatter::new()),
        "claude" => Box::new(ClaudeFormatter::new(true)),
        "semantic" => Box::new(SemanticFormatter::new(path_display_mode, args.no_emoji)),
        _ => return Err(anyhow::anyhow!("Invalid mode: {}", args.mode)),
    };

    // Format output
    let mut output = Vec::new();
    formatter.format(&mut output, &nodes, &stats, &path)?;

    // Handle different output formats
    let final_output = if args.mode == "quantum" {
        // Quantum format contains binary data, so base64-encode it for JSON safety
        use base64::{engine::general_purpose, Engine as _};
        format!(
            "QUANTUM_BASE64:{}",
            general_purpose::STANDARD.encode(&output)
        )
    } else {
        // For other formats, convert to string first
        let output_str = String::from_utf8(output)?;

        if args.compress {
            use flate2::write::ZlibEncoder;
            use flate2::Compression;
            use std::io::Write;

            let mut encoder = ZlibEncoder::new(Vec::new(), Compression::default());
            encoder.write_all(output_str.as_bytes())?;
            let compressed = encoder.finish()?;
            format!("COMPRESSED_V1:{}", hex::encode(&compressed))
        } else {
            output_str
        }
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
        Ok(SystemTime::UNIX_EPOCH
            + std::time::Duration::from_secs(datetime.and_utc().timestamp() as u64))
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
        newer_than: args
            .newer_than
            .as_ref()
            .map(|d| parse_date(d))
            .transpose()?,
        older_than: args
            .older_than
            .as_ref()
            .map(|d| parse_date(d))
            .transpose()?,
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

// New tool implementations

async fn quick_tree(args: Value, ctx: Arc<McpContext>) -> Result<Value> {
    let analyze_args = json!({
        "path": args["path"],
        "mode": "claude",
        "max_depth": args["depth"].as_u64().unwrap_or(3),
        "compress": true,
        "show_ignored": true
    });
    analyze_directory(analyze_args, ctx).await
}

async fn project_overview(args: Value, ctx: Arc<McpContext>) -> Result<Value> {
    let path = args["path"]
        .as_str()
        .ok_or_else(|| anyhow::anyhow!("Missing path"))?;

    // First get the Claude format overview (10x compression!)
    let ai_result = analyze_directory(
        json!({
            "path": path,
            "mode": "claude",
            "max_depth": 5,
            "show_ignored": true
        }),
        ctx.clone(),
    )
    .await?;

    // Then get statistics
    let stats_result = get_statistics(
        json!({
            "path": path,
            "show_hidden": false
        }),
        ctx.clone(),
    )
    .await?;

    // Combine results
    let ai_text = ai_result["content"][0]["text"].as_str().unwrap_or("");
    let stats_text = stats_result["content"][0]["text"].as_str().unwrap_or("");

    Ok(json!({
        "content": [{
            "type": "text",
            "text": format!("PROJECT OVERVIEW\n\n{}\n\nDETAILED STATISTICS:\n{}", ai_text, stats_text)
        }]
    }))
}

async fn find_code_files(args: Value, ctx: Arc<McpContext>) -> Result<Value> {
    let path = args["path"]
        .as_str()
        .ok_or_else(|| anyhow::anyhow!("Missing path"))?;
    let languages = args["languages"]
        .as_array()
        .map(|arr| arr.iter().filter_map(|v| v.as_str()).collect::<Vec<_>>())
        .unwrap_or_else(|| vec!["all"]);

    let extensions = if languages.contains(&"all") {
        vec![
            "py", "js", "ts", "tsx", "jsx", "rs", "go", "java", "cpp", "c", "h", "hpp", "rb",
            "php", "swift", "kt", "scala", "r", "jl", "cs", "vb", "lua", "pl", "sh", "bash", "zsh",
            "ps1", "dart", "elm", "ex", "exs", "clj", "cljs", "ml", "mli",
        ]
    } else {
        let mut exts = Vec::new();
        for lang in languages {
            match lang {
                "python" => exts.extend(&["py", "pyw", "pyx"]),
                "javascript" => exts.extend(&["js", "mjs", "cjs"]),
                "typescript" => exts.extend(&["ts", "tsx"]),
                "rust" => exts.push("rs"),
                "go" => exts.push("go"),
                "java" => exts.push("java"),
                "cpp" => exts.extend(&["cpp", "cxx", "cc", "c++", "hpp", "h", "hxx"]),
                "c" => exts.extend(&["c", "h"]),
                "ruby" => exts.push("rb"),
                "php" => exts.push("php"),
                "swift" => exts.push("swift"),
                "kotlin" => exts.extend(&["kt", "kts"]),
                "scala" => exts.extend(&["scala", "sc"]),
                "r" => exts.push("r"),
                "julia" => exts.push("jl"),
                _ => {}
            }
        }
        exts
    };

    let pattern = format!(r"\.({})$", extensions.join("|"));
    find_files(
        json!({
            "path": path,
            "pattern": pattern,
            "max_depth": 20
        }),
        ctx,
    )
    .await
}

async fn find_config_files(args: Value, ctx: Arc<McpContext>) -> Result<Value> {
    let path = args["path"]
        .as_str()
        .ok_or_else(|| anyhow::anyhow!("Missing path"))?;

    let pattern =
        r"\.(json|yaml|yml|toml|ini|cfg|conf|config|env|properties|xml)$|^\..*rc$|^.*config.*$";
    find_files(
        json!({
            "path": path,
            "pattern": pattern,
            "max_depth": 10
        }),
        ctx,
    )
    .await
}

async fn find_documentation(args: Value, ctx: Arc<McpContext>) -> Result<Value> {
    let path = args["path"]
        .as_str()
        .ok_or_else(|| anyhow::anyhow!("Missing path"))?;

    let pattern = r"(README|readme|CHANGELOG|changelog|LICENSE|license|CONTRIBUTING|contributing|TODO|todo|INSTALL|install|AUTHORS|authors|NOTICE|notice|HISTORY|history)(\.(md|markdown|rst|txt|adoc|org))?$|\.(md|markdown|rst|txt|adoc|org)$";
    find_files(
        json!({
            "path": path,
            "pattern": pattern,
            "max_depth": 10
        }),
        ctx,
    )
    .await
}

async fn search_in_files(args: Value, ctx: Arc<McpContext>) -> Result<Value> {
    let path = PathBuf::from(
        args["path"]
            .as_str()
            .ok_or_else(|| anyhow::anyhow!("Missing path"))?,
    );
    let keyword = args["keyword"]
        .as_str()
        .ok_or_else(|| anyhow::anyhow!("Missing keyword"))?;
    let file_type = args["file_type"].as_str();

    // Security check
    if !is_path_allowed(&path, &ctx.config) {
        return Err(anyhow::anyhow!("Access denied: path not allowed"));
    }

    let config = ScannerConfig {
        max_depth: 10,
        follow_symlinks: false,
        respect_gitignore: true,
        show_hidden: false,
        show_ignored: false,
        find_pattern: None,
        file_type_filter: file_type.map(String::from),
        min_size: None,
        max_size: None,
        newer_than: None,
        older_than: None,
        use_default_ignores: true,
        search_keyword: Some(keyword.to_string()),
        show_filesystems: false,
    };

    let scanner = Scanner::new(&path, config)?;
    let (nodes, _) = scanner.scan()?;

    // Format results showing files with matches
    let mut results = Vec::new();
    for node in &nodes {
        if let Some(matches) = &node.search_matches {
            results.push(json!({
                "file": node.path.display().to_string(),
                "first_match_line": matches.first_match.0,
                "first_match_column": matches.first_match.1,
                "total_matches": matches.total_count,
                "truncated": matches.truncated
            }));
        }
    }

    Ok(json!({
        "content": [{
            "type": "text",
            "text": serde_json::to_string_pretty(&json!({
                "keyword": keyword,
                "files_with_matches": results.len(),
                "matches": results
            }))?
        }]
    }))
}

async fn find_large_files(args: Value, ctx: Arc<McpContext>) -> Result<Value> {
    let path = args["path"]
        .as_str()
        .ok_or_else(|| anyhow::anyhow!("Missing path"))?;
    let min_size = args["min_size"].as_str().unwrap_or("10M");

    find_files(
        json!({
            "path": path,
            "min_size": min_size,
            "max_depth": 20
        }),
        ctx,
    )
    .await
}

async fn find_recent_changes(args: Value, ctx: Arc<McpContext>) -> Result<Value> {
    let path = args["path"]
        .as_str()
        .ok_or_else(|| anyhow::anyhow!("Missing path"))?;
    let days = args["days"].as_u64().unwrap_or(7);

    // Calculate date N days ago
    use chrono::{Duration, Utc};
    let date = Utc::now() - Duration::days(days as i64);
    let date_str = date.format("%Y-%m-%d").to_string();

    find_files(
        json!({
            "path": path,
            "newer_than": date_str,
            "max_depth": 20
        }),
        ctx,
    )
    .await
}

async fn compare_directories(args: Value, ctx: Arc<McpContext>) -> Result<Value> {
    let path1 = PathBuf::from(
        args["path1"]
            .as_str()
            .ok_or_else(|| anyhow::anyhow!("Missing path1"))?,
    );
    let path2 = PathBuf::from(
        args["path2"]
            .as_str()
            .ok_or_else(|| anyhow::anyhow!("Missing path2"))?,
    );

    // Security checks
    if !is_path_allowed(&path1, &ctx.config) || !is_path_allowed(&path2, &ctx.config) {
        return Err(anyhow::anyhow!("Access denied: path not allowed"));
    }

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

async fn get_git_status(args: Value, ctx: Arc<McpContext>) -> Result<Value> {
    let path = PathBuf::from(
        args["path"]
            .as_str()
            .ok_or_else(|| anyhow::anyhow!("Missing path"))?,
    );

    // Security check
    if !is_path_allowed(&path, &ctx.config) {
        return Err(anyhow::anyhow!("Access denied: path not allowed"));
    }

    // Check if it's a git repository
    let git_dir = path.join(".git");
    if !git_dir.exists() {
        return Ok(json!({
            "content": [{
                "type": "text",
                "text": "Not a git repository"
            }]
        }));
    }

    // Get tree excluding .git directory
    let tree_result = analyze_directory(
        json!({
            "path": path.display().to_string(),
            "mode": "ai",
            "max_depth": 5,
            "show_ignored": true
        }),
        ctx.clone(),
    )
    .await?;

    Ok(json!({
        "content": [{
            "type": "text",
            "text": format!(
                "GIT REPOSITORY STRUCTURE\nPath: {}\n\n{}",
                path.display(),
                tree_result["content"][0]["text"].as_str().unwrap_or("")
            )
        }]
    }))
}

async fn find_duplicates(args: Value, ctx: Arc<McpContext>) -> Result<Value> {
    let path = PathBuf::from(
        args["path"]
            .as_str()
            .ok_or_else(|| anyhow::anyhow!("Missing path"))?,
    );

    // Security check
    if !is_path_allowed(&path, &ctx.config) {
        return Err(anyhow::anyhow!("Access denied: path not allowed"));
    }

    // Get all files
    let config = ScannerConfig {
        max_depth: 20,
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

    let scanner = Scanner::new(&path, config)?;
    let (nodes, _) = scanner.scan()?;

    // Group files by size and name
    use std::collections::HashMap;
    let mut size_groups: HashMap<u64, Vec<&crate::scanner::FileNode>> = HashMap::new();

    for node in &nodes {
        if !node.is_dir {
            size_groups.entry(node.size).or_default().push(node);
        }
    }

    // Find potential duplicates
    let mut duplicates = Vec::new();
    for (size, files) in size_groups.iter() {
        if files.len() > 1 && *size > 0 {
            duplicates.push(json!({
                "size": size,
                "count": files.len(),
                "files": files.iter().map(|f| f.path.display().to_string()).collect::<Vec<_>>()
            }));
        }
    }

    Ok(json!({
        "content": [{
            "type": "text",
            "text": serde_json::to_string_pretty(&json!({
                "potential_duplicate_groups": duplicates.len(),
                "duplicates": duplicates
            }))?
        }]
    }))
}

async fn analyze_workspace(args: Value, ctx: Arc<McpContext>) -> Result<Value> {
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

async fn find_tests(args: Value, ctx: Arc<McpContext>) -> Result<Value> {
    let path = args["path"]
        .as_str()
        .ok_or_else(|| anyhow::anyhow!("Missing path"))?;

    let pattern = r"(test_|_test\.|\.test\.|tests?\.|spec\.|\.spec\.|_spec\.)|(/tests?/|/specs?/)";
    find_files(
        json!({
            "path": path,
            "pattern": pattern,
            "max_depth": 20
        }),
        ctx,
    )
    .await
}

async fn find_build_files(args: Value, ctx: Arc<McpContext>) -> Result<Value> {
    let path = args["path"]
        .as_str()
        .ok_or_else(|| anyhow::anyhow!("Missing path"))?;

    let pattern = r"^(Makefile|makefile|CMakeLists\.txt|Cargo\.toml|package\.json|pom\.xml|build\.gradle|build\.sbt|setup\.py|requirements\.txt|Gemfile|go\.mod|composer\.json|Dockerfile|docker-compose\.yml)$";
    find_files(
        json!({
            "path": path,
            "pattern": pattern,
            "max_depth": 10
        }),
        ctx,
    )
    .await
}

async fn directory_size_breakdown(args: Value, ctx: Arc<McpContext>) -> Result<Value> {
    let path = PathBuf::from(
        args["path"]
            .as_str()
            .ok_or_else(|| anyhow::anyhow!("Missing path"))?,
    );

    // Security check
    if !is_path_allowed(&path, &ctx.config) {
        return Err(anyhow::anyhow!("Access denied: path not allowed"));
    }

    // Get immediate subdirectories
    let config = ScannerConfig {
        max_depth: 1,
        follow_symlinks: false,
        respect_gitignore: false,
        show_hidden: true,
        show_ignored: true,
        find_pattern: None,
        file_type_filter: None,
        min_size: None,
        max_size: None,
        newer_than: None,
        older_than: None,
        use_default_ignores: false,
        search_keyword: None,
        show_filesystems: false,
    };

    let scanner = Scanner::new(&path, config)?;
    let (nodes, _) = scanner.scan()?;

    // Calculate size for each subdirectory
    let mut dir_sizes = Vec::new();
    for node in &nodes {
        if node.is_dir && node.path != path {
            // Get size of this directory
            let subconfig = ScannerConfig {
                max_depth: 100,
                follow_symlinks: false,
                respect_gitignore: false,
                show_hidden: true,
                show_ignored: true,
                find_pattern: None,
                file_type_filter: None,
                min_size: None,
                max_size: None,
                newer_than: None,
                older_than: None,
                use_default_ignores: false,
                search_keyword: None,
                show_filesystems: false,
            };
            let subscanner = Scanner::new(&node.path, subconfig)?;
            let (_, substats) = subscanner.scan()?;

            dir_sizes.push(json!({
                "directory": node.path.file_name().and_then(|n| n.to_str()).unwrap_or(""),
                "path": node.path.display().to_string(),
                "size": substats.total_size,
                "size_human": format!("{:.2} MB", substats.total_size as f64 / 1_048_576.0),
                "file_count": substats.total_files
            }));
        }
    }

    // Sort by size
    dir_sizes.sort_by_key(|d| -(d["size"].as_u64().unwrap_or(0) as i64));

    Ok(json!({
        "content": [{
            "type": "text",
            "text": serde_json::to_string_pretty(&json!({
                "directory": path.display().to_string(),
                "subdirectories": dir_sizes
            }))?
        }]
    }))
}

async fn find_empty_directories(args: Value, ctx: Arc<McpContext>) -> Result<Value> {
    let path = PathBuf::from(
        args["path"]
            .as_str()
            .ok_or_else(|| anyhow::anyhow!("Missing path"))?,
    );

    // Security check
    if !is_path_allowed(&path, &ctx.config) {
        return Err(anyhow::anyhow!("Access denied: path not allowed"));
    }

    let config = ScannerConfig {
        max_depth: 20,
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

    let scanner = Scanner::new(&path, config)?;
    let (nodes, _) = scanner.scan()?;

    // Find directories with no children
    let mut empty_dirs = Vec::new();
    let mut dir_children: std::collections::HashMap<PathBuf, usize> =
        std::collections::HashMap::new();

    // Count children for each directory
    for node in &nodes {
        if let Some(parent) = node.path.parent() {
            *dir_children.entry(parent.to_path_buf()).or_insert(0) += 1;
        }
    }

    // Find empty directories
    for node in &nodes {
        if node.is_dir {
            let child_count = dir_children.get(&node.path).unwrap_or(&0);
            if *child_count == 0 {
                empty_dirs.push(node.path.display().to_string());
            }
        }
    }

    Ok(json!({
        "content": [{
            "type": "text",
            "text": serde_json::to_string_pretty(&json!({
                "empty_directory_count": empty_dirs.len(),
                "empty_directories": empty_dirs
            }))?
        }]
    }))
}

async fn semantic_analysis(args: Value, ctx: Arc<McpContext>) -> Result<Value> {
    let path = args["path"]
        .as_str()
        .ok_or_else(|| anyhow::anyhow!("Missing path"))?;
    let max_depth = args["max_depth"].as_u64().unwrap_or(10) as usize;

    // Simply use analyze_directory with semantic mode
    analyze_directory(
        json!({
            "path": path,
            "mode": "semantic",
            "max_depth": max_depth,
            "no_emoji": false,
            "path_mode": "off"
        }),
        ctx,
    )
    .await
}
