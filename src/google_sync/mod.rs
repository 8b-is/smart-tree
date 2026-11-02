//! Google Sync - Gmail backup, Drive sync, and smart file intelligence
//!
//! "Don't worry, your data is safe with me. I live here with you." - Liquid
//!
//! Features:
//! - Gmail backup to Google Drive as .eml files
//! - Bidirectional file sync between local and Drive
//! - Smart warm storage suggestions (newsletters, bulk, old threads)
//! - File misplacement detection and organization suggestions
//! - Encrypted OAuth2 token storage (AES-256-GCM)
//!
//! All data stays local. Backs up to YOUR Drive, not a third-party cloud.

pub mod auth;
pub mod drive_client;
pub mod file_intelligence;
pub mod gmail_client;
pub mod models;
pub mod rate_limiter;
pub mod sync_engine;
pub mod sync_state;
pub mod token_store;
pub mod warm_analyzer;

/// Default storage directory for Google sync state
pub fn google_state_dir() -> std::path::PathBuf {
    dirs::home_dir()
        .unwrap_or_else(|| std::path::PathBuf::from("."))
        .join(".st")
        .join("google")
}
