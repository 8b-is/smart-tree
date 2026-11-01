//! Wave memory tools
//!
//! Contains handle_wave_memory handler.

use anyhow::Result;
use serde_json::{json, Value};

/// Direct access to Wave Memory system
pub async fn handle_wave_memory(args: Value) -> Result<Value> {
    use crate::mcp::wave_memory::{get_wave_memory, MemoryType};

    let operation = args["operation"]
        .as_str()
        .ok_or_else(|| anyhow::anyhow!("Missing operation"))?;

    let wave_memory = get_wave_memory();
    let mut manager = wave_memory
        .lock()
        .map_err(|e| anyhow::anyhow!("Lock error: {}", e))?;

    match operation {
        "stats" => Ok(json!({
            "operation": "stats",
            "wave_memory": manager.stats(),
            "message": "🌊 Wave Memory statistics",
        })),
        "anchor" => {
            let content = args["content"]
                .as_str()
                .ok_or_else(|| anyhow::anyhow!("Missing content for anchor"))?
                .to_string();
            let keywords: Vec<String> = args["keywords"]
                .as_array()
                .map(|arr| {
                    arr.iter()
                        .filter_map(|v| v.as_str().map(String::from))
                        .collect()
                })
                .unwrap_or_default();
            let memory_type = args["memory_type"]
                .as_str()
                .map(MemoryType::parse)
                .unwrap_or(MemoryType::Technical);
            let valence = args["valence"].as_f64().unwrap_or(0.0) as f32;
            let arousal = args["arousal"].as_f64().unwrap_or(0.5) as f32;

            let id = manager.anchor(
                content.clone(),
                keywords.clone(),
                memory_type,
                valence,
                arousal,
                "tandem:human:claude".to_string(),
                None,
            )?;

            Ok(json!({
                "operation": "anchor",
                "success": true,
                "memory_id": id,
                "content_preview": if content.len() > 50 { format!("{}...", &content[..50]) } else { content },
                "keywords": keywords,
                "memory_type": format!("{:?}", memory_type),
                "emotional_encoding": {
                    "valence": valence,
                    "arousal": arousal,
                },
                "message": "🌊 Memory anchored as wave",
            }))
        }
        "find" => {
            let keywords: Vec<String> = args["keywords"]
                .as_array()
                .map(|arr| {
                    arr.iter()
                        .filter_map(|v| v.as_str().map(String::from))
                        .collect()
                })
                .unwrap_or_default();
            let max_results = args["max_results"].as_u64().unwrap_or(10) as usize;

            let results = manager.find_by_keywords(&keywords, max_results);
            let memories: Vec<_> = results
                .iter()
                .map(|mem| {
                    json!({
                        "id": mem.id,
                        "content": mem.content,
                        "keywords": mem.keywords,
                        "memory_type": format!("{:?}", mem.memory_type),
                        "valence": mem.valence,
                        "arousal": mem.arousal,
                        "access_count": mem.access_count,
                    })
                })
                .collect();

            Ok(json!({
                "operation": "find",
                "keywords": keywords,
                "total_found": memories.len(),
                "memories": memories,
            }))
        }
        "resonance" => {
            let keywords: Vec<String> = args["keywords"]
                .as_array()
                .map(|arr| {
                    arr.iter()
                        .filter_map(|v| v.as_str().map(String::from))
                        .collect()
                })
                .unwrap_or_default();
            let memory_type = args["memory_type"]
                .as_str()
                .map(MemoryType::parse)
                .unwrap_or(MemoryType::Technical);
            let threshold = args["threshold"].as_f64().unwrap_or(0.3) as f32;
            let max_results = args["max_results"].as_u64().unwrap_or(10) as usize;

            let query = keywords.join(" ");
            let results =
                manager.find_by_resonance(&query, &keywords, memory_type, threshold, max_results);
            let memories: Vec<_> = results
                .iter()
                .map(|(mem, resonance)| {
                    json!({
                        "id": mem.id,
                        "content": mem.content,
                        "keywords": mem.keywords,
                        "memory_type": format!("{:?}", mem.memory_type),
                        "resonance_score": format!("{:.2}", resonance),
                        "valence": mem.valence,
                        "arousal": mem.arousal,
                    })
                })
                .collect();

            Ok(json!({
                "operation": "resonance",
                "search_mode": "wave_interference",
                "query": keywords,
                "threshold": threshold,
                "total_found": memories.len(),
                "memories": memories,
                "message": "🌊 Found memories by wave resonance",
            }))
        }
        "get" => {
            let id = args["memory_id"]
                .as_str()
                .ok_or_else(|| anyhow::anyhow!("Missing memory_id"))?;

            if let Some(mem) = manager.get(id) {
                Ok(json!({
                    "operation": "get",
                    "found": true,
                    "memory": {
                        "id": mem.id,
                        "content": mem.content,
                        "keywords": mem.keywords,
                        "memory_type": format!("{:?}", mem.memory_type),
                        "valence": mem.valence,
                        "arousal": mem.arousal,
                        "created_at": mem.created_at.to_rfc3339(),
                        "last_accessed": mem.last_accessed.to_rfc3339(),
                        "access_count": mem.access_count,
                        "origin": mem.origin,
                        "grid_position": { "x": mem.x, "y": mem.y, "z": mem.z },
                    }
                }))
            } else {
                Ok(json!({
                    "operation": "get",
                    "found": false,
                    "memory_id": id,
                    "message": "Memory not found",
                }))
            }
        }
        "delete" => {
            let id = args["memory_id"]
                .as_str()
                .ok_or_else(|| anyhow::anyhow!("Missing memory_id"))?;

            let deleted = manager.delete(id);
            Ok(json!({
                "operation": "delete",
                "success": deleted,
                "memory_id": id,
                "message": if deleted { "Memory deleted" } else { "Memory not found" },
            }))
        }
        _ => Err(anyhow::anyhow!(
            "Unknown wave_memory operation: {}",
            operation
        )),
    }
}
