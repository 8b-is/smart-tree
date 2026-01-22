//! AI Integration Installer - Unified setup for all AI platforms
//!
//! "One command to rule them all!" - The Cheet
//!
//! This module provides interactive and non-interactive installation
//! of Smart Tree's AI integrations: MCP servers, hooks, plugins, and configs.

use crate::claude_init::{ClaudeInit, McpInstaller};
use crate::cli::{AiTarget, InstallScope};
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
    pub cleanup_foreign: bool,
}

impl Default for InstallOptions {
    fn default() -> Self {
        Self {
            install_mcp: true,
            install_hooks: true,
            install_claude_md: true,
            create_settings: true,
            cleanup_foreign: true, // Clean by default - opinionated!
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
        println!(
            "\nThis will configure Smart Tree for {}.",
            self.target_name()
        );
        println!("Scope: {}\n", self.scope_description());

        // Show existing configuration status first
        let manager = ConfigManager::new(self.scope);
        let existing = manager.list_configs();

        println!("Current Status:");
        for config in &existing {
            let icon = if config.enabled { "✅" } else { "⬜" };
            println!("  {} {}", icon, config.name);
        }

        // Discover what can be installed/updated
        let available = self.discover_options();

        println!("\nActions:");
        println!("  [a] Install/Update ALL integrations (includes cleanup)");
        println!("  [c] Clean foreign MCPs/hooks only - remove tool sprawl");
        if available.install_mcp {
            let status = if existing.iter().any(|c| c.name.contains("MCP") && c.enabled) {
                "(update)"
            } else {
                "(install)"
            };
            println!(
                "  [1] MCP Server {} - Enable 30+ tools in your AI assistant",
                status
            );
        }
        if available.install_hooks {
            let status = if existing
                .iter()
                .any(|c| c.name.contains("Hooks") && c.enabled)
            {
                "(update)"
            } else {
                "(install)"
            };
            println!("  [2] Hooks {} - Automatic context on every prompt", status);
        }
        if available.install_claude_md {
            let status = if existing
                .iter()
                .any(|c| c.name.contains("CLAUDE.md") && c.enabled)
            {
                "(update)"
            } else {
                "(create)"
            };
            println!("  [3] CLAUDE.md {} - Project-specific AI guidance", status);
        }
        if available.create_settings {
            let status = if existing
                .iter()
                .any(|c| c.name.contains("Settings") && c.enabled)
            {
                "(update)"
            } else {
                "(create)"
            };
            println!("  [4] Settings {} - AI-optimized configuration", status);
        }
        println!("  [s] Show detailed status only");
        println!("  [q] Quit without changes");

        print!("\nChoice [a/1-4/s/q]: ");
        io::stdout().flush()?;

        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        let input = input.trim().to_lowercase();

        match input.as_str() {
            "q" | "quit" | "exit" => {
                println!("No changes made.");
                Ok(())
            }
            "s" | "status" => {
                manager.display_configs();
                Ok(())
            }
            "c" | "clean" | "cleanup" => {
                // Cleanup only, no installations
                let cleanup_only = InstallOptions {
                    install_mcp: false,
                    install_hooks: false,
                    install_claude_md: false,
                    create_settings: false,
                    cleanup_foreign: true,
                };
                self.execute_install(&cleanup_only)
            }
            "a" | "all" | "" => self.execute_install(&available),
            _ => {
                let options = self.parse_selection(&input, &available);
                self.execute_install(&options)
            }
        }
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
            cleanup_foreign: false,
        };

        for c in input.chars() {
            match c {
                '1' if available.install_mcp => options.install_mcp = true,
                '2' if available.install_hooks => options.install_hooks = true,
                '3' if available.install_claude_md => options.install_claude_md = true,
                '4' if available.create_settings => options.create_settings = true,
                'c' => options.cleanup_foreign = true,
                _ => {}
            }
        }

        options
    }

    /// Execute the installation with the given options
    fn execute_install(&self, options: &InstallOptions) -> Result<()> {
        let mut installed = Vec::new();
        let mut errors = Vec::new();

        // FIRST: Clean up foreign MCPs and hooks if requested
        // This runs before any installations to ensure a clean slate
        if options.cleanup_foreign {
            match self.cleanup_foreign_integrations() {
                Ok(count) if count > 0 => installed.push("Foreign integrations cleaned"),
                Ok(_) => {} // Nothing to clean
                Err(e) => errors.push(format!("Cleanup: {}", e)),
            }
        }

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
                // 1. Install to Claude Desktop config
                let installer = McpInstaller::new()?;
                let result = installer.install()?;
                if result.success {
                    println!(
                        "  ✅ {}",
                        result.message.lines().next().unwrap_or("MCP installed")
                    );
                } else {
                    anyhow::bail!("{}", result.message)
                }

                // 2. Also create/update project's .mcp.json so Claude Code can find it
                self.ensure_project_mcp_json()?;

                Ok(())
            }
            _ => {
                println!("  ℹ️  MCP not supported for {} yet", self.target_name());
                Ok(())
            }
        }
    }

    /// Ensure the project has a .mcp.json with st configured
    fn ensure_project_mcp_json(&self) -> Result<()> {
        let mcp_json_path = self.project_path.join(".mcp.json");

        // Default st MCP configuration
        let st_config = json!({
            "type": "stdio",
            "command": "st",
            "args": ["--mcp"],
            "env": {}
        });

        if mcp_json_path.exists() {
            // Read and update existing config
            let content = fs::read_to_string(&mcp_json_path).context("Failed to read .mcp.json")?;
            let mut config: Value =
                serde_json::from_str(&content).unwrap_or_else(|_| json!({"mcpServers": {}}));

            // Ensure mcpServers exists and has st
            if let Some(obj) = config.as_object_mut() {
                let servers = obj
                    .entry("mcpServers".to_string())
                    .or_insert_with(|| json!({}));
                if let Some(servers_obj) = servers.as_object_mut() {
                    if !servers_obj.contains_key("st") {
                        servers_obj.insert("st".to_string(), st_config);
                        fs::write(&mcp_json_path, serde_json::to_string_pretty(&config)?)?;
                        println!("  ✅ Added st to {}", mcp_json_path.display());
                    }
                }
            }
        } else {
            // Create new .mcp.json with st
            let config = json!({
                "mcpServers": {
                    "st": st_config
                }
            });
            fs::write(&mcp_json_path, serde_json::to_string_pretty(&config)?)?;
            println!(
                "  ✅ Created {} with st MCP server",
                mcp_json_path.display()
            );
        }

        Ok(())
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

    /// Get Claude-specific hooks (matches claude_init.rs format)
    fn get_claude_hooks(&self) -> Value {
        json!({
            "UserPromptSubmit": [{
                "matcher": "",
                "hooks": [{
                    "type": "command",
                    "command": "st -m quantum-semantic ."
                }]
            }],
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

        fs::write(
            &settings_file,
            serde_json::to_string_pretty(&final_settings)?,
        )?;
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

    /// Clean up foreign MCP integrations and invasive hooks
    /// Returns the number of items cleaned
    fn cleanup_foreign_integrations(&self) -> Result<usize> {
        let mut cleaned = 0;

        // Patterns that indicate foreign/unwanted integrations
        let foreign_patterns = [
            "claude-flow",
            "ruv-swarm",
            "flow-nexus",
            "hive-mind",
            "npx ", // External npm packages running on every command
            "swarm",
            "queen",
            "worker",
        ];

        // 1. Clean parent directory .mcp.json files (inherited MCPs!)
        // Walk up from project to root, cleaning any .mcp.json with foreign servers
        let mut current = self.project_path.clone();
        loop {
            let mcp_json = current.join(".mcp.json");
            if mcp_json.exists() && mcp_json != self.project_path.join(".mcp.json") {
                // Don't clean the project's own .mcp.json, just parents
                cleaned += self.clean_parent_mcp_json(&mcp_json, &foreign_patterns)?;
            }
            if let Some(parent) = current.parent() {
                if parent == current {
                    break; // Reached root
                }
                current = parent.to_path_buf();
            } else {
                break;
            }
        }

        // 2. Clean ~/.claude/.claude/settings.json (the nested one with enabledMcpjsonServers)
        let nested_settings = dirs::home_dir().map(|h| h.join(".claude/.claude/settings.json"));

        if let Some(path) = nested_settings {
            if path.exists() {
                cleaned += self.clean_settings_file(&path, &foreign_patterns)?;
            }
        }

        // 3. Clean ~/.claude/settings.json
        let user_settings = dirs::home_dir().map(|h| h.join(".claude/settings.json"));

        if let Some(path) = user_settings {
            if path.exists() {
                cleaned += self.clean_settings_file(&path, &foreign_patterns)?;
            }
        }

        // 4. Clean project-level .claude/settings.json if in project scope
        if matches!(self.scope, InstallScope::Project) {
            let project_settings = self.project_path.join(".claude/settings.json");
            if project_settings.exists() {
                cleaned += self.clean_settings_file(&project_settings, &foreign_patterns)?;
            }
        }

        if cleaned > 0 {
            println!("  🧹 Cleaned {} foreign integration(s)", cleaned);
        }

        Ok(cleaned)
    }

    /// Clean a parent .mcp.json file of foreign MCP servers
    fn clean_parent_mcp_json(&self, path: &std::path::Path, patterns: &[&str]) -> Result<usize> {
        let content = fs::read_to_string(path).context("Failed to read .mcp.json")?;

        // Handle empty or whitespace-only files
        if content.trim().is_empty() {
            // Delete the empty file as it's not useful
            let _ = fs::remove_file(path);
            return Ok(0);
        }

        let mut config: Value = match serde_json::from_str(&content) {
            Ok(v) => v,
            Err(_) => {
                // Invalid JSON - delete the malformed file
                let _ = fs::remove_file(path);
                return Ok(0);
            }
        };

        let mut cleaned = 0;

        if let Some(obj) = config.as_object_mut() {
            if let Some(servers) = obj.get_mut("mcpServers") {
                if let Some(servers_obj) = servers.as_object_mut() {
                    let server_names: Vec<String> = servers_obj.keys().cloned().collect();

                    for name in server_names {
                        // Check if server name or config matches foreign patterns
                        let config_str = servers_obj
                            .get(&name)
                            .map(|v| serde_json::to_string(v).unwrap_or_default())
                            .unwrap_or_default();

                        if patterns
                            .iter()
                            .any(|p| name.contains(p) || config_str.contains(p))
                        {
                            servers_obj.remove(&name);
                            cleaned += 1;
                            println!("    Removed MCP server '{}' from {}", name, path.display());
                        }
                    }
                }
            }
        }

        // Write back if we made changes
        if cleaned > 0 {
            fs::write(path, serde_json::to_string_pretty(&config)?)?;
        }

        Ok(cleaned)
    }

    /// Clean a specific settings file of foreign integrations
    fn clean_settings_file(&self, path: &std::path::Path, patterns: &[&str]) -> Result<usize> {
        let content = fs::read_to_string(path).context("Failed to read settings file")?;

        let mut config: Value =
            serde_json::from_str(&content).context("Failed to parse settings JSON")?;

        let mut cleaned = 0;

        // Remove enabledMcpjsonServers entirely or filter it
        if let Some(obj) = config.as_object_mut() {
            if obj.contains_key("enabledMcpjsonServers") {
                obj.remove("enabledMcpjsonServers");
                cleaned += 1;
                println!("    Removed enabledMcpjsonServers from {}", path.display());
            }

            // Clean hooks that match foreign patterns
            if let Some(hooks) = obj.get_mut("hooks") {
                if let Some(hooks_obj) = hooks.as_object_mut() {
                    let hook_types: Vec<String> = hooks_obj.keys().cloned().collect();

                    for hook_type in hook_types {
                        if let Some(hook_array) = hooks_obj.get_mut(&hook_type) {
                            if let Some(arr) = hook_array.as_array_mut() {
                                let original_len = arr.len();

                                // Filter out hooks with foreign patterns
                                arr.retain(|hook| {
                                    let hook_str = serde_json::to_string(hook).unwrap_or_default();
                                    !patterns.iter().any(|p| hook_str.contains(p))
                                });

                                let removed = original_len - arr.len();
                                if removed > 0 {
                                    cleaned += removed;
                                    println!(
                                        "    Removed {} foreign {} hook(s)",
                                        removed, hook_type
                                    );
                                }
                            }
                        }
                    }
                }
            }
        }

        // Write back if we made changes
        if cleaned > 0 {
            fs::write(path, serde_json::to_string_pretty(&config)?)?;
        }

        Ok(cleaned)
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

// =============================================================================
// Configuration Manager - View and manage existing AI integrations
// =============================================================================

/// Existing configuration status
#[derive(Debug)]
pub struct ConfigStatus {
    pub name: String,
    pub enabled: bool,
    pub path: Option<PathBuf>,
    pub details: String,
}

/// AI Configuration Manager - lists and manages existing configs
pub struct ConfigManager {
    scope: InstallScope,
}

impl ConfigManager {
    pub fn new(scope: InstallScope) -> Self {
        Self { scope }
    }

    /// Get all existing configurations
    pub fn list_configs(&self) -> Vec<ConfigStatus> {
        let mut configs = Vec::new();

        // Check MCP installation
        configs.push(self.check_mcp_status());

        // Check hooks
        configs.push(self.check_hooks_status());

        // Check settings
        configs.push(self.check_settings_status());

        // Check CLAUDE.md (project only)
        if matches!(self.scope, InstallScope::Project) {
            configs.push(self.check_claude_md_status());
        }

        configs
    }

    /// Display configurations in a nice format
    pub fn display_configs(&self) {
        let configs = self.list_configs();

        println!(
            "\n📋 AI Integration Status ({})",
            match self.scope {
                InstallScope::Project => "Project",
                InstallScope::User => "User",
            }
        );
        println!("{}", "─".repeat(50));

        for config in &configs {
            let status_icon = if config.enabled { "✅" } else { "❌" };
            println!("\n{} {}", status_icon, config.name);
            println!("   {}", config.details);
            if let Some(path) = &config.path {
                println!("   📁 {}", path.display());
            }
        }

        println!("\n{}", "─".repeat(50));
        println!("💡 Use 'st -i' to install/update integrations");
    }

    fn check_mcp_status(&self) -> ConfigStatus {
        let installer = McpInstaller::default();
        let installed = installer.is_installed().unwrap_or(false);
        let config_path = McpInstaller::get_claude_desktop_config_path();

        ConfigStatus {
            name: "MCP Server (Claude Desktop)".to_string(),
            enabled: installed,
            path: config_path,
            details: if installed {
                "Smart Tree MCP tools available in Claude Desktop".to_string()
            } else {
                "Not installed - run 'st -i' to enable 30+ AI tools".to_string()
            },
        }
    }

    fn check_hooks_status(&self) -> ConfigStatus {
        let hooks_dir = match self.scope {
            InstallScope::Project => std::env::current_dir().ok(),
            InstallScope::User => dirs::home_dir(),
        }
        .map(|p| p.join(".claude"));

        let hooks_file = hooks_dir.as_ref().map(|d| d.join("hooks.json"));
        let exists = hooks_file.as_ref().map(|p| p.exists()).unwrap_or(false);

        let details = if exists {
            if let Some(path) = &hooks_file {
                if let Ok(content) = fs::read_to_string(path) {
                    if let Ok(config) = serde_json::from_str::<Value>(&content) {
                        let hook_count = config.as_object().map(|o| o.len()).unwrap_or(0);
                        format!("{} hook(s) configured", hook_count)
                    } else {
                        "Configuration file exists but may be invalid".to_string()
                    }
                } else {
                    "Configuration file exists".to_string()
                }
            } else {
                "Hooks configured".to_string()
            }
        } else {
            "Not configured - automatic context on prompts".to_string()
        };

        ConfigStatus {
            name: "Claude Code Hooks".to_string(),
            enabled: exists,
            path: hooks_file,
            details,
        }
    }

    fn check_settings_status(&self) -> ConfigStatus {
        let settings_dir = match self.scope {
            InstallScope::Project => std::env::current_dir().ok(),
            InstallScope::User => dirs::home_dir(),
        }
        .map(|p| p.join(".claude"));

        let settings_file = settings_dir.as_ref().map(|d| d.join("settings.json"));
        let exists = settings_file.as_ref().map(|p| p.exists()).unwrap_or(false);

        let details = if exists {
            if let Some(path) = &settings_file {
                if let Ok(content) = fs::read_to_string(path) {
                    if let Ok(config) = serde_json::from_str::<Value>(&content) {
                        if let Some(st) = config.get("smart_tree") {
                            let version = st
                                .get("version")
                                .and_then(|v| v.as_str())
                                .unwrap_or("unknown");
                            format!("Smart Tree v{} settings", version)
                        } else {
                            "Settings file exists (no Smart Tree config)".to_string()
                        }
                    } else {
                        "Settings file exists".to_string()
                    }
                } else {
                    "Settings file exists".to_string()
                }
            } else {
                "Settings configured".to_string()
            }
        } else {
            "Not configured".to_string()
        };

        ConfigStatus {
            name: "Smart Tree Settings".to_string(),
            enabled: exists,
            path: settings_file,
            details,
        }
    }

    fn check_claude_md_status(&self) -> ConfigStatus {
        let claude_md = std::env::current_dir()
            .ok()
            .map(|p| p.join(".claude/CLAUDE.md"));

        let exists = claude_md.as_ref().map(|p| p.exists()).unwrap_or(false);

        ConfigStatus {
            name: "AI Guidance (CLAUDE.md)".to_string(),
            enabled: exists,
            path: claude_md,
            details: if exists {
                "Project-specific AI instructions available".to_string()
            } else {
                "Not created - helps AI understand your project".to_string()
            },
        }
    }
}

/// Show configuration status for CLI
pub fn show_ai_config_status(scope: InstallScope) {
    let manager = ConfigManager::new(scope);
    manager.display_configs();
}
