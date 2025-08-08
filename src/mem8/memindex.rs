use anyhow::{Context, Result};
use chrono::{DateTime, Local, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

/// Main memory index - the unified relationship file
#[derive(Debug, Serialize, Deserialize)]
pub struct MemIndex {
    /// Index version
    pub version: String,

    /// User identification and context
    pub user: UserContext,

    /// All memory blocks with metadata
    pub blocks: HashMap<String, BlockMeta>,

    /// Active projects and their relationships
    pub projects: HashMap<String, ProjectInfo>,

    /// Concept graph - relationships between ideas
    pub concepts: ConceptGraph,

    /// Current session context
    pub session: SessionContext,

    /// Statistics and metadata
    pub stats: IndexStats,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UserContext {
    /// User identifier (name or handle)
    pub name: String,

    /// Quick preference flags (loaded from prefs/user_flags.json)
    pub flags: HashMap<String, bool>,

    /// Style preferences (loaded from prefs/style.json)
    pub style: StylePrefs,

    /// Communication tone (loaded from prefs/tone.json)
    pub tone: TonePrefs,

    /// Current working directory preference
    pub preferred_cwd: Option<PathBuf>,

    /// Active project (if any)
    pub active_project: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct StylePrefs {
    /// Output style: terse, normal, verbose
    pub verbosity: String,

    /// Prefers bullet points
    pub bullet_preference: bool,

    /// ASCII over emoji
    pub ascii_preferred: bool,

    /// Code style preferences
    pub code_style: HashMap<String, String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TonePrefs {
    /// Humor level 0-10
    pub humor_level: u8,

    /// Warning verbosity
    pub warning_style: String, // "minimal", "normal", "detailed"

    /// Explanation depth
    pub explanation_depth: String, // "eli5", "normal", "expert"

    /// Encouragement style
    pub encouragement: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct BlockMeta {
    /// Filename in blocks/ directory
    pub filename: String,

    /// When this block was created
    pub created: DateTime<Utc>,

    /// Last accessed time
    pub last_accessed: DateTime<Utc>,

    /// Size in bytes
    pub size: usize,

    /// Number of messages/entries
    pub entry_count: usize,

    /// Key topics/concepts in this block
    pub topics: Vec<String>,

    /// Related projects
    pub projects: Vec<String>,

    /// Quick summary
    pub summary: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ProjectInfo {
    /// Project name
    pub name: String,

    /// Project root path
    pub path: PathBuf,

    /// Current status
    pub status: String, // "active", "paused", "completed"

    /// Technologies used
    pub tech_stack: Vec<String>,

    /// Related memory blocks
    pub memory_blocks: Vec<String>,

    /// Current focus/task
    pub current_focus: Option<String>,

    /// Key decisions/notes
    pub notes: Vec<String>,

    /// Last activity
    pub last_activity: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ConceptGraph {
    /// Concept -> Related concepts with weight
    pub relationships: HashMap<String, Vec<(String, f32)>>,

    /// Concept -> Memory blocks containing it
    pub concept_blocks: HashMap<String, Vec<String>>,

    /// Recent concepts (for quick access)
    pub recent: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SessionContext {
    /// Current session ID
    pub session_id: String,

    /// Session start time
    pub started: DateTime<Utc>,

    /// Topics discussed this session
    pub topics: Vec<String>,

    /// Files/directories accessed
    pub accessed_paths: Vec<PathBuf>,

    /// Tools used
    pub tools_used: Vec<String>,

    /// Nudges given (what we suggested)
    pub nudges: Vec<Nudge>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Nudge {
    /// What was suggested
    pub suggestion: String,

    /// Why it was suggested
    pub reason: String,

    /// When it was suggested
    pub timestamp: DateTime<Utc>,

    /// Was it accepted/rejected/ignored
    pub response: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct IndexStats {
    /// Total memory blocks
    pub total_blocks: usize,

    /// Total size of all blocks
    pub total_size: usize,

    /// Total conversations
    pub total_conversations: usize,

    /// Index created date
    pub created: DateTime<Utc>,

    /// Last updated
    pub last_updated: DateTime<Utc>,

    /// Compression ratio achieved
    pub avg_compression_ratio: f32,
}

impl MemIndex {
    /// Load the index from ~/.mem8/memindex.json
    pub fn load() -> Result<Self> {
        let path = Self::index_path()?;

        if path.exists() {
            let content = fs::read_to_string(&path)?;
            let mut index: MemIndex = serde_json::from_str(&content)?;

            // Load user preferences
            index.load_user_prefs()?;

            Ok(index)
        } else {
            Ok(Self::new())
        }
    }

    /// Create a new index
    pub fn new() -> Self {
        Self {
            version: "1.0.0".to_string(),
            user: UserContext {
                name: whoami::username(),
                flags: HashMap::new(),
                style: StylePrefs {
                    verbosity: "normal".to_string(),
                    bullet_preference: true,
                    ascii_preferred: false,
                    code_style: HashMap::new(),
                },
                tone: TonePrefs {
                    humor_level: 5,
                    warning_style: "normal".to_string(),
                    explanation_depth: "normal".to_string(),
                    encouragement: true,
                },
                preferred_cwd: None,
                active_project: None,
            },
            blocks: HashMap::new(),
            projects: HashMap::new(),
            concepts: ConceptGraph {
                relationships: HashMap::new(),
                concept_blocks: HashMap::new(),
                recent: Vec::new(),
            },
            session: SessionContext {
                session_id: uuid::Uuid::new_v4().to_string(),
                started: Utc::now(),
                topics: Vec::new(),
                accessed_paths: Vec::new(),
                tools_used: Vec::new(),
                nudges: Vec::new(),
            },
            stats: IndexStats {
                total_blocks: 0,
                total_size: 0,
                total_conversations: 0,
                created: Utc::now(),
                last_updated: Utc::now(),
                avg_compression_ratio: 0.0,
            },
        }
    }

    /// Save the index
    pub fn save(&self) -> Result<()> {
        let path = Self::index_path()?;

        // Ensure directory exists
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }

        // Save main index
        let content = serde_json::to_string_pretty(self)?;
        fs::write(&path, content)?;

        // Save user preferences
        self.save_user_prefs()?;

        Ok(())
    }

    /// Get index file path
    fn index_path() -> Result<PathBuf> {
        let home = dirs::home_dir().context("Could not find home directory")?;
        Ok(home.join(".mem8").join("memindex.json"))
    }

    /// Load user preferences from separate files
    fn load_user_prefs(&mut self) -> Result<()> {
        let mem8_dir = dirs::home_dir()
            .context("Could not find home directory")?
            .join(".mem8");

        // Load user flags
        let flags_path = mem8_dir.join("prefs").join("user_flags.json");
        if flags_path.exists() {
            let content = fs::read_to_string(&flags_path)?;
            self.user.flags = serde_json::from_str(&content)?;
        }

        // Load style preferences
        let style_path = mem8_dir.join("prefs").join("style.json");
        if style_path.exists() {
            let content = fs::read_to_string(&style_path)?;
            self.user.style = serde_json::from_str(&content)?;
        }

        // Load tone preferences
        let tone_path = mem8_dir.join("prefs").join("tone.json");
        if tone_path.exists() {
            let content = fs::read_to_string(&tone_path)?;
            self.user.tone = serde_json::from_str(&content)?;
        }

        Ok(())
    }

    /// Save user preferences to separate files
    fn save_user_prefs(&self) -> Result<()> {
        let prefs_dir = dirs::home_dir()
            .context("Could not find home directory")?
            .join(".mem8")
            .join("prefs");

        fs::create_dir_all(&prefs_dir)?;

        // Save user flags
        let flags_content = serde_json::to_string_pretty(&self.user.flags)?;
        fs::write(prefs_dir.join("user_flags.json"), flags_content)?;

        // Save style
        let style_content = serde_json::to_string_pretty(&self.user.style)?;
        fs::write(prefs_dir.join("style.json"), style_content)?;

        // Save tone
        let tone_content = serde_json::to_string_pretty(&self.user.tone)?;
        fs::write(prefs_dir.join("tone.json"), tone_content)?;

        Ok(())
    }

    /// Register a new memory block
    pub fn register_block(&mut self, filename: &str, path: &Path) -> Result<()> {
        let metadata = fs::metadata(path)?;

        let block_meta = BlockMeta {
            filename: filename.to_string(),
            created: Utc::now(),
            last_accessed: Utc::now(),
            size: metadata.len() as usize,
            entry_count: 0, // Would be extracted from .m8 file
            topics: Vec::new(),
            projects: Vec::new(),
            summary: format!("Memory block: {}", filename),
        };

        self.blocks.insert(filename.to_string(), block_meta);
        self.stats.total_blocks = self.blocks.len();
        self.stats.total_size = self.blocks.values().map(|b| b.size).sum();
        self.stats.last_updated = Utc::now();

        Ok(())
    }

    /// Add or update a project
    pub fn update_project(&mut self, name: &str, path: PathBuf) {
        let project = self
            .projects
            .entry(name.to_string())
            .or_insert_with(|| ProjectInfo {
                name: name.to_string(),
                path: path.clone(),
                status: "active".to_string(),
                tech_stack: Vec::new(),
                memory_blocks: Vec::new(),
                current_focus: None,
                notes: Vec::new(),
                last_activity: Utc::now(),
            });

        project.last_activity = Utc::now();
        self.stats.last_updated = Utc::now();
    }

    /// Record a nudge given to the user
    pub fn add_nudge(&mut self, suggestion: &str, reason: &str) {
        self.session.nudges.push(Nudge {
            suggestion: suggestion.to_string(),
            reason: reason.to_string(),
            timestamp: Utc::now(),
            response: None,
        });
    }

    /// Update concept relationships
    pub fn add_concept_relation(&mut self, concept1: &str, concept2: &str, weight: f32) {
        self.concepts
            .relationships
            .entry(concept1.to_string())
            .or_insert_with(Vec::new)
            .push((concept2.to_string(), weight));

        self.concepts
            .relationships
            .entry(concept2.to_string())
            .or_insert_with(Vec::new)
            .push((concept1.to_string(), weight));
    }

    /// Write daily journal entry
    pub fn write_journal_entry(&self, content: &str) -> Result<()> {
        let journal_dir = dirs::home_dir()
            .context("Could not find home directory")?
            .join(".mem8")
            .join("journal");

        fs::create_dir_all(&journal_dir)?;

        let today = Local::now().format("%Y-%m-%d");
        let journal_path = journal_dir.join(format!("{}.ctx.md", today));

        // Append to existing or create new
        let mut existing = if journal_path.exists() {
            fs::read_to_string(&journal_path)?
        } else {
            format!("# Memory Journal - {}\n\n", today)
        };

        existing.push_str(&format!(
            "\n## {} - Session {}\n\n",
            Local::now().format("%H:%M"),
            &self.session.session_id[..8]
        ));
        existing.push_str(content);
        existing.push_str("\n");

        fs::write(&journal_path, existing)?;

        Ok(())
    }
}
