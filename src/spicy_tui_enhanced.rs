// Enhanced Spicy TUI - "Like climbing a rope in gym, but cooler!" 🌶️🌲
// Tree navigation, dual search, M8 context, and ASCII art!

use anyhow::Result;
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyModifiers},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style, Stylize},
    text::{Line, Span},
    widgets::{
        Block, Borders, List, ListItem, ListState, Paragraph, Wrap, Clear,
    },
    Frame, Terminal,
};
use std::{
    collections::HashMap,
    fs,
    io::{self, BufRead},
    path::{Path, PathBuf},
    time::{Duration, SystemTime},
};
use fuzzy_matcher::skim::SkimMatcherV2;
use fuzzy_matcher::FuzzyMatcher;

use crate::{
    scanner::{Scanner, ScannerConfig},
    spicy_fuzzy::{SpicyFuzzySearch, FileMatch},
    memory_manager::MemoryManager,
};

// Enhanced color scheme
const SPICY_GREEN: Color = Color::Rgb(0, 255, 0);
const SPICY_DARK_GREEN: Color = Color::Rgb(0, 128, 0);
const SPICY_YELLOW: Color = Color::Rgb(255, 255, 0);
const SPICY_CYAN: Color = Color::Rgb(0, 255, 255);
const SPICY_MAGENTA: Color = Color::Rgb(255, 0, 255);
const SPICY_ORANGE: Color = Color::Rgb(255, 165, 0);
const SPICY_BG: Color = Color::Rgb(10, 10, 10);
const SPICY_BORDER: Color = Color::Rgb(0, 100, 0);
const HIGHLIGHT_COLOR: Color = Color::Rgb(255, 255, 100);

#[derive(Debug, Clone, PartialEq)]
enum SearchMode {
    Off,
    FileName,
    FileContent,
}

#[derive(Debug, Clone)]
struct TreeNode {
    path: PathBuf,
    name: String,
    is_dir: bool,
    is_expanded: bool,
    depth: usize,
    children: Vec<TreeNode>,
}

pub struct EnhancedSpicyTui {
    terminal: Option<Terminal<CrosstermBackend<io::Stdout>>>,
    current_path: PathBuf,
    tree: TreeNode,
    selected_path: PathBuf,
    list_state: ListState,
    preview_content: Option<String>,
    search_query: String,
    search_mode: SearchMode,
    search_results: Vec<FileMatch>,
    filtered_paths: Vec<PathBuf>,
    scroll_offset: u16,
    preview_scroll: u16,
    show_hidden: bool,
    show_help: bool,
    status_message: Option<(String, SystemTime)>,
    fuzzy_searcher: SpicyFuzzySearch,
    memory_manager: MemoryManager,
    search_history: Vec<String>,
    ascii_art_cache: HashMap<PathBuf, String>,
}

impl EnhancedSpicyTui {
    pub fn new(path: PathBuf) -> Result<Self> {
        // Setup terminal
        enable_raw_mode()?;
        let mut stdout = io::stdout();
        execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
        let backend = CrosstermBackend::new(stdout);
        let terminal = Terminal::new(backend)?;

        let fuzzy_searcher = SpicyFuzzySearch::new()?;
        let memory_manager = MemoryManager::new()?;

        let mut app = Self {
            terminal: Some(terminal),
            current_path: path.clone(),
            tree: TreeNode {
                path: path.clone(),
                name: path.file_name()
                    .unwrap_or_default()
                    .to_string_lossy()
                    .to_string(),
                is_dir: true,
                is_expanded: true,
                depth: 0,
                children: Vec::new(),
            },
            selected_path: path.clone(),
            list_state: ListState::default(),
            preview_content: None,
            search_query: String::new(),
            search_mode: SearchMode::Off,
            search_results: Vec::new(),
            filtered_paths: Vec::new(),
            scroll_offset: 0,
            preview_scroll: 0,
            show_hidden: false,
            show_help: false,
            status_message: None,
            fuzzy_searcher,
            memory_manager,
            search_history: Vec::new(),
            ascii_art_cache: HashMap::new(),
        };

        app.refresh_tree()?;
        // Initialize the list state to select the first item
        app.list_state.select(Some(0));
        Ok(app)
    }

    fn refresh_tree(&mut self) -> Result<()> {
        self.tree = self.build_tree_node(&self.current_path, 0, 3)?;
        self.update_preview()?;
        Ok(())
    }

    fn build_tree_node(&self, path: &Path, depth: usize, max_depth: usize) -> Result<TreeNode> {
        let name = path.file_name()
            .unwrap_or_default()
            .to_string_lossy()
            .to_string();

        let mut node = TreeNode {
            path: path.to_path_buf(),
            name,
            is_dir: path.is_dir(),
            is_expanded: depth == 0, // Expand root by default
            depth,
            children: Vec::new(),
        };

        if node.is_dir && depth < max_depth {
            let config = ScannerConfig {
                max_depth: 1,
                show_hidden: self.show_hidden,
                respect_gitignore: true,
                use_default_ignores: true,
                ..Default::default()
            };

            if let Ok(scanner) = Scanner::new(path, config) {
                if let Ok((children, _)) = scanner.scan() {
                    for child in children {
                        if let Ok(child_node) = self.build_tree_node(&child.path, depth + 1, max_depth) {
                            node.children.push(child_node);
                        }
                    }
                }
            }
        }

        Ok(node)
    }

    fn update_preview(&mut self) -> Result<()> {
        let path = self.selected_path.clone();

        if path.is_file() {
            // Check if it's an image
            if self.is_image(&path) {
                self.preview_content = Some(self.get_ascii_art(&path)?);
            } else {
                // Regular file preview
                match fs::metadata(&path) {
                    Ok(meta) if meta.len() > 1_000_000 => {
                        self.preview_content = Some(format!(
                            "📁 File too large for preview\nSize: {:.2} MB",
                            meta.len() as f64 / 1_048_576.0
                        ));
                    }
                    Ok(_) => {
                        if let Ok(content) = fs::read_to_string(&path) {
                            let preview = if self.search_mode == SearchMode::FileContent
                                && !self.search_query.is_empty() {
                                self.highlight_content(&content, &self.search_query)
                            } else {
                                content.lines().take(100).collect::<Vec<_>>().join("\n")
                            };
                            self.preview_content = Some(preview);
                        }
                    }
                    Err(_) => {
                        self.preview_content = Some("❌ Permission denied".to_string());
                    }
                }
            }
        } else if path.is_dir() {
            // Directory preview with tree structure
            let tree_preview = self.generate_tree_preview(&path)?;
            self.preview_content = Some(tree_preview);
        }

        Ok(())
    }

    fn is_image(&self, path: &Path) -> bool {
        if let Some(ext) = path.extension() {
            matches!(
                ext.to_str().map(|s| s.to_lowercase()).as_deref(),
                Some("png") | Some("jpg") | Some("jpeg") | Some("gif") |
                Some("bmp") | Some("webp") | Some("svg")
            )
        } else {
            false
        }
    }

    fn get_ascii_art(&mut self, path: &Path) -> Result<String> {
        // Check cache first
        if let Some(cached) = self.ascii_art_cache.get(path) {
            return Ok(cached.clone());
        }

        // Generate ASCII art using artem CLI
        let ascii = if path.exists() {
            // Try to use artem command-line tool
            match std::process::Command::new("artem")
                .arg(path)
                .arg("--width")
                .arg("40")
                .output()
            {
                Ok(output) => {
                    if output.status.success() {
                        String::from_utf8_lossy(&output.stdout).to_string()
                    } else {
                        "🖼️ Image preview not available (artem failed)".to_string()
                    }
                }
                Err(_) => {
                    // Artem not installed, just show a placeholder
                    "🖼️ Image preview (install artem for ASCII art)".to_string()
                }
            }
        } else {
            "🖼️ Unable to convert image".to_string()
        };

        // Cache it
        self.ascii_art_cache.insert(path.to_path_buf(), ascii.clone());
        Ok(ascii)
    }

    fn highlight_content(&self, content: &str, query: &str) -> String {
        let mut highlighted = String::new();
        let matcher = SkimMatcherV2::default();

        for line in content.lines().take(100) {
            if let Some((score, indices)) = matcher.fuzzy_indices(line, query) {
                let mut chars: Vec<char> = line.chars().collect();
                let mut result = String::new();

                for (i, ch) in chars.iter().enumerate() {
                    if indices.contains(&i) {
                        result.push_str(&format!(">>{}<<", ch)); // Highlight markers
                    } else {
                        result.push(*ch);
                    }
                }
                highlighted.push_str(&format!("🔍 {}\n", result));
            } else {
                highlighted.push_str(&format!("   {}\n", line));
            }
        }

        highlighted
    }

    fn generate_tree_preview(&self, path: &Path) -> Result<String> {
        let mut preview = format!("📂 {}\n\n", path.display());

        if let Ok(tree) = self.build_tree_node(path, 0, 2) {
            self.append_tree_preview(&tree, &mut preview, "", true);
        }

        Ok(preview)
    }

    fn append_tree_preview(&self, node: &TreeNode, output: &mut String, prefix: &str, is_last: bool) {
        let connector = if is_last { "└── " } else { "├── " };
        let icon = if node.is_dir { "📁" } else { self.get_file_icon(&node.path) };

        output.push_str(&format!("{}{}{} {}\n", prefix, connector, icon, node.name));

        let new_prefix = format!("{}{}", prefix, if is_last { "    " } else { "│   " });

        for (i, child) in node.children.iter().enumerate() {
            self.append_tree_preview(child, output, &new_prefix, i == node.children.len() - 1);
        }
    }

    fn get_file_icon(&self, path: &Path) -> &'static str {
        if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
            match ext.to_lowercase().as_str() {
                "rs" => "🦀",
                "py" => "🐍",
                "js" | "ts" => "📜",
                "md" => "📝",
                "png" | "jpg" | "jpeg" | "gif" => "🖼️",
                _ => "📄",
            }
        } else {
            "📄"
        }
    }

    pub async fn run(&mut self) -> Result<()> {
        loop {
            if let Some(mut terminal) = self.terminal.take() {
                terminal.draw(|f| self.draw(f))?;
                self.terminal = Some(terminal);
            }

            if event::poll(Duration::from_millis(100))? {
                if let Event::Key(key) = event::read()? {
                    match self.search_mode {
                        SearchMode::Off => self.handle_normal_input(key).await?,
                        _ => self.handle_search_input(key).await?,
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
    }

    async fn handle_normal_input(&mut self, key: event::KeyEvent) -> Result<()> {
        match key.code {
            KeyCode::Char('q') | KeyCode::Esc => std::process::exit(0),
            KeyCode::Char('?') | KeyCode::F(1) => {
                self.show_help = !self.show_help;
            }
            KeyCode::Char('/') => {
                self.search_mode = SearchMode::FileName;
                self.search_query.clear();
                self.set_status("🔍 File name search: Type to filter");
            }
            KeyCode::Char('f') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                self.search_mode = SearchMode::FileContent;
                self.search_query.clear();
                self.set_status("🔎 Content search: Find text in files");
            }
            KeyCode::Up | KeyCode::Char('k') => self.move_selection(-1),
            KeyCode::Down | KeyCode::Char('j') => self.move_selection(1),
            KeyCode::Left | KeyCode::Char('h') => self.collapse_or_navigate_up(),
            KeyCode::Right | KeyCode::Char('l') => self.expand_or_navigate_in()?,
            KeyCode::Enter => self.enter_selected()?,
            _ => {}
        }
        Ok(())
    }

    async fn handle_search_input(&mut self, key: event::KeyEvent) -> Result<()> {
        match key.code {
            KeyCode::Esc => {
                self.search_mode = SearchMode::Off;
                self.set_status("Search cancelled");
            }
            KeyCode::Enter => {
                self.execute_search().await?;
            }
            KeyCode::Backspace => {
                self.search_query.pop();
                if self.search_mode == SearchMode::FileName {
                    self.filter_files()?;
                }
            }
            KeyCode::Char(c) => {
                self.search_query.push(c);
                if self.search_mode == SearchMode::FileName {
                    self.filter_files()?;
                }
            }
            _ => {}
        }
        Ok(())
    }

    fn filter_files(&mut self) -> Result<()> {
        let matcher = SkimMatcherV2::default();
        let query = self.search_query.to_lowercase();

        self.filtered_paths = self.flatten_tree(&self.tree)
            .into_iter()
            .filter(|path| {
                let name = path.file_name()
                    .unwrap_or_default()
                    .to_string_lossy()
                    .to_lowercase();
                matcher.fuzzy_match(&name, &query).is_some()
            })
            .collect();

        self.set_status(&format!("Found {} matches", self.filtered_paths.len()));
        Ok(())
    }

    async fn execute_search(&mut self) -> Result<()> {
        if self.search_mode == SearchMode::FileContent {
            // Search file contents
            let results = self.fuzzy_searcher.search_content(
                &self.current_path,
                &self.search_query,
                50
            )?;

            // Save to M8 context - THIS IS THE COOL PART!
            if !results.is_empty() {
                self.save_search_to_m8(&results).await?;
            }

            self.search_results = results;
            self.set_status(&format!("Found {} results in files (saved to M8!)", self.search_results.len()));
        }

        self.search_history.push(self.search_query.clone());
        self.search_mode = SearchMode::Off;
        Ok(())
    }

    async fn save_search_to_m8(&mut self, results: &[FileMatch]) -> Result<()> {
        // Create a memory anchor for this search
        let keywords = vec![
            self.search_query.clone(),
            self.current_path.display().to_string(),
        ];

        let context = format!(
            "Search '{}' in {} found {} matches:\n{}",
            self.search_query,
            self.current_path.display(),
            results.len(),
            results.iter()
                .take(5)
                .map(|m| format!("  {}:{} - {}",
                    m.path.file_name().unwrap_or_default().to_string_lossy(),
                    m.line_number,
                    m.line_content.chars().take(50).collect::<String>()
                ))
                .collect::<Vec<_>>()
                .join("\n")
        );

        self.memory_manager.anchor(
            "search_result",
            keywords,
            &context,
            "spicy_tui"
        )?;

        Ok(())
    }

    fn flatten_tree(&self, node: &TreeNode) -> Vec<PathBuf> {
        let mut paths = vec![node.path.clone()];
        for child in &node.children {
            paths.extend(self.flatten_tree(child));
        }
        paths
    }

    fn move_selection(&mut self, delta: i32) {
        // Implement tree-aware selection movement
        let flat_tree = self.flatten_tree(&self.tree);
        if let Some(current_idx) = flat_tree.iter().position(|p| p == &self.selected_path) {
            let new_idx = ((current_idx as i32 + delta).max(0) as usize)
                .min(flat_tree.len().saturating_sub(1));
            self.selected_path = flat_tree[new_idx].clone();

            // Update the list state to move the selection bar
            self.list_state.select(Some(new_idx));

            self.update_preview().ok();
        }
    }

    fn collapse_or_navigate_up(&mut self) {
        if self.selected_path != self.current_path {
            if let Some(parent) = self.selected_path.parent() {
                self.selected_path = parent.to_path_buf();

                // Update list state to match the new selected path
                let flat_tree = self.flatten_tree(&self.tree);
                if let Some(idx) = flat_tree.iter().position(|p| p == &self.selected_path) {
                    self.list_state.select(Some(idx));
                }

                self.update_preview().ok();
                self.set_status("📁 Navigated up");
            }
        }
    }

    fn expand_or_navigate_in(&mut self) -> Result<()> {
        if self.selected_path.is_dir() {
            // Find and toggle the node
            let path = self.selected_path.clone();
            self.toggle_node_expansion(&path)?;

            // Update list state to maintain position
            let flat_tree = self.flatten_tree(&self.tree);
            if let Some(idx) = flat_tree.iter().position(|p| p == &self.selected_path) {
                self.list_state.select(Some(idx));
            }

            self.set_status("📂 Toggled directory");
        } else {
            // Can't expand files
            self.set_status("📄 This is a file");
        }
        Ok(())
    }

    fn toggle_node_expansion(&mut self, path: &Path) -> Result<()> {
        fn toggle_recursive(node: &mut TreeNode, target_path: &Path) -> Result<bool> {
            if node.path == target_path {
                node.is_expanded = !node.is_expanded;
                return Ok(true);
            }
            for child in &mut node.children {
                if toggle_recursive(child, target_path)? {
                    return Ok(true);
                }
            }
            Ok(false)
        }

        toggle_recursive(&mut self.tree, path)?;
        Ok(())
    }

    fn enter_selected(&mut self) -> Result<()> {
        if self.selected_path.is_dir() {
            self.current_path = self.selected_path.clone();
            self.refresh_tree()?;
            self.set_status(&format!("📁 Entered {}", self.selected_path.display()));
        }
        Ok(())
    }

    fn set_status(&mut self, msg: &str) {
        self.status_message = Some((msg.to_string(), SystemTime::now()));
    }

    fn draw(&mut self, f: &mut Frame) {
        let size = f.size();

        // Main layout
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3),  // Header
                Constraint::Min(10),    // Content
                Constraint::Length(3),  // Status
            ])
            .split(size);

        self.draw_header(f, chunks[0]);

        // Content area
        let content_chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Percentage(30),  // Tree
                Constraint::Percentage(50),  // Preview
                Constraint::Percentage(20),  // Info
            ])
            .split(chunks[1]);

        self.draw_tree(f, content_chunks[0]);
        self.draw_preview(f, content_chunks[1]);
        self.draw_info(f, content_chunks[2]);

        self.draw_status_bar(f, chunks[2]);

        if self.show_help {
            self.draw_help_overlay(f, size);
        }
    }

    fn draw_header(&self, f: &mut Frame, area: Rect) {
        let header_text = vec![
            Span::styled(" 🌶️ SPICY ", Style::default().fg(SPICY_ORANGE).bold()),
            Span::styled("TREE ", Style::default().fg(SPICY_GREEN).bold()),
            Span::styled("ENHANCED ", Style::default().fg(SPICY_YELLOW).bold()),
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
            );

        f.render_widget(header, area);
    }

    fn draw_tree(&mut self, f: &mut Frame, area: Rect) {
        // Build items first, then use them
        let items = {
            let mut items = Vec::new();
            self.build_tree_items_into(&self.tree, 0, &mut items);
            items
        };

        let title = match self.search_mode {
            SearchMode::FileName => format!(" 🔍 Files: {} ", self.search_query),
            SearchMode::FileContent => format!(" 🔎 Content: {} ", self.search_query),
            SearchMode::Off => " 🌲 Tree Navigation ".to_string(),
        };

        let list = List::new(items)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(SPICY_BORDER))
                    .title(title)
                    .title_style(Style::default().fg(SPICY_YELLOW).bold()),
            )
            .highlight_style(
                Style::default()
                    .bg(SPICY_GREEN)
                    .fg(Color::Black)
                    .add_modifier(Modifier::BOLD)
            )
            .highlight_symbol("▶ ");

        f.render_stateful_widget(list, area, &mut self.list_state);
    }

    fn build_tree_items_into(&self, node: &TreeNode, indent: usize, items: &mut Vec<ListItem>) {
        let prefix = "  ".repeat(indent);
        let icon = if node.is_dir {
            if node.is_expanded { "📂" } else { "📁" }
        } else {
            self.get_file_icon(&node.path)
        };

        let style = if node.is_dir {
            Style::default().fg(SPICY_CYAN)
        } else {
            Style::default().fg(SPICY_GREEN)
        };

        let arrow = if node.is_dir {
            if node.is_expanded { "▼ " } else { "▶ " }
        } else {
            "  "
        };

        items.push(ListItem::new(format!("{}{}{} {}", prefix, arrow, icon, node.name)).style(style));

        if node.is_expanded {
            for child in &node.children {
                self.build_tree_items_into(child, indent + 1, items);
            }
        }
    }

    fn build_tree_items(&self, node: &TreeNode, indent: usize) -> Vec<ListItem> {
        let mut items = Vec::new();

        let prefix = "  ".repeat(indent);
        let icon = if node.is_dir {
            if node.is_expanded { "📂" } else { "📁" }
        } else {
            self.get_file_icon(&node.path)
        };

        let style = if node.is_dir {
            Style::default().fg(SPICY_CYAN)
        } else {
            Style::default().fg(SPICY_GREEN)
        };

        let arrow = if node.is_dir {
            if node.is_expanded { "▼ " } else { "▶ " }
        } else {
            "  "
        };

        items.push(ListItem::new(format!("{}{}{} {}", prefix, arrow, icon, node.name)).style(style));

        if node.is_expanded {
            for child in &node.children {
                items.extend(self.build_tree_items(child, indent + 1));
            }
        }

        items
    }

    fn draw_preview(&self, f: &mut Frame, area: Rect) {
        let content = self.preview_content.as_deref().unwrap_or("No preview");

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

    fn draw_info(&self, f: &mut Frame, area: Rect) {
        let mut lines = vec![];

        lines.push(Line::from(vec![
            Span::styled("Search History:", Style::default().fg(SPICY_YELLOW).bold()),
        ]));

        for query in self.search_history.iter().rev().take(5) {
            lines.push(Line::from(vec![
                Span::styled(format!("  • {}", query), Style::default().fg(SPICY_CYAN)),
            ]));
        }

        if !self.search_results.is_empty() {
            lines.push(Line::from(""));
            lines.push(Line::from(vec![
                Span::styled("Results:", Style::default().fg(SPICY_YELLOW).bold()),
            ]));
            for result in self.search_results.iter().take(3) {
                lines.push(Line::from(vec![
                    Span::styled(
                        format!("  {}:{}",
                            result.path.file_name().unwrap_or_default().to_string_lossy(),
                            result.line_number
                        ),
                        Style::default().fg(SPICY_GREEN),
                    ),
                ]));
            }
        }

        let info = Paragraph::new(lines).block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(SPICY_BORDER))
                .title(" 📊 Info & M8 ")
                .title_style(Style::default().fg(SPICY_YELLOW).bold()),
        );

        f.render_widget(info, area);
    }

    fn draw_status_bar(&self, f: &mut Frame, area: Rect) {
        let mut spans = vec![
            Span::styled(
                " q:Quit │ /:Files │ ^F:Content │ ←→:Navigate │ Enter:Open ",
                Style::default().fg(SPICY_DARK_GREEN),
            ),
        ];

        if let Some((msg, _)) = &self.status_message {
            spans.push(Span::styled(" │ ", Style::default().fg(SPICY_BORDER)));
            spans.push(Span::styled(msg, Style::default().fg(SPICY_YELLOW)));
        }

        let status = Paragraph::new(Line::from(spans))
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
                "🌶️ ENHANCED SPICY TREE",
                Style::default().fg(SPICY_ORANGE).bold(),
            )]),
            Line::from(""),
            Line::from("🌲 Tree Navigation:"),
            Line::from("  ← / h    - Go up / Collapse"),
            Line::from("  → / l    - Go in / Expand"),
            Line::from("  ↑ / k    - Previous item"),
            Line::from("  ↓ / j    - Next item"),
            Line::from(""),
            Line::from("🔍 Search Modes:"),
            Line::from("  /        - Search file names"),
            Line::from("  Ctrl+F   - Search file contents"),
            Line::from("  Enter    - Execute search & save to M8"),
            Line::from(""),
            Line::from("🖼️ Features:"),
            Line::from("  • ASCII art for images"),
            Line::from("  • Search results saved to M8"),
            Line::from("  • Tree structure navigation"),
            Line::from("  • Highlighted search matches"),
            Line::from(""),
            Line::from("Press any key to close"),
        ];

        let help_area = centered_rect(50, 22, area);

        f.render_widget(Clear, help_area);

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
            );

        f.render_widget(help, help_area);
    }
}

impl Drop for EnhancedSpicyTui {
    fn drop(&mut self) {
        disable_raw_mode().ok();
        if let Some(mut term) = self.terminal.take() {
            execute!(
                term.backend_mut(),
                LeaveAlternateScreen,
                DisableMouseCapture
            ).ok();
            term.show_cursor().ok();
        }
    }
}

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

pub async fn run_enhanced_spicy_tui(path: PathBuf) -> Result<()> {
    let mut app = EnhancedSpicyTui::new(path)?;
    app.run().await
}