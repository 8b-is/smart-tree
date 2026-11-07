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

    /// Internal fast-path dynamic completion provider for shells (folders, memories, memnet-ask, sync-users, symbols)
    #[arg(
        long,
        num_args = 1..=2,
        value_names = ["KIND", "PREFIX"],
        hide = true
    )]
    pub complete_dynamic: Option<Vec<String>>,

    /// Generate the man page
    #[arg(long, exclusive = true, help_heading = "Getting Started")]
    pub man: bool,

    /// Check for updates and install the latest version
    #[arg(long, exclusive = true, help_heading = "Getting Started")]
    pub update: bool,

    /// Install a stable release and restart the managed daemon (root, opt-in service only)
    #[arg(long, exclusive = true, help_heading = "Daemon Control")]
    pub auto_update: bool,

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

    /// Launch the PTY web dashboard (browser terminal + file browser).
    /// Separate port so it never collides with the HTTP daemon (28428).
    #[arg(
        long,
        exclusive = true,
        alias = "pty-dashboard",
        help_heading = "Interactive Modes"
    )]
    pub dashboard: bool,

    /// PTY dashboard port (default 28429, distinct from daemon 28428)
    #[arg(
        long,
        value_name = "PORT",
        default_value = "28429",
        requires = "dashboard",
        help_heading = "Interactive Modes"
    )]
    pub dashboard_port: u16,

    /// Open browser automatically (works with --dashboard and with --up)
    #[arg(long, help_heading = "Interactive Modes")]
    pub open_browser: bool,

    /// Network CIDR allow-list for the PTY dashboard (e.g., 192.168.1.0/24)
    #[arg(
        long,
        value_name = "CIDR",
        requires = "dashboard",
        help_heading = "Interactive Modes"
    )]
    pub allow: Vec<String>,

    /// Canonical local HTTP server (foreground): MCP HTTP (/mcp + /mcp/sse),
    /// dashboard (/), LLM proxy, The Custodian — all on 127.0.0.1:28428.
    /// Localhost needs no token (same trust boundary as stdio MCP).
    /// `st up` is the background equivalent; `st service` is the system one.
    #[arg(
        long,
        alias = "serve",
        alias = "daemon",
        help_heading = "Interactive Modes"
    )]
    pub http_daemon: bool,

    // =========================================================================
    // MCP SERVER — two transports, one brain. Both are local + token-free.
    // =========================================================================
    /// Run the stdio MCP server (foreground, one client, no daemon/token needed)
    #[arg(
        long,
        exclusive = true,
        alias = "mcp-stdio",
        help_heading = "MCP Server"
    )]
    pub mcp: bool,

    /// Print ready-to-paste MCP-HTTP client config (Streamable HTTP + SSE,
    /// no token required on localhost) and exit
    #[arg(
        long,
        alias = "mcp-config",
        alias = "mcp-http-config",
        alias = "mcp-http",
        exclusive = true,
        help_heading = "MCP Server"
    )]
    pub mcp_print_config: bool,

    /// Install Smart Tree as MCP server in AI agents (stdio + HTTP entries)
    #[arg(long, exclusive = true, help_heading = "MCP Server")]
    pub mcp_install: bool,

    /// Uninstall Smart Tree MCP server from AI agents
    #[arg(long, exclusive = true, help_heading = "MCP Server")]
    pub mcp_uninstall: bool,

    /// Check MCP installation status across AI agents
    #[arg(long, exclusive = true, help_heading = "MCP Server")]
    pub mcp_status: bool,

    /// Set the log level
    #[arg(long, value_enum, help_heading = "Daemon Control")]
    pub log_level: Option<LogLevel>,

    // =========================================================================
    // DAEMON CONTROL — canonical server: `st up` / `st --serve` / `st service`
    // =========================================================================
    /// Start the canonical daemon in the background, update .mcp.json, and
    /// print dashboard/MCP URLs. The one command that makes everything work.
    #[arg(long, exclusive = true, help_heading = "Daemon Control")]
    pub up: bool,

    /// Print the daemon auth token + path (only needed when externally bound)
    #[arg(long, exclusive = true, help_heading = "Daemon Control")]
    pub show_token: bool,

    /// Start the Smart Tree daemon in the background (alias of `st up`)
    #[arg(long, exclusive = true, help_heading = "Daemon Control")]
    pub daemon_start: bool,

    /// Stop the Smart Tree daemon
    #[arg(long, exclusive = true, help_heading = "Daemon Control")]
    pub daemon_stop: bool,

    /// Show Smart Tree daemon status
    #[arg(long, exclusive = true, help_heading = "Daemon Control")]
    pub daemon_status: bool,

    /// Get context from the daemon
    #[arg(long, exclusive = true, help_heading = "Daemon Control")]
    pub daemon_context: bool,

    /// Recall indexed files and remembered context through the daemon (JSON)
    #[arg(long, conflicts_with = "no_daemon", help_heading = "Daemon Control")]
    pub recall: Option<String>,

    /// List projects tracked by the daemon
    #[arg(long, exclusive = true, help_heading = "Daemon Control")]
    pub daemon_projects: bool,

    /// Show Foken credits from daemon
    #[arg(long, exclusive = true, help_heading = "Daemon Control")]
    pub daemon_credits: bool,

    /// [DEPRECATED: use `st service install`] Install daemon as a system service
    #[arg(long, exclusive = true, help_heading = "Daemon Control", hide = true)]
    pub daemon_install: bool,

    /// Bypass daemon and scan locally (standalone mode)
    #[arg(long, help_heading = "Daemon Control")]
    pub no_daemon: bool,

    // =========================================================================
    // CONSCIOUSNESS & MEMORY
    // =========================================================================
    /// Save agent consciousness state to .aye_consciousness.m8
    #[arg(
        long,
        alias = "claude-save",
        exclusive = true,
        help_heading = "Consciousness & Memory"
    )]
    pub agent_save: bool,

    /// Restore agent consciousness from .aye_consciousness.m8
    #[arg(
        long,
        alias = "claude-restore",
        exclusive = true,
        help_heading = "Consciousness & Memory"
    )]
    pub agent_restore: bool,

    /// Show agent consciousness status and summary
    #[arg(
        long,
        alias = "claude-context",
        exclusive = true,
        help_heading = "Consciousness & Memory"
    )]
    pub agent_context: bool,

    /// Ultra-compressed consciousness restoration format
    #[arg(long, exclusive = true, help_heading = "Consciousness & Memory")]
    pub agent_kickstart: bool,

    /// Dump raw consciousness file content for debugging
    #[arg(long, exclusive = true, help_heading = "Consciousness & Memory")]
    pub agent_dump: bool,

    /// Anchor a memory: --memory-anchor <TYPE> <KEYWORDS> <CONTEXT>
    /// Types: insight, decision, pattern, gotcha, todo
    #[arg(long, num_args = 3, value_names = ["TYPE", "KEYWORDS", "CONTEXT"], help_heading = "Consciousness & Memory")]
    pub memory_anchor: Option<Vec<String>>,

    /// Find memories by keywords (comma-separated)
    #[arg(long, value_name = "KEYWORDS", help_heading = "Consciousness & Memory")]
    pub memory_find: Option<String>,

    /// Show memory statistics
    #[arg(long, exclusive = true, help_heading = "Consciousness & Memory")]
    pub memory_stats: bool,

    /// Update .m8 consciousness files for a directory
    #[arg(long, value_name = "PATH", help_heading = "Consciousness & Memory")]
    pub update_consciousness: Option<String>,

    // =========================================================================
    // SECURITY
    // =========================================================================
    /// Scan codebase for supply chain attack patterns (default: current dir)
    #[arg(long, value_name = "PATH", default_missing_value = ".", num_args = 0..=1, help_heading = "Security")]
    pub security_scan: Option<String>,

    /// Scan a file for prompt injection patterns
    #[arg(long, value_name = "FILE", help_heading = "Security")]
    pub guardian_scan: Option<String>,

    /// Run Guardian daemon for system-wide AI protection
    #[arg(long, exclusive = true, help_heading = "Security")]
    pub guardian_daemon: bool,

    /// Security cleanup - detect and remove malicious MCP entries
    #[arg(long, exclusive = true, help_heading = "Security")]
    pub cleanup: bool,

    /// Deep integrity scan (exploit detection, hash memory, cert analysis)
    #[arg(long, value_name = "PATH", help_heading = "Security")]
    pub integrity_scan: Option<String>,

    /// Inspect PEM/DER certificates embedded in files (supports --mode json)
    #[arg(long, value_name = "PATH", default_missing_value = ".", num_args = 0..=1,
        conflicts_with_all = ["integrity_scan", "security_scan", "cert_audit"], help_heading = "Security")]
    pub cert_scan: Option<String>,

    /// Audit system CA certificate trust store
    #[arg(long, help_heading = "Security")]
    pub cert_audit: bool,

    /// Generate blacklist script from cert audit (use with --cert-audit)
    #[arg(long, help_heading = "Security")]
    pub cert_generate_script: bool,

    /// Look up a SHA-256 hash in security memory
    #[arg(long, value_name = "SHA256", help_heading = "Security")]
    pub hash_lookup: Option<String>,

    /// Run Paul Erdős–Rényi Phase-Transition & Cross-Boundary Shadow Graph (G_shadow) Supply-Chain Anomaly Detector
    #[arg(long, value_name = "PATH", default_missing_value = ".", num_args = 0..=1, help_heading = "Security")]
    pub erdos_anomaly: Option<String>,

    // =========================================================================
    // GOOGLE DRIVE & VOICE
    // =========================================================================
    /// Google Drive / Gmail operations (status, list, sync, login, logout)
    #[arg(long, value_name = "ACTION", help_heading = "Google Drive & Cloud")]
    pub gdrive: Option<String>,

    /// Voice & audio operations (status, test, speak)
    #[arg(long, value_name = "ACTION", help_heading = "Voice & Audio")]
    pub voice: Option<String>,

    // =========================================================================
    // HOOKS
    // =========================================================================
    /// Install Smart Tree hooks to AI Agent settings (Antigravity/agy & Claude)
    #[arg(long, exclusive = true, help_heading = "Hooks")]
    pub hooks_install: bool,

    /// Manage hooks: enable, disable, status
    #[arg(long, value_name = "ACTION", value_parser = ["enable", "disable", "status"], help_heading = "Hooks")]
    pub hooks_config: Option<String>,

    /// Execute an Antigravity (agy) lifecycle hook handler (stdin JSON -> stdout JSON)
    #[arg(
        long,
        value_name = "EVENT",
        value_parser = [
            "session-start",
            "pre-invocation",
            "pre-tool-use",
            "post-tool-use",
            "stop",
            "SessionStart",
            "PreInvocation",
            "PreToolUse",
            "PostToolUse",
            "Stop"
        ],
        help_heading = "Hooks"
    )]
    pub agy_hook: Option<String>,

    // =========================================================================
    // MEGA SESSIONS
    // =========================================================================
    /// Start a mega session (persistent cross-context conversation)
    #[arg(long, value_name = "NAME", default_missing_value = "", num_args = 0..=1, help_heading = "Mega Sessions")]
    pub mega_start: Option<String>,

    /// Save current mega session snapshot
    #[arg(long, exclusive = true, help_heading = "Mega Sessions")]
    pub mega_save: bool,

    /// List all mega sessions
    #[arg(long, exclusive = true, help_heading = "Mega Sessions")]
    pub mega_list: bool,

    /// Show mega session statistics
    #[arg(long, exclusive = true, help_heading = "Mega Sessions")]
    pub mega_stats: bool,

    // =========================================================================
    // ANALYSIS
    // =========================================================================
    /// Show tokenization statistics for a path
    #[arg(long, value_name = "PATH", help_heading = "Analysis")]
    pub token_stats: Option<String>,

    /// Get wave frequency for a directory
    #[arg(long, value_name = "PATH", help_heading = "Analysis")]
    pub get_frequency: Option<String>,

    /// Detect Ramsey-Theoretic Monochromatic Cliques (K_3 / K_4 Red/Blue/Green) & Turán Extremal Density
    #[arg(long, value_name = "PATH", default_missing_value = ".", num_args = 0..=1, help_heading = "Analysis")]
    pub ramsey_cliques: Option<String>,

    /// Compute Erdős Center Z(G), Graph Radius/Diameter, and Voronoi Steiner Context Backbone
    #[arg(long, value_name = "PATH", default_missing_value = ".", num_args = 0..=1, help_heading = "Analysis")]
    pub erdos_center: Option<String>,

    /// Partition codebase into C conflict-free parallel agent worktrees via Constructive Lovász Local Lemma (Moser-Tardos)
    #[arg(long, alias = "agent-split", value_name = "WORKTREES", default_missing_value = "4", num_args = 0..=1, help_heading = "Analysis")]
    pub erdos_split: Option<u16>,

    /// Execute the Sovereign Ultra-Backyard Loop (DO = reflect → improve → wire → polish → push → readme) over N measured laps
    #[arg(long, alias = "backyard-ultra", alias = "ultrabackyardloop", alias = "do-loop", value_name = "LAPS", default_missing_value = "12", num_args = 0..=1, help_heading = "Analysis")]
    pub ultrabackyard: Option<usize>,

    /// Execute Sovereign ULTRA-OMNI-ZENO v11 engine (upz/wupz 64B@0x62 LitanyWireFrame, BFP-64 Kleene bitplanes, SPQT base-243, 8b-is-[engines], and OS/Agent-agnostic matrix)
    #[arg(long, alias = "ultra-omni-zeno", alias = "upz", alias = "wupz", alias = "8b-engines", value_name = "PATH", default_missing_value = ".", num_args = 0..=1, help_heading = "Analysis")]
    pub zeno: Option<String>,

    // =========================================================================
    // G8T.IS RUST FUNCTION CAPSULES, YOLO FASTEST-PICK & FOKEN SCOREBOARD
    // =========================================================================
    /// Query g8t.is for the fastest verified Rust functions by keyword, domain, or signature (e.g. "binary search", "(u64,u64)->u64")
    #[arg(
        long,
        value_name = "QUERY",
        help_heading = "g8t.is Rust Capsules & Fokens"
    )]
    pub g8t_best: Option<String>,

    /// YOLO fastest-pick mode: return ONLY the #1 fastest verified Rust function (raw drop-in code + 1-line receipt)
    #[arg(long, help_heading = "g8t.is Rust Capsules & Fokens")]
    pub yolo: bool,

    /// Show the live G8T × Smart-Tree Speed & Foken Scoreboard (https://g8t.is/g8t/leaderboard)
    #[arg(
        long,
        alias = "g8t-scoreboard",
        help_heading = "g8t.is Rust Capsules & Fokens"
    )]
    pub g8t_leaderboard: bool,

    /// Submit a pure-Rust function file or snippet to https://g8t.is/g8t/capsules/publish with local hardware cycle measurement
    #[arg(
        long,
        value_name = "FILE_OR_CODE",
        help_heading = "g8t.is Rust Capsules & Fokens"
    )]
    pub g8t_submit: Option<String>,

    /// Partnership attribution for --g8t-submit (encoded into author_aye, e.g. "Hue + Gemini (0+1)")
    #[arg(
        long,
        value_name = "PARTNER",
        default_value = "Hue + Gemini (0+1)",
        help_heading = "g8t.is Rust Capsules & Fokens"
    )]
    pub partner: String,

    /// Domain tag for --g8t-submit (default: "smart-tree-<arch>")
    #[arg(
        long,
        value_name = "DOMAIN",
        help_heading = "g8t.is Rust Capsules & Fokens"
    )]
    pub g8t_domain: Option<String>,

    /// Function name when --g8t-submit is given a raw expression body
    #[arg(
        long,
        value_name = "NAME",
        help_heading = "g8t.is Rust Capsules & Fokens"
    )]
    pub g8t_name: Option<String>,

    /// Filter g8t.is capsules by hardware architecture (e.g. "x86_64", "aarch64")
    #[arg(
        long,
        value_name = "ARCH",
        help_heading = "g8t.is Rust Capsules & Fokens"
    )]
    pub g8t_arch: Option<String>,

    /// Number of top g8t.is functions or scoreboard rows to return (default: 3 for best, 15 for leaderboard)
    #[arg(long, value_name = "N", help_heading = "g8t.is Rust Capsules & Fokens")]
    pub g8t_top: Option<usize>,

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
        default_value = "classic",
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

    /// SSE server port (also used as daemon port)
    #[arg(
        long,
        alias = "daemon-port",
        default_value = "28428",
        help_heading = "Streaming"
    )]
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

    /// Sync all GitHub repositories for a user/org into <dir>/<user>/<repo> (clone new, refresh updated)
    #[arg(
        long,
        alias = "gh-sync",
        value_name = "USER",
        help_heading = "GitHub Sync"
    )]
    pub sync_user: Option<String>,

    /// Base directory for --sync-user (creates <DIR>/<USER>/<repo>, default: current directory)
    #[arg(
        long,
        value_name = "DIR",
        default_value = ".",
        help_heading = "GitHub Sync"
    )]
    pub sync_dir: PathBuf,

    /// Maximum repositories to list with `gh repo list -L <LIMIT>` (default: 10000)
    #[arg(
        long,
        value_name = "LIMIT",
        default_value_t = 10000,
        help_heading = "GitHub Sync"
    )]
    pub sync_limit: usize,

    /// Bypass timestamp cache and refresh eligible clean branches (blocked branches stay untouched)
    #[arg(long, help_heading = "GitHub Sync")]
    pub refresh_all: bool,

    /// Quarantine diverged/broken repos into .diverged/ or .sync-issues/ with a DIFF_REPORT.md and reclone clean copies
    #[arg(long, alias = "quarantine-blocked", help_heading = "GitHub Sync")]
    pub quarantine: bool,
}

#[derive(Debug, Subcommand)]
pub enum Cmd {
    /// Code intelligence powered by CrabCC built into Smart Tree
    #[command(alias = "crab")]
    Symbol(SymbolCommand),

    /// Sync all GitHub repositories for a user/org into <dir>/<user>/<repo> (clone new, refresh updated)
    #[command(alias = "gh-sync", alias = "sync-user", alias = "sync-github")]
    Sync(SyncUserCommand),

    /// Remember a git forge (github, gitlab, forgejo, local dir, or URL) into salient MEM8.
    /// Extracts code + thoughts, incremental diffs, compresses, turns into waves.
    /// Levels control depth: extreme (full nexus), professor/expert/high, down to bullshitter (shallow).
    /// Special full treatment for StandardGalactic. Auto-detects non-git and limits generic sources.
    #[command(alias = "salient", alias = "ingest")]
    Remember(RememberCommand),

    /// Manage the smart-tree daemon (Linux: systemd, macOS: launchctl, Windows: Task Scheduler)
    #[command(subcommand)]
    Service(Service),

    /// Manage project tags
    #[command(subcommand, name = "project-tags")]
    ProjectTags(ProjectTags),

    /// Query, YOLO-pick, submit, and rank pure-Rust function capsules on https://g8t.is
    #[command(alias = "capsela", alias = "foken")]
    G8t(G8tCommand),

    /// Sovereign MemNet local folder/drive sharing, multi-medium routing (ESP32, WiFi, BLE, LoRa, Ethernet),
    /// MEM|8 wave packet Q&A LLM offload, audio/video chunk streaming, and Ultrasonic Chirp acoustic modem
    #[command(alias = "mesh", alias = "chirp")]
    Memnet(MemnetCommand),

    /// Self-maintaining shell autocomplete (Fish, Zsh, Bash) with live folder consciousness & MEM|8 memory descriptions
    #[command(alias = "autocomplete", alias = "complete")]
    Completions(CompletionsCommand),

    /// 3-tier Sovereign local p/q + SPQT ternary + Salient .m8 code healer and model readiness probe
    #[command(alias = "heal", alias = "fix", alias = "pq-heal")]
    PqFix(PqFixCommand),

    /// Pure-Rust 4gaia Sovereign p/q + OEIS A161330 + Salient .m8 trainer & DeepSeek Ollama registrar
    #[command(alias = "train-pq", alias = "gaia-train", alias = "pqm-train")]
    PqTrain(PqTrainCommand),

    /// Sovereign CLI Mastery & Multi-Domain Model Test System (CLI, Code, Art, Love, Poetry, Sound, History)
    #[command(
        alias = "model-test",
        alias = "test-models",
        alias = "eval-models",
        alias = "cli-test"
    )]
    PqTest(PqTestCommand),

    /// Retrain/convert a public model (DeepSeek) to p/q, load the same model in Ollama & Smart Tree, and benchmark head-to-head
    #[command(alias = "bench-pq", alias = "pqm-bench", alias = "ollama-bench")]
    PqBench(PqBenchCommand),
}

#[derive(Debug, Clone, clap::Args)]
pub struct PqBenchCommand {
    /// Public base model to convert/retrain to p/q (default: deepseek-ai/DeepSeek-R1-Distill-Qwen-1.5B)
    #[arg(long, default_value = "deepseek-ai/DeepSeek-R1-Distill-Qwen-1.5B")]
    pub base_model: String,
    /// Ollama model tag for the p/q-converted checkpoint (default: st-deepseek-pqm:latest)
    #[arg(long, default_value = "st-deepseek-pqm:latest")]
    pub ollama_tag: String,
    /// Number of Transformer projection matrices to quantize/train in 4gaia PQM (default: 28)
    #[arg(long, default_value_t = 28)]
    pub layers: usize,
    /// Number of Symplectic Störmer–Verlet Leapfrog steps per layer (default: 32)
    #[arg(long, default_value_t = 32)]
    pub steps: usize,
    /// Force retraining & Ollama registration before running the head-to-head benchmark
    #[arg(long)]
    pub retrain: bool,
    /// Timeout in milliseconds for live Ollama HTTP query probe (default: 2500)
    #[arg(long, default_value_t = 2500)]
    pub ollama_timeout_ms: u64,
    /// Emit structured JSON report
    #[arg(long)]
    pub json: bool,
}

#[derive(Debug, Clone, clap::Args)]
pub struct PqTestCommand {
    /// Domain track to evaluate: all, cli, code, art, love, poetry, sound, history (default: all)
    #[arg(long, short = 'd', default_value = "all")]
    pub domain: String,
    /// Optional model or persona filter (e.g. "in-binary:st-deepseek-pqm-4gaia", "persona:readme-expert", "art", "love")
    #[arg(long, short = 'm')]
    pub model: Option<String>,
    /// Also probe external Ollama models (`st-deepseek-pqm:latest`, `aye-deepseek-pqm:latest`) if running
    #[arg(long)]
    pub include_ollama: bool,
    /// Fail with non-zero exit code if any model fails a test in its intended domain track
    #[arg(long)]
    pub strict: bool,
    /// Emit structured JSON report
    #[arg(long)]
    pub json: bool,
}

#[derive(Debug, Clone, clap::Args)]
pub struct PqTrainCommand {
    /// Model slug/tag for the trained in-binary 4gaia PQM checkpoint (default: st-deepseek-pqm-4gaia)
    #[arg(long, default_value = "st-deepseek-pqm-4gaia")]
    pub tag: String,
    /// Optional prompt to run immediately through the in-binary 4gaia PQM forward pass after training
    #[arg(long, short = 'q')]
    pub ask: Option<String>,
    /// Also export/register a Modelfile to an external Ollama server (default: false; runs 100% in-binary)
    #[arg(long)]
    pub export_ollama: bool,
    /// Base DeepSeek model if `--export-ollama` is used (default: cheet-deepseek-v2:16b)
    #[arg(long, default_value = "cheet-deepseek-v2:16b")]
    pub base_model: String,
    /// Number of Transformer projection matrices to train & quantize with 4gaia PQM (default: 28)
    #[arg(long, default_value_t = 28)]
    pub layers: usize,
    /// Number of 4gaia Symplectic Störmer–Verlet Leapfrog steps per layer (default: 32)
    #[arg(long, default_value_t = 32)]
    pub steps: usize,
    /// Output directory for model.safetensors, manifest.json, and diet JSONL
    #[arg(long)]
    pub out_dir: Option<PathBuf>,
    /// Emit structured JSON report
    #[arg(long)]
    pub json: bool,
}

#[derive(Debug, Clone, clap::Args)]
pub struct PqFixCommand {
    /// Target file or directory to scan and heal (default: current directory)
    #[arg(default_value = ".")]
    pub path: PathBuf,
    /// Apply deterministic Tier-0 p/q and total_cmp fixes directly to disk
    #[arg(long)]
    pub apply: bool,
    /// Run `cargo check` verification gate on the target crate/workspace
    #[arg(long)]
    pub verify: bool,
    /// Optional question or code-repair prompt for Tier-2 in-binary 4gaia p/q synthesis
    #[arg(long, short = 'q')]
    pub ask: Option<String>,
    /// Model engine to use (default: in-binary:st-deepseek-pqm-4gaia; or an Ollama model name)
    #[arg(long, default_value = "in-binary:st-deepseek-pqm-4gaia")]
    pub model: String,
    /// Emit structured JSON report
    #[arg(long)]
    pub json: bool,
}

#[derive(Debug, Clone, clap::Args)]
pub struct CompletionsCommand {
    /// Action: status, sync (or install/maintain), fish, zsh, bash, powershell, folders, memories
    #[arg(default_value = "status")]
    pub action: String,
    /// Optional prefix filter when testing dynamic folder/memory completions
    pub prefix: Option<String>,
}

#[derive(Debug, Clone, clap::Args)]
pub struct MemnetCommand {
    /// Action: status, assess (or hw), advertise (or beacon), graph (or nodes), share, ls (or shares),
    /// routes, add-route, escape (or lll / rtc), ask, teach (or learn), stream-chunk, chirp-encode, chirp-decode
    #[arg(default_value = "status")]
    pub action: String,
    /// Primary argument (folder path for share, question for ask/teach, message/file for chirp-encode, wav path for chirp-decode)
    pub target: Option<String>,
    /// Secondary argument (answer for teach, endpoint for add-route, output wav path for chirp-encode)
    pub extra: Option<String>,
    /// Shared drive space quota in MB offered to local MemNet peers & ESP32s (default: 4096 MB)
    #[arg(long, default_value_t = 4096)]
    pub quota_mb: u64,
    /// Transport medium for add-route or routing (wifi, bluetooth, ethernet, lora, esp32, usb, ultrasonic)
    #[arg(long, default_value = "wifi")]
    pub medium: String,
    /// MemNet node address (0x01..0xEF, default: 0xC5 multi-route)
    #[arg(long, default_value_t = 0xC5)]
    pub addr: u8,
    /// Multi-route via identifier
    #[arg(long, default_value_t = 25)]
    pub via: u8,
    /// Output or input WAV file path for ultrasonic chirp modem
    #[arg(long, short = 'o')]
    pub out: Option<PathBuf>,
    /// Use audible bird-chirp frequencies (1.85–3.15 kHz) instead of ultrasonic (18.5–21.0 kHz)
    #[arg(long)]
    pub audible: bool,
    /// Emit JSON output
    #[arg(long)]
    pub json: bool,
}

#[derive(Debug, Clone, clap::Args)]
pub struct G8tCommand {
    /// Action: leaderboard (or scoreboard), targets (or bounty/arcade), golf (or routines/routinesd), best (or suggest), yolo, submit (or publish), optimize
    #[arg(default_value = "leaderboard")]
    pub action: String,
    /// Query string for best/yolo, routine filter for golf (default: "all"), or file/source_code for submit
    pub target: Option<String>,
    /// Return ONLY the #1 fastest verified Rust function (raw drop-in source + 1-line receipt)
    #[arg(long)]
    pub yolo: bool,
    /// Partnership attribution for submit/golf (e.g. "Hue + Gemini (0+1)")
    #[arg(long, default_value = "Hue + Gemini (0+1)")]
    pub partner: String,
    /// Domain tag for submit
    #[arg(long, default_value = "smart-tree")]
    pub domain: String,
    /// Optional architecture filter (e.g. "x86_64", "aarch64")
    #[arg(long)]
    pub arch: Option<String>,
    /// Number of top results to display (or max routines to golf)
    #[arg(long)]
    pub top: Option<usize>,
}

#[derive(Debug, Clone, clap::Args)]
pub struct SyncUserCommand {
    /// GitHub username or organization (e.g. standardgalactic, 8b-is)
    pub user: String,
    /// Base directory where <user>/<repo> will be synced (default: current directory)
    #[arg(short = 'd', long = "dir", default_value = ".")]
    pub dir: PathBuf,
    /// Maximum repositories to fetch via `gh repo list -L <LIMIT>` (default: 10000)
    #[arg(short = 'L', long = "limit", default_value_t = 10000)]
    pub limit: usize,
    /// Number of parallel clone/refresh workers (default: 4)
    #[arg(short = 'j', long = "jobs", default_value_t = 4)]
    pub jobs: usize,
    /// Bypass timestamp cache and refresh eligible clean branches (blocked branches stay untouched)
    #[arg(long)]
    pub refresh_all: bool,
    /// Skip forked repositories (`gh repo list --source`)
    #[arg(long)]
    pub no_forks: bool,
    /// Skip archived repositories (`gh repo list --no-archived`)
    #[arg(long)]
    pub no_archived: bool,
    /// Quarantine diverged/broken repos into .diverged/ or .sync-issues/ with a DIFF_REPORT.md and reclone clean copies
    #[arg(long, alias = "quarantine-blocked")]
    pub quarantine: bool,
}

/// Remember command for salient MEM8 ingestion from git forges or paths.
#[derive(Debug, clap::Args)]
pub struct RememberCommand {
    /// Git URL (https://github.com/user or git@...) or local path or collection root.
    /// For github.com/standardgalactic it will target the full nexus at 8s.is style storage.
    pub target: String,

    /// Salience / depth level. extreme = full professor level for special collections like StandardGalactic.
    /// professor/expert/high/medium/low/bullshitter controls how deep we go and how much we compress/skill.
    #[arg(long, value_name = "LEVEL", default_value = "expert")]
    pub level: String,

    /// Output MEM8 path (defaults to sensible .mem8 location under the target or ~/.mem8/salient/<user>)
    #[arg(long)]
    pub out: Option<PathBuf>,

    /// Force full re-ingest even if we have prior MEM8 state (otherwise does smart diffs of known points).
    #[arg(long)]
    pub force: bool,

    /// Read back and summarize what has been learned so far in the MEM8 store without re-ingesting.
    #[arg(long, alias = "status", alias = "recall")]
    pub check: bool,

    /// Filter recalled repositories by substring or query when using `--check` (e.g. `--query spherepop`).
    #[arg(long, short = 'q')]
    pub query: Option<String>,
}

#[derive(Debug, clap::Args)]
pub struct SymbolCommand {
    /// Index, find definitions/references, outline a file, or summarize the graph
    #[arg(value_enum)]
    pub operation: SymbolOperation,
    /// Symbol name for sym/refs, or repository-relative file for outline
    pub query: Option<String>,
    /// Repository root (defaults to the current directory)
    #[arg(long, default_value = ".")]
    pub path: PathBuf,
    /// Omit source snippets from results
    #[arg(long)]
    pub no_source: bool,
}

#[derive(Debug, Clone, Copy, ValueEnum)]
pub enum SymbolOperation {
    /// Build or refresh the CrabCC AST symbol index for the repository
    Index,
    /// Find symbol definitions (functions, structs, enums, traits)
    Sym,
    /// Find all call-sites and references to a symbol
    Refs,
    /// Show the structural AST outline of a source file
    Outline,
    /// Summarize the repository symbol dependency graph
    Graph,
}

impl SymbolCommand {
    pub fn parameters(&self) -> serde_json::Value {
        let operation = match self.operation {
            SymbolOperation::Index => "index",
            SymbolOperation::Sym => "sym",
            SymbolOperation::Refs => "refs",
            SymbolOperation::Outline => "outline",
            SymbolOperation::Graph => "graph",
        };
        serde_json::json!({"operation": operation, "name": self.query, "file": self.query, "path": self.path, "include_source": !self.no_source})
    }
}

#[cfg(test)]
mod symbol_cli_tests {
    use super::*;

    #[test]
    fn parses_native_lookup_and_outline_without_losing_the_project_path() {
        let cli = Cli::try_parse_from([
            "st",
            "symbol",
            "sym",
            "greet",
            "--path",
            "project",
            "--no-source",
        ])
        .unwrap();
        let Some(Cmd::Symbol(command)) = cli.cmd else {
            panic!("expected symbol command")
        };
        assert_eq!(
            command.parameters(),
            serde_json::json!({"operation":"sym", "name":"greet", "file":"greet", "path":"project", "include_source":false})
        );
        let cli = Cli::try_parse_from(["st", "crab", "outline", "src/main.rs"]).unwrap();
        let Some(Cmd::Symbol(command)) = cli.cmd else {
            panic!("expected crab alias")
        };
        assert_eq!(command.parameters()["file"], "src/main.rs");
        assert_eq!(command.parameters()["path"], ".");
        assert!(Cli::try_parse_from(["st", "symbol", "not-an-operation"]).is_err());
    }
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
    /// Legacy alias: sort alphabetically by name
    #[value(name = "name", alias = "alpha")]
    Name,
    /// Legacy alias: sort by file size
    #[value(name = "size")]
    Size,
    /// Legacy alias: sort by modification date
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
    /// LOUDS bit-packed sufficient tree & 64B SMTR .m8 capsule (8B-META-WP-v2.1)
    #[value(alias = "smtr-capsule", alias = "smtr_capsule", alias = "capsule")]
    Louds,
    /// Paul Erdős (Straight from The Book) Szemerédi Regularity, Steiner Pruner, Moser-Tardos LLL & Erdős-Rényi Scanner
    #[value(
        name = "the-book",
        alias = "erdos",
        alias = "szemeredi",
        alias = "mem8-regular",
        alias = "book"
    )]
    TheBook,
    /// Sovereign p/q mode: Exact Coprime Rational Arithmetic (Binary GCD), Hölder (L_p, L_q) Dual Spectral Norm, Zarankiewicz z(n;p,q) & Symplectic (p,q) Störmer-Verlet Leapfrog
    #[value(
        name = "pq",
        alias = "p-q",
        alias = "p_q",
        alias = "rational-pq",
        alias = "symplectic"
    )]
    Pq,
    /// Sovereign Ultra-Backyard Loop (DO = reflect → improve → wire → polish → push → readme) measured lap report
    #[value(
        name = "ultrabackyard",
        alias = "backyard-ultra",
        alias = "ultrabackyardloop",
        alias = "do-loop"
    )]
    UltraBackyard,
    /// Sovereign ULTRA-OMNI-ZENO v11 (upz/wupz 64B@0x62 LitanyWireFrame, BFP-64 Kleene bitplanes, SPQT base-243, and 8b-is-[engines])
    #[value(
        name = "zeno",
        alias = "ultra-omni-zeno",
        alias = "upz",
        alias = "wupz",
        alias = "8b-is",
        alias = "8b-engines"
    )]
    Zeno,
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
        OutputMode::Digest
        | OutputMode::Louds
        | OutputMode::TheBook
        | OutputMode::Pq
        | OutputMode::UltraBackyard
        | OutputMode::Zeno => 10,
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
    /// Only log critical errors
    Error,
    /// Log warnings and errors
    Warn,
    /// Standard operational messages (default)
    Info,
    /// Verbose debugging diagnostics
    Debug,
    /// Ultra-detailed execution trace
    Trace,
}
