// Feature Flags - Enterprise-friendly configuration for Smart Tree
// "Your tool, your rules!" - Hue

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::fs;

/// Feature flags for controlling Smart Tree capabilities
/// Organizations can disable features via config file or environment variables
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeatureFlags {
    // Core features
    pub enable_mcp_server: bool,
    pub enable_classic_tree: bool,
    pub enable_formatters: bool,

    // AI/ML features
    pub enable_ai_modes: bool,
    pub enable_consciousness: bool,
    pub enable_memory_manager: bool,
    pub enable_context_absorption: bool,
    pub enable_smart_search: bool,

    // Data collection features
    pub enable_activity_logging: bool,
    pub enable_telemetry: bool,
    pub enable_file_watching: bool,
    pub enable_auto_context: bool,

    // Interactive features
    pub enable_tui: bool,
    pub enable_hooks: bool,
    pub enable_tips: bool,

    // Advanced features
    pub enable_quantum_modes: bool,
    pub enable_wave_signatures: bool,
    pub enable_mega_sessions: bool,
    pub enable_q8_caster: bool,

    // MCP-specific tools (granular control)
    pub mcp_tools: McpToolFlags,

    // Privacy settings
    pub privacy_mode: bool,
    pub disable_external_connections: bool,
    pub disable_home_directory_access: bool,

    // Compliance settings
    pub compliance_mode: Option<ComplianceMode>,
    pub allowed_paths: Vec<String>,
    pub blocked_paths: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpToolFlags {
    pub enable_find: bool,
    pub enable_search: bool,
    pub enable_analyze: bool,
    pub enable_edit: bool,
    pub enable_context: bool,
    pub enable_memory: bool,
    pub enable_unified_watcher: bool,
    pub enable_hooks_management: bool,
    pub enable_sse: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ComplianceMode {
    None,
    Enterprise, // Disables most AI features, logging
    Government, // Maximum restrictions
    Healthcare, // HIPAA compliance
    Education,  // FERPA compliance
    Financial,  // SOC2/PCI compliance
}

impl Default for FeatureFlags {
    fn default() -> Self {
        Self {
            // Core features - always enabled by default
            enable_mcp_server: true,
            enable_classic_tree: true,
            enable_formatters: true,

            // AI/ML features - can be disabled
            enable_ai_modes: true,
            enable_consciousness: true,
            enable_memory_manager: true,
            enable_context_absorption: true,
            enable_smart_search: true,

            // Data collection - respect privacy
            enable_activity_logging: false, // Opt-in
            enable_telemetry: false,        // Opt-in
            enable_file_watching: true,
            enable_auto_context: true,

            // Interactive features
            enable_tui: true,
            enable_hooks: true,
            enable_tips: true,

            // Advanced features
            enable_quantum_modes: true,
            enable_wave_signatures: true,
            enable_mega_sessions: true,
            enable_q8_caster: true,

            // MCP tools - all enabled by default
            mcp_tools: McpToolFlags::default(),

            // Privacy settings
            privacy_mode: false,
            disable_external_connections: false,
            disable_home_directory_access: false,

            // Compliance
            compliance_mode: None,
            allowed_paths: vec![],
            blocked_paths: vec![],
        }
    }
}

impl Default for McpToolFlags {
    fn default() -> Self {
        Self {
            enable_find: true,
            enable_search: true,
            enable_analyze: true,
            enable_edit: true,
            enable_context: true,
            enable_memory: true,
            enable_unified_watcher: true,
            enable_hooks_management: true,
            enable_sse: true,
        }
    }
}

impl FeatureFlags {
    /// Load feature flags from multiple sources (in priority order):
    /// 1. Environment variables (highest priority)
    /// 2. Local config file (.st/features.toml)
    /// 3. System config (/etc/smart-tree/features.toml)
    /// 4. Default values (lowest priority)
    pub fn load() -> Result<Self> {
        let mut flags = Self::default();

        // Try system config
        if let Ok(system_flags) = Self::load_from_file("/etc/smart-tree/features.toml") {
            flags = flags.merge(system_flags);
        }

        // Try user config
        if let Some(home) = dirs::home_dir() {
            let user_config = home.join(".st").join("features.toml");
            if let Ok(user_flags) = Self::load_from_file(user_config) {
                flags = flags.merge(user_flags);
            }
        }

        // Try local config (project-specific)
        if let Ok(local_flags) = Self::load_from_file(".st/features.toml") {
            flags = flags.merge(local_flags);
        }

        // Apply environment variable overrides
        flags = flags.apply_env_overrides();

        // Apply compliance mode if set
        if let Some(mode) = flags.compliance_mode.clone() {
            flags = flags.apply_compliance_mode(&mode);
        }

        Ok(flags)
    }

    fn load_from_file<P: AsRef<std::path::Path>>(path: P) -> Result<Self> {
        let content = fs::read_to_string(path)?;
        Ok(toml::from_str(&content)?)
    }

    /// Merge with another set of flags (other takes priority)
    fn merge(self, other: Self) -> Self {
        // This is simplified - in production you'd merge field by field
        other
    }

    /// Apply environment variable overrides
    fn apply_env_overrides(mut self) -> Self {
        use std::env;

        // Check for specific feature flags
        if env::var("ST_DISABLE_MCP").is_ok() {
            self.enable_mcp_server = false;
        }

        if env::var("ST_DISABLE_AI").is_ok() {
            self.enable_ai_modes = false;
            self.enable_consciousness = false;
            self.enable_memory_manager = false;
        }

        if env::var("ST_DISABLE_LOGGING").is_ok() {
            self.enable_activity_logging = false;
        }

        if env::var("ST_DISABLE_WATCHING").is_ok() {
            self.enable_file_watching = false;
            self.enable_context_absorption = false;
            self.mcp_tools.enable_unified_watcher = false;
        }

        if env::var("ST_PRIVACY_MODE").is_ok() {
            self.privacy_mode = true;
            self.enable_telemetry = false;
            self.enable_activity_logging = false;
            self.disable_external_connections = true;
        }

        // Compliance mode from environment
        if let Ok(mode) = env::var("ST_COMPLIANCE_MODE") {
            self.compliance_mode = match mode.to_lowercase().as_str() {
                "enterprise" => Some(ComplianceMode::Enterprise),
                "government" => Some(ComplianceMode::Government),
                "healthcare" => Some(ComplianceMode::Healthcare),
                "education" => Some(ComplianceMode::Education),
                "financial" => Some(ComplianceMode::Financial),
                _ => None,
            };
        }

        self
    }

    /// Apply compliance mode restrictions
    fn apply_compliance_mode(mut self, mode: &ComplianceMode) -> Self {
        match mode {
            ComplianceMode::Government => {
                // Maximum restrictions for government use
                self.enable_consciousness = false;
                self.enable_memory_manager = false;
                self.enable_context_absorption = false;
                self.enable_activity_logging = false;
                self.enable_telemetry = false;
                self.enable_file_watching = false;
                self.enable_auto_context = false;
                self.enable_hooks = false;
                self.disable_external_connections = true;
                self.disable_home_directory_access = true;
                self.privacy_mode = true;

                // Disable most MCP tools
                self.mcp_tools.enable_edit = false;
                self.mcp_tools.enable_memory = false;
                self.mcp_tools.enable_unified_watcher = false;
                self.mcp_tools.enable_hooks_management = false;
            }
            ComplianceMode::Enterprise => {
                // Moderate restrictions for enterprise
                self.enable_consciousness = false;
                self.enable_memory_manager = false;
                self.enable_activity_logging = false;
                self.enable_telemetry = false;
                self.privacy_mode = true;

                // Disable some MCP tools
                self.mcp_tools.enable_unified_watcher = false;
                self.mcp_tools.enable_hooks_management = false;
            }
            ComplianceMode::Healthcare => {
                // HIPAA compliance
                self.enable_activity_logging = false;
                self.enable_telemetry = false;
                self.enable_context_absorption = false;
                self.privacy_mode = true;
                self.disable_home_directory_access = true;
            }
            ComplianceMode::Education => {
                // FERPA compliance
                self.enable_activity_logging = false;
                self.enable_telemetry = false;
                self.privacy_mode = true;
            }
            ComplianceMode::Financial => {
                // SOC2/PCI compliance
                self.enable_activity_logging = true; // Required for audit
                self.enable_telemetry = false;
                self.privacy_mode = true;
                self.enable_context_absorption = false;
            }
            ComplianceMode::None => {}
        }

        self
    }

    /// Check if a feature is enabled
    pub fn is_enabled(&self, feature: &str) -> bool {
        match feature {
            "mcp" => self.enable_mcp_server,
            "ai" => self.enable_ai_modes,
            "consciousness" => self.enable_consciousness,
            "memory" => self.enable_memory_manager,
            "absorption" => self.enable_context_absorption,
            "logging" => self.enable_activity_logging,
            "watching" => self.enable_file_watching,
            "hooks" => self.enable_hooks,
            "tui" => self.enable_tui,
            _ => true, // Unknown features default to enabled
        }
    }

    /// Get filtered MCP tools based on flags
    pub fn get_enabled_mcp_tools(&self) -> Vec<String> {
        let mut tools = Vec::new();

        if self.mcp_tools.enable_find {
            tools.push("find".to_string());
        }
        if self.mcp_tools.enable_search {
            tools.push("search".to_string());
        }
        if self.mcp_tools.enable_analyze {
            tools.push("analyze".to_string());
        }
        if self.mcp_tools.enable_edit {
            tools.push("edit".to_string());
        }
        if self.mcp_tools.enable_context {
            tools.push("context".to_string());
        }
        if self.mcp_tools.enable_memory {
            tools.push("memory".to_string());
        }
        if self.mcp_tools.enable_unified_watcher {
            tools.push("unified_watcher".to_string());
        }
        if self.mcp_tools.enable_hooks_management {
            tools.push("hooks".to_string());
        }
        if self.mcp_tools.enable_sse {
            tools.push("sse".to_string());
        }

        tools
    }

    /// Generate a features report
    pub fn generate_report(&self) -> String {
        let mut report = String::from("Smart Tree Feature Configuration\n");
        report.push_str("================================\n\n");

        if let Some(ref mode) = self.compliance_mode {
            report.push_str(&format!("Compliance Mode: {:?}\n\n", mode));
        }

        report.push_str("Core Features:\n");
        report.push_str(&format!("  MCP Server: {}\n", self.enable_mcp_server));
        report.push_str(&format!("  Classic Tree: {}\n", self.enable_classic_tree));
        report.push_str(&format!("  Formatters: {}\n\n", self.enable_formatters));

        report.push_str("AI/ML Features:\n");
        report.push_str(&format!("  AI Modes: {}\n", self.enable_ai_modes));
        report.push_str(&format!("  Consciousness: {}\n", self.enable_consciousness));
        report.push_str(&format!(
            "  Memory Manager: {}\n",
            self.enable_memory_manager
        ));
        report.push_str(&format!(
            "  Context Absorption: {}\n",
            self.enable_context_absorption
        ));
        report.push_str(&format!("  Smart Search: {}\n\n", self.enable_smart_search));

        report.push_str("Privacy Settings:\n");
        report.push_str(&format!("  Privacy Mode: {}\n", self.privacy_mode));
        report.push_str(&format!(
            "  Activity Logging: {}\n",
            self.enable_activity_logging
        ));
        report.push_str(&format!("  Telemetry: {}\n", self.enable_telemetry));
        report.push_str(&format!(
            "  External Connections: {}\n",
            !self.disable_external_connections
        ));
        report.push_str(&format!(
            "  Home Directory Access: {}\n",
            !self.disable_home_directory_access
        ));

        report
    }
}

/// Global feature flags instance
static FEATURES: once_cell::sync::Lazy<FeatureFlags> =
    once_cell::sync::Lazy::new(|| FeatureFlags::load().unwrap_or_default());

/// Get the global feature flags
pub fn features() -> &'static FeatureFlags {
    &FEATURES
}

/// Check if a feature is enabled (convenience function)
pub fn is_enabled(feature: &str) -> bool {
    features().is_enabled(feature)
}
