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
    digest::{
        DigestFormatter, PqModeFormatter, SmtrCapsuleFormatter, TheBookFormatter,
        UltraBackyardFormatter, ZenoModeFormatter,
    },
    hex::HexFormatter,
    json::JsonFormatter,
    ls::LsFormatter,
    markdown::MarkdownFormatter,
    marqant::MarqantFormatter,
    mermaid::{MermaidFormatter, MermaidStyle},
    projects::ProjectsFormatter,
    quantum::QuantumFormatter,
    // quantum_semantic::QuantumSemanticFormatter, // Removed: requires tree-sitter
    semantic::SemanticFormatter,
    smart::SmartFormatter,
    stats::StatsFormatter,
    summary::SummaryFormatter,
    summary_ai::SummaryAiFormatter,
    tsv::TsvFormatter,
    waste::WasteFormatter,
    Formatter,
    PathDisplayMode,
};
use crate::{parse_size, Scanner, ScannerConfig, TreeStats};
use anyhow::{Context, Result};
use axum::{extract::State, http::StatusCode, response::IntoResponse, Json};
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

    /// Enable security scanning (off by default on regular scans to avoid reading all file contents)
    #[serde(default)]
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
    /// Safety stop reason for partial scans; `None` when scan completed without hitting limits.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub truncation_reason: Option<String>,
}

/// Error response
#[derive(Debug, Serialize)]
pub struct CliErrorResponse {
    pub error: String,
    pub details: Option<String>,
}

/// Execute a CLI scan locally (standalone or shared with the HTTP handler)
pub fn execute_cli_scan(req: &CliScanRequest) -> Result<CliScanResponse> {
    let config = build_scanner_config(req)?;

    // Scanner nodes use canonical paths; formatters need the same root.
    let path = PathBuf::from(&req.path)
        .canonicalize()
        .with_context(|| format!("Cannot resolve scan path {}", req.path))?;

    // Automatically remember the scanned directory without requiring explicit memory initialization.
    if let Ok(mut mgr) = crate::memory_manager::MemoryManager::new() {
        let _ = mgr.ensure_directory_remembered(&path);
    }

    let scanner = Scanner::new(&path, config).context("Failed to create scanner")?;

    let scan_start = Instant::now();
    let (nodes, tree_stats) = scanner.scan().context("Scan failed")?;
    let scan_time = scan_start.elapsed();

    let format_start = Instant::now();
    let path_display = parse_path_mode(&req.path_mode);

    let mut output_buffer = Vec::new();
    format_output(
        req,
        &mut output_buffer,
        &nodes,
        &tree_stats,
        &path,
        path_display,
    )
    .context("Format failed")?;
    let format_time = format_start.elapsed();

    let (output, compressed) = if req.compress {
        let mut encoder = ZlibEncoder::new(Vec::new(), Compression::default());
        encoder.write_all(&output_buffer)?;
        let compressed_data = encoder.finish()?;
        (general_purpose::STANDARD.encode(&compressed_data), true)
    } else {
        let raw_output = String::from_utf8_lossy(&output_buffer).to_string();
        let everything = req.all && req.show_ignored;
        let trimmed = raw_output.trim_start();
        let is_structured_payload = matches!(req.mode.as_str(), "json" | "csv" | "tsv")
            || trimmed.starts_with('{')
            || trimmed.starts_with('[');
        let capped = if !everything && !is_structured_payload && req.top.is_none() {
            cap_output_rows(&raw_output, 100, 20)
        } else {
            raw_output
        };
        (capped, false)
    };

    Ok(CliScanResponse {
        output,
        compressed,
        stats: ScanStats {
            total_files: tree_stats.total_files,
            total_dirs: tree_stats.total_dirs,
            total_size: tree_stats.total_size,
            scan_time_ms: scan_time.as_millis() as u64,
            format_time_ms: format_time.as_millis() as u64,
            truncation_reason: tree_stats.truncation_reason,
        },
    })
}

/// Cap formatted output to `max_rows` lines (keeping the first `max_rows - tail_rows`
/// lines, a `...` elision banner with switch instructions, and the bottom `tail_rows` lines).
pub fn cap_output_rows(output: &str, max_rows: usize, tail_rows: usize) -> String {
    let lines: Vec<&str> = output.lines().collect();
    if lines.len() <= max_rows || max_rows <= tail_rows {
        return output.to_string();
    }
    let head_rows = max_rows - tail_rows;
    let elided = lines.len() - max_rows;
    let mut out = String::with_capacity(output.len().min(16_384));
    for line in &lines[..head_rows] {
        out.push_str(line);
        out.push('\n');
    }
    out.push_str(&format!(
        "...\n   ({elided} rows elided — use -e/--everything, -d/--depth <N>, or -f/--find <pattern> to see more)\n...\n"
    ));
    for line in &lines[lines.len() - tail_rows..] {
        out.push_str(line);
        out.push('\n');
    }
    out
}

/// Handle CLI scan request
pub async fn cli_scan_handler(
    State(state): State<Arc<RwLock<DaemonState>>>,
    Json(req): Json<CliScanRequest>,
) -> Result<impl IntoResponse, (StatusCode, Json<CliErrorResponse>)> {
    let req_for_scan = req.clone();
    let response = tokio::task::spawn_blocking(move || execute_cli_scan(&req_for_scan))
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(CliErrorResponse {
                    error: "Scan task panicked".to_string(),
                    details: Some(e.to_string()),
                }),
            )
        })?
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(CliErrorResponse {
                    error: "Scan failed".to_string(),
                    details: Some(e.to_string()),
                }),
            )
        })?;

    if response.compressed {
        if let Ok(mut state) = state.try_write() {
            state.credits.record_savings(
                response.stats.total_size,
                &format!("CLI scan: {}", req.path),
            );
        }
    }

    Ok(Json(response))
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
    match mode.to_lowercase().replace(['-', '_'], "").as_str() {
        "quantum" | "quantumsemantic" => 6,
        "summary" | "summaryai" | "context" => 4,
        "ai" | "semantic" | "smart" | "stats" => 5,
        "digest" | "louds" | "smtrcapsule" | "capsule" | "thebook" | "erdos" | "szemeredi"
        | "mem8regular" | "book" | "pq" | "rationalpq" | "symplectic" | "ultrabackyard"
        | "backyardultra" | "ultrabackyardloop" | "doloop" | "zeno" | "ultraomnizeno" | "upz"
        | "wupz" | "8bis" | "8bengines" => 8,
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
    let mode = req.mode.to_lowercase().replace(['-', '_'], "");
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
        "louds" | "smtrcapsule" | "capsule" => {
            let formatter = SmtrCapsuleFormatter::new();
            formatter.format(writer, nodes, stats, root_path)?;
        }
        "thebook" | "erdos" | "szemeredi" | "mem8regular" | "book" => {
            let formatter = TheBookFormatter::new();
            formatter.format(writer, nodes, stats, root_path)?;
        }
        "pq" | "rationalpq" | "symplectic" => {
            let formatter = PqModeFormatter::new();
            formatter.format(writer, nodes, stats, root_path)?;
        }
        "ultrabackyard" | "backyardultra" | "ultrabackyardloop" | "doloop" => {
            let formatter = UltraBackyardFormatter::new();
            formatter.format(writer, nodes, stats, root_path)?;
        }
        "zeno" | "ultraomnizeno" | "upz" | "wupz" | "8bis" | "8bengines" => {
            let formatter = ZenoModeFormatter::new();
            formatter.format(writer, nodes, stats, root_path)?;
        }
        "quantum" => {
            let formatter = QuantumFormatter::new();
            formatter.format(writer, nodes, stats, root_path)?;
        }
        "quantumsemantic" => {
            writeln!(writer, "QUANTUM_SEMANTIC_V2:")?;
            let formatter = SemanticFormatter::new(path_display, no_emoji);
            formatter.format(writer, nodes, stats, root_path)?;
        }
        "summary" => {
            let formatter = SummaryFormatter::new(use_color);
            formatter.format(writer, nodes, stats, root_path)?;
        }
        "summaryai" => {
            let formatter = SummaryAiFormatter::new(false);
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
            let formatter = SmartFormatter::new(use_color, !no_emoji).with_path_mode(path_display);
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn summary_and_quantum_modes_route_to_their_formatters() {
        let temp = tempfile::tempdir().unwrap();
        std::fs::write(temp.path().join("notes.md"), "Lunar geology").unwrap();
        for (mode, header) in [
            ("summary-ai", "SUMMARY_AI_V1:"),
            ("summary_ai", "SUMMARY_AI_V1:"),
            ("summaryai", "SUMMARY_AI_V1:"),
            ("quantum-semantic", "QUANTUM_SEMANTIC_V2:"),
            ("louds", "\"spec\": \"8B-META-WP-v2.1\""),
            ("smtr-capsule", "\"spec\": \"8B-META-WP-v2.1\""),
            ("smtr_capsule", "\"spec\": \"8B-META-WP-v2.1\""),
            ("capsule", "\"spec\": \"8B-META-WP-v2.1\""),
            ("the-book", "\"spec\": \"ERDOS-THE-BOOK-v1.0\""),
            ("erdos", "\"spec\": \"ERDOS-THE-BOOK-v1.0\""),
            ("szemeredi", "\"spec\": \"ERDOS-THE-BOOK-v1.0\""),
            ("pq", "\"spec\": \"SOVEREIGN-PQ-MODE-v1.0\""),
            ("p-q", "\"spec\": \"SOVEREIGN-PQ-MODE-v1.0\""),
            ("symplectic", "\"spec\": \"SOVEREIGN-PQ-MODE-v1.0\""),
            ("ultrabackyard", "\"spec\": \"ULTRABACKYARDLOOP-DO-v1.0\""),
            ("do-loop", "\"spec\": \"ULTRABACKYARDLOOP-DO-v1.0\""),
            ("zeno", "\"spec\": \"ULTRA-OMNI-ZENO-v11.0\""),
            ("upz", "\"spec\": \"ULTRA-OMNI-ZENO-v11.0\""),
            ("wupz", "\"spec\": \"ULTRA-OMNI-ZENO-v11.0\""),
            ("8b-engines", "\"spec\": \"ULTRA-OMNI-ZENO-v11.0\""),
        ] {
            let request = serde_json::from_value(serde_json::json!({
                "path": temp.path(), "mode": mode
            }))
            .unwrap();
            let output = execute_cli_scan(&request).unwrap().output;
            assert!(
                output.starts_with(header) || output.contains(header),
                "mode {mode} did not contain {header}: {output}"
            );
        }
    }

    #[cfg(unix)]
    #[test]
    fn symlinked_roots_render_their_children() {
        // Scanner intentionally prunes tmpfs; use the checkout filesystem.
        let temp = tempfile::tempdir_in(env!("CARGO_MANIFEST_DIR")).unwrap();
        let actual = temp.path().join("actual");
        std::fs::create_dir(&actual).unwrap();
        std::fs::write(actual.join("notes.md"), "Lunar geology").unwrap();
        let alias = temp.path().join("alias");
        std::os::unix::fs::symlink(&actual, &alias).unwrap();
        let request = serde_json::from_value(serde_json::json!({
            "path": alias, "mode": "classic"
        }))
        .unwrap();
        assert!(execute_cli_scan(&request)
            .unwrap()
            .output
            .contains("notes.md"));
    }

    #[test]
    fn cap_output_rows_limits_to_80_head_and_20_tail() {
        let mut big = String::new();
        for i in 1..=150 {
            big.push_str(&format!("row-{i}\n"));
        }
        let capped = cap_output_rows(&big, 100, 20);
        assert!(capped.contains("row-1\n"));
        assert!(capped.contains("row-80\n"));
        assert!(!capped.contains("row-81\n"));
        assert!(capped.contains("(50 rows elided"));
        assert!(!capped.contains("row-130\n"));
        assert!(capped.contains("row-131\n"));
        assert!(capped.contains("row-150\n"));
    }
}
