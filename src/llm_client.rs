//! LLM Client - OpenAI-compatible API client for Ollama, LM Studio, vLLM, etc.
//!
//! Saved from liquid-rust for use in smart-tree's subconscious layer.
//!
//! Usage:
//! ```rust
//! let messages = vec![json!({"role": "user", "content": "Hello"})];
//!
//! // Ollama
//! let response = call_ollama("http://localhost:11434", "llama3.2", &messages).await?;
//!
//! // OpenAI-compatible (LM Studio, vLLM, etc.)
//! let response = call_openai_compatible("http://localhost:8080/v1", None, &messages).await?;
//! ```

use anyhow::Result;
use tracing::info;

/// Call Ollama API (OpenAI-compatible endpoint)
/// Ollama serves at http://localhost:11434/v1/chat/completions
pub async fn call_ollama(
    base_url: &str,
    model: &str,
    messages: &[serde_json::Value],
) -> Result<String> {
    let client = reqwest::Client::new();

    // Ollama's OpenAI-compatible endpoint
    let url = format!("{}/v1/chat/completions", base_url.trim_end_matches('/'));

    info!("Calling Ollama at {} with model {}", url, model);

    let response = client
        .post(&url)
        .header("content-type", "application/json")
        .json(&serde_json::json!({
            "model": model,
            "max_tokens": 1024,
            "messages": messages,
            "stream": false
        }))
        .send()
        .await?;

    let status = response.status();
    let json: serde_json::Value = response.json().await?;

    if !status.is_success() {
        if let Some(error) = json["error"]["message"].as_str() {
            anyhow::bail!("Ollama error: {}", error);
        }
        anyhow::bail!("Ollama error: HTTP {}", status);
    }

    if let Some(choice) = json["choices"].get(0) {
        if let Some(text) = choice["message"]["content"].as_str() {
            return Ok(text.to_string());
        }
    }

    anyhow::bail!("Unexpected Ollama response format: {:?}", json)
}

/// Call any OpenAI-compatible API (LM Studio, vLLM, text-generation-webui, LocalAI, etc.)
pub async fn call_openai_compatible(
    base_url: &str,
    api_key: Option<&str>,
    messages: &[serde_json::Value],
) -> Result<String> {
    call_openai_compatible_with_model(base_url, api_key, "gpt-4o-mini", messages).await
}

/// Call OpenAI-compatible API with specific model
pub async fn call_openai_compatible_with_model(
    base_url: &str,
    api_key: Option<&str>,
    model: &str,
    messages: &[serde_json::Value],
) -> Result<String> {
    let client = reqwest::Client::new();

    let url = format!("{}/chat/completions", base_url.trim_end_matches('/'));

    info!("Calling OpenAI-compatible API at {}", url);

    let mut request = client
        .post(&url)
        .header("content-type", "application/json");

    // Add auth header if API key provided
    if let Some(key) = api_key {
        request = request.header("Authorization", format!("Bearer {}", key));
    }

    let response = request
        .json(&serde_json::json!({
            "model": model,
            "max_tokens": 1024,
            "messages": messages
        }))
        .send()
        .await?;

    let status = response.status();
    let json: serde_json::Value = response.json().await?;

    if !status.is_success() {
        if let Some(error) = json["error"]["message"].as_str() {
            anyhow::bail!("API error: {}", error);
        }
        anyhow::bail!("API error: HTTP {}", status);
    }

    if let Some(choice) = json["choices"].get(0) {
        if let Some(text) = choice["message"]["content"].as_str() {
            return Ok(text.to_string());
        }
    }

    anyhow::bail!("Unexpected API response format: {:?}", json)
}

/// Call OpenAI API directly
pub async fn call_openai(api_key: &str, messages: &[serde_json::Value]) -> Result<String> {
    call_openai_compatible("https://api.openai.com/v1", Some(api_key), messages).await
}

/// Call Claude API (Anthropic)
pub async fn call_claude(
    api_key: &str,
    messages: &[serde_json::Value],
    system: Option<&str>,
) -> Result<String> {
    let client = reqwest::Client::new();

    let mut body = serde_json::json!({
        "model": "claude-sonnet-4-20250514",
        "max_tokens": 1024,
        "messages": messages
    });

    if let Some(sys) = system {
        body["system"] = serde_json::json!(sys);
    }

    let response = client
        .post("https://api.anthropic.com/v1/messages")
        .header("x-api-key", api_key)
        .header("anthropic-version", "2023-06-01")
        .header("content-type", "application/json")
        .json(&body)
        .send()
        .await?;

    let status = response.status();
    let json: serde_json::Value = response.json().await?;

    if !status.is_success() {
        if let Some(error) = json["error"]["message"].as_str() {
            anyhow::bail!("Claude error: {}", error);
        }
        anyhow::bail!("Claude error: HTTP {}", status);
    }

    // Claude API returns content as array
    if let Some(content) = json["content"].as_array() {
        if let Some(first) = content.first() {
            if let Some(text) = first["text"].as_str() {
                return Ok(text.to_string());
            }
        }
    }

    anyhow::bail!("Unexpected Claude response format: {:?}", json)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    #[ignore] // Requires running Ollama
    async fn test_ollama_call() {
        let messages = vec![serde_json::json!({
            "role": "user",
            "content": "Say 'hello' and nothing else."
        })];

        let result = call_ollama("http://localhost:11434", "llama3.2", &messages).await;
        assert!(result.is_ok());
        println!("Ollama response: {}", result.unwrap());
    }
}
