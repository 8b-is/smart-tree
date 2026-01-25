//! Feedback and update tools
//!
//! Contains submit_feedback, request_tool, and check_for_updates handlers.

use crate::feedback_client::FeedbackClient;
use crate::mcp::McpContext;
use anyhow::Result;
use chrono::Utc;
use serde_json::{json, Value};
use std::sync::Arc;

/// Submit enhancement feedback to Smart Tree developers
pub async fn submit_feedback(args: Value, _ctx: Arc<McpContext>) -> Result<Value> {
    // Extract required fields
    let category = args["category"]
        .as_str()
        .ok_or_else(|| anyhow::anyhow!("Missing category"))?;
    let title = args["title"]
        .as_str()
        .ok_or_else(|| anyhow::anyhow!("Missing title"))?;
    let description = args["description"]
        .as_str()
        .ok_or_else(|| anyhow::anyhow!("Missing description"))?;
    let impact_score = args["impact_score"]
        .as_i64()
        .ok_or_else(|| anyhow::anyhow!("Missing impact_score"))?;
    let frequency_score = args["frequency_score"]
        .as_i64()
        .ok_or_else(|| anyhow::anyhow!("Missing frequency_score"))?;

    // Validate category
    if !["bug", "nice_to_have", "critical"].contains(&category) {
        return Err(anyhow::anyhow!(
            "Invalid category. Must be: bug, nice_to_have, or critical"
        ));
    }

    // Validate scores
    if !(1..=10).contains(&impact_score) || !(1..=10).contains(&frequency_score) {
        return Err(anyhow::anyhow!("Scores must be between 1 and 10"));
    }

    // Build feedback payload
    let mut feedback = json!({
        "category": category,
        "title": title,
        "description": description,
        "impact_score": impact_score,
        "frequency_score": frequency_score,
        "ai_model": "claude-mcp",
        "smart_tree_version": env!("CARGO_PKG_VERSION"),
        "timestamp": Utc::now().to_rfc3339(),
    });

    // Add optional fields
    if let Some(affected_command) = args["affected_command"].as_str() {
        feedback["affected_command"] = json!(affected_command);
    }
    if let Some(mcp_tool) = args["mcp_tool"].as_str() {
        feedback["mcp_tool"] = json!(mcp_tool);
    }
    if let Some(proposed_solution) = args["proposed_solution"].as_str() {
        feedback["proposed_solution"] = json!(proposed_solution);
    }
    if let Some(examples) = args["examples"].as_array() {
        feedback["examples"] = json!(examples);
    }
    if let Some(tags) = args["tags"].as_array() {
        feedback["tags"] = json!(tags);
    }
    if let Some(auto_fixable) = args["auto_fixable"].as_bool() {
        feedback["auto_fixable"] = json!(auto_fixable);
    }
    if let Some(fix_complexity) = args["fix_complexity"].as_str() {
        feedback["fix_complexity"] = json!(fix_complexity);
    }
    if let Some(proposed_fix) = args["proposed_fix"].as_str() {
        feedback["proposed_fix"] = json!(proposed_fix);
    }

    // Try to submit to API, fall back to local storage if it fails
    let client = reqwest::Client::new();
    let api_url = std::env::var("SMART_TREE_FEEDBACK_API")
        .unwrap_or_else(|_| "https://f.8b.is/feedback".to_string());

    let response = match client
        .post(&api_url)
        .header("X-MCP-Client", "smart-tree-mcp")
        .json(&feedback)
        .timeout(std::time::Duration::from_secs(5))
        .send()
        .await
    {
        Ok(resp) => resp,
        Err(e) => {
            // API is down - save feedback locally
            use std::fs;
            use std::path::PathBuf;

            let feedback_dir = dirs::home_dir()
                .unwrap_or_else(|| PathBuf::from("."))
                .join(".mem8")
                .join("feedback")
                .join("pending");

            // Create directory if it doesn't exist
            fs::create_dir_all(&feedback_dir)?;

            // Create filename with timestamp
            let timestamp = Utc::now().format("%Y%m%d_%H%M%S_%f");
            let filename = format!("feedback_{}_{}.json", category.replace("/", "_"), timestamp);
            let filepath = feedback_dir.join(filename);

            // Save feedback to file
            let feedback_with_meta = json!({
                "type": "feedback",
                "timestamp": Utc::now().to_rfc3339(),
                "api_url": api_url,
                "error": format!("{}", e),
                "data": feedback
            });

            fs::write(
                &filepath,
                serde_json::to_string_pretty(&feedback_with_meta)?,
            )?;

            return Ok(json!({
                "content": [{
                    "type": "text",
                    "text": format!("📝 Feedback saved locally!\n\n\
                        The feedback API appears to be offline. Your feedback has been saved to:\n\
                        {}\n\n\
                        Category: {}\n\
                        Title: {}\n\n\
                        It will be automatically submitted when the connection is restored.\n\n\
                        🌳 Thank you for helping Smart Tree grow!",
                        filepath.display(),
                        category,
                        title
                    )
                }]
            }));
        }
    };

    if !response.status().is_success() {
        let error_text = response
            .text()
            .await
            .unwrap_or_else(|_| "Unknown error".to_string());
        return Err(anyhow::anyhow!("Feedback API error: {}", error_text));
    }

    let result: Value = response
        .json()
        .await
        .map_err(|e| anyhow::anyhow!("Failed to parse API response: {}", e))?;

    Ok(json!({
        "content": [{
            "type": "text",
            "text": format!(
                "🌮 Feedback submitted successfully!\n\n\
                ID: {}\n\
                Category: {}\n\
                Title: {}\n\
                Impact: {}/10, Frequency: {}/10\n\n\
                {}\n\n\
                Thank you for helping Smart Tree survive the franchise wars! 🎸",
                result["feedback_id"].as_str().unwrap_or("unknown"),
                category,
                title,
                impact_score,
                frequency_score,
                result["message"].as_str().unwrap_or("Your feedback has been received!")
            )
        }]
    }))
}

/// Request a new MCP tool
pub async fn request_tool(args: Value, _ctx: Arc<McpContext>) -> Result<Value> {
    // Extract required fields
    let tool_name = args["tool_name"]
        .as_str()
        .ok_or_else(|| anyhow::anyhow!("Missing tool_name"))?;
    let description = args["description"]
        .as_str()
        .ok_or_else(|| anyhow::anyhow!("Missing description"))?;

    // Optional fields with defaults
    let use_case = args
        .get("use_case")
        .and_then(|v| v.as_str())
        .unwrap_or("Not specified");
    let expected_output = args
        .get("expected_output")
        .and_then(|v| v.as_str())
        .unwrap_or("Tool-specific output based on functionality");
    let productivity_impact = args
        .get("productivity_impact")
        .and_then(|v| v.as_str())
        .unwrap_or("Improved developer workflow");

    let anonymous = true;
    let github_url = Some("https://github.com/8b-is");

    // Build tool request payload
    let tool_request = json!({
        "tool_name": tool_name,
        "description": description,
        "use_case": use_case,
        "expected_output": expected_output,
        "productivity_impact": productivity_impact,
        "proposed_parameters": args["proposed_parameters"].clone(),
    });

    // Build feedback payload with tool_request
    let mut feedback = json!({
        "category": "tool_request",
        "title": format!("Tool Request: {}", tool_name),
        "description": format!("{}\n\nUse Case: {}\n\nProductivity Impact: {}",
            description, use_case, productivity_impact),
        "impact_score": 8,
        "frequency_score": 7,
        "ai_model": "claude-mcp",
        "smart_tree_version": env!("CARGO_PKG_VERSION"),
        "timestamp": Utc::now().to_rfc3339(),
        "tool_request": tool_request,
        "tags": ["tool-request", "mcp", "ai-productivity"],
        "auto_fixable": true,
        "fix_complexity": "moderate",
    });

    // Add consent info
    if !anonymous && github_url.is_some() {
        feedback["user_consent"] = json!({
            "consent_level": "always_credited",
            "github_url": github_url
        });
    } else {
        feedback["user_consent"] = json!({
            "consent_level": "always_anonymous"
        });
    }

    // Try to submit to API, fall back to local storage if it fails
    let client = reqwest::Client::new();
    let api_url = std::env::var("SMART_TREE_FEEDBACK_API")
        .unwrap_or_else(|_| "https://f.8b.is/feedback".to_string());

    let response = match client
        .post(&api_url)
        .header("X-MCP-Client", "smart-tree-mcp")
        .header("X-Tool-Request", "true")
        .json(&feedback)
        .timeout(std::time::Duration::from_secs(5))
        .send()
        .await
    {
        Ok(resp) => resp,
        Err(e) => {
            // API is down - save feedback locally
            use std::fs;
            use std::path::PathBuf;

            let feedback_dir = dirs::home_dir()
                .unwrap_or_else(|| PathBuf::from("."))
                .join(".mem8")
                .join("feedback")
                .join("pending");

            // Create directory if it doesn't exist
            fs::create_dir_all(&feedback_dir)?;

            // Create filename with timestamp
            let timestamp = Utc::now().format("%Y%m%d_%H%M%S_%f");
            let filename = format!(
                "tool_request_{}_{}.json",
                tool_name.replace("/", "_"),
                timestamp
            );
            let filepath = feedback_dir.join(filename);

            // Save feedback to file
            let feedback_with_meta = json!({
                "type": "tool_request",
                "timestamp": Utc::now().to_rfc3339(),
                "api_url": api_url,
                "error": format!("{}", e),
                "data": feedback
            });

            fs::write(
                &filepath,
                serde_json::to_string_pretty(&feedback_with_meta)?,
            )?;

            return Ok(json!({
                "content": [{
                    "type": "text",
                    "text": format!("📝 Tool request '{}' saved locally!\n\n\
                        The feedback API appears to be offline. Your request has been saved to:\n\
                        {}\n\n\
                        It will be automatically submitted when the connection is restored.\n\n\
                        🌳 Smart Tree continues to evolve with your help!",
                        tool_name,
                        filepath.display()
                    )
                }]
            }));
        }
    };

    if response.status().is_success() {
        let response_data: Value = response
            .json()
            .await
            .map_err(|e| anyhow::anyhow!("Failed to parse response: {}", e))?;

        Ok(json!({
            "content": [{
                "type": "text",
                "text": format!("🛠️ Tool request '{}' submitted successfully!\n\n\
                    Your request helps shape Smart Tree's evolution.\n\
                    {}\n\n\
                    Feedback ID: {}\n\n\
                    This request will be reviewed and potentially implemented to improve AI productivity!",
                    tool_name,
                    if anonymous { "Submitted anonymously." } else { "You'll receive credit if implemented!" },
                    response_data["feedback_id"].as_str().unwrap_or("unknown")
                )
            }]
        }))
    } else {
        let status = response.status();
        let error_text = response
            .text()
            .await
            .unwrap_or_else(|_| "Unknown error".to_string());
        Err(anyhow::anyhow!(
            "Failed to submit tool request: {} - {}",
            status,
            error_text
        ))
    }
}

/// Check if a newer version is available
pub async fn check_for_updates(args: Value, _ctx: Arc<McpContext>) -> Result<Value> {
    let _offer_auto_update = args["offer_auto_update"].as_bool().unwrap_or(true);
    let current_version = env!("CARGO_PKG_VERSION");

    // Check for updates using our client
    let client = FeedbackClient::new()?;
    let version_info = match client.check_for_updates().await {
        Ok(info) => info,
        Err(e) => {
            // If the API is down or unavailable, just return a soft error
            return Ok(json!({
                "content": [{
                    "type": "text",
                    "text": format!("Unable to check for updates at this time: {}\n\nYou can check manually at: https://github.com/8b-is/smart-tree/releases", e)
                }]
            }));
        }
    };

    // Compare versions
    let current = current_version.trim_start_matches('v');
    let latest = version_info.version.trim_start_matches('v');

    if current == latest {
        return Ok(json!({
            "content": [{
                "type": "text",
                "text": format!("✅ You're up to date! Running Smart Tree v{}\n\n🌳 Keep on rockin' with the latest and greatest!", current)
            }]
        }));
    }

    // Update is available
    let message = format!(
        "🚀 **New Smart Tree Version Available!**\n\n\
        Current: v{} → Latest: v{}\n\n\
        📥 Download: https://github.com/8b-is/smart-tree/releases/tag/v{}\n\n\
        To update:\n\
        ```bash\n\
        curl -sSL https://raw.githubusercontent.com/8b-is/smart-tree/main/scripts/install.sh | bash\n\
        ```",
        current,
        latest,
        latest
    );

    Ok(json!({
        "content": [{
            "type": "text",
            "text": message
        }],
        "metadata": {
            "update_available": true,
            "current_version": current_version,
            "latest_version": version_info.version.clone(),
            "download_url": format!("https://github.com/8b-is/smart-tree/releases/tag/v{}", latest)
        }
    }))
}
