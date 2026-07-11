use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum Severity {
    Info,
    Low,
    Medium,
    High,
    Critical,
}

impl std::fmt::Display for Severity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Info => write!(f, "info"),
            Self::Low => write!(f, "low"),
            Self::Medium => write!(f, "medium"),
            Self::High => write!(f, "high"),
            Self::Critical => write!(f, "critical"),
        }
    }
}

impl std::str::FromStr for Severity {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "info" => Ok(Self::Info),
            "low" => Ok(Self::Low),
            "medium" => Ok(Self::Medium),
            "high" => Ok(Self::High),
            "critical" => Ok(Self::Critical),
            _ => Err(format!("unknown severity: {s}")),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum FindingKind {
    MaliciousUrl {
        url: String,
    },
    LlmInjection {
        pattern: String,
        context: String,
    },
    SuspiciousString {
        value: String,
    },
    Telemetry {
        endpoint: String,
    },
    EncodedPayload {
        layers: Vec<String>,
    },
    UntrustedCertificate {
        subject: String,
        issuer_country: String,
        issuer_org: String,
        fingerprint_sha256: String,
        reason: String,
    },
    ExpiredCertificate {
        subject: String,
        not_after: String,
        fingerprint_sha256: String,
    },
    SelfSignedCertificate {
        subject: String,
        fingerprint_sha256: String,
    },
}

impl FindingKind {
    pub fn kind_str(&self) -> &'static str {
        match self {
            Self::MaliciousUrl { .. } => "malicious_url",
            Self::LlmInjection { .. } => "llm_injection",
            Self::SuspiciousString { .. } => "suspicious_string",
            Self::Telemetry { .. } => "telemetry",
            Self::EncodedPayload { .. } => "encoded_payload",
            Self::UntrustedCertificate { .. } => "untrusted_certificate",
            Self::ExpiredCertificate { .. } => "expired_certificate",
            Self::SelfSignedCertificate { .. } => "self_signed_certificate",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Finding {
    pub kind: FindingKind,
    pub severity: Severity,
    pub description: String,
    pub offset: Option<usize>,
    pub evidence: Option<String>,
}

/// Report from scanning a single file.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScanReport {
    pub file_path: String,
    pub file_name: String,
    pub sha256: String,
    pub file_size: u64,
    pub scan_duration_ms: u64,
    pub findings: Vec<Finding>,
}
