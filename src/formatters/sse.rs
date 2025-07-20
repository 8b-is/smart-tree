//! Server-Sent Events (SSE) formatter
//!
//! Streams directory changes and updates as SSE events

use crate::scanner::{FileNode, TreeStats};
use anyhow::Result;
use serde_json;
use std::io::Write;
use std::path::Path;

use super::{Formatter, StreamingFormatter};

pub struct SseFormatter {
    event_id: u64,
}

impl SseFormatter {
    pub fn new() -> Self {
        Self { event_id: 0 }
    }

    fn next_event_id(&mut self) -> u64 {
        self.event_id += 1;
        self.event_id
    }

    fn write_event(&self, writer: &mut dyn Write, event_type: &str, data: &serde_json::Value, id: u64) -> Result<()> {
        writeln!(writer, "id: {}", id)?;
        writeln!(writer, "event: {}", event_type)?;
        writeln!(writer, "data: {}", serde_json::to_string(data)?)?;
        writeln!(writer)?; // Empty line to end the event
        writer.flush()?;
        Ok(())
    }
}

impl Formatter for SseFormatter {
    fn format(
        &self,
        writer: &mut dyn Write,
        nodes: &[FileNode],
        stats: &TreeStats,
        root_path: &Path,
    ) -> Result<()> {
        let mut formatter = SseFormatter::new();
        
        // Send initial scan event
        let scan_event = serde_json::json!({
            "type": "scan_complete",
            "path": root_path.display().to_string(),
            "stats": {
                "total_files": stats.total_files,
                "total_dirs": stats.total_dirs,
                "total_size": stats.total_size,
            }
        });
        let id = formatter.next_event_id();
        formatter.write_event(writer, "scan", &scan_event, id)?;

        // Send node events
        for node in nodes {
            let node_event = serde_json::json!({
                "type": "node",
                "node": {
                    "name": node.path.file_name().unwrap_or(node.path.as_os_str()).to_string_lossy(),
                    "path": node.path.display().to_string(),
                    "is_dir": node.is_dir,
                    "size": node.size,
                    "depth": node.depth,
                }
            });
            let id = formatter.next_event_id();
            formatter.write_event(writer, "node", &node_event, id)?;
        }

        // Send completion event
        let complete_event = serde_json::json!({
            "type": "format_complete",
            "node_count": nodes.len(),
        });
        let id = formatter.next_event_id();
        formatter.write_event(writer, "complete", &complete_event, id)?;

        Ok(())
    }
}

impl StreamingFormatter for SseFormatter {
    fn start_stream(&self, writer: &mut dyn Write, root_path: &Path) -> Result<()> {
        // Send HTTP headers for SSE
        writeln!(writer, "HTTP/1.1 200 OK")?;
        writeln!(writer, "Content-Type: text/event-stream")?;
        writeln!(writer, "Cache-Control: no-cache")?;
        writeln!(writer, "Connection: keep-alive")?;
        writeln!(writer, "Access-Control-Allow-Origin: *")?;
        writeln!(writer)?; // Empty line to end headers
        
        // Send initial connection event
        let mut formatter = SseFormatter::new();
        let init_event = serde_json::json!({
            "type": "stream_start",
            "path": root_path.display().to_string(),
            "timestamp": chrono::Utc::now().to_rfc3339(),
        });
        let id = formatter.next_event_id();
        formatter.write_event(writer, "init", &init_event, id)?;
        
        Ok(())
    }

    fn format_node(
        &self,
        writer: &mut dyn Write,
        node: &FileNode,
        _root_path: &Path,
    ) -> Result<()> {
        let mut formatter = SseFormatter::new();
        
        let node_event = serde_json::json!({
            "type": "node_discovered",
            "node": {
                "name": node.path.file_name().unwrap_or(node.path.as_os_str()).to_string_lossy(),
                "path": node.path.display().to_string(),
                "is_dir": node.is_dir,
                "size": node.size,
                "depth": node.depth,
                "permissions": format!("{:o}", node.permissions),
                "modified": chrono::DateTime::<chrono::Utc>::from(node.modified).to_rfc3339(),
            }
        });
        
        let id = formatter.next_event_id();
        formatter.write_event(writer, "node", &node_event, id)?;
        Ok(())
    }

    fn end_stream(
        &self,
        writer: &mut dyn Write,
        stats: &TreeStats,
        root_path: &Path,
    ) -> Result<()> {
        let mut formatter = SseFormatter::new();
        
        // Send final statistics
        let stats_event = serde_json::json!({
            "type": "stream_complete",
            "path": root_path.display().to_string(),
            "stats": {
                "total_files": stats.total_files,
                "total_dirs": stats.total_dirs,
                "total_size": stats.total_size,
            },
            "timestamp": chrono::Utc::now().to_rfc3339(),
        });
        
        let id = formatter.next_event_id();
        formatter.write_event(writer, "complete", &stats_event, id)?;
        
        // Send close event
        let close_event = serde_json::json!({
            "type": "stream_close",
            "reason": "scan_complete",
        });
        let id = formatter.next_event_id();
        formatter.write_event(writer, "close", &close_event, id)?;
        
        Ok(())
    }
}