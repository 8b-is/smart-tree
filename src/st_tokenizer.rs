use once_cell::sync::Lazy;
use std::collections::HashMap;
use std::sync::RwLock;

pub static TOKEN_REGISTRY: Lazy<RwLock<TokenRegistry>> =
    Lazy::new(|| RwLock::new(TokenRegistry::new()));

pub type TokenId = u16;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TokenCategory {
    FileType,
    Permission,
    Size,
    Time,
    Path,
    Owner,
    Content,
    Semantic,
}

#[derive(Debug, Clone)]
pub struct SemanticToken {
    pub id: TokenId,
    pub category: TokenCategory,
    pub canonical: String,
    pub aliases: Vec<String>,
    pub frequency: u64,
}

pub struct TokenRegistry {
    tokens: HashMap<TokenId, SemanticToken>,
    pattern_map: HashMap<String, TokenId>,
    next_token_id: TokenId,
    pattern_frequency: HashMap<String, u64>,
}

impl Default for TokenRegistry {
    fn default() -> Self {
        Self::new()
    }
}

impl TokenRegistry {
    pub fn new() -> Self {
        let mut registry = Self {
            tokens: HashMap::new(),
            pattern_map: HashMap::new(),
            next_token_id: 0x0100,
            pattern_frequency: HashMap::new(),
        };
        registry.init_common_tokens();
        registry
    }

    fn init_common_tokens(&mut self) {
        self.add_token(0x0021, TokenCategory::FileType, "code.rust", vec![".rs"]);
        self.add_token(
            0x0022,
            TokenCategory::FileType,
            "code.python",
            vec![".py", ".pyw", ".pyi"],
        );
        self.add_token(
            0x0024,
            TokenCategory::FileType,
            "doc.markdown",
            vec![".md", ".markdown", ".mdown"],
        );
        self.add_token(0x0081, TokenCategory::Path, "vcs.git", vec![".git"]);
        self.add_token(
            0x0082,
            TokenCategory::Path,
            "dir.source",
            vec!["src", "source", "sources"],
        );
        self.add_token(
            0x0083,
            TokenCategory::Path,
            "dir.build.rust",
            vec!["target"],
        );
        self.add_token(
            0x0086,
            TokenCategory::Path,
            "dir.docs",
            vec!["docs", "doc", "documentation"],
        );
        self.add_token(
            0x00B0,
            TokenCategory::Semantic,
            "pkg.manifest",
            vec!["package.json", "Cargo.toml", "go.mod"],
        );
        self.add_token(
            0x00B1,
            TokenCategory::Semantic,
            "pkg.lock",
            vec!["package-lock.json", "Cargo.lock", "go.sum", "yarn.lock"],
        );
    }

    fn add_token(
        &mut self,
        id: TokenId,
        category: TokenCategory,
        canonical: &str,
        aliases: Vec<&str>,
    ) {
        let token = SemanticToken {
            id,
            category,
            canonical: canonical.to_string(),
            aliases: aliases.iter().map(|&s| s.to_string()).collect(),
            frequency: 0,
        };
        for alias in &token.aliases {
            self.pattern_map.insert(alias.clone(), id);
        }
        self.pattern_map.insert(canonical.to_string(), id);
        self.tokens.insert(id, token);
    }

    pub fn get_token(&self, pattern: &str) -> Option<TokenId> {
        self.pattern_map.get(pattern).copied()
    }

    pub fn get_semantic_token(&self, id: TokenId) -> Option<&SemanticToken> {
        self.tokens.get(&id)
    }

    pub fn record_usage(&mut self, pattern: &str) {
        *self
            .pattern_frequency
            .entry(pattern.to_string())
            .or_insert(0) += 1;
        if let Some(&token_id) = self.pattern_map.get(pattern) {
            if let Some(token) = self.tokens.get_mut(&token_id) {
                token.frequency += 1;
            }
        }
    }

    pub fn get_or_create_token(&mut self, pattern: &str, category: TokenCategory) -> TokenId {
        if let Some(&token_id) = self.pattern_map.get(pattern) {
            self.record_usage(pattern);
            return token_id;
        }
        let frequency = self.pattern_frequency.get(pattern).copied().unwrap_or(0);
        if frequency < 10 {
            self.record_usage(pattern);
            return 0;
        }
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
}

impl TokenRegistry {
    pub fn export_tokens(&self) -> Vec<(TokenId, String)> {
        let mut tokens: Vec<_> = self
            .tokens
            .iter()
            .map(|(&id, token)| (id, token.canonical.clone()))
            .collect();
        tokens.sort_by_key(|(id, _)| *id);
        tokens
    }

    pub fn are_equivalent(&self, pattern1: &str, pattern2: &str) -> bool {
        if pattern1 == pattern2 {
            return true;
        }
        match (self.get_token(pattern1), self.get_token(pattern2)) {
            (Some(id1), Some(id2)) => id1 == id2,
            _ => false,
        }
    }

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

pub fn tokenize_path(path: &str) -> Vec<TokenId> {
    let registry = TOKEN_REGISTRY.read().unwrap();
    let mut tokens = Vec::new();
    for component in path.split('/').filter(|s| !s.is_empty()) {
        if let Some(token) = registry.get_token(component) {
            tokens.push(token);
        }
    }
    tokens
}

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

fn split_basename_ext(s: &str) -> (&str, &str) {
    match s.rfind('.') {
        Some(idx) => (&s[..idx], &s[idx..]),
        None => (s, ""),
    }
}
