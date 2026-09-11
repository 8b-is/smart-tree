use std::io::Read;
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
    max_file_size: u64,
    follow_symlinks: bool,
}

#[derive(Debug, thiserror::Error)]
pub enum ScanError {
    #[error("file not found: {0}")]
    FileNotFound(String),
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
    #[error("directory traversal failed: {0}")]
    Walk(#[from] walkdir::Error),
    #[error("recipe error: {0}")]
    Recipe(#[from] crate::magiscanner::recipe::RecipeError),
    #[error("analyzer '{name}' failed: {source}")]
    Analyzer { name: String, source: anyhow::Error },
}

/// Bound reads on the opened file as well as its metadata, including growing
/// files. Nonblocking open avoids hanging on a FIFO substituted during a scan.
pub(crate) fn read_regular_file(
    path: &Path,
    max_bytes: u64,
    follow_symlinks: bool,
) -> std::io::Result<Vec<u8>> {
    let mut options = std::fs::OpenOptions::new();
    options.read(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        options.custom_flags(libc::O_NONBLOCK | if follow_symlinks { 0 } else { libc::O_NOFOLLOW });
    }
    if !follow_symlinks && std::fs::symlink_metadata(path)?.is_symlink() {
        return Err(std::io::Error::other("symlink following is disabled"));
    }
    let file = options.open(path)?;
    let metadata = file.metadata()?;
    if !metadata.is_file() {
        return Err(std::io::Error::other("not a regular file"));
    }
    if metadata.len() > max_bytes {
        return Err(std::io::Error::other(format!(
            "file exceeds {max_bytes} byte scan limit"
        )));
    }
    let mut content = Vec::new();
    file.take(max_bytes.saturating_add(1))
        .read_to_end(&mut content)?;
    if content.len() as u64 > max_bytes {
        return Err(std::io::Error::other(format!(
            "file exceeds {max_bytes} byte scan limit"
        )));
    }
    Ok(content)
}

impl Scanner {
    pub fn new(recipe: Recipe, analyzers: Vec<Box<dyn Analyzer>>) -> Self {
        Self {
            recipe,
            analyzers,
            max_file_size: 100 * 1024 * 1024,
            follow_symlinks: false,
        }
    }

    pub fn with_limits(mut self, max_file_size: u64, follow_symlinks: bool) -> Self {
        self.max_file_size = max_file_size;
        self.follow_symlinks = follow_symlinks;
        self
    }

    /// Scan a single file. Analyzers run in parallel across threads.
    pub fn scan_file(&self, path: &Path) -> Result<ScanReport, ScanError> {
        if !path.exists() {
            return Err(ScanError::FileNotFound(path.display().to_string()));
        }

        let start = Instant::now();
        let raw_content = read_regular_file(path, self.max_file_size, self.follow_symlinks)?;
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
            .map(|analyzer| {
                analyzer
                    .analyze(&context)
                    .map_err(|source| ScanError::Analyzer {
                        name: analyzer.name().to_string(),
                        source,
                    })
            })
            .collect::<Result<Vec<_>, _>>()?
            .into_iter()
            .flatten()
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
            .follow_links(self.follow_symlinks)
            .follow_root_links(self.follow_symlinks)
            .into_iter()
            .collect::<Result<Vec<_>, _>>()?
            .into_iter()
            .filter(|e| e.file_type().is_file())
            .map(|e| e.into_path())
            .collect();

        let reports: Vec<ScanReport> = files
            .par_iter()
            .map(|file_path| self.scan_file(file_path))
            .collect::<Result<Vec<_>, _>>()?;

        Ok(reports)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct FailingAnalyzer;

    impl Analyzer for FailingAnalyzer {
        fn name(&self) -> &'static str {
            "failing-test-analyzer"
        }
        fn analyze(&self, _: &AnalysisContext) -> anyhow::Result<Vec<Finding>> {
            anyhow::bail!("intentional analyzer failure")
        }
    }

    #[test]
    fn analyzer_failure_is_not_reported_as_a_clean_scan() {
        let temp = tempfile::tempdir().unwrap();
        let path = temp.path().join("file.txt");
        std::fs::write(&path, "test").unwrap();
        let scanner = Scanner::new(Recipe::new(), vec![Box::new(FailingAnalyzer)]);
        assert!(matches!(
            scanner.scan_file(&path),
            Err(ScanError::Analyzer { .. })
        ));
        assert!(scanner.scan_dir(temp.path()).is_err());
    }

    #[test]
    fn configured_limit_is_applied_to_direct_and_directory_scans() {
        let temp = tempfile::tempdir().unwrap();
        let path = temp.path().join("file.txt");
        std::fs::write(&path, "12345").unwrap();
        let scanner = Scanner::new(Recipe::new(), vec![]).with_limits(4, false);
        assert!(scanner.scan_file(&path).is_err());
        assert!(scanner.scan_dir(temp.path()).is_err());
    }

    #[cfg(unix)]
    #[test]
    fn refuses_symlinks_and_fifos_without_blocking() {
        let temp = tempfile::tempdir().unwrap();
        let file = temp.path().join("file");
        let link = temp.path().join("link");
        std::fs::write(&file, "fixture").unwrap();
        std::os::unix::fs::symlink(&file, &link).unwrap();
        assert!(read_regular_file(&link, 100, false).is_err());
        assert_eq!(read_regular_file(&link, 100, true).unwrap(), b"fixture");
        let fifo = temp.path().join("fifo");
        use std::os::unix::ffi::OsStrExt;
        let name = std::ffi::CString::new(fifo.as_os_str().as_bytes()).unwrap();
        assert_eq!(unsafe { libc::mkfifo(name.as_ptr(), 0o600) }, 0);
        assert!(read_regular_file(&fifo, 100, false).is_err());
    }
}
