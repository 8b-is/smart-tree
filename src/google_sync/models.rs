//! Data models for Google Sync
//!
//! All the structs that flow through the Gmail backup, Drive sync,
//! warm storage analysis, and file intelligence systems.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

// ── Authentication ──────────────────────────────────────────────────

/// Which auth method is configured
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum AuthMethod {
    UserOAuth2 {
        client_id: String,
        client_secret: String,
        redirect_port: Option<u16>,
    },
    ServiceAccount {
        key_path: String,
        delegate_email: Option<String>,
    },
}

/// Persisted auth configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthConfig {
    pub method: AuthMethod,
    pub scopes: Vec<String>,
    pub account_email: Option<String>,
    pub authenticated_at: Option<DateTime<Utc>>,
}

// ── Gmail ───────────────────────────────────────────────────────────

/// Lightweight email metadata for indexing and analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmailMetadata {
    pub message_id: String,
    pub thread_id: String,
    pub subject: String,
    pub from: String,
    pub to: Vec<String>,
    pub date: DateTime<Utc>,
    pub labels: Vec<String>,
    pub size_bytes: u64,
    pub has_attachments: bool,
    pub snippet: String,
    pub is_read: bool,
    pub is_replied: bool,
}

/// Gmail backup progress tracker (persisted for resume)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackupProgress {
    pub total_messages: u64,
    pub backed_up: u64,
    pub skipped: u64,
    pub failed: u64,
    pub drive_folder_id: String,
    pub started_at: DateTime<Utc>,
    pub last_message_id: Option<String>,
    pub query: Option<String>,
    pub label: Option<String>,
}

// ── Warm Storage Analysis ───────────────────────────────────────────

/// Archive suggestion for a single email
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WarmStorageScore {
    pub message_id: String,
    pub subject: String,
    pub from: String,
    pub date: DateTime<Utc>,
    pub total_score: f64,
    pub reasons: Vec<ArchiveReason>,
    pub category: ArchiveCategory,
}

/// Why an email was flagged for archiving
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum ArchiveReason {
    Newsletter { sender: String },
    AutomatedNotification { pattern: String },
    OldThread { age_days: u32 },
    LargeAttachment { size_bytes: u64 },
    ReadNotReplied { age_days: u32 },
    BulkSender { count: u32 },
    NoRecentActivity { last_activity_days: u32 },
}

/// Top-level category for the archive suggestion
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ArchiveCategory {
    Newsletter,
    Automated,
    Stale,
    Bulk,
    LargeAttachment,
    ReadNoAction,
}

// ── Drive Sync ──────────────────────────────────────────────────────

/// Persistent state for a sync mapping (local dir <-> Drive folder)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncState {
    pub sync_id: String,
    pub local_path: String,
    pub drive_folder_id: String,
    pub last_sync: Option<DateTime<Utc>>,
    pub direction: SyncDirection,
    pub file_states: Vec<FileSyncEntry>,
    pub conflict_resolution: ConflictStrategy,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum SyncDirection {
    LocalToDrive,
    DriveToLocal,
    Bidirectional,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ConflictStrategy {
    NewerWins,
    LocalWins,
    DriveWins,
    AskUser,
    KeepBoth,
}

/// Per-file sync tracking
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileSyncEntry {
    pub relative_path: String,
    pub local_hash: Option<String>,
    pub remote_hash: Option<String>,
    pub local_modified: Option<DateTime<Utc>>,
    pub remote_modified: Option<DateTime<Utc>>,
    pub drive_file_id: Option<String>,
    pub status: SyncStatus,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum SyncStatus {
    InSync,
    LocalNewer,
    RemoteNewer,
    Conflict,
    LocalOnly,
    RemoteOnly,
    Deleted,
}

/// Result of a sync operation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncReport {
    pub uploaded: u32,
    pub downloaded: u32,
    pub conflicts: u32,
    pub skipped: u32,
    pub errors: Vec<String>,
    pub duration_ms: u64,
}

/// Individual action the sync engine will take
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SyncAction {
    Upload {
        relative_path: String,
    },
    Download {
        relative_path: String,
        drive_file_id: String,
    },
    Conflict {
        relative_path: String,
        local_modified: DateTime<Utc>,
        remote_modified: DateTime<Utc>,
    },
    Skip {
        relative_path: String,
        reason: String,
    },
    Delete {
        relative_path: String,
        side: String,
    },
}

// ── Drive File Metadata ─────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DriveFileMetadata {
    pub id: String,
    pub name: String,
    pub mime_type: String,
    pub size: Option<u64>,
    pub modified_time: Option<DateTime<Utc>>,
    pub md5_checksum: Option<String>,
    pub parents: Vec<String>,
    pub is_folder: bool,
}

// ── File Intelligence ───────────────────────────────────────────────

/// Suggestion for where a misplaced file should go
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileSuggestion {
    pub file_path: String,
    pub suggested_path: String,
    pub reason: String,
    pub confidence: f64,
    pub personality_message: String,
}

/// Email importance triage result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmailTriage {
    pub message_id: String,
    pub subject: String,
    pub from: String,
    pub importance_score: f64,
    pub reason: String,
    pub action_suggestion: String,
    pub personality_message: String,
}
