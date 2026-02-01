//! User Spaces - Containerized environments for collaborators
//!
//! Each collaborator gets their own space with:
//! - Isolated or shared filesystem view
//! - Their preferred tools (template)
//! - Ability to share terminals, memories

use super::{Identity, Template};
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Container isolation level
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub enum IsolationLevel {
    /// Full container (podman) - own filesystem, network
    Podman,
    /// Linux namespace - lighter, shared kernel
    Namespace,
    /// No isolation - direct access (trust mode)
    #[default]
    None,
}

/// Configuration for a user space
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpaceConfig {
    /// Isolation level
    pub isolation: IsolationLevel,

    /// Base template to use
    pub template: Option<String>,

    /// Working directory within the space
    pub workdir: PathBuf,

    /// Environment variables to set
    pub env: Vec<(String, String)>,

    /// Paths to mount into the space (host:container)
    pub mounts: Vec<(PathBuf, PathBuf)>,

    /// Memory limit (bytes, 0 = unlimited)
    pub memory_limit: u64,

    /// CPU limit (cores, 0 = unlimited)
    pub cpu_limit: f32,

    /// Network access
    pub network: NetworkConfig,
}

impl Default for SpaceConfig {
    fn default() -> Self {
        SpaceConfig {
            isolation: IsolationLevel::None,
            template: None,
            workdir: PathBuf::from("."),
            env: Vec::new(),
            mounts: Vec::new(),
            memory_limit: 0,
            cpu_limit: 0.0,
            network: NetworkConfig::default(),
        }
    }
}

/// Network configuration for a space
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct NetworkConfig {
    /// Allow outbound internet access
    pub internet: bool,
    /// Allow connections to host services
    pub host_access: bool,
    /// Ports to expose
    pub exposed_ports: Vec<u16>,
}

/// A user's active space in a project
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserSpace {
    /// The user's identity
    pub identity: Identity,

    /// Space configuration
    pub config: SpaceConfig,

    /// Container/namespace ID (if isolated)
    pub container_id: Option<String>,

    /// Unix socket path for this space
    pub socket_path: Option<PathBuf>,

    /// PID of the space's shell process
    pub shell_pid: Option<u32>,

    /// When the space was created
    pub created_at: u64,

    /// Last activity timestamp
    pub last_active: u64,
}

impl UserSpace {
    /// Create a new user space
    pub fn new(identity: Identity, config: SpaceConfig) -> Self {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        UserSpace {
            identity,
            config,
            container_id: None,
            socket_path: None,
            shell_pid: None,
            created_at: now,
            last_active: now,
        }
    }

    /// Create a space with a template
    pub fn with_template(identity: Identity, template: &Template) -> Self {
        let config = SpaceConfig {
            template: Some(template.name.clone()),
            isolation: template.default_isolation.clone(),
            env: template.env.clone(),
            ..Default::default()
        };
        Self::new(identity, config)
    }

    /// Start the user space (create container if needed)
    pub async fn start(&mut self) -> Result<()> {
        match self.config.isolation {
            IsolationLevel::Podman => self.start_podman().await,
            IsolationLevel::Namespace => self.start_namespace().await,
            IsolationLevel::None => self.start_direct().await,
        }
    }

    /// Start with podman container
    async fn start_podman(&mut self) -> Result<()> {
        // TODO: Implement podman container creation
        // podman run -d --name {identity} -v {project}:/workspace {template_image}
        tracing::info!("Starting podman container for {}", self.identity);
        Ok(())
    }

    /// Start with Linux namespace
    async fn start_namespace(&mut self) -> Result<()> {
        // TODO: Implement namespace isolation
        // unshare --user --mount --pid --fork
        tracing::info!("Starting namespace for {}", self.identity);
        Ok(())
    }

    /// Start without isolation (trust mode)
    async fn start_direct(&mut self) -> Result<()> {
        tracing::info!("Starting direct space for {}", self.identity);
        // Just set up the working directory and environment
        Ok(())
    }

    /// Stop the user space
    pub async fn stop(&mut self) -> Result<()> {
        match self.config.isolation {
            IsolationLevel::Podman => {
                if let Some(ref id) = self.container_id {
                    // podman stop {id}
                    tracing::info!("Stopping podman container {}", id);
                }
            }
            IsolationLevel::Namespace => {
                if let Some(pid) = self.shell_pid {
                    // kill namespace process
                    tracing::info!("Stopping namespace pid {}", pid);
                }
            }
            IsolationLevel::None => {
                // Nothing to stop
            }
        }
        self.container_id = None;
        self.shell_pid = None;
        Ok(())
    }

    /// Execute a command in the space
    pub async fn exec(&self, command: &[&str]) -> Result<String> {
        match self.config.isolation {
            IsolationLevel::Podman => {
                if let Some(ref id) = self.container_id {
                    // podman exec {id} {command}
                    tracing::debug!("Exec in podman {}: {:?}", id, command);
                }
            }
            IsolationLevel::Namespace => {
                if let Some(pid) = self.shell_pid {
                    // nsenter -t {pid} -a {command}
                    tracing::debug!("Exec in namespace {}: {:?}", pid, command);
                }
            }
            IsolationLevel::None => {
                // Direct execution
                tracing::debug!("Direct exec: {:?}", command);
            }
        }
        // TODO: Actually execute command
        Ok(String::new())
    }

    /// Share terminal with another user
    pub async fn share_terminal(&self, with: &Identity) -> Result<String> {
        // Returns a session ID that the other user can join
        let session_id = format!(
            "term-{}-{}",
            self.identity.username,
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_millis()
        );
        tracing::info!(
            "Sharing terminal {} with {}",
            session_id,
            with.canonical()
        );
        Ok(session_id)
    }

    /// Update last active timestamp
    pub fn touch(&mut self) {
        self.last_active = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
    }

    /// Check if space has been idle too long
    pub fn is_idle(&self, timeout_secs: u64) -> bool {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        now - self.last_active > timeout_secs
    }
}

/// Manager for all active user spaces
#[derive(Debug, Default)]
pub struct SpaceManager {
    /// Active spaces by identity canonical name
    spaces: std::collections::HashMap<String, UserSpace>,
}

impl SpaceManager {
    pub fn new() -> Self {
        SpaceManager {
            spaces: std::collections::HashMap::new(),
        }
    }

    /// Get or create a space for a user
    pub async fn get_or_create(
        &mut self,
        identity: Identity,
        config: SpaceConfig,
    ) -> Result<&mut UserSpace> {
        let key = identity.canonical();
        if !self.spaces.contains_key(&key) {
            let mut space = UserSpace::new(identity, config);
            space.start().await?;
            self.spaces.insert(key.clone(), space);
        }
        Ok(self.spaces.get_mut(&key).unwrap())
    }

    /// Get an existing space
    pub fn get(&self, identity: &Identity) -> Option<&UserSpace> {
        self.spaces.get(&identity.canonical())
    }

    /// Remove a user's space
    pub async fn remove(&mut self, identity: &Identity) -> Result<()> {
        let key = identity.canonical();
        if let Some(mut space) = self.spaces.remove(&key) {
            space.stop().await?;
        }
        Ok(())
    }

    /// List all active spaces
    pub fn list(&self) -> Vec<&UserSpace> {
        self.spaces.values().collect()
    }

    /// Clean up idle spaces
    pub async fn cleanup_idle(&mut self, timeout_secs: u64) -> Result<usize> {
        let idle: Vec<String> = self
            .spaces
            .iter()
            .filter(|(_, s)| s.is_idle(timeout_secs))
            .map(|(k, _)| k.clone())
            .collect();

        let count = idle.len();
        for key in idle {
            if let Some(mut space) = self.spaces.remove(&key) {
                space.stop().await?;
            }
        }
        Ok(count)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_space_config_default() {
        let config = SpaceConfig::default();
        assert_eq!(config.isolation, IsolationLevel::None);
        assert!(config.template.is_none());
    }

    #[test]
    fn test_user_space_creation() {
        let identity = Identity::local("test");
        let space = UserSpace::new(identity.clone(), SpaceConfig::default());
        assert_eq!(space.identity, identity);
        assert!(space.container_id.is_none());
    }
}
