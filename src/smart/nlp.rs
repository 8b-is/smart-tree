//! 🧠 Natural Language Processing for Smart Tools
//!
//! This module provides natural language understanding capabilities
//! for parsing user queries and converting them into structured
//! search intents and parameters.

use super::FocusArea;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// 🧠 Natural language query parser
pub struct QueryParser {
    /// Intent detection patterns
    intent_patterns: HashMap<SearchIntent, Vec<String>>,
    /// Entity extraction patterns
    entity_patterns: HashMap<String, Vec<String>>,
}

/// 🎯 Search intent types
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum SearchIntent {
    FindCode,
    FindConfig,
    FindTests,
    FindDocs,
    FindBugs,
    FindSecurity,
    FindPerformance,
    FindAPI,
    FindAuth,
    FindDatabase,
    Debug,
    Optimize,
    Deploy,
    General,
}

/// 📝 Parsed query structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParsedQuery {
    /// Primary intent
    pub intent: SearchIntent,
    /// Extracted entities (file names, function names, etc.)
    pub entities: Vec<String>,
    /// Programming languages mentioned
    pub languages: Option<Vec<String>>,
    /// Keywords for content search
    pub keywords: Vec<String>,
    /// Focus areas derived from query
    pub focus_areas: Vec<FocusArea>,
    /// Original query text
    pub original_query: String,
    /// Confidence score (0.0-1.0)
    pub confidence: f32,
}

impl QueryParser {
    /// Create new query parser
    pub fn new() -> Self {
        let mut parser = Self {
            intent_patterns: HashMap::new(),
            entity_patterns: HashMap::new(),
        };

        parser.initialize_patterns();
        parser
    }

    /// 🔍 Parse natural language query
    pub fn parse(&self, query: &str) -> ParsedQuery {
        let query_lower = query.to_lowercase();

        // Detect intent
        let intent = self.detect_intent(&query_lower);

        // Extract entities
        let entities = self.extract_entities(&query_lower);

        // Extract languages
        let languages = self.extract_languages(&query_lower);

        // Extract keywords
        let keywords = self.extract_keywords(&query_lower);

        // Derive focus areas
        let focus_areas = self.derive_focus_areas(&intent, &keywords);

        // Calculate confidence
        let confidence = self.calculate_confidence(&intent, &entities, &keywords);

        ParsedQuery {
            intent,
            entities,
            languages,
            keywords,
            focus_areas,
            original_query: query.to_string(),
            confidence,
        }
    }

    /// 🎯 Detect search intent from query
    fn detect_intent(&self, query: &str) -> SearchIntent {
        for (intent, patterns) in &self.intent_patterns {
            for pattern in patterns {
                if query.contains(pattern) {
                    return intent.clone();
                }
            }
        }

        SearchIntent::General
    }

    /// 🏷️ Extract entities (file names, function names, etc.)
    fn extract_entities(&self, query: &str) -> Vec<String> {
        let mut entities = Vec::new();

        // Simple entity extraction - look for quoted strings
        let mut in_quotes = false;
        let mut current_entity = String::new();

        for ch in query.chars() {
            match ch {
                '"' | '\'' => {
                    if in_quotes && !current_entity.is_empty() {
                        entities.push(current_entity.clone());
                        current_entity.clear();
                    }
                    in_quotes = !in_quotes;
                }
                _ if in_quotes => {
                    current_entity.push(ch);
                }
                _ => {}
            }
        }

        // Also extract common file extensions and patterns
        for (entity_type, patterns) in &self.entity_patterns {
            for pattern in patterns {
                if query.contains(pattern) {
                    entities.push(format!("{}:{}", entity_type, pattern));
                }
            }
        }

        entities
    }

    /// 💻 Extract programming languages mentioned
    fn extract_languages(&self, query: &str) -> Option<Vec<String>> {
        let languages = vec![
            "rust",
            "python",
            "javascript",
            "typescript",
            "go",
            "java",
            "cpp",
            "c++",
            "c",
            "ruby",
            "php",
            "swift",
            "kotlin",
            "scala",
        ];

        let mut found_languages = Vec::new();

        for lang in languages {
            if query.contains(lang) {
                found_languages.push(lang.to_string());
            }
        }

        if found_languages.is_empty() {
            None
        } else {
            Some(found_languages)
        }
    }

    /// 🔑 Extract keywords for content search
    fn extract_keywords(&self, query: &str) -> Vec<String> {
        // Simple keyword extraction - split on common words and take meaningful terms
        let stop_words = vec![
            "find", "search", "look", "for", "in", "the", "a", "an", "and", "or", "with", "that",
            "have", "has", "is", "are", "was", "were", "be", "been", "being", "do", "does", "did",
            "will", "would", "could", "should", "may", "might", "can", "all", "any", "some",
            "this", "that", "these", "those", "i", "you", "he", "she", "it", "we", "they",
        ];

        query
            .split_whitespace()
            .map(|word| word.trim_matches(|c: char| !c.is_alphanumeric()))
            .filter(|word| !word.is_empty() && word.len() > 2)
            .filter(|word| !stop_words.contains(&word.to_lowercase().as_str()))
            .map(|word| word.to_string())
            .collect()
    }

    /// 🎯 Derive focus areas from intent and keywords
    fn derive_focus_areas(&self, intent: &SearchIntent, keywords: &[String]) -> Vec<FocusArea> {
        let mut focus_areas = Vec::new();

        // Add focus area based on intent
        match intent {
            SearchIntent::FindAuth => focus_areas.push(FocusArea::Authentication),
            SearchIntent::FindAPI => focus_areas.push(FocusArea::API),
            SearchIntent::FindDatabase => focus_areas.push(FocusArea::Database),
            SearchIntent::FindTests => focus_areas.push(FocusArea::Testing),
            SearchIntent::FindConfig => focus_areas.push(FocusArea::Configuration),
            SearchIntent::FindSecurity => focus_areas.push(FocusArea::Security),
            SearchIntent::FindPerformance => focus_areas.push(FocusArea::Performance),
            SearchIntent::FindDocs => focus_areas.push(FocusArea::Documentation),
            _ => {}
        }

        // Add focus areas based on keywords
        for keyword in keywords {
            let keyword_lower = keyword.to_lowercase();
            let focus_area = FocusArea::from_str(&keyword_lower);
            if !focus_areas.contains(&focus_area) {
                focus_areas.push(focus_area);
            }
        }

        // Default focus areas if none found
        if focus_areas.is_empty() {
            focus_areas = vec![FocusArea::API, FocusArea::Configuration];
        }

        focus_areas
    }

    /// 📊 Calculate confidence score
    fn calculate_confidence(
        &self,
        intent: &SearchIntent,
        entities: &[String],
        keywords: &[String],
    ) -> f32 {
        let mut confidence: f32 = 0.5; // Base confidence

        // Boost confidence for specific intents
        if *intent != SearchIntent::General {
            confidence += 0.2;
        }

        // Boost confidence for entities
        if !entities.is_empty() {
            confidence += 0.2;
        }

        // Boost confidence for meaningful keywords
        if keywords.len() >= 2 {
            confidence += 0.1;
        }

        confidence.min(1.0)
    }

    /// Initialize intent detection patterns
    fn initialize_patterns(&mut self) {
        // Find code patterns
        self.intent_patterns.insert(
            SearchIntent::FindCode,
            vec![
                "find code".to_string(),
                "show code".to_string(),
                "source code".to_string(),
                "functions".to_string(),
                "methods".to_string(),
                "classes".to_string(),
            ],
        );

        // Find config patterns
        self.intent_patterns.insert(
            SearchIntent::FindConfig,
            vec![
                "config".to_string(),
                "configuration".to_string(),
                "settings".to_string(),
                "env".to_string(),
                "environment".to_string(),
            ],
        );

        // Find tests patterns
        self.intent_patterns.insert(
            SearchIntent::FindTests,
            vec![
                "test".to_string(),
                "tests".to_string(),
                "testing".to_string(),
                "spec".to_string(),
                "specs".to_string(),
            ],
        );

        // Authentication patterns
        self.intent_patterns.insert(
            SearchIntent::FindAuth,
            vec![
                "auth".to_string(),
                "authentication".to_string(),
                "login".to_string(),
                "signin".to_string(),
                "password".to_string(),
                "token".to_string(),
            ],
        );

        // API patterns
        self.intent_patterns.insert(
            SearchIntent::FindAPI,
            vec![
                "api".to_string(),
                "endpoint".to_string(),
                "route".to_string(),
                "handler".to_string(),
                "controller".to_string(),
            ],
        );

        // Security patterns
        self.intent_patterns.insert(
            SearchIntent::FindSecurity,
            vec![
                "security".to_string(),
                "vulnerability".to_string(),
                "secure".to_string(),
                "encrypt".to_string(),
                "sanitize".to_string(),
            ],
        );

        // Performance patterns
        self.intent_patterns.insert(
            SearchIntent::FindPerformance,
            vec![
                "performance".to_string(),
                "optimize".to_string(),
                "slow".to_string(),
                "fast".to_string(),
                "cache".to_string(),
            ],
        );

        // Debug patterns
        self.intent_patterns.insert(
            SearchIntent::Debug,
            vec![
                "debug".to_string(),
                "debugging".to_string(),
                "bug".to_string(),
                "error".to_string(),
                "issue".to_string(),
                "problem".to_string(),
            ],
        );

        // Entity patterns
        self.entity_patterns.insert(
            "file_extension".to_string(),
            vec![
                ".rs", ".py", ".js", ".ts", ".go", ".java", ".json", ".yaml", ".toml",
            ]
            .into_iter()
            .map(String::from)
            .collect(),
        );
    }
}

impl Default for QueryParser {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_query_parsing() {
        let parser = QueryParser::new();
        let query = "find authentication code in rust files";
        let parsed = parser.parse(query);

        assert_eq!(parsed.intent, SearchIntent::FindAuth);
        assert!(parsed
            .languages
            .as_ref()
            .unwrap()
            .contains(&"rust".to_string()));
        assert!(parsed.focus_areas.contains(&FocusArea::Authentication));
    }

    #[test]
    fn test_entity_extraction() {
        let parser = QueryParser::new();
        let query = "find 'login_handler' function";
        let parsed = parser.parse(query);

        assert!(parsed.entities.contains(&"login_handler".to_string()));
    }
}
