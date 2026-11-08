// Smart Project Detector - Context-aware project name & folder detection
// "Bob is a name, but 'Project Bob' is a project! And office/atlanta is office_atlanta!" - Aye

use regex::Regex;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::RwLock;

/// Marker files that tell Aye NOT to look in or remember a folder (or any of its subfolders).
pub const DO_NOT_LOOK_MARKER_FILES: &[&str] = &[
    ".noaye",
    ".ayeignore",
    ".nom8",
    ".nost",
    ".st_no_look",
    ".dontlook",
];

/// Internal code/build/cache directory names that should not be treated as standalone loose projects
/// when nested inside an existing repository.
const STRUCTURAL_SUBDIR_NAMES: &[&str] = &[
    "src",
    "lib",
    "bin",
    "include",
    "pkg",
    "cmd",
    "internal",
    "tests",
    "test",
    "benches",
    "bench",
    "examples",
    "example",
    "target",
    "dist",
    "build",
    "out",
    "node_modules",
    "vendor",
    "scripts",
    "assets",
    "static",
    "public",
    "migrations",
    "fixtures",
    "snapshots",
    "ci",
    "coverage",
    "tmp",
    "temp",
    "logs",
    "log",
];

/// Organizational / semantic parent directory nouns where `parent/child` forms a
/// compound project identity like `office/atlanta` -> `office_atlanta`.
const ORG_HUB_PARENT_NOUNS: &[&str] = &[
    "office",
    "offices",
    "branch",
    "branches",
    "location",
    "locations",
    "site",
    "sites",
    "city",
    "cities",
    "region",
    "regions",
    "client",
    "clients",
    "customer",
    "customers",
    "account",
    "accounts",
    "team",
    "teams",
    "department",
    "departments",
    "dept",
    "division",
    "divisions",
    "studio",
    "studios",
    "lab",
    "labs",
    "campus",
    "hq",
    "store",
    "stores",
    "clinic",
    "clinics",
    "plant",
    "plants",
    "warehouse",
    "property",
    "properties",
    "vault",
    "vaults",
    "notebook",
    "notebooks",
    "course",
    "courses",
    "class",
    "classes",
    "subject",
    "subjects",
    "trip",
    "trips",
    "event",
    "events",
    "case",
    "cases",
    "matter",
    "matters",
    "campaign",
    "campaigns",
    "partner",
    "partners",
    "vendor",
    "vendors",
];

/// Generic filesystem container directory names that do not form semantic `parent_child` project slugs.
const GENERIC_CONTAINER_NAMES: &[&str] = &[
    "",
    "root",
    "home",
    "users",
    "user",
    "data",
    "mnt",
    "media",
    "volumes",
    "var",
    "tmp",
    "opt",
    "usr",
    "etc",
    "srv",
    "source",
    "sources",
    "repos",
    "repositories",
    "code",
    "dev",
    "devel",
    "development",
    "git",
    "github",
    "gitlab",
    "codeberg",
    "workspace",
    "workspaces",
    "documents",
    "desktop",
    "downloads",
];

#[derive(Default)]
struct DoNotLookCache {
    loaded: bool,
    auto_remember_projects: bool,
    paths: Vec<String>,
}

static DO_NOT_LOOK_CACHE: once_cell::sync::Lazy<RwLock<DoNotLookCache>> =
    once_cell::sync::Lazy::new(|| RwLock::new(DoNotLookCache::default()));

/// Invalidate the in-memory `do_not_look_paths` cache after updating `~/.st/config.toml`.
pub fn invalidate_do_not_look_cache() {
    if let Ok(mut guard) = DO_NOT_LOOK_CACHE.write() {
        guard.loaded = false;
    }
}

fn get_cached_do_not_look_config() -> (bool, Vec<String>) {
    if let Ok(guard) = DO_NOT_LOOK_CACHE.read() {
        if guard.loaded {
            return (guard.auto_remember_projects, guard.paths.clone());
        }
    }
    let cfg = crate::config::StConfig::load().unwrap_or_default();
    let auto_remember = cfg.safety.auto_remember_projects;
    let paths = cfg.safety.do_not_look_paths;
    if let Ok(mut guard) = DO_NOT_LOOK_CACHE.write() {
        guard.loaded = true;
        guard.auto_remember_projects = auto_remember;
        guard.paths = paths.clone();
    }
    (auto_remember, paths)
}

/// Returns `true` if `path` is an ephemeral/system/temp path (`/tmp`, `/proc`, `.tmp*`)
/// that should not be written into the user's global `~/.st/memories.m8` during background scans.
pub fn is_ephemeral_or_system_path(path: &Path) -> bool {
    let s = path.to_string_lossy();
    if s == "/"
        || s.starts_with("/tmp")
        || s.starts_with("/var/tmp")
        || s.starts_with("/proc")
        || s.starts_with("/sys")
        || s.starts_with("/dev")
        || s.starts_with("/run")
        || s.contains("/.tmp")
    {
        return true;
    }
    path.components().any(|c| {
        let name = c.as_os_str().to_string_lossy();
        name.starts_with(".tmp")
    })
}

/// Check if `path` or any of its ancestor directories contains a do-not-look marker file
/// (`.noaye`, `.ayeignore`, `.nom8`, `.nost`, `.st_no_look`, `.dontlook`).
pub fn has_do_not_look_marker(path: &Path) -> bool {
    let mut cur = Some(path);
    let mut depth = 0usize;
    while let Some(dir) = cur {
        if depth > 24 {
            break;
        }
        for marker in DO_NOT_LOOK_MARKER_FILES {
            if dir.join(marker).exists() {
                return true;
            }
        }
        // Also check if `.stignore` in `dir` has a blanket `*` ignore or names our target folder
        let stignore = dir.join(".stignore");
        if stignore.is_file() {
            if let Ok(content) = fs::read_to_string(&stignore) {
                for line in content.lines() {
                    let t = line.trim();
                    if t == "*" || t == "**" || t == "/*" {
                        return true;
                    }
                    if let Some(rel) = path.strip_prefix(dir).ok().and_then(|r| r.to_str()) {
                        if !rel.is_empty() && t.trim_matches('/') == rel.trim_matches('/') {
                            return true;
                        }
                    }
                }
            }
        }
        cur = dir.parent();
        depth += 1;
    }
    false
}

/// Returns `true` if Aye is allowed to look in and remember `path`.
/// Respects:
/// 1. Marker files (`.noaye`, `.ayeignore`, `.nom8`, `.nost`, `.st_no_look`, `.dontlook`, `.stignore`)
/// 2. Environment variables (`AYE_DO_NOT_LOOK`, `ST_NO_LOOK`)
/// 3. `~/.st/config.toml` -> `safety.auto_remember_projects` and `safety.do_not_look_paths`
/// 4. `AiGuardian` untrusted repo blocklist & build/cache artifact directories
pub fn is_aye_allowed_to_look(path: &Path) -> bool {
    let raw_str = path.to_string_lossy();
    if raw_str == "/"
        || raw_str.starts_with("/proc")
        || raw_str.starts_with("/sys")
        || raw_str.starts_with("/dev")
        || raw_str.starts_with("/run")
    {
        return false;
    }

    if crate::ai_guardian::AiGuardian::is_untrusted_repo_or_source(&raw_str) {
        return false;
    }

    // Never treat internal VCS or heavy build/dependency caches as lookable project roots
    for comp in path.components() {
        let name = comp.as_os_str().to_string_lossy();
        if matches!(
            name.as_ref(),
            ".git"
                | ".hg"
                | ".svn"
                | "node_modules"
                | "target"
                | "__pycache__"
                | ".cache"
                | ".venv"
                | "venv"
                | ".next"
                | ".nuxt"
                | ".pytest_cache"
                | ".mypy_cache"
        ) {
            return false;
        }
    }

    // Check marker files in `path` and ancestors
    if has_do_not_look_marker(path) {
        return false;
    }

    let canon = path.canonicalize().unwrap_or_else(|_| path.to_path_buf());
    let canon_str = canon.to_string_lossy();

    // Check environment variables (`AYE_DO_NOT_LOOK` / `ST_NO_LOOK`)
    for env_key in ["AYE_DO_NOT_LOOK", "ST_NO_LOOK"] {
        if let Ok(val) = std::env::var(env_key) {
            let trimmed = val.trim();
            if matches!(
                trimmed.to_ascii_lowercase().as_str(),
                "1" | "true" | "yes" | "all" | "*"
            ) {
                return false;
            }
            for entry in trimmed.split([':', ',']) {
                let e = entry.trim().trim_end_matches('/');
                if !e.is_empty()
                    && (canon_str.as_ref() == e
                        || canon_str.starts_with(&format!("{e}/"))
                        || raw_str.as_ref() == e
                        || raw_str.starts_with(&format!("{e}/")))
                {
                    return false;
                }
            }
        }
    }

    // Check `~/.st/config.toml` safety settings
    let (auto_remember, do_not_look_paths) = get_cached_do_not_look_config();
    if !auto_remember {
        return false;
    }
    for blocked in &do_not_look_paths {
        let b = blocked.trim().trim_end_matches('/');
        if b.is_empty() {
            continue;
        }
        if canon_str.as_ref() == b
            || canon_str.starts_with(&format!("{b}/"))
            || raw_str.as_ref() == b
            || raw_str.starts_with(&format!("{b}/"))
        {
            return false;
        }
    }

    true
}

/// Pack 15 balanced trits (`{-1, 0, +1}`) into 3 SPQT Base-243 bytes (`3^5 = 243 <= 255`).
#[inline(always)]
pub fn spqt_pack_15trits(trits: &[i8; 15]) -> [u8; 3] {
    let mut out = [0u8; 3];
    for (chunk_idx, slot) in out.iter_mut().enumerate() {
        let base = chunk_idx * 5;
        let mut byte_val = 0u8;
        let mut radix = 1u8;
        for k in 0..5 {
            let t = trits[base + k].clamp(-1, 1);
            let digit = (t + 1) as u8;
            byte_val = byte_val.wrapping_add(digit.wrapping_mul(radix));
            radix = radix.wrapping_mul(3);
        }
        *slot = byte_val;
    }
    out
}

/// Unpack 3 SPQT Base-243 bytes (`0..243`) back into 15 balanced trits (`{-1, 0, +1}`).
#[inline(always)]
pub fn spqt_unpack_15trits(packed: &[u8; 3]) -> [i8; 15] {
    let mut out = [0i8; 15];
    for (chunk_idx, &b) in packed.iter().enumerate() {
        let mut v = b.min(242);
        let base = chunk_idx * 5;
        for k in 0..5 {
            out[base + k] = (v % 3) as i8 - 1;
            v /= 3;
        }
    }
    out
}

/// Project archetype classified by Smart Tree's built-in SPQT Base-243 ternary model.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TernaryProjectArchetype {
    GitRepository,
    CodeWorkspace,
    ObsidianVault,
    OfficeLocationHub,
    KnowledgeArchive,
    LooseProjectFolder,
    NotAProject,
}

impl TernaryProjectArchetype {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::GitRepository => "git_repository",
            Self::CodeWorkspace => "code_workspace",
            Self::ObsidianVault => "obsidian_vault",
            Self::OfficeLocationHub => "office_location_hub",
            Self::KnowledgeArchive => "knowledge_archive",
            Self::LooseProjectFolder => "loose_project_folder",
            Self::NotAProject => "not_a_project",
        }
    }
}

/// Decision returned by the built-in local SPQT Base-243 ternary project classifier
/// (`burn/mem8-ternary-micro` · zero floats · `felix_ternary_gemm_i32`).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TernaryProjectDecision {
    pub is_project: bool,
    pub project_name: String,
    pub compound_slug: String,
    pub archetype: TernaryProjectArchetype,
    pub score_q8: i32,
    pub threshold_q8: i32,
    pub model_id: &'static str,
    pub spqt_weights_hex: String,
    pub spqt_sig_hex: String,
    pub keywords: Vec<String>,
    pub summary: String,
}

fn is_structural_subdir_name(name: &str) -> bool {
    STRUCTURAL_SUBDIR_NAMES
        .iter()
        .any(|&s| name.eq_ignore_ascii_case(s))
}

fn sanitize_slug_piece(raw: &str) -> String {
    let mut out = String::with_capacity(raw.len());
    let mut prev_underscore = false;
    for ch in raw.chars() {
        if ch.is_ascii_alphanumeric() {
            out.push(ch.to_ascii_lowercase());
            prev_underscore = false;
        } else if (ch == '-' || ch == '_' || ch == ' ' || ch == '.')
            && !out.is_empty()
            && !prev_underscore
        {
            out.push('_');
            prev_underscore = true;
        }
    }
    out.trim_matches('_').to_string()
}

/// Inspect a directory using the built-in SPQT Base-243 ternary model (`burn/mem8-ternary-micro`)
/// with zero floating-point operations (`felix_ternary_gemm_i32`).
pub fn inspect_directory_ternary(dir: &Path) -> TernaryProjectDecision {
    const THRESHOLD_Q8: i32 = 240;
    const MODEL_ID: &str = "burn/mem8-ternary-micro";

    if !dir.is_dir() || !is_aye_allowed_to_look(dir) {
        return TernaryProjectDecision {
            is_project: false,
            project_name: String::new(),
            compound_slug: String::new(),
            archetype: TernaryProjectArchetype::NotAProject,
            score_q8: 0,
            threshold_q8: THRESHOLD_Q8,
            model_id: MODEL_ID,
            spqt_weights_hex: "0x000000".to_string(),
            spqt_sig_hex: "0x00000000".to_string(),
            keywords: Vec::new(),
            summary: String::new(),
        };
    }

    let dir_name = dir
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("")
        .trim();
    if dir_name.is_empty() || dir_name.starts_with('.') {
        return TernaryProjectDecision {
            is_project: false,
            project_name: dir_name.to_string(),
            compound_slug: sanitize_slug_piece(dir_name),
            archetype: TernaryProjectArchetype::NotAProject,
            score_q8: 0,
            threshold_q8: THRESHOLD_Q8,
            model_id: MODEL_ID,
            spqt_weights_hex: "0x000000".to_string(),
            spqt_sig_hex: "0x00000000".to_string(),
            keywords: Vec::new(),
            summary: String::new(),
        };
    }

    let parent_name = dir
        .parent()
        .and_then(|p| p.file_name())
        .and_then(|n| n.to_str())
        .unwrap_or("")
        .trim();

    let dir_slug = sanitize_slug_piece(dir_name);
    let parent_slug = sanitize_slug_piece(parent_name);

    let is_org_hub_parent = !parent_slug.is_empty()
        && ORG_HUB_PARENT_NOUNS
            .iter()
            .any(|&noun| parent_slug == noun || parent_slug.ends_with(&format!("_{noun}")));

    let is_meaningful_parent = !parent_slug.is_empty()
        && !parent_name.starts_with('.')
        && !GENERIC_CONTAINER_NAMES.contains(&parent_slug.as_str())
        && !STRUCTURAL_SUBDIR_NAMES.contains(&parent_slug.as_str());

    let compound_slug = if (is_org_hub_parent || is_meaningful_parent) && !dir_slug.is_empty() {
        format!("{}_{}", parent_slug, dir_slug)
    } else {
        dir_slug.clone()
    };

    // Inspect directory markers & immediate entries (bounded O(1) scan of up to 96 entries)
    let has_git = dir.join(".git").exists() || dir.join(".gitmodules").is_file();
    let has_git_head = dir.join(".git").join("HEAD").is_file()
        || dir.join(".git").is_file()
        || dir.join(".git").join("config").is_file();

    let has_obsidian_vault = dir.join(".obsidian").is_dir()
        || dir.join(".foam").is_dir()
        || dir.join(".logseq").is_dir()
        || dir.join(".dendron.ws").is_file();

    let has_m8_consciousness = dir.join(".m8").is_file()
        || dir.join(".aye_consciousness.m8").is_file()
        || dir.join(".claude_consciousness.m8").is_file()
        || dir.join("TODO.m8").is_file();

    let has_obsidian_or_m8 =
        has_obsidian_vault || (has_m8_consciousness && !is_structural_subdir_name(dir_name));

    let manifest_files = [
        "Cargo.toml",
        "package.json",
        "pyproject.toml",
        "go.mod",
        "pom.xml",
        "build.gradle",
        "build.gradle.kts",
        "Gemfile",
        "requirements.txt",
        "setup.py",
        "CMakeLists.txt",
        "Makefile",
        "Dockerfile",
        "docker-compose.yml",
        "flake.nix",
        "mix.exs",
        "composer.json",
    ];
    let mut detected_manifests: Vec<&str> = manifest_files
        .iter()
        .copied()
        .filter(|m| dir.join(m).is_file())
        .collect();

    let doc_markers = [
        "README.md",
        "CLAUDE.md",
        "AGENTS.md",
        "GEMINI.md",
        "FAMILY_CONTEXT.md",
        "ARCHITECTURE.md",
        "INDEX.md",
        "Home.md",
        "README.org",
        "README.txt",
        "README",
    ];
    let has_doc_marker = doc_markers.iter().any(|d| dir.join(d).is_file());

    let mut md_note_count = 0i16;
    let mut code_file_count = 0i16;
    let mut data_config_count = 0i16;
    let mut subdir_count = 0i16;
    let mut visible_entry_count = 0i16;
    let mut sample_names: Vec<String> = Vec::new();

    if let Ok(entries) = fs::read_dir(dir) {
        for entry in entries.flatten().take(96) {
            let fname = entry.file_name().to_string_lossy().to_string();
            if fname.starts_with('.') {
                continue;
            }
            visible_entry_count = visible_entry_count.saturating_add(1);
            if sample_names.len() < 10 {
                sample_names.push(fname.clone());
            }
            let Ok(ft) = entry.file_type() else {
                continue;
            };
            if ft.is_dir() {
                if !STRUCTURAL_SUBDIR_NAMES.contains(&fname.to_ascii_lowercase().as_str()) {
                    subdir_count = subdir_count.saturating_add(1);
                } else if fname.eq_ignore_ascii_case("src") {
                    code_file_count = code_file_count.saturating_add(2);
                }
            } else if ft.is_file() {
                let ext = Path::new(&fname)
                    .extension()
                    .and_then(|e| e.to_str())
                    .unwrap_or("")
                    .to_ascii_lowercase();
                match ext.as_str() {
                    "md" | "markdown" | "org" | "rst" | "txt" | "adoc" | "pdf" | "docx"
                    | "xlsx" | "pptx" | "canvas" | "m8" => {
                        md_note_count = md_note_count.saturating_add(1);
                    }
                    "rs" | "py" | "ts" | "tsx" | "js" | "jsx" | "go" | "c" | "cpp" | "cc" | "h"
                    | "hpp" | "java" | "kt" | "swift" | "rb" | "sh" | "zig" | "lua" | "sql"
                    | "cs" | "fs" => {
                        code_file_count = code_file_count.saturating_add(1);
                    }
                    "csproj" | "sln" => {
                        code_file_count = code_file_count.saturating_add(2);
                        if !detected_manifests.contains(&"*.csproj") {
                            detected_manifests.push("*.csproj");
                        }
                    }
                    "toml" | "json" | "yaml" | "yml" | "ini" | "csv" | "tsv" | "jsonl"
                    | "sqlite" | "db" | "parquet" => {
                        data_config_count = data_config_count.saturating_add(1);
                    }
                    _ => {}
                }
            }
        }
    }
    sample_names.sort();

    // Check if an ancestor (1 or 2 levels up) is already a Git repo or manifest project
    let ancestor_is_repo = [dir.parent(), dir.parent().and_then(|p| p.parent())]
        .into_iter()
        .flatten()
        .any(|anc| {
            anc.join(".git").exists()
                || anc.join("Cargo.toml").is_file()
                || anc.join("package.json").is_file()
                || anc.join("pyproject.toml").is_file()
                || anc.join("go.mod").is_file()
        });

    let is_structural_subdir = is_structural_subdir_name(dir_name);

    let has_own_strong_marker = has_git || !detected_manifests.is_empty() || has_obsidian_vault;

    // If a parent folder is just a pure organizational category container (like `office/` containing `atlanta/`),
    // penalize the bare category container so `office/atlanta` (`office_atlanta`) is chosen as the project!
    let is_bare_org_category_container = ORG_HUB_PARENT_NOUNS.iter().any(|&noun| dir_slug == noun)
        && !has_own_strong_marker
        && !has_doc_marker
        && md_note_count == 0
        && code_file_count == 0
        && data_config_count == 0;

    // Build the 15 integer Q8 feature activations (`0..=256`, zero floats)
    let mut acts = [0i16; 15];
    acts[0] = if has_git { 256 } else { 0 };
    acts[1] = if !detected_manifests.is_empty() {
        256
    } else {
        0
    };
    acts[2] = if has_obsidian_or_m8 { 256 } else { 0 };
    acts[3] = if has_doc_marker { 220 } else { 0 };
    acts[4] = if is_org_hub_parent && !is_structural_subdir {
        240
    } else {
        0
    };
    acts[5] = if is_meaningful_parent && !is_structural_subdir && visible_entry_count > 0 {
        140
    } else {
        0
    };
    acts[6] = match md_note_count {
        0 => 0,
        1 => 90,
        2 => 150,
        _ => 220,
    };
    acts[7] = match code_file_count {
        0 => 0,
        1 => 80,
        2..=4 => 140,
        _ => 200,
    };
    acts[8] = match data_config_count {
        0 => 0,
        1 => 80,
        _ => 150,
    };
    acts[9] = if (1..=16).contains(&subdir_count) {
        110
    } else {
        0
    };
    acts[10] = if is_structural_subdir && !has_own_strong_marker {
        256
    } else if is_bare_org_category_container {
        256
    } else {
        0
    };
    acts[11] = if ancestor_is_repo && !has_own_strong_marker && !is_org_hub_parent {
        240
    } else {
        0
    };
    acts[12] = if visible_entry_count == 0 && !has_own_strong_marker && !is_org_hub_parent {
        256
    } else {
        0
    };
    acts[13] = if has_git_head { 160 } else { 0 };
    acts[14] = if dir_slug.len() >= 2
        && dir_slug.chars().any(|c| c.is_ascii_alphabetic())
        && !GENERIC_CONTAINER_NAMES.contains(&dir_slug.as_str())
    {
        100
    } else {
        0
    };

    // 5 SPQT Base-243 Ternary Archetype Weight Rows (15 trits -> 3 Base-243 bytes each)
    let heads: [(TernaryProjectArchetype, [i8; 15]); 5] = [
        (
            TernaryProjectArchetype::GitRepository,
            [1, 1, 0, 1, 0, 0, 0, 1, 0, 0, -1, -1, -1, 1, 1],
        ),
        (
            TernaryProjectArchetype::CodeWorkspace,
            [1, 1, 0, 1, 0, 0, 0, 1, 1, 1, -1, -1, -1, 0, 1],
        ),
        (
            TernaryProjectArchetype::ObsidianVault,
            [0, 0, 1, 1, 0, 1, 1, 0, 0, 1, -1, -1, -1, 0, 1],
        ),
        (
            TernaryProjectArchetype::OfficeLocationHub,
            [0, 0, 1, 1, 1, 1, 1, 0, 1, 1, -1, -1, -1, 0, 1],
        ),
        (
            TernaryProjectArchetype::KnowledgeArchive,
            [0, 0, 1, 1, 1, 1, 1, 0, 1, 1, -1, -1, -1, 0, 1],
        ),
    ];

    let mut best_archetype = TernaryProjectArchetype::NotAProject;
    let mut best_score_q8 = i32::MIN;
    let mut best_packed = [0u8; 3];

    for (arch, raw_trits) in &heads {
        // Gate specialized heads so they only fire when their defining structural signal is present
        match arch {
            TernaryProjectArchetype::ObsidianVault if !has_obsidian_or_m8 => continue,
            TernaryProjectArchetype::OfficeLocationHub if !is_org_hub_parent => continue,
            TernaryProjectArchetype::GitRepository if !has_git => continue,
            TernaryProjectArchetype::CodeWorkspace if detected_manifests.is_empty() && !has_git => {
                continue
            }
            _ => {}
        }

        // Pack & unpack through SPQT Base-243 (3^5 = 243) and run hardware FELIX ternary GEMM
        let packed = spqt_pack_15trits(raw_trits);
        let unpacked = spqt_unpack_15trits(&packed);
        let mut score = crate::felix_asm::felix_ternary_gemm_i32(&acts, &unpacked);

        // Disambiguate specialized archetypes when their primary marker fires
        match arch {
            TernaryProjectArchetype::ObsidianVault if has_obsidian_or_m8 => score += 96,
            TernaryProjectArchetype::OfficeLocationHub if is_org_hub_parent => score += 160,
            TernaryProjectArchetype::CodeWorkspace if !detected_manifests.is_empty() => score += 64,
            TernaryProjectArchetype::GitRepository if has_git => score += 48,
            _ => {}
        }

        if score > best_score_q8 {
            best_score_q8 = score;
            best_archetype = *arch;
            best_packed = packed;
        }
    }

    // Inside an existing Git repo or manifest workspace, only treat subdirectories as separate
    // projects if they have their own .git, their own package manifest, their own .obsidian vault,
    // or an explicit organizational hub parent (like `office/atlanta`).
    let blocked_by_ancestor_repo = ancestor_is_repo && !has_own_strong_marker && !is_org_hub_parent;

    let is_project = best_score_q8 >= THRESHOLD_Q8 && !blocked_by_ancestor_repo;
    if !is_project {
        best_archetype = TernaryProjectArchetype::NotAProject;
    } else if best_archetype == TernaryProjectArchetype::KnowledgeArchive
        && !has_obsidian_or_m8
        && !is_org_hub_parent
    {
        best_archetype = TernaryProjectArchetype::LooseProjectFolder;
    }

    // Choose canonical project name:
    // - For organizational/hierarchical folders like `office/atlanta`, use `office_atlanta`!
    // - For loose folders where `parent/child` carries the meaning (and no manifest overrides it),
    //   include `compound_slug` and use `office_atlanta` when `is_org_hub_parent` is true.
    let project_name = if is_org_hub_parent {
        compound_slug.clone()
    } else {
        dir_name.to_string()
    };

    let mut keywords = Vec::new();
    let push_kw = |kw_list: &mut Vec<String>, candidate: &str| {
        let clean = candidate.trim().to_ascii_lowercase();
        if clean.len() >= 2 && !kw_list.contains(&clean) {
            kw_list.push(clean);
        }
    };

    push_kw(&mut keywords, &project_name);
    push_kw(&mut keywords, &compound_slug);
    push_kw(&mut keywords, dir_name);
    push_kw(&mut keywords, &dir_slug);
    if is_org_hub_parent || is_meaningful_parent {
        push_kw(&mut keywords, parent_name);
        push_kw(&mut keywords, &parent_slug);
        push_kw(&mut keywords, &format!("{parent_slug}/{dir_slug}"));
    }
    push_kw(&mut keywords, best_archetype.as_str());
    if has_git {
        push_kw(&mut keywords, "git");
    }
    if dir.join(".obsidian").is_dir() {
        push_kw(&mut keywords, "obsidian");
        push_kw(&mut keywords, "vault");
    }
    for m in &detected_manifests {
        push_kw(&mut keywords, m);
    }

    let sig_input = format!(
        "{} {} {} {}",
        compound_slug,
        best_archetype.as_str(),
        detected_manifests.join(","),
        sample_names.join(",")
    );
    let (_, sig_packed) = crate::proxy::burn::compute_ternary_signature(&sig_input);
    let spqt_sig_hex = format!(
        "0x{:02x}{:02x}{:02x}{:02x}",
        sig_packed[0], sig_packed[1], sig_packed[2], sig_packed[3]
    );
    let spqt_weights_hex = format!(
        "0x{:02x}{:02x}{:02x}",
        best_packed[0], best_packed[1], best_packed[2]
    );

    let summary = format!(
        "{} [{} · ternary={} ({}/{}) · spqt={}] entries=[{}]",
        project_name,
        best_archetype.as_str(),
        MODEL_ID,
        best_score_q8,
        THRESHOLD_Q8,
        spqt_sig_hex,
        sample_names.join(", ")
    );

    TernaryProjectDecision {
        is_project,
        project_name,
        compound_slug,
        archetype: best_archetype,
        score_q8: best_score_q8,
        threshold_q8: THRESHOLD_Q8,
        model_id: MODEL_ID,
        spqt_weights_hex,
        spqt_sig_hex,
        keywords,
        summary,
    }
}

/// Evaluate whether `dir` should be loosely added to `"projects"` by Smart Tree's
/// built-in local SPQT Base-243 ternary model (`burn/mem8-ternary-micro`).
pub fn evaluate_directory_as_project_ternary(dir: &Path) -> Option<TernaryProjectDecision> {
    let decision = inspect_directory_ternary(dir);
    if decision.is_project {
        Some(decision)
    } else {
        None
    }
}

/// Walk `root` up to `max_depth` and return all directories that the built-in SPQT Base-243
/// ternary model classifies as projects, skipping any folder where Aye is told not to look.
pub fn discover_projects_ternary_in_tree(
    root: &Path,
    max_depth: usize,
) -> Vec<(PathBuf, TernaryProjectDecision)> {
    if !root.is_dir() || !is_aye_allowed_to_look(root) {
        return Vec::new();
    }

    let mut out = Vec::new();
    let walker = walkdir::WalkDir::new(root)
        .max_depth(max_depth)
        .follow_links(false)
        .into_iter()
        .filter_entry(|e| {
            if !e.file_type().is_dir() {
                return false;
            }
            if e.depth() > 0 {
                let name = e.file_name().to_string_lossy();
                if name.starts_with('.') {
                    return false;
                }
            }
            is_aye_allowed_to_look(e.path())
        });

    for entry in walker.flatten() {
        if !entry.file_type().is_dir() {
            continue;
        }
        if let Some(decision) = evaluate_directory_as_project_ternary(entry.path()) {
            out.push((entry.path().to_path_buf(), decision));
        }
    }

    out
}

/// Detect if content contains meaningful project references
pub fn contains_project_reference(content: &str, project_name: &str) -> bool {
    // Short names like "bob" need more context
    let needs_context = project_name.len() <= 3 || project_name.chars().all(char::is_alphanumeric);

    if needs_context {
        // For short/common names, require contextual markers
        contains_contextual_reference(content, project_name)
    } else {
        // For longer/unique names, simple contains is OK
        // But still check for word boundaries
        contains_word_boundary_match(content, project_name)
    }
}

/// Check for contextual references (for short/common project names)
fn contains_contextual_reference(content: &str, project_name: &str) -> bool {
    let lower_content = content.to_lowercase();
    let lower_project = project_name.to_lowercase();

    // Patterns that indicate this is a PROJECT reference, not just the word
    let context_patterns = [
        // Markdown headers
        format!("# {}", lower_project),
        format!("## {}", lower_project),
        format!("# {} project", lower_project),
        format!("# project {}", lower_project),
        // Code/config contexts
        format!("\"project\": \"{}\"", lower_project),
        format!("'project': '{}'", lower_project),
        format!("project_name: {}", lower_project),
        format!("name: \"{}\"", lower_project),
        // Documentation patterns
        format!("{} documentation", lower_project),
        format!("{} readme", lower_project),
        format!("{} notes", lower_project),
        format!("{} todo", lower_project),
        // Development contexts
        format!("{} repository", lower_project),
        format!("{} codebase", lower_project),
        format!("{} development", lower_project),
        format!("{} implementation", lower_project),
        // File paths (strong indicator)
        format!("/{}/", lower_project),
        format!("/{}/src", lower_project),
        format!("~/{}/", lower_project),
        format!("documents/{}", lower_project),
        format!("projects/{}", lower_project),
        // Git contexts
        format!("git clone {}", lower_project),
        format!("cd {}", lower_project),
        format!("working on {}", lower_project),
        // AI assistant contexts
        format!("help with {}", lower_project),
        format!("question about {}", lower_project),
        format!("{} issue", lower_project),
        format!("{} bug", lower_project),
        format!("{} feature", lower_project),
    ];

    // Check if any context pattern matches
    context_patterns
        .iter()
        .any(|pattern| lower_content.contains(pattern))
}

/// Check for word boundary matches (for longer project names)
fn contains_word_boundary_match(content: &str, project_name: &str) -> bool {
    // Build regex for word boundary matching
    let pattern = format!(r"\b{}\b", regex::escape(project_name));

    if let Ok(re) = Regex::new(&pattern) {
        re.is_match(content)
    } else {
        // Fallback to simple contains if regex fails
        content.contains(project_name)
    }
}

/// Extract project references from JSON content
pub fn extract_json_project_references(json: &Value, project_name: &str) -> Vec<String> {
    let mut references = Vec::new();

    // Check specific JSON fields that often contain project info
    let project_fields = [
        "project",
        "projectName",
        "project_name",
        "name",
        "title",
        "repository",
        "repo",
        "package",
        "module",
        "app",
        "application",
    ];

    for field in &project_fields {
        if let Some(value) = json.get(field) {
            if let Some(str_val) = value.as_str() {
                if str_val
                    .to_lowercase()
                    .contains(&project_name.to_lowercase())
                {
                    references.push(format!("{}: {}", field, str_val));
                }
            }
        }
    }

    // Check in messages/content fields for AI conversations
    let content_fields = ["message", "content", "text", "body", "prompt", "response"];

    for field in &content_fields {
        if let Some(value) = json.get(field) {
            if let Some(str_val) = value.as_str() {
                if contains_project_reference(str_val, project_name) {
                    // Extract a snippet around the reference
                    let snippet = extract_context_snippet(str_val, project_name, 100);
                    references.push(snippet);
                }
            }
        }
    }

    // Recursively check arrays and objects
    match json {
        Value::Array(arr) => {
            for item in arr {
                references.extend(extract_json_project_references(item, project_name));
            }
        }
        Value::Object(map) => {
            for (_key, value) in map {
                if value.is_object() || value.is_array() {
                    references.extend(extract_json_project_references(value, project_name));
                }
            }
        }
        _ => {}
    }

    references
}

/// Extract a context snippet around a project mention
fn extract_context_snippet(content: &str, project_name: &str, context_chars: usize) -> String {
    let lower_content = content.to_lowercase();
    let lower_project = project_name.to_lowercase();

    if let Some(pos) = lower_content.find(&lower_project) {
        let start = pos.saturating_sub(context_chars / 2);
        let end = (pos + lower_project.len() + context_chars / 2).min(content.len());

        let snippet = &content[start..end];

        // Add ellipsis if truncated
        let prefix = if start > 0 { "..." } else { "" };
        let suffix = if end < content.len() { "..." } else { "" };

        format!("{}{}{}", prefix, snippet, suffix)
    } else {
        content.chars().take(context_chars * 2).collect()
    }
}

/// Score the relevance of a project reference
pub fn score_reference_relevance(content: &str, project_name: &str) -> f64 {
    let mut score = 0.0;
    let lower_content = content.to_lowercase();
    let lower_project = project_name.to_lowercase();

    // Count exact matches
    let exact_matches = lower_content.matches(&lower_project).count();
    score += exact_matches as f64 * 1.0;

    // Bonus for being in a header
    if content.starts_with('#') && lower_content.contains(&lower_project) {
        score += 5.0;
    }

    // Bonus for being in quotes (likely a config value)
    if content.contains(&format!("\"{}\"", project_name))
        || content.contains(&format!("'{}'", project_name))
    {
        score += 3.0;
    }

    // Bonus for path-like references
    if content.contains(&format!("/{}/", project_name)) {
        score += 4.0;
    }

    // Bonus for development-related keywords nearby
    let dev_keywords = [
        "project",
        "develop",
        "implement",
        "feature",
        "bug",
        "issue",
        "repository",
        "code",
        "build",
        "test",
        "deploy",
    ];

    for keyword in &dev_keywords {
        if lower_content.contains(keyword) && lower_content.contains(&lower_project) {
            score += 0.5;
        }
    }

    // Normalize score to 0-100
    (score * 10.0).min(100.0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_short_name_detection() {
        // "bob" alone shouldn't match without context
        assert!(!contains_project_reference(
            "I talked to bob yesterday",
            "bob"
        ));

        // But with context it should
        assert!(contains_project_reference("# Bob Project Notes", "bob"));
        assert!(contains_project_reference(
            "Working on bob development",
            "bob"
        ));
        assert!(contains_project_reference("\"project\": \"bob\"", "bob"));
    }

    #[test]
    fn test_longer_name_detection() {
        // Longer names can match with word boundaries
        assert!(contains_project_reference(
            "The smart-tree project is great",
            "smart-tree"
        ));
        assert!(contains_project_reference(
            "smart-tree development",
            "smart-tree"
        ));

        // But not partial matches
        assert!(!contains_word_boundary_match("smart-trees", "smart-tree"));
    }

    #[test]
    fn test_json_extraction() {
        let json = serde_json::json!({
            "project": "bob",
            "message": "Working on bob feature implementation",
            "unrelated": "alice and bob went to the store"
        });

        let refs = extract_json_project_references(&json, "bob");
        assert_eq!(refs.len(), 2); // Should find project field and message field
    }

    #[test]
    fn test_ternary_project_detector_git_obsidian_and_office_atlanta() {
        let temp = tempfile::tempdir().unwrap();

        // 1. .git repository is always recognized as a project
        let git_repo = temp.path().join("apollo-guidance");
        fs::create_dir_all(git_repo.join(".git")).unwrap();
        fs::write(git_repo.join(".git/HEAD"), "ref: refs/heads/main\n").unwrap();
        let git_dec = evaluate_directory_as_project_ternary(&git_repo)
            .expect(".git folder should be detected as a project");
        assert!(git_dec.is_project);
        assert_eq!(git_dec.archetype, TernaryProjectArchetype::GitRepository);
        assert_eq!(git_dec.project_name, "apollo-guidance");

        // Structural code subfolder `src` inside `apollo-guidance` should NOT be a separate project
        let src_sub = git_repo.join("src");
        fs::create_dir_all(&src_sub).unwrap();
        fs::write(src_sub.join("main.rs"), "fn main() {}\n").unwrap();
        assert!(
            evaluate_directory_as_project_ternary(&src_sub).is_none(),
            "src/ inside a git repo should not be added as a separate project"
        );

        // 2. .obsidian vault is recognized as an ObsidianVault project
        let obsidian_dir = temp.path().join("second-brain");
        fs::create_dir_all(obsidian_dir.join(".obsidian")).unwrap();
        fs::write(
            obsidian_dir.join("Daily Note.md"),
            "# Daily Note\nIdeas for MEM8 wave interference.\n",
        )
        .unwrap();
        let obs_dec = evaluate_directory_as_project_ternary(&obsidian_dir)
            .expect(".obsidian folder should be detected as a project");
        assert_eq!(obs_dec.archetype, TernaryProjectArchetype::ObsidianVault);
        assert!(obs_dec.keywords.contains(&"obsidian".to_string()));
        assert!(obs_dec.keywords.contains(&"vault".to_string()));

        // 3. `office/atlanta` is loosely recognized by the built-in SPQT Base-243 ternary model
        //    with canonical compound name `office_atlanta` (while bare `office/` container is skipped)
        let office_dir = temp.path().join("office");
        let atlanta_dir = office_dir.join("atlanta");
        fs::create_dir_all(&atlanta_dir).unwrap();
        fs::write(
            atlanta_dir.join("lease_and_roster.md"),
            "# Atlanta Office\nPeachtree Street hub.\n",
        )
        .unwrap();
        assert!(
            evaluate_directory_as_project_ternary(&office_dir).is_none(),
            "bare category container `office/` should not outrank `office/atlanta`"
        );
        let atl_dec = evaluate_directory_as_project_ternary(&atlanta_dir)
            .expect("office/atlanta should be detected by the built-in ternary model");
        assert_eq!(
            atl_dec.archetype,
            TernaryProjectArchetype::OfficeLocationHub
        );
        assert_eq!(atl_dec.project_name, "office_atlanta");
        assert_eq!(atl_dec.compound_slug, "office_atlanta");
        assert!(atl_dec.keywords.contains(&"office_atlanta".to_string()));
        assert!(atl_dec.keywords.contains(&"office".to_string()));
        assert!(atl_dec.keywords.contains(&"atlanta".to_string()));

        // 4. Telling Aye not to look in a folder (.noaye marker) blocks detection
        let secret_dir = temp.path().join("private-vault");
        fs::create_dir_all(secret_dir.join(".git")).unwrap();
        fs::write(secret_dir.join(".noaye"), "do not look\n").unwrap();
        assert!(!is_aye_allowed_to_look(&secret_dir));
        assert!(evaluate_directory_as_project_ternary(&secret_dir).is_none());
    }
}
