//! Bidirectional sync engine between local directories and Google Drive
//!
//! Algorithm:
//! 1. Load previous SyncState
//! 2. Scan local: walk dir, hash files, compare with stored state
//! 3. Scan remote: list Drive folder, compare checksums with stored state
//! 4. Classify each file: InSync, LocalNewer, RemoteNewer, Conflict, etc.
//! 5. Resolve conflicts per strategy
//! 6. Execute uploads/downloads
//! 7. Save updated state

use anyhow::{Context, Result};
use chrono::Utc;
use std::collections::HashMap;
use std::path::Path;

use super::drive_client::DriveClient;
use super::models::*;
use super::sync_state::{scan_local_directory, SyncStateManager};

/// Executes sync operations between local and Drive
pub struct SyncEngine<'a> {
    drive: &'a DriveClient,
    state_manager: SyncStateManager,
}

impl<'a> SyncEngine<'a> {
    pub fn new(drive: &'a DriveClient) -> Result<Self> {
        let state_manager = SyncStateManager::default_dir()?;
        Ok(Self {
            drive,
            state_manager,
        })
    }

    /// Run a full sync operation
    pub async fn sync(&self, config: &mut SyncState) -> Result<SyncReport> {
        let start = std::time::Instant::now();
        let mut report = SyncReport {
            uploaded: 0,
            downloaded: 0,
            conflicts: 0,
            skipped: 0,
            errors: Vec::new(),
            duration_ms: 0,
        };

        // Compute what needs to happen
        let actions = self.compute_delta(config).await?;

        // Execute each action
        let local_root = Path::new(&config.local_path);
        for action in &actions {
            match action {
                SyncAction::Upload { relative_path } => {
                    let local_path = local_root.join(relative_path);
                    match std::fs::read(&local_path) {
                        Ok(content) => {
                            let mime = mime_guess::from_path(&local_path)
                                .first_or_octet_stream()
                                .to_string();
                            let name = local_path
                                .file_name()
                                .and_then(|n| n.to_str())
                                .unwrap_or("unknown");

                            match self
                                .drive
                                .upload_file(&config.drive_folder_id, name, &content, &mime)
                                .await
                            {
                                Ok(_id) => report.uploaded += 1,
                                Err(e) => {
                                    report.errors.push(format!("Upload {relative_path}: {e}"))
                                }
                            }
                        }
                        Err(e) => report.errors.push(format!("Read {relative_path}: {e}")),
                    }
                }
                SyncAction::Download {
                    relative_path,
                    drive_file_id,
                } => match self.drive.download_file(drive_file_id).await {
                    Ok(content) => {
                        let local_path = local_root.join(relative_path);
                        if let Some(parent) = local_path.parent() {
                            let _ = std::fs::create_dir_all(parent);
                        }
                        match std::fs::write(&local_path, &content) {
                            Ok(()) => report.downloaded += 1,
                            Err(e) => report.errors.push(format!("Write {relative_path}: {e}")),
                        }
                    }
                    Err(e) => report.errors.push(format!("Download {relative_path}: {e}")),
                },
                SyncAction::Conflict { relative_path, .. } => {
                    report.conflicts += 1;
                    report.errors.push(format!(
                        "Conflict: {relative_path} (manual resolution needed)"
                    ));
                }
                SyncAction::Skip {
                    relative_path,
                    reason,
                } => {
                    report.skipped += 1;
                    tracing::debug!("Skipped {}: {}", relative_path, reason);
                }
                SyncAction::Delete {
                    relative_path,
                    side,
                } => {
                    tracing::info!("Would delete {} on {}", relative_path, side);
                    report.skipped += 1; // Don't auto-delete, just report
                }
            }
        }

        // Update sync state
        config.last_sync = Some(Utc::now());
        self.state_manager.save(config)?;

        report.duration_ms = start.elapsed().as_millis() as u64;
        Ok(report)
    }

    /// Compute the delta between local and remote without executing
    pub async fn compute_delta(&self, config: &SyncState) -> Result<Vec<SyncAction>> {
        let local_root = Path::new(&config.local_path);

        // Scan local files
        let local_files =
            scan_local_directory(local_root).context("Failed to scan local directory")?;

        // Scan remote files
        let remote_files = self
            .drive
            .list_files_recursive(&config.drive_folder_id, "")
            .await
            .context("Failed to list remote files")?;

        // Build remote lookup: relative_path -> (hash, modified, file_id)
        let remote_map: HashMap<String, (&str, Option<chrono::DateTime<Utc>>, &str)> = remote_files
            .iter()
            .map(|(path, meta)| {
                (
                    path.clone(),
                    (
                        meta.md5_checksum.as_deref().unwrap_or(""),
                        meta.modified_time,
                        meta.id.as_str(),
                    ),
                )
            })
            .collect();

        // Build previous state lookup
        let prev_map: HashMap<&str, &FileSyncEntry> = config
            .file_states
            .iter()
            .map(|e| (e.relative_path.as_str(), e))
            .collect();

        let mut actions = Vec::new();

        // Check all local files
        for (path, (local_hash, local_modified)) in &local_files {
            if let Some(&(remote_hash, remote_modified, file_id)) = remote_map.get(path) {
                // File exists on both sides
                let prev = prev_map.get(path.as_str());
                let local_changed = prev
                    .and_then(|p| p.local_hash.as_ref())
                    .map(|h| h != local_hash)
                    .unwrap_or(true);
                let remote_changed = prev
                    .and_then(|p| p.remote_hash.as_ref())
                    .map(|h| h != remote_hash)
                    .unwrap_or(true);

                match (local_changed, remote_changed) {
                    (false, false) => {} // In sync, no action
                    (true, false) => {
                        if config.direction != SyncDirection::DriveToLocal {
                            actions.push(SyncAction::Upload {
                                relative_path: path.clone(),
                            });
                        }
                    }
                    (false, true) => {
                        if config.direction != SyncDirection::LocalToDrive {
                            actions.push(SyncAction::Download {
                                relative_path: path.clone(),
                                drive_file_id: file_id.to_string(),
                            });
                        }
                    }
                    (true, true) => {
                        // Conflict - resolve based on strategy
                        actions.push(self.resolve_conflict(
                            path,
                            file_id,
                            *local_modified,
                            remote_modified.unwrap_or_else(Utc::now),
                            &config.conflict_resolution,
                        ));
                    }
                }
            } else {
                // Local only
                if config.direction != SyncDirection::DriveToLocal {
                    actions.push(SyncAction::Upload {
                        relative_path: path.clone(),
                    });
                }
            }
        }

        // Check for remote-only files
        for (path, meta) in &remote_files {
            if !local_files.contains_key(path) && config.direction != SyncDirection::LocalToDrive {
                actions.push(SyncAction::Download {
                    relative_path: path.clone(),
                    drive_file_id: meta.id.clone(),
                });
            }
        }

        Ok(actions)
    }

    fn resolve_conflict(
        &self,
        path: &str,
        file_id: &str,
        local_modified: chrono::DateTime<Utc>,
        remote_modified: chrono::DateTime<Utc>,
        strategy: &ConflictStrategy,
    ) -> SyncAction {
        match strategy {
            ConflictStrategy::NewerWins => {
                if local_modified > remote_modified {
                    SyncAction::Upload {
                        relative_path: path.to_string(),
                    }
                } else {
                    SyncAction::Download {
                        relative_path: path.to_string(),
                        drive_file_id: file_id.to_string(),
                    }
                }
            }
            ConflictStrategy::LocalWins => SyncAction::Upload {
                relative_path: path.to_string(),
            },
            ConflictStrategy::DriveWins => SyncAction::Download {
                relative_path: path.to_string(),
                drive_file_id: file_id.to_string(),
            },
            ConflictStrategy::AskUser | ConflictStrategy::KeepBoth => SyncAction::Conflict {
                relative_path: path.to_string(),
                local_modified,
                remote_modified,
            },
        }
    }

    /// Get status summary for a sync mapping
    pub fn status(&self, sync_id: &str) -> Result<String> {
        match self.state_manager.load(sync_id)? {
            Some(state) => {
                let last = state
                    .last_sync
                    .map(|t| t.format("%Y-%m-%d %H:%M UTC").to_string())
                    .unwrap_or_else(|| "never".to_string());
                Ok(format!(
                    "Sync: {} <-> Drive:{}\n  Direction: {:?}\n  Last sync: {}\n  Tracked files: {}\n  Conflict strategy: {:?}",
                    state.local_path,
                    state.drive_folder_id,
                    state.direction,
                    last,
                    state.file_states.len(),
                    state.conflict_resolution,
                ))
            }
            None => Ok(format!("No sync state found for '{sync_id}'")),
        }
    }
}
