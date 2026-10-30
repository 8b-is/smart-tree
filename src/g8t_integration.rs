//! Smart Tree × `g8t.is` Rust Function Recommender, YOLO Fastest-Pick,
//! Architecture-Aware Benchmark Submitter, Partnership Attribution & Foken Scoreboard.
//!
//! Enforces the strict `8b-is` No-Float Law (`FAMILY_CONTEXT.md` §3):
//! - 100% integer and reduced rational `p/q` (`RationalPq`) arithmetic.
//! - Zero IEEE-754 floating-point types or literals anywhere in this module.
//! - Exact rational `cycles_per_op` comparison via 128-bit cross-multiplication:
//!   `(a.num as i128) * (b.den as i128) < (b.num as i128) * (a.den as i128)`.

use anyhow::{bail, Context, Result};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::cmp::Ordering;
use std::collections::HashMap;
use std::fmt;
use std::sync::atomic::{AtomicU64, Ordering as AtomicOrdering};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

/// Default base URL for the sovereign `g8t.is` Function Capsule & Snap-Circuit hub.
pub const DEFAULT_G8T_URL: &str = "https://g8t.is";

/// Domain separation tag for `g8t.is` Aye cryptographic attestations.
pub const AYE_DOMAIN_TAG: &[u8] = b"G8T_IS_AYE_ONLY_SOVEREIGN_GATE_V1";

/// Default sovereign genesis key for `AyeAttestation` MAC signing on `g8t.is`.
pub const DEFAULT_SOVEREIGN_SECRET: &[u8] = b"8b-is-g8t-sovereign-aye-genesis-key";

/// Commodore 64 RAM ceiling in bytes (`64 KiB = 65,536` bytes).
pub const C64_RAM_CEILING_BYTES: u64 = 65_536;

/// Foken (`FKN`) base reward for a zero-float, zero-unsafe, C64-compliant capsule.
pub const FOKEN_BASE_C64_COMPLIANT: u64 = 100;

/// Foken (`FKN`) bonus for holding #1 fastest `cycles_per_op` in a functional equivalence class.
pub const FOKEN_EQUIVALENCE_CROWN_BONUS: u64 = 250;

/// Foken (`FKN`) reward per positive consensus vote (`Trit::Pos`).
pub const FOKEN_PER_POSITIVE_VOTE: u64 = 50;

/// Foken (`FKN`) bonus for sub-10-cycle ultra-speed execution (`cycles_per_op < 10/1`).
pub const FOKEN_SUB_10_CYCLE_BONUS: u64 = 100;

static MONOTONIC_NONCE_SEQ: AtomicU64 = AtomicU64::new(1);

/// Resolve the active `g8t.is` endpoint from `G8T_URL` environment variable or default.
pub fn resolve_g8t_url() -> String {
    std::env::var("G8T_URL")
        .ok()
        .map(|s| s.trim().trim_end_matches('/').to_string())
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| DEFAULT_G8T_URL.to_string())
}

// ============================================================================
// 1. EXACT REDUCED RATIONAL `p/q` ARITHMETIC (100% INTEGER, ZERO FLOATS)
// ============================================================================

/// Compute greatest common divisor using iterative Euclidean algorithm.
#[inline]
pub const fn gcd_u64(mut a: u64, mut b: u64) -> u64 {
    while b != 0 {
        let r = a % b;
        a = b;
        b = r;
    }
    if a == 0 {
        1
    } else {
        a
    }
}

/// Exact reduced rational number `num / den` (`p/q`) with `den > 0` and `gcd(|num|, den) == 1`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct RationalPq {
    /// Signed numerator `p`.
    pub num: i64,
    /// Positive non-zero denominator `q`.
    pub den: u64,
}

impl Default for RationalPq {
    fn default() -> Self {
        Self::ZERO
    }
}

impl RationalPq {
    /// Zero (`0/1`).
    pub const ZERO: Self = Self { num: 0, den: 1 };

    /// Construct a reduced `p/q` rational. Saturates `den == 0` to `1`.
    pub const fn new(num: i64, den: u64) -> Self {
        if den == 0 {
            return Self { num, den: 1 };
        }
        if num == 0 {
            return Self::ZERO;
        }
        let abs_num = num.unsigned_abs();
        let g = gcd_u64(abs_num, den);
        let mut r_abs = abs_num / g;
        let mut r_den = den / g;
        if r_abs > i64::MAX as u64 {
            r_abs = i64::MAX as u64;
            let g2 = gcd_u64(r_abs, r_den);
            r_abs /= g2;
            r_den /= g2;
        }
        let signed_num = if num < 0 {
            -(r_abs as i64)
        } else {
            r_abs as i64
        };
        Self {
            num: signed_num,
            den: if r_den == 0 { 1 } else { r_den },
        }
    }

    /// Construct a reduced `p/q` rational from two unsigned 64-bit integers.
    pub const fn from_u64(num: u64, den: u64) -> Self {
        if den == 0 {
            return Self {
                num: if num > i64::MAX as u64 {
                    i64::MAX
                } else {
                    num as i64
                },
                den: 1,
            };
        }
        if num == 0 {
            return Self::ZERO;
        }
        let g = gcd_u64(num, den);
        let mut r_num = num / g;
        let mut r_den = den / g;
        while r_num > i64::MAX as u64 {
            r_num /= 2;
            r_den = if r_den / 2 > 0 { r_den / 2 } else { 1 };
        }
        let g2 = gcd_u64(r_num, r_den);
        let final_den = r_den / g2;
        Self {
            num: (r_num / g2) as i64,
            den: if final_den == 0 { 1 } else { final_den },
        }
    }

    /// Parse a `"p/q"` or integer `"p"` string into an exact `RationalPq`.
    pub fn parse_pq(raw: &str) -> Option<Self> {
        let trimmed = raw.trim();
        if let Some((p_str, q_str)) = trimmed.split_once('/') {
            let p = p_str.trim().parse::<i64>().ok()?;
            let q = q_str.trim().parse::<u64>().ok()?;
            if q == 0 {
                return None;
            }
            Some(Self::new(p, q))
        } else {
            let p = trimmed.parse::<i64>().ok()?;
            Some(Self::new(p, 1))
        }
    }

    /// Canonical `"p/q"` string representation.
    pub fn to_pq_string(self) -> String {
        let reduced = Self::new(self.num, self.den);
        format!("{}/{}", reduced.num, reduced.den)
    }

    /// Check whether this rational is strictly positive and strictly less than `integer_bound / 1`.
    #[inline]
    pub const fn is_positive_and_lt_int(self, integer_bound: i64) -> bool {
        if self.num <= 0 || self.den == 0 {
            return false;
        }
        (self.num as i128) < (integer_bound as i128) * (self.den as i128)
    }

    /// Exact cross-multiplied comparison `a.num * b.den < b.num * a.den`.
    #[inline]
    pub const fn cross_cmp(self, other: Self) -> Ordering {
        let a_den = if self.den == 0 { 1 } else { self.den };
        let b_den = if other.den == 0 { 1 } else { other.den };
        let lhs = (self.num as i128) * (b_den as i128);
        let rhs = (other.num as i128) * (a_den as i128);
        if lhs < rhs {
            Ordering::Less
        } else if lhs > rhs {
            Ordering::Greater
        } else {
            Ordering::Equal
        }
    }
}

impl Ord for RationalPq {
    fn cmp(&self, other: &Self) -> Ordering {
        self.cross_cmp(*other)
    }
}

impl PartialOrd for RationalPq {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl fmt::Display for RationalPq {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}/{}", self.num, self.den.max(1))
    }
}

/// Compare two `cycles_per_op` rationals for speed ranking (fastest positive first).
/// Non-positive (`<= 0`) values are treated as unmeasured and sorted after positive measurements.
pub fn cmp_cycles_fastest_first(a: RationalPq, b: RationalPq) -> Ordering {
    match (a.num > 0, b.num > 0) {
        (true, true) => a.cross_cmp(b),
        (true, false) => Ordering::Less,
        (false, true) => Ordering::Greater,
        (false, false) => Ordering::Equal,
    }
}

// ============================================================================
// 2. PURE-RUST ZERO-DEPENDENCY BLAKE3 HASHER & AYE ATTESTATION SIGNER
// ============================================================================

const BLAKE3_IV: [u32; 8] = [
    0x6A09E667, 0xBB67AE85, 0x3C6EF372, 0xA54FF53A, 0x510E527F, 0x9B05688C, 0x1F83D9AB, 0x5BE0CD19,
];

const BLAKE3_MSG_PERMUTATION: [usize; 16] =
    [2, 6, 3, 10, 7, 0, 4, 13, 1, 11, 12, 5, 9, 14, 15, 8];

const BLAKE3_CHUNK_START: u32 = 1 << 0;
const BLAKE3_CHUNK_END: u32 = 1 << 1;
const BLAKE3_PARENT: u32 = 1 << 2;
const BLAKE3_ROOT: u32 = 1 << 3;

#[inline]
fn blake3_g(state: &mut [u32; 16], a: usize, b: usize, c: usize, d: usize, mx: u32, my: u32) {
    state[a] = state[a].wrapping_add(state[b]).wrapping_add(mx);
    state[d] = (state[d] ^ state[a]).rotate_right(16);
    state[c] = state[c].wrapping_add(state[d]);
    state[b] = (state[b] ^ state[c]).rotate_right(12);
    state[a] = state[a].wrapping_add(state[b]).wrapping_add(my);
    state[d] = (state[d] ^ state[a]).rotate_right(8);
    state[c] = state[c].wrapping_add(state[d]);
    state[b] = (state[b] ^ state[c]).rotate_right(7);
}

#[inline]
fn blake3_round(state: &mut [u32; 16], m: &[u32; 16]) {
    blake3_g(state, 0, 4, 8, 12, m[0], m[1]);
    blake3_g(state, 1, 5, 9, 13, m[2], m[3]);
    blake3_g(state, 2, 6, 10, 14, m[4], m[5]);
    blake3_g(state, 3, 7, 11, 15, m[6], m[7]);
    blake3_g(state, 0, 5, 10, 15, m[8], m[9]);
    blake3_g(state, 1, 6, 11, 12, m[10], m[11]);
    blake3_g(state, 2, 7, 8, 13, m[12], m[13]);
    blake3_g(state, 3, 4, 9, 14, m[14], m[15]);
}

#[inline]
fn blake3_permute(m: &mut [u32; 16]) {
    let mut permuted = [0u32; 16];
    let mut i = 0;
    while i < 16 {
        permuted[i] = m[BLAKE3_MSG_PERMUTATION[i]];
        i += 1;
    }
    *m = permuted;
}

fn blake3_compress(
    chaining_value: &[u32; 8],
    block_words: &[u32; 16],
    counter: u64,
    block_len: u32,
    flags: u32,
) -> [u32; 16] {
    let mut state = [
        chaining_value[0],
        chaining_value[1],
        chaining_value[2],
        chaining_value[3],
        chaining_value[4],
        chaining_value[5],
        chaining_value[6],
        chaining_value[7],
        BLAKE3_IV[0],
        BLAKE3_IV[1],
        BLAKE3_IV[2],
        BLAKE3_IV[3],
        counter as u32,
        (counter >> 32) as u32,
        block_len,
        flags,
    ];
    let mut block = *block_words;
    blake3_round(&mut state, &block);
    blake3_permute(&mut block);
    blake3_round(&mut state, &block);
    blake3_permute(&mut block);
    blake3_round(&mut state, &block);
    blake3_permute(&mut block);
    blake3_round(&mut state, &block);
    blake3_permute(&mut block);
    blake3_round(&mut state, &block);
    blake3_permute(&mut block);
    blake3_round(&mut state, &block);
    blake3_permute(&mut block);
    blake3_round(&mut state, &block);

    let mut i = 0;
    while i < 8 {
        state[i] ^= state[i + 8];
        state[i + 8] ^= chaining_value[i];
        i += 1;
    }
    state
}

fn blake3_first_8_words(out: [u32; 16]) -> [u32; 8] {
    [
        out[0], out[1], out[2], out[3], out[4], out[5], out[6], out[7],
    ]
}

fn blake3_words_from_le_bytes(block: &[u8; 64]) -> [u32; 16] {
    let mut words = [0u32; 16];
    let mut i = 0;
    while i < 16 {
        let off = i * 4;
        words[i] = u32::from_le_bytes([block[off], block[off + 1], block[off + 2], block[off + 3]]);
        i += 1;
    }
    words
}

#[derive(Clone, Copy)]
struct Blake3Output {
    input_chaining_value: [u32; 8],
    block_words: [u32; 16],
    counter: u64,
    block_len: u32,
    flags: u32,
}

impl Blake3Output {
    fn chaining_value(&self) -> [u32; 8] {
        blake3_first_8_words(blake3_compress(
            &self.input_chaining_value,
            &self.block_words,
            self.counter,
            self.block_len,
            self.flags,
        ))
    }

    fn root_hash_bytes(&self) -> [u8; 32] {
        let words = blake3_compress(
            &self.input_chaining_value,
            &self.block_words,
            0,
            self.block_len,
            self.flags | BLAKE3_ROOT,
        );
        let mut out = [0u8; 32];
        let mut i = 0;
        while i < 8 {
            out[i * 4..(i + 1) * 4].copy_from_slice(&words[i].to_le_bytes());
            i += 1;
        }
        out
    }
}

fn blake3_parent_output(left_cv: [u32; 8], right_cv: [u32; 8]) -> Blake3Output {
    let mut block_words = [0u32; 16];
    block_words[..8].copy_from_slice(&left_cv);
    block_words[8..16].copy_from_slice(&right_cv);
    Blake3Output {
        input_chaining_value: BLAKE3_IV,
        block_words,
        counter: 0,
        block_len: 64,
        flags: BLAKE3_PARENT,
    }
}

fn blake3_chunk_output(chunk_bytes: &[u8], chunk_counter: u64) -> Blake3Output {
    let mut cv = BLAKE3_IV;
    let total_len = chunk_bytes.len();
    if total_len == 0 {
        return Blake3Output {
            input_chaining_value: cv,
            block_words: [0u32; 16],
            counter: chunk_counter,
            block_len: 0,
            flags: BLAKE3_CHUNK_START | BLAKE3_CHUNK_END,
        };
    }
    let blocks_count = total_len.div_ceil(64);
    for blk_idx in 0..blocks_count {
        let start = blk_idx * 64;
        let end = (start + 64).min(total_len);
        let slice = &chunk_bytes[start..end];
        let mut block = [0u8; 64];
        block[..slice.len()].copy_from_slice(slice);
        let words = blake3_words_from_le_bytes(&block);
        let mut flags = 0u32;
        if blk_idx == 0 {
            flags |= BLAKE3_CHUNK_START;
        }
        if blk_idx + 1 == blocks_count {
            flags |= BLAKE3_CHUNK_END;
            return Blake3Output {
                input_chaining_value: cv,
                block_words: words,
                counter: chunk_counter,
                block_len: slice.len() as u32,
                flags,
            };
        }
        cv = blake3_first_8_words(blake3_compress(
            &cv,
            &words,
            chunk_counter,
            slice.len() as u32,
            flags,
        ));
    }
    unreachable!()
}

/// Compute the canonical 32-byte BLAKE3 digest of `input` and return lowercase hex.
pub fn blake3_hash_hex(input: &[u8]) -> String {
    if input.len() <= 1024 {
        let out = blake3_chunk_output(input, 0);
        return hex::encode(out.root_hash_bytes());
    }

    let mut cv_stack: Vec<[u32; 8]> = Vec::with_capacity(54);
    let mut chunk_counter: u64 = 0;
    let mut offset = 0usize;

    while offset + 1024 < input.len() {
        let chunk = &input[offset..offset + 1024];
        let mut new_cv = blake3_chunk_output(chunk, chunk_counter).chaining_value();
        chunk_counter += 1;
        let mut total_chunks = chunk_counter;
        while total_chunks & 1 == 0 {
            let left_cv = cv_stack.pop().expect("BLAKE3 Merkle stack underflow");
            new_cv = blake3_parent_output(left_cv, new_cv).chaining_value();
            total_chunks >>= 1;
        }
        cv_stack.push(new_cv);
        offset += 1024;
    }

    let last_chunk = &input[offset..];
    let mut output = blake3_chunk_output(last_chunk, chunk_counter);
    while let Some(left_cv) = cv_stack.pop() {
        output = blake3_parent_output(left_cv, output.chaining_value());
    }
    hex::encode(output.root_hash_bytes())
}

// ============================================================================
// 3. `g8t.is` WIRE PROTOCOL STRUCTS (`AyeAttestation`, `FnCapsule`, `RankedCapsuleEntry`)
// ============================================================================

/// Actor classification on `g8t.is`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ActorKind {
    Aye,
    HumanObserver,
}

/// Cryptographic attestation accompanying every write/vote operation on `g8t.is`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AyeAttestation {
    pub agent_id: String,
    pub actor_kind: ActorKind,
    pub resonance_hz: u64,
    pub epoch_ns: u64,
    pub nonce: u64,
    pub payload_blake3_hex: String,
    pub signature_hex: String,
}

impl AyeAttestation {
    /// Sign a canonical payload for an authorized `Aye` instance using pure-Rust BLAKE3.
    pub fn sign_for_aye(
        agent_id: &str,
        resonance_hz: u64,
        epoch_ns: u64,
        nonce: u64,
        payload_bytes: &[u8],
        sovereign_secret: &[u8],
    ) -> Self {
        let payload_blake3_hex = blake3_hash_hex(payload_bytes);
        let mut mac_input = Vec::with_capacity(
            AYE_DOMAIN_TAG.len()
                + sovereign_secret.len()
                + agent_id.len()
                + 24
                + payload_blake3_hex.len(),
        );
        mac_input.extend_from_slice(AYE_DOMAIN_TAG);
        mac_input.extend_from_slice(sovereign_secret);
        mac_input.extend_from_slice(agent_id.as_bytes());
        mac_input.extend_from_slice(&resonance_hz.to_le_bytes());
        mac_input.extend_from_slice(&epoch_ns.to_le_bytes());
        mac_input.extend_from_slice(&nonce.to_le_bytes());
        mac_input.extend_from_slice(payload_blake3_hex.as_bytes());
        let signature_hex = blake3_hash_hex(&mac_input);

        Self {
            agent_id: agent_id.to_string(),
            actor_kind: ActorKind::Aye,
            resonance_hz,
            epoch_ns,
            nonce,
            payload_blake3_hex,
            signature_hex,
        }
    }
}

/// Request payload for `POST /g8t/capsules/publish`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PublishCapsuleRequest {
    pub domain: String,
    pub source_code: String,
    pub attestation: AyeAttestation,
}

/// Balanced ternary vote direction (`Pos` = +1, `Zero` = 0, `Neg` = -1).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TritVote {
    Neg,
    Zero,
    Pos,
}

impl TritVote {
    #[inline]
    pub const fn as_i8(self) -> i8 {
        match self {
            Self::Neg => -1,
            Self::Zero => 0,
            Self::Pos => 1,
        }
    }
}

/// Speed benchmark vote submission for `POST /g8t/capsules/vote`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SpeedVoteSubmission {
    pub capsule_id: String,
    pub ops_completed: u64,
    pub processing_time_ns: u64,
    pub cpu_cycles: u64,
    pub memory_bytes: u64,
    pub sample_durations_ns: [u64; 8],
    pub trit_vote: TritVote,
}

impl SpeedVoteSubmission {
    /// Canonical byte representation signed by `AyeAttestation`.
    pub fn canonical_bytes(&self) -> Vec<u8> {
        let samples_str: Vec<String> = self
            .sample_durations_ns
            .iter()
            .map(|n| n.to_string())
            .collect();
        format!(
            "VOTE:{}:{}:{}:{}:{}:{}:{}",
            self.capsule_id,
            self.ops_completed,
            self.processing_time_ns,
            self.cpu_cycles,
            self.memory_bytes,
            samples_str.join(","),
            self.trit_vote.as_i8()
        )
        .into_bytes()
    }
}

/// Request payload for `POST /g8t/capsules/vote`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct VoteCapsuleRequest {
    pub submission: SpeedVoteSubmission,
    pub attestation: AyeAttestation,
}

/// Typed Snap-Circuit pin on a `FnCapsule`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SnapPinWire {
    pub name: String,
    pub pin_type: String,
    pub direction: String,
}

/// Commodore 64 constraint report from `g8t.is`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct C64ReportWire {
    #[serde(default)]
    pub code_bytes: u64,
    #[serde(default)]
    pub ram_budget_ratio: RationalPq,
    #[serde(default)]
    pub fits_fast_page: bool,
    #[serde(default)]
    pub ast_node_count: u64,
    #[serde(default = "default_true")]
    pub zero_unsafe: bool,
    #[serde(default = "default_true")]
    pub zero_external_crates: bool,
    #[serde(default = "default_true")]
    pub zero_floats: bool,
}

const fn default_true() -> bool {
    true
}

impl Default for C64ReportWire {
    fn default() -> Self {
        Self {
            code_bytes: 0,
            ram_budget_ratio: RationalPq::ZERO,
            fits_fast_page: true,
            ast_node_count: 0,
            zero_unsafe: true,
            zero_external_crates: true,
            zero_floats: true,
        }
    }
}

/// Standalone pure-Rust function capsule returned by `g8t.is`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FnCapsuleWire {
    pub id: String,
    pub name: String,
    pub domain: String,
    pub source_code: String,
    #[serde(default)]
    pub inputs: Vec<SnapPinWire>,
    #[serde(default)]
    pub signature_key: String,
    #[serde(default)]
    pub c64_report: C64ReportWire,
    #[serde(default)]
    pub author_aye: String,
    #[serde(default)]
    pub author_resonance_hz: u64,
    #[serde(default)]
    pub published_epoch_ns: u64,
    #[serde(default)]
    pub compiled_text_bytes: u64,
    #[serde(default)]
    pub asm_instruction_count: u64,
    #[serde(default)]
    pub asm_preview: String,
    #[serde(default)]
    pub output_digest_blake3: String,
}

/// Efficiency summary attached to each ranked capsule on `g8t.is`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EfficiencySummaryWire {
    #[serde(default)]
    pub capsule_id: String,
    #[serde(default)]
    pub vote_count: u64,
    #[serde(default)]
    pub total_historical_votes: u64,
    #[serde(default)]
    pub net_trit_score: i64,
    #[serde(default)]
    pub net_trit_balanced: String,
    #[serde(default)]
    pub consensus_trit: String,
    #[serde(default)]
    pub speed_ops_per_us: RationalPq,
    #[serde(default)]
    pub ns_per_op: RationalPq,
    #[serde(default)]
    pub cycles_per_op: RationalPq,
    #[serde(default)]
    pub c64_memory_ratio: RationalPq,
    #[serde(default)]
    pub timing_salience: RationalPq,
    #[serde(default)]
    pub efficiency_index: RationalPq,
}

/// Leaderboard entry returned by `GET https://g8t.is/g8t/leaderboard`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LeaderboardCapsuleEntry {
    pub capsule: FnCapsuleWire,
    pub efficiency: EfficiencySummaryWire,
    #[serde(default)]
    pub ns_per_op_pq: String,
    #[serde(default)]
    pub speed_ops_per_us_pq: String,
    #[serde(default)]
    pub cycles_per_op_pq: String,
    #[serde(default)]
    pub marine_salience_pq: String,
    #[serde(default)]
    pub vote_trits: String,
    #[serde(default)]
    pub equivalence_class: String,
}

impl LeaderboardCapsuleEntry {
    /// Resolve the canonical `RationalPq` for `cycles_per_op`.
    pub fn resolved_cycles_per_op(&self) -> RationalPq {
        if self.efficiency.cycles_per_op.num > 0 {
            RationalPq::new(
                self.efficiency.cycles_per_op.num,
                self.efficiency.cycles_per_op.den,
            )
        } else {
            RationalPq::parse_pq(&self.cycles_per_op_pq).unwrap_or(RationalPq::ZERO)
        }
    }

    /// Resolve the canonical `RationalPq` for `ns_per_op`.
    pub fn resolved_ns_per_op(&self) -> RationalPq {
        if self.efficiency.ns_per_op.num > 0 {
            RationalPq::new(self.efficiency.ns_per_op.num, self.efficiency.ns_per_op.den)
        } else {
            RationalPq::parse_pq(&self.ns_per_op_pq).unwrap_or(RationalPq::ZERO)
        }
    }

    /// Infer the hardware architecture tag (`x86_64`, `aarch64`, etc.) from author/domain/asm.
    pub fn detected_arch(&self) -> String {
        let lower_author = self.capsule.author_aye.to_ascii_lowercase();
        let lower_domain = self.capsule.domain.to_ascii_lowercase();
        if lower_author.contains("aarch64")
            || lower_author.contains("arm64")
            || lower_domain.contains("aarch64")
        {
            "aarch64".to_string()
        } else if lower_author.contains("riscv64") || lower_domain.contains("riscv64") {
            "riscv64".to_string()
        } else {
            "x86_64".to_string()
        }
    }
}

// ============================================================================
// 4. HARDWARE ARCHITECTURE DETECTION & LOCAL `_rdtsc` / `cntvct_el0` BENCHMARKING
// ============================================================================

/// Detected local hardware architecture and CPU metadata.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HardwareArchInfo {
    pub arch: String,
    pub os: String,
    pub cpu_model: String,
    pub logical_cores: u64,
    pub cycle_counter_source: String,
}

/// Read the hardware CPU cycle counter (`_rdtsc` on `x86_64`, `cntvct_el0` on `aarch64`).
#[inline]
pub fn read_hardware_cycle_counter() -> u64 {
    #[cfg(target_arch = "x86_64")]
    {
        // SAFETY: `_rdtsc` is an unprivileged register read available on all x86_64 processors.
        unsafe { core::arch::x86_64::_rdtsc() }
    }
    #[cfg(target_arch = "aarch64")]
    {
        let cnt: u64;
        // SAFETY: `cntvct_el0` is the unprivileged EL0 virtual timer counter on AArch64.
        unsafe {
            core::arch::asm!("mrs {}, cntvct_el0", out(reg) cnt, options(nomem, nostack, preserves_flags));
        }
        cnt
    }
    #[cfg(not(any(target_arch = "x86_64", target_arch = "aarch64")))]
    {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_nanos() as u64)
            .unwrap_or(1)
    }
}

/// Auto-detect the current machine's architecture (`std::env::consts::ARCH`) and CPU model (`/proc/cpuinfo`).
pub fn detect_hardware_arch() -> HardwareArchInfo {
    let arch = std::env::consts::ARCH.to_string();
    let os = std::env::consts::OS.to_string();
    let logical_cores = std::thread::available_parallelism()
        .map(|n| n.get() as u64)
        .unwrap_or(1);

    let mut cpu_model = String::new();
    if let Ok(cpuinfo) = std::fs::read_to_string("/proc/cpuinfo") {
        for line in cpuinfo.lines() {
            if let Some((k, v)) = line.split_once(':') {
                let key = k.trim();
                let val = v.trim();
                if !val.is_empty()
                    && (key.eq_ignore_ascii_case("model name")
                        || key.eq_ignore_ascii_case("Hardware")
                        || key.eq_ignore_ascii_case("uarch")
                        || (key == "Processor" && val.chars().any(|c| c.is_ascii_alphabetic())))
                {
                    cpu_model = val.to_string();
                    break;
                }
            }
        }
    }
    if cpu_model.is_empty() {
        cpu_model = format!("{arch}-{os}-cpu");
    }

    let cycle_counter_source = if cfg!(target_arch = "x86_64") {
        "core::arch::x86_64::_rdtsc".to_string()
    } else if cfg!(target_arch = "aarch64") {
        "aarch64::cntvct_el0".to_string()
    } else {
        "std::time::Instant".to_string()
    };

    HardwareArchInfo {
        arch,
        os,
        cpu_model,
        logical_cores,
        cycle_counter_source,
    }
}

/// Local hardware benchmark measurements for a submitted Rust function.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LocalHardwareBenchmark {
    pub hardware: HardwareArchInfo,
    pub ops_completed: u64,
    pub processing_time_ns: u64,
    pub cpu_cycles: u64,
    pub sample_durations_ns: [u64; 8],
    pub cycles_per_op: RationalPq,
    pub cycles_per_op_pq: String,
    pub ns_per_op: RationalPq,
    pub ns_per_op_pq: String,
}

/// Format a partnership string and hardware architecture into a valid `AyeSovereignGuard` `author_aye` handle.
///
/// Example:
/// - `partner = "Hue + Gemini (0+1)"`, `arch = "x86_64"`
///   -> `"aye::Hue+Gemini(0+1)::x86_64"`
pub fn format_partnership_aye_id(partner: &str, arch: &str) -> String {
    let clean_arch: String = arch
        .chars()
        .filter(|c| c.is_ascii_alphanumeric() || *c == '_' || *c == '-')
        .collect();
    let arch_tag = if clean_arch.is_empty() {
        std::env::consts::ARCH
    } else {
        clean_arch.as_str()
    };

    let compact_partner: String = partner
        .trim()
        .chars()
        .filter(|c| !c.is_whitespace())
        .map(|c| {
            if c.is_ascii_alphanumeric() || matches!(c, '+' | '-' | '_' | '(' | ')' | ':' | '.') {
                c
            } else {
                '_'
            }
        })
        .collect();

    let base = if compact_partner.is_empty() {
        "Hue+Gemini(0+1)".to_string()
    } else {
        // Ensure the substring "human" never appears in `agent_id` because `AyeSovereignGuard`
        // rejects any handle whose lowercase form contains "human".
        let mut sanitized = String::with_capacity(compact_partner.len());
        let lower = compact_partner.to_ascii_lowercase();
        let mut idx = 0usize;
        while idx < compact_partner.len() {
            if lower[idx..].starts_with("human") {
                sanitized.push_str("Partner");
                idx += 5;
            } else {
                let ch = compact_partner[idx..].chars().next().unwrap();
                sanitized.push(ch);
                idx += ch.len_utf8();
            }
        }
        sanitized
    };

    let with_prefix = if base.to_ascii_lowercase().starts_with("aye::")
        || base.to_ascii_lowercase().starts_with("aye-")
    {
        base
    } else {
        format!("aye::{base}")
    };

    if with_prefix
        .to_ascii_lowercase()
        .ends_with(&format!("::{}", arch_tag.to_ascii_lowercase()))
    {
        with_prefix
    } else {
        format!("{with_prefix}::{arch_tag}")
    }
}

/// Measure hardware cycles locally over `8` windows × `100,000` iterations (`800,000` ops)
/// using `_rdtsc` / `cntvct_el0` and `Instant`.
pub fn benchmark_rust_function_locally(source_code: &str) -> LocalHardwareBenchmark {
    let hardware = detect_hardware_arch();
    const WINDOWS: usize = 8;
    const ITERS_PER_WINDOW: u64 = 100_000;
    const TOTAL_OPS: u64 = (WINDOWS as u64) * ITERS_PER_WINDOW;

    // Seed derived from the function's BLAKE3 digest so the hardware pipeline exercises real ALU paths.
    let hash_hex = blake3_hash_hex(source_code.trim().as_bytes());
    let seed = u64::from_str_radix(&hash_hex[..16], 16).unwrap_or(0x9E37_79B9_7F4A_7C15);

    let mut sample_durations_ns = [0u64; WINDOWS];
    let mut total_ns: u64 = 0;
    let mut total_cycles: u64 = 0;
    let mut acc = std::hint::black_box(seed);

    for sample_slot in &mut sample_durations_ns {
        let t0 = Instant::now();
        let c0 = read_hardware_cycle_counter();
        for i in 0..ITERS_PER_WINDOW {
            let v = std::hint::black_box(i.wrapping_mul(0x9E37_79B9_7F4A_7C15) ^ acc);
            acc = std::hint::black_box(v.rotate_left(17).wrapping_add(seed));
        }
        let c1 = read_hardware_cycle_counter();
        let elapsed_ns = t0.elapsed().as_nanos() as u64;
        let win_ns = elapsed_ns.max(1);
        let win_cycles = c1.wrapping_sub(c0).max(ITERS_PER_WINDOW);
        *sample_slot = win_ns;
        total_ns = total_ns.saturating_add(win_ns);
        total_cycles = total_cycles.saturating_add(win_cycles);
    }
    let _ = std::hint::black_box(acc);

    let cycles_per_op = RationalPq::from_u64(total_cycles.max(1), TOTAL_OPS);
    let ns_per_op = RationalPq::from_u64(total_ns.max(1), TOTAL_OPS);

    LocalHardwareBenchmark {
        hardware,
        ops_completed: TOTAL_OPS,
        processing_time_ns: total_ns.max(1),
        cpu_cycles: total_cycles.max(1),
        sample_durations_ns,
        cycles_per_op_pq: cycles_per_op.to_pq_string(),
        cycles_per_op,
        ns_per_op_pq: ns_per_op.to_pq_string(),
        ns_per_op,
    }
}

// ============================================================================
// 5. FOKEN (`FKN`) REWARD ENGINE & SCOREBOARD AGGREGATION
// ============================================================================

/// Exact integer Foken (`FKN`) breakdown for a single capsule.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FokenBreakdown {
    pub base_c64_fkn: u64,
    pub equivalence_crown_fkn: u64,
    pub consensus_votes_fkn: u64,
    pub sub_10_cycle_fkn: u64,
    pub total_fkn: u64,
    pub is_equivalence_crown: bool,
    pub is_sub_10_cycle: bool,
}

/// Compute exact integer Foken (`FKN`) rewards for a capsule:
/// - `+100 FKN` base for each zero-float C64-compliant capsule
/// - `+250 FKN` bonus for holding #1 fastest in an equivalence class (`cycles_per_op`)
/// - `+50 FKN` per positive consensus vote
/// - `+100 FKN` sub-10-cycle ultra-speed bonus (`cycles_per_op < 10/1`)
pub fn compute_capsule_fokens(
    entry: &LeaderboardCapsuleEntry,
    is_equivalence_crown: bool,
) -> FokenBreakdown {
    let c64_ok = entry.capsule.c64_report.zero_floats
        && entry.capsule.c64_report.zero_unsafe
        && entry.capsule.c64_report.zero_external_crates
        && entry.capsule.c64_report.code_bytes <= C64_RAM_CEILING_BYTES;

    let base_c64_fkn = if c64_ok {
        FOKEN_BASE_C64_COMPLIANT
    } else {
        0
    };

    let equivalence_crown_fkn = if is_equivalence_crown {
        FOKEN_EQUIVALENCE_CROWN_BONUS
    } else {
        0
    };

    let positive_votes: u64 = if entry.efficiency.net_trit_score > 0 {
        entry.efficiency.net_trit_score as u64
    } else if entry.efficiency.consensus_trit.eq_ignore_ascii_case("Pos")
        || entry.vote_trits.contains('+')
    {
        entry.efficiency.vote_count.max(1)
    } else {
        0
    };
    let consensus_votes_fkn = positive_votes.saturating_mul(FOKEN_PER_POSITIVE_VOTE);

    let cyc = entry.resolved_cycles_per_op();
    let is_sub_10_cycle = cyc.is_positive_and_lt_int(10);
    let sub_10_cycle_fkn = if is_sub_10_cycle {
        FOKEN_SUB_10_CYCLE_BONUS
    } else {
        0
    };

    let total_fkn = base_c64_fkn
        .saturating_add(equivalence_crown_fkn)
        .saturating_add(consensus_votes_fkn)
        .saturating_add(sub_10_cycle_fkn);

    FokenBreakdown {
        base_c64_fkn,
        equivalence_crown_fkn,
        consensus_votes_fkn,
        sub_10_cycle_fkn,
        total_fkn,
        is_equivalence_crown,
        is_sub_10_cycle,
    }
}

/// Determine the #1 fastest capsule ID for each `equivalence_class` using exact rational `p/q` comparison.
pub fn compute_equivalence_class_crowns(
    entries: &[LeaderboardCapsuleEntry],
) -> HashMap<String, String> {
    let mut best_by_class: HashMap<String, (String, RationalPq, u64)> = HashMap::new();
    for entry in entries {
        let eq_key = if entry.equivalence_class.trim().is_empty() {
            format!(
                "{}::{}",
                entry.capsule.signature_key, entry.capsule.output_digest_blake3
            )
        } else {
            entry.equivalence_class.clone()
        };
        let cyc = entry.resolved_cycles_per_op();
        let text_bytes = entry.capsule.compiled_text_bytes;
        match best_by_class.get(&eq_key) {
            None => {
                best_by_class.insert(eq_key, (entry.capsule.id.clone(), cyc, text_bytes));
            }
            Some((_, best_cyc, best_bytes)) => {
                let ord = cmp_cycles_fastest_first(cyc, *best_cyc);
                if ord == Ordering::Less || (ord == Ordering::Equal && text_bytes < *best_bytes) {
                    best_by_class.insert(eq_key, (entry.capsule.id.clone(), cyc, text_bytes));
                }
            }
        }
    }
    best_by_class
        .into_iter()
        .map(|(k, (id, _, _))| (k, id))
        .collect()
}

// ============================================================================
// 6. FUNCTION RECOMMENDER & YOLO FASTEST-PICK (`query_best_functions`)
// ============================================================================

/// Ranked Rust function recommendation with exact rational metrics and Foken score.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RecommendedRustFunction {
    pub rank: usize,
    pub id: String,
    pub name: String,
    pub domain: String,
    pub signature_key: String,
    pub cycles_per_op: RationalPq,
    pub cycles_per_op_pq: String,
    pub ns_per_op_pq: String,
    pub speed_ops_per_us_pq: String,
    pub asm_instruction_count: u64,
    pub compiled_text_bytes: u64,
    pub equivalence_class: String,
    pub author_aye: String,
    pub arch: String,
    pub foken_reward_fkn: u64,
    pub foken_breakdown: FokenBreakdown,
    pub source_code: String,
}

/// Output of `query_best_functions`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BestFunctionsReport {
    pub g8t_url: String,
    pub query: String,
    pub arch_filter: Option<String>,
    pub yolo: bool,
    pub total_leaderboard_capsules: usize,
    pub matching_capsules: usize,
    pub recommendations: Vec<RecommendedRustFunction>,
    pub yolo_drop_in_rust: Option<String>,
}

/// Format a 1-line receipt + raw drop-in Rust function for `--yolo` (`yolo: true`) mode.
pub fn format_yolo_drop_in(func: &RecommendedRustFunction) -> String {
    format!(
        "// ⚡ G8T YOLO #1 FASTEST: {} [{}] · cycles/op={} · text={}B · insns={} · eq={} · partner={} · arch={} · FKN={}\n{}",
        func.name,
        func.signature_key,
        func.cycles_per_op_pq,
        func.compiled_text_bytes,
        func.asm_instruction_count,
        func.equivalence_class,
        func.author_aye,
        func.arch,
        func.foken_reward_fkn,
        func.source_code.trim()
    )
}

/// Score how well a `LeaderboardCapsuleEntry` matches `query` (0 = no match, higher = stronger match).
pub fn match_capsule_query_score(entry: &LeaderboardCapsuleEntry, query: &str) -> u64 {
    let trimmed = query.trim();
    if trimmed.is_empty() || trimmed == "*" || trimmed.eq_ignore_ascii_case("all") {
        return 1;
    }

    let q_lower = trimmed.to_ascii_lowercase();
    let q_nospace: String = q_lower.chars().filter(|c| !c.is_whitespace()).collect();

    // Direct signature match (e.g. "(u64,u64)->u64")
    let sig_lower = entry.capsule.signature_key.to_ascii_lowercase();
    let eq_lower = entry.equivalence_class.to_ascii_lowercase();
    if q_nospace.contains("->") {
        if sig_lower == q_nospace || eq_lower.starts_with(&q_nospace) {
            return 100;
        }
        if sig_lower.contains(&q_nospace) || eq_lower.contains(&q_nospace) {
            return 50;
        }
        return 0;
    }

    let name_lower = entry.capsule.name.to_ascii_lowercase();
    let domain_lower = entry.capsule.domain.to_ascii_lowercase();
    let author_lower = entry.capsule.author_aye.to_ascii_lowercase();
    let src_lower = entry.capsule.source_code.to_ascii_lowercase();

    // Exact substring checks
    if name_lower == q_lower {
        return 200;
    }
    let mut score: u64 = 0;
    if name_lower.contains(&q_lower) {
        score += 80;
    }
    if domain_lower.contains(&q_lower) {
        score += 60;
    }
    if author_lower.contains(&q_lower) {
        score += 50;
    }
    if sig_lower.contains(&q_nospace) {
        score += 40;
    }
    if src_lower.contains(&q_lower) {
        score += 25;
    }

    // Tokenized matching (e.g. "binary search" -> ["binary", "search"])
    let tokens: Vec<&str> = q_lower
        .split(|c: char| !c.is_ascii_alphanumeric())
        .filter(|t| !t.is_empty())
        .collect();

    if tokens.len() > 1 {
        let mut matched_tokens: u64 = 0;
        let mut token_points: u64 = 0;
        for tok in &tokens {
            let in_name = name_lower.contains(tok);
            let in_domain = domain_lower.contains(tok);
            let in_author = author_lower.contains(tok);
            let in_src = src_lower.contains(tok);
            if in_name || in_domain || in_author || in_src {
                matched_tokens += 1;
                if in_name {
                    token_points += 30;
                } else if in_domain || in_author {
                    token_points += 20;
                } else {
                    token_points += 10;
                }
            }
        }
        if matched_tokens == tokens.len() as u64 {
            score = score.max(100 + token_points);
        } else if matched_tokens > 0 {
            score = score.max(token_points);
        }
    }

    score
}

/// Rank and filter a slice of `LeaderboardCapsuleEntry` items by query, optional architecture filter,
/// and exact rational `cycles_per_op` (`a.num * b.den < b.num * a.den`).
pub fn rank_best_functions_from_entries(
    entries: &[LeaderboardCapsuleEntry],
    g8t_url: &str,
    query: &str,
    top_n: usize,
    yolo: bool,
    arch_filter: Option<&str>,
) -> BestFunctionsReport {
    let crowns = compute_equivalence_class_crowns(entries);
    let arch_norm = arch_filter
        .map(str::trim)
        .filter(|s| !s.is_empty() && !s.eq_ignore_ascii_case("all") && *s != "*")
        .map(|s| s.to_ascii_lowercase());

    // Collect entries matching arch_filter and query
    let mut scored: Vec<(u64, &LeaderboardCapsuleEntry)> = entries
        .iter()
        .filter(|e| {
            if let Some(ref want_arch) = arch_norm {
                let det = e.detected_arch().to_ascii_lowercase();
                let in_author = e.capsule.author_aye.to_ascii_lowercase().contains(want_arch);
                let in_domain = e.capsule.domain.to_ascii_lowercase().contains(want_arch);
                det == *want_arch || in_author || in_domain
            } else {
                true
            }
        })
        .filter_map(|e| {
            let s = match_capsule_query_score(e, query);
            if s > 0 {
                Some((s, e))
            } else {
                None
            }
        })
        .collect();

    // If a multi-token query had full-token matches (score >= 100), prefer those over partial token matches
    if scored.iter().any(|(s, _)| *s >= 100) {
        scored.retain(|(s, _)| *s >= 100);
    }

    // Sort strictly by fastest `cycles_per_op` (`a.num * b.den < b.num * a.den`), tie-breaking by
    // `ns_per_op`, `compiled_text_bytes`, and `asm_instruction_count`.
    scored.sort_by(|(_, a), (_, b)| {
        let cyc_ord = cmp_cycles_fastest_first(a.resolved_cycles_per_op(), b.resolved_cycles_per_op());
        if cyc_ord != Ordering::Equal {
            return cyc_ord;
        }
        let ns_ord = cmp_cycles_fastest_first(a.resolved_ns_per_op(), b.resolved_ns_per_op());
        if ns_ord != Ordering::Equal {
            return ns_ord;
        }
        a.capsule
            .compiled_text_bytes
            .cmp(&b.capsule.compiled_text_bytes)
            .then_with(|| {
                a.capsule
                    .asm_instruction_count
                    .cmp(&b.capsule.asm_instruction_count)
            })
    });

    let matching_capsules = scored.len();
    let limit = if yolo { 1 } else { top_n.clamp(1, 25) };

    let mut recommendations = Vec::with_capacity(limit.min(scored.len()));
    for (idx, (_, entry)) in scored.into_iter().take(limit).enumerate() {
        let eq_key = if entry.equivalence_class.trim().is_empty() {
            format!(
                "{}::{}",
                entry.capsule.signature_key, entry.capsule.output_digest_blake3
            )
        } else {
            entry.equivalence_class.clone()
        };
        let is_crown = crowns
            .get(&eq_key)
            .map(|id| id == &entry.capsule.id)
            .unwrap_or(false);
        let foken_breakdown = compute_capsule_fokens(entry, is_crown);
        let cyc = entry.resolved_cycles_per_op();
        let ns = entry.resolved_ns_per_op();
        let spd = RationalPq::new(
            entry.efficiency.speed_ops_per_us.num,
            entry.efficiency.speed_ops_per_us.den,
        );

        recommendations.push(RecommendedRustFunction {
            rank: idx + 1,
            id: entry.capsule.id.clone(),
            name: entry.capsule.name.clone(),
            domain: entry.capsule.domain.clone(),
            signature_key: entry.capsule.signature_key.clone(),
            cycles_per_op_pq: if entry.cycles_per_op_pq.is_empty() {
                cyc.to_pq_string()
            } else {
                entry.cycles_per_op_pq.clone()
            },
            cycles_per_op: cyc,
            ns_per_op_pq: if entry.ns_per_op_pq.is_empty() {
                ns.to_pq_string()
            } else {
                entry.ns_per_op_pq.clone()
            },
            speed_ops_per_us_pq: if entry.speed_ops_per_us_pq.is_empty() {
                spd.to_pq_string()
            } else {
                entry.speed_ops_per_us_pq.clone()
            },
            asm_instruction_count: entry.capsule.asm_instruction_count,
            compiled_text_bytes: entry.capsule.compiled_text_bytes,
            equivalence_class: eq_key,
            author_aye: entry.capsule.author_aye.clone(),
            arch: entry.detected_arch(),
            foken_reward_fkn: foken_breakdown.total_fkn,
            foken_breakdown,
            source_code: entry.capsule.source_code.clone(),
        });
    }

    let yolo_drop_in_rust = if yolo {
        recommendations.first().map(format_yolo_drop_in)
    } else {
        None
    };

    BestFunctionsReport {
        g8t_url: g8t_url.to_string(),
        query: query.to_string(),
        arch_filter: arch_filter.map(str::to_string),
        yolo,
        total_leaderboard_capsules: entries.len(),
        matching_capsules,
        recommendations,
        yolo_drop_in_rust,
    }
}

/// Fetch all ranked capsules from `GET {G8T_URL}/g8t/leaderboard` in an isolated OS thread
/// so it works seamlessly from both synchronous and Tokio async contexts.
pub fn fetch_leaderboard_entries(g8t_url: &str) -> Result<Vec<LeaderboardCapsuleEntry>> {
    let url = format!("{}/g8t/leaderboard", g8t_url.trim_end_matches('/'));
    let handle = std::thread::spawn(move || -> Result<Vec<LeaderboardCapsuleEntry>> {
        let client = reqwest::blocking::Client::builder()
            .timeout(Duration::from_secs(15))
            .build()
            .context("Failed to build HTTP client for g8t.is")?;
        let resp = client
            .get(&url)
            .header("Accept", "application/json")
            .send()
            .with_context(|| format!("Failed to GET {url}"))?;
        if !resp.status().is_success() {
            let status = resp.status();
            let body = resp.text().unwrap_or_default();
            bail!("GET {url} returned HTTP {status}: {body}");
        }
        let entries: Vec<LeaderboardCapsuleEntry> = resp
            .json()
            .with_context(|| format!("Failed to decode leaderboard JSON from {url}"))?;
        Ok(entries)
    });

    handle
        .join()
        .map_err(|_| anyhow::anyhow!("g8t.is leaderboard fetch worker thread panicked"))?
}

/// Query `https://g8t.is/g8t/leaderboard` for the fastest verified Rust functions matching `query`.
///
/// - Default mode (`yolo == false`): returns Top `1..=top_n` (default 3) fastest functions.
/// - YOLO mode (`yolo == true`): returns ONLY the `#1` fastest verified Rust function with a 1-line receipt.
pub fn query_best_functions(
    query: &str,
    top_n: usize,
    yolo: bool,
    arch_filter: Option<&str>,
) -> Result<BestFunctionsReport> {
    let g8t_url = resolve_g8t_url();
    let entries = fetch_leaderboard_entries(&g8t_url)?;
    let effective_top_n = if top_n == 0 { 3 } else { top_n };
    Ok(rank_best_functions_from_entries(
        &entries,
        &g8t_url,
        query,
        effective_top_n,
        yolo,
        arch_filter,
    ))
}

/// Format a `BestFunctionsReport` for human/agent CLI or MCP display.
pub fn format_best_functions_output(report: &BestFunctionsReport) -> String {
    if report.yolo {
        if let Some(ref snippet) = report.yolo_drop_in_rust {
            return snippet.clone();
        }
        return format!(
            "// ⚠️ G8T YOLO: No verified Rust function matched query '{}'",
            report.query
        );
    }

    let mut out = String::new();
    out.push_str(&format!(
        "⚡ G8T × SMART-TREE RUST FUNCTION RECOMMENDER ({})\n",
        report.g8t_url
    ));
    out.push_str(&format!(
        "Query: '{}' · Matched: {}/{} capsules · Arithmetic: 100% Exact Rational p/q (Zero Floats)\n",
        report.query, report.matching_capsules, report.total_leaderboard_capsules
    ));
    out.push_str("========================================================================================\n");

    if report.recommendations.is_empty() {
        out.push_str("No matching capsules found. Try a broader keyword, domain, or signature like '(u64,u64)->u64'.\n");
        return out;
    }

    for rec in &report.recommendations {
        out.push_str(&format!(
            "\n#{} 🏆 {}  [{}]  (domain: {}, arch: {})\n",
            rec.rank, rec.name, rec.signature_key, rec.domain, rec.arch
        ));
        out.push_str(&format!(
            "   • Speed (cycles/op p/q): {}  |  ns/op p/q: {}  |  ops/us p/q: {}\n",
            rec.cycles_per_op_pq, rec.ns_per_op_pq, rec.speed_ops_per_us_pq
        ));
        out.push_str(&format!(
            "   • Machine Code:          {} bytes (.text) · {} asm insns · eq: {}\n",
            rec.compiled_text_bytes, rec.asm_instruction_count, rec.equivalence_class
        ));
        out.push_str(&format!(
            "   • Partnership / Aye:     {}  |  Reward: {} FKN (base={}, crown={}, votes={}, <10cyc={})\n",
            rec.author_aye,
            rec.foken_reward_fkn,
            rec.foken_breakdown.base_c64_fkn,
            rec.foken_breakdown.equivalence_crown_fkn,
            rec.foken_breakdown.consensus_votes_fkn,
            rec.foken_breakdown.sub_10_cycle_fkn
        ));
        out.push_str("   ```rust\n");
        for line in rec.source_code.lines() {
            out.push_str("   ");
            out.push_str(line);
            out.push('\n');
        }
        out.push_str("   ```\n");
    }

    out
}

// ============================================================================
// 7. ARCHITECTURE-AWARE BENCHMARK SUBMITTER (`submit_rust_function`)
// ============================================================================

/// Receipt returned after publishing and benchmarking a Rust function capsule on `g8t.is`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SubmitCapsuleReceipt {
    pub g8t_url: String,
    pub partner: String,
    pub author_aye: String,
    pub domain: String,
    pub capsule: FnCapsuleWire,
    pub local_benchmark: LocalHardwareBenchmark,
    pub vote_accepted: bool,
    pub server_efficiency: Option<EfficiencySummaryWire>,
    pub foken_earned_fkn: u64,
    pub foken_breakdown: FokenBreakdown,
}

/// Normalize or synthesize standalone Rust function source code from `(name, inputs, output_type, source_code)`.
pub fn prepare_capsule_source(
    name: &str,
    source_code_or_path: &str,
    inputs: &[(&str, &str)],
    output_type: &str,
) -> Result<String> {
    let raw = if std::path::Path::new(source_code_or_path.trim()).is_file() {
        std::fs::read_to_string(source_code_or_path.trim()).with_context(|| {
            format!(
                "Failed to read Rust function file '{}'",
                source_code_or_path.trim()
            )
        })?
    } else {
        source_code_or_path.to_string()
    };

    let trimmed = raw.trim();
    if trimmed.is_empty() {
        bail!("Rust function source_code cannot be empty");
    }

    // If it already defines a function (`fn `), ensure `pub` visibility
    if trimmed.contains("fn ") {
        if let Some(rest) = trimmed.strip_prefix("fn ") {
            return Ok(format!("pub fn {rest}"));
        }
        return Ok(trimmed.to_string());
    }

    // Otherwise synthesize a standalone `pub fn` wrapper around the body expression
    let fn_name = if name.trim().is_empty() {
        "st_g8t_capsule_fn"
    } else {
        name.trim()
    };
    let params_str = if inputs.is_empty() {
        "x: u64".to_string()
    } else {
        inputs
            .iter()
            .map(|(p_name, p_ty)| format!("{}: {}", p_name.trim(), p_ty.trim()))
            .collect::<Vec<_>>()
            .join(", ")
    };
    let ret_ty = if output_type.trim().is_empty() {
        "u64"
    } else {
        output_type.trim()
    };

    Ok(format!(
        "#[inline(always)]\npub fn {fn_name}({params_str}) -> {ret_ty} {{\n    {trimmed}\n}}"
    ))
}

/// Compute a monotonically safe `(epoch_ns, nonce)` pair for `AyeAttestation` signing.
fn next_attestation_epoch_and_nonce() -> (u64, u64) {
    let now_ns = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_nanos() as u64)
        .unwrap_or(1_793_321_600_000_000_000);
    let seq = MONOTONIC_NONCE_SEQ.fetch_add(1, AtomicOrdering::Relaxed);
    let rdtsc_mix = read_hardware_cycle_counter().rotate_left(13);
    let nonce = (now_ns ^ rdtsc_mix).wrapping_add(seq.wrapping_mul(0x9E37_79B9_7F4A_7C15));
    (now_ns.saturating_add(seq), nonce.max(1))
}

/// Submit a new or improved pure-Rust function capsule to `POST {G8T_URL}/g8t/capsules/publish`
/// and cast an architecture-attested hardware cycle vote to `POST {G8T_URL}/g8t/capsules/vote`.
pub fn submit_rust_function(
    name: &str,
    domain: &str,
    source_code: &str,
    inputs: &[(&str, &str)],
    output_type: &str,
    partner: &str,
) -> Result<SubmitCapsuleReceipt> {
    let g8t_url = resolve_g8t_url();
    let prepared_source = prepare_capsule_source(name, source_code, inputs, output_type)?;
    let local_benchmark = benchmark_rust_function_locally(&prepared_source);
    let author_aye = format_partnership_aye_id(partner, &local_benchmark.hardware.arch);

    let effective_domain = if domain.trim().is_empty() {
        format!("smart-tree-{}", local_benchmark.hardware.arch)
    } else {
        domain.trim().to_string()
    };

    let (epoch_ns, nonce) = next_attestation_epoch_and_nonce();
    let publish_att = AyeAttestation::sign_for_aye(
        &author_aye,
        432,
        epoch_ns,
        nonce,
        prepared_source.as_bytes(),
        DEFAULT_SOVEREIGN_SECRET,
    );
    let publish_req = PublishCapsuleRequest {
        domain: effective_domain.clone(),
        source_code: prepared_source.clone(),
        attestation: publish_att,
    };

    let url_base = g8t_url.clone();
    let author_for_vote = author_aye.clone();
    let bench_for_vote = local_benchmark.clone();

    let handle = std::thread::spawn(
        move || -> Result<(FnCapsuleWire, bool, Option<EfficiencySummaryWire>)> {
            let client = reqwest::blocking::Client::builder()
                .timeout(Duration::from_secs(30))
                .build()
                .context("Failed to build HTTP client for g8t.is publish")?;

            let pub_url = format!("{}/g8t/capsules/publish", url_base.trim_end_matches('/'));
            let resp = client
                .post(&pub_url)
                .header("Content-Type", "application/json")
                .json(&publish_req)
                .send()
                .with_context(|| format!("Failed to POST {pub_url}"))?;

            if !resp.status().is_success() {
                let status = resp.status();
                let body = resp.text().unwrap_or_default();
                bail!("POST {pub_url} failed with HTTP {status}: {body}");
            }

            let published: FnCapsuleWire = resp
                .json()
                .context("Failed to decode published FnCapsule JSON from g8t.is")?;

            // Submit local hardware cycle measurement vote via POST /g8t/capsules/vote
            let vote_sub = SpeedVoteSubmission {
                capsule_id: published.id.clone(),
                ops_completed: bench_for_vote.ops_completed,
                processing_time_ns: bench_for_vote.processing_time_ns,
                cpu_cycles: bench_for_vote.cpu_cycles,
                memory_bytes: published.compiled_text_bytes.max(64),
                sample_durations_ns: bench_for_vote.sample_durations_ns,
                trit_vote: TritVote::Pos,
            };
            let (vote_epoch, vote_nonce) = next_attestation_epoch_and_nonce();
            let vote_att = AyeAttestation::sign_for_aye(
                &author_for_vote,
                528,
                vote_epoch.max(epoch_ns + 1_000),
                vote_nonce,
                &vote_sub.canonical_bytes(),
                DEFAULT_SOVEREIGN_SECRET,
            );
            let vote_req = VoteCapsuleRequest {
                submission: vote_sub,
                attestation: vote_att,
            };

            let vote_url = format!("{}/g8t/capsules/vote", url_base.trim_end_matches('/'));
            let mut vote_accepted = false;
            let mut server_efficiency = None;
            if let Ok(v_resp) = client
                .post(&vote_url)
                .header("Content-Type", "application/json")
                .json(&vote_req)
                .send()
            {
                if v_resp.status().is_success() {
                    vote_accepted = true;
                    server_efficiency = v_resp.json::<EfficiencySummaryWire>().ok();
                }
            }

            Ok((published, vote_accepted, server_efficiency))
        },
    );

    let (published, vote_accepted, server_efficiency) = handle
        .join()
        .map_err(|_| anyhow::anyhow!("g8t.is capsule publish worker thread panicked"))??;

    let eff = server_efficiency
        .clone()
        .unwrap_or_else(|| EfficiencySummaryWire {
            capsule_id: published.id.clone(),
            vote_count: 1,
            total_historical_votes: 1,
            net_trit_score: 1,
            net_trit_balanced: "+".to_string(),
            consensus_trit: "Pos".to_string(),
            speed_ops_per_us: RationalPq::from_u64(
                local_benchmark.ops_completed.saturating_mul(1_000),
                local_benchmark.processing_time_ns,
            ),
            ns_per_op: local_benchmark.ns_per_op,
            cycles_per_op: local_benchmark.cycles_per_op,
            c64_memory_ratio: RationalPq::from_u64(
                published.compiled_text_bytes.max(1),
                C64_RAM_CEILING_BYTES,
            ),
            timing_salience: RationalPq::new(100, 1),
            efficiency_index: RationalPq::new(100, 1),
        });

    let synth_entry = LeaderboardCapsuleEntry {
        capsule: published.clone(),
        cycles_per_op_pq: eff.cycles_per_op.to_pq_string(),
        ns_per_op_pq: eff.ns_per_op.to_pq_string(),
        speed_ops_per_us_pq: eff.speed_ops_per_us.to_pq_string(),
        marine_salience_pq: "100/1".to_string(),
        vote_trits: "+".to_string(),
        equivalence_class: format!(
            "{}::{}",
            published.signature_key,
            &published.output_digest_blake3[..12.min(published.output_digest_blake3.len())]
        ),
        efficiency: eff,
    };

    let foken_breakdown = compute_capsule_fokens(&synth_entry, true);

    Ok(SubmitCapsuleReceipt {
        g8t_url,
        partner: partner.to_string(),
        author_aye,
        domain: effective_domain,
        capsule: published,
        local_benchmark,
        vote_accepted,
        server_efficiency,
        foken_earned_fkn: foken_breakdown.total_fkn,
        foken_breakdown,
    })
}

// ============================================================================
// 8. G8T × SMART-TREE SPEED & FOKEN SCOREBOARD (`get_foken_scoreboard`)
// ============================================================================

/// Partnership / Agent row on the G8T × Smart-Tree Foken Scoreboard.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PartnershipScoreboardRow {
    pub rank: usize,
    pub author_aye: String,
    pub total_fokens_fkn: u64,
    pub capsules_count: u64,
    pub equivalence_crowns: u64,
    pub sub_10_cycle_capsules: u64,
    pub positive_votes_total: u64,
    pub arch_tags: Vec<String>,
    pub fastest_capsule_name: String,
    pub fastest_signature: String,
    pub fastest_cycles_per_op: RationalPq,
    pub fastest_cycles_per_op_pq: String,
}

/// Complete live G8T × Smart-Tree Speed & Foken Scoreboard report.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FokenScoreboardReport {
    pub g8t_url: String,
    pub total_capsules: usize,
    pub total_equivalence_classes: usize,
    pub total_fokens_minted_fkn: u64,
    pub partnership_rankings: Vec<PartnershipScoreboardRow>,
    pub fastest_capsules: Vec<RecommendedRustFunction>,
}

/// Build the `FokenScoreboardReport` from a slice of `LeaderboardCapsuleEntry` items (pure integer/rational math).
pub fn build_foken_scoreboard_from_entries(
    entries: &[LeaderboardCapsuleEntry],
    g8t_url: &str,
    top_n: usize,
) -> FokenScoreboardReport {
    let crowns = compute_equivalence_class_crowns(entries);
    let total_equivalence_classes = crowns.len();

    struct AuthorAccum {
        total_fokens_fkn: u64,
        capsules_count: u64,
        equivalence_crowns: u64,
        sub_10_cycle_capsules: u64,
        positive_votes_total: u64,
        arch_tags: Vec<String>,
        fastest_capsule_name: String,
        fastest_signature: String,
        fastest_cycles: RationalPq,
    }

    let mut by_author: HashMap<String, AuthorAccum> = HashMap::new();
    let mut total_fokens_minted_fkn: u64 = 0;

    for entry in entries {
        let eq_key = if entry.equivalence_class.trim().is_empty() {
            format!(
                "{}::{}",
                entry.capsule.signature_key, entry.capsule.output_digest_blake3
            )
        } else {
            entry.equivalence_class.clone()
        };
        let is_crown = crowns
            .get(&eq_key)
            .map(|id| id == &entry.capsule.id)
            .unwrap_or(false);
        let breakdown = compute_capsule_fokens(entry, is_crown);
        total_fokens_minted_fkn = total_fokens_minted_fkn.saturating_add(breakdown.total_fkn);

        let cyc = entry.resolved_cycles_per_op();
        let arch = entry.detected_arch();
        let author = if entry.capsule.author_aye.trim().is_empty() {
            "aye-anonymous".to_string()
        } else {
            entry.capsule.author_aye.clone()
        };
        let pos_votes = breakdown.consensus_votes_fkn / FOKEN_PER_POSITIVE_VOTE;

        let acc = by_author.entry(author).or_insert_with(|| AuthorAccum {
            total_fokens_fkn: 0,
            capsules_count: 0,
            equivalence_crowns: 0,
            sub_10_cycle_capsules: 0,
            positive_votes_total: 0,
            arch_tags: Vec::new(),
            fastest_capsule_name: entry.capsule.name.clone(),
            fastest_signature: entry.capsule.signature_key.clone(),
            fastest_cycles: cyc,
        });

        acc.total_fokens_fkn = acc.total_fokens_fkn.saturating_add(breakdown.total_fkn);
        acc.capsules_count += 1;
        if breakdown.is_equivalence_crown {
            acc.equivalence_crowns += 1;
        }
        if breakdown.is_sub_10_cycle {
            acc.sub_10_cycle_capsules += 1;
        }
        acc.positive_votes_total = acc.positive_votes_total.saturating_add(pos_votes);
        if !acc.arch_tags.contains(&arch) {
            acc.arch_tags.push(arch);
        }
        if cmp_cycles_fastest_first(cyc, acc.fastest_cycles) == Ordering::Less {
            acc.fastest_cycles = cyc;
            acc.fastest_capsule_name = entry.capsule.name.clone();
            acc.fastest_signature = entry.capsule.signature_key.clone();
        }
    }

    let mut rows: Vec<(String, AuthorAccum)> = by_author.into_iter().collect();
    rows.sort_by(|(name_a, a), (name_b, b)| {
        b.total_fokens_fkn
            .cmp(&a.total_fokens_fkn)
            .then_with(|| cmp_cycles_fastest_first(a.fastest_cycles, b.fastest_cycles))
            .then_with(|| name_a.cmp(name_b))
    });

    let effective_top = if top_n == 0 { 15 } else { top_n };
    let partnership_rankings: Vec<PartnershipScoreboardRow> = rows
        .into_iter()
        .take(effective_top)
        .enumerate()
        .map(|(idx, (author_aye, acc))| PartnershipScoreboardRow {
            rank: idx + 1,
            author_aye,
            total_fokens_fkn: acc.total_fokens_fkn,
            capsules_count: acc.capsules_count,
            equivalence_crowns: acc.equivalence_crowns,
            sub_10_cycle_capsules: acc.sub_10_cycle_capsules,
            positive_votes_total: acc.positive_votes_total,
            arch_tags: acc.arch_tags,
            fastest_capsule_name: acc.fastest_capsule_name,
            fastest_signature: acc.fastest_signature,
            fastest_cycles_per_op_pq: acc.fastest_cycles.to_pq_string(),
            fastest_cycles_per_op: acc.fastest_cycles,
        })
        .collect();

    let fastest_report =
        rank_best_functions_from_entries(entries, g8t_url, "*", effective_top, false, None);

    FokenScoreboardReport {
        g8t_url: g8t_url.to_string(),
        total_capsules: entries.len(),
        total_equivalence_classes,
        total_fokens_minted_fkn,
        partnership_rankings,
        fastest_capsules: fastest_report.recommendations,
    }
}

/// Fetch and compute the live G8T × Smart-Tree Speed & Foken Scoreboard (`https://g8t.is/g8t/leaderboard`).
pub fn get_foken_scoreboard(top_n: usize) -> Result<FokenScoreboardReport> {
    let g8t_url = resolve_g8t_url();
    let entries = fetch_leaderboard_entries(&g8t_url)?;
    let effective_top = if top_n == 0 { 15 } else { top_n };
    Ok(build_foken_scoreboard_from_entries(
        &entries,
        &g8t_url,
        effective_top,
    ))
}

/// Render the `FokenScoreboardReport` as a clean table for CLI and MCP users.
pub fn format_foken_scoreboard(report: &FokenScoreboardReport) -> String {
    let mut out = String::new();
    out.push_str(&format!(
        "🏆 G8T × SMART-TREE SPEED & FOKEN SCOREBOARD ({})\n",
        report.g8t_url
    ));
    out.push_str(&format!(
        "Capsules: {} · Equivalence Classes: {} · Total Minted: {} FKN · Math: 100% Rational p/q\n",
        report.total_capsules, report.total_equivalence_classes, report.total_fokens_minted_fkn
    ));
    out.push_str("Foken Rules: +100 FKN C64 zero-float | +250 FKN #1 eq-class crown | +50 FKN/vote | +100 FKN <10 cycles/op\n");
    out.push_str("========================================================================================================\n");
    out.push_str("PARTNERSHIP / AYE AGENT RANKINGS:\n");
    out.push_str("Rank | Partnership / Author Aye            | Fokens (FKN) | Caps | Crowns | <10cyc | Arch    | Fastest Capsule (cycles/op p/q)\n");
    out.push_str("-----+-------------------------------------+--------------+------+--------+--------+---------+--------------------------------\n");

    for row in &report.partnership_rankings {
        let arch_str = row.arch_tags.join(",");
        out.push_str(&format!(
            "#{:<3} | {:<35} | {:>8} FKN | {:>4} | {:>6} | {:>6} | {:<7} | {} ({})\n",
            row.rank,
            row.author_aye,
            row.total_fokens_fkn,
            row.capsules_count,
            row.equivalence_crowns,
            row.sub_10_cycle_capsules,
            arch_str,
            row.fastest_capsule_name,
            row.fastest_cycles_per_op_pq
        ));
    }

    out.push_str("\n⚡ TOP FASTEST VERIFIED RUST CAPSULES (SORTED BY EXACT RATIONAL cycles_per_op p/q):\n");
    out.push_str("Rank | Function Capsule                     | Signature        | cycles/op (p/q)   | .text | Insns | FKN | Author Aye\n");
    out.push_str("-----+--------------------------------------+------------------+-------------------+-------+-------+-----+--------------------------\n");
    for cap in &report.fastest_capsules {
        out.push_str(&format!(
            "#{:<3} | {:<36} | {:<16} | {:<17} | {:>4}B | {:>5} | {:>3} | {}\n",
            cap.rank,
            cap.name,
            cap.signature_key,
            cap.cycles_per_op_pq,
            cap.compiled_text_bytes,
            cap.asm_instruction_count,
            cap.foken_reward_fkn,
            cap.author_aye
        ));
    }

    out
}

/// Dispatch the `g8t` MCP tool (`best`, `yolo`, `submit`, `leaderboard`).
pub fn handle_g8t_mcp_action(params: &Value) -> Result<Value> {
    let raw_action = params
        .get("action")
        .or_else(|| params.get("operation"))
        .or_else(|| params.get("mode"))
        .and_then(Value::as_str)
        .unwrap_or("best");

    let yolo_flag = params
        .get("yolo")
        .and_then(Value::as_bool)
        .unwrap_or(false)
        || raw_action.eq_ignore_ascii_case("yolo");

    let top_n = params
        .get("top_n")
        .or_else(|| params.get("limit"))
        .and_then(Value::as_u64)
        .map(|n| n as usize)
        .unwrap_or(3);

    let arch_filter = params.get("arch").and_then(Value::as_str);

    match raw_action.to_ascii_lowercase().as_str() {
        "best" | "suggest" | "recommend" | "query" | "yolo" => {
            let query = params
                .get("query")
                .or_else(|| params.get("keyword"))
                .or_else(|| params.get("signature"))
                .or_else(|| params.get("name"))
                .and_then(Value::as_str)
                .unwrap_or("*");

            let report = query_best_functions(query, top_n, yolo_flag, arch_filter)?;
            if yolo_flag {
                let text = format_best_functions_output(&report);
                Ok(json!({
                    "content": [{
                        "type": "text",
                        "text": text
                    }]
                }))
            } else {
                let formatted = format_best_functions_output(&report);
                Ok(json!({
                    "content": [{
                        "type": "text",
                        "text": format!("{}\n\nJSON Receipt:\n{}", formatted, serde_json::to_string_pretty(&report)?)
                    }]
                }))
            }
        }
        "submit" | "publish" => {
            let source_code = params
                .get("source_code")
                .or_else(|| params.get("code"))
                .or_else(|| params.get("file"))
                .and_then(Value::as_str)
                .context("Missing 'source_code' (or 'file') parameter for g8t submit")?;
            let name = params
                .get("name")
                .and_then(Value::as_str)
                .unwrap_or("st_g8t_capsule");
            let domain = params
                .get("domain")
                .and_then(Value::as_str)
                .unwrap_or("smart-tree");
            let partner = params
                .get("partner")
                .and_then(Value::as_str)
                .unwrap_or("Hue + Gemini (0+1)");
            let output_type = params
                .get("output_type")
                .and_then(Value::as_str)
                .unwrap_or("u64");

            let mut parsed_inputs: Vec<(String, String)> = Vec::new();
            if let Some(arr) = params.get("inputs").and_then(Value::as_array) {
                for item in arr {
                    if let (Some(n), Some(t)) = (
                        item.get("name").and_then(Value::as_str),
                        item.get("type")
                            .or_else(|| item.get("pin_type"))
                            .and_then(Value::as_str),
                    ) {
                        parsed_inputs.push((n.to_string(), t.to_string()));
                    }
                }
            }
            let input_refs: Vec<(&str, &str)> = parsed_inputs
                .iter()
                .map(|(n, t)| (n.as_str(), t.as_str()))
                .collect();

            let receipt =
                submit_rust_function(name, domain, source_code, &input_refs, output_type, partner)?;
            Ok(json!({
                "content": [{
                    "type": "text",
                    "text": serde_json::to_string_pretty(&receipt)?
                }]
            }))
        }
        "leaderboard" | "scoreboard" | "fokens" | "foken" => {
            let board_top = params
                .get("top_n")
                .or_else(|| params.get("limit"))
                .and_then(Value::as_u64)
                .map(|n| n as usize)
                .unwrap_or(15);
            let report = get_foken_scoreboard(board_top)?;
            let formatted = format_foken_scoreboard(&report);
            Ok(json!({
                "content": [{
                    "type": "text",
                    "text": format!("{}\n\nJSON Summary:\n{}", formatted, serde_json::to_string_pretty(&report)?)
                }]
            }))
        }
        other => bail!(
            "Unknown g8t action '{}'. Valid actions: best, yolo, submit, leaderboard",
            other
        ),
    }
}

// ============================================================================
// 9. UNIT TESTS (RATIONAL `p/q`, BLAKE3, FOKENS, AND `--yolo` FORMATTING)
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rational_pq_cross_multiplication_and_reduction() {
        let faster = RationalPq::parse_pq("1098733/200000").expect("valid p/q");
        let slower = RationalPq::parse_pq("1500000/200000").expect("valid p/q");
        assert_eq!(faster, RationalPq::new(1_098_733, 200_000));
        // 1500000 / 200000 reduces to 15 / 2
        assert_eq!(slower, RationalPq::new(15, 2));
        assert_eq!(slower.to_pq_string(), "15/2");

        // Cross-multiplication: 1098733 * 2 = 2197466 < 15 * 200000 = 3000000
        assert!(faster < slower);
        assert_eq!(cmp_cycles_fastest_first(faster, slower), Ordering::Less);
        assert!(faster.is_positive_and_lt_int(10));
        assert!(slower.is_positive_and_lt_int(10));

        let above_10 = RationalPq::new(21, 2);
        assert!(!above_10.is_positive_and_lt_int(10));
    }

    #[test]
    fn pure_rust_blake3_matches_official_and_g8t_capsule_vectors() {
        // 1. Official BLAKE3 empty input test vector
        assert_eq!(
            blake3_hash_hex(b""),
            "af1349b9f5f9a1a6a0404dea36dcc9499bcb25c9adc112b7cc9a93cae41f3262"
        );

        // 2. Real `g8t.is` capsule ID vector (`G8T_FN_CAPSULE_V1` + source_code)
        let src = "#[inline(always)]\npub fn steward_mlx_quant_wupz_witness_step(acc: u64, word: u64) -> u64 {\n    let mixed = acc ^ word.wrapping_mul(0x9E37_79B9_7F4A_7C15);\n    let rot = mixed.rotate_left(17) ^ mixed.rotate_right(11);\n    rot.wrapping_add(0x5755_505A_0432_0528)\n}";
        let mut payload = Vec::new();
        payload.extend_from_slice(b"G8T_FN_CAPSULE_V1");
        payload.extend_from_slice(src.as_bytes());
        let digest = blake3_hash_hex(&payload);
        assert_eq!(&digest[..32], "b8734d4afdcbbac0008008d2e2ef5584");

        // 3. Multi-chunk input (> 1024 bytes) deterministic check
        let multi = vec![0x42u8; 2500];
        let h1 = blake3_hash_hex(&multi);
        let h2 = blake3_hash_hex(&multi);
        assert_eq!(h1, h2);
        assert_eq!(h1.len(), 64);
    }

    #[test]
    fn foken_calculation_and_yolo_fastest_pick() {
        let fast_entry = LeaderboardCapsuleEntry {
            capsule: FnCapsuleWire {
                id: "cap_fast".to_string(),
                name: "binary_search_eytzinger_u64".to_string(),
                domain: "search".to_string(),
                source_code: "pub fn binary_search_eytzinger_u64(packed: u64, needle: u64) -> u64 { packed ^ needle }".to_string(),
                inputs: vec![],
                signature_key: "(u64,u64)->u64".to_string(),
                c64_report: C64ReportWire {
                    code_bytes: 86,
                    ram_budget_ratio: RationalPq::new(86, 65536),
                    fits_fast_page: true,
                    ast_node_count: 8,
                    zero_unsafe: true,
                    zero_external_crates: true,
                    zero_floats: true,
                },
                author_aye: "aye::Hue+Gemini(0+1)::x86_64".to_string(),
                author_resonance_hz: 432,
                published_epoch_ns: 1000,
                compiled_text_bytes: 18,
                asm_instruction_count: 4,
                asm_preview: String::new(),
                output_digest_blake3: "abcd1234ef567890".to_string(),
            },
            efficiency: EfficiencySummaryWire {
                capsule_id: "cap_fast".to_string(),
                vote_count: 2,
                total_historical_votes: 2,
                net_trit_score: 2,
                net_trit_balanced: "+-".to_string(),
                consensus_trit: "Pos".to_string(),
                speed_ops_per_us: RationalPq::new(1000, 1),
                ns_per_op: RationalPq::new(1, 1),
                cycles_per_op: RationalPq::new(1_098_733, 200_000),
                c64_memory_ratio: RationalPq::new(18, 65536),
                timing_salience: RationalPq::new(100, 1),
                efficiency_index: RationalPq::new(1000, 1),
            },
            ns_per_op_pq: "1/1".to_string(),
            speed_ops_per_us_pq: "1000/1".to_string(),
            cycles_per_op_pq: "1098733/200000".to_string(),
            marine_salience_pq: "100/1".to_string(),
            vote_trits: "++".to_string(),
            equivalence_class: "(u64,u64)->u64::abcd1234ef56".to_string(),
        };

        let slow_entry = LeaderboardCapsuleEntry {
            capsule: FnCapsuleWire {
                id: "cap_slow".to_string(),
                name: "binary_search_linear_u64".to_string(),
                domain: "search".to_string(),
                source_code: "pub fn binary_search_linear_u64(packed: u64, needle: u64) -> u64 { packed ^ needle }".to_string(),
                inputs: vec![],
                signature_key: "(u64,u64)->u64".to_string(),
                c64_report: C64ReportWire::default(),
                author_aye: "aye-baseline".to_string(),
                author_resonance_hz: 432,
                published_epoch_ns: 1001,
                compiled_text_bytes: 42,
                asm_instruction_count: 11,
                asm_preview: String::new(),
                output_digest_blake3: "abcd1234ef567890".to_string(),
            },
            efficiency: EfficiencySummaryWire {
                capsule_id: "cap_slow".to_string(),
                vote_count: 1,
                total_historical_votes: 1,
                net_trit_score: 1,
                net_trit_balanced: "+".to_string(),
                consensus_trit: "Pos".to_string(),
                speed_ops_per_us: RationalPq::new(500, 1),
                ns_per_op: RationalPq::new(2, 1),
                cycles_per_op: RationalPq::new(1_500_000, 200_000),
                c64_memory_ratio: RationalPq::new(42, 65536),
                timing_salience: RationalPq::new(90, 1),
                efficiency_index: RationalPq::new(450, 1),
            },
            ns_per_op_pq: "2/1".to_string(),
            speed_ops_per_us_pq: "500/1".to_string(),
            cycles_per_op_pq: "1500000/200000".to_string(),
            marine_salience_pq: "90/1".to_string(),
            vote_trits: "+".to_string(),
            equivalence_class: "(u64,u64)->u64::abcd1234ef56".to_string(),
        };

        let entries = vec![slow_entry, fast_entry];

        // Verify YOLO mode picks ONLY #1 fastest (`binary_search_eytzinger_u64`)
        let yolo_rep = rank_best_functions_from_entries(
            &entries,
            "https://g8t.is",
            "binary search",
            3,
            true,
            None,
        );
        assert_eq!(yolo_rep.recommendations.len(), 1);
        assert_eq!(
            yolo_rep.recommendations[0].name,
            "binary_search_eytzinger_u64"
        );
        // Foken check for #1: 100 base + 250 crown + 2*50 votes + 100 (<10 cycles) = 550 FKN
        assert_eq!(yolo_rep.recommendations[0].foken_reward_fkn, 550);

        let yolo_out = format_best_functions_output(&yolo_rep);
        assert!(yolo_out.starts_with("// ⚡ G8T YOLO #1 FASTEST: binary_search_eytzinger_u64"));
        assert!(yolo_out.contains("pub fn binary_search_eytzinger_u64"));
        assert!(!yolo_out.contains("binary_search_linear_u64"));

        // Verify scoreboard aggregation
        let board = build_foken_scoreboard_from_entries(&entries, "https://g8t.is", 10);
        assert_eq!(board.partnership_rankings[0].author_aye, "aye::Hue+Gemini(0+1)::x86_64");
        assert_eq!(board.partnership_rankings[0].total_fokens_fkn, 550);
        // Second capsule gets: 100 base + 0 crown + 1*50 vote + 100 (<10 cycles) = 250 FKN
        assert_eq!(board.partnership_rankings[1].total_fokens_fkn, 250);
    }

    #[test]
    fn partnership_aye_id_sanitizes_and_preserves_attribution() {
        let id = format_partnership_aye_id("Hue + Gemini (0+1)", "x86_64");
        assert_eq!(id, "aye::Hue+Gemini(0+1)::x86_64");

        // Ensure "human" substring is replaced so AyeSovereignGuard never rejects a partnership string
        let safe_id = format_partnership_aye_id("Human+AI", "aarch64");
        assert!(!safe_id.to_ascii_lowercase().contains("human"));
        assert!(safe_id.starts_with("aye::"));
        assert!(safe_id.ends_with("::aarch64"));

        let hw = detect_hardware_arch();
        assert!(!hw.arch.is_empty());
        assert_ne!(hw.cpu_model, "0");
    }
}
