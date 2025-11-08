//! SmartPastCode Registry Integration
//!
//! This module provides integration with the SmartPastCode universal code registry,
//! enabling automatic indexing of Rust projects and their components.

use anyhow::{Context, Result};
use reqwest::blocking::Client;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use std::time::Instant;
use syn::{File, Item, ItemFn, ItemImpl, ItemMod};

/// Component metadata for SmartPastCode registry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodeComponent {
    /// Unique identifier (content hash)
    pub id: String,

    /// Component type
    pub component_type: ComponentType,

    /// The actual Rust code
    pub content: String,

    /// Discovery metadata
    pub discovery_metadata: DiscoveryMetadata,

    /// Origin information
    pub origin: ComponentOrigin,

    /// Security clearance level
    pub clearance: ClearanceLevel,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub enum ComponentType {
    Function,
    Module,
    Class,
    MiniCrate,
    Folder,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiscoveryMetadata {
    /// Language (always "rust")
    pub language: String,

    /// Domain (networking, database, etc.)
    pub domains: Vec<String>,

    /// Purpose (authentication, parsing, etc.)
    pub purposes: Vec<String>,

    /// Keywords extracted from code
    pub keywords: Vec<String>,

    /// Is async code?
    pub is_async: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComponentOrigin {
    /// Project path
    pub project_path: String,

    /// File path
    pub file_path: String,

    /// Line number
    pub line_number: usize,

    /// Contributor (AI or human)
    pub contributor: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub enum ClearanceLevel {
    Private = 0,
    Team = 1,
    Internal = 2,
    CompanyPublic = 3,
    WorldPublic = 10,
}

/// Marine-inspired code analyzer for Rust
pub struct MarineCodeAnalyzer {
    client: Client,
    registry_url: String,
    contributor: String,
}

impl MarineCodeAnalyzer {
    /// Create a new analyzer
    pub fn new(registry_url: &str) -> Result<Self> {
        let client = Client::builder()
            .timeout(std::time::Duration::from_secs(30))
            .build()
            .context("Failed to create HTTP client")?;

        let contributor = whoami::username();

        Ok(Self {
            client,
            registry_url: registry_url.to_string(),
            contributor,
        })
    }

    /// Index a Rust file
    pub fn index_file(&self, file_path: &Path, project_path: &Path) -> Result<Vec<CodeComponent>> {
        let content = std::fs::read_to_string(file_path)
            .context("Failed to read file")?;

        let syntax: File = syn::parse_file(&content)
            .context("Failed to parse Rust file")?;

        let mut components = Vec::new();

        // Extract functions and impl blocks
        for item in &syntax.items {
            match item {
                Item::Fn(func) => {
                    if let Some(component) = self.extract_function(func, file_path, project_path, &content) {
                        components.push(component);
                    }
                }
                Item::Impl(impl_block) => {
                    for component in self.extract_impl_methods(impl_block, file_path, project_path, &content) {
                        components.push(component);
                    }
                }
                Item::Mod(module) => {
                    if let Some(component) = self.extract_module(module, file_path, project_path, &content) {
                        components.push(component);
                    }
                }
                _ => {}
            }
        }

        Ok(components)
    }

    /// Extract a function as a component
    fn extract_function(
        &self,
        func: &ItemFn,
        file_path: &Path,
        project_path: &Path,
        full_content: &str,
    ) -> Option<CodeComponent> {
        let _func_name = func.sig.ident.to_string();

        // Extract the function code
        let func_code = quote::quote!(#func).to_string();

        // Get line number
        let line_number = self.get_line_number(full_content, &func_code);

        // Analyze metadata
        let metadata = self.analyze_metadata(&func_code, &func.sig);

        // Generate ID from content hash
        let id = self.generate_id(&func_code);

        Some(CodeComponent {
            id,
            component_type: ComponentType::Function,
            content: func_code,
            discovery_metadata: metadata,
            origin: ComponentOrigin {
                project_path: project_path.display().to_string(),
                file_path: file_path.display().to_string(),
                line_number,
                contributor: self.contributor.clone(),
            },
            clearance: ClearanceLevel::WorldPublic,
        })
    }

    /// Extract methods from impl blocks
    fn extract_impl_methods(
        &self,
        impl_block: &ItemImpl,
        file_path: &Path,
        project_path: &Path,
        full_content: &str,
    ) -> Vec<CodeComponent> {
        let mut components = Vec::new();

        for item in &impl_block.items {
            if let syn::ImplItem::Fn(method) = item {
                let method_code = quote::quote!(#method).to_string();
                let line_number = self.get_line_number(full_content, &method_code);
                let metadata = self.analyze_metadata(&method_code, &method.sig);
                let id = self.generate_id(&method_code);

                components.push(CodeComponent {
                    id,
                    component_type: ComponentType::Function,
                    content: method_code,
                    discovery_metadata: metadata,
                    origin: ComponentOrigin {
                        project_path: project_path.display().to_string(),
                        file_path: file_path.display().to_string(),
                        line_number,
                        contributor: self.contributor.clone(),
                    },
                    clearance: ClearanceLevel::WorldPublic,
                });
            }
        }

        components
    }

    /// Extract module as a component
    fn extract_module(
        &self,
        module: &ItemMod,
        file_path: &Path,
        project_path: &Path,
        full_content: &str,
    ) -> Option<CodeComponent> {
        let module_code = quote::quote!(#module).to_string();
        let line_number = self.get_line_number(full_content, &module_code);
        let id = self.generate_id(&module_code);

        // Simple metadata for modules
        let metadata = DiscoveryMetadata {
            language: "rust".to_string(),
            domains: vec![],
            purposes: vec!["module".to_string()],
            keywords: vec![module.ident.to_string()],
            is_async: false,
        };

        Some(CodeComponent {
            id,
            component_type: ComponentType::Module,
            content: module_code,
            discovery_metadata: metadata,
            origin: ComponentOrigin {
                project_path: project_path.display().to_string(),
                file_path: file_path.display().to_string(),
                line_number,
                contributor: self.contributor.clone(),
            },
            clearance: ClearanceLevel::WorldPublic,
        })
    }

    /// Analyze code metadata
    fn analyze_metadata(&self, code: &str, sig: &syn::Signature) -> DiscoveryMetadata {
        let mut domains = Vec::new();
        let mut purposes = Vec::new();
        let mut keywords = Vec::new();

        // Detect async
        let is_async = sig.asyncness.is_some();
        if is_async {
            keywords.push("async".to_string());
        }

        // Detect domains from common imports
        if code.contains("tokio::net") || code.contains("async_std::net") {
            domains.push("networking".to_string());
        }
        if code.contains("sqlx") || code.contains("diesel") || code.contains("rusqlite") {
            domains.push("database".to_string());
        }
        if code.contains("serde") || code.contains("serde_json") {
            domains.push("serialization".to_string());
        }
        if code.contains("reqwest") || code.contains("hyper") {
            domains.push("http".to_string());
        }
        if code.contains("tokio::fs") || code.contains("std::fs") {
            domains.push("filesystem".to_string());
        }

        // Detect purposes from function names
        let func_name = sig.ident.to_string().to_lowercase();
        if func_name.contains("parse") {
            purposes.push("parsing".to_string());
        }
        if func_name.contains("validate") || func_name.contains("check") {
            purposes.push("validation".to_string());
        }
        if func_name.contains("auth") || func_name.contains("login") {
            purposes.push("authentication".to_string());
        }
        if func_name.contains("download") || func_name.contains("upload") {
            purposes.push("transfer".to_string());
        }
        if func_name.contains("process") || func_name.contains("handle") {
            purposes.push("processing".to_string());
        }

        // Add function name as keyword
        keywords.push(sig.ident.to_string());

        DiscoveryMetadata {
            language: "rust".to_string(),
            domains,
            purposes,
            keywords,
            is_async,
        }
    }

    /// Get line number of code in full content
    fn get_line_number(&self, full_content: &str, code: &str) -> usize {
        // Simple heuristic: count newlines before first occurrence
        let first_line = code.lines().next().unwrap_or("");
        if let Some(pos) = full_content.find(first_line) {
            full_content[..pos].lines().count() + 1
        } else {
            1
        }
    }

    /// Generate ID from content hash
    fn generate_id(&self, content: &str) -> String {
        use sha2::{Sha256, Digest};
        let mut hasher = Sha256::new();
        hasher.update(content.as_bytes());
        let result = hasher.finalize();
        hex::encode(result)
    }

    /// Submit component to registry
    pub fn submit_component(&self, component: &CodeComponent) -> Result<()> {
        let url = format!("{}/components/store", self.registry_url);

        let response = self.client
            .post(&url)
            .json(component)
            .send()
            .context("Failed to send component to registry")?;

        if !response.status().is_success() {
            anyhow::bail!(
                "Registry returned error: {} - {}",
                response.status(),
                response.text().unwrap_or_default()
            );
        }

        Ok(())
    }

    /// Batch submit components
    pub fn submit_batch(&self, components: &[CodeComponent]) -> Result<BatchResult> {
        let start = Instant::now();
        let mut success_count = 0;
        let mut error_count = 0;
        let mut errors = Vec::new();

        for component in components {
            match self.submit_component(component) {
                Ok(_) => success_count += 1,
                Err(e) => {
                    error_count += 1;
                    errors.push(format!("{}: {}", component.origin.file_path, e));
                }
            }
        }

        Ok(BatchResult {
            total: components.len(),
            success: success_count,
            errors: error_count,
            error_messages: errors,
            duration: start.elapsed(),
        })
    }
}

/// Result of batch submission
#[derive(Debug)]
pub struct BatchResult {
    pub total: usize,
    pub success: usize,
    pub errors: usize,
    pub error_messages: Vec<String>,
    pub duration: std::time::Duration,
}

/// Registry indexer for project scanning
pub struct RegistryIndexer {
    analyzer: MarineCodeAnalyzer,
}

impl RegistryIndexer {
    pub fn new(registry_url: &str) -> Result<Self> {
        Ok(Self {
            analyzer: MarineCodeAnalyzer::new(registry_url)?,
        })
    }

    /// Index a project directory
    pub fn index_project(&self, project_path: &Path) -> Result<IndexingStats> {
        let start = Instant::now();
        let mut all_components = Vec::new();
        let mut files_processed = 0;
        let mut files_skipped = 0;

        // Find all .rs files
        let rust_files = self.find_rust_files(project_path)?;

        for file_path in rust_files {
            match self.analyzer.index_file(&file_path, project_path) {
                Ok(components) => {
                    files_processed += 1;
                    all_components.extend(components);
                }
                Err(e) => {
                    eprintln!("Warning: Failed to index {}: {}", file_path.display(), e);
                    files_skipped += 1;
                }
            }
        }

        // Submit to registry
        let batch_result = self.analyzer.submit_batch(&all_components)?;

        Ok(IndexingStats {
            project_path: project_path.to_path_buf(),
            files_processed,
            files_skipped,
            functions_indexed: all_components.len(),
            duration: start.elapsed(),
            batch_result,
        })
    }

    /// Find all Rust files in project
    fn find_rust_files(&self, project_path: &Path) -> Result<Vec<PathBuf>> {
        let mut files = Vec::new();

        for entry in walkdir::WalkDir::new(project_path)
            .follow_links(false)
            .into_iter()
            .filter_entry(|e| {
                // Skip common ignore patterns
                let name = e.file_name().to_str().unwrap_or("");
                !matches!(name, "target" | "node_modules" | ".git" | "dist" | "build")
            })
            .filter_map(|e| e.ok())
        {
            if entry.file_type().is_file() {
                if let Some(ext) = entry.path().extension() {
                    if ext == "rs" {
                        files.push(entry.path().to_path_buf());
                    }
                }
            }
        }

        Ok(files)
    }
}

/// Statistics from indexing operation
#[derive(Debug)]
pub struct IndexingStats {
    pub project_path: PathBuf,
    pub files_processed: usize,
    pub files_skipped: usize,
    pub functions_indexed: usize,
    pub duration: std::time::Duration,
    pub batch_result: BatchResult,
}

impl IndexingStats {
    /// Print summary to stdout
    pub fn print_summary(&self) {
        println!("\n╔══════════════════════════════════════════════════════════╗");
        println!("║     SmartPastCode Registry Indexing Summary             ║");
        println!("╚══════════════════════════════════════════════════════════╝");
        println!();
        println!("Project: {}", self.project_path.display());
        println!();
        println!("Files Processed:      {}", self.files_processed);
        println!("Files Skipped:        {}", self.files_skipped);
        println!("Functions Indexed:    {}", self.functions_indexed);
        println!();
        println!("Registry Submission:");
        println!("  Total:              {}", self.batch_result.total);
        println!("  Success:            {}", self.batch_result.success);
        println!("  Errors:             {}", self.batch_result.errors);
        println!();
        println!("Performance:");
        println!("  Total Duration:     {:.2}s", self.duration.as_secs_f64());
        println!("  Indexing Speed:     {:.1} functions/sec",
                 self.functions_indexed as f64 / self.duration.as_secs_f64());

        if !self.batch_result.error_messages.is_empty() {
            println!();
            println!("Errors:");
            for (i, error) in self.batch_result.error_messages.iter().take(5).enumerate() {
                println!("  {}. {}", i + 1, error);
            }
            if self.batch_result.error_messages.len() > 5 {
                println!("  ... and {} more", self.batch_result.error_messages.len() - 5);
            }
        }

        println!();
    }
}
