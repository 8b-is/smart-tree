//
// -----------------------------------------------------------------------------
//  SCANNER STATE: Change Detection Between Scans
//
//  This module manages persistent state for Smart Tree's intelligent scanning.
//  By remembering what we saw last time, we can tell you "what changed" instead
//  of "everything that exists."
//
//  Key concepts:
//  - ScanState: Persisted snapshot of a directory at a point in time
//  - FileSignature: Hash + metadata for fast change detection
//  - ScanDelta: The diff between two scans
//  - HotDirectory: Directories with frequent changes worth watching
//
//  "Don't repeat what hasn't changed." - Omni
// -----------------------------------------------------------------------------
//

use crate::scanner_interest::{ChangeType, InterestLevel};
use anyhow::Result;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::fs::File;
use std::io::{BufReader, BufWriter, Read};
use std::path::{Path, PathBuf};
use std::time::SystemTime;

/// File signature for change detection
/// Uses a combination of hash and metadata for fast comparison
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct FileSignature {
    /// Blake3/SHA256 hash of file contents (for files < 10MB)
    /// For larger files or directories, this is None
    pub content_hash: Option<String>,

    /// Last modification time
    pub mtime: SystemTime,

    /// File size in bytes
    pub size: u64,

    /// File permissions (Unix mode)
    pub permissions: u32,

    /// Is this a directory?
    pub is_dir: bool,

    /// Is this a symlink?
    pub is_symlink: bool,
}

impl FileSignature {
    /// Create a signature from a path
    pub fn from_path(path: &Path) -> Result<Self> {
        let metadata = std::fs::symlink_metadata(path)?;
        let is_symlink = metadata.file_type().is_symlink();
        let is_dir = metadata.is_dir();

        // Get actual metadata (following symlinks if needed)
        let (size, mtime, permissions) = if is_symlink {
            // For symlinks, use symlink metadata
            (
                0,
                metadata.modified().unwrap_or(SystemTime::UNIX_EPOCH),
                Self::get_permissions(&metadata),
            )
        } else {
            (
                metadata.len(),
                metadata.modified().unwrap_or(SystemTime::UNIX_EPOCH),
                Self::get_permissions(&metadata),
            )
        };

        // Only hash small files (< 10MB)
        let content_hash = if !is_dir && !is_symlink && size < 10_000_000 {
            Self::hash_file(path).ok()
        } else {
            None
        };

        Ok(Self {
            content_hash,
            mtime,
            size,
            permissions,
            is_dir,
            is_symlink,
        })
    }

    /// Quick check if file might have changed (without hashing)
    pub fn quick_changed(&self, other: &Self) -> bool {
        self.mtime != other.mtime || self.size != other.size || self.permissions != other.permissions
    }

    /// Full check if file has changed (including hash if available)
    pub fn changed(&self, other: &Self) -> bool {
        if self.quick_changed(other) {
            return true;
        }

        // If both have hashes, compare them
        match (&self.content_hash, &other.content_hash) {
            (Some(h1), Some(h2)) => h1 != h2,
            _ => false, // Can't determine from hash, assume unchanged
        }
    }

    /// Hash a file's contents using SHA256
    fn hash_file(path: &Path) -> Result<String> {
        let file = File::open(path)?;
        let mut reader = BufReader::new(file);
        let mut hasher = Sha256::new();
        let mut buffer = [0u8; 8192];

        loop {
            let bytes_read = reader.read(&mut buffer)?;
            if bytes_read == 0 {
                break;
            }
            hasher.update(&buffer[..bytes_read]);
        }

        Ok(hex::encode(hasher.finalize()))
    }

    #[cfg(unix)]
    fn get_permissions(metadata: &std::fs::Metadata) -> u32 {
        use std::os::unix::fs::PermissionsExt;
        metadata.permissions().mode()
    }

    #[cfg(not(unix))]
    fn get_permissions(_metadata: &std::fs::Metadata) -> u32 {
        0o644 // Default permissions for non-Unix
    }
}

/// Persistent state for a scanned directory
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScanState {
    /// When this state was created
    pub scan_time: SystemTime,

    /// Root path that was scanned
    pub root: PathBuf,

    /// File signatures for all scanned files
    pub signatures: HashMap<PathBuf, FileSignature>,

    /// Directories marked as "hot" (frequently changing)
    pub hot_directories: Vec<HotDirectory>,

    /// Total files in this state
    pub total_files: u64,

    /// Total directories in this state
    pub total_dirs: u64,

    /// Version of the state format (for migrations)
    pub version: u32,
}

impl ScanState {
    /// Current state format version
    pub const CURRENT_VERSION: u32 = 1;

    /// Create a new empty state
    pub fn new(root: PathBuf) -> Self {
        Self {
            scan_time: SystemTime::now(),
            root,
            signatures: HashMap::new(),
            hot_directories: Vec::new(),
            total_files: 0,
            total_dirs: 0,
            version: Self::CURRENT_VERSION,
        }
    }

    /// Add a file signature to the state
    pub fn add_signature(&mut self, path: PathBuf, sig: FileSignature) {
        if sig.is_dir {
            self.total_dirs += 1;
        } else {
            self.total_files += 1;
        }
        self.signatures.insert(path, sig);
    }

    /// Get the state file path for a given directory
    pub fn state_path(root: &Path) -> PathBuf {
        let state_dir = dirs::home_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join(".st")
            .join("scan_states");

        // Create a safe filename from the path
        let safe_name = root
            .to_string_lossy()
            .replace(['/', '\\', ':'], "_")
            .trim_matches('_')
            .to_string();

        state_dir.join(format!("{}.state.json", safe_name))
    }

    /// Save state to disk
    pub fn save(&self) -> Result<PathBuf> {
        let path = Self::state_path(&self.root);

        // Ensure directory exists
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        let file = File::create(&path)?;
        let writer = BufWriter::new(file);
        serde_json::to_writer_pretty(writer, self)?;

        Ok(path)
    }

    /// Load state from disk
    pub fn load(root: &Path) -> Result<Option<Self>> {
        let path = Self::state_path(root);

        if !path.exists() {
            return Ok(None);
        }

        let file = File::open(&path)?;
        let reader = BufReader::new(file);
        let state: Self = serde_json::from_reader(reader)?;

        // Check version compatibility
        if state.version > Self::CURRENT_VERSION {
            anyhow::bail!(
                "State file version {} is newer than supported version {}",
                state.version,
                Self::CURRENT_VERSION
            );
        }

        Ok(Some(state))
    }

    /// Compare with another state and produce a delta
    pub fn diff(&self, newer: &ScanState) -> ScanDelta {
        let mut delta = ScanDelta::new(self.root.clone());

        // Find added and modified files
        for (path, new_sig) in &newer.signatures {
            match self.signatures.get(path) {
                None => {
                    // File was added
                    delta.added.push(path.clone());
                }
                Some(old_sig) => {
                    if new_sig.changed(old_sig) {
                        // Determine type of change
                        let change_type = if old_sig.permissions != new_sig.permissions
                            && old_sig.size == new_sig.size
                            && old_sig.content_hash == new_sig.content_hash
                        {
                            ChangeType::PermissionChanged
                        } else if old_sig.is_dir != new_sig.is_dir
                            || old_sig.is_symlink != new_sig.is_symlink
                        {
                            ChangeType::TypeChanged
                        } else {
                            ChangeType::Modified
                        };
                        delta.modified.push((path.clone(), change_type));
                    }
                }
            }
        }

        // Find deleted files
        for path in self.signatures.keys() {
            if !newer.signatures.contains_key(path) {
                delta.deleted.push(path.clone());
            }
        }

        // Update summary
        delta.nothing_changed =
            delta.added.is_empty() && delta.modified.is_empty() && delta.deleted.is_empty();
        delta.older_scan_time = Some(self.scan_time);
        delta.newer_scan_time = Some(newer.scan_time);

        delta
    }
}

/// The difference between two scans
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScanDelta {
    /// Root path for this delta
    pub root: PathBuf,

    /// Files that were added since last scan
    pub added: Vec<PathBuf>,

    /// Files that were modified (path and type of change)
    pub modified: Vec<(PathBuf, ChangeType)>,

    /// Files that were deleted since last scan
    pub deleted: Vec<PathBuf>,

    /// True if nothing changed at all
    pub nothing_changed: bool,

    /// Time of the older scan
    pub older_scan_time: Option<SystemTime>,

    /// Time of the newer scan
    pub newer_scan_time: Option<SystemTime>,
}

impl ScanDelta {
    /// Create a new empty delta
    pub fn new(root: PathBuf) -> Self {
        Self {
            root,
            added: Vec::new(),
            modified: Vec::new(),
            deleted: Vec::new(),
            nothing_changed: true,
            older_scan_time: None,
            newer_scan_time: None,
        }
    }

    /// Get total number of changes
    pub fn change_count(&self) -> usize {
        self.added.len() + self.modified.len() + self.deleted.len()
    }

    /// Get a summary string
    pub fn summary(&self) -> String {
        if self.nothing_changed {
            return String::from("No changes since last scan");
        }

        let mut parts = Vec::new();

        if !self.added.is_empty() {
            parts.push(format!("+{} added", self.added.len()));
        }
        if !self.modified.is_empty() {
            parts.push(format!("~{} modified", self.modified.len()));
        }
        if !self.deleted.is_empty() {
            parts.push(format!("-{} deleted", self.deleted.len()));
        }

        parts.join(", ")
    }

    /// Get paths by interest level (for smart formatting)
    pub fn paths_by_interest(&self) -> HashMap<InterestLevel, Vec<PathBuf>> {
        let mut result: HashMap<InterestLevel, Vec<PathBuf>> = HashMap::new();

        // Deleted files are important
        for path in &self.deleted {
            result
                .entry(InterestLevel::Important)
                .or_default()
                .push(path.clone());
        }

        // Modified files are notable to important
        for (path, change_type) in &self.modified {
            let level = match change_type {
                ChangeType::PermissionChanged => InterestLevel::Important,
                ChangeType::TypeChanged => InterestLevel::Important,
                _ => InterestLevel::Notable,
            };
            result.entry(level).or_default().push(path.clone());
        }

        // Added files are notable
        for path in &self.added {
            result
                .entry(InterestLevel::Notable)
                .or_default()
                .push(path.clone());
        }

        result
    }
}

/// A directory marked as "hot" due to frequent changes
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HotDirectory {
    /// Path to the hot directory
    pub path: PathBuf,

    /// Number of changes in the tracking period
    pub change_count: u32,

    /// When we started tracking this directory
    pub tracking_since: SystemTime,

    /// Average changes per day
    pub changes_per_day: f32,

    /// Most active hours (0-23)
    pub active_hours: Vec<u8>,

    /// Interest level based on activity
    pub interest_level: InterestLevel,
}

impl HotDirectory {
    /// Create a new hot directory entry
    pub fn new(path: PathBuf) -> Self {
        Self {
            path,
            change_count: 0,
            tracking_since: SystemTime::now(),
            changes_per_day: 0.0,
            active_hours: Vec::new(),
            interest_level: InterestLevel::Notable,
        }
    }

    /// Record a change in this directory
    pub fn record_change(&mut self) {
        self.change_count += 1;

        // Update changes per day
        if let Ok(duration) = SystemTime::now().duration_since(self.tracking_since) {
            let days = duration.as_secs_f32() / 86400.0;
            if days > 0.0 {
                self.changes_per_day = self.change_count as f32 / days;
            }
        }

        // Update interest level based on activity
        self.interest_level = if self.changes_per_day > 50.0 {
            InterestLevel::Critical
        } else if self.changes_per_day > 20.0 {
            InterestLevel::Important
        } else if self.changes_per_day > 5.0 {
            InterestLevel::Notable
        } else {
            InterestLevel::Background
        };
    }

    /// Check if this directory is considered "hot"
    pub fn is_hot(&self) -> bool {
        self.changes_per_day >= 10.0
    }
}

/// Statistics about change detection
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ChangeStats {
    /// Total files compared
    pub files_compared: u64,

    /// Files that were unchanged
    pub unchanged: u64,

    /// Files that were added
    pub added: u64,

    /// Files that were modified
    pub modified: u64,

    /// Files that were deleted
    pub deleted: u64,

    /// Time elapsed for comparison
    pub comparison_time_ms: u64,
}

impl ChangeStats {
    /// Get the percentage of files that changed
    pub fn change_percentage(&self) -> f32 {
        if self.files_compared == 0 {
            return 0.0;
        }
        (self.added + self.modified + self.deleted) as f32 / self.files_compared as f32 * 100.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;
        use tempfile::TempDir;

    #[test]
    fn test_file_signature_creation() {
        let tmp = TempDir::new().unwrap();
        let file_path = tmp.path().join("test.txt");
        std::fs::write(&file_path, "Hello, world!").unwrap();

        let sig = FileSignature::from_path(&file_path).unwrap();
        assert!(!sig.is_dir);
        assert!(!sig.is_symlink);
        assert_eq!(sig.size, 13);
        assert!(sig.content_hash.is_some());
    }

    #[test]
    fn test_file_signature_change_detection() {
        let tmp = TempDir::new().unwrap();
        let file_path = tmp.path().join("test.txt");

        // Create initial file
        std::fs::write(&file_path, "Hello").unwrap();
        let sig1 = FileSignature::from_path(&file_path).unwrap();

        // Modify file
        std::fs::write(&file_path, "Hello, world!").unwrap();
        let sig2 = FileSignature::from_path(&file_path).unwrap();

        assert!(sig2.changed(&sig1));
    }

    #[test]
    fn test_scan_state_persistence() {
        let tmp = TempDir::new().unwrap();
        let root = tmp.path().to_path_buf();

        // Create state
        let mut state = ScanState::new(root.clone());
        state.add_signature(
            root.join("test.txt"),
            FileSignature {
                content_hash: Some("abc123".to_string()),
                mtime: SystemTime::now(),
                size: 100,
                permissions: 0o644,
                is_dir: false,
                is_symlink: false,
            },
        );

        // Save and reload
        let save_path = state.save().unwrap();
        assert!(save_path.exists());

        let loaded = ScanState::load(&root).unwrap().unwrap();
        assert_eq!(loaded.total_files, 1);
        assert!(loaded.signatures.contains_key(&root.join("test.txt")));
    }

    #[test]
    fn test_scan_delta() {
        let root = PathBuf::from("/test");

        // Create old state
        let mut old_state = ScanState::new(root.clone());
        old_state.add_signature(
            root.join("unchanged.txt"),
            FileSignature {
                content_hash: Some("hash1".to_string()),
                mtime: SystemTime::UNIX_EPOCH,
                size: 100,
                permissions: 0o644,
                is_dir: false,
                is_symlink: false,
            },
        );
        old_state.add_signature(
            root.join("deleted.txt"),
            FileSignature {
                content_hash: Some("hash2".to_string()),
                mtime: SystemTime::UNIX_EPOCH,
                size: 50,
                permissions: 0o644,
                is_dir: false,
                is_symlink: false,
            },
        );

        // Create new state
        let mut new_state = ScanState::new(root.clone());
        new_state.add_signature(
            root.join("unchanged.txt"),
            FileSignature {
                content_hash: Some("hash1".to_string()),
                mtime: SystemTime::UNIX_EPOCH,
                size: 100,
                permissions: 0o644,
                is_dir: false,
                is_symlink: false,
            },
        );
        new_state.add_signature(
            root.join("added.txt"),
            FileSignature {
                content_hash: Some("hash3".to_string()),
                mtime: SystemTime::now(),
                size: 200,
                permissions: 0o644,
                is_dir: false,
                is_symlink: false,
            },
        );

        let delta = old_state.diff(&new_state);

        assert!(!delta.nothing_changed);
        assert_eq!(delta.added.len(), 1);
        assert_eq!(delta.deleted.len(), 1);
        assert!(delta.modified.is_empty());
        assert!(delta.added.contains(&root.join("added.txt")));
        assert!(delta.deleted.contains(&root.join("deleted.txt")));
    }

    #[test]
    fn test_hot_directory() {
        let mut hot = HotDirectory::new(PathBuf::from("/src"));

        // Record many changes
        for _ in 0..100 {
            hot.record_change();
        }

        assert!(hot.is_hot());
        assert!(hot.changes_per_day > 0.0);
    }
}
