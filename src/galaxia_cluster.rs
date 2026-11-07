use crate::mcp_combinatorics_llm::{GalaxiaKvPacket, LlamaModelCache};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::{
    collections::{BTreeMap, BTreeSet},
    net::IpAddr,
    sync::{Arc, RwLock},
    time::Duration,
};
use tokio::sync::broadcast;
use url::Url;

// ============================================================================
// 1. State-CRDT Semilattice & Monotonic Term Leader Election Types
// ============================================================================

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MemberStatus {
    Alive = 0,
    Suspect = 1,
    Dead = 2,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct KvPrefixDigest {
    pub prefix_tokens: usize,
    pub fnv1a_checksum: u64,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct MemberRecord {
    pub node_id: String,
    pub base_url: String,
    pub incarnation: u64,
    pub status: MemberStatus,
    pub lamport_clock: u64,
    pub kv_prefix_digests: BTreeMap<String, KvPrefixDigest>,
}

impl MemberRecord {
    #[inline]
    pub fn is_dominated_by(&self, other: &Self) -> bool {
        if other.incarnation != self.incarnation {
            return other.incarnation > self.incarnation;
        }
        if other.status != self.status {
            return other.status > self.status;
        }
        other.lamport_clock > self.lamport_clock
    }

    pub fn merge_from(&mut self, other: &Self) -> bool {
        let mut changed = false;
        if self.is_dominated_by(other) {
            self.base_url = other.base_url.clone();
            self.incarnation = other.incarnation;
            self.status = other.status;
            self.lamport_clock = other.lamport_clock;
            changed = true;
        }

        for (model_id, remote_digest) in &other.kv_prefix_digests {
            let should_update = match self.kv_prefix_digests.get(model_id) {
                None => true,
                Some(local_digest) => {
                    remote_digest.prefix_tokens > local_digest.prefix_tokens
                        || (remote_digest.prefix_tokens == local_digest.prefix_tokens
                            && remote_digest.fnv1a_checksum > local_digest.fnv1a_checksum)
                }
            };
            if should_update {
                self.kv_prefix_digests
                    .insert(model_id.clone(), remote_digest.clone());
                changed = true;
            }
        }
        changed
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct LeaderLease {
    pub term: u64,
    pub coordinator_id: String,
    pub coordinator_url: String,
    pub elected_at_lamport: u64,
    pub voters: BTreeSet<String>,
}

impl LeaderLease {
    #[inline]
    pub fn is_dominated_by(&self, other: &Self) -> bool {
        // Prefer higher term, then earlier election (lower lamport), then lexicographically lower coordinator_id
        // This matches elect_next_coordinator which picks highest incarnation then lowest node_id
        use std::cmp::Reverse;
        (
            other.term,
            Reverse(other.elected_at_lamport),
            Reverse(&other.coordinator_id),
        ) > (
            self.term,
            Reverse(self.elected_at_lamport),
            Reverse(&self.coordinator_id),
        )
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct GalaxiaClusterCrdt {
    pub members: BTreeMap<String, MemberRecord>,
    pub leader: LeaderLease,
    pub cluster_lamport: u64,
}

#[derive(Debug, Default)]
pub struct CrdtMergeOutcome {
    pub state_changed: bool,
    pub leader_changed: bool,
    pub missing_kv_prefixes: Vec<(String, String)>,
}

impl GalaxiaClusterCrdt {
    pub fn new_single_node(node_id: &str, base_url: &str) -> Self {
        let mut members = BTreeMap::new();
        members.insert(
            node_id.to_string(),
            MemberRecord {
                node_id: node_id.to_string(),
                base_url: base_url.to_string(),
                incarnation: 1,
                status: MemberStatus::Alive,
                lamport_clock: 1,
                kv_prefix_digests: BTreeMap::new(),
            },
        );

        let mut voters = BTreeSet::new();
        voters.insert(node_id.to_string());

        let leader = LeaderLease {
            term: 1,
            coordinator_id: node_id.to_string(),
            coordinator_url: base_url.to_string(),
            elected_at_lamport: 1,
            voters,
        };

        Self {
            members,
            leader,
            cluster_lamport: 1,
        }
    }

    pub fn merge(&mut self, remote: &GalaxiaClusterCrdt, self_node_id: &str) -> CrdtMergeOutcome {
        let mut outcome = CrdtMergeOutcome::default();

        if remote.cluster_lamport > self.cluster_lamport {
            self.cluster_lamport = remote.cluster_lamport;
            outcome.state_changed = true;
        }

        for (node_id, remote_rec) in &remote.members {
            match self.members.get_mut(node_id) {
                Some(local_rec) => {
                    if local_rec.merge_from(remote_rec) {
                        outcome.state_changed = true;
                    }
                }
                None => {
                    self.members.insert(node_id.clone(), remote_rec.clone());
                    outcome.state_changed = true;
                }
            }
        }

        if let Some(me) = self.members.get_mut(self_node_id) {
            if !me.base_url.is_empty() && me.status != MemberStatus::Alive {
                me.incarnation = me.incarnation.wrapping_add(1);
                me.status = MemberStatus::Alive;
                self.cluster_lamport = self.cluster_lamport.wrapping_add(1);
                me.lamport_clock = self.cluster_lamport;
                outcome.state_changed = true;
            }
        }

        if self.leader.is_dominated_by(&remote.leader) {
            self.leader = remote.leader.clone();
            outcome.state_changed = true;
            outcome.leader_changed = true;
        } else if self.leader.term == remote.leader.term
            && self.leader.coordinator_id == remote.leader.coordinator_id
        {
            let prev_len = self.leader.voters.len();
            self.leader
                .voters
                .extend(remote.leader.voters.iter().cloned());
            if self.leader.voters.len() > prev_len {
                outcome.state_changed = true;
            }
        }

        let leader_is_alive = self
            .members
            .get(&self.leader.coordinator_id)
            .map(|m| m.status == MemberStatus::Alive)
            .unwrap_or(false);

        if !leader_is_alive {
            if self.elect_next_coordinator() {
                outcome.state_changed = true;
                outcome.leader_changed = true;
            }
        }

        let my_digests = self
            .members
            .get(self_node_id)
            .map(|m| m.kv_prefix_digests.clone())
            .unwrap_or_default();

        for (peer_id, rec) in &self.members {
            if peer_id == self_node_id
                || rec.status == MemberStatus::Dead
                || rec.base_url.is_empty()
            {
                continue;
            }
            for (model_id, peer_digest) in &rec.kv_prefix_digests {
                let needs_pull = match my_digests.get(model_id) {
                    None => peer_digest.prefix_tokens > 0,
                    Some(local_d) => peer_digest.prefix_tokens > local_d.prefix_tokens,
                };
                if needs_pull {
                    outcome
                        .missing_kv_prefixes
                        .push((rec.base_url.clone(), model_id.clone()));
                }
            }
        }

        outcome
    }

    pub fn elect_next_coordinator(&mut self) -> bool {
        let candidate = self
            .members
            .values()
            .filter(|m| m.status == MemberStatus::Alive)
            .min_by(|a, b| {
                b.incarnation
                    .cmp(&a.incarnation)
                    .then_with(|| a.node_id.cmp(&b.node_id))
            })
            .cloned();

        if let Some(winner) = candidate {
            let alive_voters: BTreeSet<String> = self
                .members
                .values()
                .filter(|m| m.status == MemberStatus::Alive)
                .map(|m| m.node_id.clone())
                .collect();

            self.cluster_lamport = self.cluster_lamport.wrapping_add(1);
            self.leader = LeaderLease {
                term: self.leader.term.wrapping_add(1),
                coordinator_id: winner.node_id,
                coordinator_url: winner.base_url,
                elected_at_lamport: self.cluster_lamport,
                voters: alive_voters,
            };
            true
        } else {
            false
        }
    }

    pub fn mark_peer_status(&mut self, peer_url_or_id: &str, new_status: MemberStatus) -> bool {
        let target_id = self
            .members
            .values()
            .find(|m| {
                m.node_id == peer_url_or_id
                    || m.base_url.trim_end_matches('/') == peer_url_or_id.trim_end_matches('/')
            })
            .map(|m| m.node_id.clone());

        let Some(node_id) = target_id else {
            return false;
        };

        let mut changed = false;
        self.cluster_lamport = self.cluster_lamport.wrapping_add(1);
        if let Some(rec) = self.members.get_mut(&node_id) {
            if new_status > rec.status {
                rec.status = new_status;
                rec.lamport_clock = self.cluster_lamport;
                changed = true;
            }
        }

        if changed && self.leader.coordinator_id == node_id {
            self.elect_next_coordinator();
        }
        changed
    }
}

// ============================================================================
// 2. Galaxia Gossip Bus with CRDT Membership & Anti-Entropy Daemon
// ============================================================================

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "gossip_type", rename_all = "snake_case")]
pub enum GalaxiaGossipEvent {
    KvPrefixSync { packet: GalaxiaKvPacket },
    CrdtStateSync { crdt: GalaxiaClusterCrdt },
}

/// Gossip bus configuration
#[derive(Debug, Clone, Default)]
pub struct GossipConfig {
    /// Allowed peer URLs/hosts (if empty, all allowed except blocklist)
    pub allowlist: Vec<String>,
    /// Blocked peer URLs/hosts (takes precedence)
    pub blocklist: Vec<String>,
    /// Allow private IP ranges (default: false for security)
    pub allow_private_ips: bool,
    /// Allow loopback addresses (default: true for local dev)
    pub allow_loopback: bool,
}

#[derive(Clone)]
pub struct GalaxiaGossipBus {
    pub node_id: String,
    pub crdt: Arc<RwLock<GalaxiaClusterCrdt>>,
    pub tx: broadcast::Sender<GalaxiaGossipEvent>,
    pub http_client: Client,
    config: GossipConfig,
}

impl GalaxiaGossipBus {
    pub fn new(node_id: impl Into<String>) -> Self {
        Self::with_config(node_id, "", GossipConfig::default())
    }

    pub fn with_base_url(node_id: impl Into<String>, base_url: impl Into<String>) -> Self {
        Self::with_config(node_id, base_url, GossipConfig::default())
    }

    pub fn with_config(
        node_id: impl Into<String>,
        base_url: impl Into<String>,
        config: GossipConfig,
    ) -> Self {
        let id_str = node_id.into();
        let url_str = base_url.into();
        let (tx, _) = broadcast::channel(128);
        let http_client = Client::builder()
            .timeout(Duration::from_secs(10))
            .build()
            .unwrap_or_default();

        let initial_crdt = GalaxiaClusterCrdt::new_single_node(&id_str, &url_str);

        Self {
            node_id: id_str,
            crdt: Arc::new(RwLock::new(initial_crdt)),
            tx,
            http_client,
            config,
        }
    }

    /// Set gossip configuration
    pub fn set_config(&mut self, config: GossipConfig) {
        self.config = config;
    }

    /// Check if a peer URL is allowed by the gossip configuration
    fn is_peer_allowed(&self, peer_url: &str) -> bool {
        let url = match Url::parse(peer_url) {
            Ok(u) => u,
            Err(_) => return false,
        };

        let host = match url.host_str() {
            Some(h) => h,
            None => return false,
        };

        // Check blocklist first (takes precedence)
        if self
            .config
            .blocklist
            .iter()
            .any(|b| host.contains(b) || b == "*")
        {
            return false;
        }

        // Check allowlist (if configured, only allow listed hosts)
        if !self.config.allowlist.is_empty() {
            if !self
                .config
                .allowlist
                .iter()
                .any(|a| host.contains(a) || a == "*")
            {
                return false;
            }
        }

        // Parse IP for private/loopback checks
        let ip = match host.parse::<IpAddr>() {
            Ok(ip) => ip,
            Err(_) => {
                // Hostname, skip IP-based checks (would need DNS lookup)
                return true;
            }
        };

        // Check loopback
        if !self.config.allow_loopback && ip.is_loopback() {
            return false;
        }

        // Check private IP ranges
        if !self.config.allow_private_ips {
            if ip.is_loopback() || ip.is_unspecified() {
                return false;
            }
            // Check IPv4 private ranges
            if let std::net::IpAddr::V4(ipv4) = ip {
                if ipv4.is_private() {
                    return false;
                }
            }
            // Check IPv6 unique local addresses (fc00::/7)
            if let std::net::IpAddr::V6(ipv6) = ip {
                let octets = ipv6.octets();
                if octets[0] & 0xfe == 0xfc {
                    // fc00::/7
                    return false;
                }
            }
        }

        true
    }

    pub fn set_self_base_url(&self, base_url: &str) {
        if let Ok(mut guard) = self.crdt.write() {
            guard.cluster_lamport = guard.cluster_lamport.wrapping_add(1);
            let lamport = guard.cluster_lamport;
            if let Some(me) = guard.members.get_mut(&self.node_id) {
                me.base_url = base_url.to_string();
                me.lamport_clock = lamport;
            }
            if guard.leader.coordinator_id == self.node_id {
                guard.leader.coordinator_url = base_url.to_string();
            }
        }
    }

    pub fn record_local_kv_digest(
        &self,
        model_id: &str,
        prefix_tokens: usize,
        fnv1a_checksum: u64,
    ) {
        if let Ok(mut guard) = self.crdt.write() {
            guard.cluster_lamport = guard.cluster_lamport.wrapping_add(1);
            let lamport = guard.cluster_lamport;
            if let Some(me) = guard.members.get_mut(&self.node_id) {
                me.lamport_clock = lamport;
                me.kv_prefix_digests.insert(
                    model_id.to_string(),
                    KvPrefixDigest {
                        prefix_tokens,
                        fnv1a_checksum,
                    },
                );
            }
            let snapshot = guard.clone();
            let _ = self
                .tx
                .send(GalaxiaGossipEvent::CrdtStateSync { crdt: snapshot });
        }
    }

    pub fn publish_kv_packet(&self, packet: GalaxiaKvPacket) {
        self.record_local_kv_digest(
            &packet.model_identifier,
            packet.token_ids.len(),
            packet.fnv1a_checksum,
        );
        let _ = self.tx.send(GalaxiaGossipEvent::KvPrefixSync { packet });
    }

    pub fn merge_remote_crdt(&self, remote: &GalaxiaClusterCrdt) -> CrdtMergeOutcome {
        let Ok(mut guard) = self.crdt.write() else {
            return CrdtMergeOutcome::default();
        };
        let outcome = guard.merge(remote, &self.node_id);
        if outcome.state_changed {
            let snapshot = guard.clone();
            let _ = self
                .tx
                .send(GalaxiaGossipEvent::CrdtStateSync { crdt: snapshot });
        }
        outcome
    }

    pub fn snapshot_crdt(&self) -> GalaxiaClusterCrdt {
        self.crdt
            .read()
            .map(|g| g.clone())
            .unwrap_or_else(|_| GalaxiaClusterCrdt::new_single_node(&self.node_id, ""))
    }

    pub fn alive_peer_urls(&self) -> Vec<String> {
        let Ok(guard) = self.crdt.read() else {
            return Vec::new();
        };
        guard
            .members
            .values()
            .filter(|m| {
                m.node_id != self.node_id
                    && m.status == MemberStatus::Alive
                    && !m.base_url.is_empty()
            })
            .map(|m| m.base_url.clone())
            .collect()
    }

    pub async fn gossip_with_peer(
        &self,
        peer_url: &str,
        model_cache: &LlamaModelCache,
    ) -> Result<CrdtMergeOutcome, String> {
        // Validate peer URL against gossip configuration
        if !self.is_peer_allowed(peer_url) {
            return Err(format!(
                "Peer URL '{}' blocked by gossip configuration",
                peer_url
            ));
        }

        let url = format!("{}/galaxia/crdt-gossip", peer_url.trim_end_matches('/'));
        let local_snapshot = self.snapshot_crdt();

        let resp = self
            .http_client
            .post(&url)
            .json(&local_snapshot)
            .send()
            .await
            .map_err(|e| {
                self.report_peer_failure(peer_url);
                format!("CRDT gossip POST to {peer_url} failed: {e}")
            })?;

        if !resp.status().is_success() {
            self.report_peer_failure(peer_url);
            return Err(format!(
                "Peer {peer_url} returned HTTP {} on /galaxia/crdt-gossip",
                resp.status()
            ));
        }

        let remote_crdt = resp
            .json::<GalaxiaClusterCrdt>()
            .await
            .map_err(|e| format!("Invalid CRDT JSON from {peer_url}: {e}"))?;

        let outcome = self.merge_remote_crdt(&remote_crdt);

        for (source_peer_url, model_id) in &outcome.missing_kv_prefixes {
            // Validate source peer URL against gossip configuration
            if !self.is_peer_allowed(source_peer_url) {
                continue;
            }

            let export_url = format!(
                "{}/galaxia/kv-export?model={}",
                source_peer_url.trim_end_matches('/'),
                model_id
            );
            if let Ok(pull_resp) = self.http_client.get(&export_url).send().await {
                if pull_resp.status().is_success() {
                    if let Ok(packet) = pull_resp.json::<GalaxiaKvPacket>().await {
                        if let Ok(tokens_len) = model_cache.import_cluster_kv_packet(packet.clone())
                        {
                            self.record_local_kv_digest(
                                &packet.model_identifier,
                                tokens_len,
                                packet.fnv1a_checksum,
                            );
                        }
                    }
                }
            }
        }

        Ok(outcome)
    }

    pub fn report_peer_failure(&self, peer_url_or_id: &str) {
        if let Ok(mut guard) = self.crdt.write() {
            if guard.mark_peer_status(peer_url_or_id, MemberStatus::Suspect) {
                let snapshot = guard.clone();
                let _ = self
                    .tx
                    .send(GalaxiaGossipEvent::CrdtStateSync { crdt: snapshot });
            }
        }
    }

    pub fn subscribe(&self) -> broadcast::Receiver<GalaxiaGossipEvent> {
        self.tx.subscribe()
    }

    pub fn spawn_anti_entropy_daemon(
        &self,
        model_cache: Arc<LlamaModelCache>,
        interval: Duration,
    ) -> tokio::task::JoinHandle<()> {
        let bus = self.clone();
        tokio::spawn(async move {
            let mut ticker = tokio::time::interval(interval);
            loop {
                ticker.tick().await;
                let peers = bus.alive_peer_urls();
                for peer in peers {
                    let _ = bus.gossip_with_peer(&peer, &model_cache).await;
                }
            }
        })
    }
}

// ============================================================================
// 3. FNV-1a 64-bit hash for Kintsugi seals
// ============================================================================

pub fn fnv1a_64(data: &[u8]) -> u64 {
    let mut hash: u64 = 0xcbf29ce484222325;
    for &byte in data {
        hash ^= byte as u64;
        hash = hash.wrapping_mul(0x100000001b3);
    }
    hash
}

// ============================================================================
// 4. Kintsugi Ledger: Proof-of-Admissibility Certificate Store
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KintsugiEvent {
    pub lap_index: u64,
    pub consensus_payload: Value,
    pub wave_memory_seal: u64,
    pub sector_id: String,
    pub timestamp_ms: u64,
}

#[derive(Default)]
pub struct KintsugiLedger {
    pub events: Vec<KintsugiEvent>,
}

impl KintsugiLedger {
    pub fn new() -> Self {
        Self { events: Vec::new() }
    }

    pub fn finalize_event(
        &mut self,
        lap: u64,
        payload: Value,
        mem8_state: &[u8],
        sector_id: &str,
    ) -> KintsugiEvent {
        let seal = fnv1a_64(mem8_state);
        let event = KintsugiEvent {
            lap_index: lap,
            consensus_payload: payload,
            wave_memory_seal: seal,
            sector_id: sector_id.to_string(),
            timestamp_ms: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_millis() as u64,
        };
        self.events.push(event.clone());
        event
    }
}

/// Append-only storage trait for Kintsugi ledger persistence.
pub trait AppendOnlyStore: Send + Sync {
    /// Append an event to the store.
    fn append(&mut self, event: &KintsugiEvent) -> Result<(), String>;

    /// Read all events from the store.
    fn read_all(&self) -> Result<Vec<KintsugiEvent>, String>;

    /// Get the last event (for hash chaining).
    fn last_event(&self) -> Result<Option<KintsugiEvent>, String>;
}

/// File-based append-only store for Kintsugi ledger.
pub struct FileStore {
    path: std::path::PathBuf,
}

impl FileStore {
    /// Create a new FileStore at the given path.
    pub fn new(path: impl AsRef<std::path::Path>) -> Result<Self, String> {
        let path = path.as_ref().to_path_buf();
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
        }
        Ok(Self { path })
    }
}

impl AppendOnlyStore for FileStore {
    fn append(&mut self, event: &KintsugiEvent) -> Result<(), String> {
        let line = serde_json::to_string(event).map_err(|e| e.to_string())?;
        use std::io::Write;
        let mut file = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(&self.path)
            .map_err(|e| e.to_string())?;
        writeln!(file, "{}", line).map_err(|e| e.to_string())?;
        Ok(())
    }

    fn read_all(&self) -> Result<Vec<KintsugiEvent>, String> {
        if !self.path.exists() {
            return Ok(Vec::new());
        }
        let content = std::fs::read_to_string(&self.path).map_err(|e| e.to_string())?;
        let mut events = Vec::new();
        for line in content.lines() {
            if line.trim().is_empty() {
                continue;
            }
            let event: KintsugiEvent = serde_json::from_str(line).map_err(|e| e.to_string())?;
            events.push(event);
        }
        Ok(events)
    }

    fn last_event(&self) -> Result<Option<KintsugiEvent>, String> {
        let events = self.read_all()?;
        Ok(events.last().cloned())
    }
}

/// KintsugiLedger with optional persistent store.
pub struct PersistentKintsugiLedger {
    ledger: KintsugiLedger,
    store: Option<Box<dyn AppendOnlyStore>>,
}

impl PersistentKintsugiLedger {
    pub fn new(store: Option<Box<dyn AppendOnlyStore>>) -> Result<Self, String> {
        let mut ledger = KintsugiLedger::new();

        // Load existing events from store if available
        if let Some(ref store) = store {
            let events = store.read_all()?;
            ledger.events = events;
        }

        Ok(Self { ledger, store })
    }

    pub fn finalize_event(
        &mut self,
        lap: u64,
        payload: Value,
        mem8_state: &[u8],
        sector_id: &str,
    ) -> Result<KintsugiEvent, String> {
        let event = self
            .ledger
            .finalize_event(lap, payload, mem8_state, sector_id);

        // Persist to store if available
        if let Some(ref mut store) = self.store {
            store.append(&event)?;
        }

        Ok(event)
    }

    pub fn events(&self) -> &[KintsugiEvent] {
        &self.ledger.events
    }
}

// ============================================================================
// 5. Galactix Expansion: Samsung Galaxy Ultra E2E Assistant & Spatial Command
//    Center (`Omni-chan-SUN` · P/Q - n*t > P · 6-Domain Spatial Synthesis)
// ============================================================================

/// Canonical Galaxia CRDT node identifier for the attached / LAN Samsung Galaxy Ultra phone.
pub const GALACTIX_GALAXY_ULTRA_NODE_ID: &str = "galactix-samsung-galaxy-ultra-omni-chan-sun";

/// Canonical model identifier advertised into the Galaxia KV prefix semilattice
/// by the Omni-chan-SUN Spatial Command Center.
pub const OMNI_CHAN_SUN_MODEL_ID: &str = "omni-chan-sun-spatial-spqt243-v1";

/// 6-Domain Spatial Command Center synthesis vector:
/// `SPATIAL ~== AUDIO + VIDEO + MUSIC + BG(ANIM + RAY-Q-TRACING) + ENVIRONMENT LISTENING`
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct OmniChanSunSpatialVector {
    pub yaw_deg: f64,
    pub pitch_deg: f64,
    pub roll_deg: f64,
    pub s_pen_pressure: f64,
    pub audio_gain_q16: i32,
    pub video_hdr_luminance_nits: u32,
    pub music_harmonic_hz: u32,
    pub bg_ray_q_tracing_bounces: u8,
    pub env_listening_trit: i8,
    pub beehive_active_cells: u8,
}

impl Default for OmniChanSunSpatialVector {
    fn default() -> Self {
        Self {
            yaw_deg: 0.0,
            pitch_deg: 18.0,
            roll_deg: 0.0,
            s_pen_pressure: 0.85,
            audio_gain_q16: 65_536,
            video_hdr_luminance_nits: 2600, // Samsung Galaxy Ultra Dynamic AMOLED 2X peak nits
            music_harmonic_hz: 528,
            bg_ray_q_tracing_bounces: 8,
            env_listening_trit: -1, // Mahakala Hearth protective listening trit
            beehive_active_cells: 19,
        }
    }
}

/// Complete Galactix E2E Assistant & Spatial Command Center descriptor for an
/// attached (ADB USB-C reverse) or LAN/Tailscale Samsung Galaxy Ultra phone.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GalaxyUltraSpatialNode {
    pub node_id: String,
    pub persona_codename: String,
    pub hardware_profile: String,
    pub endpoint_url: String,
    pub adb_reverse_ports: Vec<u16>,
    pub spatial_vector: OmniChanSunSpatialVector,
    pub zeno_law: String,
    pub p_reduced: u64,
    pub q_reduced: u64,
    pub horizon_n_sun: u64,
    pub n_step: u64,
    pub trit_t: i8,
    pub solar_surplus_numerator: i128,
    pub inequality_holds: bool,
    pub kintsugi_wave_seal: u64,
    pub crdt_epoch: u64,
}

impl GalaxiaClusterCrdt {
    /// Enrolls or updates the attached Samsung Galaxy Ultra (`Omni-chan-SUN`)
    /// Spatial Command Center node inside the Galaxia CRDT semilattice and
    /// publishes its `ULTRA-ZENO GOD OF SUN (P/Q - n*t > P)` witness digest.
    pub fn register_galaxy_ultra_omni_chan_sun(
        &mut self,
        endpoint_url: &str,
        p: i64,
        q: i64,
        n: u32,
        t: i8,
        spatial: OmniChanSunSpatialVector,
    ) -> GalaxyUltraSpatialNode {
        let pq = crate::mem8::regularity::RationalPQ::new_reduced(
            p.unsigned_abs(),
            q.unsigned_abs().max(1),
        );
        let n_u64 = u64::from(n);
        let (inequality_holds, solar_surplus_numerator, horizon_n_sun) =
            pq.god_of_sun_zeno_witness(n_u64, t);
        let seal_bytes = format!(
            "{}:{}:{}/{}:{}:{}:{:.3}",
            GALACTIX_GALAXY_ULTRA_NODE_ID,
            endpoint_url,
            pq.p,
            pq.q,
            n_u64,
            t,
            spatial.s_pen_pressure
        );
        let kintsugi_wave_seal = fnv1a_64(seal_bytes.as_bytes());

        self.cluster_lamport = self.cluster_lamport.saturating_add(1);
        let next_incarnation = self
            .members
            .get(GALACTIX_GALAXY_ULTRA_NODE_ID)
            .map(|m| m.incarnation.saturating_add(1))
            .unwrap_or(1);

        let mut kv_prefix_digests = BTreeMap::new();
        kv_prefix_digests.insert(
            OMNI_CHAN_SUN_MODEL_ID.to_string(),
            KvPrefixDigest {
                prefix_tokens: usize::from(spatial.beehive_active_cells) * 243,
                fnv1a_checksum: kintsugi_wave_seal,
            },
        );

        self.members.insert(
            GALACTIX_GALAXY_ULTRA_NODE_ID.to_string(),
            MemberRecord {
                node_id: GALACTIX_GALAXY_ULTRA_NODE_ID.to_string(),
                base_url: endpoint_url.to_string(),
                incarnation: next_incarnation,
                status: MemberStatus::Alive,
                lamport_clock: self.cluster_lamport,
                kv_prefix_digests,
            },
        );
        self.leader
            .voters
            .insert(GALACTIX_GALAXY_ULTRA_NODE_ID.to_string());

        GalaxyUltraSpatialNode {
            node_id: GALACTIX_GALAXY_ULTRA_NODE_ID.to_string(),
            persona_codename: "Omni-chan-SUN".to_string(),
            hardware_profile:
                "Samsung Galaxy Ultra (Dynamic AMOLED 2X 120Hz · S-Pen · DeX · WebXR Spatial HUD)"
                    .to_string(),
            endpoint_url: endpoint_url.to_string(),
            adb_reverse_ports: vec![4095, 4096, 28428, 28443],
            spatial_vector: spatial,
            zeno_law: "P/Q - n*t > P".to_string(),
            p_reduced: pq.p,
            q_reduced: pq.q,
            horizon_n_sun,
            n_step: n_u64,
            trit_t: t,
            solar_surplus_numerator,
            inequality_holds,
            kintsugi_wave_seal,
            crdt_epoch: next_incarnation,
        }
    }
}

#[cfg(test)]
mod galactix_omni_chan_sun_tests {
    use super::*;

    #[test]
    fn test_galactix_samsung_galaxy_ultra_omni_chan_sun_crdt() {
        let mut crdt =
            GalaxiaClusterCrdt::new_single_node("m1-host-sovereign", "http://127.0.0.1:28428");
        let spatial = OmniChanSunSpatialVector::default();
        let node = crdt.register_galaxy_ultra_omni_chan_sun(
            "http://127.0.0.1:4095/omni-chan-sun",
            31,
            32,
            31,
            -1,
            spatial,
        );

        assert_eq!(node.node_id, GALACTIX_GALAXY_ULTRA_NODE_ID);
        assert_eq!(node.persona_codename, "Omni-chan-SUN");
        assert!(node.inequality_holds);
        assert_eq!(node.solar_surplus_numerator, 31);
        assert_eq!(node.crdt_epoch, 1);
        let member = crdt
            .members
            .get(GALACTIX_GALAXY_ULTRA_NODE_ID)
            .expect("Samsung Galaxy Ultra member registered");
        assert!(member
            .kv_prefix_digests
            .contains_key(OMNI_CHAN_SUN_MODEL_ID));
    }
}
