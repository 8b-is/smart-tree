//! 🪨 Caveman Proxy Provider — Standalone byte-safe gateway integration (127.0.0.1:8787)
//!
//! Provides automatic base-URL-swap integration with JuliusBrussee/caveman gateway:
//! - Upstream transparent LLM routing through http://127.0.0.1:8787
//! - Zero cloud dependencies, truthful per-request spend recording
//! - Inferred token savings and compression tracking

use crate::proxy::{LlmMessage, LlmProvider, LlmRequest, LlmResponse, LlmUsage};
use anyhow::{bail, Context, Result};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};

pub const DEFAULT_CAVEMAN_URL: &str = "http://127.0.0.1:8787";

#[derive(Debug, Clone)]
pub struct CavemanProvider {
    pub base_url: String,
    pub api_key: Option<String>,
    client: reqwest::Client,
}

impl CavemanProvider {
    pub fn new() -> Self {
        let base_url =
            std::env::var("CAVEMAN_BASE_URL").unwrap_or_else(|_| DEFAULT_CAVEMAN_URL.to_string());
        let api_key = std::env::var("CAVEMAN_AUTH_TOKEN")
            .or_else(|_| std::env::var("ANTHROPIC_API_KEY"))
            .or_else(|_| std::env::var("OPENAI_API_KEY"))
            .ok();

        Self {
            base_url,
            api_key,
            client: reqwest::Client::new(),
        }
    }

    pub fn with_endpoint(endpoint: impl Into<String>) -> Self {
        let mut provider = Self::new();
        provider.base_url = endpoint.into();
        provider
    }

    /// Query the Caveman stats endpoint (~/.caveman/caveman.db spend summary)
    pub async fn stats(&self) -> Result<CavemanStats> {
        let url = format!("{}/stats", self.base_url);
        let resp = self
            .client
            .get(&url)
            .send()
            .await
            .context("Failed to query caveman stats")?;

        if !resp.status().is_success() {
            bail!("Caveman stats failed with HTTP {}", resp.status());
        }

        let stats = resp.json::<CavemanStats>().await?;
        Ok(stats)
    }

    /// Check if the local Caveman proxy is running and responding
    pub async fn is_alive(&self) -> bool {
        self.client
            .get(&self.base_url)
            .send()
            .await
            .map(|r| r.status().as_u16() < 500)
            .unwrap_or(false)
    }
}

impl Default for CavemanProvider {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CavemanStats {
    pub requests: u64,
    pub total_cost_usd: f64,
    pub savings_usd: f64,
    pub compression_tokens_before: u64,
    pub compression_tokens_after: u64,
    pub compression_tokens_saved: u64,
    pub basis: String,
}

impl CavemanStats {
    /// Exact reduced rational `(tokens_after, tokens_before)` compression ratio via `felix_binary_gcd_u64` (zero floats).
    #[inline]
    pub fn compression_ratio_pq(&self) -> (u64, u64) {
        if self.compression_tokens_before == 0 {
            return (1, 1);
        }
        let g = crate::felix_asm::felix_binary_gcd_u64(
            self.compression_tokens_after,
            self.compression_tokens_before,
        )
        .max(1);
        (
            self.compression_tokens_after / g,
            self.compression_tokens_before / g,
        )
    }
}

#[derive(Serialize)]
struct CavemanChatRequest<'a> {
    model: &'a str,
    messages: &'a [LlmMessage],
    #[serde(skip_serializing_if = "Option::is_none")]
    temperature: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    max_tokens: Option<usize>,
    stream: bool,
}

#[derive(Deserialize)]
struct CavemanChatResponse {
    choices: Vec<CavemanChoice>,
    usage: Option<CavemanUsagePayload>,
}

#[derive(Deserialize)]
struct CavemanChoice {
    message: CavemanMessagePayload,
}

#[derive(Deserialize)]
struct CavemanMessagePayload {
    content: String,
}

#[derive(Deserialize)]
struct CavemanUsagePayload {
    prompt_tokens: usize,
    completion_tokens: usize,
    total_tokens: usize,
}

#[async_trait]
impl LlmProvider for CavemanProvider {
    async fn complete(&self, request: LlmRequest) -> Result<LlmResponse> {
        let url = format!(
            "{}/v1/chat/completions",
            self.base_url.trim_end_matches('/')
        );

        let mut req_builder = self.client.post(&url).json(&CavemanChatRequest {
            model: &request.model,
            messages: &request.messages,
            temperature: request.temperature,
            max_tokens: request.max_tokens,
            stream: false,
        });

        if let Some(ref key) = self.api_key {
            req_builder = req_builder
                .header("Authorization", format!("Bearer {}", key))
                .header("x-cave-api-key", key);
        }

        let resp = req_builder
            .send()
            .await
            .context("Failed to send request through Caveman gateway")?;

        if !resp.status().is_success() {
            let status = resp.status();
            let body = resp.text().await.unwrap_or_default();
            bail!("Caveman gateway error (HTTP {}): {}", status, body);
        }

        let payload: CavemanChatResponse = resp.json().await?;
        let content = payload
            .choices
            .into_iter()
            .next()
            .map(|c| c.message.content)
            .unwrap_or_default();

        let usage = payload.usage.map(|u| LlmUsage {
            prompt_tokens: u.prompt_tokens,
            completion_tokens: u.completion_tokens,
            total_tokens: u.total_tokens,
        });

        Ok(LlmResponse {
            content,
            model: request.model,
            usage,
        })
    }

    fn name(&self) -> &'static str {
        "caveman"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_caveman_provider_and_pq_compression_ratio() {
        let p = CavemanProvider::with_endpoint("http://127.0.0.1:8787");
        assert_eq!(p.name(), "caveman");
        assert_eq!(p.base_url, "http://127.0.0.1:8787");

        let stats = CavemanStats {
            requests: 42,
            total_cost_usd: 0.12,
            savings_usd: 0.36,
            compression_tokens_before: 1000,
            compression_tokens_after: 250,
            compression_tokens_saved: 750,
            basis: "spqt-base243".to_string(),
        };
        assert_eq!(stats.compression_ratio_pq(), (1, 4));
    }
}
