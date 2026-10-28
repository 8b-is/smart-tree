use sha2::{Digest, Sha256};
use x509_parser::prelude::*;

use crate::magiscanner::analyzers::{AnalysisContext, Analyzer};
use crate::magiscanner::certificates::{visit_certificates, CertificateInspection};
use crate::magiscanner::finding::{Finding, FindingKind, Severity};

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

    pub fn inspect(&self, raw: &[u8]) -> CertificateInspection {
        let mut result = CertificateInspection::default();
        let now = chrono::Utc::now();
        result.issues = visit_certificates(raw, |info, der| {
            let mut findings = self.analyze_cert_der(der, &info.encoding, now);
            for finding in &mut findings {
                finding.offset = Some(info.offset);
            }
            result.findings.extend(findings);
            result.certificates.push(info);
        });
        result.certificates.sort_by_key(|cert| cert.offset);
        result
    }

    fn analyze_cert_der(
        &self,
        der_bytes: &[u8],
        source: &str,
        now: chrono::DateTime<chrono::Utc>,
    ) -> Vec<Finding> {
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

        if now.timestamp() < cert.validity().not_before.timestamp() {
            let not_before = cert.validity().not_before.to_string();
            findings.push(Finding {
                kind: FindingKind::NotYetValidCertificate {
                    subject: subject_cn.clone(),
                    not_before: not_before.clone(),
                    fingerprint_sha256: fingerprint.clone(),
                },
                severity: Severity::High,
                description: format!(
                    "Certificate not yet valid: {subject_cn} (valid from {not_before}) ({source})"
                ),
                offset: None,
                evidence: None,
            });
        }

        // Matching names indicate self-issuance, not a verified self-signature.
        if cert.issuer() == cert.subject() {
            findings.push(Finding {
                kind: FindingKind::SelfSignedCertificate {
                    subject: subject_cn.clone(),
                    fingerprint_sha256: fingerprint.clone(),
                },
                severity: Severity::Medium,
                description: format!(
                    "Self-issued certificate: {subject_cn} (signature not verified) ({source})"
                ),
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
        Ok(self.inspect(&context.raw_content).findings)
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

    fn inventory_analyzer() -> CertificateAnalyzer {
        CertificateAnalyzer::new(CertDistrust {
            country_codes: vec![],
            org_patterns: vec![],
            require_approval: false,
            approved_fingerprints: vec![],
        })
    }

    #[test]
    fn inventory_preserves_binary_offsets_and_deduplicates_encodings() {
        let (_, pem) = x509_parser::pem::parse_x509_pem(TEST_CERT_PEM.as_bytes()).unwrap();
        let mut raw = vec![0xff, 0xfe, 0x80];
        raw.extend_from_slice(TEST_CERT_PEM.as_bytes());
        raw.extend_from_slice(&pem.contents);
        let result = inventory_analyzer().inspect(&raw);
        assert_eq!(result.certificates.len(), 1);
        let cert = &result.certificates[0];
        assert_eq!(cert.offset, 3);
        assert_eq!(cert.encoding, "PEM");
        assert!(cert.subject.contains("testca"));
        assert!(cert.issuer.contains("TestOrg"));
        assert_eq!(
            cert.fingerprint_sha256,
            hex::encode(Sha256::digest(&pem.contents))
        );
        assert_eq!(cert.validity_at(cert.not_before - 1), "not yet valid");
        assert_eq!(cert.validity_at(cert.not_before), "within validity period");
        assert_eq!(cert.validity_at(cert.not_after), "within validity period");
        assert_eq!(cert.validity_at(cert.not_after + 1), "expired");
        assert!(result
            .findings
            .iter()
            .all(|finding| finding.offset == Some(3)));
    }

    #[test]
    fn der_wrapper_does_not_hide_nested_certificate() {
        let (_, pem) = x509_parser::pem::parse_x509_pem(TEST_CERT_PEM.as_bytes()).unwrap();
        let mut wrapped = vec![0x30, 0x82];
        wrapped.extend_from_slice(&(pem.contents.len() as u16).to_be_bytes());
        wrapped.extend_from_slice(&pem.contents);
        let result = inventory_analyzer().inspect(&wrapped);
        assert_eq!(result.certificates.len(), 1);
        assert_eq!(result.certificates[0].offset, 4);
        assert_eq!(result.certificates[0].encoding, "DER");
    }

    #[test]
    fn malformed_pem_does_not_hide_following_certificate() {
        let raw = format!("-----BEGIN CERTIFICATE-----\nbroken\n{TEST_CERT_PEM}");
        let result = inventory_analyzer().inspect(raw.as_bytes());
        assert_eq!(result.issues.len(), 1);
        assert_eq!(result.issues[0].offset, 0);
        assert_eq!(result.certificates.len(), 1);
    }

    #[test]
    fn reports_both_ends_of_the_certificate_validity_window() {
        let (_, pem) = x509_parser::pem::parse_x509_pem(TEST_CERT_PEM.as_bytes()).unwrap();
        let analyzer = inventory_analyzer();
        let result = analyzer.inspect(TEST_CERT_PEM.as_bytes());
        let cert = &result.certificates[0];
        let before = chrono::DateTime::from_timestamp(cert.not_before - 1, 0).unwrap();
        let after = chrono::DateTime::from_timestamp(cert.not_after + 1, 0).unwrap();
        assert!(analyzer
            .analyze_cert_der(&pem.contents, "DER", before)
            .iter()
            .any(|finding| matches!(finding.kind, FindingKind::NotYetValidCertificate { .. })));
        assert!(analyzer
            .analyze_cert_der(&pem.contents, "DER", after)
            .iter()
            .any(|finding| matches!(finding.kind, FindingKind::ExpiredCertificate { .. })));
    }

    #[test]
    fn explicit_scan_reports_metadata_parse_errors_and_size_limits() {
        use crate::magiscanner::{certificate_scan::scan_certificates, SecurityConfig};
        let temp = tempfile::tempdir().unwrap();
        let target = temp.path().join("bundle.bin");
        std::fs::write(&target, TEST_CERT_PEM).unwrap();
        let mut config = SecurityConfig::default();
        config.database.path = temp.path().join("security.db").display().to_string();
        let result = scan_certificates(&config, &target, false).unwrap();
        assert_eq!(result.files_scanned, 1);
        assert_eq!(result.files[0].inspection.certificates.len(), 1);
        assert!(result.skipped.is_empty());
        std::fs::write(&target, "-----BEGIN CERTIFICATE-----\nbroken").unwrap();
        let result = scan_certificates(&config, &target, false).unwrap();
        assert_eq!(result.files[0].inspection.issues.len(), 1);
        config.scan.max_file_size_mb = 0;
        let result = scan_certificates(&config, &target, false).unwrap();
        assert_eq!(result.files_scanned, 0);
        assert_eq!(result.skipped.len(), 1);
    }
}
