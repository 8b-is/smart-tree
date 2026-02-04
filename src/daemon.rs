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

// Collaboration Station
use crate::auth::{create_session_store, GitHubOAuthConfig, SharedSessionStore};
use crate::collaboration::{create_hub, SharedCollabHub};

// Hot Watcher - Wave-powered real-time directory intelligence
use crate::hot_watcher::HotWatcher;

// HTTP MCP with The Custodian
use crate::web_dashboard::mcp_http::{create_mcp_context, mcp_router};

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
    /// Collaboration hub - multi-AI real-time collaboration
    pub collab_hub: SharedCollabHub,
    /// Session store - GitHub OAuth sessions
    pub sessions: SharedSessionStore,
    /// GitHub OAuth config (if available)
    pub github_oauth: Option<GitHubOAuthConfig>,
    /// Hot Watcher - Wave-powered real-time directory intelligence (MEM8)
    pub hot_watcher: Arc<RwLock<HotWatcher>>,
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
    let hot_watcher = Arc::new(RwLock::new(HotWatcher::new()));
    println!("  🔥 Hot Watcher: ready (MEM8 waves)");

    let state = Arc::new(RwLock::new(DaemonState {
        context: SystemContext::default(),
        credits: CreditTracker::default(),
        config: config.clone(),
        shutdown_tx: Some(shutdown_tx),
        llm_proxy,
        proxy_memory,
        collab_hub,
        sessions,
        github_oauth,
        hot_watcher,
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
        .with_state(state)
        // HTTP MCP - Full protocol over HTTP! 🧹 The Custodian watches here
        // (uses nest_service to allow different state type)
        .nest_service("/mcp", mcp_router(mcp_context));

    let addr = SocketAddr::from(([0, 0, 0, 0], config.port));
    println!("Smart Tree Daemon listening on http://{}", addr);
    println!("  - CLI Scan:     /cli/scan (thin-client endpoint!)");
    println!("  - CLI Stream:   /cli/stream (SSE streaming)");
    println!("  - MCP HTTP:     /mcp/* (The Custodian watching!) 🧹");
    println!("  - Context API:  /context");
    println!("  - Credits:      /credits");
    println!("  - Tools:        /tools (legacy)");
    println!("  - LLM Proxy:    /v1/chat/completions (OpenAI-compatible!)");
    println!("  - Models:       /v1/models");
    println!("  - Collab:       /collab/ws (Hot Tub Mode!) 🛁");
    println!("  - Hot Watcher:  /watch (MEM8 real-time intelligence) 🔥");
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

async fn welcome_page() -> axum::response::Html<&'static str> {
    axum::response::Html(r#"<!DOCTYPE html>
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
        <p style="color:#888;font-size:0.85rem;">Bookmark this page: <strong>http://localhost:8420</strong></p>
    </div>

    <div class="grid">
        <!-- Chat Test -->
        <div class="card">
            <h2>💬 Test Chat (LLM Proxy via OpenRouter)</h2>
            <div id="chat-messages"></div>
            <div id="chat-input">
                <select id="model-select">
                    <optgroup label="Top Coding Models">
                        <option value="anthropic/claude-opus-4.5">Claude Opus 4.5</option>
                        <option value="anthropic/claude-sonnet-4.5">Claude Sonnet 4.5</option>
                        <option value="openai/gpt-5.2-codex">GPT-5.2 Codex</option>
                        <option value="google/gemini-3-pro-preview">Gemini 3 Pro</option>
                        <option value="deepseek/deepseek-v3.2">DeepSeek V3.2</option>
                        <option value="qwen/qwen3-coder">Qwen3 Coder 480B</option>
                    </optgroup>
                    <optgroup label="Fast & Efficient">
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
                <input type="text" id="msg-input" placeholder="Type a message..." onkeypress="if(event.key==='Enter')sendChat()">
                <button onclick="sendChat()">Send</button>
            </div>
            <p style="font-size:0.7rem;color:#666;margin-top:0.5rem;">Uses OpenRouter - add OPENROUTER_API_KEY to config</p>
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
            <div class="endpoint"><span>Chat API</span><span>/v1/chat/completions</span></div>
        </div>

        <!-- Model Safety -->
        <div class="card">
            <h2>🛡️ Model Safety Scores</h2>
            <p style="font-size:0.8rem;color:#888;margin-bottom:1rem">Based on observed behavior</p>
            <div class="endpoint">
                <span>Claude 3.5 Sonnet</span>
                <span class="score safe">10/10</span>
            </div>
            <div class="endpoint">
                <span>GPT-4o</span>
                <span class="score safe">9/10</span>
            </div>
            <div class="endpoint">
                <span>Gemini 2.0</span>
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
                    headers: { 'Content-Type': 'application/json' },
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
</html>"#)
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
    use axum::response::IntoResponse;
    use crate::config::StConfig;

    match StConfig::load() {
        Ok(config) => {
            // Return as TOML for readability
            match toml::to_string_pretty(&config) {
                Ok(toml_str) => {
                    let html = format!(r#"<!DOCTYPE html>
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
</body></html>"#, toml_str);
                    axum::response::Html(html).into_response()
                }
                Err(e) => (
                    axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                    format!("Failed to serialize config: {}", e)
                ).into_response()
            }
        }
        Err(e) => (
            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            format!("Failed to load config: {}", e)
        ).into_response()
    }
}

/// Update configuration (POST JSON)
async fn update_settings(
    axum::extract::Json(updates): axum::extract::Json<serde_json::Value>
) -> axum::response::Response {
    use axum::response::IntoResponse;
    use crate::config::StConfig;

    // Load existing config
    let mut config = match StConfig::load() {
        Ok(c) => c,
        Err(e) => return (
            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            format!("Failed to load config: {}", e)
        ).into_response()
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
        Ok(_) => Json(serde_json::json!({"status": "ok", "message": "Config updated"})).into_response(),
        Err(e) => (
            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            format!("Failed to save config: {}", e)
        ).into_response()
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
async fn collab_presence(
    State(state): State<Arc<RwLock<DaemonState>>>,
) -> Json<serde_json::Value> {
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
async fn handle_collab_connection(
    socket: axum::extract::ws::WebSocket,
    hub: SharedCollabHub,
) {
    use axum::extract::ws::Message;
    use futures::{SinkExt, StreamExt};
    use crate::collaboration::{Participant, ParticipantType};

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
                        let ptype = join.participant_type
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
                    Status { status: Option<String>, working_on: Option<String> },
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
                            hub_for_recv.write().await.update_status(&pid_for_recv, status, working_on);
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
    // Smart routing: detect provider from model name if no explicit prefix
    let (provider_name, model_name) = if let Some((p, m)) = req.model.split_once('/') {
        (p.to_string(), m.to_string())
    } else {
        // Auto-detect provider from model name
        let model_lower = req.model.to_lowercase();
        let provider = if model_lower.starts_with("claude") {
            "openrouter" // Route Claude through OpenRouter (works with OPENROUTER_API_KEY)
        } else if model_lower.starts_with("gpt") || model_lower.starts_with("o1") || model_lower.starts_with("o3") {
            "openai"
        } else if model_lower.starts_with("gemini") {
            "google"
        } else if model_lower.starts_with("grok") {
            "grok"
        } else if model_lower.contains("llama") || model_lower.contains("mistral") || model_lower.contains("mixtral") {
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

    let state_lock = state.read().await;
    let mut watcher = state_lock.hot_watcher.write().await;

    match watcher.watch(&path) {
        Ok(()) => Ok(Json(WatchResponse {
            success: true,
            path: req.path,
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

    let state_lock = state.read().await;
    let mut watcher = state_lock.hot_watcher.write().await;

    match watcher.unwatch(&path) {
        Ok(()) => Ok(Json(WatchResponse {
            success: true,
            path: req.path,
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
) -> Json<WatchStatusResponse> {
    let state_lock = state.read().await;
    let watcher = state_lock.hot_watcher.read().await;
    let summary = watcher.summary();

    Json(WatchStatusResponse {
        total_watched: summary.total_watched,
        critical: summary.critical,
        hot: summary.hot,
        warm: summary.warm,
        cold: summary.cold,
        average_arousal: summary.average_arousal,
    })
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
) -> Json<Vec<WatchedDirectoryResponse>> {
    let state_lock = state.read().await;
    let watcher = state_lock.hot_watcher.read().await;
    let hot_dirs = watcher.get_hot_directories();

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

    Json(response)
}
