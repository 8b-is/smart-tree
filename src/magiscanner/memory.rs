//! Latest completed security scans, persisted by the daemon across restarts.

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

use super::{certificate_scan::CertificateScanResult, ScanReport, SecurityConfig};
use crate::mem8::record_store::RecordStore;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ScanData {
    Integrity(Vec<ScanReport>),
    Certificates(CertificateScanResult),
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ScanKind {
    Integrity,
    Certificates,
}

impl ScanKind {
    fn as_str(self) -> &'static str {
        match self {
            Self::Integrity => "integrity",
            Self::Certificates => "certificates",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StoredScan {
    pub path: String,
    pub scanned_at: i64,
    pub recursive: bool,
    pub data: ScanData,
}

pub struct ScanMemory {
    store: RecordStore,
}

impl ScanMemory {
    pub fn open(config: &SecurityConfig) -> Result<Self> {
        Ok(Self {
            store: RecordStore::open(&config.db_path().with_extension("scans.t8"))?,
        })
    }

    pub fn remember(&mut self, path: &Path, recursive: bool, data: ScanData) -> Result<()> {
        let kind = match &data {
            ScanData::Integrity(_) => ScanKind::Integrity,
            ScanData::Certificates(_) => ScanKind::Certificates,
        };
        let path = normalized_path(path)?;
        let scanned_at = match &data {
            ScanData::Certificates(result) => result.scanned_at,
            ScanData::Integrity(_) => chrono::Utc::now().timestamp(),
        };
        self.store.put(
            &memory_key(&path, kind),
            &StoredScan {
                path: path.to_string_lossy().into_owned(),
                scanned_at,
                recursive,
                data,
            },
        )
    }

    /// Historical data only: callers must explicitly request a fresh scan.
    pub fn recall(&mut self, path: &Path, kind: ScanKind) -> Result<Option<StoredScan>> {
        self.store.get(&memory_key(&normalized_path(path)?, kind))
    }
}

fn normalized_path(path: &Path) -> Result<PathBuf> {
    let absolute = std::path::absolute(path)?;
    let mut ancestor = absolute.as_path();
    let mut suffix = Vec::new();
    loop {
        if let Ok(mut resolved) = ancestor.canonicalize() {
            for name in suffix.iter().rev() {
                resolved.push(name);
            }
            return Ok(resolved);
        }
        match (ancestor.parent(), ancestor.file_name()) {
            (Some(parent), Some(name)) => {
                suffix.push(name.to_os_string());
                ancestor = parent;
            }
            _ => return Ok(absolute),
        }
    }
}

fn memory_key(path: &Path, kind: ScanKind) -> String {
    // Preserve non-UTF-8 filenames rather than collapsing them to replacement characters.
    format!(
        "{}:{}",
        kind.as_str(),
        hex::encode(path.as_os_str().as_encoded_bytes())
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::magiscanner::certificate_scan::CertificateFileReport;
    use crate::magiscanner::certificates::CertificateInspection;

    #[test]
    fn scan_kinds_roundtrip_independently_across_restart() {
        let temp = tempfile::tempdir().unwrap();
        let mut config = SecurityConfig::default();
        config.database.path = temp.path().join("security.db").display().to_string();
        let path = temp.path().join("target");
        std::fs::write(&path, "fixture").unwrap();
        let mut memory = ScanMemory::open(&config).unwrap();
        memory
            .remember(&path, true, ScanData::Integrity(vec![]))
            .unwrap();
        memory
            .remember(
                &path,
                false,
                ScanData::Certificates(CertificateScanResult {
                    scanned_at: 12345,
                    files_scanned: 1,
                    skipped: vec![],
                    files: vec![CertificateFileReport {
                        path: path.display().to_string(),
                        sha256: "abcd".to_string(),
                        inspection: CertificateInspection::default(),
                    }],
                }),
            )
            .unwrap();
        drop(memory);
        let mut memory = ScanMemory::open(&config).unwrap();
        assert!(matches!(
            memory
                .recall(&path, ScanKind::Integrity)
                .unwrap()
                .unwrap()
                .data,
            ScanData::Integrity(_)
        ));
        let restored = memory
            .recall(&path, ScanKind::Certificates)
            .unwrap()
            .unwrap();
        assert_eq!(restored.scanned_at, 12345);
        assert!(!restored.recursive);
        let ScanData::Certificates(result) = restored.data else {
            panic!("wrong scan kind")
        };
        assert_eq!(result.files_scanned, 1);
        assert_eq!(result.files[0].sha256, "abcd");
        std::fs::remove_file(&path).unwrap();
        assert!(memory
            .recall(&path, ScanKind::Certificates)
            .unwrap()
            .is_some());
    }
}
