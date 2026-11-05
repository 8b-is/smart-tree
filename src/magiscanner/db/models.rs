use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScannedFile {
    pub id: i64,
    pub file_path: String,
    pub file_name: String,
    pub sha256: String,
    pub file_size: i64,
    pub scanned_at: String,
    pub scan_duration_ms: Option<i64>,
    pub recipe_used: Option<String>,
    pub status: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FindingRow {
    pub id: i64,
    pub scan_id: i64,
    pub kind: String,
    pub severity: String,
    pub description: String,
    pub evidence: Option<String>,
    pub byte_offset: Option<i64>,
    pub metadata_json: Option<String>,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlacklistEntry {
    pub id: i64,
    pub url_pattern: String,
    pub source: Option<String>,
    pub reason: Option<String>,
    pub added_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CertTrustPolicyRow {
    pub id: i64,
    pub match_type: String,
    pub match_value: String,
    pub action: String,
    pub reason: Option<String>,
    pub added_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApprovedCertRow {
    pub id: i64,
    pub sha256_fingerprint: String,
    pub subject_cn: Option<String>,
    pub issuer_country: Option<String>,
    pub issuer_org: Option<String>,
    pub approved_by: String,
    pub approved_at: String,
    pub notes: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuarantineRow {
    pub id: i64,
    pub original_path: String,
    pub quarantine_path: String,
    pub sha256: String,
    pub file_size: i64,
    pub reason: String,
    pub severity: String,
    pub quarantined_at: String,
    pub status: String,
    pub released_at: Option<String>,
    pub scan_id: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HashActionRow {
    pub sha256: String,
    pub action: String,
    pub file_name: Option<String>,
    pub reason: Option<String>,
    pub first_seen: String,
    pub last_seen: String,
    pub times_seen: i64,
    pub max_severity: Option<String>,
    pub auto_apply: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeletedFileRow {
    pub id: i64,
    pub sha256: String,
    pub file_path: String,
    pub file_name: String,
    pub file_size: Option<i64>,
    pub last_scanned: Option<String>,
    pub deleted_detected_at: String,
    pub total_findings: i64,
    pub max_severity: Option<String>,
    pub last_action: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CustomRuleRow {
    pub id: i64,
    pub name: String,
    pub kind: String,
    pub pattern: String,
    pub severity: String,
    pub description: Option<String>,
    pub enabled: bool,
    pub created_at: String,
}
