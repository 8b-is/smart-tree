//! Dynamic tokenizer - "Learning each project's language!" - Omni
//! Automatically discovers and tokenizes common patterns in any codebase

use crate::scanner::FileNode;
use std::collections::HashMap;

/// Dynamic tokenizer that learns project-specific patterns
pub struct DynamicTokenizer {
    /// Path component frequencies
    path_components: HashMap<String, usize>,
    /// File name frequencies
    file_names: HashMap<String, usize>,
    /// Extension frequencies
    extensions: HashMap<String, usize>,
    /// Common prefixes/suffixes
    prefixes: HashMap<String, usize>,
    suffixes: HashMap<String, usize>,
    /// Generated token mappings
    tokens: HashMap<String, String>,
}

impl Default for DynamicTokenizer {
    fn default() -> Self {
        Self::new()
    }
}

impl DynamicTokenizer {
    pub fn new() -> Self {
        Self {
            path_components: HashMap::new(),
            file_names: HashMap::new(),
            extensions: HashMap::new(),
            prefixes: HashMap::new(),
            suffixes: HashMap::new(),
            tokens: HashMap::new(),
        }
    }

    /// Analyze nodes to learn patterns
    pub fn analyze(&mut self, nodes: &[FileNode]) {
        for node in nodes {
            // Analyze path components
            let path_str = node.path.to_string_lossy();

            // Split path into components
            for component in path_str.split('/').filter(|c| !c.is_empty()) {
                *self
                    .path_components
                    .entry(component.to_string())
                    .or_insert(0) += 1;
            }

            // Analyze file name
            if let Some(file_name) = node.path.file_name() {
                let name = file_name.to_string_lossy().to_string();
                *self.file_names.entry(name.clone()).or_insert(0) += 1;

                // Extract common patterns
                self.analyze_name_patterns(&name);
            }

            // Analyze extension
            if let Some(ext) = node.path.extension() {
                let ext_str = ext.to_string_lossy().to_string();
                *self.extensions.entry(ext_str).or_insert(0) += 1;
            }
        }

        // Generate optimal tokens
        self.generate_tokens();
    }

    /// Analyze file name for common patterns
    fn analyze_name_patterns(&mut self, name: &str) {
        // Common prefixes
        let prefix_patterns = ["test_", "Test", "_", "mock_", "stub_", "fake_"];
        for prefix in &prefix_patterns {
            if name.starts_with(prefix) {
                *self.prefixes.entry(prefix.to_string()).or_insert(0) += 1;
            }
        }

        // Common suffixes
        let suffix_patterns = [
            "_test",
            "Test",
            "Spec",
            "_spec",
            ".test",
            ".spec",
            "Controller",
            "Service",
            "Repository",
            "Model",
            "View",
            "Component",
            "Module",
            "Config",
        ];
        for suffix in &suffix_patterns {
            if name.contains(suffix) {
                *self.suffixes.entry(suffix.to_string()).or_insert(0) += 1;
            }
        }

        // Camel/Snake case components
        if name.contains('_') {
            // Snake case - split and analyze
            for part in name.split('_') {
                if part.len() > 2 {
                    *self.path_components.entry(part.to_string()).or_insert(0) += 1;
                }
            }
        } else if name.chars().any(|c| c.is_uppercase()) && name.chars().any(|c| c.is_lowercase()) {
            // CamelCase - split and analyze
            let parts = split_camel_case(name);
            for part in parts {
                if part.len() > 2 {
                    *self.path_components.entry(part).or_insert(0) += 1;
                }
            }
        }
    }

    /// Generate optimal token assignments
    fn generate_tokens(&mut self) {
        let mut token_id = 0x80; // Start from 128

        // Sort all patterns by frequency
        let mut all_patterns: Vec<(String, usize)> = Vec::new();

        // Collect all patterns with their frequencies
        for (pattern, count) in &self.path_components {
            if *count > 2 {
                // Only tokenize if it appears more than twice
                all_patterns.push((pattern.clone(), *count));
            }
        }

        for (pattern, count) in &self.file_names {
            if *count > 2 {
                all_patterns.push((pattern.clone(), *count));
            }
        }

        for (pattern, count) in &self.extensions {
            if *count > 5 {
                // Extensions need higher frequency
                all_patterns.push((format!(".{}", pattern), *count));
            }
        }

        // Sort by frequency (descending) and pattern length (descending for same frequency)
        all_patterns.sort_by(|a, b| b.1.cmp(&a.1).then_with(|| b.0.len().cmp(&a.0.len())));

        // Assign tokens to most frequent patterns
        for (pattern, _count) in all_patterns.iter().take(127) {
            // Max 127 tokens
            self.tokens
                .insert(pattern.clone(), format!("{:02X}", token_id));
            token_id += 1;
            if token_id > 0xFE {
                // Reserve 0xFF
                break;
            }
        }
    }

    /// Compress a path using learned tokens
    pub fn compress_path(&self, path: &str) -> String {
        let mut compressed = path.to_string();

        // Apply tokens from longest to shortest to avoid substring issues
        let mut tokens_by_length: Vec<(&String, &String)> = self.tokens.iter().collect();
        tokens_by_length.sort_by(|a, b| b.0.len().cmp(&a.0.len()));

        for (pattern, token) in tokens_by_length {
            compressed = compressed.replace(pattern, &format!("{{{}}}", token));
        }

        compressed
    }

    /// Get the token dictionary header
    pub fn get_token_header(&self) -> String {
        let mut header = String::from("TOKENS:\n");

        // Sort tokens by ID for consistent output
        let mut sorted_tokens: Vec<(&String, &String)> = self.tokens.iter().collect();
        sorted_tokens.sort_by(|a, b| a.1.cmp(b.1));

        for (pattern, token) in sorted_tokens {
            header.push_str(&format!("  {}={}\n", token, pattern));
        }

        header
    }

    /// Get compression statistics
    pub fn get_stats(&self) -> TokenizerStats {
        let total_pattern_bytes: usize = self.tokens.keys().map(|k| k.len()).sum();
        let total_token_bytes = self.tokens.len() * 3; // {XX} format

        TokenizerStats {
            patterns_found: self.path_components.len()
                + self.file_names.len()
                + self.extensions.len(),
            tokens_generated: self.tokens.len(),
            estimated_savings: total_pattern_bytes.saturating_sub(total_token_bytes),
        }
    }
}

#[derive(Debug)]
pub struct TokenizerStats {
    pub patterns_found: usize,
    pub tokens_generated: usize,
    pub estimated_savings: usize,
}

/// Split CamelCase into components
fn split_camel_case(s: &str) -> Vec<String> {
    let mut result = Vec::new();
    let mut current = String::new();

    for (i, ch) in s.chars().enumerate() {
        if i > 0 && ch.is_uppercase() && !current.is_empty() {
            result.push(current.clone());
            current.clear();
        }
        current.push(ch.to_lowercase().to_string().chars().next().unwrap());
    }

    if !current.is_empty() {
        result.push(current);
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_camel_case_split() {
        assert_eq!(
            split_camel_case("UserController"),
            vec!["user", "controller"]
        );
        assert_eq!(
            split_camel_case("HTTPSConnection"),
            vec!["h", "t", "t", "p", "s", "connection"]
        );
    }

    #[test]
    fn test_pattern_detection() {
        let mut tokenizer = DynamicTokenizer::new();

        // Simulate a typical web project
        let patterns = vec![
            "src/components/UserList.tsx",
            "src/components/UserDetail.tsx",
            "src/components/UserForm.tsx",
            "src/services/UserService.ts",
            "src/services/AuthService.ts",
            "src/services/ApiService.ts", // Added to make services appear 3 times
            "src/controllers/UserController.ts",
            "src/controllers/AuthController.ts",
            "src/controllers/ApiController.ts", // Added to make controllers appear 3 times
            "tests/unit/UserService.test.ts",
            "tests/unit/AuthService.test.ts",
            "tests/integration/ApiService.test.ts", // Added to make tests appear 3 times
        ];

        for pattern in patterns {
            tokenizer.analyze_name_patterns(pattern);
            for component in pattern.split('/') {
                *tokenizer
                    .path_components
                    .entry(component.to_string())
                    .or_insert(0) += 1;
            }
        }

        tokenizer.generate_tokens();

        // Should tokenize frequent patterns (appear > 2 times)
        assert!(tokenizer.tokens.contains_key("src"));
        assert!(tokenizer.tokens.contains_key("components"));
        assert!(tokenizer.tokens.contains_key("services"));
        assert!(tokenizer.tokens.contains_key("tests"));
        assert!(tokenizer.tokens.contains_key("controllers"));
    }

    #[test]
    fn test_compression() {
        let mut tokenizer = DynamicTokenizer::new();

        // Add patterns
        for _ in 0..10 {
            *tokenizer
                .path_components
                .entry("src".to_string())
                .or_insert(0) += 1;
            *tokenizer
                .path_components
                .entry("components".to_string())
                .or_insert(0) += 1;
        }

        tokenizer.generate_tokens();

        // Test compression
        let original = "src/components/Button.tsx";
        let compressed = tokenizer.compress_path(original);

        // Should be shorter
        assert!(compressed.len() < original.len());
        // Should contain token markers
        assert!(compressed.contains("{"));
        assert!(compressed.contains("}"));
    }

    #[test]
    fn test_token_assignment_order() {
        let mut tokenizer = DynamicTokenizer::new();

        // Add patterns with different frequencies
        *tokenizer
            .path_components
            .entry("very_frequent".to_string())
            .or_insert(0) = 100;
        *tokenizer
            .path_components
            .entry("less_frequent".to_string())
            .or_insert(0) = 50;
        *tokenizer
            .path_components
            .entry("rare".to_string())
            .or_insert(0) = 3;
        *tokenizer
            .path_components
            .entry("too_rare".to_string())
            .or_insert(0) = 1; // Won't be tokenized

        tokenizer.generate_tokens();

        // Most frequent should get lower token IDs
        let very_frequent_token = tokenizer.tokens.get("very_frequent").unwrap();
        let less_frequent_token = tokenizer.tokens.get("less_frequent").unwrap();

        assert!(very_frequent_token < less_frequent_token);

        // Too rare shouldn't be tokenized
        assert!(!tokenizer.tokens.contains_key("too_rare"));
    }
}
