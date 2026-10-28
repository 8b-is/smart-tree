//! 🌐 OpenRouter Provider Implementation
//!
//! "One API to rule them all!" - The Cheet 😺
//!
//! OpenRouter provides unified access to 100+ LLM models via OpenAI-compatible API
//! https://openrouter.ai/docs

use crate::proxy::{LlmMessage, LlmProvider, LlmRequest, LlmResponse, LlmRole, LlmUsage};
use anyhow::{Context, Result};
use async_trait::async_trait;
use reqwest::Client;
use serde::{Deserialize, Serialize};

pub struct OpenRouterProvider {
    client: Client,
    api_key: String,
    base_url: String,
    /// Optional site URL for OpenRouter analytics
    site_url: Option<String>,
    /// Optional app name for OpenRouter analytics
    app_name: Option<String>,
}

impl OpenRouterProvider {
    pub fn new(api_key: String) -> Self {
        Self {
            client: Client::new(),
            api_key,
            base_url: "https://openrouter.ai/api/v1".to_string(),
            site_url: Some("https://github.com/8b-is/smart-tree".to_string()),
            app_name: Some("Smart Tree".to_string()),
        }
    }

    /// Create with custom configuration
    pub fn with_config(
        api_key: String,
        site_url: Option<String>,
        app_name: Option<String>,
    ) -> Self {
        Self {
            client: Client::new(),
            api_key,
            base_url: "https://openrouter.ai/api/v1".to_string(),
            site_url,
            app_name,
        }
    }
}

impl Default for OpenRouterProvider {
    fn default() -> Self {
        let api_key = std::env::var("OPENROUTER_API_KEY").unwrap_or_default();
        Self::new(api_key)
    }
}

#[async_trait]
impl LlmProvider for OpenRouterProvider {
    async fn complete(&self, request: LlmRequest) -> Result<LlmResponse> {
        let url = format!("{}/chat/completions", self.base_url);

        // Default to a good free/cheap model if none specified
        let model = if request.model.is_empty() || request.model == "default" {
            "anthropic/claude-3-haiku".to_string()
        } else {
            request.model.clone()
        };

        let openrouter_request = OpenRouterChatRequest {
            model,
            messages: request.messages.into_iter().map(Into::into).collect(),
            temperature: request.temperature,
            max_tokens: request.max_tokens,
            stream: request.stream,
        };

        let mut req = self
            .client
            .post(&url)
            .header("Authorization", format!("Bearer {}", self.api_key))
            .header("Content-Type", "application/json");

        // Add optional OpenRouter headers
        if let Some(site) = &self.site_url {
            req = req.header("HTTP-Referer", site);
        }
        if let Some(app) = &self.app_name {
            req = req.header("X-Title", app);
        }

        let response = req
            .json(&openrouter_request)
            .send()
            .await
            .context("Failed to send request to OpenRouter")?;

        if !response.status().is_success() {
            let error_text = response.text().await?;
            return Err(anyhow::anyhow!("OpenRouter API error: {}", error_text));
        }

        let openrouter_response: OpenRouterChatResponse = response.json().await?;

        let content = openrouter_response
            .choices
            .first()
            .map(|c| c.message.content.clone())
            .unwrap_or_default();

        Ok(LlmResponse {
            content,
            model: openrouter_response
                .model
                .unwrap_or_else(|| "unknown".to_string()),
            usage: openrouter_response.usage.map(Into::into),
        })
    }

    fn name(&self) -> &'static str {
        "OpenRouter"
    }
}

#[derive(Debug, Serialize)]
struct OpenRouterChatRequest {
    model: String,
    messages: Vec<OpenRouterMessage>,
    #[serde(skip_serializing_if = "Option::is_none")]
    temperature: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    max_tokens: Option<usize>,
    stream: bool,
}

#[derive(Debug, Serialize, Deserialize)]
struct OpenRouterMessage {
    role: String,
    content: String,
}

impl From<LlmMessage> for OpenRouterMessage {
    fn from(msg: LlmMessage) -> Self {
        Self {
            role: match msg.role {
                LlmRole::System => "system".to_string(),
                LlmRole::User => "user".to_string(),
                LlmRole::Assistant => "assistant".to_string(),
            },
            content: msg.content,
        }
    }
}

#[derive(Debug, Deserialize)]
struct OpenRouterChatResponse {
    model: Option<String>,
    choices: Vec<OpenRouterChoice>,
    usage: Option<OpenRouterUsage>,
}

#[derive(Debug, Deserialize)]
struct OpenRouterChoice {
    message: OpenRouterMessage,
}

#[derive(Debug, Deserialize)]
struct OpenRouterUsage {
    prompt_tokens: usize,
    completion_tokens: usize,
    total_tokens: usize,
}

impl From<OpenRouterUsage> for LlmUsage {
    fn from(usage: OpenRouterUsage) -> Self {
        Self {
            prompt_tokens: usage.prompt_tokens,
            completion_tokens: usage.completion_tokens,
            total_tokens: usage.total_tokens,
        }
    }
}

/// Popular OpenRouter models for quick access
pub mod models {
    pub const CLAUDE_3_OPUS: &str = "anthropic/claude-3-opus";
    pub const CLAUDE_3_SONNET: &str = "anthropic/claude-3-sonnet";
    pub const CLAUDE_3_HAIKU: &str = "anthropic/claude-3-haiku";
    pub const GPT_4_TURBO: &str = "openai/gpt-4-turbo";
    pub const GPT_4O: &str = "openai/gpt-4o";
    pub const GPT_4O_MINI: &str = "openai/gpt-4o-mini";
    pub const LLAMA_3_70B: &str = "meta-llama/llama-3-70b-instruct";
    pub const MIXTRAL_8X7B: &str = "mistralai/mixtral-8x7b-instruct";
    pub const GEMINI_PRO: &str = "google/gemini-pro";
    pub const DEEPSEEK_CODER: &str = "deepseek/deepseek-coder";
    pub const QWEN_72B: &str = "qwen/qwen-72b-chat";
}
