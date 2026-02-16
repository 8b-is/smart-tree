//! Sync state tracking and persistence
//!
//! Tracks the state of each file in a sync mapping between
//! a local directory and a Google Drive folder.

use anyhow::{Context, Result};
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

use super::models::{FileSyncEntry, SyncState, SyncStatus};

/// Manages sync state persistence
pub struct SyncStateManager {
    state_dir: PathBuf,
}

impl SyncStateManager {
    pub fn new(state_dir: PathBuf) -> Result<Self> {
        fs::create_dir_all(&state_dir)
            .with_context(|| format!("Failed to create sync state dir: {}", state_dir.display()))?;
        Ok(Self { state_dir })
    }

    /// Default state directory: ~/.st/google/sync_states/
    pub fn default_dir() -> Result<Self> {
        Self::new(super::google_state_dir().join("sync_states"))
    }

    /// Load sync state for a given sync ID
    pub fn load(&self, sync_id: &str) -> Result<Option<SyncState>> {
        let path = self.state_dir.join(format!("{sync_id}.json"));
        if !path.exists() {
            return Ok(None);
        }
        let json = fs::read_to_string(&path)
            .with_context(|| format!("Failed to read sync state: {}", path.display()))?;
        Ok(Some(serde_json::from_str(&json)?))
    }

    /// Save sync state
    pub fn save(&self, state: &SyncState) -> Result<()> {
        let path = self.state_dir.join(format!("{}.json", state.sync_id));
        let json = serde_json::to_string_pretty(state)?;
        fs::write(&path, json)
            .with_context(|| format!("Failed to write sync state: {}", path.display()))?;
        Ok(())
    }

    /// List all sync states
    pub fn list(&self) -> Result<Vec<SyncState>> {
        let mut states = Vec::new();
        for entry in fs::read_dir(&self.state_dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.extension().map(|e| e == "json").unwrap_or(false) {
                let json = fs::read_to_string(&path)?;
                if let Ok(state) = serde_json::from_str::<SyncState>(&json) {
                    states.push(state);
                }
            }
        }
        Ok(states)
    }

    /// Delete a sync state
    pub fn delete(&self, sync_id: &str) -> Result<()> {
        let path = self.state_dir.join(format!("{sync_id}.json"));
        if path.exists() {
            fs::remove_file(&path)?;
        }
        Ok(())
    }
}

/// Scan a local directory and build a map of relative_path -> (hash, modified_time)
pub fn scan_local_directory(
    root: &Path,
) -> Result<HashMap<String, (String, chrono::DateTime<chrono::Utc>)>> {
    let mut files = HashMap::new();

    if !root.is_dir() {
        anyhow::bail!("Not a directory: {}", root.display());
    }

    scan_dir_recursive(root, root, &mut files)?;
    Ok(files)
}

fn scan_dir_recursive(
    root: &Path,
    current: &Path,
    files: &mut HashMap<String, (String, chrono::DateTime<chrono::Utc>)>,
) -> Result<()> {
    for entry in fs::read_dir(current)? {
        let entry = entry?;
        let path = entry.path();

        // Skip hidden files and common ignore patterns
        if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
            if name.starts_with('.') || name == "node_modules" || name == "target" {
                continue;
            }
        }

        if path.is_dir() {
            scan_dir_recursive(root, &path, files)?;
        } else if path.is_file() {
            let relative = path
                .strip_prefix(root)
                .context("Failed to compute relative path")?
                .to_string_lossy()
                .to_string();

            let hash = compute_file_hash(&path)?;
            let metadata = fs::metadata(&path)?;
            let modified: chrono::DateTime<chrono::Utc> = metadata.modified()?.into();

            files.insert(relative, (hash, modified));
        }
    }

    Ok(())
}

/// Compute SHA-256 hash of a file
pub fn compute_file_hash(path: &Path) -> Result<String> {
    let data = fs::read(path)
        .with_context(|| format!("Failed to read file for hashing: {}", path.display()))?;
    let mut hasher = Sha256::new();
    hasher.update(&data);
    Ok(hex::encode(hasher.finalize()))
}

/// Compare local and remote states to determine file sync status
pub fn classify_file(
    entry: &FileSyncEntry,
    local_hash: Option<&str>,
    local_modified: Option<chrono::DateTime<chrono::Utc>>,
    remote_hash: Option<&str>,
    remote_modified: Option<chrono::DateTime<chrono::Utc>>,
) -> SyncStatus {
    match (local_hash, remote_hash) {
        (None, None) => SyncStatus::Deleted,
        (Some(_), None) => SyncStatus::LocalOnly,
        (None, Some(_)) => SyncStatus::RemoteOnly,
        (Some(lh), Some(rh)) => {
            // Both exist - check if changed since last sync
            let local_changed = entry
                .local_hash
                .as_ref()
                .map(|prev| prev != lh)
                .unwrap_or(true);
            let remote_changed = entry
                .remote_hash
                .as_ref()
                .map(|prev| prev != rh)
                .unwrap_or(true);

            match (local_changed, remote_changed) {
                (false, false) => SyncStatus::InSync,
                (true, false) => SyncStatus::LocalNewer,
                (false, true) => SyncStatus::RemoteNewer,
                (true, true) => {
                    // Both changed - might still be same content
                    // Drive uses MD5, we use SHA-256, so can't compare directly
                    // Use timestamps as tiebreaker
                    match (local_modified, remote_modified) {
                        (Some(lm), Some(rm)) if lm > rm => SyncStatus::LocalNewer,
                        (Some(lm), Some(rm)) if rm > lm => SyncStatus::RemoteNewer,
                        _ => SyncStatus::Conflict,
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::google_sync::models::*;

    #[test]
    fn test_classify_both_unchanged() {
        let entry = FileSyncEntry {
            relative_path: "test.txt".to_string(),
            local_hash: Some("abc123".to_string()),
            remote_hash: Some("def456".to_string()),
            local_modified: None,
            remote_modified: None,
            drive_file_id: Some("drive_id".to_string()),
            status: SyncStatus::InSync,
        };

        let status = classify_file(&entry, Some("abc123"), None, Some("def456"), None);
        assert_eq!(status, SyncStatus::InSync);
    }

    #[test]
    fn test_classify_local_only() {
        let entry = FileSyncEntry {
            relative_path: "new.txt".to_string(),
            local_hash: None,
            remote_hash: None,
            local_modified: None,
            remote_modified: None,
            drive_file_id: None,
            status: SyncStatus::LocalOnly,
        };

        let status = classify_file(&entry, Some("abc123"), None, None, None);
        assert_eq!(status, SyncStatus::LocalOnly);
    }
}
