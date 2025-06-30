// Smart Tree Tokenizer - Semantic pattern recognition and token mapping
// This module handles the intelligent tokenization of filesystem patterns

use std::collections::HashMap;
use std::sync::RwLock;
use once_cell::sync::Lazy;

/// Global token registry for cross-system semantic equivalence
pub static TOKEN_REGISTRY: Lazy<RwLock<TokenRegistry>> = Lazy::new(|| {
    RwLock::new(TokenRegistry::new())
});

/// Token ID type - u16 gives us 65,535 possible tokens
pub type TokenId = u16;

/// Token categories for semantic grouping
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TokenCategory {
    FileType,      // Extensions, magic bytes
    Permission,    // Unix permissions
    Size,          // Size ranges
    Time,          // Date/time patterns
    Path,          // Common path components
    Owner,         // User/group patterns
    Content,       // Content-based tokens
    Semantic,      // High-level semantic tokens
}

/// A semantic token that can represent multiple equivalent forms
#[derive(Debug, Clone)]
pub struct SemanticToken {
    pub id: TokenId,
    pub category: TokenCategory,
    pub canonical: String,
    pub aliases: Vec<String>,
    pub frequency: u64,
}

/// The main token registry
pub struct TokenRegistry {
    /// Token ID to semantic token mapping
    tokens: HashMap<TokenId, SemanticToken>,
    
    /// String pattern to token ID mapping (includes all aliases)
    pattern_map: HashMap<String, TokenId>,
    
    /// Next available dynamic token ID
    next_token_id: TokenId,
    
    /// Frequency tracking for adaptive tokenization
    pattern_frequency: HashMap<String, u64>,
}

impl TokenRegistry {
    pub fn new() -> Self {
        let mut registry = Self {
            tokens: HashMap::new(),
            pattern_map: HashMap::new(),
            next_token_id: 0x0100, // Start after reserved range
            pattern_frequency: HashMap::new(),
        };
        
        // Initialize with common patterns
        registry.init_common_tokens();
        registry
    }
    
    fn init_common_tokens(&mut self) {
        // File extensions with semantic grouping
        self.add_token(0x0020, TokenCategory::FileType, "code.javascript", 
                      vec![".js", ".mjs", ".cjs", ".jsx"]);
        self.add_token(0x0021, TokenCategory::FileType, "code.rust", 
                      vec![".rs"]);
        self.add_token(0x0022, TokenCategory::FileType, "code.python", 
                      vec![".py", ".pyw", ".pyi"]);
        self.add_token(0x0023, TokenCategory::FileType, "code.go", 
                      vec![".go"]);
        self.add_token(0x0024, TokenCategory::FileType, "doc.markdown", 
                      vec![".md", ".markdown", ".mdown"]);
        self.add_token(0x0025, TokenCategory::FileType, "data.json", 
                      vec![".json", ".jsonl", ".ndjson"]);
        self.add_token(0x0026, TokenCategory::FileType, "config.yaml", 
                      vec![".yaml", ".yml"]);
        self.add_token(0x0027, TokenCategory::FileType, "doc.text", 
                      vec![".txt", ".text"]);
        self.add_token(0x0028, TokenCategory::FileType, "doc.readme", 
                      vec!["README", "README.md", "readme.md", "Readme.md"]);
        self.add_token(0x0029, TokenCategory::FileType, "legal.license", 
                      vec!["LICENSE", "LICENSE.md", "LICENSE.txt", "COPYING"]);
        
        // Common directory patterns
        self.add_token(0x0080, TokenCategory::Path, "pkg.node_modules", 
                      vec!["node_modules"]);
        self.add_token(0x0081, TokenCategory::Path, "vcs.git", 
                      vec![".git"]);
        self.add_token(0x0082, TokenCategory::Path, "dir.source", 
                      vec!["src", "source", "sources"]);
        self.add_token(0x0083, TokenCategory::Path, "dir.build.rust", 
                      vec!["target"]);
        self.add_token(0x0084, TokenCategory::Path, "dir.build", 
                      vec!["build", "out", "output"]);
        self.add_token(0x0085, TokenCategory::Path, "dir.dist", 
                      vec!["dist", "distribution"]);
        self.add_token(0x0086, TokenCategory::Path, "dir.docs", 
                      vec!["docs", "doc", "documentation"]);
        self.add_token(0x0087, TokenCategory::Path, "dir.tests", 
                      vec!["tests", "test", "__tests__", "spec"]);
        self.add_token(0x0088, TokenCategory::Path, "dir.vendor", 
                      vec!["vendor", "vendors", "third_party"]);
        self.add_token(0x0089, TokenCategory::Path, "dir.config", 
                      vec!["config", "conf", ".config", "configuration"]);
        
        // Permission patterns
        self.add_token(0x0010, TokenCategory::Permission, "perm.default_dir", 
                      vec!["755", "rwxr-xr-x"]);
        self.add_token(0x0011, TokenCategory::Permission, "perm.default_file", 
                      vec!["644", "rw-r--r--"]);
        self.add_token(0x0012, TokenCategory::Permission, "perm.world_write", 
                      vec!["777", "rwxrwxrwx"]);
        self.add_token(0x0013, TokenCategory::Permission, "perm.user_only", 
                      vec!["600", "rw-------"]);
        self.add_token(0x0014, TokenCategory::Permission, "perm.executable", 
                      vec!["755", "rwxr-xr-x", "775", "rwxrwxr-x"]);
        
        // Size range tokens
        self.add_token(0x00A0, TokenCategory::Size, "size.zero", vec!["0"]);
        self.add_token(0x00A1, TokenCategory::Size, "size.tiny", vec!["1-1K"]);
        self.add_token(0x00A2, TokenCategory::Size, "size.small", vec!["1K-100K"]);
        self.add_token(0x00A3, TokenCategory::Size, "size.medium", vec!["100K-10M"]);
        self.add_token(0x00A4, TokenCategory::Size, "size.large", vec!["10M+"]);
        
        // Semantic file patterns
        self.add_token(0x00B0, TokenCategory::Semantic, "pkg.manifest", 
                      vec!["package.json", "Cargo.toml", "go.mod", "pom.xml", "build.gradle"]);
        self.add_token(0x00B1, TokenCategory::Semantic, "pkg.lock", 
                      vec!["package-lock.json", "Cargo.lock", "go.sum", "yarn.lock"]);
        self.add_token(0x00B2, TokenCategory::Semantic, "vcs.ignore", 
                      vec![".gitignore", ".hgignore", ".svnignore"]);
        self.add_token(0x00B3, TokenCategory::Semantic, "config.env", 
                      vec![".env", ".env.local", ".env.production", "env.example"]);
        self.add_token(0x00B4, TokenCategory::Semantic, "config.editor", 
                      vec![".editorconfig", ".vscode", ".idea"]);
    }
    
    fn add_token(&mut self, id: TokenId, category: TokenCategory, 
                 canonical: &str, aliases: Vec<&str>) {
        let token = SemanticToken {
            id,
            category,
            canonical: canonical.to_string(),
            aliases: aliases.iter().map(|&s| s.to_string()).collect(),
            frequency: 0,
        };
        
        // Map all aliases to this token
        for alias in &token.aliases {
            self.pattern_map.insert(alias.clone(), id);
        }
        self.pattern_map.insert(canonical.to_string(), id);
        
        self.tokens.insert(id, token);
    }
    
    /// Look up a token by pattern
    pub fn get_token(&self, pattern: &str) -> Option<TokenId> {
        self.pattern_map.get(pattern).copied()
    }
    
    /// Get semantic token info
    pub fn get_semantic_token(&self, id: TokenId) -> Option<&SemanticToken> {
        self.tokens.get(&id)
    }
    
    /// Record pattern usage for adaptive tokenization
    pub fn record_usage(&mut self, pattern: &str) {
        *self.pattern_frequency.entry(pattern.to_string()).or_insert(0) += 1;
        
        // Update token frequency if it exists
        if let Some(&token_id) = self.pattern_map.get(pattern) {
            if let Some(token) = self.tokens.get_mut(&token_id) {
                token.frequency += 1;
            }
        }
    }
    
    /// Get or create a dynamic token for frequently used patterns
    pub fn get_or_create_token(&mut self, pattern: &str, category: TokenCategory) -> TokenId {
        // Check if we already have a token
        if let Some(&token_id) = self.pattern_map.get(pattern) {
            self.record_usage(pattern);
            return token_id;
        }
        
        // Check frequency threshold for dynamic token creation
        let frequency = self.pattern_frequency.get(pattern).copied().unwrap_or(0);
        if frequency < 10 {
            // Not frequent enough, record usage but don't create token
            self.record_usage(pattern);
            return 0; // Indicates no token
        }
        
        // Create new dynamic token
        let token_id = self.next_token_id;
        self.next_token_id += 1;
        
        let token = SemanticToken {
            id: token_id,
            category,
            canonical: pattern.to_string(),
            aliases: vec![],
            frequency,
        };
        
        self.pattern_map.insert(pattern.to_string(), token_id);
        self.tokens.insert(token_id, token);
        
        token_id
    }
    
    /// Export token table for transmission
    pub fn export_tokens(&self) -> Vec<(TokenId, String)> {
        let mut tokens: Vec<_> = self.tokens.iter()
            .map(|(&id, token)| (id, token.canonical.clone()))
            .collect();
        tokens.sort_by_key(|(id, _)| *id);
        tokens
    }
    
    /// Check semantic equivalence between patterns
    pub fn are_equivalent(&self, pattern1: &str, pattern2: &str) -> bool {
        if pattern1 == pattern2 {
            return true;
        }
        
        // Check if both map to the same token
        match (self.get_token(pattern1), self.get_token(pattern2)) {
            (Some(id1), Some(id2)) => id1 == id2,
            _ => false,
        }
    }
    
    /// Generate a semantic signature for deduplication
    pub fn semantic_signature(&self, components: &[&str]) -> u64 {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};
        
        let mut hasher = DefaultHasher::new();
        
        for component in components {
            if let Some(token_id) = self.get_token(component) {
                token_id.hash(&mut hasher);
            } else {
                component.hash(&mut hasher);
            }
        }
        
        hasher.finish()
    }
}

/// Smart tokenization of a full path
pub fn tokenize_path(path: &str) -> Vec<TokenId> {
    let registry = TOKEN_REGISTRY.read().unwrap();
    let mut tokens = Vec::new();
    
    for component in path.split('/').filter(|s| !s.is_empty()) {
        if let Some(token) = registry.get_token(component) {
            tokens.push(token);
        } else {
            // For non-tokenized components, we'd need a different encoding
            // For now, just skip
        }
    }
    
    tokens
}

/// Check if two paths are semantically equivalent
pub fn paths_equivalent(path1: &str, path2: &str) -> bool {
    let registry = TOKEN_REGISTRY.read().unwrap();

    let components1: Vec<_> = path1.split('/').filter(|s| !s.is_empty()).collect();
    let components2: Vec<_> = path2.split('/').filter(|s| !s.is_empty()).collect();

    if components1.len() != components2.len() {
        return false;
    }

    for (i, (c1, c2)) in components1.iter().zip(components2.iter()).enumerate() {
        if registry.are_equivalent(c1, c2) {
            continue;
        }
        // Special handling for the last component: check file extension equivalence
        if i == components1.len() - 1 {
            let (base1, ext1) = split_basename_ext(c1);
            let (base2, ext2) = split_basename_ext(c2);
            if base1 == base2 && registry.are_equivalent(ext1, ext2) {
                continue;
            }
        }
        return false;
    }
    true
}

// Helper function to split filename and extension
fn split_basename_ext(s: &str) -> (&str, &str) {
    match s.rfind('.') {
        Some(idx) => (&s[..idx], &s[idx..]),
        None => (s, ""),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_semantic_equivalence() {
        let registry = TOKEN_REGISTRY.read().unwrap();
        
        // Test extension equivalence
        assert!(registry.are_equivalent(".js", ".mjs"));
        assert!(registry.are_equivalent("README", "README.md"));
        assert!(registry.are_equivalent("src", "source"));
        
        // Test non-equivalence
        assert!(!registry.are_equivalent(".js", ".py"));
        assert!(!registry.are_equivalent("src", "dist"));
    }
    
    #[test]
    fn test_path_equivalence() {
        assert!(paths_equivalent("src/index.js", "source/index.mjs"));
        assert!(paths_equivalent("docs/README.md", "doc/README"));
        assert!(!paths_equivalent("src/main.rs", "src/main.py"));
    }
}