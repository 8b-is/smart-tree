use sha2::{Digest, Sha256};
use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};

#[derive(Debug, thiserror::Error)]
pub enum QuarantineError {
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
    #[error("file not found: {0}")]
    FileNotFound(String),
    #[error("already quarantined: {0}")]
    AlreadyQuarantined(String),
}

#[derive(Debug, Clone)]
pub struct QuarantineResult {
    pub original_path: String,
    pub quarantine_path: String,
    pub sha256: String,
    pub file_size: u64,
}

/// Quarantine a file: move to quarantine_dir/<sha256>.quarantined, chmod 000.
pub fn quarantine_file(
    file_path: &Path,
    quarantine_dir: &Path,
) -> Result<QuarantineResult, QuarantineError> {
    if !file_path.exists() {
        return Err(QuarantineError::FileNotFound(
            file_path.display().to_string(),
        ));
    }

    // Read and hash file
    let content = fs::read(file_path)?;
    let file_size = content.len() as u64;
    let sha256 = format!("{:x}", Sha256::digest(&content));

    // Create quarantine directory
    fs::create_dir_all(quarantine_dir)?;

    // Build quarantine path
    let original_name = file_path
        .file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_else(|| "unknown".to_string());
    let quarantine_name = format!("{sha256}_{original_name}.quarantined");
    let quarantine_path = quarantine_dir.join(&quarantine_name);

    if quarantine_path.exists() {
        return Err(QuarantineError::AlreadyQuarantined(
            quarantine_path.display().to_string(),
        ));
    }

    // Move file (try rename first, fall back to copy+delete for cross-device)
    if fs::rename(file_path, &quarantine_path).is_err() {
        fs::copy(file_path, &quarantine_path)?;
        fs::remove_file(file_path)?;
    }

    // chmod 000 — no access for anyone
    let permissions = fs::Permissions::from_mode(0o000);
    fs::set_permissions(&quarantine_path, permissions)?;

    // Try to set xattr (best effort, don't fail if not supported)
    set_xattr_best_effort(
        &quarantine_path,
        "user.magiscanner.original_path",
        &file_path.display().to_string(),
    );
    set_xattr_best_effort(
        &quarantine_path,
        "user.magiscanner.quarantined_at",
        &chrono::Utc::now().to_rfc3339(),
    );

    Ok(QuarantineResult {
        original_path: file_path.display().to_string(),
        quarantine_path: quarantine_path.display().to_string(),
        sha256,
        file_size,
    })
}

/// Release a file from quarantine: restore to original path with normal permissions.
pub fn release_file(quarantine_path: &Path, original_path: &Path) -> Result<(), QuarantineError> {
    if !quarantine_path.exists() {
        return Err(QuarantineError::FileNotFound(
            quarantine_path.display().to_string(),
        ));
    }

    // First make readable so we can move it
    let permissions = fs::Permissions::from_mode(0o644);
    fs::set_permissions(quarantine_path, permissions)?;

    // Ensure parent directory exists
    if let Some(parent) = original_path.parent() {
        fs::create_dir_all(parent)?;
    }

    // Move back
    if fs::rename(quarantine_path, original_path).is_err() {
        fs::copy(quarantine_path, original_path)?;
        fs::remove_file(quarantine_path)?;
    }

    // Set reasonable permissions
    let permissions = fs::Permissions::from_mode(0o644);
    fs::set_permissions(original_path, permissions)?;

    Ok(())
}

/// Permanently delete a quarantined file.
pub fn delete_quarantined(quarantine_path: &Path) -> Result<(), QuarantineError> {
    if !quarantine_path.exists() {
        return Err(QuarantineError::FileNotFound(
            quarantine_path.display().to_string(),
        ));
    }

    // Make accessible so we can delete
    let permissions = fs::Permissions::from_mode(0o600);
    fs::set_permissions(quarantine_path, permissions)?;
    fs::remove_file(quarantine_path)?;

    Ok(())
}

/// Resolve a path that may contain ~ for home directory.
pub fn resolve_path(path: &str) -> PathBuf {
    if path.starts_with("~/") {
        if let Some(home) = dirs::home_dir() {
            return home.join(&path[2..]);
        }
    }
    PathBuf::from(path)
}

fn set_xattr_best_effort(path: &Path, name: &str, value: &str) {
    // Use the xattr syscall directly via std::process::Command
    // This is best-effort — if it fails, we don't care
    std::process::Command::new("setfattr")
        .arg("-n")
        .arg(name)
        .arg("-v")
        .arg(value)
        .arg(path)
        .output()
        .ok();
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    #[test]
    fn test_quarantine_and_release() {
        let tmp = std::env::temp_dir().join("magiscanner_test_quarantine");
        let _ = fs::remove_dir_all(&tmp);
        fs::create_dir_all(&tmp).unwrap();

        let test_file = tmp.join("suspicious.exe");
        let quarantine_dir = tmp.join("quarantine");

        // Create test file
        {
            let mut f = fs::File::create(&test_file).unwrap();
            f.write_all(b"definitely not malware").unwrap();
        }
        assert!(test_file.exists());

        // Quarantine it
        let result = quarantine_file(&test_file, &quarantine_dir).unwrap();
        assert!(!test_file.exists(), "original should be moved");
        assert!(
            Path::new(&result.quarantine_path).exists(),
            "quarantine file should exist"
        );

        // Release it
        release_file(Path::new(&result.quarantine_path), &test_file).unwrap();
        assert!(test_file.exists(), "file should be restored");
        assert!(
            !Path::new(&result.quarantine_path).exists(),
            "quarantine file should be gone"
        );

        // Cleanup
        let _ = fs::remove_dir_all(&tmp);
    }

    #[test]
    fn test_quarantine_nonexistent() {
        let result = quarantine_file(
            Path::new("/tmp/does_not_exist_12345"),
            Path::new("/tmp/quarantine_test"),
        );
        assert!(result.is_err());
    }
}
