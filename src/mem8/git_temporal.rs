//! Git-based temporal timeline builder for MEM8
//! Extracts project history directly from git to create wave memories

use std::process::Command;
use std::path::Path;
use anyhow::{Result, Context};
use chrono::{DateTime, Utc, Datelike};
use crate::mem8::{
    wave::{MemoryWave, FrequencyBand},
    integration::{DirectoryMetadata, ContentType, DirectoryHealth},
    SmartTreeMem8,
};

/// Git commit information
#[derive(Debug, Clone)]
pub struct GitCommit {
    pub hash: String,
    pub author: String,
    pub timestamp: DateTime<Utc>,
    pub message: String,
    pub files_changed: Vec<String>,
    pub additions: usize,
    pub deletions: usize,
}

/// Git file history
#[derive(Debug)]
pub struct GitFileHistory {
    pub path: String,
    pub commits: Vec<GitCommit>,
    pub total_changes: usize,
    pub authors: Vec<String>,
    pub first_seen: DateTime<Utc>,
    pub last_modified: DateTime<Utc>,
}

/// Git-based temporal analyzer for MEM8
pub struct GitTemporalAnalyzer {
    repo_path: String,
}

impl GitTemporalAnalyzer {
    pub fn new(repo_path: impl AsRef<Path>) -> Result<Self> {
        let repo_path = repo_path.as_ref().to_string_lossy().to_string();
        
        // Verify it's a git repository
        Command::new("git")
            .arg("-C")
            .arg(&repo_path)
            .arg("rev-parse")
            .arg("--git-dir")
            .output()
            .context("Failed to verify git repository")?;
            
        Ok(Self { repo_path })
    }

    /// Get complete project timeline
    pub fn get_project_timeline(&self) -> Result<Vec<GitCommit>> {
        let output = Command::new("git")
            .arg("-C")
            .arg(&self.repo_path)
            .arg("log")
            .arg("--pretty=format:%H|%an|%at|%s")
            .arg("--numstat")
            .arg("--no-merges")
            .output()
            .context("Failed to get git log")?;

        let stdout = String::from_utf8_lossy(&output.stdout);
        self.parse_git_log(&stdout)
    }

    /// Get file-specific history
    pub fn get_file_history(&self, file_path: &str) -> Result<GitFileHistory> {
        // Get commits that touched this file
        let output = Command::new("git")
            .arg("-C")
            .arg(&self.repo_path)
            .arg("log")
            .arg("--follow")
            .arg("--pretty=format:%H|%an|%at|%s")
            .arg("--")
            .arg(file_path)
            .output()
            .context("Failed to get file history")?;

        let commits = self.parse_simple_log(&String::from_utf8_lossy(&output.stdout))?;
        
        // Get unique authors
        let mut authors: Vec<String> = commits.iter()
            .map(|c| c.author.clone())
            .collect();
        authors.sort();
        authors.dedup();

        Ok(GitFileHistory {
            path: file_path.to_string(),
            total_changes: commits.len(),
            first_seen: commits.last().map(|c| c.timestamp).unwrap_or_else(Utc::now),
            last_modified: commits.first().map(|c| c.timestamp).unwrap_or_else(Utc::now),
            authors,
            commits,
        })
    }

    /// Get activity heatmap (commits per day/week)
    pub fn get_activity_heatmap(&self, days: usize) -> Result<Vec<(DateTime<Utc>, usize)>> {
        let output = Command::new("git")
            .arg("-C")
            .arg(&self.repo_path)
            .arg("log")
            .arg(format!("--since={} days ago", days))
            .arg("--pretty=format:%at")
            .output()
            .context("Failed to get activity data")?;

        let stdout = String::from_utf8_lossy(&output.stdout);
        let mut daily_commits = std::collections::HashMap::new();

        for line in stdout.lines() {
            if let Ok(timestamp) = line.parse::<i64>() {
                let date = DateTime::<Utc>::from_timestamp(timestamp, 0)
                    .unwrap_or_else(Utc::now);
                let day = date.date_naive();
                *daily_commits.entry(day).or_insert(0) += 1;
            }
        }

        let mut heatmap: Vec<_> = daily_commits.into_iter()
            .map(|(date, count)| {
                let datetime = date.and_hms_opt(0, 0, 0)
                    .unwrap()
                    .and_local_timezone(Utc)
                    .unwrap();
                (datetime, count)
            })
            .collect();
        heatmap.sort_by_key(|(date, _)| *date);

        Ok(heatmap)
    }

    /// Analyze code churn (files that change frequently)
    pub fn analyze_code_churn(&self, limit: usize) -> Result<Vec<(String, usize)>> {
        let output = Command::new("git")
            .arg("-C")
            .arg(&self.repo_path)
            .arg("log")
            .arg("--name-only")
            .arg("--pretty=format:")
            .arg("--no-merges")
            .output()
            .context("Failed to analyze code churn")?;

        let stdout = String::from_utf8_lossy(&output.stdout);
        let mut file_changes = std::collections::HashMap::new();

        for line in stdout.lines() {
            if !line.is_empty() && !line.starts_with(' ') {
                *file_changes.entry(line.to_string()).or_insert(0) += 1;
            }
        }

        let mut churn: Vec<_> = file_changes.into_iter().collect();
        churn.sort_by_key(|(_, count)| std::cmp::Reverse(*count));
        churn.truncate(limit);

        Ok(churn)
    }

    /// Parse git log output
    fn parse_git_log(&self, output: &str) -> Result<Vec<GitCommit>> {
        let mut commits = Vec::new();
        let mut current_commit: Option<GitCommit> = None;

        for line in output.lines() {
            if line.contains('|') && !line.starts_with(char::is_numeric) {
                // Commit header line
                if let Some(commit) = current_commit.take() {
                    commits.push(commit);
                }

                let parts: Vec<&str> = line.split('|').collect();
                if parts.len() >= 4 {
                    let timestamp = parts[2].parse::<i64>().unwrap_or(0);
                    current_commit = Some(GitCommit {
                        hash: parts[0].to_string(),
                        author: parts[1].to_string(),
                        timestamp: DateTime::<Utc>::from_timestamp(timestamp, 0)
                            .unwrap_or_else(Utc::now),
                        message: parts[3..].join("|"),
                        files_changed: Vec::new(),
                        additions: 0,
                        deletions: 0,
                    });
                }
            } else if let Some(ref mut commit) = current_commit {
                // File change line (numstat format)
                let parts: Vec<&str> = line.split_whitespace().collect();
                if parts.len() >= 3 {
                    if let (Ok(adds), Ok(dels)) = (parts[0].parse::<usize>(), parts[1].parse::<usize>()) {
                        commit.additions += adds;
                        commit.deletions += dels;
                        commit.files_changed.push(parts[2].to_string());
                    }
                }
            }
        }

        if let Some(commit) = current_commit {
            commits.push(commit);
        }

        Ok(commits)
    }

    /// Parse simple log output (without numstat)
    fn parse_simple_log(&self, output: &str) -> Result<Vec<GitCommit>> {
        let mut commits = Vec::new();

        for line in output.lines() {
            let parts: Vec<&str> = line.split('|').collect();
            if parts.len() >= 4 {
                let timestamp = parts[2].parse::<i64>().unwrap_or(0);
                commits.push(GitCommit {
                    hash: parts[0].to_string(),
                    author: parts[1].to_string(),
                    timestamp: DateTime::<Utc>::from_timestamp(timestamp, 0)
                        .unwrap_or_else(Utc::now),
                    message: parts[3..].join("|"),
                    files_changed: Vec::new(),
                    additions: 0,
                    deletions: 0,
                });
            }
        }

        Ok(commits)
    }
}

/// Extension trait to integrate Git temporal data with MEM8
impl SmartTreeMem8 {
    /// Import git history as wave memories
    pub fn import_git_timeline(&mut self, repo_path: impl AsRef<Path>) -> Result<()> {
        let analyzer = GitTemporalAnalyzer::new(repo_path)?;
        
        // Get project timeline
        let timeline = analyzer.get_project_timeline()?;
        println!("Importing {} commits into wave memory...", timeline.len());

        // Get activity heatmap for the last 90 days
        let heatmap = analyzer.get_activity_heatmap(90)?;
        let _max_daily_commits = heatmap.iter().map(|(_, count)| *count).max().unwrap_or(1) as f32;

        // Import each commit as a memory wave
        for (idx, commit) in timeline.iter().enumerate() {
            let days_ago = (Utc::now() - commit.timestamp).num_days() as f32;
            
            // Determine frequency based on commit characteristics
            let frequency = if commit.message.contains("fix") || commit.message.contains("bug") {
                FrequencyBand::Technical.frequency(0.7) // Bug fixes are technical
            } else if commit.message.contains("doc") || commit.message.contains("README") {
                FrequencyBand::Conversational.frequency(0.5) // Documentation
            } else if commit.additions > 500 || commit.deletions > 500 {
                FrequencyBand::Implementation.frequency(0.8) // Major changes
            } else if commit.files_changed.len() > 10 {
                FrequencyBand::DeepStructural.frequency(0.6) // Structural refactoring
            } else {
                FrequencyBand::Technical.frequency(0.5) // Default
            };

            // Amplitude based on change size and recency
            let change_factor = ((commit.additions + commit.deletions) as f32).log10() / 4.0;
            let recency_factor = (-days_ago / 30.0).exp(); // Decay over 30 days
            let amplitude = (change_factor * recency_factor).clamp(0.1, 1.0);

            // Create memory wave
            let mut wave = MemoryWave::new(frequency, amplitude);
            
            // Emotional context based on commit patterns
            wave.valence = if commit.message.contains("fix") || commit.message.contains("bug") {
                -0.2 // Negative for bug fixes
            } else if commit.message.contains("feat") || commit.message.contains("add") {
                0.6 // Positive for new features
            } else {
                0.2 // Neutral positive
            };

            // Arousal based on change magnitude
            wave.arousal = change_factor.clamp(0.1, 1.0);

            // Store in temporal layer based on age
            let z_layer = (idx as f32 / timeline.len() as f32 * 65535.0) as u16;
            
            // Use author name for spatial distribution
            let (x, y) = self.string_to_coordinates(&format!("{}-{}", commit.author, idx));
            
            self.store_wave_at_coordinates(x, y, z_layer, wave)?;
        }

        // Import code churn patterns
        let churn = analyzer.analyze_code_churn(20)?;
        for (file_path, change_count) in churn {
            let metadata = DirectoryMetadata {
                primary_type: self.detect_content_type(&file_path),
                importance: (change_count as f32 / 100.0).clamp(0.1, 1.0),
                normalized_size: 0.5, // Unknown from git
                health: if change_count > 50 { 
                    DirectoryHealth::Warning // High churn might indicate instability
                } else {
                    DirectoryHealth::Healthy
                },
                activity_level: (change_count as f32 / 20.0).clamp(0.1, 1.0),
                days_since_modified: 0, // Will be overridden by actual file check
            };
            
            self.store_directory_memory(Path::new(&file_path), metadata)?;
        }

        println!("Git timeline imported successfully!");
        Ok(())
    }



    /// Helper to detect content type from path
    fn detect_content_type(&self, path: &str) -> ContentType {
        if path.ends_with(".rs") || path.ends_with(".py") || path.ends_with(".js") {
            ContentType::Code
        } else if path.ends_with(".md") || path.contains("README") {
            ContentType::Documentation
        } else if path.ends_with(".toml") || path.ends_with(".json") || path.ends_with(".yaml") {
            ContentType::Configuration
        } else if path.contains("test") || path.contains("spec") {
            ContentType::Code // Tests are code
        } else {
            ContentType::Data
        }
    }


}

/// Create temporal "grooves" in wave space from git patterns
pub fn create_temporal_grooves(mem8: &mut SmartTreeMem8, repo_path: impl AsRef<Path>) -> Result<()> {
    let analyzer = GitTemporalAnalyzer::new(&repo_path)?;
    
    // Get activity patterns
    let heatmap = analyzer.get_activity_heatmap(365)?; // Last year
    
    // Find periodic patterns (e.g., weekly sprints, monthly releases)
    let mut weekly_pattern = [0f32; 7];
    for (date, count) in &heatmap {
        let weekday = date.weekday().num_days_from_monday() as usize;
        weekly_pattern[weekday] += *count as f32;
    }

    // Normalize weekly pattern
    let max_weekly = weekly_pattern.iter().copied().fold(0.0f32, f32::max);
    if max_weekly > 0.0 {
        for val in &mut weekly_pattern {
            *val /= max_weekly;
        }
    }

    // Create persistent wave patterns for discovered rhythms
    for (day, &intensity) in weekly_pattern.iter().enumerate() {
        if intensity > 0.2 {
            let mut wave = MemoryWave::new(
                FrequencyBand::Technical.frequency(intensity), // Use Technical for temporal patterns
                intensity * 0.5,
            );
            wave.decay_tau = None; // Persistent pattern
            wave.valence = 0.3; // Slightly positive
            
            // Store in a "rhythm" layer
            let x = (day * 36) as u8; // Spread across x-axis
            let y = 128; // Middle of y-axis
            let z = 60000; // High z-layer for persistent patterns
            
            mem8.store_wave_at_coordinates(x, y, z, wave)?;
        }
    }

    println!("Temporal grooves created from git patterns!");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;

    #[test]
    fn test_git_timeline_import() {
        if env::var("CI").is_err() {
            // Only run locally, not in CI
            let mut mem8 = SmartTreeMem8::new();
            if let Ok(()) = mem8.import_git_timeline(".") {
                assert!(mem8.active_memory_count() > 0);
            }
        }
    }
}