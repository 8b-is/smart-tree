// Smart Edit Diff Viewer - View stored diffs for files

use crate::smart_edit_diff::DiffStorage;
use anyhow::{Context, Result};
use serde_json::{json, Value};
use std::path::Path;

/// Handle viewing diffs for a file
pub async fn _handle_view_file_diffs(params: Option<Value>) -> Result<Value> {
    let params = params.context("Parameters required")?;

    let file_path = params["file_path"].as_str().context("file_path required")?;

    let project_root = params
        .get("project_root")
        .and_then(|p| p.as_str())
        .map(Path::new)
        .unwrap_or_else(|| Path::new("."));

    let storage = DiffStorage::new(project_root)?;
    let diffs = storage.list_diffs(Path::new(file_path))?;

    let diff_list: Vec<Value> = diffs
        .into_iter()
        .map(|diff_info| {
            json!({
                "timestamp": diff_info.timestamp,
                "timestamp_str": diff_info.timestamp_str(),
                "diff_file": diff_info.path.file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or("unknown"),
                "full_path": diff_info.path.to_string_lossy(),
            })
        })
        .collect();

    Ok(json!({
        "file_path": file_path,
        "diff_count": diff_list.len(),
        "diffs": diff_list,
        "st_folder": storage.st_folder.to_string_lossy(),
    }))
}

/// Handle viewing a specific diff
pub async fn _handle_view_diff_content(params: Option<Value>) -> Result<Value> {
    let params = params.context("Parameters required")?;

    let diff_path = params["diff_path"].as_str().context("diff_path required")?;

    let content = std::fs::read_to_string(diff_path)?;

    Ok(json!({
        "diff_path": diff_path,
        "content": content,
        "lines": content.lines().count(),
    }))
}

/// Handle cleaning up old diffs
pub async fn _handle_cleanup_diffs(params: Option<Value>) -> Result<Value> {
    let params = params.context("Parameters required")?;

    let project_root = params
        .get("project_root")
        .and_then(|p| p.as_str())
        .map(Path::new)
        .unwrap_or_else(|| Path::new("."));

    let keep_count = params
        .get("keep_count")
        .and_then(|k| k.as_u64())
        .unwrap_or(10) as usize;

    let storage = DiffStorage::new(project_root)?;
    let removed = storage.cleanup_old_diffs(keep_count)?;

    Ok(json!({
        "removed_count": removed,
        "keep_count": keep_count,
        "message": format!("Removed {} old diff files", removed),
    }))
}

/// Handle getting diff statistics
pub async fn _handle_diff_stats(params: Option<Value>) -> Result<Value> {
    let params = params.context("Parameters required")?;

    let project_root = params
        .get("project_root")
        .and_then(|p| p.as_str())
        .map(Path::new)
        .unwrap_or_else(|| Path::new("."));

    let storage = DiffStorage::new(project_root)?;

    // Count all diffs in .st folder
    let mut total_diffs = 0;
    let mut total_size = 0u64;
    let mut file_stats = std::collections::HashMap::new();

    for entry in std::fs::read_dir(&storage.st_folder)? {
        let entry = entry?;
        let metadata = entry.metadata()?;

        if metadata.is_file() {
            total_diffs += 1;
            total_size += metadata.len();

            let name = entry.file_name().to_string_lossy().to_string();
            if let Some(pos) = name.rfind('-') {
                let base = &name[..pos];
                *file_stats.entry(base.to_string()).or_insert(0) += 1;
            }
        }
    }

    Ok(json!({
        "project_root": project_root.to_string_lossy(),
        "st_folder": storage.st_folder.to_string_lossy(),
        "total_diffs": total_diffs,
        "total_size_bytes": total_size,
        "total_size_human": humansize::format_size(total_size, humansize::BINARY),
        "files_tracked": file_stats.len(),
        "file_stats": file_stats,
    }))
}
