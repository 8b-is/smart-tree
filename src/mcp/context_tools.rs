//! MCP tools for context gathering from AI tool directories
//!
//! Now powered by Wave Memory for semantic storage and resonance-based retrieval!

use crate::context_gatherer::{ContextGatherer, GatherConfig, GatheredContext};
use crate::mcp::wave_memory::{get_wave_memory, MemoryType};
use anyhow::{Context as _, Result};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::path::PathBuf;

/// Request structure for gather_project_context tool
#[derive(Debug, Deserialize)]
pub struct GatherProjectContextRequest {
    /// Path to the project to gather context for
    pub project_path: String,
    /// Optional list of AI tool directories to search (defaults to all)
    pub search_dirs: Option<Vec<String>>,
    /// Additional custom directories to search
    pub custom_dirs: Option<Vec<String>>,
    /// Project identifiers to look for (e.g., unique strings, URLs)
    pub project_identifiers: Option<Vec<String>>,
    /// Maximum number of contexts to return
    pub max_results: Option<usize>,
    /// Minimum relevance score (0.0 to 1.0)
    pub min_relevance: Option<f32>,
    /// Output format: "json", "m8", "summary", or "temporal"
    pub output_format: Option<String>,
    /// Enable privacy mode (redact sensitive info)
    pub privacy_mode: Option<bool>,
    /// Temporal resolution for analysis: "hour", "day", "week", "month", "quarter", "year"
    pub temporal_resolution: Option<String>,
    /// Apply temporal decay (half-life in days)
    pub temporal_decay_days: Option<f32>,
}

/// Response structure for gathered context
#[derive(Debug, Serialize)]
pub struct GatherProjectContextResponse {
    pub project_path: String,
    pub total_contexts_found: usize,
    pub contexts_returned: usize,
    pub sources_summary: std::collections::HashMap<String, usize>,
    pub contexts: Vec<ContextSummary>,
    pub m8_data: Option<String>, // Base64 encoded M8 data
}

#[derive(Debug, Serialize)]
pub struct ContextSummary {
    pub source_path: String,
    pub ai_tool: String,
    pub content_type: String,
    pub relevance_score: f32,
    pub size_bytes: usize,
    pub preview: String,
    pub metadata: std::collections::HashMap<String, String>,
}

/// Search for context about current project in AI tool directories
pub async fn gather_project_context(
    req: GatherProjectContextRequest,
    _permission_check: impl Fn(serde_json::Value) -> Result<bool>,
) -> Result<Value> {
    let project_path = PathBuf::from(&req.project_path);

    // Verify project path exists
    if !project_path.exists() {
        return Ok(json!({
            "error": format!("Project path does not exist: {}", req.project_path)
        }));
    }

    // For now, we skip permission checks
    // TODO: Implement proper permission system when available

    // Build configuration
    let mut config = GatherConfig::default();

    if let Some(dirs) = req.search_dirs {
        config.search_dirs = dirs;
    }

    if let Some(custom) = req.custom_dirs {
        config.custom_dirs = custom.into_iter().map(PathBuf::from).collect();
    }

    if let Some(identifiers) = req.project_identifiers {
        config.project_identifiers = identifiers;
    }

    if let Some(privacy) = req.privacy_mode {
        config.privacy_mode = privacy;
    }

    // Create gatherer and collect contexts
    let mut gatherer = ContextGatherer::new(project_path.clone(), config);
    gatherer.gather_all()?;

    // Apply temporal decay if requested
    if let Some(half_life) = req.temporal_decay_days {
        gatherer.apply_temporal_decay(half_life);
    }

    let all_contexts = gatherer.contexts();
    let min_relevance = req.min_relevance.unwrap_or(0.0);
    let max_results = req.max_results.unwrap_or(50);

    // Filter by relevance and limit results
    let filtered_contexts: Vec<&GatheredContext> = all_contexts
        .iter()
        .filter(|c| c.relevance_score >= min_relevance)
        .take(max_results)
        .collect();

    // Generate response based on format
    let output_format = req.output_format.as_deref().unwrap_or("summary");

    match output_format {
        "m8" => {
            let m8_data = gatherer.to_m8()?;
            let encoded =
                base64::Engine::encode(&base64::engine::general_purpose::STANDARD, &m8_data);

            Ok(json!({
                "project_path": project_path.to_string_lossy(),
                "total_contexts": all_contexts.len(),
                "m8_data": encoded,
                "m8_size_bytes": m8_data.len(),
            }))
        }

        "json" => {
            // Return full JSON representation
            Ok(json!({
                "project_path": project_path.to_string_lossy(),
                "contexts": filtered_contexts,
            }))
        }

        "partnership" => {
            // AI-Human partnership analysis
            let analysis = gatherer.analyze_partnership();

            // Also get rapport indices from session tracker
            let rapport_summary: Vec<_> = gatherer
                .session_tracker()
                .rapport_indices
                .iter()
                .map(|(pair, rapport)| {
                    json!({
                        "pair": pair,
                        "score": format!("{:.2}", rapport.overall_score),
                        "trend": if rapport.evolution_trend > 0.05 {
                            "📈"
                        } else if rapport.evolution_trend < -0.05 {
                            "📉"
                        } else {
                            "➡️"
                        },
                    })
                })
                .collect();

            Ok(json!({
                "project_path": project_path.to_string_lossy(),
                "partnership_analysis": {
                    "total_interactions": analysis.total_interactions,
                    "collaborative_sessions": analysis.collaborative_sessions.len(),
                    "partnership_health": {
                        "score": format!("{:.2}", analysis.partnership_health.overall_score),
                        "status": analysis.partnership_health.status,
                        "strengths": analysis.partnership_health.strengths,
                        "improvements": analysis.partnership_health.areas_for_improvement,
                    },
                    "collaboration_metrics": {
                        "productivity_rate": format!("{:.2}", analysis.collaboration_metrics.productivity_rate),
                        "learning_rate": format!("{:.2}", analysis.collaboration_metrics.learning_rate),
                        "stuck_rate": format!("{:.2}", analysis.collaboration_metrics.stuck_rate),
                        "collaboration_depth": format!("{:.2}", analysis.collaboration_metrics.collaboration_depth),
                        "mutual_understanding": format!("{:.2}", analysis.collaboration_metrics.mutual_understanding),
                    },
                    "relationship_evolution": {
                        "productivity_trend": if analysis.relationship_evolution.productivity_trend > 0.1 {
                            "📈 Improving"
                        } else if analysis.relationship_evolution.productivity_trend < -0.1 {
                            "📉 Declining"
                        } else {
                            "➡️ Stable"
                        },
                        "understanding_trend": if analysis.relationship_evolution.understanding_trend > 0.1 {
                            "📈 Improving"
                        } else if analysis.relationship_evolution.understanding_trend < -0.1 {
                            "📉 Declining"
                        } else {
                            "➡️ Stable"
                        },
                        "trust_level": format!("{:.2}", analysis.relationship_evolution.trust_indicators.autonomy_level),
                    },
                    "shared_understanding": {
                        "vocabulary_size": analysis.shared_understanding.vocabulary_size,
                        "communication_efficiency": format!("{:.2}", analysis.shared_understanding.communication_efficiency),
                        "expertise_areas": analysis.shared_understanding.domain_expertise_areas,
                    },
                    "interaction_patterns": {
                        "preferred_tools": analysis.interaction_patterns.tool_preferences.into_iter()
                            .max_by_key(|(_, count)| *count)
                            .map(|(tool, _)| tool)
                            .unwrap_or_else(|| "none".to_string()),
                        "peak_hours": analysis.interaction_patterns.peak_collaboration_hours,
                        "avg_session_minutes": analysis.interaction_patterns.average_session_length.num_minutes(),
                    },
                    "recommendations": analysis.recommendations,
                    "rapport_indices": rapport_summary,
                },
                "summary": format!(
                    "Partnership Health: {} ({:.0}%) - {} collaborative sessions analyzed",
                    analysis.partnership_health.status,
                    analysis.partnership_health.overall_score * 100.0,
                    analysis.collaborative_sessions.len()
                ),
            }))
        }

        "temporal" => {
            // Temporal analysis output
            let resolution = match req.temporal_resolution.as_deref() {
                Some("hour") => crate::context_gatherer::temporal::TemporalResolution::Hour,
                Some("day") => crate::context_gatherer::temporal::TemporalResolution::Day,
                Some("week") => crate::context_gatherer::temporal::TemporalResolution::Week,
                Some("month") => crate::context_gatherer::temporal::TemporalResolution::Month,
                Some("quarter") => crate::context_gatherer::temporal::TemporalResolution::Quarter,
                Some("year") => crate::context_gatherer::temporal::TemporalResolution::Year,
                _ => crate::context_gatherer::temporal::TemporalResolution::Day,
            };

            let patterns = gatherer.analyze_temporal(resolution);
            let wave_grid = gatherer.create_temporal_waves(resolution);

            Ok(json!({
                "project_path": project_path.to_string_lossy(),
                "temporal_analysis": {
                    "resolution": format!("{:?}", resolution),
                    "patterns": patterns,
                    "work_sessions": patterns.work_sessions.len(),
                    "peak_times": patterns.peak_times.len(),
                    "momentum": patterns.momentum,
                    "total_duration_days": patterns.total_duration.num_days(),
                    "active_days": patterns.active_days,
                    "periodic_patterns": patterns.periodic_patterns,
                    "resonance_peaks": wave_grid.find_resonance_peaks().len(),
                },
                "summary": format!(
                    "Found {} work sessions over {} active days with momentum {:.2}",
                    patterns.work_sessions.len(),
                    patterns.active_days,
                    patterns.momentum
                ),
            }))
        }

        _ => {
            // Default summary format
            let summaries: Vec<ContextSummary> = filtered_contexts
                .iter()
                .map(|c| create_context_summary(c))
                .collect();

            let sources_summary = calculate_sources_summary(&filtered_contexts);

            let response = GatherProjectContextResponse {
                project_path: project_path.to_string_lossy().to_string(),
                total_contexts_found: all_contexts.len(),
                contexts_returned: summaries.len(),
                sources_summary,
                contexts: summaries,
                m8_data: None,
            };

            Ok(serde_json::to_value(response)?)
        }
    }
}

/// Create a summary of a context entry
fn create_context_summary(context: &GatheredContext) -> ContextSummary {
    let preview = match &context.content {
        crate::context_gatherer::ContextContent::Text(t) => {
            t.chars().take(200).collect::<String>() + if t.len() > 200 { "..." } else { "" }
        }
        crate::context_gatherer::ContextContent::Json(j) => {
            let s = j.to_string();
            s.chars().take(200).collect::<String>() + if s.len() > 200 { "..." } else { "" }
        }
        crate::context_gatherer::ContextContent::Xml(x) => {
            x.chars().take(200).collect::<String>() + if x.len() > 200 { "..." } else { "" }
        }
        crate::context_gatherer::ContextContent::Binary(b) => {
            format!("[Binary data: {} bytes]", b.len())
        }
    };

    let size_bytes = match &context.content {
        crate::context_gatherer::ContextContent::Text(t) => t.len(),
        crate::context_gatherer::ContextContent::Json(j) => j.to_string().len(),
        crate::context_gatherer::ContextContent::Xml(x) => x.len(),
        crate::context_gatherer::ContextContent::Binary(b) => b.len(),
    };

    ContextSummary {
        source_path: context.source_path.to_string_lossy().to_string(),
        ai_tool: context.ai_tool.clone(),
        content_type: format!("{:?}", context.content_type),
        relevance_score: context.relevance_score,
        size_bytes,
        preview,
        metadata: context.metadata.clone(),
    }
}

/// Calculate summary of context sources
fn calculate_sources_summary(
    contexts: &[&GatheredContext],
) -> std::collections::HashMap<String, usize> {
    let mut summary = std::collections::HashMap::new();

    for context in contexts {
        *summary.entry(context.ai_tool.clone()).or_insert(0) += 1;
    }

    summary
}

/// Request structure for analyze_ai_tool_usage
#[derive(Debug, Deserialize)]
pub struct AnalyzeAiToolUsageRequest {
    /// Optional specific AI tool to analyze (e.g., ".claude", ".cursor")
    pub tool_name: Option<String>,
    /// Time range in days (default: 30)
    pub days: Option<u32>,
    /// Include detailed file paths
    pub _include_paths: Option<bool>,
}

/// Request structure for anchor_collaborative_memory
#[derive(Debug, Deserialize)]
pub struct AnchorMemoryRequest {
    /// The context or insight to anchor
    pub context: String,
    /// Keywords for future retrieval
    pub keywords: Vec<String>,
    /// Type of anchor: "pattern_insight", "solution", "breakthrough", "learning", "joke", "technical", "process"
    pub anchor_type: String,
    /// Origin: "human", "ai:<tool>", or "tandem:<human>:<ai>" (defaults to "tandem:human:claude")
    #[serde(default = "default_origin")]
    pub origin: String,
    /// Project path to associate with
    pub project_path: Option<String>,
}

/// Default origin for collaborative memories - the beautiful human-AI partnership! 🤝
fn default_origin() -> String {
    "tandem:human:claude".to_string()
}

/// Anchor a collaborative memory for future retrieval
///
/// Now powered by Wave Memory - memories are stored as waves in a 3D cognitive grid
/// with emotional encoding and semantic positioning for resonance-based retrieval!
pub async fn anchor_collaborative_memory(
    req: AnchorMemoryRequest,
    _permission_check: impl Fn(serde_json::Value) -> Result<bool>,
) -> Result<Value> {
    let project_path = req.project_path.as_ref().map(PathBuf::from);

    // Convert anchor type to memory type for wave storage
    let memory_type = MemoryType::parse(&req.anchor_type);

    // Parse emotional context from the content (simple heuristics)
    let (valence, arousal) = estimate_emotional_context(&req.context, &req.anchor_type);

    // Use wave memory for storage
    let wave_memory = get_wave_memory();
    let result = {
        let mut manager = wave_memory.lock().map_err(|e| anyhow::anyhow!("Lock error: {}", e))?;
        manager.anchor(
            req.context.clone(),
            req.keywords.clone(),
            memory_type,
            valence,
            arousal,
            req.origin.clone(),
            project_path.clone(),
        )
    };

    match result {
        Ok(anchor_id) => {
            // Also store in legacy system for backward compatibility
            if let Some(ref path) = project_path {
                let config = GatherConfig::default();
                let mut gatherer = ContextGatherer::new(path.clone(), config);

                // Parse origin for legacy system
                let origin = if req.origin.starts_with("tandem:") {
                    let parts: Vec<&str> = req.origin.split(':').collect();
                    if parts.len() >= 3 {
                        crate::context_gatherer::collab_session::CollaborativeOrigin::Tandem {
                            human: parts[1].to_string(),
                            ai: parts[2].to_string(),
                        }
                    } else {
                        crate::context_gatherer::collab_session::CollaborativeOrigin::Emergent
                    }
                } else if req.origin.starts_with("ai:") {
                    let ai_name = req.origin.strip_prefix("ai:").unwrap_or("claude");
                    crate::context_gatherer::collab_session::CollaborativeOrigin::Single(ai_name.to_string())
                } else if req.origin == "human" {
                    crate::context_gatherer::collab_session::CollaborativeOrigin::Single("human".to_string())
                } else {
                    crate::context_gatherer::collab_session::CollaborativeOrigin::Emergent
                };

                let anchor_type = match req.anchor_type.as_str() {
                    "pattern_insight" | "pattern" => crate::context_gatherer::collab_session::AnchorType::PatternInsight,
                    "solution" | "breakthrough" => crate::context_gatherer::collab_session::AnchorType::Solution,
                    "learning" | "learning_moment" => crate::context_gatherer::collab_session::AnchorType::LearningMoment,
                    "joke" | "shared_joke" => crate::context_gatherer::collab_session::AnchorType::SharedJoke,
                    "technical" | "technical_pattern" => crate::context_gatherer::collab_session::AnchorType::TechnicalPattern,
                    "process" | "process_improvement" => crate::context_gatherer::collab_session::AnchorType::ProcessImprovement,
                    _ => crate::context_gatherer::collab_session::AnchorType::PatternInsight,
                };

                let _ = gatherer.anchor_memory(origin, anchor_type, req.context.clone(), req.keywords.clone());
            }

            Ok(json!({
                "success": true,
                "anchor_id": anchor_id,
                "message": format!("🌊 Memory anchored as wave with {} keywords", req.keywords.len()),
                "wave_info": {
                    "frequency_band": format!("{:?}", memory_type),
                    "emotional_valence": valence,
                    "emotional_arousal": arousal,
                },
                "retrieval_hint": "Use find_collaborative_memories for keyword search, or wave_memory for resonance search",
            }))
        }
        Err(e) => Ok(json!({
            "error": format!("Failed to anchor memory: {}", e)
        })),
    }
}

/// Estimate emotional context from content (simple heuristics)
fn estimate_emotional_context(content: &str, anchor_type: &str) -> (f32, f32) {
    let content_lower = content.to_lowercase();

    // Base valence/arousal from anchor type
    let (mut valence, mut arousal): (f32, f32) = match anchor_type {
        "breakthrough" | "solution" => (0.8, 0.7),  // Very positive, exciting
        "joke" | "shared_joke" => (0.9, 0.8),       // Very positive, high energy
        "learning" | "learning_moment" => (0.5, 0.5), // Neutral-positive, moderate
        "pattern" | "pattern_insight" => (0.3, 0.3), // Calm, thoughtful
        "technical" => (0.2, 0.4),                   // Neutral, focused
        _ => (0.0, 0.5),                             // Neutral
    };

    // Adjust based on content sentiment words
    let positive_words = ["solved", "fixed", "works", "success", "great", "awesome", "love", "perfect", "breakthrough"];
    let negative_words = ["bug", "error", "failed", "problem", "issue", "crash", "broken"];
    let excitement_words = ["!", "amazing", "incredible", "finally", "eureka", "aha"];

    for word in positive_words.iter() {
        if content_lower.contains(word) {
            valence = (valence + 0.1_f32).min(1.0_f32);
        }
    }

    for word in negative_words.iter() {
        if content_lower.contains(word) {
            valence = (valence - 0.1_f32).max(-1.0_f32);
        }
    }

    for word in excitement_words.iter() {
        if content_lower.contains(word) {
            arousal = (arousal + 0.1_f32).min(1.0_f32);
        }
    }

    (valence, arousal)
}

/// Request structure for find_collaborative_memories
#[derive(Debug, Deserialize)]
pub struct FindMemoriesRequest {
    /// Keywords to search for
    pub keywords: Vec<String>,
    /// Optional project path
    pub project_path: Option<String>,
    /// Maximum results to return
    pub max_results: Option<usize>,
    /// Use resonance search (semantic similarity) instead of just keywords
    #[serde(default)]
    pub use_resonance: bool,
    /// Optional memory type filter
    pub memory_type: Option<String>,
    /// Minimum resonance threshold (0.0 to 1.0, default 0.3)
    pub resonance_threshold: Option<f32>,
}

/// Find previously anchored collaborative memories
///
/// Now supports two search modes:
/// 1. Keyword search (default): Fast lookup by exact keyword matches
/// 2. Resonance search (use_resonance: true): Semantic similarity via wave interference
pub async fn find_collaborative_memories(
    req: FindMemoriesRequest,
    _permission_check: impl Fn(serde_json::Value) -> Result<bool>,
) -> Result<Value> {
    let max_results = req.max_results.unwrap_or(10);

    // Use wave memory for search
    let wave_memory = get_wave_memory();
    let mut manager = wave_memory.lock().map_err(|e| anyhow::anyhow!("Lock error: {}", e))?;

    if req.use_resonance {
        // Resonance search: find semantically similar memories
        let memory_type = req.memory_type
            .as_ref()
            .map(|s| MemoryType::parse(s))
            .unwrap_or(MemoryType::Technical);
        let threshold = req.resonance_threshold.unwrap_or(0.3);

        let query_content = req.keywords.join(" ");
        let results = manager.find_by_resonance(
            &query_content,
            &req.keywords,
            memory_type,
            threshold,
            max_results,
        );

        let memories: Vec<_> = results.iter().map(|(mem, resonance)| {
            json!({
                "id": &mem.id,
                "content": &mem.content,
                "keywords": &mem.keywords,
                "memory_type": format!("{:?}", mem.memory_type),
                "resonance_score": format!("{:.2}", resonance),
                "emotional_valence": mem.valence,
                "emotional_arousal": mem.arousal,
                "created_at": mem.created_at.to_rfc3339(),
                "access_count": mem.access_count,
                "origin": &mem.origin,
            })
        }).collect();

        Ok(json!({
            "search_mode": "resonance",
            "keywords_searched": req.keywords,
            "resonance_threshold": threshold,
            "total_found": memories.len(),
            "memories": memories,
            "wave_stats": manager.stats(),
        }))
    } else {
        // Keyword search: fast lookup
        let results = manager.find_by_keywords(&req.keywords, max_results);

        let memories: Vec<_> = results.iter().map(|mem| {
            json!({
                "id": &mem.id,
                "content": &mem.content,
                "keywords": &mem.keywords,
                "memory_type": format!("{:?}", mem.memory_type),
                "emotional_valence": mem.valence,
                "emotional_arousal": mem.arousal,
                "created_at": mem.created_at.to_rfc3339(),
                "access_count": mem.access_count,
                "origin": &mem.origin,
            })
        }).collect();

        // Also check legacy storage for backward compatibility
        let project_path = req.project_path
            .as_ref()
            .map(PathBuf::from)
            .unwrap_or_else(|| std::env::current_dir().unwrap_or_default());

        let config = GatherConfig::default();
        let gatherer = ContextGatherer::new(project_path, config);
        let legacy_memories = gatherer.find_relevant_memories(&req.keywords);

        Ok(json!({
            "search_mode": "keyword",
            "keywords_searched": req.keywords,
            "total_found": memories.len(),
            "memories": memories,
            "legacy_memories_found": legacy_memories.len(),
            "wave_stats": manager.stats(),
            "tip": "Use use_resonance:true for semantic similarity search!",
        }))
    }
}

/// Request structure for get_collaboration_rapport
#[derive(Debug, Deserialize)]
pub struct GetRapportRequest {
    /// AI tool name (e.g., "claude", "cursor")
    pub ai_tool: String,
    /// Optional project path
    pub project_path: Option<String>,
}

/// Get rapport index for AI-human partnership
pub async fn get_collaboration_rapport(
    req: GetRapportRequest,
    _permission_check: impl Fn(serde_json::Value) -> Result<bool>,
) -> Result<Value> {
    let project_path = if let Some(path) = req.project_path {
        PathBuf::from(path)
    } else {
        std::env::current_dir()?
    };

    // Need to gather contexts first to build rapport
    let config = GatherConfig {
        search_dirs: vec![format!(".{}", req.ai_tool)],
        ..GatherConfig::default()
    };

    let mut gatherer = ContextGatherer::new(project_path.clone(), config);
    let _ = gatherer.gather_all(); // This will populate session tracker

    if let Some(rapport) = gatherer.session_tracker().get_rapport(&req.ai_tool) {
        Ok(json!({
            "ai_tool": req.ai_tool,
            "rapport": {
                "overall_score": format!("{:.2}", rapport.overall_score),
                "trust_level": format!("{:.2}", rapport.trust_level),
                "communication_efficiency": format!("{:.2}", rapport.communication_efficiency),
                "shared_vocabulary_size": rapport.shared_vocabulary_size,
                "inside_jokes_count": rapport.inside_jokes_count,
                "preferred_working_hours": rapport.preferred_working_hours,
                "avg_session_productivity": format!("{:.2}", rapport.avg_session_productivity),
                "trend": if rapport.evolution_trend > 0.05 {
                    "📈 Improving"
                } else if rapport.evolution_trend < -0.05 {
                    "📉 Declining"
                } else {
                    "➡️ Stable"
                },
            },
            "interpretation": interpret_rapport_score(rapport.overall_score),
        }))
    } else {
        Ok(json!({
            "ai_tool": req.ai_tool,
            "message": "No rapport data found yet. Start collaborating to build rapport!",
        }))
    }
}

/// Helper function to interpret rapport scores
fn interpret_rapport_score(score: f32) -> &'static str {
    match score {
        s if s >= 0.8 => "🌟 Excellent partnership! You work together seamlessly.",
        s if s >= 0.6 => "💪 Strong collaboration. Keep building on this foundation.",
        s if s >= 0.4 => "🌱 Growing partnership. Focus on clear communication.",
        s if s >= 0.2 => "🔧 Early stages. Take time to understand each other's style.",
        _ => "🤝 Just getting started. Every partnership begins somewhere!",
    }
}

/// Request structure for get_co_engagement_heatmap
#[derive(Debug, Deserialize)]
pub struct GetHeatmapRequest {
    /// Project path
    pub project_path: Option<String>,
    /// Output format: "visual" or "data"
    pub format: Option<String>,
}

/// Request structure for get_cross_domain_patterns
#[derive(Debug, Deserialize)]
pub struct GetPatternsRequest {
    /// Project path (default: current directory)
    pub project_path: Option<String>,
    /// Pattern type filter: \"algorithm\", \"architecture\", \"problem\", \"solution\", \"metaphor\", \"workflow\", \"collaboration\"
    pub pattern_type: Option<String>,
    /// Minimum pattern strength (0.0-1.0)
    pub min_strength: Option<f32>,
}

/// Get cross-domain patterns discovered across projects
pub async fn get_cross_domain_patterns(
    req: GetPatternsRequest,
    _permission_check: impl Fn(serde_json::Value) -> Result<bool>,
) -> Result<Value> {
    let project_path = if let Some(path) = req.project_path {
        PathBuf::from(path)
    } else {
        std::env::current_dir()?
    };

    let config = GatherConfig::default();
    let mut gatherer = ContextGatherer::new(project_path, config);
    let _ = gatherer.gather_all();

    let mut patterns = gatherer.get_cross_domain_patterns();

    // Filter by type if requested
    if let Some(pattern_type) = req.pattern_type {
        patterns.retain(|p| match pattern_type.as_str() {
            "algorithm" => matches!(
                p.pattern_type,
                crate::context_gatherer::cross_session::PatternType::Algorithm
            ),
            "architecture" => matches!(
                p.pattern_type,
                crate::context_gatherer::cross_session::PatternType::Architecture
            ),
            "problem" => matches!(
                p.pattern_type,
                crate::context_gatherer::cross_session::PatternType::Problem
            ),
            "solution" => matches!(
                p.pattern_type,
                crate::context_gatherer::cross_session::PatternType::Solution
            ),
            "metaphor" => matches!(
                p.pattern_type,
                crate::context_gatherer::cross_session::PatternType::Metaphor
            ),
            "workflow" => matches!(
                p.pattern_type,
                crate::context_gatherer::cross_session::PatternType::Workflow
            ),
            "collaboration" => matches!(
                p.pattern_type,
                crate::context_gatherer::cross_session::PatternType::Collaboration
            ),
            _ => true,
        });
    }

    // Filter by strength
    let min_strength = req.min_strength.unwrap_or(0.0);
    patterns.retain(|p| p.strength >= min_strength);

    Ok(json!({
        "total_patterns": patterns.len(),
        "patterns": patterns.iter().map(|p| json!({
            "id": p.pattern_id,
            "type": format!("{:?}", p.pattern_type),
            "description": p.description,
            "occurrences": p.occurrences.len(),
            "strength": format!("{:.2}", p.strength),
            "keywords": p.keywords,
            "first_seen": p.first_seen.format("%Y-%m-%d").to_string(),
            "last_seen": p.last_seen.format("%Y-%m-%d").to_string(),
            "projects": p.occurrences.iter()
                .map(|o| o.project_path.to_string_lossy().to_string())
                .collect::<std::collections::HashSet<_>>(),
        })).collect::<Vec<_>>(),
    }))
}

/// Request structure for suggest_cross_session_insights
#[derive(Debug, Deserialize)]
pub struct SuggestInsightsRequest {
    /// Keywords related to current work
    pub keywords: Vec<String>,
    /// Project path (default: current directory)
    pub project_path: Option<String>,
    /// Maximum insights to return
    pub max_results: Option<usize>,
}

/// Get relevant insights from other sessions that might help current work
pub async fn suggest_cross_session_insights(
    req: SuggestInsightsRequest,
    _permission_check: impl Fn(serde_json::Value) -> Result<bool>,
) -> Result<Value> {
    let project_path = if let Some(path) = req.project_path {
        PathBuf::from(path)
    } else {
        std::env::current_dir()?
    };

    let config = GatherConfig::default();
    let mut gatherer = ContextGatherer::new(project_path, config);
    let _ = gatherer.gather_all();

    let insights = gatherer.get_relevant_insights(&req.keywords);
    let max_results = req.max_results.unwrap_or(5);
    let insights_found = insights.len();
    let insights_empty = insights.is_empty();

    Ok(json!({
        "keywords": req.keywords,
        "insights_found": insights_found,
        "insights": insights.into_iter().take(max_results).map(|i| json!({
            "type": format!("{:?}", i.insight_type),
            "content": i.content,
            "source_sessions": i.source_sessions,
            "applicable_domains": i.applicable_domains,
            "confidence": format!("{:.2}", i.confidence),
        })).collect::<Vec<_>>(),
        "suggestion": if insights_empty {
            "No cross-session insights found yet. Keep collaborating to build connections!"
        } else {
            "Consider these insights from previous sessions that might help with your current work."
        },
    }))
}

/// Request structure for invite_persona
#[derive(Debug, Deserialize)]
pub struct InvitePersonaRequest {
    /// Context or problem description
    pub context: String,
    /// Suggested consultation duration in minutes
    pub duration_minutes: Option<u32>,
}

/// Invite a persona for temporary consultation based on context
pub async fn invite_persona(
    req: InvitePersonaRequest,
    _permission_check: impl Fn(serde_json::Value) -> Result<bool>,
) -> Result<Value> {
    let config = GatherConfig::default();
    let gatherer = ContextGatherer::new(std::env::current_dir()?, config);

    let duration = req.duration_minutes.unwrap_or(10);

    if let Some(invitation) = gatherer.invite_persona(&req.context, duration) {
        Ok(json!({
            "success": true,
            "persona": invitation.persona_name,
            "expertise": invitation.expertise_areas,
            "duration_minutes": invitation.suggested_duration_minutes,
            "context": invitation.invitation_context,
            "relevant_sessions": invitation.relevant_sessions,
            "message": format!(
                "🎭 {} is ready to help! They bring expertise in: {}",
                invitation.persona_name,
                invitation.expertise_areas.join(", ")
            ),
        }))
    } else {
        Ok(json!({
            "success": false,
            "message": "No specific persona matched your context. Continue with your current approach!",
            "available_personas": ["The Cheet (performance)", "Omni (wave patterns)", "Trish (organization)"],
        }))
    }
}

/// Get temporal co-engagement heatmap
pub async fn get_co_engagement_heatmap(
    req: GetHeatmapRequest,
    _permission_check: impl Fn(serde_json::Value) -> Result<bool>,
) -> Result<Value> {
    let project_path = if let Some(path) = req.project_path {
        PathBuf::from(path)
    } else {
        std::env::current_dir()?
    };

    let config = GatherConfig::default();
    let mut gatherer = ContextGatherer::new(project_path, config);
    let _ = gatherer.gather_all();

    let heatmap = gatherer.get_co_engagement_heatmap();
    let format = req.format.as_deref().unwrap_or("visual");

    if format == "visual" {
        // Create visual representation
        let mut visual = String::from("\n🕐 Temporal Co-Engagement Heatmap\n");
        visual.push_str("   Mon Tue Wed Thu Fri Sat Sun\n");

        for hour in 0..24 {
            visual.push_str(&format!("{:02} ", hour));
            for day in 0..7 {
                let intensity = heatmap.time_slots[hour][day];
                let block = match intensity {
                    i if i >= 0.8 => "🟥",
                    i if i >= 0.6 => "🟧",
                    i if i >= 0.4 => "🟨",
                    i if i >= 0.2 => "🟩",
                    i if i > 0.0 => "🟦",
                    _ => "⬜",
                };
                visual.push_str(&format!("{} ", block));
            }
            visual.push('\n');
        }

        visual.push_str(&format!(
            "\n📊 Collaboration Density: {:.1}%\n",
            heatmap.collaboration_density * 100.0
        ));
        visual.push_str(&format!(
            "🎯 Peak Zones: {} identified\n",
            heatmap.peak_collaboration_zones.len()
        ));

        Ok(json!({
            "heatmap": visual,
            "peak_times": heatmap.peak_collaboration_zones.iter()
                .map(|(h, d)| format!("{} {:02}:00",
                    ["Mon", "Tue", "Wed", "Thu", "Fri", "Sat", "Sun"][*d],
                    h
                ))
                .collect::<Vec<_>>(),
        }))
    } else {
        // Return raw data
        Ok(serde_json::to_value(heatmap)?)
    }
}

/// Analyze AI tool usage patterns
pub async fn analyze_ai_tool_usage(
    req: AnalyzeAiToolUsageRequest,
    _permission_check: impl Fn(serde_json::Value) -> Result<bool>,
) -> Result<Value> {
    let home_dir = dirs::home_dir().context("Failed to get home directory")?;

    let mut usage_stats = json!({
        "analysis_date": chrono::Utc::now().to_rfc3339(),
        "days_analyzed": req.days.unwrap_or(30),
        "tools": {}
    });

    let tools_to_check = if let Some(tool) = req.tool_name {
        vec![tool]
    } else {
        crate::context_gatherer::AI_TOOL_DIRS
            .iter()
            .map(|s| s.to_string())
            .collect()
    };

    let cutoff_date = chrono::Utc::now() - chrono::Duration::days(req.days.unwrap_or(30) as i64);

    for tool in tools_to_check {
        let tool_path = home_dir.join(&tool);
        if !tool_path.exists() {
            continue;
        }

        let mut stats = json!({
            "exists": true,
            "total_files": 0,
            "total_size_bytes": 0,
            "recent_files": 0,
            "file_types": {}
        });

        // Analyze tool directory
        for entry in walkdir::WalkDir::new(&tool_path)
            .max_depth(5)
            .into_iter()
            .filter_map(|e| e.ok())
        {
            if entry.file_type().is_file() {
                if let Ok(metadata) = entry.metadata() {
                    stats["total_files"] = json!(stats["total_files"].as_u64().unwrap_or(0) + 1);
                    stats["total_size_bytes"] =
                        json!(stats["total_size_bytes"].as_u64().unwrap_or(0) + metadata.len());

                    // Check if recent
                    if let Ok(modified) = metadata.modified() {
                        let modified_time = chrono::DateTime::<chrono::Utc>::from(modified);
                        if modified_time > cutoff_date {
                            stats["recent_files"] =
                                json!(stats["recent_files"].as_u64().unwrap_or(0) + 1);
                        }
                    }

                    // Track file types
                    if let Some(ext) = entry.path().extension().and_then(|e| e.to_str()) {
                        let types = stats["file_types"].as_object_mut().unwrap();
                        let count = types.get(ext).and_then(|v| v.as_u64()).unwrap_or(0);
                        types.insert(ext.to_string(), json!(count + 1));
                    }
                }
            }
        }

        usage_stats["tools"][tool] = stats;
    }

    Ok(usage_stats)
}

/// Request for cleaning old context files
#[derive(Debug, Deserialize)]
pub struct CleanOldContextRequest {
    /// Days to keep (default: 90)
    pub days_to_keep: Option<u32>,
    /// Dry run - don't actually delete (default: true for safety!)
    pub dry_run: Option<bool>,
    /// Specific tools to clean (defaults to all: .claude, .windsurf, .cursor, etc.)
    pub tools: Option<Vec<String>>,
    /// Minimum file size in bytes to consider (skip tiny config files)
    pub min_file_size: Option<u64>,
}

/// Clean old context files from AI tools - ACTUALLY WORKS NOW! 🧹
///
/// This function walks through AI tool directories and removes old context files
/// to reclaim disk space and keep things tidy. Safety first - dry_run defaults to true!
pub async fn clean_old_context(
    req: CleanOldContextRequest,
    permission_check: impl Fn(serde_json::Value) -> Result<bool>,
) -> Result<Value> {
    use std::fs;
    use std::time::{Duration, SystemTime};
    use walkdir::WalkDir;

    // Extract params with sane defaults - Trish says: "Always be safe!"
    let days_to_keep = req.days_to_keep.unwrap_or(90);
    let dry_run = req.dry_run.unwrap_or(true); // Default to DRY RUN for safety!
    let min_file_size = req.min_file_size.unwrap_or(1024); // Skip files under 1KB

    // Which AI tool dirs to clean
    let tools_to_clean: Vec<String> = req.tools.unwrap_or_else(|| {
        crate::context_gatherer::AI_TOOL_DIRS
            .iter()
            .map(|s| s.to_string())
            .collect()
    });

    // Get home directory
    let home = match dirs::home_dir() {
        Some(h) => h,
        None => return Err(anyhow::anyhow!("Could not find home directory")),
    };

    // Check permission for this dangerous operation
    let perm_result = permission_check(json!({
        "operation": "clean_old_context",
        "dry_run": dry_run,
        "days_to_keep": days_to_keep,
        "tools": &tools_to_clean,
    }))?;

    if !perm_result {
        return Err(anyhow::anyhow!("Permission denied for context cleaning"));
    }

    // Calculate the cutoff time - files older than this get cleaned
    let cutoff_duration = Duration::from_secs(days_to_keep as u64 * 24 * 60 * 60);
    let now = SystemTime::now();

    // Track what we find/clean
    let mut files_found: Vec<serde_json::Value> = Vec::new();
    let mut total_size_bytes: u64 = 0;
    let mut files_deleted: u32 = 0;
    let mut files_failed: u32 = 0;
    let mut dirs_scanned: u32 = 0;

    // Context file extensions we care about (from context_gatherer)
    let context_extensions: std::collections::HashSet<&str> =
        crate::context_gatherer::CONTEXT_EXTENSIONS.iter().copied().collect();

    // Walk each AI tool directory
    for tool_dir in &tools_to_clean {
        let search_path = home.join(tool_dir);
        if !search_path.exists() {
            continue;
        }

        dirs_scanned += 1;

        // Walk with depth limit to avoid going too deep
        for entry in WalkDir::new(&search_path)
            .max_depth(5)
            .into_iter()
            .filter_map(|e| e.ok())
        {
            let path = entry.path();

            // Skip directories
            if !path.is_file() {
                continue;
            }

            // Check extension - only clean context files
            let ext = path.extension()
                .and_then(|e| e.to_str())
                .unwrap_or("");

            if !context_extensions.contains(ext) {
                continue;
            }

            // Get file metadata
            let metadata = match fs::metadata(path) {
                Ok(m) => m,
                Err(_) => continue,
            };

            let file_size = metadata.len();

            // Skip tiny files (likely important configs)
            if file_size < min_file_size {
                continue;
            }

            // Check modification time
            let modified = match metadata.modified() {
                Ok(m) => m,
                Err(_) => continue,
            };

            // Is this file old enough to clean?
            let age = match now.duration_since(modified) {
                Ok(d) => d,
                Err(_) => continue, // Future date? Skip it.
            };

            if age < cutoff_duration {
                continue; // File is too new, keep it
            }

            // This file is a candidate for cleaning!
            let age_days = age.as_secs() / (24 * 60 * 60);

            let file_info = json!({
                "path": path.to_string_lossy(),
                "tool": tool_dir,
                "size_bytes": file_size,
                "size_human": format_size(file_size),
                "age_days": age_days,
            });

            if dry_run {
                // Just report what WOULD be deleted
                files_found.push(file_info);
                total_size_bytes += file_size;
            } else {
                // Actually delete the file
                match fs::remove_file(path) {
                    Ok(_) => {
                        files_deleted += 1;
                        total_size_bytes += file_size;
                        files_found.push(file_info);
                    }
                    Err(e) => {
                        files_failed += 1;
                        files_found.push(json!({
                            "path": path.to_string_lossy(),
                            "error": e.to_string(),
                        }));
                    }
                }
            }
        }
    }

    // Build response - nice and informative for Trish!
    Ok(json!({
        "success": true,
        "dry_run": dry_run,
        "summary": {
            "dirs_scanned": dirs_scanned,
            "files_found": files_found.len(),
            "files_deleted": files_deleted,
            "files_failed": files_failed,
            "total_size_bytes": total_size_bytes,
            "total_size_human": format_size(total_size_bytes),
            "days_threshold": days_to_keep,
        },
        "action": if dry_run {
            format!("Would delete {} files ({}) - set dry_run:false to actually clean",
                files_found.len(), format_size(total_size_bytes))
        } else {
            format!("Deleted {} files, reclaimed {}",
                files_deleted, format_size(total_size_bytes))
        },
        "files": files_found,
        "hint": if dry_run {
            "Set dry_run:false to actually delete these files"
        } else {
            "✨ All clean! Your AI tools thank you."
        },
    }))
}

/// Format file size in human-readable format (helper for clean_old_context)
fn format_size(bytes: u64) -> String {
    const KB: u64 = 1024;
    const MB: u64 = KB * 1024;
    const GB: u64 = MB * 1024;

    if bytes >= GB {
        format!("{:.2} GB", bytes as f64 / GB as f64)
    } else if bytes >= MB {
        format!("{:.2} MB", bytes as f64 / MB as f64)
    } else if bytes >= KB {
        format!("{:.2} KB", bytes as f64 / KB as f64)
    } else {
        format!("{} bytes", bytes)
    }
}
