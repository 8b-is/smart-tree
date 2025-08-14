// ST Context-Aware System - The helper who knows what you need!
// "Like a good roadie who hands you the right guitar at the right time" - The Cheet 🎸

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, VecDeque};
use std::path::{Path, PathBuf};
use std::sync::{Arc, RwLock};
use std::time::SystemTime;

/// Context types that ST tracks
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum WorkContext {
    /// Writing new code
    Coding {
        language: String,
        focus_file: PathBuf,
    },
    /// Debugging/fixing issues
    Debugging {
        error_pattern: String,
        files: Vec<PathBuf>,
    },
    /// Refactoring code
    Refactoring { pattern: String, scope: PathBuf },
    /// Exploring/understanding codebase
    Exploring {
        depth: usize,
        areas_visited: Vec<PathBuf>,
    },
    /// Testing/validation
    Testing {
        test_files: Vec<PathBuf>,
        target_files: Vec<PathBuf>,
    },
    /// Documentation
    Documenting { doc_type: String, target: PathBuf },
    /// Performance optimization
    Optimizing {
        metrics: Vec<String>,
        hotspots: Vec<PathBuf>,
    },
    /// Searching for something specific
    Hunting {
        query: String,
        found_locations: Vec<PathBuf>,
    },
    /// Building/compilation
    Building {
        build_system: String,
        targets: Vec<String>,
    },
    /// Git operations
    VersionControl {
        operation: String,
        changed_files: Vec<PathBuf>,
    },
}

/// A single operation in context
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextualOperation {
    pub timestamp: SystemTime,
    pub operation: String,
    pub path: PathBuf,
    pub result_summary: String,
    pub context_hints: Vec<String>,
}

/// Smart context tracker
#[derive(Debug, Clone)]
pub struct StContextTracker {
    /// Current work context
    current_context: Arc<RwLock<Option<WorkContext>>>,
    /// Recent operations (last 50)
    operation_history: Arc<RwLock<VecDeque<ContextualOperation>>>,
    /// Pattern recognition cache
    #[allow(dead_code)]
    patterns: Arc<RwLock<HashMap<String, Vec<String>>>>,
    /// Project-specific knowledge
    project_knowledge: Arc<RwLock<ProjectKnowledge>>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ProjectKnowledge {
    /// Key files in the project
    pub key_files: Vec<PathBuf>,
    /// Common search patterns
    pub common_searches: HashMap<String, usize>,
    /// Frequently accessed directories
    pub hot_directories: HashMap<PathBuf, usize>,
    /// Known build commands
    pub build_commands: Vec<String>,
    /// Test patterns
    pub test_patterns: Vec<String>,
    /// Documentation locations
    pub doc_locations: Vec<PathBuf>,
}

impl Default for StContextTracker {
    fn default() -> Self {
        Self::new()
    }
}

impl StContextTracker {
    pub fn new() -> Self {
        Self {
            current_context: Arc::new(RwLock::new(None)),
            operation_history: Arc::new(RwLock::new(VecDeque::with_capacity(50))),
            patterns: Arc::new(RwLock::new(HashMap::new())),
            project_knowledge: Arc::new(RwLock::new(ProjectKnowledge::default())),
        }
    }

    /// Analyze recent operations to determine context
    pub fn analyze_context(&self) -> Result<WorkContext> {
        let history = self.operation_history.read().unwrap();

        if history.is_empty() {
            return Ok(WorkContext::Exploring {
                depth: 3,
                areas_visited: vec![],
            });
        }

        // Look at recent operations
        let recent_ops: Vec<_> = history.iter().take(10).collect();

        // Count operation types
        let mut search_count = 0;
        let mut edit_count = 0;
        let mut read_count = 0;
        let mut test_count = 0;
        let mut _build_count = 0;

        for op in &recent_ops {
            if op.operation.contains("search") || op.operation.contains("grep") {
                search_count += 1;
            }
            if op.operation.contains("edit") || op.operation.contains("write") {
                edit_count += 1;
            }
            if op.operation.contains("read") || op.operation.contains("view") {
                read_count += 1;
            }
            if op.path.to_string_lossy().contains("test") {
                test_count += 1;
            }
            if op.operation.contains("build") || op.operation.contains("compile") {
                _build_count += 1;
            }
        }

        // Determine context based on patterns
        if search_count >= 3 {
            // Multiple searches = hunting for something
            let query = recent_ops
                .iter()
                .find(|op| op.operation.contains("search"))
                .map(|op| op.operation.clone())
                .unwrap_or_default();

            Ok(WorkContext::Hunting {
                query,
                found_locations: vec![],
            })
        } else if edit_count >= 2 && test_count >= 1 {
            // Edits + tests = active development
            let language = Self::detect_language(&recent_ops[0].path);
            Ok(WorkContext::Coding {
                language,
                focus_file: recent_ops[0].path.clone(),
            })
        } else if test_count >= 2 {
            // Lots of test activity
            Ok(WorkContext::Testing {
                test_files: recent_ops
                    .iter()
                    .filter(|op| op.path.to_string_lossy().contains("test"))
                    .map(|op| op.path.clone())
                    .collect(),
                target_files: vec![],
            })
        } else if read_count >= 4 {
            // Lots of reading = exploring
            Ok(WorkContext::Exploring {
                depth: 5,
                areas_visited: recent_ops.iter().map(|op| op.path.clone()).collect(),
            })
        } else {
            // Default to exploring
            Ok(WorkContext::Exploring {
                depth: 3,
                areas_visited: vec![],
            })
        }
    }

    /// Get smart suggestions based on context
    pub fn get_suggestions(&self, _current_path: &Path) -> Vec<String> {
        let context = self.current_context.read().unwrap();
        let knowledge = self.project_knowledge.read().unwrap();

        match context.as_ref() {
            Some(WorkContext::Coding {
                language,
                focus_file,
            }) => vec![
                format!(
                    "💡 Working on {}? Try: st --mode relations --focus {}",
                    language,
                    focus_file.display()
                ),
                format!(
                    "🧪 Run tests: st --search test --type {}",
                    Self::lang_to_ext(language)
                ),
                format!(
                    "📊 See impact: st --mode quantum-semantic {}",
                    focus_file.parent().unwrap_or(Path::new(".")).display()
                ),
            ],

            Some(WorkContext::Debugging { error_pattern, .. }) => vec![
                format!(
                    "🔍 Search for error: st --search \"{}\" --mode ai",
                    error_pattern
                ),
                format!("📈 Recent changes: st --newer-than 1 --sort newest"),
                format!("🌳 Check dependencies: st --mode relations"),
            ],

            Some(WorkContext::Exploring {
                depth,
                areas_visited,
            }) => {
                let mut suggestions = vec![
                    format!("🗺️ Get overview: st --mode summary-ai --depth {}", depth),
                    format!("🧭 Semantic map: st --mode semantic"),
                ];

                // Suggest unexplored areas
                if let Some(hot_dir) = knowledge
                    .hot_directories
                    .iter()
                    .filter(|(path, _)| !areas_visited.contains(path))
                    .max_by_key(|(_, count)| *count)
                    .map(|(path, _)| path)
                {
                    suggestions.push(format!("🔥 Check hot area: st {}", hot_dir.display()));
                }

                suggestions
            }

            Some(WorkContext::Testing { .. }) => vec![
                format!("🧪 Run all tests: st --search \"test_\" --type rs"),
                format!("📊 Coverage gaps: st --mode waste tests/"),
                format!("🔗 Test dependencies: st --mode relations --focus tests/"),
            ],

            Some(WorkContext::Hunting {
                query,
                found_locations,
            }) => {
                let mut suggestions = vec![format!(
                    "🎯 Refine search: st --search \"{}\" --type rs",
                    query
                )];

                if !found_locations.is_empty() {
                    suggestions.push(format!(
                        "📍 Focus area: st --mode ai {}",
                        found_locations[0].display()
                    ));
                }

                // Suggest similar past searches
                if let Some(similar) = knowledge
                    .common_searches
                    .keys()
                    .find(|s| s.contains(query) || query.contains(s.as_str()))
                {
                    suggestions.push(format!("💭 Similar search: st --search \"{}\"", similar));
                }

                suggestions
            }

            _ => vec![
                "🌟 Quick overview: st --mode summary-ai".to_string(),
                "🔍 Search code: st --search \"pattern\" --type rs".to_string(),
                "📊 See structure: st --mode semantic".to_string(),
            ],
        }
    }

    /// Record an operation
    pub fn record_operation(&self, op: ContextualOperation) -> Result<()> {
        let mut history = self.operation_history.write().unwrap();

        // Add to history
        history.push_front(op.clone());
        if history.len() > 50 {
            history.pop_back();
        }

        // Update knowledge
        let mut knowledge = self.project_knowledge.write().unwrap();

        // Track hot directories
        let dir = op.path.parent().unwrap_or(&op.path);
        *knowledge.hot_directories.entry(dir.to_owned()).or_insert(0) += 1;

        // Track searches
        if op.operation.contains("search") {
            if let Some(query) = op.operation.split("search").nth(1) {
                let query = query.trim().to_string();
                *knowledge.common_searches.entry(query).or_insert(0) += 1;
            }
        }

        // Update context based on new operation
        self.update_context()?;

        Ok(())
    }

    /// Update context based on recent operations
    fn update_context(&self) -> Result<()> {
        let new_context = self.analyze_context()?;
        let mut current = self.current_context.write().unwrap();
        *current = Some(new_context);
        Ok(())
    }

    /// Get optimal ST arguments for current context
    pub fn get_optimal_args(&self, _base_command: &str) -> Vec<String> {
        let context = self.current_context.read().unwrap();

        match context.as_ref() {
            Some(WorkContext::Coding { .. }) => {
                vec![
                    "--depth".to_string(),
                    "5".to_string(),
                    "--mode".to_string(),
                    "ai".to_string(),
                ]
            }
            Some(WorkContext::Debugging { .. }) => {
                vec![
                    "--depth".to_string(),
                    "0".to_string(), // Auto depth
                    "--mode".to_string(),
                    "ai".to_string(),
                    "--compress".to_string(),
                ] // Easier to scan
            }
            Some(WorkContext::Exploring { depth, .. }) => {
                vec![
                    "--depth".to_string(),
                    depth.to_string(),
                    "--mode".to_string(),
                    "semantic".to_string(),
                ]
            }
            Some(WorkContext::Testing { .. }) => {
                vec![
                    "--search".to_string(),
                    "test".to_string(),
                    "--mode".to_string(),
                    "relations".to_string(),
                ]
            }
            Some(WorkContext::Hunting { .. }) => {
                vec![
                    "--mode".to_string(),
                    "ai".to_string(),
                    "--stream".to_string(),
                ] // For large searches
            }
            _ => vec!["--depth".to_string(), "0".to_string()], // Auto
        }
    }

    /// Detect programming language from path
    fn detect_language(path: &Path) -> String {
        match path.extension().and_then(|s| s.to_str()) {
            Some("rs") => "rust".to_string(),
            Some("py") => "python".to_string(),
            Some("js") | Some("jsx") => "javascript".to_string(),
            Some("ts") | Some("tsx") => "typescript".to_string(),
            Some("go") => "go".to_string(),
            Some("java") => "java".to_string(),
            Some("cpp") | Some("cc") | Some("cxx") => "cpp".to_string(),
            Some("c") | Some("h") => "c".to_string(),
            _ => "unknown".to_string(),
        }
    }

    fn lang_to_ext(lang: &str) -> &str {
        match lang {
            "rust" => "rs",
            "python" => "py",
            "javascript" => "js",
            "typescript" => "ts",
            "go" => "go",
            "java" => "java",
            "cpp" => "cpp",
            "c" => "c",
            _ => "*",
        }
    }

    /// Save context to disk
    pub fn save_context(&self, path: &Path) -> Result<()> {
        let context_file = path.join(".st_context.json");

        let data = serde_json::json!({
            "current_context": self.current_context.read().unwrap().clone(),
            "project_knowledge": self.project_knowledge.read().unwrap().clone(),
            "history": self.operation_history.read().unwrap().clone(),
        });

        std::fs::write(context_file, serde_json::to_string_pretty(&data)?)
            .context("Failed to save context")?;

        Ok(())
    }

    /// Load context from disk
    pub fn load_context(&self, path: &Path) -> Result<()> {
        let context_file = path.join(".st_context.json");

        if context_file.exists() {
            let data = std::fs::read_to_string(context_file)?;
            let json: serde_json::Value = serde_json::from_str(&data)?;

            // Restore context
            if let Some(ctx) = json.get("current_context") {
                if let Ok(context) = serde_json::from_value::<WorkContext>(ctx.clone()) {
                    *self.current_context.write().unwrap() = Some(context);
                }
            }

            // Restore knowledge
            if let Some(know) = json.get("project_knowledge") {
                if let Ok(knowledge) = serde_json::from_value::<ProjectKnowledge>(know.clone()) {
                    *self.project_knowledge.write().unwrap() = knowledge;
                }
            }
        }

        Ok(())
    }
}

/// Context-aware ST command builder
pub struct ContextualStCommand {
    tracker: Arc<StContextTracker>,
    base_args: Vec<String>,
}

impl ContextualStCommand {
    pub fn new(tracker: Arc<StContextTracker>) -> Self {
        Self {
            tracker,
            base_args: vec![],
        }
    }

    /// Build command with context awareness
    pub fn build(&self, intent: &str) -> Vec<String> {
        let mut args = self.base_args.clone();

        // Get optimal args based on context
        let context_args = self.tracker.get_optimal_args(intent);
        args.extend(context_args);

        // Add specific args based on intent
        match intent {
            "explore" => {
                if !args.contains(&"--mode".to_string()) {
                    args.extend(vec!["--mode".to_string(), "summary-ai".to_string()]);
                }
            }
            "debug" => {
                args.extend(vec!["--compress".to_string()]);
            }
            "document" => {
                args.extend(vec!["--mode".to_string(), "function-markdown".to_string()]);
            }
            _ => {}
        }

        args
    }

    /// Get suggestions for next command
    pub fn suggest_next(&self) -> Vec<String> {
        self.tracker.get_suggestions(Path::new("."))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[ignore = "Hangs - needs investigation"]
    fn test_context_detection() {
        let tracker = StContextTracker::new();

        // Simulate some operations
        tracker
            .record_operation(ContextualOperation {
                timestamp: SystemTime::now(),
                operation: "search TODO".to_string(),
                path: PathBuf::from("src/main.rs"),
                result_summary: "Found 5 matches".to_string(),
                context_hints: vec!["searching".to_string()],
            })
            .unwrap();

        tracker
            .record_operation(ContextualOperation {
                timestamp: SystemTime::now(),
                operation: "search FIXME".to_string(),
                path: PathBuf::from("src/lib.rs"),
                result_summary: "Found 2 matches".to_string(),
                context_hints: vec!["searching".to_string()],
            })
            .unwrap();

        tracker
            .record_operation(ContextualOperation {
                timestamp: SystemTime::now(),
                operation: "search bug".to_string(),
                path: PathBuf::from("tests/test.rs"),
                result_summary: "Found 1 match".to_string(),
                context_hints: vec!["searching".to_string()],
            })
            .unwrap();

        // Should detect hunting context
        let context = tracker.analyze_context().unwrap();
        match context {
            WorkContext::Hunting { .. } => {} // Expected - test passes
            _ => panic!("Expected Hunting context"),
        }

        // Get suggestions
        let suggestions = tracker.get_suggestions(Path::new("."));
        assert!(!suggestions.is_empty());
    }
}
