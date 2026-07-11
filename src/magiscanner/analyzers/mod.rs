pub mod certificate;
pub mod custom_rules;
pub mod llm_injection;
pub mod suspicious_patterns;
pub mod system_certs;
pub mod telemetry;
pub mod url_blacklist;

use crate::magiscanner::finding::Finding;

/// Context available to analyzers during analysis.
pub struct AnalysisContext {
    pub file_path: String,
    pub file_name: String,
    pub sha256: String,
    /// URLs extracted during the recipe pipeline (if any).
    pub extracted_urls: Vec<String>,
    /// Raw file content.
    pub raw_content: Vec<u8>,
    /// Content after recipe pipeline processing.
    pub processed_content: Vec<u8>,
}

/// Trait for post-pipeline security analysis.
pub trait Analyzer: Send + Sync {
    fn name(&self) -> &'static str;
    fn analyze(&self, context: &AnalysisContext) -> Result<Vec<Finding>, anyhow::Error>;
}
