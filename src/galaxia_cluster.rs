use crate::{
    gaia_omni::{run_gaia_omni_mode, GaiaOmniParams},
    kernels::{
        run_erdos_renyi_scanning, run_erdos_steiner_pruning, run_moser_tardos_splitting,
        run_ramsey_clique_analysis, run_szemeredi_compression,
    },
    model_cache::{fnv1a_64, GalaxiaKvPacket, LlamaModelCache, McpProgressEmitter},
    ErdosRenyiParams, ErdosSteinerParams, MoserTardosParams, RamseyCliqueParams, SzemerediParams,
};
use base64::{engine::general_purpose::STANDARD as BASE64_STD, Engine as _};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::{
    collections::{BTreeMap, BTreeSet},
    sync::{Arc, RwLock},
    time::Duration,
};
use tokio::sync::broadcast;
use tokio_stream::StreamExt;

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
        (
            other.term,
            other.elected_at_lamport,
            &other.coordinator_id,
        ) > (self.term, self.elected_at_lamport, &self.coordinator_id)
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

    pub fn merge(
        &mut self,
        remote: &GalaxiaClusterCrdt,
        self_node_id: &str,
    ) -> CrdtMergeOutcome {
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
            if peer_id == self_node_id || rec.status == MemberStatus::Dead || rec.base_url.is_empty()
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

#[derive(Clone)]
pub struct GalaxiaGossipBus {
    pub node_id: String,
    pub crdt: Arc<RwLock<GalaxiaClusterCrdt>>,
    pub tx: broadcast::Sender<GalaxiaGossipEvent>,
    pub http_client: Client,
}

impl GalaxiaGossipBus {
    pub fn new(node_id: impl Into<String>) -> Self {
        Self::with_base_url(node_id, "")
    }

    pub fn with_base_url(node_id: impl Into<String>, base_url: impl Into<String>) -> Self {
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
        }
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
            let _ = self.tx.send(GalaxiaGossipEvent::CrdtStateSync { crdt: snapshot });
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
            let _ = self.tx.send(GalaxiaGossipEvent::CrdtStateSync { crdt: snapshot });
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
                let _ = self.tx.send(GalaxiaGossipEvent::CrdtStateSync { crdt: snapshot });
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
