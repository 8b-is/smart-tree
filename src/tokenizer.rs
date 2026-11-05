// Smart Tree Tokenizer - Turn patterns into bytes! 🗜️
// "Like 6502 opcodes - LDA is $A9, not 'LOAD ACCUMULATOR'" - Hue

use std::collections::HashMap;

/// Escape byte used to frame non-ASCII UTF-8 characters (`[0xFF, utf8_len, ..bytes]`)
/// so multi-byte Unicode never collides with dictionary tokens (`0x80..=0xEF`).
pub const UTF8_ESCAPE_TOKEN: u8 = 0xFF;

/// Maximum byte length of any registered dictionary pattern.
const MAX_TOKEN_PATTERN_LEN: usize = 16;

/// Common patterns tokenized to single bytes with O(1) first-byte length bitmask
/// acceleration and lossless UTF-8 round-trip encoding.
pub struct Tokenizer {
    /// Pattern → Token mapping
    patterns: HashMap<String, u8>,
    /// Token → Pattern for decoding
    tokens: HashMap<u8, String>,
    /// Direct 256-entry decode table for O(1) array-indexed token expansion
    decode_table: Box<[Option<String>; 256]>,
    /// Per-first-byte bitmask of valid pattern lengths (`bit (len - 1)` set for `len in 1..=16`)
    first_byte_len_mask: [u16; 256],
}

impl Default for Tokenizer {
    fn default() -> Self {
        Self::new()
    }
}

impl Tokenizer {
    pub fn new() -> Self {
        let mut t = Tokenizer {
            patterns: HashMap::with_capacity(64),
            tokens: HashMap::with_capacity(64),
            decode_table: Box::new(std::array::from_fn(|_| None)),
            first_byte_len_mask: [0u16; 256],
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
        debug_assert!(
            !pattern.is_empty() && pattern.len() <= MAX_TOKEN_PATTERN_LEN,
            "pattern length must be within 1..={MAX_TOKEN_PATTERN_LEN}"
        );
        debug_assert_ne!(
            token, UTF8_ESCAPE_TOKEN,
            "0xFF is reserved for UTF-8 escape"
        );
        let owned = pattern.to_string();
        let first_byte = pattern.as_bytes()[0] as usize;
        self.first_byte_len_mask[first_byte] |= 1u16 << (pattern.len() - 1);
        self.decode_table[token as usize] = Some(owned.clone());
        self.patterns.insert(owned.clone(), token);
        self.tokens.insert(token, owned);
    }

    /// Tokenize a string in O(N) time using first-byte length bitmask filtering
    /// and lossless UTF-8 escape framing (`0xFF`) for non-ASCII characters.
    pub fn tokenize(&self, text: &str) -> Vec<u8> {
        let mut result = Vec::with_capacity(text.len());
        let mut remaining = text;

        while !remaining.is_empty() {
            let first_byte = remaining.as_bytes()[0];
            let max_len = remaining.len().min(MAX_TOKEN_PATTERN_LEN);
            // Mask out lengths longer than `max_len`
            let len_mask = self.first_byte_len_mask[first_byte as usize]
                & ((1u32 << max_len).wrapping_sub(1) as u16);

            let mut found = false;
            let mut probe_mask = len_mask;
            while probe_mask != 0 {
                // Probe longest candidate length first via hardware CLZ
                let bit_idx = 15 - probe_mask.leading_zeros() as usize;
                probe_mask &= !(1u16 << bit_idx);
                let len = bit_idx + 1;

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
                if first_byte < 0x80 {
                    // Single-byte ASCII character
                    result.push(first_byte);
                    remaining = &remaining[1..];
                } else if let Some(ch) = remaining.chars().next() {
                    // Multi-byte UTF-8 character: frame as [0xFF, len, ..utf8_bytes]
                    let char_len = ch.len_utf8();
                    result.push(UTF8_ESCAPE_TOKEN);
                    result.push(char_len as u8);
                    result.extend_from_slice(&remaining.as_bytes()[..char_len]);
                    remaining = &remaining[char_len..];
                }
            }
        }

        result
    }

    /// Decode tokens back to string with O(1) table lookup and lossless UTF-8 restoration.
    pub fn decode(&self, tokens: &[u8]) -> String {
        let mut result = String::with_capacity(tokens.len() * 2);
        let mut i = 0;

        while i < tokens.len() {
            // Fast-path: use FELIX 16-byte SIMD high-bit scanner (`felix_find_high_byte`)
            // to bulk-copy contiguous ASCII spans (`< 0x80`) in a single `push_str`!
            if tokens[i] < 0x80 {
                let rem = &tokens[i..];
                let ascii_len = crate::felix_asm::felix_find_high_byte(rem).unwrap_or(rem.len());
                if let Ok(ascii_str) = std::str::from_utf8(&rem[..ascii_len]) {
                    result.push_str(ascii_str);
                    i += ascii_len;
                    continue;
                }
            }

            let token = tokens[i];
            if token == UTF8_ESCAPE_TOKEN {
                if i + 1 < tokens.len() {
                    let utf8_len = tokens[i + 1] as usize;
                    if (1..=4).contains(&utf8_len) && i + 2 + utf8_len <= tokens.len() {
                        let utf8_bytes = &tokens[i + 2..i + 2 + utf8_len];
                        if let Ok(s) = std::str::from_utf8(utf8_bytes) {
                            result.push_str(s);
                            i += 2 + utf8_len;
                            continue;
                        }
                    }
                }
                result.push_str("<FF>");
                i += 1;
            } else if let Some(pattern) = &self.decode_table[token as usize] {
                result.push_str(pattern);
                i += 1;
            } else if token < 0x80 {
                result.push(token as char);
                i += 1;
            } else {
                result.push_str(&format!("<{:02X}>", token));
                i += 1;
            }
        }

        result
    }

    /// Calculate compression ratio safely (returns `1.0` for empty strings instead of `NaN`).
    pub fn compression_ratio(&self, original: &str) -> f64 {
        if original.is_empty() {
            return 1.0;
        }
        let tokenized = self.tokenize(original);
        tokenized.len() as f64 / original.len() as f64
    }
}

/// Quantum tokenizer - second-stage multi-token path bigram/trigram compression (`0xE0..=0xEF`)!
pub struct QuantumTokenizer {
    base: Tokenizer,
    /// Multi-pattern combinations (`base_tokens -> combo_token`)
    combos: HashMap<Vec<u8>, u8>,
    /// Reverse lookup (`combo_token -> base_tokens`) for lossless `quantum_decode`
    reverse_combos: HashMap<u8, Vec<u8>>,
}

impl Default for QuantumTokenizer {
    fn default() -> Self {
        Self::new()
    }
}

impl QuantumTokenizer {
    pub fn new() -> Self {
        let mut qt = QuantumTokenizer {
            base: Tokenizer::new(),
            combos: HashMap::with_capacity(16),
            reverse_combos: HashMap::with_capacity(16),
        };

        // Real multi-token path combinations produced by `base.tokenize()` (0xE0-0xEF)
        qt.add_combo(0xE0, &[0xC0, 0xA3]); // "src/" (0xC0) + "main.rs" (0xA3) -> "src/main.rs" (1 byte!)
        qt.add_combo(0xE1, &[0xC0, 0xA4]); // "src/" (0xC0) + "index.js" (0xA4) -> "src/index.js"
        qt.add_combo(0xE2, &[0xC0, 0xA5]); // "src/" (0xC0) + "app.js" (0xA5) -> "src/app.js"
        qt.add_combo(0xE3, &[0xC4, 0x80]); // "./" (0xC4) + "node_modules" (0x80) -> "./node_modules"
        qt.add_combo(0xE4, &[0x80, b'/', 0xA1]); // "node_modules/package.json" -> 1 byte!
        qt.add_combo(0xE5, &[0xC0, b'l', b'i', b'b', 0x91]); // "src/lib.rs" -> 1 byte!
        qt.add_combo(0xE6, &[0xC0, b'm', b'o', b'd', 0x91]); // "src/mod.rs" -> 1 byte!
        qt.add_combo(0xE7, &[0xC1, 0xB0]); // "tests/" (0xC1) + "test_" (0xB0) -> "tests/test_"
                                           // Legacy adjacencies preserved in 0xE8..=0xEB
        qt.add_combo(0xE8, &[0x82, 0xC0]);
        qt.add_combo(0xE9, &[0x91, 0xA3]);
        qt.add_combo(0xEA, &[0x90, 0xA4]);
        qt.add_combo(0xEB, &[0x80, 0xC4]);

        qt
    }

    fn add_combo(&mut self, token: u8, pattern: &[u8]) {
        self.combos.insert(pattern.to_vec(), token);
        self.reverse_combos.insert(token, pattern.to_vec());
    }

    pub fn quantum_tokenize(&self, text: &str) -> Vec<u8> {
        let tokens = self.base.tokenize(text);

        // Second pass: combine multi-token sequences while preserving UTF-8 escape frames
        let mut result = Vec::with_capacity(tokens.len());
        let mut i = 0;

        while i < tokens.len() {
            if tokens[i] == UTF8_ESCAPE_TOKEN && i + 1 < tokens.len() {
                let utf8_len = tokens[i + 1] as usize;
                let frame_len = (2 + utf8_len).min(tokens.len() - i);
                result.extend_from_slice(&tokens[i..i + frame_len]);
                i += frame_len;
                continue;
            }

            let mut found = false;
            let max_combo = 5.min(tokens.len() - i);
            for len in (2..=max_combo).rev() {
                let candidate = &tokens[i..i + len];
                if !candidate.contains(&UTF8_ESCAPE_TOKEN) {
                    if let Some(&combo_token) = self.combos.get(candidate) {
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

    /// Decode quantum-compressed token stream back into the original string.
    pub fn quantum_decode(&self, tokens: &[u8]) -> String {
        let mut expanded = Vec::with_capacity(tokens.len() * 2);
        let mut i = 0;

        while i < tokens.len() {
            // Fast-path: use FELIX 16-byte SIMD high-bit scanner (`felix_find_high_byte`)
            // to bulk-copy contiguous ASCII spans (`< 0x80`) into `expanded`!
            if tokens[i] < 0x80 {
                let rem = &tokens[i..];
                let ascii_len = crate::felix_asm::felix_find_high_byte(rem).unwrap_or(rem.len());
                expanded.extend_from_slice(&rem[..ascii_len]);
                i += ascii_len;
                continue;
            }

            let token = tokens[i];
            if token == UTF8_ESCAPE_TOKEN && i + 1 < tokens.len() {
                let utf8_len = tokens[i + 1] as usize;
                let frame_len = (2 + utf8_len).min(tokens.len() - i);
                expanded.extend_from_slice(&tokens[i..i + frame_len]);
                i += frame_len;
            } else if let Some(combo_seq) = self.reverse_combos.get(&token) {
                expanded.extend_from_slice(combo_seq);
                i += 1;
            } else {
                expanded.push(token);
                i += 1;
            }
        }

        self.base.decode(&expanded)
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
        let mut patterns_found = 0;
        let mut i = 0;
        while i < tokens.len() {
            if tokens[i] == UTF8_ESCAPE_TOKEN && i + 1 < tokens.len() {
                i += 2 + tokens[i + 1] as usize;
            } else {
                if tokens[i] >= 0x80 {
                    patterns_found += 1;
                }
                i += 1;
            }
        }

        let compression_ratio = if original.is_empty() {
            1.0
        } else {
            tokens.len() as f64 / original.len() as f64
        };

        TokenStats {
            original_size: original.len(),
            tokenized_size: tokens.len(),
            compression_ratio,
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

        // Empty string must not produce NaN
        assert_eq!(t.compression_ratio(""), 1.0);
        let empty_stats = TokenStats::calculate("", &t);
        assert!(!empty_stats.compression_ratio.is_nan());
        assert_eq!(empty_stats.patterns_found, 0);
    }

    #[test]
    fn test_quantum_tokenization() {
        let qt = QuantumTokenizer::new();

        let text = "src/main.rs";
        let tokens = qt.quantum_tokenize(text);
        assert_eq!(tokens, vec![0xE0]); // Single combo token for "src/main.rs"!
        assert_eq!(qt.quantum_decode(&tokens), text);

        let pkg = "node_modules/package.json";
        let pkg_tokens = qt.quantum_tokenize(pkg);
        assert_eq!(pkg_tokens, vec![0xE4]);
        assert_eq!(qt.quantum_decode(&pkg_tokens), pkg);
    }

    #[test]
    fn test_multibyte_utf8_roundtrip_without_panic() {
        let t = Tokenizer::new();
        let qt = QuantumTokenizer::new();

        let unicode_path = "src/🦀_félicité_日本語_test_main.rs";
        let tokens = t.tokenize(unicode_path);
        assert_eq!(t.decode(&tokens), unicode_path);

        let q_tokens = qt.quantum_tokenize(unicode_path);
        assert_eq!(qt.quantum_decode(&q_tokens), unicode_path);
    }
}
