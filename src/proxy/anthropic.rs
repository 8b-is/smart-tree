//! 🤖 Anthropic Provider Implementation
//!
//! "Harnessing the power of Claude for smart directory analysis!" - The Cheet 😺

use crate::proxy::{LlmMessage, LlmProvider, LlmRequest, LlmResponse, LlmRole, LlmUsage};
use anyhow::{Context, Result};
use async_trait::async_trait;
use reqwest::Client;
use serde::{Deserialize, Serialize};

pub struct AnthropicProvider {
    client: Client,
    api_key: String,
    base_url: String,
}

impl AnthropicProvider {
    pub fn new(api_key: String) -> Self {
        Self {
            client: Client::new(),
            api_key,
            base_url: "https://api.anthropic.com/v1".to_string(),
        }
    }
}

impl Default for AnthropicProvider {
    fn default() -> Self {
        let api_key = std::env::var("ANTHROPIC_API_KEY").unwrap_or_default();
        Self::new(api_key)
    }
}

#[async_trait]
impl LlmProvider for AnthropicProvider {
    async fn complete(&self, request: LlmRequest) -> Result<LlmResponse> {
        let url = format!("{}/messages", self.base_url);

        // Extract system message if present
        let mut system = None;
        let mut messages = Vec::new();

        for msg in request.messages {
            if msg.role == LlmRole::System {
                system = Some(msg.content);
            } else {
                messages.push(AnthropicMessage::from(msg));
            }
        }

        let anthropic_request = AnthropicChatRequest {
            model: request.model.clone(),
            messages,
            system,
            max_tokens: request.max_tokens.unwrap_or(1024),
            temperature: request.temperature,
            stream: request.stream,
        };

        let response = self
            .client
            .post(&url)
            .header("x-api-key", &self.api_key)
            .header("anthropic-version", "2023-06-01")
            .json(&anthropic_request)
            .send()
            .await
            .context("Failed to send request to Anthropic")?;

        if !response.status().is_success() {
            let error_text = response.text().await?;
            return Err(anyhow::anyhow!("Anthropic API error: {}", error_text));
        }

        let anthropic_response: AnthropicChatResponse = response.json().await?;

        let content = anthropic_response
            .content
            .first()
            .map(|c| c.text.clone())
            .unwrap_or_default();

        Ok(LlmResponse {
            content,
            model: anthropic_response.model,
            usage: anthropic_response.usage.map(Into::into),
        })
    }

    fn name(&self) -> &'static str {
        "Anthropic"
    }
}

#[derive(Debug, Serialize)]
struct AnthropicChatRequest {
    model: String,
    messages: Vec<AnthropicMessage>,
    #[serde(skip_serializing_if = "Option::is_none")]
    system: Option<String>,
    max_tokens: usize,
    #[serde(skip_serializing_if = "Option::is_none")]
    temperature: Option<f32>,
    stream: bool,
}

#[derive(Debug, Serialize, Deserialize)]
struct AnthropicMessage {
    role: String,
    content: String,
}

impl From<LlmMessage> for AnthropicMessage {
    fn from(msg: LlmMessage) -> Self {
        Self {
            role: match msg.role {
                LlmRole::System => "user".to_string(), // Anthropic handles system separately
                LlmRole::User => "user".to_string(),
                LlmRole::Assistant => "assistant".to_string(),
            },
            content: msg.content,
        }
    }
}

#[derive(Debug, Deserialize)]
struct AnthropicChatResponse {
    model: String,
    content: Vec<AnthropicContent>,
    usage: Option<AnthropicUsage>,
}

#[derive(Debug, Deserialize)]
struct AnthropicContent {
    text: String,
}

#[derive(Debug, Deserialize)]
struct AnthropicUsage {
    input_tokens: usize,
    output_tokens: usize,
}

impl From<AnthropicUsage> for LlmUsage {
    fn from(usage: AnthropicUsage) -> Self {
        Self {
            prompt_tokens: usage.input_tokens,
            completion_tokens: usage.output_tokens,
            total_tokens: usage.input_tokens + usage.output_tokens,
        }
    }
}
