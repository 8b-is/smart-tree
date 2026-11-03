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
    /// Generated token mappings (`pattern -> hex_id`)
    tokens: HashMap<String, String>,
    /// Reverse token mappings (`hex_id -> pattern`) for lossless decompression
    reverse_tokens: HashMap<String, String>,
    /// Pre-sorted `(pattern, hex_id)` ordered by descending pattern length for O(N) single-pass compression
    sorted_patterns: Vec<(String, String)>,
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
            reverse_tokens: HashMap::new(),
            sorted_patterns: Vec::new(),
        }
    }

    /// Analyze nodes to learn patterns
    pub fn analyze(&mut self, nodes: &[FileNode]) {
        for node in nodes {
            // Analyze path components (supporting both Unix '/' and Windows '\\')
            let path_str = node.path.to_string_lossy();

            for component in path_str.split(['/', '\\']).filter(|c| !c.is_empty()) {
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

    /// Generate optimal token assignments without duplicate overwrites or `0xF0..=0xFF` collisions.
    fn generate_tokens(&mut self) {
        self.tokens.clear();
        self.reverse_tokens.clear();
        self.sorted_patterns.clear();

        // Deduplicate patterns across path_components, file_names, and extensions
        // keeping the maximum observed frequency for each unique pattern.
        let mut deduplicated: HashMap<String, usize> = HashMap::new();

        for (pattern, &count) in &self.path_components {
            if count > 2 {
                let entry = deduplicated.entry(pattern.clone()).or_insert(0);
                *entry = (*entry).max(count);
            }
        }

        for (pattern, &count) in &self.file_names {
            if count > 2 {
                let entry = deduplicated.entry(pattern.clone()).or_insert(0);
                *entry = (*entry).max(count);
            }
        }

        for (pattern, &count) in &self.extensions {
            if count > 5 {
                let dot_ext = format!(".{}", pattern);
                let entry = deduplicated.entry(dot_ext).or_insert(0);
                *entry = (*entry).max(count);
            }
        }

        let mut all_patterns: Vec<(String, usize)> = deduplicated.into_iter().collect();

        // Sort by frequency (descending), then pattern length (descending), then lexicographically for determinism
        all_patterns.sort_by(|a, b| {
            b.1.cmp(&a.1)
                .then_with(|| b.0.len().cmp(&a.0.len()))
                .then_with(|| a.0.cmp(&b.0))
        });

        // Assign tokens in 0x80..=0xEF (reserving 0xF0..=0xFF for semantic AST tokens F0..F8 and escape markers)
        for (token_id, (pattern, _count)) in (0x80u8..=0xEF).zip(all_patterns) {
            let hex_id = format!("{:02X}", token_id);
            self.tokens.insert(pattern.clone(), hex_id.clone());
            self.reverse_tokens.insert(hex_id.clone(), pattern.clone());
            self.sorted_patterns.push((pattern, hex_id));
        }

        // Pre-sort `sorted_patterns` by descending pattern length so `compress_path`
        // performs a single left-to-right greedy pass without re-sorting or corrupting `{XX}` markers.
        self.sorted_patterns
            .sort_by(|a, b| b.0.len().cmp(&a.0.len()).then_with(|| a.1.cmp(&b.1)));
    }

    /// Compress a path using learned tokens in a single left-to-right non-overlapping scan.
    ///
    /// Unlike repeated `String::replace`, this never re-scans emitted `{XX}` markers
    /// (preventing corruption when a learned pattern contains hex characters) and
    /// avoids $O(K)$ intermediate `String` allocations per path.
    pub fn compress_path(&self, path: &str) -> String {
        if self.sorted_patterns.is_empty() || path.is_empty() {
            return path.to_string();
        }

        let mut out = String::with_capacity(path.len());
        let mut remaining = path;

        while !remaining.is_empty() {
            let mut matched = false;
            for (pattern, token) in &self.sorted_patterns {
                if remaining.starts_with(pattern.as_str()) {
                    out.push('{');
                    out.push_str(token);
                    out.push('}');
                    remaining = &remaining[pattern.len()..];
                    matched = true;
                    break;
                }
            }

            if !matched {
                if let Some(ch) = remaining.chars().next() {
                    out.push(ch);
                    remaining = &remaining[ch.len_utf8()..];
                }
            }
        }

        out
    }

    /// Decompress a `{XX}` tokenized path back into its original string.
    pub fn decompress_path(&self, compressed: &str) -> String {
        let mut out = String::with_capacity(compressed.len() * 2);
        let mut remaining = compressed;

        while !remaining.is_empty() {
            if remaining.starts_with('{') && remaining.len() >= 4 && remaining.as_bytes()[3] == b'}'
            {
                if let Some(hex_id) = remaining.get(1..3) {
                    if let Some(pattern) = self.reverse_tokens.get(hex_id) {
                        out.push_str(pattern);
                        remaining = &remaining[4..];
                        continue;
                    }
                }
            }
            if let Some(ch) = remaining.chars().next() {
                out.push(ch);
                remaining = &remaining[ch.len_utf8()..];
            }
        }

        out
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
        let total_token_bytes = self.tokens.len() * 4; // "{XX}" is 4 bytes

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

/// Split CamelCase into components without per-character String allocations.
fn split_camel_case(s: &str) -> Vec<String> {
    let mut result = Vec::new();
    let mut current = String::with_capacity(s.len());

    for (i, ch) in s.chars().enumerate() {
        if i > 0 && ch.is_uppercase() && !current.is_empty() {
            result.push(std::mem::take(&mut current));
        }
        for lower in ch.to_lowercase() {
            current.push(lower);
        }
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
    fn test_compression_and_roundtrip_without_marker_corruption() {
        let mut tokenizer = DynamicTokenizer::new();

        // Add patterns including hex-like substrings ("80", "bad") that previously corrupted "{80}" markers
        for _ in 0..10 {
            *tokenizer
                .path_components
                .entry("src".to_string())
                .or_insert(0) += 1;
            *tokenizer
                .path_components
                .entry("components".to_string())
                .or_insert(0) += 1;
            *tokenizer
                .path_components
                .entry("80".to_string())
                .or_insert(0) += 1;
        }

        tokenizer.generate_tokens();

        // Test compression and lossless decompression
        let original = "src/components/80/Button.tsx";
        let compressed = tokenizer.compress_path(original);

        assert!(compressed.len() < original.len());
        assert!(compressed.contains('{'));
        assert!(compressed.contains('}'));
        assert_eq!(tokenizer.decompress_path(&compressed), original);
    }

    #[test]
    fn test_deduplication_and_token_assignment_order() {
        let mut tokenizer = DynamicTokenizer::new();

        // Same pattern in both path_components and file_names must not waste a token ID
        *tokenizer
            .path_components
            .entry("README.md".to_string())
            .or_insert(0) = 100;
        *tokenizer
            .file_names
            .entry("README.md".to_string())
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

        // README.md must retain 80 (not overwritten by 81!) and less_frequent gets 81
        assert_eq!(tokenizer.tokens.get("README.md").unwrap(), "80");
        assert_eq!(tokenizer.tokens.get("less_frequent").unwrap(), "81");
        assert!(!tokenizer.tokens.contains_key("too_rare"));
    }
}
