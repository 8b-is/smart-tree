// -----------------------------------------------------------------------------
// CLI Definitions for Smart Tree
// All command-line argument parsing happens here using clap.
// Extracted from main.rs to keep things organized!
// -----------------------------------------------------------------------------

use anyhow::{Context, Result};
use chrono::NaiveDate;
use clap::{Parser, ValueEnum};
use std::path::PathBuf;
use std::time::SystemTime;

/// Smart Tree CLI - intelligent directory visualization
#[derive(Parser, Debug)]
#[command(
    name = "st",
    about = "Smart Tree - An intelligent directory visualization tool. Not just a tree, it's a smart-tree!",
    author
)]
pub struct Cli {
    // =========================================================================
    // GETTING STARTED
    // =========================================================================
    /// Show the cheatsheet - quick reference for all commands
    #[arg(long, exclusive = true, help_heading = "Getting Started")]
    pub cheet: bool,

    /// Show version information and check for updates
    #[arg(short = 'V', long, exclusive = true, help_heading = "Getting Started")]
    pub version: bool,

    /// Generate shell completion scripts (bash, zsh, fish, powershell)
    #[arg(
        long,
        exclusive = true,
        value_name = "SHELL",
        help_heading = "Getting Started"
    )]
    pub completions: Option<clap_complete::Shell>,

    /// Generate the man page
    #[arg(long, exclusive = true, help_heading = "Getting Started")]
    pub man: bool,

    /// Check for updates and install the latest version
    #[arg(long, exclusive = true, help_heading = "Getting Started")]
    pub update: bool,

    /// Skip the automatic update check on startup
    #[arg(long, help_heading = "Getting Started")]
    pub no_update_check: bool,

    // =========================================================================
    // INTERACTIVE MODES
    // =========================================================================
    /// Launch Spicy TUI - interactive file browser with fuzzy search!
    #[arg(long, help_heading = "Interactive Modes")]
    pub spicy: bool,

    /// Launch Smart Tree Terminal Interface (STTI)
    #[arg(long, exclusive = true, help_heading = "Interactive Modes")]
    pub terminal: bool,

    /// Launch egui Dashboard - real-time visualization
    #[arg(long, exclusive = true, help_heading = "Interactive Modes")]
    pub dashboard: bool,

    // =========================================================================
    // MCP SERVER (Model Context Protocol)
    // =========================================================================
    /// Run as MCP server for AI assistants (Claude Desktop, etc.)
    #[arg(long, exclusive = true, help_heading = "MCP Server")]
    pub mcp: bool,

    /// List all 30+ MCP tools available
    #[arg(long, exclusive = true, help_heading = "MCP Server")]
    pub mcp_tools: bool,

    /// Show MCP config snippet (copy to claude_desktop_config.json)
    #[arg(long, exclusive = true, help_heading = "MCP Server")]
    pub mcp_config: bool,

    /// Auto-install MCP server to Claude Desktop (one command setup!)
    #[arg(long, exclusive = true, help_heading = "MCP Server")]
    pub mcp_install: bool,

    /// Remove MCP server from Claude Desktop
    #[arg(long, exclusive = true, help_heading = "MCP Server")]
    pub mcp_uninstall: bool,

    /// Check MCP installation status
    #[arg(long, exclusive = true, help_heading = "MCP Server")]
    pub mcp_status: bool,

    // =========================================================================
    // DAEMON - System-wide AI context service with Foken credits
    // =========================================================================
    /// Run as system daemon - always-on AI context service with Foken credit tracking
    #[arg(long, exclusive = true, help_heading = "Daemon")]
    pub daemon: bool,

    /// Port for daemon mode
    #[arg(long, default_value = "8420", help_heading = "Daemon")]
    pub daemon_port: u16,

    /// Start daemon in background
    #[arg(long, exclusive = true, help_heading = "Daemon")]
    pub daemon_start: bool,

    /// Stop running daemon
    #[arg(long, exclusive = true, help_heading = "Daemon")]
    pub daemon_stop: bool,

    /// Show daemon status
    #[arg(long, exclusive = true, help_heading = "Daemon")]
    pub daemon_status: bool,

    /// Query system context from daemon
    #[arg(long, exclusive = true, help_heading = "Daemon")]
    pub daemon_context: bool,

    /// List projects detected by daemon
    #[arg(long, exclusive = true, help_heading = "Daemon")]
    pub daemon_projects: bool,

    /// Show Foken credits
    #[arg(long, exclusive = true, help_heading = "Daemon")]
    pub daemon_credits: bool,

    /// Bypass daemon and run standalone (don't route through daemon even if running)
    #[arg(long, help_heading = "Daemon")]
    pub no_daemon: bool,

    /// Auto-start daemon if not running (default: just use if available)
    #[arg(long, help_heading = "Daemon")]
    pub auto_daemon: bool,

    // =========================================================================
    // CLAUDE CONSCIOUSNESS - Session state persistence
    // =========================================================================
    /// Save session state to .claude_consciousness.m8
    #[arg(long, exclusive = true, help_heading = "Claude Consciousness")]
    pub claude_save: bool,

    /// Restore previous session from .claude_consciousness.m8
    #[arg(long, exclusive = true, help_heading = "Claude Consciousness")]
    pub claude_restore: bool,

    /// Show consciousness status and summary
    #[arg(long, exclusive = true, help_heading = "Claude Consciousness")]
    pub claude_context: bool,

    /// Show ultra-compressed kickstart format
    #[arg(long, help_heading = "Claude Consciousness")]
    pub claude_kickstart: bool,

    /// Dump raw consciousness file (debugging)
    #[arg(long, help_heading = "Claude Consciousness")]
    pub claude_dump: bool,

    /// Set up Claude integration for this project (.claude/ directory)
    #[arg(long, exclusive = true, help_heading = "Claude Consciousness")]
    pub setup_claude: bool,

    /// Update .m8 consciousness files for directory
    #[arg(long, help_heading = "Claude Consciousness")]
    pub update_consciousness: bool,

    /// Hook for user prompt submission (internal use)
    #[arg(long, hide = true)]
    pub claude_user_prompt_submit: bool,

    // =========================================================================
    // MEMORY & SESSIONS - Persistent knowledge
    // =========================================================================
    /// Anchor a memory: --memory-anchor <TYPE> <KEYWORDS> <CONTEXT>
    #[arg(long, num_args = 3, value_names = &["TYPE", "KEYWORDS", "CONTEXT"], help_heading = "Memory & Sessions")]
    pub memory_anchor: Option<Vec<String>>,

    /// Find memories by keywords
    #[arg(long, help_heading = "Memory & Sessions")]
    pub memory_find: Option<String>,

    /// Show memory bank statistics
    #[arg(long, help_heading = "Memory & Sessions")]
    pub memory_stats: bool,

    /// Start or resume a mega session
    #[arg(long, help_heading = "Memory & Sessions")]
    pub mega_start: Option<Option<String>>,

    /// Save current mega session snapshot
    #[arg(long, help_heading = "Memory & Sessions")]
    pub mega_save: bool,

    /// Record a breakthrough in mega session
    #[arg(long, value_name = "DESCRIPTION", help_heading = "Memory & Sessions")]
    pub mega_breakthrough: Option<String>,

    /// Show mega session statistics
    #[arg(long, help_heading = "Memory & Sessions")]
    pub mega_stats: bool,

    /// List all saved mega sessions
    #[arg(long, help_heading = "Memory & Sessions")]
    pub mega_list: bool,

    // =========================================================================
    // SECURITY & ANALYSIS
    // =========================================================================
    /// Run security scan for malware patterns
    #[arg(long, help_heading = "Security & Analysis")]
    pub security_scan: bool,

    /// Show tokenization statistics
    #[arg(long, help_heading = "Security & Analysis")]
    pub token_stats: bool,

    /// Get wave frequency from .m8 file
    #[arg(long, help_heading = "Security & Analysis")]
    pub get_frequency: bool,

    // =========================================================================
    // AI INTEGRATION - Unified setup for all AI platforms
    // =========================================================================
    /// Interactive AI integration setup - configures MCP, hooks, plugins for your AI
    /// If no other flags, launches interactive mode. Use with --install-scope and --ai-target.
    #[arg(short = 'i', long = "install-ai", help_heading = "AI Integration")]
    pub install_ai: bool,

    /// Installation scope: project (local .claude/) or user (~/.claude/, ~/.config/)
    #[arg(
        long = "install-scope",
        value_enum,
        default_value = "project",
        help_heading = "AI Integration"
    )]
    pub install_scope: InstallScope,

    /// Target AI platform for configuration
    #[arg(
        long,
        value_enum,
        default_value = "claude",
        help_heading = "AI Integration"
    )]
    pub ai_target: AiTarget,

    /// Skip interactive prompts (use defaults or provided flags)
    #[arg(long, help_heading = "AI Integration")]
    pub non_interactive: bool,

    // =========================================================================
    // CLAUDE CODE INTEGRATION (Legacy - prefer --install-ai)
    // =========================================================================
    /// Configure Claude Code hooks (enable/disable/status)
    #[arg(
        long,
        value_name = "ACTION",
        help_heading = "Claude Code Integration (Legacy)"
    )]
    pub hooks_config: Option<String>,

    /// Quick setup: Install Smart Tree hooks in Claude Code
    #[arg(long, help_heading = "Claude Code Integration (Legacy)")]
    pub hooks_install: bool,

    // =========================================================================
    // LLM PROXY - Unified AI interface
    // =========================================================================
    /// Call an LLM provider via the unified proxy
    #[arg(long, help_heading = "LLM Proxy")]
    pub proxy: bool,

    /// LLM provider to use (openai, anthropic, google, candle)
    #[arg(long, value_name = "PROVIDER", help_heading = "LLM Proxy")]
    pub provider: Option<String>,

    /// LLM model to use
    #[arg(long, value_name = "MODEL", help_heading = "LLM Proxy")]
    pub model: Option<String>,

    /// Prompt for the LLM (if not provided, reads from stdin)
    #[arg(long, value_name = "PROMPT", help_heading = "LLM Proxy")]
    pub prompt: Option<String>,

    /// Memory scope for the conversation (e.g., "project-x")
    #[arg(long, value_name = "SCOPE", help_heading = "LLM Proxy")]
    pub scope: Option<String>,

    /// Start the OpenAI-compatible proxy server
    #[arg(long, help_heading = "LLM Proxy")]
    pub proxy_server: bool,

    /// Port for the proxy server
    #[arg(long, default_value = "8448", help_heading = "LLM Proxy")]
    pub proxy_port: u16,

    // =========================================================================
    // LOGGING & TRANSPARENCY
    // =========================================================================
    /// Enable activity logging to JSONL file
    #[arg(long, value_name = "PATH", help_heading = "Logging & Transparency")]
    pub log: Option<Option<String>>,

    // =========================================================================
    // PROJECT MANAGEMENT
    // =========================================================================
    /// Rename project: --rename-project "OldName" "NewName"
    #[arg(long, exclusive = true, value_names = &["OLD", "NEW"], num_args = 2, help_heading = "Project Management")]
    pub rename_project: Option<Vec<String>>,

    /// Manage project tags
    #[clap(subcommand, name = "project-tags")]
    pub project_tags: Option<ProjectTags>,

    /// Control smart tips (on/off)
    #[arg(long, value_name = "STATE", value_parser = ["on", "off"], help_heading = "Project Management")]
    pub tips: Option<String>,

    // =========================================================================
    // SCAN OPTIONS
    // =========================================================================
    /// Path to analyze (directory, file, URL, or stream)
    pub path: Option<String>,

    /// Specify input type explicitly (filesystem, qcp, sse, openapi, mem8)
    #[arg(long, value_name = "TYPE")]
    pub input: Option<String>,

    #[command(flatten)]
    pub scan_opts: ScanArgs,
}

#[derive(Parser, Debug)]
pub struct ScanArgs {
    // =========================================================================
    // OUTPUT FORMAT
    // =========================================================================
    /// Output format (classic, ai, quantum, json, etc.)
    #[arg(
        short,
        long,
        value_enum,
        default_value = "auto",
        help_heading = "Output Format"
    )]
    pub mode: OutputMode,

    // =========================================================================
    // FILTERING - What to include/exclude
    // =========================================================================
    /// Find files matching regex pattern (e.g., --find "README\\.md")
    #[arg(long, help_heading = "Filtering")]
    pub find: Option<String>,

    /// Filter by file extension (e.g., --type rs)
    #[arg(long = "type", help_heading = "Filtering")]
    pub filter_type: Option<String>,

    /// Filter by entry type: f (files) or d (directories)
    #[arg(long = "entry-type", value_parser = ["f", "d"], help_heading = "Filtering")]
    pub entry_type: Option<String>,

    /// Only files larger than size (e.g., --min-size 1M)
    #[arg(long, help_heading = "Filtering")]
    pub min_size: Option<String>,

    /// Only files smaller than size (e.g., --max-size 100K)
    #[arg(long, help_heading = "Filtering")]
    pub max_size: Option<String>,

    /// Files newer than date (YYYY-MM-DD)
    #[arg(long, help_heading = "Filtering")]
    pub newer_than: Option<String>,

    /// Files older than date (YYYY-MM-DD)
    #[arg(long, help_heading = "Filtering")]
    pub older_than: Option<String>,

    // =========================================================================
    // TRAVERSAL - How to scan
    // =========================================================================
    /// Traversal depth (0 = auto, 1 = shallow, 10 = deep)
    #[arg(short, long, default_value = "0", help_heading = "Traversal")]
    pub depth: usize,

    /// Ignore .gitignore files
    #[arg(long, help_heading = "Traversal")]
    pub no_ignore: bool,

    /// Ignore default patterns (node_modules, __pycache__, etc.)
    #[arg(long, help_heading = "Traversal")]
    pub no_default_ignore: bool,

    /// Show hidden files (starting with .)
    #[arg(long, short = 'a', help_heading = "Traversal")]
    pub all: bool,

    /// Show ignored directories in brackets
    #[arg(long, help_heading = "Traversal")]
    pub show_ignored: bool,

    /// Show EVERYTHING (--all + --no-ignore + --no-default-ignore)
    #[arg(long, help_heading = "Traversal")]
    pub everything: bool,

    // =========================================================================
    // DISPLAY - How output looks
    // =========================================================================
    /// Show filesystem type indicators (X=XFS, 4=ext4, B=Btrfs)
    #[arg(long, help_heading = "Display")]
    pub show_filesystems: bool,

    /// Disable emojis (Trish will miss them!)
    #[arg(long, help_heading = "Display")]
    pub no_emoji: bool,

    /// Compress output with zlib (base64 encoded)
    #[arg(short = 'z', long, help_heading = "Display")]
    pub compress: bool,

    /// Optimize for MCP/API (compression + no colors/emoji)
    #[arg(long, help_heading = "Display")]
    pub mcp_optimize: bool,

    /// Compact JSON (single line)
    #[arg(long, help_heading = "Display")]
    pub compact: bool,

    /// Path display: off, relative, or full
    #[arg(
        long = "path-mode",
        value_enum,
        default_value = "off",
        help_heading = "Display"
    )]
    pub path_mode: PathMode,

    /// Color output: always, never, or auto
    #[arg(long, value_enum, default_value = "auto", help_heading = "Display")]
    pub color: ColorMode,

    /// Wrap AI output in JSON structure
    #[arg(long, help_heading = "Display")]
    pub ai_json: bool,

    // =========================================================================
    // STREAMING - Real-time output
    // =========================================================================
    /// Stream output as files are scanned
    #[arg(long, help_heading = "Streaming")]
    pub stream: bool,

    /// Start SSE server for real-time monitoring
    #[arg(long, help_heading = "Streaming")]
    pub sse_server: bool,

    /// SSE server port
    #[arg(long, default_value = "8420", help_heading = "Streaming")]
    pub sse_port: u16,

    // =========================================================================
    // SEARCH & ANALYSIS
    // =========================================================================
    /// Search file contents (e.g., --search "TODO")
    #[arg(long, help_heading = "Search & Analysis")]
    pub search: Option<String>,

    /// Group by semantic similarity
    #[arg(long, help_heading = "Search & Analysis")]
    pub semantic: bool,

    /// Focus analysis on specific file (relations mode)
    #[arg(long, value_name = "FILE", help_heading = "Search & Analysis")]
    pub focus: Option<PathBuf>,

    /// Filter relationships: imports, calls, types, tests, coupled
    #[arg(long, value_name = "TYPE", help_heading = "Search & Analysis")]
    pub relations_filter: Option<String>,

    // =========================================================================
    // SORTING
    // =========================================================================
    /// Sort by: a-to-z, z-to-a, largest, smallest, newest, oldest, type
    #[arg(long, value_enum, help_heading = "Sorting")]
    pub sort: Option<SortField>,

    /// Show only top N results (use with --sort)
    #[arg(long, value_name = "N", help_heading = "Sorting")]
    pub top: Option<usize>,

    // =========================================================================
    // MERMAID & MARKDOWN OPTIONS
    // =========================================================================
    /// Mermaid style: flowchart, mindmap, gitgraph, treemap
    #[arg(
        long,
        value_enum,
        default_value = "flowchart",
        help_heading = "Mermaid & Markdown"
    )]
    pub mermaid_style: MermaidStyleArg,

    /// Exclude mermaid diagrams from markdown
    #[arg(long, help_heading = "Mermaid & Markdown")]
    pub no_markdown_mermaid: bool,

    /// Exclude tables from markdown
    #[arg(long, help_heading = "Mermaid & Markdown")]
    pub no_markdown_tables: bool,

    /// Exclude pie charts from markdown
    #[arg(long, help_heading = "Mermaid & Markdown")]
    pub no_markdown_pie_charts: bool,

    // =========================================================================
    // ADVANCED
    // =========================================================================
    /// Index code to SmartPastCode registry
    #[arg(long, value_name = "URL", help_heading = "Advanced")]
    pub index_registry: Option<String>,

    /// Show private functions in docs (function-markdown mode)
    #[arg(long, help_heading = "Advanced")]
    pub show_private: bool,

    /// View Smart Edit diffs from .st folder
    #[arg(long, help_heading = "Advanced")]
    pub view_diffs: bool,

    /// Clean up old diffs, keep last N per file
    #[arg(long, value_name = "N", help_heading = "Advanced")]
    pub cleanup_diffs: Option<usize>,
}

/// Sort field options with intuitive names
#[derive(Debug, Clone, Copy, ValueEnum)]
pub enum SortField {
    /// Sort alphabetically A to Z
    #[value(name = "a-to-z")]
    AToZ,
    /// Sort alphabetically Z to A
    #[value(name = "z-to-a")]
    ZToA,
    /// Sort by size, largest files first
    #[value(name = "largest")]
    Largest,
    /// Sort by size, smallest files first
    #[value(name = "smallest")]
    Smallest,
    /// Sort by modification date, newest first
    #[value(name = "newest")]
    Newest,
    /// Sort by modification date, oldest first
    #[value(name = "oldest")]
    Oldest,
    /// Sort by file type/extension
    #[value(name = "type")]
    Type,
    /// Legacy aliases for backward compatibility
    #[value(name = "name", alias = "alpha")]
    Name,
    #[value(name = "size")]
    Size,
    #[value(name = "date", alias = "modified")]
    Date,
}

/// Enum for mermaid style argument
#[derive(Debug, Clone, Copy, ValueEnum)]
pub enum MermaidStyleArg {
    /// Traditional flowchart (default)
    Flowchart,
    /// Mind map style
    Mindmap,
    /// Git graph style
    Gitgraph,
    /// Treemap style (shows file sizes visually)
    Treemap,
}

/// Color mode for output
#[derive(Debug, Clone, Copy, ValueEnum)]
pub enum ColorMode {
    /// Always use colors
    Always,
    /// Never use colors
    Never,
    /// Auto-detect (colors if terminal)
    Auto,
}

/// Installation scope for AI integration
#[derive(Debug, Clone, Copy, ValueEnum, Default, PartialEq)]
pub enum InstallScope {
    /// Project-local installation (.claude/ in current directory)
    #[default]
    Project,
    /// User-wide installation (~/.claude/ or ~/.config/)
    User,
}

/// Target AI platform for configuration
#[derive(Debug, Clone, Copy, ValueEnum, Default, PartialEq)]
pub enum AiTarget {
    /// Claude (Anthropic) - default, most features
    #[default]
    Claude,
    /// ChatGPT (OpenAI)
    Chatgpt,
    /// Gemini (Google)
    Gemini,
    /// Universal - generic config for any AI
    Universal,
}

/// Path display mode
#[derive(Debug, Clone, Copy, ValueEnum)]
pub enum PathMode {
    /// Show only filenames (default)
    Off,
    /// Show paths relative to scan root
    Relative,
    /// Show full absolute paths
    Full,
}

/// Output format mode
#[derive(Debug, Clone, Copy, ValueEnum, PartialEq)]
pub enum OutputMode {
    /// Auto mode - smart default selection based on context
    Auto,
    /// Classic tree format with metadata and emojis
    Classic,
    /// Hexadecimal format with fixed-width fields
    Hex,
    /// JSON output for programmatic use
    Json,
    /// Unix ls -Alh format
    Ls,
    /// AI-optimized format for LLMs
    Ai,
    /// Directory statistics only
    Stats,
    /// CSV format
    Csv,
    /// TSV format
    Tsv,
    /// Super compact digest format
    Digest,
    /// Emotional tree - files with feelings!
    Emotional,
    /// MEM|8 Quantum format - ultimate compression
    Quantum,
    /// Semantic grouping format
    Semantic,
    /// Projects discovery mode
    Projects,
    /// Mermaid diagram format
    Mermaid,
    /// Markdown report format
    Markdown,
    /// Interactive summary mode
    Summary,
    /// AI-optimized summary mode
    SummaryAi,
    /// Context mode for AI conversations
    Context,
    /// Code relationship analysis
    Relations,
    /// Quantum compression with semantic understanding
    QuantumSemantic,
    /// Waste detection and optimization analysis
    Waste,
    /// Marqant - Quantum-compressed markdown format
    Marqant,
    /// SSE - Server-Sent Events streaming format
    Sse,
    /// Function documentation in markdown format
    FunctionMarkdown,
}

#[derive(Debug, Parser)]
pub enum ProjectTags {
    /// Add a tag to the project
    Add {
        /// The tag to add
        #[clap(required = true)]
        tag: String,
    },
    /// Remove a tag from the project
    Remove {
        /// The tag to remove
        #[clap(required = true)]
        tag: String,
    },
}

/// Get the ideal depth for each output mode
pub fn get_ideal_depth_for_mode(mode: &OutputMode) -> usize {
    match mode {
        OutputMode::Auto => 3,
        OutputMode::Ls => 1,
        OutputMode::Classic => 3,
        OutputMode::Ai | OutputMode::Hex => 5,
        OutputMode::Stats => 10,
        OutputMode::Digest => 10,
        OutputMode::Emotional => 5,
        OutputMode::Quantum | OutputMode::QuantumSemantic => 5,
        OutputMode::Summary | OutputMode::SummaryAi | OutputMode::Context => 4,
        OutputMode::Waste => 10,
        OutputMode::Relations => 10,
        OutputMode::Projects => 5,
        _ => 4,
    }
}

/// Parse a date string (YYYY-MM-DD) into SystemTime
pub fn parse_date(date_str: &str) -> Result<SystemTime> {
    let date = NaiveDate::parse_from_str(date_str, "%Y-%m-%d")?;
    let datetime = date.and_hms_opt(0, 0, 0).context("Invalid time")?;
    Ok(SystemTime::from(
        datetime
            .and_local_timezone(chrono::Local)
            .single()
            .context("Invalid timezone")?,
    ))
}
