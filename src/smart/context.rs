//! 🧠 Context Analysis Engine
//!
//! This module provides intelligent context understanding for Smart Tools.
//! It analyzes user tasks, file content, and project structure to determine
//! relevance and priority for maximum token efficiency.

use super::{FocusArea, RelevanceScore, TaskContext};
use crate::scanner::{FileCategory, FileNode};
use std::collections::HashMap;

/// 🎯 Context analyzer that understands user intent and task focus
pub struct ContextAnalyzer {
    /// Keyword mappings for different focus areas
    focus_keywords: HashMap<FocusArea, Vec<String>>,
    /// File type relevance scores
    file_type_scores: HashMap<FileCategory, f32>,
}

impl ContextAnalyzer {
    /// Create a new context analyzer with default mappings
    pub fn new() -> Self {
        let mut analyzer = Self {
            focus_keywords: HashMap::new(),
            file_type_scores: HashMap::new(),
        };

        analyzer.initialize_focus_keywords();
        analyzer.initialize_file_type_scores();
        analyzer
    }

    /// 🔍 Analyze task context from natural language description
    pub fn analyze_task(&self, task_description: &str) -> TaskContext {
        let task_lower = task_description.to_lowercase();
        let mut focus_areas = Vec::new();
        let mut relevance_threshold = 0.6;

        // Detect focus areas from task description
        for (focus_area, keywords) in &self.focus_keywords {
            for keyword in keywords {
                if task_lower.contains(keyword) && !focus_areas.contains(focus_area) {
                    focus_areas.push(focus_area.clone());
                }
            }
        }

        // Adjust relevance threshold based on task specificity
        if focus_areas.len() == 1 {
            relevance_threshold = 0.8; // Very specific task
        } else if focus_areas.len() > 4 {
            relevance_threshold = 0.5; // Broad task
        }

        // Default focus areas if none detected
        if focus_areas.is_empty() {
            focus_areas = vec![FocusArea::API, FocusArea::Configuration];
        }

        TaskContext {
            task: task_description.to_string(),
            focus_areas,
            relevance_threshold,
            max_results: Some(50),
        }
    }

    /// 📊 Score file relevance based on task context
    pub fn score_file_relevance(
        &self,
        file_node: &FileNode,
        context: &TaskContext,
    ) -> RelevanceScore {
        let mut score: f32 = 0.0;
        let mut reasons = Vec::new();
        let mut focus_matches = Vec::new();

        // Base score from file category
        if let Some(type_score) = self.file_type_scores.get(&file_node.category) {
            score += type_score;
            reasons.push(format!("File category {:?} relevance", file_node.category));
        }

        // Score based on filename and path
        let file_path = file_node.path.to_string_lossy().to_lowercase();
        let file_name = file_node
            .path
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or("")
            .to_lowercase();

        for focus_area in &context.focus_areas {
            let empty_vec = Vec::new();
            let keywords = self.focus_keywords.get(focus_area).unwrap_or(&empty_vec);
            let mut area_score = 0.0;

            for keyword in keywords {
                // Check filename
                if file_name.contains(keyword) {
                    area_score += 0.3;
                    reasons.push(format!("Filename contains '{}'", keyword));
                }

                // Check full path
                if file_path.contains(keyword) {
                    area_score += 0.2;
                    reasons.push(format!("Path contains '{}'", keyword));
                }
            }

            if area_score > 0.0 {
                score += area_score;
                focus_matches.push(focus_area.clone());
            }
        }

        // Boost score for common important files
        if self.is_important_file(&file_name) {
            score += 0.4;
            reasons.push("Important project file".to_string());
        }

        // Normalize score to 0.0-1.0 range
        score = score.min(1.0);

        RelevanceScore {
            score,
            reasons,
            focus_matches,
        }
    }

    /// 📁 Score directory relevance based on contents and context
    pub fn score_directory_relevance(
        &self,
        dir_node: &FileNode,
        context: &TaskContext,
    ) -> RelevanceScore {
        let mut score: f32 = 0.0;
        let mut reasons = Vec::new();
        let mut focus_matches = Vec::new();

        let dir_name = dir_node
            .path
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or("")
            .to_lowercase();
        let dir_path = dir_node.path.to_string_lossy().to_lowercase();

        // Score based on directory name and common patterns
        for focus_area in &context.focus_areas {
            let empty_vec = Vec::new();
            let keywords = self.focus_keywords.get(focus_area).unwrap_or(&empty_vec);

            for keyword in keywords {
                if dir_name.contains(keyword) || dir_path.contains(keyword) {
                    score += 0.4;
                    reasons.push(format!("Directory name/path contains '{}'", keyword));
                    if !focus_matches.contains(focus_area) {
                        focus_matches.push(focus_area.clone());
                    }
                }
            }
        }

        // Boost score for important directories
        if self.is_important_directory(&dir_name) {
            score += 0.3;
            reasons.push("Important project directory".to_string());
        }

        // Penalize common unimportant directories
        if self.is_unimportant_directory(&dir_name) {
            score *= 0.2;
            reasons.push("Low-priority directory".to_string());
        }

        score = score.min(1.0);

        RelevanceScore {
            score,
            reasons,
            focus_matches,
        }
    }

    /// Initialize focus area keyword mappings
    fn initialize_focus_keywords(&mut self) {
        // Authentication keywords
        self.focus_keywords.insert(
            FocusArea::Authentication,
            vec![
                "auth",
                "login",
                "password",
                "token",
                "session",
                "jwt",
                "oauth",
                "signin",
                "signup",
                "credential",
                "authenticate",
                "authorize",
            ]
            .into_iter()
            .map(String::from)
            .collect(),
        );

        // API keywords
        self.focus_keywords.insert(
            FocusArea::API,
            vec![
                "api",
                "endpoint",
                "route",
                "handler",
                "request",
                "response",
                "http",
                "rest",
                "graphql",
                "controller",
                "service",
                "client",
            ]
            .into_iter()
            .map(String::from)
            .collect(),
        );

        // Database keywords
        self.focus_keywords.insert(
            FocusArea::Database,
            vec![
                "db",
                "database",
                "sql",
                "query",
                "table",
                "schema",
                "migration",
                "model",
                "entity",
                "repository",
                "dao",
                "orm",
            ]
            .into_iter()
            .map(String::from)
            .collect(),
        );

        // Frontend keywords
        self.focus_keywords.insert(
            FocusArea::Frontend,
            vec![
                "ui",
                "component",
                "react",
                "vue",
                "angular",
                "html",
                "css",
                "js",
                "frontend",
                "client",
                "view",
                "template",
                "style",
            ]
            .into_iter()
            .map(String::from)
            .collect(),
        );

        // Backend keywords
        self.focus_keywords.insert(
            FocusArea::Backend,
            vec![
                "server",
                "service",
                "controller",
                "model",
                "business",
                "logic",
                "backend",
                "core",
                "engine",
                "processor",
            ]
            .into_iter()
            .map(String::from)
            .collect(),
        );

        // Testing keywords
        self.focus_keywords.insert(
            FocusArea::Testing,
            vec![
                "test",
                "spec",
                "mock",
                "assert",
                "expect",
                "unit",
                "integration",
                "e2e",
                "fixture",
                "stub",
                "spy",
            ]
            .into_iter()
            .map(String::from)
            .collect(),
        );

        // Configuration keywords
        self.focus_keywords.insert(
            FocusArea::Configuration,
            vec![
                "config",
                "env",
                "settings",
                "properties",
                "yaml",
                "json",
                "toml",
                "ini",
                "conf",
                "cfg",
                "setup",
            ]
            .into_iter()
            .map(String::from)
            .collect(),
        );

        // Security keywords
        self.focus_keywords.insert(
            FocusArea::Security,
            vec![
                "security",
                "vulnerability",
                "sanitize",
                "validate",
                "encrypt",
                "hash",
                "secure",
                "crypto",
                "ssl",
                "tls",
                "cert",
            ]
            .into_iter()
            .map(String::from)
            .collect(),
        );

        // Performance keywords
        self.focus_keywords.insert(
            FocusArea::Performance,
            vec![
                "performance",
                "optimize",
                "cache",
                "memory",
                "cpu",
                "benchmark",
                "perf",
                "speed",
                "fast",
                "efficient",
            ]
            .into_iter()
            .map(String::from)
            .collect(),
        );

        // Documentation keywords
        self.focus_keywords.insert(
            FocusArea::Documentation,
            vec![
                "doc",
                "readme",
                "comment",
                "documentation",
                "guide",
                "manual",
                "help",
                "tutorial",
                "example",
            ]
            .into_iter()
            .map(String::from)
            .collect(),
        );
    }

    /// Initialize file category relevance scores
    fn initialize_file_type_scores(&mut self) {
        // High relevance programming languages
        self.file_type_scores.insert(FileCategory::Rust, 0.9);
        self.file_type_scores.insert(FileCategory::Python, 0.8);
        self.file_type_scores.insert(FileCategory::JavaScript, 0.8);
        self.file_type_scores.insert(FileCategory::TypeScript, 0.8);
        self.file_type_scores.insert(FileCategory::Go, 0.8);
        self.file_type_scores.insert(FileCategory::Java, 0.8);
        self.file_type_scores.insert(FileCategory::C, 0.7);
        self.file_type_scores.insert(FileCategory::Cpp, 0.7);

        // Configuration and markup files
        self.file_type_scores.insert(FileCategory::Json, 0.6);
        self.file_type_scores.insert(FileCategory::Yaml, 0.6);
        self.file_type_scores.insert(FileCategory::Toml, 0.6);
        self.file_type_scores.insert(FileCategory::Markdown, 0.6);
        self.file_type_scores.insert(FileCategory::Html, 0.5);
        self.file_type_scores.insert(FileCategory::Css, 0.5);

        // Build and system files
        self.file_type_scores.insert(FileCategory::Makefile, 0.5);
        self.file_type_scores.insert(FileCategory::Dockerfile, 0.5);
        self.file_type_scores.insert(FileCategory::GitConfig, 0.4);

        // Lower relevance files
        self.file_type_scores.insert(FileCategory::Archive, 0.2);
        self.file_type_scores.insert(FileCategory::Image, 0.2);
        self.file_type_scores.insert(FileCategory::Video, 0.1);
        self.file_type_scores.insert(FileCategory::Audio, 0.1);
        self.file_type_scores.insert(FileCategory::Binary, 0.2);
        self.file_type_scores.insert(FileCategory::Unknown, 0.3);
    }

    /// Check if file is commonly important
    fn is_important_file(&self, filename: &str) -> bool {
        matches!(
            filename,
            "readme.md"
                | "cargo.toml"
                | "package.json"
                | "requirements.txt"
                | "dockerfile"
                | "docker-compose.yml"
                | "makefile"
                | ".gitignore"
                | "main.rs"
                | "lib.rs"
                | "mod.rs"
                | "index.js"
                | "app.py"
                | "main.py"
        )
    }

    /// Check if directory is commonly important
    fn is_important_directory(&self, dirname: &str) -> bool {
        matches!(
            dirname,
            "src"
                | "lib"
                | "api"
                | "server"
                | "client"
                | "frontend"
                | "backend"
                | "components"
                | "services"
                | "controllers"
                | "models"
                | "routes"
                | "config"
                | "configs"
                | "auth"
                | "authentication"
        )
    }

    /// Check if directory is commonly unimportant
    fn is_unimportant_directory(&self, dirname: &str) -> bool {
        matches!(
            dirname,
            "node_modules"
                | "target"
                | "dist"
                | "build"
                | ".git"
                | ".vscode"
                | "vendor"
                | "__pycache__"
                | ".pytest_cache"
                | "coverage"
                | "logs"
        )
    }
}

impl Default for ContextAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::scanner::{FileCategory, FileType, FilesystemType};
    use std::path::PathBuf;
    use std::time::SystemTime;

    #[test]
    fn test_task_analysis() {
        let analyzer = ContextAnalyzer::new();
        let context = analyzer.analyze_task("debugging authentication issues in the API");

        assert!(context.focus_areas.contains(&FocusArea::Authentication));
        assert!(context.focus_areas.contains(&FocusArea::API));
        assert_eq!(context.relevance_threshold, 0.6);
    }

    #[test]
    fn test_file_relevance_scoring() {
        let analyzer = ContextAnalyzer::new();
        let context = TaskContext {
            task: "API debugging".to_string(),
            focus_areas: vec![FocusArea::API],
            relevance_threshold: 0.6,
            max_results: Some(50),
        };

        let file_node = FileNode {
            path: PathBuf::from("src/api/api_handler.rs"),
            is_dir: false,
            size: 1024,
            permissions: 0o644,
            uid: 1000,
            gid: 1000,
            modified: SystemTime::now(),
            is_symlink: false,
            is_hidden: false,
            permission_denied: false,
            is_ignored: false,
            depth: 1,
            file_type: FileType::RegularFile,
            category: FileCategory::Rust,
            search_matches: None,
            filesystem_type: FilesystemType::Ext4,
            git_branch: None,
            traversal_context: None,
            interest: None,
            security_findings: Vec::new(),
            change_status: None,
            content_hash: None,
        };

        let score = analyzer.score_file_relevance(&file_node, &context);
        assert!(score.score > 0.5); // Should be highly relevant
        assert!(!score.reasons.is_empty());
    }
}
