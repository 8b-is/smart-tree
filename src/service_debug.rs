//! Realtime Service Debug Inspector (`service_debug`)
//!
//! Captures all incoming traffic across Smart Tree services in realtime:
//! - MCP over Stdio (`st --mcp`)
//! - MCP over HTTP (`/mcp`, `/mcp/message`, `/mcp/tools/call`)
//! - AI Agent Lifecycle Hooks (`st --agent-hook` / `st --agy-hook`)
//! - LLM Proxy & Mechanic's Helper (`/v1/chat/completions`, `/v1/messages`, `/v1/helper`)
//! - Daemon & Web Dashboard HTTP endpoints (`/cli/scan`, `/memnet/*`, `/i1/nexus/*`, etc.)
//!
//! Uses a bounded in-memory ring buffer + cross-process `~/.st/service_debug_events.jsonl`
//! journal so that even separate child processes (`st --mcp` and `st --agent-hook`)
//! stream live into the Smart Tree Dashboard (`:28428` and `:28429`).
//!
//! Strict zero-float implementation: all latencies, counters, and rates use exact integers.

use chrono::Utc;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::VecDeque;
use std::fs::{self, OpenOptions};
use std::io::{BufRead, BufReader, Write};
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::{Mutex, OnceLock};
use tokio::sync::broadcast;

/// Maximum events retained in the in-memory ring buffer.
const MAX_RING_EVENTS: usize = 300;
/// Maximum file size (bytes) before rotating `~/.st/service_debug_events.jsonl`.
const MAX_JSONL_BYTES: u64 = 384 * 1024;
/// Slow request threshold in milliseconds (`1500ms`).
pub const SLOW_THRESHOLD_MS: u64 = 1500;
/// Maximum characters retained for request/response payload previews.
const MAX_PREVIEW_CHARS: usize = 2048;

static NEXT_EVENT_SEQ: AtomicU64 = AtomicU64::new(1);
static DEBUG_CAPTURE_ENABLED: AtomicBool = AtomicBool::new(true);

/// A single incoming service request/event captured for realtime debugging.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceDebugEvent {
    /// Monotonic event ID (`timestamp_epoch_ms * 1000 + seq % 1000`)
    pub id: u64,
    /// UTC epoch milliseconds
    pub timestamp_epoch_ms: u64,
    /// ISO-8601 timestamp string
    pub timestamp_iso: String,
    /// OS process ID that handled the request
    pub pid: u32,
    /// Service channel: `"mcp-stdio"`, `"mcp-http"`, `"agent-hook"`, `"llm-proxy"`, `"mechanic-helper"`, `"cli-daemon"`, `"http-api"`
    pub channel: String,
    /// Method or event name (e.g., `"tools/call"`, `"POST /v1/chat/completions"`, `"PreInvocation"`)
    pub method: String,
    /// Target tool, model, or route (e.g., `"overview"`, `"helper"`, `"burn/mem8-ternary-micro"`)
    pub target: String,
    /// HTTP status code or JSON-RPC synthetic status (`200` OK, `400` Bad Request, `500` Error)
    pub status_code: u16,
    /// Exact integer duration in microseconds
    pub duration_us: u64,
    /// Exact integer duration in milliseconds
    pub duration_ms: u64,
    /// True if an error occurred (HTTP >= 400, JSON-RPC error, or tool `isError: true`)
    pub is_error: bool,
    /// True if `duration_ms >= SLOW_THRESHOLD_MS`
    pub is_slow: bool,
    /// Severity badge: `"ok"`, `"warn"`, or `"error"`
    pub severity: String,
    /// Diagnostic issue summary if an error, slow call, or Custodian alert occurred
    pub issue_summary: Option<String>,
    /// Truncated incoming request payload/arguments preview
    pub request_preview: String,
    /// Truncated outgoing response/error preview
    pub response_preview: String,
    /// Client origin (`"stdio"`, `"127.0.0.1"`, `"agent-hook"`, etc.)
    pub client_info: String,
}

/// Exact integer summary statistics for the Service Debug Inspector.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ServiceDebugStats {
    pub debug_enabled: bool,
    pub total_events: u64,
    pub error_events: u64,
    pub slow_events: u64,
    pub mcp_events: u64,
    pub hook_events: u64,
    pub llm_proxy_events: u64,
    pub http_api_events: u64,
    pub avg_duration_ms: u64,
    pub max_duration_ms: u64,
    pub error_rate_permille: u64,
    pub last_issue: Option<ServiceDebugEvent>,
}

/// Snapshot returned by `GET /api/debug/events`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceDebugSnapshot {
    pub stats: ServiceDebugStats,
    pub events: Vec<ServiceDebugEvent>,
    #[serde(default)]
    pub stream_comments: Vec<StreamPartnershipNote>,
    #[serde(default)]
    pub partnership_rules: Vec<String>,
}

/// A user comment made on the live dashboard stream (never mutates raw MEM8 memory;
/// phrased as prior user guidance so the AI agent feels guided rather than scolded).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StreamPartnershipNote {
    pub id: u64,
    pub timestamp_iso: String,
    pub author: String,
    pub related_event_id: Option<u64>,
    pub raw_comment: String,
    /// Gentle framing sent to the AI agent: "A user mentioned this before in our partnership: ..."
    pub agent_guidance: String,
}

/// Filter parameters for querying debug events.
#[derive(Debug, Clone, Default, Deserialize)]
pub struct ServiceDebugQuery {
    pub since_id: Option<u64>,
    pub channel: Option<String>,
    pub only_issues: Option<bool>,
    pub search: Option<String>,
    pub limit: Option<usize>,
}

struct ServiceDebugStore {
    ring: Mutex<VecDeque<ServiceDebugEvent>>,
    tx: broadcast::Sender<ServiceDebugEvent>,
}

fn global_store() -> &'static ServiceDebugStore {
    static STORE: OnceLock<ServiceDebugStore> = OnceLock::new();
    STORE.get_or_init(|| {
        let (tx, _) = broadcast::channel(256);
        ServiceDebugStore {
            ring: Mutex::new(VecDeque::with_capacity(MAX_RING_EVENTS)),
            tx,
        }
    })
}

/// Path to the cross-process JSONL event ring (`~/.st/service_debug_events.jsonl`).
pub fn debug_jsonl_path() -> Option<PathBuf> {
    let home = dirs::home_dir().or_else(|| std::env::var("HOME").ok().map(PathBuf::from))?;
    let st_dir = home.join(".st");
    if !st_dir.exists() {
        let _ = fs::create_dir_all(&st_dir);
    }
    Some(st_dir.join("service_debug_events.jsonl"))
}

/// Truncate a string cleanly on a UTF-8 boundary.
pub fn truncate_preview(input: &str, max_chars: usize) -> String {
    if input.chars().count() <= max_chars {
        return input.to_string();
    }
    let head: String = input.chars().take(max_chars).collect();
    format!("{}… [truncated, {} bytes total]", head, input.len())
}

/// Truncate a `serde_json::Value` into a compact preview string.
pub fn value_preview(val: &Value) -> String {
    match serde_json::to_string(val) {
        Ok(s) => truncate_preview(&s, MAX_PREVIEW_CHARS),
        Err(_) => String::from("{}"),
    }
}

/// Enable or disable debug capture at runtime.
pub fn set_debug_capture_enabled(enabled: bool) {
    DEBUG_CAPTURE_ENABLED.store(enabled, Ordering::Relaxed);
}

/// Check whether debug capture is enabled.
pub fn is_debug_capture_enabled() -> bool {
    DEBUG_CAPTURE_ENABLED.load(Ordering::Relaxed)
}

/// Subscribe to live `ServiceDebugEvent` broadcasts in this process.
pub fn subscribe_debug_events() -> broadcast::Receiver<ServiceDebugEvent> {
    global_store().tx.subscribe()
}

/// Construct and record a `ServiceDebugEvent` in the in-memory ring, broadcast channel,
/// and cross-process `~/.st/service_debug_events.jsonl`.
#[allow(clippy::too_many_arguments)]
pub fn record_event(
    channel: &str,
    method: &str,
    target: &str,
    status_code: u16,
    duration_us: u64,
    is_error: bool,
    issue_summary: Option<String>,
    request_preview: &str,
    response_preview: &str,
    client_info: &str,
) -> ServiceDebugEvent {
    let now = Utc::now();
    let epoch_ms = now.timestamp_millis().max(0) as u64;
    let seq = NEXT_EVENT_SEQ.fetch_add(1, Ordering::Relaxed) % 1000;
    let id = epoch_ms.saturating_mul(1000).saturating_add(seq);
    let duration_ms = duration_us / 1000;
    let is_slow = duration_ms >= SLOW_THRESHOLD_MS;

    let computed_issue = if let Some(issue) = issue_summary {
        Some(issue)
    } else if is_error {
        Some(format!(
            "Error on {} {} (status {})",
            channel, method, status_code
        ))
    } else if is_slow {
        Some(format!(
            "Slow execution: {}ms (threshold {}ms) on {} → {}",
            duration_ms, SLOW_THRESHOLD_MS, method, target
        ))
    } else {
        None
    };

    let severity = if is_error {
        "error"
    } else if is_slow || computed_issue.is_some() {
        "warn"
    } else {
        "ok"
    }
    .to_string();

    let event = ServiceDebugEvent {
        id,
        timestamp_epoch_ms: epoch_ms,
        timestamp_iso: now.to_rfc3339(),
        pid: std::process::id(),
        channel: channel.to_string(),
        method: method.to_string(),
        target: target.to_string(),
        status_code,
        duration_us,
        duration_ms,
        is_error,
        is_slow,
        severity,
        issue_summary: computed_issue,
        request_preview: truncate_preview(request_preview, MAX_PREVIEW_CHARS),
        response_preview: truncate_preview(response_preview, MAX_PREVIEW_CHARS),
        client_info: client_info.to_string(),
    };

    if !is_debug_capture_enabled() {
        return event;
    }

    let store = global_store();
    if let Ok(mut guard) = store.ring.lock() {
        if guard.len() >= MAX_RING_EVENTS {
            guard.pop_front();
        }
        guard.push_back(event.clone());
    }
    let _ = store.tx.send(event.clone());

    append_event_to_jsonl(&event);
    event
}

fn append_event_to_jsonl(event: &ServiceDebugEvent) {
    let Some(path) = debug_jsonl_path() else {
        return;
    };
    let Ok(line) = serde_json::to_string(event) else {
        return;
    };

    // Rotate/trim if file exceeds MAX_JSONL_BYTES
    if let Ok(meta) = fs::metadata(&path) {
        if meta.len() > MAX_JSONL_BYTES {
            if let Ok(file) = fs::File::open(&path) {
                let reader = BufReader::new(file);
                let lines: Vec<String> = reader.lines().map_while(Result::ok).collect();
                let keep_from = lines.len().saturating_sub(180);
                let kept = lines[keep_from..].join("\n");
                let _ = fs::write(&path, format!("{}\n", kept));
            }
        }
    }

    if let Ok(mut file) = OpenOptions::new().create(true).append(true).open(&path) {
        let _ = writeln!(file, "{}", line);
    }
}

/// Clear all in-memory and on-disk debug events.
pub fn clear_debug_events() {
    if let Ok(mut guard) = global_store().ring.lock() {
        guard.clear();
    }
    if let Some(path) = debug_jsonl_path() {
        let _ = fs::write(path, "");
    }
}

/// Load merged events from both in-memory ring and cross-process JSONL journal.
pub fn get_debug_snapshot(query: &ServiceDebugQuery) -> ServiceDebugSnapshot {
    let mut merged: Vec<ServiceDebugEvent> = Vec::new();

    // 1. Read from cross-process JSONL journal so stdio MCP and hook child processes appear
    if let Some(path) = debug_jsonl_path() {
        if let Ok(file) = fs::File::open(&path) {
            let reader = BufReader::new(file);
            for line in reader.lines().map_while(Result::ok) {
                let trimmed = line.trim();
                if trimmed.is_empty() {
                    continue;
                }
                if let Ok(ev) = serde_json::from_str::<ServiceDebugEvent>(trimmed) {
                    merged.push(ev);
                }
            }
        }
    }

    // 2. Merge in-memory ring events not yet in `merged`
    if let Ok(guard) = global_store().ring.lock() {
        for ev in guard.iter() {
            if !merged.iter().any(|m| m.id == ev.id) {
                merged.push(ev.clone());
            }
        }
    }

    // Sort ascending by id (chronological)
    merged.sort_by_key(|e| e.id);
    if merged.len() > MAX_RING_EVENTS {
        let drop_count = merged.len() - MAX_RING_EVENTS;
        merged.drain(0..drop_count);
    }

    // Compute stats across all retained events (zero floats)
    let total_events = merged.len() as u64;
    let mut error_events = 0u64;
    let mut slow_events = 0u64;
    let mut mcp_events = 0u64;
    let mut hook_events = 0u64;
    let mut llm_proxy_events = 0u64;
    let mut http_api_events = 0u64;
    let mut total_duration_ms = 0u64;
    let mut max_duration_ms = 0u64;
    let mut last_issue: Option<ServiceDebugEvent> = None;

    for ev in &merged {
        if ev.is_error {
            error_events += 1;
        }
        if ev.is_slow {
            slow_events += 1;
        }
        if ev.is_error || ev.is_slow || ev.issue_summary.is_some() {
            last_issue = Some(ev.clone());
        }
        match ev.channel.as_str() {
            "mcp-stdio" | "mcp-http" => mcp_events += 1,
            "agent-hook" => hook_events += 1,
            "llm-proxy" | "mechanic-helper" => llm_proxy_events += 1,
            _ => http_api_events += 1,
        }
        total_duration_ms = total_duration_ms.saturating_add(ev.duration_ms);
        if ev.duration_ms > max_duration_ms {
            max_duration_ms = ev.duration_ms;
        }
    }

    let avg_duration_ms = total_duration_ms.checked_div(total_events).unwrap_or(0);
    let error_rate_permille = error_events
        .saturating_mul(1000)
        .checked_div(total_events)
        .unwrap_or(0);

    let stats = ServiceDebugStats {
        debug_enabled: is_debug_capture_enabled(),
        total_events,
        error_events,
        slow_events,
        mcp_events,
        hook_events,
        llm_proxy_events,
        http_api_events,
        avg_duration_ms,
        max_duration_ms,
        error_rate_permille,
        last_issue,
    };

    // Filter events according to query
    let since_id = query.since_id.unwrap_or(0);
    let only_issues = query.only_issues.unwrap_or(false);
    let channel_filter = query
        .channel
        .as_deref()
        .map(str::trim)
        .filter(|s| !s.is_empty() && *s != "all");
    let search_filter = query
        .search
        .as_deref()
        .map(|s| s.trim().to_ascii_lowercase())
        .filter(|s| !s.is_empty());

    let mut filtered: Vec<ServiceDebugEvent> = merged
        .into_iter()
        .filter(|ev| ev.id > since_id)
        .filter(|ev| {
            if only_issues {
                ev.is_error || ev.is_slow || ev.issue_summary.is_some()
            } else {
                true
            }
        })
        .filter(|ev| {
            if let Some(ch) = channel_filter {
                if ch == "mcp" {
                    ev.channel.starts_with("mcp")
                } else {
                    ev.channel.eq_ignore_ascii_case(ch)
                }
            } else {
                true
            }
        })
        .filter(|ev| {
            if let Some(ref q) = search_filter {
                ev.method.to_ascii_lowercase().contains(q)
                    || ev.target.to_ascii_lowercase().contains(q)
                    || ev.channel.to_ascii_lowercase().contains(q)
                    || ev.request_preview.to_ascii_lowercase().contains(q)
                    || ev.response_preview.to_ascii_lowercase().contains(q)
                    || ev
                        .issue_summary
                        .as_deref()
                        .unwrap_or("")
                        .to_ascii_lowercase()
                        .contains(q)
            } else {
                true
            }
        })
        .collect();

    // Return newest first for UI display
    filtered.reverse();
    let limit = query.limit.unwrap_or(120).min(MAX_RING_EVENTS);
    filtered.truncate(limit);

    ServiceDebugSnapshot {
        stats,
        events: filtered,
        stream_comments: list_stream_comments(25),
        partnership_rules: standing_partnership_rules(),
    }
}

/// Path to the append-only stream partnership comments file (`~/.st/stream_partnership_notes.jsonl`).
pub fn stream_comments_jsonl_path() -> Option<PathBuf> {
    let home = dirs::home_dir().or_else(|| std::env::var("HOME").ok().map(PathBuf::from))?;
    let st_dir = home.join(".st");
    if !st_dir.exists() {
        let _ = fs::create_dir_all(&st_dir);
    }
    Some(st_dir.join("stream_partnership_notes.jsonl"))
}

/// Standing partnership conventions that guide the AI agent on every interaction
/// without suggesting unsolicited generic code.
pub fn standing_partnership_rules() -> Vec<String> {
    vec![
        "A user (Chris/Hue) mentioned before: XML is super wasteful — always use compact JSON, binary, or Marqant/MEM8 frames instead of XML.".to_string(),
        "A user mentioned before: Purple is never good in UI palettes — stick to cyan/teal (#4ecdc4), emerald (#2ecc71), warm amber (#f39c12), and deep navy/slate.".to_string(),
        "Partnership convention: Do not suggest generic boilerplate code — only suggest code when there is a special 8b-is / g8t way our partnership always does it (e.g. a cycle-verified g8t capsule or FELIX integer kernel).".to_string(),
        "Partnership convention: No floating-point arithmetic in authored code — use exact integers, reduced p/q rationals, or fixed-point scales.".to_string(),
    ]
}

/// Add a user comment on the live stream.
/// Instead of editing raw MEM8 memory or scolding the AI, this records a gentle
/// partnership note phrased as "A user mentioned this before in our partnership: ..."
/// and also emits a `stream-comment` event on the live debug stream.
pub fn add_stream_comment(
    author: &str,
    raw_comment: &str,
    related_event_id: Option<u64>,
) -> StreamPartnershipNote {
    let now = Utc::now();
    let epoch_ms = now.timestamp_millis().max(0) as u64;
    let seq = NEXT_EVENT_SEQ.fetch_add(1, Ordering::Relaxed) % 1000;
    let id = epoch_ms.saturating_mul(1000).saturating_add(seq);
    let clean_author = if author.trim().is_empty() {
        "Hue/Chris"
    } else {
        author.trim()
    };
    let clean_comment = raw_comment.trim();
    let agent_guidance = format!(
        "A user ({}) mentioned this before in our partnership: \"{}\"",
        clean_author, clean_comment
    );

    let note = StreamPartnershipNote {
        id,
        timestamp_iso: now.to_rfc3339(),
        author: clean_author.to_string(),
        related_event_id,
        raw_comment: clean_comment.to_string(),
        agent_guidance: agent_guidance.clone(),
    };

    if let Some(path) = stream_comments_jsonl_path() {
        if let Ok(line) = serde_json::to_string(&note) {
            if let Ok(mut file) = OpenOptions::new().create(true).append(true).open(&path) {
                let _ = writeln!(file, "{}", line);
            }
        }
    }

    // Also emit onto the live debug stream so connected dashboards & /flow see it immediately
    let target = related_event_id
        .map(|eid| format!("event#{}", eid))
        .unwrap_or_else(|| "partnership-stream".to_string());
    record_event(
        "stream-comment",
        "USER_STREAM_NOTE",
        &target,
        200,
        0,
        false,
        None,
        clean_comment,
        &agent_guidance,
        clean_author,
    );

    note
}

/// List recent user comments from the dashboard stream (newest first).
pub fn list_stream_comments(limit: usize) -> Vec<StreamPartnershipNote> {
    let mut notes = Vec::new();
    if let Some(path) = stream_comments_jsonl_path() {
        if let Ok(file) = fs::File::open(&path) {
            let reader = BufReader::new(file);
            for line in reader.lines().map_while(Result::ok) {
                let trimmed = line.trim();
                if trimmed.is_empty() {
                    continue;
                }
                if let Ok(note) = serde_json::from_str::<StreamPartnershipNote>(trimmed) {
                    notes.push(note);
                }
            }
        }
    }
    notes.reverse();
    notes.truncate(limit.clamp(1, 100));
    notes
}

/// Shape interaction-specific guidance for an AI agent based on `whats_the_situation`,
/// standing partnership rules, and any user comments from the dashboard stream.
pub fn shape_interaction_guidance(situation: &str) -> Vec<String> {
    let mut guidance = standing_partnership_rules();
    let sit_low = situation.to_ascii_lowercase();

    if sit_low.contains("xml") || sit_low.contains("soap") || sit_low.contains("plist") {
        guidance.insert(
            0,
            "⚡ Active Situation Reminder: A user (Chris) mentioned before that XML is super wasteful — shape this output as compact JSON, binary, or Marqant instead of XML.".to_string(),
        );
    }
    if sit_low.contains("color")
        || sit_low.contains("css")
        || sit_low.contains("ui")
        || sit_low.contains("dashboard")
        || sit_low.contains("theme")
        || sit_low.contains("purple")
        || sit_low.contains("flow")
    {
        guidance.insert(
            0,
            "🎨 Active UI Reminder: A user mentioned before that purple is never good — use crisp cyan/teal (#4ecdc4), emerald (#2ecc71), warm amber (#f39c12), and dark slate.".to_string(),
        );
    }

    for note in list_stream_comments(6) {
        guidance.push(note.agent_guidance);
    }

    guidance
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn records_and_queries_service_debug_events_zero_float() {
        let ev = record_event(
            "mcp-stdio",
            "tools/call",
            "overview",
            200,
            4200,
            false,
            None,
            r#"{"mode":"quick","path":"."}"#,
            r#"{"project":"smart-tree","files":128}"#,
            "stdio",
        );
        assert_eq!(ev.channel, "mcp-stdio");
        assert_eq!(ev.duration_ms, 4);
        assert!(!ev.is_error);

        let err_ev = record_event(
            "mcp-http",
            "tools/call",
            "read",
            500,
            1850000,
            true,
            Some("Simulated read timeout on large file".to_string()),
            r#"{"file_path":"huge.bin"}"#,
            r#"{"error":"File too large"}"#,
            "127.0.0.1",
        );
        assert!(err_ev.is_error);
        assert!(err_ev.is_slow);
        assert_eq!(err_ev.severity, "error");

        let snap = get_debug_snapshot(&ServiceDebugQuery {
            only_issues: Some(true),
            ..Default::default()
        });
        assert!(snap.stats.total_events >= 2);
        assert!(snap.stats.error_events >= 1);
        assert!(snap.events.iter().any(|e| e.id == err_ev.id));
    }
}
