//! Smart Tree Daemon - System-wide AI Context Service
//!
//! Runs smart-tree as a persistent background service that any AI can connect to.
//! Provides:
//! - HTTP API for context queries
//! - WebSocket for real-time updates
//! - Foken GPU credit tracking
//! - MCP-compatible tool interface
//! - **LLM Proxy** - Unified interface to multiple AI providers with memory!
//!
//! "The always-on brain for your system!" - Cheet
//!
//! ## Architecture
//! All AI features route through the daemon for persistent memory and unified state.
//! The LLM proxy (OpenAI-compatible at /v1/chat/completions) is integrated directly.

use anyhow::Result;
use axum::{
    extract::{Query, State, WebSocketUpgrade},
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::net::SocketAddr;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::oneshot;
use tokio::sync::RwLock;

// LLM Proxy integration
use crate::proxy::memory::ProxyMemory;
use crate::proxy::openai_compat::{
    OpenAiChoice, OpenAiError, OpenAiErrorResponse, OpenAiRequest, OpenAiResponse,
    OpenAiResponseMessage, OpenAiUsage,
};
use crate::proxy::{LlmMessage, LlmProxy, LlmRequest, LlmRole};

/// Daemon configuration
#[derive(Debug, Clone)]
pub struct DaemonConfig {
    /// HTTP port (default: 8420)
    pub port: u16,
    /// Directories to watch
    pub watch_paths: Vec<PathBuf>,
    /// GPU orchestrator URL for credit sync
    pub orchestrator_url: Option<String>,
    /// Enable credit tracking
    pub enable_credits: bool,
}

impl Default for DaemonConfig {
    fn default() -> Self {
        Self {
            port: 8420,
            watch_paths: vec![],
            orchestrator_url: Some("wss://gpu.foken.ai/api/credits".to_string()),
            enable_credits: true,
        }
    }
}

/// Daemon state - The unified AI brain
pub struct DaemonState {
    /// System context
    pub context: SystemContext,
    /// Foken credit balance
    pub credits: CreditTracker,
    /// Configuration
    pub config: DaemonConfig,
    /// Shutdown signal sender
    pub shutdown_tx: Option<oneshot::Sender<()>>,
    /// LLM Proxy - unified interface to all AI providers
    pub llm_proxy: LlmProxy,
    /// Proxy memory - persistent conversation history
    pub proxy_memory: ProxyMemory,
}

/// System-wide context
#[derive(Debug, Default)]
pub struct SystemContext {
    /// Known projects
    pub projects: HashMap<PathBuf, ProjectInfo>,
    /// Directory consciousnesses
    pub consciousnesses: HashMap<PathBuf, DirectoryInfo>,
    /// Last scan timestamp
    pub last_scan: Option<std::time::SystemTime>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ProjectInfo {
    pub path: String,
    pub name: String,
    pub project_type: String,
    pub key_files: Vec<String>,
    pub essence: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct DirectoryInfo {
    pub path: String,
    pub frequency: f64,
    pub file_count: usize,
    pub patterns: Vec<String>,
}

/// Credit tracker for Foken earnings
#[derive(Debug, Default)]
pub struct CreditTracker {
    pub balance: f64,
    pub total_earned: f64,
    pub total_spent: f64,
    pub transactions: Vec<Transaction>,
}

#[derive(Debug, Clone, Serialize)]
pub struct Transaction {
    pub timestamp: String,
    pub amount: f64,
    pub description: String,
}

impl CreditTracker {
    pub fn record_savings(&mut self, tokens_saved: u64, description: &str) {
        let amount = tokens_saved as f64;
        self.balance += amount;
        self.total_earned += amount;
        self.transactions.push(Transaction {
            timestamp: chrono::Utc::now().to_rfc3339(),
            amount,
            description: description.to_string(),
        });
    }
}

/// Start the daemon server
pub async fn start_daemon(config: DaemonConfig) -> Result<()> {
    println!(
        r#"
    ╔═══════════════════════════════════════════════════════════╗
    ║                                                           ║
    ║   🌳 SMART TREE DAEMON - System AI Context Service 🌳    ║
    ║                                                           ║
    ╚═══════════════════════════════════════════════════════════╝
    "#
    );

    // Create shutdown channel
    let (shutdown_tx, shutdown_rx) = oneshot::channel::<()>();

    // Initialize LLM proxy with available providers
    let llm_proxy = LlmProxy::default();
    let provider_count = llm_proxy.providers.len();

    // Initialize proxy memory for conversation persistence
    let proxy_memory = ProxyMemory::new().unwrap_or_else(|e| {
        eprintln!("Warning: Could not initialize proxy memory: {}", e);
        eprintln!("  Falling back to in-memory only mode (no persistence)");
        // Create a fallback in-memory only version that doesn't require filesystem access
        ProxyMemory::in_memory_only()
    });

    let state = Arc::new(RwLock::new(DaemonState {
        context: SystemContext::default(),
        credits: CreditTracker::default(),
        config: config.clone(),
        shutdown_tx: Some(shutdown_tx),
        llm_proxy,
        proxy_memory,
    }));

    println!("  🤖 LLM Providers: {} available", provider_count);

    // Initial context scan
    {
        let mut s = state.write().await;
        scan_system_context(&mut s.context, &config.watch_paths)?;
    }

    // Start background context watcher
    let state_clone = Arc::clone(&state);
    let watch_paths = config.watch_paths.clone();
    tokio::spawn(async move {
        loop {
            tokio::time::sleep(tokio::time::Duration::from_secs(300)).await;
            if let Ok(mut s) = state_clone.try_write() {
                let _ = scan_system_context(&mut s.context, &watch_paths);
            }
        }
    });

    let app = Router::new()
        // Health & Info
        .route("/health", get(health))
        .route("/info", get(info))
        // Context endpoints
        .route("/context", get(get_context))
        .route("/context/projects", get(get_projects))
        .route("/context/query", post(query_context))
        .route("/context/files", get(list_files))
        // Credit endpoints
        .route("/credits", get(get_credits))
        .route("/credits/record", post(record_credit))
        // MCP-style tool interface
        .route("/tools", get(list_tools))
        .route("/tools/call", post(call_tool))
        // LLM Proxy - OpenAI-compatible chat completions
        .route("/v1/chat/completions", post(chat_completions))
        .route("/v1/models", get(list_models))
        // WebSocket for real-time
        .route("/ws", get(websocket_handler))
        // Daemon control
        .route("/shutdown", post(shutdown_handler))
        .route("/ping", get(ping))
        .with_state(state);

    let addr = SocketAddr::from(([0, 0, 0, 0], config.port));
    println!("Smart Tree Daemon listening on http://{}", addr);
    println!("  - Context API:  /context");
    println!("  - Credits:      /credits");
    println!("  - Tools:        /tools");
    println!("  - LLM Proxy:    /v1/chat/completions (OpenAI-compatible!)");
    println!("  - Models:       /v1/models");
    println!("  - WebSocket:    /ws");
    println!("  - Shutdown:     POST /shutdown");

    let listener = tokio::net::TcpListener::bind(addr).await?;

    // Serve with graceful shutdown support
    axum::serve(listener, app)
        .with_graceful_shutdown(async {
            shutdown_rx.await.ok();
            println!("\n🌳 Smart Tree Daemon shutting down gracefully...");
        })
        .await?;

    println!("🌳 Smart Tree Daemon stopped.");
    Ok(())
}

// API Handlers

async fn health() -> &'static str {
    "ok"
}

#[derive(Serialize)]
struct InfoResponse {
    name: &'static str,
    version: &'static str,
    description: &'static str,
}

async fn info() -> Json<InfoResponse> {
    Json(InfoResponse {
        name: "smart-tree-daemon",
        version: env!("CARGO_PKG_VERSION"),
        description: "System-wide AI context service with Foken credit tracking",
    })
}

#[derive(Serialize)]
struct ContextResponse {
    projects_count: usize,
    directories_count: usize,
    last_scan: Option<String>,
    credits_balance: f64,
}

async fn get_context(State(state): State<Arc<RwLock<DaemonState>>>) -> Json<ContextResponse> {
    let s = state.read().await;
    Json(ContextResponse {
        projects_count: s.context.projects.len(),
        directories_count: s.context.consciousnesses.len(),
        last_scan: s
            .context
            .last_scan
            .map(|t| chrono::DateTime::<chrono::Utc>::from(t).to_rfc3339()),
        credits_balance: s.credits.balance,
    })
}

async fn get_projects(State(state): State<Arc<RwLock<DaemonState>>>) -> Json<Vec<ProjectInfo>> {
    let s = state.read().await;
    Json(s.context.projects.values().cloned().collect())
}

#[derive(Deserialize)]
struct ContextQuery {
    query: String,
}

#[derive(Serialize)]
struct QueryResult {
    projects: Vec<ProjectInfo>,
    files: Vec<String>,
    suggestion: String,
}

async fn query_context(
    State(state): State<Arc<RwLock<DaemonState>>>,
    Json(req): Json<ContextQuery>,
) -> Json<QueryResult> {
    let s = state.read().await;
    let query_lower = req.query.to_lowercase();

    // Find relevant projects
    let projects: Vec<ProjectInfo> = s
        .context
        .projects
        .values()
        .filter(|p| {
            p.name.to_lowercase().contains(&query_lower)
                || p.essence.to_lowercase().contains(&query_lower)
                || p.key_files
                    .iter()
                    .any(|f| f.to_lowercase().contains(&query_lower))
        })
        .cloned()
        .collect();

    // Find relevant files
    let files: Vec<String> = projects
        .iter()
        .flat_map(|p| p.key_files.iter().map(|f| format!("{}/{}", p.path, f)))
        .take(20)
        .collect();

    let suggestion = if projects.is_empty() {
        format!(
            "No projects found matching '{}'. Try a different query.",
            req.query
        )
    } else {
        format!(
            "Found {} projects. Top match: {}",
            projects.len(),
            projects[0].name
        )
    };

    Json(QueryResult {
        projects,
        files,
        suggestion,
    })
}

#[derive(Deserialize)]
struct ListFilesQuery {
    path: Option<String>,
    pattern: Option<String>,
    depth: Option<usize>,
}

async fn list_files(Query(params): Query<ListFilesQuery>) -> Json<Vec<String>> {
    use walkdir::WalkDir;

    let path = params.path.unwrap_or_else(|| ".".to_string());
    let depth = params.depth.unwrap_or(3);

    let files: Vec<String> = WalkDir::new(&path)
        .max_depth(depth)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.path().is_file())
        .filter(|e| {
            if let Some(ref pat) = params.pattern {
                e.path().to_string_lossy().contains(pat)
            } else {
                true
            }
        })
        .take(100)
        .map(|e| e.path().to_string_lossy().to_string())
        .collect();

    Json(files)
}

#[derive(Serialize)]
struct CreditsResponse {
    balance: f64,
    total_earned: f64,
    total_spent: f64,
    recent_transactions: Vec<Transaction>,
}

async fn get_credits(State(state): State<Arc<RwLock<DaemonState>>>) -> Json<CreditsResponse> {
    let s = state.read().await;
    Json(CreditsResponse {
        balance: s.credits.balance,
        total_earned: s.credits.total_earned,
        total_spent: s.credits.total_spent,
        recent_transactions: s
            .credits
            .transactions
            .iter()
            .rev()
            .take(10)
            .cloned()
            .collect(),
    })
}

#[derive(Deserialize)]
struct RecordCreditRequest {
    tokens_saved: u64,
    description: String,
}

async fn record_credit(
    State(state): State<Arc<RwLock<DaemonState>>>,
    Json(req): Json<RecordCreditRequest>,
) -> Json<CreditsResponse> {
    let mut s = state.write().await;
    s.credits.record_savings(req.tokens_saved, &req.description);

    Json(CreditsResponse {
        balance: s.credits.balance,
        total_earned: s.credits.total_earned,
        total_spent: s.credits.total_spent,
        recent_transactions: s
            .credits
            .transactions
            .iter()
            .rev()
            .take(10)
            .cloned()
            .collect(),
    })
}

#[derive(Serialize)]
struct Tool {
    name: String,
    description: String,
}

async fn list_tools() -> Json<Vec<Tool>> {
    Json(vec![
        Tool {
            name: "get_context".to_string(),
            description: "Get system context summary".to_string(),
        },
        Tool {
            name: "list_projects".to_string(),
            description: "List all detected projects".to_string(),
        },
        Tool {
            name: "query_context".to_string(),
            description: "Search context by keyword".to_string(),
        },
        Tool {
            name: "list_files".to_string(),
            description: "List files in a directory".to_string(),
        },
        Tool {
            name: "get_credits".to_string(),
            description: "Get Foken credit balance".to_string(),
        },
        Tool {
            name: "record_savings".to_string(),
            description: "Record token compression savings".to_string(),
        },
    ])
}

#[derive(Deserialize)]
struct ToolCall {
    name: String,
    arguments: serde_json::Value,
}

async fn call_tool(
    State(state): State<Arc<RwLock<DaemonState>>>,
    Json(call): Json<ToolCall>,
) -> impl IntoResponse {
    match call.name.as_str() {
        "get_context" => {
            let s = state.read().await;
            (
                StatusCode::OK,
                Json(serde_json::json!({
                    "projects": s.context.projects.len(),
                    "directories": s.context.consciousnesses.len(),
                    "credits": s.credits.balance
                })),
            )
        }
        "list_projects" => {
            let s = state.read().await;
            let projects: Vec<_> = s.context.projects.values().cloned().collect();
            (
                StatusCode::OK,
                Json(serde_json::json!({ "projects": projects })),
            )
        }
        "list_files" => {
            let path = call
                .arguments
                .get("path")
                .and_then(|v| v.as_str())
                .unwrap_or(".");
            let depth = call
                .arguments
                .get("depth")
                .and_then(|v| v.as_u64())
                .unwrap_or(3) as usize;

            use walkdir::WalkDir;
            let files: Vec<String> = WalkDir::new(path)
                .max_depth(depth)
                .into_iter()
                .filter_map(|e| e.ok())
                .filter(|e| e.path().is_file())
                .take(100)
                .map(|e| e.path().to_string_lossy().to_string())
                .collect();

            (StatusCode::OK, Json(serde_json::json!({ "files": files })))
        }
        _ => (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({
                "error": format!("Unknown tool: {}", call.name)
            })),
        ),
    }
}

async fn websocket_handler(
    ws: WebSocketUpgrade,
    State(_state): State<Arc<RwLock<DaemonState>>>,
) -> impl IntoResponse {
    ws.on_upgrade(|_socket| async {
        // WebSocket handling for real-time updates
        // TODO: Implement real-time context streaming
    })
}

/// Ping handler - quick check that daemon is responding
async fn ping() -> &'static str {
    "pong"
}

/// Shutdown handler - gracefully stop the daemon
async fn shutdown_handler(State(state): State<Arc<RwLock<DaemonState>>>) -> impl IntoResponse {
    // Take the shutdown sender and trigger shutdown
    let mut s = state.write().await;
    if let Some(tx) = s.shutdown_tx.take() {
        // Send shutdown signal
        let _ = tx.send(());
        (
            StatusCode::OK,
            Json(serde_json::json!({
                "status": "shutting_down",
                "message": "Smart Tree Daemon is shutting down gracefully"
            })),
        )
    } else {
        (
            StatusCode::CONFLICT,
            Json(serde_json::json!({
                "status": "error",
                "message": "Shutdown already in progress"
            })),
        )
    }
}

/// Scan system for projects and context
fn scan_system_context(context: &mut SystemContext, watch_paths: &[PathBuf]) -> Result<()> {
    use walkdir::WalkDir;

    for path in watch_paths {
        if !path.exists() {
            continue;
        }

        for entry in WalkDir::new(path)
            .max_depth(3)
            .follow_links(false)
            .into_iter()
            .filter_map(|e| e.ok())
        {
            let entry_path = entry.path();

            // Skip hidden directories
            if entry_path
                .file_name()
                .map(|n| n.to_string_lossy().starts_with('.'))
                .unwrap_or(false)
            {
                continue;
            }

            if entry_path.is_dir() {
                // Detect project
                if let Some(project) = detect_project(entry_path) {
                    context.projects.insert(entry_path.to_path_buf(), project);
                }

                // Create directory info
                if let Some(info) = create_directory_info(entry_path) {
                    context
                        .consciousnesses
                        .insert(entry_path.to_path_buf(), info);
                }
            }
        }
    }

    context.last_scan = Some(std::time::SystemTime::now());
    Ok(())
}

fn detect_project(path: &std::path::Path) -> Option<ProjectInfo> {
    let markers = [
        ("Cargo.toml", "Rust"),
        ("package.json", "JavaScript"),
        ("pyproject.toml", "Python"),
        ("go.mod", "Go"),
    ];

    for (marker, project_type) in markers {
        if path.join(marker).exists() {
            let name = path.file_name()?.to_string_lossy().to_string();

            let key_files: Vec<String> = ["README.md", "CLAUDE.md", "src/main.rs", "src/lib.rs"]
                .iter()
                .filter(|f| path.join(f).exists())
                .map(|f| f.to_string())
                .collect();

            let essence = read_essence(path).unwrap_or_else(|| format!("{} project", project_type));

            return Some(ProjectInfo {
                path: path.to_string_lossy().to_string(),
                name,
                project_type: project_type.to_string(),
                key_files,
                essence,
            });
        }
    }
    None
}

fn read_essence(path: &std::path::Path) -> Option<String> {
    for readme in ["CLAUDE.md", "README.md"] {
        let readme_path = path.join(readme);
        if readme_path.exists() {
            if let Ok(content) = std::fs::read_to_string(&readme_path) {
                for line in content.lines() {
                    let line = line.trim();
                    if !line.is_empty() && !line.starts_with('#') && !line.starts_with("```") {
                        return Some(line.chars().take(100).collect());
                    }
                }
            }
        }
    }
    None
}

fn create_directory_info(path: &std::path::Path) -> Option<DirectoryInfo> {
    use std::collections::HashSet;
    use walkdir::WalkDir;

    let mut file_count = 0;
    let mut extensions: HashSet<String> = HashSet::new();

    for entry in WalkDir::new(path)
        .max_depth(1)
        .into_iter()
        .filter_map(|e| e.ok())
    {
        if entry.path().is_file() {
            file_count += 1;
            if let Some(ext) = entry.path().extension() {
                extensions.insert(ext.to_string_lossy().to_string());
            }
        }
    }

    // Calculate frequency from path hash
    use std::hash::{Hash, Hasher};
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    path.hash(&mut hasher);
    let hash = hasher.finish();
    let frequency = 20.0 + (hash % 18000) as f64 / 100.0;

    Some(DirectoryInfo {
        path: path.to_string_lossy().to_string(),
        frequency,
        file_count,
        patterns: extensions.into_iter().collect(),
    })
}

// =============================================================================
// LLM PROXY HANDLERS - OpenAI-compatible chat completions
// =============================================================================

/// 💬 Chat completions handler - routes to appropriate LLM provider
async fn chat_completions(
    State(state): State<Arc<RwLock<DaemonState>>>,
    Json(req): Json<OpenAiRequest>,
) -> impl IntoResponse {
    // Parse provider from model name (e.g., "anthropic/claude-3" or just "gpt-4")
    let (provider_name, model_name) = if let Some((p, m)) = req.model.split_once('/') {
        (p.to_string(), m.to_string())
    } else {
        ("openai".to_string(), req.model.clone())
    };

    let internal_req = LlmRequest {
        model: model_name,
        messages: req.messages.into_iter().map(Into::into).collect(),
        temperature: req.temperature,
        max_tokens: req.max_tokens,
        stream: req.stream.unwrap_or(false),
    };

    // Use 'user' field as scope ID for memory, default to 'global'
    let scope_id = req.user.clone().unwrap_or_else(|| "global".to_string());

    // Build request with history while holding a write lock briefly
    let request_with_history = {
        let state_lock = state.read().await;

        // Get conversation history from memory
        let mut messages_with_history = Vec::new();

        // Keep system message at the top if present
        if let Some(system_msg) = internal_req
            .messages
            .iter()
            .find(|m| m.role == LlmRole::System)
            .cloned()
        {
            messages_with_history.push(system_msg);
        }

        // Add history from memory
        if let Some(scope) = state_lock.proxy_memory.get_scope(&scope_id) {
            for msg in &scope.messages {
                if msg.role != LlmRole::System {
                    messages_with_history.push(msg.clone());
                }
            }
        }

        // Add current messages (excluding system which is already added)
        for msg in &internal_req.messages {
            if msg.role != LlmRole::System {
                messages_with_history.push(msg.clone());
            }
        }

        LlmRequest {
            messages: messages_with_history,
            ..internal_req.clone()
        }
    };

    // Call the LLM provider with a read lock (doesn't need mutable access)
    let llm_result = {
        let state_lock = state.read().await;
        state_lock
            .llm_proxy
            .complete(&provider_name, request_with_history)
            .await
    };

    match llm_result {
        Ok(resp) => {
            // Reacquire write lock for memory/credits updates
            let mut state_lock = state.write().await;

            // Update memory with this exchange
            let mut new_history = Vec::new();
            if let Some(last_user_msg) = internal_req
                .messages
                .iter()
                .rev()
                .find(|m| m.role == LlmRole::User)
            {
                new_history.push(last_user_msg.clone());
            }
            new_history.push(LlmMessage {
                role: LlmRole::Assistant,
                content: resp.content.clone(),
            });
            let _ = state_lock.proxy_memory.update_scope(&scope_id, new_history);

            // Record credit for token savings (if we compressed context)
            let tokens_used = resp.usage.as_ref().map(|u| u.total_tokens).unwrap_or(0);
            if tokens_used > 0 {
                state_lock.credits.record_savings(
                    tokens_used as u64 / 10, // Award 10% as savings
                    &format!("LLM call to {} ({})", provider_name, req.model),
                );
            }

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

/// List available models from all providers
async fn list_models(State(state): State<Arc<RwLock<DaemonState>>>) -> Json<serde_json::Value> {
    let state_lock = state.read().await;

    let models: Vec<serde_json::Value> = state_lock
        .llm_proxy
        .providers
        .iter()
        .map(|p| {
            serde_json::json!({
                "id": format!("{}/default", p.name().to_lowercase()),
                "object": "model",
                "owned_by": p.name(),
            })
        })
        .collect();

    Json(serde_json::json!({
        "object": "list",
        "data": models
    }))
}
