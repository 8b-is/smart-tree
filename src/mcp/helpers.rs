//! Helper functions to reduce code duplication in MCP tools

use super::{is_path_allowed, McpContext};
use crate::scanner::{FileNode, TreeStats};
use crate::{Scanner, ScannerConfig};
use anyhow::{anyhow, Result};
use regex::Regex;
use std::path::{Path, PathBuf};
use std::time::SystemTime;

/// Helper to determine if we should use default ignores
/// We disable them for /tmp paths to support testing
pub fn should_use_default_ignores(path: &Path) -> bool {
    !path.starts_with("/tmp")
}

/// Validate path access and convert to PathBuf
pub fn validate_and_convert_path(path: &str, ctx: &McpContext) -> Result<PathBuf> {
    let path = PathBuf::from(path);
    if !is_path_allowed(&path, &ctx.config) {
        return Err(anyhow!("Access denied: path not allowed"));
    }
    Ok(path)
}

/// Check if a path is accessible (security check)
///
/// This helper is available for use in MCP tools that need standalone path validation.
/// For path conversion + validation, use `validate_and_convert_path` instead.
#[allow(dead_code)]
pub fn check_path_access(path: &Path, ctx: &McpContext) -> Result<()> {
    if !is_path_allowed(path, &ctx.config) {
        return Err(anyhow!("Access denied: path not allowed"));
    }
    Ok(())
}

/// Check if multiple paths are accessible (security check)
///
/// This helper is available for use in MCP tools that need to validate multiple paths.
#[allow(dead_code)]
pub fn check_paths_access(paths: &[&Path], ctx: &McpContext) -> Result<()> {
    for path in paths {
        if !is_path_allowed(path, &ctx.config) {
            return Err(anyhow!("Access denied: path not allowed"));
        }
    }
    Ok(())
}

/// Builder for common ScannerConfig patterns
pub struct ScannerConfigBuilder {
    config: ScannerConfig,
}

impl ScannerConfigBuilder {
    /// Create a new builder with default MCP settings
    pub fn new() -> Self {
        Self {
            config: ScannerConfig {
                max_depth: 100,
                follow_symlinks: false,
                respect_gitignore: true,
                show_hidden: false,
                show_ignored: false,
                find_pattern: None,
                file_type_filter: None,
                entry_type_filter: None,
                min_size: None,
                max_size: None,
                newer_than: None,
                older_than: None,
                use_default_ignores: true,
                search_keyword: None,
                show_filesystems: false,
                sort_field: None,
                top_n: None,
                include_line_content: false,
                // Smart scanning options (disabled by default for MCP)
                compute_interest: false,
                security_scan: false,
                min_interest: 0.0,
                track_traversal: false,
                changes_only: false,
                compare_state: None,
                smart_mode: false,
            },
        }
    }

    /// Create a config optimized for search operations
    pub fn for_search(path: &Path) -> Self {
        let mut builder = Self::new();
        builder.config.max_depth = 10;
        builder.config.use_default_ignores = should_use_default_ignores(path);
        builder
    }

    /// Create a config for quick tree operations
    ///
    /// This preset is available for tools that need a shallow directory scan.
    #[allow(dead_code)]
    pub fn for_quick_tree(path: &Path) -> Self {
        let mut builder = Self::new();
        builder.config.max_depth = 3;
        builder.config.use_default_ignores = should_use_default_ignores(path);
        builder
    }

    /// Create a config for statistics
    pub fn for_stats(path: &Path) -> Self {
        let mut builder = Self::new();
        builder.config.use_default_ignores = should_use_default_ignores(path);
        builder
    }

    pub fn max_depth(mut self, depth: usize) -> Self {
        self.config.max_depth = depth;
        self
    }

    pub fn show_hidden(mut self, show: bool) -> Self {
        self.config.show_hidden = show;
        self
    }

    pub fn show_ignored(mut self, show: bool) -> Self {
        self.config.show_ignored = show;
        self
    }

    pub fn respect_gitignore(mut self, respect: bool) -> Self {
        self.config.respect_gitignore = respect;
        self
    }

    pub fn find_pattern(mut self, pattern: Option<Regex>) -> Self {
        self.config.find_pattern = pattern;
        self
    }

    pub fn file_type_filter(mut self, filter: Option<String>) -> Self {
        self.config.file_type_filter = filter;
        self
    }

    pub fn entry_type_filter(mut self, filter: Option<String>) -> Self {
        self.config.entry_type_filter = filter;
        self
    }

    pub fn min_size(mut self, size: Option<u64>) -> Self {
        self.config.min_size = size;
        self
    }

    pub fn max_size(mut self, size: Option<u64>) -> Self {
        self.config.max_size = size;
        self
    }

    pub fn newer_than(mut self, time: Option<SystemTime>) -> Self {
        self.config.newer_than = time;
        self
    }

    pub fn older_than(mut self, time: Option<SystemTime>) -> Self {
        self.config.older_than = time;
        self
    }

    pub fn search_keyword(mut self, keyword: Option<String>) -> Self {
        self.config.search_keyword = keyword;
        self
    }

    pub fn include_line_content(mut self, include: bool) -> Self {
        self.config.include_line_content = include;
        self
    }

    pub fn use_default_ignores(mut self, use_defaults: bool) -> Self {
        self.config.use_default_ignores = use_defaults;
        self
    }

    pub fn build(self) -> ScannerConfig {
        self.config
    }
}

impl Default for ScannerConfigBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// Scan a directory with the given configuration
/// Returns (nodes, stats) tuple
pub fn scan_with_config(path: &Path, config: ScannerConfig) -> Result<(Vec<FileNode>, TreeStats)> {
    let scanner = Scanner::new(path, config)?;
    scanner.scan()
}
