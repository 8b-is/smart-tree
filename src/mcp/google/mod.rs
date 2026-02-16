//! Google MCP Tools - Consolidated handler for Gmail, Drive, and File Intelligence
//!
//! All operations are dispatched through a single `google` consolidated tool
//! with an `operation` parameter routing to the correct handler.

pub mod auth_tools;
pub mod drive_tools;
pub mod gmail_tools;

use anyhow::{Context, Result};
use serde_json::{json, Value};
use std::sync::Arc;

use crate::mcp::McpContext;

/// Consolidated Google tools dispatcher
pub async fn handle_google(params: Option<Value>, ctx: Arc<McpContext>) -> Result<Value> {
    let params = params.context("Parameters required for google tool")?;
    let operation = params["operation"]
        .as_str()
        .context("'operation' parameter is required")?;

    match operation {
        // Auth operations
        "auth_status" => auth_tools::handle_auth_status(params, ctx).await,
        "auth_login" => auth_tools::handle_auth_login(params, ctx).await,
        "auth_logout" => auth_tools::handle_auth_logout(params, ctx).await,

        // Gmail operations
        "gmail_list" => gmail_tools::handle_gmail_list(params, ctx).await,
        "gmail_search" => gmail_tools::handle_gmail_search(params, ctx).await,
        "gmail_backup" => gmail_tools::handle_gmail_backup(params, ctx).await,
        "gmail_backup_status" => gmail_tools::handle_gmail_backup_status(params, ctx).await,
        "gmail_warm_analysis" => gmail_tools::handle_warm_analysis(params, ctx).await,
        "gmail_archive" => gmail_tools::handle_gmail_archive(params, ctx).await,

        // Drive operations
        "drive_list" => drive_tools::handle_drive_list(params, ctx).await,
        "drive_upload" => drive_tools::handle_drive_upload(params, ctx).await,
        "drive_download" => drive_tools::handle_drive_download(params, ctx).await,
        "drive_sync" => drive_tools::handle_drive_sync(params, ctx).await,
        "drive_sync_status" => drive_tools::handle_drive_sync_status(params, ctx).await,
        "drive_search" => drive_tools::handle_drive_search(params, ctx).await,

        // File intelligence
        "suggest_filing" => gmail_tools::handle_suggest_filing(params, ctx).await,
        "email_triage" => gmail_tools::handle_email_triage(params, ctx).await,

        _ => Err(anyhow::anyhow!(
            "Unknown google operation: '{}'. Valid operations: auth_status, auth_login, auth_logout, \
             gmail_list, gmail_search, gmail_backup, gmail_backup_status, gmail_warm_analysis, gmail_archive, \
             drive_list, drive_upload, drive_download, drive_sync, drive_sync_status, drive_search, \
             suggest_filing, email_triage",
            operation
        )),
    }
}

/// Get the consolidated Google tool definition for MCP tool listing
pub fn get_google_tool_definition() -> Value {
    json!({
        "name": "google",
        "description": "Google Drive & Gmail integration - backup emails, sync files, smart archive suggestions, file organization intelligence. Operations: auth_status, auth_login, auth_logout, gmail_list, gmail_search, gmail_backup, gmail_backup_status, gmail_warm_analysis, gmail_archive, drive_list, drive_upload, drive_download, drive_sync, drive_sync_status, drive_search, suggest_filing, email_triage",
        "inputSchema": {
            "type": "object",
            "properties": {
                "operation": {
                    "type": "string",
                    "enum": [
                        "auth_status", "auth_login", "auth_logout",
                        "gmail_list", "gmail_search", "gmail_backup",
                        "gmail_backup_status", "gmail_warm_analysis", "gmail_archive",
                        "drive_list", "drive_upload", "drive_download",
                        "drive_sync", "drive_sync_status", "drive_search",
                        "suggest_filing", "email_triage"
                    ],
                    "description": "Google operation to perform"
                },
                "auth_method": {
                    "type": "string",
                    "enum": ["user_oauth2", "service_account"],
                    "description": "Authentication method (for auth_login)"
                },
                "client_id": { "type": "string", "description": "OAuth2 client ID" },
                "client_secret": { "type": "string", "description": "OAuth2 client secret" },
                "key_path": { "type": "string", "description": "Service account key file path" },
                "local_browser": {
                    "type": "boolean",
                    "description": "Open browser locally (true) or show URL for remote/SSH (false)",
                    "default": true
                },
                "redirect_port": {
                    "type": "integer",
                    "description": "Port for OAuth2 redirect (for port forwarding scenarios)"
                },
                "query": { "type": "string", "description": "Gmail search query or Drive search query" },
                "label": { "type": "string", "description": "Gmail label filter" },
                "max_results": { "type": "integer", "description": "Maximum results to return", "default": 50 },
                "message_ids": {
                    "type": "array",
                    "items": { "type": "string" },
                    "description": "Message IDs for batch operations (gmail_archive)"
                },
                "drive_folder_id": { "type": "string", "description": "Drive folder ID" },
                "folder_id": { "type": "string", "description": "Drive folder ID (alias)" },
                "file_id": { "type": "string", "description": "Drive file ID" },
                "local_path": { "type": "string", "description": "Local file or directory path" },
                "file_name": { "type": "string", "description": "File name for upload" },
                "sync_direction": {
                    "type": "string",
                    "enum": ["local_to_drive", "drive_to_local", "bidirectional"],
                    "default": "bidirectional"
                },
                "conflict_strategy": {
                    "type": "string",
                    "enum": ["newer_wins", "local_wins", "drive_wins", "ask_user", "keep_both"],
                    "default": "newer_wins"
                },
                "min_score": {
                    "type": "number",
                    "description": "Minimum score to suggest archiving (0.0-1.0)",
                    "default": 0.5
                },
                "dry_run": {
                    "type": "boolean",
                    "description": "Preview changes without executing",
                    "default": true
                },
                "path": {
                    "type": "string",
                    "description": "Path for suggest_filing operation"
                }
            },
            "required": ["operation"]
        }
    })
}
