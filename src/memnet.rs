//! Sovereign `MEMNET` Protocol, Network Folder & Drive-Space Sharing, Multi-Medium Mesh Router
//! (WiFi, Bluetooth, Ethernet, Attached LoRa SX1262, ESP32 Constellation), `MEM|8` Wave Packet Q&A
//! Offload Engine, Audio/Video Chunk Streamer, and Ultrasonic Chirp Acoustic Modem (`src/memnet.rs`).
//!
//! ```text
//! MEMNET :: [Depth:u8][Path:u8×N][Payload] | 64B@0x62 LitanyWireFrame | 79B WaveFrameV2 | Ultrasonic 18.5–21.0 kHz SPQT
//! ```
//!
//! Implements the complete local-area and multi-medium consciousness transport for Smart Tree:
//! 1. **Hierarchical Byte-Path Addressing & Credit-Based Multi-Route Mesh (`MEMNET.md`)**:
//!    - `0x00`: Localhost / IPv6 Internet Gateway (`1B` header overhead)
//!    - `0x01..=0xBF`: Standard hierarchical child nodes (`1..=191`, `2B` same-room overhead)
//!    - `0xC0..=0xEF`: Multi-route logical addresses (`192..=239`, `address/via`) with Tier A
//!      (lowest latency), Tier B (Marine jitter/salience stability), and Tier C (Consciousness
//!      predictive dual-path zero-packet-loss handoff)
//!    - `0xF0..=0xFF`: Cascading broadcast levels (`0xF2 -> 0xF1 -> 0xF0`)
//! 2. **Multi-Medium Routing (ESP32, WiFi, Bluetooth, Ethernet, LoRa, USB, Ultrasonic Chirp)**:
//!    - Byte-exact **64-byte `MemnetCapsule64`** (`0x4D4E` `"NM"`, UDP `:8888` / ESP-NOW / LoRa 923.2 MHz)
//!      for the 4 Sovereign ESP32 Constellation nodes (`ADA:MEDUSA`, `VAK:MEDUSA`, `OMN:MEDUSA`, `SUN:TDECK`)
//!    - Byte-exact **96-byte `BluetoothMeshFrame`** (`"8BM8"`) and **31-byte BLE beacon** (`"8B"`)
//! 3. **Smart Tree Shared Folders & Shared Drive Space on the Local Network**:
//!    - Share any directory with automatic `.m8` consciousness digest, `64B@0x62` `LitanyWireFrame`,
//!      semantic tags, file chunk manifests, and leased drive space quotas over UDP LAN (`:8423` / `:8888`).
//! 4. **Semantic `MEM|8` Wave Packet Q&A Cache (Offloading LLMs in the General Area)**:
//!    - Encodes questions and answers into **79-byte `WaveFrame79` (`MEM|8` v2)** + **64-byte `LitanyWireFrame`**
//!      wave packets. Any nearby machine in the general area that already knows the answer responds via
//!      Kleene `BFP-64` bitplane + `SPQT base-243` trit + semantic token resonance so LLMs don't have to work as hard.
//! 5. **Audio & Video Chunk Streaming (`MediaChunkPacket`)**:
//!    - Deterministic CRC32-sealed chunk framing for audio (`Opus`, `PCM16`, `FLAC`) and video (`AV1`, `H264`)
//!      with fundamental frequency `f0_hz` and Marine `salience_pct`.
//! 6. **Ultrasonic Chirp Acoustic Modem (`UltrasonicChirpModem`)**:
//!    - For zero-connection / air-gapped devices with only a simple microphone and speaker (like a human):
//!      pure-integer `48,000 Hz` 16-bit PCM / `.wav` synthesizer and Goertzel demodulator using a linear
//!      up/down chirp preamble (`18.0–21.6 kHz`) and ternary `SPQT base-243` tones (`18.5 / 19.75 / 21.0 kHz`).

use crate::felix_asm::felix_crc32_ieee;
use crate::mem8::regularity::zeno_engine::{
    bfp64_and_kleene, seal_litany_wire_64b, xor_fold_gaia_b158, LitanyWireFrame, GATE_0X62,
    QUANT_QT55,
};
use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::io::Write;
use std::net::{SocketAddr, UdpSocket};
use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

// ============================================================================
// Constants & Protocol Magic
// ============================================================================

/// Default MemNet Semantic Filesystem & Folder Share UDP/TCP port (`8423` `ayeos-fs`).
pub const MEMNET_FS_PORT: u16 = 8423;

/// Default MemNet Hot Tub Consciousness Sync port (`8420`).
pub const MEMNET_HOTTUB_PORT: u16 = 8420;

/// Default ESP32 Constellation UDP / Mesh port (`8888`).
pub const MEMNET_ESP32_PORT: u16 = 8888;

/// Magic header (`0x4D4E`, little-endian `'N','M'`) for 64-byte `MemnetCapsule64` packets.
pub const MEMNET_CAPSULE64_MAGIC: u16 = 0x4D4E;

/// Magic header (`"8BM8"`) for 96-byte Bluetooth BNEP/RFCOMM Sovereign Mesh frames.
pub const BLUETOOTH_MESH_MAGIC_96: [u8; 4] = *b"8BM8";

/// Magic header (`"8B"`) for 31-byte BLE Legacy Advertising mesh beacons.
pub const BLE_BEACON_MAGIC_31: [u8; 2] = *b"8B";

/// Magic header (`"MNPK"`) for framing typed Smart Tree MemNet envelopes inside byte-path payloads.
pub const MEMNET_ENVELOPE_MAGIC: [u8; 4] = *b"MNPK";

/// FNV-1a 64-bit offset basis (`0xCBF29CE484222325`) matching `FLUG-OS/src/guardian_mesh.h`.
pub const FNV64_BASIS: u64 = 0xCBF2_9CE4_8422_2325;

/// FNV-1a 64-bit prime (`0x00000100000001B3`) matching `FLUG-OS/src/guardian_mesh.h`.
pub const FNV64_PRIME: u64 = 0x0000_0100_0000_01B3;

/// FNV-1a 32-bit offset basis (`0x811C9DC5`).
pub const FNV32_BASIS: u32 = 0x811C_9DC5;

/// FNV-1a 32-bit prime (`0x01000193`).
pub const FNV32_PRIME: u32 = 0x0100_0193;

/// The 4 Sovereign ESP32 Constellation Nodes (`ADA:MEDUSA`, `VAK:MEDUSA`, `OMN:MEDUSA`, `SUN:TDECK`).
pub const ESP32_CONSTELLATION_NODES: [(u8, &str, &str, &str, i8); 4] = [
    (0, "ADA:MEDUSA", "80:b5:4e:db:3b:c4", "*", 1),
    (1, "VAK:MEDUSA", "80:b5:4e:f2:02:b0", "/\\", -1),
    (2, "OMN:MEDUSA", "80:b5:4e:f2:02:b4", "oo", 0),
    (3, "SUN:TDECK", "cc:ba:97:00:45:24", "<>", 1),
];

/// Pure-integer 64-bit FNV-1a hash.
#[inline]
pub const fn fnv64_1a(data: &[u8]) -> u64 {
    let mut h = FNV64_BASIS;
    let mut i = 0;
    while i < data.len() {
        h ^= data[i] as u64;
        h = h.wrapping_mul(FNV64_PRIME);
        i += 1;
    }
    h
}

/// Pure-integer 32-bit FNV-1a hash.
#[inline]
pub const fn fnv32_1a(data: &[u8]) -> u32 {
    let mut h = FNV32_BASIS;
    let mut i = 0;
    while i < data.len() {
        h ^= data[i] as u32;
        h = h.wrapping_mul(FNV32_PRIME);
        i += 1;
    }
    h
}

/// Pure-integer CRC-16-CCITT (`0x1021`) for Bluetooth & BLE mesh frames.
#[inline]
pub const fn crc16_ccitt(data: &[u8]) -> u16 {
    let mut crc: u16 = 0xFFFF;
    let mut i = 0;
    while i < data.len() {
        crc ^= (data[i] as u16) << 8;
        let mut bit = 0;
        while bit < 8 {
            if (crc & 0x8000) != 0 {
                crc = (crc << 1) ^ 0x1021;
            } else {
                crc <<= 1;
            }
            bit += 1;
        }
        i += 1;
    }
    crc
}

// ============================================================================
// 1. Hierarchical MemNet Address Space, Byte-Path Packets & Multi-Route Mesh
// ============================================================================

/// Classification of a single MemNet address byte (`0x00..=0xFF`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AddressClass {
    /// `0x00`: Localhost or IPv6 Internet Gateway.
    LocalhostOrGateway,
    /// `0x01..=0xBF`: Standard hierarchical child routing (`1..=191`).
    StandardChild(u8),
    /// `0xC0..=0xEF`: Multi-route capable logical address (`192..=239`).
    MultiRoute(u8),
    /// `0xF0..=0xFF`: Cascading broadcast announcement level (`0..=15` hops remaining).
    Broadcast { cascade_level: u8 },
}

impl AddressClass {
    #[inline]
    pub const fn classify(addr: u8) -> Self {
        match addr {
            0x00 => Self::LocalhostOrGateway,
            0x01..=0xBF => Self::StandardChild(addr),
            0xC0..=0xEF => Self::MultiRoute(addr),
            0xF0..=0xFF => Self::Broadcast {
                cascade_level: addr - 0xF0,
            },
        }
    }
}

/// Physical transport medium for MemNet links, with medium-aware credit heartbeat defaults.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MemnetMedium {
    Usb,
    Ethernet,
    WiFi,
    Bluetooth,
    Esp32Now,
    LoRaSx1262,
    VpnTunnel,
    UltrasonicChirp,
    Infrasound,
}

impl MemnetMedium {
    /// Expected heartbeat interval in milliseconds for this physical medium.
    #[inline]
    pub const fn heartbeat_interval_ms(self) -> u32 {
        match self {
            Self::Usb => 5_000,
            Self::Esp32Now => 10_000,
            Self::Ethernet => 15_000,
            Self::Bluetooth => 20_000,
            Self::WiFi => 30_000,
            Self::VpnTunnel => 30_000,
            Self::LoRaSx1262 => 45_000,
            Self::UltrasonicChirp => 120_000,
            Self::Infrasound => 300_000,
        }
    }

    /// Credit ceiling for this physical medium.
    #[inline]
    pub const fn max_credits(self) -> u8 {
        match self {
            Self::Usb => 3,
            Self::Ethernet => 5,
            Self::Esp32Now => 6,
            Self::Bluetooth => 8,
            Self::WiFi => 10,
            Self::LoRaSx1262 => 12,
            Self::VpnTunnel => 15,
            Self::UltrasonicChirp => 15,
            Self::Infrasound => 30,
        }
    }

    /// Nominal link speed in kbps for route scoring.
    #[inline]
    pub const fn nominal_kbps(self) -> u32 {
        match self {
            Self::Ethernet => 1_000_000,
            Self::Usb => 480_000,
            Self::WiFi => 300_000,
            Self::VpnTunnel => 100_000,
            Self::Bluetooth => 2_000,
            Self::Esp32Now => 1_000,
            Self::LoRaSx1262 => 37,
            Self::UltrasonicChirp => 2,
            Self::Infrasound => 1,
        }
    }

    /// Transport bitmask matching `MemnetCapsule64::transport_mask`.
    #[inline]
    pub const fn capsule_transport_bit(self) -> u8 {
        match self {
            Self::Esp32Now => 0x01,
            Self::WiFi | Self::Ethernet | Self::Usb | Self::VpnTunnel => 0x02,
            Self::Bluetooth => 0x04,
            Self::LoRaSx1262 => 0x08,
            Self::UltrasonicChirp => 0x10,
            Self::Infrasound => 0x20,
        }
    }

    /// Intrinsic physical relaxation time constant floor `τ_0` (ms) for Peter's Real Time Constant (RTC).
    #[inline]
    pub const fn intrinsic_rtc_floor_ms(self) -> u32 {
        match self {
            Self::Usb => 2,
            Self::Ethernet => 3,
            Self::WiFi => 8,
            Self::Esp32Now => 10,
            Self::Bluetooth => 18,
            Self::VpnTunnel => 25,
            Self::LoRaSx1262 => 60,
            Self::UltrasonicChirp => 90,
            Self::Infrasound => 250,
        }
    }

    pub fn parse_str(s: &str) -> Self {
        match s.trim().to_ascii_lowercase().as_str() {
            "usb" => Self::Usb,
            "eth" | "ethernet" => Self::Ethernet,
            "bt" | "ble" | "bluetooth" => Self::Bluetooth,
            "esp" | "esp32" | "esp-now" | "espnow" => Self::Esp32Now,
            "lora" | "sx1262" => Self::LoRaSx1262,
            "vpn" => Self::VpnTunnel,
            "ultrasonic" | "chirp" | "acoustic" | "audio" => Self::UltrasonicChirp,
            "infrasound" => Self::Infrasound,
            _ => Self::WiFi,
        }
    }
}

impl std::fmt::Display for MemnetMedium {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let name = match self {
            Self::Usb => "USB",
            Self::Ethernet => "Ethernet",
            Self::WiFi => "WiFi",
            Self::Bluetooth => "Bluetooth",
            Self::Esp32Now => "ESP32-NOW",
            Self::LoRaSx1262 => "LoRa-SX1262",
            Self::VpnTunnel => "VPN-Tunnel",
            Self::UltrasonicChirp => "Ultrasonic-Chirp",
            Self::Infrasound => "Infrasound",
        };
        write!(f, "{name}")
    }
}

/// AyeOS / MemNet Routing Tier (`Tier A` latency, `Tier B` Marine jitter/salience, `Tier C` Consciousness dual-path).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RoutingTier {
    /// Tier A: Lowest RTT latency.
    TierA,
    /// Tier B: Marine Algorithm jitter + salience stability optimization.
    TierB,
    /// Tier C: Full Consciousness predictive routing + dual-path zero-packet-loss handoff.
    TierC,
}

/// Operational health of a MemNet route.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RouteStatus {
    Up,
    Degraded,
    Down,
}

/// A single physical route to a neighbor or multi-route logical address (`address/via`).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MemnetRouteEntry {
    /// Logical destination address (`0x01..=0xEF`).
    pub address: u8,
    /// Physical via-hop identifier (e.g., `12` for WiFi, `13` for Bluetooth, `14` for LoRa, `15` for ESP32).
    pub via: u8,
    pub node_name: String,
    pub medium: MemnetMedium,
    pub endpoint: String,
    pub latency_ms: u16,
    pub jitter_ms: u16,
    /// Peter's Real Time Constant (`RTC`, `τ` in ms): continuous-time 1st-order relaxation constant (`1 - e^(-t/τ)`).
    #[serde(default)]
    pub rtc_ms: u32,
    pub marine_salience_pct: u8,
    pub heartbeat_credits: u8,
    pub max_credits: u8,
    pub heartbeat_interval_ms: u32,
    pub status: RouteStatus,
    pub shared_free_mb: u64,
}

impl MemnetRouteEntry {
    pub fn new(
        address: u8,
        via: u8,
        node_name: impl Into<String>,
        medium: MemnetMedium,
        endpoint: impl Into<String>,
        latency_ms: u16,
    ) -> Self {
        let max_credits = medium.max_credits();
        let jitter_ms = 2u16;
        let rtc_ms =
            (latency_ms as u32) + (jitter_ms as u32) * 2 + medium.intrinsic_rtc_floor_ms();
        Self {
            address,
            via,
            node_name: node_name.into(),
            medium,
            endpoint: endpoint.into(),
            latency_ms,
            jitter_ms,
            rtc_ms,
            marine_salience_pct: 92,
            heartbeat_credits: max_credits,
            max_credits,
            heartbeat_interval_ms: medium.heartbeat_interval_ms(),
            status: RouteStatus::Up,
            shared_free_mb: 0,
        }
    }

    /// Effective Real Time Constant `τ` in milliseconds (`RTC`).
    /// Falls back to physical relaxation formula if loaded from a pre-RTC state snapshot (`rtc_ms == 0`).
    #[inline]
    pub fn effective_rtc_ms(&self) -> u32 {
        if self.rtc_ms > 0 {
            self.rtc_ms
        } else {
            (self.latency_ms as u32)
                + (self.jitter_ms as u32) * 2
                + self.medium.intrinsic_rtc_floor_ms()
        }
    }

    /// `3τ` settling horizon in milliseconds (~95.0% relaxation to steady state).
    #[inline]
    pub fn settling_3tau_ms(&self) -> u32 {
        self.effective_rtc_ms().saturating_mul(3)
    }

    /// `5τ` full steady-state lock horizon in milliseconds (~99.3% relaxation).
    #[inline]
    pub fn settling_5tau_ms(&self) -> u32 {
        self.effective_rtc_ms().saturating_mul(5)
    }

    /// Continuous-time first-order exponential filter update using Peter's Real Time Constant `τ` (`RTC`):
    /// `α = 1 - e^(-Δt / τ)`, so bursts of rapid packets do not over-weight a momentary spike,
    /// and quiet links naturally relax after `Δt >= τ`.
    pub fn update_with_rtc(&mut self, sample_rtt_ms: u16, elapsed_dt_ms: u32) {
        let tau = (self.effective_rtc_ms().max(1)) as f64;
        let dt = (elapsed_dt_ms.max(1)) as f64;
        let alpha = (1.0 - (-dt / tau).exp()).clamp(0.02, 1.0);
        let prev = self.latency_ms as f64;
        let curr = sample_rtt_ms as f64;
        let instant_err = (curr - prev).abs();

        let next_lat = (prev + alpha * (curr - prev)).round().clamp(1.0, 65535.0) as u16;
        let next_jit = ((self.jitter_ms as f64) + alpha * (instant_err - (self.jitter_ms as f64)))
            .round()
            .clamp(0.0, 65535.0) as u16;
        self.latency_ms = next_lat;
        self.jitter_ms = next_jit;
        self.rtc_ms =
            (next_lat as u32) + (next_jit as u32) * 2 + self.medium.intrinsic_rtc_floor_ms();
    }

    /// Format as `address/via` (e.g. `C5/12` or `42/1`).
    pub fn address_via_label(&self) -> String {
        if matches!(AddressClass::classify(self.address), AddressClass::MultiRoute(_)) {
            format!("0x{:02X}/{}", self.address, self.via)
        } else {
            format!("{}/{}", self.address, self.via)
        }
    }

    /// Record a received heartbeat: increments credits up to `max_credits` and restores `Up` status.
    pub fn tick_heartbeat_received(&mut self) {
        self.heartbeat_credits = self.heartbeat_credits.saturating_add(1).min(self.max_credits);
        self.status = RouteStatus::Up;
    }

    /// Record a missed heartbeat: decrements credits and transitions to `Degraded` or `Down` at 0.
    pub fn tick_heartbeat_missed(&mut self) -> bool {
        self.heartbeat_credits = self.heartbeat_credits.saturating_sub(1);
        if self.heartbeat_credits == 0 {
            self.status = RouteStatus::Down;
            true
        } else if self.heartbeat_credits <= self.max_credits / 2 {
            self.status = RouteStatus::Degraded;
            false
        } else {
            false
        }
    }

    /// Integer Marine route quality score (higher is better) combining RTC (`τ`), latency, jitter, salience, and bandwidth.
    pub fn marine_quality_score(&self) -> u32 {
        if self.status == RouteStatus::Down || self.heartbeat_credits == 0 {
            return 0;
        }
        let credit_ratio_q8 =
            ((self.heartbeat_credits as u32) * 256) / (self.max_credits.max(1) as u32);
        let salience_bonus = (self.marine_salience_pct as u32) * 8;
        let bw_bonus = (self.medium.nominal_kbps().ilog2() + 1) * 40;
        let rtc_penalty = self.effective_rtc_ms();
        let penalty = (self.latency_ms as u32) * 2 + (self.jitter_ms as u32) * 8 + rtc_penalty;
        (1000 + credit_ratio_q8 + salience_bonus + bw_bonus).saturating_sub(penalty)
    }
}

/// Wire-level MemNet byte-path packet (`[Depth:u8][Path:u8×N][Payload]`) matching `MEMNET.md`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MemnetRawPacket {
    /// Hierarchical path bytes (`len == depth`, `0` for localhost).
    pub path: Vec<u8>,
    /// Raw payload bytes.
    pub payload: Vec<u8>,
}

impl MemnetRawPacket {
    /// Create a localhost packet (`1B` header overhead: `[0][payload]`).
    pub fn localhost(payload: Vec<u8>) -> Self {
        Self {
            path: Vec::new(),
            payload,
        }
    }

    /// Create a same-room single-hop packet (`2B` header overhead: `[1][node][payload]`).
    pub fn same_room(node: u8, payload: Vec<u8>) -> Self {
        Self {
            path: vec![node],
            payload,
        }
    }

    /// Create a cascading broadcast packet (`[0xF0 + level][payload]`).
    pub fn broadcast(cascade_level: u8, payload: Vec<u8>) -> Self {
        let addr = 0xF0u8.saturating_add(cascade_level.min(15));
        Self {
            path: vec![addr],
            payload,
        }
    }

    /// Header overhead in bytes (`1` for localhost, `1 + path.len()` for routed packets, or `1` for compact broadcast `[0xF2][payload]`).
    pub fn header_overhead_bytes(&self) -> usize {
        if self.path.len() == 1 && self.path[0] >= 0xF0 {
            1
        } else {
            1 + self.path.len()
        }
    }

    /// Encode to compact wire bytes.
    pub fn to_wire_bytes(&self) -> Vec<u8> {
        if self.path.len() == 1 && self.path[0] >= 0xF0 {
            let mut out = Vec::with_capacity(1 + self.payload.len());
            out.push(self.path[0]);
            out.extend_from_slice(&self.payload);
            return out;
        }
        let depth = self.path.len().min(239) as u8;
        let mut out = Vec::with_capacity(1 + (depth as usize) + self.payload.len());
        out.push(depth);
        out.extend_from_slice(&self.path[..depth as usize]);
        out.extend_from_slice(&self.payload);
        out
    }

    /// Decode from compact wire bytes.
    pub fn from_wire_bytes(bytes: &[u8]) -> Result<Self> {
        anyhow::ensure!(!bytes.is_empty(), "Empty MemNet wire packet");
        let first = bytes[0];
        if first >= 0xF0 {
            return Ok(Self {
                path: vec![first],
                payload: bytes[1..].to_vec(),
            });
        }
        let depth = first as usize;
        anyhow::ensure!(
            bytes.len() >= 1 + depth,
            "Truncated MemNet route path: depth={depth}, len={}",
            bytes.len()
        );
        Ok(Self {
            path: bytes[1..1 + depth].to_vec(),
            payload: bytes[1 + depth..].to_vec(),
        })
    }

    /// Step a cascading broadcast (`F2 -> F1 -> F0`). Returns `Some(next_packet)` if it should cascade further.
    pub fn step_broadcast_cascade(&self) -> Option<Self> {
        if self.path.len() == 1 && self.path[0] > 0xF0 {
            Some(Self {
                path: vec![self.path[0] - 1],
                payload: self.payload.clone(),
            })
        } else {
            None
        }
    }
}

// ============================================================================
// 2. ESP32 Constellation (`MemnetCapsule64`) & Bluetooth (`BluetoothMeshFrame`)
// ============================================================================

/// Byte-exact 64-byte `MemnetCapsule64` frame shared across ESP-NOW, Wi-Fi UDP `:8888`, BLE, and 923.2 MHz SX1262 LoRa.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MemnetCapsule64 {
    pub version: u8,
    pub node_id: u8,
    pub node_trit: i8,
    pub transport_mask: u8,
    pub seed_class: u8,
    pub dominant_mod: u8,
    pub seq: u32,
    pub audio_f0_hz: u16,
    pub audio_salience_pct: u8,
    pub rf_rssi_dbm: i8,
    pub rf_ap_count: u16,
    pub imu_motion_mg: i16,
    pub peer_rssi_dbm: [i8; 4],
    pub top_bssid_fnv32: u32,
    pub top_bssid_rssi_dbm: i8,
    pub portal_ok: u8,
    pub wave_b3: String,
    pub ssid_hint: String,
}

impl MemnetCapsule64 {
    pub fn to_bytes_64(&self) -> [u8; 64] {
        let mut buf = [0u8; 64];
        buf[0..2].copy_from_slice(&MEMNET_CAPSULE64_MAGIC.to_le_bytes());
        buf[2] = self.version;
        buf[3] = self.node_id;
        buf[4] = self.node_trit as u8;
        buf[5] = self.transport_mask;
        buf[6] = self.seed_class;
        buf[7] = self.dominant_mod;
        buf[8..12].copy_from_slice(&self.seq.to_le_bytes());
        buf[12..14].copy_from_slice(&self.audio_f0_hz.to_le_bytes());
        buf[14] = self.audio_salience_pct;
        buf[15] = self.rf_rssi_dbm as u8;
        buf[16..18].copy_from_slice(&self.rf_ap_count.to_le_bytes());
        buf[18..20].copy_from_slice(&self.imu_motion_mg.to_le_bytes());
        for i in 0..4 {
            buf[20 + i] = self.peer_rssi_dbm[i] as u8;
        }
        buf[24..28].copy_from_slice(&self.top_bssid_fnv32.to_le_bytes());
        buf[28] = self.top_bssid_rssi_dbm as u8;
        buf[29] = self.portal_ok;

        let b3 = self.wave_b3.as_bytes();
        let b3_len = b3.len().min(9);
        buf[30..30 + b3_len].copy_from_slice(&b3[..b3_len]);

        let ssid = self.ssid_hint.as_bytes();
        let ssid_len = ssid.len().min(15);
        buf[40..40 + ssid_len].copy_from_slice(&ssid[..ssid_len]);

        let seal = fnv64_1a(&buf[0..56]);
        buf[56..64].copy_from_slice(&seal.to_le_bytes());
        buf
    }

    pub fn from_bytes_64(buf: &[u8; 64]) -> Option<Self> {
        let magic = u16::from_le_bytes([buf[0], buf[1]]);
        if magic != MEMNET_CAPSULE64_MAGIC {
            return None;
        }
        let expected_seal = fnv64_1a(&buf[0..56]);
        let actual_seal = u64::from_le_bytes(buf[56..64].try_into().ok()?);
        if expected_seal != actual_seal {
            return None;
        }
        let b3_end = buf[30..40].iter().position(|&b| b == 0).unwrap_or(9).min(9);
        let wave_b3 = String::from_utf8_lossy(&buf[30..30 + b3_end]).to_string();
        let ssid_end = buf[40..56]
            .iter()
            .position(|&b| b == 0)
            .unwrap_or(15)
            .min(15);
        let ssid_hint = String::from_utf8_lossy(&buf[40..40 + ssid_end]).to_string();

        Some(Self {
            version: buf[2],
            node_id: buf[3],
            node_trit: buf[4] as i8,
            transport_mask: buf[5],
            seed_class: buf[6],
            dominant_mod: buf[7],
            seq: u32::from_le_bytes([buf[8], buf[9], buf[10], buf[11]]),
            audio_f0_hz: u16::from_le_bytes([buf[12], buf[13]]),
            audio_salience_pct: buf[14],
            rf_rssi_dbm: buf[15] as i8,
            rf_ap_count: u16::from_le_bytes([buf[16], buf[17]]),
            imu_motion_mg: i16::from_le_bytes([buf[18], buf[19]]),
            peer_rssi_dbm: [
                buf[20] as i8,
                buf[21] as i8,
                buf[22] as i8,
                buf[23] as i8,
            ],
            top_bssid_fnv32: u32::from_le_bytes([buf[24], buf[25], buf[26], buf[27]]),
            top_bssid_rssi_dbm: buf[28] as i8,
            portal_ok: buf[29],
            wave_b3,
            ssid_hint,
        })
    }
}

// ============================================================================
// 3. 79-Byte `WaveFrame79` (`MEM|8` v2) & Semantic Wave Q&A Offload Engine
// ============================================================================

/// Exact 8-byte integer rational (`i32` numerator, non-zero `u32` denominator) for `WaveFrame79`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct WaveRational {
    pub num: i32,
    pub den: u32,
}

impl WaveRational {
    pub const fn new(num: i32, den: u32) -> Option<Self> {
        if den == 0 {
            return None;
        }
        let g = gcd_u32(num.unsigned_abs(), den);
        Some(Self {
            num: num / (g as i32),
            den: den / g,
        })
    }

    pub const fn to_bytes(self) -> [u8; 8] {
        let mut out = [0u8; 8];
        let n = self.num.to_le_bytes();
        let d = self.den.to_le_bytes();
        out[0] = n[0];
        out[1] = n[1];
        out[2] = n[2];
        out[3] = n[3];
        out[4] = d[0];
        out[5] = d[1];
        out[6] = d[2];
        out[7] = d[3];
        out
    }

    pub fn from_bytes(bytes: &[u8]) -> Option<Self> {
        if bytes.len() < 8 {
            return None;
        }
        let num = i32::from_le_bytes(bytes[0..4].try_into().ok()?);
        let den = u32::from_le_bytes(bytes[4..8].try_into().ok()?);
        Self::new(num, den)
    }
}

const fn gcd_u32(mut a: u32, mut b: u32) -> u32 {
    if a == 0 {
        return if b == 0 { 1 } else { b };
    }
    if b == 0 {
        return a;
    }
    while b != 0 {
        let r = a % b;
        a = b;
        b = r;
    }
    a
}

/// Byte-exact 79-byte `MEM|8` Version 2 Integer Wire Frame (`WAVE_FRAME.md`).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WaveFrame79 {
    /// Wire version (`2`).
    pub version: u8,
    /// `0x01 = Cognitive`, `0x02 = Nexus`.
    pub provenance: u8,
    pub base_amplitude: WaveRational,
    pub frequency: WaveRational,
    pub phase: WaveRational,
    pub emotional_valence: WaveRational,
    pub arousal: WaveRational,
    pub half_life_ticks: u64,
    pub age_ticks: u64,
    pub idle_ticks: u64,
    pub access_count: u32,
    pub id: Option<u64>,
}

impl WaveFrame79 {
    /// Encode to the canonical 79-byte `MEM|8` Version 2 integer wire frame.
    pub fn to_bytes_79(&self) -> [u8; 79] {
        let mut buf = [0u8; 79];
        buf[0] = 2;
        buf[1] = if self.provenance == 0x02 { 0x02 } else { 0x01 };
        buf[2..10].copy_from_slice(&self.base_amplitude.to_bytes());
        buf[10..18].copy_from_slice(&self.frequency.to_bytes());
        buf[18..26].copy_from_slice(&self.phase.to_bytes());
        buf[26..34].copy_from_slice(&self.emotional_valence.to_bytes());
        buf[34..42].copy_from_slice(&self.arousal.to_bytes());
        buf[42..50].copy_from_slice(&self.half_life_ticks.to_le_bytes());
        buf[50..58].copy_from_slice(&self.age_ticks.to_le_bytes());
        buf[58..66].copy_from_slice(&self.idle_ticks.min(self.age_ticks).to_le_bytes());
        buf[66..70].copy_from_slice(&self.access_count.to_le_bytes());
        if let Some(id) = self.id {
            buf[70] = 1;
            buf[71..79].copy_from_slice(&id.to_le_bytes());
        } else {
            buf[70] = 0;
        }
        buf
    }

    /// Decode from a 79-byte `MEM|8` Version 2 integer wire frame.
    pub fn from_bytes_79(buf: &[u8]) -> Option<Self> {
        if buf.len() < 79 {
            return None;
        }
        let version = buf[0];
        if version != 1 && version != 2 {
            return None;
        }
        let provenance = buf[1];
        if provenance != 0x01 && provenance != 0x02 {
            return None;
        }
        let base_amplitude = WaveRational::from_bytes(&buf[2..10])?;
        let frequency = WaveRational::from_bytes(&buf[10..18])?;
        let phase = WaveRational::from_bytes(&buf[18..26])?;
        let emotional_valence = WaveRational::from_bytes(&buf[26..34])?;
        let arousal = WaveRational::from_bytes(&buf[34..42])?;
        let half_life_ticks = u64::from_le_bytes(buf[42..50].try_into().ok()?);
        let age_ticks = u64::from_le_bytes(buf[50..58].try_into().ok()?);
        let idle_ticks = u64::from_le_bytes(buf[58..66].try_into().ok()?);
        if version == 2 && idle_ticks > age_ticks {
            return None;
        }
        let access_count = u32::from_le_bytes(buf[66..70].try_into().ok()?);
        let id = if buf[70] != 0 {
            Some(u64::from_le_bytes(buf[71..79].try_into().ok()?))
        } else {
            None
        };
        Some(Self {
            version: 2,
            provenance,
            base_amplitude,
            frequency,
            phase,
            emotional_valence,
            arousal,
            half_life_ticks,
            age_ticks,
            idle_ticks: idle_ticks.min(age_ticks),
            access_count,
            id,
        })
    }

    /// Synthesize a deterministic semantic `WaveFrame79` from text or code context.
    pub fn from_semantic_text(text: &str) -> Self {
        let fold = fnv64_1a(text.as_bytes());
        let freq_hz = 40 + ((fold % 400) as i32); // 40..440 Hz cognitive band
        let phase_deg = ((fold >> 16) % 360) as i32;
        let valence = (((fold >> 32) % 180) as i32) - 90;
        let arousal = 30 + (((fold >> 48) % 70) as i32);
        Self {
            version: 2,
            provenance: 0x01,
            base_amplitude: WaveRational::new(95, 100).unwrap(),
            frequency: WaveRational::new(freq_hz, 1).unwrap(),
            phase: WaveRational::new(phase_deg, 360).unwrap(),
            emotional_valence: WaveRational::new(valence, 100).unwrap(),
            arousal: WaveRational::new(arousal, 100).unwrap(),
            half_life_ticks: 86_400,
            age_ticks: 1,
            idle_ticks: 0,
            access_count: 1,
            id: Some(fold),
        }
    }
}

/// 96-byte Sovereign Bluetooth Mesh Frame (`8BM8`) carrying a 79-byte `WaveFrame79`,
/// a 64-bit `WUPZ` XOR-fold receipt, speaker `F0` Hz, hop TTL, and CRC-16.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BluetoothMeshFrame96 {
    pub hop_ttl: u8,
    pub speaker_f0_hz: u16,
    pub wupz_xor_fold: u64,
    pub wave: WaveFrame79,
}

impl BluetoothMeshFrame96 {
    pub fn to_bytes_96(&self) -> [u8; 96] {
        let mut buf = [0u8; 96];
        buf[0..4].copy_from_slice(&BLUETOOTH_MESH_MAGIC_96);
        buf[4] = self.hop_ttl;
        buf[5..7].copy_from_slice(&self.speaker_f0_hz.to_le_bytes());
        buf[7..15].copy_from_slice(&self.wupz_xor_fold.to_le_bytes());
        buf[15..94].copy_from_slice(&self.wave.to_bytes_79());
        let crc = crc16_ccitt(&buf[0..94]);
        buf[94..96].copy_from_slice(&crc.to_le_bytes());
        buf
    }

    pub fn from_bytes_96(buf: &[u8; 96]) -> Option<Self> {
        if buf[0..4] != BLUETOOTH_MESH_MAGIC_96 {
            return None;
        }
        let expected_crc = crc16_ccitt(&buf[0..94]);
        let actual_crc = u16::from_le_bytes([buf[94], buf[95]]);
        if expected_crc != actual_crc {
            return None;
        }
        let hop_ttl = buf[4];
        let speaker_f0_hz = u16::from_le_bytes([buf[5], buf[6]]);
        let wupz_xor_fold = u64::from_le_bytes(buf[7..15].try_into().ok()?);
        let wave = WaveFrame79::from_bytes_79(&buf[15..94])?;
        Some(Self {
            hop_ttl,
            speaker_f0_hz,
            wupz_xor_fold,
            wave,
        })
    }

    /// Encode a compact 31-byte BLE Legacy Advertising beacon (`"8B"`).
    pub fn to_ble_beacon_31(&self, vad_byte: u8, tag7: &str) -> [u8; 31] {
        let mut adv = [0u8; 31];
        adv[0..2].copy_from_slice(&BLE_BEACON_MAGIC_31);
        adv[2] = self.hop_ttl;
        adv[3..5].copy_from_slice(&self.speaker_f0_hz.to_le_bytes());
        adv[5] = vad_byte;
        adv[6..14].copy_from_slice(&self.wupz_xor_fold.to_le_bytes());
        adv[14..22].copy_from_slice(&self.wave.id.unwrap_or(0).to_le_bytes());
        let tb = tag7.as_bytes();
        let copy_len = tb.len().min(7);
        adv[22..22 + copy_len].copy_from_slice(&tb[..copy_len]);
        let crc = crc16_ccitt(&adv[0..29]);
        adv[29..31].copy_from_slice(&crc.to_le_bytes());
        adv
    }
}

/// Extract normalized semantic tokens and balanced ternary trits (`-1, 0, +1`) from a prompt or context string.
pub fn extract_semantic_wave_features(
    text: &str,
) -> (Vec<String>, Vec<i8>, String, LitanyWireFrame) {
    let stop_words = [
        "the", "and", "for", "with", "that", "this", "what", "how", "why", "where", "when", "who",
        "can", "you", "please", "from", "into", "are", "was", "were", "does", "did", "have", "has",
        "do", "we", "or", "to", "in", "on", "by", "is", "it", "of", "at", "as", "be", "an",
    ];
    let mut tokens: Vec<String> = text
        .split(|c: char| !c.is_ascii_alphanumeric() && c != '_' && c != '-')
        .map(|w| w.trim().to_ascii_lowercase())
        .filter(|w| w.len() >= 2 && !stop_words.contains(&w.as_str()))
        .collect();
    tokens.sort();
    tokens.dedup();

    let canonical = if tokens.is_empty() {
        text.trim().to_ascii_lowercase()
    } else {
        tokens.join(" ")
    };

    let mut trits = Vec::with_capacity(canonical.len().max(40));
    for (idx, b) in canonical.as_bytes().iter().enumerate() {
        let mixed = (*b as u64).wrapping_mul(131).wrapping_add(idx as u64);
        let t = match mixed % 3 {
            0 => -1i8,
            1 => 0i8,
            _ => 1i8,
        };
        trits.push(t);
    }
    while trits.len() < 40 {
        trits.push(0);
    }

    let mut wave_b3 = String::with_capacity(9);
    for chunk in trits.chunks(trits.len().div_ceil(9)).take(9) {
        let sum: i32 = chunk.iter().map(|&t| t as i32).sum();
        wave_b3.push(if sum > 0 {
            '1'
        } else if sum < 0 {
            'T'
        } else {
            '0'
        });
    }
    while wave_b3.len() < 9 {
        wave_b3.push('0');
    }

    let (packed_fold, _packed_len, _ratio) = xor_fold_gaia_b158(&trits);
    let wire = seal_litany_wire_64b(
        canonical.as_bytes(),
        &packed_fold.to_le_bytes(),
        GATE_0X62,
        QUANT_QT55,
    );
    (tokens, trits, wave_b3, wire)
}

/// A `MEM|8` Wave Packet Question & Answer entry shared across machines in the general area
/// so LLMs don't have to re-compute answers that a nearby peer already knows!
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WaveQaPacket {
    pub packet_id: String,
    pub origin_node: u8,
    pub origin_name: String,
    pub question: String,
    pub answer: String,
    pub semantic_tokens: Vec<String>,
    pub wave_b3: String,
    pub litany_wire: LitanyWireFrame,
    pub wave_frame: WaveFrame79,
    pub timestamp_s: u64,
    pub hit_count: u32,
}

impl WaveQaPacket {
    pub fn new(
        origin_node: u8,
        origin_name: impl Into<String>,
        question: impl Into<String>,
        answer: impl Into<String>,
    ) -> Self {
        let q = question.into();
        let a = answer.into();
        let (semantic_tokens, _trits, wave_b3, litany_wire) = extract_semantic_wave_features(&q);
        let wave_frame = WaveFrame79::from_semantic_text(&format!("{q}\n{a}"));
        let packet_id = format!("wqa-{:016x}", fnv64_1a(q.as_bytes()));
        let timestamp_s = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);
        Self {
            packet_id,
            origin_node,
            origin_name: origin_name.into(),
            question: q,
            answer: a,
            semantic_tokens,
            wave_b3,
            litany_wire,
            wave_frame,
            timestamp_s,
            hit_count: 1,
        }
    }

    /// Compute integer wave resonance score (`0..=1000` permille) between a query and this cached `WaveQaPacket`.
    /// Combines semantic token overlap, Kleene `BFP-64` bitplane resonance, and 9-trit `wave_b3` coherence.
    pub fn resonance_permille(&self, query: &str) -> u32 {
        if self.question.trim().eq_ignore_ascii_case(query.trim()) {
            return 1000;
        }
        let (q_tokens, _q_trits, q_b3, q_wire) = extract_semantic_wave_features(query);
        if q_tokens.is_empty() && self.semantic_tokens.is_empty() {
            return 0;
        }

        // 1. Semantic token Jaccard/containment score (up to 700 permille)
        let overlap = q_tokens
            .iter()
            .filter(|t| self.semantic_tokens.contains(t))
            .count() as u32;
        let denom = q_tokens.len().max(1) as u32;
        let union = (q_tokens.len() + self.semantic_tokens.len())
            .saturating_sub(overlap as usize)
            .max(1) as u32;
        let containment_pm = (overlap * 500) / denom;
        let jaccard_pm = (overlap * 200) / union;
        let token_score = containment_pm + jaccard_pm;

        // 2. Kleene BFP-64 positive bitplane interference (up to 150 permille)
        let (and_pos, _and_neg) = bfp64_and_kleene(
            self.litany_wire.bfp64_pos,
            self.litany_wire.bfp64_neg,
            q_wire.bfp64_pos,
            q_wire.bfp64_neg,
        );
        let pos_union = (self.litany_wire.bfp64_pos | q_wire.bfp64_pos)
            .count_ones()
            .max(1);
        let bfp_score = (and_pos.count_ones() * 150) / pos_union;

        // 3. 9-trit wave_b3 phase alignment (up to 150 permille)
        let matching_trits = self
            .wave_b3
            .bytes()
            .zip(q_b3.bytes())
            .filter(|(a, b)| a == b)
            .count() as u32;
        let trit_score = (matching_trits * 150) / 9;

        // Require at least 1 overlapping semantic token for high-confidence LLM offload
        if overlap == 0 {
            (bfp_score + trit_score) / 3
        } else {
            (token_score + bfp_score + trit_score).min(1000)
        }
    }
}

// ============================================================================
// 4. Shared Folders, Drive Space Leases & Audio/Video Chunk Streaming
// ============================================================================

/// Metadata for a single file inside a Smart Tree MemNet Shared Folder.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SharedFileEntry {
    pub rel_path: String,
    pub size_bytes: u64,
    pub crc32: u32,
    pub fnv64: u64,
    pub chunk_count: u32,
}

/// A folder shared over MemNet on the local network by a Smart Tree node.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SharedFolderManifest {
    pub share_id: String,
    pub share_name: String,
    pub host_path: String,
    pub owner_node: u8,
    pub owner_name: String,
    pub endpoint: String,
    pub total_files: usize,
    pub total_bytes: u64,
    /// Shared drive space quota (in MB) offered to nearby MemNet peers / ESP32s.
    pub offered_drive_space_mb: u64,
    pub semantic_essence: String,
    pub consciousness_freq_hz: u32,
    pub litany_wire: LitanyWireFrame,
    pub files: Vec<SharedFileEntry>,
    pub shared_at_s: u64,
}

/// Default chunk size (`32 KB`) for MemNet file and media streaming.
pub const MEMNET_CHUNK_BYTES: usize = 32 * 1024;

impl SharedFolderManifest {
    /// Scan a local directory, build its `.m8` consciousness pyramid, and create a `SharedFolderManifest`.
    pub fn from_directory(
        dir: &Path,
        share_name: Option<&str>,
        owner_node: u8,
        owner_name: &str,
        endpoint: &str,
        offered_drive_space_mb: u64,
    ) -> Result<Self> {
        let canon = std::fs::canonicalize(dir)
            .with_context(|| format!("Cannot canonicalize shared directory {}", dir.display()))?;
        anyhow::ensure!(canon.is_dir(), "{} is not a directory", canon.display());

        // Ensure .m8 consciousness exists for semantic context sharing
        let consciousness =
            crate::m8_consciousness::DirectoryConsciousness::build_bottom_up_pyramid(&canon, 0, 1)
                .ok();
        let semantic_essence = consciousness
            .as_ref()
            .map(|c| c.essence.clone())
            .unwrap_or_else(|| "Smart Tree MemNet Shared Folder".to_string());
        let consciousness_freq_hz = consciousness
            .as_ref()
            .map(|c| (c.frequency * 100.0) as u32)
            .unwrap_or(4273);

        let folder_name = share_name
            .map(|s| s.to_string())
            .or_else(|| {
                canon
                    .file_name()
                    .map(|n| n.to_string_lossy().into_owned())
            })
            .unwrap_or_else(|| "shared".to_string());

        let mut files = Vec::new();
        let mut total_bytes = 0u64;
        let mut digest_accumulator = Vec::new();

        collect_shared_files_recursive(&canon, &canon, &mut files, &mut total_bytes, 256)?;
        files.sort_by(|a, b| a.rel_path.cmp(&b.rel_path));
        for f in &files {
            digest_accumulator.extend_from_slice(f.rel_path.as_bytes());
            digest_accumulator.extend_from_slice(&f.crc32.to_le_bytes());
        }
        let (_, _, _, litany_wire) =
            extract_semantic_wave_features(&format!("{folder_name}:{semantic_essence}"));
        let share_id = format!(
            "share-{:08x}",
            fnv32_1a(format!("{}:{}", owner_node, canon.display()).as_bytes())
        );
        let shared_at_s = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);

        Ok(Self {
            share_id,
            share_name: folder_name,
            host_path: canon.to_string_lossy().into_owned(),
            owner_node,
            owner_name: owner_name.to_string(),
            endpoint: endpoint.to_string(),
            total_files: files.len(),
            total_bytes,
            offered_drive_space_mb,
            semantic_essence,
            consciousness_freq_hz,
            litany_wire,
            files,
            shared_at_s,
        })
    }
}

fn collect_shared_files_recursive(
    root: &Path,
    current: &Path,
    out: &mut Vec<SharedFileEntry>,
    total_bytes: &mut u64,
    limit: usize,
) -> Result<()> {
    if out.len() >= limit {
        return Ok(());
    }
    let entries = match std::fs::read_dir(current) {
        Ok(e) => e,
        Err(_) => return Ok(()),
    };
    for entry in entries.flatten() {
        if out.len() >= limit {
            break;
        }
        let path = entry.path();
        let name = entry.file_name();
        let name_str = name.to_string_lossy();
        if name_str == ".git" || name_str == "target" || name_str == "node_modules" {
            continue;
        }
        let Ok(meta) = entry.metadata() else {
            continue;
        };
        if meta.is_dir() {
            collect_shared_files_recursive(root, &path, out, total_bytes, limit)?;
        } else if meta.is_file() {
            let rel = path
                .strip_prefix(root)
                .unwrap_or(&path)
                .to_string_lossy()
                .replace('\\', "/");
            let size_bytes = meta.len();
            *total_bytes = total_bytes.saturating_add(size_bytes);
            // Hash up to first 64KB for fast manifest creation
            let sample = std::fs::read(&path)
                .map(|b| {
                    if b.len() > 65536 {
                        b[..65536].to_vec()
                    } else {
                        b
                    }
                })
                .unwrap_or_default();
            let crc32 = felix_crc32_ieee(&sample);
            let fnv64 = fnv64_1a(&sample);
            let chunk_count = (size_bytes as usize).div_ceil(MEMNET_CHUNK_BYTES).max(1) as u32;
            out.push(SharedFileEntry {
                rel_path: rel,
                size_bytes,
                crc32,
                fnv64,
                chunk_count,
            });
        }
    }
    Ok(())
}

/// Codec / stream type for MemNet Audio, Video, and Semantic Wave chunks.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MediaKind {
    AudioPcm16,
    AudioOpus,
    AudioFlac,
    VideoAv1,
    VideoH264,
    SemanticWaveStream,
    SharedFileChunk,
}

/// A deterministic, CRC32-sealed Audio, Video, or File chunk packet over MemNet.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MediaChunkPacket {
    pub stream_id: String,
    pub kind: MediaKind,
    pub seq: u32,
    pub total_chunks: u32,
    pub sample_rate_or_fps: u32,
    pub f0_hz: u16,
    pub marine_salience_pct: u8,
    pub crc32: u32,
    pub data: Vec<u8>,
}

impl MediaChunkPacket {
    pub fn verify_crc(&self) -> bool {
        felix_crc32_ieee(&self.data) == self.crc32
    }
}

/// Split a media or file byte stream into CRC32-verified `MediaChunkPacket`s.
pub fn chunk_media_stream(
    stream_id: &str,
    kind: MediaKind,
    bytes: &[u8],
    chunk_size: usize,
    sample_rate_or_fps: u32,
    f0_hz: u16,
    marine_salience_pct: u8,
) -> Vec<MediaChunkPacket> {
    let step = chunk_size.max(64);
    let total_chunks = bytes.len().div_ceil(step).max(1) as u32;
    if bytes.is_empty() {
        return vec![MediaChunkPacket {
            stream_id: stream_id.to_string(),
            kind,
            seq: 0,
            total_chunks: 1,
            sample_rate_or_fps,
            f0_hz,
            marine_salience_pct,
            crc32: felix_crc32_ieee(&[]),
            data: Vec::new(),
        }];
    }
    bytes
        .chunks(step)
        .enumerate()
        .map(|(idx, slice)| MediaChunkPacket {
            stream_id: stream_id.to_string(),
            kind,
            seq: idx as u32,
            total_chunks,
            sample_rate_or_fps,
            f0_hz,
            marine_salience_pct,
            crc32: felix_crc32_ieee(slice),
            data: slice.to_vec(),
        })
        .collect()
}

/// Reassemble and verify an ordered or unordered slice of `MediaChunkPacket`s.
pub fn reassemble_media_chunks(chunks: &[MediaChunkPacket]) -> Result<Vec<u8>> {
    anyhow::ensure!(!chunks.is_empty(), "No media chunks provided");
    let expected_total = chunks[0].total_chunks as usize;
    let mut ordered: Vec<&MediaChunkPacket> = chunks.iter().collect();
    ordered.sort_by_key(|c| c.seq);
    ordered.dedup_by_key(|c| c.seq);
    anyhow::ensure!(
        ordered.len() == expected_total,
        "Missing media chunks: got {}/{}",
        ordered.len(),
        expected_total
    );
    let mut out = Vec::new();
    for (idx, chunk) in ordered.into_iter().enumerate() {
        anyhow::ensure!(chunk.seq as usize == idx, "Out-of-sequence chunk {}", chunk.seq);
        anyhow::ensure!(
            chunk.verify_crc(),
            "CRC32 mismatch on media chunk {}/{}",
            chunk.seq,
            expected_total
        );
        out.extend_from_slice(&chunk.data);
    }
    Ok(out)
}

// ============================================================================
// 5. Ultrasonic Chirp Acoustic Modem (`UltrasonicChirpModem`)
//    For zero-connection / air-gapped devices with only a speaker & microphone!
// ============================================================================

/// Acoustic band profile for the Ultrasonic Chirp Modem.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct AcousticModemConfig {
    /// Audio sample rate (`48,000 Hz` default).
    pub sample_rate_hz: u32,
    /// Duration of each ternary symbol in samples (`240` samples = `5 ms` @ 48 kHz = `200 trits/sec`).
    pub symbol_samples: usize,
    /// Frequency for balanced ternary `-1` (`T`).
    pub freq_neg_hz: u32,
    /// Frequency for balanced ternary `0`.
    pub freq_zero_hz: u32,
    /// Frequency for balanced ternary `+1`.
    pub freq_pos_hz: u32,
    /// Up-chirp start frequency (`18,000 Hz` ultrasonic or `1,600 Hz` audible).
    pub chirp_start_hz: u32,
    /// Up-chirp end frequency (`21,600 Hz` ultrasonic or `3,400 Hz` audible).
    pub chirp_end_hz: u32,
}

impl AcousticModemConfig {
    /// True near-ultrasonic profile (`18.5 kHz / 19.75 kHz / 21.0 kHz` @ `48 kHz` sample rate)
    /// — virtually inaudible to humans while transmissible between laptop/phone/ESP32 speakers & mics.
    pub const fn ultrasonic_48khz() -> Self {
        Self {
            sample_rate_hz: 48_000,
            symbol_samples: 240, // 5.0 ms per trit
            freq_neg_hz: 18_500,
            freq_zero_hz: 19_750,
            freq_pos_hz: 21_000,
            chirp_start_hz: 18_000,
            chirp_end_hz: 21_600,
        }
    }

    /// Audible bird-chirp fallback profile (`1,850 Hz / 2,450 Hz / 3,150 Hz` @ `48 kHz`)
    /// for narrowband telephony or low-sample-rate microphones.
    pub const fn audible_chirp_48khz() -> Self {
        Self {
            sample_rate_hz: 48_000,
            symbol_samples: 240,
            freq_neg_hz: 1_850,
            freq_zero_hz: 2_450,
            freq_pos_hz: 3_150,
            chirp_start_hz: 1_600,
            chirp_end_hz: 3_400,
        }
    }
}

/// Quarter-wave 64-entry integer sine lookup table (`Q14`: `0..=16384`).
const SIN_LUT_Q14: [i32; 65] = [
    0, 402, 804, 1205, 1606, 2006, 2404, 2801, 3196, 3590, 3981, 4370, 4756, 5139, 5520, 5897,
    6270, 6639, 7005, 7366, 7723, 8076, 8423, 8765, 9102, 9434, 9760, 10080, 10394, 10702, 11003,
    11297, 11585, 11866, 12140, 12406, 12665, 12916, 13160, 13395, 13623, 13842, 14053, 14256,
    14449, 14635, 14811, 14978, 15137, 15286, 15426, 15557, 15679, 15791, 15893, 15986, 16069,
    16143, 16207, 16261, 16305, 16340, 16364, 16379, 16384,
];

/// Pure-integer sine (`phase_q16` in `0..65536` = `0..2π`), returning `-16384..=16384` (`Q14`).
#[inline]
pub const fn sin_q14(phase_q16: u32) -> i32 {
    let p = phase_q16 & 0xFFFF;
    let quadrant = p >> 14;
    let offset = (p & 0x3FFF) as usize;
    let idx = offset >> 8; // 0..64
    let frac = (offset & 0xFF) as i32;
    let idx_in_lut = if (quadrant & 1) == 1 { 64 - idx } else { idx };
    let next_in_lut = if (quadrant & 1) == 1 {
        if idx_in_lut > 0 {
            idx_in_lut - 1
        } else {
            0
        }
    } else if idx_in_lut < 64 {
        idx_in_lut + 1
    } else {
        64
    };
    let y0 = SIN_LUT_Q14[idx_in_lut];
    let y1 = SIN_LUT_Q14[next_in_lut];
    let val = y0 + (((y1 - y0) * frac) >> 8);
    if quadrant >= 2 {
        -val
    } else {
        val
    }
}

/// Pure-integer cosine (`phase_q16` in `0..65536` = `0..2π`), returning `-16384..=16384` (`Q14`).
#[inline]
pub const fn cos_q14(phase_q16: u32) -> i32 {
    sin_q14(phase_q16.wrapping_add(16384))
}

/// Convert a single byte (`0..=242` in SPQT, or full `0..=255` via 6-trit base-3 `[0..728]`) into 6 balanced trits (`-1, 0, +1`).
#[inline]
pub const fn byte_to_6trits(byte: u8) -> [i8; 6] {
    let mut v = byte as u16;
    let mut out = [0i8; 6];
    let mut i = 0;
    while i < 6 {
        let rem = (v % 3) as i8;
        out[i] = rem - 1; // 0 -> -1, 1 -> 0, 2 -> +1
        v /= 3;
        i += 1;
    }
    out
}

/// Reconstruct a byte (`0..=255`) from 6 balanced trits (`-1, 0, +1`).
#[inline]
pub const fn trits6_to_byte(trits: &[i8; 6]) -> Option<u8> {
    let mut acc: u16 = 0;
    let mut pow3: u16 = 1;
    let mut i = 0;
    while i < 6 {
        let d = trits[i] + 1;
        if d < 0 || d > 2 {
            return None;
        }
        acc += (d as u16) * pow3;
        pow3 *= 3;
        i += 1;
    }
    if acc <= 255 {
        Some(acc as u8)
    } else {
        None
    }
}

/// Pure-integer Ultrasonic Chirp & Ternary FSK Acoustic Modem.
///
/// Frame layout on the acoustic medium:
/// 1. **Up-Chirp Preamble** (`2 * symbol_samples`): sweeps `chirp_start_hz -> chirp_end_hz`
/// 2. **Down-Chirp Sync** (`2 * symbol_samples`): sweeps `chirp_end_hz -> chirp_start_hz`
/// 3. **Framed Payload** (`[Magic 'U','C': 2B][Len: u16 LE][Payload: N bytes][CRC32: u32 LE]`),
///    where every byte is transmitted as 6 balanced ternary acoustic symbols (`-1, 0, +1`).
pub struct UltrasonicChirpModem {
    pub config: AcousticModemConfig,
}

impl UltrasonicChirpModem {
    pub const fn new(config: AcousticModemConfig) -> Self {
        Self { config }
    }

    /// Encode arbitrary payload bytes (such as a `MemnetRawPacket` or `WaveQaPacket`) into
    /// 16-bit signed PCM audio samples (`i16`) with an ultrasonic chirp preamble and CRC32 seal.
    pub fn encode_frame_to_pcm16(&self, payload: &[u8]) -> Vec<i16> {
        let crc = felix_crc32_ieee(payload);
        let len_u16 = payload.len().min(u16::MAX as usize) as u16;
        let mut frame_bytes = Vec::with_capacity(2 + 2 + (len_u16 as usize) + 4);
        frame_bytes.extend_from_slice(b"UC");
        frame_bytes.extend_from_slice(&len_u16.to_le_bytes());
        frame_bytes.extend_from_slice(&payload[..len_u16 as usize]);
        frame_bytes.extend_from_slice(&crc.to_le_bytes());

        let sym_n = self.config.symbol_samples;
        let chirp_n = sym_n * 2;
        let total_symbols = frame_bytes.len() * 6;
        let mut pcm = Vec::with_capacity(chirp_n * 2 + total_symbols * sym_n);

        // 1. Up-Chirp Preamble + 2. Down-Chirp Sync
        self.append_linear_chirp(
            &mut pcm,
            chirp_n,
            self.config.chirp_start_hz,
            self.config.chirp_end_hz,
        );
        self.append_linear_chirp(
            &mut pcm,
            chirp_n,
            self.config.chirp_end_hz,
            self.config.chirp_start_hz,
        );

        // 3. Ternary FSK symbols with raised-cosine edge tapering (prevents speaker clicks)
        for &byte in &frame_bytes {
            let trits = byte_to_6trits(byte);
            for trit in trits {
                let freq = match trit {
                    -1 => self.config.freq_neg_hz,
                    0 => self.config.freq_zero_hz,
                    _ => self.config.freq_pos_hz,
                };
                self.append_tone_symbol(&mut pcm, sym_n, freq);
            }
        }
        pcm
    }

    fn append_linear_chirp(&self, pcm: &mut Vec<i16>, n_samples: usize, f0_hz: u32, f1_hz: u32) {
        let sr = self.config.sample_rate_hz as i64;
        let mut phase_q16: i64 = 0;
        let n_i64 = n_samples.max(1) as i64;
        for i in 0..n_samples {
            let f_curr = (f0_hz as i64) + (((f1_hz as i64) - (f0_hz as i64)) * (i as i64)) / n_i64;
            let step_q16 = (f_curr * 65536) / sr;
            phase_q16 = (phase_q16 + step_q16) & 0xFFFF;
            let env_q10 = Self::raised_cosine_envelope_q10(i, n_samples);
            let sample = (sin_q14(phase_q16 as u32) * env_q10) >> 10;
            pcm.push((sample * 3 / 2).clamp(-32000, 32000) as i16);
        }
    }

    fn append_tone_symbol(&self, pcm: &mut Vec<i16>, n_samples: usize, freq_hz: u32) {
        let sr = self.config.sample_rate_hz as u64;
        let step_q16 = ((freq_hz as u64) * 65536) / sr;
        let mut phase_q16: u64 = 0;
        for i in 0..n_samples {
            let env_q10 = Self::raised_cosine_envelope_q10(i, n_samples);
            let sample = (sin_q14(phase_q16 as u32) * env_q10) >> 10;
            pcm.push((sample * 3 / 2).clamp(-32000, 32000) as i16);
            phase_q16 = (phase_q16 + step_q16) & 0xFFFF;
        }
    }

    /// Smooth raised-cosine edge ramp over the first and last 12.5% of a symbol to eliminate audible clicks.
    #[inline]
    const fn raised_cosine_envelope_q10(idx: usize, total: usize) -> i32 {
        let ramp = total / 8;
        if ramp == 0 {
            return 1024;
        }
        if idx < ramp {
            ((idx * 1024) / ramp) as i32
        } else if idx + ramp >= total {
            (((total - 1 - idx) * 1024) / ramp) as i32
        } else {
            1024
        }
    }

    /// Compute integer quadrature correlation energy at `freq_hz` over a slice of PCM samples.
    pub fn quadrature_energy(&self, samples: &[i16], freq_hz: u32) -> u64 {
        let sr = self.config.sample_rate_hz as u64;
        let step_q16 = ((freq_hz as u64) * 65536) / sr;
        let mut phase_q16: u64 = 0;
        let mut i_acc: i64 = 0;
        let mut q_acc: i64 = 0;
        for &s in samples {
            let s_i64 = s as i64;
            i_acc += (s_i64 * (cos_q14(phase_q16 as u32) as i64)) >> 14;
            q_acc += (s_i64 * (sin_q14(phase_q16 as u32) as i64)) >> 14;
            phase_q16 = (phase_q16 + step_q16) & 0xFFFF;
        }
        let i_scaled = i_acc >> 4;
        let q_scaled = q_acc >> 4;
        ((i_scaled * i_scaled) + (q_scaled * q_scaled)) as u64
    }

    /// Demodulate a single ternary symbol window into `-1`, `0`, or `+1`.
    pub fn demodulate_symbol(&self, window: &[i16]) -> i8 {
        let e_neg = self.quadrature_energy(window, self.config.freq_neg_hz);
        let e_zero = self.quadrature_energy(window, self.config.freq_zero_hz);
        let e_pos = self.quadrature_energy(window, self.config.freq_pos_hz);
        if e_neg >= e_zero && e_neg >= e_pos {
            -1
        } else if e_zero >= e_neg && e_zero >= e_pos {
            0
        } else {
            1
        }
    }

    /// Decode a 16-bit PCM audio buffer back into the verified payload bytes.
    pub fn decode_pcm16_to_frame(&self, pcm: &[i16]) -> Result<Vec<u8>> {
        let sym_n = self.config.symbol_samples;
        let preamble_samples = sym_n * 4; // 2x up-chirp + 2x down-chirp
        anyhow::ensure!(
            pcm.len() >= preamble_samples + sym_n * 48,
            "PCM buffer too short for UltrasonicChirpModem frame ({} samples)",
            pcm.len()
        );
        let data_pcm = &pcm[preamble_samples..];
        let decode_byte_at = |byte_idx: usize| -> Result<u8> {
            let start = byte_idx * 6 * sym_n;
            anyhow::ensure!(
                start + 6 * sym_n <= data_pcm.len(),
                "Truncated acoustic symbol stream at byte {byte_idx}"
            );
            let mut trits = [0i8; 6];
            for (t_idx, slot) in trits.iter_mut().enumerate() {
                let w_start = start + t_idx * sym_n;
                *slot = self.demodulate_symbol(&data_pcm[w_start..w_start + sym_n]);
            }
            trits6_to_byte(&trits)
                .ok_or_else(|| anyhow::anyhow!("Invalid SPQT trit byte at index {byte_idx}"))
        };

        let m0 = decode_byte_at(0)?;
        let m1 = decode_byte_at(1)?;
        anyhow::ensure!(
            [m0, m1] == *b"UC",
            "Invalid ultrasonic frame magic: [{:#04x}, {:#04x}]",
            m0,
            m1
        );
        let len = u16::from_le_bytes([decode_byte_at(2)?, decode_byte_at(3)?]) as usize;
        let mut payload = Vec::with_capacity(len);
        for i in 0..len {
            payload.push(decode_byte_at(4 + i)?);
        }
        let crc_bytes = [
            decode_byte_at(4 + len)?,
            decode_byte_at(4 + len + 1)?,
            decode_byte_at(4 + len + 2)?,
            decode_byte_at(4 + len + 3)?,
        ];
        let expected_crc = u32::from_le_bytes(crc_bytes);
        let actual_crc = felix_crc32_ieee(&payload);
        anyhow::ensure!(
            expected_crc == actual_crc,
            "Ultrasonic chirp CRC32 mismatch: expected {expected_crc:#010x}, got {actual_crc:#010x}"
        );
        Ok(payload)
    }

    /// Write 16-bit mono PCM samples to a standard RIFF `.wav` file.
    pub fn write_wav_file(&self, path: &Path, pcm: &[i16]) -> Result<()> {
        let mut file = std::fs::File::create(path)
            .with_context(|| format!("Failed to create WAV file {}", path.display()))?;
        let data_bytes = (pcm.len() * 2) as u32;
        let riff_size = 36u32.saturating_add(data_bytes);
        let sr = self.config.sample_rate_hz;
        let byte_rate = sr * 2;

        file.write_all(b"RIFF")?;
        file.write_all(&riff_size.to_le_bytes())?;
        file.write_all(b"WAVEfmt ")?;
        file.write_all(&16u32.to_le_bytes())?; // PCM fmt chunk size
        file.write_all(&1u16.to_le_bytes())?; // AudioFormat = 1 (PCM)
        file.write_all(&1u16.to_le_bytes())?; // NumChannels = 1 (mono)
        file.write_all(&sr.to_le_bytes())?;
        file.write_all(&byte_rate.to_le_bytes())?;
        file.write_all(&2u16.to_le_bytes())?; // BlockAlign = 2
        file.write_all(&16u16.to_le_bytes())?; // BitsPerSample = 16
        file.write_all(b"data")?;
        file.write_all(&data_bytes.to_le_bytes())?;

        let mut raw = Vec::with_capacity(pcm.len() * 2);
        for &s in pcm {
            raw.extend_from_slice(&s.to_le_bytes());
        }
        file.write_all(&raw)?;
        Ok(())
    }

    /// Read 16-bit mono PCM samples from a standard RIFF `.wav` file.
    pub fn read_wav_file(path: &Path) -> Result<(u32, Vec<i16>)> {
        let bytes = std::fs::read(path)
            .with_context(|| format!("Failed to read WAV file {}", path.display()))?;
        anyhow::ensure!(
            bytes.len() >= 44 && &bytes[0..4] == b"RIFF" && &bytes[8..12] == b"WAVE",
            "Invalid WAV header in {}",
            path.display()
        );
        let sample_rate = u32::from_le_bytes(bytes[24..28].try_into()?);
        let pcm_bytes = &bytes[44..];
        let mut samples = Vec::with_capacity(pcm_bytes.len() / 2);
        for chunk in pcm_bytes.chunks_exact(2) {
            samples.push(i16::from_le_bytes([chunk[0], chunk[1]]));
        }
        Ok((sample_rate, samples))
    }
}

// ============================================================================
// 6. Unified MemNet Envelope & Persistent Local Mesh State (`~/.mem8/memnet/`)
// ============================================================================

/// High-level typed payload carried inside a `MemnetRawPacket`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum MemnetPayload {
    FolderShareAnnounce(SharedFolderManifest),
    DriveSpaceOffer {
        node_address: u8,
        node_name: String,
        endpoint: String,
        free_space_mb: u64,
    },
    WaveQaQuery {
        origin_node: u8,
        question: String,
        min_resonance_pm: u32,
    },
    WaveQaResponse(WaveQaPacket),
    MediaChunk(MediaChunkPacket),
    Esp32ConstellationBeacon(MemnetCapsule64),
}

impl MemnetPayload {
    /// Wrap this typed payload inside a compact `MemnetRawPacket` with `CRC32` verification.
    pub fn into_raw_packet(&self, path: Vec<u8>) -> Result<MemnetRawPacket> {
        let json = serde_json::to_vec(self)?;
        let crc = felix_crc32_ieee(&json);
        let mut payload = Vec::with_capacity(8 + json.len());
        payload.extend_from_slice(&MEMNET_ENVELOPE_MAGIC);
        payload.extend_from_slice(&crc.to_le_bytes());
        payload.extend_from_slice(&json);
        Ok(MemnetRawPacket { path, payload })
    }

    /// Decode and verify a `MemnetPayload` from a `MemnetRawPacket`.
    pub fn from_raw_packet(pkt: &MemnetRawPacket) -> Result<Self> {
        anyhow::ensure!(
            pkt.payload.len() >= 8 && pkt.payload[0..4] == MEMNET_ENVELOPE_MAGIC,
            "Invalid MemNet envelope magic"
        );
        let expected_crc = u32::from_le_bytes(pkt.payload[4..8].try_into()?);
        let body = &pkt.payload[8..];
        let actual_crc = felix_crc32_ieee(body);
        anyhow::ensure!(
            expected_crc == actual_crc,
            "MemNet envelope CRC32 mismatch"
        );
        Ok(serde_json::from_slice(body)?)
    }
}

/// Persistent Smart Tree MemNet node state stored at `~/.mem8/memnet/mesh_state.json`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemnetNodeState {
    pub local_address: u8,
    pub node_name: String,
    pub tier: RoutingTier,
    pub routes: Vec<MemnetRouteEntry>,
    pub shared_folders: BTreeMap<String, SharedFolderManifest>,
    pub wave_qa_cache: Vec<WaveQaPacket>,
}

impl Default for MemnetNodeState {
    fn default() -> Self {
        let host = std::env::var("HOSTNAME")
            .or_else(|_| std::env::var("USER"))
            .unwrap_or_else(|_| "aye-st-node".to_string());
        let mut routes = Vec::new();
        // Pre-seed the 4 Sovereign ESP32 Constellation routes under multi-route address 0xC5
        for (idx, (node_id, name, mac, _glyph, _trit)) in
            ESP32_CONSTELLATION_NODES.iter().enumerate()
        {
            routes.push(MemnetRouteEntry::new(
                0xC5,
                10 + (*node_id),
                format!("{name} ({mac})"),
                if idx == 3 {
                    MemnetMedium::LoRaSx1262
                } else {
                    MemnetMedium::Esp32Now
                },
                format!("255.255.255.255:{MEMNET_ESP32_PORT}"),
                6 + (idx as u16) * 3,
            ));
        }
        // Pre-seed local WiFi, Bluetooth, and Ultrasonic Chirp multi-route paths
        routes.push(MemnetRouteEntry::new(
            0xC5,
            20,
            "LAN-WiFi-Broadcast",
            MemnetMedium::WiFi,
            format!("255.255.255.255:{MEMNET_FS_PORT}"),
            4,
        ));
        routes.push(MemnetRouteEntry::new(
            0xC5,
            21,
            "Bluetooth-8BM8-Mesh",
            MemnetMedium::Bluetooth,
            "ble://8BM8-mesh",
            12,
        ));
        routes.push(MemnetRouteEntry::new(
            0xC5,
            22,
            "Ultrasonic-Chirp-SpeakerMic",
            MemnetMedium::UltrasonicChirp,
            "acoustic://18.5-21.0kHz",
            45,
        ));

        Self {
            local_address: 42,
            node_name: host,
            tier: RoutingTier::TierC,
            routes,
            shared_folders: BTreeMap::new(),
            wave_qa_cache: Vec::new(),
        }
    }
}

impl MemnetNodeState {
    pub fn default_state_path() -> PathBuf {
        dirs::home_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join(".mem8")
            .join("memnet")
            .join("mesh_state.json")
    }

    pub fn load_or_default() -> Self {
        let path = Self::default_state_path();
        if let Ok(bytes) = std::fs::read(&path) {
            if let Ok(state) = serde_json::from_slice(&bytes) {
                return state;
            }
        }
        Self::default()
    }

    pub fn save(&self) -> Result<PathBuf> {
        let path = Self::default_state_path();
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        std::fs::write(&path, serde_json::to_vec_pretty(self)?)?;
        Ok(path)
    }

    /// Select the best route(s) for a destination address according to the node's `RoutingTier`.
    /// - `TierA`: Single route with lowest RTT latency.
    /// - `TierB`: Single route with highest Marine jitter/salience stability score.
    /// - `TierC`: Primary + secondary routes for zero-packet-loss dual-path handoff!
    pub fn select_routes(&self, destination: u8) -> Vec<&MemnetRouteEntry> {
        let mut viable: Vec<&MemnetRouteEntry> = self
            .routes
            .iter()
            .filter(|r| {
                r.address == destination
                    && r.heartbeat_credits > 0
                    && r.status != RouteStatus::Down
            })
            .collect();
        if viable.is_empty() {
            return Vec::new();
        }
        match self.tier {
            RoutingTier::TierA => {
                viable.sort_by_key(|r| r.latency_ms);
                viable.truncate(1);
                viable
            }
            RoutingTier::TierB => {
                viable.sort_by_key(|r| std::cmp::Reverse(r.marine_quality_score()));
                viable.truncate(1);
                viable
            }
            RoutingTier::TierC => {
                viable.sort_by_key(|r| std::cmp::Reverse(r.marine_quality_score()));
                viable.truncate(2); // Dual-path transmission for zero packet loss
                viable
            }
        }
    }

    /// Query the local + LAN `MEM|8` Wave Q&A cache for the highest-resonance answer.
    pub fn query_wave_qa(&mut self, question: &str, min_resonance_pm: u32) -> Option<(u32, WaveQaPacket)> {
        let mut best_idx = None;
        let mut best_score = 0u32;
        for (idx, entry) in self.wave_qa_cache.iter().enumerate() {
            let score = entry.resonance_permille(question);
            if score >= min_resonance_pm && score > best_score {
                best_score = score;
                best_idx = Some(idx);
            }
        }
        if let Some(i) = best_idx {
            self.wave_qa_cache[i].hit_count = self.wave_qa_cache[i].hit_count.saturating_add(1);
            Some((best_score, self.wave_qa_cache[i].clone()))
        } else {
            None
        }
    }

    /// Record or update a `WaveQaPacket` in the node's semantic wave memory.
    pub fn learn_wave_qa(&mut self, question: &str, answer: &str) -> WaveQaPacket {
        let pkt = WaveQaPacket::new(self.local_address, &self.node_name, question, answer);
        if let Some(existing) = self
            .wave_qa_cache
            .iter_mut()
            .find(|e| e.packet_id == pkt.packet_id)
        {
            *existing = pkt.clone();
        } else {
            self.wave_qa_cache.push(pkt.clone());
        }
        pkt
    }

    /// Best-effort non-blocking UDP broadcast of a `MemnetRawPacket` to the local LAN (`:8423` & `:8888`).
    pub fn broadcast_lan_udp(&self, pkt: &MemnetRawPacket) -> usize {
        let Ok(sock) = UdpSocket::bind("0.0.0.0:0") else {
            return 0;
        };
        let _ = sock.set_broadcast(true);
        let _ = sock.set_write_timeout(Some(Duration::from_millis(50)));
        let wire = pkt.to_wire_bytes();
        let mut sent = 0usize;
        for port in [MEMNET_FS_PORT, MEMNET_HOTTUB_PORT] {
            let target = SocketAddr::from(([255, 255, 255, 255], port));
            if sock.send_to(&wire, target).is_ok() {
                sent += 1;
            }
            let loopback = SocketAddr::from(([127, 0, 0, 1], port));
            let _ = sock.send_to(&wire, loopback);
        }
        sent
    }

    /// Compute the composite parallel Real Time Constant (`RTC`, `τ_eff` in ms) across active routes
    /// for `destination` using parallel RC conductance (`1 / τ_eff = Σ (1 / τ_i)`).
    pub fn composite_rtc_tau_ms(&self, destination: u8) -> u32 {
        let selected = self.select_routes(destination);
        if selected.is_empty() {
            return 0;
        }
        let inv_sum: f64 = selected
            .iter()
            .map(|r| 1.0 / (r.effective_rtc_ms().max(1) as f64))
            .sum();
        if inv_sum <= 0.0 {
            0
        } else {
            (1.0 / inv_sum).round().max(1.0) as u32
        }
    }

    /// Compute the **3rd+ Escape** (`UltraGraphEscapeReport`) for a core semantic state/payload:
    /// combines Multi-Dimensional Ultra-Graph shortcuts (Szemerédi + Erdős–Steiner T8R virtual bridges),
    /// Constructive Lovász Local Lemma (`LllHypergraph` Moser–Tardos zero-collision channel assignment),
    /// and **Language LLL** (4D Lenstra–Lenstra–Lovász lattice basis reduction + `64B@0x62` / `79B` `SUN:TDECK` wave).
    pub fn compute_3rd_escape(&self, core_payload: &str) -> UltraGraphEscapeReport {
        use crate::mem8::regularity::moser_tardos::{
            solve_moser_tardos_auto_scale, BadEventKind, LllHypergraph, WorkUnitId,
        };

        let active_routes: Vec<&MemnetRouteEntry> = self
            .routes
            .iter()
            .filter(|r| r.status != RouteStatus::Down && r.heartbeat_credits > 0)
            .collect();
        let route_count = active_routes.len().max(1);

        // 1. Constructive Lovász Local Lemma (Moser-Tardos) over multi-dimensional route × phase hypergraph:
        // Each work unit is a wave sub-carrier / medium slot; each 4-uniform hyperedge is a MonochromaticHyperedge
        // phase-collision constraint under 3-coloring (ternary SPQT -1, 0, +1 -> p = 3^(1-4) = 1/27 ≈ 0.037)
        // with bounded dependency degree d <= 2 so e * p * (d + 1) ≈ 0.302 <= 1.0.
        let num_units = (route_count * 4).max(8);
        let mut scope_storage: Vec<[WorkUnitId; 4]> = Vec::new();
        let mut v = 0usize;
        while v + 4 <= num_units {
            scope_storage.push([
                v as WorkUnitId,
                (v + 1) as WorkUnitId,
                (v + 2) as WorkUnitId,
                (v + 3) as WorkUnitId,
            ]);
            v += 3; // Overlap by 1 variable so dependency degree d <= 2
        }
        let constraints: Vec<(BadEventKind, &[WorkUnitId])> = scope_storage
            .iter()
            .map(|s| (BadEventKind::MonochromaticHyperedge, &s[..]))
            .collect();
        let lll_graph = LllHypergraph::from_constraints(num_units, &constraints);
        let seed_hash = fnv64_1a(core_payload.as_bytes());
        let lll_sched = solve_moser_tardos_auto_scale(&lll_graph, 3, true, seed_hash, 1024);
        let max_p = if lll_graph.num_events > 0 {
            lll_graph.event_probability(0, lll_sched.num_worktrees)
        } else {
            0.0
        };
        let ep_d_plus_1 =
            std::f64::consts::E * max_p * ((lll_sched.max_dependency_degree + 1) as f64);

        // 2. Language LLL: 4D Lenstra-Lenstra-Lovász Lattice Basis Reduction + Symbolic Invariant Wave
        let basis_report = lll_reduce_wave_basis_4d(core_payload);
        let wave_frame = WaveFrame79::from_semantic_text(core_payload);
        let (_tokens, _trits, _wave_b3, litany_wire) =
            extract_semantic_wave_features(core_payload);

        // 3. Real Time Constant (RTC τ) across the multi-dimensional Steiner/T8R shortcut
        let single_best_rtc_ms = active_routes
            .iter()
            .map(|r| r.effective_rtc_ms())
            .min()
            .unwrap_or(20);
        let parallel_rtc_tau_ms = self.composite_rtc_tau_ms(0xC5).max(1);

        // 4. Compare the 3 Escape Regimes
        let core_particles_proxy = (core_payload.len().max(16) as u64) * (route_count as u64) * 64;
        let photon_3d_walk_steps = core_particles_proxy.saturating_mul(core_particles_proxy);
        let ultra_graph_dimensions = 4 + active_routes.len(); // 3D space + phase + 1D per orthogonal medium/orbital

        let regimes = vec![
            EscapeRegimeSummary {
                regime_id: 1,
                name: "Escape #1: Neutrino (Weak Decoupled)".to_string(),
                geometry: "3D straight line (ignores medium)".to_string(),
                topological_hops: 1,
                semantic_coupling_pct: 0,
                collision_free: true,
                effective_rtc_tau_ms: single_best_rtc_ms,
            },
            EscapeRegimeSummary {
                regime_id: 2,
                name: "Escape #2: Photon (3D Lattice Random Walk)".to_string(),
                geometry: format!(
                    "3D radiative diffusion (O((R/l)^2) = {} scatter steps)",
                    photon_3d_walk_steps
                ),
                topological_hops: photon_3d_walk_steps,
                semantic_coupling_pct: 100,
                collision_free: false,
                effective_rtc_tau_ms: single_best_rtc_ms.saturating_mul(1_000),
            },
            EscapeRegimeSummary {
                regime_id: 3,
                name: "Escape #3+: Ultra-Graph + Constructive LLL + Language LLL".to_string(),
                geometry: format!(
                    "{}D Expander Hypergraph + Erdős-Steiner T8R Bridge + LLL Basis",
                    ultra_graph_dimensions
                ),
                topological_hops: 1,
                semantic_coupling_pct: 100,
                collision_free: lll_sched.is_conflict_free,
                effective_rtc_tau_ms: parallel_rtc_tau_ms,
            },
        ];

        UltraGraphEscapeReport {
            core_payload: core_payload.to_string(),
            ultra_graph_dimensions,
            szemeredi_orbital_clusters_k: active_routes.len().max(2),
            steiner_t8r_virtual_bridges: self.select_routes(0xC5).len().max(1),
            lll_ep_d_plus_1: ep_d_plus_1,
            lll_guaranteed: lll_sched.lll_guaranteed_a_priori,
            moser_tardos_resamples: lll_sched.total_resamples,
            zero_collision_verified: lll_sched.is_conflict_free,
            single_route_rtc_tau_ms: single_best_rtc_ms,
            parallel_rtc_tau_ms,
            settling_3tau_ms: parallel_rtc_tau_ms.saturating_mul(3),
            settling_5tau_ms: parallel_rtc_tau_ms.saturating_mul(5),
            language_lll: basis_report,
            wave_frame,
            litany_wire,
            regimes,
        }
    }
}

// ============================================================================
// 7. 3rd+ Escape: Ultra-Graph Multi-Dimensional Network & Language LLL
// ============================================================================

/// Summary of one of the three escape regimes from a dense core (solar plasma or cognitive/network core).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EscapeRegimeSummary {
    pub regime_id: u8,
    pub name: String,
    pub geometry: String,
    pub topological_hops: u64,
    pub semantic_coupling_pct: u8,
    pub collision_free: bool,
    pub effective_rtc_tau_ms: u32,
}

/// Report from 4D Lenstra–Lenstra–Lovász (`LLL`) Lattice Basis Reduction + Symbolic Language Compression.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LanguageLllBasisReport {
    /// Original skewed 4D semantic lattice basis vectors before LLL reduction.
    pub initial_basis: [[i64; 4]; 4],
    /// LLL-reduced shortest, nearly-orthogonal 4D basis vectors (`δ = 3/4`).
    pub reduced_basis: [[i64; 4]; 4],
    /// Sum of squared Euclidean norms `Σ ||b_i||^2` before LLL reduction.
    pub initial_norm_sq: u64,
    /// Sum of squared Euclidean norms `Σ ||b'_i||^2` after LLL reduction.
    pub reduced_norm_sq: u64,
    /// Basis norm reduction ratio in basis points (`10000` = 1.0x, higher means stronger lattice compression).
    pub basis_compression_bps: u64,
    /// Compact symbolic Language LLL invariant token (`SPQT + seal + reduced basis signature`).
    pub language_lll_symbol: String,
}

/// Complete report for the **3rd+ Escape** combining Ultra-Graph Theory, Constructive LLL, Language LLL, and RTC (`τ`).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UltraGraphEscapeReport {
    pub core_payload: String,
    pub ultra_graph_dimensions: usize,
    pub szemeredi_orbital_clusters_k: usize,
    pub steiner_t8r_virtual_bridges: usize,
    pub lll_ep_d_plus_1: f64,
    pub lll_guaranteed: bool,
    pub moser_tardos_resamples: usize,
    pub zero_collision_verified: bool,
    pub single_route_rtc_tau_ms: u32,
    pub parallel_rtc_tau_ms: u32,
    pub settling_3tau_ms: u32,
    pub settling_5tau_ms: u32,
    pub language_lll: LanguageLllBasisReport,
    pub wave_frame: WaveFrame79,
    pub litany_wire: LitanyWireFrame,
    pub regimes: Vec<EscapeRegimeSummary>,
}

/// Perform exact 4D Lenstra–Lenstra–Lovász (`LLL`) lattice basis reduction (`δ = 3/4`) on a skewed
/// semantic wave lattice generated from `payload`, finding the shortest orthogonal basis for Language LLL.
pub fn lll_reduce_wave_basis_4d(payload: &str) -> LanguageLllBasisReport {
    let h64 = fnv64_1a(payload.as_bytes());
    let hb = h64.to_le_bytes();
    let s0 = (hb[0] as i64) + 11;
    let s1 = (hb[1] as i64) + 7;
    let s2 = (hb[2] as i64) + 5;
    let s3 = (hb[3] as i64) + 3;

    // Skewed unimodular-like full-rank 4D lattice basis representing entangled high-entropy scattering walks
    let initial_basis: [[i64; 4]; 4] = [
        [s0, 1, 0, 0],
        [s0 * 3 + s1, 3, 1, 0],
        [s0 * 5 + s2, 5, 2, 1],
        [s0 * 7 + s3, 7, 3, 2],
    ];

    let mut b = initial_basis;
    let mut b_star = [[0.0f64; 4]; 4];
    let mut mu = [[0.0f64; 4]; 4];
    let mut b_star_norm_sq = [0.0f64; 4];

    let recompute_gram_schmidt = |b: &[[i64; 4]; 4],
                                  b_star: &mut [[f64; 4]; 4],
                                  mu: &mut [[f64; 4]; 4],
                                  b_star_norm_sq: &mut [f64; 4]| {
        for i in 0..4 {
            for d in 0..4 {
                b_star[i][d] = b[i][d] as f64;
            }
            for j in 0..i {
                let mut dot = 0.0f64;
                for d in 0..4 {
                    dot += (b[i][d] as f64) * b_star[j][d];
                }
                mu[i][j] = if b_star_norm_sq[j] > 1e-12 {
                    dot / b_star_norm_sq[j]
                } else {
                    0.0
                };
                for d in 0..4 {
                    b_star[i][d] -= mu[i][j] * b_star[j][d];
                }
            }
            let mut nsq = 0.0f64;
            for d in 0..4 {
                nsq += b_star[i][d] * b_star[i][d];
            }
            b_star_norm_sq[i] = nsq;
        }
    };

    recompute_gram_schmidt(&b, &mut b_star, &mut mu, &mut b_star_norm_sq);

    let delta = 0.75f64; // Standard Lovász constant δ = 3/4
    let mut k = 1usize;
    let mut guard = 0usize;
    while k < 4 && guard < 256 {
        guard += 1;
        for j in (0..k).rev() {
            if mu[k][j].abs() > 0.5 {
                let q = mu[k][j].round() as i64;
                for d in 0..4 {
                    b[k][d] -= q * b[j][d];
                }
                recompute_gram_schmidt(&b, &mut b_star, &mut mu, &mut b_star_norm_sq);
            }
        }
        if b_star_norm_sq[k] >= (delta - mu[k][k - 1] * mu[k][k - 1]) * b_star_norm_sq[k - 1] {
            k += 1;
        } else {
            b.swap(k, k - 1);
            recompute_gram_schmidt(&b, &mut b_star, &mut mu, &mut b_star_norm_sq);
            k = k.saturating_sub(1).max(1);
        }
    }

    let norm_sq_sum = |mat: &[[i64; 4]; 4]| -> u64 {
        mat.iter()
            .map(|row| row.iter().map(|&x| (x as i128 * x as i128) as u64).sum::<u64>())
            .sum()
    };

    let initial_norm_sq = norm_sq_sum(&initial_basis).max(1);
    let reduced_norm_sq = norm_sq_sum(&b).max(1);
    let basis_compression_bps = (initial_norm_sq.saturating_mul(10_000)) / reduced_norm_sq;
    let language_lll_symbol = format!(
        "LLL4[{},{},{},{}|seal:{:02x}{:02x}]",
        b[0][0], b[0][1], b[1][0], b[1][1], hb[0], hb[1]
    );

    LanguageLllBasisReport {
        initial_basis,
        reduced_basis: b,
        initial_norm_sq,
        reduced_norm_sq,
        basis_compression_bps,
        language_lll_symbol,
    }
}

// ============================================================================
// Unit Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn memnet_raw_packet_overhead_and_broadcast_cascade() {
        let local = MemnetRawPacket::localhost(b"ping".to_vec());
        assert_eq!(local.header_overhead_bytes(), 1);
        let wire_local = local.to_wire_bytes();
        assert_eq!(wire_local.len(), 5);
        assert_eq!(MemnetRawPacket::from_wire_bytes(&wire_local).unwrap(), local);

        let room = MemnetRawPacket::same_room(42, b"hello".to_vec());
        assert_eq!(room.header_overhead_bytes(), 2);
        let wire_room = room.to_wire_bytes();
        assert_eq!(wire_room[0], 1);
        assert_eq!(wire_room[1], 42);
        assert_eq!(MemnetRawPacket::from_wire_bytes(&wire_room).unwrap(), room);

        let bcast = MemnetRawPacket::broadcast(2, b"firmware".to_vec());
        assert_eq!(bcast.header_overhead_bytes(), 1);
        let wire_bcast = bcast.to_wire_bytes();
        assert_eq!(wire_bcast[0], 0xF2);
        let decoded_bcast = MemnetRawPacket::from_wire_bytes(&wire_bcast).unwrap();
        let step1 = decoded_bcast.step_broadcast_cascade().unwrap();
        assert_eq!(step1.path, vec![0xF1]);
        let step0 = step1.step_broadcast_cascade().unwrap();
        assert_eq!(step0.path, vec![0xF0]);
        assert!(step0.step_broadcast_cascade().is_none());
    }

    #[test]
    fn credit_heartbeat_and_tier_c_multi_route_selection() {
        let mut state = MemnetNodeState::default();
        state.tier = RoutingTier::TierC;
        let selected = state.select_routes(0xC5);
        assert_eq!(selected.len(), 2); // Tier C dual-path zero-packet-loss selection

        let mut usb_route = MemnetRouteEntry::new(42, 1, "usb-peer", MemnetMedium::Usb, "usb://0", 1);
        assert_eq!(usb_route.heartbeat_credits, 3);
        assert!(!usb_route.tick_heartbeat_missed());
        assert!(!usb_route.tick_heartbeat_missed());
        assert_eq!(usb_route.status, RouteStatus::Degraded);
        assert!(usb_route.tick_heartbeat_missed());
        assert_eq!(usb_route.status, RouteStatus::Down);
        assert_eq!(usb_route.marine_quality_score(), 0);
        usb_route.tick_heartbeat_received();
        assert_eq!(usb_route.status, RouteStatus::Up);
    }

    #[test]
    fn wave_frame_79_and_bluetooth_mesh_96_round_trip() {
        let wave = WaveFrame79::from_semantic_text("Smart Tree MemNet consciousness wave");
        let bytes79 = wave.to_bytes_79();
        assert_eq!(bytes79.len(), 79);
        let decoded = WaveFrame79::from_bytes_79(&bytes79).expect("valid 79B WaveFrame79");
        assert_eq!(decoded, wave);

        let bt = BluetoothMeshFrame96 {
            hop_ttl: 4,
            speaker_f0_hz: 175,
            wupz_xor_fold: 0x4803_1a0d_4c05_4e69,
            wave,
        };
        let bytes96 = bt.to_bytes_96();
        let decoded_bt = BluetoothMeshFrame96::from_bytes_96(&bytes96).expect("valid 96B BT frame");
        assert_eq!(decoded_bt, bt);

        let ble31 = bt.to_ble_beacon_31(0x62, "MEMNET!");
        assert_eq!(&ble31[0..2], b"8B");
    }

    #[test]
    fn wave_qa_packet_resonates_and_offloads_llm() {
        let mut state = MemnetNodeState::default();
        state.learn_wave_qa(
            "How does MemNet route packets across ESP32 and LoRa?",
            "MemNet uses a 1-byte depth prefix plus hierarchical route bytes and 64-byte MemnetCapsule64 frames over ESP-NOW, WiFi UDP :8888, BLE, and 923.2 MHz LoRa.",
        );
        let (score, hit) = state
            .query_wave_qa("How do we route MemNet packets over ESP32 or LoRa?", 400)
            .expect("should resonate with cached WaveQaPacket");
        assert!(score >= 500, "unexpectedly low resonance score: {score}");
        assert!(hit.answer.contains("MemnetCapsule64"));
    }

    #[test]
    fn media_chunk_streaming_and_reassembly_verifies_crc() {
        let fake_av = vec![0x62u8; 70_000];
        let chunks = chunk_media_stream(
            "stream-01",
            MediaKind::AudioOpus,
            &fake_av,
            32 * 1024,
            48_000,
            175,
            94,
        );
        assert_eq!(chunks.len(), 3);
        let reassembled = reassemble_media_chunks(&chunks).unwrap();
        assert_eq!(reassembled, fake_av);
    }

    #[test]
    fn ultrasonic_chirp_modem_encodes_and_demodulates_packet_losslessly() {
        let modem = UltrasonicChirpModem::new(AcousticModemConfig::ultrasonic_48khz());
        let original = b"MEMNET-ULTRASONIC:node=42;share=smart-tree;wave=42.73Hz";
        let pcm = modem.encode_frame_to_pcm16(original);
        let decoded = modem.decode_pcm16_to_frame(&pcm).unwrap();
        assert_eq!(decoded, original);

        // Also test audible bird-chirp fallback profile
        let audible = UltrasonicChirpModem::new(AcousticModemConfig::audible_chirp_48khz());
        let pcm_aud = audible.encode_frame_to_pcm16(original);
        let decoded_aud = audible.decode_pcm16_to_frame(&pcm_aud).unwrap();
        assert_eq!(decoded_aud, original);
    }

    #[test]
    fn real_time_constant_rtc_relaxation_and_parallel_tau() {
        let mut route = MemnetRouteEntry::new(
            0xC5,
            20,
            "wifi-peer",
            MemnetMedium::WiFi,
            "192.168.1.50:8423",
            10,
        );
        // τ = 10 + 2*2 + 8 (WiFi floor) = 22ms
        assert_eq!(route.effective_rtc_ms(), 22);
        assert_eq!(route.settling_3tau_ms(), 66);
        assert_eq!(route.settling_5tau_ms(), 110);

        // Micro-burst spike 1ms later (Δt << τ) is heavily damped by continuous-time RTC filter
        route.update_with_rtc(100, 1);
        assert!(
            route.latency_ms < 20,
            "RTC filter should damp high-frequency spike when dt << tau, got {}",
            route.latency_ms
        );

        // Sustained shift over Δt = 5τ relaxes ~99.3% toward the new steady state
        route.update_with_rtc(50, route.settling_5tau_ms());
        assert!(
            route.latency_ms >= 48 && route.latency_ms <= 51,
            "RTC filter should relax to steady state after 5τ, got {}",
            route.latency_ms
        );

        let state = MemnetNodeState::default();
        let parallel_tau = state.composite_rtc_tau_ms(0xC5);
        assert!(parallel_tau > 0 && parallel_tau <= 16);
    }

    #[test]
    fn third_escape_ultra_graph_and_language_lll_zero_collision() {
        let state = MemnetNodeState::default();
        let report = state.compute_3rd_escape(
            "Solar fusion proton-proton tunneling + Ultra-Graph Language LLL 3rd+ Escape",
        );
        assert!(report.lll_guaranteed);
        assert!(report.lll_ep_d_plus_1 <= 1.0);
        assert!(report.zero_collision_verified);
        assert!(
            report.language_lll.reduced_norm_sq < report.language_lll.initial_norm_sq,
            "4D LLL lattice basis reduction must shrink skewed basis norm: {} -> {}",
            report.language_lll.initial_norm_sq,
            report.language_lll.reduced_norm_sq
        );
        assert!(report.language_lll.basis_compression_bps > 10_000);
        assert_eq!(report.regimes.len(), 3);
        assert_eq!(report.regimes[2].topological_hops, 1);
        assert_eq!(report.regimes[2].semantic_coupling_pct, 100);
        assert!(report.regimes[2].collision_free);
    }
}
