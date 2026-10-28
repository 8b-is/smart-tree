//! 🔍 Code Review Module - AI-powered code review
//!
//! "Let the machines judge your code!" - The Cheet 😺
//!
//! Supports multiple review modes:
//! - Local: Just show the diff with syntax highlighting
//! - Grok: Use X.AI's Grok for witty, thorough reviews
//! - OpenRouter: Access 100+ models for reviews
//! - Any configured LLM provider

use crate::proxy::{LlmMessage, LlmProxy, LlmRequest, LlmRole};
use anyhow::{Context, Result};
use std::process::Command;

/// Code review provider selection
#[derive(Debug, Clone, Default)]
pub enum ReviewProvider {
    /// Local analysis only (no LLM)
    #[default]
    Local,
    /// Use Grok (X.AI)
    Grok,
    /// Use OpenRouter with optional model
    OpenRouter(Option<String>),
    /// Use any configured provider by name
    Custom(String, Option<String>),
}

/// Code review configuration
#[derive(Debug, Clone)]
pub struct CodeReviewConfig {
    /// Provider to use for review
    pub provider: ReviewProvider,
    /// Review staged changes only
    pub staged: bool,
    /// Review specific files
    pub files: Vec<String>,
    /// Compare against branch
    pub compare_branch: Option<String>,
    /// Include context lines
    pub context_lines: usize,
    /// Focus areas for review
    pub focus: Vec<String>,
}

impl Default for CodeReviewConfig {
    fn default() -> Self {
        Self {
            provider: ReviewProvider::Local,
            staged: false,
            files: Vec::new(),
            compare_branch: None,
            context_lines: 3,
            focus: Vec::new(),
        }
    }
}

/// Code review result
#[derive(Debug)]
pub struct CodeReviewResult {
    pub diff: String,
    pub review: Option<String>,
    pub provider_used: String,
    pub files_reviewed: Vec<String>,
}

/// Run code review with the given configuration
pub async fn run_code_review(config: CodeReviewConfig) -> Result<CodeReviewResult> {
    // Get the diff
    let diff = get_diff(&config)?;

    if diff.trim().is_empty() {
        return Ok(CodeReviewResult {
            diff: String::new(),
            review: Some("No changes to review.".to_string()),
            provider_used: "none".to_string(),
            files_reviewed: Vec::new(),
        });
    }

    // Extract file list from diff
    let files_reviewed = extract_files_from_diff(&diff);

    // Run review based on provider
    let (review, provider_used) = match &config.provider {
        ReviewProvider::Local => {
            // Just format and display the diff
            (None, "local".to_string())
        }
        ReviewProvider::Grok => {
            let review = review_with_llm("grok", "grok-beta", &diff, &config).await?;
            (Some(review), "grok".to_string())
        }
        ReviewProvider::OpenRouter(model) => {
            let model = model.as_deref().unwrap_or("anthropic/claude-3-haiku");
            let review = review_with_llm("openrouter", model, &diff, &config).await?;
            (Some(review), format!("openrouter/{}", model))
        }
        ReviewProvider::Custom(provider, model) => {
            let model = model.as_deref().unwrap_or("default");
            let review = review_with_llm(provider, model, &diff, &config).await?;
            (Some(review), format!("{}/{}", provider, model))
        }
    };

    Ok(CodeReviewResult {
        diff,
        review,
        provider_used,
        files_reviewed,
    })
}

/// Get the diff based on configuration
fn get_diff(config: &CodeReviewConfig) -> Result<String> {
    let mut cmd = Command::new("git");
    cmd.arg("diff");

    // Add context lines
    cmd.arg(format!("-U{}", config.context_lines));

    // Staged only?
    if config.staged {
        cmd.arg("--staged");
    }

    // Compare against branch?
    if let Some(branch) = &config.compare_branch {
        cmd.arg(branch);
    }

    // Specific files?
    if !config.files.is_empty() {
        cmd.arg("--");
        for file in &config.files {
            cmd.arg(file);
        }
    }

    let output = cmd.output().context("Failed to run git diff")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(anyhow::anyhow!("git diff failed: {}", stderr));
    }

    Ok(String::from_utf8_lossy(&output.stdout).to_string())
}

/// Extract file names from diff output
fn extract_files_from_diff(diff: &str) -> Vec<String> {
    diff.lines()
        .filter(|line| line.starts_with("diff --git"))
        .filter_map(|line| {
            // Format: diff --git a/file b/file
            line.split(" b/").nth(1).map(|s| s.to_string())
        })
        .collect()
}

/// Review code using an LLM provider
async fn review_with_llm(
    provider_name: &str,
    model: &str,
    diff: &str,
    config: &CodeReviewConfig,
) -> Result<String> {
    let proxy = LlmProxy::default();

    // Build the system prompt
    let mut system_prompt = String::from(
        "You are an expert code reviewer. Review the following git diff and provide:

1. **Summary**: Brief overview of changes
2. **Positives**: What's good about the changes
3. **Issues**: Potential bugs, security issues, or code smells
4. **Suggestions**: Specific improvements with code examples
5. **Rating**: Overall quality (1-10)

Be concise but thorough. Use markdown formatting.
",
    );

    // Add focus areas if specified
    if !config.focus.is_empty() {
        system_prompt.push_str("\nFocus especially on: ");
        system_prompt.push_str(&config.focus.join(", "));
        system_prompt.push('\n');
    }

    // Build the request
    let request = LlmRequest {
        model: model.to_string(),
        messages: vec![
            LlmMessage {
                role: LlmRole::System,
                content: system_prompt,
            },
            LlmMessage {
                role: LlmRole::User,
                content: format!("Please review this diff:\n\n```diff\n{}\n```", diff),
            },
        ],
        temperature: Some(0.3), // Lower temperature for more consistent reviews
        max_tokens: Some(2000),
        stream: false,
    };

    let response = proxy.complete(provider_name, request).await?;
    Ok(response.content)
}

/// Display code review result in a nice format
pub fn display_review(result: &CodeReviewResult) {
    println!("\n🔍 Code Review Results");
    println!("═══════════════════════════════════════════════════════════════\n");

    // Show files reviewed
    if !result.files_reviewed.is_empty() {
        println!("📁 Files reviewed ({}):", result.files_reviewed.len());
        for file in &result.files_reviewed {
            println!("   • {}", file);
        }
        println!();
    }

    // Show provider used
    println!("🤖 Provider: {}\n", result.provider_used);

    // Show the diff if local mode
    if result.review.is_none() && !result.diff.is_empty() {
        println!("📝 Diff:");
        println!("{}", "-".repeat(60));
        // Colorize the diff output
        for line in result.diff.lines() {
            if line.starts_with('+') && !line.starts_with("+++") {
                println!("\x1b[32m{}\x1b[0m", line); // Green for additions
            } else if line.starts_with('-') && !line.starts_with("---") {
                println!("\x1b[31m{}\x1b[0m", line); // Red for deletions
            } else if line.starts_with("@@") {
                println!("\x1b[36m{}\x1b[0m", line); // Cyan for hunks
            } else if line.starts_with("diff --git") {
                println!("\x1b[1;34m{}\x1b[0m", line); // Bold blue for file headers
            } else {
                println!("{}", line);
            }
        }
        println!();
    }

    // Show the AI review if available
    if let Some(review) = &result.review {
        println!("📋 Review:");
        println!("{}", "-".repeat(60));
        println!("{}", review);
        println!();
    }

    println!("═══════════════════════════════════════════════════════════════");
}

/// Quick helper to run a local review
pub async fn review_local() -> Result<()> {
    let config = CodeReviewConfig::default();
    let result = run_code_review(config).await?;
    display_review(&result);
    Ok(())
}

/// Quick helper to run a Grok review
pub async fn review_with_grok() -> Result<()> {
    let config = CodeReviewConfig {
        provider: ReviewProvider::Grok,
        ..Default::default()
    };
    let result = run_code_review(config).await?;
    display_review(&result);
    Ok(())
}

/// Quick helper to run an OpenRouter review
pub async fn review_with_openrouter(model: Option<String>) -> Result<()> {
    let config = CodeReviewConfig {
        provider: ReviewProvider::OpenRouter(model),
        ..Default::default()
    };
    let result = run_code_review(config).await?;
    display_review(&result);
    Ok(())
}
