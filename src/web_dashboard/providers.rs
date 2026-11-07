//! Universal AI Providers, OAuth Hub & Grandma's 1-Click Website Builder
//!
//! Combines the best of `vscode-universal-chat-provider` (CLIProxyAPI OAuth,
//! supplier headers, live `/v1/models` upstream discovery) and `opencode`
//! (`Auth.Info` oauth/api store, auto/code callback flows, provider methods)
//! with **Smart Tree's Sovereign Local Models as Priority #1**.
//!
//! Core rule:
//! - **Local providers** (`st-sovereign`, `ollama`, `lmstudio`, `llamacpp`, `vllm`, `openapi`)
//!   are Priority #1 and enabled by default.
//! - **External cloud providers** (`openrouter`, `claude`, `openai`, `google`, `grok`,
//!   `github-copilot`, `kimi`, `devin`, `meta`, `zai`) are **OFF by default** until the
//!   user signs in via OAuth or saves an API key (and can be toggled OFF at any time).

use super::SharedState;
use crate::mem8::persona::LifePersona;
use crate::proxy::oauth;
use crate::proxy::token_store::{self, StoredToken};
use crate::proxy::{LlmMessage, LlmProvider, LlmRequest, LlmRole};
use anyhow::Result;
use axum::{
    extract::{Path, Query, State},
    http::{header, StatusCode},
    response::{Html, IntoResponse},
    Json,
};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, HashMap};
use std::path::PathBuf;
use std::sync::{Arc, OnceLock};
use std::time::Duration;
use tokio::sync::RwLock;

/// Supplier headers adapted from `vscode-universal-chat-provider` and `opencode`.
pub fn supplier_headers_for_url(base_url: &str) -> Vec<(&'static str, &'static str)> {
    let lower = base_url.to_ascii_lowercase();
    if lower.contains("openrouter.ai") {
        vec![
            ("HTTP-Referer", "https://github.com/8b-is/smart-tree"),
            ("X-Title", "Smart Tree"),
            ("x-session-id", "st-universal-session"),
        ]
    } else if lower.contains("opencode.ai") {
        vec![("x-opencode-session", "st-universal-session")]
    } else if lower.contains("anthropic.com") {
        vec![(
            "anthropic-beta",
            "claude-code-20250219,interleaved-thinking-2025-05-14,fine-grained-tool-streaming-2025-05-14",
        )]
    } else {
        Vec::new()
    }
}

/// Auth mode for a provider (inspired by `opencode`'s `Auth.Info` + `vscode-universal-chat-provider`).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum ProviderAuthKind {
    #[default]
    None,
    LocalBuiltIn,
    LocalServer,
    OAuth,
    ApiKey,
    CliProxyApi,
    OpenCodeImported,
}

/// Persisted state for a single provider in `~/.st/providers.json`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderEntryConfig {
    pub id: String,
    /// Strict gate: Cloud providers are OFF (`false`) unless explicitly enabled!
    pub enabled: bool,
    pub auth_kind: ProviderAuthKind,
    #[serde(default)]
    pub base_url: String,
    #[serde(default)]
    pub api_key: Option<String>,
    #[serde(default)]
    pub account_label: Option<String>,
    #[serde(default)]
    pub default_model: Option<String>,
    #[serde(default)]
    pub custom_models: Vec<String>,
}

/// Global provider registry persisted at `~/.st/providers.json`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProvidersRegistryFile {
    /// Active primary provider ID (defaults to `"st-sovereign"` — our own local models!)
    pub active_provider: String,
    /// Active primary model ID (defaults to `"in-binary:st-deepseek-pqm-4gaia"`)
    pub active_model: String,
    /// Optional CLIProxyAPI management URL (default `http://127.0.0.1:8317`)
    #[serde(default = "default_cliproxy_url")]
    pub cliproxy_url: String,
    #[serde(default)]
    pub cliproxy_management_key: Option<String>,
    pub providers: BTreeMap<String, ProviderEntryConfig>,
}

fn default_cliproxy_url() -> String {
    "http://127.0.0.1:8317".to_string()
}

impl Default for ProvidersRegistryFile {
    fn default() -> Self {
        let mut providers = BTreeMap::new();

        // === TIER 1: OUR OWN LOCAL MODELS (PRIORITY #1 — ON BY DEFAULT) ===
        providers.insert(
            "st-sovereign".to_string(),
            ProviderEntryConfig {
                id: "st-sovereign".to_string(),
                enabled: true,
                auth_kind: ProviderAuthKind::LocalBuiltIn,
                base_url: "in-binary://smart-tree".to_string(),
                api_key: None,
                account_label: Some("Built-in 4gaia PQM & .m8 Souls".to_string()),
                default_model: Some("in-binary:st-deepseek-pqm-4gaia".to_string()),
                custom_models: Vec::new(),
            },
        );
        providers.insert(
            "ollama".to_string(),
            ProviderEntryConfig {
                id: "ollama".to_string(),
                enabled: true,
                auth_kind: ProviderAuthKind::LocalServer,
                base_url: "http://localhost:11434".to_string(),
                api_key: None,
                account_label: Some("Local & MemNet Ollama".to_string()),
                default_model: Some("st-deepseek-pqm:latest".to_string()),
                custom_models: Vec::new(),
            },
        );
        providers.insert(
            "lmstudio".to_string(),
            ProviderEntryConfig {
                id: "lmstudio".to_string(),
                enabled: true,
                auth_kind: ProviderAuthKind::LocalServer,
                base_url: "http://127.0.0.1:1234/v1".to_string(),
                api_key: None,
                account_label: Some("LM Studio Local (:1234)".to_string()),
                default_model: None,
                custom_models: Vec::new(),
            },
        );
        providers.insert(
            "llamacpp".to_string(),
            ProviderEntryConfig {
                id: "llamacpp".to_string(),
                enabled: true,
                auth_kind: ProviderAuthKind::LocalServer,
                base_url: "http://127.0.0.1:8080/v1".to_string(),
                api_key: None,
                account_label: Some("llama.cpp Server (:8080)".to_string()),
                default_model: None,
                custom_models: Vec::new(),
            },
        );
        providers.insert(
            "vllm".to_string(),
            ProviderEntryConfig {
                id: "vllm".to_string(),
                enabled: true,
                auth_kind: ProviderAuthKind::LocalServer,
                base_url: "http://127.0.0.1:8000/v1".to_string(),
                api_key: None,
                account_label: Some("vLLM Local/LAN (:8000)".to_string()),
                default_model: None,
                custom_models: Vec::new(),
            },
        );
        providers.insert(
            "openapi".to_string(),
            ProviderEntryConfig {
                id: "openapi".to_string(),
                enabled: false,
                auth_kind: ProviderAuthKind::None,
                base_url: "http://127.0.0.1:8090/v1".to_string(),
                api_key: None,
                account_label: None,
                default_model: None,
                custom_models: Vec::new(),
            },
        );

        // === TIER 2: BIG CLOUD PROVIDERS (STRICTLY OFF BY DEFAULT UNTIL OAUTH/KEY) ===
        for (id, base_url, default_model) in [
            (
                "openrouter",
                "https://openrouter.ai/api/v1",
                "anthropic/claude-3.5-sonnet",
            ),
            (
                "claude",
                "https://api.anthropic.com/v1",
                "claude-sonnet-4-6",
            ),
            ("openai", "https://api.openai.com/v1", "gpt-4o"),
            (
                "google",
                "https://generativelanguage.googleapis.com/v1beta",
                "gemini-2.0-flash",
            ),
            ("grok", "https://api.x.ai/v1", "grok-2-latest"),
            (
                "github-copilot",
                "https://api.githubcopilot.com",
                "claude-sonnet-4",
            ),
            ("kimi", "https://api.moonshot.cn/v1", "kimi-k2"),
            ("devin", "https://api.devin.ai/v1", "devin-default"),
            ("meta", "https://api.llama.com/v1", "llama-3.3-70b"),
            ("zai", "https://open.bigmodel.cn/api/paas/v4", "glm-4-plus"),
        ] {
            providers.insert(
                id.to_string(),
                ProviderEntryConfig {
                    id: id.to_string(),
                    enabled: false, // Strictly OFF until connected!
                    auth_kind: ProviderAuthKind::None,
                    base_url: base_url.to_string(),
                    api_key: None,
                    account_label: None,
                    default_model: Some(default_model.to_string()),
                    custom_models: Vec::new(),
                },
            );
        }

        Self {
            active_provider: "st-sovereign".to_string(),
            active_model: "in-binary:st-deepseek-pqm-4gaia".to_string(),
            cliproxy_url: default_cliproxy_url(),
            cliproxy_management_key: None,
            providers,
        }
    }
}

impl ProvidersRegistryFile {
    pub fn path() -> PathBuf {
        dirs::home_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join(".st")
            .join("providers.json")
    }

    pub fn load() -> Self {
        let path = Self::path();
        let mut base = Self::default();
        if let Ok(content) = std::fs::read_to_string(&path) {
            if let Ok(saved) = serde_json::from_str::<ProvidersRegistryFile>(&content) {
                base.active_provider = saved.active_provider;
                base.active_model = saved.active_model;
                base.cliproxy_url = saved.cliproxy_url;
                base.cliproxy_management_key = saved.cliproxy_management_key;
                for (k, v) in saved.providers {
                    base.providers.insert(k, v);
                }
            }
        }
        // Check if token_store has OAuth tokens for any provider
        for (id, entry) in base.providers.iter_mut() {
            if entry.auth_kind == ProviderAuthKind::None {
                let oauth_name = match id.as_str() {
                    "google" => "antigravity",
                    "openai" => "codex",
                    other => other,
                };
                if let Ok(Some(tok)) = token_store::load(oauth_name, "default") {
                    if !tok.access_token.is_empty() {
                        entry.auth_kind = ProviderAuthKind::OAuth;
                    }
                }
            }
        }
        base
    }

    pub fn save(&self) -> Result<()> {
        let path = Self::path();
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let json = serde_json::to_string_pretty(self)?;
        std::fs::write(&path, json)?;
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            if let Ok(meta) = std::fs::metadata(&path) {
                let mut perms = meta.permissions();
                perms.set_mode(0o600);
                let _ = std::fs::set_permissions(&path, perms);
            }
        }
        Ok(())
    }

    /// Check whether a provider is allowed to run.
    /// Enforces: "Otherwise those other providers need to be off."
    pub fn is_provider_enabled_and_authenticated(&self, id: &str) -> bool {
        let norm = id.to_ascii_lowercase();
        let key = match norm.as_str() {
            "anthropic" => "claude",
            "gemini" | "antigravity" => "google",
            "codex" | "chatgpt" => "openai",
            "xai" => "grok",
            "burn" | "local" | "mem8" => "st-sovereign",
            other => other,
        };
        let Some(entry) = self.providers.get(key) else {
            return false;
        };
        if !entry.enabled {
            return false;
        }
        match key {
            "st-sovereign" | "ollama" | "lmstudio" | "llamacpp" | "vllm" => true,
            _ => {
                entry.auth_kind != ProviderAuthKind::None
                    || entry.api_key.as_ref().is_some_and(|k| !k.is_empty())
            }
        }
    }

    /// Resolve effective API key or OAuth access token for a provider if it is enabled.
    pub fn resolve_credential(&self, id: &str) -> Option<String> {
        if !self.is_provider_enabled_and_authenticated(id) {
            return None;
        }
        let entry = self.providers.get(id)?;
        if let Some(key) = &entry.api_key {
            if !key.trim().is_empty() {
                return Some(key.trim().to_string());
            }
        }
        let oauth_name = match id {
            "google" => "antigravity",
            "openai" => "codex",
            other => other,
        };
        if let Ok(Some(tok)) = token_store::load(oauth_name, "default") {
            if !tok.access_token.is_empty() {
                return Some(tok.access_token);
            }
        }
        None
    }
}

/// Metadata + live state for a single provider card in the Web Dashboard.
#[derive(Debug, Clone, Serialize)]
pub struct DashboardProviderCard {
    pub id: String,
    pub name: String,
    pub badge: String,
    pub icon: String,
    pub tagline: String,
    pub category: String, // "local" (Priority #1) or "cloud"
    pub priority: u8,
    pub enabled: bool,
    pub connected: bool,
    pub status_text: String,
    pub auth_kind: ProviderAuthKind,
    pub supports_oauth: bool,
    pub supports_api_key: bool,
    pub supports_base_url: bool,
    pub base_url: String,
    pub has_api_key: bool,
    pub account_label: Option<String>,
    pub default_model: Option<String>,
    pub available_models: Vec<String>,
    pub oauth_help: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct ProvidersOverviewResponse {
    pub active_provider: String,
    pub active_model: String,
    pub local_priority_mode: bool,
    pub cliproxy_detected: bool,
    pub opencode_auth_detected: bool,
    pub providers: Vec<DashboardProviderCard>,
}

/// Static display metadata for all supported local & cloud providers.
struct ProviderCatalogMeta {
    id: &'static str,
    name: &'static str,
    badge: &'static str,
    icon: &'static str,
    tagline: &'static str,
    category: &'static str,
    priority: u8,
    supports_oauth: bool,
    supports_api_key: bool,
    supports_base_url: bool,
    oauth_help: &'static str,
    builtin_models: &'static [&'static str],
}

const PROVIDER_CATALOG: &[ProviderCatalogMeta] = &[
    // === PRIORITY #1: OUR OWN LOCAL MODELS ===
    ProviderCatalogMeta {
        id: "st-sovereign",
        name: "Smart Tree Local Sovereign Engine",
        badge: "PRIORITY #1 · BUILT-IN",
        icon: "🌲",
        tagline: "Zero-cloud 4gaia PQM, Grandma Jody & 11 .m8 Souls — instant, private & free!",
        category: "local",
        priority: 1,
        supports_oauth: false,
        supports_api_key: false,
        supports_base_url: false,
        oauth_help: "Always built-in — no sign-in or internet needed!",
        builtin_models: &[
            "in-binary:st-deepseek-pqm-4gaia",
            "persona:grandma-jody",
            "in-binary:st-art-triad",
            "in-binary:st-love-heart",
            "in-binary:st-poetry-verse",
            "in-binary:st-sound-phoenix",
            "burn/mem8-ternary-micro",
        ],
    },
    ProviderCatalogMeta {
        id: "ollama",
        name: "Ollama (Local & MemNet Close-By)",
        badge: "LOCAL · AUTO-DETECT",
        icon: "🦙",
        tagline: "Local GGUF/Safetensors models (`st-deepseek-pqm:latest`, Llama, Qwen, DeepSeek).",
        category: "local",
        priority: 2,
        supports_oauth: false,
        supports_api_key: false,
        supports_base_url: true,
        oauth_help: "Connects automatically to localhost:11434 or :11436.",
        builtin_models: &["st-deepseek-pqm:latest", "aye-deepseek-pqm:latest"],
    },
    ProviderCatalogMeta {
        id: "lmstudio",
        name: "LM Studio Local Server",
        badge: "LOCAL · :1234",
        icon: "🖥️",
        tagline: "Local LM Studio OpenAI-compatible server on port 1234.",
        category: "local",
        priority: 3,
        supports_oauth: false,
        supports_api_key: false,
        supports_base_url: true,
        oauth_help: "Start LM Studio Local Server on port 1234 and click Scan.",
        builtin_models: &[],
    },
    ProviderCatalogMeta {
        id: "llamacpp",
        name: "llama.cpp Server",
        badge: "LOCAL · :8080",
        icon: "⚡",
        tagline: "Pure C/C++ `llama-server` OpenAI-compatible endpoint on port 8080.",
        category: "local",
        priority: 4,
        supports_oauth: false,
        supports_api_key: false,
        supports_base_url: true,
        oauth_help: "Run `llama-server --port 8080` and click Scan.",
        builtin_models: &[],
    },
    ProviderCatalogMeta {
        id: "vllm",
        name: "vLLM High-Throughput Server",
        badge: "LOCAL/LAN · :8000",
        icon: "🚀",
        tagline: "Local or LAN GPU vLLM OpenAI-compatible server on port 8000.",
        category: "local",
        priority: 5,
        supports_oauth: false,
        supports_api_key: true,
        supports_base_url: true,
        oauth_help: "Point to http://localhost:8000/v1 (or LAN GPU box).",
        builtin_models: &[],
    },
    ProviderCatalogMeta {
        id: "openapi",
        name: "Custom OpenAPI / OpenAI-Compatible",
        badge: "UNIVERSAL /v1",
        icon: "🔌",
        tagline: "Any OpenAI-compatible `/v1` server (Jan, LocalAI, Tabby, OpenWebUI, custom).",
        category: "local",
        priority: 6,
        supports_oauth: false,
        supports_api_key: true,
        supports_base_url: true,
        oauth_help: "Enter any http(s) `/v1` base URL + optional key to auto-discover models.",
        builtin_models: &[],
    },
    // === TIER 2: BIG CLOUD PROVIDERS (OFF BY DEFAULT UNTIL OAUTH / KEY) ===
    ProviderCatalogMeta {
        id: "openrouter",
        name: "OpenRouter (200+ Models)",
        badge: "1-CLICK PKCE OAUTH",
        icon: "🌐",
        tagline:
            "One-click browser OAuth (zero setup!) or API key for Claude, GPT, DeepSeek & Llama.",
        category: "cloud",
        priority: 10,
        supports_oauth: true,
        supports_api_key: true,
        supports_base_url: true,
        oauth_help: "Click 'Sign in with OAuth' to authorize directly on openrouter.ai!",
        builtin_models: &[
            "anthropic/claude-3.5-sonnet",
            "deepseek/deepseek-chat",
            "google/gemini-2.0-flash-001",
            "meta-llama/llama-3.3-70b-instruct",
            "openai/gpt-4o-mini",
        ],
    },
    ProviderCatalogMeta {
        id: "claude",
        name: "Anthropic Claude",
        badge: "OAUTH / API KEY",
        icon: "🤖",
        tagline:
            "Claude Opus 4.6, Sonnet 4.6 & Haiku 4.5 via OAuth, OpenCode, CLIProxyAPI, or Key.",
        category: "cloud",
        priority: 11,
        supports_oauth: true,
        supports_api_key: true,
        supports_base_url: true,
        oauth_help: "Sign in with Claude OAuth, import from OpenCode, or paste sk-ant-...",
        builtin_models: &["claude-sonnet-4-6", "claude-opus-4-6", "claude-haiku-4-5"],
    },
    ProviderCatalogMeta {
        id: "openai",
        name: "OpenAI (ChatGPT / Codex)",
        badge: "OAUTH / API KEY",
        icon: "✨",
        tagline: "GPT-4o, o3-mini & Codex via ChatGPT/Codex OAuth, CLIProxyAPI, or API Key.",
        category: "cloud",
        priority: 12,
        supports_oauth: true,
        supports_api_key: true,
        supports_base_url: true,
        oauth_help: "Sign in with OpenAI Codex OAuth or paste sk-proj-...",
        builtin_models: &["gpt-4o", "gpt-4o-mini", "o3-mini"],
    },
    ProviderCatalogMeta {
        id: "google",
        name: "Google Gemini / Antigravity",
        badge: "OAUTH / API KEY",
        icon: "💎",
        tagline: "Gemini 2.5 Pro & 2.0 Flash via Google OAuth 2.0 PKCE or AI Studio API Key.",
        category: "cloud",
        priority: 13,
        supports_oauth: true,
        supports_api_key: true,
        supports_base_url: true,
        oauth_help: "Sign in with Google OAuth or paste AI Studio API Key.",
        builtin_models: &["gemini-2.0-flash", "gemini-2.5-pro", "gemini-2.5-flash"],
    },
    ProviderCatalogMeta {
        id: "grok",
        name: "xAI Grok",
        badge: "OAUTH / API KEY",
        icon: "⚡",
        tagline: "Grok-2 & Grok-3 via xAI OAuth, CLIProxyAPI, or XAI_API_KEY.",
        category: "cloud",
        priority: 14,
        supports_oauth: true,
        supports_api_key: true,
        supports_base_url: true,
        oauth_help: "Sign in with xAI OAuth or paste xai-...",
        builtin_models: &["grok-2-latest", "grok-beta"],
    },
    ProviderCatalogMeta {
        id: "github-copilot",
        name: "GitHub Copilot",
        badge: "OAUTH / OPENCODE",
        icon: "🐙",
        tagline: "Use your GitHub Copilot subscription via GitHub OAuth or OpenCode import.",
        category: "cloud",
        priority: 15,
        supports_oauth: true,
        supports_api_key: true,
        supports_base_url: false,
        oauth_help: "Sign in with GitHub OAuth or click 'Import from OpenCode'.",
        builtin_models: &["claude-sonnet-4", "gpt-4o", "o3-mini"],
    },
    ProviderCatalogMeta {
        id: "kimi",
        name: "Moonshot Kimi",
        badge: "OAUTH / API KEY",
        icon: "🌙",
        tagline: "Kimi K2 long-context models via Moonshot OAuth or API Key.",
        category: "cloud",
        priority: 16,
        supports_oauth: true,
        supports_api_key: true,
        supports_base_url: true,
        oauth_help: "Sign in with Kimi OAuth or paste Moonshot API key.",
        builtin_models: &["kimi-k2", "moonshot-v1-128k"],
    },
    ProviderCatalogMeta {
        id: "devin",
        name: "Devin (Cognition)",
        badge: "OAUTH / CLIPROXY",
        icon: "🛠️",
        tagline: "Devin coding subscription via CLIProxyAPI or OAuth.",
        category: "cloud",
        priority: 17,
        supports_oauth: true,
        supports_api_key: true,
        supports_base_url: true,
        oauth_help: "Sign in via OAuth / CLIProxyAPI or paste API key.",
        builtin_models: &["devin-default"],
    },
    ProviderCatalogMeta {
        id: "meta",
        name: "Meta Muse / Llama API",
        badge: "OAUTH / API KEY",
        icon: "♾️",
        tagline: "Meta Muse & Llama 3.3 models via OAuth or Llama API Key.",
        category: "cloud",
        priority: 18,
        supports_oauth: true,
        supports_api_key: true,
        supports_base_url: true,
        oauth_help: "Sign in with Meta Muse OAuth or paste API key.",
        builtin_models: &["llama-3.3-70b", "llama-3.1-405b"],
    },
    ProviderCatalogMeta {
        id: "zai",
        name: "Z.AI (GLM-4)",
        badge: "API KEY",
        icon: "🔮",
        tagline: "GLM-4-Plus & GLM-Zero reasoning models.",
        category: "cloud",
        priority: 19,
        supports_oauth: false,
        supports_api_key: true,
        supports_base_url: true,
        oauth_help: "Paste your Z.AI / BigModel API key.",
        builtin_models: &["glm-4-plus", "glm-4-flash"],
    },
];

/// Probe an OpenAI-compatible `/v1/models` endpoint (just like `discoverUpstreamModels` in `vscode-universal-chat-provider`).
pub async fn discover_upstream_models(
    base_url: &str,
    api_key: Option<&str>,
    timeout_ms: u64,
) -> Vec<String> {
    let trimmed = base_url.trim().trim_end_matches('/');
    if trimmed.is_empty() || trimmed.starts_with("in-binary://") {
        return Vec::new();
    }
    let client = match Client::builder()
        .timeout(Duration::from_millis(timeout_ms))
        .build()
    {
        Ok(c) => c,
        Err(_) => return Vec::new(),
    };

    // 1. If Ollama port or `/api/tags`
    if trimmed.contains(":11434") || trimmed.contains(":11436") {
        let root = trimmed.trim_end_matches("/v1");
        if let Ok(resp) = client.get(format!("{root}/api/tags")).send().await {
            if let Ok(json) = resp.json::<serde_json::Value>().await {
                if let Some(arr) = json.get("models").and_then(|v| v.as_array()) {
                    let mut models: Vec<String> = arr
                        .iter()
                        .filter_map(|m| m.get("name").and_then(|n| n.as_str()).map(String::from))
                        .collect();
                    // Sort our own st-deepseek-pqm / aye-deepseek-pqm models to the top!
                    models.sort_by_key(|m| {
                        if m.starts_with("st-deepseek-pqm") {
                            0
                        } else if m.starts_with("aye-deepseek-pqm") {
                            1
                        } else {
                            2
                        }
                    });
                    if !models.is_empty() {
                        return models;
                    }
                }
            }
        }
    }

    // 2. Standard OpenAI-compatible `{base_url}/models` or `{base_url}/v1/models`
    let candidate_urls = if trimmed.ends_with("/v1") || trimmed.contains("/v1beta") {
        vec![format!("{trimmed}/models")]
    } else {
        vec![format!("{trimmed}/v1/models"), format!("{trimmed}/models")]
    };

    for url in candidate_urls {
        let mut req = client.get(&url);
        if let Some(k) = api_key {
            if !k.trim().is_empty() {
                req = req.header("Authorization", format!("Bearer {}", k.trim()));
            }
        }
        for (hk, hv) in supplier_headers_for_url(trimmed) {
            req = req.header(hk, hv);
        }
        if let Ok(resp) = req.send().await {
            if resp.status().is_success() {
                if let Ok(json) = resp.json::<serde_json::Value>().await {
                    if let Some(arr) = json.get("data").and_then(|v| v.as_array()) {
                        let out: Vec<String> = arr
                            .iter()
                            .filter_map(|m| {
                                m.get("id").and_then(|id| id.as_str()).map(String::from)
                            })
                            .take(60)
                            .collect();
                        if !out.is_empty() {
                            return out;
                        }
                    } else if let Some(arr) = json.get("models").and_then(|v| v.as_array()) {
                        let out: Vec<String> = arr
                            .iter()
                            .filter_map(|m| {
                                m.get("name")
                                    .or_else(|| m.get("id"))
                                    .and_then(|id| id.as_str())
                                    .map(|s| s.trim_start_matches("models/").to_string())
                            })
                            .take(60)
                            .collect();
                        if !out.is_empty() {
                            return out;
                        }
                    }
                }
            }
        }
    }
    Vec::new()
}

/// Check if `CLIProxyAPI` (`vscode-universal-chat-provider` backend) is running on `:8317`.
async fn detect_cliproxy_server(base_url: &str) -> bool {
    let Ok(client) = Client::builder()
        .timeout(Duration::from_millis(250))
        .build()
    else {
        return false;
    };
    let url = format!("{}/v1/models", base_url.trim_end_matches('/'));
    client.get(&url).send().await.is_ok()
}

/// Check if OpenCode auth file exists at `~/.local/share/opencode/auth.json` or `~/.config/opencode/opencode.json`.
fn detect_opencode_auth_files() -> bool {
    let Some(home) = dirs::home_dir() else {
        return false;
    };
    home.join(".local/share/opencode/auth.json").exists()
        || home.join(".config/opencode/opencode.json").exists()
}

/// Build the full list of provider cards for `GET /api/providers`.
pub async fn build_providers_overview(probe_live: bool) -> ProvidersOverviewResponse {
    let reg = ProvidersRegistryFile::load();
    let cliproxy_detected = detect_cliproxy_server(&reg.cliproxy_url).await;
    let opencode_auth_detected = detect_opencode_auth_files();

    // Gather all trained `.m8` personas for `st-sovereign`
    let mut sovereign_models: Vec<String> = PROVIDER_CATALOG[0]
        .builtin_models
        .iter()
        .map(|s| (*s).to_string())
        .collect();
    if let Ok(personas) = LifePersona::list_all() {
        for p in personas {
            let pid = format!("persona:{}", p.slug);
            if !sovereign_models.contains(&pid) {
                sovereign_models.push(pid);
            }
        }
    }

    let mut cards = Vec::with_capacity(PROVIDER_CATALOG.len());

    for meta in PROVIDER_CATALOG {
        let entry = reg
            .providers
            .get(meta.id)
            .cloned()
            .unwrap_or(ProviderEntryConfig {
                id: meta.id.to_string(),
                enabled: meta.category == "local" && meta.id != "openapi",
                auth_kind: if meta.id == "st-sovereign" {
                    ProviderAuthKind::LocalBuiltIn
                } else if meta.category == "local" {
                    ProviderAuthKind::LocalServer
                } else {
                    ProviderAuthKind::None
                },
                base_url: String::new(),
                api_key: None,
                account_label: None,
                default_model: meta.builtin_models.first().map(|s| (*s).to_string()),
                custom_models: Vec::new(),
            });

        let has_api_key = entry.api_key.as_ref().is_some_and(|k| !k.trim().is_empty());
        let has_oauth = matches!(
            entry.auth_kind,
            ProviderAuthKind::OAuth
                | ProviderAuthKind::CliProxyApi
                | ProviderAuthKind::OpenCodeImported
        );

        let mut available_models: Vec<String> = if meta.id == "st-sovereign" {
            sovereign_models.clone()
        } else {
            entry.custom_models.clone()
        };

        // Probe local servers (`ollama`, `lmstudio`, `llamacpp`, `vllm`, `openapi`) when enabled
        let mut live_reachable = meta.id == "st-sovereign";
        if probe_live && entry.enabled && meta.category == "local" && meta.id != "st-sovereign" {
            let discovered =
                discover_upstream_models(&entry.base_url, entry.api_key.as_deref(), 350).await;
            if !discovered.is_empty() {
                live_reachable = true;
                for m in discovered {
                    if !available_models.contains(&m) {
                        available_models.push(m);
                    }
                }
            }
        }

        for bm in meta.builtin_models {
            let s = (*bm).to_string();
            if !available_models.contains(&s) {
                available_models.push(s);
            }
        }

        let connected = if meta.id == "st-sovereign" {
            true
        } else if meta.category == "local" {
            live_reachable || !entry.custom_models.is_empty()
        } else {
            has_oauth || has_api_key
        };

        let status_text = if meta.id == "st-sovereign" {
            format!(
                "ACTIVE · {} local models & souls ready",
                available_models.len()
            )
        } else if !entry.enabled {
            "OFF (Disabled — zero cloud traffic)".to_string()
        } else if meta.category == "local" {
            if live_reachable {
                format!("ONLINE · {} local models detected", available_models.len())
            } else {
                format!("Standby ({}) — start server or click Scan", entry.base_url)
            }
        } else if connected {
            let via = match entry.auth_kind {
                ProviderAuthKind::OAuth => "OAuth Connected",
                ProviderAuthKind::CliProxyApi => "CLIProxyAPI OAuth",
                ProviderAuthKind::OpenCodeImported => "OpenCode Auth",
                ProviderAuthKind::ApiKey => "API Key Saved",
                _ => "Connected",
            };
            format!("ON · {via}")
        } else {
            "OFF · Sign in with OAuth or paste API Key to enable".to_string()
        };

        cards.push(DashboardProviderCard {
            id: meta.id.to_string(),
            name: meta.name.to_string(),
            badge: meta.badge.to_string(),
            icon: meta.icon.to_string(),
            tagline: meta.tagline.to_string(),
            category: meta.category.to_string(),
            priority: meta.priority,
            enabled: entry.enabled && (meta.category == "local" || connected),
            connected,
            status_text,
            auth_kind: entry.auth_kind,
            supports_oauth: meta.supports_oauth,
            supports_api_key: meta.supports_api_key,
            supports_base_url: meta.supports_base_url,
            base_url: entry.base_url,
            has_api_key,
            account_label: entry.account_label,
            default_model: entry
                .default_model
                .or_else(|| available_models.first().cloned()),
            available_models,
            oauth_help: meta.oauth_help.to_string(),
        });
    }

    cards.sort_by_key(|c| c.priority);

    ProvidersOverviewResponse {
        active_provider: reg.active_provider,
        active_model: reg.active_model,
        local_priority_mode: true,
        cliproxy_detected,
        opencode_auth_detected,
        providers: cards,
    }
}

// ============================================================================
// In-Memory OAuth Flow Tracker for Web Dashboard Polling
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OAuthPollState {
    pub provider_id: String,
    pub status: String, // "wait", "ok", "error"
    pub auth_url: String,
    pub message: String,
    pub cliproxy_state: Option<String>,
}

fn oauth_sessions() -> &'static Arc<RwLock<HashMap<String, OAuthPollState>>> {
    static SESSIONS: OnceLock<Arc<RwLock<HashMap<String, OAuthPollState>>>> = OnceLock::new();
    SESSIONS.get_or_init(|| Arc::new(RwLock::new(HashMap::new())))
}

#[derive(Debug, Deserialize)]
pub struct StartOAuthRequest {
    pub provider_id: String,
    pub client_id: Option<String>,
    pub client_secret: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct StartOAuthResponse {
    pub state: String,
    pub auth_url: String,
    pub provider_id: String,
    pub flow_kind: String,
    pub instructions: String,
}

#[derive(Debug, Deserialize)]
pub struct OAuthStatusQuery {
    pub state: String,
}

#[derive(Debug, Deserialize)]
pub struct ToggleProviderRequest {
    pub provider_id: String,
    pub enabled: bool,
    pub set_active: Option<bool>,
    pub default_model: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct ConfigureProviderRequest {
    pub provider_id: String,
    pub enabled: Option<bool>,
    pub api_key: Option<String>,
    pub base_url: Option<String>,
    pub default_model: Option<String>,
    pub custom_models: Option<Vec<String>>,
    pub set_active: Option<bool>,
}

#[derive(Debug, Deserialize)]
pub struct DisconnectProviderRequest {
    pub provider_id: String,
}

/// `GET /api/providers`
pub async fn get_providers() -> Json<ProvidersOverviewResponse> {
    Json(build_providers_overview(true).await)
}

/// `POST /api/providers/discover`
pub async fn discover_providers() -> Json<ProvidersOverviewResponse> {
    let mut reg = ProvidersRegistryFile::load();
    for (id, entry) in reg.providers.iter_mut() {
        if entry.enabled && id != "st-sovereign" {
            let found =
                discover_upstream_models(&entry.base_url, entry.api_key.as_deref(), 1200).await;
            if !found.is_empty() {
                entry.custom_models = found.clone();
                if entry.default_model.is_none() {
                    entry.default_model = found.first().cloned();
                }
            }
        }
    }
    let _ = reg.save();
    Json(build_providers_overview(true).await)
}

/// `POST /api/providers/toggle`
pub async fn toggle_provider(
    Json(req): Json<ToggleProviderRequest>,
) -> Result<Json<ProvidersOverviewResponse>, (StatusCode, String)> {
    let mut reg = ProvidersRegistryFile::load();
    let entry = reg.providers.get_mut(&req.provider_id).ok_or_else(|| {
        (
            StatusCode::NOT_FOUND,
            format!("Unknown provider {}", req.provider_id),
        )
    })?;

    entry.enabled = req.enabled;
    if let Some(dm) = &req.default_model {
        if !dm.trim().is_empty() {
            entry.default_model = Some(dm.trim().to_string());
        }
    }

    if req.set_active.unwrap_or(false) && req.enabled {
        reg.active_provider = req.provider_id.clone();
        if let Some(dm) = entry.default_model.clone() {
            reg.active_model = dm;
        }
    } else if !req.enabled && reg.active_provider == req.provider_id {
        // Fall back to our own local Sovereign model whenever a provider is turned OFF!
        reg.active_provider = "st-sovereign".to_string();
        reg.active_model = "in-binary:st-deepseek-pqm-4gaia".to_string();
    }

    reg.save()
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(Json(build_providers_overview(false).await))
}

/// `POST /api/providers/configure`
pub async fn configure_provider(
    Json(req): Json<ConfigureProviderRequest>,
) -> Result<Json<ProvidersOverviewResponse>, (StatusCode, String)> {
    let mut reg = ProvidersRegistryFile::load();
    let entry = reg.providers.get_mut(&req.provider_id).ok_or_else(|| {
        (
            StatusCode::NOT_FOUND,
            format!("Unknown provider {}", req.provider_id),
        )
    })?;

    if let Some(url) = &req.base_url {
        let trimmed = url.trim().trim_end_matches('/');
        if !trimmed.is_empty() {
            entry.base_url = trimmed.to_string();
        }
    }

    if let Some(key) = &req.api_key {
        let trimmed = key.trim();
        if !trimmed.is_empty() {
            entry.api_key = Some(trimmed.to_string());
            entry.auth_kind = ProviderAuthKind::ApiKey;
            entry.enabled = true;
            // Also store in token_store for unified proxy access
            let _ = token_store::save(
                &req.provider_id,
                "default",
                &StoredToken {
                    access_token: trimmed.to_string(),
                    refresh_token: None,
                    expires_at: None,
                    scope: Some("api_key".to_string()),
                    token_type: Some("Bearer".to_string()),
                },
            );
        }
    }

    if let Some(en) = req.enabled {
        entry.enabled = en;
    } else if entry.auth_kind != ProviderAuthKind::None || req.base_url.is_some() {
        entry.enabled = true;
    }

    if let Some(custom) = req.custom_models {
        if !custom.is_empty() {
            entry.custom_models = custom;
        }
    }

    // Auto-discover models from endpoint (like `vscode-universal-chat-provider`)
    let discovered =
        discover_upstream_models(&entry.base_url, entry.api_key.as_deref(), 1500).await;
    if !discovered.is_empty() {
        for m in discovered {
            if !entry.custom_models.contains(&m) {
                entry.custom_models.push(m);
            }
        }
    }

    if let Some(dm) = &req.default_model {
        if !dm.trim().is_empty() {
            entry.default_model = Some(dm.trim().to_string());
        }
    } else if entry.default_model.is_none() {
        entry.default_model = entry.custom_models.first().cloned();
    }

    if req.set_active.unwrap_or(false) && entry.enabled {
        reg.active_provider = req.provider_id.clone();
        if let Some(dm) = entry.default_model.clone() {
            reg.active_model = dm;
        }
    }

    reg.save()
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(Json(build_providers_overview(false).await))
}

/// `POST /api/providers/disconnect`
pub async fn disconnect_provider(
    Json(req): Json<DisconnectProviderRequest>,
) -> Result<Json<ProvidersOverviewResponse>, (StatusCode, String)> {
    let mut reg = ProvidersRegistryFile::load();
    if let Some(entry) = reg.providers.get_mut(&req.provider_id) {
        if req.provider_id != "st-sovereign" {
            entry.enabled = false;
            entry.api_key = None;
            entry.account_label = None;
            if entry.auth_kind != ProviderAuthKind::LocalServer {
                entry.auth_kind = ProviderAuthKind::None;
            }
        }
    }
    let oauth_name = match req.provider_id.as_str() {
        "google" => "antigravity",
        "openai" => "codex",
        other => other,
    };
    let _ = token_store::delete(oauth_name, "default");
    let _ = token_store::delete(&req.provider_id, "default");

    if reg.active_provider == req.provider_id {
        reg.active_provider = "st-sovereign".to_string();
        reg.active_model = "in-binary:st-deepseek-pqm-4gaia".to_string();
    }
    reg.save()
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(Json(build_providers_overview(false).await))
}

/// `POST /api/providers/oauth/start`
/// Starts a browser OAuth 2.0 PKCE flow (or CLIProxyAPI management flow if running).
pub async fn start_oauth_flow(
    Json(req): Json<StartOAuthRequest>,
) -> Result<Json<StartOAuthResponse>, (StatusCode, String)> {
    let provider_id = req.provider_id.trim().to_lowercase();
    if let Some(cid) = &req.client_id {
        if !cid.trim().is_empty() {
            let env_key = match provider_id.as_str() {
                "google" | "antigravity" => "ANTIGRAVITY_CLIENT_ID",
                "openai" | "codex" => "CODEX_CLIENT_ID",
                "claude" => "CLAUDE_CLIENT_ID",
                "grok" | "xai" => "XAI_CLIENT_ID",
                "github-copilot" | "github" => "GITHUB_CLIENT_ID",
                "kimi" => "KIMI_CLIENT_ID",
                _ => "ST_OAUTH_CLIENT_ID",
            };
            std::env::set_var(env_key, cid.trim());
        }
    }
    if let Some(sec) = &req.client_secret {
        if !sec.trim().is_empty() {
            let env_key = match provider_id.as_str() {
                "google" | "antigravity" => "ANTIGRAVITY_CLIENT_SECRET",
                "github-copilot" | "github" => "GITHUB_CLIENT_SECRET",
                _ => "ST_OAUTH_CLIENT_SECRET",
            };
            std::env::set_var(env_key, sec.trim());
        }
    }

    let reg = ProvidersRegistryFile::load();

    // 1. Check if CLIProxyAPI (`vscode-universal-chat-provider` server) is running and has `/v8/management/oauth/auth-url`
    let cliproxy_provider = match provider_id.as_str() {
        "openai" => "codex",
        "google" => "antigravity",
        "grok" => "xai",
        other => other,
    };
    if detect_cliproxy_server(&reg.cliproxy_url).await {
        if let Ok(client) = Client::builder().timeout(Duration::from_secs(3)).build() {
            let url = format!(
                "{}/v8/management/oauth/auth-url?provider={}&is_webui=true",
                reg.cliproxy_url.trim_end_matches('/'),
                cliproxy_provider
            );
            let mut http_req = client.get(&url);
            if let Some(mgmt_key) = &reg.cliproxy_management_key {
                http_req = http_req.header("Authorization", format!("Bearer {mgmt_key}"));
            }
            if let Ok(resp) = http_req.send().await {
                if resp.status().is_success() {
                    if let Ok(json) = resp.json::<serde_json::Value>().await {
                        if let (Some(auth_url), Some(cpa_state)) = (
                            json.get("url").and_then(|v| v.as_str()),
                            json.get("state").and_then(|v| v.as_str()),
                        ) {
                            let state_id = format!("cpa-{cpa_state}");
                            oauth_sessions().write().await.insert(
                                state_id.clone(),
                                OAuthPollState {
                                    provider_id: provider_id.clone(),
                                    status: "wait".to_string(),
                                    auth_url: auth_url.to_string(),
                                    message: "Waiting for browser OAuth sign-in via CLIProxyAPI..."
                                        .to_string(),
                                    cliproxy_state: Some(cpa_state.to_string()),
                                },
                            );
                            return Ok(Json(StartOAuthResponse {
                                state: state_id,
                                auth_url: auth_url.to_string(),
                                provider_id,
                                flow_kind: "cliproxy".to_string(),
                                instructions:
                                    "Complete sign-in in the opened browser tab; Smart Tree will connect automatically."
                                        .to_string(),
                            }));
                        }
                    }
                }
            }
        }
    }

    // 2. Native Rust OAuth 2.0 + PKCE Loopback Flow (`src/proxy/oauth.rs`)
    let cfg = oauth::providers::by_name(&provider_id).ok_or_else(|| {
        (
            StatusCode::BAD_REQUEST,
            format!("Provider '{}' does not support OAuth", provider_id),
        )
    })?;

    let started = oauth::begin(cfg, "default".to_string())
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let state_id = started.state.clone();
    let auth_url = started.auth_url.clone();
    let provider_for_task = provider_id.clone();
    let state_for_task = state_id.clone();

    oauth_sessions().write().await.insert(
        state_id.clone(),
        OAuthPollState {
            provider_id: provider_id.clone(),
            status: "wait".to_string(),
            auth_url: auth_url.clone(),
            message: "Waiting for browser OAuth callback...".to_string(),
            cliproxy_state: None,
        },
    );

    tokio::spawn(async move {
        match started.wait(Duration::from_secs(180)).await {
            Ok(tok) => {
                let mut reg = ProvidersRegistryFile::load();
                if let Some(entry) = reg.providers.get_mut(&provider_for_task) {
                    entry.enabled = true;
                    entry.auth_kind = ProviderAuthKind::OAuth;
                    entry.api_key = Some(tok.access_token.clone());
                    entry.account_label = Some("OAuth Account".to_string());
                }
                let _ = reg.save();
                if let Some(sess) = oauth_sessions().write().await.get_mut(&state_for_task) {
                    sess.status = "ok".to_string();
                    sess.message = format!("{} connected via OAuth!", provider_for_task);
                }
            }
            Err(e) => {
                if let Some(sess) = oauth_sessions().write().await.get_mut(&state_for_task) {
                    sess.status = "error".to_string();
                    sess.message = e.to_string();
                }
            }
        }
    });

    Ok(Json(StartOAuthResponse {
        state: state_id,
        auth_url,
        provider_id,
        flow_kind: "pkce_loopback".to_string(),
        instructions:
            "Complete sign-in in your browser. As soon as it redirects back, this provider turns ON automatically!"
                .to_string(),
    }))
}

/// `GET /api/providers/oauth/status?state=...`
pub async fn get_oauth_status(
    Query(query): Query<OAuthStatusQuery>,
) -> Result<Json<OAuthPollState>, (StatusCode, String)> {
    let maybe_sess = oauth_sessions().read().await.get(&query.state).cloned();
    let Some(mut sess) = maybe_sess else {
        return Err((StatusCode::NOT_FOUND, "OAuth session not found".to_string()));
    };

    // If backed by CLIProxyAPI, poll `/v8/management/oauth/status?state=...`
    if sess.status == "wait" {
        if let Some(cpa_state) = &sess.cliproxy_state {
            let reg = ProvidersRegistryFile::load();
            if let Ok(client) = Client::builder().timeout(Duration::from_secs(2)).build() {
                let url = format!(
                    "{}/v8/management/oauth/status?state={}",
                    reg.cliproxy_url.trim_end_matches('/'),
                    cpa_state
                );
                let mut req = client.get(&url);
                if let Some(k) = &reg.cliproxy_management_key {
                    req = req.header("Authorization", format!("Bearer {k}"));
                }
                if let Ok(resp) = req.send().await {
                    if let Ok(json) = resp.json::<serde_json::Value>().await {
                        if let Some(st) = json.get("status").and_then(|v| v.as_str()) {
                            if st == "ok" {
                                sess.status = "ok".to_string();
                                sess.message = "Connected via CLIProxyAPI OAuth!".to_string();
                                let mut reg_mut = ProvidersRegistryFile::load();
                                if let Some(entry) = reg_mut.providers.get_mut(&sess.provider_id) {
                                    entry.enabled = true;
                                    entry.auth_kind = ProviderAuthKind::CliProxyApi;
                                    entry.account_label = Some("CLIProxyAPI Account".to_string());
                                }
                                let _ = reg_mut.save();
                                oauth_sessions()
                                    .write()
                                    .await
                                    .insert(query.state.clone(), sess.clone());
                            } else if st == "error" {
                                sess.status = "error".to_string();
                                sess.message = json
                                    .get("error")
                                    .and_then(|v| v.as_str())
                                    .unwrap_or("OAuth failed")
                                    .to_string();
                                oauth_sessions()
                                    .write()
                                    .await
                                    .insert(query.state.clone(), sess.clone());
                            }
                        }
                    }
                }
            }
        }
    }

    Ok(Json(sess))
}

#[derive(Debug, Serialize)]
pub struct ImportAuthResponse {
    pub imported_providers: Vec<String>,
    pub overview: ProvidersOverviewResponse,
}

/// `POST /api/providers/import-opencode`
/// Imports existing OAuth and API credentials from OpenCode (`~/.local/share/opencode/auth.json`
/// and `~/.config/opencode/opencode.json`) so users don't even have to log in twice!
pub async fn import_opencode_auth() -> Result<Json<ImportAuthResponse>, (StatusCode, String)> {
    let mut reg = ProvidersRegistryFile::load();
    let mut imported = Vec::new();

    if let Some(home) = dirs::home_dir() {
        // 1. Check `~/.local/share/opencode/auth.json` (`Auth.Info` format from opencode)
        let auth_path = home.join(".local/share/opencode/auth.json");
        if let Ok(raw) = std::fs::read_to_string(&auth_path) {
            if let Ok(map) =
                serde_json::from_str::<serde_json::Map<String, serde_json::Value>>(&raw)
            {
                for (k, val) in map {
                    let k_lower = k.to_ascii_lowercase();
                    let target_id = match k_lower.as_str() {
                        "anthropic" | "claude" => "claude",
                        "openai" | "codex" => "openai",
                        "openrouter" => "openrouter",
                        "google" | "gemini" => "google",
                        "github-copilot" | "copilot" => "github-copilot",
                        "xai" | "grok" => "grok",
                        other => other,
                    };
                    let token_opt = val
                        .get("key")
                        .or_else(|| val.get("access"))
                        .or_else(|| val.get("token"))
                        .and_then(|v| v.as_str());
                    if let Some(token_str) = token_opt {
                        if !token_str.trim().is_empty() {
                            if let Some(entry) = reg.providers.get_mut(target_id) {
                                entry.enabled = true;
                                entry.auth_kind = ProviderAuthKind::OpenCodeImported;
                                entry.api_key = Some(token_str.trim().to_string());
                                entry.account_label = Some(format!("Imported from OpenCode ({k})"));
                                imported.push(target_id.to_string());
                            }
                        }
                    }
                }
            }
        }

        // 2. Check `~/.config/opencode/opencode.json` for custom local/OpenAI-compatible providers
        let cfg_path = home.join(".config/opencode/opencode.json");
        if let Ok(raw) = std::fs::read_to_string(&cfg_path) {
            if let Ok(json) = serde_json::from_str::<serde_json::Value>(&raw) {
                if let Some(prov_obj) = json.get("provider").and_then(|v| v.as_object()) {
                    for (pk, pval) in prov_obj {
                        let base_url = pval
                            .get("options")
                            .and_then(|o| o.get("baseURL").or_else(|| o.get("baseUrl")))
                            .and_then(|u| u.as_str());
                        if let Some(url) = base_url {
                            let target_id = if pk.contains("ollama") || url.contains(":11434") {
                                "ollama"
                            } else if pk.contains("vllm") || url.contains(":8000") {
                                "vllm"
                            } else if pk.contains("lmstudio") || url.contains(":1234") {
                                "lmstudio"
                            } else if pk.contains("llama") || url.contains(":8080") {
                                "llamacpp"
                            } else {
                                "openapi"
                            };
                            if let Some(entry) = reg.providers.get_mut(target_id) {
                                entry.enabled = true;
                                entry.base_url = url.trim_end_matches('/').to_string();
                                entry.account_label = Some(format!("OpenCode ({pk})"));
                                if !imported.contains(&target_id.to_string()) {
                                    imported.push(target_id.to_string());
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    reg.save()
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(Json(ImportAuthResponse {
        imported_providers: imported,
        overview: build_providers_overview(true).await,
    }))
}

#[derive(Debug, Deserialize, Default)]
pub struct RunPqBenchWebRequest {
    pub retrain: Option<bool>,
    pub base_model: Option<String>,
    pub ollama_tag: Option<String>,
}

/// `POST /api/providers/pq-bench`
/// Runs the head-to-head `p/q` benchmark comparing Ollama (`st-deepseek-pqm:latest`)
/// vs. Smart Tree (`in-binary:st-deepseek-pqm-4gaia`) on the same `p/q`-converted model.
pub async fn run_pq_bench_endpoint(
    Json(req): Json<RunPqBenchWebRequest>,
) -> Result<Json<crate::pq_bench::PqBenchReport>, (StatusCode, String)> {
    let cmd = crate::cli::PqBenchCommand {
        base_model: req
            .base_model
            .unwrap_or_else(|| "deepseek-ai/DeepSeek-R1-Distill-Qwen-1.5B".to_string()),
        ollama_tag: req
            .ollama_tag
            .unwrap_or_else(|| "st-deepseek-pqm:latest".to_string()),
        layers: 28,
        steps: 16,
        retrain: req.retrain.unwrap_or(false),
        ollama_timeout_ms: 1500,
        json: true,
    };
    let report = crate::pq_bench::run_pq_benchmark(&cmd)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    Ok(Json(report))
}

// ============================================================================
// Grandma's 1-Click Website Builder (`POST /api/website/generate`)
// ============================================================================

#[derive(Debug, Deserialize)]
pub struct GrandmaWebsiteRequest {
    /// Website title (e.g., "Grandma Jody's Secret Cookie Kitchen")
    pub title: String,
    /// Plain-English description of what Grandma (or any user) wants on the website
    pub description: String,
    /// Template preset: "recipe", "garden", "craft", "family", "project", or "custom"
    #[serde(default = "default_template")]
    pub template: String,
    /// Color vibe: "cozy-cream", "sunny-garden", "berry-rose", "ocean-breeze", "qwave-neon"
    #[serde(default = "default_vibe")]
    pub vibe: String,
    /// Optional provider override (defaults to `"st-sovereign"` — our own local models!)
    pub provider_id: Option<String>,
    /// Optional model override (defaults to `"persona:grandma-jody"` + `"in-binary:st-deepseek-pqm-4gaia"`)
    pub model_id: Option<String>,
}

fn default_template() -> String {
    "recipe".to_string()
}
fn default_vibe() -> String {
    "cozy-cream".to_string()
}

#[derive(Debug, Serialize)]
pub struct GrandmaWebsiteResponse {
    pub slug: String,
    pub title: String,
    pub file_path: String,
    pub rel_path: String,
    pub preview_url: String,
    pub provider_used: String,
    pub model_used: String,
    pub grandma_note: String,
    pub html: String,
}

/// Sanitize a title into a clean folder slug (`"Grandma's Cookies!"` -> `"grandmas-cookies"`).
fn slugify_title(title: &str) -> String {
    let mut out = String::new();
    let mut last_dash = false;
    for ch in title.chars() {
        if ch.is_ascii_alphanumeric() {
            out.push(ch.to_ascii_lowercase());
            last_dash = false;
        } else if (ch == ' ' || ch == '-' || ch == '_') && !last_dash && !out.is_empty() {
            out.push('-');
            last_dash = true;
        }
    }
    let trimmed = out.trim_matches('-').to_string();
    if trimmed.is_empty() {
        "my-cozy-website".to_string()
    } else {
        trimmed
    }
}

/// Generate a complete, heartwarming, responsive website using our Local Sovereign Engine
/// (`persona:grandma-jody` + `in-binary:st-art-triad` + `in-binary:st-deepseek-pqm-4gaia`)
/// or an enabled local/cloud LLM when requested.
pub async fn generate_grandma_website(
    State(state): State<SharedState>,
    Json(req): Json<GrandmaWebsiteRequest>,
) -> Result<Json<GrandmaWebsiteResponse>, (StatusCode, String)> {
    let title = if req.title.trim().is_empty() {
        "Grandma's Cozy Corner".to_string()
    } else {
        req.title.trim().to_string()
    };
    let description = if req.description.trim().is_empty() {
        "A warm, welcoming homemade website filled with favorite family recipes, stories, and photos."
            .to_string()
    } else {
        req.description.trim().to_string()
    };

    let reg = ProvidersRegistryFile::load();
    let requested_provider = req
        .provider_id
        .clone()
        .unwrap_or_else(|| reg.active_provider.clone());

    // Priority #1: Always prefer our own local models if the requested provider is not enabled!
    let effective_provider = if reg.is_provider_enabled_and_authenticated(&requested_provider) {
        requested_provider
    } else {
        "st-sovereign".to_string()
    };

    let effective_model = req.model_id.clone().unwrap_or_else(|| {
        if effective_provider == "st-sovereign" {
            "persona:grandma-jody + in-binary:st-deepseek-pqm-4gaia".to_string()
        } else {
            reg.providers
                .get(&effective_provider)
                .and_then(|p| p.default_model.clone())
                .unwrap_or_else(|| reg.active_model.clone())
        }
    });

    // 1. Recall a warm personal blessing from `persona:grandma-jody` (or synthesize one locally)
    let grandma_note = if let Ok(p) = LifePersona::load("grandma-jody") {
        if !p.episodes.is_empty() {
            "Made with love by Grandma Jody (.m8 Soul) & Smart Tree Local Engine — \"Take your time, sweetheart, every good thing is made by hand and shared from the heart.\"".to_string()
        } else {
            "Made with love by Grandma Jody & Smart Tree Local Engine <3<3<3+1".to_string()
        }
    } else {
        "Made with love by Smart Tree Local Engine <3<3<3+1".to_string()
    };

    // 2. If an external or local LLM provider (like Ollama or an enabled cloud provider) is selected,
    //    try asking it for custom section copy, falling back gracefully to our deterministic local builder.
    let mut ai_custom_copy: Option<String> = None;
    if effective_provider != "st-sovereign" {
        if let Some(copy) = try_generate_custom_copy_via_provider(
            &reg,
            &effective_provider,
            &effective_model,
            &title,
            &description,
        )
        .await
        {
            ai_custom_copy = Some(copy);
        }
    }

    // 3. Build the complete, self-contained, responsive HTML5 website
    let html = build_responsive_website_html(
        &title,
        &description,
        &req.template,
        &req.vibe,
        &grandma_note,
        ai_custom_copy.as_deref(),
        &effective_provider,
        &effective_model,
    );

    let slug = slugify_title(&title);
    let cwd = state.read().await.cwd.clone();
    let site_dir = cwd.join("websites").join(&slug);
    std::fs::create_dir_all(&site_dir)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    let index_path = site_dir.join("index.html");
    std::fs::write(&index_path, &html)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let rel_path = format!("websites/{slug}/index.html");
    let preview_url = format!("/api/website/preview/{slug}");

    Ok(Json(GrandmaWebsiteResponse {
        slug,
        title,
        file_path: index_path.to_string_lossy().to_string(),
        rel_path,
        preview_url,
        provider_used: effective_provider,
        model_used: effective_model,
        grandma_note,
        html,
    }))
}

async fn try_generate_custom_copy_via_provider(
    reg: &ProvidersRegistryFile,
    provider_id: &str,
    model_id: &str,
    title: &str,
    description: &str,
) -> Option<String> {
    let req = LlmRequest {
        model: model_id.to_string(),
        messages: vec![
            LlmMessage {
                role: LlmRole::System,
                content: "Write 2 warm, welcoming paragraphs for the homepage of this website. Plain text only, no markdown fences.".to_string(),
            },
            LlmMessage {
                role: LlmRole::User,
                content: format!("Website Title: {title}\nAbout: {description}"),
            },
        ],
        temperature: Some(0.7),
        max_tokens: Some(280),
        stream: false,
    };

    match provider_id {
        "ollama" => {
            let p = crate::proxy::ollama::OllamaProvider::ollama();
            p.complete(req).await.ok().map(|r| r.content)
        }
        "openrouter" => {
            let key = reg.resolve_credential("openrouter")?;
            let p = crate::proxy::openrouter::OpenRouterProvider::new(key);
            p.complete(req).await.ok().map(|r| r.content)
        }
        "claude" => {
            let key = reg.resolve_credential("claude")?;
            let p = crate::proxy::claude::AnthropicProvider::new(key);
            p.complete(req).await.ok().map(|r| r.content)
        }
        "openai" => {
            let key = reg.resolve_credential("openai")?;
            let p = crate::proxy::openai::OpenAiProvider::new(key);
            p.complete(req).await.ok().map(|r| r.content)
        }
        "google" => {
            let key = reg.resolve_credential("google")?;
            let p = crate::proxy::google::GoogleProvider::new(key);
            p.complete(req).await.ok().map(|r| r.content)
        }
        _ => None,
    }
}

/// `GET /api/website/preview/:slug`
pub async fn preview_website(
    State(state): State<SharedState>,
    Path(slug): Path<String>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let safe_slug = slugify_title(&slug);
    let cwd = state.read().await.cwd.clone();
    let index_path = cwd.join("websites").join(&safe_slug).join("index.html");
    let html = std::fs::read_to_string(&index_path).map_err(|_| {
        (
            StatusCode::NOT_FOUND,
            format!("Website '{}' not found", safe_slug),
        )
    })?;
    Ok((
        [(header::CONTENT_TYPE, "text/html; charset=utf-8")],
        Html(html),
    ))
}

#[allow(clippy::too_many_arguments)]
fn build_responsive_website_html(
    title: &str,
    description: &str,
    template: &str,
    vibe: &str,
    grandma_note: &str,
    ai_custom_copy: Option<&str>,
    provider_used: &str,
    model_used: &str,
) -> String {
    let (bg, surface, fg, accent, accent2, font_heading) = match vibe {
        "sunny-garden" => (
            "#f6fbf4",
            "#ffffff",
            "#1e3528",
            "#2f855a",
            "#d69e2e",
            "'Georgia', serif",
        ),
        "berry-rose" => (
            "#fff7f9",
            "#ffffff",
            "#3b1d2a",
            "#b83280",
            "#dd6b20",
            "'Georgia', serif",
        ),
        "ocean-breeze" => (
            "#f0f8ff",
            "#ffffff",
            "#142d4c",
            "#1f7a8c",
            "#e07a5f",
            "'Trebuchet MS', sans-serif",
        ),
        "qwave-neon" => (
            "#0b111e",
            "#141f36",
            "#e6f4ff",
            "#00d9ff",
            "#00ff88",
            "'Inter', system-ui, sans-serif",
        ),
        _ => (
            "#fdf8f0",
            "#ffffff",
            "#2d221b",
            "#b85d27",
            "#2f7a5e",
            "'Georgia', serif",
        ),
    };

    let (hero_emoji, section_heading, cards) = match template {
        "garden" => (
            "🌻",
            "In Full Bloom This Season",
            vec![
                (
                    "🌷",
                    "Morning Sun Perennials",
                    "Planted along the south fence where the robins gather every April morning.",
                    "Spring & Summer",
                ),
                (
                    "🍅",
                    "Heirloom Kitchen Garden",
                    "Sweet cherry tomatoes, basil, rosemary, and crisp garden cucumbers.",
                    "Harvest Ready",
                ),
                (
                    "🐝",
                    "Pollinator Meadow Patch",
                    "Lavender, echinacea, and wildflowers that keep the honeybees happy all afternoon.",
                    "Bee Friendly",
                ),
            ],
        ),
        "craft" => (
            "🧶",
            "Handmade Treasures & Stitches",
            vec![
                (
                    "🪡",
                    "Heirloom Patchwork Quilts",
                    "Pieced by hand with cotton scraps, warm batting, and a story sewn into every corner.",
                    "Made to Order",
                ),
                (
                    "🧣",
                    "Cozy Winter Knits",
                    "Soft merino scarves, mittens, and baby blankets knitted by the fireplace.",
                    "100% Natural Wool",
                ),
                (
                    "🧺",
                    "Patterns & Workshop Notes",
                    "Step-by-step beginner-friendly guides so anyone in the family can learn to stitch.",
                    "Free Guides",
                ),
            ],
        ),
        "family" => (
            "💌",
            "Family Stories, Photos & Milestones",
            vec![
                (
                    "📖",
                    "Our Family Timeline",
                    "From porch summers and handwritten letters to new adventures across the generations.",
                    "Living Archive",
                ),
                (
                    "🎂",
                    "Upcoming Gatherings",
                    "Sunday potlucks, birthday picnics, and holiday baking weekends.",
                    "RSVP Below",
                ),
                (
                    "🕯️",
                    "Cherished Memories",
                    "Keeping our favorite family sayings, songs, and wisdom safe for the grandkids.",
                    "Forever Saved",
                ),
            ],
        ),
        "project" => (
            "🚀",
            "Built Local, Fast & Sovereign",
            vec![
                (
                    "🌲",
                    "Local-First Architecture",
                    "Powered by Smart Tree's in-binary 4gaia PQM engine and MEM|8 wave memory.",
                    "Zero Cloud Needed",
                ),
                (
                    "⚡",
                    "Instant Browser Experience",
                    "Clean, accessible HTML5 & CSS3 with zero heavy frameworks or tracking scripts.",
                    "100% Yours",
                ),
                (
                    "🤝",
                    "Open & Collaborative",
                    "Easy to edit right inside the Smart Tree Web Dashboard or share with friends.",
                    "Ready to Ship",
                ),
            ],
        ),
        _ => (
            "🍪",
            "Fresh From The Kitchen",
            vec![
                (
                    "🥧",
                    "Cinnamon Lattice Apple Pie",
                    "Crisp honeycrisp apples tossed with nutmeg, brown sugar, and a flaky butter crust.",
                    "45 mins · Serves 8",
                ),
                (
                    "🍪",
                    "Brown-Butter Chocolate Chip Cookies",
                    "Golden edges, soft centers, and a pinch of flaky sea salt right out of the oven.",
                    "12 mins · 24 cookies",
                ),
                (
                    "🍞",
                    "Sunday Morning Banana Bread",
                    "Ripe bananas, toasted walnuts, and warm vanilla — best served warm with butter.",
                    "55 mins · 1 loaf",
                ),
            ],
        ),
    };

    let extra_story = ai_custom_copy.unwrap_or(description);

    let mut cards_html = String::new();
    for (icon, ctitle, cbody, ctag) in cards {
        cards_html.push_str(&format!(
            r#"<article class="card">
                <div class="card-icon">{icon}</div>
                <span class="card-tag">{ctag}</span>
                <h3>{ctitle}</h3>
                <p>{cbody}</p>
            </article>"#
        ));
    }

    format!(
        r##"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>{title}</title>
    <style>
        :root {{
            --bg: {bg};
            --surface: {surface};
            --fg: {fg};
            --accent: {accent};
            --accent2: {accent2};
            --font-heading: {font_heading};
        }}
        * {{ box-sizing: border-box; margin: 0; padding: 0; }}
        body {{
            background: var(--bg);
            color: var(--fg);
            font-family: system-ui, -apple-system, 'Segoe UI', Roboto, sans-serif;
            line-height: 1.65;
            font-size: 18px;
        }}
        header.site-nav {{
            background: var(--surface);
            border-bottom: 2px solid rgba(0,0,0,0.08);
            padding: 18px 32px;
            display: flex;
            justify-content: space-between;
            align-items: center;
            flex-wrap: wrap;
            gap: 12px;
            position: sticky;
            top: 0;
            z-index: 10;
        }}
        .brand {{
            font-family: var(--font-heading);
            font-size: 1.45rem;
            font-weight: 700;
            color: var(--accent);
            display: flex;
            align-items: center;
            gap: 10px;
        }}
        .nav-links {{ display: flex; gap: 20px; flex-wrap: wrap; }}
        .nav-links a {{
            color: var(--fg);
            text-decoration: none;
            font-weight: 600;
            padding: 6px 12px;
            border-radius: 999px;
            transition: background 0.2s;
        }}
        .nav-links a:hover {{ background: rgba(128,128,128,0.12); }}
        .hero {{
            max-width: 960px;
            margin: 42px auto 28px;
            padding: 48px 36px;
            background: var(--surface);
            border-radius: 24px;
            box-shadow: 0 12px 32px rgba(0,0,0,0.07);
            text-align: center;
            border: 2px solid rgba(128,128,128,0.12);
        }}
        .hero-emoji {{ font-size: 3.6rem; margin-bottom: 12px; }}
        .hero h1 {{
            font-family: var(--font-heading);
            font-size: clamp(2rem, 4vw, 3rem);
            color: var(--accent);
            margin-bottom: 14px;
        }}
        .hero p.lead {{
            font-size: 1.2rem;
            max-width: 680px;
            margin: 0 auto 24px;
            opacity: 0.92;
        }}
        .hero-btn {{
            display: inline-block;
            background: var(--accent);
            color: #fff;
            font-weight: 700;
            font-size: 1.05rem;
            padding: 14px 28px;
            border-radius: 999px;
            text-decoration: none;
            border: none;
            cursor: pointer;
            box-shadow: 0 4px 14px rgba(0,0,0,0.15);
        }}
        .section {{
            max-width: 960px;
            margin: 36px auto;
            padding: 0 24px;
        }}
        .section-title {{
            font-family: var(--font-heading);
            font-size: 1.85rem;
            text-align: center;
            margin-bottom: 24px;
            color: var(--accent);
        }}
        .grid {{
            display: grid;
            grid-template-columns: repeat(auto-fit, minmax(260px, 1fr));
            gap: 22px;
        }}
        .card {{
            background: var(--surface);
            padding: 28px;
            border-radius: 20px;
            box-shadow: 0 8px 24px rgba(0,0,0,0.05);
            border: 1px solid rgba(128,128,128,0.15);
            display: flex;
            flex-direction: column;
            gap: 10px;
        }}
        .card-icon {{ font-size: 2.3rem; }}
        .card-tag {{
            align-self: flex-start;
            background: var(--accent2);
            color: #fff;
            font-size: 0.78rem;
            font-weight: 700;
            padding: 4px 10px;
            border-radius: 999px;
            letter-spacing: 0.03em;
        }}
        .card h3 {{ font-family: var(--font-heading); font-size: 1.35rem; }}
        .story-box {{
            background: var(--surface);
            padding: 32px;
            border-radius: 20px;
            border-left: 6px solid var(--accent);
            box-shadow: 0 6px 20px rgba(0,0,0,0.05);
        }}
        .guestbook {{
            background: var(--surface);
            padding: 32px;
            border-radius: 20px;
            box-shadow: 0 6px 20px rgba(0,0,0,0.05);
            display: flex;
            flex-direction: column;
            gap: 14px;
        }}
        .guestbook input, .guestbook textarea {{
            width: 100%;
            padding: 12px 16px;
            border-radius: 12px;
            border: 1px solid rgba(128,128,128,0.35);
            font-size: 1rem;
            font-family: inherit;
        }}
        .guest-entries {{ margin-top: 14px; display: flex; flex-direction: column; gap: 10px; }}
        .guest-entry {{
            padding: 12px 16px;
            border-radius: 12px;
            background: rgba(128,128,128,0.08);
        }}
        footer {{
            text-align: center;
            padding: 40px 20px;
            opacity: 0.85;
            font-size: 0.95rem;
        }}
    </style>
</head>
<body>
    <header class="site-nav">
        <div class="brand"><span>{hero_emoji}</span> <span>{title}</span></div>
        <nav class="nav-links">
            <a href="#featured">Highlights</a>
            <a href="#story">Our Story</a>
            <a href="#guestbook">Leave a Note</a>
            <a href="javascript:window.print()">🖨️ Print Page</a>
        </nav>
    </header>

    <section class="hero">
        <div class="hero-emoji">{hero_emoji}</div>
        <h1>{title}</h1>
        <p class="lead">{description}</p>
        <a class="hero-btn" href="#featured">Come On In &amp; Explore ↓</a>
    </section>

    <section class="section" id="featured">
        <h2 class="section-title">{section_heading}</h2>
        <div class="grid">
            {cards_html}
        </div>
    </section>

    <section class="section" id="story">
        <h2 class="section-title">From Our Heart to Yours</h2>
        <div class="story-box">
            <p style="margin-bottom: 14px;">{extra_story}</p>
            <p style="font-style: italic; opacity: 0.9;">{grandma_note}</p>
        </div>
    </section>

    <section class="section" id="guestbook">
        <h2 class="section-title">Sign Our Family Guestbook ✍️</h2>
        <div class="guestbook">
            <input type="text" id="gbName" placeholder="Your name (e.g., Aunt Sarah)" />
            <textarea id="gbMsg" rows="3" placeholder="Write a warm hello or share a favorite memory..."></textarea>
            <button class="hero-btn" type="button" onclick="addGuestbookNote()">Post My Note 💌</button>
            <div class="guest-entries" id="gbEntries">
                <div class="guest-entry"><strong>Grandma Jody:</strong> Welcome to our brand-new website! Pull up a chair and make yourself at home. ❤️</div>
            </div>
        </div>
    </section>

    <footer>
        <p>{grandma_note}</p>
        <p style="margin-top:6px;font-size:0.82rem;">Built in 1 click with Smart Tree Local Engine (<code>{provider_used}</code> · <code>{model_used}</code>)</p>
    </footer>

    <script>
        function addGuestbookNote() {{
            var name = document.getElementById('gbName').value.trim() || 'A Friend';
            var msg = document.getElementById('gbMsg').value.trim();
            if (!msg) return;
            var box = document.getElementById('gbEntries');
            var div = document.createElement('div');
            div.className = 'guest-entry';
            var strong = document.createElement('strong');
            strong.textContent = name + ': ';
            div.appendChild(strong);
            div.appendChild(document.createTextNode(msg));
            box.prepend(div);
            document.getElementById('gbMsg').value = '';
        }}
    </script>
</body>
</html>"##
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn verify_cloud_providers_default_to_off_and_local_models_are_priority_one() {
        let reg = ProvidersRegistryFile::default();
        assert_eq!(reg.active_provider, "st-sovereign");
        assert_eq!(reg.active_model, "in-binary:st-deepseek-pqm-4gaia");

        // Local models must be ON by default
        assert!(reg.is_provider_enabled_and_authenticated("st-sovereign"));
        assert!(reg.is_provider_enabled_and_authenticated("ollama"));
        assert!(reg.is_provider_enabled_and_authenticated("lmstudio"));
        assert!(reg.is_provider_enabled_and_authenticated("llamacpp"));
        assert!(reg.is_provider_enabled_and_authenticated("vllm"));

        // All cloud providers MUST be OFF by default until OAuth or API key is configured!
        for cloud in [
            "openrouter",
            "claude",
            "openai",
            "google",
            "grok",
            "github-copilot",
            "kimi",
            "devin",
            "meta",
            "zai",
        ] {
            assert!(
                !reg.is_provider_enabled_and_authenticated(cloud),
                "Cloud provider {cloud} must be OFF by default!"
            );
        }
    }

    #[test]
    fn verify_grandma_website_builder_produces_complete_html() {
        let html = build_responsive_website_html(
            "Grandma Jody's Cookie Kitchen",
            "Our favorite family recipes and Sunday stories.",
            "recipe",
            "cozy-cream",
            "Made with love <3<3<3+1",
            None,
            "st-sovereign",
            "persona:grandma-jody",
        );
        assert!(html.contains("<!DOCTYPE html>"));
        assert!(html.contains("Grandma Jody's Cookie Kitchen"));
        assert!(html.contains("Sign Our Family Guestbook"));
        assert!(html.contains("st-sovereign"));
    }
}
