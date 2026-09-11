//! Wave-based Memory Manager for Claude Code
//!
//! Bridges MCP tools with the MEM8 wave grid for semantic memory storage.
//! Features: resonance-based retrieval, emotional encoding, temporal decay.
//!
//! This is THE memory system for the best Claude Code experience.

use crate::mem8::record_store::RecordStore;
use crate::mem8::{FrequencyBand, MemoryWave, WaveGrid};
use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeSet, HashMap};
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::{Arc, RwLock};

/// Memory types mapped to frequency bands
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub enum MemoryType {
    /// Deep patterns, architecture decisions (Delta: 0-100Hz)
    Pattern,
    /// Solutions, breakthroughs (Theta: 100-200Hz)
    Solution,
    /// Conversational context, rapport (Alpha: 200-300Hz)
    Conversation,
    /// Technical insights, code patterns (Beta: 300-500Hz)
    Technical,
    /// Learning moments, aha! insights (Gamma: 500-800Hz)
    Learning,
    /// Jokes, shared humor (HyperGamma: 800-1000Hz)
    Joke,
}

impl MemoryType {
    /// Convert to frequency band
    pub fn to_frequency_band(&self) -> FrequencyBand {
        match self {
            Self::Pattern => FrequencyBand::Delta,
            Self::Solution => FrequencyBand::Theta,
            Self::Conversation => FrequencyBand::Alpha,
            Self::Technical => FrequencyBand::Beta,
            Self::Learning => FrequencyBand::Gamma,
            Self::Joke => FrequencyBand::HyperGamma,
        }
    }

    /// Get center frequency for this memory type
    pub fn frequency(&self) -> f32 {
        let (min, max) = self.to_frequency_band().range();
        (min + max) / 2.0
    }

    /// Parse from string
    pub fn parse(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "pattern" | "pattern_insight" => Self::Pattern,
            "solution" | "breakthrough" => Self::Solution,
            "conversation" | "rapport" => Self::Conversation,
            "technical" | "technical_pattern" => Self::Technical,
            "learning" | "learning_moment" => Self::Learning,
            "joke" | "shared_joke" => Self::Joke,
            _ => Self::Technical, // Default
        }
    }
}

/// A memory anchored in the wave grid
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnchoredMemory {
    /// Unique identifier
    pub id: String,
    /// The actual content
    pub content: String,
    /// Keywords for fast lookup
    pub keywords: Vec<String>,
    /// Memory type (determines frequency band)
    pub memory_type: MemoryType,
    /// Emotional valence (-1.0 to 1.0): negative to positive
    pub valence: f32,
    /// Emotional arousal (0.0 to 1.0): calm to excited
    pub arousal: f32,
    /// Grid coordinates (semantic position)
    pub x: u8,
    pub y: u8,
    pub z: u16,
    /// When anchored
    pub created_at: DateTime<Utc>,
    /// When last accessed (for reinforcement)
    pub last_accessed: DateTime<Utc>,
    /// Access count (reinforcement strength)
    pub access_count: u32,
    /// Origin (human, ai:claude, tandem:human:claude)
    pub origin: String,
    /// Project path this memory is associated with
    #[serde(with = "crate::mem8::path_serde::option")]
    pub project_path: Option<PathBuf>,
}

impl AnchoredMemory {
    /// Calculate semantic coordinates from content + keywords
    pub fn calculate_coordinates(
        content: &str,
        keywords: &[String],
        memory_type: MemoryType,
    ) -> (u8, u8, u16) {
        // X-axis: content hash for semantic distribution
        let content_hash = Self::hash_string(content);
        let x = (content_hash % 256) as u8;

        // Y-axis: emotional/type spectrum (different memory types spread across Y)
        let type_offset = match memory_type {
            MemoryType::Pattern => 0,
            MemoryType::Solution => 42,
            MemoryType::Conversation => 84,
            MemoryType::Technical => 128,
            MemoryType::Learning => 170,
            MemoryType::Joke => 212,
        };
        let keyword_hash = keywords.iter().map(|k| Self::hash_string(k)).sum::<u64>();
        let y = ((type_offset as u64 + keyword_hash % 43) % 256) as u8;

        // Z-axis: temporal depth (newer = higher Z, using timestamp)
        let now = Utc::now().timestamp() as u64;
        let z = ((now / 60) % 65536) as u16; // Minutes resolution

        (x, y, z)
    }

    /// Simple hash function for string
    fn hash_string(s: &str) -> u64 {
        s.bytes()
            .fold(0u64, |acc, b| acc.wrapping_mul(31).wrapping_add(b as u64))
    }

    /// Calculate resonance with another memory (0.0 to 1.0)
    pub fn resonance_with(&self, other: &AnchoredMemory) -> f32 {
        // Frequency similarity (same type = high resonance)
        let freq_sim = if self.memory_type == other.memory_type {
            1.0
        } else {
            0.5
        };

        // Keyword overlap
        let overlap = self
            .keywords
            .iter()
            .filter(|k| other.keywords.contains(k))
            .count() as f32;
        let total = (self.keywords.len() + other.keywords.len()).max(1) as f32;
        let keyword_sim = 2.0 * overlap / total;

        // Emotional similarity
        let valence_diff = (self.valence - other.valence).abs();
        let arousal_diff = (self.arousal - other.arousal).abs();
        let emotion_sim = 1.0 - (valence_diff + arousal_diff) / 4.0;

        // Spatial proximity in grid
        let dx = (self.x as i32 - other.x as i32).abs() as f32 / 256.0;
        let dy = (self.y as i32 - other.y as i32).abs() as f32 / 256.0;
        let spatial_sim = 1.0 - (dx + dy) / 2.0;

        // Weighted combination
        0.3 * freq_sim + 0.4 * keyword_sim + 0.2 * emotion_sim + 0.1 * spatial_sim
    }

    /// Convert to MemoryWave for grid storage
    pub fn to_wave(&self) -> MemoryWave {
        let frequency = self.memory_type.frequency();
        let amplitude = 0.5 + (self.access_count as f32 / 100.0).min(0.5); // 0.5 to 1.0

        let mut wave = MemoryWave::new(frequency, amplitude);
        wave.valence = self.valence;
        wave.arousal = self.arousal;

        // Phase encodes temporal relationship (when created)
        let hours = self.created_at.timestamp() as f32 / 3600.0;
        wave.phase = (hours % (2.0 * std::f32::consts::PI)).abs();

        wave
    }
}

/// Index for fast keyword lookup
#[derive(Debug, Default, Serialize, Deserialize)]
struct KeywordIndex {
    /// Keyword -> list of memory IDs
    keywords: HashMap<String, Vec<String>>,
}

impl KeywordIndex {
    fn add(&mut self, keyword: &str, memory_id: &str) {
        self.keywords
            .entry(keyword.to_lowercase())
            .or_default()
            .push(memory_id.to_string());
    }

    fn find(&self, keywords: &[String]) -> Vec<String> {
        let mut scores: HashMap<String, usize> = HashMap::new();

        for keyword in keywords {
            if let Some(ids) = self.keywords.get(&keyword.to_lowercase()) {
                for id in ids {
                    *scores.entry(id.clone()).or_default() += 1;
                }
            }
        }

        // Sort by score (most matching keywords first)
        let mut results: Vec<_> = scores.into_iter().collect();
        results.sort_by_key(|a| std::cmp::Reverse(a.1));
        results.into_iter().map(|(id, _)| id).collect()
    }
}

/// The unified Wave Memory Manager
///
/// This is THE memory system for Claude Code - combining:
/// - Wave-based semantic storage (fast, meaningful)
/// - Keyword indexing (instant lookup)
/// - Resonance search (find similar memories)
/// - Emotional context (valence + arousal)
/// - Temporal decay (old memories fade)
pub struct WaveMemoryManager {
    /// The wave grid for semantic storage
    wave_grid: Arc<RwLock<WaveGrid>>,
    /// All anchored memories
    memories: HashMap<String, AnchoredMemory>,
    /// Keyword index for fast lookup
    keyword_index: KeywordIndex,
    /// Path to persistence file
    storage_path: PathBuf,
    /// Whether changes need saving
    dirty: bool,
    dirty_ids: BTreeSet<String>,
    store: Option<RecordStore>,
    storage_error: Option<String>,
}

impl WaveMemoryManager {
    /// Create or load memory manager
    /// Uses the compact grid; persistent native records remain authoritative.
    pub fn new(storage_dir: Option<&Path>) -> Self {
        let storage_path = storage_dir
            .map(|p| p.join(".st").join("mem8").join("wave_memory.m8"))
            .unwrap_or_else(|| {
                std::env::current_dir()
                    .unwrap_or_else(|_| PathBuf::from("."))
                    .join(".st")
                    .join("mem8")
                    .join("wave_memory.m8")
            });

        let mut manager = Self {
            wave_grid: Arc::new(RwLock::new(WaveGrid::new_compact())),
            memories: HashMap::new(),
            keyword_index: KeywordIndex::default(),
            storage_path,
            dirty: false,
            dirty_ids: BTreeSet::new(),
            store: None,
            storage_error: None,
        };

        // Try to load existing memories
        if let Err(e) = manager.load() {
            tracing::error!(error = %e, "Wave memory unavailable; writes disabled");
            manager.storage_error = Some(e.to_string());
        }

        manager
    }

    /// Create memory manager with compact grid (256×256×256)
    /// Use this for daemons and memory-constrained environments
    pub fn new_compact(storage_dir: Option<&Path>) -> Self {
        let storage_path = storage_dir
            .map(|p| p.join(".st").join("mem8").join("wave_memory.m8"))
            .unwrap_or_else(|| {
                std::env::var_os("ST_MEMORY_DIR")
                    .map(PathBuf::from)
                    .unwrap_or_else(|| {
                        dirs::home_dir()
                            .unwrap_or_else(|| PathBuf::from("."))
                            .join(".mem8")
                    })
                    .join("wave_memory.m8")
            });

        let mut manager = Self {
            wave_grid: Arc::new(RwLock::new(WaveGrid::new_compact())),
            memories: HashMap::new(),
            keyword_index: KeywordIndex::default(),
            storage_path,
            dirty: false,
            dirty_ids: BTreeSet::new(),
            store: None,
            storage_error: None,
        };

        // Try to load existing memories
        if let Err(e) = manager.load() {
            tracing::error!(error = %e, "Wave memory unavailable; writes disabled");
            manager.storage_error = Some(e.to_string());
        }

        manager
    }

    pub fn try_new_compact(storage_dir: Option<&Path>) -> Result<Self> {
        let manager = Self::new_compact(storage_dir);
        anyhow::ensure!(
            manager.storage_error.is_none(),
            "Cannot restore wave memory: {}",
            manager.storage_error.as_deref().unwrap_or("unknown error")
        );
        Ok(manager)
    }

    /// Create memory manager with smaller grid for testing
    /// Uses 256×256×256 grid instead of 256×256×65536 (256x smaller)
    #[cfg(test)]
    pub fn new_test(storage_dir: Option<&Path>) -> Self {
        let storage_path = storage_dir
            .map(|p| p.join(".st").join("mem8").join("wave_memory_test.m8"))
            .unwrap_or_else(|| {
                std::env::current_dir()
                    .unwrap_or_else(|_| PathBuf::from("."))
                    .join(".st")
                    .join("mem8")
                    .join("wave_memory_test.m8")
            });

        let mut manager = Self {
            wave_grid: Arc::new(RwLock::new(WaveGrid::new_test())),
            memories: HashMap::new(),
            keyword_index: KeywordIndex::default(),
            storage_path,
            dirty: false,
            dirty_ids: BTreeSet::new(),
            store: None,
            storage_error: None,
        };

        // Try to load existing memories (same as regular new())
        if let Err(error) = manager.load() {
            manager.storage_error = Some(error.to_string());
        }

        manager
    }

    /// Anchor a new memory
    #[allow(clippy::too_many_arguments)] // Builder pattern alternative considered but this is clearer
    pub fn anchor(
        &mut self,
        content: String,
        keywords: Vec<String>,
        memory_type: MemoryType,
        valence: f32,
        arousal: f32,
        origin: String,
        project_path: Option<PathBuf>,
    ) -> Result<String> {
        anyhow::ensure!(
            valence.is_finite() && arousal.is_finite(),
            "Memory emotion must be finite"
        );
        let id = uuid::Uuid::new_v4().to_string();
        let (x, y, z) = AnchoredMemory::calculate_coordinates(&content, &keywords, memory_type);

        let memory = AnchoredMemory {
            id: id.clone(),
            content,
            keywords: keywords.clone(),
            memory_type,
            valence: valence.clamp(-1.0, 1.0),
            arousal: arousal.clamp(0.0, 1.0),
            x,
            y,
            z,
            created_at: Utc::now(),
            last_accessed: Utc::now(),
            access_count: 1,
            origin,
            project_path,
        };

        self.store
            .as_mut()
            .context("Wave memory is unavailable; no volatile fallback is permitted")?
            .put_with_wave(
                &format!("anchor/v1/{id}"),
                &Some(&memory),
                &crate::mem8_lite::Wave::new(
                    f64::from(memory.memory_type.frequency()),
                    f64::from(memory.valence),
                    f64::from(memory.arousal),
                ),
            )?;

        // Store in wave grid
        let wave = memory.to_wave();
        if let Ok(mut grid) = self.wave_grid.write() {
            grid.store(x, y, z, wave);
        }

        // Index keywords
        for keyword in &keywords {
            self.keyword_index.add(keyword, &id);
        }

        // Store memory
        self.memories.insert(id.clone(), memory);

        Ok(id)
    }

    /// Find memories by keywords (fast lookup)
    /// Returns cloned memories to avoid borrow conflicts
    pub fn find_by_keywords(
        &mut self,
        keywords: &[String],
        max_results: usize,
    ) -> Vec<AnchoredMemory> {
        let ids = self.keyword_index.find(keywords);
        let found_ids: Vec<String> = ids.iter().take(max_results).cloned().collect();

        // Collect results first
        let results: Vec<AnchoredMemory> = found_ids
            .iter()
            .filter_map(|id| self.memories.get(id).cloned())
            .collect();

        // Update access counts for found memories
        for id in &found_ids {
            if let Some(mem) = self.memories.get_mut(id) {
                mem.access_count += 1;
                mem.last_accessed = Utc::now();
                self.dirty = true;
                self.dirty_ids.insert(id.clone());
            }
        }

        results
    }

    /// Find memories by resonance (semantic similarity)
    /// Returns cloned memories with resonance scores
    pub fn find_by_resonance(
        &mut self,
        query_content: &str,
        query_keywords: &[String],
        query_type: MemoryType,
        threshold: f32,
        max_results: usize,
    ) -> Vec<(AnchoredMemory, f32)> {
        // Create a query memory for comparison
        let (x, y, z) =
            AnchoredMemory::calculate_coordinates(query_content, query_keywords, query_type);
        let query = AnchoredMemory {
            id: String::new(),
            content: query_content.to_string(),
            keywords: query_keywords.to_vec(),
            memory_type: query_type,
            valence: 0.0,
            arousal: 0.5,
            x,
            y,
            z,
            created_at: Utc::now(),
            last_accessed: Utc::now(),
            access_count: 0,
            origin: String::new(),
            project_path: None,
        };

        // Calculate resonance with all memories and collect with IDs
        let mut resonances: Vec<(String, AnchoredMemory, f32)> = self
            .memories
            .values()
            .map(|mem| (mem.id.clone(), mem.clone(), mem.resonance_with(&query)))
            .filter(|(_, _, r)| *r >= threshold)
            .collect();

        // Sort by resonance (highest first)
        resonances.sort_by(|a, b| b.2.partial_cmp(&a.2).unwrap_or(std::cmp::Ordering::Equal));

        // Get the IDs to update
        let update_ids: Vec<String> = resonances
            .iter()
            .take(max_results)
            .map(|(id, _, _)| id.clone())
            .collect();

        // Update access counts
        for id in &update_ids {
            if let Some(m) = self.memories.get_mut(id) {
                m.access_count += 1;
                m.last_accessed = Utc::now();
                self.dirty = true;
                self.dirty_ids.insert(id.clone());
            }
        }

        resonances
            .into_iter()
            .take(max_results)
            .map(|(_, mem, r)| (mem, r))
            .collect()
    }

    /// Get wave interference pattern at a location
    pub fn get_interference(&self, x: u8, y: u8, z: u16, t: f32) -> f32 {
        if let Ok(grid) = self.wave_grid.read() {
            grid.calculate_interference(x, y, z, t)
        } else {
            0.0
        }
    }

    /// Get memory statistics
    pub fn stats(&self) -> serde_json::Value {
        let type_counts: HashMap<String, usize> =
            self.memories.values().fold(HashMap::new(), |mut acc, mem| {
                *acc.entry(format!("{:?}", mem.memory_type)).or_default() += 1;
                acc
            });

        let active_count = if let Ok(grid) = self.wave_grid.read() {
            grid.active_memory_count()
        } else {
            0
        };

        serde_json::json!({
            "total_memories": self.memories.len(),
            "active_waves": active_count,
            "unique_keywords": self.keyword_index.keywords.len(),
            "by_type": type_counts,
            "storage_path": self.storage_path.with_extension("native.m8").display().to_string(),
            "storage_error": self.storage_error,
        })
    }

    /// Save memories to disk
    pub fn save(&mut self) -> Result<()> {
        if !self.dirty {
            return Ok(());
        }

        let store = self
            .store
            .as_mut()
            .context("Native wave memory is unavailable")?;
        for id in self.dirty_ids.clone() {
            if let Some(memory) = self.memories.get(&id) {
                store.put_with_wave(
                    &format!("anchor/v1/{id}"),
                    &Some(memory),
                    &crate::mem8_lite::Wave::new(
                        f64::from(memory.memory_type.frequency()),
                        f64::from(memory.valence),
                        f64::from(memory.arousal),
                    ),
                )?;
            }
            self.dirty_ids.remove(&id);
        }
        self.dirty = false;
        Ok(())
    }

    /// Load memories from disk
    pub fn load(&mut self) -> Result<()> {
        if self.store.is_some() {
            return Ok(());
        }
        let mut store = RecordStore::open_memory(&self.storage_path.with_extension("native.m8"))?;
        if store.get::<bool>("migration/wave-json-v1")? != Some(true) {
            #[derive(Deserialize)]
            struct Legacy {
                version: u32,
                memories: HashMap<String, AnchoredMemory>,
            }
            match fs::read(&self.storage_path) {
                Ok(bytes) => {
                    let legacy: Legacy = serde_json::from_slice(&bytes)
                        .context("Invalid legacy wave memory; original file preserved")?;
                    anyhow::ensure!(
                        legacy.version == 1,
                        "Unsupported legacy wave memory version"
                    );
                    for (id, memory) in legacy.memories {
                        anyhow::ensure!(id == memory.id, "Invalid legacy wave memory ID");
                        let key = format!("anchor/v1/{id}");
                        if store.get::<Option<AnchoredMemory>>(&key)?.is_none() {
                            store.put_with_wave(
                                &key,
                                &Some(&memory),
                                &crate::mem8_lite::Wave::new(
                                    f64::from(memory.memory_type.frequency()),
                                    f64::from(memory.valence),
                                    f64::from(memory.arousal),
                                ),
                            )?;
                        }
                    }
                }
                Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
                Err(error) => return Err(error).context("Cannot read legacy wave memory"),
            }
            store.put("migration/wave-json-v1", &true)?;
        }
        let mut memories = HashMap::new();
        let mut keyword_index = KeywordIndex::default();
        for key in store.keys_with_prefix("anchor/v1/") {
            if let Some(Some(mut memory)) = store.get::<Option<AnchoredMemory>>(&key)? {
                anyhow::ensure!(
                    key == format!("anchor/v1/{}", memory.id),
                    "Invalid persisted memory ID"
                );
                if let Some(wave) = store.get_wave(&key)? {
                    memory.valence = wave.emotional_valence as f32;
                    memory.arousal = wave.arousal as f32;
                }
                for keyword in &memory.keywords {
                    keyword_index.add(keyword, &memory.id);
                }
                memories.insert(memory.id.clone(), memory);
            }
        }
        // Hydrate the working grid from persisted waves; no semantic re-analysis.
        if let Ok(mut grid) = self.wave_grid.write() {
            for memory in memories.values() {
                let wave = memory.to_wave();
                grid.store(memory.x, memory.y, memory.z, wave);
            }
        }
        self.memories = memories;
        self.keyword_index = keyword_index;
        self.store = Some(store);
        self.storage_error = None;
        Ok(())
    }

    /// Get a memory by ID
    pub fn get(&self, id: &str) -> Option<&AnchoredMemory> {
        self.memories.get(id)
    }

    /// Delete a memory
    pub fn delete(&mut self, id: &str) -> bool {
        match self.try_delete(id) {
            Ok(deleted) => deleted,
            Err(error) => {
                tracing::error!(%error, "Could not persist memory deletion");
                false
            }
        }
    }

    pub fn try_delete(&mut self, id: &str) -> Result<bool> {
        if !self.memories.contains_key(id) {
            return Ok(false);
        }
        self.store
            .as_mut()
            .context("Native wave memory is unavailable")?
            .forget::<AnchoredMemory>(&format!("anchor/v1/{id}"))?;
        if let Some(memory) = self.memories.remove(id) {
            // Note: We don't remove from wave grid (it will decay naturally)
            // But we do remove from keyword index
            for keyword in &memory.keywords {
                if let Some(ids) = self.keyword_index.keywords.get_mut(&keyword.to_lowercase()) {
                    ids.retain(|i| i != id);
                }
            }
        }
        self.dirty_ids.remove(id);
        Ok(true)
    }
}

impl Drop for WaveMemoryManager {
    fn drop(&mut self) {
        // Best effort save on drop
        let _ = self.save();
    }
}

/// Global instance for MCP tool access
static WAVE_MEMORY: std::sync::OnceLock<std::sync::Mutex<WaveMemoryManager>> =
    std::sync::OnceLock::new();

/// Get the global wave memory manager
pub fn get_wave_memory() -> &'static std::sync::Mutex<WaveMemoryManager> {
    WAVE_MEMORY.get_or_init(|| std::sync::Mutex::new(WaveMemoryManager::new(None)))
}

/// Initialize wave memory with a specific storage directory
pub fn init_wave_memory(storage_dir: &Path) {
    let _ = WAVE_MEMORY.set(std::sync::Mutex::new(WaveMemoryManager::new(Some(
        storage_dir,
    ))));
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn native_migration_and_delete_preserve_legacy_without_resurrection() {
        let dir = tempdir().unwrap();
        let mut original = WaveMemoryManager::new_test(Some(dir.path()));
        let id = original
            .anchor(
                "saved conversation".into(),
                vec!["family".into()],
                MemoryType::Conversation,
                0.25,
                0.75,
                "user".into(),
                None,
            )
            .unwrap();
        let memory = original.get(&id).unwrap().clone();
        let legacy_path = original.storage_path.clone();
        let native_path = legacy_path.with_extension("native.m8");
        drop(original);
        // Simulate an existing deployment that only has the legacy JSON.
        std::fs::remove_file(&native_path).unwrap();
        let legacy = serde_json::to_vec(
            &serde_json::json!({ "version": 1, "memories": { id.clone(): memory } }),
        )
        .unwrap();
        std::fs::write(&legacy_path, &legacy).unwrap();
        let mut restored = WaveMemoryManager::new_test(Some(dir.path()));
        assert_eq!(restored.get(&id).unwrap().content, "saved conversation");
        assert!(restored.try_delete(&id).unwrap());
        drop(restored);
        let restored = WaveMemoryManager::new_test(Some(dir.path()));
        assert!(restored.get(&id).is_none());
        assert_eq!(std::fs::read(legacy_path).unwrap(), legacy);
        assert_eq!(
            &std::fs::read(native_path).unwrap()[8..12],
            &0x4d454d38u32.to_le_bytes()
        );
    }

    #[test]
    fn corrupt_legacy_wave_memory_disables_writes() {
        let dir = tempdir().unwrap();
        let parent = dir.path().join(".st/mem8");
        std::fs::create_dir_all(&parent).unwrap();
        let path = parent.join("wave_memory_test.m8");
        std::fs::write(&path, b"corrupt legacy memory").unwrap();
        let mut memory = WaveMemoryManager::new_test(Some(dir.path()));
        assert!(memory.storage_error.is_some());
        assert!(memory
            .anchor(
                "new".into(),
                vec![],
                MemoryType::Conversation,
                0.0,
                0.5,
                "user".into(),
                None
            )
            .is_err());
        assert_eq!(std::fs::read(path).unwrap(), b"corrupt legacy memory");
    }

    #[test]
    fn test_anchor_and_find() {
        let dir = tempdir().unwrap();
        let mut manager = WaveMemoryManager::new_test(Some(dir.path()));

        // Anchor a memory
        let id = manager
            .anchor(
                "The solution to the authentication bug was using JWT refresh tokens".to_string(),
                vec!["auth".to_string(), "jwt".to_string(), "bug".to_string()],
                MemoryType::Solution,
                0.8, // Positive valence
                0.7, // High arousal (exciting!)
                "tandem:hue:claude".to_string(),
                None,
            )
            .unwrap();

        assert!(!id.is_empty());

        // Find by keywords
        let results = manager.find_by_keywords(&["auth".to_string(), "jwt".to_string()], 10);
        assert_eq!(results.len(), 1);
        assert!(results[0].content.contains("JWT refresh tokens"));
    }

    #[test]
    fn test_resonance_search() {
        let dir = tempdir().unwrap();
        let mut manager = WaveMemoryManager::new_test(Some(dir.path()));

        // Anchor several memories
        manager
            .anchor(
                "Rust async/await pattern for error handling".to_string(),
                vec!["rust".to_string(), "async".to_string(), "error".to_string()],
                MemoryType::Technical,
                0.3,
                0.5,
                "tandem:hue:claude".to_string(),
                None,
            )
            .unwrap();

        manager
            .anchor(
                "Go channels for concurrent error propagation".to_string(),
                vec![
                    "go".to_string(),
                    "channels".to_string(),
                    "error".to_string(),
                ],
                MemoryType::Technical,
                0.2,
                0.4,
                "tandem:hue:claude".to_string(),
                None,
            )
            .unwrap();

        // Search by resonance
        let results = manager.find_by_resonance(
            "error handling in async code",
            &["async".to_string(), "error".to_string()],
            MemoryType::Technical,
            0.3, // threshold
            10,
        );

        // Should find the Rust memory with higher resonance
        assert!(!results.is_empty());
        assert!(results[0].0.content.contains("Rust") || results[0].0.content.contains("error"));
    }

    #[test]
    fn test_persistence() {
        let dir = tempdir().unwrap();

        // Create and save
        {
            let mut manager = WaveMemoryManager::new_test(Some(dir.path()));
            manager
                .anchor(
                    "Aye loves Elvis!".to_string(),
                    vec!["aye".to_string(), "elvis".to_string()],
                    MemoryType::Joke,
                    1.0,
                    1.0,
                    "tandem:hue:claude".to_string(),
                    None,
                )
                .unwrap();
            manager.save().unwrap();
        }

        // Load and verify
        {
            let mut manager = WaveMemoryManager::new_test(Some(dir.path()));
            let results = manager.find_by_keywords(&["elvis".to_string()], 10);
            assert_eq!(results.len(), 1);
            assert!(results[0].content.contains("Elvis"));
        }
    }

    #[test]
    fn test_memory_types_to_frequencies() {
        assert!(MemoryType::Pattern.frequency() < MemoryType::Solution.frequency());
        assert!(MemoryType::Solution.frequency() < MemoryType::Technical.frequency());
        assert!(MemoryType::Technical.frequency() < MemoryType::Joke.frequency());
    }
}
