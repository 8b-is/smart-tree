//! Foken Network Integration - Secure Distributed Compute
//!
//! This module implements the security infrastructure for the Foken GPU sharing network.
//! It provides multi-layered trust verification, honeypot monitoring, and cryptographic
//! auditing to ensure safe execution of distributed workloads.
//!
//! # Architecture
//!
//! ```text
//! ┌─────────────────────────────────────┐
//! │      Foken Network (Trust Root)     │
//! │  - Compiles unique auditors         │
//! │  - Manages node reputation          │
//! │  - Deploys honeypots                │
//! └──────────────┬──────────────────────┘
//!                │
//!      ┌─────────▼─────────┐
//!      │ Auditor (Per-Job) │
//!      │ - Read-only       │
//!      │ - Time-limited    │
//!      │ - Cryptosigned    │
//!      └─────────┬─────────┘
//!                │ Validates
//!      ┌─────────▼─────────┐
//!      │  Smart Tree Node  │
//!      │ - SHA256 verified │
//!      │ - Open source     │
//!      └─────────┬─────────┘
//!                │ Executes
//!      ┌─────────▼─────────┐
//!      │ Secure Sandbox    │
//!      │ - Encrypted       │
//!      │ - CRC monitored   │
//!      │ - Network locked  │
//!      └───────────────────┘
//! ```
//!
//! # Security Stages
//!
//! ## Stage 1: Public Data Sharing
//! - Low-risk jobs only ("How to SUM in Excel")
//! - No sensitive data exposure
//! - Build initial trust
//!
//! ## Stage 2: Verified Execution
//! - Encrypted container images
//! - CRC verification pre/post execution
//! - Honeypot canaries to detect leaks
//! - Network activity monitoring
//!
//! ## Stage 3: Cryptographic Auditing
//! - Unique auditor per job (can't be pre-analyzed)
//! - Audit-only system rights
//! - Validates Smart Tree binary SHA256
//! - Real-time behavioral monitoring
//! - Signed by Foken network
//!
//! ## Stage 4: Data Obfuscation
//! - Homomorphic encryption (compute on encrypted data)
//! - Differential privacy
//! - Zero-knowledge proofs
//! - Secret sharing across nodes

pub mod auditor;
pub mod daemon;
pub mod honeypot;
pub mod sandbox;
pub mod types;

pub use auditor::{Auditor, AuditorCompiler, AuditorRuntime, ValidationResult};
pub use daemon::{FokenDaemon, ServiceConfig};
pub use honeypot::{Canary, CanaryType, HoneypotSystem, LeakDetection};
pub use sandbox::{SecureExecutionEnvironment, SandboxConfig};
pub use types::*;

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Hardware capabilities profile for Foken GPU sharing
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HardwareProfile {
    /// GPU VRAM in GB (0 if no GPU)
    pub gpu_vram_gb: u32,

    /// GPU compute capability (CUDA cores, etc)
    pub gpu_compute_units: u32,

    /// Total system RAM in GB
    pub ram_gb: u32,

    /// CPU cores available
    pub cpu_cores: u32,

    /// Has dedicated NPU (Neural Processing Unit)
    pub has_npu: bool,

    /// Storage type (SSD, NVMe, HDD)
    pub storage_type: StorageType,

    /// Network bandwidth estimate (Mbps)
    pub network_bandwidth_mbps: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum StorageType {
    NVMe,
    SSD,
    HDD,
    Unknown,
}

impl HardwareProfile {
    /// Detect hardware capabilities of this system
    pub fn detect() -> Result<Self> {
        // TODO: Implement actual hardware detection
        // For now, return sensible defaults
        Ok(Self {
            gpu_vram_gb: 0,
            gpu_compute_units: 0,
            ram_gb: 16,
            cpu_cores: 8,
            has_npu: false,
            storage_type: StorageType::SSD,
            network_bandwidth_mbps: 100,
        })
    }

    /// Check if this hardware is worth sharing on Foken network
    pub fn is_worth_sharing(&self) -> bool {
        // Only ask users with good hardware
        self.gpu_vram_gb >= 8  // 8GB+ GPU
            || self.has_npu     // Or has NPU
            || self.ram_gb >= 32 // Or 32GB+ RAM
    }

    /// Estimate daily Foken earnings at current rates
    pub fn estimate_daily_fokens(&self, idle_hours: f32) -> f32 {
        let base_rate = if self.gpu_vram_gb >= 24 {
            100.0 // High-end GPU (4090, A6000, etc)
        } else if self.gpu_vram_gb >= 12 {
            50.0  // Mid-range GPU (3080, 4070, etc)
        } else if self.gpu_vram_gb >= 8 {
            25.0  // Entry GPU (3060, 4060, etc)
        } else if self.has_npu {
            30.0  // NPU-capable (Apple Silicon, etc)
        } else if self.ram_gb >= 32 {
            10.0  // CPU-only but good RAM
        } else {
            0.0   // Not worth it
        };

        base_rate * (idle_hours / 24.0)
    }
}

/// User's sharing preferences and permissions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SharingPreferences {
    /// Level of sharing (how much to contribute)
    pub level: SharingLevel,

    /// Idle time threshold before sharing (minutes)
    pub idle_threshold_minutes: u32,

    /// Types of workloads allowed
    pub allowed_workloads: Vec<WorkloadType>,

    /// Security monitoring enabled
    pub enable_security_monitoring: bool,

    /// File system access monitoring
    pub monitor_file_access: bool,

    /// Browser extension monitoring
    pub monitor_browser_extensions: bool,

    /// Auto-quarantine suspicious software
    pub auto_quarantine: bool,

    /// Network connection monitoring
    pub monitor_network: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SharingLevel {
    /// Share only when completely idle, conservative limits
    Conservative,

    /// Share when idle with moderate resource usage
    Moderate,

    /// Share aggressively, maximize earnings
    Generous,

    /// Share disabled
    Disabled,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum WorkloadType {
    /// Text-to-speech synthesis
    TTS,

    /// Image diffusion (Stable Diffusion, etc)
    Diffusion,

    /// LLM inference
    LLMInference,

    /// Video encoding
    VideoEncoding,

    /// General compute
    GeneralCompute,
}

impl Default for SharingPreferences {
    fn default() -> Self {
        Self {
            level: SharingLevel::Conservative,
            idle_threshold_minutes: 15,
            allowed_workloads: vec![
                WorkloadType::TTS,
                WorkloadType::Diffusion,
            ],
            enable_security_monitoring: true,
            monitor_file_access: true,
            monitor_browser_extensions: true,
            auto_quarantine: false, // User must opt-in
            monitor_network: true,
        }
    }
}

/// Initialize Foken integration with Smart Tree
pub fn initialize(config: SharingPreferences) -> Result<()> {
    // Detect hardware
    let hardware = HardwareProfile::detect()?;

    // Check if worth sharing
    if !hardware.is_worth_sharing() {
        eprintln!("⚠️  Hardware not recommended for Foken sharing");
        eprintln!("   Recommended: 8GB+ GPU, NPU, or 32GB+ RAM");
        return Ok(());
    }

    // Estimate earnings
    let daily_fokens = hardware.estimate_daily_fokens(8.0); // Assume 8 hours idle
    println!("💰 Estimated earnings: ~{:.0} Fokens/day", daily_fokens);
    println!("   (Based on 8 hours idle time)");
    println!();
    println!("🔗 Learn more: https://st.foken.ai");

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hardware_detection() {
        let hw = HardwareProfile::detect().unwrap();
        assert!(hw.cpu_cores > 0);
        assert!(hw.ram_gb > 0);
    }

    #[test]
    fn test_sharing_worthiness() {
        let high_end = HardwareProfile {
            gpu_vram_gb: 24,
            gpu_compute_units: 16384,
            ram_gb: 64,
            cpu_cores: 16,
            has_npu: false,
            storage_type: StorageType::NVMe,
            network_bandwidth_mbps: 1000,
        };
        assert!(high_end.is_worth_sharing());

        let low_end = HardwareProfile {
            gpu_vram_gb: 0,
            gpu_compute_units: 0,
            ram_gb: 8,
            cpu_cores: 4,
            has_npu: false,
            storage_type: StorageType::HDD,
            network_bandwidth_mbps: 50,
        };
        assert!(!low_end.is_worth_sharing());
    }

    #[test]
    fn test_earnings_estimation() {
        let profile = HardwareProfile {
            gpu_vram_gb: 24,
            gpu_compute_units: 16384,
            ram_gb: 64,
            cpu_cores: 16,
            has_npu: false,
            storage_type: StorageType::NVMe,
            network_bandwidth_mbps: 1000,
        };

        let earnings = profile.estimate_daily_fokens(8.0);
        assert!(earnings > 0.0);
        assert!(earnings >= 30.0); // High-end GPU should earn well
    }
}
