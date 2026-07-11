//! High-level security scan and certificate audit orchestration.

use std::collections::HashMap;
use std::path::Path;

use anyhow::{bail, Context, Result};
use colored::Colorize;

use crate::magiscanner::analyzers::certificate::{CertDistrust, CertificateAnalyzer};
use crate::magiscanner::analyzers::custom_rules::{CustomRule, CustomRuleAnalyzer, RuleKind};
use crate::magiscanner::analyzers::llm_injection::LlmInjectionAnalyzer;
use crate::magiscanner::analyzers::suspicious_patterns::SuspiciousPatternAnalyzer;
use crate::magiscanner::analyzers::system_certs::{
    audit_system_certs, generate_blacklist_script, scan_system_certs, SystemCertInfo,
};
use crate::magiscanner::analyzers::telemetry::TelemetryAnalyzer;
use crate::magiscanner::analyzers::url_blacklist::UrlBlacklistAnalyzer;
use crate::magiscanner::analyzers::Analyzer;
use crate::magiscanner::config::SecurityConfig;
use crate::magiscanner::db::Database;
use crate::magiscanner::finding::{ScanReport, Severity};
use crate::magiscanner::operation::OperationRegistry;
use crate::magiscanner::quarantine;
use crate::magiscanner::recipe::{Recipe, RecipeStep};
use crate::magiscanner::scanner::Scanner;

/// Result of a certificate trust audit.
#[derive(Debug, serde::Serialize)]
pub struct CertAuditResult {
    pub total_certs: usize,
    pub flagged: Vec<SystemCertInfo>,
    pub distrusted_countries: Vec<String>,
    pub distrusted_orgs: Vec<String>,
}

/// Scan a file or directory, persist results, and return reports.
pub fn scan_path(
    config: &SecurityConfig,
    path: &Path,
    recursive: bool,
    recipe_override: Option<&str>,
) -> Result<Vec<ScanReport>> {
    if !path.exists() {
        bail!("Path not found: {}", path.display());
    }

    let db = Database::open(&config.db_path()).context("Failed to open security database")?;
    let scanner = build_scanner(config, &db, recipe_override)?;

    let reports = if path.is_file() {
        vec![scanner.scan_file(path)?]
    } else if recursive {
        scanner.scan_dir(path)?
    } else {
        bail!(
            "{} is a directory; use --recursive to scan it",
            path.display()
        );
    };

    for report in &reports {
        persist_report(config, &db, report)?;
    }

    Ok(reports)
}

/// Audit the system CA trust store against distrust policies.
pub fn audit_system_certificates(config: &SecurityConfig) -> Result<CertAuditResult> {
    let db = Database::open(&config.db_path()).context("Failed to open security database")?;

    let mut distrusted_countries = config.certificates.distrusted_countries.clone();
    distrusted_countries.extend(db.get_distrusted_countries()?);
    distrusted_countries.sort();
    distrusted_countries.dedup();

    let mut distrusted_orgs = config.certificates.distrusted_orgs.clone();
    distrusted_orgs.extend(db.get_distrusted_orgs()?);
    distrusted_orgs.sort();
    distrusted_orgs.dedup();

    let extra_dirs: Vec<String> = config.certificates.system_cert_dirs.clone();
    let all_certs = scan_system_certs(&extra_dirs)?;
    let flagged = audit_system_certs(&all_certs, &distrusted_countries, &distrusted_orgs);

    Ok(CertAuditResult {
        total_certs: all_certs.len(),
        flagged,
        distrusted_countries,
        distrusted_orgs,
    })
}

/// Generate a shell script to blacklist flagged certificates.
pub fn cert_blacklist_script(flagged: &[SystemCertInfo]) -> String {
    generate_blacklist_script(flagged)
}

fn build_scanner(
    config: &SecurityConfig,
    db: &Database,
    recipe_override: Option<&str>,
) -> Result<Scanner> {
    let recipe = build_recipe(config, recipe_override)?;

    let blacklist_patterns = db.get_blacklist_patterns()?;
    let mut analyzers: Vec<Box<dyn Analyzer>> = vec![
        Box::new(UrlBlacklistAnalyzer::new(blacklist_patterns)),
        Box::new(LlmInjectionAnalyzer::new()),
        Box::new(TelemetryAnalyzer::new()),
        Box::new(SuspiciousPatternAnalyzer::new()),
    ];

    if config.certificates.enabled {
        let mut distrusted_countries = config.certificates.distrusted_countries.clone();
        distrusted_countries.extend(db.get_distrusted_countries()?);
        distrusted_countries.sort();
        distrusted_countries.dedup();

        let mut distrusted_orgs = config.certificates.distrusted_orgs.clone();
        distrusted_orgs.extend(db.get_distrusted_orgs()?);
        distrusted_orgs.sort();
        distrusted_orgs.dedup();

        let approved = db.get_approved_fingerprints()?;

        analyzers.push(Box::new(CertificateAnalyzer::new(CertDistrust {
            country_codes: distrusted_countries,
            org_patterns: distrusted_orgs,
            require_approval: config.certificates.require_approval,
            approved_fingerprints: approved,
        })));
    }

    let custom_rule_rows = db.get_enabled_custom_rules()?;
    if !custom_rule_rows.is_empty() {
        let rules: Vec<CustomRule> = custom_rule_rows
            .into_iter()
            .filter_map(|row| {
                let severity = row.severity.parse().unwrap_or(Severity::High);
                let description = row.description.unwrap_or_default();
                let kind = match row.kind.as_str() {
                    "regex" => regex::Regex::new(&row.pattern)
                        .ok()
                        .map(|r| RuleKind::Regex { pattern: r }),
                    "tld" => Some(RuleKind::Tld { tld: row.pattern }),
                    "company" => Some(RuleKind::Company { name: row.pattern }),
                    "ipfs" => Some(RuleKind::Ipfs),
                    "password_guard" => Some(RuleKind::PasswordGuard {
                        argon2_hash: row.pattern,
                    }),
                    _ => None,
                };
                kind.map(|k| CustomRule {
                    name: row.name,
                    kind: k,
                    severity,
                    description,
                })
            })
            .collect();

        if !rules.is_empty() {
            analyzers.push(Box::new(CustomRuleAnalyzer::new(rules)));
        }
    }

    Ok(Scanner::new(recipe, analyzers))
}

fn build_recipe(config: &SecurityConfig, override_str: Option<&str>) -> Result<Recipe> {
    let registry = OperationRegistry::new();
    let mut recipe = Recipe::new();

    let op_names: Vec<String> = if let Some(s) = override_str {
        s.split(',').map(|s| s.trim().to_string()).collect()
    } else {
        config.scan.default_recipe.clone()
    };

    for name in &op_names {
        if let Some(op) = registry.create(name) {
            recipe.add_step(RecipeStep {
                operation: op,
                args: HashMap::new(),
                disabled: false,
            });
        } else {
            tracing::warn!(operation = name, "unknown operation, skipping");
        }
    }

    Ok(recipe)
}

fn persist_report(config: &SecurityConfig, db: &Database, report: &ScanReport) -> Result<()> {
    let scan_id = db.insert_scan(report)?;
    let max_severity = report.findings.iter().map(|f| f.severity).max();

    if let Ok(Some(prior)) = db.get_hash_action(&report.sha256) {
        tracing::info!(
            sha256 = %report.sha256,
            times_seen = prior.times_seen,
            action = %prior.action,
            "known file hash"
        );
        db.touch_hash(&report.sha256)?;
    } else {
        let action = match max_severity {
            Some(Severity::Critical) | Some(Severity::High) => "flag",
            _ if report.findings.is_empty() => "allow",
            _ => "flag",
        };
        db.set_hash_action(
            &report.sha256,
            action,
            Some(&report.file_name),
            None,
            max_severity.map(|s| s.to_string()).as_deref(),
        )?;
    }

    if config.quarantine.enabled {
        let threshold: Severity = config
            .quarantine
            .auto_quarantine_severity
            .parse()
            .unwrap_or(Severity::High);

        if let Some(max) = max_severity {
            if max >= threshold {
                let quarantine_dir = quarantine::resolve_path(&config.quarantine.directory);
                if let Ok(qr) =
                    quarantine::quarantine_file(Path::new(&report.file_path), &quarantine_dir)
                {
                    let reason = format!(
                        "Auto-quarantined: {} finding(s), max severity {}",
                        report.findings.len(),
                        max
                    );
                    db.insert_quarantine(
                        &qr.original_path,
                        &qr.quarantine_path,
                        &qr.sha256,
                        qr.file_size as i64,
                        &reason,
                        &max.to_string(),
                        Some(scan_id),
                    )?;
                    tracing::warn!(
                        path = %qr.quarantine_path,
                        "file quarantined"
                    );
                }
            }
        }
    }

    Ok(())
}

/// Print scan reports to stdout (human-readable).
pub fn print_reports(reports: &[ScanReport]) {
    for report in reports {
        println!("\n{} {}", "Scanned:".bold(), report.file_path);
        println!(
            "  SHA256: {}  Size: {} bytes  Duration: {}ms",
            &report.sha256[..16],
            report.file_size,
            report.scan_duration_ms
        );

        if report.findings.is_empty() {
            println!("  {}", "OK — no findings".green().bold());
        } else {
            println!(
                "  {} {} finding(s)",
                "!!".red().bold(),
                report.findings.len()
            );
            for finding in &report.findings {
                let severity = match finding.severity {
                    Severity::Critical => finding.severity.to_string().red().bold(),
                    Severity::High => finding.severity.to_string().red(),
                    Severity::Medium => finding.severity.to_string().yellow(),
                    Severity::Low => finding.severity.to_string().blue(),
                    Severity::Info => finding.severity.to_string().normal(),
                };
                let evidence = finding
                    .evidence
                    .as_deref()
                    .unwrap_or("-")
                    .chars()
                    .take(80)
                    .collect::<String>();
                println!("    [{severity}] {} — {evidence}", finding.description);
            }
        }
    }

    let total_findings: usize = reports.iter().map(|r| r.findings.len()).sum();
    println!(
        "\n{} file(s) scanned, {} finding(s) total",
        reports.len(),
        total_findings
    );
}

/// Print certificate audit results to stdout.
pub fn print_cert_audit(result: &CertAuditResult) {
    println!(
        "\n{} {} CA certificates in system trust store",
        "Found:".bold(),
        result.total_certs
    );

    if result.flagged.is_empty() {
        println!("{}", "OK — no distrusted certificates found".green().bold());
        return;
    }

    println!(
        "\n{} {} certificate(s) match distrust policies:",
        "!!".red().bold(),
        result.flagged.len()
    );

    for cert in &result.flagged {
        println!(
            "  {} (C={}, O={}) — {}",
            cert.subject_cn.bold(),
            cert.issuer_country,
            cert.issuer_org,
            &cert.fingerprint_sha256[..16]
        );
        println!("    {}", cert.path.dimmed());
    }
}

/// Look up a known hash in the security database.
pub fn lookup_hash(
    config: &SecurityConfig,
    sha256: &str,
) -> Result<Option<crate::magiscanner::db::models::HashActionRow>> {
    let db = Database::open(&config.db_path())?;
    db.get_hash_action(sha256).map_err(Into::into)
}
