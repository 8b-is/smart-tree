//! MCP handlers for Google authentication operations

use anyhow::Result;
use serde_json::{json, Value};
use std::sync::Arc;

use crate::mcp::McpContext;

/// Check authentication status
pub async fn handle_auth_status(_params: Value, _ctx: Arc<McpContext>) -> Result<Value> {
    let auth = crate::google_sync::auth::GoogleAuth::default_store()?;
    let status = auth.status_summary();

    Ok(json!({
        "content": [{
            "type": "text",
            "text": format!("Google Auth Status\n==================\n{}", status)
        }]
    }))
}

/// Start OAuth2 or service account authentication
pub async fn handle_auth_login(params: Value, _ctx: Arc<McpContext>) -> Result<Value> {
    let auth = crate::google_sync::auth::GoogleAuth::default_store()?;

    let auth_method = params["auth_method"]
        .as_str()
        .unwrap_or("user_oauth2");

    match auth_method {
        "user_oauth2" => {
            let client_id = params["client_id"]
                .as_str()
                .ok_or_else(|| anyhow::anyhow!("client_id is required for user_oauth2"))?;
            let client_secret = params["client_secret"]
                .as_str()
                .ok_or_else(|| anyhow::anyhow!("client_secret is required for user_oauth2"))?;
            let local_browser = params["local_browser"].as_bool().unwrap_or(true);
            let port = params["redirect_port"].as_u64().map(|p| p as u16);

            let _authenticator = auth
                .authenticate_user(client_id, client_secret, local_browser, port)
                .await?;

            let mode = if local_browser {
                "Browser opened for Google sign-in"
            } else {
                "Check your terminal for the authorization URL"
            };

            Ok(json!({
                "content": [{
                    "type": "text",
                    "text": format!(
                        "Google OAuth2 Authentication\n============================\n{}\n\n\
                         Scopes requested: Gmail (read/modify/labels) + Drive (full access)\n\
                         Tokens will be encrypted and stored at ~/.st/google/\n\n\
                         Your data is safe with me. I live here with you.",
                        mode
                    )
                }]
            }))
        }
        "service_account" => {
            let key_path = params["key_path"]
                .as_str()
                .ok_or_else(|| anyhow::anyhow!("key_path is required for service_account"))?;
            let delegate = params["delegate_email"].as_str();

            let _authenticator = auth
                .authenticate_service_account(key_path, delegate)
                .await?;

            Ok(json!({
                "content": [{
                    "type": "text",
                    "text": format!(
                        "Service Account Authentication\n==============================\n\
                         Key: {}\n\
                         Delegate: {}\n\n\
                         Authenticated successfully. Ready for Gmail and Drive operations.",
                        key_path,
                        delegate.unwrap_or("none")
                    )
                }]
            }))
        }
        _ => Err(anyhow::anyhow!(
            "Unknown auth_method: '{}'. Use 'user_oauth2' or 'service_account'",
            auth_method
        )),
    }
}

/// Revoke and clear stored credentials
pub async fn handle_auth_logout(_params: Value, _ctx: Arc<McpContext>) -> Result<Value> {
    let auth = crate::google_sync::auth::GoogleAuth::default_store()?;
    auth.logout()?;

    Ok(json!({
        "content": [{
            "type": "text",
            "text": "Google credentials cleared.\n\
                     Tokens removed from ~/.st/google/\n\
                     You'll need to auth_login again to use Gmail/Drive features."
        }]
    }))
}
