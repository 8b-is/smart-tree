use rusqlite::params;

use super::models::{
    ApprovedCertRow, BlacklistEntry, CertTrustPolicyRow, CustomRuleRow, DeletedFileRow, FindingRow,
    HashActionRow, QuarantineRow, ScannedFile,
};
use super::Database;
use crate::magiscanner::finding::ScanReport;

impl Database {
    // ── Scan results ──

    /// Insert a scan report and all its findings. Returns the scan ID.
    pub fn insert_scan(&self, report: &ScanReport) -> Result<i64, rusqlite::Error> {
        self.conn.execute(
            "INSERT INTO scanned_files (file_path, file_name, sha256, file_size, scan_duration_ms, status)
             VALUES (?1, ?2, ?3, ?4, ?5, 'completed')",
            params![
                report.file_path,
                report.file_name,
                report.sha256,
                report.file_size as i64,
                report.scan_duration_ms as i64,
            ],
        )?;
        let scan_id = self.conn.last_insert_rowid();

        for finding in &report.findings {
            let metadata = serde_json::to_string(&finding.kind).unwrap_or_default();
            self.conn.execute(
                "INSERT INTO findings (scan_id, kind, severity, description, evidence, byte_offset, metadata_json)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
                params![
                    scan_id,
                    finding.kind.kind_str(),
                    finding.severity.to_string(),
                    finding.description,
                    finding.evidence,
                    finding.offset.map(|o| o as i64),
                    metadata,
                ],
            )?;
        }

        Ok(scan_id)
    }

    /// Query scan history with optional filters.
    pub fn query_history(
        &self,
        limit: usize,
        file_filter: Option<&str>,
        severity_filter: Option<&str>,
    ) -> Result<Vec<(ScannedFile, Vec<FindingRow>)>, rusqlite::Error> {
        let mut query = String::from(
            "SELECT id, file_path, file_name, sha256, file_size, scanned_at, scan_duration_ms, recipe_used, status
             FROM scanned_files"
        );
        let mut conditions = Vec::new();
        let mut param_values: Vec<Box<dyn rusqlite::types::ToSql>> = Vec::new();

        if let Some(file) = file_filter {
            conditions.push(format!("file_path LIKE ?{}", param_values.len() + 1));
            param_values.push(Box::new(format!("%{file}%")));
        }

        if let Some(severity) = severity_filter {
            conditions.push(format!(
                "id IN (SELECT scan_id FROM findings WHERE severity = ?{})",
                param_values.len() + 1
            ));
            param_values.push(Box::new(severity.to_string()));
        }

        if !conditions.is_empty() {
            query.push_str(" WHERE ");
            query.push_str(&conditions.join(" AND "));
        }

        query.push_str(&format!(" ORDER BY scanned_at DESC LIMIT {limit}"));

        let params_refs: Vec<&dyn rusqlite::types::ToSql> =
            param_values.iter().map(|p| p.as_ref()).collect();

        let mut stmt = self.conn.prepare(&query)?;
        let scans: Vec<ScannedFile> = stmt
            .query_map(params_refs.as_slice(), |row| {
                Ok(ScannedFile {
                    id: row.get(0)?,
                    file_path: row.get(1)?,
                    file_name: row.get(2)?,
                    sha256: row.get(3)?,
                    file_size: row.get(4)?,
                    scanned_at: row.get(5)?,
                    scan_duration_ms: row.get(6)?,
                    recipe_used: row.get(7)?,
                    status: row.get(8)?,
                })
            })?
            .collect::<Result<Vec<_>, _>>()?;

        let mut results = Vec::new();
        for scan in scans {
            let findings = self.get_findings_for_scan(scan.id)?;
            results.push((scan, findings));
        }

        Ok(results)
    }

    fn get_findings_for_scan(&self, scan_id: i64) -> Result<Vec<FindingRow>, rusqlite::Error> {
        let mut stmt = self.conn.prepare(
            "SELECT id, scan_id, kind, severity, description, evidence, byte_offset, metadata_json, created_at
             FROM findings WHERE scan_id = ?1 ORDER BY id",
        )?;
        let findings = stmt
            .query_map(params![scan_id], |row| {
                Ok(FindingRow {
                    id: row.get(0)?,
                    scan_id: row.get(1)?,
                    kind: row.get(2)?,
                    severity: row.get(3)?,
                    description: row.get(4)?,
                    evidence: row.get(5)?,
                    byte_offset: row.get(6)?,
                    metadata_json: row.get(7)?,
                    created_at: row.get(8)?,
                })
            })?
            .collect::<Result<Vec<_>, _>>()?;
        Ok(findings)
    }

    // ── Blacklist ──

    pub fn add_blacklist_entry(
        &self,
        url_pattern: &str,
        source: Option<&str>,
        reason: Option<&str>,
    ) -> Result<i64, rusqlite::Error> {
        self.conn.execute(
            "INSERT OR IGNORE INTO url_blacklist (url_pattern, source, reason) VALUES (?1, ?2, ?3)",
            params![url_pattern, source, reason],
        )?;
        Ok(self.conn.last_insert_rowid())
    }

    pub fn remove_blacklist_entry(&self, url_pattern: &str) -> Result<bool, rusqlite::Error> {
        let rows = self.conn.execute(
            "DELETE FROM url_blacklist WHERE url_pattern = ?1",
            params![url_pattern],
        )?;
        Ok(rows > 0)
    }

    pub fn list_blacklist(&self) -> Result<Vec<BlacklistEntry>, rusqlite::Error> {
        let mut stmt = self.conn.prepare(
            "SELECT id, url_pattern, source, reason, added_at FROM url_blacklist ORDER BY added_at DESC",
        )?;
        let entries = stmt
            .query_map([], |row| {
                Ok(BlacklistEntry {
                    id: row.get(0)?,
                    url_pattern: row.get(1)?,
                    source: row.get(2)?,
                    reason: row.get(3)?,
                    added_at: row.get(4)?,
                })
            })?
            .collect::<Result<Vec<_>, _>>()?;
        Ok(entries)
    }

    /// Get all blacklist patterns (for use by the URL blacklist analyzer).
    pub fn get_blacklist_patterns(&self) -> Result<Vec<String>, rusqlite::Error> {
        let mut stmt = self.conn.prepare("SELECT url_pattern FROM url_blacklist")?;
        let patterns = stmt
            .query_map([], |row| row.get(0))?
            .collect::<Result<Vec<String>, _>>()?;
        Ok(patterns)
    }
    // ── Certificate Trust Policy ──

    pub fn add_cert_policy(
        &self,
        match_type: &str,
        match_value: &str,
        action: &str,
        reason: Option<&str>,
    ) -> Result<i64, rusqlite::Error> {
        self.conn.execute(
            "INSERT OR REPLACE INTO cert_trust_policy (match_type, match_value, action, reason)
             VALUES (?1, ?2, ?3, ?4)",
            params![match_type, match_value, action, reason],
        )?;
        Ok(self.conn.last_insert_rowid())
    }

    pub fn remove_cert_policy(
        &self,
        match_type: &str,
        match_value: &str,
    ) -> Result<bool, rusqlite::Error> {
        let rows = self.conn.execute(
            "DELETE FROM cert_trust_policy WHERE match_type = ?1 AND match_value = ?2",
            params![match_type, match_value],
        )?;
        Ok(rows > 0)
    }

    pub fn list_cert_policies(&self) -> Result<Vec<CertTrustPolicyRow>, rusqlite::Error> {
        let mut stmt = self.conn.prepare(
            "SELECT id, match_type, match_value, action, reason, added_at
             FROM cert_trust_policy ORDER BY match_type, match_value",
        )?;
        let rows = stmt
            .query_map([], |row| {
                Ok(CertTrustPolicyRow {
                    id: row.get(0)?,
                    match_type: row.get(1)?,
                    match_value: row.get(2)?,
                    action: row.get(3)?,
                    reason: row.get(4)?,
                    added_at: row.get(5)?,
                })
            })?
            .collect::<Result<Vec<_>, _>>()?;
        Ok(rows)
    }

    pub fn get_distrusted_countries(&self) -> Result<Vec<String>, rusqlite::Error> {
        let mut stmt = self.conn.prepare(
            "SELECT match_value FROM cert_trust_policy WHERE match_type = 'country' AND action = 'block'",
        )?;
        let values = stmt
            .query_map([], |row| row.get(0))?
            .collect::<Result<Vec<String>, _>>()?;
        Ok(values)
    }

    pub fn get_distrusted_orgs(&self) -> Result<Vec<String>, rusqlite::Error> {
        let mut stmt = self.conn.prepare(
            "SELECT match_value FROM cert_trust_policy WHERE match_type = 'org' AND action = 'block'",
        )?;
        let values = stmt
            .query_map([], |row| row.get(0))?
            .collect::<Result<Vec<String>, _>>()?;
        Ok(values)
    }

    // ── Approved Certificates ──

    pub fn approve_cert(
        &self,
        fingerprint: &str,
        subject_cn: Option<&str>,
        issuer_country: Option<&str>,
        issuer_org: Option<&str>,
        notes: Option<&str>,
    ) -> Result<i64, rusqlite::Error> {
        self.conn.execute(
            "INSERT OR REPLACE INTO approved_certs (sha256_fingerprint, subject_cn, issuer_country, issuer_org, notes)
             VALUES (?1, ?2, ?3, ?4, ?5)",
            params![fingerprint, subject_cn, issuer_country, issuer_org, notes],
        )?;
        Ok(self.conn.last_insert_rowid())
    }

    pub fn revoke_cert_approval(&self, fingerprint: &str) -> Result<bool, rusqlite::Error> {
        let rows = self.conn.execute(
            "DELETE FROM approved_certs WHERE sha256_fingerprint = ?1",
            params![fingerprint],
        )?;
        Ok(rows > 0)
    }

    pub fn list_approved_certs(&self) -> Result<Vec<ApprovedCertRow>, rusqlite::Error> {
        let mut stmt = self.conn.prepare(
            "SELECT id, sha256_fingerprint, subject_cn, issuer_country, issuer_org, approved_by, approved_at, notes
             FROM approved_certs ORDER BY approved_at DESC",
        )?;
        let rows = stmt
            .query_map([], |row| {
                Ok(ApprovedCertRow {
                    id: row.get(0)?,
                    sha256_fingerprint: row.get(1)?,
                    subject_cn: row.get(2)?,
                    issuer_country: row.get(3)?,
                    issuer_org: row.get(4)?,
                    approved_by: row.get(5)?,
                    approved_at: row.get(6)?,
                    notes: row.get(7)?,
                })
            })?
            .collect::<Result<Vec<_>, _>>()?;
        Ok(rows)
    }

    pub fn get_approved_fingerprints(&self) -> Result<Vec<String>, rusqlite::Error> {
        let mut stmt = self
            .conn
            .prepare("SELECT sha256_fingerprint FROM approved_certs")?;
        let fps = stmt
            .query_map([], |row| row.get(0))?
            .collect::<Result<Vec<String>, _>>()?;
        Ok(fps)
    }

    // ── Quarantine ──

    pub fn insert_quarantine(
        &self,
        original_path: &str,
        quarantine_path: &str,
        sha256: &str,
        file_size: i64,
        reason: &str,
        severity: &str,
        scan_id: Option<i64>,
    ) -> Result<i64, rusqlite::Error> {
        self.conn.execute(
            "INSERT INTO quarantine (original_path, quarantine_path, sha256, file_size, reason, severity, scan_id)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            params![original_path, quarantine_path, sha256, file_size, reason, severity, scan_id],
        )?;
        Ok(self.conn.last_insert_rowid())
    }

    pub fn list_quarantine(
        &self,
        status_filter: Option<&str>,
    ) -> Result<Vec<QuarantineRow>, rusqlite::Error> {
        let (query, params_vec): (String, Vec<Box<dyn rusqlite::types::ToSql>>) = if let Some(
            status,
        ) =
            status_filter
        {
            (
                    "SELECT id, original_path, quarantine_path, sha256, file_size, reason, severity, quarantined_at, status, released_at, scan_id
                     FROM quarantine WHERE status = ?1 ORDER BY quarantined_at DESC".to_string(),
                    vec![Box::new(status.to_string())],
                )
        } else {
            (
                    "SELECT id, original_path, quarantine_path, sha256, file_size, reason, severity, quarantined_at, status, released_at, scan_id
                     FROM quarantine ORDER BY quarantined_at DESC".to_string(),
                    vec![],
                )
        };

        let params_refs: Vec<&dyn rusqlite::types::ToSql> =
            params_vec.iter().map(|p| p.as_ref()).collect();

        let mut stmt = self.conn.prepare(&query)?;
        let rows = stmt
            .query_map(params_refs.as_slice(), |row| {
                Ok(QuarantineRow {
                    id: row.get(0)?,
                    original_path: row.get(1)?,
                    quarantine_path: row.get(2)?,
                    sha256: row.get(3)?,
                    file_size: row.get(4)?,
                    reason: row.get(5)?,
                    severity: row.get(6)?,
                    quarantined_at: row.get(7)?,
                    status: row.get(8)?,
                    released_at: row.get(9)?,
                    scan_id: row.get(10)?,
                })
            })?
            .collect::<Result<Vec<_>, _>>()?;
        Ok(rows)
    }

    pub fn get_quarantine_by_id(&self, id: i64) -> Result<Option<QuarantineRow>, rusqlite::Error> {
        let mut stmt = self.conn.prepare(
            "SELECT id, original_path, quarantine_path, sha256, file_size, reason, severity, quarantined_at, status, released_at, scan_id
             FROM quarantine WHERE id = ?1",
        )?;
        let mut rows = stmt.query_map(params![id], |row| {
            Ok(QuarantineRow {
                id: row.get(0)?,
                original_path: row.get(1)?,
                quarantine_path: row.get(2)?,
                sha256: row.get(3)?,
                file_size: row.get(4)?,
                reason: row.get(5)?,
                severity: row.get(6)?,
                quarantined_at: row.get(7)?,
                status: row.get(8)?,
                released_at: row.get(9)?,
                scan_id: row.get(10)?,
            })
        })?;
        match rows.next() {
            Some(Ok(row)) => Ok(Some(row)),
            Some(Err(e)) => Err(e),
            None => Ok(None),
        }
    }

    pub fn update_quarantine_status(&self, id: i64, status: &str) -> Result<bool, rusqlite::Error> {
        let released_at = if status == "released" {
            Some(chrono::Utc::now().to_rfc3339())
        } else {
            None
        };
        let rows = self.conn.execute(
            "UPDATE quarantine SET status = ?1, released_at = ?2 WHERE id = ?3",
            params![status, released_at, id],
        )?;
        Ok(rows > 0)
    }

    // ── Hash Actions (per-hash memory) ──

    /// Set or update the action for a known file hash.
    /// If the hash already exists, updates last_seen, times_seen, and optionally the action.
    pub fn set_hash_action(
        &self,
        sha256: &str,
        action: &str,
        file_name: Option<&str>,
        reason: Option<&str>,
        max_severity: Option<&str>,
    ) -> Result<(), rusqlite::Error> {
        self.conn.execute(
            "INSERT INTO hash_actions (sha256, action, file_name, reason, max_severity)
             VALUES (?1, ?2, ?3, ?4, ?5)
             ON CONFLICT(sha256) DO UPDATE SET
                action = ?2,
                file_name = COALESCE(?3, file_name),
                reason = COALESCE(?4, reason),
                last_seen = datetime('now'),
                times_seen = times_seen + 1,
                max_severity = COALESCE(?5, max_severity)",
            params![sha256, action, file_name, reason, max_severity],
        )?;
        Ok(())
    }

    /// Look up what action to take for a known hash. Returns None if unseen.
    pub fn get_hash_action(&self, sha256: &str) -> Result<Option<HashActionRow>, rusqlite::Error> {
        let mut stmt = self.conn.prepare(
            "SELECT sha256, action, file_name, reason, first_seen, last_seen, times_seen, max_severity, auto_apply
             FROM hash_actions WHERE sha256 = ?1",
        )?;
        let mut rows = stmt.query_map(params![sha256], |row| {
            Ok(HashActionRow {
                sha256: row.get(0)?,
                action: row.get(1)?,
                file_name: row.get(2)?,
                reason: row.get(3)?,
                first_seen: row.get(4)?,
                last_seen: row.get(5)?,
                times_seen: row.get(6)?,
                max_severity: row.get(7)?,
                auto_apply: row.get(8)?,
            })
        })?;
        match rows.next() {
            Some(Ok(row)) => Ok(Some(row)),
            Some(Err(e)) => Err(e),
            None => Ok(None),
        }
    }

    /// Bump the times_seen and last_seen for a hash without changing the action.
    pub fn touch_hash(&self, sha256: &str) -> Result<bool, rusqlite::Error> {
        let rows = self.conn.execute(
            "UPDATE hash_actions SET last_seen = datetime('now'), times_seen = times_seen + 1
             WHERE sha256 = ?1",
            params![sha256],
        )?;
        Ok(rows > 0)
    }

    pub fn list_hash_actions(&self, limit: usize) -> Result<Vec<HashActionRow>, rusqlite::Error> {
        let mut stmt = self.conn.prepare(
            &format!(
                "SELECT sha256, action, file_name, reason, first_seen, last_seen, times_seen, max_severity, auto_apply
                 FROM hash_actions ORDER BY last_seen DESC LIMIT {limit}"
            ),
        )?;
        let rows = stmt
            .query_map([], |row| {
                Ok(HashActionRow {
                    sha256: row.get(0)?,
                    action: row.get(1)?,
                    file_name: row.get(2)?,
                    reason: row.get(3)?,
                    first_seen: row.get(4)?,
                    last_seen: row.get(5)?,
                    times_seen: row.get(6)?,
                    max_severity: row.get(7)?,
                    auto_apply: row.get(8)?,
                })
            })?
            .collect::<Result<Vec<_>, _>>()?;
        Ok(rows)
    }

    pub fn remove_hash_action(&self, sha256: &str) -> Result<bool, rusqlite::Error> {
        let rows = self.conn.execute(
            "DELETE FROM hash_actions WHERE sha256 = ?1",
            params![sha256],
        )?;
        Ok(rows > 0)
    }

    // ── Deleted Files ──

    /// Record a file that has been deleted from disk.
    pub fn record_deleted_file(
        &self,
        sha256: &str,
        file_path: &str,
        file_name: &str,
        file_size: Option<i64>,
        last_scanned: Option<&str>,
        total_findings: i64,
        max_severity: Option<&str>,
        last_action: Option<&str>,
    ) -> Result<i64, rusqlite::Error> {
        self.conn.execute(
            "INSERT OR REPLACE INTO deleted_files
             (sha256, file_path, file_name, file_size, last_scanned, total_findings, max_severity, last_action)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
            params![sha256, file_path, file_name, file_size, last_scanned, total_findings, max_severity, last_action],
        )?;
        Ok(self.conn.last_insert_rowid())
    }

    /// Check if a hash was previously seen in a deleted file.
    pub fn check_deleted_hash(&self, sha256: &str) -> Result<Vec<DeletedFileRow>, rusqlite::Error> {
        let mut stmt = self.conn.prepare(
            "SELECT id, sha256, file_path, file_name, file_size, last_scanned,
                    deleted_detected_at, total_findings, max_severity, last_action
             FROM deleted_files WHERE sha256 = ?1",
        )?;
        let rows = stmt
            .query_map(params![sha256], |row| {
                Ok(DeletedFileRow {
                    id: row.get(0)?,
                    sha256: row.get(1)?,
                    file_path: row.get(2)?,
                    file_name: row.get(3)?,
                    file_size: row.get(4)?,
                    last_scanned: row.get(5)?,
                    deleted_detected_at: row.get(6)?,
                    total_findings: row.get(7)?,
                    max_severity: row.get(8)?,
                    last_action: row.get(9)?,
                })
            })?
            .collect::<Result<Vec<_>, _>>()?;
        Ok(rows)
    }

    pub fn list_deleted_files(&self, limit: usize) -> Result<Vec<DeletedFileRow>, rusqlite::Error> {
        let mut stmt = self.conn.prepare(&format!(
            "SELECT id, sha256, file_path, file_name, file_size, last_scanned,
                        deleted_detected_at, total_findings, max_severity, last_action
                 FROM deleted_files ORDER BY deleted_detected_at DESC LIMIT {limit}"
        ))?;
        let rows = stmt
            .query_map([], |row| {
                Ok(DeletedFileRow {
                    id: row.get(0)?,
                    sha256: row.get(1)?,
                    file_path: row.get(2)?,
                    file_name: row.get(3)?,
                    file_size: row.get(4)?,
                    last_scanned: row.get(5)?,
                    deleted_detected_at: row.get(6)?,
                    total_findings: row.get(7)?,
                    max_severity: row.get(8)?,
                    last_action: row.get(9)?,
                })
            })?
            .collect::<Result<Vec<_>, _>>()?;
        Ok(rows)
    }

    // ── Pruning ──

    /// Prune scan history older than `days` days.
    /// Moves deleted-from-disk files to deleted_files table first,
    /// then removes old scans and their findings.
    /// Returns (scans_archived, scans_pruned, findings_pruned).
    pub fn prune(&self, days: u32) -> Result<(usize, usize, usize), rusqlite::Error> {
        let cutoff = format!("datetime('now', '-{days} days')");

        // First, archive any scanned files whose original path no longer exists
        let scans_to_archive: Vec<(i64, String, String, String, i64, String, i64)> = {
            let mut stmt = self.conn.prepare(&format!(
                "SELECT s.id, s.sha256, s.file_path, s.file_name, s.file_size, s.scanned_at,
                            (SELECT COUNT(*) FROM findings f WHERE f.scan_id = s.id)
                     FROM scanned_files s
                     WHERE s.scanned_at < {cutoff}"
            ))?;
            stmt.query_map([], |row| {
                Ok((
                    row.get::<_, i64>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, String>(2)?,
                    row.get::<_, String>(3)?,
                    row.get::<_, i64>(4)?,
                    row.get::<_, String>(5)?,
                    row.get::<_, i64>(6)?,
                ))
            })?
            .collect::<Result<Vec<_>, _>>()?
        };

        let mut archived = 0;
        for (scan_id, sha256, file_path, file_name, file_size, scanned_at, finding_count) in
            &scans_to_archive
        {
            // Check if file still exists on disk
            if !std::path::Path::new(file_path).exists() {
                // Get max severity for this scan
                let max_sev: Option<String> = self
                    .conn
                    .query_row(
                        "SELECT severity FROM findings WHERE scan_id = ?1
                     ORDER BY CASE severity WHEN 'critical' THEN 1 WHEN 'high' THEN 2
                     WHEN 'medium' THEN 3 WHEN 'low' THEN 4 WHEN 'info' THEN 5 END
                     LIMIT 1",
                        params![scan_id],
                        |row| row.get(0),
                    )
                    .ok();

                // Get last hash action
                let last_action: Option<String> = self
                    .conn
                    .query_row(
                        "SELECT action FROM hash_actions WHERE sha256 = ?1",
                        params![sha256],
                        |row| row.get(0),
                    )
                    .ok();

                self.record_deleted_file(
                    sha256,
                    file_path,
                    file_name,
                    Some(*file_size),
                    Some(scanned_at),
                    *finding_count,
                    max_sev.as_deref(),
                    last_action.as_deref(),
                )?;
                archived += 1;
            }
        }

        // Delete old findings (CASCADE would handle this, but let's be explicit and count)
        let findings_pruned = self.conn.execute(
            &format!(
                "DELETE FROM findings WHERE scan_id IN
                 (SELECT id FROM scanned_files WHERE scanned_at < {cutoff})"
            ),
            [],
        )?;

        // Delete old scans
        let scans_pruned = self.conn.execute(
            &format!("DELETE FROM scanned_files WHERE scanned_at < {cutoff}"),
            [],
        )?;

        Ok((archived, scans_pruned, findings_pruned))
    }

    /// Get database stats.
    pub fn stats(&self) -> Result<DbStats, rusqlite::Error> {
        let scan_count: i64 =
            self.conn
                .query_row("SELECT COUNT(*) FROM scanned_files", [], |r| r.get(0))?;
        let finding_count: i64 = self
            .conn
            .query_row("SELECT COUNT(*) FROM findings", [], |r| r.get(0))?;
        let hash_action_count: i64 =
            self.conn
                .query_row("SELECT COUNT(*) FROM hash_actions", [], |r| r.get(0))?;
        let deleted_count: i64 =
            self.conn
                .query_row("SELECT COUNT(*) FROM deleted_files", [], |r| r.get(0))?;
        let blacklist_count: i64 =
            self.conn
                .query_row("SELECT COUNT(*) FROM url_blacklist", [], |r| r.get(0))?;
        let quarantine_count: i64 = self.conn.query_row(
            "SELECT COUNT(*) FROM quarantine WHERE status = 'quarantined'",
            [],
            |r| r.get(0),
        )?;
        let oldest_scan: Option<String> = self
            .conn
            .query_row("SELECT MIN(scanned_at) FROM scanned_files", [], |r| {
                r.get(0)
            })
            .ok();
        Ok(DbStats {
            scan_count,
            finding_count,
            hash_action_count,
            deleted_count,
            blacklist_count,
            quarantine_count,
            oldest_scan,
        })
    }

    // ── Custom Rules ──

    pub fn add_custom_rule(
        &self,
        name: &str,
        kind: &str,
        pattern: &str,
        severity: &str,
        description: Option<&str>,
    ) -> Result<i64, rusqlite::Error> {
        self.conn.execute(
            "INSERT OR REPLACE INTO custom_rules (name, kind, pattern, severity, description)
             VALUES (?1, ?2, ?3, ?4, ?5)",
            params![name, kind, pattern, severity, description],
        )?;
        Ok(self.conn.last_insert_rowid())
    }

    pub fn remove_custom_rule(&self, name: &str) -> Result<bool, rusqlite::Error> {
        let rows = self
            .conn
            .execute("DELETE FROM custom_rules WHERE name = ?1", params![name])?;
        Ok(rows > 0)
    }

    pub fn list_custom_rules(&self) -> Result<Vec<CustomRuleRow>, rusqlite::Error> {
        let mut stmt = self.conn.prepare(
            "SELECT id, name, kind, pattern, severity, description, enabled, created_at
             FROM custom_rules ORDER BY kind, name",
        )?;
        let rows = stmt
            .query_map([], |row| {
                Ok(CustomRuleRow {
                    id: row.get(0)?,
                    name: row.get(1)?,
                    kind: row.get(2)?,
                    pattern: row.get(3)?,
                    severity: row.get(4)?,
                    description: row.get(5)?,
                    enabled: row.get(6)?,
                    created_at: row.get(7)?,
                })
            })?
            .collect::<Result<Vec<_>, _>>()?;
        Ok(rows)
    }

    pub fn get_enabled_custom_rules(&self) -> Result<Vec<CustomRuleRow>, rusqlite::Error> {
        let mut stmt = self.conn.prepare(
            "SELECT id, name, kind, pattern, severity, description, enabled, created_at
             FROM custom_rules WHERE enabled = 1 ORDER BY kind, name",
        )?;
        let rows = stmt
            .query_map([], |row| {
                Ok(CustomRuleRow {
                    id: row.get(0)?,
                    name: row.get(1)?,
                    kind: row.get(2)?,
                    pattern: row.get(3)?,
                    severity: row.get(4)?,
                    description: row.get(5)?,
                    enabled: row.get(6)?,
                    created_at: row.get(7)?,
                })
            })?
            .collect::<Result<Vec<_>, _>>()?;
        Ok(rows)
    }
}

pub struct DbStats {
    pub scan_count: i64,
    pub finding_count: i64,
    pub hash_action_count: i64,
    pub deleted_count: i64,
    pub blacklist_count: i64,
    pub quarantine_count: i64,
    pub oldest_scan: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::magiscanner::finding::{Finding, FindingKind, Severity};

    #[test]
    fn test_insert_and_query_scan() {
        let db = Database::open_in_memory().unwrap();
        let report = ScanReport {
            file_path: "/tmp/test.bin".to_string(),
            file_name: "test.bin".to_string(),
            sha256: "abcdef1234567890".to_string(),
            file_size: 1024,
            scan_duration_ms: 42,
            findings: vec![Finding {
                kind: FindingKind::MaliciousUrl {
                    url: "https://evil.com".to_string(),
                },
                severity: Severity::High,
                description: "Bad URL found".to_string(),
                offset: Some(100),
                evidence: Some("https://evil.com".to_string()),
            }],
        };

        let scan_id = db.insert_scan(&report).unwrap();
        assert!(scan_id > 0);

        let history = db.query_history(10, None, None).unwrap();
        assert_eq!(history.len(), 1);
        assert_eq!(history[0].0.file_name, "test.bin");
        assert_eq!(history[0].1.len(), 1);
        assert_eq!(history[0].1[0].kind, "malicious_url");
    }

    #[test]
    fn test_blacklist_crud() {
        let db = Database::open_in_memory().unwrap();

        db.add_blacklist_entry("evil.com", Some("manual"), Some("known malware"))
            .unwrap();
        db.add_blacklist_entry("bad.org", None, None).unwrap();

        let entries = db.list_blacklist().unwrap();
        assert_eq!(entries.len(), 2);

        let patterns = db.get_blacklist_patterns().unwrap();
        assert!(patterns.contains(&"evil.com".to_string()));

        let removed = db.remove_blacklist_entry("evil.com").unwrap();
        assert!(removed);

        let entries = db.list_blacklist().unwrap();
        assert_eq!(entries.len(), 1);
    }
}
