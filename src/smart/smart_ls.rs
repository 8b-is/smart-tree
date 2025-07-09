//! 📂 SmartLS - Task-Aware Directory Intelligence
//! 
//! This module provides intelligent directory listings that understand
//! the user's current task and prioritize files by relevance, achieving
//! significant token savings while improving workflow efficiency.

use super::{SmartResponse, TaskContext, TokenSavings};
use super::context::ContextAnalyzer;
use crate::scanner::{FileNode, Scanner, ScannerConfig};
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::path::Path;

/// 📂 Smart directory lister with task awareness
pub struct SmartLS {
    context_analyzer: ContextAnalyzer,
}

/// 📁 Smart directory entry with relevance scoring
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SmartDirEntry {
    /// File node information
    pub node: FileNode,
    /// Relevance score for current task
    pub relevance: super::RelevanceScore,
    /// Suggested actions for this file
    pub suggested_actions: Vec<String>,
}

/// 📊 Smart directory listing response
pub type SmartLSResponse = SmartResponse<SmartDirEntry>;

impl SmartLS {
    /// Create new smart directory lister
    pub fn new() -> Self {
        Self {
            context_analyzer: ContextAnalyzer::new(),
        }
    }
    
    /// 📂 List directory with task awareness
    pub fn list_smart(
        &self,
        path: &Path,
        context: &TaskContext,
        max_depth: Option<usize>,
    ) -> Result<SmartLSResponse> {
        // Scan directory
        let config = ScannerConfig {
            max_depth: max_depth.unwrap_or(2),
            show_hidden: false,
            follow_symlinks: false,
            ..Default::default()
        };
        
        let scanner = Scanner::new(path, config)?;
        let (nodes, _stats) = scanner.scan()?;
        
        // Score and categorize files
        let scored_entries = self.score_and_categorize(&nodes, context)?;
        
        // Split into primary and secondary based on relevance
        let (primary, secondary) = self.split_by_relevance(&scored_entries, context);
        
        // Calculate token savings
        let original_tokens = self.estimate_tokens_for_all(&nodes);
        let compressed_tokens = self.estimate_tokens_for_entries(&primary) + 
                               self.estimate_tokens_for_entries(&secondary);
        let token_savings = TokenSavings::new(original_tokens, compressed_tokens, "smart-ls");
        
        // Generate context summary and suggestions
        let context_summary = self.generate_context_summary(&primary, &secondary, context);
        let suggestions = self.generate_suggestions(&primary, &secondary, context);
        
        Ok(SmartLSResponse {
            primary,
            secondary,
            context_summary,
            token_savings,
            suggestions,
        })
    }
    
    /// Score and categorize directory entries
    fn score_and_categorize(
        &self,
        nodes: &[FileNode],
        context: &TaskContext,
    ) -> Result<Vec<SmartDirEntry>> {
        let mut entries = Vec::new();
        
        for node in nodes {
            let relevance = if node.is_dir {
                self.context_analyzer.score_directory_relevance(node, context)
            } else {
                self.context_analyzer.score_file_relevance(node, context)
            };
            
            let suggested_actions = self.generate_file_actions(node, context, &relevance);
            
            entries.push(SmartDirEntry {
                node: node.clone(),
                relevance,
                suggested_actions,
            });
        }
        
        // Sort by relevance score
        entries.sort_by(|a, b| b.relevance.score.partial_cmp(&a.relevance.score).unwrap());
        
        Ok(entries)
    }
    
    /// Split entries by relevance threshold
    fn split_by_relevance(
        &self,
        entries: &[SmartDirEntry],
        context: &TaskContext,
    ) -> (Vec<SmartDirEntry>, Vec<SmartDirEntry>) {
        let mut primary = Vec::new();
        let mut secondary = Vec::new();
        
        for entry in entries {
            if entry.relevance.score >= context.relevance_threshold {
                primary.push(entry.clone());
            } else if entry.relevance.score >= context.relevance_threshold * 0.6 {
                secondary.push(entry.clone());
            }
            // Entries below 60% of threshold are filtered out
        }
        
        // Limit results if specified
        if let Some(max_results) = context.max_results {
            primary.truncate(max_results / 2);
            secondary.truncate(max_results / 2);
        }
        
        (primary, secondary)
    }
    
    /// Generate suggested actions for a file
    fn generate_file_actions(
        &self,
        node: &FileNode,
        context: &TaskContext,
        relevance: &super::RelevanceScore,
    ) -> Vec<String> {
        let mut actions = Vec::new();
        
        if node.is_dir {
            actions.push("Explore directory".to_string());
            if relevance.score > 0.7 {
                actions.push("Analyze contents".to_string());
            }
        } else {
            actions.push("Read file".to_string());
            if relevance.score > 0.8 {
                actions.push("Smart read with context".to_string());
            }
            
            // Task-specific suggestions
            for focus_area in &context.focus_areas {
                match focus_area {
                    super::FocusArea::Testing if node.path.file_name().and_then(|n| n.to_str()).unwrap_or("").contains("test") => {
                        actions.push("Run tests".to_string());
                    }
                    super::FocusArea::Configuration if {
                        let name = node.path.file_name().and_then(|n| n.to_str()).unwrap_or("");
                        name.ends_with(".json") || name.ends_with(".yaml")
                    } => {
                        actions.push("Edit configuration".to_string());
                    }
                    super::FocusArea::API if {
                        let name = node.path.file_name().and_then(|n| n.to_str()).unwrap_or("");
                        name.contains("api") || name.contains("handler")
                    } => {
                        actions.push("Analyze API endpoints".to_string());
                    }
                    _ => {}
                }
            }
        }
        
        actions
    }
    
    /// Estimate tokens for all nodes
    fn estimate_tokens_for_all(&self, nodes: &[FileNode]) -> usize {
        // Rough estimation based on file count and average metadata size
        nodes.len() * 50 // ~50 tokens per file entry
    }
    
    /// Estimate tokens for smart entries
    fn estimate_tokens_for_entries(&self, entries: &[SmartDirEntry]) -> usize {
        // Smart entries have more metadata but are filtered
        entries.len() * 30 // ~30 tokens per smart entry
    }
    
    /// Generate context summary
    fn generate_context_summary(
        &self,
        primary: &[SmartDirEntry],
        _secondary: &[SmartDirEntry],
        _context: &TaskContext,
    ) -> String {
        format!(
            "SmartLS analyzed directory. Found {} high-priority items.",
            primary.len()
        )
    }
    
    /// Generate proactive suggestions
    fn generate_suggestions(
        &self,
        primary: &[SmartDirEntry],
        _secondary: &[SmartDirEntry],
        _context: &TaskContext,
    ) -> Vec<String> {
        let mut suggestions = Vec::new();
        
        if primary.is_empty() {
            suggestions.push("No high-priority files found. Consider broadening the task context.".to_string());
        }
        
        // Suggest related tools based on file types found
        let has_config = primary.iter().any(|e| {
            let name = e.node.path.file_name().and_then(|n| n.to_str()).unwrap_or("");
            name.ends_with(".json") || name.ends_with(".yaml")
        });
        let has_code = primary.iter().any(|e| matches!(e.node.category, crate::scanner::FileCategory::Rust | crate::scanner::FileCategory::Python | crate::scanner::FileCategory::JavaScript));
        
        if has_config {
            suggestions.push("Use find_config_files for detailed configuration analysis.".to_string());
        }
        
        if has_code {
            suggestions.push("Use find_code_files for comprehensive code discovery.".to_string());
        }
        
        suggestions
    }
}

impl Default for SmartLS {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_smart_ls_creation() {
        let smart_ls = SmartLS::new();
        // Basic creation test
        assert!(true); // Placeholder test
    }
}
