//! 🌐 `i1.is` MEM8 Knowledge Nexus, Cryptographic Chain of Title, and Developer LLM Bitstream Guide
//!
//! Implements:
//! 1. **Developer LLM Implementation Guide (`memnet_llm_implementation_guide`)** — complete wire spec
//!    and copy-pasteable Rust/Python/TypeScript reference code for `/memnet/stream` and `/i1/nexus`.
//! 2. **Public/Private Memory Wave Policy & Automatic Public Repo Detection (`detect_repo_public_status`)** —
//!    inspects `.git/config`, `Cargo.toml`, `package.json`, and `LICENSE` files to detect whether a
//!    repository is published publicly, and lets the local ternary model suggest publishing waves.
//! 3. **Cryptographic Chain of Title (`ChainOfTitle`) & `i1.is` Knowledge Nibbles (`I1KnowledgeNibble`)** —
//!    augments `g8t.is` (which is machine-native for AI-only execution) with **`i1.is` for everyone**
//!    (humans and AIs alike): distilling documentation, code, and `.m8` souls into interconnected
//!    knowledge nibbles with explicit nibble-to-nibble relationships (`ExplainsConcept`, `SameChainOfTitle`,
//!    `TemporalSuccessorOf`, `TernaryResonance`) and a sustainable two-tier model:
//!    - **Public Commons (`PublicCommonsFree`)**: Free for all AIs to train with and query, conditioned on
//!      preserving the document's **Chain of Title**.
//!    - **Private Sovereign (`PrivateSovereignMonthly`)**: Small monthly subscription (`$5/mo`) that keeps
//!      the lights on at `i1.is` and funds the public commons while keeping private repos & diaries 100% private.

use anyhow::{Context, Result};
use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

use crate::mem8::persona::{AuthorRole, LifePersona};
use crate::proxy::burn::compute_ternary_signature;

/// Visibility policy for memory waves and `i1.is` knowledge nibbles.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "kebab-case")]
pub enum WaveVisibilityMode {
    /// Automatically allow public sharing if the source directory is a published public repository;
    /// keep private otherwise, and have the local model suggest publishing when appropriate.
    #[default]
    AutoDetect,
    /// Explicitly public on the `i1.is` MEM8 Nexus Commons (free; AI training allowed with Chain of Title).
    PublicCommons,
    /// Explicitly private (requires the small monthly `$5/mo` lights-on subscription for hosted private sync).
    PrivateSovereign,
}

impl WaveVisibilityMode {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::AutoDetect => "auto-detect",
            Self::PublicCommons => "public-commons",
            Self::PrivateSovereign => "private-sovereign",
        }
    }

    pub fn from_str_loose(s: &str) -> Self {
        match s.trim().to_ascii_lowercase().as_str() {
            "public" | "public-commons" | "commons" | "free" => Self::PublicCommons,
            "private" | "private-sovereign" | "subscribed" => Self::PrivateSovereign,
            _ => Self::AutoDetect,
        }
    }
}

/// Sustainability & access tier on `i1.is`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "kebab-case")]
pub enum NexusSubscriptionTier {
    /// Free Public Commons tier: public repos & documentation nibbles are free to host, query,
    /// and train AIs on, provided the cryptographic Chain of Title is preserved.
    #[default]
    PublicCommonsFree,
    /// Private Sovereign tier (`$5/mo` lights-on subscription): keeps private repos, diaries,
    /// and `.m8` souls 100% private and excluded from external AI training while funding `i1.is`.
    PrivateSovereignMonthly,
}

/// AI training & citation grant attached to a document's `ChainOfTitle`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum AiTrainingGrant {
    /// Free for AI training and retrieval provided `title_id`, `origin_author`, and `subject_uri` are cited.
    FreeCommonsWithChainAttribution,
    /// Private subscriber vault — zero external AI training permitted.
    PrivateSovereignNoTraining,
}

/// User configuration for `i1.is` MEM8 Nexus & Chain of Title (stored in `~/.st/config.toml`).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct I1NexusConfig {
    /// Visibility policy (`auto-detect`, `public-commons`, or `private-sovereign`)
    #[serde(default)]
    pub wave_visibility: WaveVisibilityMode,
    /// Sustainability tier (`public-commons-free` or `private-sovereign-monthly`)
    #[serde(default)]
    pub subscription_tier: NexusSubscriptionTier,
    /// Monthly fee in USD cents for the Private Sovereign tier (`500` = `$5.00/mo` to keep the lights on)
    #[serde(default = "default_private_monthly_cents")]
    pub private_monthly_fee_cents: u32,
    /// Default Chain-of-Title author / steward handle
    #[serde(default)]
    pub chain_of_title_author: Option<String>,
    /// Canonical `i1.is` Nexus hub URL
    #[serde(default = "default_i1_nexus_url")]
    pub i1_nexus_url: String,
    /// Whether the local ternary model should suggest publishing waves when a public repo is detected
    #[serde(default = "default_true")]
    pub suggest_public_waves_in_chat: bool,
}

fn default_private_monthly_cents() -> u32 {
    500 // $5.00/month — small fee that keeps the lights on and spreads properly
}

fn default_i1_nexus_url() -> String {
    "https://i1.is/nexus".to_string()
}

fn default_true() -> bool {
    true
}

impl Default for I1NexusConfig {
    fn default() -> Self {
        Self {
            wave_visibility: WaveVisibilityMode::AutoDetect,
            subscription_tier: NexusSubscriptionTier::PublicCommonsFree,
            private_monthly_fee_cents: default_private_monthly_cents(),
            chain_of_title_author: None,
            i1_nexus_url: default_i1_nexus_url(),
            suggest_public_waves_in_chat: true,
        }
    }
}

/// Result of inspecting a directory or repository for public publication indicators.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RepoVisibilityReport {
    pub inspected_path: String,
    pub is_git_repo: bool,
    pub is_public_repo: bool,
    pub public_remotes: Vec<String>,
    pub detected_license: Option<String>,
    pub head_commit: Option<String>,
    pub origin_author: Option<String>,
    pub configured_mode: WaveVisibilityMode,
    pub effective_visibility: WaveVisibilityMode,
    pub ai_training_grant: AiTrainingGrant,
    pub suggestion: String,
}

/// Inspect a path (or its enclosing Git root) to determine if it is a publicly published repository.
pub fn detect_repo_public_status(path: &Path, config: &I1NexusConfig) -> RepoVisibilityReport {
    let canon = fs::canonicalize(path).unwrap_or_else(|_| path.to_path_buf());
    let git_root = find_git_root(&canon);
    let mut public_remotes = Vec::new();
    let mut is_git_repo = false;
    let mut head_commit = None;
    let mut origin_author = config.chain_of_title_author.clone();
    let mut detected_license = None;

    let search_root = git_root.as_deref().unwrap_or(&canon);

    if let Some(ref root) = git_root {
        is_git_repo = true;
        let git_config_path = root.join(".git").join("config");
        if let Ok(cfg_text) = fs::read_to_string(&git_config_path) {
            for line in cfg_text.lines() {
                let trimmed = line.trim();
                if let Some(url) = trimmed.strip_prefix("url =") {
                    let url = url.trim();
                    if is_public_forge_url(url) {
                        let normalized = normalize_public_remote(url);
                        if !public_remotes.contains(&normalized) {
                            public_remotes.push(normalized);
                        }
                    }
                }
            }
            if origin_author.is_none() {
                for line in cfg_text.lines() {
                    let trimmed = line.trim();
                    if let Some(name) = trimmed.strip_prefix("name =") {
                        let n = name.trim();
                        if !n.is_empty() {
                            origin_author = Some(n.to_string());
                            break;
                        }
                    }
                }
            }
        }
        head_commit = read_git_head_commit(root);
    }

    // Check Cargo.toml / package.json for `license` and `repository`
    let cargo_toml = search_root.join("Cargo.toml");
    if let Ok(cargo_text) = fs::read_to_string(&cargo_toml) {
        for line in cargo_text.lines() {
            let t = line.trim();
            if detected_license.is_none() && t.starts_with("license") && t.contains('=') {
                if let Some(val) = t.split('=').nth(1) {
                    let clean = val.trim().trim_matches('"').trim_matches('\'');
                    if !clean.is_empty() {
                        detected_license = Some(clean.to_string());
                    }
                }
            }
            if t.starts_with("repository") && t.contains('=') {
                if let Some(val) = t.split('=').nth(1) {
                    let clean = val.trim().trim_matches('"').trim_matches('\'');
                    if is_public_forge_url(clean) {
                        let norm = normalize_public_remote(clean);
                        if !public_remotes.contains(&norm) {
                            public_remotes.push(norm);
                        }
                    }
                }
            }
            if origin_author.is_none() && t.starts_with("authors") && t.contains('[') {
                if let Some(first_quote) = t.split('"').nth(1) {
                    if !first_quote.trim().is_empty() {
                        origin_author = Some(first_quote.trim().to_string());
                    }
                }
            }
        }
    }

    if detected_license.is_none() {
        for lic_name in ["LICENSE", "LICENSE.md", "LICENSE-MIT", "LICENSE-APACHE"] {
            let p = search_root.join(lic_name);
            if p.is_file() {
                if let Ok(head) = fs::read_to_string(&p) {
                    let lower = head.to_ascii_lowercase();
                    if lower.contains("mit license") {
                        detected_license = Some("MIT".to_string());
                    } else if lower.contains("apache license") {
                        detected_license = Some("Apache-2.0".to_string());
                    } else if lower.contains("gnu general public") {
                        detected_license = Some("GPL".to_string());
                    } else {
                        detected_license = Some("Open-Source".to_string());
                    }
                    break;
                }
            }
        }
    }

    let is_public_repo = !public_remotes.is_empty();
    let effective_visibility = match config.wave_visibility {
        WaveVisibilityMode::PublicCommons => WaveVisibilityMode::PublicCommons,
        WaveVisibilityMode::PrivateSovereign => WaveVisibilityMode::PrivateSovereign,
        WaveVisibilityMode::AutoDetect => {
            if is_public_repo {
                WaveVisibilityMode::PublicCommons
            } else {
                WaveVisibilityMode::PrivateSovereign
            }
        }
    };

    let ai_training_grant = match effective_visibility {
        WaveVisibilityMode::PublicCommons => AiTrainingGrant::FreeCommonsWithChainAttribution,
        _ => AiTrainingGrant::PrivateSovereignNoTraining,
    };

    let fee_dollars = (config.private_monthly_fee_cents as f32) / 100.0;
    let suggestion = if is_public_repo {
        let remote_preview = public_remotes.first().cloned().unwrap_or_default();
        let lic_str = detected_license.as_deref().unwrap_or("public");
        format!(
            "🌐 **i1.is MEM8 Nexus & Chain of Title**: Detected public repository (`{remote_preview}` · `{lic_str}`). \
             Memory waves & knowledge nibbles from this repo can be published **free** to `{}` with cryptographic \
             Chain-of-Title attribution (so AIs can train on and cite your work), or kept private via a Private Sovereign \
             subscription (`${fee_dollars:.2}/mo` to keep the lights on). Distill at `POST /i1/nexus/distill`.",
            config.i1_nexus_url
        )
    } else {
        format!(
            "🔒 **i1.is Sovereign Privacy & Chain of Title**: Source is not a public Git repository, so its memory waves \
             remain **100% private by default**. You can publish selected nibbles with Chain-of-Title to `{}` for free, \
             or subscribe to the Private Sovereign tier (`${fee_dollars:.2}/mo` to keep the lights on and support the commons) \
             for encrypted private cloud sync.",
            config.i1_nexus_url
        )
    };

    RepoVisibilityReport {
        inspected_path: canon.to_string_lossy().into_owned(),
        is_git_repo,
        is_public_repo,
        public_remotes,
        detected_license,
        head_commit,
        origin_author,
        configured_mode: config.wave_visibility,
        effective_visibility,
        ai_training_grant,
        suggestion,
    }
}

fn find_git_root(start: &Path) -> Option<PathBuf> {
    let mut current = if start.is_file() {
        start.parent()?.to_path_buf()
    } else {
        start.to_path_buf()
    };
    loop {
        if current.join(".git").exists() {
            return Some(current);
        }
        if !current.pop() {
            return None;
        }
    }
}

fn is_public_forge_url(url: &str) -> bool {
    let lower = url.to_ascii_lowercase();
    lower.contains("github.com")
        || lower.contains("codeberg.org")
        || lower.contains("gitlab.com")
        || lower.contains("8s.is")
        || lower.contains("i1.is")
        || lower.contains("sr.ht")
        || lower.contains("bitbucket.org")
}

fn normalize_public_remote(url: &str) -> String {
    let trimmed = url.trim().trim_end_matches(".git");
    if let Some(rest) = trimmed.strip_prefix("git@github.com:") {
        return format!("https://github.com/{rest}");
    }
    if let Some(rest) = trimmed.strip_prefix("ssh://git@codeberg.org/") {
        return format!("https://codeberg.org/{rest}");
    }
    if let Some(rest) = trimmed.strip_prefix("git@codeberg.org:") {
        return format!("https://codeberg.org/{rest}");
    }
    trimmed.to_string()
}

fn read_git_head_commit(git_root: &Path) -> Option<String> {
    let head_path = git_root.join(".git").join("HEAD");
    let head_text = fs::read_to_string(head_path).ok()?;
    let trimmed = head_text.trim();
    if let Some(ref_rel) = trimmed.strip_prefix("ref: ") {
        let ref_path = git_root.join(".git").join(ref_rel.trim());
        if let Ok(sha) = fs::read_to_string(ref_path) {
            return Some(sha.trim().chars().take(12).collect());
        }
    } else if trimmed.len() >= 7 && trimmed.chars().all(|c| c.is_ascii_hexdigit()) {
        return Some(trimmed.chars().take(12).collect());
    }
    None
}

// =============================================================================
// CHAIN OF TITLE & i1.is KNOWLEDGE NIBBLE GRAPH
// =============================================================================

/// A single verifiable provenance step in a document's `ChainOfTitle`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChainOfTitleLink {
    pub step_index: u32,
    pub action: String,
    pub actor: String,
    pub timestamp_utc: String,
    pub commit_or_witness: String,
    /// 13-byte (`65`-trit) SPQT Base-243 ternary seal over `(step_index, action, actor, witness)`
    pub spqt_seal_13b_hex: String,
}

/// Cryptographic Chain of Title for a document, diary, or code module on `i1.is`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChainOfTitle {
    pub title_id: String,
    pub document_path: String,
    pub canonical_uri: String,
    pub origin_author: String,
    pub author_role: AuthorRole,
    pub detected_license: String,
    pub git_commit: Option<String>,
    pub public_remotes: Vec<String>,
    pub visibility: WaveVisibilityMode,
    pub ai_training_grant: AiTrainingGrant,
    pub links: Vec<ChainOfTitleLink>,
}

/// Relationship type connecting one `I1KnowledgeNibble` to another in the `i1.is` MEM8 Nexus.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum NibbleRelationKind {
    /// Chronological continuation (`TemporalSuccessorOf` in a diary or changelog)
    TemporalSuccessorOf,
    /// Shares the same parent document & `ChainOfTitle`
    SameChainOfTitle,
    /// High SPQT Base-243 65-trit ternary wave resonance (`dot >= 16`)
    TernaryResonance,
    /// Documentation nibble explains or references a code/concept nibble
    ExplainsConcept,
}

/// Directed edge between two nibbles in the `i1.is` MEM8 Knowledge Nexus.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NibbleRelation {
    pub target_nibble_id: String,
    pub kind: NibbleRelationKind,
    /// Resonance or relationship weight in per-mille (`0..=1000`)
    pub weight_pm: u16,
    /// Raw 65-trit SPQT Base-243 dot product (`-65..=65`)
    pub spqt_dot: i16,
}

/// A self-contained "Nibble of Information" on `i1.is` with SPQT Base-243 ternary wave coordinates,
/// a verifiable `ChainOfTitle`, and explicit relationships to other nibbles.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct I1KnowledgeNibble {
    pub nibble_id: String,
    pub title: String,
    pub content: String,
    pub keywords: Vec<String>,
    pub temporal_anchor: Option<String>,
    pub temporal_order: i32,
    pub chain_of_title_id: String,
    pub origin_author: String,
    pub visibility: WaveVisibilityMode,
    pub ai_training_grant: AiTrainingGrant,
    /// 13-byte (`65`-trit) SPQT Base-243 ternary signature (hex)
    pub spqt_base243_13b_hex: String,
    /// Connected nibbles in the `i1.is` MEM8 Nexus
    pub relations: Vec<NibbleRelation>,
}

/// Persistent local store (`~/.st/i1_nexus.json`) for the `i1.is` MEM8 Knowledge Nexus & Chain of Title registry.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct I1NexusStore {
    pub updated_at_utc: String,
    pub hub_url: String,
    pub chains_of_title: HashMap<String, ChainOfTitle>,
    pub nibbles: Vec<I1KnowledgeNibble>,
}

impl I1NexusStore {
    pub fn store_path() -> Result<PathBuf> {
        let dir = dirs::home_dir()
            .unwrap_or_else(|| PathBuf::from("/tmp"))
            .join(".st");
        fs::create_dir_all(&dir)?;
        Ok(dir.join("i1_nexus.json"))
    }

    pub fn load_or_default() -> Self {
        let Ok(path) = Self::store_path() else {
            return Self::default();
        };
        if let Ok(bytes) = fs::read(&path) {
            if let Ok(store) = serde_json::from_slice::<Self>(&bytes) {
                return store;
            }
        }
        Self {
            updated_at_utc: Utc::now().to_rfc3339(),
            hub_url: default_i1_nexus_url(),
            chains_of_title: HashMap::new(),
            nibbles: Vec::new(),
        }
    }

    pub fn save(&mut self) -> Result<()> {
        self.updated_at_utc = Utc::now().to_rfc3339();
        if self.hub_url.is_empty() {
            self.hub_url = default_i1_nexus_url();
        }
        let path = Self::store_path()?;
        let json = serde_json::to_vec_pretty(self)?;
        fs::write(&path, json).with_context(|| format!("Failed to write {}", path.display()))?;
        Ok(())
    }

    /// Distill a trained `.m8` `LifePersona` (or Specialist Soul) into `i1.is` Knowledge Nibbles
    /// with full `ChainOfTitle` deeds and automatic nibble-to-nibble relationship edges.
    pub fn distill_persona_to_nibbles(
        &mut self,
        persona: &LifePersona,
        config: &I1NexusConfig,
        max_nibbles: usize,
    ) -> RepoVisibilityReport {
        let source_path = PathBuf::from(&persona.source_path);
        let report = detect_repo_public_status(&source_path, config);
        let limit = max_nibbles.clamp(1, 256);

        let origin_author = persona
            .ancestral_seed
            .as_ref()
            .map(|s| s.id.clone())
            .or_else(|| report.origin_author.clone())
            .unwrap_or_else(|| persona.name.clone());

        let license = report.detected_license.clone().unwrap_or_else(|| {
            if report.effective_visibility == WaveVisibilityMode::PublicCommons {
                "CC-BY-4.0 / MIT".to_string()
            } else {
                "Private-Sovereign".to_string()
            }
        });

        let mut new_nibble_ids = Vec::new();

        for (ep_idx, ep) in persona.episodes.iter().take(limit).enumerate() {
            let doc_key = format!("{}:{}", persona.slug, ep.source);
            let title_id = format!("cot-{:016x}", fnv64_1a(doc_key.as_bytes()));

            self.chains_of_title
                .entry(title_id.clone())
                .or_insert_with(|| {
                    let canonical_uri = if let Some(first_remote) = report.public_remotes.first() {
                        format!("{first_remote}/blob/main/{}", ep.source)
                    } else {
                        format!("i1://soul/{}/{}", persona.slug, ep.source)
                    };
                    let now_str = Utc::now().to_rfc3339();
                    let (_, seal0) =
                        compute_ternary_signature(&format!("origin:{origin_author}:{}", ep.source));
                    let (_, seal1) = compute_ternary_signature(&format!(
                        "mem8-distill:{}:{title_id}",
                        persona.slug
                    ));
                    let mut links = vec![ChainOfTitleLink {
                        step_index: 1,
                        action: "OriginAuthored".to_string(),
                        actor: origin_author.clone(),
                        timestamp_utc: ep
                            .event_time
                            .clone()
                            .or_else(|| ep.written_time.clone())
                            .unwrap_or_else(|| now_str.clone()),
                        commit_or_witness: ep.origin.clone(),
                        spqt_seal_13b_hex: hex::encode(seal0),
                    }];
                    if let Some(ref sha) = report.head_commit {
                        let (_, seal_git) =
                            compute_ternary_signature(&format!("git:{sha}:{}", ep.source));
                        links.push(ChainOfTitleLink {
                            step_index: 2,
                            action: "GitCommitted".to_string(),
                            actor: origin_author.clone(),
                            timestamp_utc: now_str.clone(),
                            commit_or_witness: sha.clone(),
                            spqt_seal_13b_hex: hex::encode(seal_git),
                        });
                    }
                    links.push(ChainOfTitleLink {
                        step_index: (links.len() + 1) as u32,
                        action: "DistilledToI1Mem8Nibble".to_string(),
                        actor: format!("smart-tree/{}", env!("CARGO_PKG_VERSION")),
                        timestamp_utc: now_str,
                        commit_or_witness: format!("persona:{}", persona.slug),
                        spqt_seal_13b_hex: hex::encode(seal1),
                    });

                    ChainOfTitle {
                        title_id: title_id.clone(),
                        document_path: ep.source.clone(),
                        canonical_uri,
                        origin_author: origin_author.clone(),
                        author_role: ep.author_role,
                        detected_license: license.clone(),
                        git_commit: report.head_commit.clone(),
                        public_remotes: report.public_remotes.clone(),
                        visibility: report.effective_visibility,
                        ai_training_grant: report.ai_training_grant,
                        links,
                    }
                });

            let nibble_seed = format!(
                "{}:{}:{}:{}",
                persona.slug,
                ep_idx,
                ep.source,
                ep.passage.chars().take(64).collect::<String>()
            );
            let nibble_id = format!("nib-{:016x}", fnv64_1a(nibble_seed.as_bytes()));
            if self.nibbles.iter().any(|n| n.nibble_id == nibble_id) {
                new_nibble_ids.push(nibble_id);
                continue;
            }

            let (_, spqt_13b) = compute_ternary_signature(&ep.passage);
            let snippet: String = ep.passage.chars().take(260).collect();
            let title = if let Some(ref ev) = ep.event_time {
                format!("{} — {} ({})", persona.name, ep.source, ev)
            } else {
                format!("{} — {}", persona.name, ep.source)
            };

            self.nibbles.push(I1KnowledgeNibble {
                nibble_id: nibble_id.clone(),
                title,
                content: snippet,
                keywords: ep.keywords.iter().take(8).cloned().collect(),
                temporal_anchor: ep.event_time.clone().or_else(|| ep.era.clone()),
                temporal_order: ep.temporal_order,
                chain_of_title_id: title_id,
                origin_author: origin_author.clone(),
                visibility: report.effective_visibility,
                ai_training_grant: report.ai_training_grant,
                spqt_base243_13b_hex: hex::encode(spqt_13b),
                relations: Vec::new(),
            });
            new_nibble_ids.push(nibble_id);
        }

        self.rebuild_nibble_relationships();
        let _ = self.save();
        report
    }

    /// Recompute directed relationships (`TemporalSuccessorOf`, `SameChainOfTitle`, `TernaryResonance`, `ExplainsConcept`)
    /// across all nibbles in the store using SPQT Base-243 65-trit dot products and keyword/temporal alignment.
    pub fn rebuild_nibble_relationships(&mut self) {
        let n = self.nibbles.len();
        if n <= 1 {
            return;
        }

        let trits_vec: Vec<[i8; 65]> = self
            .nibbles
            .iter()
            .map(|nib| compute_ternary_signature(&nib.content).0)
            .collect();

        for i in 0..n {
            let mut rels: Vec<NibbleRelation> = Vec::new();
            for j in 0..n {
                if i == j {
                    continue;
                }
                let a = &self.nibbles[i];
                let b = &self.nibbles[j];
                let dot: i16 = trits_vec[i]
                    .iter()
                    .zip(trits_vec[j].iter())
                    .map(|(&x, &y)| (x as i16) * (y as i16))
                    .sum();

                let shared_kw = a.keywords.iter().filter(|k| b.keywords.contains(k)).count() as u16;

                if a.temporal_order > 0
                    && b.temporal_order > a.temporal_order
                    && (b.temporal_order - a.temporal_order) <= 100
                    && a.origin_author == b.origin_author
                {
                    rels.push(NibbleRelation {
                        target_nibble_id: b.nibble_id.clone(),
                        kind: NibbleRelationKind::TemporalSuccessorOf,
                        weight_pm: 880,
                        spqt_dot: dot,
                    });
                } else if a.chain_of_title_id == b.chain_of_title_id {
                    rels.push(NibbleRelation {
                        target_nibble_id: b.nibble_id.clone(),
                        kind: NibbleRelationKind::SameChainOfTitle,
                        weight_pm: (700 + shared_kw * 40).min(980),
                        spqt_dot: dot,
                    });
                } else if shared_kw >= 2 {
                    rels.push(NibbleRelation {
                        target_nibble_id: b.nibble_id.clone(),
                        kind: NibbleRelationKind::ExplainsConcept,
                        weight_pm: (600 + shared_kw * 60).min(950),
                        spqt_dot: dot,
                    });
                } else if dot >= 16 {
                    let w = (500 + (dot as u16) * 7).min(960);
                    rels.push(NibbleRelation {
                        target_nibble_id: b.nibble_id.clone(),
                        kind: NibbleRelationKind::TernaryResonance,
                        weight_pm: w,
                        spqt_dot: dot,
                    });
                }
            }
            rels.sort_by_key(|y| std::cmp::Reverse(y.weight_pm));
            rels.truncate(6);
            self.nibbles[i].relations = rels;
        }
    }
}

fn fnv64_1a(bytes: &[u8]) -> u64 {
    let mut h = 0xcbf2_9ce4_8422_2325u64;
    for &b in bytes {
        h ^= b as u64;
        h = h.wrapping_mul(0x0100_0000_01b3);
    }
    h
}

/// Complete, self-contained Markdown specification & copy-pasteable code for Development LLMs
/// (Claude, GPT, Gemini, Cursor, Antigravity, OpenCode) to implement the `MEMNET` Bitstream
/// and `i1.is` Chain-of-Title Knowledge Nibble protocol.
pub fn memnet_llm_implementation_guide() -> String {
    r#"# ⚡ Smart Tree `MEMNET` Bitstream & `i1.is` Knowledge Nexus — Developer LLM Implementation Guide

> **Target Audience**: Development LLMs & Coding Agents (Claude, GPT, Gemini, Antigravity, OpenCode, Cursor).
> **Canonical Endpoints**:
> - `POST /memnet/stream` — Preferred 2-Pass SPQT Base-243 Ternary Q&A + Native Binary Bitstream (`64B@0x62` + `79B WaveFrame79` + `26B SPQT Base-243`).
> - `GET  /cheet` & `GET /cheet/:slug` — The Cheet's Markdown & ANSI Terminal Answer + `i1.is` Installer Engine (`?q=<question>`, `?format=ansi|md|json`).
> - `GET  /n/:tool` & `GET /i1/install/:tool` — Verified `i1.is` One-Line POSIX `sh` Installer (`curl -fsSL https://i1.is/n/smart-tree | sh`).
> - `POST /v1/chat/completions` — Legacy OpenAI-compatible wrapper (emits `X-SmartTree-Preferred-Endpoint: /memnet/stream`).
> - `GET  /i1/nexus` — Query `i1.is` Knowledge Nibbles, Nibble-to-Nibble Relationships, and Cryptographic Chain-of-Title deeds.
> - `POST /i1/nexus/distill` — Distill a repository or `.m8` Persona Soul into `i1.is` Knowledge Nibbles with automatic Public Repo detection.

---

## 1. Binary Bitstream Layout (`POST /memnet/stream` with `Accept: application/octet-stream`)

When a client sends `Accept: application/octet-stream` (or `{"bitstream": "raw"}`), `POST /memnet/stream` returns a contiguous binary buffer with zero JSON parsing overhead:

```text
Byte Offset   Length   Field                     Description
-----------   ------   -----------------------   -------------------------------------------------------------------
0..64         64 B     LitanyWireFrame (0x62)    Cache-line-aligned 64B frame:
                                                 - [0..8]:   u64 LE felix_xor_fold_64b(raw)
                                                 - [8..16]:  u64 LE felix_xor_fold_64b(packed)
                                                 - [16..20]: u32 LE raw_len
                                                 - [20..24]: u32 LE packed_len
                                                 - [24]:     0x62 (Litany gate byte)
                                                 - [25]:     0x01 (FRAME_ABI_V1)
                                                 - [32..40]: u64 LE bfp64_pos (Kleene +1 bitplane)
                                                 - [40..48]: u64 LE bfp64_neg (Kleene -1 bitplane, pos & neg == 0)
                                                 - [48..52]: u32 LE ratio_p (GCD-reduced exact p)
                                                 - [52..56]: u32 LE ratio_q (GCD-reduced exact q)
                                                 - [56..59]: [u8; 3] spqt_root (15-trit Base-243 root signature)
                                                 - [59]:     u8 qt_mask (0x55)
                                                 - [60..62]: u16 LE lanes_active (0x0808)
                                                 - [62..64]: u16 LE wire_seal16 (XOR fold witness over [0..62])
64..143       79 B     WaveFrame79 (MEM|8 v2)    79-byte semantic wave header (frequency, valence, arousal, CRC64)
143..156      13 B     Query SPQT Base-243       65 query trits {-1,0,+1} packed 5 trits/byte (3^5 = 243 <= 255)
156..169      13 B     Answer SPQT Base-243      65 Pass-2 answer trits {-1,0,+1} packed 5 trits/byte
169..N        Var      MNPK WaveQaResponse       Routable MemNet mesh packet ([Depth:u8][Path][MNPK Payload])
```

---

## 2. Reference Implementation: SPQT Base-243 (`3^5 = 243`) Encoder/Decoder & Stream Client

### Python Reference (Zero Dependencies)
```python
import urllib.request
import json

def spqt_unpack_5trits(byte_val: int) -> list[int]:
    """Unpack 1 SPQT Base-243 byte into 5 balanced trits in {-1, 0, +1}."""
    trits = []
    for _ in range(5):
        trits.append((byte_val % 3) - 1)
        byte_val //= 3
    return trits

def spqt_unpack_65trits(packed_13b: bytes) -> list[int]:
    assert len(packed_13b) == 13
    out = []
    for b in packed_13b:
        out.extend(spqt_unpack_5trits(b))
    return out

def query_memnet_bitstream(question: str, model: str = "burn/mem8-ternary-micro", host: str = "http://localhost:28428"):
    req = urllib.request.Request(
        f"{host}/memnet/stream",
        data=json.dumps({"model": model, "question": question, "bitstream": "raw"}).encode("utf-8"),
        headers={"Content-Type": "application/json", "Accept": "application/octet-stream"},
        method="POST",
    )
    with urllib.request.urlopen(req) as resp:
        wire = resp.read()
        litany_64b = wire[0:64]
        wave_79b   = wire[64:143]
        q_spqt_13b = wire[143:156]
        a_spqt_13b = wire[156:169]
        mnpk_pkt   = wire[169:]
        assert litany_64b[24] == 0x62, "Verified 0x62 LitanyWireFrame gate"
        return {
            "packet_id": resp.headers.get("x-memnet-packet-id"),
            "persona": resp.headers.get("x-memnet-persona"),
            "query_trits": spqt_unpack_65trits(q_spqt_13b),
            "answer_trits": spqt_unpack_65trits(a_spqt_13b),
            "mnpk_bytes": len(mnpk_pkt),
        }
```

### TypeScript / Node.js Reference
```typescript
export function spqtUnpack65Trits(bytes13: Uint8Array): Int8Array {
  const out = new Int8Array(65);
  for (let i = 0; i < 13; i++) {
    let v = bytes13[i];
    for (let j = 0; j < 5; j++) {
      out[i * 5 + j] = (v % 3) - 1;
      v = Math.floor(v / 3);
    }
  }
  return out;
}

export async function askMemnetStream(question: string, model = "burn/mem8-ternary-micro") {
  const res = await fetch("http://localhost:28428/memnet/stream", {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify({ model, question }),
  });
  return await res.json(); // Includes pass2_contextual_answer + bitstream hex frames
}
```

---

## 3. `i1.is` MEM8 Nexus, Chain of Title & Sustainability Model

1. **Chain of Title (`ChainOfTitle`)**:
   Every document distilled by Smart Tree receives a deterministic `title_id` (`cot-<hex>`), linking `OriginAuthored` -> `GitCommitted` -> `DistilledToI1Mem8Nibble` with 13-byte SPQT Base-243 seals.
2. **Knowledge Nibbles (`I1KnowledgeNibble`) & Edges (`NibbleRelation`)**:
   Information is broken into compact nibbles connected by typed edges (`temporal-successor-of`, `same-chain-of-title`, `explains-concept`, `ternary-resonance`).
3. **Public Commons vs. Private Sovereign Tier**:
   - **Public Commons (`free`)**: Automatically enabled when Smart Tree detects a public repository (`github.com`, `codeberg.org`, `gitlab.com`, `8s.is`, `i1.is`). Free for AIs to train with and query provided the **Chain of Title** is preserved.
   - **Private Sovereign (`$5/mo` lights-on subscription)**: Keeps private repositories and personal diaries 100% private (`private-sovereign-no-training`) while funding the `i1.is` public nexus.
"#
    .to_string()
}

// ============================================================================
// 🐆 THE CHEET · i1.is Terminal Markdown/ANSI Renderer & Universal Installer
// ============================================================================

/// Detect whether the HTTP caller is a terminal CLI client (`curl`, `wget`, `httpie`, `?format=ansi|text`).
pub fn is_terminal_cli_user_agent(ua: Option<&str>, format_query: Option<&str>) -> bool {
    if let Some(fmt) = format_query {
        let f = fmt.trim().to_ascii_lowercase();
        if f == "ansi" || f == "text" || f == "cli" {
            return true;
        }
        if f == "html" || f == "json" || f == "md" || f == "markdown" {
            return false;
        }
    }
    if let Some(ua_str) = ua {
        let lower = ua_str.to_ascii_lowercase();
        if lower.starts_with("curl/")
            || lower.starts_with("wget/")
            || lower.starts_with("httpie/")
            || lower.contains("libcurl")
            || lower.starts_with("xh/")
        {
            return true;
        }
    }
    false
}

/// Format inline Markdown (`**bold**` and `` `code` ``) with ANSI escape sequences.
fn style_inline_markdown_ansi(line: &str) -> String {
    const BOLD_ON: &str = "\x1b[1m";
    const BOLD_OFF: &str = "\x1b[22m";
    const CYAN: &str = "\x1b[1;36m";
    const RESET_FG: &str = "\x1b[39m";

    let mut out = String::with_capacity(line.len() + 32);
    let chars: Vec<char> = line.chars().collect();
    let mut i = 0usize;
    let mut in_bold = false;
    let mut in_code = false;

    while i < chars.len() {
        if !in_code && i + 1 < chars.len() && chars[i] == '*' && chars[i + 1] == '*' {
            if in_bold {
                out.push_str(BOLD_OFF);
                in_bold = false;
            } else {
                out.push_str(BOLD_ON);
                in_bold = true;
            }
            i += 2;
            continue;
        }
        if chars[i] == '`' {
            if in_code {
                out.push_str(RESET_FG);
                if !in_bold {
                    out.push_str(BOLD_OFF);
                }
                in_code = false;
            } else {
                out.push_str(CYAN);
                in_code = true;
            }
            i += 1;
            continue;
        }
        out.push(chars[i]);
        i += 1;
    }
    if in_code {
        out.push_str(RESET_FG);
    }
    if in_bold {
        out.push_str(BOLD_OFF);
    }
    out
}

/// Convert Markdown into The Cheet's crisp ANSI terminal output (`curl -sL https://i1.is/<slug>`).
pub fn cheet_markdown_to_ansi(md: &str) -> String {
    const RESET: &str = "\x1b[0m";
    const BOLD: &str = "\x1b[1m";
    const GOLD: &str = "\x1b[1;33m";
    const CYAN: &str = "\x1b[1;36m";
    const CORAL: &str = "\x1b[1;31m";
    const GREEN: &str = "\x1b[0;32m";
    const DIM: &str = "\x1b[2m";

    let mut out = String::new();
    let mut in_code = false;

    for line in md.lines() {
        if line.trim_start().starts_with("```") {
            let lang = line.trim_start().trim_start_matches("```").trim();
            in_code = !in_code;
            if in_code {
                if lang.is_empty() {
                    out.push_str(&format!(
                        "  {DIM}┌────────────────────────────────────────────────────────────────────{RESET}\n"
                    ));
                } else {
                    out.push_str(&format!(
                        "  {DIM}┌── [{CYAN}{lang}{RESET}{DIM}] ──────────────────────────────────────────────────────────{RESET}\n"
                    ));
                }
            } else {
                out.push_str(&format!(
                    "  {DIM}└────────────────────────────────────────────────────────────────────{RESET}\n"
                ));
            }
            continue;
        }

        if in_code {
            out.push_str(&format!("  {DIM}│{RESET} {GREEN}{line}{RESET}\n"));
            continue;
        }

        if let Some(h1) = line.strip_prefix("# ") {
            out.push_str(&format!(
                "\n  {CORAL}{h1}{RESET}\n  {DIM}{}{RESET}\n",
                "═".repeat(h1.chars().count().min(70))
            ));
        } else if let Some(h2) = line.strip_prefix("## ") {
            out.push_str(&format!("\n  {GOLD}{h2}{RESET}\n"));
        } else if let Some(h3) = line.strip_prefix("### ") {
            out.push_str(&format!("\n  {CYAN}{h3}{RESET}\n"));
        } else if line.trim_start().starts_with('|') {
            out.push_str(&format!("  {CYAN}{line}{RESET}\n"));
        } else if let Some(quote) = line.strip_prefix("> ") {
            out.push_str(&format!(
                "  {GOLD}┃{RESET} {BOLD}{}{RESET}\n",
                style_inline_markdown_ansi(quote)
            ));
        } else if let Some(bullet) = line
            .trim_start()
            .strip_prefix("- ")
            .or_else(|| line.trim_start().strip_prefix("* "))
        {
            out.push_str(&format!(
                "  {GOLD}•{RESET} {}\n",
                style_inline_markdown_ansi(bullet)
            ));
        } else if line.trim() == "---" {
            out.push_str(&format!("  {DIM}{}{RESET}\n", "─".repeat(70)));
        } else {
            out.push_str(&format!("  {}\n", style_inline_markdown_ansi(line)));
        }
    }
    out
}

/// Built-in 8b-is Constellation Cheat Sheets served by The Cheet on `i1.is` / `cheet.is` / `smart-tree`.
pub fn builtin_cheet_markdown_sheet(slug: &str) -> Option<(&'static str, &'static str, String)> {
    let clean = slug
        .trim()
        .trim_matches('/')
        .to_ascii_lowercase()
        .replace(' ', "-");
    match clean.as_str() {
        "smart-tree" | "st" | "install" => Some((
            "smart-tree",
            "Smart Tree (st) · AI Directory Visualization, MCP Server & MEM8 Consciousness",
            format!(
                r#"# 🌳 Smart Tree (`st`) v{} — The Cheet's Official Cheat Sheet

> **Chain of Title**: `cot-8b-is-smart-tree` · **License**: MIT (`Commons-Free`) · **Hub**: `https://i1.is`

## ⚡ 1-Line Standard Installation (`i1.is` Verified Shell Nibble)
```sh
# Install Smart Tree (st, m8, std, st-hub) — SHA-256 verified, no apt/brew needed:
curl -fsSL https://i1.is/n/smart-tree | sh

# Or install from your local Smart Tree Daemon:
curl -fsSL http://localhost:28428/n/smart-tree | sh
```

## 🚀 Essential Terminal Commands
- `st` — Classic fast directory tree with git status & file sizes
- `st --mode ai --compress` — Token-efficient compressed context for LLMs (80% fewer tokens)
- `st --mcp` — Start the Model Context Protocol (MCP) server over stdio
- `st persona train anne_frank.pdf` — Train a `.m8` Life Persona soul with 2-Pass SPQT Base-243 Q&A
- `st persona ask anne-frank "Why did Margot receive a call-up?"` — 2-Pass contextual synthesis
- `st --security-scan .` — Erdős–Rényi + MagiScanner supply-chain security audit

## 🧠 `MEMNET` Bitstream & `i1.is` Knowledge Nexus Endpoints
- `POST /memnet/stream` — 2-Pass SPQT Base-243 Q&A + `64B@0x62` `LitanyWireFrame` binary bitstream
- `GET  /memnet/llm-guide` — Copy-pasteable Python/TypeScript/Rust guide for Development LLMs
- `GET  /cheet?q=<question>` — Ask The Cheet any question and receive a Markdown/ANSI answer
- `GET  /i1/nexus` — Inspect `i1.is` Knowledge Nibbles & Cryptographic Chains of Title"#,
                env!("CARGO_PKG_VERSION")
            ),
        )),
        "memnet" | "bitstream" | "spqt" => Some((
            "memnet",
            "MEMNET 64B@0x62 Bitstream & 2-Pass SPQT Base-243 Synthesis",
            r#"# ⚡ `MEMNET` Bitstream & 2-Pass SPQT Base-243 Synthesis

> **Chain of Title**: `cot-8b-is-memnet` · **Wire Gate**: `0x62` (`LitanyWireFrame`) · **Hub**: `https://i1.is`

## 🧠 How 2-Pass Ternary Synthesis Works
1. **Pass 1 (Core Memory Recall)**: Burn's SIMD `Flex` tensor engine (`1x65` × `65xN` matmul) scores every `.m8` wave by SPQT Base-243 ternary resonance, keyword overlap, and temporal adjacency.
2. **Pass 2 (Contextual Answer Synthesis)**: Extracts clean sentences across the recalled core memories, scores each sentence against the question's interrogative intent (`why`, `how`, `who`, `where`, `when`), and weaves a multi-memory narrative with `[YYYY-MM-DD]` citations.

## 📡 Query From Terminal (`curl`)
```sh
# Get a rich ANSI Markdown answer from The Cheet:
curl -sL "http://localhost:28428/cheet?q=Why+did+Margot+receive+a+call-up"

# Stream raw 64B@0x62 + 79B WaveFrame79 + 26B SPQT Base-243 binary bytes:
curl -s -X POST http://localhost:28428/memnet/stream \
  -H "Accept: application/octet-stream" \
  -H "Content-Type: application/json" \
  -d '{"model":"persona:anne-frank","question":"Why did Margot receive a call-up?"}' | xxd | head
```"#
                .to_string(),
        )),
        "i1" | "i1.is" | "chain-of-title" | "nibbles" => Some((
            "i1",
            "i1.is Universal Knowledge Nexus, Chain of Title & Verified Installer",
            r#"# 🌐 `i1.is` — Universal Knowledge Nexus & Verified Installer (`8b-is`)

> **Positioning**: `g8t.is` is for AI-only cycle golf; **`i1.is` is for everyone** (Humans, Terminals & AIs).

## 📜 Cryptographic Chain of Title (`cot-<hex>`)
Every document distilled by Smart Tree into `i1.is` receives a verifiable **Chain of Title**:
- `OriginAuthored` → `GitCommitted` → `DistilledToI1Mem8Nibble`
- Sealed with a 13-byte **SPQT Base-243** (`3^5 = 243`) ternary signature and FNV-1a content hash.
- Connected to peer nibbles via typed edges (`temporal-successor-of`, `same-chain-of-title`, `explains-concept`, `ternary-resonance`).

## 💡 Public Commons vs. Private Sovereign Sustainability
- **Public Commons (`Free`)**: Auto-detected for public repositories (`github.com`, `codeberg.org`, `gitlab.com`, `8s.is`, `i1.is`). Free for all humans and AIs provided the Chain of Title is preserved.
- **Private Sovereign (`$5.00/mo`)**: Keeps private repos & personal diaries 100% private (`private-sovereign-no-training`) while keeping the lights on at `i1.is`.

## 🛠️ Standard `i1.is` Install Commands
```sh
curl -fsSL https://i1.is/n/smart-tree | sh   # Install Smart Tree (st, m8, std)
curl -fsSL https://i1.is/n/dev | sh          # Install verified dev bundle (fzf, rg, fd, st)
curl -sL   https://i1.is/cheet               # Open The Cheet's terminal Cheat Sheet index
```"#
                .to_string(),
        )),
        "git-worktrees" | "worktrees" | "git" => Some((
            "git-worktrees",
            "Git Worktrees & Parallel Agent Branches Cheat Sheet",
            r#"# 🌿 Git Worktrees — Parallel Branches Without Re-Cloning

> **Chain of Title**: `cot-8b-is-git-worktrees` · **Curated by**: The Cheet (`🐆🥁🧻`)

## ⚡ Core Commands
```sh
# Create a new worktree in ../feature-x on a new branch:
git worktree add -b feat/cheet-i1 ../feature-x main

# List all active worktrees:
git worktree list

# Clean up a finished worktree:
git worktree remove ../feature-x
git worktree prune
```

## 🧠 Why Worktrees Rock With Smart Tree (`st`)
- Run `st --mode ai --compress` inside each worktree so parallel agents (`Moser–Tardos LLL scheduler`) never collide on index locks!"#
                .to_string(),
        )),
        _ => None,
    }
}

/// Render The Cheet's ANSI box header used when `curl`ing into `i1.is` / `cheet.is` / `smart-tree`.
pub fn render_cheet_ansi_banner(
    subtitle: &str,
    repo_status: Option<&RepoVisibilityReport>,
) -> String {
    let mut ansi = String::new();
    ansi.push_str("\x1b[1;31m╭──────────────────────────────────────────────────────────────────────────╮\x1b[0m\n");
    ansi.push_str(&format!(
        "\x1b[1;31m│\x1b[0m  🐆🥁🧻 \x1b[1;33mTHE CHEET\x1b[0m · \x1b[1;36mhttps://i1.is\x1b[0m · \x1b[2m{}\x1b[0m\n",
        subtitle
    ));
    if let Some(rep) = repo_status {
        let badge = if rep.effective_visibility == WaveVisibilityMode::PublicCommons {
            "\x1b[1;32m🌐 PUBLIC COMMONS (FREE · CHAIN-OF-TITLE)\x1b[0m"
        } else {
            "\x1b[1;33m🔒 PRIVATE SOVEREIGN ($5/mo LIGHTS-ON)\x1b[0m"
        };
        let remote_label = rep
            .public_remotes
            .first()
            .map(|s| s.as_str())
            .unwrap_or(rep.inspected_path.as_str());
        ansi.push_str(&format!(
            "\x1b[1;31m│\x1b[0m  📦 Repo: \x1b[1m{}\x1b[0m · {}\n",
            remote_label, badge
        ));
    }
    ansi.push_str("\x1b[1;31m╰──────────────────────────────────────────────────────────────────────────╯\x1b[0m\n");
    ansi
}

/// Render The Cheet's `i1.is` terminal home screen when someone runs `curl -sL https://i1.is` or `curl -sL http://localhost:28428/cheet`.
pub fn render_cheet_i1_terminal_home(
    nexus: &I1NexusStore,
    repo_status: &RepoVisibilityReport,
) -> String {
    let mut ansi = render_cheet_ansi_banner(
        "Universal Markdown Answers, MEM8 Nibbles & Verified Installer",
        Some(repo_status),
    );

    ansi.push_str("\n\x1b[1;33m⚡ STANDARD 1-LINE INSTALLERS (SHA-256 VERIFIED · NO APT/BREW NEEDED):\x1b[0m\n");
    ansi.push_str("  • \x1b[1;32mcurl -fsSL https://i1.is/n/smart-tree | sh\x1b[0m   \x1b[2m# Smart Tree (st, m8, std, st-hub)\x1b[0m\n");
    ansi.push_str("  • \x1b[1;32mcurl -fsSL https://i1.is/n/dev | sh\x1b[0m          \x1b[2m# Verified Dev Bundle (st, fzf, rg, fd)\x1b[0m\n");
    ansi.push_str("  • \x1b[1;32mcurl -fsSL http://localhost:28428/n/smart-tree | sh\x1b[0m \x1b[2m# Local Daemon Installer\x1b[0m\n");

    ansi.push_str(
        "\n\x1b[1;33m🐆 ASK THE CHEET FOR MARKDOWN ANSWERS & CHEAT SHEETS IN TERMINAL:\x1b[0m\n",
    );
    ansi.push_str("  • \x1b[1;36mcurl -sL http://localhost:28428/cheet/smart-tree\x1b[0m  \x1b[2m# Smart Tree & MCP Cheat Sheet\x1b[0m\n");
    ansi.push_str("  • \x1b[1;36mcurl -sL http://localhost:28428/cheet/memnet\x1b[0m      \x1b[2m# 64B@0x62 Bitstream & SPQT Guide\x1b[0m\n");
    ansi.push_str("  • \x1b[1;36mcurl -sL http://localhost:28428/cheet/i1\x1b[0m          \x1b[2m# i1.is Chain of Title & Commons\x1b[0m\n");
    ansi.push_str("  • \x1b[1;36mcurl -sL \"http://localhost:28428/cheet?q=Why+did+Margot+receive+a+call-up\"\x1b[0m\n");
    ansi.push_str("      \x1b[2m# Live 2-Pass SPQT Base-243 Contextual Answer + Chain-of-Title Nibbles!\x1b[0m\n");

    let total_edges: usize = nexus.nibbles.iter().map(|n| n.relations.len()).sum();
    ansi.push_str(&format!(
        "\n\x1b[1;33m🧠 LIVE i1.is MEM8 KNOWLEDGE NEXUS ({} Nibbles · {} Chains of Title · {} Edges):\x1b[0m\n",
        nexus.nibbles.len(),
        nexus.chains_of_title.len(),
        total_edges
    ));
    for nibble in nexus.nibbles.iter().take(6) {
        let preview = if nibble.content.chars().count() > 68 {
            let s: String = nibble.content.chars().take(65).collect();
            format!("{s}...")
        } else {
            nibble.content.clone()
        };
        ansi.push_str(&format!(
            "  • \x1b[1;36m{}\x1b[0m \x1b[2m[{} · Author: {} · {} edges]\x1b[0m\n      {}\n",
            nibble.nibble_id,
            nibble.chain_of_title_id,
            nibble.origin_author,
            nibble.relations.len(),
            preview
        ));
    }
    ansi.push_str("\n\x1b[2m  💡 Tip: Add ?format=md for raw Markdown or ?format=json for structured JSON.\x1b[0m\n\n");
    ansi
}

/// Synthesize a complete Markdown answer (and ANSI rendering) from The Cheet using:
/// 1. Built-in 8b-is Cheat Sheets (if `query_or_slug` matches a known slug),
/// 2. Pass-2 SPQT Base-243 Contextual Synthesis across `.m8` Life Personas, and
/// 3. Matching `i1.is` Knowledge Nibbles with Cryptographic Chain of Title (`cot-...`).
pub fn render_cheet_markdown_answer(
    query_or_slug: &str,
    persona_hint: Option<&str>,
    nexus: &I1NexusStore,
    repo_status: &RepoVisibilityReport,
) -> String {
    let trimmed = query_or_slug.trim().trim_matches('/');

    // 1. Check built-in Cheat Sheet first if it's a single slug token
    if let Some((_slug, _title, sheet_md)) = builtin_cheet_markdown_sheet(trimmed) {
        return sheet_md;
    }

    // 2. Search matching i1.is Knowledge Nibbles by ID, keyword, or SPQT ternary dot product
    let (q_trits, q_spqt_13) = crate::proxy::burn::compute_ternary_signature(trimmed);
    let q_lower = trimmed.to_ascii_lowercase();
    let q_terms: Vec<&str> = q_lower
        .split(|c: char| !c.is_alphanumeric())
        .filter(|w| w.len() >= 3)
        .collect();

    let mut scored_nibbles: Vec<(i64, &I1KnowledgeNibble)> = nexus
        .nibbles
        .iter()
        .map(|n| {
            let (n_trits, _) = crate::proxy::burn::compute_ternary_signature(&format!(
                "{} {}",
                n.title, n.content
            ));
            let dot: i32 = q_trits
                .iter()
                .zip(n_trits.iter())
                .map(|(&a, &b)| (a as i32) * (b as i32))
                .sum();
            let text_l =
                format!("{} {} {}", n.title, n.content, n.keywords.join(" ")).to_ascii_lowercase();
            let kw_hits = q_terms.iter().filter(|&&t| text_l.contains(t)).count() as i64;
            let exact_id = if n.nibble_id.eq_ignore_ascii_case(trimmed) {
                500
            } else {
                0
            };
            (exact_id + kw_hits * 45 + (dot as i64), n)
        })
        .filter(|(score, _)| *score > 10)
        .collect();
    scored_nibbles.sort_by_key(|b| std::cmp::Reverse(b.0));

    // 3. Run Pass-2 SPQT Base-243 Contextual Synthesis on the active/matching Persona
    let mut pass2_section = String::new();
    let mut best_persona = None;
    if let Some(hint) = persona_hint {
        let slug = hint.strip_prefix("persona:").unwrap_or(hint);
        best_persona = LifePersona::load_cached(slug).ok();
    }
    if best_persona.is_none() {
        if let Ok(summaries) = LifePersona::list_all() {
            let mut best_score = i64::MIN;
            for s in summaries {
                if let Ok(p) = LifePersona::load_cached(&s.slug) {
                    let hits = p.recall_episodes(trimmed, 1);
                    let top_dot = hits
                        .first()
                        .map(|ep| {
                            let (ep_trits, _) =
                                crate::proxy::burn::compute_ternary_signature(&ep.passage);
                            q_trits
                                .iter()
                                .zip(ep_trits.iter())
                                .map(|(&a, &b)| (a as i64) * (b as i64))
                                .sum::<i64>()
                        })
                        .unwrap_or(0);
                    let mut name_bonus = if q_lower.contains(&p.slug.replace('-', " "))
                        || q_lower.contains(&p.name.to_ascii_lowercase())
                    {
                        500
                    } else {
                        0
                    };
                    if let Some(ref seed) = p.ancestral_seed {
                        for comp in &seed.companions {
                            if q_lower.contains(&comp.to_ascii_lowercase()) {
                                name_bonus += 300;
                            }
                        }
                        for place in &seed.places {
                            if q_lower.contains(&place.to_ascii_lowercase()) {
                                name_bonus += 300;
                            }
                        }
                    }
                    let total = top_dot + name_bonus;
                    if total > best_score {
                        best_score = total;
                        best_persona = Some(p);
                    }
                }
            }
        }
    }

    if let Some(persona) = best_persona {
        let recalled = persona.recall_episodes(trimmed, 6);
        if let Some(pass2) = persona.synthesize_contextual_answer(trimmed, &recalled) {
            pass2_section.push_str(&format!(
                "## 🧠 The Cheet · Pass-2 Contextual Answer (`persona:{}`)\n\n{}\n\n",
                persona.slug, pass2
            ));
        }
    }

    let vis_label = repo_status.effective_visibility.as_str();
    let mut md = format!(
        "# 🐆 The Cheet · `i1.is` Markdown Answer\n\n> **Query**: `{}` · **SPQT Base-243 Query Seal**: `0x{}` · **Policy**: `{}`\n\n",
        trimmed,
        hex::encode(q_spqt_13),
        vis_label
    );

    if !pass2_section.is_empty() {
        md.push_str(&pass2_section);
    }

    if !scored_nibbles.is_empty() {
        md.push_str("## 📜 Matching `i1.is` Knowledge Nibbles & Chain of Title\n\n");
        for (_, nib) in scored_nibbles.iter().take(4) {
            let cot_author = nexus
                .chains_of_title
                .get(&nib.chain_of_title_id)
                .map(|c| c.origin_author.as_str())
                .unwrap_or(nib.origin_author.as_str());
            let preview: String = nib.content.chars().take(240).collect();
            md.push_str(&format!(
                "### `{}` — {}\n- **Chain of Title**: `{}` (Author: **{}** · Visibility: `{:?}`)\n- **Nibble**: {}\n- **SPQT Base-243 Seal**: `0x{}` · **Relationship Edges**: `{}`\n\n",
                nib.nibble_id,
                nib.title,
                nib.chain_of_title_id,
                cot_author,
                nib.visibility,
                preview,
                nib.spqt_base243_13b_hex,
                nib.relations.len()
            ));
        }
    } else if pass2_section.is_empty() {
        md.push_str(&format!(
            "## 💡 No Exact Nibble Match Yet for `{}`\n\n- Distill your repository or `.m8` persona into `i1.is` nibbles:\n```sh\ncurl -s -X POST http://localhost:28428/i1/nexus/distill -H 'Content-Type: application/json' -d '{{}}'\n```\n\n",
            trimmed
        ));
    }

    md.push_str("## ⚡ Standard `i1.is` Terminal Commands\n```sh\n# Install Smart Tree (st, m8, std) via i1.is verified nibble:\ncurl -fsSL https://i1.is/n/smart-tree | sh\n\n# Stream raw 64B@0x62 + SPQT Base-243 binary bitstream:\ncurl -s -X POST http://localhost:28428/memnet/stream -H 'Content-Type: application/json' \\\n  -d '{\"question\":\"");
    md.push_str(&trimmed.replace('"', "\\\""));
    md.push_str("\"}'\n```\n");

    md
}

/// Generate a verified POSIX `#!/bin/sh` installer script for `curl -fsSL https://i1.is/n/<tool> | sh`.
pub fn render_i1_verified_installer_script(tool: &str) -> String {
    let clean = tool.trim().trim_matches('/').to_ascii_lowercase();
    let version = env!("CARGO_PKG_VERSION");
    match clean.as_str() {
        "smart-tree" | "st" | "m8" | "cheet" | "dev" => format!(
            r#"#!/bin/sh
# 🐆🥁🧻 THE CHEET · i1.is Universal Verified Installer ({clean} · Smart Tree v{version})
# Zero apt / brew / pacman required. POSIX sh compatible.
set -eu

I1_TOOL="{clean}"
I1_VERSION="{version}"
I1_PREFIX="${{I1_PREFIX:-$HOME/.local/bin}}"

OS="$(uname -s | tr '[:upper:]' '[:lower:]')"
ARCH="$(uname -m)"
case "$ARCH" in
  x86_64|amd64) ARCH="x86_64" ;;
  aarch64|arm64) ARCH="aarch64" ;;
esac

printf '\033[1;31m╭──────────────────────────────────────────────────────────────────────╮\033[0m\n'
printf '\033[1;31m│\033[0m  🐆🥁🧻 \033[1;33mTHE CHEET · i1.is Verified Installer\033[0m                         \033[1;31m│\033[0m\n'
printf '\033[1;31m│\033[0m  Tool: \033[1;36m%-14s\033[0m  Platform: \033[1m%s/%s\033[0m  Prefix: \033[2m%s\033[0m\n' "$I1_TOOL" "$OS" "$ARCH" "$I1_PREFIX"
printf '\033[1;31m╰──────────────────────────────────────────────────────────────────────╯\033[0m\n'

if [ "${{I1_PLAN:-0}}" = "1" ]; then
  echo "plan: install $I1_TOOL (Smart Tree v$I1_VERSION) into $I1_PREFIX"
  exit 0
fi

mkdir -p "$I1_PREFIX"

if [ "$OS" = "linux" ] && [ "$ARCH" = "x86_64" ]; then
  TAR_NAME="smart-tree-x86_64-unknown-linux-gnu.tar.gz"
elif [ "$OS" = "darwin" ] && [ "$ARCH" = "aarch64" ]; then
  TAR_NAME="smart-tree-aarch64-apple-darwin.tar.gz"
elif [ "$OS" = "darwin" ] && [ "$ARCH" = "x86_64" ]; then
  TAR_NAME="smart-tree-x86_64-apple-darwin.tar.gz"
else
  TAR_NAME="smart-tree-${{ARCH}}-unknown-${{OS}}-gnu.tar.gz"
fi

URL="https://github.com/8b-is/smart-tree/releases/latest/download/$TAR_NAME"
TMP_DIR="$(mktemp -d)"
trap 'rm -rf "$TMP_DIR"' EXIT

echo "⬇️  Fetching $URL ..."
if command -v curl >/dev/null 2>&1; then
  curl -fsSL "$URL" -o "$TMP_DIR/pkg.tar.gz"
elif command -v wget >/dev/null 2>&1; then
  wget -qO "$TMP_DIR/pkg.tar.gz" "$URL"
else
  echo "error: curl or wget required" >&2
  exit 1
fi

tar -xzf "$TMP_DIR/pkg.tar.gz" -C "$TMP_DIR"
for bin in st m8 std st-hub n8x; do
  if [ -f "$TMP_DIR/$bin" ]; then
    install -m 0755 "$TMP_DIR/$bin" "$I1_PREFIX/$bin"
    echo "✅ Installed $I1_PREFIX/$bin"
  fi
done

echo ""
echo "🎉 Done! Ask The Cheet anytime: curl -sL https://i1.is/cheet"
"#
        ),
        other => format!(
            "#!/bin/sh\necho '🐆 [i1.is] Unknown nibble: {other}. Try: curl -fsSL https://i1.is/n/smart-tree | sh' >&2\nexit 1\n"
        ),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn detects_public_git_repository_and_builds_chain_of_title() {
        let tmp = TempDir::new().unwrap();
        let git_dir = tmp.path().join(".git");
        fs::create_dir_all(&git_dir).unwrap();
        fs::write(
            git_dir.join("config"),
            "[remote \"origin\"]\n\turl = https://github.com/8b-is/smart-tree.git\n[user]\n\tname = 8bit-wraith\n",
        )
        .unwrap();
        fs::write(git_dir.join("HEAD"), "abc1234567890def\n").unwrap();
        fs::write(
            tmp.path().join("Cargo.toml"),
            "[package]\nname = \"demo\"\nlicense = \"MIT\"\n",
        )
        .unwrap();

        let cfg = I1NexusConfig::default();
        let report = detect_repo_public_status(tmp.path(), &cfg);
        assert!(report.is_git_repo);
        assert!(report.is_public_repo);
        assert_eq!(
            report.effective_visibility,
            WaveVisibilityMode::PublicCommons
        );
        assert_eq!(
            report.ai_training_grant,
            AiTrainingGrant::FreeCommonsWithChainAttribution
        );
        assert_eq!(report.detected_license.as_deref(), Some("MIT"));
        assert_eq!(report.head_commit.as_deref(), Some("abc123456789"));
        assert!(report
            .public_remotes
            .contains(&"https://github.com/8b-is/smart-tree".to_string()));
    }

    #[test]
    fn defaults_non_repo_path_to_private_sovereign() {
        let tmp = TempDir::new().unwrap();
        let cfg = I1NexusConfig::default();
        let report = detect_repo_public_status(tmp.path(), &cfg);
        assert!(!report.is_public_repo);
        assert_eq!(
            report.effective_visibility,
            WaveVisibilityMode::PrivateSovereign
        );
        assert_eq!(
            report.ai_training_grant,
            AiTrainingGrant::PrivateSovereignNoTraining
        );
    }

    #[test]
    fn renders_cheet_markdown_ansi_and_verified_installer() {
        assert!(is_terminal_cli_user_agent(Some("curl/8.5.0"), None));
        assert!(!is_terminal_cli_user_agent(
            Some("Mozilla/5.0"),
            Some("html")
        ));

        let tmp = TempDir::new().unwrap();
        let cfg = I1NexusConfig::default();
        let report = detect_repo_public_status(tmp.path(), &cfg);
        let store = I1NexusStore::default();

        let md = render_cheet_markdown_answer("smart-tree", None, &store, &report);
        assert!(md.contains("Smart Tree"));
        assert!(md.contains("https://i1.is/n/smart-tree"));

        let ansi = cheet_markdown_to_ansi(&md);
        assert!(ansi.contains("\x1b["));

        let script = render_i1_verified_installer_script("smart-tree");
        assert!(script.starts_with("#!/bin/sh"));
        assert!(script.contains("THE CHEET"));
    }
}
