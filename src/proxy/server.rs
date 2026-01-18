//! 🌐 OpenAI-Compatible Proxy Server
//!
//! This module provides an HTTP server that implements the OpenAI Chat Completions API,
//! allowing any OpenAI-compatible client to use Smart Tree as a backend.
//!
//! "Serving AI requests with a smile and a tree!" - The Cheet 😺

use crate::proxy::{LlmMessage, LlmRequest, LlmRole};
use crate::proxy::memory::MemoryProxy;
use axum::{
    extract::State,
    routing::post,
    Json, Router,
};
use serde::{Deserialize, Serialize};
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::sync::Mutex;
use anyhow::Result;

/// 🚀 Start the OpenAI-compatible proxy server
pub async fn start_proxy_server(port: u16) -> Result<()> {
    let proxy = Arc::new(Mutex::new(MemoryProxy::new()?));
    
    let app = Router::new()
        .route("/v1/chat/completions", post(chat_completions))
        .with_state(proxy);

    let addr = SocketAddr::from(([127, 0, 0, 1], port));
    println!("🚀 Smart Tree LLM Proxy Server running on http://{}", addr);
    println!("🌳 OpenAI-compatible endpoint: http://{}/v1/chat/completions", addr);
    println!("🧠 Memory enabled with scoped conversation history!");

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}

/// 💬 OpenAI-compatible chat completions handler
async fn chat_completions(
    State(proxy): State<Arc<Mutex<MemoryProxy>>>,
    Json(req): Json<OpenAiRequest>,
) -> Json<OpenAiResponse> {
    // Map OpenAI request to our internal LlmRequest
    // We use the model name to determine the provider if it's prefixed, e.g., "anthropic/claude-3"
    let (provider_name, model_name) = if let Some((p, m)) = req.model.split_once('/') {
        (p.to_string(), m.to_string())
    } else {
        // Default to OpenAI if no prefix
        ("openai".to_string(), req.model.clone())
    };

    let internal_req = LlmRequest {
        model: model_name,
        messages: req.messages.into_iter().map(Into::into).collect(),
        temperature: req.temperature,
        max_tokens: req.max_tokens,
        stream: req.stream.unwrap_or(false),
    };

    // Use the 'user' field as the scope ID, or default to 'global'
    let scope_id = req.user.unwrap_or_else(|| "global".to_string());

    // Call the proxy with memory
    let mut proxy_lock = proxy.lock().await;
    match proxy_lock.complete_with_memory(&provider_name, &scope_id, internal_req).await {
        Ok(resp) => {
            // Map back to OpenAI response
            Json(OpenAiResponse {
                id: format!("st-{}", uuid::Uuid::new_v4()),
                object: "chat.completion".to_string(),
                created: chrono::Utc::now().timestamp() as u64,
                model: req.model,
                choices: vec![OpenAiChoice {
                    index: 0,
                    message: OpenAiResponseMessage {
                        role: "assistant".to_string(),
                        content: resp.content,
                    },
                    finish_reason: "stop".to_string(),
                }],
                usage: resp.usage.map(|u| OpenAiUsage {
                    prompt_tokens: u.prompt_tokens,
                    completion_tokens: u.completion_tokens,
                    total_tokens: u.total_tokens,
                }),
            })
        }
        Err(e) => {
            // Return an error response
            Json(OpenAiResponse {
                id: "error".to_string(),
                object: "error".to_string(),
                created: 0,
                model: req.model,
                choices: vec![OpenAiChoice {
                    index: 0,
                    message: OpenAiResponseMessage {
                        role: "assistant".to_string(),
                        content: format!("Error: {}", e),
                    },
                    finish_reason: "error".to_string(),
                }],
                usage: None,
            })
        }
    }
}

// --- OpenAI API Types ---

#[derive(Debug, Deserialize)]
struct OpenAiRequest {
    model: String,
    messages: Vec<OpenAiMessage>,
    temperature: Option<f32>,
    #[serde(rename = "max_tokens")]
    max_tokens: Option<usize>,
    stream: Option<bool>,
    user: Option<String>,
}

#[derive(Debug, Deserialize)]
struct OpenAiMessage {
    role: String,
    content: String,
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

#[derive(Debug, Serialize)]
struct OpenAiResponse {
    id: String,
    object: String,
    created: u64,
    model: String,
    choices: Vec<OpenAiChoice>,
    #[serde(skip_serializing_if = "Option::is_none")]
    usage: Option<OpenAiUsage>,
}

#[derive(Debug, Serialize)]
struct OpenAiChoice {
    index: usize,
    message: OpenAiResponseMessage,
    finish_reason: String,
}

#[derive(Debug, Serialize)]
struct OpenAiResponseMessage {
    role: String,
    content: String,
}

#[derive(Debug, Serialize)]
struct OpenAiUsage {
    prompt_tokens: usize,
    completion_tokens: usize,
    total_tokens: usize,
}
