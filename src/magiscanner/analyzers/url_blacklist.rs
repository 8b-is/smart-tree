use crate::magiscanner::analyzers::{AnalysisContext, Analyzer};
use crate::magiscanner::finding::{Finding, FindingKind, Severity};

/// Checks extracted URLs against a blacklist.
/// The blacklist is provided as a list of patterns at construction time.
pub struct UrlBlacklistAnalyzer {
    patterns: Vec<String>,
}

impl UrlBlacklistAnalyzer {
    pub fn new(patterns: Vec<String>) -> Self {
        Self { patterns }
    }

    fn matches(&self, url: &str) -> Option<&str> {
        let url_lower = url.to_lowercase();
        for pattern in &self.patterns {
            let pattern_lower = pattern.to_lowercase();
            // Simple matching: check if the URL contains the pattern
            // Patterns can be domains ("evil.com") or partial URLs
            if url_lower.contains(&pattern_lower) {
                return Some(pattern);
            }
        }
        None
    }
}

impl Analyzer for UrlBlacklistAnalyzer {
    fn name(&self) -> &'static str {
        "url_blacklist"
    }

    fn analyze(&self, context: &AnalysisContext) -> Result<Vec<Finding>, anyhow::Error> {
        let mut findings = Vec::new();

        for url in &context.extracted_urls {
            if let Some(pattern) = self.matches(url) {
                findings.push(Finding {
                    kind: FindingKind::MaliciousUrl { url: url.clone() },
                    severity: Severity::High,
                    description: format!("URL matches blacklist pattern '{pattern}': {url}"),
                    offset: None,
                    evidence: Some(url.clone()),
                });
            }
        }

        Ok(findings)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_context(urls: Vec<&str>) -> AnalysisContext {
        AnalysisContext {
            file_path: "test.bin".to_string(),
            file_name: "test.bin".to_string(),
            sha256: "abc123".to_string(),
            extracted_urls: urls.into_iter().map(|s| s.to_string()).collect(),
            raw_content: vec![],
            processed_content: vec![],
        }
    }

    #[test]
    fn test_blacklist_match() {
        let analyzer = UrlBlacklistAnalyzer::new(vec!["evil.com".to_string()]);
        let ctx = make_context(vec!["https://evil.com/payload", "https://good.com"]);
        let findings = analyzer.analyze(&ctx).unwrap();
        assert_eq!(findings.len(), 1);
        assert!(
            matches!(&findings[0].kind, FindingKind::MaliciousUrl { url } if url.contains("evil.com"))
        );
    }

    #[test]
    fn test_no_matches() {
        let analyzer = UrlBlacklistAnalyzer::new(vec!["evil.com".to_string()]);
        let ctx = make_context(vec!["https://good.com", "https://safe.org"]);
        let findings = analyzer.analyze(&ctx).unwrap();
        assert!(findings.is_empty());
    }
}
