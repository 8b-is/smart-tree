//! Claude integration initializer for Smart Tree
//! Automatically sets up optimal .claude directory configuration for any project

use anyhow::{Context, Result};
use serde_json::{json, Value};
use std::fs;
use std::path::{Path, PathBuf};

use crate::scanner::{Scanner, ScannerConfig};
use crate::TreeStats;

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
    fn detect_project_type(nodes: &[crate::FileNode], stats: &TreeStats) -> ProjectType {
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

        // Generate settings.json with optimal hooks
        self.create_settings_json(claude_dir)?;

        // Generate CLAUDE.md with project-specific guidance
        self.create_claude_md(claude_dir)?;

        println!(
            "✨ Claude integration initialized for {:?} project!",
            self.project_type
        );
        println!("📁 Created .claude/ directory with:");
        println!("   • settings.json - Smart hooks configured");
        println!("   • CLAUDE.md - Project-specific AI guidance");
        println!("\n💡 Tip: Run 'st --setup-claude' anytime to update with latest optimizations");

        Ok(())
    }

    /// Update existing Claude configuration
    fn update_existing(&self, claude_dir: &Path) -> Result<()> {
        println!("🔄 Updating existing Claude integration...");

        // Check what exists
        let settings_path = claude_dir.join("settings.json");
        let claude_md_path = claude_dir.join("CLAUDE.md");

        let mut updated = false;

        // Update or create settings.json
        if settings_path.exists() {
            // Read existing to check if it's auto-configured
            let existing: Value = serde_json::from_str(&fs::read_to_string(&settings_path)?)?;

            // Only update if it was auto-configured or user confirms
            if existing
                .get("smart_tree")
                .and_then(|st| st.get("auto_configured"))
                .and_then(|v| v.as_bool())
                .unwrap_or(false)
            {
                self.create_settings_json(claude_dir)?;
                println!("   ✅ Updated settings.json with latest hooks");
                updated = true;
            } else {
                println!("   ⚠️  settings.json exists (manual config) - skipping update");
                println!("      To force update, delete .claude/settings.json and run again");
            }
        } else {
            self.create_settings_json(claude_dir)?;
            println!("   ✅ Created missing settings.json");
            updated = true;
        }

        // Always update CLAUDE.md with fresh stats
        self.create_claude_md(claude_dir)?;
        if claude_md_path.exists() {
            println!("   ✅ Updated CLAUDE.md with current project stats");
        } else {
            println!("   ✅ Created missing CLAUDE.md");
        }
        updated = true;

        if updated {
            println!(
                "\n🎉 Claude integration updated for {:?} project!",
                self.project_type
            );
            println!(
                "   Files: {} | Dirs: {} | Size: {} bytes",
                self.stats.total_files, self.stats.total_dirs, self.stats.total_size
            );
        } else {
            println!("\n✨ Claude integration is up to date!");
        }

        Ok(())
    }

    /// Create settings.json with smart hook configuration
    fn create_settings_json(&self, claude_dir: &Path) -> Result<()> {
        let settings_path = claude_dir.join("settings.json");

        // Choose hook mode based on project size and type
        let hook_mode = if self.stats.total_files > 1000 {
            "quantum-semantic" // Ultra compression for large projects
        } else if self.stats.total_files > 100 {
            "quantum" // Good compression for medium projects
        } else {
            "context" // Rich context for small projects
        };

        // Build hook configuration
        let hooks = match self.project_type {
            ProjectType::Rust => {
                json!({
                    "UserPromptSubmit": [{
                        "matcher": "",
                        "hooks": [{
                            "type": "command",
                            "command": format!("st -m {} .", hook_mode)
                        }]
                    }],
                    "ToolCall": [{
                        "matcher": "cargo (build|test|run)",
                        "hooks": [{
                            "type": "command",
                            "command": "st -m summary --depth 1 target/"
                        }]
                    }]
                })
            }
            ProjectType::Python => {
                json!({
                    "UserPromptSubmit": [{
                        "matcher": "",
                        "hooks": [{
                            "type": "command",
                            "command": format!("st -m {} .", hook_mode)
                        }]
                    }],
                    "ToolCall": [{
                        "matcher": "pytest|python.*test",
                        "hooks": [{
                            "type": "command",
                            "command": "st -m summary --depth 2 tests/"
                        }]
                    }]
                })
            }
            ProjectType::JavaScript | ProjectType::TypeScript => {
                json!({
                    "UserPromptSubmit": [{
                        "matcher": "",
                        "hooks": [{
                            "type": "command",
                            "command": format!("st -m {} .", hook_mode)
                        }]
                    }],
                    "ToolCall": [{
                        "matcher": "npm (test|build|run)",
                        "hooks": [{
                            "type": "command",
                            "command": "st -m summary --no-emoji node_modules --exclude"
                        }]
                    }]
                })
            }
            _ => {
                // Generic configuration
                json!({
                    "UserPromptSubmit": [{
                        "matcher": "",
                        "hooks": [{
                            "type": "command",
                            "command": format!("st -m {} .", hook_mode)
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

        fs::write(settings_path, serde_json::to_string_pretty(&settings)?)?;

        Ok(())
    }

    /// Create CLAUDE.md with project-specific guidance
    fn create_claude_md(&self, claude_dir: &Path) -> Result<()> {
        let claude_md_path = claude_dir.join("CLAUDE.md");

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

        Ok(())
    }
}
