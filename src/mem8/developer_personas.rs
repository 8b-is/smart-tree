//! Developer Persona Analysis for MEM8
//! Creates unique wave signatures for each developer based on their git history

use std::collections::HashMap;
use std::path::Path;
use anyhow::Result;
use chrono::{DateTime, Utc, Timelike, Datelike};
use crate::mem8::{
    wave::{MemoryWave, FrequencyBand},
    integration::SmartTreeMem8,
    git_temporal::{GitTemporalAnalyzer, GitCommit},
};

/// Developer persona with unique characteristics
#[derive(Debug, Clone)]
pub struct DeveloperPersona {
    /// Developer name/email
    pub identity: String,
    
    /// Coding style signature
    pub style_signature: CodingStyle,
    
    /// Temporal patterns (when they work)
    pub temporal_pattern: TemporalPattern,
    
    /// Emotional profile from commit messages
    pub emotional_profile: EmotionalProfile,
    
    /// Collaboration patterns
    pub collaboration: CollaborationPattern,
    
    /// Expertise areas (files/directories they work on)
    pub expertise_map: HashMap<String, f32>,
    
    /// Overall contribution metrics
    pub metrics: ContributionMetrics,
}

#[derive(Debug, Clone)]
pub struct CodingStyle {
    /// Average commit size (lines changed)
    pub avg_commit_size: f32,
    
    /// Preference for large refactors vs small changes
    pub refactor_tendency: f32, // 0.0 = small changes, 1.0 = large refactors
    
    /// Bug fix ratio
    pub bugfix_ratio: f32,
    
    /// Feature development ratio
    pub feature_ratio: f32,
    
    /// Documentation contribution ratio
    pub documentation_ratio: f32,
    
    /// Test writing ratio
    pub test_ratio: f32,
}

#[derive(Debug, Clone)]
pub struct TemporalPattern {
    /// Preferred hours of day (0-23)
    pub active_hours: [f32; 24],
    
    /// Preferred days of week (0=Monday, 6=Sunday)
    pub active_days: [f32; 7],
    
    /// Night owl vs early bird (-1.0 = night owl, 1.0 = early bird)
    pub chronotype: f32,
    
    /// Weekend warrior score (0.0 = weekday only, 1.0 = weekend heavy)
    pub weekend_warrior: f32,
    
    /// Consistency score (0.0 = sporadic, 1.0 = very regular)
    pub consistency: f32,
}

#[derive(Debug, Clone)]
pub struct EmotionalProfile {
    /// Overall positivity in commit messages
    pub positivity: f32,
    
    /// Excitement level (exclamation marks, enthusiastic words)
    pub excitement: f32,
    
    /// Frustration level (curse words, "fix", "bug", "broken")
    pub frustration: f32,
    
    /// Professionalism (formal vs casual language)
    pub professionalism: f32,
    
    /// Humor level (jokes, puns, emojis)
    pub humor: f32,
}

#[derive(Debug, Clone)]
pub struct CollaborationPattern {
    /// Solo vs team player (0.0 = solo, 1.0 = highly collaborative)
    pub collaboration_score: f32,
    
    /// Developers they frequently work with
    pub frequent_collaborators: HashMap<String, f32>,
    
    /// Response time to others' changes
    pub responsiveness: f32,
    
    /// Code review participation
    pub review_participation: f32,
}

#[derive(Debug, Clone)]
pub struct ContributionMetrics {
    pub total_commits: usize,
    pub total_additions: usize,
    pub total_deletions: usize,
    pub files_touched: usize,
    pub first_commit: DateTime<Utc>,
    pub last_commit: DateTime<Utc>,
    pub active_days: usize,
}

/// Persona analyzer for git repositories
pub struct PersonaAnalyzer {
    analyzer: GitTemporalAnalyzer,
}

impl PersonaAnalyzer {
    pub fn new(repo_path: impl AsRef<Path>) -> Result<Self> {
        Ok(Self {
            analyzer: GitTemporalAnalyzer::new(repo_path)?,
        })
    }

    /// Analyze all developers in the repository
    pub fn analyze_all_developers(&self) -> Result<HashMap<String, DeveloperPersona>> {
        let commits = self.analyzer.get_project_timeline()?;
        
        // Group commits by author
        let mut author_commits: HashMap<String, Vec<GitCommit>> = HashMap::new();
        for commit in commits {
            author_commits.entry(commit.author.clone())
                .or_insert_with(Vec::new)
                .push(commit);
        }

        // Analyze each developer
        let mut personas = HashMap::new();
        for (author, commits) in author_commits {
            if commits.len() >= 5 { // Need at least 5 commits for meaningful analysis
                let persona = self.analyze_developer(&author, commits)?;
                personas.insert(author, persona);
            }
        }

        Ok(personas)
    }

    /// Analyze a specific developer
    fn analyze_developer(&self, identity: &str, commits: Vec<GitCommit>) -> Result<DeveloperPersona> {
        let style = self.analyze_coding_style(&commits);
        let temporal = self.analyze_temporal_pattern(&commits);
        let emotional = self.analyze_emotional_profile(&commits);
        let collaboration = self.analyze_collaboration(&commits);
        let expertise = self.analyze_expertise(&commits);
        let metrics = self.calculate_metrics(&commits);

        Ok(DeveloperPersona {
            identity: identity.to_string(),
            style_signature: style,
            temporal_pattern: temporal,
            emotional_profile: emotional,
            collaboration,
            expertise_map: expertise,
            metrics,
        })
    }

    fn analyze_coding_style(&self, commits: &[GitCommit]) -> CodingStyle {
        let total = commits.len() as f32;
        
        // Calculate average commit size
        let avg_changes: f32 = commits.iter()
            .map(|c| (c.additions + c.deletions) as f32)
            .sum::<f32>() / total;

        // Categorize commits
        let mut bugfixes = 0;
        let mut features = 0;
        let mut docs = 0;
        let mut tests = 0;
        let mut large_commits = 0;

        for commit in commits {
            let msg = commit.message.to_lowercase();
            if msg.contains("fix") || msg.contains("bug") {
                bugfixes += 1;
            }
            if msg.contains("feat") || msg.contains("add") || msg.contains("implement") {
                features += 1;
            }
            if msg.contains("doc") || msg.contains("readme") {
                docs += 1;
            }
            if msg.contains("test") || msg.contains("spec") {
                tests += 1;
            }
            if commit.additions + commit.deletions > 500 {
                large_commits += 1;
            }
        }

        CodingStyle {
            avg_commit_size: avg_changes,
            refactor_tendency: (large_commits as f32 / total).min(1.0),
            bugfix_ratio: (bugfixes as f32 / total).min(1.0),
            feature_ratio: (features as f32 / total).min(1.0),
            documentation_ratio: (docs as f32 / total).min(1.0),
            test_ratio: (tests as f32 / total).min(1.0),
        }
    }

    fn analyze_temporal_pattern(&self, commits: &[GitCommit]) -> TemporalPattern {
        let mut hour_counts = [0f32; 24];
        let mut day_counts = [0f32; 7];
        let mut morning_commits = 0;
        let mut evening_commits = 0;
        let mut weekend_commits = 0;

        for commit in commits {
            let hour = commit.timestamp.hour() as usize;
            let day = commit.timestamp.weekday().num_days_from_monday() as usize;
            
            hour_counts[hour] += 1.0;
            day_counts[day] += 1.0;

            if hour >= 5 && hour < 12 {
                morning_commits += 1;
            } else if hour >= 20 || hour < 5 {
                evening_commits += 1;
            }

            if day >= 5 { // Saturday or Sunday
                weekend_commits += 1;
            }
        }

        // Normalize
        let max_hour = hour_counts.iter().fold(0.0f32, |a, &b| a.max(b)).max(1.0);
        let max_day = day_counts.iter().fold(0.0f32, |a, &b| a.max(b)).max(1.0);
        
        for h in &mut hour_counts {
            *h /= max_hour;
        }
        for d in &mut day_counts {
            *d /= max_day;
        }

        // Calculate chronotype
        let chronotype = if evening_commits > morning_commits {
            -((evening_commits as f32) / (evening_commits + morning_commits) as f32)
        } else {
            (morning_commits as f32) / (evening_commits + morning_commits).max(1) as f32
        };

        // Calculate consistency (standard deviation of commit times)
        let commit_intervals = Self::calculate_commit_intervals(commits);
        let consistency = 1.0 / (1.0 + commit_intervals);

        TemporalPattern {
            active_hours: hour_counts,
            active_days: day_counts,
            chronotype,
            weekend_warrior: weekend_commits as f32 / commits.len() as f32,
            consistency,
        }
    }

    fn analyze_emotional_profile(&self, commits: &[GitCommit]) -> EmotionalProfile {
        let mut positivity = 0.0;
        let mut excitement = 0.0;
        let mut frustration = 0.0;
        let mut professionalism = 0.0;
        let mut humor = 0.0;

        for commit in commits {
            let msg = &commit.message;
            
            // Positivity indicators
            if msg.contains("awesome") || msg.contains("great") || msg.contains("excellent") {
                positivity += 1.0;
            }
            
            // Excitement indicators
            excitement += msg.matches('!').count() as f32;
            if msg.contains("finally") || msg.contains("yay") {
                excitement += 1.0;
            }
            
            // Frustration indicators
            if msg.contains("fix") || msg.contains("bug") || msg.contains("broken") {
                frustration += 1.0;
            }
            if msg.contains("damn") || msg.contains("crap") || msg.contains("wtf") {
                frustration += 2.0;
            }
            
            // Professionalism (longer, structured messages)
            if msg.len() > 50 && msg.contains(':') {
                professionalism += 1.0;
            }
            
            // Humor indicators
            if msg.contains("🤣") || msg.contains("😂") || msg.contains("lol") {
                humor += 1.0;
            }
        }

        let total = commits.len() as f32;
        EmotionalProfile {
            positivity: (positivity / total).min(1.0),
            excitement: (excitement / (total * 2.0)).min(1.0),
            frustration: (frustration / total).min(1.0),
            professionalism: (professionalism / total).min(1.0),
            humor: (humor / total).min(1.0),
        }
    }

    fn analyze_collaboration(&self, _commits: &[GitCommit]) -> CollaborationPattern {
        // Simplified collaboration analysis
        // In a real implementation, we'd analyze co-authored commits,
        // PR reviews, and response times
        
        CollaborationPattern {
            collaboration_score: 0.5, // Default middle ground
            frequent_collaborators: HashMap::new(),
            responsiveness: 0.5,
            review_participation: 0.5,
        }
    }

    fn analyze_expertise(&self, commits: &[GitCommit]) -> HashMap<String, f32> {
        let mut file_counts: HashMap<String, usize> = HashMap::new();
        
        for commit in commits {
            for file in &commit.files_changed {
                // Extract directory or file type as expertise area
                let expertise_key = if let Some(dir_end) = file.find('/') {
                    file[..dir_end].to_string()
                } else if let Some(ext_start) = file.rfind('.') {
                    format!("*.{}", &file[ext_start + 1..])
                } else {
                    file.clone()
                };
                
                *file_counts.entry(expertise_key).or_insert(0) += 1;
            }
        }

        // Normalize to 0-1 range
        let max_count = file_counts.values().max().copied().unwrap_or(1) as f32;
        file_counts.into_iter()
            .map(|(k, v)| (k, v as f32 / max_count))
            .collect()
    }

    fn calculate_metrics(&self, commits: &[GitCommit]) -> ContributionMetrics {
        let total_additions: usize = commits.iter().map(|c| c.additions).sum();
        let total_deletions: usize = commits.iter().map(|c| c.deletions).sum();
        
        let mut unique_files = std::collections::HashSet::new();
        for commit in commits {
            for file in &commit.files_changed {
                unique_files.insert(file.clone());
            }
        }

        ContributionMetrics {
            total_commits: commits.len(),
            total_additions,
            total_deletions,
            files_touched: unique_files.len(),
            first_commit: commits.last().map(|c| c.timestamp).unwrap_or_else(Utc::now),
            last_commit: commits.first().map(|c| c.timestamp).unwrap_or_else(Utc::now),
            active_days: Self::count_active_days(commits),
        }
    }

    fn calculate_commit_intervals(commits: &[GitCommit]) -> f32 {
        if commits.len() < 2 {
            return 0.0;
        }

        let mut intervals = Vec::new();
        for i in 1..commits.len() {
            let interval = (commits[i-1].timestamp - commits[i].timestamp).num_hours() as f32;
            intervals.push(interval);
        }

        // Calculate standard deviation
        let mean = intervals.iter().sum::<f32>() / intervals.len() as f32;
        let variance = intervals.iter()
            .map(|&x| (x - mean).powi(2))
            .sum::<f32>() / intervals.len() as f32;
        
        variance.sqrt() / (mean + 1.0) // Normalized by mean
    }

    fn count_active_days(commits: &[GitCommit]) -> usize {
        let mut active_days = std::collections::HashSet::new();
        for commit in commits {
            active_days.insert(commit.timestamp.date_naive());
        }
        active_days.len()
    }
}

/// Extension for MEM8 to create developer-specific wave patterns
impl SmartTreeMem8 {
    /// Import developer personas as unique wave signatures
    pub fn import_developer_personas(&mut self, repo_path: impl AsRef<Path>) -> Result<()> {
        let analyzer = PersonaAnalyzer::new(repo_path)?;
        let personas = analyzer.analyze_all_developers()?;
        
        println!("Found {} developer personas", personas.len());

        for (developer, persona) in personas {
            println!("\nCreating wave signature for: {}", developer);
            
            // Create base frequency from coding style
            let base_freq = if persona.style_signature.refactor_tendency > 0.5 {
                FrequencyBand::DeepStructural.frequency(0.7) // Architects
            } else if persona.style_signature.bugfix_ratio > 0.5 {
                FrequencyBand::Technical.frequency(0.8) // Fixers
            } else if persona.style_signature.feature_ratio > 0.5 {
                FrequencyBand::Implementation.frequency(0.6) // Builders
            } else {
                FrequencyBand::Conversational.frequency(0.5) // Generalists
            };

            // Create temporal rhythm from work patterns
            for (hour, &intensity) in persona.temporal_pattern.active_hours.iter().enumerate() {
                if intensity > 0.2 {
                    let mut wave = MemoryWave::new(
                        base_freq + (hour as f32 * 10.0), // Slight frequency shift per hour
                        intensity * 0.8,
                    );
                    
                    // Emotional modulation
                    wave.valence = persona.emotional_profile.positivity - persona.emotional_profile.frustration;
                    wave.arousal = persona.emotional_profile.excitement;
                    wave.decay_tau = None; // Persistent persona pattern
                    
                    // Store in persona layer (high Z values)
                    let x = (self.simple_hash(&developer) & 0xFF) as u8;
                    let y = (hour * 10) as u8;
                    let z = 64000 + (self.simple_hash(&format!("{}-{}", developer, hour)) & 0x3FF) as u16;
                    
                    self.store_wave_at_coordinates(x, y, z, wave)?;
                }
            }

            // Create expertise signatures
            for (area, expertise) in persona.expertise_map {
                if expertise > 0.3 {
                    let mut wave = MemoryWave::new(
                        base_freq + 100.0, // Expertise frequency band
                        expertise,
                    );
                    
                    wave.valence = 0.7; // Positive association with expertise
                    wave.decay_tau = None; // Persistent
                    
                    let (x, y) = self.string_to_coordinates(&format!("{}-{}", developer, area));
                    let z = 63000;
                    
                    self.store_wave_at_coordinates(x, y, z, wave)?;
                }
            }

            // Print persona summary
            println!("  Style: {:.0}% features, {:.0}% bugfixes, {:.0}% refactoring",
                persona.style_signature.feature_ratio * 100.0,
                persona.style_signature.bugfix_ratio * 100.0,
                persona.style_signature.refactor_tendency * 100.0
            );
            println!("  Chronotype: {} ({:.1})",
                if persona.temporal_pattern.chronotype < -0.3 { "Night Owl 🦉" }
                else if persona.temporal_pattern.chronotype > 0.3 { "Early Bird 🐦" }
                else { "Flexible ⏰" },
                persona.temporal_pattern.chronotype
            );
            println!("  Emotional: {:.0}% positive, {:.0}% excited",
                persona.emotional_profile.positivity * 100.0,
                persona.emotional_profile.excitement * 100.0
            );
            println!("  Contributions: {} commits, {} files touched",
                persona.metrics.total_commits,
                persona.metrics.files_touched
            );
        }

        Ok(())
    }

    /// Query memories specific to a developer
    pub fn query_developer_memories(&self, _developer_name: &str) -> Vec<(MemoryWave, String)> {
        // Implementation would search for waves in the developer's frequency/spatial range
        // For now, return empty vec as placeholder
        Vec::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_persona_analysis() {
        // Test would run on a real git repo
        if let Ok(analyzer) = PersonaAnalyzer::new(".") {
            if let Ok(personas) = analyzer.analyze_all_developers() {
                for (dev, persona) in personas {
                    println!("Developer: {} - {} commits", dev, persona.metrics.total_commits);
                }
            }
        }
    }
}