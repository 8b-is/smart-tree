//! File history tracking tools
//!
//! Contains track_file_operation, get_file_history, and get_project_history_summary handlers.

use super::definitions::{GetFileHistoryArgs, GetProjectHistorySummaryArgs, TrackFileOperationArgs};
use crate::mcp::{is_path_allowed, McpContext};
use anyhow::Result;
use serde_json::{json, Value};
use std::path::PathBuf;
use std::sync::Arc;
use std::time::SystemTime;

/// Track file operations with hash-based change detection
pub async fn track_file_operation(args: Value, ctx: Arc<McpContext>) -> Result<Value> {
    let args: TrackFileOperationArgs = serde_json::from_value(args)?;
    let path = PathBuf::from(&args.file_path);

    // Check if path is allowed
    if !is_path_allowed(&path, &ctx.config) {
        return Err(anyhow::anyhow!("Path not allowed: {}", path.display()));
    }

    // Import file history types
    use crate::file_history::FileHistoryTracker;

    // Create tracker
    let tracker = FileHistoryTracker::new()?;

    // Generate session ID if not provided (safe - EPOCH always in past)
    let session_id = args.session_id.unwrap_or_else(|| {
        let now = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        format!("mcp_{}", now)
    });

    // Determine operation
    if let Some(op_str) = args.operation {
        match op_str.as_str() {
            "read" => {
                let hash = tracker.track_read(&path, &args.agent, &session_id)?;
                Ok(json!({
                    "content": [{
                        "type": "text",
                        "text": format!("✓ Tracked read operation for {}\nFile hash: {}", path.display(), hash)
                    }]
                }))
            }
            "write" | "append" | "prepend" | "insert" | "delete" | "replace" | "create"
            | "remove" => {
                // These require content
                if args.new_content.is_none() && op_str != "remove" {
                    return Err(anyhow::anyhow!(
                        "new_content required for {} operation",
                        op_str
                    ));
                }

                let op = tracker.track_write(
                    &path,
                    args.old_content.as_deref(),
                    args.new_content.as_deref().unwrap_or(""),
                    &args.agent,
                    &session_id,
                )?;

                Ok(json!({
                    "content": [{
                        "type": "text",
                        "text": format!("✓ Tracked {} operation for {}\nOperation: {}", op_str, path.display(), op)
                    }]
                }))
            }
            _ => Err(anyhow::anyhow!("Unknown operation: {}", op_str)),
        }
    } else {
        // Auto-detect operation from content - require new_content
        let new_content = args
            .new_content
            .as_deref()
            .ok_or_else(|| anyhow::anyhow!("Either operation or new_content must be provided"))?;

        let op = tracker.track_write(
            &path,
            args.old_content.as_deref(),
            new_content,
            &args.agent,
            &session_id,
        )?;

        Ok(json!({
            "content": [{
                "type": "text",
                "text": format!("✓ Auto-tracked operation for {}\nDetected operation: {}\nAgent: {}\nSession: {}",
                    path.display(), op, args.agent, session_id)
            }]
        }))
    }
}

/// Get complete operation history for a file
pub async fn get_file_history(args: Value, ctx: Arc<McpContext>) -> Result<Value> {
    let args: GetFileHistoryArgs = serde_json::from_value(args)?;
    let path = PathBuf::from(&args.file_path);

    // Check if path is allowed
    if !is_path_allowed(&path, &ctx.config) {
        return Err(anyhow::anyhow!("Path not allowed: {}", path.display()));
    }

    use crate::file_history::FileHistoryTracker;

    let tracker = FileHistoryTracker::new()?;
    let history = tracker.get_file_history(&path)?;

    let mut output = format!("📜 File History for {}\n\n", path.display());

    if history.is_empty() {
        output.push_str("No history found for this file.");
    } else {
        output.push_str(&format!("Found {} operations:\n\n", history.len()));

        for (i, entry) in history.iter().enumerate() {
            let datetime = chrono::DateTime::<chrono::Utc>::from(
                SystemTime::UNIX_EPOCH + std::time::Duration::from_secs(entry.timestamp),
            );

            output.push_str(&format!(
                "{}. [{}] {} - {}\n   Agent: {}, Session: {}\n   Bytes affected: {}\n",
                i + 1,
                datetime.format("%Y-%m-%d %H:%M:%S"),
                entry.operation.code(),
                entry.operation.description(),
                entry.agent,
                entry.session_id,
                entry.context.bytes_affected
            ));

            if let Some(old_hash) = &entry.context.old_hash {
                output.push_str(&format!("   Old hash: {}\n", &old_hash[..8]));
            }
            if let Some(new_hash) = &entry.context.new_hash {
                output.push_str(&format!("   New hash: {}\n", &new_hash[..8]));
            }
            output.push('\n');
        }
    }

    Ok(json!({
        "content": [{
            "type": "text",
            "text": output
        }],
        "metadata": {
            "operation_count": history.len(),
            "file_path": path.to_string_lossy()
        }
    }))
}

/// Get summary of all AI operations in a project
pub async fn get_project_history_summary(args: Value, ctx: Arc<McpContext>) -> Result<Value> {
    let args: GetProjectHistorySummaryArgs = serde_json::from_value(args)?;
    let path = PathBuf::from(&args.project_path);

    // Check if path is allowed
    if !is_path_allowed(&path, &ctx.config) {
        return Err(anyhow::anyhow!("Path not allowed: {}", path.display()));
    }

    use crate::file_history::FileHistoryTracker;

    let tracker = FileHistoryTracker::new()?;
    let summary = tracker.get_project_summary(&path)?;

    let mut output = format!("📊 Project History Summary for {}\n\n", path.display());
    output.push_str(&format!("Total operations: {}\n", summary.total_operations));
    output.push_str(&format!("Files modified: {}\n\n", summary.files_modified));

    if !summary.operation_counts.is_empty() {
        output.push_str("Operations breakdown:\n");
        let mut ops: Vec<_> = summary.operation_counts.iter().collect();
        ops.sort_by_key(|(_, count)| std::cmp::Reverse(**count));

        for (op, count) in ops {
            output.push_str(&format!("  {} ({}): {} times\n", op, op.code(), count));
        }
    }

    Ok(json!({
        "content": [{
            "type": "text",
            "text": output
        }],
        "metadata": summary
    }))
}
