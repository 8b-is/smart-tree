//! HTTP API handlers for the security sentinel (mounted by the daemon).

use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::Json;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::RwLock;

use super::certificate_scan::{scan_certificates, CertificateScanResult};
use super::memory::{ScanData, ScanKind, StoredScan};
use crate::config::StConfig;
use crate::daemon::DaemonState;
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

#[derive(Debug, Serialize, Deserialize)]
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
    State(state): State<Arc<RwLock<DaemonState>>>,
    Json(req): Json<SecurityScanRequest>,
) -> Result<Json<SecurityScanResponse>, (StatusCode, String)> {
    let memory = Arc::clone(&state.read().await.scan_memory);
    let reports = tokio::task::spawn_blocking(move || {
        let config = StConfig::load().map_err(internal_error)?.security;
        let path = std::path::Path::new(&req.path);
        let reports = scan_path(&config, path, req.recursive, req.recipe.as_deref())
            .map_err(|e| (StatusCode::BAD_REQUEST, e.to_string()))?;
        memory
            .lock()
            .map_err(internal_error)?
            .remember(path, req.recursive, ScanData::Integrity(reports.clone()))
            .map_err(internal_error)?;
        Ok::<_, (StatusCode, String)>(reports)
    })
    .await
    .map_err(internal_error)??;

    let total_findings = reports.iter().map(|r| r.findings.len()).sum();

    Ok(Json(SecurityScanResponse {
        reports,
        total_findings,
    }))
}

#[derive(Debug, Deserialize)]
pub struct CertificateScanRequest {
    pub path: String,
    #[serde(default = "default_true")]
    pub recursive: bool,
}

/// POST /security/certs/scan — inspect file certificates and retain the result.
pub async fn certificate_scan_handler(
    State(state): State<Arc<RwLock<DaemonState>>>,
    Json(req): Json<CertificateScanRequest>,
) -> Result<Json<CertificateScanResult>, (StatusCode, String)> {
    let memory = Arc::clone(&state.read().await.scan_memory);
    let result = tokio::task::spawn_blocking(move || {
        let config = StConfig::load().map_err(internal_error)?.security;
        let path = std::path::Path::new(&req.path);
        let result = scan_certificates(&config, path, req.recursive)
            .map_err(|e| (StatusCode::BAD_REQUEST, e.to_string()))?;
        memory
            .lock()
            .map_err(internal_error)?
            .remember(path, req.recursive, ScanData::Certificates(result.clone()))
            .map_err(internal_error)?;
        Ok::<_, (StatusCode, String)>(result)
    })
    .await
    .map_err(internal_error)??;
    Ok(Json(result))
}

#[derive(Debug, Deserialize)]
pub struct ScanHistoryRequest {
    pub path: String,
    pub kind: ScanKind,
}

/// GET /security/history?path=...&kind=certificates — latest stored scan.
pub async fn scan_history_handler(
    State(state): State<Arc<RwLock<DaemonState>>>,
    Query(req): Query<ScanHistoryRequest>,
) -> Result<Json<StoredScan>, (StatusCode, String)> {
    let memory = Arc::clone(&state.read().await.scan_memory);
    let result = tokio::task::spawn_blocking(move || {
        memory
            .lock()
            .map_err(internal_error)?
            .recall(std::path::Path::new(&req.path), req.kind)
            .map_err(internal_error)
    })
    .await
    .map_err(internal_error)??;
    result.map(Json).ok_or((
        StatusCode::NOT_FOUND,
        "No stored scan for this path and kind".to_string(),
    ))
}

fn internal_error(error: impl std::fmt::Display) -> (StatusCode, String) {
    (StatusCode::INTERNAL_SERVER_ERROR, error.to_string())
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
