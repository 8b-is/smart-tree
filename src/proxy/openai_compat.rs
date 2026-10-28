//! 🔄 OpenAI-Compatible Request/Response Types
//!
//! Shared types for OpenAI API compatibility across daemon and standalone proxy.
//! This ensures consistent behavior between both implementations.
//!
//! "One schema to rule them all!" - The Cheet 😺

use crate::proxy::{LlmMessage, LlmRole};
use serde::{Deserialize, Serialize};

/// OpenAI-compatible request format
#[derive(Debug, Deserialize)]
pub struct OpenAiRequest {
    pub model: String,
    pub messages: Vec<OpenAiMessage>,
    pub temperature: Option<f32>,
    #[serde(rename = "max_tokens")]
    pub max_tokens: Option<usize>,
    pub stream: Option<bool>,
    /// Use 'user' field as scope ID for memory persistence
    pub user: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct OpenAiMessage {
    pub role: String,
    pub content: String,
}

impl From<OpenAiMessage> for LlmMessage {
    fn from(msg: OpenAiMessage) -> Self {
        Self {
            role: match msg.role.as_str() {
                "system" => LlmRole::System,
                "assistant" => LlmRole::Assistant,
                _ => LlmRole::User,
            },
            content: msg.content,
        }
    }
}

/// OpenAI-compatible response format
#[derive(Debug, Serialize)]
pub struct OpenAiResponse {
    pub id: String,
    pub object: String,
    pub created: u64,
    pub model: String,
    pub choices: Vec<OpenAiChoice>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub usage: Option<OpenAiUsage>,
}

/// OpenAI-compatible error response format
#[derive(Debug, Serialize)]
pub struct OpenAiErrorResponse {
    pub error: OpenAiError,
}

#[derive(Debug, Serialize)]
pub struct OpenAiError {
    pub message: String,
    #[serde(rename = "type")]
    pub error_type: String,
    pub code: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct OpenAiChoice {
    pub index: usize,
    pub message: OpenAiResponseMessage,
    pub finish_reason: String,
}

#[derive(Debug, Serialize)]
pub struct OpenAiResponseMessage {
    pub role: String,
    pub content: String,
}

#[derive(Debug, Serialize)]
pub struct OpenAiUsage {
    pub prompt_tokens: usize,
    pub completion_tokens: usize,
    pub total_tokens: usize,
}
