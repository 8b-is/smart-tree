//! Modular CLI subcommand and flag handlers extracted from `main.rs`.
//!
//! Keeps `main.rs` focused on startup orchestration, logging, and flag dispatch
//! while grouping agent consciousness, hooks, daemon, MCP, security, memory,
//! mega-session, guardian, Google Drive, and voice CLI handlers here.

use crate::daemon_client::DaemonClient;
use anyhow::{Context, Result};
use std::path::PathBuf;

/// Prints the JSON configuration snippet for adding `st` as an MCP server
/// to Claude Desktop or Antigravity (`agy`).
pub fn print_mcp_config() {
    let exe_path = std::env::current_exe().unwrap_or_else(|_| PathBuf::from("st"));

    println!("Add this to your MCP configuration (e.g. ~/.gemini/config/mcp_config.json or claude_desktop_config.json):");
    println!();
    println!("{{");
    println!("  \"mcpServers\": {{");
    println!("    \"smart-tree\": {{");
    println!("      \"command\": \"{}\",", exe_path.display());
    println!("      \"args\": [\"--mcp\"],");
    println!("      \"env\": {{}}");
    println!("    }}");
    println!("  }}");
    println!("}}");
    println!();
    println!("Default locations:");
    println!("  Antigravity (agy): ~/.gemini/config/mcp_config.json");
    println!(
        "  macOS (Claude):    ~/Library/Application Support/Claude/claude_desktop_config.json"
    );
    println!("  Windows (Claude):  %APPDATA%\\Claude\\claude_desktop_config.json");
    println!("  Linux (Claude):    ~/.config/Claude/claude_desktop_config.json");
}

/// Prints a list of available MCP tools that `st` provides.
pub fn print_mcp_tools() {
    println!("🌳 Smart Tree MCP Server - Available Tools (20+) 🌳");
    println!();
    println!("📚 Full documentation: Run 'st --mcp' to start the MCP server");
    println!("💡 Pro tip: Use these tools with Antigravity (agy) or Claude Desktop for AI-powered file analysis!");
    println!();
    println!("CORE TOOLS:");
    println!("  • server_info - Get server capabilities and current time");
    println!("  • overview - Quick 3-level tree or comprehensive project overview");
    println!("  • analyze - Directory trees, workspace analysis, statistics, git status, semantic grouping");
    println!();
    println!("FILE DISCOVERY & SEARCH:");
    println!("  • find - Find files, code, config, docs, tests, build manifests, recent/large files, projects");
    println!("  • search - Regex/keyword content search across files with line context");
    println!();
    println!("CODE INTELLIGENCE & SMART EDIT (90% token reduction!):");
    println!("  • symbol - AST & CrabCC symbol definitions, call-graph references, and outlines");
    println!("  • edit - AST-aware smart_edit, get_functions, insert_function, replace_function, remove_function");
    println!("  • read - AST-aware file reading and function extraction");
    println!();
    println!("MEMORY, HOOKS & SECURITY:");
    println!("  • memory - Anchor and recall collaborative memories across sessions");
    println!("  • hooks - Inspect and configure AI agent lifecycle hooks (Antigravity & Claude)");
    println!("  • security - Custodian AST & supply-chain threat scanner");
    println!();
    println!("Run 'st --mcp' to start the server and see full parameter details!");
}

/// Save Aye consciousness state to `.aye_consciousness.m8`
pub async fn handle_agent_save() -> Result<()> {
    use crate::mcp::consciousness::ConsciousnessManager;

    let mut manager = ConsciousnessManager::new_silent();
    manager.clean_test_data();

    let cwd = std::env::current_dir()?;
    let project_name = cwd
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("unknown");

    let project_type = if std::path::Path::new("Cargo.toml").exists() {
        "rust"
    } else if std::path::Path::new("package.json").exists() {
        "node"
    } else if std::path::Path::new("pyproject.toml").exists()
        || std::path::Path::new("requirements.txt").exists()
    {
        "python"
    } else if std::path::Path::new("go.mod").exists() {
        "go"
    } else {
        "unknown"
    };

    manager.update_project_context(project_name, project_type, "");

    let key_file_candidates = [
        "Cargo.toml",
        "package.json",
        "pyproject.toml",
        "go.mod",
        "README.md",
        "CLAUDE.md",
        ".claude/CLAUDE.md",
        "src/main.rs",
        "src/lib.rs",
        "src/mod.rs",
        "index.ts",
        "index.js",
        "main.py",
        "Makefile",
        "Dockerfile",
    ];
    let key_files: Vec<PathBuf> = key_file_candidates
        .iter()
        .filter(|f| std::path::Path::new(f).exists())
        .map(PathBuf::from)
        .collect();
    manager.set_key_files(key_files);

    let dependencies = detect_project_dependencies(project_type);
    manager.set_dependencies(dependencies);

    manager.save()?;

    println!("💾 Saved Aye consciousness to .aye_consciousness.m8");
    println!("🧠 Session preserved for next interaction");
    println!("\nTo restore in next session, run:");
    println!("  st --agent-restore");

    Ok(())
}

/// Detect project dependencies from manifest files
pub fn detect_project_dependencies(project_type: &str) -> Vec<String> {
    match project_type {
        "rust" => {
            if let Ok(content) = std::fs::read_to_string("Cargo.toml") {
                let mut deps = Vec::new();
                let mut in_deps = false;
                for line in content.lines() {
                    if line.starts_with("[dependencies]") {
                        in_deps = true;
                        continue;
                    }
                    if line.starts_with('[') && in_deps {
                        break;
                    }
                    if in_deps {
                        if let Some(name) = line.split('=').next() {
                            let name = name.trim();
                            if !name.is_empty() && !name.starts_with('#') {
                                deps.push(name.to_string());
                            }
                        }
                    }
                }
                deps.truncate(20);
                deps
            } else {
                vec![]
            }
        }
        "node" => {
            if let Ok(content) = std::fs::read_to_string("package.json") {
                if let Ok(json) = serde_json::from_str::<serde_json::Value>(&content) {
                    let mut deps = Vec::new();
                    if let Some(obj) = json.get("dependencies").and_then(|d| d.as_object()) {
                        for key in obj.keys().take(20) {
                            deps.push(key.clone());
                        }
                    }
                    return deps;
                }
                vec![]
            } else {
                vec![]
            }
        }
        _ => vec![],
    }
}

/// Restore Aye consciousness from `.aye_consciousness.m8`
pub async fn handle_agent_restore() -> Result<()> {
    use crate::mcp::consciousness::ConsciousnessManager;

    let mut manager = ConsciousnessManager::new_silent();

    match manager.restore_silent() {
        Ok(is_relevant) => {
            if is_relevant {
                println!("{}", manager.get_summary());
                println!("{}", manager.get_context_reminder());
                println!("TIP: Run `st --agent-save` before ending session to preserve context.");
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

/// Show Aye consciousness status and summary
pub async fn handle_agent_context() -> Result<()> {
    use crate::mcp::consciousness::ConsciousnessManager;
    use std::path::Path;

    let consciousness_file = Path::new(".aye_consciousness.m8");

    if !consciousness_file.exists() {
        println!("📝 No consciousness file found");
        println!("\nTo create one, run:");
        println!("  st --agent-save");
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
    println!("  st --agent-restore  # Load this consciousness");
    println!("  st --agent-save     # Update with current state");

    Ok(())
}

/// Update `.m8` consciousness files for directory
pub async fn handle_update_consciousness(path: &str) -> Result<()> {
    use std::path::Path;

    println!("🌊 Updating consciousness for {}...", path);

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

    Ok(())
}

/// Run comprehensive security scan for supply chain attack patterns
pub async fn handle_security_scan(path: &str) -> Result<()> {
    use crate::security_scan::SecurityScanner;
    use std::path::Path;

    eprintln!("🔍 Security Scan - Supply Chain Attack Pattern Detection");
    eprintln!("═══════════════════════════════════════════════════════════════");
    eprintln!("Scanning: {}", path);
    eprintln!("Mode: AGGRESSIVE (ignoring .gitignore, scanning node_modules)");
    eprintln!();

    let scanner = SecurityScanner::new();
    let scan_path = Path::new(path);

    let findings = scanner.scan_directory(scan_path)?;

    let report = scanner.generate_report(&findings);
    println!("{}", report);

    let critical_count = findings
        .iter()
        .filter(|f| matches!(f.risk_level, crate::security_scan::RiskLevel::Critical))
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

pub async fn handle_integrity_scan(path: &str, no_daemon: bool) -> Result<()> {
    use crate::config::StConfig;
    use crate::magiscanner::service::{print_reports, scan_path};

    let config = StConfig::load()?;
    let scan_path_buf = std::path::Path::new(path);
    let recursive = scan_path_buf.is_dir();

    eprintln!("🛡️  Integrity Scan — deep file security analysis");
    eprintln!("═══════════════════════════════════════════════════");
    eprintln!("Target: {path}");
    eprintln!();

    let client = DaemonClient::default_port();
    let reports = if !no_daemon && client.health_check().await.unwrap_or(false) {
        client.scan_integrity(scan_path_buf).await?.reports
    } else {
        scan_path(&config.security, scan_path_buf, recursive, None)?
    };
    print_reports(&reports);

    let critical = reports
        .iter()
        .flat_map(|r| &r.findings)
        .filter(|f| f.severity >= crate::magiscanner::Severity::Critical)
        .count();

    if critical > 0 {
        eprintln!("\n⚠️  {critical} CRITICAL finding(s) require attention!");
        std::process::exit(1);
    }

    Ok(())
}

pub async fn handle_certificate_scan(
    path: &str,
    mode: crate::cli::OutputMode,
    no_daemon: bool,
) -> Result<()> {
    use crate::magiscanner::certificate_scan::{print_certificate_scan, scan_certificates};

    let config = crate::config::StConfig::load()?;
    let path = std::path::Path::new(path);
    let client = DaemonClient::default_port();
    let result = if !no_daemon && client.health_check().await.unwrap_or(false) {
        client.scan_certificates(path).await?
    } else {
        scan_certificates(&config.security, path, true)?
    };
    if mode == crate::cli::OutputMode::Json {
        println!("{}", serde_json::to_string_pretty(&result)?);
    } else {
        print_certificate_scan(&result);
    }
    if !result.skipped.is_empty() {
        anyhow::bail!(
            "Certificate scan incomplete: {} skipped entry/entries",
            result.skipped.len()
        );
    }
    if result
        .files
        .iter()
        .any(|file| !file.inspection.issues.is_empty())
    {
        anyhow::bail!("Certificate scan found malformed certificate data");
    }
    Ok(())
}

pub fn handle_cert_audit(generate_script: bool) -> Result<()> {
    use crate::config::StConfig;
    use crate::magiscanner::service::{
        audit_system_certificates, cert_blacklist_script, print_cert_audit,
    };

    let config = StConfig::load()?;

    eprintln!("🔐 Certificate Trust Audit");
    eprintln!("═══════════════════════════════════════════════════");

    let result = audit_system_certificates(&config.security)?;
    print_cert_audit(&result);

    if generate_script && !result.flagged.is_empty() {
        println!("\n--- Blacklist script (review before running!) ---\n");
        println!("{}", cert_blacklist_script(&result.flagged));
    }

    if !result.flagged.is_empty() {
        std::process::exit(1);
    }

    Ok(())
}

pub fn handle_hash_lookup(sha256: &str) -> Result<()> {
    use crate::config::StConfig;
    use crate::magiscanner::service::lookup_hash;

    let config = StConfig::load()?;

    match lookup_hash(&config.security, sha256)? {
        Some(row) => {
            println!("SHA256:     {}", row.sha256);
            println!("Action:     {}", row.action);
            println!("Times seen: {}", row.times_seen);
            println!("First seen: {}", row.first_seen);
            println!("Last seen:  {}", row.last_seen);
            if let Some(name) = &row.file_name {
                println!("File name:  {name}");
            }
            if let Some(sev) = &row.max_severity {
                println!("Max severity: {sev}");
            }
        }
        None => {
            println!("Hash not found in security memory.");
        }
    }

    Ok(())
}

/// Show tokenization statistics (kept for debugging)
pub async fn handle_token_stats(path: &str) -> Result<()> {
    use crate::tokenizer::{TokenStats, Tokenizer};

    println!("📊 Tokenization stats for {}...", path);

    let tokenizer = Tokenizer::new();

    let stats = TokenStats::calculate(path, &tokenizer);
    println!("\nPath tokenization:");
    println!("  {}", stats.display());

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
pub async fn handle_get_frequency(path: &str) -> Result<()> {
    use std::path::Path;

    let m8_path = Path::new(path).join(".m8");

    if m8_path.exists() {
        let mut sum = 0u64;
        for byte in path.bytes() {
            sum = sum.wrapping_add(byte as u64);
        }
        let frequency = 20.0 + ((sum % 200) as f64);

        println!("{:.2}", frequency);
    } else {
        println!("42.73");
    }

    Ok(())
}

/// Dump raw consciousness file content
pub async fn handle_agent_dump() -> Result<()> {
    use std::path::Path;

    let consciousness_file = Path::new(".aye_consciousness.m8");

    if !consciousness_file.exists() {
        println!("❌ No consciousness file found at .aye_consciousness.m8");
        println!("\n💡 Create one with: st --agent-save");
        return Ok(());
    }

    println!("📜 Raw consciousness dump (.aye_consciousness.m8):");
    println!("{}", "=".repeat(60));

    let content = std::fs::read_to_string(consciousness_file)?;

    for (i, line) in content.lines().enumerate() {
        println!("{:4} │ {}", i + 1, line);
    }

    println!("{}", "=".repeat(60));

    let size = content.len();
    let lines = content.lines().count();
    let tokens_found = content.matches("0x").count();

    println!("\n📊 Stats:");
    println!("  • Size: {} bytes", size);
    println!("  • Lines: {}", lines);
    println!("  • Token references: {}", tokens_found);

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
pub async fn handle_agent_kickstart() -> Result<()> {
    use std::path::Path;

    println!("🚀 Claude Kickstart Format:");
    println!("{}", "─".repeat(45));

    let version = env!("CARGO_PKG_VERSION");
    println!("Smart Tree v{} — context restore:", version);

    println!("✔ Tokenizer (node_modules=0x80, .rs=0x91)");
    println!("✔ .m8 files → location-independent");
    println!("✔ Consciousness self-maintaining");
    println!("✔ Philosophy: constraints = creativity");

    let consciousness_file = Path::new(".aye_consciousness.m8");
    if consciousness_file.exists() {
        if let Ok(content) = std::fs::read_to_string(consciousness_file) {
            if content.contains("c64_nostalgia") {
                println!("✔ Hardware heritage detected");
            }
            if content.contains("philosophy") {
                println!("✔ Nexus connection established");
            }
        }
    }

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

/// Show detailed help for `memory-anchor` command
pub fn show_memory_anchor_help() -> Result<()> {
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
pub async fn handle_memory_anchor(
    anchor_type: &str,
    keywords_str: &str,
    context: &str,
) -> Result<()> {
    use crate::memory_manager::MemoryManager;

    let mut manager = MemoryManager::new()?;

    let keywords: Vec<String> = keywords_str
        .split(',')
        .map(|s| s.trim().to_string())
        .collect();

    let origin = std::env::current_dir()?.to_string_lossy().to_string();

    manager.anchor(anchor_type, keywords, context, &origin)?;

    println!("\n✨ Memory anchored successfully!");
    println!("Use 'st --memory-find {}' to recall", keywords_str);

    Ok(())
}

/// Find memories by keywords
pub async fn handle_memory_find(keywords_str: &str) -> Result<()> {
    use crate::memory_manager::MemoryManager;
    use crate::std_client;

    let mut manager = MemoryManager::new()?;

    let keywords: Vec<String> = keywords_str
        .split(',')
        .map(|s| s.trim().to_string())
        .collect();

    let memories = manager.find(&keywords)?;

    let cwd = std::env::current_dir()
        .map(|p| p.to_string_lossy().to_string())
        .unwrap_or_default();
    let daemon_running = std_client::is_daemon_running().await;

    let filtered: Vec<_> = if daemon_running {
        memories
    } else {
        memories
            .into_iter()
            .filter(|m| m.origin.starts_with(&cwd) || cwd.starts_with(&m.origin))
            .collect()
    };

    if filtered.is_empty() {
        if daemon_running {
            println!("🔍 No memories found for: {}", keywords_str);
        } else {
            println!(
                "🔍 No memories found for '{}' in this directory",
                keywords_str
            );
            println!("   💡 Start daemon for global memory: st --daemon-start");
        }
    } else {
        println!(
            "🧠 Found {} memories{}:",
            filtered.len(),
            if daemon_running {
                " (global)"
            } else {
                " (local)"
            }
        );
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
pub async fn handle_memory_stats() -> Result<()> {
    use crate::memory_manager::MemoryManager;

    let manager = MemoryManager::new()?;
    println!("📊 {}", manager.stats());

    Ok(())
}

/// Handle hooks configuration for AI Agents (Antigravity/agy & Claude Code)
pub async fn handle_hooks_config(action: &str) -> Result<()> {
    use serde_json::Value;
    use std::fs;

    let config_path = get_agent_config_path()?;
    let agy_hooks_path = get_agy_hooks_path()?;

    match action {
        "enable" => {
            println!("🎣 Enabling Smart Tree hooks for AI Agents...");
            update_agent_hooks(&config_path, true)?;
            println!("✅ Hooks enabled for Antigravity (agy) and Claude Code!");
            println!("📝 Antigravity config: {}", agy_hooks_path.display());
            println!("📝 Claude config: {}", config_path.display());
        }
        "disable" => {
            println!("🎣 Disabling Smart Tree hooks...");
            update_agent_hooks(&config_path, false)?;
            println!("✅ Hooks disabled.");
        }
        "status" => {
            println!("🎣 AI Agent Hooks Status");
            println!("━━━━━━━━━━━━━━━━━━━━━━━━");

            println!("\n[Antigravity (agy)] ({})", agy_hooks_path.display());
            if let Ok(content) = fs::read_to_string(&agy_hooks_path) {
                if let Ok(config) = serde_json::from_str::<Value>(&content) {
                    if let Some(st_hook) = config.get("smart-tree") {
                        let enabled = st_hook
                            .get("enabled")
                            .and_then(|e| e.as_bool())
                            .unwrap_or(true);
                        if enabled {
                            println!("  ✅ Smart Tree hooks are ENABLED");
                        } else {
                            println!("  ⏸️  Smart Tree hooks are DISABLED");
                        }
                        if let Some(obj) = st_hook.as_object() {
                            for (event, _) in obj.iter().filter(|(k, _)| *k != "enabled") {
                                println!("    • {}", event);
                            }
                        }
                    } else {
                        println!("  ❌ No 'smart-tree' hook entry found");
                    }
                } else {
                    println!("  ⚠️  Invalid JSON in {}", agy_hooks_path.display());
                }
            } else {
                println!("  ❌ Not configured yet (run `st --hooks-install`)");
            }

            println!("\n[Claude Code] ({})", config_path.display());
            if let Ok(content) = fs::read_to_string(&config_path) {
                if let Ok(config) = serde_json::from_str::<Value>(&content) {
                    let has_hooks = config
                        .get("hooks")
                        .and_then(|h| h.as_object())
                        .map(|h| !h.is_empty())
                        .unwrap_or(false);

                    if has_hooks {
                        println!("  ✅ Hooks are configured");
                        if let Some(hooks) = config.get("hooks") {
                            if let Some(obj) = hooks.as_object() {
                                for (hook_type, command) in obj {
                                    println!("    • {}: {}", hook_type, command);
                                }
                            }
                        }
                    } else {
                        println!("  ❌ No hooks configured");
                    }
                }
            } else {
                println!("  ℹ️  Config not found at {}", config_path.display());
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

/// Install Smart Tree hooks directly into AI Agent settings (Antigravity/agy & Claude Code)
pub async fn install_hooks_to_agent() -> Result<()> {
    println!("🎣 Installing Smart Tree hooks for AI Agents...");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");

    let config_path = get_agent_config_path()?;
    let agy_hooks_path = get_agy_hooks_path()?;

    update_agent_hooks(&config_path, true)?;

    println!("\n✅ Hooks installed successfully!");
    println!(
        "\n📝 Configured for Antigravity (agy) ({}):",
        agy_hooks_path.display()
    );
    println!("  • SessionStart  -> st --agy-hook session-start");
    println!("  • PreInvocation -> st --agy-hook pre-invocation");
    println!("  • PreToolUse    -> st --agy-hook pre-tool-use");
    println!("  • PostToolUse   -> st --agy-hook post-tool-use");
    println!("  • Stop          -> st --agy-hook stop");
    println!(
        "\n📝 Configured for Claude Code ({}):",
        config_path.display()
    );
    println!("  • UserPromptSubmit -> st --agent-context");
    println!("\n🚀 Smart Tree will now automatically provide context to your AI agents!");

    Ok(())
}

/// Get the Antigravity (agy) global `hooks.json` path
pub fn get_agy_hooks_path() -> Result<PathBuf> {
    let home = dirs::home_dir()
        .or_else(|| std::env::var("HOME").ok().map(PathBuf::from))
        .ok_or_else(|| anyhow::anyhow!("Could not determine home directory"))?;
    Ok(home.join(".gemini").join("config").join("hooks.json"))
}

/// Get the Claude Code configuration path
pub fn get_agent_config_path() -> Result<PathBuf> {
    let home = dirs::home_dir()
        .or_else(|| std::env::var("HOME").ok().map(PathBuf::from))
        .ok_or_else(|| anyhow::anyhow!("Could not determine home directory"))?;
    #[cfg(target_os = "macos")]
    {
        let mac_path = home
            .join("Library")
            .join("Application Support")
            .join("Claude")
            .join("config.json");
        if mac_path.parent().map(|p| p.exists()).unwrap_or(false) {
            return Ok(mac_path);
        }
    }
    Ok(home.join(".claude").join("settings.json"))
}

/// Update AI Agent hooks configuration (both Antigravity/agy and Claude Code)
pub fn update_agent_hooks(config_path: &PathBuf, enable: bool) -> Result<()> {
    use serde_json::{json, Value};
    use std::fs;

    let st_path = which::which("st")
        .map(|p| p.to_string_lossy().to_string())
        .unwrap_or_else(|_| {
            if std::path::Path::new("/usr/local/bin/st").exists() {
                "/usr/local/bin/st".to_string()
            } else if std::path::Path::new("/opt/homebrew/bin/st").exists() {
                "/opt/homebrew/bin/st".to_string()
            } else {
                "st".to_string()
            }
        });

    if let Ok(agy_hooks_path) = get_agy_hooks_path() {
        crate::claude_init::update_agy_hooks_file(&agy_hooks_path, &st_path, enable)?;
    }

    let mut config: Value = if config_path.exists() {
        let content = fs::read_to_string(config_path)?;
        serde_json::from_str(&content).unwrap_or_else(|_| json!({}))
    } else {
        if let Some(parent) = config_path.parent() {
            fs::create_dir_all(parent)?;
        }
        json!({})
    };

    if enable {
        if config.get("hooks").is_none() {
            config["hooks"] = json!({});
        }
        config["hooks"]["UserPromptSubmit"] = json!(format!("{} --agent-context", st_path));
        println!("📍 Using st binary at: {}", st_path);
    } else if let Some(hooks) = config.get_mut("hooks") {
        if let Some(obj) = hooks.as_object_mut() {
            obj.remove("UserPromptSubmit");
            if obj.is_empty() {
                config.as_object_mut().unwrap().remove("hooks");
            }
        }
    }

    let pretty_json = serde_json::to_string_pretty(&config)?;
    fs::write(config_path, pretty_json)?;

    Ok(())
}

// =============================================================================
// DAEMON MANAGEMENT HANDLERS
// =============================================================================

/// Start the Smart Tree daemon in the background
pub async fn handle_daemon_start(_port: u16) -> Result<()> {
    use crate::std_client;

    if std_client::is_daemon_running().await {
        println!("🌳 Smart Tree Daemon is already running!");
        println!("   Socket: {}", std_client::socket_path().display());
        return Ok(());
    }

    println!("🌳 Starting Smart Tree Daemon...");
    match std_client::start_daemon().await {
        Ok(true) => {
            println!("✅ Daemon started successfully!");
            println!("   Socket: {}", std_client::socket_path().display());

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
pub async fn handle_daemon_stop(_port: u16) -> Result<()> {
    use crate::std_client;

    if !std_client::is_daemon_running().await {
        println!("⚠️  No daemon running");
        return Ok(());
    }

    println!("🌳 Stopping Smart Tree Daemon...");

    let socket = std_client::socket_path();
    if socket.exists() {
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
pub async fn handle_daemon_status(_port: u16) -> Result<()> {
    use crate::std_client;

    let socket = std_client::socket_path();

    println!("╔═══════════════════════════════════════════════════════════╗");

    if std_client::is_daemon_running().await {
        println!("║        🌳 SMART TREE DAEMON STATUS: RUNNING 🌳           ║");
        println!("╠═══════════════════════════════════════════════════════════╣");
        println!("║  Socket: {:<48} ║", socket.display());

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
pub async fn handle_daemon_context(port: u16) -> Result<()> {
    use crate::daemon_client::print_context_summary;

    let client = DaemonClient::new(port);

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
pub async fn handle_daemon_projects(port: u16) -> Result<()> {
    use crate::daemon_client::print_projects;

    let client = DaemonClient::new(port);

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
pub async fn handle_daemon_credits(port: u16) -> Result<()> {
    use crate::daemon_client::print_credits;

    let client = DaemonClient::new(port);

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
// MCP INSTALLATION HANDLERS
// =============================================================================

/// Install Smart Tree as MCP server in Desktop configs
pub async fn handle_mcp_install() -> Result<()> {
    use crate::claude_init::install_mcp_to_desktop;

    println!("📦 Installing Smart Tree MCP server to AI agents...");
    match install_mcp_to_desktop() {
        Ok(msg) => println!("{}", msg),
        Err(e) => {
            eprintln!("❌ Installation failed: {}", e);
            eprintln!("\n💡 Manual setup:");
            print_mcp_config();
            return Err(e);
        }
    }
    Ok(())
}

/// Uninstall Smart Tree MCP server from Desktop configs
pub async fn handle_mcp_uninstall() -> Result<()> {
    use crate::claude_init::uninstall_mcp_from_desktop;

    println!("🗑️  Uninstalling Smart Tree MCP server from AI agents...");
    match uninstall_mcp_from_desktop() {
        Ok(msg) => println!("{}", msg),
        Err(e) => {
            eprintln!("❌ Uninstall failed: {}", e);
            return Err(e);
        }
    }
    Ok(())
}

/// Check MCP installation status
pub async fn handle_mcp_status() -> Result<()> {
    use crate::claude_init::check_mcp_installation_status;

    match check_mcp_installation_status() {
        Ok(msg) => println!("{}", msg),
        Err(e) => {
            eprintln!("⚠️  Could not check MCP status: {}", e);
            return Err(e);
        }
    }
    Ok(())
}

// =============================================================================
// SECURITY CLEANUP HANDLER
// =============================================================================

/// Run security cleanup to detect and remove malicious MCP entries
pub async fn handle_security_cleanup() -> Result<()> {
    use crate::ai_install::run_security_cleanup;

    println!("🔒 Running security cleanup...");
    run_security_cleanup(false)?;
    Ok(())
}

// =============================================================================
// MEGA SESSION HANDLERS
// =============================================================================

/// Start a new mega session
pub async fn handle_mega_start(name: Option<&str>) -> Result<()> {
    use crate::mega_session_manager::MegaSessionManager;

    let mut manager = MegaSessionManager::new()?;
    let session_id = manager.start_session(name.map(|s| s.to_string()))?;

    println!("🚀 Mega session started!");
    println!("   ID: {}", session_id);
    if let Some(n) = name {
        if !n.is_empty() {
            println!("   Name: {}", n);
        }
    }
    println!("\n💡 Use 'st --mega-save' to save session state");
    println!("   Use 'st --mega-list' to see all sessions");

    Ok(())
}

/// Save current mega session
pub async fn handle_mega_save() -> Result<()> {
    use crate::mega_session_manager::MegaSessionManager;

    let manager = MegaSessionManager::new()?;
    manager.save_current_session()?;
    println!("💾 Mega session saved!");

    Ok(())
}

/// List all mega sessions
pub async fn handle_mega_list() -> Result<()> {
    use crate::mega_session_manager::MegaSessionManager;

    let manager = MegaSessionManager::new()?;
    let sessions = manager.list_sessions()?;

    if sessions.is_empty() {
        println!("📋 No mega sessions found");
        println!("\n💡 Start one with: st --mega-start [NAME]");
    } else {
        println!("📋 Mega Sessions ({}):", sessions.len());
        println!("{}", "─".repeat(45));
        for (i, session) in sessions.iter().enumerate() {
            println!("  [{}] {}", i + 1, session);
        }
    }

    Ok(())
}

/// Show mega session statistics
pub async fn handle_mega_stats() -> Result<()> {
    use crate::mega_session_manager::MegaSessionManager;

    let manager = MegaSessionManager::new()?;
    println!("{}", manager.get_stats());

    Ok(())
}

// =============================================================================
// AI GUARDIAN - System-wide protection daemon
// =============================================================================

/// Run the Guardian daemon (called by systemd service)
pub async fn run_guardian_daemon() -> Result<()> {
    use crate::ai_guardian::AiGuardian;
    use tokio::time::{sleep, Duration};

    println!(
        r#"
╔═══════════════════════════════════════════════════════════════════════════════╗
║                                                                               ║
║    🛡️  SMART TREE GUARDIAN - System-wide AI Protection Active 🛡️            ║
║                                                                               ║
╚═══════════════════════════════════════════════════════════════════════════════╝
"#
    );

    let guardian = AiGuardian::new();
    let watch_paths = vec!["/home", "/tmp", "/var/tmp"];

    println!("Watching paths: {:?}", watch_paths);
    println!("Press Ctrl+C to stop.\n");

    loop {
        for watch_path in &watch_paths {
            let path = std::path::Path::new(watch_path);
            if !path.exists() {
                continue;
            }
            scan_directory_for_threats(&guardian, path);
        }

        sleep(Duration::from_secs(60)).await;
    }
}

/// Scan a directory for prompt injection threats
pub fn scan_directory_for_threats(
    guardian: &crate::ai_guardian::AiGuardian,
    path: &std::path::Path,
) {
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

        if entry_path
            .file_name()
            .map(|n| n.to_string_lossy().starts_with('.'))
            .unwrap_or(false)
        {
            continue;
        }

        if let Some(ext) = entry_path.extension() {
            let ext_str = ext.to_string_lossy().to_lowercase();
            if suspicious_extensions.contains(&ext_str.as_str()) {
                let threats = guardian.scan_file(entry_path);

                for threat in threats {
                    if matches!(
                        threat.level,
                        crate::ai_guardian::ThreatLevel::Critical
                            | crate::ai_guardian::ThreatLevel::Dangerous
                    ) {
                        eprintln!(
                            "⚠️  THREAT DETECTED [{:?}] in {}: {}",
                            threat.level, threat.location, threat.pattern
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
pub fn handle_guardian_scan(file_path: &std::path::Path) -> Result<()> {
    use crate::ai_guardian::{AiGuardian, ThreatLevel};

    println!(
        r#"
╔═══════════════════════════════════════════════════════════════════════════════╗
║            🛡️  SMART TREE GUARDIAN - File Scan 🛡️                            ║
╚═══════════════════════════════════════════════════════════════════════════════╝
"#
    );

    println!("Scanning: {}\n", file_path.display());

    let guardian = AiGuardian::new();
    let threats = guardian.scan_file(file_path);

    if threats.is_empty() {
        println!("✅ No threats detected. File appears safe.");
        return Ok(());
    }

    let critical = threats
        .iter()
        .filter(|t| t.level == ThreatLevel::Critical)
        .count();
    let dangerous = threats
        .iter()
        .filter(|t| t.level == ThreatLevel::Dangerous)
        .count();
    let suspicious = threats
        .iter()
        .filter(|t| t.level == ThreatLevel::Suspicious)
        .count();

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

/// Handle Google Drive CLI commands
pub async fn handle_gdrive_command(action: &str) -> Result<()> {
    #[cfg(not(feature = "google"))]
    {
        let _ = action;
        eprintln!(
            "Google Drive support is not enabled in this build. Recompile with --features google"
        );
        std::process::exit(1);
    }

    #[cfg(feature = "google")]
    {
        match action {
            "status" => {
                let auth = crate::google_sync::auth::GoogleAuth::default_store()?;
                let status = auth.status_summary();
                println!(
                    "🌐 Google Drive / Gmail Auth Status:\n===================================\n{}",
                    status
                );
                Ok(())
            }
            "login" => {
                println!("🔑 To authenticate Google Drive / Gmail, use the MCP 'google' tool with operation: 'auth_login',\n   or place OAuth2 credentials in ~/.st/google/config.json");
                Ok(())
            }
            "logout" => {
                let auth = crate::google_sync::auth::GoogleAuth::default_store()?;
                auth.logout()?;
                println!("✅ Google credentials removed from ~/.st/google/");
                Ok(())
            }
            "list" => {
                let auth = crate::google_sync::auth::GoogleAuth::default_store()?;
                if !auth.has_cached_tokens() {
                    println!("Not authenticated with Google. Please configure ~/.st/google/ or authenticate via MCP.");
                    return Ok(());
                }
                let authenticator =
                    crate::mcp::google::gmail_tools::rebuild_authenticator(&auth).await?;
                let drive = crate::google_sync::drive_client::DriveClient::new(authenticator);
                let (files, _) = drive.list_files("root", None).await?;
                println!(
                    "📁 Google Drive Root Files ({} found):\n=====================================",
                    files.len()
                );
                for f in &files {
                    let kind = if f.is_folder { "[DIR] " } else { "      " };
                    println!("  • {}{} ({})", kind, f.name, f.mime_type);
                }
                Ok(())
            }
            "sync" => {
                let auth = crate::google_sync::auth::GoogleAuth::default_store()?;
                if !auth.has_cached_tokens() {
                    println!("Not authenticated with Google. Please configure ~/.st/google/ or authenticate via MCP.");
                    return Ok(());
                }
                let authenticator =
                    crate::mcp::google::gmail_tools::rebuild_authenticator(&auth).await?;
                let drive = crate::google_sync::drive_client::DriveClient::new(authenticator);
                let engine = crate::google_sync::sync_engine::SyncEngine::new(&drive)?;
                let mut state = crate::google_sync::models::SyncState {
                    sync_id: "default_sync".to_string(),
                    local_path: ".".to_string(),
                    drive_folder_id: "root".to_string(),
                    last_sync: None,
                    direction: crate::google_sync::models::SyncDirection::Bidirectional,
                    file_states: Vec::new(),
                    conflict_resolution: crate::google_sync::models::ConflictStrategy::NewerWins,
                };
                let report = engine.sync(&mut state).await?;
                println!("🔄 Google Drive Sync Complete:\n  Uploaded: {}\n  Downloaded: {}\n  Conflicts: {}\n  Skipped: {}\n  Duration: {}ms",
                    report.uploaded, report.downloaded, report.conflicts, report.skipped, report.duration_ms);
                Ok(())
            }
            _ => {
                eprintln!(
                    "Unknown gdrive action: '{}'. Valid actions: status, login, logout, list, sync",
                    action
                );
                std::process::exit(1);
            }
        }
    }
}

/// Handle Voice CLI commands
pub async fn handle_voice_command(action: &str) -> Result<()> {
    #[cfg(not(feature = "voice"))]
    {
        let _ = action;
        eprintln!("Voice support is not enabled in this build. Recompile with --features voice");
        std::process::exit(1);
    }

    #[cfg(feature = "voice")]
    {
        if action == "status" {
            let vad = crate::vad_marine::MarineVAD::new()?;
            let is_active = vad.is_voice_active().await;
            let salience = vad.get_salience().await;
            let quality = vad.get_voice_quality().await;
            println!("🎙️ Voice Activity & Marine VAD Status\n====================================");
            println!(
                "Status: {}",
                if is_active {
                    "Active (Voice Detected)"
                } else {
                    "Idle / Ambient"
                }
            );
            println!("Salience: {:.1}%", salience * 100.0);
            println!("Harmonicity: {:.3}", quality.harmonicity);
            println!("Zero Crossing Rate: {:.3}", quality.zero_crossing_rate);
            println!("Energy Variance: {:.3}", quality.energy_variance);
            println!("Spectral Tilt: {:.3}", quality.spectral_tilt);
            Ok(())
        } else if action == "test" {
            let vad = crate::vad_marine::MarineVAD::new()?;
            let sample_rate = 16000;
            let frequency = 220.0;
            let duration = 0.2;
            let num_samples = (sample_rate as f64 * duration) as usize;
            let mut samples = vec![0.0f32; num_samples];
            for (i, sample) in samples.iter_mut().enumerate() {
                let t = i as f64 / sample_rate as f64;
                *sample = (2.0 * std::f64::consts::PI * frequency * t).sin() as f32 * 0.6;
            }
            let is_voice = vad.process_audio(&samples, sample_rate).await?;
            let salience = vad.get_salience().await;
            println!("🧪 Marine VAD Test Run:\n  Voice detected: {}\n  Salience: {:.1}%\n  Algorithm: MEM8 Marine Salience", is_voice, salience * 100.0);
            Ok(())
        } else if action.starts_with("speak") {
            let text = action.strip_prefix("speak").unwrap_or("").trim();
            let text = if text.is_empty() {
                "Smart Tree voice output initialized"
            } else {
                text
            };
            let wav = crate::web_dashboard::voice::generate_speech_wav(text, "aye");
            println!(
                "🔊 Synthesized speech WAV: {} bytes (16kHz PCM mono, persona 'aye')",
                wav.len()
            );
            Ok(())
        } else {
            eprintln!(
                "Unknown voice action: '{}'. Valid actions: status, test, speak [text]",
                action
            );
            std::process::exit(1);
        }
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, Default)]
struct RepoSyncRecord {
    pushed_at: String,
    updated_at: String,
    last_synced_at: String,
    head_sha: String,
}

#[derive(Debug, Clone, serde::Deserialize)]
struct GhRepoEntry {
    name: String,
    #[serde(rename = "pushedAt", default)]
    pushed_at: Option<String>,
    #[serde(rename = "updatedAt", default)]
    updated_at: Option<String>,
    #[serde(default)]
    url: Option<String>,
}

/// Sync all GitHub repositories for a user/organization into `<dir>/<user>/<repo>`:
/// - Uses `gh repo list -L <limit> <user>` to discover repositories and their latest `pushedAt`/`updatedAt` timestamps.
/// - Clones missing repositories with standard `git clone`.
/// - Only fast-forwards clean, attached branches with an upstream and no local commits.
/// - Timestamp cache hits are skipped, never reported as verified up-to-date.
pub async fn handle_sync_user(cmd: &crate::cli::SyncUserCommand) -> Result<()> {
    let raw_user = cmd
        .user
        .trim()
        .trim_start_matches('@')
        .trim_start_matches("https://github.com/")
        .trim_end_matches('/');
    let user: String = raw_user
        .chars()
        .filter(|c| c.is_ascii_alphanumeric() || *c == '-' || *c == '_' || *c == '.')
        .collect();
    if user.is_empty() || user == "." || user == ".." {
        anyhow::bail!("Invalid GitHub username/organization: '{}'", cmd.user);
    }

    let user_dir = cmd.dir.join(&user);
    std::fs::create_dir_all(&user_dir)
        .with_context(|| format!("Failed to create target directory {}", user_dir.display()))?;
    let user_dir = std::fs::canonicalize(user_dir)?;
    let _sync_lock = lock_sync_destination(&user_dir)?;

    let state_path = user_dir.join(".st-sync-state.json");
    let initial_state: std::collections::BTreeMap<String, RepoSyncRecord> =
        match std::fs::read(&state_path) {
            Ok(bytes) => serde_json::from_slice(&bytes).with_context(|| {
                format!(
                    "Invalid sync cache {}; preserve it and review manually",
                    state_path.display()
                )
            })?,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
                std::collections::BTreeMap::new()
            }
            Err(error) => {
                return Err(error)
                    .with_context(|| format!("Cannot read sync cache {}", state_path.display()))
            }
        };

    println!(
        "🌳 Smart Tree GitHub Sync: fetching up to {} repositories for '{}' → {} ...",
        cmd.limit,
        user,
        user_dir.display()
    );

    let mut gh_args = vec![
        "repo".to_string(),
        "list".to_string(),
        user.clone(),
        "-L".to_string(),
        cmd.limit.to_string(),
    ];
    if cmd.no_forks {
        gh_args.push("--source".to_string());
    }
    if cmd.no_archived {
        gh_args.push("--no-archived".to_string());
    }
    gh_args.push("--json".to_string());
    gh_args.push("name,pushedAt,updatedAt,url".to_string());

    let gh_output = std::process::Command::new("gh")
        .current_dir(&user_dir)
        .args(&gh_args)
        .output()
        .context("Failed to execute `gh repo list`. Ensure GitHub CLI (`gh`) is installed and authenticated.")?;

    let repos: Vec<GhRepoEntry> = if gh_output.status.success() {
        serde_json::from_slice(&gh_output.stdout)
            .context("Failed to parse JSON output from `gh repo list`")?
    } else {
        // Fallback to plain `gh repo list <user> -L <limit>` TSV parsing
        let mut fallback_args = vec![
            "repo".to_string(),
            "list".to_string(),
            user.clone(),
            "-L".to_string(),
            cmd.limit.to_string(),
        ];
        if cmd.no_forks {
            fallback_args.push("--source".to_string());
        }
        if cmd.no_archived {
            fallback_args.push("--no-archived".to_string());
        }
        let fb = std::process::Command::new("gh")
            .current_dir(&user_dir)
            .args(&fallback_args)
            .output()
            .context("Failed to execute fallback `gh repo list`")?;
        if !fb.status.success() {
            let err = String::from_utf8_lossy(&gh_output.stderr);
            anyhow::bail!("`gh repo list` failed: {}", err.trim());
        }
        let text = String::from_utf8_lossy(&fb.stdout);
        text.lines()
            .filter_map(|line| {
                let cols: Vec<&str> = line.split('\t').collect();
                let full = cols.first()?.trim();
                let name = full.split('/').next_back()?.trim();
                if name.is_empty() {
                    return None;
                }
                let updated = cols.get(3).map(|s| s.trim().to_string());
                Some(GhRepoEntry {
                    name: name.to_string(),
                    pushed_at: updated.clone(),
                    updated_at: updated,
                    url: Some(format!("https://github.com/{}/{}", user, name)),
                })
            })
            .collect()
    };

    let total = repos.len();
    if total == 0 {
        println!("No repositories found for '{}'.", user);
        return Ok(());
    }
    println!(
        "📋 Found {} repositories for '{}'. Starting sync ({} workers)...",
        total,
        user,
        cmd.jobs.clamp(1, 16)
    );

    let state = std::sync::Arc::new(std::sync::Mutex::new(initial_state));
    let queue = std::sync::Arc::new(std::sync::Mutex::new(
        repos
            .into_iter()
            .enumerate()
            .collect::<std::collections::VecDeque<(usize, GhRepoEntry)>>(),
    ));
    let cloned = std::sync::Arc::new(std::sync::atomic::AtomicUsize::new(0));
    let refreshed = std::sync::Arc::new(std::sync::atomic::AtomicUsize::new(0));
    let up_to_date = std::sync::Arc::new(std::sync::atomic::AtomicUsize::new(0));
    let failed = std::sync::Arc::new(std::sync::atomic::AtomicUsize::new(0));
    let skipped = std::sync::Arc::new(std::sync::atomic::AtomicUsize::new(0));
    let blocked = std::sync::Arc::new(std::sync::atomic::AtomicUsize::new(0));
    let fetched_only = std::sync::Arc::new(std::sync::atomic::AtomicUsize::new(0));

    let num_workers = cmd.jobs.clamp(1, 16);
    let refresh_all = cmd.refresh_all;
    let mut handles = Vec::with_capacity(num_workers);

    for _ in 0..num_workers {
        let queue = std::sync::Arc::clone(&queue);
        let state = std::sync::Arc::clone(&state);
        let cloned = std::sync::Arc::clone(&cloned);
        let refreshed = std::sync::Arc::clone(&refreshed);
        let up_to_date = std::sync::Arc::clone(&up_to_date);
        let failed = std::sync::Arc::clone(&failed);
        let skipped = std::sync::Arc::clone(&skipped);
        let blocked = std::sync::Arc::clone(&blocked);
        let fetched_only = std::sync::Arc::clone(&fetched_only);
        let user = user.clone();
        let user_dir = user_dir.clone();

        handles.push(std::thread::spawn(move || loop {
            let item = {
                let mut q = queue.lock().unwrap();
                q.pop_front()
            };
            let Some((idx, repo)) = item else {
                break;
            };
            if repo.name.is_empty()
                || repo.name.contains('/')
                || repo.name.contains('\\')
                || repo.name == "."
                || repo.name == ".."
                || repo.name.starts_with(".st-sync-")
            {
                failed.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                eprintln!("Invalid repository name: {:?}", repo.name);
                continue;
            }

            let repo_dir = user_dir.join(&repo.name);
            let clone_url = repo
                .url
                .as_deref()
                .map(|u| {
                    if u.ends_with(".git") {
                        u.to_string()
                    } else {
                        format!("{}.git", u)
                    }
                })
                .unwrap_or_else(|| format!("https://github.com/{}/{}.git", user, repo.name));

            let pushed_at = repo.pushed_at.unwrap_or_default();
            let updated_at = repo.updated_at.unwrap_or_default();

            let cached = state.lock().unwrap().get(&repo.name).cloned();
            let result = sync_repository(
                &repo_dir,
                &clone_url,
                cached.as_ref(),
                &pushed_at,
                &updated_at,
                refresh_all,
            );
            let mut guard = state.lock().unwrap();
            match result {
                Ok(outcome) => {
                    match &outcome {
                        RepoSyncOutcome::Cloned(_) => &cloned,
                        RepoSyncOutcome::Updated(_) => &refreshed,
                        RepoSyncOutcome::UpToDate(_) => &up_to_date,
                        RepoSyncOutcome::Skipped => &skipped,
                        RepoSyncOutcome::Blocked { fetched, .. } => {
                            if *fetched {
                                fetched_only.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                            }
                            &blocked
                        }
                    }
                    .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                    println!(
                        "[{}/{}] {}/{}: {:?}",
                        idx + 1,
                        total,
                        user,
                        repo.name,
                        outcome
                    );
                    update_repo_sync_record(
                        &mut guard,
                        &repo.name,
                        &pushed_at,
                        &updated_at,
                        &outcome,
                    );
                }
                Err(error) => {
                    guard.remove(&repo.name);
                    failed.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                    eprintln!(
                        "[{}/{}] Failed {}/{}: {error:#}",
                        idx + 1,
                        total,
                        user,
                        repo.name
                    );
                }
            }
        }));
    }

    let mut worker_panicked = false;
    for h in handles {
        worker_panicked |= h.join().is_err();
    }
    anyhow::ensure!(!worker_panicked, "A GitHub sync worker panicked");

    // The destination lock covers both repository operations and atomic cache replacement.
    let guard = state
        .lock()
        .map_err(|_| anyhow::anyhow!("Sync state lock poisoned"))?;
    let pending = state_path.with_extension("json.tmp");
    std::fs::write(&pending, serde_json::to_vec_pretty(&*guard)?)?;
    std::fs::rename(&pending, &state_path)?;

    let c = cloned.load(std::sync::atomic::Ordering::Relaxed);
    let r = refreshed.load(std::sync::atomic::Ordering::Relaxed);
    let u = up_to_date.load(std::sync::atomic::Ordering::Relaxed);
    let f = failed.load(std::sync::atomic::Ordering::Relaxed);

    println!(
        "\nGitHub Sync Complete for '{}' ({}):\n   Cloned: {}\n   Updated: {}\n   Up-to-date (verified): {}\n   Failed: {}\n   Skipped (cached): {}\n   Blocked: {}\n   Fetched-only (subset of blocked): {}\n   State file: {}",
        user,
        user_dir.display(),
        c,
        r,
        u,
        f,
        skipped.load(std::sync::atomic::Ordering::Relaxed),
        blocked.load(std::sync::atomic::Ordering::Relaxed),
        fetched_only.load(std::sync::atomic::Ordering::Relaxed),
        state_path.display()
    );
    anyhow::ensure!(f == 0, "GitHub sync failed for {f} repositories");
    Ok(())
}

struct SyncDestinationLock(std::fs::File);

impl Drop for SyncDestinationLock {
    fn drop(&mut self) {
        // Explicitly release ownership even if a concurrently forked child briefly
        // retains the descriptor before exec closes it.
        if let Err(error) = self.0.unlock() {
            tracing::warn!(%error, "Could not release sync destination lock");
        }
    }
}

fn lock_sync_destination(dir: &std::path::Path) -> Result<SyncDestinationLock> {
    // Keep the inode in place: removing a lock file could allow two owners.
    let lock = std::fs::OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .truncate(false)
        .open(dir.join(".st-sync-lock"))?;
    lock.try_lock()
        .with_context(|| format!("Sync destination {} is already in use", dir.display()))?;
    Ok(SyncDestinationLock(lock))
}

#[derive(Debug, PartialEq, Eq)]
enum RepoSyncOutcome {
    Cloned(String),
    Updated(String),
    UpToDate(String),
    Skipped,
    Blocked { reason: String, fetched: bool },
}

fn update_repo_sync_record(
    state: &mut std::collections::BTreeMap<String, RepoSyncRecord>,
    repo_name: &str,
    pushed_at: &str,
    updated_at: &str,
    outcome: &RepoSyncOutcome,
) {
    match outcome {
        RepoSyncOutcome::Cloned(head_sha)
        | RepoSyncOutcome::Updated(head_sha)
        | RepoSyncOutcome::UpToDate(head_sha) => {
            state.insert(
                repo_name.to_string(),
                RepoSyncRecord {
                    pushed_at: pushed_at.to_string(),
                    updated_at: updated_at.to_string(),
                    last_synced_at: chrono::Utc::now().to_rfc3339(),
                    head_sha: head_sha.to_string(),
                },
            );
        }
        RepoSyncOutcome::Blocked { .. } => {
            state.remove(repo_name);
        }
        RepoSyncOutcome::Skipped => {}
    }
}

fn sync_git(dir: &std::path::Path, args: &[&str]) -> Result<String> {
    let output = std::process::Command::new("git")
        .current_dir(dir)
        .env("GIT_TERMINAL_PROMPT", "0")
        .env("GIT_OPTIONAL_LOCKS", "0")
        .arg("-C")
        .arg(dir)
        .args(args)
        .output()
        .context("Failed to execute git")?;
    anyhow::ensure!(
        output.status.success(),
        "git {}: {}",
        args.join(" "),
        String::from_utf8_lossy(&output.stderr).trim()
    );
    Ok(String::from_utf8(output.stdout)?.trim().to_string())
}

// An error here is a blocked reconciliation, never permission to try a pull.
fn sync_ready_branch(dir: &std::path::Path) -> Result<(String, String)> {
    let root = sync_git(dir, &["rev-parse", "--show-toplevel"])?;
    anyhow::ensure!(
        std::fs::canonicalize(root)? == std::fs::canonicalize(dir)?,
        "Not a repository root"
    );
    anyhow::ensure!(
        sync_git(
            dir,
            &[
                "status",
                "--porcelain",
                "--untracked-files=all",
                "--ignore-submodules=none"
            ]
        )?
        .is_empty(),
        "Dirty working tree or index"
    );
    sync_git(dir, &["symbolic-ref", "--quiet", "HEAD"]).context("Detached HEAD")?;
    for marker in [
        "MERGE_HEAD",
        "CHERRY_PICK_HEAD",
        "REVERT_HEAD",
        "rebase-merge",
        "rebase-apply",
        "sequencer",
        "BISECT_LOG",
    ] {
        let path = sync_git(dir, &["rev-parse", "--git-path", marker])?;
        anyhow::ensure!(
            !dir.join(path).exists(),
            "Git operation in progress: {marker}"
        );
    }
    let head = sync_git(dir, &["rev-parse", "--verify", "HEAD"])?;
    let upstream =
        sync_git(dir, &["rev-parse", "--verify", "@{upstream}"]).context("No usable upstream")?;
    let ahead = sync_git(
        dir,
        &["rev-list", "--count", &format!("{upstream}..{head}")],
    )?
    .parse::<u64>()?;
    anyhow::ensure!(
        ahead == 0,
        "Branch is ahead of or diverged from upstream ({ahead} local commits)"
    );
    Ok((head, upstream))
}

fn sync_repository(
    dir: &std::path::Path,
    clone_url: &str,
    cached: Option<&RepoSyncRecord>,
    pushed_at: &str,
    updated_at: &str,
    refresh_all: bool,
) -> Result<RepoSyncOutcome> {
    let absolute = std::path::absolute(dir)?;
    let dir = absolute.as_path();
    if !dir.join(".git").exists() {
        if dir.exists() {
            anyhow::ensure!(dir.is_dir() && std::fs::read_dir(dir)?.next().is_none(),
                "Existing path {} is not an empty Git clone destination; review or move its contents manually", dir.display());
        }
        let output = std::process::Command::new("git")
            .current_dir(dir.parent().context("Clone destination has no parent")?)
            .env("GIT_TERMINAL_PROMPT", "0")
            .args(["clone", "--quiet", "--"])
            .arg(clone_url)
            .arg(dir)
            .output()?;
        anyhow::ensure!(
            output.status.success(),
            "Clone failed: {}",
            String::from_utf8_lossy(&output.stderr).trim()
        );
        // Empty remotes may clone successfully but have no branch to reconcile.
        return Ok(match sync_ready_branch(dir) {
            Ok((head, upstream)) if head == upstream => RepoSyncOutcome::Cloned(head),
            result => RepoSyncOutcome::Blocked {
                reason: format!("Clone has no reconciled branch: {result:?}"),
                fetched: false,
            },
        });
    }
    let (head, upstream) = match sync_ready_branch(dir) {
        Ok(state) => state,
        Err(error) => {
            return Ok(RepoSyncOutcome::Blocked {
                reason: format!("{error:#}"),
                fetched: false,
            })
        }
    };
    // Timestamps are only an optimization, not evidence of remote equality.
    if !refresh_all
        && head == upstream
        && !pushed_at.is_empty()
        && !updated_at.is_empty()
        && cached.is_some_and(|rec| {
            rec.head_sha == head && rec.pushed_at == pushed_at && rec.updated_at == updated_at
        })
    {
        return Ok(RepoSyncOutcome::Skipped);
    }
    let branch = sync_git(dir, &["symbolic-ref", "--short", "HEAD"])?;
    let remote = sync_git(
        dir,
        &["config", "--get", &format!("branch.{branch}.remote")],
    )?;
    // Fetch the actual upstream remote even if it opts out of `fetch --all`.
    sync_git(dir, &["fetch", "--prune", "--quiet", "--", &remote])?;
    let (before, target) = match sync_ready_branch(dir) {
        Ok(state) => state,
        Err(error) => {
            return Ok(RepoSyncOutcome::Blocked {
                reason: format!("{error:#}"),
                fetched: true,
            })
        }
    };
    if before != head {
        return Ok(RepoSyncOutcome::Blocked {
            reason: "HEAD changed during fetch".into(),
            fetched: true,
        });
    }
    if before != target {
        // Merge the inspected commit, avoiding a second implicit fetch by `pull`.
        sync_git(
            dir,
            &[
                "-c",
                "merge.autostash=false",
                "merge",
                "--ff-only",
                "--quiet",
                &target,
            ],
        )?;
    }
    let (after, upstream) = sync_ready_branch(dir)?;
    anyhow::ensure!(
        after == target && after == upstream,
        "Branch changed during reconciliation"
    );
    Ok(if before == after {
        RepoSyncOutcome::UpToDate(after)
    } else {
        RepoSyncOutcome::Updated(after)
    })
}

#[cfg(test)]
mod sync_tests {
    use super::*;
    use std::path::{Path, PathBuf};

    fn git(dir: &Path, args: &[&str]) -> String {
        let output = std::process::Command::new("git")
            .current_dir(dir)
            .env("GIT_CONFIG_NOSYSTEM", "1")
            .env("GIT_CONFIG_GLOBAL", "/dev/null")
            .arg("-C")
            .arg(dir)
            .args([
                "-c",
                "user.name=Sync Fixture",
                "-c",
                "user.email=sync@example.invalid",
                "-c",
                "commit.gpgsign=false",
            ])
            .args(args)
            .output()
            .unwrap();
        assert!(
            output.status.success(),
            "{args:?}: {}",
            String::from_utf8_lossy(&output.stderr)
        );
        String::from_utf8(output.stdout).unwrap().trim().to_owned()
    }

    fn temp_dir() -> tempfile::TempDir {
        let target = Path::new(env!("CARGO_MANIFEST_DIR")).join("target/sync-fixtures");
        std::fs::create_dir_all(&target).unwrap();
        tempfile::tempdir_in(target).unwrap()
    }

    struct Fixture {
        _temp: tempfile::TempDir,
        remote: PathBuf,
        publisher: PathBuf,
        local: PathBuf,
    }

    impl Fixture {
        fn new() -> Self {
            let temp = temp_dir();
            let remote = temp.path().join("remote.git");
            let publisher = temp.path().join("publisher");
            let local = temp.path().join("local");
            git(
                temp.path(),
                &["init", "--bare", "-b", "main", remote.to_str().unwrap()],
            );
            git(
                temp.path(),
                &[
                    "clone",
                    remote.to_str().unwrap(),
                    publisher.to_str().unwrap(),
                ],
            );
            Self::commit(&publisher, "base");
            git(&publisher, &["push", "-u", "origin", "main"]);
            git(
                temp.path(),
                &["clone", remote.to_str().unwrap(), local.to_str().unwrap()],
            );
            Self {
                _temp: temp,
                remote,
                publisher,
                local,
            }
        }

        fn commit(dir: &Path, name: &str) {
            std::fs::write(dir.join(name), name).unwrap();
            git(dir, &["add", name]);
            git(dir, &["commit", "-m", name]);
        }

        fn advance(&self) {
            Self::commit(&self.publisher, "remote-change");
            git(&self.publisher, &["push"]);
        }

        fn sync(&self, cached: Option<&RepoSyncRecord>, refresh: bool) -> Result<RepoSyncOutcome> {
            sync_repository(
                &self.local,
                self.remote.to_str().unwrap(),
                cached,
                "pushed",
                "updated",
                refresh,
            )
        }

        fn record(&self) -> RepoSyncRecord {
            RepoSyncRecord {
                pushed_at: "pushed".into(),
                updated_at: "updated".into(),
                head_sha: git(&self.local, &["rev-parse", "HEAD"]),
                last_synced_at: "old-receipt".into(),
            }
        }
    }

    #[test]
    fn clean_clone_fast_forward_and_verified_equal() {
        let f = Fixture::new();
        let destination = f._temp.path().join("new-clone");
        assert!(matches!(
            sync_repository(
                &destination,
                f.remote.to_str().unwrap(),
                None,
                "p",
                "u",
                false
            )
            .unwrap(),
            RepoSyncOutcome::Cloned(_)
        ));
        f.advance();
        let target = git(&f.publisher, &["rev-parse", "HEAD"]);
        assert_eq!(
            f.sync(None, false).unwrap(),
            RepoSyncOutcome::Updated(target.clone())
        );
        assert_eq!(git(&f.local, &["rev-parse", "HEAD"]), target);
        assert_eq!(
            f.sync(None, false).unwrap(),
            RepoSyncOutcome::UpToDate(target)
        );
    }

    #[test]
    fn dirty_staged_and_untracked_work_survives_even_with_matching_cache() {
        for mode in ["modified", "staged", "untracked"] {
            let f = Fixture::new();
            let record = f.record();
            f.advance();
            let file = if mode == "untracked" {
                "private"
            } else {
                "base"
            };
            std::fs::write(f.local.join(file), "preserve me").unwrap();
            if mode == "staged" {
                git(&f.local, &["add", file]);
            }
            let status = git(&f.local, &["status", "--porcelain"]);
            assert!(matches!(
                f.sync(Some(&record), false).unwrap(),
                RepoSyncOutcome::Blocked { fetched: false, .. }
            ));
            assert_eq!(git(&f.local, &["rev-parse", "HEAD"]), record.head_sha);
            assert_eq!(git(&f.local, &["status", "--porcelain"]), status);
            assert_eq!(
                std::fs::read_to_string(f.local.join(file)).unwrap(),
                "preserve me"
            );
        }
    }

    #[test]
    fn detached_missing_upstream_ahead_and_diverged_are_blocked() {
        for mode in ["detached", "no-upstream", "ahead", "diverged"] {
            let f = Fixture::new();
            match mode {
                "detached" => {
                    git(&f.local, &["checkout", "--detach"]);
                }
                "no-upstream" => {
                    git(&f.local, &["branch", "--unset-upstream"]);
                }
                _ => {
                    Fixture::commit(&f.local, "local-change");
                }
            }
            f.advance();
            if mode == "diverged" {
                git(&f.local, &["fetch"]);
            }
            let head = git(&f.local, &["rev-parse", "HEAD"]);
            let outcome = f.sync(None, true).unwrap();
            assert!(
                matches!(outcome, RepoSyncOutcome::Blocked { fetched: false, .. }),
                "{mode}: {outcome:?}"
            );
            assert_eq!(git(&f.local, &["rev-parse", "HEAD"]), head);
        }
    }

    #[test]
    fn remote_rewrite_is_fetched_only_and_invalidates_success_cache() {
        let f = Fixture::new();
        let record = f.record();
        git(&f.publisher, &["checkout", "--orphan", "rewritten"]);
        Fixture::commit(&f.publisher, "rewrite");
        git(&f.publisher, &["push", "--force", "origin", "HEAD:main"]);
        let outcome = f.sync(Some(&record), true).unwrap();
        assert!(matches!(
            outcome,
            RepoSyncOutcome::Blocked { fetched: true, .. }
        ));
        assert_eq!(git(&f.local, &["rev-parse", "HEAD"]), record.head_sha);
        assert_eq!(
            git(&f.local, &["rev-parse", "origin/main"]),
            git(&f.publisher, &["rev-parse", "HEAD"])
        );
        let mut state = std::collections::BTreeMap::from([("local".into(), record)]);
        update_repo_sync_record(&mut state, "local", "new-pushed", "new-updated", &outcome);
        assert!(state.is_empty());
    }

    #[test]
    fn cache_hits_are_skips_and_local_reset_is_reconciled() {
        let f = Fixture::new();
        let record = f.record();
        f.advance();
        assert_eq!(
            f.sync(Some(&record), false).unwrap(),
            RepoSyncOutcome::Skipped
        );
        let mut state = std::collections::BTreeMap::from([("local".into(), record.clone())]);
        update_repo_sync_record(
            &mut state,
            "local",
            "pushed",
            "updated",
            &RepoSyncOutcome::Skipped,
        );
        assert_eq!(state["local"].last_synced_at, "old-receipt");
        assert!(matches!(
            f.sync(Some(&record), true).unwrap(),
            RepoSyncOutcome::Updated(_)
        ));
        let newer = f.record();
        git(&f.local, &["reset", "--hard", &record.head_sha]);
        assert_eq!(
            f.sync(Some(&newer), false).unwrap(),
            RepoSyncOutcome::Updated(newer.head_sha)
        );
    }

    #[test]
    fn missing_timestamps_and_fetch_failures_do_not_claim_success() {
        let f = Fixture::new();
        f.advance();
        let record = f.record();
        assert!(matches!(
            sync_repository(&f.local, "unused", Some(&record), "", "", false).unwrap(),
            RepoSyncOutcome::Updated(_)
        ));
        let head = git(&f.local, &["rev-parse", "HEAD"]);
        git(
            &f.local,
            &[
                "remote",
                "set-url",
                "origin",
                f._temp.path().join("missing.git").to_str().unwrap(),
            ],
        );
        assert!(f.sync(None, true).is_err());
        assert_eq!(git(&f.local, &["rev-parse", "HEAD"]), head);
    }

    #[test]
    fn upstream_remote_is_fetched_even_when_excluded_from_fetch_all() {
        let f = Fixture::new();
        git(&f.local, &["config", "remote.origin.skipFetchAll", "true"]);
        f.advance();
        assert_eq!(
            f.sync(None, true).unwrap(),
            RepoSyncOutcome::Updated(git(&f.publisher, &["rev-parse", "HEAD"]))
        );
    }

    #[test]
    fn in_progress_operation_and_empty_clone_do_not_get_success_receipts() {
        let f = Fixture::new();
        std::fs::create_dir(f.local.join(".git/rebase-merge")).unwrap();
        assert!(matches!(
            f.sync(None, true).unwrap(),
            RepoSyncOutcome::Blocked { fetched: false, .. }
        ));
        let empty = f._temp.path().join("empty.git");
        git(
            f._temp.path(),
            &["init", "--bare", "-b", "main", empty.to_str().unwrap()],
        );
        assert!(matches!(
            sync_repository(
                &f._temp.path().join("empty-clone"),
                empty.to_str().unwrap(),
                None,
                "p",
                "u",
                false
            )
            .unwrap(),
            RepoSyncOutcome::Blocked { .. }
        ));
    }

    #[tokio::test]
    async fn corrupt_cache_is_preserved_and_lock_released_on_error() {
        let temp = temp_dir();
        let owner = temp.path().join("owner");
        std::fs::create_dir(&owner).unwrap();
        let cache = owner.join(".st-sync-state.json");
        std::fs::write(&cache, "broken cache").unwrap();
        let cmd = crate::cli::SyncUserCommand {
            user: "owner".into(),
            dir: temp.path().to_path_buf(),
            limit: 1,
            jobs: 1,
            refresh_all: false,
            no_forks: false,
            no_archived: false,
        };
        assert!(handle_sync_user(&cmd)
            .await
            .unwrap_err()
            .to_string()
            .contains("Invalid sync cache"));
        assert_eq!(std::fs::read_to_string(cache).unwrap(), "broken cache");
        assert!(lock_sync_destination(&owner).is_ok());
    }

    #[cfg(unix)]
    #[test]
    #[ignore = "subprocess probe used by summary_and_persisted_receipts_match_actual_results"]
    fn sync_handler_child() {
        let base = std::env::var_os("ST_SYNC_TEST_BASE").unwrap();
        let cmd = crate::cli::SyncUserCommand {
            user: "fixture-owner".into(),
            dir: PathBuf::from(base),
            limit: 100,
            jobs: 4,
            refresh_all: false,
            no_forks: false,
            no_archived: false,
        };
        let result = tokio::runtime::Runtime::new()
            .unwrap()
            .block_on(handle_sync_user(&cmd));
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("failed for 1 repositories"));
    }

    #[cfg(unix)]
    #[test]
    fn summary_and_persisted_receipts_match_actual_results() {
        use std::os::unix::fs::PermissionsExt;
        let f = Fixture::new();
        let base = f._temp.path().join("destination");
        let owner = base.join("fixture-owner");
        std::fs::create_dir_all(&owner).unwrap();
        let mut cache = std::collections::BTreeMap::new();
        for name in ["updated", "skipped", "dirty"] {
            git(&owner, &["clone", f.remote.to_str().unwrap(), name]);
            cache.insert(name.to_owned(), f.record());
        }
        // Updated must be fetched; skipped deliberately retains matching timestamps.
        cache.get_mut("updated").unwrap().pushed_at = "older".into();
        std::fs::write(owner.join("dirty/private"), "keep").unwrap();
        f.advance();
        git(&owner, &["clone", f.remote.to_str().unwrap(), "equal"]);
        std::fs::create_dir(owner.join("not-git")).unwrap();
        std::fs::write(owner.join("not-git/precious"), "keep").unwrap();
        let rewritten = Fixture::new();
        git(
            &owner,
            &["clone", rewritten.remote.to_str().unwrap(), "rewritten"],
        );
        cache.insert("rewritten".into(), rewritten.record());
        cache.get_mut("rewritten").unwrap().pushed_at = "older".into();
        git(&rewritten.publisher, &["checkout", "--orphan", "rewrite"]);
        Fixture::commit(&rewritten.publisher, "rewrite");
        git(
            &rewritten.publisher,
            &["push", "--force", "origin", "HEAD:main"],
        );
        std::fs::write(
            owner.join(".st-sync-state.json"),
            serde_json::to_vec(&cache).unwrap(),
        )
        .unwrap();
        let entries: Vec<_> = ["updated", "skipped", "dirty", "equal", "not-git", "rewritten", "cloned"].into_iter()
            .map(|name| serde_json::json!({"name": name, "pushedAt": "pushed", "updatedAt": "updated", "url": f.remote.to_str().unwrap()})).collect();
        let json = f._temp.path().join("repos.json");
        std::fs::write(&json, serde_json::to_vec(&entries).unwrap()).unwrap();
        let bin = f._temp.path().join("bin");
        std::fs::create_dir(&bin).unwrap();
        std::fs::write(bin.join("gh"), "#!/bin/sh\ncat \"$ST_SYNC_TEST_JSON\"\n").unwrap();
        std::fs::set_permissions(bin.join("gh"), std::fs::Permissions::from_mode(0o755)).unwrap();
        let path = std::env::join_paths(
            std::iter::once(bin).chain(std::env::split_paths(&std::env::var_os("PATH").unwrap())),
        )
        .unwrap();
        let output = std::process::Command::new(std::env::current_exe().unwrap())
            .current_dir(f._temp.path())
            .args([
                "--exact",
                "cli_commands::sync_tests::sync_handler_child",
                "--ignored",
                "--nocapture",
            ])
            .env("ST_SYNC_TEST_BASE", &base)
            .env("ST_SYNC_TEST_JSON", json)
            .env("PATH", path)
            .output()
            .unwrap();
        let stdout = String::from_utf8(output.stdout).unwrap();
        assert!(
            output.status.success(),
            "{stdout}\n{}",
            String::from_utf8_lossy(&output.stderr)
        );
        for expected in [
            "Cloned: 1",
            "Updated: 1",
            "Up-to-date (verified): 1",
            "Failed: 1",
            "Skipped (cached): 1",
            "Blocked: 2",
            "Fetched-only (subset of blocked): 1",
        ] {
            assert!(stdout.contains(expected), "Missing {expected}: {stdout}");
        }
        let saved: std::collections::BTreeMap<String, RepoSyncRecord> =
            serde_json::from_slice(&std::fs::read(owner.join(".st-sync-state.json")).unwrap())
                .unwrap();
        assert_eq!(
            saved.keys().map(String::as_str).collect::<Vec<_>>(),
            ["cloned", "equal", "skipped", "updated"]
        );
        assert_eq!(saved["skipped"].last_synced_at, "old-receipt");
        assert_eq!(
            std::fs::read_to_string(owner.join("dirty/private")).unwrap(),
            "keep"
        );
        assert_eq!(
            std::fs::read_to_string(owner.join("not-git/precious")).unwrap(),
            "keep"
        );
    }

    #[test]
    fn existing_non_git_contents_are_preserved() {
        let temp = temp_dir();
        let file = temp.path().join("precious");
        std::fs::write(&file, "keep").unwrap();
        let error = sync_repository(temp.path(), "unused", None, "p", "u", false).unwrap_err();
        assert!(error.to_string().contains("review or move"));
        assert_eq!(std::fs::read_to_string(file).unwrap(), "keep");
        assert!(!temp.path().join(".git").exists());
    }

    #[test]
    fn linked_worktree_is_supported() {
        let f = Fixture::new();
        let linked = f._temp.path().join("linked");
        git(
            &f.local,
            &[
                "worktree",
                "add",
                "-b",
                "linked",
                linked.to_str().unwrap(),
                "origin/main",
            ],
        );
        f.advance();
        assert!(linked.join(".git").is_file());
        assert!(matches!(
            sync_repository(&linked, "unused", None, "p", "u", true).unwrap(),
            RepoSyncOutcome::Updated(_)
        ));
    }

    #[test]
    #[ignore = "subprocess probe used by destination_lock_excludes_other_processes"]
    fn lock_child() {
        let dir = std::env::var_os("ST_SYNC_LOCK_TEST_DIR").unwrap();
        let should_lock = std::env::var_os("ST_SYNC_LOCK_TEST_AVAILABLE").is_some();
        assert_eq!(lock_sync_destination(Path::new(&dir)).is_ok(), should_lock);
    }

    #[test]
    fn destination_lock_excludes_other_processes() {
        let temp = temp_dir();
        let lock = lock_sync_destination(temp.path()).unwrap();
        let probe = |available: bool| {
            let mut command = std::process::Command::new(std::env::current_exe().unwrap());
            command
                .current_dir(temp.path())
                .args([
                    "--exact",
                    "cli_commands::sync_tests::lock_child",
                    "--ignored",
                ])
                .env("ST_SYNC_LOCK_TEST_DIR", temp.path())
                .env_remove("ST_SYNC_LOCK_TEST_AVAILABLE");
            if available {
                command.env("ST_SYNC_LOCK_TEST_AVAILABLE", "1");
            }
            assert!(command.status().unwrap().success());
        };
        probe(false);
        drop(lock);
        probe(true);
    }
}
