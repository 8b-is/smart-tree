//! 🤖 Google Gemini Provider Implementation
//!
//! "Expanding our horizons with Google's Gemini!" - The Cheet 😺

use crate::proxy::{LlmMessage, LlmProvider, LlmRequest, LlmResponse, LlmRole, LlmUsage};
use anyhow::{Context, Result};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use reqwest::Client;

pub struct GoogleProvider {
    client: Client,
    api_key: String,
    base_url: String,
}

impl GoogleProvider {
    pub fn new(api_key: String) -> Self {
        Self {
            client: Client::new(),
            api_key,
            base_url: "https://generativelanguage.googleapis.com/v1beta".to_string(),
        }
    }
}

impl Default for GoogleProvider {
    fn default() -> Self {
        let api_key = std::env::var("GOOGLE_API_KEY").unwrap_or_default();
        Self::new(api_key)
    }
}

#[async_trait]
impl LlmProvider for GoogleProvider {
    async fn complete(&self, request: LlmRequest) -> Result<LlmResponse> {
        let url = format!("{}/models/{}:generateContent?key={}", self.base_url, request.model, self.api_key);
        
        let google_request = GoogleChatRequest {
            contents: request.messages.into_iter().map(Into::into).collect(),
            generation_config: Some(GoogleGenerationConfig {
                temperature: request.temperature,
                max_output_tokens: request.max_tokens,
            }),
        };

        let response = self.client
            .post(&url)
            .json(&google_request)
            .send()
            .await
            .context("Failed to send request to Google Gemini")?;

        if !response.status().is_success() {
            let error_text = response.text().await?;
            return Err(anyhow::anyhow!("Google Gemini API error: {}", error_text));
        }

        let google_response: GoogleChatResponse = response.json().await?;
        
        let content = google_response.candidates.first()
            .and_then(|c| c.content.parts.first())
            .map(|p| p.text.clone())
            .unwrap_or_default();

        Ok(LlmResponse {
            content,
            model: request.model,
            usage: google_response.usage_metadata.map(Into::into),
        })
    }

    fn name(&self) -> &'static str {
        "Google"
    }
}

#[derive(Debug, Serialize)]
struct GoogleChatRequest {
    contents: Vec<GoogleContent>,
    #[serde(skip_serializing_if = "Option::is_none")]
    generation_config: Option<GoogleGenerationConfig>,
}

#[derive(Debug, Serialize, Deserialize)]
struct GoogleContent {
    role: String,
    parts: Vec<GooglePart>,
}

#[derive(Debug, Serialize, Deserialize)]
struct GooglePart {
    text: String,
}

impl From<LlmMessage> for GoogleContent {
    fn from(msg: LlmMessage) -> Self {
        Self {
            role: match msg.role {
                LlmRole::System => "user".to_string(), // Gemini uses systemInstruction separately or just user
                LlmRole::User => "user".to_string(),
                LlmRole::Assistant => "model".to_string(),
            },
            parts: vec![GooglePart { text: msg.content }],
        }
    }
}

#[derive(Debug, Serialize)]
struct GoogleGenerationConfig {
    #[serde(skip_serializing_if = "Option::is_none")]
    temperature: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    max_output_tokens: Option<usize>,
}

#[derive(Debug, Deserialize)]
struct GoogleChatResponse {
    candidates: Vec<GoogleCandidate>,
    #[serde(rename = "usageMetadata")]
    usage_metadata: Option<GoogleUsageMetadata>,
}

#[derive(Debug, Deserialize)]
struct GoogleCandidate {
    content: GoogleContent,
}

#[derive(Debug, Deserialize)]
struct GoogleUsageMetadata {
    #[serde(rename = "promptTokenCount")]
    prompt_token_count: usize,
    #[serde(rename = "candidatesTokenCount")]
    candidates_token_count: usize,
    #[serde(rename = "totalTokenCount")]
    total_token_count: usize,
}

impl From<GoogleUsageMetadata> for LlmUsage {
    fn from(usage: GoogleUsageMetadata) -> Self {
        Self {
            prompt_tokens: usage.prompt_token_count,
            completion_tokens: usage.candidates_token_count,
            total_tokens: usage.total_token_count,
        }
    }
}
