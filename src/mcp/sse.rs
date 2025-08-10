//! Server-Sent Events (SSE) support for MCP server
//!
//! Provides real-time streaming of directory changes and analysis results

use anyhow::Result;
use futures_util::StreamExt;
use notify::{Config, Event, RecommendedWatcher, RecursiveMode, Watcher};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::mpsc;
use tokio::time::interval;

use super::{is_path_allowed, McpContext};
use crate::formatters::{ai::AiFormatter, hex::HexFormatter, quantum::QuantumFormatter, Formatter};
use crate::scanner::{FileNode, Scanner, ScannerConfig};

/// SSE event types
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum SseEvent {
    /// Initial scan complete
    ScanComplete { path: String, stats: ScanStats },
    /// File or directory created
    Created { path: String, node: FileNode },
    /// File or directory modified
    Modified { path: String, node: FileNode },
    /// File or directory deleted
    Deleted { path: String },
    /// Directory analysis update
    Analysis {
        path: String,
        format: String,
        data: String,
    },
    /// Periodic statistics update
    Stats { path: String, stats: ScanStats },
    /// Error occurred
    Error { message: String },
    /// Heartbeat to keep connection alive
    Heartbeat,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScanStats {
    pub total_files: u64,
    pub total_dirs: u64,
    pub total_size: u64,
    pub scan_time_ms: u64,
}

/// SSE stream configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SseConfig {
    /// Path to watch
    pub path: PathBuf,
    /// Output format for analysis
    pub format: OutputFormat,
    /// Send heartbeat every N seconds
    pub heartbeat_interval: u64,
    /// Send stats update every N seconds
    pub stats_interval: u64,
    /// Include file contents in events
    pub include_content: bool,
    /// Maximum depth for recursive watching
    pub max_depth: Option<usize>,
    /// File patterns to include
    pub include_patterns: Vec<String>,
    /// File patterns to exclude
    pub exclude_patterns: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OutputFormat {
    Hex,
    Ai,
    Quantum,
    QuantumSemantic,
    Json,
    Summary,
}

impl Default for SseConfig {
    fn default() -> Self {
        Self {
            path: PathBuf::from("."),
            format: OutputFormat::Ai,
            heartbeat_interval: 30,
            stats_interval: 60,
            include_content: false,
            max_depth: None,
            include_patterns: vec![],
            exclude_patterns: vec![],
        }
    }
}

/// Handle SSE stream request
#[allow(dead_code)]
pub async fn handle_sse_stream(
    config: SseConfig,
    ctx: Arc<McpContext>,
) -> Result<impl futures_util::Stream<Item = Result<SseEvent>>> {
    // Validate path
    if !is_path_allowed(&config.path, &ctx.config) {
        anyhow::bail!("Path not allowed: {:?}", config.path);
    }

    let (tx, rx) = mpsc::channel::<SseEvent>(100);

    // Spawn watcher task
    let watcher_tx = tx.clone();
    let watcher_config = config.clone();
    let watcher_ctx = ctx.clone();
    tokio::spawn(async move {
        if let Err(e) = watch_directory(watcher_config, watcher_ctx, watcher_tx).await {
            eprintln!("Watcher error: {}", e);
        }
    });

    // Spawn heartbeat task
    let heartbeat_tx = tx.clone();
    let heartbeat_interval = config.heartbeat_interval;
    tokio::spawn(async move {
        let mut interval = interval(Duration::from_secs(heartbeat_interval));
        loop {
            interval.tick().await;
            if heartbeat_tx.send(SseEvent::Heartbeat).await.is_err() {
                break;
            }
        }
    });

    // Spawn stats task
    let stats_tx = tx;
    let stats_config = config.clone();
    let stats_interval = config.stats_interval;
    tokio::spawn(async move {
        let mut interval = interval(Duration::from_secs(stats_interval));
        loop {
            interval.tick().await;
            if let Ok(stats) = gather_stats(&stats_config.path).await {
                let event = SseEvent::Stats {
                    path: stats_config.path.display().to_string(),
                    stats,
                };
                if stats_tx.send(event).await.is_err() {
                    break;
                }
            }
        }
    });

    // Create stream from receiver
    Ok(tokio_stream::wrappers::ReceiverStream::new(rx).map(Ok))
}

/// Watch directory for changes
#[allow(dead_code)]
async fn watch_directory(
    config: SseConfig,
    _ctx: Arc<McpContext>,
    tx: mpsc::Sender<SseEvent>,
) -> Result<()> {
    // Initial scan
    let scanner_config = ScannerConfig {
        max_depth: config.max_depth.unwrap_or(usize::MAX),
        show_hidden: false,
        follow_symlinks: false,
        show_ignored: false,
        search_keyword: None,
        file_type_filter: None,
        ..Default::default()
    };

    let scanner = Scanner::new(&config.path, scanner_config)?;
    let start = std::time::Instant::now();
    let (nodes, stats) = scanner.scan()?;
    let scan_time_ms = start.elapsed().as_millis() as u64;

    // Send initial scan complete event
    tx.send(SseEvent::ScanComplete {
        path: config.path.display().to_string(),
        stats: ScanStats {
            total_files: stats.total_files,
            total_dirs: stats.total_dirs,
            total_size: stats.total_size,
            scan_time_ms,
        },
    })
    .await?;

    // Send initial analysis
    if let Ok(analysis) = format_nodes(&nodes, &stats, &config.path, &config.format) {
        tx.send(SseEvent::Analysis {
            path: config.path.display().to_string(),
            format: format!("{:?}", config.format),
            data: analysis,
        })
        .await?;
    }

    // Set up file watcher
    let (watcher_tx, mut watcher_rx) = mpsc::channel(100);
    let mut watcher = RecommendedWatcher::new(
        move |res: Result<Event, notify::Error>| {
            if let Ok(event) = res {
                let _ = watcher_tx.blocking_send(event);
            }
        },
        Config::default(),
    )?;

    watcher.watch(&config.path, RecursiveMode::Recursive)?;

    // Process file system events
    while let Some(event) = watcher_rx.recv().await {
        match event.kind {
            notify::EventKind::Create(_) => {
                for path in event.paths {
                    if let Ok(node) = scan_single_path(&path).await {
                        tx.send(SseEvent::Created {
                            path: path.display().to_string(),
                            node,
                        })
                        .await?;
                    }
                }
            }
            notify::EventKind::Modify(_) => {
                for path in event.paths {
                    if let Ok(node) = scan_single_path(&path).await {
                        tx.send(SseEvent::Modified {
                            path: path.display().to_string(),
                            node,
                        })
                        .await?;
                    }
                }
            }
            notify::EventKind::Remove(_) => {
                for path in event.paths {
                    tx.send(SseEvent::Deleted {
                        path: path.display().to_string(),
                    })
                    .await?;
                }
            }
            _ => {}
        }
    }

    Ok(())
}

/// Scan a single path and create FileNode
#[allow(dead_code)]
async fn scan_single_path(path: &Path) -> Result<FileNode> {
    let metadata = tokio::fs::metadata(path).await?;

    // Get Unix permissions
    #[cfg(unix)]
    let permissions = {
        use std::os::unix::fs::PermissionsExt;
        metadata.permissions().mode()
    };
    #[cfg(not(unix))]
    let permissions = 0o755;

    // Get Unix uid/gid
    #[cfg(unix)]
    let (uid, gid) = {
        use std::os::unix::fs::MetadataExt;
        (metadata.uid(), metadata.gid())
    };
    #[cfg(not(unix))]
    let (uid, gid) = (0, 0);

    let is_hidden = path
        .file_name()
        .and_then(|n| n.to_str())
        .map(|n| n.starts_with('.'))
        .unwrap_or(false);

    let file_type = if metadata.is_dir() {
        crate::scanner::FileType::Directory
    } else if metadata.file_type().is_symlink() {
        crate::scanner::FileType::Symlink
    } else if metadata.is_file() {
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            if metadata.permissions().mode() & 0o111 != 0 {
                crate::scanner::FileType::Executable
            } else {
                crate::scanner::FileType::RegularFile
            }
        }
        #[cfg(not(unix))]
        crate::scanner::FileType::RegularFile
    } else {
        crate::scanner::FileType::RegularFile
    };

    // Use Scanner's internal method for determining category
    // For now, we'll use Unknown since get_file_category is private
    let category = crate::scanner::FileCategory::Unknown;

    Ok(FileNode {
        path: path.to_path_buf(),
        is_dir: metadata.is_dir(),
        size: metadata.len(),
        permissions,
        uid,
        gid,
        modified: metadata.modified()?,
        is_symlink: metadata.file_type().is_symlink(),
        is_hidden,
        permission_denied: false,
        depth: 0,
        is_ignored: false,
        file_type,
        category,
        search_matches: None,
        filesystem_type: crate::scanner::FilesystemType::Unknown,
    })
}

/// Gather current statistics for a path
#[allow(dead_code)]
async fn gather_stats(path: &Path) -> Result<ScanStats> {
    let scanner_config = ScannerConfig::default();
    let scanner = Scanner::new(path, scanner_config)?;
    let start = std::time::Instant::now();
    let (_, stats) = scanner.scan()?;
    let scan_time_ms = start.elapsed().as_millis() as u64;

    Ok(ScanStats {
        total_files: stats.total_files,
        total_dirs: stats.total_dirs,
        total_size: stats.total_size,
        scan_time_ms,
    })
}

/// Format nodes using the specified output format
#[allow(dead_code)]
fn format_nodes(
    nodes: &[FileNode],
    stats: &crate::scanner::TreeStats,
    root_path: &Path,
    format: &OutputFormat,
) -> Result<String> {
    let mut output = Vec::new();

    match format {
        OutputFormat::Hex => {
            let formatter = HexFormatter::new(
                false,
                false,
                false,
                crate::formatters::PathDisplayMode::Off,
                false,
            );
            formatter.format(&mut output, nodes, stats, root_path)?;
        }
        OutputFormat::Ai => {
            let formatter = AiFormatter::new(false, crate::formatters::PathDisplayMode::Off);
            formatter.format(&mut output, nodes, stats, root_path)?;
        }
        OutputFormat::Quantum => {
            let formatter = QuantumFormatter::new();
            formatter.format(&mut output, nodes, stats, root_path)?;
        }
        OutputFormat::Json => {
            let json = serde_json::json!({
                "nodes": nodes.len(),
                "stats": {
                    "total_files": stats.total_files,
                    "total_dirs": stats.total_dirs,
                    "total_size": stats.total_size,
                },
                "root": root_path.display().to_string()
            });
            serde_json::to_writer_pretty(&mut output, &json)?;
        }
        _ => {
            // For other formats, use JSON as fallback
            let json = serde_json::json!({
                "nodes": nodes.len(),
                "stats": {
                    "total_files": stats.total_files,
                    "total_dirs": stats.total_dirs,
                    "total_size": stats.total_size,
                },
                "root": root_path.display().to_string()
            });
            serde_json::to_writer_pretty(&mut output, &json)?;
        }
    }

    Ok(String::from_utf8_lossy(&output).to_string())
}

/// Create SSE response format
#[allow(dead_code)]
pub fn format_sse_event(event: &SseEvent) -> Result<String> {
    let json = serde_json::to_string(event)?;
    Ok(format!("data: {}\n\n", json))
}
