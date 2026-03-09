//! Claude API Client - Comprehensive Anthropic Claude integration via raw reqwest
//!
//! This module provides two interfaces:
//!
//! 1. **`ClaudeClient`** - Rich API with full access to all Claude features:
//!    adaptive thinking, tool use, streaming, vision, prompt caching, etc.
//!
//! 2. **`AnthropicProvider`** - Backward-compatible `LlmProvider` wrapper that
//!    plugs into the existing `LlmProxy` system. Existing code keeps working.
//!
//! # Quick Example
//! ```rust,no_run
//! let client = ClaudeClient::new("sk-ant-...".to_string());
//!
//! // Simple text request
//! let response = client.messages()
//!     .opus()
//!     .system("You are a Rust expert")
//!     .user("Explain lifetimes")
//!     .thinking_adaptive()
//!     .max_tokens(4096)
//!     .send()
//!     .await?;
//!
//! println!("{}", response.text().unwrap_or("no text"));
//! ```
//!
//! # Streaming Example
//! ```rust,no_run
//! let mut parser = client.messages()
//!     .sonnet()
//!     .user("Write a haiku about Rust")
//!     .stream()
//!     .await?;
//!
//! while let Some(event) = parser.next_event().await? {
//!     if let StreamEvent::ContentBlockDelta { delta, .. } = event {
//!         if let ContentDelta::TextDelta { text } = delta {
//!             print!("{}", text);
//!         }
//!     }
//! }
//! ```

pub mod builder;
pub mod error;
pub mod stream;
pub mod types;

// Re-export key types so callers can do `use proxy::claude::*`
pub use builder::MessageRequestBuilder;
pub use error::ClaudeApiError;
pub use stream::{ContentDelta, MessageAccumulator, SseParser, StreamEvent};
pub use types::*;

use crate::proxy::{LlmProvider, LlmRequest, LlmResponse, LlmRole, LlmUsage};
use anyhow::{Context, Result};
use async_trait::async_trait;
use reqwest::Client;

/// Current Anthropic API version header value
const API_VERSION: &str = "2023-06-01";

// ---------------------------------------------------------------------------
// ClaudeClient - the rich API
// ---------------------------------------------------------------------------

/// Full-featured Claude API client built on raw reqwest.
///
/// Supports every Messages API feature: adaptive thinking, tool use,
/// streaming (SSE), vision, prompt caching, structured outputs, beta headers.
///
/// Create one and reuse it - the inner reqwest::Client uses connection pooling.
pub struct ClaudeClient {
    pub(crate) client: Client,
    pub(crate) api_key: String,
    pub(crate) base_url: String,
    pub(crate) default_model: String,
}

impl ClaudeClient {
    /// Create a new client with the given API key.
    /// Default model: Sonnet 4.6. Default base URL: Anthropic's API.
    pub fn new(api_key: String) -> Self {
        Self {
            client: Client::new(),
            api_key,
            base_url: "https://api.anthropic.com/v1".to_string(),
            default_model: models::SONNET_4_6.to_string(),
        }
    }

    /// Override the base URL (e.g. for a proxy or testing)
    pub fn with_base_url(mut self, url: String) -> Self {
        self.base_url = url;
        self
    }

    /// Set the default model used when `.model()` isn't called on the builder
    pub fn with_default_model(mut self, model: String) -> Self {
        self.default_model = model;
        self
    }

    /// Start building a messages request. Returns a fluent builder.
    pub fn messages(&self) -> MessageRequestBuilder<'_> {
        MessageRequestBuilder::new(self)
    }

    /// Low-level: send a raw `MessagesRequest` and return the raw HTTP response.
    /// Used internally by the builder's `.send()` and `.stream()` methods.
    ///
    /// Adds the required auth and version headers automatically.
    pub(crate) async fn send_request(
        &self,
        request: &MessagesRequest,
        extra_betas: &[String],
    ) -> Result<reqwest::Response, ClaudeApiError> {
        let url = format!("{}/messages", self.base_url);

        let mut req = self
            .client
            .post(&url)
            .header("x-api-key", &self.api_key)
            .header("anthropic-version", API_VERSION)
            .header("content-type", "application/json");

        // Add any beta feature headers
        if !extra_betas.is_empty() {
            req = req.header("anthropic-beta", extra_betas.join(","));
        }

        let response = req
            .json(request)
            .send()
            .await
            .map_err(ClaudeApiError::Network)?;

        // Check for API errors (4xx/5xx)
        if !response.status().is_success() {
            let status = response.status().as_u16();
            let body = response.text().await.unwrap_or_default();
            return Err(ClaudeApiError::from_response(status, &body));
        }

        Ok(response)
    }
}

// ---------------------------------------------------------------------------
// AnthropicProvider - backward-compatible LlmProvider wrapper
// ---------------------------------------------------------------------------

/// Backward-compatible provider implementing `LlmProvider`.
///
/// Wraps `ClaudeClient` for use with the existing `LlmProxy` system.
/// All existing code that creates `AnthropicProvider::default()` or calls
/// `.complete()` continues to work unchanged.
///
/// For rich API access (thinking, tools, streaming), use `.claude_client()`.
pub struct AnthropicProvider {
    client: ClaudeClient,
}

impl AnthropicProvider {
    pub fn new(api_key: String) -> Self {
        Self {
            client: ClaudeClient::new(api_key),
        }
    }

    /// Get a reference to the underlying `ClaudeClient` for full API access.
    ///
    /// ```rust,no_run
    /// let provider = AnthropicProvider::default();
    /// let response = provider.claude_client().messages()
    ///     .opus()
    ///     .user("Hello!")
    ///     .thinking_adaptive()
    ///     .send()
    ///     .await?;
    /// ```
    pub fn claude_client(&self) -> &ClaudeClient {
        &self.client
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
    /// Complete a request using the simple LlmRequest interface.
    /// Extracts the first text block from the response for backward compat.
    async fn complete(&self, request: LlmRequest) -> Result<LlmResponse> {
        // Build a rich request from the simple LlmRequest
        let mut builder = self
            .client
            .messages()
            .model(&request.model)
            .max_tokens(request.max_tokens.unwrap_or(1024));

        if let Some(temp) = request.temperature {
            builder = builder.temperature(temp);
        }

        // Separate system messages from conversation messages
        for msg in request.messages {
            match msg.role {
                LlmRole::System => {
                    builder = builder.system(&msg.content);
                }
                LlmRole::User => {
                    builder = builder.user(&msg.content);
                }
                LlmRole::Assistant => {
                    builder = builder.assistant(&msg.content);
                }
            }
        }

        let response = builder
            .send()
            .await
            .context("Failed to complete Claude request")?;

        // Extract the first text block (backward compatible behavior)
        let content = response.text().unwrap_or("").to_string();

        Ok(LlmResponse {
            content,
            model: response.model,
            usage: Some(LlmUsage::from(response.usage)),
        })
    }

    fn name(&self) -> &'static str {
        "Anthropic"
    }
}
