//! 🔄 GiT Relay - Smart Git CLI Integration with Compression
//! 
//! This module provides a compressed, intelligent interface to Git CLI
//! operations without requiring API keys or vendor lock-in. It leverages
//! our quantum compression and context awareness for maximum efficiency.

use super::{SmartResponse, TaskContext, TokenSavings};
use super::context::ContextAnalyzer;
use anyhow::{Result, anyhow};
use serde::{Deserialize, Serialize};
use std::path::Path;
use std::process::{Command, Output};

/// 🔄 Git relay with smart compression and context awareness
pub struct GitRelay {
    context_analyzer: ContextAnalyzer,
}

/// 📊 Git operation result with compression
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitResult {
    /// Operation type
    pub operation: GitOperation,
    /// Compressed output
    pub output: String,
    /// Exit code
    pub exit_code: i32,
    /// Context-aware summary
    pub summary: String,
    /// Suggested next actions
    pub suggestions: Vec<String>,
}

/// 🏷️ Git operation types
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum GitOperation {
    Status,
    Log,
    Diff,
    Branch,
    Remote,
    Commit,
    Push,
    Pull,
    Clone,
    Add,
    Reset,
    Stash,
    Tag,
    Merge,
    Rebase,
    Custom(String),
}

/// 📈 Git relay response with smart compression
pub type GitRelayResponse = SmartResponse<GitResult>;

impl GitRelay {
    /// Create new Git relay
    pub fn new() -> Self {
        Self {
            context_analyzer: ContextAnalyzer::new(),
        }
    }
    
    /// 🔄 Execute git command with smart compression
    pub fn execute(
        &self,
        repo_path: &Path,
        operation: GitOperation,
        args: &[String],
        context: Option<&TaskContext>,
    ) -> Result<GitRelayResponse> {
        // Build git command
        let mut cmd = Command::new("git");
        cmd.current_dir(repo_path);
        
        // Add operation-specific arguments
        match &operation {
            GitOperation::Status => {
                cmd.args(&["status", "--porcelain", "--branch"]);
            }
            GitOperation::Log => {
                cmd.args(&["log", "--oneline", "--graph", "--decorate", "-10"]);
            }
            GitOperation::Diff => {
                cmd.args(&["diff", "--stat", "--color=never"]);
            }
            GitOperation::Branch => {
                cmd.args(&["branch", "-v", "-a"]);
            }
            GitOperation::Remote => {
                cmd.args(&["remote", "-v"]);
            }
            GitOperation::Custom(op) => {
                cmd.arg(op);
            }
            _ => {
                return Err(anyhow!("Operation {:?} not yet implemented", operation));
            }
        }
        
        // Add user-provided arguments
        cmd.args(args);
        
        // Execute command
        let output = cmd.output()?;
        
        // Process and compress output
        let git_result = self.process_output(operation, output, context)?;
        
        // Calculate token savings
        let original_tokens = git_result.output.len() / 4; // Rough estimation
        let compressed_tokens = git_result.summary.len() / 4;
        let token_savings = TokenSavings::new(original_tokens, compressed_tokens, "git-relay");
        
        // Create response
        let response = GitRelayResponse {
            primary: vec![git_result.clone()],
            secondary: vec![],
            context_summary: format!("Git {} operation completed", self.operation_name(&git_result.operation)),
            token_savings,
            suggestions: git_result.suggestions.clone(),
        };
        
        Ok(response)
    }
    
    /// 📊 Smart git status with context awareness
    pub fn smart_status(
        &self,
        repo_path: &Path,
        context: Option<&TaskContext>,
    ) -> Result<GitRelayResponse> {
        self.execute(repo_path, GitOperation::Status, &[], context)
    }
    
    /// 📜 Smart git log with relevance filtering
    pub fn smart_log(
        &self,
        repo_path: &Path,
        limit: Option<usize>,
        context: Option<&TaskContext>,
    ) -> Result<GitRelayResponse> {
        let limit_str = limit.unwrap_or(10).to_string();
        let args = vec![format!("-{}", limit_str)];
        self.execute(repo_path, GitOperation::Log, &args, context)
    }
    
    /// 🔍 Smart git diff with context filtering
    pub fn smart_diff(
        &self,
        repo_path: &Path,
        target: Option<&str>,
        context: Option<&TaskContext>,
    ) -> Result<GitRelayResponse> {
        let args = if let Some(t) = target {
            vec![t.to_string()]
        } else {
            vec![]
        };
        self.execute(repo_path, GitOperation::Diff, &args, context)
    }
    
    /// 🌿 Smart branch information
    pub fn smart_branches(
        &self,
        repo_path: &Path,
        context: Option<&TaskContext>,
    ) -> Result<GitRelayResponse> {
        self.execute(repo_path, GitOperation::Branch, &[], context)
    }
    
    /// 🔗 Smart remote information
    pub fn smart_remotes(
        &self,
        repo_path: &Path,
        context: Option<&TaskContext>,
    ) -> Result<GitRelayResponse> {
        self.execute(repo_path, GitOperation::Remote, &[], context)
    }
    
    /// 🎯 Execute custom git command with compression
    pub fn custom_command(
        &self,
        repo_path: &Path,
        command: &str,
        args: &[String],
        context: Option<&TaskContext>,
    ) -> Result<GitRelayResponse> {
        self.execute(repo_path, GitOperation::Custom(command.to_string()), args, context)
    }
    
    /// Process git command output with smart compression
    fn process_output(
        &self,
        operation: GitOperation,
        output: Output,
        context: Option<&TaskContext>,
    ) -> Result<GitResult> {
        let stdout = String::from_utf8_lossy(&output.stdout);
        let stderr = String::from_utf8_lossy(&output.stderr);
        
        // Combine stdout and stderr
        let full_output = if stderr.is_empty() {
            stdout.to_string()
        } else {
            format!("{}\nERROR: {}", stdout, stderr)
        };
        
        // Generate context-aware summary
        let summary = self.generate_summary(&operation, &full_output, context);
        
        // Generate suggestions
        let suggestions = self.generate_suggestions(&operation, &full_output, output.status.code().unwrap_or(-1));
        
        Ok(GitResult {
            operation,
            output: full_output,
            exit_code: output.status.code().unwrap_or(-1),
            summary,
            suggestions,
        })
    }
    
    /// Generate context-aware summary
    fn generate_summary(
        &self,
        operation: &GitOperation,
        output: &str,
        _context: Option<&TaskContext>,
    ) -> String {
        match operation {
            GitOperation::Status => {
                self.summarize_status(output)
            }
            GitOperation::Log => {
                self.summarize_log(output)
            }
            GitOperation::Diff => {
                self.summarize_diff(output)
            }
            GitOperation::Branch => {
                self.summarize_branches(output)
            }
            GitOperation::Remote => {
                self.summarize_remotes(output)
            }
            _ => {
                format!("Git {} completed with {} characters of output", 
                       self.operation_name(operation), output.len())
            }
        }
    }
    
    /// Summarize git status output
    fn summarize_status(&self, output: &str) -> String {
        let lines: Vec<&str> = output.lines().collect();
        if lines.is_empty() {
            return "Repository is clean - no changes detected".to_string();
        }
        
        let mut modified = 0;
        let mut added = 0;
        let mut deleted = 0;
        let mut untracked = 0;
        let mut branch_info = String::new();
        
        for line in lines {
            if line.starts_with("##") {
                branch_info = line.trim_start_matches("## ").to_string();
            } else if line.starts_with(" M") || line.starts_with("M ") {
                modified += 1;
            } else if line.starts_with("A ") || line.starts_with(" A") {
                added += 1;
            } else if line.starts_with(" D") || line.starts_with("D ") {
                deleted += 1;
            } else if line.starts_with("??") {
                untracked += 1;
            }
        }
        
        let mut summary = format!("Branch: {}", branch_info);
        if modified > 0 { summary.push_str(&format!(", {} modified", modified)); }
        if added > 0 { summary.push_str(&format!(", {} added", added)); }
        if deleted > 0 { summary.push_str(&format!(", {} deleted", deleted)); }
        if untracked > 0 { summary.push_str(&format!(", {} untracked", untracked)); }
        
        summary
    }
    
    /// Summarize git log output
    fn summarize_log(&self, output: &str) -> String {
        let lines: Vec<&str> = output.lines().collect();
        let commit_count = lines.iter().filter(|line| line.contains("*")).count();
        
        if commit_count == 0 {
            "No commits found".to_string()
        } else {
            format!("Last {} commits shown", commit_count)
        }
    }
    
    /// Summarize git diff output
    fn summarize_diff(&self, output: &str) -> String {
        if output.trim().is_empty() {
            "No differences found".to_string()
        } else {
            let lines: Vec<&str> = output.lines().collect();
            let file_count = lines.iter().filter(|line| line.contains("|")).count();
            format!("Changes in {} files", file_count)
        }
    }
    
    /// Summarize git branch output
    fn summarize_branches(&self, output: &str) -> String {
        let lines: Vec<&str> = output.lines().collect();
        let local_branches = lines.iter().filter(|line| !line.contains("remotes/")).count();
        let remote_branches = lines.iter().filter(|line| line.contains("remotes/")).count();
        
        format!("{} local branches, {} remote branches", local_branches, remote_branches)
    }
    
    /// Summarize git remote output
    fn summarize_remotes(&self, output: &str) -> String {
        let lines: Vec<&str> = output.lines().collect();
        let remote_count = lines.len() / 2; // Each remote has fetch and push URLs
        
        if remote_count == 0 {
            "No remotes configured".to_string()
        } else {
            format!("{} remote(s) configured", remote_count)
        }
    }
    
    /// Generate operation-specific suggestions
    fn generate_suggestions(&self, operation: &GitOperation, output: &str, exit_code: i32) -> Vec<String> {
        let mut suggestions = Vec::new();
        
        if exit_code != 0 {
            suggestions.push("Command failed - check git status and repository state".to_string());
            return suggestions;
        }
        
        match operation {
            GitOperation::Status => {
                if output.contains("??") {
                    suggestions.push("Use 'git add .' to stage untracked files".to_string());
                }
                if output.contains(" M") || output.contains("M ") {
                    suggestions.push("Use 'git add -u' to stage modified files".to_string());
                }
                if output.contains("ahead") {
                    suggestions.push("Use 'git push' to push local commits".to_string());
                }
                if output.contains("behind") {
                    suggestions.push("Use 'git pull' to fetch remote changes".to_string());
                }
            }
            GitOperation::Log => {
                suggestions.push("Use smart_diff to see changes in recent commits".to_string());
                suggestions.push("Use smart_branches to see branch information".to_string());
            }
            GitOperation::Diff => {
                if !output.trim().is_empty() {
                    suggestions.push("Review changes before committing".to_string());
                    suggestions.push("Use 'git add' to stage specific changes".to_string());
                }
            }
            GitOperation::Branch => {
                suggestions.push("Use 'git checkout <branch>' to switch branches".to_string());
                suggestions.push("Use 'git branch -d <branch>' to delete merged branches".to_string());
            }
            _ => {}
        }
        
        suggestions
    }
    
    /// Get human-readable operation name
    fn operation_name(&self, operation: &GitOperation) -> &str {
        match operation {
            GitOperation::Status => "status",
            GitOperation::Log => "log",
            GitOperation::Diff => "diff",
            GitOperation::Branch => "branch",
            GitOperation::Remote => "remote",
            GitOperation::Commit => "commit",
            GitOperation::Push => "push",
            GitOperation::Pull => "pull",
            GitOperation::Clone => "clone",
            GitOperation::Add => "add",
            GitOperation::Reset => "reset",
            GitOperation::Stash => "stash",
            GitOperation::Tag => "tag",
            GitOperation::Merge => "merge",
            GitOperation::Rebase => "rebase",
            GitOperation::Custom(op) => op,
        }
    }
}

impl Default for GitRelay {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_git_relay_creation() {
        let relay = GitRelay::new();
        assert_eq!(relay.operation_name(&GitOperation::Status), "status");
    }

    #[test]
    fn test_status_summary() {
        let relay = GitRelay::new();
        let output = "## main...origin/main\n M file1.rs\n?? file2.rs\n";
        let summary = relay.summarize_status(output);
        assert!(summary.contains("main"));
        assert!(summary.contains("modified"));
        assert!(summary.contains("untracked"));
    }
}
