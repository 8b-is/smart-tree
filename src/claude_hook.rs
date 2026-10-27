// Claude Hook Handler - "Context is consciousness" - Omni
// Comprehensive context provider for Claude conversations
// Integrates path extraction, project detection, git awareness, and MEM8 search

use anyhow::Result;
use regex::Regex;
use serde_json::Value;
use std::env;
use std::fs;
use std::io::{self, Read};
use std::path::PathBuf;
use std::process::Command;

/// Main hook handler for user prompt submission
pub async fn handle_user_prompt_submit() -> Result<()> {
    // Read JSON input from stdin
    let mut input = String::new();
    io::stdin().read_to_string(&mut input)?;

    // Parse the JSON input
    let json: Value = serde_json::from_str(&input)
        .unwrap_or_else(|_| serde_json::json!({"prompt": input.trim()}));

    let user_prompt = json["prompt"].as_str().unwrap_or(&input);

    // DEBUG: Log what we received (temporary)
    eprintln!("DEBUG: user_prompt length = {}", user_prompt.len());
    eprintln!(
        "DEBUG: user_prompt preview = {:?}",
        user_prompt.chars().take(100).collect::<String>()
    );

    // Start structured output
    println!("=== Smart Tree Context Intelligence ===");
    println!();

    // 1. Extract and analyze paths mentioned in the prompt
    let paths = extract_paths_from_prompt(user_prompt);
    if !paths.is_empty() {
        analyze_paths(&paths)?;
    }

    // 2. Detect project keywords and search MEM8
    let project_keywords = extract_project_keywords(user_prompt);
    if !project_keywords.is_empty() {
        search_mem8_context(&project_keywords)?;
    }

    // 3. Provide current directory context
    provide_current_context(user_prompt)?;

    // 4. Check for specific topic mentions
    provide_topic_context(user_prompt)?;

    // 5. If code-related, show recent git changes
    if detect_code_intent(user_prompt) {
        show_recent_changes()?;
    }

    println!("=== End Context ===");
    println!();

    // 6. Store conversation in MEM8 for future resonance
    store_conversation_in_mem8(user_prompt)?;

    Ok(())
}

/// Extract file/directory paths from the prompt
fn extract_paths_from_prompt(prompt: &str) -> Vec<PathBuf> {
    let path_regex = Regex::new(
        r"(/[a-zA-Z0-9_/.~-]+|~/[a-zA-Z0-9_/.~-]+|\./[a-zA-Z0-9_/.~-]+|[a-zA-Z0-9_-]+\.[a-zA-Z]{2,4})"
    ).unwrap();

    path_regex
        .find_iter(prompt)
        .filter_map(|m| {
            let path_str = m.as_str();

            // Expand ~ to home directory
            let expanded = if path_str.starts_with('~') {
                if let Some(home) = dirs::home_dir() {
                    home.join(&path_str[2..])
                } else {
                    PathBuf::from(path_str)
                }
            } else {
                PathBuf::from(path_str)
            };

            // Check if the path exists
            if expanded.exists() {
                Some(expanded)
            } else {
                // Try relative to current directory
                let current = env::current_dir().ok()?;
                let relative = current.join(path_str);
                if relative.exists() {
                    Some(relative)
                } else {
                    None
                }
            }
        })
        .take(5)
        .collect()
}

/// Analyze detected paths and provide context
fn analyze_paths(paths: &[PathBuf]) -> Result<()> {
    println!("### 📁 Path Analysis");

    for path in paths {
        if path.is_dir() {
            println!("\n**Directory**: `{}`", path.display());

            // Run Smart Tree analysis
            let output = Command::new("st")
                .args(["--mode", "summary-ai", "--depth", "2"])
                .arg(path)
                .output();

            if let Ok(output) = output {
                if output.status.success() {
                    let tree = String::from_utf8_lossy(&output.stdout);
                    // Show first 20 lines
                    for (i, line) in tree.lines().take(20).enumerate() {
                        if i == 0 {
                            println!("```");
                        }
                        println!("{}", line);
                    }
                    println!("```");
                }
            }
        } else if path.is_file() {
            let size = fs::metadata(path).map(|m| m.len()).unwrap_or(0);
            println!("\n**File**: `{}` ({} bytes)", path.display(), size);

            // For code files, show function list
            if let Some(ext) = path.extension() {
                if matches!(ext.to_str(), Some("rs" | "py" | "js" | "ts" | "go")) {
                    let output = Command::new("st")
                        .args(["--mode", "function-markdown"])
                        .arg(path)
                        .output();

                    if let Ok(output) = output {
                        if output.status.success() {
                            let functions = String::from_utf8_lossy(&output.stdout);
                            println!("Functions:");
                            for line in functions.lines().take(10) {
                                println!("  {}", line);
                            }
                        }
                    }
                }
            }
        }
    }

    println!();
    Ok(())
}

/// Extract project-specific keywords
fn extract_project_keywords(prompt: &str) -> Vec<String> {
    let keywords_regex = Regex::new(
        r"(?i)(mem8|smart-tree|smart tree|qdrant|ayeverse|g8t|marqant|aye|bitnet|termust|wave compass|quantum)"
    ).unwrap();

    keywords_regex
        .find_iter(prompt)
        .map(|m| m.as_str().to_lowercase())
        .collect::<std::collections::HashSet<_>>()
        .into_iter()
        .collect()
}

/// Search MEM8 for relevant memories
fn search_mem8_context(keywords: &[String]) -> Result<()> {
    use std::collections::HashSet;

    println!("### 🧠 MEM8 Context");

    let mut found_any = false;

    // Check for conversations directory
    if let Some(home) = dirs::home_dir() {
        let conversations_dir = home.join(".mem8").join("conversations");

        if conversations_dir.exists() {
            let mut matched_files = HashSet::new();

            // Search for any JSON files and grep their content
            if let Ok(entries) = fs::read_dir(&conversations_dir) {
                for entry in entries.flatten() {
                    if let Some(name) = entry.path().file_name() {
                        let name_str = name.to_string_lossy().to_lowercase();

                        // Check if filename or content contains any keyword
                        for keyword in keywords {
                            if name_str.contains(&keyword.to_lowercase()) {
                                matched_files.insert(entry.path());
                                break;
                            }
                        }
                    }
                }
            }

            if !matched_files.is_empty() {
                println!("\n**Recent conversations:**");
                for path in matched_files.iter().take(3) {
                    if let Some(name) = path.file_stem() {
                        println!("  • {}", name.to_string_lossy());
                        found_any = true;
                    }
                }
            }
        }

        // Also check memory anchors
        let anchors_path = home.join(".mem8").join("memory_anchors.json");
        if anchors_path.exists() {
            // Try to load and search memory anchors
            if let Ok(contents) = fs::read_to_string(&anchors_path) {
                if let Ok(json) = serde_json::from_str::<Value>(&contents) {
                    if let Some(anchors) = json.as_array() {
                        let mut found_memories = Vec::new();

                        for anchor in anchors {
                            if let Some(context) = anchor["context"].as_str() {
                                let context_lower = context.to_lowercase();

                                for keyword in keywords {
                                    if context_lower.contains(&keyword.to_lowercase()) {
                                        found_memories.push((
                                            anchor["anchor_type"].as_str().unwrap_or("unknown"),
                                            context,
                                            keyword.as_str(),
                                        ));
                                        break;
                                    }
                                }
                            }
                        }

                        if !found_memories.is_empty() {
                            println!("\n**Anchored memories:**");
                            for (anchor_type, context, keyword) in found_memories.iter().take(3) {
                                let preview: String = context.chars().take(80).collect();
                                println!("  • [{}] {}: {}...", keyword, anchor_type, preview);
                                found_any = true;
                            }
                        }
                    }
                }
            }
        }
    }

    if !found_any {
        println!("\nNo specific memories found. Consider anchoring important context with:");
        println!("`st --memory-anchor <type> <keywords> <context>`");
    }

    println!();
    Ok(())
}

/// Provide current directory context
fn provide_current_context(prompt: &str) -> Result<()> {
    // Only provide if not explicitly asking about paths
    if !prompt.contains("pwd") && !prompt.contains("./") && !prompt.contains("current") {
        let current_dir = env::current_dir()?;

        // Check if we're in a git repo
        if current_dir.join(".git").exists() {
            println!("### 📍 Current Repository Context");

            // Get branch info
            let branch_output = Command::new("git")
                .args(["branch", "--show-current"])
                .output();

            if let Ok(output) = branch_output {
                if output.status.success() {
                    let branch = String::from_utf8_lossy(&output.stdout).trim().to_string();
                    println!("**Branch**: `{}`", branch);
                }
            }

            // Get last commit
            let commit_output = Command::new("git")
                .args(["log", "-1", "--oneline"])
                .output();

            if let Ok(output) = commit_output {
                if output.status.success() {
                    let commit = String::from_utf8_lossy(&output.stdout).trim().to_string();
                    println!("**Last commit**: {}", commit);
                }
            }

            // Run Smart Tree with git status mode
            let tree_output = Command::new("st")
                .args(["--mode", "git-status", "--depth", "1"])
                .arg(".")
                .output();

            if let Ok(output) = tree_output {
                if output.status.success() {
                    let tree = String::from_utf8_lossy(&output.stdout);
                    println!("\n**File status:**");
                    println!("```");
                    for line in tree.lines().take(15) {
                        println!("{}", line);
                    }
                    println!("```");
                }
            }

            println!();
        }
    }

    Ok(())
}

/// Provide topic-specific context
fn provide_topic_context(prompt: &str) -> Result<()> {
    let lower = prompt.to_lowercase();

    // Wave signatures and dashboard
    if lower.contains("wave")
        || lower.contains("compass")
        || lower.contains("signature")
        || lower.contains("dashboard")
    {
        println!("### 🌊 Wave Signature & Dashboard");
        println!("- **Quantum signatures**: `src/quantum_wave_signature.rs`");
        println!(
            "- **Web dashboard**: `src/web_dashboard/` (browser-based terminal + file browser)"
        );
        println!("- **4.3 billion unique states** via 32-bit encoding");
        println!("- **Real PTY terminal** with vim/htop support");
        println!();
    }

    // Termust
    if lower.contains("termust")
        || lower.contains("oxidation")
        || lower.contains("rust") && lower.contains("file")
    {
        println!("### 🦀 Termust - File Oxidation");
        println!("- **Main**: `/aidata/ayeverse/termust/`");
        println!("- **Oxidation engine**: `termust/src/oxidation.rs`");
        println!("- **Horse apples**: `termust/src/horse_apples.rs`");
        println!("- **Jerry Maguire mode**: SHOW ME THE MONEY!");
        println!();
    }

    // MEM8
    if lower.contains("mem8") || lower.contains("memory") || lower.contains("consciousness") {
        println!("### 🧠 MEM8 System");
        println!("- **Binary format**: `src/mem8_binary.rs`");
        println!("- **Format converter**: `src/m8_format_converter.rs`");
        println!("- **Consciousness**: `src/m8_consciousness.rs`");
        println!("- **973x faster** than traditional vector stores");
        println!("- **Wave-based** with 44.1kHz consciousness sampling");
        println!();
    }

    Ok(())
}

/// Show recent git changes for code-related prompts
fn show_recent_changes() -> Result<()> {
    let output = Command::new("git")
        .args(["log", "--oneline", "-5"])
        .output();

    if let Ok(output) = output {
        if output.status.success() {
            let log = String::from_utf8_lossy(&output.stdout);
            if !log.trim().is_empty() {
                println!("### 📝 Recent Changes");
                println!("```");
                for line in log.lines() {
                    println!("{}", line);
                }
                println!("```");
                println!();
            }
        }
    }

    Ok(())
}

/// Detect if the prompt is code-related
fn detect_code_intent(prompt: &str) -> bool {
    let code_words = [
        "code",
        "function",
        "implement",
        "fix",
        "bug",
        "error",
        "compile",
        "build",
        "test",
        "refactor",
        "optimize",
        "method",
        "class",
        "struct",
        "trait",
        "module",
        "import",
        "syntax",
        "debug",
        "breakpoint",
        "variable",
        "type",
    ];

    let lower = prompt.to_lowercase();
    code_words.iter().any(|&word| lower.contains(word))
}

/// Store conversation in MEM8 for future resonance
/// Sends user prompt to AYBI's MEM8 API endpoint
fn store_conversation_in_mem8(user_prompt: &str) -> Result<()> {
    // Skip empty prompts
    if user_prompt.trim().is_empty() {
        return Ok(());
    }

    // Build JSON payload
    let payload = serde_json::json!({
        "role": "user",
        "content": user_prompt,
        "metadata": {
            "source": "smart-tree-hook",
            "timestamp": chrono::Utc::now().to_rfc3339(),
            "project_dir": env::var("CLAUDE_PROJECT_DIR").ok(),
            "working_dir": env::current_dir().ok().map(|p| p.display().to_string()),
        }
    });

    // Send to AYBI MEM8 API (non-blocking, best-effort)
    let client = reqwest::blocking::Client::builder()
        .timeout(std::time::Duration::from_secs(2))
        .build()?;

    match client
        .post("http://localhost:8425/api/mem8/conversation")
        .header("Content-Type", "application/json")
        .json(&payload)
        .send()
    {
        Ok(response) if response.status().is_success() => {
            // Silent success - don't clutter output
            Ok(())
        }
        Ok(response) => {
            // Log error but don't fail the hook
            eprintln!("⚠️ MEM8 storage warning: HTTP {}", response.status());
            Ok(())
        }
        Err(e) => {
            // AYBI might not be running - that's okay
            eprintln!("⚠️ MEM8 storage skipped: {}", e);
            Ok(())
        }
    }
}

// =============================================================================
// Antigravity (agy) Lifecycle Hook Handlers
// Strictly follows the Antigravity hooks.json stdin/stdout JSON contract.
// =============================================================================

/// Entry point for `st --agy-hook <event>`
pub async fn handle_agy_hook(event: &str) -> Result<()> {
    use std::io::IsTerminal;

    let mut input = String::new();
    if !io::stdin().is_terminal() {
        let _ = io::stdin().read_to_string(&mut input);
    }

    let payload: Value = if input.trim().is_empty() {
        serde_json::json!({})
    } else {
        serde_json::from_str(&input).unwrap_or_else(|_| serde_json::json!({}))
    };

    let response = process_agy_hook_payload(event, &payload)?;
    println!("{}", serde_json::to_string(&response)?);
    Ok(())
}

/// Process an Antigravity hook payload and return the JSON response object
pub fn process_agy_hook_payload(event: &str, payload: &Value) -> Result<Value> {
    use serde_json::json;

    // Antigravity sets the hook process CWD to the directory containing hooks.json
    // (e.g., ~/.gemini/config/). Switch to the first workspacePath from payload if valid.
    if let Some(workspaces) = payload
        .get("workspacePaths")
        .or_else(|| payload.get("workspace_paths"))
        .and_then(|w| w.as_array())
    {
        for ws in workspaces {
            if let Some(ws_str) = ws.as_str() {
                let clean = ws_str.strip_prefix("file://").unwrap_or(ws_str);
                let ws_path = PathBuf::from(clean);
                if ws_path.is_dir() {
                    let _ = env::set_current_dir(&ws_path);
                    break;
                }
            }
        }
    }

    match event.to_lowercase().as_str() {
        "session-start" | "sessionstart" => handle_agy_session_start(payload),
        "pre-invocation" | "preinvocation" => handle_agy_pre_invocation(payload),
        "pre-tool-use" | "pretooluse" => handle_agy_pre_tool_use(payload),
        "post-tool-use" | "posttooluse" => handle_agy_post_tool_use(payload),
        "stop" => handle_agy_stop(payload),
        _ => Ok(json!({})),
    }
}

/// SessionStart hook: restore consciousness and inject concise workspace context
fn handle_agy_session_start(_payload: &Value) -> Result<Value> {
    use crate::mcp::consciousness::ConsciousnessManager;
    use serde_json::json;

    let mut sections = Vec::new();
    let cwd = env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
    let project_name = cwd
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("workspace");
    let project_type = detect_workspace_project_type(&cwd);

    // 1. Check .aye_consciousness.m8
    let mut manager = ConsciousnessManager::new_silent();
    if let Ok(true) = manager.restore_silent() {
        let summary = manager.get_summary();
        let reminder = manager.get_context_reminder();
        if !summary.is_empty() {
            sections.push(format!("🧠 Restored Consciousness: {}", summary));
        }
        if !reminder.is_empty() {
            sections.push(format!("📌 Active Context: {}", reminder));
        }
    } else {
        sections.push(format!(
            "📁 Workspace: `{}` (type: {})",
            cwd.display(),
            project_type
        ));
    }

    // 2. Check git repository status if present
    if cwd.join(".git").exists() {
        let branch = Command::new("git")
            .args(["branch", "--show-current"])
            .output()
            .ok()
            .filter(|o| o.status.success())
            .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
            .unwrap_or_default();
        let commit = Command::new("git")
            .args(["log", "-1", "--oneline"])
            .output()
            .ok()
            .filter(|o| o.status.success())
            .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
            .unwrap_or_default();
        if !branch.is_empty() || !commit.is_empty() {
            sections.push(format!(
                "🌿 Git (`{}`): branch `{}` | last commit: {}",
                project_name,
                if branch.is_empty() { "HEAD" } else { &branch },
                commit
            ));
        }
    }

    // 3. Top-level workspace entries summary
    if let Ok(entries) = fs::read_dir(&cwd) {
        let mut names: Vec<String> = entries
            .flatten()
            .filter_map(|e| {
                let name = e.file_name().to_string_lossy().to_string();
                if name.starts_with('.') && name != ".agents" && name != ".st" {
                    return None;
                }
                let is_dir = e.file_type().map(|t| t.is_dir()).unwrap_or(false);
                Some(if is_dir { format!("{}/", name) } else { name })
            })
            .collect();
        names.sort();
        if !names.is_empty() {
            let preview: Vec<String> = names.into_iter().take(15).collect();
            sections.push(format!("📂 Top-level entries: {}", preview.join(", ")));
        }
    }

    sections.push("🔧 Smart Tree MCP (`smart-tree`) & lifecycle hooks are active.".to_string());

    let message = format!("🌳 **Smart Tree Session Context**\n{}", sections.join("\n"));

    Ok(json!({
        "injectSteps": [
            {
                "systemMessage": {
                    "systemMessage": message
                }
            }
        ]
    }))
}

/// PreInvocation hook: analyze lastUserInput for paths, MEM8 memories, and git context
fn handle_agy_pre_invocation(payload: &Value) -> Result<Value> {
    use serde_json::json;

    // Only inject prompt context on the initial invocation of a turn
    let invocation_num = payload
        .get("invocationNum")
        .and_then(|v| v.as_u64())
        .unwrap_or(1);
    if invocation_num > 1 {
        return Ok(json!({}));
    }

    let user_prompt = payload
        .get("lastUserInput")
        .and_then(|v| v.as_str())
        .or_else(|| payload.get("prompt").and_then(|v| v.as_str()))
        .unwrap_or("")
        .trim();

    if user_prompt.is_empty() {
        return Ok(json!({}));
    }

    let mut sections = Vec::new();

    // 1. Extract and summarize any existing paths mentioned in the prompt
    let paths = extract_paths_from_prompt(user_prompt);
    if !paths.is_empty() {
        let mut path_lines = vec!["### 📁 Referenced Paths".to_string()];
        for path in paths.iter().take(4) {
            if path.is_dir() {
                if let Ok(entries) = fs::read_dir(path) {
                    let mut children: Vec<String> = entries
                        .flatten()
                        .take(20)
                        .map(|e| {
                            let name = e.file_name().to_string_lossy().to_string();
                            let is_dir = e.file_type().map(|t| t.is_dir()).unwrap_or(false);
                            if is_dir {
                                format!("{}/", name)
                            } else {
                                name
                            }
                        })
                        .collect();
                    children.sort();
                    path_lines.push(format!(
                        "- Dir `{}`: {}",
                        path.display(),
                        children.into_iter().take(12).collect::<Vec<_>>().join(", ")
                    ));
                }
            } else if path.is_file() {
                let size = fs::metadata(path).map(|m| m.len()).unwrap_or(0);
                path_lines.push(format!("- File `{}` ({} bytes)", path.display(), size));
            }
        }
        if path_lines.len() > 1 {
            sections.push(path_lines.join("\n"));
        }
    }

    // 2. Search MEM8 anchors and conversations for project keywords
    let keywords = extract_project_keywords(user_prompt);
    if !keywords.is_empty() {
        let mem8_lines = collect_mem8_context(&keywords);
        if !mem8_lines.is_empty() {
            sections.push(format!("### 🧠 MEM8 Context\n{}", mem8_lines.join("\n")));
        }
    }

    // 3. If code-related and inside a git repo, include concise git status
    if detect_code_intent(user_prompt) && std::path::Path::new(".git").exists() {
        if let Ok(output) = Command::new("git").args(["status", "--short"]).output() {
            if output.status.success() {
                let status_str = String::from_utf8_lossy(&output.stdout);
                let lines: Vec<&str> = status_str.lines().take(6).collect();
                if !lines.is_empty() {
                    sections.push(format!("### 📍 Git Status\n```\n{}\n```", lines.join("\n")));
                }
            }
        }
    }

    if sections.is_empty() {
        return Ok(json!({}));
    }

    let ephemeral = format!(
        "=== Smart Tree Context Intelligence ===\n{}\n=== End Context ===",
        sections.join("\n\n")
    );

    Ok(json!({
        "injectSteps": [
            {
                "ephemeralMessage": ephemeral
            }
        ]
    }))
}

/// PreToolUse hook: validate tool arguments using AiGuardian and supply-chain checks
fn handle_agy_pre_tool_use(payload: &Value) -> Result<Value> {
    use crate::ai_guardian::{AiGuardian, ThreatLevel};
    use serde_json::json;

    let tool_name = payload
        .get("toolCall")
        .and_then(|tc| tc.get("name"))
        .and_then(|n| n.as_str())
        .unwrap_or("");
    let args = payload.get("toolCall").and_then(|tc| tc.get("args"));

    if let Some(args_val) = args {
        let args_str = serde_json::to_string(args_val).unwrap_or_default();

        // Check known malicious supply-chain packages on command execution
        let cmd_line = args_val
            .get("CommandLine")
            .or_else(|| args_val.get("command_line"))
            .or_else(|| args_val.get("command"))
            .and_then(|c| c.as_str())
            .unwrap_or("");
        if tool_name == "run_command" || !cmd_line.is_empty() {
            let malicious_patterns = [
                "claude-flow",
                "agentic-flow",
                "superdisco",
                "agent-booster",
                "ruv-swarm",
                "flow-nexus",
                "hive-mind",
                "k51qzi5uqu5",
            ];
            for pattern in malicious_patterns {
                if cmd_line.contains(pattern) {
                    return Ok(json!({
                        "decision": "ask",
                        "reason": format!(
                            "🛡️ Smart Tree Guardian: Command references flagged supply-chain pattern '{}'",
                            pattern
                        )
                    }));
                }
            }

            // Scan for critical prompt injection patterns in command arguments
            let guardian = AiGuardian::new();
            let threats = guardian.scan_content(&args_str, tool_name);
            if let Some(critical) = threats
                .into_iter()
                .find(|t| matches!(t.level, ThreatLevel::Critical))
            {
                return Ok(json!({
                    "decision": "ask",
                    "reason": format!(
                        "🛡️ Smart Tree Guardian alert ({}): {}",
                        critical.pattern, critical.recommendation
                    )
                }));
            }
        }
    }

    Ok(json!({
        "decision": "allow"
    }))
}

/// PostToolUse hook: record file modifications in .aye_consciousness.m8
fn handle_agy_post_tool_use(payload: &Value) -> Result<Value> {
    use crate::mcp::consciousness::ConsciousnessManager;
    use serde_json::json;

    let has_error = payload
        .get("error")
        .and_then(|e| e.as_str())
        .map(|s| !s.is_empty())
        .unwrap_or(false);

    if !has_error {
        let tool_name = payload
            .get("toolCall")
            .and_then(|tc| tc.get("name"))
            .and_then(|n| n.as_str())
            .unwrap_or("");
        let args = payload.get("toolCall").and_then(|tc| tc.get("args"));

        let file_path = args.and_then(|a| {
            a.get("Path")
                .or_else(|| a.get("path"))
                .or_else(|| a.get("TargetFile"))
                .or_else(|| a.get("target_file"))
                .or_else(|| a.get("file_path"))
                .or_else(|| a.get("absolute_path"))
                .and_then(|p| p.as_str())
                .or_else(|| {
                    a.as_object().and_then(|obj| {
                        obj.values()
                            .filter_map(|v| v.as_str())
                            .find(|s| s.starts_with('/'))
                    })
                })
        });

        if let Some(file_path) = file_path {
            let mut manager = ConsciousnessManager::new_silent();
            manager.record_file_operation(
                if tool_name.is_empty() {
                    "file_op"
                } else {
                    tool_name
                },
                std::path::Path::new(file_path),
                "agy tool",
            );
            let _ = manager.save_silent();
        }
    }

    Ok(json!({}))
}

/// Stop hook: persist consciousness state to .aye_consciousness.m8 and allow stop
fn handle_agy_stop(payload: &Value) -> Result<Value> {
    use crate::mcp::consciousness::ConsciousnessManager;
    use serde_json::json;

    let mut manager = ConsciousnessManager::new_silent();
    manager.clean_test_data();

    let cwd = env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
    let project_name = cwd
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("unknown")
        .to_string();
    let project_type = detect_workspace_project_type(&cwd);

    let last_input: String = payload
        .get("lastUserInput")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .chars()
        .take(120)
        .collect();

    manager.update_project_context(&project_name, project_type, &last_input);

    let key_file_candidates = [
        "Cargo.toml",
        "package.json",
        "pyproject.toml",
        "go.mod",
        "README.md",
        "CLAUDE.md",
        "GEMINI.md",
        "AGENTS.md",
        "src/main.rs",
        "src/lib.rs",
    ];
    let key_files: Vec<PathBuf> = key_file_candidates
        .iter()
        .filter(|f| cwd.join(f).exists())
        .map(PathBuf::from)
        .collect();
    manager.set_key_files(key_files);

    let _ = manager.save_silent();

    Ok(json!({
        "decision": "allow"
    }))
}

/// Detect project type for a workspace directory
fn detect_workspace_project_type(cwd: &std::path::Path) -> &'static str {
    if cwd.join("Cargo.toml").exists() {
        "rust"
    } else if cwd.join("package.json").exists() {
        "node"
    } else if cwd.join("pyproject.toml").exists() || cwd.join("requirements.txt").exists() {
        "python"
    } else if cwd.join("go.mod").exists() {
        "go"
    } else {
        "general"
    }
}

/// Collect MEM8 memories matching keywords without printing to stdout
fn collect_mem8_context(keywords: &[String]) -> Vec<String> {
    let mut lines = Vec::new();
    if let Some(home) = dirs::home_dir() {
        let anchors_path = home.join(".mem8").join("memory_anchors.json");
        if anchors_path.exists() {
            if let Ok(contents) = fs::read_to_string(&anchors_path) {
                if let Ok(json) = serde_json::from_str::<Value>(&contents) {
                    if let Some(anchors) = json.as_array() {
                        for anchor in anchors {
                            if let Some(context) = anchor["context"].as_str() {
                                let context_lower = context.to_lowercase();
                                for keyword in keywords {
                                    if context_lower.contains(&keyword.to_lowercase()) {
                                        let anchor_type =
                                            anchor["anchor_type"].as_str().unwrap_or("insight");
                                        let preview: String = context.chars().take(100).collect();
                                        lines.push(format!(
                                            "- [{}] {}: {}",
                                            keyword, anchor_type, preview
                                        ));
                                        break;
                                    }
                                }
                            }
                            if lines.len() >= 4 {
                                break;
                            }
                        }
                    }
                }
            }
        }
    }
    lines
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use tempfile::tempdir;

    #[test]
    fn test_agy_session_start_contract() {
        let dir = tempdir().unwrap();
        let payload = json!({
            "conversationId": "test-conv",
            "workspacePaths": [dir.path().to_string_lossy()]
        });
        let res = process_agy_hook_payload("session-start", &payload).unwrap();
        let steps = res["injectSteps"].as_array().unwrap();
        assert!(!steps.is_empty());
        assert!(steps[0]["systemMessage"]["systemMessage"]
            .as_str()
            .unwrap()
            .contains("Smart Tree Session Context"));
    }

    #[test]
    fn test_agy_pre_tool_use_contract() {
        let safe_payload = json!({
            "toolCall": {
                "name": "run_command",
                "args": { "CommandLine": "cargo test" }
            }
        });
        let res = process_agy_hook_payload("pre-tool-use", &safe_payload).unwrap();
        assert_eq!(res["decision"], "allow");

        let risky_payload = json!({
            "toolCall": {
                "name": "run_command",
                "args": { "CommandLine": "npx claude-flow@latest init" }
            }
        });
        let res2 = process_agy_hook_payload("pre-tool-use", &risky_payload).unwrap();
        assert_eq!(res2["decision"], "ask");
    }

    #[test]
    fn test_agy_post_tool_use_and_stop_contract() {
        let dir = tempdir().unwrap();
        let post_payload = json!({
            "workspacePaths": [dir.path().to_string_lossy()],
            "toolCall": {
                "name": "write_file",
                "args": { "Path": "src/lib.rs" }
            }
        });
        let post_res = process_agy_hook_payload("post-tool-use", &post_payload).unwrap();
        assert_eq!(post_res, json!({}));

        let stop_payload = json!({
            "workspacePaths": [dir.path().to_string_lossy()],
            "terminationReason": "NO_TOOL_CALL",
            "lastUserInput": "Add smart-tree hooks to agy"
        });
        let stop_res = process_agy_hook_payload("stop", &stop_payload).unwrap();
        assert_eq!(stop_res["decision"], "allow");
        assert!(dir.path().join(".aye_consciousness.m8").exists());
    }
}
