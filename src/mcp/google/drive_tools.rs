//! MCP handlers for Google Drive operations

use anyhow::Result;
use serde_json::{json, Value};
use std::sync::Arc;

use crate::mcp::McpContext;

/// List files in a Drive folder
pub async fn handle_drive_list(params: Value, _ctx: Arc<McpContext>) -> Result<Value> {
    let auth = crate::google_sync::auth::GoogleAuth::default_store()?;
    if !auth.has_cached_tokens() {
        return Ok(super::gmail_tools::not_authenticated_response());
    }

    let folder_id = params["folder_id"]
        .as_str()
        .or_else(|| params["drive_folder_id"].as_str())
        .unwrap_or("root");

    let authenticator = super::gmail_tools::rebuild_authenticator(&auth).await?;
    let drive = crate::google_sync::drive_client::DriveClient::new(authenticator);

    let (files, next_page) = drive.list_files(folder_id, None).await?;

    let mut output = format!("Drive Folder: {} ({} items)\n", folder_id, files.len());
    output.push_str(&"=".repeat(40));
    output.push('\n');

    for file in &files {
        let icon = if file.is_folder { "dir" } else { "   " };
        let size = file
            .size
            .map(|s| humansize::format_size(s, humansize::BINARY))
            .unwrap_or_default();
        let modified = file
            .modified_time
            .map(|t| t.format("%Y-%m-%d %H:%M").to_string())
            .unwrap_or_default();

        output.push_str(&format!(
            "  [{icon}] {name:<40} {size:>10} {modified}\n       ID: {id}\n",
            icon = icon,
            name = file.name,
            size = size,
            modified = modified,
            id = file.id,
        ));
    }

    if let Some(token) = next_page {
        output.push_str(&format!("\n(More results, page token: {})", token));
    }

    Ok(json!({
        "content": [{ "type": "text", "text": output }]
    }))
}

/// Upload a local file to Drive
pub async fn handle_drive_upload(params: Value, _ctx: Arc<McpContext>) -> Result<Value> {
    let auth = crate::google_sync::auth::GoogleAuth::default_store()?;
    if !auth.has_cached_tokens() {
        return Ok(super::gmail_tools::not_authenticated_response());
    }

    let local_path = params["local_path"]
        .as_str()
        .ok_or_else(|| anyhow::anyhow!("'local_path' required for drive_upload"))?;
    let folder_id = params["folder_id"]
        .as_str()
        .or_else(|| params["drive_folder_id"].as_str())
        .unwrap_or("root");

    let expanded = shellexpand::tilde(local_path);
    let path = std::path::Path::new(expanded.as_ref());

    if !path.is_file() {
        return Err(anyhow::anyhow!("Not a file: {}", local_path));
    }

    let content = std::fs::read(path)?;
    let file_name = params["file_name"]
        .as_str()
        .or_else(|| path.file_name().and_then(|n| n.to_str()))
        .unwrap_or("uploaded_file");

    let mime = mime_guess::from_path(path)
        .first_or_octet_stream()
        .to_string();

    let authenticator = super::gmail_tools::rebuild_authenticator(&auth).await?;
    let drive = crate::google_sync::drive_client::DriveClient::new(authenticator);

    let file_id = drive
        .upload_file(folder_id, file_name, &content, &mime)
        .await?;

    Ok(json!({
        "content": [{
            "type": "text",
            "text": format!(
                "Uploaded to Google Drive\n=======================\n\
                 File: {}\n\
                 Size: {}\n\
                 Drive ID: {}\n\
                 Folder: {}\n\n\
                 Your file is safe in the cloud now!",
                file_name,
                humansize::format_size(content.len() as u64, humansize::BINARY),
                file_id,
                folder_id
            )
        }]
    }))
}

/// Download a Drive file to local
pub async fn handle_drive_download(params: Value, _ctx: Arc<McpContext>) -> Result<Value> {
    let auth = crate::google_sync::auth::GoogleAuth::default_store()?;
    if !auth.has_cached_tokens() {
        return Ok(super::gmail_tools::not_authenticated_response());
    }

    let file_id = params["file_id"]
        .as_str()
        .ok_or_else(|| anyhow::anyhow!("'file_id' required for drive_download"))?;
    let local_path = params["local_path"]
        .as_str()
        .ok_or_else(|| anyhow::anyhow!("'local_path' required for drive_download"))?;

    let authenticator = super::gmail_tools::rebuild_authenticator(&auth).await?;
    let drive = crate::google_sync::drive_client::DriveClient::new(authenticator);

    // Get metadata first for the filename
    let metadata = drive.get_file_metadata(file_id).await?;
    let content = drive.download_file(file_id).await?;

    let expanded = shellexpand::tilde(local_path);
    let dest = std::path::Path::new(expanded.as_ref());

    // If local_path is a directory, append the filename
    let final_path = if dest.is_dir() {
        dest.join(&metadata.name)
    } else {
        dest.to_path_buf()
    };

    if let Some(parent) = final_path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    std::fs::write(&final_path, &content)?;

    Ok(json!({
        "content": [{
            "type": "text",
            "text": format!(
                "Downloaded from Google Drive\n===========================\n\
                 File: {}\n\
                 Size: {}\n\
                 Saved to: {}\n\n\
                 Got it! File is home safe.",
                metadata.name,
                humansize::format_size(content.len() as u64, humansize::BINARY),
                final_path.display()
            )
        }]
    }))
}

/// Start or run bidirectional sync
pub async fn handle_drive_sync(params: Value, _ctx: Arc<McpContext>) -> Result<Value> {
    let auth = crate::google_sync::auth::GoogleAuth::default_store()?;
    if !auth.has_cached_tokens() {
        return Ok(super::gmail_tools::not_authenticated_response());
    }

    let local_path = params["local_path"]
        .as_str()
        .ok_or_else(|| anyhow::anyhow!("'local_path' required for drive_sync"))?;
    let folder_id = params["folder_id"]
        .as_str()
        .or_else(|| params["drive_folder_id"].as_str())
        .ok_or_else(|| anyhow::anyhow!("'folder_id' required for drive_sync"))?;

    let direction = match params["sync_direction"].as_str().unwrap_or("bidirectional") {
        "local_to_drive" => crate::google_sync::models::SyncDirection::LocalToDrive,
        "drive_to_local" => crate::google_sync::models::SyncDirection::DriveToLocal,
        _ => crate::google_sync::models::SyncDirection::Bidirectional,
    };

    let conflict_strategy = match params["conflict_strategy"].as_str().unwrap_or("newer_wins") {
        "local_wins" => crate::google_sync::models::ConflictStrategy::LocalWins,
        "drive_wins" => crate::google_sync::models::ConflictStrategy::DriveWins,
        "ask_user" => crate::google_sync::models::ConflictStrategy::AskUser,
        "keep_both" => crate::google_sync::models::ConflictStrategy::KeepBoth,
        _ => crate::google_sync::models::ConflictStrategy::NewerWins,
    };

    let expanded = shellexpand::tilde(local_path);
    let sync_id = format!("sync_{}", expanded.replace(['/', '\\', ' '], "_"));

    let mut state = crate::google_sync::models::SyncState {
        sync_id,
        local_path: expanded.to_string(),
        drive_folder_id: folder_id.to_string(),
        last_sync: None,
        direction,
        file_states: Vec::new(),
        conflict_resolution: conflict_strategy,
    };

    let authenticator = super::gmail_tools::rebuild_authenticator(&auth).await?;
    let drive = crate::google_sync::drive_client::DriveClient::new(authenticator);
    let engine = crate::google_sync::sync_engine::SyncEngine::new(&drive)?;

    let report = engine.sync(&mut state).await?;

    let mut output = "Drive Sync Complete\n===================\n".to_string();
    output.push_str(&format!("  Uploaded: {}\n", report.uploaded));
    output.push_str(&format!("  Downloaded: {}\n", report.downloaded));
    output.push_str(&format!("  Conflicts: {}\n", report.conflicts));
    output.push_str(&format!("  Skipped: {}\n", report.skipped));
    output.push_str(&format!("  Duration: {}ms\n", report.duration_ms));

    if !report.errors.is_empty() {
        output.push_str("\nErrors:\n");
        for error in &report.errors {
            output.push_str(&format!("  - {}\n", error));
        }
    }

    output.push_str("\nYour files are synced. Everything's in its place!");

    Ok(json!({
        "content": [{ "type": "text", "text": output }]
    }))
}

/// Show sync status
pub async fn handle_drive_sync_status(params: Value, _ctx: Arc<McpContext>) -> Result<Value> {
    let local_path = params["local_path"].as_str().unwrap_or("");

    let state_manager = crate::google_sync::sync_state::SyncStateManager::default_dir()?;
    let states = state_manager.list()?;

    if states.is_empty() {
        return Ok(json!({
            "content": [{
                "type": "text",
                "text": "No sync mappings configured. Use drive_sync to set one up."
            }]
        }));
    }

    let mut output = "Drive Sync Status\n=================\n\n".to_string();

    for state in &states {
        if !local_path.is_empty() && !state.local_path.contains(local_path) {
            continue;
        }
        let last = state
            .last_sync
            .map(|t| t.format("%Y-%m-%d %H:%M UTC").to_string())
            .unwrap_or_else(|| "never".to_string());

        output.push_str(&format!(
            "{} <-> Drive:{}\n  Direction: {:?}\n  Last sync: {}\n  Files: {}\n  Strategy: {:?}\n\n",
            state.local_path,
            state.drive_folder_id,
            state.direction,
            last,
            state.file_states.len(),
            state.conflict_resolution,
        ));
    }

    Ok(json!({
        "content": [{ "type": "text", "text": output }]
    }))
}

/// Search Drive files
pub async fn handle_drive_search(params: Value, _ctx: Arc<McpContext>) -> Result<Value> {
    let auth = crate::google_sync::auth::GoogleAuth::default_store()?;
    if !auth.has_cached_tokens() {
        return Ok(super::gmail_tools::not_authenticated_response());
    }

    let query = params["query"]
        .as_str()
        .ok_or_else(|| anyhow::anyhow!("'query' required for drive_search"))?;

    let authenticator = super::gmail_tools::rebuild_authenticator(&auth).await?;
    let drive = crate::google_sync::drive_client::DriveClient::new(authenticator);

    let files = drive.search(query).await?;

    let mut output = format!("Drive Search: '{}' - {} results\n", query, files.len());
    output.push_str(&"=".repeat(40));
    output.push('\n');

    for file in &files {
        let size = file
            .size
            .map(|s| humansize::format_size(s, humansize::BINARY))
            .unwrap_or_default();
        output.push_str(&format!(
            "\n  {} ({}) {}\n  ID: {}\n",
            file.name, file.mime_type, size, file.id
        ));
    }

    Ok(json!({
        "content": [{ "type": "text", "text": output }]
    }))
}
