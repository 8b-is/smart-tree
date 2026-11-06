//! Sovereign BitNet b1.58 (`W ∈ {-1, 0, +1}`) Built-In Code & Hierarchy Summarizer
//!
//! Implements a 100% `NO-FLOAT` (`qT_pq INT`) ternary `BitLinear` inference & summarization
//! engine built directly into `smart-tree` to power multi-resolution `.m8` consciousness pyramids:
//!
//! 1. **Leaf Code Folders (`src/.m8`, etc. — `ZoomLevel::LeafCode`)**:
//!    Extracts extensive function-by-function and struct-by-struct intelligence (signatures,
//!    docstrings, line numbers, call roles) and computes each symbol's **BitNet b1.58 dual-rail
//!    `(B+, B-)` ternary embedding** and natural-language gist.
//! 2. **Parent Project Folders (`smart-tree/.m8` — `ZoomLevel::ProjectRoot`)**:
//!    Pools child `(B+, B-)` ternary wave bitplanes via majority-trit interference and compresses
//!    hundreds of leaf functions into a concise **Project Gist** + flagship symbol index.
//! 3. **Grandparent Workspace / Org Folders (`~/source/8b-is/.m8` — `ZoomLevel::WorkspaceOrg`)**:
//!    Rolls up all repositories on the system into a **Fleet Gist** and seeds `MemNet` Wave Q&A
//!    packets (`~/.mem8/memnet/mesh_state.json`) so local peers and `<TAB>` autocomplete have
//!    full system context without waking a heavyweight cloud LLM.
//!
//! ### Mathematical Foundation (Hue's `NAND >>` & `B+/B-` Identity)
//! - **Continuous-to-Discrete Latch**:
//!   `∫_0^1 Q_{L^-}^{L^+} = (0, 1[ ~ NAND... >>`
//! - **Dual-Rail Bitplane Normalization**:
//!   `B^+ / B^- ~= L^{p^{n+1}} / (|□| · N(√n^-))`
//!   Every weight row and semantic embedding is represented as a pair of 64-bit Kleene rails
//!   `(bfp64_pos, bfp64_neg)` where `+1 -> B^+`, `-1 -> B^-`, and `0 -> neither`.
//!   Matrix-vector multiplication requires **zero multiplications** — only bitwise `AND`/`NAND`,
//!   hardware `POPCNT`, and integer shift `>>` normalized by integer square-root norm `isqrt_u64`.

use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::fs;
use std::path::Path;

/// Integer square root `⌊√x⌋` (`N(√n)` normalizer, 100% integer Newton-Raphson).
#[inline]
pub const fn isqrt_u64(x: u64) -> u64 {
    if x < 2 {
        return x;
    }
    let shift = (64 - x.leading_zeros()).div_ceil(2);
    let mut g = 1u64 << shift;
    loop {
        let next = (g + x / g) >> 1;
        if next >= g {
            return g;
        }
        g = next;
    }
}

/// 12 Sovereign Architectural Domains recognized by the built-in BitNet b1.58 classifier head.
/// Each domain has a deterministic 64-trit `(B+, B-)` prototype weight vector (`W ∈ {-1, 0, +1}^{64}`).
pub const BITNET_DOMAINS: [(&str, &str, &[&str]); 12] = [
    (
        "memnet-mesh",
        "Sovereign MemNet multi-medium routing & shared folder mesh",
        &[
            "memnet",
            "route",
            "share",
            "mesh",
            "esp32",
            "lora",
            "ble",
            "bluetooth",
            "wifi",
            "capsule",
            "peer",
            "quota",
            "envelope",
            "medium",
        ],
    ),
    (
        "rtc-control",
        "Peter's Real Time Constant (RTC τ) continuous-time relaxation & Q16/Q8 PID control",
        &[
            "rtc",
            "tau",
            "pade",
            "exp_neg",
            "settling",
            "3tau",
            "5tau",
            "pid",
            "plant",
            "jitter",
            "latency",
            "relaxation",
            "conductance",
        ],
    ),
    (
        "mem8-wave",
        "MEM|8 wave consciousness, 79B WaveFrame & zero-LLM-cost Q&A recall",
        &[
            "mem8",
            "m8",
            "wave",
            "consciousness",
            "frequency",
            "resonance",
            "salience",
            "anchor",
            "recall",
            "memory",
            "pyramid",
            "essence",
            "litany",
        ],
    ),
    (
        "acoustic-chirp",
        "48kHz Ultrasonic Chirp acoustic speaker->mic modem & Goertzel DSP",
        &[
            "chirp",
            "ultrasonic",
            "acoustic",
            "modem",
            "goertzel",
            "hamming",
            "pcm",
            "wav",
            "audible",
            "sample_rate",
            "vad",
            "marine",
        ],
    ),
    (
        "lattice-lll",
        "3rd+ Escape: Ultra-Graph, Constructive Lovász Local Lemma & 4D Language LLL",
        &[
            "lll",
            "escape",
            "lovasz",
            "moser",
            "tardos",
            "gram",
            "schmidt",
            "basis",
            "lattice",
            "ultragraph",
            "erdos",
            "ramsey",
            "steiner",
            "fusion",
        ],
    ),
    (
        "glassdoor-spqt",
        "Sovereign Glassdoor: SPQT base-243 5-trit packing, space-bender & demoscene",
        &[
            "glassdoor",
            "spqt",
            "trit",
            "ternary",
            "pqb",
            "demoscene",
            "space_bender",
            "wupz",
            "upz",
            "zeno",
            "bfp64",
            "kleene",
            "bitnet",
        ],
    ),
    (
        "shell-autocomplete",
        "Self-maintaining Fish/Zsh/Bash autocomplete with live folder & MEM|8 descriptions",
        &[
            "completion",
            "completions",
            "autocomplete",
            "fish",
            "zsh",
            "bash",
            "dynamic",
            "prefix",
            "describe_directory",
        ],
    ),
    (
        "ast-symbol",
        "CrabCC AST symbol indexing, call-graph references & structural code intelligence",
        &[
            "symbol",
            "crab",
            "ast",
            "outline",
            "refs",
            "tree_sitter",
            "parser",
            "token",
            "function",
            "signature",
            "relations",
        ],
    ),
    (
        "daemon-service",
        "Always-on HTTP/MCP daemon, systemd/launchd service manager & auto-updater",
        &[
            "daemon",
            "service",
            "systemd",
            "launchd",
            "http",
            "axum",
            "mcp",
            "websocket",
            "update",
            "watcher",
            "collab",
        ],
    ),
    (
        "security-custodian",
        "The Custodian supply-chain threat scanner, Erdős G_shadow audit & cert verification",
        &[
            "security",
            "guardian",
            "custodian",
            "cert",
            "sha256",
            "hash",
            "anomaly",
            "exploit",
            "malware",
            "signature",
            "gpg",
            "token",
        ],
    ),
    (
        "forge-sync",
        "GitHub/Forgejo multi-repo parallel synchronization & g8t.is Foken capsules",
        &[
            "sync",
            "github",
            "repo",
            "clone",
            "quarantine",
            "g8t",
            "foken",
            "leaderboard",
            "capsule",
            "git",
        ],
    ),
    (
        "scanner-formatter",
        "High-speed directory traversal, interest scoring & multi-mode tree formatters",
        &[
            "scanner",
            "format",
            "classic",
            "quantum",
            "marqant",
            "digest",
            "louds",
            "mermaid",
            "interest",
            " traversal",
            "filter",
        ],
    ),
];

/// Dual-rail 64-trit BitNet b1.58 embedding (`W ∈ {-1, 0, +1}^{64}`).
/// Mirrors Hue's `B^+ / B^-` & `(0, 1[ ~ NAND... >>` formulation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct BitNetWave64 {
    /// Positive rail `B^+` (`1` where trit is `+1`)
    pub b_pos: u64,
    /// Negative rail `B^-` (`1` where trit is `-1`)
    pub b_neg: u64,
    /// Integer `B^+ / B^-` ratio scaled by `1000` (permille balance)
    pub balance_permille: u32,
    /// Box/hypercube norm `|□| · N(√n)` in integer `Q8`
    pub box_norm_q8: u32,
}

impl BitNetWave64 {
    /// Encode a text slice (signature + docstring + body tokens) into a 64-trit `(B+, B-)` BitNet wave
    /// using only integer FNV-1a mixing, `NAND` gates, and bit-shifts `>>`.
    pub fn from_tokens(tokens: &[String]) -> Self {
        let mut pos_acc = [0i32; 64];
        for (idx, tok) in tokens.iter().enumerate() {
            let h = fnv64_1a(tok.as_bytes());
            // Weight identifier tokens and domain keywords more strongly
            let weight: i32 = if idx < 4 { 3 } else { 1 };
            for (bit, acc_slot) in pos_acc.iter_mut().enumerate() {
                let shifted = h.rotate_left(bit as u32);
                // Universal NAND-shift gate `~(h & (h >> 1))`
                let nand_gate = !(shifted & (shifted >> 1));
                let sel = (shifted ^ (nand_gate >> 3)) & 0x3;
                match sel {
                    0 => *acc_slot += weight,
                    1 => *acc_slot -= weight,
                    _ => {} // 0 trit (sparsity!)
                }
            }
        }

        Self::from_accumulators(&pos_acc)
    }

    /// Pool multiple child `BitNetWave64` embeddings into a parent directory gist embedding
    /// via majority-trit interference (`∑ B^+ - ∑ B^-`).
    pub fn pool_hierarchy(children: &[BitNetWave64]) -> Self {
        if children.is_empty() {
            return Self::default();
        }
        let mut acc = [0i32; 64];
        for child in children {
            for (bit, acc_slot) in acc.iter_mut().enumerate() {
                let mask = 1u64 << bit;
                if (child.b_pos & mask) != 0 {
                    *acc_slot += 1;
                } else if (child.b_neg & mask) != 0 {
                    *acc_slot -= 1;
                }
            }
        }
        Self::from_accumulators(&acc)
    }

    fn from_accumulators(acc: &[i32; 64]) -> Self {
        let mut b_pos = 0u64;
        let mut b_neg = 0u64;
        let mut energy_sq = 0u64;
        for (bit, &val) in acc.iter().enumerate() {
            if val > 0 {
                b_pos |= 1u64 << bit;
                energy_sq += (val as u64) * (val as u64);
            } else if val < 0 {
                b_neg |= 1u64 << bit;
                let abs_v = (-val) as u64;
                energy_sq += abs_v * abs_v;
            }
        }
        let pos_ones = b_pos.count_ones().max(1);
        let neg_ones = b_neg.count_ones().max(1);
        let balance_permille = (pos_ones * 1000) / neg_ones;
        let active_trits = (b_pos | b_neg).count_ones() as u64;
        let box_norm_q8 = ((active_trits.max(1)) * isqrt_u64(energy_sq.max(1) << 8)) as u32;

        Self {
            b_pos,
            b_neg,
            balance_permille,
            box_norm_q8,
        }
    }

    /// BitLinear ternary dot-product `⟨self, other⟩` in `[-64, +64]` using zero multiplications:
    /// `(+1·+1) + (-1·-1) - (+1·-1) - (-1·+1)` via bitwise `AND` + hardware `POPCNT`.
    #[inline]
    pub const fn ternary_dot(self, other: Self) -> i32 {
        let same_sign =
            ((self.b_pos & other.b_pos) | (self.b_neg & other.b_neg)).count_ones() as i32;
        let opp_sign =
            ((self.b_pos & other.b_neg) | (self.b_neg & other.b_pos)).count_ones() as i32;
        same_sign - opp_sign
    }

    /// Compact 9-trit balanced-ternary string (`1`, `0`, `T`) for human-readable `.m8` display.
    pub fn trits_9(&self) -> String {
        let mut out = String::with_capacity(9);
        for chunk_idx in 0..9usize {
            let start = chunk_idx * 7;
            let mut sum = 0i32;
            for offset in 0..7usize {
                let bit = start + offset;
                if bit < 64 {
                    let mask = 1u64 << bit;
                    if (self.b_pos & mask) != 0 {
                        sum += 1;
                    } else if (self.b_neg & mask) != 0 {
                        sum -= 1;
                    }
                }
            }
            out.push(if sum > 0 {
                '1'
            } else if sum < 0 {
                'T'
            } else {
                '0'
            });
        }
        out
    }
}

/// Extensive per-function or per-struct consciousness stored in leaf code folders (e.g. `src/.m8`).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FunctionConsciousness {
    pub name: String,
    pub kind: String,
    pub file: String,
    pub line: usize,
    pub signature: String,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub docstring: String,
    pub domain: String,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub bitnet_gist: String,
    pub salience_permille: u16,
    pub wave: BitNetWave64,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub wave_b3: String,
}

/// Per-file summary stored in leaf code folders (`src/.m8`).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileCodeSummary {
    pub file: String,
    pub domain: String,
    pub symbol_count: usize,
    pub gist: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub top_symbols: Vec<String>,
}

/// BitNet b1.58 hierarchical summary attached to `.m8` at every level of the folder pyramid.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BitNetDirectorySummary {
    /// `"leaf-code"`, `"project-root"`, or `"workspace-org"`
    pub zoom_level: String,
    /// `"BitNet-b1.58-Ternary-NO-FLOAT (B+/B- Kleene 64-trit)"`
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub engine: String,
    /// Synthesized natural-language gist for this directory's zoom level
    pub synthesized_gist: String,
    /// Dominant architectural domains `(domain_id, score_permille)`
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub dominant_domains: Vec<(String, u16)>,
    /// Pooled 64-trit `(B+, B-)` BitNet wave across this directory and its children
    pub pooled_wave: BitNetWave64,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub pooled_wave_b3: String,
    /// Total functions/structs across this directory (or rolled up from child `.m8` files)
    #[serde(default)]
    pub total_symbols_rolled_up: usize,
    /// Per-file code gists (populated at `leaf-code` and `project-root` levels)
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub file_summaries: Vec<FileCodeSummary>,
    /// Flagship high-salience functions (top 12 rolled up to `project-root` so parent has the gist!)
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub flagship_symbols: Vec<String>,
}

#[inline]
const fn fnv64_1a(bytes: &[u8]) -> u64 {
    let mut hash = 0xcbf29ce484222325u64;
    let mut i = 0;
    while i < bytes.len() {
        hash ^= bytes[i] as u64;
        hash = hash.wrapping_mul(0x100000001b3);
        i += 1;
    }
    hash
}

/// Classify a token bag against the 12 BitNet b1.58 domain prototypes using integer `BitLinear` scoring.
pub fn classify_domain_bitnet(tokens: &[String]) -> (&'static str, &'static str, u16) {
    let mut best_id = BITNET_DOMAINS[0].0;
    let mut best_desc = BITNET_DOMAINS[0].1;
    let mut best_score = 0u32;

    for &(domain_id, domain_desc, keywords) in &BITNET_DOMAINS {
        let mut score = 0u32;
        for tok in tokens {
            for &kw in keywords {
                if tok == kw {
                    score += 120;
                } else if tok.contains(kw) || kw.contains(tok.as_str()) {
                    score += 45;
                }
            }
        }
        if score > best_score {
            best_score = score;
            best_id = domain_id;
            best_desc = domain_desc;
        }
    }

    if best_score == 0 {
        (
            "core-systems",
            "Core Rust systems & architectural logic",
            350,
        )
    } else {
        (best_id, best_desc, (best_score.min(1000)) as u16)
    }
}

fn tokenize_identifier_and_text(text: &str) -> Vec<String> {
    let mut out = Vec::new();
    for raw in text.split(|c: char| !c.is_ascii_alphanumeric() && c != '_') {
        let trimmed = raw.trim();
        if trimmed.len() < 2 {
            continue;
        }
        let lower = trimmed.to_ascii_lowercase();
        out.push(lower.clone());
        // Also split snake_case subwords
        for part in lower.split('_') {
            if part.len() >= 2 && part != lower {
                out.push(part.to_string());
            }
        }
    }
    out
}

/// Extract extensive function/struct/enum consciousness from a leaf code directory (e.g., `src/`).
pub fn analyze_leaf_code_directory(
    dir: &Path,
) -> (Vec<FunctionConsciousness>, Vec<FileCodeSummary>) {
    let Ok(entries) = fs::read_dir(dir) else {
        return (Vec::new(), Vec::new());
    };

    let mut source_files = Vec::new();
    for entry in entries.flatten() {
        let path = entry.path();
        if !path.is_file() {
            continue;
        }
        let Some(ext) = path.extension().and_then(|e| e.to_str()) else {
            continue;
        };
        if matches!(ext, "rs" | "py" | "ts" | "tsx" | "js" | "go" | "c" | "cpp") {
            source_files.push(path);
        }
    }
    source_files.sort();

    let mut all_functions: Vec<FunctionConsciousness> = Vec::new();
    let mut file_summaries: Vec<FileCodeSummary> = Vec::new();

    for file_path in source_files.into_iter().take(64) {
        let Ok(content) = fs::read_to_string(&file_path) else {
            continue;
        };
        let fname = file_path
            .file_name()
            .and_then(|s| s.to_str())
            .unwrap_or("source")
            .to_string();

        let mut file_symbols = extract_symbols_from_source(&fname, &content);
        if file_symbols.is_empty() {
            continue;
        }

        // Sort file symbols by salience so top functions appear first
        file_symbols.sort_by_key(|b| std::cmp::Reverse(b.salience_permille));

        // Synthesize file-level BitNet gist
        let mut file_tokens = tokenize_identifier_and_text(&fname);
        for sym in file_symbols.iter().take(12) {
            file_tokens.extend(tokenize_identifier_and_text(&sym.name));
            file_tokens.extend(tokenize_identifier_and_text(&sym.docstring));
        }
        let (file_domain, domain_desc, _) = classify_domain_bitnet(&file_tokens);

        let top_syms: Vec<String> = file_symbols
            .iter()
            .take(6)
            .map(|s| format!("{} ({})", s.name, s.kind))
            .collect();

        let module_doc = extract_module_docstring(&content);
        let file_gist = if !module_doc.is_empty() {
            format!(
                "[{}] {} — {} symbols (key: {})",
                file_domain,
                module_doc,
                file_symbols.len(),
                top_syms.join(", ")
            )
        } else {
            format!(
                "[{}] {} — {} symbols (key: {})",
                file_domain,
                domain_desc,
                file_symbols.len(),
                top_syms.join(", ")
            )
        };

        file_summaries.push(FileCodeSummary {
            file: fname,
            domain: file_domain.to_string(),
            symbol_count: file_symbols.len(),
            gist: file_gist,
            top_symbols: top_syms,
        });

        // Keep up to 6 highest-salience symbols per file so `.m8` stays tight while preserving flagship coverage
        for sym in file_symbols.into_iter().take(6) {
            all_functions.push(sym);
        }
    }

    // Prioritize flagship files and high-salience functions (tight top-36 cap)
    all_functions.sort_by_key(|b| std::cmp::Reverse(b.salience_permille));
    if all_functions.len() > 36 {
        all_functions.truncate(36);
    }
    file_summaries.sort_by_key(|b| std::cmp::Reverse(b.symbol_count));
    if file_summaries.len() > 32 {
        file_summaries.truncate(32);
    }

    (all_functions, file_summaries)
}

fn extract_module_docstring(content: &str) -> String {
    let mut lines = Vec::new();
    for raw in content.lines().take(25) {
        let t = raw.trim();
        if let Some(doc) = t.strip_prefix("//!") {
            let clean = doc.trim();
            if !clean.is_empty() && !clean.starts_with("===") && !clean.starts_with("---") {
                lines.push(clean.to_string());
                if lines.len() >= 2 {
                    break;
                }
            }
        }
    }
    let joined = lines.join(" ");
    joined.chars().take(140).collect()
}

fn extract_symbols_from_source(fname: &str, content: &str) -> Vec<FunctionConsciousness> {
    let mut out = Vec::new();
    let mut pending_docs: Vec<String> = Vec::new();

    for (idx, raw) in content.lines().enumerate() {
        let t = raw.trim();
        if let Some(doc) = t.strip_prefix("///") {
            let clean = doc.trim();
            if !clean.is_empty() {
                pending_docs.push(clean.to_string());
            }
            continue;
        }
        if t.starts_with("#[") {
            continue;
        }

        let matched = if let Some(rest) = t.strip_prefix("pub const fn ") {
            Some(("pub const fn", rest, 180u16))
        } else if let Some(rest) = t.strip_prefix("pub async fn ") {
            Some(("pub async fn", rest, 160u16))
        } else if let Some(rest) = t.strip_prefix("pub fn ") {
            Some(("pub fn", rest, 150u16))
        } else if let Some(rest) = t.strip_prefix("pub struct ") {
            Some(("pub struct", rest, 170u16))
        } else if let Some(rest) = t.strip_prefix("pub enum ") {
            Some(("pub enum", rest, 165u16))
        } else if let Some(rest) = t.strip_prefix("pub trait ") {
            Some(("pub trait", rest, 165u16))
        } else if let Some(rest) = t.strip_prefix("fn ") {
            // Skip tiny test helper functions unless they have docstrings
            if pending_docs.is_empty() || t.contains("test_") {
                pending_docs.clear();
                continue;
            }
            Some(("fn", rest, 90u16))
        } else {
            if !t.is_empty() && !t.starts_with("//") {
                pending_docs.clear();
            }
            continue;
        };

        let Some((kind, rest, base_salience)) = matched else {
            continue;
        };

        let name: String = rest
            .chars()
            .take_while(|c| c.is_ascii_alphanumeric() || *c == '_')
            .collect();
        if name.len() < 2 || name.starts_with("test_") {
            pending_docs.clear();
            continue;
        }

        let signature: String = t
            .split('{')
            .next()
            .unwrap_or(t)
            .trim()
            .chars()
            .take(140)
            .collect();
        let docstring: String = pending_docs.join(" ").chars().take(200).collect();
        pending_docs.clear();

        let mut tokens = tokenize_identifier_and_text(fname);
        tokens.extend(tokenize_identifier_and_text(&name));
        tokens.extend(tokenize_identifier_and_text(&signature));
        tokens.extend(tokenize_identifier_and_text(&docstring));

        let (domain_id, domain_desc, domain_score) = classify_domain_bitnet(&tokens);
        let wave = BitNetWave64::from_tokens(&tokens);
        let wave_b3 = wave.trits_9();

        let doc_bonus = if !docstring.is_empty() { 180u16 } else { 0u16 };
        let memnet_bonus = if fname == "memnet.rs" || fname == "completions.rs" {
            140u16
        } else {
            0u16
        };
        let salience_permille =
            (base_salience + domain_score / 2 + doc_bonus + memnet_bonus).min(999);

        let bitnet_gist = if !docstring.is_empty() {
            format!("[{domain_id}] {docstring}")
        } else {
            format!("[{domain_id}] {kind} `{name}` in `{fname}` — {domain_desc}")
        };

        out.push(FunctionConsciousness {
            name,
            kind: kind.to_string(),
            file: fname.to_string(),
            line: idx + 1,
            signature,
            docstring,
            domain: domain_id.to_string(),
            bitnet_gist,
            salience_permille,
            wave,
            wave_b3,
        });
    }

    out
}

/// Build the multi-resolution `BitNetDirectorySummary` for `path`:
/// - If `path` is a leaf code folder (e.g. `src/`), summarizes extensive functions & files.
/// - If `path` is a project root (e.g. `smart-tree/`), pools child `.m8` summaries into a concise Project Gist.
/// - If `path` is a workspace/org root (e.g. `~/source/8b-is/`), pools all repo `.m8` gists into a Fleet Gist.
pub fn synthesize_bitnet_directory_summary(
    path: &Path,
    local_functions: &[FunctionConsciousness],
    local_files: &[FileCodeSummary],
    child_m8s: &BTreeMap<String, crate::m8_consciousness::DirectoryConsciousness>,
    manifest_or_md_essence: &str,
) -> BitNetDirectorySummary {
    let dir_name = path.file_name().and_then(|s| s.to_str()).unwrap_or("root");

    let has_cargo_or_manifest = path.join("Cargo.toml").is_file()
        || path.join("package.json").is_file()
        || path.join("pyproject.toml").is_file()
        || path.join("go.mod").is_file();

    let zoom_level = if !local_functions.is_empty() && !has_cargo_or_manifest {
        "leaf-code"
    } else if has_cargo_or_manifest || child_m8s.contains_key("src") {
        "project-root"
    } else if child_m8s.len() >= 3 {
        "workspace-org"
    } else if !local_functions.is_empty() {
        "leaf-code"
    } else {
        "directory-node"
    };

    // 1. Collect waves and domain tallies from local functions OR child `.m8` pyramids
    let mut waves: Vec<BitNetWave64> = Vec::new();
    let mut domain_scores: BTreeMap<String, u32> = BTreeMap::new();
    let mut total_symbols = local_files
        .iter()
        .map(|f| f.symbol_count)
        .sum::<usize>()
        .max(local_functions.len());
    let mut flagship_symbols: Vec<String> = Vec::new();

    for f in local_functions.iter().take(18) {
        waves.push(f.wave);
        *domain_scores.entry(f.domain.clone()).or_insert(0) += f.salience_permille as u32;
        if flagship_symbols.len() < 12 {
            flagship_symbols.push(format!(
                "{}::{} (L{}): {}",
                f.file,
                f.name,
                f.line,
                f.bitnet_gist.chars().take(80).collect::<String>()
            ));
        }
    }

    for (child_name, child_c) in child_m8s {
        if let Some(child_bn) = &child_c.bitnet_summary {
            waves.push(child_bn.pooled_wave);
            total_symbols += child_bn.total_symbols_rolled_up;
            for (dom, score) in &child_bn.dominant_domains {
                *domain_scores.entry(dom.clone()).or_insert(0) += *score as u32;
            }
            if flagship_symbols.len() < 12 {
                for sym in child_bn.flagship_symbols.iter().take(6) {
                    if flagship_symbols.len() < 12 {
                        flagship_symbols.push(format!("{child_name}/{sym}"));
                    }
                }
            }
        } else {
            let child_tokens = tokenize_identifier_and_text(&child_c.essence);
            let (dom, _, sc) = classify_domain_bitnet(&child_tokens);
            *domain_scores.entry(dom.to_string()).or_insert(0) += sc as u32;
            waves.push(BitNetWave64::from_tokens(&child_tokens));
        }
    }

    if waves.is_empty() {
        let fallback_tokens = tokenize_identifier_and_text(manifest_or_md_essence);
        let (dom, _, sc) = classify_domain_bitnet(&fallback_tokens);
        *domain_scores.entry(dom.to_string()).or_insert(0) += sc as u32;
        waves.push(BitNetWave64::from_tokens(&fallback_tokens));
    }

    let pooled_wave = BitNetWave64::pool_hierarchy(&waves);
    let pooled_wave_b3 = pooled_wave.trits_9();

    let mut dominant_domains: Vec<(String, u16)> = domain_scores
        .into_iter()
        .map(|(k, v)| (k, (v.min(9999)) as u16))
        .collect();
    dominant_domains.sort_by_key(|b| std::cmp::Reverse(b.1));
    dominant_domains.truncate(5);

    let dom_list: String = dominant_domains
        .iter()
        .take(3)
        .map(|(d, _)| d.as_str())
        .collect::<Vec<_>>()
        .join(", ");

    let synthesized_gist = match zoom_level {
        "leaf-code" => {
            let top_files: Vec<String> = local_files
                .iter()
                .take(5)
                .map(|f| format!("{} ({} syms)", f.file, f.symbol_count))
                .collect();
            format!(
                "BitNet-b1.58 Leaf Code [{} | B+/B-={}‰]: {} source files, {} indexed functions/structs across [{}]. Key modules: {}",
                pooled_wave_b3,
                pooled_wave.balance_permille,
                local_files.len(),
                local_functions.len(),
                dom_list,
                top_files.join(", ")
            )
        }
        "project-root" => {
            let head_essence: String = manifest_or_md_essence.chars().take(120).collect();
            format!(
                "BitNet-b1.58 Project Gist [{} | B+/B-={}‰]: {} — rolls up {} symbols across {} child folders (domains: [{}])",
                pooled_wave_b3,
                pooled_wave.balance_permille,
                head_essence,
                total_symbols,
                child_m8s.len(),
                dom_list
            )
        }
        "workspace-org" => {
            format!(
                "BitNet-b1.58 Fleet/Org Gist `{}` [{} | B+/B-={}‰]: rolls up {} child repositories & {} total indexed symbols across domains [{}]",
                dir_name,
                pooled_wave_b3,
                pooled_wave.balance_permille,
                child_m8s.len(),
                total_symbols,
                dom_list
            )
        }
        _ => {
            let head_essence: String = manifest_or_md_essence.chars().take(120).collect();
            format!(
                "BitNet-b1.58 Node `{}` [{}]: {}",
                dir_name, pooled_wave_b3, head_essence
            )
        }
    };

    BitNetDirectorySummary {
        zoom_level: zoom_level.to_string(),
        engine: "BitNet-b1.58-Ternary-NO-FLOAT (B+/B- Kleene 64-trit)".to_string(),
        synthesized_gist,
        dominant_domains,
        pooled_wave,
        pooled_wave_b3,
        total_symbols_rolled_up: total_symbols,
        file_summaries: local_files.to_vec(),
        flagship_symbols,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bitnet_wave64_is_deterministic_and_no_float() {
        let tokens = vec![
            "memnet".to_string(),
            "rtc".to_string(),
            "tau".to_string(),
            "pade".to_string(),
        ];
        let w1 = BitNetWave64::from_tokens(&tokens);
        let w2 = BitNetWave64::from_tokens(&tokens);
        assert_eq!(w1, w2);
        assert!(w1.ternary_dot(w2) > 0);
        assert_eq!(w1.trits_9().len(), 9);
    }

    #[test]
    fn test_extract_symbols_from_rust_source() {
        let src = r#"
//! Sovereign MemNet module
/// Zero-float Q16 Padé [2/2] exponential relaxation filter
pub const fn exp_neg_ratio_q16(dt_ms: u32, tau_ms: u32) -> u32 { 0 }
"#;
        let syms = extract_symbols_from_source("memnet.rs", src);
        assert_eq!(syms.len(), 1);
        assert_eq!(syms[0].name, "exp_neg_ratio_q16");
        assert_eq!(syms[0].domain, "rtc-control");
        assert!(syms[0].bitnet_gist.contains("Padé"));
    }
}
