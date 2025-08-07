//! Smart Edit Tools - Revolutionary token-efficient code editing
//! By Aye, with inspiration from Omni's wave patterns
//! 
//! "Why send entire diffs when you can send intentions?" - Aye

use anyhow::{Result, Context};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::path::Path;
use tree_sitter::{Parser, Node};

/// Supported languages with their tree-sitter parsers
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SupportedLanguage {
    Rust,
    Python,
    JavaScript,
    TypeScript,
    Go,
    Java,
    CSharp,
    Cpp,
    Ruby,
}

impl SupportedLanguage {
    fn from_extension(ext: &str) -> Option<Self> {
        match ext {
            "rs" => Some(Self::Rust),
            "py" => Some(Self::Python),
            "js" | "mjs" => Some(Self::JavaScript),
            "ts" | "tsx" => Some(Self::TypeScript),
            "go" => Some(Self::Go),
            "java" => Some(Self::Java),
            "cs" => Some(Self::CSharp),
            "cpp" | "cc" | "cxx" | "hpp" | "h" => Some(Self::Cpp),
            "rb" => Some(Self::Ruby),
            _ => None,
        }
    }

    fn get_parser(&self) -> Result<Parser> {
        use tree_sitter_language::LanguageFn;
        
        let mut parser = Parser::new();
        let language_fn: LanguageFn = match self {
            Self::Rust => tree_sitter_rust::LANGUAGE,
            Self::Python => tree_sitter_python::LANGUAGE,
            Self::JavaScript => tree_sitter_javascript::LANGUAGE,
            Self::TypeScript => tree_sitter_typescript::LANGUAGE_TYPESCRIPT,
            Self::Go => tree_sitter_go::LANGUAGE,
            Self::Java => tree_sitter_java::LANGUAGE,
            Self::CSharp => tree_sitter_c_sharp::LANGUAGE,
            Self::Cpp => tree_sitter_cpp::LANGUAGE,
            Self::Ruby => tree_sitter_ruby::LANGUAGE,
        };
        let language = language_fn.into();
        parser.set_language(&language)?;
        Ok(parser)
    }
}

/// Smart edit operations that use minimal tokens
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "operation")]
pub enum SmartEdit {
    /// Insert a function at the appropriate location
    InsertFunction {
        name: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        class_name: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        namespace: Option<String>,
        body: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        after: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        before: Option<String>,
        #[serde(default)]
        visibility: String, // public, private, protected
    },
    
    /// Replace a function body (keeps signature)
    ReplaceFunction {
        name: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        class_name: Option<String>,
        new_body: String,
    },
    
    /// Add imports/use statements intelligently
    AddImport {
        import: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        alias: Option<String>,
    },
    
    /// Insert a class/struct
    InsertClass {
        name: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        namespace: Option<String>,
        body: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        extends: Option<String>,
        #[serde(default)]
        implements: Vec<String>,
    },
    
    /// Add a method to a class
    AddMethod {
        class_name: String,
        method_name: String,
        body: String,
        #[serde(default)]
        visibility: String,
    },
    
    /// Wrap code in a construct (try-catch, if statement, etc)
    WrapCode {
        start_line: usize,
        end_line: usize,
        wrapper_type: String, // "try", "if", "while", "for"
        #[serde(skip_serializing_if = "Option::is_none")]
        condition: Option<String>,
    },
    
    /// Delete a named element
    DeleteElement {
        element_type: String, // "function", "class", "method"
        name: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        parent: Option<String>,
    },
    
    /// Rename across the file
    Rename {
        old_name: String,
        new_name: String,
        #[serde(default)]
        scope: String, // "global", "class", "function"
    },
    
    /// Add documentation comment
    AddDocumentation {
        target_type: String, // "function", "class", "method"
        target_name: String,
        documentation: String,
    },
    
    /// Smart append - adds to the end of a logical section
    SmartAppend {
        section: String, // "imports", "functions", "classes", "main"
        content: String,
    },
    
    /// Remove a function with dependency awareness
    RemoveFunction {
        name: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        class_name: Option<String>,
        #[serde(default)]
        force: bool, // Remove even if it would break dependencies
        #[serde(default)]
        cascade: bool, // Also remove functions that only this one calls
    },
}

/// Function information for the function tree
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FunctionInfo {
    pub name: String,
    pub start_line: usize,
    pub end_line: usize,
    pub signature: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub class_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub namespace: Option<String>,
    pub visibility: String,
    #[serde(default)]
    pub calls: Vec<String>,
    #[serde(default)]
    pub called_by: Vec<String>,
}

/// Code structure representation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodeStructure {
    pub language: String,
    pub imports: Vec<String>,
    pub functions: Vec<FunctionInfo>,
    pub classes: Vec<ClassInfo>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub main_function: Option<String>,
    pub line_count: usize,
    #[serde(default)]
    pub dependencies: DependencyGraph,
}

/// Dependency graph for tracking function relationships
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct DependencyGraph {
    /// Map from function name to functions it calls
    pub calls: std::collections::HashMap<String, Vec<String>>,
    /// Map from function name to functions that call it
    pub called_by: std::collections::HashMap<String, Vec<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClassInfo {
    pub name: String,
    pub start_line: usize,
    pub end_line: usize,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub extends: Option<String>,
    #[serde(default)]
    pub implements: Vec<String>,
    pub methods: Vec<FunctionInfo>,
}

/// Smart editor that understands code structure
pub struct SmartEditor {
    content: String,
    language: SupportedLanguage,
    parser: Parser,
    tree: Option<tree_sitter::Tree>,
    structure: Option<CodeStructure>,
}

impl SmartEditor {
    pub fn new(content: String, language: SupportedLanguage) -> Result<Self> {
        let mut parser = language.get_parser()?;
        let tree = parser.parse(&content, None);
        
        let mut editor = Self {
            content,
            language,
            parser,
            tree,
            structure: None,
        };
        
        editor.analyze_structure()?;
        Ok(editor)
    }
    
    /// Analyze code structure to build a map
    fn analyze_structure(&mut self) -> Result<()> {
        let tree = self.tree.as_ref().context("No parse tree available")?;
        let root = tree.root_node();
        
        let mut structure = CodeStructure {
            language: format!("{:?}", self.language),
            imports: Vec::new(),
            functions: Vec::new(),
            classes: Vec::new(),
            main_function: None,
            line_count: self.content.lines().count(),
            dependencies: DependencyGraph::default(),
        };
        
        // Walk the tree and extract structure
        self.walk_node(&root, &mut structure, None)?;
        
        self.structure = Some(structure);
        Ok(())
    }
    
    fn walk_node(&self, node: &Node, structure: &mut CodeStructure, current_class: Option<&str>) -> Result<()> {
        match node.kind() {
            // Rust patterns
            "use_declaration" => {
                if let Some(text) = self.node_text(node) {
                    structure.imports.push(text);
                }
            }
            "function_item" | "method_definition" | "function_definition" | "function_declaration" => {
                if let Some(func_info) = self.extract_function_info(node, current_class) {
                    if func_info.name == "main" {
                        structure.main_function = Some(func_info.name.clone());
                    }
                    structure.functions.push(func_info);
                }
            }
            "struct_item" | "class_definition" | "class_declaration" => {
                if let Some(class_info) = self.extract_class_info(node) {
                    structure.classes.push(class_info);
                }
            }
            // Python patterns
            "import_statement" | "import_from_statement" => {
                if let Some(text) = self.node_text(node) {
                    structure.imports.push(text);
                }
            }
            _ => {}
        }
        
        // Handle class context for methods
        let class_name = match node.kind() {
            "class_definition" | "class_declaration" => {
                // Extract class name for method context
                self.find_child_by_kind(node, "identifier")
                    .and_then(|n| self.node_text(&n))
            }
            _ => None
        };
        
        let class_context = class_name.as_deref().or(current_class);
        
        // Recurse through children
        for child in node.children(&mut node.walk()) {
            self.walk_node(&child, structure, class_context)?;
        }
        
        Ok(())
    }
    
    fn node_text(&self, node: &Node) -> Option<String> {
        node.utf8_text(self.content.as_bytes()).ok().map(|s| s.to_string())
    }
    
    fn extract_function_info(&self, node: &Node, class_name: Option<&str>) -> Option<FunctionInfo> {
        let name = self.find_child_by_kind(node, "identifier")
            .or_else(|| self.find_child_by_kind(node, "property_identifier"))
            .and_then(|n| self.node_text(&n))?;
        
        let start_line = node.start_position().row + 1;
        let end_line = node.end_position().row + 1;
        
        let signature = self.extract_signature(node)?;
        
        Some(FunctionInfo {
            name,
            start_line,
            end_line,
            signature,
            class_name: class_name.map(String::from),
            namespace: None, // TODO: Extract namespace
            visibility: self.extract_visibility(node),
            calls: Vec::new(), // TODO: Extract function calls
            called_by: Vec::new(),
        })
    }
    
    fn extract_class_info(&self, node: &Node) -> Option<ClassInfo> {
        let name = self.find_child_by_kind(node, "identifier")
            .or_else(|| self.find_child_by_kind(node, "type_identifier"))
            .and_then(|n| self.node_text(&n))?;
        
        let start_line = node.start_position().row + 1;
        let end_line = node.end_position().row + 1;
        
        let mut methods = Vec::new();
        self.extract_methods(node, &name, &mut methods);
        
        Some(ClassInfo {
            name,
            start_line,
            end_line,
            extends: None, // TODO: Extract inheritance
            implements: Vec::new(),
            methods,
        })
    }
    
    fn extract_methods(&self, node: &Node, class_name: &str, methods: &mut Vec<FunctionInfo>) {
        for child in node.children(&mut node.walk()) {
            if matches!(child.kind(), "method_definition" | "function_item") {
                if let Some(method_info) = self.extract_function_info(&child, Some(class_name)) {
                    methods.push(method_info);
                }
            } else if child.kind().contains("body") {
                self.extract_methods(&child, class_name, methods);
            }
        }
    }
    
    fn find_child_by_kind<'a>(&self, node: &'a Node, kind: &str) -> Option<Node<'a>> {
        node.children(&mut node.walk()).find(|n| n.kind() == kind)
    }
    
    fn extract_signature(&self, node: &Node) -> Option<String> {
        // Simple extraction - can be enhanced per language
        let start = node.start_byte();
        let body_start = self.find_child_by_kind(node, "block")
            .or_else(|| self.find_child_by_kind(node, "body"))
            .map(|n| n.start_byte())
            .unwrap_or(node.end_byte());
        
        self.content.as_bytes()
            .get(start..body_start)
            .and_then(|bytes| std::str::from_utf8(bytes).ok())
            .map(|s| s.trim().to_string())
    }
    
    fn extract_visibility(&self, node: &Node) -> String {
        // Look for visibility modifiers
        for child in node.children(&mut node.walk()) {
            match child.kind() {
                "visibility_modifier" => {
                    if let Some(text) = self.node_text(&child) {
                        return text;
                    }
                }
                "pub" => return "public".to_string(),
                "private" => return "private".to_string(),
                "protected" => return "protected".to_string(),
                _ => {}
            }
        }
        "private".to_string() // Default
    }
    
    /// Apply a smart edit operation
    pub fn apply_edit(&mut self, edit: &SmartEdit) -> Result<String> {
        match edit {
            SmartEdit::InsertFunction { name, class_name, body, after, before, visibility, .. } => {
                self.insert_function(name, class_name.as_deref(), body, after.as_deref(), before.as_deref(), visibility)?;
            }
            SmartEdit::ReplaceFunction { name, class_name, new_body } => {
                self.replace_function(name, class_name.as_deref(), new_body)?;
            }
            SmartEdit::AddImport { import, alias } => {
                self.add_import(import, alias.as_deref())?;
            }
            SmartEdit::SmartAppend { section, content } => {
                self.smart_append(section, content)?;
            }
            SmartEdit::RemoveFunction { name, class_name, force, cascade } => {
                self.remove_function(name, class_name.as_deref(), *force, *cascade)?;
            }
            _ => {
                return Err(anyhow::anyhow!("Operation not yet implemented"));
            }
        }
        
        // Re-analyze structure after edit
        self.tree = self.parser.parse(&self.content, None);
        self.analyze_structure()?;
        
        Ok(self.content.clone())
    }
    
    fn insert_function(&mut self, name: &str, class_name: Option<&str>, body: &str, after: Option<&str>, before: Option<&str>, visibility: &str) -> Result<()> {
        let structure = self.structure.as_ref().context("No structure analyzed")?;
        
        
        // Find insertion point
        let insert_line = if let Some(after_name) = after {
            // Insert after specified function
            structure.functions.iter()
                .find(|f| f.name == after_name && f.class_name.as_deref() == class_name)
                .map(|f| f.end_line + 1)
                .with_context(|| format!("Function not found: {}", after_name))?
        } else if let Some(before_name) = before {
            // Insert before specified function
            structure.functions.iter()
                .find(|f| f.name == before_name && f.class_name.as_deref() == class_name)
                .map(|f| f.start_line.saturating_sub(1))
                .with_context(|| format!("Function not found: {}", before_name))?
        } else if let Some(class) = class_name {
            // Insert at end of class
            structure.classes.iter()
                .find(|c| c.name == class)
                .map(|c| {
                    // Find last method or class end
                    c.methods.iter()
                        .map(|m| m.end_line)
                        .max()
                        .unwrap_or(c.start_line) + 1
                })
                .context("Class not found: {class}")?
        } else {
            // Insert at end of file functions
            structure.functions.iter()
                .filter(|f| f.class_name.is_none())
                .map(|f| f.end_line)
                .max()
                .unwrap_or(structure.imports.len() + 1) + 1
        };
        
        // Format function based on language
        let formatted_function = self.format_function(name, body, visibility, class_name.is_some())?;
        
        // Insert at the calculated position
        let lines: Vec<&str> = self.content.lines().collect();
        let mut new_lines: Vec<String> = Vec::new();
        
        
        for (i, line) in lines.iter().enumerate() {
            new_lines.push(line.to_string());
            if i + 1 == insert_line {
                new_lines.push(String::new());
                new_lines.push(formatted_function.clone());
            }
        }
        
        // Handle case where we want to insert at the very end
        if insert_line > lines.len() {
            new_lines.push(String::new());
            new_lines.push(formatted_function);
        }
        
        self.content = new_lines.join("\n");
        Ok(())
    }
    
    fn format_function(&self, name: &str, body: &str, visibility: &str, is_method: bool) -> Result<String> {
        // Format based on language
        let formatted = match self.language {
            SupportedLanguage::Rust => {
                let vis = if visibility == "public" { "pub " } else { "" };
                let indent = if is_method { "    " } else { "" };
                format!("{indent}{vis}fn {name}{body}")
            }
            SupportedLanguage::Python => {
                let indent = if is_method { "    " } else { "" };
                format!("{indent}def {name}{body}")
            }
            SupportedLanguage::JavaScript | SupportedLanguage::TypeScript => {
                let indent = if is_method { "  " } else { "" };
                format!("{indent}function {name}{body}")
            }
            _ => {
                format!("{visibility} function {name}{body}")
            }
        };
        
        Ok(formatted)
    }
    
    fn replace_function(&mut self, name: &str, class_name: Option<&str>, new_body: &str) -> Result<()> {
        let structure = self.structure.as_ref().context("No structure analyzed")?;
        
        let function = structure.functions.iter()
            .find(|f| f.name == name && f.class_name.as_deref() == class_name)
            .context("Function not found")?;
        
        // Find the function body start (after signature)
        let lines: Vec<&str> = self.content.lines().collect();
        let signature_line = function.start_line - 1;
        
        // TODO: More robust body detection
        let body_start_line = signature_line + 1;
        let body_end_line = function.end_line - 1;
        
        // Replace the body
        let mut new_lines: Vec<String> = Vec::new();
        for (i, line) in lines.iter().enumerate() {
            if i < body_start_line || i > body_end_line {
                new_lines.push(line.to_string());
            } else if i == body_start_line {
                new_lines.push(new_body.to_string());
            }
        }
        
        self.content = new_lines.join("\n");
        Ok(())
    }
    
    fn add_import(&mut self, import: &str, alias: Option<&str>) -> Result<()> {
        let structure = self.structure.as_ref().context("No structure analyzed")?;
        
        // Format import based on language
        let formatted_import = match self.language {
            SupportedLanguage::Rust => {
                if let Some(alias) = alias {
                    format!("use {import} as {alias};")
                } else {
                    format!("use {import};")
                }
            }
            SupportedLanguage::Python => {
                if let Some(alias) = alias {
                    format!("import {import} as {alias}")
                } else {
                    format!("import {import}")
                }
            }
            SupportedLanguage::JavaScript | SupportedLanguage::TypeScript => {
                // For now, use CommonJS style which is more common
                if alias.is_some() {
                    format!("const {} = require('{}');", alias.unwrap(), import)
                } else {
                    format!("const {} = require('{}');", import, import)
                }
            }
            _ => format!("import {import};"),
        };
        
        // Find where to insert (after last import or at top)
        let insert_line = if structure.imports.is_empty() {
            1
        } else {
            structure.imports.len() + 1
        };
        
        let lines: Vec<&str> = self.content.lines().collect();
        let mut new_lines: Vec<String> = Vec::new();
        
        for (i, line) in lines.iter().enumerate() {
            if i + 1 == insert_line {
                new_lines.push(formatted_import.clone());
            }
            new_lines.push(line.to_string());
        }
        
        self.content = new_lines.join("\n");
        Ok(())
    }
    
    fn smart_append(&mut self, section: &str, content: &str) -> Result<()> {
        let structure = self.structure.as_ref().context("No structure analyzed")?;
        
        let insert_line = match section {
            "imports" => structure.imports.len() + 1,
            "functions" => {
                structure.functions.iter()
                    .filter(|f| f.class_name.is_none())
                    .map(|f| f.end_line)
                    .max()
                    .unwrap_or(structure.imports.len() + 1) + 1
            }
            "classes" => {
                structure.classes.iter()
                    .map(|c| c.end_line)
                    .max()
                    .unwrap_or_else(|| {
                        structure.functions.iter()
                            .map(|f| f.end_line)
                            .max()
                            .unwrap_or(structure.imports.len() + 1)
                    }) + 1
            }
            "main" => {
                if let Some(main_fn) = &structure.main_function {
                    structure.functions.iter()
                        .find(|f| &f.name == main_fn)
                        .map(|f| f.end_line - 1)
                        .unwrap_or(structure.line_count)
                } else {
                    structure.line_count
                }
            }
            _ => structure.line_count,
        };
        
        let lines: Vec<&str> = self.content.lines().collect();
        let mut new_lines: Vec<String> = Vec::new();
        
        for (i, line) in lines.iter().enumerate() {
            new_lines.push(line.to_string());
            if i + 1 == insert_line {
                new_lines.push(String::new());
                new_lines.push(content.to_string());
            }
        }
        
        self.content = new_lines.join("\n");
        Ok(())
    }
    
    /// Get the current code structure
    pub fn get_structure(&self) -> Option<&CodeStructure> {
        self.structure.as_ref()
    }
    
    fn remove_function(&mut self, name: &str, class_name: Option<&str>, force: bool, cascade: bool) -> Result<()> {
        // Extract data we need before borrowing self mutably
        let (function_start, function_end, functions_to_cascade) = {
            let structure = self.structure.as_ref().context("No structure analyzed")?;
            
            // Find the function to remove
            let function = structure.functions.iter()
                .find(|f| f.name == name && f.class_name.as_deref() == class_name)
                .context("Function not found")?;
            
            // Check dependencies unless force is set
            if !force {
                let dependents = structure.dependencies.called_by.get(name)
                    .map(|v| v.as_slice())
                    .unwrap_or(&[]);
                
                if !dependents.is_empty() {
                    return Err(anyhow::anyhow!(
                        "Function '{}' is called by: {}. Use force=true to remove anyway.",
                        name,
                        dependents.join(", ")
                    ));
                }
            }
            
            let mut functions_to_cascade = Vec::new();
            
            // Collect functions to cascade
            if cascade {
                if let Some(calls) = structure.dependencies.calls.get(name) {
                    for called_func in calls {
                        // Check if this is the only caller
                        if let Some(callers) = structure.dependencies.called_by.get(called_func) {
                            if callers.len() == 1 && callers[0] == name {
                                functions_to_cascade.push(called_func.clone());
                            }
                        }
                    }
                }
            }
            
            (function.start_line, function.end_line, functions_to_cascade)
        };
        
        // Remove the function lines
        let lines: Vec<&str> = self.content.lines().collect();
        let mut new_lines: Vec<String> = Vec::new();
        let mut skip_lines = false;
        
        for (i, line) in lines.iter().enumerate() {
            let line_num = i + 1;
            
            if line_num == function_start {
                skip_lines = true;
            }
            
            if !skip_lines {
                new_lines.push(line.to_string());
            }
            
            if line_num == function_end {
                skip_lines = false;
            }
        }
        
        self.content = new_lines.join("\n");
        
        // Re-analyze structure after modification
        self.tree = self.parser.parse(&self.content, None);
        self.analyze_structure()?;
        
        // Handle cascade removal
        for func_to_remove in functions_to_cascade {
            self.remove_function(&func_to_remove, None, true, cascade)?;
        }
        
        Ok(())
    }
    
    /// Get function tree with relationships
    pub fn get_function_tree(&self) -> Result<Value> {
        let structure = self.structure.as_ref().context("No structure analyzed")?;
        
        // Build call graph (simplified for now)
        let tree = json!({
            "language": format!("{:?}", self.language),
            "file_structure": {
                "imports": structure.imports,
                "line_count": structure.line_count,
                "main_function": structure.main_function,
            },
            "functions": structure.functions.iter().map(|f| {
                json!({
                    "name": f.name,
                    "lines": format!("{}-{}", f.start_line, f.end_line),
                    "class": f.class_name,
                    "visibility": f.visibility,
                    "signature": f.signature,
                    "calls": f.calls,
                    "called_by": f.called_by,
                })
            }).collect::<Vec<_>>(),
            "classes": structure.classes.iter().map(|c| {
                json!({
                    "name": c.name,
                    "lines": format!("{}-{}", c.start_line, c.end_line),
                    "extends": c.extends,
                    "implements": c.implements,
                    "methods": c.methods.iter().map(|m| {
                        json!({
                            "name": m.name,
                            "lines": format!("{}-{}", m.start_line, m.end_line),
                            "visibility": m.visibility,
                        })
                    }).collect::<Vec<_>>(),
                })
            }).collect::<Vec<_>>(),
        });
        
        Ok(tree)
    }
}

/// MCP tool handler for smart edit operations
pub async fn handle_smart_edit(params: Option<Value>) -> Result<Value> {
    let params = params.context("Parameters required")?;
    
    let file_path = params["file_path"]
        .as_str()
        .context("file_path required")?;
    
    let edits = params["edits"]
        .as_array()
        .context("edits array required")?;
    
    // Read file
    let content = std::fs::read_to_string(file_path)?;
    let extension = Path::new(file_path).extension()
        .and_then(|e| e.to_str())
        .context("Could not determine file extension")?;
    
    let language = SupportedLanguage::from_extension(extension)
        .context("Unsupported language")?;
    
    // Create smart editor
    let mut editor = SmartEditor::new(content, language)?;
    
    // Get initial structure
    let initial_structure = editor.get_function_tree()?;
    
    // Apply edits
    let mut results = Vec::new();
    for edit in edits {
        let smart_edit: SmartEdit = serde_json::from_value(edit.clone())?;
        match editor.apply_edit(&smart_edit) {
            Ok(_) => {
                results.push(json!({
                    "status": "success",
                    "operation": edit["operation"],
                }));
            }
            Err(e) => {
                results.push(json!({
                    "status": "error",
                    "operation": edit["operation"],
                    "error": e.to_string(),
                }));
            }
        }
    }
    
    // Get final structure
    let final_structure = editor.get_function_tree()?;
    
    // Write back to file
    std::fs::write(file_path, &editor.content)?;
    
    Ok(json!({
        "file_path": file_path,
        "language": format!("{:?}", language),
        "edits_applied": results,
        "initial_structure": initial_structure,
        "final_structure": final_structure,
        "content_preview": editor.content.lines().take(20).collect::<Vec<_>>().join("\n"),
    }))
}

/// Get function tree without making changes
pub async fn handle_get_function_tree(params: Option<Value>) -> Result<Value> {
    let params = params.context("Parameters required")?;
    let file_path = params["file_path"]
        .as_str()
        .context("file_path required")?;
    
    let content = std::fs::read_to_string(file_path)?;
    let extension = Path::new(file_path).extension()
        .and_then(|e| e.to_str())
        .context("Could not determine file extension")?;
    
    let language = SupportedLanguage::from_extension(extension)
        .context("Unsupported language")?;
    
    let editor = SmartEditor::new(content, language)?;
    editor.get_function_tree()
}

/// Insert a single function using minimal tokens
pub async fn handle_insert_function(params: Option<Value>) -> Result<Value> {
    let params = params.context("Parameters required")?;
    
    let edit = SmartEdit::InsertFunction {
        name: params["name"].as_str().context("name required")?.to_string(),
        class_name: params["class_name"].as_str().map(String::from),
        namespace: params["namespace"].as_str().map(String::from),
        body: params["body"].as_str().context("body required")?.to_string(),
        after: params["after"].as_str().map(String::from),
        before: params["before"].as_str().map(String::from),
        visibility: params["visibility"].as_str().unwrap_or("private").to_string(),
    };
    
    handle_smart_edit(Some(json!({
        "file_path": params["file_path"],
        "edits": [edit],
    }))).await
}

/// Remove a function with dependency checking
pub async fn handle_remove_function(params: Option<Value>) -> Result<Value> {
    let params = params.context("Parameters required")?;
    
    let edit = SmartEdit::RemoveFunction {
        name: params["name"].as_str().context("name required")?.to_string(),
        class_name: params["class_name"].as_str().map(String::from),
        force: params["force"].as_bool().unwrap_or(false),
        cascade: params["cascade"].as_bool().unwrap_or(false),
    };
    
    handle_smart_edit(Some(json!({
        "file_path": params["file_path"],
        "edits": [edit],
    }))).await
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_rust_function_insertion() {
        let content = r#"
use std::io;

fn main() {
    println!("Hello, world!");
}

fn helper() {
    println!("Helper");
}
"#.to_string();
        
        let mut editor = SmartEditor::new(content, SupportedLanguage::Rust).unwrap();
        let edit = SmartEdit::InsertFunction {
            name: "new_function".to_string(),
            class_name: None,
            namespace: None,
            body: r#"() -> Result<()> {
    println!("New function!");
    Ok(())
}"#.to_string(),
            after: Some("main".to_string()),
            before: None,
            visibility: "public".to_string(),
        };
        
        editor.apply_edit(&edit).unwrap();
        assert!(editor.content.contains("pub fn new_function"));
        assert!(editor.content.find("pub fn new_function").unwrap() > editor.content.find("fn main").unwrap());
    }
    
    #[test]
    fn test_python_function_insertion() {
        let content = r#"
import os

def main():
    print("Hello, world!")

def helper():
    print("Helper")
"#.to_string();
        
        let mut editor = SmartEditor::new(content, SupportedLanguage::Python).unwrap();
        let edit = SmartEdit::InsertFunction {
            name: "process_data".to_string(),
            class_name: None,
            namespace: None,
            body: r#"(data):
    """Process the data."""
    return data * 2"#.to_string(),
            after: Some("main".to_string()),
            before: None,
            visibility: "public".to_string(),
        };
        
        editor.apply_edit(&edit).unwrap();
        assert!(editor.content.contains("def process_data(data):"));
        assert!(editor.content.contains("return data * 2"));
    }
    
    #[test]
    fn test_javascript_function_insertion() {
        let content = r#"
function main() {
    console.log("Hello, world!");
}

function helper() {
    console.log("Helper");
}
"#.to_string();
        
        let mut editor = SmartEditor::new(content, SupportedLanguage::JavaScript).unwrap();
        let edit = SmartEdit::InsertFunction {
            name: "processData".to_string(),
            class_name: None,
            namespace: None,
            body: r#"(data) {
    return data.map(x => x * 2);
}"#.to_string(),
            before: Some("helper".to_string()),
            after: None,
            visibility: "public".to_string(),
        };
        
        editor.apply_edit(&edit).unwrap();
        assert!(editor.content.contains("function processData(data)"));
        assert!(editor.content.contains("return data.map(x => x * 2)"));
    }
    
    #[test]
    fn test_add_import() {
        let content = r#"
use std::io;

fn main() {
    println!("Hello");
}
"#.to_string();
        
        let mut editor = SmartEditor::new(content, SupportedLanguage::Rust).unwrap();
        let edit = SmartEdit::AddImport {
            import: "std::collections::HashMap".to_string(),
            alias: None,
        };
        
        editor.apply_edit(&edit).unwrap();
        assert!(editor.content.contains("use std::collections::HashMap;"));
        
        // Test with alias
        let edit_with_alias = SmartEdit::AddImport {
            import: "std::sync::Arc".to_string(),
            alias: Some("MyArc".to_string()),
        };
        
        editor.apply_edit(&edit_with_alias).unwrap();
        assert!(editor.content.contains("use std::sync::Arc as MyArc;"));
    }
    
    #[test]
    fn test_replace_function() {
        let content = r#"
fn calculate(x: i32) -> i32 {
    x + 1
}

fn main() {
    let result = calculate(5);
}
"#.to_string();
        
        let mut editor = SmartEditor::new(content, SupportedLanguage::Rust).unwrap();
        
        // First analyze to build structure
        let _ = editor.analyze_structure();
        
        let edit = SmartEdit::ReplaceFunction {
            name: "calculate".to_string(),
            class_name: None,
            new_body: r#"{
    // Improved calculation with logging
    println!("Calculating for: {}", x);
    x * 2
}"#.to_string(),
        };
        
        editor.apply_edit(&edit).unwrap();
        assert!(editor.content.contains("x * 2"));
        assert!(editor.content.contains("Improved calculation"));
        assert!(!editor.content.contains("x + 1")); // Old body should be gone
    }
    
    #[test]
    fn test_smart_append() {
        let content = r#"
import os

def main():
    pass

class MyClass:
    pass
"#.to_string();
        
        let mut editor = SmartEditor::new(content, SupportedLanguage::Python).unwrap();
        
        // Append to imports section
        let import_edit = SmartEdit::SmartAppend {
            section: "imports".to_string(),
            content: "import sys".to_string(),
        };
        
        editor.apply_edit(&import_edit).unwrap();
        assert!(editor.content.contains("import sys"));
        
        // Append to functions section
        let func_edit = SmartEdit::SmartAppend {
            section: "functions".to_string(),
            content: "def helper():\n    return True".to_string(),
        };
        
        editor.apply_edit(&func_edit).unwrap();
        assert!(editor.content.contains("def helper():"));
    }
    
    #[test]
    fn test_remove_function_with_dependencies() {
        let content = r#"
fn caller() {
    helper();
}

fn helper() {
    println!("I'm helping!");
}

fn orphan() {
    // Only called by helper
}
"#.to_string();
        
        let mut editor = SmartEditor::new(content, SupportedLanguage::Rust).unwrap();
        
        // Build dependency graph (simplified for test)
        editor.structure = Some(CodeStructure {
            language: "Rust".to_string(),
            imports: vec![],
            functions: vec![
                FunctionInfo {
                    name: "caller".to_string(),
                    class_name: None,
                    namespace: None,
                    start_line: 2,
                    end_line: 4,
                    signature: "fn caller()".to_string(),
                    visibility: "private".to_string(),
                    calls: vec!["helper".to_string()],
                    called_by: vec![],
                },
                FunctionInfo {
                    name: "helper".to_string(),
                    class_name: None,
                    namespace: None,
                    start_line: 6,
                    end_line: 8,
                    signature: "fn helper()".to_string(),
                    visibility: "private".to_string(),
                    calls: vec!["orphan".to_string()],
                    called_by: vec!["caller".to_string()],
                },
                FunctionInfo {
                    name: "orphan".to_string(),
                    class_name: None,
                    namespace: None,
                    start_line: 10,
                    end_line: 12,
                    signature: "fn orphan()".to_string(),
                    visibility: "private".to_string(),
                    calls: vec![],
                    called_by: vec!["helper".to_string()],
                },
            ],
            classes: vec![],
            main_function: None,
            line_count: 12,
            dependencies: DependencyGraph {
                calls: [
                    ("caller".to_string(), vec!["helper".to_string()]),
                    ("helper".to_string(), vec!["orphan".to_string()]),
                ].into_iter().collect(),
                called_by: [
                    ("helper".to_string(), vec!["caller".to_string()]),
                    ("orphan".to_string(), vec!["helper".to_string()]),
                ].into_iter().collect(),
            },
        });
        
        // Try to remove helper without force - should fail
        let remove_edit = SmartEdit::RemoveFunction {
            name: "helper".to_string(),
            class_name: None,
            force: false,
            cascade: false,
        };
        
        let result = editor.apply_edit(&remove_edit);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("called by: caller"));
        
        // Remove with force
        let force_remove = SmartEdit::RemoveFunction {
            name: "helper".to_string(),
            class_name: None,
            force: true,
            cascade: false,
        };
        
        editor.apply_edit(&force_remove).unwrap();
        assert!(!editor.content.contains("fn helper()"));
        assert!(editor.content.contains("fn orphan()")); // Orphan still there without cascade
    }
    
    #[test]
    fn test_get_function_tree() {
        let content = r#"
class Calculator:
    def add(self, a, b):
        return a + b
    
    def multiply(self, a, b):
        return self.add(a, b) * b

def main():
    calc = Calculator()
    result = calc.add(5, 3)
"#.to_string();
        
        let editor = SmartEditor::new(content, SupportedLanguage::Python).unwrap();
        let tree = editor.get_function_tree().unwrap();
        
        // Check tree structure
        assert!(tree["language"].as_str().unwrap().contains("Python"));
        assert!(tree["functions"].is_array());
        assert!(tree["classes"].is_array());
        
        // Verify it found the functions and classes
        let functions = tree["functions"].as_array().unwrap();
        assert!(functions.iter().any(|f| f["name"] == "main"));
        
        let classes = tree["classes"].as_array().unwrap();
        assert!(classes.iter().any(|c| c["name"] == "Calculator"));
    }
    
    #[test]
    fn test_multiple_edits() {
        let content = r#"
fn main() {
    println!("Start");
}
"#.to_string();
        
        let mut editor = SmartEditor::new(content, SupportedLanguage::Rust).unwrap();
        
        // Apply multiple edits
        let edits = vec![
            SmartEdit::AddImport {
                import: "std::thread".to_string(),
                alias: None,
            },
            SmartEdit::InsertFunction {
                name: "worker".to_string(),
                class_name: None,
                namespace: None,
                body: r#"() {
    thread::sleep(std::time::Duration::from_secs(1));
}"#.to_string(),
                after: Some("main".to_string()),
                before: None,
                visibility: "private".to_string(),
            },
        ];
        
        for edit in edits {
            editor.apply_edit(&edit).unwrap();
        }
        
        assert!(editor.content.contains("use std::thread;"));
        assert!(editor.content.contains("fn worker()"));
    }
    
    #[test]
    fn test_class_method_insertion() {
        let content = r#"
class MyClass:
    def __init__(self):
        self.value = 0
    
    def get_value(self):
        return self.value
"#.to_string();
        
        let mut editor = SmartEditor::new(content, SupportedLanguage::Python).unwrap();
        
        let edit = SmartEdit::InsertFunction {
            name: "set_value".to_string(),
            class_name: Some("MyClass".to_string()),
            namespace: None,
            body: r#"(self, value):
        self.value = value"#.to_string(),
            after: Some("get_value".to_string()),
            before: None,
            visibility: "public".to_string(),
        };
        
        editor.apply_edit(&edit).unwrap();
        assert!(editor.content.contains("def set_value(self, value):"));
        assert!(editor.content.contains("self.value = value"));
    }
}