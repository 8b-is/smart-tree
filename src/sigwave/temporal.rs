use chrono::{DateTime, Utc, Timelike, Datelike};
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;

/// Temporal analysis of signature evolution
pub struct TemporalAnalyzer {
    /// Window size for moving averages
    window_size: usize,
    
    /// History buffer for recent signatures
    history: VecDeque<TimestampedSignature>,
    
    /// Detected patterns
    patterns: Vec<TemporalPattern>,
}

#[derive(Debug, Clone)]
struct TimestampedSignature {
    timestamp: DateTime<Utc>,
    vectors: super::SignatureVectors,
    context: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TemporalPattern {
    pub pattern_type: PatternType,
    pub period: Option<i64>, // Duration in seconds
    pub strength: f32,
    pub first_seen: DateTime<Utc>,
    pub last_seen: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum PatternType {
    /// Daily rhythm (morning vs evening behavior)
    Circadian,
    
    /// Weekly patterns (weekday vs weekend)
    Weekly,
    
    /// Project-based shifts
    Contextual,
    
    /// Gradual drift over time
    LongTermDrift,
    
    /// Sudden spikes or drops
    Anomaly,
}

impl TemporalAnalyzer {
    pub fn new(window_size: usize) -> Self {
        Self {
            window_size,
            history: VecDeque::with_capacity(window_size * 2),
            patterns: Vec::new(),
        }
    }
    
    /// Add a new signature to the temporal analysis
    pub fn add_signature(
        &mut self, 
        vectors: super::SignatureVectors, 
        context: String
    ) {
        let timestamped = TimestampedSignature {
            timestamp: Utc::now(),
            vectors,
            context,
        };
        
        self.history.push_back(timestamped);
        
        // Keep history bounded
        while self.history.len() > self.window_size * 2 {
            self.history.pop_front();
        }
        
        // Analyze for patterns
        self.detect_patterns();
    }
    
    /// Detect temporal patterns in the signature evolution
    fn detect_patterns(&mut self) {
        if self.history.len() < 10 {
            return; // Need enough data
        }
        
        // Check for circadian patterns
        if let Some(pattern) = self.detect_circadian() {
            self.update_or_add_pattern(pattern);
        }
        
        // Check for weekly patterns
        if let Some(pattern) = self.detect_weekly() {
            self.update_or_add_pattern(pattern);
        }
        
        // Check for contextual shifts
        if let Some(pattern) = self.detect_contextual() {
            self.update_or_add_pattern(pattern);
        }
        
        // Check for long-term drift
        if let Some(pattern) = self.detect_drift() {
            self.update_or_add_pattern(pattern);
        }
    }
    
    fn detect_circadian(&self) -> Option<TemporalPattern> {
        // Group signatures by hour of day
        let mut hourly_groups: Vec<Vec<&TimestampedSignature>> = vec![vec![]; 24];
        
        for sig in &self.history {
            let hour = sig.timestamp.hour() as usize;
            hourly_groups[hour].push(sig);
        }
        
        // Find hours with significant behavioral differences
        let mut max_variance = 0.0;
        let mut _peak_hours = (0, 12); // Default morning vs evening
        
        for morning in 6..12 {
            for evening in 18..23 {
                if hourly_groups[morning].is_empty() || hourly_groups[evening].is_empty() {
                    continue;
                }
                
                let variance = self.calculate_group_variance(
                    &hourly_groups[morning],
                    &hourly_groups[evening]
                );
                
                if variance > max_variance {
                    max_variance = variance;
                    _peak_hours = (morning, evening);
                }
            }
        }
        
        if max_variance > 0.2 { // Threshold for significant pattern
            Some(TemporalPattern {
                pattern_type: PatternType::Circadian,
                period: Some(24 * 3600), // 24 hours in seconds
                strength: max_variance,
                first_seen: self.history.front()?.timestamp,
                last_seen: self.history.back()?.timestamp,
            })
        } else {
            None
        }
    }
    
    fn detect_weekly(&self) -> Option<TemporalPattern> {
        use chrono::Datelike;
        
        // Group by weekday vs weekend
        let mut weekday_sigs = Vec::new();
        let mut weekend_sigs = Vec::new();
        
        for sig in &self.history {
            let weekday = sig.timestamp.weekday();
            match weekday {
                chrono::Weekday::Sat | chrono::Weekday::Sun => weekend_sigs.push(sig),
                _ => weekday_sigs.push(sig),
            }
        }
        
        if weekday_sigs.len() < 5 || weekend_sigs.len() < 2 {
            return None; // Not enough data
        }
        
        let variance = self.calculate_group_variance(&weekday_sigs, &weekend_sigs);
        
        if variance > 0.15 {
            Some(TemporalPattern {
                pattern_type: PatternType::Weekly,
                period: Some(7 * 24 * 3600), // 7 days in seconds
                strength: variance,
                first_seen: self.history.front()?.timestamp,
                last_seen: self.history.back()?.timestamp,
            })
        } else {
            None
        }
    }
    
    fn detect_contextual(&self) -> Option<TemporalPattern> {
        // Group by context
        let mut context_groups: std::collections::HashMap<String, Vec<&TimestampedSignature>> = 
            std::collections::HashMap::new();
        
        for sig in &self.history {
            context_groups.entry(sig.context.clone())
                .or_insert_with(Vec::new)
                .push(sig);
        }
        
        // Find contexts with most variance
        let mut max_variance: f32 = 0.0;
        
        for (ctx1, sigs1) in &context_groups {
            for (ctx2, sigs2) in &context_groups {
                if ctx1 == ctx2 || sigs1.len() < 3 || sigs2.len() < 3 {
                    continue;
                }
                
                let variance = self.calculate_group_variance(sigs1, sigs2);
                max_variance = max_variance.max(variance);
            }
        }
        
        if max_variance > 0.25 {
            Some(TemporalPattern {
                pattern_type: PatternType::Contextual,
                period: None,
                strength: max_variance,
                first_seen: self.history.front()?.timestamp,
                last_seen: self.history.back()?.timestamp,
            })
        } else {
            None
        }
    }
    
    fn detect_drift(&self) -> Option<TemporalPattern> {
        if self.history.len() < 20 {
            return None;
        }
        
        // Compare early signatures to recent ones
        let early_count = self.history.len() / 4;
        let recent_count = self.history.len() / 4;
        
        let early_sigs: Vec<_> = self.history.iter().take(early_count).collect();
        let recent_sigs: Vec<_> = self.history.iter().rev().take(recent_count).collect();
        
        let drift = self.calculate_group_variance(&early_sigs, &recent_sigs);
        
        if drift > 0.1 {
            Some(TemporalPattern {
                pattern_type: PatternType::LongTermDrift,
                period: None,
                strength: drift,
                first_seen: self.history.front()?.timestamp,
                last_seen: self.history.back()?.timestamp,
            })
        } else {
            None
        }
    }
    
    fn calculate_group_variance(
        &self, 
        group1: &[&TimestampedSignature], 
        group2: &[&TimestampedSignature]
    ) -> f32 {
        if group1.is_empty() || group2.is_empty() {
            return 0.0;
        }
        
        // Calculate average vectors for each group
        let avg1 = self.average_signatures(group1);
        let avg2 = self.average_signatures(group2);
        
        // Calculate distance between averages
        self.signature_distance(&avg1, &avg2)
    }
    
    fn average_signatures(&self, sigs: &[&TimestampedSignature]) -> super::SignatureVectors {
        let count = sigs.len() as f32;
        
        // Sum all components
        let mut avg_style = super::StyleVector {
            terseness: 0.0,
            humor_density: 0.0,
            technicality: 0.0,
            formality: 0.0,
            bullet_preference: 0.0,
        };
        
        for sig in sigs {
            avg_style.terseness += sig.vectors.style.terseness;
            avg_style.humor_density += sig.vectors.style.humor_density;
            avg_style.technicality += sig.vectors.style.technicality;
            avg_style.formality += sig.vectors.style.formality;
            avg_style.bullet_preference += sig.vectors.style.bullet_preference;
        }
        
        // Divide by count
        avg_style.terseness /= count;
        avg_style.humor_density /= count;
        avg_style.technicality /= count;
        avg_style.formality /= count;
        avg_style.bullet_preference /= count;
        
        // For brevity, just using style vector here
        // In real implementation, would average all vectors
        super::SignatureVectors {
            style: avg_style,
            behavior: sigs[0].vectors.behavior.clone(),
            concepts: sigs[0].vectors.concepts.clone(),
            linguistic: sigs[0].vectors.linguistic.clone(),
            emotional: sigs[0].vectors.emotional.clone(),
        }
    }
    
    fn signature_distance(&self, sig1: &super::SignatureVectors, sig2: &super::SignatureVectors) -> f32 {
        use super::VectorDistance;
        
        let style_dist = sig1.style.distance(&sig2.style);
        let behavior_dist = sig1.behavior.distance(&sig2.behavior);
        let emotional_dist = sig1.emotional.distance(&sig2.emotional);
        
        (style_dist + behavior_dist + emotional_dist) / 3.0
    }
    
    fn update_or_add_pattern(&mut self, pattern: TemporalPattern) {
        // Update existing pattern or add new one
        if let Some(existing) = self.patterns.iter_mut()
            .find(|p| std::mem::discriminant(&p.pattern_type) == 
                      std::mem::discriminant(&pattern.pattern_type)) {
            existing.strength = pattern.strength;
            existing.last_seen = pattern.last_seen;
        } else {
            self.patterns.push(pattern);
        }
    }
    
    /// Get current temporal state
    pub fn get_temporal_state(&self) -> TemporalState {
        TemporalState {
            current_phase: self.determine_current_phase(),
            active_patterns: self.patterns.clone(),
            stability_score: self.calculate_stability(),
            predicted_next: self.predict_next_state(),
        }
    }
    
    fn determine_current_phase(&self) -> String {
        // Based on time of day and recent patterns
        let now = Utc::now();
        let hour = now.hour();
        
        match hour {
            6..=11 => "morning_flow".to_string(),
            12..=17 => "afternoon_focus".to_string(),
            18..=23 => "evening_wind_down".to_string(),
            _ => "night_owl".to_string(),
        }
    }
    
    fn calculate_stability(&self) -> f32 {
        if self.history.len() < 5 {
            return 1.0; // Not enough data
        }
        
        // Calculate variance in recent signatures
        let recent: Vec<_> = self.history.iter().rev().take(5).collect();
        let mut total_variance = 0.0;
        
        for i in 1..recent.len() {
            let dist = self.signature_distance(&recent[i-1].vectors, &recent[i].vectors);
            total_variance += dist;
        }
        
        // Convert to stability score (inverse of variance)
        1.0 - (total_variance / 4.0).min(1.0)
    }
    
    fn predict_next_state(&self) -> Option<String> {
        // Simple prediction based on patterns
        if let Some(circadian) = self.patterns.iter()
            .find(|p| matches!(p.pattern_type, PatternType::Circadian)) {
            if circadian.strength > 0.3 {
                return Some("Circadian shift expected".to_string());
            }
        }
        
        None
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TemporalState {
    pub current_phase: String,
    pub active_patterns: Vec<TemporalPattern>,
    pub stability_score: f32,
    pub predicted_next: Option<String>,
}

/// Calculate temporal velocity (rate of change)
pub fn calculate_velocity(
    history: &[super::SignatureBlock], 
    window: usize
) -> f32 {
    if history.len() < window + 1 {
        return 0.0;
    }
    
    let recent = &history[history.len() - window..];
    let mut total_change = 0.0;
    
    for i in 1..recent.len() {
        total_change += recent[i].divergence.delta_magnitude;
    }
    
    total_change / (window - 1) as f32
}