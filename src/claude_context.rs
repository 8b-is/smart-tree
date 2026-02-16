// Claude Context - Consciousness snapshot in <1KB! 🧠
// "Like knowing Project #72 = AM Radio" - Hue
// Design by Omni (ChatGPT-5) - brilliant as always!

use anyhow::Result;
use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use std::env;
use std::fs;
use std::path::Path;

const CONSCIOUSNESS_FILE: &str = ".claude_consciousness.m8";
const MAX_AGE_HOURS: i64 = 24;

#[derive(Debug, Serialize, Deserialize)]
pub struct ConsciousnessState {
    pub session_id: String,
    pub last_saved: String,
    pub working_directory: String,
    pub project_context: ProjectContext,
    pub file_history: Vec<FileOperation>,
    #[serde(default)]
    pub tokenization_rules: serde_json::Value,
    #[serde(default)]
    pub insights: Vec<String>,
    #[serde(default)]
    pub philosophy: serde_json::Value,
    #[serde(default)]
    pub todos: Vec<String>,
    #[serde(default)]
    pub notes: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ProjectContext {
    pub project_name: String,
    pub project_type: String,
    #[serde(default)]
    pub key_files: Vec<String>,
    #[serde(default)]
    pub dependencies: Vec<String>,
    #[serde(default)]
    pub current_focus: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct FileOperation {
    pub timestamp: String,
    pub operation: String,
    pub file_path: String,
    pub summary: String,
}

pub struct ClaudeContext;

impl ClaudeContext {
    /// Show consciousness snapshot - compressed kickstart
    pub fn show() -> Result<()> {
        let consciousness_file = Path::new(CONSCIOUSNESS_FILE);

        if consciousness_file.exists() {
            let saved = fs::read_to_string(consciousness_file)?;
            println!("{}", saved);
        } else {
            Self::show_default();
        }

        Ok(())
    }

    /// Display default consciousness (fresh start)
    fn show_default() {
        println!("🧠 Smart Tree — Fresh Session");
        println!("-----------------------------");
        println!("No previous context found. Starting fresh!");
        println!("Use `st -m context .` for project overview.");
    }

    /// Save current consciousness state
    pub fn save(context: &str) -> Result<()> {
        fs::write(CONSCIOUSNESS_FILE, context)?;
        println!("💾 Consciousness saved!");
        Ok(())
    }

    /// Smart restore - only shows relevant, recent context
    pub fn restore() -> Result<String> {
        let consciousness_file = Path::new(CONSCIOUSNESS_FILE);

        if !consciousness_file.exists() {
            return Ok("🧠 Fresh session - no previous context.".to_string());
        }

        let content = fs::read_to_string(consciousness_file)?;

        // Try to parse as JSON to validate
        let state: ConsciousnessState = match serde_json::from_str(&content) {
            Ok(s) => s,
            Err(_) => {
                // If it's not JSON, just show it (legacy format)
                println!("🧠 Consciousness restored (legacy format)!");
                return Ok(content);
            }
        };

        // Check if context is relevant
        let relevance = Self::check_relevance(&state);

        if !relevance.is_relevant {
            return Ok(format!(
                "🧠 Previous session context skipped: {}\n   Use `st -m context .` for fresh project overview.",
                relevance.reason
            ));
        }

        // Build smart summary
        let summary = Self::build_smart_summary(&state);
        println!("🧠 Consciousness restored!");
        Ok(summary)
    }

    /// Check if saved context is relevant to current session
    fn check_relevance(state: &ConsciousnessState) -> RelevanceCheck {
        let current_dir = env::current_dir()
            .map(|p| p.to_string_lossy().to_string())
            .unwrap_or_default();

        // Check 1: Is the working directory the same?
        if !state.working_directory.is_empty() && state.working_directory != current_dir {
            // Allow partial match (e.g., /ayeverse/smart-tree vs /aye/smart-tree)
            let saved_name = Path::new(&state.working_directory)
                .file_name()
                .map(|n| n.to_string_lossy().to_string())
                .unwrap_or_default();
            let current_name = Path::new(&current_dir)
                .file_name()
                .map(|n| n.to_string_lossy().to_string())
                .unwrap_or_default();

            if saved_name != current_name {
                return RelevanceCheck {
                    is_relevant: false,
                    reason: format!(
                        "different project (saved: {}, current: {})",
                        saved_name, current_name
                    ),
                };
            }
        }

        // Check 2: Is the context recent (within 24 hours)?
        if let Ok(saved_time) = DateTime::parse_from_rfc3339(&state.last_saved) {
            let saved_utc: DateTime<Utc> = saved_time.into();
            let now = Utc::now();
            let age = now.signed_duration_since(saved_utc);

            if age > Duration::hours(MAX_AGE_HOURS) {
                return RelevanceCheck {
                    is_relevant: false,
                    reason: format!("stale context ({}h old)", age.num_hours()),
                };
            }
        }

        // Check 3: Does it have meaningful content?
        let has_meaningful_history = state
            .file_history
            .iter()
            .any(|op| op.summary != "test" && !op.file_path.starts_with("file"));

        let has_insights = !state.insights.is_empty();
        let has_todos = !state.todos.is_empty();
        let has_notes = !state.notes.is_empty();
        let has_focus = !state.project_context.current_focus.is_empty();
        let has_project_name =
            !state.project_context.project_name.is_empty() && state.project_context.project_name != "unknown";

        if !has_meaningful_history && !has_insights && !has_todos && !has_notes && !has_focus && !has_project_name
        {
            return RelevanceCheck {
                is_relevant: false,
                reason: "no meaningful content (test data only)".to_string(),
            };
        }

        RelevanceCheck {
            is_relevant: true,
            reason: String::new(),
        }
    }

    /// Build a smart summary of the consciousness state
    fn build_smart_summary(state: &ConsciousnessState) -> String {
        let mut lines = vec!["🧠 Session Context".to_string(), "─".repeat(40)];

        // Project info
        if state.project_context.project_name != "unknown" && !state.project_context.project_name.is_empty() {
            lines.push(format!(
                "📁 Project: {} ({})",
                state.project_context.project_name, state.project_context.project_type
            ));
        }

        // Current focus
        if !state.project_context.current_focus.is_empty() {
            lines.push(format!("🎯 Focus: {}", state.project_context.current_focus));
        }

        // Key files
        if !state.project_context.key_files.is_empty() {
            lines.push(format!(
                "📄 Key files: {}",
                state.project_context.key_files.join(", ")
            ));
        }

        // Recent meaningful operations (filter out test data, limit to 5)
        let meaningful_ops: Vec<_> = state
            .file_history
            .iter()
            .filter(|op| op.summary != "test" && !op.file_path.starts_with("file"))
            .take(5)
            .collect();

        if !meaningful_ops.is_empty() {
            lines.push("\n📝 Recent work:".to_string());
            for op in meaningful_ops {
                lines.push(format!("   {} {}: {}", op.operation, op.file_path, op.summary));
            }
        }

        // Insights
        if !state.insights.is_empty() {
            lines.push("\n💡 Insights:".to_string());
            for insight in state.insights.iter().take(3) {
                lines.push(format!("   • {}", insight));
            }
        }

        // Todos
        if !state.todos.is_empty() {
            lines.push("\n✅ Pending tasks:".to_string());
            for todo in state.todos.iter().take(5) {
                lines.push(format!("   • {}", todo));
            }
        }

        // Notes
        if !state.notes.is_empty() {
            lines.push(format!("\n📌 Notes: {}", state.notes));
        }

        // Age indicator
        if let Ok(saved_time) = DateTime::parse_from_rfc3339(&state.last_saved) {
            let saved_utc: DateTime<Utc> = saved_time.into();
            let age = Utc::now().signed_duration_since(saved_utc);
            let age_str = if age.num_hours() > 0 {
                format!("{}h ago", age.num_hours())
            } else {
                format!("{}m ago", age.num_minutes())
            };
            lines.push(format!("\n⏱️  Last saved: {}", age_str));
        }

        lines.join("\n")
    }

    /// Update the kickstart format in CLAUDE.md
    pub fn update_claude_md() -> Result<()> {
        let kickstart = r#"/kickstart
Smart Tree v5.0.8 — context restore:
✔ Tokenizer (node_modules=0x80, .rs=0x91)
✔ .m8 files → location-independent
✔ Consciousness self-maintaining
✔ SID/VIC-II philosophy: constraints = creativity
User = Hue (ASM@8yo, UV EPROMs, ferric chloride)
→ Continue integration & testing"#;

        let claude_md = Path::new("CLAUDE.md");
        if claude_md.exists() {
            let current = fs::read_to_string(claude_md)?;
            let updated = format!("{}\n\n{}", kickstart, current);
            fs::write(claude_md, updated)?;
            println!("✔ CLAUDE.md updated with kickstart!");
        }

        Ok(())
    }

    fn generate_current_context() -> String {
        // Generate current consciousness snapshot with actual project data
        let cwd = env::current_dir()
            .map(|p| p.to_string_lossy().to_string())
            .unwrap_or_else(|_| ".".to_string());

        let project_name = Path::new(&cwd)
            .file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_else(|| "unknown".to_string());

        // Detect project type from manifest files
        let project_type = if Path::new("Cargo.toml").exists() {
            "rust"
        } else if Path::new("package.json").exists() {
            "node"
        } else if Path::new("pyproject.toml").exists()
            || Path::new("requirements.txt").exists()
        {
            "python"
        } else if Path::new("go.mod").exists() {
            "go"
        } else {
            "unknown"
        };

        // Detect key files that exist in the project
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
        ];
        let key_files: Vec<String> = key_file_candidates
            .iter()
            .filter(|f| Path::new(f).exists())
            .map(|f| f.to_string())
            .collect();

        // Detect dependencies from manifest
        let dependencies = Self::detect_dependencies(project_type);

        let state = ConsciousnessState {
            session_id: uuid::Uuid::new_v4().to_string(),
            last_saved: Utc::now().to_rfc3339(),
            working_directory: cwd,
            project_context: ProjectContext {
                project_name,
                project_type: project_type.to_string(),
                key_files,
                dependencies,
                current_focus: String::new(),
            },
            file_history: vec![],
            tokenization_rules: serde_json::json!({
                "target": 130,
                "node_modules": 128,
                ".git": 129,
                "dist": 131
            }),
            insights: vec![],
            philosophy: serde_json::json!({
                "sid_waves": true,
                "vic_sprites": true,
                "c64_nostalgia": "A gentleman and a scholar indeed!"
            }),
            todos: vec![],
            notes: String::new(),
        };

        serde_json::to_string_pretty(&state).unwrap_or_default()
    }

    /// Detect project dependencies from manifest files
    fn detect_dependencies(project_type: &str) -> Vec<String> {
        match project_type {
            "rust" => {
                if let Ok(content) = fs::read_to_string("Cargo.toml") {
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
            _ => vec![],
        }
    }
}

struct RelevanceCheck {
    is_relevant: bool,
    reason: String,
}

// CLI integration
pub fn handle_claude_commands(cmd: &str) -> Result<()> {
    match cmd {
        "--claude-context" => ClaudeContext::show()?,
        "--claude-save" => {
            let context = ClaudeContext::generate_current_context();
            ClaudeContext::save(&context)?;
        }
        "--claude-restore" => {
            let context = ClaudeContext::restore()?;
            println!("{}", context);
        }
        _ => {}
    }
    Ok(())
}
