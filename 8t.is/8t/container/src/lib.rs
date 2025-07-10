use anyhow::Result;
use chrono::{DateTime, Utc, Timelike};
use eighty_core::{Context, ContextManager};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::time::Instant;
use sysinfo::System;
use tokio::sync::RwLock;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContainerConfig {
    pub max_memory: usize,
    pub offload_threshold: f32,
    pub health_check_interval: u64,
    pub maintenance_window: MaintenanceWindow,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MaintenanceWindow {
    pub start_hour: u8,
    pub duration_hours: u8,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContainerHealth {
    pub status: HealthStatus,
    pub memory_usage: f64,
    pub cpu_usage: f64,
    pub context_count: usize,
    pub uptime: u64,
    pub last_check: DateTime<Utc>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum HealthStatus {
    Healthy,
    Warning,
    Critical,
    Maintenance,
}

pub struct Container {
    id: String,
    config: ContainerConfig,
    context_manager: Arc<ContextManager>,
    health: Arc<RwLock<ContainerHealth>>,
    start_time: Instant,
    peers: dashmap::DashMap<String, PeerContainer>,
}

#[derive(Clone)]
struct PeerContainer {
    endpoint: String,
    health_status: HealthStatus,
}

impl Container {
    pub fn new(config: ContainerConfig) -> Self {
        let context_manager = Arc::new(ContextManager::new(config.max_memory));
        let health = Arc::new(RwLock::new(ContainerHealth {
            status: HealthStatus::Healthy,
            memory_usage: 0.0,
            cpu_usage: 0.0,
            context_count: 0,
            uptime: 0,
            last_check: Utc::now(),
        }));
        
        use uuid::Uuid;
        Self {
            id: blake3::hash(Uuid::new_v4().as_bytes()).to_hex().to_string(),
            config,
            context_manager,
            health,
            start_time: Instant::now(),
            peers: dashmap::DashMap::new(),
        }
    }
    
    pub async fn start(&self) -> Result<()> {
        // Start health monitoring task
        let health = self.health.clone();
        let interval = self.config.health_check_interval;
        let start_time = self.start_time;
        
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(
                std::time::Duration::from_secs(interval)
            );
            
            loop {
                interval.tick().await;
                
                let sys = System::new_all();
                
                // For sysinfo 0.33, we'll use simpler metrics
                let memory_usage = sys.used_memory() as f64 / 1024.0; // MB
                let cpu_usage = sys.global_cpu_usage() as f64;
                
                let mut health_guard = health.write().await;
                health_guard.memory_usage = memory_usage;
                health_guard.cpu_usage = cpu_usage;
                health_guard.uptime = start_time.elapsed().as_secs();
                health_guard.last_check = Utc::now();
                
                // Update status based on thresholds
                health_guard.status = if cpu_usage > 90.0 || memory_usage > 90.0 {
                    HealthStatus::Critical
                } else if cpu_usage > 70.0 || memory_usage > 70.0 {
                    HealthStatus::Warning
                } else {
                    HealthStatus::Healthy
                };
            }
        });
        
        // Start maintenance task
        let config = self.config.clone();
        let context_manager = self.context_manager.clone();
        let health = self.health.clone();
        
        tokio::spawn(async move {
            loop {
                tokio::time::sleep(std::time::Duration::from_secs(3600)).await; // Check hourly
                
                let now = Utc::now();
                let hour = now.hour() as u8;
                
                if hour >= config.maintenance_window.start_hour
                    && hour < config.maintenance_window.start_hour + config.maintenance_window.duration_hours
                {
                    // Enter maintenance mode
                    health.write().await.status = HealthStatus::Maintenance;
                    
                    // Offload low-importance contexts
                    let offloaded = context_manager.offload_contexts(config.offload_threshold);
                    
                    if !offloaded.is_empty() {
                        tracing::info!(
                            "Maintenance: Offloaded {} contexts",
                            offloaded.len()
                        );
                    }
                    
                    // Exit maintenance mode
                    health.write().await.status = HealthStatus::Healthy;
                }
            }
        });
        
        Ok(())
    }
    
    pub async fn add_context(&self, context: Context) -> Result<()> {
        self.context_manager.add_context(context)
            .map_err(|e| anyhow::anyhow!(e))?;
        self.health.write().await.context_count += 1;
        Ok(())
    }
    
    pub async fn get_context(&self, id: &str) -> Option<Arc<Context>> {
        self.context_manager.get_context(id)
    }
    
    pub async fn register_peer(&self, peer_id: String, endpoint: String) {
        self.peers.insert(
            peer_id,
            PeerContainer {
                endpoint,
                health_status: HealthStatus::Healthy,
            },
        );
    }
    
    pub async fn needs_offload(&self) -> bool {
        let health = self.health.read().await;
        matches!(health.status, HealthStatus::Critical | HealthStatus::Maintenance)
    }
    
    pub async fn offload_to_peer(&self, context_id: &str) -> Result<Option<String>> {
        // Find a healthy peer
        let healthy_peer = self.peers
            .iter()
            .find(|p| p.health_status == HealthStatus::Healthy)
            .map(|p| (p.key().clone(), p.endpoint.clone()));
        
        if let Some((peer_id, _endpoint)) = healthy_peer {
            if let Some(_context) = self.context_manager.get_context(context_id) {
                // In a real implementation, this would make an HTTP request
                // to the peer's endpoint to transfer the context
                tracing::info!(
                    "Offloading context {} to peer {}",
                    context_id,
                    peer_id
                );
                
                // Remove from local storage
                // (In real implementation, only after confirming peer received it)
                self.context_manager.offload_contexts(100.0); // Force offload
                
                return Ok(Some(peer_id));
            }
        }
        
        Ok(None)
    }
    
    pub async fn get_health(&self) -> ContainerHealth {
        self.health.read().await.clone()
    }
    
    pub fn get_id(&self) -> &str {
        &self.id
    }
}

impl Default for ContainerConfig {
    fn default() -> Self {
        Self {
            max_memory: 1024 * 1024 * 1024, // 1GB
            offload_threshold: 0.3, // Offload contexts with <30% importance
            health_check_interval: 60, // Check every minute
            maintenance_window: MaintenanceWindow {
                start_hour: 3, // 3 AM
                duration_hours: 2, // 2 hour window
            },
        }
    }
}