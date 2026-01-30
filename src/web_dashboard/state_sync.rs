//! Real-time state synchronization between MCP and browser dashboard
//!
//! Provides WebSocket endpoint `/ws/state` for:
//! - Pushing MCP activity updates to browser at 60fps
//! - Receiving user hints/nudges from browser

use axum::{
    extract::{
        ws::{Message, WebSocket, WebSocketUpgrade},
        State,
    },
    response::Response,
};
use chrono::{DateTime, Utc};
use futures_util::{SinkExt, StreamExt};
use serde::{Deserialize, Serialize};
use std::collections::{HashSet, VecDeque};
use std::path::PathBuf;
use std::sync::Arc;
use std::time::{Duration, Instant};

use super::SharedState;

// ============================================================================
// MCP Activity Tracking Types
// ============================================================================

/// Currently executing MCP tool
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActiveTool {
    /// Tool name (e.g., "search", "read_file")
    pub name: String,
    /// When execution started (serialized as epoch millis)
    #[serde(with = "instant_serde")]
    pub started_at: Instant,
    /// Tool parameters (simplified for display)
    pub parameters: serde_json::Value,
    /// Estimated progress 0.0 to 1.0 (if available)
    pub progress: Option<f32>,
}

/// File access event for Wave Compass visualization
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileAccessEvent {
    /// File path accessed
    pub path: PathBuf,
    /// Type of access
    pub access_type: AccessType,
    /// When access occurred (serialized as epoch millis)
    #[serde(with = "instant_serde")]
    pub timestamp: Instant,
    /// Which MCP tool accessed it
    pub tool_name: String,
}

/// Type of file access
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum AccessType {
    Read,
    Write,
    Analyze,
    Search,
}

/// Completed tool execution record
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolExecution {
    /// Tool name
    pub name: String,
    /// When execution completed
    pub completed_at: DateTime<Utc>,
    /// How long it took
    pub duration_ms: u64,
    /// Whether it succeeded
    pub success: bool,
    /// Brief result summary
    pub summary: String,
}

/// Real-time MCP activity state for dashboard visualization
#[derive(Debug, Default)]
pub struct McpActivityState {
    /// Currently executing tool (if any)
    pub active_tool: Option<ActiveTool>,
    /// Recent file access events (ring buffer, last 100)
    pub file_access_log: VecDeque<FileAccessEvent>,
    /// Human-readable current operation
    pub current_operation: String,
    /// Files touched this session
    pub files_touched: HashSet<PathBuf>,
    /// Directories explored this session
    pub directories_explored: HashSet<PathBuf>,
    /// Recent tool executions (last 20)
    pub tool_history: VecDeque<ToolExecution>,
    /// Last state update timestamp
    pub last_update: Option<Instant>,
}

impl McpActivityState {
    /// Maximum file access events to keep
    const MAX_FILE_EVENTS: usize = 100;
    /// Maximum tool history entries
    const MAX_TOOL_HISTORY: usize = 20;

    /// Record start of a tool execution
    pub fn start_tool(&mut self, name: &str, parameters: serde_json::Value) {
        self.active_tool = Some(ActiveTool {
            name: name.to_string(),
            started_at: Instant::now(),
            parameters,
            progress: None,
        });
        self.current_operation = format!("Executing {}...", name);
        self.last_update = Some(Instant::now());
    }

    /// Update tool progress
    pub fn update_progress(&mut self, progress: f32) {
        if let Some(ref mut tool) = self.active_tool {
            tool.progress = Some(progress.clamp(0.0, 1.0));
            self.last_update = Some(Instant::now());
        }
    }

    /// Record file access event
    pub fn record_file_access(&mut self, path: PathBuf, access_type: AccessType, tool_name: &str) {
        let event = FileAccessEvent {
            path: path.clone(),
            access_type,
            timestamp: Instant::now(),
            tool_name: tool_name.to_string(),
        };

        self.file_access_log.push_back(event);
        if self.file_access_log.len() > Self::MAX_FILE_EVENTS {
            self.file_access_log.pop_front();
        }

        self.files_touched.insert(path.clone());
        if let Some(parent) = path.parent() {
            self.directories_explored.insert(parent.to_path_buf());
        }

        self.last_update = Some(Instant::now());
    }

    /// Complete tool execution
    pub fn complete_tool(&mut self, success: bool, summary: &str) {
        if let Some(tool) = self.active_tool.take() {
            let duration = tool.started_at.elapsed();
            let execution = ToolExecution {
                name: tool.name,
                completed_at: Utc::now(),
                duration_ms: duration.as_millis() as u64,
                success,
                summary: summary.to_string(),
            };

            self.tool_history.push_back(execution);
            if self.tool_history.len() > Self::MAX_TOOL_HISTORY {
                self.tool_history.pop_front();
            }
        }

        self.current_operation = if success {
            "Ready".to_string()
        } else {
            "Error occurred".to_string()
        };
        self.last_update = Some(Instant::now());
    }

    /// Get recent file events with age in milliseconds
    pub fn recent_file_events(&self, max_age_ms: u64) -> Vec<FileEventDto> {
        let now = Instant::now();
        self.file_access_log
            .iter()
            .filter_map(|event| {
                let age = now.duration_since(event.timestamp).as_millis() as u64;
                if age <= max_age_ms {
                    Some(FileEventDto {
                        path: event.path.to_string_lossy().to_string(),
                        access_type: event.access_type,
                        age_ms: age,
                        tool_name: event.tool_name.clone(),
                    })
                } else {
                    None
                }
            })
            .collect()
    }
}

// ============================================================================
// User Hints Types
// ============================================================================

/// User hint from browser to AI
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserHint {
    /// Type of hint
    pub hint_type: HintType,
    /// Content/description
    pub content: String,
    /// When hint was sent
    pub timestamp: DateTime<Utc>,
    /// Whether hint has been consumed by MCP
    #[serde(default)]
    pub consumed: bool,
}

/// Type of user hint
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum HintType {
    /// User clicked on Wave Compass region
    Click { target: String },
    /// User typed text hint
    Text { message: String },
    /// User voice input (transcribed)
    Voice { transcript: String, salience: f32 },
}

/// Queue of user hints waiting to be consumed by MCP
#[derive(Debug, Default)]
pub struct UserHintsQueue {
    pub hints: VecDeque<UserHint>,
    pub max_size: usize,
}

impl UserHintsQueue {
    const DEFAULT_MAX_SIZE: usize = 50;

    pub fn new() -> Self {
        Self {
            hints: VecDeque::new(),
            max_size: Self::DEFAULT_MAX_SIZE,
        }
    }

    /// Add a new hint
    pub fn push(&mut self, hint: UserHint) {
        self.hints.push_back(hint);
        while self.hints.len() > self.max_size {
            self.hints.pop_front();
        }
    }

    /// Get and consume the next unconsumed hint
    pub fn consume_next(&mut self) -> Option<UserHint> {
        for hint in &mut self.hints {
            if !hint.consumed {
                hint.consumed = true;
                return Some(hint.clone());
            }
        }
        None
    }

    /// Peek at unconsumed hints without consuming
    pub fn peek_unconsumed(&self) -> Vec<&UserHint> {
        self.hints.iter().filter(|h| !h.consumed).collect()
    }

    /// Count unconsumed hints
    pub fn unconsumed_count(&self) -> usize {
        self.hints.iter().filter(|h| !h.consumed).count()
    }

    /// Clear consumed hints older than threshold
    pub fn gc(&mut self, max_age: Duration) {
        let now = Utc::now();
        self.hints.retain(|h| {
            !h.consumed || (now - h.timestamp).num_milliseconds() < max_age.as_millis() as i64
        });
    }
}

// ============================================================================
// WebSocket Protocol DTOs
// ============================================================================

/// State update sent to browser (60fps)
#[derive(Debug, Serialize)]
pub struct StateUpdateDto {
    #[serde(rename = "type")]
    pub msg_type: &'static str,
    pub timestamp: i64,
    pub mcp: McpStateDto,
    pub file_log: Vec<FileEventDto>,
    pub wave_compass: WaveCompassDto,
    pub hints_pending: usize,
}

#[derive(Debug, Serialize)]
pub struct McpStateDto {
    pub active_tool: Option<String>,
    pub current_operation: String,
    pub progress: Option<f32>,
    pub tools_executed: usize,
}

#[derive(Debug, Serialize)]
pub struct FileEventDto {
    pub path: String,
    pub access_type: AccessType,
    pub age_ms: u64,
    pub tool_name: String,
}

#[derive(Debug, Serialize)]
pub struct WaveCompassDto {
    pub hot_regions: Vec<HotRegion>,
    pub trail: Vec<[f32; 2]>,
}

#[derive(Debug, Serialize)]
pub struct HotRegion {
    pub x: f32,
    pub y: f32,
    pub intensity: f32,
    pub label: String,
}

/// Hint message from browser
#[derive(Debug, Deserialize)]
pub struct HintMessageDto {
    #[serde(rename = "type")]
    pub msg_type: String,
    pub hint_type: String,
    #[serde(default)]
    pub target: Option<String>,
    #[serde(default)]
    pub content: Option<String>,
    #[serde(default)]
    pub transcript: Option<String>,
    #[serde(default)]
    pub salience: Option<f32>,
}

// ============================================================================
// WebSocket Handler
// ============================================================================

/// Handle WebSocket upgrade for state sync
pub async fn state_handler(
    ws: WebSocketUpgrade,
    State(state): State<SharedState>,
) -> Response {
    ws.on_upgrade(|socket| handle_state_socket(socket, state))
}

/// Handle the WebSocket connection for real-time state sync
async fn handle_state_socket(socket: WebSocket, state: SharedState) {
    let (mut sender, mut receiver) = socket.split();

    // Increment connection count
    {
        let mut dashboard = state.write().await;
        dashboard.connections += 1;
    }

    // Clone handles for tasks
    let mcp_activity = {
        let dashboard = state.read().await;
        Arc::clone(&dashboard.mcp_activity)
    };
    let user_hints = {
        let dashboard = state.read().await;
        Arc::clone(&dashboard.user_hints)
    };

    // Task: Send state updates at 60fps
    let mcp_activity_send = Arc::clone(&mcp_activity);
    let user_hints_send = Arc::clone(&user_hints);
    let send_task = tokio::spawn(async move {
        let mut interval = tokio::time::interval(Duration::from_millis(16)); // ~60fps
        let mut last_update: Option<Instant> = None;

        loop {
            interval.tick().await;

            let activity = mcp_activity_send.read().await;

            // Skip if no updates since last send
            if activity.last_update == last_update {
                continue;
            }
            last_update = activity.last_update;

            // Build state update
            let update = build_state_update(&activity, &*user_hints_send.read().await);
            drop(activity);

            let json = match serde_json::to_string(&update) {
                Ok(j) => j,
                Err(_) => continue,
            };

            if sender.send(Message::Text(json)).await.is_err() {
                break; // Client disconnected
            }
        }
    });

    // Task: Receive hints from browser
    let recv_task = tokio::spawn(async move {
        while let Some(Ok(msg)) = receiver.next().await {
            if let Message::Text(text) = msg {
                if let Ok(hint_msg) = serde_json::from_str::<HintMessageDto>(&text) {
                    if hint_msg.msg_type == "hint" {
                        let hint = parse_hint_message(hint_msg);
                        user_hints.write().await.push(hint);
                    }
                }
            }
        }
    });

    // Wait for either task to complete
    tokio::select! {
        _ = send_task => {},
        _ = recv_task => {},
    }

    // Decrement connection count
    {
        let mut dashboard = state.write().await;
        dashboard.connections = dashboard.connections.saturating_sub(1);
    }
}

/// Build state update DTO from current activity state
fn build_state_update(activity: &McpActivityState, hints: &UserHintsQueue) -> StateUpdateDto {
    let file_log = activity.recent_file_events(10_000); // Last 10 seconds

    // Build wave compass data from file events
    let wave_compass = build_wave_compass(&file_log, &activity.directories_explored);

    StateUpdateDto {
        msg_type: "state_update",
        timestamp: Utc::now().timestamp_millis(),
        mcp: McpStateDto {
            active_tool: activity.active_tool.as_ref().map(|t| t.name.clone()),
            current_operation: activity.current_operation.clone(),
            progress: activity.active_tool.as_ref().and_then(|t| t.progress),
            tools_executed: activity.tool_history.len(),
        },
        file_log,
        wave_compass,
        hints_pending: hints.unconsumed_count(),
    }
}

/// Build Wave Compass visualization data
fn build_wave_compass(file_log: &[FileEventDto], _directories: &HashSet<PathBuf>) -> WaveCompassDto {
    use std::collections::HashMap;

    // Aggregate intensity by directory
    let mut dir_intensity: HashMap<String, f32> = HashMap::new();
    for event in file_log {
        let dir = PathBuf::from(&event.path)
            .parent()
            .map(|p| p.to_string_lossy().to_string())
            .unwrap_or_default();

        // Decay intensity by age (newer = brighter)
        let intensity = 1.0 - (event.age_ms as f32 / 10_000.0).min(1.0);
        *dir_intensity.entry(dir).or_default() += intensity * 0.3;
    }

    // Convert to hot regions with pseudo-random but stable coordinates
    let hot_regions: Vec<HotRegion> = dir_intensity
        .into_iter()
        .map(|(dir, intensity)| {
            let (x, y) = path_to_coords(&dir);
            HotRegion {
                x,
                y,
                intensity: intensity.min(1.0),
                label: dir.split('/').next_back().unwrap_or(&dir).to_string(),
            }
        })
        .filter(|r| r.intensity > 0.05)
        .collect();

    // Build trail from recent file accesses
    let trail: Vec<[f32; 2]> = file_log
        .iter()
        .rev()
        .take(20)
        .map(|e| {
            let (x, y) = path_to_coords(&e.path);
            [x, y]
        })
        .collect();

    WaveCompassDto { hot_regions, trail }
}

/// Convert file path to Wave Compass coordinates using directory clustering
fn path_to_coords(path: &str) -> (f32, f32) {
    // Simple but stable hash-based positioning with directory clustering
    let parts: Vec<&str> = path.split('/').filter(|s| !s.is_empty()).collect();

    if parts.is_empty() {
        return (0.5, 0.5);
    }

    // First directory determines quadrant
    let quadrant = match parts.first().copied() {
        Some("src") => (0.0, 0.0),   // Upper-left
        Some("tests") => (0.5, 0.0), // Upper-right
        Some("docs") => (0.0, 0.5),  // Lower-left
        Some("scripts") => (0.5, 0.5), // Lower-right
        Some("examples") => (0.25, 0.25),
        _ => (0.25, 0.75),
    };

    // Sub-path determines position within quadrant
    let sub_hash = simple_hash(&parts[1..].join("/"));
    let x = quadrant.0 + (sub_hash % 100) as f32 / 200.0;
    let y = quadrant.1 + ((sub_hash / 100) % 100) as f32 / 200.0;

    (x.clamp(0.02, 0.98), y.clamp(0.02, 0.98))
}

/// Simple string hash for stable positioning
fn simple_hash(s: &str) -> u32 {
    s.bytes().fold(0u32, |acc, b| acc.wrapping_mul(31).wrapping_add(b as u32))
}

/// Parse hint message from browser
fn parse_hint_message(msg: HintMessageDto) -> UserHint {
    let content = msg.content.unwrap_or_default();

    let hint_type = match msg.hint_type.as_str() {
        "click" => HintType::Click {
            target: msg.target.unwrap_or_default(),
        },
        "text" => HintType::Text {
            message: content.clone(),
        },
        "voice" => HintType::Voice {
            transcript: msg.transcript.unwrap_or_default(),
            salience: msg.salience.unwrap_or(0.5),
        },
        _ => HintType::Text {
            message: content.clone(),
        },
    };

    UserHint {
        hint_type,
        content,
        timestamp: Utc::now(),
        consumed: false,
    }
}

// ============================================================================
// Instant serde helper (serialize as duration since UNIX_EPOCH-ish)
// ============================================================================

mod instant_serde {
    use serde::{Deserialize, Deserializer, Serialize, Serializer};
    use std::time::Instant;

    // We can't serialize Instant to absolute time, so we store "age" in ms
    // When deserializing, we approximate by subtracting from now

    pub fn serialize<S>(instant: &Instant, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        // Serialize as milliseconds ago
        let age_ms = instant.elapsed().as_millis() as u64;
        age_ms.serialize(serializer)
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Instant, D::Error>
    where
        D: Deserializer<'de>,
    {
        let age_ms = u64::deserialize(deserializer)?;
        Ok(Instant::now() - std::time::Duration::from_millis(age_ms))
    }
}
