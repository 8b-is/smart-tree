// -----------------------------------------------------------------------------
// CLI Definitions for Smart Tree
// All command-line argument parsing happens here using clap.
// Extracted from main.rs to keep things organized!
// -----------------------------------------------------------------------------

use anyhow::{Context, Result};
use chrono::NaiveDate;
use clap::{Parser, Subcommand, ValueEnum};
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

    /// Launch web dashboard (browser-based terminal + file browser)
    #[arg(long, exclusive = true, help_heading = "Interactive Modes")]
    pub dashboard: bool,

    /// Open browser automatically when starting dashboard
    #[arg(long, requires = "dashboard", help_heading = "Interactive Modes")]
    pub open_browser: bool,

    /// Start HTTP daemon (MCP over HTTP, LLM proxy, The Custodian)
    #[arg(long, exclusive = true, help_heading = "Interactive Modes")]
    pub http_daemon: bool,

    // =========================================================================
    // MCP SERVER (via Daemon)
    // =========================================================================
    /// Run as MCP server for AI assistants (auto-starts daemon)
    #[arg(long, exclusive = true, help_heading = "MCP Server")]
    pub mcp: bool,

    // =========================================================================
    // DAEMON CONTROL
    // =========================================================================
    /// Set the log level
    #[arg(long, value_enum, help_heading = "Daemon Control")]
    pub log_level: Option<LogLevel>,

    // =========================================================================
    // LOGGING & TRANSPARENCY
    // =========================================================================
    /// Enable activity logging to JSONL file
    #[arg(long, value_name = "PATH", help_heading = "Logging & Transparency")]
    pub log: Option<Option<String>>,

    /// Control smart tips (on/off)
    #[arg(long, value_name = "STATE", value_parser = ["on", "off"], help_heading = "Logging & Transparency")]
    pub tips: Option<String>,

    // =========================================================================
    // TOP-LEVEL COMMANDS
    // =========================================================================
    #[command(subcommand)]
    pub cmd: Option<Cmd>,

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
    /// Find files matching regex pattern (e.g., --find "README\.md")
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
    // SMART SCANNING - Intelligent context-aware output
    // =========================================================================
    /// Enable smart mode - surface what matters, not everything
    /// Groups output by interest: security, changes, important, background
    #[arg(long, help_heading = "Smart Scanning")]
    pub smart: bool,

    /// Only show changes since last scan
    #[arg(long, help_heading = "Smart Scanning")]
    pub changes_only: bool,

    /// Minimum interest level (0.0-1.0) to display
    #[arg(long, default_value = "0.0", help_heading = "Smart Scanning")]
    pub min_interest: f32,

    /// Disable security scanning during traversal
    #[arg(long, help_heading = "Smart Scanning")]
    pub no_security: bool,

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

#[derive(Debug, Subcommand)]
pub enum Cmd {
    /// Manage the smart-tree daemon (Linux: systemd, macOS: launchctl, Windows: Task Scheduler)
    #[command(subcommand)]
    Service(Service),

    /// Manage project tags
    #[command(subcommand, name = "project-tags")]
    ProjectTags(ProjectTags),
}

#[derive(Debug, Subcommand)]
pub enum Service {
    /// Install the smart-tree daemon as a system service
    Install,
    /// Uninstall the service
    Uninstall,
    /// Start the service for the current project
    Start,
    /// Stop the service
    Stop,
    /// Show service status
    Status,
    /// Show service logs
    Logs,
}

#[derive(Debug, Subcommand)]
pub enum ProjectTags {
    /// Add a tag to the project
    Add {
        /// The tag to add
        #[arg(required = true)]
        tag: String,
    },
    /// Remove a tag from the project
    Remove {
        /// The tag to remove
        #[arg(required = true)]
        tag: String,
    },
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
    /// HexTree - readable quantum compression with tree structure
    HexTree,
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
        OutputMode::Quantum | OutputMode::QuantumSemantic | OutputMode::HexTree => 5,
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

#[derive(Debug, Clone, Copy, ValueEnum)]
pub enum LogLevel {
    Error,
    Warn,
    Info,
    Debug,
    Trace,
}
