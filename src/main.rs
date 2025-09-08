// -----------------------------------------------------------------------------
// HEY THERE, ROCKSTAR! You've found main.rs, the backstage pass to st!
// This is where the show starts. We grab the user's request from the command
// line, tune up the scanner, and tell the formatters to make some beautiful music.
//
// Think of this file as the band's charismatic frontman: it gets all the
// attention and tells everyone else what to do.
//
// Brought to you by The Cheet - making code understandable and fun! 🥁🧻
// -----------------------------------------------------------------------------
use anyhow::Result;
use chrono::NaiveDate;
use clap::{CommandFactory, Parser, ValueEnum};
use clap_complete::generate;
// To make our output as vibrant as Trish's spreadsheets!
use flate2::write::ZlibEncoder;
use flate2::Compression;
use regex::Regex;
use std::io::{self, IsTerminal, Write};
use std::path::PathBuf;
use std::time::SystemTime;

// Pulling in the brains of the operation from our library modules.
use st::{
    claude_init::ClaudeInit,
    formatters::{
        ai::AiFormatter,
        ai_json::AiJsonFormatter,
        classic::ClassicFormatter,
        csv::CsvFormatter,
        digest::DigestFormatter,
        hex::HexFormatter,
        json::JsonFormatter,
        ls::LsFormatter,
        markdown::MarkdownFormatter,
        marqant::MarqantFormatter,
        mermaid::{MermaidFormatter, MermaidStyle},
        quantum::QuantumFormatter,
        semantic::SemanticFormatter,
        sse::SseFormatter,
        stats::StatsFormatter,
        tsv::TsvFormatter,
        waste::WasteFormatter,
        Formatter, PathDisplayMode, StreamingFormatter,
    },
    inputs::InputProcessor,
    parse_size,
    rename_project::{rename_project, RenameOptions},
    terminal::SmartTreeTerminal,
    Scanner,
    ScannerConfig, // The mighty Scanner and its configuration.
};

/// We're using the `clap` crate to make this as easy as pie.
/// Why write an argument parser from scratch when you can `clap`? *ba-dum-tss*
#[derive(Parser, Debug)]
#[command(
    name = "st",
    about = "Smart Tree - An intelligent directory visualization tool. Not just a tree, it's a smart-tree!",
    // Custom version handling with update checking
    author   // Automatically pulls authors from Cargo.toml - "8bit-wraith" and "Claude" - what a team!
)]
struct Cli {
    // --- Action Flags ---
    /// Show the cheatsheet.
    #[arg(long, exclusive = true)]
    cheet: bool,

    /// Generate shell completion scripts.
    #[arg(long, exclusive = true, value_name = "SHELL")]
    completions: Option<clap_complete::Shell>,

    /// Generate the man page.
    #[arg(long, exclusive = true)]
    man: bool,

    /// Run `st` as an MCP (Model Context Protocol) server.
    #[arg(long, exclusive = true)]
    mcp: bool,

    /// List the tools `st` provides when running as an MCP server.
    #[arg(long, exclusive = true)]
    mcp_tools: bool,

    /// Show the configuration snippet for the MCP server.
    #[arg(long, exclusive = true)]
    mcp_config: bool,

    /// Show version information and check for updates.
    #[arg(short = 'V', long, exclusive = true)]
    version: bool,

    /// Launch Smart Tree Terminal Interface (STTI) - Your coding companion!
    /// This starts an interactive terminal that anticipates your needs.
    #[arg(long, exclusive = true)]
    terminal: bool,

    /// Rename project - elegant identity transition (format: "OldName" "NewName")
    #[arg(long, exclusive = true, value_names = &["OLD", "NEW"], num_args = 2)]
    rename_project: Option<Vec<String>>,

    /// Set up or update Claude integration for this project.
    /// Smart command that creates .claude/ if missing or updates if it exists.
    /// Automatically detects project type and configures optimal hooks.
    #[arg(long, exclusive = true)]
    setup_claude: bool,

    // --- Scan Arguments ---
    /// Path to the directory or file you want to analyze.
    /// Can also be a URL (http://), QCP query (qcp://), SSE stream, or MEM8 stream (mem8://)
    path: Option<String>,

    /// Specify input type explicitly (filesystem, qcp, sse, openapi, mem8)
    #[arg(long, value_name = "TYPE")]
    input: Option<String>,

    #[command(flatten)]
    scan_opts: ScanArgs,
}

#[derive(Parser, Debug)]
struct ScanArgs {
    /// Choose your adventure! Selects the output format.
    /// From classic human-readable to AI-optimized hex, we've got options.
    #[arg(short, long, value_enum, default_value = "auto")]
    mode: OutputMode,

    /// Feeling like a detective? Find files/directories matching this regex pattern.
    /// Example: --find "README\\.md"
    #[arg(long)]
    find: Option<String>,

    /// Filter by file extension. Show only files of this type (e.g., "rs", "txt").
    /// No leading dot needed, just the extension itself.
    #[arg(long = "type")]
    filter_type: Option<String>,
    /// Filter to show only files (f) or directories (d).
    #[arg(long = "entry-type", value_parser = ["f", "d"])]
    entry_type: Option<String>,

    /// Only show files larger than this size.
    /// Accepts human-readable sizes like "1M" (1 Megabyte), "500K" (500 Kilobytes), "100B" (100 Bytes).
    #[arg(long)]
    min_size: Option<String>,

    /// Only show files smaller than this size.
    /// Same format as --min-size. Let's find those tiny files!
    #[arg(long)]
    max_size: Option<String>,

    /// Time traveler? Show files newer than this date (YYYY-MM-DD format).
    #[arg(long)]
    newer_than: Option<String>,

    /// Or perhaps you prefer antiques? Show files older than this date (YYYY-MM-DD format).
    #[arg(long)]
    older_than: Option<String>,

    /// How deep should we dig? Limits the traversal depth.
    /// Default is 0 (auto) which lets each mode pick its ideal depth.
    /// Set explicitly to override: 1 for shallow, 10 for deep exploration.
    #[arg(short, long, default_value = "0")]
    depth: usize,

    /// Daredevil mode: Ignores `.gitignore` files. See everything, even what Git tries to hide!
    #[arg(long)]
    no_ignore: bool,

    /// Double daredevil: Ignores our built-in default ignore patterns too (like `node_modules`, `__pycache__`).
    /// Use with caution, or you might see more than you bargained for!
    #[arg(long)]
    no_default_ignore: bool,

    /// Show all files, including hidden ones (those starting with a `.`).
    /// The `-a` is for "all", naturally.
    #[arg(long, short = 'a')]
    all: bool,

    /// Want to see what's being ignored? This flag shows ignored directories in brackets `[dirname]`.
    /// Useful for debugging your ignore patterns or just satisfying curiosity.
    #[arg(long)]
    show_ignored: bool,

    /// SHOW ME EVERYTHING! The nuclear option that combines --all, --no-ignore, and --no-default-ignore.
    /// This reveals absolutely everything: hidden files, git directories, node_modules, the works!
    /// Warning: May produce overwhelming output in large codebases.
    #[arg(long)]
    everything: bool,

    /// Show filesystem type indicators in output (e.g., X=XFS, 4=ext4, B=Btrfs).
    /// Each file/directory gets a single character showing what filesystem it's on.
    /// Great for understanding storage layout and mount points!
    #[arg(long)]
    show_filesystems: bool,

    /// Not a fan of emojis? This flag disables them for a plain text experience.
    /// (But Trish loves the emojis, just saying!) 🌳✨
    #[arg(long)]
    no_emoji: bool,

    /// Compress the output using zlib. Great for sending large tree structures over the wire
    /// or for AI models that appreciate smaller inputs. Output will be base64 encoded.
    #[arg(short = 'z', long)]
    compress: bool,

    /// MCP/API optimization mode. Automatically enables compression, disables colors/emoji,
    /// and optimizes output for machine consumption. Perfect for MCP servers, LLM APIs, and tools.
    /// Works with any output mode to make it API-friendly!
    #[arg(long)]
    mcp_optimize: bool,

    /// For JSON output, this makes it compact (one line) instead of pretty-printed.
    /// Saves space, but might make Trish's eyes water if she tries to read it directly.
    #[arg(long)]
    compact: bool,

    /// Controls how file paths are displayed in the output.
    #[arg(long = "path-mode", value_enum, default_value = "off")]
    path_mode: PathMode,

    /// When should we splash some color on the output?
    /// `auto` (default) uses colors if outputting to a terminal.
    #[arg(long, value_enum, default_value = "auto")]
    color: ColorMode,

    /// For AI mode, wraps the output in a JSON structure.
    /// Makes it easier for programmatic consumption by our AI overlords (just kidding... mostly).
    #[arg(long)]
    ai_json: bool,

    /// Stream output as files are scanned. This is a game-changer for very large directories!
    /// You'll see results trickling in, rather than waiting for the whole scan to finish.
    /// Note: Compression is disabled in stream mode for now.
    #[arg(long)]
    stream: bool,

    /// Start SSE server mode for real-time directory monitoring (experimental).
    /// This starts an HTTP server that streams directory changes as Server-Sent Events.
    /// Example: st --sse-server --sse-port 8420 /path/to/watch
    #[arg(long)]
    sse_server: bool,

    /// Port for SSE server mode (default: 8420)
    #[arg(long, default_value = "8420")]
    sse_port: u16,

    /// Search for a keyword within file contents.
    /// Best used with `--type` to limit search to specific file types (e.g., `--type rs --search "TODO"`).
    /// This is like having X-ray vision for your files!
    #[arg(long)]
    search: Option<String>,

    /// Group files by semantic similarity (inspired by Omni's wisdom!).
    /// Uses content-aware tokenization to identify conceptually related files.
    /// Perfect for understanding project structure at a higher level.
    /// Example groups: "tests", "documentation", "configuration", "source code"
    #[arg(long)]
    semantic: bool,

    /// Mermaid diagram style (only used with --mode mermaid).
    /// Options: flowchart (default), mindmap, gitgraph
    #[arg(long, value_enum, default_value = "flowchart")]
    mermaid_style: MermaidStyleArg,

    /// Exclude mermaid diagrams from markdown report (only used with --mode markdown).
    #[arg(long)]
    no_markdown_mermaid: bool,

    /// Exclude tables from markdown report (only used with --mode markdown).
    #[arg(long)]
    no_markdown_tables: bool,

    /// Exclude pie charts from markdown report (only used with --mode markdown).
    #[arg(long)]
    no_markdown_pie_charts: bool,

    /// Focus analysis on specific file (for relations mode).
    /// Shows all relationships for a particular file.
    #[arg(long, value_name = "FILE")]
    focus: Option<PathBuf>,

    /// Filter relationships by type (for relations mode).
    /// Options: imports, calls, types, tests, coupled
    #[arg(long, value_name = "TYPE")]
    relations_filter: Option<String>,

    /// Sort results by: a-to-z, z-to-a, largest, smallest, newest, oldest, type
    /// Examples: --sort largest (biggest files first), --sort newest (recent files first)
    /// Use with --top to get "top 10 largest files" or "20 newest files"
    #[arg(long, value_enum)]
    sort: Option<SortField>,

    /// Show only the top N results (useful with --sort)
    /// Examples: --sort size --top 10 (10 largest files)
    /// --sort date --top 20 (20 most recent files)
    #[arg(long, value_name = "N")]
    top: Option<usize>,

    /// Include private functions in function documentation (for function-markdown mode)
    /// By default, only public functions are shown
    #[arg(long)]
    show_private: bool,

    /// View diffs stored in the .st folder (Smart Edit history)
    /// Shows all diffs for files modified by Smart Edit operations
    #[arg(long)]
    view_diffs: bool,

    /// Clean up old diffs in .st folder, keeping only last N per file
    /// Example: --cleanup-diffs 5 (keep last 5 diffs per file)
    #[arg(long, value_name = "N")]
    cleanup_diffs: Option<usize>,
}

/// Sort field options with intuitive names
#[derive(Debug, Clone, Copy, ValueEnum)]
enum SortField {
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
enum MermaidStyleArg {
    /// Traditional flowchart (default)
    Flowchart,
    /// Mind map style
    Mindmap,
    /// Git graph style
    Gitgraph,
    /// Treemap style (shows file sizes visually)
    Treemap,
}

/// Enum defining how color should be used in the output.
/// Because life's too short for monochrome (unless you ask for it).
#[derive(Debug, Clone, Copy, ValueEnum)]
enum ColorMode {
    /// Always use colors, no matter what. Go vibrant!
    Always,
    /// Never use colors. For the minimalists.
    Never,
    /// Use colors if the output is a terminal (tty), otherwise disable. This is the default smart behavior.
    Auto,
}

/// Enum defining how paths should be displayed.
/// Sometimes you want the full story, sometimes just the filename.
#[derive(Debug, Clone, Copy, ValueEnum)]
enum PathMode {
    /// Show only filenames (default). Clean and simple.
    Off,
    /// Show paths relative to the scan root. Good for context within the project.
    Relative,
    /// Show full absolute paths. Leaves no doubt where things are.
    Full,
}

/// Enum defining the available output modes.
/// Each mode tailors the output for a specific purpose or audience.
#[derive(Debug, Clone, Copy, ValueEnum, PartialEq)]
enum OutputMode {
    /// Auto mode - smart default selection based on context
    Auto,
    /// Classic tree format, human-readable with metadata and emojis (unless disabled). Our beloved default.
    Classic,
    /// Hexadecimal format with fixed-width fields. Excellent for AI parsing or detailed analysis.
    Hex,
    /// JSON output. Structured data for easy programmatic use.
    Json,
    /// Unix ls -Alh format. Familiar directory listing with human-readable sizes and permissions.
    Ls,
    /// AI-optimized format. A special blend of hex tree and statistics, designed for LLMs.
    Ai,
    /// Directory statistics only. Get a summary without the full tree.
    Stats,
    /// CSV (Comma Separated Values) format. Spreadsheet-friendly.
    Csv,
    /// TSV (Tab Separated Values) format. Another spreadsheet favorite.
    Tsv,
    /// Super compact digest format. A single line with a hash and minimal stats, perfect for quick AI pre-checks.
    Digest,
    /// Emotional tree - Files with FEELINGS! They get happy, sad, anxious, proud!
    Emotional,
    /// MEM|8 Quantum format. The ultimate compression with bitfield headers and tokenization.
    Quantum,
    /// Semantic grouping format. Groups files by conceptual similarity (inspired by Omni!).
    Semantic,
    /// Mermaid diagram format. Perfect for embedding in documentation!
    Mermaid,
    /// Markdown report format. Combines mermaid, tables, and charts for beautiful documentation!
    Markdown,
    /// Interactive summary mode (default for humans in terminal)
    Summary,
    /// AI-optimized summary mode (default for AI/piped output)
    SummaryAi,
    /// Context mode for AI conversations - provides git, memory, and structure context
    Context,
    /// Code relationship analysis
    Relations,
    /// Quantum compression with semantic understanding (Omni's nuclear option!)
    QuantumSemantic,
    /// Waste detection and optimization analysis (Marie Kondo mode!)
    Waste,
    /// Marqant - Quantum-compressed markdown format (.mq)
    Marqant,
    /// SSE - Server-Sent Events streaming format for real-time monitoring
    Sse,
    /// Function documentation in markdown format - living blueprints of your code!
    FunctionMarkdown,
}

/// Parses a date string (YYYY-MM-DD) into a `SystemTime` object.
/// This is our time machine! It parses a date string (like "2025-12-25")
/// into a `SystemTime` object that Rust can understand.
/// Perfect for finding files from the past or... well, not the future. Yet.
/// Get the ideal depth for each output mode
///
/// When users don't specify depth (depth = 0), each mode gets its optimal default:
/// - LS mode: 1 (like real ls, shows only immediate children)
/// - Classic: 3 (good overview without overwhelming)
/// - AI/Hex: 5 (more detail for analysis)
/// - Stats: 10 (comprehensive statistics)
/// - Others: 4 (balanced default)
fn get_ideal_depth_for_mode(mode: &OutputMode) -> usize {
    match mode {
        OutputMode::Auto => 3, // Should never reach here, but default to classic depth
        OutputMode::Ls => 1,   // Mimics 'ls' behavior
        OutputMode::Classic => 3, // Balanced tree view
        OutputMode::Ai | OutputMode::Hex => 5, // More detail for analysis
        OutputMode::Stats => 10, // Comprehensive stats
        OutputMode::Digest => 10, // Full scan for accurate digest
        OutputMode::Emotional => 5, // Enough to see personality development
        OutputMode::Quantum | OutputMode::QuantumSemantic => 5, // Good compression balance
        OutputMode::Summary | OutputMode::SummaryAi | OutputMode::Context => 4, // Reasonable summary depth
        OutputMode::Waste => 10,     // Deep scan to find all duplicates
        OutputMode::Relations => 10, // Deep scan for relationships
        _ => 4,                      // Default for other modes
    }
}

fn parse_date(date_str: &str) -> Result<SystemTime> {
    // Attempt to parse the date string.
    let date = NaiveDate::parse_from_str(date_str, "%Y-%m-%d")?;
    // Assume midnight (00:00:00) for the given date.
    let datetime = date.and_hms_opt(0, 0, 0).unwrap(); // This unwrap is safe as 00:00:00 is always valid.
                                                       // Convert to SystemTime.
    Ok(SystemTime::UNIX_EPOCH
        + std::time::Duration::from_secs(datetime.and_utc().timestamp() as u64))
}

/// And now, the moment you've all been waiting for: the `main` function!
/// This is the heart of the st concert. It's where we parse the arguments,
/// configure the scanner, pick the right formatter for the job, and let it rip.
/// It returns a `Result` because sometimes, even rockstars hit a wrong note.
#[tokio::main]
async fn main() -> Result<()> {
    // Parse the command-line arguments provided by the user.
    let cli = Cli::parse();

    // Handle exclusive action flags first.
    if cli.cheet {
        let markdown = std::fs::read_to_string("docs/st-cheetsheet.md")?;
        let skin = termimad::MadSkin::default();
        skin.print_text(&markdown);
        return Ok(());
    }
    if let Some(shell) = cli.completions {
        let mut cmd = Cli::command();
        let bin_name = cmd.get_name().to_string();
        generate(shell, &mut cmd, bin_name, &mut io::stdout());
        return Ok(());
    }
    if cli.man {
        let cmd = Cli::command();
        let man = clap_mangen::Man::new(cmd);
        man.render(&mut io::stdout())?;
        return Ok(());
    }
    if cli.mcp {
        return run_mcp_server().await;
    }
    if cli.mcp_tools {
        print_mcp_tools();
        return Ok(());
    }
    if cli.mcp_config {
        print_mcp_config();
        return Ok(());
    }
    // Handle diff storage operations
    if cli.scan_opts.view_diffs {
        return handle_view_diffs().await;
    }
    if let Some(keep_count) = cli.scan_opts.cleanup_diffs {
        return handle_cleanup_diffs(keep_count).await;
    }

    if cli.terminal {
        return run_terminal().await;
    }
    if cli.version {
        return show_version_with_updates().await;
    }
    if let Some(names) = cli.rename_project {
        if names.len() != 2 {
            eprintln!("Error: rename-project requires exactly two arguments: OLD_NAME NEW_NAME");
            return Ok(());
        }
        let options = RenameOptions::default();
        return rename_project(&names[0], &names[1], options).await;
    }

    // Handle Claude integration setup (smart init or update)
    if cli.setup_claude {
        let project_path = std::env::current_dir()?;
        let initializer = ClaudeInit::new(project_path)?;
        return initializer.setup();
    }

    // If no action flag was given, proceed with the scan.
    let args = cli.scan_opts;
    let input_str = cli.path.unwrap_or_else(|| ".".to_string());

    // --- Environment Variable Overrides ---
    // Check for ST_DEFAULT_MODE environment variable to override the default output mode.
    // This allows users to set a persistent preference.
    let default_mode_env = std::env::var("ST_DEFAULT_MODE")
        .ok() // Convert Result to Option
        .and_then(|m| match m.to_lowercase().as_str() {
            // Match on the lowercase string value
            "classic" => Some(OutputMode::Classic),
            "hex" => Some(OutputMode::Hex),
            "json" => Some(OutputMode::Json),
            "ls" => Some(OutputMode::Ls),
            "ai" => Some(OutputMode::Ai),
            "stats" => Some(OutputMode::Stats),
            "csv" => Some(OutputMode::Csv),
            "tsv" => Some(OutputMode::Tsv),
            "digest" => Some(OutputMode::Digest),
            "quantum" => Some(OutputMode::Quantum),
            "semantic" => Some(OutputMode::Semantic),
            "mermaid" => Some(OutputMode::Mermaid),
            "markdown" => Some(OutputMode::Markdown),
            "relations" => Some(OutputMode::Relations),
            "summary" => Some(OutputMode::Summary),
            "summary-ai" => Some(OutputMode::SummaryAi),
            "context" => Some(OutputMode::Context),
            "quantum-semantic" => Some(OutputMode::QuantumSemantic),
            "waste" => Some(OutputMode::Waste),
            "marqant" => Some(OutputMode::Marqant),
            "sse" => Some(OutputMode::Sse),
            "function-markdown" => Some(OutputMode::FunctionMarkdown),
            _ => None, // Unknown mode string, ignore.
        });

    // Determine the final mode and compression settings.
    // The AI_TOOLS environment variable takes highest precedence.
    // Then, --semantic flag overrides the mode.
    // Then, the command-line --mode flag.
    // Then, ST_DEFAULT_MODE environment variable.
    // Finally, the default mode from clap.
    let is_ai_caller =
        std::env::var("AI_TOOLS").is_ok_and(|v| v == "1" || v.to_lowercase() == "true");

    // MCP optimization: enables compression and AI-friendly settings
    let mcp_mode = args.mcp_optimize || is_ai_caller;

    let (mut mode, compress) = if mcp_mode {
        // If MCP optimization is requested or AI_TOOLS is set, use API-optimized settings
        match args.mode {
            OutputMode::Auto => (OutputMode::Ai, true), // Default to AI mode for MCP
            OutputMode::Summary => (OutputMode::SummaryAi, true), // Auto-switch to AI version
            other => (other, true),                     // Keep other modes but enable compression
        }
    } else if args.semantic {
        // If --semantic flag is set, use semantic mode (Omni's wisdom!)
        (OutputMode::Semantic, args.compress)
    } else if args.mode != OutputMode::Auto {
        // User explicitly specified a mode via command line - this takes precedence!
        (args.mode, args.compress)
    } else if let Some(env_mode) = default_mode_env {
        // If ST_DEFAULT_MODE is set and user didn't specify mode, use it
        (env_mode, args.compress)
    } else {
        // No explicit mode specified anywhere, use classic as default
        (OutputMode::Classic, args.compress)
    };

    // Auto-switch to ls mode when using --top with classic mode
    if args.top.is_some() && matches!(mode, OutputMode::Classic) {
        // Check if user explicitly provided --mode flag by looking at CLI args
        let user_specified_mode = std::env::args().any(|arg| arg == "--mode" || arg == "-m");

        if !user_specified_mode && default_mode_env.is_none() {
            // Auto-switch only if user didn't explicitly choose a mode
            eprintln!(
                "💡 Auto-switching to ls mode for --top results (use --mode classic to override)"
            );
            mode = OutputMode::Ls;
        } else {
            // User explicitly chose classic mode or set env var, just show the note
            eprintln!("💡 Note: --top doesn't limit results in classic tree mode.");
            eprintln!("   Tree mode needs all entries to build the structure.");
            eprintln!("   Use --mode ls for limited results: st --mode ls --sort largest --top 10");
        }
    }

    // --- Color Configuration ---
    // Determine if colors should be used in the output.
    // MCP mode disables colors for clean API output.
    // Command-line --color flag takes precedence.
    // Then, ST_COLOR or NO_COLOR environment variables.
    // Finally, auto-detect based on whether stdout is a TTY.
    let use_color = if mcp_mode {
        false // MCP mode always disables colors
    } else {
        match args.color {
            ColorMode::Always => true,
            ColorMode::Never => false,
            ColorMode::Auto => {
                // Check environment variables first for explicit overrides.
                if std::env::var("ST_COLOR").as_deref() == Ok("always") {
                    true
                } else if std::env::var("NO_COLOR").is_ok()
                    || std::env::var("ST_COLOR").as_deref() == Ok("never")
                {
                    false
                } else {
                    // If no env var override, check if stdout is a terminal.
                    std::io::stdout().is_terminal()
                }
            }
        }
    };

    // If colors are disabled, globally turn them off for the `colored` crate.
    if !use_color {
        colored::control::set_override(false);
    }

    // MCP mode also disables emoji for clean output
    let no_emoji = args.no_emoji || mcp_mode;

    // --- Scanner Configuration ---
    // Build the configuration for the directory scanner.
    // Handle the --everything flag which overrides other visibility settings
    let (no_ignore_final, no_default_ignore_final, all_final) = if args.everything {
        (true, true, true) // Override all ignore/hide settings
    } else {
        // Check if the root path itself is a commonly ignored directory
        // If so, automatically disable default ignores AND show hidden files
        let path = PathBuf::from(&input_str);
        let root_name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");

        // List of directory names that should disable default ignores when explicitly targeted
        let auto_disable_ignores = matches!(
            root_name,
            ".git"
                | ".svn"
                | ".hg"
                | "node_modules"
                | "target"
                | "build"
                | "dist"
                | "__pycache__"
                | ".cache"
                | ".pytest_cache"
                | ".mypy_cache"
        );

        // For hidden directories like .git, also enable showing hidden files
        let auto_show_hidden = auto_disable_ignores && root_name.starts_with('.');

        let no_default_ignore = args.no_default_ignore || auto_disable_ignores;
        let all = args.all || auto_show_hidden;
        (args.no_ignore, no_default_ignore, all)
    };

    // For AI mode, we automatically enable `show_ignored` to provide maximum context to the AI,
    // unless the user explicitly set `show_ignored` (which `args.show_ignored` would capture).
    let show_ignored_final =
        args.show_ignored || matches!(mode, OutputMode::Ai | OutputMode::Digest) || args.everything;

    // Smart defaults: Each mode has an ideal depth when user doesn't specify (depth = 0)
    let effective_depth = if args.depth == 0 {
        get_ideal_depth_for_mode(&mode)
    } else {
        args.depth // User explicitly set depth, respect their choice
    };

    let scanner_config = ScannerConfig {
        max_depth: effective_depth,
        follow_symlinks: false, // Symlink following is generally off for safety and simplicity.
        respect_gitignore: !no_ignore_final,
        show_hidden: all_final,
        show_ignored: show_ignored_final,
        // Attempt to compile the find pattern string into a Regex.
        find_pattern: args.find.as_ref().map(|p| Regex::new(p)).transpose()?,
        file_type_filter: args.filter_type.clone(),
        entry_type_filter: args.entry_type.clone(),
        // Parse human-readable size strings (e.g., "1M") into u64 bytes.
        min_size: args.min_size.as_ref().map(|s| parse_size(s)).transpose()?,
        max_size: args.max_size.as_ref().map(|s| parse_size(s)).transpose()?,
        // Parse date strings (YYYY-MM-DD) into SystemTime.
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
        use_default_ignores: !no_default_ignore_final,
        search_keyword: args.search.clone(),
        show_filesystems: args.show_filesystems,
        sort_field: args.sort.map(|f| match f {
            SortField::AToZ | SortField::Name => "a-to-z".to_string(),
            SortField::ZToA => "z-to-a".to_string(),
            SortField::Largest | SortField::Size => "largest".to_string(),
            SortField::Smallest => "smallest".to_string(),
            SortField::Newest | SortField::Date => "newest".to_string(),
            SortField::Oldest => "oldest".to_string(),
            SortField::Type => "type".to_string(),
        }),
        top_n: args.top,
        include_line_content: false,
    };

    // 🌊 Universal Input Processing
    // Detect input type and determine root path
    let (is_traditional_fs, scan_path) = if cli.input.is_some()
        || !input_str.starts_with(".") && !PathBuf::from(&input_str).exists()
    {
        (false, PathBuf::from(&input_str))
    } else {
        (true, PathBuf::from(&input_str))
    };

    // Convert the command-line PathMode enum to the formatter's PathDisplayMode enum.
    let path_display_mode = match args.path_mode {
        PathMode::Off => PathDisplayMode::Off,
        PathMode::Relative => PathDisplayMode::Relative,
        PathMode::Full => PathDisplayMode::Full,
    };

    // --- Output Generation ---
    // Decide whether to use streaming mode or normal (scan-all-then-format) mode.
    // Streaming is enabled by --stream flag AND if compression is NOT requested (as zlib needs the whole buffer).
    // Note: Streaming is only supported for traditional filesystem scanning
    if args.stream && !compress && is_traditional_fs {
        // Streaming mode is only supported for certain formatters that implement StreamingFormatter.
        match mode {
            OutputMode::Hex | OutputMode::Ai | OutputMode::Quantum => {
                // For streaming, we use threads: one for scanning, one for formatting/printing.
                // A channel is used for communication between them.
                use std::sync::mpsc; // Multi-producer, single-consumer channel.
                use std::thread;

                let (tx, rx) = mpsc::channel(); // Create the communication channel.

                // Recreate scanner for streaming (since we consumed it above)
                let path = PathBuf::from(&input_str);
                let scanner = Scanner::new(&path, scanner_config)?;
                let scanner_root = scanner.root().to_path_buf(); // Get the canonicalized root before moving scanner

                // Spawn the scanner thread. It will send FileNode objects through the channel.
                let scanner_thread = thread::spawn(move || {
                    // The scanner's scan_stream method takes the sender part of the channel.
                    scanner.scan_stream(tx)
                });

                // Create the appropriate streaming formatter based on the selected mode.
                let streaming_formatter: Box<dyn StreamingFormatter> = match mode {
                    OutputMode::Hex => Box::new(HexFormatter::new(
                        use_color,
                        no_emoji,
                        show_ignored_final,
                        path_display_mode,
                        args.show_filesystems,
                    )),
                    OutputMode::Ai => Box::new(AiFormatter::new(no_emoji, path_display_mode)),
                    OutputMode::Quantum => Box::new(QuantumFormatter::new()),
                    _ => unreachable!(), // Should not happen due to the outer match.
                };

                // Get a lock on stdout for writing.
                let stdout = io::stdout();
                let mut handle = stdout.lock();

                // Initialize the stream with the formatter (e.g., print headers).
                streaming_formatter.start_stream(&mut handle, &scanner_root)?;

                // Receive and format nodes as they arrive from the scanner thread.
                while let Ok(node) = rx.recv() {
                    // Loop until the channel is closed.
                    streaming_formatter.format_node(&mut handle, &node, &scanner_root)?;
                }

                // Wait for the scanner thread to finish and get the final statistics.
                // The `??` propagates errors from thread panic and from the scan_stream result.
                let stats = scanner_thread
                    .join()
                    .map_err(|_| anyhow::anyhow!("Scanner thread panicked"))??;

                // Finalize the stream with the formatter (e.g., print footers or summary stats).
                streaming_formatter.end_stream(&mut handle, &stats, &scanner_root)?;
            }
            _ => {
                // If streaming is requested for an unsupported mode, print an error and exit.
                // Trish says clear error messages are important!
                eprintln!("Streaming mode is currently only supported for 'hex' and 'ai' output modes when not using compression.");
                std::process::exit(1); // Exit with a non-zero status code to indicate an error.
            }
        }
    } else {
        // Normal (non-streaming) mode or when compression is enabled.
        // Scan the entire directory structure first.
        // Get nodes and stats based on input type
        let (nodes, stats, root_path) = if is_traditional_fs {
            let scanner = Scanner::new(&scan_path, scanner_config)?;
            let root = scanner.root().to_path_buf();
            let (n, s) = scanner.scan()?;
            (n, s, root)
        } else {
            // Use universal input processor for non-filesystem inputs
            eprintln!("🌊 Detecting input type...");
            let input_processor = InputProcessor::new();
            let input_source = InputProcessor::detect_input_type(&input_str);

            let context_root = input_processor.process(input_source).await?;

            // Convert context nodes to file nodes
            let file_nodes = st::inputs::context_to_file_nodes(context_root);

            // Create synthetic stats
            let stats = st::TreeStats {
                total_files: file_nodes.iter().filter(|n| !n.is_dir).count() as u64,
                total_dirs: file_nodes.iter().filter(|n| n.is_dir).count() as u64,
                total_size: file_nodes.iter().map(|n| n.size).sum(),
                file_types: std::collections::HashMap::new(),
                largest_files: vec![],
                newest_files: vec![],
                oldest_files: vec![],
            };

            (file_nodes, stats, scan_path.clone())
        };

        // Create the appropriate formatter based on the selected mode.
        let formatter: Box<dyn Formatter> = match mode {
            OutputMode::Auto => {
                // Auto mode should have been resolved to a specific mode by now
                unreachable!("Auto mode should have been resolved before formatter selection")
            }
            OutputMode::Classic => {
                // Special handling for classic mode with --find: default to relative paths
                // if user hasn't explicitly set a path_mode other than 'off'.
                // This makes --find output more useful by showing where found items are.
                let classic_path_mode =
                    if args.find.is_some() && matches!(args.path_mode, PathMode::Off) {
                        PathDisplayMode::Relative
                    } else {
                        path_display_mode // Use the user-specified or default path_mode.
                    };
                let formatter = ClassicFormatter::new(no_emoji, use_color, classic_path_mode);
                let sort_field = args.sort.map(|f| match f {
                    SortField::AToZ | SortField::Name => "a-to-z".to_string(),
                    SortField::ZToA => "z-to-a".to_string(),
                    SortField::Largest | SortField::Size => "largest".to_string(),
                    SortField::Smallest => "smallest".to_string(),
                    SortField::Newest | SortField::Date => "newest".to_string(),
                    SortField::Oldest => "oldest".to_string(),
                    SortField::Type => "type".to_string(),
                });
                Box::new(formatter.with_sort(sort_field))
            }
            OutputMode::Hex => Box::new(HexFormatter::new(
                use_color,
                no_emoji,
                show_ignored_final,
                path_display_mode,
                args.show_filesystems,
            )),
            OutputMode::Json => Box::new(JsonFormatter::new(args.compact)),
            OutputMode::Ls => Box::new(LsFormatter::new(!no_emoji, use_color)),
            OutputMode::Ai => {
                // AI mode can optionally be wrapped in JSON.
                if args.ai_json {
                    Box::new(AiJsonFormatter::new(no_emoji, path_display_mode))
                } else {
                    Box::new(AiFormatter::new(no_emoji, path_display_mode))
                }
            }
            OutputMode::Stats => Box::new(StatsFormatter::new()),
            OutputMode::Csv => Box::new(CsvFormatter::new()),
            OutputMode::Tsv => Box::new(TsvFormatter::new()),
            OutputMode::Digest => Box::new(DigestFormatter::new()),
            OutputMode::Emotional => {
                use st::formatters::emotional_new::EmotionalFormatter;
                Box::new(EmotionalFormatter::new(use_color))
            }
            OutputMode::Quantum => Box::new(QuantumFormatter::new()),
            OutputMode::Semantic => Box::new(SemanticFormatter::new(path_display_mode, no_emoji)),
            OutputMode::Mermaid => {
                // Convert CLI arg enum to formatter enum
                let style = match args.mermaid_style {
                    MermaidStyleArg::Flowchart => MermaidStyle::Flowchart,
                    MermaidStyleArg::Mindmap => MermaidStyle::Mindmap,
                    MermaidStyleArg::Gitgraph => MermaidStyle::GitGraph,
                    MermaidStyleArg::Treemap => MermaidStyle::Treemap,
                };
                Box::new(MermaidFormatter::new(style, no_emoji, path_display_mode))
            }
            OutputMode::Markdown => {
                // Create a comprehensive markdown report with all visualizations!
                Box::new(MarkdownFormatter::new(
                    path_display_mode,
                    no_emoji,
                    !args.no_markdown_mermaid, // Include mermaid unless disabled
                    !args.no_markdown_tables,  // Include tables unless disabled
                    !args.no_markdown_pie_charts, // Include pie charts unless disabled
                ))
            }
            OutputMode::Marqant => Box::new(MarqantFormatter::new(path_display_mode, no_emoji)),
            OutputMode::Sse => {
                // SSE streaming format for real-time monitoring
                Box::new(SseFormatter::new())
            }
            OutputMode::Relations => {
                // Code relationship analysis - "Semantic X-ray vision!" - Omni
                use st::formatters::relations_formatter::RelationsFormatter;
                Box::new(RelationsFormatter::new(
                    args.relations_filter.clone(),
                    args.focus.clone(),
                ))
            }
            OutputMode::Summary => {
                // Interactive summary for humans - "Smart defaults!" - Omni
                use st::formatters::summary::SummaryFormatter;
                Box::new(SummaryFormatter::new(use_color))
            }
            OutputMode::SummaryAi => {
                // Compressed summary for AI consumption
                use st::formatters::summary_ai::SummaryAiFormatter;
                Box::new(SummaryAiFormatter::new(compress))
            }
            OutputMode::Context => {
                // Context mode for AI conversations
                use st::formatters::context::ContextFormatter;
                Box::new(ContextFormatter::new())
            }
            OutputMode::QuantumSemantic => {
                // Semantic-aware quantum compression - "The nuclear option!" - Omni
                use st::formatters::quantum_semantic::QuantumSemanticFormatter;
                Box::new(QuantumSemanticFormatter::new())
            }
            OutputMode::Waste => {
                // Waste detection and optimization analysis - "Marie Kondo mode!" - Hue & Aye
                Box::new(WasteFormatter::new())
            }
            OutputMode::FunctionMarkdown => {
                // Function documentation in markdown - "Living blueprints of your code!" - Trisha
                use st::formatters::function_markdown::FunctionMarkdownFormatter;
                Box::new(FunctionMarkdownFormatter::new(
                    args.show_private,
                    true, // show_complexity
                    true, // show_call_graph
                ))
            }
        };

        // Format the collected nodes and stats into a byte vector.
        let mut output_buffer = Vec::new();
        formatter.format(&mut output_buffer, &nodes, &stats, &root_path)?;

        // Handle compression if requested.
        if compress {
            let compressed_data = compress_output(&output_buffer)?;
            // Print the compressed data as a base64 encoded string, prefixed for identification.
            // Using hex encoding for binary data to make it printable.
            println!("COMPRESSED_V1:{}", hex::encode(&compressed_data));
        } else {
            // If not compressing, write the formatted output directly to stdout.
            io::stdout().write_all(&output_buffer)?;

            // Show helpful tips for human users (not when piped, redirected, or in AI modes)
            if IsTerminal::is_terminal(&io::stdout())
                && !compress
                && !matches!(
                    mode,
                    OutputMode::Ai
                        | OutputMode::Hex
                        | OutputMode::Digest
                        | OutputMode::Quantum
                        | OutputMode::QuantumSemantic
                )
            {
                show_helpful_tips(&mode, effective_depth, &args)?;
            }
        }
    }

    // If we've reached here, everything went well!
    Ok(())
}

/// Show helpful tips to human users to improve their smart-tree experience
///
/// This provides contextual suggestions based on current usage patterns,
/// helping users discover powerful features and optimize their workflow.
/// Trish loves these little nuggets of wisdom! 💡
fn show_helpful_tips(mode: &OutputMode, depth: usize, args: &ScanArgs) -> Result<()> {
    use rand::seq::SliceRandom;

    let mut tips = Vec::new();

    // Mode-specific tips
    match mode {
        OutputMode::Auto => {
            // Auto mode should never reach here, but just in case
            tips.push("🤖 Auto mode intelligently selects the best format for your use case!");
        }
        OutputMode::Classic => {
            if depth > 5 {
                tips.push("💡 Deep trees can be overwhelming. Try reducing depth with -d 3 or use --mode ls for a clean listing!");
            }
            if depth == 0 || depth == 3 {
                tips.push("🌳 Classic mode auto-selects depth 3 when depth is 0 (auto). Use -d to override!");
            }
            tips.push("🚀 Pro tip: Set ST_DEFAULT_MODE=ls for instant directory listings!");
        }
        OutputMode::Ls => {
            tips.push("✨ You're using LS mode! This mimics 'ls -Alh' but with smart-tree magic.");
            if depth == 1 {
                tips.push(
                    "🎯 Perfect! Depth 1 is ideal for LS mode - just like the real ls command.",
                );
            }
            tips.push("💾 Save time: export ST_DEFAULT_MODE=ls to make this your default!");
        }
        OutputMode::Waste => {
            tips.push("🧹 Great choice! Waste mode helps you Marie Kondo your projects.");
            tips.push("💰 Run the suggested cleanup commands to reclaim disk space!");
        }
        OutputMode::Ai => {
            tips.push("🤖 AI mode provides LLM-optimized output for perfect code analysis.");
            tips.push("⚡ Combine with --compress for ultra-efficient AI input!");
        }
        _ => {
            // General tips for other modes
            tips.push("🗂️ Try --mode ls -d 1 for a clean directory view like 'ls -Alh'!");
            tips.push("🧹 Use --mode waste to find duplicate files and reclaim disk space!");
        }
    }

    // Depth-specific tips
    if depth > 5 {
        tips.push("🌳 Deep exploration detected! Consider -d 2 or -d 3 for better readability.");
    }

    // Feature discovery tips
    if !args.all {
        tips.push("👀 Add -a to see hidden files and directories (like .gitignore).");
    }

    if args.filter_type.is_none() && args.entry_type.is_none() {
        tips.push(
            "🔍 Filter by type: --entry-type f (files only) or --entry-type d (directories only).",
        );
    }

    // Random general tips
    let general_tips = [
        "📚 Check out --mode markdown for beautiful project documentation!",
        "🔥 Use --mode quantum for ultra-compressed output perfect for AI analysis!",
        "📊 Try --mode stats for quick project metrics and insights!",
        "🎨 Smart-tree respects your terminal colors and emoji preferences!",
        "⚡ Set ST_DEFAULT_MODE environment variable to save your preferred mode!",
        "🔧 Use --find 'pattern' to search for specific files across your tree!",
    ];

    // Add 1-2 random general tips
    let mut rng = rand::thread_rng();
    let selected_general: Vec<_> = general_tips.choose_multiple(&mut rng, 2).collect();
    for tip in selected_general {
        tips.push(tip);
    }

    // Show a maximum of 3 tips to avoid overwhelming the user
    let selected_tips: Vec<_> = tips.choose_multiple(&mut rng, 3.min(tips.len())).collect();

    if !selected_tips.is_empty() {
        eprintln!(); // Add some space
        eprintln!("\x1b[2m───────────────────────────────────────────────────────\x1b[0m");
        for tip in selected_tips {
            eprintln!("\x1b[2m{tip}\x1b[0m");
        }
        eprintln!("\x1b[2m───────────────────────────────────────────────────────\x1b[0m");
    }

    Ok(())
}

/// Ever tried to send a whole drum kit through a tiny tube? That's what this
/// function does. It takes our beautiful output, squishes it down with Zlib
/// compression, and makes it super easy to send over the network or to an AI
/// that appreciates brevity.
fn compress_output(data: &[u8]) -> Result<Vec<u8>> {
    // Create a Zlib encoder with default compression level.
    let mut encoder = ZlibEncoder::new(Vec::new(), Compression::default());
    // Write all input data to the encoder.
    encoder.write_all(data)?;
    // Finalize the compression and return the resulting byte vector.
    Ok(encoder.finish()?)
}

// --- MCP Helper Functions (only compiled if "mcp" feature is enabled) ---

/// Prints the JSON configuration snippet for adding `st` as an MCP server
/// to Claude Desktop. This helps users easily integrate `st`.
fn print_mcp_config() {
    // Try to get the current executable's path. Fallback to "st" if it fails.
    let exe_path = std::env::current_exe().unwrap_or_else(|_| PathBuf::from("st")); // Graceful fallback.

    // Print the JSON structure, making it easy for users to copy-paste.
    // Using println! for each line for clarity.
    println!("Add this to your Claude Desktop configuration (claude_desktop_config.json):");
    println!(); // Blank line for spacing.
    println!("{{"); // Start of JSON object.
    println!("  \"mcpServers\": {{");
    println!("    \"smart-tree\": {{"); // Server name: "smart-tree".
    println!("      \"command\": \"{}\",", exe_path.display()); // Path to the st executable.
    println!("      \"args\": [\"--mcp\"],"); // Arguments to run st in MCP server mode.
    println!("      \"env\": {{}}"); // Optional environment variables for the server process.
    println!("    }}");
    println!("  }}");
    println!("}}"); // End of JSON object.
    println!();
    // Provide common locations for the Claude Desktop config file.
    println!("Default locations for claude_desktop_config.json:");
    println!("  macOS:   ~/Library/Application Support/Claude/claude_desktop_config.json");
    println!("  Windows: %APPDATA%\\Claude\\claude_desktop_config.json");
    println!("  Linux:   ~/.config/Claude/claude_desktop_config.json");
}

/// Prints a list of available MCP tools that `st` provides.
/// This helps users (or AI) understand what actions can be performed via MCP.
fn print_mcp_tools() {
    println!("🌳 Smart Tree MCP Server - Available Tools (20+) 🌳");
    println!();
    println!("📚 Full documentation: Run 'st --mcp' to start the MCP server");
    println!("💡 Pro tip: Use these tools with Claude Desktop for AI-powered file analysis!");
    println!();
    println!("CORE TOOLS:");
    println!("  • server_info - Get server capabilities and current time");
    println!("  • analyze_directory - Main workhorse with multiple output formats");
    println!("  • quick_tree - Lightning-fast 3-level overview (10x compression)");
    println!("  • project_overview - Comprehensive project analysis");
    println!();
    println!("FILE DISCOVERY:");
    println!("  • find_files - Search with regex, size, date filters");
    println!("  • find_code_files - Find source code by language");
    println!("  • find_config_files - Locate all configuration files");
    println!("  • find_documentation - Find README, docs, licenses");
    println!("  • find_tests - Locate test files across languages");
    println!("  • find_build_files - Find Makefile, Cargo.toml, etc.");
    println!();
    println!("CONTENT SEARCH:");
    println!("  • search_in_files - Powerful content search (like grep)");
    println!("  • find_large_files - Identify space consumers");
    println!("  • find_recent_changes - Files modified in last N days");
    println!("  • find_in_timespan - Files modified in date range");
    println!();
    println!("ANALYSIS:");
    println!("  • get_statistics - Comprehensive directory stats");
    println!("  • get_digest - SHA256 hash for change detection");
    println!("  • directory_size_breakdown - Size by subdirectory");
    println!("  • find_empty_directories - Cleanup opportunities");
    println!("  • find_duplicates - Detect potential duplicate files");
    println!("  • semantic_analysis - Group files by purpose");
    println!();
    println!("ADVANCED:");
    println!("  • compare_directories - Find differences between dirs");
    println!("  • get_git_status - Git-aware directory structure");
    println!("  • analyze_workspace - Multi-project workspace analysis");
    println!();
    println!("SMART EDIT (90% token reduction!):");
    println!("  • smart_edit - Apply multiple AST-based edits efficiently");
    println!("  • get_function_tree - Analyze code structure");
    println!("  • insert_function - Add functions with minimal tokens");
    println!("  • remove_function - Remove functions with dependency awareness");
    println!("  • track_file_operation - Track AI file manipulations (.st folder)");
    println!("  • get_file_history - View operation history for files");
    println!();
    println!("FEEDBACK:");
    println!("  • submit_feedback - Help improve Smart Tree");
    println!("  • request_tool - Request new MCP tools");
    println!("  • check_for_updates - Check for newer versions");
    println!();
    println!("Run 'st --mcp' to start the server and see full parameter details!");
}

/// Show version information with optional update checking
/// This combines the traditional --version output with smart update detection
/// Elvis would love this modern approach! 🕺
async fn show_version_with_updates() -> Result<()> {
    let current_version = env!("CARGO_PKG_VERSION");

    // Always show current version info first
    println!(
        "🌟 Smart Tree v{} - The Gradient Enhancement Release! 🌈",
        current_version
    );
    println!("🔧 Target: {}", std::env::consts::ARCH);
    println!("📦 Repository: {}", env!("CARGO_PKG_REPOSITORY"));
    println!("🎯 Authors: {}", env!("CARGO_PKG_AUTHORS"));
    println!("📝 Description: {}", env!("CARGO_PKG_DESCRIPTION"));

    // Check for updates (but don't fail if update service is unavailable)
    match check_for_updates_cli().await {
        Ok(update_info) => {
            if update_info.is_empty() {
                println!("✅ You're running the latest version! 🎉");
            } else {
                println!();
                println!("{}", update_info);
            }
        }
        Err(e) => {
            // Don't fail the whole command if update check fails
            eprintln!("⚠️  Update check unavailable: {}", e);
            println!("💡 Check https://github.com/8b-is/smart-tree for the latest releases");
        }
    }

    println!();
    println!("🚀 Ready to make your directories beautiful! Try: st --help");
    println!("🎭 Trish from Accounting loves the colorful tree views! 🎨");

    Ok(())
}

/// Handle viewing diffs from the .st folder
async fn handle_view_diffs() -> Result<()> {
    use st::smart_edit_diff::DiffStorage;

    let project_root = std::env::current_dir()?;
    let storage = DiffStorage::new(&project_root)?;

    // List all diffs
    let diffs = storage.list_all_diffs()?;

    if diffs.is_empty() {
        println!("📁 No diffs found in .st folder");
        println!("💡 Smart Edit operations automatically store diffs when files are modified");
        return Ok(());
    }

    println!("📜 Smart Edit Diff History");
    println!("{}", "=".repeat(60));

    // Group diffs by file
    let mut by_file: std::collections::HashMap<String, Vec<(String, u64)>> =
        std::collections::HashMap::new();

    for (file_path, timestamp) in diffs {
        by_file
            .entry(file_path.clone())
            .or_default()
            .push((file_path, timestamp));
    }

    for (file, mut entries) in by_file {
        // Sort by timestamp (newest first)
        entries.sort_by(|a, b| b.1.cmp(&a.1));

        println!("\n📄 {}", file);
        for (_, timestamp) in entries.iter().take(5) {
            let dt = chrono::DateTime::<chrono::Utc>::from_timestamp(*timestamp as i64, 0)
                .unwrap_or_default();
            println!("  • {} ({})", dt.format("%Y-%m-%d %H:%M:%S UTC"), timestamp);
        }

        if entries.len() > 5 {
            println!("  ... and {} more", entries.len() - 5);
        }
    }

    println!("\n💡 Use 'st --cleanup-diffs N' to keep only the last N diffs per file");

    Ok(())
}

/// Handle cleaning up old diffs
async fn handle_cleanup_diffs(keep_count: usize) -> Result<()> {
    use st::smart_edit_diff::DiffStorage;

    let project_root = std::env::current_dir()?;
    let storage = DiffStorage::new(&project_root)?;

    println!(
        "🧹 Cleaning up old diffs, keeping last {} per file...",
        keep_count
    );

    let removed = storage.cleanup_old_diffs(keep_count)?;

    if removed == 0 {
        println!("✨ No diffs needed cleanup");
    } else {
        println!("✅ Removed {} old diff files", removed);
    }

    Ok(())
}

/// Check for updates from our feedback API (CLI version)
/// Returns update message if available, empty string if up-to-date
async fn check_for_updates_cli() -> Result<String> {
    let current_version = env!("CARGO_PKG_VERSION");

    // Skip update check if explicitly disabled
    if std::env::var("SMART_TREE_NO_UPDATE_CHECK").is_ok() {
        return Ok(String::new());
    }

    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(2)) // Global timeout for all operations
        .connect_timeout(std::time::Duration::from_secs(1)) // Quick connect timeout
        .build()?;

    let api_url =
        std::env::var("SMART_TREE_FEEDBACK_API").unwrap_or_else(|_| "https://f.8b.is".to_string());

    let check_url = format!("{}/version/check/{}", api_url, current_version);

    let response = client
        .get(&check_url)
        .send()
        .await
        .map_err(|e| anyhow::anyhow!("Network error: {}", e))?;

    if !response.status().is_success() {
        return Err(anyhow::anyhow!("Service returned: {}", response.status()));
    }

    let update_info: serde_json::Value = response
        .json()
        .await
        .map_err(|e| anyhow::anyhow!("Failed to parse response: {}", e))?;

    if !update_info["update_available"].as_bool().unwrap_or(false) {
        return Ok(String::new()); // Up to date
    }

    // Format update message for CLI
    let latest_version = update_info["latest_version"].as_str().unwrap_or("unknown");
    let release_notes = &update_info["release_notes"];

    let highlights = release_notes["highlights"]
        .as_array()
        .map(|arr| {
            arr.iter()
                .filter_map(|v| v.as_str())
                .map(|s| format!("  • {}", s))
                .collect::<Vec<_>>()
                .join("\n")
        })
        .unwrap_or_default();

    let ai_benefits = release_notes["ai_benefits"]
        .as_array()
        .map(|arr| {
            arr.iter()
                .filter_map(|v| v.as_str())
                .map(|s| format!("  • {}", s))
                .collect::<Vec<_>>()
                .join("\n")
        })
        .unwrap_or_default();

    let mut message = format!(
        "🚀 \x1b[1;32mNew Version Available!\x1b[0m\n\n\
        📊 Current: v{} → Latest: \x1b[1;36mv{}\x1b[0m\n\n\
        🎯 \x1b[1m{}\x1b[0m\n",
        current_version,
        latest_version,
        release_notes["title"].as_str().unwrap_or("New Release")
    );

    if !highlights.is_empty() {
        message.push_str(&format!("\n\x1b[1mWhat's New:\x1b[0m\n{}\n", highlights));
    }

    if !ai_benefits.is_empty() {
        message.push_str(&format!("\n\x1b[1mAI Benefits:\x1b[0m\n{}\n", ai_benefits));
    }

    // Add update instructions
    message.push_str(
        "\n\x1b[1mUpdate Instructions:\x1b[0m\n\
        • Cargo: \x1b[36mcargo install st --force\x1b[0m\n\
        • GitHub: Download from https://github.com/8b-is/smart-tree/releases\n\
        • Check: \x1b[36mst --version\x1b[0m (after update)\n",
    );

    Ok(message)
}

/// run_mcp_server is an async function that starts the MCP server.
/// When --mcp is passed, we start a server that communicates via stdio.
async fn run_mcp_server() -> Result<()> {
    // Import MCP server components. These are only available if "mcp" feature is enabled.
    use st::mcp::{load_config, McpServer};

    // Load MCP server-specific configuration (e.g., allowed paths, cache settings).
    let mcp_config = load_config().unwrap_or_default(); // Load or use defaults.
    let server = McpServer::new(mcp_config);

    // Run the MCP server directly - no need for nested runtime!
    // `run_stdio` handles communication over stdin/stdout.
    server.run_stdio().await
}

/// Run the Smart Tree Terminal Interface - Your coding companion!
async fn run_terminal() -> Result<()> {
    // Create and run the terminal interface
    let mut terminal = SmartTreeTerminal::new()?;
    terminal.run().await
}
