//! 🌐 LLM Proxy - Unified interface for multiple LLM providers
//!
//! This module provides a unified proxy for calling various LLMs,
//! including OpenAI, Anthropic, Google Gemini, and local Candle-based models.
//!
//! "Why talk to one AI when you can talk to them all?" - The Cheet 😺

use anyhow::Result;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};

pub mod burn;
pub mod caveman;
pub mod claude;
// Re-export for backward compatibility - `proxy::anthropic::AnthropicProvider` still works
pub use claude as anthropic;
pub mod google;
pub mod grok;
pub mod memory;
pub mod oauth;
pub mod ollama;
pub mod openai;
pub mod openai_compat;
pub mod openrouter;
pub mod server;
pub mod token_store;
pub mod zai;

/// 🤖 Common interface for all LLM providers
#[async_trait]
pub trait LlmProvider: Send + Sync {
    /// Send a prompt to the LLM and get a response
    async fn complete(&self, request: LlmRequest) -> Result<LlmResponse>;

    /// Get the provider name
    fn name(&self) -> &'static str;
}

/// 📝 Request structure for LLM completion
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LlmRequest {
    pub model: String,
    pub messages: Vec<LlmMessage>,
    pub temperature: Option<f32>,
    pub max_tokens: Option<usize>,
    pub stream: bool,
}

/// 💬 A single message in a conversation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LlmMessage {
    pub role: LlmRole,
    pub content: String,
}

/// 🎭 Roles in a conversation
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum LlmRole {
    System,
    User,
    Assistant,
}

/// 📦 Response structure from LLM completion
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LlmResponse {
    pub content: String,
    pub model: String,
    pub usage: Option<LlmUsage>,
}

/// 📊 Token usage statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LlmUsage {
    pub prompt_tokens: usize,
    pub completion_tokens: usize,
    pub total_tokens: usize,
}

/// 🛠️ Factory for creating LLM providers
pub struct LlmProxy {
    pub providers: Vec<Box<dyn LlmProvider>>,
}

impl LlmProxy {
    pub fn new() -> Self {
        Self {
            providers: Vec::new(),
        }
    }

    pub fn add_provider(&mut self, provider: Box<dyn LlmProvider>) {
        self.providers.push(provider);
    }

    pub async fn complete(&self, provider_name: &str, request: LlmRequest) -> Result<LlmResponse> {
        for provider in &self.providers {
            if provider.name().to_lowercase() == provider_name.to_lowercase() {
                return provider.complete(request).await;
            }
        }
        Err(anyhow::anyhow!("Provider '{}' not found", provider_name))
    }
}

impl LlmProxy {
    /// Create a proxy with auto-detection of local LLMs (Ollama, LM Studio, Caveman + Kompress)
    /// This is async because it needs to probe local ports concurrently.
    pub async fn with_local_detection() -> Self {
        let mut proxy = Self::default();
        let caveman_probe = caveman::CavemanProvider::new();

        // Concurrently probe local Ollama/LM Studio and Caveman/Kompress-Super-Ultra (:8787 / :8799)
        let (local_llms, (caveman_alive, kompress_alive)) =
            tokio::join!(ollama::detect_local_llms(), caveman_probe.unified_health());

        if caveman_alive || kompress_alive {
            eprintln!(
                "🪨 Detected Sovereign Local Gateway (caveman:8787={}, kompress-super-ultra:8799={})",
                caveman_alive, kompress_alive
            );
            proxy.add_provider(Box::new(caveman_probe));
        }

        for info in local_llms {
            match info.server_type {
                ollama::LocalLlmType::Ollama => {
                    eprintln!("🦙 Detected Ollama with {} model(s)", info.models.len());
                    proxy.add_provider(Box::new(ollama::OllamaProvider::ollama()));
                }
                ollama::LocalLlmType::LmStudio => {
                    eprintln!("🖥️  Detected LM Studio with {} model(s)", info.models.len());
                    proxy.add_provider(Box::new(ollama::OllamaProvider::lmstudio()));
                }
            }
        }

        proxy
    }

    /// List all available providers
    pub fn list_providers(&self) -> Vec<&'static str> {
        self.providers.iter().map(|p| p.name()).collect()
    }
}

impl Default for LlmProxy {
    fn default() -> Self {
        let mut proxy = Self::new();

        // Priority #1: Always add Smart Tree's own local ternary & MEM8 provider first!
        proxy.add_provider(Box::new(burn::BurnProvider::default()));

        // Cloud providers are strictly OFF unless enabled & authenticated in `~/.st/providers.json`
        let reg = crate::web_dashboard::providers::ProvidersRegistryFile::load();

        if let Some(key) = reg.resolve_credential("openrouter") {
            proxy.add_provider(Box::new(openrouter::OpenRouterProvider::new(key)));
        }

        if let Some(key) = reg.resolve_credential("claude") {
            proxy.add_provider(Box::new(anthropic::AnthropicProvider::new(key)));
        }

        if let Some(key) = reg.resolve_credential("openai") {
            proxy.add_provider(Box::new(openai::OpenAiProvider::new(key)));
        }

        if let Some(key) = reg.resolve_credential("google") {
            proxy.add_provider(Box::new(google::GoogleProvider::new(key)));
        }

        if let Some(key) = reg.resolve_credential("grok") {
            proxy.add_provider(Box::new(grok::GrokProvider::new(key)));
        }

        if let Some(key) = reg.resolve_credential("zai") {
            proxy.add_provider(Box::new(zai::ZaiProvider::new(key)));
        }

        proxy
    }
}
