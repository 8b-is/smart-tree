//! AI-Human Partnership Analysis
//! 
//! This module analyzes the quality and patterns of AI-human collaborations,
//! helping both parties understand and improve their working relationship.

use std::collections::HashMap;
use chrono::{DateTime, Utc, Duration, Timelike};
use serde::{Deserialize, Serialize};

use super::{GatheredContext, ContextType, ContextContent};

/// Partnership analyzer for AI-human collaboration
pub struct PartnershipAnalyzer {
    contexts: Vec<GatheredContext>,
}

impl PartnershipAnalyzer {
    pub fn new(contexts: Vec<GatheredContext>) -> Self {
        Self { contexts }
    }
    
    /// Analyze the overall partnership health
    pub fn analyze_partnership(&self) -> PartnershipAnalysis {
        let sessions = self.detect_collaborative_sessions();
        let conversation_flows = self.analyze_conversation_flows();
        let interaction_patterns = self.analyze_interaction_patterns();
        let collaboration_metrics = self.calculate_collaboration_metrics(&sessions);
        let relationship_evolution = self.analyze_relationship_evolution();
        let shared_understanding = self.measure_shared_understanding();
        
        PartnershipAnalysis {
            total_interactions: self.contexts.len(),
            collaborative_sessions: sessions,
            conversation_flows,
            interaction_patterns,
            collaboration_metrics,
            relationship_evolution,
            shared_understanding,
            partnership_health: self.calculate_partnership_health(&collaboration_metrics),
            recommendations: self.generate_recommendations(&collaboration_metrics),
        }
    }
    
    /// Detect collaborative sessions where human and AI work together
    fn detect_collaborative_sessions(&self) -> Vec<CollaborativeSession> {
        let mut sessions = Vec::new();
        let mut current_session: Option<CollaborativeSession> = None;
        
        // Group contexts by time proximity (within 30 minutes)
        let session_gap = Duration::minutes(30);
        
        let mut sorted_contexts = self.contexts.clone();
        sorted_contexts.sort_by_key(|c| c.timestamp);
        
        for context in sorted_contexts {
            if let Some(ref mut session) = current_session {
                let gap = context.timestamp - session.end_time;
                
                if gap > session_gap || context.ai_tool != session.primary_tool {
                    // End current session
                    sessions.push(session.clone());
                    current_session = Some(CollaborativeSession::new(context));
                } else {
                    // Continue session
                    session.update(context);
                }
            } else {
                // Start first session
                current_session = Some(CollaborativeSession::new(context));
            }
        }
        
        if let Some(session) = current_session {
            sessions.push(session);
        }
        
        sessions
    }
    
    /// Analyze conversation flow patterns
    fn analyze_conversation_flows(&self) -> Vec<ConversationFlow> {
        let mut flows = Vec::new();
        
        // Extract chat histories
        let chat_contexts: Vec<_> = self.contexts.iter()
            .filter(|c| matches!(c.content_type, ContextType::ChatHistory))
            .collect();
        
        for context in chat_contexts {
            if let ContextContent::Json(json) = &context.content {
                if let Some(messages) = json.get("messages").and_then(|m| m.as_array()) {
                    let flow = self.analyze_message_flow(messages, &context.ai_tool);
                    flows.push(flow);
                }
            }
        }
        
        flows
    }
    
    /// Analyze a single message flow
    fn analyze_message_flow(&self, messages: &[serde_json::Value], tool: &str) -> ConversationFlow {
        let mut turn_count = 0;
        let mut question_count = 0;
        let mut clarification_count = 0;
        let mut completion_count = 0;
        let mut context_switches = 0;
        let mut last_topic = String::new();
        
        for (i, msg) in messages.iter().enumerate() {
            if let Some(content) = msg.get("content").and_then(|c| c.as_str()) {
                turn_count += 1;
                
                // Detect questions
                if content.contains('?') {
                    question_count += 1;
                }
                
                // Detect clarifications
                if content.contains("clarify") || content.contains("what do you mean") || 
                   content.contains("could you explain") {
                    clarification_count += 1;
                }
                
                // Detect completions
                if content.contains("done") || content.contains("complete") || 
                   content.contains("finished") || content.contains("works") {
                    completion_count += 1;
                }
                
                // Detect topic switches (simplified)
                let current_topic = self.extract_topic(content);
                if i > 0 && current_topic != last_topic {
                    context_switches += 1;
                }
                last_topic = current_topic;
            }
        }
        
        ConversationFlow {
            tool: tool.to_string(),
            turn_count,
            question_count,
            clarification_count,
            completion_count,
            context_switches,
            flow_smoothness: self.calculate_flow_smoothness(
                turn_count, 
                clarification_count, 
                context_switches
            ),
        }
    }
    
    /// Calculate how smooth the conversation flow is
    fn calculate_flow_smoothness(&self, turns: usize, clarifications: usize, switches: usize) -> f32 {
        if turns == 0 {
            return 0.0;
        }
        
        let disruptions = clarifications + switches;
        let smoothness = 1.0 - (disruptions as f32 / turns as f32).min(1.0);
        smoothness
    }
    
    /// Extract topic from content (simplified)
    fn extract_topic(&self, content: &str) -> String {
        // Simple keyword-based topic extraction
        let keywords = ["function", "file", "error", "bug", "feature", "test", "deploy", "design"];
        
        for keyword in keywords {
            if content.to_lowercase().contains(keyword) {
                return keyword.to_string();
            }
        }
        
        "general".to_string()
    }
    
    /// Analyze interaction patterns
    fn analyze_interaction_patterns(&self) -> InteractionPatterns {
        let mut tool_preferences = HashMap::new();
        let mut time_of_day_activity = vec![0; 24]; // 24 hours
        let mut response_patterns = ResponsePatterns::default();
        
        for context in &self.contexts {
            // Track tool preferences
            *tool_preferences.entry(context.ai_tool.clone()).or_insert(0) += 1;
            
            // Track time of day
            let hour = context.timestamp.hour() as usize;
            time_of_day_activity[hour] += 1;
            
            // Analyze response patterns (simplified)
            if let ContextContent::Json(json) = &context.content {
                if let Some(messages) = json.get("messages").and_then(|m| m.as_array()) {
                    self.analyze_response_patterns(messages, &mut response_patterns);
                }
            }
        }
        
        // Find peak hours
        let peak_hours = self.find_peak_hours(&time_of_day_activity);
        
        InteractionPatterns {
            tool_preferences,
            peak_collaboration_hours: peak_hours,
            average_session_length: self.calculate_average_session_length(),
            response_patterns,
        }
    }
    
    /// Analyze response patterns in conversations
    fn analyze_response_patterns(&self, messages: &[serde_json::Value], patterns: &mut ResponsePatterns) {
        let mut last_was_human = false;
        let mut response_times: Vec<f32> = Vec::new();
        
        for (i, msg) in messages.iter().enumerate() {
            if let Some(role) = msg.get("role").and_then(|r| r.as_str()) {
                if role == "user" {
                    last_was_human = true;
                } else if role == "assistant" && last_was_human {
                    // AI response to human
                    if let Some(content) = msg.get("content").and_then(|c| c.as_str()) {
                        // Measure response characteristics
                        if content.len() > 500 {
                            patterns.detailed_responses += 1;
                        } else if content.len() < 100 {
                            patterns.brief_responses += 1;
                        }
                        
                        if content.contains("```") {
                            patterns.code_heavy_responses += 1;
                        }
                        
                        if content.contains("Let me") || content.contains("I'll") {
                            patterns.proactive_responses += 1;
                        }
                    }
                    last_was_human = false;
                }
            }
        }
        
        patterns.total_responses += messages.len() / 2; // Rough estimate
    }
    
    /// Find peak collaboration hours
    fn find_peak_hours(&self, activity: &[usize]) -> Vec<usize> {
        let mut hours_with_activity: Vec<(usize, usize)> = activity.iter()
            .enumerate()
            .map(|(hour, &count)| (hour, count))
            .filter(|(_, count)| *count > 0)
            .collect();
        
        hours_with_activity.sort_by_key(|(_, count)| std::cmp::Reverse(*count));
        
        hours_with_activity.into_iter()
            .take(3)
            .map(|(hour, _)| hour)
            .collect()
    }
    
    /// Calculate average session length
    fn calculate_average_session_length(&self) -> Duration {
        let sessions = self.detect_collaborative_sessions();
        if sessions.is_empty() {
            return Duration::zero();
        }
        
        let total_duration: Duration = sessions.iter()
            .map(|s| s.end_time - s.start_time)
            .sum();
        
        let avg_seconds = total_duration.num_seconds() / sessions.len() as i64;
        Duration::seconds(avg_seconds)
    }
    
    /// Calculate collaboration metrics
    fn calculate_collaboration_metrics(&self, sessions: &[CollaborativeSession]) -> CollaborationMetrics {
        let total_sessions = sessions.len();
        let mut productive_sessions = 0;
        let mut stuck_sessions = 0;
        let mut learning_sessions = 0;
        
        for session in sessions {
            if session.outcomes_achieved > 0 {
                productive_sessions += 1;
            }
            if session.clarifications_needed > 2 {
                stuck_sessions += 1;
            }
            if session.new_concepts_introduced > 0 {
                learning_sessions += 1;
            }
        }
        
        CollaborationMetrics {
            productivity_rate: if total_sessions > 0 { 
                productive_sessions as f32 / total_sessions as f32 
            } else { 0.0 },
            learning_rate: if total_sessions > 0 { 
                learning_sessions as f32 / total_sessions as f32 
            } else { 0.0 },
            stuck_rate: if total_sessions > 0 { 
                stuck_sessions as f32 / total_sessions as f32 
            } else { 0.0 },
            collaboration_depth: self.calculate_collaboration_depth(sessions),
            mutual_understanding: self.calculate_mutual_understanding(sessions),
        }
    }
    
    /// Calculate depth of collaboration
    fn calculate_collaboration_depth(&self, sessions: &[CollaborativeSession]) -> f32 {
        if sessions.is_empty() {
            return 0.0;
        }
        
        let avg_interactions = sessions.iter()
            .map(|s| s.interaction_count)
            .sum::<usize>() as f32 / sessions.len() as f32;
        
        // Normalize to 0-1 scale (assuming 20+ interactions is deep)
        (avg_interactions / 20.0).min(1.0)
    }
    
    /// Calculate mutual understanding score
    fn calculate_mutual_understanding(&self, sessions: &[CollaborativeSession]) -> f32 {
        if sessions.is_empty() {
            return 0.0;
        }
        
        let understanding_factors = sessions.iter()
            .map(|s| {
                let clarity = 1.0 - (s.clarifications_needed as f32 / s.interaction_count.max(1) as f32);
                let efficiency = s.outcomes_achieved as f32 / s.interaction_count.max(1) as f32;
                (clarity + efficiency) / 2.0
            })
            .sum::<f32>();
        
        understanding_factors / sessions.len() as f32
    }
    
    /// Analyze how the relationship has evolved
    fn analyze_relationship_evolution(&self) -> RelationshipEvolution {
        let sessions = self.detect_collaborative_sessions();
        if sessions.len() < 2 {
            return RelationshipEvolution::default();
        }
        
        // Split sessions into early and recent
        let mid_point = sessions.len() / 2;
        let early_sessions = &sessions[..mid_point];
        let recent_sessions = &sessions[mid_point..];
        
        let early_metrics = self.calculate_collaboration_metrics(early_sessions);
        let recent_metrics = self.calculate_collaboration_metrics(recent_sessions);
        
        RelationshipEvolution {
            productivity_trend: recent_metrics.productivity_rate - early_metrics.productivity_rate,
            understanding_trend: recent_metrics.mutual_understanding - early_metrics.mutual_understanding,
            complexity_trend: recent_sessions.iter()
                .map(|s| s.new_concepts_introduced)
                .sum::<usize>() as f32 / 
                early_sessions.iter()
                .map(|s| s.new_concepts_introduced)
                .sum::<usize>().max(1) as f32,
            trust_indicators: self.calculate_trust_indicators(&sessions),
        }
    }
    
    /// Calculate trust indicators
    fn calculate_trust_indicators(&self, sessions: &[CollaborativeSession]) -> TrustIndicators {
        let mut autonomy_given = 0;
        let mut corrections_accepted = 0;
        let mut suggestions_followed = 0;
        
        // Simplified trust calculation based on session patterns
        for session in sessions {
            if session.interaction_count > 5 && session.clarifications_needed < 2 {
                autonomy_given += 1;
            }
            if session.outcomes_achieved > 0 {
                suggestions_followed += 1;
            }
        }
        
        TrustIndicators {
            autonomy_level: autonomy_given as f32 / sessions.len().max(1) as f32,
            correction_acceptance: 0.8, // Placeholder - would need more detailed analysis
            suggestion_adoption_rate: suggestions_followed as f32 / sessions.len().max(1) as f32,
        }
    }
    
    /// Measure shared understanding between AI and human
    fn measure_shared_understanding(&self) -> SharedUnderstanding {
        let mut shared_vocabulary = HashMap::new();
        let mut concept_alignment = 0.0;
        let mut communication_efficiency = 0.0;
        
        // Analyze vocabulary usage
        for context in &self.contexts {
            if let ContextContent::Text(text) = &context.content {
                // Extract technical terms (simplified)
                for word in text.split_whitespace() {
                    if word.len() > 5 && !word.chars().all(|c| c.is_lowercase()) {
                        *shared_vocabulary.entry(word.to_string()).or_insert(0) += 1;
                    }
                }
            }
        }
        
        // Calculate efficiency based on decreasing clarifications over time
        let sessions = self.detect_collaborative_sessions();
        if sessions.len() > 1 {
            let early_clarifications = sessions[..sessions.len()/2].iter()
                .map(|s| s.clarifications_needed)
                .sum::<usize>() as f32;
            let recent_clarifications = sessions[sessions.len()/2..].iter()
                .map(|s| s.clarifications_needed)
                .sum::<usize>() as f32;
            
            communication_efficiency = 1.0 - (recent_clarifications / early_clarifications.max(1.0)).min(1.0);
        }
        
        SharedUnderstanding {
            vocabulary_size: shared_vocabulary.len(),
            concept_alignment: 0.75, // Placeholder
            communication_efficiency,
            domain_expertise_areas: self.identify_expertise_areas(&shared_vocabulary),
        }
    }
    
    /// Identify areas of shared expertise
    fn identify_expertise_areas(&self, vocabulary: &HashMap<String, usize>) -> Vec<String> {
        // Simple domain identification based on vocabulary
        let domains = vec![
            ("rust", vec!["impl", "trait", "async", "tokio", "cargo"]),
            ("javascript", vec!["const", "async", "await", "npm", "react"]),
            ("python", vec!["def", "import", "pip", "django", "flask"]),
            ("ai", vec!["model", "training", "neural", "embedding", "transformer"]),
            ("database", vec!["query", "table", "index", "postgres", "sqlite"]),
        ];
        
        let mut identified_domains = Vec::new();
        
        for (domain, keywords) in domains {
            let matches = keywords.iter()
                .filter(|k| vocabulary.contains_key(&k.to_string()))
                .count();
            
            if matches >= 2 {
                identified_domains.push(domain.to_string());
            }
        }
        
        identified_domains
    }
    
    /// Calculate overall partnership health
    fn calculate_partnership_health(&self, metrics: &CollaborationMetrics) -> PartnershipHealth {
        let score = (metrics.productivity_rate * 0.3 + 
                    metrics.learning_rate * 0.2 + 
                    (1.0 - metrics.stuck_rate) * 0.2 +
                    metrics.collaboration_depth * 0.15 +
                    metrics.mutual_understanding * 0.15).min(1.0);
        
        let status = match score {
            s if s >= 0.8 => "Thriving",
            s if s >= 0.6 => "Healthy",
            s if s >= 0.4 => "Developing",
            _ => "Needs Attention",
        };
        
        PartnershipHealth {
            overall_score: score,
            status: status.to_string(),
            strengths: self.identify_strengths(metrics),
            areas_for_improvement: self.identify_improvements(metrics),
        }
    }
    
    /// Identify partnership strengths
    fn identify_strengths(&self, metrics: &CollaborationMetrics) -> Vec<String> {
        let mut strengths = Vec::new();
        
        if metrics.productivity_rate > 0.7 {
            strengths.push("High productivity in achieving goals".to_string());
        }
        if metrics.learning_rate > 0.5 {
            strengths.push("Continuous learning and growth".to_string());
        }
        if metrics.mutual_understanding > 0.7 {
            strengths.push("Strong mutual understanding".to_string());
        }
        if metrics.collaboration_depth > 0.6 {
            strengths.push("Deep, meaningful collaborations".to_string());
        }
        
        strengths
    }
    
    /// Identify areas for improvement
    fn identify_improvements(&self, metrics: &CollaborationMetrics) -> Vec<String> {
        let mut improvements = Vec::new();
        
        if metrics.stuck_rate > 0.3 {
            improvements.push("Reduce getting stuck - try breaking down complex tasks".to_string());
        }
        if metrics.productivity_rate < 0.5 {
            improvements.push("Focus on completing more outcomes per session".to_string());
        }
        if metrics.mutual_understanding < 0.5 {
            improvements.push("Improve clarity in communication".to_string());
        }
        
        improvements
    }
    
    /// Generate recommendations for improving the partnership
    fn generate_recommendations(&self, metrics: &CollaborationMetrics) -> Vec<String> {
        let mut recommendations = Vec::new();
        
        // Base recommendations on metrics
        if metrics.stuck_rate > 0.2 {
            recommendations.push(
                "💡 When stuck, try: 1) Break down the problem, 2) Provide more context, 3) Ask for alternative approaches".to_string()
            );
        }
        
        if metrics.learning_rate < 0.3 {
            recommendations.push(
                "📚 Explore new areas together - try asking about unfamiliar technologies or concepts".to_string()
            );
        }
        
        if metrics.collaboration_depth < 0.5 {
            recommendations.push(
                "🤝 Deepen collaboration by working on longer-term projects together".to_string()
            );
        }
        
        if metrics.mutual_understanding < 0.6 {
            recommendations.push(
                "🗣️ Improve understanding by being more specific about requirements and constraints".to_string()
            );
        }
        
        // Always add one positive reinforcement
        recommendations.push(
            "🌟 Keep building on your collaborative strengths!".to_string()
        );
        
        recommendations
    }
}

/// A collaborative session between human and AI
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CollaborativeSession {
    pub start_time: DateTime<Utc>,
    pub end_time: DateTime<Utc>,
    pub primary_tool: String,
    pub interaction_count: usize,
    pub clarifications_needed: usize,
    pub outcomes_achieved: usize,
    pub new_concepts_introduced: usize,
    pub session_mood: SessionMood,
}

impl CollaborativeSession {
    fn new(context: GatheredContext) -> Self {
        let mut session = Self {
            start_time: context.timestamp,
            end_time: context.timestamp,
            primary_tool: context.ai_tool.clone(),
            interaction_count: 0,
            clarifications_needed: 0,
            outcomes_achieved: 0,
            new_concepts_introduced: 0,
            session_mood: SessionMood::Neutral,
        };
        
        // Analyze the first context
        session.update(context);
        session
    }
    
    fn update(&mut self, context: GatheredContext) {
        self.end_time = context.timestamp;
        self.interaction_count += 1;
        
        // Analyze content to update metrics
        if let ContextContent::Json(json) = &context.content {
            if let Some(messages) = json.get("messages").and_then(|m| m.as_array()) {
                for msg in messages {
                    if let Some(content) = msg.get("content").and_then(|c| c.as_str()) {
                        // Check for completion indicators
                        if content.contains("done") || content.contains("complete") || 
                           content.contains("finished") || content.contains("works") ||
                           content.contains("successfully") {
                            self.outcomes_achieved += 1;
                        }
                        
                        // Check for clarifications
                        if content.contains("?") || content.contains("clarify") || 
                           content.contains("what do you mean") {
                            self.clarifications_needed += 1;
                        }
                        
                        // Check for new concepts
                        if content.contains("What's") || content.contains("How does") || 
                           content.contains("explain") || content.contains("is a ") {
                            self.new_concepts_introduced += 1;
                        }
                    }
                }
            }
        }
        
        // Update mood based on metrics
        if self.outcomes_achieved > self.clarifications_needed {
            self.session_mood = SessionMood::Productive;
        } else if self.clarifications_needed > 3 {
            self.session_mood = SessionMood::Stuck;
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SessionMood {
    Frustrated,
    Stuck,
    Neutral,
    Productive,
    Excited,
}

/// Analysis of conversation flow
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConversationFlow {
    pub tool: String,
    pub turn_count: usize,
    pub question_count: usize,
    pub clarification_count: usize,
    pub completion_count: usize,
    pub context_switches: usize,
    pub flow_smoothness: f32,
}

/// Interaction patterns between human and AI
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InteractionPatterns {
    pub tool_preferences: HashMap<String, usize>,
    pub peak_collaboration_hours: Vec<usize>,
    pub average_session_length: Duration,
    pub response_patterns: ResponsePatterns,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ResponsePatterns {
    pub total_responses: usize,
    pub detailed_responses: usize,
    pub brief_responses: usize,
    pub code_heavy_responses: usize,
    pub proactive_responses: usize,
}

/// Metrics for collaboration quality
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct CollaborationMetrics {
    pub productivity_rate: f32,
    pub learning_rate: f32,
    pub stuck_rate: f32,
    pub collaboration_depth: f32,
    pub mutual_understanding: f32,
}

/// How the relationship has evolved over time
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct RelationshipEvolution {
    pub productivity_trend: f32, // Positive = improving
    pub understanding_trend: f32,
    pub complexity_trend: f32,   // Handling more complex tasks
    pub trust_indicators: TrustIndicators,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TrustIndicators {
    pub autonomy_level: f32,
    pub correction_acceptance: f32,
    pub suggestion_adoption_rate: f32,
}

/// Shared understanding metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SharedUnderstanding {
    pub vocabulary_size: usize,
    pub concept_alignment: f32,
    pub communication_efficiency: f32,
    pub domain_expertise_areas: Vec<String>,
}

/// Overall partnership health
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PartnershipHealth {
    pub overall_score: f32,
    pub status: String,
    pub strengths: Vec<String>,
    pub areas_for_improvement: Vec<String>,
}

/// Complete partnership analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PartnershipAnalysis {
    pub total_interactions: usize,
    pub collaborative_sessions: Vec<CollaborativeSession>,
    pub conversation_flows: Vec<ConversationFlow>,
    pub interaction_patterns: InteractionPatterns,
    pub collaboration_metrics: CollaborationMetrics,
    pub relationship_evolution: RelationshipEvolution,
    pub shared_understanding: SharedUnderstanding,
    pub partnership_health: PartnershipHealth,
    pub recommendations: Vec<String>,
}