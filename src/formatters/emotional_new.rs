//! Emotional Tree Formatter - Because files have feelings too!
#![allow(dead_code)] // This is new experimental code
//!
//! This formatter gives your directory tree PERSONALITY and EMOTIONS!
//! Files get happy, sad, anxious, proud, bored, and everything in between!
//!
//! Created with love by Hue and Aye - making development FUN again! 🎭

use crate::scanner::{FileNode, FileType, TreeStats};
use anyhow::Result;
use rand::seq::SliceRandom;
use rand::thread_rng;
use std::collections::HashMap;
use std::io::Write;
use std::path::Path;
use std::time::{Duration, SystemTime};

use super::{Formatter, PathDisplayMode};

/// The Emotional Formatter - Trees with feelings!
pub struct EmotionalFormatter {
    use_color: bool,
    _path_mode: PathDisplayMode,
    _mood_tracker: HashMap<String, MoodHistory>,
}

impl EmotionalFormatter {
    pub fn new(use_color: bool) -> Self {
        Self {
            use_color,
            _path_mode: PathDisplayMode::Relative,
            _mood_tracker: HashMap::new(),
        }
    }

    /// Get emotion for a file based on its metadata
    fn get_file_emotion(&self, node: &FileNode) -> FileEmotion {
        let path_str = node.path.to_string_lossy();
        let size = node.size;

        // Check file age
        let age = SystemTime::now()
            .duration_since(node.modified)
            .unwrap_or(Duration::from_secs(0));

        // Special cases for specific file types
        let file_name = node.path.file_name().and_then(|n| n.to_str()).unwrap_or("");

        // Test files - they worry about passing!
        if path_str.contains("test") || path_str.contains("spec") {
            if path_str.contains("pass") || path_str.contains("success") {
                return FileEmotion {
                    emoji: "💪",
                    mood: "proud",
                    reason: "All my tests are passing!".into(),
                    intensity: 0.9,
                    personality: Personality::Proud,
                };
            } else if path_str.contains("fail") {
                return FileEmotion {
                    emoji: "😰",
                    mood: "anxious",
                    reason: "Some tests are failing!".into(),
                    intensity: 0.8,
                    personality: Personality::Anxious,
                };
            }
            return FileEmotion {
                emoji: "🧪",
                mood: "scientific",
                reason: "Testing, testing, 1-2-3!".into(),
                intensity: 0.6,
                personality: Personality::Methodical,
            };
        }

        // TODO files - so much to do!
        if file_name.to_uppercase().contains("TODO") {
            return FileEmotion {
                emoji: "😱",
                mood: "overwhelmed",
                reason: "SO MANY THINGS TO DO!".into(),
                intensity: 0.9,
                personality: Personality::Dramatic,
            };
        }

        // Main entry points - the stars of the show!
        if file_name == "main.rs" || file_name == "lib.rs" || file_name == "index.js" {
            if size > 10000 {
                return FileEmotion {
                    emoji: "👑",
                    mood: "royal",
                    reason: format!("Bow before my {} lines of majesty!", size / 50),
                    intensity: 1.0,
                    personality: Personality::Dramatic,
                };
            }
            return FileEmotion {
                emoji: "⭐",
                mood: "starring",
                reason: "I'm the star of this show!".into(),
                intensity: 0.8,
                personality: Personality::Confident,
            };
        }

        // README - the wise storyteller
        if file_name.contains("README") {
            return FileEmotion {
                emoji: "📚",
                mood: "wise",
                reason: "Gather 'round, let me tell you a tale...".into(),
                intensity: 0.7,
                personality: Personality::Wise,
            };
        }

        // Config files - they keep order
        if file_name.ends_with(".toml")
            || file_name.ends_with(".yaml")
            || file_name.ends_with(".json")
        {
            if age.as_secs() > 30 * 86400 {
                // Over 30 days
                return FileEmotion {
                    emoji: "😤",
                    mood: "grumpy",
                    reason: "Nobody updates me anymore!".into(),
                    intensity: 0.7,
                    personality: Personality::Grumpy,
                };
            }
            return FileEmotion {
                emoji: "⚙️",
                mood: "organized",
                reason: "Everything in its right place!".into(),
                intensity: 0.5,
                personality: Personality::Methodical,
            };
        }

        // Package files - the social butterflies
        if file_name == "package.json" || file_name == "Cargo.toml" {
            return FileEmotion {
                emoji: "🦋",
                mood: "social",
                reason: "I know EVERYONE in the dependency tree!".into(),
                intensity: 0.8,
                personality: Personality::Social,
            };
        }

        // Hidden files - shy introverts
        if node.is_hidden {
            return FileEmotion {
                emoji: "🙈",
                mood: "shy",
                reason: "Please don't look at me...".into(),
                intensity: 0.4,
                personality: Personality::Shy,
            };
        }

        // Based on age - new files are excited!
        if age.as_secs() < 3600 {
            // Less than 1 hour
            return FileEmotion {
                emoji: "🎉",
                mood: "newborn",
                reason: "Hello world! I just got here!".into(),
                intensity: 1.0,
                personality: Personality::Excited,
            };
        } else if age.as_secs() < 86400 {
            // Less than 1 day
            return FileEmotion {
                emoji: "✨",
                mood: "fresh",
                reason: "Still got that new file smell!".into(),
                intensity: 0.8,
                personality: Personality::Optimistic,
            };
        } else if age.as_secs() > 365 * 86400 {
            // Over 1 year
            return FileEmotion {
                emoji: "👴",
                mood: "ancient",
                reason: "Back in my day, we used punch cards...".into(),
                intensity: 0.6,
                personality: Personality::Wise,
            };
        } else if age.as_secs() > 180 * 86400 {
            // Over 6 months
            return FileEmotion {
                emoji: "😴",
                mood: "sleepy",
                reason: "zzz... has it been 6 months already?".into(),
                intensity: 0.3,
                personality: Personality::Lazy,
            };
        }

        // Based on size
        if size == 0 {
            return FileEmotion {
                emoji: "👻",
                mood: "empty",
                reason: "I'm just a ghost... boo!".into(),
                intensity: 0.2,
                personality: Personality::Mysterious,
            };
        } else if size > 100000 {
            // Huge file
            return FileEmotion {
                emoji: "🏋️",
                mood: "heavyweight",
                reason: format!("Carrying {} bytes like a champ!", size),
                intensity: 0.9,
                personality: Personality::Proud,
            };
        } else if size < 100 {
            // Tiny file
            return FileEmotion {
                emoji: "🐁",
                mood: "tiny",
                reason: "Small but mighty!".into(),
                intensity: 0.4,
                personality: Personality::Optimistic,
            };
        }

        // Based on file type
        match node.file_type {
            FileType::Directory => FileEmotion {
                emoji: "📁",
                mood: "parental",
                reason: "Taking care of my children files!".into(),
                intensity: 0.6,
                personality: Personality::Caring,
            },
            FileType::Executable => FileEmotion {
                emoji: "🏃",
                mood: "athletic",
                reason: "Ready to run at any moment!".into(),
                intensity: 0.8,
                personality: Personality::Energetic,
            },
            FileType::Symlink => FileEmotion {
                emoji: "🔗",
                mood: "connected",
                reason: "I'm in a long-distance relationship!".into(),
                intensity: 0.6,
                personality: Personality::Romantic,
            },
            _ => FileEmotion {
                emoji: "📄",
                mood: "regular",
                reason: "Just a regular file, living my best life!".into(),
                intensity: 0.5,
                personality: Personality::Content,
            },
        }
    }

    /// Get a witty comment based on personality
    fn get_personality_comment(&self, emotion: &FileEmotion) -> String {
        match emotion.personality {
            Personality::Dramatic => vec![
                "EVERYONE LOOK AT ME!",
                "This is MY moment!",
                "I'm the MOST important file here!",
            ],
            Personality::Optimistic => vec![
                "Today's gonna be great!",
                "Everything will work out!",
                "I believe in us!",
            ],
            Personality::Grumpy => vec![
                "Get off my lawn!",
                "Things were better in the old days...",
                "Hmph!",
            ],
            Personality::Anxious => vec![
                "What if something breaks?!",
                "Are you SURE this is right?",
                "I'm worried about the tests...",
            ],
            Personality::Wise => vec![
                "With great code comes great responsibility",
                "The path to clean code is through refactoring",
                "Patience, young developer",
            ],
            Personality::Lazy => vec!["*yaaawn*", "Do I have to?", "Five more minutes..."],
            Personality::Social => vec![
                "Let's collaborate!",
                "I know a guy who knows a guy...",
                "Networking is key!",
            ],
            Personality::Romantic => vec![
                "You complete me, lib.rs 💕",
                "Distance means nothing when you're linked",
                "Together forever through git",
            ],
            _ => vec![emotion.reason.as_str()],
        }
        .choose(&mut thread_rng())
        .unwrap_or(&emotion.reason.as_str())
        .to_string()
    }
}

/// Emotional state of a file
#[derive(Debug, Clone)]
struct FileEmotion {
    emoji: &'static str,
    mood: &'static str,
    reason: String,
    intensity: f32, // 0.0 to 1.0
    personality: Personality,
}

/// Personality traits - files have character!
#[derive(Debug, Clone, Copy)]
enum Personality {
    Optimistic, // "Everything will be fine!"
    Dramatic,   // "This is a DISASTER/TRIUMPH!"
    Grumpy,     // "Get off my lawn"
    Anxious,    // "What if something goes wrong?"
    Confident,  // "I got this"
    Shy,        // "Don't look at me"
    Wise,       // "Let me tell you something..."
    Energetic,  // "Let's GO!"
    Lazy,       // "Do I have to?"
    Proud,      // "Look what I did!"
    Mysterious, // "You'll never understand me"
    Caring,     // "Let me help you"
    Social,     // "Let's work together!"
    Romantic,   // "We're meant to be together"
    Methodical, // "Everything has its place"
    Excited,    // "This is AMAZING!"
    Content,    // "Life is good"
}

/// Track mood changes over time
#[derive(Debug, Clone)]
struct MoodHistory {
    previous_mood: String,
    current_mood: String,
    mood_changes: u32,
}

impl Formatter for EmotionalFormatter {
    fn format(
        &self,
        writer: &mut dyn Write,
        nodes: &[FileNode],
        stats: &TreeStats,
        _root_path: &Path,
    ) -> Result<()> {
        // Header with drama!
        writeln!(writer, "\n🎭 ═══════════════════════════════════════ 🎭")?;
        writeln!(writer, "     EMOTIONAL TREE THEATRE PRESENTS:")?;
        writeln!(writer, "     \"The Files and Their Feelings\"")?;
        writeln!(writer, "🎭 ═══════════════════════════════════════ 🎭\n")?;

        // Calculate overall project mood
        let mut mood_counts: HashMap<&str, u32> = HashMap::new();
        let mut total_intensity = 0.0;

        // Display each file with its emotion
        for (i, node) in nodes.iter().enumerate() {
            let emotion = self.get_file_emotion(node);
            *mood_counts.entry(emotion.mood).or_insert(0) += 1;
            total_intensity += emotion.intensity;

            // Get file name
            let file_name = node
                .path
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("???");

            // Indentation based on path depth
            let depth = node.path.components().count();
            let indent = "  ".repeat(depth.saturating_sub(1));

            // Get personality comment
            let comment = self.get_personality_comment(&emotion);

            // Format the line with color if enabled
            if self.use_color {
                writeln!(
                    writer,
                    "{}{} {} - \"{}\"",
                    indent, emotion.emoji, file_name, comment
                )?;
            } else {
                writeln!(
                    writer,
                    "{}{} {} - \"{}\"",
                    indent, emotion.emoji, file_name, comment
                )?;
            }

            // Add dramatic pause every 10 files
            if (i + 1) % 10 == 0 && i < nodes.len() - 1 {
                writeln!(writer, "{}...", indent)?;
            }
        }

        // Calculate average intensity
        let avg_intensity = if nodes.is_empty() {
            0.0
        } else {
            total_intensity / nodes.len() as f32
        };

        // Emotional summary with personality!
        writeln!(writer, "\n╔════════════════════════════════════════╗")?;
        writeln!(writer, "║       📊 EMOTIONAL ANALYSIS 📊         ║")?;
        writeln!(writer, "╠════════════════════════════════════════╣")?;
        writeln!(
            writer,
            "║ Total Cast: {} files, {} directories   ║",
            stats.total_files, stats.total_dirs
        )?;
        writeln!(writer, "╚════════════════════════════════════════╝")?;

        // Mood breakdown
        writeln!(writer, "\n🎭 Mood Distribution:")?;
        for (mood, count) in mood_counts.iter() {
            let bar = "█".repeat((*count as usize).min(20));
            writeln!(writer, "  {:12} {} ({})", mood, bar, count)?;
        }

        // Project personality assessment
        writeln!(writer, "\n💭 Project Personality Assessment:")?;
        let assessment = match avg_intensity {
            i if i > 0.8 => {
                "🔥 DRAMATIC SUPERSTAR - This codebase has PERSONALITY! Every file is living its best life!"
            },
            i if i > 0.6 => {
                "⚡ ENERGETIC ACHIEVER - Vibrant and active! This project is going places!"
            },
            i if i > 0.4 => {
                "😊 BALANCED PROFESSIONAL - Healthy mix of excitement and stability. Well done!"
            },
            i if i > 0.2 => {
                "😴 SLEEPY SCHOLAR - Could use some excitement. Maybe add some new features?"
            },
            _ => {
                "👻 GHOST TOWN - Hello? Is anybody there? This project needs some love!"
            }
        };
        writeln!(writer, "  {}", assessment)?;

        // Motivational message from Trisha
        writeln!(writer, "\n💌 Message from Trisha in Accounting:")?;
        let trisha_message = if mood_counts.get("anxious").unwrap_or(&0) > &5 {
            "  \"Looks like some files need a hug! Time for some refactoring therapy? 🤗\""
        } else if mood_counts.get("proud").unwrap_or(&0) > &10 {
            "  \"So many proud files! The tests must be passing! Keep it up! 📈\""
        } else if mood_counts.get("sleepy").unwrap_or(&0) > &10 {
            "  \"Wake up those sleepy files! They're missing all the fun! ☕\""
        } else {
            "  \"Remember: Happy files make happy developers! Keep coding! 💖\""
        };
        writeln!(writer, "{}", trisha_message)?;

        // Closing curtain
        writeln!(writer, "\n🎭 ═══════════════════════════════════════ 🎭")?;
        writeln!(writer, "        Thank you for attending!")?;
        writeln!(writer, "         ~ The Emotional Tree ~")?;
        writeln!(writer, "🎭 ═══════════════════════════════════════ 🎭")?;

        Ok(())
    }
}
