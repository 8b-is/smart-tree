//! Safety mechanisms for scanning large directories
//! 
//! This module provides safety limits and optimizations to prevent
//! crashes when scanning very large directories like home directories.

use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::{Duration, Instant};

/// Safety limits for directory scanning
#[derive(Debug, Clone)]
pub struct ScannerSafetyLimits {
    /// Maximum number of files to scan (0 = unlimited)
    pub max_files: usize,
    /// Maximum time to spend scanning
    pub max_duration: Duration,
    /// Maximum memory usage in bytes (estimated)
    pub max_memory_bytes: usize,
    /// Warn when exceeding this many files
    pub warn_threshold: usize,
}

impl Default for ScannerSafetyLimits {
    fn default() -> Self {
        Self {
            max_files: 1_000_000,        // 1 million files max by default
            max_duration: Duration::from_secs(300), // 5 minutes max
            max_memory_bytes: 2 * 1024 * 1024 * 1024, // 2GB max
            warn_threshold: 100_000,      // Warn at 100k files
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
            max_files: 500_000,           // 500k files max for home dirs
            max_duration: Duration::from_secs(120), // 2 minutes max
            max_memory_bytes: 1024 * 1024 * 1024, // 1GB max
            warn_threshold: 50_000,       // Warn at 50k files
        }
    }
    
    /// Create limits for MCP operations (more conservative)
    pub fn for_mcp() -> Self {
        Self {
            max_files: 100_000,           // 100k files max for MCP
            max_duration: Duration::from_secs(60),  // 1 minute max
            max_memory_bytes: 512 * 1024 * 1024,   // 512MB max
            warn_threshold: 10_000,       // Warn at 10k files
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
    
    /// Check if we should continue scanning
    pub fn should_continue(&self) -> Result<(), String> {
        // Check file count
        let count = self.file_count.load(Ordering::Relaxed);
        if self.limits.max_files > 0 && count >= self.limits.max_files {
            return Err(format!(
                "Scan aborted: Reached maximum file limit of {} files", 
                self.limits.max_files
            ));
        }
        
        // Check duration
        if self.start_time.elapsed() > self.limits.max_duration {
            return Err(format!(
                "Scan aborted: Exceeded maximum duration of {:?}", 
                self.limits.max_duration
            ));
        }
        
        // Check memory (estimated)
        let memory = self.estimated_memory.load(Ordering::Relaxed);
        if memory > self.limits.max_memory_bytes {
            return Err(format!(
                "Scan aborted: Estimated memory usage ({} MB) exceeds limit ({} MB)", 
                memory / (1024 * 1024),
                self.limits.max_memory_bytes / (1024 * 1024)
            ));
        }
        
        // Warn if approaching limits
        if count > self.limits.warn_threshold && self.warned.load(Ordering::Relaxed) == 0 {
            self.warned.store(1, Ordering::Relaxed);
            eprintln!("⚠️  Warning: Scanning large directory ({} files so far)", count);
            eprintln!("   Consider using --max-depth or --stream mode");
        }
        
        Ok(())
    }
    
    /// Increment file count
    pub fn add_file(&self, estimated_node_size: usize) {
        self.file_count.fetch_add(1, Ordering::Relaxed);
        self.estimated_memory.fetch_add(estimated_node_size, Ordering::Relaxed);
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
pub fn estimate_node_size(path_len: usize) -> usize {
    // Base struct size + path string + some overhead
    std::mem::size_of::<crate::scanner::FileNode>() + path_len + 64
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
        
        let tracker = ScannerSafetyTracker::new(limits);
        
        // Should start OK
        assert!(tracker.should_continue().is_ok());
        
        // Add files until we hit the limit
        for _ in 0..10 {
            tracker.add_file(100);
        }
        
        // Should now fail
        assert!(tracker.should_continue().is_err());
    }
}