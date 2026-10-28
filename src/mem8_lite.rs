//! MEM8 Lite - Minimal wave-based memory for Smart Tree
//!
//! This is a self-contained subset of mem8-core from the Aye project.
//! It provides the Wave struct for hot directory tracking without
//! requiring the full MEM8 crate as a dependency.
//!
//! "Memory is wave interference patterns in cognitive space." - MEM8

use serde::{Deserialize, Serialize};

/// A wave pattern representing activity/state
///
/// For hot directory tracking, we use:
/// - frequency: Change rate (events per hour)
/// - emotional_valence: Security concern (-1.0 danger to +1.0 safe)
/// - arousal: Activity level (0.0 cold to 1.0 hot)
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Wave {
    /// Frequency in Hz (for directories: change rate)
    pub frequency: f64,
    /// Emotional valence (-1.0 to 1.0) - security concern level
    pub emotional_valence: f64,
    /// Arousal level (0.0 to 1.0) - activity/hotness
    pub arousal: f64,
}

impl Wave {
    /// Create a new wave
    pub fn new(frequency: f64, emotional_valence: f64, arousal: f64) -> Self {
        Self {
            frequency,
            emotional_valence: emotional_valence.clamp(-1.0, 1.0),
            arousal: arousal.clamp(0.0, 1.0),
        }
    }

    /// Compute resonance score with another wave
    ///
    /// Higher score = more similar frequencies and emotional states.
    /// Used to detect directories with correlated activity patterns.
    pub fn resonance_with(&self, other: &Wave) -> f64 {
        // Frequency similarity (closer = higher score)
        let freq_diff = (self.frequency - other.frequency).abs();
        let max_freq = self.frequency.max(other.frequency);
        let freq_score = if max_freq > 0.0 {
            (-freq_diff / max_freq).exp()
        } else {
            1.0
        };

        // Emotional similarity
        let valence_diff = (self.emotional_valence - other.emotional_valence).abs();
        let arousal_diff = (self.arousal - other.arousal).abs();
        let emotion_score = 1.0 - ((valence_diff + arousal_diff) / 4.0);

        // Combined resonance (weighted average)
        0.7 * freq_score + 0.3 * emotion_score
    }
}

impl Default for Wave {
    fn default() -> Self {
        Self::new(1.0, 0.0, 0.5)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_wave_creation() {
        let wave = Wave::new(10.0, 0.5, 0.8);
        assert_eq!(wave.frequency, 10.0);
        assert_eq!(wave.emotional_valence, 0.5);
        assert_eq!(wave.arousal, 0.8);
    }

    #[test]
    fn test_clamping() {
        let wave = Wave::new(1.0, 2.0, -0.5);
        assert_eq!(wave.emotional_valence, 1.0); // Clamped
        assert_eq!(wave.arousal, 0.0); // Clamped
    }

    #[test]
    fn test_resonance_identical() {
        let wave1 = Wave::new(10.0, 0.5, 0.5);
        let wave2 = Wave::new(10.0, 0.5, 0.5);
        let resonance = wave1.resonance_with(&wave2);
        assert!(resonance > 0.95); // Nearly identical
    }

    #[test]
    fn test_resonance_different() {
        let wave1 = Wave::new(10.0, 0.5, 0.5);
        let wave2 = Wave::new(100.0, -0.5, 0.1);
        let similar_resonance = wave1.resonance_with(&Wave::new(11.0, 0.5, 0.5));
        let different_resonance = wave1.resonance_with(&wave2);
        assert!(similar_resonance > different_resonance);
    }
}
