//! Web Dashboard - Browser-based terminal + file browser
//!
//! A lightweight alternative to egui that runs in any browser.
//! Features:
//! - Real PTY terminal (bash/zsh with colors, vim support)
//! - File browser with navigation
//! - Markdown preview
//! - Cool terminal aesthetic
//! - Real-time MCP activity visualization (Wave Compass)
//! - User hints/nudges for AI collaboration
//! - HTTP MCP endpoints (The Custodian watches all operations)

mod api;
mod assets;
mod collab;
pub mod mcp_http;
mod pty;
mod server;
pub mod state_sync;
pub mod voice;
mod websocket;

pub use server::start_server;
pub use state_sync::{McpActivityState, UserHintsQueue};
pub use mcp_http::{SharedMcpContext, create_mcp_context, mcp_router};

use crate::in_memory_logger::InMemoryLogStore;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Shared MCP activity state (thread-safe access from MCP and dashboard)
pub type SharedMcpActivity = Arc<RwLock<McpActivityState>>;

/// Shared user hints queue (browser → MCP)
pub type SharedUserHints = Arc<RwLock<UserHintsQueue>>;

/// Shared state for the web dashboard
#[derive(Debug)]
pub struct DashboardState {
    /// Current working directory for file browser
    pub cwd: PathBuf,
    /// Active PTY sessions
    pub pty_sessions: HashMap<String, pty::PtyHandle>,
    /// Number of active WebSocket connections
    pub connections: usize,
    /// In-memory store for recent log entries
    pub log_store: InMemoryLogStore,
    /// Real-time MCP activity tracking (for Wave Compass)
    pub mcp_activity: SharedMcpActivity,
    /// User hints queue (from browser to AI)
    pub user_hints: SharedUserHints,
}

impl DashboardState {
    pub fn new(cwd: PathBuf, log_store: InMemoryLogStore) -> Self {
        Self {
            cwd,
            pty_sessions: HashMap::new(),
            connections: 0,
            log_store,
            mcp_activity: Arc::new(RwLock::new(McpActivityState::default())),
            user_hints: Arc::new(RwLock::new(UserHintsQueue::default())),
        }
    }

    /// Get a clone of the MCP activity state for sharing with MCP context
    pub fn mcp_activity_handle(&self) -> SharedMcpActivity {
        Arc::clone(&self.mcp_activity)
    }

    /// Get a clone of the user hints queue for sharing with MCP context
    pub fn user_hints_handle(&self) -> SharedUserHints {
        Arc::clone(&self.user_hints)
    }
}

/// Message types for terminal WebSocket
#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum TerminalMessage {
    /// Input from client to PTY
    Input { data: String },
    /// Resize terminal
    Resize { cols: u16, rows: u16 },
    /// Output from PTY to client
    Output { data: String },
    /// A system message (e.g., connection info)
    System { message: String },
    /// PTY process exited
    Exit { code: i32 },
    /// Error occurred
    Error { message: String },
    /// Keepalive ping
    Ping,
    /// Keepalive pong
    Pong,
}

/// File tree node for API responses
#[derive(Debug, Serialize, Deserialize)]
pub struct FileTreeNode {
    pub name: String,
    pub path: String,
    pub is_dir: bool,
    pub size: u64,
    pub modified: i64,
    pub file_type: String,
}

pub type SharedState = Arc<RwLock<DashboardState>>;
