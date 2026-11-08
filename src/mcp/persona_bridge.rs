//! MCP ↔ MEM8 Life Persona (`m8 persona`) Bridge.
//!
//! Exposes the full `m8 persona` command suite (`list`, `ask`, `recall`, `inspect`,
//! `train`, `specialties`) as a first-class MCP tool (`persona`) in both the stdio
//! MCP server (`st --mcp`) and the HTTP MCP server (`st up` → `/mcp`).
//!
//! Zero floats in the Phoenix dyadic temperature, inner-voice register, `u16` clitic
//! tokenizer, and chronological wave-interference engine (`Q8 = 256`, `millix = 1000`).

use crate::mcp::McpContext;
use crate::mem8::persona::{
    parse_rational_temp_q8, InnerVoiceRegister, LifePersona, PersonaSpecialty,
};
use anyhow::{Context, Result};
use serde_json::{json, Value};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;

/// MCP tool definition for the `persona` tool (`m8 persona` in MCP).
pub fn persona_tool_definition() -> Value {
    json!({
        "name": "persona",
        "description": "🧬 MEM8 LIFE PERSONA & PHOENIX SOUL BRIDGE (`m8 persona`) — Train, list, inspect, recall episodic memories, or converse with `.m8` Life Personas (such as 'Grandma Jody' or workspace specialists) with zero-float Q8 chronological wave interference, SPQT Base-243 Pass-2 synthesis, Inner-Voice Register contours, and optional neural + formant-calibrated speech synthesis.\n\n🎯 OPERATIONS:\n• `operation:'list'` — List all trained `.m8` Life Personas in `~/.st/personas/` with episode counts, word counts, voice warmth, and signature phrases.\n• `operation:'ask'` — Converse with a trained `.m8` Life Persona (e.g. `name:'Grandma Jody'`, `prompt:'Hi Grandma, it\\'s me, I miss you so much since you passed'`, `temp:'184/256'`, `speak:true`).\n• `operation:'recall'` — Retrieve the top-K resonant episodic memories (`PersonaEpisode`) with their chronological era, `AuthorRole`, `InnerVoiceRegister` (`voice_f0_hz`), `interference_q8` standing-wave amplitude, and `u16` token count.\n• `operation:'inspect'` — Inspect a persona's full `.m8` Soul metadata, ancestral seed, voice signature, and inner-voice register distribution.\n• `operation:'train'` — Train a new `.m8` Life Persona from a directory or file (`name`, `path`, optional `specialty`).\n• `operation:'specialties'` — Train all 3 built-in workspace specialists (`History Documenter`, `Code Analyst`, `README Expert`) on a repository path.\n\nEXAMPLES:\n✓ List personas: persona {operation:'list'}\n✓ Ask Grandma Jody: persona {operation:'ask', name:'Grandma Jody', prompt:'Hi Grandma, it\\'s me, I miss you so much since you passed', temp:'184/256'}\n✓ Ask + Speak WAV: persona {operation:'ask', name:'Grandma Jody', prompt:'Tell me about Fred and Platoro', speak:true}\n✓ Recall episodes: persona {operation:'recall', name:'Grandma Jody', prompt:'Mayday farmhouse horses', top_k:5}\n✓ Train persona: persona {operation:'train', name:'Grandma Jody', path:'/path/to/archive', specialty:'life-biographer'}",
        "inputSchema": {
            "type": "object",
            "properties": {
                "operation": {
                    "type": "string",
                    "enum": ["list", "ask", "recall", "inspect", "train", "specialties"],
                    "description": "Persona operation: list, ask, recall, inspect, train, or specialties (auto-inferred if omitted)"
                },
                "name": {
                    "type": "string",
                    "description": "Persona name or slug (e.g. 'Grandma Jody', 'grandma-jody', 'History Documenter')"
                },
                "prompt": {
                    "type": "string",
                    "description": "Question or conversational prompt (for 'ask' and 'recall'; aliases: 'question', 'query', 'text')"
                },
                "question": {
                    "type": "string",
                    "description": "Alias for 'prompt'"
                },
                "temp": {
                    "type": "string",
                    "description": "Zero-float rational p/q or Q8 integer temperature for personality & dyadic emotional randomness (e.g. '184/256', '240', '0'; default: '184/256')",
                    "default": "184/256"
                },
                "speak": {
                    "description": "Pass true or a '.wav' file path string to synthesize spoken WAV audio modulated by inner-voice register, rational Q8 temperature, and dyadic emotion (for 'ask')"
                },
                "top_k": {
                    "type": "integer",
                    "description": "Number of resonant episodes to retrieve (for 'recall', default: 8)",
                    "default": 8
                },
                "path": {
                    "type": "string",
                    "description": "Directory or file path for 'train' or 'specialties'"
                },
                "specialty": {
                    "type": "string",
                    "enum": ["life-biographer", "history-documenter", "code-analyst", "readme-expert"],
                    "description": "Specialty lens for 'train' (default: 'life-biographer')",
                    "default": "life-biographer"
                }
            },
            "required": []
        }
    })
}

fn extract_prompt(params: &Value) -> Option<String> {
    for key in ["prompt", "question", "query", "text"] {
        if let Some(s) = params.get(key).and_then(|v| v.as_str()) {
            let trimmed = s.trim();
            if !trimmed.is_empty() {
                return Some(trimmed.to_string());
            }
        }
    }
    None
}

fn extract_name(params: &Value) -> Option<String> {
    for key in ["name", "persona", "slug"] {
        if let Some(s) = params.get(key).and_then(|v| v.as_str()) {
            let trimmed = s.trim();
            if !trimmed.is_empty() {
                return Some(trimmed.to_string());
            }
        }
    }
    None
}

fn extract_temp_q8(params: &Value) -> i32 {
    match params.get("temp") {
        Some(Value::String(s)) => parse_rational_temp_q8(s),
        Some(Value::Number(n)) => {
            if let Some(i) = n.as_i64() {
                if i <= 4 {
                    ((i as i32) * 256).clamp(0, 1024)
                } else {
                    (i as i32).clamp(0, 1024)
                }
            } else {
                184
            }
        }
        _ => 184,
    }
}

fn extract_speak_path(params: &Value, persona_name: &str) -> Option<PathBuf> {
    match params.get("speak") {
        Some(Value::Bool(true)) => {
            let slug = LifePersona::slugify(persona_name);
            let default_dir = dirs::home_dir()
                .unwrap_or_else(|| PathBuf::from("."))
                .join(".st")
                .join("personas");
            Some(default_dir.join(format!("{slug}_phoenix_reply.wav")))
        }
        Some(Value::String(s)) => {
            let trimmed = s.trim();
            if trimmed.is_empty() || trimmed.eq_ignore_ascii_case("true") {
                let slug = LifePersona::slugify(persona_name);
                let default_dir = dirs::home_dir()
                    .unwrap_or_else(|| PathBuf::from("."))
                    .join(".st")
                    .join("personas");
                Some(default_dir.join(format!("{slug}_phoenix_reply.wav")))
            } else if trimmed.eq_ignore_ascii_case("false") {
                None
            } else {
                Some(PathBuf::from(trimmed))
            }
        }
        _ => None,
    }
}

/// Dispatch one `persona` MCP tool call (`list`, `ask`, `recall`, `inspect`, `train`, `specialties`).
pub async fn handle_persona(params: Option<Value>, ctx: Arc<McpContext>) -> Result<Value> {
    let mut params = params.unwrap_or_else(|| json!({}));
    super::helpers::normalize_path_args(&mut params, &ctx);

    let prompt_opt = extract_prompt(&params);
    let name_opt = extract_name(&params);
    let has_path = params
        .get("path")
        .and_then(|v| v.as_str())
        .is_some_and(|s| !s.trim().is_empty());

    let raw_op = params
        .get("operation")
        .and_then(|v| v.as_str())
        .unwrap_or("");
    let operation = match raw_op {
        "persona_list" => "list",
        "persona_ask" => "ask",
        "persona_recall" => "recall",
        "persona_inspect" => "inspect",
        "persona_train" => "train",
        "persona_specialties" => "specialties",
        "" => {
            if prompt_opt.is_some() {
                "ask"
            } else if has_path && name_opt.is_some() {
                "train"
            } else if name_opt.is_some() {
                "inspect"
            } else {
                "list"
            }
        }
        other => other,
    };

    match operation {
        "list" | "ls" => {
            let summaries = tokio::task::spawn_blocking(LifePersona::list_all)
                .await
                .context("persona list worker failed")??;

            let mut md = String::new();
            if summaries.is_empty() {
                md.push_str("No trained `.m8` personas found in `~/.st/personas/`.\n");
                md.push_str(
                    "Train one with: `persona {operation:'train', name:'Grandma Jody', path:'/path/to/writings'}`",
                );
            } else {
                md.push_str(&format!(
                    "🧬 **Trained MEM8 Life Personas (`{}` in `~/.st/personas/`)**:\n\n",
                    summaries.len()
                ));
                for p in &summaries {
                    let size_kib = p.m8_size_bytes / 1024;
                    let size_rem = ((p.m8_size_bytes % 1024) * 10) / 1024;
                    md.push_str(&format!(
                        "- {} **{}** (`{}`) [{}] — **{} episodes**, {} words, {} files (`{}.{}` KiB `.m8`)\n",
                        p.specialty.badge(),
                        p.name,
                        p.slug,
                        p.specialty.slug(),
                        p.episode_count,
                        p.total_words,
                        p.files_ingested,
                        size_kib,
                        size_rem
                    ));
                    if !p.sample_phrases.is_empty() {
                        md.push_str(&format!(
                            "  _Cadence_: {}\n",
                            p.sample_phrases.join(" · ")
                        ));
                    }
                }
            }

            Ok(json!({
                "content": [{
                    "type": "text",
                    "text": md
                }],
                "personas": summaries
            }))
        }
        "ask" | "chat" | "converse" => {
            let name = name_opt.context(
                "Missing 'name' parameter for persona ask (e.g. 'Grandma Jody' or 'grandma-jody')",
            )?;
            let prompt = prompt_opt.context(
                "Missing 'prompt' (or 'question') parameter for persona ask",
            )?;
            let temp_q8 = extract_temp_q8(&params);
            let speak_path = extract_speak_path(&params, &name);

            let (reply, slug, episodes_count) = tokio::task::spawn_blocking(move || -> Result<(String, String, usize)> {
                let persona = LifePersona::load_cached(&name)?;
                let text = persona.respond_with_phoenix(
                    &prompt,
                    temp_q8,
                    speak_path.as_deref(),
                );
                Ok((text, persona.slug.clone(), persona.episodes.len()))
            })
            .await
            .context("persona ask worker failed")??;

            Ok(json!({
                "content": [{
                    "type": "text",
                    "text": reply
                }],
                "persona": slug,
                "episodes_count": episodes_count,
                "temp_q8": format!("{temp_q8}/256")
            }))
        }
        "recall" | "episodes" | "search" => {
            let name = name_opt.context(
                "Missing 'name' parameter for persona recall (e.g. 'Grandma Jody')",
            )?;
            let prompt = prompt_opt.context(
                "Missing 'prompt' (or 'question') parameter for persona recall",
            )?;
            let temp_q8 = extract_temp_q8(&params);
            let top_k = params
                .get("top_k")
                .and_then(|v| v.as_u64())
                .map(|v| (v as usize).clamp(1, 32))
                .unwrap_or(8);

            let res = tokio::task::spawn_blocking(move || -> Result<Value> {
                let persona = LifePersona::load_cached(&name)?;
                let mut rng = ((temp_q8 as u64).wrapping_mul(0x9E3779B97F4A7C15)) ^ 0xBF58476D1CE4E5B9;
                let recalled = persona.recall_episodes_with_temp(&prompt, top_k, temp_q8, &mut rng);
                let items: Vec<Value> = recalled
                    .iter()
                    .map(|ep| {
                        json!({
                            "source": ep.source,
                            "era": ep.era,
                            "event_time": ep.event_time,
                            "written_time": ep.written_time,
                            "temporal_order": ep.temporal_order,
                            "author_role": format!("{:?}", ep.author_role),
                            "origin": ep.origin,
                            "inner_voice": ep.inner_voice.label(),
                            "voice_f0_hz": ep.voice_f0_hz,
                            "inner_voice_spqt": ep.inner_voice_spqt,
                            "interference_q8": format!("{}/256", ep.interference_q8),
                            "token_ids_count": ep.token_ids.len(),
                            "keywords": ep.keywords,
                            "passage": ep.passage,
                        })
                    })
                    .collect();
                Ok(json!({
                    "persona": persona.name,
                    "slug": persona.slug,
                    "query": prompt,
                    "temp_q8": format!("{temp_q8}/256"),
                    "count": items.len(),
                    "episodes": items,
                }))
            })
            .await
            .context("persona recall worker failed")??;

            Ok(json!({
                "content": [{
                    "type": "text",
                    "text": serde_json::to_string_pretty(&res)?
                }]
            }))
        }
        "inspect" | "info" | "status" | "soul" => {
            let name = name_opt.context(
                "Missing 'name' parameter for persona inspect (e.g. 'Grandma Jody')",
            )?;

            let info = tokio::task::spawn_blocking(move || -> Result<Value> {
                let persona = LifePersona::load_cached(&name)?;
                let mut register_counts: HashMap<&'static str, usize> = HashMap::new();
                let mut max_wave_q8: i16 = 0;
                let mut min_wave_q8: i16 = i16::MAX;
                let mut total_tokens_u16: usize = 0;

                for ep in &persona.episodes {
                    let lbl = match ep.inner_voice {
                        InnerVoiceRegister::WhisperedDiary => "WhisperedDiary (182 Hz)",
                        InnerVoiceRegister::HearthStoryteller => "HearthStoryteller (188 Hz)",
                        InnerVoiceRegister::WryHumorBanter => "WryHumorBanter (194 Hz)",
                        InnerVoiceRegister::FierceConviction => "FierceConviction (198 Hz)",
                        InnerVoiceRegister::TenderElegy => "TenderElegy (178 Hz)",
                        InnerVoiceRegister::AncestralEcho => "AncestralEcho (184 Hz)",
                    };
                    *register_counts.entry(lbl).or_insert(0) += 1;
                    if ep.interference_q8 > max_wave_q8 {
                        max_wave_q8 = ep.interference_q8;
                    }
                    if ep.interference_q8 < min_wave_q8 {
                        min_wave_q8 = ep.interference_q8;
                    }
                    total_tokens_u16 += ep.token_ids.len();
                }
                if persona.episodes.is_empty() {
                    min_wave_q8 = 0;
                }

                Ok(json!({
                    "name": persona.name,
                    "slug": persona.slug,
                    "specialty": persona.specialty.slug(),
                    "badge": persona.specialty.badge(),
                    "source_path": persona.source_path,
                    "trained_at": persona.trained_at,
                    "files_ingested": persona.files_ingested,
                    "total_words": persona.total_words,
                    "episodes_count": persona.episodes.len(),
                    "total_u16_token_ids": total_tokens_u16,
                    "standing_wave_range_q8": format!("{min_wave_q8}/256 .. {max_wave_q8}/256"),
                    "inner_voice_registers": register_counts,
                    "ancestral_seed": persona.ancestral_seed,
                    "voice": {
                        "avg_sentence_words": persona.voice.avg_sentence_words,
                        "warmth": persona.voice.warmth,
                        "expressiveness": persona.voice.expressiveness,
                        "signature_phrases": persona.voice.signature_phrases,
                        "top_vocabulary": persona.voice.vocabulary.iter().take(24).collect::<Vec<_>>(),
                        "sample_openings": persona.voice.sample_openings,
                    }
                }))
            })
            .await
            .context("persona inspect worker failed")??;

            Ok(json!({
                "content": [{
                    "type": "text",
                    "text": serde_json::to_string_pretty(&info)?
                }]
            }))
        }
        "train" => {
            let name = name_opt.context(
                "Missing 'name' parameter for persona train (e.g. 'Grandma Jody')",
            )?;
            let path_str = params
                .get("path")
                .and_then(|v| v.as_str())
                .filter(|s| !s.trim().is_empty())
                .context("Missing 'path' parameter for persona train")?
                .to_string();
            let spec_str = params
                .get("specialty")
                .and_then(|v| v.as_str())
                .unwrap_or("life-biographer");
            let spec = PersonaSpecialty::from_str_loose(spec_str);

            let out = tokio::task::spawn_blocking(move || -> Result<Value> {
                let (persona, saved_path, bytes) =
                    LifePersona::train_with_specialty(&name, PathBuf::from(&path_str).as_path(), spec)?;
                let size_kib = bytes / 1024;
                Ok(json!({
                    "ok": true,
                    "name": persona.name,
                    "slug": persona.slug,
                    "specialty": persona.specialty.slug(),
                    "badge": persona.specialty.badge(),
                    "source_path": persona.source_path,
                    "files_ingested": persona.files_ingested,
                    "total_words": persona.total_words,
                    "episodes_count": persona.episodes.len(),
                    "saved_m8_path": saved_path.to_string_lossy(),
                    "m8_size_bytes": bytes,
                    "m8_size_kib": size_kib,
                    "signature_phrases": persona.voice.signature_phrases,
                }))
            })
            .await
            .context("persona train worker failed")??;

            Ok(json!({
                "content": [{
                    "type": "text",
                    "text": serde_json::to_string_pretty(&out)?
                }]
            }))
        }
        "specialties" => {
            let default_cwd = ctx
                .cwd
                .lock()
                .ok()
                .map(|p| p.clone())
                .or_else(|| std::env::current_dir().ok())
                .unwrap_or_else(|| PathBuf::from("."));
            let target_path = params
                .get("path")
                .and_then(|v| v.as_str())
                .filter(|s| !s.trim().is_empty())
                .map(PathBuf::from)
                .unwrap_or(default_cwd);

            let out = tokio::task::spawn_blocking(move || -> Result<Value> {
                let trained = LifePersona::train_workspace_specialties(&target_path);
                let mut items: Vec<Value> = Vec::new();
                for s in trained.into_iter().flatten() {
                    items.push(json!({
                        "name": s.name,
                        "slug": s.slug,
                        "specialty": s.specialty.slug(),
                        "badge": s.specialty.badge(),
                        "episode_count": s.episode_count,
                        "total_words": s.total_words,
                        "files_ingested": s.files_ingested,
                        "m8_size_bytes": s.m8_size_bytes,
                        "sample_phrases": s.sample_phrases,
                    }));
                }
                Ok(json!({
                    "ok": true,
                    "workspace": target_path.to_string_lossy(),
                    "specialists_trained": items.len(),
                    "specialists": items,
                }))
            })
            .await
            .context("persona specialties worker failed")??;

            Ok(json!({
                "content": [{
                    "type": "text",
                    "text": serde_json::to_string_pretty(&out)?
                }]
            }))
        }
        other => anyhow::bail!(
            "Unknown persona operation '{}'. Valid operations: list, ask, recall, inspect, train, specialties",
            other
        ),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tool_definition_is_persona() {
        let def = persona_tool_definition();
        assert_eq!(def["name"], "persona");
        let ops = def["inputSchema"]["properties"]["operation"]["enum"]
            .as_array()
            .expect("operation enum");
        assert!(ops.contains(&json!("list")));
        assert!(ops.contains(&json!("ask")));
        assert!(ops.contains(&json!("recall")));
        assert!(ops.contains(&json!("inspect")));
        assert!(ops.contains(&json!("train")));
        assert!(ops.contains(&json!("specialties")));
    }

    #[tokio::test]
    async fn persona_mcp_dispatch_list_and_memory_bridge_work() {
        let server = crate::mcp::McpServer::new(crate::mcp::McpConfig::default());
        let ctx = server.context.clone();

        let list_resp = crate::mcp::tools_consolidated::dispatch_consolidated_tool(
            "persona",
            Some(json!({"operation": "list"})),
            ctx.clone(),
        )
        .await
        .expect("persona list via MCP dispatch");
        assert!(list_resp["content"][0]["text"].as_str().is_some());

        let mem_resp = crate::mcp::tools_consolidated::dispatch_consolidated_tool(
            "memory",
            Some(json!({"operation": "persona_list"})),
            ctx,
        )
        .await
        .expect("memory persona_list via MCP dispatch");
        assert!(mem_resp["content"][0]["text"].as_str().is_some());
    }
}
