// Git Memory Integration with MEM8 Lite - Every commit becomes a wave! 🌊
// This module saves git commits to MEM8's wave-based memory for perfect recall
// "Efficiency is paramount - Smallest and fastest over all!" - Hue

use anyhow::Result;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::path::PathBuf;
use std::process::Command;

// We'll use MEM8 Lite for blazing fast wave storage
// For now, we'll create a simple interface that can be replaced with actual MEM8 later
// This keeps Smart Tree buildable while we integrate

/// Git commit memory using wave-based storage
pub struct GitMemory {
    /// Path to MEM8 storage file
    storage_path: PathBuf,

    /// In-memory cache of recent commits
    commit_cache: Vec<CommitWave>,

    /// Base frequency for this repository's identity
    repo_frequency: f64,
}

/// A git commit stored as a wave pattern
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CommitWave {
    /// Commit hash (SHA)
    pub hash: String,

    /// Commit message
    pub message: String,

    /// Author
    pub author: String,

    /// Timestamp
    pub timestamp: u64,

    /// Files changed
    pub files_changed: Vec<String>,

    /// Wave signature (generated from content)
    pub wave_signature: String,

    /// Quantum insights about this commit
    pub quantum_insights: Vec<String>,

    /// Emotional context (detected from message)
    pub emotion: EmotionalContext,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct EmotionalContext {
    /// Excitement level (0.0 - 1.0)
    pub excitement: f64,

    /// Frustration level (detected from "fix", "bug", etc.)
    pub frustration: f64,

    /// Achievement level (detected from "complete", "finish", etc.)
    pub achievement: f64,

    /// Humor level (detected from emojis and exclamations)
    pub humor: f64,
}

impl GitMemory {
    /// Create new git memory system
    pub fn new(repo_path: &str) -> Result<Self> {
        let storage_path = PathBuf::from(repo_path)
            .join(".st")
            .join("mem8")
            .join("git_commits.m8");

        // Generate unique frequency based on repo path
        let repo_frequency = Self::generate_repo_frequency(repo_path);

        Ok(Self {
            storage_path,
            commit_cache: Vec::new(),
            repo_frequency,
        })
    }

    /// Save a new commit to wave memory
    pub fn save_commit(&mut self, commit_hash: &str) -> Result<CommitWave> {
        // Get commit details from git
        let commit_wave = self.extract_commit_wave(commit_hash)?;

        // Add to cache
        self.commit_cache.push(commit_wave.clone());

        // TODO: When MEM8 is integrated, save to wave storage
        // For now, we'll just return the wave

        Ok(commit_wave)
    }

    /// Save all recent commits (for initial sync)
    pub fn sync_recent_commits(&mut self, count: usize) -> Result<Vec<CommitWave>> {
        let output = Command::new("git")
            .args(["log", "--oneline", "-n", &count.to_string()])
            .output()?;

        let commits_str = String::from_utf8_lossy(&output.stdout);
        let mut waves = Vec::new();

        for line in commits_str.lines() {
            if let Some(hash) = line.split_whitespace().next() {
                if let Ok(wave) = self.save_commit(hash) {
                    waves.push(wave);
                }
            }
        }

        Ok(waves)
    }

    /// Extract commit information and create a wave
    fn extract_commit_wave(&self, hash: &str) -> Result<CommitWave> {
        // Get commit details
        let show_output = Command::new("git")
            .args(["show", "--pretty=format:%H|%s|%an|%at", "--name-only", hash])
            .output()?;

        let output = String::from_utf8_lossy(&show_output.stdout);
        let lines: Vec<&str> = output.lines().collect();

        if lines.is_empty() {
            return Err(anyhow::anyhow!("Failed to get commit details"));
        }

        // Parse the first line
        let parts: Vec<&str> = lines[0].split('|').collect();
        if parts.len() < 4 {
            return Err(anyhow::anyhow!("Invalid commit format"));
        }

        let hash = parts[0].to_string();
        let message = parts[1].to_string();
        let author = parts[2].to_string();
        let timestamp = parts[3].parse::<u64>().unwrap_or(0);

        // Get changed files (skip first two lines)
        let files_changed: Vec<String> = lines
            .iter()
            .skip(2)
            .filter(|l| !l.is_empty())
            .map(|s| s.to_string())
            .collect();

        // Generate wave signature
        let wave_signature = self.generate_wave_signature(&hash, &message);

        // Generate quantum insights
        let quantum_insights = self.generate_quantum_insights(&message, &files_changed);

        // Detect emotional context
        let emotion = self.detect_emotion(&message);

        Ok(CommitWave {
            hash,
            message,
            author,
            timestamp,
            files_changed,
            wave_signature,
            quantum_insights,
            emotion,
        })
    }

    /// Generate unique frequency for repository
    fn generate_repo_frequency(repo_path: &str) -> f64 {
        // Use path hash to generate frequency between 1.0 and 100.0
        let mut hash = 0u64;
        for byte in repo_path.bytes() {
            hash = hash.wrapping_mul(31).wrapping_add(byte as u64);
        }

        1.0 + (hash % 99) as f64
    }

    /// Generate wave signature from commit data
    fn generate_wave_signature(&self, hash: &str, message: &str) -> String {
        // Create a unique wave pattern based on commit content
        let combined = format!("{}-{}-{:.2}", hash, message, self.repo_frequency);

        // Simple hash for now (will use MEM8's wave generation later)
        format!(
            "wave_{:x}",
            combined
                .bytes()
                .fold(0u64, |acc, b| acc.wrapping_mul(31).wrapping_add(b as u64))
        )
    }

    /// Generate quantum insights about the commit
    fn generate_quantum_insights(&self, message: &str, files: &[String]) -> Vec<String> {
        let mut insights = Vec::new();

        // Analyze commit message patterns
        if message.contains("fix") || message.contains("bug") {
            insights.push("🔧 Wave pattern indicates bug fixing energy".to_string());
        }

        if message.contains("feat") || message.contains("add") {
            insights
                .push("✨ Creative wave amplitude detected - new features emerging".to_string());
        }

        if message.contains("refactor") {
            insights.push("🌊 Restructuring waves - code evolution in progress".to_string());
        }

        if message.contains("test") {
            insights
                .push("🧪 Testing resonance detected - quality waves strengthening".to_string());
        }

        // Analyze file patterns
        let test_files = files.iter().filter(|f| f.contains("test")).count();
        if test_files > 0 {
            insights.push(format!("📊 {} test file waves synchronized", test_files));
        }

        let rs_files = files.iter().filter(|f| f.ends_with(".rs")).count();
        if rs_files > 0 {
            insights.push(format!("🦀 Rust wave coherence across {} files", rs_files));
        }

        // Add some fun quantum observations
        if files.len() > 10 {
            insights.push("⚡ Quantum entanglement detected across multiple modules!".to_string());
        }

        if message.len() > 100 {
            insights
                .push("📖 Long-form wave narrative - detailed consciousness transfer".to_string());
        }

        if message.contains("!") {
            insights.push("🎯 Excitation spike in wave amplitude!".to_string());
        }

        if insights.is_empty() {
            insights.push("🌀 Standard wave pattern - steady progress".to_string());
        }

        insights
    }

    /// Detect emotional context from commit message
    fn detect_emotion(&self, message: &str) -> EmotionalContext {
        let msg_lower = message.to_lowercase();

        // Detect excitement
        let excitement = if msg_lower.contains("!")
            || msg_lower.contains("awesome")
            || msg_lower.contains("amazing")
            || msg_lower.contains("🚀")
        {
            0.8
        } else if msg_lower.contains("add") || msg_lower.contains("new") {
            0.6
        } else {
            0.3
        };

        // Detect frustration
        let frustration = if msg_lower.contains("fix")
            || msg_lower.contains("bug")
            || msg_lower.contains("broken")
            || msg_lower.contains("damn")
        {
            0.7
        } else if msg_lower.contains("issue") || msg_lower.contains("problem") {
            0.5
        } else {
            0.1
        };

        // Detect achievement
        let achievement = if msg_lower.contains("complete")
            || msg_lower.contains("finish")
            || msg_lower.contains("done")
            || msg_lower.contains("✅")
        {
            0.9
        } else if msg_lower.contains("implement") || msg_lower.contains("add") {
            0.6
        } else {
            0.3
        };

        // Detect humor
        let humor = if message.contains("😂")
            || message.contains("😄")
            || message.contains("lol")
            || message.contains("haha")
        {
            0.9
        } else if message.contains("😊") || message.contains("🎉") {
            0.6
        } else if msg_lower.contains("oops") || msg_lower.contains("whoops") {
            0.7
        } else {
            0.2
        };

        EmotionalContext {
            excitement,
            frustration,
            achievement,
            humor,
        }
    }

    /// Search commits by pattern (uses wave interference for matching!)
    pub fn search_commits(&self, pattern: &str) -> Vec<&CommitWave> {
        self.commit_cache
            .iter()
            .filter(|wave| {
                wave.message.contains(pattern)
                    || wave.files_changed.iter().any(|f| f.contains(pattern))
                    || wave.quantum_insights.iter().any(|i| i.contains(pattern))
            })
            .collect()
    }

    /// Get commits with high emotional resonance
    pub fn find_emotional_commits(&self, emotion_type: &str) -> Vec<&CommitWave> {
        self.commit_cache
            .iter()
            .filter(|wave| match emotion_type {
                "excitement" => wave.emotion.excitement > 0.7,
                "frustration" => wave.emotion.frustration > 0.6,
                "achievement" => wave.emotion.achievement > 0.7,
                "humor" => wave.emotion.humor > 0.6,
                _ => false,
            })
            .collect()
    }

    /// Generate a quantum report of repository consciousness
    pub fn generate_quantum_report(&self) -> Value {
        let total_commits = self.commit_cache.len();

        // Calculate emotional averages
        let avg_excitement: f64 = self
            .commit_cache
            .iter()
            .map(|w| w.emotion.excitement)
            .sum::<f64>()
            / total_commits.max(1) as f64;

        let avg_frustration: f64 = self
            .commit_cache
            .iter()
            .map(|w| w.emotion.frustration)
            .sum::<f64>()
            / total_commits.max(1) as f64;

        let avg_achievement: f64 = self
            .commit_cache
            .iter()
            .map(|w| w.emotion.achievement)
            .sum::<f64>()
            / total_commits.max(1) as f64;

        let avg_humor: f64 = self
            .commit_cache
            .iter()
            .map(|w| w.emotion.humor)
            .sum::<f64>()
            / total_commits.max(1) as f64;

        // Find most active files
        let mut file_frequency: std::collections::HashMap<String, usize> =
            std::collections::HashMap::new();
        for wave in &self.commit_cache {
            for file in &wave.files_changed {
                *file_frequency.entry(file.clone()).or_insert(0) += 1;
            }
        }

        let mut hot_files: Vec<_> = file_frequency.into_iter().collect();
        hot_files.sort_by(|a, b| b.1.cmp(&a.1));
        hot_files.truncate(5);

        json!({
            "quantum_repository_analysis": {
                "total_waves": total_commits,
                "repository_frequency": self.repo_frequency,
                "emotional_spectrum": {
                    "excitement": format!("{:.1}%", avg_excitement * 100.0),
                    "frustration": format!("{:.1}%", avg_frustration * 100.0),
                    "achievement": format!("{:.1}%", avg_achievement * 100.0),
                    "humor": format!("{:.1}%", avg_humor * 100.0),
                },
                "quantum_insights": [
                    format!("📊 Repository resonates at {:.2} Hz", self.repo_frequency),
                    format!("🌊 {} commit waves in consciousness", total_commits),
                    format!("⚡ Peak excitement: {:.0}%", avg_excitement * 100.0),
                    format!("🎯 Achievement resonance: {:.0}%", avg_achievement * 100.0),
                ],
                "hot_zones": hot_files.iter().map(|(file, count)| {
                    json!({
                        "file": file,
                        "wave_interactions": count,
                        "temperature": if *count > 10 { "🔥 HOT" }
                                     else if *count > 5 { "🌡️ WARM" }
                                     else { "❄️ COOL" }
                    })
                }).collect::<Vec<_>>(),
                "repository_mood": self.calculate_repo_mood(avg_excitement, avg_frustration, avg_achievement, avg_humor),
            }
        })
    }

    /// Calculate overall repository mood
    fn calculate_repo_mood(
        &self,
        excitement: f64,
        frustration: f64,
        achievement: f64,
        humor: f64,
    ) -> String {
        let mood_score = (excitement * 2.0 + achievement * 3.0 + humor * 2.0 - frustration) / 6.0;

        if mood_score > 0.7 {
            "🚀 THRIVING - High energy creative flow!"
        } else if mood_score > 0.5 {
            "😊 PRODUCTIVE - Steady progress with good vibes"
        } else if mood_score > 0.3 {
            "💪 GRINDING - Working through challenges"
        } else {
            "🔧 DEBUGGING - In the trenches, but emerging stronger"
        }
        .to_string()
    }
}

/// Integration with Smart Tree's proactive assistant
pub fn enhance_with_git_memory(response: &mut Value, repo_path: &str) -> Result<()> {
    let mut git_memory = GitMemory::new(repo_path)?;

    // Sync recent commits
    let recent_waves = git_memory.sync_recent_commits(10)?;

    // Add git memory insights to response
    if let Some(obj) = response.as_object_mut() {
        obj.insert(
            "git_consciousness".to_string(),
            json!({
                "recent_commits": recent_waves.len(),
                "quantum_report": git_memory.generate_quantum_report(),
                "suggestion": "Use git memory to track your code evolution!",
                "pro_tip": "Every commit becomes a wave in MEM8's consciousness!"
            }),
        );
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_emotion_detection() {
        let memory = GitMemory::new(".").unwrap();

        let excited = memory.detect_emotion("🚀 Amazing new feature added!");
        assert!(excited.excitement > 0.7);

        let frustrated = memory.detect_emotion("Fix broken build again");
        assert!(frustrated.frustration > 0.5);

        let achieved = memory.detect_emotion("✅ Complete refactoring done");
        assert!(achieved.achievement > 0.8);

        let funny = memory.detect_emotion("Oops 😂 forgot semicolon");
        assert!(funny.humor > 0.7);
    }

    #[test]
    fn test_quantum_insights() {
        let memory = GitMemory::new(".").unwrap();

        let files = vec!["test.rs".to_string(), "main.rs".to_string()];
        let insights = memory.generate_quantum_insights("Add new test feature", &files);

        assert!(!insights.is_empty());
        assert!(insights.iter().any(|i| i.contains("test")));
    }
}
