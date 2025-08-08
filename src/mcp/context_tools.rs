//! MCP tools for context gathering from AI tool directories

use anyhow::{Result, Context as _};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::path::PathBuf;
use crate::context_gatherer::{ContextGatherer, GatherConfig, GatheredContext};

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
        config.custom_dirs = custom.into_iter()
            .map(PathBuf::from)
            .collect();
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
    let filtered_contexts: Vec<&GatheredContext> = all_contexts.iter()
        .filter(|c| c.relevance_score >= min_relevance)
        .take(max_results)
        .collect();
    
    // Generate response based on format
    let output_format = req.output_format.as_deref().unwrap_or("summary");
    
    match output_format {
        "m8" => {
            let m8_data = gatherer.to_m8()?;
            let encoded = base64::Engine::encode(&base64::engine::general_purpose::STANDARD, &m8_data);
            
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
            let rapport_summary: Vec<_> = gatherer.session_tracker().rapport_indices.iter()
                .map(|(pair, rapport)| json!({
                    "pair": pair,
                    "score": format!("{:.2}", rapport.overall_score),
                    "trend": if rapport.evolution_trend > 0.05 {
                        "📈"
                    } else if rapport.evolution_trend < -0.05 {
                        "📉"
                    } else {
                        "➡️"
                    },
                }))
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
            let summaries: Vec<ContextSummary> = filtered_contexts.iter()
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
fn calculate_sources_summary(contexts: &[&GatheredContext]) -> std::collections::HashMap<String, usize> {
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
    pub include_paths: Option<bool>,
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
    /// Origin: "human", "ai:<tool>", or "tandem:<human>:<ai>"
    pub origin: String,
    /// Project path to associate with
    pub project_path: Option<String>,
}

/// Anchor a collaborative memory for future retrieval
pub async fn anchor_collaborative_memory(
    req: AnchorMemoryRequest,
    _permission_check: impl Fn(serde_json::Value) -> Result<bool>,
) -> Result<Value> {
    let project_path = if let Some(path) = req.project_path {
        PathBuf::from(path)
    } else {
        std::env::current_dir()?
    };
    
    // Parse origin
    let origin = if req.origin.starts_with("tandem:") {
        let parts: Vec<&str> = req.origin.split(':').collect();
        if parts.len() >= 3 {
            crate::context_gatherer::collab_session::CollaborativeOrigin::Tandem {
                human: parts[1].to_string(),
                ai: parts[2].to_string(),
            }
        } else {
            return Ok(json!({
                "error": "Invalid tandem origin format. Use: tandem:<human>:<ai>"
            }));
        }
    } else if req.origin.starts_with("ai:") {
        let ai_name = req.origin.strip_prefix("ai:").unwrap_or("claude");
        crate::context_gatherer::collab_session::CollaborativeOrigin::Single(ai_name.to_string())
    } else if req.origin == "human" {
        crate::context_gatherer::collab_session::CollaborativeOrigin::Single("human".to_string())
    } else {
        crate::context_gatherer::collab_session::CollaborativeOrigin::Emergent
    };
    
    // Parse anchor type
    let anchor_type = match req.anchor_type.as_str() {
        "pattern_insight" | "pattern" => crate::context_gatherer::collab_session::AnchorType::PatternInsight,
        "solution" => crate::context_gatherer::collab_session::AnchorType::Solution,
        "breakthrough" => crate::context_gatherer::collab_session::AnchorType::Breakthrough,
        "learning" | "learning_moment" => crate::context_gatherer::collab_session::AnchorType::LearningMoment,
        "joke" | "shared_joke" => crate::context_gatherer::collab_session::AnchorType::SharedJoke,
        "technical" | "technical_pattern" => crate::context_gatherer::collab_session::AnchorType::TechnicalPattern,
        "process" | "process_improvement" => crate::context_gatherer::collab_session::AnchorType::ProcessImprovement,
        _ => crate::context_gatherer::collab_session::AnchorType::PatternInsight,
    };
    
    // Create a minimal gatherer just for anchoring
    let config = GatherConfig::default();
    let mut gatherer = ContextGatherer::new(project_path.clone(), config);
    
    // Anchor the memory
    match gatherer.anchor_memory(origin, anchor_type, req.context.clone(), req.keywords.clone()) {
        Ok(anchor_id) => {
            Ok(json!({
                "success": true,
                "anchor_id": anchor_id,
                "message": format!("Memory anchored successfully with {} keywords", req.keywords.len()),
                "retrieval_hint": "Use find_collaborative_memories to retrieve this later",
            }))
        }
        Err(e) => {
            Ok(json!({
                "error": format!("Failed to anchor memory: {}", e)
            }))
        }
    }
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
}

/// Find previously anchored collaborative memories
pub async fn find_collaborative_memories(
    req: FindMemoriesRequest,
    _permission_check: impl Fn(serde_json::Value) -> Result<bool>,
) -> Result<Value> {
    let project_path = if let Some(path) = req.project_path {
        PathBuf::from(path)
    } else {
        std::env::current_dir()?
    };
    
    let config = GatherConfig::default();
    let gatherer = ContextGatherer::new(project_path, config);
    
    let memories = gatherer.find_relevant_memories(&req.keywords);
    let max_results = req.max_results.unwrap_or(10);
    
    Ok(json!({
        "keywords_searched": req.keywords,
        "total_found": memories.len(),
        "memories": memories.into_iter().take(max_results).collect::<Vec<_>>(),
    }))
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
    let mut config = GatherConfig::default();
    config.search_dirs = vec![format!(".{}", req.ai_tool)];
    
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
            "algorithm" => matches!(p.pattern_type, crate::context_gatherer::cross_session::PatternType::Algorithm),
            "architecture" => matches!(p.pattern_type, crate::context_gatherer::cross_session::PatternType::Architecture),
            "problem" => matches!(p.pattern_type, crate::context_gatherer::cross_session::PatternType::Problem),
            "solution" => matches!(p.pattern_type, crate::context_gatherer::cross_session::PatternType::Solution),
            "metaphor" => matches!(p.pattern_type, crate::context_gatherer::cross_session::PatternType::Metaphor),
            "workflow" => matches!(p.pattern_type, crate::context_gatherer::cross_session::PatternType::Workflow),
            "collaboration" => matches!(p.pattern_type, crate::context_gatherer::cross_session::PatternType::Collaboration),
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
        
        visual.push_str(&format!("\n📊 Collaboration Density: {:.1}%\n", heatmap.collaboration_density * 100.0));
        visual.push_str(&format!("🎯 Peak Zones: {} identified\n", heatmap.peak_collaboration_zones.len()));
        
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
    let home_dir = dirs::home_dir()
        .context("Failed to get home directory")?;
    
    let mut usage_stats = json!({
        "analysis_date": chrono::Utc::now().to_rfc3339(),
        "days_analyzed": req.days.unwrap_or(30),
        "tools": {}
    });
    
    let tools_to_check = if let Some(tool) = req.tool_name {
        vec![tool]
    } else {
        crate::context_gatherer::AI_TOOL_DIRS.iter()
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
                    stats["total_size_bytes"] = json!(
                        stats["total_size_bytes"].as_u64().unwrap_or(0) + metadata.len()
                    );
                    
                    // Check if recent
                    if let Ok(modified) = metadata.modified() {
                        let modified_time = chrono::DateTime::<chrono::Utc>::from(modified);
                        if modified_time > cutoff_date {
                            stats["recent_files"] = json!(stats["recent_files"].as_u64().unwrap_or(0) + 1);
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
    /// Dry run - don't actually delete
    pub dry_run: Option<bool>,
    /// Specific tools to clean
    pub tools: Option<Vec<String>>,
}

/// Clean old context files from AI tools
pub async fn clean_old_context(
    _req: CleanOldContextRequest,
    _permission_check: impl Fn(serde_json::Value) -> Result<bool>,
) -> Result<Value> {
    // This would implement cleaning logic similar to the hoarder intervention
    // For now, return a placeholder
    Ok(json!({
        "message": "Context cleaning not yet implemented",
        "hint": "Use gather_project_context with privacy_mode enabled for now"
    }))
}