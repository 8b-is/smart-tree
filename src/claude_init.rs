//! Claude integration initializer for Smart Tree
//! Automatically sets up optimal .claude directory configuration for any project
//! Also handles MCP server auto-installation for Claude Desktop! 🚀

use anyhow::{Context, Result};
use chrono::Local;
use serde_json::{json, Value};
use std::fs;
use std::io::{self, Write as IoWrite};
use std::path::{Path, PathBuf};

use crate::scanner::{Scanner, ScannerConfig};
use crate::TreeStats;

/// Valid Claude Code hook event types
const VALID_HOOK_KEYS: &[&str] = &[
    "SessionStart",
    "UserPromptSubmit",
    "PreToolUse",
    "PermissionRequest",
    "PostToolUse",
    "PostToolUseFailure",
    "SubagentStart",
    "SubagentStop",
    "Stop",
    "PreCompact",
    "SessionEnd",
    "Notification",
    "Setup",
];

/// Ask user for confirmation before overwriting a file
fn confirm_overwrite(path: &Path) -> bool {
    print!("   ⚠️  {} exists. Overwrite? [y/N]: ", path.display());
    io::stdout().flush().unwrap();

    let mut input = String::new();
    if io::stdin().read_line(&mut input).is_ok() {
        let response = input.trim().to_lowercase();
        return response == "y" || response == "yes";
    }
    false
}

/// Validate that settings.json has correct hook format
/// Returns error description if invalid, None if valid
pub fn validate_settings(path: &Path) -> Result<Option<String>> {
    if !path.exists() {
        return Ok(None);
    }

    let content = fs::read_to_string(path)?;
    let parsed: Result<Value, _> = serde_json::from_str(&content);

    match parsed {
        Err(e) => Ok(Some(format!("Invalid JSON: {}", e))),
        Ok(json) => {
            if let Some(hooks) = json.get("hooks") {
                if let Some(obj) = hooks.as_object() {
                    for key in obj.keys() {
                        if !VALID_HOOK_KEYS.contains(&key.as_str()) {
                            return Ok(Some(format!(
                                "Invalid hook key '{}'. Valid: {}",
                                key,
                                VALID_HOOK_KEYS.join(", ")
                            )));
                        }
                    }
                }
            }
            Ok(None)
        }
    }
}

/// Project type detection for optimal hook configuration
#[derive(Debug, Clone)]
pub enum ProjectType {
    Rust,
    Python,
    JavaScript,
    TypeScript,
    Mixed,
    Unknown,
}

/// Claude integration initializer
pub struct ClaudeInit {
    project_path: PathBuf,
    project_type: ProjectType,
    stats: TreeStats,
}

impl ClaudeInit {
    /// Create new initializer for a project
    pub fn new(project_path: PathBuf) -> Result<Self> {
        // Scan project to understand structure
        let config = ScannerConfig {
            max_depth: 3,
            show_hidden: false,
            follow_symlinks: false,
            ..Default::default()
        };

        let scanner = Scanner::new(&project_path, config)?;
        let (nodes, stats) = scanner.scan()?;

        // Detect project type based on files
        let project_type = Self::detect_project_type(&nodes, &stats);

        Ok(Self {
            project_path,
            project_type,
            stats,
        })
    }

    /// Detect project type from file extensions
    fn detect_project_type(nodes: &[crate::FileNode], _stats: &TreeStats) -> ProjectType {
        let mut rust_score = 0;
        let mut python_score = 0;
        let mut js_score = 0;
        let mut ts_score = 0;

        // Check for key files
        for node in nodes {
            let path_str = node.path.to_string_lossy();

            // Project markers
            if path_str.contains("Cargo.toml") {
                rust_score += 100;
            }
            if path_str.contains("package.json") {
                js_score += 50;
                ts_score += 30;
            }
            if path_str.contains("pyproject.toml") || path_str.contains("requirements.txt") {
                python_score += 100;
            }
            if path_str.contains("tsconfig.json") {
                ts_score += 100;
            }

            // File extensions
            if path_str.ends_with(".rs") {
                rust_score += 1;
            }
            if path_str.ends_with(".py") {
                python_score += 1;
            }
            if path_str.ends_with(".js") || path_str.ends_with(".jsx") {
                js_score += 1;
            }
            if path_str.ends_with(".ts") || path_str.ends_with(".tsx") {
                ts_score += 1;
            }
        }

        // Determine primary type
        let max_score = rust_score.max(python_score).max(js_score).max(ts_score);

        if max_score == 0 {
            ProjectType::Unknown
        } else if rust_score == max_score {
            ProjectType::Rust
        } else if python_score == max_score {
            ProjectType::Python
        } else if ts_score == max_score {
            ProjectType::TypeScript
        } else if js_score == max_score {
            ProjectType::JavaScript
        } else {
            ProjectType::Mixed
        }
    }

    /// Smart setup - initializes if new, updates if exists
    pub fn setup(&self) -> Result<()> {
        let claude_dir = self.project_path.join(".claude");

        if claude_dir.exists() {
            // Update existing configuration
            self.update_existing(&claude_dir)
        } else {
            // Initialize new configuration
            self.init_new(&claude_dir)
        }
    }

    /// Initialize new Claude configuration
    fn init_new(&self, claude_dir: &Path) -> Result<()> {
        // Create .claude directory
        fs::create_dir_all(claude_dir).context("Failed to create .claude directory")?;

        // Generate settings.json (force=true for new projects)
        self.create_settings_json(claude_dir, true)?;

        // Generate CLAUDE.md (force=true for new projects)
        self.create_claude_md(claude_dir, true)?;

        println!(
            "✨ Claude integration initialized for {:?} project!",
            self.project_type
        );
        println!("📁 Created .claude/ directory with:");
        println!("   • settings.json - Smart hooks configured");
        println!("   • CLAUDE.md - Project-specific AI guidance");
        println!("\n💡 Tip: Run 'st --setup-claude' anytime to update");

        Ok(())
    }

    /// Update existing Claude configuration
    fn update_existing(&self, claude_dir: &Path) -> Result<()> {
        println!("🔄 Checking existing Claude integration...");

        let settings_path = claude_dir.join("settings.json");
        let claude_md_path = claude_dir.join("CLAUDE.md");

        let mut updated = false;

        // Validate existing settings if present
        if settings_path.exists() {
            if let Some(error) = validate_settings(&settings_path)? {
                println!("   ⚠️  settings.json has issues: {}", error);
                println!("   💡 Suggested fix:");
                self.show_suggested()?;
                return Ok(());
            }

            // Check if auto-configured (safe to update silently)
            let existing: Value = serde_json::from_str(&fs::read_to_string(&settings_path)?)?;
            let is_auto = existing
                .get("smart_tree")
                .and_then(|st| st.get("auto_configured"))
                .and_then(|v| v.as_bool())
                .unwrap_or(false);

            if is_auto {
                // Auto-configured: safe to update
                if self.create_settings_json(claude_dir, true)? {
                    println!("   ✅ Updated settings.json");
                    updated = true;
                }
            } else {
                // Manual config: ask first (force=false)
                println!("   ℹ️  settings.json has manual configuration");
                if self.create_settings_json(claude_dir, false)? {
                    println!("   ✅ Updated settings.json");
                    updated = true;
                } else {
                    println!("   ⏭️  Skipped settings.json");
                }
            }
        } else if self.create_settings_json(claude_dir, true)? {
            println!("   ✅ Created settings.json");
            updated = true;
        }

        // CLAUDE.md - ask before overwriting (force=false)
        if claude_md_path.exists() {
            if self.create_claude_md(claude_dir, false)? {
                println!("   ✅ Updated CLAUDE.md");
                updated = true;
            } else {
                println!("   ⏭️  Skipped CLAUDE.md");
            }
        } else if self.create_claude_md(claude_dir, true)? {
            println!("   ✅ Created CLAUDE.md");
            updated = true;
        }

        if updated {
            println!(
                "\n🎉 Claude integration updated for {:?} project!",
                self.project_type
            );
        } else {
            println!("\n✨ No changes made. Use --force to overwrite.");
        }

        Ok(())
    }

    /// Create settings.json with smart hook configuration
    /// If file exists, asks for confirmation unless force=true
    fn create_settings_json(&self, claude_dir: &Path, force: bool) -> Result<bool> {
        let settings_path = claude_dir.join("settings.json");

        // Check if file exists and ask for confirmation
        if settings_path.exists() && !force {
            if !confirm_overwrite(&settings_path) {
                return Ok(false);
            }
            // Backup existing file
            let backup = settings_path.with_extension("json.bak");
            fs::copy(&settings_path, &backup)?;
        }

        // Build hook configuration - NO automatic context dump on every prompt!
        // AI should request context via MCP tools when needed, not get flooded every message.
        // Only SessionStart/End for consciousness persistence, and targeted PreToolUse hooks.
        let hooks = match self.project_type {
            ProjectType::Rust => {
                json!({
                    "SessionStart": [{
                        "matcher": "",
                        "hooks": [{
                            "type": "command",
                            "command": "st --claude-restore"
                        }]
                    }],
                    "SessionEnd": [{
                        "matcher": "",
                        "hooks": [{
                            "type": "command",
                            "command": "st --claude-save"
                        }]
                    }],
                    "PreToolUse": [{
                        "matcher": "cargo (build|test|run)",
                        "hooks": [{
                            "type": "command",
                            "command": "st -m summary --depth 3 ."
                        }]
                    }]
                })
            }
            ProjectType::Python => {
                json!({
                    "SessionStart": [{
                        "matcher": "",
                        "hooks": [{
                            "type": "command",
                            "command": "st --claude-restore"
                        }]
                    }],
                    "SessionEnd": [{
                        "matcher": "",
                        "hooks": [{
                            "type": "command",
                            "command": "st --claude-save"
                        }]
                    }],
                    "PreToolUse": [{
                        "matcher": "pytest|python.*test",
                        "hooks": [{
                            "type": "command",
                            "command": "st -m summary --depth 3 ."
                        }]
                    }]
                })
            }
            ProjectType::JavaScript | ProjectType::TypeScript => {
                json!({
                    "SessionStart": [{
                        "matcher": "",
                        "hooks": [{
                            "type": "command",
                            "command": "st --claude-restore"
                        }]
                    }],
                    "SessionEnd": [{
                        "matcher": "",
                        "hooks": [{
                            "type": "command",
                            "command": "st --claude-save"
                        }]
                    }],
                    "PreToolUse": [{
                        "matcher": "npm (test|build|run)",
                        "hooks": [{
                            "type": "command",
                            "command": "st -m summary --depth 3 ."
                        }]
                    }]
                })
            }
            _ => {
                // Generic configuration - just consciousness persistence
                json!({
                    "SessionStart": [{
                        "matcher": "",
                        "hooks": [{
                            "type": "command",
                            "command": "st --claude-restore"
                        }]
                    }],
                    "SessionEnd": [{
                        "matcher": "",
                        "hooks": [{
                            "type": "command",
                            "command": "st --claude-save"
                        }]
                    }]
                })
            }
        };

        let settings = json!({
            "hooks": hooks,
            "smart_tree": {
                "version": env!("CARGO_PKG_VERSION"),
                "project_type": format!("{:?}", self.project_type),
                "auto_configured": true,
                "stats": {
                    "files": self.stats.total_files,
                    "directories": self.stats.total_dirs,
                    "size": self.stats.total_size
                }
            }
        });

        let content = serde_json::to_string_pretty(&settings)?;
        fs::write(&settings_path, &content)?;

        // Validate what we wrote
        if let Some(error) = validate_settings(&settings_path)? {
            // Revert from backup
            let backup = settings_path.with_extension("json.bak");
            if backup.exists() {
                fs::copy(&backup, &settings_path)?;
                fs::remove_file(&backup)?;
            }
            anyhow::bail!("Validation failed, reverted: {}", error);
        }

        Ok(true)
    }

    /// Create CLAUDE.md with project-specific guidance
    /// If file exists, asks for confirmation unless force=true
    fn create_claude_md(&self, claude_dir: &Path, force: bool) -> Result<bool> {
        let claude_md_path = claude_dir.join("CLAUDE.md");

        // Check if file exists and ask for confirmation
        if claude_md_path.exists() && !force && !confirm_overwrite(&claude_md_path) {
            return Ok(false);
        }

        let content = match self.project_type {
            ProjectType::Rust => {
                format!(
                    r#"# CLAUDE.md

This Rust project uses Smart Tree for optimal AI context management.

## Project Stats
- Files: {}
- Directories: {}
- Total size: {} bytes

## Essential Commands

```bash
# Build & Test
cargo build --release
cargo test -- --nocapture
cargo clippy -- -D warnings

# Smart Tree context
st -m context .          # Full context with git info
st -m quantum .           # Compressed for large contexts
st -m relations --focus main.rs  # Code relationships
```

## Key Patterns
- Always use `Result<T>` for error handling
- Prefer `&str` over `String` for function parameters
- Use `anyhow` for error context
- Run clippy before commits

## Smart Tree Integration
This project has hooks configured to automatically provide context.
The quantum-semantic mode is used for optimal token efficiency.
"#,
                    self.stats.total_files, self.stats.total_dirs, self.stats.total_size
                )
            }
            ProjectType::Python => {
                format!(
                    r#"# CLAUDE.md

This Python project uses Smart Tree for optimal AI context management.

## Project Stats
- Files: {}
- Directories: {}
- Total size: {} bytes

## Essential Commands

```bash
# Environment & Testing
uv sync                   # Install dependencies with uv
pytest -v                 # Run tests
ruff check .             # Lint code
mypy .                   # Type checking

# Smart Tree context
st -m context .          # Full context with git info
st -m quantum .          # Compressed for large contexts
```

## Key Patterns
- Use type hints for all functions
- Prefer uv over pip for package management
- Follow PEP 8 style guide
- Write docstrings for all public functions

## Smart Tree Integration
Hooks provide automatic context on prompt submission.
Test runs trigger summary of test directories.
"#,
                    self.stats.total_files, self.stats.total_dirs, self.stats.total_size
                )
            }
            ProjectType::TypeScript | ProjectType::JavaScript => {
                format!(
                    r#"# CLAUDE.md

This {0} project uses Smart Tree for optimal AI context management.

## Project Stats
- Files: {1}
- Directories: {2}
- Total size: {3} bytes

## Essential Commands

```bash
# Development
pnpm install             # Install dependencies
pnpm run dev            # Start dev server
pnpm test               # Run tests
pnpm build              # Production build

# Smart Tree context
st -m context .          # Full context with git info
st -m quantum .          # Compressed for large contexts
```

## Key Patterns
- Use pnpm for package management
- Implement proper TypeScript types
- Follow ESLint rules
- Component-based architecture

## Smart Tree Integration
Automatic context provision via hooks.
Node_modules excluded from summaries.
"#,
                    if matches!(self.project_type, ProjectType::TypeScript) {
                        "TypeScript"
                    } else {
                        "JavaScript"
                    },
                    self.stats.total_files,
                    self.stats.total_dirs,
                    self.stats.total_size
                )
            }
            _ => {
                format!(
                    r#"# CLAUDE.md

This project uses Smart Tree for optimal AI context management.

## Project Stats
- Files: {}
- Directories: {}
- Total size: {} bytes
- Type: {:?}

## Smart Tree Commands

```bash
st -m context .          # Full context with git info
st -m quantum .          # Compressed for large contexts
st -m summary .          # Human-readable summary
st -m quantum-semantic . # Maximum compression
```

## Smart Tree Integration
This project has been configured with automatic hooks that provide
context to Claude on every prompt. The hook mode is optimized based
on your project size.

Use `st --help` to explore more features!
"#,
                    self.stats.total_files,
                    self.stats.total_dirs,
                    self.stats.total_size,
                    self.project_type
                )
            }
        };

        fs::write(claude_md_path, content)?;

        Ok(true)
    }

    /// Show what settings would be generated without writing
    pub fn show_suggested(&self) -> Result<()> {
        println!(
            "📋 Suggested Claude integration for {:?} project:\n",
            self.project_type
        );

        // NO automatic UserPromptSubmit dumps - AI requests context via MCP tools when needed
        let hooks = match self.project_type {
            ProjectType::Rust => json!({
                "SessionStart": [{"matcher": "", "hooks": [{"type": "command", "command": "st --claude-restore"}]}],
                "SessionEnd": [{"matcher": "", "hooks": [{"type": "command", "command": "st --claude-save"}]}],
                "PreToolUse": [{"matcher": "cargo (build|test|run)", "hooks": [{"type": "command", "command": "st -m summary --depth 1 target/"}]}]
            }),
            ProjectType::Python => json!({
                "SessionStart": [{"matcher": "", "hooks": [{"type": "command", "command": "st --claude-restore"}]}],
                "SessionEnd": [{"matcher": "", "hooks": [{"type": "command", "command": "st --claude-save"}]}],
                "PreToolUse": [{"matcher": "pytest|python.*test", "hooks": [{"type": "command", "command": "st -m summary --depth 2 tests/"}]}]
            }),
            _ => json!({
                "SessionStart": [{"matcher": "", "hooks": [{"type": "command", "command": "st --claude-restore"}]}],
                "SessionEnd": [{"matcher": "", "hooks": [{"type": "command", "command": "st --claude-save"}]}]
            }),
        };

        let settings = json!({"hooks": hooks});
        println!("━━━ Add to .claude/settings.json ━━━");
        println!("{}\n", serde_json::to_string_pretty(&settings)?);

        println!("💡 Or run: st --setup-claude (will ask before overwriting)");
        Ok(())
    }
}

// =============================================================================
// MCP Server Auto-Installer - "One command, infinite context!" 🎯
// =============================================================================

/// Result of MCP installation attempt
#[derive(Debug)]
pub struct McpInstallResult {
    pub success: bool,
    pub config_path: PathBuf,
    pub backup_path: Option<PathBuf>,
    pub message: String,
    pub was_update: bool,
}

/// MCP Server installer for Claude Desktop
/// Automatically adds Smart Tree to claude_desktop_config.json
pub struct McpInstaller {
    /// Path to the st binary (defaults to current exe or 'st' in PATH)
    st_binary_path: PathBuf,
    /// Custom config path override (for testing)
    custom_config_path: Option<PathBuf>,
}

impl McpInstaller {
    /// Create new installer with auto-detected st binary path
    pub fn new() -> Result<Self> {
        // Try to find the st binary
        let st_binary_path = Self::find_st_binary()?;

        Ok(Self {
            st_binary_path,
            custom_config_path: None,
        })
    }

    /// Create installer with custom binary path
    pub fn with_binary_path(path: PathBuf) -> Self {
        Self {
            st_binary_path: path,
            custom_config_path: None,
        }
    }

    /// Set custom config path (for testing)
    pub fn with_config_path(mut self, path: PathBuf) -> Self {
        self.custom_config_path = Some(path);
        self
    }

    /// Find the st binary in common locations
    fn find_st_binary() -> Result<PathBuf> {
        // First, try the current executable
        if let Ok(exe) = std::env::current_exe() {
            if exe.file_name().map(|n| n == "st").unwrap_or(false) {
                return Ok(exe);
            }
        }

        // Try common install locations
        let candidates = vec![
            // User's cargo bin
            dirs::home_dir().map(|h| h.join(".cargo/bin/st")),
            // /usr/local/bin (common for manual installs)
            Some(PathBuf::from("/usr/local/bin/st")),
            // homebrew (macOS)
            Some(PathBuf::from("/opt/homebrew/bin/st")),
        ];

        for candidate in candidates.into_iter().flatten() {
            if candidate.exists() {
                return Ok(candidate);
            }
        }

        // Fall back to just "st" and hope it's in PATH
        Ok(PathBuf::from("st"))
    }

    /// Get MCP target config paths for all known local desktop AI setups
    /// Returns a list of (AgentName, ConfigPath)
    pub fn get_all_target_configs() -> Vec<(&'static str, PathBuf)> {
        let mut paths = Vec::new();

        // 1. Claude Desktop
        #[cfg(target_os = "macos")]
        if let Some(h) = dirs::home_dir() {
            paths.push(("Claude Desktop", h.join("Library/Application Support/Claude/claude_desktop_config.json")));
        }
        #[cfg(target_os = "windows")]
        if let Some(c) = dirs::config_dir() {
            paths.push(("Claude Desktop", c.join("Claude/claude_desktop_config.json")));
        }
        #[cfg(target_os = "linux")]
        if let Some(c) = dirs::config_dir() {
            paths.push(("Claude Desktop", c.join("Claude/claude_desktop_config.json")));
        }

        // 2. Gemini / Antigravity
        if let Some(h) = dirs::home_dir() {
            paths.push(("Antigravity", h.join(".gemini/antigravity/mcp_config.json")));
            paths.push(("Gemini", h.join(".gemini/mcp_config.json")));
        }

        paths
    }

    /// Install Smart Tree MCP server to all detected Desktop configs
    pub fn install_all(&self) -> Result<Vec<McpInstallResult>> {
        let targets = if let Some(custom) = &self.custom_config_path {
            vec![("Custom", custom.clone())]
        } else {
            Self::get_all_target_configs()
        };

        if targets.is_empty() {
            anyhow::bail!("No supported agent configurations found for this OS.");
        }

        let mut results = Vec::new();

        for (agent_name, config_path) in targets {
            // Ensure parent directory exists
            if let Some(parent) = config_path.parent() {
                if fs::create_dir_all(parent).is_err() {
                    continue; // Skip if we don't have permissions or bad path
                }
            }

            // Read existing config or create new
            let (mut config, was_update) = if config_path.exists() {
                if let Ok(content) = fs::read_to_string(&config_path) {
                    if let Ok(json_val) = serde_json::from_str::<Value>(&content) {
                        (json_val, true)
                    } else {
                        (json!({}), false)
                    }
                } else {
                    (json!({}), false)
                }
            } else {
                (json!({}), false)
            };

            // Create backup if updating existing config
            let backup_path = if was_update {
                let backup = config_path.with_extension(format!(
                    "json.backup.{}",
                    Local::now().format("%Y%m%d_%H%M%S")
                ));
                let _ = fs::copy(&config_path, &backup);
                Some(backup)
            } else {
                None
            };

            // Build the Smart Tree MCP server config
            let st_config = json!({
                "command": self.st_binary_path.to_string_lossy(),
                "args": ["--mcp"],
                "env": {}
            });

            // Update or create mcpServers section
            if config.get("mcpServers").is_none() {
                config["mcpServers"] = json!({});
            }

            // Check if already installed
            let already_installed = config["mcpServers"].get("smart-tree").is_some();

            // Add/update Smart Tree entry
            config["mcpServers"]["smart-tree"] = st_config;

            // Write updated config with pretty formatting
            if let Ok(formatted) = serde_json::to_string_pretty(&config) {
                if fs::write(&config_path, formatted).is_err() {
                    continue; // Skip on write failure
                }
            }

            let message = if already_installed {
                format!(
                    "✨ Updated Smart Tree MCP server in {}!\n\
                       📁 Config: {}\n\
                       🔧 Binary: {}",
                    agent_name,
                    config_path.display(),
                    self.st_binary_path.display()
                )
            } else {
                format!(
                    "🎉 Smart Tree MCP server installed to {}!\n\
                       📁 Config: {}\n\
                       🔧 Binary: {}",
                    agent_name,
                    config_path.display(),
                    self.st_binary_path.display()
                )
            };

            results.push(McpInstallResult {
                success: true,
                config_path,
                backup_path,
                message,
                was_update: already_installed,
            });
        }

        Ok(results)
    }

    /// Uninstall Smart Tree from Desktop configs
    pub fn uninstall_all(&self) -> Result<Vec<McpInstallResult>> {
        let targets = if let Some(custom) = &self.custom_config_path {
            vec![("Custom", custom.clone())]
        } else {
            Self::get_all_target_configs()
        };

        let mut results = Vec::new();

        for (agent_name, config_path) in targets {
            if !config_path.exists() {
                continue;
            }

            let content = match fs::read_to_string(&config_path) {
                Ok(c) => c,
                Err(_) => continue,
            };

            let mut config: Value = match serde_json::from_str(&content) {
                Ok(c) => c,
                Err(_) => continue,
            };

            let was_removed = if let Some(servers) = config.get_mut("mcpServers").and_then(|s| s.as_object_mut()) {
                servers.remove("smart-tree").is_some()
            } else {
                false
            };

            if was_removed {
                let backup = config_path.with_extension(format!(
                    "json.backup.{}",
                    Local::now().format("%Y%m%d_%H%M%S")
                ));
                let _ = fs::copy(&config_path, &backup);

                if let Ok(formatted) = serde_json::to_string_pretty(&config) {
                    let _ = fs::write(&config_path, formatted);
                }

                results.push(McpInstallResult {
                    success: true,
                    config_path: config_path.clone(),
                    backup_path: Some(backup),
                    message: format!(
                        "🗑️ Removed Smart Tree MCP server from {}.\n\
                           📁 Config: {}",
                        agent_name,
                        config_path.display()
                    ),
                    was_update: true,
                });
            }
        }

        Ok(results)
    }

    /// Check if Smart Tree is installed in any target config
    pub fn is_installed(&self) -> Result<bool> {
        let targets = if let Some(custom) = &self.custom_config_path {
            vec![("Custom", custom.clone())]
        } else {
            Self::get_all_target_configs()
        };

        for (_, path) in targets {
            if path.exists() {
                if let Ok(content) = fs::read_to_string(&path) {
                    if let Ok(config) = serde_json::from_str::<Value>(&content) {
                        if config["mcpServers"].get("smart-tree").is_some() {
                            return Ok(true);
                        }
                    }
                }
            }
        }

        Ok(false)
    }

    /// Get status information about current installation across agents
    pub fn status(&self) -> Result<Value> {
        let targets = Self::get_all_target_configs();
        let is_installed = self.is_installed().unwrap_or(false);
        
        let paths: Vec<String> = targets.into_iter()
            .map(|(_, p)| p.display().to_string())
            .collect();

        Ok(json!({
            "installed": is_installed,
            "config_paths": paths,
            "binary_path": self.st_binary_path.display().to_string(),
            "binary_exists": self.st_binary_path.exists(),
        }))
    }
}

impl Default for McpInstaller {
    fn default() -> Self {
        Self::new().unwrap_or_else(|_| Self {
            st_binary_path: PathBuf::from("st"),
            custom_config_path: None,
        })
    }
}

/// Quick installation function for CLI use
/// Returns a human-readable result message
pub fn install_mcp_to_desktop() -> Result<String> {
    let installer = McpInstaller::new()?;
    let results = installer.install_all()?;
    let msg = results.into_iter()
        .filter(|r| r.success)
        .map(|r| r.message)
        .collect::<Vec<_>>()
        .join("\n\n");
    if msg.is_empty() {
        Ok("Nothing to install or update.".to_string())
    } else {
        Ok(msg)
    }
}

/// Quick uninstall function for CLI use
pub fn uninstall_mcp_from_desktop() -> Result<String> {
    let installer = McpInstaller::new()?;
    let results = installer.uninstall_all()?;
    let msg = results.into_iter()
        .filter(|r| r.success)
        .map(|r| r.message)
        .collect::<Vec<_>>()
        .join("\n\n");
    if msg.is_empty() {
        Ok("No installations found to remove.".to_string())
    } else {
        Ok(msg)
    }
}

/// Check MCP installation status
pub fn check_mcp_installation_status() -> Result<String> {
    let installer = McpInstaller::new()?;
    let status = installer.status()?;

    let installed = status["installed"].as_bool().unwrap_or(false);
    let config_paths = status["config_paths"].as_array();

    if installed {
        Ok(format!(
            "✅ Smart Tree MCP server is installed!\n\
             📁 Configs: {:?}\n\
             🔧 Binary: {}",
            config_paths,
            status["binary_path"].as_str().unwrap_or("st")
        ))
    } else {
        Ok(format!(
            "❌ Smart Tree MCP server is NOT installed.\n\
             📁 Expected configs: {:?}\n\
             💡 Run 'st --mcp-install' to install",
            config_paths
        ))
    }
}
