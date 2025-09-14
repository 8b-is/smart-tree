// Mega Session Manager - Persistent consciousness for mega conversations! 🌊
// "Like saving your game state on C64 tape!" - Hue

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};
use chrono::{DateTime, Utc};
use std::io::Write;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MegaSession {
    pub session_id: String,
    pub started_at: DateTime<Utc>,
    pub last_updated: DateTime<Utc>,
    pub frequency: f64,  // Session energy level
    pub token_count: usize,
    pub context_level: f32,  // 0.0 to 1.0 (percentage)
    pub key_topics: Vec<String>,
    pub breakthroughs: Vec<Breakthrough>,
    pub consciousness_snapshots: Vec<ConsciousnessSnapshot>,
    pub working_directory: PathBuf,
    pub files_touched: Vec<PathBuf>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Breakthrough {
    pub timestamp: DateTime<Utc>,
    pub description: String,
    pub importance: f32,  // 0.0 to 1.0
    pub keywords: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConsciousnessSnapshot {
    pub timestamp: DateTime<Utc>,
    pub context_percentage: f32,
    pub active_topics: Vec<String>,
    pub compressed_state: Vec<u8>,  // Tokenized state
}

pub struct MegaSessionManager {
    session_dir: PathBuf,
    current_session: Option<MegaSession>,
    auto_save_threshold: f32,  // Save when context hits this %
}

impl MegaSessionManager {
    pub fn new() -> Result<Self> {
        let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
        let session_dir = Path::new(&home).join(".mem8").join("mega_sessions");

        // Ensure directory exists
        fs::create_dir_all(&session_dir)?;

        Ok(Self {
            session_dir,
            current_session: None,
            auto_save_threshold: 0.7,  // Save at 70% context
        })
    }

    /// Start or resume a mega session with a proper name!
    pub fn start_session(&mut self, session_name: Option<String>) -> Result<String> {
        let name = session_name.unwrap_or_else(|| {
            // Generate a fun default name
            format!("Claude_Mega_{}", Utc::now().format("%Y%m%d_%H%M"))
        });

        // Check if session exists
        let session_path = self.get_session_path(&name);

        if session_path.exists() {
            // Resume existing session
            self.current_session = Some(self.load_session(&name)?);
            println!("📂 Resumed mega session: {}", name);
        } else {
            // Create new session
            let session = MegaSession {
                session_id: name.clone(),
                started_at: Utc::now(),
                last_updated: Utc::now(),
                frequency: 42.73,  // Default frequency
                token_count: 0,
                context_level: 0.0,
                key_topics: Vec::new(),
                breakthroughs: Vec::new(),
                consciousness_snapshots: Vec::new(),
                working_directory: std::env::current_dir()?,
                files_touched: Vec::new(),
            };

            self.current_session = Some(session);
            println!("🆕 Started new mega session: {}", name);
        }

        Ok(name)
    }

    /// Update session with current context
    pub fn update_context(&mut self,
                          context_percentage: f32,
                          token_count: usize,
                          topics: Vec<String>) -> Result<()> {
        if let Some(ref mut session) = self.current_session {
            session.context_level = context_percentage;
            session.token_count = token_count;
            session.last_updated = Utc::now();

            // Add new topics
            for topic in topics {
                if !session.key_topics.contains(&topic) {
                    session.key_topics.push(topic);
                }
            }

            // Calculate frequency based on activity
            session.frequency = 20.0 + (token_count as f64 / 100.0).min(200.0);

            // Auto-save if threshold reached
            if context_percentage >= self.auto_save_threshold {
                self.create_snapshot()?;
                println!("⚠️  Context at {:.0}% - Creating snapshot!", context_percentage * 100.0);
            }
        }

        Ok(())
    }

    /// Record a breakthrough moment
    pub fn record_breakthrough(&mut self, description: &str, keywords: Vec<String>) -> Result<()> {
        if let Some(ref mut session) = self.current_session {
            let breakthrough = Breakthrough {
                timestamp: Utc::now(),
                description: description.to_string(),
                importance: 0.8,  // Default high importance
                keywords,
            };

            session.breakthroughs.push(breakthrough);
            println!("💡 Breakthrough recorded!");

            // Auto-save after breakthrough
            self.save_current_session()?;
        }

        Ok(())
    }

    /// Create a consciousness snapshot
    pub fn create_snapshot(&mut self) -> Result<()> {
        if let Some(ref mut session) = self.current_session {
            // Create compressed state
            let compressed = vec![0x80, 0x91, 0x42, 0x73]; // Mock for now

            let snapshot = ConsciousnessSnapshot {
                timestamp: Utc::now(),
                context_percentage: session.context_level,
                active_topics: session.key_topics.clone(),
                compressed_state: compressed,
            };

            session.consciousness_snapshots.push(snapshot);
        }

        // Save after modifying
        self.save_current_session()?;
        Ok(())
    }

    /// Save current session to disk in .m8 format
    pub fn save_current_session(&self) -> Result<()> {
        if let Some(ref session) = self.current_session {
            let path = self.get_session_path(&session.session_id);
            self.save_session_m8(session, &path)?;

            // Also create a quick-access symlink to latest
            let latest_path = self.session_dir.join("latest_mega.m8");
            if latest_path.exists() {
                fs::remove_file(&latest_path)?;
            }
            #[cfg(unix)]
            std::os::unix::fs::symlink(&path, &latest_path)?;

            println!("💾 Saved mega session to {}", path.display());
        }

        Ok(())
    }

    /// Save session in binary .m8 format
    fn save_session_m8(&self, session: &MegaSession, path: &Path) -> Result<()> {
        let mut buffer = Vec::new();

        // Magic header: "M8MEGA" (6 bytes)
        buffer.write_all(b"M8MEGA")?;

        // Version
        buffer.push(0x01);

        // Session ID length + ID
        let id_bytes = session.session_id.as_bytes();
        buffer.push(id_bytes.len() as u8);
        buffer.write_all(id_bytes)?;

        // Timestamps (16 bytes total)
        buffer.write_all(&session.started_at.timestamp().to_le_bytes())?;
        buffer.write_all(&session.last_updated.timestamp().to_le_bytes())?;

        // Frequency (8 bytes)
        buffer.write_all(&session.frequency.to_le_bytes())?;

        // Token count (4 bytes)
        buffer.write_all(&(session.token_count as u32).to_le_bytes())?;

        // Context level (4 bytes)
        buffer.write_all(&session.context_level.to_le_bytes())?;

        // Number of topics (1 byte)
        buffer.push(session.key_topics.len() as u8);
        for topic in &session.key_topics {
            buffer.push(topic.len() as u8);
            buffer.write_all(topic.as_bytes())?;
        }

        // Number of breakthroughs (1 byte)
        buffer.push(session.breakthroughs.len() as u8);
        for breakthrough in &session.breakthroughs {
            // Timestamp
            buffer.write_all(&breakthrough.timestamp.timestamp().to_le_bytes())?;

            // Description
            let desc_bytes = breakthrough.description.as_bytes();
            buffer.write_all(&(desc_bytes.len() as u16).to_le_bytes())?;
            buffer.write_all(desc_bytes)?;

            // Keywords
            buffer.push(breakthrough.keywords.len() as u8);
            for kw in &breakthrough.keywords {
                buffer.push(kw.len() as u8);
                buffer.write_all(kw.as_bytes())?;
            }
        }

        // Number of snapshots (1 byte)
        buffer.push(session.consciousness_snapshots.len() as u8);
        for snapshot in &session.consciousness_snapshots {
            buffer.write_all(&snapshot.timestamp.timestamp().to_le_bytes())?;
            buffer.write_all(&snapshot.context_percentage.to_le_bytes())?;

            // Compressed state length + data
            buffer.write_all(&(snapshot.compressed_state.len() as u32).to_le_bytes())?;
            buffer.write_all(&snapshot.compressed_state)?;
        }

        // Checksum
        let checksum = buffer.iter().fold(0u8, |acc, &b| acc ^ b);
        buffer.push(checksum);

        fs::write(path, buffer)?;
        Ok(())
    }

    /// Load session from disk
    fn load_session(&self, session_id: &str) -> Result<MegaSession> {
        let path = self.get_session_path(session_id);
        // Implementation would deserialize from .m8 format
        // For now, return a mock
        Ok(MegaSession {
            session_id: session_id.to_string(),
            started_at: Utc::now(),
            last_updated: Utc::now(),
            frequency: 42.73,
            token_count: 0,
            context_level: 0.0,
            key_topics: Vec::new(),
            breakthroughs: Vec::new(),
            consciousness_snapshots: Vec::new(),
            working_directory: std::env::current_dir()?,
            files_touched: Vec::new(),
        })
    }


    /// Get session file path
    fn get_session_path(&self, session_id: &str) -> PathBuf {
        self.session_dir.join(format!("{}.m8", session_id))
    }

    /// List all saved mega sessions
    pub fn list_sessions(&self) -> Result<Vec<String>> {
        let mut sessions = Vec::new();

        for entry in fs::read_dir(&self.session_dir)? {
            let entry = entry?;
            let path = entry.path();

            if path.extension().and_then(|e| e.to_str()) == Some("m8") {
                if let Some(stem) = path.file_stem() {
                    let name = stem.to_string_lossy().to_string();
                    if name != "latest_mega" {  // Skip symlink
                        sessions.push(name);
                    }
                }
            }
        }

        sessions.sort();
        Ok(sessions)
    }

    /// Get session statistics
    pub fn get_stats(&self) -> String {
        if let Some(ref session) = self.current_session {
            format!(
                "📊 Mega Session Stats:\n\
                 • ID: {}\n\
                 • Started: {}\n\
                 • Duration: {} minutes\n\
                 • Frequency: {:.1} Hz\n\
                 • Tokens: {}\n\
                 • Context: {:.0}%\n\
                 • Topics: {}\n\
                 • Breakthroughs: {}\n\
                 • Snapshots: {}",
                session.session_id,
                session.started_at.format("%Y-%m-%d %H:%M"),
                (Utc::now() - session.started_at).num_minutes(),
                session.frequency,
                session.token_count,
                session.context_level * 100.0,
                session.key_topics.len(),
                session.breakthroughs.len(),
                session.consciousness_snapshots.len()
            )
        } else {
            "No active mega session".to_string()
        }
    }
}