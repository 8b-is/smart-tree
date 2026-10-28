//! OpenAI-Compatible Proxy Server
//!
//! HTTP server implementing the OpenAI Chat Completions API plus a small admin
//! surface for status and provider listing.
//!
//! Endpoints:
//!   POST /v1/chat/completions  - OpenAI-compatible chat
//!   GET  /v1/models            - List available models (across providers)
//!   GET  /admin/status         - Proxy status: providers + auth state
//!
//! Optional bearer auth: set `ST_PROXY_API_KEY` in the env. When set, every
//! request must carry `Authorization: Bearer <key>`. When unset, the proxy is
//! open (loopback only by default).

use crate::proxy::memory::MemoryProxy;
use crate::proxy::openai_compat::{
    OpenAiChoice, OpenAiError, OpenAiErrorResponse, OpenAiRequest, OpenAiResponse,
    OpenAiResponseMessage, OpenAiUsage,
};
use crate::proxy::LlmRequest;
use anyhow::Result;
use axum::{
    extract::State,
    http::{Request, StatusCode},
    middleware::{self, Next},
    response::{IntoResponse, Response},
    routing::{get, post},
    Json, Router,
};
use serde::Serialize;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::sync::RwLock;

type SharedProxy = Arc<RwLock<MemoryProxy>>;

/// Start the OpenAI-compatible proxy server.
pub async fn start_proxy_server(port: u16) -> Result<()> {
    let proxy: SharedProxy = Arc::new(RwLock::new(MemoryProxy::new()?));

    let app = Router::new()
        .route("/v1/chat/completions", post(chat_completions))
        .route("/v1/models", get(list_models))
        .route("/admin/status", get(admin_status))
        .layer(middleware::from_fn(bearer_auth))
        .with_state(proxy);

    let addr = SocketAddr::from(([127, 0, 0, 1], port));
    println!("Smart Tree LLM Proxy on http://{}", addr);
    println!("  POST /v1/chat/completions");
    println!("  GET  /v1/models");
    println!("  GET  /admin/status");
    if std::env::var("ST_PROXY_API_KEY").is_ok() {
        println!("  Auth: bearer (ST_PROXY_API_KEY required)");
    } else {
        println!("  Auth: open (set ST_PROXY_API_KEY to require Bearer token)");
    }

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;
    Ok(())
}

/// Bearer-token middleware. No-op when ST_PROXY_API_KEY is unset.
async fn bearer_auth(req: Request<axum::body::Body>, next: Next) -> Response {
    let Ok(expected) = std::env::var("ST_PROXY_API_KEY") else {
        return next.run(req).await;
    };

    let provided = req
        .headers()
        .get("authorization")
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.strip_prefix("Bearer "))
        .map(|s| s.trim().to_string());

    match provided {
        Some(token) if token == expected => next.run(req).await,
        _ => (
            StatusCode::UNAUTHORIZED,
            Json(OpenAiErrorResponse {
                error: OpenAiError {
                    message: "missing or invalid bearer token".into(),
                    error_type: "authentication_error".into(),
                    code: Some("invalid_api_key".into()),
                },
            }),
        )
            .into_response(),
    }
}

async fn chat_completions(
    State(proxy): State<SharedProxy>,
    Json(req): Json<OpenAiRequest>,
) -> Response {
    let (provider_name, model_name) = match req.model.split_once('/') {
        Some((p, m)) => (p.to_string(), m.to_string()),
        None => ("openai".to_string(), req.model.clone()),
    };

    let internal_req = LlmRequest {
        model: model_name,
        messages: req.messages.into_iter().map(Into::into).collect(),
        temperature: req.temperature,
        max_tokens: req.max_tokens,
        stream: req.stream.unwrap_or(false),
    };

    let scope_id = req.user.unwrap_or_else(|| "global".to_string());

    let mut proxy_lock = proxy.write().await;
    match proxy_lock
        .complete_with_memory(&provider_name, &scope_id, internal_req)
        .await
    {
        Ok(resp) => (
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
            .into_response(),
        Err(e) => {
            let msg = e.to_string();
            let status = if msg.contains("not found") || msg.contains("invalid") {
                StatusCode::BAD_REQUEST
            } else if msg.contains("unauthorized") || msg.contains("authentication") {
                StatusCode::UNAUTHORIZED
            } else {
                StatusCode::INTERNAL_SERVER_ERROR
            };
            (
                status,
                Json(OpenAiErrorResponse {
                    error: OpenAiError {
                        message: msg,
                        error_type: "api_error".into(),
                        code: None,
                    },
                }),
            )
                .into_response()
        }
    }
}

#[derive(Serialize)]
struct ModelEntry {
    id: String,
    object: &'static str,
    owned_by: String,
}

#[derive(Serialize)]
struct ModelList {
    object: &'static str,
    data: Vec<ModelEntry>,
}

/// GET /v1/models — returns an OpenAI-style list.
/// Each provider contributes a single placeholder entry `<provider>/default`;
/// callers can request specific models with the `<provider>/<model>` syntax.
async fn list_models(State(proxy): State<SharedProxy>) -> Response {
    let lock = proxy.read().await;
    let data: Vec<ModelEntry> = lock
        .inner
        .list_providers()
        .into_iter()
        .map(|p| ModelEntry {
            id: format!("{}/default", p.to_lowercase()),
            object: "model",
            owned_by: p.to_string(),
        })
        .collect();

    Json(ModelList {
        object: "list",
        data,
    })
    .into_response()
}

#[derive(Serialize)]
struct AdminStatus {
    running: bool,
    auth_required: bool,
    providers: Vec<&'static str>,
    version: &'static str,
}

async fn admin_status(State(proxy): State<SharedProxy>) -> Response {
    let lock = proxy.read().await;
    Json(AdminStatus {
        running: true,
        auth_required: std::env::var("ST_PROXY_API_KEY").is_ok(),
        providers: lock.inner.list_providers(),
        version: env!("CARGO_PKG_VERSION"),
    })
    .into_response()
}
