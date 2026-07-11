use regex::Regex;
use sha2::{Digest, Sha256};
use std::sync::LazyLock;
use x509_parser::prelude::*;

use crate::magiscanner::analyzers::{AnalysisContext, Analyzer};
use crate::magiscanner::finding::{Finding, FindingKind, Severity};

static PEM_REGEX: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"-----BEGIN CERTIFICATE-----[\s\S]*?-----END CERTIFICATE-----").unwrap()
});

/// Policy for which certificates to distrust.
#[derive(Debug, Clone)]
pub struct CertDistrust {
    pub country_codes: Vec<String>,
    pub org_patterns: Vec<String>,
    pub require_approval: bool,
    pub approved_fingerprints: Vec<String>,
}

pub struct CertificateAnalyzer {
    distrust: CertDistrust,
}

impl CertificateAnalyzer {
    pub fn new(distrust: CertDistrust) -> Self {
        Self { distrust }
    }

    fn analyze_cert_der(&self, der_bytes: &[u8], source: &str) -> Vec<Finding> {
        let mut findings = Vec::new();

        let cert = match X509Certificate::from_der(der_bytes) {
            Ok((_, cert)) => cert,
            Err(_) => return findings,
        };

        let fingerprint = format!("{:x}", Sha256::digest(der_bytes));

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

        // Check distrusted countries
        if !issuer_country.is_empty() {
            let country_upper = issuer_country.to_uppercase();
            for dc in &self.distrust.country_codes {
                if country_upper == dc.to_uppercase() {
                    findings.push(Finding {
                        kind: FindingKind::UntrustedCertificate {
                            subject: subject_cn.clone(),
                            issuer_country: issuer_country.clone(),
                            issuer_org: issuer_org.clone(),
                            fingerprint_sha256: fingerprint.clone(),
                            reason: format!("Issuer country '{issuer_country}' is distrusted"),
                        },
                        severity: Severity::Critical,
                        description: format!(
                            "Certificate from distrusted country {issuer_country}: {subject_cn} ({source})"
                        ),
                        offset: None,
                        evidence: Some(format!("C={issuer_country}, O={issuer_org}, CN={subject_cn}")),
                    });
                    break;
                }
            }
        }

        // Check distrusted organizations
        let org_lower = issuer_org.to_lowercase();
        for pattern in &self.distrust.org_patterns {
            if org_lower.contains(&pattern.to_lowercase()) {
                findings.push(Finding {
                    kind: FindingKind::UntrustedCertificate {
                        subject: subject_cn.clone(),
                        issuer_country: issuer_country.clone(),
                        issuer_org: issuer_org.clone(),
                        fingerprint_sha256: fingerprint.clone(),
                        reason: format!("Issuer org matches distrusted pattern '{pattern}'"),
                    },
                    severity: Severity::Critical,
                    description: format!(
                        "Certificate from distrusted org matching '{pattern}': {subject_cn} ({source})"
                    ),
                    offset: None,
                    evidence: Some(format!("O={issuer_org}, CN={subject_cn}")),
                });
                break;
            }
        }

        // Check expiration
        let now = chrono::Utc::now();
        let not_after = cert.validity().not_after.to_datetime();
        if let chrono::LocalResult::Single(expiry) =
            chrono::DateTime::from_timestamp(not_after.unix_timestamp(), 0)
                .map(chrono::LocalResult::Single)
                .unwrap_or(chrono::LocalResult::None)
        {
            if now > expiry {
                findings.push(Finding {
                    kind: FindingKind::ExpiredCertificate {
                        subject: subject_cn.clone(),
                        not_after: expiry.to_rfc3339(),
                        fingerprint_sha256: fingerprint.clone(),
                    },
                    severity: Severity::High,
                    description: format!(
                        "Expired certificate: {subject_cn} (expired {}) ({source})",
                        expiry.format("%Y-%m-%d")
                    ),
                    offset: None,
                    evidence: Some(format!("CN={subject_cn}, expired={}", expiry.to_rfc3339())),
                });
            }
        }

        // Check self-signed
        if cert.issuer() == cert.subject() {
            findings.push(Finding {
                kind: FindingKind::SelfSignedCertificate {
                    subject: subject_cn.clone(),
                    fingerprint_sha256: fingerprint.clone(),
                },
                severity: Severity::Medium,
                description: format!("Self-signed certificate: {subject_cn} ({source})"),
                offset: None,
                evidence: Some(format!(
                    "CN={subject_cn}, fingerprint={}",
                    &fingerprint[..16]
                )),
            });
        }

        // Check approval requirement
        if self.distrust.require_approval {
            let fp_lower = fingerprint.to_lowercase();
            let approved = self
                .distrust
                .approved_fingerprints
                .iter()
                .any(|f| f.to_lowercase() == fp_lower);
            if !approved {
                findings.push(Finding {
                    kind: FindingKind::UntrustedCertificate {
                        subject: subject_cn.clone(),
                        issuer_country: issuer_country.clone(),
                        issuer_org: issuer_org.clone(),
                        fingerprint_sha256: fingerprint.clone(),
                        reason: "Certificate not approved".to_string(),
                    },
                    severity: Severity::High,
                    description: format!(
                        "Unapproved certificate: {subject_cn} (requires explicit approval) ({source})"
                    ),
                    offset: None,
                    evidence: Some(format!("CN={subject_cn}, fingerprint={fingerprint}")),
                });
            }
        }

        findings
    }
}

impl Analyzer for CertificateAnalyzer {
    fn name(&self) -> &'static str {
        "certificate"
    }

    fn analyze(&self, context: &AnalysisContext) -> Result<Vec<Finding>, anyhow::Error> {
        let mut findings = Vec::new();
        let text = String::from_utf8_lossy(&context.raw_content);

        // Find PEM certificates
        for pem_match in PEM_REGEX.find_iter(&text) {
            let pem_text = pem_match.as_str();
            if let Ok((_, pem)) = x509_parser::pem::parse_x509_pem(pem_text.as_bytes()) {
                findings.extend(self.analyze_cert_der(&pem.contents, "embedded PEM"));
            }
        }

        // Look for DER-encoded certificates (magic bytes: 0x30 0x82)
        let raw = &context.raw_content;
        let mut i = 0;
        while i + 4 < raw.len() {
            if raw[i] == 0x30 && raw[i + 1] == 0x82 {
                let len = ((raw[i + 2] as usize) << 8) | (raw[i + 3] as usize);
                let total_len = len + 4;
                if i + total_len <= raw.len() {
                    let der_slice = &raw[i..i + total_len];
                    findings.extend(self.analyze_cert_der(der_slice, "embedded DER"));
                    i += total_len;
                    continue;
                }
            }
            i += 1;
        }

        // Deduplicate by fingerprint
        let mut seen = std::collections::HashSet::new();
        findings.retain(|f| {
            let key = match &f.kind {
                FindingKind::UntrustedCertificate {
                    fingerprint_sha256,
                    reason,
                    ..
                } => {
                    format!("{fingerprint_sha256}:{reason}")
                }
                FindingKind::ExpiredCertificate {
                    fingerprint_sha256, ..
                } => {
                    format!("{fingerprint_sha256}:expired")
                }
                FindingKind::SelfSignedCertificate {
                    fingerprint_sha256, ..
                } => {
                    format!("{fingerprint_sha256}:self_signed")
                }
                _ => return true,
            };
            seen.insert(key)
        });

        Ok(findings)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // A valid self-signed test certificate (C=US, O=TestOrg, CN=testca)
    const TEST_CERT_PEM: &str = "-----BEGIN CERTIFICATE-----
MIIDQTCCAimgAwIBAgIUDF9CKGlRJ494GcpKCotE84Pg+0wwDQYJKoZIhvcNAQEL
BQAwMDELMAkGA1UEBhMCVVMxEDAOBgNVBAoMB1Rlc3RPcmcxDzANBgNVBAMMBnRl
c3RjYTAeFw0yNjA0MTEwMTE4MTVaFw0yNzA0MTEwMTE4MTVaMDAxCzAJBgNVBAYT
AlVTMRAwDgYDVQQKDAdUZXN0T3JnMQ8wDQYDVQQDDAZ0ZXN0Y2EwggEiMA0GCSqG
SIb3DQEBAQUAA4IBDwAwggEKAoIBAQCHm5AHM0Uw8U9dpWWBnMB90rh+SuRhIpEM
0/jm6h87Mq+pEb60fcVnYUZf1eFqPaCZ1KNCEApBrW2nggRJQtn/LqDr9cImMMm4
7eD9aFO8kFkNdcHPlncDdM8vEpSxhbbHqSZdu4rV6hBfOsUmqB1LciG+tNWwZoIZ
QS3HpnBkhp7ZzMQn0e/ZMfmSlg1AHxuXffuyZfBqyTYQM13BwYBWA12RPIRJbBqt
3Q5+M1dFLn1w2/9Otp/J6w2O2EOyta4hJYWTmr5KxdVJCXQxoW42Nu4grFwhOlwa
SD6pbdsXIb4sTfwijJLGVvFzJ+ke3brGlNsXZ2YlrZj+h0xqAcTJAgMBAAGjUzBR
MB0GA1UdDgQWBBSttvWYE1iKvoTcQ1z4GMB9oohONDAfBgNVHSMEGDAWgBSttvWY
E1iKvoTcQ1z4GMB9oohONDAPBgNVHRMBAf8EBTADAQH/MA0GCSqGSIb3DQEBCwUA
A4IBAQA2KW+XhbpeMIG7VkjeI9zc7MzG9baDB3Hs0fO0TsHS9kBBdoOBUezO0AD/
4WnR7ADRTZpPNZ7L8JrxGfFrL2FPfD5XxGJBab1lBG2Dvbot03j1LolllQ9Hzfl6
DD8I1mwvAhlqmwIzxk/nHBpTduZVgX6XZ4k7c2/ZtwPqIoivTcPOlXhhitHBEfzq
qvfLhAt9qKE9i48lZE72ufHHgITG0lgd+Jqt2/gnxd/ncL/5ZGMrVWGQAc8f7+YR
/HMZBrGMtrQea2q3uxSaDdk0CBlQEwODrUR6HNlxQKKsnEYFsHqtSa6Nd8ON2D3S
k8Hg+bcBK1/MGXxkN+/GFpZpnakt
-----END CERTIFICATE-----";

    fn make_context(content: &[u8]) -> AnalysisContext {
        AnalysisContext {
            file_path: "test.pem".to_string(),
            file_name: "test.pem".to_string(),
            sha256: "abc123".to_string(),
            extracted_urls: vec![],
            raw_content: content.to_vec(),
            processed_content: vec![],
        }
    }

    #[test]
    fn test_detects_self_signed() {
        let analyzer = CertificateAnalyzer::new(CertDistrust {
            country_codes: vec![],
            org_patterns: vec![],
            require_approval: false,
            approved_fingerprints: vec![],
        });
        let ctx = make_context(TEST_CERT_PEM.as_bytes());
        let findings = analyzer.analyze(&ctx).unwrap();
        assert!(
            findings
                .iter()
                .any(|f| matches!(&f.kind, FindingKind::SelfSignedCertificate { .. })),
            "should detect self-signed cert"
        );
    }

    #[test]
    fn test_no_certs_in_plain_text() {
        let analyzer = CertificateAnalyzer::new(CertDistrust {
            country_codes: vec!["CN".to_string()],
            org_patterns: vec![],
            require_approval: false,
            approved_fingerprints: vec![],
        });
        let ctx = make_context(b"just plain text, no certificates here");
        let findings = analyzer.analyze(&ctx).unwrap();
        assert!(findings.is_empty());
    }

    #[test]
    fn test_require_approval_flags_unapproved() {
        let analyzer = CertificateAnalyzer::new(CertDistrust {
            country_codes: vec![],
            org_patterns: vec![],
            require_approval: true,
            approved_fingerprints: vec![],
        });
        let ctx = make_context(TEST_CERT_PEM.as_bytes());
        let findings = analyzer.analyze(&ctx).unwrap();
        assert!(
            findings.iter().any(|f| matches!(
                &f.kind,
                FindingKind::UntrustedCertificate { reason, .. } if reason.contains("not approved")
            )),
            "should flag unapproved cert"
        );
    }
}
