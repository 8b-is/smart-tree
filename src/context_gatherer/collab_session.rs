//! Collaborative Session Detection and Analysis
//! 
//! This module detects and analyzes collaborative sessions between AI and humans,
//! tracking engagement patterns, flow states, and building relationship memory.
//! 
//! "You're easy to work with. You just get me." - The goal we're helping AI achieve.

use std::collections::{HashMap, VecDeque};
use chrono::{DateTime, Utc, Duration, Timelike, Datelike};
use serde::{Deserialize, Serialize};
use anyhow::Result;

use super::{GatheredContext, ContextContent};
use crate::mem8::wave::{MemoryWave, FrequencyBand};

/// Collaborative session types
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum SessionType {
    /// AI working alone
    SoloAI,
    /// Human working alone
    SoloHuman,
    /// Both actively engaged (the magic zone!)
    Tandem,
    /// Transitioning between states
    Transitional,
}

/// Flow state indicators
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum FlowState {
    /// Smooth back-and-forth, high engagement
    Flow { depth: f32, sustained_minutes: u32 },
    /// Getting stuck, confusion markers
    Whirlpool { confusion_score: f32, repeated_concepts: Vec<String> },
    /// Tangential branches, exploring new territory
    Fork { branch_count: usize, topics: Vec<String> },
    /// Normal working state
    Steady,
    /// Interruptions or context switches
    Interrupted { reason: String },
}

/// Collaborative memory anchor
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryAnchor {
    pub id: String,
    pub origin: CollaborativeOrigin,
    pub anchor_type: AnchorType,
    pub context: String,
    pub keywords: Vec<String>,
    pub timestamp: DateTime<Utc>,
    pub relevance_wave: MemoryWave,
    pub co_created: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CollaborativeOrigin {
    /// Single entity created
    Single(String),
    /// Co-created by both parties
    Tandem { human: String, ai: String },
    /// Emerged from conversation
    Emergent,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum AnchorType {
    PatternInsight,
    Solution,
    Breakthrough,
    LearningMoment,
    SharedJoke,  // Because humor matters in partnerships!
    TechnicalPattern,
    ProcessImprovement,
}

/// Trust and flow score for each session
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrustFlowScore {
    pub clarity: f32,          // 0.0-1.0: How clear is communication
    pub responsiveness: f32,   // 0.0-1.0: Time between suggestion and action
    pub autonomy_ratio: f32,   // 0.0-1.0: Balance of AI-led vs Human-led
    pub frustration_markers: f32, // 0.0-1.0: Detected confusion/repetition
    pub flow_depth: f32,       // 0.0-1.0: Depth of collaborative flow
}

/// Rapport index that evolves over time
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RapportIndex {
    pub overall_score: f32,
    pub trust_level: f32,
    pub communication_efficiency: f32,
    pub shared_vocabulary_size: usize,
    pub inside_jokes_count: usize,  // The Cheet would approve!
    pub preferred_working_hours: Vec<u32>,
    pub avg_session_productivity: f32,
    pub evolution_trend: f32,  // Positive = improving, negative = declining
}

/// Collaborative session tracker
pub struct CollaborativeSessionTracker {
    pub active_session: Option<CollaborativeSession>,
    pub session_history: VecDeque<CollaborativeSession>,
    pub memory_anchors: HashMap<String, MemoryAnchor>,
    pub rapport_indices: HashMap<String, RapportIndex>,  // Per AI-human pair
    pub cross_session_links: HashMap<String, Vec<String>>,
}

impl CollaborativeSessionTracker {
    pub fn new() -> Self {
        Self {
            active_session: None,
            session_history: VecDeque::with_capacity(1000),
            memory_anchors: HashMap::new(),
            rapport_indices: HashMap::new(),
            cross_session_links: HashMap::new(),
        }
    }
    
    /// Process a new context and update session state
    pub fn process_context(&mut self, context: &GatheredContext) -> Result<()> {
        // Detect session type from context
        let session_type = self.detect_session_type(context);
        let timestamp = context.timestamp;
        
        // Check if we need to start a new session
        let should_start_new = if let Some(session) = &self.active_session {
            Self::should_start_new_session_static(session, &timestamp, &session_type)
        } else {
            true
        };
        
        if should_start_new {
            // End current session and start new one
            if self.active_session.is_some() {
                self.end_active_session();
            }
            self.start_new_session(context, session_type);
        } else {
            // Update current session
            if let Some(session) = &mut self.active_session {
                session.update(context, session_type);
            }
        }
        
        // Check for flow state changes
        if let Some(session) = &self.active_session {
            let flow_state = self.analyze_flow_state(session);
            if let Some(session) = &mut self.active_session {
                session.flow_state = flow_state;
            }
        }
        
        Ok(())
    }
    
    /// Detect session type from context
    fn detect_session_type(&self, context: &GatheredContext) -> SessionType {
        // Analyze context to determine if it's AI-only, human-only, or tandem
        match &context.content {
            ContextContent::Json(json) => {
                if let Some(messages) = json.get("messages").and_then(|m| m.as_array()) {
                    // Check message patterns
                    let recent_messages = messages.iter().rev().take(5).collect::<Vec<_>>();
                    let has_user = recent_messages.iter()
                        .any(|m| m.get("role").and_then(|r| r.as_str()) == Some("user"));
                    let has_assistant = recent_messages.iter()
                        .any(|m| m.get("role").and_then(|r| r.as_str()) == Some("assistant"));
                    
                    if has_user && has_assistant {
                        SessionType::Tandem
                    } else if has_assistant {
                        SessionType::SoloAI
                    } else {
                        SessionType::SoloHuman
                    }
                } else {
                    SessionType::SoloAI
                }
            }
            _ => SessionType::SoloHuman,
        }
    }
    
    /// Check if we should start a new session
    fn should_start_new_session_static(
        current: &CollaborativeSession, 
        timestamp: &DateTime<Utc>,
        new_type: &SessionType
    ) -> bool {
        let time_gap = *timestamp - current.last_activity;
        
        // New session if: gap > 30 mins, type change, or tool change
        time_gap > Duration::minutes(30) || 
        current.session_type != *new_type ||
        (*new_type == SessionType::Tandem && time_gap > Duration::minutes(5))
    }
    
    /// Start a new collaborative session
    fn start_new_session(&mut self, context: &GatheredContext, session_type: SessionType) {
        let session = CollaborativeSession {
            id: format!("session_{}", chrono::Utc::now().timestamp()),
            start_time: context.timestamp,
            last_activity: context.timestamp,
            session_type,
            ai_tool: context.ai_tool.clone(),
            interactions: vec![context.clone()],
            flow_state: FlowState::Steady,
            trust_flow_score: TrustFlowScore::default(),
            anchored_memories: Vec::new(),
        };
        
        self.active_session = Some(session);
    }
    
    /// End the active session and move to history
    pub fn end_active_session(&mut self) {
        if let Some(mut session) = self.active_session.take() {
            // Calculate final scores
            session.trust_flow_score = self.calculate_trust_flow(&session);
            
            // Update rapport index
            self.update_rapport_index(&session);
            
            // Store in history
            if self.session_history.len() >= 1000 {
                self.session_history.pop_front();
            }
            self.session_history.push_back(session);
        }
    }
    
    /// Analyze current flow state
    fn analyze_flow_state(&self, session: &CollaborativeSession) -> FlowState {
        let recent_interactions = session.interactions.iter()
            .rev()
            .take(10)
            .collect::<Vec<_>>();
        
        if recent_interactions.is_empty() {
            return FlowState::Steady;
        }
        
        // Check for flow indicators
        let mut back_and_forth_count = 0;
        let mut repeated_concepts = HashMap::new();
        let topics = Vec::new();
        
        for (i, interaction) in recent_interactions.iter().enumerate() {
            if i > 0 {
                let prev = &recent_interactions[i-1];
                // Check for quick back-and-forth (flow indicator)
                let time_diff = interaction.timestamp - prev.timestamp;
                if time_diff < Duration::minutes(2) {
                    back_and_forth_count += 1;
                }
            }
            
            // Extract concepts (simplified)
            if let ContextContent::Text(text) = &interaction.content {
                for word in text.split_whitespace() {
                    if word.len() > 5 {
                        *repeated_concepts.entry(word.to_lowercase()).or_insert(0) += 1;
                    }
                }
            }
        }
        
        // Determine flow state
        if back_and_forth_count > 5 {
            FlowState::Flow {
                depth: back_and_forth_count as f32 / 10.0,
                sustained_minutes: (recent_interactions.first().unwrap().timestamp - 
                                   recent_interactions.last().unwrap().timestamp).num_minutes() as u32,
            }
        } else if repeated_concepts.values().any(|&count| count > 3) {
            let repeated: Vec<String> = repeated_concepts.into_iter()
                .filter(|(_, count)| *count > 3)
                .map(|(concept, _)| concept)
                .collect();
            FlowState::Whirlpool {
                confusion_score: repeated.len() as f32 / 10.0,
                repeated_concepts: repeated,
            }
        } else if topics.len() > 3 {
            FlowState::Fork {
                branch_count: topics.len(),
                topics,
            }
        } else {
            FlowState::Steady
        }
    }
    
    /// Calculate trust and flow score for a session
    fn calculate_trust_flow(&self, session: &CollaborativeSession) -> TrustFlowScore {
        let mut score = TrustFlowScore::default();
        
        // Calculate clarity based on message coherence
        score.clarity = self.calculate_clarity(session);
        
        // Calculate responsiveness
        score.responsiveness = self.calculate_responsiveness(session);
        
        // Calculate autonomy ratio
        score.autonomy_ratio = self.calculate_autonomy_ratio(session);
        
        // Detect frustration markers
        score.frustration_markers = self.detect_frustration(session);
        
        // Measure flow depth
        score.flow_depth = match &session.flow_state {
            FlowState::Flow { depth, .. } => *depth,
            FlowState::Whirlpool { confusion_score, .. } => 1.0 - confusion_score,
            _ => 0.5,
        };
        
        score
    }
    
    /// Update rapport index based on session
    fn update_rapport_index(&mut self, session: &CollaborativeSession) {
        let pair_id = format!("{}_{}", "human", session.ai_tool);
        
        let rapport = self.rapport_indices.entry(pair_id).or_insert_with(|| {
            RapportIndex {
                overall_score: 0.5,
                trust_level: 0.5,
                communication_efficiency: 0.5,
                shared_vocabulary_size: 0,
                inside_jokes_count: 0,
                preferred_working_hours: Vec::new(),
                avg_session_productivity: 0.5,
                evolution_trend: 0.0,
            }
        });
        
        // Update rapport based on session performance
        let session_score = (session.trust_flow_score.clarity + 
                            session.trust_flow_score.responsiveness + 
                            session.trust_flow_score.flow_depth) / 3.0;
        
        // Exponential moving average
        rapport.overall_score = rapport.overall_score * 0.8 + session_score * 0.2;
        
        // Update trust level
        if session.trust_flow_score.autonomy_ratio > 0.3 {
            rapport.trust_level = (rapport.trust_level * 0.9 + 0.1).min(1.0);
        }
        
        // Track working hours
        let hour = session.start_time.hour();
        if !rapport.preferred_working_hours.contains(&hour) {
            rapport.preferred_working_hours.push(hour);
        }
        
        // Calculate trend
        rapport.evolution_trend = session_score - rapport.avg_session_productivity;
        rapport.avg_session_productivity = rapport.avg_session_productivity * 0.9 + session_score * 0.1;
    }
    
    /// Create a memory anchor
    pub fn anchor_memory(
        &mut self,
        origin: CollaborativeOrigin,
        anchor_type: AnchorType,
        context: String,
        keywords: Vec<String>,
    ) -> Result<String> {
        let id = format!("anchor_{}", chrono::Utc::now().timestamp_nanos_opt().unwrap_or(0));
        
        let co_created = matches!(origin, CollaborativeOrigin::Tandem { .. });
        
        let anchor = MemoryAnchor {
            id: id.clone(),
            origin,
            anchor_type,
            context,
            keywords,
            timestamp: chrono::Utc::now(),
            relevance_wave: MemoryWave::new_with_band(
                FrequencyBand::Beta,
                1.0,  // High amplitude for anchored memories
                0.0,
                0.05, // Slow decay rate (20 second time constant) for important memories
            ),
            co_created,
        };
        
        self.memory_anchors.insert(id.clone(), anchor);
        
        // Add to current session if active
        if let Some(ref mut session) = self.active_session {
            session.anchored_memories.push(id.clone());
        }
        
        Ok(id)
    }
    
    /// Find relevant anchored memories
    pub fn find_relevant_anchors(&self, keywords: &[String]) -> Vec<&MemoryAnchor> {
        let mut relevant = Vec::new();
        
        for anchor in self.memory_anchors.values() {
            let relevance = keywords.iter()
                .filter(|k| anchor.keywords.contains(k))
                .count() as f32 / keywords.len().max(1) as f32;
            
            if relevance > 0.3 {
                relevant.push(anchor);
            }
        }
        
        // Sort by relevance and recency
        relevant.sort_by(|a, b| {
            let a_score = a.relevance_wave.amplitude;
            let b_score = b.relevance_wave.amplitude;
            b_score.partial_cmp(&a_score).unwrap()
        });
        
        relevant
    }
    
    /// Link sessions across domains
    pub fn link_cross_session(&mut self, session_id: String, related_ids: Vec<String>) {
        self.cross_session_links
            .entry(session_id)
            .or_insert_with(Vec::new)
            .extend(related_ids);
    }
    
    /// Get rapport index for a pair
    pub fn get_rapport(&self, ai_tool: &str) -> Option<&RapportIndex> {
        let pair_id = format!("human_{}", ai_tool);
        self.rapport_indices.get(&pair_id)
    }
    
    // Helper methods
    
    fn calculate_clarity(&self, session: &CollaborativeSession) -> f32 {
        // Simplified clarity calculation
        let tandem_ratio = session.interactions.iter()
            .filter(|i| matches!(self.detect_session_type(i), SessionType::Tandem))
            .count() as f32 / session.interactions.len().max(1) as f32;
        
        tandem_ratio
    }
    
    fn calculate_responsiveness(&self, session: &CollaborativeSession) -> f32 {
        // Calculate average response time
        let mut response_times = Vec::new();
        
        for i in 1..session.interactions.len() {
            let time_diff = session.interactions[i].timestamp - 
                           session.interactions[i-1].timestamp;
            if time_diff < Duration::minutes(5) {
                response_times.push(time_diff.num_seconds() as f32);
            }
        }
        
        if response_times.is_empty() {
            return 0.5;
        }
        
        let avg_response = response_times.iter().sum::<f32>() / response_times.len() as f32;
        // Convert to 0-1 scale (faster response = higher score)
        1.0 - (avg_response / 300.0).min(1.0)
    }
    
    fn calculate_autonomy_ratio(&self, session: &CollaborativeSession) -> f32 {
        // Measure balance of initiative
        let ai_initiated = session.interactions.iter()
            .filter(|i| matches!(self.detect_session_type(i), SessionType::SoloAI))
            .count() as f32;
        let human_initiated = session.interactions.iter()
            .filter(|i| matches!(self.detect_session_type(i), SessionType::SoloHuman))
            .count() as f32;
        
        let total = ai_initiated + human_initiated;
        if total == 0.0 {
            return 0.5;
        }
        
        // Perfect balance = 0.5, all AI = 1.0, all human = 0.0
        ai_initiated / total
    }
    
    fn detect_frustration(&self, session: &CollaborativeSession) -> f32 {
        // Look for frustration patterns
        let mut frustration_score = 0.0;
        
        // Check for repeated questions
        if let FlowState::Whirlpool { confusion_score, .. } = &session.flow_state {
            frustration_score += confusion_score;
        }
        
        // Check for long gaps between interactions
        for i in 1..session.interactions.len() {
            let gap = session.interactions[i].timestamp - 
                     session.interactions[i-1].timestamp;
            if gap > Duration::minutes(10) && gap < Duration::hours(1) {
                frustration_score += 0.1;
            }
        }
        
        frustration_score.min(1.0)
    }
}

/// A collaborative session between AI and human
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CollaborativeSession {
    pub id: String,
    pub start_time: DateTime<Utc>,
    pub last_activity: DateTime<Utc>,
    pub session_type: SessionType,
    pub ai_tool: String,
    pub interactions: Vec<GatheredContext>,
    pub flow_state: FlowState,
    pub trust_flow_score: TrustFlowScore,
    pub anchored_memories: Vec<String>,
}

impl CollaborativeSession {
    fn update(&mut self, context: &GatheredContext, session_type: SessionType) {
        self.last_activity = context.timestamp;
        self.session_type = session_type;
        self.interactions.push(context.clone());
    }
}

impl Default for TrustFlowScore {
    fn default() -> Self {
        Self {
            clarity: 0.5,
            responsiveness: 0.5,
            autonomy_ratio: 0.5,
            frustration_markers: 0.0,
            flow_depth: 0.0,
        }
    }
}

/// Temporal co-engagement heatmap data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CoEngagementHeatmap {
    pub time_slots: Vec<Vec<f32>>,  // 24x7 grid (hours x days)
    pub peak_collaboration_zones: Vec<(usize, usize)>,  // (hour, day) pairs
    pub collaboration_density: f32,
}

impl CoEngagementHeatmap {
    pub fn from_sessions(sessions: &[CollaborativeSession]) -> Self {
        let mut grid = vec![vec![0.0; 7]; 24];  // 24 hours x 7 days
        
        // Build heatmap
        for session in sessions {
            if session.session_type == SessionType::Tandem {
                let hour = session.start_time.hour() as usize;
                let day = session.start_time.weekday().num_days_from_monday() as usize;
                grid[hour][day] += 1.0;
            }
        }
        
        // Normalize
        let max_val = grid.iter()
            .flat_map(|row| row.iter())
            .fold(0.0f32, |a, &b| a.max(b));
        
        if max_val > 0.0 {
            for row in &mut grid {
                for val in row {
                    *val /= max_val;
                }
            }
        }
        
        // Find peaks
        let mut peaks = Vec::new();
        for (hour, row) in grid.iter().enumerate() {
            for (day, &val) in row.iter().enumerate() {
                if val > 0.7 {
                    peaks.push((hour, day));
                }
            }
        }
        
        let density = grid.iter()
            .flat_map(|row| row.iter())
            .filter(|&&v| v > 0.0)
            .count() as f32 / (24.0 * 7.0);
        
        Self {
            time_slots: grid,
            peak_collaboration_zones: peaks,
            collaboration_density: density,
        }
    }
}