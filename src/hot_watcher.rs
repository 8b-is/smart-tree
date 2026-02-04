//
// -----------------------------------------------------------------------------
//  HOT WATCHER: Wave-Powered Directory Intelligence
//
//  This module uses MEM8 wave memory to track directory activity in real-time.
//  Each watched path becomes a Wave with properties that evolve:
//
//  - Frequency: Change rate (changes per hour)
//  - Arousal: Current activity level (0.0 = cold, 1.0 = HOT)
//  - Valence: Security concern (-1.0 = danger, +1.0 = safe)
//  - Decay: Old activity fades, recent activity persists
//
//  When waves resonate, we detect patterns - like coordinated attacks or
//  related changes across the codebase.
//
//  "Memory is wave interference patterns in cognitive space." - MEM8
// -----------------------------------------------------------------------------
//

use crate::scanner_interest::InterestLevel;
use crate::security_scan::SecurityFinding;
use anyhow::Result;
use crate::mem8_lite::Wave;
use notify::{
    event::{CreateKind, ModifyKind, RemoveKind},
    Event, EventKind, RecommendedWatcher, RecursiveMode, Watcher,
};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::{Arc, RwLock};
use std::time::{Duration, Instant};
use tokio::sync::mpsc;

/// A directory being watched with its wave state
#[derive(Debug, Clone)]
pub struct WatchedDirectory {
    /// The path being watched
    pub path: PathBuf,
    /// Wave representing this directory's activity pattern
    pub wave: Wave,
    /// Recent events (for burst detection)
    pub recent_events: Vec<WatchEvent>,
    /// Security findings in this directory
    pub security_findings: Vec<SecurityFinding>,
    /// Computed interest level
    pub interest_level: InterestLevel,
    /// Last time we computed the interest level
    pub interest_computed_at: Instant,
}

impl WatchedDirectory {
    /// Create a new watched directory with initial wave state
    pub fn new(path: PathBuf) -> Self {
        // Start with a neutral wave at low arousal
        let wave = Wave::new(
            1.0,  // 1 Hz base frequency (1 change per second baseline)
            0.0,  // Neutral valence (no security concern yet)
            0.1,  // Low initial arousal
        );

        Self {
            path,
            wave,
            recent_events: Vec::new(),
            security_findings: Vec::new(),
            interest_level: InterestLevel::Background,
            interest_computed_at: Instant::now(),
        }
    }

    /// Record a file system event, updating the wave
    pub fn record_event(&mut self, event: WatchEvent) {
        // Update wave properties based on event type
        match event.kind {
            WatchEventKind::Created => {
                // New files increase arousal
                self.wave.arousal = (self.wave.arousal + 0.2).min(1.0);
                self.wave.frequency += 0.5;
            }
            WatchEventKind::Modified => {
                // Modifications increase arousal moderately
                self.wave.arousal = (self.wave.arousal + 0.1).min(1.0);
                self.wave.frequency += 0.2;
            }
            WatchEventKind::Deleted => {
                // Deletions are notable
                self.wave.arousal = (self.wave.arousal + 0.15).min(1.0);
                self.wave.frequency += 0.3;
            }
            WatchEventKind::SecurityConcern => {
                // Security concerns lower valence (toward danger)
                self.wave.emotional_valence = (self.wave.emotional_valence - 0.3).max(-1.0);
                self.wave.arousal = 1.0; // Immediate high alert
            }
        }

        // Track recent events for burst detection
        self.recent_events.push(event);

        // Prune old events (keep last 5 minutes)
        let cutoff = Instant::now() - Duration::from_secs(300);
        self.recent_events.retain(|e| e.timestamp > cutoff);

        // Check for burst activity
        if self.recent_events.len() > 10 {
            // Lots of events = very hot
            self.wave.arousal = 1.0;
            self.wave.frequency = self.recent_events.len() as f64 / 300.0 * 3600.0; // events per hour
        }

        // Update interest level
        self.recompute_interest();
    }

    /// Record a security finding
    pub fn record_security_finding(&mut self, finding: SecurityFinding) {
        self.security_findings.push(finding);

        // Lower valence based on risk
        let valence_penalty = match self.security_findings.last().map(|f| &f.risk_level) {
            Some(crate::security_scan::RiskLevel::Critical) => 0.5,
            Some(crate::security_scan::RiskLevel::High) => 0.3,
            Some(crate::security_scan::RiskLevel::Medium) => 0.2,
            Some(crate::security_scan::RiskLevel::Low) => 0.1,
            None => 0.0,
        };
        self.wave.emotional_valence = (self.wave.emotional_valence - valence_penalty).max(-1.0);

        // Security findings always trigger high arousal
        self.wave.arousal = (self.wave.arousal + 0.5).min(1.0);

        self.recompute_interest();
    }

    /// Apply decay to the wave (call periodically)
    pub fn apply_decay(&mut self, elapsed_secs: f64) {
        // Arousal decays toward baseline
        let decay_factor = (-0.001 * elapsed_secs).exp();
        self.wave.arousal *= decay_factor;

        // Frequency decays toward 1.0 (baseline)
        self.wave.frequency = 1.0 + (self.wave.frequency - 1.0) * decay_factor;

        // Valence slowly recovers toward neutral (if no new threats)
        if self.wave.emotional_valence < 0.0 {
            self.wave.emotional_valence = (self.wave.emotional_valence + 0.0001 * elapsed_secs).min(0.0);
        }
    }

    /// Compute interest level from wave properties
    fn recompute_interest(&mut self) {
        self.interest_level = if self.wave.emotional_valence < -0.5 {
            // Security concern
            InterestLevel::Critical
        } else if self.wave.arousal > 0.8 {
            // Very hot
            InterestLevel::Important
        } else if self.wave.arousal > 0.4 || self.wave.frequency > 10.0 {
            // Notable activity
            InterestLevel::Notable
        } else if self.wave.arousal > 0.1 {
            // Some activity
            InterestLevel::Background
        } else {
            // Cold
            InterestLevel::Boring
        };

        self.interest_computed_at = Instant::now();
    }

    /// Check if this directory is "hot" (worth watching closely)
    pub fn is_hot(&self) -> bool {
        self.wave.arousal > 0.5 || self.wave.emotional_valence < -0.3
    }

    /// Compute resonance with another watched directory
    pub fn resonance_with(&self, other: &WatchedDirectory) -> f64 {
        self.wave.resonance_with(&other.wave)
    }
}

/// Type of watch event
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WatchEventKind {
    Created,
    Modified,
    Deleted,
    SecurityConcern,
}

/// A file system event
#[derive(Debug, Clone)]
pub struct WatchEvent {
    pub path: PathBuf,
    pub kind: WatchEventKind,
    pub timestamp: Instant,
}

/// The Hot Watcher - real-time directory intelligence
pub struct HotWatcher {
    /// Watched directories indexed by path
    directories: Arc<RwLock<HashMap<PathBuf, WatchedDirectory>>>,
    /// The file system watcher
    watcher: Option<RecommendedWatcher>,
    /// Event receiver
    event_rx: Option<mpsc::Receiver<WatchEvent>>,
    /// Event sender (for internal use)
    event_tx: mpsc::Sender<WatchEvent>,
    /// Last decay application
    last_decay: Instant,
}

impl HotWatcher {
    /// Create a new hot watcher
    pub fn new() -> Self {
        let (event_tx, event_rx) = mpsc::channel(1000);

        Self {
            directories: Arc::new(RwLock::new(HashMap::new())),
            watcher: None,
            event_rx: Some(event_rx),
            event_tx,
            last_decay: Instant::now(),
        }
    }

    /// Start watching a directory
    pub fn watch(&mut self, path: &Path) -> Result<()> {
        // Add to our tracked directories
        {
            let mut dirs = self.directories.write().unwrap();
            if !dirs.contains_key(path) {
                dirs.insert(path.to_path_buf(), WatchedDirectory::new(path.to_path_buf()));
            }
        }

        // Set up file system watcher if not already done
        if self.watcher.is_none() {
            let tx = self.event_tx.clone();
            let dirs = Arc::clone(&self.directories);

            let watcher = notify::recommended_watcher(move |res: notify::Result<Event>| {
                if let Ok(event) = res {
                    let kind = match event.kind {
                        EventKind::Create(CreateKind::File | CreateKind::Folder) => {
                            Some(WatchEventKind::Created)
                        }
                        EventKind::Modify(ModifyKind::Data(_) | ModifyKind::Name(_)) => {
                            Some(WatchEventKind::Modified)
                        }
                        EventKind::Remove(RemoveKind::File | RemoveKind::Folder) => {
                            Some(WatchEventKind::Deleted)
                        }
                        _ => None,
                    };

                    if let Some(kind) = kind {
                        for path in event.paths {
                            // Find the watched directory this belongs to
                            let dirs_read = dirs.read().unwrap();
                            for watched_path in dirs_read.keys() {
                                if path.starts_with(watched_path) {
                                    let watch_event = WatchEvent {
                                        path: path.clone(),
                                        kind,
                                        timestamp: Instant::now(),
                                    };
                                    let _ = tx.blocking_send(watch_event);
                                    break;
                                }
                            }
                        }
                    }
                }
            })?;

            self.watcher = Some(watcher);
        }

        // Add path to the watcher
        if let Some(ref mut watcher) = self.watcher {
            watcher.watch(path, RecursiveMode::Recursive)?;
        }

        Ok(())
    }

    /// Stop watching a directory
    pub fn unwatch(&mut self, path: &Path) -> Result<()> {
        if let Some(ref mut watcher) = self.watcher {
            watcher.unwatch(path)?;
        }

        let mut dirs = self.directories.write().unwrap();
        dirs.remove(path);

        Ok(())
    }

    /// Process pending events (call periodically)
    pub async fn process_events(&mut self) {
        // Apply decay
        let elapsed = self.last_decay.elapsed().as_secs_f64();
        if elapsed > 1.0 {
            let mut dirs = self.directories.write().unwrap();
            for dir in dirs.values_mut() {
                dir.apply_decay(elapsed);
            }
            self.last_decay = Instant::now();
        }

        // Process new events
        if let Some(ref mut rx) = self.event_rx {
            while let Ok(event) = rx.try_recv() {
                let mut dirs = self.directories.write().unwrap();

                // Find the parent watched directory
                for (watched_path, dir) in dirs.iter_mut() {
                    if event.path.starts_with(watched_path) {
                        dir.record_event(event.clone());
                        break;
                    }
                }
            }
        }
    }

    /// Get all hot directories (sorted by arousal)
    pub fn get_hot_directories(&self) -> Vec<WatchedDirectory> {
        let dirs = self.directories.read().unwrap();
        let mut hot: Vec<_> = dirs.values().filter(|d| d.is_hot()).cloned().collect();
        hot.sort_by(|a, b| b.wave.arousal.partial_cmp(&a.wave.arousal).unwrap());
        hot
    }

    /// Get directories by interest level
    pub fn get_by_interest(&self, level: InterestLevel) -> Vec<WatchedDirectory> {
        let dirs = self.directories.read().unwrap();
        dirs.values()
            .filter(|d| d.interest_level == level)
            .cloned()
            .collect()
    }

    /// Find directories that resonate with a given pattern
    pub fn find_resonating(&self, wave: &Wave, min_resonance: f64) -> Vec<(WatchedDirectory, f64)> {
        let dirs = self.directories.read().unwrap();
        let mut resonating: Vec<_> = dirs
            .values()
            .map(|d| {
                let resonance = d.wave.resonance_with(wave);
                (d.clone(), resonance)
            })
            .filter(|(_, r)| *r >= min_resonance)
            .collect();

        resonating.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
        resonating
    }

    /// Get a summary of watched directories
    pub fn summary(&self) -> HotWatcherSummary {
        let dirs = self.directories.read().unwrap();

        let mut critical = 0;
        let mut hot = 0;
        let mut warm = 0;
        let mut cold = 0;
        let mut total_arousal = 0.0;

        for dir in dirs.values() {
            total_arousal += dir.wave.arousal;
            match dir.interest_level {
                InterestLevel::Critical => critical += 1,
                InterestLevel::Important => hot += 1,
                InterestLevel::Notable => warm += 1,
                _ => cold += 1,
            }
        }

        let avg_arousal = if dirs.is_empty() {
            0.0
        } else {
            total_arousal / dirs.len() as f64
        };

        HotWatcherSummary {
            total_watched: dirs.len(),
            critical,
            hot,
            warm,
            cold,
            average_arousal: avg_arousal,
        }
    }
}

impl Default for HotWatcher {
    fn default() -> Self {
        Self::new()
    }
}

/// Summary of hot watcher state
#[derive(Debug, Clone)]
pub struct HotWatcherSummary {
    pub total_watched: usize,
    pub critical: usize,
    pub hot: usize,
    pub warm: usize,
    pub cold: usize,
    pub average_arousal: f64,
}

impl std::fmt::Display for HotWatcherSummary {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Watching {} dirs: {} critical, {} hot, {} warm, {} cold (avg arousal: {:.2})",
            self.total_watched,
            self.critical,
            self.hot,
            self.warm,
            self.cold,
            self.average_arousal
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_watched_directory_creation() {
        let dir = WatchedDirectory::new(PathBuf::from("/test"));
        assert_eq!(dir.wave.arousal, 0.1);
        assert_eq!(dir.interest_level, InterestLevel::Background);
    }

    #[test]
    fn test_event_increases_arousal() {
        let mut dir = WatchedDirectory::new(PathBuf::from("/test"));
        let initial_arousal = dir.wave.arousal;

        dir.record_event(WatchEvent {
            path: PathBuf::from("/test/file.rs"),
            kind: WatchEventKind::Created,
            timestamp: Instant::now(),
        });

        assert!(dir.wave.arousal > initial_arousal);
    }

    #[test]
    fn test_security_concern_lowers_valence() {
        let mut dir = WatchedDirectory::new(PathBuf::from("/test"));

        dir.record_event(WatchEvent {
            path: PathBuf::from("/test/evil.js"),
            kind: WatchEventKind::SecurityConcern,
            timestamp: Instant::now(),
        });

        assert!(dir.wave.emotional_valence < 0.0);
        assert_eq!(dir.wave.arousal, 1.0);
    }

    #[test]
    fn test_decay_reduces_arousal() {
        let mut dir = WatchedDirectory::new(PathBuf::from("/test"));
        dir.wave.arousal = 1.0;

        dir.apply_decay(1000.0); // 1000 seconds

        assert!(dir.wave.arousal < 1.0);
    }

    #[test]
    fn test_resonance() {
        let dir1 = WatchedDirectory::new(PathBuf::from("/test1"));
        let mut dir2 = WatchedDirectory::new(PathBuf::from("/test2"));

        // Make them similar
        dir2.wave.frequency = dir1.wave.frequency;
        dir2.wave.emotional_valence = dir1.wave.emotional_valence;
        dir2.wave.arousal = dir1.wave.arousal;

        let resonance = dir1.resonance_with(&dir2);
        assert!(resonance > 0.9); // Should be highly resonant
    }
}
