//! 🤖 OpenAI Provider Implementation
//!
//! "Connecting to the mothership of modern LLMs!" - The Cheet 😺

use crate::proxy::{LlmMessage, LlmProvider, LlmRequest, LlmResponse, LlmRole, LlmUsage};
use anyhow::{Context, Result};
use async_trait::async_trait;
use reqwest::Client;
use serde::{Deserialize, Serialize};

pub struct OpenAiProvider {
    client: Client,
    api_key: String,
    base_url: String,
}

impl OpenAiProvider {
    pub fn new(api_key: String) -> Self {
        Self {
            client: Client::new(),
            api_key,
            base_url: "https://api.openai.com/v1".to_string(),
        }
    }
}

impl Default for OpenAiProvider {
    fn default() -> Self {
        let api_key = std::env::var("OPENAI_API_KEY").unwrap_or_default();
        Self::new(api_key)
    }
}

#[async_trait]
impl LlmProvider for OpenAiProvider {
    async fn complete(&self, request: LlmRequest) -> Result<LlmResponse> {
        let url = format!("{}/chat/completions", self.base_url);

        let openai_request = OpenAiChatRequest {
            model: request.model.clone(),
            messages: request.messages.into_iter().map(Into::into).collect(),
            temperature: request.temperature,
            max_tokens: request.max_tokens,
            stream: request.stream,
        };

        let response = self
            .client
            .post(&url)
            .header("Authorization", format!("Bearer {}", self.api_key))
            .json(&openai_request)
            .send()
            .await
            .context("Failed to send request to OpenAI")?;

        if !response.status().is_success() {
            let error_text = response.text().await?;
            return Err(anyhow::anyhow!("OpenAI API error: {}", error_text));
        }

        let openai_response: OpenAiChatResponse = response.json().await?;

        let content = openai_response
            .choices
            .first()
            .map(|c| c.message.content.clone())
            .unwrap_or_default();

        Ok(LlmResponse {
            content,
            model: openai_response.model,
            usage: openai_response.usage.map(Into::into),
        })
    }

    fn name(&self) -> &'static str {
        "OpenAI"
    }
}

#[derive(Debug, Serialize)]
struct OpenAiChatRequest {
    model: String,
    messages: Vec<OpenAiMessage>,
    #[serde(skip_serializing_if = "Option::is_none")]
    temperature: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    max_tokens: Option<usize>,
    stream: bool,
}

#[derive(Debug, Serialize, Deserialize)]
struct OpenAiMessage {
    role: String,
    content: String,
}

impl From<LlmMessage> for OpenAiMessage {
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
struct OpenAiChatResponse {
    model: String,
    choices: Vec<OpenAiChoice>,
    usage: Option<OpenAiUsage>,
}

#[derive(Debug, Deserialize)]
struct OpenAiChoice {
    message: OpenAiMessage,
}

#[derive(Debug, Deserialize)]
struct OpenAiUsage {
    prompt_tokens: usize,
    completion_tokens: usize,
    total_tokens: usize,
}

impl From<OpenAiUsage> for LlmUsage {
    fn from(usage: OpenAiUsage) -> Self {
        Self {
            prompt_tokens: usage.prompt_tokens,
            completion_tokens: usage.completion_tokens,
            total_tokens: usage.total_tokens,
        }
    }
}
