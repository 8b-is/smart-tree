use rusqlite::Connection;

pub fn create_tables(conn: &Connection) -> Result<(), rusqlite::Error> {
    conn.execute_batch(
        "
        CREATE TABLE IF NOT EXISTS scanned_files (
            id              INTEGER PRIMARY KEY AUTOINCREMENT,
            file_path       TEXT NOT NULL,
            file_name       TEXT NOT NULL,
            sha256          TEXT NOT NULL,
            file_size       INTEGER NOT NULL,
            scanned_at      TEXT NOT NULL DEFAULT (datetime('now')),
            scan_duration_ms INTEGER,
            recipe_used     TEXT,
            status          TEXT NOT NULL DEFAULT 'completed'
        );

        CREATE INDEX IF NOT EXISTS idx_scanned_files_sha256 ON scanned_files(sha256);
        CREATE INDEX IF NOT EXISTS idx_scanned_files_path ON scanned_files(file_path);

        CREATE TABLE IF NOT EXISTS findings (
            id              INTEGER PRIMARY KEY AUTOINCREMENT,
            scan_id         INTEGER NOT NULL REFERENCES scanned_files(id) ON DELETE CASCADE,
            kind            TEXT NOT NULL,
            severity        TEXT NOT NULL,
            description     TEXT NOT NULL,
            evidence        TEXT,
            byte_offset     INTEGER,
            metadata_json   TEXT,
            created_at      TEXT NOT NULL DEFAULT (datetime('now'))
        );

        CREATE INDEX IF NOT EXISTS idx_findings_scan_id ON findings(scan_id);
        CREATE INDEX IF NOT EXISTS idx_findings_severity ON findings(severity);

        CREATE TABLE IF NOT EXISTS url_blacklist (
            id              INTEGER PRIMARY KEY AUTOINCREMENT,
            url_pattern     TEXT NOT NULL UNIQUE,
            source          TEXT,
            reason          TEXT,
            added_at        TEXT NOT NULL DEFAULT (datetime('now'))
        );

        CREATE INDEX IF NOT EXISTS idx_url_blacklist_pattern ON url_blacklist(url_pattern);

        CREATE TABLE IF NOT EXISTS cert_trust_policy (
            id              INTEGER PRIMARY KEY AUTOINCREMENT,
            match_type      TEXT NOT NULL CHECK(match_type IN ('country', 'org')),
            match_value     TEXT NOT NULL,
            action          TEXT NOT NULL CHECK(action IN ('block', 'warn', 'allow')),
            reason          TEXT,
            added_at        TEXT NOT NULL DEFAULT (datetime('now')),
            UNIQUE(match_type, match_value)
        );

        CREATE INDEX IF NOT EXISTS idx_cert_policy_type ON cert_trust_policy(match_type);

        CREATE TABLE IF NOT EXISTS approved_certs (
            id                  INTEGER PRIMARY KEY AUTOINCREMENT,
            sha256_fingerprint  TEXT NOT NULL UNIQUE,
            subject_cn          TEXT,
            issuer_country      TEXT,
            issuer_org          TEXT,
            approved_by         TEXT NOT NULL DEFAULT 'user',
            approved_at         TEXT NOT NULL DEFAULT (datetime('now')),
            notes               TEXT
        );

        CREATE INDEX IF NOT EXISTS idx_approved_certs_fp ON approved_certs(sha256_fingerprint);

        CREATE TABLE IF NOT EXISTS quarantine (
            id              INTEGER PRIMARY KEY AUTOINCREMENT,
            original_path   TEXT NOT NULL,
            quarantine_path TEXT NOT NULL,
            sha256          TEXT NOT NULL,
            file_size       INTEGER NOT NULL,
            reason          TEXT NOT NULL,
            severity        TEXT NOT NULL,
            quarantined_at  TEXT NOT NULL DEFAULT (datetime('now')),
            status          TEXT NOT NULL DEFAULT 'quarantined'
                CHECK(status IN ('quarantined', 'released', 'deleted')),
            released_at     TEXT,
            scan_id         INTEGER REFERENCES scanned_files(id)
        );

        CREATE INDEX IF NOT EXISTS idx_quarantine_status ON quarantine(status);
        CREATE INDEX IF NOT EXISTS idx_quarantine_sha256 ON quarantine(sha256);

        -- Per-hash action memory: remember what to do when we see a known hash
        CREATE TABLE IF NOT EXISTS hash_actions (
            sha256          TEXT PRIMARY KEY,
            action          TEXT NOT NULL CHECK(action IN ('allow', 'quarantine', 'delete', 'flag')),
            file_name       TEXT,
            reason          TEXT,
            first_seen      TEXT NOT NULL DEFAULT (datetime('now')),
            last_seen       TEXT NOT NULL DEFAULT (datetime('now')),
            times_seen      INTEGER NOT NULL DEFAULT 1,
            max_severity    TEXT,
            auto_apply      INTEGER NOT NULL DEFAULT 1
        );

        -- Track files that have been deleted from disk but we remember
        CREATE TABLE IF NOT EXISTS deleted_files (
            id                  INTEGER PRIMARY KEY AUTOINCREMENT,
            sha256              TEXT NOT NULL,
            file_path           TEXT NOT NULL,
            file_name           TEXT NOT NULL,
            file_size           INTEGER,
            last_scanned        TEXT,
            deleted_detected_at TEXT NOT NULL DEFAULT (datetime('now')),
            total_findings      INTEGER NOT NULL DEFAULT 0,
            max_severity        TEXT,
            last_action         TEXT,
            UNIQUE(sha256, file_path)
        );

        CREATE INDEX IF NOT EXISTS idx_deleted_files_sha256 ON deleted_files(sha256);

        -- User-defined custom blocking rules
        CREATE TABLE IF NOT EXISTS custom_rules (
            id              INTEGER PRIMARY KEY AUTOINCREMENT,
            name            TEXT NOT NULL UNIQUE,
            kind            TEXT NOT NULL CHECK(kind IN ('regex', 'tld', 'company', 'ipfs', 'password_guard')),
            pattern         TEXT NOT NULL,
            severity        TEXT NOT NULL DEFAULT 'high',
            description     TEXT,
            enabled         INTEGER NOT NULL DEFAULT 1,
            created_at      TEXT NOT NULL DEFAULT (datetime('now'))
        );

        CREATE INDEX IF NOT EXISTS idx_custom_rules_kind ON custom_rules(kind);

        PRAGMA foreign_keys = ON;
        ",
    )
}
