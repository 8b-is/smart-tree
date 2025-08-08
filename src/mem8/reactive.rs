//! Reactive memory layers for MEM8 - hierarchical processing from reflexes to consciousness
//! Implements 4 layers: 0-10ms hardware reflexes to >200ms conscious deliberation

use crate::mem8::wave::{MemoryWave, WaveGrid};
use std::sync::{Arc, RwLock};
use std::time::{Duration, Instant};

/// Reactive layer types with their response time windows
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ReactiveLayer {
    /// Layer 0: Hardware reflexes (0-10ms) - direct sensor-to-response
    HardwareReflex,
    /// Layer 1: Subcortical reactions (10-50ms) - pattern-matched responses
    SubcorticalReaction,
    /// Layer 2: Emotional responses (50-200ms) - emotionally-modulated decisions
    EmotionalResponse,
    /// Layer 3: Conscious deliberation (>200ms) - full cognitive processing
    ConsciousDeliberation,
}

impl ReactiveLayer {
    /// Get the response time window for this layer
    pub fn response_window(&self) -> (Duration, Option<Duration>) {
        match self {
            Self::HardwareReflex => (Duration::ZERO, Some(Duration::from_millis(10))),
            Self::SubcorticalReaction => {
                (Duration::from_millis(10), Some(Duration::from_millis(50)))
            }
            Self::EmotionalResponse => {
                (Duration::from_millis(50), Some(Duration::from_millis(200)))
            }
            Self::ConsciousDeliberation => (Duration::from_millis(200), None),
        }
    }

    /// Determine which layer should handle based on response time
    pub fn from_response_time(elapsed: Duration) -> Self {
        match elapsed.as_millis() {
            0..=10 => Self::HardwareReflex,
            11..=50 => Self::SubcorticalReaction,
            51..=200 => Self::EmotionalResponse,
            _ => Self::ConsciousDeliberation,
        }
    }
}

/// A reactive response pattern
#[derive(Clone)]
pub struct ReactivePattern {
    /// Pattern identifier
    pub id: String,
    /// Minimum activation threshold
    pub threshold: f32,
    /// Weight for this pattern
    pub weight: f32,
    /// Pattern-specific response
    pub response: Arc<dyn Fn() -> ReactiveResponse + Send + Sync>,
}

/// Response from a reactive layer
#[derive(Debug, Clone)]
pub struct ReactiveResponse {
    /// Layer that generated this response
    pub layer: ReactiveLayer,
    /// Response strength (0.0 to 1.0)
    pub strength: f32,
    /// Action to take
    pub action: String,
    /// Response latency
    pub latency: Duration,
}

/// Hierarchical reactive memory system
pub struct ReactiveMemory {
    /// Wave grid for memory storage
    wave_grid: Arc<RwLock<WaveGrid>>,
    /// Registered patterns for each layer
    patterns: Vec<Vec<ReactivePattern>>,
    /// Start time for latency tracking
    start_time: Instant,
}

impl ReactiveMemory {
    /// Create a new reactive memory system
    pub fn new(wave_grid: Arc<RwLock<WaveGrid>>) -> Self {
        Self {
            wave_grid,
            patterns: vec![Vec::new(); 4], // 4 layers
            start_time: Instant::now(),
        }
    }

    /// Register a reactive pattern for a specific layer
    pub fn register_pattern(&mut self, layer: ReactiveLayer, pattern: ReactivePattern) {
        let layer_idx = layer as usize;
        self.patterns[layer_idx].push(pattern);
    }

    /// Process input through all reactive layers
    pub fn process(&self, input: &SensorInput) -> Option<ReactiveResponse> {
        let start = Instant::now();

        // Check each layer in order (fastest to slowest)
        for layer in [
            ReactiveLayer::HardwareReflex,
            ReactiveLayer::SubcorticalReaction,
            ReactiveLayer::EmotionalResponse,
            ReactiveLayer::ConsciousDeliberation,
        ] {
            if let Some(response) = self.process_layer(layer, input, start) {
                // Check if we should bypass higher layers
                if self.should_bypass(layer, &response) {
                    return Some(response);
                }
            }
        }

        None
    }

    /// Process input for a specific layer
    fn process_layer(
        &self,
        layer: ReactiveLayer,
        input: &SensorInput,
        start: Instant,
    ) -> Option<ReactiveResponse> {
        let layer_idx = layer as usize;
        let elapsed = start.elapsed();

        // Check if we're within this layer's time window
        let (min_time, max_time) = layer.response_window();
        if elapsed < min_time || (max_time.is_some() && elapsed > max_time.unwrap()) {
            return None;
        }

        // Evaluate patterns for this layer
        let mut best_response: Option<ReactiveResponse> = None;
        let mut best_strength = 0.0;

        for pattern in &self.patterns[layer_idx] {
            let activation = self.calculate_activation(pattern, input);

            if activation > pattern.threshold && activation > best_strength {
                best_strength = activation;
                best_response = Some((pattern.response)());
            }
        }

        best_response
    }

    /// Calculate pattern activation strength
    fn calculate_activation(&self, pattern: &ReactivePattern, input: &SensorInput) -> f32 {
        // Layer-specific activation calculation
        match input {
            SensorInput::Visual { intensity, .. } => pattern.weight * intensity,
            SensorInput::Audio { amplitude, .. } => pattern.weight * amplitude,
            SensorInput::Threat { severity, .. } => pattern.weight * severity,
            _ => 0.0,
        }
    }

    /// Determine if we should bypass higher layers
    fn should_bypass(&self, layer: ReactiveLayer, response: &ReactiveResponse) -> bool {
        // Critical responses bypass higher processing
        match layer {
            ReactiveLayer::HardwareReflex => response.strength > 0.9,
            ReactiveLayer::SubcorticalReaction => response.strength > 0.8,
            ReactiveLayer::EmotionalResponse => response.strength > 0.7,
            ReactiveLayer::ConsciousDeliberation => true, // Always final
        }
    }

    /// Calculate bypass probability based on threat level
    pub fn bypass_probability(layer: ReactiveLayer, threat_level: f32) -> f32 {
        const K: f32 = 2.0; // Sensitivity constant
        let layer_idx = layer as usize;

        1.0 - (-K * (3.0 - layer_idx as f32) * threat_level).exp()
    }
}

/// Sensor input types
#[derive(Debug, Clone)]
pub enum SensorInput {
    Visual {
        intensity: f32,
        motion: f32,
        looming: bool,
    },
    Audio {
        amplitude: f32,
        frequency: f32,
        sudden: bool,
    },
    Threat {
        severity: f32,
        proximity: f32,
        pattern: String,
    },
    Network {
        packet_malformed: bool,
        attack_signature: Option<String>,
    },
}

/// Looming detection for reflexive responses
pub struct LoomingDetector {
    /// Previous angular size measurements
    history: Vec<(Instant, f32)>,
    /// Response threshold (typically 0.5-1.0 seconds)
    threshold: f32,
}

impl LoomingDetector {
    pub fn new(threshold: f32) -> Self {
        Self {
            history: Vec::new(),
            threshold,
        }
    }

    /// Update with new angular size measurement
    pub fn update(&mut self, angular_size: f32) -> Option<f32> {
        let now = Instant::now();
        self.history.push((now, angular_size));

        // Keep only recent history (last 500ms)
        self.history
            .retain(|(t, _)| now.duration_since(*t) < Duration::from_millis(500));

        // Need at least 2 points to calculate expansion
        if self.history.len() < 2 {
            return None;
        }

        // Calculate angular expansion rate (tau^-1)
        let (t1, theta1) = self.history[self.history.len() - 2];
        let (t2, theta2) = self.history[self.history.len() - 1];

        let dt = t2.duration_since(t1).as_secs_f32();
        let d_theta = theta2 - theta1;

        if dt > 0.0 && theta2 > 0.0 {
            let tau_inv = d_theta / (theta2 * dt);

            // Calculate urgency
            let urgency = 1.0 - (-self.threshold / tau_inv.max(0.001)).exp();
            Some(urgency)
        } else {
            None
        }
    }
}

/// Multi-modal sensor coherence calculator
pub struct SensorCoherence {
    /// Sensor readings with phase information
    sensors: Vec<(f32, f32)>, // (amplitude, phase)
}

impl SensorCoherence {
    pub fn new() -> Self {
        Self {
            sensors: Vec::new(),
        }
    }

    /// Add a sensor reading
    pub fn add_sensor(&mut self, amplitude: f32, phase: f32) {
        self.sensors.push((amplitude, phase));
    }

    /// Calculate coherence (0.0 = disagreement, 1.0 = agreement)
    pub fn calculate(&self) -> f32 {
        if self.sensors.is_empty() {
            return 0.0;
        }

        // Calculate coherence using phase relationships
        let mut sum_real = 0.0;
        let mut sum_imag = 0.0;
        let mut sum_amplitude_sq = 0.0;

        for &(amplitude, phase) in &self.sensors {
            sum_real += amplitude * phase.cos();
            sum_imag += amplitude * phase.sin();
            sum_amplitude_sq += amplitude * amplitude;
        }

        if sum_amplitude_sq > 0.0 {
            let magnitude_sq = sum_real * sum_real + sum_imag * sum_imag;
            magnitude_sq / sum_amplitude_sq
        } else {
            0.0
        }
    }
}

/// Subliminal processing for below-threshold stimuli
pub struct SubliminalProcessor {
    /// Threshold for conscious awareness
    awareness_threshold: f32,
    /// Subliminal amplitude range
    subliminal_range: (f32, f32),
}

impl SubliminalProcessor {
    pub fn new() -> Self {
        Self {
            awareness_threshold: 0.15,
            subliminal_range: (0.01, 0.15),
        }
    }

    /// Check if stimulus is subliminal
    pub fn is_subliminal(&self, amplitude: f32) -> bool {
        amplitude >= self.subliminal_range.0 && amplitude < self.subliminal_range.1
    }

    /// Process subliminal stimulus
    pub fn process(&self, wave: &MemoryWave) -> Option<SubconsciousEffect> {
        if self.is_subliminal(wave.amplitude) {
            // Subliminal processing occurs below conscious threshold
            Some(SubconsciousEffect {
                priming: wave.frequency / 1000.0, // Normalized frequency as priming strength
                emotional_bias: wave.valence * 0.3, // Reduced emotional influence
                pattern_activation: wave.arousal * 0.2,
            })
        } else {
            None
        }
    }
}

/// Effects of subliminal processing
#[derive(Debug, Clone)]
pub struct SubconsciousEffect {
    /// Priming strength for related concepts
    pub priming: f32,
    /// Emotional bias influence
    pub emotional_bias: f32,
    /// Pattern activation level
    pub pattern_activation: f32,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_reactive_layers() {
        assert_eq!(
            ReactiveLayer::from_response_time(Duration::from_millis(5)),
            ReactiveLayer::HardwareReflex
        );
        assert_eq!(
            ReactiveLayer::from_response_time(Duration::from_millis(30)),
            ReactiveLayer::SubcorticalReaction
        );
        assert_eq!(
            ReactiveLayer::from_response_time(Duration::from_millis(100)),
            ReactiveLayer::EmotionalResponse
        );
        assert_eq!(
            ReactiveLayer::from_response_time(Duration::from_millis(300)),
            ReactiveLayer::ConsciousDeliberation
        );
    }

    #[test]
    fn test_looming_detection() {
        let mut detector = LoomingDetector::new(0.5);

        // Simulate approaching object
        detector.update(0.1);
        std::thread::sleep(Duration::from_millis(100));
        detector.update(0.15);
        std::thread::sleep(Duration::from_millis(100));

        if let Some(urgency) = detector.update(0.25) {
            assert!(urgency > 0.0);
            assert!(urgency <= 1.0);
        }
    }

    #[test]
    fn test_sensor_coherence() {
        let mut coherence = SensorCoherence::new();

        // Add coherent sensors (similar phases)
        coherence.add_sensor(1.0, 0.0);
        coherence.add_sensor(0.8, 0.1);
        coherence.add_sensor(0.9, -0.1);

        let c = coherence.calculate();
        assert!(c > 0.9); // High coherence

        // Add incoherent sensor
        coherence.add_sensor(1.0, std::f32::consts::PI); // Opposite phase
        let c2 = coherence.calculate();
        assert!(c2 < c); // Reduced coherence
    }
}
