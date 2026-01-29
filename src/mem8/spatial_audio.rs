//! Spatial Audio Processing using MEM8 Wave Grid
//!
//! The 256×256 grid becomes a spatial "room" where:
//! - Sound sources are placed at (x, y) positions
//! - Two "ears" sample interference patterns at fixed positions
//! - Wave propagation creates natural stereo separation
//!
//! Grid interpretation:
//! - X,Y: Spatial position (u8 × u8 = 256×256 room)
//! - Z (u16): Intensity/amplitude at that position
//! - Time: Observation rate (sampling frequency)

use super::wave::{MemoryWave, WaveGrid};
use std::sync::{Arc, RwLock};

/// Speed of sound in grid units per second
/// (tuned for the 256×256 space - about 34 units = 1 "meter")
const SPEED_OF_SOUND: f32 = 343.0 / 10.0; // ~34 grid units/sec

/// Default ear separation (~17cm = ~6 grid units at our scale)
const DEFAULT_EAR_SEPARATION: u8 = 6;

/// Position in the spatial grid
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Position {
    pub x: u8,
    pub y: u8,
}

impl Position {
    pub fn new(x: u8, y: u8) -> Self {
        Self { x, y }
    }

    /// Distance to another position
    pub fn distance_to(&self, other: &Position) -> f32 {
        let dx = self.x as f32 - other.x as f32;
        let dy = self.y as f32 - other.y as f32;
        (dx * dx + dy * dy).sqrt()
    }

    /// Angle to another position (radians, 0 = right, PI/2 = up)
    pub fn angle_to(&self, other: &Position) -> f32 {
        let dx = other.x as f32 - self.x as f32;
        let dy = other.y as f32 - self.y as f32;
        dy.atan2(dx)
    }
}

/// A sound source in the spatial field
#[derive(Debug, Clone)]
pub struct SoundSource {
    /// Position in the grid
    pub position: Position,
    /// The wave definition (frequency, amplitude, phase)
    pub wave: MemoryWave,
    /// Decay factor (how much amplitude decreases with distance)
    pub decay: f32,
    /// Whether this source is currently active
    pub active: bool,
}

impl SoundSource {
    pub fn new(x: u8, y: u8, frequency: f32, amplitude: f32) -> Self {
        Self {
            position: Position::new(x, y),
            wave: MemoryWave::new(frequency, amplitude),
            decay: 1.0, // Linear decay by default
            active: true,
        }
    }

    /// Calculate the wave value at a listener position and time
    pub fn sample_at(&self, listener: &Position, t: f32) -> f32 {
        if !self.active {
            return 0.0;
        }

        let distance = self.position.distance_to(listener);

        // Time delay based on distance and speed of sound
        let delay = distance / SPEED_OF_SOUND;

        // Amplitude decay with distance (inverse square law approximation)
        let amplitude_factor = 1.0 / (1.0 + self.decay * distance * 0.1);

        // Calculate wave value at delayed time
        self.wave.calculate(t - delay) * amplitude_factor
    }
}

/// Stereo sample output
#[derive(Debug, Clone, Copy, Default)]
pub struct StereoSample {
    pub left: f32,
    pub right: f32,
}

impl StereoSample {
    pub fn new(left: f32, right: f32) -> Self {
        Self { left, right }
    }

    /// Mix with another sample
    pub fn mix(&mut self, other: StereoSample) {
        self.left += other.left;
        self.right += other.right;
    }

    /// Apply gain
    pub fn apply_gain(&mut self, gain: f32) {
        self.left *= gain;
        self.right *= gain;
    }

    /// Clamp to valid range
    pub fn clamp(&mut self) {
        self.left = self.left.clamp(-1.0, 1.0);
        self.right = self.right.clamp(-1.0, 1.0);
    }
}

/// Spatial audio processor using the MEM8 wave grid
pub struct SpatialAudioField {
    /// The underlying wave grid (for storing/retrieving wave definitions)
    grid: Arc<RwLock<WaveGrid>>,

    /// Active sound sources
    sources: Vec<SoundSource>,

    /// Left ear position
    left_ear: Position,

    /// Right ear position
    right_ear: Position,

    /// Head center (for calculating angles)
    head_center: Position,

    /// Current time (advances with each sample)
    current_time: f32,

    /// Sample rate (samples per second)
    sample_rate: f32,
}

impl SpatialAudioField {
    /// Create a new spatial audio field with default ear positions
    /// Ears are placed at the center of the grid, separated horizontally
    pub fn new() -> Self {
        let center_y = 128u8;
        let center_x = 128u8;
        let half_sep = DEFAULT_EAR_SEPARATION / 2;

        Self {
            grid: Arc::new(RwLock::new(WaveGrid::new())),
            sources: Vec::new(),
            left_ear: Position::new(center_x - half_sep, center_y),
            right_ear: Position::new(center_x + half_sep, center_y),
            head_center: Position::new(center_x, center_y),
            current_time: 0.0,
            sample_rate: 44100.0, // CD quality default
        }
    }

    /// Create with custom ear positions
    pub fn with_ears(left: Position, right: Position) -> Self {
        let center_x = (left.x as u16 + right.x as u16) / 2;
        let center_y = (left.y as u16 + right.y as u16) / 2;

        Self {
            grid: Arc::new(RwLock::new(WaveGrid::new())),
            sources: Vec::new(),
            left_ear: left,
            right_ear: right,
            head_center: Position::new(center_x as u8, center_y as u8),
            current_time: 0.0,
            sample_rate: 44100.0,
        }
    }

    /// Set sample rate
    pub fn set_sample_rate(&mut self, rate: f32) {
        self.sample_rate = rate;
    }

    /// Add a sound source to the field
    pub fn add_source(&mut self, source: SoundSource) -> usize {
        let idx = self.sources.len();

        // Also store in grid at the source position
        if let Ok(mut grid) = self.grid.write() {
            grid.store(
                source.position.x,
                source.position.y,
                (source.wave.amplitude * 65535.0) as u16,
                source.wave.clone(),
            );
        }

        self.sources.push(source);
        idx
    }

    /// Add a simple tone at a position
    pub fn add_tone(&mut self, x: u8, y: u8, frequency: f32, amplitude: f32) -> usize {
        self.add_source(SoundSource::new(x, y, frequency, amplitude))
    }

    /// Remove a sound source
    pub fn remove_source(&mut self, idx: usize) -> Option<SoundSource> {
        if idx < self.sources.len() {
            Some(self.sources.remove(idx))
        } else {
            None
        }
    }

    /// Activate/deactivate a source
    pub fn set_source_active(&mut self, idx: usize, active: bool) {
        if let Some(source) = self.sources.get_mut(idx) {
            source.active = active;
        }
    }

    /// Move a source to a new position
    pub fn move_source(&mut self, idx: usize, new_pos: Position) {
        if let Some(source) = self.sources.get_mut(idx) {
            source.position = new_pos;
        }
    }

    /// Sample all sources at the current time, returning stereo output
    pub fn sample(&mut self) -> StereoSample {
        let mut output = StereoSample::default();

        for source in &self.sources {
            let left_sample = source.sample_at(&self.left_ear, self.current_time);
            let right_sample = source.sample_at(&self.right_ear, self.current_time);

            output.left += left_sample;
            output.right += right_sample;
        }

        // Advance time
        self.current_time += 1.0 / self.sample_rate;

        output.clamp();
        output
    }

    /// Sample N frames and return as interleaved stereo buffer
    pub fn sample_frames(&mut self, num_frames: usize) -> Vec<f32> {
        let mut buffer = Vec::with_capacity(num_frames * 2);

        for _ in 0..num_frames {
            let sample = self.sample();
            buffer.push(sample.left);
            buffer.push(sample.right);
        }

        buffer
    }

    /// Calculate the perceived direction of a position from the listener
    /// Returns angle in degrees (-90 = full left, 0 = center, 90 = full right)
    pub fn direction_of(&self, pos: &Position) -> f32 {
        let angle = self.head_center.angle_to(pos);
        // Convert to degrees and adjust so 0 = forward
        let degrees = angle.to_degrees();
        // Assuming "forward" is +Y direction
        degrees - 90.0
    }

    /// Get Interaural Time Difference for a position (in seconds)
    pub fn itd_for(&self, pos: &Position) -> f32 {
        let dist_left = pos.distance_to(&self.left_ear);
        let dist_right = pos.distance_to(&self.right_ear);
        (dist_left - dist_right) / SPEED_OF_SOUND
    }

    /// Get Interaural Level Difference for a position (as ratio)
    pub fn ild_for(&self, pos: &Position) -> f32 {
        let dist_left = pos.distance_to(&self.left_ear);
        let dist_right = pos.distance_to(&self.right_ear);

        // Ratio of amplitudes (inverse of distance ratio, simplified)
        if dist_left > 0.1 && dist_right > 0.1 {
            dist_right / dist_left
        } else {
            1.0
        }
    }

    /// Current time in seconds
    pub fn time(&self) -> f32 {
        self.current_time
    }

    /// Reset time to zero
    pub fn reset_time(&mut self) {
        self.current_time = 0.0;
    }

    /// Number of active sources
    pub fn source_count(&self) -> usize {
        self.sources.iter().filter(|s| s.active).count()
    }
}

impl Default for SpatialAudioField {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_position_distance() {
        let p1 = Position::new(0, 0);
        let p2 = Position::new(3, 4);
        assert!((p1.distance_to(&p2) - 5.0).abs() < 0.001);
    }

    #[test]
    fn test_stereo_separation() {
        let mut field = SpatialAudioField::new();

        // Add a source to the left of center
        field.add_tone(64, 128, 440.0, 0.5); // Left side

        // Sample multiple times to get past the initial zero-crossing
        // and accumulate total power in each channel
        let mut left_power = 0.0f32;
        let mut right_power = 0.0f32;

        for _ in 0..1000 {
            let sample = field.sample();
            left_power += sample.left * sample.left;
            right_power += sample.right * sample.right;
        }

        // RMS power should be higher in left channel for left-positioned source
        assert!(left_power > right_power,
                "Left should be louder for left-positioned source (L:{:.4} R:{:.4})",
                left_power.sqrt(), right_power.sqrt());
    }

    #[test]
    fn test_itd_calculation() {
        let field = SpatialAudioField::new();

        // Source directly to the left
        let left_source = Position::new(64, 128);
        let itd = field.itd_for(&left_source);

        // ITD should be negative (arrives at left ear first)
        assert!(itd < 0.0, "ITD should be negative for left source");
    }

    #[test]
    fn test_center_source_equal() {
        let mut field = SpatialAudioField::new();

        // Add a source at center (directly in front)
        field.add_tone(128, 200, 440.0, 0.5); // Centered, in front

        // Sample multiple times and check L/R are similar
        for _ in 0..100 {
            let sample = field.sample();
            let diff = (sample.left - sample.right).abs();
            assert!(diff < 0.1, "Center source should have similar L/R");
        }
    }
}
