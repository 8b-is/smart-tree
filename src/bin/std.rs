//! STD - Smart Tree Daemon
//!
//! Persistent daemon providing context, security, and API services.
//! Listens on Unix socket using the ST binary protocol.
//!
//! ## Usage
//!
//! ```bash
//! std start              # Start daemon
//! std stop               # Stop daemon
//! std status             # Health check
//! ```

use anyhow::{Context, Result};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{UnixListener, UnixStream};
use tokio::sync::RwLock;
use tracing::{debug, error, info};

use st::formatters::{
    ai::AiFormatter, classic::ClassicFormatter, digest::DigestFormatter, hex::HexFormatter,
    json::JsonFormatter, quantum::QuantumFormatter, stats::StatsFormatter, Formatter,
    PathDisplayMode,
};
use st::scanner::{Scanner, ScannerConfig};
use st_protocol::{Address, AuthLevel, Frame, Payload, PayloadDecoder, SecurityContext, Verb};

/// Daemon configuration
#[derive(Debug, Clone)]
pub struct DaemonConfig {
    /// Socket path
    pub socket_path: PathBuf,
    /// PID file path
    pub pid_path: PathBuf,
    /// Log level
    pub log_level: String,
}

impl Default for DaemonConfig {
    fn default() -> Self {
        let runtime_dir = std::env::var("XDG_RUNTIME_DIR")
            .map(PathBuf::from)
            .unwrap_or_else(|_| PathBuf::from("/tmp"));

        DaemonConfig {
            socket_path: runtime_dir.join("st.sock"),
            pid_path: runtime_dir.join("st.pid"),
            log_level: "info".to_string(),
        }
    }
}

/// Session state for a connected client
#[derive(Debug)]
#[allow(dead_code)]
struct ClientSession {
    security: SecurityContext,
    address: Address,
}

impl Default for ClientSession {
    fn default() -> Self {
        ClientSession {
            security: SecurityContext::new(),
            address: Address::Local,
        }
    }
}

/// Daemon state shared across connections
#[allow(dead_code)]
struct DaemonState {
    config: DaemonConfig,
    // Future: MCP context, caches, etc.
}

impl DaemonState {
    fn new(config: DaemonConfig) -> Self {
        DaemonState { config }
    }
}

/// Handle a single client connection
async fn handle_client(
    mut stream: UnixStream,
    state: Arc<RwLock<DaemonState>>,
) -> Result<()> {
    let mut session = ClientSession::default();
    let mut buf = vec![0u8; 65536]; // Max frame size

    loop {
        // Read frame header (at least verb + END = 2 bytes)
        let n = stream.read(&mut buf).await?;
        if n == 0 {
            debug!("Client disconnected");
            return Ok(());
        }

        // Find frame end
        let frame_end = match Frame::find_end(&buf[..n]) {
            Some(end) => end,
            None => {
                // Incomplete frame - need more data (simplified: error for now)
                let response = Frame::error("Incomplete frame");
                stream.write_all(&response.encode()).await?;
                continue;
            }
        };

        // Decode frame
        let frame = match Frame::decode(&buf[..frame_end]) {
            Ok(f) => f,
            Err(e) => {
                let response = Frame::error(&format!("Decode error: {e:?}"));
                stream.write_all(&response.encode()).await?;
                continue;
            }
        };

        debug!("Received {:?}", frame.verb());

        // Check security level
        let required_level = frame.verb().security_level();
        if (session.security.level() as u8) < required_level {
            let response = Frame::error(&format!(
                "Requires auth level {}",
                AuthLevel::from_byte(required_level)
                    .map(|l| l.name())
                    .unwrap_or("unknown")
            ));
            stream.write_all(&response.encode()).await?;
            continue;
        }

        // Handle verb
        let response = handle_verb(frame, &mut session, &state).await;
        stream.write_all(&response.encode()).await?;

        // Check for EndStream
        if response.verb() == Verb::EndStream {
            break;
        }
    }

    Ok(())
}

/// Handle a single verb and return response frame
async fn handle_verb(
    frame: Frame,
    session: &mut ClientSession,
    state: &Arc<RwLock<DaemonState>>,
) -> Frame {
    match frame.verb() {
        Verb::Ping => Frame::ok(),

        Verb::Scan => handle_scan(frame.into_payload(), state).await,

        Verb::Format => handle_format(frame.into_payload(), state).await,

        Verb::Search => handle_search(frame.into_payload(), state).await,

        Verb::Stats => handle_stats(state).await,

        Verb::Context => handle_context(frame.into_payload(), state).await,

        Verb::Session => {
            // Return session info
            let mut payload = Payload::new();
            payload.push_byte(session.security.level() as u8);
            Frame::new(Verb::Ok, payload)
        }

        Verb::EndStream => Frame::simple(Verb::EndStream),

        Verb::Subscribe => {
            // TODO: implement file watching
            Frame::error("Subscribe not yet implemented")
        }

        Verb::Unsubscribe => {
            // TODO: implement file watching
            Frame::error("Unsubscribe not yet implemented")
        }

        // Auth verbs
        Verb::AuthStart | Verb::AuthEnd => {
            Frame::error("Auth block expected, not standalone verb")
        }

        Verb::Elevate => {
            // TODO: FIDO2 integration
            Frame::error("Elevate not yet implemented")
        }

        // Memory verbs
        Verb::M8Wave | Verb::Remember | Verb::Recall | Verb::Forget => {
            // TODO: M8 memory integration
            Frame::error("Memory operations not yet implemented")
        }

        // Admin verbs
        Verb::Permit | Verb::Deny | Verb::Audit => {
            Frame::error("Admin operations not yet implemented")
        }

        // Misc
        Verb::Ok | Verb::Error | Verb::Alert => {
            // These are response verbs, not request verbs
            Frame::error("Invalid request verb")
        }

        Verb::Back | Verb::Next | Verb::Clear | Verb::Complete | Verb::User | Verb::Cancel => {
            Frame::error("Not implemented")
        }
    }
}

/// Handle SCAN verb
async fn handle_scan(payload: Payload, _state: &Arc<RwLock<DaemonState>>) -> Frame {
    let mut decoder = PayloadDecoder::new(&payload);

    // Parse path (length-prefixed string)
    let path = match decoder.string() {
        Some(p) => p.to_string(),
        None => ".".to_string(),
    };

    // Parse depth
    let depth = decoder.byte().unwrap_or(3);

    debug!("SCAN path={} depth={}", path, depth);

    // Use st scanner with config
    let config = ScannerConfig {
        max_depth: depth as usize,
        ..ScannerConfig::default()
    };

    let path = Path::new(&path);
    match Scanner::new(path, config).and_then(|s| s.scan()) {
        Ok((nodes, stats)) => {
            // Encode result as JSON for now (will optimize later)
            let result = serde_json::json!({
                "files": stats.total_files,
                "dirs": stats.total_dirs,
                "total_size": stats.total_size,
                "nodes": nodes,
            });
            Frame::new(Verb::Ok, Payload::from_string(&result.to_string()))
        }
        Err(e) => Frame::error(&format!("Scan failed: {e}")),
    }
}

/// Handle FORMAT verb - scan and format in one operation
/// Payload: [mode string][path string][depth byte]
async fn handle_format(payload: Payload, _state: &Arc<RwLock<DaemonState>>) -> Frame {
    let mut decoder = PayloadDecoder::new(&payload);

    // Parse mode (length-prefixed string)
    let mode = decoder.string().unwrap_or("classic");

    // Parse path (length-prefixed string)
    let path_str = decoder.string().unwrap_or(".");

    // Parse depth
    let depth = decoder.byte().unwrap_or(3);

    debug!("FORMAT mode={} path={} depth={}", mode, path_str, depth);

    // Scan the directory
    let config = ScannerConfig {
        max_depth: depth as usize,
        ..ScannerConfig::default()
    };

    let path = Path::new(path_str);
    let (nodes, stats) = match Scanner::new(path, config).and_then(|s| s.scan()) {
        Ok(result) => result,
        Err(e) => return Frame::error(&format!("Scan failed: {e}")),
    };

    // Get the appropriate formatter
    let formatter: Box<dyn Formatter> = match mode {
        "classic" => Box::new(ClassicFormatter::new(false, false, PathDisplayMode::Relative)),
        "ai" => Box::new(AiFormatter::new(false, PathDisplayMode::Relative)),
        "json" => Box::new(JsonFormatter::new(false)),
        "hex" => Box::new(HexFormatter::new(false, false, false, PathDisplayMode::Relative, false)),
        "quantum" => Box::new(QuantumFormatter::new()),
        "stats" => Box::new(StatsFormatter::new()),
        "digest" => Box::new(DigestFormatter::new()),
        _ => return Frame::error(&format!("Unknown format mode: {mode}")),
    };

    // Format to a buffer
    let mut output = Vec::new();
    if let Err(e) = formatter.format(&mut output, &nodes, &stats, path) {
        return Frame::error(&format!("Format failed: {e}"));
    }

    // Return formatted output
    let output_str = String::from_utf8_lossy(&output);
    Frame::new(Verb::Ok, Payload::from_string(&output_str))
}

/// Handle SEARCH verb
/// Payload: [path string][pattern string][max_results byte]
async fn handle_search(payload: Payload, _state: &Arc<RwLock<DaemonState>>) -> Frame {
    let mut decoder = PayloadDecoder::new(&payload);

    // Parse path (length-prefixed string)
    let path_str = decoder.string().unwrap_or(".");

    // Parse pattern (length-prefixed string)
    let pattern = decoder.string().unwrap_or("");

    // Parse max results
    let max_results = decoder.byte().unwrap_or(50) as usize;

    debug!("SEARCH path={} pattern={} max={}", path_str, pattern, max_results);

    if pattern.is_empty() {
        return Frame::error("Search pattern required");
    }

    let path = Path::new(path_str);

    // Use scanner with search_keyword for content search
    let config = ScannerConfig {
        max_depth: 10,
        search_keyword: Some(pattern.to_string()),
        include_line_content: true,
        ..ScannerConfig::default()
    };

    let (nodes, _stats) = match Scanner::new(path, config).and_then(|s| s.scan()) {
        Ok(result) => result,
        Err(e) => return Frame::error(&format!("Search failed: {e}")),
    };

    // Collect files with matches
    let mut results: Vec<_> = nodes
        .iter()
        .filter_map(|node| {
            let matches = node.search_matches.as_ref()?;
            if matches.total_count == 0 {
                return None;
            }

            let mut match_info = serde_json::json!({
                "path": node.path.display().to_string(),
                "matches": matches.total_count,
                "truncated": matches.truncated
            });

            // Include line content if available
            if let Some(ref lines) = matches.line_content {
                let line_results: Vec<_> = lines
                    .iter()
                    .take(10) // Limit lines per file
                    .map(|(line_num, content, col)| serde_json::json!({
                        "line": line_num,
                        "content": content,
                        "col": col
                    }))
                    .collect();
                match_info["lines"] = serde_json::json!(line_results);
            }

            Some((matches.total_count, match_info))
        })
        .collect();

    // Sort by match count descending, limit results
    results.sort_by(|a, b| b.0.cmp(&a.0));
    results.truncate(max_results);

    let results: Vec<_> = results.into_iter().map(|(_, info)| info).collect();

    let response = serde_json::json!({
        "pattern": pattern,
        "count": results.len(),
        "results": results
    });

    Frame::new(Verb::Ok, Payload::from_string(&response.to_string()))
}

/// Handle STATS verb
async fn handle_stats(_state: &Arc<RwLock<DaemonState>>) -> Frame {
    let stats = serde_json::json!({
        "version": env!("CARGO_PKG_VERSION"),
        "protocol": st_protocol::VERSION,
        "uptime_secs": 0, // TODO: track uptime
    });
    Frame::new(Verb::Ok, Payload::from_string(&stats.to_string()))
}

/// Handle CONTEXT verb
async fn handle_context(payload: Payload, _state: &Arc<RwLock<DaemonState>>) -> Frame {
    let path = payload.as_str().unwrap_or(".");
    debug!("CONTEXT path={}", path);

    // TODO: integrate with MCP context gathering
    Frame::error("Context gathering not yet implemented")
}

/// Start the daemon
async fn start_daemon(config: DaemonConfig) -> Result<()> {
    // Remove stale socket
    if config.socket_path.exists() {
        std::fs::remove_file(&config.socket_path)
            .context("Failed to remove stale socket")?;
    }

    // Create listener
    let listener = UnixListener::bind(&config.socket_path)
        .context("Failed to bind socket")?;

    info!("STD listening on {:?}", config.socket_path);

    // Write PID file
    let pid = std::process::id();
    std::fs::write(&config.pid_path, pid.to_string())
        .context("Failed to write PID file")?;

    // Shared state
    let state = Arc::new(RwLock::new(DaemonState::new(config.clone())));

    // Accept connections
    loop {
        match listener.accept().await {
            Ok((stream, _addr)) => {
                let state = state.clone();
                tokio::spawn(async move {
                    if let Err(e) = handle_client(stream, state).await {
                        error!("Client error: {e}");
                    }
                });
            }
            Err(e) => {
                error!("Accept error: {e}");
            }
        }
    }
}

/// Check daemon status
fn check_status(config: &DaemonConfig) -> Result<bool> {
    if !config.pid_path.exists() {
        println!("STD is not running");
        return Ok(false);
    }

    let pid_str = std::fs::read_to_string(&config.pid_path)?;
    let pid: u32 = pid_str.trim().parse()?;

    // Check if process exists
    let proc_path = format!("/proc/{}", pid);
    if std::path::Path::new(&proc_path).exists() {
        println!("STD is running (PID {})", pid);
        Ok(true)
    } else {
        println!("STD is not running (stale PID file)");
        // Clean up stale files
        let _ = std::fs::remove_file(&config.pid_path);
        let _ = std::fs::remove_file(&config.socket_path);
        Ok(false)
    }
}

/// Stop the daemon
fn stop_daemon(config: &DaemonConfig) -> Result<()> {
    if !config.pid_path.exists() {
        println!("STD is not running");
        return Ok(());
    }

    let pid_str = std::fs::read_to_string(&config.pid_path)?;
    let pid: i32 = pid_str.trim().parse()?;

    // Send SIGTERM
    unsafe {
        libc::kill(pid, libc::SIGTERM);
    }

    // Wait a moment
    std::thread::sleep(std::time::Duration::from_millis(100));

    // Clean up
    let _ = std::fs::remove_file(&config.pid_path);
    let _ = std::fs::remove_file(&config.socket_path);

    println!("STD stopped");
    Ok(())
}

#[tokio::main]
async fn main() -> Result<()> {
    // Parse command
    let args: Vec<String> = std::env::args().collect();
    let command = args.get(1).map(|s| s.as_str()).unwrap_or("start");

    let config = DaemonConfig::default();

    // Initialize logging
    tracing_subscriber::fmt()
        .with_env_filter(&config.log_level)
        .init();

    match command {
        "start" => {
            // Check if already running
            if check_status(&config)? {
                println!("STD is already running");
                return Ok(());
            }
            start_daemon(config).await
        }
        "stop" => stop_daemon(&config),
        "status" => {
            check_status(&config)?;
            Ok(())
        }
        "restart" => {
            stop_daemon(&config)?;
            std::thread::sleep(std::time::Duration::from_millis(200));
            start_daemon(config).await
        }
        "--help" | "-h" => {
            println!("STD - Smart Tree Daemon");
            println!();
            println!("Usage: std <command>");
            println!();
            println!("Commands:");
            println!("  start    Start the daemon");
            println!("  stop     Stop the daemon");
            println!("  status   Check daemon status");
            println!("  restart  Restart the daemon");
            Ok(())
        }
        _ => {
            eprintln!("Unknown command: {}", command);
            eprintln!("Run 'std --help' for usage");
            std::process::exit(1);
        }
    }
}
