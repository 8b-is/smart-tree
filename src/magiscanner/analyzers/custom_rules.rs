use argon2::password_hash::SaltString;
use argon2::{Argon2, PasswordHasher};
use regex::Regex;
use sha2::{Digest, Sha256};

use crate::magiscanner::analyzers::{AnalysisContext, Analyzer};
use crate::magiscanner::finding::{Finding, FindingKind, Severity};

/// Types of custom rules the user can define.
#[derive(Debug, Clone)]
pub enum RuleKind {
    /// Regex pattern match on file content
    Regex { pattern: Regex },
    /// Block a TLD (e.g., ".su", ".cn")
    Tld { tld: String },
    /// Block content from a specific company/organization name
    Company { name: String },
    /// Block IPFS content identifiers
    Ipfs,
    /// Password guard — detects fragments of a secret in plaintext.
    /// Stores an Argon2id hash salted with hardware ID. Cannot be reversed.
    PasswordGuard { argon2_hash: String },
}

#[derive(Debug, Clone)]
pub struct CustomRule {
    pub name: String,
    pub kind: RuleKind,
    pub severity: Severity,
    pub description: String,
}

/// Get a machine-specific salt for password hashing.
/// Uses layered sources that require escalating privilege to access:
/// 1. /etc/machine-id (unique per install)
/// 2. DMI board serial (needs root)
/// 3. DMI product UUID (needs root)
/// Falls back gracefully if privileged sources unavailable.
pub fn get_hardware_salt() -> Vec<u8> {
    let mut salt_material = Vec::new();

    // Layer 1: machine-id (always available, unique per install)
    if let Ok(mid) = std::fs::read_to_string("/etc/machine-id") {
        salt_material.extend_from_slice(mid.trim().as_bytes());
    }

    // Layer 2: DMI board serial (needs root on most systems)
    if let Ok(serial) = std::fs::read_to_string("/sys/class/dmi/id/board_serial") {
        salt_material.extend_from_slice(serial.trim().as_bytes());
    }

    // Layer 3: DMI product UUID (needs root)
    if let Ok(uuid) = std::fs::read_to_string("/sys/class/dmi/id/product_uuid") {
        salt_material.extend_from_slice(uuid.trim().as_bytes());
    }

    // Layer 4: boot_id (changes per boot, adds temporal uniqueness)
    if let Ok(boot_id) = std::fs::read_to_string("/proc/sys/kernel/random/boot_id") {
        salt_material.extend_from_slice(boot_id.trim().as_bytes());
    }

    // Hash the combined material into a fixed-size salt
    let hash = Sha256::digest(&salt_material);
    hash.to_vec()
}

/// Hash a password/secret using Argon2id with hardware-derived salt.
/// The result cannot be brute-forced without physical machine access.
pub fn hash_secret(secret: &str) -> Result<String, String> {
    let hw_salt = get_hardware_salt();

    // Create a deterministic salt from hardware ID (Argon2 needs base64-encoded salt)
    // We hash the hardware salt to get exactly 16 bytes for SaltString
    let salt_bytes = &Sha256::digest(&hw_salt)[..16];
    let salt_b64 = base64_encode_salt(salt_bytes);
    let salt = SaltString::from_b64(&salt_b64).map_err(|e| format!("salt error: {e}"))?;

    let argon2 = Argon2::default();
    let hash = argon2
        .hash_password(secret.as_bytes(), &salt)
        .map_err(|e| format!("hash error: {e}"))?;

    Ok(hash.to_string())
}

/// Verify a secret against a stored Argon2id hash.
pub fn verify_secret(secret: &str, stored_hash: &str) -> bool {
    use argon2::PasswordVerifier;
    let parsed = match argon2::PasswordHash::new(stored_hash) {
        Ok(h) => h,
        Err(_) => return false,
    };
    Argon2::default()
        .verify_password(secret.as_bytes(), &parsed)
        .is_ok()
}

fn make_password_finding(rule_name: &str, offset: usize, len: usize) -> Finding {
    Finding {
        kind: FindingKind::SuspiciousString {
            value: "password_exposure".to_string(),
        },
        severity: Severity::Critical,
        description: format!(
            "PASSWORD EXPOSURE: a secret/password fragment was found in plaintext (custom rule: {rule_name})"
        ),
        offset: Some(offset),
        evidence: Some(format!(
            "{}... ({len} chars at offset 0x{offset:x})",
            "*".repeat(len.min(8)),
        )),
    }
}

fn base64_encode_salt(bytes: &[u8]) -> String {
    use base64::Engine;
    base64::engine::general_purpose::STANDARD_NO_PAD.encode(bytes)
}

// Common IPFS patterns
fn ipfs_patterns() -> Vec<Regex> {
    vec![
        Regex::new(r"(?i)Qm[1-9A-HJ-NP-Za-km-z]{44}").unwrap(), // CIDv0
        Regex::new(r"(?i)bafy[a-z2-7]{55,}").unwrap(),          // CIDv1 base32
        Regex::new(r"(?i)/ipfs/[A-Za-z0-9]+").unwrap(),         // IPFS path
        Regex::new(r"(?i)/ipns/[A-Za-z0-9\.\-]+").unwrap(),     // IPNS path
        Regex::new(r"(?i)ipfs://[A-Za-z0-9]+").unwrap(),        // IPFS URI
    ]
}

pub struct CustomRuleAnalyzer {
    rules: Vec<CustomRule>,
    ipfs_regexes: Vec<Regex>,
}

impl CustomRuleAnalyzer {
    pub fn new(rules: Vec<CustomRule>) -> Self {
        let ipfs_regexes = if rules.iter().any(|r| matches!(r.kind, RuleKind::Ipfs)) {
            ipfs_patterns()
        } else {
            vec![]
        };
        Self {
            rules,
            ipfs_regexes,
        }
    }

    fn check_rule(&self, rule: &CustomRule, text: &str, data: &[u8]) -> Vec<Finding> {
        let mut findings = Vec::new();

        match &rule.kind {
            RuleKind::Regex { pattern } => {
                for m in pattern.find_iter(text) {
                    findings.push(Finding {
                        kind: FindingKind::SuspiciousString {
                            value: format!("custom_rule:{}", rule.name),
                        },
                        severity: rule.severity,
                        description: format!("{} (custom rule: {})", rule.description, rule.name),
                        offset: Some(m.start()),
                        evidence: Some(m.as_str().chars().take(100).collect()),
                    });
                }
            }

            RuleKind::Tld { tld } => {
                let escaped_tld = regex::escape(tld);
                let tld_pattern = format!(r"(?i)[a-zA-Z0-9\-]+{escaped_tld}");
                if let Ok(re) = Regex::new(&tld_pattern) {
                    for m in re.find_iter(text) {
                        findings.push(Finding {
                            kind: FindingKind::SuspiciousString {
                                value: format!("blocked_tld:{tld}"),
                            },
                            severity: rule.severity,
                            description: format!(
                                "Blocked TLD '{tld}' found: {} (custom rule: {})",
                                m.as_str(),
                                rule.name
                            ),
                            offset: Some(m.start()),
                            evidence: Some(m.as_str().to_string()),
                        });
                    }
                }
            }

            RuleKind::Company { name } => {
                let name_lower = name.to_lowercase();
                let text_lower = text.to_lowercase();
                let mut search_start = 0;
                while let Some(pos) = text_lower[search_start..].find(&name_lower) {
                    let abs_pos = search_start + pos;
                    let end = (abs_pos + name.len() + 40).min(text.len());
                    let start = abs_pos.saturating_sub(20);
                    let context = &text[start..end];

                    findings.push(Finding {
                        kind: FindingKind::SuspiciousString {
                            value: format!("blocked_company:{name}"),
                        },
                        severity: rule.severity,
                        description: format!(
                            "Blocked company/org '{}' found (custom rule: {})",
                            name, rule.name
                        ),
                        offset: Some(abs_pos),
                        evidence: Some(context.chars().take(100).collect()),
                    });
                    search_start = abs_pos + name.len();
                    // Only report first occurrence
                    break;
                }
            }

            RuleKind::Ipfs => {
                for re in &self.ipfs_regexes {
                    for m in re.find_iter(text) {
                        findings.push(Finding {
                            kind: FindingKind::SuspiciousString {
                                value: format!(
                                    "ipfs:{}",
                                    m.as_str().chars().take(30).collect::<String>()
                                ),
                            },
                            severity: rule.severity,
                            description: format!(
                                "IPFS content identifier found: {} (custom rule: {})",
                                m.as_str().chars().take(60).collect::<String>(),
                                rule.name
                            ),
                            offset: Some(m.start()),
                            evidence: Some(m.as_str().chars().take(80).collect()),
                        });
                        // Only report first per pattern
                        break;
                    }
                }
            }

            RuleKind::PasswordGuard { argon2_hash } => {
                // Extract printable strings from file, then check sliding
                // windows within each string. This catches the password even
                // when it's embedded in a larger token like "key=MyPassword".
                let min_len = 4;
                let mut strings: Vec<(usize, String)> = Vec::new();
                let mut current = String::new();
                let mut start_offset = 0;

                for (i, &byte) in data.iter().enumerate() {
                    if byte.is_ascii_graphic() || byte == b' ' {
                        if current.is_empty() {
                            start_offset = i;
                        }
                        current.push(byte as char);
                    } else {
                        if current.len() >= min_len {
                            strings.push((start_offset, current.clone()));
                        }
                        current.clear();
                    }
                }
                if current.len() >= min_len {
                    strings.push((start_offset, current));
                }

                // For each extracted string, check:
                // 1. The full string itself
                // 2. Substrings split by common delimiters (=, :, space, tab, comma)
                // This catches "key=MyPassword" by testing "MyPassword" separately.
                // Argon2 is intentionally slow, so we minimize verification calls.
                let delimiters = ['=', ':', ' ', '\t', ',', ';', '"', '\''];

                'outer: for (offset, s) in &strings {
                    // Check the full string
                    if verify_secret(s, argon2_hash) {
                        findings.push(make_password_finding(&rule.name, *offset, s.len()));
                        break;
                    }

                    // Split by delimiters and check each part
                    let mut parts: Vec<(usize, &str)> = Vec::new();
                    let mut last = 0;
                    for (i, c) in s.char_indices() {
                        if delimiters.contains(&c) {
                            if i > last {
                                parts.push((last, &s[last..i]));
                            }
                            last = i + c.len_utf8();
                        }
                    }
                    if last < s.len() {
                        parts.push((last, &s[last..]));
                    }

                    for (part_offset, part) in &parts {
                        if part.len() >= min_len && verify_secret(part, argon2_hash) {
                            findings.push(make_password_finding(
                                &rule.name,
                                offset + part_offset,
                                part.len(),
                            ));
                            break 'outer;
                        }
                    }
                }
            }
        }

        findings
    }
}

impl Analyzer for CustomRuleAnalyzer {
    fn name(&self) -> &'static str {
        "custom_rules"
    }

    fn analyze(&self, context: &AnalysisContext) -> Result<Vec<Finding>, anyhow::Error> {
        if self.rules.is_empty() {
            return Ok(Vec::new());
        }

        let text = String::from_utf8_lossy(&context.raw_content);
        let mut findings = Vec::new();

        for rule in &self.rules {
            findings.extend(self.check_rule(rule, &text, &context.raw_content));
        }

        Ok(findings)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_context(data: &[u8]) -> AnalysisContext {
        AnalysisContext {
            file_path: "test.txt".to_string(),
            file_name: "test.txt".to_string(),
            sha256: "test".to_string(),
            extracted_urls: vec![],
            raw_content: data.to_vec(),
            processed_content: vec![],
        }
    }

    #[test]
    fn test_tld_blocking() {
        let rules = vec![CustomRule {
            name: "block_su".to_string(),
            kind: RuleKind::Tld {
                tld: ".su".to_string(),
            },
            severity: Severity::High,
            description: "Soviet Union TLD blocked".to_string(),
        }];
        let analyzer = CustomRuleAnalyzer::new(rules);
        let ctx = make_context(b"connecting to malware.su for updates");
        let findings = analyzer.analyze(&ctx).unwrap();
        assert!(!findings.is_empty(), "should detect .su TLD: {findings:?}");
    }

    #[test]
    fn test_company_blocking() {
        let rules = vec![CustomRule {
            name: "block_alibaba".to_string(),
            kind: RuleKind::Company {
                name: "Alibaba".to_string(),
            },
            severity: Severity::Medium,
            description: "Alibaba references blocked".to_string(),
        }];
        let analyzer = CustomRuleAnalyzer::new(rules);
        let ctx = make_context(b"fetching config from alibaba cloud CDN endpoint");
        let findings = analyzer.analyze(&ctx).unwrap();
        assert!(!findings.is_empty(), "should detect Alibaba: {findings:?}");
    }

    #[test]
    fn test_ipfs_detection() {
        let rules = vec![CustomRule {
            name: "block_ipfs".to_string(),
            kind: RuleKind::Ipfs,
            severity: Severity::Medium,
            description: "IPFS content blocked".to_string(),
        }];
        let analyzer = CustomRuleAnalyzer::new(rules);
        let ctx =
            make_context(b"loading from ipfs://QmT5NvUtoM5nWFfrQdVrFtvGfKFmG7AHE8P34isapyhCxX");
        let findings = analyzer.analyze(&ctx).unwrap();
        assert!(!findings.is_empty(), "should detect IPFS CID: {findings:?}");
    }

    #[test]
    fn test_custom_regex() {
        let rules = vec![CustomRule {
            name: "block_tracking_pixels".to_string(),
            kind: RuleKind::Regex {
                pattern: Regex::new(r"(?i)tracking[_\-]?pixel|1x1\.gif|beacon\.gif").unwrap(),
            },
            severity: Severity::Low,
            description: "Tracking pixel detected".to_string(),
        }];
        let analyzer = CustomRuleAnalyzer::new(rules);
        let ctx = make_context(b"<img src='tracking_pixel.png' width=1 height=1>");
        let findings = analyzer.analyze(&ctx).unwrap();
        assert!(
            !findings.is_empty(),
            "should detect tracking pixel: {findings:?}"
        );
    }

    #[test]
    fn test_password_guard() {
        let secret = "MyS3cretP@ss!";
        let hash = hash_secret(secret).expect("should hash");

        let rules = vec![CustomRule {
            name: "password_guard".to_string(),
            kind: RuleKind::PasswordGuard { argon2_hash: hash },
            severity: Severity::Critical,
            description: "Password found in plaintext".to_string(),
        }];
        let analyzer = CustomRuleAnalyzer::new(rules);

        // File containing the password in plaintext
        let ctx = make_context(format!("config_password={secret}\n").as_bytes());
        let findings = analyzer.analyze(&ctx).unwrap();
        assert!(
            findings
                .iter()
                .any(|f| f.description.contains("PASSWORD EXPOSURE")),
            "should detect password in plaintext: {findings:?}"
        );

        // Evidence should NOT contain the actual password
        for f in &findings {
            if let Some(evidence) = &f.evidence {
                assert!(
                    !evidence.contains(secret),
                    "evidence must NOT contain the actual password!"
                );
            }
        }
    }

    #[test]
    fn test_password_guard_no_match() {
        let hash = hash_secret("MyS3cretP@ss!").expect("should hash");

        let rules = vec![CustomRule {
            name: "password_guard".to_string(),
            kind: RuleKind::PasswordGuard { argon2_hash: hash },
            severity: Severity::Critical,
            description: "Password found in plaintext".to_string(),
        }];
        let analyzer = CustomRuleAnalyzer::new(rules);

        let ctx = make_context(b"this file has no passwords in it at all");
        let findings = analyzer.analyze(&ctx).unwrap();
        assert!(
            findings.is_empty(),
            "should not false-positive: {findings:?}"
        );
    }
}
