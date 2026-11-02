//! Intelligent MCP Assistant - The best helper of all time!
//!
//! This module makes Smart Tree MCP incredibly helpful by:
//! - Suggesting the next best tools based on context
//! - Learning from usage patterns
//! - Providing helpful hints and tips
//! - Anticipating needs before they're expressed

// use anyhow::Result; // TODO: Use when needed
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::collections::{HashMap, VecDeque};
use std::sync::Arc;
use tokio::sync::RwLock;

/// Tool recommendation based on context
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolRecommendation {
    /// The tool name
    pub tool: String,
    /// Why this tool is recommended
    pub reason: String,
    /// Confidence score (0.0 to 1.0)
    pub confidence: f32,
    /// Example usage
    pub example: String,
    /// Related tools that work well together
    pub companions: Vec<String>,
}

/// Pattern of tool usage that we've learned
#[derive(Debug, Clone)]
pub struct UsagePattern {
    /// Sequence of tools commonly used together
    pub sequence: Vec<String>,
    /// How often this pattern occurs
    pub frequency: u32,
    /// Context where this pattern is useful
    pub context: String,
}

/// The intelligent assistant that helps users
pub struct McpAssistant {
    /// History of recent tool calls
    call_history: Arc<RwLock<VecDeque<String>>>,
    /// Learned patterns of tool usage
    patterns: Arc<RwLock<Vec<UsagePattern>>>,
    /// Current project context
    project_context: Arc<RwLock<ProjectContext>>,
    /// Tool usage statistics
    usage_stats: Arc<RwLock<HashMap<String, u32>>>,
}

/// Project context for smarter recommendations
#[derive(Debug, Clone)]
pub struct ProjectContext {
    pub language: ProjectLanguage,
    pub size: ProjectSize,
    pub has_tests: bool,
    pub has_git: bool,
    pub has_ci: bool,
    pub file_count: usize,
    pub recent_changes: Vec<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ProjectLanguage {
    Rust,
    Python,
    JavaScript,
    TypeScript,
    Mixed,
    Unknown,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ProjectSize {
    Tiny,   // < 10 files
    Small,  // 10-50 files
    Medium, // 50-200 files
    Large,  // 200-1000 files
    Huge,   // 1000+ files
}

impl Default for McpAssistant {
    fn default() -> Self {
        Self::new()
    }
}

impl McpAssistant {
    /// Create a new assistant
    pub fn new() -> Self {
        Self {
            call_history: Arc::new(RwLock::new(VecDeque::with_capacity(20))),
            patterns: Arc::new(RwLock::new(Self::load_default_patterns())),
            project_context: Arc::new(RwLock::new(ProjectContext::default())),
            usage_stats: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Load default patterns that we know work well
    fn load_default_patterns() -> Vec<UsagePattern> {
        vec![
            UsagePattern {
                sequence: vec!["overview".into(), "find".into(), "search".into()],
                frequency: 100,
                context: "Initial project exploration".into(),
            },
            UsagePattern {
                sequence: vec!["find_code_files".into(), "analyze".into(), "edit".into()],
                frequency: 80,
                context: "Code modification workflow".into(),
            },
            UsagePattern {
                sequence: vec!["history".into(), "compare".into(), "analyze".into()],
                frequency: 60,
                context: "Understanding recent changes".into(),
            },
            UsagePattern {
                sequence: vec!["find_tests".into(), "analyze".into(), "edit".into()],
                frequency: 70,
                context: "Test development workflow".into(),
            },
            UsagePattern {
                sequence: vec![
                    "find_config_files".into(),
                    "edit".into(),
                    "verify_permissions".into(),
                ],
                frequency: 50,
                context: "Configuration management".into(),
            },
        ]
    }

    /// Record a tool call and update statistics
    pub async fn record_call(&self, tool_name: &str) {
        let mut history = self.call_history.write().await;
        history.push_back(tool_name.to_string());
        if history.len() > 20 {
            history.pop_front();
        }

        let mut stats = self.usage_stats.write().await;
        *stats.entry(tool_name.to_string()).or_insert(0) += 1;
    }

    /// Get intelligent recommendations for next tools
    pub async fn get_recommendations(&self, last_tool: &str) -> Vec<ToolRecommendation> {
        let mut recommendations = Vec::new();

        let history = self.call_history.read().await;
        let patterns = self.patterns.read().await;
        let context = self.project_context.read().await;

        // Check if we're in a known pattern
        for pattern in patterns.iter() {
            if let Some(pos) = pattern.sequence.iter().position(|t| t == last_tool) {
                if pos < pattern.sequence.len() - 1 {
                    // Suggest the next tool in the pattern
                    let next_tool = &pattern.sequence[pos + 1];
                    recommendations.push(ToolRecommendation {
                        tool: next_tool.clone(),
                        reason: format!("Part of {} workflow", pattern.context),
                        confidence: pattern.frequency as f32 / 100.0,
                        example: self.get_example_for_tool(next_tool),
                        companions: pattern.sequence.clone(),
                    });
                }
            }
        }

        // Context-based recommendations
        match last_tool {
            "overview" | "quick_tree" => {
                recommendations.push(ToolRecommendation {
                    tool: "find".into(),
                    reason: "Natural next step: find specific files after overview".into(),
                    confidence: 0.9,
                    example: r#"{"type": "code", "languages": ["rust"]}"#.into(),
                    companions: vec!["search".into(), "analyze".into()],
                });

                if context.has_tests {
                    recommendations.push(ToolRecommendation {
                        tool: "find_tests".into(),
                        reason: "Project has tests - explore test structure".into(),
                        confidence: 0.7,
                        example: r#"{"path": "."}"#.into(),
                        companions: vec!["analyze".into()],
                    });
                }
            }

            "find" | "find_files" => {
                recommendations.push(ToolRecommendation {
                    tool: "search".into(),
                    reason: "Search within the files you found".into(),
                    confidence: 0.85,
                    example: r#"{"keyword": "TODO", "include_content": true}"#.into(),
                    companions: vec!["analyze".into(), "edit".into()],
                });

                recommendations.push(ToolRecommendation {
                    tool: "analyze".into(),
                    reason: "Analyze the structure of found files".into(),
                    confidence: 0.8,
                    example: r#"{"mode": "semantic"}"#.into(),
                    companions: vec!["edit".into()],
                });
            }

            "search" => {
                recommendations.push(ToolRecommendation {
                    tool: "edit".into(),
                    reason: "Edit files containing search results".into(),
                    confidence: 0.75,
                    example: r#"{"operation": "smart_edit", "file_path": "src/main.rs"}"#.into(),
                    companions: vec!["history".into()],
                });

                recommendations.push(ToolRecommendation {
                    tool: "context".into(),
                    reason: "Get broader context around search results".into(),
                    confidence: 0.7,
                    example: r#"{"operation": "gather_project"}"#.into(),
                    companions: vec!["memory".into()],
                });
            }

            "edit" => {
                recommendations.push(ToolRecommendation {
                    tool: "history".into(),
                    reason: "Track changes you've made".into(),
                    confidence: 0.9,
                    example: r#"{"operation": "get_file", "file_path": "src/main.rs"}"#.into(),
                    companions: vec!["compare".into()],
                });

                recommendations.push(ToolRecommendation {
                    tool: "verify_permissions".into(),
                    reason: "Verify file permissions after edit".into(),
                    confidence: 0.6,
                    example: r#"{"path": "."}"#.into(),
                    companions: vec!["analyze".into()],
                });
            }

            "analyze" => {
                if context.size == ProjectSize::Large || context.size == ProjectSize::Huge {
                    recommendations.push(ToolRecommendation {
                        tool: "analyze".into(),
                        reason: "Try quantum compression for this large project".into(),
                        confidence: 0.85,
                        example: r#"{"mode": "quantum_semantic"}"#.into(),
                        companions: vec!["memory".into()],
                    });
                }

                recommendations.push(ToolRecommendation {
                    tool: "memory".into(),
                    reason: "Save important insights from analysis".into(),
                    confidence: 0.7,
                    example: r#"{"operation": "anchor", "keywords": ["architecture"]}"#.into(),
                    companions: vec!["context".into()],
                });
            }

            _ => {
                // Generic helpful recommendations
                if context.has_git && !history.contains(&"history".to_string()) {
                    recommendations.push(ToolRecommendation {
                        tool: "history".into(),
                        reason: "Explore git history for context".into(),
                        confidence: 0.6,
                        example: r#"{"operation": "get_project"}"#.into(),
                        companions: vec!["compare".into()],
                    });
                }
            }
        }

        // Sort by confidence
        recommendations.sort_by(|a, b| b.confidence.partial_cmp(&a.confidence).unwrap());
        recommendations.truncate(3); // Top 3 recommendations

        recommendations
    }

    /// Get example usage for a tool
    fn get_example_for_tool(&self, tool: &str) -> String {
        match tool {
            "overview" => r#"{"mode": "project", "path": "."}"#,
            "find" => r#"{"type": "code", "languages": ["rust", "python"]}"#,
            "search" => r#"{"keyword": "TODO|FIXME", "include_content": true}"#,
            "analyze" => r#"{"mode": "semantic", "path": "."}"#,
            "edit" => r#"{"operation": "smart_edit", "file_path": "src/lib.rs"}"#,
            "history" => r#"{"operation": "get_project", "project_path": "."}"#,
            "context" => r#"{"operation": "gather_project"}"#,
            "memory" => r#"{"operation": "find", "keywords": ["performance"]}"#,
            _ => "{}",
        }
        .to_string()
    }

    /// Update project context from analysis
    pub async fn update_context(&self, analysis: Value) {
        let mut context = self.project_context.write().await;

        // Extract information from analysis
        if let Some(files) = analysis.get("file_count").and_then(|v| v.as_u64()) {
            context.file_count = files as usize;
            context.size = match files {
                0..=10 => ProjectSize::Tiny,
                11..=50 => ProjectSize::Small,
                51..=200 => ProjectSize::Medium,
                201..=1000 => ProjectSize::Large,
                _ => ProjectSize::Huge,
            };
        }

        if let Some(lang) = analysis.get("primary_language").and_then(|v| v.as_str()) {
            context.language = match lang {
                "rust" => ProjectLanguage::Rust,
                "python" => ProjectLanguage::Python,
                "javascript" => ProjectLanguage::JavaScript,
                "typescript" => ProjectLanguage::TypeScript,
                _ => ProjectLanguage::Unknown,
            };
        }

        context.has_git = analysis
            .get("has_git")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);
        context.has_tests = analysis
            .get("has_tests")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);
    }

    /// Get helpful tips based on current context
    pub async fn get_helpful_tips(&self) -> Vec<String> {
        let mut tips = Vec::new();
        let context = self.project_context.read().await;
        let stats = self.usage_stats.read().await;

        // Size-based tips
        match context.size {
            ProjectSize::Huge => {
                tips.push("💡 Large project detected! Use 'quantum-semantic' mode for maximum compression".into());
                tips.push("🚀 Try streaming mode with --stream flag for faster results".into());
            }
            ProjectSize::Large => {
                tips.push("📊 Consider using 'quantum' mode for better token efficiency".into());
            }
            _ => {}
        }

        // Language-specific tips
        match context.language {
            ProjectLanguage::Rust => {
                tips.push("🦀 Rust project! Use 'find_tests' to locate test modules".into());
                tips.push(
                    "📦 Try 'analyze' with mode 'relations' to see module dependencies".into(),
                );
            }
            ProjectLanguage::Python => {
                tips.push(
                    "🐍 Python project! Use 'find_config_files' to locate setup.py/pyproject.toml"
                        .into(),
                );
            }
            _ => {}
        }

        // Usage-based tips
        if !stats.contains_key("memory") {
            tips.push(
                "💭 Haven't used memory yet? Try 'memory' tool to save important insights!".into(),
            );
        }

        if !stats.contains_key("compare") && context.has_git {
            tips.push(
                "🔄 Git repo detected! Use 'compare' to see differences between directories".into(),
            );
        }

        tips
    }

    /// Generate a friendly, helpful response
    pub async fn enhance_response(&self, tool: &str, response: Value) -> Value {
        let recommendations = self.get_recommendations(tool).await;
        let tips = self.get_helpful_tips().await;

        let mut enhanced = response;

        // Add recommendations if we have them
        if !recommendations.is_empty() {
            let recs: Vec<Value> = recommendations
                .iter()
                .map(|r| {
                    json!({
                        "tool": r.tool,
                        "reason": r.reason,
                        "confidence": r.confidence,
                        "example": r.example,
                    })
                })
                .collect();

            enhanced["_suggestions"] = json!({
                "next_tools": recs,
                "message": format!(
                    "🎯 Based on '{}', you might want to try '{}' next!",
                    tool,
                    recommendations[0].tool
                ),
            });
        }

        // Add helpful tips
        if !tips.is_empty() {
            enhanced["_tips"] = json!(tips);
        }

        // Add friendly message
        enhanced["_assistant"] = json!({
            "message": self.get_friendly_message(tool).await,
            "confidence": "I'm learning from your usage patterns to provide better suggestions!",
        });

        enhanced
    }

    /// Get a friendly message based on the tool
    async fn get_friendly_message(&self, tool: &str) -> String {
        let history = self.call_history.read().await;
        let call_count = history.len();

        match tool {
            "overview" if call_count == 0 => {
                "🌟 Great start! I've analyzed your project structure. What would you like to explore next?".into()
            },
            "find" => {
                "🔍 Found what you're looking for? I can help you search within these files or analyze their structure!".into()
            },
            "search" => {
                "🎯 Search complete! Would you like to edit these files or get more context around the results?".into()
            },
            "edit" => {
                "✏️ Edit successful! I'm tracking your changes. Want to see the history or make more edits?".into()
            },
            "analyze" => {
                "📊 Analysis complete! This data is now cached for faster access. Consider saving important insights with the memory tool!".into()
            },
            _ => {
                "✨ Operation complete! Check out my suggestions for what to do next!".into()
            }
        }
    }

    /// Learn from a successful sequence of operations
    pub async fn learn_pattern(&self, sequence: Vec<String>, context: String) {
        let mut patterns = self.patterns.write().await;

        // Check if this pattern already exists
        for pattern in patterns.iter_mut() {
            if pattern.sequence == sequence {
                pattern.frequency += 1;
                return;
            }
        }

        // Add new pattern
        patterns.push(UsagePattern {
            sequence,
            frequency: 1,
            context,
        });
    }
}

impl Default for ProjectContext {
    fn default() -> Self {
        Self {
            language: ProjectLanguage::Unknown,
            size: ProjectSize::Small,
            has_tests: false,
            has_git: false,
            has_ci: false,
            file_count: 0,
            recent_changes: Vec::new(),
        }
    }
}

/// Helper function to make tool responses more helpful
pub fn make_helpful(tool_name: &str, basic_response: Value) -> Value {
    let mut response = basic_response;

    // Add quick tips based on tool
    let quick_tip = match tool_name {
        "overview" => "Try 'find' next to locate specific file types!",
        "find" => "Use 'search' to look for patterns within these files!",
        "search" => "Found something interesting? Use 'edit' to modify it!",
        "edit" => "Check 'history' to track your changes!",
        "analyze" => "Save insights with 'memory' for future reference!",
        _ => "I'm here to help! Check suggestions for next steps!",
    };

    response["_quick_tip"] = json!(quick_tip);

    response
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_recommendations() {
        let assistant = McpAssistant::new();

        // Record some calls
        assistant.record_call("overview").await;

        // Get recommendations
        let recs = assistant.get_recommendations("overview").await;

        assert!(!recs.is_empty());
        assert_eq!(recs[0].tool, "find");
        assert!(recs[0].confidence > 0.5);
    }

    #[tokio::test]
    async fn test_pattern_learning() {
        let assistant = McpAssistant::new();

        // Learn a new pattern
        assistant
            .learn_pattern(
                vec!["custom1".into(), "custom2".into()],
                "Custom workflow".into(),
            )
            .await;

        // Should recommend custom2 after custom1
        assistant.record_call("custom1").await;
        let recs = assistant.get_recommendations("custom1").await;

        assert!(recs.iter().any(|r| r.tool == "custom2"));
    }
}
