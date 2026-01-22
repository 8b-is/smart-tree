//! AI Integration Installer - Unified setup for all AI platforms
//!
//! "One command to rule them all!" - The Cheet
//!
//! This module provides interactive and non-interactive installation
//! of Smart Tree's AI integrations: MCP servers, hooks, plugins, and configs.

use crate::cli::{AiTarget, InstallScope};
use crate::claude_init::{ClaudeInit, McpInstaller};
use anyhow::{Context, Result};
use serde_json::{json, Value};
use std::fs;
use std::io::{self, Write};
use std::path::PathBuf;

/// AI Integration Installer - handles setup for all AI platforms
pub struct AiInstaller {
    /// Installation scope (project-local or user-wide)
    scope: InstallScope,
    /// Target AI platform
    target: AiTarget,
    /// Whether to run in interactive mode
    interactive: bool,
    /// Project path (for project-scoped installations)
    project_path: PathBuf,
}

/// Installation options discovered during interactive mode
#[derive(Debug, Clone)]
pub struct InstallOptions {
    pub install_mcp: bool,
    pub install_hooks: bool,
    pub install_claude_md: bool,
    pub create_settings: bool,
}

impl Default for InstallOptions {
    fn default() -> Self {
        Self {
            install_mcp: true,
            install_hooks: true,
            install_claude_md: true,
            create_settings: true,
        }
    }
}

impl AiInstaller {
    /// Create a new AI installer
    pub fn new(scope: InstallScope, target: AiTarget, interactive: bool) -> Result<Self> {
        let project_path = std::env::current_dir().context("Failed to get current directory")?;
        Ok(Self {
            scope,
            target,
            interactive,
            project_path,
        })
    }

    /// Run the installation process
    pub fn install(&self) -> Result<()> {
        println!("\n{}", self.get_header());

        if self.interactive {
            self.run_interactive()
        } else {
            self.run_non_interactive()
        }
    }

    /// Get a colorful header based on target
    fn get_header(&self) -> String {
        match self.target {
            AiTarget::Claude => "🤖 Smart Tree AI Integration - Claude Setup".to_string(),
            AiTarget::Chatgpt => "🤖 Smart Tree AI Integration - ChatGPT Setup".to_string(),
            AiTarget::Gemini => "🤖 Smart Tree AI Integration - Gemini Setup".to_string(),
            AiTarget::Universal => "🤖 Smart Tree AI Integration - Universal Setup".to_string(),
        }
    }

    /// Run interactive installation with user prompts
    fn run_interactive(&self) -> Result<()> {
        println!("\nThis will configure Smart Tree for {}.", self.target_name());
        println!("Scope: {}\n", self.scope_description());

        // Discover what can be installed
        let available = self.discover_options();

        // Show available options
        println!("Available integrations:");
        if available.install_mcp {
            println!("  [1] MCP Server - Enable 30+ tools in your AI assistant");
        }
        if available.install_hooks {
            println!("  [2] Hooks - Automatic context on every prompt");
        }
        if available.install_claude_md {
            println!("  [3] CLAUDE.md - Project-specific AI guidance");
        }
        if available.create_settings {
            println!("  [4] Settings - AI-optimized configuration");
        }

        println!("\nPress Enter to install all, or type numbers to select (e.g., '1,3'): ");
        io::stdout().flush()?;

        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        let input = input.trim();

        let options = if input.is_empty() {
            available
        } else {
            self.parse_selection(input, &available)
        };

        self.execute_install(&options)
    }

    /// Run non-interactive installation with defaults
    fn run_non_interactive(&self) -> Result<()> {
        let options = InstallOptions::default();
        self.execute_install(&options)
    }

    /// Discover what installation options are available
    fn discover_options(&self) -> InstallOptions {
        let mut options = InstallOptions::default();

        match self.scope {
            InstallScope::Project => {
                // Project-level installations
                options.install_claude_md = true;
                options.create_settings = true;
                options.install_hooks = true;

                // MCP is user-level only for Claude Desktop
                options.install_mcp = matches!(self.target, AiTarget::Claude | AiTarget::Universal);
            }
            InstallScope::User => {
                // User-level installations
                options.install_mcp = matches!(self.target, AiTarget::Claude | AiTarget::Universal);
                options.install_hooks = true;
                options.install_claude_md = false; // No project to add CLAUDE.md to
                options.create_settings = true;
            }
        }

        options
    }

    /// Parse user selection
    fn parse_selection(&self, input: &str, available: &InstallOptions) -> InstallOptions {
        let mut options = InstallOptions {
            install_mcp: false,
            install_hooks: false,
            install_claude_md: false,
            create_settings: false,
        };

        for c in input.chars() {
            match c {
                '1' if available.install_mcp => options.install_mcp = true,
                '2' if available.install_hooks => options.install_hooks = true,
                '3' if available.install_claude_md => options.install_claude_md = true,
                '4' if available.create_settings => options.create_settings = true,
                _ => {}
            }
        }

        options
    }

    /// Execute the installation with the given options
    fn execute_install(&self, options: &InstallOptions) -> Result<()> {
        let mut installed = Vec::new();
        let mut errors = Vec::new();

        // Install MCP server
        if options.install_mcp {
            match self.install_mcp() {
                Ok(_) => installed.push("MCP Server"),
                Err(e) => errors.push(format!("MCP: {}", e)),
            }
        }

        // Install hooks
        if options.install_hooks {
            match self.install_hooks() {
                Ok(_) => installed.push("Hooks"),
                Err(e) => errors.push(format!("Hooks: {}", e)),
            }
        }

        // Create CLAUDE.md (or equivalent for other AIs)
        if options.install_claude_md {
            match self.create_ai_guidance() {
                Ok(_) => installed.push("AI Guidance File"),
                Err(e) => errors.push(format!("AI Guidance: {}", e)),
            }
        }

        // Create settings
        if options.create_settings {
            match self.create_settings() {
                Ok(_) => installed.push("Settings"),
                Err(e) => errors.push(format!("Settings: {}", e)),
            }
        }

        // Summary
        println!("\n📋 Installation Summary:");
        if !installed.is_empty() {
            println!("  ✅ Installed: {}", installed.join(", "));
        }
        if !errors.is_empty() {
            println!("  ❌ Errors:");
            for error in &errors {
                println!("     • {}", error);
            }
        }

        if errors.is_empty() {
            println!("\n🎉 Smart Tree AI integration complete!");
            self.show_next_steps();
            Ok(())
        } else if !installed.is_empty() {
            println!("\n⚠️  Some components installed with errors");
            self.show_next_steps();
            Ok(())
        } else {
            anyhow::bail!("Installation failed: {}", errors.join("; "))
        }
    }

    /// Install MCP server
    fn install_mcp(&self) -> Result<()> {
        match self.target {
            AiTarget::Claude | AiTarget::Universal => {
                let installer = McpInstaller::new()?;
                let result = installer.install()?;
                if result.success {
                    println!("  ✅ {}", result.message.lines().next().unwrap_or("MCP installed"));
                    Ok(())
                } else {
                    anyhow::bail!("{}", result.message)
                }
            }
            _ => {
                println!("  ℹ️  MCP not supported for {} yet", self.target_name());
                Ok(())
            }
        }
    }

    /// Install hooks
    fn install_hooks(&self) -> Result<()> {
        let hooks_dir = match self.scope {
            InstallScope::Project => self.project_path.join(".claude"),
            InstallScope::User => dirs::home_dir()
                .ok_or_else(|| anyhow::anyhow!("Could not find home directory"))?
                .join(".claude"),
        };

        fs::create_dir_all(&hooks_dir)?;

        let hooks_config = match self.target {
            AiTarget::Claude => self.get_claude_hooks(),
            AiTarget::Chatgpt => self.get_generic_hooks("chatgpt"),
            AiTarget::Gemini => self.get_generic_hooks("gemini"),
            AiTarget::Universal => self.get_generic_hooks("universal"),
        };

        let hooks_file = hooks_dir.join("hooks.json");
        fs::write(&hooks_file, serde_json::to_string_pretty(&hooks_config)?)?;
        println!("  ✅ Hooks configured at {}", hooks_file.display());
        Ok(())
    }

    /// Get Claude-specific hooks
    fn get_claude_hooks(&self) -> Value {
        json!({
            "UserPromptSubmit": {
                "command": "st --claude-user-prompt-submit",
                "enabled": true,
                "description": "Provides intelligent context based on user prompts"
            },
            "SessionStart": {
                "command": "st --claude-restore",
                "enabled": true,
                "description": "Restores previous session consciousness"
            }
        })
    }

    /// Get generic hooks for other AI platforms
    fn get_generic_hooks(&self, platform: &str) -> Value {
        json!({
            "context_provider": {
                "command": format!("st -m context --depth 3 ."),
                "platform": platform,
                "description": "Provides project context on demand"
            }
        })
    }

    /// Create AI guidance file (CLAUDE.md or equivalent)
    fn create_ai_guidance(&self) -> Result<()> {
        if matches!(self.scope, InstallScope::User) {
            println!("  ℹ️  AI guidance file is project-specific, skipping for user scope");
            return Ok(());
        }

        let init = ClaudeInit::new(self.project_path.clone())?;
        init.setup()?;
        Ok(())
    }

    /// Create settings file
    fn create_settings(&self) -> Result<()> {
        let settings_dir = match self.scope {
            InstallScope::Project => self.project_path.join(".claude"),
            InstallScope::User => dirs::home_dir()
                .ok_or_else(|| anyhow::anyhow!("Could not find home directory"))?
                .join(".claude"),
        };

        fs::create_dir_all(&settings_dir)?;

        let settings = json!({
            "smart_tree": {
                "version": env!("CARGO_PKG_VERSION"),
                "target": self.target_name(),
                "scope": match self.scope {
                    InstallScope::Project => "project",
                    InstallScope::User => "user",
                },
                "auto_configured": true,
                "features": {
                    "context_on_prompt": true,
                    "session_persistence": true,
                    "mcp_integration": matches!(self.target, AiTarget::Claude | AiTarget::Universal)
                }
            }
        });

        let settings_file = settings_dir.join("settings.json");

        // Merge with existing if present
        let final_settings = if settings_file.exists() {
            let existing: Value = serde_json::from_str(&fs::read_to_string(&settings_file)?)?;
            self.merge_settings(existing, settings)
        } else {
            settings
        };

        fs::write(&settings_file, serde_json::to_string_pretty(&final_settings)?)?;
        println!("  ✅ Settings saved to {}", settings_file.display());
        Ok(())
    }

    /// Merge existing settings with new ones
    fn merge_settings(&self, existing: Value, new: Value) -> Value {
        let mut result = existing;
        if let (Some(existing_obj), Some(new_obj)) = (result.as_object_mut(), new.as_object()) {
            for (key, value) in new_obj {
                existing_obj.insert(key.clone(), value.clone());
            }
        }
        result
    }

    /// Get human-readable target name
    fn target_name(&self) -> &'static str {
        match self.target {
            AiTarget::Claude => "Claude",
            AiTarget::Chatgpt => "ChatGPT",
            AiTarget::Gemini => "Gemini",
            AiTarget::Universal => "Universal AI",
        }
    }

    /// Get scope description
    fn scope_description(&self) -> &'static str {
        match self.scope {
            InstallScope::Project => "Project-local (.claude/ in current directory)",
            InstallScope::User => "User-wide (~/.claude/ or ~/.config/)",
        }
    }

    /// Show next steps after installation
    fn show_next_steps(&self) {
        println!("\n📚 Next Steps:");

        match self.target {
            AiTarget::Claude => {
                println!("  1. Restart Claude Desktop to load MCP tools");
                println!("  2. Try: 'st -m context .' to see project context");
                println!("  3. Use '/hooks' in Claude Code to manage hooks");
            }
            AiTarget::Chatgpt | AiTarget::Gemini => {
                println!("  1. Run 'st -m context .' and paste the output");
                println!("  2. The AI will understand your project structure");
            }
            AiTarget::Universal => {
                println!("  1. Use 'st -m ai' for AI-optimized output");
                println!("  2. Use 'st -m quantum' for compressed context");
                println!("  3. MCP integration available for Claude Desktop");
            }
        }

        println!("\n💡 Pro tip: Run 'st --help' to explore all features!");
    }
}

/// Quick installation function for CLI use
pub fn run_ai_install(scope: InstallScope, target: AiTarget, interactive: bool) -> Result<()> {
    let installer = AiInstaller::new(scope, target, interactive)?;
    installer.install()
}
