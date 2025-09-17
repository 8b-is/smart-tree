// Q8-Caster Bridge - "Bridging the casting chasm!" 🌉
// Integrates q8-caster functionality into Smart Tree's Rust Shell
// "One shell to cast them all!" - Hue

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::process::Command;

/// Bridge to q8-caster functionality
pub struct Q8CasterBridge {
    q8_caster_path: PathBuf,
    api_port: u16,
}

impl Q8CasterBridge {
    pub fn new() -> Result<Self> {
        // Check if q8-caster is available
        let q8_path = PathBuf::from("/aidata/ayeverse/q8-caster");
        if !q8_path.exists() {
            anyhow::bail!("q8-caster not found at {:?}", q8_path);
        }

        Ok(Self {
            q8_caster_path: q8_path,
            api_port: 8888, // Default q8-caster port
        })
    }

    /// Start q8-caster server if not running
    pub async fn ensure_running(&self) -> Result<()> {
        // Check if already running
        if self.is_running().await? {
            return Ok(());
        }

        // Start q8-caster using its manage.sh script
        let manage_script = self.q8_caster_path.join("scripts/manage.sh");
        if !manage_script.exists() {
            // Try direct binary
            let binary = self.q8_caster_path.join("target/release/q8-caster");
            if binary.exists() {
                Command::new(binary)
                    .arg("--port")
                    .arg(self.api_port.to_string())
                    .spawn()
                    .context("Failed to start q8-caster")?;
            } else {
                anyhow::bail!("q8-caster binary not found. Run 'cargo build --release' in q8-caster directory");
            }
        } else {
            Command::new("bash")
                .arg(manage_script)
                .arg("start")
                .spawn()
                .context("Failed to start q8-caster via manage.sh")?;
        }

        // Wait for it to start
        tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;

        Ok(())
    }

    /// Check if q8-caster is running
    async fn is_running(&self) -> Result<bool> {
        // Try to connect to the API port
        match reqwest::get(format!("http://localhost:{}/health", self.api_port)).await {
            Ok(resp) => Ok(resp.status().is_success()),
            Err(_) => Ok(false),
        }
    }

    /// Discover available cast devices
    pub async fn discover_devices(&self) -> Result<Vec<CastDevice>> {
        self.ensure_running().await?;

        let client = reqwest::Client::new();
        let resp = client
            .get(format!("http://localhost:{}/api/devices", self.api_port))
            .send()
            .await
            .context("Failed to query devices")?;

        if !resp.status().is_success() {
            anyhow::bail!("Failed to get devices: {}", resp.status());
        }

        let devices: Vec<CastDevice> = resp
            .json()
            .await
            .context("Failed to parse devices response")?;

        Ok(devices)
    }

    /// Cast content to a specific device
    pub async fn cast_to_device(&self, device_id: &str, content: &CastContent) -> Result<()> {
        self.ensure_running().await?;

        let client = reqwest::Client::new();
        let resp = client
            .post(format!("http://localhost:{}/api/cast", self.api_port))
            .json(&CastRequest {
                device_id: device_id.to_string(),
                content: content.clone(),
            })
            .send()
            .await
            .context("Failed to cast content")?;

        if !resp.status().is_success() {
            let error_text = resp.text().await.unwrap_or_default();
            anyhow::bail!("Failed to cast: {}", error_text);
        }

        Ok(())
    }

    /// Start web dashboard
    pub async fn start_dashboard(&self, port: u16) -> Result<String> {
        self.ensure_running().await?;

        // The dashboard is served by q8-caster itself
        Ok(format!("http://localhost:{}/dashboard", port))
    }

    /// Cast to ESP32 display
    pub async fn cast_to_esp32(&self, address: &str, content: &str) -> Result<()> {
        // ESP32 devices are handled specially through q8-caster
        let esp_content = CastContent::Text {
            text: content.to_string(),
            format: "plain".to_string(),
        };

        self.cast_to_device(&format!("esp32:{}", address), &esp_content)
            .await
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CastDevice {
    pub id: String,
    pub name: String,
    pub device_type: DeviceType,
    pub address: String,
    pub capabilities: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum DeviceType {
    Chromecast,
    AppleTv,
    Miracast,
    Esp32,
    WebDashboard,
}

impl std::fmt::Display for DeviceType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DeviceType::Chromecast => write!(f, "Chromecast"),
            DeviceType::AppleTv => write!(f, "Apple TV"),
            DeviceType::Miracast => write!(f, "Miracast"),
            DeviceType::Esp32 => write!(f, "ESP32"),
            DeviceType::WebDashboard => write!(f, "Web Dashboard"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum CastContent {
    Text {
        text: String,
        format: String,
    },
    Html {
        html: String,
    },
    Markdown {
        markdown: String,
        theme: Option<String>,
    },
    Image {
        url: String,
    },
    Video {
        url: String,
    },
    Dashboard {
        widgets: Vec<serde_json::Value>,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct CastRequest {
    device_id: String,
    content: CastContent,
}

/// Integration with rust_shell
impl Q8CasterBridge {
    /// Convert rust_shell DisplayTarget to q8-caster device lookup
    pub async fn find_device_for_target(
        &self,
        target: &crate::rust_shell::DisplayTarget,
    ) -> Result<Option<CastDevice>> {
        let devices = self.discover_devices().await?;

        let device = match target {
            crate::rust_shell::DisplayTarget::AppleTV { name, .. } => devices
                .into_iter()
                .find(|d| d.device_type == DeviceType::AppleTv && d.name == *name),
            crate::rust_shell::DisplayTarget::Chromecast { name, .. } => devices
                .into_iter()
                .find(|d| d.device_type == DeviceType::Chromecast && d.name == *name),
            crate::rust_shell::DisplayTarget::ESP32Display { address, .. } => devices
                .into_iter()
                .find(|d| d.device_type == DeviceType::Esp32 && d.address == *address),
            _ => None,
        };

        Ok(device)
    }

    /// Adapt rust_shell content for q8-caster
    pub fn adapt_content(
        &self,
        content: &str,
        format: &crate::rust_shell::OutputFormat,
    ) -> CastContent {
        match format {
            crate::rust_shell::OutputFormat::HTML => CastContent::Html {
                html: content.to_string(),
            },
            crate::rust_shell::OutputFormat::Markdown => CastContent::Markdown {
                markdown: content.to_string(),
                theme: Some("dark".to_string()),
            },
            _ => CastContent::Text {
                text: content.to_string(),
                format: "plain".to_string(),
            },
        }
    }
}

/// Q8-Caster enhanced functionality for rust_shell
pub async fn enhance_rust_shell_with_q8(_shell: &mut crate::rust_shell::RustShell) -> Result<()> {
    println!("🚀 Enhancing Rust Shell with Q8-Caster capabilities...");

    let bridge = Q8CasterBridge::new()?;

    // Ensure q8-caster is running
    bridge.ensure_running().await?;

    // Discover and add devices
    let devices = bridge.discover_devices().await?;
    println!("  Found {} Q8-Caster devices", devices.len());

    for device in devices {
        println!(
            "  • {} ({}): {}",
            device.name, device.device_type, device.address
        );
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_q8_bridge_creation() {
        // This test will only pass if q8-caster is available
        if PathBuf::from("/aidata/ayeverse/q8-caster").exists() {
            let bridge = Q8CasterBridge::new();
            assert!(bridge.is_ok());
        }
    }
}
