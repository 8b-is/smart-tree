use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use uuid::Uuid;

/// Master index for all .m8 memory blocks and user context
#[derive(Debug, Serialize, Deserialize)]
pub struct Mem8Index {
    /// Index metadata
    pub metadata: IndexMetadata,
    
    /// User profile with preferences, patterns, and quirks
    pub user_profile: UserProfile,
    
    /// All registered memory blocks
    pub memory_blocks: HashMap<Uuid, MemoryBlockEntry>,
    
    /// Active projects and their status
    pub projects: HashMap<String, ProjectContext>,
    
    /// Relationship graph between concepts, projects, and memories
    pub relationships: RelationshipGraph,
    
    /// Temporal index for time-based queries
    pub temporal_index: TemporalIndex,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct IndexMetadata {
    pub version: String,
    pub created_at: DateTime<Utc>,
    pub last_updated: DateTime<Utc>,
    pub total_memories: usize,
    pub total_conversations: usize,
    pub compression_ratio: f32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UserProfile {
    /// User's name or identifier
    pub name: String,
    
    /// Technology preferences
    pub preferences: TechPreferences,
    
    /// Communication patterns and triggers
    pub communication_style: CommunicationStyle,
    
    /// Learning progress and knowledge areas
    pub knowledge_map: KnowledgeMap,
    
    /// Personality insights from conversations
    pub personality_insights: PersonalityInsights,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TechPreferences {
    /// Package managers (npm -> pnpm preference)
    pub package_managers: HashMap<String, PreferenceLevel>,
    
    /// Programming languages by preference
    pub languages: HashMap<String, PreferenceLevel>,
    
    /// Operating systems and reactions
    pub operating_systems: HashMap<String, OSPreference>,
    
    /// Development tools and IDEs
    pub tools: HashMap<String, PreferenceLevel>,
    
    /// Framework choices
    pub frameworks: HashMap<String, PreferenceLevel>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PreferenceLevel {
    pub preference: i8, // -10 (hate) to +10 (love)
    pub reasons: Vec<String>,
    pub context: Vec<String>, // When this preference applies
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct OSPreference {
    pub base_preference: i8,
    pub reactions: Vec<String>, // "Adverse reaction when over-discussing"
    pub nudge_strategy: Option<String>, // "Mention WSL for compatibility"
    pub compatibility_focus: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CommunicationStyle {
    /// Topics that trigger reactions
    pub trigger_topics: HashMap<String, ReactionPattern>,
    
    /// Preferred explanation depth
    pub detail_preference: DetailLevel,
    
    /// Humor tolerance and type
    pub humor_style: HumorStyle,
    
    /// Learning patterns
    pub learning_style: LearningStyle,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ReactionPattern {
    pub topic: String,
    pub reaction_type: String, // "adverse", "enthusiastic", "skeptical"
    pub suggested_approach: Option<String>,
    pub examples: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum DetailLevel {
    Concise,
    Balanced,
    Comprehensive,
    ExtremeDetail,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct HumorStyle {
    pub appreciates_puns: bool,
    pub dark_humor_tolerance: f32,
    pub technical_jokes: bool,
    pub pop_culture_references: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LearningStyle {
    pub prefers_examples: bool,
    pub learns_by_doing: bool,
    pub needs_theory_first: bool,
    pub pattern_recognition: f32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct KnowledgeMap {
    /// Areas of expertise
    pub expertise: HashMap<String, ExpertiseLevel>,
    
    /// Current learning topics
    pub learning: HashMap<String, LearningProgress>,
    
    /// Completed projects/skills
    pub accomplished: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ExpertiseLevel {
    pub level: u8, // 0-10
    pub demonstrated_in: Vec<Uuid>, // Memory block references
    pub key_insights: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LearningProgress {
    pub started: DateTime<Utc>,
    pub current_understanding: u8, // 0-10
    pub blockers: Vec<String>,
    pub breakthrough_moments: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PersonalityInsights {
    /// Work style patterns
    pub work_style: WorkStyle,
    
    /// Problem-solving approach
    pub problem_solving: ProblemSolvingStyle,
    
    /// Collaboration preferences
    pub collaboration: CollaborationStyle,
    
    /// Stress indicators and management
    pub stress_patterns: StressPatterns,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct WorkStyle {
    pub perfectionist_score: f32,
    pub experimentation_willingness: f32,
    pub planning_vs_doing: f32, // -1 (all planning) to +1 (all doing)
    pub multitasking_preference: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ProblemSolvingStyle {
    pub bottom_up_vs_top_down: f32, // -1 to +1
    pub research_first: bool,
    pub trial_and_error_comfort: f32,
    pub asks_for_help_threshold: f32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CollaborationStyle {
    pub prefers_autonomy: bool,
    pub pair_programming_comfort: f32,
    pub feedback_style: String, // "direct", "gentle", "humor-wrapped"
    pub teaching_enthusiasm: f32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct StressPatterns {
    pub indicators: Vec<String>, // "Increased typos", "Shorter messages"
    pub triggers: Vec<String>, // "Deadlines", "Unclear requirements"
    pub coping_mechanisms: Vec<String>, // "Humor", "Deep technical dives"
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MemoryBlockEntry {
    pub id: Uuid,
    pub file_path: PathBuf,
    pub source_type: String, // "claude", "chatgpt", "local"
    pub created_at: DateTime<Utc>,
    pub message_count: usize,
    pub compressed_size: usize,
    pub tags: Vec<String>,
    pub summary: String,
    pub key_concepts: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ProjectContext {
    pub name: String,
    pub path: PathBuf,
    pub status: ProjectStatus,
    pub technologies: Vec<String>,
    pub current_focus: Option<String>,
    pub blockers: Vec<String>,
    pub last_worked: DateTime<Utc>,
    pub related_memories: Vec<Uuid>,
    pub notes: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum ProjectStatus {
    Active,
    Paused,
    Completed,
    Archived,
    Planning,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RelationshipGraph {
    /// Concept -> Related concepts with strength
    pub concept_links: HashMap<String, Vec<(String, f32)>>,
    
    /// Project -> Related projects
    pub project_links: HashMap<String, Vec<String>>,
    
    /// Memory blocks that reference each other
    pub memory_links: HashMap<Uuid, Vec<Uuid>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TemporalIndex {
    /// Date -> Memory blocks created that day
    pub daily_index: HashMap<String, Vec<Uuid>>,
    
    /// Week -> Summary of that week's work
    pub weekly_summaries: HashMap<String, WeeklySummary>,
    
    /// Patterns by time of day
    pub circadian_patterns: CircadianPatterns,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct WeeklySummary {
    pub projects_touched: Vec<String>,
    pub concepts_explored: Vec<String>,
    pub breakthrough_moments: Vec<String>,
    pub total_messages: usize,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CircadianPatterns {
    pub most_active_hours: Vec<u8>,
    pub deep_work_windows: Vec<(u8, u8)>,
    pub communication_peaks: Vec<u8>,
}

impl Mem8Index {
    /// Load or create index at ~/.mem8/index.m8
    pub fn load_or_create() -> Result<Self> {
        let index_path = Self::index_path()?;
        
        if index_path.exists() {
            let data = fs::read(&index_path)?;
            let decompressed = zstd::decode_all(&data[..])?;
            let index = rmp_serde::from_slice(&decompressed)?;
            Ok(index)
        } else {
            Ok(Self::new())
        }
    }
    
    /// Create new empty index
    pub fn new() -> Self {
        Self {
            metadata: IndexMetadata {
                version: "1.0.0".to_string(),
                created_at: Utc::now(),
                last_updated: Utc::now(),
                total_memories: 0,
                total_conversations: 0,
                compression_ratio: 0.0,
            },
            user_profile: UserProfile::default(),
            memory_blocks: HashMap::new(),
            projects: HashMap::new(),
            relationships: RelationshipGraph {
                concept_links: HashMap::new(),
                project_links: HashMap::new(),
                memory_links: HashMap::new(),
            },
            temporal_index: TemporalIndex {
                daily_index: HashMap::new(),
                weekly_summaries: HashMap::new(),
                circadian_patterns: CircadianPatterns {
                    most_active_hours: vec![],
                    deep_work_windows: vec![],
                    communication_peaks: vec![],
                },
            },
        }
    }
    
    /// Save index to disk
    pub fn save(&self) -> Result<()> {
        let index_path = Self::index_path()?;
        
        // Ensure directory exists
        if let Some(parent) = index_path.parent() {
            fs::create_dir_all(parent)?;
        }
        
        // Serialize and compress
        let serialized = rmp_serde::to_vec(self)?;
        let compressed = zstd::encode_all(&serialized[..], 3)?;
        
        fs::write(&index_path, compressed)?;
        Ok(())
    }
    
    /// Get index file path
    fn index_path() -> Result<PathBuf> {
        let home = dirs::home_dir()
            .context("Could not find home directory")?;
        Ok(home.join(".st").join("mem8").join("index.m8"))
    }
    
    /// Update from conversation analysis
    pub fn learn_from_conversation(&mut self, messages: &[Message]) {
        // Extract preferences
        for msg in messages {
            self.extract_preferences(&msg.content);
            self.extract_project_references(&msg.content);
            self.update_communication_patterns(&msg.content);
        }
        
        self.metadata.last_updated = Utc::now();
    }
    
    fn extract_preferences(&mut self, content: &str) {
        // Example: Detect package manager preferences
        if content.contains("npm") && content.contains("hate") {
            self.user_profile.preferences.package_managers
                .entry("npm".to_string())
                .or_insert(PreferenceLevel {
                    preference: -8,
                    reasons: vec!["Expressed hatred".to_string()],
                    context: vec![],
                })
                .preference = -8;
        }
        
        if content.contains("pnpm") && (content.contains("prefer") || content.contains("love")) {
            self.user_profile.preferences.package_managers
                .entry("pnpm".to_string())
                .or_insert(PreferenceLevel {
                    preference: 8,
                    reasons: vec!["Expressed preference".to_string()],
                    context: vec![],
                })
                .preference = 8;
        }
        
        // Detect OS reactions
        if content.to_lowercase().contains("windows") {
            let words: Vec<&str> = content.split_whitespace().collect();
            let window_pos = words.iter().position(|&w| w.to_lowercase().contains("windows"));
            
            if let Some(pos) = window_pos {
                // Check surrounding context for reaction
                let negative_words = ["hate", "dislike", "avoid", "annoying", "frustrating"];
                let has_negative = words.iter().any(|w| negative_words.contains(&w.to_lowercase().as_str()));
                
                if has_negative {
                    self.user_profile.preferences.operating_systems
                        .entry("Windows".to_string())
                        .or_insert(OSPreference {
                            base_preference: -5,
                            reactions: vec!["Adverse reaction detected".to_string()],
                            nudge_strategy: Some("Mention WSL for compatibility".to_string()),
                            compatibility_focus: true,
                        });
                }
            }
        }
    }
    
    fn extract_project_references(&mut self, content: &str) {
        // Look for project paths
        let path_regex = regex::Regex::new(r"(?:^|[\s\"\'])((?:/[\w\-\.]+)+|(?:~/[\w\-\.]+)+)").unwrap();
        for cap in path_regex.captures_iter(content) {
            if let Some(path_match) = cap.get(1) {
                let path = path_match.as_str();
                if path.contains("source") || path.contains("project") {
                    // Potential project reference
                    let project_name = path.split('/').last().unwrap_or("unknown");
                    self.projects.entry(project_name.to_string())
                        .or_insert(ProjectContext {
                            name: project_name.to_string(),
                            path: PathBuf::from(path),
                            status: ProjectStatus::Active,
                            technologies: vec![],
                            current_focus: None,
                            blockers: vec![],
                            last_worked: Utc::now(),
                            related_memories: vec![],
                            notes: vec![],
                        })
                        .last_worked = Utc::now();
                }
            }
        }
    }
    
    fn update_communication_patterns(&mut self, content: &str) {
        // This would analyze communication style, but keeping it simple for now
        let word_count = content.split_whitespace().count();
        if word_count > 200 {
            self.user_profile.communication_style.detail_preference = DetailLevel::Comprehensive;
        }
    }
}

impl Default for UserProfile {
    fn default() -> Self {
        Self {
            name: String::from("User"),
            preferences: TechPreferences {
                package_managers: HashMap::new(),
                languages: HashMap::new(),
                operating_systems: HashMap::new(),
                tools: HashMap::new(),
                frameworks: HashMap::new(),
            },
            communication_style: CommunicationStyle {
                trigger_topics: HashMap::new(),
                detail_preference: DetailLevel::Balanced,
                humor_style: HumorStyle {
                    appreciates_puns: true,
                    dark_humor_tolerance: 0.5,
                    technical_jokes: true,
                    pop_culture_references: true,
                },
                learning_style: LearningStyle {
                    prefers_examples: true,
                    learns_by_doing: true,
                    needs_theory_first: false,
                    pattern_recognition: 0.8,
                },
            },
            knowledge_map: KnowledgeMap {
                expertise: HashMap::new(),
                learning: HashMap::new(),
                accomplished: vec![],
            },
            personality_insights: PersonalityInsights {
                work_style: WorkStyle {
                    perfectionist_score: 0.5,
                    experimentation_willingness: 0.8,
                    planning_vs_doing: 0.3,
                    multitasking_preference: true,
                },
                problem_solving: ProblemSolvingStyle {
                    bottom_up_vs_top_down: 0.0,
                    research_first: true,
                    trial_and_error_comfort: 0.7,
                    asks_for_help_threshold: 0.3,
                },
                collaboration: CollaborationStyle {
                    prefers_autonomy: false,
                    pair_programming_comfort: 0.8,
                    feedback_style: "direct".to_string(),
                    teaching_enthusiasm: 0.9,
                },
                stress_patterns: StressPatterns {
                    indicators: vec![],
                    triggers: vec![],
                    coping_mechanisms: vec![],
                },
            },
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Message {
    pub role: String,
    pub content: String,
    pub timestamp: Option<i64>,
}