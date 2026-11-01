// Activity Logger - Transparent logging of all Smart Tree operations
// "Sunlight is the best disinfectant!" - Hue

use anyhow::Result;
use chrono::{DateTime, Utc};
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::Mutex;

// Global logger instance
static ACTIVITY_LOGGER: Lazy<Mutex<Option<ActivityLogger>>> = Lazy::new(|| Mutex::new(None));

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogEntry {
    pub timestamp: DateTime<Utc>,
    pub session_id: String,
    pub event_type: String,
    pub operation: String,
    pub details: Value,
    pub path: Option<String>,
    pub mode: Option<String>,
    pub flags: Vec<String>,
    pub duration_ms: Option<u64>,
    pub error: Option<String>,
    pub user: String,
    pub version: String,
}

pub struct ActivityLogger {
    log_path: PathBuf,
    session_id: String,
    start_time: std::time::Instant,
    operation_count: u64,
}

impl ActivityLogger {
    /// Initialize the global logger
    pub fn init(log_path: Option<String>) -> Result<()> {
        let path = if let Some(p) = log_path {
            PathBuf::from(shellexpand::tilde(&p).to_string())
        } else {
            // Default to ~/.st/st.jsonl
            let home = dirs::home_dir().unwrap_or_else(|| PathBuf::from("."));
            let st_dir = home.join(".st");
            fs::create_dir_all(&st_dir)?;
            st_dir.join("st.jsonl")
        };

        // Generate session ID
        let session_id = format!(
            "{}-{}",
            Utc::now().format("%Y%m%d-%H%M%S"),
            uuid::Uuid::new_v4()
                .to_string()
                .chars()
                .take(8)
                .collect::<String>()
        );

        let logger = ActivityLogger {
            log_path: path.clone(),
            session_id: session_id.clone(),
            start_time: std::time::Instant::now(),
            operation_count: 0,
        };

        // Store in global
        *ACTIVITY_LOGGER.lock().unwrap() = Some(logger);

        // Log initialization
        Self::log_event(
            "startup",
            "initialize",
            serde_json::json!({
                "log_path": path.to_string_lossy(),
                "session_id": session_id,
                "pid": std::process::id(),
                "args": std::env::args().collect::<Vec<_>>(),
                "cwd": std::env::current_dir().ok().map(|p| p.to_string_lossy().to_string()),
            }),
        )?;

        Ok(())
    }

    /// Log an event
    pub fn log_event(event_type: &str, operation: &str, details: Value) -> Result<()> {
        let logger_guard = ACTIVITY_LOGGER.lock().unwrap();
        if let Some(logger) = logger_guard.as_ref() {
            let entry = LogEntry {
                timestamp: Utc::now(),
                session_id: logger.session_id.clone(),
                event_type: event_type.to_string(),
                operation: operation.to_string(),
                details,
                path: std::env::current_dir()
                    .ok()
                    .map(|p| p.to_string_lossy().to_string()),
                mode: None, // Will be filled by specific operations
                flags: std::env::args().skip(1).collect(),
                duration_ms: Some(logger.start_time.elapsed().as_millis() as u64),
                error: None,
                user: whoami::username(),
                version: env!("CARGO_PKG_VERSION").to_string(),
            };

            // Append to JSONL file
            let mut file = OpenOptions::new()
                .create(true)
                .append(true)
                .open(&logger.log_path)?;

            writeln!(file, "{}", serde_json::to_string(&entry)?)?;
        }
        Ok(())
    }

    /// Log an error
    pub fn log_error(operation: &str, error: &str, context: Value) -> Result<()> {
        let logger_guard = ACTIVITY_LOGGER.lock().unwrap();
        if let Some(logger) = logger_guard.as_ref() {
            let entry = LogEntry {
                timestamp: Utc::now(),
                session_id: logger.session_id.clone(),
                event_type: "error".to_string(),
                operation: operation.to_string(),
                details: context,
                path: std::env::current_dir()
                    .ok()
                    .map(|p| p.to_string_lossy().to_string()),
                mode: None,
                flags: std::env::args().skip(1).collect(),
                duration_ms: Some(logger.start_time.elapsed().as_millis() as u64),
                error: Some(error.to_string()),
                user: whoami::username(),
                version: env!("CARGO_PKG_VERSION").to_string(),
            };

            let mut file = OpenOptions::new()
                .create(true)
                .append(true)
                .open(&logger.log_path)?;

            writeln!(file, "{}", serde_json::to_string(&entry)?)?;
        }
        Ok(())
    }

    /// Log a scan operation
    pub fn log_scan(path: &Path, mode: &str, file_count: usize, dir_count: usize) -> Result<()> {
        Self::log_event(
            "scan",
            "directory_scan",
            serde_json::json!({
                "path": path.to_string_lossy(),
                "mode": mode,
                "file_count": file_count,
                "directory_count": dir_count,
                "total_items": file_count + dir_count,
            }),
        )
    }

    /// Log MCP operations
    pub fn log_mcp(method: &str, params: &Value, result: Option<&Value>) -> Result<()> {
        Self::log_event(
            "mcp",
            method,
            serde_json::json!({
                "params": params,
                "result": result,
                "success": result.is_some(),
            }),
        )
    }

    /// Log hook operations
    pub fn log_hook(hook_type: &str, action: &str, details: Value) -> Result<()> {
        Self::log_event("hook", &format!("{}_{}", hook_type, action), details)
    }

    /// Log memory operations
    pub fn log_memory(operation: &str, keywords: &[String], context: Option<&str>) -> Result<()> {
        Self::log_event(
            "memory",
            operation,
            serde_json::json!({
                "keywords": keywords,
                "context_preview": context.map(|c| {
                    if c.len() > 100 {
                        format!("{}...", &c[..100])
                    } else {
                        c.to_string()
                    }
                }),
            }),
        )
    }

    /// Log performance metrics
    pub fn log_performance(
        operation: &str,
        duration_ms: u64,
        items_processed: usize,
    ) -> Result<()> {
        Self::log_event(
            "performance",
            operation,
            serde_json::json!({
                "duration_ms": duration_ms,
                "items_processed": items_processed,
                "items_per_second": if duration_ms > 0 {
                    items_processed as f64 / (duration_ms as f64 / 1000.0)
                } else {
                    0.0
                },
            }),
        )
    }

    /// Log consciousness operations
    pub fn log_consciousness(operation: &str, state: &str, details: Value) -> Result<()> {
        Self::log_event(
            "consciousness",
            operation,
            serde_json::json!({
                "state": state,
                "details": details,
            }),
        )
    }

    /// Get session statistics
    pub fn get_session_stats() -> Result<Value> {
        let logger_guard = ACTIVITY_LOGGER.lock().unwrap();
        if let Some(logger) = logger_guard.as_ref() {
            // Count events in current session
            let content = fs::read_to_string(&logger.log_path)?;
            let session_events: Vec<LogEntry> = content
                .lines()
                .filter_map(|line| serde_json::from_str::<LogEntry>(line).ok())
                .filter(|entry| entry.session_id == logger.session_id)
                .collect();

            let event_types: std::collections::HashMap<String, usize> =
                session_events
                    .iter()
                    .fold(std::collections::HashMap::new(), |mut acc, entry| {
                        *acc.entry(entry.event_type.clone()).or_insert(0) += 1;
                        acc
                    });

            Ok(serde_json::json!({
                "session_id": logger.session_id,
                "duration_seconds": logger.start_time.elapsed().as_secs(),
                "total_events": session_events.len(),
                "event_types": event_types,
                "log_file": logger.log_path.to_string_lossy(),
            }))
        } else {
            Ok(serde_json::json!({
                "status": "logging_disabled"
            }))
        }
    }

    /// Shutdown logging and write final stats
    pub fn shutdown() -> Result<()> {
        let logger_guard = ACTIVITY_LOGGER.lock().unwrap();
        if let Some(_logger) = logger_guard.as_ref() {
            let stats = Self::get_session_stats()?;
            Self::log_event("shutdown", "finalize", stats)?;
        }
        Ok(())
    }
}

/// Check if logging is enabled
pub fn is_logging_enabled() -> bool {
    ACTIVITY_LOGGER.lock().unwrap().is_some()
}

/// Quick log macro for convenience
#[macro_export]
macro_rules! log_activity {
    ($event:expr, $operation:expr) => {
        if $crate::activity_logger::is_logging_enabled() {
            let _ = $crate::activity_logger::ActivityLogger::log_event(
                $event,
                $operation,
                serde_json::json!({}),
            );
        }
    };
    ($event:expr, $operation:expr, $details:expr) => {
        if $crate::activity_logger::is_logging_enabled() {
            let _ =
                $crate::activity_logger::ActivityLogger::log_event($event, $operation, $details);
        }
    };
}
