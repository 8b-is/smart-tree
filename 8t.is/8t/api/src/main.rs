use axum::{
    extract::{Path, State},
    http::{header, StatusCode},
    response::{IntoResponse, Response},
    routing::{get, post},
    Json, Router,
};
use eighty_core::{Protocol, ToolInfo, ToolRegistry};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tower_http::cors::CorsLayer;
use tracing::info;

#[derive(Clone)]
struct AppState {
    registry: Arc<ToolRegistry>,
}

#[derive(Serialize)]
struct ApiInfo {
    name: String,
    version: String,
    description: String,
    endpoints: Vec<EndpointInfo>,
    supported_protocols: Vec<String>,
}

#[derive(Serialize)]
struct EndpointInfo {
    path: String,
    method: String,
    description: String,
}

#[derive(Serialize)]
struct ErrorResponse {
    error: String,
    code: u16,
}

#[derive(Deserialize)]
struct ProcessRequest {
    data: Vec<u8>,
    protocol: Option<Protocol>,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive("eighty_api=debug".parse()?)
        )
        .init();

    let registry = Arc::new(ToolRegistry::new());
    let state = AppState { registry };

    let app = Router::new()
        .route("/", get(root_handler))
        .route("/info", get(info_handler))
        .route("/tools", get(list_tools))
        .route("/tools/:name", post(process_tool))
        .route("/feedback", post(submit_feedback))
        .route("/health", get(health_check))
        .layer(CorsLayer::permissive())
        .with_state(state);

    let addr = "[::]:8420";
    info!("🚀 8t API server listening on {}", addr);
    
    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;
    
    Ok(())
}

async fn root_handler() -> impl IntoResponse {
    "8t API - Get 80 before you get 80x the context! 🎸"
}

async fn info_handler(headers: axum::http::HeaderMap) -> Response {
    let info = ApiInfo {
        name: "8t API Server".to_string(),
        version: env!("CARGO_PKG_VERSION").to_string(),
        description: "Quantum context protocol and semantic tool orchestration".to_string(),
        endpoints: vec![
            EndpointInfo {
                path: "/".to_string(),
                method: "GET".to_string(),
                description: "Root endpoint with welcome message".to_string(),
            },
            EndpointInfo {
                path: "/info".to_string(),
                method: "GET".to_string(),
                description: "API information (content-type aware)".to_string(),
            },
            EndpointInfo {
                path: "/tools".to_string(),
                method: "GET".to_string(),
                description: "List available tools".to_string(),
            },
            EndpointInfo {
                path: "/tools/{name}".to_string(),
                method: "POST".to_string(),
                description: "Process data with specific tool".to_string(),
            },
            EndpointInfo {
                path: "/feedback".to_string(),
                method: "POST".to_string(),
                description: "Submit AI feedback for continuous improvement".to_string(),
            },
            EndpointInfo {
                path: "/health".to_string(),
                method: "GET".to_string(),
                description: "Health check endpoint".to_string(),
            },
        ],
        supported_protocols: vec![
            "json".to_string(),
            "messagepack".to_string(),
            "qcp".to_string(),
        ],
    };

    // Content negotiation based on Accept header
    let accept = headers
        .get(header::ACCEPT)
        .and_then(|v| v.to_str().ok())
        .unwrap_or("application/json");

    match accept {
        "application/json" | "*/*" => {
            Json(info).into_response()
        }
        "text/plain" => {
            let text = format!(
                "{} v{}\n{}\n\nEndpoints:\n{}",
                info.name,
                info.version,
                info.description,
                info.endpoints
                    .iter()
                    .map(|e| format!("  {} {} - {}", e.method, e.path, e.description))
                    .collect::<Vec<_>>()
                    .join("\n")
            );
            text.into_response()
        }
        _ => {
            // For now, default to JSON for unknown types
            Json(info).into_response()
        }
    }
}

async fn list_tools(State(state): State<AppState>) -> Json<Vec<ToolInfo>> {
    Json(state.registry.list())
}

async fn process_tool(
    State(state): State<AppState>,
    Path(name): Path<String>,
    Json(req): Json<ProcessRequest>,
) -> Response {
    let protocol = req.protocol.unwrap_or(Protocol::Json);
    
    match state.registry.process(&name, &req.data, protocol) {
        Ok(result) => {
            // Return raw bytes with appropriate content-type
            let content_type = match protocol {
                Protocol::Json => "application/json",
                Protocol::MessagePack => "application/msgpack",
                Protocol::Qcp => "application/octet-stream",
            };
            
            Response::builder()
                .status(StatusCode::OK)
                .header(header::CONTENT_TYPE, content_type)
                .body(result.into())
                .unwrap()
        }
        Err(e) => {
            let error = ErrorResponse {
                error: e.to_string(),
                code: 400,
            };
            (StatusCode::BAD_REQUEST, Json(error)).into_response()
        }
    }
}

async fn health_check() -> impl IntoResponse {
    Json(serde_json::json!({
        "status": "healthy",
        "timestamp": chrono::Utc::now().to_rfc3339(),
    }))
}

async fn submit_feedback(Json(submission): Json<serde_json::Value>) -> Response {
    // For now, just acknowledge receipt
    // In production, this would forward to the feedback system
    info!("Received AI feedback: {:?}", submission);
    
    Json(serde_json::json!({
        "status": "accepted",
        "message": "Feedback received and queued for processing",
        "timestamp": chrono::Utc::now().to_rfc3339(),
    })).into_response()
}