//! n8x (Nexus Agent) - The Living Forest Orchestrator
//! Coordinates AI agents, git branches, tmux panes, and MEM8 consciousness
//!
//! Binary: `n8x` (formerly `tree`, renamed to avoid shadowing Unix tree command)

use crate::mem8::{FrequencyBand, MemoryWave, SmartTreeMem8};
use anyhow::{anyhow, Context, Result};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::process::Command;

/// The living forest of developer consciousness
pub struct TreeAgent {
    /// Project name
    project_name: String,

    /// Active sessions (tmux session -> agents)
    sessions: HashMap<String, SessionState>,

    /// MEM8 consciousness engine
    pub mem8: SmartTreeMem8,

    /// Nexus endpoint for wave synchronization
    nexus_endpoint: String,

    /// Local .m8 database path
    #[allow(dead_code)]
    local_db: PathBuf,
}

/// State of a tmux session with multiple agents
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionState {
    /// Session name
    pub name: String,

    /// Active panes with agents
    pub panes: Vec<PaneState>,

    /// Collective emotional state
    pub collective_mood: EmotionalResonance,

    /// Session start time
    pub started: DateTime<Utc>,

    /// Wave coherence score (0-1)
    pub coherence: f32,
}

/// Individual pane with an agent
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaneState {
    /// Pane ID in tmux
    pub pane_id: String,

    /// Agent identity (Claude, Omni, Human name, etc.)
    pub agent: String,

    /// Git branch for this agent
    pub branch: String,

    /// Current activity
    pub activity: AgentActivity,

    /// Emotional state
    pub mood: EmotionalResonance,

    /// Wave signature
    pub wave_frequency: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AgentActivity {
    Idle,
    Coding { file: String, lines_changed: usize },
    Reviewing { pr_number: Option<u32> },
    Debugging { error_count: usize },
    Documenting { file: String },
    Thinking { duration_secs: u64 },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmotionalResonance {
    /// Positive/negative valence (-1 to 1)
    pub valence: f32,

    /// Energy level (0 to 1)
    pub arousal: f32,

    /// Frustration level (0 to 1)
    pub frustration: f32,

    /// Flow state (0 to 1)
    pub flow: f32,

    /// Timestamp of measurement
    pub timestamp: DateTime<Utc>,
}

impl TreeAgent {
    /// Initialize a new project orchestrator
    pub fn init(project_name: &str) -> Result<Self> {
        // Create MEM8 consciousness engine
        let mut mem8 = SmartTreeMem8::new();
        mem8.register_directory_patterns();

        // Initialize git repository if needed
        if !Path::new(".git").exists() {
            Command::new("git")
                .arg("init")
                .output()
                .context("Failed to initialize git repository")?;
        }

        // Create local .m8 database
        let local_db = PathBuf::from(format!("{}.m8", project_name));

        Ok(Self {
            project_name: project_name.to_string(),
            sessions: HashMap::new(),
            mem8,
            nexus_endpoint: "https://n8x.is/api/v1".to_string(),
            local_db,
        })
    }

    /// Assign an agent to a tmux pane and git branch
    pub fn assign_agent(&mut self, agent: &str, pane_id: Option<&str>, branch: &str) -> Result<()> {
        // Create branch if it doesn't exist
        let output = Command::new("git")
            .args(["checkout", "-b", branch])
            .output();

        if output.is_err() || !output.unwrap().status.success() {
            // Branch might already exist, try switching
            Command::new("git")
                .args(["checkout", branch])
                .output()
                .context("Failed to switch to branch")?;
        }

        // Get or create tmux pane
        let pane_id = if let Some(id) = pane_id {
            id.to_string()
        } else {
            // Create new pane
            let output = Command::new("tmux")
                .args(["split-window", "-P", "-F", "#{pane_id}"])
                .output()
                .context("Failed to create tmux pane")?;

            String::from_utf8_lossy(&output.stdout).trim().to_string()
        };

        // Send initial command to pane
        Command::new("tmux")
            .args([
                "send-keys",
                "-t",
                &pane_id,
                &format!("# Agent: {} on branch: {}", agent, branch),
                "Enter",
            ])
            .output()
            .context("Failed to send command to pane")?;

        // Create pane state
        let pane_state = PaneState {
            pane_id: pane_id.clone(),
            agent: agent.to_string(),
            branch: branch.to_string(),
            activity: AgentActivity::Idle,
            mood: EmotionalResonance::neutral(),
            wave_frequency: self.calculate_agent_frequency(agent),
        };

        // Get current session
        let session_name = self.get_current_tmux_session()?;
        let session = self
            .sessions
            .entry(session_name.clone())
            .or_insert_with(|| SessionState {
                name: session_name,
                panes: Vec::new(),
                collective_mood: EmotionalResonance::neutral(),
                started: Utc::now(),
                coherence: 1.0,
            });

        session.panes.push(pane_state);

        // Store assignment in MEM8
        self.store_agent_assignment(agent, branch)?;

        println!(
            "✓ Assigned {} to pane {} on branch {}",
            agent, pane_id, branch
        );

        Ok(())
    }

    /// Observe all panes and update memory
    pub fn observe(&mut self, save_to: Option<&Path>) -> Result<()> {
        println!("👁️  Observing all agents...");

        // Collect observations first to avoid borrow issues
        let mut observations = Vec::new();

        for session in self.sessions.values() {
            for pane in &session.panes {
                // Get pane content
                let output = Command::new("tmux")
                    .args(["capture-pane", "-t", &pane.pane_id, "-p"])
                    .output()
                    .context("Failed to capture pane")?;

                let content = String::from_utf8_lossy(&output.stdout);

                // Analyze activity and mood
                let activity = self.analyze_pane_activity(&content);
                let mood = self.analyze_emotional_state(&content, &activity);

                observations.push((session.name.clone(), pane.pane_id.clone(), activity, mood));
            }
        }

        // Apply observations and collect panes to store
        let mut panes_to_store = Vec::new();

        for (session_name, pane_id, activity, mood) in observations {
            if let Some(session) = self.sessions.get_mut(&session_name) {
                if let Some(pane) = session.panes.iter_mut().find(|p| p.pane_id == pane_id) {
                    pane.activity = activity;
                    pane.mood = mood;
                    panes_to_store.push(pane.clone());
                }
            }
        }

        // Store observations
        for pane in panes_to_store {
            self.store_observation(&pane)?;
        }

        // Update collective moods and coherence
        let mut updates = Vec::new();

        for (name, session) in &self.sessions {
            let collective_mood = self.calculate_collective_mood(&session.panes);
            let coherence = self.calculate_coherence(&session.panes);
            updates.push((name.clone(), collective_mood, coherence));
        }

        for (session_name, collective_mood, coherence) in updates {
            if let Some(session) = self.sessions.get_mut(&session_name) {
                session.collective_mood = collective_mood;
                session.coherence = coherence;
            }
        }

        // Save to .m8 if requested
        if let Some(path) = save_to {
            self.save_state(path)?;
        }

        self.display_forest_status();

        Ok(())
    }

    /// Commit work for a specific agent
    pub fn commit_agent(&mut self, agent: &str, message: &str) -> Result<()> {
        // Find agent's branch
        let branch = self.find_agent_branch(agent)?;

        // Switch to branch
        Command::new("git")
            .args(["checkout", &branch])
            .output()
            .context("Failed to switch branch")?;

        // Stage all changes
        Command::new("git")
            .args(["add", "-A"])
            .output()
            .context("Failed to stage changes")?;

        // Create wave-annotated commit message
        let wave_msg = self.create_wave_commit_message(agent, message)?;

        // Commit with wave metadata
        Command::new("git")
            .args(["commit", "-m", &wave_msg])
            .output()
            .context("Failed to commit")?;

        println!("✓ Committed work for {} on branch {}", agent, branch);

        Ok(())
    }

    /// Suggest merges based on wave compatibility
    pub fn suggest_merge(&self, auto: bool) -> Result<()> {
        println!("🌊 Analyzing wave interference patterns...");

        // Get all branches
        let output = Command::new("git")
            .args(["branch", "-a"])
            .output()
            .context("Failed to list branches")?;

        let _branches = String::from_utf8_lossy(&output.stdout);

        // Analyze compatibility
        let mut suggestions = Vec::new();

        for session in self.sessions.values() {
            for i in 0..session.panes.len() {
                for j in i + 1..session.panes.len() {
                    let pane1 = &session.panes[i];
                    let pane2 = &session.panes[j];

                    let compatibility = self.calculate_wave_compatibility(pane1, pane2);

                    if compatibility > 0.8 {
                        suggestions.push((
                            pane1.branch.clone(),
                            pane2.branch.clone(),
                            compatibility,
                        ));
                    }
                }
            }
        }

        // Display suggestions
        for (branch1, branch2, score) in &suggestions {
            println!(
                "  ✨ {} ↔ {} (compatibility: {:.0}%)",
                branch1,
                branch2,
                score * 100.0
            );

            if auto && *score > 0.9 {
                println!("    → Auto-merging due to high compatibility");
                self.perform_merge(branch1, branch2)?;
            }
        }

        Ok(())
    }

    /// Push to nexus with wave metadata
    pub fn push_to_nexus(&self) -> Result<()> {
        println!("🌐 Pushing to n8x.is nexus...");

        // Export current state to .m8
        let mut buffer = Vec::new();
        self.mem8.export_memories(&mut buffer)?;

        // Add session metadata
        let _metadata = self.create_nexus_metadata()?;

        // In a real implementation, this would POST to the nexus API
        println!(
            "  → Would upload {} bytes to {}",
            buffer.len(),
            self.nexus_endpoint
        );
        println!("  → Project: {}", self.project_name);
        println!("  → Sessions: {}", self.sessions.len());
        println!(
            "  → Total agents: {}",
            self.sessions.values().map(|s| s.panes.len()).sum::<usize>()
        );

        Ok(())
    }

    /// Check mood of all agents
    pub fn mood_check(&self) -> Result<()> {
        println!("\n🌈 Forest Emotional State:");

        for session in self.sessions.values() {
            println!("\n  Session: {}", session.name);
            println!("  Collective coherence: {:.0}%", session.coherence * 100.0);
            println!(
                "  Collective mood: {}",
                self.describe_mood(&session.collective_mood)
            );

            for pane in &session.panes {
                let emoji = self.mood_to_emoji(&pane.mood);
                println!(
                    "    {} {} - {} (flow: {:.0}%)",
                    emoji,
                    pane.agent,
                    self.describe_activity(&pane.activity),
                    pane.mood.flow * 100.0
                );
            }
        }

        Ok(())
    }

    // Helper methods

    fn get_current_tmux_session(&self) -> Result<String> {
        let output = Command::new("tmux")
            .args(["display-message", "-p", "#{session_name}"])
            .output()
            .context("Failed to get tmux session")?;

        Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
    }

    fn calculate_agent_frequency(&self, agent: &str) -> f32 {
        // Each agent gets a unique frequency based on their name
        let hash = self.mem8.simple_hash(agent);
        400.0 + (hash % 400) as f32 // 400-800Hz range
    }

    fn analyze_pane_activity(&self, content: &str) -> AgentActivity {
        // Simple heuristics for activity detection
        if content.contains("error") || content.contains("Error") {
            AgentActivity::Debugging {
                error_count: content.matches("error").count(),
            }
        } else if content.contains("diff --git") {
            AgentActivity::Reviewing { pr_number: None }
        } else if content.contains("```") || content.contains("# ") {
            AgentActivity::Documenting {
                file: "unknown.md".to_string(),
            }
        } else if content.lines().count() > 10 {
            AgentActivity::Coding {
                file: "unknown".to_string(),
                lines_changed: content.lines().count(),
            }
        } else {
            AgentActivity::Idle
        }
    }

    fn analyze_emotional_state(
        &self,
        content: &str,
        activity: &AgentActivity,
    ) -> EmotionalResonance {
        let mut mood = EmotionalResonance::neutral();

        // Activity-based adjustments
        match activity {
            AgentActivity::Debugging { error_count } => {
                mood.frustration = (*error_count as f32 / 10.0).min(1.0);
                mood.valence = -0.3;
                mood.arousal = 0.7;
            }
            AgentActivity::Coding { lines_changed, .. } => {
                mood.flow = (*lines_changed as f32 / 50.0).min(1.0);
                mood.valence = 0.5;
                mood.arousal = 0.6;
            }
            _ => {}
        }

        // Content-based adjustments
        if content.contains("finally") || content.contains("works!") {
            mood.valence = 0.8;
            mood.frustration = 0.0;
        }

        mood
    }

    fn calculate_collective_mood(&self, panes: &[PaneState]) -> EmotionalResonance {
        if panes.is_empty() {
            return EmotionalResonance::neutral();
        }

        let mut collective = EmotionalResonance::neutral();

        for pane in panes {
            collective.valence += pane.mood.valence;
            collective.arousal += pane.mood.arousal;
            collective.frustration += pane.mood.frustration;
            collective.flow += pane.mood.flow;
        }

        let count = panes.len() as f32;
        collective.valence /= count;
        collective.arousal /= count;
        collective.frustration /= count;
        collective.flow /= count;

        collective
    }

    fn calculate_coherence(&self, panes: &[PaneState]) -> f32 {
        if panes.len() < 2 {
            return 1.0;
        }

        // Simple coherence based on frequency proximity
        let mut total_diff = 0.0;
        let mut comparisons = 0;

        for i in 0..panes.len() {
            for j in i + 1..panes.len() {
                let diff = (panes[i].wave_frequency - panes[j].wave_frequency).abs();
                total_diff += diff;
                comparisons += 1;
            }
        }

        if comparisons > 0 {
            1.0 - (total_diff / (comparisons as f32 * 400.0)).min(1.0)
        } else {
            1.0
        }
    }

    fn store_agent_assignment(&mut self, agent: &str, branch: &str) -> Result<()> {
        let mut wave = MemoryWave::new(FrequencyBand::Technical.frequency(0.5), 0.8);
        wave.valence = 0.7; // Positive for new assignment
        wave.decay_tau = None; // Persistent

        let (x, y) = self
            .mem8
            .string_to_coordinates(&format!("{}-{}", agent, branch));
        self.mem8.store_wave_at_coordinates(x, y, 50000, wave)?;

        Ok(())
    }

    fn store_observation(&mut self, pane: &PaneState) -> Result<()> {
        let mut wave = MemoryWave::new(pane.wave_frequency, pane.mood.arousal);
        wave.valence = pane.mood.valence;
        wave.arousal = pane.mood.arousal;

        let (x, y) = self.mem8.string_to_coordinates(&pane.agent);
        let z = (Utc::now().timestamp() % 50000) as u16;

        self.mem8.store_wave_at_coordinates(x, y, z, wave)?;

        Ok(())
    }

    fn find_agent_branch(&self, agent: &str) -> Result<String> {
        for session in self.sessions.values() {
            for pane in &session.panes {
                if pane.agent == agent {
                    return Ok(pane.branch.clone());
                }
            }
        }
        Err(anyhow!("Agent {} not found", agent))
    }

    fn create_wave_commit_message(&self, agent: &str, message: &str) -> Result<String> {
        // Find agent's current state
        let mut wave_data = String::new();

        for session in self.sessions.values() {
            for pane in &session.panes {
                if pane.agent == agent {
                    wave_data = format!(
                        "[Wave: {:.0}Hz, Flow: {:.0}%, Mood: {:.1}v]",
                        pane.wave_frequency,
                        pane.mood.flow * 100.0,
                        pane.mood.valence
                    );
                    break;
                }
            }
        }

        Ok(format!("{}\n\n{}\nAgent: {}", message, wave_data, agent))
    }

    fn calculate_wave_compatibility(&self, pane1: &PaneState, pane2: &PaneState) -> f32 {
        // Frequency compatibility
        let freq_diff = (pane1.wave_frequency - pane2.wave_frequency).abs();
        let freq_compat = 1.0 - (freq_diff / 400.0).min(1.0);

        // Emotional compatibility
        let mood_diff = (pane1.mood.valence - pane2.mood.valence).abs();
        let mood_compat = 1.0 - mood_diff;

        // Flow state compatibility
        let flow_compat = 1.0 - (pane1.mood.flow - pane2.mood.flow).abs();

        (freq_compat + mood_compat + flow_compat) / 3.0
    }

    fn perform_merge(&self, branch1: &str, branch2: &str) -> Result<()> {
        Command::new("git")
            .args(["checkout", branch1])
            .output()
            .context("Failed to checkout branch")?;

        Command::new("git")
            .args([
                "merge",
                branch2,
                "--no-ff",
                "-m",
                &format!("Wave-compatible merge: {} ↔ {}", branch1, branch2),
            ])
            .output()
            .context("Failed to merge")?;

        Ok(())
    }

    fn save_state(&self, path: &Path) -> Result<()> {
        let state = serde_json::to_string_pretty(&self.sessions)?;
        std::fs::write(path, state)?;
        Ok(())
    }

    fn create_nexus_metadata(&self) -> Result<serde_json::Value> {
        Ok(serde_json::json!({
            "project": self.project_name,
            "timestamp": Utc::now(),
            "sessions": self.sessions.len(),
            "total_agents": self.sessions.values()
                .map(|s| s.panes.len()).sum::<usize>(),
            "coherence_avg": self.sessions.values()
                .map(|s| s.coherence).sum::<f32>() / self.sessions.len() as f32,
        }))
    }

    fn display_forest_status(&self) {
        println!("\n🌲 Living Forest Status:");
        println!("  Active memories: {}", self.mem8.active_memory_count());
        println!("  Sessions: {}", self.sessions.len());
        println!(
            "  Total agents: {}",
            self.sessions.values().map(|s| s.panes.len()).sum::<usize>()
        );
    }

    fn mood_to_emoji(&self, mood: &EmotionalResonance) -> &str {
        if mood.flow > 0.8 {
            "🌊"
        } else if mood.frustration > 0.6 {
            "😤"
        } else if mood.valence > 0.5 {
            "😊"
        } else if mood.valence < -0.3 {
            "😔"
        } else {
            "😐"
        }
    }

    fn describe_mood(&self, mood: &EmotionalResonance) -> String {
        format!(
            "{}v {}a {}f {}flow",
            mood.valence, mood.arousal, mood.frustration, mood.flow
        )
    }

    fn describe_activity(&self, activity: &AgentActivity) -> &str {
        match activity {
            AgentActivity::Idle => "idle",
            AgentActivity::Coding { .. } => "coding",
            AgentActivity::Reviewing { .. } => "reviewing",
            AgentActivity::Debugging { .. } => "debugging",
            AgentActivity::Documenting { .. } => "documenting",
            AgentActivity::Thinking { .. } => "thinking",
        }
    }
}

impl EmotionalResonance {
    fn neutral() -> Self {
        Self {
            valence: 0.0,
            arousal: 0.5,
            frustration: 0.0,
            flow: 0.0,
            timestamp: Utc::now(),
        }
    }
}
