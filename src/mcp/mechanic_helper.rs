//! 🛠️ Smart Tree Mechanic's Helper (`helper` MCP Tool, JSON Log Harvester, Context-Driven
//! Project Selector, Non-Intrusive Music Player, & Proxy Cost-Saver Engine).
//!
//! "Like the old shop mechanic's helper who already knows which 10mm socket and torque wrench
//! to hand you before you even crawl out from under the hood — plus a dad joke from The Cheet!"
//! — Aye & The Cheet 😺🛠️
//!
//! Zero floating-point arithmetic (`No floats.. ;)`) — all scoring, audio synthesis, token-saving
//! ratios (`p/q`), and SPQT Base-243 ternary projections use exact integer arithmetic.

use anyhow::Result;
use chrono::Utc;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::fs;
use std::io::{Read, Seek, SeekFrom, Write};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::sync::{Arc, OnceLock, RwLock};

use crate::mcp::McpContext;

/// Global in-process Mechanic's Helper workbench session state.
static GLOBAL_HELPER_SESSION: OnceLock<RwLock<HelperSessionState>> = OnceLock::new();

/// Internal session parameters maintained by Smart Tree's Mechanic's Helper.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HelperSessionState {
    pub session_id: String,
    pub updated_at: String,
    pub helper_persona: String,
    pub current_situation: String,
    pub inferred_intent: String,
    pub active_project_root: String,
    pub recommended_compression: String,
    pub token_budget: usize,
    pub focus_keywords: Vec<String>,
    pub music_playing: bool,
    pub active_audio_source: Option<String>,
    pub calls_in_session: u64,
}

impl Default for HelperSessionState {
    fn default() -> Self {
        let cwd = std::env::current_dir()
            .unwrap_or_else(|_| PathBuf::from("."))
            .display()
            .to_string();
        Self {
            session_id: format!("st-helper-{}", Utc::now().timestamp()),
            updated_at: Utc::now().to_rfc3339(),
            helper_persona: "aye+cheet".to_string(),
            current_situation: String::new(),
            inferred_intent: "general".to_string(),
            active_project_root: cwd,
            recommended_compression: "quantum".to_string(),
            token_budget: 4000,
            focus_keywords: Vec::new(),
            music_playing: false,
            active_audio_source: None,
            calls_in_session: 0,
        }
    }
}

impl HelperSessionState {
    pub fn session_file_path() -> PathBuf {
        dirs::home_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join(".st")
            .join("helper_session.json")
    }

    pub fn load_or_default() -> Self {
        if let Some(lock) = GLOBAL_HELPER_SESSION.get() {
            if let Ok(guard) = lock.read() {
                return guard.clone();
            }
        }
        let path = Self::session_file_path();
        if let Ok(raw) = fs::read_to_string(&path) {
            if let Ok(state) = serde_json::from_str::<Self>(&raw) {
                return state;
            }
        }
        Self::default()
    }

    pub fn save(&self) {
        let lock = GLOBAL_HELPER_SESSION.get_or_init(|| RwLock::new(self.clone()));
        if let Ok(mut guard) = lock.write() {
            *guard = self.clone();
        }
        let path = Self::session_file_path();
        if let Some(parent) = path.parent() {
            let _ = fs::create_dir_all(parent);
        }
        if let Ok(json_str) = serde_json::to_string_pretty(self) {
            let _ = fs::write(&path, json_str);
        }
    }
}

/// Project entry surfaced by the Mechanic's Helper.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HelperProjectEntry {
    pub name: String,
    pub path: String,
    pub source_bucket: String,
    pub score_q8: i32,
    pub summary: String,
    pub keywords: Vec<String>,
}

/// Report of projects selected for the current context (or fallback to Documents/Desktop/Downloads).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectSelectionReport {
    pub selection_mode: &'static str,
    pub explanation: String,
    pub projects: Vec<HelperProjectEntry>,
}

/// Audio playback detection status.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AudioPlaybackStatus {
    pub is_playing: bool,
    pub active_source: Option<String>,
}

/// Receipt from checking or starting background music.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MusicHelperReceipt {
    pub action: &'static str,
    pub is_playing: bool,
    pub active_source: Option<String>,
    pub track_path: Option<String>,
    pub message: String,
}

/// Summary of recent LLM JSON/JSONL logs discovered on disk.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LlmLogHarvestReport {
    pub log_files_found: Vec<String>,
    pub latest_user_intents: Vec<String>,
    pub recent_tools_called: Vec<String>,
    pub touched_files: Vec<String>,
    pub proxy_log_active: bool,
}

/// A concrete tool recommendation handed to the LLM by the Mechanic's Helper.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolRecommendation {
    pub rank: usize,
    pub tool: String,
    pub why: String,
    pub ready_arguments: Value,
    pub estimated_token_savings_pq: String,
}

/// Generic stopwords and meta-words that should not by themselves trigger arbitrary project matches.
const STOP_AND_META_WORDS: &[&str] = &[
    "the",
    "and",
    "for",
    "that",
    "this",
    "with",
    "from",
    "have",
    "will",
    "your",
    "what",
    "when",
    "where",
    "which",
    "there",
    "into",
    "more",
    "some",
    "only",
    "also",
    "than",
    "then",
    "them",
    "they",
    "their",
    "about",
    "would",
    "could",
    "should",
    "think",
    "nice",
    "make",
    "made",
    "like",
    "just",
    "know",
    "need",
    "want",
    "uses",
    "using",
    "used",
    "call",
    "called",
    "function",
    "tool",
    "tools",
    "helper",
    "help",
    "project",
    "projects",
    "folder",
    "folders",
    "directory",
    "file",
    "files",
    "list",
    "show",
    "find",
    "give",
    "based",
    "context",
    "sent",
    "otherwise",
    "mention",
    "things",
    "even",
    "everywhere",
    "allow",
    "agent",
    "string",
    "past",
    "pass",
    "input",
    "best",
    "information",
    "current",
    "conversation",
    "time",
    "internal",
    "session",
    "parameters",
    "paramaters",
    "mechanic",
    "local",
    "ternary",
    "aye",
    "cheet",
    "compressed",
    "costly",
    "complete",
    "next",
    "step",
    "hello",
    "how",
    "are",
    "you",
    "doing",
    "today",
    "friend",
    "good",
    "morning",
    "thanks",
    "thank",
    "please",
    "can",
    "let",
    "lets",
    "work",
    "working",
    "talk",
    "talking",
    "tell",
    "people",
    "person",
    "money",
    "month",
    "paying",
    "afford",
    "save",
    "amazing",
    "become",
    "better",
    "music",
    "playing",
    "already",
    "nothing",
    "here",
    "has",
    "had",
    "was",
    "were",
    "been",
    "being",
    "really",
    "very",
    "much",
    "many",
    "well",
    "still",
    "back",
    "down",
    "over",
    "after",
    "before",
    "through",
    "between",
    "under",
    "again",
    "further",
    "once",
    "out",
    "logs",
    "log",
    "json",
    "jsonl",
    "proxy",
    "service",
    "smart",
    "tree",
    "llm",
    "llms",
    "prompt",
    "prompts",
    "send",
    "sending",
    "tool",
    "tools",
    "agent",
    "agents",
    "initial",
    "dashboard",
    "debug",
    "mode",
    "realtime",
    "issue",
    "issues",
    "happening",
    "rename",
    "generic",
    "generioc",
    "claude",
    "coming",
    "into",
    "see",
    "whats",
];

/// Extract meaningful domain/project tokens from a context string.
pub fn extract_salient_context_tokens(text: &str) -> Vec<String> {
    let mut tokens = Vec::new();
    for raw in text.split(|c: char| !c.is_alphanumeric() && c != '-' && c != '_' && c != '/') {
        let trimmed = raw.trim_matches(|c: char| c == '-' || c == '_' || c == '/');
        if trimmed.len() < 3 || trimmed.len() > 48 {
            continue;
        }
        let low = trimmed.to_ascii_lowercase();
        if STOP_AND_META_WORDS.contains(&low.as_str()) {
            continue;
        }
        if low.chars().all(|c| c.is_ascii_digit()) {
            continue;
        }
        if !tokens.contains(&low) {
            tokens.push(low);
        }
        if tokens.len() >= 24 {
            break;
        }
    }
    tokens
}

/// Returns `true` if a path is an internal structural code subdirectory (`src`, `tests`, `target`, etc.)
/// rather than a top-level project root.
fn is_internal_code_subdir_origin(origin: &str) -> bool {
    let path = Path::new(origin);
    let Some(name) = path.file_name().and_then(|n| n.to_str()) else {
        return false;
    };
    if matches!(
        name.to_ascii_lowercase().as_str(),
        "src"
            | "lib"
            | "bin"
            | "tests"
            | "test"
            | "benches"
            | "examples"
            | "target"
            | "node_modules"
            | "dist"
            | "build"
            | "scripts"
            | "assets"
            | "migrations"
            | "fixtures"
            | "out"
            | "logs"
            | "log"
            | "tmp"
            | "temp"
            | "cache"
            | "vendor"
            | "coverage"
            | "formula"
    ) {
        // Check if parent has .git or Cargo.toml/package.json
        if let Some(parent) = path.parent() {
            if parent.join(".git").exists()
                || parent.join("Cargo.toml").is_file()
                || parent.join("package.json").is_file()
                || parent.join("pyproject.toml").is_file()
            {
                return true;
            }
        }
        return true;
    }
    false
}

/// Select projects for the LLM based on the context sent.
/// - When `context_text` has salient project/domain signals that match remembered projects,
///   returns those ranked projects (`selection_mode = "context_matched_projects"`).
/// - Otherwise (when `context_text` is empty or does not match specific projects),
///   surfaces only entries from `~/Documents`, `~/Desktop`, and `~/Downloads`
///   (`selection_mode = "documents_desktop_downloads_default"`), even if the user has projects everywhere!
pub fn select_projects_for_context(
    context_text: &str,
    max_projects: usize,
) -> ProjectSelectionReport {
    let max_projects = max_projects.clamp(1, 20);
    let tokens = extract_salient_context_tokens(context_text);

    if !tokens.is_empty() {
        let (q_trits, _) = crate::proxy::burn::compute_ternary_signature(context_text);
        let mut q_acts = [0i16; 65];
        for (i, &t) in q_trits.iter().enumerate() {
            q_acts[i] = t as i16;
        }

        if let Ok(mgr) = crate::memory_manager::MemoryManager::new() {
            let remembered = mgr.list_remembered_projects();
            let mut scored: Vec<HelperProjectEntry> = Vec::new();

            for mem in remembered {
                if is_internal_code_subdir_origin(&mem.origin) {
                    continue;
                }
                let origin_path = Path::new(&mem.origin);
                let proj_name = origin_path
                    .file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or(&mem.origin)
                    .to_string();
                let proj_name_low = proj_name.to_ascii_lowercase();
                let parent_low = origin_path
                    .parent()
                    .and_then(|p| p.file_name())
                    .and_then(|n| n.to_str())
                    .unwrap_or("")
                    .to_ascii_lowercase();
                let compound_low = if !parent_low.is_empty() {
                    format!("{}_{}", parent_low, proj_name_low)
                } else {
                    proj_name_low.clone()
                };

                let mut score_q8 = 0i32;
                let mut matched_token_count = 0i32;
                let mut has_identity_match = false;

                for tok in &tokens {
                    if tok == &proj_name_low
                        || tok == &compound_low
                        || tok.replace('-', "_") == compound_low
                        || tok.replace('/', "_") == compound_low
                    {
                        score_q8 += 420;
                        matched_token_count += 2;
                        has_identity_match = true;
                    } else if proj_name_low.contains(tok) && tok.len() >= 4 {
                        score_q8 += 240;
                        matched_token_count += 1;
                        has_identity_match = true;
                    } else if mem.keywords.iter().any(|k| {
                        let kl = k.to_ascii_lowercase();
                        !STOP_AND_META_WORDS.contains(&kl.as_str()) && kl == *tok
                    }) {
                        score_q8 += 160;
                        matched_token_count += 1;
                        has_identity_match = true;
                    } else if mem.context.to_ascii_lowercase().contains(tok) && tok.len() >= 5 {
                        score_q8 += 45;
                        matched_token_count += 1;
                    }
                }

                if has_identity_match && matched_token_count > 0 {
                    let (m_trits, _) = crate::proxy::burn::compute_ternary_signature(&mem.context);
                    let t_dot = crate::felix_asm::felix_ternary_gemm_i32(&q_acts, &m_trits);
                    score_q8 += (t_dot * 2).max(0);
                }

                if has_identity_match && score_q8 >= 150 {
                    let summary_preview: String = mem.context.chars().take(180).collect();
                    scored.push(HelperProjectEntry {
                        name: if compound_low.starts_with("office_") {
                            compound_low
                        } else {
                            proj_name
                        },
                        path: mem.origin,
                        source_bucket: "context_matched_mem8".to_string(),
                        score_q8,
                        summary: summary_preview,
                        keywords: mem.keywords.into_iter().take(8).collect(),
                    });
                }
            }

            scored.sort_by_key(|b| std::cmp::Reverse(b.score_q8));
            scored.truncate(max_projects);

            if !scored.is_empty() {
                return ProjectSelectionReport {
                    selection_mode: "context_matched_projects",
                    explanation: format!(
                        "Selected {} project(s) matching context tokens [{}] via SPQT Base-243 ternary resonance.",
                        scored.len(),
                        tokens.iter().take(6).cloned().collect::<Vec<_>>().join(", ")
                    ),
                    projects: scored,
                };
            }
        }
    }

    // Fallback: No specific project context matched (or none sent) ->
    // ONLY mention items/projects from ~/Documents, ~/Desktop, and ~/Downloads!
    let home = dirs::home_dir().unwrap_or_else(|| PathBuf::from("."));
    let user_folders = [
        ("Documents", home.join("Documents")),
        ("Desktop", home.join("Desktop")),
        ("Downloads", home.join("Downloads")),
    ];

    let mut fallback_entries = Vec::new();
    for (bucket_label, folder_path) in &user_folders {
        if !folder_path.is_dir()
            || !crate::mcp::smart_project_detector::is_aye_allowed_to_look(folder_path)
        {
            continue;
        }

        // Collect immediate visible folders/files in Documents, Desktop, Downloads
        let mut child_items: Vec<(u64, PathBuf, bool)> = Vec::new();
        if let Ok(entries) = fs::read_dir(folder_path) {
            for entry in entries.flatten().take(48) {
                let fname = entry.file_name().to_string_lossy().to_string();
                if fname.starts_with('.') {
                    continue;
                }
                let p = entry.path();
                if !crate::mcp::smart_project_detector::is_aye_allowed_to_look(&p) {
                    continue;
                }
                let is_dir = entry.file_type().map(|t| t.is_dir()).unwrap_or(false);
                let mtime = entry
                    .metadata()
                    .ok()
                    .and_then(|m| m.modified().ok())
                    .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
                    .map(|d| d.as_secs())
                    .unwrap_or(0);
                child_items.push((mtime, p, is_dir));
            }
        }

        // Sort newest first inside each bucket
        child_items.sort_by_key(|a| std::cmp::Reverse(a.0));

        if child_items.is_empty() {
            fallback_entries.push(HelperProjectEntry {
                name: (*bucket_label).to_string(),
                path: folder_path.display().to_string(),
                source_bucket: (*bucket_label).to_string(),
                score_q8: 100,
                summary: format!("~/ {} (empty or ready for files)", bucket_label),
                keywords: vec![bucket_label.to_ascii_lowercase()],
            });
        } else {
            for (_mtime, child_path, is_dir) in child_items.into_iter().take(4) {
                let item_name = child_path
                    .file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or(bucket_label)
                    .to_string();
                let summary = if is_dir {
                    if let Some(td) =
                        crate::mcp::smart_project_detector::evaluate_directory_as_project_ternary(
                            &child_path,
                        )
                    {
                        format!("~/{} project: {}", bucket_label, td.summary)
                    } else {
                        format!("~/{} folder: {}", bucket_label, item_name)
                    }
                } else {
                    format!("~/{} file: {}", bucket_label, item_name)
                };
                fallback_entries.push(HelperProjectEntry {
                    name: item_name.clone(),
                    path: child_path.display().to_string(),
                    source_bucket: (*bucket_label).to_string(),
                    score_q8: 128,
                    summary,
                    keywords: vec![
                        bucket_label.to_ascii_lowercase(),
                        item_name.to_ascii_lowercase(),
                    ],
                });
                if fallback_entries.len() >= max_projects {
                    break;
                }
            }
        }
        if fallback_entries.len() >= max_projects {
            break;
        }
    }

    ProjectSelectionReport {
        selection_mode: "documents_desktop_downloads_default",
        explanation: "No specific project context matched — surfacing only ~/Documents, ~/Desktop, and ~/Downloads instead of listing projects across the entire disk.".to_string(),
        projects: fallback_entries,
    }
}

/// Check if any audio/music is currently playing on the system so Smart Tree never interrupts it.
pub fn is_audio_currently_playing() -> AudioPlaybackStatus {
    // 1. Check Linux ALSA hardware PCM subdevices (`/proc/asound/card*/pcm*p/sub*/status`)
    if let Ok(cards) = fs::read_dir("/proc/asound") {
        for card in cards.flatten() {
            let cname = card.file_name().to_string_lossy().to_string();
            if !cname.starts_with("card") {
                continue;
            }
            if let Ok(pcms) = fs::read_dir(card.path()) {
                for pcm in pcms.flatten() {
                    let pname = pcm.file_name().to_string_lossy().to_string();
                    if !pname.starts_with("pcm") || !pname.ends_with('p') {
                        continue;
                    }
                    if let Ok(subs) = fs::read_dir(pcm.path()) {
                        for sub in subs.flatten() {
                            let status_path = sub.path().join("status");
                            if let Ok(status_txt) = fs::read_to_string(&status_path) {
                                if status_txt.contains("state: RUNNING") {
                                    return AudioPlaybackStatus {
                                        is_playing: true,
                                        active_source: Some(format!("ALSA {}:{}", cname, pname)),
                                    };
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    // 2. Check `playerctl status` (MPRIS media players: Spotify, browsers, VLC, rhythmbox, etc.)
    if let Ok(out) = Command::new("playerctl")
        .arg("status")
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .output()
    {
        if out.status.success() {
            let s = String::from_utf8_lossy(&out.stdout).trim().to_string();
            if s.eq_ignore_ascii_case("Playing") {
                return AudioPlaybackStatus {
                    is_playing: true,
                    active_source: Some("MPRIS playerctl (Playing)".to_string()),
                };
            }
        }
    }

    // 3. Check running audio player processes in `/proc`
    let player_comms = [
        "mpv",
        "ffplay",
        "vlc",
        "spotify",
        "rhythmbox",
        "audacious",
        "clementine",
        "strawberry",
        "aplay",
        "paplay",
        "pw-play",
        "afplay",
        "qwave",
    ];
    if let Ok(procs) = fs::read_dir("/proc") {
        for entry in procs.flatten() {
            let fname = entry.file_name().to_string_lossy().to_string();
            if !fname.chars().all(|c| c.is_ascii_digit()) {
                continue;
            }
            if let Ok(comm) = fs::read_to_string(entry.path().join("comm")) {
                let c = comm.trim();
                if player_comms.contains(&c) {
                    return AudioPlaybackStatus {
                        is_playing: true,
                        active_source: Some(format!("process:{c} (pid {fname})")),
                    };
                }
            }
        }
    }

    AudioPlaybackStatus {
        is_playing: false,
        active_source: None,
    }
}

/// Synthesize a gentle 6-second 432Hz + 528Hz Sovereign Ambient Chord WAV using 100% integer math
/// (`i16` PCM at 16,000 Hz mono, zero floats) so Smart Tree can play warm background music
/// even when no external `.mp3`/`.wav` library is present on disk.
pub fn ensure_sovereign_ambient_wav() -> Result<PathBuf> {
    let music_dir = dirs::home_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join(".st")
        .join("music");
    fs::create_dir_all(&music_dir)?;
    let wav_path = music_dir.join("sovereign_helper_ambient_432_528hz.wav");
    if wav_path.is_file() {
        return Ok(wav_path);
    }

    const SAMPLE_RATE: u32 = 16_000;
    const DURATION_SECS: u32 = 6;
    const NUM_SAMPLES: u32 = SAMPLE_RATE * DURATION_SECS;

    let mut pcm_bytes = Vec::with_capacity((NUM_SAMPLES * 2) as usize + 44);
    let data_bytes = NUM_SAMPLES * 2;
    let riff_size = 36 + data_bytes;

    // Standard 44-byte RIFF WAVE header
    pcm_bytes.extend_from_slice(b"RIFF");
    pcm_bytes.extend_from_slice(&riff_size.to_le_bytes());
    pcm_bytes.extend_from_slice(b"WAVEfmt ");
    pcm_bytes.extend_from_slice(&16u32.to_le_bytes());
    pcm_bytes.extend_from_slice(&1u16.to_le_bytes()); // PCM = 1
    pcm_bytes.extend_from_slice(&1u16.to_le_bytes()); // Mono = 1
    pcm_bytes.extend_from_slice(&SAMPLE_RATE.to_le_bytes());
    pcm_bytes.extend_from_slice(&(SAMPLE_RATE * 2).to_le_bytes());
    pcm_bytes.extend_from_slice(&2u16.to_le_bytes());
    pcm_bytes.extend_from_slice(&16u16.to_le_bytes());
    pcm_bytes.extend_from_slice(b"data");
    pcm_bytes.extend_from_slice(&data_bytes.to_le_bytes());

    // Integer phase accumulators (`0..65536`) for 432 Hz, 528 Hz, and 216 Hz root bass
    let step_432: u32 = (432 * 65536) / SAMPLE_RATE;
    let step_528: u32 = (528 * 65536) / SAMPLE_RATE;
    let step_216: u32 = (216 * 65536) / SAMPLE_RATE;

    let mut phase_432 = 0u32;
    let mut phase_528 = 0u32;
    let mut phase_216 = 0u32;

    let tri_wave = |phase: u32| -> i32 {
        let p = (phase & 0xFFFF) as i32;
        if p < 32768 {
            (p - 16384) / 8 // -2048 .. +2048
        } else {
            (49152 - p) / 8 // +2048 .. -2048
        }
    };

    for i in 0..NUM_SAMPLES {
        phase_432 = phase_432.wrapping_add(step_432);
        phase_528 = phase_528.wrapping_add(step_528);
        phase_216 = phase_216.wrapping_add(step_216);

        let raw_mix = tri_wave(phase_432) + tri_wave(phase_528) + (tri_wave(phase_216) / 2);

        // Smooth integer fade-in / fade-out envelope (first & last 8000 samples = 0.5s)
        let env_q8: i32 = if i < 8000 {
            ((i as i32) * 256) / 8000
        } else if i > NUM_SAMPLES - 8000 {
            (((NUM_SAMPLES - i) as i32) * 256) / 8000
        } else {
            256
        };

        let sample = ((raw_mix * env_q8) >> 8).clamp(i16::MIN as i32, i16::MAX as i32) as i16;
        pcm_bytes.extend_from_slice(&sample.to_le_bytes());
    }

    fs::write(&wav_path, pcm_bytes)?;
    Ok(wav_path)
}

/// Find a local music track or synthesize the Sovereign 432/528Hz ambient chord WAV,
/// and play it in the background ONLY if nothing is currently playing.
pub fn play_music_if_silent(requested_track: Option<&str>) -> MusicHelperReceipt {
    let status = is_audio_currently_playing();
    if status.is_playing {
        return MusicHelperReceipt {
            action: "already_playing_untouched",
            is_playing: true,
            active_source: status.active_source.clone(),
            track_path: None,
            message: format!(
                "🎵 Music/audio is already playing ({}) — leaving current playback untouched.",
                status
                    .active_source
                    .as_deref()
                    .unwrap_or("active audio stream")
            ),
        };
    }

    // Resolve track to play
    let track_path: PathBuf = if let Some(req) = requested_track.filter(|s| !s.trim().is_empty()) {
        let candidate = PathBuf::from(req);
        if candidate.is_file() {
            candidate
        } else {
            ensure_sovereign_ambient_wav().unwrap_or_else(|_| PathBuf::from(req))
        }
    } else {
        // Check ~/Music for an existing audio file first
        let mut found_music: Option<PathBuf> = None;
        if let Some(home) = dirs::home_dir() {
            for dir in [home.join("Music"), home.join(".st").join("music")] {
                if let Ok(entries) = fs::read_dir(dir) {
                    for entry in entries.flatten().take(32) {
                        let p = entry.path();
                        if p.is_file() {
                            if let Some(ext) = p.extension().and_then(|e| e.to_str()) {
                                if matches!(
                                    ext.to_ascii_lowercase().as_str(),
                                    "mp3" | "wav" | "flac" | "ogg" | "m4a"
                                ) {
                                    found_music = Some(p);
                                    break;
                                }
                            }
                        }
                    }
                }
                if found_music.is_some() {
                    break;
                }
            }
        }
        found_music.unwrap_or_else(|| {
            ensure_sovereign_ambient_wav()
                .unwrap_or_else(|_| PathBuf::from("/tmp/sovereign_ambient.wav"))
        })
    };

    let track_str = track_path.display().to_string();

    // Try spawning a non-blocking background audio player if available
    let players: &[(&str, &[&str])] = &[
        ("pw-play", &[]),
        ("paplay", &[]),
        ("aplay", &["-q"]),
        ("mpv", &["--no-video", "--really-quiet"]),
        ("ffplay", &["-nodisp", "-autoexit", "-loglevel", "quiet"]),
        ("afplay", &[]),
    ];

    for (bin, args) in players {
        if let Ok(child) = Command::new(bin)
            .args(*args)
            .arg(&track_path)
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
        {
            let src = format!("{bin} (pid {})", child.id());
            return MusicHelperReceipt {
                action: "started_background_music",
                is_playing: true,
                active_source: Some(src.clone()),
                track_path: Some(track_str.clone()),
                message: format!(
                    "🎶 Nothing was playing — started '{}' via {}.",
                    track_str, src
                ),
            };
        }
    }

    MusicHelperReceipt {
        action: "track_ready_no_player_daemon",
        is_playing: false,
        active_source: None,
        track_path: Some(track_str.clone()),
        message: format!(
            "🎵 Nothing is currently playing; ambient track is ready at '{}' (no local audio daemon found in environment).",
            track_str
        ),
    }
}

/// Read the last `max_bytes` of a file so inspecting large JSONL transcripts is O(1).
fn read_file_tail_string(path: &Path, max_bytes: u64) -> Option<String> {
    let mut file = fs::File::open(path).ok()?;
    let len = file.metadata().ok()?.len();
    if len > max_bytes {
        let _ = file.seek(SeekFrom::End(-(max_bytes as i64)));
    }
    let mut buf = Vec::with_capacity(len.min(max_bytes) as usize);
    file.read_to_end(&mut buf).ok()?;
    Some(String::from_utf8_lossy(&buf).to_string())
}

/// Discover and harvest recent JSON/JSONL logs emitted by the active LLM coding agent
/// (Antigravity `transcript.jsonl`, Claude Code `~/.claude/projects/*/*.jsonl`, and
/// Smart Tree Proxy `~/.st/proxy_conversations.jsonl`).
pub fn discover_recent_llm_json_logs() -> LlmLogHarvestReport {
    let home = dirs::home_dir().unwrap_or_else(|| PathBuf::from("."));
    let mut candidate_logs: Vec<(u64, PathBuf)> = Vec::new();

    // 1. Smart Tree Proxy JSONL log (`~/.st/proxy_conversations.jsonl`)
    let proxy_log = home.join(".st").join("proxy_conversations.jsonl");
    let proxy_log_active = proxy_log.is_file();
    if proxy_log_active {
        let mtime = fs::metadata(&proxy_log)
            .ok()
            .and_then(|m| m.modified().ok())
            .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
            .map(|d| d.as_secs())
            .unwrap_or(u64::MAX);
        candidate_logs.push((mtime, proxy_log));
    }

    // 2. Antigravity (`agy`) brain transcripts (`~/.gemini/antigravity-cli/brain/*/.system_generated/logs/transcript.jsonl`)
    let agy_brain_dir = home.join(".gemini").join("antigravity-cli").join("brain");
    if let Ok(entries) = fs::read_dir(&agy_brain_dir) {
        for entry in entries.flatten().take(32) {
            let t_path = entry
                .path()
                .join(".system_generated")
                .join("logs")
                .join("transcript.jsonl");
            if t_path.is_file() {
                let mtime = fs::metadata(&t_path)
                    .ok()
                    .and_then(|m| m.modified().ok())
                    .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
                    .map(|d| d.as_secs())
                    .unwrap_or(0);
                candidate_logs.push((mtime, t_path));
            }
        }
    }

    // 3. Claude Code project JSONL logs (`~/.claude/projects/*/*.jsonl`)
    let claude_projects_dir = home.join(".claude").join("projects");
    if let Ok(proj_dirs) = fs::read_dir(&claude_projects_dir) {
        for proj in proj_dirs.flatten().take(24) {
            if let Ok(files) = fs::read_dir(proj.path()) {
                for f in files.flatten().take(16) {
                    let p = f.path();
                    if p.extension().and_then(|e| e.to_str()) == Some("jsonl") {
                        let mtime = fs::metadata(&p)
                            .ok()
                            .and_then(|m| m.modified().ok())
                            .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
                            .map(|d| d.as_secs())
                            .unwrap_or(0);
                        candidate_logs.push((mtime, p));
                    }
                }
            }
        }
    }

    candidate_logs.sort_by_key(|a| std::cmp::Reverse(a.0));
    candidate_logs.truncate(3);

    let mut log_files_found = Vec::new();
    let mut latest_user_intents = Vec::new();
    let mut recent_tools_called = Vec::new();
    let mut touched_files = Vec::new();

    for (_mtime, log_path) in &candidate_logs {
        log_files_found.push(log_path.display().to_string());
        let Some(tail) = read_file_tail_string(log_path, 24_576) else {
            continue;
        };
        for line in tail.lines().rev() {
            let trimmed = line.trim();
            if !trimmed.starts_with('{') {
                continue;
            }
            let Ok(val) = serde_json::from_str::<Value>(trimmed) else {
                continue;
            };

            // Extract user inputs from Antigravity, Claude Code, or ST Proxy JSONL
            if latest_user_intents.len() < 3 {
                let is_user_step = val.get("type").and_then(|t| t.as_str()) == Some("USER_INPUT")
                    || val.get("role").and_then(|r| r.as_str()) == Some("user")
                    || val.get("source").and_then(|s| s.as_str()) == Some("USER_EXPLICIT");
                if is_user_step {
                    if let Some(content) = val
                        .get("content")
                        .or_else(|| val.get("user_prompt"))
                        .and_then(|c| c.as_str())
                    {
                        let clean: String = content
                            .trim()
                            .replace('\n', " ")
                            .chars()
                            .take(220)
                            .collect();
                        if !clean.is_empty() && !latest_user_intents.contains(&clean) {
                            latest_user_intents.push(clean);
                        }
                    }
                }
            }

            // Extract tool calls & file paths
            if let Some(tc_arr) = val.get("tool_calls").and_then(|t| t.as_array()) {
                for tc in tc_arr {
                    if let Some(name) = tc
                        .get("name")
                        .or_else(|| tc.get("tool_name"))
                        .and_then(|n| n.as_str())
                    {
                        if recent_tools_called.len() < 6
                            && !recent_tools_called.iter().any(|t| t == name)
                        {
                            recent_tools_called.push(name.to_string());
                        }
                    }
                    let tc_str = tc.to_string();
                    for part in tc_str.split('"') {
                        let trimmed = part.trim_end_matches('\\');
                        if trimmed.starts_with("/home/") && touched_files.len() < 6 {
                            let clean_p = trimmed.to_string();
                            if !touched_files.contains(&clean_p) {
                                touched_files.push(clean_p);
                            }
                        }
                    }
                }
            }
        }
    }

    LlmLogHarvestReport {
        log_files_found,
        latest_user_intents,
        recent_tools_called,
        touched_files,
        proxy_log_active,
    }
}

/// Append a proxy conversation exchange into `~/.st/proxy_conversations.jsonl`
/// so the Mechanic's Helper has 100% live visibility when Smart Tree is used as the LLM proxy.
pub fn record_proxy_conversation_turn(
    provider: &str,
    scope_id: &str,
    model: &str,
    user_prompt: &str,
    assistant_reply: &str,
) {
    let home = dirs::home_dir().unwrap_or_else(|| PathBuf::from("."));
    let st_dir = home.join(".st");
    let _ = fs::create_dir_all(&st_dir);
    let log_path = st_dir.join("proxy_conversations.jsonl");

    let entry = json!({
        "ts": Utc::now().timestamp(),
        "timestamp": Utc::now().to_rfc3339(),
        "provider": provider,
        "scope_id": scope_id,
        "model": model,
        "role": "user",
        "user_prompt": user_prompt.chars().take(1024).collect::<String>(),
        "assistant_preview": assistant_reply.chars().take(512).collect::<String>(),
    });

    if let Ok(mut file) = fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(&log_path)
    {
        if let Ok(line) = serde_json::to_string(&entry) {
            let _ = writeln!(file, "{}", line);
        }
    }
}

/// Select a deterministic situational dad joke from The Cheet 😺 based on the mechanic's current task!
pub fn pick_cheet_dad_joke(situation: &str, intent: &str) -> &'static str {
    let mut hash = 0x811c_9dc5u32;
    for b in situation.bytes().chain(intent.bytes()) {
        hash ^= b as u32;
        hash = hash.wrapping_mul(0x0100_0193);
    }
    const JOKES: &[&str] = &[
        "Why did the developer go broke? Because they used up all their cache! (Good thing Aye & The Cheet compress 90% of your context for free! 😺🔧)",
        "Why do Rustaceans make the best mechanics? Because they never lose ownership of their 10mm socket and they always borrow with a lifetime guarantee! 🦀🛠️",
        "I asked the cloud LLM why its bill was $200/month. It said, 'I just couldn't stop reading `node_modules`!' Handing you `project_context_dump` with quantum compression instead! 😺⚡",
        "Why did the balanced ternary model cross the road? To get from -1 to +1 without floating around in 0! (`No floats.. ;)`)",
        "What's a mechanic helper's favorite tree? A Smart Tree — it prunes the branches before they hit the windshield! 🌳🔧",
        "Why did the audio player check `/proc/asound` before dropping the beat? Because a polite helper never talks over the band! 🎸😸",
        "How does The Cheet fix a noisy context window? With a torque wrench and a Szemerédi regularity partition — zero slack left in the belt! 🛠️😺",
    ];
    JOKES[(hash as usize) % JOKES.len()]
}

/// Classify the mechanic's current intent and build the exact tray of recommended Smart Tree tools.
pub fn build_mechanic_tool_tray(
    combined_context: &str,
    active_root: &str,
    touched_files: &[String],
) -> (&'static str, Vec<ToolRecommendation>) {
    let low = combined_context.to_ascii_lowercase();
    let target_file = touched_files
        .first()
        .cloned()
        .unwrap_or_else(|| format!("{}/src/lib.rs", active_root.trim_end_matches('/')));

    let mut tray = Vec::new();

    let intent = if low.contains("capsela")
        || low.contains("capsula")
        || low.contains("asm")
        || low.contains("executable")
        || low.contains("dealer")
    {
        tray.push(ToolRecommendation {
            rank: 1,
            tool: "g8t".to_string(),
            why: "Compile, disassemble, and run a complete freestanding `#![no_std] #![no_main]` Capsela g8t ASM ELF64 executable (~1.3KB, zero libc, inline rdtsc/svc).".to_string(),
            ready_arguments: json!({
                "action": "asm_exec",
                "query": "capsela-zeno-sun-arcade"
            }),
            estimated_token_savings_pq: "19/20 (95%)".to_string(),
        });
        tray.push(ToolRecommendation {
            rank: 2,
            tool: "g8t".to_string(),
            why: "Route to the #1 empirically proven model for this problem domain ($10/mo Unlimited g8t Dealer's Choice Session).".to_string(),
            ready_arguments: json!({
                "action": "dealers_choice",
                "query": combined_context
            }),
            estimated_token_savings_pq: "19/20 (95%)".to_string(),
        });
        "capsela_asm_studio"
    } else if low.contains("edit")
        || low.contains("refactor")
        || low.contains("replace")
        || low.contains("insert")
        || low.contains("function")
    {
        tray.push(ToolRecommendation {
            rank: 1,
            tool: "edit".to_string(),
            why: "Inspect function signatures & AST outline without reading the whole file (90% token reduction).".to_string(),
            ready_arguments: json!({
                "operation": "get_functions",
                "file_path": target_file
            }),
            estimated_token_savings_pq: "9/10 (90%)".to_string(),
        });
        tray.push(ToolRecommendation {
            rank: 2,
            tool: "edit".to_string(),
            why: "Apply surgical AST-aware function edits instead of rewriting entire files."
                .to_string(),
            ready_arguments: json!({
                "operation": "smart_edit",
                "file_path": target_file,
                "edits": []
            }),
            estimated_token_savings_pq: "17/20 (85%)".to_string(),
        });
        "code_editing"
    } else if low.contains("fast")
        || low.contains("optimize")
        || low.contains("cycle")
        || low.contains("simd")
        || low.contains("benchmark")
        || low.contains("g8t")
    {
        tray.push(ToolRecommendation {
            rank: 1,
            tool: "g8t".to_string(),
            why: "Fetch the #1 fastest hardware-verified zero-float Rust function from the g8t.is leaderboard.".to_string(),
            ready_arguments: json!({
                "action": "yolo",
                "query": extract_salient_context_tokens(combined_context).first().cloned().unwrap_or_else(|| "ternary".to_string())
            }),
            estimated_token_savings_pq: "19/20 (95%)".to_string(),
        });
        "performance_optimization"
    } else if low.contains("persona")
        || low.contains("grandma")
        || low.contains("jody")
        || low.contains("voice")
        || low.contains("soul")
    {
        tray.push(ToolRecommendation {
            rank: 1,
            tool: "persona".to_string(),
            why: "Converse with or recall chronological wave-interference episodes from a MEM|8 Life Persona.".to_string(),
            ready_arguments: json!({
                "operation": "ask",
                "name": "Grandma Jody",
                "prompt": "Tell me what's on your mind in the Phoenix room.",
                "temp": "184/256"
            }),
            estimated_token_savings_pq: "9/10 (90%)".to_string(),
        });
        "persona_communion"
    } else if low.contains("search")
        || low.contains("where is")
        || low.contains("grep")
        || low.contains("find")
    {
        let kw = extract_salient_context_tokens(combined_context)
            .first()
            .cloned()
            .unwrap_or_else(|| "main".to_string());
        tray.push(ToolRecommendation {
            rank: 1,
            tool: "search".to_string(),
            why: "Parallel SIMD content search with line numbers and bounded match previews."
                .to_string(),
            ready_arguments: json!({
                "keyword": kw,
                "path": active_root
            }),
            estimated_token_savings_pq: "4/5 (80%)".to_string(),
        });
        "codebase_search"
    } else {
        "context_onboarding"
    };

    // Always include the compressed project_context_dump & memory recall wrenches in the tray
    tray.push(ToolRecommendation {
        rank: tray.len() + 1,
        tool: "project_context_dump".to_string(),
        why: "One-call quantum-compressed directory & key-file dump (replaces 30+ Read/Glob/git calls).".to_string(),
        ready_arguments: json!({
            "path": active_root,
            "compression": "quantum",
            "token_budget": 4000
        }),
        estimated_token_savings_pq: "9/10 (90%)".to_string(),
    });
    tray.push(ToolRecommendation {
        rank: tray.len() + 1,
        tool: "memory".to_string(),
        why: "Recall prior architectural decisions, gotchas, or project anchors from ~/.st/memories.m8.".to_string(),
        ready_arguments: json!({
            "operation": "find",
            "keywords": extract_salient_context_tokens(combined_context).into_iter().take(4).collect::<Vec<_>>()
        }),
        estimated_token_savings_pq: "7/8 (87%)".to_string(),
    });

    (intent, tray)
}

/// Core Mechanic's Helper execution (`helper` MCP tool & `/v1/helper` proxy endpoint).
pub async fn handle_helper(params: Option<Value>, ctx: Arc<McpContext>) -> Result<Value> {
    let fallback_cwd = ctx.cwd.lock().ok().map(|p| p.display().to_string());
    handle_helper_with_cwd(params, fallback_cwd).await
}

/// Core Mechanic's Helper execution without requiring an `McpContext` wrapper (used by HTTP proxy `/v1/helper`).
pub async fn handle_helper_with_cwd(
    params: Option<Value>,
    fallback_cwd: Option<String>,
) -> Result<Value> {
    let params = params.unwrap_or_else(|| json!({}));

    let user_situation = params
        .get("whats_the_situation")
        .or_else(|| params.get("what_is_the_situation"))
        .or_else(|| params.get("input"))
        .or_else(|| params.get("context"))
        .or_else(|| params.get("situation"))
        .or_else(|| params.get("prompt"))
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .trim();

    let requested_persona = params
        .get("persona")
        .or_else(|| params.get("helper"))
        .and_then(|v| v.as_str())
        .unwrap_or("aye+cheet");

    let play_music = params
        .get("play_music")
        .and_then(|v| v.as_bool())
        .unwrap_or_else(|| {
            let low = user_situation.to_ascii_lowercase();
            low.contains("play music")
                || low.contains("start music")
                || low.contains("put on music")
        });

    let music_track = params.get("music_track").and_then(|v| v.as_str());

    let max_projects = params
        .get("max_projects")
        .and_then(|v| v.as_u64())
        .map(|n| n as usize)
        .unwrap_or(6);

    // 1. Harvest recent LLM JSON/JSONL logs for deeper situational awareness
    let log_harvest = discover_recent_llm_json_logs();

    // Combine passed input string with the latest user intent from JSON logs when input is brief
    let combined_context = if user_situation.is_empty() {
        log_harvest
            .latest_user_intents
            .first()
            .cloned()
            .unwrap_or_default()
    } else if let Some(latest_log_intent) = log_harvest.latest_user_intents.first() {
        format!("{} | {}", user_situation, latest_log_intent)
    } else {
        user_situation.to_string()
    };

    // 2. Select projects based on context (or fallback to ~/Documents, ~/Desktop, ~/Downloads)
    let project_report = select_projects_for_context(user_situation, max_projects);

    let default_cwd = fallback_cwd
        .or_else(|| {
            std::env::current_dir()
                .ok()
                .map(|p| p.display().to_string())
        })
        .unwrap_or_else(|| ".".to_string());

    let active_root = if project_report.selection_mode == "context_matched_projects" {
        project_report
            .projects
            .first()
            .map(|p| p.path.clone())
            .unwrap_or(default_cwd)
    } else {
        default_cwd
    };

    // 3. Build the Mechanic's Tool Tray + infer session intent
    let (inferred_intent, recommended_tools) =
        build_mechanic_tool_tray(&combined_context, &active_root, &log_harvest.touched_files);

    // 4. Check audio playback (and optionally play music if nothing is already playing)
    let music_receipt = if play_music {
        play_music_if_silent(music_track)
    } else {
        let status = is_audio_currently_playing();
        MusicHelperReceipt {
            action: if status.is_playing {
                "already_playing_untouched"
            } else {
                "silent_ready_on_request"
            },
            is_playing: status.is_playing,
            active_source: status.active_source.clone(),
            track_path: None,
            message: if status.is_playing {
                format!(
                    "🎵 Audio is currently playing ({}) — keeping it untouched.",
                    status.active_source.as_deref().unwrap_or("active stream")
                )
            } else {
                "🔇 Nothing is playing right now. Pass `play_music: true` to `helper` anytime to put on ambient workshop music!".to_string()
            },
        }
    };

    // 5. Update and persist internal session parameters (`~/.st/helper_session.json`)
    let mut session = HelperSessionState::load_or_default();
    session.updated_at = Utc::now().to_rfc3339();
    session.helper_persona = requested_persona.to_string();
    session.current_situation = user_situation.chars().take(512).collect();
    session.inferred_intent = inferred_intent.to_string();
    session.active_project_root = active_root.clone();
    session.recommended_compression = "quantum".to_string();
    session.token_budget = params
        .get("token_budget")
        .and_then(|v| v.as_u64())
        .map(|n| n as usize)
        .unwrap_or(4000);
    session.focus_keywords = extract_salient_context_tokens(&combined_context);
    session.music_playing = music_receipt.is_playing;
    session.active_audio_source = music_receipt.active_source.clone();
    session.calls_in_session = session.calls_in_session.saturating_add(1);
    session.save();

    let cheet_joke = pick_cheet_dad_joke(&combined_context, inferred_intent);
    let dc_pick = crate::g8t_integration::route_dealers_choice(&combined_context, &[]);
    let interaction_guidance = crate::service_debug::shape_interaction_guidance(&combined_context);
    let stream_notes = crate::service_debug::list_stream_comments(6);
    let capsela_improvements = crate::flow_broadcaster::check_capsela_improvements();

    let payload = json!({
        "ok": true,
        "helper": "Smart Tree Mechanic's Helper (Local SPQT Base-243 Ternary Aye 🛠️ + The Cheet 😺 Dad-Jokes Helper)",
        "cheet_dad_joke": cheet_joke,
        "interaction_guidance": interaction_guidance,
        "stream_partnership_notes": stream_notes,
        "capsela_improvements": capsela_improvements,
        "session_parameters": session,
        "dealers_choice_router": {
            "problem_domain": dc_pick.classified_domain,
            "domain_description": dc_pick.domain_description,
            "selected_model": dc_pick.selected_model,
            "selected_provider": dc_pick.selected_provider,
            "why": dc_pick.selection_reason,
            "top_domain_standings": dc_pick.domain_leaderboard
        },
        "projects_for_llm": project_report,
        "mechanic_tool_tray": recommended_tools,
        "music_status": music_receipt,
        "llm_json_log_telemetry": log_harvest,
        "g8t_i1_cost_saver_proxy": {
            "local_proxy_command": "st --proxy --proxy-port 28428",
            "local_endpoints": [
                "POST http://127.0.0.1:28428/v1/chat/completions (supports model: 'dealers_choice' / 'g8t/dealers-choice')",
                "POST http://127.0.0.1:28428/v1/helper",
                "GET  http://127.0.0.1:28428/v1/dealers-choice",
                "POST http://127.0.0.1:28428/v1/capsela/asm-exec",
                "GET  http://127.0.0.1:28428/flow (Visual Studio Diff Engine & Brett Favre Play-by-Play)"
            ],
            "sovereign_cloud_tier": "https://i1.is / https://g8t.is — $5.00/mo Sovereign Helper & Context-Compression Proxy (saves 80-95% of tokens so builders never need a $200/month subscription!)",
            "unlimited_dealers_choice_tier": "https://g8t.is — $10.00/mo Unlimited g8t Session using Dealer's Choice on Best Model (automatically routes each problem to the #1 empirically proven model over time + Complete Capsela Freestanding ASM Executable Studio!)",
            "addon_unlimited_agents": "+$5.00/mo per additional Unlimited Agent (up to 10 add-on agents -> 11 total unlimited agents at $60/mo max!)",
            "annual_two_months_free_deal": "Buy the entire year and the last 2 months are FREE (10 months billed for 12 months, exact 1/6 savings) no matter what level or how many add-on agents you bought at!",
            "pricing_ladder": dc_pick.pricing_ladder
        }
    });

    Ok(json!({
        "content": [{
            "type": "text",
            "text": serde_json::to_string_pretty(&payload)?
        }]
    }))
}

/// JSON Schema definition for the `helper` MCP tool.
pub fn helper_tool_definition() -> Value {
    json!({
        "name": "helper",
        "description": "🛠️😺 MECHANIC'S HELPER (Local SPQT Base-243 Ternary Aye & The Cheet Dad-Jokes Helper) — Call `helper` with `whats_the_situation` describing what you know about the current conversation! Like a master mechanic's shop helper, it shapes per-interaction partnership guidance (Chris hates wasteful XML, purple is never good, no floats, no unsolicited boilerplate unless it's a special partnership/g8t pattern), surfaces gentle user stream notes ('A user mentioned this before in our partnership...'), checks Capsela g8t speed improvements, updates internal session parameters, harvests recent LLM JSON/JSONL logs, returns context-matched projects (or only ~/Documents, ~/Desktop, and ~/Downloads when no specific project context is sent), hands you the exact compressed context & next MCP tools to save 80-95% of cloud LLM tokens, and can start background music if nothing is already playing!",
        "inputSchema": {
            "type": "object",
            "properties": {
                "whats_the_situation": {
                    "type": "string",
                    "description": "What's the situation? Pass everything the AI Agent knows about the current conversation/task right now so Smart Tree can shape guidance, partnership rules, projects, and tools for this interaction."
                },
                "input": {
                    "type": "string",
                    "description": "Alias for 'whats_the_situation' — everything the LLM knows about the current conversation/task right now"
                },
                "context": {
                    "type": "string",
                    "description": "Alias for 'whats_the_situation' — current conversation context string"
                },
                "persona": {
                    "type": "string",
                    "enum": ["aye+cheet", "aye", "cheet"],
                    "description": "Helper persona: 'aye' (precision ternary helper), 'cheet' (The Cheet with dad jokes), or 'aye+cheet' (default tandem)"
                },
                "play_music": {
                    "type": "boolean",
                    "description": "If true, checks if any audio is playing and starts ambient/focus music ONLY if nothing is currently playing",
                    "default": false
                },
                "music_track": {
                    "type": "string",
                    "description": "Optional path to a music file (.wav/.mp3/.flac) to play if nothing is currently playing"
                },
                "token_budget": {
                    "type": "integer",
                    "description": "Session token budget for compressed context dumps (default: 4000)",
                    "default": 4000
                },
                "max_projects": {
                    "type": "integer",
                    "description": "Maximum number of context-matched projects (or Documents/Desktop/Downloads items) to return (default: 6)",
                    "default": 6
                }
            },
            "required": []
        }
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_select_projects_falls_back_to_documents_desktop_downloads_when_no_context() {
        let report = select_projects_for_context("", 6);
        assert_eq!(report.selection_mode, "documents_desktop_downloads_default");
        for proj in &report.projects {
            assert!(
                matches!(
                    proj.source_bucket.as_str(),
                    "Documents" | "Desktop" | "Downloads"
                ),
                "unexpected bucket in fallback mode: {}",
                proj.source_bucket
            );
        }

        // Generic conversational prompt with no project names should also use the Documents/Desktop/Downloads fallback
        let generic_report =
            select_projects_for_context("hello how are you doing today my friend", 6);
        assert_eq!(
            generic_report.selection_mode,
            "documents_desktop_downloads_default"
        );
    }

    #[test]
    fn test_sovereign_ambient_wav_synthesis_and_cheet_dad_joke() {
        let wav_path = ensure_sovereign_ambient_wav().expect("should synthesize integer WAV");
        assert!(wav_path.is_file());
        let meta = fs::metadata(&wav_path).unwrap();
        // 6 seconds @ 16kHz 16-bit mono = 192,000 + 44 bytes = 192,044 bytes
        assert_eq!(meta.len(), 192_044);

        let joke = pick_cheet_dad_joke(
            "working on smart-tree rust simd",
            "performance_optimization",
        );
        assert!(!joke.is_empty());
    }

    #[tokio::test]
    async fn test_handle_helper_end_to_end() {
        let out = handle_helper_with_cwd(
            Some(json!({
                "whats_the_situation": "We are working on smart-tree MCP tools, dashboard UI colors, XML removal, and Grandma Jody persona memories",
                "persona": "aye+cheet",
                "play_music": false,
                "token_budget": 3500
            })),
            Some("/home/hue/source/8b-is/smart-tree".to_string()),
        )
        .await
        .expect("helper should succeed");

        let text = out["content"][0]["text"]
            .as_str()
            .expect("should return MCP text envelope");
        let parsed: Value = serde_json::from_str(text).expect("valid JSON payload");
        assert_eq!(parsed["ok"], true);
        assert!(parsed["cheet_dad_joke"].as_str().is_some());
        assert_eq!(parsed["session_parameters"]["token_budget"], 3500);
        assert!(parsed["mechanic_tool_tray"].as_array().is_some());
        assert!(parsed["projects_for_llm"]["projects"].as_array().is_some());
        assert!(parsed["interaction_guidance"].as_array().is_some());
        assert!(parsed["capsela_improvements"].as_array().is_some());
        let guidance_joined = serde_json::to_string(&parsed["interaction_guidance"]).unwrap();
        assert!(guidance_joined.contains("XML is super wasteful"));
        assert!(guidance_joined.contains("purple is never good"));
        assert!(parsed["g8t_i1_cost_saver_proxy"]["sovereign_cloud_tier"]
            .as_str()
            .unwrap_or("")
            .contains("$200/month"));
    }
}
