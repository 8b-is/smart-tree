//! Safety mechanisms for scanning large directories
//!
//! This module provides safety limits and optimizations to prevent
//! crashes when scanning very large directories like home directories.

use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::{Duration, Instant};

/// Safety limits for directory scanning
#[derive(Debug, Clone)]
pub struct ScannerSafetyLimits {
    /// Maximum admitted nodes, including directories and error placeholders (0 = unlimited).
    pub max_files: usize,
    /// Maximum time to spend scanning
    pub max_duration: Duration,
    /// Maximum cumulative estimated node bytes (not a resident-memory bound).
    pub max_memory_bytes: usize,
    /// Warn when exceeding this many admitted nodes.
    pub warn_threshold: usize,
}

impl Default for ScannerSafetyLimits {
    fn default() -> Self {
        Self {
            max_files: 1_000_000,                     // 1 million nodes max by default
            max_duration: Duration::from_secs(300),   // 5 minutes max
            max_memory_bytes: 2 * 1024 * 1024 * 1024, // 2GB max
            warn_threshold: 100_000,                  // Warn at 100k files
        }
    }
}

impl ScannerSafetyLimits {
    /// Create unlimited safety limits (use with caution!)
    pub fn unlimited() -> Self {
        Self {
            max_files: 0,
            max_duration: Duration::from_secs(u64::MAX),
            max_memory_bytes: usize::MAX,
            warn_threshold: usize::MAX,
        }
    }

    /// Create limits suitable for home directory scanning
    pub fn for_home_directory() -> Self {
        Self {
            max_files: 500_000,                     // 500k nodes max for home dirs
            max_duration: Duration::from_secs(120), // 2 minutes max
            max_memory_bytes: 1024 * 1024 * 1024,   // 1GB max
            warn_threshold: 50_000,                 // Warn at 50k files
        }
    }

    /// Create limits for MCP operations (more conservative)
    pub fn for_mcp() -> Self {
        Self {
            max_files: 100_000,                    // 100k nodes max for MCP
            max_duration: Duration::from_secs(60), // 1 minute max
            max_memory_bytes: 512 * 1024 * 1024,   // 512MB max
            warn_threshold: 10_000,                // Warn at 10k files
        }
    }
}

/// Tracks safety metrics during scanning
pub struct ScannerSafetyTracker {
    start_time: Instant,
    file_count: AtomicUsize,
    estimated_memory: AtomicUsize,
    limits: ScannerSafetyLimits,
    warned: AtomicUsize,
}

impl ScannerSafetyTracker {
    pub fn new(limits: ScannerSafetyLimits) -> Self {
        Self {
            start_time: Instant::now(),
            file_count: AtomicUsize::new(0),
            estimated_memory: AtomicUsize::new(0),
            limits,
            warned: AtomicUsize::new(0),
        }
    }

    /// Check elapsed time while walking, including entries that are not admitted.
    /// Count and byte limits are checked prospectively by `add_file`, so an exact
    /// cap followed only by hidden/ignored entries is not falsely truncated.
    pub fn should_continue(&self) -> Result<(), String> {
        // Check duration
        if self.start_time.elapsed() > self.limits.max_duration {
            return Err(format!(
                "Scan truncated: Exceeded maximum duration of {:?}",
                self.limits.max_duration
            ));
        }

        // Warn if approaching limits
        let count = self.file_count.load(Ordering::Relaxed);
        if count > self.limits.warn_threshold && self.warned.load(Ordering::Relaxed) == 0 {
            self.warned.store(1, Ordering::Relaxed);
            eprintln!(
                "⚠️  Warning: Scanning large directory ({} nodes so far)",
                count
            );
            eprintln!("   Consider using --max-depth or --stream mode");
        }

        Ok(())
    }

    /// Admit one node before retaining it. Exclusive access makes the two
    /// counter updates indivisible to callers; rejection leaves both unchanged.
    pub fn add_file(&mut self, estimated_node_size: usize) -> Result<(), String> {
        self.should_continue()?;
        let count = self
            .file_count
            .load(Ordering::Relaxed)
            .checked_add(1)
            .ok_or("Scan truncated: Node count overflow")?;
        let memory = self
            .estimated_memory
            .load(Ordering::Relaxed)
            .checked_add(estimated_node_size)
            .ok_or("Scan truncated: Estimated node bytes overflow")?;

        if self.limits.max_files > 0 && count > self.limits.max_files {
            return Err(format!(
                "Scan truncated: Reached maximum node limit of {} nodes",
                self.limits.max_files
            ));
        }
        if memory > self.limits.max_memory_bytes {
            return Err(format!(
                "Scan truncated: Estimated node bytes ({}) exceed limit ({})",
                memory, self.limits.max_memory_bytes
            ));
        }

        self.file_count.store(count, Ordering::Relaxed);
        self.estimated_memory.store(memory, Ordering::Relaxed);
        Ok(())
    }

    /// Get current stats
    pub fn stats(&self) -> (usize, Duration, usize) {
        (
            self.file_count.load(Ordering::Relaxed),
            self.start_time.elapsed(),
            self.estimated_memory.load(Ordering::Relaxed),
        )
    }
}

/// Estimate memory size of a FileNode (rough approximation)
pub fn estimate_node_size(path_len: usize) -> Result<usize, String> {
    // Base struct size + path string + some overhead
    std::mem::size_of::<crate::scanner::FileNode>()
        .checked_add(path_len)
        .and_then(|size| size.checked_add(64))
        .ok_or_else(|| "Scan truncated: Node size estimate overflow".to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_safety_limits() {
        let limits = ScannerSafetyLimits::for_home_directory();
        assert_eq!(limits.max_files, 500_000);

        let mcp_limits = ScannerSafetyLimits::for_mcp();
        assert!(mcp_limits.max_files < limits.max_files);
    }

    #[test]
    fn test_safety_tracker() {
        let limits = ScannerSafetyLimits {
            max_files: 10,
            max_duration: Duration::from_secs(1),
            max_memory_bytes: 1024,
            warn_threshold: 5,
        };

        let mut tracker = ScannerSafetyTracker::new(limits);

        // Should start OK
        assert!(tracker.should_continue().is_ok());

        // Add files until we hit the limit
        for _ in 0..10 {
            tracker.add_file(100).unwrap();
        }

        // Exactly at the cap is not truncation; only another admission fails.
        assert!(tracker.should_continue().is_ok());
        assert!(tracker.add_file(1).is_err());
        assert_eq!(tracker.stats().0, 10);
        assert_eq!(tracker.stats().2, 1000);
    }

    #[test]
    fn prospective_byte_limit_rejects_without_changing_counters() {
        let mut tracker = ScannerSafetyTracker::new(ScannerSafetyLimits {
            max_memory_bytes: 100,
            ..ScannerSafetyLimits::unlimited()
        });
        tracker.add_file(100).unwrap();
        assert!(tracker.should_continue().is_ok());
        assert!(tracker.add_file(1).unwrap_err().contains("exceed limit"));
        assert_eq!((tracker.stats().0, tracker.stats().2), (1, 100));
    }

    #[test]
    fn zero_file_limit_is_unlimited() {
        let mut tracker = ScannerSafetyTracker::new(ScannerSafetyLimits::unlimited());
        for _ in 0..100 {
            tracker.add_file(1).unwrap();
        }
        assert_eq!((tracker.stats().0, tracker.stats().2), (100, 100));
    }

    #[test]
    fn counter_overflow_does_not_partially_admit_a_node() {
        let mut tracker = ScannerSafetyTracker::new(ScannerSafetyLimits::unlimited());
        tracker.add_file(usize::MAX).unwrap();
        assert!(tracker.add_file(1).unwrap_err().contains("bytes overflow"));
        assert_eq!((tracker.stats().0, tracker.stats().2), (1, usize::MAX));

        tracker.file_count.store(usize::MAX, Ordering::Relaxed);
        tracker.estimated_memory.store(0, Ordering::Relaxed);
        assert!(tracker.add_file(1).unwrap_err().contains("count overflow"));
        assert_eq!((tracker.stats().0, tracker.stats().2), (usize::MAX, 0));
    }

    #[test]
    fn node_estimate_checks_both_additions() {
        let base = std::mem::size_of::<crate::scanner::FileNode>();
        assert_eq!(estimate_node_size(10).unwrap(), base + 10 + 64);
        assert!(estimate_node_size(usize::MAX).is_err());
        assert!(estimate_node_size(usize::MAX - base).is_err());
        assert_eq!(
            estimate_node_size(usize::MAX - base - 64).unwrap(),
            usize::MAX
        );
    }

    #[test]
    fn expired_duration_does_not_admit_a_node() {
        let mut tracker = ScannerSafetyTracker::new(ScannerSafetyLimits {
            max_duration: Duration::from_secs(1),
            ..ScannerSafetyLimits::unlimited()
        });
        tracker.start_time = Instant::now().checked_sub(Duration::from_secs(2)).unwrap();
        assert!(tracker.add_file(1).unwrap_err().contains("duration"));
        assert_eq!((tracker.stats().0, tracker.stats().2), (0, 0));
    }
}
