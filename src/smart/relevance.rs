//! 📊 Relevance Engine - Advanced Scoring Algorithms
//! 
//! This module provides sophisticated relevance scoring algorithms that
//! understand code semantics, project structure, and task context to
//! achieve maximum token efficiency.

use super::{FocusArea, RelevanceScore, TaskContext};
use crate::scanner::{FileNode, FileCategory};
use std::collections::HashMap;

/// 🎯 Advanced relevance scoring engine
pub struct RelevanceEngine {
    /// Cached scoring patterns
    patterns: HashMap<String, f32>,
    /// File type weights
    type_weights: HashMap<FileCategory, f32>,
}

impl RelevanceEngine {
    /// Create new relevance engine
    pub fn new() -> Self {
        let mut engine = Self {
            patterns: HashMap::new(),
            type_weights: HashMap::new(),
        };
        
        engine.initialize_patterns();
        engine.initialize_type_weights();
        engine
    }
    
    /// 🔍 Score file relevance with advanced algorithms
    pub fn score_advanced_relevance(
        &self,
        file_node: &FileNode,
        context: &TaskContext,
        project_context: Option<&ProjectContext>,
    ) -> RelevanceScore {
        let mut score = 0.0;
        let mut reasons = Vec::new();
        let mut focus_matches = Vec::new();
        
        // Base file category score
        if let Some(weight) = self.type_weights.get(&file_node.category) {
            score += weight;
            reasons.push(format!("File category {:?} base score", file_node.category));
        }
        
        // Pattern matching score
        let file_path = file_node.path.to_string_lossy().to_lowercase();
        for (pattern, pattern_score) in &self.patterns {
            if file_path.contains(pattern) {
                score += pattern_score;
                reasons.push(format!("Matches pattern '{}'", pattern));
            }
        }
        
        // Focus area matching
        for focus_area in &context.focus_areas {
            let focus_score = self.calculate_focus_score(&file_node, focus_area);
            if focus_score > 0.0 {
                score += focus_score;
                focus_matches.push(focus_area.clone());
                reasons.push(format!("Relevant to {:?}", focus_area));
            }
        }
        
        // Project context boost
        if let Some(proj_ctx) = project_context {
            score += self.calculate_project_context_boost(&file_node, proj_ctx);
        }
        
        // Normalize score
        score = score.min(1.0);
        
        RelevanceScore {
            score,
            reasons,
            focus_matches,
        }
    }
    
    /// Calculate focus-specific scoring
    fn calculate_focus_score(&self, file_node: &FileNode, focus_area: &FocusArea) -> f32 {
        let file_name = file_node.path.file_name()
            .and_then(|name| name.to_str())
            .unwrap_or("")
            .to_lowercase();
        let file_path = file_node.path.to_string_lossy().to_lowercase();
        
        let keywords = focus_area.keywords();
        let mut score = 0.0;
        
        for keyword in keywords {
            if file_name.contains(keyword) {
                score += 0.4; // High score for filename match
            } else if file_path.contains(keyword) {
                score += 0.2; // Medium score for path match
            }
        }
        
        score
    }
    
    /// Calculate project context boost
    fn calculate_project_context_boost(&self, file_node: &FileNode, context: &ProjectContext) -> f32 {
        let mut boost = 0.0;
        
        // Recently modified files get boost
        if context.recent_files.contains(&file_node.path) {
            boost += 0.3;
        }
        
        // Core project files get boost
        if context.core_files.contains(&file_node.path) {
            boost += 0.4;
        }
        
        boost
    }
    
    /// Initialize scoring patterns
    fn initialize_patterns(&mut self) {
        // Authentication patterns
        self.patterns.insert("auth".to_string(), 0.3);
        self.patterns.insert("login".to_string(), 0.3);
        self.patterns.insert("session".to_string(), 0.2);
        
        // API patterns
        self.patterns.insert("api".to_string(), 0.3);
        self.patterns.insert("endpoint".to_string(), 0.3);
        self.patterns.insert("handler".to_string(), 0.2);
        
        // Configuration patterns
        self.patterns.insert("config".to_string(), 0.2);
        self.patterns.insert("env".to_string(), 0.2);
        self.patterns.insert("settings".to_string(), 0.2);
        
        // Test patterns
        self.patterns.insert("test".to_string(), 0.2);
        self.patterns.insert("spec".to_string(), 0.2);
        
        // Documentation patterns
        self.patterns.insert("readme".to_string(), 0.2);
        self.patterns.insert("doc".to_string(), 0.1);
    }
    
    /// Initialize file type weights
    fn initialize_type_weights(&mut self) {
        self.type_weights.insert(FileCategory::Rust, 0.8);
        self.type_weights.insert(FileCategory::Python, 0.8);
        self.type_weights.insert(FileCategory::JavaScript, 0.7);
        self.type_weights.insert(FileCategory::TypeScript, 0.7);
        self.type_weights.insert(FileCategory::Json, 0.5);
        self.type_weights.insert(FileCategory::Yaml, 0.5);
        self.type_weights.insert(FileCategory::Markdown, 0.4);
        self.type_weights.insert(FileCategory::Unknown, 0.3);
    }
}

/// 🏗️ Project context for enhanced relevance scoring
#[derive(Debug, Clone)]
pub struct ProjectContext {
    /// Recently modified files
    pub recent_files: Vec<std::path::PathBuf>,
    /// Core project files (main.rs, package.json, etc.)
    pub core_files: Vec<std::path::PathBuf>,
    /// Project type (rust, node, python, etc.)
    pub project_type: ProjectType,
}

/// 🏷️ Project type detection
#[derive(Debug, Clone, PartialEq)]
pub enum ProjectType {
    Rust,
    Node,
    Python,
    Go,
    Java,
    Mixed,
    Unknown,
}

impl Default for RelevanceEngine {
    fn default() -> Self {
        Self::new()
    }
}
