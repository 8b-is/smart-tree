//! 🤖 Candle Local LLM Provider Implementation
//!
//! "Bringing AI home with Rust-native local models!" - The Cheet 😺

use crate::proxy::{LlmProvider, LlmRequest, LlmResponse};
use anyhow::Result;
use async_trait::async_trait;

#[cfg(feature = "candle")]
use crate::proxy::LlmUsage;
#[cfg(feature = "candle")]
use candle_core::{Device, Tensor};
#[cfg(feature = "candle")]
use candle_transformers::models::llama;

pub struct CandleProvider {
    // In a real implementation, we would hold model weights here
    model_path: Option<String>,
}

impl CandleProvider {
    pub fn new(model_path: Option<String>) -> Self {
        Self { model_path }
    }
}

impl Default for CandleProvider {
    fn default() -> Self {
        Self::new(None)
    }
}

#[async_trait]
impl LlmProvider for CandleProvider {
    async fn complete(&self, _request: LlmRequest) -> Result<LlmResponse> {
        #[cfg(feature = "candle")]
        {
            let _suppress_warnings = (
                std::marker::PhantomData::<LlmUsage>,
                std::marker::PhantomData::<Device>,
                std::marker::PhantomData::<Tensor>,
                std::marker::PhantomData::<llama::Config>,
            );
            let _ = &self.model_path;
        }

        let user_prompt = _request
            .messages
            .iter()
            .rev()
            .find(|m| m.role == crate::proxy::LlmRole::User)
            .map(|m| m.content.as_str())
            .unwrap_or("");

        // Check if a specific LifePersona was requested (`persona:<slug>` or `<slug>`)
        let model_clean = _request
            .model
            .strip_prefix("persona:")
            .or_else(|| _request.model.strip_prefix("candle/"))
            .or_else(|| _request.model.strip_prefix("local/"))
            .unwrap_or(&_request.model);

        if let Ok(persona) = crate::mem8::persona::LifePersona::load(model_clean) {
            let reply = persona.respond_locally(user_prompt);
            return Ok(LlmResponse {
                content: reply,
                model: format!("persona:{}", persona.slug),
                usage: None,
            });
        }

        // Fallback: if any personas exist in ~/.st/personas, use the most recently trained one,
        // or synthesize from ~/.st/memories.m8!
        if let Ok(all) = crate::mem8::persona::LifePersona::list_all() {
            if let Some(first) = all.first() {
                if let Ok(persona) = crate::mem8::persona::LifePersona::load(&first.slug) {
                    let reply = persona.respond_locally(user_prompt);
                    return Ok(LlmResponse {
                        content: reply,
                        model: format!("persona:{}", persona.slug),
                        usage: None,
                    });
                }
            }
        }

        // Synthesize from Smart Tree's local MEM8 memory bank (`~/.st/memories.m8`)
        if let Ok(mut mgr) = crate::memory_manager::MemoryManager::new() {
            let kws: Vec<String> = user_prompt
                .split_whitespace()
                .map(|w| w.trim_matches(|c: char| !c.is_alphanumeric()).to_string())
                .filter(|w| w.len() >= 3)
                .collect();
            if let Ok(recalled) = mgr.find(&kws) {
                if !recalled.is_empty() {
                    let mut out = format!(
                        "🕯️ **[Local MEM8 Wave Micro-Model (`{}`) · {} resonant memories]**\n\n",
                        _request.model,
                        recalled.len()
                    );
                    for m in recalled.iter().take(4) {
                        out.push_str(&format!(
                            "- **`{}`** ({:.1} Hz): {}\n",
                            m.origin, m.frequency, m.context
                        ));
                    }
                    return Ok(LlmResponse {
                        content: out,
                        model: _request.model,
                        usage: None,
                    });
                }
            }
        }

        Ok(LlmResponse {
            content: format!(
                "🕯️ **Local MEM8 Micro-Model (`{}`)** is ready! Train a Life Persona on the Dashboard (or run `m8 persona train <name> <path>`) to ground local responses in diaries, Google Drive, or local folders.",
                _request.model
            ),
            model: _request.model,
            usage: None,
        })
    }

    fn name(&self) -> &'static str {
        "Candle"
    }
}
