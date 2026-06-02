//! MCP handlers for Gmail operations and file intelligence

use anyhow::Result;
use serde_json::{json, Value};
use std::sync::Arc;

use crate::mcp::McpContext;

/// List recent emails
pub async fn handle_gmail_list(params: Value, _ctx: Arc<McpContext>) -> Result<Value> {
    let auth = crate::google_sync::auth::GoogleAuth::default_store()?;
    if !auth.has_cached_tokens() {
        return Ok(not_authenticated_response());
    }

    let query = params["query"].as_str();
    let label = params["label"].as_str();
    let max_results = params["max_results"].as_u64().unwrap_or(20) as u32;

    // Build label filter
    let labels: Option<Vec<&str>> = label.map(|l| vec![l]);

    let authenticator = rebuild_authenticator(&auth).await?;
    let client = crate::google_sync::gmail_client::GmailClient::new(authenticator);

    let (emails, next_page) = client
        .list_messages(query, labels.as_deref(), max_results, None)
        .await?;

    let mut output = format!("Gmail - {} messages found\n", emails.len());
    output.push_str("========================\n\n");

    for email in &emails {
        let read_marker = if email.is_read { " " } else { "*" };
        let attach = if email.has_attachments { " [att]" } else { "" };
        output.push_str(&format!(
            "{} {} | {} | {}{}\n   {}\n\n",
            read_marker,
            email.date.format("%Y-%m-%d %H:%M"),
            email.from,
            email.subject,
            attach,
            email.snippet,
        ));
    }

    if let Some(token) = next_page {
        output.push_str(&format!(
            "\n(More results available, page token: {})",
            token
        ));
    }

    Ok(json!({
        "content": [{ "type": "text", "text": output }]
    }))
}

/// Search emails
pub async fn handle_gmail_search(params: Value, _ctx: Arc<McpContext>) -> Result<Value> {
    let auth = crate::google_sync::auth::GoogleAuth::default_store()?;
    if !auth.has_cached_tokens() {
        return Ok(not_authenticated_response());
    }

    let query = params["query"]
        .as_str()
        .ok_or_else(|| anyhow::anyhow!("'query' parameter required for gmail_search"))?;
    let max_results = params["max_results"].as_u64().unwrap_or(20) as u32;

    let authenticator = rebuild_authenticator(&auth).await?;
    let client = crate::google_sync::gmail_client::GmailClient::new(authenticator);

    let emails = client.search(query, max_results).await?;

    let mut output = format!("Gmail Search: '{}' - {} results\n", query, emails.len());
    output.push_str(&"=".repeat(40));
    output.push('\n');

    for email in &emails {
        output.push_str(&format!(
            "\n[{}] {}\n  From: {}\n  Date: {}\n  {}\n",
            email.message_id,
            email.subject,
            email.from,
            email.date.format("%Y-%m-%d %H:%M"),
            email.snippet,
        ));
    }

    Ok(json!({
        "content": [{ "type": "text", "text": output }]
    }))
}

/// Backup emails to Google Drive as .eml files
pub async fn handle_gmail_backup(params: Value, _ctx: Arc<McpContext>) -> Result<Value> {
    let auth = crate::google_sync::auth::GoogleAuth::default_store()?;
    if !auth.has_cached_tokens() {
        return Ok(not_authenticated_response());
    }

    let query = params["query"].as_str();
    let label = params["label"].as_str();
    let drive_folder_id = params["drive_folder_id"]
        .as_str()
        .ok_or_else(|| anyhow::anyhow!("'drive_folder_id' required for gmail_backup"))?;
    let max_results = params["max_results"].as_u64().unwrap_or(50) as u32;

    let authenticator = rebuild_authenticator(&auth).await?;
    let gmail = crate::google_sync::gmail_client::GmailClient::new(authenticator.clone());
    let drive = crate::google_sync::drive_client::DriveClient::new(authenticator);

    // List messages to back up
    let labels: Option<Vec<&str>> = label.map(|l| vec![l]);
    let (emails, _) = gmail
        .list_messages(query, labels.as_deref(), max_results, None)
        .await?;

    let total = emails.len();
    let mut backed_up = 0u32;
    let mut failed = 0u32;

    for email in &emails {
        match gmail.get_message_raw(&email.message_id).await {
            Ok(raw_bytes) => {
                let filename = format!(
                    "{}_{}.eml",
                    email.date.format("%Y%m%d_%H%M%S"),
                    &email.message_id[..8.min(email.message_id.len())]
                );
                match drive
                    .upload_file(drive_folder_id, &filename, &raw_bytes, "message/rfc822")
                    .await
                {
                    Ok(_) => backed_up += 1,
                    Err(e) => {
                        tracing::warn!("Failed to upload {}: {}", email.message_id, e);
                        failed += 1;
                    }
                }
            }
            Err(e) => {
                tracing::warn!("Failed to download {}: {}", email.message_id, e);
                failed += 1;
            }
        }
    }

    Ok(json!({
        "content": [{
            "type": "text",
            "text": format!(
                "Gmail Backup Complete\n====================\n\
                 Total: {}\n\
                 Backed up: {}\n\
                 Failed: {}\n\
                 Drive folder: {}\n\n\
                 Your emails are safe. They live with you now.",
                total, backed_up, failed, drive_folder_id
            )
        }]
    }))
}

/// Check backup progress
pub async fn handle_gmail_backup_status(_params: Value, _ctx: Arc<McpContext>) -> Result<Value> {
    let progress_dir = crate::google_sync::google_state_dir().join("backup_progress");
    if !progress_dir.exists() {
        return Ok(json!({
            "content": [{ "type": "text", "text": "No backup progress found. Start a backup with gmail_backup." }]
        }));
    }

    let mut output = "Gmail Backup Status\n===================\n".to_string();
    for entry in std::fs::read_dir(&progress_dir)? {
        let entry = entry?;
        if let Ok(json) = std::fs::read_to_string(entry.path()) {
            if let Ok(progress) =
                serde_json::from_str::<crate::google_sync::models::BackupProgress>(&json)
            {
                output.push_str(&format!(
                    "\nBackup: {}/{} messages (started: {})\n",
                    progress.backed_up,
                    progress.total_messages,
                    progress.started_at.format("%Y-%m-%d %H:%M")
                ));
            }
        }
    }

    Ok(json!({
        "content": [{ "type": "text", "text": output }]
    }))
}

/// Analyze emails for warm storage suggestions
pub async fn handle_warm_analysis(params: Value, _ctx: Arc<McpContext>) -> Result<Value> {
    let auth = crate::google_sync::auth::GoogleAuth::default_store()?;
    if !auth.has_cached_tokens() {
        return Ok(not_authenticated_response());
    }

    let max_results = params["max_results"].as_u64().unwrap_or(100) as u32;
    let min_score = params["min_score"].as_f64().unwrap_or(0.5);

    let authenticator = rebuild_authenticator(&auth).await?;
    let client = crate::google_sync::gmail_client::GmailClient::new(authenticator);

    // Fetch emails for analysis
    let (emails, _) = client
        .list_messages(None, Some(&["INBOX"]), max_results, None)
        .await?;

    let config = crate::google_sync::warm_analyzer::WarmConfig {
        min_score_to_suggest: min_score,
        ..Default::default()
    };

    let analyzer = crate::google_sync::warm_analyzer::WarmAnalyzer::new(config);
    let scores = analyzer.analyze(&emails);

    let summary = crate::google_sync::warm_analyzer::WarmAnalyzer::summarize(&scores);

    Ok(json!({
        "content": [{
            "type": "text",
            "text": format!(
                "Warm Storage Analysis\n====================\n\
                 Analyzed {} emails, {} suggested for archiving\n\n{}\n\n\
                 Use gmail_archive with message_ids to archive these.\n\
                 Don't worry about the noise. I'll help you find what matters!",
                emails.len(),
                scores.len(),
                summary
            )
        }]
    }))
}

/// Archive emails (remove from INBOX)
pub async fn handle_gmail_archive(params: Value, _ctx: Arc<McpContext>) -> Result<Value> {
    let auth = crate::google_sync::auth::GoogleAuth::default_store()?;
    if !auth.has_cached_tokens() {
        return Ok(not_authenticated_response());
    }

    let message_ids: Vec<String> = params["message_ids"]
        .as_array()
        .ok_or_else(|| anyhow::anyhow!("'message_ids' array required for gmail_archive"))?
        .iter()
        .filter_map(|v| v.as_str().map(String::from))
        .collect();

    let dry_run = params["dry_run"].as_bool().unwrap_or(true);

    if dry_run {
        return Ok(json!({
            "content": [{
                "type": "text",
                "text": format!(
                    "DRY RUN - Would archive {} emails\n\
                     Message IDs: {:?}\n\n\
                     Set dry_run=false to actually archive.",
                    message_ids.len(),
                    message_ids
                )
            }]
        }));
    }

    let authenticator = rebuild_authenticator(&auth).await?;
    let client = crate::google_sync::gmail_client::GmailClient::new(authenticator);

    let mut archived = 0u32;
    for id in &message_ids {
        match client.modify_labels(id, &[], &["INBOX"]).await {
            Ok(()) => archived += 1,
            Err(e) => tracing::warn!("Failed to archive {}: {}", id, e),
        }
    }

    Ok(json!({
        "content": [{
            "type": "text",
            "text": format!(
                "Archived {}/{} emails (removed INBOX label)\n\
                 They're still in All Mail, just tidied up!",
                archived,
                message_ids.len()
            )
        }]
    }))
}

/// Suggest filing for misplaced files
pub async fn handle_suggest_filing(params: Value, _ctx: Arc<McpContext>) -> Result<Value> {
    let path = params["path"].as_str().unwrap_or("~/Downloads");

    let expanded = shellexpand::tilde(path);
    let scan_path = std::path::Path::new(expanded.as_ref());

    if !scan_path.is_dir() {
        return Err(anyhow::anyhow!("Not a directory: {}", path));
    }

    let suggestions = crate::google_sync::file_intelligence::suggest_filing(scan_path);

    if suggestions.is_empty() {
        return Ok(json!({
            "content": [{
                "type": "text",
                "text": format!(
                    "Scanned {} - everything looks organized! Nothing to suggest.\n\
                     Your files are in good shape!",
                    path
                )
            }]
        }));
    }

    let mut output = format!(
        "File Organization Suggestions ({})\n{}\n\n",
        path,
        "=".repeat(40)
    );

    for suggestion in &suggestions {
        output.push_str(&format!(
            "[{:.0}%] {}\n  -> {}\n  Reason: {}\n  {}\n\n",
            suggestion.confidence * 100.0,
            suggestion.file_path,
            suggestion.suggested_path,
            suggestion.reason,
            suggestion.personality_message,
        ));
    }

    Ok(json!({
        "content": [{ "type": "text", "text": output }]
    }))
}

/// Triage emails by importance
pub async fn handle_email_triage(params: Value, _ctx: Arc<McpContext>) -> Result<Value> {
    let auth = crate::google_sync::auth::GoogleAuth::default_store()?;
    if !auth.has_cached_tokens() {
        return Ok(not_authenticated_response());
    }

    let max_results = params["max_results"].as_u64().unwrap_or(30) as u32;

    let authenticator = rebuild_authenticator(&auth).await?;
    let client = crate::google_sync::gmail_client::GmailClient::new(authenticator);

    let (emails, _) = client
        .list_messages(None, Some(&["INBOX"]), max_results, None)
        .await?;

    let triages = crate::google_sync::file_intelligence::triage_emails(&emails);

    if triages.is_empty() {
        return Ok(json!({
            "content": [{
                "type": "text",
                "text": "No urgent emails found. You're all caught up!"
            }]
        }));
    }

    let mut output =
        "Email Triage - Needs Your Attention\n===================================\n\n".to_string();

    for triage in &triages {
        output.push_str(&format!(
            "[{:.0}% importance] {}\n  From: {}\n  Action: {}\n  {}\n\n",
            triage.importance_score * 100.0,
            triage.subject,
            triage.from,
            triage.action_suggestion,
            triage.personality_message,
        ));
    }

    Ok(json!({
        "content": [{ "type": "text", "text": output }]
    }))
}

// ── Helpers ─────────────────────────────────────────────────────────

pub(crate) fn not_authenticated_response() -> Value {
    json!({
        "content": [{
            "type": "text",
            "text": "Not authenticated with Google.\n\n\
                     Use the google tool with operation: 'auth_login' to connect:\n\
                     - auth_method: 'user_oauth2' (opens browser)\n\
                     - client_id: your Google Cloud OAuth2 client ID\n\
                     - client_secret: your client secret\n\n\
                     Need help? Visit console.cloud.google.com to create OAuth2 credentials."
        }]
    })
}

/// Rebuild authenticator from stored config
pub(crate) async fn rebuild_authenticator(
    auth: &crate::google_sync::auth::GoogleAuth,
) -> Result<crate::google_sync::auth::GoogleAuthenticator> {
    let config = auth
        .get_config()?
        .ok_or_else(|| anyhow::anyhow!("No stored auth config found"))?;

    match &config.method {
        crate::google_sync::models::AuthMethod::UserOAuth2 {
            client_id,
            client_secret,
            redirect_port,
        } => {
            // For re-auth, we use the cached tokens (no browser needed)
            // yup-oauth2 will auto-refresh from the token cache
            auth.authenticate_user(client_id, client_secret, false, *redirect_port)
                .await
        }
        crate::google_sync::models::AuthMethod::ServiceAccount {
            key_path,
            delegate_email,
        } => {
            auth.authenticate_service_account(key_path, delegate_email.as_deref())
                .await
        }
    }
}
