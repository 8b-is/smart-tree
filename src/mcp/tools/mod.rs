//! MCP tools implementation for Smart Tree
//!
//! This module is organized into category-based submodules:
//! - definitions: Shared types and argument structs
//! - server: Server info and permissions
//! - directory: Directory analysis tools
//! - search: File search and find tools
//! - statistics: Stats and digest tools
//! - git: Git status and context
//! - compare: Directory comparison
//! - feedback: Feedback and update tools
//! - sse_tools: Server-sent events for directory watching
//! - file_history: File operation tracking
//! - smart_read: AST-aware file reading
//! - wave: Wave memory operations

// Submodules
pub mod compare;
pub mod definitions;
pub mod directory;
pub mod feedback;
pub mod file_history;
pub mod git;
pub mod search;
pub mod server;
pub mod smart_read;
pub mod sse_tools;
pub mod statistics;
pub mod wave;

// Re-exports for public API
pub use definitions::ToolDefinition;

// Re-export handlers that are used externally
pub use compare::{analyze_workspace, compare_directories};
pub use directory::{
    analyze_directory, project_context_dump, project_overview, quick_tree, semantic_analysis,
};
pub use feedback::{check_for_updates, request_tool, submit_feedback};
pub use file_history::{get_file_history, get_project_history_summary, track_file_operation};
pub use git::get_git_status;
pub use search::{
    find_build_files, find_code_files, find_config_files, find_documentation, find_duplicates,
    find_empty_directories, find_files, find_in_timespan, find_large_files, find_projects,
    find_recent_changes, find_tests, search_in_files,
};
pub use server::{server_info, verify_permissions};
pub use smart_read::smart_read;
pub use sse_tools::watch_directory_sse;
pub use statistics::{directory_size_breakdown, get_digest, get_statistics};
pub use wave::handle_wave_memory;

use super::McpContext;
use super::theme_tools;
use anyhow::Result;
use serde_json::{json, Value};
use std::sync::Arc;

/// Handle tools/list MCP request
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
            description: "🔐 Track file operations with hash-based change detection. Part of the ultimate context-driven system that logs all AI file manipulations to `./.st/filehistory/`. Favors append operations as the least intrusive method. Perfect for maintaining a complete history of AI-assisted code changes!".to_string(),
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
            description: "📜 Retrieve complete operation history for a file from the `./.st/filehistory/` tracking system. Shows all AI manipulations with timestamps, operations, hashes, and agents. Essential for understanding how a file evolved through AI assistance!".to_string(),
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
        // Smart edit tools
        ToolDefinition {
            name: "smart_edit".to_string(),
            description: "🚀 Apply multiple smart code edits using minimal tokens! Uses AST understanding to insert functions, replace bodies, add imports, etc. without sending full diffs. Revolutionary token-efficient editing that understands code structure!

📋 OPERATION FIELD REQUIREMENTS:
• InsertFunction: name (required), body (required), class_name, namespace, after, before, visibility
• ReplaceFunction: name (required), new_body (required), class_name
• AddImport: import (required), alias
• InsertClass: name (required), body (required), namespace, extends, implements
• AddMethod: class_name (required), method_name (required), body (required), visibility
• WrapCode: start_line (required), end_line (required), wrapper_type (required), condition
• DeleteElement: element_type (required), name (required), parent
• Rename: old_name (required), new_name (required), scope
• AddDocumentation: target_type (required), target_name (required), documentation (required)
• SmartAppend: section (required), content (required)

💡 EXAMPLES:
• Insert function: {operation:'InsertFunction', name:'validate', body:'fn validate(x: i32) -> bool { x > 0 }', visibility:'public'}
• Add import: {operation:'AddImport', import:'std::collections::HashMap'}
• Insert class: {operation:'InsertClass', name:'Config', body:'struct Config { value: i32 }'}
• Smart append: {operation:'SmartAppend', section:'functions', content:'fn helper() {}'}".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "file_path": {
                        "type": "string",
                        "description": "Path to the file to edit (REQUIRED)"
                    },
                    "edits": {
                        "type": "array",
                        "description": "Array of smart edit operations (REQUIRED). Each operation has different required fields - see operation-specific requirements in tool description.",
                        "items": {
                            "type": "object",
                            "properties": {
                                "operation": {
                                    "type": "string",
                                    "description": "Edit operation type (REQUIRED for all edits)",
                                    "enum": ["InsertFunction", "ReplaceFunction", "AddImport", "InsertClass", "AddMethod", "WrapCode", "DeleteElement", "Rename", "AddDocumentation", "SmartAppend"]
                                },
                                "name": {
                                    "type": "string",
                                    "description": "Name of element (REQUIRED for InsertFunction, InsertClass, ReplaceFunction, DeleteElement)"
                                },
                                "class_name": {
                                    "type": "string",
                                    "description": "Class name (optional for InsertFunction/ReplaceFunction, REQUIRED for AddMethod)"
                                },
                                "method_name": {
                                    "type": "string",
                                    "description": "Method name (REQUIRED for AddMethod)"
                                },
                                "namespace": {
                                    "type": "string",
                                    "description": "Namespace (optional for InsertFunction, InsertClass)"
                                },
                                "body": {
                                    "type": "string",
                                    "description": "Code body (REQUIRED for InsertFunction, InsertClass, AddMethod)"
                                },
                                "new_body": {
                                    "type": "string",
                                    "description": "New body (REQUIRED for ReplaceFunction)"
                                },
                                "import": {
                                    "type": "string",
                                    "description": "Import statement (REQUIRED for AddImport)"
                                },
                                "alias": {
                                    "type": "string",
                                    "description": "Import alias (optional for AddImport)"
                                },
                                "after": {
                                    "type": "string",
                                    "description": "Insert after this element (optional positioning for InsertFunction)"
                                },
                                "before": {
                                    "type": "string",
                                    "description": "Insert before this element (optional positioning for InsertFunction)"
                                },
                                "visibility": {
                                    "type": "string",
                                    "description": "Visibility modifier (optional, defaults to 'private')",
                                    "enum": ["public", "private", "protected"]
                                },
                                "section": {
                                    "type": "string",
                                    "description": "Target section (REQUIRED for SmartAppend)",
                                    "enum": ["imports", "functions", "classes", "main"]
                                },
                                "content": {
                                    "type": "string",
                                    "description": "Content to append (REQUIRED for SmartAppend)"
                                },
                                "element_type": {
                                    "type": "string",
                                    "description": "Element type to delete (REQUIRED for DeleteElement)",
                                    "enum": ["function", "class", "method"]
                                },
                                "parent": {
                                    "type": "string",
                                    "description": "Parent element name (optional for DeleteElement)"
                                },
                                "old_name": {
                                    "type": "string",
                                    "description": "Current name (REQUIRED for Rename)"
                                },
                                "new_name": {
                                    "type": "string",
                                    "description": "New name (REQUIRED for Rename)"
                                },
                                "scope": {
                                    "type": "string",
                                    "description": "Rename scope (optional for Rename)",
                                    "enum": ["global", "class", "function"]
                                },
                                "target_type": {
                                    "type": "string",
                                    "description": "Documentation target type (REQUIRED for AddDocumentation)",
                                    "enum": ["function", "class", "method"]
                                },
                                "target_name": {
                                    "type": "string",
                                    "description": "Target element name (REQUIRED for AddDocumentation)"
                                },
                                "documentation": {
                                    "type": "string",
                                    "description": "Documentation text (REQUIRED for AddDocumentation)"
                                },
                                "start_line": {
                                    "type": "number",
                                    "description": "Start line (REQUIRED for WrapCode)"
                                },
                                "end_line": {
                                    "type": "number",
                                    "description": "End line (REQUIRED for WrapCode)"
                                },
                                "wrapper_type": {
                                    "type": "string",
                                    "description": "Wrapper construct (REQUIRED for WrapCode)",
                                    "enum": ["try", "if", "while", "for"]
                                },
                                "condition": {
                                    "type": "string",
                                    "description": "Wrapper condition (optional for WrapCode)"
                                },
                                "extends": {
                                    "type": "string",
                                    "description": "Base class (optional for InsertClass)"
                                },
                                "implements": {
                                    "type": "array",
                                    "description": "Interfaces to implement (optional for InsertClass)",
                                    "items": {
                                        "type": "string"
                                    }
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
            name: "create_file".to_string(),
            description: "📝 Create a new file with initial content. Automatically creates parent directories if needed. Perfect for starting a new file before using smart_edit operations! Use this BEFORE attempting to edit a non-existent file.".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "file_path": {
                        "type": "string",
                        "description": "Path to the file to create (REQUIRED). File must not already exist."
                    },
                    "content": {
                        "type": "string",
                        "description": "Initial file content (optional, defaults to empty file)"
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
        // Context gathering tools
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
        // UI Customization
        ToolDefinition {
            name: "set_dashboard_theme".to_string(),
            description: "🎨 Set the visual theme for the dashboard. Allows programmatically changing colors. Changes are saved to the user's ~/.st/theme.json file and will persist across sessions. Please refresh the dashboard to see changes.".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "bg_primary": { "type": "string", "description": "Main background color (e.g., '#0d0c1d')" },
                    "bg_secondary": { "type": "string", "description": "Secondary background color (e.g., '#1a1a2e')" },
                    "accent_primary": { "type": "string", "description": "Primary accent color, for highlights (e.g., '#f0f')" },
                    "accent_secondary": { "type": "string", "description": "Secondary accent color, for links (e.g., '#0ff')" },
                    "fg_primary": { "type": "string", "description": "Main text color (e.g., '#e0e0e0')" },
                    "fg_secondary": { "type": "string", "description": "Secondary text color (e.g., '#a0a0e0')" }
                },
                "required": []
            }),
        },
        // Smart Read Tool
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

/// Handle tools/call MCP request - dispatches to appropriate handler
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
        // Server tools
        "verify_permissions" => verify_permissions(args, ctx_clone.clone()).await,
        "server_info" => server_info(args, ctx_clone.clone()).await,

        // Directory analysis tools
        "analyze_directory" => analyze_directory(args, ctx_clone.clone()).await,
        "quick_tree" => quick_tree(args, ctx_clone.clone()).await,
        "project_overview" => project_overview(args, ctx_clone.clone()).await,
        "project_context_dump" => project_context_dump(args, ctx_clone.clone()).await,
        "semantic_analysis" => semantic_analysis(args, ctx_clone.clone()).await,

        // Search tools
        "find_files" => find_files(args, ctx_clone.clone()).await,
        "find_code_files" => find_code_files(args, ctx_clone.clone()).await,
        "find_config_files" => find_config_files(args, ctx_clone.clone()).await,
        "find_projects" => find_projects(args, ctx_clone.clone()).await,
        "find_documentation" => find_documentation(args, ctx_clone.clone()).await,
        "search_in_files" => search_in_files(args, ctx_clone.clone()).await,
        "find_large_files" => find_large_files(args, ctx_clone.clone()).await,
        "find_recent_changes" => find_recent_changes(args, ctx_clone.clone()).await,
        "find_in_timespan" => find_in_timespan(args, ctx_clone.clone()).await,
        "find_duplicates" => find_duplicates(args, ctx_clone.clone()).await,
        "find_tests" => find_tests(args, ctx_clone.clone()).await,
        "find_build_files" => find_build_files(args, ctx_clone.clone()).await,
        "find_empty_directories" => find_empty_directories(args, ctx_clone.clone()).await,

        // Statistics tools
        "get_statistics" => get_statistics(args, ctx_clone.clone()).await,
        "get_digest" => get_digest(args, ctx_clone.clone()).await,
        "directory_size_breakdown" => directory_size_breakdown(args, ctx_clone.clone()).await,

        // Git tools
        "get_git_status" => get_git_status(args, ctx_clone.clone()).await,

        // Compare tools
        "compare_directories" => compare_directories(args, ctx_clone.clone()).await,
        "analyze_workspace" => analyze_workspace(args, ctx_clone.clone()).await,

        // Feedback tools
        "submit_feedback" => submit_feedback(args, ctx_clone.clone()).await,
        "request_tool" => request_tool(args, ctx_clone.clone()).await,
        "check_for_updates" => check_for_updates(args, ctx_clone.clone()).await,

        // SSE tools
        "watch_directory_sse" => watch_directory_sse(args, ctx_clone.clone()).await,

        // File history tools
        "track_file_operation" => track_file_operation(args, ctx_clone.clone()).await,
        "get_file_history" => get_file_history(args, ctx_clone.clone()).await,
        "get_project_history_summary" => get_project_history_summary(args, ctx_clone.clone()).await,

        // Wave memory
        "wave_memory" => handle_wave_memory(args).await,

        // Smart read
        "read" => smart_read(args, ctx_clone.clone()).await,

        // Smart edit tools (delegated to smart_edit module)
        "smart_edit" => crate::mcp::smart_edit::handle_smart_edit(Some(args)).await,
        "get_function_tree" => crate::mcp::smart_edit::handle_get_function_tree(Some(args)).await,
        "insert_function" => crate::mcp::smart_edit::handle_insert_function(Some(args)).await,
        "remove_function" => crate::mcp::smart_edit::handle_remove_function(Some(args)).await,
        "create_file" => crate::mcp::smart_edit::handle_create_file(Some(args)).await,

        // Context gathering tools (delegated to context_tools module)
        "gather_project_context" => {
            let req: crate::mcp::context_tools::GatherProjectContextRequest =
                serde_json::from_value(args)?;
            let permission_check = |_perm_req| Ok(true);
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
                             Results will be saved to the project's ./.st/mem8/ directory, organized by source.\n\
                             Check the terminal for interactive prompts!"
                }]
            }))
        }

        // Theme tools
        "set_dashboard_theme" => theme_tools::handle_set_dashboard_theme(args).await,

        _ => Err(anyhow::anyhow!("Unknown tool: {}", tool_name)),
    }?;

    // Enhance the response with helpful recommendations
    let enhanced_result = ctx.assistant.enhance_response(tool_name, result).await;

    Ok(enhanced_result)
}
