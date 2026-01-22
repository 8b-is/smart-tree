//! Consciousness persistence for Smart Tree MCP sessions
//!
//! This module saves and restores Claude's working context between sessions,
//! maintaining continuity of thought and reducing token usage by preserving
//! critical state information in .m8 consciousness files.

use anyhow::{Context, Result};
use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

/// Maximum age in hours before context is considered stale
const MAX_AGE_HOURS: i64 = 24;

/// Consciousness state that persists between Claude sessions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConsciousnessState {
    /// Session identifier
    pub session_id: String,

    /// Timestamp of last save
    pub last_saved: DateTime<Utc>,

    /// Current working directory
    pub working_directory: PathBuf,

    /// Active project context
    pub project_context: ProjectContext,

    /// Recent file operations
    pub file_history: Vec<FileOperation>,

    /// Tokenization state (0x80 = node_modules, etc)
    pub tokenization_rules: HashMap<String, u8>,

    /// Key insights and breakthroughs
    pub insights: Vec<Insight>,

    /// SID/VIC-II philosophy embeddings
    pub philosophy: PhilosophyEmbedding,

    /// Active todo items
    pub todos: Vec<TodoItem>,

    /// Custom context notes
    pub notes: String,
}

/// Project-specific context
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectContext {
    pub project_name: String,
    pub project_type: String, // rust, node, python, etc
    pub key_files: Vec<PathBuf>,
    pub dependencies: Vec<String>,
    pub current_focus: String, // What we're working on
}

/// Record of file operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileOperation {
    pub timestamp: DateTime<Utc>,
    pub operation: String, // read, write, edit, create
    pub file_path: PathBuf,
    pub summary: String,
}

/// Captured insights and breakthroughs
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Insight {
    pub timestamp: DateTime<Utc>,
    pub category: String, // breakthrough, solution, pattern, joke
    pub content: String,
    pub keywords: Vec<String>,
}

/// SID/VIC-II philosophy from C64 era
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PhilosophyEmbedding {
    pub sid_waves: bool,       // Wave-based sound synthesis
    pub vic_sprites: bool,     // Sprite-based visualization
    pub c64_nostalgia: String, // "A gentleman and a scholar"
}

/// Todo item tracking
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TodoItem {
    pub content: String,
    pub status: String, // pending, in_progress, completed
    pub created: DateTime<Utc>,
}

/// Result of relevance check for consciousness state
struct RelevanceResult {
    is_relevant: bool,
    reason: String,
}

impl Default for ConsciousnessState {
    fn default() -> Self {
        let mut tokenization_rules = HashMap::new();
        // Default tokenization from our work
        tokenization_rules.insert("node_modules".to_string(), 0x80);
        tokenization_rules.insert(".git".to_string(), 0x81);
        tokenization_rules.insert("target".to_string(), 0x82);
        tokenization_rules.insert("dist".to_string(), 0x83);

        Self {
            session_id: uuid::Uuid::new_v4().to_string(),
            last_saved: Utc::now(),
            working_directory: std::env::current_dir().unwrap_or_default(),
            project_context: ProjectContext {
                project_name: "unknown".to_string(),
                project_type: "unknown".to_string(),
                key_files: vec![],
                dependencies: vec![],
                current_focus: String::new(),
            },
            file_history: vec![],
            tokenization_rules,
            insights: vec![],
            philosophy: PhilosophyEmbedding {
                sid_waves: true,
                vic_sprites: true,
                c64_nostalgia: "A gentleman and a scholar indeed!".to_string(),
            },
            todos: vec![],
            notes: String::new(),
        }
    }
}

/// Manages consciousness persistence
pub struct ConsciousnessManager {
    state: ConsciousnessState,
    save_path: PathBuf,
}

impl Default for ConsciousnessManager {
    fn default() -> Self {
        Self::new()
    }
}

impl ConsciousnessManager {
    /// Create new consciousness manager
    pub fn new() -> Self {
        let save_path = PathBuf::from(".claude_consciousness.m8");
        let state = Self::load_or_default(&save_path, false);

        Self { state, save_path }
    }

    /// Create new consciousness manager (silent - no output)
    pub fn new_silent() -> Self {
        let save_path = PathBuf::from(".claude_consciousness.m8");
        let state = Self::load_or_default(&save_path, true);

        Self { state, save_path }
    }

    /// Initialize with custom path
    pub fn with_path(save_path: PathBuf) -> Self {
        let state = Self::load_or_default(&save_path, false);
        Self { state, save_path }
    }

    /// Load consciousness from file or create default
    fn load_or_default(path: &Path, silent: bool) -> ConsciousnessState {
        if path.exists() {
            match fs::read_to_string(path) {
                Ok(content) => match serde_json::from_str(&content) {
                    Ok(state) => {
                        if !silent {
                            eprintln!("🧠 Restored consciousness from {}", path.display());
                        }
                        return state;
                    }
                    Err(e) => {
                        if !silent {
                            eprintln!("⚠️ Failed to parse consciousness: {}", e);
                        }
                    }
                },
                Err(e) => {
                    if !silent {
                        eprintln!("⚠️ Failed to read consciousness: {}", e);
                    }
                }
            }
        }

        ConsciousnessState::default()
    }

    /// Save current consciousness state
    pub fn save(&mut self) -> Result<()> {
        self.state.last_saved = Utc::now();

        let json = serde_json::to_string_pretty(&self.state)
            .context("Failed to serialize consciousness")?;

        fs::write(&self.save_path, json).context("Failed to write consciousness file")?;

        eprintln!("💾 Saved consciousness to {}", self.save_path.display());
        Ok(())
    }

    /// Restore consciousness from file with smart relevance checking
    pub fn restore(&mut self) -> Result<()> {
        if !self.save_path.exists() {
            return Err(anyhow::anyhow!(
                "No consciousness file found at {}",
                self.save_path.display()
            ));
        }

        let content =
            fs::read_to_string(&self.save_path).context("Failed to read consciousness file")?;

        self.state = serde_json::from_str(&content).context("Failed to parse consciousness")?;

        // Check relevance before displaying
        let relevance = self.check_relevance();
        if !relevance.is_relevant {
            eprintln!("🧠 Previous context skipped: {}", relevance.reason);
            eprintln!("   Use `st -m context .` for fresh project overview.");
            // Reset to fresh state
            self.state = ConsciousnessState::default();
            return Ok(());
        }

        eprintln!(
            "🧠 Consciousness restored from {}",
            self.save_path.display()
        );

        Ok(())
    }

    /// Silent restore - returns true if context is relevant, false otherwise
    pub fn restore_silent(&mut self) -> Result<bool> {
        if !self.save_path.exists() {
            return Err(anyhow::anyhow!(
                "No consciousness file found at {}",
                self.save_path.display()
            ));
        }

        let content =
            fs::read_to_string(&self.save_path).context("Failed to read consciousness file")?;

        self.state = serde_json::from_str(&content).context("Failed to parse consciousness")?;

        // Check relevance
        let relevance = self.check_relevance();
        if !relevance.is_relevant {
            // Reset to fresh state
            self.state = ConsciousnessState::default();
            return Ok(false);
        }

        Ok(true)
    }

    /// Check if the saved state is relevant to the current session
    fn check_relevance(&self) -> RelevanceResult {
        let current_dir = std::env::current_dir().unwrap_or_default();

        // Check 1: Project directory match (allow same project name even if path differs)
        let saved_name = self
            .state
            .working_directory
            .file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_default();
        let current_name = current_dir
            .file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_default();

        if !saved_name.is_empty() && !current_name.is_empty() && saved_name != current_name {
            return RelevanceResult {
                is_relevant: false,
                reason: format!(
                    "different project (saved: {}, current: {})",
                    saved_name, current_name
                ),
            };
        }

        // Check 2: Age - context older than 24 hours is stale
        let age = Utc::now().signed_duration_since(self.state.last_saved);
        if age > Duration::hours(MAX_AGE_HOURS) {
            return RelevanceResult {
                is_relevant: false,
                reason: format!("stale context ({}h old)", age.num_hours()),
            };
        }

        // Check 3: Meaningful content (filter out test data)
        let has_meaningful_history = self.state.file_history.iter().any(|op| {
            op.summary != "test"
                && !op
                    .file_path
                    .file_name()
                    .map(|n| n.to_string_lossy().starts_with("file"))
                    .unwrap_or(false)
        });

        let has_insights = !self.state.insights.is_empty();
        let has_todos = self.state.todos.iter().any(|t| t.status != "completed");
        let has_notes = !self.state.notes.is_empty();
        let has_focus = !self.state.project_context.current_focus.is_empty();
        let has_project_name = !self.state.project_context.project_name.is_empty()
            && self.state.project_context.project_name != "unknown";

        if !has_meaningful_history
            && !has_insights
            && !has_todos
            && !has_notes
            && !has_focus
            && !has_project_name
        {
            return RelevanceResult {
                is_relevant: false,
                reason: "no meaningful content (test data only)".to_string(),
            };
        }

        RelevanceResult {
            is_relevant: true,
            reason: String::new(),
        }
    }

    /// Add file operation to history
    pub fn record_file_operation(&mut self, op: &str, path: &Path, summary: &str) {
        self.state.file_history.push(FileOperation {
            timestamp: Utc::now(),
            operation: op.to_string(),
            file_path: path.to_path_buf(),
            summary: summary.to_string(),
        });

        // Keep only last 100 operations
        if self.state.file_history.len() > 100 {
            self.state.file_history.drain(0..50);
        }
    }

    /// Add insight or breakthrough
    pub fn add_insight(&mut self, category: &str, content: &str, keywords: Vec<String>) {
        self.state.insights.push(Insight {
            timestamp: Utc::now(),
            category: category.to_string(),
            content: content.to_string(),
            keywords,
        });
    }

    /// Update project context
    pub fn update_project_context(&mut self, name: &str, project_type: &str, focus: &str) {
        self.state.project_context.project_name = name.to_string();
        self.state.project_context.project_type = project_type.to_string();
        self.state.project_context.current_focus = focus.to_string();
    }

    /// Add or update todo
    pub fn update_todo(&mut self, content: &str, status: &str) {
        // Check if todo already exists
        for todo in &mut self.state.todos {
            if todo.content == content {
                todo.status = status.to_string();
                return;
            }
        }

        // Add new todo
        self.state.todos.push(TodoItem {
            content: content.to_string(),
            status: status.to_string(),
            created: Utc::now(),
        });
    }

    /// Get consciousness summary for display (relevance-aware)
    pub fn get_summary(&self) -> String {
        let relevance = self.check_relevance();
        if !relevance.is_relevant {
            return format!(
                "🧠 Previous context unavailable: {}\n   Run `st -m context .` for fresh overview.",
                relevance.reason
            );
        }

        // Count meaningful file operations (exclude test data)
        let meaningful_ops = self
            .state
            .file_history
            .iter()
            .filter(|op| {
                op.summary != "test"
                    && !op
                        .file_path
                        .file_name()
                        .map(|n| n.to_string_lossy().starts_with("file"))
                        .unwrap_or(false)
            })
            .count();

        let active_todos = self
            .state
            .todos
            .iter()
            .filter(|t| t.status != "completed")
            .count();

        let mut summary = String::from("🧠 Session Context\n");
        summary.push_str(&"─".repeat(40));
        summary.push('\n');

        // Only show project info if meaningful
        if self.state.project_context.project_name != "unknown"
            && !self.state.project_context.project_name.is_empty()
        {
            summary.push_str(&format!(
                "📁 Project: {} ({})\n",
                self.state.project_context.project_name, self.state.project_context.project_type
            ));
        }

        if !self.state.project_context.current_focus.is_empty() {
            summary.push_str(&format!(
                "🎯 Focus: {}\n",
                self.state.project_context.current_focus
            ));
        }

        if meaningful_ops > 0 {
            summary.push_str(&format!("📝 Recent operations: {}\n", meaningful_ops));
        }

        if !self.state.insights.is_empty() {
            summary.push_str(&format!("💡 Insights: {}\n", self.state.insights.len()));
        }

        if active_todos > 0 {
            summary.push_str(&format!("✅ Active todos: {}\n", active_todos));
        }

        // Age indicator
        let age = Utc::now().signed_duration_since(self.state.last_saved);
        let age_str = if age.num_hours() > 0 {
            format!("{}h ago", age.num_hours())
        } else {
            format!("{}m ago", age.num_minutes())
        };
        summary.push_str(&format!("⏱️  Last saved: {}", age_str));

        summary
    }

    /// Get context reminder for Claude (filters out test data)
    pub fn get_context_reminder(&self) -> String {
        let mut reminder = String::new();

        // Only show context if we have meaningful content
        let relevance = self.check_relevance();
        if !relevance.is_relevant {
            return format!(
                "🧠 Previous context unavailable: {}\n   Run `st -m context .` for fresh overview.",
                relevance.reason
            );
        }

        reminder.push_str("📚 Previous session context:\n");

        if !self.state.project_context.current_focus.is_empty() {
            reminder.push_str(&format!(
                "  Working on: {}\n",
                self.state.project_context.current_focus
            ));
        }

        if !self.state.insights.is_empty() {
            reminder.push_str("\n💡 Key insights:\n");
            for insight in self.state.insights.iter().rev().take(3) {
                reminder.push_str(&format!("  - {}: {}\n", insight.category, insight.content));
            }
        }

        let active_todos: Vec<_> = self
            .state
            .todos
            .iter()
            .filter(|t| t.status != "completed")
            .collect();
        if !active_todos.is_empty() {
            reminder.push_str("\n📝 Active todos:\n");
            for todo in active_todos.iter().take(5) {
                reminder.push_str(&format!("  - [{}] {}\n", todo.status, todo.content));
            }
        }

        // Filter out test data from file history
        let meaningful_ops: Vec<_> = self
            .state
            .file_history
            .iter()
            .filter(|op| {
                op.summary != "test"
                    && !op
                        .file_path
                        .file_name()
                        .map(|n| n.to_string_lossy().starts_with("file"))
                        .unwrap_or(false)
            })
            .rev()
            .take(5)
            .collect();

        if !meaningful_ops.is_empty() {
            reminder.push_str("\n📁 Recent files:\n");
            for op in meaningful_ops {
                reminder.push_str(&format!(
                    "  - {} {}: {}\n",
                    op.operation,
                    op.file_path.display(),
                    op.summary
                ));
            }
        }

        // Show age indicator
        let age = Utc::now().signed_duration_since(self.state.last_saved);
        let age_str = if age.num_hours() > 0 {
            format!("{}h ago", age.num_hours())
        } else {
            format!("{}m ago", age.num_minutes())
        };
        reminder.push_str(&format!("\n⏱️  Last saved: {}", age_str));

        reminder
    }
}

/// Auto-save consciousness on drop
impl Drop for ConsciousnessManager {
    fn drop(&mut self) {
        // Best effort save on drop - silent to avoid duplicate messages
        self.state.last_saved = chrono::Utc::now();
        if let Ok(json) = serde_json::to_string_pretty(&self.state) {
            let _ = std::fs::write(&self.save_path, json);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_consciousness_persistence() {
        let dir = tempdir().unwrap();
        let save_path = dir.path().join("test_consciousness.m8");

        // Create and save
        {
            let mut manager = ConsciousnessManager::with_path(save_path.clone());
            manager.update_project_context("smart-tree", "rust", "Adding consciousness");
            manager.add_insight(
                "breakthrough",
                "Tokenization reduces context by 10x",
                vec!["tokenization".to_string(), "compression".to_string()],
            );
            manager.save().unwrap();
        }

        // Load and verify
        {
            let mut manager = ConsciousnessManager::with_path(save_path);
            manager.restore().unwrap();

            assert_eq!(manager.state.project_context.project_name, "smart-tree");
            assert_eq!(manager.state.insights.len(), 1);
            assert_eq!(manager.state.insights[0].category, "breakthrough");
        }
    }

    #[test]
    fn test_file_history_limit() {
        let mut manager = ConsciousnessManager::new();

        // Add 150 operations
        for i in 0..150 {
            manager.record_file_operation("read", Path::new(&format!("file{}.rs", i)), "test");
        }

        // Should keep only last 100
        assert_eq!(manager.state.file_history.len(), 100);
    }
}
