use sha2::{Digest, Sha256};
use std::path::Path;
use walkdir::WalkDir;
use x509_parser::prelude::*;

/// Information about a system CA certificate.
#[derive(Debug, Clone)]
pub struct SystemCertInfo {
    pub path: String,
    pub subject_cn: String,
    pub issuer_country: String,
    pub issuer_org: String,
    pub not_before: String,
    pub not_after: String,
    pub fingerprint_sha256: String,
    pub is_expired: bool,
    pub is_self_signed: bool,
}

/// Default directories to scan for system CA certificates.
pub const DEFAULT_CERT_DIRS: &[&str] = &[
    "/etc/ca-certificates/extracted/cadir/",
    "/etc/pki/tls/certs/",
    "/usr/share/ca-certificates/",
    "/etc/ssl/certs/",
];

/// Scan system CA certificate directories and parse all found certificates.
pub fn scan_system_certs(extra_dirs: &[String]) -> Result<Vec<SystemCertInfo>, anyhow::Error> {
    let mut certs = Vec::new();
    let mut seen_fingerprints = std::collections::HashSet::new();

    let dirs: Vec<&str> = DEFAULT_CERT_DIRS
        .iter()
        .copied()
        .chain(extra_dirs.iter().map(|s| s.as_str()))
        .collect();

    for dir in dirs {
        let dir_path = Path::new(dir);
        if !dir_path.exists() {
            continue;
        }

        for entry in WalkDir::new(dir_path)
            .follow_links(true)
            .into_iter()
            .flatten()
        {
            if !entry.file_type().is_file() {
                continue;
            }

            let path = entry.path();
            let content = match std::fs::read(path) {
                Ok(c) => c,
                Err(_) => continue,
            };

            // Try PEM first
            let parsed = parse_pem_certs(&content, path);
            if !parsed.is_empty() {
                for cert in parsed {
                    if seen_fingerprints.insert(cert.fingerprint_sha256.clone()) {
                        certs.push(cert);
                    }
                }
                continue;
            }

            // Try DER
            if let Some(cert) = parse_der_cert(&content, path) {
                if seen_fingerprints.insert(cert.fingerprint_sha256.clone()) {
                    certs.push(cert);
                }
            }
        }
    }

    certs.sort_by(|a, b| a.issuer_country.cmp(&b.issuer_country));
    Ok(certs)
}

fn parse_pem_certs(content: &[u8], path: &Path) -> Vec<SystemCertInfo> {
    let mut certs = Vec::new();
    let text = match std::str::from_utf8(content) {
        Ok(t) => t,
        Err(_) => return certs,
    };

    for pem in Pem::iter_from_buffer(text.as_bytes()) {
        let pem = match pem {
            Ok(p) => p,
            Err(_) => continue,
        };
        if pem.label != "CERTIFICATE" {
            continue;
        }
        if let Some(info) = der_to_cert_info(&pem.contents, path) {
            certs.push(info);
        }
    }

    certs
}

fn parse_der_cert(content: &[u8], path: &Path) -> Option<SystemCertInfo> {
    der_to_cert_info(content, path)
}

fn der_to_cert_info(der: &[u8], path: &Path) -> Option<SystemCertInfo> {
    let (_, cert) = X509Certificate::from_der(der).ok()?;

    let fingerprint = format!("{:x}", Sha256::digest(der));

    let subject_cn = cert
        .subject()
        .iter_common_name()
        .next()
        .and_then(|cn| cn.as_str().ok())
        .unwrap_or("unknown")
        .to_string();

    let issuer_country = cert
        .issuer()
        .iter_country()
        .next()
        .and_then(|c| c.as_str().ok())
        .unwrap_or("")
        .to_string();

    let issuer_org = cert
        .issuer()
        .iter_organization()
        .next()
        .and_then(|o| o.as_str().ok())
        .unwrap_or("")
        .to_string();

    let not_before = cert.validity().not_before.to_rfc2822().unwrap_or_default();
    let not_after = cert.validity().not_after.to_rfc2822().unwrap_or_default();

    let now_ts = chrono::Utc::now().timestamp();
    let expiry_ts = cert.validity().not_after.to_datetime().unix_timestamp();
    let is_expired = now_ts > expiry_ts;

    let is_self_signed = cert.issuer() == cert.subject();

    Some(SystemCertInfo {
        path: path.display().to_string(),
        subject_cn,
        issuer_country,
        issuer_org,
        not_before,
        not_after,
        fingerprint_sha256: fingerprint,
        is_expired,
        is_self_signed,
    })
}

/// Filter certs that match distrusted countries or orgs.
pub fn audit_system_certs(
    certs: &[SystemCertInfo],
    distrusted_countries: &[String],
    distrusted_orgs: &[String],
) -> Vec<SystemCertInfo> {
    certs
        .iter()
        .filter(|cert| {
            let country_match = distrusted_countries
                .iter()
                .any(|dc| cert.issuer_country.eq_ignore_ascii_case(dc));

            let org_match = distrusted_orgs.iter().any(|pattern| {
                cert.issuer_org
                    .to_lowercase()
                    .contains(&pattern.to_lowercase())
            });

            country_match || org_match
        })
        .cloned()
        .collect()
}

/// Generate a shell script to blacklist flagged certificates.
pub fn generate_blacklist_script(flagged: &[SystemCertInfo]) -> String {
    let mut script = String::new();
    script.push_str("#!/bin/bash\n");
    script.push_str("# MagiSCanner — Certificate blacklist script\n");
    script.push_str("# Review carefully before running!\n");
    script.push_str("# This will distrust the listed CA certificates system-wide.\n\n");
    script.push_str("set -e\n\n");
    script.push_str("BLACKLIST_DIR=\"/etc/pki/ca-trust/source/blacklist\"\n");
    script.push_str("mkdir -p \"$BLACKLIST_DIR\"\n\n");

    for cert in flagged {
        script.push_str(&format!(
            "# {} (Country: {}, Org: {})\n",
            cert.subject_cn, cert.issuer_country, cert.issuer_org
        ));
        script.push_str(&format!("# Fingerprint: {}\n", cert.fingerprint_sha256));
        let safe_name = cert
            .subject_cn
            .replace(' ', "_")
            .replace('/', "_")
            .replace('\\', "_");
        script.push_str(&format!(
            "cp \"{}\" \"$BLACKLIST_DIR/{}.pem\"\n\n",
            cert.path, safe_name
        ));
    }

    script.push_str("# Regenerate trust store\n");
    script.push_str("update-ca-trust extract\n");
    script.push_str("\necho \"Blacklisted {} certificate(s). Trust store updated.\"\n");

    // Fix the placeholder
    script = script.replace(
        "Blacklisted {} certificate(s)",
        &format!("Blacklisted {} certificate(s)", flagged.len()),
    );

    script
}
