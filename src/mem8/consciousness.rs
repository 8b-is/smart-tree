//! Consciousness simulation framework for MEM8
//! Implements awareness, attention allocation, and sensory arbitration

use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use std::time::{Duration, Instant};
use crate::mem8::wave::{MemoryWave, WaveGrid, FrequencyBand};

/// Consciousness state at time t
pub struct ConsciousnessState {
    /// Current attention weights for different memory regions
    pub attention_weights: HashMap<MemoryRegion, f32>,
    /// Active memories in consciousness
    pub active_memories: Vec<Arc<MemoryWave>>,
    /// Reflexive response components
    pub reflexive_responses: Vec<ReflexiveComponent>,
    /// Current awareness level (0.0 to 1.0)
    pub awareness_level: f32,
    /// Last update timestamp
    pub last_update: Instant,
}

impl ConsciousnessState {
    pub fn new() -> Self {
        Self {
            attention_weights: HashMap::new(),
            active_memories: Vec::new(),
            reflexive_responses: Vec::new(),
            awareness_level: 0.5,
            last_update: Instant::now(),
        }
    }

    /// Update consciousness state with new memories and responses
    pub fn update(&mut self, memories: Vec<Arc<MemoryWave>>, responses: Vec<ReflexiveComponent>) {
        self.active_memories = memories;
        self.reflexive_responses = responses;
        self.last_update = Instant::now();
        
        // Update awareness based on activity
        self.awareness_level = self.calculate_awareness();
    }

    /// Calculate current awareness level based on activity
    fn calculate_awareness(&self) -> f32 {
        let memory_activity = (self.active_memories.len() as f32 / 100.0).min(1.0);
        let attention_focus = self.attention_weights.values().sum::<f32>() / self.attention_weights.len().max(1) as f32;
        
        (memory_activity + attention_focus) / 2.0
    }
}

/// Memory region identifiers for attention allocation
#[derive(Debug, Clone, Hash, Eq, PartialEq)]
pub enum MemoryRegion {
    Visual(u8, u8),      // x, y coordinates
    Auditory(u16),       // frequency band
    Temporal(u16),       // time layer (z-axis)
    Semantic(String),    // semantic category
    Emotional(String),   // emotional category
}

/// Reflexive response component
#[derive(Clone)]
pub struct ReflexiveComponent {
    pub trigger: String,
    pub response: String,
    pub strength: f32,
}

/// Multi-grid sensor architecture
pub struct SensorGrid {
    /// Grid identifier
    pub id: String,
    /// Grid type (e.g., "color_r", "motion_h", "edge_0")
    pub grid_type: SensorGridType,
    /// The wave grid itself
    pub grid: Arc<RwLock<WaveGrid>>,
    /// Temporal blanket configuration
    pub temporal_blanket: TemporalBlanket,
}

/// Types of sensor grids
#[derive(Debug, Clone)]
pub enum SensorGridType {
    // Visual grids (10-15 per eye)
    ColorChannel(ColorChannel),
    Motion(MotionDirection),
    EdgeDetection(u16), // Angle in degrees
    Depth,
    Saliency,
    Luminance,
    
    // Audio grids
    FrequencyBand(f32, f32), // Min, max frequency
    Amplitude,
    Phase,
    
    // Other modalities
    Temporal,
    Context,
    Semantic,
}

#[derive(Debug, Clone)]
pub enum ColorChannel {
    Red,
    Green,
    Blue,
}

#[derive(Debug, Clone)]
pub enum MotionDirection {
    Horizontal,
    Vertical,
}

/// Temporal blanket for environmental adaptation
pub struct TemporalBlanket {
    /// Interest-based adjustment factor
    pub alpha: f32,
    /// Attention-based decay rate
    pub lambda: f32,
    /// Environmental calibration
    pub beta_calib: f32,
    /// Hard blankets (fixed calibration patterns)
    pub hard_blankets: Vec<CalibrationPattern>,
    /// Soft blankets (adaptive filters)
    pub soft_blankets: Vec<AdaptiveFilter>,
}

impl TemporalBlanket {
    pub fn new() -> Self {
        Self {
            alpha: 1.0,
            lambda: 0.1,
            beta_calib: 0.0,
            hard_blankets: Vec::new(),
            soft_blankets: Vec::new(),
        }
    }

    /// Calculate blanket value at time t
    pub fn calculate(&self, t: f32, interest: f32) -> f32 {
        self.alpha * (-self.lambda * interest * t).exp() + self.beta_calib
    }

    /// Apply environmental adaptation
    pub fn adapt_to_environment(&mut self, env_changes: &[(String, f32)]) {
        let mut delta_sum = 0.0;
        
        for (change_type, magnitude) in env_changes {
            let weight = match change_type.as_str() {
                "lighting" => 0.4,
                "motion" => 0.3,
                "noise" => 0.2,
                _ => 0.1,
            };
            delta_sum += weight * magnitude;
        }
        
        self.beta_calib = self.beta_calib * 0.9 + delta_sum * 0.1;
    }
}

#[derive(Clone)]
pub struct CalibrationPattern {
    pub name: String,
    pub pattern: Vec<f32>,
}

#[derive(Clone)]
pub struct AdaptiveFilter {
    pub name: String,
    pub strength: f32,
    pub adaptation_rate: f32,
}

/// Sensor arbitration system with human-AI control
pub struct SensorArbitrator {
    /// Human control weight (0.0 to 1.0)
    pub human_weight: f32,
    /// AI control weight (0.0 to 1.0)
    pub ai_weight: f32,
    /// Sensor grids
    pub sensor_grids: HashMap<String, SensorGrid>,
    /// AI interest weights
    pub ai_interests: HashMap<String, f32>,
    /// Subconscious influence weights
    pub subconscious_weights: HashMap<String, f32>,
}

impl SensorArbitrator {
    pub fn new(human_weight: f32, ai_weight: f32) -> Self {
        assert!((human_weight + ai_weight - 1.0).abs() < 0.001, "Weights must sum to 1.0");
        
        Self {
            human_weight,
            ai_weight,
            sensor_grids: HashMap::new(),
            ai_interests: HashMap::new(),
            subconscious_weights: HashMap::new(),
        }
    }

    /// Calculate weighted sensor output
    pub fn arbitrate(&self, sensor_id: &str, human_value: f32, ai_value: f32) -> f32 {
        self.human_weight * human_value + self.ai_weight * ai_value
    }

    /// Calculate weighted interest for a sensor
    pub fn calculate_weighted_interest(&self, sensor_id: &str, base_interest: f32) -> f32 {
        let subconscious_weight = self.subconscious_weights.get(sensor_id).unwrap_or(&0.0);
        let ai_weight = self.ai_interests.get(sensor_id).unwrap_or(&0.0);
        
        base_interest + 0.3 * base_interest * subconscious_weight + 0.7 * base_interest * ai_weight
    }

    /// Check if AI can override noise floor
    pub fn should_process(&self, sensor_id: &str, signal_strength: f32, noise_floor: f32) -> bool {
        let ai_weight = self.ai_interests.get(sensor_id).unwrap_or(&0.0);
        
        // AI override when weight > 0.8
        if *ai_weight > 0.8 {
            return true;
        }
        
        // Normal processing
        let weighted_interest = self.calculate_weighted_interest(sensor_id, signal_strength);
        weighted_interest > noise_floor
    }
}

/// Consciousness simulation engine
pub struct ConsciousnessEngine {
    /// Wave grid for memory storage
    pub wave_grid: Arc<RwLock<WaveGrid>>,
    /// Current consciousness state
    pub state: RwLock<ConsciousnessState>,
    /// Sensor arbitrator
    pub arbitrator: SensorArbitrator,
    /// Attention allocation strategy
    pub attention_strategy: AttentionStrategy,
}

impl ConsciousnessEngine {
    pub fn new(wave_grid: Arc<RwLock<WaveGrid>>) -> Self {
        Self {
            wave_grid,
            state: RwLock::new(ConsciousnessState::new()),
            arbitrator: SensorArbitrator::new(0.3, 0.7), // 30% human, 70% AI control
            attention_strategy: AttentionStrategy::default(),
        }
    }

    /// Update consciousness state based on current memories
    pub fn update(&self) {
        let grid = self.wave_grid.read().unwrap();
        let mut state = self.state.write().unwrap();
        
        // Collect active memories based on attention
        let active_memories = self.collect_active_memories(&grid);
        
        // Update attention weights
        self.update_attention_weights(&mut state, &active_memories);
        
        // Generate reflexive responses
        let reflexive = self.generate_reflexive_responses(&active_memories);
        
        state.update(active_memories, reflexive);
    }

    /// Collect memories that are currently active in consciousness
    fn collect_active_memories(&self, grid: &WaveGrid) -> Vec<Arc<MemoryWave>> {
        let mut active = Vec::new();
        let attention_threshold = 0.3;
        
        // Sample based on attention weights
        let state = self.state.read().unwrap();
        
        for (region, &weight) in &state.attention_weights {
            if weight > attention_threshold {
                // Sample memories from this region
                match region {
                    MemoryRegion::Visual(x, y) => {
                        // Sample around visual coordinates
                        for z in 0..100 {
                            if let Some(wave) = grid.get(*x, *y, z) {
                                if wave.calculate_decay() > 0.1 {
                                    active.push(wave.clone());
                                }
                            }
                        }
                    }
                    MemoryRegion::Temporal(z) => {
                        // Sample from temporal layer
                        for x in 0..16 {
                            for y in 0..16 {
                                if let Some(wave) = grid.get(x * 16, y * 16, *z) {
                                    if wave.calculate_decay() > 0.1 {
                                        active.push(wave.clone());
                                    }
                                }
                            }
                        }
                    }
                    _ => {} // Handle other regions
                }
            }
        }
        
        active
    }

    /// Update attention weights based on current activity
    fn update_attention_weights(&self, state: &mut ConsciousnessState, memories: &[Arc<MemoryWave>]) {
        // Decay existing weights
        for weight in state.attention_weights.values_mut() {
            *weight *= 0.95;
        }
        
        // Boost weights for active memory regions
        for memory in memories {
            // Determine region based on frequency
            let band = FrequencyBand::from_frequency(memory.frequency);
            let region = match band {
                FrequencyBand::DeepStructural => MemoryRegion::Semantic("structural".to_string()),
                FrequencyBand::Conversational => MemoryRegion::Semantic("conversational".to_string()),
                FrequencyBand::Technical => MemoryRegion::Semantic("technical".to_string()),
                FrequencyBand::Implementation => MemoryRegion::Semantic("implementation".to_string()),
                FrequencyBand::Abstract => MemoryRegion::Semantic("abstract".to_string()),
            };
            
            *state.attention_weights.entry(region).or_insert(0.0) += 0.1;
        }
        
        // Normalize weights
        let sum: f32 = state.attention_weights.values().sum();
        if sum > 0.0 {
            for weight in state.attention_weights.values_mut() {
                *weight /= sum;
            }
        }
    }

    /// Generate reflexive responses based on active memories
    fn generate_reflexive_responses(&self, memories: &[Arc<MemoryWave>]) -> Vec<ReflexiveComponent> {
        let mut responses = Vec::new();
        
        for memory in memories {
            // High arousal memories trigger reflexive responses
            if memory.arousal > 0.7 {
                responses.push(ReflexiveComponent {
                    trigger: format!("High arousal memory ({}Hz)", memory.frequency),
                    response: "Heightened attention".to_string(),
                    strength: memory.arousal,
                });
            }
            
            // Negative valence with high amplitude
            if memory.valence < -0.5 && memory.amplitude > 0.8 {
                responses.push(ReflexiveComponent {
                    trigger: "Negative high-amplitude memory".to_string(),
                    response: "Defensive stance".to_string(),
                    strength: memory.amplitude * memory.valence.abs(),
                });
            }
        }
        
        responses
    }
}

/// Attention allocation strategies
#[derive(Debug, Clone)]
pub enum AttentionStrategy {
    /// Focus on high-amplitude memories
    AmplitudeBased,
    /// Focus on emotionally salient memories
    EmotionBased,
    /// Focus on novel/unfamiliar patterns
    NoveltyBased,
    /// Balanced across all factors
    Balanced,
}

impl Default for AttentionStrategy {
    fn default() -> Self {
        Self::Balanced
    }
}

/// Subliminal forgetting processor
pub struct ForgettingProcessor {
    /// Processing frequency (Hz)
    pub frequency: f32,
    /// Forgetting curves
    pub curves: HashMap<String, ForgetCurve>,
}

impl ForgettingProcessor {
    pub fn new() -> Self {
        let mut curves = HashMap::new();
        
        // Define standard forgetting curves
        curves.insert("flash".to_string(), ForgetCurve::Flash(Duration::from_millis(500)));
        curves.insert("fade".to_string(), ForgetCurve::Fade(Duration::from_secs(5)));
        curves.insert("linger".to_string(), ForgetCurve::Linger(Duration::from_secs(30)));
        curves.insert("persist".to_string(), ForgetCurve::Persist(Duration::from_secs(300)));
        curves.insert("consolidate".to_string(), ForgetCurve::Consolidate);
        
        Self {
            frequency: 100.0, // 100Hz processing
            curves,
        }
    }

    /// Process memory for context-aware forgetting
    pub fn process(&self, memory: &mut MemoryWave, context: &str) -> ForgetCurve {
        match context {
            "transient_detail" => ForgetCurve::Flash(Duration::from_millis(500)),
            "resolved_threat" => ForgetCurve::Fade(Duration::from_secs(5)),
            "familiar_anomaly" => ForgetCurve::Linger(Duration::from_secs(30)),
            "actionable_info" => ForgetCurve::Persist(Duration::from_secs(300)),
            "learned_pattern" => ForgetCurve::Consolidate,
            _ => ForgetCurve::Fade(Duration::from_secs(10)),
        }
    }
}

/// Forgetting curve types
#[derive(Debug, Clone)]
pub enum ForgetCurve {
    Flash(Duration),    // Very short retention
    Fade(Duration),     // Quick fade
    Linger(Duration),   // Medium retention
    Persist(Duration),  // Long retention
    Consolidate,        // Permanent memory
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_consciousness_state() {
        let mut state = ConsciousnessState::new();
        assert_eq!(state.awareness_level, 0.5);
        
        // Add some attention weights
        state.attention_weights.insert(MemoryRegion::Visual(128, 128), 0.8);
        state.attention_weights.insert(MemoryRegion::Temporal(1000), 0.6);
        
        // Update with active memories
        let wave = Arc::new(MemoryWave::new(440.0, 0.8));
        state.update(vec![wave], vec![]);
        
        assert!(state.awareness_level > 0.5);
    }

    #[test]
    fn test_sensor_arbitration() {
        let arbitrator = SensorArbitrator::new(0.3, 0.7);
        
        let human_value = 0.5;
        let ai_value = 0.8;
        
        let result = arbitrator.arbitrate("test_sensor", human_value, ai_value);
        assert!((result - (0.3 * 0.5 + 0.7 * 0.8)).abs() < 0.001);
    }

    #[test]
    fn test_ai_override() {
        let mut arbitrator = SensorArbitrator::new(0.3, 0.7);
        arbitrator.ai_interests.insert("critical_sensor".to_string(), 0.9);
        
        // AI override should process even below noise floor
        assert!(arbitrator.should_process("critical_sensor", 0.05, 0.1));
    }
}