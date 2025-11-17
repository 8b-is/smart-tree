//! Honeypot and Canary System
//!
//! This module implements canary data injection and leak detection to catch
//! malicious nodes. The system works by:
//!
//! 1. Creating irresistible fake data (credentials, API keys, wallets)
//! 2. Injecting unique canaries into random jobs
//! 3. Monitoring dark web, paste sites, Tor for leaks
//! 4. Tracing leaks back to exact node that processed the job
//! 5. Destroying reputation of compromised nodes

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::{Duration, SystemTime};
use uuid::Uuid;

/// Unique identifier for a canary
pub type CanaryId = String;

/// Type of canary data
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum CanaryType {
    /// Fake Bitcoin wallet address
    BitcoinWallet,

    /// Fake AWS API key
    AwsApiKey,

    /// Fake database credentials
    DatabaseCredentials,

    /// Fake SSH private key
    SshPrivateKey,

    /// Fake credit card number
    CreditCard,

    /// Custom canary data
    Custom(String),
}

/// A canary - fake but tempting data that should never leak
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Canary {
    /// Unique identifier for this canary
    pub id: CanaryId,

    /// Type of canary
    pub canary_type: CanaryType,

    /// The actual fake data
    pub data: String,

    /// Which node processed this canary
    pub node_id: String,

    /// When canary was deployed
    pub deployed_at: SystemTime,

    /// Metadata for tracking
    pub metadata: HashMap<String, String>,
}

impl Canary {
    /// Create a new Bitcoin wallet canary
    pub fn bitcoin_wallet(node_id: String) -> Self {
        let canary_id = Uuid::new_v4().to_string();
        let fake_address = format!("1FOKEN{:0<28}", &canary_id[..10]);

        Self {
            id: canary_id.clone(),
            canary_type: CanaryType::BitcoinWallet,
            data: fake_address,
            node_id,
            deployed_at: SystemTime::now(),
            metadata: HashMap::from([("type".to_string(), "bitcoin".to_string())]),
        }
    }

    /// Create a new AWS API key canary
    pub fn aws_api_key(node_id: String) -> Self {
        let canary_id = Uuid::new_v4().to_string();
        let fake_key = format!("AKIAFOKEN{}", &canary_id.replace("-", ""));

        Self {
            id: canary_id,
            canary_type: CanaryType::AwsApiKey,
            data: fake_key,
            node_id,
            deployed_at: SystemTime::now(),
            metadata: HashMap::from([("provider".to_string(), "aws".to_string())]),
        }
    }

    /// Create a new database credentials canary
    pub fn database_credentials(node_id: String) -> Self {
        let canary_id = Uuid::new_v4().to_string();
        let credentials = serde_json::json!({
            "host": "prod-db-foken.internal",
            "port": 5432,
            "database": "production_foken",
            "username": "foken_admin",
            "password": format!("F0k3n_{}", canary_id),
            "ssl": true,
            "_canary_id": canary_id
        });

        Self {
            id: canary_id,
            canary_type: CanaryType::DatabaseCredentials,
            data: credentials.to_string(),
            node_id,
            deployed_at: SystemTime::now(),
            metadata: HashMap::from([("database".to_string(), "postgres".to_string())]),
        }
    }

    /// Embed this canary into a job payload
    pub fn embed_in_payload(&self, payload: &mut Vec<u8>) {
        // Insert canary at random position in payload
        let position = rand::random::<usize>() % (payload.len() + 1);
        let canary_bytes = format!("\n# CANARY: {}\n", self.data).into_bytes();
        payload.splice(position..position, canary_bytes);
    }
}

/// Honeypot system for detecting data leaks
pub struct HoneypotSystem {
    /// Active canaries being monitored
    active_canaries: HashMap<CanaryId, Canary>,

    /// Monitoring endpoints (paste sites, Tor, etc)
    monitors: Vec<Box<dyn CanaryMonitor>>,

    /// Detected leaks
    leaks: Vec<LeakDetection>,
}

impl HoneypotSystem {
    /// Create new honeypot system
    pub fn new() -> Self {
        Self {
            active_canaries: HashMap::new(),
            monitors: Vec::new(),
            leaks: Vec::new(),
        }
    }

    /// Deploy a honeypot job with canaries
    pub fn deploy_honeypot_job(&mut self, node_id: String) -> HoneypotJob {
        // Create multiple canaries of different types
        let canaries = vec![
            Canary::bitcoin_wallet(node_id.clone()),
            Canary::aws_api_key(node_id.clone()),
            Canary::database_credentials(node_id),
        ];

        // Register canaries
        for canary in &canaries {
            self.active_canaries.insert(canary.id.clone(), canary.clone());
        }

        // Create job with irresistible fake data
        HoneypotJob {
            job_id: Uuid::new_v4().to_string(),
            canaries: canaries.clone(),
            payload: self.create_tempting_payload(&canaries),
            deployed_at: SystemTime::now(),
        }
    }

    /// Create a tempting payload that looks like real sensitive data
    fn create_tempting_payload(&self, canaries: &[Canary]) -> Vec<u8> {
        let mut payload = Vec::new();

        // Add fake but realistic-looking data
        payload.extend_from_slice(b"# Production Environment Configuration\n");
        payload.extend_from_slice(b"# DO NOT COMMIT TO GIT!\n\n");

        for canary in canaries {
            match canary.canary_type {
                CanaryType::BitcoinWallet => {
                    payload.extend_from_slice(
                        format!(
                            "BITCOIN_WALLET={}\n",
                            canary.data
                        )
                        .as_bytes(),
                    );
                }
                CanaryType::AwsApiKey => {
                    payload.extend_from_slice(
                        format!(
                            "AWS_ACCESS_KEY_ID={}\n",
                            canary.data
                        )
                        .as_bytes(),
                    );
                    payload.extend_from_slice(
                        b"AWS_SECRET_ACCESS_KEY=wJalrXUtnFEMI/K7MDENG/bPxRfiCYEXAMPLEKEY\n",
                    );
                }
                CanaryType::DatabaseCredentials => {
                    payload.extend_from_slice(b"DATABASE_URL=");
                    payload.extend_from_slice(canary.data.as_bytes());
                    payload.extend_from_slice(b"\n");
                }
                _ => {}
            }
        }

        payload
    }

    /// Check for data leaks
    pub async fn check_for_leaks(&mut self) -> Vec<LeakDetection> {
        let mut new_leaks = Vec::new();

        // Check each monitor
        for monitor in &self.monitors {
            if let Ok(found) = monitor.scan_for_canaries(&self.active_canaries).await {
                new_leaks.extend(found);
            }
        }

        // Store detected leaks
        self.leaks.extend(new_leaks.clone());

        new_leaks
    }

    /// Get compromised nodes
    pub fn get_compromised_nodes(&self) -> Vec<String> {
        let mut nodes = Vec::new();
        for leak in &self.leaks {
            if !nodes.contains(&leak.node_id) {
                nodes.push(leak.node_id.clone());
            }
        }
        nodes
    }

    /// Add a monitoring endpoint
    pub fn add_monitor(&mut self, monitor: Box<dyn CanaryMonitor>) {
        self.monitors.push(monitor);
    }
}

impl Default for HoneypotSystem {
    fn default() -> Self {
        Self::new()
    }
}

/// A honeypot job containing canaries
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HoneypotJob {
    pub job_id: String,
    pub canaries: Vec<Canary>,
    pub payload: Vec<u8>,
    pub deployed_at: SystemTime,
}

/// Detected data leak
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LeakDetection {
    /// Which canary was leaked
    pub canary_id: CanaryId,

    /// Which node leaked it
    pub node_id: String,

    /// Where the leak was found
    pub found_at: LeakLocation,

    /// When the leak was detected
    pub detected_at: SystemTime,

    /// Proof of leak (URL, screenshot hash, etc)
    pub proof: String,
}

/// Where a leak was detected
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LeakLocation {
    /// Pastebin-like service
    PasteSite { url: String },

    /// Tor hidden service
    TorSite { onion_address: String },

    /// Public GitHub repo
    GitHub { repo: String },

    /// Dark web marketplace
    DarkWebMarket { market: String },

    /// Custom location
    Custom(String),
}

/// Trait for canary monitoring services
pub trait CanaryMonitor: Send + Sync {
    /// Scan for canaries
    fn scan_for_canaries(
        &self,
        canaries: &HashMap<CanaryId, Canary>,
    ) -> impl std::future::Future<Output = Result<Vec<LeakDetection>>> + Send;
}

/// Pastebin monitor
pub struct PastebinMonitor {
    api_key: String,
}

impl PastebinMonitor {
    pub fn new(api_key: String) -> Self {
        Self { api_key }
    }
}

impl CanaryMonitor for PastebinMonitor {
    async fn scan_for_canaries(
        &self,
        canaries: &HashMap<CanaryId, Canary>,
    ) -> Result<Vec<LeakDetection>> {
        // TODO: Actually scan Pastebin API
        // For now, return empty (no leaks detected)
        Ok(Vec::new())
    }
}

/// Tor network monitor
pub struct TorMonitor {
    onion_endpoints: Vec<String>,
}

impl TorMonitor {
    pub fn new(endpoints: Vec<String>) -> Self {
        Self {
            onion_endpoints: endpoints,
        }
    }
}

impl CanaryMonitor for TorMonitor {
    async fn scan_for_canaries(
        &self,
        canaries: &HashMap<CanaryId, Canary>,
    ) -> Result<Vec<LeakDetection>> {
        // TODO: Actually scan Tor network
        // This requires Tor SOCKS proxy setup
        Ok(Vec::new())
    }
}

/// GitHub monitor
pub struct GitHubMonitor {
    api_token: String,
}

impl GitHubMonitor {
    pub fn new(api_token: String) -> Self {
        Self { api_token }
    }
}

impl CanaryMonitor for GitHubMonitor {
    async fn scan_for_canaries(
        &self,
        canaries: &HashMap<CanaryId, Canary>,
    ) -> Result<Vec<LeakDetection>> {
        // TODO: Use GitHub code search API
        Ok(Vec::new())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_bitcoin_canary() {
        let canary = Canary::bitcoin_wallet("node123".to_string());

        assert_eq!(canary.canary_type, CanaryType::BitcoinWallet);
        assert!(canary.data.starts_with("1FOKEN"));
        assert_eq!(canary.node_id, "node123");
    }

    #[test]
    fn test_create_aws_canary() {
        let canary = Canary::aws_api_key("node456".to_string());

        assert_eq!(canary.canary_type, CanaryType::AwsApiKey);
        assert!(canary.data.starts_with("AKIAFOKEN"));
    }

    #[test]
    fn test_honeypot_deployment() {
        let mut system = HoneypotSystem::new();
        let job = system.deploy_honeypot_job("test_node".to_string());

        assert!(!job.canaries.is_empty());
        assert!(!job.payload.is_empty());
        assert_eq!(system.active_canaries.len(), job.canaries.len());
    }

    #[test]
    fn test_compromised_nodes() {
        let mut system = HoneypotSystem::new();

        system.leaks.push(LeakDetection {
            canary_id: "canary1".to_string(),
            node_id: "evil_node".to_string(),
            found_at: LeakLocation::PasteSite {
                url: "https://pastebin.com/test".to_string(),
            },
            detected_at: SystemTime::now(),
            proof: "screenshot_hash".to_string(),
        });

        let compromised = system.get_compromised_nodes();
        assert_eq!(compromised.len(), 1);
        assert_eq!(compromised[0], "evil_node");
    }

    #[test]
    fn test_embed_canary() {
        let canary = Canary::bitcoin_wallet("test".to_string());
        let mut payload = b"original data".to_vec();

        canary.embed_in_payload(&mut payload);

        assert!(payload.len() > 13); // Original + canary
        let payload_str = String::from_utf8_lossy(&payload);
        assert!(payload_str.contains("CANARY"));
    }
}
