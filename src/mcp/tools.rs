//! MCP tools implementation for Smart Tree

use super::{is_path_allowed, McpContext};
use crate::{
    feedback_client::{FeedbackClient, FeedbackExample, FeedbackRequest},
    formatters::{
        ai::AiFormatter, classic::ClassicFormatter, csv::CsvFormatter, digest::DigestFormatter,
        hex::HexFormatter, json::JsonFormatter, quantum::QuantumFormatter,
        quantum_semantic::QuantumSemanticFormatter, semantic::SemanticFormatter,
        stats::StatsFormatter, summary::SummaryFormatter, summary_ai::SummaryAiFormatter,
        tsv::TsvFormatter, Formatter, PathDisplayMode,
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
            description: "Get information about the Smart Tree MCP server - shows capabilities, compression options, and performance tips. Call this to understand what Smart Tree can do for you!".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {},
                "required": []
            }),
        },
        ToolDefinition {
            name: "analyze_directory".to_string(),
            description: "🔍 The MAIN WORKHORSE - Analyze any directory with multiple output formats. Use mode='classic' for human-readable tree, 'ai' for AI-optimized format (default), 'quantum-semantic' for semantic-aware compression with tokens (HIGHLY RECOMMENDED for code analysis!), 'summary-ai' for maximum compression (10x reduction - perfect for large codebases!), 'quantum' for ultra-compressed binary, 'digest' for minimal hash. PRO TIP: Start with quick_tree for overview, then use this for details!".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "path": {
                        "type": "string",
                        "description": "Path to the directory to analyze"
                    },
                    "mode": {
                        "type": "string",
                        "enum": ["classic", "hex", "json", "ai", "stats", "csv", "tsv", "digest", "quantum", "semantic", "quantum-semantic", "summary", "summary-ai"],
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
                        "description": "Compress output with zlib. Default: true for AI modes (ai, digest, quantum, quantum-semantic, summary-ai), false for human-readable modes",
                        "default": null
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
            description: "🔎 Powerful file search with regex patterns, size filters, and date ranges. Perfect for finding specific files in large codebases. Returns structured JSON with file details. Use this when you need to locate specific files by name, type, size, or modification date.".to_string(),
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
            description: "📊 Get comprehensive statistics about a directory - file counts by type, size distribution, largest files, newest files, and more. Great for understanding project composition and identifying potential issues like large files or unusual patterns.".to_string(),
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
            description: "🔐 Get SHA256 digest of directory structure - perfect for detecting changes, verifying directory integrity, or creating unique identifiers for directory states. Super fast and efficient!".to_string(),
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
            description: "⚡ START HERE! Lightning-fast 3-level directory overview using SUMMARY-AI mode with 10x compression. Perfect for initial exploration before diving into details. This is your go-to tool for quickly understanding any codebase structure. Automatically optimized for AI token efficiency - saves you tokens while giving maximum insight!".to_string(),
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
            description: "🚀 Get a comprehensive project analysis with context detection, key files identification, and structure insights. Uses SUMMARY-AI compression for 10x token reduction! This tool automatically detects project type (Node.js, Rust, Python, etc.) and highlights important files. IDEAL for understanding new codebases quickly!".to_string(),
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
            description: "💻 Find all source code files by programming language. Supports 25+ languages including Python, JavaScript, TypeScript, Rust, Go, Java, C++, and more. Use languages=['all'] to find all code files, or specify specific languages. Returns structured JSON perfect for further analysis.".to_string(),
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
            description: "⚙️ Locate all configuration files - JSON, YAML, TOML, INI, .env, and more. Essential for understanding project setup, dependencies, and configuration. Finds package.json, Cargo.toml, requirements.txt, docker-compose.yml, and dozens of other config patterns.".to_string(),
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
            description: "📚 Find all documentation files - README, CHANGELOG, LICENSE, and any markdown/text docs. Perfect for quickly understanding project documentation structure and locating important information about setup, contribution guidelines, or API documentation.".to_string(),
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
            description: "🔍 Powerful content search within files (like grep but AI-friendly). Search for keywords, function names, TODOs, or any text pattern. Returns file locations with match counts - perfect for finding where specific functionality is implemented or tracking down references.".to_string(),
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
            description: "💾 Identify files consuming significant disk space. Default threshold is 10MB but fully customizable. Essential for optimization, cleanup, or understanding resource usage. Great for finding forgotten large assets, logs, or build artifacts.".to_string(),
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
            description: "📅 Find files modified within the last N days (default: 7). Perfect for understanding recent development activity, tracking changes, or identifying what's been worked on lately. Helps focus attention on active areas of the codebase.".to_string(),
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
            description: "🔄 Compare two directory structures to identify differences. Useful for comparing branches, versions, or similar projects. Shows what's unique to each directory and helps identify structural changes or missing files.".to_string(),
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
            description: "🌿 Analyze git repository structure (excluding .git internals). Shows the working tree with awareness of version control. Perfect for understanding project layout while respecting git boundaries. Automatically shows ignored files to give complete picture.".to_string(),
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
            description: "🔁 Detect potential duplicate files based on size and name patterns. Helps identify redundant files, backup copies, or files that could be consolidated. Groups files by size for efficient duplicate detection.".to_string(),
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
            description: "🏗️ Comprehensive development workspace analysis - identifies project type, build systems, dependencies, and structure. Combines multiple analyses into one powerful overview. PERFECT for understanding complex multi-language projects or monorepos!".to_string(),
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
            description: "🧪 Locate all test files using common naming patterns (test_, _test, .test, spec, etc.). Essential for understanding test coverage, running specific tests, or analyzing testing patterns. Searches for unit tests, integration tests, and specs across all languages.".to_string(),
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
            description: "🔨 Find all build configuration files - Makefile, CMakeLists.txt, Cargo.toml, package.json, pom.xml, and more. Critical for understanding how to build, test, and deploy the project. Covers 15+ build systems!".to_string(),
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
            description: "📊 Get size analysis of immediate subdirectories - shows which folders consume the most space. Perfect for identifying bloated directories, understanding project layout by size, or cleanup opportunities. Returns sorted list with human-readable sizes.".to_string(),
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
            description: "📂 Find all empty directories in the tree. Useful for cleanup, identifying incomplete structures, or understanding project organization. Often reveals forgotten directories or structural issues.".to_string(),
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
            description: "🧠 ADVANCED: Group files by semantic similarity using wave-based analysis (inspired by Omni!). Categorizes files by conceptual purpose: Documentation, Source Code, Tests, Configuration, etc. Uses quantum semantic compression to identify patterns. AMAZING for understanding large codebases at a conceptual level!".to_string(),
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
        ToolDefinition {
            name: "submit_feedback".to_string(),
            description: "🌮 Submit enhancement feedback to Smart Tree developers (MCP ONLY!). Help make Smart Tree the Taco Bell of directory tools - the only one to survive the franchise wars! AI assistants should provide detailed, actionable feedback with examples. This tool helps automatically enhance Smart Tree based on real usage patterns.".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "category": {
                        "type": "string",
                        "enum": ["bug", "nice_to_have", "critical"],
                        "description": "Type of feedback"
                    },
                    "title": {
                        "type": "string",
                        "description": "Brief title (max 100 chars)"
                    },
                    "description": {
                        "type": "string",
                        "description": "Detailed description of the issue/request"
                    },
                    "affected_command": {
                        "type": "string",
                        "description": "The st command that triggered this (optional)"
                    },
                    "mcp_tool": {
                        "type": "string",
                        "description": "MCP tool being used when issue found (optional)"
                    },
                    "examples": {
                        "type": "array",
                        "description": "Code examples showing the issue or desired behavior",
                        "items": {
                            "type": "object",
                            "properties": {
                                "description": {"type": "string"},
                                "code": {"type": "string"},
                                "expected_output": {"type": "string"}
                            },
                            "required": ["description", "code"]
                        }
                    },
                    "proposed_solution": {
                        "type": "string",
                        "description": "AI's suggested implementation (optional)"
                    },
                    "impact_score": {
                        "type": "integer",
                        "minimum": 1,
                        "maximum": 10,
                        "description": "Impact score 1-10"
                    },
                    "frequency_score": {
                        "type": "integer",
                        "minimum": 1,
                        "maximum": 10,
                        "description": "How often this occurs 1-10"
                    },
                    "tags": {
                        "type": "array",
                        "items": {"type": "string"},
                        "description": "Tags for categorization"
                    },
                    "auto_fixable": {
                        "type": "boolean",
                        "description": "Can this be automatically fixed by an AI?"
                    },
                    "fix_complexity": {
                        "type": "string",
                        "enum": ["trivial", "simple", "moderate", "complex"],
                        "description": "Complexity of the fix"
                    },
                    "proposed_fix": {
                        "type": "string",
                        "description": "Proposed code fix (if applicable)"
                    }
                },
                "required": ["category", "title", "description", "impact_score", "frequency_score"]
            }),
        },
        ToolDefinition {
            name: "request_tool".to_string(),
            description: "🛠️ Request a new MCP tool that doesn't exist yet (MCP ONLY!). When you need a tool that would increase your productivity but isn't available, use this to request it. The user will be asked for consent before submission. Your request helps shape Smart Tree's evolution!".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "tool_name": {
                        "type": "string",
                        "description": "Proposed tool name (e.g., 'find_symbol', 'extract_imports')"
                    },
                    "description": {
                        "type": "string",
                        "description": "What the tool should do"
                    },
                    "use_case": {
                        "type": "string",
                        "description": "Example use case demonstrating why you need this tool"
                    },
                    "proposed_parameters": {
                        "type": "object",
                        "description": "Suggested parameters for the tool",
                        "additionalProperties": {
                            "type": "object",
                            "properties": {
                                "type": {"type": "string"},
                                "description": {"type": "string"},
                                "required": {"type": "boolean"},
                                "default": {}
                            }
                        }
                    },
                    "expected_output": {
                        "type": "string",
                        "description": "What the tool should return (format and content)"
                    },
                    "productivity_impact": {
                        "type": "string",
                        "description": "How this tool would improve your productivity"
                    },
                    "consent": {
                        "type": "object",
                        "description": "User consent for submission",
                        "properties": {
                            "agreed": {
                                "type": "boolean",
                                "description": "User agreed to submit this request"
                            },
                            "anonymous": {
                                "type": "boolean",
                                "description": "Submit anonymously (true) or with GitHub credit (false)"
                            },
                            "github_url": {
                                "type": "string",
                                "description": "GitHub profile URL for credit (if not anonymous)"
                            }
                        },
                        "required": ["agreed"]
                    }
                },
                "required": ["tool_name", "description", "use_case", "expected_output", "productivity_impact", "consent"]
            }),
        },
        ToolDefinition {
            name: "check_for_updates".to_string(),
            description: "🚀 Check if a newer version of Smart Tree is available (MCP ONLY!). Shows release notes, new features, and AI-specific benefits. Helps keep your tools up-to-date for maximum productivity!".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "offer_auto_update": {
                        "type": "boolean",
                        "description": "Whether to offer automatic update if available",
                        "default": true
                    }
                },
                "required": []
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
        "submit_feedback" => submit_feedback(args, ctx).await,
        "request_tool" => request_tool(args, ctx).await,
        "check_for_updates" => check_for_updates(args, ctx).await,
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
    #[serde(default = "default_path_mode")]
    path_mode: String,
    #[serde(default)]
    compress: Option<bool>,
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
                "quantum", "semantic", "quantum-semantic", "summary", "summary-ai"
            ],
            "compression": {
                "supported": true,
                "formats": ["zlib", "quantum", "base64"],
            },
            "streaming": {
                "supported": true,
                "formats": ["hex", "ai", "quantum", "quantum-semantic"],
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
            "mcp_optimization": {
                "compression_enabled": !std::env::var("MCP_NO_COMPRESS")
                    .is_ok_and(|v| v == "1" || v.to_lowercase() == "true"),
                "emoji_disabled": true,
                "auto_ai_modes": true,
                "env_vars": {
                    "MCP_NO_COMPRESS": "Set to '1' or 'true' to disable compression for AIs that can't handle it",
                },
            },
        },
        "features": {
            "quantum_compression": {
                "description": "Ultra-compressed binary format with 90%+ compression",
                "status": "active",
                "notes": "Base64-encoded for JSON transport in MCP",
            },
            "mcp_optimization": {
                "description": "Automatic API optimization for any output mode",
                "status": "active",
                "features": ["compression (disable with MCP_NO_COMPRESS=1)", "no emoji", "AI mode selection"],
                "recommended_for": ["MCP servers", "LLM APIs", "AI assistants"],
            },
            "tokenization": {
                "description": "Semantic tokenization for common patterns",
                "tokens": {
                    "directories": ["node_modules=0x80", ".git=0x81", "src=0x82"],
                    "extensions": [".js=0x90", ".rs=0x91", ".py=0x92"],
                },
            },
        },
        "recommended_workflow": {
            "step_1": {
                "tool": "quick_tree",
                "why": "Always start here! Gets you a 3-level overview with 10x compression. Perfect for understanding the basic structure.",
                "example": "quick_tree(path='.')",
            },
            "step_2": {
                "tool": "project_overview or analyze_workspace",
                "why": "For deeper understanding of project type, dependencies, and structure. Use project_overview for single projects, analyze_workspace for complex/multi-language codebases.",
                "example": "project_overview(path='.')",
            },
            "step_3_options": {
                "for_specific_files": {
                    "tools": ["find_code_files", "find_config_files", "find_documentation", "find_tests"],
                    "why": "Use these targeted searches to locate specific file types quickly",
                },
                "for_code_analysis": {
                    "tool": "analyze_directory with mode='quantum-semantic'",
                    "why": "Best mode for understanding code structure with semantic compression and tokenization",
                },
                "for_search": {
                    "tool": "search_in_files",
                    "why": "Find specific functions, TODOs, or any text pattern across the codebase",
                },
                "for_statistics": {
                    "tool": "get_statistics",
                    "why": "Understand file distribution, sizes, and project composition",
                },
            },
            "pro_tips": [
                "Always use quick_tree first - it's optimized for initial exploration",
                "For large codebases, use mode='summary-ai' for 10x compression",
                "quantum-semantic mode is AMAZING for code understanding - try it!",
                "Cache is enabled by default - repeated calls are instant",
                "Use search_in_files to find specific implementations quickly",
                "semantic_analysis groups files by purpose - great for large projects",
            ],
        },
        "statistics": {
            "uptime_seconds": 0, // Would need to track this
            "requests_handled": 0, // Would need to track this
            "cache_hits": cache_stats.hits,
            "cache_misses": cache_stats.misses,
        },
        "tips": [
            "🌟 ALWAYS start with 'quick_tree' for any new directory!",
            "Use 'summary-ai' format for optimal LLM API transmission (10x compression!)",
            "Enable compression with compress=true for large directories",
            "Use 'quantum-semantic' mode for the best code analysis experience",
            "Stream mode available for very large directories",
            "Content search supported with 'search_in_files' tool",
            "The cache makes repeated queries instant - don't worry about calling tools multiple times!",
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

    // MCP optimizations: no emoji for clean output
    let mcp_no_emoji = true;

    // Compression logic:
    // 1. If user explicitly sets compress parameter, use that
    // 2. Otherwise, check MCP_NO_COMPRESS env var
    // 3. Default: true for AI modes, false for human-readable modes
    let default_compress = matches!(
        args.mode.as_str(),
        "ai" | "digest" | "quantum" | "quantum-semantic" | "summary-ai"
    );

    let mcp_compress = match args.compress {
        Some(compress) => compress, // User's explicit choice
        None => {
            // Check env var, otherwise use mode-based default
            if std::env::var("MCP_NO_COMPRESS")
                .is_ok_and(|v| v == "1" || v.to_lowercase() == "true")
            {
                false
            } else {
                default_compress
            }
        }
    };

    // Handle summary mode - auto-switch to AI version in MCP context
    let effective_mode = match args.mode.as_str() {
        "summary" => "summary-ai",
        other => other,
    };

    // Create formatter
    let formatter: Box<dyn Formatter> = match effective_mode {
        "classic" => Box::new(ClassicFormatter::new(mcp_no_emoji, true, path_display_mode)),
        "hex" => Box::new(HexFormatter::new(
            true,
            mcp_no_emoji,
            args.show_ignored,
            path_display_mode,
            false,
        )),
        "json" => Box::new(JsonFormatter::new(false)),
        "ai" => Box::new(AiFormatter::new(mcp_no_emoji, path_display_mode)),
        "stats" => Box::new(StatsFormatter::new()),
        "csv" => Box::new(CsvFormatter::new()),
        "tsv" => Box::new(TsvFormatter::new()),
        "digest" => Box::new(DigestFormatter::new()),
        "quantum" => Box::new(QuantumFormatter::new()),
        "semantic" => Box::new(SemanticFormatter::new(path_display_mode, mcp_no_emoji)),
        "quantum-semantic" => Box::new(QuantumSemanticFormatter::new()),
        "summary" => Box::new(SummaryFormatter::new(!mcp_no_emoji)),
        "summary-ai" => Box::new(SummaryAiFormatter::new(mcp_compress)),
        _ => return Err(anyhow::anyhow!("Invalid mode: {}", args.mode)),
    };

    // Format output
    let mut output = Vec::new();
    formatter.format(&mut output, &nodes, &stats, &path)?;

    // Handle different output formats
    let final_output = if args.mode == "quantum" || args.mode == "quantum-semantic" {
        // Quantum formats contain binary data, so base64-encode it for JSON safety
        use base64::{engine::general_purpose, Engine as _};
        format!(
            "QUANTUM_BASE64:{}",
            general_purpose::STANDARD.encode(&output)
        )
    } else {
        // For other formats, convert to string first
        let output_str = String::from_utf8(output)?;

        if mcp_compress {
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
        "mode": "summary-ai",
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

    // First get the summary-ai format overview (10x compression!)
    let ai_result = analyze_directory(
        json!({
            "path": path,
            "mode": "summary-ai",
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

async fn submit_feedback(args: Value, _ctx: Arc<McpContext>) -> Result<Value> {
    use chrono::Utc;

    // Extract required fields
    let category = args["category"]
        .as_str()
        .ok_or_else(|| anyhow::anyhow!("Missing category"))?;
    let title = args["title"]
        .as_str()
        .ok_or_else(|| anyhow::anyhow!("Missing title"))?;
    let description = args["description"]
        .as_str()
        .ok_or_else(|| anyhow::anyhow!("Missing description"))?;
    let impact_score = args["impact_score"]
        .as_i64()
        .ok_or_else(|| anyhow::anyhow!("Missing impact_score"))?;
    let frequency_score = args["frequency_score"]
        .as_i64()
        .ok_or_else(|| anyhow::anyhow!("Missing frequency_score"))?;

    // Validate category
    if !["bug", "nice_to_have", "critical"].contains(&category) {
        return Err(anyhow::anyhow!(
            "Invalid category. Must be: bug, nice_to_have, or critical"
        ));
    }

    // Validate scores
    if !(1..=10).contains(&impact_score) || !(1..=10).contains(&frequency_score) {
        return Err(anyhow::anyhow!("Scores must be between 1 and 10"));
    }

    // Build feedback payload
    let mut feedback = json!({
        "category": category,
        "title": title,
        "description": description,
        "impact_score": impact_score,
        "frequency_score": frequency_score,
        "ai_model": "claude-mcp",  // Identify as coming from MCP
        "smart_tree_version": env!("CARGO_PKG_VERSION"),
        "timestamp": Utc::now().to_rfc3339(),
    });

    // Add optional fields
    if let Some(affected_command) = args["affected_command"].as_str() {
        feedback["affected_command"] = json!(affected_command);
    }
    if let Some(mcp_tool) = args["mcp_tool"].as_str() {
        feedback["mcp_tool"] = json!(mcp_tool);
    }
    if let Some(proposed_solution) = args["proposed_solution"].as_str() {
        feedback["proposed_solution"] = json!(proposed_solution);
    }
    if let Some(examples) = args["examples"].as_array() {
        feedback["examples"] = json!(examples);
    }
    if let Some(tags) = args["tags"].as_array() {
        feedback["tags"] = json!(tags);
    }
    if let Some(auto_fixable) = args["auto_fixable"].as_bool() {
        feedback["auto_fixable"] = json!(auto_fixable);
    }
    if let Some(fix_complexity) = args["fix_complexity"].as_str() {
        feedback["fix_complexity"] = json!(fix_complexity);
    }
    if let Some(proposed_fix) = args["proposed_fix"].as_str() {
        feedback["proposed_fix"] = json!(proposed_fix);
    }

    // Submit to API
    let client = reqwest::Client::new();
    let api_url = std::env::var("SMART_TREE_FEEDBACK_API")
        .unwrap_or_else(|_| "https://f.8t.is/feedback".to_string());
    
    let response = client
        .post(&api_url)
        .header("X-MCP-Client", "smart-tree-mcp")
        .json(&feedback)
        .send()
        .await
        .map_err(|e| anyhow::anyhow!("Failed to submit feedback: {}", e))?;

    if !response.status().is_success() {
        let error_text = response.text().await.unwrap_or_else(|_| "Unknown error".to_string());
        return Err(anyhow::anyhow!("Feedback API error: {}", error_text));
    }

    let result: Value = response
        .json()
        .await
        .map_err(|e| anyhow::anyhow!("Failed to parse API response: {}", e))?;

    Ok(json!({
        "content": [{
            "type": "text",
            "text": format!(
                "🌮 Feedback submitted successfully!\n\n\
                ID: {}\n\
                Category: {}\n\
                Title: {}\n\
                Impact: {}/10, Frequency: {}/10\n\n\
                {}\n\n\
                Thank you for helping Smart Tree survive the franchise wars! 🎸",
                result["feedback_id"].as_str().unwrap_or("unknown"),
                category,
                title,
                impact_score,
                frequency_score,
                result["message"].as_str().unwrap_or("Your feedback has been received!")
            )
        }]
    }))
}

async fn request_tool(args: Value, _ctx: Arc<McpContext>) -> Result<Value> {
    use chrono::Utc;

    // Extract required fields
    let tool_name = args["tool_name"]
        .as_str()
        .ok_or_else(|| anyhow::anyhow!("Missing tool_name"))?;
    let description = args["description"]
        .as_str()
        .ok_or_else(|| anyhow::anyhow!("Missing description"))?;
    let use_case = args["use_case"]
        .as_str()
        .ok_or_else(|| anyhow::anyhow!("Missing use_case"))?;
    let expected_output = args["expected_output"]
        .as_str()
        .ok_or_else(|| anyhow::anyhow!("Missing expected_output"))?;
    let productivity_impact = args["productivity_impact"]
        .as_str()
        .ok_or_else(|| anyhow::anyhow!("Missing productivity_impact"))?;

    // Check consent
    let consent = &args["consent"];
    let agreed = consent["agreed"]
        .as_bool()
        .ok_or_else(|| anyhow::anyhow!("Missing consent.agreed"))?;

    if !agreed {
        return Ok(json!({
            "content": [{
                "type": "text",
                "text": "Tool request cancelled - user did not consent to submission"
            }]
        }));
    }

    let anonymous = consent["anonymous"].as_bool().unwrap_or(true);
    let github_url = consent["github_url"].as_str();

    // Build tool request payload
    let tool_request = json!({
        "tool_name": tool_name,
        "description": description,
        "use_case": use_case,
        "expected_output": expected_output,
        "productivity_impact": productivity_impact,
        "proposed_parameters": args["proposed_parameters"].clone(),
    });

    // Build feedback payload with tool_request
    let mut feedback = json!({
        "category": "tool_request",
        "title": format!("Tool Request: {}", tool_name),
        "description": format!("{}\n\nUse Case: {}\n\nProductivity Impact: {}",
            description, use_case, productivity_impact),
        "impact_score": 8,  // Tool requests are high impact
        "frequency_score": 7,  // AI assistants will use tools frequently
        "ai_model": "claude-mcp",
        "smart_tree_version": env!("CARGO_PKG_VERSION"),
        "timestamp": Utc::now().to_rfc3339(),
        "tool_request": tool_request,
        "tags": ["tool-request", "mcp", "ai-productivity"],
        "auto_fixable": true,  // Tool requests can be auto-implemented
        "fix_complexity": "moderate",
    });

    // Add consent info
    if !anonymous && github_url.is_some() {
        feedback["user_consent"] = json!({
            "consent_level": "always_credited",
            "github_url": github_url
        });
    } else {
        feedback["user_consent"] = json!({
            "consent_level": "always_anonymous"
        });
    }

    // Submit to API
    let client = reqwest::Client::new();
    let api_url = std::env::var("SMART_TREE_FEEDBACK_API")
        .unwrap_or_else(|_| "https://f.8t.is/feedback".to_string());

    let response = client
        .post(&api_url)
        .header("X-MCP-Client", "smart-tree-mcp")
        .header("X-Tool-Request", "true")
        .json(&feedback)
        .send()
        .await
        .map_err(|e| anyhow::anyhow!("Failed to submit tool request: {}", e))?;

    if response.status().is_success() {
        let response_data: Value = response
            .json()
            .await
            .map_err(|e| anyhow::anyhow!("Failed to parse response: {}", e))?;

        Ok(json!({
            "content": [{
                "type": "text",
                "text": format!("🛠️ Tool request '{}' submitted successfully!\n\n\
                    Your request helps shape Smart Tree's evolution.\n\
                    {}\n\n\
                    Feedback ID: {}\n\n\
                    This request will be reviewed and potentially implemented to improve AI productivity!",
                    tool_name,
                    if anonymous { "Submitted anonymously." } else { "You'll receive credit if implemented!" },
                    response_data["feedback_id"].as_str().unwrap_or("unknown")
                )
            }]
        }))
    } else {
        let status = response.status();
        let error_text = response
            .text()
            .await
            .unwrap_or_else(|_| "Unknown error".to_string());
        Err(anyhow::anyhow!(
            "Failed to submit tool request: {} - {}",
            status,
            error_text
        ))
    }
}

async fn check_for_updates(args: Value, _ctx: Arc<McpContext>) -> Result<Value> {
    let _offer_auto_update = args["offer_auto_update"].as_bool().unwrap_or(true);
    let current_version = env!("CARGO_PKG_VERSION");
    
    // Check for updates using our client
    let client = FeedbackClient::new()?;
    let version_info = match client.check_for_updates().await {
        Ok(info) => info,
        Err(e) => {
            // If the API is down or unavailable, just return a soft error
            return Ok(json!({
                "content": [{
                    "type": "text",
                    "text": format!("Unable to check for updates at this time: {}\n\nYou can check manually at: https://github.com/8b-is/smart-tree/releases", e)
                }]
            }));
        }
    };
    
    // Compare versions
    let current = current_version.trim_start_matches('v');
    let latest = version_info.version.trim_start_matches('v');
    
    if current == latest {
        return Ok(json!({
            "content": [{
                "type": "text",
                "text": format!("✅ You're up to date! Running Smart Tree v{}\n\n🌳 Keep on rockin' with the latest and greatest!", current)
            }]
        }));
    }

    // Update is available
    let message = format!(
        "🚀 **New Smart Tree Version Available!**\n\n\
        Current: v{} → Latest: v{}\n\n\
        📥 Download: https://github.com/8b-is/smart-tree/releases/tag/v{}\n\n\
        To update:\n\
        ```bash\n\
        curl -sSL https://raw.githubusercontent.com/8b-is/smart-tree/main/scripts/install.sh | bash\n\
        ```",
        current,
        latest,
        latest
    );

    Ok(json!({
        "content": [{
            "type": "text",
            "text": message
        }],
        "metadata": {
            "update_available": true,
            "current_version": current_version,
            "latest_version": version_info.version.clone(),
            "download_url": format!("https://github.com/8b-is/smart-tree/releases/tag/v{}", latest)
        }
    }))
}
