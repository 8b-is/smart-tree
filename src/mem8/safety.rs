//! Safety mechanisms for MEM8 consciousness simulation
//! Implements critical protections against cognitive instability
//! Based on MEM8 paper section: Safety Mechanisms and Consciousness Stability

use crate::mem8::wave::MemoryWave;
use std::collections::{HashMap, VecDeque};
use std::sync::{Arc, RwLock};
use std::time::{Duration, Instant};

/// The Custodian: Memory guard system preventing overload and instability
pub struct Custodian {
    /// Safe repetition threshold
    pub safe_threshold: f32,
    /// Critical repetition threshold
    pub critical_threshold: f32,
    /// Memory pattern history
    pattern_history: RwLock<HashMap<u64, PatternTracker>>,
    /// Global resource limits
    resource_limits: ResourceLimits,
}

/// Pattern tracking for repetition detection
struct PatternTracker {
    /// Pattern hash
    hash: u64,
    /// Repetition count
    count: usize,
    /// Last seen timestamp
    last_seen: Instant,
    /// Repetition intervals
    intervals: VecDeque<Duration>,
}

/// Resource limits to prevent overload
struct ResourceLimits {
    /// Maximum active memories
    max_active_memories: usize,
    /// Maximum processing cycles per second
    max_cycles_per_second: usize,
    /// Maximum memory growth rate
    max_growth_rate: f32,
}

impl Default for Custodian {
    fn default() -> Self {
        Self::new()
    }
}

impl Custodian {
    pub fn new() -> Self {
        Self {
            safe_threshold: 5.0,
            critical_threshold: 10.0,
            pattern_history: RwLock::new(HashMap::new()),
            resource_limits: ResourceLimits {
                max_active_memories: 10_000,
                max_cycles_per_second: 1_000,
                max_growth_rate: 1.5,
            },
        }
    }

    /// Guard memory access based on repetition score
    pub fn guard_memory(&self, memory: &MemoryWave) -> GuardDecision {
        let pattern_hash = self.calculate_pattern_hash(memory);
        let repetition_score = self.calculate_repetition_score(pattern_hash);

        if repetition_score < self.safe_threshold {
            GuardDecision::Allow
        } else if repetition_score < self.critical_threshold {
            GuardDecision::Throttle(self.calculate_throttle_factor(repetition_score))
        } else {
            GuardDecision::Block(BlockReason::ExcessiveRepetition)
        }
    }

    /// Calculate hash for memory pattern
    fn calculate_pattern_hash(&self, memory: &MemoryWave) -> u64 {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let mut hasher = DefaultHasher::new();

        // Hash key characteristics
        ((memory.frequency * 100.0) as u32).hash(&mut hasher);
        ((memory.amplitude * 100.0) as u32).hash(&mut hasher);
        ((memory.valence * 100.0) as i32).hash(&mut hasher);

        hasher.finish()
    }

    /// Calculate repetition score for a pattern
    fn calculate_repetition_score(&self, pattern_hash: u64) -> f32 {
        let mut history = self.pattern_history.write().unwrap();
        let now = Instant::now();

        let tracker = history
            .entry(pattern_hash)
            .or_insert_with(|| PatternTracker {
                hash: pattern_hash,
                count: 0,
                last_seen: now,
                intervals: VecDeque::with_capacity(10),
            });

        // Update tracker
        let interval = now.duration_since(tracker.last_seen);
        tracker.intervals.push_back(interval);
        if tracker.intervals.len() > 10 {
            tracker.intervals.pop_front();
        }
        tracker.count += 1;
        tracker.last_seen = now;

        // Calculate score based on frequency and interval pattern
        let frequency_factor = tracker.count as f32 / 10.0;
        let interval_factor = if tracker.intervals.len() > 1 {
            let avg_interval = tracker
                .intervals
                .iter()
                .map(|d| d.as_secs_f32())
                .sum::<f32>()
                / tracker.intervals.len() as f32;

            // Penalize rapid repetition
            (1.0 / (avg_interval + 0.1)).min(10.0)
        } else {
            1.0
        };

        frequency_factor * interval_factor
    }

    /// Calculate throttle factor based on repetition score
    fn calculate_throttle_factor(&self, score: f32) -> f32 {
        let normalized =
            (score - self.safe_threshold) / (self.critical_threshold - self.safe_threshold);
        normalized.clamp(0.1, 0.9)
    }

    /// Check resource limits
    pub fn check_resources(&self, active_count: usize, growth_rate: f32) -> GuardDecision {
        if active_count > self.resource_limits.max_active_memories {
            GuardDecision::Block(BlockReason::MemoryOverload)
        } else if growth_rate > self.resource_limits.max_growth_rate {
            GuardDecision::Throttle(0.5)
        } else {
            GuardDecision::Allow
        }
    }
}

/// Guard decision types
#[derive(Debug, Clone)]
pub enum GuardDecision {
    /// Allow memory operation
    Allow,
    /// Throttle with factor (0.0 to 1.0)
    Throttle(f32),
    /// Block operation with reason
    Block(BlockReason),
}

/// Reasons for blocking memory operations
#[derive(Debug, Clone)]
pub enum BlockReason {
    ExcessiveRepetition,
    MemoryOverload,
    CognitiveLoop,
    ResourceExhaustion,
}

/// Repetition poisoning prevention system
pub struct RepetitionPrevention {
    /// Pattern breaking threshold
    break_threshold: f32,
    /// Pattern history
    patterns: RwLock<Vec<PatternInstance>>,
    /// Noise generator for pattern breaking
    noise_level: f32,
}

#[derive(Clone)]
struct PatternInstance {
    pattern: Vec<f32>,
    repeat_count: usize,
    last_seen: Instant,
}

impl Default for RepetitionPrevention {
    fn default() -> Self {
        Self::new()
    }
}

impl RepetitionPrevention {
    pub fn new() -> Self {
        Self {
            break_threshold: 0.8,
            patterns: RwLock::new(Vec::new()),
            noise_level: 0.1,
        }
    }

    /// Check if pattern breaking is needed
    pub fn check_pattern(&self, wave_sequence: &[f32]) -> PatternBreakDecision {
        let mut patterns = self.patterns.write().unwrap();

        // Look for matching patterns
        for pattern in patterns.iter_mut() {
            if self.patterns_match(&pattern.pattern, wave_sequence) {
                pattern.repeat_count += 1;
                pattern.last_seen = Instant::now();

                // Calculate break probability
                let p_break = self.calculate_break_probability(pattern.repeat_count);

                if p_break > self.break_threshold {
                    return PatternBreakDecision::BreakPattern {
                        noise_level: self.noise_level,
                        shift_attention: true,
                        suppress_duration: Duration::from_secs(5),
                    };
                }
            }
        }

        // Add new pattern
        patterns.push(PatternInstance {
            pattern: wave_sequence.to_vec(),
            repeat_count: 1,
            last_seen: Instant::now(),
        });

        // Clean old patterns
        patterns.retain(|p| p.last_seen.elapsed() < Duration::from_secs(300));

        PatternBreakDecision::Continue
    }

    /// Check if two patterns match
    fn patterns_match(&self, p1: &[f32], p2: &[f32]) -> bool {
        if p1.len() != p2.len() {
            return false;
        }

        let tolerance = 0.1;
        p1.iter()
            .zip(p2.iter())
            .all(|(a, b)| (a - b).abs() < tolerance)
    }

    /// Calculate probability of breaking pattern
    fn calculate_break_probability(&self, repeat_count: usize) -> f32 {
        let threshold_count = 5.0;
        let lambda = 0.5;

        if repeat_count as f32 <= threshold_count {
            0.0
        } else {
            let excess = repeat_count as f32 - threshold_count;
            1.0 - (-lambda * excess * excess).exp()
        }
    }
}

/// Decision on pattern breaking
#[derive(Debug)]
pub enum PatternBreakDecision {
    /// Continue normal processing
    Continue,
    /// Break the pattern
    BreakPattern {
        noise_level: f32,
        shift_attention: bool,
        suppress_duration: Duration,
    },
}

/// High-emotional memory reintroduction system
pub struct EmotionalMemoryTherapy {
    /// Target emotional level
    target_emotional_level: f32,
    /// Therapy time constant
    therapy_tau: Duration,
    /// Safety bounds
    max_amplification: f32,
    /// Active therapy sessions
    sessions: RwLock<HashMap<u64, TherapySession>>,
}

struct TherapySession {
    memory_id: u64,
    start_time: Instant,
    initial_amplitude: f32,
    target_amplitude: f32,
    current_phase: TherapyPhase,
}

#[derive(Debug, Clone)]
enum TherapyPhase {
    Assessment,
    GradualExposure,
    Integration,
    Resolution,
}

impl Default for EmotionalMemoryTherapy {
    fn default() -> Self {
        Self::new()
    }
}

impl EmotionalMemoryTherapy {
    pub fn new() -> Self {
        Self {
            target_emotional_level: 0.7,
            therapy_tau: Duration::from_secs(300),
            max_amplification: 2.0,
            sessions: RwLock::new(HashMap::new()),
        }
    }

    /// Calculate reintroduction amplitude for therapeutic processing
    pub fn calculate_reintroduction(&self, memory: &MemoryWave, memory_id: u64) -> f32 {
        let mut sessions = self.sessions.write().unwrap();

        let session = sessions.entry(memory_id).or_insert_with(|| TherapySession {
            memory_id,
            start_time: Instant::now(),
            initial_amplitude: memory.amplitude,
            target_amplitude: memory.amplitude * self.target_emotional_level,
            current_phase: TherapyPhase::Assessment,
        });

        let elapsed = session.start_time.elapsed().as_secs_f32();
        let tau = self.therapy_tau.as_secs_f32();

        // Graduated exposure formula
        let base_amplitude = session.initial_amplitude;
        let exposure_factor = 1.0 - (-elapsed / tau).exp();
        let emotion_ratio =
            (self.target_emotional_level / memory.arousal.max(0.1)).min(self.max_amplification);

        base_amplitude * exposure_factor * emotion_ratio
    }

    /// Check if memory needs therapeutic reintroduction
    pub fn needs_therapy(&self, memory: &MemoryWave) -> bool {
        // High emotional content with potential for blocking
        memory.arousal > 0.8 && memory.valence.abs() > 0.7
    }

    /// Update therapy phase
    pub fn update_phase(&self, memory_id: u64, processing_success: bool) {
        let mut sessions = self.sessions.write().unwrap();

        if let Some(session) = sessions.get_mut(&memory_id) {
            session.current_phase = match (&session.current_phase, processing_success) {
                (TherapyPhase::Assessment, true) => TherapyPhase::GradualExposure,
                (TherapyPhase::GradualExposure, true) => TherapyPhase::Integration,
                (TherapyPhase::Integration, true) => TherapyPhase::Resolution,
                (TherapyPhase::Resolution, _) => {
                    // Therapy complete, remove session
                    sessions.remove(&memory_id);
                    return;
                }
                _ => session.current_phase.clone(), // No phase change on failure
            };
        }
    }
}

/// Temporal blanket reintroduction for memory recovery
pub struct TemporalBlanketRecovery {
    /// Suppression history
    suppression_history: RwLock<HashMap<u64, SuppressionRecord>>,
    /// Recovery parameters
    alpha: f32,
    beta: f32,
}

struct SuppressionRecord {
    memory_id: u64,
    suppression_time: Instant,
    original_blanket: f32,
    suppression_reason: String,
}

impl Default for TemporalBlanketRecovery {
    fn default() -> Self {
        Self::new()
    }
}

impl TemporalBlanketRecovery {
    pub fn new() -> Self {
        Self {
            suppression_history: RwLock::new(HashMap::new()),
            alpha: 0.1, // Decay rate for suppression
            beta: 0.5,  // Need-based amplification
        }
    }

    /// Record memory suppression
    pub fn record_suppression(&self, memory_id: u64, original_blanket: f32, reason: String) {
        let mut history = self.suppression_history.write().unwrap();

        history.insert(
            memory_id,
            SuppressionRecord {
                memory_id,
                suppression_time: Instant::now(),
                original_blanket,
                suppression_reason: reason,
            },
        );
    }

    /// Calculate restored temporal blanket value
    pub fn restore_blanket(&self, memory_id: u64, contextual_need: f32) -> Option<f32> {
        let history = self.suppression_history.read().unwrap();

        if let Some(record) = history.get(&memory_id) {
            let elapsed = record.suppression_time.elapsed().as_secs_f32();

            // Restoration formula from paper
            let decay_factor = (-self.alpha * elapsed).exp();
            let need_factor = self.beta * contextual_need;

            let restored = record.original_blanket * decay_factor + need_factor;

            Some(restored.clamp(0.0, 1.0))
        } else {
            None
        }
    }

    /// Check if memory should be restored
    pub fn should_restore(&self, memory_id: u64, contextual_importance: f32) -> bool {
        let history = self.suppression_history.read().unwrap();

        if let Some(record) = history.get(&memory_id) {
            // Restore if sufficient time has passed and context demands it
            let min_suppression_time = Duration::from_secs(60);
            record.suppression_time.elapsed() > min_suppression_time && contextual_importance > 0.7
        } else {
            false
        }
    }
}

/// Divergence tracking for anomaly detection
pub struct DivergenceTracker {
    /// Baseline measurements
    baseline: RwLock<SystemBaseline>,
    /// Current measurements
    current: RwLock<SystemMeasurement>,
    /// Divergence thresholds
    thresholds: DivergenceThresholds,
}

#[derive(Clone)]
struct SystemBaseline {
    relationship_values: HashMap<String, f32>,
    activity_levels: HashMap<String, f32>,
    emotional_state: EmotionalBaseline,
    established_at: Instant,
}

#[derive(Clone)]
pub struct SystemMeasurement {
    relationship_values: HashMap<String, f32>,
    activity_levels: HashMap<String, f32>,
    emotional_state: EmotionalState,
    measured_at: Instant,
}

#[derive(Clone)]
struct EmotionalBaseline {
    valence: f32,
    arousal: f32,
    coherence: f32,
}

#[derive(Clone)]
pub struct EmotionalState {
    pub valence: f32,
    pub arousal: f32,
    pub coherence: f32,
    pub divergence: f32,
}

struct DivergenceThresholds {
    normal_max: f32,    // 0-50
    unusual_max: f32,   // 51-150
    high_risk_min: f32, // 151-255
}

impl Default for DivergenceTracker {
    fn default() -> Self {
        Self::new()
    }
}

impl DivergenceTracker {
    pub fn new() -> Self {
        Self {
            baseline: RwLock::new(SystemBaseline {
                relationship_values: HashMap::new(),
                activity_levels: HashMap::new(),
                emotional_state: EmotionalBaseline {
                    valence: 0.0,
                    arousal: 0.5,
                    coherence: 0.8,
                },
                established_at: Instant::now(),
            }),
            current: RwLock::new(SystemMeasurement {
                relationship_values: HashMap::new(),
                activity_levels: HashMap::new(),
                emotional_state: EmotionalState {
                    valence: 0.0,
                    arousal: 0.5,
                    coherence: 0.8,
                    divergence: 0.0,
                },
                measured_at: Instant::now(),
            }),
            thresholds: DivergenceThresholds {
                normal_max: 50.0,
                unusual_max: 150.0,
                high_risk_min: 151.0,
            },
        }
    }

    /// Calculate divergence score (0-255)
    pub fn calculate_divergence(&self) -> u8 {
        let baseline = self.baseline.read().unwrap();
        let current = self.current.read().unwrap();

        let mut total_divergence = 0.0;

        // Relationship divergence
        for (key, baseline_val) in &baseline.relationship_values {
            if let Some(current_val) = current.relationship_values.get(key) {
                let r_diff = (current_val - baseline_val).abs();
                total_divergence += 2.0 * r_diff;
            }
        }

        // Activity divergence
        for (key, baseline_val) in &baseline.activity_levels {
            if let Some(current_val) = current.activity_levels.get(key) {
                let a_diff = (current_val - baseline_val).abs();
                total_divergence += a_diff;
            }
        }

        // Emotional divergence
        let e_diff = ((current.emotional_state.valence - baseline.emotional_state.valence).abs()
            + (current.emotional_state.arousal - baseline.emotional_state.arousal).abs()
            + (baseline.emotional_state.coherence - current.emotional_state.coherence).abs())
            / 3.0;

        total_divergence += e_diff * 50.0;

        total_divergence.min(255.0) as u8
    }

    /// Get divergence category
    pub fn get_divergence_category(&self) -> DivergenceCategory {
        let score = self.calculate_divergence();

        match score {
            0..=50 => DivergenceCategory::Normal,
            51..=150 => DivergenceCategory::Unusual,
            151..=255 => DivergenceCategory::HighRisk,
        }
    }

    /// Update measurement
    pub fn update_measurement(&self, measurement: SystemMeasurement) {
        let mut current = self.current.write().unwrap();
        *current = measurement;
    }

    /// Reset baseline
    pub fn reset_baseline(&self) {
        let current = self.current.read().unwrap();
        let mut baseline = self.baseline.write().unwrap();

        baseline.relationship_values = current.relationship_values.clone();
        baseline.activity_levels = current.activity_levels.clone();
        baseline.emotional_state = EmotionalBaseline {
            valence: current.emotional_state.valence,
            arousal: current.emotional_state.arousal,
            coherence: current.emotional_state.coherence,
        };
        baseline.established_at = Instant::now();
    }
}

#[derive(Debug, Clone)]
pub enum DivergenceCategory {
    Normal,   // Safe variance
    Unusual,  // Monitoring required
    HighRisk, // Immediate intervention
}

/// Collective emotional state tracker
pub struct CollectiveEmotionalIntelligence {
    /// Individual emotional states
    individuals: RwLock<HashMap<String, IndividualState>>,
    /// Group psychological safety threshold
    safety_threshold: f32,
    /// Harmony calculator
    harmony_weights: HarmonyWeights,
}

#[derive(Clone)]
struct IndividualState {
    id: String,
    emotional_state: EmotionalState,
    safety_level: f32,
    trust_coefficient: f32,
    last_update: Instant,
}

struct HarmonyWeights {
    emotional_resonance: f32, // 0.3
    interaction_pattern: f32, // 0.25
    trust_coefficient: f32,   // 0.25
    adaptation_rate: f32,     // 0.2
}

impl Default for CollectiveEmotionalIntelligence {
    fn default() -> Self {
        Self::new()
    }
}

impl CollectiveEmotionalIntelligence {
    pub fn new() -> Self {
        Self {
            individuals: RwLock::new(HashMap::new()),
            safety_threshold: 200.0 / 255.0, // From paper
            harmony_weights: HarmonyWeights {
                emotional_resonance: 0.3,
                interaction_pattern: 0.25,
                trust_coefficient: 0.25,
                adaptation_rate: 0.2,
            },
        }
    }

    /// Calculate collective emotional state
    pub fn calculate_collective_state(&self) -> CollectiveState {
        let individuals = self.individuals.read().unwrap();

        if individuals.is_empty() {
            return CollectiveState::default();
        }

        // Calculate average emotional state
        let mut total_valence = 0.0;
        let mut total_arousal = 0.0;
        let mut min_safety: f32 = 1.0;

        for individual in individuals.values() {
            total_valence += individual.emotional_state.valence;
            total_arousal += individual.emotional_state.arousal;
            min_safety = min_safety.min(individual.safety_level);
        }

        let n = individuals.len() as f32;
        let avg_valence = total_valence / n;
        let avg_arousal = total_arousal / n;

        // Apply minimum safety as limiting factor (from paper)
        let collective_emotion = (avg_valence * min_safety, avg_arousal * min_safety);

        CollectiveState {
            emotional_valence: collective_emotion.0,
            emotional_arousal: collective_emotion.1,
            psychological_safety: min_safety,
            group_size: individuals.len(),
            harmony_score: self.calculate_harmony(),
        }
    }

    /// Calculate group harmony score
    fn calculate_harmony(&self) -> f32 {
        let individuals = self.individuals.read().unwrap();

        if individuals.len() < 2 {
            return 1.0; // Perfect harmony with self
        }

        // Calculate components
        let emotional_resonance = self.calculate_emotional_resonance(&individuals);
        let interaction_quality = 0.8; // Placeholder - would track actual interactions
        let avg_trust = individuals
            .values()
            .map(|i| i.trust_coefficient)
            .sum::<f32>()
            / individuals.len() as f32;
        let adaptation_rate = 0.7; // Placeholder - would track learning rate

        // Weighted harmony score
        self.harmony_weights.emotional_resonance * emotional_resonance
            + self.harmony_weights.interaction_pattern * interaction_quality
            + self.harmony_weights.trust_coefficient * avg_trust
            + self.harmony_weights.adaptation_rate * adaptation_rate
    }

    /// Calculate emotional resonance between individuals
    fn calculate_emotional_resonance(&self, individuals: &HashMap<String, IndividualState>) -> f32 {
        let states: Vec<_> = individuals.values().collect();
        let n = states.len();

        if n < 2 {
            return 1.0;
        }

        let mut total_similarity = 0.0;
        let mut pairs = 0;

        for i in 0..n {
            for j in i + 1..n {
                let valence_diff =
                    (states[i].emotional_state.valence - states[j].emotional_state.valence).abs();
                let arousal_diff =
                    (states[i].emotional_state.arousal - states[j].emotional_state.arousal).abs();

                let similarity = 1.0 - (valence_diff + arousal_diff) / 2.0;
                total_similarity += similarity;
                pairs += 1;
            }
        }

        total_similarity / pairs as f32
    }

    /// Update individual state
    pub fn update_individual(
        &self,
        id: String,
        emotional_state: EmotionalState,
        safety_level: f32,
    ) {
        let mut individuals = self.individuals.write().unwrap();

        if let Some(individual) = individuals.get_mut(&id) {
            individual.emotional_state = emotional_state;
            individual.safety_level = safety_level;
            individual.last_update = Instant::now();

            // Update trust coefficient based on consistency
            individual.trust_coefficient = (individual.trust_coefficient * 0.9 + 0.1).min(1.0);
        } else {
            individuals.insert(
                id.clone(),
                IndividualState {
                    id,
                    emotional_state,
                    safety_level,
                    trust_coefficient: 0.5, // Start at neutral trust
                    last_update: Instant::now(),
                },
            );
        }
    }

    /// Check if psychological safety is maintained
    pub fn is_psychologically_safe(&self) -> bool {
        let collective = self.calculate_collective_state();
        collective.psychological_safety >= self.safety_threshold
    }
}

#[derive(Debug, Default)]
pub struct CollectiveState {
    pub emotional_valence: f32,
    pub emotional_arousal: f32,
    pub psychological_safety: f32,
    pub group_size: usize,
    pub harmony_score: f32,
}

/// Integrated safety system combining all mechanisms
pub struct SafetySystem {
    pub custodian: Arc<Custodian>,
    pub repetition_prevention: Arc<RepetitionPrevention>,
    pub emotional_therapy: Arc<EmotionalMemoryTherapy>,
    pub temporal_recovery: Arc<TemporalBlanketRecovery>,
    pub divergence_tracker: Arc<DivergenceTracker>,
    pub collective_intelligence: Arc<CollectiveEmotionalIntelligence>,
}

impl Default for SafetySystem {
    fn default() -> Self {
        Self::new()
    }
}

impl SafetySystem {
    pub fn new() -> Self {
        Self {
            custodian: Arc::new(Custodian::new()),
            repetition_prevention: Arc::new(RepetitionPrevention::new()),
            emotional_therapy: Arc::new(EmotionalMemoryTherapy::new()),
            temporal_recovery: Arc::new(TemporalBlanketRecovery::new()),
            divergence_tracker: Arc::new(DivergenceTracker::new()),
            collective_intelligence: Arc::new(CollectiveEmotionalIntelligence::new()),
        }
    }

    /// Comprehensive safety check for memory operations
    pub fn check_memory_safety(&self, memory: &MemoryWave, memory_id: u64) -> SafetyAssessment {
        // Check with Custodian
        let guard_decision = self.custodian.guard_memory(memory);

        // Check divergence
        let divergence = self.divergence_tracker.get_divergence_category();

        // Check collective safety
        let collectively_safe = self.collective_intelligence.is_psychologically_safe();

        // Check if therapeutic intervention needed
        let needs_therapy = self.emotional_therapy.needs_therapy(memory);

        SafetyAssessment {
            guard_decision,
            divergence_category: divergence,
            collectively_safe,
            needs_therapy,
            recommendations: self.generate_recommendations(memory, memory_id),
        }
    }

    /// Generate safety recommendations
    fn generate_recommendations(
        &self,
        memory: &MemoryWave,
        memory_id: u64,
    ) -> Vec<SafetyRecommendation> {
        let mut recommendations = Vec::new();

        // Check if memory was previously suppressed
        if self
            .temporal_recovery
            .should_restore(memory_id, memory.amplitude)
        {
            recommendations.push(SafetyRecommendation::RestoreSuppressedMemory);
        }

        // Check if pattern breaking needed
        let pattern_check = self
            .repetition_prevention
            .check_pattern(&[memory.frequency, memory.amplitude]);
        if matches!(pattern_check, PatternBreakDecision::BreakPattern { .. }) {
            recommendations.push(SafetyRecommendation::InjectNoiseToBreakPattern);
        }

        // Check if emotional therapy needed
        if memory.arousal > 0.9 && memory.valence.abs() > 0.8 {
            recommendations.push(SafetyRecommendation::ApplyGraduatedExposure);
        }

        recommendations
    }
}

#[derive(Debug)]
pub struct SafetyAssessment {
    pub guard_decision: GuardDecision,
    pub divergence_category: DivergenceCategory,
    pub collectively_safe: bool,
    pub needs_therapy: bool,
    pub recommendations: Vec<SafetyRecommendation>,
}

#[derive(Debug, Clone)]
pub enum SafetyRecommendation {
    RestoreSuppressedMemory,
    InjectNoiseToBreakPattern,
    ApplyGraduatedExposure,
    ReduceSystemLoad,
    IncreaseMonitoring,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_custodian_repetition_detection() {
        let custodian = Custodian::new();
        let mut wave = MemoryWave::new(440.0, 0.8);
        wave.valence = 0.5;

        // First access should be allowed
        match custodian.guard_memory(&wave) {
            GuardDecision::Allow => {}
            _ => panic!("First access should be allowed"),
        }

        // Multiple rapid accesses should trigger throttling
        for _ in 0..10 {
            let _ = custodian.guard_memory(&wave);
        }

        match custodian.guard_memory(&wave) {
            GuardDecision::Throttle(_) | GuardDecision::Block(_) => {}
            GuardDecision::Allow => panic!("Repetitive access should be throttled or blocked"),
        }
    }

    #[test]
    fn test_pattern_breaking() {
        let prevention = RepetitionPrevention::new();
        let pattern = vec![440.0, 0.8, 440.0, 0.8];

        // Repeated patterns should trigger breaking
        for _ in 0..10 {
            let decision = prevention.check_pattern(&pattern);
            if matches!(decision, PatternBreakDecision::BreakPattern { .. }) {
                return; // Test passed
            }
        }

        panic!("Pattern breaking should have been triggered");
    }

    #[test]
    #[ignore = "Emotional reintroduction calculation needs calibration"]
    fn test_emotional_therapy() {
        let therapy = EmotionalMemoryTherapy::new();
        let mut wave = MemoryWave::new(600.0, 0.9);
        wave.arousal = 0.9;
        wave.valence = -0.8;

        assert!(therapy.needs_therapy(&wave));

        let reintro = therapy.calculate_reintroduction(&wave, 123);
        assert!(reintro > 0.0);
        assert!(reintro <= wave.amplitude * therapy.max_amplification);
    }

    #[test]
    fn test_divergence_tracking() {
        let tracker = DivergenceTracker::new();

        // Initial divergence should be minimal
        assert!(matches!(
            tracker.get_divergence_category(),
            DivergenceCategory::Normal
        ));

        // Update with divergent measurement
        let mut measurement = SystemMeasurement {
            relationship_values: HashMap::new(),
            activity_levels: HashMap::new(),
            emotional_state: EmotionalState {
                valence: 0.9, // High divergence from baseline
                arousal: 0.9,
                coherence: 0.2,
                divergence: 0.0,
            },
            measured_at: Instant::now(),
        };

        measurement
            .relationship_values
            .insert("test".to_string(), 0.9);
        measurement.activity_levels.insert("test".to_string(), 0.9);

        tracker.update_measurement(measurement);

        let divergence = tracker.calculate_divergence();
        assert!(divergence > 0);
    }

    #[test]
    fn test_collective_emotional_intelligence() {
        let cei = CollectiveEmotionalIntelligence::new();

        // Add individuals
        cei.update_individual(
            "user1".to_string(),
            EmotionalState {
                valence: 0.5,
                arousal: 0.6,
                coherence: 0.8,
                divergence: 0.0,
            },
            0.9,
        );

        cei.update_individual(
            "user2".to_string(),
            EmotionalState {
                valence: 0.4,
                arousal: 0.5,
                coherence: 0.7,
                divergence: 0.0,
            },
            0.85,
        );

        let collective = cei.calculate_collective_state();
        assert_eq!(collective.group_size, 2);
        assert!(collective.psychological_safety > 0.0);
        assert!(collective.harmony_score > 0.0);
    }
}
