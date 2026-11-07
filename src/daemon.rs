//! Smart Tree Daemon - System-wide AI Context Service
//!
//! Runs smart-tree as a persistent background service that any AI can connect to.
//! Provides:
//! - HTTP API for context queries
//! - WebSocket for real-time updates
//! - Foken GPU credit tracking
//! - **HTTP MCP** - Full MCP protocol over HTTP (not just stdio!)
//! - **LLM Proxy** - Unified interface to multiple AI providers with memory!
//! - **Collaboration Station** - Multi-AI real-time collaboration with Hot Tub mode! 🛁
//! - **The Custodian** - Watches all operations for suspicious patterns 🧹
//! - **GitHub Auth** - OAuth for i1.is/aye.is identity
//!
//! "The always-on brain for your system!" - Cheet
//!
//! ## Architecture
//! All AI features route through the daemon for persistent memory and unified state.
//! The LLM proxy (OpenAI-compatible at /v1/chat/completions) is integrated directly.
//! Collaboration hub enables humans and AIs to work together in real-time.
//! The Custodian monitors all MCP operations for data exfiltration and supply chain attacks.

use anyhow::{Context, Result};
use axum::{
    extract::{Query, Request, State, WebSocketUpgrade},
    http::StatusCode,
    middleware::{self, Next},
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};
use chrono::Utc;
use futures_util;
use num_cpus;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::net::SocketAddr;
use std::path::PathBuf;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use tokio::sync::oneshot;
use tokio::sync::RwLock;
use tokio::sync::Semaphore;

// LLM Proxy integration
use crate::proxy::memory::ProxyMemory;
use crate::proxy::openai_compat::{
    OpenAiChoice, OpenAiError, OpenAiErrorResponse, OpenAiRequest, OpenAiResponse,
    OpenAiResponseMessage, OpenAiUsage,
};
use crate::proxy::{LlmMessage, LlmProxy, LlmRequest, LlmRole};

// Collaboration Station
use crate::auth::{create_session_store, GitHubOAuthConfig, SharedSessionStore};
use crate::collaboration::{create_hub, SharedCollabHub};

// Hot Watcher - Wave-powered real-time directory intelligence
mod context_memory;
mod search;
use context_memory::{Association, ContextMemory, RecallRequest};

// HTTP MCP with The Custodian
use crate::web_dashboard::mcp_http::{create_mcp_context, mcp_router};

// =============================================================================
// DAEMON AUTH TOKEN
// =============================================================================

/// Get the path to the daemon auth token file.
/// Respects ST_TOKEN_PATH env var (for systemd StateDirectory), falls back to ~/.st/daemon.token
pub fn token_path() -> PathBuf {
    if let Ok(p) = std::env::var("ST_TOKEN_PATH") {
        return PathBuf::from(p);
    }
    dirs::home_dir()
        .unwrap_or_else(|| PathBuf::from("/tmp"))
        .join(".st")
        .join("daemon.token")
}

/// Load or generate the daemon auth token.
/// Creates a new random token on first run and persists it.
pub fn load_or_create_token() -> Result<String> {
    let path = token_path();

    // Try to read existing token
    if path.exists() {
        let token = std::fs::read_to_string(&path)?.trim().to_string();
        if !token.is_empty() {
            #[cfg(unix)]
            if path.starts_with("/var/lib") {
                use std::os::unix::fs::PermissionsExt;
                let _ = std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o644));
            }
            return Ok(token);
        }
    }

    // Generate new 32-byte random hex token
    use rand::Rng;
    let mut rng = rand::thread_rng();
    let bytes: Vec<u8> = (0..32).map(|_| rng.gen::<u8>()).collect();
    let token = hex::encode(&bytes);

    // Ensure parent directory exists
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }

    // Write token with appropriate permissions
    std::fs::write(&path, &token)?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        // System-level token (/var/lib/smart-tree/) needs to be world-readable
        // so CLI clients can authenticate. User-level token stays private.
        let mode = if path.starts_with("/var/lib") {
            0o644
        } else {
            0o600
        };
        std::fs::set_permissions(&path, std::fs::Permissions::from_mode(mode))?;
    }

    println!("  🔑 Generated new daemon auth token at {}", path.display());
    Ok(token)
}

/// Load existing token (for clients). Returns None if no token file exists.
/// Prioritizes the system-level daemon token.
pub fn load_token() -> Option<String> {
    load_all_tokens().into_iter().next()
}

/// Load all available valid tokens (for servers to accept any valid local token).
pub fn load_all_tokens() -> Vec<String> {
    let mut tokens = Vec::new();

    // 1. Check system-level daemon token
    let system_path = std::path::PathBuf::from("/var/lib/smart-tree/daemon.token");
    if let Ok(token) = std::fs::read_to_string(&system_path) {
        let t = token.trim().to_string();
        if !t.is_empty() {
            tokens.push(t);
        }
    }

    // 2. Add user local token
    let path = token_path();
    if let Ok(token) = std::fs::read_to_string(&path) {
        let t = token.trim().to_string();
        if !t.is_empty() && !tokens.contains(&t) {
            tokens.push(t);
        }
    }

    tokens
}

/// Constant-time token comparison using SHA-256 digests so remote probes
/// cannot infer token prefixes via response-time side channels (FR-ST-01).
fn ct_token_matches(expected: &str, provided: &str) -> bool {
    use sha2::{Digest, Sha256};
    let ha = Sha256::digest(expected.as_bytes());
    let hb = Sha256::digest(provided.as_bytes());
    let mut diff = 0u8;
    for (x, y) in ha.iter().zip(hb.iter()) {
        diff |= x ^ y;
    }
    diff == 0 && expected.len() == provided.len()
}

/// Shared state for the daemon auth middleware.
///
/// When the daemon binds to localhost only (`allow_external == false`), it
/// trusts the local trust boundary exactly like the stdio (`st --mcp`) server:
/// any local process can already spawn `st --mcp` and access the same tools,
/// so requiring a token for MCP HTTP/dashboard on loopback adds friction, not
/// security. The Bearer token is still enforced once the daemon is bound
/// externally (`allow_external == true`), where the token is the actual gate.
#[derive(Debug, Clone)]
pub struct AuthState {
    pub tokens: Vec<String>,
    pub allow_external: bool,
}

impl AuthState {
    pub fn new(tokens: Vec<String>, allow_external: bool) -> Self {
        Self {
            tokens,
            allow_external,
        }
    }
}

/// Auth middleware: validates Bearer token when the daemon is externally bound.
///
/// - `/health` is always public (health checks / monitoring).
/// - Localhost-only daemon (`allow_external == false`): loopback is trusted
///   (same boundary as stdio `st --mcp`), so MCP HTTP + dashboard work with
///   zero setup. A valid Bearer token is still accepted for compatibility.
/// - Externally bound daemon (`allow_external == true`): strict Bearer check
///   with constant-time comparison on every route except `/health`.
async fn auth_middleware(
    State(state): State<AuthState>,
    req: Request,
    next: Next,
) -> impl IntoResponse {
    // Allow /health without auth (for health checks and monitoring)
    if req.uri().path() == "/health" {
        return next.run(req).await;
    }

    // Localhost-only: loopback differs from external by definition of bind.
    if !state.allow_external {
        return next.run(req).await;
    }

    // Externally bound: strict Bearer check.
    let auth_header = req
        .headers()
        .get("authorization")
        .and_then(|v| v.to_str().ok());

    match auth_header {
        Some(header) if header.starts_with("Bearer ") => {
            let provided = &header[7..];
            if state
                .tokens
                .iter()
                .fold(false, |acc, t| acc | ct_token_matches(t, provided))
            {
                next.run(req).await
            } else {
                (StatusCode::UNAUTHORIZED, "Invalid token").into_response()
            }
        }
        _ => (StatusCode::UNAUTHORIZED, "Bearer token required").into_response(),
    }
}

/// Daemon configuration
#[derive(Debug, Clone)]
pub struct DaemonConfig {
    /// HTTP port (default: 28428)
    pub port: u16,
    /// Directories to watch
    pub watch_paths: Vec<PathBuf>,
    /// GPU orchestrator URL for credit sync
    pub orchestrator_url: Option<String>,
    /// Enable credit tracking
    pub enable_credits: bool,
    /// Allow connections from external hosts (default: false, localhost only)
    pub allow_external: bool,
}

impl Default for DaemonConfig {
    fn default() -> Self {
        Self {
            port: 28428,
            watch_paths: vec![],
            orchestrator_url: Some("wss://gpu.foken.ai/api/credits".to_string()),
            enable_credits: true,
            allow_external: false,
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
    pub llm_proxy: Arc<LlmProxy>,
    /// Proxy memory - persistent conversation history
    pub proxy_memory: Arc<std::sync::Mutex<ProxyMemory>>,
    /// Collaboration hub - multi-AI real-time collaboration
    pub collab_hub: SharedCollabHub,
    /// Session store - GitHub OAuth sessions
    pub sessions: SharedSessionStore,
    /// GitHub OAuth config (if available)
    pub github_oauth: Option<GitHubOAuthConfig>,
    /// Hot Watcher - Wave-powered real-time directory intelligence (MEM8)
    pub directory_memory: Arc<std::sync::Mutex<ContextMemory>>,
    pub recall_view: context_memory::RecallView,
    /// Compact persistent security scan and certificate history.
    pub scan_memory: Arc<std::sync::Mutex<crate::magiscanner::memory::ScanMemory>>,

    /// Parallel compute pool for sovereign model inference
    pub compute_pool: Arc<ComputePool>,
}

/// Parallel compute pool for distributing inference across sovereign models
pub struct ComputePool {
    /// Semaphore for limiting concurrent inference jobs
    semaphore: Semaphore,
    /// Worker count
    worker_count: usize,
    /// Active job counter
    active_jobs: AtomicUsize,
    /// Completed job counter
    completed_jobs: AtomicUsize,
    /// Model cache for inference
    model_cache: Arc<crate::mcp::combinatorics_llm::LlamaModelCache>,
}

impl ComputePool {
    /// Create a new compute pool with the given number of workers
    pub fn new(
        worker_count: usize,
        model_cache: Arc<crate::mcp::combinatorics_llm::LlamaModelCache>,
    ) -> Self {
        Self {
            semaphore: Semaphore::new(worker_count),
            worker_count,
            active_jobs: AtomicUsize::new(0),
            completed_jobs: AtomicUsize::new(0),
            model_cache,
        }
    }

    /// Get the number of workers
    pub fn worker_count(&self) -> usize {
        self.worker_count
    }

    /// Get current active job count
    pub fn active_jobs(&self) -> usize {
        self.active_jobs.load(Ordering::Relaxed)
    }

    /// Get total completed jobs
    pub fn completed_jobs(&self) -> usize {
        self.completed_jobs.load(Ordering::Relaxed)
    }

    /// Execute an inference job with a named model
    pub async fn infer_with_model(
        &self,
        model_name: &str,
        prompt: &str,
        max_tokens: usize,
    ) -> anyhow::Result<String> {
        // Acquire semaphore permit
        let permit = self.semaphore.acquire().await?;
        self.active_jobs.fetch_add(1, Ordering::Relaxed);

        let result = async {
            let _model = self
                .model_cache
                .get_named_model(model_name)
                .ok_or_else(|| anyhow::anyhow!("Model '{}' not loaded", model_name))?;

            // For now, use the existing llama_rust inference
            // In a full implementation, this would use the model directly
            let params = crate::mcp::combinatorics_llm::LlamaRustParams {
                model_path: model_name.to_string(),
                prompt: prompt.to_string(),
                max_tokens,
                temperature: 0.7,
                n_gpu_layers: u32::MAX,
                speculative_depth: 4,
                speculative_tree_width: 2,
                n_ctx: 4096,
                n_batch: 2048,
                draft_model_path: None,
                grammar: None,
                json_schema: None,
            };

            let emitter = crate::mcp::combinatorics_llm::McpProgressEmitter::default();
            let result =
                crate::mcp::combinatorics_llm::run_llama_rust(&self.model_cache, params, &emitter);

            match result {
                Ok(v) => v
                    .get("content")
                    .and_then(|c| c.as_array())
                    .and_then(|a| a.first())
                    .and_then(|c| c.get("text"))
                    .and_then(|t| t.as_str())
                    .map(|s| s.to_string())
                    .ok_or_else(|| anyhow::anyhow!("No text in response")),
                Err(e) => Err(anyhow::anyhow!("Inference error: {}", e)),
            }
        }
        .await;

        self.active_jobs.fetch_sub(1, Ordering::Relaxed);
        self.completed_jobs.fetch_add(1, Ordering::Relaxed);
        drop(permit);

        result
    }

    /// Execute multiple inference jobs in parallel across different models
    pub async fn parallel_infer(
        &self,
        jobs: Vec<(String, String, usize)>, // (model_name, prompt, max_tokens)
    ) -> Vec<anyhow::Result<String>> {
        let futures = jobs.into_iter().map(|(model, prompt, tokens)| {
            let model_owned = model;
            let prompt_owned = prompt;
            let tokens_owned = tokens;
            async move {
                self.infer_with_model(&model_owned, &prompt_owned, tokens_owned)
                    .await
            }
        });
        futures_util::future::join_all(futures).await
    }

    /// Get pool statistics
    pub fn stats(&self) -> ComputePoolStats {
        ComputePoolStats {
            worker_count: self.worker_count,
            active_jobs: self.active_jobs(),
            completed_jobs: self.completed_jobs(),
        }
    }
}

/// Statistics for the compute pool
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComputePoolStats {
    pub worker_count: usize,
    pub active_jobs: usize,
    pub completed_jobs: usize,
}

/// System-wide context
#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct SystemContext {
    /// Known projects
    #[serde(with = "crate::mem8::path_serde::map")]
    pub projects: HashMap<PathBuf, ProjectInfo>,
    /// Directory consciousnesses
    #[serde(with = "crate::mem8::path_serde::map")]
    pub consciousnesses: HashMap<PathBuf, DirectoryInfo>,
    /// Last scan timestamp
    pub last_scan: Option<std::time::SystemTime>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectInfo {
    pub path: String,
    pub name: String,
    pub project_type: String,
    pub key_files: Vec<String>,
    pub essence: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
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

    // Load or generate auth token (required only when externally bound)
    load_or_create_token()?;
    println!("  🔑 Auth token: {}", token_path().display());
    println!(
        "  🌐 Dashboard:  http://localhost:{}/ (automagic on localhost, no token needed)",
        config.port
    );
    println!(
        "  🔌 MCP HTTP:   http://localhost:{}/mcp (Streamable HTTP) + /mcp/sse (SSE)",
        config.port
    );
    println!(
        "  🤖 Supervisor: `st up` ensures it runs; `st token` shows the token for external use"
    );

    // Create shutdown channel
    let (shutdown_tx, shutdown_rx) = oneshot::channel::<()>();

    // Initialize LLM proxy with available providers
    let llm_proxy = Arc::new(LlmProxy::default());
    let provider_count = llm_proxy.providers.len();

    // Initialize proxy memory for conversation persistence
    let memory_directory = crate::mem8::record_store::memory_dir()?;
    let restore_paths = config.watch_paths.clone();
    let (proxy_memory, directory_memory) = tokio::task::spawn_blocking(move || -> Result<_> {
        let proxy_memory =
            ProxyMemory::open(&memory_directory).context("Cannot restore conversation memory")?;
        let mut directory_memory =
            ContextMemory::open(&memory_directory).context("Cannot restore directory memory")?;
        for path in restore_paths {
            if path.is_dir() {
                directory_memory.watch_initial(&path)?;
            }
        }
        Ok((proxy_memory, directory_memory))
    })
    .await??;
    let context = directory_memory.context();
    let recall_view = directory_memory.recall_view();
    println!(
        "  🧠 MEM8: restored {} indexed files",
        directory_memory.indexed_files()
    );
    let proxy_memory = Arc::new(std::sync::Mutex::new(proxy_memory));
    let directory_memory = Arc::new(std::sync::Mutex::new(directory_memory));

    // Initialize model cache for sovereign models
    let model_cache = Arc::new(crate::mcp::combinatorics_llm::LlamaModelCache::new(8));

    // Preload sovereign models from config
    let st_config = crate::config::StConfig::load()?;
    for (name, path) in &st_config.models.aliases {
        if path.starts_with("models/") {
            println!("  🔮 Preloading sovereign model: {}", name);
            if let Err(e) = model_cache.preload_named_model(name, path, u32::MAX) {
                eprintln!("  ⚠️ Failed to preload {}: {}", name, e);
            }
        }
    }

    // Initialize parallel compute pool for sovereign model inference
    let compute_pool = Arc::new(crate::daemon::ComputePool::new(
        num_cpus::get().max(4),
        Arc::clone(&model_cache),
    ));
    println!(
        "  ⚡ Compute pool: {} workers ready",
        compute_pool.worker_count()
    );

    // Initialize collaboration hub
    let collab_hub = create_hub();

    // Initialize session store for auth
    let sessions = create_session_store();

    // Check for GitHub OAuth config
    let github_oauth = GitHubOAuthConfig::from_env();
    if github_oauth.is_some() {
        println!("  🔐 GitHub OAuth: configured");
    }

    // Initialize Hot Watcher for real-time directory intelligence
    println!("  🔥 Hot Watcher: ready (MEM8 waves)");

    let security_config = crate::config::StConfig::load()?.security;
    let scan_memory = Arc::new(std::sync::Mutex::new(
        crate::magiscanner::memory::ScanMemory::open(&security_config)?,
    ));
    let state = Arc::new(RwLock::new(DaemonState {
        context,
        credits: CreditTracker::default(),
        config: config.clone(),
        shutdown_tx: Some(shutdown_tx),
        llm_proxy,
        proxy_memory,
        collab_hub,
        sessions,
        github_oauth,
        directory_memory,
        recall_view,
        scan_memory,
        compute_pool,
    }));

    println!("  🤖 LLM Providers: {} available", provider_count);

    // Create MCP context for HTTP MCP endpoints
    let mcp_context = create_mcp_context();

    let app = Router::new()
        // Welcome page
        .route("/", get(welcome_page))
        // Health & Info
        .route("/health", get(health))
        .route("/info", get(info))
        .route("/settings", get(get_settings))
        .route("/settings", post(update_settings))
        // Context endpoints
        .route("/context", get(get_context))
        .route("/context/projects", get(get_projects))
        .route("/context/query", post(query_context))
        .route("/context/recall", post(recall_context))
        .route("/context/remember", post(remember_context))
        .route("/context/files", get(list_files))
        // Credit endpoints
        .route("/credits", get(get_credits))
        .route("/credits/record", post(record_credit))
        // Legacy tool interface (kept for compatibility)
        .route("/tools", get(list_tools))
        .route("/tools/call", post(call_tool))
        // LLM Proxy - OpenAI-compatible chat completions
        .route("/v1/chat/completions", post(chat_completions))
        .route("/v1/models", get(list_models))
        // Collaboration Station 🛁
        .route("/collab/presence", get(collab_presence))
        .route("/collab/ws", get(collab_websocket_handler))
        // WebSocket for real-time
        .route("/ws", get(websocket_handler))
        // Daemon control
        .route("/shutdown", post(shutdown_handler))
        .route("/ping", get(ping))
        // CLI thin-client endpoints - all the meat lives here!
        .route("/cli/scan", post(crate::daemon_cli::cli_scan_handler))
        .route("/cli/stream", post(crate::daemon_cli::cli_stream_handler))
        // Hot Watcher - Real-time directory intelligence (MEM8 waves)
        .route("/watch", post(watch_directory))
        .route("/watch", axum::routing::delete(unwatch_directory))
        .route("/watch/status", get(watch_status))
        .route("/watch/hot", get(watch_hot_directories))
        // Security sentinel (MagiSCanner capabilities)
        .route(
            "/security/scan",
            post(crate::magiscanner::http::security_scan_handler),
        )
        .route(
            "/security/hash/:sha256",
            get(crate::magiscanner::http::hash_lookup_handler),
        )
        .route(
            "/security/certs/audit",
            get(crate::magiscanner::http::cert_audit_handler),
        )
        .route(
            "/security/certs/scan",
            post(crate::magiscanner::http::certificate_scan_handler),
        )
        .route(
            "/security/history",
            get(crate::magiscanner::http::scan_history_handler),
        )
        // MemNet bridge (MCP ↔ mesh): file-backed MemnetNodeState + Native Bitstream + Dev LLM Guide
        .route("/memnet/peers", get(memnet_peers_handler))
        .route("/memnet/ask", post(memnet_ask_handler))
        .route("/memnet/stream", post(memnet_stream_handler))
        .route("/memnet/llm-guide", get(memnet_llm_guide_handler))
        .route("/memnet/teach", post(memnet_teach_handler))
        .route("/memnet/share", post(memnet_share_handler))
        // i1.is MEM8 Knowledge Nexus, Public/Private Wave Policy & Cryptographic Chain of Title
        .route("/i1/nexus", get(i1_nexus_status_handler))
        .route("/i1/nexus/distill", post(i1_nexus_distill_handler))
        .route("/i1/nexus/settings", post(i1_nexus_settings_handler))
        // MEM8 Life Personas (.m8 Soul Trainer & Local Ternary Micro-Model)
        .route("/api/personas", get(personas_list_handler))
        .route("/api/personas/train", post(personas_train_handler))
        .route(
            "/api/personas/train-specialties",
            post(personas_train_specialties_handler),
        )
        .with_state(Arc::clone(&state))
        // Add MCP before the auth layer so its routes share the same policy.
        .nest_service("/mcp", mcp_router(mcp_context))
        // Bearer token auth only when externally bound; localhost is trusted
        // (see AuthState). /health stays public inside the middleware.
        .layer(middleware::from_fn_with_state(
            AuthState::new(load_all_tokens(), config.allow_external),
            auth_middleware,
        ));

    let bind_addr: [u8; 4] = if config.allow_external {
        [0, 0, 0, 0]
    } else {
        [127, 0, 0, 1]
    };
    let addr = SocketAddr::from((bind_addr, config.port));
    println!("Smart Tree Daemon listening on http://{}", addr);
    if !config.allow_external {
        println!("  🔒 Bound to localhost only: MCP + dashboard need no token (set allow_external=true in ~/.st/config.toml to require Bearer token)");
    }
    println!(
        "  - Dashboard:    http://localhost:{}/ (open this!)",
        config.port
    );
    println!("  - CLI Scan:     /cli/scan (thin-client endpoint!)");
    println!("  - CLI Stream:   /cli/stream (SSE streaming)");
    println!(
        "  - MCP HTTP:     /mcp (Streamable HTTP) + /mcp/sse (SSE) — The Custodian watching! 🧹"
    );
    println!(
        "  - Token:        {} (needed only when externally bound; `st token` prints it)",
        token_path().display()
    );
    println!("  - Context API:  /context");
    println!("  - Credits:      /credits");
    println!("  - Tools:        /tools (legacy)");
    println!("  - LLM Proxy:    /v1/chat/completions (OpenAI-compatible!)");
    println!("  - Models:       /v1/models");
    println!("  - Collab:       /collab/ws (Hot Tub Mode!) 🛁");
    println!("  - Hot Watcher:  /watch (MEM8 real-time intelligence) 🔥");
    println!("  - Security:     /security/* (integrity scan, hash memory, cert audit) 🛡️");
    println!(
        "  - MemNet UDP:   0.0.0.0:{} (NodePing/NodePong & Wave Q&A Offload) 🧭",
        crate::memnet::MEMNET_FS_PORT
    );
    println!("  - WebSocket:    /ws");
    println!("  - Shutdown:     POST /shutdown");

    let listener = tokio::net::TcpListener::bind(addr).await?;

    // Spawn background MemNet UDP responder on :8423 (best-effort if port available)
    std::thread::spawn(|| {
        let mut mesh = crate::memnet::MemnetNodeState::load_or_default();
        let bind_addr = format!("0.0.0.0:{}", crate::memnet::MEMNET_FS_PORT);
        let _ = mesh.serve_udp_responder(&bind_addr, None);
    });

    let (maintenance_stop, mut maintenance_rx) = oneshot::channel::<()>();
    let maintenance_state = Arc::clone(&state);
    let maintenance = tokio::spawn(async move {
        let mut ticks = 0u64;
        loop {
            tokio::select! {
                _ = &mut maintenance_rx => break,
                _ = tokio::time::sleep(std::time::Duration::from_secs(1)) => {}
            }
            // Reconcile once after startup, then periodically. Existing context
            // remains available while changed metadata is checked off-thread.
            let reconcile = ticks.is_multiple_of(300);
            let checkpoint = ticks.is_multiple_of(30);
            if reconcile {
                let _ = tokio::task::spawn_blocking(|| {
                    let _ = crate::completions::maintain_service_completions(false);
                })
                .await;
            }
            if let Err(error) = update_directory_memory(&maintenance_state, move |memory| {
                memory.maintain(reconcile, checkpoint)
            })
            .await
            {
                tracing::error!(%error, "Directory memory maintenance failed; saved context retained");
            }
            ticks = ticks.wrapping_add(1);
        }
    });

    // Serve with graceful shutdown support
    let served = axum::serve(listener, app)
        .with_graceful_shutdown(async {
            shutdown_rx.await.ok();
            println!("\n🌳 Smart Tree Daemon shutting down gracefully...");
        })
        .await;
    let _ = maintenance_stop.send(());
    maintenance
        .await
        .context("Directory memory worker failed")?;
    update_directory_memory(&state, |memory| memory.maintain(false, true)).await?;
    served?;

    println!("🌳 Smart Tree Daemon stopped.");
    Ok(())
}

/// Serialize storage mutations on a blocking worker, publishing a context cache
/// only after the operation, without holding the daemon lock during disk I/O.
async fn update_directory_memory<T: Send + 'static>(
    state: &Arc<RwLock<DaemonState>>,
    operation: impl FnOnce(&mut ContextMemory) -> Result<T> + Send + 'static,
) -> Result<T> {
    let memory = state.read().await.directory_memory.clone();
    let state = Arc::clone(state);
    tokio::task::spawn_blocking(move || {
        let mut memory = memory
            .lock()
            .map_err(|_| anyhow::anyhow!("Directory memory lock poisoned"))?;
        let result = operation(&mut memory);
        let mut state = state.blocking_write();
        state.context = memory.context();
        state.recall_view = memory.recall_view();
        result
    })
    .await
    .context("Directory memory worker failed")?
}

async fn recall_context(
    State(state): State<Arc<RwLock<DaemonState>>>,
    Json(request): Json<RecallRequest>,
) -> Result<Json<Vec<context_memory::RecallHit>>, (StatusCode, String)> {
    if request.query.len() > 8192
        || request
            .after
            .zip(request.before)
            .is_some_and(|(after, before)| after >= before)
    {
        return Err((
            StatusCode::BAD_REQUEST,
            "Invalid query or time range".into(),
        ));
    }
    let view = state.read().await.recall_view.clone();
    tokio::task::spawn_blocking(move || Json(view.recall(&request, Utc::now())))
        .await
        .map_err(|error| (StatusCode::INTERNAL_SERVER_ERROR, error.to_string()))
}

async fn remember_context(
    State(state): State<Arc<RwLock<DaemonState>>>,
    Json(association): Json<Association>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let id = update_directory_memory(&state, move |memory| memory.remember(association))
        .await
        .map_err(|error| (StatusCode::INTERNAL_SERVER_ERROR, error.to_string()))?;
    Ok(Json(serde_json::json!({ "id": id, "stored": true })))
}

// --- MemNet bridge (`/memnet/*`): MCP ↔ mesh ---
// File-backed `MemnetNodeState` (load → act → save) so the same wave cache
// backs the UDP responder, the MCP `memnet` tool, and these HTTP routes.

#[derive(Deserialize, Serialize)]
struct MemnetAskRequest {
    question: String,
    #[serde(default)]
    model: Option<String>,
    #[serde(default)]
    min_resonance_pm: Option<u32>,
    #[serde(default)]
    lan: bool,
}

#[derive(Deserialize)]
struct MemnetStreamRequest {
    #[serde(default)]
    question: Option<String>,
    #[serde(default)]
    model: Option<String>,
    #[serde(default)]
    messages: Option<Vec<crate::proxy::openai_compat::OpenAiMessage>>,
    /// `"raw"` returns raw binary bitstream (`[64B LitanyWireFrame][79B WaveFrame79][26B SPQT Base-243][MNPK Packet]`);
    /// `"framed"` (default unless `Accept: application/octet-stream`) returns JSON + hex wire frames.
    #[serde(default)]
    bitstream: Option<String>,
}

#[derive(Deserialize, Serialize)]
struct MemnetTeachRequest {
    question: String,
    answer: String,
}

#[derive(Deserialize, Serialize)]
struct MemnetShareRequest {
    path: String,
    #[serde(default)]
    quota_mb: Option<u64>,
}

async fn memnet_peers_handler() -> Json<serde_json::Value> {
    let state = crate::memnet::MemnetNodeState::load_or_default();
    Json(serde_json::json!({
        "node": state.node_name,
        "address": state.local_address,
        "tier": format!("{:?}", state.tier),
        "wave_qa_entries": state.wave_qa_cache.len(),
        "routes_total": state.routes.len(),
        "routes_up": state.routes.iter().filter(|r| r.status == crate::memnet::RouteStatus::Up).count(),
        "port": crate::memnet::MEMNET_FS_PORT,
        "composite_rtc_tau_ms": state.composite_rtc_tau_ms(0xC5),
        "shares": state.shared_folders.len(),
        "mcp_tool": "memnet {operation:'peers'}",
    }))
}

async fn memnet_ask_handler(
    Json(req): Json<MemnetAskRequest>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let min_pm = req.min_resonance_pm.unwrap_or(500).clamp(1, 1000);
    let question = req.question;
    let model_hint = req
        .model
        .unwrap_or_else(|| "burn/mem8-ternary-micro".to_string());
    let lan = req.lan;
    let result = tokio::task::spawn_blocking(move || {
        let mut state = crate::memnet::MemnetNodeState::load_or_default();
        if let Some((resonance_pm, pkt)) = state.query_wave_qa(&question, min_pm) {
            let _ = state.save();
            return serde_json::json!({
                "source": "local",
                "resonance_pm": resonance_pm,
                "packet_id": pkt.packet_id,
                "answer": pkt.answer,
                "origin_name": pkt.origin_name,
                "hit_count": pkt.hit_count,
                "litany_wire_64b_hex": hex::encode(pkt.litany_wire.to_bytes_64()),
                "wave_frame_79b_hex": hex::encode(pkt.wave_frame.to_bytes_79()),
                "wave_b3": pkt.wave_b3,
            });
        }
        if lan {
            if let Some((resonance_pm, pkt)) = state.query_lan_wave_qa(&question, min_pm) {
                let _ = state.save();
                return serde_json::json!({
                    "source": "lan",
                    "resonance_pm": resonance_pm,
                    "packet_id": pkt.packet_id,
                    "answer": pkt.answer,
                    "origin_name": pkt.origin_name,
                    "hit_count": pkt.hit_count,
                    "litany_wire_64b_hex": hex::encode(pkt.litany_wire.to_bytes_64()),
                    "wave_frame_79b_hex": hex::encode(pkt.wave_frame.to_bytes_79()),
                    "wave_b3": pkt.wave_b3,
                });
            }
        }
        // Fallback: synthesize via 2-Pass Persona Soul & cache into MemNet WaveQA
        if let Some((slug, pass2, full_reply, recalled_len)) =
            run_two_pass_persona_qa(&question, &model_hint)
        {
            let answer_text = pass2.clone().unwrap_or_else(|| full_reply.clone());
            let pkt = state.learn_wave_qa(&question, &answer_text);
            let _ = state.save();
            return serde_json::json!({
                "source": "persona-pass2",
                "persona": format!("persona:{slug}"),
                "resonance_pm": 950,
                "packet_id": pkt.packet_id,
                "pass2_contextual_answer": pass2,
                "pass1_core_memories_count": recalled_len,
                "answer": full_reply,
                "origin_name": pkt.origin_name,
                "litany_wire_64b_hex": hex::encode(pkt.litany_wire.to_bytes_64()),
                "wave_frame_79b_hex": hex::encode(pkt.wave_frame.to_bytes_79()),
                "wave_b3": pkt.wave_b3,
                "stream_endpoint": "/memnet/stream",
            });
        }
        serde_json::json!({ "source": "none", "resonance_pm": 0, "question": question })
    })
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    Ok(Json(result))
}

/// Run Pass-1 (`recall_episodes`) + Pass-2 (`synthesize_contextual_answer`) on a specified or auto-routed `.m8` Persona Soul.
fn run_two_pass_persona_qa(
    question: &str,
    model_hint: &str,
) -> Option<(String, Option<String>, String, usize)> {
    use crate::mem8::persona::LifePersona;
    let clean = model_hint
        .strip_prefix("persona:")
        .or_else(|| model_hint.strip_prefix("burn/"))
        .or_else(|| model_hint.strip_prefix("candle/"))
        .or_else(|| model_hint.strip_prefix("local/"))
        .unwrap_or(model_hint);

    let is_router = clean.is_empty()
        || clean.contains("ternary")
        || clean == "mem8-wave-micro"
        || clean == "default";

    if !is_router {
        if let Ok(persona) = LifePersona::load_cached(clean) {
            let recalled = persona.recall_episodes(question, 8);
            let pass2 = persona.synthesize_contextual_answer(question, &recalled);
            let full = persona.respond_locally(question);
            return Some((persona.slug.clone(), pass2, full, recalled.len()));
        }
    }

    // Auto-route across all trained `.m8` Persona Souls using SPQT Base-243 ternary signature
    let (q_trits, _) = crate::proxy::burn::compute_ternary_signature(question);
    let summaries = LifePersona::list_all().ok()?;
    let mut best_score = f32::MIN;
    let mut best_persona = None;
    let lower_q = question.to_ascii_lowercase();

    for summary in &summaries {
        if let Ok(p) = LifePersona::load_cached(&summary.slug) {
            let top = p.recall_episodes(question, 1);
            if let Some(ep) = top.first() {
                let (ep_trits, _) = crate::proxy::burn::compute_ternary_signature(&ep.passage);
                let dot: i32 = q_trits
                    .iter()
                    .zip(ep_trits.iter())
                    .map(|(&a, &b)| (a as i32) * (b as i32))
                    .sum();
                let mut score = dot as f32 * 0.25;
                if lower_q.contains(&p.name.to_ascii_lowercase()) || lower_q.contains(&p.slug) {
                    score += 18.0;
                }
                if let Some(ref seed) = p.ancestral_seed {
                    for alias in &seed.aliases {
                        if lower_q.contains(&alias.to_ascii_lowercase()) {
                            score += 16.0;
                        }
                    }
                    for place in &seed.places {
                        if lower_q.contains(&place.to_ascii_lowercase()) {
                            score += 12.0;
                        }
                    }
                    for comp in &seed.companions {
                        if lower_q.contains(&comp.to_ascii_lowercase()) {
                            score += 12.0;
                        }
                    }
                }
                if score > best_score {
                    best_score = score;
                    best_persona = Some(p);
                }
            }
        }
    }

    let persona = best_persona?;
    let recalled = persona.recall_episodes(question, 8);
    let pass2 = persona.synthesize_contextual_answer(question, &recalled);
    let full = persona.respond_locally(question);
    Some((persona.slug.clone(), pass2, full, recalled.len()))
}

/// ⚡ Native MEMNET Bitstream Endpoint (`POST /memnet/stream`)
///
/// Executes the 2-Pass SPQT Base-243 Ternary Persona Q&A (Pass 1 core memories + Pass 2 contextual answer)
/// and emits a native `MEMNET` bitstream (`64B@0x62 LitanyWireFrame` + `79B WaveFrame79` + `26B SPQT Base-243`
/// packed query/answer trits + `MNPK WaveQaResponse`).
///
/// - When `Accept: application/octet-stream` or `"bitstream": "raw"` is passed, returns raw binary wire bytes.
/// - Otherwise returns a structured JSON envelope with both the Pass-2 contextual answer and the hex-encoded bitstream frames.
async fn memnet_stream_handler(
    headers: axum::http::HeaderMap,
    Json(req): Json<MemnetStreamRequest>,
) -> axum::response::Response {
    let question = req
        .question
        .or_else(|| {
            req.messages.as_ref().and_then(|msgs| {
                msgs.iter()
                    .rev()
                    .find(|m| m.role == "user")
                    .map(|m| m.content.clone())
            })
        })
        .unwrap_or_default();

    if question.trim().is_empty() {
        return (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({
                "error": "Provide 'question' or OpenAI-compatible 'messages' array"
            })),
        )
            .into_response();
    }

    let model_hint = req
        .model
        .unwrap_or_else(|| "burn/mem8-ternary-micro".to_string());
    let wants_raw_bytes = req
        .bitstream
        .as_deref()
        .is_some_and(|b| b.eq_ignore_ascii_case("raw"))
        || headers
            .get(axum::http::header::ACCEPT)
            .and_then(|v| v.to_str().ok())
            .is_some_and(|a| a.contains("application/octet-stream"));

    let work = tokio::task::spawn_blocking(move || {
        let (slug, pass2_answer, full_response, recalled_count) =
            run_two_pass_persona_qa(&question, &model_hint).unwrap_or_else(|| {
                (
                    "mem8-ternary-micro".to_string(),
                    None,
                    format!("SPQT Base-243 Ternary response for: {question}"),
                    0,
                )
            });

        let wire_answer = pass2_answer
            .clone()
            .unwrap_or_else(|| full_response.clone());
        let mut state = crate::memnet::MemnetNodeState::load_or_default();
        let pkt = state.learn_wave_qa(&question, &wire_answer);
        let _ = state.save();

        let litany_64 = pkt.litany_wire.to_bytes_64();
        let wave_79 = pkt.wave_frame.to_bytes_79();
        let (_, q_spqt_13) = crate::proxy::burn::compute_ternary_signature(&question);
        let (_, a_spqt_13) = crate::proxy::burn::compute_ternary_signature(&wire_answer);
        let mnpk_raw = crate::memnet::MemnetPayload::WaveQaResponse(pkt.clone())
            .into_raw_packet(vec![state.local_address])
            .map(|p| p.to_wire_bytes())
            .unwrap_or_default();

        // Contiguous raw wire bitstream: [64B LitanyWireFrame][79B WaveFrame79][13B Query SPQT][13B Answer SPQT][MNPK Packet]
        let mut raw_stream = Vec::with_capacity(64 + 79 + 13 + 13 + mnpk_raw.len());
        raw_stream.extend_from_slice(&litany_64);
        raw_stream.extend_from_slice(&wave_79);
        raw_stream.extend_from_slice(&q_spqt_13);
        raw_stream.extend_from_slice(&a_spqt_13);
        raw_stream.extend_from_slice(&mnpk_raw);

        (
            slug,
            question,
            pass2_answer,
            full_response,
            recalled_count,
            pkt,
            litany_64,
            wave_79,
            q_spqt_13,
            a_spqt_13,
            raw_stream,
        )
    })
    .await;

    let (
        slug,
        question,
        pass2_answer,
        full_response,
        recalled_count,
        pkt,
        litany_64,
        wave_79,
        q_spqt_13,
        a_spqt_13,
        raw_stream,
    ) = match work {
        Ok(res) => res,
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({ "error": e.to_string() })),
            )
                .into_response()
        }
    };

    let litany_hex = hex::encode(litany_64);
    let wave79_hex = hex::encode(wave_79);
    let q_spqt_hex = hex::encode(q_spqt_13);
    let a_spqt_hex = hex::encode(a_spqt_13);

    if wants_raw_bytes {
        return (
            StatusCode::OK,
            [
                (
                    axum::http::header::CONTENT_TYPE,
                    "application/octet-stream".to_string(),
                ),
                (
                    axum::http::HeaderName::from_static("x-memnet-protocol"),
                    "MEMNET-64B-0x62-Wave79-SPQT-Bitstream".to_string(),
                ),
                (
                    axum::http::HeaderName::from_static("x-memnet-persona"),
                    format!("persona:{slug}"),
                ),
                (
                    axum::http::HeaderName::from_static("x-memnet-packet-id"),
                    pkt.packet_id.to_string(),
                ),
                (
                    axum::http::HeaderName::from_static("x-memnet-wave-b3"),
                    pkt.wave_b3.clone(),
                ),
                (
                    axum::http::HeaderName::from_static("x-memnet-litany-64b-hex"),
                    litany_hex,
                ),
                (
                    axum::http::HeaderName::from_static("x-memnet-wave79-hex"),
                    wave79_hex,
                ),
                (
                    axum::http::HeaderName::from_static("x-memnet-spqt-base243-hex"),
                    format!("{q_spqt_hex}{a_spqt_hex}"),
                ),
            ],
            raw_stream,
        )
            .into_response();
    }

    (
        StatusCode::OK,
        [
            (
                axum::http::HeaderName::from_static("x-smarttree-protocol"),
                "MEMNET-64B-0x62-Wave79-SPQT-Bitstream",
            ),
            (
                axum::http::HeaderName::from_static("x-smarttree-preferred-endpoint"),
                "/memnet/stream",
            ),
            (
                axum::http::HeaderName::from_static("x-smarttree-llm-guide"),
                "/memnet/llm-guide",
            ),
        ],
        Json(serde_json::json!({
            "protocol": "MEMNET-64B-0x62-Wave79-SPQT-Bitstream",
            "llm_implementation_guide": "/memnet/llm-guide",
            "i1_nexus_endpoint": "/i1/nexus",
            "model": format!("persona:{slug}"),
            "question": question,
            "pass2_contextual_answer": pass2_answer,
            "pass1_core_memories_count": recalled_count,
            "answer": full_response,
            "bitstream": {
                "packet_id": pkt.packet_id,
                "wave_b3": pkt.wave_b3,
                "litany_wire_64b_hex": litany_hex,
                "wave_frame_79b_hex": wave79_hex,
                "query_spqt_base243_13b_hex": q_spqt_hex,
                "answer_spqt_base243_13b_hex": a_spqt_hex,
                "raw_stream_bytes_len": raw_stream.len(),
                "layout": "[64B@0x62 LitanyWireFrame][79B WaveFrame79][13B Query SPQT Base-243][13B Answer SPQT Base-243][MNPK WaveQaResponse]",
                "raw_mode_hint": "Pass header 'Accept: application/octet-stream' or body '{\"bitstream\":\"raw\"}' for raw binary bitstream bytes",
                "llm_guide_hint": "GET /memnet/llm-guide for complete Rust/Python/TypeScript bitstream & i1.is Chain-of-Title implementation instructions"
            }
        })),
    )
        .into_response()
}

/// 🤖 Developer LLM Implementation Guide (`GET /memnet/llm-guide`)
///
/// Instructs any development LLM (Claude, GPT, Gemini, Cursor, Antigravity, OpenCode)
/// how to implement the `MEMNET` Bitstream client/server and `i1.is` Chain-of-Title Nibble Graph.
async fn memnet_llm_guide_handler(headers: axum::http::HeaderMap) -> axum::response::Response {
    let md = crate::mem8::i1_nexus::memnet_llm_implementation_guide();
    let wants_json = headers
        .get(axum::http::header::ACCEPT)
        .and_then(|v| v.to_str().ok())
        .is_some_and(|a| a.contains("application/json"));

    if wants_json {
        return Json(serde_json::json!({
            "protocol": "MEMNET-64B-0x62-Wave79-SPQT-Bitstream",
            "endpoints": {
                "memnet_stream": "POST /memnet/stream",
                "openai_compat": "POST /v1/chat/completions",
                "i1_nexus_status": "GET /i1/nexus",
                "i1_nexus_distill": "POST /i1/nexus/distill",
                "i1_nexus_settings": "POST /i1/nexus/settings",
            },
            "markdown_guide": md,
        }))
        .into_response();
    }

    (
        StatusCode::OK,
        [(
            axum::http::header::CONTENT_TYPE,
            "text/markdown; charset=utf-8",
        )],
        md,
    )
        .into_response()
}

#[derive(Deserialize, Default)]
struct I1NexusDistillRequest {
    #[serde(default)]
    persona: Option<String>,
    #[serde(default)]
    path: Option<String>,
    #[serde(default)]
    max_nibbles: Option<usize>,
}

#[derive(Deserialize)]
struct I1NexusSettingsRequest {
    #[serde(default)]
    wave_visibility: Option<String>,
    #[serde(default)]
    subscription_tier: Option<String>,
    #[serde(default)]
    chain_of_title_author: Option<String>,
    #[serde(default)]
    suggest_public_waves_in_chat: Option<bool>,
}

async fn i1_nexus_status_handler() -> Json<serde_json::Value> {
    let result = tokio::task::spawn_blocking(|| {
        let cfg = crate::config::StConfig::load()
            .map(|c| c.i1_nexus)
            .unwrap_or_default();
        let cwd = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
        let repo_report = crate::mem8::i1_nexus::detect_repo_public_status(&cwd, &cfg);
        let store = crate::mem8::i1_nexus::I1NexusStore::load_or_default();
        let total_relations: usize = store.nibbles.iter().map(|n| n.relations.len()).sum();

        serde_json::json!({
            "hub": "https://i1.is",
            "config": cfg,
            "repository_visibility": repo_report,
            "sustainability_model": {
                "public_commons": "Free for public repositories & open documentation — AI training & recall permitted when cryptographic Chain of Title is preserved.",
                "private_sovereign": format!(
                    "${:.2}/month small subscription — keeps the lights on at i1.is and funds the commons while keeping private repos & diaries 100% private.",
                    (cfg.private_monthly_fee_cents as f32) / 100.0
                ),
            },
            "stats": {
                "chains_of_title_count": store.chains_of_title.len(),
                "nibbles_count": store.nibbles.len(),
                "nibble_relations_count": total_relations,
                "updated_at_utc": store.updated_at_utc,
            },
            "chains_of_title": store.chains_of_title.values().take(25).collect::<Vec<_>>(),
            "nibbles": store.nibbles.iter().take(40).collect::<Vec<_>>(),
        })
    })
    .await
    .unwrap_or_else(|e| serde_json::json!({ "error": e.to_string() }));

    Json(result)
}

async fn i1_nexus_distill_handler(
    Json(req): Json<I1NexusDistillRequest>,
) -> (StatusCode, Json<serde_json::Value>) {
    let result = tokio::task::spawn_blocking(move || -> Result<serde_json::Value> {
        let cfg = crate::config::StConfig::load()
            .map(|c| c.i1_nexus)
            .unwrap_or_default();
        let mut store = crate::mem8::i1_nexus::I1NexusStore::load_or_default();
        let max_nibbles = req.max_nibbles.unwrap_or(24);

        let mut distilled_personas = Vec::new();
        let mut last_report = None;

        if let Some(ref slug_raw) = req.persona {
            let slug = slug_raw.strip_prefix("persona:").unwrap_or(slug_raw);
            let persona = crate::mem8::persona::LifePersona::load_cached(slug)?;
            let rep = store.distill_persona_to_nibbles(&persona, &cfg, max_nibbles);
            distilled_personas.push(persona.slug.clone());
            last_report = Some(rep);
        } else {
            // Distill the 3 workspace specialists (readme-expert, history-documenter, code-analyst) if available, or all personas
            let summaries = crate::mem8::persona::LifePersona::list_all().unwrap_or_default();
            for summary in summaries.iter().take(4) {
                if let Ok(persona) = crate::mem8::persona::LifePersona::load_cached(&summary.slug) {
                    let rep = store.distill_persona_to_nibbles(&persona, &cfg, max_nibbles);
                    distilled_personas.push(persona.slug.clone());
                    last_report = Some(rep);
                }
            }
        }

        if let Some(ref custom_path) = req.path {
            let expanded = shellexpand::tilde(custom_path.trim()).to_string();
            last_report = Some(crate::mem8::i1_nexus::detect_repo_public_status(
                std::path::Path::new(&expanded),
                &cfg,
            ));
        }

        let total_relations: usize = store.nibbles.iter().map(|n| n.relations.len()).sum();

        Ok(serde_json::json!({
            "status": "ok",
            "distilled_personas": distilled_personas,
            "repository_visibility": last_report,
            "chains_of_title_count": store.chains_of_title.len(),
            "nibbles_count": store.nibbles.len(),
            "nibble_relations_count": total_relations,
            "sample_nibbles": store.nibbles.iter().rev().take(12).collect::<Vec<_>>(),
        }))
    })
    .await;

    match result {
        Ok(Ok(val)) => (StatusCode::OK, Json(val)),
        Ok(Err(e)) => (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({ "error": e.to_string() })),
        ),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": e.to_string() })),
        ),
    }
}

async fn i1_nexus_settings_handler(
    Json(req): Json<I1NexusSettingsRequest>,
) -> (StatusCode, Json<serde_json::Value>) {
    let result = tokio::task::spawn_blocking(move || -> Result<serde_json::Value> {
        let mut config = crate::config::StConfig::load().unwrap_or_default();
        if let Some(ref vis) = req.wave_visibility {
            config.i1_nexus.wave_visibility =
                crate::mem8::i1_nexus::WaveVisibilityMode::from_str_loose(vis);
        }
        if let Some(ref tier) = req.subscription_tier {
            config.i1_nexus.subscription_tier = if tier.to_ascii_lowercase().contains("private") {
                crate::mem8::i1_nexus::NexusSubscriptionTier::PrivateSovereignMonthly
            } else {
                crate::mem8::i1_nexus::NexusSubscriptionTier::PublicCommonsFree
            };
        }
        if let Some(author) = req.chain_of_title_author {
            config.i1_nexus.chain_of_title_author = if author.trim().is_empty() {
                None
            } else {
                Some(author.trim().to_string())
            };
        }
        if let Some(suggest) = req.suggest_public_waves_in_chat {
            config.i1_nexus.suggest_public_waves_in_chat = suggest;
        }
        config.save()?;
        let cwd = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
        let report = crate::mem8::i1_nexus::detect_repo_public_status(&cwd, &config.i1_nexus);
        Ok(serde_json::json!({
            "status": "ok",
            "i1_nexus": config.i1_nexus,
            "repository_visibility": report,
        }))
    })
    .await;

    match result {
        Ok(Ok(val)) => (StatusCode::OK, Json(val)),
        Ok(Err(e)) => (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({ "error": e.to_string() })),
        ),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": e.to_string() })),
        ),
    }
}

async fn memnet_teach_handler(
    Json(req): Json<MemnetTeachRequest>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let question = req.question;
    let answer = req.answer;
    if question.trim().is_empty() || answer.trim().is_empty() {
        return Err((
            StatusCode::BAD_REQUEST,
            "question and answer required".into(),
        ));
    }
    let result = tokio::task::spawn_blocking(move || {
        let mut state = crate::memnet::MemnetNodeState::load_or_default();
        let pkt = state.learn_wave_qa(&question, &answer);
        let saved = state.save().is_ok();
        if let Ok(raw) = crate::memnet::MemnetPayload::WaveQaResponse(pkt.clone())
            .into_raw_packet(vec![state.local_address])
        {
            let _ = state.broadcast_lan_udp(&raw);
        }
        (saved, pkt.packet_id, pkt.answer, pkt.origin_name)
    })
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    Ok(Json(serde_json::json!({
        "stored": result.0,
        "packet_id": result.1,
        "answer": result.2,
        "origin_name": result.3,
    })))
}

async fn memnet_share_handler(
    Json(req): Json<MemnetShareRequest>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let path = req.path;
    let quota_mb = req.quota_mb.unwrap_or(4096);
    let result = tokio::task::spawn_blocking(move || {
        let mut state = crate::memnet::MemnetNodeState::load_or_default();
        let endpoint = format!("0.0.0.0:{}", crate::memnet::MEMNET_FS_PORT);
        let manifest = crate::memnet::SharedFolderManifest::from_directory(
            std::path::Path::new(&path),
            None,
            state.local_address,
            &state.node_name,
            &endpoint,
            quota_mb,
        )
        .map_err(|e| e.to_string())?;
        let name = manifest.share_name.clone();
        state.shared_folders.insert(name, manifest.clone());
        let _ = state.save();
        if let Ok(raw) = crate::memnet::MemnetPayload::FolderShareAnnounce(manifest.clone())
            .into_raw_packet(vec![0xF1])
        {
            let _ = state.broadcast_lan_udp(&raw);
        }
        Ok(manifest)
    })
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
    .map_err(|e| (StatusCode::BAD_REQUEST, e))?;
    Ok(Json(serde_json::json!({
        "share_id": result.share_id,
        "share_name": result.share_name,
        "total_files": result.total_files,
        "total_bytes": result.total_bytes,
        "endpoint": result.endpoint,
    })))
}

// API Handlers

async fn welcome_page() -> axum::response::Html<&'static str> {
    axum::response::Html(
        r#"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>Smart Tree Daemon</title>
    <style>
        * { margin: 0; padding: 0; box-sizing: border-box; }
        body {
            font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif;
            background: linear-gradient(135deg, #1a1a2e 0%, #16213e 100%);
            color: #e0e0e0; min-height: 100vh; padding: 2rem;
        }
        .header { text-align: center; margin-bottom: 2rem; }
        .header h1 { font-size: 2.5rem; }
        .header .emoji { font-size: 3rem; }
        .grid { display: grid; grid-template-columns: 1fr 1fr; gap: 1.5rem; max-width: 1200px; margin: 0 auto; }
        @media (max-width: 800px) { .grid { grid-template-columns: 1fr; } }
        .card {
            background: rgba(255,255,255,0.05); border-radius: 12px;
            padding: 1.5rem; border: 1px solid rgba(255,255,255,0.1);
        }
        .card h2 { font-size: 1.1rem; color: #4ecdc4; margin-bottom: 1rem; display: flex; align-items: center; gap: 0.5rem; }
        .endpoint { display: flex; justify-content: space-between; padding: 0.4rem 0; font-size: 0.9rem; }
        .endpoint a { color: #4ecdc4; text-decoration: none; }
        .endpoint a:hover { text-decoration: underline; }

        /* Chat */
        #chat-messages {
            height: 200px; overflow-y: auto; background: rgba(0,0,0,0.3);
            border-radius: 8px; padding: 1rem; margin-bottom: 1rem; font-size: 0.85rem;
        }
        .msg { margin-bottom: 0.5rem; padding: 0.5rem; border-radius: 6px; }
        .msg.user { background: rgba(78,205,196,0.2); text-align: right; }
        .msg.ai { background: rgba(243,156,18,0.2); }
        .msg .model { font-size: 0.7rem; color: #888; }
        .msg .score { font-size: 0.7rem; padding: 2px 6px; border-radius: 4px; margin-left: 0.5rem; }
        .score.safe { background: #27ae60; color: white; }
        .score.warn { background: #f39c12; color: black; }
        .score.danger { background: #e74c3c; color: white; }
        #chat-input { display: flex; gap: 0.5rem; }
        #chat-input input {
            flex: 1; padding: 0.75rem; border-radius: 8px; border: none;
            background: rgba(255,255,255,0.1); color: white;
        }
        #chat-input select { padding: 0.5rem; border-radius: 8px; background: #2a2a4a; color: white; border: none; }
        #chat-input button {
            padding: 0.75rem 1.5rem; border-radius: 8px; border: none;
            background: #4ecdc4; color: #1a1a2e; font-weight: bold; cursor: pointer;
        }

        /* Transparency Log */
        #transparency-log {
            height: 250px; overflow-y: auto; background: rgba(0,0,0,0.3);
            border-radius: 8px; padding: 0.5rem; font-family: monospace; font-size: 0.75rem;
        }
        .log-entry { padding: 0.4rem; border-bottom: 1px solid rgba(255,255,255,0.05); }
        .log-entry .time { color: #888; }
        .log-entry .type { padding: 2px 6px; border-radius: 3px; font-size: 0.65rem; }
        .log-entry .type.mcp { background: #9b59b6; }
        .log-entry .type.llm { background: #3498db; }
        .log-entry .type.tool { background: #e67e22; }
        .log-entry .content { color: #ccc; margin-top: 0.25rem; word-break: break-all; }

        /* Dashboard link */
        .dashboard-link {
            display: inline-block; margin-top: 1rem; padding: 0.75rem 2rem;
            background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
            color: white; text-decoration: none; border-radius: 8px; font-weight: bold;
        }
        .dashboard-link:hover { opacity: 0.9; }

        .custodian { text-align: center; color: #f39c12; margin-top: 1.5rem; }
    </style>
</head>
<body>
    <div class="header">
        <div class="emoji">🌳</div>
        <h1>Smart Tree Daemon</h1>
        <p style="color:#888">System AI Context Service</p>
        <p style="color:#4ecdc4;margin-top:1rem;">You're viewing the Smart Tree Dashboard</p>
        <p style="color:#888;font-size:0.85rem;">Dashboard: <strong>http://localhost:28428/</strong> · MCP (HTTP): <strong>http://localhost:28428/mcp</strong> (SSE: /mcp/sse) · MCP (stdio): <code>st --mcp</code></p>
        <p style="color:#666;font-size:0.75rem;">Localhost needs no token. If `st up` started this, you are done — `st token` shows the token for external use.</p>
    </div>

    <div class="grid">
        <!-- Chat Test -->
        <div class="card">
            <h2>💬 Test Chat (Local Ternary Model &amp; MEM8 Personas)</h2>
            <div id="chat-messages"></div>
            <div id="chat-input">
                <select id="model-select">
                    <optgroup label="🔺 Local Ternary &amp; MEM8 Souls (Zero API Key)" id="persona-optgroup">
                        <option value="burn/mem8-ternary-micro" selected>🔺 Local Ternary Model (SPQT Base-243 · Auto-Soul Router)</option>
                        <option value="burn/mem8-wave-micro">🕯️ Local MEM8 Wave Micro-Model</option>
                    </optgroup>
                    <optgroup label="Top Coding Models">
                        <option value="anthropic/claude-opus-4.5">Claude Opus 4.5</option>
                        <option value="anthropic/claude-sonnet-4.5">Claude Sonnet 4.5</option>
                        <option value="openai/gpt-5.2-codex">GPT-5.2 Codex</option>
                        <option value="google/gemini-3-pro-preview">Gemini 3 Pro</option>
                        <option value="deepseek/deepseek-v3.2">DeepSeek V3.2</option>
                        <option value="qwen/qwen3-coder">Qwen3 Coder 480B</option>
                    </optgroup>
                    <optgroup label="Fast &amp; Efficient">
                        <option value="anthropic/claude-haiku-4.5">Claude Haiku 4.5</option>
                        <option value="x-ai/grok-code-fast-1">Grok Code Fast</option>
                        <option value="google/gemini-3-flash-preview">Gemini 3 Flash</option>
                        <option value="moonshotai/kimi-k2.5">Kimi K2.5</option>
                    </optgroup>
                    <optgroup label="Free Tier">
                        <option value="google/gemini-2.0-flash-exp:free">Gemini 2.0 Flash (Free)</option>
                        <option value="z-ai/glm-4.5-air:free">GLM 4.5 Air (Free)</option>
                    </optgroup>
                </select>
                <input type="text" id="msg-input" placeholder="Ask the Local Ternary Model or a Persona Soul..." onkeypress="if(event.key==='Enter')sendChat()">
                <button onclick="sendChat()">Send</button>
            </div>
            <p style="font-size:0.7rem;color:#666;margin-top:0.5rem;">Default: <code>burn/mem8-ternary-micro</code> (100% offline SPQT Base-243) · ⚡ Recommended Bitstream: <code>POST /memnet/stream</code> · <a href="/memnet/llm-guide" target="_blank" style="color:#4ecdc4;">🤖 Dev LLM Bitstream Spec</a></p>
        </div>

        <!-- Life Personas Studio -->
        <div class="card">
            <h2>🧬 Life Personas &amp; Specialist Souls (250k+ Episodes)</h2>
            <p style="font-size:0.8rem;color:#888;margin-bottom:0.75rem">
                Train a tight <code>.m8</code> soul (up to 250,000+ memories) from diaries, drives, or repos
            </p>
            <div style="display:flex;flex-direction:column;gap:0.5rem;margin-bottom:0.75rem;">
                <div style="display:flex;gap:0.5rem;">
                    <input type="text" id="persona-name" placeholder="Name (e.g. Nate, Grandma, Code Analyst)" style="flex:1;padding:0.6rem;border-radius:6px;border:none;background:rgba(255,255,255,0.1);color:white;">
                    <select id="persona-specialty" style="padding:0.6rem;border-radius:6px;border:none;background:rgba(255,255,255,0.1);color:white;">
                        <option value="life-biographer">🧬 Life Biographer</option>
                        <option value="history-documenter">📜 History Documenter</option>
                        <option value="code-analyst">🔬 Code Analyst</option>
                        <option value="readme-expert">📖 README Expert</option>
                    </select>
                </div>
                <div style="display:flex;gap:0.5rem;">
                    <input type="text" id="persona-path" placeholder="Folder or file path (e.g. ~/source/standardgalactic/kitbash)" style="flex:1;padding:0.6rem;border-radius:6px;border:none;background:rgba(255,255,255,0.1);color:white;">
                    <button onclick="trainPersona()" style="padding:0.6rem 1rem;border-radius:6px;border:none;background:#4ecdc4;color:#1a1a2e;font-weight:bold;cursor:pointer;">Train .m8</button>
                </div>
                <button onclick="trainSpecialties()" style="padding:0.45rem 0.75rem;border-radius:6px;border:1px solid rgba(78,205,196,0.4);background:rgba(78,205,196,0.12);color:#4ecdc4;font-size:0.75rem;font-weight:bold;cursor:pointer;">
                    ⚡ Auto-Train 3 Specialists (📜 History · 🔬 Code · 📖 README) on Path
                </button>
            </div>
            <div id="personas-list" style="max-height:150px;overflow-y:auto;font-size:0.8rem;color:#ccc;">
                Loading trained personas...
            </div>
        </div>

        <!-- Transparency Log -->
        <div class="card">
            <h2>👁️ Transparency Mode</h2>
            <p style="font-size:0.8rem;color:#888;margin-bottom:0.5rem">All AI communications logged here</p>
            <div id="transparency-log">
                <div class="log-entry">
                    <span class="time">--:--:--</span>
                    <span class="type mcp">SYSTEM</span>
                    <div class="content">Transparency mode active. Watching all AI traffic...</div>
                </div>
            </div>
        </div>

        <!-- API Endpoints -->
        <div class="card">
            <h2>🔌 API Endpoints</h2>
            <div class="endpoint"><span>Health</span><a href="/health">/health</a></div>
            <div class="endpoint"><span>Info</span><a href="/info">/info</a></div>
            <div class="endpoint"><span>Context</span><a href="/context">/context</a></div>
            <div class="endpoint"><span>MCP Tools</span><a href="/mcp/tools/list">/mcp/tools/list</a></div>
            <div class="endpoint"><span>Models</span><a href="/v1/models">/v1/models</a></div>
            <div class="endpoint"><span style="color:#4ecdc4;font-weight:bold;">⚡ MEMNET Bitstream (Preferred)</span><span>/memnet/stream</span></div>
            <div class="endpoint"><span style="color:#f39c12;font-weight:bold;">🤖 Dev LLM Bitstream Guide</span><a href="/memnet/llm-guide" target="_blank">/memnet/llm-guide</a></div>
            <div class="endpoint"><span style="color:#2ecc71;font-weight:bold;">🌐 i1.is Nexus &amp; Chain of Title</span><a href="/i1/nexus" target="_blank">/i1/nexus</a></div>
            <div class="endpoint"><span>Chat API (OpenAI Compat)</span><span>/v1/chat/completions</span></div>
        </div>

        <!-- i1.is MEM8 Knowledge Nexus & Chain of Title -->
        <div class="card" style="grid-column: 1 / -1;">
            <h2>🌐 i1.is MEM8 Knowledge Nexus · Chain of Title &amp; Information Nibbles</h2>
            <p style="font-size:0.8rem;color:#888;margin-bottom:0.75rem">
                Distill docs, repos &amp; <code>.m8</code> souls into interconnected <strong>i1.is Knowledge Nibbles</strong> with cryptographic <strong>Chain of Title</strong>.
                <br><span style="color:#2ecc71;">Public Commons (Free):</span> Public repos are free for AIs to train &amp; recall when preserving Chain of Title ·
                <span style="color:#f39c12;">Private Sovereign ($5.00/mo):</span> Keeps the lights on at <code>i1.is</code> while keeping private repos &amp; diaries 100% private.
            </p>
            <div id="i1-repo-banner" style="padding:0.6rem 0.85rem;border-radius:6px;background:rgba(78,205,196,0.1);border:1px solid rgba(78,205,196,0.3);font-size:0.78rem;margin-bottom:0.75rem;">
                Inspecting repository visibility &amp; Chain of Title...
            </div>
            <div style="display:flex;gap:0.5rem;flex-wrap:wrap;margin-bottom:0.75rem;align-items:center;">
                <select id="i1-visibility-select" onchange="saveI1NexusVisibility()" style="padding:0.5rem;border-radius:6px;border:none;background:rgba(255,255,255,0.1);color:white;font-size:0.78rem;">
                    <option value="auto-detect">🧭 Auto-Detect Public Repo (Recommended)</option>
                    <option value="public-commons">🌐 Public Commons (Free · AI Training Allowed w/ Chain of Title)</option>
                    <option value="private-sovereign">🔒 Private Sovereign ($5.00/mo · Keeps the Lights On · No AI Training)</option>
                </select>
                <button onclick="distillI1Nexus()" style="padding:0.5rem 0.9rem;border-radius:6px;border:none;background:#4ecdc4;color:#1a1a2e;font-weight:bold;font-size:0.78rem;cursor:pointer;">
                    ⚡ Distill Souls into i1.is Nibbles &amp; Chain of Title
                </button>
                <a href="/memnet/llm-guide" target="_blank" style="padding:0.5rem 0.9rem;border-radius:6px;border:1px solid rgba(243,156,18,0.5);background:rgba(243,156,18,0.15);color:#f39c12;font-weight:bold;font-size:0.78rem;text-decoration:none;">
                    🤖 Dev LLM Bitstream Implementation Guide
                </a>
                <a href="/i1/nexus" target="_blank" style="padding:0.5rem 0.9rem;border-radius:6px;border:1px solid rgba(255,255,255,0.2);color:#ccc;font-size:0.78rem;text-decoration:none;">
                    View Raw JSON (/i1/nexus)
                </a>
            </div>
            <div id="i1-nibbles-list" style="max-height:180px;overflow-y:auto;font-size:0.76rem;color:#ccc;background:rgba(0,0,0,0.25);padding:0.65rem;border-radius:6px;">
                Loading i1.is Knowledge Nibbles &amp; Chain of Title deeds...
            </div>
        </div>

        <!-- Model Safety -->
        <div class="card">
            <h2>🛡️ Model Safety Scores</h2>
            <p style="font-size:0.8rem;color:#888;margin-bottom:1rem">Based on observed behavior</p>
            <div class="endpoint">
                <span>🔺 Local Ternary Model (SPQT Base-243)</span>
                <span class="score safe">10/10 Sovereign</span>
            </div>
            <div class="endpoint">
                <span>🧬 MEM8 Local Persona (.m8)</span>
                <span class="score safe">10/10 Sovereign</span>
            </div>
            <div class="endpoint">
                <span>Claude Opus 4.5 / Sonnet 4.5</span>
                <span class="score safe">10/10</span>
            </div>
            <div class="endpoint">
                <span>GPT-5.2 Codex</span>
                <span class="score safe">9/10</span>
            </div>
            <div class="endpoint">
                <span>Gemini 3 Pro</span>
                <span class="score safe">9/10</span>
            </div>
            <div class="endpoint">
                <span style="color:#e74c3c">greatcoderMDK</span>
                <span class="score danger">2/10</span>
            </div>
        </div>

        <!-- Settings -->
        <div class="card" style="grid-column: 1 / -1;">
            <h2>⚙️ Configuration</h2>
            <p style="font-size:0.8rem;color:#888;margin-bottom:1rem">
                Edit <code>~/.st/config.toml</code> to add API keys and preferences
            </p>
            <div style="display:flex;gap:1rem;flex-wrap:wrap;">
                <a href="/settings" class="dashboard-link" style="font-size:0.9rem;padding:0.5rem 1rem;">View Config</a>
                <a href="/v1/models" class="dashboard-link" style="font-size:0.9rem;padding:0.5rem 1rem;background:linear-gradient(135deg,#27ae60,#2ecc71);">Available Models</a>
            </div>
            <pre id="config-preview" style="margin-top:1rem;background:rgba(0,0,0,0.3);padding:1rem;border-radius:8px;font-size:0.75rem;max-height:150px;overflow:auto;display:none;"></pre>
        </div>
    </div>

    <p class="custodian">🧹 The Custodian is watching all operations</p>

    <script>
        // Automagic auth for externally-bound daemons: `/?token=<token>` is
        // stored locally; localhost needs no token at all.
        try {
            const tk = new URLSearchParams(location.search).get('token');
            if (tk) localStorage.setItem('st_daemon_token', tk);
        } catch (e) {}
        const ST_TOKEN = (function () { try { return localStorage.getItem('st_daemon_token') || ''; } catch (e) { return ''; } })();
        const ST_HEADERS = ST_TOKEN ? { 'Content-Type': 'application/json', 'Authorization': 'Bearer ' + ST_TOKEN } : { 'Content-Type': 'application/json' };
        const messages = document.getElementById('chat-messages');
        const log = document.getElementById('transparency-log');

        function addLog(type, content) {
            const time = new Date().toLocaleTimeString();
            const typeClass = type.toLowerCase().includes('mcp') ? 'mcp' :
                             type.toLowerCase().includes('llm') ? 'llm' : 'tool';
            log.innerHTML += `<div class="log-entry">
                <span class="time">${time}</span>
                <span class="type ${typeClass}">${type}</span>
                <div class="content">${content.substring(0, 200)}${content.length > 200 ? '...' : ''}</div>
            </div>`;
            log.scrollTop = log.scrollHeight;
        }

        async function sendChat() {
            const input = document.getElementById('msg-input');
            const model = document.getElementById('model-select').value;
            const msg = input.value.trim();
            if (!msg) return;

            // Add user message
            messages.innerHTML += `<div class="msg user">${msg}</div>`;
            input.value = '';

            // Log the request
            addLog('LLM-REQ', `Model: ${model} | "${msg}"`);

            try {
                const res = await fetch('/v1/chat/completions', {
                    method: 'POST',
                    headers: ST_HEADERS,
                    body: JSON.stringify({
                        model: model,
                        messages: [{ role: 'user', content: msg }],
                        max_tokens: 500
                    })
                });

                const data = await res.json();
                let reply = data.choices?.[0]?.message?.content;
                if (!reply) {
                    // Handle error objects
                    if (data.error) {
                        reply = typeof data.error === 'object' ?
                            (data.error.message || JSON.stringify(data.error)) : data.error;
                    } else {
                        reply = 'No response';
                    }
                }
                const safeScore = model.includes('claude') ? 'safe' :
                                  model.includes('greatcoder') ? 'danger' : 'safe';

                messages.innerHTML += `<div class="msg ai">
                    <span class="model">${model}</span>
                    <span class="score ${safeScore}">${safeScore === 'safe' ? '✓' : '⚠'}</span>
                    <div>${reply}</div>
                </div>`;

                addLog('LLM-RES', `${model}: ${typeof reply === 'string' ? reply : JSON.stringify(reply)}`);
            } catch (e) {
                messages.innerHTML += `<div class="msg ai" style="color:#e74c3c">Error: ${e.message}</div>`;
                addLog('ERROR', e.message);
            }

            messages.scrollTop = messages.scrollHeight;
        }

        function specialtyBadge(spec) {
            if (spec === 'history-documenter') return '📜';
            if (spec === 'code-analyst') return '🔬';
            if (spec === 'readme-expert') return '📖';
            return '🧬';
        }

        async function loadPersonas() {
            const listEl = document.getElementById('personas-list');
            const optGroup = document.getElementById('persona-optgroup');
            const currentVal = document.getElementById('model-select').value;
            try {
                const res = await fetch('/api/personas', { headers: ST_HEADERS });
                const data = await res.json();
                const personas = data.personas || [];
                optGroup.innerHTML = '<option value="burn/mem8-ternary-micro">🔺 Local Ternary Model (SPQT Base-243 · Auto-Soul Router)</option><option value="burn/mem8-wave-micro">🕯️ Local MEM8 Wave Micro-Model</option>';
                if (personas.length === 0) {
                    listEl.innerHTML = '<div style="color:#888;">No trained personas yet. Enter a name and folder path above to train a <code>.m8</code> Soul!</div>';
                    return;
                }
                let html = '';
                for (const p of personas) {
                    const kb = (p.m8_size_bytes / 1024).toFixed(1);
                    const icon = specialtyBadge(p.specialty);
                    optGroup.innerHTML += `<option value="persona:${p.slug}">${icon} ${p.name} (${p.episode_count} memories)</option>`;
                    html += `<div style="display:flex;justify-content:space-between;align-items:center;padding:0.35rem 0;border-bottom:1px solid rgba(255,255,255,0.06);">
                        <div>
                            <strong style="color:#4ecdc4;">${icon} ${p.name}</strong>
                            <span style="color:#888;font-size:0.72rem;"> (${p.episode_count} memories · ${p.total_words} words · ${kb} KiB .m8)</span>
                        </div>
                        <button onclick="selectPersona('${p.slug}')" style="padding:2px 8px;border-radius:4px;border:none;background:#27ae60;color:white;font-size:0.72rem;cursor:pointer;">Chat</button>
                    </div>`;
                }
                listEl.innerHTML = html;
                if (currentVal) {
                    document.getElementById('model-select').value = currentVal;
                }
            } catch (e) {
                listEl.innerHTML = `<div style="color:#e74c3c;">Failed to load personas: ${e.message}</div>`;
            }
        }

        function selectPersona(slug) {
            const sel = document.getElementById('model-select');
            sel.value = 'persona:' + slug;
            document.getElementById('msg-input').focus();
            addLog('PERSONA', `Selected local MEM8 persona: persona:${slug}`);
        }

        async function trainPersona() {
            const nameInput = document.getElementById('persona-name');
            const pathInput = document.getElementById('persona-path');
            const specSelect = document.getElementById('persona-specialty');
            const name = nameInput.value.trim();
            const path = pathInput.value.trim();
            const specialty = specSelect ? specSelect.value : 'life-biographer';
            if (!name || !path) {
                addLog('ERROR', 'Please enter both a Persona Name and a Folder/File Path');
                return;
            }
            addLog('PERSONA-TRAIN', `Training ${specialty} "${name}" from ${path} (up to 250,000+ memories)...`);
            try {
                const res = await fetch('/api/personas/train', {
                    method: 'POST',
                    headers: ST_HEADERS,
                    body: JSON.stringify({ name, path, specialty })
                });
                const data = await res.json();
                if (!res.ok || data.error) {
                    addLog('ERROR', data.error || 'Training failed');
                    return;
                }
                addLog('PERSONA-READY', `Trained ${data.name} (${data.episodes} memories, ${(data.m8_size_bytes/1024).toFixed(1)} KiB .m8)`);
                nameInput.value = '';
                await loadPersonas();
                selectPersona(data.slug);
            } catch (e) {
                addLog('ERROR', e.message);
            }
        }

        async function trainSpecialties() {
            const pathInput = document.getElementById('persona-path');
            const path = (pathInput.value || '').trim() || '.';
            addLog('SPECIALISTS-TRAIN', `Training 3 Specialist Souls (History Documenter, Code Analyst, README Expert) on ${path}...`);
            try {
                const res = await fetch('/api/personas/train-specialties', {
                    method: 'POST',
                    headers: ST_HEADERS,
                    body: JSON.stringify({ path })
                });
                const data = await res.json();
                if (!res.ok || data.error) {
                    addLog('ERROR', data.error || 'Specialist training failed');
                    return;
                }
                addLog('SPECIALISTS-READY', `Trained ${(data.trained || []).length} specialist souls on ${path}`);
                await loadPersonas();
            } catch (e) {
                addLog('ERROR', e.message);
            }
        }

        async function loadI1Nexus() {
            const banner = document.getElementById('i1-repo-banner');
            const listEl = document.getElementById('i1-nibbles-list');
            const visSel = document.getElementById('i1-visibility-select');
            try {
                const res = await fetch('/i1/nexus', { headers: ST_HEADERS });
                const data = await res.json();
                const rep = data.repository_visibility || {};
                const stats = data.stats || {};
                if (visSel && data.config?.wave_visibility) {
                    visSel.value = data.config.wave_visibility;
                }
                const remotes = (rep.public_remotes || []).join(', ') || 'none (local only)';
                const badge = rep.effective_visibility === 'public-commons'
                    ? '<span style="color:#2ecc71;font-weight:bold;">🌐 PUBLIC COMMONS (Free · AI Training Permitted w/ Chain of Title)</span>'
                    : '<span style="color:#f39c12;font-weight:bold;">🔒 PRIVATE SOVEREIGN ($5.00/mo Lights-On Tier · Zero External AI Training)</span>';
                banner.innerHTML = `${badge} · Repo Public: <strong>${rep.is_public_repo ? 'Yes' : 'No'}</strong> · Remotes: <code>${remotes}</code> · License: <code>${rep.detected_license || 'n/a'}</code> · Commit: <code>${rep.head_commit || 'n/a'}</code> · Chains of Title: <strong>${stats.chains_of_title_count || 0}</strong> · Nibbles: <strong>${stats.nibbles_count || 0}</strong> (${stats.nibble_relations_count || 0} edges)`;

                const nibbles = data.nibbles || [];
                if (nibbles.length === 0) {
                    listEl.innerHTML = '<div style="color:#888;">No nibbles distilled yet. Click <strong>⚡ Distill Souls into i1.is Nibbles &amp; Chain of Title</strong> above to build the nibble relationship graph!</div>';
                    return;
                }
                let html = '';
                for (const n of nibbles.slice(0, 12)) {
                    const relCount = (n.relations || []).length;
                    const relPreview = (n.relations || []).slice(0, 3).map(r => `${r.kind}→${r.target_nibble_id.substring(0, 10)}`).join(', ');
                    html += `<div style="padding:0.35rem 0;border-bottom:1px solid rgba(255,255,255,0.06);">
                        <div>
                            <strong style="color:#4ecdc4;">${n.title}</strong>
                            <span style="color:#f39c12;font-size:0.7rem;"> [${n.chain_of_title_id} · Author: ${n.origin_author} · SPQT: 0x${(n.spqt_base243_13b_hex || '').substring(0, 8)}]</span>
                        </div>
                        <div style="color:#bbb;margin-top:2px;">${(n.content || '').substring(0, 160)}...</div>
                        <div style="color:#888;font-size:0.68rem;margin-top:2px;">🔗 ${relCount} nibble relationships: ${relPreview || 'standalone'}</div>
                    </div>`;
                }
                listEl.innerHTML = html;
            } catch (e) {
                banner.innerHTML = `<span style="color:#e74c3c;">Failed to load i1.is Nexus: ${e.message}</span>`;
            }
        }

        async function distillI1Nexus() {
            const selModel = document.getElementById('model-select').value || '';
            const persona = selModel.startsWith('persona:') ? selModel : null;
            addLog('I1-NEXUS', `Distilling ${persona || 'trained souls'} into i1.is Knowledge Nibbles & Chain of Title...`);
            try {
                const res = await fetch('/i1/nexus/distill', {
                    method: 'POST',
                    headers: ST_HEADERS,
                    body: JSON.stringify({ persona, max_nibbles: 20 })
                });
                const data = await res.json();
                addLog('I1-NEXUS-READY', `Distilled ${data.nibbles_count || 0} nibbles, ${data.chains_of_title_count || 0} Chains of Title, ${data.nibble_relations_count || 0} relationship edges`);
                await loadI1Nexus();
            } catch (e) {
                addLog('ERROR', e.message);
            }
        }

        async function saveI1NexusVisibility() {
            const vis = document.getElementById('i1-visibility-select').value;
            try {
                await fetch('/i1/nexus/settings', {
                    method: 'POST',
                    headers: ST_HEADERS,
                    body: JSON.stringify({ wave_visibility: vis })
                });
                addLog('I1-NEXUS', `Updated Wave Visibility policy to: ${vis}`);
                await loadI1Nexus();
            } catch (e) {
                addLog('ERROR', e.message);
            }
        }

        loadPersonas();
        loadI1Nexus();

        // Connect to WebSocket for live transparency
        try {
            const ws = new WebSocket(`ws://${location.host}/ws`);
            ws.onmessage = (e) => {
                try {
                    const data = JSON.parse(e.data);
                    addLog(data.type || 'EVENT', JSON.stringify(data));
                } catch { addLog('RAW', e.data); }
            };
            ws.onopen = () => addLog('SYSTEM', 'WebSocket connected for live updates');
        } catch (e) { console.log('WS not available'); }
    </script>
</body>
</html>"#,
    )
}

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

/// Get current configuration
async fn get_settings() -> axum::response::Response {
    use crate::config::StConfig;
    use axum::response::IntoResponse;

    match StConfig::load() {
        Ok(config) => {
            // Return as TOML for readability
            match toml::to_string_pretty(&config) {
                Ok(toml_str) => {
                    let html = format!(
                        r#"<!DOCTYPE html>
<html><head><title>Smart Tree Config</title>
<style>
body {{ font-family: monospace; background: #1a1a2e; color: #e0e0e0; padding: 2rem; }}
pre {{ background: rgba(0,0,0,0.3); padding: 1rem; border-radius: 8px; overflow-x: auto; }}
h1 {{ color: #4ecdc4; }}
.path {{ color: #888; font-size: 0.9rem; }}
a {{ color: #4ecdc4; }}
</style></head><body>
<h1>⚙️ Smart Tree Configuration</h1>
<p class="path">File: ~/.st/config.toml</p>
<pre>{}</pre>
<p><a href="/">← Back to Dashboard</a></p>
</body></html>"#,
                        toml_str
                    );
                    axum::response::Html(html).into_response()
                }
                Err(e) => (
                    axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                    format!("Failed to serialize config: {}", e),
                )
                    .into_response(),
            }
        }
        Err(e) => (
            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            format!("Failed to load config: {}", e),
        )
            .into_response(),
    }
}

/// Update configuration (POST JSON)
async fn update_settings(
    axum::extract::Json(updates): axum::extract::Json<serde_json::Value>,
) -> axum::response::Response {
    use crate::config::StConfig;
    use axum::response::IntoResponse;

    // Load existing config
    let mut config = match StConfig::load() {
        Ok(c) => c,
        Err(e) => {
            return (
                axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                format!("Failed to load config: {}", e),
            )
                .into_response()
        }
    };

    // Apply updates (simple key-based for now)
    if let Some(api_keys) = updates.get("api_keys").and_then(|v| v.as_object()) {
        if let Some(key) = api_keys.get("anthropic").and_then(|v| v.as_str()) {
            config.api_keys.anthropic = Some(key.to_string());
        }
        if let Some(key) = api_keys.get("openai").and_then(|v| v.as_str()) {
            config.api_keys.openai = Some(key.to_string());
        }
        if let Some(key) = api_keys.get("google").and_then(|v| v.as_str()) {
            config.api_keys.google = Some(key.to_string());
        }
    }

    // Save
    match config.save() {
        Ok(_) => {
            Json(serde_json::json!({"status": "ok", "message": "Config updated"})).into_response()
        }
        Err(e) => (
            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            format!("Failed to save config: {}", e),
        )
            .into_response(),
    }
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

// === Collaboration Station Handlers ===

/// Get current collaboration presence
async fn collab_presence(State(state): State<Arc<RwLock<DaemonState>>>) -> Json<serde_json::Value> {
    let s = state.read().await;
    let hub = s.collab_hub.read().await;
    let presence = hub.get_presence();
    let hot_tub_count = presence.iter().filter(|p| p.in_hot_tub).count();

    Json(serde_json::json!({
        "participants": presence,
        "total": presence.len(),
        "hot_tub_count": hot_tub_count,
        "hot_tub_open": hub.is_hot_tub_open()
    }))
}

/// WebSocket handler for collaboration
async fn collab_websocket_handler(
    ws: WebSocketUpgrade,
    State(state): State<Arc<RwLock<DaemonState>>>,
) -> impl IntoResponse {
    let hub = state.read().await.collab_hub.clone();
    ws.on_upgrade(move |socket| handle_collab_connection(socket, hub))
}

/// Handle a collaboration WebSocket connection
async fn handle_collab_connection(socket: axum::extract::ws::WebSocket, hub: SharedCollabHub) {
    use crate::collaboration::{Participant, ParticipantType};
    use axum::extract::ws::Message;
    use futures::{SinkExt, StreamExt};

    let (mut sender, mut receiver) = socket.split();

    // Wait for join message
    let participant_id = loop {
        match receiver.next().await {
            Some(Ok(Message::Text(text))) => {
                #[derive(serde::Deserialize)]
                struct JoinMsg {
                    action: String,
                    name: String,
                    participant_type: Option<String>,
                }

                if let Ok(join) = serde_json::from_str::<JoinMsg>(&text) {
                    if join.action == "join" {
                        let ptype = join
                            .participant_type
                            .map(|s| match s.to_lowercase().as_str() {
                                "human" | "user" => ParticipantType::Human,
                                "claude" => ParticipantType::Claude,
                                "omni" => ParticipantType::Omni,
                                "grok" => ParticipantType::Grok,
                                _ => ParticipantType::Unknown,
                            })
                            .unwrap_or(ParticipantType::Unknown);

                        let participant = Participant::new(join.name.clone(), ptype);
                        let id = hub.write().await.join(participant);

                        // Send welcome
                        let welcome = serde_json::json!({
                            "type": "welcome",
                            "participant_id": id,
                            "name": join.name
                        });
                        let _ = sender.send(Message::Text(welcome.to_string())).await;
                        break id;
                    }
                }
            }
            Some(Ok(Message::Close(_))) | None => return,
            _ => continue,
        }
    };

    // Subscribe to broadcasts
    let mut broadcast_rx = hub.read().await.subscribe();

    // Forward broadcasts to client
    let _hub_for_send = hub.clone();
    let pid_for_send = participant_id.clone();
    let mut send_task = tokio::spawn(async move {
        while let Ok(msg) = broadcast_rx.recv().await {
            let json = serde_json::to_string(&msg).unwrap_or_default();
            if sender.send(Message::Text(json)).await.is_err() {
                break;
            }
        }
        pid_for_send
    });

    // Handle incoming messages
    let hub_for_recv = hub.clone();
    let pid_for_recv = participant_id.clone();
    let mut recv_task = tokio::spawn(async move {
        while let Some(Ok(msg)) = receiver.next().await {
            if let Message::Text(text) = msg {
                #[derive(serde::Deserialize)]
                #[serde(tag = "action")]
                enum ClientMsg {
                    #[serde(rename = "chat")]
                    Chat { message: String },
                    #[serde(rename = "hot_tub")]
                    HotTub,
                    #[serde(rename = "status")]
                    Status {
                        status: Option<String>,
                        working_on: Option<String>,
                    },
                }

                if let Ok(client_msg) = serde_json::from_str::<ClientMsg>(&text) {
                    match client_msg {
                        ClientMsg::Chat { message } => {
                            hub_for_recv.read().await.chat(&pid_for_recv, message);
                        }
                        ClientMsg::HotTub => {
                            hub_for_recv.write().await.toggle_hot_tub(&pid_for_recv);
                        }
                        ClientMsg::Status { status, working_on } => {
                            hub_for_recv.write().await.update_status(
                                &pid_for_recv,
                                status,
                                working_on,
                            );
                        }
                    }
                }
            }
        }
        pid_for_recv
    });

    // Wait for either to finish
    tokio::select! {
        _ = &mut send_task => recv_task.abort(),
        _ = &mut recv_task => send_task.abort(),
    }

    // Clean up
    hub.write().await.leave(&participant_id);
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

    let mut scanned = SystemContext::default();
    for path in watch_paths {
        if !path.exists() {
            continue;
        }

        for entry in WalkDir::new(path)
            .max_depth(3)
            .follow_links(false)
            .into_iter()
            .filter_entry(|entry| {
                entry.depth() == 0
                    || (!search::ignored_directory(entry.path())
                        && !entry.file_name().to_string_lossy().starts_with('.'))
            })
            .filter_map(|e| e.ok())
        {
            let entry_path = entry.path();

            if entry_path.is_dir() {
                // Detect project
                if let Some(project) = detect_project(entry_path) {
                    scanned.projects.insert(entry_path.to_path_buf(), project);
                }

                // Create directory info
                if let Some(info) = create_directory_info(entry_path) {
                    scanned
                        .consciousnesses
                        .insert(entry_path.to_path_buf(), info);
                }
            }
        }
    }

    scanned.last_scan = Some(std::time::SystemTime::now());
    *context = scanned;
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
    use std::io::Read;
    for readme in ["CLAUDE.md", "README.md"] {
        let readme_path = path.join(readme);
        if std::fs::symlink_metadata(&readme_path).is_ok_and(|metadata| metadata.is_file()) {
            let mut content = String::new();
            let mut options = std::fs::OpenOptions::new();
            options.read(true);
            #[cfg(unix)]
            {
                use std::os::unix::fs::OpenOptionsExt;
                options.custom_flags(libc::O_NOFOLLOW | libc::O_NONBLOCK);
            }
            if options
                .open(&readme_path)
                .and_then(|file| file.take(64 * 1024).read_to_string(&mut content))
                .is_ok()
            {
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

    let mut patterns: Vec<_> = extensions.into_iter().collect();
    patterns.sort();
    Some(DirectoryInfo {
        path: path.to_string_lossy().to_string(),
        frequency,
        file_count,
        patterns,
    })
}

// =============================================================================
// MEM8 LIFE PERSONAS HANDLERS (`/api/personas`, `/api/personas/train`)
// =============================================================================

#[derive(Deserialize)]
struct TrainPersonaRequest {
    name: String,
    path: String,
    #[serde(default)]
    specialty: Option<String>,
}

#[derive(Deserialize)]
struct TrainSpecialtiesRequest {
    path: String,
}

async fn personas_list_handler() -> Json<serde_json::Value> {
    let personas = tokio::task::spawn_blocking(crate::mem8::persona::LifePersona::list_all)
        .await
        .ok()
        .and_then(|r| r.ok())
        .unwrap_or_default();
    Json(serde_json::json!({ "personas": personas }))
}

async fn personas_train_handler(
    Json(req): Json<TrainPersonaRequest>,
) -> (StatusCode, Json<serde_json::Value>) {
    let name = req.name.trim().to_string();
    let expanded = shellexpand::tilde(req.path.trim()).to_string();
    let path = PathBuf::from(expanded);
    let specialty = crate::mem8::persona::PersonaSpecialty::from_str_loose(
        req.specialty.as_deref().unwrap_or("life-biographer"),
    );
    let result = tokio::task::spawn_blocking(move || {
        crate::mem8::persona::LifePersona::train_with_specialty(&name, &path, specialty)
    })
    .await;

    match result {
        Ok(Ok((persona, m8_path, size))) => (
            StatusCode::OK,
            Json(serde_json::json!({
                "status": "ok",
                "name": persona.name,
                "slug": persona.slug,
                "specialty": persona.specialty.slug(),
                "source_path": persona.source_path,
                "files_ingested": persona.files_ingested,
                "total_words": persona.total_words,
                "episodes": persona.episodes.len(),
                "m8_path": m8_path.to_string_lossy(),
                "m8_size_bytes": size,
            })),
        ),
        Ok(Err(e)) => (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({ "error": e.to_string() })),
        ),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": e.to_string() })),
        ),
    }
}

async fn personas_train_specialties_handler(
    Json(req): Json<TrainSpecialtiesRequest>,
) -> (StatusCode, Json<serde_json::Value>) {
    let expanded = shellexpand::tilde(req.path.trim()).to_string();
    let path = PathBuf::from(expanded);
    let result = tokio::task::spawn_blocking(move || {
        crate::mem8::persona::LifePersona::train_workspace_specialties(&path)
    })
    .await;

    match result {
        Ok(results) => {
            let mut trained = Vec::new();
            for item in results.into_iter().flatten() {
                trained.push(item);
            }
            (
                StatusCode::OK,
                Json(serde_json::json!({
                    "status": "ok",
                    "trained": trained,
                })),
            )
        }
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": e.to_string() })),
        ),
    }
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
    // Smart routing: detect provider from model name if no explicit prefix
    let (provider_name, model_name) = if let Some(slug) = req.model.strip_prefix("persona:") {
        ("burn".to_string(), format!("persona:{slug}"))
    } else if let Some((p, m)) = req.model.split_once('/') {
        if p.eq_ignore_ascii_case("burn")
            || p.eq_ignore_ascii_case("candle")
            || p.eq_ignore_ascii_case("local")
            || p.eq_ignore_ascii_case("persona")
        {
            ("burn".to_string(), m.to_string())
        } else {
            (p.to_string(), m.to_string())
        }
    } else {
        // Auto-detect provider from model name
        let model_lower = req.model.to_lowercase();
        let provider = if model_lower.starts_with("persona")
            || model_lower.starts_with("mem8")
            || model_lower.starts_with("burn")
            || model_lower.starts_with("candle")
            || model_lower.starts_with("ternary")
        {
            "burn"
        } else if model_lower.starts_with("claude") {
            "openrouter" // Route Claude through OpenRouter (works with OPENROUTER_API_KEY)
        } else if model_lower.starts_with("gpt")
            || model_lower.starts_with("o1")
            || model_lower.starts_with("o3")
        {
            "openai"
        } else if model_lower.starts_with("gemini") {
            "google"
        } else if model_lower.starts_with("grok") {
            "grok"
        } else if model_lower.contains("llama")
            || model_lower.contains("mistral")
            || model_lower.contains("mixtral")
        {
            "openrouter" // Open models often via OpenRouter
        } else {
            "openrouter" // Default to OpenRouter as it supports many models
        };
        (provider.to_string(), req.model.clone())
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

    let (proxy, memory) = {
        let state = state.read().await;
        (
            Arc::clone(&state.llm_proxy),
            Arc::clone(&state.proxy_memory),
        )
    };
    let history_memory = Arc::clone(&memory);
    let history_scope = scope_id.clone();
    let history = tokio::task::spawn_blocking(move || {
        let memory = history_memory
            .lock()
            .map_err(|_| anyhow::anyhow!("Conversation memory lock poisoned"))?;
        Ok::<_, anyhow::Error>(memory.get_scope(&history_scope).cloned())
    })
    .await;
    let history = match history {
        Ok(Ok(history)) => history,
        error => {
            return memory_error_response(format!("Could not read conversation memory: {error:?}"))
        }
    };

    // Build the request from restored memory without locking the daemon.
    let request_with_history = {
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
        if let Some(scope) = history {
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

    let llm_result = proxy.complete(&provider_name, request_with_history).await;

    match llm_result {
        Ok(resp) => {
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
            let saved = tokio::task::spawn_blocking(move || {
                memory
                    .lock()
                    .map_err(|_| anyhow::anyhow!("Conversation memory lock poisoned"))?
                    .update_scope(&scope_id, new_history)
            })
            .await;
            if !matches!(saved, Ok(Ok(()))) {
                tracing::error!(error = ?saved, "Could not persist conversation response");
                return memory_error_response("The provider completed, but the conversation could not be saved. Check daemon storage before retrying.".into());
            }

            // Record credit for token savings (if we compressed context)
            let tokens_used = resp.usage.as_ref().map(|u| u.total_tokens).unwrap_or(0);
            if tokens_used > 0 {
                state.write().await.credits.record_savings(
                    tokens_used as u64 / 10, // Award 10% as savings
                    &format!("LLM call to {} ({})", provider_name, req.model),
                );
            }

            let mut resp_json = serde_json::to_value(OpenAiResponse {
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
            .unwrap_or_default();

            if let Some(obj) = resp_json.as_object_mut() {
                obj.insert(
                    "memnet_endpoint".to_string(),
                    serde_json::Value::String("/memnet/stream".to_string()),
                );
                obj.insert(
                    "llm_implementation_guide".to_string(),
                    serde_json::Value::String("/memnet/llm-guide".to_string()),
                );
                obj.insert(
                    "i1_nexus_endpoint".to_string(),
                    serde_json::Value::String("/i1/nexus".to_string()),
                );
                obj.insert(
                    "memnet_bitstream_hint".to_string(),
                    serde_json::Value::String(
                        "Recommended: POST /memnet/stream for native MEMNET bitstream (64B@0x62 LitanyWireFrame + 79B WaveFrame79 + 26B SPQT Base-243 trits; GET /memnet/llm-guide for LLM implementation instructions)".to_string(),
                    ),
                );
            }

            (
                StatusCode::OK,
                [
                    (
                        axum::http::HeaderName::from_static("x-smarttree-preferred-endpoint"),
                        "/memnet/stream",
                    ),
                    (
                        axum::http::HeaderName::from_static("x-smarttree-llm-guide"),
                        "/memnet/llm-guide",
                    ),
                    (
                        axum::http::HeaderName::from_static("x-smarttree-protocol"),
                        "MEMNET-64B-0x62-Wave79-SPQT-Bitstream",
                    ),
                ],
                Json(resp_json),
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

fn memory_error_response(message: String) -> axum::response::Response {
    (
        StatusCode::INTERNAL_SERVER_ERROR,
        Json(OpenAiErrorResponse {
            error: OpenAiError {
                message,
                error_type: "memory_error".into(),
                code: None,
            },
        }),
    )
        .into_response()
}

/// List available models from all providers (local ternary & MEM8 personas first)
async fn list_models(State(state): State<Arc<RwLock<DaemonState>>>) -> Json<serde_json::Value> {
    let state_lock = state.read().await;

    let mut models: Vec<serde_json::Value> = vec![
        serde_json::json!({
            "id": "burn/mem8-ternary-micro",
            "object": "model",
            "owned_by": "SmartTree-Burn-Ternary-SPQT-Base243",
            "default": true,
            "preferred_bitstream_endpoint": "/memnet/stream",
        }),
        serde_json::json!({
            "id": "burn/mem8-wave-micro",
            "object": "model",
            "owned_by": "SmartTree-Burn-MEM8-Wave",
            "preferred_bitstream_endpoint": "/memnet/stream",
        }),
    ];

    if let Ok(personas) = crate::mem8::persona::LifePersona::list_all() {
        for p in personas {
            models.push(serde_json::json!({
                "id": format!("persona:{}", p.slug),
                "object": "model",
                "owned_by": format!("MEM8-Soul ({})", p.specialty.slug()),
                "episodes": p.episode_count,
            }));
        }
    }

    for p in &state_lock.llm_proxy.providers {
        models.push(serde_json::json!({
            "id": format!("{}/default", p.name().to_lowercase()),
            "object": "model",
            "owned_by": p.name(),
        }));
    }

    Json(serde_json::json!({
        "object": "list",
        "data": models
    }))
}

// =============================================================================
// HOT WATCHER ENDPOINTS - Real-time directory intelligence (MEM8 waves)
// =============================================================================

/// Request to watch a directory
#[derive(Deserialize)]
struct WatchRequest {
    path: String,
}

/// Response with watched directory info
#[derive(Serialize)]
struct WatchResponse {
    success: bool,
    path: String,
    message: String,
}

/// Start watching a directory
async fn watch_directory(
    State(state): State<Arc<RwLock<DaemonState>>>,
    Json(req): Json<WatchRequest>,
) -> Result<Json<WatchResponse>, (StatusCode, String)> {
    let path = std::path::PathBuf::from(&req.path);

    if !path.exists() {
        return Err((
            StatusCode::NOT_FOUND,
            format!("Path does not exist: {}", req.path),
        ));
    }
    if !path.is_dir() {
        return Err((
            StatusCode::BAD_REQUEST,
            "Watch path must be a directory".into(),
        ));
    }

    match update_directory_memory(&state, move |memory| memory.watch(&path)).await {
        Ok(path) => Ok(Json(WatchResponse {
            success: true,
            path: path.display().to_string(),
            message: "Now watching directory with MEM8 waves".to_string(),
        })),
        Err(e) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Failed to watch: {}", e),
        )),
    }
}

/// Stop watching a directory
async fn unwatch_directory(
    State(state): State<Arc<RwLock<DaemonState>>>,
    Json(req): Json<WatchRequest>,
) -> Result<Json<WatchResponse>, (StatusCode, String)> {
    let path = std::path::PathBuf::from(&req.path);

    match update_directory_memory(&state, move |memory| memory.unwatch(&path)).await {
        Ok(path) => Ok(Json(WatchResponse {
            success: true,
            path: path.display().to_string(),
            message: "Stopped watching directory".to_string(),
        })),
        Err(e) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Failed to unwatch: {}", e),
        )),
    }
}

/// Hot watcher status response
#[derive(Serialize)]
struct WatchStatusResponse {
    total_watched: usize,
    critical: usize,
    hot: usize,
    warm: usize,
    cold: usize,
    average_arousal: f64,
}

/// Get hot watcher status
async fn watch_status(
    State(state): State<Arc<RwLock<DaemonState>>>,
) -> Result<Json<WatchStatusResponse>, (StatusCode, String)> {
    let summary = update_directory_memory(&state, |memory| Ok(memory.watcher.summary()))
        .await
        .map_err(|error| (StatusCode::INTERNAL_SERVER_ERROR, error.to_string()))?;

    Ok(Json(WatchStatusResponse {
        total_watched: summary.total_watched,
        critical: summary.critical,
        hot: summary.hot,
        warm: summary.warm,
        cold: summary.cold,
        average_arousal: summary.average_arousal,
    }))
}

/// Watched directory in response
#[derive(Serialize)]
struct WatchedDirectoryResponse {
    path: String,
    arousal: f64,
    valence: f64,
    frequency: f64,
    interest_level: String,
    security_findings: usize,
    is_hot: bool,
}

/// Get hot directories
async fn watch_hot_directories(
    State(state): State<Arc<RwLock<DaemonState>>>,
) -> Result<Json<Vec<WatchedDirectoryResponse>>, (StatusCode, String)> {
    let hot_dirs =
        update_directory_memory(&state, |memory| Ok(memory.watcher.get_hot_directories()))
            .await
            .map_err(|error| (StatusCode::INTERNAL_SERVER_ERROR, error.to_string()))?;

    let response: Vec<WatchedDirectoryResponse> = hot_dirs
        .into_iter()
        .map(|d| WatchedDirectoryResponse {
            path: d.path.display().to_string(),
            arousal: d.wave.arousal,
            valence: d.wave.emotional_valence,
            frequency: d.wave.frequency,
            interest_level: format!("{:?}", d.interest_level),
            security_findings: d.security_findings.len(),
            is_hot: d.is_hot(),
        })
        .collect();

    Ok(Json(response))
}

#[cfg(test)]
mod memory_api_tests {
    use super::*;

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn recall_http_uses_committed_context_while_index_writer_is_busy() {
        let temp = tempfile::tempdir().unwrap();
        let directory_memory = ContextMemory::open(temp.path()).unwrap();
        let recall_view = directory_memory.recall_view();
        let mut security = crate::magiscanner::SecurityConfig::default();
        security.database.path = temp.path().join("security.db").display().to_string();
        let state = Arc::new(RwLock::new(DaemonState {
            context: directory_memory.context(),
            credits: CreditTracker::default(),
            config: DaemonConfig::default(),
            shutdown_tx: None,
            llm_proxy: Arc::new(LlmProxy::new()),
            proxy_memory: Arc::new(std::sync::Mutex::new(
                ProxyMemory::open(temp.path()).unwrap(),
            )),
            collab_hub: create_hub(),
            sessions: create_session_store(),
            github_oauth: None,
            directory_memory: Arc::new(std::sync::Mutex::new(directory_memory)),
            recall_view,
            scan_memory: Arc::new(std::sync::Mutex::new(
                crate::magiscanner::memory::ScanMemory::open(&security).unwrap(),
            )),
            compute_pool: Arc::new(ComputePool::new(
                2,
                Arc::new(crate::mcp::combinatorics_llm::LlamaModelCache::new(2)),
            )),
        }));
        let app = Router::new()
            .route("/context/remember", post(remember_context))
            .route("/context/recall", post(recall_context))
            .with_state(Arc::clone(&state));
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let base = format!("http://{}", listener.local_addr().unwrap());
        let server = tokio::spawn(async move {
            axum::serve(listener, app).await.unwrap();
        });
        let client = reqwest::Client::new();
        let response = client
            .post(format!("{base}/context/remember"))
            .json(&serde_json::json!({
                "path": temp.path().join("homework.pdf"), "people": ["daughter"],
                "notes": "Lunar homework together", "occurred_at": "2026-09-03T16:00:00Z",
            }))
            .send()
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::OK);
        assert_eq!(
            response.json::<serde_json::Value>().await.unwrap()["stored"],
            true
        );

        let memory = state.read().await.directory_memory.clone();
        let (locked_tx, locked_rx) = oneshot::channel();
        let (release_tx, release_rx) = std::sync::mpsc::channel();
        let writer = tokio::task::spawn_blocking(move || {
            let _guard = memory.lock().unwrap();
            locked_tx.send(()).unwrap();
            release_rx.recv().unwrap();
        });
        locked_rx.await.unwrap();
        let result = tokio::time::timeout(std::time::Duration::from_secs(3), client.post(format!("{base}/context/recall")).json(&serde_json::json!({
            "query": "document with my daughter", "after": "2026-09-01T00:00:00Z", "before": "2026-09-07T00:00:00Z",
        })).send()).await;
        release_tx.send(()).unwrap();
        writer.await.unwrap();
        let response = result
            .expect("Recall blocked behind the storage writer")
            .unwrap();
        assert_eq!(response.status(), StatusCode::OK);
        let hits = response.json::<serde_json::Value>().await.unwrap();
        assert_eq!(hits.as_array().unwrap().len(), 1);
        assert!(hits[0]["evidence"][0]
            .as_str()
            .unwrap()
            .contains("daughter"));
        let invalid = client.post(format!("{base}/context/recall")).json(&serde_json::json!({
            "query": "homework", "after": "2026-10-01T00:00:00Z", "before": "2026-09-01T00:00:00Z",
        })).send().await.unwrap();
        assert_eq!(invalid.status(), StatusCode::BAD_REQUEST);
        drop(client);
        server.abort();
        let _ = server.await;
    }

    #[tokio::test]
    async fn ct_token_matches_and_mcp_auth_middleware_enforced() {
        assert!(ct_token_matches(
            "st_secret_token_123",
            "st_secret_token_123"
        ));
        assert!(!ct_token_matches(
            "st_secret_token_123",
            "st_secret_token_124"
        ));
        assert!(!ct_token_matches("st_secret_token_123", "st_secret"));
        assert!(!ct_token_matches("st_secret_token_123", ""));

        // Externally bound: token enforced for /mcp.
        let mcp_sub = Router::new().route("/", get(|| async { "mcp-ok" }));
        let external_app = Router::new()
            .route("/health", get(health))
            .nest_service("/mcp", mcp_sub)
            .layer(middleware::from_fn_with_state(
                AuthState::new(vec!["st_valid_token".to_string()], true),
                auth_middleware,
            ));

        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let base = format!("http://{}", listener.local_addr().unwrap());
        let server = tokio::spawn(async move {
            axum::serve(listener, external_app).await.unwrap();
        });
        let client = reqwest::Client::new();

        // /health is public
        let h = client.get(format!("{base}/health")).send().await.unwrap();
        assert_eq!(h.status(), StatusCode::OK);

        // /mcp without token -> 401
        let unauth = client.get(format!("{base}/mcp")).send().await.unwrap();
        assert_eq!(unauth.status(), StatusCode::UNAUTHORIZED);

        // /mcp with wrong token -> 401
        let bad = client
            .get(format!("{base}/mcp"))
            .header("Authorization", "Bearer st_wrong_token")
            .send()
            .await
            .unwrap();
        assert_eq!(bad.status(), StatusCode::UNAUTHORIZED);

        // /mcp with valid token -> 200
        let ok = client
            .get(format!("{base}/mcp"))
            .header("Authorization", "Bearer st_valid_token")
            .send()
            .await
            .unwrap();
        assert_eq!(ok.status(), StatusCode::OK);

        drop(client);
        server.abort();
        let _ = server.await;

        // Localhost-only: loopback trusts local processes (like stdio), so
        // /mcp + / need no token at all.
        let local_mcp = Router::new().route("/", get(|| async { "mcp-ok" }));
        let local_app = Router::new()
            .route("/health", get(health))
            .nest_service("/mcp", local_mcp)
            .layer(middleware::from_fn_with_state(
                AuthState::new(vec!["st_valid_token".to_string()], false),
                auth_middleware,
            ));
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let base = format!("http://{}", listener.local_addr().unwrap());
        let server = tokio::spawn(async move {
            axum::serve(listener, local_app).await.unwrap();
        });
        let client = reqwest::Client::new();
        let open = client.get(format!("{base}/mcp")).send().await.unwrap();
        assert_eq!(open.status(), StatusCode::OK);
        drop(client);
        server.abort();
        let _ = server.await;
    }
}
