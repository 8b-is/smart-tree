//! STD Client - Talk to the Smart Tree Daemon via Unix socket
//!
//! This module provides a client for the ST binary protocol daemon.
//! Falls back gracefully to local operation if daemon isn't running.
//!
//! "The thin client to the fat brain!" - Cheet

use anyhow::{Context, Result};
use std::path::PathBuf;
use std::process::{Command, Stdio};
use std::time::Duration;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::UnixStream;

use st_protocol::{Frame, Verb};

/// Get the default socket path
pub fn socket_path() -> PathBuf {
    std::env::var("XDG_RUNTIME_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from("/tmp"))
        .join("st.sock")
}

/// Check if the STD daemon is running
pub async fn is_daemon_running() -> bool {
    let path = socket_path();
    if !path.exists() {
        return false;
    }

    // Try to connect and ping
    match UnixStream::connect(&path).await {
        Ok(mut stream) => {
            let ping = Frame::ping();
            if stream.write_all(&ping.encode()).await.is_err() {
                return false;
            }

            let mut buf = [0u8; 256];
            match tokio::time::timeout(Duration::from_millis(500), stream.read(&mut buf)).await {
                Ok(Ok(n)) if n > 0 => {
                    // Got a response - daemon is alive
                    true
                }
                _ => false,
            }
        }
        Err(_) => false,
    }
}

/// Start the STD daemon in the background
pub async fn start_daemon() -> Result<bool> {
    if is_daemon_running().await {
        return Ok(false); // Already running
    }

    // Find the std binary - try same directory as current exe first
    let exe_path = std::env::current_exe().ok();
    let exe_dir = exe_path.as_ref().and_then(|p| p.parent());

    let std_path = if let Some(dir) = exe_dir {
        let candidate = dir.join("std");
        if candidate.exists() {
            candidate
        } else {
            // Fall back to PATH
            PathBuf::from("std")
        }
    } else {
        PathBuf::from("std")
    };

    // Start daemon as background process using setsid to fully detach
    #[cfg(unix)]
    {
        use std::os::unix::process::CommandExt;

        // Use setsid to create a new session, fully detaching the daemon
        let mut cmd = Command::new(&std_path);
        cmd.arg("start")
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null());

        // Create new process group
        unsafe {
            cmd.pre_exec(|| {
                libc::setsid();
                Ok(())
            });
        }

        cmd.spawn().context("Failed to start std daemon")?;
    }

    #[cfg(windows)]
    {
        Command::new(&std_path)
            .arg("start")
            .creation_flags(0x00000008) // DETACHED_PROCESS
            .spawn()
            .context("Failed to start std daemon")?;
    }

    // Wait for daemon to become ready (up to 5 seconds)
    // Daemon may take time to load memories
    for _ in 0..50 {
        tokio::time::sleep(Duration::from_millis(100)).await;
        if is_daemon_running().await {
            return Ok(true);
        }
    }

    Err(anyhow::anyhow!("Daemon started but not responding after 5 seconds"))
}

/// Client for communicating with the STD daemon
pub struct StdClient {
    stream: Option<UnixStream>,
}

impl StdClient {
    /// Connect to the daemon (returns None if not running)
    pub async fn connect() -> Option<Self> {
        let path = socket_path();
        match UnixStream::connect(&path).await {
            Ok(stream) => Some(Self {
                stream: Some(stream),
            }),
            Err(_) => None,
        }
    }

    /// Connect or start daemon if not running
    pub async fn connect_or_start() -> Result<Self> {
        if let Some(client) = Self::connect().await {
            return Ok(client);
        }

        // Not running - start it
        start_daemon().await?;

        // Try again
        Self::connect()
            .await
            .ok_or_else(|| anyhow::anyhow!("Failed to connect after starting daemon"))
    }

    /// Send a frame and get response
    pub async fn send(&mut self, frame: &Frame) -> Result<Vec<u8>> {
        let stream = self
            .stream
            .as_mut()
            .ok_or_else(|| anyhow::anyhow!("Not connected"))?;

        stream
            .write_all(&frame.encode())
            .await
            .context("Failed to send frame")?;

        let mut buf = vec![0u8; 65536];
        let n = stream.read(&mut buf).await.context("Failed to read response")?;
        buf.truncate(n);
        Ok(buf)
    }

    /// Ping the daemon
    pub async fn ping(&mut self) -> Result<bool> {
        let resp = self.send(&Frame::ping()).await?;
        Ok(!resp.is_empty() && resp[0] == Verb::Ping as u8)
    }

    /// Scan a directory via daemon
    pub async fn scan(&mut self, path: &str, depth: u8) -> Result<String> {
        let frame = Frame::scan(path, depth);
        let resp = self.send(&frame).await?;

        // Response format: [SCAN verb][payload...][END]
        if resp.is_empty() {
            return Ok(String::new());
        }

        // Skip verb byte and END byte, decode payload
        if resp.len() > 2 {
            let payload = &resp[1..resp.len() - 1];
            String::from_utf8(payload.to_vec()).context("Invalid UTF-8 in scan response")
        } else {
            Ok(String::new())
        }
    }

    /// Format directory via daemon (7 modes: classic, ai, json, hex, quantum, stats, digest)
    pub async fn format(&mut self, path: &str, depth: u8, mode: &str) -> Result<String> {
        let frame = Frame::format_path(mode, path, depth);
        let resp = self.send(&frame).await?;

        if resp.len() > 2 {
            let payload = &resp[1..resp.len() - 1];
            String::from_utf8(payload.to_vec()).context("Invalid UTF-8 in format response")
        } else {
            Ok(String::new())
        }
    }

    /// Search content via daemon
    pub async fn search(&mut self, path: &str, pattern: &str, max_results: u8) -> Result<String> {
        let frame = Frame::search_path(path, pattern, max_results);
        let resp = self.send(&frame).await?;

        if resp.len() > 2 {
            let payload = &resp[1..resp.len() - 1];
            String::from_utf8(payload.to_vec()).context("Invalid UTF-8 in search response")
        } else {
            Ok(String::new())
        }
    }

    /// Store a memory
    pub async fn remember(
        &mut self,
        content: &str,
        keywords: &str,
        memory_type: &str,
    ) -> Result<String> {
        let frame = Frame::remember(content, keywords, memory_type);
        let resp = self.send(&frame).await?;

        if resp.len() > 2 {
            let payload = &resp[1..resp.len() - 1];
            String::from_utf8(payload.to_vec()).context("Invalid UTF-8 in remember response")
        } else {
            Ok(String::new())
        }
    }

    /// Recall memories
    pub async fn recall(&mut self, keywords: &str, max_results: u8) -> Result<String> {
        let frame = Frame::recall(keywords, max_results);
        let resp = self.send(&frame).await?;

        if resp.len() > 2 {
            let payload = &resp[1..resp.len() - 1];
            String::from_utf8(payload.to_vec()).context("Invalid UTF-8 in recall response")
        } else {
            Ok(String::new())
        }
    }

    /// Get daemon stats (version, memories, grid info)
    pub async fn stats(&mut self) -> Result<serde_json::Value> {
        let frame = Frame::stats();
        let resp = self.send(&frame).await?;

        if resp.len() > 2 {
            let payload = &resp[1..resp.len() - 1];
            let json_str = String::from_utf8(payload.to_vec())
                .context("Invalid UTF-8 in stats response")?;
            serde_json::from_str(&json_str).context("Invalid JSON in stats response")
        } else {
            Ok(serde_json::json!({}))
        }
    }

    /// Get wave grid state
    pub async fn m8_wave(&mut self) -> Result<String> {
        let frame = Frame::m8_wave();
        let resp = self.send(&frame).await?;

        if resp.len() > 2 {
            let payload = &resp[1..resp.len() - 1];
            String::from_utf8(payload.to_vec()).context("Invalid UTF-8 in wave response")
        } else {
            Ok(String::new())
        }
    }
}

/// Ensure daemon is running, with user feedback
pub async fn ensure_daemon(quiet: bool) -> Result<()> {
    if is_daemon_running().await {
        return Ok(());
    }

    if !quiet {
        eprintln!("🌳 Starting Smart Tree daemon...");
    }

    start_daemon().await?;

    if !quiet {
        eprintln!("✓ Daemon ready");
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_socket_path() {
        let path = socket_path();
        assert!(path.to_string_lossy().contains("st.sock"));
    }

    #[tokio::test]
    async fn test_daemon_check() {
        // Just verify it doesn't panic
        let _ = is_daemon_running().await;
    }
}
