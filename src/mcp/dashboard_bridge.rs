//! Dashboard Bridge - Connects MCP tool execution to real-time dashboard
//!
//! When the dashboard is running, this bridge pushes tool activity events
//! to the shared state for visualization in the Wave Compass.

use crate::web_dashboard::state_sync::{AccessType, McpActivityState, UserHint, UserHintsQueue};
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Bridge between MCP context and Dashboard state
#[derive(Clone)]
pub struct DashboardBridge {
    /// Shared MCP activity state (read by dashboard WebSocket)
    activity: Arc<RwLock<McpActivityState>>,
    /// Shared user hints queue (written by dashboard WebSocket)
    hints: Arc<RwLock<UserHintsQueue>>,
}

impl DashboardBridge {
    /// Create a new dashboard bridge with the given shared state handles
    pub fn new(
        activity: Arc<RwLock<McpActivityState>>,
        hints: Arc<RwLock<UserHintsQueue>>,
    ) -> Self {
        Self { activity, hints }
    }

    /// Record the start of a tool execution
    pub async fn tool_started(&self, tool_name: &str, parameters: serde_json::Value) {
        let mut activity = self.activity.write().await;
        activity.start_tool(tool_name, parameters);
    }

    /// Update tool progress (0.0 to 1.0)
    pub async fn tool_progress(&self, progress: f32) {
        let mut activity = self.activity.write().await;
        activity.update_progress(progress);
    }

    /// Record a file access event
    pub async fn file_accessed(&self, path: PathBuf, access_type: AccessType, tool_name: &str) {
        let mut activity = self.activity.write().await;
        activity.record_file_access(path, access_type, tool_name);
    }

    /// Record a file read
    pub async fn file_read(&self, path: impl Into<PathBuf>, tool_name: &str) {
        self.file_accessed(path.into(), AccessType::Read, tool_name)
            .await;
    }

    /// Record a file write
    pub async fn file_written(&self, path: impl Into<PathBuf>, tool_name: &str) {
        self.file_accessed(path.into(), AccessType::Write, tool_name)
            .await;
    }

    /// Record a file analysis (e.g., AST parsing)
    pub async fn file_analyzed(&self, path: impl Into<PathBuf>, tool_name: &str) {
        self.file_accessed(path.into(), AccessType::Analyze, tool_name)
            .await;
    }

    /// Record a search operation on a path
    pub async fn path_searched(&self, path: impl Into<PathBuf>, tool_name: &str) {
        self.file_accessed(path.into(), AccessType::Search, tool_name)
            .await;
    }

    /// Record tool completion
    pub async fn tool_completed(&self, success: bool, summary: &str) {
        let mut activity = self.activity.write().await;
        activity.complete_tool(success, summary);
    }

    /// Update the current operation description
    pub async fn set_operation(&self, operation: &str) {
        let mut activity = self.activity.write().await;
        activity.current_operation = operation.to_string();
    }

    /// Get and consume the next unconsumed user hint
    pub async fn consume_hint(&self) -> Option<UserHint> {
        let mut hints = self.hints.write().await;
        hints.consume_next()
    }

    /// Peek at unconsumed hints without consuming them
    pub async fn peek_hints(&self) -> Vec<UserHint> {
        let hints = self.hints.read().await;
        hints.peek_unconsumed().into_iter().cloned().collect()
    }

    /// Check if there are any pending hints
    pub async fn has_pending_hints(&self) -> bool {
        let hints = self.hints.read().await;
        hints.unconsumed_count() > 0
    }

    /// Get the current MCP activity state for display
    pub async fn get_activity_snapshot(&self) -> McpActivitySnapshot {
        let activity = self.activity.read().await;
        McpActivitySnapshot {
            active_tool: activity.active_tool.as_ref().map(|t| t.name.clone()),
            current_operation: activity.current_operation.clone(),
            files_touched_count: activity.files_touched.len(),
            directories_explored_count: activity.directories_explored.len(),
            tools_executed_count: activity.tool_history.len(),
        }
    }
}

/// Lightweight snapshot of MCP activity for logging/display
#[derive(Debug, Clone)]
pub struct McpActivitySnapshot {
    pub active_tool: Option<String>,
    pub current_operation: String,
    pub files_touched_count: usize,
    pub directories_explored_count: usize,
    pub tools_executed_count: usize,
}

/// Macro to wrap tool execution with dashboard bridge reporting
#[macro_export]
macro_rules! with_dashboard_bridge {
    ($ctx:expr, $tool_name:expr, $params:expr, $body:expr) => {{
        // Report tool start if dashboard is connected
        if let Some(ref bridge) = $ctx.dashboard_bridge {
            bridge.tool_started($tool_name, $params.clone()).await;
        }

        // Execute the tool
        let result = $body;

        // Report completion
        if let Some(ref bridge) = $ctx.dashboard_bridge {
            match &result {
                Ok(val) => {
                    let summary = match val.as_str() {
                        Some(s) if s.len() > 100 => format!("{}...", &s[..100]),
                        Some(s) => s.to_string(),
                        None => "OK".to_string(),
                    };
                    bridge.tool_completed(true, &summary).await;
                }
                Err(e) => {
                    bridge.tool_completed(false, &e.to_string()).await;
                }
            }
        }

        result
    }};
}
