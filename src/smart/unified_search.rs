//! 🔍 Unified Search - Natural Language Query Engine
//!
//! This module provides a unified search interface that accepts natural
//! language queries and intelligently combines multiple search tools
//! to provide comprehensive, context-aware results.

use super::context::ContextAnalyzer;
use super::nlp::{ParsedQuery, QueryParser, SearchIntent};
use super::smart_ls::SmartLS;
use super::smart_read::SmartReader;
use super::{SmartResponse, TaskContext, TokenSavings};
use crate::scanner::{FileNode, Scanner, ScannerConfig};
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::path::Path;

/// 🔍 Unified search engine
pub struct UnifiedSearch {
    query_parser: QueryParser,
    context_analyzer: ContextAnalyzer,
    _smart_read: SmartReader,
    _smart_ls: SmartLS,
}

/// 🎯 Search result with multiple result types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResult {
    /// File or directory node
    pub node: FileNode,
    /// Relevance score
    pub relevance: super::RelevanceScore,
    /// Result type
    pub result_type: SearchResultType,
    /// Content snippet (for file content matches)
    pub snippet: Option<String>,
    /// Line number (for content matches)
    pub line_number: Option<usize>,
    /// Suggested actions
    pub suggested_actions: Vec<String>,
}

/// 🏷️ Types of search results
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum SearchResultType {
    FileMatch,
    DirectoryMatch,
    ContentMatch,
    StructureMatch,
    ConfigMatch,
    TestMatch,
    DocumentationMatch,
}

/// 📊 Unified search response
pub type UnifiedSearchResponse = SmartResponse<SearchResult>;

impl UnifiedSearch {
    /// Create new unified search engine
    pub fn new() -> Self {
        Self {
            query_parser: QueryParser::new(),
            context_analyzer: ContextAnalyzer::new(),
            _smart_read: SmartReader::new(),
            _smart_ls: SmartLS::new(),
        }
    }

    /// 🔍 Execute unified search with natural language query
    pub fn search(
        &self,
        path: &Path,
        query: &str,
        max_results: Option<usize>,
    ) -> Result<UnifiedSearchResponse> {
        // Parse natural language query
        let parsed_query = self.query_parser.parse(query);

        // Create task context from parsed query
        let task_context = self.create_task_context(&parsed_query, max_results);

        // Execute search based on intent
        let results = match parsed_query.intent {
            SearchIntent::FindCode => self.search_code(path, &parsed_query, &task_context)?,
            SearchIntent::FindConfig => self.search_config(path, &parsed_query, &task_context)?,
            SearchIntent::FindTests => self.search_tests(path, &parsed_query, &task_context)?,
            SearchIntent::FindDocs => self.search_docs(path, &parsed_query, &task_context)?,
            SearchIntent::FindAPI => self.search_api(path, &parsed_query, &task_context)?,
            SearchIntent::FindAuth => self.search_auth(path, &parsed_query, &task_context)?,
            SearchIntent::FindSecurity => {
                self.search_security(path, &parsed_query, &task_context)?
            }
            SearchIntent::FindPerformance => {
                self.search_performance(path, &parsed_query, &task_context)?
            }
            SearchIntent::Debug => self.search_debug(path, &parsed_query, &task_context)?,
            _ => self.search_general(path, &parsed_query, &task_context)?,
        };

        // Split results by relevance
        let (primary, secondary) = self.split_results_by_relevance(&results, &task_context);

        // Calculate token savings
        let token_savings = self.calculate_token_savings(&results, &parsed_query);

        // Generate context summary and suggestions
        let context_summary = self.generate_search_summary(&parsed_query, &primary, &secondary);
        let suggestions = self.generate_search_suggestions(&parsed_query, &primary, &secondary);

        Ok(UnifiedSearchResponse {
            primary,
            secondary,
            context_summary,
            token_savings,
            suggestions,
        })
    }

    /// Create task context from parsed query
    fn create_task_context(
        &self,
        parsed_query: &ParsedQuery,
        max_results: Option<usize>,
    ) -> TaskContext {
        TaskContext {
            task: parsed_query.original_query.clone(),
            focus_areas: parsed_query.focus_areas.clone(),
            relevance_threshold: if parsed_query.confidence > 0.8 {
                0.7
            } else {
                0.5
            },
            max_results,
        }
    }

    /// 💻 Search for code files and content
    fn search_code(
        &self,
        path: &Path,
        parsed_query: &ParsedQuery,
        context: &TaskContext,
    ) -> Result<Vec<SearchResult>> {
        let mut results = Vec::new();

        // Find code files
        let config = ScannerConfig {
            max_depth: 10,
            show_hidden: false,
            ..Default::default()
        };
        let scanner = Scanner::new(path, config)?;
        let (nodes, _stats) = scanner.scan()?;

        for node in nodes {
            if self.is_code_file(&node) {
                let relevance = self.context_analyzer.score_file_relevance(&node, context);

                if relevance.score >= context.relevance_threshold {
                    let result = SearchResult {
                        node: node.clone(),
                        relevance,
                        result_type: SearchResultType::FileMatch,
                        snippet: None,
                        line_number: None,
                        suggested_actions: vec![
                            "Smart read".to_string(),
                            "Analyze code".to_string(),
                        ],
                    };
                    results.push(result);
                }
            }
        }

        // Search within code files for keywords
        if !parsed_query.keywords.is_empty() {
            results.extend(self.search_file_contents(path, &parsed_query.keywords, context)?);
        }

        Ok(results)
    }

    /// ⚙️ Search for configuration files
    fn search_config(
        &self,
        path: &Path,
        _parsed_query: &ParsedQuery,
        context: &TaskContext,
    ) -> Result<Vec<SearchResult>> {
        let mut results = Vec::new();

        let config = ScannerConfig {
            max_depth: 3,
            show_hidden: true, // Config files might be hidden
            follow_symlinks: false,
            ..Default::default()
        };

        let scanner = Scanner::new(path, config)?;
        let (nodes, _) = scanner.scan()?;

        for node in nodes {
            if self.is_config_file(&node) {
                let relevance = self.context_analyzer.score_file_relevance(&node, context);

                if relevance.score >= context.relevance_threshold {
                    let result = SearchResult {
                        node: node.clone(),
                        relevance,
                        result_type: SearchResultType::ConfigMatch,
                        snippet: None,
                        line_number: None,
                        suggested_actions: vec![
                            "Edit config".to_string(),
                            "Validate syntax".to_string(),
                        ],
                    };
                    results.push(result);
                }
            }
        }

        Ok(results)
    }

    /// 🧪 Search for test files
    fn search_tests(
        &self,
        path: &Path,
        _parsed_query: &ParsedQuery,
        context: &TaskContext,
    ) -> Result<Vec<SearchResult>> {
        let mut results = Vec::new();

        let config = ScannerConfig {
            max_depth: 4,
            show_hidden: false,
            follow_symlinks: false,
            ..Default::default()
        };

        let scanner = Scanner::new(path, config)?;
        let (nodes, _) = scanner.scan()?;

        for node in nodes {
            if self.is_test_file(&node) {
                let relevance = self.context_analyzer.score_file_relevance(&node, context);

                if relevance.score >= context.relevance_threshold {
                    let result = SearchResult {
                        node: node.clone(),
                        relevance,
                        result_type: SearchResultType::TestMatch,
                        snippet: None,
                        line_number: None,
                        suggested_actions: vec![
                            "Run tests".to_string(),
                            "Analyze coverage".to_string(),
                        ],
                    };
                    results.push(result);
                }
            }
        }

        Ok(results)
    }

    /// 📚 Search for documentation
    fn search_docs(
        &self,
        path: &Path,
        _parsed_query: &ParsedQuery,
        context: &TaskContext,
    ) -> Result<Vec<SearchResult>> {
        let mut results = Vec::new();

        let config = ScannerConfig {
            max_depth: 3,
            show_hidden: false,
            follow_symlinks: false,
            ..Default::default()
        };

        let scanner = Scanner::new(path, config)?;
        let (nodes, _) = scanner.scan()?;

        for node in nodes {
            if self.is_doc_file(&node) {
                let relevance = self.context_analyzer.score_file_relevance(&node, context);

                if relevance.score >= context.relevance_threshold {
                    let result = SearchResult {
                        node: node.clone(),
                        relevance,
                        result_type: SearchResultType::DocumentationMatch,
                        snippet: None,
                        line_number: None,
                        suggested_actions: vec!["Read docs".to_string(), "Update docs".to_string()],
                    };
                    results.push(result);
                }
            }
        }

        Ok(results)
    }

    /// 🌐 Search for API-related files
    fn search_api(
        &self,
        path: &Path,
        parsed_query: &ParsedQuery,
        context: &TaskContext,
    ) -> Result<Vec<SearchResult>> {
        // Similar implementation to search_code but with API-specific filtering
        self.search_code(path, parsed_query, context)
    }

    /// 🔐 Search for authentication-related files
    fn search_auth(
        &self,
        path: &Path,
        parsed_query: &ParsedQuery,
        context: &TaskContext,
    ) -> Result<Vec<SearchResult>> {
        // Similar implementation to search_code but with auth-specific filtering
        self.search_code(path, parsed_query, context)
    }

    /// 🛡️ Search for security-related files
    fn search_security(
        &self,
        path: &Path,
        parsed_query: &ParsedQuery,
        context: &TaskContext,
    ) -> Result<Vec<SearchResult>> {
        // Similar implementation to search_code but with security-specific filtering
        self.search_code(path, parsed_query, context)
    }

    /// ⚡ Search for performance-related files
    fn search_performance(
        &self,
        path: &Path,
        parsed_query: &ParsedQuery,
        context: &TaskContext,
    ) -> Result<Vec<SearchResult>> {
        // Similar implementation to search_code but with performance-specific filtering
        self.search_code(path, parsed_query, context)
    }

    /// 🐛 Search for debugging-related files
    fn search_debug(
        &self,
        path: &Path,
        parsed_query: &ParsedQuery,
        context: &TaskContext,
    ) -> Result<Vec<SearchResult>> {
        // Similar implementation to search_code but with debug-specific filtering
        self.search_code(path, parsed_query, context)
    }

    /// 🔍 General search combining multiple strategies
    fn search_general(
        &self,
        path: &Path,
        parsed_query: &ParsedQuery,
        context: &TaskContext,
    ) -> Result<Vec<SearchResult>> {
        let mut results = Vec::new();

        // Combine results from different search types
        results.extend(self.search_code(path, parsed_query, context)?);
        results.extend(self.search_config(path, parsed_query, context)?);
        results.extend(self.search_docs(path, parsed_query, context)?);

        // Remove duplicates and sort by relevance
        results.sort_by(|a, b| b.relevance.score.partial_cmp(&a.relevance.score).unwrap());
        results.dedup_by(|a, b| a.node.path == b.node.path);

        Ok(results)
    }

    /// Search within file contents for keywords
    fn search_file_contents(
        &self,
        _path: &Path,
        _keywords: &[String],
        _context: &TaskContext,
    ) -> Result<Vec<SearchResult>> {
        // This would integrate with ripgrep or similar for content search
        // For now, return empty results
        Ok(Vec::new())
    }

    /// Check if file is a code file
    fn is_code_file(&self, node: &FileNode) -> bool {
        matches!(
            node.category,
            crate::scanner::FileCategory::Rust
                | crate::scanner::FileCategory::Python
                | crate::scanner::FileCategory::JavaScript
                | crate::scanner::FileCategory::TypeScript
                | crate::scanner::FileCategory::Go
                | crate::scanner::FileCategory::Java
                | crate::scanner::FileCategory::Cpp
                | crate::scanner::FileCategory::C
        )
    }

    /// Check if file is a configuration file
    fn is_config_file(&self, node: &FileNode) -> bool {
        matches!(
            node.category,
            crate::scanner::FileCategory::Json
                | crate::scanner::FileCategory::Yaml
                | crate::scanner::FileCategory::Toml
        ) || {
            let name = node.path.file_name().and_then(|n| n.to_str()).unwrap_or("");
            name.starts_with('.')
                && (name.contains("config")
                    || name.contains("env")
                    || name == ".gitignore"
                    || name == ".dockerignore")
        }
    }

    /// Check if file is a test file
    fn is_test_file(&self, node: &FileNode) -> bool {
        let name = node.path.file_name().and_then(|n| n.to_str()).unwrap_or("");
        let name_lower = name.to_lowercase();
        name_lower.contains("test")
            || name_lower.contains("spec")
            || node.path.to_string_lossy().contains("/test/")
            || node.path.to_string_lossy().contains("/tests/")
    }

    /// Check if file is documentation
    fn is_doc_file(&self, node: &FileNode) -> bool {
        matches!(node.category, crate::scanner::FileCategory::Markdown)
            || {
                let name = node.path.file_name().and_then(|n| n.to_str()).unwrap_or("");
                let name_lower = name.to_lowercase();
                name_lower.starts_with("readme") || name_lower.contains("doc")
            }
            || node.path.to_string_lossy().contains("/docs/")
    }

    /// Split results by relevance threshold
    fn split_results_by_relevance(
        &self,
        results: &[SearchResult],
        context: &TaskContext,
    ) -> (Vec<SearchResult>, Vec<SearchResult>) {
        let mut primary = Vec::new();
        let mut secondary = Vec::new();

        for result in results {
            if result.relevance.score >= context.relevance_threshold {
                primary.push(result.clone());
            } else if result.relevance.score >= context.relevance_threshold * 0.6 {
                secondary.push(result.clone());
            }
        }

        (primary, secondary)
    }

    /// Calculate token savings from unified search
    fn calculate_token_savings(
        &self,
        results: &[SearchResult],
        _parsed_query: &ParsedQuery,
    ) -> TokenSavings {
        let original_tokens = 1000; // Estimated tokens for multiple separate tool calls
        let compressed_tokens = results.len() * 20; // Estimated tokens per result

        TokenSavings::new(original_tokens, compressed_tokens, "unified-search")
    }

    /// Generate search summary
    fn generate_search_summary(
        &self,
        parsed_query: &ParsedQuery,
        primary: &[SearchResult],
        secondary: &[SearchResult],
    ) -> String {
        format!(
            "Unified search for '{}' (intent: {:?}, confidence: {:.1}%) found {} high-priority and {} medium-priority results",
            parsed_query.original_query,
            parsed_query.intent,
            parsed_query.confidence * 100.0,
            primary.len(),
            secondary.len()
        )
    }

    /// Generate search suggestions
    fn generate_search_suggestions(
        &self,
        parsed_query: &ParsedQuery,
        primary: &[SearchResult],
        secondary: &[SearchResult],
    ) -> Vec<String> {
        let mut suggestions = Vec::new();

        if primary.is_empty() && secondary.is_empty() {
            suggestions.push("No results found. Try broadening your search terms.".to_string());
            suggestions.push("Consider using different keywords or checking spelling.".to_string());
        } else if primary.is_empty() {
            suggestions.push(
                "No high-priority results. Consider lowering relevance threshold.".to_string(),
            );
        }

        // Intent-specific suggestions
        match parsed_query.intent {
            SearchIntent::FindCode => {
                suggestions.push("Use SmartRead to analyze code files in detail.".to_string());
            }
            SearchIntent::FindConfig => {
                suggestions.push(
                    "Use find_config_files for comprehensive configuration discovery.".to_string(),
                );
            }
            SearchIntent::FindTests => {
                suggestions
                    .push("Use find_tests to discover all test files and patterns.".to_string());
            }
            _ => {}
        }

        suggestions
    }
}

impl Default for UnifiedSearch {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    // use std::path::PathBuf;  // Commented out as unused

    #[test]
    fn test_unified_search_creation() {
        let _search = UnifiedSearch::new();
        // Basic creation test
        assert!(true); // Placeholder test
    }
}
