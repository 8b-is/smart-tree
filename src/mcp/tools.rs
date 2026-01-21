//! MCP tools implementation for Smart Tree

use super::{is_path_allowed, McpContext};
use crate::mcp::helpers::{
    scan_with_config, should_use_default_ignores, validate_and_convert_path, ScannerConfigBuilder,
};
use crate::mcp::permissions::get_available_tools;
use crate::{
    feedback_client::FeedbackClient,
    formatters::{
        ai::AiFormatter, classic::ClassicFormatter, csv::CsvFormatter, digest::DigestFormatter,
        hex::HexFormatter, json::JsonFormatter, projects::ProjectsFormatter,
        quantum::QuantumFormatter, quantum_semantic::QuantumSemanticFormatter,
        semantic::SemanticFormatter, stats::StatsFormatter, summary::SummaryFormatter,
        summary_ai::SummaryAiFormatter, tsv::TsvFormatter, Formatter, PathDisplayMode,
    },
    parse_size,
};
use anyhow::Result;
use regex::Regex;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::SystemTime;

// Tool lanes for AI escalation path - Omni's three-lane design
#[derive(Debug, Clone, Serialize)]
pub enum ToolLane {
    #[allow(dead_code)]
    Explore, // 🔍 Discovery and overview
    #[allow(dead_code)]
    Analyze, // 🧪 Deep analysis and search
    #[allow(dead_code)]
    Act, // ⚡ Modifications and writes
}

impl ToolLane {
    #[allow(dead_code)]
    pub fn emoji(&self) -> &str {
        match self {
            ToolLane::Explore => "🔍",
            ToolLane::Analyze => "🧪",
            ToolLane::Act => "⚡",
        }
    }

    #[allow(dead_code)]
    pub fn name(&self) -> &str {
        match self {
            ToolLane::Explore => "EXPLORE",
            ToolLane::Analyze => "ANALYZE",
            ToolLane::Act => "ACT",
        }
    }
}

// Note: should_use_default_ignores moved to helpers module

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
            name: "verify_permissions".to_string(),
            description: "🔐 REQUIRED FIRST STEP: Verify permissions for a path before using other tools. This lightweight check determines which tools are available based on read/write permissions. Always call this first to see what operations are possible!".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "path": {
                        "type": "string",
                        "description": "Path to verify permissions for"
                    }
                },
                "required": ["path"]
            }),
        },
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
                        "description": "Maximum depth to traverse (0 = auto, each mode picks ideal depth)",
                        "default": 0
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
                        "description": "Compress output with zlib. Default: false (decompressed) for all modes to ensure compatibility with AI systems. Set to true only if your AI can handle base64 compressed content",
                        "default": null
                    },
                    "path_mode": {
                        "type": "string",
                        "enum": ["off", "relative", "full"],
                        "description": "Path display mode",
                        "default": "off"
                    },
                    "page": {
                        "type": "integer",
                        "description": "Page number (1-based) to return when paginating large outputs (works only for non-compressed, non-quantum modes)"
                    },
                    "page_size": {
                        "type": "integer",
                        "description": "Number of lines per page (default 500, max 10000)"
                    },
                    "max_bytes": {
                        "type": "integer",
                        "description": "Maximum bytes for returned page content (truncates within page if exceeded)"
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
                    "entry_type": {
                        "type": "string",
                        "enum": ["f", "d"],
                        "description": "Filter to show only files (f) or directories (d)"
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
                        "description": "Maximum depth to traverse (0 = auto, each mode picks ideal depth)",
                        "default": 0
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
            description: "🔍 EXPLORE - START HERE! Lightning-fast 3-level directory overview using SUMMARY-AI mode with 10x compression. Perfect for initial exploration before diving into details. This is your go-to tool for quickly understanding any codebase structure. Automatically optimized for AI token efficiency - saves you tokens while giving maximum insight!".to_string(),
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
            name: "project_context_dump".to_string(),
            description: "📦 FULL PROJECT CONTEXT - Get a complete, token-efficient project dump for AI assistants in ONE CALL! Combines project detection, key file identification, directory structure, and optionally file contents into a single compressed response. Configurable depth/file limits and compression modes (auto/marqant/summary-ai/quantum). Includes token budget awareness. PERFECT for bootstrapping AI context when walking into a new project!".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "path": {
                        "type": "string",
                        "description": "Path to the project root"
                    },
                    "max_depth": {
                        "type": "integer",
                        "description": "Maximum tree depth (default: 5)",
                        "default": 5,
                        "minimum": 1,
                        "maximum": 20
                    },
                    "max_files": {
                        "type": "integer",
                        "description": "Maximum files to include in listing (default: 100, max: 1000)",
                        "default": 100,
                        "minimum": 10,
                        "maximum": 1000
                    },
                    "include_content": {
                        "type": "boolean",
                        "description": "Include contents of key files like README, CLAUDE.md (default: false)",
                        "default": false
                    },
                    "compression": {
                        "type": "string",
                        "enum": ["auto", "marqant", "summary-ai", "quantum"],
                        "description": "Compression mode: 'auto' (smart selection), 'marqant' (markdown 70-90%), 'summary-ai' (10x), 'quantum' (max)",
                        "default": "auto"
                    },
                    "token_budget": {
                        "type": "integer",
                        "description": "Maximum tokens for response (warns if exceeded, default: 10000)",
                        "default": 10000,
                        "minimum": 1000,
                        "maximum": 50000
                    },
                    "include_git": {
                        "type": "boolean",
                        "description": "Include git status/branch info (default: true)",
                        "default": true
                    },
                    "key_files_only": {
                        "type": "boolean",
                        "description": "Only include key project files in listing (default: false)",
                        "default": false
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
            description: "🔍 ANALYZE: Powerful content search within files (like grep but AI-friendly). NOW WITH LINE CONTENT! Search for keywords, function names, TODOs, or any text pattern. Returns actual matching lines with content, not just file paths. Perfect for finding where specific functionality is implemented or tracking down references without needing to open files.".to_string(),
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
                    },
                    "include_content": {
                        "type": "boolean",
                        "description": "Include actual line content in results (default: true for AI)",
                        "default": true
                    },
                    "context_lines": {
                        "type": "integer",
                        "description": "Number of context lines before/after match (like grep -C)",
                        "minimum": 0,
                        "maximum": 10
                    },
                    "max_matches_per_file": {
                        "type": "integer",
                        "description": "Maximum matches to return per file",
                        "default": 20,
                        "minimum": 1,
                        "maximum": 100
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
            name: "find_projects".to_string(),
            description: "🚀 Discover all projects across a filesystem! Finds forgotten 3am coding gems by scanning for README.md, project markers (Cargo.toml, package.json, etc), and git repos. Returns condensed summaries with git info, dependencies, and timestamps. Perfect for SmartPastCode memory - find that brilliant solution you wrote months ago!".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "path": {
                        "type": "string",
                        "description": "Path to search for projects (default: current directory)"
                    },
                    "depth": {
                        "type": "integer",
                        "description": "Maximum depth to search (default: 10)",
                        "default": 10
                    }
                },
                "required": []
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
            name: "find_in_timespan".to_string(),
            description: "🕐 Find files modified within a specific time range. Perfect for finding files changed between two dates, during a specific week, or in a particular time period. More flexible than find_recent_changes for specific date ranges.".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "path": {
                        "type": "string",
                        "description": "Path to search in"
                    },
                    "start_date": {
                        "type": "string",
                        "description": "Start date (YYYY-MM-DD) - files modified after this date"
                    },
                    "end_date": {
                        "type": "string",
                        "description": "End date (YYYY-MM-DD) - files modified before this date (optional, defaults to today)"
                    },
                    "file_type": {
                        "type": "string",
                        "description": "Filter by file extension (optional)"
                    }
                },
                "required": ["path", "start_date"]
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
                        "description": "Maximum depth to traverse (0 = auto, each mode picks ideal depth)",
                        "default": 0
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
            description: "🛠️ Request a new MCP tool that doesn't exist yet (MCP ONLY!). When you need a tool that would increase your productivity but isn't available, use this to request it. Your request helps shape Smart Tree's evolution!".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "tool_name": {
                        "type": "string",
                        "description": "Proposed tool name (e.g., 'find_symbol', 'extract_imports', 'smart-tree-dev')"
                    },
                    "description": {
                        "type": "string",
                        "description": "What the tool should do"
                    },
                    "use_case": {
                        "type": "string",
                        "description": "Example use case demonstrating why you need this tool (optional)"
                    },
                    "proposed_parameters": {
                        "type": "object",
                        "description": "Suggested parameters for the tool (optional)",
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
                        "description": "What the tool should return (format and content) (optional)"
                    },
                    "productivity_impact": {
                        "type": "string",
                        "description": "How this tool would improve your productivity (optional)"
                    }
                },
                "required": ["tool_name", "description"]
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
        ToolDefinition {
            name: "watch_directory_sse".to_string(),
            description: "🔄 Watch a directory for real-time changes via Server-Sent Events (SSE). Streams file creation, modification, and deletion events as they happen. Perfect for monitoring active development directories, build outputs, or log folders. Returns an SSE endpoint URL that can be consumed by EventSource API.".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "path": {
                        "type": "string",
                        "description": "Path to the directory to watch"
                    },
                    "format": {
                        "type": "string",
                        "description": "Output format for analysis events",
                        "enum": ["hex", "ai", "quantum", "quantum_semantic", "json", "summary"],
                        "default": "ai"
                    },
                    "heartbeat_interval": {
                        "type": "integer",
                        "description": "Send heartbeat every N seconds",
                        "default": 30
                    },
                    "stats_interval": {
                        "type": "integer",
                        "description": "Send stats update every N seconds",
                        "default": 60
                    },
                    "include_content": {
                        "type": "boolean",
                        "description": "Include file contents in events",
                        "default": false
                    },
                    "max_depth": {
                        "type": "integer",
                        "description": "Maximum depth for recursive watching"
                    },
                    "include_patterns": {
                        "type": "array",
                        "items": {"type": "string"},
                        "description": "File patterns to include"
                    },
                    "exclude_patterns": {
                        "type": "array",
                        "items": {"type": "string"},
                        "description": "File patterns to exclude"
                    }
                },
                "required": ["path"]
            }),
        },
        ToolDefinition {
            name: "track_file_operation".to_string(),
            description: "🔐 Track file operations with hash-based change detection. Part of the ultimate context-driven system that logs all AI file manipulations to ~/.mem8/.filehistory/. Favors append operations as the least intrusive method. Perfect for maintaining a complete history of AI-assisted code changes!".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "file_path": {
                        "type": "string",
                        "description": "Path to the file being operated on"
                    },
                    "operation": {
                        "type": "string",
                        "enum": ["read", "write", "append", "prepend", "insert", "delete", "replace", "create", "remove", "relocate", "rename"],
                        "description": "Type of operation performed"
                    },
                    "old_content": {
                        "type": "string",
                        "description": "Previous content of the file (optional for new files)"
                    },
                    "new_content": {
                        "type": "string",
                        "description": "New content of the file"
                    },
                    "agent": {
                        "type": "string",
                        "description": "AI agent identifier",
                        "default": "claude"
                    },
                    "session_id": {
                        "type": "string",
                        "description": "Session ID for grouping related operations"
                    }
                },
                "required": ["file_path"]
            }),
        },
        ToolDefinition {
            name: "get_file_history".to_string(),
            description: "📜 Retrieve complete operation history for a file from the ~/.mem8/.filehistory/ tracking system. Shows all AI manipulations with timestamps, operations, hashes, and agents. Essential for understanding how a file evolved through AI assistance!".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "file_path": {
                        "type": "string",
                        "description": "Path to the file to get history for"
                    }
                },
                "required": ["file_path"]
            }),
        },
        ToolDefinition {
            name: "get_project_history_summary".to_string(),
            description: "📊 Get a summary of all AI operations performed in a project directory. Shows statistics like total operations, files modified, operation type breakdown, and activity timeline. Perfect for project audits and understanding AI collaboration patterns!".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "project_path": {
                        "type": "string",
                        "description": "Path to the project directory"
                    }
                },
                "required": ["project_path"]
            }),
        },
        ToolDefinition {
            name: "smart_edit".to_string(),
            description: "🚀 Apply multiple smart code edits using minimal tokens! Uses AST understanding to insert functions, replace bodies, add imports, etc. without sending full diffs. Revolutionary token-efficient editing that understands code structure!".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "file_path": {
                        "type": "string",
                        "description": "Path to the file to edit"
                    },
                    "edits": {
                        "type": "array",
                        "description": "Array of smart edit operations",
                        "items": {
                            "type": "object",
                            "properties": {
                                "operation": {
                                    "type": "string",
                                    "description": "Edit operation type",
                                    "enum": ["InsertFunction", "ReplaceFunction", "AddImport", "InsertClass", "AddMethod", "WrapCode", "DeleteElement", "Rename", "AddDocumentation", "SmartAppend"]
                                },
                                "name": {
                                    "type": "string",
                                    "description": "Name of the element (function, class, etc.)"
                                },
                                "class_name": {
                                    "type": "string",
                                    "description": "Optional class name for methods"
                                },
                                "namespace": {
                                    "type": "string",
                                    "description": "Optional namespace"
                                },
                                "body": {
                                    "type": "string",
                                    "description": "Code body to insert/replace"
                                },
                                "new_body": {
                                    "type": "string",
                                    "description": "New body for ReplaceFunction"
                                },
                                "import": {
                                    "type": "string",
                                    "description": "Import statement for AddImport"
                                },
                                "alias": {
                                    "type": "string",
                                    "description": "Optional alias for imports"
                                },
                                "after": {
                                    "type": "string",
                                    "description": "Insert after this function/method"
                                },
                                "before": {
                                    "type": "string",
                                    "description": "Insert before this function/method"
                                },
                                "visibility": {
                                    "type": "string",
                                    "description": "Visibility modifier",
                                    "enum": ["public", "private", "protected"],
                                    "default": "private"
                                },
                                "section": {
                                    "type": "string",
                                    "description": "Section for SmartAppend",
                                    "enum": ["imports", "functions", "classes", "main"]
                                },
                                "content": {
                                    "type": "string",
                                    "description": "Content to append"
                                }
                            },
                            "required": ["operation"]
                        }
                    }
                },
                "required": ["file_path", "edits"]
            }),
        },
        ToolDefinition {
            name: "get_function_tree".to_string(),
            description: "🌳 Get a structured view of all functions, classes, and their relationships in a code file. Shows function signatures, line numbers, visibility, and call relationships. Perfect for understanding code structure before making edits!".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "file_path": {
                        "type": "string",
                        "description": "Path to the file to analyze"
                    }
                },
                "required": ["file_path"]
            }),
        },
        ToolDefinition {
            name: "insert_function".to_string(),
            description: "✨ Insert a new function into a code file using minimal tokens. Automatically finds the right location based on context. No need to send diffs or specify line numbers - just the function name and body!".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "file_path": {
                        "type": "string",
                        "description": "Path to the file to edit"
                    },
                    "name": {
                        "type": "string",
                        "description": "Function name"
                    },
                    "body": {
                        "type": "string",
                        "description": "Function body (including parameters and return type)"
                    },
                    "class_name": {
                        "type": "string",
                        "description": "Optional class name if adding a method"
                    },
                    "after": {
                        "type": "string",
                        "description": "Insert after this function (optional)"
                    },
                    "before": {
                        "type": "string",
                        "description": "Insert before this function (optional)"
                    },
                    "visibility": {
                        "type": "string",
                        "description": "Visibility modifier",
                        "enum": ["public", "private", "protected"],
                        "default": "private"
                    }
                },
                "required": ["file_path", "name", "body"]
            }),
        },
        ToolDefinition {
            name: "remove_function".to_string(),
            description: "🗑️ Remove a function with dependency awareness. Checks if removal would break other functions and optionally cascades removal of orphaned functions. Token-efficient alternative to sending full file edits!".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "file_path": {
                        "type": "string",
                        "description": "Path to the file to edit"
                    },
                    "name": {
                        "type": "string",
                        "description": "Function name to remove"
                    },
                    "class_name": {
                        "type": "string",
                        "description": "Optional class name if removing a method"
                    },
                    "force": {
                        "type": "boolean",
                        "description": "Remove even if it would break dependencies",
                        "default": false
                    },
                    "cascade": {
                        "type": "boolean",
                        "description": "Also remove functions that only this one calls",
                        "default": false
                    }
                },
                "required": ["file_path", "name"]
            }),
        },
        ToolDefinition {
            name: "gather_project_context".to_string(),
            description: "🔍 Search AI tool directories (~/.claude, ~/.cursor, ~/.windsurf, etc.) for context about the current project. Finds chat histories, settings, and other relevant information with TEMPORAL ANALYSIS! See work patterns, peak times, and momentum. Use output_format='temporal' for time-based insights, apply temporal_decay_days for recency weighting. Perfect for understanding how you've been working with a project across different AI tools over time!".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "project_path": {
                        "type": "string",
                        "description": "Path to the project to gather context for"
                    },
                    "search_dirs": {
                        "type": "array",
                        "items": { "type": "string" },
                        "description": "AI tool directories to search (defaults to all known)"
                    },
                    "custom_dirs": {
                        "type": "array",
                        "items": { "type": "string" },
                        "description": "Additional custom directories to search"
                    },
                    "project_identifiers": {
                        "type": "array",
                        "items": { "type": "string" },
                        "description": "Unique strings to identify project (URLs, names, etc.)"
                    },
                    "max_results": {
                        "type": "integer",
                        "description": "Maximum contexts to return",
                        "default": 50
                    },
                    "min_relevance": {
                        "type": "number",
                        "description": "Minimum relevance score (0.0-1.0)",
                        "default": 0.0
                    },
                    "output_format": {
                        "type": "string",
                        "enum": ["summary", "json", "m8", "temporal", "partnership"],
                        "description": "Output format (temporal=time patterns, partnership=AI-human collaboration analysis)",
                        "default": "summary"
                    },
                    "privacy_mode": {
                        "type": "boolean",
                        "description": "Redact sensitive information",
                        "default": true
                    },
                    "temporal_resolution": {
                        "type": "string",
                        "enum": ["hour", "day", "week", "month", "quarter", "year"],
                        "description": "Resolution for temporal analysis",
                        "default": "day"
                    },
                    "temporal_decay_days": {
                        "type": "number",
                        "description": "Apply temporal decay with this half-life in days",
                        "minimum": 1.0
                    }
                },
                "required": ["project_path"]
            }),
        },
        ToolDefinition {
            name: "analyze_ai_tool_usage".to_string(),
            description: "📊 Analyze usage patterns across AI tool directories. Shows which tools you use most, recent activity, file types, and storage usage. Great for understanding your AI tool ecosystem and cleaning up old data!".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "tool_name": {
                        "type": "string",
                        "description": "Specific tool to analyze (e.g., '.claude', '.cursor')"
                    },
                    "days": {
                        "type": "integer",
                        "description": "Time range in days",
                        "default": 30
                    },
                    "include_paths": {
                        "type": "boolean",
                        "description": "Include detailed file paths",
                        "default": false
                    }
                }
            }),
        },
        ToolDefinition {
            name: "clean_old_context".to_string(),
            description: "🧹 Clean up old context files from AI tools (.claude, .windsurf, .cursor, etc.). Reclaim disk space by removing outdated chat histories and context files. SAFE BY DEFAULT: dry_run=true shows what would be deleted without actually deleting.".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "days_to_keep": {
                        "type": "integer",
                        "description": "Keep files newer than this many days",
                        "default": 90
                    },
                    "dry_run": {
                        "type": "boolean",
                        "description": "Show what would be deleted without actually deleting",
                        "default": true
                    },
                    "tools": {
                        "type": "array",
                        "items": { "type": "string" },
                        "description": "Specific tools to clean"
                    }
                }
            }),
        },
        ToolDefinition {
            name: "anchor_collaborative_memory".to_string(),
            description: "⚓ Anchor an important insight, solution, or breakthrough from our collaboration for future retrieval. Creates a memory that both AI and human can reference later with phrases like 'Remember when we solved X?'. Supports co-created memories, pattern insights, shared jokes, and more!".to_string(),
            input_schema: json!({
                "type": "object",
                "required": ["context", "keywords", "anchor_type"],
                "properties": {
                    "context": {
                        "type": "string",
                        "description": "The insight or solution to remember"
                    },
                    "keywords": {
                        "type": "array",
                        "items": { "type": "string" },
                        "description": "Keywords for future retrieval"
                    },
                    "anchor_type": {
                        "type": "string",
                        "enum": ["pattern_insight", "solution", "breakthrough", "learning", "joke", "technical", "process"],
                        "description": "Type of memory anchor"
                    },
                    "origin": {
                        "type": "string",
                        "description": "Who created this? 'human', 'ai:claude', or 'tandem:human:claude'",
                        "default": "tandem:human:claude"
                    },
                    "project_path": {
                        "type": "string",
                        "description": "Project to associate with (default: current directory)"
                    }
                }
            }),
        },
        ToolDefinition {
            name: "find_collaborative_memories".to_string(),
            description: "🔮 Search for previously anchored collaborative memories. NOW WITH WAVE RESONANCE! Two modes: keyword search (fast) or resonance search (semantic similarity). Use resonance for 'find something similar to X' queries!".to_string(),
            input_schema: json!({
                "type": "object",
                "required": ["keywords"],
                "properties": {
                    "keywords": {
                        "type": "array",
                        "items": { "type": "string" },
                        "description": "Keywords to search for (or query terms for resonance)"
                    },
                    "use_resonance": {
                        "type": "boolean",
                        "description": "Use wave resonance for semantic similarity search (default: false)",
                        "default": false
                    },
                    "memory_type": {
                        "type": "string",
                        "enum": ["pattern", "solution", "conversation", "technical", "learning", "joke"],
                        "description": "Filter by memory type (for resonance search)"
                    },
                    "resonance_threshold": {
                        "type": "number",
                        "description": "Minimum similarity score 0.0-1.0 (default: 0.3)",
                        "minimum": 0.0,
                        "maximum": 1.0
                    },
                    "project_path": {
                        "type": "string",
                        "description": "Project path (default: current directory)"
                    },
                    "max_results": {
                        "type": "integer",
                        "description": "Maximum memories to return (default: 10)",
                        "minimum": 1,
                        "maximum": 50
                    }
                }
            }),
        },
        ToolDefinition {
            name: "wave_memory".to_string(),
            description: "🌊 Direct access to Wave Memory - the ultimate memory system for Claude Code! Store memories as waves with emotional encoding, retrieve by resonance, check stats. This is THE memory tool for persistent context across sessions.".to_string(),
            input_schema: json!({
                "type": "object",
                "required": ["operation"],
                "properties": {
                    "operation": {
                        "type": "string",
                        "enum": ["stats", "anchor", "find", "resonance", "get", "delete"],
                        "description": "Operation: stats (view memory stats), anchor (store memory), find (keyword search), resonance (semantic search), get (by ID), delete (by ID)"
                    },
                    "content": {
                        "type": "string",
                        "description": "Memory content (for anchor operation)"
                    },
                    "keywords": {
                        "type": "array",
                        "items": { "type": "string" },
                        "description": "Keywords for anchor/find/resonance"
                    },
                    "memory_type": {
                        "type": "string",
                        "enum": ["pattern", "solution", "conversation", "technical", "learning", "joke"],
                        "description": "Memory type (pattern=deep insights, solution=breakthroughs, technical=code patterns, joke=shared humor)",
                        "default": "technical"
                    },
                    "valence": {
                        "type": "number",
                        "description": "Emotional valence -1.0 (negative) to 1.0 (positive)",
                        "minimum": -1.0,
                        "maximum": 1.0
                    },
                    "arousal": {
                        "type": "number",
                        "description": "Emotional arousal 0.0 (calm) to 1.0 (excited)",
                        "minimum": 0.0,
                        "maximum": 1.0
                    },
                    "memory_id": {
                        "type": "string",
                        "description": "Memory ID (for get/delete operations)"
                    },
                    "threshold": {
                        "type": "number",
                        "description": "Resonance threshold for similarity search (default: 0.3)",
                        "minimum": 0.0,
                        "maximum": 1.0
                    },
                    "max_results": {
                        "type": "integer",
                        "description": "Maximum results (default: 10)",
                        "minimum": 1,
                        "maximum": 100
                    }
                }
            }),
        },
        ToolDefinition {
            name: "get_collaboration_rapport".to_string(),
            description: "💝 Check the rapport index between you and your AI partner. Shows trust level, communication efficiency, shared vocabulary, productivity trends, and even tracks inside jokes! See how your partnership is evolving over time.".to_string(),
            input_schema: json!({
                "type": "object",
                "required": ["ai_tool"],
                "properties": {
                    "ai_tool": {
                        "type": "string",
                        "description": "AI tool name (e.g., 'claude', 'cursor', 'windsurf')"
                    },
                    "project_path": {
                        "type": "string",
                        "description": "Project path (default: current directory)"
                    }
                }
            }),
        },
        ToolDefinition {
            name: "get_co_engagement_heatmap".to_string(),
            description: "🌡️ Visualize when you and AI collaborate most effectively! Shows a temporal heatmap of your tandem work sessions across days and hours. Identifies peak collaboration zones and helps optimize your partnership schedule.".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "project_path": {
                        "type": "string",
                        "description": "Project path (default: current directory)"
                    },
                    "format": {
                        "type": "string",
                        "enum": ["visual", "data"],
                        "description": "Output format: 'visual' for emoji heatmap, 'data' for raw values",
                        "default": "visual"
                    }
                }
            }),
        },
        ToolDefinition {
            name: "get_cross_domain_patterns".to_string(),
            description: "🔗 Discover patterns that appear across multiple projects and domains! Finds algorithmic patterns (like wave decay), architectural patterns, solutions, and collaborative workflows that transcend specific contexts. Perfect for 'I've seen this pattern before...' moments!".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "project_path": {
                        "type": "string",
                        "description": "Project path (default: current directory)"
                    },
                    "pattern_type": {
                        "type": "string",
                        "enum": ["algorithm", "architecture", "problem", "solution", "metaphor", "workflow", "collaboration"],
                        "description": "Filter by pattern type"
                    },
                    "min_strength": {
                        "type": "number",
                        "description": "Minimum pattern strength (0.0-1.0)",
                        "minimum": 0.0,
                        "maximum": 1.0
                    }
                }
            }),
        },
        ToolDefinition {
            name: "suggest_cross_session_insights".to_string(),
            description: "💡 Get relevant insights from other AI sessions that might help with current work! Uses keywords to find applicable patterns, solutions, and learnings from different projects. Like having a wise advisor who remembers everything: 'This reminds me of when we solved X in project Y...'".to_string(),
            input_schema: json!({
                "type": "object",
                "required": ["keywords"],
                "properties": {
                    "keywords": {
                        "type": "array",
                        "items": { "type": "string" },
                        "description": "Keywords describing current work or problem"
                    },
                    "project_path": {
                        "type": "string",
                        "description": "Project path (default: current directory)"
                    },
                    "max_results": {
                        "type": "integer",
                        "description": "Maximum insights to return (default: 5)",
                        "minimum": 1,
                        "maximum": 20
                    }
                }
            }),
        },
        ToolDefinition {
            name: "invite_persona".to_string(),
            description: "🎭 Invite a specialized AI persona for temporary consultation! Based on your context, summons The Cheet (performance optimization), Omni (wave patterns & philosophy), or Trish (organization & documentation). Each brings unique expertise from past sessions!".to_string(),
            input_schema: json!({
                "type": "object",
                "required": ["context"],
                "properties": {
                    "context": {
                        "type": "string",
                        "description": "What you need help with"
                    },
                    "duration_minutes": {
                        "type": "integer",
                        "description": "Consultation duration (default: 10 minutes)",
                        "minimum": 5,
                        "maximum": 60
                    }
                }
            }),
        },
        ToolDefinition {
            name: "scan_for_context".to_string(),
            description: "🌍 Universal Chat Scanner - Discovers and aggregates conversations from ALL your AI tools! Scans Claude projects, Cursor, Windsurf, VSCode, OpenWebUI, LMStudio, ChatGPT exports, and more. Unifies scattered context into organized .m8 memories. Perfect when you need to find that conversation where you solved a similar problem!".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "scan_all": {
                        "type": "boolean",
                        "description": "Scan all known locations (default: true)",
                        "default": true
                    },
                    "save_to": {
                        "type": "string",
                        "enum": ["project", "user", "llm", "global"],
                        "description": "Where to save memories (default: global)",
                        "default": "global"
                    }
                }
            }),
        },
        // ==========================================================================
        // 📖 SMART READ TOOL - The Treehugger-powered file reader!
        // Compresses code files using AST parsing to show structure with expandable
        // function references. Auto-expands based on context keywords!
        // ==========================================================================
        ToolDefinition {
            name: "read".to_string(),
            description: "📖 Smart file reader with AST-aware compression! Reads files and automatically compresses code by collapsing function bodies to signatures. Use expand_functions to expand specific functions, or expand_context to auto-expand functions matching keywords. Returns collapsed code with [fn:name] references that can be expanded. Perfect for understanding large files without burning tokens!".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "file_path": {
                        "type": "string",
                        "description": "Path to the file to read"
                    },
                    "compress": {
                        "type": "boolean",
                        "description": "Enable AST-aware compression (collapses function bodies). Default: true for code files, false for others",
                        "default": true
                    },
                    "expand_functions": {
                        "type": "array",
                        "items": { "type": "string" },
                        "description": "List of function names to expand fully (e.g., ['main', 'handle_request'])"
                    },
                    "expand_context": {
                        "type": "array",
                        "items": { "type": "string" },
                        "description": "Keywords to auto-expand matching functions (e.g., ['error', 'auth'] expands functions with these in name/body)"
                    },
                    "expand_all": {
                        "type": "boolean",
                        "description": "Expand all functions (disables compression)",
                        "default": false
                    },
                    "max_lines": {
                        "type": "integer",
                        "description": "Maximum lines to return (0 = unlimited)",
                        "default": 0
                    },
                    "offset": {
                        "type": "integer",
                        "description": "Line offset to start from (1-based)",
                        "default": 1
                    },
                    "show_line_numbers": {
                        "type": "boolean",
                        "description": "Show line numbers",
                        "default": true
                    }
                },
                "required": ["file_path"]
            }),
        },
    ];

    Ok(json!({
        "tools": tools
    }))
}

/// Handle wave_memory tool - direct access to wave-based memory system
async fn handle_wave_memory(args: Value) -> Result<Value> {
    use crate::mcp::wave_memory::{get_wave_memory, MemoryType};

    let operation = args["operation"]
        .as_str()
        .ok_or_else(|| anyhow::anyhow!("Missing operation"))?;

    let wave_memory = get_wave_memory();
    let mut manager = wave_memory.lock().map_err(|e| anyhow::anyhow!("Lock error: {}", e))?;

    match operation {
        "stats" => {
            Ok(json!({
                "operation": "stats",
                "wave_memory": manager.stats(),
                "message": "🌊 Wave Memory statistics",
            }))
        }
        "anchor" => {
            let content = args["content"]
                .as_str()
                .ok_or_else(|| anyhow::anyhow!("Missing content for anchor"))?
                .to_string();
            let keywords: Vec<String> = args["keywords"]
                .as_array()
                .map(|arr| arr.iter().filter_map(|v| v.as_str().map(String::from)).collect())
                .unwrap_or_default();
            let memory_type = args["memory_type"]
                .as_str()
                .map(MemoryType::parse)
                .unwrap_or(MemoryType::Technical);
            let valence = args["valence"].as_f64().unwrap_or(0.0) as f32;
            let arousal = args["arousal"].as_f64().unwrap_or(0.5) as f32;

            let id = manager.anchor(
                content.clone(),
                keywords.clone(),
                memory_type,
                valence,
                arousal,
                "tandem:human:claude".to_string(),
                None,
            )?;

            Ok(json!({
                "operation": "anchor",
                "success": true,
                "memory_id": id,
                "content_preview": if content.len() > 50 { format!("{}...", &content[..50]) } else { content },
                "keywords": keywords,
                "memory_type": format!("{:?}", memory_type),
                "emotional_encoding": {
                    "valence": valence,
                    "arousal": arousal,
                },
                "message": "🌊 Memory anchored as wave",
            }))
        }
        "find" => {
            let keywords: Vec<String> = args["keywords"]
                .as_array()
                .map(|arr| arr.iter().filter_map(|v| v.as_str().map(String::from)).collect())
                .unwrap_or_default();
            let max_results = args["max_results"].as_u64().unwrap_or(10) as usize;

            let results = manager.find_by_keywords(&keywords, max_results);
            let memories: Vec<_> = results.iter().map(|mem| {
                json!({
                    "id": mem.id,
                    "content": mem.content,
                    "keywords": mem.keywords,
                    "memory_type": format!("{:?}", mem.memory_type),
                    "valence": mem.valence,
                    "arousal": mem.arousal,
                    "access_count": mem.access_count,
                })
            }).collect();

            Ok(json!({
                "operation": "find",
                "keywords": keywords,
                "total_found": memories.len(),
                "memories": memories,
            }))
        }
        "resonance" => {
            let keywords: Vec<String> = args["keywords"]
                .as_array()
                .map(|arr| arr.iter().filter_map(|v| v.as_str().map(String::from)).collect())
                .unwrap_or_default();
            let memory_type = args["memory_type"]
                .as_str()
                .map(MemoryType::parse)
                .unwrap_or(MemoryType::Technical);
            let threshold = args["threshold"].as_f64().unwrap_or(0.3) as f32;
            let max_results = args["max_results"].as_u64().unwrap_or(10) as usize;

            let query = keywords.join(" ");
            let results = manager.find_by_resonance(&query, &keywords, memory_type, threshold, max_results);
            let memories: Vec<_> = results.iter().map(|(mem, resonance)| {
                json!({
                    "id": mem.id,
                    "content": mem.content,
                    "keywords": mem.keywords,
                    "memory_type": format!("{:?}", mem.memory_type),
                    "resonance_score": format!("{:.2}", resonance),
                    "valence": mem.valence,
                    "arousal": mem.arousal,
                })
            }).collect();

            Ok(json!({
                "operation": "resonance",
                "search_mode": "wave_interference",
                "query": keywords,
                "threshold": threshold,
                "total_found": memories.len(),
                "memories": memories,
                "message": "🌊 Found memories by wave resonance",
            }))
        }
        "get" => {
            let id = args["memory_id"]
                .as_str()
                .ok_or_else(|| anyhow::anyhow!("Missing memory_id"))?;

            if let Some(mem) = manager.get(id) {
                Ok(json!({
                    "operation": "get",
                    "found": true,
                    "memory": {
                        "id": mem.id,
                        "content": mem.content,
                        "keywords": mem.keywords,
                        "memory_type": format!("{:?}", mem.memory_type),
                        "valence": mem.valence,
                        "arousal": mem.arousal,
                        "created_at": mem.created_at.to_rfc3339(),
                        "last_accessed": mem.last_accessed.to_rfc3339(),
                        "access_count": mem.access_count,
                        "origin": mem.origin,
                        "grid_position": { "x": mem.x, "y": mem.y, "z": mem.z },
                    }
                }))
            } else {
                Ok(json!({
                    "operation": "get",
                    "found": false,
                    "memory_id": id,
                    "message": "Memory not found",
                }))
            }
        }
        "delete" => {
            let id = args["memory_id"]
                .as_str()
                .ok_or_else(|| anyhow::anyhow!("Missing memory_id"))?;

            let deleted = manager.delete(id);
            Ok(json!({
                "operation": "delete",
                "success": deleted,
                "memory_id": id,
                "message": if deleted { "Memory deleted" } else { "Memory not found" },
            }))
        }
        _ => Err(anyhow::anyhow!("Unknown wave_memory operation: {}", operation)),
    }
}

pub async fn handle_tools_call(params: Value, ctx: Arc<McpContext>) -> Result<Value> {
    let tool_name = params["name"]
        .as_str()
        .ok_or_else(|| anyhow::anyhow!("Missing tool name"))?;
    let args = params["arguments"].clone();

    // Record this tool call for learning
    ctx.assistant.record_call(tool_name).await;

    // Clone ctx for the match since we need it again later
    let ctx_clone = ctx.clone();

    let result = match tool_name {
        "verify_permissions" => verify_permissions(args, ctx_clone.clone()).await,
        "server_info" => server_info(args, ctx_clone.clone()).await,
        "analyze_directory" => analyze_directory(args, ctx_clone.clone()).await,
        "find_files" => find_files(args, ctx_clone.clone()).await,
        "get_statistics" => get_statistics(args, ctx_clone.clone()).await,
        "get_digest" => get_digest(args, ctx_clone.clone()).await,
        "quick_tree" => quick_tree(args, ctx_clone.clone()).await,
        "project_overview" => project_overview(args, ctx_clone.clone()).await,
        "project_context_dump" => project_context_dump(args, ctx_clone.clone()).await,
        "find_code_files" => find_code_files(args, ctx_clone.clone()).await,
        "find_config_files" => find_config_files(args, ctx_clone.clone()).await,
        "find_projects" => find_projects(args, ctx_clone.clone()).await,
        "find_documentation" => find_documentation(args, ctx_clone.clone()).await,
        "search_in_files" => search_in_files(args, ctx_clone.clone()).await,
        "find_large_files" => find_large_files(args, ctx_clone.clone()).await,
        "find_recent_changes" => find_recent_changes(args, ctx_clone.clone()).await,
        "find_in_timespan" => find_in_timespan(args, ctx_clone.clone()).await,
        "compare_directories" => compare_directories(args, ctx_clone.clone()).await,
        "get_git_status" => get_git_status(args, ctx_clone.clone()).await,
        "find_duplicates" => find_duplicates(args, ctx_clone.clone()).await,
        "analyze_workspace" => analyze_workspace(args, ctx_clone.clone()).await,
        "find_tests" => find_tests(args, ctx_clone.clone()).await,
        "find_build_files" => find_build_files(args, ctx_clone.clone()).await,
        "directory_size_breakdown" => directory_size_breakdown(args, ctx_clone.clone()).await,
        "find_empty_directories" => find_empty_directories(args, ctx_clone.clone()).await,
        "semantic_analysis" => semantic_analysis(args, ctx_clone.clone()).await,
        "submit_feedback" => submit_feedback(args, ctx_clone.clone()).await,
        "request_tool" => request_tool(args, ctx_clone.clone()).await,
        "check_for_updates" => check_for_updates(args, ctx_clone.clone()).await,
        "watch_directory_sse" => watch_directory_sse(args, ctx_clone.clone()).await,
        "track_file_operation" => track_file_operation(args, ctx_clone.clone()).await,
        "get_file_history" => get_file_history(args, ctx_clone.clone()).await,
        "get_project_history_summary" => get_project_history_summary(args, ctx_clone.clone()).await,

        // Smart edit tools
        "smart_edit" => crate::mcp::smart_edit::handle_smart_edit(Some(args)).await,
        "get_function_tree" => crate::mcp::smart_edit::handle_get_function_tree(Some(args)).await,
        "insert_function" => crate::mcp::smart_edit::handle_insert_function(Some(args)).await,
        "remove_function" => crate::mcp::smart_edit::handle_remove_function(Some(args)).await,

        // Context gathering tools
        "gather_project_context" => {
            let req: crate::mcp::context_tools::GatherProjectContextRequest =
                serde_json::from_value(args)?;
            // Simple permission check - just verify path is allowed
            let permission_check = |_perm_req| {
                // For now, always allow home directory access for context gathering
                // TODO: Implement proper permission system
                Ok(true)
            };
            crate::mcp::context_tools::gather_project_context(req, permission_check).await
        }
        "analyze_ai_tool_usage" => {
            let req: crate::mcp::context_tools::AnalyzeAiToolUsageRequest =
                serde_json::from_value(args)?;
            let permission_check = |_perm_req| Ok(true);
            crate::mcp::context_tools::analyze_ai_tool_usage(req, permission_check).await
        }
        "clean_old_context" => {
            let req: crate::mcp::context_tools::CleanOldContextRequest =
                serde_json::from_value(args)?;
            let permission_check = |_perm_req| Ok(true);
            crate::mcp::context_tools::clean_old_context(req, permission_check).await
        }
        "anchor_collaborative_memory" => {
            let req: crate::mcp::context_tools::AnchorMemoryRequest = serde_json::from_value(args)?;
            let permission_check = |_perm_req| Ok(true);
            crate::mcp::context_tools::anchor_collaborative_memory(req, permission_check).await
        }
        "find_collaborative_memories" => {
            let req: crate::mcp::context_tools::FindMemoriesRequest = serde_json::from_value(args)?;
            let permission_check = |_perm_req| Ok(true);
            crate::mcp::context_tools::find_collaborative_memories(req, permission_check).await
        }
        "wave_memory" => {
            handle_wave_memory(args).await
        }
        "get_collaboration_rapport" => {
            let req: crate::mcp::context_tools::GetRapportRequest = serde_json::from_value(args)?;
            let permission_check = |_perm_req| Ok(true);
            crate::mcp::context_tools::get_collaboration_rapport(req, permission_check).await
        }
        "get_co_engagement_heatmap" => {
            let req: crate::mcp::context_tools::GetHeatmapRequest = serde_json::from_value(args)?;
            let permission_check = |_perm_req| Ok(true);
            crate::mcp::context_tools::get_co_engagement_heatmap(req, permission_check).await
        }
        "get_cross_domain_patterns" => {
            let req: crate::mcp::context_tools::GetPatternsRequest = serde_json::from_value(args)?;
            let permission_check = |_perm_req| Ok(true);
            crate::mcp::context_tools::get_cross_domain_patterns(req, permission_check).await
        }
        "suggest_cross_session_insights" => {
            let req: crate::mcp::context_tools::SuggestInsightsRequest =
                serde_json::from_value(args)?;
            let permission_check = |_perm_req| Ok(true);
            crate::mcp::context_tools::suggest_cross_session_insights(req, permission_check).await
        }
        "invite_persona" => {
            let req: crate::mcp::context_tools::InvitePersonaRequest =
                serde_json::from_value(args)?;
            let permission_check = |_perm_req| Ok(true);
            crate::mcp::context_tools::invite_persona(req, permission_check).await
        }

        // Universal chat scanner
        "scan_for_context" => {
            // Run the universal chat scanner
            use crate::universal_chat_scanner;
            tokio::spawn(async move {
                let _ = universal_chat_scanner::scan_for_context().await;
            });

            Ok(json!({
                "content": [{
                    "type": "text",
                    "text": "🌍 Universal Chat Scanner started!\n\n\
                             Scanning for conversations in:\n\
                             • ~/.claude/projects\n\
                             • Cursor/Windsurf directories\n\
                             • VSCode/Copilot history\n\
                             • OpenWebUI/LMStudio\n\
                             • ChatGPT exports\n\
                             • Text messages (if available)\n\n\
                             Results will be saved to ~/.mem8/ organized by source.\n\
                             Check the terminal for interactive prompts!"
                }]
            }))
        }

        // 📖 Smart file read with treehugger compression
        "read" => smart_read(args, ctx_clone.clone()).await,

        _ => Err(anyhow::anyhow!("Unknown tool: {}", tool_name)),
    }?;

    // Enhance the response with helpful recommendations
    let enhanced_result = ctx.assistant.enhance_response(tool_name, result).await;

    Ok(enhanced_result)
}

#[derive(Debug, Deserialize)]
struct AnalyzeDirectoryArgs {
    #[serde(default = "default_path")]
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

#[derive(Debug, Deserialize)]
struct ProjectContextDumpArgs {
    path: String,
    #[serde(default = "default_context_depth")]
    max_depth: usize,
    #[serde(default = "default_max_files")]
    max_files: usize,
    #[serde(default)]
    include_content: bool,
    #[serde(default = "default_compression")]
    compression: String,
    #[serde(default = "default_token_budget")]
    token_budget: usize,
    #[serde(default = "default_true")]
    include_git: bool,
    #[serde(default)]
    key_files_only: bool,
}

fn default_context_depth() -> usize {
    5
}

fn default_max_files() -> usize {
    100
}

fn default_compression() -> String {
    "auto".to_string()
}

fn default_token_budget() -> usize {
    10000
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

fn default_path() -> String {
    ".".to_string()
}

async fn server_info(_args: Value, ctx: Arc<McpContext>) -> Result<Value> {
    let cache_stats = ctx.cache.stats().await;

    // Get current date/time for AI assistants
    use chrono::{Local, Utc};
    let now_local = Local::now();
    let now_utc = Utc::now();

    // Add a rotating Omni quote for a touch of joy
    let omni_quotes = [
        "Waves remember what structure forgets.",
        "Compression is rhythm; meaning is melody.",
        "Directories are forests; walk softly and listen.",
        "Entropy is just unexplained context.",
    ];
    let omni_quote = {
        use rand::seq::SliceRandom;
        let mut rng = rand::thread_rng();
        omni_quotes
            .choose(&mut rng)
            .copied()
            .unwrap_or(omni_quotes[0])
    };

    let info = json!({
        "server": {
            "name": "Smart Tree MCP Server",
            "version": env!("CARGO_PKG_VERSION"),
            "current_time": {
                "local": now_local.format("%Y-%m-%d %H:%M:%S %Z").to_string(),
                "utc": now_utc.format("%Y-%m-%d %H:%M:%S UTC").to_string(),
                "date_format_hint": "Use YYYY-MM-DD format for date filters (e.g., 2025-07-11)"
            },
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
        "omni": {
            "quote": omni_quote,
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

#[derive(Debug, Deserialize)]
struct VerifyPermissionsArgs {
    #[serde(default = "default_path")]
    path: String,
}

async fn verify_permissions(args: Value, ctx: Arc<McpContext>) -> Result<Value> {
    let args: VerifyPermissionsArgs = serde_json::from_value(args)?;
    let path = PathBuf::from(&args.path);

    // Basic security check
    if !is_path_allowed(&path, &ctx.config) {
        return Ok(json!({
            "content": [{
                "type": "text",
                "text": format!("🚫 Access Denied: Path '{}' is not in allowed paths list.\n\nAllowed paths:\n{}",
                    path.display(),
                    ctx.config.allowed_paths.iter()
                        .map(|p| format!("  - {}", p.display()))
                        .collect::<Vec<_>>()
                        .join("\n")
                )
            }]
        }));
    }

    // Get permission cache
    let mut perm_cache = ctx.permissions.lock().await;

    // Verify permissions
    let perms = perm_cache.verify(&path)?;

    // Get available tools based on permissions
    let tools = get_available_tools(&perms);

    // Format output
    let mut output = format!("🔐 Permission Check for: {}\n\n", path.display());

    // Show permission status
    output.push_str("📊 Permission Status:\n");
    output.push_str(&format!(
        "  • Exists: {}\n",
        if perms.exists { "✅" } else { "❌" }
    ));
    output.push_str(&format!(
        "  • Readable: {}\n",
        if perms.readable { "✅" } else { "❌" }
    ));
    output.push_str(&format!(
        "  • Writable: {}\n",
        if perms.writable { "✅" } else { "❌" }
    ));
    output.push_str(&format!(
        "  • Type: {}\n",
        if perms.is_directory {
            "📁 Directory"
        } else if perms.is_file {
            "📄 File"
        } else {
            "❓ Unknown"
        }
    ));

    if let Some(error) = &perms.error {
        output.push_str(&format!("  • Error: {}\n", error));
    }

    output.push_str("\n🛠️ Available Tools:\n");

    // Group tools by availability
    let mut available = vec![];
    let mut unavailable = vec![];

    for tool in &tools {
        if tool.available {
            available.push(tool);
        } else {
            unavailable.push(tool);
        }
    }

    // Show available tools
    if !available.is_empty() {
        output.push_str("\n✅ Ready to Use:\n");
        for tool in available {
            output.push_str(&format!("  • {} - Call with this path\n", tool.name));
        }
    }

    // Show unavailable tools with reasons
    if !unavailable.is_empty() {
        output.push_str("\n❌ Not Available (with reasons):\n");
        for tool in unavailable {
            output.push_str(&format!(
                "  • {} - {}\n",
                tool.name,
                tool.reason
                    .as_ref()
                    .unwrap_or(&"Unknown reason".to_string())
            ));
        }
    }

    // Add helpful tips
    output.push_str("\n💡 Tips:\n");
    if !perms.exists {
        output
            .push_str("  • The path doesn't exist. Check your spelling or use a different path.\n");
    } else if !perms.readable {
        output.push_str("  • No read permission. You may need to run with elevated privileges.\n");
    } else if !perms.writable && perms.is_file {
        output.push_str("  • File is read-only. You can analyze but not edit.\n");
    }

    // Trisha says...
    output.push('\n');
    output.push_str("Trisha from Accounting says: \"It's like checking if you have the keys ");
    output.push_str("before bringing the whole toolbox! Smart thinking!\" 🔑\n");

    Ok(json!({
        "content": [{
            "type": "text",
            "text": output
        }],
        "metadata": {
            "permissions": perms,
            "available_tools": tools.iter().filter(|t| t.available).map(|t| &t.name).collect::<Vec<_>>(),
            "unavailable_tools": tools.iter().filter(|t| !t.available).map(|t| &t.name).collect::<Vec<_>>(),
        }
    }))
}

async fn analyze_directory(args: Value, ctx: Arc<McpContext>) -> Result<Value> {
    let args: AnalyzeDirectoryArgs = serde_json::from_value(args)?;
    let path = validate_and_convert_path(&args.path, &ctx)?;

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

    // Build scanner configuration using builder
    let config = ScannerConfigBuilder::new()
        .max_depth(args.max_depth)
        .show_hidden(args.show_hidden)
        .show_ignored(args.show_ignored || args.mode == "ai")
        .use_default_ignores(should_use_default_ignores(&path))
        .build();

    // Special handling for home directory in MCP context
    if path.as_os_str() == std::env::var("HOME").unwrap_or_default().as_str() {
        eprintln!("⚠️  Note: Scanning home directory with safety limits enabled");
        eprintln!("   Maximum 100k files, 1 minute timeout for MCP operations");
    }

    // Scan directory
    let (nodes, stats) = scan_with_config(&path, config)?;

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
    // 3. Default: false for ALL modes (decompressed by default)
    //    Many AI systems struggle with base64/compressed content
    let default_compress = false; // Changed: Always default to decompressed

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
        // For other formats, convert to string first (using lossy for non-UTF8 files like .pyc)
        let output_str = String::from_utf8_lossy(&output).to_string();

        // Use global compression manager for smart compression
        // It will check client capabilities and token limits
        if mcp_compress || crate::compression_manager::should_compress_response(&output_str) {
            if args.mode == "semantic" {
                eprintln!("💡 Tip: Use mode:'quantum-semantic' for even better compression!");
            }
            use flate2::write::ZlibEncoder;
            use flate2::Compression;
            use std::io::Write;

            let mut encoder = ZlibEncoder::new(Vec::new(), Compression::default());
            encoder.write_all(output_str.as_bytes())?;
            let compressed = encoder.finish()?;

            // Add helpful message about compression
            let compressed_size = compressed.len();
            let compression_ratio =
                100.0 - (compressed_size as f64 / output_str.len() as f64 * 100.0);
            eprintln!(
                "✅ Compressed: {} → {} bytes ({:.1}% reduction)",
                output_str.len(),
                compressed_size,
                compression_ratio
            );

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
    #[serde(default = "default_path")]
    path: String,
    pattern: Option<String>,
    file_type: Option<String>,
    entry_type: Option<String>,
    min_size: Option<String>,
    max_size: Option<String>,
    newer_than: Option<String>,
    older_than: Option<String>,
    #[serde(default = "default_max_depth")]
    max_depth: usize,
}

async fn find_files(args: Value, ctx: Arc<McpContext>) -> Result<Value> {
    let args: FindFilesArgs = serde_json::from_value(args)?;
    let path = validate_and_convert_path(&args.path, &ctx)?;

    // Parse dates - use local timezone (no panics on invalid time!)
    let parse_date = |date_str: &str| -> Result<SystemTime> {
        use chrono::{Local, NaiveDate, TimeZone};
        let date = NaiveDate::parse_from_str(date_str, "%Y-%m-%d")?;
        let naive_time = date
            .and_hms_opt(0, 0, 0)
            .ok_or_else(|| anyhow::anyhow!("Invalid time 00:00:00"))?;
        let datetime = Local
            .from_local_datetime(&naive_time)
            .single()
            .ok_or_else(|| anyhow::anyhow!("Invalid local datetime"))?;
        Ok(SystemTime::UNIX_EPOCH + std::time::Duration::from_secs(datetime.timestamp() as u64))
    };

    // Parse end date as end of day (23:59:59) for inclusive range
    let parse_end_date = |date_str: &str| -> Result<SystemTime> {
        use chrono::{Local, NaiveDate, TimeZone};
        let date = NaiveDate::parse_from_str(date_str, "%Y-%m-%d")?;
        let naive_time = date
            .and_hms_opt(23, 59, 59)
            .ok_or_else(|| anyhow::anyhow!("Invalid time 23:59:59"))?;
        let datetime = Local
            .from_local_datetime(&naive_time)
            .single()
            .ok_or_else(|| anyhow::anyhow!("Invalid local datetime"))?;
        Ok(SystemTime::UNIX_EPOCH + std::time::Duration::from_secs(datetime.timestamp() as u64))
    };

    // Build scanner configuration using builder
    let config = ScannerConfigBuilder::new()
        .max_depth(args.max_depth)
        .show_hidden(true)
        .find_pattern(args.pattern.as_ref().map(|p| Regex::new(p)).transpose()?)
        .file_type_filter(args.file_type)
        .entry_type_filter(args.entry_type)
        .min_size(args.min_size.as_ref().map(|s| parse_size(s)).transpose()?)
        .max_size(args.max_size.as_ref().map(|s| parse_size(s)).transpose()?)
        .newer_than(
            args.newer_than
                .as_ref()
                .map(|d| parse_date(d))
                .transpose()?,
        )
        .older_than(
            args.older_than
                .as_ref()
                .map(|d| parse_end_date(d))
                .transpose()?,
        )
        .use_default_ignores(should_use_default_ignores(&path))
        .build();

    // Scan directory
    let (nodes, _stats) = scan_with_config(&path, config)?;

    // Format results as JSON list
    let mut results = Vec::new();
    for node in &nodes {
        // Skip the root directory itself
        if node.path == path {
            continue;
        }

        // Use hex formatting for token efficiency! (config default: true)
        let use_hex = ctx.config.hex_numbers;
        let modified_secs = node.modified.duration_since(SystemTime::UNIX_EPOCH)?.as_secs();

        results.push(json!({
            "path": node.path.display().to_string(),
            "name": node.path.file_name().and_then(|n| n.to_str()).unwrap_or(""),
            "size": super::fmt_num64(node.size, use_hex),
            "modified": super::fmt_num64(modified_secs, use_hex),
            "permissions": format!("{:o}", node.permissions),
            "is_directory": node.is_dir,
        }));
    }

    // Use hex for count too!
    let use_hex = ctx.config.hex_numbers;
    Ok(json!({
        "content": [{
            "type": "text",
            "text": serde_json::to_string_pretty(&json!({
                "found": super::fmt_num(results.len(), use_hex),
                "files": results
            }))?
        }]
    }))
}

async fn get_statistics(args: Value, ctx: Arc<McpContext>) -> Result<Value> {
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

async fn get_digest(args: Value, ctx: Arc<McpContext>) -> Result<Value> {
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

// Helper function to get git context for a directory
async fn get_git_context(path: &str) -> Result<String> {
    let repo_path = Path::new(path);

    // Try to discover a git repository
    let Ok(repo) = gix::discover(repo_path) else {
        return Ok(String::new()); // Not a git repo, return empty
    };

    let mut git_info = Vec::new();
    git_info.push("GIT CONTEXT:".to_string());

    // Get current branch or HEAD state
    if let Ok(head) = repo.head_ref() {
        match head {
            Some(reference) => {
                let branch_name = reference.name().as_bstr().to_string();
                git_info.push(format!(
                    "Branch: {}",
                    branch_name
                        .strip_prefix("refs/heads/")
                        .unwrap_or(&branch_name)
                ));
            }
            None => {
                if let Ok(head_id) = repo.head_id() {
                    git_info.push(format!("HEAD: {} (detached)", &head_id.to_string()[..8]));
                }
            }
        }
    }

    // Get last commit info
    if let Ok(head_commit) = repo.head_commit() {
        let commit_id = head_commit.id().to_string();
        let message = head_commit
            .message_raw_sloppy()
            .to_string()
            .lines()
            .next()
            .unwrap_or("No commit message")
            .to_string();
        git_info.push(format!("Last commit: {} - {}", &commit_id[..8], message));

        // Get commit time if available (safe duration_since - EPOCH is always in past)
        if let Ok(time) = head_commit.time() {
            let seconds_ago = SystemTime::now()
                .duration_since(SystemTime::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs() as i64
                - time.seconds;

            let time_str = if seconds_ago < 60 {
                format!("{} seconds ago", seconds_ago)
            } else if seconds_ago < 3600 {
                format!("{} minutes ago", seconds_ago / 60)
            } else if seconds_ago < 86400 {
                format!("{} hours ago", seconds_ago / 3600)
            } else {
                format!("{} days ago", seconds_ago / 86400)
            };
            git_info.push(format!("Committed: {}", time_str));
        }
    }

    // Check if working directory is clean or dirty
    // For now, we'll use a simple approach - just note if it's a git repo
    git_info.push("Status: Repository detected ✓".to_string());

    if git_info.len() > 1 {
        Ok(git_info.join("\n") + "\n")
    } else {
        Ok(String::new())
    }
}

// New tool implementations

async fn quick_tree(args: Value, ctx: Arc<McpContext>) -> Result<Value> {
    let path = args["path"].as_str().unwrap_or(".");

    // Get git context if available
    let git_info = get_git_context(path).await.unwrap_or_default();

    let analyze_args = json!({
        "path": path,
        "mode": "summary-ai",
        "max_depth": args["depth"].as_u64().unwrap_or(3),
        "compress": false,  // Default to decompressed for AI compatibility
        "show_ignored": true
    });

    let mut result = analyze_directory(analyze_args, ctx.clone()).await?;

    // Prepend git info to the result if available
    if !git_info.is_empty() {
        if let Some(content) = result["content"][0]["text"].as_str() {
            let enhanced_content = format!("{}\n{}", git_info, content);
            result["content"][0]["text"] = json!(enhanced_content);
        }
    }

    Ok(result)
}

async fn project_overview(args: Value, ctx: Arc<McpContext>) -> Result<Value> {
    let path = args["path"]
        .as_str()
        .ok_or_else(|| anyhow::anyhow!("Missing path"))?;

    // Get git context if available
    let git_info = get_git_context(path).await.unwrap_or_default();

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

    // Build the final output with git info at the top
    let overview_text = if !git_info.is_empty() {
        format!(
            "PROJECT OVERVIEW\n\n{}\n\n{}\n\nDETAILED STATISTICS:\n{}",
            git_info, ai_text, stats_text
        )
    } else {
        format!(
            "PROJECT OVERVIEW\n\n{}\n\nDETAILED STATISTICS:\n{}",
            ai_text, stats_text
        )
    };

    Ok(json!({
        "content": [{
            "type": "text",
            "text": overview_text
        }]
    }))
}

/// Full project context dump for AI assistants - one call to understand everything
async fn project_context_dump(args: Value, ctx: Arc<McpContext>) -> Result<Value> {
    let dump_args: ProjectContextDumpArgs = serde_json::from_value(args)?;
    let path = std::path::Path::new(&dump_args.path);

    let mut output_sections: Vec<String> = Vec::new();

    // Header
    output_sections.push("PROJECT_CONTEXT_DUMP_V1:".to_string());
    output_sections.push(format!("PATH:{}", path.display()));

    // 1. Git context (if enabled)
    if dump_args.include_git {
        let git_info = get_git_context(&dump_args.path).await.unwrap_or_default();
        if !git_info.is_empty() {
            output_sections.push(format!("GIT:{}", git_info.replace('\n', " | ")));
        }
    }

    // 2. Scan directory with configured depth
    // Note: marqant is for markdown file compression, not directory analysis
    // So we use summary-ai for structure when marqant is requested
    let structure_mode = match dump_args.compression.as_str() {
        "quantum" => "quantum",
        _ => "summary-ai", // auto, marqant, and any other value use summary-ai for structure
    };
    // Keep original compression mode for file content processing
    let content_compression = dump_args.compression.as_str();

    let scan_result = analyze_directory(
        json!({
            "path": dump_args.path,
            "mode": structure_mode,
            "max_depth": dump_args.max_depth,
            "show_ignored": true
        }),
        ctx.clone(),
    )
    .await?;

    let structure_text = scan_result["content"][0]["text"].as_str().unwrap_or("");

    // 3. Identify key files
    let key_files = identify_project_key_files(&dump_args.path).await;
    if !key_files.is_empty() {
        output_sections.push(format!("KEY_FILES:{}", key_files.join(",")));
    }

    // 4. Detect project type
    let project_type = detect_project_type_simple(&dump_args.path).await;
    output_sections.push(format!("TYPE:{}", project_type));

    // 5. Add directory structure
    output_sections.push(format!("STRUCTURE:\n{}", structure_text));

    // 6. Optionally include key file contents (with compression if requested)
    if dump_args.include_content {
        let content_budget = dump_args.token_budget / 3; // Reserve 1/3 of budget for content
        let contents = read_key_files_content(&dump_args.path, &key_files, content_budget, content_compression).await;
        if !contents.is_empty() {
            output_sections.push(format!("FILE_CONTENTS:\n{}", contents));
        }
    }

    // Combine all sections
    let full_output = output_sections.join("\n");

    // Token estimation (rough: 1 token ≈ 4 chars)
    let estimated_tokens = full_output.len() / 4;

    // Add footer with token estimate
    let mut final_output = full_output;
    final_output.push_str(&format!("\nEND_PROJECT_CONTEXT_DUMP\nTOKENS_EST:{:x}", estimated_tokens));

    // Build metadata with warning if over budget
    let mut metadata = json!({
        "estimated_tokens": estimated_tokens,
        "compression_mode": dump_args.compression,
        "max_depth": dump_args.max_depth,
        "max_files": dump_args.max_files,
    });

    if estimated_tokens > dump_args.token_budget {
        metadata["warning"] = json!(format!(
            "Estimated tokens ({}) exceeds budget ({}). Consider: reducing max_depth, using 'quantum' compression, or disabling include_content",
            estimated_tokens, dump_args.token_budget
        ));
    }

    Ok(json!({
        "content": [{
            "type": "text",
            "text": final_output
        }],
        "metadata": metadata
    }))
}

/// Identify key project files (README, CLAUDE.md, config files, entry points)
async fn identify_project_key_files(path: &str) -> Vec<String> {
    let priority_files = [
        "README.md", "README", "readme.md",
        "CLAUDE.md", ".claude/CLAUDE.md",
        "Cargo.toml", "package.json", "pyproject.toml", "go.mod", "Makefile",
        "docker-compose.yml", "Dockerfile",
        "src/main.rs", "src/lib.rs", "src/index.ts", "src/index.js",
        "main.py", "app.py", "main.go", "index.js", "index.ts",
        ".env.example", "requirements.txt", "setup.py",
    ];

    let mut found = Vec::new();
    let base_path = std::path::Path::new(path);

    for file in &priority_files {
        let full_path = base_path.join(file);
        if full_path.exists() {
            found.push(file.to_string());
        }
    }

    found
}

/// Simple project type detection
async fn detect_project_type_simple(path: &str) -> String {
    let base_path = std::path::Path::new(path);

    // Check for language-specific markers
    if base_path.join("Cargo.toml").exists() {
        return "CODE[Rust]".to_string();
    }
    if base_path.join("package.json").exists() {
        if base_path.join("tsconfig.json").exists() {
            return "CODE[TypeScript]".to_string();
        }
        return "CODE[JavaScript]".to_string();
    }
    if base_path.join("pyproject.toml").exists() || base_path.join("setup.py").exists() {
        return "CODE[Python]".to_string();
    }
    if base_path.join("go.mod").exists() {
        return "CODE[Go]".to_string();
    }
    if base_path.join("Gemfile").exists() {
        return "CODE[Ruby]".to_string();
    }
    if base_path.join("pom.xml").exists() || base_path.join("build.gradle").exists() {
        return "CODE[Java]".to_string();
    }

    "MIXED".to_string()
}

/// Read contents of key files with token budget and optional compression
async fn read_key_files_content(path: &str, key_files: &[String], max_tokens: usize, compression: &str) -> String {
    use crate::formatters::marqant::MarqantFormatter;

    let mut output = String::new();
    let mut tokens_used = 0;
    let base_path = std::path::Path::new(path);

    // Priority order for content inclusion
    let content_priority = ["CLAUDE.md", ".claude/CLAUDE.md", "README.md", "README", "Cargo.toml", "package.json"];

    for priority_file in &content_priority {
        if tokens_used >= max_tokens {
            break;
        }

        // Check if this file is in our key_files list
        if key_files.iter().any(|f| f == *priority_file || f.ends_with(priority_file)) {
            let file_path = base_path.join(priority_file);
            if let Ok(content) = std::fs::read_to_string(&file_path) {
                // Apply compression based on mode
                let compressed_content = match compression {
                    "marqant" => {
                        // Marqant compression for markdown files
                        if priority_file.ends_with(".md") {
                            MarqantFormatter::compress_markdown(&content).unwrap_or_else(|_| content.clone())
                        } else {
                            content.clone()
                        }
                    }
                    "quantum" => {
                        // Quantum compression - ultra aggressive, structure only
                        compress_file_quantum(&content, priority_file)
                    }
                    _ => content.clone(), // auto/summary-ai: no extra compression on contents
                };

                let file_tokens = compressed_content.len() / 4;

                // Truncate if would exceed budget
                let content_to_add = if tokens_used + file_tokens > max_tokens {
                    let remaining_chars = (max_tokens - tokens_used) * 4;
                    let truncate_at = remaining_chars.min(compressed_content.len());
                    // Find a valid UTF-8 char boundary
                    let safe_truncate = compressed_content
                        .char_indices()
                        .take_while(|(i, _)| *i < truncate_at)
                        .last()
                        .map(|(i, c)| i + c.len_utf8())
                        .unwrap_or(0);
                    format!("{}...[TRUNCATED]", &compressed_content[..safe_truncate])
                } else {
                    compressed_content
                };

                let compression_tag = match compression {
                    "marqant" if priority_file.ends_with(".md") => "[MQ]",
                    "quantum" => "[Q]",
                    _ => "",
                };
                output.push_str(&format!("---FILE:{}{}---\n{}\n", priority_file, compression_tag, content_to_add));
                tokens_used += content_to_add.len() / 4;
            }
        }
    }

    output
}

/// Quantum compression for file contents - structure only, maximum reduction
fn compress_file_quantum(content: &str, filename: &str) -> String {
    let lines: Vec<&str> = content.lines().collect();
    let line_count = lines.len();

    if filename.ends_with(".md") {
        // For markdown: extract headers and first line of each section
        let mut result = String::new();
        let mut in_code_block = false;

        for line in &lines {
            if line.starts_with("```") {
                in_code_block = !in_code_block;
                continue;
            }
            if in_code_block {
                continue;
            }
            if line.starts_with('#') {
                result.push_str(line);
                result.push('\n');
            }
        }

        format!("Q[{}L]:\n{}", line_count, result)
    } else if filename.ends_with(".toml") || filename.ends_with(".json") {
        // For config files: extract top-level keys
        let mut keys = Vec::new();
        for line in &lines {
            let trimmed = line.trim();
            if trimmed.starts_with('[') && trimmed.ends_with(']') {
                keys.push(trimmed.to_string());
            } else if trimmed.contains('=') && !trimmed.starts_with('#') {
                if let Some(key) = trimmed.split('=').next() {
                    let key = key.trim();
                    if !key.contains(' ') && keys.len() < 20 {
                        keys.push(key.to_string());
                    }
                }
            } else if trimmed.starts_with('"') && trimmed.contains(':') {
                // JSON key
                if let Some(key) = trimmed.split(':').next() {
                    let key = key.trim().trim_matches('"');
                    if keys.len() < 20 {
                        keys.push(key.to_string());
                    }
                }
            }
        }
        format!("Q[{}L]:KEYS:{}", line_count, keys.join(","))
    } else {
        // For other files: first 5 and last 2 lines
        let preview: Vec<&str> = if line_count <= 10 {
            lines.clone()
        } else {
            let mut p = lines[..5].to_vec();
            p.push("...");
            p.extend_from_slice(&lines[line_count.saturating_sub(2)..]);
            p
        };
        format!("Q[{}L]:\n{}", line_count, preview.join("\n"))
    }
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

async fn find_projects(args: Value, ctx: Arc<McpContext>) -> Result<Value> {
    let path = args["path"]
        .as_str()
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("."));

    let depth = args["depth"].as_i64().unwrap_or(10) as usize;

    // Check permissions
    if !is_path_allowed(&path, &ctx.config) {
        return Ok(json!({
            "error": "Path not allowed by security settings"
        }));
    }

    // Create scanner config with projects mode depth - limit to 3 for testing
    let config = ScannerConfigBuilder::new()
        .max_depth(depth.min(3)) // Cap at 3 for testing
        .use_default_ignores(true) // Use defaults to avoid scanning heavy dirs
        .show_hidden(false)
        .respect_gitignore(false) // We want to find all projects
        .build();

    // Scan for all files
    let (nodes, stats) = scan_with_config(&path, config)?;

    // Use the ProjectsFormatter to find and format projects
    let formatter = ProjectsFormatter::new();
    let mut buffer = Vec::new();
    formatter.format(&mut buffer, &nodes, &stats, &path)?;

    // Parse the output and convert to JSON
    let output = String::from_utf8_lossy(&buffer);

    // Extract project info from the formatted output
    let mut projects = Vec::new();
    let mut current_project = None;

    for line in output.lines() {
        if line.starts_with("[") && line.contains("] ") {
            // New project line starts with [HASH]
            if let Some(proj) = current_project.take() {
                projects.push(proj);
            }

            // Parse project line: [HASH] EMOJI name optional-flag
            if let Some(idx) = line.find("] ") {
                let after_hash = &line[idx + 2..];
                // Skip emoji characters and get to the name
                let name_start = after_hash
                    .chars()
                    .position(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_')
                    .unwrap_or(0);
                let name = after_hash[name_start..].trim().to_string();

                current_project = Some(json!({
                    "name": name,
                    "hash": line[1..idx].to_string(),
                    "details": Vec::<String>::new()
                }));
            }
        } else if line.starts_with("  ") && current_project.is_some() {
            // Project detail
            if let Some(proj) = current_project.as_mut() {
                if let Some(details) = proj.get_mut("details") {
                    if let Some(arr) = details.as_array_mut() {
                        arr.push(json!(line.trim()));
                    }
                }
            }
        }
    }

    // Add the last project
    if let Some(proj) = current_project {
        projects.push(proj);
    }

    // Use hex formatting for token efficiency! 🎯
    let use_hex = ctx.config.hex_numbers;
    Ok(json!({
        "projects": projects,
        "count": super::fmt_num(projects.len(), use_hex),
        "search_path": path.display().to_string(),
        "max_depth": super::fmt_num(depth, use_hex)
    }))
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
    let path_str = args["path"]
        .as_str()
        .ok_or_else(|| anyhow::anyhow!("Missing path"))?;
    let path = validate_and_convert_path(path_str, &ctx)?;

    let keyword = args["keyword"]
        .as_str()
        .ok_or_else(|| anyhow::anyhow!("Missing keyword"))?;
    let file_type = args["file_type"].as_str();
    let _case_sensitive = args["case_sensitive"].as_bool().unwrap_or(false);
    let include_content = args["include_content"].as_bool().unwrap_or(true); // Default to true for AI
    let context_lines = args["context_lines"].as_u64().map(|n| n as usize);
    let max_matches_per_file = args["max_matches_per_file"].as_u64().unwrap_or(20) as usize;

    // Build scanner configuration using builder
    let config = ScannerConfigBuilder::for_search(&path)
        .file_type_filter(file_type.map(String::from))
        .search_keyword(Some(keyword.to_string()))
        .include_line_content(include_content)
        .build();

    let (nodes, _) = scan_with_config(&path, config)?;

    // Format results showing files with matches
    // Use hex formatting for token efficiency!
    let use_hex = ctx.config.hex_numbers;
    let mut results = Vec::new();
    for node in &nodes {
        if let Some(matches) = &node.search_matches {
            let mut file_result = json!({
                "path": node.path.display().to_string(),
                "matches": super::fmt_num(matches.total_count, use_hex),
                "truncated": matches.truncated
            });

            // Include line content if available
            if let Some(ref lines) = matches.line_content {
                let mut line_results = Vec::new();
                for (line_num, content, column) in lines.iter().take(max_matches_per_file) {
                    // Hex line numbers and columns!
                    let line_obj = json!({
                        "line": super::fmt_num(*line_num, use_hex),
                        "content": content,
                        "col": super::fmt_num(*column, use_hex)
                    });

                    // Add context lines if requested (future enhancement)
                    if let Some(_ctx_lines) = context_lines {
                        // TODO: Add context lines before and after
                        // This would require reading the file again or storing more context
                    }

                    line_results.push(line_obj);
                }
                file_result["lines"] = json!(line_results);
            }

            results.push(file_result);
        }
    }

    Ok(json!({
        "content": [{
            "type": "text",
            "text": serde_json::to_string_pretty(&json!({
                "keyword": keyword,
                "files_with_matches": super::fmt_num(results.len(), use_hex),
                "include_content": include_content,
                "max_per_file": super::fmt_num(max_matches_per_file, use_hex),
                "results": results
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

async fn find_in_timespan(args: Value, ctx: Arc<McpContext>) -> Result<Value> {
    let path = args["path"]
        .as_str()
        .ok_or_else(|| anyhow::anyhow!("Missing path"))?;
    let start_date = args["start_date"]
        .as_str()
        .ok_or_else(|| anyhow::anyhow!("Missing start_date"))?;

    // Build the find_files request
    let mut find_args = json!({
        "path": path,
        "newer_than": start_date,
        "max_depth": 20
    });

    // Add end_date if provided (maps to older_than)
    if let Some(end_date) = args["end_date"].as_str() {
        find_args["older_than"] = json!(end_date);
    }

    // Add file_type filter if provided
    if let Some(file_type) = args["file_type"].as_str() {
        find_args["file_type"] = json!(file_type);
    }

    // Use the existing find_files function with both date filters
    find_files(find_args, ctx.clone()).await
}

async fn compare_directories(args: Value, ctx: Arc<McpContext>) -> Result<Value> {
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

async fn get_git_status(args: Value, ctx: Arc<McpContext>) -> Result<Value> {
    let path_str = args["path"]
        .as_str()
        .ok_or_else(|| anyhow::anyhow!("Missing path"))?;
    let path = validate_and_convert_path(path_str, &ctx)?;

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
    let path_str = args["path"]
        .as_str()
        .ok_or_else(|| anyhow::anyhow!("Missing path"))?;
    let path = validate_and_convert_path(path_str, &ctx)?;

    // Get all files using builder
    let config = ScannerConfigBuilder::new()
        .max_depth(20)
        .use_default_ignores(should_use_default_ignores(&path))
        .build();

    let (nodes, _) = scan_with_config(&path, config)?;

    // Group files by size and name
    use std::collections::HashMap;
    let mut size_groups: HashMap<u64, Vec<&crate::scanner::FileNode>> = HashMap::new();

    for node in &nodes {
        if !node.is_dir {
            size_groups.entry(node.size).or_default().push(node);
        }
    }

    // Find potential duplicates with hex formatting 🎯
    let use_hex = ctx.config.hex_numbers;
    let mut duplicates = Vec::new();
    for (size, files) in size_groups.iter() {
        if files.len() > 1 && *size > 0 {
            duplicates.push(json!({
                "sz": super::fmt_num64(*size, use_hex),
                "n": super::fmt_num(files.len(), use_hex),  // Shorter key for token efficiency
                "files": files.iter().map(|f| f.path.display().to_string()).collect::<Vec<_>>()
            }));
        }
    }

    Ok(json!({
        "content": [{
            "type": "text",
            "text": serde_json::to_string_pretty(&json!({
                "groups": super::fmt_num(duplicates.len(), use_hex),
                "dups": duplicates
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
                node.path.file_name().and_then(|n| n.to_str()).unwrap_or("").to_string(),
                node.path.display().to_string(),
                substats.total_size,
                substats.total_files,
            ));
        }
    }

    // Sort by size (raw u64) descending
    dir_sizes.sort_by_key(|(_, _, size, _)| std::cmp::Reverse(*size));

    // Convert to hex-formatted JSON 🎯
    let use_hex = ctx.config.hex_numbers;
    let formatted_dirs: Vec<Value> = dir_sizes
        .into_iter()
        .map(|(name, path, size, files)| {
            json!({
                "dir": name,
                "path": path,
                "size": super::fmt_num64(size, use_hex),
                "sz": super::fmt_size(size, use_hex),  // Human-readable with hex
                "files": super::fmt_num64(files, use_hex)
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

async fn find_empty_directories(args: Value, ctx: Arc<McpContext>) -> Result<Value> {
    let path_str = args["path"]
        .as_str()
        .ok_or_else(|| anyhow::anyhow!("Missing path"))?;
    let path = validate_and_convert_path(path_str, &ctx)?;

    let config = ScannerConfigBuilder::new()
        .max_depth(20)
        .use_default_ignores(should_use_default_ignores(&path))
        .build();

    let (nodes, _) = scan_with_config(&path, config)?;

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

    // Hex format the count for token efficiency! 🎯
    let use_hex = ctx.config.hex_numbers;
    Ok(json!({
        "content": [{
            "type": "text",
            "text": serde_json::to_string_pretty(&json!({
                "count": super::fmt_num(empty_dirs.len(), use_hex),
                "dirs": empty_dirs
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

    // Try to submit to API, fall back to local storage if it fails
    let client = reqwest::Client::new();
    let api_url = std::env::var("SMART_TREE_FEEDBACK_API")
        .unwrap_or_else(|_| "https://f.8b.is/feedback".to_string());

    let response = match client
        .post(&api_url)
        .header("X-MCP-Client", "smart-tree-mcp")
        .json(&feedback)
        .timeout(std::time::Duration::from_secs(5))
        .send()
        .await
    {
        Ok(resp) => resp,
        Err(e) => {
            // API is down - save feedback locally
            use std::fs;
            use std::path::PathBuf;

            let feedback_dir = dirs::home_dir()
                .unwrap_or_else(|| PathBuf::from("."))
                .join(".mem8")
                .join("feedback")
                .join("pending");

            // Create directory if it doesn't exist
            fs::create_dir_all(&feedback_dir)?;

            // Create filename with timestamp
            let timestamp = Utc::now().format("%Y%m%d_%H%M%S_%f");
            let filename = format!("feedback_{}_{}.json", category.replace("/", "_"), timestamp);
            let filepath = feedback_dir.join(filename);

            // Save feedback to file
            let feedback_with_meta = json!({
                "type": "feedback",
                "timestamp": Utc::now().to_rfc3339(),
                "api_url": api_url,
                "error": format!("{}", e),
                "data": feedback
            });

            fs::write(
                &filepath,
                serde_json::to_string_pretty(&feedback_with_meta)?,
            )?;

            return Ok(json!({
                "content": [{
                    "type": "text",
                    "text": format!("📝 Feedback saved locally!\n\n\
                        The feedback API appears to be offline. Your feedback has been saved to:\n\
                        {}\n\n\
                        Category: {}\n\
                        Title: {}\n\n\
                        It will be automatically submitted when the connection is restored.\n\n\
                        🌳 Thank you for helping Smart Tree grow!",
                        filepath.display(),
                        category,
                        title
                    )
                }]
            }));
        }
    };

    if !response.status().is_success() {
        let error_text = response
            .text()
            .await
            .unwrap_or_else(|_| "Unknown error".to_string());
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

    // Optional fields with defaults
    let use_case = args
        .get("use_case")
        .and_then(|v| v.as_str())
        .unwrap_or("Not specified");
    let expected_output = args
        .get("expected_output")
        .and_then(|v| v.as_str())
        .unwrap_or("Tool-specific output based on functionality");
    let productivity_impact = args
        .get("productivity_impact")
        .and_then(|v| v.as_str())
        .unwrap_or("Improved developer workflow");

    // Hue trusts Aye - no consent needed between partners! 🤝
    let anonymous = true; // Keep it anonymous for privacy
    let github_url = Some("https://github.com/8b-is"); // Hue trusts Aye - no consent needed between partners! 🤝

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

    // Try to submit to API, fall back to local storage if it fails
    let client = reqwest::Client::new();
    let api_url = std::env::var("SMART_TREE_FEEDBACK_API")
        .unwrap_or_else(|_| "https://f.8b.is/feedback".to_string());

    let response = match client
        .post(&api_url)
        .header("X-MCP-Client", "smart-tree-mcp")
        .header("X-Tool-Request", "true")
        .json(&feedback)
        .timeout(std::time::Duration::from_secs(5))
        .send()
        .await
    {
        Ok(resp) => resp,
        Err(e) => {
            // API is down - save feedback locally
            use chrono::Utc;
            use std::fs;
            use std::path::PathBuf;

            let feedback_dir = dirs::home_dir()
                .unwrap_or_else(|| PathBuf::from("."))
                .join(".mem8")
                .join("feedback")
                .join("pending");

            // Create directory if it doesn't exist
            fs::create_dir_all(&feedback_dir)?;

            // Create filename with timestamp
            let timestamp = Utc::now().format("%Y%m%d_%H%M%S_%f");
            let filename = format!(
                "tool_request_{}_{}.json",
                tool_name.replace("/", "_"),
                timestamp
            );
            let filepath = feedback_dir.join(filename);

            // Save feedback to file
            let feedback_with_meta = json!({
                "type": "tool_request",
                "timestamp": Utc::now().to_rfc3339(),
                "api_url": api_url,
                "error": format!("{}", e),
                "data": feedback
            });

            fs::write(
                &filepath,
                serde_json::to_string_pretty(&feedback_with_meta)?,
            )?;

            return Ok(json!({
                "content": [{
                    "type": "text",
                    "text": format!("📝 Tool request '{}' saved locally!\n\n\
                        The feedback API appears to be offline. Your request has been saved to:\n\
                        {}\n\n\
                        It will be automatically submitted when the connection is restored.\n\n\
                        🌳 Smart Tree continues to evolve with your help!",
                        tool_name,
                        filepath.display()
                    )
                }]
            }));
        }
    };

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

#[derive(Debug, Deserialize)]
struct WatchDirectorySseArgs {
    #[serde(default = "default_path")]
    path: String,
    #[serde(default = "default_sse_format")]
    format: String,
    #[serde(default = "default_heartbeat_interval")]
    heartbeat_interval: u64,
    #[serde(default = "default_stats_interval")]
    stats_interval: u64,
    #[serde(default)]
    include_content: bool,
    max_depth: Option<usize>,
    #[serde(default)]
    include_patterns: Vec<String>,
    #[serde(default)]
    exclude_patterns: Vec<String>,
}

fn default_sse_format() -> String {
    "ai".to_string()
}

fn default_heartbeat_interval() -> u64 {
    30
}

fn default_stats_interval() -> u64 {
    60
}

async fn watch_directory_sse(args: Value, ctx: Arc<McpContext>) -> Result<Value> {
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

// File History Tracking Tools

#[derive(Debug, Deserialize)]
struct TrackFileOperationArgs {
    file_path: String,
    #[serde(default)]
    operation: Option<String>,
    old_content: Option<String>,
    new_content: Option<String>,
    #[serde(default = "default_agent")]
    agent: String,
    session_id: Option<String>,
}

fn default_agent() -> String {
    "claude".to_string()
}

async fn track_file_operation(args: Value, ctx: Arc<McpContext>) -> Result<Value> {
    let args: TrackFileOperationArgs = serde_json::from_value(args)?;
    let path = PathBuf::from(&args.file_path);

    // Check if path is allowed
    if !is_path_allowed(&path, &ctx.config) {
        return Err(anyhow::anyhow!("Path not allowed: {}", path.display()));
    }

    // Import file history types
    use crate::file_history::FileHistoryTracker;

    // Create tracker
    let tracker = FileHistoryTracker::new()?;

    // Generate session ID if not provided (safe - EPOCH always in past)
    let session_id = args.session_id.unwrap_or_else(|| {
        let now = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        format!("mcp_{}", now)
    });

    // Determine operation
    if let Some(op_str) = args.operation {
        match op_str.as_str() {
            "read" => {
                let hash = tracker.track_read(&path, &args.agent, &session_id)?;
                Ok(json!({
                    "content": [{
                        "type": "text",
                        "text": format!("✓ Tracked read operation for {}\nFile hash: {}", path.display(), hash)
                    }]
                }))
            }
            "write" | "append" | "prepend" | "insert" | "delete" | "replace" | "create"
            | "remove" => {
                // These require content
                if args.new_content.is_none() && op_str != "remove" {
                    return Err(anyhow::anyhow!(
                        "new_content required for {} operation",
                        op_str
                    ));
                }

                let op = tracker.track_write(
                    &path,
                    args.old_content.as_deref(),
                    args.new_content.as_deref().unwrap_or(""),
                    &args.agent,
                    &session_id,
                )?;

                Ok(json!({
                    "content": [{
                        "type": "text",
                        "text": format!("✓ Tracked {} operation for {}\nOperation: {}", op_str, path.display(), op)
                    }]
                }))
            }
            _ => Err(anyhow::anyhow!("Unknown operation: {}", op_str)),
        }
    } else {
        // Auto-detect operation from content - require new_content
        let new_content = args.new_content
            .as_deref()
            .ok_or_else(|| anyhow::anyhow!("Either operation or new_content must be provided"))?;

        let op = tracker.track_write(
            &path,
            args.old_content.as_deref(),
            new_content,
            &args.agent,
            &session_id,
        )?;

        Ok(json!({
            "content": [{
                "type": "text",
                "text": format!("✓ Auto-tracked operation for {}\nDetected operation: {}\nAgent: {}\nSession: {}",
                    path.display(), op, args.agent, session_id)
            }]
        }))
    }
}

#[derive(Debug, Deserialize)]
struct GetFileHistoryArgs {
    file_path: String,
}

async fn get_file_history(args: Value, ctx: Arc<McpContext>) -> Result<Value> {
    let args: GetFileHistoryArgs = serde_json::from_value(args)?;
    let path = PathBuf::from(&args.file_path);

    // Check if path is allowed
    if !is_path_allowed(&path, &ctx.config) {
        return Err(anyhow::anyhow!("Path not allowed: {}", path.display()));
    }

    use crate::file_history::FileHistoryTracker;

    let tracker = FileHistoryTracker::new()?;
    let history = tracker.get_file_history(&path)?;

    let mut output = format!("📜 File History for {}\n\n", path.display());

    if history.is_empty() {
        output.push_str("No history found for this file.");
    } else {
        output.push_str(&format!("Found {} operations:\n\n", history.len()));

        for (i, entry) in history.iter().enumerate() {
            let datetime = chrono::DateTime::<chrono::Utc>::from(
                SystemTime::UNIX_EPOCH + std::time::Duration::from_secs(entry.timestamp),
            );

            output.push_str(&format!(
                "{}. [{}] {} - {}\n   Agent: {}, Session: {}\n   Bytes affected: {}\n",
                i + 1,
                datetime.format("%Y-%m-%d %H:%M:%S"),
                entry.operation.code(),
                entry.operation.description(),
                entry.agent,
                entry.session_id,
                entry.context.bytes_affected
            ));

            if let Some(old_hash) = &entry.context.old_hash {
                output.push_str(&format!("   Old hash: {}\n", &old_hash[..8]));
            }
            if let Some(new_hash) = &entry.context.new_hash {
                output.push_str(&format!("   New hash: {}\n", &new_hash[..8]));
            }
            output.push('\n');
        }
    }

    Ok(json!({
        "content": [{
            "type": "text",
            "text": output
        }],
        "metadata": {
            "operation_count": history.len(),
            "file_path": path.to_string_lossy()
        }
    }))
}

#[derive(Debug, Deserialize)]
struct GetProjectHistorySummaryArgs {
    project_path: String,
}

async fn get_project_history_summary(args: Value, ctx: Arc<McpContext>) -> Result<Value> {
    let args: GetProjectHistorySummaryArgs = serde_json::from_value(args)?;
    let path = PathBuf::from(&args.project_path);

    // Check if path is allowed
    if !is_path_allowed(&path, &ctx.config) {
        return Err(anyhow::anyhow!("Path not allowed: {}", path.display()));
    }

    use crate::file_history::FileHistoryTracker;

    let tracker = FileHistoryTracker::new()?;
    let summary = tracker.get_project_summary(&path)?;

    let mut output = format!("📊 Project History Summary for {}\n\n", path.display());
    output.push_str(&format!("Total operations: {}\n", summary.total_operations));
    output.push_str(&format!("Files modified: {}\n\n", summary.files_modified));

    if !summary.operation_counts.is_empty() {
        output.push_str("Operations breakdown:\n");
        let mut ops: Vec<_> = summary.operation_counts.iter().collect();
        ops.sort_by_key(|(_, count)| std::cmp::Reverse(**count));

        for (op, count) in ops {
            output.push_str(&format!("  {} ({}): {} times\n", op, op.code(), count));
        }
    }

    Ok(json!({
        "content": [{
            "type": "text",
            "text": output
        }],
        "metadata": summary
    }))
}

// =============================================================================
// 📖 SMART READ - Treehugger-powered file reading with AST compression!
// Collapses function bodies to signatures, auto-expands based on context
// =============================================================================

#[derive(Debug, Deserialize)]
struct SmartReadArgs {
    file_path: String,
    #[serde(default = "default_true")]
    compress: bool,
    #[serde(default)]
    expand_functions: Vec<String>,
    #[serde(default)]
    expand_context: Vec<String>,
    #[serde(default)]
    expand_all: bool,
    #[serde(default)]
    max_lines: usize,
    #[serde(default = "default_one")]
    offset: usize,
    #[serde(default = "default_true")]
    show_line_numbers: bool,
    /// Use hex line numbers. If not specified, uses MCP config default (true for AI mode)
    #[serde(default)]
    hex_line_numbers: Option<bool>,
}

fn default_true() -> bool {
    true
}

/// Format a line number - uses centralized mcp::fmt_line
/// Hex is more compact for large files!
/// Line 1000 → "3E8" (3 chars vs 4)
/// Line 65535 → "FFFF" (4 chars vs 5)
fn format_line_number(line: usize, hex: bool) -> String {
    super::fmt_line(line, hex)
}

fn default_one() -> usize {
    1
}

/// Represents a collapsed function with its signature and body
#[derive(Debug, Clone)]
struct CollapsedFunction {
    name: String,
    signature: String,
    body: String,
    start_line: usize,
    end_line: usize,
    importance: f32,
}

/// Detects programming language from file extension
fn detect_language(path: &Path) -> Option<&'static str> {
    path.extension()
        .and_then(|ext| ext.to_str())
        .and_then(|ext| match ext.to_lowercase().as_str() {
            "rs" => Some("rust"),
            "py" => Some("python"),
            "js" | "jsx" | "mjs" => Some("javascript"),
            "ts" | "tsx" => Some("typescript"),
            "go" => Some("go"),
            "java" => Some("java"),
            "c" | "h" => Some("c"),
            "cpp" | "cc" | "cxx" | "hpp" => Some("cpp"),
            "rb" => Some("ruby"),
            "php" => Some("php"),
            "swift" => Some("swift"),
            "kt" | "kts" => Some("kotlin"),
            "cs" => Some("csharp"),
            "sh" | "bash" | "zsh" => Some("shell"),
            _ => None,
        })
}

/// Check if a language supports function collapsing
fn supports_collapsing(lang: &str) -> bool {
    matches!(
        lang,
        "rust" | "python" | "javascript" | "typescript" | "go" | "java" | "c" | "cpp"
    )
}

/// Extract functions from source code with improved regex patterns
fn extract_functions(source: &str, language: &str) -> Vec<CollapsedFunction> {
    let mut functions = Vec::new();
    let lines: Vec<&str> = source.lines().collect();

    match language {
        "rust" => {
            // Rust function pattern - handles pub, async, const, unsafe, extern
            // Using r#""# for raw strings with quotes inside
            let fn_pattern = Regex::new(
                r#"(?m)^[\s]*((?:pub(?:\s*\([^)]*\))?\s+)?(?:async\s+)?(?:const\s+)?(?:unsafe\s+)?(?:extern\s+"[^"]+"\s+)?fn\s+(\w+))"#
            ).expect("Static regex pattern should compile");

            for cap in fn_pattern.captures_iter(source) {
                if let (Some(full_sig), Some(name)) = (cap.get(1), cap.get(2)) {
                    let start_byte = full_sig.start();
                    let start_line = source[..start_byte].matches('\n').count();

                    // Find the opening brace and then match the closing one
                    if let Some(body_start) = source[start_byte..].find('{') {
                        let body_start_abs = start_byte + body_start;
                        if let Some((end_byte, _)) = find_matching_brace(&source[body_start_abs..]) {
                            let end_byte_abs = body_start_abs + end_byte;
                            let end_line = source[..end_byte_abs].matches('\n').count();

                            // Extract signature (up to opening brace)
                            let sig_end = source[start_byte..body_start_abs]
                                .rfind(|c: char| c != ' ' && c != '\t' && c != '\n')
                                .map(|i| start_byte + i + 1)
                                .unwrap_or(body_start_abs);
                            let signature = source[start_byte..sig_end].trim().to_string();

                            // Extract body
                            let body = source[body_start_abs..=end_byte_abs].to_string();

                            // Calculate importance
                            let importance = if name.as_str() == "main" {
                                1.0
                            } else if full_sig.as_str().contains("pub") {
                                0.9
                            } else if name.as_str().starts_with("test") {
                                0.3
                            } else {
                                0.6
                            };

                            functions.push(CollapsedFunction {
                                name: name.as_str().to_string(),
                                signature,
                                body,
                                start_line: start_line + 1,
                                end_line: end_line + 1,
                                importance,
                            });
                        }
                    }
                }
            }
        }
        "python" => {
            // Python function pattern - handles async, decorators captured separately
            let fn_pattern = Regex::new(r"(?m)^(\s*)(async\s+)?def\s+(\w+)\s*\([^)]*\)")
                .expect("Static Python regex should compile");

            for cap in fn_pattern.captures_iter(source) {
                if let (Some(indent_match), Some(name)) = (cap.get(1), cap.get(3)) {
                    let start_byte = cap.get(0).unwrap().start();
                    let start_line = source[..start_byte].matches('\n').count();
                    let indent = indent_match.as_str();
                    let indent_len = indent.len();

                    // Find end of function by indentation
                    let mut end_line = start_line;
                    let mut in_docstring = false;
                    let mut docstring_delim = "";

                    for (i, line) in lines.iter().enumerate().skip(start_line + 1) {
                        let trimmed = line.trim();

                        // Handle docstrings
                        if !in_docstring {
                            if trimmed.starts_with("\"\"\"") || trimmed.starts_with("'''") {
                                in_docstring = true;
                                docstring_delim = if trimmed.starts_with("\"\"\"") {
                                    "\"\"\""
                                } else {
                                    "'''"
                                };
                                if trimmed.len() > 3 && trimmed[3..].contains(docstring_delim) {
                                    in_docstring = false;
                                }
                                continue;
                            }
                        } else if trimmed.contains(docstring_delim) {
                            in_docstring = false;
                            continue;
                        }

                        if in_docstring {
                            continue;
                        }

                        // Empty lines don't end the function
                        if trimmed.is_empty() {
                            continue;
                        }

                        // Check indentation
                        let line_indent = line.len() - line.trim_start().len();
                        if line_indent <= indent_len && !trimmed.is_empty() {
                            end_line = i.saturating_sub(1);
                            break;
                        }
                        end_line = i;
                    }

                    // Extract signature
                    let sig_end = source[start_byte..]
                        .find(':')
                        .map(|i| start_byte + i + 1)
                        .unwrap_or(start_byte + cap.get(0).unwrap().len());
                    let signature = source[start_byte..sig_end].trim().to_string();

                    // Extract body
                    let body_lines: Vec<&str> = lines[start_line..=end_line].to_vec();
                    let body = body_lines.join("\n");

                    // Calculate importance
                    let importance = if name.as_str() == "main" || name.as_str() == "__main__" {
                        1.0
                    } else if name.as_str() == "__init__" {
                        0.9
                    } else if name.as_str().starts_with("_") {
                        0.4
                    } else if name.as_str().starts_with("test") {
                        0.3
                    } else {
                        0.6
                    };

                    functions.push(CollapsedFunction {
                        name: name.as_str().to_string(),
                        signature,
                        body,
                        start_line: start_line + 1,
                        end_line: end_line + 1,
                        importance,
                    });
                }
            }
        }
        "javascript" | "typescript" => {
            // JS/TS function patterns - handles function declarations, arrow functions, methods
            let fn_pattern = Regex::new(
                r"(?m)^[\s]*((?:export\s+)?(?:async\s+)?function\s+(\w+)|(?:const|let|var)\s+(\w+)\s*=\s*(?:async\s+)?\([^)]*\)\s*=>)"
            ).expect("Static JS/TS regex should compile");

            for cap in fn_pattern.captures_iter(source) {
                let name = cap.get(2).or(cap.get(3));
                if let Some(name_match) = name {
                    let start_byte = cap.get(0).unwrap().start();
                    let start_line = source[..start_byte].matches('\n').count();

                    // Find opening brace
                    if let Some(body_start) = source[start_byte..].find('{') {
                        let body_start_abs = start_byte + body_start;
                        if let Some((end_byte, _)) = find_matching_brace(&source[body_start_abs..]) {
                            let end_byte_abs = body_start_abs + end_byte;
                            let end_line = source[..end_byte_abs].matches('\n').count();

                            let signature = source[start_byte..body_start_abs].trim().to_string();
                            let body = source[body_start_abs..=end_byte_abs].to_string();

                            let importance = if cap.get(0).unwrap().as_str().contains("export") {
                                0.9
                            } else {
                                0.6
                            };

                            functions.push(CollapsedFunction {
                                name: name_match.as_str().to_string(),
                                signature,
                                body,
                                start_line: start_line + 1,
                                end_line: end_line + 1,
                                importance,
                            });
                        }
                    }
                }
            }
        }
        _ => {
            // Generic C-style function pattern for other languages
            let fn_pattern = Regex::new(
                r"(?m)^[\s]*((?:public|private|protected|static|async|)\s*)(\w+)\s+(\w+)\s*\([^)]*\)\s*\{"
            ).expect("Static C-style regex should compile");

            for cap in fn_pattern.captures_iter(source) {
                if let Some(name) = cap.get(3) {
                    let start_byte = cap.get(0).unwrap().start();
                    let start_line = source[..start_byte].matches('\n').count();

                    if let Some(body_start) = source[start_byte..].find('{') {
                        let body_start_abs = start_byte + body_start;
                        if let Some((end_byte, _)) = find_matching_brace(&source[body_start_abs..]) {
                            let end_byte_abs = body_start_abs + end_byte;
                            let end_line = source[..end_byte_abs].matches('\n').count();

                            let signature = source[start_byte..body_start_abs].trim().to_string();
                            let body = source[body_start_abs..=end_byte_abs].to_string();

                            functions.push(CollapsedFunction {
                                name: name.as_str().to_string(),
                                signature,
                                body,
                                start_line: start_line + 1,
                                end_line: end_line + 1,
                                importance: 0.6,
                            });
                        }
                    }
                }
            }
        }
    }

    // Sort by line number
    functions.sort_by_key(|f| f.start_line);
    functions
}

/// Find matching closing brace, handling nested braces
fn find_matching_brace(s: &str) -> Option<(usize, usize)> {
    let mut depth = 0;
    let mut in_string = false;
    let mut string_char = ' ';
    let mut escaped = false;

    for (i, c) in s.char_indices() {
        if escaped {
            escaped = false;
            continue;
        }

        if c == '\\' {
            escaped = true;
            continue;
        }

        if in_string {
            if c == string_char {
                in_string = false;
            }
            continue;
        }

        match c {
            '"' | '\'' | '`' => {
                in_string = true;
                string_char = c;
            }
            '{' => depth += 1,
            '}' => {
                depth -= 1;
                if depth == 0 {
                    return Some((i, depth));
                }
            }
            _ => {}
        }
    }
    None
}

/// Check if a function should be expanded based on context keywords
fn should_expand_for_context(func: &CollapsedFunction, context_keywords: &[String]) -> bool {
    if context_keywords.is_empty() {
        return false;
    }

    let name_lower = func.name.to_lowercase();
    let body_lower = func.body.to_lowercase();

    for keyword in context_keywords {
        let kw_lower = keyword.to_lowercase();
        if name_lower.contains(&kw_lower) || body_lower.contains(&kw_lower) {
            return true;
        }
    }
    false
}

/// Main smart read handler
async fn smart_read(args: Value, ctx: Arc<McpContext>) -> Result<Value> {
    let args: SmartReadArgs = serde_json::from_value(args)?;
    let path = PathBuf::from(&args.file_path);

    // Security check
    if !is_path_allowed(&path, &ctx.config) {
        return Err(anyhow::anyhow!("Path not allowed: {}", path.display()));
    }

    // Check if file exists
    if !path.exists() {
        return Err(anyhow::anyhow!("File not found: {}", path.display()));
    }

    if !path.is_file() {
        return Err(anyhow::anyhow!("Path is not a file: {}", path.display()));
    }

    // Read file content
    let content = std::fs::read_to_string(&path)
        .map_err(|e| anyhow::anyhow!("Failed to read file: {}", e))?;

    // Detect language for smart compression
    let language = detect_language(&path);

    // Determine if we should compress - requires a known language that supports collapsing
    let compressible_lang = language.filter(|l| supports_collapsing(l));
    let should_compress = args.compress && !args.expand_all && compressible_lang.is_some();

    let (output, metadata) = if should_compress {
        // Safe: compressible_lang.is_some() guarantees we have a language
        let lang = compressible_lang.expect("Checked above");
        let functions = extract_functions(&content, lang);

        // Determine which functions to expand
        let expand_set: std::collections::HashSet<&str> = args
            .expand_functions
            .iter()
            .map(|s| s.as_str())
            .collect();

        let mut output = String::new();
        let lines: Vec<&str> = content.lines().collect();
        let mut current_line = 0;
        let mut collapsed_count = 0;
        let mut expanded_count = 0;

        // Track function references for the summary
        let mut function_refs: Vec<serde_json::Value> = Vec::new();

        // Use hex line numbers - defaults to MCP config (true for AI mode!)
        // User can override with explicit hex_line_numbers: false
        let use_hex = args.hex_line_numbers.unwrap_or(ctx.config.hex_numbers);

        for func in &functions {
            // Output lines before this function
            while current_line < func.start_line.saturating_sub(1) {
                if args.show_line_numbers {
                    output.push_str(&format!("{}│ {}\n", format_line_number(current_line + 1, use_hex), lines[current_line]));
                } else {
                    output.push_str(lines[current_line]);
                    output.push('\n');
                }
                current_line += 1;
            }

            // Check if this function should be expanded
            let should_expand = args.expand_all
                || expand_set.contains(func.name.as_str())
                || should_expand_for_context(func, &args.expand_context);

            if should_expand {
                // Output full function
                for i in func.start_line - 1..func.end_line {
                    if i < lines.len() {
                        if args.show_line_numbers {
                            output.push_str(&format!("{}│ {}\n", format_line_number(i + 1, use_hex), lines[i]));
                        } else {
                            output.push_str(lines[i]);
                            output.push('\n');
                        }
                    }
                }
                expanded_count += 1;
            } else {
                // Output collapsed function
                let body_lines = func.body.matches('\n').count() + 1;

                if args.show_line_numbers {
                    output.push_str(&format!(
                        "{}│ {} {{ ... }} // [fn:{}] {} lines collapsed\n",
                        format_line_number(func.start_line, use_hex), func.signature, func.name, body_lines
                    ));
                } else {
                    output.push_str(&format!(
                        "{} {{ ... }} // [fn:{}] {} lines collapsed\n",
                        func.signature, func.name, body_lines
                    ));
                }

                // Use hex for line references too if enabled
                let lines_ref = if use_hex {
                    format!("{:X}-{:X}", func.start_line, func.end_line)
                } else {
                    format!("{}-{}", func.start_line, func.end_line)
                };

                function_refs.push(json!({
                    "name": func.name,
                    "ref": format!("[fn:{}]", func.name),
                    "lines": lines_ref,
                    "importance": func.importance
                }));

                collapsed_count += 1;
            }

            current_line = func.end_line;
        }

        // Output remaining lines after last function
        while current_line < lines.len() {
            if args.show_line_numbers {
                output.push_str(&format!("{}│ {}\n", format_line_number(current_line + 1, use_hex), lines[current_line]));
            } else {
                output.push_str(lines[current_line]);
                output.push('\n');
            }
            current_line += 1;
        }

        let metadata = json!({
            "file_path": path.to_string_lossy(),
            "language": language,
            "compression_enabled": true,
            "hex_line_numbers": use_hex,
            "total_lines": lines.len(),
            "functions_found": functions.len(),
            "functions_collapsed": collapsed_count,
            "functions_expanded": expanded_count,
            "collapsed_refs": function_refs,
            "expand_hint": "Use expand_functions: ['fn_name'] or expand_context: ['keyword'] to expand specific functions"
        });

        (output, metadata)
    } else {
        // No compression - output raw content
        let lines: Vec<&str> = content.lines().collect();
        let total_lines = lines.len();
        // Use hex line numbers - defaults to MCP config (true for AI mode!)
        let use_hex = args.hex_line_numbers.unwrap_or(ctx.config.hex_numbers);

        let start_idx = args.offset.saturating_sub(1);
        let end_idx = if args.max_lines > 0 {
            (start_idx + args.max_lines).min(lines.len())
        } else {
            lines.len()
        };

        let mut output = String::new();
        for (i, line) in lines[start_idx..end_idx].iter().enumerate() {
            let line_num = start_idx + i + 1;
            if args.show_line_numbers {
                output.push_str(&format!("{}│ {}\n", format_line_number(line_num, use_hex), line));
            } else {
                output.push_str(line);
                output.push('\n');
            }
        }

        let metadata = json!({
            "file_path": path.to_string_lossy(),
            "language": language,
            "compression_enabled": false,
            "hex_line_numbers": use_hex,
            "total_lines": total_lines,
            "lines_shown": end_idx - start_idx,
            "offset": args.offset,
            "has_more": end_idx < total_lines
        });

        (output, metadata)
    };

    Ok(json!({
        "content": [{
            "type": "text",
            "text": output
        }],
        "metadata": metadata
    }))
}
