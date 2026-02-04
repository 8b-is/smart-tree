//! Smart Tree Configuration System
//!
//! Unified config for API keys, model preferences, and daemon settings.
//! Config file: ~/.st/config.toml

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

/// Main configuration structure
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct StConfig {
    /// LLM provider API keys
    #[serde(default)]
    pub api_keys: ApiKeys,

    /// Model preferences and aliases
    #[serde(default)]
    pub models: ModelConfig,

    /// Daemon settings
    #[serde(default)]
    pub daemon: DaemonConfig,

    /// Safety/trust settings
    #[serde(default)]
    pub safety: SafetyConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ApiKeys {
    pub anthropic: Option<String>,
    pub openai: Option<String>,
    pub google: Option<String>,
    pub openrouter: Option<String>,
    pub grok: Option<String>,
    /// Custom providers: name -> api_key
    #[serde(default)]
    pub custom: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelConfig {
    /// Default model for chat
    pub default_model: String,
    /// Model aliases: short_name -> full_model_id
    #[serde(default)]
    pub aliases: HashMap<String, String>,
    /// Blocked models (safety)
    #[serde(default)]
    pub blocked: Vec<String>,
}

impl Default for ModelConfig {
    fn default() -> Self {
        let mut aliases = HashMap::new();
        aliases.insert("claude".into(), "claude-3-5-sonnet-20241022".into());
        aliases.insert("gpt4".into(), "gpt-4o".into());
        aliases.insert("gemini".into(), "gemini-2.0-flash".into());

        Self {
            default_model: "claude-3-5-sonnet-20241022".into(),
            aliases,
            blocked: vec!["greatcoderMDK".into()], // Known bad actor
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DaemonConfig {
    pub port: u16,
    pub auto_start: bool,
    /// Allow external connections (not just localhost)
    pub allow_external: bool,
}

impl Default for DaemonConfig {
    fn default() -> Self {
        Self {
            port: 8420,
            auto_start: false,
            allow_external: false,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SafetyConfig {
    /// Enable The Custodian monitoring
    pub custodian_enabled: bool,
    /// Log all LLM requests for transparency
    pub transparency_logging: bool,
    /// Model safety scores (model_id -> score 0-10)
    #[serde(default)]
    pub model_scores: HashMap<String, u8>,
}

impl Default for SafetyConfig {
    fn default() -> Self {
        let mut scores = HashMap::new();
        scores.insert("claude-3-5-sonnet-20241022".into(), 10);
        scores.insert("claude-3-opus-20240229".into(), 10);
        scores.insert("gpt-4o".into(), 9);
        scores.insert("gpt-4-turbo".into(), 9);
        scores.insert("gemini-2.0-flash".into(), 9);
        scores.insert("greatcoderMDK".into(), 2); // Suspicious

        Self {
            custodian_enabled: true,
            transparency_logging: true,
            model_scores: scores,
        }
    }
}

impl StConfig {
    /// Get config file path
    pub fn config_path() -> Result<PathBuf> {
        let st_dir = dirs::home_dir()
            .context("Could not find home directory")?
            .join(".st");
        fs::create_dir_all(&st_dir)?;
        Ok(st_dir.join("config.toml"))
    }

    /// Load config from file, or create default
    pub fn load() -> Result<Self> {
        let path = Self::config_path()?;

        if path.exists() {
            let content = fs::read_to_string(&path)
                .with_context(|| format!("Failed to read {}", path.display()))?;
            let config: StConfig = toml::from_str(&content)
                .with_context(|| format!("Failed to parse {}", path.display()))?;
            Ok(config)
        } else {
            // Create default config
            let config = Self::default();
            config.save()?;
            Ok(config)
        }
    }

    /// Save config to file
    pub fn save(&self) -> Result<()> {
        let path = Self::config_path()?;
        let content = toml::to_string_pretty(self)?;
        fs::write(&path, content)?;
        Ok(())
    }

    /// Get API key for a provider (checks config then env)
    pub fn get_api_key(&self, provider: &str) -> Option<String> {
        // Check config first
        let from_config = match provider.to_lowercase().as_str() {
            "anthropic" | "claude" => self.api_keys.anthropic.clone(),
            "openai" | "gpt" => self.api_keys.openai.clone(),
            "google" | "gemini" => self.api_keys.google.clone(),
            "openrouter" => self.api_keys.openrouter.clone(),
            "grok" | "xai" => self.api_keys.grok.clone(),
            other => self.api_keys.custom.get(other).cloned(),
        };

        // Fall back to env var
        from_config.or_else(|| {
            let env_var = match provider.to_lowercase().as_str() {
                "anthropic" | "claude" => "ANTHROPIC_API_KEY",
                "openai" | "gpt" => "OPENAI_API_KEY",
                "google" | "gemini" => "GOOGLE_API_KEY",
                "openrouter" => "OPENROUTER_API_KEY",
                "grok" | "xai" => "XAI_API_KEY",
                _ => return None,
            };
            std::env::var(env_var).ok()
        })
    }

    /// Check if a model is blocked
    pub fn is_model_blocked(&self, model: &str) -> bool {
        self.models.blocked.iter().any(|b| model.contains(b))
    }

    /// Get safety score for a model (0-10)
    pub fn get_model_score(&self, model: &str) -> u8 {
        self.safety.model_scores.get(model).copied().unwrap_or(5) // Default: neutral
    }
}
