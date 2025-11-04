// Semantic Novelty Tracking - "The New Car Smell Algorithm"
// The most valuable thought is the one never thought before

use super::semantic::SemanticUnit;
use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};

/// Tracks the novelty and decay of semantic patterns
pub struct NoveltyTracker {
    /// Map of semantic fingerprints to their history
    seen_patterns: HashMap<String, PatternHistory>,
    /// Global novelty threshold
    novelty_threshold: f32,
}

#[derive(Debug, Clone)]
struct PatternHistory {
    first_seen: u64,       // Timestamp of first occurrence
    last_seen: u64,        // Timestamp of last occurrence
    occurrence_count: u32, // How many times we've seen this
    peak_novelty: f32,     // The highest novelty it achieved
    current_value: f32,    // Current semantic value
}

impl Default for NoveltyTracker {
    fn default() -> Self {
        Self::new()
    }
}

impl NoveltyTracker {
    pub fn new() -> Self {
        Self {
            seen_patterns: HashMap::new(),
            novelty_threshold: 0.1, // Below this, it's "background noise"
        }
    }

    /// Calculate the novelty score of semantic units
    pub fn calculate_novelty(&mut self, units: &[SemanticUnit]) -> NoveltyScore {
        let fingerprint = self.generate_fingerprint(units);
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        match self.seen_patterns.get_mut(&fingerprint) {
            None => {
                // First time seeing this pattern - MAXIMUM NOVELTY!
                let history = PatternHistory {
                    first_seen: now,
                    last_seen: now,
                    occurrence_count: 1,
                    peak_novelty: 1.0,
                    current_value: 1.0,
                };

                self.seen_patterns.insert(fingerprint.clone(), history);

                NoveltyScore {
                    value: 1.0,
                    is_novel: true,
                    occurrence: 1,
                    decay_factor: 0.0,
                    classification: NoveltyClass::Revolutionary,
                }
            }
            Some(history) => {
                // We've seen this before - apply decay
                history.occurrence_count += 1;
                history.last_seen = now;

                // Time decay factor (ideas get stale)
                let time_since_first = (now - history.first_seen) as f32;
                let time_decay = 1.0 / (1.0 + time_since_first / 86400.0); // Daily decay

                // Repetition decay (the more we see it, the less novel)
                let repetition_decay = 1.0 / (history.occurrence_count as f32).sqrt();

                // Combined decay
                let decay_factor = time_decay * repetition_decay;
                history.current_value = history.peak_novelty * decay_factor;

                // Classify the novelty level
                let classification = match history.current_value {
                    v if v > 0.8 => NoveltyClass::Fresh,
                    v if v > 0.5 => NoveltyClass::Interesting,
                    v if v > 0.2 => NoveltyClass::Familiar,
                    v if v > self.novelty_threshold => NoveltyClass::Stale,
                    _ => NoveltyClass::BackgroundNoise,
                };

                NoveltyScore {
                    value: history.current_value,
                    is_novel: false,
                    occurrence: history.occurrence_count,
                    decay_factor,
                    classification,
                }
            }
        }
    }

    /// Generate a unique fingerprint for semantic units
    fn generate_fingerprint(&self, units: &[SemanticUnit]) -> String {
        let mut tokens = Vec::new();

        for unit in units {
            for token in &unit.tokens {
                tokens.push(*token as u8);
            }
        }

        // Simple hash - in production would use proper hashing
        tokens.sort();
        tokens.dedup();

        tokens
            .iter()
            .map(|t| format!("{:02x}", t))
            .collect::<Vec<_>>()
            .join("-")
    }

    /// Get the most novel patterns in the system
    pub fn get_top_novel(&self, limit: usize) -> Vec<(String, f32)> {
        let mut patterns: Vec<_> = self
            .seen_patterns
            .iter()
            .map(|(fp, hist)| (fp.clone(), hist.current_value))
            .collect();

        patterns.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
        patterns.truncate(limit);
        patterns
    }

    /// Decay all patterns over time (call periodically)
    pub fn apply_temporal_decay(&mut self, decay_rate: f32) {
        for history in self.seen_patterns.values_mut() {
            history.current_value *= 1.0 - decay_rate;
        }

        // Remove patterns that have decayed to background noise
        self.seen_patterns
            .retain(|_, hist| hist.current_value > self.novelty_threshold);
    }
}

#[derive(Debug, Clone)]
pub struct NoveltyScore {
    pub value: f32,        // 0.0 to 1.0
    pub is_novel: bool,    // First time seen?
    pub occurrence: u32,   // How many times seen
    pub decay_factor: f32, // Current decay level
    pub classification: NoveltyClass,
}

#[derive(Debug, Clone, PartialEq)]
pub enum NoveltyClass {
    Revolutionary,   // Never seen before (1.0)
    Fresh,           // Still new and exciting (0.8-1.0)
    Interesting,     // Seen a few times (0.5-0.8)
    Familiar,        // Common but relevant (0.2-0.5)
    Stale,           // Overused (0.1-0.2)
    BackgroundNoise, // Too common to matter (<0.1)
}

impl NoveltyClass {
    pub fn emoji(&self) -> &str {
        match self {
            Self::Revolutionary => "💎",   // Diamond - precious and rare
            Self::Fresh => "🌟",           // Sparkles - still shining
            Self::Interesting => "💡",     // Light bulb - worth noting
            Self::Familiar => "📝",        // Note - documented
            Self::Stale => "📰",           // Old news
            Self::BackgroundNoise => "💤", // Sleeping - ignorable
        }
    }
}

/// DNS integration - rank results by novelty
pub fn generate_novelty_dns(score: &NoveltyScore, base_domain: &str) -> String {
    let novelty_prefix = match score.classification {
        NoveltyClass::Revolutionary => "revolutionary",
        NoveltyClass::Fresh => "fresh",
        NoveltyClass::Interesting => "interesting",
        NoveltyClass::Familiar => "familiar",
        NoveltyClass::Stale => "stale",
        NoveltyClass::BackgroundNoise => "noise",
    };

    format!("{}.{}.q7.is", novelty_prefix, base_domain)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::marqant::semantic::{SemanticToken, SemanticUnit};

    #[test]
    fn test_novelty_decay() {
        let mut tracker = NoveltyTracker::new();

        // Create a semantic pattern
        let unit = SemanticUnit {
            tokens: vec![SemanticToken::ActionLearning, SemanticToken::ContextRust],
            metadata: HashMap::new(),
            intensity: 0.8,
        };

        // First time - should be revolutionary
        let score1 = tracker.calculate_novelty(&[unit.clone()]);
        assert_eq!(score1.classification, NoveltyClass::Revolutionary);
        assert_eq!(score1.value, 1.0);
        assert!(score1.is_novel);

        // Second time - should decay
        let score2 = tracker.calculate_novelty(&[unit.clone()]);
        assert!(!score2.is_novel);
        assert!(score2.value < 1.0);
        assert_eq!(score2.occurrence, 2);

        // Many times - should become stale
        for _ in 0..100 {
            tracker.calculate_novelty(&[unit.clone()]);
        }

        let score_final = tracker.calculate_novelty(&[unit.clone()]);
        assert!(score_final.value < 0.2);
        assert!(score_final.occurrence > 100);
    }

    #[test]
    fn test_novelty_classification() {
        let classifications = [
            (1.0, NoveltyClass::Revolutionary, "💎"),
            (0.9, NoveltyClass::Fresh, "🌟"),
            (0.6, NoveltyClass::Interesting, "💡"),
            (0.3, NoveltyClass::Familiar, "📝"),
            (0.15, NoveltyClass::Stale, "📰"),
            (0.05, NoveltyClass::BackgroundNoise, "💤"),
        ];

        for (_, class, emoji) in classifications {
            assert_eq!(class.emoji(), emoji);
        }
    }
}
