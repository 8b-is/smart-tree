//! BUI Dashboard for Smart Tree
//!
//! Main dashboard view combining tree view, chat, and monitoring.

#[cfg(feature = "bui")]
use crate::bui::components::{ChatPanel, CommandPalette, StatusBar, TreeView};
#[cfg(feature = "bui")]
use crate::bui::theme::BuiTheme;
#[cfg(feature = "bui")]
use blenderui::prelude::*;

#[cfg(feature = "bui")]
pub struct BuiDashboard {
    pub tree_view: TreeView,
    pub chat_panel: ChatPanel,
    pub status_bar: StatusBar,
    pub command_palette: CommandPalette,
    pub theme: BuiTheme,
    pub scan_results: Vec<ScanResult>,
    pub active_tab: DashboardTab,
}

#[cfg(feature = "bui")]
#[derive(Clone, Copy, PartialEq)]
pub enum DashboardTab {
    Tree,
    Scan,
    Chat,
    Monitor,
    Settings,
}

#[cfg(feature = "bui")]
#[derive(Clone)]
pub struct ScanResult {
    pub path: std::path::PathBuf,
    pub size: u64,
    pub file_type: String,
    pub interest_score: f32,
    pub tags: Vec<String>,
}

#[cfg(feature = "bui")]
impl BuiDashboard {
    pub fn new(theme: BuiTheme) -> Self {
        Self {
            tree_view: TreeView::new(),
            chat_panel: ChatPanel::new(),
            status_bar: StatusBar::new(),
            command_palette: CommandPalette::new(),
            theme,
            scan_results: Vec::new(),
            active_tab: DashboardTab::Tree,
        }
    }

    pub fn set_scan_results(&mut self, results: Vec<ScanResult>) {
        self.scan_results = results;
    }

    pub fn set_active_tab(&mut self, tab: DashboardTab) {
        self.active_tab = tab;
    }

    #[cfg(feature = "bui")]
    pub fn render(&self, cx: &mut Context) -> impl Element {
        VStack::new()
            .child(self.render_header(cx))
            .child(self.render_tabs(cx))
            .child(self.render_content(cx))
            .child(self.status_bar.render(cx))
            .style(|s| s.flex(1).background(self.theme.background_color()))
    }

    #[cfg(feature = "bui")]
    fn render_header(&self, cx: &mut Context) -> impl Element {
        HStack::new()
            .child(
                Text::new("Smart Tree")
                    .style(|s| s.font_size(24).bold().color(Color::WHITE).margin(20)),
            )
            .child(Space::new().flex(1))
            .child(
                HStack::new()
                    .child(
                        Button::new("Scan")
                            .on_click(|| { /* trigger scan */ })
                            .style(|s| s.padding(8..16).margin_right(10)),
                    )
                    .child(
                        Button::new("Settings")
                            .on_click(|| { /* open settings */ })
                            .style(|s| s.padding(8..16).margin_right(10)),
                    )
                    .child(
                        Button::new("⌘K")
                            .on_click(|| { /* toggle command palette */ })
                            .style(|s| s.padding(8..16)),
                    ),
            )
            .style(|s| {
                s.padding(10..20)
                    .background(Color::from_hex("#1a1a1a"))
                    .border_bottom(1, Color::from_hex("#333"))
            })
    }

    #[cfg(feature = "bui")]
    fn render_tabs(&self, cx: &mut Context) -> impl Element {
        HStack::new()
            .children(
                [
                    (DashboardTab::Tree, "🌳 Tree"),
                    (DashboardTab::Scan, "🔍 Scan"),
                    (DashboardTab::Chat, "💬 Chat"),
                    (DashboardTab::Monitor, "📊 Monitor"),
                    (DashboardTab::Settings, "⚙️ Settings"),
                ]
                .into_iter()
                .map(|(tab, label)| {
                    let is_active = self.active_tab == tab;
                    Button::new(label)
                        .style(move |s| {
                            s.padding(8..16)
                                .rounded(4)
                                .background(if is_active {
                                    Color::from_hex("#27ae60")
                                } else {
                                    Color::from_hex("#2c3e50")
                                })
                                .color(if is_active {
                                    Color::WHITE
                                } else {
                                    Color::WHITE
                                })
                        })
                        .on_click(move || { /* set active tab */ })
                }),
            )
            .style(|s| {
                s.padding(0..20)
                    .background(Color::from_hex("#1a1a1a"))
                    .border_bottom(1, Color::from_hex("#333"))
            })
    }

    #[cfg(feature = "bui")]
    fn render_content(&self, cx: &mut Context) -> impl Element {
        match self.active_tab {
            DashboardTab::Tree => self.tree_view.render(cx).into_element(),
            DashboardTab::Scan => self.render_scan_tab(cx),
            DashboardTab::Chat => self.chat_panel.render(cx).into_element(),
            DashboardTab::Monitor => self.render_monitor_tab(cx),
            DashboardTab::Settings => self.render_settings_tab(cx),
        }
    }

    #[cfg(feature = "bui")]
    fn render_scan_tab(&self, cx: &mut Context) -> impl Element {
        VStack::new()
            .child(Text::new("Directory Scan Results").style(|s| s.font_size(18).bold().margin(20)))
            .child(
                Table::new()
                    .header(["Path", "Size", "Type", "Score", "Tags"])
                    .rows(self.scan_results.iter().map(|r| {
                        vec![
                            r.path.display().to_string(),
                            format_bytes(r.size),
                            r.file_type.clone(),
                            format!("{:.2}", r.interest_score),
                            r.tags.join(", "),
                        ]
                    }))
                    .style(|s| s.flex(1).margin(20)),
            )
            .style(|s| s.flex(1))
    }

    #[cfg(feature = "bui")]
    fn render_monitor_tab(&self, cx: &mut Context) -> impl Element {
        VStack::new()
            .child(Text::new("System Monitor").style(|s| s.font_size(18).bold().margin(20)))
            .child(
                Grid::new()
                    .columns(3)
                    .child(StatCard::new("CPU", "45%", Color::BLUE))
                    .child(StatCard::new("Memory", "2.3 GB / 16 GB", Color::BLUE))
                    .child(StatCard::new("Disk I/O", "12 MB/s", Color::BLUE))
                    .child(StatCard::new(
                        "Network",
                        "1.2 MB/s ↑ / 0.8 MB/s ↓",
                        Color::BLUE,
                    ))
                    .child(StatCard::new("GPU", "N/A", Color::GRAY))
                    .child(StatCard::new("Foken Credits", "1,234 FKN", Color::GOLD))
                    .style(|s| s.margin(20)),
            )
            .style(|s| s.flex(1))
    }

    #[cfg(feature = "bui")]
    fn render_settings_tab(&self, cx: &mut Context) -> impl Element {
        VStack::new()
            .child(Text::new("Settings").style(|s| s.font_size(18).bold().margin(20)))
            .child(
                VStack::new()
                    .child(SettingRow::new("Theme", "Dark", || { /* toggle theme */ }))
                    .child(SettingRow::new("Auto-scan", "Enabled", || { /* toggle */ }))
                    .child(SettingRow::new("Auto-sync", "Disabled", || { /* toggle */ }))
                    .child(SettingRow::new(
                        "Foken Auto-claim",
                        "Enabled",
                        || { /* toggle */ },
                    ))
                    .style(|s| s.margin(20)),
            )
            .style(|s| s.flex(1))
    }
}

#[cfg(feature = "bui")]
struct StatCard {
    label: String,
    value: String,
    color: Color,
}

#[cfg(feature = "bui")]
impl StatCard {
    fn new(label: &str, value: &str, color: Color) -> impl Element {
        VStack::new()
            .child(Text::new(label).style(|s| s.font_size(12).color(Color::GRAY)))
            .child(Text::new(value).style(|s| s.font_size(24).bold().color(color)))
            .style(|s| {
                s.padding(20)
                    .background(Color::from_hex("#1e1e1e"))
                    .border(1, Color::from_hex("#333"))
                    .rounded(8)
                    .flex(1)
            })
    }
}

#[cfg(feature = "bui")]
struct SettingRow {
    label: String,
    value: String,
    on_click: Box<dyn Fn()>,
}

#[cfg(feature = "bui")]
impl SettingRow {
    fn new(label: &str, value: &str, on_click: impl Fn() + 'static) -> impl Element {
        HStack::new()
            .child(Text::new(label).style(|s| s.flex(1).color(Color::WHITE)))
            .child(Text::new(value).style(|s| s.color(Color::GREEN)))
            .child(Button::new("✏️").on_click(Box::new(on_click)))
            .style(|s| {
                s.padding(10)
                    .background(Color::from_hex("#1e1e1e"))
                    .border_bottom(1, Color::from_hex("#333"))
            })
    }
}

fn format_bytes(bytes: u64) -> String {
    const UNITS: &[&str] = &["B", "KB", "MB", "GB", "TB"];
    let mut size = bytes as f64;
    let mut unit = 0;
    while size >= 1024.0 && unit < UNITS.len() - 1 {
        size /= 1024.0;
        unit += 1;
    }
    format!("{:.1} {}", size, UNITS[unit])
}
