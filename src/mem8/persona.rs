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

    /// Build an `AncestralSeed` dynamically from any corpus using SPQT Base-243 ternary wave projection
    /// and structural entity/place/date discovery (zero hardcoded names).
    pub fn build_ternary_ancestral_seed(
        persona_name: &str,
        episodes: &[PersonaEpisode],
        voice: &VoiceSignature,
    ) -> AncestralSeed {
        let mut aliases: Vec<String> = Vec::new();
        let mut places_counts: HashMap<String, usize> = HashMap::new();
        let mut companion_counts: HashMap<String, usize> = HashMap::new();
        let mut born_year: Option<u16> = None;

        let name_tokens: HashSet<String> = persona_name
            .split_whitespace()
            .map(|t| {
                t.trim_matches(|c: char| !c.is_alphabetic())
                    .to_ascii_lowercase()
            })
            .filter(|t| t.len() >= 3)
            .collect();

        for ep in episodes {
            let text = &ep.passage;
            let lower = text.to_ascii_lowercase();

            // 1. Discover birth year from explicit "born ... YYYY" patterns
            if born_year.is_none() {
                if let Some(pos) = lower.find("born ") {
                    let window: String = text[pos..].chars().take(48).collect();
                    if let Some(y) = detect_lived_year_in_text(&window) {
                        if (1850..=2015).contains(&y) {
                            born_year = Some(y);
                        }
                    }
                }
            }

            // 2. Discover epistolary salutations/signatures ("Dearest X", "Yours, X", "THE PROPERTY OF X")
            for line in text.lines() {
                let l = line.trim();
                for prefix in ["Yours,", "Dearest ", "Dear ", "THE PROPERTY OF "] {
                    if let Some(rest) = l.strip_prefix(prefix) {
                        let clean = rest
                            .trim_matches(|c: char| !c.is_alphanumeric() && c != ' ' && c != '.')
                            .trim();
                        if clean.len() >= 3
                            && clean.len() <= 32
                            && clean.chars().next().is_some_and(|c| c.is_ascii_uppercase())
                            && !aliases.iter().any(|a| a.eq_ignore_ascii_case(clean))
                        {
                            if prefix == "Dearest " || prefix == "Dear " {
                                *companion_counts
                                    .entry(format!("{clean} (diary confidante)"))
                                    .or_insert(0) += 4;
                            } else {
                                aliases.push(clean.to_string());
                            }
                        }
                    }
                }
            }

            // 3. Discover places following spatial prepositions ("in ", "at ", "near ", "to ") or place suffixes
            let tokens: Vec<&str> = text.split_whitespace().collect();
            for i in 0..tokens.len() {
                let w_low = tokens[i]
                    .trim_matches(|c: char| !c.is_ascii_alphabetic())
                    .to_ascii_lowercase();
                if matches!(w_low.as_str(), "in" | "at" | "near" | "from") {
                    if let Some(&next) = tokens.get(i + 1) {
                        let clean_next = next.trim_matches(|c: char| !c.is_alphanumeric());
                        if clean_next.len() >= 4
                            && clean_next
                                .chars()
                                .next()
                                .is_some_and(|c| c.is_ascii_uppercase())
                            && !is_calendar_or_common_cap(&clean_next.to_ascii_lowercase())
                        {
                            let mut phrase = clean_next.to_string();
                            if let Some(&next2) = tokens.get(i + 2) {
                                let clean2 = next2.trim_matches(|c: char| !c.is_alphanumeric());
                                if clean2
                                    .chars()
                                    .next()
                                    .is_some_and(|c| c.is_ascii_uppercase())
                                    && !is_calendar_or_common_cap(&clean2.to_ascii_lowercase())
                                {
                                    phrase.push(' ');
                                    phrase.push_str(clean2);
                                }
                            }
                            *places_counts.entry(phrase).or_insert(0) += 1;
                        }
                    }
                }

                // 4. Discover recurring capitalized companion names in first-person episodes
                if matches!(
                    ep.author_role,
                    AuthorRole::PrimaryDiary | AuthorRole::PrimaryMemoir
                ) {
                    let clean_cap = tokens[i].trim_matches(|c: char| !c.is_ascii_alphabetic());
                    if clean_cap.len() >= 4
                        && clean_cap
                            .chars()
                            .next()
                            .is_some_and(|c| c.is_ascii_uppercase())
                        && clean_cap[1..].chars().all(|c| c.is_ascii_lowercase())
                    {
                        let low = clean_cap.to_ascii_lowercase();
                        if !is_calendar_or_common_cap(&low) && !name_tokens.contains(&low) {
                            *companion_counts.entry(clean_cap.to_string()).or_insert(0) += 1;
                        }
                    }
                }
            }
        }

        let mut place_vec: Vec<(String, usize)> =
            places_counts.into_iter().filter(|(_, c)| *c >= 2).collect();
        place_vec.sort_by_key(|b| std::cmp::Reverse(b.1));
        let places: Vec<String> = place_vec.into_iter().take(5).map(|(p, _)| p).collect();

        let mut comp_vec: Vec<(String, usize)> = companion_counts
            .into_iter()
            .filter(|(_, c)| *c >= 3)
            .collect();
        comp_vec.sort_by_key(|b| std::cmp::Reverse(b.1));
        let companions: Vec<String> = comp_vec.into_iter().take(6).map(|(c, _)| c).collect();

        // SPQT Base-243 ternary style wave projection (quantize 10 corpus features into 2 Base-243 bytes)
        let n_eps = episodes.len().max(1) as f32;
        let diary_ratio = episodes
            .iter()
            .filter(|e| e.author_role == AuthorRole::PrimaryDiary)
            .count() as f32
            / n_eps;
        let memoir_ratio = episodes
            .iter()
            .filter(|e| e.author_role == AuthorRole::PrimaryMemoir)
            .count() as f32
            / n_eps;
        let temporal_ratio =
            episodes.iter().filter(|e| e.temporal_order > 0).count() as f32 / n_eps;

        let raw_features: [f32; 8] = [
            voice.warmth,
            voice.expressiveness,
            (voice.avg_sentence_words - 14.0) / 14.0,
            diary_ratio - 0.3,
            memoir_ratio - 0.3,
            temporal_ratio - 0.4,
            voice.warmth * 0.5 + diary_ratio * 0.5,
            voice.expressiveness * 0.5 + temporal_ratio * 0.5,
        ];
        let mut trits_a = [0i8; 5];
        let mut trits_b = [0i8; 5];
        for i in 0..5 {
            trits_a[i] = if raw_features[i] > 0.04 {
                1
            } else if raw_features[i] < -0.04 {
                -1
            } else {
                0
            };
        }
        for i in 0..3 {
            trits_b[i] = if raw_features[5 + i] > 0.04 {
                1
            } else if raw_features[5 + i] < -0.04 {
                -1
            } else {
                0
            };
        }
        let packed_a = spqt_pack_5trits(trits_a);
        let packed_b = spqt_pack_5trits(trits_b);
        let un_a = spqt_unpack_5trits(packed_a);
        let un_b = spqt_unpack_5trits(packed_b);
        let style_wave = [
            (un_a[0] as f32) * 0.22 + raw_features[0] * 0.1,
            (un_a[1] as f32) * 0.22 + raw_features[1] * 0.1,
            (un_a[2] as f32) * 0.22 + raw_features[2] * 0.1,
            (un_a[3] as f32) * 0.22 + raw_features[3] * 0.1,
            (un_a[4] as f32) * 0.22 + raw_features[4] * 0.1,
            (un_b[0] as f32) * 0.22 + raw_features[5] * 0.1,
            (un_b[1] as f32) * 0.22 + raw_features[6] * 0.1,
            (un_b[2] as f32) * 0.22 + raw_features[7] * 0.1,
        ];

        AncestralSeed {
            id: persona_name.to_string(),
            aliases,
            born_year,
            places,
            companions,
            style_wave,
            value_adjust: (voice.warmth + 0.15).clamp(0.05, 0.45),
            domain_bias: [
                diary_ratio.clamp(0.05, 0.65),
                memoir_ratio.clamp(0.05, 0.65),
                temporal_ratio.clamp(0.05, 0.65),
                voice.expressiveness.clamp(0.05, 0.65),
            ],
            origin: persona_name.to_string(),
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
                "pdf",
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

        let voice = VoiceSignature {
            avg_sentence_words,
            warmth,
            expressiveness,
            vocabulary,
            signature_phrases,
            sample_openings,
        };

        let ancestral_seed = if specialty == PersonaSpecialty::LifeBiographer {
            Some(Self::build_ternary_ancestral_seed(name, &episodes, &voice))
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
            voice,
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
            || lower_query.contains("entry")
            || lower_query.contains("letter");
        let wants_mom = lower_query.contains("mom")
            || lower_query.contains("mother")
            || lower_query.contains("mama");
        let wants_dad = lower_query.contains("dad")
            || lower_query.contains("father")
            || lower_query.contains("daddy")
            || lower_query.contains("foreword")
            || lower_query.contains("afterword");
        let wants_dialogue = lower_query.contains("daughter")
            || lower_query.contains("son")
            || lower_query.contains("sister")
            || lower_query.contains("answer")
            || lower_query.contains("replied")
            || lower_query.contains("traits");

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
                        score += if wants_diary { 6.0 } else { 1.8 };
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
                        if wants_dialogue {
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
    /// using a 2-Pass SPQT Base-243 Ternary architecture:
    /// - **Pass 1 (`recall_episodes`)**: retrieves core episodic memories with temporal ordering (`event_time` vs `written_time`).
    /// - **Pass 2 (`synthesize_contextual_answer`)**: scores and synthesizes the exact answering sentences across recalled memories to directly answer the user's question in context.
    pub fn respond_locally(&self, prompt: &str) -> String {
        let recalled = self.recall_episodes(prompt, 8);
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
            let places_str = if seed.places.is_empty() {
                String::new()
            } else {
                format!(
                    " · Places: {}",
                    seed.places
                        .iter()
                        .take(3)
                        .cloned()
                        .collect::<Vec<_>>()
                        .join(" · ")
                )
            };
            let born_str = seed
                .born_year
                .map(|y| format!(" (b. {y})"))
                .unwrap_or_default();
            out.push_str(&format!(
                "_Identity anchor: **{}**{}{}_\n\n",
                seed.id, born_str, places_str
            ));
        }

        // Pass 2: Contextual Question-Answering Synthesis over Pass-1 core memories
        if let Some(synthesized) = self.synthesize_contextual_answer(prompt, &recalled) {
            out.push_str(&format!(
                "💡 **Contextual Answer (SPQT Base-243 Pass-2 Synthesis):**\n{}\n\n",
                synthesized
            ));
        }

        let core_memories: Vec<&PersonaEpisode> = recalled.iter().copied().take(5).collect();

        if let Some(primary) = core_memories.first() {
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
                    "📜 **From my mother's diary & notes, which I preserved** (`{}`{})",
                    primary.source, temporal_badge
                ),
                (_, AuthorRole::PaternalArchive) => {
                    let label = if primary.origin.is_empty() {
                        "From my father's story & historical record"
                    } else {
                        primary.origin.as_str()
                    };
                    format!("📜 **{}** (`{}`{})", label, primary.source, temporal_badge)
                }
                (_, AuthorRole::FamilyDialogue) => format!(
                    "💌 **From a family letter & commentary exchange** (`{}`{})",
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

        if core_memories.len() > 1 {
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
            let mut supporting: Vec<&PersonaEpisode> =
                core_memories.iter().skip(1).copied().collect();
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
                    AuthorRole::MaternalArchive => "📜 *Mother's diary (preserved)*",
                    AuthorRole::PaternalArchive => "📜 *Father's / historical record*",
                    AuthorRole::FamilyDialogue => "💌 *Family dialogue*",
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

    /// **Pass 2 — SPQT Base-243 Ternary Contextual Question-Answering Synthesis**.
    ///
    /// After Pass 1 (`recall_episodes`) retrieves the top candidate episodic memories, Pass 2:
    /// 1. Decomposes the user's question into focus query tokens and a 10-trit interrogative intent signature
    ///    packed into 2 SPQT Base-243 bytes (`spqt_pack_5trits` / `spqt_unpack_5trits`).
    /// 2. Splits the recalled episodes into candidate sentences and evaluates each sentence via
    ///    `felix_ternary_gemm_i32` over a 10-feature alignment vector (rewarding query token hits,
    ///    novel query token coverage across multiple episodes, interrogative intent alignment,
    ///    first-person diary voice, and temporal anchors).
    /// 3. Selects up to 4 complementary sentences across episodes, orders them chronologically
    ///    by `temporal_order`, and synthesizes a cohesive contextual answer in the persona's voice.
    pub fn synthesize_contextual_answer(
        &self,
        prompt: &str,
        recalled: &[&PersonaEpisode],
    ) -> Option<String> {
        if recalled.is_empty() {
            return None;
        }

        let lower_q = prompt.to_ascii_lowercase();
        let asks_when = lower_q.contains("when")
            || lower_q.contains("what date")
            || lower_q.contains("what year")
            || lower_q.contains("what day")
            || lower_q.contains("timeline");
        let asks_who = lower_q.contains("who")
            || lower_q.contains("friend")
            || lower_q.contains("family")
            || lower_q.contains("teacher")
            || lower_q.contains("mother")
            || lower_q.contains("father");
        let asks_where = lower_q.contains("where")
            || lower_q.contains("place")
            || lower_q.contains("house")
            || lower_q.contains("room")
            || lower_q.contains("town");
        let asks_why_how =
            lower_q.contains("why") || lower_q.contains("how") || lower_q.contains("because");
        let asks_what = lower_q.contains("what")
            || lower_q.contains("tell")
            || lower_q.contains("describe")
            || lower_q.contains("explain")
            || (!asks_when && !asks_who && !asks_where && !asks_why_how);

        // Pack 10-trit interrogative intent weights into 2 SPQT Base-243 bytes
        let b0 = spqt_pack_5trits([
            1, // f0: lexical query overlap
            1, // f1: novel token coverage across multi-part questions
            if asks_when { 1 } else { 0 },
            if asks_who { 1 } else { 0 },
            if asks_where { 1 } else { 0 },
        ]);
        let b1 = spqt_pack_5trits([
            if asks_why_how { 1 } else { 0 },
            if asks_what { 1 } else { 0 },
            if self.specialty == PersonaSpecialty::LifeBiographer {
                1
            } else {
                0
            },
            1,  // f8: vivid/narrative sentence completeness
            -1, // f9: boilerplate/fragment penalty
        ]);
        let u0 = spqt_unpack_5trits(b0);
        let u1 = spqt_unpack_5trits(b1);
        let q_trits: [i8; 10] = [
            u0[0], u0[1], u0[2], u0[3], u0[4], u1[0], u1[1], u1[2], u1[3], u1[4],
        ];

        // Extract meaningful query focus tokens
        let mut query_tokens: Vec<String> = Vec::new();
        for raw in prompt.split(|c: char| !c.is_alphanumeric()) {
            let w = raw.to_ascii_lowercase();
            if w.len() >= 3
                && !is_stopword(&w)
                && !matches!(
                    w.as_str(),
                    "what"
                        | "when"
                        | "where"
                        | "who"
                        | "why"
                        | "how"
                        | "did"
                        | "does"
                        | "tell"
                        | "write"
                        | "wrote"
                        | "written"
                        | "diary"
                        | "remember"
                        | "entry"
                        | "entries"
                )
                && !query_tokens.contains(&w)
            {
                query_tokens.push(w);
            }
        }

        struct CandidateSentence<'a> {
            ep: &'a PersonaEpisode,
            ep_rank: usize,
            sent_idx: usize,
            text: String,
            matched_tokens: Vec<String>,
            static_features: [i16; 10],
        }

        let mut candidates: Vec<CandidateSentence<'_>> = Vec::new();
        for (ep_rank, &ep) in recalled.iter().take(8).enumerate() {
            for (sent_idx, sentence) in split_into_clean_sentences(&ep.passage)
                .into_iter()
                .enumerate()
            {
                let word_count = sentence.split_whitespace().count();
                if word_count < 4 {
                    continue;
                }
                let lower_s = sentence.to_ascii_lowercase();
                let mut matched = Vec::new();
                for qt in &query_tokens {
                    if lower_s.contains(qt.as_str()) {
                        matched.push(qt.clone());
                    }
                }

                let f0 = ((matched.len() as i16) * 45).min(240)
                    + (8i16.saturating_sub(ep_rank as i16) * 6);
                let has_time_cue = ep.event_time.is_some()
                    || lower_s.contains("19")
                    || lower_s.contains("20")
                    || lower_s.contains("morning")
                    || lower_s.contains("evening")
                    || lower_s.contains("night")
                    || lower_s.contains("sunday")
                    || lower_s.contains("monday")
                    || lower_s.contains("tuesday")
                    || lower_s.contains("wednesday")
                    || lower_s.contains("thursday")
                    || lower_s.contains("friday")
                    || lower_s.contains("saturday");
                let f2 = if has_time_cue { 70 } else { 0 };

                let cap_interior = sentence
                    .split_whitespace()
                    .skip(1)
                    .filter(|w| {
                        let clean = w.trim_matches(|c: char| !c.is_alphabetic());
                        clean.len() >= 2
                            && clean.chars().next().is_some_and(|c| c.is_ascii_uppercase())
                            && clean != "I"
                    })
                    .count() as i16;
                let f3 = (cap_interior * 25).min(120);

                let has_place_prep = lower_s.contains(" in ")
                    || lower_s.contains(" at ")
                    || lower_s.contains(" to ")
                    || lower_s.contains(" from ")
                    || lower_s.contains(" home")
                    || lower_s.contains(" room")
                    || lower_s.contains(" annex");
                let f4 = if has_place_prep { 65 } else { 0 };

                let has_causal = lower_s.contains("because")
                    || lower_s.contains("so ")
                    || lower_s.contains("since ")
                    || lower_s.contains("therefore")
                    || lower_s.contains("wanted")
                    || lower_s.contains("decided");
                let f5 = if has_causal { 75 } else { 0 };

                let f6 = if (7..=42).contains(&word_count) {
                    60
                } else {
                    20
                };

                let has_fp = lower_s.contains(" i ")
                    || lower_s.starts_with("i ")
                    || lower_s.contains(" my ")
                    || lower_s.contains(" we ")
                    || lower_s.contains(" our ");
                let f7 = if has_fp || ep.author_role == AuthorRole::PrimaryDiary {
                    65
                } else {
                    15
                };

                let f8 = ((ep.valence.abs() + ep.arousal) * 30.0) as i16;
                let is_boilerplate = lower_s.contains("all rights reserved")
                    || lower_s.contains("copyright")
                    || word_count > 75;
                let f9 = if is_boilerplate { 160 } else { 0 };

                candidates.push(CandidateSentence {
                    ep,
                    ep_rank,
                    sent_idx,
                    text: sentence,
                    matched_tokens: matched,
                    static_features: [f0, 0, f2, f3, f4, f5, f6, f7, f8, f9],
                });
            }
        }

        if candidates.is_empty() {
            return None;
        }

        // Greedy SPQT Base-243 ternary coverage selection across up to 3 complementary sentences
        let mut selected_indices: Vec<usize> = Vec::new();
        let mut covered_tokens: Vec<String> = Vec::new();

        for _step in 0..3 {
            let mut best_idx = None;
            let mut best_score = i32::MIN;

            for (idx, cand) in candidates.iter().enumerate() {
                if selected_indices.contains(&idx) {
                    continue;
                }
                let novel_hits = cand
                    .matched_tokens
                    .iter()
                    .filter(|t| !covered_tokens.contains(t))
                    .count() as i16;
                // After the first sentence, require at least 1 novel query token hit (or top episode rank if query had <=1 token)
                if !selected_indices.is_empty() && novel_hits == 0 && query_tokens.len() > 1 {
                    continue;
                }
                let mut feats = cand.static_features;
                feats[1] = (novel_hits * 75).min(240);
                let score = crate::felix_asm::felix_ternary_gemm_i32(&feats, &q_trits);
                if score > best_score {
                    best_score = score;
                    best_idx = Some(idx);
                }
            }

            if let Some(chosen) = best_idx {
                for tok in &candidates[chosen].matched_tokens {
                    if !covered_tokens.contains(tok) {
                        covered_tokens.push(tok.clone());
                    }
                }
                selected_indices.push(chosen);
            } else {
                break;
            }
        }

        if selected_indices.is_empty() {
            return None;
        }

        // Order selected sentences chronologically when temporal anchors exist, preserving intra-episode sentence order
        selected_indices.sort_by(|&a, &b| {
            let ca = &candidates[a];
            let cb = &candidates[b];
            let ta = if ca.ep.temporal_order > 0 {
                ca.ep.temporal_order
            } else {
                i32::MAX
            };
            let tb = if cb.ep.temporal_order > 0 {
                cb.ep.temporal_order
            } else {
                i32::MAX
            };
            ta.cmp(&tb)
                .then_with(|| ca.ep_rank.cmp(&cb.ep_rank))
                .then_with(|| ca.sent_idx.cmp(&cb.sent_idx))
        });

        let mut parts: Vec<String> = Vec::new();
        let mut last_badge = String::new();
        for idx in selected_indices {
            let cand = &candidates[idx];
            let badge = match (&cand.ep.event_time, &cand.ep.era) {
                (Some(ev), _) => format!("**[{ev}]** "),
                (None, Some(era)) => format!("**[{era}]** "),
                (None, None) => String::new(),
            };
            if !badge.is_empty() && badge != last_badge {
                last_badge = badge.clone();
                parts.push(format!("{badge}{}", cand.text));
            } else {
                parts.push(cand.text.clone());
            }
        }

        Some(parts.join(" "))
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
             3. If a memory comes from your mother's diary (MaternalArchive), father's story (PaternalArchive), or family dialogue (FamilyDialogue), credit them warmly and accurately.\n\n\
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

/// Split an episodic passage into clean sentences, stripping leading diary date headers
/// so Pass 2 can prepend a structured temporal badge cleanly.
fn split_into_clean_sentences(passage: &str) -> Vec<String> {
    let normalized = passage.replace('\n', " ");
    let mut raw_sentences = Vec::new();
    let mut current = String::new();
    let chars: Vec<char> = normalized.chars().collect();
    let len = chars.len();
    let mut i = 0;

    while i < len {
        let ch = chars[i];
        current.push(ch);
        if matches!(ch, '.' | '!' | '?') {
            // Avoid splitting on common abbreviations like "Mr.", "Mrs.", "Dr.", "Wed.", "Thurs."
            let trimmed = current.trim_end();
            let last_word = trimmed
                .rsplit(|c: char| c.is_whitespace())
                .next()
                .unwrap_or("");
            let is_abbrev = matches!(
                last_word,
                "Mr."
                    | "Mrs."
                    | "Ms."
                    | "Dr."
                    | "St."
                    | "Jr."
                    | "Sr."
                    | "Mon."
                    | "Tue."
                    | "Tues."
                    | "Wed."
                    | "Thu."
                    | "Thur."
                    | "Thurs."
                    | "Fri."
                    | "Sat."
                    | "Sun."
                    | "Jan."
                    | "Feb."
                    | "Mar."
                    | "Apr."
                    | "Aug."
                    | "Sep."
                    | "Sept."
                    | "Oct."
                    | "Nov."
                    | "Dec."
            );
            let next_is_space_or_end = i + 1 == len || chars[i + 1].is_whitespace();
            if !is_abbrev && next_is_space_or_end {
                let clean = strip_leading_date_header(current.trim());
                if !clean.is_empty() {
                    raw_sentences.push(clean);
                }
                current.clear();
            }
        }
        i += 1;
    }

    let tail = strip_leading_date_header(current.trim());
    if !tail.is_empty() {
        raw_sentences.push(tail);
    }
    raw_sentences
}

/// Strip leading diary date/salutation headers (e.g. `"SATURDAY, JUNE 20, 1942 "` or `"January 3 Thurs. "`)
/// from the front of a sentence when the temporal badge already carries the date.
fn strip_leading_date_header(sentence: &str) -> String {
    let mut s = sentence.trim();
    let tokens: Vec<&str> = s.split_whitespace().collect();
    if tokens.len() >= 5 {
        let t0 = tokens[0].trim_matches(|c: char| !c.is_alphabetic());
        let t1 = tokens[1].trim_matches(|c: char| !c.is_alphabetic());
        let t2 = tokens[2].trim_matches(|c: char| !c.is_ascii_digit());
        let t3 = tokens[3].trim_matches(|c: char| !c.is_ascii_digit());
        // Pattern A: "SATURDAY, JUNE 20, 1942 <rest>"
        if parse_weekday(&t0.to_ascii_lowercase()).is_some()
            && parse_month_name(&t1.to_ascii_lowercase()).is_some()
            && t2.parse::<u8>().is_ok()
            && t3.len() == 4
            && t3.parse::<u16>().is_ok()
        {
            let prefix_end = tokens[..4].join(" ").len();
            if s.len() > prefix_end {
                s = s[prefix_end..].trim_start_matches(|c: char| c.is_whitespace() || c == ',');
            }
        } else if parse_month_name(&t0.to_ascii_lowercase()).is_some()
            && tokens[1]
                .trim_matches(|c: char| !c.is_ascii_digit())
                .parse::<u8>()
                .is_ok()
            && parse_weekday(&t2.to_ascii_lowercase()).is_some()
        {
            // Pattern B: "January 3 Thurs. <rest>"
            let prefix_end = tokens[..3].join(" ").len();
            if s.len() > prefix_end {
                s = s[prefix_end..]
                    .trim_start_matches(|c: char| c.is_whitespace() || c == '.' || c == ',');
            }
        }
    }
    s.trim().to_string()
}

/// Pack 5 trits (`{-1, 0, +1}`) into a single SPQT Base-243 (`3^5 = 243`) byte.
#[inline(always)]
fn spqt_pack_5trits(trits: [i8; 5]) -> u8 {
    let mut byte_val = 0u8;
    let mut radix = 1u8;
    for t in trits {
        let digit = (t.clamp(-1, 1) + 1) as u8;
        byte_val = byte_val.wrapping_add(digit.wrapping_mul(radix));
        radix = radix.wrapping_mul(3);
    }
    byte_val
}

/// Unpack a single SPQT Base-243 byte back into 5 trits (`{-1, 0, +1}`).
#[inline(always)]
fn spqt_unpack_5trits(mut packed: u8) -> [i8; 5] {
    let mut out = [0i8; 5];
    for slot in &mut out {
        *slot = ((packed % 3) as i8) - 1;
        packed /= 3;
    }
    out
}

/// SPQT Base-243 Ternary Authorship & Provenance Classifier (`W in {-1, 0, +1}^{6 x 10}`)
/// Evaluates 10 structural & linguistic features using packed Base-243 ternary weights and
/// `felix_ternary_gemm_i32` (zero hardcoded names).
fn ternary_classify_author_and_origin(
    source_label: &str,
    full_content_header: &str,
    passage: &str,
) -> (AuthorRole, String, Option<String>) {
    use crate::felix_asm::felix_ternary_gemm_i32;

    let lower_src = source_label.to_ascii_lowercase();
    let lower_hdr = full_content_header.to_ascii_lowercase();
    let lower_pass = passage.to_ascii_lowercase();

    // Count first-person vs third-person pronouns in the passage
    let mut fp_sing = 0usize;
    let mut fp_plur = 0usize;
    let mut tp_bio = 0usize;
    let mut word_total = 0usize;
    for tok in lower_pass.split(|c: char| !c.is_ascii_alphabetic() && c != '\'') {
        if tok.is_empty() {
            continue;
        }
        word_total += 1;
        match tok {
            "i" | "my" | "me" | "myself" | "mine" | "i'm" | "i've" | "i'll" | "i'd" => {
                fp_sing += 1;
            }
            "we" | "our" | "us" | "ourselves" | "we're" | "we've" => {
                fp_plur += 1;
            }
            "she" | "her" | "hers" | "he" | "his" | "him" => {
                tp_bio += 1;
            }
            _ => {}
        }
    }
    let denom = (word_total.max(1)) as f32;
    let x0_fp_sing = ((fp_sing as f32) / denom * 18.0).min(3.5);
    let x1_fp_plur = ((fp_plur as f32) / denom * 18.0).min(2.5);

    let editorial_markers = lower_src.contains("foreword")
        || lower_src.contains("afterword")
        || lower_pass.contains("book flap")
        || lower_pass.contains("-- : -foreword")
        || lower_pass.contains("----afterword")
        || lower_pass.contains("translated by")
        || lower_pass.contains("definitive edition")
        || lower_pass.contains("concentration camp");
    let x2_tp_editorial =
        ((tp_bio as f32) / denom * 12.0).min(2.5) + if editorial_markers { 3.5 } else { 0.0 };

    let maternal_src = lower_src.contains("mom")
        || lower_src.contains("mama")
        || lower_src.contains("mother")
        || lower_src.contains("short diary")
        || lower_src.contains("colorado diary")
        || lower_src.contains("prospecting")
        || (lower_hdr.contains("diary of ") && lower_hdr.contains("edited by"));
    let x5_maternal = if maternal_src { 3.8 } else { 0.0 };

    let paternal_src = lower_src.contains("dad")
        || lower_src.contains("daddy")
        || lower_src.contains("father")
        || lower_src.contains("foreword")
        || lower_src.contains("afterword")
        || editorial_markers;
    let x6_paternal = if paternal_src { 3.6 } else { 0.0 };

    let dialogue_src = lower_src.contains("answer")
        || lower_src.contains("letter")
        || lower_src.contains("dialogue");
    let x7_dialogue = if dialogue_src { 3.6 } else { 0.0 };

    let diary_signal = (!maternal_src
        && (lower_src.contains("diary") || lower_hdr.contains("yearslong diary")))
        || lower_pass.contains("dearest ")
        || lower_pass.contains("dear ")
        || lower_pass.contains("yours, ");
    let x3_diary = if diary_signal { 3.2 } else { 0.0 };

    let memoir_signal = lower_src.contains("story")
        || lower_src.contains("first 15 years")
        || lower_src.contains("beginning")
        || lower_src.contains("poetry")
        || lower_src.contains("gems_of_thought")
        || lower_src.contains("glory")
        || lower_src.contains("match_race")
        || lower_src.contains("traits")
        || lower_src.contains("raamblings");
    let x4_memoir = if memoir_signal { 2.6 } else { 0.0 };

    let has_entry_date = detect_month_day_in_text(passage, None).is_some();
    let x8_date_hdr = if has_entry_date { 2.4 } else { 0.0 };

    let editor_override = lower_pass.contains("this book is dedicated to")
        || lower_pass.contains("epilog by")
        || lower_pass.contains("edited by")
        || lower_pass.starts_with("note this edition");
    let x9_editor_note = if editor_override { 4.2 } else { 0.0 };

    let x: [f32; 10] = [
        x0_fp_sing,
        x1_fp_plur,
        x2_tp_editorial,
        x3_diary,
        x4_memoir,
        x5_maternal,
        x6_paternal,
        x7_dialogue,
        x8_date_hdr,
        x9_editor_note,
    ];

    // 6x10 Ternary Weight Matrix W in {-1, 0, +1}, packed via SPQT Base-243 (2 bytes per row)
    let raw_rows: [([i8; 5], [i8; 5], f32, AuthorRole); 6] = [
        // PrimaryDiary: +fp_sing, +fp_plur, -tp_editorial, +diary, 0, -maternal, -paternal, -dialogue, +date_hdr, 0
        (
            [1, 1, -1, 1, 0],
            [-1, -1, -1, 1, 0],
            0.2,
            AuthorRole::PrimaryDiary,
        ),
        // PrimaryMemoir: +fp_sing, +fp_plur, -tp_editorial, -diary, +memoir, -maternal, -paternal, 0, 0, +editor_note
        (
            [1, 1, -1, -1, 1],
            [-1, -1, 0, 0, 1],
            0.1,
            AuthorRole::PrimaryMemoir,
        ),
        // MaternalArchive: 0, 0, 0, 0, 0, +maternal, -paternal, -dialogue, 0, -editor_note
        (
            [0, 0, 0, 0, 0],
            [1, -1, -1, 0, -1],
            0.0,
            AuthorRole::MaternalArchive,
        ),
        // PaternalArchive: -fp_sing, 0, +tp_editorial, -diary, 0, -maternal, +paternal, -dialogue, -date_hdr, -editor_note
        (
            [-1, 0, 1, -1, 0],
            [-1, 1, -1, -1, -1],
            0.0,
            AuthorRole::PaternalArchive,
        ),
        // FamilyDialogue: +fp_sing, 0, 0, -diary, 0, -maternal, -paternal, +dialogue, -date_hdr, 0
        (
            [1, 0, 0, -1, 0],
            [-1, -1, 1, -1, 0],
            0.0,
            AuthorRole::FamilyDialogue,
        ),
        // GeneralCorpus: baseline fallback
        (
            [0, 0, 0, -1, -1],
            [-1, -1, -1, 0, 0],
            0.5,
            AuthorRole::GeneralCorpus,
        ),
    ];

    let mut acts_i16 = [0i16; 10];
    for j in 0..10 {
        acts_i16[j] = (x[j] * 100.0).round() as i16;
    }

    let mut best_role = AuthorRole::GeneralCorpus;
    let mut best_score = i32::MIN;

    for (hi, lo, bias, role) in raw_rows {
        let p_hi = spqt_pack_5trits(hi);
        let p_lo = spqt_pack_5trits(lo);
        let u_hi = spqt_unpack_5trits(p_hi);
        let u_lo = spqt_unpack_5trits(p_lo);
        let trits_10: [i8; 10] = [
            u_hi[0], u_hi[1], u_hi[2], u_hi[3], u_hi[4], u_lo[0], u_lo[1], u_lo[2], u_lo[3],
            u_lo[4],
        ];
        let acc = felix_ternary_gemm_i32(&acts_i16, &trits_10) + (bias * 100.0).round() as i32;
        if acc > best_score {
            best_score = acc;
            best_role = role;
        }
    }

    let written = detect_date_or_era(source_label);
    let origin = match best_role {
        AuthorRole::PrimaryDiary => "First-Person Lived Diary".to_string(),
        AuthorRole::PrimaryMemoir => {
            if editor_override {
                "First-Person Editor's Dedication & Epilog".to_string()
            } else {
                "First-Person Memoir, Stories & Poetry".to_string()
            }
        }
        AuthorRole::MaternalArchive => "Mother's Diary & Notes (Preserved in Archive)".to_string(),
        AuthorRole::PaternalArchive => {
            if editorial_markers
                || lower_src.contains("foreword")
                || lower_src.contains("afterword")
            {
                "From the Historical Foreword & Afterword".to_string()
            } else {
                "From my father's story & advice".to_string()
            }
        }
        AuthorRole::FamilyDialogue => "Family Letter & Commentary Exchange".to_string(),
        AuthorRole::GeneralCorpus => "Primary Corpus".to_string(),
    };

    (best_role, origin, written)
}

/// Read text from a document file, automatically extracting and unwrapping PDFs via `pdftotext` when needed.
fn read_document_text(file_path: &Path) -> Option<String> {
    let ext = file_path
        .extension()
        .and_then(|e| e.to_str())
        .map(|s| s.to_ascii_lowercase())
        .unwrap_or_default();
    if ext == "pdf" {
        let out = std::process::Command::new("pdftotext")
            .arg(file_path)
            .arg("-")
            .output()
            .ok()?;
        if !out.status.success() {
            return None;
        }
        let raw = String::from_utf8_lossy(&out.stdout);
        return Some(unwrap_pdf_text(&raw));
    }
    fs::read_to_string(file_path).ok()
}

/// Reconstruct clean paragraphs and diary date headers from raw `pdftotext` output
fn unwrap_pdf_text(raw: &str) -> String {
    let cleaned = raw
        .replace('\u{000c}', "\n")
        .replace("]ANUARY", "JANUARY")
        .replace("-- : -June 12, 1942", "FRIDAY, JUNE 12, 1942");

    let mut paras: Vec<String> = Vec::new();
    let mut cur: Vec<String> = Vec::new();

    let flush = |cur: &mut Vec<String>, paras: &mut Vec<String>| {
        if !cur.is_empty() {
            let joined = cur.join(" ");
            let norm: String = joined.split_whitespace().collect::<Vec<_>>().join(" ");
            if !norm.is_empty() {
                paras.push(norm);
            }
            cur.clear();
        }
    };

    for raw_line in cleaned.lines() {
        let line = raw_line.trim();
        if line.is_empty() {
            continue;
        }
        // Skip page numbers / running headers or e-book markers
        if line.chars().all(|c| c.is_ascii_digit()) || line.starts_with("-- : -") {
            if line.contains("FOREWORD") || line.contains("BOOK FLAP") {
                flush(&mut cur, &mut paras);
                cur.push(line.to_string());
                flush(&mut cur, &mut paras);
            }
            continue;
        }
        if line.contains("DIARY ENDS HERE") || line.contains("----AFTERWORD") {
            flush(&mut cur, &mut paras);
            cur.push(line.to_string());
            flush(&mut cur, &mut paras);
            continue;
        }

        // Check if this line is a standalone date header (e.g. "SUNDAY, JUNE 14, 1942")
        if line.len() <= 48 && detect_month_day_in_text(line, None).is_some() {
            let alpha_upper = line
                .chars()
                .filter(|c| c.is_ascii_alphabetic())
                .all(|c| c.is_ascii_uppercase());
            if alpha_upper || is_short_date_header(line) {
                flush(&mut cur, &mut paras);
                cur.push(line.to_string());
                continue;
            }
        }

        if line.starts_with("Dearest ")
            || line.starts_with("Dear ")
            || line.starts_with("Yours, ")
            || line.starts_with("COMMENT ADDED")
        {
            if line.starts_with("Yours, ") {
                cur.push(line.to_string());
                flush(&mut cur, &mut paras);
            } else {
                // Keep any immediately preceding date header attached to "Dearest ..."
                if cur.len() == 1
                    && cur[0].len() <= 48
                    && detect_month_day_in_text(&cur[0], None).is_some()
                {
                    cur.push(line.to_string());
                } else {
                    flush(&mut cur, &mut paras);
                    cur.push(line.to_string());
                }
            }
            continue;
        }

        if let Some(prev) = cur.last() {
            let prev_ends_sent = prev.ends_with(['.', '!', '?', '"', '\'', ':']);
            let starts_upper = line
                .chars()
                .next()
                .is_some_and(|c| c.is_ascii_uppercase() || c == '"' || c == '(');
            if prev_ends_sent && prev.len() < 68 && starts_upper {
                flush(&mut cur, &mut paras);
            }
        }
        cur.push(line.to_string());
    }
    flush(&mut cur, &mut paras);
    paras.join("\n\n")
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
    let content = read_document_text(file_path)?;
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
        ternary_classify_author_and_origin(&source_label, &header_preview, &header_preview);

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

    // Running temporal context across the file: if no explicit year in header, solve via perpetual calendar!
    let mut running_year: Option<u16> = detect_initial_year(&source_label, &header_preview)
        .or_else(|| infer_year_from_weekday_constraints(&chunks));
    let mut running_month: Option<u8> = None;
    let mut running_day: Option<(u8, u8, Option<&'static str>)> = None;
    let mut in_editorial_section =
        header_preview.contains("BOOK FLAP") || header_preview.contains("-- : -FOREWORD");
    let is_diary_doc = file_role == AuthorRole::PrimaryDiary
        || (source_label.to_ascii_lowercase().contains("diary")
            && file_role != AuthorRole::MaternalArchive);

    for (idx, para) in chunks.iter().enumerate() {
        let lower_p = para.to_ascii_lowercase();
        if lower_p.contains("-- : -book flap")
            || lower_p.contains("-- : -foreword")
            || lower_p.contains("----afterword")
            || lower_p.contains("diary ends here")
        {
            in_editorial_section = true;
            running_day = None;
        }
        // Check the opening 60 chars of the chunk for a diary entry date header
        let hdr_prefix: String = para.chars().take(60).collect();
        let first_line = hdr_prefix.lines().next().unwrap_or(&hdr_prefix).trim();
        let entry_day = if in_editorial_section {
            // Only exit an editorial Foreword when a genuine standalone/uppercase diary date header starts the chunk
            let starts_with_date = first_line
                .split(|c: char| c.is_whitespace() || c == ',')
                .next()
                .is_some_and(|tok| parse_weekday(tok).is_some() || parse_month_name(tok).is_some());
            if starts_with_date {
                detect_month_day_in_text(&hdr_prefix, running_month)
            } else {
                None
            }
        } else {
            let starts_with_date = first_line
                .split(|c: char| c.is_whitespace() || c == ',')
                .next()
                .is_some_and(|tok| parse_weekday(tok).is_some() || parse_month_name(tok).is_some());
            if starts_with_date {
                detect_month_day_in_text(&hdr_prefix, running_month)
            } else {
                None
            }
        };

        if let Some((m, d, wk)) = entry_day {
            running_month = Some(m);
            running_day = Some((m, d, wk));
            in_editorial_section = false;
            if let Some(y) = detect_lived_year_in_text(&hdr_prefix) {
                running_year = Some(y);
            }
        } else if !is_diary_doc {
            if let Some(y) = detect_lived_year_in_text(para) {
                running_year = Some(y);
            }
        }
        let parsed_day = if is_diary_doc && !in_editorial_section {
            entry_day.or(running_day)
        } else {
            entry_day.or_else(|| detect_month_day_in_text(&hdr_prefix, running_month))
        };

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
                ternary_classify_author_and_origin(&source_label, &header_preview, para);
            let (role, origin) = if in_editorial_section {
                (
                    AuthorRole::PaternalArchive,
                    "From the Historical Foreword & Afterword".to_string(),
                )
            } else if is_diary_doc && running_day.is_some() {
                (
                    AuthorRole::PrimaryDiary,
                    "First-Person Lived Diary".to_string(),
                )
            } else if ep_role != AuthorRole::GeneralCorpus {
                (ep_role, ep_origin)
            } else {
                (file_role, file_origin.clone())
            };
            let written_time = ep_written.or_else(|| file_written_time.clone());

            let (event_time, temporal_order) = resolve_episode_temporal(
                para,
                &source_label,
                running_year,
                parsed_day,
                role == AuthorRole::PrimaryDiary,
            );
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
    detect_lived_year_in_text(header).or_else(|| detect_lived_year_in_text(source_label))
}

/// Sakamoto's Perpetual Calendar Algorithm: returns `"Sun".."Sat"` for any Gregorian `(year, month, day)`.
fn weekday_for_ymd(y: i32, m: u8, d: u8) -> &'static str {
    const T: [i32; 12] = [0, 3, 2, 5, 0, 3, 5, 1, 4, 6, 2, 4];
    let mut yr = y;
    if m < 3 {
        yr -= 1;
    }
    let idx = (m.saturating_sub(1) as usize).min(11);
    let dow = (yr + yr / 4 - yr / 100 + yr / 400 + T[idx] + (d as i32)).rem_euclid(7);
    match dow {
        0 => "Sun",
        1 => "Mon",
        2 => "Tue",
        3 => "Wed",
        4 => "Thu",
        5 => "Fri",
        _ => "Sat",
    }
}

/// Solve for the calendar year of an undated diary from its `(month, day, weekday)` constraints!
fn infer_year_from_weekday_constraints(chunks: &[String]) -> Option<u16> {
    let mut constraints: Vec<(u8, u8, &'static str)> = Vec::new();
    for c in chunks.iter().take(40) {
        let prefix: String = c.chars().take(75).collect();
        if let Some((m, d, Some(wk))) = detect_month_day_in_text(&prefix, None) {
            constraints.push((m, d, wk));
            if constraints.len() >= 6 {
                break;
            }
        }
    }
    if constraints.is_empty() {
        return None;
    }
    // Search candidate years centered around the mid-20th century (1964 first, then outward)
    let mut candidates: Vec<u16> = (1920..=2025).collect();
    candidates.sort_by_key(|&y| (y as i32 - 1964).abs());
    let mut best_year = None;
    let mut best_matches = 0usize;
    for y in candidates {
        let matches = constraints
            .iter()
            .filter(|&&(m, d, wk)| weekday_for_ymd(y as i32, m, d) == wk)
            .count();
        if matches > best_matches && matches * 2 >= constraints.len() {
            best_matches = matches;
            best_year = Some(y);
            if matches == constraints.len() {
                break;
            }
        }
    }
    best_year
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

/// Detect `"January 2 Wed."`, `"SUNDAY, JUNE 14, 1942"`, `"MONDAY EVENING, NOVEMBER 8, 1943"`, etc.
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
                            .or_else(|| tokens.get(i + 3).and_then(|w| parse_weekday(w)))
                            .or_else(|| {
                                i.checked_sub(1)
                                    .and_then(|idx| tokens.get(idx))
                                    .and_then(|w| parse_weekday(w))
                            })
                            .or_else(|| {
                                i.checked_sub(2)
                                    .and_then(|idx| tokens.get(idx))
                                    .and_then(|w| parse_weekday(w))
                            });
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
    is_diary: bool,
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

    // In a diary, prefer the running entry year over flashback years mentioned mid-paragraph
    let year_opt = if is_diary {
        running_year
            .or_else(|| detect_lived_year_in_text(source_label))
            .or_else(|| detect_lived_year_in_text(para))
    } else {
        detect_lived_year_in_text(para)
            .or(running_year)
            .or_else(|| detect_lived_year_in_text(source_label))
    };

    if let (Some(year), Some((month, day, wk_opt))) = (year_opt, parsed_day) {
        let wk = wk_opt.unwrap_or_else(|| weekday_for_ymd(year as i32, month, day));
        let label = format!("{year:04}-{month:02}-{day:02} ({wk})");
        let sort_key = (year as i32) * 10_000 + (month as i32) * 100 + (day as i32);
        return (Some(label), sort_key);
    }

    if let Some(year) = year_opt {
        return (Some(year.to_string()), (year as i32) * 10_000);
    }

    (None, 0)
}

fn is_calendar_or_common_cap(low: &str) -> bool {
    parse_month_name(low).is_some()
        || parse_weekday(low).is_some()
        || is_stopword(low)
        || matches!(
            low,
            "the"
                | "this"
                | "that"
                | "then"
                | "there"
                | "when"
                | "after"
                | "before"
                | "since"
                | "while"
                | "every"
                | "some"
                | "many"
                | "most"
                | "other"
                | "another"
                | "today"
                | "yesterday"
                | "tomorrow"
                | "morning"
                | "evening"
                | "afternoon"
                | "night"
                | "dearest"
                | "dear"
                | "yours"
                | "diary"
                | "page"
                | "book"
                | "chapter"
                | "note"
                | "comment"
                | "part"
                | "time"
                | "year"
                | "years"
                | "english"
                | "dutch"
                | "french"
                | "german"
                | "latin"
                | "spanish"
        )
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
        assert!(reply.contains("Contextual Answer (SPQT Base-243 Pass-2 Synthesis)"));
    }

    #[test]
    fn pass2_contextual_synthesis_combines_multiple_diary_entries() {
        let tmp = TempDir::new().unwrap();
        let diary = tmp.path().join("diary_1942.md");
        fs::write(
            &diary,
            "SATURDAY, JUNE 20, 1942\n\n\
             Dearest Kitty, Hello, I founded the Little Dipper Minus Two Ping-Pong club with my girlfriends \
             and we always went to the nearest ice-cream parlor afterward.\n\n\
             SUNDAY, JUNE 21, 1942\n\n\
             Dearest Kitty, Mr. Keesing, the old fogey who teaches math, was annoyed with me for talking so much \
             and assigned me an extra essay called A Chatterbox.",
        )
        .unwrap();

        let (persona, _, _) = LifePersona::train_from_path("Anne Frank", tmp.path()).unwrap();
        let recalled = persona.recall_episodes(
            "What did you write about the Ping-Pong club and Mr. Keesing Chatterbox essay?",
            8,
        );
        let synth = persona
            .synthesize_contextual_answer(
                "What did you write about the Ping-Pong club and Mr. Keesing Chatterbox essay?",
                &recalled,
            )
            .expect("Pass 2 should synthesize an answer");
        assert!(synth.contains("1942-06-20 (Sat)"));
        assert!(synth.contains("Ping-Pong"));
        assert!(synth.contains("1942-06-21 (Sun)"));
        assert!(synth.contains("Chatterbox"));
    }
}
