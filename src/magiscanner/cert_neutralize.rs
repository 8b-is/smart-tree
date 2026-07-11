use std::path::Path;

#[derive(Debug, thiserror::Error)]
pub enum NeutralizeError {
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
    #[error("not a certificate file: {0}")]
    NotCert(String),
    #[error("chattr failed: {0}")]
    ChattrFailed(String),
}

#[derive(Debug, Clone)]
pub struct NeutralizeResult {
    pub path: String,
    pub subject: String,
    pub original_size: u64,
    pub was_immutable: bool,
}

/// The stub that replaces the real cert. Looks like a PEM file but contains
/// no valid certificate data. Any parser that tries to use it will fail.
fn neutered_stub(subject: &str, country: &str, org: &str, fingerprint: &str) -> String {
    format!(
        r#"-----BEGIN X509 CERTIFICATE NEUTRALIZED BY MAGISCANNER-----
This certificate has been neutralized by MagiSCanner.
It cannot be used for TLS validation, code signing, or any cryptographic purpose.

Original Subject: {subject}
Issuer Country:   {country}
Issuer Org:       {org}
SHA-256:          {fingerprint}
Neutralized At:   {timestamp}
Reason:           Distrusted by user policy

This file is intentionally kept (not deleted) so package managers
do not attempt to restore it. The immutable flag (chattr +i) prevents
overwriting. To restore, run: magiscanner certs restore <fingerprint>
-----END X509 CERTIFICATE NEUTRALIZED BY MAGISCANNER-----
"#,
        timestamp = chrono::Utc::now().to_rfc3339()
    )
}

/// Neutralize a certificate file: replace content with invalid stub, set immutable.
/// Requires root/sudo for chattr and writing to system cert dirs.
pub fn neutralize_cert(
    cert_path: &Path,
    subject: &str,
    country: &str,
    org: &str,
    fingerprint: &str,
    backup_dir: &Path,
) -> Result<NeutralizeResult, NeutralizeError> {
    if !cert_path.exists() {
        return Err(NeutralizeError::NotCert(cert_path.display().to_string()));
    }

    let original = std::fs::read(cert_path)?;
    let original_size = original.len() as u64;

    // Check if already neutered
    if String::from_utf8_lossy(&original).contains("NEUTRALIZED BY MAGISCANNER") {
        return Err(NeutralizeError::NotCert(format!(
            "{} is already neutralized",
            cert_path.display()
        )));
    }

    // Backup the original cert
    std::fs::create_dir_all(backup_dir)?;
    let backup_name = format!(
        "{}_{}.pem.bak",
        fingerprint.chars().take(16).collect::<String>(),
        cert_path.file_name().unwrap_or_default().to_string_lossy()
    );
    let backup_path = backup_dir.join(&backup_name);
    std::fs::write(&backup_path, &original)?;

    // Remove immutable flag if set (so we can write)
    let was_immutable = remove_immutable(cert_path);

    // Replace with neutered stub
    let stub = neutered_stub(subject, country, org, fingerprint);
    std::fs::write(cert_path, stub)?;

    // Set immutable flag to prevent package updates from restoring it
    set_immutable(cert_path)?;

    Ok(NeutralizeResult {
        path: cert_path.display().to_string(),
        subject: subject.to_string(),
        original_size,
        was_immutable,
    })
}

/// Restore a neutralized certificate from backup.
pub fn restore_cert(
    cert_path: &Path,
    backup_dir: &Path,
    fingerprint: &str,
) -> Result<(), NeutralizeError> {
    // Find the backup
    let backup_prefix = fingerprint.chars().take(16).collect::<String>();
    let backup = std::fs::read_dir(backup_dir)?
        .flatten()
        .find(|e| e.file_name().to_string_lossy().starts_with(&backup_prefix));

    let backup_entry = backup.ok_or_else(|| {
        NeutralizeError::NotCert(format!("No backup found for fingerprint {fingerprint}"))
    })?;

    let original_data = std::fs::read(backup_entry.path())?;

    // Remove immutable flag
    remove_immutable(cert_path);

    // Restore original
    std::fs::write(cert_path, &original_data)?;

    // Remove backup
    std::fs::remove_file(backup_entry.path())?;

    Ok(())
}

fn set_immutable(path: &Path) -> Result<(), NeutralizeError> {
    let output = std::process::Command::new("chattr")
        .arg("+i")
        .arg(path)
        .output();

    match output {
        Ok(o) if o.status.success() => Ok(()),
        Ok(o) => Err(NeutralizeError::ChattrFailed(
            String::from_utf8_lossy(&o.stderr).to_string(),
        )),
        Err(e) => Err(NeutralizeError::ChattrFailed(format!(
            "chattr not found or failed: {e}"
        ))),
    }
}

fn remove_immutable(path: &Path) -> bool {
    std::process::Command::new("chattr")
        .arg("-i")
        .arg(path)
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}
