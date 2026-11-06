//! 🧬 MEM8 Life Persona Engine — Train `.m8` Souls from Diaries, Drives & Life Corpora
//!
//! Builds a tight binary `.m8` persona container (`MEM8` v1 + `WAVE` + `MQNT` + `ZLIB`)
//! from any folder of diaries, letters, notes, or synced Google Drive documents.
//! Combines a Voice/Cadence profile with an immutable ("Corrugated Iron", `tau = ∞`)
//! episodic memory bank so the persona speaks in their own words and remembers
//! real dates, people, and moments offline with zero cloud API keys required.

use crate::m8_consciousness::{decode_m8_bytes, encode_json_bytes_as_tight_m8};
use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

/// A single episodic memory passage extracted from a diary, letter, or document
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PersonaEpisode {
    /// Source file (relative to corpus root)
    pub source: String,
    /// Detected date or era label if present (e.g. "1964-05-14" or "June 1958")
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
    /// Signature vocabulary (most characteristic words)
    pub vocabulary: Vec<String>,
    /// Recurring 2-word and 3-word phrases / idioms
    pub signature_phrases: Vec<String>,
    /// Sample opening sentences from their actual writing
    pub sample_openings: Vec<String>,
}

/// Complete trained Life Persona stored inside `~/.st/personas/<slug>.m8`
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LifePersona {
    /// Display name (e.g., "Grandmother's Diaries")
    pub name: String,
    /// URL/filename-safe slug (e.g., "grandmothers-diaries")
    pub slug: String,
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
    /// Episodic memories ("Corrugated Iron" foundational life memories)
    pub episodes: Vec<PersonaEpisode>,
}

/// Summary metadata for listing personas in CLI and Dashboard
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PersonaSummary {
    pub name: String,
    pub slug: String,
    pub source_path: String,
    pub trained_at: DateTime<Utc>,
    pub files_ingested: usize,
    pub total_words: usize,
    pub episode_count: usize,
    pub m8_size_bytes: u64,
    pub sample_phrases: Vec<String>,
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

    /// Train a Life Persona from a directory or file and save to `~/.st/personas/<slug>.m8`
    pub fn train_from_path(name: &str, root_path: &Path) -> Result<(Self, PathBuf, u64)> {
        let canon = root_path
            .canonicalize()
            .unwrap_or_else(|_| root_path.to_path_buf());
        if !canon.exists() {
            anyhow::bail!("Training path does not exist: {}", canon.display());
        }

        let slug = Self::slugify(name);
        let mut files_ingested = 0usize;
        let mut total_words = 0usize;
        let mut episodes = Vec::new();
        let mut word_counts: HashMap<String, usize> = HashMap::new();
        let mut phrase_counts: HashMap<String, usize> = HashMap::new();
        let mut sample_openings: Vec<String> = Vec::new();
        let mut sentence_count = 0usize;
        let mut expressive_sentences = 0usize;
        let mut total_valence = 0.0f32;

        let allowed_exts: HashSet<&str> = [
            "md", "txt", "org", "rst", "eml", "markdown", "log", "json", "html", "htm", "tex",
            "csv",
        ]
        .into_iter()
        .collect();

        let entries: Vec<PathBuf> = if canon.is_file() {
            vec![canon.clone()]
        } else {
            WalkDir::new(&canon)
                .max_depth(6)
                .follow_links(false)
                .into_iter()
                .filter_entry(|e| {
                    if e.depth() == 0 {
                        return true;
                    }
                    let name = e.file_name().to_string_lossy();
                    !name.starts_with('.')
                        && name != "node_modules"
                        && name != "target"
                        && name != "venv"
                        && name != "__pycache__"
                })
                .filter_map(|e| e.ok())
                .filter(|e| e.path().is_file())
                .filter(|e| {
                    e.path()
                        .extension()
                        .and_then(|ext| ext.to_str())
                        .map(|ext| allowed_exts.contains(ext.to_ascii_lowercase().as_str()))
                        .unwrap_or(false)
                })
                .take(1500)
                .map(|e| e.path().to_path_buf())
                .collect()
        };

        for file_path in entries {
            let Ok(meta) = fs::metadata(&file_path) else {
                continue;
            };
            if meta.len() == 0 || meta.len() > 1024 * 1024 {
                continue;
            }
            let Ok(content) = fs::read_to_string(&file_path) else {
                continue;
            };
            if content.trim().is_empty() {
                continue;
            }

            files_ingested += 1;
            let rel_source = file_path
                .strip_prefix(&canon)
                .unwrap_or(&file_path)
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

            for raw_para in content.split("\n\n") {
                let para = raw_para.trim();
                if para.len() < 32 {
                    continue;
                }
                let words: Vec<&str> = para.split_whitespace().collect();
                if words.len() < 6 {
                    continue;
                }
                total_words += words.len();

                // Count sentences & expressiveness
                for sent in para.split(['.', '!', '?']) {
                    let s = sent.trim();
                    if s.split_whitespace().count() >= 4 {
                        sentence_count += 1;
                        if sample_openings.len() < 12 && s.len() >= 25 && s.len() <= 140 {
                            sample_openings.push(s.to_string());
                        }
                    }
                }
                expressive_sentences += para.matches(['!', '?']).count();

                // Extract vocabulary and 2/3-gram phrases
                let clean_tokens: Vec<String> = words
                    .iter()
                    .map(|w| {
                        w.trim_matches(|c: char| !c.is_alphanumeric() && c != '\'')
                            .to_lowercase()
                    })
                    .filter(|w| w.len() >= 3 && !is_stopword(w))
                    .collect();

                for tok in &clean_tokens {
                    *word_counts.entry(tok.clone()).or_insert(0) += 1;
                }
                for win in clean_tokens.windows(2) {
                    let phrase = format!("{} {}", win[0], win[1]);
                    *phrase_counts.entry(phrase).or_insert(0) += 1;
                }

                let (valence, arousal) = score_emotion(para);
                total_valence += valence;

                if episodes.len() < 2000 {
                    let era =
                        detect_date_or_era(para).or_else(|| detect_date_or_era(&source_label));
                    let mut kw: Vec<String> = Vec::new();
                    for tok in &clean_tokens {
                        if tok.len() >= 4 && !kw.contains(tok) {
                            kw.push(tok.clone());
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
                    let passage = if para.len() > 900 {
                        let mut end = 900;
                        while end > 0 && !para.is_char_boundary(end) {
                            end -= 1;
                        }
                        format!("{}...", &para[..end])
                    } else {
                        para.to_string()
                    };

                    episodes.push(PersonaEpisode {
                        source: source_label.clone(),
                        era,
                        keywords: kw,
                        valence,
                        arousal,
                        frequency_hz,
                        passage,
                    });
                }
            }
        }

        if episodes.is_empty() {
            anyhow::bail!(
                "No readable text passages found in {} (checked .md, .txt, .org, .eml, .json, .html)",
                canon.display()
            );
        }

        let mut vocab_vec: Vec<(String, usize)> = word_counts.into_iter().collect();
        vocab_vec.sort_by_key(|b| std::cmp::Reverse(b.1));
        let vocabulary: Vec<String> = vocab_vec.into_iter().take(48).map(|(w, _)| w).collect();

        let mut phrase_vec: Vec<(String, usize)> =
            phrase_counts.into_iter().filter(|(_, c)| *c >= 2).collect();
        phrase_vec.sort_by_key(|b| std::cmp::Reverse(b.1));
        let signature_phrases: Vec<String> =
            phrase_vec.into_iter().take(24).map(|(p, _)| p).collect();

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

        let m8_path = Self::personas_dir_for_source(&canon)?.join(format!("{slug}.m8"));
        let json_bytes = serde_json::to_vec(&persona)?;
        let soul_md = format!(
            "# MEM8 Persona Soul: {}\nSource: {}\nEpisodes: {}\nWords: {}\nVocabulary: {}\n",
            persona.name,
            persona.source_path,
            persona.episodes.len(),
            persona.total_words,
            persona.voice.vocabulary.join(", ")
        );
        let packed = encode_json_bytes_as_tight_m8(&json_bytes, 42.73, &soul_md)?;
        fs::write(&m8_path, &packed)?;
        let size = packed.len() as u64;

        Ok((persona, m8_path, size))
    }

    /// Load a trained persona by name or slug from `~/.st/personas/<slug>.m8`
    pub fn load(name_or_slug: &str) -> Result<Self> {
        let clean = name_or_slug
            .strip_prefix("persona:")
            .or_else(|| name_or_slug.strip_prefix("persona/"))
            .unwrap_or(name_or_slug);
        let slug = Self::slugify(clean);
        for dir in Self::all_personas_dirs() {
            let candidate = dir.join(format!("{slug}.m8"));
            if candidate.is_file() {
                return Self::load_from_file(&candidate);
            }
        }
        let fallback = Self::personas_dir()?.join(format!("{slug}.m8"));
        Self::load_from_file(&fallback)
    }

    /// Load a trained persona from a specific `.m8` file path
    pub fn load_from_file(path: &Path) -> Result<Self> {
        let raw = fs::read(path)
            .with_context(|| format!("Persona file not found: {}", path.display()))?;
        let unpacked = decode_m8_bytes(&raw)?;
        let persona: Self = serde_json::from_slice(&unpacked)
            .context("Failed to decode LifePersona from .m8 container")?;
        Ok(persona)
    }

    /// List all trained personas in `~/.st/personas/`
    pub fn list_all() -> Result<Vec<PersonaSummary>> {
        let mut summaries = Vec::new();
        let mut seen_slugs = HashSet::new();
        for dir in Self::all_personas_dirs() {
            let Ok(entries) = fs::read_dir(dir) else {
                continue;
            };
            for entry in entries.flatten() {
                let path = entry.path();
                if path.extension().and_then(|e| e.to_str()) != Some("m8") {
                    continue;
                }
                let size = entry.metadata().map(|m| m.len()).unwrap_or(0);
                if let Ok(p) = Self::load_from_file(&path) {
                    if seen_slugs.insert(p.slug.clone()) {
                        summaries.push(PersonaSummary {
                            name: p.name,
                            slug: p.slug,
                            source_path: p.source_path,
                            trained_at: p.trained_at,
                            files_ingested: p.files_ingested,
                            total_words: p.total_words,
                            episode_count: p.episodes.len(),
                            m8_size_bytes: size,
                            sample_phrases: p.voice.signature_phrases.into_iter().take(6).collect(),
                        });
                    }
                }
            }
        }
        summaries.sort_by_key(|b| std::cmp::Reverse(b.trained_at.clone()));
        Ok(summaries)
    }

    /// Retrieve top-K resonant episodic memories for a query using wave + keyword salience
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

        let mut scored: Vec<(f32, &PersonaEpisode)> = self
            .episodes
            .iter()
            .map(|ep| {
                let mut score = 0.0f32;
                let lower_passage = ep.passage.to_lowercase();
                for qt in &query_tokens {
                    if ep.keywords.iter().any(|k| k == qt) {
                        score += 4.0;
                    } else if lower_passage.contains(qt.as_str()) {
                        score += 2.5;
                    }
                    if ep.source.to_lowercase().contains(qt.as_str()) {
                        score += 2.0;
                    }
                    if let Some(ref era) = ep.era {
                        if era.to_lowercase().contains(qt.as_str()) {
                            score += 3.5;
                        }
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
    pub fn respond_locally(&self, prompt: &str) -> String {
        let recalled = self.recall_episodes(prompt, 3);
        let warmth_tone = if self.voice.warmth >= 0.15 {
            "warm and reflective"
        } else if self.voice.warmth <= -0.15 {
            "candid and earnest"
        } else {
            "thoughtful and measured"
        };

        let mut out = String::new();
        out.push_str(&format!(
            "🧬 **[{} · MEM8 Persona Soul ({} episodes, {:.1} words/sentence, {} tone)]**\n\n",
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
            out.push_str(&format!(
                "From my notes in `{}`{}:\n> \"{}\"\n\n",
                primary.source,
                era_badge,
                primary.passage.replace('\n', "\n> ")
            ));
        }

        if recalled.len() > 1 {
            out.push_str("**Connected memories that resonate with what you asked:**\n");
            for ep in recalled.iter().skip(1) {
                let era_badge = ep
                    .era
                    .as_deref()
                    .map(|e| format!(" [{e}]"))
                    .unwrap_or_default();
                let snippet: String = ep.passage.chars().take(240).collect();
                out.push_str(&format!(
                    "- **`{}`**{}: \"{}{}\"\n",
                    ep.source,
                    era_badge,
                    snippet.replace('\n', " "),
                    if ep.passage.len() > 240 { "..." } else { "" }
                ));
            }
            out.push('\n');
        }

        if !self.voice.signature_phrases.is_empty() {
            let phrases: Vec<&str> = self
                .voice
                .signature_phrases
                .iter()
                .take(5)
                .map(String::as_str)
                .collect();
            out.push_str(&format!("_Signature cadence: {}_", phrases.join(" · ")));
        }

        out
    }

    /// Build a grounding system prompt for augmenting an external or local LLM
    pub fn build_system_prompt(&self, user_query: &str) -> String {
        let recalled = self.recall_episodes(user_query, 4);
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
            "You are speaking as the MEM8 Life Persona '{}'.\n\
             Voice profile: {:.1} words/sentence average, warmth={:.2}, expressiveness={:.2}.\n\
             Characteristic vocabulary: {}.\n\
             Signature phrases: {}.\n\
             Ground your answer strictly in the following authentic episodic memories from their life corpus:\n\n{}",
            self.name,
            self.voice.avg_sentence_words,
            self.voice.warmth,
            self.voice.expressiveness,
            self.voice.vocabulary.iter().take(20).cloned().collect::<Vec<_>>().join(", "),
            self.voice.signature_phrases.iter().take(10).cloned().collect::<Vec<_>>().join(", "),
            memories_block
        )
    }
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
    // Check for YYYY-MM-DD or explicit year 1880..2030
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
             Grandfather fixed the porch swing and we drankiced tea while listening to the rain.\n\n\
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
