//! Temporal aspects of context gathering
//!
//! This module adds time-based features to context gathering, allowing for
//! understanding how project context evolves over time.

use chrono::{DateTime, Datelike, Duration, TimeZone, Timelike, Utc};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, HashMap};

use super::{ContextType, GatheredContext};

/// Temporal resolution for context analysis
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum TemporalResolution {
    Hour,
    Day,
    Week,
    Month,
    Quarter,
    Year,
}

impl TemporalResolution {
    /// Get the duration for this resolution
    pub fn duration(&self) -> Duration {
        match self {
            Self::Hour => Duration::hours(1),
            Self::Day => Duration::days(1),
            Self::Week => Duration::weeks(1),
            Self::Month => Duration::days(30), // Approximate
            Self::Quarter => Duration::days(90),
            Self::Year => Duration::days(365),
        }
    }

    /// Truncate a datetime to this resolution
    pub fn truncate(&self, dt: DateTime<Utc>) -> DateTime<Utc> {
        match self {
            Self::Hour => Utc
                .with_ymd_and_hms(dt.year(), dt.month(), dt.day(), dt.hour(), 0, 0)
                .single()
                .unwrap_or(dt),
            Self::Day => Utc
                .with_ymd_and_hms(dt.year(), dt.month(), dt.day(), 0, 0, 0)
                .single()
                .unwrap_or(dt),
            Self::Week => {
                let days_since_monday = dt.weekday().num_days_from_monday();
                let start_of_week = dt - Duration::days(days_since_monday as i64);
                Utc.with_ymd_and_hms(
                    start_of_week.year(),
                    start_of_week.month(),
                    start_of_week.day(),
                    0,
                    0,
                    0,
                )
                .single()
                .unwrap_or(dt)
            }
            Self::Month => Utc
                .with_ymd_and_hms(dt.year(), dt.month(), 1, 0, 0, 0)
                .single()
                .unwrap_or(dt),
            Self::Quarter => {
                let quarter_month = ((dt.month() - 1) / 3) * 3 + 1;
                Utc.with_ymd_and_hms(dt.year(), quarter_month, 1, 0, 0, 0)
                    .single()
                    .unwrap_or(dt)
            }
            Self::Year => Utc
                .with_ymd_and_hms(dt.year(), 1, 1, 0, 0, 0)
                .single()
                .unwrap_or(dt),
        }
    }
}

/// Temporal context analyzer
pub struct TemporalContextAnalyzer {
    pub contexts: Vec<GatheredContext>,
    resolution: TemporalResolution,
}

impl TemporalContextAnalyzer {
    pub fn new(contexts: Vec<GatheredContext>, resolution: TemporalResolution) -> Self {
        Self {
            contexts,
            resolution,
        }
    }

    /// Group contexts by time period
    pub fn group_by_time(&self) -> BTreeMap<DateTime<Utc>, Vec<&GatheredContext>> {
        let mut groups = BTreeMap::new();

        for context in &self.contexts {
            let period = self.resolution.truncate(context.timestamp);
            groups.entry(period).or_insert_with(Vec::new).push(context);
        }

        groups
    }

    /// Calculate activity intensity over time
    pub fn activity_timeline(&self) -> Vec<TimelinePoint> {
        let groups = self.group_by_time();
        let mut timeline = Vec::new();

        for (time, contexts) in groups {
            let intensity = contexts.len() as f32;
            let avg_relevance =
                contexts.iter().map(|c| c.relevance_score).sum::<f32>() / contexts.len() as f32;

            timeline.push(TimelinePoint {
                timestamp: time,
                activity_count: contexts.len(),
                intensity,
                average_relevance: avg_relevance,
                dominant_type: Self::get_dominant_type(&contexts),
                tools_used: Self::get_tools_used(&contexts),
            });
        }

        timeline
    }

    /// Detect temporal patterns
    pub fn detect_patterns(&self) -> TemporalPatterns {
        let timeline = self.activity_timeline();

        // Detect work sessions (clusters of activity)
        let sessions = self.detect_work_sessions(&timeline);

        // Find peak activity times
        let peak_times = self.find_peak_times(&timeline);

        // Calculate momentum (increasing/decreasing activity)
        let momentum = self.calculate_momentum(&timeline);

        // Detect periodic patterns
        let periodic_patterns = self.detect_periodic_patterns(&timeline);

        TemporalPatterns {
            work_sessions: sessions,
            peak_times,
            momentum,
            periodic_patterns,
            total_duration: self.calculate_total_duration(),
            active_days: self.count_active_days(),
        }
    }

    /// Create a temporal wave representation
    pub fn create_temporal_waves(&self) -> TemporalWaveGrid {
        let mut grid = TemporalWaveGrid::new(self.resolution);
        let groups = self.group_by_time();

        for (time, contexts) in groups {
            // Create waves for each time period
            for context in contexts {
                let wave = TemporalWave {
                    timestamp: time,
                    frequency: self.calculate_frequency(context),
                    amplitude: context.relevance_score,
                    phase: self.calculate_phase(context),
                    decay_rate: self.calculate_decay_rate(&time),
                    context_type: context.content_type.clone(),
                    tool: context.ai_tool.clone(),
                };

                grid.add_wave(wave);
            }
        }

        grid
    }

    /// Apply temporal decay to relevance scores
    pub fn apply_temporal_decay(&mut self, half_life_days: f32) {
        let now = Utc::now();

        for context in &mut self.contexts {
            let age_days = (now - context.timestamp).num_days() as f32;
            let decay_factor = 0.5_f32.powf(age_days / half_life_days);
            context.relevance_score *= decay_factor;
        }

        // Re-sort by decayed relevance
        self.contexts
            .sort_by(|a, b| b.relevance_score.partial_cmp(&a.relevance_score).unwrap());
    }

    // Helper methods

    fn get_dominant_type(contexts: &[&GatheredContext]) -> ContextType {
        let mut type_counts = HashMap::new();
        for context in contexts {
            *type_counts.entry(context.content_type.clone()).or_insert(0) += 1;
        }

        type_counts
            .into_iter()
            .max_by_key(|(_, count)| *count)
            .map(|(ctx_type, _)| ctx_type)
            .unwrap_or(ContextType::Configuration)
    }

    fn get_tools_used(contexts: &[&GatheredContext]) -> Vec<String> {
        let mut tools = contexts
            .iter()
            .map(|c| c.ai_tool.clone())
            .collect::<Vec<_>>();
        tools.sort();
        tools.dedup();
        tools
    }

    fn detect_work_sessions(&self, timeline: &[TimelinePoint]) -> Vec<WorkSession> {
        let mut sessions = Vec::new();
        let mut current_session: Option<WorkSession> = None;

        let session_gap = match self.resolution {
            TemporalResolution::Hour => Duration::hours(4),
            TemporalResolution::Day => Duration::days(3),
            _ => Duration::weeks(1),
        };

        for point in timeline.iter() {
            if let Some(ref mut session) = current_session {
                let gap = point.timestamp - session.end_time;

                if gap > session_gap {
                    // End current session and start new one
                    sessions.push(session.clone());
                    current_session = Some(WorkSession {
                        start_time: point.timestamp,
                        end_time: point.timestamp,
                        total_activities: point.activity_count,
                        average_intensity: point.intensity,
                    });
                } else {
                    // Continue current session
                    session.end_time = point.timestamp;
                    session.total_activities += point.activity_count;
                    session.average_intensity = (session.average_intensity + point.intensity) / 2.0;
                }
            } else {
                // Start first session
                current_session = Some(WorkSession {
                    start_time: point.timestamp,
                    end_time: point.timestamp,
                    total_activities: point.activity_count,
                    average_intensity: point.intensity,
                });
            }
        }

        if let Some(session) = current_session {
            sessions.push(session);
        }

        sessions
    }

    fn find_peak_times(&self, timeline: &[TimelinePoint]) -> Vec<PeakTime> {
        let mut peaks = timeline
            .iter()
            .map(|point| PeakTime {
                timestamp: point.timestamp,
                intensity: point.intensity,
                resolution: self.resolution,
            })
            .collect::<Vec<_>>();

        // Sort by intensity and take top 10
        peaks.sort_by(|a, b| b.intensity.partial_cmp(&a.intensity).unwrap());
        peaks.truncate(10);

        peaks
    }

    fn calculate_momentum(&self, timeline: &[TimelinePoint]) -> f32 {
        if timeline.len() < 2 {
            return 0.0;
        }

        // Calculate trend over recent periods
        let recent_count = timeline.len().min(10);
        let recent = &timeline[timeline.len() - recent_count..];

        let mut momentum = 0.0;
        for i in 1..recent.len() {
            let change = recent[i].intensity - recent[i - 1].intensity;
            momentum += change * (i as f32 / recent.len() as f32); // Weight recent changes more
        }

        momentum / recent.len() as f32
    }

    fn detect_periodic_patterns(&self, timeline: &[TimelinePoint]) -> Vec<PeriodicPattern> {
        let mut patterns = Vec::new();

        // Check for daily patterns (if resolution allows)
        if matches!(self.resolution, TemporalResolution::Hour) {
            if let Some(pattern) = self.detect_daily_pattern(timeline) {
                patterns.push(pattern);
            }
        }

        // Check for weekly patterns
        if matches!(
            self.resolution,
            TemporalResolution::Day | TemporalResolution::Hour
        ) {
            if let Some(pattern) = self.detect_weekly_pattern(timeline) {
                patterns.push(pattern);
            }
        }

        patterns
    }

    fn detect_daily_pattern(&self, timeline: &[TimelinePoint]) -> Option<PeriodicPattern> {
        // Group by hour of day
        let mut hour_activities = HashMap::new();

        for point in timeline {
            let hour = point.timestamp.hour();
            hour_activities
                .entry(hour)
                .or_insert_with(Vec::new)
                .push(point.intensity);
        }

        // Find peak hours
        let mut peak_hours = hour_activities
            .iter()
            .map(|(hour, intensities)| {
                let avg = intensities.iter().sum::<f32>() / intensities.len() as f32;
                (*hour, avg)
            })
            .collect::<Vec<_>>();

        peak_hours.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());

        if peak_hours.is_empty() {
            return None;
        }

        Some(PeriodicPattern {
            period_type: "daily".to_string(),
            peak_periods: peak_hours
                .iter()
                .take(3)
                .map(|(h, _)| format!("{:02}:00", h))
                .collect(),
            strength: 0.0, // TODO: Calculate pattern strength
        })
    }

    fn detect_weekly_pattern(&self, timeline: &[TimelinePoint]) -> Option<PeriodicPattern> {
        // Group by day of week
        let mut day_activities = HashMap::new();

        for point in timeline {
            let day = point.timestamp.weekday();
            day_activities
                .entry(day)
                .or_insert_with(Vec::new)
                .push(point.intensity);
        }

        // Find peak days
        let mut peak_days = day_activities
            .iter()
            .map(|(day, intensities)| {
                let avg = intensities.iter().sum::<f32>() / intensities.len() as f32;
                (*day, avg)
            })
            .collect::<Vec<_>>();

        peak_days.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());

        if peak_days.is_empty() {
            return None;
        }

        Some(PeriodicPattern {
            period_type: "weekly".to_string(),
            peak_periods: peak_days
                .iter()
                .take(3)
                .map(|(d, _)| format!("{:?}", d))
                .collect(),
            strength: 0.0, // TODO: Calculate pattern strength
        })
    }

    fn calculate_total_duration(&self) -> Duration {
        if self.contexts.is_empty() {
            return Duration::zero();
        }

        let min_time = self.contexts.iter().map(|c| c.timestamp).min().unwrap();
        let max_time = self.contexts.iter().map(|c| c.timestamp).max().unwrap();

        max_time - min_time
    }

    fn count_active_days(&self) -> usize {
        let mut days = self
            .contexts
            .iter()
            .map(|c| c.timestamp.date_naive())
            .collect::<Vec<_>>();
        days.sort();
        days.dedup();
        days.len()
    }

    fn calculate_frequency(&self, context: &GatheredContext) -> f32 {
        // Map context type to frequency band
        match context.content_type {
            ContextType::ChatHistory => 0.1,
            ContextType::ProjectSettings => 0.05,
            ContextType::CodeSnippets => 0.2,
            ContextType::Documentation => 0.08,
            ContextType::Configuration => 0.06,
            ContextType::SearchHistory => 0.15,
            ContextType::Bookmarks => 0.07,
            ContextType::CustomPrompts => 0.12,
            ContextType::ModelPreferences => 0.04,
            ContextType::WorkspaceState => 0.09,
        }
    }

    fn calculate_phase(&self, context: &GatheredContext) -> f32 {
        // Use timestamp to create phase offset
        let minutes_since_epoch = context.timestamp.timestamp() / 60;
        ((minutes_since_epoch % 360) as f32) * std::f32::consts::PI / 180.0
    }

    fn calculate_decay_rate(&self, time: &DateTime<Utc>) -> f32 {
        let age_days = (Utc::now() - *time).num_days() as f32;
        1.0 / (1.0 + age_days / 30.0) // Decay over 30 days
    }
}

/// Point on the activity timeline
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimelinePoint {
    pub timestamp: DateTime<Utc>,
    pub activity_count: usize,
    pub intensity: f32,
    pub average_relevance: f32,
    pub dominant_type: ContextType,
    pub tools_used: Vec<String>,
}

/// Detected temporal patterns
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemporalPatterns {
    pub work_sessions: Vec<WorkSession>,
    pub peak_times: Vec<PeakTime>,
    pub momentum: f32,
    pub periodic_patterns: Vec<PeriodicPattern>,
    pub total_duration: Duration,
    pub active_days: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkSession {
    pub start_time: DateTime<Utc>,
    pub end_time: DateTime<Utc>,
    pub total_activities: usize,
    pub average_intensity: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PeakTime {
    pub timestamp: DateTime<Utc>,
    pub intensity: f32,
    pub resolution: TemporalResolution,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PeriodicPattern {
    pub period_type: String,
    pub peak_periods: Vec<String>,
    pub strength: f32,
}

/// Temporal wave representation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemporalWave {
    pub timestamp: DateTime<Utc>,
    pub frequency: f32,
    pub amplitude: f32,
    pub phase: f32,
    pub decay_rate: f32,
    pub context_type: ContextType,
    pub tool: String,
}

/// Grid of temporal waves
pub struct TemporalWaveGrid {
    resolution: TemporalResolution,
    waves: Vec<TemporalWave>,
    time_slots: BTreeMap<DateTime<Utc>, Vec<usize>>, // Time -> wave indices
}

impl TemporalWaveGrid {
    pub fn new(resolution: TemporalResolution) -> Self {
        Self {
            resolution,
            waves: Vec::new(),
            time_slots: BTreeMap::new(),
        }
    }

    pub fn add_wave(&mut self, wave: TemporalWave) {
        let time_slot = self.resolution.truncate(wave.timestamp);
        let wave_idx = self.waves.len();
        self.waves.push(wave);

        self.time_slots
            .entry(time_slot)
            .or_default()
            .push(wave_idx);
    }

    /// Get interference pattern at a specific time
    pub fn get_interference_at(&self, time: DateTime<Utc>) -> f32 {
        let time_slot = self.resolution.truncate(time);

        if let Some(indices) = self.time_slots.get(&time_slot) {
            let mut total = 0.0;

            for &idx in indices {
                let wave = &self.waves[idx];
                let age_factor = wave.decay_rate;
                let value = wave.amplitude
                    * age_factor
                    * (2.0 * std::f32::consts::PI * wave.frequency * time.timestamp() as f32
                        + wave.phase)
                        .sin();
                total += value;
            }

            total / indices.len() as f32
        } else {
            0.0
        }
    }

    /// Navigate through time to find interesting moments
    pub fn find_resonance_peaks(&self) -> Vec<DateTime<Utc>> {
        let mut peaks = Vec::new();

        for (&time, indices) in &self.time_slots {
            if indices.len() > 3 {
                // Multiple waves at same time
                let interference = self.get_interference_at(time);
                if interference.abs() > 0.7 {
                    peaks.push(time);
                }
            }
        }

        peaks
    }
}
