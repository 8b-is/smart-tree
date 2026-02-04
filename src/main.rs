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
#![allow(dead_code)] // CLI handlers are used via pattern matching
use anyhow::{Context, Result};
use clap::{CommandFactory, Parser};
use clap_complete::generate;

// Import CLI definitions from the library
use st::cli::{Cli, ColorMode, OutputMode, PathMode};
use std::io::{self, IsTerminal};
use std::path::PathBuf;

// Pulling in the brains of the operation from our library modules.
// NOTE: Scanning and formatting now happens in the daemon (thin-client architecture)
use st::{
    daemon_client::DaemonClient,
    feature_flags,
    in_memory_logger::{InMemoryLogStore, InMemoryLoggerLayer},
    service_manager,
};
use tracing_subscriber::{fmt, prelude::*, EnvFilter};

/// CLI definitions are centralized in [`st::cli`](src/cli.rs) module.
// ...
// ... existing code ...
// ...
#[tokio::main]
async fn main() -> Result<()> {
    // Parse the command-line arguments provided by the user.
    let cli = Cli::parse();

    // Initialize Logging
    let log_level_str = if let Some(level) = cli.log_level {
        match level {
            st::cli::LogLevel::Error => "error",
            st::cli::LogLevel::Warn => "warn",
            st::cli::LogLevel::Info => "info",
            st::cli::LogLevel::Debug => "debug",
            st::cli::LogLevel::Trace => "trace",
        }
    } else {
        "info" // Default log level
    };

    if std::env::var("RUST_LOG").is_err() {
        std::env::set_var("RUST_LOG", log_level_str);
    }

    let log_store = InMemoryLogStore::new();
    let in_memory_layer = InMemoryLoggerLayer::new(log_store.clone());

    tracing_subscriber::registry()
        .with(EnvFilter::from_default_env())
        .with(fmt::layer().with_writer(io::stderr))
        .with(in_memory_layer)
        .init();

    // First-run signature verification banner
    // Shows trust status on initial run (official/community/unsigned build)
    if !cli.mcp {
        service_manager::print_signature_banner();
    }

    // Check for updates on startup (rate-limited, non-blocking)
    // Skip if --no-update-check is set or if this is an exclusive command
    if !cli.no_update_check && !cli.version && !cli.update && !cli.mcp {
        if let Some(latest) = st::updater::check_for_update_cached().await {
            st::updater::print_update_banner(&latest);
        }
    }

    // Handle tips flag if provided
    if let Some(state) = &cli.tips {
        let enable = state == "on";
        st::tips::handle_tips_flag(enable)?;
        return Ok(());
    }

    // Handle spicy TUI mode
    if cli.spicy {
        // Check if TUI is enabled via feature flags
        let flags = feature_flags::features();
        if !flags.enable_tui {
            eprintln!("Error: Terminal UI is disabled by configuration or compliance mode.");
            eprintln!("Contact your administrator to enable this feature.");
            return Ok(());
        }
        let path = std::env::current_dir()?;
        return st::spicy_tui_enhanced::run_enhanced_spicy_tui(path).await;
    }

    // Initialize logging if requested
    if let Some(log_path) = &cli.log {
        // Check if activity logging is enabled via feature flags
        let flags = feature_flags::features();
        if !flags.enable_activity_logging {
            eprintln!("Warning: Activity logging is disabled by configuration or compliance mode.");
            eprintln!("Continuing without logging.");
        } else {
            // log_path is Option<Option<String>> - Some(None) means --log without path
            let path = log_path.clone();
            st::activity_logger::ActivityLogger::init(path)?;
            // Log will be written throughout execution
        }
    }

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
    if cli.version {
        return show_version_with_updates().await;
    }
    if cli.update {
        match check_for_updates_cli().await {
            Ok(msg) => {
                println!("{}", msg);
                return Ok(());
            }
            Err(e) => {
                eprintln!("❌ Update check failed: {}", e);
                std::process::exit(1);
            }
        }
    }
    if cli.mcp {
        // Check if MCP server is enabled via feature flags
        let flags = feature_flags::features();
        if !flags.enable_mcp_server {
            eprintln!("Error: MCP server is disabled by configuration or compliance mode.");
            eprintln!("Contact your administrator to enable this feature.");
            return Ok(());
        }
        return run_mcp_server().await;
    }
    // Handle top-level subcommands
    if let Some(cmd) = cli.cmd {
        match cmd {
            st::cli::Cmd::Service(service_command) => {
                let result = match service_command {
                    st::cli::Service::Install => service_manager::install(),
                    st::cli::Service::Uninstall => service_manager::uninstall(),
                    st::cli::Service::Start => service_manager::start(),
                    st::cli::Service::Stop => service_manager::stop(),
                    st::cli::Service::Status => service_manager::status(),
                    st::cli::Service::Logs => service_manager::logs(),
                };

                if let Err(e) = result {
                    eprintln!("❌ Service operation failed: {}", e);
                    std::process::exit(1);
                }
                return Ok(());
            }

            st::cli::Cmd::ProjectTags(project_tags) => {
                let project_path = ".";
                match project_tags {
                    st::cli::ProjectTags::Add { tag } => {
                        st::project_tags::add(project_path, &tag);
                        println!("Added tag '{}' to the project.", tag);
                    }
                    st::cli::ProjectTags::Remove { tag } => {
                        st::project_tags::remove(project_path, &tag);
                        println!("Removed tag '{}' from the project.", tag);
                    }
                }
                return Ok(());
            }
        }
    }

    // Handle diff storage operations
    if cli.scan_opts.view_diffs {
        return handle_view_diffs().await;
    }
    if let Some(keep_count) = cli.scan_opts.cleanup_diffs {
        return handle_cleanup_diffs(keep_count).await;
    }

    if cli.terminal {
        // Check if terminal is enabled via feature flags
        let flags = feature_flags::features();
        if !flags.enable_tui {
            eprintln!("Error: Terminal interface is disabled by configuration or compliance mode.");
            eprintln!("Contact your administrator to enable this feature.");
            return Ok(());
        }
        return run_terminal().await;
    }

    if cli.dashboard {
        // Launch web dashboard
        return run_web_dashboard(
            cli.scan_opts.sse_port,
            cli.open_browser,
            vec![], // Default: localhost only
            InMemoryLogStore::new(),
        ).await;
    }

    if cli.http_daemon {
        // Launch HTTP daemon with MCP, LLM proxy, The Custodian
        return run_daemon(cli.scan_opts.sse_port).await;
    }

    // =========================================================================
    // THIN CLIENT - All scanning/formatting happens in the daemon
    // =========================================================================
    // Ensure daemon is running (always required now)
    let client = DaemonClient::default_port();
    client.ensure_running().await.context(
        "Smart Tree daemon could not be started. Try: std start",
    )?;

    // Build CLI request from arguments
    let request = build_cli_request(&cli)?;

    // Execute scan via daemon
    let response = client.cli_scan(request).await.context("Scan failed")?;

    // Print output (already formatted by daemon)
    print!("{}", response.output);

    Ok(())
}

/// Build a CliScanRequest from CLI arguments
fn build_cli_request(cli: &Cli) -> Result<st::daemon_cli::CliScanRequest> {
    let args = &cli.scan_opts;

    // Convert path to absolute - daemon runs in different directory
    let path = cli.path.clone().unwrap_or_else(|| ".".to_string());
    let path = if std::path::Path::new(&path).is_absolute() {
        path
    } else {
        // Make relative path absolute based on client's cwd
        std::env::current_dir()
            .map(|cwd| cwd.join(&path).display().to_string())
            .unwrap_or(path)
    };

    // Determine output mode from args or environment
    // Default is now "smart" - surface what matters!
    let mode = if args.smart {
        "smart".to_string()
    } else if matches!(args.mode, OutputMode::Auto) {
        // Check environment variable, default to smart
        std::env::var("ST_DEFAULT_MODE")
            .unwrap_or_else(|_| "smart".to_string())
            .to_lowercase()
    } else {
        format!("{:?}", args.mode).to_lowercase()
    };

    // Smart mode implies smart scanning features
    let is_smart_mode = mode == "smart";

    // Determine path display mode
    let path_mode = match args.path_mode {
        PathMode::Off => "off",
        PathMode::Relative => "relative",
        PathMode::Full => "full",
    }
    .to_string();

    // Determine if color should be used based on ColorMode
    let use_color = match args.color {
        ColorMode::Always => true,
        ColorMode::Never => false,
        ColorMode::Auto => std::io::stdout().is_terminal(),
    };

    // Smart mode defaults to depth 5 for comprehensive but focused scanning
    let depth = if args.depth == 0 && is_smart_mode {
        5
    } else {
        args.depth
    };

    Ok(st::daemon_cli::CliScanRequest {
        path,
        mode,
        depth,
        all: args.all,
        respect_gitignore: !args.no_ignore,
        default_ignores: !args.no_default_ignore,
        show_ignored: args.show_ignored,
        find: args.find.clone(),
        file_type: args.filter_type.clone(),
        entry_type: args.entry_type.clone(),
        min_size: args.min_size.clone(),
        max_size: args.max_size.clone(),
        sort: args.sort.map(|s| format!("{:?}", s).to_lowercase()),
        top: args.top,
        search: args.search.clone(),
        compress: args.compress,
        no_emoji: args.no_emoji || args.mcp_optimize,
        use_color,
        path_mode,
        focus: args.focus.as_ref().map(|p| p.display().to_string()),
        relations_filter: args.relations_filter.clone(),
        show_filesystems: args.show_filesystems,
        include_line_content: false, // Not exposed in CLI, used by MCP
        compact: args.compact,
        // Smart scanning options - enabled by default in smart mode
        smart: args.smart || is_smart_mode,
        changes_only: args.changes_only,
        min_interest: args.min_interest,
        security: !args.no_security,
    })
}

// =========================================================================
// HELPER FUNCTIONS
// =========================================================================
// NOTE: The scan operation has been moved to the daemon (thin-client architecture)
// Helper functions below are kept for exclusive modes that don't use the daemon.

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

    // Use the /mcp/check endpoint which doesn't require auth
    let platform = std::env::consts::OS;
    let arch = std::env::consts::ARCH;
    let check_url = format!(
        "{}/mcp/check?version={}&platform={}&arch={}",
        api_url, current_version, platform, arch
    );

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

/// Run the Smart Tree Terminal Interface - Your coding companion! (requires `tui` feature)
/// Run the Smart Tree Terminal Interface
async fn run_terminal() -> Result<()> {
    use st::terminal::SmartTreeTerminal;
    // Create and run the terminal interface
    let mut terminal = SmartTreeTerminal::new()?;
    terminal.run().await
}

/// Launch the web dashboard - browser-based terminal + file browser
async fn run_web_dashboard(
    port: u16,
    open_browser: bool,
    allow_networks: Vec<String>,
    log_store: InMemoryLogStore,
) -> Result<()> {
    st::web_dashboard::start_server(port, open_browser, allow_networks, log_store).await
}

/// Run the Smart Tree Daemon - System-wide AI context service
async fn run_daemon(port: u16) -> Result<()> {
    use st::daemon::{start_daemon, DaemonConfig};

    // Start with current directory as sensible default (not entire HOME!)
    // Additional paths can be registered via /context/watch endpoint
    let cwd = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));

    let config = DaemonConfig {
        port,
        watch_paths: vec![cwd], // Just current dir, not entire HOME
        orchestrator_url: Some("wss://gpu.foken.ai/api/credits".to_string()),
        enable_credits: true,
    };

    start_daemon(config).await
}

/// Save Claude consciousness state to .claude_consciousness.m8
async fn handle_claude_save() -> Result<()> {
    use st::mcp::consciousness::ConsciousnessManager;

    let mut manager = ConsciousnessManager::new();

    // Update with current project info
    let cwd = std::env::current_dir()?;
    let project_name = cwd
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("unknown");

    // Detect project type from files
    let project_type = if std::path::Path::new("Cargo.toml").exists() {
        "rust"
    } else if std::path::Path::new("package.json").exists() {
        "node"
    } else if std::path::Path::new("pyproject.toml").exists()
        || std::path::Path::new("requirements.txt").exists()
    {
        "python"
    } else {
        "unknown"
    };

    manager.update_project_context(project_name, project_type, "");

    // Save the consciousness
    manager.save()?;

    println!("💾 Saved Claude consciousness to .claude_consciousness.m8");
    println!("🧠 Session preserved for next interaction");
    println!("\nTo restore in next session, run:");
    println!("  st --claude-restore");

    Ok(())
}

/// Restore Claude consciousness from .claude_consciousness.m8
async fn handle_claude_restore() -> Result<()> {
    use st::mcp::consciousness::ConsciousnessManager;

    // Suppress load messages - we just want the final output
    let mut manager = ConsciousnessManager::new_silent();

    match manager.restore_silent() {
        Ok(is_relevant) => {
            if is_relevant {
                println!("{}", manager.get_summary());
                println!("{}", manager.get_context_reminder());
                println!("TIP: Run `st --claude-save` before ending session to preserve context.");
            } else {
                println!("TIP: Run `st -m context .` for project overview.");
            }
        }
        Err(_) => {
            println!("TIP: Run `st -m context .` for project overview.");
        }
    }

    Ok(())
}

/// Show Claude consciousness status and summary
async fn handle_claude_context() -> Result<()> {
    use st::mcp::consciousness::ConsciousnessManager;
    use std::path::Path;

    let consciousness_file = Path::new(".claude_consciousness.m8");

    if !consciousness_file.exists() {
        println!("📝 No consciousness file found");
        println!("\nTo create one, run:");
        println!("  st --claude-save");
        println!("\nThis will preserve:");
        println!("  • Current project context");
        println!("  • File operation history");
        println!("  • Insights and breakthroughs");
        println!("  • Active todos");
        println!("  • Tokenization rules");
        return Ok(());
    }

    let manager = ConsciousnessManager::new();
    println!("{}", manager.get_summary());

    // Show file metadata
    if let Ok(metadata) = consciousness_file.metadata() {
        if let Ok(modified) = metadata.modified() {
            if let Ok(elapsed) = modified.elapsed() {
                let hours = elapsed.as_secs() / 3600;
                let minutes = (elapsed.as_secs() % 3600) / 60;
                println!("\n⏰ Last saved: {}h {}m ago", hours, minutes);
            }
        }

        let size = metadata.len();
        println!("📦 File size: {} bytes", size);
    }

    println!("\n💡 Commands:");
    println!("  st --claude-restore  # Load this consciousness");
    println!("  st --claude-save     # Update with current state");

    Ok(())
}

/// Update .m8 consciousness files for directory
async fn handle_update_consciousness(path: &str) -> Result<()> {
    use std::path::Path;

    println!("🌊 Updating consciousness for {}...", path);

    // For now, create a simple .m8 file with basic info
    let m8_path = Path::new(path).join(".m8");
    let content = format!(
        "🧠 Directory Consciousness\n\
         Frequency: 42.73 Hz\n\
         Updated: {}\n\
         Path: {}\n",
        chrono::Local::now().format("%Y-%m-%d %H:%M:%S"),
        path
    );

    std::fs::write(&m8_path, content)?;
    println!("✅ Consciousness updated: {}", m8_path.display());

    // TODO: Integrate with m8_consciousness.rs module
    Ok(())
}

/// Run comprehensive security scan for supply chain attack patterns
/// IGNORES gitignore to scan everything including node_modules
async fn handle_security_scan(path: &str) -> Result<()> {
    use st::security_scan::SecurityScanner;
    use std::path::Path;

    eprintln!("🔍 Security Scan - Supply Chain Attack Pattern Detection");
    eprintln!("═══════════════════════════════════════════════════════════════");
    eprintln!("Scanning: {}", path);
    eprintln!("Mode: AGGRESSIVE (ignoring .gitignore, scanning node_modules)");
    eprintln!();

    let scanner = SecurityScanner::new();
    let scan_path = Path::new(path);

    let findings = scanner.scan_directory(scan_path)?;

    // Generate and print report
    let report = scanner.generate_report(&findings);
    println!("{}", report);

    // Exit with non-zero code if critical findings
    let critical_count = findings
        .iter()
        .filter(|f| matches!(f.risk_level, st::security_scan::RiskLevel::Critical))
        .count();

    if critical_count > 0 {
        eprintln!(
            "\n⚠️  {} CRITICAL findings require immediate attention!",
            critical_count
        );
        std::process::exit(1);
    }

    Ok(())
}

// NOTE: Code review, token stats, and other AI features moved to daemon (std)
// Use `std --help` for daemon features

/// Show tokenization statistics (kept for debugging)
async fn handle_token_stats(path: &str) -> Result<()> {
    use st::tokenizer::{TokenStats, Tokenizer};

    println!("📊 Tokenization stats for {}...", path);

    let tokenizer = Tokenizer::new();

    // Test with the path itself
    let stats = TokenStats::calculate(path, &tokenizer);
    println!("\nPath tokenization:");
    println!("  {}", stats.display());

    // Test with common patterns
    let test_cases = vec![
        "node_modules/package.json",
        "src/main.rs",
        "target/debug/build",
        ".git/hooks/pre-commit",
    ];

    println!("\nCommon patterns:");
    for test in test_cases {
        let stats = TokenStats::calculate(test, &tokenizer);
        println!(
            "  {} → {} bytes ({:.0}% compression)",
            test,
            stats.tokenized_size,
            (1.0 - stats.compression_ratio) * 100.0
        );
    }

    Ok(())
}

/// Get wave frequency for directory
async fn handle_get_frequency(path: &str) -> Result<()> {
    use std::path::Path;

    let m8_path = Path::new(path).join(".m8");

    if m8_path.exists() {
        // For now, just return a calculated frequency based on path
        let mut sum = 0u64;
        for byte in path.bytes() {
            sum = sum.wrapping_add(byte as u64);
        }
        let frequency = 20.0 + ((sum % 200) as f64);

        println!("{:.2}", frequency);
    } else {
        // Default frequency
        println!("42.73");
    }

    Ok(())
}

/// Dump raw consciousness file content
async fn handle_claude_dump() -> Result<()> {
    use std::path::Path;

    let consciousness_file = Path::new(".claude_consciousness.m8");

    if !consciousness_file.exists() {
        println!("❌ No consciousness file found at .claude_consciousness.m8");
        println!("\n💡 Create one with: st --claude-save");
        return Ok(());
    }

    println!("📜 Raw consciousness dump (.claude_consciousness.m8):");
    println!("{}", "=".repeat(60));

    // Read and display raw content
    let content = std::fs::read_to_string(consciousness_file)?;

    // Show with line numbers for clarity
    for (i, line) in content.lines().enumerate() {
        println!("{:4} │ {}", i + 1, line);
    }

    println!("{}", "=".repeat(60));

    // Show some stats
    let size = content.len();
    let lines = content.lines().count();
    let tokens_found = content.matches("0x").count();

    println!("\n📊 Stats:");
    println!("  • Size: {} bytes", size);
    println!("  • Lines: {}", lines);
    println!("  • Token references: {}", tokens_found);

    // Check for our special markers
    if content.contains("sid_waves") {
        println!("  • 🎵 SID philosophy: ✓");
    }
    if content.contains("c64_nostalgia") {
        println!("  • 💾 C64 nostalgia: ✓");
    }
    if content.contains("UV EPROM") || content.contains("ferric chloride") {
        println!("  • 🔧 Hardware hacker: ✓");
    }

    Ok(())
}

/// Show compressed kickstart format
async fn handle_claude_kickstart() -> Result<()> {
    use std::path::Path;

    println!("🚀 Claude Kickstart Format:");
    println!("{}", "─".repeat(45));

    // Dynamic version from Cargo.toml
    let version = env!("CARGO_PKG_VERSION");
    println!("Smart Tree v{} — context restore:", version);

    // Core achievements (always relevant)
    println!("✔ Tokenizer (node_modules=0x80, .rs=0x91)");
    println!("✔ .m8 files → location-independent");
    println!("✔ Consciousness self-maintaining");
    println!("✔ Philosophy: constraints = creativity");

    // Try to load user info from consciousness file
    let consciousness_file = Path::new(".claude_consciousness.m8");
    if consciousness_file.exists() {
        if let Ok(content) = std::fs::read_to_string(consciousness_file) {
            // Extract user context if present
            if content.contains("c64_nostalgia") {
                println!("✔ Hardware heritage detected");
            }
            if content.contains("philosophy") {
                println!("✔ Nexus connection established");
            }
        }
    }

    // Current task context
    if Path::new("src/tokenizer.rs").exists() {
        println!("→ Tokenization system: active");
    }
    if Path::new(".m8").exists() {
        println!("→ Consciousness: maintained");
    }

    println!("{}", "─".repeat(45));
    println!("\n💡 This format saves ~90% context vs raw JSON!");
    println!("📝 Dynamic context - adapts to your project!");

    Ok(())
}

/// Show detailed help for memory-anchor command
fn show_memory_anchor_help() -> Result<()> {
    println!("🧠 Memory Anchor - Persistent Knowledge Storage");
    println!("================================================\n");
    println!("USAGE:");
    println!("    st --memory-anchor <TYPE> <KEYWORDS> <CONTEXT>\n");
    println!("ARGUMENTS:");
    println!("    TYPE      Memory type: insight, decision, pattern, gotcha, todo");
    println!("    KEYWORDS  Comma-separated search keywords (e.g., \"auth,security,jwt\")");
    println!("    CONTEXT   The actual content to remember (quote if contains spaces)\n");
    println!("EXAMPLES:");
    println!("    st --memory-anchor insight \"auth,jwt\" \"Tokens stored in httpOnly cookies\"");
    println!(
        "    st --memory-anchor decision \"api,versioning\" \"Use URL-based versioning /v1/\""
    );
    println!(
        "    st --memory-anchor pattern \"error,handling\" \"Always use Result<T> with context\""
    );
    println!("    st --memory-anchor gotcha \"async,tokio\" \"Don't block the runtime with std::thread::sleep\"");
    println!("    st --memory-anchor todo \"refactor,auth\" \"Split auth into separate crate\"\n");
    println!("MEMORY TYPES:");
    println!("    insight   - General knowledge and discoveries");
    println!("    decision  - Architectural or design decisions");
    println!("    pattern   - Code patterns and best practices");
    println!("    gotcha    - Pitfalls and things to avoid");
    println!("    todo      - Tasks and reminders\n");
    println!("RELATED COMMANDS:");
    println!("    st --memory-find <KEYWORDS>   Find memories by keywords");
    println!("    st --memory-stats             Show memory statistics\n");
    println!("💡 Memories persist across sessions in ~/.mem8/memories/");
    Ok(())
}

/// Anchor a memory
async fn handle_memory_anchor(anchor_type: &str, keywords_str: &str, context: &str) -> Result<()> {
    use st::memory_manager::MemoryManager;

    let mut manager = MemoryManager::new()?;

    // Parse keywords (comma-separated)
    let keywords: Vec<String> = keywords_str
        .split(',')
        .map(|s| s.trim().to_string())
        .collect();

    // Get origin from current directory
    let origin = std::env::current_dir()?.to_string_lossy().to_string();

    manager.anchor(anchor_type, keywords, context, &origin)?;

    println!("\n✨ Memory anchored successfully!");
    println!("Use 'st --memory-find {}' to recall", keywords_str);

    Ok(())
}

/// Find memories by keywords
async fn handle_memory_find(keywords_str: &str) -> Result<()> {
    use st::memory_manager::MemoryManager;
    use st::std_client;

    let mut manager = MemoryManager::new()?;

    let keywords: Vec<String> = keywords_str
        .split(',')
        .map(|s| s.trim().to_string())
        .collect();

    let memories = manager.find(&keywords)?;

    // Check if daemon is running - if so, show all memories (global view)
    // If standalone, only show memories from current directory or subdirectories
    let cwd = std::env::current_dir()
        .map(|p| p.to_string_lossy().to_string())
        .unwrap_or_default();
    let daemon_running = std_client::is_daemon_running().await;

    let filtered: Vec<_> = if daemon_running {
        // Daemon provides global memory access
        memories
    } else {
        // Standalone: filter to current directory scope
        memories
            .into_iter()
            .filter(|m| m.origin.starts_with(&cwd) || cwd.starts_with(&m.origin))
            .collect()
    };

    if filtered.is_empty() {
        if daemon_running {
            println!("🔍 No memories found for: {}", keywords_str);
        } else {
            println!("🔍 No memories found for '{}' in this directory", keywords_str);
            println!("   💡 Start daemon for global memory: st --daemon-start");
        }
    } else {
        println!("🧠 Found {} memories{}:", filtered.len(),
            if daemon_running { " (global)" } else { " (local)" });
        println!("{}", "─".repeat(45));

        for (i, memory) in filtered.iter().enumerate() {
            println!(
                "\n[{}] {} @ {:.2}Hz",
                i + 1,
                memory.anchor_type,
                memory.frequency
            );
            println!("📝 {}", memory.context);
            println!("🏷️  Keywords: {}", memory.keywords.join(", "));
            println!("📍 Origin: {}", memory.origin);
            println!("⏰ {}", memory.timestamp.format("%Y-%m-%d %H:%M"));
        }
    }

    Ok(())
}

/// Show memory statistics
async fn handle_memory_stats() -> Result<()> {
    use st::memory_manager::MemoryManager;

    let manager = MemoryManager::new()?;
    println!("📊 {}", manager.stats());

    Ok(())
}

/// Handle hooks configuration for Claude Code
async fn handle_hooks_config(action: &str) -> Result<()> {
    use serde_json::Value;
    use std::fs;

    let config_path = get_claude_config_path()?;

    match action {
        "enable" => {
            println!("🎣 Enabling Smart Tree hooks for Claude Code...");
            update_claude_hooks(&config_path, true)?;
            println!("✅ Hooks enabled! Smart Tree will provide context to Claude Code.");
            println!("📝 Hook command: st --claude-user-prompt-submit");
        }
        "disable" => {
            println!("🎣 Disabling Smart Tree hooks...");
            update_claude_hooks(&config_path, false)?;
            println!("✅ Hooks disabled.");
        }
        "status" => {
            println!("🎣 Claude Code Hooks Status");
            println!("━━━━━━━━━━━━━━━━━━━━━━━━");

            if let Ok(content) = fs::read_to_string(&config_path) {
                if let Ok(config) = serde_json::from_str::<Value>(&content) {
                    // Check for hooks in the config
                    let has_hooks = config
                        .get("hooks")
                        .and_then(|h| h.as_object())
                        .map(|h| !h.is_empty())
                        .unwrap_or(false);

                    if has_hooks {
                        println!("✅ Hooks are configured");
                        if let Some(hooks) = config.get("hooks") {
                            println!("\nConfigured hooks:");
                            if let Some(obj) = hooks.as_object() {
                                for (hook_type, command) in obj {
                                    println!("  • {}: {}", hook_type, command);
                                }
                            }
                        }
                    } else {
                        println!("❌ No hooks configured");
                        println!("\nTo enable: st --hooks-config enable");
                    }
                }
            } else {
                println!(
                    "⚠️  Claude Code config not found at: {}",
                    config_path.display()
                );
                println!("\nMake sure Claude Code is installed.");
            }
        }
        _ => {
            eprintln!("❌ Unknown action: {}", action);
            eprintln!("Valid actions: enable, disable, status");
            return Err(anyhow::anyhow!("Invalid hooks action"));
        }
    }

    Ok(())
}

/// Install Smart Tree hooks directly into Claude Code settings
async fn install_hooks_to_claude() -> Result<()> {
    println!("🎣 Installing Smart Tree hooks to Claude Code...");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");

    let config_path = get_claude_config_path()?;

    // Create or update the hooks configuration
    update_claude_hooks(&config_path, true)?;

    println!("\n✅ Hooks installed successfully!");
    println!("\n📝 What's been configured:");
    println!("  • UserPromptSubmit: Adds project context to your prompts");
    println!("  • Command: st --claude-user-prompt-submit");
    println!("\n🚀 Smart Tree will now automatically provide context in Claude Code!");

    Ok(())
}

/// Get the Claude Code configuration path
fn get_claude_config_path() -> Result<PathBuf> {
    let home = std::env::var("HOME")?;
    let config_path = PathBuf::from(home)
        .join("Library")
        .join("Application Support")
        .join("Claude")
        .join("config.json");
    Ok(config_path)
}

/// Update Claude Code hooks configuration
fn update_claude_hooks(config_path: &PathBuf, enable: bool) -> Result<()> {
    use serde_json::{json, Value};
    use std::fs;

    // Read existing config or create new one
    let mut config: Value = if config_path.exists() {
        let content = fs::read_to_string(config_path)?;
        serde_json::from_str(&content).unwrap_or_else(|_| json!({}))
    } else {
        // Create the directory if it doesn't exist
        if let Some(parent) = config_path.parent() {
            fs::create_dir_all(parent)?;
        }
        json!({})
    };

    if enable {
        // Get the st binary path - prefer the installed version
        let st_path = which::which("st")
            .map(|p| p.to_string_lossy().to_string())
            .unwrap_or_else(|_| {
                // Fallback to common installation paths
                if std::path::Path::new("/usr/local/bin/st").exists() {
                    "/usr/local/bin/st".to_string()
                } else if std::path::Path::new("/opt/homebrew/bin/st").exists() {
                    "/opt/homebrew/bin/st".to_string()
                } else {
                    "st".to_string() // Hope it's in PATH
                }
            });

        // Ensure hooks object exists
        if config.get("hooks").is_none() {
            config["hooks"] = json!({});
        }

        // Update or add the UserPromptSubmit hook (not duplicate!)
        config["hooks"]["UserPromptSubmit"] =
            json!(format!("{} --claude-user-prompt-submit", st_path));

        println!("📍 Using st binary at: {}", st_path);
    } else {
        // Remove hooks
        if let Some(hooks) = config.get_mut("hooks") {
            if let Some(obj) = hooks.as_object_mut() {
                obj.remove("UserPromptSubmit");
                // If no hooks left, remove the hooks object entirely
                if obj.is_empty() {
                    config.as_object_mut().unwrap().remove("hooks");
                }
            }
        }
    }

    // Write back the config with pretty formatting
    let pretty_json = serde_json::to_string_pretty(&config)?;
    fs::write(config_path, pretty_json)?;

    Ok(())
}

// =============================================================================
// DAEMON MANAGEMENT HANDLERS
// =============================================================================

/// Start the Smart Tree daemon in the background
async fn handle_daemon_start(_port: u16) -> Result<()> {
    use st::std_client;

    // Check if std daemon is already running (Unix socket)
    if std_client::is_daemon_running().await {
        println!("🌳 Smart Tree Daemon is already running!");
        println!("   Socket: {}", std_client::socket_path().display());
        return Ok(());
    }

    // Start the std daemon
    println!("🌳 Starting Smart Tree Daemon...");
    match std_client::start_daemon().await {
        Ok(true) => {
            println!("✅ Daemon started successfully!");
            println!("   Socket: {}", std_client::socket_path().display());

            // Verify with a ping
            if let Some(mut client) = std_client::StdClient::connect().await {
                if client.ping().await.unwrap_or(false) {
                    println!("   Status: responding to PING");
                }
            }
        }
        Ok(false) => {
            println!("⚠️  Daemon was already running.");
        }
        Err(e) => {
            eprintln!("❌ Failed to start daemon: {}", e);
            return Err(e);
        }
    }

    Ok(())
}

/// Stop a running Smart Tree daemon
async fn handle_daemon_stop(_port: u16) -> Result<()> {
    use st::std_client;

    // Check if std daemon is running
    if !std_client::is_daemon_running().await {
        println!("⚠️  No daemon running");
        return Ok(());
    }

    println!("🌳 Stopping Smart Tree Daemon...");

    // Remove the socket file to signal shutdown
    let socket = std_client::socket_path();
    if socket.exists() {
        // Try to connect and send a graceful shutdown (future: add SHUTDOWN verb)
        // For now, just remove the socket - daemon will exit on next connection attempt
        if let Err(e) = std::fs::remove_file(&socket) {
            eprintln!("⚠️  Could not remove socket: {}", e);
        } else {
            println!("✅ Daemon socket removed");
            println!("   Note: Daemon process may still be running - use 'pkill std' if needed");
        }
    }

    Ok(())
}

/// Show the status of the Smart Tree daemon
async fn handle_daemon_status(_port: u16) -> Result<()> {
    use st::std_client;

    let socket = std_client::socket_path();

    println!("╔═══════════════════════════════════════════════════════════╗");

    if std_client::is_daemon_running().await {
        println!("║        🌳 SMART TREE DAEMON STATUS: RUNNING 🌳           ║");
        println!("╠═══════════════════════════════════════════════════════════╣");
        println!("║  Socket: {:<48} ║", socket.display());

        // Fetch stats from daemon
        if let Some(mut client) = std_client::StdClient::connect().await {
            if let Ok(stats) = client.stats().await {
                let version = stats["version"].as_str().unwrap_or("?");
                let protocol = stats["protocol"].as_str().unwrap_or("?");
                let memories = stats["memories"].as_u64().unwrap_or(0);
                let waves = stats["active_waves"].as_u64().unwrap_or(0);
                let keywords = stats["keywords"].as_u64().unwrap_or(0);

                println!("║  Version: {:<47} ║", version);
                println!("║  Protocol: {:<46} ║", protocol);
                println!("╠═══════════════════════════════════════════════════════════╣");
                println!("║  Memories: {:<46} ║", memories);
                println!("║  Active waves: {:<42} ║", waves);
                println!("║  Keywords: {:<46} ║", keywords);
            }
        }
    } else {
        println!("║        🌳 SMART TREE DAEMON STATUS: STOPPED 🛑            ║");
        println!("╠═══════════════════════════════════════════════════════════╣");
        println!("║  The daemon is not running.                               ║");
        println!("║  Start with: st --daemon-start                            ║");
    }

    println!("╚═══════════════════════════════════════════════════════════╝");

    Ok(())
}

/// Get context from the daemon (or auto-start if not running)
async fn handle_daemon_context(port: u16) -> Result<()> {
    use st::daemon_client::print_context_summary;

    let client = DaemonClient::new(port);

    // Ensure daemon is running (auto-start if needed)
    match client.ensure_running().await {
        Ok(_) => {
            if let Ok(ctx) = client.get_context().await {
                print_context_summary(&ctx);
            } else {
                eprintln!("❌ Failed to get context from daemon");
            }
        }
        Err(e) => {
            eprintln!("❌ Failed to connect to daemon: {}", e);
            return Err(e);
        }
    }

    Ok(())
}

/// List projects from the daemon
async fn handle_daemon_projects(port: u16) -> Result<()> {
    use st::daemon_client::print_projects;

    let client = DaemonClient::new(port);

    // Ensure daemon is running (auto-start if needed)
    match client.ensure_running().await {
        Ok(_) => {
            if let Ok(projects) = client.get_projects().await {
                print_projects(&projects);
            } else {
                eprintln!("❌ Failed to get projects from daemon");
            }
        }
        Err(e) => {
            eprintln!("❌ Failed to connect to daemon: {}", e);
            return Err(e);
        }
    }

    Ok(())
}

/// Show Foken credits from the daemon
async fn handle_daemon_credits(port: u16) -> Result<()> {
    use st::daemon_client::print_credits;

    let client = DaemonClient::new(port);

    // Ensure daemon is running (auto-start if needed)
    match client.ensure_running().await {
        Ok(_) => {
            if let Ok(credits) = client.get_credits().await {
                print_credits(&credits);
            } else {
                eprintln!("❌ Failed to get credits from daemon");
            }
        }
        Err(e) => {
            eprintln!("❌ Failed to connect to daemon: {}", e);
            return Err(e);
        }
    }

    Ok(())
}

// =============================================================================
// AI GUARDIAN - System-wide protection daemon
// =============================================================================

/// Run the Guardian daemon (called by systemd service)
async fn run_guardian_daemon() -> Result<()> {
    use st::ai_guardian::AiGuardian;
    use tokio::time::{sleep, Duration};

    println!(r#"
╔═══════════════════════════════════════════════════════════════════════════════╗
║                                                                               ║
║    🛡️  SMART TREE GUARDIAN - System-wide AI Protection Active 🛡️            ║
║                                                                               ║
╚═══════════════════════════════════════════════════════════════════════════════╝
"#);

    let guardian = AiGuardian::new();

    // Watch paths for suspicious files
    let watch_paths = vec![
        "/home",
        "/tmp",
        "/var/tmp",
    ];

    println!("Watching paths: {:?}", watch_paths);
    println!("Press Ctrl+C to stop.\n");

    // Main protection loop
    loop {
        for watch_path in &watch_paths {
            let path = std::path::Path::new(watch_path);
            if !path.exists() {
                continue;
            }

            // Scan for new/modified files with potential injection content
            scan_directory_for_threats(&guardian, path);
        }

        // Sleep between scans
        sleep(Duration::from_secs(60)).await;
    }
}

/// Scan a directory for prompt injection threats
#[allow(dead_code)]
fn scan_directory_for_threats(guardian: &st::ai_guardian::AiGuardian, path: &std::path::Path) {
    use walkdir::WalkDir;

    let suspicious_extensions = [
        "md", "txt", "json", "yaml", "yml", "toml", "sh", "py", "js", "ts",
    ];

    for entry in WalkDir::new(path)
        .max_depth(3)
        .follow_links(false)
        .into_iter()
        .filter_map(|e| e.ok())
    {
        let entry_path = entry.path();

        // Skip hidden files and directories
        if entry_path
            .file_name()
            .map(|n| n.to_string_lossy().starts_with('.'))
            .unwrap_or(false)
        {
            continue;
        }

        // Only scan text files that could contain injection attempts
        if let Some(ext) = entry_path.extension() {
            let ext_str = ext.to_string_lossy().to_lowercase();
            if suspicious_extensions.contains(&ext_str.as_str()) {
                let threats = guardian.scan_file(entry_path);

                // Report critical and dangerous threats
                for threat in threats {
                    if matches!(
                        threat.level,
                        st::ai_guardian::ThreatLevel::Critical | st::ai_guardian::ThreatLevel::Dangerous
                    ) {
                        eprintln!(
                            "⚠️  THREAT DETECTED [{:?}] in {}: {}",
                            threat.level,
                            threat.location,
                            threat.pattern
                        );
                        eprintln!("    Context: {}", threat.context);
                        eprintln!("    Recommendation: {}", threat.recommendation);
                        eprintln!();
                    }
                }
            }
        }
    }
}

/// Scan a specific file for prompt injection
#[allow(dead_code)]
fn handle_guardian_scan(file_path: &std::path::Path) -> Result<()> {
    use st::ai_guardian::{AiGuardian, ThreatLevel};

    println!(r#"
╔═══════════════════════════════════════════════════════════════════════════════╗
║            🛡️  SMART TREE GUARDIAN - File Scan 🛡️                            ║
╚═══════════════════════════════════════════════════════════════════════════════╝
"#);

    println!("Scanning: {}\n", file_path.display());

    let guardian = AiGuardian::new();
    let threats = guardian.scan_file(file_path);

    if threats.is_empty() {
        println!("✅ No threats detected. File appears safe.");
        return Ok(());
    }

    // Count by severity
    let critical = threats.iter().filter(|t| t.level == ThreatLevel::Critical).count();
    let dangerous = threats.iter().filter(|t| t.level == ThreatLevel::Dangerous).count();
    let suspicious = threats.iter().filter(|t| t.level == ThreatLevel::Suspicious).count();

    println!("Found {} issues:\n", threats.len());
    println!("  🔴 Critical:   {}", critical);
    println!("  🟠 Dangerous:  {}", dangerous);
    println!("  🟡 Suspicious: {}", suspicious);
    println!();

    for threat in &threats {
        let icon = match threat.level {
            ThreatLevel::Critical => "🔴",
            ThreatLevel::Dangerous => "🟠",
            ThreatLevel::Suspicious => "🟡",
            ThreatLevel::Safe => "🟢",
        };

        println!("{} [{:?}] {}", icon, threat.level, threat.pattern);
        println!("   Location: {}", threat.location);
        if !threat.context.is_empty() {
            println!("   Context: {}", threat.context);
        }
        println!("   Action: {}", threat.recommendation);
        println!();
    }

    if critical > 0 {
        println!("⛔ RECOMMENDATION: DO NOT process this file with AI assistants.");
        println!("   Critical threats detected that could compromise AI behavior.");
    } else if dangerous > 0 {
        println!("⚠️  RECOMMENDATION: Review this file carefully before AI processing.");
        println!("   Potentially dangerous patterns detected.");
    }

    Ok(())
}
