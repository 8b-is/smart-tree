//! Explicit, bounded certificate inventory scans. Files are never modified.

use anyhow::{bail, Context, Result};
use serde::{Deserialize, Serialize};
use std::path::Path;
use walkdir::WalkDir;

use super::certificates::CertificateInspection;
use super::config::SecurityConfig;
use super::db::Database;
use super::scanner::read_regular_file;
use super::service::build_certificate_analyzer;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CertificateFileReport {
    pub path: String,
    pub sha256: String,
    pub inspection: CertificateInspection,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkippedEntry {
    pub path: String,
    pub reason: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CertificateScanResult {
    pub scanned_at: i64,
    pub files_scanned: usize,
    /// Includes files with certificates or malformed PEM blocks.
    pub files: Vec<CertificateFileReport>,
    pub skipped: Vec<SkippedEntry>,
}

pub fn scan_certificates(
    config: &SecurityConfig,
    path: &Path,
    recursive: bool,
) -> Result<CertificateScanResult> {
    use sha2::{Digest, Sha256};

    let metadata = std::fs::symlink_metadata(path)
        .with_context(|| format!("Cannot access {}", path.display()))?;
    if !metadata.is_dir() && !metadata.is_file() && !metadata.is_symlink() {
        bail!("Certificate scan requires a regular file or directory");
    }
    let db = Database::open(&config.db_path()).context("Failed to open security database")?;
    let analyzer = build_certificate_analyzer(config, &db)?;
    let mut result = CertificateScanResult {
        scanned_at: chrono::Utc::now().timestamp(),
        files_scanned: 0,
        files: Vec::new(),
        skipped: Vec::new(),
    };

    let walker = WalkDir::new(path)
        .follow_links(config.scan.follow_symlinks)
        .follow_root_links(config.scan.follow_symlinks)
        .max_depth(if recursive { usize::MAX } else { 1 });
    for entry in walker {
        let entry = match entry {
            Ok(entry) => entry,
            Err(error) => {
                result.skipped.push(SkippedEntry {
                    path: error.path().unwrap_or(path).display().to_string(),
                    reason: error.to_string(),
                });
                continue;
            }
        };
        if entry.file_type().is_dir() {
            continue;
        }
        let file_path = entry.path();
        let raw = match read_regular_file(
            file_path,
            config.scan.max_file_size_mb.saturating_mul(1024 * 1024),
            config.scan.follow_symlinks,
        ) {
            Ok(raw) => raw,
            Err(error) => {
                result.skipped.push(SkippedEntry {
                    path: file_path.display().to_string(),
                    reason: error.to_string(),
                });
                continue;
            }
        };
        result.files_scanned += 1;
        let inspection = analyzer.inspect(&raw);
        if !inspection.certificates.is_empty() || !inspection.issues.is_empty() {
            result.files.push(CertificateFileReport {
                path: file_path.display().to_string(),
                sha256: hex::encode(Sha256::digest(&raw)),
                inspection,
            });
        }
    }
    result.files.sort_by(|a, b| a.path.cmp(&b.path));
    result.skipped.sort_by(|a, b| a.path.cmp(&b.path));
    Ok(result)
}

pub fn print_certificate_scan(result: &CertificateScanResult) {
    let count: usize = result
        .files
        .iter()
        .map(|file| file.inspection.certificates.len())
        .sum();
    println!(
        "{count} certificate(s) in {} scanned file(s)",
        result.files_scanned
    );
    println!("Metadata inspection; signatures, chain trust and revocation are not verified.");
    for file in &result.files {
        println!("\n{}", terminal_text(&file.path));
        for cert in &file.inspection.certificates {
            println!("  Subject: {}", terminal_text(&cert.subject));
            println!("  Issuer: {}", terminal_text(&cert.issuer));
            println!("  Serial: {}", cert.serial);
            println!("  SHA256: {}", cert.fingerprint_sha256);
            println!(
                "  Valid: {} to {} ({})",
                format_timestamp(cert.not_before),
                format_timestamp(cert.not_after),
                cert.validity_at(result.scanned_at)
            );
            println!(
                "  CA: {}  Self-issued: {}  {} at byte {}",
                cert.is_ca, cert.is_self_issued, cert.encoding, cert.offset
            );
            if !cert.subject_alternative_names.is_empty() {
                println!(
                    "  Alternative names: {}",
                    terminal_text(&cert.subject_alternative_names.join(", "))
                );
            }
        }
        for finding in &file.inspection.findings {
            println!(
                "  [{}] {}",
                finding.severity,
                terminal_text(&finding.description)
            );
        }
        for issue in &file.inspection.issues {
            println!("  [parse error at byte {}] {}", issue.offset, issue.message);
        }
    }
    for entry in &result.skipped {
        println!(
            "Skipped {}: {}",
            terminal_text(&entry.path),
            terminal_text(&entry.reason)
        );
    }
}

fn format_timestamp(timestamp: i64) -> String {
    chrono::DateTime::from_timestamp(timestamp, 0)
        .map(|date| date.to_rfc3339())
        .unwrap_or_else(|| timestamp.to_string())
}

fn terminal_text(value: &str) -> String {
    value
        .chars()
        .flat_map(|ch| {
            if ch.is_control() {
                ch.escape_default().collect::<Vec<_>>()
            } else {
                vec![ch]
            }
        })
        .collect()
}
