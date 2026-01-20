// egui Dashboard - "Immediate mode, immediate collaboration!" 🎨
// Real-time dashboard for Rust Shell using egui
// "Every frame is a fresh start!" - Hue

use anyhow::Result;
use chrono::{DateTime, Utc};
use egui::{CentralPanel, Context, SidePanel, TopBottomPanel};
use egui::{Color32, Pos2, Stroke, Vec2};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::cmp::Reverse;
use std::collections::VecDeque;
use std::sync::{Arc, RwLock};
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use tokio::time::sleep;

const DEFAULT_STATUS_FEED_URL: &str = "http://127.0.0.1:8430/status/feed";
const STATUS_FEED_POLL_INTERVAL: Duration = Duration::from_secs(5);
const STATUS_FEED_ENV_KEYS: [&str; 2] = ["SMART_TREE_G8T_STATUS_FEED", "G8T_STATUS_FEED_URL"];

pub fn default_status_feed_url() -> String {
    for key in STATUS_FEED_ENV_KEYS {
        if let Ok(value) = std::env::var(key) {
            if !value.is_empty() {
                return value;
            }
        }
    }

    DEFAULT_STATUS_FEED_URL.to_string()
}

// ============================================================================
// MCP Integration Types - "The AI can see everything now!" 🔮
// ============================================================================

/// Real-time MCP activity tracking
#[derive(Clone, Debug)]
pub struct McpActivity {
    /// Current operation being performed
    pub current_operation: String,

    /// Files touched during current operation
    pub files_touched: Vec<String>,

    /// Status of the operation
    pub status: ActivityStatus,

    /// Progress percentage (0.0 to 1.0)
    pub progress: f32,

    /// Operation start time
    pub started_at: DateTime<Utc>,
}

#[derive(Clone, Debug, PartialEq)]
pub enum ActivityStatus {
    Idle,
    Thinking,  // AI is processing
    Reading,   // Reading files
    Writing,   // Writing files
    Searching, // Searching codebase
    Analyzing, // Deep analysis
    Waiting,   // Waiting for user hint
}

/// File access event for Wave Compass visualization
#[derive(Clone, Debug)]
pub struct FileAccessEvent {
    /// File path that was accessed
    pub path: String,

    /// Type of access
    pub access_type: FileAccessType,

    /// When it happened
    pub timestamp: DateTime<Utc>,

    /// Which MCP tool accessed it
    pub tool_name: String,

    /// Duration spent on this file (in ms)
    pub duration_ms: u64,
}

#[derive(Clone, Debug, PartialEq)]
pub enum FileAccessType {
    Read,
    Write,
    Search,
    Analyze,
}

/// Currently executing MCP tool
#[derive(Clone, Debug)]
pub struct ToolExecution {
    /// Name of the tool (e.g., "search", "edit", "analyze")
    pub tool_name: String,

    /// When it started
    pub started_at: DateTime<Utc>,

    /// Tool parameters (JSON-encoded)
    pub parameters: String,

    /// Current progress description
    pub progress: String,
}

/// User hint for mid-task guidance - "Nudge me gently!" 💫
#[derive(Clone, Debug)]
pub struct UserHint {
    /// Type of hint
    pub hint_type: HintType,

    /// When the hint was given
    pub timestamp: DateTime<Utc>,

    /// Whether the AI has acknowledged this hint
    pub acknowledged: bool,
}

#[derive(Clone, Debug)]
pub enum HintType {
    /// User clicked on a signature/file in Wave Compass
    Click { path: String, signature: u64 },

    /// User typed a text hint
    TextInput { text: String },

    /// User sent voice command
    Voice { transcript: String, confidence: f32 },

    /// User adjusted a parameter slider
    ParameterAdjust { param_name: String, value: f32 },
}

// ============================================================================
// G8T Status Feed Types - "Real-time repo radar" 📡
// ============================================================================

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct RepoBranchKey {
    pub owner: String,
    pub repo: String,
    pub branch: String,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq)]
pub struct RepoStatusSnapshot {
    pub push_count: u64,
    pub pull_count: u64,
    pub last_commit: Option<String>,
    pub last_activity: Option<RepoActivityKind>,
    pub updated_at: u64,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct RepoStatusUpdate {
    pub repo: RepoBranchKey,
    pub totals: RepoStatusSnapshot,
    pub event: RepoStatusEvent,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub enum RepoStatusEvent {
    Activity {
        kind: RepoActivityKind,
        commit: Option<String>,
    },
    Snapshot,
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum RepoActivityKind {
    Push,
    Pull,
}

impl Default for McpActivity {
    fn default() -> Self {
        Self {
            current_operation: "Idle".to_string(),
            files_touched: Vec::new(),
            status: ActivityStatus::Idle,
            progress: 0.0,
            started_at: Utc::now(),
        }
    }
}

/// Dashboard state shared between rust shell and GUI
pub struct DashboardState {
    /// Command history
    pub command_history: Arc<RwLock<VecDeque<CommandEntry>>>,

    /// Active displays
    pub active_displays: Arc<RwLock<Vec<DisplayInfo>>>,

    /// Voice activity state
    pub voice_active: Arc<RwLock<bool>>,
    pub voice_salience: Arc<RwLock<f64>>,

    /// Memory usage from .m8 files
    pub memory_usage: Arc<RwLock<MemoryStats>>,

    /// Chat scanner results
    pub found_chats: Arc<RwLock<Vec<ChatSource>>>,

    /// Current casting status
    pub cast_status: Arc<RwLock<CastStatus>>,

    /// Ideas buffer - where we both add ideas!
    pub ideas_buffer: Arc<RwLock<Vec<IdeaEntry>>>,

    // ========================================================================
    // MCP Integration - "Telepathic pair programming!" 🧠↔️🤖
    // ========================================================================
    /// Real-time MCP activity - what AI is doing RIGHT NOW
    pub mcp_activity: Arc<RwLock<McpActivity>>,

    /// File access log for Wave Compass lighting up
    pub file_access_log: Arc<RwLock<Vec<FileAccessEvent>>>,

    /// Currently executing MCP tool (None if idle)
    pub active_tool: Arc<RwLock<Option<ToolExecution>>>,

    /// User hints queue - nudges for the AI during execution
    pub user_hints: Arc<RwLock<VecDeque<UserHint>>>,

    /// WebSocket connection count (for status display)
    pub ws_connections: Arc<RwLock<usize>>,

    /// Cached g8t repo status feed snapshot
    pub repo_status_feed: Arc<RwLock<Vec<RepoStatusUpdate>>>,

    /// Currently configured status feed endpoint
    pub status_feed_endpoint: Arc<RwLock<String>>,
}

impl DashboardState {
    pub fn status_feed_endpoint(&self) -> String {
        self.status_feed_endpoint
            .read()
            .map(|value| value.clone())
            .unwrap_or_else(|_| DEFAULT_STATUS_FEED_URL.to_string())
    }

    pub fn set_status_feed_endpoint(&self, endpoint: String) {
        if let Ok(mut current) = self.status_feed_endpoint.write() {
            *current = endpoint;
        }
    }

    pub fn update_status_feed(&self, updates: Vec<RepoStatusUpdate>) {
        if let Ok(mut feed) = self.repo_status_feed.write() {
            *feed = updates;
        }
    }
}

#[derive(Clone)]
pub struct CommandEntry {
    pub timestamp: String,
    pub command: String,
    pub output: String,
    pub success: bool,
}

#[derive(Clone)]
pub struct DisplayInfo {
    pub name: String,
    pub display_type: String,
    pub resolution: (u16, u16),
    pub active: bool,
}

#[derive(Clone)]
pub struct MemoryStats {
    pub total_memories: usize,
    pub token_efficiency: f32,
    pub backwards_position: usize,
    pub importance_scores: Vec<(String, f32)>,
}

#[derive(Clone)]
pub struct ChatSource {
    pub platform: String,
    pub count: usize,
    pub last_updated: String,
}

#[derive(Clone)]
pub struct CastStatus {
    pub casting_to: Option<String>,
    pub content_type: String,
    pub latency_ms: f32,
}

#[derive(Clone)]
pub struct IdeaEntry {
    pub author: String, // "Hue" or "Aye"
    pub idea: String,
    pub timestamp: String,
    pub implemented: bool,
    pub priority: Priority,
}

#[derive(Clone, PartialEq)]
pub enum Priority {
    Now,   // Do it immediately!
    Soon,  // Next few commits
    Later, // When we get time
    Maybe, // Cool but not essential
}

/// The egui dashboard app
pub struct Dashboard {
    state: Arc<DashboardState>,

    // UI state
    selected_tab: Tab,
    command_input: String,
    idea_input: String,
    show_raw_memory: bool,
    voice_graph: VecDeque<f32>,

    // Wave compass for drift visualization (Omni's contribution!)
    wave_compass: crate::wave_compass::WaveCompass,

    // Theme
    dark_mode: bool,

    // G8T feed configuration
    g8t_endpoint_input: String,
}

#[derive(PartialEq)]
enum Tab {
    Overview,
    Displays,
    Memory,
    Voice,
    Ideas,
    WaveCompass, // Omni's wave drift visualizer!
    McpActivity, // Real-time AI collaboration! 🤖💫
    G8tStatus,   // g8t repo activity feed 📡
    Debug,
}

impl Dashboard {
    pub fn new(state: Arc<DashboardState>) -> Self {
        let g8t_endpoint_input = state.status_feed_endpoint();
        Self {
            state,
            selected_tab: Tab::Overview,
            command_input: String::new(),
            idea_input: String::new(),
            show_raw_memory: false,
            voice_graph: VecDeque::with_capacity(100),
            wave_compass: crate::wave_compass::WaveCompass::new(),
            dark_mode: true,
            g8t_endpoint_input,
        }
    }

    /// Main update function called each frame
    pub fn update_ui(&mut self, ctx: &Context, _frame: &mut eframe::Frame) {
        // Top panel with tabs
        TopBottomPanel::top("top_panel").show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.heading("🚀 Rust Shell Dashboard");
                ui.separator();

                ui.selectable_value(&mut self.selected_tab, Tab::Overview, "Overview");
                ui.selectable_value(&mut self.selected_tab, Tab::Displays, "Displays");
                ui.selectable_value(&mut self.selected_tab, Tab::Memory, "Memory");
                ui.selectable_value(&mut self.selected_tab, Tab::Voice, "Voice");
                ui.selectable_value(&mut self.selected_tab, Tab::Ideas, "💡 Ideas");
                ui.selectable_value(&mut self.selected_tab, Tab::WaveCompass, "🧭 Waves");
                ui.selectable_value(&mut self.selected_tab, Tab::McpActivity, "🤖 MCP Live");
                ui.selectable_value(&mut self.selected_tab, Tab::G8tStatus, "g8t Fleet");
                ui.selectable_value(&mut self.selected_tab, Tab::Debug, "Debug");

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui.button(if self.dark_mode { "☀" } else { "🌙" }).clicked() {
                        self.dark_mode = !self.dark_mode;
                    }
                });
            });
        });

        // Side panel for quick status
        SidePanel::left("side_panel").show(ctx, |ui| {
            ui.heading("Status");
            ui.separator();

            // Voice indicator
            let voice_active = self.state.voice_active.read().unwrap();
            if *voice_active {
                ui.colored_label(Color32::GREEN, "🎤 Voice Active");
            } else {
                ui.label("🔇 Voice Inactive");
            }

            // Cast status
            let cast_status = self.state.cast_status.read().unwrap();
            if let Some(target) = &cast_status.casting_to {
                ui.colored_label(Color32::BLUE, format!("📺 Casting to {}", target));
                ui.label(format!("Latency: {:.1}ms", cast_status.latency_ms));
            }

            // Memory stats
            let mem_stats = self.state.memory_usage.read().unwrap();
            ui.separator();
            ui.label(format!("Memories: {}", mem_stats.total_memories));
            ui.label(format!(
                "Token Eff: {:.1}%",
                mem_stats.token_efficiency * 100.0
            ));

            // Chat sources
            ui.separator();
            ui.label("Chat Sources:");
            let chats = self.state.found_chats.read().unwrap();
            for chat in chats.iter() {
                ui.label(format!("  {} ({})", chat.platform, chat.count));
            }
        });

        // Central panel with selected tab content
        CentralPanel::default().show(ctx, |ui| match self.selected_tab {
            Tab::Overview => self.show_overview(ui),
            Tab::Displays => self.show_displays(ui),
            Tab::Memory => self.show_memory(ui),
            Tab::Voice => self.show_voice(ui),
            Tab::Ideas => self.show_ideas(ui),
            Tab::WaveCompass => self.show_wave_compass(ui),
            Tab::McpActivity => self.show_mcp_activity(ui),
            Tab::G8tStatus => self.show_g8t_status(ui),
            Tab::Debug => self.show_debug(ui),
        });

        // Bottom panel for command input
        TopBottomPanel::bottom("bottom_panel").show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.label("Command:");
                let response = ui.text_edit_singleline(&mut self.command_input);

                if response.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)) {
                    self.execute_command();
                }

                if ui.button("Execute").clicked() {
                    self.execute_command();
                }
            });
        });
    }

    fn show_overview(&mut self, ui: &mut egui::Ui) {
        ui.heading("System Overview");
        ui.separator();

        // Command history
        ui.collapsing("Recent Commands", |ui| {
            let history = self.state.command_history.read().unwrap();
            for entry in history.iter().rev().take(10) {
                let color = if entry.success {
                    Color32::GREEN
                } else {
                    Color32::RED
                };
                ui.horizontal(|ui| {
                    ui.colored_label(color, &entry.timestamp);
                    ui.monospace(&entry.command);
                });
                if !entry.output.is_empty() {
                    ui.add_space(10.0);
                    ui.monospace(&entry.output);
                }
            }
        });

        // Active displays grid
        ui.separator();
        ui.heading("Active Displays");

        let displays = self.state.active_displays.read().unwrap();
        ui.horizontal_wrapped(|ui| {
            for display in displays.iter() {
                ui.group(|ui| {
                    ui.vertical(|ui| {
                        ui.label(&display.name);
                        ui.label(&display.display_type);
                        ui.label(format!("{}x{}", display.resolution.0, display.resolution.1));
                        if display.active {
                            ui.colored_label(Color32::GREEN, "Active");
                        }
                    });
                });
            }
        });
    }

    fn show_displays(&mut self, ui: &mut egui::Ui) {
        ui.heading("Display Management");
        ui.separator();

        if ui.button("🔍 Discover Displays").clicked() {
            // Trigger display discovery
            // This would call rust_shell.discover_displays()
        }

        ui.separator();

        // Display list with cast controls
        let displays = self.state.active_displays.read().unwrap();
        for display in displays.iter() {
            ui.group(|ui| {
                ui.horizontal(|ui| {
                    ui.label(&display.name);
                    ui.label(format!("({})", display.display_type));

                    if ui.button("Cast").clicked() {
                        // Trigger cast to this display
                    }

                    if display.active {
                        if ui.button("Stop").clicked() {
                            // Stop casting
                        }
                    }
                });
            });
        }
    }

    fn show_memory(&mut self, ui: &mut egui::Ui) {
        ui.heading("Memory System (.m8)");
        ui.separator();

        let mem_stats = self.state.memory_usage.read().unwrap();

        // Memory visualization
        ui.label(format!("Total Memories: {}", mem_stats.total_memories));
        ui.label(format!(
            "Backwards Position: {}",
            mem_stats.backwards_position
        ));

        ui.separator();
        ui.checkbox(&mut self.show_raw_memory, "Show Raw Memory");

        if self.show_raw_memory {
            ui.group(|ui| {
                ui.monospace("0x80: node_modules");
                ui.monospace("0x81: .git");
                ui.monospace("0x82: target");
                ui.monospace("0x91: .rs");
                ui.monospace("0xFFFE: Claude");
            });
        }

        // Importance scores
        ui.separator();
        ui.heading("Important Memories");
        for (memory, score) in mem_stats.importance_scores.iter().take(5) {
            ui.horizontal(|ui| {
                // Draw importance bar
                let bar_width = 200.0 * score;
                let (rect, _response) =
                    ui.allocate_exact_size(Vec2::new(bar_width, 20.0), egui::Sense::hover());
                ui.painter()
                    .rect_filled(rect, 0.0, Color32::from_rgb(100, 200, 100));

                ui.label(memory);
                ui.label(format!("{:.1}%", score * 100.0));
            });
        }
    }

    fn show_voice(&mut self, ui: &mut egui::Ui) {
        ui.heading("Voice Activity (Marine Algorithm)");
        ui.separator();

        let voice_active = *self.state.voice_active.read().unwrap();
        let salience = *self.state.voice_salience.read().unwrap();

        // Voice status
        ui.horizontal(|ui| {
            if voice_active {
                ui.colored_label(Color32::GREEN, "🎤 VOICE DETECTED");
            } else {
                ui.label("🔇 No Voice");
            }

            ui.label(format!("Salience: {:.2}", salience));
        });

        // Voice activity graph
        ui.separator();
        ui.label("Voice Activity Graph:");

        // Update graph data
        self.voice_graph.push_back(salience as f32);
        if self.voice_graph.len() > 100 {
            self.voice_graph.pop_front();
        }

        // Draw graph
        let graph_size = Vec2::new(400.0, 100.0);
        let (rect, _response) = ui.allocate_exact_size(graph_size, egui::Sense::hover());

        let painter = ui.painter();
        painter.rect_filled(rect, 0.0, Color32::from_gray(20));

        if self.voice_graph.len() > 1 {
            let points: Vec<Pos2> = self
                .voice_graph
                .iter()
                .enumerate()
                .map(|(i, &val)| {
                    let x = rect.left() + (i as f32 / 100.0) * rect.width();
                    let y = rect.bottom() - val * rect.height();
                    Pos2::new(x, y)
                })
                .collect();

            for window in points.windows(2) {
                painter.line_segment([window[0], window[1]], Stroke::new(2.0, Color32::GREEN));
            }
        }

        // Threshold line
        let threshold_y = rect.bottom() - 0.5 * rect.height();
        painter.line_segment(
            [
                Pos2::new(rect.left(), threshold_y),
                Pos2::new(rect.right(), threshold_y),
            ],
            Stroke::new(1.0, Color32::RED),
        );
    }

    fn show_ideas(&mut self, ui: &mut egui::Ui) {
        ui.heading("💡 Ideas Buffer - Collaborative Brainstorming!");
        ui.separator();

        // Input for new ideas
        ui.horizontal(|ui| {
            ui.label("New Idea:");
            ui.text_edit_singleline(&mut self.idea_input);

            if ui.button("Add Hue's Idea").clicked() {
                self.add_idea("Hue", Priority::Soon);
            }
            if ui.button("Add Aye's Idea").clicked() {
                self.add_idea("Aye", Priority::Soon);
            }
        });

        ui.separator();

        // Ideas list
        let mut ideas = self.state.ideas_buffer.write().unwrap();

        // Group by priority
        for priority in [
            Priority::Now,
            Priority::Soon,
            Priority::Later,
            Priority::Maybe,
        ] {
            let priority_ideas: Vec<_> = ideas
                .iter()
                .filter(|i| i.priority == priority)
                .cloned()
                .collect();

            if !priority_ideas.is_empty() {
                let header = match priority {
                    Priority::Now => "🔥 DO IT NOW!",
                    Priority::Soon => "⚡ Soon",
                    Priority::Later => "📅 Later",
                    Priority::Maybe => "🤔 Maybe",
                };

                ui.collapsing(header, |ui| {
                    for idea in priority_ideas {
                        ui.group(|ui| {
                            ui.horizontal(|ui| {
                                let author_color = if idea.author == "Hue" {
                                    Color32::from_rgb(100, 150, 255)
                                } else {
                                    Color32::from_rgb(255, 150, 100)
                                };

                                ui.colored_label(author_color, &idea.author);
                                ui.label(&idea.timestamp);

                                if idea.implemented {
                                    ui.colored_label(Color32::GREEN, "✓");
                                }
                            });

                            ui.label(&idea.idea);

                            ui.horizontal(|ui| {
                                if !idea.implemented {
                                    if ui.button("Mark Done").clicked() {
                                        // Find and mark as implemented
                                        if let Some(idx) =
                                            ideas.iter().position(|i| i.idea == idea.idea)
                                        {
                                            ideas[idx].implemented = true;
                                        }
                                    }
                                }
                            });
                        });
                    }
                });
            }
        }
    }

    fn show_wave_compass(&mut self, ui: &mut egui::Ui) {
        // Gather current directory signatures (would come from actual scanning)
        // For now, let's create some example signatures from our semantic categories
        let signatures = vec![
            crate::wave_compass::WaveSig::from_quantum(
                "src".into(),
                &crate::quantum_wave_signature::QuantumWaveSignature::from_raw(0x73A9E2F5),
            ),
            crate::wave_compass::WaveSig::from_quantum(
                "tests".into(),
                &crate::quantum_wave_signature::QuantumWaveSignature::from_raw(0x9F2E6B31),
            ),
            crate::wave_compass::WaveSig::from_quantum(
                "mcp".into(),
                &crate::quantum_wave_signature::QuantumWaveSignature::from_raw(0x2C7DB5A3),
            ),
            crate::wave_compass::WaveSig::from_quantum(
                "mem8".into(),
                &crate::quantum_wave_signature::QuantumWaveSignature::from_raw(0xE4739AC2),
            ),
            crate::wave_compass::WaveSig::from_quantum(
                "formatters".into(),
                &crate::quantum_wave_signature::QuantumWaveSignature::from_raw(0xA7E2C94D),
            ),
            crate::wave_compass::WaveSig::from_quantum(
                "generated".into(),
                &crate::quantum_wave_signature::QuantumWaveSignature::from_raw(0xD2B847A6),
            ),
        ];

        // Update compass with current signatures
        self.wave_compass.update(signatures);

        // Show the compass
        self.wave_compass.show(ui);

        // Additional controls
        ui.separator();
        ui.horizontal(|ui| {
            if ui.button("🔄 Refresh").clicked() {
                // Would trigger a rescan of directories
            }

            ui.label("Resonance Threshold:");
            let mut threshold = 0.5_f32;
            ui.add(egui::Slider::new(&mut threshold, 0.0..=1.0));

            if ui.button("📸 Snapshot").clicked() {
                // Would save current state to .m8 file
            }
        });

        // Show detected resonances
        ui.separator();
        ui.heading("Detected Resonances");

        let resonances = crate::wave_compass::find_resonances(&self.wave_compass.signatures, 0.5);
        for res in resonances {
            let sig1 = &self.wave_compass.signatures[res.sig1_idx];
            let sig2 = &self.wave_compass.signatures[res.sig2_idx];

            let emoji = if res.is_harmonic { "🎵" } else { "🌊" };
            ui.label(format!(
                "{} {} ↔ {} ({:.0}% resonance)",
                emoji,
                sig1.name,
                sig2.name,
                res.strength * 100.0
            ));
        }
    }

    fn show_mcp_activity(&mut self, ui: &mut egui::Ui) {
        ui.heading("🤖 Real-Time AI Collaboration - \"I can see you seeing me!\"");
        ui.separator();

        let mcp_activity = self.state.mcp_activity.read().unwrap();
        let active_tool = self.state.active_tool.read().unwrap();
        let file_log = self.state.file_access_log.read().unwrap();
        let hints = self.state.user_hints.read().unwrap();

        // Current AI Status - Big and Bold!
        ui.group(|ui| {
            ui.vertical(|ui| {
                // Status with color coding
                let (status_text, status_color) = match mcp_activity.status {
                    ActivityStatus::Idle => ("💤 Idle", Color32::GRAY),
                    ActivityStatus::Thinking => {
                        ("🧠 Thinking...", Color32::from_rgb(150, 100, 255))
                    }
                    ActivityStatus::Reading => {
                        ("📖 Reading Files", Color32::from_rgb(100, 200, 255))
                    }
                    ActivityStatus::Writing => {
                        ("✍️ Writing Code", Color32::from_rgb(255, 200, 100))
                    }
                    ActivityStatus::Searching => {
                        ("🔍 Searching...", Color32::from_rgb(100, 255, 200))
                    }
                    ActivityStatus::Analyzing => {
                        ("🔬 Deep Analysis", Color32::from_rgb(255, 100, 200))
                    }
                    ActivityStatus::Waiting => {
                        ("⏸️ Waiting for You", Color32::from_rgb(255, 255, 100))
                    }
                };

                ui.horizontal(|ui| {
                    ui.heading(status_text);
                    ui.colored_label(status_color, "●"); // Status indicator
                });

                // Current operation
                if !mcp_activity.current_operation.is_empty() {
                    ui.label(format!("Operation: {}", mcp_activity.current_operation));
                }

                // Progress bar
                if mcp_activity.progress > 0.0 {
                    let progress_bar = egui::ProgressBar::new(mcp_activity.progress)
                        .text(format!("{:.0}%", mcp_activity.progress * 100.0));
                    ui.add(progress_bar);
                }

                // Duration
                let duration = Utc::now().signed_duration_since(mcp_activity.started_at);
                ui.label(format!(
                    "Duration: {}.{}s",
                    duration.num_seconds(),
                    duration.num_milliseconds() % 1000
                ));

                // Files touched in current operation
                if !mcp_activity.files_touched.is_empty() {
                    ui.label(format!(
                        "Files touched: {}",
                        mcp_activity.files_touched.len()
                    ));
                    ui.collapsing("Show files", |ui| {
                        for file in &mcp_activity.files_touched {
                            ui.monospace(file);
                        }
                    });
                }
            });
        });

        ui.separator();

        // Active Tool Details
        if let Some(tool) = active_tool.as_ref() {
            ui.heading("🔧 Active Tool");
            ui.group(|ui| {
                ui.label(format!("Tool: {}", tool.tool_name));
                ui.label(format!("Progress: {}", tool.progress));

                ui.collapsing("Parameters", |ui| {
                    ui.monospace(&tool.parameters);
                });

                let tool_duration = Utc::now().signed_duration_since(tool.started_at);
                ui.label(format!(
                    "Running for: {}.{}s",
                    tool_duration.num_seconds(),
                    tool_duration.num_milliseconds() % 1000
                ));
            });
            ui.separator();
        }

        // File Access Log with Timeline
        ui.heading("📂 File Access Timeline");
        ui.label(format!("Total accesses: {}", file_log.len()));

        egui::ScrollArea::vertical()
            .max_height(200.0)
            .show(ui, |ui| {
                // Show recent file accesses (last 50)
                for event in file_log.iter().rev().take(50) {
                    let (icon, color) = match event.access_type {
                        FileAccessType::Read => ("📖", Color32::from_rgb(100, 200, 255)),
                        FileAccessType::Write => ("✍️", Color32::from_rgb(255, 200, 100)),
                        FileAccessType::Search => ("🔍", Color32::from_rgb(100, 255, 200)),
                        FileAccessType::Analyze => ("🔬", Color32::from_rgb(255, 100, 200)),
                    };

                    ui.horizontal(|ui| {
                        ui.colored_label(color, icon);
                        ui.label(event.timestamp.format("%H:%M:%S%.3f").to_string());
                        ui.monospace(&event.path);
                        ui.label(format!("{}ms", event.duration_ms));
                        ui.label(format!("({})", event.tool_name));
                    });
                }
            });

        ui.separator();

        // User Hints Section - "Nudge the AI!"
        ui.heading("💫 Your Hints to AI");
        ui.label("Send nudges without stopping the AI!");

        ui.horizontal(|ui| {
            if ui.button("👆 Click Hint").clicked() {
                // This would be triggered when user clicks on Wave Compass
                ui.label("Click on Wave Compass signatures to send location hints!");
            }

            if ui.button("💬 Text Hint").clicked() {
                // Open text input dialog
            }

            if ui.button("🎤 Voice Hint").clicked() {
                // Trigger voice input
            }
        });

        // Show pending hints
        if !hints.is_empty() {
            ui.separator();
            ui.label(format!("Pending hints: {}", hints.len()));

            for hint in hints.iter().rev().take(5) {
                let (icon, hint_text) = match &hint.hint_type {
                    HintType::Click { path, signature } => {
                        ("👆", format!("Clicked: {} (sig: {:X})", path, signature))
                    }
                    HintType::TextInput { text } => ("💬", format!("Text: {}", text)),
                    HintType::Voice {
                        transcript,
                        confidence,
                    } => (
                        "🎤",
                        format!("Voice: {} ({:.0}%)", transcript, confidence * 100.0),
                    ),
                    HintType::ParameterAdjust { param_name, value } => {
                        ("🎚️", format!("Adjust {}: {:.2}", param_name, value))
                    }
                };

                ui.horizontal(|ui| {
                    ui.label(icon);
                    ui.label(hint.timestamp.format("%H:%M:%S").to_string());
                    ui.label(hint_text);

                    if hint.acknowledged {
                        ui.colored_label(Color32::GREEN, "✓ Seen");
                    } else {
                        ui.colored_label(Color32::YELLOW, "⏳ Pending");
                    }
                });
            }
        }

        ui.separator();

        // WebSocket Connection Status
        let ws_count = *self.state.ws_connections.read().unwrap();
        ui.horizontal(|ui| {
            ui.label("WebSocket connections:");
            if ws_count > 0 {
                ui.colored_label(Color32::GREEN, format!("{} active", ws_count));
            } else {
                ui.colored_label(Color32::RED, "Disconnected");
            }
        });
    }

    fn show_g8t_status(&mut self, ui: &mut egui::Ui) {
        ui.heading("g8t Fleet Activity");
        ui.separator();

        let mut apply_endpoint = false;

        ui.horizontal(|ui| {
            ui.label("Feed URL:");
            if self.g8t_endpoint_input.is_empty() {
                self.g8t_endpoint_input = self.state.status_feed_endpoint();
            }

            let response = ui.text_edit_singleline(&mut self.g8t_endpoint_input);
            if response.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)) {
                apply_endpoint = true;
            }

            if ui.button("Apply").clicked() {
                apply_endpoint = true;
            }

            if ui.button("Refresh").clicked() {
                let state_clone = self.state.clone();
                tokio::spawn(async move {
                    if let Err(err) = fetch_status_feed_once(state_clone).await {
                        eprintln!("⚠️ Failed to refresh g8t status feed: {}", err);
                    }
                });
            }
        });

        if apply_endpoint {
            let endpoint = self.g8t_endpoint_input.trim().to_string();
            if !endpoint.is_empty() {
                self.state.set_status_feed_endpoint(endpoint.clone());
                println!("🌐 Updated g8t status endpoint to {}", endpoint);
            }
        }

        ui.separator();

        let mut feed = self
            .state
            .repo_status_feed
            .read()
            .map(|entries| entries.clone())
            .unwrap_or_default();

        if feed.is_empty() {
            ui.label("No g8t activity yet. Push or pull via g8t to light this up!");
            return;
        }

        feed.sort_by_key(|update| Reverse(update.totals.updated_at));

        let total_repos = feed.len();
        let total_pushes: u64 = feed.iter().map(|u| u.totals.push_count).sum();
        let total_pulls: u64 = feed.iter().map(|u| u.totals.pull_count).sum();

        ui.horizontal(|ui| {
            ui.label(format!("Repos tracked: {}", total_repos));
            ui.separator();
            ui.label(format!("Pushes: {}", total_pushes));
            ui.separator();
            ui.label(format!("Pulls: {}", total_pulls));
            if let Some(latest) = feed.first() {
                ui.separator();
                ui.label(format!(
                    "Last activity: {}",
                    format_relative_time(latest.totals.updated_at)
                ));
            }
        });

        ui.separator();

        egui::ScrollArea::vertical()
            .id_salt("g8t_status_scroll")
            .show(ui, |ui| {
                egui::Grid::new("g8t_status_grid")
                    .striped(true)
                    .num_columns(6)
                    .show(ui, |ui| {
                        ui.strong("Repository");
                        ui.strong("Branch");
                        ui.strong("Push/Pull");
                        ui.strong("Last Commit");
                        ui.strong("Activity");
                        ui.strong("Updated");
                        ui.end_row();

                        for update in feed.iter().take(50) {
                            let repo_name = format!("{}/{}", update.repo.owner, update.repo.repo);
                            ui.label(repo_name);
                            ui.label(update.repo.branch.clone());
                            ui.label(format!(
                                "{}/{}",
                                update.totals.push_count, update.totals.pull_count
                            ));
                            ui.label(short_commit(&update.totals.last_commit));
                            let (icon, label, color) =
                                summarize_activity(update.totals.last_activity);
                            ui.colored_label(color, format!("{} {}", icon, label));
                            ui.label(format_relative_time(update.totals.updated_at));
                            ui.end_row();
                        }
                    });
            });
    }

    fn show_debug(&mut self, ui: &mut egui::Ui) {
        ui.heading("Debug Information");
        ui.separator();

        ui.monospace(format!(
            "FPS: {:.1}",
            ui.ctx().input(|i| i.stable_dt).recip()
        ));
        ui.monospace(format!("Time: {:.3}s", ui.ctx().input(|i| i.time)));

        ui.separator();
        ui.collapsing("Memory Layout", |ui| {
            ui.monospace("DashboardState size: {} bytes");
            ui.monospace("Ideas buffer capacity: {}");
        });
    }

    fn execute_command(&mut self) {
        if self.command_input.is_empty() {
            return;
        }

        // Add to history
        let mut history = self.state.command_history.write().unwrap();
        history.push_back(CommandEntry {
            timestamp: chrono::Local::now().format("%H:%M:%S").to_string(),
            command: self.command_input.clone(),
            output: String::new(), // Would be filled by actual execution
            success: true,
        });

        // Keep history limited
        if history.len() > 100 {
            history.pop_front();
        }

        self.command_input.clear();
    }

    fn add_idea(&mut self, author: &str, priority: Priority) {
        if self.idea_input.is_empty() {
            return;
        }

        let mut ideas = self.state.ideas_buffer.write().unwrap();
        ideas.push(IdeaEntry {
            author: author.to_string(),
            idea: self.idea_input.clone(),
            timestamp: chrono::Local::now().format("%H:%M:%S").to_string(),
            implemented: false,
            priority,
        });

        self.idea_input.clear();
    }
}

/// Start the egui dashboard
pub async fn start_dashboard(state: Arc<DashboardState>) -> Result<()> {
    // Start background poller for g8t status feed
    tokio::spawn(poll_status_feed(state.clone()));

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([800.0, 600.0])
            .with_title("Rust Shell Dashboard"),
        ..Default::default()
    };

    eframe::run_native(
        "Rust Shell Dashboard",
        options,
        Box::new(|cc| {
            // Configure fonts and style
            let mut style = (*cc.egui_ctx.style()).clone();
            style.visuals.dark_mode = true;
            cc.egui_ctx.set_style(style);

            Ok(Box::new(Dashboard::new(state)))
        }),
    )
    .map_err(|e| anyhow::anyhow!("Failed to start dashboard: {}", e))
}

impl eframe::App for Dashboard {
    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        self.update_ui(ctx, frame);

        // Request repaint for animations
        ctx.request_repaint();
    }
}

async fn poll_status_feed(state: Arc<DashboardState>) {
    loop {
        if let Err(err) = fetch_status_feed_once(state.clone()).await {
            eprintln!("⚠️ Status feed polling error: {}", err);
        }

        sleep(STATUS_FEED_POLL_INTERVAL).await;
    }
}

async fn fetch_status_feed_once(state: Arc<DashboardState>) -> Result<()> {
    let endpoint = state.status_feed_endpoint();
    let client = Client::new();

    let response = client.get(&endpoint).send().await?;
    if !response.status().is_success() {
        anyhow::bail!(
            "Status feed request failed with {} for {}",
            response.status(),
            endpoint
        );
    }

    let updates: Vec<RepoStatusUpdate> = response.json().await?;
    state.update_status_feed(updates);

    Ok(())
}

fn summarize_activity(activity: Option<RepoActivityKind>) -> (&'static str, &'static str, Color32) {
    match activity {
        Some(RepoActivityKind::Push) => ("⬆", "Push", Color32::from_rgb(120, 220, 120)),
        Some(RepoActivityKind::Pull) => ("⬇", "Pull", Color32::from_rgb(120, 180, 255)),
        None => ("…", "Idle", Color32::GRAY),
    }
}

fn short_commit(commit: &Option<String>) -> String {
    commit
        .as_ref()
        .map(|hash| {
            if hash.len() <= 8 {
                hash.clone()
            } else {
                hash.chars().take(8).collect::<String>()
            }
        })
        .unwrap_or_else(|| "—".to_string())
}

fn format_relative_time(timestamp_ms: u64) -> String {
    if timestamp_ms == 0 {
        return "never".to_string();
    }

    let now = current_millis();
    if timestamp_ms >= now {
        return "just now".to_string();
    }

    let diff = now - timestamp_ms;
    let seconds = diff / 1000;
    if seconds < 5 {
        return "just now".to_string();
    }
    if seconds < 60 {
        return format!("{}s ago", seconds);
    }

    let minutes = seconds / 60;
    if minutes < 60 {
        return format!("{}m ago", minutes);
    }

    let hours = minutes / 60;
    if hours < 24 {
        return format!("{}h ago", hours);
    }

    let days = hours / 24;
    format!("{}d ago", days)
}

fn current_millis() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|dur| dur.as_millis() as u64)
        .unwrap_or_default()
}
