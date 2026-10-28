//! Integration helpers for easier Smart Tree usage in other applications
//!
//! This module provides simplified APIs and helper functions that make it easier
//! to integrate Smart Tree functionality into other applications without dealing
//! with all the low-level details.

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use std::time::SystemTime;

use crate::{detect_project_context, FileNode, Scanner, ScannerConfig, TreeStats};

/// Simplified project analysis result optimized for external integrations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectAnalysis {
    pub project_path: PathBuf,
    pub project_type: String,
    pub project_name: String,
    pub total_files: usize,
    pub total_directories: usize,
    pub total_size: u64,
    pub key_files: Vec<String>,
    pub recent_files: Vec<String>,
    pub file_types: std::collections::HashMap<String, usize>,
    pub insights: Vec<String>,
}

/// Quick project analyzer for integration use cases
pub struct ProjectAnalyzer {
    default_config: ScannerConfig,
}

impl Default for ProjectAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}

impl ProjectAnalyzer {
    /// Create a new project analyzer with sensible defaults for integration
    pub fn new() -> Self {
        let config = ScannerConfig {
            max_depth: 10,
            show_hidden: false,
            respect_gitignore: true,
            ..ScannerConfig::default()
        };

        Self {
            default_config: config,
        }
    }

    /// Analyze a project directory and return simplified results
    pub fn analyze_project(&self, project_path: &Path) -> Result<ProjectAnalysis> {
        let scanner = Scanner::new(project_path, self.default_config.clone())?;
        let (nodes, stats) = scanner.scan()?;

        // Detect project type
        let project_type =
            detect_project_context(project_path).unwrap_or_else(|| "Unknown".to_string());

        // Get project name from directory
        let project_name = project_path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("Unknown")
            .to_string();

        // Find key files
        let key_files = Self::extract_key_files(&nodes);

        // Find recent files (last hour)
        let recent_files = Self::find_recent_files(&nodes, 1);

        // Analyze file types
        let file_types = Self::analyze_file_types(&nodes);

        // Generate insights
        let insights = Self::generate_insights(&stats, &project_type, &nodes);

        Ok(ProjectAnalysis {
            project_path: project_path.to_path_buf(),
            project_type,
            project_name,
            total_files: stats.total_files as usize,
            total_directories: stats.total_dirs as usize,
            total_size: stats.total_size,
            key_files,
            recent_files,
            file_types,
            insights,
        })
    }

    /// Quick analysis for dashboard use - limited depth and faster execution
    pub fn quick_analysis(&self, project_path: &Path) -> Result<ProjectAnalysis> {
        let mut config = self.default_config.clone();
        config.max_depth = 2; // Very shallow for quick analysis

        let scanner = Scanner::new(project_path, config)?;
        let (_nodes, stats) = scanner.quick_scan()?;

        let project_type =
            detect_project_context(project_path).unwrap_or_else(|| "Unknown".to_string());

        let project_name = project_path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("Unknown")
            .to_string();

        Ok(ProjectAnalysis {
            project_path: project_path.to_path_buf(),
            project_type: project_type.clone(),
            project_name,
            total_files: stats.total_files as usize,
            total_directories: stats.total_dirs as usize,
            total_size: stats.total_size,
            key_files: vec![],                            // Skip for quick analysis
            recent_files: vec![],                         // Skip for quick analysis
            file_types: std::collections::HashMap::new(), // Skip for quick analysis
            insights: vec![format!(
                "{} project with {} files",
                project_type, stats.total_files
            )],
        })
    }

    /// Find files modified within the last N hours
    pub fn find_recent_activity(&self, project_path: &Path, hours: u64) -> Result<Vec<FileNode>> {
        let scanner = Scanner::new(project_path, self.default_config.clone())?;
        scanner.find_recent_files(hours)
    }

    /// Get only key project files for quick overview
    pub fn get_key_files(&self, project_path: &Path) -> Result<Vec<FileNode>> {
        let scanner = Scanner::new(project_path, self.default_config.clone())?;
        scanner.find_key_files()
    }

    // Helper methods
    fn extract_key_files(nodes: &[FileNode]) -> Vec<String> {
        let important_patterns = [
            "main.rs",
            "lib.rs",
            "mod.rs",
            "package.json",
            "Cargo.toml",
            "requirements.txt",
            "pyproject.toml",
            "README.md",
            "LICENSE",
            "Makefile",
            "CMakeLists.txt",
            "index.js",
            "app.js",
            "server.js",
            "main.js",
            "main.py",
            "__init__.py",
            "setup.py",
            "go.mod",
            "main.go",
        ];

        let mut key_files = Vec::new();
        for node in nodes {
            if !node.is_dir {
                let file_name = node.path.file_name().and_then(|n| n.to_str()).unwrap_or("");

                for pattern in &important_patterns {
                    if file_name == *pattern {
                        key_files.push(node.path.to_string_lossy().to_string());
                        break;
                    }
                }
            }
        }

        key_files.sort();
        key_files.dedup();
        key_files
    }

    fn find_recent_files(nodes: &[FileNode], hours_ago: u64) -> Vec<String> {
        let cutoff_time = SystemTime::now() - std::time::Duration::from_secs(hours_ago * 3600);

        nodes
            .iter()
            .filter(|node| !node.is_dir && node.modified > cutoff_time)
            .map(|node| node.path.to_string_lossy().to_string())
            .collect()
    }

    fn analyze_file_types(nodes: &[FileNode]) -> std::collections::HashMap<String, usize> {
        let mut types = std::collections::HashMap::new();

        for node in nodes {
            if !node.is_dir {
                let category = format!("{:?}", node.category);
                *types.entry(category).or_insert(0) += 1;
            }
        }

        types
    }

    fn generate_insights(stats: &TreeStats, project_type: &str, nodes: &[FileNode]) -> Vec<String> {
        let mut insights = Vec::new();

        // Size insights
        if stats.total_files > 1000 {
            insights.push("Large codebase with extensive structure".to_string());
        } else if stats.total_files > 100 {
            insights.push("Medium-sized project".to_string());
        } else {
            insights.push("Focused project with concise structure".to_string());
        }

        // Technology insights
        insights.push(format!("{} project", project_type));

        // Feature detection
        let has_tests = nodes.iter().any(|n| {
            let path_str = n.path.to_string_lossy();
            path_str.contains("test") || path_str.contains("spec")
        });
        if has_tests {
            insights.push("Includes test suite".to_string());
        }

        let has_docs = nodes.iter().any(|n| {
            let path_str = n.path.to_string_lossy();
            path_str.contains("README") || path_str.contains("doc")
        });
        if has_docs {
            insights.push("Well-documented project".to_string());
        }

        insights
    }
}

/// Convenience function for one-off project analysis
pub fn analyze_project(project_path: &Path) -> Result<ProjectAnalysis> {
    let analyzer = ProjectAnalyzer::new();
    analyzer.analyze_project(project_path)
}

/// Convenience function for quick project overview
pub fn quick_project_overview(project_path: &Path) -> Result<String> {
    let analyzer = ProjectAnalyzer::new();
    let analysis = analyzer.quick_analysis(project_path)?;

    Ok(format!(
        "{} | {} ({} files, {} dirs)",
        analysis.project_name,
        analysis.project_type,
        analysis.total_files,
        analysis.total_directories
    ))
}
