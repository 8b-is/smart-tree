// -----------------------------------------------------------------------------
// 🌮 Feedback API Client - Helping Smart Tree Survive the Franchise Wars!
// -----------------------------------------------------------------------------
// This module handles communication with f.8t.is for feedback submission and
// update checking. All feedback helps make Smart Tree better!
//
// Endpoints:
// - POST https://f.8t.is/api/feedback - Submit feedback and feature requests
// - GET  https://f.8t.is/api/smart-tree/latest - Get latest version info (cached)
// -----------------------------------------------------------------------------

use anyhow::Result;
use reqwest::{Client, StatusCode};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::time::Duration;

const FEEDBACK_API_BASE: &str = "https://f.8t.is";
const USER_AGENT: &str = concat!("smart-tree/", env!("CARGO_PKG_VERSION"));

/// Feedback submission request structure
#[derive(Debug, Serialize)]
pub struct FeedbackRequest {
    pub category: String,
    pub title: String,
    pub description: String,
    pub impact_score: u8,
    pub frequency_score: u8,
    pub affected_command: Option<String>,
    pub mcp_tool: Option<String>,
    pub proposed_fix: Option<String>,
    pub proposed_solution: Option<String>,
    pub fix_complexity: Option<String>,
    pub auto_fixable: Option<bool>,
    pub tags: Vec<String>,
    pub examples: Vec<FeedbackExample>,
    pub smart_tree_version: String,
    pub anonymous: bool,
    pub github_url: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct FeedbackExample {
    pub description: String,
    pub code: String,
    pub expected_output: Option<String>,
}

/// Tool request structure
#[derive(Debug, Serialize)]
pub struct ToolRequest {
    pub tool_name: String,
    pub description: String,
    pub use_case: String,
    pub expected_output: String,
    pub productivity_impact: String,
    pub proposed_parameters: Option<Value>,
    pub smart_tree_version: String,
    pub anonymous: bool,
    pub github_url: Option<String>,
}

/// Response from feedback API
#[derive(Debug, Deserialize)]
pub struct FeedbackResponse {
    pub feedback_id: String,
    pub message: String,
    pub status: String,
}

/// Latest version info
#[derive(Debug, Deserialize)]
pub struct VersionInfo {
    pub version: String,
    pub release_date: String,
    pub download_url: String,
    pub release_notes_url: String,
    pub features: Vec<String>,
    pub ai_benefits: Vec<String>,
}

/// API client for f.8t.is
pub struct FeedbackClient {
    client: Client,
}

impl FeedbackClient {
    pub fn new() -> Result<Self> {
        let client = Client::builder()
            .user_agent(USER_AGENT)
            .timeout(Duration::from_secs(30))
            .build()?;

        Ok(Self { client })
    }

    /// Submit feedback to f.8t.is
    pub async fn submit_feedback(&self, feedback: FeedbackRequest) -> Result<FeedbackResponse> {
        let url = format!("{}/api/feedback", FEEDBACK_API_BASE);

        let response = self.client.post(&url).json(&feedback).send().await?;

        match response.status() {
            StatusCode::OK => {
                let data = response.json::<FeedbackResponse>().await?;
                Ok(data)
            }
            StatusCode::TOO_MANY_REQUESTS => Err(anyhow::anyhow!(
                "Rate limit exceeded. Please try again later."
            )),
            status => {
                let error_text = response
                    .text()
                    .await
                    .unwrap_or_else(|_| "Unknown error".to_string());
                Err(anyhow::anyhow!("API error ({}): {}", status, error_text))
            }
        }
    }

    /// Submit tool request to f.8t.is
    pub async fn submit_tool_request(&self, request: ToolRequest) -> Result<FeedbackResponse> {
        let url = format!("{}/api/tool-request", FEEDBACK_API_BASE);

        let response = self.client.post(&url).json(&request).send().await?;

        match response.status() {
            StatusCode::OK => {
                let data = response.json::<FeedbackResponse>().await?;
                Ok(data)
            }
            StatusCode::TOO_MANY_REQUESTS => Err(anyhow::anyhow!(
                "Rate limit exceeded. Please try again later."
            )),
            status => {
                let error_text = response
                    .text()
                    .await
                    .unwrap_or_else(|_| "Unknown error".to_string());
                Err(anyhow::anyhow!("API error ({}): {}", status, error_text))
            }
        }
    }

    /// Check for latest version (cached on server for 1 hour)
    pub async fn check_for_updates(&self) -> Result<VersionInfo> {
        let url = format!("{}/api/smart-tree/latest", FEEDBACK_API_BASE);

        let response = self.client.get(&url).send().await?;

        match response.status() {
            StatusCode::OK => {
                let data = response.json::<VersionInfo>().await?;
                Ok(data)
            }
            status => {
                let error_text = response
                    .text()
                    .await
                    .unwrap_or_else(|_| "Unknown error".to_string());
                Err(anyhow::anyhow!("API error ({}): {}", status, error_text))
            }
        }
    }
}

impl Default for FeedbackClient {
    fn default() -> Self {
        Self::new().expect("Failed to create feedback client")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_feedback_client_creation() {
        let client = FeedbackClient::new();
        assert!(client.is_ok());
    }
}
