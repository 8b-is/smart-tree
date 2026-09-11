//! Public certificate metadata extracted from PEM bundles and embedded DER.
//! Parsing does not establish certificate-chain trust or verify signatures.

use regex::bytes::Regex;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::HashSet;
use std::sync::LazyLock;
use x509_parser::prelude::*;

static PEM_START: LazyLock<Regex> =
    LazyLock::new(|| Regex::new("-----BEGIN CERTIFICATE-----").unwrap());
const PEM_END: &[u8] = b"-----END CERTIFICATE-----";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CertificateInfo {
    pub subject: String,
    pub issuer: String,
    pub serial: String,
    pub fingerprint_sha256: String,
    pub not_before: i64,
    pub not_after: i64,
    pub subject_alternative_names: Vec<String>,
    pub is_ca: bool,
    /// Matching names only; the signature is not verified.
    pub is_self_issued: bool,
    pub encoding: String,
    /// Byte offset in the original file, before any text decoding.
    pub offset: usize,
}

impl CertificateInfo {
    pub fn validity_at(&self, timestamp: i64) -> &'static str {
        if timestamp < self.not_before {
            "not yet valid"
        } else if timestamp > self.not_after {
            "expired"
        } else {
            "within validity period"
        }
    }

    fn from_cert(cert: &X509Certificate<'_>, der: &[u8], encoding: &str, offset: usize) -> Self {
        Self {
            subject: cert.subject().to_string(),
            issuer: cert.issuer().to_string(),
            serial: cert.raw_serial_as_string(),
            fingerprint_sha256: hex::encode(Sha256::digest(der)),
            not_before: cert.validity().not_before.timestamp(),
            not_after: cert.validity().not_after.timestamp(),
            subject_alternative_names: cert
                .subject_alternative_name()
                .ok()
                .flatten()
                .map(|san| {
                    san.value
                        .general_names
                        .iter()
                        .map(ToString::to_string)
                        .collect()
                })
                .unwrap_or_default(),
            is_ca: cert.is_ca(),
            is_self_issued: cert.issuer() == cert.subject(),
            encoding: encoding.to_string(),
            offset,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CertificateIssue {
    pub offset: usize,
    pub message: String,
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct CertificateInspection {
    pub certificates: Vec<CertificateInfo>,
    pub findings: Vec<crate::magiscanner::finding::Finding>,
    pub issues: Vec<CertificateIssue>,
}

/// Visit each distinct certificate once. Only successful parses advance past a
/// DER object, so a non-certificate ASN.1 wrapper cannot hide a certificate.
pub(crate) fn visit_certificates(
    raw: &[u8],
    mut visit: impl FnMut(CertificateInfo, &[u8]),
) -> Vec<CertificateIssue> {
    let mut seen = HashSet::new();
    let mut issues = Vec::new();
    let mut accept = |der: &[u8], encoding: &str, offset| {
        let Ok((remaining, cert)) = X509Certificate::from_der(der) else {
            return false;
        };
        if !remaining.is_empty() {
            return false;
        }
        let info = CertificateInfo::from_cert(&cert, der, encoding, offset);
        if seen.insert(info.fingerprint_sha256.clone()) {
            visit(info, der);
        }
        true
    };

    let starts: Vec<_> = PEM_START.find_iter(raw).map(|m| m.start()).collect();
    for (index, &offset) in starts.iter().enumerate() {
        // A broken block must not consume the following valid PEM block.
        let limit = starts.get(index + 1).copied().unwrap_or(raw.len());
        let block = &raw[offset..limit];
        let valid = block
            .windows(PEM_END.len())
            .position(|window| window == PEM_END)
            .and_then(|end| x509_parser::pem::parse_x509_pem(&block[..end + PEM_END.len()]).ok())
            .is_some_and(|(_, pem)| accept(&pem.contents, "PEM", offset));
        if !valid {
            issues.push(CertificateIssue {
                offset,
                message: "Malformed or truncated PEM certificate".to_string(),
            });
        }
    }

    let mut offset = 0;
    while offset < raw.len() {
        if let Some(length) = der_sequence_length(&raw[offset..]) {
            if accept(&raw[offset..offset + length], "DER", offset) {
                offset += length;
                continue;
            }
        }
        offset += 1;
    }
    issues
}

/// Read definite ASN.1 lengths without trusting them for allocation or skipping.
fn der_sequence_length(raw: &[u8]) -> Option<usize> {
    if raw.first() != Some(&0x30) {
        return None;
    }
    let first = *raw.get(1)?;
    let (header, content) = if first < 0x80 {
        (2, usize::from(first))
    } else {
        let count = usize::from(first & 0x7f);
        if count == 0 || count > std::mem::size_of::<usize>() {
            return None;
        }
        let mut content = 0usize;
        for &byte in raw.get(2..2 + count)? {
            content = content.checked_mul(256)?.checked_add(usize::from(byte))?;
        }
        (2 + count, content)
    };
    let total = content.checked_add(header)?;
    (total <= raw.len()).then_some(total)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn handles_definite_lengths_and_rejects_overflow_or_truncation() {
        assert_eq!(der_sequence_length(&[0x30, 1, 0]), Some(3));
        for length_bytes in 1..=3 {
            let mut raw = vec![0x30, 0x80 + length_bytes as u8];
            raw.extend(vec![0; length_bytes - 1]);
            raw.push(128);
            raw.extend(vec![0; 128]);
            assert_eq!(der_sequence_length(&raw), Some(raw.len()));
            raw.pop();
            assert_eq!(der_sequence_length(&raw), None);
        }
        assert_eq!(der_sequence_length(&[0x30, 0x80, 0, 0]), None);
        assert_eq!(der_sequence_length(&[0x30, 0xff]), None);
        let mut overflow = vec![0x30, 0x88];
        overflow.extend([0xff; 8]);
        assert_eq!(der_sequence_length(&overflow), None);
    }
}
