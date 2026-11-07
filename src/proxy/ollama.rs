//! 🦙 Ollama & LM Studio Provider - Local LLM Auto-Detection
//!
//! Automatically detects and connects to local LLM servers:
//! - Ollama at localhost:11434
//! - LM Studio at localhost:1234
//!
//! Both use OpenAI-compatible APIs, so we handle them uniformly.
//!
//! "Why pay for clouds when you've got a llama at home?" - The Cheet 🦙

use anyhow::{Context, Result};
use async_trait::async_trait;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::time::Duration;

use super::{LlmProvider, LlmRequest, LlmResponse, LlmUsage};

/// Default ports for local LLM servers
pub const OLLAMA_PORT: u16 = 11434;
pub const OLLAMA_BRIDGE_PORT: u16 = 11436;
pub const LMSTUDIO_PORT: u16 = 1234;

/// Normalize an endpoint or `OLLAMA_HOST` string into an `http://host:port` base URL.
fn normalize_base_url(raw: &str) -> String {
    let trimmed = raw.trim().trim_end_matches('/');
    if trimmed.starts_with("http://") || trimmed.starts_with("https://") {
        trimmed.to_string()
    } else if trimmed.contains(':') {
        format!("http://{trimmed}")
    } else {
        format!("http://{trimmed}:{OLLAMA_PORT}")
    }
}

/// Ordered candidate Ollama URLs from `OLLAMA_HOST`, sovereign bridge `:11436`, local `:11434`, and `[memnet].endpoints`.
pub fn candidate_ollama_base_urls() -> Vec<String> {
    let mut urls = Vec::new();
    let mut push_unique = |u: String| {
        if !u.is_empty() && !urls.contains(&u) {
            urls.push(u);
        }
    };
    if let Ok(env_host) = std::env::var("OLLAMA_HOST") {
        if !env_host.trim().is_empty() {
            push_unique(normalize_base_url(&env_host));
        }
    }
    if let Ok(cfg) = crate::config::StConfig::load() {
        for ep in cfg.memnet.endpoints {
            // Skip Smart Tree daemon/liquid-rust endpoints when building Ollama URLs
            if !ep.ends_with(":28428") && !ep.ends_with(":8420") && !ep.ends_with(":8786") {
                push_unique(normalize_base_url(&ep));
            }
        }
    }
    push_unique(format!("http://127.0.0.1:{OLLAMA_BRIDGE_PORT}"));
    push_unique(format!("http://localhost:{OLLAMA_PORT}"));
    urls
}

/// Detected local LLM server type
#[derive(Debug, Clone, PartialEq)]
pub enum LocalLlmType {
    Ollama,
    LmStudio,
}

impl std::fmt::Display for LocalLlmType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LocalLlmType::Ollama => write!(f, "Ollama"),
            LocalLlmType::LmStudio => write!(f, "LM Studio"),
        }
    }
}

/// Information about a detected local LLM server
#[derive(Debug, Clone)]
pub struct LocalLlmInfo {
    pub server_type: LocalLlmType,
    pub base_url: String,
    pub models: Vec<String>,
}

/// 🦙 Provider for local LLM servers (Ollama, LM Studio)
pub struct OllamaProvider {
    client: Client,
    base_url: String,
    fallback_urls: Vec<String>,
    server_type: LocalLlmType,
    default_model: String,
}

impl OllamaProvider {
    /// Create a new Ollama provider with explicit URL
    pub fn new(base_url: &str, server_type: LocalLlmType) -> Self {
        let default_model = crate::config::StConfig::load()
            .ok()
            .map(|c| c.models.default_model)
            .filter(|s| !s.is_empty())
            .unwrap_or_else(|| "st-deepseek-pqm:latest".to_string());
        let primary = normalize_base_url(base_url);
        let fallback_urls = if server_type == LocalLlmType::Ollama {
            candidate_ollama_base_urls()
                .into_iter()
                .filter(|u| u != &primary)
                .collect()
        } else {
            Vec::new()
        };
        Self {
            client: Client::builder()
                .timeout(Duration::from_secs(300)) // Local models can be slow
                .build()
                .expect("Failed to create HTTP client"),
            base_url: primary,
            fallback_urls,
            server_type,
            default_model,
        }
    }

    /// Create provider for Ollama using `OLLAMA_HOST` / sovereign bridge / local port
    pub fn ollama() -> Self {
        let candidates = candidate_ollama_base_urls();
        let primary = candidates
            .first()
            .cloned()
            .unwrap_or_else(|| format!("http://localhost:{OLLAMA_PORT}"));
        Self::new(&primary, LocalLlmType::Ollama)
    }

    /// Create provider for LM Studio at default port
    pub fn lmstudio() -> Self {
        Self::new(
            &format!("http://localhost:{}", LMSTUDIO_PORT),
            LocalLlmType::LmStudio,
        )
    }

    /// Set the default model to use
    pub fn with_model(mut self, model: &str) -> Self {
        self.default_model = model.to_string();
        self
    }

    /// List available models from the server
    pub async fn list_models(&self) -> Result<Vec<String>> {
        match self.server_type {
            LocalLlmType::Ollama => self.list_ollama_models().await,
            LocalLlmType::LmStudio => self.list_lmstudio_models().await,
        }
    }

    async fn list_ollama_models(&self) -> Result<Vec<String>> {
        let mut all_models = Vec::new();
        let mut last_err = None;
        let mut urls = vec![self.base_url.clone()];
        urls.extend(self.fallback_urls.iter().cloned());

        for base in urls {
            let url = format!("{base}/api/tags");
            match self.client.get(&url).send().await {
                Ok(response) => {
                    if let Ok(tags) = response.json::<OllamaTagsResponse>().await {
                        for m in tags.models {
                            if !all_models.contains(&m.name) {
                                all_models.push(m.name);
                            }
                        }
                    }
                }
                Err(e) => {
                    last_err = Some(e);
                }
            }
        }

        if all_models.is_empty() {
            if let Some(err) = last_err {
                return Err(err).context("Failed to connect to Ollama");
            }
        }
        Ok(all_models)
    }

    async fn list_lmstudio_models(&self) -> Result<Vec<String>> {
        let url = format!("{}/v1/models", self.base_url);
        let response = self
            .client
            .get(&url)
            .send()
            .await
            .context("Failed to connect to LM Studio")?;

        let models: OpenAiModelsResponse = response
            .json()
            .await
            .context("Failed to parse LM Studio models response")?;

        Ok(models.data.into_iter().map(|m| m.id).collect())
    }
}

impl Default for OllamaProvider {
    fn default() -> Self {
        Self::ollama()
    }
}

#[async_trait]
impl LlmProvider for OllamaProvider {
    async fn complete(&self, request: LlmRequest) -> Result<LlmResponse> {
        let raw_model = if request.model.is_empty() || request.model == "default" {
            self.default_model.clone()
        } else {
            request.model.clone()
        };
        let model = crate::config::StConfig::load()
            .ok()
            .map(|c| c.resolve_model(&raw_model))
            .unwrap_or(raw_model);

        let openai_request = OpenAiChatRequest {
            model: model.clone(),
            messages: request
                .messages
                .iter()
                .map(|m| OpenAiMessage {
                    role: match m.role {
                        super::LlmRole::System => "system".to_string(),
                        super::LlmRole::User => "user".to_string(),
                        super::LlmRole::Assistant => "assistant".to_string(),
                    },
                    content: m.content.clone(),
                })
                .collect(),
            temperature: request.temperature,
            max_tokens: request.max_tokens,
            stream: false, // We don't handle streaming in this basic impl
        };

        let mut candidate_bases = vec![self.base_url.clone()];
        candidate_bases.extend(self.fallback_urls.iter().cloned());

        let mut last_err = None;
        for base in candidate_bases {
            let url = format!("{base}/v1/chat/completions");
            let response = match self.client.post(&url).json(&openai_request).send().await {
                Ok(r) => r,
                Err(e) => {
                    last_err = Some(anyhow::anyhow!(
                        "Failed to send request to {} ({}): {}",
                        self.server_type,
                        base,
                        e
                    ));
                    continue;
                }
            };

            if !response.status().is_success() {
                let status = response.status();
                let error_text = response.text().await.unwrap_or_default();
                last_err = Some(anyhow::anyhow!(
                    "{} ({}) returned error {}: {}",
                    self.server_type,
                    base,
                    status,
                    error_text
                ));
                continue;
            }

            let openai_response: OpenAiChatResponse = response
                .json()
                .await
                .context("Failed to parse response from local LLM")?;

            let content = openai_response
                .choices
                .first()
                .map(|c| c.message.content.clone())
                .unwrap_or_default();

            return Ok(LlmResponse {
                content,
                model: openai_response.model,
                usage: openai_response.usage.map(|u| LlmUsage {
                    prompt_tokens: u.prompt_tokens,
                    completion_tokens: u.completion_tokens,
                    total_tokens: u.total_tokens,
                }),
            });
        }

        Err(last_err.unwrap_or_else(|| {
            anyhow::anyhow!("No reachable {} endpoint succeeded", self.server_type)
        }))
    }

    fn name(&self) -> &'static str {
        match self.server_type {
            LocalLlmType::Ollama => "ollama",
            LocalLlmType::LmStudio => "lmstudio",
        }
    }
}

// ============================================================================
// Auto-Detection
// ============================================================================

/// Check if a local LLM server is running at the given port
pub async fn check_server(host: &str, port: u16, timeout_ms: u64) -> bool {
    let client = match Client::builder()
        .timeout(Duration::from_millis(timeout_ms))
        .build()
    {
        Ok(c) => c,
        Err(_) => return false,
    };

    // Try the health/version endpoint first (fast)
    let health_url = format!("http://{}:{}/", host, port);
    if client.get(&health_url).send().await.is_ok() {
        return true;
    }

    // Fallback: try the models endpoint
    let models_url = format!("http://{}:{}/v1/models", host, port);
    client.get(&models_url).send().await.is_ok()
}

/// Detect all available local LLM servers
pub async fn detect_local_llms() -> Vec<LocalLlmInfo> {
    let mut detected = Vec::new();

    // Check Ollama (across OLLAMA_HOST, bridge :11436, local :11434, and configured endpoints)
    if check_server("127.0.0.1", OLLAMA_BRIDGE_PORT, 500).await
        || check_server("localhost", OLLAMA_PORT, 500).await
    {
        let provider = OllamaProvider::ollama();
        let models = provider.list_models().await.unwrap_or_default();
        detected.push(LocalLlmInfo {
            server_type: LocalLlmType::Ollama,
            base_url: provider.base_url.clone(),
            models,
        });
    }

    // Check LM Studio
    if check_server("localhost", LMSTUDIO_PORT, 500).await {
        let provider = OllamaProvider::lmstudio();
        let models = provider.list_models().await.unwrap_or_default();
        detected.push(LocalLlmInfo {
            server_type: LocalLlmType::LmStudio,
            base_url: format!("http://localhost:{}", LMSTUDIO_PORT),
            models,
        });
    }

    detected
}

/// Quick check if any local LLM is available (non-blocking, fast timeout)
pub async fn any_local_llm_available() -> bool {
    tokio::select! {
        bridge = check_server("127.0.0.1", OLLAMA_BRIDGE_PORT, 200) => {
            if bridge { return true; }
        }
        ollama = check_server("localhost", OLLAMA_PORT, 200) => {
            if ollama { return true; }
        }
        lmstudio = check_server("localhost", LMSTUDIO_PORT, 200) => {
            if lmstudio { return true; }
        }
    }

    // Check remaining
    check_server("127.0.0.1", OLLAMA_BRIDGE_PORT, 200).await
        || check_server("localhost", OLLAMA_PORT, 200).await
        || check_server("localhost", LMSTUDIO_PORT, 200).await
}

// ============================================================================
// API Types
// ============================================================================

#[derive(Debug, Deserialize)]
struct OllamaTagsResponse {
    models: Vec<OllamaModel>,
}

#[derive(Debug, Deserialize)]
struct OllamaModel {
    name: String,
    #[allow(dead_code)]
    modified_at: Option<String>,
    #[allow(dead_code)]
    size: Option<u64>,
}

#[derive(Debug, Deserialize)]
struct OpenAiModelsResponse {
    data: Vec<OpenAiModelInfo>,
}

#[derive(Debug, Deserialize)]
struct OpenAiModelInfo {
    id: String,
}

#[derive(Debug, Serialize)]
struct OpenAiChatRequest {
    model: String,
    messages: Vec<OpenAiMessage>,
    #[serde(skip_serializing_if = "Option::is_none")]
    temperature: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    max_tokens: Option<usize>,
    stream: bool,
}

#[derive(Debug, Serialize, Deserialize)]
struct OpenAiMessage {
    role: String,
    content: String,
}

#[derive(Debug, Deserialize)]
struct OpenAiChatResponse {
    model: String,
    choices: Vec<OpenAiChoice>,
    usage: Option<OpenAiUsageInfo>,
}

#[derive(Debug, Deserialize)]
struct OpenAiChoice {
    message: OpenAiMessage,
}

#[derive(Debug, Deserialize)]
struct OpenAiUsageInfo {
    prompt_tokens: usize,
    completion_tokens: usize,
    total_tokens: usize,
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_detect_local_llms() {
        // This test will pass whether or not local LLMs are running
        let detected = detect_local_llms().await;
        println!("Detected {} local LLM server(s)", detected.len());
        for info in &detected {
            println!(
                "  - {} at {} with {} models",
                info.server_type,
                info.base_url,
                info.models.len()
            );
            for model in &info.models {
                println!("      • {}", model);
            }
        }
    }

    #[tokio::test]
    async fn test_check_server_timeout() {
        // Should timeout quickly on non-existent server
        let start = std::time::Instant::now();
        let result = check_server("localhost", 59999, 100).await;
        let elapsed = start.elapsed();

        assert!(!result);
        assert!(
            elapsed.as_millis() < 500,
            "Timeout took too long: {:?}",
            elapsed
        );
    }
}
