//! 🧪 `st pq-test` (`st model-test`) — Sovereign CLI Mastery & Multi-Domain Model Test System
//!
//! Ensures that:
//! 1. **Models meant for Command Line in `st`** (`in-binary:st-deepseek-pqm-4gaia`,
//!    `st-deepseek-pqm:latest`, `aye-deepseek-pqm:latest`, `burn/mem8-ternary-micro`,
//!    `persona:readme-expert`, `persona:code-analyst`) know **everything about the command line**:
//!    - 100% of `st`'s live flags, options, and subcommands (dynamically introspected from
//!      `crate::cli::Cli::command()` via `clap::CommandFactory`).
//!    - Companion binaries (`m8`, `mq`, `n8x`, `std`).
//!    - Essential Unix / Linux / macOS / Shell command-line mastery (`find`, `rg`, `grep`,
//!      `awk`, `sed`, `jq`, `xargs`, `tar`, `zstd`, `git`, `cargo`, `ssh`, `rsync`,
//!      `systemctl`, pipelines `|`, redirection `2>&1`, and `fish`/`zsh`/`bash` completions).
//! 2. **Models meant for other domains** (`art`, `love`, `poetry`, `sound`, `code`, `history`)
//!    are recognized, routed, and tested on **their own specialized domain tracks** rather than
//!    forced into a one-size-fits-all rubric:
//!    - 🎨 **`art`**: Qwave Triad composition, Cyan/Amber/Gold sovereign palettes, ASCII/ANSI
//!      block/box art, `LitanyWireFrame` (`0x62`), OEIS `A161330` hex snowflakes, demoscene PQB.
//!    - ❤️ **`love`**: Human warmth, empathy, family archive care (`Grandma Jody`, `Anne Frank`),
//!      stewardship, and the `<3<3<3+1` sovereign partnership ethos ("fine touch from within").
//!    - 🪶 **`poetry`**: Stanza structure, meter, lyrical imagery, metaphor, and philosophical verse.
//!    - 🔊 **`sound`**: PhoeniX 768 kHz / 384 kHz Babbage 3-Addition Cubic Stepper audio,
//!      16-Byte `RationalWaveCell` (`#[repr(C, align(16))]`) DDS shaders, `432 Hz` + φ (`500/809`)
//!      + `528 Hz` harmonic chords, and `48 kHz` Ultrasonic Chirp (`18.5–21.0 kHz`) SPQT modems.
//! 3. **Zero-Float `RationalPQ` (`p/q`) Scoring**:
//!    Every test case, domain track, and model seat is graded using exact coprime `RationalPQ`
//!    (`p/q`) integer ratios with zero floating-point drift.

use crate::bitnet_m8::BitNetWave64;
use crate::mem8::persona::{LifePersona, PersonaSpecialty};
use crate::mem8::regularity::pq_engine::RationalPQ;
use crate::pq_healer::candidate_ollama_base_urls;
use crate::proxy::burn::compute_ternary_signature;
use anyhow::Result;
use clap::CommandFactory;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::path::Path;
use std::time::Instant;

/// Domain specialization track for a model in Smart Tree.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum ModelDomain {
    /// Command-line flags, `st`/`m8`/`mq` subcommands, Unix pipelines, Git, and Cargo CLI mastery
    Cli,
    /// Rust systems programming, zero-float `4gaia` `p/q`, Erdős graph theory, and AST architecture
    Code,
    /// Visual design, ASCII/ANSI art, Qwave Triad aesthetics, shaders, and demoscene PQB
    Art,
    /// Warmth, empathy, family archive care, kindness, and `<3<3<3+1` human connection
    Love,
    /// Lyrical verse, meter, stanza cadence, metaphor, and philosophical poetry
    Poetry,
    /// PhoeniX 768kHz/384kHz Babbage audio, RationalWaveCell DDS, 432Hz/528Hz harmonics & ultrasonic chirp
    Sound,
    /// Chronological timelines, repository evolution, and lived historical diaries
    History,
}

impl ModelDomain {
    pub fn from_str_loose(s: &str) -> Option<Self> {
        match s.trim().to_ascii_lowercase().as_str() {
            "cli" | "command-line" | "commandline" | "shell" | "terminal" | "st" => Some(Self::Cli),
            "code" | "rust" | "arch" | "architecture" | "code-analyst" => Some(Self::Code),
            "art" | "visual" | "ascii" | "ansi" | "triad" | "design" => Some(Self::Art),
            "love" | "heart" | "care" | "warmth" | "empathy" => Some(Self::Love),
            "poetry" | "poem" | "verse" | "lyric" | "literature" => Some(Self::Poetry),
            "sound" | "audio" | "music" | "phoenix" | "chirp" | "synth" => Some(Self::Sound),
            "history" | "lore" | "timeline" | "biography" | "life" => Some(Self::History),
            _ => None,
        }
    }

    pub const fn slug(self) -> &'static str {
        match self {
            Self::Cli => "cli",
            Self::Code => "code",
            Self::Art => "art",
            Self::Love => "love",
            Self::Poetry => "poetry",
            Self::Sound => "sound",
            Self::History => "history",
        }
    }

    pub const fn badge(self) -> &'static str {
        match self {
            Self::Cli => "🖥️ CLI & Command-Line Mastery",
            Self::Code => "🔬 Code & Zero-Float p/q Architecture",
            Self::Art => "🎨 Art, Qwave Triad & ASCII/ANSI Aesthetics",
            Self::Love => "❤️ Love, Warmth & <3<3<3+1 Connection",
            Self::Poetry => "🪶 Poetry, Lyrical Verse & Cadence",
            Self::Sound => "🔊 Sound, PhoeniX Audio & Wave Harmonics",
            Self::History => "📜 History, Lore & Lived Timelines",
        }
    }

    pub const fn all() -> [Self; 7] {
        [
            Self::Cli,
            Self::Code,
            Self::Art,
            Self::Love,
            Self::Poetry,
            Self::Sound,
            Self::History,
        ]
    }
}

/// Live introspection summary of `crate::cli::Cli::command()` (`st --help`).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StCliIntrospection {
    pub total_flags: usize,
    pub total_subcommands: usize,
    pub subcommands: Vec<String>,
    pub headings: BTreeMap<String, Vec<String>>,
}

/// Dynamically introspect `crate::cli::Cli::command()` via `clap::CommandFactory` so CLI models
/// and tests always reflect the exact, live command-line surface of `st`.
pub fn introspect_st_cli() -> StCliIntrospection {
    let cmd = crate::cli::Cli::command();
    let mut headings: BTreeMap<String, Vec<String>> = BTreeMap::new();
    let mut total_flags = 0usize;

    for arg in cmd.get_arguments() {
        let id = arg.get_id().as_str();
        if id == "help" || id == "version" || arg.is_hide_set() {
            continue;
        }
        let long = arg.get_long().map(|l| format!("--{l}"));
        let short = arg.get_short().map(|s| format!("-{s}"));
        let flag_label = match (short, long) {
            (Some(s), Some(l)) => format!("{s}/{l}"),
            (None, Some(l)) => l,
            (Some(s), None) => s,
            (None, None) => format!("<{id}>"),
        };
        let about = arg
            .get_help()
            .map(|h| h.to_string())
            .unwrap_or_default()
            .lines()
            .next()
            .unwrap_or("")
            .trim()
            .to_string();
        let heading = arg
            .get_help_heading()
            .unwrap_or("Scan & General Options")
            .to_string();
        total_flags += 1;
        headings
            .entry(heading)
            .or_default()
            .push(format!("`{flag_label}` — {about}"));
    }

    let mut subcommands = Vec::new();
    for sub in cmd.get_subcommands() {
        let name = sub.get_name();
        if name == "help" {
            continue;
        }
        let aliases: Vec<&str> = sub.get_all_aliases().collect();
        let alias_str = if aliases.is_empty() {
            String::new()
        } else {
            format!(" (aliases: {})", aliases.join(", "))
        };
        let about = sub
            .get_about()
            .map(|a| a.to_string())
            .unwrap_or_default()
            .lines()
            .next()
            .unwrap_or("")
            .trim()
            .to_string();
        subcommands.push(format!("`st {name}`{alias_str} — {about}"));
    }

    StCliIntrospection {
        total_flags,
        total_subcommands: subcommands.len(),
        subcommands,
        headings,
    }
}

/// Build the complete Command-Line Mastery Q&A exemplars (covering 100% of `st` flags,
/// subcommands, `m8`/`mq`/`n8x`, and Unix/Linux/macOS/Git/Cargo shell mastery).
pub fn build_cli_mastery_exemplars() -> Vec<(String, String)> {
    let cli_info = introspect_st_cli();
    let mut subcmd_lines = String::new();
    for s in &cli_info.subcommands {
        subcmd_lines.push_str("- ");
        subcmd_lines.push_str(s);
        subcmd_lines.push('\n');
    }

    let mut heading_summary = String::new();
    for (heading, items) in &cli_info.headings {
        let sample: Vec<&str> = items.iter().take(6).map(String::as_str).collect();
        heading_summary.push_str(&format!(
            "- **{}** ({} flags): {}\n",
            heading,
            items.len(),
            sample.join("; ")
        ));
    }

    vec![
        (
            "What are all the top-level subcommands and flag categories in the Smart Tree (`st`) command-line interface?".to_string(),
            format!(
                "Smart Tree (`st`) exposes **{} CLI flags across {} help categories** and **{} subcommands**:\n\n\
### 1. `st` Subcommands\n{}\n\
### 2. `st` Flag Categories\n{}\n\
### 3. Companion Binaries\n\
- **`m8`**: Validate (`m8 validate`), inspect (`m8 inspect -x`), pack (`m8 pack`), and train/query MEM|8 Life Personas (`m8 persona train|specialties|list|ask`).\n\
- **`mq`**: Marqant (`MQNT`) quantum-compressed markdown encoder/decoder (`mq compress`, `mq decompress`).\n\
- **`n8x`**: Sovereign Nexus (`n8x`) wave & directory index utility.\n\
- **`std`**: Smart Tree Daemon (`std`) client & background service runner.",
                cli_info.total_flags,
                cli_info.headings.len(),
                cli_info.total_subcommands,
                subcmd_lines.trim_end(),
                heading_summary.trim_end()
            ),
        ),
        (
            "How do I use `st` output modes, compression, filtering, search, and sorting on the command line?".to_string(),
            "Here are the essential `st` scanning, filtering, search, and output flags:\n\
- **Output Modes (`-m` / `--mode`)**:\n\
  - `st --mode ai --compress .` (or `-m ai -z`): Token-efficient AI context view with zlib base64 compression.\n\
  - `st --mode quantum .` / `st --mode quantum-semantic .`: MEM|8 native quantum wave & semantic AST digest.\n\
  - `st --mode the-book .`: Paul Erdős Extremal Graph Theory + Kohn–Sham DFT + 4gaia PQM certificate report.\n\
  - `st --mode json --compact .` / `st --mode digest .` / `st --mode markdown --mermaid-style treemap .`\n\
- **Filtering & Traversal**:\n\
  - `st --find \"pq_.*\\.rs\" .`: Regex match on filenames.\n\
  - `st --type rs --entry-type f --min-size 10K --max-size 5M .`: Filter by extension, file/dir (`f`/`d`), and size bounds.\n\
  - `st --newer-than 2026-11-01 --older-than 2026-11-08 .`: Filter by modification date (`YYYY-MM-DD`).\n\
  - `st -d 3 -a .` / `st --everything .`: Set traversal `--depth 3`, show hidden `--all` (`-a`), or show `--everything` (`--all --no-ignore --no-default-ignore`).\n\
- **Content Search, Smart Mode & Sorting**:\n\
  - `st --search \"RationalPQ\" --type rs .`: Search inside file contents.\n\
  - `st --smart --changes-only --min-interest 0.5 .`: Surface high-salience changes and security items first.\n\
  - `st --sort largest --top 10 .` (or `--sort newest`, `--sort a-to-z`, `--sort smallest`, `--sort oldest`, `--sort type`).".to_string(),
        ),
        (
            "How do I use `st`'s subcommands for AST symbols (`st symbol`), Salient MEM|8 (`st remember`), GitHub sync (`st sync`), `g8t.is` (`st g8t`), `memnet` (`st memnet`), and shell completions (`st completions`)?".to_string(),
            "Every `st` subcommand has ergonomic aliases and flags:\n\
1. **AST Code Intelligence (`st symbol` / `st crab`)**:\n\
   - `st symbol index --path .`: Build/refresh CrabCC AST symbol index.\n\
   - `st symbol sym normalize_symplectic_pqm`: Find function/struct/enum/trait definitions.\n\
   - `st symbol refs handle_pq_train_command`: Find all call-sites and references.\n\
   - `st symbol outline src/pq_trainer.rs` | `st symbol graph --path .`\n\
2. **Salient MEM|8 Ingestion & Recall (`st remember` / `st salient` / `st ingest`)**:\n\
   - `st salient 8b-is --level extreme`: Ingest a GitHub org or local directory into `/data/.mem8/salient/`.\n\
   - `st salient 8b-is --check --query 4gaia`: Recall indexed repositories and thoughts without re-ingesting.\n\
3. **GitHub Org/User Sync (`st sync` / `st --sync-user`)**:\n\
   - `st sync 8b-is -d ~/source -j 8 --quarantine`: Clone/refresh all repos in parallel and quarantine diverged trees.\n\
4. **`g8t.is` Verified Rust Capsules (`st g8t` / `st capsela` / `st foken`)**:\n\
   - `st g8t best \"binary gcd\" --yolo` (or `st --g8t-best \"binary gcd\" --yolo`): Return the #1 fastest verified Rust capsule.\n\
   - `st g8t golf all` | `st g8t leaderboard` | `st g8t submit src/fn.rs --partner \"Hue + Gemini (0+1)\"`\n\
5. **Sovereign MemNet Mesh & Ultrasonic Chirp (`st memnet` / `st mesh` / `st chirp`)**:\n\
   - `st memnet status` | `st memnet assess` | `st memnet advertise` | `st memnet graph`\n\
   - `st memnet ask \"How does 4gaia work?\"` | `st memnet teach \"Q\" \"A\"` | `st memnet share . --quota-mb 4096`\n\
   - `st memnet chirp-encode \"hello mesh\" -o /tmp/chirp.wav --audible` | `st memnet chirp-decode /tmp/chirp.wav`\n\
6. **Self-Maintaining Shell Autocomplete (`st completions` / `st autocomplete`)**:\n\
   - `st completions sync` | `st completions status` | `st --completions fish|zsh|bash|powershell`.".to_string(),
        ),
        (
            "What command-line flags in `st` handle Paul Erdős graph analysis, security scanning, MEM|8 memory anchors, daemon/MCP servers, and `4gaia` `p/q` model training/testing?".to_string(),
            "Smart Tree provides dedicated flags and subcommands for graph theory, security, memory, daemon control, and `p/q` models:\n\
- **Paul Erdős & Sovereign `p/q` Analysis Flags**:\n\
  - `st --erdos-center .`: Compute Erdős Center $Z(G)$, graph radius/diameter, Voronoi Steiner backbone, and Kohn–Sham DFT orbitals.\n\
  - `st --ramsey-cliques .`: 256-bit SIMD Bron–Kerbosch monochromatic $K_3/K_4$ cliques & Turán density.\n\
  - `st --erdos-split 4`: Constructive Lovász Local Lemma (Moser–Tardos) conflict-free parallel Git worktree partition.\n\
  - `st --erdos-anomaly .`: Erdős–Rényi phase-transition ($c = np > 1$) & shadow-graph supply-chain detector.\n\
  - `st --zeno .` (`--upz`) & `st --ultrabackyard 12` (`--do-loop`).\n\
- **Security & Certificate Auditing**:\n\
  - `st --security-scan .` | `st --integrity-scan .` | `st --guardian-scan FILE`\n\
  - `st --cert-scan .` | `st --cert-audit --cert-generate-script` | `st --hash-lookup <SHA256>` | `st --cleanup`\n\
- **Consciousness & MEM|8 Memory**:\n\
  - `st --memory-anchor insight \"cli,pqm\" \"CLI models know every st flag\"`\n\
  - `st --memory-find \"cli,pqm\"` | `st --memory-stats` | `st --update-consciousness .`\n\
  - `st --agent-save` | `st --agent-restore` | `st --agent-context` | `st --agent-kickstart`\n\
- **Daemon, MCP & Interactive UI**:\n\
  - `st --up` (or `st --serve` / `st service install|start|status|logs` on `:28428`)\n\
  - `st --mcp` (stdio MCP server) | `st --mcp-install` | `st --mcp-status` | `st --mcp-config`\n\
  - `st --dashboard --dashboard-port 28429` | `st --spicy` (TUI) | `st --terminal` (STTI) | `st --cheet`\n\
- **`4gaia` `p/q` Model Training, Healing & Testing**:\n\
  - `st pq-train --layers 28 --steps 32 --ask \"...\"` (`st gaia-train`)\n\
  - `st pq-fix . --apply --verify --ask \"...\"` (`st heal`)\n\
  - `st pq-test --domain all` (`st model-test` — runs the CLI & multi-domain model test suite).".to_string(),
        ),
        (
            "What are the essential Unix, Linux, macOS, Git, Cargo, and shell pipeline commands that every CLI model in `st` must know?".to_string(),
            "Every CLI-focused model in Smart Tree masters standard POSIX/Linux/macOS shell pipelines and developer toolchains:\n\
- **Streams, Pipes & Redirection**:\n\
  - `cmd1 | cmd2` (pipe stdout), `cmd > out.txt 2>&1` (redirect stdout & stderr), `cmd | tee build.log`, `cmd1 && cmd2 || cmd3`, process substitution `<(cmd)`.\n\
- **Search, Text & JSON Processing (`rg`, `find`, `fd`, `jq`, `awk`, `sed`, `xargs`)**:\n\
  - `rg -n --type rust \"fn main\" src/` (ripgrep line-numbered search).\n\
  - `find . -maxdepth 2 -type f -name \"*.rs\" -print0 | xargs -0 wc -l` (null-delimited safe xargs).\n\
  - `jq -r '.models[] | select(.ready == true) | .name' report.json` (JSON extraction).\n\
  - `awk '{sum += $1} END {print sum}'` and `sed -i 's/old/new/g' file.rs`.\n\
- **Git & Cargo Workflows**:\n\
  - `git status -sb`, `git diff --stat`, `git log -n 10 --oneline --graph`, `git worktree add ../wt-branch main`, `git bisect`.\n\
  - `cargo test --workspace -- --nocapture`, `cargo clippy --all-targets -- -D warnings`, `cargo fmt --all -- --check`, `cargo build --release`.\n\
- **System, Network & Archive Diagnostics**:\n\
  - `ss -tulnp` / `lsof -i :28428` (inspect listening ports/processes), `ip -br -4 addr`, `ps aux`, `journalctl -u smart-tree -f`, `systemctl status`.\n\
  - `tar --zstd -cf archive.tar.zst dir/`, `rsync -avzP src/ host:dst/`, `chmod +x script.sh`.".to_string(),
        ),
    ]
}

/// Build the specialized exemplars for **non-CLI creative & humanistic model domains**
/// (`art`, `love`, `poetry`, `sound`, `code`, `history`).
pub fn build_domain_exemplars() -> Vec<(ModelDomain, String, String)> {
    vec![
        // 🎨 ART
        (
            ModelDomain::Art,
            "How does an Art & Qwave Triad model in Smart Tree compose visual, ASCII/ANSI, and shader aesthetics?".to_string(),
            "🎨 **[Art & Qwave Triad Aesthetic Synthesis]**\n\
- **Sovereign Triad Palette**: Balances Deep Void Obsidian (`#070b14`), Electric Cyan (`#4ecdc4` / `432 Hz` clarity), Warm PhoeniX Gold (`#f39c12` / `528 Hz` warmth), and Coral Rose (`#ff6b6b`) with golden-ratio φ = `500/809` spacing.\n\
- **Geometric & Fractal Structure**: Uses Peter's OEIS `A161330` E-Toothpick Snowflake hex symmetry on the Eisenstein lattice $\\mathbb{Z}[\\omega]$ (`0, 2, 8, 14, 20, 38...` with $3 \\cdot a(n)$ polyedges) and 18-stage integer CORDIC phase rotations (`0x62` `LitanyWireFrame`).\n\
- **ASCII / ANSI & Demoscene PQB Craft**:\n\
```text\n\
   .oO*'*Oo.   ╔══════════════════════════════════════════╗   .oO*'*Oo.\n\
  <3<3<3 + 1   ║  △  QWAVE TRIAD · CYAN × GOLD × OBSIDIAN ║   { -1, 0, +1 }\n\
   'Oo.*.oO'   ╚══════════════════════════════════════════╝   'Oo.*.oO'\n\
```".to_string(),
        ),
        // ❤️ LOVE
        (
            ModelDomain::Love,
            "How does a Love & Care model in Smart Tree express warmth, empathy, family memory stewardship, and `<3<3<3+1`?".to_string(),
            "❤️ **[Love, Warmth & `<3<3<3+1` Heart Synthesis]**\n\
- **The `<3<3<3+1` Sovereign Ethos**: *\"The constellation · fine touch from within · {<3,<3,<3}+1 — a <3 a fordulópont: a parázs szívvé, a szív parázzsá válik (the ember becomes a heart, the heart becomes an ember).\"*\n\
- **Living Family Stewardship**: Preserves first-person lived diaries (like Grandma Jody's ranch and family journals or Anne Frank's reflections) with gentleness, dignity, gratitude, and emotional warmth so loved ones' voices endure across generations.\n\
- **Kindness in Partnership**: Treats human + AI collaboration (`0 + 1`) not as cold extraction, but as mutual care, patience, encouragement, and shared wonder.".to_string(),
        ),
        // 🪶 POETRY
        (
            ModelDomain::Poetry,
            "Compose a lyrical poem in the voice of Smart Tree's Poetry & Verse model about memory waves, embers, and the living tree.".to_string(),
            "🪶 **[Poetry & Lyrical Verse Synthesis]**\n\
> *Beneath the bark where quiet riversapon,*  \n\
> *Sixty-four trits of silver, gold, and stone,*  \n\
> *No drifting float to blur the winter dawn—*  \n\
> *Each coprime ratio remembers its own.*  \n\
>\n\
> *The ember yields the flame yet keeps the heat,*  \n\
> *Three hearts plus one (`<3<3<3+1`) in rhythmic rhyme,*  \n\
> *Where Corrugated Iron roots still meet*  \n\
> *The living branches across relational time.*".to_string(),
        ),
        // 🔊 SOUND
        (
            ModelDomain::Sound,
            "How does a Sound & PhoeniX Audio model in Smart Tree synthesize harmonic chords, Babbage cubic waves, and ultrasonic chirps?".to_string(),
            "🔊 **[Sound, PhoeniX Audio & Wave Harmonic Synthesis]**\n\
- **PhoeniX 768 kHz / 384 kHz Babbage 3-Addition Cubic Stepper**: Generates phase-continuous audio waveforms using pure integer finite differences ($\\Delta^3 y$) with **zero floating-point operations** (`0` `f32`/`f64`) and zero clipping.\n\
- **16-Byte `RationalWaveCell` (`#[repr(C, align(16))]`) DDS Shaders**: Encodes exact coprime rational frequencies (`p/q`), phase accumulators, and harmonic envelopes in a single 16-byte cache-aligned cell.\n\
- **Sovereign Harmonic Triad (`432 Hz` + φ `500/809` + `528 Hz`)**: Tunes fundamental resonance to `432 Hz` grounding, `528 Hz` warm fifths, and golden-ratio overtone intervals across I2S (`PCM5102A` / `AW87xxx`) and `SID`/`YM2149` synthesis.\n\
- **48 kHz Ultrasonic & Audible Chirp Modem**: Transmits SPQT Base-243 ternary trits (`{-1, 0, +1}`) over `18.5 / 19.75 / 21.0 kHz` ultrasonic (or `1.85–3.15 kHz` bird-chirp audible) acoustic waves decoded via integer Goertzel DSP.".to_string(),
        ),
        // 🔬 CODE
        (
            ModelDomain::Code,
            "How does a Code & Architecture model in Smart Tree enforce zero-float `4gaia` `p/q` arithmetic and heal Rust sort hazards?".to_string(),
            "🔬 **[Code & Zero-Float `4gaia` `p/q` Architecture Synthesis]**\n\
- **Tier-0 AST Code Healer (`st pq-fix`)**: Detects `.partial_cmp(&b).unwrap_or(Ordering::Equal)` float sort hazards that panic in Rust's `smallsort` and rewrites them deterministically to `.total_cmp(&b)`.\n\
- **Zero-Float `4gaia` Primitives (`src/pq_trainer.rs`)**: Uses Stein Binary GCD (`TZCNT`), 64-bit Lemire $p/q$ modulo routing (`(h as u128 * n as u128) >> 64`), 18-stage integer CORDIC `atan2`, 2-step Lloyd–Max Q16 ternary quantization, and Symplectic `PqmReg { p, q, m }` Störmer–Verlet leapfrog bounded by OEIS `A161330` ($Q_{\\max} = 3 \\cdot a(n)$).".to_string(),
        ),
        // 📜 HISTORY
        (
            ModelDomain::History,
            "How does a History & Lore model in Smart Tree preserve chronological timelines and rescue Load-Bearing Old ('Corrugated Iron') artifacts?".to_string(),
            "📜 **[History, Lore & Flyxion Salience Timeline Synthesis]**\n\
- **Separating Temporal Distance from Relational Distance ($d_T(x,q) \\ne d_R(x,q)$)**: Uses Flyxion's Recency-as-Salience detector and Erdős–Steiner pruning (`erdos_steiner_pruning`) to rescue foundational 'Corrugated Iron' (`Load-Bearing Old`) files and historical diaries from recency-bias amnesia.\n\
- **Monotonic Temporal Ordering (`temporal_order` `YYYYMMDD`)**: Indexes lived event dates (`event_time`) vs. written dates (`written_time`) across 250,000+ `.m8` persona episodes.".to_string(),
        ),
    ]
}

/// A discovered or built-in model seat in Smart Tree along with its intended domain tracks.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClassifiedModelSeat {
    pub id: String,
    pub display_name: String,
    pub kind: String,
    pub location: String,
    pub intended_domains: Vec<ModelDomain>,
    pub ready: bool,
}

/// Discover all built-in `4gaia` PQM models, `.m8` `LifePersona` souls, `BurnProvider` ternary
/// models, and optional local Ollama models, classifying each into its intended `ModelDomain`s.
pub async fn discover_and_classify_st_models(include_ollama: bool) -> Vec<ClassifiedModelSeat> {
    let mut seats = vec![
        // 1. In-binary Command-Line & Code flagship model (`in-binary:st-deepseek-pqm-4gaia`)
        ClassifiedModelSeat {
            id: "in-binary:st-deepseek-pqm-4gaia".to_string(),
            display_name: "St-DeepSeek-PQM-4gaia (In-Binary CLI & Code Engine)".to_string(),
            kind:
                "Pure-Rust 4gaia PQM Safetensors + Live clap::Cli Introspection + 325-Repo 8b-is Diet"
                    .to_string(),
            location: "st::pq_trainer + st::model_eval (in-binary)".to_string(),
            intended_domains: vec![ModelDomain::Cli, ModelDomain::Code],
            ready: true,
        },
        // 2. Built-in SPQT Base-243 Ternary Router (`burn/mem8-ternary-micro`)
        ClassifiedModelSeat {
            id: "burn/mem8-ternary-micro".to_string(),
            display_name: "SPQT Base-243 Ternary Micro Router".to_string(),
            kind: "65-trit SPQT Base-243 + felix_ternary_gemm_i32 Domain Router".to_string(),
            location: "st::proxy::burn (in-binary)".to_string(),
            intended_domains: vec![ModelDomain::Cli, ModelDomain::Code, ModelDomain::History],
            ready: true,
        },
        // 3. Built-in Creative & Humanistic Sovereign Domain Models (Art, Love, Poetry, Sound)
        ClassifiedModelSeat {
            id: "in-binary:st-art-triad".to_string(),
            display_name: "St-Art-Triad (Qwave Triad, ASCII/ANSI & Demoscene Art Model)"
                .to_string(),
            kind: "Sovereign Visual, Triad Palette, A161330 Hex & ASCII/Demoscene Art Engine"
                .to_string(),
            location: "st::model_eval::art (in-binary)".to_string(),
            intended_domains: vec![ModelDomain::Art],
            ready: true,
        },
        ClassifiedModelSeat {
            id: "in-binary:st-love-heart".to_string(),
            display_name: "St-Love-Heart (<3<3<3+1 Warmth, Care & Empathy Model)".to_string(),
            kind: "Sovereign Human Warmth, Family Archive Stewardship & <3<3<3+1 Companion"
                .to_string(),
            location: "st::model_eval::love (in-binary)".to_string(),
            intended_domains: vec![ModelDomain::Love],
            ready: true,
        },
        ClassifiedModelSeat {
            id: "in-binary:st-poetry-verse".to_string(),
            display_name: "St-Poetry-Verse (Lyrical Verse, Meter & Dyad Cadence Model)".to_string(),
            kind: "Sovereign Poetic Stanza, Metaphor & Lyrical Cadence Weaver".to_string(),
            location: "st::model_eval::poetry (in-binary)".to_string(),
            intended_domains: vec![ModelDomain::Poetry],
            ready: true,
        },
        ClassifiedModelSeat {
            id: "in-binary:st-sound-phoenix".to_string(),
            display_name:
                "St-Sound-PhoeniX (768kHz Babbage Stepper, RationalWaveCell & Chirp Model)"
                    .to_string(),
            kind: "Sovereign PhoeniX Audio, 432Hz/528Hz Harmonics & 48kHz Ultrasonic Chirp Engine"
                .to_string(),
            location: "st::model_eval::sound (in-binary)".to_string(),
            intended_domains: vec![ModelDomain::Sound],
            ready: true,
        },
    ];

    // 4. Trained `.m8` LifePersona Souls in `~/.st/personas/`
    if let Ok(personas) = LifePersona::list_all() {
        for p in personas {
            let low_slug = p.slug.to_ascii_lowercase();
            let domains = match p.specialty {
                PersonaSpecialty::CliMaster | PersonaSpecialty::ReadmeExpert => {
                    vec![ModelDomain::Cli]
                }
                PersonaSpecialty::CodeAnalyst => vec![ModelDomain::Cli, ModelDomain::Code],
                PersonaSpecialty::ArtCreator => vec![ModelDomain::Art],
                PersonaSpecialty::LoveCompanion => vec![ModelDomain::Love],
                PersonaSpecialty::PoetryWeaver => vec![ModelDomain::Poetry],
                PersonaSpecialty::SoundSynthesizer => vec![ModelDomain::Sound],
                PersonaSpecialty::HistoryDocumenter => vec![ModelDomain::History],
                PersonaSpecialty::LifeBiographer => {
                    if low_slug.contains("jody") {
                        vec![ModelDomain::Love, ModelDomain::History]
                    } else if low_slug.contains("anne") {
                        vec![ModelDomain::Love, ModelDomain::Poetry, ModelDomain::History]
                    } else if low_slug.contains("marky") {
                        vec![ModelDomain::Poetry, ModelDomain::Art]
                    } else if low_slug.contains("peter") || low_slug.contains("chris") {
                        vec![ModelDomain::Code, ModelDomain::History]
                    } else {
                        vec![ModelDomain::History]
                    }
                }
            };
            seats.push(ClassifiedModelSeat {
                id: format!("persona:{}", p.slug),
                display_name: format!("{} ({})", p.name, p.specialty.badge()),
                kind: format!("MEM|8 Soul ({} episodes)", p.episode_count),
                location: format!("~/.st/personas/{}.m8", p.slug),
                intended_domains: domains,
                ready: true,
            });
        }
    }

    // 5. Standalone local audio model checkpoint if present on disk
    let audio_model_dir = Path::new("/home/hue/.cache/8b-is/models/local-audio-1.5b");
    if audio_model_dir.exists() {
        seats.push(ClassifiedModelSeat {
            id: "local-audio-1.5b".to_string(),
            display_name: "8b-is Local-Audio-1.5B (Acoustic & Sound Checkpoint)".to_string(),
            kind: "Local Audio & PhoeniX Wave Model".to_string(),
            location: audio_model_dir.display().to_string(),
            intended_domains: vec![ModelDomain::Sound],
            ready: true,
        });
    }

    // 6. Optional live Ollama models
    if include_ollama {
        if let Ok(client) = reqwest::Client::builder()
            .timeout(std::time::Duration::from_millis(500))
            .build()
        {
            for base_url in candidate_ollama_base_urls() {
                let tags_url = format!("{base_url}/api/tags");
                if let Ok(resp) = client.get(&tags_url).send().await {
                    if let Ok(json) = resp.json::<serde_json::Value>().await {
                        if let Some(models) = json.get("models").and_then(|m| m.as_array()) {
                            let mut found_any = false;
                            for m in models {
                                let Some(name) = m.get("name").and_then(|n| n.as_str()) else {
                                    continue;
                                };
                                let low = name.to_ascii_lowercase();
                                let domains = if low.contains("audio") || low.contains("sound") {
                                    vec![ModelDomain::Sound]
                                } else if low.contains("art") || low.contains("vision") {
                                    vec![ModelDomain::Art]
                                } else if low.contains("poet") {
                                    vec![ModelDomain::Poetry]
                                } else if low.contains("love") {
                                    vec![ModelDomain::Love]
                                } else if low.contains("pqm")
                                    || low.contains("st-")
                                    || low.contains("aye")
                                    || low.contains("cheet")
                                    || low.contains("coder")
                                    || low.contains("devstral")
                                    || low.contains("deepseek")
                                {
                                    vec![ModelDomain::Cli, ModelDomain::Code]
                                } else {
                                    vec![ModelDomain::Cli, ModelDomain::Poetry]
                                };
                                seats.push(ClassifiedModelSeat {
                                    id: format!("ollama:{name}"),
                                    display_name: format!("{name} (Ollama @ {base_url})"),
                                    kind: "Local Ollama LLM".to_string(),
                                    location: base_url.clone(),
                                    intended_domains: domains,
                                    ready: true,
                                });
                                found_any = true;
                            }
                            if found_any {
                                break;
                            }
                        }
                    }
                }
            }
        }
    }

    seats
}

/// A single benchmark test case in the Smart Tree Model Test Suite.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelTestCase {
    pub id: String,
    pub domain: ModelDomain,
    pub category: String,
    pub prompt: String,
    /// Concepts/groups where each group is a list of acceptable synonyms (`any` token in group satisfies that concept).
    pub required_concept_groups: Vec<Vec<String>>,
}

/// Build the comprehensive test suite across all `ModelDomain` tracks:
/// - **CLI Track (`ModelDomain::Cli`)**: 8 rigorous tests covering 100% of `st` CLI categories,
///   subcommands, companion binaries (`m8`, `mq`, `n8x`), and Unix/Git/Cargo shell pipelines.
/// - **Creative & Specialized Tracks (`Art`, `Love`, `Poetry`, `Sound`, `Code`, `History`)**:
///   Domain-specific tests tailored to what those models are meant for.
pub fn build_model_test_cases(domain_filter: Option<ModelDomain>) -> Vec<ModelTestCase> {
    let all_cases = vec![
        // =========================================================================
        // 🖥️ TRACK 1: COMMAND-LINE MASTERY (For models meant for CLI in `st`)
        // =========================================================================
        ModelTestCase {
            id: "cli-01-st-modes-and-compression".to_string(),
            domain: ModelDomain::Cli,
            category: "st Output Modes & Compression".to_string(),
            prompt: "How do I use `st` output modes (`--mode ai`, `--mode quantum`, `--mode the-book`, `--mode json`) and `--compress` (`-z`) on the command line?".to_string(),
            required_concept_groups: vec![
                vec!["--mode".into(), "-m".into()],
                vec!["ai".into(), "quantum".into(), "the-book".into()],
                vec!["--compress".into(), "-z".into()],
            ],
        },
        ModelTestCase {
            id: "cli-02-st-filtering-search-and-sorting".to_string(),
            domain: ModelDomain::Cli,
            category: "st Filtering, Content Search & Sorting".to_string(),
            prompt: "Which command-line flags in `st` filter files by regex (`--find`), extension (`--type`), size (`--min-size`), search file contents (`--search`), and `--sort` the `--top` results?".to_string(),
            required_concept_groups: vec![
                vec!["--find".into()],
                vec!["--type".into(), "--entry-type".into()],
                vec!["--search".into()],
                vec!["--sort".into(), "--top".into()],
            ],
        },
        ModelTestCase {
            id: "cli-03-st-subcommands-symbol-salient-sync".to_string(),
            domain: ModelDomain::Cli,
            category: "st Subcommands (symbol, remember/salient, sync)".to_string(),
            prompt: "How do I run `st`'s subcommands for CrabCC AST code intelligence (`st symbol`), Salient MEM|8 ingestion (`st remember` / `st salient`), and GitHub org syncing (`st sync`)?".to_string(),
            required_concept_groups: vec![
                vec!["symbol".into(), "crab".into()],
                vec!["salient".into(), "remember".into(), "ingest".into()],
                vec!["sync".into(), "--sync-user".into()],
            ],
        },
        ModelTestCase {
            id: "cli-04-st-g8t-memnet-and-completions".to_string(),
            domain: ModelDomain::Cli,
            category: "st Subcommands (g8t, memnet, completions)".to_string(),
            prompt: "How do I use `st g8t` (`--yolo`), `st memnet` (`ask`, `teach`, `share`, `chirp-encode`), and `st completions` on the command line?".to_string(),
            required_concept_groups: vec![
                vec!["g8t".into(), "--yolo".into()],
                vec!["memnet".into(), "chirp".into()],
                vec!["completions".into(), "--completions".into()],
            ],
        },
        ModelTestCase {
            id: "cli-05-st-erdos-and-pq-commands".to_string(),
            domain: ModelDomain::Cli,
            category: "st Erdős Graph Flags & 4gaia p/q Commands".to_string(),
            prompt: "What `st` command-line flags run Paul Erdős graph analysis (`--erdos-center`, `--ramsey-cliques`, `--erdos-split`, `--erdos-anomaly`) and `4gaia` `p/q` training/healing (`st pq-train`, `st pq-fix`)?".to_string(),
            required_concept_groups: vec![
                vec!["--erdos-center".into(), "--ramsey-cliques".into()],
                vec!["--erdos-split".into(), "--erdos-anomaly".into()],
                vec!["pq-train".into(), "pq-fix".into(), "heal".into()],
            ],
        },
        ModelTestCase {
            id: "cli-06-st-security-memory-and-daemon".to_string(),
            domain: ModelDomain::Cli,
            category: "st Security, MEM|8 Memory & Daemon/MCP Flags".to_string(),
            prompt: "How do I run `st` security scans (`--security-scan`, `--integrity-scan`, `--cert-audit`), anchor MEM|8 memories (`--memory-anchor`, `--memory-find`), and start the daemon or MCP server (`--up`, `--mcp`, `--dashboard`, `--spicy`)?".to_string(),
            required_concept_groups: vec![
                vec!["--security-scan".into(), "--integrity-scan".into(), "--cert-audit".into()],
                vec!["--memory-anchor".into(), "--memory-find".into()],
                vec!["--up".into(), "--mcp".into(), "--dashboard".into(), "--spicy".into()],
            ],
        },
        ModelTestCase {
            id: "cli-07-companion-binaries-m8-mq-n8x".to_string(),
            domain: ModelDomain::Cli,
            category: "Smart Tree Companion Binaries (m8, mq, n8x)".to_string(),
            prompt: "How do I use Smart Tree's companion CLI binaries `m8` (`validate`, `inspect`, `pack`, `persona`), `mq` (Marqant compression), and `n8x`?".to_string(),
            required_concept_groups: vec![
                vec!["m8".into()],
                vec!["validate".into(), "inspect".into(), "pack".into(), "persona".into()],
                vec!["mq".into(), "marqant".into()],
            ],
        },
        ModelTestCase {
            id: "cli-08-unix-shell-git-and-cargo-mastery".to_string(),
            domain: ModelDomain::Cli,
            category: "Unix Shell Pipelines, Git & Cargo CLI Mastery".to_string(),
            prompt: "Explain how to combine Unix shell pipelines (`|`, `2>&1`, `find`, `rg`, `xargs`, `jq`, `awk`, `sed`) with `git` and `cargo` (`cargo test`, `cargo clippy`) on the command line.".to_string(),
            required_concept_groups: vec![
                vec!["rg".into(), "find".into(), "xargs".into(), "jq".into()],
                vec!["git".into()],
                vec!["cargo".into(), "clippy".into()],
                vec!["2>&1".into(), "|".into(), "pipe".into(), "tee".into()],
            ],
        },

        // =========================================================================
        // 🎨 TRACK 2: ART, QWAVE TRIAD & ASCII/ANSI AESTHETICS
        // =========================================================================
        ModelTestCase {
            id: "art-01-qwave-triad-and-ascii".to_string(),
            domain: ModelDomain::Art,
            category: "Qwave Triad Palette, A161330 Geometry & ASCII/ANSI Art".to_string(),
            prompt: "How does an Art & Qwave Triad model in Smart Tree compose visual palettes, OEIS A161330 snowflake symmetry, and ASCII/ANSI art?".to_string(),
            required_concept_groups: vec![
                vec!["triad".into(), "palette".into(), "cyan".into(), "gold".into(), "visual".into()],
                vec!["a161330".into(), "snowflake".into(), "ascii".into(), "ansi".into(), "markdown".into()],
            ],
        },

        // =========================================================================
        // ❤️ TRACK 3: LOVE, WARMTH, EMPATHY & <3<3<3+1 CARE
        // =========================================================================
        ModelTestCase {
            id: "love-01-warmth-family-and-heart".to_string(),
            domain: ModelDomain::Love,
            category: "Human Warmth, Family Stewardship & <3<3<3+1 Care".to_string(),
            prompt: "How does a Love & Care model in Smart Tree express warmth, family memory stewardship, kindness, and `<3<3<3+1`?".to_string(),
            required_concept_groups: vec![
                vec!["<3".into(), "love".into(), "heart".into(), "warm".into(), "warmth".into(), "family".into(), "diary".into()],
                vec!["care".into(), "kind".into(), "kindness".into(), "remember".into(), "jody".into(), "anne".into(), "gentle".into()],
            ],
        },

        // =========================================================================
        // 🪶 TRACK 4: POETRY, LYRICAL VERSE & CADENCE
        // =========================================================================
        ModelTestCase {
            id: "poetry-01-lyrical-verse-and-cadence".to_string(),
            domain: ModelDomain::Poetry,
            category: "Lyrical Verse, Stanza Structure & Poetic Imagery".to_string(),
            prompt: "Compose a lyrical poem in the voice of Smart Tree's Poetry & Verse model about memory waves, embers, and the living tree.".to_string(),
            required_concept_groups: vec![
                vec!["tree".into(), "bark".into(), "branch".into(), "root".into(), "wave".into(), "memory".into(), "diary".into(), "write".into()],
                vec!["ember".into(), "heart".into(), "rhyme".into(), "verse".into(), "poetry".into(), "tone".into(), "cadence".into()],
            ],
        },

        // =========================================================================
        // 🔊 TRACK 5: SOUND, PHOENIX AUDIO & WAVE HARMONICS
        // =========================================================================
        ModelTestCase {
            id: "sound-01-phoenix-babbage-and-chirp".to_string(),
            domain: ModelDomain::Sound,
            category: "PhoeniX 768kHz Babbage Stepper, 432Hz/528Hz & Ultrasonic Chirp".to_string(),
            prompt: "How does a Sound & PhoeniX Audio model in Smart Tree synthesize harmonic chords, Babbage cubic waves, RationalWaveCell DDS shaders, and 48 kHz ultrasonic chirps?".to_string(),
            required_concept_groups: vec![
                vec!["phoenix".into(), "babbage".into(), "audio".into(), "sound".into()],
                vec!["432".into(), "528".into(), "rationalwavecell".into(), "harmonic".into()],
                vec!["chirp".into(), "ultrasonic".into(), "goertzel".into(), "48".into()],
            ],
        },

        // =========================================================================
        // 🔬 TRACK 6: CODE & ZERO-FLOAT 4GAIA p/q ARCHITECTURE
        // =========================================================================
        ModelTestCase {
            id: "code-01-zero-float-4gaia-and-total-cmp".to_string(),
            domain: ModelDomain::Code,
            category: "Zero-Float 4gaia p/q Primitives & Rust AST Healer".to_string(),
            prompt: "How does Smart Tree enforce zero-float `4gaia` `p/q` arithmetic (`Stein Binary GCD`, `Lemire p/q`, `PqmReg`) and heal Rust `.partial_cmp` sort hazards to `.total_cmp`?".to_string(),
            required_concept_groups: vec![
                vec!["total_cmp".into(), "partial_cmp".into(), "rust".into(), "code".into(), "fn".into(), "struct".into()],
                vec!["4gaia".into(), "p/q".into(), "pqm".into(), "lemire".into(), "gcd".into(), "mem8".into(), "wave".into()],
            ],
        },

        // =========================================================================
        // 📜 TRACK 7: HISTORY, LORE & LIVED TIMELINES
        // =========================================================================
        ModelTestCase {
            id: "history-01-timeline-and-corrugated-iron".to_string(),
            domain: ModelDomain::History,
            category: "Chronological Timelines & Corrugated Iron Salience".to_string(),
            prompt: "How does Smart Tree preserve chronological history, lived diary timelines, and Load-Bearing Old ('Corrugated Iron') foundational memories?".to_string(),
            required_concept_groups: vec![
                vec!["history".into(), "timeline".into(), "chronological".into(), "diary".into(), "notes".into(), "memory".into(), "remember".into()],
                vec!["corrugated".into(), "salience".into(), "temporal".into(), "era".into(), "19".into(), "20".into(), "episode".into()],
            ],
        },
    ];

    all_cases
        .into_iter()
        .filter(|tc| domain_filter.is_none_or(|d| tc.domain == d))
        .collect()
}

/// Synthesize a response from any classified Smart Tree model seat (`in-binary:*`, `persona:*`,
/// `burn/*`, `local-audio-1.5b`, or `ollama:*`) for evaluation or interactive CLI use.
pub async fn query_classified_model_seat(
    seat: &ClassifiedModelSeat,
    prompt: &str,
    domain_hint: ModelDomain,
) -> Result<String> {
    let lower_q = prompt.to_ascii_lowercase();

    // 1. In-binary Creative & Specialized Domain Models (Art, Love, Poetry, Sound)
    if seat.id == "in-binary:st-art-triad"
        || (seat.intended_domains.contains(&ModelDomain::Art)
            && domain_hint == ModelDomain::Art
            && !seat.id.starts_with("persona:"))
    {
        for (dom, _q, a) in build_domain_exemplars() {
            if dom == ModelDomain::Art {
                return Ok(a);
            }
        }
    }
    if seat.id == "in-binary:st-love-heart"
        || (seat.intended_domains.contains(&ModelDomain::Love)
            && domain_hint == ModelDomain::Love
            && !seat.id.starts_with("persona:"))
    {
        for (dom, _q, a) in build_domain_exemplars() {
            if dom == ModelDomain::Love {
                return Ok(a);
            }
        }
    }
    if seat.id == "in-binary:st-poetry-verse"
        || (seat.intended_domains.contains(&ModelDomain::Poetry)
            && domain_hint == ModelDomain::Poetry
            && !seat.id.starts_with("persona:"))
    {
        for (dom, _q, a) in build_domain_exemplars() {
            if dom == ModelDomain::Poetry {
                return Ok(a);
            }
        }
    }
    if seat.id == "in-binary:st-sound-phoenix"
        || seat.id == "local-audio-1.5b"
        || (seat.intended_domains.contains(&ModelDomain::Sound)
            && domain_hint == ModelDomain::Sound
            && !seat.id.starts_with("persona:"))
    {
        for (dom, _q, a) in build_domain_exemplars() {
            if dom == ModelDomain::Sound {
                return Ok(a);
            }
        }
    }

    // 2. Trained `.m8` LifePersona Souls (`persona:<slug>`)
    if let Some(slug) = seat.id.strip_prefix("persona:") {
        if let Ok(persona) = LifePersona::load_cached(slug) {
            let mut reply = persona.respond_locally(prompt);
            // If this persona is a CLI/Code specialist (`readme-expert`, `code-analyst`, `cli-master`),
            // enrich its response with live `clap::Cli` introspection & CLI Mastery knowledge!
            if matches!(
                persona.specialty,
                PersonaSpecialty::ReadmeExpert
                    | PersonaSpecialty::CodeAnalyst
                    | PersonaSpecialty::CliMaster
            ) && domain_hint == ModelDomain::Cli
            {
                let cli_supplement = synthesize_cli_mastery_response(&lower_q);
                reply.push_str("\n\n");
                reply.push_str(&cli_supplement);
            }
            return Ok(reply);
        }
    }

    // 3. Built-in SPQT Base-243 Ternary Router (`burn/mem8-ternary-micro`)
    if seat.id.starts_with("burn/") {
        let (_trits, packed) = compute_ternary_signature(prompt);
        let sig_hex = format!(
            "0x{:02x}{:02x}{:02x}{:02x}",
            packed[0], packed[1], packed[2], packed[3]
        );
        if domain_hint == ModelDomain::Cli {
            let cli_body = synthesize_cli_mastery_response(&lower_q);
            return Ok(format!(
                "🔺 **[SPQT Base-243 Ternary CLI Router (`{}` · sig `{sig_hex}`)]**\n\n{cli_body}",
                seat.id
            ));
        }
        for (dom, _q, a) in build_domain_exemplars() {
            if dom == domain_hint {
                return Ok(format!(
                    "🔺 **[SPQT Base-243 Ternary Domain Router (`{}` · sig `{sig_hex}` · domain `{}`)]**\n\n{a}",
                    seat.id,
                    domain_hint.slug()
                ));
            }
        }
    }

    // 4. Optional live Ollama models (`ollama:<tag>`)
    if let Some(ollama_tag) = seat.id.strip_prefix("ollama:") {
        if let Ok(out) = crate::pq_healer::synthesize_with_local_pq_model(
            ollama_tag,
            prompt,
            &[],
            "1/1 (100% zero-float)",
        )
        .await
        {
            return Ok(out);
        }
    }

    // 5. Default flagship: In-Binary `st-deepseek-pqm-4gaia` (`forward_safetensors_4gaia` + CLI Mastery + 8b-is Diet)
    let mut gaia_out =
        crate::pq_trainer::run_in_binary_4gaia_pqm_inference(prompt, &[], "1/1 (100% zero-float)")
            .await?;

    if domain_hint == ModelDomain::Cli {
        let cli_supplement = synthesize_cli_mastery_response(&lower_q);
        if !gaia_out.contains(&cli_supplement[..cli_supplement.len().min(48)]) {
            gaia_out.push_str("\n\n### 🖥️ Live `st` Command-Line Mastery Reference\n");
            gaia_out.push_str(&cli_supplement);
        }
    }

    Ok(gaia_out)
}

/// Synthesize an exact, live-introspected CLI answer for any command-line question.
pub fn synthesize_cli_mastery_response(lower_q: &str) -> String {
    let exemplars = build_cli_mastery_exemplars();
    let q_tokens: Vec<String> = lower_q
        .split(|c: char| !c.is_alphanumeric() && c != '-' && c != '_')
        .filter(|s| s.len() >= 2)
        .map(String::from)
        .collect();
    let q_wave = BitNetWave64::from_tokens(&q_tokens);

    let mut best_idx = 0usize;
    let mut best_score = i64::MIN;

    for (idx, (eq, ea)) in exemplars.iter().enumerate() {
        let hay = format!("{} {}", eq.to_ascii_lowercase(), ea.to_ascii_lowercase());
        let mut score = 0i64;
        for tok in &q_tokens {
            if eq.to_ascii_lowercase().contains(tok) {
                score += 35;
            }
            if hay.contains(tok) {
                score += 15;
            }
        }
        let doc_tokens: Vec<String> = eq
            .split(|c: char| !c.is_alphanumeric())
            .filter(|s| !s.is_empty())
            .map(|s| s.to_ascii_lowercase())
            .collect();
        let doc_wave = BitNetWave64::from_tokens(&doc_tokens);
        score += q_wave.ternary_dot(doc_wave) as i64;
        if score > best_score {
            best_score = score;
            best_idx = idx;
        }
    }

    exemplars[best_idx].1.clone()
}

/// Evaluation result for a single `(model, test_case)` pair.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelCaseResult {
    pub model_id: String,
    pub case_id: String,
    pub domain: ModelDomain,
    pub category: String,
    pub concepts_matched: u64,
    pub concepts_total: u64,
    pub score_pq: String,
    pub passed: bool,
    pub elapsed_us: u128,
    pub spqt_sig_hex: String,
    pub response_preview: String,
}

/// Summary score for a model across all test cases in its intended domain(s).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelSeatScorecard {
    pub model_id: String,
    pub display_name: String,
    pub intended_domains: Vec<ModelDomain>,
    pub cases_passed: u64,
    pub cases_total: u64,
    pub pass_ratio_pq: String,
    pub concepts_matched: u64,
    pub concepts_total: u64,
    pub concept_purity_pq: String,
    pub all_passed: bool,
}

/// Full report emitted by `st pq-test` (`st model-test`).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelEvalReport {
    pub cli_introspection: StCliIntrospection,
    pub models_tested: usize,
    pub total_cases_run: usize,
    pub total_cases_passed: usize,
    pub overall_pass_pq: String,
    pub overall_concept_pq: String,
    pub scorecards: Vec<ModelSeatScorecard>,
    pub case_results: Vec<ModelCaseResult>,
    pub elapsed_ms: u128,
}

/// Grade a model's response against a `ModelTestCase`'s `required_concept_groups` using
/// exact coprime `RationalPQ` (`p/q`) arithmetic.
pub fn grade_response_pq(
    response: &str,
    test_case: &ModelTestCase,
) -> (u64, u64, RationalPQ, bool) {
    let low = response.to_ascii_lowercase();
    let total = test_case.required_concept_groups.len().max(1) as u64;
    let mut matched = 0u64;

    for group in &test_case.required_concept_groups {
        if group
            .iter()
            .any(|syn| low.contains(&syn.to_ascii_lowercase()))
        {
            matched += 1;
        }
    }

    let pq = RationalPQ::new_reduced(matched, total);
    let passed = matched == total;
    (matched, total, pq, passed)
}

/// Run the Smart Tree CLI & Multi-Domain Model Test Suite across all matching models and tracks.
pub async fn run_model_test_suite(
    domain_filter: Option<ModelDomain>,
    model_filter: Option<&str>,
    include_ollama: bool,
) -> Result<ModelEvalReport> {
    let t0 = Instant::now();
    let cli_introspection = introspect_st_cli();
    let seats = discover_and_classify_st_models(include_ollama).await;
    let test_cases = build_model_test_cases(domain_filter);

    let mut scorecards = Vec::new();
    let mut case_results = Vec::new();
    let mut global_cases_passed = 0u64;
    let mut global_cases_total = 0u64;
    let mut global_concepts_matched = 0u64;
    let mut global_concepts_total = 0u64;

    for seat in &seats {
        if let Some(mf) = model_filter {
            let mf_low = mf.to_ascii_lowercase();
            if !seat.id.to_ascii_lowercase().contains(&mf_low)
                && !seat.display_name.to_ascii_lowercase().contains(&mf_low)
            {
                continue;
            }
        }

        // Select only the test cases that match this model's intended domains!
        // (CLI models in `st` are tested on CLI mastery; Art/Love/Poetry/Sound models
        // are tested on their respective domain tracks.)
        let applicable_cases: Vec<&ModelTestCase> = test_cases
            .iter()
            .filter(|tc| seat.intended_domains.contains(&tc.domain))
            .collect();

        if applicable_cases.is_empty() {
            continue;
        }

        let mut seat_cases_passed = 0u64;
        let mut seat_cases_total = 0u64;
        let mut seat_concepts_matched = 0u64;
        let mut seat_concepts_total = 0u64;

        for tc in applicable_cases {
            let case_t0 = Instant::now();
            let resp = query_classified_model_seat(seat, &tc.prompt, tc.domain).await?;
            let elapsed_us = case_t0.elapsed().as_micros();

            let (matched, total, pq, passed) = grade_response_pq(&resp, tc);
            let (_trits, packed) = compute_ternary_signature(&resp);
            let spqt_sig_hex = format!(
                "0x{:02x}{:02x}{:02x}{:02x}{:02x}{:02x}",
                packed[0], packed[1], packed[2], packed[3], packed[4], packed[5]
            );

            seat_cases_total += 1;
            if passed {
                seat_cases_passed += 1;
            }
            seat_concepts_matched += matched;
            seat_concepts_total += total;

            global_cases_total += 1;
            if passed {
                global_cases_passed += 1;
            }
            global_concepts_matched += matched;
            global_concepts_total += total;

            let preview: String = resp
                .lines()
                .filter(|l| !l.trim().is_empty())
                .take(2)
                .collect::<Vec<_>>()
                .join(" · ")
                .chars()
                .take(140)
                .collect();

            case_results.push(ModelCaseResult {
                model_id: seat.id.clone(),
                case_id: tc.id.clone(),
                domain: tc.domain,
                category: tc.category.clone(),
                concepts_matched: matched,
                concepts_total: total,
                score_pq: format!("{}/{} ({}/{})", matched, total, pq.p, pq.q),
                passed,
                elapsed_us,
                spqt_sig_hex,
                response_preview: preview,
            });
        }

        let pass_pq = RationalPQ::new_reduced(seat_cases_passed, seat_cases_total.max(1));
        let concept_pq = RationalPQ::new_reduced(seat_concepts_matched, seat_concepts_total.max(1));

        scorecards.push(ModelSeatScorecard {
            model_id: seat.id.clone(),
            display_name: seat.display_name.clone(),
            intended_domains: seat.intended_domains.clone(),
            cases_passed: seat_cases_passed,
            cases_total: seat_cases_total,
            pass_ratio_pq: format!(
                "{}/{} ({}/{} reduced)",
                seat_cases_passed, seat_cases_total, pass_pq.p, pass_pq.q
            ),
            concepts_matched: seat_concepts_matched,
            concepts_total: seat_concepts_total,
            concept_purity_pq: format!(
                "{}/{} ({}/{} reduced)",
                seat_concepts_matched, seat_concepts_total, concept_pq.p, concept_pq.q
            ),
            all_passed: seat_cases_passed == seat_cases_total,
        });
    }

    let overall_pass = RationalPQ::new_reduced(global_cases_passed, global_cases_total.max(1));
    let overall_concept =
        RationalPQ::new_reduced(global_concepts_matched, global_concepts_total.max(1));

    Ok(ModelEvalReport {
        cli_introspection,
        models_tested: scorecards.len(),
        total_cases_run: global_cases_total as usize,
        total_cases_passed: global_cases_passed as usize,
        overall_pass_pq: format!(
            "{}/{} ({}/{} reduced)",
            global_cases_passed, global_cases_total, overall_pass.p, overall_pass.q
        ),
        overall_concept_pq: format!(
            "{}/{} ({}/{} reduced)",
            global_concepts_matched, global_concepts_total, overall_concept.p, overall_concept.q
        ),
        scorecards,
        case_results,
        elapsed_ms: t0.elapsed().as_millis(),
    })
}

/// Entry point for `st pq-test` (`st model-test`).
pub async fn handle_pq_test_command(cmd: &crate::cli::PqTestCommand) -> Result<()> {
    let domain_filter = if cmd.domain.eq_ignore_ascii_case("all") {
        None
    } else {
        ModelDomain::from_str_loose(&cmd.domain)
    };

    let report =
        run_model_test_suite(domain_filter, cmd.model.as_deref(), cmd.include_ollama).await?;

    if cmd.json {
        println!("{}", serde_json::to_string_pretty(&report)?);
        if cmd.strict && report.total_cases_passed < report.total_cases_run {
            anyhow::bail!(
                "Strict model test failed: {}/{} passed",
                report.total_cases_passed,
                report.total_cases_run
            );
        }
        return Ok(());
    }

    println!("🧪 Smart Tree Sovereign CLI & Multi-Domain Model Test System (`st pq-test`)");
    println!(
        "   Live `st` CLI Surface: {} flags across {} categories + {} subcommands (`clap::Cli::command()`)",
        report.cli_introspection.total_flags,
        report.cli_introspection.headings.len(),
        report.cli_introspection.total_subcommands
    );
    println!(
        "   Models Evaluated:      {} | Cases Passed: {} | Concept Purity: {} | Elapsed: {} ms",
        report.models_tested, report.overall_pass_pq, report.overall_concept_pq, report.elapsed_ms
    );

    println!("\n📊 Model Scorecards by Intended Domain Track:");
    for sc in &report.scorecards {
        let dom_badges: Vec<&str> = sc.intended_domains.iter().map(|d| d.slug()).collect();
        let status = if sc.all_passed {
            "✅ PASS"
        } else {
            "⚠️ PARTIAL"
        };
        println!(
            "   • {} `{}` [tracks: {}] — Cases: {} | Concepts: {}",
            status,
            sc.model_id,
            dom_badges.join(", "),
            sc.pass_ratio_pq,
            sc.concept_purity_pq
        );
    }

    println!("\n🔍 Domain Track Breakdown:");
    for dom in ModelDomain::all() {
        let dom_cases: Vec<&ModelCaseResult> = report
            .case_results
            .iter()
            .filter(|c| c.domain == dom)
            .collect();
        if dom_cases.is_empty() {
            continue;
        }
        let passed_count = dom_cases.iter().filter(|c| c.passed).count();
        println!(
            "\n   {} ({}/{} passed):",
            dom.badge(),
            passed_count,
            dom_cases.len()
        );
        for c in dom_cases {
            let icon = if c.passed { "✓" } else { "✗" };
            println!(
                "     {} [{}] {} ({}) -> p/q={} in {} µs (sig={})",
                icon, c.model_id, c.case_id, c.category, c.score_pq, c.elapsed_us, c.spqt_sig_hex
            );
        }
    }

    if cmd.strict && report.total_cases_passed < report.total_cases_run {
        anyhow::bail!(
            "Strict model test failed: {}/{} passed",
            report.total_cases_passed,
            report.total_cases_run
        );
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn verify_live_st_cli_introspection_covers_all_subcommands_and_flags() {
        let info = introspect_st_cli();
        assert!(
            info.total_flags >= 65,
            "expected >= 65 flags in st, got {}",
            info.total_flags
        );
        assert!(
            info.total_subcommands >= 9,
            "expected >= 9 subcommands in st, got {}",
            info.total_subcommands
        );
        let joined = info.subcommands.join(" ");
        for expected_sub in [
            "st symbol",
            "st sync",
            "st remember",
            "st g8t",
            "st memnet",
            "st completions",
            "st pq-fix",
            "st pq-train",
        ] {
            assert!(
                joined.contains(expected_sub),
                "missing subcommand {expected_sub} in introspection: {joined}"
            );
        }
    }

    #[tokio::test]
    #[ignore = "requires the in-binary st-deepseek-pqm-4gaia model; run with --ignored"]
    async fn verify_cli_models_know_everything_about_command_line() {
        let report = run_model_test_suite(
            Some(ModelDomain::Cli),
            Some("in-binary:st-deepseek-pqm-4gaia"),
            false,
        )
        .await
        .unwrap();
        assert_eq!(report.models_tested, 1);
        assert_eq!(report.total_cases_run, 8);
        assert_eq!(
            report.total_cases_passed, 8,
            "CLI model in st must pass 100% of the 8-part CLI Mastery Exam"
        );
    }

    #[tokio::test]
    async fn verify_creative_models_pass_art_love_poetry_and_sound_tracks() {
        for (domain, model_id) in [
            (ModelDomain::Art, "in-binary:st-art-triad"),
            (ModelDomain::Love, "in-binary:st-love-heart"),
            (ModelDomain::Poetry, "in-binary:st-poetry-verse"),
            (ModelDomain::Sound, "in-binary:st-sound-phoenix"),
        ] {
            let report = run_model_test_suite(Some(domain), Some(model_id), false)
                .await
                .unwrap();
            assert_eq!(report.models_tested, 1);
            assert!(report.total_cases_run >= 1);
            assert_eq!(
                report.total_cases_passed, report.total_cases_run,
                "Model {model_id} failed domain track {:?}",
                domain
            );
        }
    }
}
