//! SSE Streaming Parser for the Claude Messages API
//!
//! The Claude API sends Server-Sent Events (SSE) when `stream: true`.
//! Format: `event: <name>\ndata: <json>\n\n`
//!
//! This module provides:
//! - `SseParser`: reads raw bytes from reqwest and yields typed `StreamEvent`s
//! - `MessageAccumulator`: builds a complete `MessagesResponse` from events
//!
//! The tricky part: bytes arrive in arbitrary chunks that may split mid-line.
//! We buffer until we see `\n\n` (end of SSE event), then parse.

use super::error::ClaudeApiError;
use super::types::{ContentBlock, MessagesResponse, StopReason, Usage};
use futures_util::StreamExt;
use serde::Deserialize;
use std::pin::Pin;

// ---------------------------------------------------------------------------
// Stream event types (deserialized from SSE `data:` payloads)
// ---------------------------------------------------------------------------

/// All possible SSE event types from the Claude streaming API.
/// Each variant matches an `event: <name>` line in the SSE stream.
#[derive(Debug, Clone, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum StreamEvent {
    /// First event - contains the message skeleton (id, model, usage)
    MessageStart { message: MessagesResponse },
    /// A new content block is starting at the given index
    ContentBlockStart {
        index: usize,
        content_block: ContentBlock,
    },
    /// Incremental content for the block at the given index
    ContentBlockDelta { index: usize, delta: ContentDelta },
    /// The block at the given index is complete
    ContentBlockStop { index: usize },
    /// Message-level update (stop_reason, final usage)
    MessageDelta {
        delta: MessageDeltaPayload,
        #[serde(skip_serializing_if = "Option::is_none")]
        usage: Option<Usage>,
    },
    /// Message is complete - stream will end after this
    MessageStop,
    /// Keepalive ping (ignore)
    Ping,
    /// Server-side error delivered via the stream
    Error { error: super::error::ApiErrorBody },
}

/// Incremental content within a `content_block_delta` event
#[derive(Debug, Clone, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ContentDelta {
    /// Incremental text (append to current text block)
    TextDelta { text: String },
    /// Incremental thinking (append to current thinking block)
    ThinkingDelta { thinking: String },
    /// Incremental signature for thinking block verification
    SignatureDelta { signature: String },
    /// Incremental JSON for tool input (append and parse when block stops)
    InputJsonDelta { partial_json: String },
}

/// Top-level message metadata update
#[derive(Debug, Clone, Deserialize)]
pub struct MessageDeltaPayload {
    pub stop_reason: Option<StopReason>,
    pub stop_sequence: Option<String>,
}

// ---------------------------------------------------------------------------
// SSE Parser - reads bytes, yields StreamEvents
// ---------------------------------------------------------------------------

/// Parses SSE events from a reqwest byte stream.
///
/// # Example
/// ```rust,no_run
/// let response = client.post(url).send().await?;
/// let mut parser = SseParser::new(response);
/// while let Some(event) = parser.next_event().await? {
///     match event {
///         StreamEvent::ContentBlockDelta { delta, .. } => { /* handle */ }
///         StreamEvent::MessageStop => break,
///         _ => {}
///     }
/// }
/// ```
pub struct SseParser {
    /// The raw byte stream from reqwest
    stream: Pin<Box<dyn futures_util::Stream<Item = Result<bytes::Bytes, reqwest::Error>> + Send>>,
    /// Accumulated text that hasn't been parsed into an event yet
    buffer: String,
    /// Whether the stream has ended
    done: bool,
}

impl SseParser {
    /// Wrap a reqwest response in an SSE parser
    pub fn new(response: reqwest::Response) -> Self {
        Self {
            stream: Box::pin(response.bytes_stream()),
            buffer: String::new(),
            done: false,
        }
    }

    /// Read the next complete SSE event from the stream.
    /// Returns `None` when the stream ends.
    pub async fn next_event(&mut self) -> Result<Option<StreamEvent>, ClaudeApiError> {
        if self.done {
            return Ok(None);
        }

        loop {
            // Check if we already have a complete event in the buffer
            // SSE events are delimited by blank lines (\n\n)
            if let Some(event) = self.try_parse_event()? {
                return Ok(Some(event));
            }

            // Need more data from the stream
            match self.stream.next().await {
                Some(Ok(bytes)) => {
                    // Append raw bytes to our text buffer
                    let text = String::from_utf8_lossy(&bytes);
                    self.buffer.push_str(&text);
                }
                Some(Err(e)) => {
                    self.done = true;
                    return Err(ClaudeApiError::Network(e));
                }
                None => {
                    // Stream ended - try to parse any remaining data
                    self.done = true;
                    return self.try_parse_event();
                }
            }
        }
    }

    /// Try to extract one complete SSE event from the buffer.
    /// An event ends with a blank line (\n\n). Each event has:
    /// - `event: <name>` line (the event type)
    /// - `data: <json>` line (the payload, may span multiple lines)
    fn try_parse_event(&mut self) -> Result<Option<StreamEvent>, ClaudeApiError> {
        // Find the next complete event (double newline boundary)
        let boundary = match self.buffer.find("\n\n") {
            Some(pos) => pos,
            None => return Ok(None),
        };

        // Extract the raw event text and remove it from the buffer
        let raw_event = self.buffer[..boundary].to_string();
        self.buffer = self.buffer[boundary + 2..].to_string();

        // Parse the SSE fields
        let mut event_name = String::new();
        let mut data_lines = Vec::new();

        for line in raw_event.lines() {
            if let Some(name) = line.strip_prefix("event: ") {
                event_name = name.trim().to_string();
            } else if let Some(data) = line.strip_prefix("data: ") {
                data_lines.push(data);
            } else if let Some(stripped) = line.strip_prefix("data:") {
                // `data:` with no space (empty data)
                data_lines.push(stripped);
            }
            // Ignore other lines (comments starting with :, etc.)
        }

        // Skip events with no data (like keepalive comments)
        if data_lines.is_empty() {
            return Ok(None);
        }

        let data = data_lines.join("\n");

        // Parse the JSON payload based on event type
        // The Claude API sets the `type` field in the JSON to match the event name,
        // so we can use serde's tagged enum directly
        match serde_json::from_str::<StreamEvent>(&data) {
            Ok(event) => Ok(Some(event)),
            Err(e) => {
                // If we can't parse it, log context and return error
                Err(ClaudeApiError::StreamError {
                    message: format!(
                        "Failed to parse SSE event '{}': {} (data: {})",
                        event_name, e, data
                    ),
                })
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Message Accumulator - builds complete response from stream events
// ---------------------------------------------------------------------------

/// Accumulates streaming events into a complete `MessagesResponse`.
///
/// Useful when you want streaming for timeout protection but need the
/// final assembled message (like reqwest's `.get_final_message()` pattern).
#[derive(Default)]
pub struct MessageAccumulator {
    /// The response skeleton from message_start
    response: Option<MessagesResponse>,
    /// Content blocks being built up from deltas
    blocks: Vec<BlockBuilder>,
}

/// Internal: tracks a content block being assembled from deltas
struct BlockBuilder {
    text: String,
    thinking: String,
    tool_id: String,
    tool_name: String,
    partial_json: String,
    signature: String,
    is_thinking: bool,
    is_tool_use: bool,
}

impl BlockBuilder {
    fn new_text() -> Self {
        Self {
            text: String::new(),
            thinking: String::new(),
            tool_id: String::new(),
            tool_name: String::new(),
            partial_json: String::new(),
            signature: String::new(),
            is_thinking: false,
            is_tool_use: false,
        }
    }

    fn new_thinking() -> Self {
        Self {
            is_thinking: true,
            ..Self::new_text()
        }
    }

    fn new_tool(id: String, name: String) -> Self {
        Self {
            tool_id: id,
            tool_name: name,
            is_tool_use: true,
            ..Self::new_text()
        }
    }

    /// Convert the accumulated data into a final ContentBlock
    fn finish(self) -> ContentBlock {
        if self.is_thinking {
            ContentBlock::Thinking {
                thinking: self.thinking,
                signature: if self.signature.is_empty() {
                    None
                } else {
                    Some(self.signature)
                },
            }
        } else if self.is_tool_use {
            let input = serde_json::from_str(&self.partial_json)
                .unwrap_or(serde_json::Value::Object(serde_json::Map::new()));
            ContentBlock::ToolUse {
                id: self.tool_id,
                name: self.tool_name,
                input,
            }
        } else {
            ContentBlock::Text {
                text: self.text,
                cache_control: None,
            }
        }
    }
}

impl MessageAccumulator {
    pub fn new() -> Self {
        Self::default()
    }

    /// Process a single stream event, updating internal state
    pub fn process_event(&mut self, event: &StreamEvent) {
        match event {
            StreamEvent::MessageStart { message } => {
                self.response = Some(message.clone());
            }
            StreamEvent::ContentBlockStart { content_block, .. } => match content_block {
                ContentBlock::Thinking { .. } => self.blocks.push(BlockBuilder::new_thinking()),
                ContentBlock::ToolUse { id, name, .. } => {
                    self.blocks
                        .push(BlockBuilder::new_tool(id.clone(), name.clone()));
                }
                _ => self.blocks.push(BlockBuilder::new_text()),
            },
            StreamEvent::ContentBlockDelta { index, delta } => {
                if let Some(block) = self.blocks.get_mut(*index) {
                    match delta {
                        ContentDelta::TextDelta { text } => block.text.push_str(text),
                        ContentDelta::ThinkingDelta { thinking } => {
                            block.thinking.push_str(thinking)
                        }
                        ContentDelta::InputJsonDelta { partial_json } => {
                            block.partial_json.push_str(partial_json);
                        }
                        ContentDelta::SignatureDelta { signature } => {
                            block.signature.push_str(signature);
                        }
                    }
                }
            }
            StreamEvent::MessageDelta { delta, usage } => {
                if let Some(ref mut resp) = self.response {
                    resp.stop_reason = delta.stop_reason.clone();
                    resp.stop_sequence = delta.stop_sequence.clone();
                    if let Some(u) = usage {
                        resp.usage.output_tokens = u.output_tokens;
                    }
                }
            }
            // ContentBlockStop, MessageStop, Ping - no accumulation needed
            _ => {}
        }
    }

    /// Finalize and return the complete `MessagesResponse`
    pub fn finish(mut self) -> Result<MessagesResponse, ClaudeApiError> {
        let mut response = self
            .response
            .take()
            .ok_or_else(|| ClaudeApiError::StreamError {
                message: "Stream ended without message_start event".to_string(),
            })?;

        // Replace the skeleton content blocks with our accumulated ones
        response.content = self.blocks.into_iter().map(|b| b.finish()).collect();

        Ok(response)
    }
}
