//! Tree-sitter based quantum compression - "Semantic awareness meets compression!" - Omni
//! Uses AST parsing to extract only the most meaningful code structures

use anyhow::Result;
use std::collections::HashMap;

// For now, we'll create a trait that can be implemented with tree-sitter later
pub trait LanguageQuantumParser {
    /// Extract semantically important nodes from source code
    fn extract_quantum_nodes(&self, source: &str) -> Result<Vec<QuantumNode>>;

    /// Score the importance of a node (0.0 to 1.0)
    fn score_importance(&self, node: &QuantumNode) -> f32;
}

#[derive(Debug, Clone)]
pub struct QuantumNode {
    pub kind: NodeKind,
    pub name: String,
    pub content: String,
    pub byte_range: (usize, usize),
    pub importance: f32,
}

#[derive(Debug, Clone, PartialEq)]
pub enum NodeKind {
    Function,
    Struct,
    Enum,
    Trait,
    Module,
    Import,
    Constant,
    Type,
    Test,
    Comment,
}

/// Rust language quantum parser
pub struct RustQuantumParser;

impl RustQuantumParser {
    pub fn new() -> Self {
        Self
    }

    /// Simplified version without tree-sitter dependency for now
    /// This demonstrates the concept until we add tree-sitter
    pub fn summarize_rust_code(&self, source_code: &str) -> Vec<String> {
        let mut highlights = vec![];

        // Simple regex-based extraction for now
        // TODO: Replace with tree-sitter AST parsing

        // Extract function signatures
        let fn_regex =
            regex::Regex::new(r"(?m)^[\s]*(?:pub\s+)?(?:async\s+)?fn\s+(\w+)[^{]+").unwrap();
        for cap in fn_regex.captures_iter(source_code) {
            if let Some(sig) = cap.get(0) {
                let sig_str = sig.as_str().trim();
                // Take only the signature, not the body
                if let Some(paren_end) = sig_str.rfind(')') {
                    let end = sig_str[paren_end..]
                        .find('{')
                        .map(|i| paren_end + i)
                        .unwrap_or(sig_str.len());
                    highlights.push(format!("fn: {}", sig_str[..end].trim()));
                }
            }
        }

        // Extract struct definitions
        let struct_regex = regex::Regex::new(r"(?m)^[\s]*(?:pub\s+)?struct\s+(\w+)").unwrap();
        for cap in struct_regex.captures_iter(source_code) {
            if let Some(name) = cap.get(1) {
                highlights.push(format!("struct: {}", name.as_str()));
            }
        }

        // Extract trait definitions
        let trait_regex = regex::Regex::new(r"(?m)^[\s]*(?:pub\s+)?trait\s+(\w+)").unwrap();
        for cap in trait_regex.captures_iter(source_code) {
            if let Some(name) = cap.get(1) {
                highlights.push(format!("trait: {}", name.as_str()));
            }
        }

        // Extract module definitions
        let mod_regex = regex::Regex::new(r"(?m)^[\s]*(?:pub\s+)?mod\s+(\w+)").unwrap();
        for cap in mod_regex.captures_iter(source_code) {
            if let Some(name) = cap.get(1) {
                highlights.push(format!("mod: {}", name.as_str()));
            }
        }

        highlights
    }
}

impl LanguageQuantumParser for RustQuantumParser {
    fn extract_quantum_nodes(&self, source: &str) -> Result<Vec<QuantumNode>> {
        let mut nodes = Vec::new();

        // Function extraction with importance scoring
        let fn_regex =
            regex::Regex::new(r"(?m)^[\s]*(?:pub\s+)?(?:async\s+)?fn\s+(\w+)[^{]+").unwrap();
        for cap in fn_regex.captures_iter(source) {
            if let (Some(full_match), Some(name)) = (cap.get(0), cap.get(1)) {
                let importance = if full_match.as_str().contains("pub") {
                    0.9
                } else if name.as_str() == "main" {
                    1.0
                } else if name.as_str().starts_with("test_") {
                    0.3
                } else {
                    0.6
                };

                nodes.push(QuantumNode {
                    kind: NodeKind::Function,
                    name: name.as_str().to_string(),
                    content: full_match.as_str().to_string(),
                    byte_range: (full_match.start(), full_match.end()),
                    importance,
                });
            }
        }

        // Sort by importance
        nodes.sort_by(|a, b| b.importance.partial_cmp(&a.importance).unwrap());

        Ok(nodes)
    }

    fn score_importance(&self, node: &QuantumNode) -> f32 {
        node.importance
    }
}

/// Python language quantum parser
pub struct PythonQuantumParser;

impl PythonQuantumParser {
    pub fn new() -> Self {
        Self
    }
}

impl LanguageQuantumParser for PythonQuantumParser {
    fn extract_quantum_nodes(&self, source: &str) -> Result<Vec<QuantumNode>> {
        let mut nodes = Vec::new();

        // Class extraction
        let class_regex = regex::Regex::new(r"(?m)^class\s+(\w+)").unwrap();
        for cap in class_regex.captures_iter(source) {
            if let (Some(full_match), Some(name)) = (cap.get(0), cap.get(1)) {
                nodes.push(QuantumNode {
                    kind: NodeKind::Struct, // Using Struct for classes
                    name: name.as_str().to_string(),
                    content: full_match.as_str().to_string(),
                    byte_range: (full_match.start(), full_match.end()),
                    importance: 0.8,
                });
            }
        }

        // Function extraction
        let fn_regex = regex::Regex::new(r"(?m)^def\s+(\w+)").unwrap();
        for cap in fn_regex.captures_iter(source) {
            if let (Some(full_match), Some(name)) = (cap.get(0), cap.get(1)) {
                let importance = if name.as_str() == "__init__" {
                    0.9
                } else if name.as_str().starts_with("_") {
                    0.4
                } else if name.as_str() == "main" {
                    1.0
                } else {
                    0.6
                };

                nodes.push(QuantumNode {
                    kind: NodeKind::Function,
                    name: name.as_str().to_string(),
                    content: full_match.as_str().to_string(),
                    byte_range: (full_match.start(), full_match.end()),
                    importance,
                });
            }
        }

        nodes.sort_by(|a, b| b.importance.partial_cmp(&a.importance).unwrap());
        Ok(nodes)
    }

    fn score_importance(&self, node: &QuantumNode) -> f32 {
        node.importance
    }
}

/// Factory for creating language-specific quantum parsers
pub struct QuantumParserFactory;

impl QuantumParserFactory {
    pub fn create_parser(language: &str) -> Option<Box<dyn LanguageQuantumParser>> {
        match language.to_lowercase().as_str() {
            "rust" | "rs" => Some(Box::new(RustQuantumParser::new())),
            "python" | "py" => Some(Box::new(PythonQuantumParser::new())),
            _ => None,
        }
    }
}

/// Quantum compression that uses semantic analysis
pub struct SemanticQuantumCompressor {
    parsers: HashMap<String, Box<dyn LanguageQuantumParser>>,
}

impl SemanticQuantumCompressor {
    pub fn new() -> Self {
        let mut parsers = HashMap::new();

        // Pre-register parsers
        parsers.insert(
            "rust".to_string(),
            Box::new(RustQuantumParser::new()) as Box<dyn LanguageQuantumParser>,
        );
        parsers.insert(
            "python".to_string(),
            Box::new(PythonQuantumParser::new()) as Box<dyn LanguageQuantumParser>,
        );

        Self { parsers }
    }

    /// Compress source code using semantic understanding
    pub fn compress_semantic(
        &self,
        source: &str,
        language: &str,
        max_nodes: usize,
    ) -> Result<String> {
        let parser = self
            .parsers
            .get(language)
            .ok_or_else(|| anyhow::anyhow!("Unsupported language: {}", language))?;

        let nodes = parser.extract_quantum_nodes(source)?;

        // Take only the most important nodes up to max_nodes
        let important_nodes: Vec<_> = nodes.into_iter().take(max_nodes).collect();

        // Build compressed representation
        let mut output = format!("QUANTUM_SEMANTIC_V1:lang={}\n", language);

        for node in important_nodes {
            output.push_str(&format!(
                "{:?}:{} [{:.2}]\n",
                node.kind, node.name, node.importance
            ));
        }

        Ok(output)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rust_quantum_parser() {
        let source = r#"
pub struct Scanner {
    root: PathBuf,
}

impl Scanner {
    pub fn new(path: &Path) -> Result<Self> {
        Ok(Self { root: path.to_path_buf() })
    }
    
    fn internal_method(&self) -> bool {
        true
    }
}

fn main() {
    println!("Hello!");
}

#[test]
fn test_scanner() {
    // test
}
"#;

        let parser = RustQuantumParser::new();
        let nodes = parser.extract_quantum_nodes(source).unwrap();

        // Should prioritize main > pub fn > private fn > test
        assert!(nodes[0].name == "main");
        assert!(nodes.iter().any(|n| n.name == "new" && n.importance > 0.8));
        assert!(nodes
            .iter()
            .any(|n| n.name == "test_scanner" && n.importance < 0.5));
    }
}
