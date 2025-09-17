// Spicy TUI Mode - "Making directory browsing cyberpunk cool!" 🌶️
// Inspired by spicy-fzf's beautiful terminal aesthetic

use anyhow::Result;
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyModifiers},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style, Stylize},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph, Wrap},
    Frame, Terminal,
};
use std::path::Path;
use std::{
    fs,
    io::{self, BufRead, BufReader},
    path::PathBuf,
    time::{Duration, SystemTime},
};
use syntect::{highlighting::ThemeSet, parsing::SyntaxSet};

use crate::scanner::{FileNode, Scanner, ScannerConfig};

// Color scheme inspired by spicy-fzf
const SPICY_GREEN: Color = Color::Rgb(0, 255, 0);
const SPICY_DARK_GREEN: Color = Color::Rgb(0, 128, 0);
const SPICY_YELLOW: Color = Color::Rgb(255, 255, 0);
const SPICY_CYAN: Color = Color::Rgb(0, 255, 255);
const SPICY_MAGENTA: Color = Color::Rgb(255, 0, 255);
const SPICY_ORANGE: Color = Color::Rgb(255, 165, 0);
const SPICY_BG: Color = Color::Rgb(10, 10, 10);
const SPICY_BORDER: Color = Color::Rgb(0, 100, 0);

pub struct SpicyTui {
    terminal: Option<Terminal<CrosstermBackend<io::Stdout>>>,
    current_path: PathBuf,
    file_nodes: Vec<FileNode>,
    selected_index: usize,
    list_state: ListState,
    preview_content: Option<String>,
    search_query: String,
    search_mode: bool,
    filtered_indices: Vec<usize>,
    scroll_offset: u16,
    preview_scroll: u16,
    show_hidden: bool,
    show_help: bool,
    syntax_set: SyntaxSet,
    theme_set: ThemeSet,
    status_message: Option<(String, SystemTime)>,
}

impl SpicyTui {
    pub fn new(path: PathBuf) -> Result<Self> {
        // Setup terminal
        enable_raw_mode()?;
        let mut stdout = io::stdout();
        execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
        let backend = CrosstermBackend::new(stdout);
        let terminal = Terminal::new(backend)?;

        // Load syntax highlighting
        let syntax_set = SyntaxSet::load_defaults_newlines();
        let theme_set = ThemeSet::load_defaults();

        let mut app = Self {
            terminal: Some(terminal),
            current_path: path.clone(),
            file_nodes: Vec::new(),
            selected_index: 0,
            list_state: ListState::default(),
            preview_content: None,
            search_query: String::new(),
            search_mode: false,
            filtered_indices: Vec::new(),
            scroll_offset: 0,
            preview_scroll: 0,
            show_hidden: false,
            show_help: false,
            syntax_set,
            theme_set,
            status_message: None,
        };

        app.refresh_directory()?;
        Ok(app)
    }

    fn refresh_directory(&mut self) -> Result<()> {
        let config = ScannerConfig {
            max_depth: 1,
            show_hidden: self.show_hidden,
            respect_gitignore: true,
            use_default_ignores: true,
            ..Default::default()
        };

        let scanner = Scanner::new(&self.current_path, config)?;
        let (nodes, _) = scanner.scan()?;
        self.file_nodes = nodes;
        self.filtered_indices = (0..self.file_nodes.len()).collect();
        self.selected_index = 0;
        self.list_state.select(Some(0));
        self.update_preview()?;
        Ok(())
    }

    fn update_preview(&mut self) -> Result<()> {
        if self.filtered_indices.is_empty() {
            self.preview_content = None;
            return Ok(());
        }

        let actual_index = self.filtered_indices[self.selected_index];
        let node = &self.file_nodes[actual_index];
        let path = &node.path;

        if path.is_file() {
            // Try to read file preview
            match fs::metadata(path) {
                Ok(meta) if meta.len() > 1_000_000 => {
                    self.preview_content = Some(format!(
                        "📁 File too large for preview\nSize: {:.2} MB",
                        meta.len() as f64 / 1_048_576.0
                    ));
                }
                Ok(_) => {
                    // Read first 100 lines
                    if let Ok(file) = fs::File::open(path) {
                        let reader = BufReader::new(file);
                        let lines: Vec<String> =
                            reader.lines().take(100).filter_map(|l| l.ok()).collect();
                        self.preview_content = Some(lines.join("\n"));
                    } else {
                        self.preview_content = Some("⚠️ Cannot read file".to_string());
                    }
                }
                Err(_) => {
                    self.preview_content = Some("❌ Permission denied".to_string());
                }
            }
        } else if path.is_dir() {
            // Show directory info
            let config = ScannerConfig {
                max_depth: 1,
                show_hidden: self.show_hidden,
                respect_gitignore: true,
                use_default_ignores: true,
                ..Default::default()
            };

            if let Ok(scanner) = Scanner::new(path, config) {
                if let Ok((children, stats)) = scanner.scan() {
                    let info = format!(
                    "📂 Directory\n\nFiles: {}\nDirectories: {}\nTotal Size: {:.2} MB\n\n--- Contents ---\n{}",
                    stats.total_files,
                    stats.total_dirs,
                    stats.total_size as f64 / 1_048_576.0,
                    children.iter()
                        .take(20)
                        .map(|n| format!("  {} {}",
                            if n.path.is_dir() { "📁" } else { "📄" },
                            n.path.file_name().unwrap_or_default().to_string_lossy()
                        ))
                        .collect::<Vec<_>>()
                        .join("\n")
                );
                    self.preview_content = Some(info);
                }
            }
        }

        Ok(())
    }

    fn apply_search_filter(&mut self) {
        if self.search_query.is_empty() {
            self.filtered_indices = (0..self.file_nodes.len()).collect();
        } else {
            let query = self.search_query.to_lowercase();
            self.filtered_indices = self
                .file_nodes
                .iter()
                .enumerate()
                .filter(|(_, node)| {
                    node.path
                        .file_name()
                        .unwrap_or_default()
                        .to_string_lossy()
                        .to_lowercase()
                        .contains(&query)
                })
                .map(|(i, _)| i)
                .collect();
        }

        // Reset selection
        self.selected_index = 0;
        if !self.filtered_indices.is_empty() {
            self.list_state.select(Some(0));
            self.update_preview().ok();
        } else {
            self.list_state.select(None);
            self.preview_content = None;
        }
    }

    pub async fn run(&mut self) -> Result<()> {
        loop {
            // Temporarily take ownership of terminal to avoid borrow conflicts
            if let Some(mut terminal) = self.terminal.take() {
                terminal.draw(|f| self.draw(f))?;
                self.terminal = Some(terminal);
            }

            if event::poll(Duration::from_millis(100))? {
                if let Event::Key(key) = event::read()? {
                    if self.search_mode {
                        match key.code {
                            KeyCode::Esc => {
                                self.search_mode = false;
                                self.set_status("Search cancelled");
                            }
                            KeyCode::Enter => {
                                self.search_mode = false;
                                self.set_status(&format!(
                                    "Filtered: {} results",
                                    self.filtered_indices.len()
                                ));
                            }
                            KeyCode::Backspace => {
                                self.search_query.pop();
                                self.apply_search_filter();
                            }
                            KeyCode::Char(c) => {
                                self.search_query.push(c);
                                self.apply_search_filter();
                            }
                            _ => {}
                        }
                    } else {
                        match key.code {
                            KeyCode::Char('q') | KeyCode::Esc => break,
                            KeyCode::Char('?') | KeyCode::F(1) => {
                                self.show_help = !self.show_help;
                            }
                            KeyCode::Char('/') => {
                                self.search_mode = true;
                                self.search_query.clear();
                                self.set_status("Search mode: Type to filter files");
                            }
                            KeyCode::Char('h') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                                self.show_hidden = !self.show_hidden;
                                self.refresh_directory()?;
                                self.set_status(&format!(
                                    "Hidden files: {}",
                                    if self.show_hidden { "shown" } else { "hidden" }
                                ));
                            }
                            KeyCode::Up | KeyCode::Char('k') => self.move_selection(-1),
                            KeyCode::Down | KeyCode::Char('j') => self.move_selection(1),
                            KeyCode::PageUp => self.move_selection(-10),
                            KeyCode::PageDown => self.move_selection(10),
                            KeyCode::Home | KeyCode::Char('g') => {
                                if !self.filtered_indices.is_empty() {
                                    self.selected_index = 0;
                                    self.list_state.select(Some(0));
                                    self.update_preview()?;
                                }
                            }
                            KeyCode::End | KeyCode::Char('G') => {
                                if !self.filtered_indices.is_empty() {
                                    self.selected_index = self.filtered_indices.len() - 1;
                                    self.list_state.select(Some(self.selected_index));
                                    self.update_preview()?;
                                }
                            }
                            KeyCode::Enter | KeyCode::Char(' ') => {
                                self.enter_selected()?;
                            }
                            KeyCode::Backspace | KeyCode::Char('-') => {
                                if let Some(parent) = self.current_path.parent() {
                                    self.current_path = parent.to_path_buf();
                                    self.refresh_directory()?;
                                    self.set_status(&format!(
                                        "Navigated to: {}",
                                        self.current_path.display()
                                    ));
                                }
                            }
                            KeyCode::Char('r') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                                self.refresh_directory()?;
                                self.set_status("Directory refreshed");
                            }
                            _ => {}
                        }
                    }
                }
            }

            // Clear old status messages
            if let Some((_, time)) = &self.status_message {
                if time.elapsed().unwrap_or_default() > Duration::from_secs(3) {
                    self.status_message = None;
                }
            }
        }

        Ok(())
    }

    fn move_selection(&mut self, delta: i32) {
        if self.filtered_indices.is_empty() {
            return;
        }

        let len = self.filtered_indices.len() as i32;
        let new_index = (self.selected_index as i32 + delta).clamp(0, len - 1) as usize;

        if new_index != self.selected_index {
            self.selected_index = new_index;
            self.list_state.select(Some(new_index));
            self.update_preview().ok();
        }
    }

    fn enter_selected(&mut self) -> Result<()> {
        if self.filtered_indices.is_empty() {
            return Ok(());
        }

        let actual_index = self.filtered_indices[self.selected_index];
        let node = &self.file_nodes[actual_index];
        let path = node.path.clone(); // Clone the path to avoid borrow conflict

        if path.is_dir() {
            self.current_path = path.clone();
            self.refresh_directory()?;
            self.search_query.clear();
            self.set_status(&format!("Entered: {}", path.display()));
        } else {
            self.set_status(&format!("Selected: {}", path.display()));
        }

        Ok(())
    }

    fn set_status(&mut self, msg: &str) {
        self.status_message = Some((msg.to_string(), SystemTime::now()));
    }

    fn draw(&mut self, f: &mut Frame) {
        let size = f.size();

        // Create main layout
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3), // Header
                Constraint::Min(10),   // Main content
                Constraint::Length(3), // Status bar
            ])
            .split(size);

        // Draw header with spicy styling
        self.draw_header(f, chunks[0]);

        // Split main content area
        let main_chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Percentage(35), // File list
                Constraint::Percentage(45), // Preview
                Constraint::Percentage(20), // Info panel
            ])
            .split(chunks[1]);

        // Draw components
        self.draw_file_list(f, main_chunks[0]);
        self.draw_preview(f, main_chunks[1]);
        self.draw_info_panel(f, main_chunks[2]);

        // Draw status bar
        self.draw_status_bar(f, chunks[2]);

        // Draw help overlay if active
        if self.show_help {
            self.draw_help_overlay(f, size);
        }
    }

    fn draw_header(&self, f: &mut Frame, area: Rect) {
        let header_text = vec![
            Span::styled(" 🌶️ SPICY ", Style::default().fg(SPICY_ORANGE).bold()),
            Span::styled("TREE ", Style::default().fg(SPICY_GREEN).bold()),
            Span::styled("│ ", Style::default().fg(SPICY_BORDER)),
            Span::styled(
                self.current_path.display().to_string(),
                Style::default().fg(SPICY_CYAN),
            ),
        ];

        let header = Paragraph::new(Line::from(header_text))
            .style(Style::default().bg(SPICY_BG))
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(SPICY_BORDER))
                    .border_type(ratatui::widgets::BorderType::Double),
            )
            .alignment(Alignment::Left);

        f.render_widget(header, area);
    }

    fn draw_file_list(&mut self, f: &mut Frame, area: Rect) {
        let items: Vec<ListItem> = self
            .filtered_indices
            .iter()
            .enumerate()
            .map(|(display_idx, &actual_idx)| {
                let node = &self.file_nodes[actual_idx];
                let name = node.path.file_name().unwrap_or_default().to_string_lossy();

                let icon = if node.path.is_dir() {
                    "📁"
                } else {
                    icon_for(&node.path)
                };

                let style = if display_idx == self.selected_index {
                    Style::default()
                        .fg(Color::Black)
                        .bg(SPICY_GREEN)
                        .add_modifier(Modifier::BOLD)
                } else if node.path.is_dir() {
                    Style::default().fg(SPICY_CYAN)
                } else {
                    Style::default().fg(SPICY_GREEN)
                };

                ListItem::new(format!(" {} {}", icon, name)).style(style)
            })
            .collect();

        let title = if self.search_mode {
            format!(" 🔍 Search: {} ", self.search_query)
        } else if !self.search_query.is_empty() {
            format!(" 📂 Files (filtered: {}) ", self.filtered_indices.len())
        } else {
            " 📂 Files ".to_string()
        };

        let list = List::new(items)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(SPICY_BORDER))
                    .title(title)
                    .title_style(Style::default().fg(SPICY_YELLOW).bold()),
            )
            .highlight_style(Style::default());

        f.render_stateful_widget(list, area, &mut self.list_state);
    }

    fn draw_preview(&self, f: &mut Frame, area: Rect) {
        let content = self
            .preview_content
            .as_deref()
            .unwrap_or("No preview available");

        let preview = Paragraph::new(content)
            .style(Style::default().fg(SPICY_GREEN))
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(SPICY_BORDER))
                    .title(" 👁️ Preview ")
                    .title_style(Style::default().fg(SPICY_YELLOW).bold()),
            )
            .wrap(Wrap { trim: false })
            .scroll((self.preview_scroll, 0));

        f.render_widget(preview, area);
    }

    fn draw_info_panel(&self, f: &mut Frame, area: Rect) {
        let mut info_lines = vec![];

        if !self.filtered_indices.is_empty() {
            let actual_index = self.filtered_indices[self.selected_index];
            let node = &self.file_nodes[actual_index];

            info_lines.push(Line::from(vec![
                Span::styled("Type: ", Style::default().fg(SPICY_DARK_GREEN)),
                Span::styled(
                    if node.path.is_dir() {
                        "Directory"
                    } else {
                        "File"
                    },
                    Style::default().fg(SPICY_CYAN),
                ),
            ]));

            if let Ok(meta) = fs::metadata(&node.path) {
                // Size
                let size = if meta.is_file() {
                    format!("{:.2} KB", meta.len() as f64 / 1024.0)
                } else {
                    "N/A".to_string()
                };
                info_lines.push(Line::from(vec![
                    Span::styled("Size: ", Style::default().fg(SPICY_DARK_GREEN)),
                    Span::styled(size, Style::default().fg(SPICY_CYAN)),
                ]));

                // Permissions
                #[cfg(unix)]
                {
                    use std::os::unix::fs::PermissionsExt;
                    let mode = meta.permissions().mode();
                    let perms = format!("{:o}", mode & 0o777);
                    info_lines.push(Line::from(vec![
                        Span::styled("Perms: ", Style::default().fg(SPICY_DARK_GREEN)),
                        Span::styled(perms, Style::default().fg(SPICY_CYAN)),
                    ]));
                }
            }

            // Add some stats
            info_lines.push(Line::from(""));
            info_lines.push(Line::from(vec![Span::styled(
                "─────────",
                Style::default().fg(SPICY_BORDER),
            )]));
            info_lines.push(Line::from(vec![
                Span::styled("Total: ", Style::default().fg(SPICY_DARK_GREEN)),
                Span::styled(
                    format!("{}", self.file_nodes.len()),
                    Style::default().fg(SPICY_CYAN),
                ),
            ]));
            info_lines.push(Line::from(vec![
                Span::styled("Shown: ", Style::default().fg(SPICY_DARK_GREEN)),
                Span::styled(
                    format!("{}", self.filtered_indices.len()),
                    Style::default().fg(SPICY_CYAN),
                ),
            ]));
        }

        let info = Paragraph::new(info_lines).block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(SPICY_BORDER))
                .title(" 📊 Info ")
                .title_style(Style::default().fg(SPICY_YELLOW).bold()),
        );

        f.render_widget(info, area);
    }

    fn draw_status_bar(&self, f: &mut Frame, area: Rect) {
        let mut status_spans = vec![];

        // Left side - shortcuts
        status_spans.push(Span::styled(
            " q:Quit │ /:Search │ Enter:Open │ ?:Help ",
            Style::default().fg(SPICY_DARK_GREEN),
        ));

        // Right side - status message or default
        if let Some((msg, _)) = &self.status_message {
            status_spans.push(Span::styled(" │ ", Style::default().fg(SPICY_BORDER)));
            status_spans.push(Span::styled(msg, Style::default().fg(SPICY_YELLOW)));
        }

        let status = Paragraph::new(Line::from(status_spans))
            .style(Style::default().bg(SPICY_BG))
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(SPICY_BORDER)),
            );

        f.render_widget(status, area);
    }

    fn draw_help_overlay(&self, f: &mut Frame, area: Rect) {
        let help_text = vec![
            Line::from(vec![Span::styled(
                "🌶️ SPICY TREE HELP",
                Style::default().fg(SPICY_ORANGE).bold(),
            )]),
            Line::from(""),
            Line::from(vec![Span::styled(
                "Navigation:",
                Style::default().fg(SPICY_YELLOW).bold(),
            )]),
            Line::from("  ↑/k     - Move up"),
            Line::from("  ↓/j     - Move down"),
            Line::from("  Enter   - Open directory / Select file"),
            Line::from("  Backspace - Go to parent directory"),
            Line::from("  g/Home  - Go to first item"),
            Line::from("  G/End   - Go to last item"),
            Line::from(""),
            Line::from(vec![Span::styled(
                "Search:",
                Style::default().fg(SPICY_YELLOW).bold(),
            )]),
            Line::from("  /       - Start search mode"),
            Line::from("  Esc     - Cancel search"),
            Line::from("  Enter   - Apply search filter"),
            Line::from(""),
            Line::from(vec![Span::styled(
                "Display:",
                Style::default().fg(SPICY_YELLOW).bold(),
            )]),
            Line::from("  Ctrl+H  - Toggle hidden files"),
            Line::from("  Ctrl+R  - Refresh directory"),
            Line::from("  ?/F1    - Toggle this help"),
            Line::from(""),
            Line::from(vec![Span::styled(
                "Press any key to close help",
                Style::default().fg(SPICY_DARK_GREEN).italic(),
            )]),
        ];

        let help_width = 50;
        let help_height = 24;
        let help_area = centered_rect(help_width, help_height, area);

        let help = Paragraph::new(help_text)
            .style(Style::default().fg(SPICY_GREEN))
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(SPICY_BORDER))
                    .border_type(ratatui::widgets::BorderType::Double)
                    .title(" ❓ Help ")
                    .title_style(Style::default().fg(SPICY_YELLOW).bold())
                    .style(Style::default().bg(SPICY_BG)),
            )
            .alignment(Alignment::Left);

        // Draw background overlay
        let overlay = Block::default().style(Style::default().bg(Color::Rgb(0, 0, 0)));
        f.render_widget(overlay, area);

        // Draw help dialog
        f.render_widget(help, help_area);
    }
}

impl Drop for SpicyTui {
    fn drop(&mut self) {
        // Restore terminal
        disable_raw_mode().ok();
        if let Some(mut term) = self.terminal.take() {
            execute!(
                term.backend_mut(),
                LeaveAlternateScreen,
                DisableMouseCapture
            )
            .ok();
            term.show_cursor().ok();
        }
    }
}

// Helper function to center a rect
fn centered_rect(width: u16, height: u16, area: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length((area.height - height) / 2),
            Constraint::Length(height),
            Constraint::Length((area.height - height) / 2),
        ])
        .split(area);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Length((area.width - width) / 2),
            Constraint::Length(width),
            Constraint::Length((area.width - width) / 2),
        ])
        .split(popup_layout[1])[1]
}

// Simple icon heuristic without heavy content detection
fn icon_for(path: &Path) -> &'static str {
    if let Some(ext) = path
        .extension()
        .and_then(|e| e.to_str())
        .map(|s| s.to_ascii_lowercase())
    {
        match ext.as_str() {
            "rs" => "🦀",
            "py" => "🐍",
            "js" | "ts" | "tsx" | "jsx" => "📜",
            "md" | "markdown" => "📝",
            "png" | "jpg" | "jpeg" | "gif" | "webp" | "svg" => "🖼️",
            "exe" | "bin" | "dll" | "so" | "dylib" => "⚙️",
            _ => "📄",
        }
    } else {
        "📄"
    }
}

// Public entry point
pub async fn run_spicy_tui(path: PathBuf) -> Result<()> {
    let mut app = SpicyTui::new(path)?;
    app.run().await
}
