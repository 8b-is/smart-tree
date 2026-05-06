use crate::mcp::McpContext;
use anyhow::{Context, Result};
use serde_json::{json, Value};
use std::sync::Arc;
use std::env;

/// Send a prompt to the user via the local dashboard and wait for their response
pub async fn ask_user(params: Option<Value>, _ctx: Arc<McpContext>) -> Result<Value> {
    let params = params.context("Parameters required")?;
    let question = params["question"]
        .as_str()
        .context("question parameter required")?;

    let port = env::var("ST_DASHBOARD_PORT").unwrap_or_else(|_| "8765".to_string());
    let url = format!("http://127.0.0.1:{}/api/prompt", port);

    // Make an HTTP POST request to the local dashboard
    let client = reqwest::Client::new();
    let response = client
        .post(&url)
        .json(&json!({ "question": question }))
        .send()
        .await
        .context("Failed to connect to dashboard. Is the Smart Tree web dashboard running?")?;

    if !response.status().is_success() {
        let status = response.status();
        let text = response.text().await.unwrap_or_default();
        return Err(anyhow::anyhow!("Dashboard returned error {}: {}", status, text));
    }

    // Parse the response back
    let response_json: Value = response.json().await.context("Failed to parse dashboard response")?;
    
    // Return the answer as plain text or JSON
    let answer = response_json["answer"].as_str().unwrap_or("").to_string();
    
    Ok(json!({
        "answer": answer
    }))
}
