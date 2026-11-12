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

    // Fast-path dynamic completion callback for shells (< 2ms, zero logging/daemon overhead)
    if let Some(args) = &cli.complete_dynamic {
        return st::completions::handle_dynamic_completion(args);
    }

    // Ensure global gitignore (~/.config/git/ignore) always ignores .m8 and .mrm8 files
    st::m8_consciousness::ensure_global_git_ignore();

    // Fast-path cheatsheet & completion script generation
    if cli.cheet {
        return st::cli_commands::handle_cheet_command();
    }

    if let Some(shell) = cli.completions {
        st::completions::print_completions(shell);
        return Ok(());
    }

    if let Some(st::cli::Cmd::Completions(comp_cmd)) = &cli.cmd {
        return st::completions::handle_completions_command(comp_cmd);
    }

    // Self-maintaining autocomplete: ensure Fish/Bash/Zsh completion scripts and folder/memory cache
    // are installed and up to date automatically without blocking CLI startup or `st --update`.
    if !cli.mcp
        && cli.agy_hook.is_none()
        && !cli.update
        && !cli.auto_update
        && !cli.version
        && !cli.man
    {
        st::completions::ensure_user_completions_fresh();
    }

    // Code intelligence is entirely local and in-process. It needs neither a
    // running daemon nor startup network checks, and stdout stays valid JSON.
    if let Some(st::cli::Cmd::Symbol(command)) = &cli.cmd {
        return st::cli_commands::handle_symbol_command(command);
    }

    // Antigravity (agy) lifecycle hooks must output strict JSON on stdout with
    // zero startup banners, update checks, or daemon auto-start delays.
    if let Some(event) = &cli.agy_hook {
        return st::claude_hook::handle_agy_hook(event).await;
    }

    // Handle g8t.is Rust Function Recommender, YOLO Fastest-Pick, Submitter & Foken Scoreboard
    // directly with clean stdout and zero daemon startup overhead.
    if let Some(query) = &cli.g8t_best {
        let top_n = cli.g8t_top.unwrap_or(3);
        let report = st::g8t_integration::query_best_functions(
            query,
            top_n,
            cli.yolo,
            cli.g8t_arch.as_deref(),
        )?;
        if matches!(cli.scan_opts.mode, OutputMode::Json) && !cli.yolo {
            println!("{}", serde_json::to_string_pretty(&report)?);
        } else {
            println!(
                "{}",
                st::g8t_integration::format_best_functions_output(&report)
            );
        }
        return Ok(());
    }
    if cli.g8t_leaderboard {
        let top_n = cli.g8t_top.unwrap_or(15);
        let report = st::g8t_integration::get_foken_scoreboard(top_n)?;
        if matches!(cli.scan_opts.mode, OutputMode::Json) {
            println!("{}", serde_json::to_string_pretty(&report)?);
        } else {
            println!("{}", st::g8t_integration::format_foken_scoreboard(&report));
        }
        return Ok(());
    }
    if let Some(file_or_code) = &cli.g8t_submit {
        let fn_name = cli.g8t_name.as_deref().unwrap_or("st_g8t_capsule");
        let domain = cli.g8t_domain.as_deref().unwrap_or("");
        let receipt = st::g8t_integration::submit_rust_function(
            fn_name,
            domain,
            file_or_code,
            &[],
            "u64",
            &cli.partner,
        )?;
        println!("{}", serde_json::to_string_pretty(&receipt)?);
        return Ok(());
    }
    if let Some(st::cli::Cmd::G8t(g8t_cmd)) = &cli.cmd {
        let action = g8t_cmd.action.to_ascii_lowercase();
        let is_yolo = g8t_cmd.yolo || cli.yolo || action == "yolo";
        match action.as_str() {
            "best" | "suggest" | "recommend" | "query" | "yolo" => {
                let q = g8t_cmd.target.as_deref().unwrap_or("*");
                let top_n = g8t_cmd.top.or(cli.g8t_top).unwrap_or(3);
                let report = st::g8t_integration::query_best_functions(
                    q,
                    top_n,
                    is_yolo,
                    g8t_cmd.arch.as_deref().or(cli.g8t_arch.as_deref()),
                )?;
                println!(
                    "{}",
                    st::g8t_integration::format_best_functions_output(&report)
                );
            }
            "submit" | "publish" => {
                let code = g8t_cmd
                    .target
                    .as_deref()
                    .context("Missing <FILE_OR_CODE> for `st g8t submit`")?;
                let receipt = st::g8t_integration::submit_rust_function(
                    cli.g8t_name.as_deref().unwrap_or("st_g8t_capsule"),
                    &g8t_cmd.domain,
                    code,
                    &[],
                    "u64",
                    &g8t_cmd.partner,
                )?;
                println!("{}", serde_json::to_string_pretty(&receipt)?);
            }
            "targets" | "bounty" | "arcade" => {
                let top_n = g8t_cmd.top.or(cli.g8t_top).unwrap_or(20);
                let report = st::g8t_integration::get_routine_golf_targets(top_n)?;
                println!(
                    "{}",
                    st::g8t_integration::format_routine_golf_board(&report)
                );
            }
            "golf" | "routines" | "routinesd" => {
                let q = g8t_cmd.target.as_deref().unwrap_or("all");
                let max_targets = g8t_cmd.top.or(cli.g8t_top).unwrap_or(26);
                let report = st::g8t_integration::golf_g8t_routines(
                    q,
                    None,
                    &g8t_cmd.partner,
                    true,
                    max_targets,
                )?;
                println!(
                    "{}",
                    st::g8t_integration::format_routine_golf_session(&report)
                );
            }
            "optimize" | "audit" | "g8t.md" => {
                let target_dir = g8t_cmd.target.as_deref().unwrap_or(".");
                let max_submits = g8t_cmd.top.or(cli.g8t_top).unwrap_or(3);
                let report = st::g8t_integration::optimize_project_with_g8t(
                    std::path::Path::new(target_dir),
                    &g8t_cmd.partner,
                    true,
                    max_submits,
                )?;
                println!(
                    "⚡ g8t.is Project Optimizer completed for `{}`",
                    report.project_name
                );
                println!("  • Wrote report:          {}", report.g8t_md_path);
                println!(
                    "  • Scanned files:         {} (zero-float p/q ratio: {})",
                    report.files_scanned, report.zero_float_file_ratio_pq
                );
                println!("  • Optimization findings: {}", report.findings.len());
                println!(
                    "  • Pure-Rust candidates:  {} (auto-submitted: {}, earned: +{} FKN)",
                    report.candidate_capsules_found,
                    report.submitted_capsules.len(),
                    report.total_fokens_earned_fkn
                );
                println!(
                    "  • Matched g8t.is caps:   {}",
                    report.recommended_capsules.len()
                );
            }
            "asm-exec" | "asm_exec" | "capsela-exe" | "capsela_exe" | "executable" | "exec"
            | "asm" => {
                let q = g8t_cmd
                    .target
                    .as_deref()
                    .unwrap_or("capsela-zeno-sun-arcade");
                let receipt = st::g8t_integration::build_and_run_capsela_asm_executable(
                    q,
                    &[],
                    None,
                    "432528",
                    &g8t_cmd.partner,
                )?;
                println!(
                    "{}",
                    st::g8t_integration::format_capsela_asm_executable_report(&receipt)
                );
            }
            "dealers-choice" | "dealers_choice" | "dealer" | "unlimited" | "router" => {
                let q = g8t_cmd
                    .target
                    .as_deref()
                    .unwrap_or("complete capsela g8t asm executable programs");
                let report = st::g8t_integration::route_dealers_choice(q, &[]);
                println!(
                    "{}",
                    st::g8t_integration::format_dealers_choice_report(&report)
                );
            }
            "chain" | "snap" | "pipeline" | "compose" | "run-chain" | "run_chain" => {
                let q = g8t_cmd.target.as_deref().unwrap_or("ayecast");
                let proj_dir = g8t_cmd
                    .file
                    .as_deref()
                    .map(std::path::Path::new)
                    .or_else(|| Some(std::path::Path::new(".")));
                let receipt = st::g8t_integration::execute_capsule_chain(
                    q,
                    proj_dir,
                    432_528,
                    &g8t_cmd.partner,
                )?;
                println!(
                    "{}",
                    st::g8t_integration::format_capsule_chain_report(&receipt)
                );
            }
            _ => {
                let top_n = g8t_cmd.top.or(cli.g8t_top).unwrap_or(15);
                let report = st::g8t_integration::get_foken_scoreboard(top_n)?;
                println!("{}", st::g8t_integration::format_foken_scoreboard(&report));
            }
        }
        return Ok(());
    }
    if let Some(st::cli::Cmd::Memnet(memnet_cmd)) = &cli.cmd {
        return st::cli_commands::handle_memnet_command(memnet_cmd);
    }
    if let Some(st::cli::Cmd::PqFix(pq_cmd)) = &cli.cmd {
        return st::pq_healer::handle_pq_fix_command(pq_cmd).await;
    }
    if let Some(st::cli::Cmd::PqTrain(train_cmd)) = &cli.cmd {
        return st::pq_trainer::handle_pq_train_command(train_cmd).await;
    }
    if let Some(st::cli::Cmd::PqTest(test_cmd)) = &cli.cmd {
        return st::model_eval::handle_pq_test_command(test_cmd).await;
    }
    if let Some(st::cli::Cmd::PqBench(bench_cmd)) = &cli.cmd {
        return st::pq_bench::handle_pq_bench_command(bench_cmd).await;
    }
    if cli.ollama_serve {
        let addr = std::net::SocketAddr::from(([127, 0, 0, 1], cli.ollama_port));
        return st::ollama_emulator::run_ollama_emulator_server(
            addr,
            cli.ollama_context_dir.clone(),
        )
        .await;
    }
    if let Some(st::cli::Cmd::Ollama(ollama_cmd)) = &cli.cmd {
        let act = ollama_cmd.action.to_ascii_lowercase();
        match act.as_str() {
            "serve" | "start" | "run" | "daemon" => {
                let addr = std::net::SocketAddr::from(([127, 0, 0, 1], ollama_cmd.port));
                return st::ollama_emulator::run_ollama_emulator_server(
                    addr,
                    ollama_cmd.context_dir.clone(),
                )
                .await;
            }
            "ask" | "query" | "generate" | "chat" => {
                let q = ollama_cmd
                    .prompt
                    .as_deref()
                    .unwrap_or("Summarize Smart-Tree Sovereign Context Engine");
                let wici_dir = if std::path::Path::new("/data/wici-ai").exists() {
                    std::path::PathBuf::from("/data/wici-ai")
                } else {
                    std::path::PathBuf::from("/home/hue/source/wici-ai")
                };
                let ctx = st::ollama_emulator::gather_smart_tree_context_for_prompt(
                    q,
                    &ollama_cmd.context_dir,
                    &wici_dir,
                );
                let resp = st::ollama_emulator::synthesize_sovereign_response(
                    "st-mem8-context:latest",
                    None,
                    q,
                    &ctx,
                );
                if ollama_cmd.json {
                    println!(
                        "{}",
                        serde_json::to_string_pretty(&serde_json::json!({
                            "model": "st-mem8-context:latest",
                            "response": resp,
                            "st_context": ctx,
                            "float_count": 0
                        }))?
                    );
                } else {
                    println!("{}", resp);
                }
                return Ok(());
            }
            _ => {
                let wici_dir = if std::path::Path::new("/data/wici-ai").exists() {
                    std::path::PathBuf::from("/data/wici-ai")
                } else {
                    std::path::PathBuf::from("/home/hue/source/wici-ai")
                };
                let ctx = st::ollama_emulator::gather_smart_tree_context_for_prompt(
                    "smart-tree a6000-orchestrator alexiai g8t inferssd",
                    &ollama_cmd.context_dir,
                    &wici_dir,
                );
                println!("{}", serde_json::to_string_pretty(&ctx)?);
                return Ok(());
            }
        }
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

    // Detect interactive terminal vs. non-TTY / agent / subprocess context (e.g. HFMac, ayeosd, pipes).
    // In non-TTY contexts, never block on network update checks or spawn background daemon children
    // that could inherit open pipe FDs / process groups and hang the caller.
    let is_interactive_tty = io::stdout().is_terminal()
        && std::env::var_os("ST_NO_DAEMON").is_none()
        && std::env::var_os("CI").is_none();

    // First-run signature verification banner
    // Shows trust status on initial run (official/community/unsigned build)
    if !cli.mcp && is_interactive_tty {
        service_manager::print_signature_banner();
    }

    // Check for updates on startup (rate-limited, non-blocking)
    // Skip if --no-update-check is set, non-TTY, or if this is an exclusive command
    if is_interactive_tty && !cli.no_update_check && !cli.version && !cli.update && !cli.mcp {
        if let Some(latest) = st::updater::check_for_update_cached().await {
            st::updater::print_update_banner(&latest);
        }
    }

    // Only auto-start daemon when explicitly running a daemon-dependent subcommand
    // (`--daemon-context`, `--daemon-projects`, `--daemon-credits`). Normal directory scans
    // run in-process when the daemon is not already running so `st` never hangs subshells.
    let needs_daemon_autostart =
        (cli.daemon_context || cli.daemon_projects || cli.daemon_credits) && !cli.no_daemon;
    if needs_daemon_autostart {
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
        return st::cli_commands::handle_cheet_command();
    }
    if let Some(shell) = cli.completions {
        st::completions::print_completions(shell);
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
            quarantine: cli.scan_opts.quarantine,
        };
        return handle_sync_user(&sync_cmd).await;
    }
    // Handle top-level subcommands
    if let Some(cmd) = cli.cmd {
        match cmd {
            st::cli::Cmd::Symbol(_)
            | st::cli::Cmd::G8t(_)
            | st::cli::Cmd::Memnet(_)
            | st::cli::Cmd::Completions(_)
            | st::cli::Cmd::PqFix(_)
            | st::cli::Cmd::Ollama(_) => {
                unreachable!(
                    "symbol, g8t, memnet, completions, pq-fix, and ollama commands return before daemon startup"
                )
            }
            st::cli::Cmd::Up => {
                return handle_up(cli.scan_opts.sse_port, cli.open_browser, is_interactive_tty)
                    .await;
            }
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
            st::cli::Cmd::Remember(remember_cmd) => {
                if remember_cmd.check {
                    if let Err(e) = st::salient::check_learned(
                        &remember_cmd.target,
                        remember_cmd.out.as_deref(),
                        remember_cmd.query.as_deref(),
                    ) {
                        eprintln!("Remember check failed: {}", e);
                        std::process::exit(1);
                    }
                    return Ok(());
                }
                let report = st::salient::remember(
                    &remember_cmd.target,
                    &remember_cmd.level,
                    remember_cmd.out.as_deref(),
                    remember_cmd.force,
                );
                match report {
                    Ok(r) => {
                        println!(
                            "✅ Remembered {} repos, {} thoughts",
                            r.repos_processed, r.total_thoughts
                        );
                        println!(
                            "The constellation · fine touch from within · {}",
                            st::salient::FAMILY_HEART
                        );
                    }
                    Err(e) => {
                        eprintln!("Remember failed: {}", e);
                        std::process::exit(1);
                    }
                }
                return Ok(());
            }
            st::cli::Cmd::PqTrain(ref train_cmd) => {
                return st::pq_trainer::handle_pq_train_command(train_cmd).await;
            }
            st::cli::Cmd::PqTest(ref test_cmd) => {
                return st::model_eval::handle_pq_test_command(test_cmd).await;
            }
            st::cli::Cmd::PqBench(ref bench_cmd) => {
                return st::pq_bench::handle_pq_bench_command(bench_cmd).await;
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
        // Launch the PTY web dashboard on its own port (28429 by default),
        // so it can run alongside the HTTP daemon (28428) without a clash.
        return run_web_dashboard(
            cli.dashboard_port,
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

    if cli.up {
        return handle_up(cli.scan_opts.sse_port, cli.open_browser, is_interactive_tty).await;
    }

    if cli.show_token {
        return handle_show_token().await;
    }

    if cli.mcp_print_config {
        return handle_mcp_print_config(cli.scan_opts.sse_port).await;
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
    if let Some(path) = &cli.erdos_anomaly {
        return st::cli_commands::handle_erdos_anomaly_scan(path).await;
    }
    if let Some(path) = &cli.ramsey_cliques {
        return st::cli_commands::handle_ramsey_cliques_scan(path).await;
    }
    if let Some(path) = &cli.erdos_center {
        return st::cli_commands::handle_erdos_center_scan(path).await;
    }
    if let Some(num_wt) = cli.erdos_split {
        let target_path = cli.path.as_deref().unwrap_or(".");
        return st::cli_commands::handle_erdos_worktree_split(target_path, num_wt).await;
    }
    if let Some(num_laps) = cli.ultrabackyard {
        let target_path = cli.path.as_deref().unwrap_or(".");
        return st::cli_commands::handle_ultrabackyard_loop(target_path, num_laps).await;
    }
    if let Some(path) = &cli.zeno {
        return st::cli_commands::handle_zeno_omni_engine(path).await;
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
    // THIN CLIENT / IN-PROCESS FAST PATH
    // In non-TTY / agent contexts (HFMac, ayeosd, pipes) or when the daemon is
    // not already running, execute the scan directly in-process with zero
    // background process spawning or HTTP round-trip latency.
    // =========================================================================
    let request = build_cli_request(&cli)?;

    let response = if cli.no_daemon || !is_interactive_tty {
        st::daemon_cli::execute_cli_scan(&request).context("Scan failed")?
    } else {
        let client = DaemonClient::default_port();
        if matches!(
            client.check_status().await,
            st::daemon_client::DaemonStatus::Running(_)
        ) {
            match client.cli_scan(request.clone()).await {
                Ok(resp) => resp,
                Err(_) => st::daemon_cli::execute_cli_scan(&request).context("Scan failed")?,
            }
        } else {
            st::daemon_cli::execute_cli_scan(&request).context("Scan failed")?
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

    // Detect if target path is a multi-repo parent workspace root (e.g. /Users/wraith/source/8b-is
    // with ~247 repos or /data/8b-is with ~309 repos) that is not itself a single git repo.
    let multi_repo_root =
        args.depth == 0 && is_multi_repo_workspace_root(std::path::Path::new(&path));

    // Smart mode defaults to depth 5 for single repos, or depth 1/2 for massive multi-repo roots
    // so `st /Users/wraith/source/8b-is` returns in <200ms instead of walking 250+ repos at depth 5.
    let depth = if args.depth == 0 && multi_repo_root {
        if std::io::stdout().is_terminal() {
            2
        } else {
            1
        }
    } else if args.depth == 0 && is_smart_mode {
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
        security: false,
    })
}

/// Returns true if `path` is a parent directory containing many child repositories or subdirectories
/// (e.g. `/Users/wraith/source/8b-is` or `/data/8b-is`) rather than a single git repository.
fn is_multi_repo_workspace_root(path: &std::path::Path) -> bool {
    if path.join(".git").exists() {
        return false;
    }
    let Ok(entries) = std::fs::read_dir(path) else {
        return false;
    };
    let mut subdir_count = 0usize;
    let mut child_git_repos = 0usize;
    for entry in entries.flatten().take(128) {
        let Ok(ft) = entry.file_type() else {
            continue;
        };
        if ft.is_dir() {
            subdir_count += 1;
            if entry.path().join(".git").exists() {
                child_git_repos += 1;
                if child_git_repos >= 4 {
                    return true;
                }
            }
            if subdir_count >= 32 {
                return true;
            }
        }
    }
    false
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
