use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Security sentinel configuration (MagiSCanner capabilities integrated into Smart Tree).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityConfig {
    #[serde(default = "ScanConfig::default")]
    pub scan: ScanConfig,
    #[serde(default = "DatabaseConfig::default")]
    pub database: DatabaseConfig,
    #[serde(default = "WatchConfig::default")]
    pub watch: WatchConfig,
    #[serde(default = "CertificateConfig::default")]
    pub certificates: CertificateConfig,
    #[serde(default = "QuarantineConfig::default")]
    pub quarantine: QuarantineConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScanConfig {
    #[serde(default = "default_recipe")]
    pub default_recipe: Vec<String>,
    #[serde(default = "default_max_file_size_mb")]
    pub max_file_size_mb: u64,
    #[serde(default)]
    pub follow_symlinks: bool,
    #[serde(default = "default_true")]
    pub recursive: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabaseConfig {
    #[serde(default = "default_db_path")]
    pub path: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WatchConfig {
    #[serde(default = "default_watch_dirs")]
    pub directories: Vec<String>,
    #[serde(default = "default_poll_interval")]
    pub poll_interval_secs: u64,
}

fn default_recipe() -> Vec<String> {
    vec!["extract_strings".to_string(), "extract_urls".to_string()]
}

fn default_max_file_size_mb() -> u64 {
    100
}

fn default_true() -> bool {
    true
}

fn default_db_path() -> String {
    "~/.st/security.db".to_string()
}

fn default_watch_dirs() -> Vec<String> {
    if let Some(home) = dirs::home_dir() {
        vec![home.join("Downloads").to_string_lossy().to_string()]
    } else {
        vec!["~/Downloads".to_string()]
    }
}

fn default_poll_interval() -> u64 {
    5
}

fn default_cert_dirs() -> Vec<String> {
    vec![
        "/etc/ca-certificates/extracted/cadir/".to_string(),
        "/etc/pki/tls/certs/".to_string(),
        "/usr/share/ca-certificates/".to_string(),
        "/etc/ssl/certs/".to_string(),
    ]
}

fn default_quarantine_severity() -> String {
    "high".to_string()
}

fn default_quarantine_dir() -> String {
    "~/.st/quarantine".to_string()
}

impl Default for ScanConfig {
    fn default() -> Self {
        Self {
            default_recipe: default_recipe(),
            max_file_size_mb: default_max_file_size_mb(),
            follow_symlinks: false,
            recursive: true,
        }
    }
}

impl Default for DatabaseConfig {
    fn default() -> Self {
        Self {
            path: default_db_path(),
        }
    }
}

impl Default for WatchConfig {
    fn default() -> Self {
        Self {
            directories: default_watch_dirs(),
            poll_interval_secs: default_poll_interval(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CertificateConfig {
    #[serde(default)]
    pub distrusted_countries: Vec<String>,
    #[serde(default)]
    pub distrusted_orgs: Vec<String>,
    #[serde(default)]
    pub require_approval: bool,
    #[serde(default = "default_cert_dirs")]
    pub system_cert_dirs: Vec<String>,
    #[serde(default = "default_true")]
    pub enabled: bool,
}

impl Default for CertificateConfig {
    fn default() -> Self {
        Self {
            distrusted_countries: vec![],
            distrusted_orgs: vec![],
            require_approval: false,
            system_cert_dirs: default_cert_dirs(),
            enabled: true,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuarantineConfig {
    #[serde(default)]
    pub enabled: bool,
    #[serde(default = "default_quarantine_severity")]
    pub auto_quarantine_severity: String,
    #[serde(default = "default_quarantine_dir")]
    pub directory: String,
}

impl Default for QuarantineConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            auto_quarantine_severity: default_quarantine_severity(),
            directory: default_quarantine_dir(),
        }
    }
}

impl Default for SecurityConfig {
    fn default() -> Self {
        Self {
            scan: ScanConfig::default(),
            database: DatabaseConfig::default(),
            watch: WatchConfig::default(),
            certificates: CertificateConfig::default(),
            quarantine: QuarantineConfig::default(),
        }
    }
}

impl SecurityConfig {
    /// Resolve the security database path (expand ~ to home dir).
    pub fn db_path(&self) -> PathBuf {
        expand_tilde(&self.database.path)
    }

    /// Resolve the quarantine directory path.
    pub fn quarantine_path(&self) -> PathBuf {
        expand_tilde(&self.quarantine.directory)
    }
}

fn expand_tilde(path: &str) -> PathBuf {
    if let Some(rest) = path.strip_prefix("~/") {
        if let Some(home) = dirs::home_dir() {
            return home.join(rest);
        }
    }
    PathBuf::from(path)
}
