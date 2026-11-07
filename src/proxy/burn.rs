//! 🤖 Burn & SPQT Base-243 Ternary Local LLM Provider
//!
//! "Bringing AI home with Rust-native Burn SIMD tensors + SPQT Base-243 ternary & wave models!" - The Cheet 😺

use crate::mem8::persona::{LifePersona, PersonaSpecialty};
use crate::proxy::{LlmProvider, LlmRequest, LlmResponse};
use anyhow::Result;
use async_trait::async_trait;

#[derive(Default)]
pub struct BurnProvider {
    _model_path: Option<String>,
}

impl BurnProvider {
    pub fn new(model_path: Option<String>) -> Self {
        Self {
            _model_path: model_path,
        }
    }
}

/// Project text into a 65-trit (`{-1, 0, +1}`) SPQT Base-243 ternary signature (13 packed base-243 bytes)
pub fn compute_ternary_signature(text: &str) -> ([i8; 65], [u8; 13]) {
    let mut accum = [0i32; 65];
    for (idx, word) in text.split_whitespace().enumerate() {
        let mut h = 0x811c_9dc5u32;
        for b in word.bytes() {
            h ^= b.to_ascii_lowercase() as u32;
            h = h.wrapping_mul(0x0100_0193);
        }
        let slot_a = (h as usize) % 65;
        let slot_b = ((h >> 8) as usize + idx) % 65;
        let sign_a = if (h & 0x1000) == 0 { 1 } else { -1 };
        let sign_b = if (h & 0x2000) == 0 { 1 } else { -1 };
        accum[slot_a] += sign_a;
        accum[slot_b] += sign_b;
    }

    let mut trits = [0i8; 65];
    for (i, &val) in accum.iter().enumerate() {
        trits[i] = val.signum() as i8;
    }

    // Pack 5 trits per byte in SPQT base-243 (`3^5 = 243`)
    let mut packed = [0u8; 13];
    for (chunk_idx, chunk) in trits.chunks_exact(5).enumerate() {
        let mut byte_val = 0u8;
        let mut radix = 1u8;
        for &t in chunk {
            let digit = (t + 1) as u8; // {-1,0,+1} -> {0,1,2}
            byte_val = byte_val.wrapping_add(digit.wrapping_mul(radix));
            radix = radix.wrapping_mul(3);
        }
        packed[chunk_idx] = byte_val;
    }
    (trits, packed)
}

/// Unpack a 13-byte SPQT Base-243 (`3^5 = 243`) signature back into 65 balanced trits (`{-1, 0, +1}`).
#[inline(always)]
pub fn unpack_spqt_base243_65(packed: &[u8; 13]) -> [i8; 65] {
    let mut out = [0i8; 65];
    for (chunk_idx, &b) in packed.iter().enumerate() {
        let mut v = b.min(242);
        let base = chunk_idx * 5;
        for k in 0..5 {
            out[base + k] = (v % 3) as i8 - 1;
            v /= 3;
        }
    }
    out
}

/// Compute ternary resonance between two 65-trit (`{-1, 0, +1}`) vectors using
/// zero-heap hardware SIMD `felix_ternary_gemm_i32` (NEON on AArch64, SSE2 `PMADDWD` on x86_64).
#[inline(always)]
fn ternary_dot(a: &[i8; 65], b: &[i8; 65]) -> i32 {
    let mut acts = [0i16; 65];
    let mut i = 0usize;
    while i < 65 {
        acts[i] = a[i] as i16;
        i += 1;
    }
    crate::felix_asm::felix_ternary_gemm_i32(&acts, b)
}

/// Detect which specialist domain a prompt leans toward
fn prompt_specialty_affinity(prompt: &str, spec: PersonaSpecialty) -> f32 {
    let lower = prompt.to_ascii_lowercase();
    match spec {
        PersonaSpecialty::HistoryDocumenter => {
            if lower.contains("history")
                || lower.contains("when")
                || lower.contains("timeline")
                || lower.contains("year")
                || lower.contains("changelog")
                || lower.contains("diary")
                || lower.contains("evolution")
            {
                3.5
            } else {
                0.0
            }
        }
        PersonaSpecialty::CodeAnalyst => {
            if lower.contains("code")
                || lower.contains("rust")
                || lower.contains("struct")
                || lower.contains("function")
                || lower.contains("impl")
                || lower.contains("architecture")
                || lower.contains("scanner")
                || lower.contains("algorithm")
            {
                3.5
            } else {
                0.0
            }
        }
        PersonaSpecialty::ReadmeExpert => {
            if lower.contains("readme")
                || lower.contains("install")
                || lower.contains("flag")
                || lower.contains("cli")
                || lower.contains("usage")
                || lower.contains("command")
                || lower.contains("how do i")
            {
                3.5
            } else {
                0.0
            }
        }
        PersonaSpecialty::LifeBiographer => 1.0,
    }
}

#[async_trait]
impl LlmProvider for BurnProvider {
    async fn complete(&self, request: LlmRequest) -> Result<LlmResponse> {
        let user_prompt = request
            .messages
            .iter()
            .rev()
            .find(|m| m.role == crate::proxy::LlmRole::User)
            .map(|m| m.content.as_str())
            .unwrap_or("");

        let model_clean = request
            .model
            .strip_prefix("persona:")
            .or_else(|| request.model.strip_prefix("burn/"))
            .or_else(|| request.model.strip_prefix("candle/"))
            .or_else(|| request.model.strip_prefix("local/"))
            .unwrap_or(&request.model);

        let is_ternary_router = model_clean.contains("ternary")
            || model_clean == "mem8-wave-micro"
            || model_clean == "default";

        // 1. Direct persona request (`persona:<slug>`)
        if !is_ternary_router {
            if let Ok(persona) = LifePersona::load_cached(model_clean) {
                let reply = persona.respond_locally(user_prompt);
                return Ok(LlmResponse {
                    content: reply,
                    model: format!("persona:{}", persona.slug),
                    usage: None,
                });
            }
        }

        // 2. SPQT Base-243 Ternary Local Model Router (`burn/mem8-ternary-micro`)
        let (q_trits, q_packed) = compute_ternary_signature(user_prompt);
        let hex_sig = format!(
            "0x{:02x}{:02x}{:02x}{:02x}",
            q_packed[0], q_packed[1], q_packed[2], q_packed[3]
        );

        if let Ok(summaries) = LifePersona::list_all() {
            let mut best_score = f32::MIN;
            let mut best_persona = None;

            for summary in &summaries {
                if let Ok(p) = LifePersona::load_cached(&summary.slug) {
                    let top_eps = p.recall_episodes(user_prompt, 1);
                    if let Some(ep) = top_eps.first() {
                        let (ep_trits, _) = compute_ternary_signature(&ep.passage);
                        let t_dot = ternary_dot(&q_trits, &ep_trits) as f32;
                        let mut score =
                            t_dot * 0.25 + prompt_specialty_affinity(user_prompt, p.specialty);
                        let lower_prompt = user_prompt.to_ascii_lowercase();
                        if lower_prompt.contains(&p.name.to_ascii_lowercase())
                            || lower_prompt.contains(&p.slug)
                        {
                            score += 18.0;
                        }
                        if let Some(ref seed) = p.ancestral_seed {
                            for alias in &seed.aliases {
                                if lower_prompt.contains(&alias.to_ascii_lowercase()) {
                                    score += 16.0;
                                }
                            }
                            for place in &seed.places {
                                if lower_prompt.contains(&place.to_ascii_lowercase()) {
                                    score += 12.0;
                                }
                            }
                            for comp in &seed.companions {
                                if lower_prompt.contains(&comp.to_ascii_lowercase()) {
                                    score += 12.0;
                                }
                            }
                        }
                        for w in user_prompt.split_whitespace() {
                            let clean = w
                                .trim_matches(|c: char| !c.is_alphanumeric())
                                .to_ascii_lowercase();
                            if clean.len() >= 3 {
                                if ep.keywords.iter().any(|k| k == &clean) {
                                    score += 6.0;
                                } else if ep.passage.to_ascii_lowercase().contains(&clean) {
                                    score += 3.0;
                                }
                            }
                        }
                        if score > best_score {
                            best_score = score;
                            best_persona = Some(p);
                        }
                    }
                }
            }

            if let Some(persona) = best_persona {
                let mut reply = format!(
                    "🔺 **[SPQT Base-243 Ternary Local Model (`burn/mem8-ternary-micro` · sig `{hex_sig}`) → Routed to `{}` ({})]**\n\n",
                    persona.name,
                    persona.specialty.badge()
                );
                reply.push_str(&persona.respond_locally(user_prompt));
                return Ok(LlmResponse {
                    content: reply,
                    model: "burn/mem8-ternary-micro".to_string(),
                    usage: None,
                });
            }
        }

        // 3. Fallback: Synthesize from Smart Tree's local MEM8 memory bank (`~/.st/memories.m8`)
        if let Ok(mut mgr) = crate::memory_manager::MemoryManager::new() {
            let kws: Vec<String> = user_prompt
                .split_whitespace()
                .map(|w| w.trim_matches(|c: char| !c.is_alphanumeric()).to_string())
                .filter(|w| w.len() >= 3)
                .collect();
            if let Ok(recalled) = mgr.find(&kws) {
                if !recalled.is_empty() {
                    let mut out = format!(
                        "🔺 **[SPQT Base-243 Ternary Local Model (`{}` · sig `{hex_sig}`) · {} resonant memories]**\n\n",
                        request.model,
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
                        model: request.model,
                        usage: None,
                    });
                }
            }
        }

        Ok(LlmResponse {
            content: format!(
                "🔺 **SPQT Base-243 Ternary Local Model (`{}` · sig `{hex_sig}`)** is active! Train a Life Persona or Specialist Soul on the Dashboard (or run `m8 persona specialties .`) to ground local ternary responses.",
                request.model
            ),
            model: request.model,
            usage: None,
        })
    }

    fn name(&self) -> &'static str {
        "Burn"
    }
}
