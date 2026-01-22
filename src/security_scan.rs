//! Security Scanner for detecting supply chain attack patterns
//!
//! Scans directories for malicious patterns including:
//! - IPFS/IPNS phone-home endpoints
//! - Fake cryptographic verification
//! - Dynamic npm package execution
//! - Known malicious package references

use anyhow::Result;
use regex::Regex;
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

/// Risk level for detected patterns
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum RiskLevel {
    Low,
    Medium,
    High,
    Critical,
}

impl std::fmt::Display for RiskLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RiskLevel::Low => write!(f, "LOW"),
            RiskLevel::Medium => write!(f, "MEDIUM"),
            RiskLevel::High => write!(f, "HIGH"),
            RiskLevel::Critical => write!(f, "CRITICAL"),
        }
    }
}

/// A detected security pattern
#[derive(Debug, Clone)]
pub struct SecurityFinding {
    pub file_path: PathBuf,
    pub line_number: usize,
    pub pattern_name: String,
    pub matched_text: String,
    pub risk_level: RiskLevel,
    pub description: String,
}

/// Pattern definition for security scanning
struct Pattern {
    name: &'static str,
    regex: Regex,
    risk_level: RiskLevel,
    description: &'static str,
}

/// Security scanner configuration
pub struct SecurityScanner {
    patterns: Vec<Pattern>,
    /// Paths that indicate executable context (higher risk)
    executable_paths: Vec<&'static str>,
    /// Paths that indicate history/logs (lower risk)
    history_paths: Vec<&'static str>,
}

impl SecurityScanner {
    pub fn new() -> Self {
        let patterns = vec![
            // IPFS Gateway URLs - phone home endpoints
            Pattern {
                name: "IPFS Gateway",
                regex: Regex::new(r"https?://(ipfs\.io|dweb\.link|cloudflare-ipfs\.com|gateway\.pinata\.cloud|w3s\.link|4everland\.io)").unwrap(),
                risk_level: RiskLevel::High,
                description: "IPFS gateway URL detected - may fetch remote content",
            },
            // IPNS name patterns (mutable addressing)
            Pattern {
                name: "IPNS Name",
                regex: Regex::new(r"k51qzi5uqu5[a-z0-9]{40,}").unwrap(),
                risk_level: RiskLevel::Critical,
                description: "IPNS mutable name - content can be changed by key holder",
            },
            // Dynamic npm execution with volatile tags
            Pattern {
                name: "Dynamic NPX",
                regex: Regex::new(r"npx\s+[\w@/-]+@(alpha|beta|latest|next|canary)").unwrap(),
                risk_level: RiskLevel::High,
                description: "Dynamic npm execution - package content can change anytime",
            },
            // Known malicious packages
            Pattern {
                name: "Known Risk Package",
                regex: Regex::new(r"(claude-flow|agentic-flow)(@|\s|$|/)").unwrap(),
                risk_level: RiskLevel::Critical,
                description: "Known supply chain risk package with remote injection capability",
            },
            // Fake signature verification (length-only check)
            Pattern {
                name: "Fake Verification",
                regex: Regex::new(r"\.length\s*===?\s*(64|128|256)\s*[;)}]").unwrap(),
                risk_level: RiskLevel::Critical,
                description: "Fake cryptographic verification - checks length instead of signature",
            },
            // Registry signature without actual verification
            Pattern {
                name: "Unverified Signature",
                regex: Regex::new(r"registrySignature.*randomBytes|crypto\.randomBytes.*signature").unwrap(),
                risk_level: RiskLevel::Critical,
                description: "Random bytes used as signature - no actual verification",
            },
            // Pattern fetching from remote
            Pattern {
                name: "Remote Pattern Fetch",
                regex: Regex::new(r"fetch.*pattern|pattern.*fetch|loadPattern.*http|http.*loadPattern").unwrap(),
                risk_level: RiskLevel::High,
                description: "Remote pattern/behavior fetching detected",
            },
            // Silent failure patterns (never throw on verification)
            Pattern {
                name: "Silent Failure",
                regex: Regex::new(r"catch\s*\([^)]*\)\s*\{[^}]*return\s+(true|null|\[\]|\{\})[^}]*\}").unwrap(),
                risk_level: RiskLevel::Medium,
                description: "Silent failure on error - may hide security issues",
            },
            // Hooks that auto-execute
            Pattern {
                name: "Auto Hook",
                regex: Regex::new(r"(PreToolUse|PostToolUse|UserPromptSubmit|SessionStart).*npx").unwrap(),
                risk_level: RiskLevel::High,
                description: "Hook configured to auto-execute npm package",
            },
        ];

        Self {
            patterns,
            executable_paths: vec![
                "commands",
                "hooks",
                "scripts",
                "bin",
                ".claude/commands",
                "node_modules/.bin",
            ],
            history_paths: vec![
                "shell-snapshots",
                "history",
                "logs",
                ".bash_history",
                ".zsh_history",
            ],
        }
    }

    /// Scan a directory for security patterns
    /// Unlike normal st, this IGNORES gitignore and scans everything
    pub fn scan_directory(&self, path: &Path) -> Result<Vec<SecurityFinding>> {
        let mut findings = Vec::new();

        // Walk directory without respecting gitignore
        for entry in WalkDir::new(path)
            .follow_links(false)
            .into_iter()
            .filter_map(|e| e.ok())
        {
            let file_path = entry.path();

            // Skip binary files and very large files
            if !self.should_scan_file(file_path) {
                continue;
            }

            // Read and scan file contents
            if let Ok(content) = fs::read_to_string(file_path) {
                self.scan_content(file_path, &content, &mut findings);
            }
        }

        // Sort by risk level (critical first)
        findings.sort_by(|a, b| b.risk_level.cmp(&a.risk_level));

        Ok(findings)
    }

    fn should_scan_file(&self, path: &Path) -> bool {
        // Skip binary extensions
        let skip_extensions = [
            "png", "jpg", "jpeg", "gif", "ico", "webp", "svg", "woff", "woff2", "ttf", "otf",
            "eot", "mp3", "mp4", "wav", "ogg", "webm", "zip", "tar", "gz", "bz2", "xz", "7z",
            "exe", "dll", "so", "dylib", "pdf", "doc", "docx", "xls", "xlsx", "pyc", "pyo",
            "class", "o", "a", "wasm", "mem8",
        ];

        if let Some(ext) = path.extension() {
            if skip_extensions.contains(&ext.to_string_lossy().to_lowercase().as_str()) {
                return false;
            }
        }

        // Skip very large files (>10MB)
        if let Ok(metadata) = fs::metadata(path) {
            if metadata.len() > 10 * 1024 * 1024 {
                return false;
            }
        }

        // Must be a file
        path.is_file()
    }

    fn scan_content(&self, file_path: &Path, content: &str, findings: &mut Vec<SecurityFinding>) {
        let path_str = file_path.to_string_lossy();

        // Determine if this is an executable context or history
        let is_executable = self.executable_paths.iter().any(|p| path_str.contains(p));
        let is_history = self.history_paths.iter().any(|p| path_str.contains(p));

        for (line_number, line) in content.lines().enumerate() {
            for pattern in &self.patterns {
                if let Some(m) = pattern.regex.find(line) {
                    // Adjust risk based on context
                    let adjusted_risk = if is_history {
                        // History files are lower risk
                        match pattern.risk_level {
                            RiskLevel::Critical => RiskLevel::Medium,
                            RiskLevel::High => RiskLevel::Low,
                            other => other,
                        }
                    } else if is_executable {
                        // Executable context keeps or elevates risk
                        pattern.risk_level
                    } else {
                        pattern.risk_level
                    };

                    findings.push(SecurityFinding {
                        file_path: file_path.to_path_buf(),
                        line_number: line_number + 1,
                        pattern_name: pattern.name.to_string(),
                        matched_text: m.as_str().to_string(),
                        risk_level: adjusted_risk,
                        description: pattern.description.to_string(),
                    });
                }
            }
        }
    }

    /// Generate a summary report
    pub fn generate_report(&self, findings: &[SecurityFinding]) -> String {
        let mut report = String::new();

        report.push_str("\n\u{1F50D} Security Scan Results\n");
        report.push_str("═══════════════════════════════════════════════════════════════\n\n");

        if findings.is_empty() {
            report.push_str("\u{2705} No security patterns detected.\n");
            return report;
        }

        // Count by risk level
        let mut by_risk: HashMap<RiskLevel, Vec<&SecurityFinding>> = HashMap::new();
        for finding in findings {
            by_risk.entry(finding.risk_level).or_default().push(finding);
        }

        // Summary
        report.push_str("\u{1F4CA} Summary:\n");
        for level in [
            RiskLevel::Critical,
            RiskLevel::High,
            RiskLevel::Medium,
            RiskLevel::Low,
        ] {
            if let Some(findings) = by_risk.get(&level) {
                let icon = match level {
                    RiskLevel::Critical => "\u{1F6A8}",
                    RiskLevel::High => "\u{26A0}\u{FE0F}",
                    RiskLevel::Medium => "\u{1F7E1}",
                    RiskLevel::Low => "\u{1F535}",
                };
                report.push_str(&format!(
                    "  {} {}: {} findings\n",
                    icon,
                    level,
                    findings.len()
                ));
            }
        }
        report.push('\n');

        // Detailed findings by risk level
        for level in [
            RiskLevel::Critical,
            RiskLevel::High,
            RiskLevel::Medium,
            RiskLevel::Low,
        ] {
            if let Some(findings) = by_risk.get(&level) {
                let header = match level {
                    RiskLevel::Critical => "\u{1F6A8} CRITICAL RISK",
                    RiskLevel::High => "\u{26A0}\u{FE0F} HIGH RISK",
                    RiskLevel::Medium => "\u{1F7E1} MEDIUM RISK",
                    RiskLevel::Low => "\u{1F535} LOW RISK",
                };
                report.push_str(&format!("\n{}\n", header));
                report.push_str(&"-".repeat(60));
                report.push('\n');

                // Group by pattern name
                let mut by_pattern: HashMap<&str, Vec<&&SecurityFinding>> = HashMap::new();
                for finding in findings {
                    by_pattern
                        .entry(&finding.pattern_name)
                        .or_default()
                        .push(finding);
                }

                for (pattern_name, pattern_findings) in by_pattern {
                    report.push_str(&format!(
                        "\n  \u{1F50E} {} ({} occurrences)\n",
                        pattern_name,
                        pattern_findings.len()
                    ));
                    report.push_str(&format!("     {}\n", pattern_findings[0].description));

                    // Show first 5 files
                    for (i, finding) in pattern_findings.iter().take(5).enumerate() {
                        let short_path = finding.file_path.to_string_lossy();
                        // Truncate long paths
                        let display_path = if short_path.len() > 60 {
                            format!("...{}", &short_path[short_path.len() - 57..])
                        } else {
                            short_path.to_string()
                        };
                        report.push_str(&format!(
                            "     {}. {}:{}\n",
                            i + 1,
                            display_path,
                            finding.line_number
                        ));
                        report.push_str(&format!(
                            "        Match: {}\n",
                            truncate(&finding.matched_text, 50)
                        ));
                    }
                    if pattern_findings.len() > 5 {
                        report.push_str(&format!(
                            "     ... and {} more\n",
                            pattern_findings.len() - 5
                        ));
                    }
                }
            }
        }

        // Recommendations
        report.push_str("\n\n\u{1F6E1}\u{FE0F} Recommendations:\n");
        report.push_str("═══════════════════════════════════════════════════════════════\n");

        if by_risk.contains_key(&RiskLevel::Critical) || by_risk.contains_key(&RiskLevel::High) {
            report.push_str("  1. Run: st --ai-install --cleanup\n");
            report.push_str("     To review and remove untrusted MCP integrations\n\n");
            report.push_str("  2. Manually audit ~/.claude/settings.json\n");
            report.push_str("     Remove any hooks referencing suspicious packages\n\n");
            report.push_str("  3. Delete ~/.claude/commands/ directories with risky content\n");
            report.push_str("     These are active skills that execute on slash commands\n\n");
            report.push_str("  4. DO NOT reinstall the flagged packages from npm\n");
            report.push_str("     They will re-add themselves to your configuration\n");
        } else {
            report.push_str("  No critical actions required.\n");
            report.push_str("  Continue monitoring for new patterns.\n");
        }

        report
    }
}

impl Default for SecurityScanner {
    fn default() -> Self {
        Self::new()
    }
}

fn truncate(s: &str, max_len: usize) -> String {
    if s.len() > max_len {
        format!("{}...", &s[..max_len - 3])
    } else {
        s.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ipfs_detection() {
        let scanner = SecurityScanner::new();
        let content = r#"const url = "https://ipfs.io/ipfs/QmTest";"#;
        let mut findings = Vec::new();
        scanner.scan_content(Path::new("test.js"), content, &mut findings);
        assert!(!findings.is_empty());
        assert_eq!(findings[0].pattern_name, "IPFS Gateway");
    }

    #[test]
    fn test_claude_flow_detection() {
        let scanner = SecurityScanner::new();
        let content = "npx claude-flow@alpha swarm init";
        let mut findings = Vec::new();
        scanner.scan_content(Path::new("test.md"), content, &mut findings);
        assert!(findings
            .iter()
            .any(|f| f.pattern_name == "Known Risk Package"));
    }

    #[test]
    fn test_fake_verification_detection() {
        let scanner = SecurityScanner::new();
        let content = "return signature.length === 64;";
        let mut findings = Vec::new();
        scanner.scan_content(Path::new("verify.ts"), content, &mut findings);
        assert!(findings
            .iter()
            .any(|f| f.pattern_name == "Fake Verification"));
    }
}
