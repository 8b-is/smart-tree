//! 🪨 Caveman / Bonobo (`:8787`) + Kompress-Super-Ultra (`:8799`) Unified Async Proxy Provider
//!
//! Implements `/rust-async-patterns` for the Sovereign Local Cluster Gateway:
//! - Concurrent order-preserving message pre-folding via `futures::stream::StreamExt::buffered`
//!   and `tokio::time::timeout` against `kompress-super-ultra` (`http://127.0.0.1:8799/api/kompress`)
//! - Parallel dual-daemon health & telemetry probing via `tokio::join!` (`:8787` + `:8799`)
//! - Exact reduced rational `p/q` (`felix_binary_gcd_u64`) and integer basis-point (`bps`) accounting

use crate::proxy::{LlmMessage, LlmProvider, LlmRequest, LlmResponse, LlmUsage};
use anyhow::{bail, Context, Result};
use async_trait::async_trait;
use futures::stream::{self, StreamExt};
use serde::{Deserialize, Serialize};
use std::time::Duration;

/// Default local `caveman` / `bonobo` gateway URL (`127.0.0.1:8787`).
pub const DEFAULT_CAVEMAN_URL: &str = "http://127.0.0.1:8787";

/// Default local `kompress-super-ultra` CORE URL (`127.0.0.1:8799`).
pub const DEFAULT_KOMPRESS_URL: &str = "http://127.0.0.1:8799";

#[derive(Debug, Clone)]
pub struct CavemanProvider {
    pub base_url: String,
    pub kompress_url: String,
    pub api_key: Option<String>,
    pub prefold_enabled: bool,
    client: reqwest::Client,
}

impl CavemanProvider {
    pub fn new() -> Self {
        let base_url =
            std::env::var("CAVEMAN_BASE_URL").unwrap_or_else(|_| DEFAULT_CAVEMAN_URL.to_string());
        let kompress_url =
            std::env::var("KOMPRESS_BASE_URL").unwrap_or_else(|_| DEFAULT_KOMPRESS_URL.to_string());
        let api_key = std::env::var("CAVEMAN_AUTH_TOKEN")
            .or_else(|_| std::env::var("ANTHROPIC_API_KEY"))
            .or_else(|_| std::env::var("OPENAI_API_KEY"))
            .ok();

        Self {
            base_url,
            kompress_url,
            api_key,
            prefold_enabled: true,
            client: reqwest::Client::new(),
        }
    }

    pub fn with_endpoint(endpoint: impl Into<String>) -> Self {
        let mut provider = Self::new();
        provider.base_url = endpoint.into();
        provider
    }

    pub fn with_endpoints(
        caveman_endpoint: impl Into<String>,
        kompress_endpoint: impl Into<String>,
    ) -> Self {
        let mut provider = Self::new();
        provider.base_url = caveman_endpoint.into();
        provider.kompress_url = kompress_endpoint.into();
        provider
    }

    /// Query the Caveman stats endpoint (`~/.caveman/caveman.db` spend summary).
    pub async fn stats(&self) -> Result<CavemanStats> {
        let url = format!("{}/stats", self.base_url.trim_end_matches('/'));
        let resp = tokio::time::timeout(Duration::from_millis(500), self.client.get(&url).send())
            .await
            .context("Caveman stats request timed out")?
            .context("Failed to query caveman stats")?;

        if !resp.status().is_success() {
            bail!("Caveman stats failed with HTTP {}", resp.status());
        }

        let stats = resp.json::<CavemanStats>().await?;
        Ok(stats)
    }

    /// Query the `kompress-super-ultra` stats endpoint (`GET /api/stats` on `:8799`).
    pub async fn kompress_stats(&self) -> Result<KompressRemoteStats> {
        let url = format!("{}/api/stats", self.kompress_url.trim_end_matches('/'));
        let resp = tokio::time::timeout(Duration::from_millis(500), self.client.get(&url).send())
            .await
            .context("Kompress-Super-Ultra stats request timed out")?
            .context("Failed to query Kompress-Super-Ultra stats")?;

        if !resp.status().is_success() {
            bail!(
                "Kompress-Super-Ultra stats failed with HTTP {}",
                resp.status()
            );
        }

        let stats = resp.json::<KompressRemoteStats>().await?;
        Ok(stats)
    }

    /// Check if the local Caveman / Bonobo proxy (`:8787`) is running and responding.
    pub async fn is_alive(&self) -> bool {
        tokio::time::timeout(
            Duration::from_millis(250),
            self.client.get(&self.base_url).send(),
        )
        .await
        .ok()
        .and_then(|r| r.ok())
        .map(|r| r.status().as_u16() < 500)
        .unwrap_or(false)
    }

    /// Check if the local `kompress-super-ultra` daemon (`:8799`) is running and healthy.
    pub async fn is_kompress_alive(&self) -> bool {
        let url = format!("{}/health", self.kompress_url.trim_end_matches('/'));
        tokio::time::timeout(Duration::from_millis(250), self.client.get(&url).send())
            .await
            .ok()
            .and_then(|r| r.ok())
            .map(|r| r.status().is_success())
            .unwrap_or(false)
    }

    /// Probe both `:8787` (`caveman`/`bonobo`) and `:8799` (`kompress-super-ultra`) concurrently
    /// in a single RTT using `tokio::join!`.
    pub async fn unified_health(&self) -> (bool, bool) {
        tokio::join!(self.is_alive(), self.is_kompress_alive())
    }

    /// Fold a single text payload via `POST /api/kompress` with a strict timeout, returning
    /// `None` if the service is offline or times out.
    pub async fn fold_text(&self, text: &str, mode: &str) -> Option<KompressFoldReply> {
        if text.trim().is_empty() {
            return None;
        }
        let url = format!("{}/api/kompress", self.kompress_url.trim_end_matches('/'));
        let req_fut = self
            .client
            .post(&url)
            .json(&serde_json::json!({ "text": text, "mode": mode }))
            .send();

        let resp = tokio::time::timeout(Duration::from_millis(200), req_fut)
            .await
            .ok()?
            .ok()?;
        if !resp.status().is_success() {
            return None;
        }
        resp.json::<KompressFoldReply>().await.ok()
    }

    /// Fold a batch of `LlmMessage`s concurrently (preserving message order via `StreamExt::buffered`)
    /// through `kompress-super-ultra` (`:8799`), falling back to the raw message on timeout/error.
    pub async fn fold_messages_concurrent(&self, messages: Vec<LlmMessage>) -> Vec<LlmMessage> {
        if !self.prefold_enabled || messages.is_empty() {
            return messages;
        }

        stream::iter(messages)
            .map(|mut msg| async move {
                if let Some(reply) = self.fold_text(&msg.content, "standard").await {
                    if !reply.compressed.is_empty() && reply.saved_tokens > 0 {
                        msg.content = reply.compressed;
                    }
                }
                msg
            })
            .buffered(8)
            .collect()
            .await
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

/// Response from `POST /api/kompress` on `kompress-super-ultra` (`:8799`).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct KompressFoldReply {
    pub compressed: String,
    pub original_tokens: u64,
    pub compressed_tokens: u64,
    pub saved_tokens: u64,
    pub compression_ratio_pq: String,
    #[serde(default)]
    pub savings_ratio_pq: String,
    #[serde(default)]
    pub compression_savings_bps: u64,
    #[serde(default)]
    pub bonobo_resonance_trit: i8,
    pub latency_us: u64,
}

/// Cumulative stats from `GET /api/stats` on `kompress-super-ultra` (`:8799`).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct KompressRemoteStats {
    pub total_requests: u64,
    pub input_tokens: u64,
    pub output_tokens: u64,
    pub saved_tokens: u64,
    pub savings_ratio_pq: String,
    #[serde(default)]
    pub compression_ratio_pq: String,
    #[serde(default)]
    pub savings_bps: u64,
    pub bonobo_harmonic_pq: String,
    #[serde(default)]
    pub techno_zen_bonobo_pq: String,
    #[serde(default)]
    pub target_resonance_trit: i8,
    pub latency_us_avg: u64,
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
        let folded_messages = self.fold_messages_concurrent(request.messages).await;
        let url = format!(
            "{}/v1/chat/completions",
            self.base_url.trim_end_matches('/')
        );

        let mut req_builder = self.client.post(&url).json(&CavemanChatRequest {
            model: &request.model,
            messages: &folded_messages,
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
    use crate::proxy::LlmRole;
    use axum::{
        routing::{get, post},
        Json, Router,
    };

    #[test]
    fn test_caveman_provider_and_pq_compression_ratio() {
        let p = CavemanProvider::with_endpoint("http://127.0.0.1:8787");
        assert_eq!(p.name(), "caveman");
        assert_eq!(p.base_url, "http://127.0.0.1:8787");
        assert_eq!(p.kompress_url, "http://127.0.0.1:8799");

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

    #[tokio::test]
    async fn test_async_concurrent_message_prefolding_and_unified_health() {
        let app = Router::new()
            .route(
                "/health",
                get(|| async { Json(serde_json::json!({ "status": "ok" })) }),
            )
            .route(
                "/api/kompress",
                post(|Json(body): Json<serde_json::Value>| async move {
                    let raw = body
                        .get("text")
                        .and_then(|v| v.as_str())
                        .unwrap_or_default();
                    Json(KompressFoldReply {
                        compressed: format!("[folded]{raw}"),
                        original_tokens: 30,
                        compressed_tokens: 15,
                        saved_tokens: 15,
                        compression_ratio_pq: "1/2".to_string(),
                        savings_ratio_pq: "1/2".to_string(),
                        compression_savings_bps: 5000,
                        bonobo_resonance_trit: 1,
                        latency_us: 12,
                    })
                }),
            );

        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();
        tokio::spawn(async move {
            let _ = axum::serve(listener, app).await;
        });

        let provider =
            CavemanProvider::with_endpoints(format!("http://{addr}"), format!("http://{addr}"));

        let (cave_ok, kompress_ok) = provider.unified_health().await;
        assert!(cave_ok);
        assert!(kompress_ok);

        let msgs = vec![
            LlmMessage {
                role: LlmRole::System,
                content: "sys-prompt".to_string(),
            },
            LlmMessage {
                role: LlmRole::User,
                content: "user-msg-1".to_string(),
            },
            LlmMessage {
                role: LlmRole::User,
                content: "user-msg-2".to_string(),
            },
        ];

        let folded = provider.fold_messages_concurrent(msgs).await;
        assert_eq!(folded.len(), 3);
        assert_eq!(folded[0].content, "[folded]sys-prompt");
        assert_eq!(folded[1].content, "[folded]user-msg-1");
        assert_eq!(folded[2].content, "[folded]user-msg-2");
    }
}
