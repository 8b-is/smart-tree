use std::path::Path;
use std::sync::Arc;
use std::time::Instant;

use rayon::prelude::*;
use sha2::{Digest, Sha256};
use walkdir::WalkDir;

use crate::magiscanner::analyzers::{AnalysisContext, Analyzer};
use crate::magiscanner::dish::Dish;
use crate::magiscanner::finding::{Finding, ScanReport};
use crate::magiscanner::operation::url_extract::ExtractUrls;
use crate::magiscanner::operation::Operation;
use crate::magiscanner::recipe::Recipe;

/// Top-level orchestrator: reads files, runs recipe pipeline, runs analyzers, collects findings.
/// Analyzers run in parallel via rayon. Directory scans process files in parallel.
pub struct Scanner {
    pub recipe: Recipe,
    pub analyzers: Vec<Box<dyn Analyzer>>,
}

#[derive(Debug, thiserror::Error)]
pub enum ScanError {
    #[error("file not found: {0}")]
    FileNotFound(String),
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
    #[error("recipe error: {0}")]
    Recipe(#[from] crate::magiscanner::recipe::RecipeError),
    #[error("analyzer '{name}' failed: {source}")]
    Analyzer { name: String, source: anyhow::Error },
}

impl Scanner {
    pub fn new(recipe: Recipe, analyzers: Vec<Box<dyn Analyzer>>) -> Self {
        Self { recipe, analyzers }
    }

    /// Scan a single file. Analyzers run in parallel across threads.
    pub fn scan_file(&self, path: &Path) -> Result<ScanReport, ScanError> {
        if !path.exists() {
            return Err(ScanError::FileNotFound(path.display().to_string()));
        }

        let start = Instant::now();
        let raw_content = std::fs::read(path)?;
        let file_size = raw_content.len() as u64;

        // Hash the file
        let sha256 = {
            let mut hasher = Sha256::new();
            hasher.update(&raw_content);
            format!("{:x}", hasher.finalize())
        };

        let file_name = path
            .file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_default();
        let file_path = path.display().to_string();

        // Run the recipe pipeline
        let dish = Dish::new(raw_content.clone());
        let processed = self.recipe.execute(dish)?;
        let processed_content = processed.into_bytes();

        // Extract URLs from both raw and processed content for analyzer context
        let url_op = ExtractUrls;
        let extracted_urls = {
            let mut urls = Vec::new();
            if let Ok(raw_urls) = url_op.run(&raw_content, &std::collections::HashMap::new()) {
                let raw_url_str = String::from_utf8_lossy(&raw_urls);
                urls.extend(raw_url_str.lines().map(|s| s.to_string()));
            }
            if let Ok(proc_urls) = url_op.run(&processed_content, &std::collections::HashMap::new())
            {
                let proc_url_str = String::from_utf8_lossy(&proc_urls);
                for url in proc_url_str.lines() {
                    if !urls.contains(&url.to_string()) {
                        urls.push(url.to_string());
                    }
                }
            }
            urls
        };

        // Build analysis context (Arc-wrapped for sharing across threads)
        let context = Arc::new(AnalysisContext {
            file_path: file_path.clone(),
            file_name: file_name.clone(),
            sha256: sha256.clone(),
            extracted_urls,
            raw_content,
            processed_content,
        });

        // Run all analyzers in parallel
        let findings: Vec<Finding> = self
            .analyzers
            .par_iter()
            .flat_map(|analyzer| match analyzer.analyze(&context) {
                Ok(f) => f,
                Err(e) => {
                    tracing::warn!(analyzer = analyzer.name(), error = %e, "analyzer failed");
                    Vec::new()
                }
            })
            .collect();

        let scan_duration_ms = start.elapsed().as_millis() as u64;

        Ok(ScanReport {
            file_path,
            file_name,
            sha256,
            file_size,
            scan_duration_ms,
            findings,
        })
    }

    /// Scan all files in a directory recursively. Files are processed in parallel.
    pub fn scan_dir(&self, path: &Path) -> Result<Vec<ScanReport>, ScanError> {
        if !path.is_dir() {
            return Err(ScanError::FileNotFound(path.display().to_string()));
        }

        // Collect file paths first, then scan in parallel
        let files: Vec<_> = WalkDir::new(path)
            .follow_links(false)
            .into_iter()
            .flatten()
            .filter(|e| e.file_type().is_file())
            .map(|e| e.into_path())
            .collect();

        let reports: Vec<ScanReport> = files
            .par_iter()
            .filter_map(|file_path| match self.scan_file(file_path) {
                Ok(report) => Some(report),
                Err(e) => {
                    tracing::warn!(file = %file_path.display(), error = %e, "scan failed");
                    None
                }
            })
            .collect();

        Ok(reports)
    }
}
