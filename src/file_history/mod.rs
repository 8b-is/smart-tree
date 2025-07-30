//! File History Tracking Module - The Ultimate Context-Driven System!
//! 
//! Tracks all AI file manipulations in ~/.mem8/.filehistory/
//! with hash-based change detection and append-first operations.
//! 
//! 🎸 The Cheet says: "Every file tells a story, let's remember them all!"

use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};
use chrono::{DateTime, Utc, Timelike, Datelike};
use sha2::{Sha256, Digest};
use serde::{Serialize, Deserialize};
use anyhow::Result;

pub mod operations;
pub mod tracker;

pub use operations::*;
pub use tracker::*;

/// File history configuration
pub struct FileHistoryConfig {
    /// Base directory for file history (default: ~/.mem8/.filehistory/)
    pub base_dir: PathBuf,
    /// Whether to auto-create directories
    pub auto_create: bool,
    /// Prefer append operations when possible
    pub prefer_append: bool,
}

impl Default for FileHistoryConfig {
    fn default() -> Self {
        let home = dirs::home_dir().unwrap_or_else(|| PathBuf::from("."));
        Self {
            base_dir: home.join(".mem8").join(".filehistory"),
            auto_create: true,
            prefer_append: true,
        }
    }
}

/// Get timestamp at 10-minute resolution
pub fn get_time_bucket() -> (String, u64) {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs();
    
    let datetime = DateTime::<Utc>::from(UNIX_EPOCH + std::time::Duration::from_secs(now));
    let minute = datetime.minute() / 10 * 10; // Round down to 10-minute bucket
    
    let filename = format!(
        "{:04}{:02}{:02}_{:02}{:02}",
        datetime.year(),
        datetime.month(),
        datetime.day(),
        datetime.hour(),
        minute
    );
    
    (filename, now)
}

/// Calculate SHA256 hash of file contents
pub fn hash_file(path: &Path) -> Result<String> {
    let contents = fs::read(path)?;
    let mut hasher = Sha256::new();
    hasher.update(&contents);
    Ok(format!("{:x}", hasher.finalize()))
}

/// Get project ID from path (uses canonical path as ID)
pub fn get_project_id(path: &Path) -> Result<String> {
    let canonical = path.canonicalize()?;
    let mut hasher = Sha256::new();
    hasher.update(canonical.to_string_lossy().as_bytes());
    Ok(format!("{:x}", hasher.finalize())[..16].to_string())
}