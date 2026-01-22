//! 🌐 OpenAI-Compatible Proxy Server
//!
//! This module provides an HTTP server that implements the OpenAI Chat Completions API,
//! allowing any OpenAI-compatible client to use Smart Tree as a backend.
//!
//! "Serving AI requests with a smile and a tree!" - The Cheet 😺

use crate::proxy::memory::MemoryProxy;
use crate::proxy::openai_compat::{
    OpenAiChoice, OpenAiError, OpenAiErrorResponse, OpenAiRequest, OpenAiResponse,
    OpenAiResponseMessage, OpenAiUsage,
};
use crate::proxy::LlmRequest;
use anyhow::Result;
use axum::{extract::State, http::StatusCode, response::IntoResponse, routing::post, Json, Router};
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::sync::RwLock;

/// 🚀 Start the OpenAI-compatible proxy server
pub async fn start_proxy_server(port: u16) -> Result<()> {
    let proxy = Arc::new(RwLock::new(MemoryProxy::new()?));

    let app = Router::new()
        .route("/v1/chat/completions", post(chat_completions))
        .with_state(proxy);

    let addr = SocketAddr::from(([127, 0, 0, 1], port));
    println!("🚀 Smart Tree LLM Proxy Server running on http://{}", addr);
    println!(
        "🌳 OpenAI-compatible endpoint: http://{}/v1/chat/completions",
        addr
    );
    println!("🧠 Memory enabled with scoped conversation history!");

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}

/// 💬 OpenAI-compatible chat completions handler
async fn chat_completions(
    State(proxy): State<Arc<RwLock<MemoryProxy>>>,
    Json(req): Json<OpenAiRequest>,
) -> impl IntoResponse {
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

    // Call the proxy with memory - lock is only held during the complete_with_memory call
    let mut proxy_lock = proxy.write().await;
    match proxy_lock
        .complete_with_memory(&provider_name, &scope_id, internal_req)
        .await
    {
        Ok(resp) => {
            // Map back to OpenAI response
            (
                StatusCode::OK,
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
                }),
            )
                .into_response()
        }
        Err(e) => {
            let error_msg = format!("{}", e);
            let status = if error_msg.contains("not found") || error_msg.contains("invalid") {
                StatusCode::BAD_REQUEST
            } else if error_msg.contains("unauthorized") || error_msg.contains("authentication") {
                StatusCode::UNAUTHORIZED
            } else {
                StatusCode::INTERNAL_SERVER_ERROR
            };

            (
                status,
                Json(OpenAiErrorResponse {
                    error: OpenAiError {
                        message: error_msg,
                        error_type: "api_error".to_string(),
                        code: None,
                    },
                }),
            )
                .into_response()
        }
    }
}
