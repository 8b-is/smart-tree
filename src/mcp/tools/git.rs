//! Git-related tools
//!
//! Contains get_git_context and get_git_status handlers.

use super::directory::analyze_directory;
use crate::mcp::helpers::validate_and_convert_path;
use crate::mcp::McpContext;
use anyhow::Result;
use serde_json::{json, Value};
use std::path::Path;
use std::sync::Arc;
use std::time::SystemTime;

/// Get git context for a directory (branch, last commit, etc.)
pub async fn get_git_context(path: &str) -> Result<String> {
    let repo_path = Path::new(path);

    // Try to discover a git repository
    let Ok(repo) = gix::discover(repo_path) else {
        return Ok(String::new()); // Not a git repo, return empty
    };

    let mut git_info = Vec::new();
    git_info.push("GIT CONTEXT:".to_string());

    // Get current branch or HEAD state
    if let Ok(head) = repo.head_ref() {
        match head {
            Some(reference) => {
                let branch_name = reference.name().as_bstr().to_string();
                git_info.push(format!(
                    "Branch: {}",
                    branch_name
                        .strip_prefix("refs/heads/")
                        .unwrap_or(&branch_name)
                ));
            }
            None => {
                if let Ok(head_id) = repo.head_id() {
                    git_info.push(format!("HEAD: {} (detached)", &head_id.to_string()[..8]));
                }
            }
        }
    }

    // Get last commit info
    if let Ok(head_commit) = repo.head_commit() {
        let commit_id = head_commit.id().to_string();
        let message = head_commit
            .message_raw_sloppy()
            .to_string()
            .lines()
            .next()
            .unwrap_or("No commit message")
            .to_string();
        git_info.push(format!("Last commit: {} - {}", &commit_id[..8], message));

        // Get commit time if available (safe duration_since - EPOCH is always in past)
        if let Ok(time) = head_commit.time() {
            let seconds_ago = SystemTime::now()
                .duration_since(SystemTime::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs() as i64
                - time.seconds;

            let time_str = if seconds_ago < 60 {
                format!("{} seconds ago", seconds_ago)
            } else if seconds_ago < 3600 {
                format!("{} minutes ago", seconds_ago / 60)
            } else if seconds_ago < 86400 {
                format!("{} hours ago", seconds_ago / 3600)
            } else {
                format!("{} days ago", seconds_ago / 86400)
            };
            git_info.push(format!("Committed: {}", time_str));
        }
    }

    // Check if working directory is clean or dirty
    git_info.push("Status: Repository detected ✓".to_string());

    if git_info.len() > 1 {
        Ok(git_info.join("\n") + "\n")
    } else {
        Ok(String::new())
    }
}

/// Get git repository structure
pub async fn get_git_status(args: Value, ctx: Arc<McpContext>) -> Result<Value> {
    let path_str = args["path"]
        .as_str()
        .ok_or_else(|| anyhow::anyhow!("Missing path"))?;
    let path = validate_and_convert_path(path_str, &ctx)?;

    // Check if it's a git repository
    let git_dir = path.join(".git");
    if !git_dir.exists() {
        return Ok(json!({
            "content": [{
                "type": "text",
                "text": "Not a git repository"
            }]
        }));
    }

    // Get tree excluding .git directory
    let tree_result = analyze_directory(
        json!({
            "path": path.display().to_string(),
            "mode": "ai",
            "max_depth": 5,
            "show_ignored": true
        }),
        ctx.clone(),
    )
    .await?;

    Ok(json!({
        "content": [{
            "type": "text",
            "text": format!(
                "GIT REPOSITORY STRUCTURE\nPath: {}\n\n{}",
                path.display(),
                tree_result["content"][0]["text"].as_str().unwrap_or("")
            )
        }]
    }))
}
