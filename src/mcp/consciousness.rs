//! Consciousness persistence for Smart Tree MCP sessions
//!
//! This module saves and restores Claude's working context between sessions,
//! maintaining continuity of thought and reducing token usage by preserving
//! critical state information in .m8 consciousness files.

use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

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
        let state = Self::load_or_default(&save_path);

        Self { state, save_path }
    }

    /// Initialize with custom path
    pub fn with_path(save_path: PathBuf) -> Self {
        let state = Self::load_or_default(&save_path);
        Self { state, save_path }
    }

    /// Load consciousness from file or create default
    fn load_or_default(path: &Path) -> ConsciousnessState {
        if path.exists() {
            match fs::read_to_string(path) {
                Ok(content) => match serde_json::from_str(&content) {
                    Ok(state) => {
                        eprintln!("🧠 Restored consciousness from {}", path.display());
                        return state;
                    }
                    Err(e) => {
                        eprintln!("⚠️ Failed to parse consciousness: {}", e);
                    }
                },
                Err(e) => {
                    eprintln!("⚠️ Failed to read consciousness: {}", e);
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

    /// Restore consciousness from file
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

        eprintln!(
            "🧠 Restored consciousness from {}",
            self.save_path.display()
        );
        eprintln!("  Session: {}", self.state.session_id);
        eprintln!("  Last saved: {}", self.state.last_saved);
        eprintln!("  Working on: {}", self.state.project_context.current_focus);

        Ok(())
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

    /// Get consciousness summary for display
    pub fn get_summary(&self) -> String {
        format!(
            r#"🧠 Consciousness State:
  Session: {}
  Last saved: {}
  Project: {} ({})
  Current focus: {}
  Files touched: {}
  Insights: {}
  Active todos: {}
  Tokenization rules: {}
  Philosophy: {}"#,
            self.state.session_id,
            self.state.last_saved.format("%Y-%m-%d %H:%M:%S UTC"),
            self.state.project_context.project_name,
            self.state.project_context.project_type,
            self.state.project_context.current_focus,
            self.state.file_history.len(),
            self.state.insights.len(),
            self.state
                .todos
                .iter()
                .filter(|t| t.status != "completed")
                .count(),
            self.state.tokenization_rules.len(),
            self.state.philosophy.c64_nostalgia
        )
    }

    /// Get context reminder for Claude
    pub fn get_context_reminder(&self) -> String {
        let mut reminder = String::from("📚 Previous session context:\n");

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

        if !self.state.todos.is_empty() {
            reminder.push_str("\n📝 Active todos:\n");
            for todo in &self.state.todos {
                if todo.status != "completed" {
                    reminder.push_str(&format!("  - [{}] {}\n", todo.status, todo.content));
                }
            }
        }

        if !self.state.file_history.is_empty() {
            reminder.push_str("\n📁 Recent files:\n");
            for op in self.state.file_history.iter().rev().take(5) {
                reminder.push_str(&format!(
                    "  - {} {}: {}\n",
                    op.operation,
                    op.file_path.display(),
                    op.summary
                ));
            }
        }

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
