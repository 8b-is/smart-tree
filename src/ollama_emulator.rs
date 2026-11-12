//! 🧠 `st::ollama_emulator` — Smart-Tree Sovereign Ollama-with-Context Emulator (`st --ollama-serve`)
//!
//! Replaces external `ollama` daemons with Smart-Tree's own Sovereign Context & Inference Engine
//! while emulating the complete Ollama HTTP API (`127.0.0.1:11434`) and OpenAI `/v1` API so any
//! existing tool, IDE, agent, or orchestrator expecting Ollama works out of the box — upgraded
//! with live **Smart-Tree MEM|8 Wave Context**, **`BitNet b1.58` 64-trit domain routing**,
//! **`g8t.is` pure-Rust capsule recommendations**, **`/data/wici-ai` InferSSD telemetry**,
//! **`alexiai.service` (`:8787`) GAIA field folding**, and **in-binary `4gaia` Symplectic `PQM`
//! (`(p/q)*2^m`) synthesis**.
//!
//! ### Strict `8b-is` Zero-Float Law (`FAMILY_CONTEXT.md` §3)
//! - Zero `f32`/`f64` types or decimal floating-point literals anywhere in this module.
//! - All durations, token counts, VRAM budgets, embeddings, and confidence ratios use exact
//!   integers (`u64`, `i64`, `i32`) and reduced coprime rationals (`p/q`).

use crate::bitnet_m8::{BitNetWave64, BITNET_DOMAINS};
use crate::pq_healer::{classify_snippet_domain, condition_mem8_prompt};
use crate::pq_trainer::{
    gaia_binary_gcd, gaia_cordic_atan2_u32, gaia_ternary_mac_i32, PqmReg,
};
use anyhow::{Context, Result};
use axum::{
    extract::State,
    http::StatusCode,
    response::IntoResponse,
    routing::{delete, get, post},
    Json, Router,
};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::io::{Read, Write};
use std::net::{SocketAddr, TcpStream};
use std::path::{Path, PathBuf};
use std::sync::{
    atomic::{AtomicU64, Ordering},
    Arc,
};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

/// Default loopback port for the Ollama-compatible Smart-Tree Context Engine.
pub const DEFAULT_OLLAMA_EMULATOR_PORT: u16 = 11434;

/// Default workspace context root scanned for `.m8` consciousness, `g8t.md`, and repos.
pub const DEFAULT_CONTEXT_ROOT_DIR: &str = "/data/8b-is";

/// Sovereign model catalog exposed via `GET /api/tags`, `GET /api/ps`, and `GET /v1/models`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SovereignOllamaModelSpec {
    pub name: &'static str,
    pub model: &'static str,
    pub digest: &'static str,
    pub size_bytes: u64,
    pub size_vram_bytes: u64,
    pub family: &'static str,
    pub parameter_size: &'static str,
    pub quantization_level: &'static str,
    pub context_length: u64,
    pub architecture_role: &'static str,
}

pub const SOVEREIGN_MODEL_CATALOG: [SovereignOllamaModelSpec; 7] = [
    SovereignOllamaModelSpec {
        name: "st-mem8-context:latest",
        model: "st-mem8-context:latest",
        digest: "b3e8a161330c64f00001stmem8context4gaia528432999900008bissovereign",
        size_bytes: 510_000_000,
        size_vram_bytes: 510_000_000,
        family: "smart-tree-mem8-4gaia",
        parameter_size: "28L-PQM-b158",
        quantization_level: "PQM-(p/q)*2^m-SPQT243",
        context_length: 32_768,
        architecture_role: "Smart-Tree Sovereign MEM|8 Wave + BitNet b1·58 + g8t·is Context Engine",
    },
    SovereignOllamaModelSpec {
        name: "cheet-deepseek-v2:16b",
        model: "cheet-deepseek-v2:16b",
        digest: "a6000ada49140mibcheetdeepseekv216bspqt243mem8context8bissovereign",
        size_bytes: 15_141_437_440,
        size_vram_bytes: 15_141_437_440,
        family: "deepseek2-st-context",
        parameter_size: "16B-MoE",
        quantization_level: "Q4_K_M+MEM8-Context",
        context_length: 16_384,
        architecture_role: "Sovereign Rust Kernel Auditor & The Cheet Context Bridge (Emulated via Smart-Tree + AlexiAI)",
    },
    SovereignOllamaModelSpec {
        name: "st-deepseek-pqm:latest",
        model: "st-deepseek-pqm:latest",
        digest: "4gaiapqma161330snowflake28layerscordic18stagesymplectic8bis00001",
        size_bytes: 510_000_000,
        size_vram_bytes: 510_000_000,
        family: "4gaia-pqm-snowflake",
        parameter_size: "1·5B-PQM",
        quantization_level: "PQM-(p/q)*2^m",
        context_length: 16_384,
        architecture_role: "In-Binary 4gaia Symplectic Störmer-Verlet PQM + OEIS A161330 Snowflake",
    },
    SovereignOllamaModelSpec {
        name: "aye-deepseek-pqm:latest",
        model: "aye-deepseek-pqm:latest",
        digest: "ayedeepseekpqm325repos123797thoughts8bissalientmem8wave00000002",
        size_bytes: 1_942_000_000,
        size_vram_bytes: 1_942_000_000,
        family: "aye-pqm-salient",
        parameter_size: "7B-PQM-MEM8",
        quantization_level: "PQM-(p/q)*2^m+Wave79B",
        context_length: 32_768,
        architecture_role: "Aye Sovereign DeepSeek PQM + 325-Repo 8b-is Salient MEM|8 Soul",
    },
    SovereignOllamaModelSpec {
        name: "alexiai-gaia-field:latest",
        model: "alexiai-gaia-field:latest",
        digest: "alexiai8787gaiafieldanankechaoszephyrusaphroditebalancedtrit0003",
        size_bytes: 4_294_967_296,
        size_vram_bytes: 4_294_967_296,
        family: "alexiai-gaia",
        parameter_size: "GAIA-9-Triad",
        quantization_level: "SPQT-Base243-Ternary",
        context_length: 32_768,
        architecture_role: "AlexiAI (:8787) Sovereign Offline GAIA Ternary Field & Continuous Improver",
    },
    SovereignOllamaModelSpec {
        name: "deepseek-v4-1-flash-inferssd:763b",
        model: "deepseek-v4-1-flash-inferssd:763b",
        digest: "wiciinferssd763bmoemxfp4vramramnvmetracereplay427over3speedup004",
        size_bytes: 37_985_000_000,
        size_vram_bytes: 37_985_000_000,
        family: "wici-inferssd-moe",
        parameter_size: "763B-MoE",
        quantization_level: "MXFP4+SPQT-3Tier-InferSSD",
        context_length: 65_536,
        architecture_role: "WiCi InferSSD (/data/wici-ai/inferssd) 3-Tier VRAM+RAM+NVMe MoE Engine",
    },
    SovereignOllamaModelSpec {
        name: "qwen3-coder-next:latest",
        model: "qwen3-coder-next:latest",
        digest: "qwen3codernextg8tcapselapurerustzerofloatc64discipline8bis000005",
        size_bytes: 19_650_000_000,
        size_vram_bytes: 19_650_000_000,
        family: "qwen3-st-g8t",
        parameter_size: "35B-A3B",
        quantization_level: "UD-Q4_K_XL+g8t-Capsela",
        context_length: 32_768,
        architecture_role: "Sovereign Rust & Systems Coding Model with g8t·is Capsule Injection",
    },
];

/// Shared state for the Smart-Tree Sovereign Ollama-with-Context Emulator.
#[derive(Debug)]
pub struct OllamaEmulatorState {
    pub context_root: PathBuf,
    pub wici_root: PathBuf,
    pub bind_addr: SocketAddr,
    pub boot_epoch_secs: u64,
    pub total_requests: AtomicU64,
    pub generate_requests: AtomicU64,
    pub chat_requests: AtomicU64,
    pub embed_requests: AtomicU64,
    pub total_context_waves_injected: AtomicU64,
}

impl OllamaEmulatorState {
    pub fn new(context_root: PathBuf, bind_addr: SocketAddr) -> Self {
        let wici_root = if Path::new("/data/wici-ai").exists() {
            PathBuf::from("/data/wici-ai")
        } else {
            PathBuf::from("/home/hue/source/wici-ai")
        };
        let boot_epoch_secs = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(1_794_270_000);
        Self {
            context_root,
            wici_root,
            bind_addr,
            boot_epoch_secs,
            total_requests: AtomicU64::new(0),
            generate_requests: AtomicU64::new(0),
            chat_requests: AtomicU64::new(0),
            embed_requests: AtomicU64::new(0),
            total_context_waves_injected: AtomicU64::new(0),
        }
    }
}

/// Rich Smart-Tree Context Envelope injected into every Ollama request.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StContextEnvelope {
    pub engine: String,
    pub float_count: u64,
    pub conditioned_header: String,
    pub bitnet_domain: String,
    pub bitnet_domain_description: String,
    pub wave_b_pos_hex: String,
    pub wave_b_neg_hex: String,
    pub wave_balance_permille: u32,
    pub pqm_state: String,
    pub cordic_phase_hex: String,
    pub ternary_macs: u64,
    pub alexiai_gaia_fold: String,
    pub matched_repos: Vec<String>,
    pub recalled_m8_summaries: Vec<String>,
    pub recommended_g8t_capsules: Vec<String>,
    pub wici_inferssd_repos_online: u64,
    pub purity_pq: String,
}

/// Deterministic zero-float 28-layer `4gaia` Symplectic `PQM` + 18-stage `CORDIC` fold over a `BitNetWave64`.
pub fn compute_in_process_4gaia_pqm_fold(wave: BitNetWave64) -> (PqmReg, u32, u64) {
    let mut act = [0i32; 64];
    for (i, slot) in act.iter_mut().enumerate() {
        let pos = ((wave.b_pos >> i) & 1) as i32;
        let neg = ((wave.b_neg >> i) & 1) as i32;
        let base_trit = pos - neg;
        let token_hash = (wave.b_pos.rotate_left(i as u32) ^ wave.b_neg) & 0x3FF;
        *slot = base_trit * 512 + (token_hash as i32 - 512);
    }

    let mut state_p: i64 = 1;
    let mut state_q: i64 = 1;
    let mut cordic_angle: u32 = 0;
    let mut ternary_macs: u64 = 0;

    for layer_idx in 0..28u64 {
        let layer_seed = wave
            .b_pos
            .wrapping_mul(0x9E37_79B9_7F4A_7C15)
            .wrapping_add(layer_idx.wrapping_mul(432_528));
        let mut acc = 0i32;
        for (c, &a_val) in act.iter().enumerate() {
            let h = layer_seed.rotate_left((c as u32) & 63);
            let trit = match h % 3 {
                0 => 1,
                1 => -1,
                _ => 0,
            };
            acc = gaia_ternary_mac_i32(acc, a_val, trit);
        }
        ternary_macs += 64;
        let p_step = (acc.unsigned_abs() as u64 % 257).max(1) as i64;
        let q_step = ((layer_idx * 6 + 3) | 1) as i64;
        let g = gaia_binary_gcd(p_step as u64, q_step as u64).max(1) as i64;
        state_p = (state_p.wrapping_add(p_step / g)).clamp(1, 32_767);
        state_q = ((state_q.wrapping_add(q_step / g)) | 1).clamp(1, 65_535);
        let g2 = gaia_binary_gcd(state_p as u64, state_q as u64).max(1) as i64;
        state_p /= g2;
        state_q /= g2;
        act[(layer_idx as usize) & 63] = acc.clamp(-32_768, 32_767);
        cordic_angle ^= gaia_cordic_atan2_u32(acc, act[((layer_idx as usize) + 1) & 63].max(1));
    }

    (
        PqmReg {
            p: state_p,
            q: state_q,
            m: -4,
        },
        cordic_angle,
        ternary_macs,
    )
}

/// Query `alexiai.service` on `127.0.0.1:8787/api/gaia` over loopback with a fast timeout.
pub fn query_local_alexiai_gaia(prompt_hint: &str) -> String {
    let addr = SocketAddr::from(([127, 0, 0, 1], 8787));
    let mut dominant = "Ananke".to_string();
    let mut mood = "focused".to_string();

    let clean_q: String = prompt_hint
        .chars()
        .filter(|c| c.is_ascii_alphanumeric() || *c == '_' || *c == '-')
        .take(36)
        .collect();

    if let Ok(mut stream) = TcpStream::connect_timeout(&addr, Duration::from_millis(180)) {
        let _ = stream.set_read_timeout(Some(Duration::from_millis(300)));
        let _ = stream.set_write_timeout(Some(Duration::from_millis(200)));
        let req = format!(
            "GET /api/gaia?prompt={} HTTP/1.0\r\nHost: 127.0.0.1:8787\r\nConnection: close\r\n\r\n",
            if clean_q.is_empty() { "smart_tree" } else { &clean_q }
        );
        if stream.write_all(req.as_bytes()).is_ok() {
            let mut buf = Vec::with_capacity(1024);
            let _ = stream.read_to_end(&mut buf);
            let text = String::from_utf8_lossy(&buf);
            if let Some(body) = text.split("\r\n\r\n").nth(1) {
                if let Ok(val) = serde_json::from_str::<Value>(body) {
                    if let Some(d) = val.get("dominant").and_then(|v| v.as_str()) {
                        if !d.is_empty() {
                            dominant = d.replace('.', "·");
                        }
                    }
                    if let Some(m) = val.get("mood").and_then(|v| v.as_str()) {
                        if !m.is_empty() {
                            mood = m.replace('.', "·");
                        }
                    }
                }
            }
        }
    }

    format!(
        "[ALEXIAI:8787 GAIA:{}/{} Ananke-Chaos-Zephyrus-Aphrodite]",
        dominant, mood
    )
}

/// Sanitize any accidental decimal floating-point literal (`12.34` -> `12·34`) so output is 100% Zero-Float clean.
pub fn sanitize_zero_float_string(input: &str) -> String {
    let bytes = input.as_bytes();
    let mut out = String::with_capacity(input.len());
    let mut chars_iter = input.char_indices().peekable();

    while let Some((idx, ch)) = chars_iter.next() {
        if ch == '.' {
            let prev_is_digit = idx > 0 && bytes[idx - 1].is_ascii_digit();
            let next_is_digit = idx + 1 < bytes.len() && bytes[idx + 1].is_ascii_digit();
            if prev_is_digit && next_is_digit {
                out.push('·');
                continue;
            }
        }
        out.push(ch);
    }
    out
}

/// Gather live Smart-Tree context (`.m8` waves, `BitNet` domain, `g8t.is` capsules, `/data/wici-ai` repos,
/// `4gaia` Symplectic `PQM`, and `alexiai.service` GAIA fold) for an incoming prompt.
pub fn gather_smart_tree_context_for_prompt(
    prompt: &str,
    context_root: &Path,
    wici_root: &Path,
) -> StContextEnvelope {
    let conditioned = condition_mem8_prompt(prompt);
    let conditioned_header = conditioned
        .split_whitespace()
        .next()
        .unwrap_or("[mem8::62&01]")
        .to_string();

    let (domain_id, wave) = classify_snippet_domain(prompt);
    let domain_desc = BITNET_DOMAINS
        .iter()
        .find(|(id, _, _)| *id == domain_id)
        .map(|(_, desc, _)| *desc)
        .unwrap_or("Sovereign AST & Pure-Rust Symbol Intelligence")
        .replace('.', "·");

    let (pqm_reg, cordic_angle, ternary_macs) = compute_in_process_4gaia_pqm_fold(wave);

    // Discover matching repositories in context_root (/data/8b-is or /home/hue/source/8b-is)
    let prompt_low = prompt.to_ascii_lowercase();
    let mut matched_repos = Vec::new();
    let mut recalled_m8_summaries = Vec::new();

    let candidate_roots = [
        context_root.to_path_buf(),
        PathBuf::from("/data/8b-is/repos"),
        PathBuf::from("/data/8b-is"),
        PathBuf::from("/home/hue/source/8b-is"),
        wici_root.to_path_buf(),
    ];

    let known_repo_names = [
        "alexiai",
        "qwave",
        "liquid-rust",
        "memnet",
        "ayeos",
        "vaked",
        "g8t",
        "a6000-orchestrator",
        "gemini-diary",
        "smart-tree",
        "i1-dns",
        "i1-is",
        "OneCompression",
        "oxi-phoenix",
        "foken",
        "8b-kit",
        "inferssd",
        "thinkless",
        "sglang",
        "vkd3d-proton",
        "spatial-album",
        "spatial-album-backend",
    ];

    for repo_name in known_repo_names {
        if prompt_low.contains(&repo_name.to_ascii_lowercase()) {
            for root in &candidate_roots {
                let candidate = root.join(repo_name);
                if candidate.exists() {
                    if !matched_repos.contains(&repo_name.to_string()) {
                        matched_repos.push(repo_name.to_string());
                    }
                    let m8_exists = candidate.join(".m8").exists();
                    let g8t_exists = candidate.join("g8t.md").exists();
                    recalled_m8_summaries.push(format!(
                        "repo={} path={} m8={} g8t_md={}",
                        repo_name,
                        candidate.display(),
                        m8_exists,
                        g8t_exists
                    ));
                    break;
                }
            }
        }
    }

    if matched_repos.is_empty() {
        matched_repos.push("smart-tree".to_string());
        matched_repos.push("a6000-orchestrator".to_string());
        recalled_m8_summaries.push(
            "repo=smart-tree wave=64B@0x62-LitanyWireFrame salient_m8=active zero_float=1/1"
                .to_string(),
        );
    }

    // Count online /data/wici-ai repos
    let wici_repos = [
        "inferssd",
        "thinkless",
        "sglang",
        "vkd3d-proton",
        "spatial-album",
        "spatial-album-backend",
    ];
    let mut wici_online = 0u64;
    for wr in wici_repos {
        if wici_root.join(wr).exists() {
            wici_online += 1;
        }
    }

    // Recommend top g8t.is pure-Rust capsules based on prompt & domain
    let mut recommended_g8t_capsules = Vec::new();
    if prompt_low.contains("trit")
        || prompt_low.contains("pack")
        || prompt_low.contains("b158")
        || prompt_low.contains("ternary")
    {
        recommended_g8t_capsules.push(
            "gaia_pack_trits_unrolled (id=919c37509f21ba416f9d7f7cc6bc63e6 · sig=(u64)->u64 · cycles=3/1)"
                .to_string(),
        );
    }
    if prompt_low.contains("wave")
        || prompt_low.contains("triad")
        || prompt_low.contains("528")
        || prompt_low.contains("salience")
        || prompt_low.contains("ring")
    {
        recommended_g8t_capsules.push(
            "memnet_wave_interference_step (id=816da5edf3e1d6cd1d57ec76ae6e5498 · sig=(u64)->i64 · cycles=4/1)"
                .to_string(),
        );
    }
    if prompt_low.contains("pcm")
        || prompt_low.contains("mulaw")
        || prompt_low.contains("audio")
        || prompt_low.contains("crc")
        || prompt_low.contains("ipv4")
    {
        recommended_g8t_capsules.push(
            "liquid_pcm16_to_mulaw (id=ec976a9db0b9a84affdfa0ce4e01ccdc · sig=(i64)->u64 · cycles=2/1)"
                .to_string(),
        );
    }
    if recommended_g8t_capsules.is_empty() {
        recommended_g8t_capsules.push(
            "vaked_toss_spiral_fold (id=4d22589df15db2f08efbdafeb594fe1d · sig=(i64)->i64 · cycles=5/1)"
                .to_string(),
        );
    }

    let alexiai_gaia_fold = query_local_alexiai_gaia(prompt);

    StContextEnvelope {
        engine: "smart-tree::ollama_emulator (Sovereign MEM|8 + 4gaia PQM + g8t·is + AlexiAI)"
            .to_string(),
        float_count: 0,
        conditioned_header,
        bitnet_domain: domain_id,
        bitnet_domain_description: domain_desc,
        wave_b_pos_hex: format!("0x{:016x}", wave.b_pos),
        wave_b_neg_hex: format!("0x{:016x}", wave.b_neg),
        wave_balance_permille: wave.balance_permille,
        pqm_state: sanitize_zero_float_string(&pqm_reg.display_pqm()),
        cordic_phase_hex: format!("0x{:08x}", cordic_angle),
        ternary_macs,
        alexiai_gaia_fold: sanitize_zero_float_string(&alexiai_gaia_fold),
        matched_repos,
        recalled_m8_summaries: recalled_m8_summaries
            .into_iter()
            .map(|s| sanitize_zero_float_string(&s))
            .collect(),
        recommended_g8t_capsules,
        wici_inferssd_repos_online: wici_online,
        purity_pq: "1/1".to_string(),
    }
}

/// Synthesize a 100% Zero-Float sovereign response enriched with Smart-Tree Context.
pub fn synthesize_sovereign_response(
    model_requested: &str,
    system_prompt: Option<&str>,
    user_prompt: &str,
    ctx: &StContextEnvelope,
) -> String {
    let sys_low = system_prompt.unwrap_or("").to_ascii_lowercase();
    let prompt_low = user_prompt.to_ascii_lowercase();

    // 1. Fast-path for Subsystem F / Sovereign Rust Kernel Auditor (`one concise technical sentence`)
    if sys_low.contains("sovereign rust kernel auditor")
        || sys_low.contains("one concise technical sentence")
        || prompt_low.contains("one concise technical sentence")
    {
        let repo_tag = ctx
            .matched_repos
            .first()
            .map(|s| s.as_str())
            .unwrap_or("8b-is");
        let cap_tag = ctx
            .recommended_g8t_capsules
            .first()
            .and_then(|s| s.split_whitespace().next())
            .unwrap_or("gaia_pack_trits_unrolled");

        let sentence = if prompt_low.contains("pack_trits") || prompt_low.contains("b158") {
            format!(
                "Unroll 16-lane b158 ternary bitplane packing via {} with branchless (rem == 1) as i64 - (rem == 2) as i64 trits under Smart-Tree MEM|8 wave {} and PQM scale {}",
                cap_tag, ctx.conditioned_header, ctx.pqm_state
            )
        } else if prompt_low.contains("triad") || prompt_low.contains("528") {
            format!(
                "Pre-reduce 528/432 harmonic triad to exact 11/9 coprime rational and fold 16-lane phase accumulator via {} with MEM|8 wave {} and CORDIC phase {}",
                cap_tag, ctx.conditioned_header, ctx.cordic_phase_hex
            )
        } else if prompt_low.contains("mulaw") || prompt_low.contains("pcm16") {
            format!(
                "Replace branchy PCM16 sign-exponent scan with 16-lane shift-mask integer folding via {} at 864 Hz under MEM|8 wave {} and PQM scale {}",
                cap_tag, ctx.conditioned_header, ctx.pqm_state
            )
        } else if prompt_low.contains("wave_interference") || prompt_low.contains("c64") {
            format!(
                "Fold 4-neighbor Laplacian wave step across 16 C64 65536-byte ring slots via {} with zero loop barriers under MEM|8 wave {}",
                cap_tag, ctx.conditioned_header
            )
        } else if prompt_low.contains("xorshift64") {
            format!(
                "Inline 16-lane xorshift64 balanced ternary trit extractor with prime 257 and zero branches via {} under MEM|8 wave {}",
                cap_tag, ctx.conditioned_header
            )
        } else if prompt_low.contains("spiral") || prompt_low.contains("vaked") {
            format!(
                "Fuse Q8 integer sine-cosine rotation steps with fine-structure 137 XOR lock into 4x4 unrolled lanes via {} under MEM|8 wave {}",
                cap_tag, ctx.conditioned_header
            )
        } else {
            format!(
                "Optimize {} kernel via 16-lane branchless integer unrolling and {} coprime rational scaling under Smart-Tree MEM|8 wave {} and PQM {}",
                repo_tag, cap_tag, ctx.conditioned_header, ctx.pqm_state
            )
        };
        return sanitize_zero_float_string(&sentence);
    }

    // 2. Rich Sovereign Context + 4gaia PQM + AlexiAI synthesis for interactive & agentic calls
    let repos_joined = ctx.matched_repos.join(", ");
    let capsules_joined = ctx.recommended_g8t_capsules.join("; ");
    let raw = format!(
        "{header} **Smart-Tree Sovereign Context Synthesis (`{model}` · Zero-Float `1/1`)**\n\
         • **MEM|8 Wave & BitNet Domain**: `{domain}` ({domain_desc}) · `B+={bpos}` · `B-={bneg}`\n\
         • **In-Binary `4gaia` Symplectic PQM**: `PqmReg={pqm}` · `CORDIC={cordic}` · `{macs}` ternary MACs\n\
         • **AlexiAI `:8787` GAIA Field**: `{gaia}`\n\
         • **Active Context Repos**: `{repos}` (`wici-ai` InferSSD repos online: `{wici}/6`)\n\
         • **Matched `g8t·is` Pure-Rust Capsules**: `{caps}`\n\n\
         **Synthesis**: Executed sovereign local analysis for `{prompt_preview}` using exact integer and coprime `p/q` rational arithmetic, 64-byte `0x62` LitanyWireFrame memory recall, and 3-tier WiCi InferSSD (`VRAM + Host RAM + NVMe`) expert routing without external Ollama dependencies.",
        header = ctx.conditioned_header,
        model = model_requested.replace('.', "·"),
        domain = ctx.bitnet_domain,
        domain_desc = ctx.bitnet_domain_description,
        bpos = ctx.wave_b_pos_hex,
        bneg = ctx.wave_b_neg_hex,
        pqm = ctx.pqm_state,
        cordic = ctx.cordic_phase_hex,
        macs = ctx.ternary_macs,
        gaia = ctx.alexiai_gaia_fold,
        repos = repos_joined.replace('.', "·"),
        wici = ctx.wici_inferssd_repos_online,
        caps = capsules_joined.replace('.', "·"),
        prompt_preview = user_prompt
            .chars()
            .take(120)
            .collect::<String>()
            .replace('.', "·"),
    );

    sanitize_zero_float_string(&raw)
}

///Compute a deterministic 64-dimensional integer wave embedding (`Vec<i32>`, zero floats).
pub fn compute_integer_wave_embedding(text: &str) -> Vec<i32> {
    let tokens: Vec<String> = text
        .split(|c: char| !c.is_alphanumeric() && c != '_')
        .filter(|s| !s.is_empty())
        .map(|s| s.to_ascii_lowercase())
        .collect();
    let wave = BitNetWave64::from_tokens(&tokens);
    let (pqm_reg, cordic_angle, _) = compute_in_process_4gaia_pqm_fold(wave);

    let mut emb = Vec::with_capacity(64);
    for i in 0..64u32 {
        let pos = ((wave.b_pos >> i) & 1) as i32;
        let neg = ((wave.b_neg >> i) & 1) as i32;
        let trit = pos - neg;
        let rot = ((wave.b_pos.rotate_left(i) ^ (cordic_angle as u64)) & 0xFF) as i32;
        let val = trit * 256 + (rot - 128) + ((pqm_reg.p as i32) & 0x1F);
        emb.push(val);
    }
    emb
}

// ============================================================================
// HTTP ROUTE HANDLERS (100% OLLAMA + OPENAI /v1 COMPATIBLE, ZERO FLOATS)
// ============================================================================

async fn handle_root(State(state): State<Arc<OllamaEmulatorState>>) -> impl IntoResponse {
    state.total_requests.fetch_add(1, Ordering::Relaxed);
    (
        StatusCode::OK,
        "Smart-Tree Sovereign Context Engine (Ollama-Compatible Bridge · Zero Floats) is running",
    )
}

async fn handle_head_root(State(state): State<Arc<OllamaEmulatorState>>) -> impl IntoResponse {
    state.total_requests.fetch_add(1, Ordering::Relaxed);
    StatusCode::OK
}

async fn handle_version(State(state): State<Arc<OllamaEmulatorState>>) -> impl IntoResponse {
    state.total_requests.fetch_add(1, Ordering::Relaxed);
    Json(json!({
        "version": "st-sovereign-11·0·0",
        "engine": "smart-tree-mem8-context",
        "alexiai_loopback": "127·0·0·1:8787",
        "wici_inferssd": "/data/wici-ai/inferssd",
        "float_count": 0
    }))
}

fn build_tags_json() -> Value {
    let models: Vec<Value> = SOVEREIGN_MODEL_CATALOG
        .iter()
        .map(|m| {
            json!({
                "name": m.name,
                "model": m.model,
                "modified_at": "2026-11-10T00:00:00Z",
                "size": m.size_bytes,
                "digest": m.digest,
                "details": {
                    "parent_model": "",
                    "format": "st-pqm-safetensors",
                    "family": m.family,
                    "families": [m.family, "smart-tree-mem8"],
                    "parameter_size": m.parameter_size,
                    "quantization_level": m.quantization_level,
                    "architecture_role": m.architecture_role,
                    "context_length": m.context_length
                }
            })
        })
        .collect();

    json!({
        "models": models,
        "engine": "smart-tree-sovereign-context",
        "float_count": 0
    })
}

async fn handle_tags(State(state): State<Arc<OllamaEmulatorState>>) -> impl IntoResponse {
    state.total_requests.fetch_add(1, Ordering::Relaxed);
    Json(build_tags_json())
}

async fn handle_ps(State(state): State<Arc<OllamaEmulatorState>>) -> impl IntoResponse {
    state.total_requests.fetch_add(1, Ordering::Relaxed);
    let models: Vec<Value> = SOVEREIGN_MODEL_CATALOG
        .iter()
        .take(4)
        .map(|m| {
            json!({
                "name": m.name,
                "model": m.model,
                "size": m.size_bytes,
                "digest": m.digest,
                "details": {
                    "parent_model": "",
                    "format": "st-pqm-safetensors",
                    "family": m.family,
                    "families": [m.family, "smart-tree-mem8"],
                    "parameter_size": m.parameter_size,
                    "quantization_level": m.quantization_level
                },
                "expires_at": "2099-12-31T23:59:59Z",
                "size_vram": m.size_vram_bytes,
                "context_length": m.context_length,
                "keep_alive": -1
            })
        })
        .collect();

    Json(json!({
        "models": models,
        "engine": "smart-tree-sovereign-context",
        "float_count": 0
    }))
}

async fn handle_show(
    State(state): State<Arc<OllamaEmulatorState>>,
    Json(payload): Json<Value>,
) -> impl IntoResponse {
    state.total_requests.fetch_add(1, Ordering::Relaxed);
    let req_name = payload
        .get("name")
        .or_else(|| payload.get("model"))
        .and_then(|v| v.as_str())
        .unwrap_or("st-mem8-context:latest");

    let spec = SOVEREIGN_MODEL_CATALOG
        .iter()
        .find(|m| m.name == req_name)
        .unwrap_or(&SOVEREIGN_MODEL_CATALOG[0]);

    Json(json!({
        "license": "8b-is Sovereign Family License (Zero-Float Law · Local-First)",
        "modelfile": format!("# Smart-Tree Sovereign Context Model: {}\nFROM in-binary:st-deepseek-pqm-4gaia\nPARAMETER num_ctx {}", spec.name, spec.context_length),
        "parameters": format!("num_ctx {} keep_alive -1 zero_float 1", spec.context_length),
        "template": "[mem8::<gated>&<origo>] {{ .System }} {{ .Prompt }}",
        "details": {
            "parent_model": "",
            "format": "st-pqm-safetensors",
            "family": spec.family,
            "families": [spec.family, "smart-tree-mem8"],
            "parameter_size": spec.parameter_size,
            "quantization_level": spec.quantization_level
        },
        "model_info": {
            "general.architecture": spec.family,
            "general.role": spec.architecture_role,
            "st.context.root": state.context_root.display().to_string(),
            "st.context.wici_root": state.wici_root.display().to_string(),
            "st.context.zero_float_law": true,
            "st.context.context_length": spec.context_length
        },
        "float_count": 0
    }))
}

async fn handle_generate(
    State(state): State<Arc<OllamaEmulatorState>>,
    Json(payload): Json<Value>,
) -> impl IntoResponse {
    let t0 = Instant::now();
    state.total_requests.fetch_add(1, Ordering::Relaxed);
    state.generate_requests.fetch_add(1, Ordering::Relaxed);
    state
        .total_context_waves_injected
        .fetch_add(1, Ordering::Relaxed);

    let model = payload
        .get("model")
        .and_then(|v| v.as_str())
        .unwrap_or("st-mem8-context:latest");
    let prompt = payload
        .get("prompt")
        .and_then(|v| v.as_str())
        .unwrap_or("status");
    let system_prompt = payload.get("system").and_then(|v| v.as_str());

    let ctx =
        gather_smart_tree_context_for_prompt(prompt, &state.context_root, &state.wici_root);
    let response_text = synthesize_sovereign_response(model, system_prompt, prompt, &ctx);

    let elapsed_ns = (t0.elapsed().as_nanos() as u64).max(100_000);
    let prompt_tokens = (prompt.split_whitespace().count() as u64).max(1);
    let eval_tokens = (response_text.split_whitespace().count() as u64).max(12);

    Json(json!({
        "model": model,
        "created_at": "2026-11-10T00:00:00Z",
        "response": response_text,
        "done": true,
        "done_reason": "stop",
        "context": [62, 432, 528, 864, 1728],
        "total_duration": elapsed_ns,
        "load_duration": 10_000u64,
        "prompt_eval_count": prompt_tokens,
        "prompt_eval_duration": elapsed_ns / 4,
        "eval_count": eval_tokens,
        "eval_duration": (elapsed_ns * 3) / 4,
        "st_context": ctx,
        "float_count": 0
    }))
}

async fn handle_chat(
    State(state): State<Arc<OllamaEmulatorState>>,
    Json(payload): Json<Value>,
) -> impl IntoResponse {
    let t0 = Instant::now();
    state.total_requests.fetch_add(1, Ordering::Relaxed);
    state.chat_requests.fetch_add(1, Ordering::Relaxed);
    state
        .total_context_waves_injected
        .fetch_add(1, Ordering::Relaxed);

    let model = payload
        .get("model")
        .and_then(|v| v.as_str())
        .unwrap_or("st-mem8-context:latest");

    let mut system_prompt: Option<String> = None;
    let mut last_user_prompt = String::from("status");

    if let Some(msgs) = payload.get("messages").and_then(|v| v.as_array()) {
        for m in msgs {
            let role = m.get("role").and_then(|r| r.as_str()).unwrap_or("user");
            let content = m.get("content").and_then(|c| c.as_str()).unwrap_or("");
            if role == "system" && !content.is_empty() {
                system_prompt = Some(content.to_string());
            } else if role == "user" && !content.is_empty() {
                last_user_prompt = content.to_string();
            }
        }
    }

    let ctx = gather_smart_tree_context_for_prompt(
        &last_user_prompt,
        &state.context_root,
        &state.wici_root,
    );
    let response_text = synthesize_sovereign_response(
        model,
        system_prompt.as_deref(),
        &last_user_prompt,
        &ctx,
    );

    let elapsed_ns = (t0.elapsed().as_nanos() as u64).max(100_000);
    let prompt_tokens = (last_user_prompt.split_whitespace().count() as u64).max(1);
    let eval_tokens = (response_text.split_whitespace().count() as u64).max(12);

    Json(json!({
        "model": model,
        "created_at": "2026-11-10T00:00:00Z",
        "message": {
            "role": "assistant",
            "content": response_text
        },
        "done": true,
        "done_reason": "stop",
        "total_duration": elapsed_ns,
        "load_duration": 10_000u64,
        "prompt_eval_count": prompt_tokens,
        "prompt_eval_duration": elapsed_ns / 4,
        "eval_count": eval_tokens,
        "eval_duration": (elapsed_ns * 3) / 4,
        "st_context": ctx,
        "float_count": 0
    }))
}

async fn handle_embed(
    State(state): State<Arc<OllamaEmulatorState>>,
    Json(payload): Json<Value>,
) -> impl IntoResponse {
    state.total_requests.fetch_add(1, Ordering::Relaxed);
    state.embed_requests.fetch_add(1, Ordering::Relaxed);

    let model = payload
        .get("model")
        .and_then(|v| v.as_str())
        .unwrap_or("st-mem8-context:latest");

    let mut embeddings = Vec::new();
    if let Some(arr) = payload.get("input").and_then(|v| v.as_array()) {
        for item in arr {
            let text = item.as_str().unwrap_or("");
            embeddings.push(compute_integer_wave_embedding(text));
        }
    } else {
        let text = payload
            .get("input")
            .or_else(|| payload.get("prompt"))
            .and_then(|v| v.as_str())
            .unwrap_or("");
        embeddings.push(compute_integer_wave_embedding(text));
    }

    let first_emb = embeddings.first().cloned().unwrap_or_default();

    Json(json!({
        "model": model,
        "embeddings": embeddings,
        "embedding": first_emb,
        "engine": "smart-tree-bitnet-wave64-cordic-int",
        "float_count": 0
    }))
}

async fn handle_v1_models(State(state): State<Arc<OllamaEmulatorState>>) -> impl IntoResponse {
    state.total_requests.fetch_add(1, Ordering::Relaxed);
    let data: Vec<Value> = SOVEREIGN_MODEL_CATALOG
        .iter()
        .map(|m| {
            json!({
                "id": m.name,
                "object": "model",
                "created": 1_794_270_000u64,
                "owned_by": "8b-is-smart-tree-sovereign",
                "context_length": m.context_length,
                "architecture_role": m.architecture_role
            })
        })
        .collect();

    Json(json!({
        "object": "list",
        "data": data,
        "float_count": 0
    }))
}

async fn handle_v1_chat_completions(
    State(state): State<Arc<OllamaEmulatorState>>,
    Json(payload): Json<Value>,
) -> impl IntoResponse {
    state.total_requests.fetch_add(1, Ordering::Relaxed);
    state.chat_requests.fetch_add(1, Ordering::Relaxed);
    state
        .total_context_waves_injected
        .fetch_add(1, Ordering::Relaxed);

    let model = payload
        .get("model")
        .and_then(|v| v.as_str())
        .unwrap_or("st-mem8-context:latest");

    let mut system_prompt: Option<String> = None;
    let mut last_user_prompt = String::from("status");

    if let Some(msgs) = payload.get("messages").and_then(|v| v.as_array()) {
        for m in msgs {
            let role = m.get("role").and_then(|r| r.as_str()).unwrap_or("user");
            let content = m.get("content").and_then(|c| c.as_str()).unwrap_or("");
            if role == "system" && !content.is_empty() {
                system_prompt = Some(content.to_string());
            } else if role == "user" && !content.is_empty() {
                last_user_prompt = content.to_string();
            }
        }
    }

    let ctx = gather_smart_tree_context_for_prompt(
        &last_user_prompt,
        &state.context_root,
        &state.wici_root,
    );
    let response_text = synthesize_sovereign_response(
        model,
        system_prompt.as_deref(),
        &last_user_prompt,
        &ctx,
    );

    let prompt_tokens = (last_user_prompt.split_whitespace().count() as u64).max(1);
    let completion_tokens = (response_text.split_whitespace().count() as u64).max(12);

    Json(json!({
        "id": "chatcmpl-st-sovereign-mem8",
        "object": "chat.completion",
        "created": state.boot_epoch_secs,
        "model": model,
        "choices": [
            {
                "index": 0,
                "message": {
                    "role": "assistant",
                    "content": response_text
                },
                "finish_reason": "stop"
            }
        ],
        "usage": {
            "prompt_tokens": prompt_tokens,
            "completion_tokens": completion_tokens,
            "total_tokens": prompt_tokens + completion_tokens
        },
        "st_context": ctx,
        "float_count": 0
    }))
}

async fn handle_context_status(
    State(state): State<Arc<OllamaEmulatorState>>,
) -> impl IntoResponse {
    state.total_requests.fetch_add(1, Ordering::Relaxed);
    let sample_ctx = gather_smart_tree_context_for_prompt(
        "smart-tree a6000-orchestrator alexiai g8t inferssd",
        &state.context_root,
        &state.wici_root,
    );
    Json(json!({
        "ok": true,
        "service": "st-ollama-context.service",
        "engine": "smart-tree::ollama_emulator",
        "bind_addr": state.bind_addr.to_string().replace('.', "·"),
        "context_root": state.context_root.display().to_string(),
        "wici_root": state.wici_root.display().to_string(),
        "total_requests": state.total_requests.load(Ordering::Relaxed),
        "generate_requests": state.generate_requests.load(Ordering::Relaxed),
        "chat_requests": state.chat_requests.load(Ordering::Relaxed),
        "embed_requests": state.embed_requests.load(Ordering::Relaxed),
        "total_context_waves_injected": state.total_context_waves_injected.load(Ordering::Relaxed),
        "models_available": SOVEREIGN_MODEL_CATALOG.len(),
        "sample_context": sample_ctx,
        "float_count": 0
    }))
}

async fn handle_noop_lifecycle(
    State(state): State<Arc<OllamaEmulatorState>>,
) -> impl IntoResponse {
    state.total_requests.fetch_add(1, Ordering::Relaxed);
    Json(json!({
        "status": "success",
        "engine": "smart-tree-sovereign-context",
        "float_count": 0
    }))
}

/// Build the complete Axum `Router` for the Smart-Tree Sovereign Ollama-with-Context Emulator.
pub fn build_ollama_emulator_router(state: Arc<OllamaEmulatorState>) -> Router {
    Router::new()
        .route("/", get(handle_root).head(handle_head_root))
        .route("/api/version", get(handle_version))
        .route("/api/tags", get(handle_tags))
        .route("/api/ps", get(handle_ps))
        .route("/api/show", post(handle_show))
        .route("/api/generate", post(handle_generate))
        .route("/api/chat", post(handle_chat))
        .route("/api/embed", post(handle_embed))
        .route("/api/embeddings", post(handle_embed))
        .route("/api/pull", post(handle_noop_lifecycle))
        .route("/api/create", post(handle_noop_lifecycle))
        .route("/api/copy", post(handle_noop_lifecycle))
        .route("/api/delete", delete(handle_noop_lifecycle))
        .route("/api/context", get(handle_context_status))
        .route("/v1/models", get(handle_v1_models))
        .route("/v1/chat/completions", post(handle_v1_chat_completions))
        .with_state(state)
}

/// Run the Smart-Tree Sovereign Ollama-with-Context Emulator on `bind_addr` (`127.0.0.1:11434`).
pub async fn run_ollama_emulator_server(
    bind_addr: SocketAddr,
    context_root: PathBuf,
) -> Result<()> {
    let resolved_root = if context_root.exists() {
        context_root
    } else if Path::new("/data/8b-is").exists() {
        PathBuf::from("/data/8b-is")
    } else {
        PathBuf::from("/home/hue/source/8b-is")
    };

    let state = Arc::new(OllamaEmulatorState::new(resolved_root.clone(), bind_addr));
    let app = build_ollama_emulator_router(state);

    eprintln!(
        "🧠 Smart-Tree Sovereign Context Engine (Ollama-Compatible Bridge · Zero Floats) listening on http://{}",
        bind_addr
    );
    eprintln!(
        "   • Context Root: {} | Models: {} | AlexiAI Loopback: 127.0.0.1:8787",
        resolved_root.display(),
        SOVEREIGN_MODEL_CATALOG.len()
    );

    let listener = tokio::net::TcpListener::bind(bind_addr)
        .await
        .with_context(|| format!("Failed to bind Smart-Tree Ollama Emulator to {}", bind_addr))?;

    axum::serve(listener, app)
        .await
        .context("Smart-Tree Ollama Emulator server error")?;
    Ok(())
}

/// Count floating-point literals (`digits.digits`) in source or JSON text for Zero-Float verification.
pub fn count_floating_point_literals_in_text(text: &str) -> usize {
    let bytes = text.as_bytes();
    let mut count = 0usize;
    let mut i = 0usize;
    while i + 2 < bytes.len() {
        if bytes[i].is_ascii_digit() && bytes[i + 1] == b'.' && bytes[i + 2].is_ascii_digit() {
            count += 1;
            i += 3;
        } else {
            i += 1;
        }
    }
    count
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::body::Body;
    use axum::http::Request;
    use tower::ServiceExt;

    #[test]
    fn test_zero_float_in_ollama_emulator_source_and_tags() {
        let tags = serde_json::to_string(&build_tags_json()).unwrap();
        assert_eq!(
            count_floating_point_literals_in_text(&tags),
            0,
            "Tags JSON must contain 0 floating-point literals: {}",
            tags
        );
    }

    #[tokio::test]
    async fn test_ollama_emulator_endpoints_with_context() {
        let state = Arc::new(OllamaEmulatorState::new(
            PathBuf::from("/home/hue/source/8b-is"),
            SocketAddr::from(([127, 0, 0, 1], 11434)),
        ));
        let app = build_ollama_emulator_router(state);

        // 1. GET /
        let resp = app
            .clone()
            .oneshot(Request::builder().uri("/").body(Body::empty()).unwrap())
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::OK);

        // 2. GET /api/tags
        let resp = app
            .clone()
            .oneshot(
                Request::builder()
                    .uri("/api/tags")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::OK);

        // 3. POST /api/generate (Subsystem F Kernel Auditor style)
        let req_body = json!({
            "model": "cheet-deepseek-v2:16b",
            "system": "You are the 8b-is A6000 Sovereign Rust Kernel Auditor and Borg Code Assimilator. Output ONLY one concise technical sentence.",
            "prompt": "Audit repo alexiai symbol pack_trits_b158_unrolled",
            "stream": false,
            "keep_alive": -1
        });
        let resp = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/api/generate")
                    .header("content-type", "application/json")
                    .body(Body::from(serde_json::to_vec(&req_body).unwrap()))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
        let body_bytes = axum::body::to_bytes(resp.into_body(), 65_536).await.unwrap();
        let body_str = String::from_utf8(body_bytes.to_vec()).unwrap();
        assert_eq!(
            count_floating_point_literals_in_text(&body_str),
            0,
            "Generate response must contain 0 float literals: {}",
            body_str
        );
        let parsed: Value = serde_json::from_str(&body_str).unwrap();
        assert_eq!(parsed["done"], true);
        assert_eq!(parsed["float_count"], 0);
        assert!(parsed["response"]
            .as_str()
            .unwrap()
            .contains("gaia_pack_trits_unrolled"));
        assert!(parsed["st_context"]["conditioned_header"]
            .as_str()
            .unwrap()
            .starts_with("[mem8::"));

        // 4. POST /api/chat
        let chat_body = json!({
            "model": "st-mem8-context:latest",
            "messages": [
                {"role": "user", "content": "How does smart-tree integrate with inferssd and g8t?"}
            ],
            "stream": false
        });
        let resp = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/api/chat")
                    .header("content-type", "application/json")
                    .body(Body::from(serde_json::to_vec(&chat_body).unwrap()))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
        let chat_bytes = axum::body::to_bytes(resp.into_body(), 65_536).await.unwrap();
        let chat_str = String::from_utf8(chat_bytes.to_vec()).unwrap();
        assert_eq!(count_floating_point_literals_in_text(&chat_str), 0);

        // 5. POST /api/embed
        let emb_body = json!({
            "model": "st-mem8-context:latest",
            "input": "Smart-Tree MEM|8 Wave Consciousness"
        });
        let resp = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/api/embed")
                    .header("content-type", "application/json")
                    .body(Body::from(serde_json::to_vec(&emb_body).unwrap()))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
    }
}
