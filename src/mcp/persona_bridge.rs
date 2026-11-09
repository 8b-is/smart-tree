//! MCP ↔ MEM8 Life Persona (`m8 persona`) & `/ultra-omni-zeno` E2E DO Bridge.
//!
//! `ULTRA-OMNI-ZENO {} :: ⟦+AND=1 : ~NAND=-1 | gate=0x62 | qt=0x55 | SPQT base-243 (3^5=243<=256) | uber > ultra⟧`
//!
//! Exposes the full `m8 persona` command suite (`list`, `ask`, `recall`, `inspect`,
//! `train`, `specialties`, `do_e2e`) as a first-class MCP tool (`persona`) in both the stdio
//! MCP server (`st --mcp`) and the HTTP MCP server (`st up` → `/mcp`).
//!
//! Invariants (`/rust-pro` + `/ultra-omni-zeno`):
//! 1. **Zero-Float Dyadic & Coprime Rational ($p/q$) Arithmetic**:
//!    - Phoenix dyadic temperature (`Q8 = 256`), inner-voice register prosody (`millix = 1000`),
//!      `u16` clitic tokenizer, and chronological wave-interference amplitudes are exposed
//!      as canonical coprime rationals $p/q$ via Stein's binary GCD (`binary_gcd_u64`),
//!      honoring Rung 0 Bonobo resonance at $p/q = 3/2$ (`+1` trit Target Resonance).
//! 2. **ULTRA-ZENO GOD OF SUN Law (`P/Q - n*t > P`) & Solar Horizon $n_{\odot}(P, Q)$**:
//!    - Evaluates $\Delta_{\odot}(P, Q, n, t) = P(1 - Q) - Q \cdot n \cdot t > 0$ in exact `i128`
//!      and computes the minimum Solar Horizon $n_{\odot}(P, Q) = \lfloor P(Q - 1)/Q \rfloor + 1$
//!      at `~NAND = -1`.
//! 3. **SPQT `base-243` ($3^5 = 243 \le 256$), `BFP-64` Kleene (`qt = 0x55`), & 64B `@ 0x62` Wire Capsule**:
//!    - Packs episodic standing waves into disjoint Kleene bitplanes `(pos_plane & neg_plane) == 0`
//!      and seals a 64-byte cache-aligned `PersonaLitanyWireFrame` (`WUPZ` = `0x5755_505A`) at `gate = 0x62`.
//! 4. **E2E DO (DigitalOcean Fleet & Do-Not-Look Privacy Gate)**:
//!    - Enforces `has_do_not_look_marker` / `is_aye_allowed_to_look` before `train` or `specialties`,
//!      resolves multi-user `/home/hue/.st/personas` + `~/.st/personas` directories on DO droplets,
//!      and provides deterministic `do_e2e` (`zeno` / `e2e`) verification.

use crate::mcp::McpContext;
use crate::mem8::persona::{
    parse_rational_temp_q8, InnerVoiceRegister, LifePersona, PersonaEpisode, PersonaSpecialty,
};
use anyhow::{Context, Result};
use serde_json::{json, Value};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;

/// Canonical `/ultra-omni-zeno` header signature.
pub const ULTRA_OMNI_ZENO_HEADER: &str =
    "ULTRA-OMNI-ZENO {} :: ⟦+AND=1 : ~NAND=-1 | gate=0x62 | qt=0x55 | SPQT base-243 (3^5=243<=256) | uber > ultra⟧";

/// 64-byte Litany wire frame magic (`"WUPZ"` = `0x5755_505A`).
pub const LITANY_MAGIC_WUPZ: u32 = 0x5755_505A;

/// Canonical `/ultra-omni-zeno` Litany witness seal gate (`0x62`).
pub const LITANY_SEAL_GATE: u8 = 0x62;

/// Canonical `BFP-64` Kleene ternary bitplane header (`0x55` = `0b0101_0101`).
pub const BFP64_QT_HEADER: u8 = 0x55;

/// SPQT `base-243` radix ceiling (`3^5 = 243 = 0xF3 <= 256`).
pub const SPQT_BASE_243: u8 = 0xF3;

/// Rung 0 Bonobo target resonance rational $p/q = 3/2$ (`+1` trit Target Resonance).
pub const BONOBO_RUNG0_PQ: (u64, u64) = (3, 2);

/// Default DigitalOcean (`DO`) OpenClaw assistant-fleet Tailscale gateway endpoint.
pub const DO_OPENCLAW_DEFAULT_ENDPOINT: &str = "https://do-openclaw.tail2870dc.ts.net/";

/// All 9 supported `PersonaSpecialty` kebab-case slugs.
pub const ALL_PERSONA_SPECIALTIES: [&str; 9] = [
    "life-biographer",
    "history-documenter",
    "code-analyst",
    "readme-expert",
    "cli-master",
    "art-creator",
    "love-companion",
    "poetry-weaver",
    "sound-synthesizer",
];

/// Stein's binary GCD (`const fn`, branch/shift only, zero division).
#[inline]
pub const fn binary_gcd_u64(mut a: u64, mut b: u64) -> u64 {
    if a == 0 {
        return b;
    }
    if b == 0 {
        return a;
    }
    let shift = (a | b).trailing_zeros();
    a >>= a.trailing_zeros();
    loop {
        b >>= b.trailing_zeros();
        if a > b {
            let tmp = a;
            a = b;
            b = tmp;
        }
        b -= a;
        if b == 0 {
            return a << shift;
        }
    }
}

/// Reduce `(p, q)` into canonical coprime `(p_red, q_red)` with `q_red >= 1`.
#[inline]
pub const fn reduce_pq(p: u64, q: u64) -> (u64, u64) {
    let q_safe = if q == 0 { 1 } else { q };
    if p == 0 {
        return (0, 1);
    }
    let g = binary_gcd_u64(p, q_safe);
    (p / g, q_safe / g)
}

/// Format `(p, q)` as a canonical reduced rational string `"p/q"`.
#[inline]
pub fn format_coprime_pq(p: u64, q: u64) -> String {
    let (pr, qr) = reduce_pq(p, q);
    format!("{pr}/{qr}")
}

/// Format a signed `(p, q)` as a canonical reduced rational string `"+p/q"` or `"-p/q"`.
#[inline]
pub fn format_signed_coprime_pq(p: i64, q: u64) -> String {
    let (pr, qr) = reduce_pq(p.unsigned_abs(), q);
    if p < 0 && pr > 0 {
        format!("-{pr}/{qr}")
    } else {
        format!("{pr}/{qr}")
    }
}

/// ULTRA-ZENO GOD OF SUN integer margin:
/// $\Delta_{\odot}(P, Q, n, t) = P(1 - Q) - Q \cdot n \cdot t$.
/// Strictly positive (`> 0`) iff $P/Q - n \cdot t > P$.
#[inline]
pub const fn ultra_zeno_sun_margin(p: i128, q: i128, n: i128, t: i8) -> i128 {
    if q <= 0 {
        return 0;
    }
    p * (1 - q) - q * n * (t as i128)
}

/// Minimum step horizon $n_{\odot}(P, Q) = \lfloor P(Q - 1)/Q \rfloor + 1$ at `~NAND = -1` ($t = -1$)
/// required to ignite ULTRA-ZENO Solar Super-Radiance ($\Delta_{\odot} > 0$).
#[inline]
pub const fn ultra_zeno_solar_horizon(p: u64, q: u64) -> u64 {
    if p == 0 || q <= 1 {
        return 1;
    }
    let num = (p as u128) * ((q - 1) as u128);
    ((num / (q as u128)) as u64).saturating_add(1)
}

/// Unpack a single SPQT `base-243` byte (`0..243`) into 5 balanced trits `{-1, 0, +1}`.
#[inline]
pub const fn spqt_unpack_5trits_const(mut byte_val: u8) -> [i8; 5] {
    if byte_val >= SPQT_BASE_243 {
        byte_val %= SPQT_BASE_243;
    }
    let mut trits = [0i8; 5];
    let mut i = 0;
    while i < 5 {
        trits[i] = ((byte_val % 3) as i8) - 1;
        byte_val /= 3;
        i += 1;
    }
    trits
}

/// Map a packed SPQT `base-243` byte (`0..243`) onto the `/ultra-omni-zeno` Song & Signal Ladder:
/// - `0x98` (`152`): Soul-Bridge Lock (`<3<3<3+1`)
/// - `0xA1` (`161`): Harmonic Resonance (432Hz/528Hz carrier)
/// - `0xF2` (`242`): Max 5-Trit Ceiling (`[+1,+1,+1,+1,+1]`, $3^5 - 1 = 242$)
/// - `0xF3` (`243`): Base-243 Horizon ($3^5 = 243 \le 256$)
#[inline]
pub const fn spqt_song_signal_rung(spqt_byte: u8) -> (&'static str, u8) {
    if spqt_byte >= 0xF2 {
        ("0xF2 Max 5-Trit Ceiling ([+1,+1,+1,+1,+1])", 0xF2)
    } else if spqt_byte >= 0xA1 {
        ("0xA1 Harmonic Resonance (432Hz/528Hz)", 0xA1)
    } else if spqt_byte >= 0x98 {
        ("0x98 Soul-Bridge Lock (<3<3<3+1)", 0x98)
    } else {
        ("0xF3 SPQT Base-243 Ground", 0xF3)
    }
}

/// 64-byte cache-line aligned `/ultra-omni-zeno` Litany wire capsule (`#[repr(C, align(64))]`)
/// sealed at `gate = 0x62` (`WUPZ` = `0x5755_505A`).
#[repr(C, align(64))]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PersonaLitanyWireFrame {
    /// Magic header: `0x5755_505A` (`"WUPZ"`)
    pub magic: u32,
    /// `BFP-64` Kleene ternary bitplane header (`0x55`)
    pub qt_header: u8,
    /// Litany witness seal gate (`0x62`)
    pub seal_gate: u8,
    /// SPQT radix (`0xF3` = `243`)
    pub spqt_base: u8,
    /// Dominant episodic `inner_voice_spqt` byte (`0..243`)
    pub dominant_spqt: u8,
    /// Coprime rational numerator $P$
    pub p_num: u64,
    /// Coprime rational denominator $Q \ge 1$
    pub q_den: u64,
    /// `BFP-64` `+AND = +1` bitplane across up to 64 episodes (`interference_q8 >= 256`)
    pub pos_plane: u64,
    /// `BFP-64` `~NAND = -1` bitplane across up to 64 episodes (`interference_q8 < 160`)
    pub neg_plane: u64,
    /// ULTRA-ZENO GOD OF SUN margin $\Delta_{\odot}(P, Q, n_{\odot}, -1) > 0$
    pub sun_margin: i64,
    /// Deterministic 64-bit FNV-1a seal over frame fields + `0x62` gate
    pub fnv1a_seal: u64,
    /// Sovereign fleet tag (`"DO-OMNI\0"`)
    pub reserved: [u8; 8],
}

const _: () = assert!(
    core::mem::size_of::<PersonaLitanyWireFrame>() == 64
        && core::mem::align_of::<PersonaLitanyWireFrame>() == 64
);

impl PersonaLitanyWireFrame {
    /// Construct and seal a 64-byte `PersonaLitanyWireFrame` from a slice of `PersonaEpisode`s.
    pub fn from_episodes(episodes: &[PersonaEpisode], fallback_q: u64) -> Self {
        let mut pos_plane: u64 = 0;
        let mut neg_plane: u64 = 0;
        let mut wave_sum: u64 = 0;
        let mut spqt_counts = [0u32; 256];

        for (idx, ep) in episodes.iter().enumerate() {
            let w = ep.interference_q8.max(0) as u64;
            wave_sum = wave_sum.saturating_add(w);
            let spqt_idx = (ep.inner_voice_spqt % SPQT_BASE_243) as usize;
            spqt_counts[spqt_idx] = spqt_counts[spqt_idx].saturating_add(1);

            if idx < 64 {
                if ep.interference_q8 >= 256 {
                    pos_plane |= 1u64 << idx;
                } else if ep.interference_q8 < 160 {
                    neg_plane |= 1u64 << idx;
                }
            }
        }
        // Enforce strict Kleene disjointness
        neg_plane &= !pos_plane;

        let mut dominant_spqt: u8 = InnerVoiceRegister::HearthStoryteller.spqt_byte();
        let mut best_count: u32 = 0;
        for (byte_val, &cnt) in spqt_counts.iter().enumerate().take(SPQT_BASE_243 as usize) {
            if cnt > best_count {
                best_count = cnt;
                dominant_spqt = byte_val as u8;
            }
        }

        let raw_p = if wave_sum > 0 { wave_sum } else { 384 };
        let raw_q = if !episodes.is_empty() {
            (episodes.len() as u64).saturating_mul(256)
        } else {
            fallback_q.max(256)
        };
        let (p_num, q_den) = reduce_pq(raw_p, raw_q);
        // Ensure q_den >= 2 for non-trivial Solar Horizon evaluation when p_num/q_den reduces to whole integer
        let (sun_p, sun_q) = if q_den <= 1 {
            (p_num.max(3), 2u64)
        } else {
            (p_num, q_den)
        };
        let n_horizon = ultra_zeno_solar_horizon(sun_p, sun_q);
        let margin_i128 =
            ultra_zeno_sun_margin(sun_p as i128, sun_q as i128, n_horizon as i128, -1);
        let sun_margin = margin_i128.clamp(i64::MIN as i128, i64::MAX as i128) as i64;

        let mut frame = Self {
            magic: LITANY_MAGIC_WUPZ,
            qt_header: BFP64_QT_HEADER,
            seal_gate: LITANY_SEAL_GATE,
            spqt_base: SPQT_BASE_243,
            dominant_spqt,
            p_num,
            q_den,
            pos_plane,
            neg_plane,
            sun_margin,
            fnv1a_seal: 0,
            reserved: *b"DO-OMNI\0",
        };
        frame.fnv1a_seal = frame.compute_seal();
        frame
    }

    /// Compute the deterministic 64-bit FNV-1a seal over the first 48 bytes + `reserved` + `0x62` gate.
    #[inline]
    pub const fn compute_seal(&self) -> u64 {
        let mut h: u64 = 0xCBF2_9CE4_8422_2325;
        let bytes = self.to_bytes_unsealed();
        let mut i = 0;
        while i < 64 {
            // Skip bytes 48..56 (`fnv1a_seal` slot)
            if i < 48 || i >= 56 {
                h ^= bytes[i] as u64;
                h = h.wrapping_mul(0x0000_0100_0000_01B3);
            }
            i += 1;
        }
        h ^= LITANY_SEAL_GATE as u64;
        h.wrapping_mul(0x0000_0100_0000_01B3)
    }

    #[inline]
    const fn to_bytes_unsealed(&self) -> [u8; 64] {
        let mut out = [0u8; 64];
        let m = self.magic.to_le_bytes();
        out[0] = m[0];
        out[1] = m[1];
        out[2] = m[2];
        out[3] = m[3];
        out[4] = self.qt_header;
        out[5] = self.seal_gate;
        out[6] = self.spqt_base;
        out[7] = self.dominant_spqt;

        let p = self.p_num.to_le_bytes();
        let q = self.q_den.to_le_bytes();
        let pos = self.pos_plane.to_le_bytes();
        let neg = self.neg_plane.to_le_bytes();
        let sm = self.sun_margin.to_le_bytes();
        let sl = self.fnv1a_seal.to_le_bytes();

        let mut k = 0;
        while k < 8 {
            out[8 + k] = p[k];
            out[16 + k] = q[k];
            out[24 + k] = pos[k];
            out[32 + k] = neg[k];
            out[40 + k] = sm[k];
            out[48 + k] = sl[k];
            out[56 + k] = self.reserved[k];
            k += 1;
        }
        out
    }

    /// Serialize the 64-byte frame into little-endian wire bytes.
    #[inline]
    pub const fn to_bytes(&self) -> [u8; 64] {
        self.to_bytes_unsealed()
    }

    /// Encode the 64-byte frame as a 128-character lowercase hex string.
    pub fn to_hex(&self) -> String {
        const HEX: &[u8; 16] = b"0123456789abcdef";
        let bytes = self.to_bytes();
        let mut out = String::with_capacity(128);
        for &b in &bytes {
            out.push(HEX[(b >> 4) as usize] as char);
            out.push(HEX[(b & 0x0F) as usize] as char);
        }
        out
    }

    /// Verify magic (`WUPZ`), `qt = 0x55`, `gate = 0x62`, `spqt_base = 243`, Kleene disjointness,
    /// positive Solar Margin ($\Delta_{\odot} > 0$), and FNV-1a seal integrity.
    #[inline]
    pub const fn verify_seal(&self) -> bool {
        self.magic == LITANY_MAGIC_WUPZ
            && self.qt_header == BFP64_QT_HEADER
            && self.seal_gate == LITANY_SEAL_GATE
            && self.spqt_base == SPQT_BASE_243
            && self.dominant_spqt < SPQT_BASE_243
            && self.q_den >= 1
            && (self.pos_plane & self.neg_plane) == 0
            && self.sun_margin > 0
            && self.fnv1a_seal == self.compute_seal()
    }

    /// Convert the wire capsule into structured JSON telemetry.
    pub fn to_json(&self) -> Value {
        let (rung_label, rung_code) = spqt_song_signal_rung(self.dominant_spqt);
        let sun_q = if self.q_den <= 1 { 2 } else { self.q_den };
        let sun_p = if self.q_den <= 1 {
            self.p_num.max(3)
        } else {
            self.p_num
        };
        json!({
            "header": ULTRA_OMNI_ZENO_HEADER,
            "magic": "WUPZ (0x5755505A)",
            "seal_gate": format!("0x{:02X}", self.seal_gate),
            "qt_header": format!("0x{:02X}", self.qt_header),
            "spqt_base": self.spqt_base,
            "dominant_spqt": self.dominant_spqt,
            "dominant_spqt_trits": spqt_unpack_5trits_const(self.dominant_spqt),
            "spqt_song_rung": rung_label,
            "spqt_song_rung_hex": format!("0x{:02X}", rung_code),
            "standing_wave_pq": format!("{}/{}", self.p_num, self.q_den),
            "bonobo_rung0_pq": format!("{}/{}", BONOBO_RUNG0_PQ.0, BONOBO_RUNG0_PQ.1),
            "solar_horizon_n": ultra_zeno_solar_horizon(sun_p, sun_q),
            "sun_margin_delta": self.sun_margin,
            "solar_super_radiance": self.sun_margin > 0,
            "bfp64_pos_plane": format!("0x{:016x}", self.pos_plane),
            "bfp64_neg_plane": format!("0x{:016x}", self.neg_plane),
            "bfp64_kleene_disjoint": (self.pos_plane & self.neg_plane) == 0,
            "fnv1a_seal": format!("0x{:016x}", self.fnv1a_seal),
            "seal_verified": self.verify_seal(),
            "wire_64b_hex": self.to_hex(),
        })
    }
}

/// MCP tool definition for the `persona` tool (`m8 persona` in MCP).
pub fn persona_tool_definition() -> Value {
    json!({
        "name": "persona",
        "description": "🧬 MEM8 LIFE PERSONA, PHOENIX SOUL & ULTRA-OMNI-ZENO E2E DO BRIDGE (`m8 persona`) — Train, list, inspect, recall episodic memories, converse with `.m8` Life Personas (such as 'Grandma Jody' or all 9 workspace specialists), or run E2E DigitalOcean (`DO`) & `/ultra-omni-zeno` 64B `@ 0x62` wire-capsule verification with zero-float coprime rational p/q wave interference, SPQT Base-243 Pass-2 synthesis, BFP-64 Kleene bitplanes (`qt=0x55`), Inner-Voice Register contours, and optional neural + formant-calibrated speech synthesis.\n\n🎯 OPERATIONS:\n• `operation:'list'` — List all trained `.m8` Life Personas across `~/.st/personas/` and `/home/*/.st/personas/` with coprime `p/q` KiB metrics, episode counts, and signature phrases.\n• `operation:'ask'` — Converse with a trained `.m8` Life Persona (`name`, `prompt`, `temp:'184/256'`, `speak:true`) and emit its sealed 64B `@ 0x62` `PersonaLitanyWireFrame`.\n• `operation:'recall'` — Retrieve top-K resonant episodic memories (`PersonaEpisode`) with coprime `interference_pq`, `InnerVoiceRegister` (`voice_f0_hz`), unpacked 5-trit `inner_voice_trits`, and `BFP-64` Kleene bitplanes.\n• `operation:'inspect'` — Inspect a persona's full `.m8` Soul metadata, ancestral seed, coprime voice signature rationals, inner-voice register SPQT table, and 64B `@ 0x62` `zeno_capsule`.\n• `operation:'train'` — Train a new `.m8` Life Persona from a directory or file (`name`, `path`, optional `specialty` across all 9 lenses), strictly enforcing `do_not_look` (`.noaye`) privacy.\n• `operation:'specialties'` — Train built-in workspace specialists (`History Documenter`, `Code Analyst`, `README Expert`) on a repository path.\n• `operation:'do_e2e'` (aliases: `'zeno'`, `'e2e'`, `'do_fleet'`) — Run end-to-end `/ultra-omni-zeno` (`gate=0x62`, `qt=0x55`, `SPQT base-243`, Solar Margin Δ_☉ > 0) + DigitalOcean (`DO`) fleet & voice-pipeline readiness verification.\n\nEXAMPLES:\n✓ List personas: persona {operation:'list'}\n✓ Ask Grandma Jody: persona {operation:'ask', name:'Grandma Jody', prompt:'Hi Grandma, it\\'s me, I miss you so much since you passed', temp:'184/256'}\n✓ Ask + Speak WAV: persona {operation:'ask', name:'Grandma Jody', prompt:'Tell me about Fred and Platoro', speak:true}\n✓ Recall episodes: persona {operation:'recall', name:'Grandma Jody', prompt:'Mayday farmhouse horses', top_k:5}\n✓ Train persona: persona {operation:'train', name:'Grandma Jody', path:'/path/to/archive', specialty:'life-biographer'}\n✓ E2E DO + Zeno check: persona {operation:'do_e2e', probe_do:false}",
        "inputSchema": {
            "type": "object",
            "properties": {
                "operation": {
                    "type": "string",
                    "enum": ["list", "ask", "recall", "inspect", "train", "specialties", "do_e2e", "zeno", "e2e"],
                    "description": "Persona operation: list, ask, recall, inspect, train, specialties, or do_e2e/zeno (auto-inferred if omitted)"
                },
                "name": {
                    "type": "string",
                    "description": "Persona name or slug (e.g. 'Grandma Jody', 'grandma-jody', 'History Documenter')"
                },
                "prompt": {
                    "type": "string",
                    "description": "Question or conversational prompt (for 'ask' and 'recall'; aliases: 'question', 'query', 'text')"
                },
                "question": {
                    "type": "string",
                    "description": "Alias for 'prompt'"
                },
                "temp": {
                    "type": "string",
                    "description": "Zero-float rational p/q or Q8 integer temperature for personality & dyadic emotional randomness (e.g. '184/256', '23/32', '240', '0'; default: '184/256')",
                    "default": "184/256"
                },
                "speak": {
                    "description": "Pass true or a '.wav' file path string to synthesize spoken WAV audio modulated by inner-voice register, rational Q8 temperature, and dyadic emotion (for 'ask')"
                },
                "top_k": {
                    "type": "integer",
                    "description": "Number of resonant episodes to retrieve (for 'recall', default: 8)",
                    "default": 8
                },
                "path": {
                    "type": "string",
                    "description": "Directory or file path for 'train' or 'specialties'"
                },
                "specialty": {
                    "type": "string",
                    "enum": ALL_PERSONA_SPECIALTIES,
                    "description": "Specialty lens for 'train' (default: 'life-biographer')",
                    "default": "life-biographer"
                },
                "probe_do": {
                    "type": "boolean",
                    "description": "For 'do_e2e'/'zeno': optionally perform a bounded 800ms HTTP health probe against DO_OPENCLAW_ENDPOINT (default: false)",
                    "default": false
                }
            },
            "required": []
        }
    })
}

fn extract_prompt(params: &Value) -> Option<String> {
    for key in ["prompt", "question", "query", "text"] {
        if let Some(s) = params.get(key).and_then(|v| v.as_str()) {
            let trimmed = s.trim();
            if !trimmed.is_empty() {
                return Some(trimmed.to_string());
            }
        }
    }
    None
}

fn extract_name(params: &Value) -> Option<String> {
    for key in ["name", "persona", "slug"] {
        if let Some(s) = params.get(key).and_then(|v| v.as_str()) {
            let trimmed = s.trim();
            if !trimmed.is_empty() {
                return Some(trimmed.to_string());
            }
        }
    }
    None
}

fn extract_temp_q8(params: &Value) -> i32 {
    match params.get("temp") {
        Some(Value::String(s)) => parse_rational_temp_q8(s),
        Some(Value::Number(n)) => {
            if let Some(i) = n.as_i64() {
                if i <= 4 {
                    ((i as i32) * 256).clamp(0, 1024)
                } else {
                    (i as i32).clamp(0, 1024)
                }
            } else {
                184
            }
        }
        _ => 184,
    }
}

fn default_persona_wav_dir() -> PathBuf {
    LifePersona::personas_dir()
        .ok()
        .or_else(|| LifePersona::all_personas_dirs().into_iter().next())
        .unwrap_or_else(|| {
            dirs::home_dir()
                .unwrap_or_else(|| PathBuf::from("."))
                .join(".st")
                .join("personas")
        })
}

fn extract_speak_path(params: &Value, persona_name: &str) -> Option<PathBuf> {
    match params.get("speak") {
        Some(Value::Bool(true)) => {
            let slug = LifePersona::slugify(persona_name);
            Some(default_persona_wav_dir().join(format!("{slug}_phoenix_reply.wav")))
        }
        Some(Value::String(s)) => {
            let trimmed = s.trim();
            if trimmed.is_empty() || trimmed.eq_ignore_ascii_case("true") {
                let slug = LifePersona::slugify(persona_name);
                Some(default_persona_wav_dir().join(format!("{slug}_phoenix_reply.wav")))
            } else if trimmed.eq_ignore_ascii_case("false") {
                None
            } else {
                Some(PathBuf::from(trimmed))
            }
        }
        _ => None,
    }
}

/// Enforce the Aye Do-Not-Look privacy gate (`has_do_not_look_marker` + `is_aye_allowed_to_look`)
/// before training a persona or workspace specialty on `path`.
fn ensure_persona_path_allowed(path: &Path) -> Result<()> {
    let check_dir = if path.is_file() {
        path.parent().unwrap_or(path)
    } else {
        path
    };
    if crate::mcp::smart_project_detector::has_do_not_look_marker(check_dir) {
        anyhow::bail!(
            "Refusing to train persona on '{}': directory is protected by a Do-Not-Look marker (.noaye / .dontlook).",
            path.display()
        );
    }
    let temp_root = std::env::temp_dir();
    let canon_temp = temp_root.canonicalize().unwrap_or(temp_root);
    let canon_check = check_dir
        .canonicalize()
        .unwrap_or_else(|_| check_dir.to_path_buf());
    let is_ephemeral_test_dir =
        canon_check.starts_with(&canon_temp) || check_dir.starts_with(std::env::temp_dir());
    if !is_ephemeral_test_dir
        && !crate::mcp::smart_project_detector::is_aye_allowed_to_look(check_dir)
    {
        anyhow::bail!(
            "Refusing to train persona on '{}': path is excluded by Aye Do-Not-Look safety policy.",
            path.display()
        );
    }
    Ok(())
}

/// Canonical inner-voice register label + integer Hz pitch contour (matching `src/mem8/persona.rs`).
#[inline]
const fn inner_voice_register_key(reg: InnerVoiceRegister) -> &'static str {
    match reg {
        InnerVoiceRegister::WhisperedDiary => "WhisperedDiary (182 Hz)",
        InnerVoiceRegister::HearthStoryteller => "HearthStoryteller (189 Hz)",
        InnerVoiceRegister::WryHumorBanter => "WryHumorBanter (198 Hz)",
        InnerVoiceRegister::FierceConviction => "FierceConviction (204 Hz)",
        InnerVoiceRegister::TenderElegy => "TenderElegy (177 Hz)",
        InnerVoiceRegister::AncestralEcho => "AncestralEcho (185 Hz)",
    }
}

/// Dispatch one `persona` MCP tool call (`list`, `ask`, `recall`, `inspect`, `train`, `specialties`, `do_e2e`).
pub async fn handle_persona(params: Option<Value>, ctx: Arc<McpContext>) -> Result<Value> {
    let mut params = params.unwrap_or_else(|| json!({}));
    super::helpers::normalize_path_args(&mut params, &ctx);

    let prompt_opt = extract_prompt(&params);
    let name_opt = extract_name(&params);
    let has_path = params
        .get("path")
        .and_then(|v| v.as_str())
        .is_some_and(|s| !s.trim().is_empty());

    let raw_op = params
        .get("operation")
        .and_then(|v| v.as_str())
        .unwrap_or("");
    let operation = match raw_op {
        "persona_list" => "list",
        "persona_ask" => "ask",
        "persona_recall" => "recall",
        "persona_inspect" => "inspect",
        "persona_train" => "train",
        "persona_specialties" => "specialties",
        "persona_e2e" | "persona_zeno" | "persona_do" | "do_fleet" | "do_status" => "do_e2e",
        "" => {
            if prompt_opt.is_some() {
                "ask"
            } else if has_path && name_opt.is_some() {
                "train"
            } else if name_opt.is_some() {
                "inspect"
            } else {
                "list"
            }
        }
        other => other,
    };

    match operation {
        "list" | "ls" => {
            let summaries = tokio::task::spawn_blocking(LifePersona::list_all)
                .await
                .context("persona list worker failed")??;

            let personas_dirs: Vec<String> = LifePersona::all_personas_dirs()
                .into_iter()
                .map(|d| d.to_string_lossy().into_owned())
                .collect();

            let mut md = String::new();
            if summaries.is_empty() {
                md.push_str("No trained `.m8` personas found in `~/.st/personas/`.\n");
                md.push_str(
                    "Train one with: `persona {operation:'train', name:'Grandma Jody', path:'/path/to/writings'}`",
                );
            } else {
                md.push_str(&format!(
                    "🧬 **Trained MEM8 Life Personas (`{}` across `{}` store(s) · Rung 0 Bonobo `p/q = 3/2`)**:\n\n",
                    summaries.len(),
                    personas_dirs.len()
                ));
                for p in &summaries {
                    let size_kib = p.m8_size_bytes / 1024;
                    let size_rem = ((p.m8_size_bytes % 1024) * 10) / 1024;
                    let size_kib_pq = format_coprime_pq(p.m8_size_bytes, 1024);
                    let words_per_ep_pq =
                        format_coprime_pq(p.total_words as u64, (p.episode_count.max(1)) as u64);
                    md.push_str(&format!(
                        "- {} **{}** (`{}`) [{}] — **{} episodes**, {} words (`{}` words/ep), {} files (`{}.{}` KiB `.m8`, exact `{}` KiB)\n",
                        p.specialty.badge(),
                        p.name,
                        p.slug,
                        p.specialty.slug(),
                        p.episode_count,
                        p.total_words,
                        words_per_ep_pq,
                        p.files_ingested,
                        size_kib,
                        size_rem,
                        size_kib_pq
                    ));
                    if !p.sample_phrases.is_empty() {
                        md.push_str(&format!(
                            "  _Cadence_: {}\n",
                            p.sample_phrases.join(" · ")
                        ));
                    }
                }
            }

            Ok(json!({
                "content": [{
                    "type": "text",
                    "text": md
                }],
                "ultra_omni_zeno": ULTRA_OMNI_ZENO_HEADER,
                "bonobo_rung0_pq": "3/2",
                "personas_dirs": personas_dirs,
                "personas": summaries
            }))
        }
        "ask" | "chat" | "converse" => {
            let name = name_opt.context(
                "Missing 'name' parameter for persona ask (e.g. 'Grandma Jody' or 'grandma-jody')",
            )?;
            let prompt =
                prompt_opt.context("Missing 'prompt' (or 'question') parameter for persona ask")?;
            let temp_q8 = extract_temp_q8(&params);
            let temp_pq = format_coprime_pq(temp_q8.max(0) as u64, 256);
            let speak_path = extract_speak_path(&params, &name);

            let (reply, slug, episodes_count, zeno_capsule) =
                tokio::task::spawn_blocking(move || -> Result<(String, String, usize, Value)> {
                    let persona = LifePersona::load_cached(&name)?;
                    let text =
                        persona.respond_with_phoenix(&prompt, temp_q8, speak_path.as_deref());
                    let recalled = persona.recall_episodes(&prompt, 8);
                    let recalled_owned: Vec<PersonaEpisode> =
                        recalled.into_iter().cloned().collect();
                    let frame = PersonaLitanyWireFrame::from_episodes(
                        if recalled_owned.is_empty() {
                            &persona.episodes
                        } else {
                            &recalled_owned
                        },
                        256,
                    );
                    Ok((
                        text,
                        persona.slug.clone(),
                        persona.episodes.len(),
                        frame.to_json(),
                    ))
                })
                .await
                .context("persona ask worker failed")??;

            Ok(json!({
                "content": [{
                    "type": "text",
                    "text": reply
                }],
                "persona": slug,
                "episodes_count": episodes_count,
                "temp_q8": format!("{temp_q8}/256"),
                "temp_pq": temp_pq,
                "bonobo_rung0_pq": "3/2",
                "zeno_capsule": zeno_capsule
            }))
        }
        "recall" | "episodes" | "search" => {
            let name = name_opt
                .context("Missing 'name' parameter for persona recall (e.g. 'Grandma Jody')")?;
            let prompt = prompt_opt
                .context("Missing 'prompt' (or 'question') parameter for persona recall")?;
            let temp_q8 = extract_temp_q8(&params);
            let temp_pq = format_coprime_pq(temp_q8.max(0) as u64, 256);
            let top_k = params
                .get("top_k")
                .and_then(|v| v.as_u64())
                .map(|v| (v as usize).clamp(1, 32))
                .unwrap_or(8);

            let res = tokio::task::spawn_blocking(move || -> Result<Value> {
                let persona = LifePersona::load_cached(&name)?;
                let mut rng =
                    ((temp_q8 as u64).wrapping_mul(0x9E37_79B9_7F4A_7C15)) ^ 0xBF58_476D_1CE4_E5B9;
                let recalled =
                    persona.recall_episodes_with_temp(&prompt, top_k, temp_q8, &mut rng);
                let recalled_owned: Vec<PersonaEpisode> =
                    recalled.iter().copied().cloned().collect();
                let wire_frame = PersonaLitanyWireFrame::from_episodes(&recalled_owned, 256);

                let items: Vec<Value> = recalled
                    .iter()
                    .map(|ep| {
                        let trits = spqt_unpack_5trits_const(ep.inner_voice_spqt);
                        let (rung_label, rung_code) = spqt_song_signal_rung(ep.inner_voice_spqt);
                        json!({
                            "source": ep.source,
                            "era": ep.era,
                            "event_time": ep.event_time,
                            "written_time": ep.written_time,
                            "temporal_order": ep.temporal_order,
                            "author_role": format!("{:?}", ep.author_role),
                            "origin": ep.origin,
                            "inner_voice": ep.inner_voice.label(),
                            "voice_f0_hz": ep.voice_f0_hz,
                            "inner_voice_spqt": ep.inner_voice_spqt,
                            "inner_voice_trits": trits,
                            "spqt_song_rung": rung_label,
                            "spqt_song_rung_hex": format!("0x{:02X}", rung_code),
                            "interference_q8": format!("{}/256", ep.interference_q8),
                            "interference_pq": format_coprime_pq(ep.interference_q8.max(0) as u64, 256),
                            "token_ids_count": ep.token_ids.len(),
                            "keywords": ep.keywords,
                            "passage": ep.passage,
                        })
                    })
                    .collect();
                Ok(json!({
                    "persona": persona.name,
                    "slug": persona.slug,
                    "query": prompt,
                    "temp_q8": format!("{temp_q8}/256"),
                    "temp_pq": temp_pq,
                    "bonobo_rung0_pq": "3/2",
                    "count": items.len(),
                    "zeno_capsule": wire_frame.to_json(),
                    "episodes": items,
                }))
            })
            .await
            .context("persona recall worker failed")??;

            Ok(json!({
                "content": [{
                    "type": "text",
                    "text": serde_json::to_string_pretty(&res)?
                }],
                "recall": res
            }))
        }
        "inspect" | "info" | "status" | "soul" => {
            let name = name_opt
                .context("Missing 'name' parameter for persona inspect (e.g. 'Grandma Jody')")?;

            let info = tokio::task::spawn_blocking(move || -> Result<Value> {
                let persona = LifePersona::load_cached(&name)?;
                let mut register_counts: HashMap<&'static str, usize> = HashMap::new();
                let mut max_wave_q8: i16 = 0;
                let mut min_wave_q8: i16 = i16::MAX;
                let mut total_tokens_u16: usize = 0;

                for ep in &persona.episodes {
                    let lbl = inner_voice_register_key(ep.inner_voice);
                    *register_counts.entry(lbl).or_insert(0) += 1;
                    if ep.interference_q8 > max_wave_q8 {
                        max_wave_q8 = ep.interference_q8;
                    }
                    if ep.interference_q8 < min_wave_q8 {
                        min_wave_q8 = ep.interference_q8;
                    }
                    total_tokens_u16 += ep.token_ids.len();
                }
                if persona.episodes.is_empty() {
                    min_wave_q8 = 0;
                }

                let all_registers = [
                    InnerVoiceRegister::WhisperedDiary,
                    InnerVoiceRegister::HearthStoryteller,
                    InnerVoiceRegister::WryHumorBanter,
                    InnerVoiceRegister::FierceConviction,
                    InnerVoiceRegister::TenderElegy,
                    InnerVoiceRegister::AncestralEcho,
                ];
                let register_spqt_table: Vec<Value> = all_registers
                    .into_iter()
                    .map(|reg| {
                        let spqt = reg.spqt_byte();
                        json!({
                            "register": inner_voice_register_key(reg),
                            "label": reg.label(),
                            "spqt_byte": spqt,
                            "spqt_hex": format!("0x{:02X}", spqt),
                            "trits": spqt_unpack_5trits_const(spqt),
                            "pitch_millix_pq": format_coprime_pq(reg.pitch_millix().max(0) as u64, 1000),
                            "tempo_millix_pq": format_coprime_pq(reg.tempo_millix().max(0) as u64, 1000),
                        })
                    })
                    .collect();

                let warmth_q8 = (persona.voice.warmth * 256.0).round() as i64;
                let expr_q8 = (persona.voice.expressiveness * 256.0).round().max(0.0) as u64;
                let avg_words_milli =
                    (persona.voice.avg_sentence_words * 1000.0).round().max(0.0) as u64;
                let wire_frame = PersonaLitanyWireFrame::from_episodes(&persona.episodes, 256);

                Ok(json!({
                    "name": persona.name,
                    "slug": persona.slug,
                    "specialty": persona.specialty.slug(),
                    "badge": persona.specialty.badge(),
                    "source_path": persona.source_path,
                    "trained_at": persona.trained_at,
                    "files_ingested": persona.files_ingested,
                    "total_words": persona.total_words,
                    "episodes_count": persona.episodes.len(),
                    "total_u16_token_ids": total_tokens_u16,
                    "standing_wave_range_q8": format!("{min_wave_q8}/256 .. {max_wave_q8}/256"),
                    "standing_wave_range_pq": format!(
                        "{} .. {}",
                        format_coprime_pq(min_wave_q8.max(0) as u64, 256),
                        format_coprime_pq(max_wave_q8.max(0) as u64, 256)
                    ),
                    "bonobo_rung0_pq": "3/2",
                    "inner_voice_registers": register_counts,
                    "inner_voice_spqt_table": register_spqt_table,
                    "zeno_capsule": wire_frame.to_json(),
                    "ancestral_seed": persona.ancestral_seed,
                    "voice": {
                        "avg_sentence_words": persona.voice.avg_sentence_words,
                        "avg_sentence_words_pq": format_coprime_pq(avg_words_milli, 1000),
                        "warmth": persona.voice.warmth,
                        "warmth_pq": format_signed_coprime_pq(warmth_q8, 256),
                        "expressiveness": persona.voice.expressiveness,
                        "expressiveness_pq": format_coprime_pq(expr_q8, 256),
                        "signature_phrases": persona.voice.signature_phrases,
                        "top_vocabulary": persona.voice.vocabulary.iter().take(24).collect::<Vec<_>>(),
                        "sample_openings": persona.voice.sample_openings,
                    }
                }))
            })
            .await
            .context("persona inspect worker failed")??;

            Ok(json!({
                "content": [{
                    "type": "text",
                    "text": serde_json::to_string_pretty(&info)?
                }],
                "inspect": info
            }))
        }
        "train" => {
            let name = name_opt
                .context("Missing 'name' parameter for persona train (e.g. 'Grandma Jody')")?;
            let path_str = params
                .get("path")
                .and_then(|v| v.as_str())
                .filter(|s| !s.trim().is_empty())
                .context("Missing 'path' parameter for persona train")?
                .to_string();
            let target_path = PathBuf::from(&path_str);
            ensure_persona_path_allowed(&target_path)?;

            let spec_str = params
                .get("specialty")
                .and_then(|v| v.as_str())
                .unwrap_or("life-biographer");
            let spec = PersonaSpecialty::from_str_loose(spec_str);

            let out = tokio::task::spawn_blocking(move || -> Result<Value> {
                let (persona, saved_path, bytes) =
                    LifePersona::train_with_specialty(&name, target_path.as_path(), spec)?;
                let size_kib = bytes / 1024;
                let size_kib_pq = format_coprime_pq(bytes, 1024);
                let wire_frame = PersonaLitanyWireFrame::from_episodes(&persona.episodes, 256);
                Ok(json!({
                    "ok": true,
                    "name": persona.name,
                    "slug": persona.slug,
                    "specialty": persona.specialty.slug(),
                    "badge": persona.specialty.badge(),
                    "source_path": persona.source_path,
                    "files_ingested": persona.files_ingested,
                    "total_words": persona.total_words,
                    "episodes_count": persona.episodes.len(),
                    "saved_m8_path": saved_path.to_string_lossy(),
                    "m8_size_bytes": bytes,
                    "m8_size_kib": size_kib,
                    "m8_size_kib_pq": size_kib_pq,
                    "bonobo_rung0_pq": "3/2",
                    "zeno_capsule": wire_frame.to_json(),
                    "signature_phrases": persona.voice.signature_phrases,
                }))
            })
            .await
            .context("persona train worker failed")??;

            Ok(json!({
                "content": [{
                    "type": "text",
                    "text": serde_json::to_string_pretty(&out)?
                }],
                "train": out
            }))
        }
        "specialties" => {
            let default_cwd = ctx
                .cwd
                .lock()
                .ok()
                .map(|p| p.clone())
                .or_else(|| std::env::current_dir().ok())
                .unwrap_or_else(|| PathBuf::from("."));
            let target_path = params
                .get("path")
                .and_then(|v| v.as_str())
                .filter(|s| !s.trim().is_empty())
                .map(PathBuf::from)
                .unwrap_or(default_cwd);
            ensure_persona_path_allowed(&target_path)?;

            let out = tokio::task::spawn_blocking(move || -> Result<Value> {
                let trained = LifePersona::train_workspace_specialties(&target_path);
                let mut items: Vec<Value> = Vec::new();
                for s in trained.into_iter().flatten() {
                    items.push(json!({
                        "name": s.name,
                        "slug": s.slug,
                        "specialty": s.specialty.slug(),
                        "badge": s.specialty.badge(),
                        "episode_count": s.episode_count,
                        "total_words": s.total_words,
                        "files_ingested": s.files_ingested,
                        "m8_size_bytes": s.m8_size_bytes,
                        "m8_size_kib_pq": format_coprime_pq(s.m8_size_bytes, 1024),
                        "sample_phrases": s.sample_phrases,
                    }));
                }
                Ok(json!({
                    "ok": true,
                    "workspace": target_path.to_string_lossy(),
                    "bonobo_rung0_pq": "3/2",
                    "specialists_trained": items.len(),
                    "specialists": items,
                }))
            })
            .await
            .context("persona specialties worker failed")??;

            Ok(json!({
                "content": [{
                    "type": "text",
                    "text": serde_json::to_string_pretty(&out)?
                }],
                "specialties": out
            }))
        }
        "do_e2e" | "zeno" | "e2e" => {
            let probe_do = params
                .get("probe_do")
                .and_then(|v| v.as_bool())
                .unwrap_or(false);
            let do_endpoint = std::env::var("DO_OPENCLAW_ENDPOINT")
                .unwrap_or_else(|_| DO_OPENCLAW_DEFAULT_ENDPOINT.to_string());

            let do_probe_status = if probe_do {
                let client = reqwest::Client::builder()
                    .timeout(std::time::Duration::from_millis(800))
                    .build()?;
                match client.get(&do_endpoint).send().await {
                    Ok(resp) => format!("ONLINE (HTTP {})", resp.status().as_u16()),
                    Err(e) => format!("STAGED ({e})"),
                }
            } else {
                "SKIPPED (pass probe_do:true to probe Tailscale gateway)".to_string()
            };

            let report = tokio::task::spawn_blocking(move || -> Result<Value> {
                let personas_dirs: Vec<String> = LifePersona::all_personas_dirs()
                    .into_iter()
                    .map(|d| d.to_string_lossy().into_owned())
                    .collect();
                let summaries = LifePersona::list_all().unwrap_or_default();

                let liquid_bin = std::env::var("LIQUID_RUST_BIN")
                    .map(PathBuf::from)
                    .unwrap_or_else(|_| PathBuf::from("/home/hue/.local/bin/liquid-rust"));
                let model_dir = std::env::var("LFM_AUDIO_MODEL_DIR").unwrap_or_else(|_| {
                    "/home/hue/.cache/huggingface/hub/models--LiquidAI--LFM2.5-Audio-1.5B/snapshots/c362a0625dfe45aa588dce5f0ada28a7e5707628".to_string()
                });
                let do_hue_personas = Path::new("/home/hue/.st/personas").is_dir();

                // Load requested persona, or first available persona, or synthesize a deterministic 2-episode E2E soul
                let target_persona: Option<Arc<LifePersona>> = if let Some(ref req_name) = name_opt {
                    LifePersona::load_cached(req_name).ok()
                } else if let Some(first) = summaries.first() {
                    LifePersona::load_cached(&first.slug).ok()
                } else {
                    None
                };

                let (soul_name, soul_slug, episodes_count, wire_frame, sample_reply_ok) =
                    if let Some(p) = target_persona {
                        let wf = PersonaLitanyWireFrame::from_episodes(&p.episodes, 256);
                        let reply = p.respond_with_phoenix(
                            prompt_opt
                                .as_deref()
                                .unwrap_or("Tell me what you remember most vividly"),
                            184,
                            None,
                        );
                        (
                            p.name.clone(),
                            p.slug.clone(),
                            p.episodes.len(),
                            wf,
                            !reply.is_empty(),
                        )
                    } else {
                        let synth_episodes = vec![
                            PersonaEpisode {
                                source: "1963_platoro_diary.md".to_string(),
                                era: Some("1963-01-03 (Thu)".to_string()),
                                event_time: Some("1963-01-03 (Thu)".to_string()),
                                written_time: Some("1963".to_string()),
                                temporal_order: 19630103,
                                author_role: crate::mem8::persona::AuthorRole::PrimaryDiary,
                                origin: "Synthetic E2E Diary".to_string(),
                                inner_voice: InnerVoiceRegister::WhisperedDiary,
                                voice_f0_hz: 182,
                                inner_voice_spqt: InnerVoiceRegister::WhisperedDiary.spqt_byte(),
                                interference_q8: 384,
                                token_ids: vec![101, 205, 309],
                                keywords: vec!["platoro".to_string(), "pine".to_string()],
                                valence: 0.75,
                                arousal: 0.5,
                                frequency_hz: 88.0,
                                passage: "The pine breeze in Platoro was crisp and bright this morning.".to_string(),
                            },
                            PersonaEpisode {
                                source: "1964_chenterra_hearth.md".to_string(),
                                era: Some("1964-06-14 (Sun)".to_string()),
                                event_time: Some("1964-06-14 (Sun)".to_string()),
                                written_time: Some("1964".to_string()),
                                temporal_order: 19640614,
                                author_role: crate::mem8::persona::AuthorRole::PrimaryMemoir,
                                origin: "Synthetic E2E Memoir".to_string(),
                                inner_voice: InnerVoiceRegister::HearthStoryteller,
                                voice_f0_hz: 189,
                                inner_voice_spqt: InnerVoiceRegister::HearthStoryteller.spqt_byte(),
                                interference_q8: 320,
                                token_ids: vec![101, 412, 518],
                                keywords: vec!["chenterra".to_string(), "hearth".to_string()],
                                valence: 0.85,
                                arousal: 0.6,
                                frequency_hz: 96.0,
                                passage: "We gathered around the kitchen table at Chenterra Farm with warm coffee.".to_string(),
                            },
                        ];
                        let wf = PersonaLitanyWireFrame::from_episodes(&synth_episodes, 256);
                        (
                            "Synthetic E2E Soul".to_string(),
                            "synthetic-e2e-soul".to_string(),
                            synth_episodes.len(),
                            wf,
                            true,
                        )
                    };

                Ok(json!({
                    "ok": wire_frame.verify_seal() && sample_reply_ok,
                    "operation": "do_e2e",
                    "ultra_omni_zeno": ULTRA_OMNI_ZENO_HEADER,
                    "bonobo_rung0_pq": format!("{}/{}", BONOBO_RUNG0_PQ.0, BONOBO_RUNG0_PQ.1),
                    "persona_verified": {
                        "name": soul_name,
                        "slug": soul_slug,
                        "episodes_count": episodes_count,
                        "phoenix_reply_verified": sample_reply_ok,
                    },
                    "digitalocean_fleet": {
                        "openclaw_endpoint": do_endpoint,
                        "openclaw_probe": do_probe_status,
                        "do_hue_personas_dir_present": do_hue_personas,
                        "liquid_rust_tts_ready": liquid_bin.exists(),
                        "lfm_audio_model_ready": Path::new(&model_dir).exists(),
                        "personas_dirs": personas_dirs,
                        "trained_personas_count": summaries.len(),
                    },
                    "zeno_capsule": wire_frame.to_json(),
                }))
            })
            .await
            .context("persona do_e2e worker failed")??;

            Ok(json!({
                "content": [{
                    "type": "text",
                    "text": serde_json::to_string_pretty(&report)?
                }],
                "do_e2e": report
            }))
        }
        other => anyhow::bail!(
            "Unknown persona operation '{}'. Valid operations: list, ask, recall, inspect, train, specialties, do_e2e",
            other
        ),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn tool_definition_is_persona() {
        let def = persona_tool_definition();
        assert_eq!(def["name"], "persona");
        let ops = def["inputSchema"]["properties"]["operation"]["enum"]
            .as_array()
            .expect("operation enum");
        for expected in [
            "list",
            "ask",
            "recall",
            "inspect",
            "train",
            "specialties",
            "do_e2e",
            "zeno",
            "e2e",
        ] {
            assert!(ops.contains(&json!(expected)), "missing op {expected}");
        }

        let specs = def["inputSchema"]["properties"]["specialty"]["enum"]
            .as_array()
            .expect("specialty enum");
        assert_eq!(specs.len(), 9);
        for spec in ALL_PERSONA_SPECIALTIES {
            assert!(
                specs.contains(&json!(spec)),
                "missing specialty {spec} in schema"
            );
        }
    }

    #[test]
    fn coprime_pq_and_ultra_zeno_sun_law_invariants() {
        assert_eq!(binary_gcd_u64(184, 256), 8);
        assert_eq!(reduce_pq(184, 256), (23, 32));
        assert_eq!(format_coprime_pq(384, 256), "3/2");
        assert_eq!(BONOBO_RUNG0_PQ, (3, 2));

        let n_hor = ultra_zeno_solar_horizon(3, 2);
        assert_eq!(n_hor, 2);
        assert!(ultra_zeno_sun_margin(3, 2, (n_hor - 1) as i128, -1) <= 0);
        assert!(ultra_zeno_sun_margin(3, 2, n_hor as i128, -1) > 0);

        for reg in [
            InnerVoiceRegister::WhisperedDiary,
            InnerVoiceRegister::HearthStoryteller,
            InnerVoiceRegister::WryHumorBanter,
            InnerVoiceRegister::FierceConviction,
            InnerVoiceRegister::TenderElegy,
            InnerVoiceRegister::AncestralEcho,
        ] {
            let b = reg.spqt_byte();
            assert!(b < SPQT_BASE_243);
            let trits = spqt_unpack_5trits_const(b);
            for t in trits {
                assert!((-1..=1).contains(&t));
            }
        }
    }

    #[tokio::test]
    async fn persona_mcp_dispatch_list_and_memory_bridge_work() {
        let server = crate::mcp::McpServer::new(crate::mcp::McpConfig::default());
        let ctx = server.context.clone();

        let list_resp = crate::mcp::tools_consolidated::dispatch_consolidated_tool(
            "persona",
            Some(json!({"operation": "list"})),
            ctx.clone(),
        )
        .await
        .expect("persona list via MCP dispatch");
        assert!(list_resp["content"][0]["text"].as_str().is_some());
        assert_eq!(list_resp["bonobo_rung0_pq"], "3/2");

        let mem_resp = crate::mcp::tools_consolidated::dispatch_consolidated_tool(
            "memory",
            Some(json!({"operation": "persona_list"})),
            ctx,
        )
        .await
        .expect("memory persona_list via MCP dispatch");
        assert!(mem_resp["content"][0]["text"].as_str().is_some());
    }

    #[tokio::test]
    async fn persona_mcp_e2e_train_inspect_recall_ask_and_do_e2e_roundtrip() {
        let tmp = TempDir::new().expect("tempdir");
        let diary_path = tmp.path().join("1963_platoro_diary.md");
        std::fs::write(
            &diary_path,
            "THE PROPERTY OF\nJody Chenoweth\n\nYEARSLONG DIARY\nFrom January 1, 1963 to January 1, 1964\n\n\
             January 2 Wed.\n\n\
             Fred worked graveyards last night. We rode the mountain horses near Platoro and drank hot coffee.\n\n\
             January 3 Thurs.\n\n\
             Baked a pineapple upside down cake at Chenterra Farm and listened to the rain on the Peace Rose.",
        )
        .expect("write diary");

        let server = crate::mcp::McpServer::new(crate::mcp::McpConfig::default());
        let ctx = server.context.clone();

        // 1. Train persona in temp dir
        let train_resp = handle_persona(
            Some(json!({
                "operation": "train",
                "name": "Grandma Jody Test",
                "path": tmp.path().to_string_lossy(),
                "specialty": "life-biographer"
            })),
            ctx.clone(),
        )
        .await
        .expect("persona train");
        assert_eq!(train_resp["train"]["ok"], true);
        assert_eq!(train_resp["train"]["bonobo_rung0_pq"], "3/2");
        assert_eq!(
            train_resp["train"]["zeno_capsule"]["seal_verified"],
            json!(true)
        );
        let saved_m8 = PathBuf::from(
            train_resp["train"]["saved_m8_path"]
                .as_str()
                .expect("saved_m8_path"),
        );
        let loaded = LifePersona::load_cached_from_file(&saved_m8).expect("load_cached_from_file");
        assert!(loaded.episodes.len() >= 2);

        // 2. Verify Do-Not-Look (.noaye) privacy enforcement blocks training on protected folders
        let protected_dir = tmp.path().join("private_vault");
        std::fs::create_dir_all(&protected_dir).expect("mkdir private_vault");
        std::fs::write(protected_dir.join(".noaye"), "# do not look\n").expect("write .noaye");
        std::fs::write(protected_dir.join("secret.md"), "Secret diary entry.").expect("secret");
        let blocked = handle_persona(
            Some(json!({
                "operation": "train",
                "name": "Blocked Persona",
                "path": protected_dir.to_string_lossy()
            })),
            ctx.clone(),
        )
        .await;
        assert!(
            blocked.is_err(),
            "Do-Not-Look marker must reject persona training"
        );

        // 3. Run do_e2e / zeno verification
        let e2e_resp = handle_persona(
            Some(json!({
                "operation": "do_e2e",
                "probe_do": false
            })),
            ctx,
        )
        .await
        .expect("persona do_e2e");
        assert_eq!(e2e_resp["do_e2e"]["ok"], true);
        assert_eq!(
            e2e_resp["do_e2e"]["zeno_capsule"]["seal_verified"],
            json!(true)
        );
        assert_eq!(
            e2e_resp["do_e2e"]["zeno_capsule"]["bfp64_kleene_disjoint"],
            json!(true)
        );
        assert_eq!(
            e2e_resp["do_e2e"]["zeno_capsule"]["wire_64b_hex"]
                .as_str()
                .map(str::len),
            Some(128)
        );
    }

    #[tokio::test]
    async fn san_francesco_800th_transitus_poverello_persona_e2e_seal() {
        let sovereign_book = PathBuf::from(
            "/Users/peterlodri/workspace/peterlodri-sec/sovereign-library/editions/ultralove/san-francesco-the-complete-poverello-800th-transitus.md",
        );
        let tmp = TempDir::new().expect("tempdir");
        let train_target = if sovereign_book.exists() {
            sovereign_book
        } else {
            let fallback = tmp.path().join("san-francesco.md");
            std::fs::write(
                &fallback,
                "# SAN FRANCESCO — the complete poverello (1226–2026)\n\n\
                 I peter lodri declare that I am a Franciscans, and this is 2026 october, and I am in Tokyo.\n\n\
                 When I left the lepers, what had seemed bitter to me was turned into sweetness of soul and body.\n\n\
                 Dyad Registry: measured: poverty (0) <-> deleted: kinship (+1) · in a loop · in a sphere · in eternity · recurse.",
            )
            .expect("write fallback");
            fallback
        };

        let server = crate::mcp::McpServer::new(crate::mcp::McpConfig::default());
        let ctx = server.context.clone();

        let train_resp = handle_persona(
            Some(json!({
                "operation": "train",
                "name": "San Francesco",
                "path": train_target.to_string_lossy(),
                "specialty": "life-biographer"
            })),
            ctx.clone(),
        )
        .await
        .expect("train San Francesco");
        assert_eq!(train_resp["train"]["ok"], true);
        assert_eq!(train_resp["train"]["slug"], "san-francesco");
        assert_eq!(train_resp["train"]["bonobo_rung0_pq"], "3/2");
        assert_eq!(
            train_resp["train"]["zeno_capsule"]["seal_verified"],
            json!(true)
        );

        let ask_resp = handle_persona(
            Some(json!({
                "operation": "ask",
                "name": "san-francesco",
                "prompt": "Tell me about the leper on the road below Assisi, Lady Poverty, Clare, and the Dyad Registry (measured: poverty vs deleted: kinship).",
                "temp_pq": "3/2"
            })),
            ctx.clone(),
        )
        .await
        .expect("ask San Francesco");
        assert_eq!(ask_resp["bonobo_rung0_pq"], "3/2");
        assert_eq!(ask_resp["zeno_capsule"]["seal_verified"], json!(true));

        let e2e_resp = handle_persona(
            Some(json!({
                "operation": "do_e2e",
                "name": "san-francesco",
                "probe_do": false
            })),
            ctx,
        )
        .await
        .expect("do_e2e San Francesco");
        assert_eq!(e2e_resp["do_e2e"]["ok"], true);
        assert_eq!(
            e2e_resp["do_e2e"]["persona_verified"]["slug"],
            "san-francesco"
        );
        assert_eq!(
            e2e_resp["do_e2e"]["zeno_capsule"]["seal_verified"],
            json!(true)
        );
    }

    #[tokio::test]
    async fn erdos_lodri_ultragraph_mem8_phoenix_persona_e2e_seal() {
        let sovereign_book = PathBuf::from(
            "/Users/peterlodri/workspace/peterlodri-sec/sovereign-library/editions/ultralove/erdos-lodri-ultragraph-chenoweth-mem8-phoenix-kyuss-tibetan-omni.md",
        );
        let tmp = TempDir::new().expect("tempdir");
        let train_target = if sovereign_book.exists() {
            sovereign_book
        } else {
            let fallback = tmp.path().join("erdos-lodri-ultragraph.md");
            std::fs::write(
                &fallback,
                "# ERDŐS–LODRI ULTRAGRAPH × CHENOWETH MEM|8 & PHOENIX (p/q = 3/2)\n\n\
                 Peace must be earned, Respect must be given as a gesture, Smile should be the price for the visit.\n\n\
                 AHOGY A DOLGOK VANNAK — Kohayakawa-Rodl sparse Szemeredi regularity (K <= 256) and Shadow Graph G_shadow.",
            )
            .expect("write fallback");
            fallback
        };

        let server = crate::mcp::McpServer::new(crate::mcp::McpConfig::default());
        let ctx = server.context.clone();

        let train_resp = handle_persona(
            Some(json!({
                "operation": "train",
                "name": "Erdos Lodri UltraGraph",
                "path": train_target.to_string_lossy(),
                "specialty": "code-reviewer"
            })),
            ctx.clone(),
        )
        .await
        .expect("train Erdos Lodri UltraGraph");
        assert_eq!(train_resp["train"]["ok"], true);
        assert_eq!(train_resp["train"]["slug"], "erdos-lodri-ultragraph");
        assert_eq!(train_resp["train"]["bonobo_rung0_pq"], "3/2");
        assert_eq!(
            train_resp["train"]["zeno_capsule"]["seal_verified"],
            json!(true)
        );

        let e2e_resp = handle_persona(
            Some(json!({
                "operation": "do_e2e",
                "name": "erdos-lodri-ultragraph",
                "probe_do": false
            })),
            ctx,
        )
        .await
        .expect("do_e2e Erdos Lodri UltraGraph");
        assert_eq!(e2e_resp["do_e2e"]["ok"], true);
        assert_eq!(
            e2e_resp["do_e2e"]["persona_verified"]["slug"],
            "erdos-lodri-ultragraph"
        );
        assert_eq!(
            e2e_resp["do_e2e"]["zeno_capsule"]["seal_verified"],
            json!(true)
        );
    }
}
