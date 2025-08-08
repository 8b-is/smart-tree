// ST Unified Tool System - Replace all traditional tools with Smart Tree!
// "Why use 20 tools when ST can do it all?" - The Cheet 🎸

use std::path::{Path, PathBuf};
use std::process::Command;
use anyhow::{Result, Context};

/// Unified ST interface for all file operations
pub struct StUnified {
    st_binary: PathBuf,
}

impl StUnified {
    pub fn new() -> Result<Self> {
        // Find st binary
        let st_binary = std::env::current_exe()
            .ok()
            .and_then(|p| {
                let dir = p.parent()?;
                let st = dir.join("st");
                if st.exists() { Some(st) } else { None }
            })
            .unwrap_or_else(|| PathBuf::from("./target/release/st"));

        Ok(Self { st_binary })
    }

    /// List files (replaces LS tool)
    pub fn ls(&self, path: &Path, pattern: Option<&str>) -> Result<String> {
        let mut cmd = Command::new(&self.st_binary);
        cmd.arg("--mode").arg("ls")
           .arg("--depth").arg("1")
           .arg("--no-emoji")
           .arg(path);

        if let Some(pat) = pattern {
            cmd.arg("--find").arg(pat);
        }

        let output = cmd.output()
            .context("Failed to run st for ls")?;

        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    }

    /// Read file (replaces Read tool)
    pub fn read(&self, path: &Path, offset: Option<usize>, limit: Option<usize>) -> Result<String> {
        // ST doesn't read file contents, so use standard fs
        let content = std::fs::read_to_string(path)
            .context("Failed to read file")?;

        let lines: Vec<&str> = content.lines().collect();
        let start = offset.unwrap_or(0);
        let end = start + limit.unwrap_or(lines.len());
        
        Ok(lines[start.min(lines.len())..end.min(lines.len())]
            .iter()
            .enumerate()
            .map(|(i, line)| format!("{:6}→{}", start + i + 1, line))
            .collect::<Vec<_>>()
            .join("\n"))
    }

    /// Search in files (replaces Grep tool)
    pub fn grep(&self, pattern: &str, path: &Path, file_type: Option<&str>) -> Result<String> {
        let mut cmd = Command::new(&self.st_binary);
        cmd.arg("--search").arg(pattern)
           .arg("--mode").arg("ai")
           .arg("--depth").arg("0")
           .arg(path);

        if let Some(ft) = file_type {
            cmd.arg("--type").arg(ft);
        }

        let output = cmd.output()
            .context("Failed to run st for search")?;

        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    }

    /// Find files by pattern (replaces Glob tool)
    pub fn glob(&self, pattern: &str, path: &Path) -> Result<String> {
        let output = Command::new(&self.st_binary)
            .arg("--find").arg(pattern)
            .arg("--mode").arg("json")
            .arg("--depth").arg("0")
            .arg("--compact")
            .arg(path)
            .output()
            .context("Failed to run st for glob")?;

        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    }

    /// Analyze directory (replaces basic tree viewing)
    pub fn analyze(&self, path: &Path, mode: &str, depth: usize) -> Result<String> {
        let output = Command::new(&self.st_binary)
            .arg("--mode").arg(mode)
            .arg("--depth").arg(depth.to_string())
            .arg(path)
            .output()
            .context("Failed to run st for analysis")?;

        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    }

    /// Get file/directory stats
    pub fn stats(&self, path: &Path) -> Result<String> {
        let output = Command::new(&self.st_binary)
            .arg("--mode").arg("stats")
            .arg("--depth").arg("0")
            .arg(path)
            .output()
            .context("Failed to run st for stats")?;

        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    }

    /// Semantic analysis (unique to ST!)
    pub fn semantic_analyze(&self, path: &Path) -> Result<String> {
        let output = Command::new(&self.st_binary)
            .arg("--mode").arg("semantic")
            .arg("--depth").arg("0")
            .arg(path)
            .output()
            .context("Failed to run st for semantic analysis")?;

        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    }

    /// Quick overview (replaces quick checks)
    pub fn quick(&self, path: &Path) -> Result<String> {
        let output = Command::new(&self.st_binary)
            .arg("--mode").arg("summary-ai")
            .arg("--depth").arg("3")
            .arg(path)
            .output()
            .context("Failed to run st for quick view")?;

        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    }

    /// Project understanding (replaces multiple analysis tools)
    pub fn understand_project(&self, path: &Path) -> Result<String> {
        let mut results = Vec::new();

        results.push("=== QUICK OVERVIEW ===".to_string());
        results.push(self.quick(path)?);

        results.push("\n=== SEMANTIC GROUPS ===".to_string());
        results.push(self.semantic_analyze(path)?);

        results.push("\n=== STATISTICS ===".to_string());
        results.push(self.stats(path)?);

        Ok(results.join("\n"))
    }
}