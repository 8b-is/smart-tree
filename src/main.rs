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

// Import CLI definitions and handlers from the library
use st::cli::{Cli, ColorMode, OutputMode, PathMode};
use st::cli_commands::*;
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

    // Code intelligence is entirely local and in-process. It needs neither a
    // running daemon nor startup network checks, and stdout stays valid JSON.
    if let Some(st::cli::Cmd::Symbol(command)) = &cli.cmd {
        let result = st::mcp::symbol::execute(
            command.parameters(),
            &std::env::current_dir()?,
            &st::mcp::load_config()?,
        )?;
        println!("{}", serde_json::to_string_pretty(&result)?);
        return Ok(());
    }

    // Antigravity (agy) lifecycle hooks must output strict JSON on stdout with
    // zero startup banners, update checks, or daemon auto-start delays.
    if let Some(event) = &cli.agy_hook {
        return st::claude_hook::handle_agy_hook(event).await;
    }

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

    if cli.auto_update {
        return st::auto_update::run().await;
    }

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

    // Auto-start daemon for any command that might need it.
    // Skip for modes that run their own servers or are purely informational.
    let skip_autostart = cli.mcp
        || cli.http_daemon
        || cli.guardian_daemon
        || cli.version
        || cli.update
        || cli.cheet
        || cli.man
        || cli.completions.is_some()
        || cli.daemon_start
        || cli.daemon_install
        || cli.no_daemon
        || cli.recall.is_some()
        || cli.scan_opts.sync_user.is_some()
        || matches!(
            cli.cmd,
            Some(st::cli::Cmd::Service(_) | st::cli::Cmd::Sync(_))
        );
    if !skip_autostart {
        let client = DaemonClient::default_port();
        if let Err(e) = client.ensure_running().await {
            eprintln!("⚠️  Daemon auto-start failed: {}", e);
            eprintln!("   Try: st --daemon-start");
        }
    }

    // Handle tips flag if provided
    if let Some(state) = &cli.tips {
        let enable = state == "on";
        st::tips::handle_tips_flag(enable)?;
        return Ok(());
    }

    // Handle Aye consciousness commands
    if cli.agent_save {
        return handle_agent_save().await;
    }
    if cli.agent_restore {
        return handle_agent_restore().await;
    }
    if cli.agent_context {
        return handle_agent_context().await;
    }
    if cli.agent_kickstart {
        return handle_agent_kickstart().await;
    }
    if cli.agent_dump {
        return handle_agent_dump().await;
    }
    if let Some(args) = &cli.memory_anchor {
        if args.len() == 3 {
            return handle_memory_anchor(&args[0], &args[1], &args[2]).await;
        } else {
            return show_memory_anchor_help();
        }
    }
    if let Some(keywords) = &cli.memory_find {
        return handle_memory_find(keywords).await;
    }
    if cli.memory_stats {
        return handle_memory_stats().await;
    }
    if let Some(path) = &cli.update_consciousness {
        return handle_update_consciousness(path).await;
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
        return st::updater::run_update(false).await;
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
    if cli.mcp_install {
        return handle_mcp_install().await;
    }
    if cli.mcp_uninstall {
        return handle_mcp_uninstall().await;
    }
    if cli.mcp_status {
        return handle_mcp_status().await;
    }
    if let Some(user) = &cli.scan_opts.sync_user {
        let sync_cmd = st::cli::SyncUserCommand {
            user: user.clone(),
            dir: cli.scan_opts.sync_dir.clone(),
            limit: cli.scan_opts.sync_limit,
            jobs: 4,
            refresh_all: cli.scan_opts.refresh_all,
            no_forks: false,
            no_archived: false,
        };
        return handle_sync_user(&sync_cmd).await;
    }
    // Handle top-level subcommands
    if let Some(cmd) = cli.cmd {
        match cmd {
            st::cli::Cmd::Symbol(_) => unreachable!("symbol commands return before daemon startup"),
            st::cli::Cmd::Sync(sync_cmd) => {
                return handle_sync_user(&sync_cmd).await;
            }
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
            cli.allow.clone(),
            InMemoryLogStore::new(),
        )
        .await;
    }

    if cli.http_daemon {
        // Launch HTTP daemon with MCP, LLM proxy, The Custodian
        return run_daemon(cli.scan_opts.sse_port).await;
    }

    // Handle security commands
    if let Some(path) = &cli.security_scan {
        return handle_security_scan(path).await;
    }
    if let Some(file) = &cli.guardian_scan {
        return handle_guardian_scan(std::path::Path::new(file));
    }
    if cli.guardian_daemon {
        return run_guardian_daemon().await;
    }
    if cli.cleanup {
        return handle_security_cleanup().await;
    }
    if let Some(path) = &cli.integrity_scan {
        return handle_integrity_scan(path, cli.no_daemon).await;
    }
    if let Some(path) = &cli.cert_scan {
        return handle_certificate_scan(path, cli.scan_opts.mode, cli.no_daemon).await;
    }
    if cli.cert_audit {
        return handle_cert_audit(cli.cert_generate_script);
    }
    if let Some(hash) = &cli.hash_lookup {
        return handle_hash_lookup(hash);
    }

    // Handle Google Drive / Sync commands
    if let Some(action) = &cli.gdrive {
        return handle_gdrive_command(action).await;
    }

    // Handle Voice commands
    if let Some(action) = &cli.voice {
        return handle_voice_command(action).await;
    }

    // Handle hooks commands
    if cli.hooks_install {
        return install_hooks_to_agent().await;
    }
    if let Some(action) = &cli.hooks_config {
        return handle_hooks_config(action).await;
    }

    // Handle daemon control
    if cli.daemon_start {
        return handle_daemon_start(cli.scan_opts.sse_port).await;
    }
    if cli.daemon_stop {
        return handle_daemon_stop(cli.scan_opts.sse_port).await;
    }
    if cli.daemon_status {
        return handle_daemon_status(cli.scan_opts.sse_port).await;
    }
    if cli.daemon_context {
        return handle_daemon_context(cli.scan_opts.sse_port).await;
    }
    if let Some(query) = &cli.recall {
        let response = DaemonClient::new(cli.scan_opts.sse_port)
            .recall_context(query)
            .await?;
        println!("{}", serde_json::to_string_pretty(&response)?);
        return Ok(());
    }
    if cli.daemon_projects {
        return handle_daemon_projects(cli.scan_opts.sse_port).await;
    }
    if cli.daemon_credits {
        return handle_daemon_credits(cli.scan_opts.sse_port).await;
    }
    if cli.daemon_install {
        return service_manager::daemon_install_system();
    }

    // Handle mega sessions
    if let Some(name) = &cli.mega_start {
        let name_opt = if name.is_empty() {
            None
        } else {
            Some(name.as_str())
        };
        return handle_mega_start(name_opt).await;
    }
    if cli.mega_save {
        return handle_mega_save().await;
    }
    if cli.mega_list {
        return handle_mega_list().await;
    }
    if cli.mega_stats {
        return handle_mega_stats().await;
    }

    // Handle analysis commands
    if let Some(path) = &cli.token_stats {
        return handle_token_stats(path).await;
    }
    if let Some(path) = &cli.get_frequency {
        return handle_get_frequency(path).await;
    }

    // =========================================================================
    // THIN CLIENT - Prefer daemon, fall back to local scan when needed
    // =========================================================================
    let request = build_cli_request(&cli)?;

    let response = if cli.no_daemon {
        st::daemon_cli::execute_cli_scan(&request).context("Scan failed")?
    } else {
        let client = DaemonClient::default_port();
        match client.ensure_running().await {
            Ok(_) => client
                .cli_scan(request.clone())
                .await
                .context("Scan failed")?,
            Err(e) => {
                eprintln!("⚠️  Daemon unavailable ({})", e);
                eprintln!(
                    "   Falling back to standalone scan. Use --no-daemon to skip this warning."
                );
                st::daemon_cli::execute_cli_scan(&request).context("Scan failed")?
            }
        }
    };

    // Print Treehouse ASCII banner for interactive non-machine modes
    if std::io::stdout().is_terminal() && (request.mode == "smart" || request.mode == "classic") {
        print_treehouse_banner();
    }

    // Print output (already formatted by daemon)
    print!("{}", response.output);

    Ok(())
}

/// Prints a cool cyber-botanical Tree-House ASCII banner
fn print_treehouse_banner() {
    let banner = "
[1;32m      ⋆          ⋆          ⋆[0m
[1;32m  ⋆       [1;36m/ \\[1;32m         ⋆[0m
[1;32m       [1;36m/   o \\[1;32m    ⋆[0m
[1;32m   ⋆  [1;36m/_________\\[1;32m       ⋆[0m
[1;36m         |   | [0m
[1;36m         |   | [0m
[1;32m       _ |_ _|_ _       [0m
[1;36m   S M A R T  T R E E H O U S E[0m
";
    println!("{}", banner);
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
// Modular CLI command handlers live in `st::cli_commands` (`src/cli_commands.rs`).

/// Show version information with optional update checking
/// This combines the traditional --version output with smart update detection
/// Elvis would love this modern approach! 🕺
async fn show_version_with_updates() -> Result<()> {
    let current_version = env!("CARGO_PKG_VERSION");

    // Always show current version info first
    println!("🌳 Smart Tree v{}", current_version);
    if let Some(build) = option_env!("ST_BUILD_ID") {
        println!("🔖 Build: {}", build);
    }
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
        entries.sort_by_key(|a| std::cmp::Reverse(a.1));

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

/// Check for updates via Ed25519-signed Secure DNS (`_smart-tree.i1.is`) with fallback to feedback API
/// Returns update message if available, empty string if up-to-date
async fn check_for_updates_cli() -> Result<String> {
    if std::env::var("SMART_TREE_NO_UPDATE_CHECK").is_ok() {
        return Ok(String::new());
    }
    if let Ok(Ok(rel)) = tokio::time::timeout(
        std::time::Duration::from_secs(2),
        st::updater::query_secure_dns_version(),
    )
    .await
    {
        println!(
            "🔐 Secure DNS (_smart-tree.i1.is): v{} · Ed25519 keytag={} verified",
            rel.version, rel.key_tag
        );
        if !st::updater::is_newer_version(env!("CARGO_PKG_VERSION"), &rel.version) {
            return Ok(String::new());
        }
        return Ok(format!(
            "Smart Tree v{} is available (installed: v{}).\nRun st --update or visit https://i1.is/#smart-tree",
            rel.version,
            env!("CARGO_PKG_VERSION")
        ));
    }
    let client = st::feedback_client::FeedbackClient::new()?;
    let info = tokio::time::timeout(
        std::time::Duration::from_secs(2),
        client.check_for_updates(),
    )
    .await??;
    if !st::updater::is_newer_version(env!("CARGO_PKG_VERSION"), &info.version) {
        return Ok(String::new());
    }
    Ok(format!(
        "Smart Tree v{} is available (installed: v{}).\nRun st --update or visit {}",
        info.version,
        env!("CARGO_PKG_VERSION"),
        info.download_url
    ))
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

    // Load user config for daemon settings
    let st_config = st::config::StConfig::load().unwrap_or_default();

    // Start with current directory as sensible default (not entire HOME!)
    // Additional paths can be registered via /context/watch endpoint
    let cwd = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));

    let config = DaemonConfig {
        port,
        watch_paths: vec![cwd], // Just current dir, not entire HOME
        orchestrator_url: Some("wss://gpu.foken.ai/api/credits".to_string()),
        enable_credits: true,
        allow_external: st_config.daemon.allow_external,
    };

    start_daemon(config).await
}
