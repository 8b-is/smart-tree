//! Cache implementation for MCP server

use dashmap::DashMap;
use std::time::{Duration, Instant};

/// Cached entry with expiration
#[derive(Clone)]
struct CacheEntry {
    value: String,
    expires_at: Instant,
}

/// Thread-safe cache for analysis results
pub struct AnalysisCache {
    entries: DashMap<String, CacheEntry>,
    ttl: Duration,
}

impl AnalysisCache {
    /// Create a new cache with the given TTL in seconds
    pub fn new(ttl_seconds: u64) -> Self {
        Self {
            entries: DashMap::new(),
            ttl: Duration::from_secs(ttl_seconds),
        }
    }

    /// Get a value from the cache if it exists and hasn't expired
    pub async fn get(&self, key: &str) -> Option<String> {
        self.entries.get(key).and_then(|entry| {
            if entry.expires_at > Instant::now() {
                Some(entry.value.clone())
            } else {
                // Remove expired entry
                drop(entry);
                self.entries.remove(key);
                None
            }
        })
    }

    /// Set a value in the cache
    pub async fn set(&self, key: String, value: String) {
        let entry = CacheEntry {
            value,
            expires_at: Instant::now() + self.ttl,
        };
        self.entries.insert(key, entry);
    }

    /// Clear all expired entries
    pub async fn cleanup(&self) {
        let now = Instant::now();
        self.entries.retain(|_, entry| entry.expires_at > now);
    }

    /// Get the number of cached entries
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// Check if cache is empty
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// Clear all entries
    pub fn clear(&self) {
        self.entries.clear();
    }

    /// Get cache statistics
    pub async fn stats(&self) -> CacheStats {
        let mut total_size = 0;
        let mut expired = 0;
        let now = Instant::now();

        for entry in self.entries.iter() {
            total_size += entry.value.len();
            if entry.expires_at <= now {
                expired += 1;
            }
        }

        CacheStats {
            entries: self.entries.len(),
            size: total_size,
            expired,
            hits: 0,   // Would need to track this
            misses: 0, // Would need to track this
            hit_rate: 0.0,
        }
    }
}

/// Cache statistics
pub struct CacheStats {
    pub entries: usize,
    pub size: usize,
    pub expired: usize,
    pub hits: u64,
    pub misses: u64,
    pub hit_rate: f64,
}
