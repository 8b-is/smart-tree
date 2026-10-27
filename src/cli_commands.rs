//! Modular CLI subcommand and flag handlers extracted from `main.rs`.
//!
//! Keeps `main.rs` focused on startup orchestration, logging, and flag dispatch
//! while grouping agent consciousness, hooks, daemon, MCP, security, memory,
//! mega-session, guardian, Google Drive, and voice CLI handlers here.

use crate::daemon_client::DaemonClient;
use anyhow::Result;
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
