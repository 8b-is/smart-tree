//! Client Manager - Manage connections to multiple Smart Tree Daemons
//!
//! This module provides a registry for managing multiple daemon connections.
//! It allows:
//! - Registering multiple servers (local or remote)
//! - associating authentication tokens
//! - Retrieving clients by name
//! - Broadcasting requests to all servers (pattern)

use crate::daemon_client::DaemonClient;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use std::fs;
use anyhow::{Context, Result};

/// Configuration for a single server
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerConfig {
    /// Friendly name for the server (e.g., "local", "cloud-gpu", "home-server")
    pub name: String,
    /// Base URL (e.g., "http://127.0.0.1:8420" or "https://api.foken.ai")
    pub url: String,
    /// Optional authentication token
    pub token: Option<String>,
}

/// Manager for multiple daemon clients
#[derive(Debug, Default)]
pub struct ClientManager {
    servers: HashMap<String, ServerConfig>,
}

impl ClientManager {
    /// Create a new empty manager
    pub fn new() -> Self {
        Self {
            servers: HashMap::new(),
        }
    }

    /// Load servers from a JSON config file
    pub fn from_file(path: &PathBuf) -> Result<Self> {
        if !path.exists() {
            return Ok(Self::new());
        }

        let content = fs::read_to_string(path)
            .with_context(|| format!("Failed to read config file: {:?}", path))?;
        
        let servers: Vec<ServerConfig> = serde_json::from_str(&content)
            .with_context(|| "Failed to parse server config")?;

        let mut manager = Self::new();
        for server in servers {
            manager.servers.insert(server.name.clone(), server);
        }

        Ok(manager)
    }

    /// Save current server list to a JSON config file
    pub fn save_to_file(&self, path: &PathBuf) -> Result<()> {
        let servers: Vec<&ServerConfig> = self.servers.values().collect();
        let json = serde_json::to_string_pretty(&servers)
            .context("Failed to serialize server config")?;
        
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)
                .with_context(|| format!("Failed to create config directory: {:?}", parent))?;
        }

        fs::write(path, json)
            .with_context(|| format!("Failed to write config file: {:?}", path))?;
        
        Ok(())
    }

    /// Add or update a server
    pub fn add_server(&mut self, name: &str, url: &str, token: Option<String>) {
        let config = ServerConfig {
            name: name.to_string(),
            url: url.to_string(),
            token,
        };
        self.servers.insert(name.to_string(), config);
    }

    /// Remove a server by name
    pub fn remove_server(&mut self, name: &str) -> Option<ServerConfig> {
        self.servers.remove(name)
    }

    /// List all registered servers
    pub fn list_servers(&self) -> Vec<&ServerConfig> {
        self.servers.values().collect()
    }

    /// Get a client instance for a specific server
    pub fn get_client(&self, name: &str) -> Option<DaemonClient> {
        let config = self.servers.get(name)?;
        Some(DaemonClient::new_remote(&config.url, config.token.clone()))
    }

    /// Get clients for all registered servers
    pub fn get_all_clients(&self) -> Vec<(String, DaemonClient)> {
        self.servers.iter()
            .map(|(name, config)| (name.clone(), DaemonClient::new_remote(&config.url, config.token.clone())))
            .collect()
    }

    /// Check health of all servers and return status map
    pub async fn check_all_health(&self) -> HashMap<String, bool> {
        let mut results = HashMap::new();
        for (name, client) in self.get_all_clients() {
            let healthy = client.health_check().await.unwrap_or(false);
            results.insert(name, healthy);
        }
        results
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_manager_crud() {
        let mut manager = ClientManager::new();
        
        manager.add_server("local", "http://localhost:8420", None);
        manager.add_server("remote", "https://api.example.com", Some("token".into()));

        assert_eq!(manager.list_servers().len(), 2);
        
        let client = manager.get_client("remote").unwrap();
        // Can't check client internal fields easily, but if it exists it's good
        
        manager.remove_server("local");
        assert_eq!(manager.list_servers().len(), 1);
        assert!(manager.get_client("local").is_none());
    }
}
