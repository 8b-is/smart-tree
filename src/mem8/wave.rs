//! Wave-based memory model for MEM8 cognitive architecture
//! Based on the MEM8 paper - 256×256×65536 wave grid with interference patterns

use std::f32::consts::PI;
use std::sync::Arc;
use std::time::{Duration, Instant};

/// Memory wave at a specific grid position
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct MemoryWave {
    /// Amplitude modulated by emotion and time
    pub amplitude: f32,
    /// Wave frequency encoding semantic content (0-1000Hz)
    pub frequency: f32,
    /// Phase encoding temporal relationships
    pub phase: f32,
    /// Emotional valence (-1.0 to 1.0)
    pub valence: f32,
    /// Emotional arousal (0.0 to 1.0)
    pub arousal: f32,
    /// Creation timestamp
    #[serde(skip, default = "Instant::now")]
    pub created_at: Instant,
    /// Decay time constant (None = infinite)
    #[serde(skip, default)]
    pub decay_tau: Option<Duration>,
}

impl MemoryWave {
    /// Create a new memory wave
    pub fn new(frequency: f32, amplitude: f32) -> Self {
        Self {
            amplitude,
            frequency: frequency.clamp(0.0, 1000.0),
            phase: 0.0,
            valence: 0.0,
            arousal: 0.0,
            created_at: Instant::now(),
            decay_tau: Some(Duration::from_secs(5)), // Default 5s decay
        }
    }

    /// Create a new memory wave with FrequencyBand
    pub fn new_with_band(band: FrequencyBand, amplitude: f32, phase: f32, decay_rate: f32) -> Self {
        let frequency = band.center_frequency();
        let decay_tau = if decay_rate > 0.0 {
            Some(Duration::from_secs_f32(1.0 / decay_rate))
        } else {
            None
        };

        Self {
            amplitude,
            frequency: frequency.clamp(0.0, 1000.0),
            phase,
            valence: 0.0,
            arousal: 0.0,
            created_at: Instant::now(),
            decay_tau,
        }
    }

    /// Calculate wave value at time t with decay and emotional modulation
    pub fn calculate(&self, t: f32) -> f32 {
        let decay = self.calculate_decay();
        let emotional_mod = self.calculate_emotional_modulation();

        self.amplitude * decay * emotional_mod * (2.0 * PI * self.frequency * t + self.phase).sin()
    }

    /// Calculate temporal decay
    pub fn calculate_decay(&self) -> f32 {
        match self.decay_tau {
            Some(tau) => {
                let elapsed = self.created_at.elapsed().as_secs_f32();
                (-elapsed / tau.as_secs_f32()).exp()
            }
            None => 1.0, // No decay
        }
    }

    /// Calculate emotional modulation based on valence and arousal
    pub fn calculate_emotional_modulation(&self) -> f32 {
        const ALPHA: f32 = 0.3; // Valence influence
        const BETA: f32 = 0.5; // Arousal influence

        (1.0 + ALPHA * self.valence) * (1.0 + BETA * self.arousal)
    }

    /// Apply context-aware decay based on relevance, familiarity, and threat
    pub fn apply_context_decay(&mut self, relevance: f32, familiarity: f32, threat: f32) {
        if let Some(base_tau) = self.decay_tau {
            let r_factor = relevance.clamp(0.5, 2.0);
            let f_factor = familiarity.clamp(0.8, 1.5);
            let t_factor = threat.clamp(0.3, 1.0);

            let adjusted_tau = base_tau.as_secs_f32() * r_factor * f_factor * t_factor;
            self.decay_tau = Some(Duration::from_secs_f32(adjusted_tau));
        }
    }
}

/// 3D Wave Grid: 256×256×65536 (8-bit × 8-bit × 16-bit)
pub struct WaveGrid {
    /// Grid dimensions
    pub width: usize, // 256
    pub height: usize, // 256
    pub depth: usize,  // 65536

    /// The actual grid storage (flattened for performance)
    grid: Vec<Option<Arc<MemoryWave>>>,

    /// Noise floor threshold for adaptive filtering
    pub noise_floor: f32,
}

impl Default for WaveGrid {
    fn default() -> Self {
        Self::new()
    }
}

impl WaveGrid {
    /// Create a new wave grid with standard MEM8 dimensions
    pub fn new() -> Self {
        const WIDTH: usize = 256;
        const HEIGHT: usize = 256;
        const DEPTH: usize = 65536;

        Self {
            width: WIDTH,
            height: HEIGHT,
            depth: DEPTH,
            grid: vec![None; WIDTH * HEIGHT * DEPTH],
            noise_floor: 0.1,
        }
    }

    /// Get linear index from 3D coordinates
    fn get_index(&self, x: u8, y: u8, z: u16) -> usize {
        let x = x as usize;
        let y = y as usize;
        let z = z as usize;

        z * self.width * self.height + y * self.width + x
    }

    /// Store a memory wave at specific coordinates
    pub fn store(&mut self, x: u8, y: u8, z: u16, wave: MemoryWave) {
        let idx = self.get_index(x, y, z);

        // Apply noise floor filtering
        if wave.amplitude > self.noise_floor {
            self.grid[idx] = Some(Arc::new(wave));
        }
    }

    /// Retrieve a memory wave at specific coordinates
    pub fn get(&self, x: u8, y: u8, z: u16) -> Option<&Arc<MemoryWave>> {
        let idx = self.get_index(x, y, z);
        self.grid[idx].as_ref()
    }

    /// Calculate interference pattern at a specific point
    pub fn calculate_interference(&self, x: u8, y: u8, z: u16, t: f32) -> f32 {
        let mut total = 0.0;

        // Check 3x3x3 neighborhood for interference
        for dx in -1i8..=1 {
            for dy in -1i8..=1 {
                for dz in -1i16..=1 {
                    let nx = (x as i16 + dx as i16).clamp(0, 255) as u8;
                    let ny = (y as i16 + dy as i16).clamp(0, 255) as u8;
                    let nz = (z as i32 + dz as i32).clamp(0, 65535) as u16;

                    if let Some(wave) = self.get(nx, ny, nz) {
                        // Weight by distance (closer neighbors have more influence)
                        let distance = ((dx * dx + dy * dy) as f32 + (dz * dz) as f32).sqrt();
                        let weight = 1.0 / (1.0 + distance);
                        total += wave.calculate(t) * weight;
                    }
                }
            }
        }

        total
    }

    /// Adaptive noise floor adjustment based on environmental conditions
    pub fn adjust_noise_floor(&mut self, environmental_noise: f32) {
        // Adapt noise floor to environmental conditions
        self.noise_floor = (self.noise_floor * 0.9 + environmental_noise * 0.1).clamp(0.01, 0.5);
    }

    /// Count active (non-decayed) memories
    pub fn active_memory_count(&self) -> usize {
        self.grid
            .iter()
            .filter_map(|slot| slot.as_ref())
            .filter(|wave| wave.calculate_decay() > 0.01)
            .count()
    }
}

/// Frequency bands for different content types
#[derive(Debug, Clone, Copy, serde::Serialize, serde::Deserialize)]
pub enum FrequencyBand {
    DeepStructural, // 0-200Hz
    Conversational, // 200-400Hz
    Technical,      // 400-600Hz
    Implementation, // 600-800Hz
    Abstract,       // 800-1000Hz
    // Brain wave bands (for cognitive states)
    Beta,  // 13-30Hz (active, alert)
    Gamma, // 30-100Hz (conscious awareness)
}

impl FrequencyBand {
    /// Get the frequency range for this band
    pub fn range(&self) -> (f32, f32) {
        match self {
            Self::DeepStructural => (0.0, 200.0),
            Self::Conversational => (200.0, 400.0),
            Self::Technical => (400.0, 600.0),
            Self::Implementation => (600.0, 800.0),
            Self::Abstract => (800.0, 1000.0),
            // Brain wave bands (lower frequencies)
            Self::Beta => (13.0, 30.0),
            Self::Gamma => (30.0, 100.0),
        }
    }

    /// Get the center frequency for this band
    pub fn center_frequency(&self) -> f32 {
        let (min, max) = self.range();
        (min + max) / 2.0
    }

    /// Get a frequency within this band
    pub fn frequency(&self, position: f32) -> f32 {
        let (min, max) = self.range();
        min + (max - min) * position.clamp(0.0, 1.0)
    }

    /// Determine band from frequency
    pub fn from_frequency(freq: f32) -> Self {
        match freq {
            f if f < 200.0 => Self::DeepStructural,
            f if f < 400.0 => Self::Conversational,
            f if f < 600.0 => Self::Technical,
            f if f < 800.0 => Self::Implementation,
            _ => Self::Abstract,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_memory_wave_creation() {
        let wave = MemoryWave::new(440.0, 0.8);
        assert_eq!(wave.frequency, 440.0);
        assert_eq!(wave.amplitude, 0.8);
    }

    #[test]
    fn test_wave_grid_storage() {
        let mut grid = WaveGrid::new();
        let wave = MemoryWave::new(440.0, 0.5);

        grid.store(128, 128, 32768, wave);

        assert!(grid.get(128, 128, 32768).is_some());
        assert!(grid.get(0, 0, 0).is_none());
    }

    #[test]
    fn test_emotional_modulation() {
        let mut wave = MemoryWave::new(440.0, 1.0);
        wave.valence = 0.5; // Positive
        wave.arousal = 0.8; // High arousal

        let modulation = wave.calculate_emotional_modulation();
        assert!(modulation > 1.0); // Should amplify
    }
}
