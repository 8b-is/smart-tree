use regex::Regex;
use std::sync::LazyLock;

use crate::magiscanner::analyzers::{AnalysisContext, Analyzer};
use crate::magiscanner::finding::{Finding, FindingKind, Severity};

/// Known telemetry/tracking domains and patterns.
static TELEMETRY_PATTERNS: LazyLock<Vec<(&'static str, Regex)>> = LazyLock::new(|| {
    vec![
        ("Google Analytics", Regex::new(r"(?i)google[_-]?analytics|googletagmanager\.com|ga\('send'|gtag\(").unwrap()),
        ("Facebook Pixel", Regex::new(r"(?i)facebook\.net/en_US/fbevents|fbq\(|connect\.facebook\.net").unwrap()),
        ("Sentry", Regex::new(r"(?i)sentry\.io/api|dsn.*sentry|Sentry\.init").unwrap()),
        ("Mixpanel", Regex::new(r"(?i)mixpanel\.com|mixpanel\.track|mixpanel\.init").unwrap()),
        ("Segment", Regex::new(r"(?i)segment\.com/analytics|analytics\.identify|analytics\.track").unwrap()),
        ("Amplitude", Regex::new(r"(?i)amplitude\.com|amplitude\.getInstance|logEvent").unwrap()),
        ("Hotjar", Regex::new(r"(?i)hotjar\.com|hj\('trigger'|hotjar\.init").unwrap()),
        ("Phone home", Regex::new(r"(?i)phone[_\s-]?home|telemetry[_\s-]?endpoint|beacon[_\s-]?url|tracking[_\s-]?pixel").unwrap()),
    ]
});

/// Detects telemetry and tracking beacons in file content.
pub struct TelemetryAnalyzer;

impl TelemetryAnalyzer {
    pub fn new() -> Self {
        Self
    }
}

impl Default for TelemetryAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}

impl Analyzer for TelemetryAnalyzer {
    fn name(&self) -> &'static str {
        "telemetry"
    }

    fn analyze(&self, context: &AnalysisContext) -> Result<Vec<Finding>, anyhow::Error> {
        let mut findings = Vec::new();
        let text = String::from_utf8_lossy(&context.raw_content);

        for (name, pattern) in TELEMETRY_PATTERNS.iter() {
            for m in pattern.find_iter(&text) {
                findings.push(Finding {
                    kind: FindingKind::Telemetry {
                        endpoint: m.as_str().to_string(),
                    },
                    severity: Severity::Info,
                    description: format!("Telemetry/tracking detected: {name}"),
                    offset: Some(m.start()),
                    evidence: Some(m.as_str().to_string()),
                });
            }
        }

        // Also check extracted URLs for known tracking domains
        let tracking_domains = [
            "google-analytics.com",
            "googletagmanager.com",
            "facebook.net",
            "segment.com",
            "mixpanel.com",
            "amplitude.com",
            "hotjar.com",
            "fullstory.com",
            "clarity.ms",
            "newrelic.com",
            "bugsnag.com",
        ];

        for url in &context.extracted_urls {
            let url_lower = url.to_lowercase();
            for domain in &tracking_domains {
                if url_lower.contains(domain) {
                    findings.push(Finding {
                        kind: FindingKind::Telemetry {
                            endpoint: url.clone(),
                        },
                        severity: Severity::Low,
                        description: format!("URL points to tracking service: {domain}"),
                        offset: None,
                        evidence: Some(url.clone()),
                    });
                }
            }
        }

        Ok(findings)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_context(text: &str, urls: Vec<&str>) -> AnalysisContext {
        AnalysisContext {
            file_path: "test.js".to_string(),
            file_name: "test.js".to_string(),
            sha256: "abc123".to_string(),
            extracted_urls: urls.into_iter().map(|s| s.to_string()).collect(),
            raw_content: text.as_bytes().to_vec(),
            processed_content: vec![],
        }
    }

    #[test]
    fn test_detects_google_analytics() {
        let analyzer = TelemetryAnalyzer::new();
        let ctx = make_context("ga('send', 'pageview');", vec![]);
        let findings = analyzer.analyze(&ctx).unwrap();
        assert!(!findings.is_empty());
    }

    #[test]
    fn test_detects_tracking_url() {
        let analyzer = TelemetryAnalyzer::new();
        let ctx = make_context("", vec!["https://www.google-analytics.com/collect"]);
        let findings = analyzer.analyze(&ctx).unwrap();
        assert!(!findings.is_empty());
    }

    #[test]
    fn test_clean_content() {
        let analyzer = TelemetryAnalyzer::new();
        let ctx = make_context("function add(a, b) { return a + b; }", vec![]);
        let findings = analyzer.analyze(&ctx).unwrap();
        assert!(findings.is_empty());
    }
}
