//! Web Dashboard - Browser-based terminal + file browser
//!
//! A lightweight alternative to egui that runs in any browser.
//! Features:
//! - Real PTY terminal (bash/zsh with colors, vim support)
//! - File browser with navigation
//! - Markdown preview
//! - Cool terminal aesthetic

mod api;
mod assets;
mod pty;
mod server;
mod websocket;

pub use server::start_server;

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Shared state for the web dashboard
#[derive(Debug, Default)]
pub struct DashboardState {
    /// Current working directory for file browser
    pub cwd: PathBuf,
    /// Active PTY sessions
    pub pty_sessions: HashMap<String, pty::PtyHandle>,
    /// Number of active WebSocket connections
    pub connections: usize,
}

impl DashboardState {
    pub fn new(cwd: PathBuf) -> Self {
        Self {
            cwd,
            pty_sessions: HashMap::new(),
            connections: 0,
        }
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
