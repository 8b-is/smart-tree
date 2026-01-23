//! 🤖 Grok Provider Implementation (X.AI)
//!
//! "Elon's AI enters the chat!" - The Cheet 😺
//!
//! Grok uses an OpenAI-compatible API at https://api.x.ai/v1

use crate::proxy::{LlmMessage, LlmProvider, LlmRequest, LlmResponse, LlmRole, LlmUsage};
use anyhow::{Context, Result};
use async_trait::async_trait;
use reqwest::Client;
use serde::{Deserialize, Serialize};

pub struct GrokProvider {
    client: Client,
    api_key: String,
    base_url: String,
}

impl GrokProvider {
    pub fn new(api_key: String) -> Self {
        Self {
            client: Client::new(),
            api_key,
            base_url: "https://api.x.ai/v1".to_string(),
        }
    }

    /// Create with custom base URL (for testing or proxies)
    pub fn with_base_url(api_key: String, base_url: String) -> Self {
        Self {
            client: Client::new(),
            api_key,
            base_url,
        }
    }
}

impl Default for GrokProvider {
    fn default() -> Self {
        let api_key = std::env::var("XAI_API_KEY")
            .or_else(|_| std::env::var("GROK_API_KEY"))
            .unwrap_or_default();
        Self::new(api_key)
    }
}

#[async_trait]
impl LlmProvider for GrokProvider {
    async fn complete(&self, request: LlmRequest) -> Result<LlmResponse> {
        let url = format!("{}/chat/completions", self.base_url);

        // Default to grok-beta if no model specified
        let model = if request.model.is_empty() || request.model == "default" {
            "grok-beta".to_string()
        } else {
            request.model.clone()
        };

        let grok_request = GrokChatRequest {
            model,
            messages: request.messages.into_iter().map(Into::into).collect(),
            temperature: request.temperature,
            max_tokens: request.max_tokens,
            stream: request.stream,
        };

        let response = self
            .client
            .post(&url)
            .header("Authorization", format!("Bearer {}", self.api_key))
            .header("Content-Type", "application/json")
            .json(&grok_request)
            .send()
            .await
            .context("Failed to send request to Grok API")?;

        if !response.status().is_success() {
            let error_text = response.text().await?;
            return Err(anyhow::anyhow!("Grok API error: {}", error_text));
        }

        let grok_response: GrokChatResponse = response.json().await?;

        let content = grok_response
            .choices
            .first()
            .map(|c| c.message.content.clone())
            .unwrap_or_default();

        Ok(LlmResponse {
            content,
            model: grok_response.model,
            usage: grok_response.usage.map(Into::into),
        })
    }

    fn name(&self) -> &'static str {
        "Grok"
    }
}

#[derive(Debug, Serialize)]
struct GrokChatRequest {
    model: String,
    messages: Vec<GrokMessage>,
    #[serde(skip_serializing_if = "Option::is_none")]
    temperature: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    max_tokens: Option<usize>,
    stream: bool,
}

#[derive(Debug, Serialize, Deserialize)]
struct GrokMessage {
    role: String,
    content: String,
}

impl From<LlmMessage> for GrokMessage {
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
struct GrokChatResponse {
    model: String,
    choices: Vec<GrokChoice>,
    usage: Option<GrokUsage>,
}

#[derive(Debug, Deserialize)]
struct GrokChoice {
    message: GrokMessage,
}

#[derive(Debug, Deserialize)]
struct GrokUsage {
    prompt_tokens: usize,
    completion_tokens: usize,
    total_tokens: usize,
}

impl From<GrokUsage> for LlmUsage {
    fn from(usage: GrokUsage) -> Self {
        Self {
            prompt_tokens: usage.prompt_tokens,
            completion_tokens: usage.completion_tokens,
            total_tokens: usage.total_tokens,
        }
    }
}
