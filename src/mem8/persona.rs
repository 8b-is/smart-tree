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

/// Authorship & voice provenance for an episodic memory so first-person diaries are remembered
/// as lived first-person experiences while inherited family writings preserve clear attribution.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "kebab-case")]
pub enum AuthorRole {
    /// First-person diary written by the persona as events unfolded ("I remember writing in my diary...")
    PrimaryDiary,
    /// First-person memoir, story, poem, or reflection written by the persona ("As I wrote in...")
    PrimaryMemoir,
    /// Maternal diary/archive preserved or edited by the persona ("From my mother's diary that I edited...")
    MaternalArchive,
    /// Paternal story/advice preserved by the persona ("From my dad's writings that I preserved...")
    PaternalArchive,
    /// Family dialogue or annotated exchange ("From a family exchange with...")
    FamilyDialogue,
    /// Standard corpus document or note
    #[default]
    GeneralCorpus,
}

/// Ancestral & biographical seed profile (e.g., `default_jody_seed()` for Grandma Jody)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AncestralSeed {
    /// Canonical full name
    pub id: String,
    /// Known aliases, barn names, and archive folder spellings
    pub aliases: Vec<String>,
    /// Birth year if known
    pub born_year: Option<u16>,
    /// Key places anchored in memory
    pub places: Vec<String>,
    /// Family members, companions, and beloved animals
    pub companions: Vec<String>,
    /// 8-band style wave modulation
    pub style_wave: [f32; 8],
    /// Warmth & care modulation bias
    pub value_adjust: f32,
    /// Domain salience weights [family/care, home/ranch, craft/writing, ethics/stewardship]
    pub domain_bias: [f32; 4],
    /// Provenance origin tag
    pub origin: String,
}

/// A single episodic memory passage extracted from a diary, letter, doc, or source file
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PersonaEpisode {
    /// Source file (relative to corpus root)
    pub source: String,
    /// Detected date or era label if present (e.g. "1963-01-02 (Wed)" or "1945")
    pub era: Option<String>,
    /// Lived event time when the experience occurred (e.g. "1963-01-02", "1939–1946")
    #[serde(default)]
    pub event_time: Option<String>,
    /// Written/compiled time when the document was penned or edited (e.g. "1963", "1998", "2009")
    #[serde(default)]
    pub written_time: Option<String>,
    /// Monotonic chronological sort key (YYYYMMDD, e.g. 19630102) for temporal traversal
    #[serde(default)]
    pub temporal_order: i32,
    /// Authorship & voice provenance role
    #[serde(default)]
    pub author_role: AuthorRole,
    /// Explicit provenance attribution string
    #[serde(default)]
    pub origin: String,
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
    /// Display name (e.g., "Grandma Jody" or "Nate")
    pub name: String,
    /// URL/filename-safe slug (e.g., "grandma-jody" or "nate")
    pub slug: String,
    /// Specialty lens
    #[serde(default)]
    pub specialty: PersonaSpecialty,
    /// Optional ancestral biographical seed (e.g. Alice Joan (Jody) Hawkins Chenoweth)
    #[serde(default)]
    pub ancestral_seed: Option<AncestralSeed>,
    /// Source path used for training
    pub source_path: String,
    /// When the persona was trained (imported/indexed time)
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
        #[cfg(test)]
        if source.starts_with(std::env::temp_dir()) {
            let test_dir = if source.is_dir() {
                source.join(".st_test_personas")
            } else {
                source.parent().unwrap_or(source).join(".st_test_personas")
            };
            fs::create_dir_all(&test_dir)?;
            return Ok(test_dir);
        }

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
    /// Canonical AncestralSeed for Grandma Jody ("Alice Joan (Jody) Hawkins Chenoweth" / "Alice Jody Hawkings")
    pub fn default_jody_seed() -> AncestralSeed {
        AncestralSeed {
            id: "Alice Joan (Jody) Hawkins Chenoweth".to_string(),
            aliases: vec![
                "Alice Jody Hawkings".to_string(),
                "Grandma Jody".to_string(),
                "Jody Chenoweth".to_string(),
                "Jody Hawkins".to_string(),
            ],
            born_year: Some(1930),
            places: vec![
                "Platoro, Colorado (X3 Lake Fork Dude Ranch & Skyview Lodge)".to_string(),
                "Vallecito, Colorado".to_string(),
                "Crestone & Great Sand Dunes, Colorado".to_string(),
                "Kansas City, Kansas (Wyandotte High & Triple Star Riding Stables)".to_string(),
                "Chenterra Farm (between Tonganoxie & Leavenworth, Kansas)".to_string(),
            ],
            companions: vec![
                "Fred Chenoweth (husband, m. 1948)".to_string(),
                "Alice Wilson Hawkins (mother)".to_string(),
                "Mike Hawkins (father)".to_string(),
                "Judy Stauffer & Susie Ross (sisters)".to_string(),
                "Mike, Kane, Kerry, and Jill (children)".to_string(),
                "Buster (ornery black Shetland pony)".to_string(),
                "Chenterra Phoebus Dinkums ('Dink', 27.5\" AMHA herd sire)".to_string(),
                "Ch. Chenterra Thunderation ROM ('Thunder', top-winning Sheltie)".to_string(),
            ],
            style_wave: [0.18, 0.25, -0.05, 0.22, 0.14, 0.08, 0.19, 0.11],
            value_adjust: 0.24,
            domain_bias: [0.25, 0.15, 0.08, 0.22],
            origin: "Jody".to_string(),
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
                    let lower_fname = fname.to_ascii_lowercase();
                    !fname.starts_with('.')
                        && fname != "node_modules"
                        && fname != "target"
                        && fname != "venv"
                        && fname != "__pycache__"
                        && fname != "dist"
                        && fname != "build"
                        && fname != "vendor"
                        && lower_fname != "backup_originals"
                })
                .filter_map(|e| e.ok())
                .filter(|e| e.path().is_file())
                .filter(|e| {
                    let path = e.path();
                    let fname = path
                        .file_name()
                        .map(|f| f.to_string_lossy().to_ascii_lowercase())
                        .unwrap_or_default();
                    // Never ingest dependency lockfiles, minified bundles, or third-party background reports
                    if matches!(
                        fname.as_str(),
                        "package-lock.json"
                            | "cargo.lock"
                            | "yarn.lock"
                            | "pnpm-lock.yaml"
                            | "composer.lock"
                            | "poetry.lock"
                            | "bun.lockb"
                            | "next_steps.md"
                    ) || fname.ends_with(".min.js")
                        || fname.ends_with(".min.css")
                        || fname.contains("instantcheckmate")
                        || fname.contains("6001_wax_ball")
                        || fname.contains("webcollex")
                    {
                        return false;
                    }
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
        let per_file_cap = (MAX_PERSONA_EPISODES / file_count).clamp(200, 25_000);

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
        let mut seen_passage_hashes: HashSet<u64> = HashSet::new();

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
                    let mut p_hash = 0xcbf29ce484222325u64;
                    for b in ep.passage.bytes() {
                        if !b.is_ascii_whitespace() {
                            p_hash ^= b.to_ascii_lowercase() as u64;
                            p_hash = p_hash.wrapping_mul(0x100000001b3);
                        }
                    }
                    if seen_passage_hashes.insert(p_hash) {
                        episodes.push(ep);
                    }
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

        let is_jody = slug.contains("jody")
            || canon
                .to_string_lossy()
                .to_ascii_lowercase()
                .contains("jody");
        let ancestral_seed = if is_jody {
            Some(Self::default_jody_seed())
        } else {
            None
        };

        let persona = Self {
            name: name.to_string(),
            slug: slug.clone(),
            specialty,
            ancestral_seed,
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

    /// Retrieve top-K resonant episodic memories for a query using parallel wave + temporal + keyword salience
    pub fn recall_episodes(&self, query: &str, top_k: usize) -> Vec<&PersonaEpisode> {
        let lower_query = query.to_ascii_lowercase();
        let query_tokens: Vec<String> = query
            .split_whitespace()
            .map(|w| {
                w.trim_matches(|c: char| !c.is_alphanumeric())
                    .to_lowercase()
            })
            .filter(|w| w.len() >= 3 && !is_stopword(w))
            .collect();

        let wants_diary = lower_query.contains("diary")
            || lower_query.contains("wrote")
            || lower_query.contains("write")
            || lower_query.contains("journal")
            || lower_query.contains("day in")
            || lower_query.contains("1963")
            || lower_query.contains("1964");
        let wants_mom = lower_query.contains("mom")
            || lower_query.contains("mother")
            || lower_query.contains("mama")
            || lower_query.contains("alice wilson");
        let wants_dad = lower_query.contains("dad")
            || lower_query.contains("father")
            || lower_query.contains("daddy")
            || lower_query.contains("mike hawkins");
        let wants_jill = lower_query.contains("jill") || lower_query.contains("traits");

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
                let lower_passage = ep.passage.to_ascii_lowercase();
                let lower_source = ep.source.to_ascii_lowercase();

                for qt in &query_tokens {
                    if ep.keywords.iter().any(|k| k == qt) {
                        score += 5.0;
                        matched_any = true;
                    } else if lower_passage.contains(qt.as_str()) {
                        score += 2.5;
                        matched_any = true;
                    }
                    if lower_source.contains(qt.as_str()) {
                        score += 2.5;
                        matched_any = true;
                    }
                    if let Some(ref era) = ep.era {
                        if era.to_ascii_lowercase().contains(qt.as_str()) {
                            score += 5.5;
                            matched_any = true;
                        }
                    }
                    if let Some(ref ev) = ep.event_time {
                        if ev.to_ascii_lowercase().contains(qt.as_str()) {
                            score += 5.0;
                            matched_any = true;
                        }
                    }
                }

                // Voice & authorship alignment
                match ep.author_role {
                    AuthorRole::PrimaryDiary => {
                        score += if wants_diary { 6.0 } else { 1.5 };
                    }
                    AuthorRole::PrimaryMemoir => {
                        score += 1.2;
                    }
                    AuthorRole::MaternalArchive => {
                        if wants_mom {
                            score += 5.5;
                        }
                    }
                    AuthorRole::PaternalArchive => {
                        if wants_dad {
                            score += 5.5;
                        }
                    }
                    AuthorRole::FamilyDialogue => {
                        if wants_jill {
                            score += 5.5;
                        }
                    }
                    AuthorRole::GeneralCorpus => {}
                }

                if is_history && ep.era.is_some() {
                    score += if matched_any { 2.5 } else { 0.4 };
                }
                // Temporal richness bonus when an episode has an explicit lived date
                if ep.temporal_order > 0 && matched_any {
                    score += 0.8;
                }
                // Narrative substance bonus: prefer rich paragraphs over short headings/fragments
                if matched_any {
                    let plen = ep.passage.len();
                    if plen >= 160 {
                        score += 2.8;
                    } else if plen < 90 {
                        score -= 2.5;
                    }
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
    /// with temporal memory awareness (`event_time` vs `written_time`) and first-person diary recall.
    pub fn respond_locally(&self, prompt: &str) -> String {
        let recalled = self.recall_episodes(prompt, 5);
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

        if let Some(ref seed) = self.ancestral_seed {
            out.push_str(&format!(
                "_Identity anchor: **{}** (b. {}) · Places: {}_\n\n",
                seed.id,
                seed.born_year
                    .map(|y| y.to_string())
                    .unwrap_or_else(|| "unknown".to_string()),
                seed.places
                    .iter()
                    .take(3)
                    .cloned()
                    .collect::<Vec<_>>()
                    .join(" · ")
            ));
        }

        if let Some(primary) = recalled.first() {
            let temporal_badge = format_temporal_badge(primary);
            let lead_header = match (self.specialty, primary.author_role) {
                (_, AuthorRole::PrimaryDiary) => format!(
                    "📓 **I remember writing in my diary{}** (`{}` — *first-person lived diary*)",
                    temporal_badge, primary.source
                ),
                (_, AuthorRole::PrimaryMemoir) => format!(
                    "✍️ **As I wrote in `{}`**{}",
                    primary.source, temporal_badge
                ),
                (_, AuthorRole::MaternalArchive) => format!(
                    "📜 **From my mother Alice Wilson Hawkins's diary, which I edited** (`{}`{})",
                    primary.source, temporal_badge
                ),
                (_, AuthorRole::PaternalArchive) => format!(
                    "📜 **From my dad Mike Hawkins's story & advice that I preserved** (`{}`{})",
                    primary.source, temporal_badge
                ),
                (_, AuthorRole::FamilyDialogue) => format!(
                    "💌 **From a family exchange with my daughter Jill** (`{}`{})",
                    primary.source, temporal_badge
                ),
                (PersonaSpecialty::HistoryDocumenter, _) => {
                    format!(
                        "Chronological record in `{}`{}",
                        primary.source, temporal_badge
                    )
                }
                (PersonaSpecialty::CodeAnalyst, _) => {
                    format!(
                        "Architectural implementation in `{}`{}",
                        primary.source, temporal_badge
                    )
                }
                (PersonaSpecialty::ReadmeExpert, _) => {
                    format!(
                        "Documentation reference in `{}`{}",
                        primary.source, temporal_badge
                    )
                }
                (PersonaSpecialty::LifeBiographer, _) => {
                    format!("From my notes in `{}`{}", primary.source, temporal_badge)
                }
            };
            out.push_str(&format!(
                "{}:\n> \"{}\"\n\n",
                lead_header,
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
                    "**Connected memories across time (ordered chronologically):**\n"
                }
            };
            out.push_str(section_heading);

            // Order supporting memories chronologically when temporal anchors exist
            let mut supporting: Vec<&PersonaEpisode> = recalled.iter().skip(1).copied().collect();
            if self.specialty == PersonaSpecialty::LifeBiographer
                || self.specialty == PersonaSpecialty::HistoryDocumenter
            {
                supporting.sort_by_key(|ep| {
                    if ep.temporal_order > 0 {
                        ep.temporal_order
                    } else {
                        i32::MAX
                    }
                });
            }

            for ep in supporting {
                let role_tag = match ep.author_role {
                    AuthorRole::PrimaryDiary => "📓 *My diary*",
                    AuthorRole::PrimaryMemoir => "✍️ *My writing*",
                    AuthorRole::MaternalArchive => "📜 *Mom's diary (edited by me)*",
                    AuthorRole::PaternalArchive => "📜 *Dad's writing*",
                    AuthorRole::FamilyDialogue => "💌 *With Jill*",
                    AuthorRole::GeneralCorpus => "📄 *Note*",
                };
                let t_badge = format_temporal_badge(ep);
                let snippet: String = ep.passage.chars().take(280).collect();
                out.push_str(&format!(
                    "- {} · **`{}`**{}: \"{}{}\"\n",
                    role_tag,
                    ep.source,
                    t_badge,
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
            let t_badge = format_temporal_badge(ep);
            let origin = if ep.origin.is_empty() {
                "Primary corpus"
            } else {
                ep.origin.as_str()
            };
            memories_block.push_str(&format!(
                "[Memory #{} | Source: {} | Provenance: {}{}]\n{}\n\n",
                idx + 1,
                ep.source,
                origin,
                t_badge,
                ep.passage
            ));
        }
        format!(
            "You are speaking as the MEM8 Persona '{}' ({}).\n\
             Voice profile: {:.1} words/sentence average, warmth={:.2}, expressiveness={:.2}.\n\
             Characteristic vocabulary: {}.\n\
             Signature phrases: {}.\n\
             Temporal & Attribution Rules:\n\
             1. If a memory's provenance is PrimaryDiary or PrimaryMemoir, remember and speak of it in the first person as your own lived diary and writing.\n\
             2. Distinguish lived event time from written/compiled time and never confuse file import dates with lived dates.\n\
             3. If a memory comes from your mother's diary (MaternalArchive), father's story (PaternalArchive), or daughter's annotations (FamilyDialogue), credit them warmly and accurately.\n\n\
             Ground your answer strictly in the following authentic episodic memories:\n\n{}",
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

fn format_temporal_badge(ep: &PersonaEpisode) -> String {
    match (&ep.event_time, &ep.written_time, &ep.era) {
        (Some(ev), Some(wr), _) if ev != wr && !ev.starts_with(wr.as_str()) => {
            format!(" [lived {ev}, written {wr}]")
        }
        (Some(ev), _, _) => format!(" [{ev}]"),
        (None, Some(wr), _) => format!(" [written {wr}]"),
        (None, None, Some(era)) => format!(" [{era}]"),
        (None, None, None) => String::new(),
    }
}

/// Classify authorship and provenance role for a document and passage
fn classify_author_and_origin(
    source_label: &str,
    full_content_header: &str,
    passage: &str,
) -> (AuthorRole, String, Option<String>) {
    let lower_src = source_label.to_ascii_lowercase();
    let lower_hdr = full_content_header.to_ascii_lowercase();
    let lower_pass = passage.to_ascii_lowercase();

    // 1. First-person diaries written by Jody (or any primary diarist)
    if lower_src.contains("bestdiary")
        || lower_src.contains("my_diary")
        || lower_src.contains("my diary")
        || lower_src.contains("diary")
            && !lower_src.contains("short diary")
            && !lower_src.contains("colorado diary")
        || lower_hdr.contains("the property of\njody chenoweth")
        || lower_hdr.contains("yearslong diary")
    {
        let written = if lower_src.contains("19what") || lower_hdr.contains("january 1964") {
            Some("1964".to_string())
        } else if lower_hdr.contains("1963") {
            Some("1963".to_string())
        } else {
            detect_date_or_era(source_label)
        };
        return (
            AuthorRole::PrimaryDiary,
            "Jody Chenoweth (First-Person Lived Diary)".to_string(),
            written,
        );
    }

    // 2. Family dialogue with daughter Jill
    if lower_src.contains("jill") {
        return (
            AuthorRole::FamilyDialogue,
            "Jody Chenoweth & daughter Jill".to_string(),
            None,
        );
    }

    // 3. Paternal writings (Mike Hawkins / Dad's story & advice)
    if lower_src.contains("dad")
        || lower_src.contains("daddy")
        || lower_src.contains("o_hell_hawkins")
        || lower_src.contains("o hell hawkins")
    {
        let written = if lower_src.contains("2014") {
            Some("2014".to_string())
        } else {
            None
        };
        return (
            AuthorRole::PaternalArchive,
            "Mike Hawkins (Dad's Story & Advice, preserved by Jody)".to_string(),
            written,
        );
    }

    // 4. Maternal diary & advice (Alice Wilson Hawkins, edited with Epilog by Jody)
    if lower_src.contains("short diary")
        || lower_src.contains("colorado diary")
        || lower_src.contains("colorado prospecting")
        || lower_src.contains("moms book")
        || lower_src.contains("moms advice")
        || lower_src.contains("mama")
        || lower_hdr.contains("diary of alice wilson hawkins")
    {
        let written = if lower_src.contains("2010") {
            Some("2010".to_string())
        } else if lower_hdr.contains("2009932375") {
            Some("2009".to_string())
        } else {
            None
        };
        // If this specific passage is Jody's own dedication, NOTE, or Epilog, mark as PrimaryMemoir
        if lower_pass.contains("this book is dedicated to")
            || lower_pass.contains("epilog by jody")
            || lower_pass.contains("edited by jody")
            || lower_pass.starts_with("note this edition")
        {
            return (
                AuthorRole::PrimaryMemoir,
                "Jody Hawkins Chenoweth (Editor's Dedication & Epilog)".to_string(),
                written,
            );
        }
        return (
            AuthorRole::MaternalArchive,
            "Alice Wilson Hawkins (Mom's Diary, edited by Jody)".to_string(),
            written,
        );
    }

    // 5. Jody's own memoirs, stories, poems, and Gems of Thought
    if lower_src.contains("our story")
        || lower_src.contains("chenterra")
        || lower_src.contains("first 15 years")
        || lower_src.contains("raamblings")
        || lower_src.contains("patchwork of poetry")
        || lower_src.contains("gems_of_thought")
        || lower_src.contains("glory_book")
        || lower_src.contains("match_race")
        || lower_src.contains("widder_maude")
    {
        let written =
            if lower_src.contains("our story") || lower_pass.contains("50th wedding anniversary") {
                Some("1998".to_string())
            } else {
                None
            };
        return (
            AuthorRole::PrimaryMemoir,
            "Jody Hawkins Chenoweth (First-Person Memoir, Stories & Poetry)".to_string(),
            written,
        );
    }

    (
        AuthorRole::GeneralCorpus,
        "Primary Corpus".to_string(),
        None,
    )
}

/// Ingest a single file in parallel, splitting long transcripts/documents into clean 300..850 char episodes
/// while tracking stateful chronological dates (`event_time`, `written_time`, `temporal_order`).
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

    let header_preview: String = content.chars().take(1200).collect();
    let (file_role, file_origin, file_written_time) =
        classify_author_and_origin(&source_label, &header_preview, "");

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

    // Running temporal context across the file (so daily diary entries like "January 2 Wed."
    // inherit the diary's year from the title page or preceding section header).
    let mut running_year: Option<u16> = detect_initial_year(&source_label, &header_preview);
    let mut running_month: Option<u8> = None;

    for (idx, para) in chunks.iter().enumerate() {
        // Update running year/month even on short or skipped chunks so temporal context stays accurate
        if let Some(y) = detect_lived_year_in_text(para) {
            running_year = Some(y);
        }
        let parsed_day = detect_month_day_in_text(para, running_month);
        if let Some((m, _, _)) = parsed_day {
            running_month = Some(m);
        }

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

            let (ep_role, ep_origin, ep_written) =
                classify_author_and_origin(&source_label, &header_preview, para);
            let role = if ep_role != AuthorRole::GeneralCorpus {
                ep_role
            } else {
                file_role
            };
            let origin = if ep_role != AuthorRole::GeneralCorpus {
                ep_origin
            } else {
                file_origin.clone()
            };
            let written_time = ep_written.or_else(|| file_written_time.clone());

            let (event_time, temporal_order) =
                resolve_episode_temporal(para, &source_label, running_year, parsed_day);
            let era = event_time.clone().or_else(|| written_time.clone());

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
                event_time,
                written_time,
                temporal_order,
                author_role: role,
                origin,
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

/// Split document content into clean ~100..850 character episodic chunks without mid-word truncation,
/// keeping short diary date headers (e.g. `"January 2 Wed."`) attached to their diary entry body.
fn split_into_episodic_chunks(content: &str, specialty: PersonaSpecialty) -> Vec<String> {
    let mut chunks = Vec::new();
    let min_len = if specialty == PersonaSpecialty::CodeAnalyst {
        24
    } else {
        32
    };

    let mut pending_header = String::new();

    for raw_block in content.split("\n\n") {
        let trimmed = raw_block.trim_matches(|c: char| c.is_whitespace() || c == '\u{000c}');
        if trimmed.is_empty() {
            continue;
        }
        // If this block is a short date/section header (e.g. "January 2 Wed." or "Tues.\nUp at 7:00 am"),
        // hold it and prepend it to the next paragraph so diary dates are never orphaned!
        if trimmed.len() < min_len || is_short_date_header(trimmed) {
            if trimmed.len() <= 90 {
                if !pending_header.is_empty() {
                    pending_header.push(' ');
                }
                pending_header.push_str(&trimmed.replace('\n', " "));
                if pending_header.len() > 160 {
                    pending_header.clear();
                }
            }
            continue;
        }

        let block_with_hdr = if !pending_header.is_empty() {
            let hdr = std::mem::take(&mut pending_header);
            format!("{hdr} — {trimmed}")
        } else {
            trimmed.to_string()
        };

        if block_with_hdr.len() <= 850 {
            chunks.push(block_with_hdr);
            continue;
        }
        // Long single-newline block (e.g. transcript, chat export, or source file):
        // group lines into ~400..800 char episodic passages.
        let mut current = String::new();
        for line in block_with_hdr.lines() {
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

fn is_short_date_header(text: &str) -> bool {
    if text.len() > 64 {
        return false;
    }
    detect_month_day_in_text(text, None).is_some()
        || matches!(
            text.trim().to_ascii_lowercase().as_str(),
            "mon." | "tues." | "wed." | "thurs." | "fri." | "sat." | "sun."
        )
}

fn detect_initial_year(source_label: &str, header: &str) -> Option<u16> {
    let lower_src = source_label.to_ascii_lowercase();
    let lower_hdr = header.to_ascii_lowercase();
    if lower_src.contains("19what") || lower_hdr.contains("january 1964") {
        return Some(1964);
    }
    if lower_hdr.contains("january 1, 1963") || lower_src.contains("bestdiary") {
        return Some(1963);
    }
    if lower_src.contains("glory_book") || lower_src.contains("match_race") {
        return Some(1945);
    }
    detect_lived_year_in_text(header).or_else(|| detect_lived_year_in_text(source_label))
}

fn detect_lived_year_in_text(text: &str) -> Option<u16> {
    let lower = text.to_ascii_lowercase();
    // Ignore copyright/ISBN/control lines when inferring lived year
    if lower.contains("copyright")
        || lower.contains("isbn")
        || lower.contains("library of congress")
    {
        return None;
    }
    for token in text.split(|c: char| !c.is_ascii_alphanumeric()) {
        // Handle OCR "l998" / "l989" where '1' was scanned as lowercase 'l'
        let normalized = if token.len() == 4
            && token.starts_with('l')
            && token[1..].chars().all(|c| c.is_ascii_digit())
        {
            format!("1{}", &token[1..])
        } else {
            token.to_string()
        };
        if normalized.len() == 4 && normalized.chars().all(|c| c.is_ascii_digit()) {
            if let Ok(year) = normalized.parse::<u16>() {
                if (1880..=2026).contains(&year) {
                    return Some(year);
                }
            }
        }
    }
    None
}

fn parse_month_name(word: &str) -> Option<u8> {
    match word.trim_matches('.').to_ascii_lowercase().as_str() {
        "january" | "jan" => Some(1),
        "february" | "feb" => Some(2),
        "march" | "mar" => Some(3),
        "april" | "apr" => Some(4),
        "may" => Some(5),
        "june" | "jun" => Some(6),
        "july" | "jul" => Some(7),
        "august" | "aug" => Some(8),
        "september" | "sep" | "sept" => Some(9),
        "october" | "oct" => Some(10),
        "november" | "nov" => Some(11),
        "december" | "dec" => Some(12),
        _ => None,
    }
}

fn parse_weekday(word: &str) -> Option<&'static str> {
    match word
        .trim_matches(|c: char| !c.is_ascii_alphabetic())
        .to_ascii_lowercase()
        .as_str()
    {
        "mon" | "monday" => Some("Mon"),
        "tue" | "tues" | "tuesday" => Some("Tue"),
        "wed" | "wednesday" => Some("Wed"),
        "thu" | "thur" | "thurs" | "thursday" => Some("Thu"),
        "fri" | "friday" => Some("Fri"),
        "sat" | "saturday" => Some("Sat"),
        "sun" | "sunday" => Some("Sun"),
        _ => None,
    }
}

/// Detect `"January 2 Wed."`, `"Jan. 18 Sat."`, `"June 21st"`, etc. in the first 120 chars of a chunk
fn detect_month_day_in_text(
    text: &str,
    _fallback_month: Option<u8>,
) -> Option<(u8, u8, Option<&'static str>)> {
    let prefix: String = text.chars().take(120).collect();
    let tokens: Vec<&str> = prefix
        .split(|c: char| c.is_whitespace() || c == ',' || c == '—' || c == '-')
        .filter(|s| !s.is_empty())
        .collect();
    for i in 0..tokens.len() {
        if let Some(month) = parse_month_name(tokens[i]) {
            if let Some(&next) = tokens.get(i + 1) {
                let digits: String = next.chars().take_while(|c| c.is_ascii_digit()).collect();
                if let Ok(day) = digits.parse::<u8>() {
                    if (1..=31).contains(&day) {
                        let wk = tokens
                            .get(i + 2)
                            .and_then(|w| parse_weekday(w))
                            .or_else(|| tokens.get(i + 3).and_then(|w| parse_weekday(w)));
                        return Some((month, day, wk));
                    }
                }
            }
        }
    }
    None
}

fn resolve_episode_temporal(
    para: &str,
    source_label: &str,
    running_year: Option<u16>,
    parsed_day: Option<(u8, u8, Option<&'static str>)>,
) -> (Option<String>, i32) {
    // 1. Check explicit ISO date first
    if let Some(iso) = detect_date_or_era(para) {
        if iso.len() == 10 {
            let y: i32 = iso[0..4].parse().unwrap_or(0);
            let m: i32 = iso[5..7].parse().unwrap_or(0);
            let d: i32 = iso[8..10].parse().unwrap_or(0);
            return (Some(iso), y * 10_000 + m * 100 + d);
        }
    }

    let year_opt = detect_lived_year_in_text(para)
        .or(running_year)
        .or_else(|| detect_lived_year_in_text(source_label));

    if let (Some(year), Some((month, day, wk_opt))) = (year_opt, parsed_day) {
        let label = match wk_opt {
            Some(wk) => format!("{year:04}-{month:02}-{day:02} ({wk})"),
            None => format!("{year:04}-{month:02}-{day:02}"),
        };
        let sort_key = (year as i32) * 10_000 + (month as i32) * 100 + (day as i32);
        return (Some(label), sort_key);
    }

    if let Some(year) = year_opt {
        return (Some(year.to_string()), (year as i32) * 10_000);
    }

    (None, 0)
}

/// Detect machine-generated log/JSON/LaTeX font noise so it doesn't pollute episodes or voice cadence
fn is_machine_noise_passage(para: &str) -> bool {
    let lower = para.to_ascii_lowercase();
    if lower.contains("avg_logprob")
        || lower.contains("no_speech_prob")
        || lower.contains("compression_ratio")
        || lower.contains("missing character: there is no")
        || lower.contains("mode=node;language=dflt")
        || lower.contains("\"integrity\": \"sha512-")
        || lower.contains("\"resolved\": \"https://registry.npmjs.org/")
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

    #[test]
    fn diary_temporal_and_first_person_recall_works() {
        let tmp = TempDir::new().unwrap();
        let diary = tmp.path().join("BESTDIARY58 last oneMay13.md");
        fs::write(
            &diary,
            "THE PROPERTY OF\nJody Chenoweth\n\nYEARSLONG DIARY\nFrom January 1, 1963 to January 1, 1964\n\n\
             January 2 Wed.\n\n\
             Fred worked graveyards last night. Kerry and I went to town and exchanged Kane's boots. \
             Evening we played Password and Fred fixed Chatty Cathy.\n\n\
             January 3 Thurs.\n\n\
             Baked a pineapple upside down cake and brought Shorty home to the pasture.",
        )
        .unwrap();

        let (persona, _, _) = LifePersona::train_from_path("Grandma Jody", tmp.path()).unwrap();
        assert!(persona.ancestral_seed.is_some());
        let hits = persona.recall_episodes("pineapple upside down cake January 1963", 1);
        assert_eq!(hits.len(), 1);
        assert_eq!(hits[0].author_role, AuthorRole::PrimaryDiary);
        assert_eq!(hits[0].event_time.as_deref(), Some("1963-01-03 (Thu)"));
        assert_eq!(hits[0].temporal_order, 19630103);

        let reply = persona.respond_locally("What did you bake in your diary in January 1963?");
        assert!(reply.contains("I remember writing in my diary"));
        assert!(reply.contains("1963-01-03 (Thu)"));
    }
}
