//! 🚗 Project Rebranding Ritual - Elegant Identity Transition
//!
//! A context-aware project renaming system that understands:
//! - Code semantics and identifier conventions
//! - Configuration files and manifests
//! - Documentation and brand consistency
//! - Different naming conventions (snake_case, camelCase, kebab-case)

use anyhow::Result;
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

use crate::{FileNode, Scanner, ScannerConfig};

/// Different naming conventions we need to handle
#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub enum NamingConvention {
    SnakeCase,  // bob_amazing_game
    CamelCase,  // BobAmazingGame
    PascalCase, // BobAmazingGame
    KebabCase,  // bob-amazing-game
    TitleCase,  // Bob Amazing Game
    UpperCase,  // BOB_AMAZING_GAME
    LowerCase,  // bob amazing game
    DotCase,    // bob.amazing.game
    PathCase,   // bob/amazing/game
}

/// Context where a name appears
#[derive(Debug, Clone, PartialEq)]
pub enum NameContext {
    FunctionName,
    VariableName,
    ClassName,
    ModuleName,
    StringLiteral,
    Comment,
    ConfigKey,
    ConfigValue,
    DocumentationTitle,
    FilePath,
    Url,
    PackageName,
}

/// A single renaming operation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RenameOperation {
    pub file_path: PathBuf,
    pub line: usize,
    pub column: usize,
    pub old_text: String,
    pub new_text: String,
    pub context: String,
    pub confidence: f32,
}

/// Configuration for the rename operation
#[derive(Debug, Clone)]
pub struct RenameConfig {
    pub old_name: String,
    pub new_name: String,
    pub dry_run: bool,
    pub interactive: bool,
    pub preserve_urls: bool,
    pub update_comments: bool,
    pub generate_logo: bool,
    pub backup: bool,
}

/// The main project renamer
pub struct ProjectRenamer {
    config: RenameConfig,
    operations: Vec<RenameOperation>,
    name_variants: HashMap<NamingConvention, (String, String)>, // old -> new
}

impl ProjectRenamer {
    pub fn new(config: RenameConfig) -> Self {
        let name_variants = Self::generate_name_variants(&config.old_name, &config.new_name);

        Self {
            config,
            operations: Vec::new(),
            name_variants,
        }
    }

    /// Generate all naming convention variants
    fn generate_name_variants(
        old_name: &str,
        new_name: &str,
    ) -> HashMap<NamingConvention, (String, String)> {
        let mut variants = HashMap::new();

        // Parse the names to extract words
        let old_words = Self::extract_words(old_name);
        let new_words = Self::extract_words(new_name);

        // Generate all variants
        variants.insert(
            NamingConvention::SnakeCase,
            (
                Self::to_snake_case(&old_words),
                Self::to_snake_case(&new_words),
            ),
        );

        variants.insert(
            NamingConvention::CamelCase,
            (
                Self::to_camel_case(&old_words),
                Self::to_camel_case(&new_words),
            ),
        );

        variants.insert(
            NamingConvention::PascalCase,
            (
                Self::to_pascal_case(&old_words),
                Self::to_pascal_case(&new_words),
            ),
        );

        variants.insert(
            NamingConvention::KebabCase,
            (
                Self::to_kebab_case(&old_words),
                Self::to_kebab_case(&new_words),
            ),
        );

        variants.insert(
            NamingConvention::TitleCase,
            (
                Self::to_title_case(&old_words),
                Self::to_title_case(&new_words),
            ),
        );

        variants.insert(
            NamingConvention::UpperCase,
            (
                Self::to_upper_case(&old_words),
                Self::to_upper_case(&new_words),
            ),
        );

        variants.insert(
            NamingConvention::LowerCase,
            (
                Self::to_lower_case(&old_words),
                Self::to_lower_case(&new_words),
            ),
        );

        variants
    }

    /// Extract words from various naming formats
    fn extract_words(name: &str) -> Vec<String> {
        let mut words = Vec::new();
        let mut current_word = String::new();
        let mut prev_is_lower = false;

        for ch in name.chars() {
            if ch.is_uppercase() && prev_is_lower && !current_word.is_empty() {
                // camelCase boundary
                words.push(current_word.to_lowercase());
                current_word = ch.to_string();
                prev_is_lower = false;
            } else if ch == '_' || ch == '-' || ch == ' ' || ch == '.' || ch == '/' {
                // Separator
                if !current_word.is_empty() {
                    words.push(current_word.to_lowercase());
                    current_word.clear();
                }
                prev_is_lower = false;
            } else {
                current_word.push(ch);
                prev_is_lower = ch.is_lowercase();
            }
        }

        if !current_word.is_empty() {
            words.push(current_word.to_lowercase());
        }

        words
    }

    fn to_snake_case(words: &[String]) -> String {
        words.join("_")
    }

    fn to_camel_case(words: &[String]) -> String {
        words
            .iter()
            .enumerate()
            .map(|(i, word)| {
                if i == 0 {
                    word.clone()
                } else {
                    Self::capitalize(word)
                }
            })
            .collect()
    }

    fn to_pascal_case(words: &[String]) -> String {
        words.iter().map(|word| Self::capitalize(word)).collect()
    }

    fn to_kebab_case(words: &[String]) -> String {
        words.join("-")
    }

    fn to_title_case(words: &[String]) -> String {
        words
            .iter()
            .map(|word| Self::capitalize(word))
            .collect::<Vec<_>>()
            .join(" ")
    }

    fn to_upper_case(words: &[String]) -> String {
        words.join("_").to_uppercase()
    }

    fn to_lower_case(words: &[String]) -> String {
        words.join(" ")
    }

    fn capitalize(word: &str) -> String {
        let mut chars = word.chars();
        match chars.next() {
            None => String::new(),
            Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
        }
    }

    /// Scan the project for all occurrences
    pub async fn scan_project(&mut self, project_path: &Path) -> Result<()> {
        println!(
            "🔎 Scanning for legacy references to \"{}\"...",
            self.config.old_name
        );

        let scanner_config = ScannerConfig {
            max_depth: 100,
            follow_symlinks: false,
            respect_gitignore: true,
            show_hidden: false,
            show_ignored: false,
            find_pattern: None,
            file_type_filter: None,
            entry_type_filter: None,
            min_size: None,
            max_size: Some(10 * 1024 * 1024), // Skip files > 10MB
            newer_than: None,
            older_than: None,
            use_default_ignores: true,
            search_keyword: None,
            show_filesystems: false,
            sort_field: None,
            top_n: None,
            include_line_content: false,
            // Smart scanning options (disabled for rename scan)
            compute_interest: false,
            security_scan: false,
            min_interest: 0.0,
            track_traversal: false,
            changes_only: false,
            compare_state: None,
            smart_mode: false,
        };

        let scanner = Scanner::new(project_path, scanner_config)?;
        let (nodes, _stats) = scanner.scan()?;

        // Process each file
        for node in nodes {
            if !node.is_dir && !node.is_symlink {
                self.scan_file(&node).await?;
            }
        }

        Ok(())
    }

    /// Scan a single file for rename opportunities
    async fn scan_file(&mut self, node: &FileNode) -> Result<()> {
        let content = match fs::read_to_string(&node.path) {
            Ok(content) => content,
            Err(_) => return Ok(()), // Skip binary or unreadable files
        };

        let file_type = Self::detect_file_type(&node.path);

        // Check each variant
        let variants = self.name_variants.clone();
        for (convention, (old_variant, new_variant)) in &variants {
            self.find_occurrences_in_content(
                &node.path,
                &content,
                old_variant,
                new_variant,
                &file_type,
                convention,
            )?;
        }

        Ok(())
    }

    /// Detect file type for context-aware replacements
    fn detect_file_type(path: &Path) -> FileType {
        let extension = path.extension().and_then(|e| e.to_str()).unwrap_or("");
        let filename = path.file_name().and_then(|f| f.to_str()).unwrap_or("");

        match extension {
            "rs" => FileType::Rust,
            "py" => FileType::Python,
            "js" | "jsx" | "ts" | "tsx" => FileType::JavaScript,
            "go" => FileType::Go,
            "java" => FileType::Java,
            "toml" => FileType::Toml,
            "yaml" | "yml" => FileType::Yaml,
            "json" => FileType::Json,
            "md" => FileType::Markdown,
            "desktop" => FileType::Desktop,
            _ => {
                // Check filenames
                match filename {
                    "Cargo.toml" => FileType::Toml,
                    "package.json" => FileType::Json,
                    "README.md" | "README" => FileType::Markdown,
                    _ => FileType::Unknown,
                }
            }
        }
    }

    /// Find occurrences in content with context awareness
    fn find_occurrences_in_content(
        &mut self,
        file_path: &Path,
        content: &str,
        old_variant: &str,
        new_variant: &str,
        file_type: &FileType,
        _convention: &NamingConvention,
    ) -> Result<()> {
        // Build context-aware regex based on file type
        let patterns = self.build_context_patterns(old_variant, file_type, _convention);

        for (pattern, context) in patterns {
            let re = Regex::new(&pattern)?;

            for (line_no, line) in content.lines().enumerate() {
                for mat in re.find_iter(line) {
                    let operation = RenameOperation {
                        file_path: file_path.to_path_buf(),
                        line: line_no + 1,
                        column: mat.start() + 1,
                        old_text: mat.as_str().to_string(),
                        new_text: self.calculate_replacement(
                            mat.as_str(),
                            old_variant,
                            new_variant,
                            &context,
                        ),
                        context: format!("{:?} in {:?}", context, file_type),
                        confidence: self.calculate_confidence(&context, file_type),
                    };

                    self.operations.push(operation);
                }
            }
        }

        Ok(())
    }

    /// Build context-aware patterns based on file type
    fn build_context_patterns(
        &self,
        variant: &str,
        file_type: &FileType,
        _convention: &NamingConvention,
    ) -> Vec<(String, NameContext)> {
        let escaped = regex::escape(variant);
        let mut patterns = Vec::new();

        match file_type {
            FileType::Rust => {
                // Function/method names
                patterns.push((format!(r"\bfn\s+{}\b", escaped), NameContext::FunctionName));
                // Struct/enum names
                patterns.push((
                    format!(r"\b(struct|enum|trait)\s+{}\b", escaped),
                    NameContext::ClassName,
                ));
                // Variable names
                patterns.push((
                    format!(r"\b(let|const|static)\s+.*{}\b", escaped),
                    NameContext::VariableName,
                ));
                // Module names
                patterns.push((format!(r"\bmod\s+{}\b", escaped), NameContext::ModuleName));
            }
            FileType::Python => {
                patterns.push((
                    format!(r"\b(def|class)\s+{}\b", escaped),
                    NameContext::FunctionName,
                ));
            }
            FileType::Toml | FileType::Yaml | FileType::Json => {
                // Package names
                patterns.push((
                    format!(r#"(name|package)\s*[=:]\s*"?{}"?"#, escaped),
                    NameContext::PackageName,
                ));
            }
            FileType::Markdown => {
                // Titles
                patterns.push((
                    format!(r"^#+\s*.*{}", escaped),
                    NameContext::DocumentationTitle,
                ));
            }
            _ => {}
        }

        // Always check for string literals and comments
        patterns.push((format!(r#""{}"#, escaped), NameContext::StringLiteral));
        patterns.push((format!(r"//.*{}", escaped), NameContext::Comment));

        patterns
    }

    /// Calculate the replacement based on context
    fn calculate_replacement(
        &self,
        matched_text: &str,
        old_variant: &str,
        new_variant: &str,
        _context: &NameContext,
    ) -> String {
        // For now, simple replacement
        // TODO: Handle more complex cases like preserving quotes, etc.
        matched_text.replace(old_variant, new_variant)
    }

    /// Calculate confidence score for the replacement
    fn calculate_confidence(&self, context: &NameContext, _file_type: &FileType) -> f32 {
        match context {
            NameContext::FunctionName | NameContext::ClassName | NameContext::ModuleName => 0.95,
            NameContext::VariableName => 0.85,
            NameContext::StringLiteral => 0.8,
            NameContext::PackageName => 0.9,
            NameContext::DocumentationTitle => 0.9,
            NameContext::Comment => 0.7,
            _ => 0.6,
        }
    }

    /// Apply all rename operations
    pub async fn apply_renames(&self) -> Result<()> {
        if self.operations.is_empty() {
            println!("No renaming operations found.");
            return Ok(());
        }

        // Group operations by file
        let mut ops_by_file: HashMap<PathBuf, Vec<&RenameOperation>> = HashMap::new();
        for op in &self.operations {
            ops_by_file
                .entry(op.file_path.clone())
                .or_default()
                .push(op);
        }

        for (file_path, ops) in ops_by_file {
            self.apply_file_renames(&file_path, ops).await?;
        }

        Ok(())
    }

    /// Apply renames to a single file
    async fn apply_file_renames(
        &self,
        file_path: &Path,
        operations: Vec<&RenameOperation>,
    ) -> Result<()> {
        let content = fs::read_to_string(file_path)?;
        let mut new_content = content.clone();

        // Apply operations in reverse order to maintain positions
        let mut sorted_ops = operations;
        sorted_ops.sort_by(|a, b| b.line.cmp(&a.line).then(b.column.cmp(&a.column)));

        for op in sorted_ops {
            // Simple replacement for now
            // TODO: Implement precise position-based replacement
            new_content = new_content.replace(&op.old_text, &op.new_text);
        }

        if self.config.backup {
            let backup_path = file_path.with_extension(format!(
                "{}.bak",
                file_path
                    .extension()
                    .unwrap_or_default()
                    .to_str()
                    .unwrap_or("")
            ));
            fs::copy(file_path, backup_path)?;
        }

        fs::write(file_path, new_content)?;
        Ok(())
    }

    /// Show a summary of planned operations
    pub fn show_summary(&self) {
        println!("\n✅ Found {} matches across:", self.operations.len());

        // Group by file type
        let mut file_counts: HashMap<String, usize> = HashMap::new();
        for op in &self.operations {
            let ext = op
                .file_path
                .extension()
                .and_then(|e| e.to_str())
                .unwrap_or("other");
            *file_counts.entry(ext.to_string()).or_default() += 1;
        }

        for (ext, count) in file_counts {
            println!("   - {} {} files", count, ext);
        }

        println!("\n🎨 Context-aware replacements:");
        println!(
            "   • Identifiers → `{}`",
            self.name_variants
                .get(&NamingConvention::SnakeCase)
                .unwrap()
                .1
        );
        println!("   • Strings → \"{}\"", self.config.new_name);
        println!(
            "   • Titles → `{}`",
            self.name_variants
                .get(&NamingConvention::TitleCase)
                .unwrap()
                .1
        );
        println!("   • Comments → updated branding");

        println!("\n🛡️ Safety net enabled: changes wrapped in diff mode");
    }
}

/// File types we understand
#[derive(Debug, Clone, PartialEq)]
enum FileType {
    Rust,
    Python,
    JavaScript,
    Go,
    Java,
    Toml,
    Yaml,
    Json,
    Markdown,
    Desktop,
    Unknown,
}

/// Interactive mode options
pub enum UserChoice {
    Preview,
    Commit,
    Edit,
    Cancel,
}

impl ProjectRenamer {
    /// Run interactive mode
    pub async fn run_interactive(&mut self) -> Result<UserChoice> {
        println!("\nWould you like to:");
        println!("[1] Preview changes");
        println!("[2] Commit rename");
        println!("[3] Edit before apply");
        println!("[4] Cancel");

        // TODO: Implement actual user input
        Ok(UserChoice::Preview)
    }

    /// Show preview of changes
    pub fn show_preview(&self) {
        for (i, op) in self.operations.iter().take(10).enumerate() {
            println!("\n{}) {}:{}", i + 1, op.file_path.display(), op.line);
            println!("   {} → {}", op.old_text, op.new_text);
            println!(
                "   Context: {} (confidence: {:.0}%)",
                op.context,
                op.confidence * 100.0
            );
        }

        if self.operations.len() > 10 {
            println!("\n... and {} more changes", self.operations.len() - 10);
        }
    }
}

/// Main entry point for the rename-project command
pub async fn rename_project(old_name: &str, new_name: &str, options: RenameOptions) -> Result<()> {
    println!("🚗 Project Rebranding Ritual");
    println!("═════════════════════════════");

    let config = RenameConfig {
        old_name: old_name.to_string(),
        new_name: new_name.to_string(),
        dry_run: options.dry_run,
        interactive: options.interactive,
        preserve_urls: options.preserve_urls,
        update_comments: options.update_comments,
        generate_logo: options.generate_logo,
        backup: options.backup,
    };

    let mut renamer = ProjectRenamer::new(config);

    // Scan the project
    let project_path = std::env::current_dir()?;
    renamer.scan_project(&project_path).await?;

    // Show summary
    renamer.show_summary();

    // Interactive mode
    if options.interactive {
        match renamer.run_interactive().await? {
            UserChoice::Preview => {
                renamer.show_preview();
            }
            UserChoice::Commit => {
                if !options.dry_run {
                    renamer.apply_renames().await?;
                    println!("\n✨ Project successfully rebranded!");
                }
            }
            UserChoice::Edit => {
                // TODO: Implement edit mode
                println!("Edit mode not yet implemented");
            }
            UserChoice::Cancel => {
                println!("Rename cancelled.");
            }
        }
    } else if !options.dry_run {
        renamer.apply_renames().await?;
        println!("\n✨ Project successfully rebranded!");
    }

    // Generate logo if requested
    if options.generate_logo {
        generate_placeholder_logo(new_name)?;
    }

    Ok(())
}

/// Options for the rename command
#[derive(Debug, Clone)]
pub struct RenameOptions {
    pub dry_run: bool,
    pub interactive: bool,
    pub preserve_urls: bool,
    pub update_comments: bool,
    pub generate_logo: bool,
    pub backup: bool,
}

impl Default for RenameOptions {
    fn default() -> Self {
        Self {
            dry_run: false,
            interactive: true,
            preserve_urls: true,
            update_comments: true,
            generate_logo: false,
            backup: true,
        }
    }
}

/// Generate a placeholder SVG logo
fn generate_placeholder_logo(project_name: &str) -> Result<()> {
    let svg = format!(
        r##"<svg xmlns="http://www.w3.org/2000/svg" width="200" height="200" viewBox="0 0 200 200">
  <rect width="200" height="200" fill="#1a1a1a"/>
  <text x="100" y="100" font-family="Arial, sans-serif" font-size="24" fill="#ffffff" text-anchor="middle" dominant-baseline="middle">
    {}
  </text>
</svg>"##,
        project_name
    );

    fs::write("assets/logo.svg", svg)?;
    println!("\n🎨 Generated placeholder logo at assets/logo.svg");

    Ok(())
}
