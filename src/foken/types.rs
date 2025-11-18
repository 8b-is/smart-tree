//! Common types for Foken integration

use serde::{Deserialize, Serialize};
use std::time::SystemTime;

/// A Foken job to be executed
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FokenJob {
    /// Unique job identifier
    pub job_id: String,

    /// Job payload (encrypted or obfuscated)
    pub payload: Vec<u8>,

    /// Data sensitivity level
    pub sensitivity: DataSensitivity,

    /// Maximum execution time (seconds)
    pub max_execution_time_seconds: u64,

    /// Resource requirements
    pub resources: ResourceRequirements,

    /// Submitted at timestamp
    pub submitted_at: SystemTime,
}

/// Data sensitivity classification
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum DataSensitivity {
    /// Public data, anyone can see
    Public,

    /// Anonymized data, stripped of PII
    Anonymized,

    /// Encrypted data, only sender/receiver can read
    Encrypted,

    /// Sensitive data, never leaves local
    Sensitive,
}

/// Resource requirements for a job
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceRequirements {
    /// GPU VRAM required (GB)
    pub gpu_vram_gb: u32,

    /// RAM required (GB)
    pub ram_gb: u32,

    /// CPU cores required
    pub cpu_cores: u32,

    /// Disk space required (GB)
    pub disk_gb: u32,

    /// Network bandwidth required (Mbps)
    pub network_mbps: u32,
}

impl Default for ResourceRequirements {
    fn default() -> Self {
        Self {
            gpu_vram_gb: 0,
            ram_gb: 4,
            cpu_cores: 2,
            disk_gb: 10,
            network_mbps: 10,
        }
    }
}

/// Result of job execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JobResult {
    /// Job identifier
    pub job_id: String,

    /// Whether execution succeeded
    pub success: bool,

    /// Output data (encrypted if needed)
    pub output: Vec<u8>,

    /// Execution time (milliseconds)
    pub execution_time_ms: u64,

    /// Error message if failed
    pub error_message: Option<String>,
}

/// Node status in the Foken network
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeStatus {
    /// Node identifier
    pub node_id: String,

    /// Current state
    pub state: NodeState,

    /// Hardware capabilities
    pub hardware: super::HardwareProfile,

    /// Jobs completed
    pub jobs_completed: u64,

    /// Jobs failed
    pub jobs_failed: u64,

    /// Total Fokens earned
    pub fokens_earned: f64,

    /// Reputation score (0-1000)
    pub reputation: u32,

    /// Last seen timestamp
    pub last_seen: SystemTime,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum NodeState {
    /// Node is idle and available
    Idle,

    /// Node is executing a job
    Busy,

    /// Node is offline
    Offline,

    /// Node is compromised (caught by honeypot)
    Compromised,

    /// Node is suspended (temporary ban)
    Suspended,
}
