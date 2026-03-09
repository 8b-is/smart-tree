//! Claude API Types - Complete request/response model for the Messages API
//!
//! Every struct here maps 1:1 to the Claude API JSON schema.
//! Optional fields use `skip_serializing_if` to keep payloads clean.
//!
//! Key design: `ContentBlock` is a tagged enum that handles ALL content types
//! (text, image, thinking, tool_use, tool_result) in both requests and responses.

use crate::proxy::{LlmMessage, LlmRole, LlmUsage};
use serde::{Deserialize, Serialize};

// ---------------------------------------------------------------------------
// Model constants - always use these, never construct model IDs manually
// ---------------------------------------------------------------------------

/// Current Claude model IDs (as of 2026-03)
pub mod models {
    /// Most intelligent model - agents, coding, deep reasoning
    pub const OPUS_4_6: &str = "claude-opus-4-6";
    /// Best speed/intelligence balance - general purpose
    pub const SONNET_4_6: &str = "claude-sonnet-4-6";
    /// Fastest and cheapest - simple tasks
    pub const HAIKU_4_5: &str = "claude-haiku-4-5";

    // Legacy (still active) - use only when explicitly requested
    pub const OPUS_4_5: &str = "claude-opus-4-5";
    pub const SONNET_4_5: &str = "claude-sonnet-4-5";
}

// ---------------------------------------------------------------------------
// Request types
// ---------------------------------------------------------------------------

/// The main request body sent to `POST /v1/messages`
#[derive(Debug, Clone, Serialize)]
pub struct MessagesRequest {
    pub model: String,
    pub messages: Vec<Message>,
    pub max_tokens: usize,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub system: Option<SystemContent>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub temperature: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub top_p: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub top_k: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stop_sequences: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub thinking: Option<ThinkingConfig>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tools: Option<Vec<Tool>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_choice: Option<ToolChoice>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub output_config: Option<OutputConfig>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<Metadata>,
    pub stream: bool,
}

/// System prompt - either a plain string or structured blocks with cache_control.
/// Serde's `untagged` tries String first, then Vec<SystemBlock>.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum SystemContent {
    Text(String),
    Blocks(Vec<SystemBlock>),
}

/// A system block with optional cache_control for prompt caching
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemBlock {
    #[serde(rename = "type")]
    pub block_type: String, // always "text"
    pub text: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cache_control: Option<CacheControl>,
}

// ---------------------------------------------------------------------------
// Messages and content blocks
// ---------------------------------------------------------------------------

/// A single message in the conversation (user or assistant)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    pub role: MessageRole,
    pub content: MessageContent,
}

impl Message {
    /// Quick constructor for a user text message
    pub fn user(text: impl Into<String>) -> Self {
        Self {
            role: MessageRole::User,
            content: MessageContent::Text(text.into()),
        }
    }

    /// Quick constructor for an assistant text message
    pub fn assistant(text: impl Into<String>) -> Self {
        Self {
            role: MessageRole::Assistant,
            content: MessageContent::Text(text.into()),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum MessageRole {
    User,
    Assistant,
}

/// Content can be a simple string or an array of typed blocks.
/// The API accepts both forms; responses always use Blocks.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum MessageContent {
    /// Simple string content (convenience for text-only messages)
    Text(String),
    /// Array of typed content blocks (full power mode)
    Blocks(Vec<ContentBlock>),
}

/// All possible content block types in request AND response messages.
///
/// Tagged on the `type` field so `{"type": "text", "text": "hello"}` maps to
/// `ContentBlock::Text { text: "hello", .. }`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ContentBlock {
    /// Plain text content
    Text {
        text: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        cache_control: Option<CacheControl>,
    },
    /// Image content (base64 or URL)
    Image {
        source: ImageSource,
        #[serde(skip_serializing_if = "Option::is_none")]
        cache_control: Option<CacheControl>,
    },
    /// Model's thinking/reasoning (returned with adaptive thinking enabled)
    Thinking {
        thinking: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        signature: Option<String>,
    },
    /// Redacted thinking block (safety-filtered reasoning)
    RedactedThinking { data: String },
    /// Model wants to call a tool (response only)
    ToolUse {
        id: String,
        name: String,
        input: serde_json::Value,
    },
    /// Result of a tool call (request only - sent back by the client)
    ToolResult {
        tool_use_id: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        content: Option<ToolResultContent>,
        #[serde(skip_serializing_if = "Option::is_none")]
        is_error: Option<bool>,
    },
}

impl ContentBlock {
    /// Extract text content if this is a Text block
    pub fn as_text(&self) -> Option<&str> {
        match self {
            ContentBlock::Text { text, .. } => Some(text),
            _ => None,
        }
    }

    /// Extract thinking content if this is a Thinking block
    pub fn as_thinking(&self) -> Option<&str> {
        match self {
            ContentBlock::Thinking { thinking, .. } => Some(thinking),
            _ => None,
        }
    }
}

// ---------------------------------------------------------------------------
// Image source variants
// ---------------------------------------------------------------------------

/// How an image is provided: inline base64 or external URL
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ImageSource {
    Base64 {
        media_type: String, // "image/jpeg", "image/png", "image/gif", "image/webp"
        data: String,
    },
    Url {
        url: String,
    },
}

// ---------------------------------------------------------------------------
// Tool result content
// ---------------------------------------------------------------------------

/// Content of a tool result - either a plain string or structured blocks
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ToolResultContent {
    Text(String),
    Blocks(Vec<ContentBlock>),
}

// ---------------------------------------------------------------------------
// Thinking configuration
// ---------------------------------------------------------------------------

/// Controls Claude's internal reasoning.
/// - `Adaptive`: Claude decides when/how much to think (Opus 4.6 / Sonnet 4.6)
/// - `Enabled`: Fixed thinking budget (older models only)
/// - `Disabled`: No thinking
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ThinkingConfig {
    Adaptive,
    Enabled { budget_tokens: usize },
    Disabled,
}

// ---------------------------------------------------------------------------
// Tool definitions and choice
// ---------------------------------------------------------------------------

/// A tool definition telling Claude what functions are available
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Tool {
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    pub input_schema: serde_json::Value,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cache_control: Option<CacheControl>,
    /// When true, Claude guarantees valid parameters matching the schema
    #[serde(skip_serializing_if = "Option::is_none")]
    pub strict: Option<bool>,
}

/// Controls whether/how Claude selects tools
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ToolChoice {
    Auto {
        #[serde(skip_serializing_if = "Option::is_none")]
        disable_parallel_tool_use: Option<bool>,
    },
    Any {
        #[serde(skip_serializing_if = "Option::is_none")]
        disable_parallel_tool_use: Option<bool>,
    },
    Tool {
        name: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        disable_parallel_tool_use: Option<bool>,
    },
    None,
}

// ---------------------------------------------------------------------------
// Output config (effort + structured outputs)
// ---------------------------------------------------------------------------

/// Controls output behavior: thinking effort and structured format
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OutputConfig {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub effort: Option<Effort>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub format: Option<OutputFormat>,
}

/// How hard Claude should think. Default is `High`.
/// `Max` is Opus 4.6 only.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Effort {
    Low,
    Medium,
    High,
    Max,
}

/// Structured output format constraint
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum OutputFormat {
    JsonSchema { schema: serde_json::Value },
}

// ---------------------------------------------------------------------------
// Cache control and metadata
// ---------------------------------------------------------------------------

/// Prompt caching directive - attach to system blocks, messages, or tools
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheControl {
    #[serde(rename = "type")]
    pub control_type: String, // "ephemeral"
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ttl: Option<String>, // "5m", "1h"
}

impl CacheControl {
    /// Default 5-minute ephemeral cache
    pub fn ephemeral() -> Self {
        Self {
            control_type: "ephemeral".to_string(),
            ttl: None,
        }
    }

    /// 1-hour ephemeral cache for large documents
    pub fn ephemeral_1h() -> Self {
        Self {
            control_type: "ephemeral".to_string(),
            ttl: Some("1h".to_string()),
        }
    }
}

/// Optional metadata for tracking/billing purposes
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Metadata {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user_id: Option<String>,
}

// ---------------------------------------------------------------------------
// Response types
// ---------------------------------------------------------------------------

/// Full response from `POST /v1/messages` (non-streaming)
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct MessagesResponse {
    pub id: String,
    #[serde(rename = "type")]
    pub response_type: String, // "message"
    pub role: String, // "assistant"
    pub content: Vec<ContentBlock>,
    pub model: String,
    pub stop_reason: Option<StopReason>,
    pub stop_sequence: Option<String>,
    pub usage: Usage,
}

impl MessagesResponse {
    /// Extract the first text content from the response (most common case)
    pub fn text(&self) -> Option<&str> {
        self.content.iter().find_map(|b| b.as_text())
    }

    /// Extract all thinking blocks concatenated
    pub fn thinking(&self) -> Option<String> {
        let parts: Vec<&str> = self
            .content
            .iter()
            .filter_map(|b| b.as_thinking())
            .collect();
        if parts.is_empty() {
            None
        } else {
            Some(parts.join("\n"))
        }
    }

    /// Check if Claude wants to use tools
    pub fn has_tool_use(&self) -> bool {
        self.stop_reason == Some(StopReason::ToolUse)
    }

    /// Extract all tool use blocks from the response
    pub fn tool_calls(&self) -> Vec<(&str, &str, &serde_json::Value)> {
        self.content
            .iter()
            .filter_map(|b| match b {
                ContentBlock::ToolUse { id, name, input } => {
                    Some((id.as_str(), name.as_str(), input))
                }
                _ => None,
            })
            .collect()
    }
}

/// Why Claude stopped generating
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum StopReason {
    EndTurn,
    MaxTokens,
    StopSequence,
    ToolUse,
    PauseTurn,
    Refusal,
}

/// Token usage statistics from the API
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Usage {
    pub input_tokens: usize,
    pub output_tokens: usize,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cache_creation_input_tokens: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cache_read_input_tokens: Option<usize>,
}

// ---------------------------------------------------------------------------
// Conversions to/from the shared LlmMessage/LlmUsage types
// ---------------------------------------------------------------------------

impl From<LlmMessage> for Message {
    fn from(msg: LlmMessage) -> Self {
        Self {
            role: match msg.role {
                LlmRole::System => MessageRole::User, // system handled separately
                LlmRole::User => MessageRole::User,
                LlmRole::Assistant => MessageRole::Assistant,
            },
            content: MessageContent::Text(msg.content),
        }
    }
}

impl From<Usage> for LlmUsage {
    fn from(u: Usage) -> Self {
        Self {
            prompt_tokens: u.input_tokens,
            completion_tokens: u.output_tokens,
            total_tokens: u.input_tokens + u.output_tokens,
        }
    }
}
