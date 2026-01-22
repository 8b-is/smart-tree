//! Daemon Client - CLI interface to the Smart Tree daemon
//!
//! This module provides a client for communicating with the Smart Tree daemon.
//! It handles:
//! - Health checks to see if daemon is running
//! - Auto-starting the daemon if not running
//! - Sending commands to the daemon via HTTP
//! - Managing daemon lifecycle (start/stop/status)
//!
//! "The messenger between CLI and brain!" - Cheet

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::process::Command;
#[cfg(unix)]
use std::process::Stdio;
use std::time::Duration;

#[cfg(windows)]
use std::os::windows::process::CommandExt;

/// Simple percent-encoding for URL query parameters
fn percent_encode(s: &str) -> String {
    s.chars()
        .map(|c| match c {
            'A'..='Z' | 'a'..='z' | '0'..='9' | '-' | '_' | '.' | '~' => c.to_string(),
            ' ' => "+".to_string(),
            _ => format!("%{:02X}", c as u8),
        })
        .collect()
}

/// Default daemon port (Foken's magic number!)
pub const DEFAULT_DAEMON_PORT: u16 = 8420;

/// Daemon client configuration
#[derive(Debug, Clone)]
pub struct DaemonClient {
    /// The port the daemon is running on
    port: u16,
    /// Base URL for daemon API
    base_url: String,
    /// HTTP client with timeout
    client: reqwest::Client,
}

/// Response from daemon info endpoint
#[derive(Debug, Deserialize, Serialize)]
pub struct DaemonInfo {
    pub name: String,
    pub version: String,
    pub description: String,
}

/// Response from daemon context endpoint
#[derive(Debug, Deserialize, Serialize)]
pub struct ContextResponse {
    pub projects_count: usize,
    pub directories_count: usize,
    pub last_scan: Option<String>,
    pub credits_balance: f64,
}

/// Response from daemon credits endpoint
#[derive(Debug, Deserialize, Serialize)]
pub struct CreditsResponse {
    pub balance: f64,
    pub total_earned: f64,
    pub total_spent: f64,
    pub recent_transactions: Vec<Transaction>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Transaction {
    pub timestamp: String,
    pub amount: f64,
    pub description: String,
}

/// Project info from daemon
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct ProjectInfo {
    pub path: String,
    pub name: String,
    pub project_type: String,
    pub key_files: Vec<String>,
    pub essence: String,
}

/// Tool call request
#[derive(Debug, Serialize)]
pub struct ToolCallRequest {
    pub name: String,
    pub arguments: serde_json::Value,
}

/// Status of the daemon
#[derive(Debug)]
pub enum DaemonStatus {
    /// Daemon is running and healthy
    Running(DaemonInfo),
    /// Daemon is not running
    NotRunning,
    /// Daemon is starting up
    Starting,
    /// Error checking daemon status
    Error(String),
}

impl DaemonClient {
    /// Create a new daemon client
    pub fn new(port: u16) -> Self {
        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(5))
            .build()
            .unwrap_or_default();

        Self {
            port,
            base_url: format!("http://127.0.0.1:{}", port),
            client,
        }
    }

    /// Create with default port (8420)
    pub fn default_port() -> Self {
        Self::new(DEFAULT_DAEMON_PORT)
    }

    /// Check if the daemon is running
    pub async fn check_status(&self) -> DaemonStatus {
        match self.health_check().await {
            Ok(true) => {
                // Daemon is healthy, get info
                match self.get_info().await {
                    Ok(info) => DaemonStatus::Running(info),
                    Err(_) => DaemonStatus::Running(DaemonInfo {
                        name: "smart-tree-daemon".to_string(),
                        version: "unknown".to_string(),
                        description: "Running".to_string(),
                    }),
                }
            }
            Ok(false) => DaemonStatus::NotRunning,
            Err(e) => {
                // Check if it's a connection error (daemon not running)
                let err_str = e.to_string().to_lowercase();
                if err_str.contains("connection refused")
                    || err_str.contains("tcp connect error")
                    || err_str.contains("connect error")
                    || err_str.contains("error sending request")
                {
                    DaemonStatus::NotRunning
                } else {
                    DaemonStatus::Error(e.to_string())
                }
            }
        }
    }

    /// Health check - returns true if daemon is responsive
    pub async fn health_check(&self) -> Result<bool> {
        let url = format!("{}/health", self.base_url);
        match self.client.get(&url).send().await {
            Ok(resp) => Ok(resp.status().is_success()),
            Err(e) => Err(anyhow::anyhow!("Health check failed: {}", e)),
        }
    }

    /// Get daemon info
    pub async fn get_info(&self) -> Result<DaemonInfo> {
        let url = format!("{}/info", self.base_url);
        let resp = self
            .client
            .get(&url)
            .send()
            .await
            .context("Failed to connect to daemon")?;

        resp.json::<DaemonInfo>()
            .await
            .context("Failed to parse daemon info")
    }

    /// Get system context summary
    pub async fn get_context(&self) -> Result<ContextResponse> {
        let url = format!("{}/context", self.base_url);
        let resp = self
            .client
            .get(&url)
            .send()
            .await
            .context("Failed to connect to daemon")?;

        resp.json::<ContextResponse>()
            .await
            .context("Failed to parse context response")
    }

    /// Get list of detected projects
    pub async fn get_projects(&self) -> Result<Vec<ProjectInfo>> {
        let url = format!("{}/context/projects", self.base_url);
        let resp = self
            .client
            .get(&url)
            .send()
            .await
            .context("Failed to connect to daemon")?;

        resp.json::<Vec<ProjectInfo>>()
            .await
            .context("Failed to parse projects response")
    }

    /// Query context by keyword
    pub async fn query_context(&self, query: &str) -> Result<serde_json::Value> {
        let url = format!("{}/context/query", self.base_url);
        let resp = self
            .client
            .post(&url)
            .json(&serde_json::json!({ "query": query }))
            .send()
            .await
            .context("Failed to connect to daemon")?;

        resp.json::<serde_json::Value>()
            .await
            .context("Failed to parse query response")
    }

    /// List files in a directory via daemon
    pub async fn list_files(
        &self,
        path: Option<&str>,
        pattern: Option<&str>,
        depth: Option<usize>,
    ) -> Result<Vec<String>> {
        let mut url = format!("{}/context/files?", self.base_url);

        if let Some(p) = path {
            url.push_str(&format!("path={}&", percent_encode(p)));
        }
        if let Some(pat) = pattern {
            url.push_str(&format!("pattern={}&", percent_encode(pat)));
        }
        if let Some(d) = depth {
            url.push_str(&format!("depth={}", d));
        }

        let resp = self
            .client
            .get(&url)
            .send()
            .await
            .context("Failed to connect to daemon")?;

        resp.json::<Vec<String>>()
            .await
            .context("Failed to parse files response")
    }

    /// Get Foken credits
    pub async fn get_credits(&self) -> Result<CreditsResponse> {
        let url = format!("{}/credits", self.base_url);
        let resp = self
            .client
            .get(&url)
            .send()
            .await
            .context("Failed to connect to daemon")?;

        resp.json::<CreditsResponse>()
            .await
            .context("Failed to parse credits response")
    }

    /// Record token savings for credits
    pub async fn record_savings(
        &self,
        tokens_saved: u64,
        description: &str,
    ) -> Result<CreditsResponse> {
        let url = format!("{}/credits/record", self.base_url);
        let resp = self
            .client
            .post(&url)
            .json(&serde_json::json!({
                "tokens_saved": tokens_saved,
                "description": description
            }))
            .send()
            .await
            .context("Failed to connect to daemon")?;

        resp.json::<CreditsResponse>()
            .await
            .context("Failed to parse credits response")
    }

    /// Call a daemon tool
    pub async fn call_tool(
        &self,
        name: &str,
        arguments: serde_json::Value,
    ) -> Result<serde_json::Value> {
        let url = format!("{}/tools/call", self.base_url);
        let req = ToolCallRequest {
            name: name.to_string(),
            arguments,
        };

        let resp = self
            .client
            .post(&url)
            .json(&req)
            .send()
            .await
            .context("Failed to connect to daemon")?;

        resp.json::<serde_json::Value>()
            .await
            .context("Failed to parse tool response")
    }

    /// List available daemon tools
    pub async fn list_tools(&self) -> Result<Vec<serde_json::Value>> {
        let url = format!("{}/tools", self.base_url);
        let resp = self
            .client
            .get(&url)
            .send()
            .await
            .context("Failed to connect to daemon")?;

        resp.json::<Vec<serde_json::Value>>()
            .await
            .context("Failed to parse tools list")
    }

    /// Start the daemon in the background
    ///
    /// Returns Ok(true) if daemon was started, Ok(false) if already running
    pub async fn start_daemon(&self) -> Result<bool> {
        // First check if already running
        if matches!(self.check_status().await, DaemonStatus::Running(_)) {
            return Ok(false);
        }

        // Get the path to our own executable
        let exe_path = std::env::current_exe().context("Failed to get current executable path")?;

        // Start daemon as a background process
        // We use setsid on Unix to detach from the terminal
        #[cfg(unix)]
        {
            Command::new(&exe_path)
                .args(["--daemon", "--daemon-port", &self.port.to_string()])
                .stdin(Stdio::null())
                .stdout(Stdio::null())
                .stderr(Stdio::null())
                .spawn()
                .context("Failed to start daemon process")?;
        }

        #[cfg(windows)]
        {
            Command::new(&exe_path)
                .args(["--daemon", "--daemon-port", &self.port.to_string()])
                .creation_flags(0x00000008) // DETACHED_PROCESS
                .spawn()
                .context("Failed to start daemon process")?;
        }

        // Wait a moment for daemon to start
        tokio::time::sleep(Duration::from_millis(500)).await;

        // Wait up to 5 seconds for daemon to become healthy
        for _ in 0..10 {
            if self.health_check().await.unwrap_or(false) {
                return Ok(true);
            }
            tokio::time::sleep(Duration::from_millis(500)).await;
        }

        Err(anyhow::anyhow!(
            "Daemon started but failed to become healthy within 5 seconds"
        ))
    }

    /// Stop the daemon
    ///
    /// Note: This requires the daemon to have a shutdown endpoint or we use a signal
    pub async fn stop_daemon(&self) -> Result<bool> {
        // Check if running first
        if !matches!(self.check_status().await, DaemonStatus::Running(_)) {
            return Ok(false);
        }

        // Try to send a shutdown request (we'll add this endpoint to daemon)
        let url = format!("{}/shutdown", self.base_url);
        match self.client.post(&url).send().await {
            Ok(_) => {
                // Wait for daemon to stop
                tokio::time::sleep(Duration::from_millis(500)).await;
                Ok(true)
            }
            Err(_) => {
                // If endpoint doesn't exist, try finding and killing the process
                #[cfg(unix)]
                {
                    // Find process listening on our port and kill it
                    let output = Command::new("lsof")
                        .args(["-ti", &format!(":{}", self.port)])
                        .output();

                    if let Ok(output) = output {
                        if let Ok(pid_str) = String::from_utf8(output.stdout) {
                            for pid in pid_str.lines() {
                                if let Ok(pid) = pid.trim().parse::<i32>() {
                                    let _ = Command::new("kill").arg(pid.to_string()).output();
                                }
                            }
                            return Ok(true);
                        }
                    }
                }

                Err(anyhow::anyhow!("Failed to stop daemon"))
            }
        }
    }

    /// Ensure daemon is running, starting it if necessary
    ///
    /// This is the main entry point for daemon-first architecture.
    /// Returns the daemon info if running/started successfully.
    pub async fn ensure_running(&self) -> Result<DaemonInfo> {
        match self.check_status().await {
            DaemonStatus::Running(info) => Ok(info),
            DaemonStatus::NotRunning => {
                eprintln!("🌳 Starting Smart Tree daemon on port {}...", self.port);
                self.start_daemon().await?;
                self.get_info().await
            }
            DaemonStatus::Starting => {
                // Wait for it to finish starting
                tokio::time::sleep(Duration::from_secs(2)).await;
                self.get_info().await
            }
            DaemonStatus::Error(e) => Err(anyhow::anyhow!("Daemon error: {}", e)),
        }
    }
}

/// Print daemon status in a nice format
pub fn print_daemon_status(status: &DaemonStatus) {
    match status {
        DaemonStatus::Running(info) => {
            println!("╔═══════════════════════════════════════════════════════════╗");
            println!("║        🌳 SMART TREE DAEMON STATUS: RUNNING 🌳           ║");
            println!("╠═══════════════════════════════════════════════════════════╣");
            println!("║  Name:        {:<45} ║", info.name);
            println!("║  Version:     {:<45} ║", info.version);
            println!(
                "║  Description: {:<45} ║",
                truncate_str(&info.description, 45)
            );
            println!("╚═══════════════════════════════════════════════════════════╝");
        }
        DaemonStatus::NotRunning => {
            println!("╔═══════════════════════════════════════════════════════════╗");
            println!("║        🌳 SMART TREE DAEMON STATUS: STOPPED 🛑            ║");
            println!("╠═══════════════════════════════════════════════════════════╣");
            println!("║  The daemon is not running.                               ║");
            println!("║  Start with: st --daemon-start                            ║");
            println!("╚═══════════════════════════════════════════════════════════╝");
        }
        DaemonStatus::Starting => {
            println!("╔═══════════════════════════════════════════════════════════╗");
            println!("║        🌳 SMART TREE DAEMON STATUS: STARTING ⏳           ║");
            println!("╠═══════════════════════════════════════════════════════════╣");
            println!("║  The daemon is starting up...                             ║");
            println!("╚═══════════════════════════════════════════════════════════╝");
        }
        DaemonStatus::Error(e) => {
            println!("╔═══════════════════════════════════════════════════════════╗");
            println!("║        🌳 SMART TREE DAEMON STATUS: ERROR ❌              ║");
            println!("╠═══════════════════════════════════════════════════════════╣");
            println!("║  Error: {:<50} ║", truncate_str(e, 50));
            println!("╚═══════════════════════════════════════════════════════════╝");
        }
    }
}

/// Print context summary from daemon
pub fn print_context_summary(ctx: &ContextResponse) {
    println!("╔═══════════════════════════════════════════════════════════╗");
    println!("║           📊 SYSTEM CONTEXT SUMMARY 📊                    ║");
    println!("╠═══════════════════════════════════════════════════════════╣");
    println!("║  Projects detected:    {:<35} ║", ctx.projects_count);
    println!("║  Directories tracked:  {:<35} ║", ctx.directories_count);
    println!(
        "║  Last scan:            {:<35} ║",
        ctx.last_scan.as_deref().unwrap_or("Never")
    );
    println!("║  Foken balance:        {:<35.2} ║", ctx.credits_balance);
    println!("╚═══════════════════════════════════════════════════════════╝");
}

/// Print credits summary
pub fn print_credits(credits: &CreditsResponse) {
    println!("╔═══════════════════════════════════════════════════════════╗");
    println!("║           💰 FOKEN CREDITS SUMMARY 💰                     ║");
    println!("╠═══════════════════════════════════════════════════════════╣");
    println!("║  Current Balance:  {:<38.2} ║", credits.balance);
    println!("║  Total Earned:     {:<38.2} ║", credits.total_earned);
    println!("║  Total Spent:      {:<38.2} ║", credits.total_spent);
    if !credits.recent_transactions.is_empty() {
        println!("╠═══════════════════════════════════════════════════════════╣");
        println!("║  Recent Transactions:                                     ║");
        for tx in credits.recent_transactions.iter().take(5) {
            println!(
                "║    +{:>8.0} - {:<43} ║",
                tx.amount,
                truncate_str(&tx.description, 43)
            );
        }
    }
    println!("╚═══════════════════════════════════════════════════════════╝");
}

/// Print projects list
pub fn print_projects(projects: &[ProjectInfo]) {
    println!("╔═══════════════════════════════════════════════════════════╗");
    println!("║           📁 DETECTED PROJECTS 📁                         ║");
    println!("╠═══════════════════════════════════════════════════════════╣");
    if projects.is_empty() {
        println!("║  No projects detected yet.                                ║");
        println!("║  Add directories to watch with: st --daemon-watch <path>  ║");
    } else {
        for p in projects.iter().take(10) {
            println!("║  📦 {:<53} ║", truncate_str(&p.name, 53));
            println!("║     Type: {:<47} ║", p.project_type);
            println!("║     Path: {:<47} ║", truncate_str(&p.path, 47));
            if !p.key_files.is_empty() {
                println!(
                    "║     Files: {:<46} ║",
                    truncate_str(&p.key_files.join(", "), 46)
                );
            }
        }
        if projects.len() > 10 {
            println!(
                "║  ... and {} more projects                                ║",
                projects.len() - 10
            );
        }
    }
    println!("╚═══════════════════════════════════════════════════════════╝");
}

/// Helper to truncate strings for display
fn truncate_str(s: &str, max_len: usize) -> String {
    if s.len() <= max_len {
        s.to_string()
    } else {
        format!("{}...", &s[..max_len - 3])
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_client_creation() {
        let client = DaemonClient::new(8420);
        assert_eq!(client.port, 8420);
        assert_eq!(client.base_url, "http://127.0.0.1:8420");
    }

    #[test]
    fn test_default_port() {
        let client = DaemonClient::default_port();
        assert_eq!(client.port, DEFAULT_DAEMON_PORT);
    }

    #[tokio::test]
    async fn test_status_when_not_running() {
        // Use a random high port unlikely to have anything
        let client = DaemonClient::new(59999);
        let status = client.check_status().await;
        assert!(matches!(
            status,
            DaemonStatus::NotRunning | DaemonStatus::Error(_)
        ));
    }
}
