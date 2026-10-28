// Smart Compression Manager - Token-aware compression for all outputs
// "Compression so smart, it knows when to squeeze!" - Aye

use anyhow::Result;
use once_cell::sync::Lazy;
use serde_json::Value;
use std::sync::RwLock;

/// Global compression state
static COMPRESSION_STATE: Lazy<RwLock<CompressionState>> =
    Lazy::new(|| RwLock::new(CompressionState::default()));

#[derive(Debug, Clone)]
pub struct CompressionState {
    /// Whether the client supports compressed content
    pub client_supports_compression: Option<bool>,

    /// Maximum tokens before auto-compression kicks in
    pub max_tokens: usize,

    /// Whether to always compress (override)
    pub force_compression: bool,

    /// Whether to never compress (override)
    pub disable_compression: bool,

    /// Statistics for debugging
    pub stats: CompressionStats,
}

#[derive(Debug, Clone, Default)]
pub struct CompressionStats {
    pub total_compressions: usize,
    pub bytes_saved: usize,
    pub tokens_saved: usize,
    pub failed_decompressions: usize,
}

impl Default for CompressionState {
    fn default() -> Self {
        Self {
            client_supports_compression: None, // Unknown until tested
            max_tokens: 20000,                 // Safe limit (MCP allows 25k)
            force_compression: false,
            disable_compression: false,
            stats: CompressionStats::default(),
        }
    }
}

/// Test if client supports compression by including a small compressed hint
pub fn create_compression_test() -> Value {
    // Create a small compressed message that won't break non-supporting clients
    let test_message = "COMPRESSION_SUPPORTED";
    let compressed = compress_string(test_message).unwrap_or_default();

    serde_json::json!({
        "_compression_test": compressed,
        "_compression_hint": "This server supports compressed responses. If you can read the _compression_test field after decompressing, reply with 'compression:ok' in your next request."
    })
}

/// Check if a client response indicates compression support
pub fn check_client_compression_support(request: &Value) -> bool {
    // Check for explicit compression acknowledgment
    if let Some(params) = request.get("params") {
        if let Some(compression) = params.get("compression") {
            if compression.as_str() == Some("ok") {
                set_compression_support(true);
                return true;
            }
        }

        // Check for compression capability in client info
        if let Some(capabilities) = params.get("capabilities") {
            if let Some(compression) = capabilities.get("compression") {
                let supported = compression.as_bool().unwrap_or(false);
                set_compression_support(supported);
                return supported;
            }
        }
    }

    false
}

/// Set global compression support status
pub fn set_compression_support(supported: bool) {
    if let Ok(mut state) = COMPRESSION_STATE.write() {
        state.client_supports_compression = Some(supported);
        eprintln!(
            "🗜️ Client compression support: {}",
            if supported { "YES" } else { "NO" }
        );
    }
}

/// Check if we should compress a response based on its size
pub fn should_compress_response(content: &str) -> bool {
    let state = COMPRESSION_STATE.read().unwrap();

    // Check overrides
    if state.disable_compression {
        return false;
    }
    if state.force_compression {
        return true;
    }

    // If we don't know if client supports compression, don't compress
    if state.client_supports_compression != Some(true) {
        return false;
    }

    // Estimate tokens (rough: 1 token ≈ 4 characters)
    let estimated_tokens = content.len() / 4;

    estimated_tokens > state.max_tokens
}

/// Compress a string using zlib
pub fn compress_string(content: &str) -> Result<String> {
    use flate2::write::ZlibEncoder;
    use flate2::Compression;
    use std::io::Write;

    let mut encoder = ZlibEncoder::new(Vec::new(), Compression::default());
    encoder.write_all(content.as_bytes())?;
    let compressed = encoder.finish()?;

    // Update stats
    if let Ok(mut state) = COMPRESSION_STATE.write() {
        state.stats.total_compressions += 1;
        state.stats.bytes_saved += content.len().saturating_sub(compressed.len());
        state.stats.tokens_saved += (content.len() / 4).saturating_sub(compressed.len() / 4);
    }

    Ok(format!("COMPRESSED_V1:{}", hex::encode(&compressed)))
}

/// Smart compress any MCP response content
pub fn smart_compress_mcp_response(response: &mut Value) -> Result<()> {
    // Look for content in the response
    if let Some(content) = response.get_mut("content") {
        if let Some(content_array) = content.as_array_mut() {
            for item in content_array {
                if let Some(text) = item.get_mut("text") {
                    if let Some(text_str) = text.as_str() {
                        // Check if we should compress
                        if should_compress_response(text_str) {
                            let compressed = compress_string(text_str)?;

                            // Calculate compression stats
                            let original_size = text_str.len();
                            let compressed_size = compressed.len();
                            let ratio =
                                100.0 - (compressed_size as f64 / original_size as f64 * 100.0);

                            eprintln!(
                                "🗜️ Auto-compressed response: {} → {} bytes ({:.1}% reduction)",
                                original_size, compressed_size, ratio
                            );
                            eprintln!(
                                "💡 Estimated tokens saved: {}",
                                (original_size / 4).saturating_sub(compressed_size / 4)
                            );

                            *text = Value::String(compressed);

                            // Add compression metadata
                            item["_compressed"] = serde_json::json!(true);
                            item["_original_size"] = serde_json::json!(original_size);
                            item["_compression_ratio"] = serde_json::json!(ratio);
                        }
                    }
                }
            }
        }
    }

    // Also check result field for tool responses
    if let Some(result) = response.get_mut("result") {
        if let Some(content) = result.get_mut("content") {
            if let Some(content_array) = content.as_array_mut() {
                for item in content_array {
                    if let Some(text) = item.get_mut("text") {
                        if let Some(text_str) = text.as_str() {
                            if should_compress_response(text_str) {
                                let compressed = compress_string(text_str)?;

                                let original_size = text_str.len();
                                let compressed_size = compressed.len();
                                let ratio =
                                    100.0 - (compressed_size as f64 / original_size as f64 * 100.0);

                                eprintln!(
                                    "🗜️ Auto-compressed result: {} → {} bytes ({:.1}% reduction)",
                                    original_size, compressed_size, ratio
                                );

                                *text = Value::String(compressed);

                                item["_compressed"] = serde_json::json!(true);
                                item["_original_size"] = serde_json::json!(original_size);
                            }
                        }
                    }
                }
            }
        }
    }

    Ok(())
}

/// Get compression statistics
pub fn get_compression_stats() -> CompressionStats {
    COMPRESSION_STATE.read().unwrap().stats.clone()
}

/// Configure compression settings
pub fn configure_compression(
    max_tokens: Option<usize>,
    force: Option<bool>,
    disable: Option<bool>,
) {
    if let Ok(mut state) = COMPRESSION_STATE.write() {
        if let Some(max) = max_tokens {
            state.max_tokens = max;
        }
        if let Some(f) = force {
            state.force_compression = f;
        }
        if let Some(d) = disable {
            state.disable_compression = d;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compression() {
        let content = "Hello World!".repeat(1000);
        let compressed = compress_string(&content).unwrap();
        assert!(compressed.starts_with("COMPRESSED_V1:"));
        assert!(compressed.len() < content.len());
    }

    #[test]
    fn test_should_compress() {
        set_compression_support(true);

        let small_content = "small";
        assert!(!should_compress_response(small_content));

        let large_content = "x".repeat(100000);
        assert!(should_compress_response(&large_content));
    }
}
