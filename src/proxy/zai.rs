//! Z.AI Provider Implementation (Zhipu / GLM family)
//!
//! Z.AI exposes an OpenAI-compatible endpoint at https://open.bigmodel.cn/api/paas/v4
//! Models: glm-4-plus, glm-4.7, glm-4.6, glm-4-air, glm-4-flash, etc.

use crate::proxy::{LlmMessage, LlmProvider, LlmRequest, LlmResponse, LlmRole, LlmUsage};
use anyhow::{Context, Result};
use async_trait::async_trait;
use reqwest::Client;
use serde::{Deserialize, Serialize};

pub struct ZaiProvider {
    client: Client,
    api_key: String,
    base_url: String,
}

impl ZaiProvider {
    pub fn new(api_key: String) -> Self {
        Self {
            client: Client::new(),
            api_key,
            base_url: "https://open.bigmodel.cn/api/paas/v4".to_string(),
        }
    }

    pub fn with_base_url(api_key: String, base_url: String) -> Self {
        Self {
            client: Client::new(),
            api_key,
            base_url,
        }
    }
}

impl Default for ZaiProvider {
    fn default() -> Self {
        let api_key = std::env::var("ZAI_API_KEY")
            .or_else(|_| std::env::var("ZHIPU_API_KEY"))
            .unwrap_or_default();
        Self::new(api_key)
    }
}

#[async_trait]
impl LlmProvider for ZaiProvider {
    async fn complete(&self, request: LlmRequest) -> Result<LlmResponse> {
        let url = format!("{}/chat/completions", self.base_url);

        let model = if request.model.is_empty() || request.model == "default" {
            "glm-4-plus".to_string()
        } else {
            request.model.clone()
        };

        let zai_request = ZaiChatRequest {
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
            .json(&zai_request)
            .send()
            .await
            .context("Failed to send request to Z.AI")?;

        if !response.status().is_success() {
            let error_text = response.text().await?;
            return Err(anyhow::anyhow!("Z.AI API error: {}", error_text));
        }

        let zai_response: ZaiChatResponse = response.json().await?;

        let content = zai_response
            .choices
            .first()
            .map(|c| c.message.content.clone())
            .unwrap_or_default();

        Ok(LlmResponse {
            content,
            model: zai_response.model.unwrap_or_else(|| "glm".to_string()),
            usage: zai_response.usage.map(Into::into),
        })
    }

    fn name(&self) -> &'static str {
        "ZAI"
    }
}

#[derive(Debug, Serialize)]
struct ZaiChatRequest {
    model: String,
    messages: Vec<ZaiMessage>,
    #[serde(skip_serializing_if = "Option::is_none")]
    temperature: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    max_tokens: Option<usize>,
    stream: bool,
}

#[derive(Debug, Serialize, Deserialize)]
struct ZaiMessage {
    role: String,
    content: String,
}

impl From<LlmMessage> for ZaiMessage {
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
struct ZaiChatResponse {
    model: Option<String>,
    choices: Vec<ZaiChoice>,
    usage: Option<ZaiUsage>,
}

#[derive(Debug, Deserialize)]
struct ZaiChoice {
    message: ZaiMessage,
}

#[derive(Debug, Deserialize)]
struct ZaiUsage {
    prompt_tokens: usize,
    completion_tokens: usize,
    total_tokens: usize,
}

impl From<ZaiUsage> for LlmUsage {
    fn from(u: ZaiUsage) -> Self {
        Self {
            prompt_tokens: u.prompt_tokens,
            completion_tokens: u.completion_tokens,
            total_tokens: u.total_tokens,
        }
    }
}

pub mod models {
    pub const GLM_4_PLUS: &str = "glm-4-plus";
    pub const GLM_4_7: &str = "glm-4.7";
    pub const GLM_4_6: &str = "glm-4.6";
    pub const GLM_4_AIR: &str = "glm-4-air";
    pub const GLM_4_FLASH: &str = "glm-4-flash";
}
