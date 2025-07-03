//! Interactive mode for Smart Tree
//! 
//! This module provides an interactive terminal UI for exploring directories
//! with various visualization modes and filters.

use anyhow::Result;
use inquire::{Select, MultiSelect, Text, Confirm, InquireError};
use std::path::PathBuf;
use colored::Colorize;

use crate::{
    Scanner, ScannerConfig, TreeStats,
    scanner::FileNode,
    formatters::{Formatter, PathDisplayMode},
    content_detector::{ContentDetector, DirectoryType},
};

/// Main interactive menu options
#[derive(Debug, Clone)]
enum MainAction {
    ShowTree,
    ChangeView,
    FilterFiles,
    SearchContent,
    AnalyzeRelations,
    ExportData,
    ChangeDirectory,
    Quit,
}

impl std::fmt::Display for MainAction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MainAction::ShowTree => write!(f, "📁 Show directory tree"),
            MainAction::ChangeView => write!(f, "👁️  Change view mode"),
            MainAction::FilterFiles => write!(f, "🔍 Filter files"),
            MainAction::SearchContent => write!(f, "🔎 Search in file contents"),
            MainAction::AnalyzeRelations => write!(f, "🔗 Analyze code relationships"),
            MainAction::ExportData => write!(f, "💾 Export for AI/documentation"),
            MainAction::ChangeDirectory => write!(f, "📂 Change directory"),
            MainAction::Quit => write!(f, "👋 Quit"),
        }
    }
}

/// View modes available in interactive mode
#[derive(Debug, Clone)]
enum ViewMode {
    Classic,
    Summary,
    Semantic,
    Relations,
    Mermaid,
    Markdown,
}

impl std::fmt::Display for ViewMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ViewMode::Classic => write!(f, "Classic tree view"),
            ViewMode::Summary => write!(f, "Summary overview"),
            ViewMode::Semantic => write!(f, "Semantic grouping"),
            ViewMode::Relations => write!(f, "Code relationships"),
            ViewMode::Mermaid => write!(f, "Mermaid diagram"),
            ViewMode::Markdown => write!(f, "Markdown report"),
        }
    }
}

/// Export format options
#[derive(Debug, Clone)]
enum ExportFormat {
    Ai,
    Quantum,
    QuantumSemantic,
    Json,
    Csv,
}

impl std::fmt::Display for ExportFormat {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ExportFormat::Ai => write!(f, "AI-optimized format"),
            ExportFormat::Quantum => write!(f, "Quantum compressed (90%+ compression)"),
            ExportFormat::QuantumSemantic => write!(f, "Quantum semantic (with AST analysis)"),
            ExportFormat::Json => write!(f, "JSON format"),
            ExportFormat::Csv => write!(f, "CSV spreadsheet"),
        }
    }
}

pub struct InteractiveMode {
    current_path: PathBuf,
    current_view: ViewMode,
    filter_extensions: Vec<String>,
    search_keyword: Option<String>,
    show_hidden: bool,
    max_depth: usize,
}

impl InteractiveMode {
    pub fn new(initial_path: PathBuf) -> Self {
        Self {
            current_path: initial_path,
            current_view: ViewMode::Summary,
            filter_extensions: Vec::new(),
            search_keyword: None,
            show_hidden: false,
            max_depth: 5,
        }
    }

    /// Run the interactive mode main loop
    pub async fn run(&mut self) -> Result<()> {
        loop {
            // Clear screen and show header
            print!("\x1B[2J\x1B[1;1H"); // Clear screen
            self.show_header()?;
            
            // Show current state
            self.show_current_state()?;
            
            // Show main menu
            let action = self.show_main_menu()?;
            
            match action {
                MainAction::ShowTree => self.show_tree().await?,
                MainAction::ChangeView => self.change_view()?,
                MainAction::FilterFiles => self.filter_files()?,
                MainAction::SearchContent => self.search_content()?,
                MainAction::AnalyzeRelations => self.analyze_relations().await?,
                MainAction::ExportData => self.export_data().await?,
                MainAction::ChangeDirectory => self.change_directory()?,
                MainAction::Quit => {
                    println!("\n{}", "Thanks for using Smart Tree! 🌳✨".green());
                    break;
                }
            }
            
            // Wait for user to continue
            if !matches!(action, MainAction::Quit) {
                println!("\n{}", "Press Enter to continue...".dimmed());
                let _ = std::io::stdin().read_line(&mut String::new());
            }
        }
        
        Ok(())
    }

    fn show_header(&self) -> Result<()> {
        println!("{}", "═".repeat(80).blue());
        println!("{}", "Smart Tree Interactive Mode 🌳".bold().green());
        println!("{}", "═".repeat(80).blue());
        Ok(())
    }

    fn show_current_state(&self) -> Result<()> {
        println!("\n{}", "Current State:".bold());
        println!("  📍 Path: {}", self.current_path.display().to_string().cyan());
        println!("  👁️  View: {}", format!("{:?}", self.current_view).yellow());
        
        if !self.filter_extensions.is_empty() {
            println!("  🔍 Filters: {}", self.filter_extensions.join(", ").magenta());
        }
        
        if let Some(ref keyword) = self.search_keyword {
            println!("  🔎 Search: {}", keyword.red());
        }
        
        println!("  📊 Depth: {}", self.max_depth);
        println!("  👻 Hidden: {}", if self.show_hidden { "shown" } else { "hidden" });
        
        // Quick content detection
        if let Ok((nodes, _)) = self.scan_directory() {
            match ContentDetector::detect(&nodes, &self.current_path) {
                DirectoryType::CodeProject { language, .. } => {
                    println!("\n  🚀 Detected: {} project", format!("{:?}", language).green());
                }
                DirectoryType::MediaLibrary { .. } => {
                    println!("\n  🎬 Detected: Media library");
                }
                DirectoryType::DocumentArchive { .. } => {
                    println!("\n  📚 Detected: Document archive");
                }
                DirectoryType::MixedContent { .. } => {
                    println!("\n  📦 Detected: Mixed content");
                }
                DirectoryType::PhotoCollection { .. } => {
                    println!("\n  📷 Detected: Photo collection");
                }
                DirectoryType::DataScience { .. } => {
                    println!("\n  🔬 Detected: Data science workspace");
                }
            }
        }
        
        Ok(())
    }

    fn show_main_menu(&self) -> Result<MainAction> {
        let options = vec![
            MainAction::ShowTree,
            MainAction::ChangeView,
            MainAction::FilterFiles,
            MainAction::SearchContent,
            MainAction::AnalyzeRelations,
            MainAction::ExportData,
            MainAction::ChangeDirectory,
            MainAction::Quit,
        ];

        let selection = Select::new("\nWhat would you like to do?", options)
            .with_help_message("Use arrow keys to navigate, Enter to select")
            .prompt();

        match selection {
            Ok(action) => Ok(action),
            Err(InquireError::OperationCanceled) => Ok(MainAction::Quit),
            Err(e) => Err(e.into()),
        }
    }

    async fn show_tree(&self) -> Result<()> {
        println!("\n{}", "Generating tree view...".dimmed());
        
        let (nodes, stats) = self.scan_directory()?;
        
        // Create appropriate formatter based on view mode
        let formatter = self.create_formatter();
        
        // Format and display
        let mut output = Vec::new();
        formatter.format(&mut output, &nodes, &stats, &self.current_path)?;
        
        println!("\n{}", String::from_utf8_lossy(&output));
        
        Ok(())
    }

    fn change_view(&mut self) -> Result<()> {
        let options = vec![
            ViewMode::Classic,
            ViewMode::Summary,
            ViewMode::Semantic,
            ViewMode::Relations,
            ViewMode::Mermaid,
            ViewMode::Markdown,
        ];

        let selection = Select::new("Select view mode:", options)
            .with_starting_cursor(match self.current_view {
                ViewMode::Classic => 0,
                ViewMode::Summary => 1,
                ViewMode::Semantic => 2,
                ViewMode::Relations => 3,
                ViewMode::Mermaid => 4,
                ViewMode::Markdown => 5,
            })
            .prompt()?;

        self.current_view = selection;
        println!("\n✅ View mode changed to: {}", format!("{:?}", self.current_view).green());
        
        Ok(())
    }

    fn filter_files(&mut self) -> Result<()> {
        let common_extensions = vec![
            "rs", "py", "js", "ts", "go", "java", "cpp", "c", "h",
            "md", "txt", "json", "yaml", "toml", "xml",
            "jpg", "png", "gif", "mp4", "mp3",
        ];

        let selected = MultiSelect::new("Select file extensions to show:", common_extensions)
            .with_help_message("Space to select/deselect, Enter to confirm")
            .prompt()?;

        self.filter_extensions = selected.into_iter().map(|s| s.to_string()).collect();
        
        if self.filter_extensions.is_empty() {
            println!("\n✅ Filters cleared - showing all files");
        } else {
            println!("\n✅ Filtering by: {}", self.filter_extensions.join(", ").green());
        }
        
        Ok(())
    }

    fn search_content(&mut self) -> Result<()> {
        let keyword = Text::new("Enter search keyword:")
            .with_placeholder("e.g., TODO, FIXME, function_name")
            .prompt()?;

        if keyword.is_empty() {
            self.search_keyword = None;
            println!("\n✅ Search cleared");
        } else {
            self.search_keyword = Some(keyword.clone());
            println!("\n✅ Searching for: {}", keyword.red());
        }
        
        Ok(())
    }

    async fn analyze_relations(&self) -> Result<()> {
        println!("\n{}", "Analyzing code relationships...".dimmed());
        
        // This would use the relations formatter
        let (nodes, stats) = self.scan_directory()?;
        
        use crate::formatters::relations_formatter::RelationsFormatter;
        let formatter = RelationsFormatter::new(None, None);
        
        let mut output = Vec::new();
        formatter.format(&mut output, &nodes, &stats, &self.current_path)?;
        
        println!("\n{}", String::from_utf8_lossy(&output));
        
        Ok(())
    }

    async fn export_data(&self) -> Result<()> {
        let formats = vec![
            ExportFormat::Ai,
            ExportFormat::Quantum,
            ExportFormat::QuantumSemantic,
            ExportFormat::Json,
            ExportFormat::Csv,
        ];

        let format = Select::new("Select export format:", formats).prompt()?;
        
        let compress = Confirm::new("Compress output?")
            .with_default(matches!(format, ExportFormat::Ai | ExportFormat::Quantum | ExportFormat::QuantumSemantic))
            .prompt()?;

        let filename = Text::new("Output filename:")
            .with_default(match format {
                ExportFormat::Ai => "tree_ai.txt",
                ExportFormat::Quantum => "tree_quantum.bin",
                ExportFormat::QuantumSemantic => "tree_quantum_semantic.bin",
                ExportFormat::Json => "tree.json",
                ExportFormat::Csv => "tree.csv",
            })
            .prompt()?;

        println!("\n{}", "Exporting...".dimmed());
        
        // Scan and format
        let (nodes, stats) = self.scan_directory()?;
        let formatter = self.create_export_formatter(&format);
        
        let mut output = Vec::new();
        formatter.format(&mut output, &nodes, &stats, &self.current_path)?;
        
        // Optionally compress
        let final_output = if compress {
            use flate2::write::ZlibEncoder;
            use flate2::Compression;
            use std::io::Write;
            
            let mut encoder = ZlibEncoder::new(Vec::new(), Compression::default());
            encoder.write_all(&output)?;
            encoder.finish()?
        } else {
            output
        };
        
        // Write to file
        std::fs::write(&filename, final_output)?;
        
        println!("\n✅ Exported to: {}", filename.green());
        println!("   Size: {}", humansize::format_size(std::fs::metadata(&filename)?.len(), humansize::BINARY));
        
        Ok(())
    }

    fn change_directory(&mut self) -> Result<()> {
        let new_path = Text::new("Enter new directory path:")
            .with_default(".")
            .with_placeholder("/path/to/directory")
            .prompt()?;

        let path = PathBuf::from(new_path);
        if path.exists() && path.is_dir() {
            self.current_path = path.canonicalize()?;
            println!("\n✅ Changed to: {}", self.current_path.display().to_string().green());
        } else {
            println!("\n❌ Invalid directory path");
        }
        
        Ok(())
    }

    fn scan_directory(&self) -> Result<(Vec<FileNode>, TreeStats)> {
        let config = ScannerConfig {
            max_depth: self.max_depth,
            follow_symlinks: false,
            respect_gitignore: true,
            show_hidden: self.show_hidden,
            show_ignored: false,
            find_pattern: None,
            file_type_filter: if self.filter_extensions.is_empty() {
                None
            } else {
                Some(self.filter_extensions.join("|"))
            },
            min_size: None,
            max_size: None,
            newer_than: None,
            older_than: None,
            use_default_ignores: true,  // This should skip .venv, node_modules, etc.
            search_keyword: self.search_keyword.clone(),
            show_filesystems: false,
        };

        let scanner = Scanner::new(&self.current_path, config)?;
        scanner.scan()
    }

    fn create_formatter(&self) -> Box<dyn Formatter> {
        use crate::formatters::{
            classic::ClassicFormatter,
            summary::SummaryFormatter,
            semantic::SemanticFormatter,
            relations_formatter::RelationsFormatter,
            mermaid::MermaidFormatter,
            markdown::MarkdownFormatter,
        };

        match self.current_view {
            ViewMode::Classic => Box::new(ClassicFormatter::new(false, true, PathDisplayMode::Off)),
            ViewMode::Summary => Box::new(SummaryFormatter::new(true)),
            ViewMode::Semantic => Box::new(SemanticFormatter::new(PathDisplayMode::Off, false)),
            ViewMode::Relations => Box::new(RelationsFormatter::new(None, None)),
            ViewMode::Mermaid => Box::new(MermaidFormatter::new(
                crate::formatters::mermaid::MermaidStyle::Flowchart,
                false,
                PathDisplayMode::Off,
            )),
            ViewMode::Markdown => Box::new(MarkdownFormatter::new(
                PathDisplayMode::Off,
                true,
                true,
                true,
                false,
            )),
        }
    }

    fn create_export_formatter(&self, format: &ExportFormat) -> Box<dyn Formatter> {
        use crate::formatters::{
            ai::AiFormatter,
            quantum::QuantumFormatter,
            quantum_semantic::QuantumSemanticFormatter,
            json::JsonFormatter,
            csv::CsvFormatter,
        };

        match format {
            ExportFormat::Ai => Box::new(AiFormatter::new(true, PathDisplayMode::Off)),
            ExportFormat::Quantum => Box::new(QuantumFormatter::new()),
            ExportFormat::QuantumSemantic => Box::new(QuantumSemanticFormatter::new()),
            ExportFormat::Json => Box::new(JsonFormatter::new(false)),
            ExportFormat::Csv => Box::new(CsvFormatter::new()),
        }
    }
}