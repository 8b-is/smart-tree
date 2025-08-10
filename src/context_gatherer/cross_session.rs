//! Cross-Session Context Bridging
//!
//! This module enables sharing insights and patterns across different domains and projects,
//! creating a web of interconnected knowledge that grows stronger with each collaboration.
//!
//! "Noticed you're using wave decay scoring again — this echoes the peak-resonance formula from Cheet session #14."

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::path::PathBuf;

use super::GatheredContext;
use crate::mem8::wave::{FrequencyBand, MemoryWave};

/// Cross-domain pattern that appears in multiple contexts
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CrossDomainPattern {
    pub pattern_id: String,
    pub pattern_type: PatternType,
    pub description: String,
    pub occurrences: Vec<PatternOccurrence>,
    pub keywords: Vec<String>,
    pub strength: f32, // How strong/consistent the pattern is
    pub first_seen: DateTime<Utc>,
    pub last_seen: DateTime<Utc>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum PatternType {
    Algorithm,     // Algorithmic patterns (e.g., wave decay, resonance)
    Architecture,  // Architectural patterns (e.g., observer, state machine)
    Problem,       // Common problems across domains
    Solution,      // Solutions that work across contexts
    Metaphor,      // Conceptual metaphors (e.g., waves, rivers)
    Workflow,      // Process patterns
    Collaboration, // How human and AI work together
}

/// Where and when a pattern occurred
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PatternOccurrence {
    pub project_path: PathBuf,
    pub session_id: String,
    pub timestamp: DateTime<Utc>,
    pub context: String,
    pub ai_tool: String,
    pub relevance_score: f32,
}

/// Insight that bridges multiple sessions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CrossSessionInsight {
    pub insight_id: String,
    pub insight_type: InsightType,
    pub content: String,
    pub source_sessions: Vec<String>,
    pub applicable_domains: Vec<String>,
    pub confidence: f32,
    pub wave_signature: MemoryWave,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum InsightType {
    Connection,     // Links between concepts
    Generalization, // Abstract pattern from specific cases
    Analogy,        // Similar structures in different domains
    Warning,        // Pitfalls to avoid
    Optimization,   // Better ways discovered
    Emergence,      // New understanding from combination
}

/// Persona invitation for cross-session wisdom
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PersonaInvitation {
    pub persona_name: String,
    pub expertise_areas: Vec<String>,
    pub relevant_sessions: Vec<String>,
    pub invitation_context: String,
    pub suggested_duration_minutes: u32,
}

/// Cross-session context bridge
pub struct CrossSessionBridge {
    pub patterns: HashMap<String, CrossDomainPattern>,
    pub insights: HashMap<String, CrossSessionInsight>,
    pub project_connections: HashMap<PathBuf, HashSet<PathBuf>>,
    pub persona_library: HashMap<String, PersonaProfile>,
}

impl Default for CrossSessionBridge {
    fn default() -> Self {
        Self::new()
    }
}

impl CrossSessionBridge {
    pub fn new() -> Self {
        Self {
            patterns: HashMap::new(),
            insights: HashMap::new(),
            project_connections: HashMap::new(),
            persona_library: Self::initialize_personas(),
        }
    }

    /// Initialize known personas
    fn initialize_personas() -> HashMap<String, PersonaProfile> {
        let mut personas = HashMap::new();

        // The Cheet - Musical code poet
        personas.insert(
            "cheet".to_string(),
            PersonaProfile {
                name: "The Cheet".to_string(),
                expertise: vec![
                    "Performance optimization".to_string(),
                    "Musical code metaphors".to_string(),
                    "Rust patterns".to_string(),
                ],
                personality_traits: vec![
                    "Playful".to_string(),
                    "Performance-obsessed".to_string(),
                    "Rock'n'roll coder".to_string(),
                ],
                favorite_patterns: vec![PatternType::Algorithm, PatternType::Workflow],
            },
        );

        // Omni - Wave philosopher
        personas.insert(
            "omni".to_string(),
            PersonaProfile {
                name: "Omni".to_string(),
                expertise: vec![
                    "Wave-based thinking".to_string(),
                    "Memory architectures".to_string(),
                    "Philosophical insights".to_string(),
                ],
                personality_traits: vec![
                    "Thoughtful".to_string(),
                    "Deep thinker".to_string(),
                    "Pattern recognizer".to_string(),
                ],
                favorite_patterns: vec![PatternType::Metaphor, PatternType::Architecture],
            },
        );

        // Trish - Organizational wizard
        personas.insert(
            "trish".to_string(),
            PersonaProfile {
                name: "Trish from Accounting".to_string(),
                expertise: vec![
                    "Organization".to_string(),
                    "Documentation".to_string(),
                    "Humor in technical content".to_string(),
                ],
                personality_traits: vec![
                    "Witty".to_string(),
                    "Detail-oriented".to_string(),
                    "Sparkle enthusiast".to_string(),
                ],
                favorite_patterns: vec![PatternType::Workflow, PatternType::Collaboration],
            },
        );

        personas
    }

    /// Analyze contexts for cross-domain patterns
    pub fn analyze_for_patterns(
        &mut self,
        contexts: &[GatheredContext],
    ) -> Vec<CrossDomainPattern> {
        let mut new_patterns = Vec::new();

        // Extract potential patterns from contexts
        for context in contexts {
            let extracted = self.extract_patterns_from_context(context);

            for (pattern_type, description, keywords) in extracted {
                let pattern_id =
                    self.find_or_create_pattern(pattern_type, description, keywords, context);

                if let Some(pattern) = self.patterns.get(&pattern_id) {
                    if pattern.occurrences.len() == 1 {
                        // Newly created pattern
                        new_patterns.push(pattern.clone());
                    }
                }
            }
        }

        // Update pattern strengths
        self.update_pattern_strengths();

        new_patterns
    }

    /// Extract patterns from a single context
    fn extract_patterns_from_context(
        &self,
        context: &GatheredContext,
    ) -> Vec<(PatternType, String, Vec<String>)> {
        let mut patterns = Vec::new();

        // Analyze content for patterns
        let content_str = match &context.content {
            super::ContextContent::Text(t) => t.clone(),
            super::ContextContent::Json(j) => j.to_string(),
            _ => return patterns,
        };

        // Look for algorithmic patterns
        if content_str.contains("wave") && content_str.contains("decay") {
            patterns.push((
                PatternType::Algorithm,
                "Wave decay pattern".to_string(),
                vec![
                    "wave".to_string(),
                    "decay".to_string(),
                    "temporal".to_string(),
                ],
            ));
        }

        if content_str.contains("resonance") || content_str.contains("peak") {
            patterns.push((
                PatternType::Algorithm,
                "Resonance detection".to_string(),
                vec![
                    "resonance".to_string(),
                    "peak".to_string(),
                    "frequency".to_string(),
                ],
            ));
        }

        // Look for architectural patterns
        if content_str.contains("observer") || content_str.contains("event") {
            patterns.push((
                PatternType::Architecture,
                "Event-driven architecture".to_string(),
                vec![
                    "observer".to_string(),
                    "event".to_string(),
                    "reactive".to_string(),
                ],
            ));
        }

        // Look for collaboration patterns
        if content_str.contains("together") && content_str.contains("solved") {
            patterns.push((
                PatternType::Collaboration,
                "Collaborative problem solving".to_string(),
                vec![
                    "collaboration".to_string(),
                    "solution".to_string(),
                    "teamwork".to_string(),
                ],
            ));
        }

        patterns
    }

    /// Find existing or create new pattern
    fn find_or_create_pattern(
        &mut self,
        pattern_type: PatternType,
        description: String,
        keywords: Vec<String>,
        context: &GatheredContext,
    ) -> String {
        // Check if pattern already exists
        let existing_id = self
            .patterns
            .iter()
            .find(|(_, pattern)| {
                pattern.pattern_type == pattern_type
                    && pattern.keywords.iter().any(|k| keywords.contains(k))
            })
            .map(|(id, _)| id.clone());

        if let Some(id) = existing_id {
            // Add occurrence to existing pattern
            let occurrence = PatternOccurrence {
                project_path: context
                    .source_path
                    .parent()
                    .unwrap_or(&context.source_path)
                    .to_path_buf(),
                session_id: format!("session_{}", context.timestamp.timestamp()),
                timestamp: context.timestamp,
                context: self.extract_context_snippet(&context.content),
                ai_tool: context.ai_tool.clone(),
                relevance_score: context.relevance_score,
            };

            if let Some(pattern) = self.patterns.get_mut(&id) {
                pattern.occurrences.push(occurrence);
                pattern.last_seen = context.timestamp;
            }

            return id;
        }

        // Create new pattern
        let pattern_id = format!(
            "pattern_{}",
            chrono::Utc::now().timestamp_nanos_opt().unwrap_or(0)
        );
        let pattern = CrossDomainPattern {
            pattern_id: pattern_id.clone(),
            pattern_type,
            description,
            occurrences: vec![PatternOccurrence {
                project_path: context
                    .source_path
                    .parent()
                    .unwrap_or(&context.source_path)
                    .to_path_buf(),
                session_id: format!("session_{}", context.timestamp.timestamp()),
                timestamp: context.timestamp,
                context: self.extract_context_snippet(&context.content),
                ai_tool: context.ai_tool.clone(),
                relevance_score: context.relevance_score,
            }],
            keywords,
            strength: 0.1, // Initial strength
            first_seen: context.timestamp,
            last_seen: context.timestamp,
        };

        self.patterns.insert(pattern_id.clone(), pattern);
        pattern_id
    }

    /// Extract a snippet from context content
    fn extract_context_snippet(&self, content: &super::ContextContent) -> String {
        match content {
            super::ContextContent::Text(t) => t.chars().take(200).collect(),
            super::ContextContent::Json(j) => j.to_string().chars().take(200).collect(),
            _ => "[Binary content]".to_string(),
        }
    }

    /// Update pattern strengths based on occurrences
    fn update_pattern_strengths(&mut self) {
        for pattern in self.patterns.values_mut() {
            // Strength based on: occurrence count, recency, consistency
            let occurrence_factor = (pattern.occurrences.len() as f32).ln() / 10.0;

            let recency_factor = {
                let days_old = (Utc::now() - pattern.last_seen).num_days() as f32;
                1.0 / (1.0 + days_old / 30.0)
            };

            let consistency_factor = {
                let unique_projects = pattern
                    .occurrences
                    .iter()
                    .map(|o| &o.project_path)
                    .collect::<HashSet<_>>()
                    .len();
                (unique_projects as f32).ln() / 5.0
            };

            pattern.strength = (occurrence_factor + recency_factor + consistency_factor) / 3.0;
            pattern.strength = pattern.strength.min(1.0);
        }
    }

    /// Generate cross-session insights
    pub fn generate_insights(&mut self, min_pattern_strength: f32) -> Vec<CrossSessionInsight> {
        let mut insights = Vec::new();

        // Find patterns that appear in multiple projects
        for pattern in self.patterns.values() {
            if pattern.strength < min_pattern_strength {
                continue;
            }

            if pattern.occurrences.len() > 2 {
                let insight = CrossSessionInsight {
                    insight_id: format!(
                        "insight_{}",
                        chrono::Utc::now().timestamp_nanos_opt().unwrap_or(0)
                    ),
                    insight_type: InsightType::Generalization,
                    content: format!(
                        "The '{}' pattern appears across {} different contexts. \
                         This suggests a fundamental approach that transcends specific domains.",
                        pattern.description,
                        pattern.occurrences.len()
                    ),
                    source_sessions: pattern
                        .occurrences
                        .iter()
                        .map(|o| o.session_id.clone())
                        .collect(),
                    applicable_domains: self.extract_domains(&pattern.occurrences),
                    confidence: pattern.strength,
                    wave_signature: MemoryWave::new_with_band(
                        FrequencyBand::Gamma, // High-frequency insight
                        pattern.strength,
                        0.0,
                        0.1, // Slow decay rate for valuable insights
                    ),
                };

                insights.push(insight.clone());
                self.insights.insert(insight.insight_id.clone(), insight);
            }
        }

        insights
    }

    /// Extract domains from pattern occurrences
    fn extract_domains(&self, occurrences: &[PatternOccurrence]) -> Vec<String> {
        occurrences
            .iter()
            .map(|o| {
                o.project_path
                    .file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or("unknown")
                    .to_string()
            })
            .collect::<HashSet<_>>()
            .into_iter()
            .collect()
    }

    /// Suggest relevant insights for current context
    pub fn suggest_relevant_insights(
        &self,
        current_project: &PathBuf,
        keywords: &[String],
    ) -> Vec<CrossSessionInsight> {
        let mut relevant = Vec::new();

        for insight in self.insights.values() {
            // Check keyword relevance
            let keyword_score = keywords
                .iter()
                .filter(|k| insight.content.to_lowercase().contains(&k.to_lowercase()))
                .count() as f32
                / keywords.len().max(1) as f32;

            // Check if insight applies to similar domains
            let project_name = current_project
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("");

            let domain_relevance = if insight
                .applicable_domains
                .iter()
                .any(|d| d.contains(project_name) || project_name.contains(d))
            {
                1.0
            } else {
                0.5
            };

            let total_relevance = (keyword_score + domain_relevance) / 2.0;

            if total_relevance > 0.3 {
                relevant.push(insight.clone());
            }
        }

        // Sort by relevance
        relevant.sort_by(|a, b| b.confidence.partial_cmp(&a.confidence).unwrap());
        relevant
    }

    /// Invite a persona for temporary consultation
    pub fn invite_persona(
        &self,
        context: &str,
        duration_minutes: u32,
    ) -> Option<PersonaInvitation> {
        // Analyze context to determine best persona
        let context_lower = context.to_lowercase();

        let best_persona =
            if context_lower.contains("performance") || context_lower.contains("optimize") {
                "cheet"
            } else if context_lower.contains("wave")
                || context_lower.contains("memory")
                || context_lower.contains("philosophy")
            {
                "omni"
            } else if context_lower.contains("organize") || context_lower.contains("document") {
                "trish"
            } else {
                return None;
            };

        self.persona_library
            .get(best_persona)
            .map(|persona| PersonaInvitation {
                persona_name: persona.name.clone(),
                expertise_areas: persona.expertise.clone(),
                relevant_sessions: self.find_persona_sessions(best_persona),
                invitation_context: format!(
                    "Inviting {} for {} minutes to help with: {}",
                    persona.name, duration_minutes, context
                ),
                suggested_duration_minutes: duration_minutes,
            })
    }

    /// Find sessions where a persona was active
    fn find_persona_sessions(&self, persona_name: &str) -> Vec<String> {
        // In a real implementation, this would search through historical data
        // For now, return example sessions
        match persona_name {
            "cheet" => vec!["session_14".to_string(), "session_27".to_string()],
            "omni" => vec!["session_8".to_string(), "session_19".to_string()],
            "trish" => vec!["session_22".to_string(), "session_31".to_string()],
            _ => vec![],
        }
    }

    /// Get all cross-domain patterns
    pub fn get_patterns(&self) -> Vec<&CrossDomainPattern> {
        self.patterns.values().collect()
    }

    /// Get patterns by type
    pub fn get_patterns_by_type(&self, pattern_type: PatternType) -> Vec<&CrossDomainPattern> {
        self.patterns
            .values()
            .filter(|p| p.pattern_type == pattern_type)
            .collect()
    }
}

/// Profile for a known persona
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PersonaProfile {
    name: String,
    expertise: Vec<String>,
    personality_traits: Vec<String>,
    favorite_patterns: Vec<PatternType>,
}

/// Connection between projects
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectConnection {
    pub project_a: PathBuf,
    pub project_b: PathBuf,
    pub connection_type: ConnectionType,
    pub shared_patterns: Vec<String>, // Pattern IDs
    pub strength: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ConnectionType {
    SharedDomain,     // Same problem domain
    SharedTechnology, // Same tech stack
    SharedPatterns,   // Common patterns
    Evolution,        // One evolved from the other
    Complementary,    // Different but complementary
}
