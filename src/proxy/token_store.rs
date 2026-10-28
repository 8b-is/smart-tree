//! Secure token storage for proxy OAuth providers.
//!
//! Uses the OS keychain (macOS Keychain / freedesktop Secret Service / Windows
//! Credential Manager) via the `keyring` crate. Falls back to a 0600 file under
//! `~/.st/proxy_tokens/` when no backend is available (headless Linux etc.).

use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

const SERVICE: &str = "smart-tree-proxy";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StoredToken {
    pub access_token: String,
    pub refresh_token: Option<String>,
    pub expires_at: Option<DateTime<Utc>>,
    pub scope: Option<String>,
    pub token_type: Option<String>,
}

impl StoredToken {
    pub fn is_expired(&self) -> bool {
        match self.expires_at {
            Some(exp) => Utc::now() >= exp - chrono::Duration::seconds(30),
            None => false,
        }
    }
}

/// Save a token for `provider` under account `account` (email / user id / "default").
pub fn save(provider: &str, account: &str, token: &StoredToken) -> Result<()> {
    let payload = serde_json::to_string(token)?;
    let entry_user = format!("{}:{}", provider, account);

    match keyring::Entry::new(SERVICE, &entry_user).and_then(|e| e.set_password(&payload)) {
        Ok(()) => Ok(()),
        Err(_) => save_file(provider, account, &payload),
    }
}

pub fn load(provider: &str, account: &str) -> Result<Option<StoredToken>> {
    let entry_user = format!("{}:{}", provider, account);

    if let Ok(entry) = keyring::Entry::new(SERVICE, &entry_user) {
        match entry.get_password() {
            Ok(payload) => return Ok(Some(serde_json::from_str(&payload)?)),
            Err(keyring::Error::NoEntry) => return Ok(None),
            Err(_) => {}
        }
    }
    load_file(provider, account)
}

pub fn delete(provider: &str, account: &str) -> Result<()> {
    let entry_user = format!("{}:{}", provider, account);
    if let Ok(entry) = keyring::Entry::new(SERVICE, &entry_user) {
        let _ = entry.delete_credential();
    }
    let path = file_path(provider, account)?;
    if path.exists() {
        std::fs::remove_file(path)?;
    }
    Ok(())
}

fn fallback_dir() -> Result<PathBuf> {
    let home = std::env::var("HOME").context("HOME not set")?;
    let dir = PathBuf::from(home).join(".st").join("proxy_tokens");
    std::fs::create_dir_all(&dir)?;
    Ok(dir)
}

fn file_path(provider: &str, account: &str) -> Result<PathBuf> {
    let safe = |s: &str| s.replace(['/', '\\', ':'], "_");
    Ok(fallback_dir()?.join(format!("{}__{}.json", safe(provider), safe(account))))
}

fn save_file(provider: &str, account: &str, payload: &str) -> Result<()> {
    let path = file_path(provider, account)?;
    std::fs::write(&path, payload)?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = std::fs::metadata(&path)?.permissions();
        perms.set_mode(0o600);
        std::fs::set_permissions(&path, perms)?;
    }
    Ok(())
}

fn load_file(provider: &str, account: &str) -> Result<Option<StoredToken>> {
    let path = file_path(provider, account)?;
    if !path.exists() {
        return Ok(None);
    }
    let payload = std::fs::read_to_string(path)?;
    Ok(Some(serde_json::from_str(&payload)?))
}
