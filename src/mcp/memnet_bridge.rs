//! MCP ↔ MemNet bridge: expose the MemNet wave-Q&A offload mesh as an MCP tool.
//!
//! The local HTTP MCP server (`st up` → `/mcp`) and the stdio MCP server
//! (`st --mcp`) both dispatch `memnet` to [`handle_memnet`].
//!
//! - `status` | `peers`: node + route table (`MemnetNodeState`), integer RTC.
//! - `ask`: local `WaveQaPacket` resonance (`0..=1000` permille, no floats)
//!   with optional LAN broadcast fallback via UDP `:8423`.
//! - `teach`: anchor a Q&A packet in the wave cache, persist, broadcast.
//! - `share`: publish a directory as a `SharedFolderManifest`.
//!
//! Local trust only, like stdio MCP — no token required.

use anyhow::{Context, Result};
use serde_json::{json, Value};

/// MCP tool definition for the MemNet bridge.
pub fn memnet_tool_definition() -> Value {
    json!({
        "name": "memnet",
        "description": "🧭 MEMNET MCP BRIDGE — Local mesh of machines sharing `MEM|8` `WaveQaPacket`s (semantic cache), shared folders, and routes over UDP `:8423` / ESP / LoRa / Bluetooth / ultrasonic chirp.\n\n🎯 PREFER THIS OVER offloading a question to an LLM or re-scanning the disk when a nearby mesh peer may already know:\n• `operation:'ask'` consults the local wave cache first (integer resonance `0..=1000` permille) and only broadcasts on the LAN when `lan:true`.\n• `operation:'teach'` anchors a Q&A so the next asker (here or over the mesh) gets it without an LLM round-trip.\n• `operation:'peers'` lists live routes with Peter's RTC (`τ`, ms, integer Marine scoring).\n• `operation:'share'` publishes a folder for mesh peers.\n\nEXAMPLES:\n✓ Local asker: memnet {operation:'ask', question:'how does the indexer skip node_modules?'}\n✓ LAN asker: memnet {operation:'ask', lan:true, question:'what does 0x62 gate?'}\n✓ Teach: memnet {operation:'teach', question:'...', answer:'...'}\n✓ Peers: memnet {operation:'peers'}",
        "inputSchema": {
            "type": "object",
            "properties": {
                "operation": {
                    "type": "string",
                    "enum": ["status", "peers", "models", "compute", "ask", "teach", "share"],
                    "description": "status/peers = mesh + route table; models/compute = close-by models + live compute headroom; ask = local/LAN wave Q&A + compute-aware close-by model routing; teach = anchor Q&A; share = publish folder",
                    "default": "peers"
                },
                "question": {
                    "type": "string",
                    "description": "Question text (for 'ask' and 'teach')"
                },
                "model": {
                    "type": "string",
                    "description": "Optional model name or alias from ~/.st/config.toml (for 'ask', e.g. 'st-deepseek', 'gemma4', 'devstral', 'persona:peter')"
                },
                "answer": {
                    "type": "string",
                    "description": "Answer text (for 'teach')"
                },
                "min_resonance_pm": {
                    "type": "integer",
                    "description": "Minimum wave resonance in permille `0..=1000` (default: 500)",
                    "default": 500
                },
                "lan": {
                    "type": "boolean",
                    "description": "Also broadcast the 'ask' query over UDP `:8423` when no local hit (default: false)",
                    "default": false
                },
                "probe": {
                    "type": "boolean",
                    "description": "Run live discovery/probing before reporting peers (default: false; slower, touches the network)",
                    "default": false
                },
                "path": {
                    "type": "string",
                    "description": "Directory to share (for 'share')"
                },
                "quota_mb": {
                    "type": "integer",
                    "description": "Drive-space quota MB offered to mesh peers (for 'share', default: 4096)"
                }
            },
            "required": []
        }
    })
}

fn clamp_pm(raw: Option<&Value>) -> u32 {
    raw.and_then(|v| v.as_u64())
        .map(|v| (v as u32).clamp(1, 1000))
        .unwrap_or(500)
}

fn node_status(probe: bool) -> Result<Value> {
    let mut state = crate::memnet::MemnetNodeState::load_or_default();
    let mut probed = 0usize;
    if probe {
        probed = state.discover_and_probe_live();
        let _ = state.save();
    }
    let up_routes = state
        .routes
        .iter()
        .filter(|r| r.status == crate::memnet::RouteStatus::Up)
        .count();
    let shares: Vec<Value> = state
        .shared_folders
        .values()
        .map(|s| {
            json!({
                "share_id": s.share_id,
                "share_name": s.share_name,
                "total_files": s.total_files,
                "total_bytes": s.total_bytes,
                "endpoint": s.endpoint,
            })
        })
        .collect();
    let routes: Vec<Value> = state
        .routes
        .iter()
        .map(|r| {
            json!({
                "address_via": r.address_via_label(),
                "node_name": r.node_name,
                "medium": r.medium.to_string(),
                "endpoint": r.endpoint,
                "latency_ms": r.latency_ms,
                "jitter_ms": r.jitter_ms,
                "rtc_ms": r.effective_rtc_ms(),
                "marine_quality": r.marine_quality_score(),
                "status": format!("{:?}", r.status),
            })
        })
        .collect();
    Ok(json!({
        "node": state.node_name,
        "address": state.local_address,
        "tier": format!("{:?}", state.tier),
        "wave_qa_entries": state.wave_qa_cache.len(),
        "routes_total": state.routes.len(),
        "routes_up": up_routes,
        "probed_up": probed,
        "routes": routes,
        "shares": shares,
        "port": crate::memnet::MEMNET_FS_PORT,
        "composite_rtc_tau_ms": state.composite_rtc_tau_ms(0xC5),
    }))
}

fn models_status() -> Result<Value> {
    let state = crate::memnet::MemnetNodeState::load_or_default();
    let hw = crate::memnet::assess_host_hardware();
    let endpoints = state.discover_nearby_model_endpoints();
    let cfg = crate::config::StConfig::load().unwrap_or_default();
    Ok(json!({
        "node": state.node_name,
        "default_model": cfg.models.default_model,
        "aliases": cfg.models.aliases,
        "memnet_config": cfg.memnet,
        "compute_headroom_permille": hw.compute_headroom_permille,
        "vram_total_mb": hw.vram_total_mb,
        "vram_available_mb": hw.vram_available_mb,
        "gpu_utilization_pct": hw.gpu_utilization_pct,
        "cpu_load_pct": hw.cpu_load_pct,
        "warm_loaded_models": hw.warm_loaded_models,
        "recommended_local_models": hw.recommended_local_models,
        "nearby_model_endpoints": endpoints,
    }))
}

fn ask(question: &str, model_hint: Option<&str>, min_pm: u32, lan: bool) -> Result<Value> {
    anyhow::ensure!(
        !question.trim().is_empty(),
        "question is required for memnet ask"
    );
    let mut state = crate::memnet::MemnetNodeState::load_or_default();
    if let Some(ans) = state.answer_with_nearby_models(question, model_hint, min_pm, lan) {
        return Ok(json!({
            "source": ans.source,
            "model_used": ans.model,
            "endpoint": ans.endpoint,
            "rtc_tau_ms": ans.rtc_ms,
            "compute_headroom_permille": ans.compute_headroom_permille,
            "resonance_pm": ans.resonance_pm,
            "packet": packet_view(&ans.packet),
        }));
    }
    Ok(json!({
        "source": "none",
        "resonance_pm": 0,
        "question": question,
        "hint": "No cached wave answer or reachable close-by model; teach one with memnet teach or ask with lower min_resonance_pm.",
    }))
}

fn packet_view(pkt: &crate::memnet::WaveQaPacket) -> Value {
    json!({
        "packet_id": pkt.packet_id,
        "origin_node": pkt.origin_node,
        "origin_name": pkt.origin_name,
        "question": pkt.question,
        "answer": pkt.answer,
        "wave_b3": pkt.wave_b3,
        "semantic_tokens": pkt.semantic_tokens,
        "hit_count": pkt.hit_count,
        "timestamp_s": pkt.timestamp_s,
    })
}

fn teach(question: &str, answer: &str) -> Result<Value> {
    anyhow::ensure!(
        !question.trim().is_empty(),
        "question is required for memnet teach"
    );
    anyhow::ensure!(
        !answer.trim().is_empty(),
        "answer is required for memnet teach"
    );
    let mut state = crate::memnet::MemnetNodeState::load_or_default();
    let pkt = state.learn_wave_qa(question, answer);
    state
        .save()
        .context("Failed to persist MemNet wave state")?;
    // Best-effort LAN offer; ignore if UDP is unavailable.
    if let Ok(raw) = crate::memnet::MemnetPayload::WaveQaResponse(pkt.clone())
        .into_raw_packet(vec![state.local_address])
    {
        let _ = state.broadcast_lan_udp(&raw);
    }
    Ok(json!({ "stored": true, "packet": packet_view(&pkt) }))
}

fn share(path: &str, quota_mb: u64) -> Result<Value> {
    let mut state = crate::memnet::MemnetNodeState::load_or_default();
    let endpoint = format!("0.0.0.0:{}", crate::memnet::MEMNET_FS_PORT);
    let manifest = crate::memnet::SharedFolderManifest::from_directory(
        std::path::Path::new(path),
        None,
        state.local_address,
        &state.node_name,
        &endpoint,
        quota_mb,
    )
    .context("Failed to build share manifest")?;
    let announce = crate::memnet::MemnetPayload::FolderShareAnnounce(manifest.clone());
    state
        .shared_folders
        .insert(manifest.share_name.clone(), manifest.clone());
    state.save().context("Failed to persist shared folder")?;
    if let Ok(raw) = announce.into_raw_packet(vec![0xF1]) {
        let _ = state.broadcast_lan_udp(&raw);
    }
    Ok(json!({
        "share_id": manifest.share_id,
        "share_name": manifest.share_name,
        "total_files": manifest.total_files,
        "total_bytes": manifest.total_bytes,
        "endpoint": manifest.endpoint,
        "offered_drive_space_mb": manifest.offered_drive_space_mb,
        "semantic_essence": manifest.semantic_essence,
    }))
}

/// Dispatch one `memnet` MCP tool call.
pub async fn handle_memnet(params: Option<Value>) -> Result<Value> {
    let params = params.unwrap_or_else(|| json!({}));
    let op = params
        .get("operation")
        .and_then(|v| v.as_str())
        .unwrap_or("peers");
    let text = match op {
        "status" | "peers" => {
            let probe = params
                .get("probe")
                .and_then(|v| v.as_bool())
                .unwrap_or(false);
            serde_json::to_string_pretty(&node_status(probe)?)?
        }
        "models" | "compute" => serde_json::to_string_pretty(&models_status()?)?,
        "ask" => {
            let q = params
                .get("question")
                .and_then(|v| v.as_str())
                .unwrap_or("");
            let model_hint = params.get("model").and_then(|v| v.as_str());
            let lan = params.get("lan").and_then(|v| v.as_bool()).unwrap_or(true);
            serde_json::to_string_pretty(&ask(
                q,
                model_hint,
                clamp_pm(params.get("min_resonance_pm")),
                lan,
            )?)?
        }
        "teach" => {
            let q = params
                .get("question")
                .and_then(|v| v.as_str())
                .unwrap_or("");
            let a = params.get("answer").and_then(|v| v.as_str()).unwrap_or("");
            serde_json::to_string_pretty(&teach(q, a)?)?
        }
        "share" => {
            let p = params.get("path").and_then(|v| v.as_str()).unwrap_or(".");
            let quota = params
                .get("quota_mb")
                .and_then(|v| v.as_u64())
                .unwrap_or(4096);
            serde_json::to_string_pretty(&share(p, quota)?)?
        }
        other => anyhow::bail!(
            "Unknown memnet operation '{}'. Valid: status, peers, models, compute, ask, teach, share",
            other
        ),
    };
    Ok(json!({ "content": [{ "type": "text", "text": text }] }))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn clamp_pm_bounds() {
        assert_eq!(clamp_pm(Some(&json!(0))), 1);
        assert_eq!(clamp_pm(Some(&json!(5000))), 1000);
        assert_eq!(clamp_pm(None), 500);
    }

    #[test]
    fn tool_definition_is_memnet() {
        let def = memnet_tool_definition();
        assert_eq!(def["name"], "memnet");
    }
}
