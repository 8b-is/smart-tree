//! 🧬 MEM8 Life Persona & Specialist Soul Engine (250,000+ Episodes)
//!
//! Builds a tight binary `.m8` persona container (`MEM8` v1 + `WAVE` + `MQNT` + `ZLIB`)
//! from any folder of diaries, letters, notes, codebases, or synced Google Drive documents.
//! Supports both personal Life Biographer souls and domain-specialized souls:
//! - `life-biographer`    (🧬 Life Biographer / Personal Soul)
//! - `history-documenter` (📜 History & Timeline Documenter)
//! - `code-analyst`       (🔬 Code & Architecture Analyst)
//! - `readme-expert`      (📖 README & Documentation Expert)

use crate::m8_consciousness::{decode_m8_bytes, encode_json_bytes_as_tight_m8};
use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use once_cell::sync::Lazy;
use rayon::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::{Arc, RwLock};
use std::time::SystemTime;
use walkdir::WalkDir;

/// Maximum episodic memories stored in a single `.m8` soul container (250,000+)
pub const MAX_PERSONA_EPISODES: usize = 250_000;

/// Maximum files scanned per persona training run
pub const MAX_PERSONA_FILES: usize = 50_000;

/// Maximum bytes per individual source file (16 MiB)
pub const MAX_FILE_BYTES: u64 = 16 * 1024 * 1024;

/// In-memory cache of loaded personas keyed by canonical `.m8` path and file mtime
type PersonaCacheMap = HashMap<PathBuf, (Option<SystemTime>, Arc<LifePersona>)>;

static PERSONA_CACHE: Lazy<RwLock<PersonaCacheMap>> = Lazy::new(|| RwLock::new(HashMap::new()));

/// Specialty lens for a trained MEM8 Persona
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "kebab-case")]
pub enum PersonaSpecialty {
    /// Personal life corpus, diaries, journals, letters, and voice preservation
    #[default]
    LifeBiographer,
    /// Chronological timelines, changelogs, diaries, git history, and era evolution
    HistoryDocumenter,
    /// Source code architecture, structs/functions, performance, and system design
    CodeAnalyst,
    /// READMEs, onboarding guides, CLI references, and developer documentation
    ReadmeExpert,
}

impl PersonaSpecialty {
    pub fn from_str_loose(s: &str) -> Self {
        match s.trim().to_ascii_lowercase().as_str() {
            "history" | "history-documenter" | "history_documenter" | "historian" => {
                Self::HistoryDocumenter
            }
            "code" | "code-analyst" | "code_analyst" | "analyst" | "architect" => Self::CodeAnalyst,
            "readme" | "readme-expert" | "readme_expert" | "docs" | "doc-expert" => {
                Self::ReadmeExpert
            }
            _ => Self::LifeBiographer,
        }
    }

    pub fn slug(&self) -> &'static str {
        match self {
            Self::LifeBiographer => "life-biographer",
            Self::HistoryDocumenter => "history-documenter",
            Self::CodeAnalyst => "code-analyst",
            Self::ReadmeExpert => "readme-expert",
        }
    }

    pub fn badge(&self) -> &'static str {
        match self {
            Self::LifeBiographer => "🧬 Life Biographer",
            Self::HistoryDocumenter => "📜 History Documenter",
            Self::CodeAnalyst => "🔬 Code Analyst",
            Self::ReadmeExpert => "📖 README Expert",
        }
    }
}

/// A single episodic memory passage extracted from a diary, letter, doc, or source file
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PersonaEpisode {
    /// Source file (relative to corpus root)
    pub source: String,
    /// Detected date or era label if present (e.g. "1964-05-14" or "1996")
    pub era: Option<String>,
    /// Salient keywords from this passage
    pub keywords: Vec<String>,
    /// Emotional valence (-1.0 to 1.0)
    pub valence: f32,
    /// Emotional arousal (0.0 to 1.0)
    pub arousal: f32,
    /// Wave frequency in Hz (20.0..220.0)
    pub frequency_hz: f32,
    /// The actual passage text
    pub passage: String,
}

/// Extracted voice, cadence, and vocabulary signature
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VoiceSignature {
    /// Average words per sentence
    pub avg_sentence_words: f32,
    /// Overall emotional warmth / positivity (-1.0 to 1.0)
    pub warmth: f32,
    /// Expressiveness / exclamation & question cadence (0.0 to 1.0)
    pub expressiveness: f32,
    /// Signature vocabulary (most characteristic natural words)
    pub vocabulary: Vec<String>,
    /// Recurring 2-word natural phrases / idioms
    pub signature_phrases: Vec<String>,
    /// Sample opening sentences from their actual writing
    pub sample_openings: Vec<String>,
}

/// Complete trained Life Persona stored inside `~/.st/personas/<slug>.m8`
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LifePersona {
    /// Display name (e.g., "Grandmother's Diaries" or "Nate")
    pub name: String,
    /// URL/filename-safe slug (e.g., "grandmothers-diaries" or "nate")
    pub slug: String,
    /// Specialty lens
    #[serde(default)]
    pub specialty: PersonaSpecialty,
    /// Source path used for training
    pub source_path: String,
    /// When the persona was trained
    pub trained_at: DateTime<Utc>,
    /// Number of source files ingested
    pub files_ingested: usize,
    /// Total words analyzed
    pub total_words: usize,
    /// Voice & style profile
    pub voice: VoiceSignature,
    /// Episodic memories ("Corrugated Iron" foundational memories, up to 250,000+)
    pub episodes: Vec<PersonaEpisode>,
}

/// Summary metadata for listing personas in CLI and Dashboard (`<slug>.meta.json`)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PersonaSummary {
    pub name: String,
    pub slug: String,
    #[serde(default)]
    pub specialty: PersonaSpecialty,
    pub source_path: String,
    pub trained_at: DateTime<Utc>,
    pub files_ingested: usize,
    pub total_words: usize,
    pub episode_count: usize,
    pub m8_size_bytes: u64,
    pub sample_phrases: Vec<String>,
}

struct FileIngestStats {
    words: usize,
    sentences: usize,
    expressive: usize,
    valence_sum: f32,
    word_counts: HashMap<String, usize>,
    phrase_counts: HashMap<String, usize>,
    openings: Vec<String>,
    episodes: Vec<PersonaEpisode>,
}

impl LifePersona {
    /// Directory where `.m8` personas live (`~/.st/personas`)
    pub fn personas_dir() -> Result<PathBuf> {
        let home = std::env::var("HOME")
            .ok()
            .map(PathBuf::from)
            .or_else(dirs::home_dir)
            .unwrap_or_else(|| PathBuf::from("."));
        let dir = home.join(".st").join("personas");
        fs::create_dir_all(&dir)?;
        Ok(dir)
    }

    /// Resolve `.st/personas` directory preferring the home directory that owns `source`
    /// (so a root-owned system daemon training `/home/hue/...` stores into `/home/hue/.st/personas`).
    pub fn personas_dir_for_source(source: &Path) -> Result<PathBuf> {
        let comps: Vec<_> = source.components().collect();
        if comps.len() >= 3 {
            let first = comps[1].as_os_str().to_string_lossy();
            let second = comps[2].as_os_str().to_string_lossy();
            if (first == "home" || first == "Users") && !second.is_empty() {
                let candidate_home = PathBuf::from("/")
                    .join(first.as_ref())
                    .join(second.as_ref());
                if candidate_home.is_dir() {
                    let dir = candidate_home.join(".st").join("personas");
                    if fs::create_dir_all(&dir).is_ok() {
                        return Ok(dir);
                    }
                }
            }
        }
        Self::personas_dir()
    }

    /// Return all `.st/personas` directories visible on the host (current user + `/home/*/.st/personas`).
    pub fn all_personas_dirs() -> Vec<PathBuf> {
        let mut dirs = Vec::new();
        if let Ok(primary) = Self::personas_dir() {
            dirs.push(primary);
        }
        for base in ["/home", "/Users"] {
            if let Ok(entries) = fs::read_dir(base) {
                for entry in entries.flatten() {
                    let p = entry.path().join(".st").join("personas");
                    if p.is_dir() && !dirs.contains(&p) {
                        dirs.push(p);
                    }
                }
            }
        }
        dirs
    }

    /// Normalize a persona name into a clean slug
    pub fn slugify(name: &str) -> String {
        let mut slug = String::new();
        let mut prev_dash = false;
        for ch in name.chars() {
            if ch.is_ascii_alphanumeric() {
                slug.push(ch.to_ascii_lowercase());
                prev_dash = false;
            } else if (ch == ' ' || ch == '-' || ch == '_') && !prev_dash && !slug.is_empty() {
                slug.push('-');
                prev_dash = true;
            }
        }
        let trimmed = slug.trim_matches('-').to_string();
        if trimmed.is_empty() {
            "persona".to_string()
        } else {
            trimmed
        }
    }

    /// Train a Life Persona with default `LifeBiographer` specialty
    pub fn train_from_path(name: &str, root_path: &Path) -> Result<(Self, PathBuf, u64)> {
        Self::train_with_specialty(name, root_path, PersonaSpecialty::LifeBiographer)
    }

    /// Train a Persona with an explicit `PersonaSpecialty` and save up to 250,000+ episodes to `~/.st/personas/<slug>.m8`
    pub fn train_with_specialty(
        name: &str,
        root_path: &Path,
        specialty: PersonaSpecialty,
    ) -> Result<(Self, PathBuf, u64)> {
        let canon = root_path
            .canonicalize()
            .unwrap_or_else(|_| root_path.to_path_buf());
        if !canon.exists() {
            anyhow::bail!("Training path does not exist: {}", canon.display());
        }

        let slug = Self::slugify(name);

        let allowed_exts: HashSet<&str> = match specialty {
            PersonaSpecialty::CodeAnalyst => [
                "rs", "py", "ts", "tsx", "js", "jsx", "go", "c", "cpp", "h", "hpp", "swift", "sh",
                "toml", "yaml", "yml", "sql", "md", "txt",
            ]
            .into_iter()
            .collect(),
            PersonaSpecialty::ReadmeExpert => ["md", "markdown", "txt", "org", "rst", "toml"]
                .into_iter()
                .collect(),
            PersonaSpecialty::HistoryDocumenter | PersonaSpecialty::LifeBiographer => [
                "md", "txt", "org", "rst", "eml", "markdown", "json", "html", "htm", "tex", "csv",
            ]
            .into_iter()
            .collect(),
        };

        let entries: Vec<PathBuf> = if canon.is_file() {
            vec![canon.clone()]
        } else {
            let mut collected: Vec<PathBuf> = WalkDir::new(&canon)
                .max_depth(12)
                .follow_links(false)
                .into_iter()
                .filter_entry(|e| {
                    if e.depth() == 0 {
                        return true;
                    }
                    let fname = e.file_name().to_string_lossy();
                    !fname.starts_with('.')
                        && fname != "node_modules"
                        && fname != "target"
                        && fname != "venv"
                        && fname != "__pycache__"
                        && fname != "dist"
                        && fname != "build"
                        && fname != "vendor"
                })
                .filter_map(|e| e.ok())
                .filter(|e| e.path().is_file())
                .filter(|e| {
                    let path = e.path();
                    let ext_ok = path
                        .extension()
                        .and_then(|ext| ext.to_str())
                        .map(|ext| allowed_exts.contains(ext.to_ascii_lowercase().as_str()))
                        .unwrap_or(false);
                    if !ext_ok {
                        return false;
                    }
                    if specialty == PersonaSpecialty::ReadmeExpert {
                        let lower_path = path.to_string_lossy().to_ascii_lowercase();
                        let fname = path
                            .file_name()
                            .map(|f| f.to_string_lossy().to_ascii_lowercase())
                            .unwrap_or_default();
                        return fname.contains("readme")
                            || fname.contains("agents")
                            || fname.contains("claude")
                            || fname.contains("skill")
                            || fname.contains("guide")
                            || fname.contains("arch")
                            || fname.contains("changelog")
                            || fname.contains("contributing")
                            || lower_path.contains("/docs/")
                            || lower_path.ends_with(".md");
                    }
                    true
                })
                .take(MAX_PERSONA_FILES)
                .map(|e| e.path().to_path_buf())
                .collect();

            // If ReadmeExpert or HistoryDocumenter, sort priority files first
            if specialty == PersonaSpecialty::ReadmeExpert {
                collected.sort_by_key(|p| {
                    let fname = p
                        .file_name()
                        .map(|f| f.to_string_lossy().to_ascii_lowercase())
                        .unwrap_or_default();
                    if fname.starts_with("readme") {
                        0usize
                    } else if fname.contains("agents") || fname.contains("architecture") {
                        1
                    } else {
                        2
                    }
                });
            }
            collected
        };

        // Compute per-file episode cap so large multi-file corpora distribute coverage evenly
        // while still allowing up to MAX_PERSONA_EPISODES (250,000) total episodes.
        let file_count = entries.len().max(1);
        let per_file_cap = ((MAX_PERSONA_EPISODES / file_count).max(200)).min(25_000);

        // Parallel ingestion across all CPU cores
        let per_file_results: Vec<FileIngestStats> = entries
            .par_iter()
            .filter_map(|file_path| ingest_single_file(file_path, &canon, specialty, per_file_cap))
            .collect();

        let files_ingested = per_file_results.len();
        let mut total_words = 0usize;
        let mut sentence_count = 0usize;
        let mut expressive_sentences = 0usize;
        let mut total_valence = 0.0f32;
        let mut word_counts: HashMap<String, usize> = HashMap::new();
        let mut phrase_counts: HashMap<String, usize> = HashMap::new();
        let mut sample_openings: Vec<String> = Vec::new();
        let mut episodes: Vec<PersonaEpisode> = Vec::new();

        for stats in per_file_results {
            total_words += stats.words;
            sentence_count += stats.sentences;
            expressive_sentences += stats.expressive;
            total_valence += stats.valence_sum;
            for (w, c) in stats.word_counts {
                *word_counts.entry(w).or_insert(0) += c;
            }
            for (p, c) in stats.phrase_counts {
                *phrase_counts.entry(p).or_insert(0) += c;
            }
            for op in stats.openings {
                if sample_openings.len() < 16 {
                    sample_openings.push(op);
                }
            }
            for ep in stats.episodes {
                if episodes.len() < MAX_PERSONA_EPISODES {
                    episodes.push(ep);
                }
            }
        }

        if episodes.is_empty() {
            anyhow::bail!(
                "No readable passages found in {} for specialty {}",
                canon.display(),
                specialty.slug()
            );
        }

        let mut vocab_vec: Vec<(String, usize)> = word_counts.into_iter().collect();
        vocab_vec.sort_by_key(|b| std::cmp::Reverse(b.1));
        let vocabulary: Vec<String> = vocab_vec.into_iter().take(64).map(|(w, _)| w).collect();

        let mut phrase_vec: Vec<(String, usize)> =
            phrase_counts.into_iter().filter(|(_, c)| *c >= 2).collect();
        phrase_vec.sort_by_key(|b| std::cmp::Reverse(b.1));
        let signature_phrases: Vec<String> =
            phrase_vec.into_iter().take(32).map(|(p, _)| p).collect();

        let avg_sentence_words = if sentence_count > 0 {
            (total_words as f32) / (sentence_count as f32)
        } else {
            14.0
        };
        let warmth = (total_valence / (episodes.len() as f32)).clamp(-1.0, 1.0);
        let expressiveness = if sentence_count > 0 {
            ((expressive_sentences as f32) / (sentence_count as f32)).clamp(0.0, 1.0)
        } else {
            0.2
        };

        let persona = Self {
            name: name.to_string(),
            slug: slug.clone(),
            specialty,
            source_path: canon.to_string_lossy().to_string(),
            trained_at: Utc::now(),
            files_ingested,
            total_words,
            voice: VoiceSignature {
                avg_sentence_words,
                warmth,
                expressiveness,
                vocabulary,
                signature_phrases,
                sample_openings,
            },
            episodes,
        };

        let target_dir = Self::personas_dir_for_source(&canon)?;
        let m8_path = target_dir.join(format!("{slug}.m8"));
        let meta_path = target_dir.join(format!("{slug}.meta.json"));

        let json_bytes = serde_json::to_vec(&persona)?;
        let soul_md = format!(
            "# MEM8 Persona Soul: {} ({})\nSource: {}\nEpisodes: {}\nWords: {}\nVocabulary: {}\n",
            persona.name,
            persona.specialty.badge(),
            persona.source_path,
            persona.episodes.len(),
            persona.total_words,
            persona.voice.vocabulary.join(", ")
        );
        let packed = encode_json_bytes_as_tight_m8(&json_bytes, 42.73, &soul_md)?;
        fs::write(&m8_path, &packed)?;
        let size = packed.len() as u64;

        // Write lightweight sidecar metadata for O(1) listing
        let summary = persona.to_summary(size);
        if let Ok(meta_json) = serde_json::to_vec_pretty(&summary) {
            let _ = fs::write(&meta_path, meta_json);
        }

        // Update in-memory cache immediately
        let mtime = fs::metadata(&m8_path).and_then(|m| m.modified()).ok();
        if let Ok(mut cache) = PERSONA_CACHE.write() {
            cache.insert(m8_path.clone(), (mtime, Arc::new(persona.clone())));
        }

        Ok((persona, m8_path, size))
    }

    /// Train all 3 built-in workspace specialist personas (`History Documenter`, `Code Analyst`, `README Expert`)
    /// on a target workspace root.
    pub fn train_workspace_specialties(root_path: &Path) -> Vec<Result<PersonaSummary>> {
        let specs = [
            ("History Documenter", PersonaSpecialty::HistoryDocumenter),
            ("Code Analyst", PersonaSpecialty::CodeAnalyst),
            ("README Expert", PersonaSpecialty::ReadmeExpert),
        ];
        specs
            .into_iter()
            .map(|(title, spec)| {
                let (p, _path, size) = Self::train_with_specialty(title, root_path, spec)?;
                Ok(p.to_summary(size))
            })
            .collect()
    }

    pub fn to_summary(&self, m8_size_bytes: u64) -> PersonaSummary {
        PersonaSummary {
            name: self.name.clone(),
            slug: self.slug.clone(),
            specialty: self.specialty,
            source_path: self.source_path.clone(),
            trained_at: self.trained_at,
            files_ingested: self.files_ingested,
            total_words: self.total_words,
            episode_count: self.episodes.len(),
            m8_size_bytes,
            sample_phrases: self
                .voice
                .signature_phrases
                .iter()
                .take(6)
                .cloned()
                .collect(),
        }
    }

    /// Load a trained persona by name or slug from `~/.st/personas/<slug>.m8` (using in-RAM cache)
    pub fn load(name_or_slug: &str) -> Result<Self> {
        Ok((*Self::load_cached(name_or_slug)?).clone())
    }

    /// Load a trained persona as a shared `Arc<LifePersona>` from the in-memory mtime-validated cache
    pub fn load_cached(name_or_slug: &str) -> Result<Arc<Self>> {
        let clean = name_or_slug
            .strip_prefix("persona:")
            .or_else(|| name_or_slug.strip_prefix("persona/"))
            .unwrap_or(name_or_slug);
        let slug = Self::slugify(clean);
        for dir in Self::all_personas_dirs() {
            let candidate = dir.join(format!("{slug}.m8"));
            if candidate.is_file() {
                return Self::load_cached_from_file(&candidate);
            }
        }
        let fallback = Self::personas_dir()?.join(format!("{slug}.m8"));
        Self::load_cached_from_file(&fallback)
    }

    /// Load a trained persona from a specific `.m8` file path with mtime cache validation
    pub fn load_cached_from_file(path: &Path) -> Result<Arc<Self>> {
        let mtime = fs::metadata(path).and_then(|m| m.modified()).ok();
        if let Ok(cache) = PERSONA_CACHE.read() {
            if let Some((cached_mtime, arc_p)) = cache.get(path) {
                if *cached_mtime == mtime {
                    return Ok(Arc::clone(arc_p));
                }
            }
        }
        let loaded = Arc::new(Self::load_from_file(path)?);
        if let Ok(mut cache) = PERSONA_CACHE.write() {
            cache.insert(path.to_path_buf(), (mtime, Arc::clone(&loaded)));
        }
        Ok(loaded)
    }

    /// Load a trained persona directly from a `.m8` file on disk
    pub fn load_from_file(path: &Path) -> Result<Self> {
        let raw = fs::read(path)
            .with_context(|| format!("Persona file not found: {}", path.display()))?;
        let unpacked = decode_m8_bytes(&raw)?;
        let persona: Self = serde_json::from_slice(&unpacked)
            .context("Failed to decode LifePersona from .m8 container")?;
        Ok(persona)
    }

    /// List all trained personas in `~/.st/personas/` (uses `<slug>.meta.json` sidecar for instant listing)
    pub fn list_all() -> Result<Vec<PersonaSummary>> {
        let mut summaries = Vec::new();
        let mut seen_slugs = HashSet::new();
        for dir in Self::all_personas_dirs() {
            let Ok(entries) = fs::read_dir(&dir) else {
                continue;
            };
            for entry in entries.flatten() {
                let path = entry.path();
                if path.extension().and_then(|e| e.to_str()) != Some("m8") {
                    continue;
                }
                let size = entry.metadata().map(|m| m.len()).unwrap_or(0);
                let stem = path
                    .file_stem()
                    .and_then(|s| s.to_str())
                    .unwrap_or_default()
                    .to_string();
                if stem.is_empty() || seen_slugs.contains(&stem) {
                    continue;
                }

                let meta_path = dir.join(format!("{stem}.meta.json"));
                let summary_opt = if let Ok(meta_bytes) = fs::read(&meta_path) {
                    serde_json::from_slice::<PersonaSummary>(&meta_bytes)
                        .ok()
                        .map(|mut s| {
                            s.m8_size_bytes = size;
                            s
                        })
                } else {
                    None
                };

                let summary = match summary_opt {
                    Some(s) => Some(s),
                    None => {
                        if let Ok(p) = Self::load_cached_from_file(&path) {
                            let s = p.to_summary(size);
                            if let Ok(meta_json) = serde_json::to_vec_pretty(&s) {
                                let _ = fs::write(&meta_path, meta_json);
                            }
                            Some(s)
                        } else {
                            None
                        }
                    }
                };

                if let Some(s) = summary {
                    if seen_slugs.insert(s.slug.clone()) {
                        summaries.push(s);
                    }
                }
            }
        }
        summaries.sort_by_key(|b| std::cmp::Reverse(b.trained_at));
        Ok(summaries)
    }

    /// Retrieve top-K resonant episodic memories for a query using parallel wave + keyword salience
    pub fn recall_episodes(&self, query: &str, top_k: usize) -> Vec<&PersonaEpisode> {
        let query_tokens: Vec<String> = query
            .split_whitespace()
            .map(|w| {
                w.trim_matches(|c: char| !c.is_alphanumeric())
                    .to_lowercase()
            })
            .filter(|w| w.len() >= 3 && !is_stopword(w))
            .collect();

        let mut query_hash = 0u64;
        for b in query.bytes() {
            query_hash = query_hash.wrapping_mul(31).wrapping_add(b as u64);
        }
        let query_freq = 20.0 + ((query_hash % 200) as f32);
        let is_history = self.specialty == PersonaSpecialty::HistoryDocumenter;

        let mut scored: Vec<(f32, &PersonaEpisode)> = self
            .episodes
            .par_iter()
            .map(|ep| {
                let mut score = 0.0f32;
                let mut matched_any = false;
                for qt in &query_tokens {
                    if ep.keywords.iter().any(|k| k == qt) {
                        score += 5.0;
                        matched_any = true;
                    } else if ep.passage.to_ascii_lowercase().contains(qt.as_str()) {
                        score += 2.5;
                        matched_any = true;
                    }
                    if ep.source.to_ascii_lowercase().contains(qt.as_str()) {
                        score += 2.5;
                        matched_any = true;
                    }
                    if let Some(ref era) = ep.era {
                        if era.to_ascii_lowercase().contains(qt.as_str()) {
                            score += 4.5;
                            matched_any = true;
                        }
                    }
                }
                if is_history && ep.era.is_some() {
                    score += if matched_any { 2.5 } else { 0.4 };
                }
                // Wave resonance harmonic bonus
                let freq_diff = (ep.frequency_hz - query_freq).abs();
                let resonance = 1.0 / (1.0 + freq_diff * 0.05);
                score += resonance * 0.5 + ep.arousal * 0.2;
                (score, ep)
            })
            .collect();

        scored.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap_or(std::cmp::Ordering::Equal));
        scored.into_iter().take(top_k).map(|(_, ep)| ep).collect()
    }

    /// Generate a grounded local response directly from the `.m8` Persona Soul (zero API keys)
    pub fn respond_locally(&self, prompt: &str) -> String {
        let recalled = self.recall_episodes(prompt, 4);
        let warmth_tone = if self.voice.warmth >= 0.15 {
            "warm and reflective"
        } else if self.voice.warmth <= -0.15 {
            "candid and earnest"
        } else {
            "thoughtful and measured"
        };

        let mut out = String::new();
        out.push_str(&format!(
            "{} **[{} · MEM8 Soul ({} episodes, {:.1} words/sentence, {} tone)]**\n\n",
            self.specialty.badge(),
            self.name,
            self.episodes.len(),
            self.voice.avg_sentence_words,
            warmth_tone
        ));

        if let Some(primary) = recalled.first() {
            let era_badge = primary
                .era
                .as_deref()
                .map(|e| format!(" ({e})"))
                .unwrap_or_default();
            let lead_label = match self.specialty {
                PersonaSpecialty::HistoryDocumenter => "Chronological record in",
                PersonaSpecialty::CodeAnalyst => "Architectural implementation in",
                PersonaSpecialty::ReadmeExpert => "Documentation reference in",
                PersonaSpecialty::LifeBiographer => "From my notes in",
            };
            out.push_str(&format!(
                "{} `{}`{}:\n> \"{}\"\n\n",
                lead_label,
                primary.source,
                era_badge,
                primary.passage.replace('\n', "\n> ")
            ));
        }

        if recalled.len() > 1 {
            let section_heading = match self.specialty {
                PersonaSpecialty::HistoryDocumenter => {
                    "**Timeline & connected historical entries:**\n"
                }
                PersonaSpecialty::CodeAnalyst => {
                    "**Related code paths & structural invariants:**\n"
                }
                PersonaSpecialty::ReadmeExpert => "**Related guides & usage notes:**\n",
                PersonaSpecialty::LifeBiographer => {
                    "**Connected memories that resonate with what you asked:**\n"
                }
            };
            out.push_str(section_heading);
            for ep in recalled.iter().skip(1) {
                let era_badge = ep
                    .era
                    .as_deref()
                    .map(|e| format!(" [{e}]"))
                    .unwrap_or_default();
                let snippet: String = ep.passage.chars().take(280).collect();
                out.push_str(&format!(
                    "- **`{}`**{}: \"{}{}\"\n",
                    ep.source,
                    era_badge,
                    snippet.replace('\n', " "),
                    if ep.passage.chars().count() > 280 {
                        "..."
                    } else {
                        ""
                    }
                ));
            }
            out.push('\n');
        }

        if !self.voice.signature_phrases.is_empty() {
            let phrases: Vec<&str> = self
                .voice
                .signature_phrases
                .iter()
                .take(6)
                .map(String::as_str)
                .collect();
            out.push_str(&format!("_Signature cadence: {}_", phrases.join(" · ")));
        }

        out
    }

    /// Build a grounding system prompt for augmenting an external or local LLM
    pub fn build_system_prompt(&self, user_query: &str) -> String {
        let recalled = self.recall_episodes(user_query, 5);
        let mut memories_block = String::new();
        for (idx, ep) in recalled.iter().enumerate() {
            let era = ep.era.as_deref().unwrap_or("undated");
            memories_block.push_str(&format!(
                "[Memory #{} | Source: {} | Era: {}]\n{}\n\n",
                idx + 1,
                ep.source,
                era,
                ep.passage
            ));
        }
        format!(
            "You are speaking as the MEM8 Persona '{}' ({}).\n\
             Voice profile: {:.1} words/sentence average, warmth={:.2}, expressiveness={:.2}.\n\
             Characteristic vocabulary: {}.\n\
             Signature phrases: {}.\n\
             Ground your answer strictly in the following authentic episodic memories from the corpus:\n\n{}",
            self.name,
            self.specialty.badge(),
            self.voice.avg_sentence_words,
            self.voice.warmth,
            self.voice.expressiveness,
            self.voice.vocabulary.iter().take(20).cloned().collect::<Vec<_>>().join(", "),
            self.voice.signature_phrases.iter().take(10).cloned().collect::<Vec<_>>().join(", "),
            memories_block
        )
    }
}

/// Ingest a single file in parallel, splitting long transcripts/documents into clean 300..850 char episodes
fn ingest_single_file(
    file_path: &Path,
    canon_root: &Path,
    specialty: PersonaSpecialty,
    per_file_cap: usize,
) -> Option<FileIngestStats> {
    let meta = fs::metadata(file_path).ok()?;
    if meta.len() == 0 || meta.len() > MAX_FILE_BYTES {
        return None;
    }
    let content = fs::read_to_string(file_path).ok()?;
    if content.trim().is_empty() {
        return None;
    }

    let rel_source = file_path
        .strip_prefix(canon_root)
        .unwrap_or(file_path)
        .to_string_lossy()
        .to_string();
    let source_label = if rel_source.is_empty() {
        file_path
            .file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_else(|| "document".to_string())
    } else {
        rel_source
    };

    let chunks = split_into_episodic_chunks(&content, specialty);
    if chunks.is_empty() {
        return None;
    }

    // If a single huge file has more chunks than `per_file_cap`, stride-sample evenly across the file
    // so beginning, middle, and end of the file are all represented.
    let step = if chunks.len() > per_file_cap && per_file_cap > 0 {
        chunks.len() / per_file_cap
    } else {
        1
    };

    let mut words_total = 0usize;
    let mut sentence_count = 0usize;
    let mut expressive_sentences = 0usize;
    let mut valence_sum = 0.0f32;
    let mut word_counts: HashMap<String, usize> = HashMap::new();
    let mut phrase_counts: HashMap<String, usize> = HashMap::new();
    let mut openings: Vec<String> = Vec::new();
    let mut episodes: Vec<PersonaEpisode> = Vec::new();

    for (idx, para) in chunks.iter().enumerate() {
        if is_machine_noise_passage(para) {
            continue;
        }
        let words: Vec<&str> = para.split_whitespace().collect();
        if words.len() < 6 {
            continue;
        }
        words_total += words.len();

        for sent in para.split(['.', '!', '?']) {
            let s = sent.trim();
            if s.split_whitespace().count() >= 4 {
                sentence_count += 1;
                if openings.len() < 4 && s.len() >= 25 && s.len() <= 140 {
                    openings.push(s.to_string());
                }
            }
        }
        expressive_sentences += para.matches(['!', '?']).count();

        // Natural alphabetic tokens for authentic voice/cadence extraction
        let natural_tokens: Vec<String> = words
            .iter()
            .map(|w| {
                w.trim_matches(|c: char| !c.is_ascii_alphabetic() && c != '\'')
                    .to_ascii_lowercase()
            })
            .filter(|w| is_natural_voice_word(w))
            .collect();

        for tok in &natural_tokens {
            *word_counts.entry(tok.clone()).or_insert(0) += 1;
        }
        for win in natural_tokens.windows(2) {
            let phrase = format!("{} {}", win[0], win[1]);
            *phrase_counts.entry(phrase).or_insert(0) += 1;
        }

        if idx % step == 0 && episodes.len() < per_file_cap {
            let (valence, arousal) = score_emotion(para);
            valence_sum += valence;
            let era = detect_date_or_era(para).or_else(|| detect_date_or_era(&source_label));

            // Keywords include both natural words and code/domain identifiers
            let mut kw: Vec<String> = Vec::new();
            for w in &words {
                let clean = w
                    .trim_matches(|c: char| !c.is_alphanumeric() && c != '_')
                    .to_ascii_lowercase();
                if clean.len() >= 4 && !is_stopword(&clean) && !kw.contains(&clean) {
                    kw.push(clean);
                    if kw.len() >= 12 {
                        break;
                    }
                }
            }

            let mut freq_hash = 0u64;
            for b in para.bytes() {
                freq_hash = freq_hash.wrapping_mul(31).wrapping_add(b as u64);
            }
            let frequency_hz = 20.0 + ((freq_hash % 200) as f32);

            episodes.push(PersonaEpisode {
                source: source_label.clone(),
                era,
                keywords: kw,
                valence,
                arousal,
                frequency_hz,
                passage: para.clone(),
            });
        }
    }

    if episodes.is_empty() {
        return None;
    }

    Some(FileIngestStats {
        words: words_total,
        sentences: sentence_count,
        expressive: expressive_sentences,
        valence_sum,
        word_counts,
        phrase_counts,
        openings,
        episodes,
    })
}

/// Split document content into clean ~100..850 character episodic chunks without mid-word truncation
fn split_into_episodic_chunks(content: &str, specialty: PersonaSpecialty) -> Vec<String> {
    let mut chunks = Vec::new();
    let min_len = if specialty == PersonaSpecialty::CodeAnalyst {
        24
    } else {
        32
    };

    for block in content.split("\n\n") {
        let trimmed = block.trim();
        if trimmed.len() < min_len {
            continue;
        }
        if trimmed.len() <= 850 {
            chunks.push(trimmed.to_string());
            continue;
        }
        // Long single-newline block (e.g. transcript, chat export, or source file):
        // group lines into ~400..800 char episodic passages.
        let mut current = String::new();
        for line in trimmed.lines() {
            let l = line.trim();
            if l.is_empty() {
                continue;
            }
            if !current.is_empty() && current.len() + l.len() + 1 > 800 {
                if current.len() >= min_len {
                    chunks.push(std::mem::take(&mut current));
                } else {
                    current.clear();
                }
            }
            if l.len() > 800 {
                // Split very long single lines on sentence boundaries
                for sent in l.split_inclusive(['.', '!', '?']) {
                    let s = sent.trim();
                    if s.is_empty() {
                        continue;
                    }
                    if !current.is_empty() && current.len() + s.len() + 1 > 800 {
                        if current.len() >= min_len {
                            chunks.push(std::mem::take(&mut current));
                        } else {
                            current.clear();
                        }
                    }
                    if !current.is_empty() {
                        current.push(' ');
                    }
                    current.push_str(s);
                }
            } else {
                if !current.is_empty() {
                    current.push('\n');
                }
                current.push_str(l);
            }
        }
        if current.len() >= min_len {
            chunks.push(current);
        }
    }
    chunks
}

/// Detect machine-generated log/JSON/LaTeX font noise so it doesn't pollute episodes or voice cadence
fn is_machine_noise_passage(para: &str) -> bool {
    let lower = para.to_ascii_lowercase();
    if lower.contains("avg_logprob")
        || lower.contains("no_speech_prob")
        || lower.contains("compression_ratio")
        || lower.contains("missing character: there is no")
        || lower.contains("mode=node;language=dflt")
    {
        return true;
    }
    let alpha_count = para.chars().filter(|c| c.is_alphabetic()).count();
    alpha_count * 3 < para.len()
}

/// Check whether a lowercase word is a clean natural-language word for voice/phrase signature extraction
fn is_natural_voice_word(w: &str) -> bool {
    if w.len() < 3 || w.len() > 22 {
        return false;
    }
    if !w.chars().all(|c| c.is_ascii_lowercase() || c == '\'')
        || w.starts_with('\'')
        || w.ends_with('\'')
    {
        return false;
    }
    !is_stopword(w)
        && !matches!(
            w,
            "avg"
                | "logprob"
                | "temperature"
                | "missing"
                | "character"
                | "font"
                | "fonts"
                | "ttf"
                | "otf"
                | "mode"
                | "node"
                | "language"
                | "dflt"
                | "tlig"
                | "http"
                | "https"
                | "www"
                | "com"
                | "org"
                | "json"
                | "null"
                | "true"
                | "false"
                | "string"
                | "number"
                | "object"
                | "array"
                | "boolean"
                | "div"
                | "span"
                | "class"
                | "style"
                | "href"
                | "dijo"
                | "chatgpt"
        )
}

fn is_stopword(w: &str) -> bool {
    matches!(
        w,
        "the"
            | "and"
            | "for"
            | "that"
            | "this"
            | "with"
            | "from"
            | "have"
            | "was"
            | "were"
            | "are"
            | "but"
            | "not"
            | "you"
            | "your"
            | "they"
            | "their"
            | "them"
            | "his"
            | "her"
            | "she"
            | "him"
            | "had"
            | "has"
            | "been"
            | "would"
            | "could"
            | "should"
            | "will"
            | "can"
            | "all"
            | "one"
            | "out"
            | "about"
            | "into"
            | "when"
            | "what"
            | "which"
            | "who"
            | "there"
            | "where"
            | "then"
            | "than"
            | "some"
            | "more"
            | "very"
            | "just"
            | "like"
            | "only"
            | "also"
            | "over"
            | "after"
            | "before"
            | "our"
            | "how"
            | "why"
            | "its"
            | "let"
            | "now"
            | "any"
            | "each"
            | "other"
            | "such"
            | "even"
            | "most"
            | "much"
            | "many"
            | "well"
            | "back"
            | "through"
            | "because"
            | "while"
            | "being"
            | "those"
            | "these"
    )
}

fn score_emotion(text: &str) -> (f32, f32) {
    let lower = text.to_lowercase();
    let pos_words = [
        "love",
        "loved",
        "happy",
        "joy",
        "wonderful",
        "grateful",
        "blessed",
        "sweet",
        "warm",
        "kind",
        "beautiful",
        "peace",
        "smile",
        "laugh",
        "hope",
        "good",
        "dear",
        "friend",
        "brilliant",
        "clarity",
        "insight",
    ];
    let neg_words = [
        "sad",
        "hard",
        "pain",
        "lost",
        "grief",
        "worried",
        "afraid",
        "cold",
        "lonely",
        "tired",
        "miss",
        "missed",
        "difficult",
        "storm",
        "broken",
        "hurt",
    ];
    let mut pos = 0usize;
    let mut neg = 0usize;
    for w in lower.split(|c: char| !c.is_alphanumeric()) {
        if pos_words.contains(&w) {
            pos += 1;
        } else if neg_words.contains(&w) {
            neg += 1;
        }
    }
    let total = (pos + neg).max(1) as f32;
    let valence = ((pos as f32 - neg as f32) / total).clamp(-1.0, 1.0);
    let exclam = text.matches(['!', '?']).count() as f32;
    let arousal = ((pos + neg) as f32 * 0.1 + exclam * 0.15).clamp(0.1, 1.0);
    (valence, arousal)
}

fn detect_date_or_era(text: &str) -> Option<String> {
    for token in
        text.split(|c: char| c.is_whitespace() || c == ',' || c == ';' || c == '(' || c == ')')
    {
        let t = token.trim_matches(|c: char| !c.is_ascii_alphanumeric() && c != '-');
        if t.len() == 10
            && t.as_bytes()[4] == b'-'
            && t.as_bytes()[7] == b'-'
            && t[..4].chars().all(|c| c.is_ascii_digit())
        {
            return Some(t.to_string());
        }
        if t.len() == 4 && t.chars().all(|c| c.is_ascii_digit()) {
            if let Ok(year) = t.parse::<u16>() {
                if (1850..=2035).contains(&year) {
                    return Some(year.to_string());
                }
            }
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn train_and_recall_life_persona_roundtrip() {
        let tmp = TempDir::new().unwrap();
        let diary = tmp.path().join("1958_summer_diary.md");
        fs::write(
            &diary,
            "# June 14, 1958\n\n\
             We planted the heirloom tomatoes and sweet basil by the cedar fence this morning. \
             Grandfather fixed the porch swing and we drank iced tea while listening to the rain.\n\n\
             # August 3, 1958\n\n\
             Baked cinnamon peach cobbler for the Sunday family gathering. Everyone loved the warm crust \
             and we sang old songs until dusk.",
        )
        .unwrap();

        let (persona, m8_path, size) =
            LifePersona::train_from_path("Grandmother", tmp.path()).unwrap();
        assert_eq!(persona.episodes.len(), 2);
        assert!(size > 0);
        assert!(m8_path.exists());

        let loaded = LifePersona::load_from_file(&m8_path).unwrap();
        assert_eq!(loaded.name, "Grandmother");
        let hits = loaded.recall_episodes("peach cobbler Sunday", 1);
        assert_eq!(hits.len(), 1);
        assert!(hits[0].passage.contains("peach cobbler"));

        let reply = loaded.respond_locally("Tell me about the peach cobbler");
        assert!(reply.contains("peach cobbler"));
    }
}
