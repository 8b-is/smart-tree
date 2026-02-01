//! Environment Templates - Pre-configured development environments
//!
//! Templates define what a user's space looks like:
//! - Tools and languages installed
//! - Shell configuration
//! - Default environment variables
//! - Container image (if using podman)

use super::space::IsolationLevel;
use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

/// A development environment template
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Template {
    /// Template name (e.g., "rust-dev", "node-dev", "minimal")
    pub name: String,

    /// Human-readable description
    pub description: String,

    /// Base container image (for podman isolation)
    pub image: Option<String>,

    /// Default isolation level
    #[serde(default)]
    pub default_isolation: IsolationLevel,

    /// Shell to use
    #[serde(default = "default_shell")]
    pub shell: String,

    /// Environment variables
    #[serde(default)]
    pub env: Vec<(String, String)>,

    /// Packages/tools to install
    #[serde(default)]
    pub packages: Vec<String>,

    /// Shell init commands (run on space start)
    #[serde(default)]
    pub init_commands: Vec<String>,

    /// Files to copy into the space
    #[serde(default)]
    pub files: HashMap<String, String>,

    /// Author of the template
    pub author: Option<String>,

    /// Tags for discoverability
    #[serde(default)]
    pub tags: Vec<String>,
}

fn default_shell() -> String {
    "/bin/bash".to_string()
}

impl Default for Template {
    fn default() -> Self {
        Template {
            name: "minimal".to_string(),
            description: "Minimal development environment".to_string(),
            image: None,
            default_isolation: IsolationLevel::None,
            shell: default_shell(),
            env: Vec::new(),
            packages: Vec::new(),
            init_commands: Vec::new(),
            files: HashMap::new(),
            author: None,
            tags: vec!["minimal".to_string()],
        }
    }
}

impl Template {
    /// Create a new template
    pub fn new(name: &str, description: &str) -> Self {
        Template {
            name: name.to_string(),
            description: description.to_string(),
            ..Default::default()
        }
    }

    /// Rust development template
    pub fn rust_dev() -> Self {
        Template {
            name: "rust-dev".to_string(),
            description: "Rust development with cargo, clippy, rustfmt".to_string(),
            image: Some("rust:latest".to_string()),
            default_isolation: IsolationLevel::None,
            shell: "/bin/bash".to_string(),
            env: vec![
                ("CARGO_HOME".to_string(), "/usr/local/cargo".to_string()),
                ("RUSTUP_HOME".to_string(), "/usr/local/rustup".to_string()),
            ],
            packages: vec![
                "rust-analyzer".to_string(),
                "cargo-watch".to_string(),
                "cargo-edit".to_string(),
            ],
            init_commands: vec!["rustup component add clippy rustfmt".to_string()],
            files: HashMap::new(),
            author: Some("smart-tree".to_string()),
            tags: vec!["rust".to_string(), "systems".to_string()],
        }
    }

    /// Node.js development template
    pub fn node_dev() -> Self {
        Template {
            name: "node-dev".to_string(),
            description: "Node.js/TypeScript development with pnpm".to_string(),
            image: Some("node:20".to_string()),
            default_isolation: IsolationLevel::None,
            shell: "/bin/bash".to_string(),
            env: vec![("PNPM_HOME".to_string(), "/usr/local/pnpm".to_string())],
            packages: vec![
                "pnpm".to_string(),
                "typescript".to_string(),
                "tsx".to_string(),
            ],
            init_commands: vec!["corepack enable".to_string()],
            files: HashMap::new(),
            author: Some("smart-tree".to_string()),
            tags: vec!["node".to_string(), "typescript".to_string(), "web".to_string()],
        }
    }

    /// Python development template
    pub fn python_dev() -> Self {
        Template {
            name: "python-dev".to_string(),
            description: "Python development with uv and ruff".to_string(),
            image: Some("python:3.12".to_string()),
            default_isolation: IsolationLevel::None,
            shell: "/bin/bash".to_string(),
            env: vec![
                ("UV_SYSTEM_PYTHON".to_string(), "1".to_string()),
                ("PYTHONDONTWRITEBYTECODE".to_string(), "1".to_string()),
            ],
            packages: vec!["uv".to_string(), "ruff".to_string(), "mypy".to_string()],
            init_commands: vec![],
            files: HashMap::new(),
            author: Some("smart-tree".to_string()),
            tags: vec!["python".to_string(), "ml".to_string(), "data".to_string()],
        }
    }

    /// AI assistant template (for Claude, etc.)
    pub fn ai_assistant() -> Self {
        Template {
            name: "ai-assistant".to_string(),
            description: "Environment for AI code assistants".to_string(),
            image: None,
            default_isolation: IsolationLevel::Namespace, // Light isolation for safety
            shell: "/bin/bash".to_string(),
            env: vec![
                ("AI_MODE".to_string(), "1".to_string()),
                ("TERM".to_string(), "xterm-256color".to_string()),
            ],
            packages: vec![], // AI brings its own tools via MCP
            init_commands: vec![],
            files: HashMap::new(),
            author: Some("smart-tree".to_string()),
            tags: vec!["ai".to_string(), "assistant".to_string(), "mcp".to_string()],
        }
    }

    /// Load a template from a TOML file
    pub fn load(path: &PathBuf) -> Result<Self> {
        let content = std::fs::read_to_string(path)
            .with_context(|| format!("Failed to read template: {:?}", path))?;
        toml::from_str(&content).with_context(|| format!("Failed to parse template: {:?}", path))
    }

    /// Save template to a TOML file
    pub fn save(&self, path: &PathBuf) -> Result<()> {
        let content = toml::to_string_pretty(self)?;
        std::fs::write(path, content)
            .with_context(|| format!("Failed to write template: {:?}", path))?;
        Ok(())
    }
}

/// Template registry - manages available templates
#[derive(Debug, Default)]
pub struct TemplateRegistry {
    templates: HashMap<String, Template>,
    search_paths: Vec<PathBuf>,
}

impl TemplateRegistry {
    /// Create a new registry with default templates
    pub fn new() -> Self {
        let mut registry = TemplateRegistry {
            templates: HashMap::new(),
            search_paths: vec![],
        };

        // Register built-in templates
        registry.register(Template::default());
        registry.register(Template::rust_dev());
        registry.register(Template::node_dev());
        registry.register(Template::python_dev());
        registry.register(Template::ai_assistant());

        registry
    }

    /// Add a search path for template files
    pub fn add_search_path(&mut self, path: PathBuf) {
        self.search_paths.push(path);
    }

    /// Register a template
    pub fn register(&mut self, template: Template) {
        self.templates.insert(template.name.clone(), template);
    }

    /// Get a template by name
    pub fn get(&self, name: &str) -> Option<&Template> {
        self.templates.get(name)
    }

    /// List all available templates
    pub fn list(&self) -> Vec<&Template> {
        self.templates.values().collect()
    }

    /// Search templates by tag
    pub fn search_by_tag(&self, tag: &str) -> Vec<&Template> {
        self.templates
            .values()
            .filter(|t| t.tags.iter().any(|t| t.contains(tag)))
            .collect()
    }

    /// Load templates from all search paths
    pub fn load_all(&mut self) -> Result<usize> {
        let mut count = 0;
        for path in self.search_paths.clone() {
            if path.is_dir() {
                for entry in std::fs::read_dir(&path)? {
                    let entry = entry?;
                    let file_path = entry.path();
                    if file_path.extension().is_some_and(|e| e == "toml") {
                        if let Ok(template) = Template::load(&file_path) {
                            self.register(template);
                            count += 1;
                        }
                    }
                }
            }
        }
        Ok(count)
    }

    /// Create a template from the current environment
    pub fn capture_current(name: &str, description: &str) -> Result<Template> {
        let mut template = Template::new(name, description);

        // Capture current shell
        if let Ok(shell) = std::env::var("SHELL") {
            template.shell = shell;
        }

        // Capture relevant environment variables
        let capture_vars = ["CARGO_HOME", "RUSTUP_HOME", "PNPM_HOME", "GOPATH", "PYENV_ROOT"];
        for var in capture_vars {
            if let Ok(value) = std::env::var(var) {
                template.env.push((var.to_string(), value));
            }
        }

        Ok(template)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_template() {
        let template = Template::default();
        assert_eq!(template.name, "minimal");
        assert_eq!(template.shell, "/bin/bash");
    }

    #[test]
    fn test_rust_dev_template() {
        let template = Template::rust_dev();
        assert_eq!(template.name, "rust-dev");
        assert!(template.tags.contains(&"rust".to_string()));
    }

    #[test]
    fn test_registry() {
        let registry = TemplateRegistry::new();
        assert!(registry.get("minimal").is_some());
        assert!(registry.get("rust-dev").is_some());
        assert!(registry.get("node-dev").is_some());
    }

    #[test]
    fn test_search_by_tag() {
        let registry = TemplateRegistry::new();
        let web = registry.search_by_tag("web");
        assert!(!web.is_empty());
    }
}
