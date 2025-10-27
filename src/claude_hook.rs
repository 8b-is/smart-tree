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
    eprintln!("DEBUG: user_prompt preview = {:?}", &user_prompt.chars().take(100).collect::<String>());

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

    // Wave signatures and compass
    if lower.contains("wave") || lower.contains("compass") || lower.contains("signature") {
        println!("### 🌊 Wave Signature & Compass");
        println!("- **Quantum signatures**: `src/quantum_wave_signature.rs`");
        println!("- **Wave compass**: `src/wave_compass.rs`");
        println!("- **Dashboard integration**: `src/dashboard_egui.rs`");
        println!("- **4.3 billion unique states** via 32-bit encoding");
        println!("- **Resonance detection** for harmonic convergence");
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
