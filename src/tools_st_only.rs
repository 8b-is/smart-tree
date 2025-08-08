// Smart Tree Only Tools - Replacing all traditional file tools!
// "One tool to rule them all!" - The Cheet 🎸

use std::path::{Path, PathBuf};
use std::process::Command;
use anyhow::{Result, Context, bail};

/// Configuration for ST-based tools
#[derive(Debug, Clone)]
pub struct StToolsConfig {
    /// Path to st binary
    pub st_binary: PathBuf,
    /// Default mode for operations
    pub default_mode: String,
    /// Whether to use emoji
    pub use_emoji: bool,
    /// Whether to compress output
    pub compress: bool,
}

impl Default for StToolsConfig {
    fn default() -> Self {
        Self {
            st_binary: std::env::current_exe()
                .ok()
                .and_then(|p| {
                    let dir = p.parent()?;
                    let st = dir.join("st");
                    if st.exists() { Some(st) } else { None }
                })
                .unwrap_or_else(|| PathBuf::from("./target/release/st")),
            default_mode: "ai".to_string(),
            use_emoji: false,
            compress: false,
        }
    }
}

/// Main ST-only tools provider
pub struct StOnlyTools {
    config: StToolsConfig,
}

impl StOnlyTools {
    pub fn new() -> Self {
        Self {
            config: StToolsConfig::default(),
        }
    }

    pub fn with_config(config: StToolsConfig) -> Self {
        Self { config }
    }

    /// Core ST execution
    fn run_st(&self, args: Vec<String>) -> Result<String> {
        let mut cmd = Command::new(&self.config.st_binary);
        
        // Add standard flags
        if !self.config.use_emoji {
            cmd.arg("--no-emoji");
        }
        if self.config.compress {
            cmd.arg("--compress");
        }

        // Add provided args
        for arg in args {
            cmd.arg(arg);
        }

        let output = cmd.output()
            .context("Failed to execute st")?;

        if !output.status.success() {
            bail!("ST failed: {}", String::from_utf8_lossy(&output.stderr));
        }

        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    }

    /// List directory contents
    pub fn list(&self, path: &Path, options: ListOptions) -> Result<String> {
        let mut args = vec![
            "--mode".to_string(), 
            "ls".to_string(), 
            "--depth".to_string(), 
            "1".to_string()
        ];

        if let Some(pattern) = &options.pattern {
            args.push("--find".to_string());
            args.push(pattern.clone());
        }

        if let Some(file_type) = &options.file_type {
            args.push("--type".to_string());
            args.push(file_type.clone());
        }

        if let Some(sort) = &options.sort {
            args.push("--sort".to_string());
            args.push(sort.clone());
        }

        if let Some(limit) = options.limit {
            args.push("--top".to_string());
            args.push(limit.to_string());
        }

        args.push(path.to_str().unwrap().to_string());

        self.run_st(args)
    }

    /// Search in files
    pub fn search(&self, pattern: &str, path: &Path, options: SearchOptions) -> Result<String> {
        let mut args = vec![
            "--search".to_string(), 
            pattern.to_string(),
            "--mode".to_string(), 
            "ai".to_string(),
            "--depth".to_string(), 
            "0".to_string(),
        ];

        if let Some(file_type) = &options.file_type {
            args.push("--type".to_string());
            args.push(file_type.clone());
        }

        args.push(path.to_str().unwrap().to_string());

        self.run_st(args)
    }

    /// Get directory overview
    pub fn overview(&self, path: &Path, depth: Option<usize>) -> Result<String> {
        let args = vec![
            "--mode".to_string(), 
            "summary-ai".to_string(),
            "--depth".to_string(), 
            depth.unwrap_or(0).to_string(),
            path.to_str().unwrap().to_string()
        ];
        
        self.run_st(args)
    }

    /// Get statistics
    pub fn stats(&self, path: &Path) -> Result<String> {
        self.run_st(vec![
            "--mode".to_string(), 
            "stats".to_string(),
            "--depth".to_string(), 
            "0".to_string(),
            path.to_str().unwrap().to_string()
        ])
    }

    /// Semantic analysis
    pub fn semantic(&self, path: &Path) -> Result<String> {
        self.run_st(vec![
            "--mode".to_string(), 
            "semantic".to_string(),
            "--depth".to_string(), 
            "0".to_string(),
            path.to_str().unwrap().to_string()
        ])
    }
}

// Options structures
#[derive(Default, Clone)]
pub struct ListOptions {
    pub pattern: Option<String>,
    pub file_type: Option<String>,
    pub sort: Option<String>,
    pub limit: Option<usize>,
}

#[derive(Default, Clone)]
pub struct SearchOptions {
    pub file_type: Option<String>,
    pub show_line_numbers: bool,
    pub case_sensitive: bool,
}