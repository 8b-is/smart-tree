// WebSocket Server for Real-Time Dashboard Communication
// "60fps telepathy between human and AI!" 🧠↔️🤖
//
// This module provides a WebSocket server that runs alongside the MCP server,
// enabling real-time bidirectional communication with the egui dashboard.
//
// Architecture:
// - Embedded axum HTTP server on localhost:8420
// - WebSocket endpoint at /ws for dashboard connection
// - Shared Arc<DashboardState> between MCP tools and WebSocket handlers
// - Delta-based state updates to minimize bandwidth
// - 60fps update rate (16ms intervals)

use anyhow::Result;
use axum::{
    extract::{
        ws::{Message, WebSocket, WebSocketUpgrade},
        State as AxumState,
    },
    response::IntoResponse,
    routing::get,
    Router,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::broadcast;

use crate::dashboard_egui::{
    ActivityStatus, DashboardState, FileAccessEvent, FileAccessType, HintType, UserHint,
};

// ============================================================================
// State Update Messages - "Only send what changed!" 📦
// ============================================================================

/// Delta update sent to dashboard (60fps = every 16ms)
#[derive(Clone, Debug, Serialize)]
#[serde(tag = "type")]
pub enum StateUpdate {
    /// MCP activity changed
    McpActivity {
        current_operation: String,
        files_touched: Vec<String>,
        status: String,
        progress: f32,
    },

    /// New file accessed (for Wave Compass lighting)
    FileAccess {
        path: String,
        access_type: String,
        tool_name: String,
        duration_ms: u64,
    },

    /// Tool started executing
    ToolStarted {
        tool_name: String,
        parameters: String,
    },

    /// Tool finished
    ToolFinished { tool_name: String, success: bool },

    /// User hint acknowledged by AI
    HintAcknowledged { hint_id: usize },

    /// WebSocket connection count changed
    ConnectionCount { count: usize },
}

/// User hint sent from dashboard to MCP
#[derive(Clone, Debug, Deserialize)]
#[serde(tag = "type")]
pub enum UserMessage {
    /// User clicked on Wave Compass signature
    ClickHint { path: String, signature: u64 },

    /// User typed a text hint
    TextHint { text: String },

    /// User sent voice command
    VoiceHint { transcript: String, confidence: f32 },

    /// User adjusted a parameter
    ParameterHint { param_name: String, value: f32 },

    /// Ping to keep connection alive
    Ping,
}

// ============================================================================
// WebSocket Server - "Real-time collaboration server!" 🚀
// ============================================================================

/// Broadcast channel for state updates (many receivers)
pub type UpdateSender = broadcast::Sender<StateUpdate>;

/// WebSocket server state shared between handlers
#[derive(Clone)]
pub struct WsServerState {
    /// Shared dashboard state (read by dashboard, written by MCP)
    pub dashboard_state: Arc<DashboardState>,

    /// Broadcast channel for sending state updates to all connected clients
    pub update_tx: UpdateSender,
}

/// Start the WebSocket server on localhost:8420
///
/// This runs in a background tokio task and doesn't block.
/// The dashboard (egui) connects via WebSocket to receive real-time updates.
///
/// # Architecture Note
/// The server runs in the same process as the MCP server, sharing the
/// DashboardState. This enables sub-millisecond latency for updates.
pub async fn start_ws_server(dashboard_state: Arc<DashboardState>) -> Result<()> {
    // Create broadcast channel for state updates (capacity: 100 messages)
    let (update_tx, _) = broadcast::channel::<StateUpdate>(100);

    let server_state = WsServerState {
        dashboard_state: dashboard_state.clone(),
        update_tx: update_tx.clone(),
    };

    // Build router with WebSocket endpoint
    let app = Router::new()
        .route("/ws", get(ws_handler))
        .route("/health", get(health_handler))
        .with_state(server_state);

    let listener = tokio::net::TcpListener::bind("127.0.0.1:8420").await?;

    println!("🌐 Dashboard WebSocket server running on ws://127.0.0.1:8420/ws");
    println!("💡 Connect your browser to see real-time AI activity!");

    // Spawn server in background task
    tokio::spawn(async move {
        if let Err(e) = axum::serve(listener, app).await {
            eprintln!("❌ WebSocket server error: {}", e);
        }
    });

    Ok(())
}

/// Health check endpoint
async fn health_handler() -> impl IntoResponse {
    "OK"
}

/// WebSocket upgrade handler
async fn ws_handler(
    ws: WebSocketUpgrade,
    AxumState(state): AxumState<WsServerState>,
) -> impl IntoResponse {
    ws.on_upgrade(|socket| handle_socket(socket, state))
}

/// Handle individual WebSocket connection
async fn handle_socket(socket: WebSocket, state: WsServerState) {
    println!("🔌 New dashboard connection!");

    // Increment connection count
    {
        let mut count = state.dashboard_state.ws_connections.write().unwrap();
        *count += 1;
        println!("📊 Active connections: {}", *count);
    }

    // Subscribe to broadcast updates
    let mut update_rx = state.update_tx.subscribe();

    // Split socket into sender and receiver
    let (mut sender, mut receiver) = socket.split();

    // Spawn task to send state updates to dashboard (60fps)
    let dashboard_state_clone = state.dashboard_state.clone();
    let send_task = tokio::spawn(async move {
        // Send initial full state
        if let Ok(initial_state) = get_full_state(&dashboard_state_clone) {
            if let Ok(json) = serde_json::to_string(&initial_state) {
                let _ = sender.send(Message::Text(json)).await;
            }
        }

        // Then send delta updates as they arrive
        while let Ok(update) = update_rx.recv().await {
            if let Ok(json) = serde_json::to_string(&update) {
                if sender.send(Message::Text(json)).await.is_err() {
                    break; // Client disconnected
                }
            }
        }
    });

    // Spawn task to receive user hints from dashboard
    let dashboard_state_clone = state.dashboard_state.clone();
    let recv_task = tokio::spawn(async move {
        while let Some(msg) = receiver.next().await {
            if let Ok(Message::Text(text)) = msg {
                if let Ok(user_msg) = serde_json::from_str::<UserMessage>(&text) {
                    handle_user_message(user_msg, &dashboard_state_clone);
                }
            }
        }
    });

    // Wait for either task to finish (connection closed)
    tokio::select! {
        _ = send_task => {},
        _ = recv_task => {},
    }

    // Decrement connection count
    {
        let mut count = state.dashboard_state.ws_connections.write().unwrap();
        *count = count.saturating_sub(1);
        println!("📊 Active connections: {}", *count);
    }

    println!("🔌 Dashboard disconnected");
}

/// Get full dashboard state for initial sync
fn get_full_state(state: &Arc<DashboardState>) -> Result<Vec<StateUpdate>> {
    let mut updates = Vec::new();

    // Send current MCP activity
    {
        let activity = state.mcp_activity.read().unwrap();
        updates.push(StateUpdate::McpActivity {
            current_operation: activity.current_operation.clone(),
            files_touched: activity.files_touched.clone(),
            status: format!("{:?}", activity.status),
            progress: activity.progress,
        });
    }

    // Send recent file accesses (last 50)
    {
        let file_log = state.file_access_log.read().unwrap();
        for event in file_log.iter().rev().take(50) {
            updates.push(StateUpdate::FileAccess {
                path: event.path.clone(),
                access_type: format!("{:?}", event.access_type),
                tool_name: event.tool_name.clone(),
                duration_ms: event.duration_ms,
            });
        }
    }

    // Send connection count
    {
        let count = *state.ws_connections.read().unwrap();
        updates.push(StateUpdate::ConnectionCount { count });
    }

    Ok(updates)
}

/// Handle user message from dashboard
fn handle_user_message(msg: UserMessage, state: &Arc<DashboardState>) {
    match msg {
        UserMessage::ClickHint { path, signature } => {
            println!("👆 User clicked: {} (sig: {:X})", path, signature);

            let hint = UserHint {
                hint_type: HintType::Click { path, signature },
                timestamp: chrono::Utc::now(),
                acknowledged: false,
            };

            let mut hints = state.user_hints.write().unwrap();
            hints.push_back(hint);

            // Keep queue limited to 100 hints
            while hints.len() > 100 {
                hints.pop_front();
            }
        }

        UserMessage::TextHint { text } => {
            println!("💬 User text hint: {}", text);

            let hint = UserHint {
                hint_type: HintType::TextInput { text },
                timestamp: chrono::Utc::now(),
                acknowledged: false,
            };

            let mut hints = state.user_hints.write().unwrap();
            hints.push_back(hint);

            while hints.len() > 100 {
                hints.pop_front();
            }
        }

        UserMessage::VoiceHint {
            transcript,
            confidence,
        } => {
            println!("🎤 User voice hint: {} ({:.0}%)", transcript, confidence * 100.0);

            let hint = UserHint {
                hint_type: HintType::Voice {
                    transcript,
                    confidence,
                },
                timestamp: chrono::Utc::now(),
                acknowledged: false,
            };

            let mut hints = state.user_hints.write().unwrap();
            hints.push_back(hint);

            while hints.len() > 100 {
                hints.pop_front();
            }
        }

        UserMessage::ParameterHint { param_name, value } => {
            println!("🎚️ User parameter adjust: {} = {:.2}", param_name, value);

            let hint = UserHint {
                hint_type: HintType::ParameterAdjust { param_name, value },
                timestamp: chrono::Utc::now(),
                acknowledged: false,
            };

            let mut hints = state.user_hints.write().unwrap();
            hints.push_back(hint);

            while hints.len() > 100 {
                hints.pop_front();
            }
        }

        UserMessage::Ping => {
            // Just a keepalive, no action needed
        }
    }
}

// ============================================================================
// Helper Functions for MCP Tools - "Notify dashboard of activity!" 📡
// ============================================================================

/// Notify dashboard that AI started an operation
pub fn notify_operation_start(
    state: &Arc<DashboardState>,
    operation: &str,
    status: ActivityStatus,
) {
    let mut activity = state.mcp_activity.write().unwrap();
    activity.current_operation = operation.to_string();
    activity.status = status;
    activity.progress = 0.0;
    activity.files_touched.clear();
    activity.started_at = chrono::Utc::now();
}

/// Notify dashboard of progress update
pub fn notify_progress(state: &Arc<DashboardState>, progress: f32) {
    let mut activity = state.mcp_activity.write().unwrap();
    activity.progress = progress.clamp(0.0, 1.0);
}

/// Notify dashboard that a file was accessed
pub fn notify_file_access(
    state: &Arc<DashboardState>,
    path: &str,
    access_type: FileAccessType,
    tool_name: &str,
    duration_ms: u64,
) {
    // Add to activity files
    {
        let mut activity = state.mcp_activity.write().unwrap();
        if !activity.files_touched.contains(&path.to_string()) {
            activity.files_touched.push(path.to_string());
        }
    }

    // Add to file access log
    {
        let event = FileAccessEvent {
            path: path.to_string(),
            access_type,
            timestamp: chrono::Utc::now(),
            tool_name: tool_name.to_string(),
            duration_ms,
        };

        let mut log = state.file_access_log.write().unwrap();
        log.push(event);

        // Keep log limited to 1000 entries
        while log.len() > 1000 {
            log.remove(0);
        }
    }
}

/// Notify dashboard that operation completed
pub fn notify_operation_complete(state: &Arc<DashboardState>) {
    let mut activity = state.mcp_activity.write().unwrap();
    activity.current_operation = "Idle".to_string();
    activity.status = ActivityStatus::Idle;
    activity.progress = 0.0;
}

/// Check for pending user hints (AI should look at these!)
pub fn get_pending_hints(state: &Arc<DashboardState>) -> Vec<UserHint> {
    let hints = state.user_hints.read().unwrap();
    hints
        .iter()
        .filter(|h| !h.acknowledged)
        .cloned()
        .collect()
}

/// Mark a hint as acknowledged
pub fn acknowledge_hint(state: &Arc<DashboardState>, hint_index: usize) {
    let mut hints = state.user_hints.write().unwrap();
    if let Some(hint) = hints.get_mut(hint_index) {
        hint.acknowledged = true;
    }
}

// ============================================================================
// Axum re-exports for split()
// ============================================================================

use futures_util::{SinkExt, StreamExt};
