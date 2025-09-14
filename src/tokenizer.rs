// Smart Tree Tokenizer - Turn patterns into bytes! 🗜️
// "Like 6502 opcodes - LDA is $A9, not 'LOAD ACCUMULATOR'" - Hue

use std::collections::HashMap;

/// Common patterns tokenized to single bytes
pub struct Tokenizer {
    /// Pattern → Token mapping
    patterns: HashMap<String, u8>,
    /// Token → Pattern for decoding
    tokens: HashMap<u8, String>,
}

impl Tokenizer {
    pub fn new() -> Self {
        let mut t = Tokenizer {
            patterns: HashMap::new(),
            tokens: HashMap::new(),
        };

        // Directory tokens (0x80-0x8F)
        t.add(0x80, "node_modules");
        t.add(0x81, ".git");
        t.add(0x82, "src");
        t.add(0x83, "target");
        t.add(0x84, "dist");
        t.add(0x85, "build");
        t.add(0x86, "docs");
        t.add(0x87, "tests");
        t.add(0x88, "examples");
        t.add(0x89, ".vscode");
        t.add(0x8A, ".github");

        // File extensions (0x90-0x9F)
        t.add(0x90, ".js");
        t.add(0x91, ".rs");
        t.add(0x92, ".py");
        t.add(0x93, ".ts");
        t.add(0x94, ".json");
        t.add(0x95, ".md");
        t.add(0x96, ".toml");
        t.add(0x97, ".yaml");
        t.add(0x98, ".tsx");
        t.add(0x99, ".jsx");
        t.add(0x9A, ".go");
        t.add(0x9B, ".java");
        t.add(0x9C, ".cpp");
        t.add(0x9D, ".c");
        t.add(0x9E, ".h");

        // Common filenames (0xA0-0xAF)
        t.add(0xA0, "README.md");
        t.add(0xA1, "package.json");
        t.add(0xA2, "Cargo.toml");
        t.add(0xA3, "main.rs");
        t.add(0xA4, "index.js");
        t.add(0xA5, "app.js");
        t.add(0xA6, ".gitignore");
        t.add(0xA7, "LICENSE");
        t.add(0xA8, "Makefile");
        t.add(0xA9, "Dockerfile");
        t.add(0xAA, "tsconfig.json");
        t.add(0xAB, "setup.py");
        t.add(0xAC, "go.mod");

        // Patterns (0xB0-0xBF)
        t.add(0xB0, "test_");
        t.add(0xB1, "_test");
        t.add(0xB2, ".min.");
        t.add(0xB3, ".spec.");
        t.add(0xB4, "TODO");
        t.add(0xB5, "FIXME");
        t.add(0xB6, "function");
        t.add(0xB7, "async");
        t.add(0xB8, "import");
        t.add(0xB9, "export");
        t.add(0xBA, "class");
        t.add(0xBB, "struct");
        t.add(0xBC, "impl");
        t.add(0xBD, "trait");

        // Common paths (0xC0-0xCF)
        t.add(0xC0, "src/");
        t.add(0xC1, "tests/");
        t.add(0xC2, "docs/");
        t.add(0xC3, "../");
        t.add(0xC4, "./");
        t.add(0xC5, "~/");

        t
    }

    fn add(&mut self, token: u8, pattern: &str) {
        self.patterns.insert(pattern.to_string(), token);
        self.tokens.insert(token, pattern.to_string());
    }

    /// Tokenize a string
    pub fn tokenize(&self, text: &str) -> Vec<u8> {
        let mut result = Vec::new();
        let mut remaining = text;

        while !remaining.is_empty() {
            let mut found = false;

            // Try to match longest pattern first
            for len in (1..=remaining.len()).rev() {
                if let Some(chunk) = remaining.get(0..len) {
                    if let Some(&token) = self.patterns.get(chunk) {
                        result.push(token);
                        remaining = &remaining[len..];
                        found = true;
                        break;
                    }
                }
            }

            if !found {
                // No pattern matched, store as raw byte
                result.push(remaining.as_bytes()[0]);
                remaining = &remaining[1..];
            }
        }

        result
    }

    /// Decode tokens back to string
    pub fn decode(&self, tokens: &[u8]) -> String {
        let mut result = String::new();

        for &token in tokens {
            if let Some(pattern) = self.tokens.get(&token) {
                result.push_str(pattern);
            } else if token < 128 {
                // ASCII character
                result.push(token as char);
            } else {
                // Unknown token
                result.push_str(&format!("<{:02X}>", token));
            }
        }

        result
    }

    /// Calculate compression ratio
    pub fn compression_ratio(&self, original: &str) -> f64 {
        let tokenized = self.tokenize(original);
        tokenized.len() as f64 / original.len() as f64
    }
}

/// Quantum tokenizer - even more compression!
pub struct QuantumTokenizer {
    base: Tokenizer,
    /// Multi-pattern combinations
    combos: HashMap<Vec<u8>, u8>,
}

impl QuantumTokenizer {
    pub fn new() -> Self {
        let mut qt = QuantumTokenizer {
            base: Tokenizer::new(),
            combos: HashMap::new(),
        };

        // Common combinations (0xE0-0xEF)
        qt.add_combo(0xE0, &[0x82, 0xC0]); // "src" + "src/" = "src/"
        qt.add_combo(0xE1, &[0x91, 0xA3]); // ".rs" + "main.rs"
        qt.add_combo(0xE2, &[0x90, 0xA4]); // ".js" + "index.js"
        qt.add_combo(0xE3, &[0x80, 0xC4]); // "node_modules" + "./"

        qt
    }

    fn add_combo(&mut self, token: u8, pattern: &[u8]) {
        self.combos.insert(pattern.to_vec(), token);
    }

    pub fn quantum_tokenize(&self, text: &str) -> Vec<u8> {
        let tokens = self.base.tokenize(text);

        // Second pass: combine tokens
        let mut result = Vec::new();
        let mut i = 0;

        while i < tokens.len() {
            let mut found = false;

            // Try to match combo patterns
            for len in (2..=4).rev() {
                if i + len <= tokens.len() {
                    if let Some(&combo_token) = self.combos.get(&tokens[i..i+len]) {
                        result.push(combo_token);
                        i += len;
                        found = true;
                        break;
                    }
                }
            }

            if !found {
                result.push(tokens[i]);
                i += 1;
            }
        }

        result
    }
}

/// Statistics for tokenization
pub struct TokenStats {
    pub original_size: usize,
    pub tokenized_size: usize,
    pub compression_ratio: f64,
    pub patterns_found: usize,
}

impl TokenStats {
    pub fn calculate(original: &str, tokenizer: &Tokenizer) -> Self {
        let tokens = tokenizer.tokenize(original);
        let patterns_found = tokens.iter()
            .filter(|&&t| t >= 0x80)
            .count();

        TokenStats {
            original_size: original.len(),
            tokenized_size: tokens.len(),
            compression_ratio: tokens.len() as f64 / original.len() as f64,
            patterns_found,
        }
    }

    pub fn display(&self) -> String {
        format!(
            "Tokenization: {} → {} bytes ({:.1}% ratio), {} patterns",
            self.original_size,
            self.tokenized_size,
            self.compression_ratio * 100.0,
            self.patterns_found
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_tokenization() {
        let t = Tokenizer::new();

        // Test directory tokenization
        let tokens = t.tokenize("node_modules");
        assert_eq!(tokens, vec![0x80]);

        // Test decoding
        let decoded = t.decode(&tokens);
        assert_eq!(decoded, "node_modules");
    }

    #[test]
    fn test_path_tokenization() {
        let t = Tokenizer::new();

        let original = "src/main.rs";
        let tokens = t.tokenize(original);
        assert!(tokens.len() < original.len());

        let decoded = t.decode(&tokens);
        assert_eq!(decoded, original);
    }

    #[test]
    fn test_compression_ratio() {
        let t = Tokenizer::new();

        let text = "node_modules/package.json";
        let ratio = t.compression_ratio(text);
        assert!(ratio < 0.5); // Should compress to less than 50%
    }

    #[test]
    fn test_quantum_tokenization() {
        let qt = QuantumTokenizer::new();

        let text = "src/main.rs";
        let tokens = qt.quantum_tokenize(text);
        assert!(tokens.len() <= 3); // Should be highly compressed
    }
}