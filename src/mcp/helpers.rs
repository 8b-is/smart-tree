//! Helper functions to reduce code duplication in MCP tools

use super::{is_path_allowed, McpContext};
use crate::scanner::{FileNode, TreeStats};
use crate::{Scanner, ScannerConfig};
use anyhow::{anyhow, Result};
use regex::Regex;
use serde_json::Value;
use std::path::{Component, Path, PathBuf};
use std::time::SystemTime;

/// Argument keys that carry a single filesystem path. Normalized on every
/// MCP tool call so agents can pass `~/...`, `$VAR`, or relative paths freely.
const PATH_KEYS: &[&str] = &[
    "path",
    "file_path",
    "project_path",
    "path1",
    "path2",
    "directory",
    "dir",
];

/// Keys that, when present, set the session's remembered current path.
/// `file_path` is excluded here and handled specially (we remember its parent).
const SESSION_DIR_KEYS: &[&str] = &["path", "project_path", "directory", "dir"];

/// Expand `~`/`~user`/`$VAR` and resolve relative inputs against `base`.
///
/// Returns an absolute, lexically-cleaned path. Does NOT touch the filesystem
/// (no `canonicalize`) so it works for paths that don't exist yet, e.g. a
/// `create_file` target, and never follows symlinks unexpectedly.
pub fn resolve_path(input: &str, base: &Path) -> PathBuf {
    // Expand `~` and environment variables. If env expansion fails (e.g. an
    // undefined `$VAR`), fall back to bare tilde expansion, then to the raw input.
    let expanded = shellexpand::full(input)
        .map(|c| c.into_owned())
        .unwrap_or_else(|_| shellexpand::tilde(input).into_owned());

    let p = PathBuf::from(expanded);
    let abs = if p.is_absolute() { p } else { base.join(p) };
    normalize_lexically(&abs)
}

/// Resolve `.` and `..` purely lexically, without consulting the filesystem.
fn normalize_lexically(path: &Path) -> PathBuf {
    let mut out = PathBuf::new();
    for comp in path.components() {
        match comp {
            Component::CurDir => {}
            Component::ParentDir => {
                // Pop a normal segment; keep `..` only when there's nothing
                // poppable and we're not already anchored at a root/prefix.
                if !out.pop() && !out.has_root() {
                    out.push("..");
                }
            }
            other => out.push(other.as_os_str()),
        }
    }
    out
}

/// Normalize all path-bearing arguments of an MCP tool call in place, then
/// remember the directory used as the session's current path.
///
/// This is the single choke point that makes `~/`, `$HOME`, and relative paths
/// "just work" for every tool, and lets agents omit the path on follow-up calls.
pub fn normalize_path_args(args: &mut Value, ctx: &McpContext) {
    let base = ctx
        .cwd
        .lock()
        .map(|g| g.clone())
        .unwrap_or_else(|_| PathBuf::from("."));

    // Coerce a missing/null argument payload into an empty object so a default
    // path can still be injected for directory tools called with no arguments.
    if args.is_null() {
        *args = Value::Object(serde_json::Map::new());
    }

    let Some(obj) = args.as_object_mut() else {
        return;
    };

    let mut new_cwd: Option<PathBuf> = None;

    for key in PATH_KEYS {
        if let Some(s) = obj.get(*key).and_then(Value::as_str) {
            let resolved = resolve_path(s, &base);

            // First session-defining key wins as the remembered directory.
            if new_cwd.is_none() {
                if SESSION_DIR_KEYS.contains(key) {
                    new_cwd = Some(resolved.clone());
                } else if *key == "file_path" {
                    // Remember the file's parent directory.
                    new_cwd = Some(
                        resolved
                            .parent()
                            .map(Path::to_path_buf)
                            .unwrap_or_else(|| resolved.clone()),
                    );
                }
            }

            obj.insert(
                key.to_string(),
                Value::String(resolved.to_string_lossy().into_owned()),
            );
        }
    }

    // Normalize a `paths` array (e.g. multi-path tools) without affecting cwd.
    if let Some(Value::Array(arr)) = obj.get_mut("paths") {
        for item in arr.iter_mut() {
            if let Some(s) = item.as_str() {
                let resolved = resolve_path(s, &base);
                *item = Value::String(resolved.to_string_lossy().into_owned());
            }
        }
    }

    // If the call carries no path at all, default `path` to the remembered
    // session directory so agents can omit it on follow-up calls. Tools that
    // don't take a path simply ignore the extra field.
    let has_any_path = PATH_KEYS.iter().any(|k| obj.contains_key(*k)) || obj.contains_key("paths");
    if !has_any_path {
        obj.insert(
            "path".to_string(),
            Value::String(base.to_string_lossy().into_owned()),
        );
    }

    if let Some(dir) = new_cwd {
        // Only follow into an actual directory; if the path is a known file,
        // keep its parent (already handled above for `file_path`).
        let dir = if dir.is_file() {
            dir.parent().map(Path::to_path_buf).unwrap_or(dir)
        } else {
            dir
        };
        if let Ok(mut guard) = ctx.cwd.lock() {
            *guard = dir;
        }
    }
}

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

#[cfg(test)]
mod path_tests {
    use super::*;

    #[test]
    fn tilde_expands_to_home() {
        let home = dirs::home_dir().expect("home dir");
        let resolved = resolve_path("~/source/foo", Path::new("/some/base"));
        assert_eq!(resolved, home.join("source/foo"));
    }

    #[test]
    fn bare_tilde_is_home() {
        let home = dirs::home_dir().expect("home dir");
        assert_eq!(resolve_path("~", Path::new("/base")), home);
    }

    #[test]
    fn relative_resolves_against_base() {
        assert_eq!(
            resolve_path("src/main.rs", Path::new("/work/proj")),
            PathBuf::from("/work/proj/src/main.rs")
        );
    }

    #[test]
    fn dot_resolves_to_base() {
        assert_eq!(
            resolve_path(".", Path::new("/work/proj")),
            PathBuf::from("/work/proj")
        );
    }

    #[test]
    fn absolute_passes_through_cleaned() {
        assert_eq!(
            resolve_path("/a/b/../c", Path::new("/ignored")),
            PathBuf::from("/a/c")
        );
    }

    #[test]
    fn parent_dir_in_relative_collapses() {
        assert_eq!(
            resolve_path("../sibling", Path::new("/work/proj")),
            PathBuf::from("/work/sibling")
        );
    }

    #[test]
    fn env_var_expands() {
        // $HOME is reliably set in test environments.
        if let Some(home) = dirs::home_dir() {
            // Only assert when the env var matches dirs (CI parity).
            if std::env::var("HOME").map(PathBuf::from).ok() == Some(home.clone()) {
                assert_eq!(resolve_path("$HOME/x", Path::new("/base")), home.join("x"));
            }
        }
    }
}
