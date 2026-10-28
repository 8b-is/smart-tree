// ST Unified Tool System - Replace all traditional tools with Smart Tree!
// "Why use 20 tools when ST can do it all?" - The Cheet 🎸

use anyhow::{ensure, Context, Result};
use std::path::{Path, PathBuf};
use std::process::Command;

/// Unified ST interface for all file operations
pub struct StUnified {
    st_binary: PathBuf,
}

/// Translate tool glob patterns to the CLI's path regex without changing --find.
pub(crate) fn glob_as_find_regex(pattern: &str) -> Result<String> {
    let glob_path = Path::new(pattern);
    let pattern = if glob_path.is_absolute() {
        // Resolve existing prefixes, retaining components containing wildcards.
        glob_path
            .ancestors()
            .find_map(|ancestor| {
                ancestor.canonicalize().ok().and_then(|base| {
                    glob_path.strip_prefix(ancestor).ok().map(|suffix| {
                        if suffix.as_os_str().is_empty() {
                            base
                        } else {
                            base.join(suffix)
                        }
                        .to_string_lossy()
                        .into_owned()
                    })
                })
            })
            .unwrap_or_else(|| pattern.to_string())
    } else {
        format!("**/{pattern}")
    };
    let glob = globset::GlobBuilder::new(&pattern)
        .literal_separator(true)
        .build()
        .context("Invalid file glob")?;
    // Globset escapes Unicode as UTF-8 bytes; restore scalar literals for
    // the scanner's text regex, including Unicode character classes.
    let mut expression = glob
        .regex()
        .strip_prefix("(?-u)")
        .unwrap_or(glob.regex())
        .to_string();
    for character in pattern.chars().filter(|c| !c.is_ascii()) {
        let literal = character.to_string();
        let bytes = literal
            .bytes()
            .map(|b| format!("\\x{b:02x}"))
            .collect::<String>();
        expression = expression.replace(&bytes, &literal);
    }
    Ok(expression)
}

impl StUnified {
    /// Use an explicit binary, for embedded tools and Cargo integration tests.
    pub fn with_binary(st_binary: impl Into<PathBuf>) -> Result<Self> {
        let st_binary = st_binary.into();
        ensure!(
            st_binary.is_file(),
            "ST binary not found: {}",
            st_binary.display()
        );
        Ok(Self { st_binary })
    }

    pub fn new() -> Result<Self> {
        // Find st binary
        let st_binary = std::env::current_exe()
            .ok()
            .and_then(|p| {
                let dir = p.parent()?;
                let st = dir.join("st");
                if st.exists() {
                    Some(st)
                } else {
                    None
                }
            })
            .unwrap_or_else(|| PathBuf::from("./target/release/st"));

        Ok(Self { st_binary })
    }

    fn command(&self) -> Command {
        let mut command = Command::new(&self.st_binary);
        command.args(["--no-daemon", "--no-update-check"]);
        command
    }

    fn output(&self, command: &mut Command) -> Result<String> {
        let output = command.output().context("Failed to execute st")?;
        ensure!(
            output.status.success(),
            "ST failed: {}",
            String::from_utf8_lossy(&output.stderr)
        );
        Ok(String::from_utf8_lossy(&output.stdout).into_owned())
    }

    /// List files (replaces LS tool)
    pub fn ls(&self, path: &Path, pattern: Option<&str>) -> Result<String> {
        let mut cmd = self.command();
        cmd.arg("--mode")
            .arg("ls")
            .arg("--depth")
            .arg("1")
            .arg("--no-emoji")
            .arg(path);

        if let Some(pat) = pattern {
            cmd.arg("--find").arg(glob_as_find_regex(pat)?);
        }

        self.output(&mut cmd)
    }

    /// Read file (replaces Read tool)
    pub fn read(&self, path: &Path, offset: Option<usize>, limit: Option<usize>) -> Result<String> {
        // ST doesn't read file contents, so use standard fs
        let content = std::fs::read_to_string(path).context("Failed to read file")?;

        let lines: Vec<&str> = content.lines().collect();
        let start = offset.unwrap_or(0);
        let end = start.saturating_add(limit.unwrap_or(lines.len()));

        Ok(lines[start.min(lines.len())..end.min(lines.len())]
            .iter()
            .enumerate()
            .map(|(i, line)| format!("{:6}→{}", start + i + 1, line))
            .collect::<Vec<_>>()
            .join("\n"))
    }

    /// Search in files (replaces Grep tool)
    pub fn grep(&self, pattern: &str, path: &Path, file_type: Option<&str>) -> Result<String> {
        let mut cmd = self.command();
        cmd.arg("--search")
            .arg(pattern)
            .arg("--mode")
            .arg("ai")
            .arg("--depth")
            .arg("0")
            .arg(path);

        if let Some(ft) = file_type {
            cmd.arg("--type").arg(ft);
        }

        self.output(&mut cmd)
    }

    /// Find files by pattern (replaces Glob tool)
    pub fn glob(&self, pattern: &str, path: &Path) -> Result<String> {
        self.output(
            self.command()
                .arg("--find")
                .arg(glob_as_find_regex(pattern)?)
                .arg("--mode")
                .arg("json")
                .arg("--depth")
                .arg("0")
                .arg("--compact")
                .arg(path),
        )
    }

    /// Analyze directory (replaces basic tree viewing)
    pub fn analyze(&self, path: &Path, mode: &str, depth: usize) -> Result<String> {
        self.output(
            self.command()
                .arg("--mode")
                .arg(mode)
                .arg("--depth")
                .arg(depth.to_string())
                .arg(path),
        )
    }

    /// Get file/directory stats
    pub fn stats(&self, path: &Path) -> Result<String> {
        self.output(
            self.command()
                .arg("--mode")
                .arg("stats")
                .arg("--depth")
                .arg("0")
                .arg(path),
        )
    }

    /// Semantic analysis (unique to ST!)
    pub fn semantic_analyze(&self, path: &Path) -> Result<String> {
        self.output(
            self.command()
                .arg("--mode")
                .arg("semantic")
                .arg("--depth")
                .arg("0")
                .arg(path),
        )
    }

    /// Quick overview (replaces quick checks)
    pub fn quick(&self, path: &Path) -> Result<String> {
        self.output(
            self.command()
                .arg("--mode")
                .arg("summary-ai")
                .arg("--depth")
                .arg("3")
                .arg(path),
        )
    }

    /// Project understanding (replaces multiple analysis tools)
    pub fn understand_project(&self, path: &Path) -> Result<String> {
        let results = [
            "=== QUICK OVERVIEW ===".to_string(),
            self.quick(path)?,
            "\n=== SEMANTIC GROUPS ===".to_string(),
            self.semantic_analyze(path)?,
            "\n=== STATISTICS ===".to_string(),
            self.stats(path)?,
        ];

        Ok(results.join("\n"))
    }
}
