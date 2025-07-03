//! Code relationship analyzer - "Semantic X-ray vision for codebases" - Omni
//! Tracks imports, function calls, type usage, and test relationships

use anyhow::{Context, Result};
use regex::Regex;
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

/// Types of relationships between files
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum RelationType {
    /// Direct import/use/require
    Imports,
    /// Function defined here, called there
    FunctionCall,
    /// Type/struct/class defined here, used there
    TypeUsage,
    /// Test file testing this source
    TestedBy,
    /// Module exports this
    Exports,
    /// Tight coupling detected
    Coupled,
}

/// A relationship between two files
#[derive(Debug, Clone)]
pub struct FileRelation {
    /// Source file path
    pub source: PathBuf,
    /// Target file path
    pub target: PathBuf,
    /// Type of relationship
    pub relation_type: RelationType,
    /// Specific items involved (function names, types, etc.)
    pub items: Vec<String>,
    /// Strength of relationship (1-10)
    pub strength: u8,
}

/// Analyzes code relationships in a project
pub struct RelationAnalyzer {
    /// All discovered relationships
    relations: Vec<FileRelation>,
    /// Language-specific parsers
    parsers: HashMap<String, Box<dyn LanguageParser>>,
    /// File cache to avoid re-reading
    file_cache: HashMap<PathBuf, String>,
}

/// Language-specific parsing trait
trait LanguageParser: Send + Sync {
    /// Parse imports/uses from file content
    fn parse_imports(&self, content: &str, file_path: &Path) -> Vec<(String, Vec<String>)>;

    /// Parse function definitions
    fn parse_functions(&self, content: &str) -> Vec<String>;

    /// Parse function calls
    fn parse_function_calls(&self, content: &str) -> Vec<String>;

    /// Parse type definitions
    fn parse_types(&self, content: &str) -> Vec<String>;

    /// Parse type usages
    fn parse_type_usages(&self, content: &str) -> Vec<String>;
}

/// Rust language parser
struct RustParser;

impl LanguageParser for RustParser {
    fn parse_imports(&self, content: &str, _file_path: &Path) -> Vec<(String, Vec<String>)> {
        let mut imports = Vec::new();

        // Match use statements
        let use_re = Regex::new(r"use\s+(crate::)?([a-zA-Z0-9_:]+)(?:::\{([^}]+)\})?").unwrap();
        for cap in use_re.captures_iter(content) {
            let module = cap.get(2).map_or("", |m| m.as_str());
            let items = cap.get(3).map_or(vec![], |m| {
                m.as_str()
                    .split(',')
                    .map(|s| s.trim().to_string())
                    .collect()
            });
            imports.push((module.to_string(), items));
        }

        // Match mod statements
        let mod_re = Regex::new(r"mod\s+([a-zA-Z0-9_]+)").unwrap();
        for cap in mod_re.captures_iter(content) {
            let module = cap.get(1).map_or("", |m| m.as_str());
            imports.push((module.to_string(), vec![]));
        }

        imports
    }

    fn parse_functions(&self, content: &str) -> Vec<String> {
        let fn_re = Regex::new(r"(?:pub\s+)?fn\s+([a-zA-Z0-9_]+)").unwrap();
        fn_re
            .captures_iter(content)
            .filter_map(|cap| cap.get(1).map(|m| m.as_str().to_string()))
            .collect()
    }

    fn parse_function_calls(&self, content: &str) -> Vec<String> {
        let call_re = Regex::new(r"([a-zA-Z0-9_]+)\s*\(").unwrap();
        call_re
            .captures_iter(content)
            .filter_map(|cap| cap.get(1).map(|m| m.as_str().to_string()))
            .collect()
    }

    fn parse_types(&self, content: &str) -> Vec<String> {
        let mut types = Vec::new();

        // Structs
        let struct_re = Regex::new(r"(?:pub\s+)?struct\s+([A-Z][a-zA-Z0-9_]*)").unwrap();
        types.extend(
            struct_re
                .captures_iter(content)
                .filter_map(|cap| cap.get(1).map(|m| m.as_str().to_string())),
        );

        // Enums
        let enum_re = Regex::new(r"(?:pub\s+)?enum\s+([A-Z][a-zA-Z0-9_]*)").unwrap();
        types.extend(
            enum_re
                .captures_iter(content)
                .filter_map(|cap| cap.get(1).map(|m| m.as_str().to_string())),
        );

        // Traits
        let trait_re = Regex::new(r"(?:pub\s+)?trait\s+([A-Z][a-zA-Z0-9_]*)").unwrap();
        types.extend(
            trait_re
                .captures_iter(content)
                .filter_map(|cap| cap.get(1).map(|m| m.as_str().to_string())),
        );

        types
    }

    fn parse_type_usages(&self, content: &str) -> Vec<String> {
        let type_re = Regex::new(r":\s*([A-Z][a-zA-Z0-9_]*)").unwrap();
        type_re
            .captures_iter(content)
            .filter_map(|cap| cap.get(1).map(|m| m.as_str().to_string()))
            .collect()
    }
}

/// Python language parser
struct PythonParser;

impl LanguageParser for PythonParser {
    fn parse_imports(&self, content: &str, _file_path: &Path) -> Vec<(String, Vec<String>)> {
        let mut imports = Vec::new();

        // import module
        let import_re = Regex::new(r"import\s+([a-zA-Z0-9_.]+)").unwrap();
        for cap in import_re.captures_iter(content) {
            let module = cap.get(1).map_or("", |m| m.as_str());
            imports.push((module.to_string(), vec![]));
        }

        // from module import items
        let from_re = Regex::new(r"from\s+([a-zA-Z0-9_.]+)\s+import\s+(.+)").unwrap();
        for cap in from_re.captures_iter(content) {
            let module = cap.get(1).map_or("", |m| m.as_str());
            let items = cap.get(2).map_or(vec![], |m| {
                m.as_str()
                    .split(',')
                    .map(|s| s.trim().split_whitespace().next().unwrap_or("").to_string())
                    .collect()
            });
            imports.push((module.to_string(), items));
        }

        imports
    }

    fn parse_functions(&self, content: &str) -> Vec<String> {
        let fn_re = Regex::new(r"def\s+([a-zA-Z0-9_]+)").unwrap();
        fn_re
            .captures_iter(content)
            .filter_map(|cap| cap.get(1).map(|m| m.as_str().to_string()))
            .collect()
    }

    fn parse_function_calls(&self, content: &str) -> Vec<String> {
        let call_re = Regex::new(r"([a-zA-Z0-9_]+)\s*\(").unwrap();
        call_re
            .captures_iter(content)
            .filter_map(|cap| cap.get(1).map(|m| m.as_str().to_string()))
            .filter(|name| {
                !["if", "while", "for", "print", "len", "str", "int"].contains(&name.as_str())
            })
            .collect()
    }

    fn parse_types(&self, content: &str) -> Vec<String> {
        let class_re = Regex::new(r"class\s+([A-Z][a-zA-Z0-9_]*)").unwrap();
        class_re
            .captures_iter(content)
            .filter_map(|cap| cap.get(1).map(|m| m.as_str().to_string()))
            .collect()
    }

    fn parse_type_usages(&self, content: &str) -> Vec<String> {
        // Python type hints
        let type_re = Regex::new(r":\s*([A-Z][a-zA-Z0-9_\[\]]*)").unwrap();
        type_re
            .captures_iter(content)
            .filter_map(|cap| cap.get(1).map(|m| m.as_str().to_string()))
            .collect()
    }
}

impl RelationAnalyzer {
    /// Create a new analyzer
    pub fn new() -> Self {
        let mut parsers: HashMap<String, Box<dyn LanguageParser>> = HashMap::new();
        parsers.insert("rs".to_string(), Box::new(RustParser));
        parsers.insert("py".to_string(), Box::new(PythonParser));

        Self {
            relations: Vec::new(),
            parsers,
            file_cache: HashMap::new(),
        }
    }

    /// Analyze a directory for code relationships
    pub fn analyze_directory(&mut self, path: &Path) -> Result<()> {
        // First pass: collect all source files and their content
        self.collect_files(path)?;

        // Second pass: analyze relationships
        let files: Vec<PathBuf> = self.file_cache.keys().cloned().collect();
        for file in &files {
            self.analyze_file(file)?;
        }

        // Third pass: detect coupling and test relationships
        self.detect_coupling();
        self.detect_test_relationships();

        Ok(())
    }

    /// Collect all source files
    fn collect_files(&mut self, path: &Path) -> Result<()> {
        use walkdir::WalkDir;

        for entry in WalkDir::new(path)
            .follow_links(true)
            .into_iter()
            .filter_map(|e| e.ok())
            .filter(|e| e.file_type().is_file())
        {
            let path = entry.path();
            if let Some(ext) = path.extension() {
                if self.parsers.contains_key(ext.to_str().unwrap_or("")) {
                    let content =
                        fs::read_to_string(path).context(format!("Failed to read {path:?}"))?;
                    self.file_cache.insert(path.to_path_buf(), content);
                }
            }
        }

        Ok(())
    }

    /// Analyze a single file for relationships
    fn analyze_file(&mut self, file_path: &Path) -> Result<()> {
        let content = self
            .file_cache
            .get(file_path)
            .ok_or_else(|| anyhow::anyhow!("File not in cache"))?
            .clone();

        let ext = file_path.extension().and_then(|e| e.to_str()).unwrap_or("");

        if let Some(parser) = self.parsers.get(ext) {
            // Parse imports
            let imports = parser.parse_imports(&content, file_path);
            for (module, items) in imports {
                if let Some(target) = self.resolve_import(file_path, &module) {
                    self.relations.push(FileRelation {
                        source: file_path.to_path_buf(),
                        target,
                        relation_type: RelationType::Imports,
                        items,
                        strength: 8,
                    });
                }
            }

            // Parse functions and types for cross-referencing
            let _functions = parser.parse_functions(&content);
            let _types = parser.parse_types(&content);
            let _function_calls = parser.parse_function_calls(&content);
            let _type_usages = parser.parse_type_usages(&content);

            // Store for later cross-referencing
            // (In a real implementation, we'd build an index here to track
            // where functions are called and types are used, enabling deeper
            // analysis like call graphs and type dependency chains)
        }

        Ok(())
    }

    /// Resolve an import to a file path
    fn resolve_import(&self, from_file: &Path, module: &str) -> Option<PathBuf> {
        // Simplified resolution - in reality would be more complex
        let base_dir = from_file.parent()?;

        // Try direct file
        let direct = base_dir.join(format!("{}.rs", module));
        if self.file_cache.contains_key(&direct) {
            return Some(direct);
        }

        // Try module directory
        let mod_file = base_dir.join(module).join("mod.rs");
        if self.file_cache.contains_key(&mod_file) {
            return Some(mod_file);
        }

        None
    }

    /// Detect tightly coupled files
    fn detect_coupling(&mut self) {
        // Count bidirectional imports
        let mut import_pairs: HashMap<(PathBuf, PathBuf), u8> = HashMap::new();

        for rel in &self.relations {
            if rel.relation_type == RelationType::Imports {
                let pair = if rel.source < rel.target {
                    (rel.source.clone(), rel.target.clone())
                } else {
                    (rel.target.clone(), rel.source.clone())
                };
                *import_pairs.entry(pair).or_insert(0) += 1;
            }
        }

        // Mark bidirectional imports as coupled
        for ((file1, file2), count) in import_pairs {
            if count >= 2 {
                self.relations.push(FileRelation {
                    source: file1,
                    target: file2,
                    relation_type: RelationType::Coupled,
                    items: vec![],
                    strength: count.min(10),
                });
            }
        }
    }

    /// Detect test relationships
    fn detect_test_relationships(&mut self) {
        for file in self.file_cache.keys() {
            let file_str = file.to_string_lossy();

            // Is this a test file?
            if file_str.contains("test") || file_str.contains("_test") {
                // Find what it's testing
                let base_name = file
                    .file_stem()
                    .and_then(|s| s.to_str())
                    .unwrap_or("")
                    .replace("_test", "")
                    .replace("test_", "");

                // Look for matching source file
                for source in self.file_cache.keys() {
                    if source != file
                        && source
                            .file_stem()
                            .and_then(|s| s.to_str())
                            .map_or(false, |s| s == base_name)
                    {
                        self.relations.push(FileRelation {
                            source: source.clone(),
                            target: file.clone(),
                            relation_type: RelationType::TestedBy,
                            items: vec![],
                            strength: 10,
                        });
                    }
                }
            }
        }
    }

    /// Get all relationships
    pub fn get_relations(&self) -> &[FileRelation] {
        &self.relations
    }

    /// Get relationships for a specific file
    pub fn get_file_relations(&self, file: &Path) -> Vec<&FileRelation> {
        self.relations
            .iter()
            .filter(|r| r.source == file || r.target == file)
            .collect()
    }

    /// Get coupling score between two files
    pub fn get_coupling_score(&self, file1: &Path, file2: &Path) -> u8 {
        self.relations
            .iter()
            .filter(|r| {
                (r.source == file1 && r.target == file2) || (r.source == file2 && r.target == file1)
            })
            .map(|r| r.strength)
            .sum()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rust_parser() {
        let parser = RustParser;
        let content = r#"
use std::collections::HashMap;
use crate::scanner::{Scanner, FileInfo};
mod formatters;

pub fn process_file() {
    let scanner = Scanner::new();
}
"#;

        let imports = parser.parse_imports(content, Path::new("test.rs"));
        assert_eq!(imports.len(), 3);

        let functions = parser.parse_functions(content);
        assert_eq!(functions, vec!["process_file"]);
    }
}
