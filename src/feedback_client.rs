// -----------------------------------------------------------------------------
// 🌮 Feedback API Client - Helping Smart Tree Survive the Franchise Wars!
// -----------------------------------------------------------------------------
// This module handles communication with 8s.is for feedback submission and
// update checking. All feedback helps make Smart Tree better!
//
// Endpoints:
// - POST https://8s.is/api/feedback - Submit feedback and feature requests
// - GET  https://8s.is/api/smart-tree/latest - Get latest version info
// -----------------------------------------------------------------------------

use anyhow::Result;
use reqwest::{Client, StatusCode};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::time::Duration;

pub const DEFAULT_HUB_URL: &str = "https://8s.is";
const USER_AGENT: &str = concat!("smart-tree/", env!("CARGO_PKG_VERSION"));

pub fn feedback_endpoint() -> String {
    std::env::var("SMART_TREE_FEEDBACK_API").unwrap_or_else(|_| {
        format!(
            "{}/api/feedback",
            std::env::var("SMART_TREE_HUB_URL")
                .unwrap_or_else(|_| DEFAULT_HUB_URL.to_owned())
                .trim_end_matches('/')
        )
    })
}

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

/// API client for Smart Tree Hub.
pub struct FeedbackClient {
    client: Client,
    base_url: String,
}

impl FeedbackClient {
    pub fn new() -> Result<Self> {
        Self::with_base_url(
            &std::env::var("SMART_TREE_HUB_URL").unwrap_or_else(|_| DEFAULT_HUB_URL.to_owned()),
        )
    }

    pub fn with_base_url(base_url: &str) -> Result<Self> {
        let url = reqwest::Url::parse(base_url)?;
        anyhow::ensure!(
            matches!(url.scheme(), "http" | "https")
                && url.username().is_empty()
                && url.password().is_none()
                && url.query().is_none()
                && url.fragment().is_none(),
            "Invalid Smart Tree hub URL"
        );
        let client = Client::builder()
            .user_agent(USER_AGENT)
            .timeout(Duration::from_secs(30))
            .redirect(reqwest::redirect::Policy::none())
            .build()?;

        Ok(Self {
            client,
            base_url: base_url.trim_end_matches('/').to_owned(),
        })
    }

    /// Submit feedback to the hub.
    pub async fn submit_feedback(&self, feedback: FeedbackRequest) -> Result<FeedbackResponse> {
        let url = format!("{}/api/feedback", self.base_url);

        let response = self.client.post(&url).json(&feedback).send().await?;

        match response.status() {
            StatusCode::OK | StatusCode::CREATED | StatusCode::ACCEPTED => {
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

    /// Submit a tool request to the hub.
    pub async fn submit_tool_request(&self, request: ToolRequest) -> Result<FeedbackResponse> {
        let url = format!("{}/api/tool-request", self.base_url);

        let response = self.client.post(&url).json(&request).send().await?;

        match response.status() {
            StatusCode::OK | StatusCode::CREATED | StatusCode::ACCEPTED => {
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
        let url = format!("{}/api/smart-tree/latest", self.base_url);

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

    #[tokio::test]
    async fn custom_hub_accepts_created_tool_request() {
        use axum::{http::StatusCode, routing::post, Json, Router};
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let address = listener.local_addr().unwrap();
        let app = Router::new().route(
            "/api/tool-request",
            post(|Json(value): Json<Value>| async move {
                assert_eq!(value["tool_name"], "archive_recall");
                (
                    StatusCode::CREATED,
                    Json(serde_json::json!({
                        "feedback_id": "stored-request",
                        "message": "Saved",
                        "status": "received"
                    })),
                )
            }),
        );
        let server = tokio::spawn(async move { axum::serve(listener, app).await.unwrap() });
        let result = FeedbackClient::with_base_url(&format!("http://{address}/"))
            .unwrap()
            .submit_tool_request(ToolRequest {
                tool_name: "archive_recall".into(),
                description: "Find an archived document".into(),
                use_case: "Repository recall".into(),
                expected_output: "Source passages".into(),
                productivity_impact: "Less repeated searching".into(),
                proposed_parameters: None,
                smart_tree_version: env!("CARGO_PKG_VERSION").into(),
                anonymous: true,
                github_url: None,
            })
            .await;
        server.abort();
        assert_eq!(result.unwrap().feedback_id, "stored-request");
    }
}
