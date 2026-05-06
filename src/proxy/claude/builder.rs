//! Fluent Request Builder for the Claude Messages API
//!
//! Provides an ergonomic builder pattern for constructing requests:
//!
//! ```rust,no_run
//! let response = client.messages()
//!     .opus()
//!     .system("You are a Rust expert")
//!     .user("Explain lifetimes")
//!     .thinking_adaptive()
//!     .effort("high")
//!     .max_tokens(4096)
//!     .send()
//!     .await?;
//! ```
//!
//! The builder accumulates all parameters, then sends via the parent `ClaudeClient`.

use super::error::ClaudeApiError;
use super::stream::SseParser;
use super::types::*;

// ---------------------------------------------------------------------------
// Builder struct - accumulates request parameters
// ---------------------------------------------------------------------------

/// Fluent builder for Claude API message requests.
///
/// Created via `ClaudeClient::messages()`. Chain methods to configure,
/// then call `.send()` for the full response or `.stream()` for SSE events.
pub struct MessageRequestBuilder<'a> {
    /// Reference to the parent client (holds HTTP client, API key, base URL)
    client: &'a super::ClaudeClient,
    model: String,
    messages: Vec<Message>,
    system: Option<SystemContent>,
    max_tokens: usize,
    temperature: Option<f32>,
    top_p: Option<f32>,
    top_k: Option<usize>,
    stop_sequences: Option<Vec<String>>,
    thinking: Option<ThinkingConfig>,
    tools: Option<Vec<Tool>>,
    tool_choice: Option<ToolChoice>,
    output_config: Option<OutputConfig>,
    metadata: Option<Metadata>,
    stream: bool,
    beta_headers: Vec<String>,
}

impl<'a> MessageRequestBuilder<'a> {
    /// Create a new builder with sensible defaults
    pub(crate) fn new(client: &'a super::ClaudeClient) -> Self {
        Self {
            client,
            model: client.default_model.clone(),
            messages: Vec::new(),
            system: None,
            max_tokens: 4096,
            temperature: None,
            top_p: None,
            top_k: None,
            stop_sequences: None,
            thinking: None,
            tools: None,
            tool_choice: None,
            output_config: None,
            metadata: None,
            stream: false,
            beta_headers: Vec::new(),
        }
    }

    // === Model selection ===

    /// Set the model by ID (e.g. "claude-opus-4-6")
    pub fn model(mut self, model: &str) -> Self {
        self.model = model.to_string();
        self
    }

    /// Use Claude Opus 4.6 - most intelligent, best for agents and coding
    pub fn opus(self) -> Self {
        self.model(models::OPUS_4_6)
    }

    /// Use Claude Sonnet 4.6 - best speed/intelligence balance
    pub fn sonnet(self) -> Self {
        self.model(models::SONNET_4_6)
    }

    /// Use Claude Haiku 4.5 - fastest and cheapest
    pub fn haiku(self) -> Self {
        self.model(models::HAIKU_4_5)
    }

    // === Messages ===

    /// Set the system prompt (plain text)
    pub fn system(mut self, text: &str) -> Self {
        self.system = Some(SystemContent::Text(text.to_string()));
        self
    }

    /// Set the system prompt with prompt caching enabled
    pub fn system_cached(mut self, text: &str) -> Self {
        self.system = Some(SystemContent::Blocks(vec![SystemBlock {
            block_type: "text".to_string(),
            text: text.to_string(),
            cache_control: Some(CacheControl::ephemeral()),
        }]));
        self
    }

    /// Add a user text message
    pub fn user(mut self, text: &str) -> Self {
        self.messages.push(Message::user(text));
        self
    }

    /// Add a user message with an image (base64 encoded)
    pub fn user_with_image_base64(
        mut self,
        text: &str,
        media_type: &str,
        base64_data: &str,
    ) -> Self {
        self.messages.push(Message {
            role: MessageRole::User,
            content: MessageContent::Blocks(vec![
                ContentBlock::Image {
                    source: ImageSource::Base64 {
                        media_type: media_type.to_string(),
                        data: base64_data.to_string(),
                    },
                    cache_control: None,
                },
                ContentBlock::Text {
                    text: text.to_string(),
                    cache_control: None,
                },
            ]),
        });
        self
    }

    /// Add a user message with an image URL
    pub fn user_with_image_url(mut self, text: &str, url: &str) -> Self {
        self.messages.push(Message {
            role: MessageRole::User,
            content: MessageContent::Blocks(vec![
                ContentBlock::Image {
                    source: ImageSource::Url {
                        url: url.to_string(),
                    },
                    cache_control: None,
                },
                ContentBlock::Text {
                    text: text.to_string(),
                    cache_control: None,
                },
            ]),
        });
        self
    }

    /// Add an assistant text message (for multi-turn conversations)
    pub fn assistant(mut self, text: &str) -> Self {
        self.messages.push(Message::assistant(text));
        self
    }

    /// Set the full message list (for agentic loops where you manage messages)
    pub fn messages(mut self, msgs: Vec<Message>) -> Self {
        self.messages = msgs;
        self
    }

    /// Add tool results as a user message (for the agentic tool loop)
    pub fn tool_results(mut self, results: Vec<ContentBlock>) -> Self {
        self.messages.push(Message {
            role: MessageRole::User,
            content: MessageContent::Blocks(results),
        });
        self
    }

    // === Thinking ===

    /// Enable adaptive thinking (recommended for Opus 4.6 / Sonnet 4.6).
    /// Claude dynamically decides when and how much to think.
    pub fn thinking_adaptive(mut self) -> Self {
        self.thinking = Some(ThinkingConfig::Adaptive);
        self
    }

    /// Enable thinking with a fixed token budget (older models only).
    /// `budget_tokens` must be less than `max_tokens` (minimum 1024).
    pub fn thinking_enabled(mut self, budget_tokens: usize) -> Self {
        self.thinking = Some(ThinkingConfig::Enabled { budget_tokens });
        self
    }

    /// Explicitly disable thinking
    pub fn thinking_disabled(mut self) -> Self {
        self.thinking = Some(ThinkingConfig::Disabled);
        self
    }

    // === Effort ===

    /// Set the effort level: "low", "medium", "high" (default), or "max" (Opus only).
    /// Lower effort = cheaper/faster. Higher effort = deeper reasoning.
    pub fn effort(mut self, level: &str) -> Self {
        let effort = match level {
            "low" => Effort::Low,
            "medium" => Effort::Medium,
            "max" => Effort::Max,
            _ => Effort::High, // default
        };
        // Merge into existing output_config or create new one
        match self.output_config.as_mut() {
            Some(config) => config.effort = Some(effort),
            None => {
                self.output_config = Some(OutputConfig {
                    effort: Some(effort),
                    format: None,
                });
            }
        }
        self
    }

    // === Tools ===

    /// Set the available tools for Claude to call
    pub fn tools(mut self, tools: Vec<Tool>) -> Self {
        self.tools = Some(tools);
        self
    }

    /// Let Claude decide whether to use tools (default)
    pub fn tool_choice_auto(mut self) -> Self {
        self.tool_choice = Some(ToolChoice::Auto {
            disable_parallel_tool_use: None,
        });
        self
    }

    /// Force Claude to use at least one tool
    pub fn tool_choice_any(mut self) -> Self {
        self.tool_choice = Some(ToolChoice::Any {
            disable_parallel_tool_use: None,
        });
        self
    }

    /// Force Claude to use a specific tool by name
    pub fn tool_choice_specific(mut self, name: &str) -> Self {
        self.tool_choice = Some(ToolChoice::Tool {
            name: name.to_string(),
            disable_parallel_tool_use: None,
        });
        self
    }

    /// Prevent Claude from using any tools
    pub fn tool_choice_none(mut self) -> Self {
        self.tool_choice = Some(ToolChoice::None);
        self
    }

    // === Structured output ===

    /// Constrain the response to match a JSON schema
    pub fn json_schema(mut self, schema: serde_json::Value) -> Self {
        let format = Some(OutputFormat::JsonSchema { schema });
        match self.output_config.as_mut() {
            Some(config) => config.format = format,
            None => {
                self.output_config = Some(OutputConfig {
                    effort: None,
                    format,
                });
            }
        }
        self
    }

    // === Parameters ===

    /// Maximum tokens to generate (default: 4096)
    pub fn max_tokens(mut self, n: usize) -> Self {
        self.max_tokens = n;
        self
    }

    /// Sampling temperature (0.0 = deterministic, 1.0 = creative)
    pub fn temperature(mut self, t: f32) -> Self {
        self.temperature = Some(t);
        self
    }

    /// Nucleus sampling parameter
    pub fn top_p(mut self, p: f32) -> Self {
        self.top_p = Some(p);
        self
    }

    /// Top-k sampling parameter
    pub fn top_k(mut self, k: usize) -> Self {
        self.top_k = Some(k);
        self
    }

    /// Custom stop sequences
    pub fn stop_sequences(mut self, seqs: Vec<String>) -> Self {
        self.stop_sequences = Some(seqs);
        self
    }

    /// Set a user ID for tracking/billing
    pub fn user_id(mut self, id: &str) -> Self {
        self.metadata = Some(Metadata {
            user_id: Some(id.to_string()),
        });
        self
    }

    /// Enable a beta feature by header string (e.g. "compact-2026-01-12")
    pub fn beta(mut self, header: &str) -> Self {
        self.beta_headers.push(header.to_string());
        self
    }

    // === Build / Send / Stream ===

    /// Build the `MessagesRequest` from current state (internal helper)
    fn build_request(&self, force_stream: bool) -> MessagesRequest {
        MessagesRequest {
            model: self.model.clone(),
            messages: self.messages.clone(),
            max_tokens: self.max_tokens,
            system: self.system.clone(),
            temperature: self.temperature,
            top_p: self.top_p,
            top_k: self.top_k,
            stop_sequences: self.stop_sequences.clone(),
            thinking: self.thinking.clone(),
            tools: self.tools.clone(),
            tool_choice: self.tool_choice.clone(),
            output_config: self.output_config.clone(),
            metadata: self.metadata.clone(),
            stream: force_stream || self.stream,
        }
    }

    /// Build the request without sending (useful for inspection/testing)
    pub fn build(self) -> MessagesRequest {
        self.build_request(false)
    }

    /// Send the request and return the full response (non-streaming).
    pub async fn send(&self) -> Result<MessagesResponse, ClaudeApiError> {
        let request = self.build_request(false);
        let response = self
            .client
            .send_request(&request, &self.beta_headers)
            .await?;
        response
            .json::<MessagesResponse>()
            .await
            .map_err(ClaudeApiError::from)
    }

    /// Send the request with streaming, returning an SSE event parser.
    /// Use `parser.next_event()` to consume events one at a time.
    pub async fn stream(&self) -> Result<SseParser, ClaudeApiError> {
        let request = self.build_request(true);
        let response = self
            .client
            .send_request(&request, &self.beta_headers)
            .await?;
        Ok(SseParser::new(response))
    }
}
