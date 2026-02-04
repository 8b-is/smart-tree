//! Daemon CLI Handlers - HTTP endpoints for thin-client CLI operations
//!
//! This module provides the daemon endpoints that the `st` thin client
//! calls to perform scanning and formatting operations. All the heavy
//! lifting happens here in the daemon.
//!
//! "The meat stays in the daemon!" - Hue

use crate::formatters::{
    ai::AiFormatter,
    classic::ClassicFormatter,
    csv::CsvFormatter,
    digest::DigestFormatter,
    hex::HexFormatter,
    json::JsonFormatter,
    ls::LsFormatter,
    markdown::MarkdownFormatter,
    marqant::MarqantFormatter,
    mermaid::{MermaidFormatter, MermaidStyle},
    projects::ProjectsFormatter,
    quantum::QuantumFormatter,
    semantic::SemanticFormatter,
    smart::SmartFormatter,
    stats::StatsFormatter,
    tsv::TsvFormatter,
    waste::WasteFormatter,
    Formatter, PathDisplayMode,
};
use crate::{parse_size, Scanner, ScannerConfig, TreeStats};
use anyhow::{Context, Result};
use axum::{
    extract::State,
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use base64::{engine::general_purpose, Engine as _};
use flate2::write::ZlibEncoder;
use flate2::Compression;
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::io::Write;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Instant;
use tokio::sync::RwLock;

use crate::daemon::DaemonState;

/// CLI scan request - all options from the CLI
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CliScanRequest {
    /// Path to scan (required)
    pub path: String,

    /// Output mode (classic, ai, quantum, json, etc.)
    #[serde(default = "default_mode")]
    pub mode: String,

    /// Max depth (0 = auto based on mode)
    #[serde(default)]
    pub depth: usize,

    /// Show hidden files
    #[serde(default)]
    pub all: bool,

    /// Respect .gitignore
    #[serde(default = "default_true")]
    pub respect_gitignore: bool,

    /// Use default ignores (node_modules, etc.)
    #[serde(default = "default_true")]
    pub default_ignores: bool,

    /// Show ignored entries
    #[serde(default)]
    pub show_ignored: bool,

    /// Find pattern (regex for filename matching)
    pub find: Option<String>,

    /// File type filter (e.g., "rs", "py")
    pub file_type: Option<String>,

    /// Entry type filter ("f" for files, "d" for directories)
    pub entry_type: Option<String>,

    /// Min file size (e.g., "1M", "500K")
    pub min_size: Option<String>,

    /// Max file size
    pub max_size: Option<String>,

    /// Sort field (name, size, date, type)
    pub sort: Option<String>,

    /// Top N results (used with sort)
    pub top: Option<usize>,

    /// Search content keyword
    pub search: Option<String>,

    /// Enable zlib compression on output
    #[serde(default)]
    pub compress: bool,

    /// No emoji in output
    #[serde(default)]
    pub no_emoji: bool,

    /// Use color in output
    #[serde(default = "default_true")]
    pub use_color: bool,

    /// Path display mode (off, relative, full)
    #[serde(default = "default_path_mode")]
    pub path_mode: String,

    /// Focus file (for relations mode)
    pub focus: Option<String>,

    /// Relations filter
    pub relations_filter: Option<String>,

    /// Show filesystem type indicators
    #[serde(default)]
    pub show_filesystems: bool,

    /// Include line content in search results
    #[serde(default)]
    pub include_line_content: bool,

    /// Compact JSON output
    #[serde(default)]
    pub compact: bool,

    // --- Smart Scanning Options (Phase 2: Intelligent Context-Aware Scanning) ---

    /// Enable smart mode - groups by interest, shows changes, minimal output
    #[serde(default)]
    pub smart: bool,

    /// Only show changes since last scan
    #[serde(default)]
    pub changes_only: bool,

    /// Minimum interest level to show (0.0-1.0)
    #[serde(default)]
    pub min_interest: f32,

    /// Enable security scanning
    #[serde(default = "default_true")]
    pub security: bool,
}

fn default_mode() -> String {
    "classic".to_string()
}

fn default_true() -> bool {
    true
}

fn default_path_mode() -> String {
    "relative".to_string()
}

/// CLI scan response
#[derive(Debug, Serialize, Deserialize)]
pub struct CliScanResponse {
    /// Formatted output (ready to print)
    pub output: String,

    /// Was output compressed?
    pub compressed: bool,

    /// Stats about the scan
    pub stats: ScanStats,
}

/// Scan statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScanStats {
    pub total_files: u64,
    pub total_dirs: u64,
    pub total_size: u64,
    pub scan_time_ms: u64,
    pub format_time_ms: u64,
}

/// Error response
#[derive(Debug, Serialize)]
pub struct CliErrorResponse {
    pub error: String,
    pub details: Option<String>,
}

/// Handle CLI scan request
pub async fn cli_scan_handler(
    State(state): State<Arc<RwLock<DaemonState>>>,
    Json(req): Json<CliScanRequest>,
) -> Result<impl IntoResponse, (StatusCode, Json<CliErrorResponse>)> {
    // Build scanner config from request
    let config = build_scanner_config(&req).map_err(|e| {
        (
            StatusCode::BAD_REQUEST,
            Json(CliErrorResponse {
                error: "Invalid request".to_string(),
                details: Some(e.to_string()),
            }),
        )
    })?;

    // Resolve path
    let path = PathBuf::from(&req.path);
    let path = if path.is_absolute() {
        path
    } else {
        std::env::current_dir()
            .unwrap_or_else(|_| PathBuf::from("."))
            .join(&path)
    };

    // Create scanner and scan
    let scanner = Scanner::new(&path, config).map_err(|e| {
        (
            StatusCode::BAD_REQUEST,
            Json(CliErrorResponse {
                error: "Failed to create scanner".to_string(),
                details: Some(e.to_string()),
            }),
        )
    })?;

    let scan_start = Instant::now();
    let (nodes, tree_stats) = scanner.scan().map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(CliErrorResponse {
                error: "Scan failed".to_string(),
                details: Some(e.to_string()),
            }),
        )
    })?;
    let scan_time = scan_start.elapsed();

    // Select formatter and format output
    let format_start = Instant::now();
    let path_display = parse_path_mode(&req.path_mode);

    let mut output_buffer = Vec::new();
    format_output(&req, &mut output_buffer, &nodes, &tree_stats, &path, path_display).map_err(
        |e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(CliErrorResponse {
                    error: "Format failed".to_string(),
                    details: Some(e.to_string()),
                }),
            )
        },
    )?;
    let format_time = format_start.elapsed();

    // Optionally compress
    let (output, compressed) = if req.compress {
        let mut encoder = ZlibEncoder::new(Vec::new(), Compression::default());
        encoder.write_all(&output_buffer).map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(CliErrorResponse {
                    error: "Compression failed".to_string(),
                    details: Some(e.to_string()),
                }),
            )
        })?;
        let compressed_data = encoder.finish().map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(CliErrorResponse {
                    error: "Compression failed".to_string(),
                    details: Some(e.to_string()),
                }),
            )
        })?;
        (general_purpose::STANDARD.encode(&compressed_data), true)
    } else {
        (
            String::from_utf8_lossy(&output_buffer).to_string(),
            false,
        )
    };

    // Build stats
    let stats = ScanStats {
        total_files: tree_stats.total_files,
        total_dirs: tree_stats.total_dirs,
        total_size: tree_stats.total_size,
        scan_time_ms: scan_time.as_millis() as u64,
        format_time_ms: format_time.as_millis() as u64,
    };

    // Record token savings in daemon state (if compressed)
    if compressed {
        let savings = output_buffer.len().saturating_sub(output.len()) as u64;
        if let Ok(mut state) = state.try_write() {
            state
                .credits
                .record_savings(savings, &format!("CLI scan: {}", req.path));
        }
    }

    Ok(Json(CliScanResponse {
        output,
        compressed,
        stats,
    }))
}

/// Handle streaming CLI scan request (SSE) - simplified version
/// For now, this just returns the full response as JSON
/// TODO: Implement proper SSE streaming
pub async fn cli_stream_handler(
    State(state): State<Arc<RwLock<DaemonState>>>,
    Json(req): Json<CliScanRequest>,
) -> Result<impl IntoResponse, (StatusCode, Json<CliErrorResponse>)> {
    // For now, just use the regular handler
    // Real SSE streaming can be added later
    cli_scan_handler(State(state), Json(req)).await
}

/// Build ScannerConfig from CliScanRequest
fn build_scanner_config(req: &CliScanRequest) -> Result<ScannerConfig> {
    let find_pattern = if let Some(ref pattern) = req.find {
        Some(Regex::new(pattern).context("Invalid find pattern regex")?)
    } else {
        None
    };

    let min_size = if let Some(ref s) = req.min_size {
        Some(parse_size(s).context("Invalid min_size")?)
    } else {
        None
    };

    let max_size = if let Some(ref s) = req.max_size {
        Some(parse_size(s).context("Invalid max_size")?)
    } else {
        None
    };

    // Determine depth based on mode if not specified
    let max_depth = if req.depth == 0 {
        get_ideal_depth_for_mode(&req.mode)
    } else {
        req.depth
    };

    Ok(ScannerConfig {
        max_depth,
        follow_symlinks: false,
        respect_gitignore: req.respect_gitignore,
        show_hidden: req.all,
        show_ignored: req.show_ignored,
        find_pattern,
        file_type_filter: req.file_type.clone(),
        entry_type_filter: req.entry_type.clone(),
        min_size,
        max_size,
        newer_than: None, // TODO: parse date strings
        older_than: None,
        use_default_ignores: req.default_ignores,
        search_keyword: req.search.clone(),
        show_filesystems: req.show_filesystems,
        sort_field: req.sort.clone(),
        top_n: req.top,
        include_line_content: req.include_line_content,
        // Smart scanning options
        compute_interest: req.smart,
        security_scan: req.security,
        min_interest: req.min_interest,
        track_traversal: req.smart,
        changes_only: req.changes_only,
        compare_state: None,
        smart_mode: req.smart,
    })
}

/// Get ideal depth for a given mode
fn get_ideal_depth_for_mode(mode: &str) -> usize {
    match mode.to_lowercase().as_str() {
        "quantum" | "quantum_semantic" => 10,
        "ai" | "semantic" | "smart" => 5,
        "digest" | "stats" => 20,
        "relations" => 3,
        "projects" => 5,
        _ => 3, // Default for classic, json, etc.
    }
}

/// Parse path display mode
fn parse_path_mode(mode: &str) -> PathDisplayMode {
    match mode.to_lowercase().as_str() {
        "off" | "none" => PathDisplayMode::Off,
        "full" | "absolute" => PathDisplayMode::Full,
        _ => PathDisplayMode::Relative,
    }
}

/// Format output using the appropriate formatter
fn format_output(
    req: &CliScanRequest,
    writer: &mut dyn Write,
    nodes: &[crate::FileNode],
    stats: &TreeStats,
    root_path: &std::path::Path,
    path_display: PathDisplayMode,
) -> Result<()> {
    let mode = req.mode.to_lowercase();
    let no_emoji = req.no_emoji;
    let use_color = req.use_color;

    match mode.as_str() {
        "classic" => {
            let formatter = ClassicFormatter::new(no_emoji, use_color, path_display);
            formatter.format(writer, nodes, stats, root_path)?;
        }
        "hex" => {
            let formatter = HexFormatter::new(
                use_color,
                no_emoji,
                req.show_ignored,
                path_display,
                req.show_filesystems,
            );
            formatter.format(writer, nodes, stats, root_path)?;
        }
        "json" => {
            let formatter = JsonFormatter::new(req.compact);
            formatter.format(writer, nodes, stats, root_path)?;
        }
        "ls" => {
            let formatter = LsFormatter::new(!no_emoji, use_color);
            formatter.format(writer, nodes, stats, root_path)?;
        }
        "ai" => {
            let formatter = AiFormatter::new(no_emoji, path_display);
            formatter.format(writer, nodes, stats, root_path)?;
        }
        "stats" => {
            let formatter = StatsFormatter::new();
            formatter.format(writer, nodes, stats, root_path)?;
        }
        "csv" => {
            let formatter = CsvFormatter::new();
            formatter.format(writer, nodes, stats, root_path)?;
        }
        "tsv" => {
            let formatter = TsvFormatter::new();
            formatter.format(writer, nodes, stats, root_path)?;
        }
        "digest" => {
            let formatter = DigestFormatter::new();
            formatter.format(writer, nodes, stats, root_path)?;
        }
        "quantum" => {
            let formatter = QuantumFormatter::new();
            formatter.format(writer, nodes, stats, root_path)?;
        }
        "semantic" => {
            let formatter = SemanticFormatter::new(path_display, no_emoji);
            formatter.format(writer, nodes, stats, root_path)?;
        }
        "projects" => {
            let formatter = ProjectsFormatter::new();
            formatter.format(writer, nodes, stats, root_path)?;
        }
        "mermaid" => {
            let formatter = MermaidFormatter::new(MermaidStyle::Flowchart, no_emoji, path_display);
            formatter.format(writer, nodes, stats, root_path)?;
        }
        "markdown" => {
            let formatter = MarkdownFormatter::new(path_display, no_emoji, true, true, true);
            formatter.format(writer, nodes, stats, root_path)?;
        }
        "waste" => {
            let formatter = WasteFormatter::new();
            formatter.format(writer, nodes, stats, root_path)?;
        }
        "marqant" => {
            let formatter = MarqantFormatter::new(path_display, no_emoji);
            formatter.format(writer, nodes, stats, root_path)?;
        }
        "smart" => {
            // The star of the show! Surface what matters, not everything.
            let formatter = SmartFormatter::new(use_color, !no_emoji)
                .with_path_mode(path_display);
            formatter.format(writer, nodes, stats, root_path)?;
        }
        // Default to classic for unknown modes
        _ => {
            let formatter = ClassicFormatter::new(no_emoji, use_color, path_display);
            formatter.format(writer, nodes, stats, root_path)?;
        }
    }

    Ok(())
}
