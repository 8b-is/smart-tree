//! 🌐 LLM Proxy - Unified interface for multiple LLM providers
//!
//! This module provides a unified proxy for calling various LLMs,
//! including OpenAI, Anthropic, Google Gemini, and local Candle-based models.
//!
//! "Why talk to one AI when you can talk to them all?" - The Cheet 😺

use anyhow::Result;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};

pub mod anthropic;
pub mod candle;
pub mod google;
pub mod grok;
pub mod memory;
pub mod openai;
pub mod openai_compat;
pub mod openrouter;
pub mod server;

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

impl Default for LlmProxy {
    fn default() -> Self {
        let mut proxy = Self::new();

        // Add default providers if API keys are present in environment
        if std::env::var("OPENAI_API_KEY").is_ok() {
            proxy.add_provider(Box::new(openai::OpenAiProvider::default()));
        }

        if std::env::var("ANTHROPIC_API_KEY").is_ok() {
            proxy.add_provider(Box::new(anthropic::AnthropicProvider::default()));
        }

        if std::env::var("GOOGLE_API_KEY").is_ok() {
            proxy.add_provider(Box::new(google::GoogleProvider::default()));
        }

        // Add Grok provider if XAI_API_KEY or GROK_API_KEY is present
        if std::env::var("XAI_API_KEY").is_ok() || std::env::var("GROK_API_KEY").is_ok() {
            proxy.add_provider(Box::new(grok::GrokProvider::default()));
        }

        // Add OpenRouter provider if OPENROUTER_API_KEY is present (access to 100+ models!)
        if std::env::var("OPENROUTER_API_KEY").is_ok() {
            proxy.add_provider(Box::new(openrouter::OpenRouterProvider::default()));
        }

        // Always add Candle provider (it will check for feature at runtime/compile time)
        proxy.add_provider(Box::new(candle::CandleProvider::default()));

        proxy
    }
}
