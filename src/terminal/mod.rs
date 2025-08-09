//! Smart Tree Terminal Interface (STTI) - Your Coding Companion 🌳
//!
//! Like a construction helper who hands you tools before you ask!
//! This module provides context-aware terminal assistance.

use anyhow::Result;
use crossterm::{
    event::{self, Event, KeyCode, KeyEvent},
    terminal::{self, EnterAlternateScreen, LeaveAlternateScreen},
    ExecutableCommand,
};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span, Text},
    widgets::{Block, Borders, List, ListItem, Paragraph},
    Frame, Terminal,
};
use std::{
    io::{self, Stdout},
    path::PathBuf,
    sync::{Arc, Mutex},
    time::{Duration, Instant},
};
use tokio::sync::mpsc;

use crate::smart::ProjectType;

/// Terminal UI state
#[derive(Debug, Clone)]
pub struct TerminalState {
    /// Current working directory
    pub cwd: PathBuf,

    /// Active file being edited
    pub active_file: Option<PathBuf>,

    /// Recent file changes
    pub recent_changes: Vec<FileChange>,

    /// Current suggestions
    pub suggestions: Vec<Suggestion>,

    /// Command history
    pub command_history: Vec<String>,

    /// Current input buffer
    pub input: String,

    /// Cursor position in input
    pub cursor_pos: usize,

    /// Project context
    pub project_type: Option<ProjectType>,

    /// Status message
    pub status_message: Option<StatusMessage>,
}

/// File change event
#[derive(Debug, Clone)]
pub struct FileChange {
    pub path: PathBuf,
    pub change_type: ChangeType,
    pub timestamp: Instant,
}

#[derive(Debug, Clone)]
pub enum ChangeType {
    Created,
    Modified,
    Deleted,
    Renamed { from: PathBuf },
}

/// Suggestion from the AI assistant
#[derive(Debug, Clone)]
pub struct Suggestion {
    pub icon: &'static str,
    pub title: String,
    pub description: String,
    pub action: SuggestionAction,
    pub confidence: f32,
}

#[derive(Debug, Clone)]
pub enum SuggestionAction {
    InsertText(String),
    RunCommand(String),
    OpenFile(PathBuf),
    CreateFile { path: PathBuf, content: String },
    RefactorCode { file: PathBuf, operation: String },
}

/// Status message with severity
#[derive(Debug, Clone)]
pub struct StatusMessage {
    pub text: String,
    pub severity: MessageSeverity,
    pub timestamp: Instant,
}

#[derive(Debug, Clone, Copy)]
pub enum MessageSeverity {
    Info,
    Success,
    Warning,
    Error,
}

/// Main terminal interface
pub struct SmartTreeTerminal {
    /// Terminal handle
    terminal: Terminal<CrosstermBackend<Stdout>>,

    /// Current state
    state: Arc<Mutex<TerminalState>>,

    /// Context watcher
    context_watcher: ContextWatcher,

    /// Pattern analyzer
    pattern_analyzer: PatternAnalyzer,

    /// Suggestion receiver
    suggestion_rx: mpsc::Receiver<Suggestion>,

    /// Suggestion sender (for background tasks)
    _suggestion_tx: mpsc::Sender<Suggestion>,
}

impl SmartTreeTerminal {
    /// Create new terminal interface
    pub fn new() -> Result<Self> {
        // Setup terminal
        terminal::enable_raw_mode()?;
        let mut stdout = io::stdout();
        stdout.execute(EnterAlternateScreen)?;

        let backend = CrosstermBackend::new(stdout);
        let terminal = Terminal::new(backend)?;

        // Create channels
        let (suggestion_tx, suggestion_rx) = mpsc::channel(100);

        // Initial state
        let state = Arc::new(Mutex::new(TerminalState {
            cwd: std::env::current_dir()?,
            active_file: None,
            recent_changes: Vec::new(),
            suggestions: Vec::new(),
            command_history: Vec::new(),
            input: String::new(),
            cursor_pos: 0,
            project_type: None,
            status_message: None,
        }));

        Ok(Self {
            terminal,
            state: state.clone(),
            context_watcher: ContextWatcher::new(state.clone(), suggestion_tx.clone()),
            pattern_analyzer: PatternAnalyzer::new(state.clone(), suggestion_tx.clone()),
            suggestion_rx,
            _suggestion_tx: suggestion_tx,
        })
    }

    /// Run the terminal interface
    pub async fn run(&mut self) -> Result<()> {
        // Start background tasks
        self.context_watcher.start().await?;
        self.pattern_analyzer.start().await?;

        loop {
            // Draw UI
            self.draw()?;

            // Handle events
            if event::poll(Duration::from_millis(100))? {
                if let Event::Key(key) = event::read()? {
                    if self.handle_key(key).await? {
                        break;
                    }
                }
            }

            // Process suggestions
            while let Ok(suggestion) = self.suggestion_rx.try_recv() {
                let mut state = self.state.lock().unwrap();
                state.suggestions.push(suggestion);
                // Keep only the 5 most recent suggestions
                if state.suggestions.len() > 5 {
                    state.suggestions.remove(0);
                }
            }
        }

        // Cleanup
        terminal::disable_raw_mode()?;
        self.terminal.backend_mut().execute(LeaveAlternateScreen)?;

        Ok(())
    }

    /// Draw the UI
    fn draw(&mut self) -> Result<()> {
        let state = self.state.lock().unwrap().clone();

        self.terminal.draw(|f| {
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Length(3), // Header
                    Constraint::Length(3), // Context bar
                    Constraint::Min(10),   // Main area
                    Constraint::Length(3), // Input
                    Constraint::Length(1), // Status bar
                ])
                .split(f.size());

            // Header
            Self::draw_header(f, chunks[0], &state);

            // Context bar
            Self::draw_context(f, chunks[1], &state);

            // Main area (split into suggestions and history)
            let main_chunks = Layout::default()
                .direction(Direction::Horizontal)
                .constraints([
                    Constraint::Percentage(60), // History/Output
                    Constraint::Percentage(40), // Suggestions
                ])
                .split(chunks[2]);

            Self::draw_history(f, main_chunks[0], &state);
            Self::draw_suggestions(f, main_chunks[1], &state);

            // Input area
            Self::draw_input(f, chunks[3], &state);

            // Status bar
            Self::draw_status(f, chunks[4], &state);
        })?;

        Ok(())
    }

    /// Draw header
    fn draw_header(f: &mut Frame, area: Rect, _state: &TerminalState) {
        let header = Paragraph::new(Text::from(vec![Line::from(vec![
            Span::styled(
                "Smart Tree Terminal",
                Style::default()
                    .fg(Color::Green)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw(" v4.0 - "),
            Span::styled("Your Coding Companion ", Style::default().fg(Color::Cyan)),
            Span::raw("🌳"),
        ])]))
        .block(Block::default().borders(Borders::ALL))
        .alignment(ratatui::layout::Alignment::Center);

        f.render_widget(header, area);
    }

    /// Draw context information
    fn draw_context(f: &mut Frame, area: Rect, state: &TerminalState) {
        let mut context_items = vec![Span::styled("Context: ", Style::default().fg(Color::Gray))];

        if let Some(file) = &state.active_file {
            context_items.push(Span::styled(
                format!("Editing: {} ", file.display()),
                Style::default().fg(Color::Yellow),
            ));
        }

        if let Some(project) = &state.project_type {
            context_items.push(Span::styled(
                format!("| Project: {:?} ", project),
                Style::default().fg(Color::Blue),
            ));
        }

        let context = Paragraph::new(Line::from(context_items))
            .block(Block::default().borders(Borders::LEFT | Borders::RIGHT));

        f.render_widget(context, area);
    }

    /// Draw command history
    fn draw_history(f: &mut Frame, area: Rect, state: &TerminalState) {
        let history_items: Vec<ListItem> = state
            .command_history
            .iter()
            .rev()
            .take(area.height as usize - 2)
            .map(|cmd| ListItem::new(cmd.as_str()))
            .collect();

        let history =
            List::new(history_items).block(Block::default().title("History").borders(Borders::ALL));

        f.render_widget(history, area);
    }

    /// Draw suggestions panel
    fn draw_suggestions(f: &mut Frame, area: Rect, state: &TerminalState) {
        let suggestion_items: Vec<ListItem> = state
            .suggestions
            .iter()
            .map(|s| {
                ListItem::new(vec![
                    Line::from(vec![
                        Span::raw(s.icon),
                        Span::raw(" "),
                        Span::styled(&s.title, Style::default().add_modifier(Modifier::BOLD)),
                    ]),
                    Line::from(Span::styled(
                        &s.description,
                        Style::default().fg(Color::Gray),
                    )),
                ])
            })
            .collect();

        let suggestions = List::new(suggestion_items).block(
            Block::default()
                .title("💡 Suggestions")
                .borders(Borders::ALL),
        );

        f.render_widget(suggestions, area);
    }

    /// Draw input area
    fn draw_input(f: &mut Frame, area: Rect, state: &TerminalState) {
        let input = Paragraph::new(state.input.as_str()).block(
            Block::default()
                .title(format!("{}$ ", state.cwd.display()))
                .borders(Borders::ALL),
        );

        f.render_widget(input, area);

        // Set cursor position
        f.set_cursor(area.x + state.cursor_pos as u16 + 1, area.y + 1);
    }

    /// Draw status bar
    fn draw_status(f: &mut Frame, area: Rect, state: &TerminalState) {
        let status_text = if let Some(msg) = &state.status_message {
            let color = match msg.severity {
                MessageSeverity::Info => Color::Blue,
                MessageSeverity::Success => Color::Green,
                MessageSeverity::Warning => Color::Yellow,
                MessageSeverity::Error => Color::Red,
            };

            Span::styled(&msg.text, Style::default().fg(color))
        } else {
            Span::raw("Ready")
        };

        let status = Paragraph::new(Line::from(vec![
            status_text,
            Span::raw(" | "),
            Span::raw("Press Ctrl+C to exit"),
        ]));

        f.render_widget(status, area);
    }

    /// Handle keyboard input
    async fn handle_key(&mut self, key: KeyEvent) -> Result<bool> {
        match key.code {
            KeyCode::Char('c') if key.modifiers.contains(event::KeyModifiers::CONTROL) => {
                return Ok(true); // Exit
            }
            KeyCode::Char(c) => {
                let mut state = self.state.lock().unwrap();
                let cursor_pos = state.cursor_pos;
                state.input.insert(cursor_pos, c);
                state.cursor_pos += 1;

                // Trigger pattern analysis on input change
                drop(state); // Release lock before async call
                self.pattern_analyzer.analyze_input().await?;
            }
            KeyCode::Backspace => {
                let mut state = self.state.lock().unwrap();
                if state.cursor_pos > 0 {
                    let cursor_pos = state.cursor_pos;
                    state.input.remove(cursor_pos - 1);
                    state.cursor_pos -= 1;
                }
            }
            KeyCode::Enter => {
                let mut state = self.state.lock().unwrap();
                let command = state.input.clone();
                state.command_history.push(command.clone());
                state.input.clear();
                state.cursor_pos = 0;

                // Process command
                drop(state); // Release lock
                self.process_command(&command).await?;
            }
            KeyCode::Tab => {
                // Accept top suggestion
                let state = self.state.lock().unwrap();
                if let Some(suggestion) = state.suggestions.first() {
                    let action = suggestion.action.clone();
                    drop(state);
                    self.apply_suggestion(action).await?;
                }
            }
            _ => {}
        }

        Ok(false)
    }

    /// Process a command
    async fn process_command(&mut self, command: &str) -> Result<()> {
        // This is where we'd integrate with the shell
        // For now, just update status
        let mut state = self.state.lock().unwrap();
        state.status_message = Some(StatusMessage {
            text: format!("Executed: {}", command),
            severity: MessageSeverity::Info,
            timestamp: Instant::now(),
        });

        Ok(())
    }

    /// Apply a suggestion
    async fn apply_suggestion(&mut self, action: SuggestionAction) -> Result<()> {
        match action {
            SuggestionAction::InsertText(text) => {
                let mut state = self.state.lock().unwrap();
                let cursor_pos = state.cursor_pos;
                state.input.insert_str(cursor_pos, &text);
                state.cursor_pos += text.len();
            }
            SuggestionAction::RunCommand(cmd) => {
                self.process_command(&cmd).await?;
            }
            _ => {
                // TODO: Implement other actions
            }
        }

        Ok(())
    }
}

/// Context watcher - monitors file system and project state
pub struct ContextWatcher {
    state: Arc<Mutex<TerminalState>>,
    _suggestion_tx: mpsc::Sender<Suggestion>,
}

impl ContextWatcher {
    fn new(state: Arc<Mutex<TerminalState>>, _suggestion_tx: mpsc::Sender<Suggestion>) -> Self {
        Self {
            state,
            _suggestion_tx,
        }
    }

    async fn start(&self) -> Result<()> {
        // TODO: Implement file watching
        // For now, detect project type
        let cwd = self.state.lock().unwrap().cwd.clone();

        if cwd.join("Cargo.toml").exists() {
            let mut state = self.state.lock().unwrap();
            state.project_type = Some(ProjectType::Rust);

            // Send a suggestion
            let _ = self
                ._suggestion_tx
                .send(Suggestion {
                    icon: "🦀",
                    title: "Rust Project Detected".to_string(),
                    description: "Run 'cargo build' to compile".to_string(),
                    action: SuggestionAction::RunCommand("cargo build".to_string()),
                    confidence: 0.9,
                })
                .await;
        }

        Ok(())
    }
}

/// Pattern analyzer - analyzes coding patterns and suggests actions
pub struct PatternAnalyzer {
    state: Arc<Mutex<TerminalState>>,
    _suggestion_tx: mpsc::Sender<Suggestion>,
}

impl PatternAnalyzer {
    fn new(state: Arc<Mutex<TerminalState>>, _suggestion_tx: mpsc::Sender<Suggestion>) -> Self {
        Self {
            state,
            _suggestion_tx,
        }
    }

    async fn start(&self) -> Result<()> {
        // TODO: Implement pattern learning
        Ok(())
    }

    async fn analyze_input(&self) -> Result<()> {
        let state = self.state.lock().unwrap();
        let input = state.input.clone();
        drop(state);

        // Simple pattern matching for demo
        if input.starts_with("git com") {
            let _ = self
                ._suggestion_tx
                .send(Suggestion {
                    icon: "📝",
                    title: "Git Commit".to_string(),
                    description: "Commit recent changes".to_string(),
                    action: SuggestionAction::InsertText("mit -m \"".to_string()),
                    confidence: 0.8,
                })
                .await;
        } else if input.contains("import") {
            let _ = self
                ._suggestion_tx
                .send(Suggestion {
                    icon: "📦",
                    title: "Import Suggestion".to_string(),
                    description: "Add commonly used imports".to_string(),
                    action: SuggestionAction::InsertText(" { useState } from 'react'".to_string()),
                    confidence: 0.7,
                })
                .await;
        }

        Ok(())
    }
}

// Trisha says: "This terminal is like having a personal assistant who knows
// exactly which receipt you need before you even open the filing cabinet!" 📁
