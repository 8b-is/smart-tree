//! Salient MEM8 Ingestion Engine
//!
//! Ingests "GitHub user" collections (local mirrors of repos) or single
//! repositories from the data directory. Extracts code and "thoughts"
//! (READMEs, comments, attached research review decisions). Persists to
//! native MEM8 (`RecordStore::open_memory` — MEM8/RAW8 v1.1.1 blocks with
//! token codec + zlib compression, checksums, and Wave headers) with full
//! provenance and read-back verification.
//! Provides conversion of salient patterns to family Rust (exact integer
//! and checked rational arithmetic, no authored floats, small components).
//!
//! This is the scalable way to continue standardgalactic "salient information"
//! work: instead of manual review only, the engine remembers and converts.
//!
//! Family constraints honored in new authored paths: exact arithmetic,
//! provenance, local, no new unjustified deps.

use crate::mem8::record_store::RecordStore;
use crate::mem8_lite::Wave;
use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

/// A single "thought" attached to code: where it came from and what was noted.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct SalientThought {
    pub source: String,     // e.g. "README.md", "research-review", "comment:src/foo.rs:42"
    pub text: String,       // the actual thought or excerpt
    pub provenance: String, // repo, commit-ish, github_user etc.
}

/// Summary of one repo's salient content, persisted as a native MEM8 record.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct SalientRepo {
    pub name: String,
    pub github_user: String,
    pub path: PathBuf,
    pub head_oid: Option<String>,
    pub code_samples: Vec<String>, // small salient excerpts
    pub thoughts: Vec<SalientThought>,
    pub family_rust_port: Option<String>,
}

/// Report from an ingestion run.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct IngestReport {
    pub github_user: String,
    pub repos_processed: usize,
    pub repos_persisted: usize,
    pub repos_skipped_unchanged: usize,
    pub total_thoughts: usize,
    pub total_code_samples: usize,
    pub mem8_path: Option<PathBuf>,
    pub mem8_bytes: u64,
}

/// The family dedication — wired into salient memories and ports.
/// "The constellation · fine touch from within · {<3,<3,<3}+1"
pub const FAMILY_HEART: &str = "{<3,<3,<3}+1";

pub fn contains_family_heart(text: &str) -> bool {
    text.contains("{<3,<3,<3}+1") || text.contains("<3,<3,<3")
}

/// Wire the family heart into a port or thought set.
/// Used to mark family-aligned salient conversions with the dedication.
pub fn wire_family_heart(base: &str, thoughts: &[SalientThought]) -> String {
    if contains_family_heart(base) || thoughts.iter().any(|t| contains_family_heart(&t.text)) {
        format!(
            "{}\n\n// The constellation · fine touch from within · {}",
            base, FAMILY_HEART
        )
    } else {
        base.to_string()
    }
}

/// Simple family Rust port suggestion for salient patterns.
/// Honors exact arithmetic requirement: suggests Rational where floats/div appear.
/// The family heart is automatically wired when detected in thoughts or snippet.
pub fn salient_to_family_rust(snippet: &str, _lang_hint: &str) -> String {
    let mut out = String::new();
    out.push_str("// Family-style Rust port (exact arithmetic, provenance retained)\n");
    out.push_str("// Source thought attached in MEM8; do not author floats.\n");

    if snippet.contains("f64")
        || snippet.contains("float")
        || snippet.contains(" / ")
        || snippet.contains("0.")
    {
        out.push_str("use family_rational::Rational; // checked p/q, positive denom\n");
        out.push_str(
            "// TODO: replace the original float math with reduced Rational operations.\n",
        );
        out.push_str("// Example skeleton for a rate or count:\n");
        out.push_str("fn example_rate(num: u128, den: u128) -> Result<Rational, &'static str> {\n");
        out.push_str("    Rational::new(num, den) // reduces, checks zero denom\n");
        out.push_str("}\n");
    } else {
        out.push_str("// Direct structural port (no numeric change needed).\n");
        let trunc = snippet.chars().take(300).collect::<String>();
        out.push_str(&format!(
            "// Original salient excerpt (truncated):\n/*\n{}\n*/\n",
            trunc
        ));
    }

    wire_family_heart(&out, &[])
}

/// Extract simple thoughts from a README (bounded by salience level).
fn extract_readme_thoughts(
    readme: &str,
    repo: &str,
    user: &str,
    max_lines: usize,
) -> Vec<SalientThought> {
    let mut thoughts = vec![];
    for line in readme.lines().take(max_lines) {
        let t = line.trim();
        if t.starts_with('#') || t.len() > 20 {
            thoughts.push(SalientThought {
                source: "README".to_string(),
                text: t.to_string(),
                provenance: format!("{}@{}", user, repo),
            });
        }
    }
    thoughts
}

/// Lightweight comment extractor for code (`//` or `#` lines), bounded by level.
fn extract_code_comments(
    content: &str,
    path: &Path,
    user: &str,
    repo: &str,
    max_lines: usize,
) -> Vec<SalientThought> {
    let mut out = vec![];
    for (i, line) in content.lines().enumerate().take(max_lines) {
        let t = line.trim();
        if (t.starts_with("//") || t.starts_with('#')) && t.len() > 10 {
            out.push(SalientThought {
                source: format!("comment:{}:{}", path.display(), i + 1),
                text: t.to_string(),
                provenance: format!("{}@{}:{}", user, repo, path.display()),
            });
        }
    }
    out
}

/// Resolve a git repository's HEAD commit OID without spawning a subprocess.
fn resolve_git_head_oid(repo_path: &Path) -> Option<String> {
    let git_dir = repo_path.join(".git");
    let head_str = fs::read_to_string(git_dir.join("HEAD")).ok()?;
    let trimmed = head_str.trim();
    if let Some(ref_rel) = trimmed.strip_prefix("ref:") {
        let ref_path = git_dir.join(ref_rel.trim());
        if let Ok(oid) = fs::read_to_string(ref_path) {
            let oid = oid.trim().to_string();
            if !oid.is_empty() {
                return Some(oid);
            }
        }
        // Check packed-refs fallback
        if let Ok(packed) = fs::read_to_string(git_dir.join("packed-refs")) {
            let target_ref = ref_rel.trim();
            for line in packed.lines() {
                if line.starts_with('#') || line.starts_with('^') {
                    continue;
                }
                let mut parts = line.split_whitespace();
                if let (Some(oid), Some(rname)) = (parts.next(), parts.next()) {
                    if rname == target_ref {
                        return Some(oid.to_string());
                    }
                }
            }
        }
        None
    } else if trimmed.len() >= 7 && trimmed.chars().all(|c| c.is_ascii_hexdigit()) {
        Some(trimmed.to_string())
    } else {
        None
    }
}

/// Load review thoughts from a single `.jsonl` or `.json` file, or a directory containing `review-*.json`.
fn load_review_thoughts(review_path: Option<&Path>, is_sg: bool) -> HashMap<String, String> {
    let mut map = HashMap::new();
    let mut paths_to_load: Vec<PathBuf> = Vec::new();
    if let Some(p) = review_path {
        paths_to_load.push(p.to_path_buf());
    } else if is_sg {
        let sg_research = Path::new("/data/.agent-state/research/standardgalactic-20261028");
        if sg_research.is_dir() {
            if let Ok(rd) = fs::read_dir(sg_research) {
                for entry in rd.flatten() {
                    let name = entry.file_name().to_string_lossy().to_string();
                    if name.starts_with("review-") && name.ends_with(".json") {
                        paths_to_load.push(entry.path());
                    }
                }
            }
            let pass02 = sg_research.join("pass-02");
            if let Ok(rd) = fs::read_dir(&pass02) {
                for entry in rd.flatten() {
                    let name = entry.file_name().to_string_lossy().to_string();
                    if name.starts_with("review-") && name.ends_with(".json") {
                        paths_to_load.push(entry.path());
                    }
                }
            }
        }
    }

    for path in paths_to_load {
        let Ok(text) = fs::read_to_string(&path) else {
            continue;
        };
        if path.extension().and_then(|e| e.to_str()) == Some("json") {
            if let Ok(v) = serde_json::from_str::<serde_json::Value>(&text) {
                if let Some(arr) = v.get("deep_reviewed").and_then(|x| x.as_array()) {
                    for item in arr {
                        if let (Some(name), Some(decision)) = (
                            item.get("name").and_then(|x| x.as_str()),
                            item.get("decision").and_then(|x| x.as_str()),
                        ) {
                            let mech = item
                                .get("mechanism")
                                .and_then(|x| x.as_str())
                                .unwrap_or("");
                            map.insert(
                                name.to_string(),
                                format!("research-decision: {} (mechanism: {})", decision, mech),
                            );
                        }
                    }
                }
            }
        } else {
            for line in text.lines() {
                if let Ok(v) = serde_json::from_str::<serde_json::Value>(line) {
                    if let (Some(name), Some(decision)) = (
                        v.get("name").and_then(|x| x.as_str()),
                        v.get("decision").and_then(|x| x.as_str()),
                    ) {
                        let mech = v.get("mechanism").and_then(|x| x.as_str()).unwrap_or("");
                        map.insert(
                            name.to_string(),
                            format!("research-decision: {} (mechanism: {})", decision, mech),
                        );
                    }
                }
            }
        }
    }
    map
}

/// Inspect a single repository directory and extract its salient thoughts, samples, and family Rust port.
fn extract_salient_repo(
    repo_path: &Path,
    repo_name: &str,
    github_user: &str,
    level: SalienceLevel,
    review_thoughts: &HashMap<String, String>,
) -> SalientRepo {
    let head_oid = resolve_git_head_oid(repo_path);
    let mut repo = SalientRepo {
        name: repo_name.to_string(),
        github_user: github_user.to_string(),
        path: repo_path.to_path_buf(),
        head_oid,
        code_samples: vec![],
        thoughts: vec![],
        family_rust_port: None,
    };

    let depth = level.depth_limit();
    let max_readme_lines = depth.saturating_mul(2).max(10);
    let max_comment_lines = depth.saturating_mul(5).max(15);
    let max_samples = (depth / 3).clamp(1, 8);

    // 1. README thoughts
    for readme_name in ["README.md", "readme.md", "README.rst", "README.txt", "README"] {
        let p = repo_path.join(readme_name);
        if p.is_file() {
            if let Ok(readme) = fs::read_to_string(&p) {
                repo.thoughts.extend(extract_readme_thoughts(
                    &readme,
                    repo_name,
                    github_user,
                    max_readme_lines,
                ));
                if !repo.thoughts.is_empty() {
                    repo.code_samples.push(readme.chars().take(200).collect());
                }
                break;
            }
        }
    }

    // 2. Scan code files at repo root and common source subdirectories (`src`, `lib`, `crates`, `code`)
    let mut candidate_dirs = vec![repo_path.to_path_buf()];
    for sub in ["src", "lib", "code", "bin", "tests"] {
        let sub_path = repo_path.join(sub);
        if sub_path.is_dir() {
            candidate_dirs.push(sub_path);
        }
    }

    let mut files_visited = 0usize;
    'outer: for dir in candidate_dirs {
        let Ok(rd) = fs::read_dir(&dir) else {
            continue;
        };
        for f in rd.flatten() {
            if files_visited >= depth {
                break 'outer;
            }
            let fpath = f.path();
            if !fpath.is_file() {
                continue;
            }
            if let Some(ext) = fpath.extension().and_then(|e| e.to_str()) {
                if matches!(
                    ext,
                    "rs" | "py" | "c" | "h" | "cpp" | "js" | "ts" | "go" | "zig" | "sh" | "m" | "tex"
                ) {
                    files_visited += 1;
                    if let Ok(content) = fs::read_to_string(&fpath) {
                        let comments = extract_code_comments(
                            &content,
                            &fpath,
                            github_user,
                            repo_name,
                            max_comment_lines,
                        );
                        repo.thoughts.extend(comments);
                        if repo.code_samples.len() < max_samples {
                            repo.code_samples.push(content.chars().take(160).collect());
                        }
                    }
                }
            }
        }
    }

    // 3. Attach research review thought if present (ties directly to standardgalactic work).
    if let Some(thought) = review_thoughts.get(repo_name) {
        repo.thoughts.push(SalientThought {
            source: "research-review".to_string(),
            text: thought.clone(),
            provenance: format!("standardgalactic-review for {}", repo_name),
        });
    }

    // 4. Wire the family heart specially when the dedication motif appears in code or thoughts.
    if repo.thoughts.iter().any(|t| contains_family_heart(&t.text))
        || repo.code_samples.iter().any(|s| contains_family_heart(s))
    {
        repo.thoughts.push(SalientThought {
            source: "family-heart".to_string(),
            text: FAMILY_HEART.to_string(),
            provenance: format!(
                "{}@{} (wired from motif in source/thoughts)",
                github_user, repo_name
            ),
        });
    }

    // 5. Produce a family Rust candidate for the first salient sample.
    if let Some(first) = repo.code_samples.first() {
        repo.family_rust_port = Some(wire_family_heart(
            &salient_to_family_rust(first, "unknown"),
            &repo.thoughts,
        ));
    }

    repo
}

/// Construct a native MEM8 Wave header for a repository record.
/// Uses exact integer counts bounded into the native WaveAtom range:
/// - 432 Hz base frequency when family heart or research review is attached, else 108 Hz + thought count.
/// - Positive valence (0.75) for reviewed/heart repos, neutral-positive (0.25) otherwise.
/// - Arousal scaled by level depth.
fn repo_wave(repo: &SalientRepo, level: SalienceLevel) -> Wave {
    let has_heart_or_review = repo
        .thoughts
        .iter()
        .any(|t| t.source == "family-heart" || t.source == "research-review");
    let freq = if has_heart_or_review {
        432.0
    } else {
        let bounded_thoughts = repo.thoughts.len().min(324) as u16;
        f64::from(108u16.saturating_add(bounded_thoughts))
    };
    let valence = if has_heart_or_review { 0.75 } else { 0.25 };
    let arousal = match level {
        SalienceLevel::Extreme | SalienceLevel::Professor => 0.9,
        SalienceLevel::Expert => 0.75,
        SalienceLevel::High => 0.6,
        SalienceLevel::Medium => 0.5,
        SalienceLevel::Low => 0.3,
        SalienceLevel::Bullshitter => 0.1,
    };
    Wave::new(freq, valence, arousal)
}

/// Ingest a collection (or single repository) into a durable native MEM8 store (`mem8_out`).
/// If `review_jsonl` is `Some` (or auto-discovered for `standardgalactic`), it attaches
/// research review decisions as extra `"research-review"` thoughts.
pub fn ingest_github_user_collection(
    github_user: &str,
    collection_root: &Path,
    mem8_out: &Path,
    review_jsonl: Option<&Path>,
) -> Result<IngestReport> {
    ingest_with_options(
        github_user,
        collection_root,
        mem8_out,
        review_jsonl,
        SalienceLevel::Expert,
        true,
    )
}

pub fn ingest_with_options(
    github_user: &str,
    collection_root: &Path,
    mem8_out: &Path,
    review_jsonl: Option<&Path>,
    level: SalienceLevel,
    force: bool,
) -> Result<IngestReport> {
    let is_sg = github_user.eq_ignore_ascii_case("standardgalactic");
    let review_thoughts = load_review_thoughts(review_jsonl, is_sg);

    let mut store = RecordStore::open_memory(mem8_out)
        .with_context(|| format!("failed to open native MEM8 store at {}", mem8_out.display()))?;

    let mut report = IngestReport {
        github_user: github_user.to_string(),
        repos_processed: 0,
        repos_persisted: 0,
        repos_skipped_unchanged: 0,
        total_thoughts: 0,
        total_code_samples: 0,
        mem8_path: Some(mem8_out.to_path_buf()),
        mem8_bytes: 0,
    };

    // Auto-detect whether `collection_root` is a single repo (`.git` inside `collection_root`)
    // or a collection directory of multiple repos.
    let is_single_repo = collection_root.join(".git").exists()
        || (collection_root.join("README.md").is_file()
            && (collection_root.join("Cargo.toml").is_file()
                || collection_root.join("pyproject.toml").is_file()
                || collection_root.join("package.json").is_file()
                || collection_root.join("go.mod").is_file()));

    let repo_targets: Vec<(String, PathBuf)> = if is_single_repo {
        let name = collection_root
            .canonicalize()
            .ok()
            .and_then(|p| p.file_name().map(|n| n.to_string_lossy().to_string()))
            .or_else(|| {
                collection_root
                    .file_name()
                    .map(|n| n.to_string_lossy().to_string())
            })
            .unwrap_or_else(|| "repo".to_string());
        vec![(name, collection_root.to_path_buf())]
    } else {
        let entries = fs::read_dir(collection_root).with_context(|| {
            format!(
                "cannot read collection root {}",
                collection_root.display()
            )
        })?;
        let mut list = Vec::new();
        for entry in entries.flatten() {
            if !entry.file_type().map(|t| t.is_dir()).unwrap_or(false) {
                continue;
            }
            let repo_name = entry.file_name().to_string_lossy().to_string();
            if repo_name.starts_with('.') {
                continue;
            }
            list.push((repo_name, entry.path()));
        }
        list.sort_by(|a, b| a.0.cmp(&b.0));
        list
    };

    let mut last_persisted_key: Option<String> = None;

    for (repo_name, repo_path) in repo_targets {
        let key = format!("salient:repo:{}/{}", github_user, repo_name);
        let current_oid = resolve_git_head_oid(&repo_path);

        // Incremental skilling: if not `--force`, check if this repo's exact HEAD OID is already in MEM8.
        if !force {
            if let Some(ref oid) = current_oid {
                if let Ok(Some(existing)) = store.get::<SalientRepo>(&key) {
                    if existing.head_oid.as_deref() == Some(oid.as_str()) {
                        report.repos_processed += 1;
                        report.repos_skipped_unchanged += 1;
                        report.total_thoughts += existing.thoughts.len();
                        report.total_code_samples += existing.code_samples.len();
                        continue;
                    }
                }
            }
        }

        let repo = extract_salient_repo(&repo_path, &repo_name, github_user, level, &review_thoughts);
        report.repos_processed += 1;
        report.total_thoughts += repo.thoughts.len();
        report.total_code_samples += repo.code_samples.len();

        if !repo.thoughts.is_empty() || !repo.code_samples.is_empty() {
            let wave = repo_wave(&repo, level);
            store.put_with_wave_buffered(&key, &repo, &wave)?;
            report.repos_persisted += 1;
            last_persisted_key = Some(key);
        }
    }

    // Persist summary receipt record in the same native MEM8 store (flushes & syncs to disk)
    let summary_key = format!("salient:summary:{}", github_user);
    store.put_with_wave(
        &summary_key,
        &report,
        &Wave::new(432.0, 0.8, 0.8),
    )?;
    store.flush_sync()?;

    // Read-back verification on the summary and last persisted repo record
    let verified_summary: Option<IngestReport> = store.get(&summary_key)?;
    anyhow::ensure!(
        verified_summary.is_some(),
        "MEM8 read-back verification failed for {}",
        summary_key
    );
    if let Some(ref k) = last_persisted_key {
        let verified_repo: Option<SalientRepo> = store.get(k)?;
        anyhow::ensure!(
            verified_repo.is_some(),
            "MEM8 read-back verification failed for {}",
            k
        );
    }

    drop(store);
    report.mem8_bytes = fs::metadata(mem8_out).map(|m| m.len()).unwrap_or(0);

    Ok(report)
}

/// High-level remember entry point for `st remember <target> --level <LEVEL>`.
/// Supports URLs (parsed for owner/repo), local paths (single repo or user collection), and git forges.
/// Handles incremental diffs from known MEM8 state, token+zlib compression, MEM8 wave blocks, and skilling.
/// Special full-nexus treatment for StandardGalactic.
/// Limits "too much" (`extreme`/`professor`) on non-StandardGalactic sources politely to `Expert`.
pub fn remember(
    target: &str,
    level_str: &str,
    out: Option<&std::path::Path>,
    force: bool,
) -> Result<IngestReport> {
    let requested_level = SalienceLevel::from_str(level_str);
    let is_standard_galactic = target.to_lowercase().contains("standardgalactic")
        || target.to_lowercase().contains("nate");

    let effective_level = if is_standard_galactic {
        SalienceLevel::Extreme
    } else if matches!(
        requested_level,
        SalienceLevel::Extreme | SalienceLevel::Professor
    ) {
        eprintln!("dude.. That's too much.. :D  Falling back to expert for this source.");
        SalienceLevel::Expert
    } else {
        requested_level
    };

    // Resolve target (URL -> local collection in /data/<owner>[/<repo>] if available)
    let (github_user, collection_path): (String, PathBuf) =
        if target.starts_with("http://")
            || target.starts_with("https://")
            || target.starts_with("git@")
        {
            if is_standard_galactic {
                (
                    "standardgalactic".to_string(),
                    PathBuf::from("/data/standardgalactic"),
                )
            } else {
                // Parse https://github.com/<owner>[/<repo>] or git@github.com:<owner>[/<repo>]
                let cleaned = target
                    .trim_end_matches('/')
                    .trim_end_matches(".git")
                    .split("://")
                    .last()
                    .unwrap_or(target);
                let path_part = cleaned
                    .split_once('/')
                    .map(|(_, p)| p)
                    .or_else(|| cleaned.split_once(':').map(|(_, p)| p))
                    .unwrap_or("");
                let segments: Vec<&str> = path_part.split('/').filter(|s| !s.is_empty()).collect();
                if segments.len() >= 2 {
                    let owner = segments[0];
                    let repo = segments[1];
                    let candidate = PathBuf::from("/data").join(owner).join(repo);
                    if candidate.exists() {
                        (owner.to_string(), candidate)
                    } else {
                        (owner.to_string(), PathBuf::from("."))
                    }
                } else if segments.len() == 1 {
                    let owner = segments[0];
                    let candidate = PathBuf::from("/data").join(owner);
                    if candidate.exists() {
                        (owner.to_string(), candidate)
                    } else {
                        (owner.to_string(), PathBuf::from("."))
                    }
                } else {
                    ("unknown".to_string(), PathBuf::from("."))
                }
            }
        } else {
            let p = PathBuf::from(target);
            let owner = if is_standard_galactic {
                "standardgalactic".to_string()
            } else {
                // Derive user from parent or directory name (e.g. /data/peterlodri-sec/taiko-01-protocol-demo -> peterlodri-sec)
                let canon = p.canonicalize().unwrap_or_else(|_| p.clone());
                if canon.join(".git").exists() {
                    canon
                        .parent()
                        .and_then(|parent| parent.file_name())
                        .map(|n| n.to_string_lossy().to_string())
                        .unwrap_or_else(|| "local".to_string())
                } else {
                    canon
                        .file_name()
                        .map(|n| n.to_string_lossy().to_string())
                        .unwrap_or_else(|| "local".to_string())
                }
            };
            (owner, p)
        };

    let mem8_out = out.map(|p| p.to_path_buf()).unwrap_or_else(|| {
        if is_standard_galactic {
            PathBuf::from("/data/.mem8/nexus/standardgalactic.m8")
        } else {
            let slug = collection_path
                .file_name()
                .map(|n| n.to_string_lossy().to_string())
                .unwrap_or_else(|| github_user.clone());
            PathBuf::from("/data/.mem8/salient").join(format!("{}-{}.m8", github_user, slug))
        }
    });

    println!(
        "Remembering target {:?} (owner: {}) at level {:?} (SG special: {}) -> {:?}",
        collection_path, github_user, effective_level, is_standard_galactic, mem8_out
    );

    let report = ingest_with_options(
        &github_user,
        &collection_path,
        &mem8_out,
        None,
        effective_level,
        force,
    )?;

    println!(
        "MEM8 native store persisted & read-back verified: {} ({} bytes, {} written, {} unchanged skipped).",
        mem8_out.display(),
        report.mem8_bytes,
        report.repos_persisted,
        report.repos_skipped_unchanged
    );

    Ok(report)
}

/// Resolve the default MEM8 file path and owner for a target string.
pub fn resolve_target_and_mem8_path(
    target: &str,
    out: Option<&std::path::Path>,
) -> (String, PathBuf, PathBuf) {
    let is_standard_galactic = target.to_lowercase().contains("standardgalactic")
        || target.to_lowercase().contains("nate");

    let (github_user, collection_path): (String, PathBuf) =
        if target.starts_with("http://")
            || target.starts_with("https://")
            || target.starts_with("git@")
        {
            if is_standard_galactic {
                (
                    "standardgalactic".to_string(),
                    PathBuf::from("/data/standardgalactic"),
                )
            } else {
                let cleaned = target
                    .trim_end_matches('/')
                    .trim_end_matches(".git")
                    .split("://")
                    .last()
                    .unwrap_or(target);
                let path_part = cleaned
                    .split_once('/')
                    .map(|(_, p)| p)
                    .or_else(|| cleaned.split_once(':').map(|(_, p)| p))
                    .unwrap_or("");
                let segments: Vec<&str> = path_part.split('/').filter(|s| !s.is_empty()).collect();
                if segments.len() >= 2 {
                    let owner = segments[0];
                    let repo = segments[1];
                    let candidate = PathBuf::from("/data").join(owner).join(repo);
                    if candidate.exists() {
                        (owner.to_string(), candidate)
                    } else {
                        (owner.to_string(), PathBuf::from("."))
                    }
                } else if segments.len() == 1 {
                    let owner = segments[0];
                    let candidate = PathBuf::from("/data").join(owner);
                    if candidate.exists() {
                        (owner.to_string(), candidate)
                    } else {
                        (owner.to_string(), PathBuf::from("."))
                    }
                } else {
                    ("unknown".to_string(), PathBuf::from("."))
                }
            }
        } else {
            let p = PathBuf::from(target);
            if p.extension().and_then(|e| e.to_str()) == Some("m8") && p.is_file() {
                let stem = p
                    .file_stem()
                    .map(|s| s.to_string_lossy().to_string())
                    .unwrap_or_else(|| "local".to_string());
                return (stem, p.clone(), p);
            }
            let owner = if is_standard_galactic {
                "standardgalactic".to_string()
            } else {
                let canon = p.canonicalize().unwrap_or_else(|_| p.clone());
                if canon.join(".git").exists() {
                    canon
                        .parent()
                        .and_then(|parent| parent.file_name())
                        .map(|n| n.to_string_lossy().to_string())
                        .unwrap_or_else(|| "local".to_string())
                } else {
                    canon
                        .file_name()
                        .map(|n| n.to_string_lossy().to_string())
                        .unwrap_or_else(|| "local".to_string())
                }
            };
            (owner, p)
        };

    let mem8_out = out.map(|p| p.to_path_buf()).unwrap_or_else(|| {
        if is_standard_galactic {
            PathBuf::from("/data/.mem8/nexus/standardgalactic.m8")
        } else {
            let slug = collection_path
                .file_name()
                .map(|n| n.to_string_lossy().to_string())
                .unwrap_or_else(|| github_user.clone());
            PathBuf::from("/data/.mem8/salient").join(format!("{}-{}.m8", github_user, slug))
        }
    });

    (github_user, collection_path, mem8_out)
}

/// Inspect and summarize what has been learned so far in a salient MEM8 store (`st remember <target> --check [-q <query>]`).
pub fn check_learned(
    target: &str,
    out: Option<&std::path::Path>,
    query: Option<&str>,
) -> Result<()> {
    let (github_user, _collection_path, mem8_path) = resolve_target_and_mem8_path(target, out);
    anyhow::ensure!(
        mem8_path.exists(),
        "No MEM8 store found at {} yet. Run `st remember {}` first.",
        mem8_path.display(),
        target
    );

    let file_bytes = fs::metadata(&mem8_path).map(|m| m.len()).unwrap_or(0);
    let mut store = RecordStore::open_memory(&mem8_path)
        .with_context(|| format!("failed to open native MEM8 store at {}", mem8_path.display()))?;

    let mut repo_keys = store.keys_with_prefix("salient:repo:");
    repo_keys.sort();

    let summary_key = format!("salient:summary:{}", github_user);
    let summary: Option<IngestReport> = store.get(&summary_key).ok().flatten();

    println!("========================================================================");
    println!("🧠 Salient MEM|8 Checkpoint — What We Have Learned So Far");
    println!("========================================================================");
    println!("  Store Path:          {}", mem8_path.display());
    println!("  Native Size:         {} bytes ({} MiB)", file_bytes, file_bytes / (1024 * 1024));
    println!("  Indexed Repo Keys:   {}", repo_keys.len());
    if let Some(ref s) = summary {
        println!(
            "  Last Run Receipt:    {} repos processed ({} written, {} skipped), {} thoughts, {} code samples",
            s.repos_processed, s.repos_persisted, s.repos_skipped_unchanged, s.total_thoughts, s.total_code_samples
        );
    }

    let q_lower = query.map(|q| q.to_lowercase());
    let mut reviewed_repos: Vec<SalientRepo> = Vec::new();
    let mut heart_repos: Vec<String> = Vec::new();
    let mut rational_candidates = 0usize;
    let mut matched_repos: Vec<SalientRepo> = Vec::new();

    // Sample or scan records: if a query is provided or total keys <= 500, scan all;
    // for a 10,000-repo store without a query, scan all keys for research-reviews and sample top entries.
    for key in &repo_keys {
        if let Some(ref q) = q_lower {
            let Ok(Some(repo)) = store.get::<SalientRepo>(key) else {
                continue;
            };
            let name_match = repo.name.to_lowercase().contains(q);
            let thought_match = repo
                .thoughts
                .iter()
                .any(|t| t.text.to_lowercase().contains(q) || t.source.to_lowercase().contains(q));
            if name_match || thought_match {
                matched_repos.push(repo);
            }
        } else {
            let Ok(Some(repo)) = store.get::<SalientRepo>(key) else {
                continue;
            };
            if repo
                .family_rust_port
                .as_ref()
                .is_some_and(|p| p.contains("Rational"))
            {
                rational_candidates += 1;
            }
            if repo.thoughts.iter().any(|t| t.source == "family-heart") {
                heart_repos.push(repo.name.clone());
            }
            if repo.thoughts.iter().any(|t| t.source == "research-review") {
                reviewed_repos.push(repo);
            } else if matched_repos.len() < 8 {
                matched_repos.push(repo);
            }
        }
    }

    if let Some(q) = query {
        println!("\n🔍 Query `{}` matched {} repositories in MEM|8:", q, matched_repos.len());
        for repo in matched_repos.iter().take(15) {
            let wave = store
                .get_wave(&format!("salient:repo:{}/{}", repo.github_user, repo.name))
                .ok()
                .flatten();
            let hz = wave.as_ref().map(|w| w.frequency as u32).unwrap_or(0);
            println!(
                "\n  • {}/{} (HEAD: {}, Wave: {} Hz, {} thoughts, {} samples)",
                repo.github_user,
                repo.name,
                repo.head_oid.as_deref().unwrap_or("none"),
                hz,
                repo.thoughts.len(),
                repo.code_samples.len()
            );
            for t in repo.thoughts.iter().take(5) {
                let snippet: String = t.text.chars().take(110).collect();
                println!("      [{}] {}", t.source, snippet);
            }
            if let Some(ref port) = repo.family_rust_port {
                let first_line = port.lines().nth(2).unwrap_or("// port ready");
                println!("      [family-rust] {}", first_line);
            }
        }
    } else {
        println!("  Exact-Math (p/q) Candidates: {} repos flagged for Rational conversion", rational_candidates);
        println!("  Research-Reviewed in MEM|8:  {} repos with attached pass-01/pass-02 decisions", reviewed_repos.len());
        println!("  Family Heart Motif Wired:    {} repos carrying {}", heart_repos.len(), FAMILY_HEART);

        if !reviewed_repos.is_empty() {
            println!("\n📋 Sample Research-Reviewed Repositories Stored at 432 Hz:");
            for repo in reviewed_repos.iter().take(12) {
                let rev = repo
                    .thoughts
                    .iter()
                    .find(|t| t.source == "research-review")
                    .map(|t| t.text.as_str())
                    .unwrap_or("");
                let snippet: String = rev.chars().take(105).collect();
                println!(
                    "  • {:36} | {:7} | {}...",
                    repo.name,
                    &repo.head_oid.as_deref().unwrap_or("no-git")[..7.min(repo.head_oid.as_deref().unwrap_or("no-git").len())],
                    snippet
                );
            }
        }

        if !matched_repos.is_empty() {
            println!("\n🌱 Sample Unreviewed Salient Repositories Ready for Triage:");
            for repo in matched_repos.iter().take(8) {
                let first_thought = repo
                    .thoughts
                    .first()
                    .map(|t| t.text.chars().take(90).collect::<String>())
                    .unwrap_or_default();
                let needs_pq = repo
                    .family_rust_port
                    .as_ref()
                    .is_some_and(|p| p.contains("Rational"));
                println!(
                    "  • {:28} | {:4} thoughts | p/q candidate: {:5} | {}",
                    repo.name,
                    repo.thoughts.len(),
                    needs_pq,
                    first_thought
                );
            }
        }
    }

    println!("\nThe constellation · fine touch from within · {}", FAMILY_HEART);
    Ok(())
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SalienceLevel {
    Extreme,
    Professor,
    Expert,
    High,
    Medium,
    Low,
    Bullshitter,
}

impl SalienceLevel {
    pub fn from_str(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "extreme" => Self::Extreme,
            "professor" => Self::Professor,
            "expert" => Self::Expert,
            "high" => Self::High,
            "medium" => Self::Medium,
            "low" => Self::Low,
            "bullshitter" | "bs" => Self::Bullshitter,
            _ => Self::Expert,
        }
    }

    pub fn depth_limit(&self) -> usize {
        match self {
            Self::Extreme => 100,
            Self::Professor | Self::Expert => 20,
            Self::High => 10,
            Self::Medium => 5,
            Self::Low => 2,
            Self::Bullshitter => 1,
        }
    }

    pub fn should_compress(&self) -> bool {
        !matches!(self, Self::Bullshitter)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::tempdir;

    #[test]
    fn converts_floats_to_rational_suggestion() {
        let port = salient_to_family_rust("x = a / 3.14", "py");
        assert!(port.contains("Rational"));
        assert!(port.contains("exact arithmetic"));
    }

    #[test]
    fn ingests_collection_and_attaches_review_thoughts_and_persists_mem8() {
        let dir = tempdir().unwrap();
        let user_dir = dir.path().join("testuser");
        fs::create_dir(&user_dir).unwrap();
        let repo = user_dir.join("demo-repo");
        fs::create_dir(&repo).unwrap();
        fs::write(repo.join("README.md"), "# Demo\nThis remembers salient code.").unwrap();
        fs::write(repo.join("main.py"), "# thought: use exact math here\nprint(1/3)").unwrap();

        let review = dir.path().join("reviews.jsonl");
        fs::write(
            &review,
            "{\"name\":\"demo-repo\",\"decision\":\"adapt\",\"mechanism\":\"exact port\"}\n",
        )
        .unwrap();

        let out_m8 = dir.path().join("out.m8");
        let report = ingest_github_user_collection(
            "testuser",
            &user_dir,
            &out_m8,
            Some(&review),
        )
        .unwrap();

        assert_eq!(report.repos_processed, 1);
        assert_eq!(report.repos_persisted, 1);
        assert!(report.total_thoughts >= 2);
        assert!(out_m8.exists());
        assert!(report.mem8_bytes > 0);

        // Verify native MEM8 read-back
        let mut store = RecordStore::open_memory(&out_m8).unwrap();
        let stored: SalientRepo = store
            .get("salient:repo:testuser/demo-repo")
            .unwrap()
            .expect("stored repo must exist");
        assert_eq!(stored.name, "demo-repo");
        assert!(stored
            .thoughts
            .iter()
            .any(|t| t.source == "research-review" && t.text.contains("exact port")));
    }

    #[test]
    fn single_repo_and_polite_fallback_work() {
        let dir = tempdir().unwrap();
        let repo = dir.path().join("single-repo");
        fs::create_dir_all(repo.join(".git")).unwrap();
        fs::write(repo.join(".git/HEAD"), "0123456789abcdef0123456789abcdef01234567\n").unwrap();
        fs::write(repo.join("README.md"), "# Single Repo\nLove the work {<3,<3,<3}+1").unwrap();
        fs::write(repo.join("lib.rs"), "// salient comment in rust file\npub fn add(a: i64, b: i64) -> i64 { a + b }\n").unwrap();

        let out_m8 = dir.path().join("single.m8");
        // Requesting "extreme" on non-SG repo should politely fall back to Expert and ingest the single repo
        let report = remember(
            repo.to_str().unwrap(),
            "extreme",
            Some(&out_m8),
            false,
        )
        .unwrap();

        assert_eq!(report.repos_processed, 1);
        assert_eq!(report.repos_persisted, 1);
        assert!(report.total_thoughts >= 2);
        assert!(out_m8.exists());

        // Second run without --force should skip unchanged repo via HEAD OID match
        let report2 = remember(
            repo.to_str().unwrap(),
            "high",
            Some(&out_m8),
            false,
        )
        .unwrap();
        assert_eq!(report2.repos_processed, 1);
        assert_eq!(report2.repos_persisted, 0);
        assert_eq!(report2.repos_skipped_unchanged, 1);
    }
}
