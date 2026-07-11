//! HTTP API handlers for the security sentinel (mounted by the daemon).

use axum::extract::Path;
use axum::http::StatusCode;
use axum::Json;
use serde::{Deserialize, Serialize};

use crate::config::StConfig;
use crate::magiscanner::service::{
    audit_system_certificates, cert_blacklist_script, scan_path, CertAuditResult,
};
use crate::magiscanner::ScanReport;

#[derive(Debug, Deserialize)]
pub struct SecurityScanRequest {
    pub path: String,
    #[serde(default = "default_true")]
    pub recursive: bool,
    pub recipe: Option<String>,
}

fn default_true() -> bool {
    true
}

#[derive(Debug, Serialize)]
pub struct SecurityScanResponse {
    pub reports: Vec<ScanReport>,
    pub total_findings: usize,
}

#[derive(Debug, Serialize)]
pub struct HashLookupResponse {
    pub found: bool,
    pub action: Option<String>,
    pub times_seen: Option<i64>,
    pub last_seen: Option<String>,
    pub file_name: Option<String>,
    pub max_severity: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct CertAuditResponse {
    pub audit: CertAuditResult,
    pub blacklist_script: Option<String>,
}

fn load_security_config() -> crate::magiscanner::SecurityConfig {
    StConfig::load().map(|c| c.security).unwrap_or_default()
}

/// POST /security/scan — deep integrity scan of a file or directory.
pub async fn security_scan_handler(
    Json(req): Json<SecurityScanRequest>,
) -> Result<Json<SecurityScanResponse>, (StatusCode, String)> {
    let config = load_security_config();
    let path = std::path::Path::new(&req.path);

    let reports = scan_path(&config, path, req.recursive, req.recipe.as_deref())
        .map_err(|e| (StatusCode::BAD_REQUEST, e.to_string()))?;

    let total_findings = reports.iter().map(|r| r.findings.len()).sum();

    Ok(Json(SecurityScanResponse {
        reports,
        total_findings,
    }))
}

/// GET /security/hash/:sha256 — look up a known file hash.
pub async fn hash_lookup_handler(
    Path(sha256): Path<String>,
) -> Result<Json<HashLookupResponse>, (StatusCode, String)> {
    let config = load_security_config();
    let db = crate::magiscanner::Database::open(&config.db_path())
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    match db
        .get_hash_action(&sha256)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
    {
        Some(row) => Ok(Json(HashLookupResponse {
            found: true,
            action: Some(row.action),
            times_seen: Some(row.times_seen),
            last_seen: Some(row.last_seen),
            file_name: row.file_name,
            max_severity: row.max_severity,
        })),
        None => Ok(Json(HashLookupResponse {
            found: false,
            action: None,
            times_seen: None,
            last_seen: None,
            file_name: None,
            max_severity: None,
        })),
    }
}

/// GET /security/certs/audit — audit system CA trust store.
pub async fn cert_audit_handler() -> Result<Json<CertAuditResponse>, (StatusCode, String)> {
    let config = load_security_config();

    let audit = audit_system_certificates(&config)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let blacklist_script = if audit.flagged.is_empty() {
        None
    } else {
        Some(cert_blacklist_script(&audit.flagged))
    };

    Ok(Json(CertAuditResponse {
        audit,
        blacklist_script,
    }))
}
