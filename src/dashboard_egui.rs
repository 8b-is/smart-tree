// egui Dashboard - "Immediate mode, immediate collaboration!" 🎨
// Real-time dashboard for Rust Shell using egui
// "Every frame is a fresh start!" - Hue

use anyhow::Result;
use egui::{CentralPanel, Context, SidePanel, TopBottomPanel};
use egui::{Color32, Pos2, Stroke, Vec2};
use std::collections::VecDeque;
use std::sync::Arc;
use tokio::sync::RwLock;

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

    // Theme
    dark_mode: bool,
}

#[derive(PartialEq)]
enum Tab {
    Overview,
    Displays,
    Memory,
    Voice,
    Ideas,
    Debug,
}

impl Dashboard {
    pub fn new(state: Arc<DashboardState>) -> Self {
        Self {
            state,
            selected_tab: Tab::Overview,
            command_input: String::new(),
            idea_input: String::new(),
            show_raw_memory: false,
            voice_graph: VecDeque::with_capacity(100),
            dark_mode: true,
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
            let voice_active = self.state.voice_active.blocking_read();
            if *voice_active {
                ui.colored_label(Color32::GREEN, "🎤 Voice Active");
            } else {
                ui.label("🔇 Voice Inactive");
            }

            // Cast status
            let cast_status = self.state.cast_status.blocking_read();
            if let Some(target) = &cast_status.casting_to {
                ui.colored_label(Color32::BLUE, format!("📺 Casting to {}", target));
                ui.label(format!("Latency: {:.1}ms", cast_status.latency_ms));
            }

            // Memory stats
            let mem_stats = self.state.memory_usage.blocking_read();
            ui.separator();
            ui.label(format!("Memories: {}", mem_stats.total_memories));
            ui.label(format!(
                "Token Eff: {:.1}%",
                mem_stats.token_efficiency * 100.0
            ));

            // Chat sources
            ui.separator();
            ui.label("Chat Sources:");
            let chats = self.state.found_chats.blocking_read();
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
            let history = self.state.command_history.blocking_read();
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

        let displays = self.state.active_displays.blocking_read();
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
        let displays = self.state.active_displays.blocking_read();
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

        let mem_stats = self.state.memory_usage.blocking_read();

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

        let voice_active = *self.state.voice_active.blocking_read();
        let salience = *self.state.voice_salience.blocking_read();

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
        let mut ideas = self.state.ideas_buffer.blocking_write();

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
        let mut history = self.state.command_history.blocking_write();
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

        let mut ideas = self.state.ideas_buffer.blocking_write();
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
