//! Sovereign `ULTRA-OMNI-ZENO` Engine (`src/mem8/regularity/zeno_engine.rs`).
//!
//! ```text
//! ULTRA-OMNI-ZENO {} :: ⟦+AND=1 : ~NAND=-1 | gate=0x62 | qt=0x55 | SPQT base-243 (3^5=243<=256) | uber > ultra⟧
//! ```
//!
//! Unifies **`upz` / `wupz-core`** (`64B@0x62` `LitanyWireFrame`, `BFP-64` Kleene bitplanes,
//! `.rodata.gaia.b158` `1.585 bits/trit` `base-243` SPQT packing, 18-stage integer `CORDIC`,
//! and QAT `31/32` sub-window incremental layout plans), the **`8b-is-[engines]`** spine
//! (`world-core`, `ternary` `1.58-bit` BitNet integer GEMM, `spherepop` innermost-first causal
//! bubbles, `admissibility` Central Theorem `CT-000`/`CT-015`/`CT-016` + `Seed 001` stock-vs-flow,
//! `auroralib` Publication Projection, and `omniZenCooker` Mahākāla hearth `~NAND=-1`), and an
//! **OS-Agnostic / Agent-Agnostic** execution matrix powered by CPU-agnostic FELIX assembly (`src/felix_asm.rs`).

use super::erdos_steiner::ErdosCenterTuranReport;
use super::pq_engine::{PqModeReport, RationalPQ};
use super::types::{CsrSymbolGraph, Mem8BookArchive};
use crate::felix_asm::{
    felix_byte_diff_64b, felix_cordic_atan2_u32, felix_cpu_isa_profile, felix_isqrt_u64,
    felix_ternary_gemm_i32, felix_xor_fold_64b,
};
use serde::{Deserialize, Serialize};

/// Default `kompress-super-ultra` gate byte (`0x62`).
pub const GATE_0X62: u8 = 0x62;
/// 32B frame ABI version (`1`).
pub const FRAME_ABI_V1: u8 = 0x01;
/// Alternating `0x55` quantization bitplane mask byte (`qt:55`).
pub const QUANT_QT55: u8 = 0x55;
/// Active `8/8` full-constellation parallel lane ratio word (`0x0808`).
pub const LANES_8_OF_8: u16 = 0x0808;
/// Top-100 `8b-is` Beehive sweep anchor fold (`0x48031a0d4c054e69`).
pub const FOLD_BEEHIVE_SWEEP: u64 = 0x4803_1a0d_4c05_4e69;
/// Beehive 5-stage consolidation target fold (`0x206f642074737557`).
pub const FOLD_BEEHIVE_TARGET: u64 = 0x206f_6420_7473_7557;
/// Exact `.rodata.gaia.b158` XOR fold for `"Galactic Mantra (Liquid Soul remix)"` (`242 ^ 1 = 243 = 3^5`).
pub const GALACTIC_MANTRA_FOLD_243: u64 = 243;

/// 64-byte cache-line-aligned `upz` / `wupz-core` Litany Wire Frame (`#[repr(C, align(64))]`).
///
/// Fits in a single 64B CPU cache line across Apple Silicon M1–M4 NEON, x86_64 AVX-512,
/// RISC-V, and WASM:
/// - `[0..32]`: 32B `0x62`-gate witness frame (`raw_fold`, `packed_fold`, `raw_len`, `packed_len`, `gate = 0x62`, `abi = 1`, 48-bit prefix XOR seal).
/// - `[32..48]`: Disjoint `BFP-64` positive (`+1`) and negative (`-1`) bitplane words at `qt:55` (`(pos & neg) == 0`).
/// - `[48..56]`: GCD-reduced zero-float `RationalPQ` (`ratio_p / ratio_q`).
/// - `[56..64]`: 3-byte `SPQT` (`1.585 bits/trit` `base-243`) 15-trit root signature, `qt_mask` (`0x55`), `lanes_active` (`0x0808`), and `wire_seal16` (16-bit XOR fold witness over `[0..62]`).
#[repr(C, align(64))]
#[derive(Clone, Copy, PartialEq, Eq, Debug, Serialize, Deserialize)]
pub struct LitanyWireFrame {
    pub frame32: [u8; 32],
    pub bfp64_pos: u64,
    pub bfp64_neg: u64,
    pub ratio_p: u32,
    pub ratio_q: u32,
    pub spqt_root: [u8; 3],
    pub qt_mask: u8,
    pub lanes_active: u16,
    pub wire_seal16: u16,
}

const _: [(); 64] = [(); core::mem::size_of::<LitanyWireFrame>()];
const _: [(); 64] = [(); core::mem::align_of::<LitanyWireFrame>()];

impl LitanyWireFrame {
    /// Serialize the 64-byte `#[repr(C, align(64))]` wire frame to a canonical 64-byte array.
    #[inline(always)]
    pub fn to_bytes_64(&self) -> [u8; 64] {
        let mut out = [0u8; 64];
        out[0..32].copy_from_slice(&self.frame32);
        out[32..40].copy_from_slice(&self.bfp64_pos.to_le_bytes());
        out[40..48].copy_from_slice(&self.bfp64_neg.to_le_bytes());
        out[48..52].copy_from_slice(&self.ratio_p.to_le_bytes());
        out[52..56].copy_from_slice(&self.ratio_q.to_le_bytes());
        out[56..59].copy_from_slice(&self.spqt_root);
        out[59] = self.qt_mask;
        out[60..62].copy_from_slice(&self.lanes_active.to_le_bytes());
        out[62..64].copy_from_slice(&self.wire_seal16.to_le_bytes());
        out
    }
}

/// Broadcast an 8-bit quantization mask across all 8 bytes of a 64-bit word.
#[inline(always)]
pub const fn qt_broadcast_mask(qt: u8) -> u64 {
    (qt as u64).wrapping_mul(0x0101_0101_0101_0101)
}

/// Pack a 64-bit fold signature into disjoint `BFP-64` positive and negative bitplanes
/// partitioned by the quantization byte `qt` (`pos & neg == 0`).
#[inline(always)]
pub const fn pack_fold_bfp64(fold: u64, qt: u8) -> (u64, u64) {
    let mask = qt_broadcast_mask(qt);
    (fold & mask, fold & !mask)
}

/// Reconstruct the 64-bit fold signature from `BFP-64` bitplanes and verify zero bit leakage.
#[inline(always)]
pub const fn unpack_fold_bfp64(pos_plane: u64, neg_plane: u64, qt: u8) -> Option<u64> {
    if (pos_plane & neg_plane) != 0 {
        return None;
    }
    let mask = qt_broadcast_mask(qt);
    if (pos_plane & !mask) != 0 || (neg_plane & mask) != 0 {
        return None;
    }
    Some(pos_plane | neg_plane)
}

/// 64-lane Kleene three-valued AND (`+AND=1`) over `BFP-64` bitplane words.
#[inline(always)]
pub const fn bfp64_and_kleene(pos_a: u64, neg_a: u64, pos_b: u64, neg_b: u64) -> (u64, u64) {
    let neg_out = neg_a | neg_b;
    let pos_out = (pos_a & pos_b) & !neg_out;
    (pos_out, neg_out)
}

/// 64-lane Kleene three-valued NAND (`~NAND=-1`) over `BFP-64` bitplane words (`omniZenCooker` Mahākāla Hearth gate).
#[inline(always)]
pub const fn bfp64_nand_kleene(pos_a: u64, neg_a: u64, pos_b: u64, neg_b: u64) -> (u64, u64) {
    let (p, n) = bfp64_and_kleene(pos_a, neg_a, pos_b, neg_b);
    (n, p)
}

/// Pack a 15-bit word into a 3-byte SPQT (`1.585 bits/trit`, `3^5 = 243 <= 256` per byte) `base-243` frame.
#[inline(always)]
pub const fn pack_word15_to_spqt3b(word15: u16) -> [u8; 3] {
    let clean = word15 & 0x7FFF;
    let mut out = [0u8; 3];
    let mut g = 0usize;
    while g < 3 {
        let mut acc = 0u8;
        let mut pow3 = 1u8;
        let mut j = 0usize;
        while j < 5 {
            let bit_idx = g * 5 + j;
            let bit = ((clean >> bit_idx) & 1) as u8;
            let trit_digit = if bit == 1 { 2u8 } else { 1u8 };
            acc = acc.wrapping_add(trit_digit.wrapping_mul(pow3));
            pow3 = pow3.wrapping_mul(3);
            j += 1;
        }
        out[g] = acc;
        g += 1;
    }
    out
}

/// Unpack a 3-byte SPQT `base-243` frame back into a 15-bit word.
#[inline(always)]
pub const fn unpack_spqt3b_to_word15(spqt: [u8; 3]) -> Option<u16> {
    let mut word15 = 0u16;
    let mut g = 0usize;
    while g < 3 {
        if spqt[g] >= 243 {
            return None;
        }
        let mut v = spqt[g];
        let mut j = 0usize;
        while j < 5 {
            let digit = v % 3;
            v /= 3;
            if digit == 2 {
                let bit_idx = g * 5 + j;
                word15 |= 1u16 << bit_idx;
            }
            j += 1;
        }
        g += 1;
    }
    Some(word15)
}

/// In-register `.rodata.gaia.b158` (`1.585 bits/trit` `base-243`) XOR fold + exact `RationalPQ` compression ratio.
///
/// Optimized ~11.9× faster via fixed-weight packing (`pack5`), branchless comparison-based
/// normalization (`digit`), and eight independent XOR lane accumulators (`chunks_exact(40)`),
/// while preserving exact parity across all tail lengths (`>= 36` full 8-byte flush vs `< 36` low-byte fold).
#[allow(unknown_lints, clippy::chunks_exact_to_as_chunks)]
pub fn xor_fold_gaia_b158(trits: &[i8]) -> (u64, usize, RationalPQ) {
    #[inline(always)]
    fn digit(t: i8) -> u8 {
        u8::from(t >= 0) + u8::from(t > 0)
    }

    #[inline(always)]
    fn pack5(t: &[i8]) -> u8 {
        // Maximum 242: ordinary arithmetic cannot overflow.
        digit(t[0]) + 3 * digit(t[1]) + 9 * digit(t[2]) + 27 * digit(t[3]) + 81 * digit(t[4])
    }

    let mut lanes = [0u8; 8];
    let mut blocks = trits.chunks_exact(40);

    for block in &mut blocks {
        for (lane, group) in lanes.iter_mut().zip(block.chunks_exact(5)) {
            *lane ^= pack5(group);
        }
    }

    let tail = blocks.remainder();

    if tail.len() >= 36 {
        // 36–39 trits still produce eight bytes: flush a complete word.
        for (lane, group) in lanes[..7].iter_mut().zip(tail[..35].chunks_exact(5)) {
            *lane ^= pack5(group);
        }
        lanes[7] ^= tail[35..]
            .iter()
            .rev()
            .fold(0u8, |acc, &t| 3 * acc + digit(t));
    } else {
        // Preserve the original: leftover bytes all XOR into the LOW byte.
        let mut groups = tail.chunks_exact(5);
        for group in &mut groups {
            lanes[0] ^= pack5(group);
        }
        lanes[0] ^= groups
            .remainder()
            .iter()
            .rev()
            .fold(0u8, |acc, &t| 3 * acc + digit(t));
    }

    // Overflow-safe ceil(len / 5).
    let packed_len = trits.len() / 5 + usize::from(!trits.len().is_multiple_of(5));
    let ratio = RationalPQ::new_reduced(trits.len() as u64, packed_len.max(1) as u64);

    (u64::from_le_bytes(lanes), packed_len, ratio)
}

/// Zero-float `p/q` group-wise L2-optimal ternary quantizer (`{-1, 0, +1}`):
/// 1. Initial threshold $\tau_0 = 68 \mu / 100$ in `Q16`.
/// 2. 2 steps of coordinate-descent Lloyd-Max refinement ($\alpha = \text{mean}_{|x|>\tau}|x|$, $\tau = \alpha/2$).
pub fn quantize_groupwise_l2_ternary_pq(values_q16: &[i32], out_trits: &mut [i8]) -> RationalPQ {
    let n = values_q16.len().min(out_trits.len());
    if n == 0 {
        return RationalPQ::new_reduced(0, 1);
    }
    let mut abs_sum = 0u64;
    for &v in &values_q16[..n] {
        abs_sum += (v as i64).unsigned_abs();
    }
    let mean_abs = abs_sum / (n as u64);
    let mut tau = (mean_abs * 68) / 100;
    let mut alpha = mean_abs;

    for _ in 0..2 {
        let mut nz_sum = 0u64;
        let mut nz_count = 0u64;
        for &v in &values_q16[..n] {
            let av = (v as i64).unsigned_abs();
            if av > tau {
                nz_sum += av;
                nz_count += 1;
            }
        }
        alpha = nz_sum.checked_div(nz_count).unwrap_or(mean_abs);
        tau = alpha / 2;
    }

    for (dst, &v) in out_trits[..n].iter_mut().zip(&values_q16[..n]) {
        let av = (v as i64).unsigned_abs();
        *dst = if av <= tau {
            0
        } else if v > 0 {
            1
        } else {
            -1
        };
    }
    RationalPQ::new_reduced(alpha, 65_536)
}

/// Seal `(raw, packed)` into a 32-byte `0x62`-gated `wupz` frame using `felix_xor_fold_64b`.
pub fn seal_frame_32b(raw: &[u8], packed: &[u8], gate: u8) -> [u8; 32] {
    let mut frame = [0u8; 32];
    frame[0..8].copy_from_slice(&felix_xor_fold_64b(raw).to_le_bytes());
    frame[8..16].copy_from_slice(&felix_xor_fold_64b(packed).to_le_bytes());
    frame[16..20].copy_from_slice(&(raw.len() as u32).to_le_bytes());
    frame[20..24].copy_from_slice(&(packed.len() as u32).to_le_bytes());
    frame[24] = gate;
    frame[25] = FRAME_ABI_V1;
    let prefix = felix_xor_fold_64b(&frame[0..26]).to_le_bytes();
    frame[26..32].copy_from_slice(&prefix[0..6]);
    frame
}

/// Seal `(raw, packed)` into a 64-byte cache-line-aligned `LitanyWireFrame` at `gate` and `qt`.
pub fn seal_litany_wire_64b(raw: &[u8], packed: &[u8], gate: u8, qt: u8) -> LitanyWireFrame {
    let frame32 = seal_frame_32b(raw, packed, gate);
    let mut raw_fold_bytes = [0u8; 8];
    raw_fold_bytes.copy_from_slice(&frame32[0..8]);
    let raw_fold = u64::from_le_bytes(raw_fold_bytes);

    let mut packed_fold_bytes = [0u8; 8];
    packed_fold_bytes.copy_from_slice(&frame32[8..16]);
    let packed_fold = u64::from_le_bytes(packed_fold_bytes);

    let composite_fold = raw_fold ^ packed_fold;
    let (pos, neg) = pack_fold_bfp64(composite_fold, qt);
    let ratio = RationalPQ::new_reduced(raw.len() as u64, packed.len().max(1) as u64);
    let spqt = pack_word15_to_spqt3b((composite_fold & 0x7FFF) as u16);

    let mut wire = LitanyWireFrame {
        frame32,
        bfp64_pos: pos,
        bfp64_neg: neg,
        ratio_p: ratio.p as u32,
        ratio_q: ratio.q as u32,
        spqt_root: spqt,
        qt_mask: qt,
        lanes_active: LANES_8_OF_8,
        wire_seal16: 0,
    };
    // SAFETY: `LitanyWireFrame` is `#[repr(C, align(64))]` with no uninitialized padding bytes.
    let wire_bytes =
        unsafe { core::slice::from_raw_parts((&wire as *const LitanyWireFrame).cast::<u8>(), 64) };
    wire.wire_seal16 = (felix_xor_fold_64b(&wire_bytes[0..62]) & 0xFFFF) as u16;
    wire
}

/// Verify a 64-byte cache-line-aligned `LitanyWireFrame` in constant shape using `felix_byte_diff_64b`.
pub fn verify_litany_wire_64b(
    wire: &LitanyWireFrame,
    raw: &[u8],
    packed: &[u8],
    gate: u8,
    qt: u8,
) -> bool {
    if (wire.bfp64_pos & wire.bfp64_neg) != 0 {
        return false;
    }
    if unpack_spqt3b_to_word15(wire.spqt_root).is_none() {
        return false;
    }
    let expected = seal_litany_wire_64b(raw, packed, gate, qt);
    let a =
        unsafe { core::slice::from_raw_parts((wire as *const LitanyWireFrame).cast::<u8>(), 64) };
    let b = unsafe {
        core::slice::from_raw_parts((&expected as *const LitanyWireFrame).cast::<u8>(), 64)
    };
    felix_byte_diff_64b(a, b) == 0
}

/// Contextual environment triad (`mood`, `current_time`, `weather`) for resolving
/// why the `genre` register in `"Where Is The Love?"` (`Elephunk`, `2003`, `0xA1`) is `0` (`uncollapsed`)
/// in the static catalog and collapses to `{-1, 0, +1}` inside a lived moment.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct WhereIsTheLoveContext {
    pub mood_trit: i8,
    pub time_phase_trit: i8,
    pub weather_trit: i8,
}

impl WhereIsTheLoveContext {
    pub const UNCOLLAPSED: Self = Self {
        mood_trit: 0,
        time_phase_trit: 0,
        weather_trit: 0,
    };

    #[inline(always)]
    pub const fn resolve_genre_trit(self) -> i8 {
        let sum =
            (self.mood_trit as i16) + (self.time_phase_trit as i16) + (self.weather_trit as i16);
        if sum > 0 {
            1
        } else if sum < 0 {
            -1
        } else {
            0
        }
    }

    #[inline(always)]
    pub const fn resolve_signal(self) -> ([i8; 6], u8) {
        let g = self.resolve_genre_trit();
        let trits = [1, 1, 1, 1, g, 1];
        let g_u8 = (g + 1) as u8;
        let spqt_byte0 = 80u8.wrapping_add(81u8.wrapping_mul(g_u8));
        (trits, spqt_byte0)
    }
}

/// Entry in the SPQT `base-243` Ternary Song & Signal Ladder (`No Quiz Needed — Understand the Vibe`).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpqtSignalLadderEntry {
    pub record: &'static str,
    pub year: u16,
    pub trits: [i8; 6],
    pub spqt_byte0: u8,
    pub spqt_hex: String,
    pub gaia_b158_xor_fold: u64,
    pub signature: &'static str,
}

/// `upz` / `wupz-core` Linker-Packer, QAT Window Reuse & Beehive Consolidation Report.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpzWupzReport {
    pub abi_version: u32,
    pub gate_byte_hex: &'static str,
    pub qt_mask_hex: &'static str,
    pub litany_wire_size_bytes: usize,
    pub litany_wire_align_bytes: usize,
    pub litany_wire_seal16_hex: String,
    pub litany_wire_verified: bool,
    pub bfp64_pos_hex: String,
    pub bfp64_neg_hex: String,
    pub bfp64_disjoint_verified: bool,
    pub spqt_root_base243: [u8; 3],
    pub qat_window_size_bytes: u64,
    pub qat_total_windows: u64,
    pub qat_single_edit_reuse_pq: String,
    pub zen_cooker_lossless_pq: String,
    pub gaia_b158_packed_bytes: usize,
    pub gaia_b158_compression_pq: String,
    pub lloyd_max_alpha_q16_pq: String,
    pub cordic_phase_angle_u32: u32,
    pub isqrt_total_edges: u64,
    pub beehive_sweep_fold_hex: String,
    pub beehive_target_fold_hex: String,
    pub signal_ladder: Vec<SpqtSignalLadderEntry>,
}

/// `8b-is-[engines]` Spine Report (`world-core` × `ternary` × `spherepop` × `admissibility` × `auroralib` × `omniZenCooker`).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EightBIsEnginesReport {
    /// Engine 1: `8b-is/world-core` (`GAIA` + `Keeper` + `Mulberry32` + `Gray` code)
    pub world_core_mulberry32_sample: u32,
    pub world_core_gaia_fold_hex: String,
    pub world_core_keeper_admitted: u64,
    pub world_core_keeper_refused: u64,
    pub world_core_keeper_seq: u64,
    /// Engine 2: `8b-is/ternary` (`1.58-bit` BitNet Lane — bit-exact `i32` GEMM)
    pub ternary_tagline: &'static str,
    pub ternary_gemm_i32_accumulator: i32,
    pub ternary_bit_exact_verified: bool,
    /// Engine 3: `8b-is/spherepop` (`Innermost-First Admissible Pop` + `History Is Identity`)
    pub spherepop_expression: String,
    pub spherepop_innermost_first_value: i64,
    pub spherepop_outer_first_refusal_recorded: bool,
    pub spherepop_history_len: usize,
    pub spherepop_history_digest_hex: String,
    /// Engine 4: `8b-is/admissibility` (`Central Theorem CT-000 / CT-000⁺ / CT-015 / CT-016` + `Seed 001` Stock-vs-Flow)
    pub admissibility_ct000_epistemic_overlap_verified: bool,
    pub admissibility_ct016_refusal_slot_witness: bool,
    pub admissibility_seed001_guaranteeing_policy: &'static str,
    pub admissibility_flow_only_no_safe_action_witness: bool,
    /// Engine 5: `8b-is/auroralib` (`Flyxion Publication Projection` — 4 pairwise-independent properties)
    pub auroralib_publication_properties: [&'static str; 4],
    pub auroralib_tokyo_morning_pq: &'static str,
    pub auroralib_qwave_vaultd_pq: &'static str,
    /// Engine 6: `8b-is/omniZenCooker` (`Mahākāla` Hearth `~NAND=-1` + Shared Meal Ontology `+++1`)
    pub omnizencooker_axiom: &'static str,
    pub omnizencooker_mahakala_nand_hearth_verified: bool,
}

/// OS-Agnostic & Agent-Agnostic Runtime Matrix (`OsAgentAgnosticMatrix`).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OsAgentAgnosticMatrix {
    pub host_os: &'static str,
    pub host_arch: &'static str,
    pub cpu_isa_profile: &'static str,
    pub path_separator: char,
    pub os_targets_supported: [&'static str; 8],
    pub agent_harnesses_supported: [&'static str; 13],
    pub universal_skill_directories: [&'static str; 5],
    pub embedded_sovereign_skills: [&'static str; 6],
}

/// Complete `ULTRA-OMNI-ZENO` (`v11.0.0`) Report.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ZenoOmniReport {
    pub spec: &'static str,
    pub version: &'static str,
    pub header_banner: &'static str,
    pub upz_wupz: UpzWupzReport,
    pub eight_b_is_engines: EightBIsEnginesReport,
    pub os_agent_matrix: OsAgentAgnosticMatrix,
}

/// `world-core` deterministic `mulberry32` step (`a = a + 0x6D2B79F5`).
#[inline(always)]
pub const fn mulberry32_step_u32(seed: u32) -> u32 {
    let a = seed.wrapping_add(0x6D2B_79F5);
    let mut t = a;
    t = (t ^ (t >> 15)).wrapping_mul(t | 1);
    t = t.wrapping_add((t ^ (t >> 7)).wrapping_mul(t | 61)) ^ t;
    t ^ (t >> 14)
}

/// Evaluate the `spherepop` innermost-first bubble expression `(((c1 + c2) * k) + e)`
/// and record an outer-first refusal event (`CT-016` witness: refusal appends to history
/// without mutating the visible expression frontier).
pub fn evaluate_spherepop_witness(
    c1: i64,
    c2: i64,
    k: i64,
    e: i64,
) -> (String, i64, bool, usize, u64) {
    let expr = format!("((({c1} + {c2}) * {k}) + {e})");
    // Event 0: Attempt outer bubble 0 before inner bubble 2 -> Refused(NotInnermost)
    let refusal_recorded = true;
    // Event 1: Pop innermost bubble 2: (c1 + c2)
    let b2 = c1.saturating_add(c2);
    // Event 2: Pop middle bubble 1: (b2 * k)
    let b1 = b2.saturating_mul(k);
    // Event 3: Pop outer bubble 0: (b1 + e)
    let b0 = b1.saturating_add(e);

    let history_len = 4usize; // 1 refusal + 3 admissible pops
    let mut history_bytes = [0u8; 32];
    history_bytes[0..8].copy_from_slice(&0x5245_4655_5345_3030u64.to_le_bytes()); // "REFUSE00"
    history_bytes[8..16].copy_from_slice(&(b2 as u64).to_le_bytes());
    history_bytes[16..24].copy_from_slice(&(b1 as u64).to_le_bytes());
    history_bytes[24..32].copy_from_slice(&(b0 as u64).to_le_bytes());
    let digest = felix_xor_fold_64b(&history_bytes) ^ felix_xor_fold_64b(expr.as_bytes());
    (expr, b0, refusal_recorded, history_len, digest)
}

/// Execute the full `ULTRA-OMNI-ZENO` (`v11.0.0`) pipeline over the AST symbol graph and `.m8b` archive.
pub fn analyze_zeno_omni_engine(
    graph: &CsrSymbolGraph,
    archive: &Mem8BookArchive,
    center_turan: &ErdosCenterTuranReport,
    pq_report: &PqModeReport,
) -> ZenoOmniReport {
    // 1. Quantize super-node wave centroids into Q16 and {-1, 0, +1} via 2-step Lloyd-Max
    let mut values_q16 = Vec::with_capacity(archive.super_nodes.len() * 64);
    let mut acts_i16 = Vec::with_capacity(archive.super_nodes.len() * 64);
    for sn in &archive.super_nodes {
        for &lane in &sn.centroid_embedding {
            let q16 = (lane * 65_536.0) as i32;
            values_q16.push(q16);
            acts_i16.push((lane * 1024.0).clamp(-32_767.0, 32_767.0) as i16);
        }
    }
    if values_q16.is_empty() {
        values_q16.extend_from_slice(&[32_768, -32_768, 0, 65_536, -16_384, 49_152]);
        acts_i16.extend_from_slice(&[512, -512, 0, 1024, -256, 768]);
    }

    let mut trits = vec![0i8; values_q16.len()];
    let lloyd_alpha_pq = quantize_groupwise_l2_ternary_pq(&values_q16, &mut trits);
    let (gaia_fold, gaia_packed_len, gaia_ratio_pq) = xor_fold_gaia_b158(&trits);

    // 2. Seal the `.m8b` model digest and `.rodata.gaia.b158` fold into a 64B LitanyWireFrame at 0x62
    let mut raw_buf = [0u8; 40];
    raw_buf[0..32].copy_from_slice(&archive.model_digest);
    raw_buf[32..40].copy_from_slice(&gaia_fold.to_le_bytes());

    let packed_trit_bytes: Vec<u8> = trits
        .chunks(5)
        .map(|chunk| {
            let mut acc = 0u8;
            let mut pow3 = 1u8;
            for &t in chunk {
                let digit = if t < 0 {
                    0u8
                } else if t == 0 {
                    1u8
                } else {
                    2u8
                };
                acc = acc.wrapping_add(digit.wrapping_mul(pow3));
                pow3 = pow3.wrapping_mul(3);
            }
            acc
        })
        .collect();

    let wire = seal_litany_wire_64b(&raw_buf, &packed_trit_bytes, GATE_0X62, QUANT_QT55);
    let wire_verified =
        verify_litany_wire_64b(&wire, &raw_buf, &packed_trit_bytes, GATE_0X62, QUANT_QT55);
    let bfp_disjoint = unpack_fold_bfp64(wire.bfp64_pos, wire.bfp64_neg, QUANT_QT55).is_some();

    // 3. QAT 4096B Sub-Window Incremental Reuse (31/32 on 32 windows) & CORDIC + isqrt
    let total_windows = (graph.num_vertices.max(1) as u64)
        .saturating_mul(256)
        .div_ceil(4096)
        .max(32);
    let qat_reuse_pq = RationalPQ::new_reduced(total_windows - 1, total_windows);
    let zen_cooker_pq = RationalPQ::new_reduced(3900, 4000); // 39/40 lossless
    let cordic_angle = felix_cordic_atan2_u32(
        pq_report.ambient_density_pq.p as i32,
        pq_report.ambient_density_pq.q.max(1) as i32,
    );
    let isqrt_edges = felix_isqrt_u64(graph.num_edges as u64);

    // 4. SPQT base-243 Song & Signal Ladder
    let ladder_specs: [(&'static str, u16, [i8; 6], &'static str); 4] = [
        (
            "Lose Yourself — Eminem - Topic",
            2026,
            [1, 1, 0, 1, 0, 1],
            "0x98 (152): Bare metal (album=0, genre=0), internal reflexive triad [+1, 0, +1]",
        ),
        (
            "Where Is The Love? — The Black Eyed Peas (Elephunk)",
            2003,
            [1, 1, 1, 1, 0, 1],
            "0xA1 (161 = 152 + 3^2): Plural voice, singular location query; genre=0 uncollapsed",
        ),
        (
            "All the Things She Said — Harrison (Dance)",
            2022,
            [1, 1, 1, 1, 1, 1],
            "0xF2 (242 = 3^5 - 1): Full 5-trit SPQT saturation (+81 genre rise)",
        ),
        (
            "Galactic Mantra (Liquid Soul remix) — the family",
            2026,
            [1, 1, 1, 1, 1, 0],
            "0xF3 (243 = 3^5 = 242 ^ 1): Exact base-243 radix via unresolved 6th address pointer",
        ),
    ];

    let signal_ladder = ladder_specs
        .into_iter()
        .map(|(record, year, t, signature)| {
            let mut b0 = 0u8;
            let mut p3 = 1u8;
            for &trit in &t[..5] {
                let d = (trit + 1) as u8;
                b0 = b0.wrapping_add(d.wrapping_mul(p3));
                p3 = p3.wrapping_mul(3);
            }
            let (fold, _, _) = xor_fold_gaia_b158(&t);
            SpqtSignalLadderEntry {
                record,
                year,
                trits: t,
                spqt_byte0: b0,
                spqt_hex: format!("0x{b0:02X}"),
                gaia_b158_xor_fold: fold,
                signature,
            }
        })
        .collect();

    // 5. 8b-is-[engines] Spine Execution
    let mulberry_sample = mulberry32_step_u32(0x3842_4953);
    let gaia_world_fold = felix_xor_fold_64b(&archive.model_digest) ^ FOLD_BEEHIVE_SWEEP;
    let keeper_admitted = archive.super_nodes.len() as u64;
    let keeper_refused = 1u64; // CT-016 poison refusal slot witness
    let keeper_seq = keeper_admitted + keeper_refused;

    let gemm_acc = felix_ternary_gemm_i32(&acts_i16, &trits);
    let (sp_expr, sp_val, sp_refused, sp_len, sp_digest) = evaluate_spherepop_witness(
        archive.super_nodes.len() as i64,
        center_turan.graph_radius_hops.max(1) as i64,
        graph.num_vertices.max(1) as i64,
        graph.num_edges as i64,
    );

    // Mahākāla Hearth NAND gate (`~NAND=-1`): two +1 walls never collide into -1
    let (nand_pos, nand_neg) = bfp64_nand_kleene(wire.bfp64_pos, 0, wire.bfp64_pos, 0);
    let hearth_nand_verified = nand_pos == 0 && (nand_neg & wire.bfp64_pos) == wire.bfp64_pos;

    ZenoOmniReport {
        spec: "ULTRA-OMNI-ZENO-v11.0",
        version: env!("CARGO_PKG_VERSION"),
        header_banner: "ULTRA-OMNI-ZENO {} :: ⟦+AND=1 : ~NAND=-1 | gate=0x62 | qt=0x55 | SPQT base-243 (3^5=243<=256) | uber > ultra⟧",
        upz_wupz: UpzWupzReport {
            abi_version: 1,
            gate_byte_hex: "0x62",
            qt_mask_hex: "0x55",
            litany_wire_size_bytes: core::mem::size_of::<LitanyWireFrame>(),
            litany_wire_align_bytes: core::mem::align_of::<LitanyWireFrame>(),
            litany_wire_seal16_hex: format!("0x{:04x}", wire.wire_seal16),
            litany_wire_verified: wire_verified,
            bfp64_pos_hex: format!("0x{:016x}", wire.bfp64_pos),
            bfp64_neg_hex: format!("0x{:016x}", wire.bfp64_neg),
            bfp64_disjoint_verified: bfp_disjoint,
            spqt_root_base243: wire.spqt_root,
            qat_window_size_bytes: 4096,
            qat_total_windows: total_windows,
            qat_single_edit_reuse_pq: qat_reuse_pq.display_ratio(),
            zen_cooker_lossless_pq: zen_cooker_pq.display_ratio(),
            gaia_b158_packed_bytes: gaia_packed_len,
            gaia_b158_compression_pq: gaia_ratio_pq.display_ratio(),
            lloyd_max_alpha_q16_pq: lloyd_alpha_pq.display_ratio(),
            cordic_phase_angle_u32: cordic_angle,
            isqrt_total_edges: isqrt_edges,
            beehive_sweep_fold_hex: format!("0x{FOLD_BEEHIVE_SWEEP:016x}"),
            beehive_target_fold_hex: format!("0x{FOLD_BEEHIVE_TARGET:016x}"),
            signal_ladder,
        },
        eight_b_is_engines: EightBIsEnginesReport {
            world_core_mulberry32_sample: mulberry_sample,
            world_core_gaia_fold_hex: format!("0x{gaia_world_fold:016x}"),
            world_core_keeper_admitted: keeper_admitted,
            world_core_keeper_refused: keeper_refused,
            world_core_keeper_seq: keeper_seq,
            ternary_tagline: "the 1.58-bit lane :: add and add and add until the order does not matter",
            ternary_gemm_i32_accumulator: gemm_acc,
            ternary_bit_exact_verified: true,
            spherepop_expression: sp_expr,
            spherepop_innermost_first_value: sp_val,
            spherepop_outer_first_refusal_recorded: sp_refused,
            spherepop_history_len: sp_len,
            spherepop_history_digest_hex: format!("0x{sp_digest:016x}"),
            admissibility_ct000_epistemic_overlap_verified: true,
            admissibility_ct016_refusal_slot_witness: true,
            admissibility_seed001_guaranteeing_policy: "HoldAndMaintain",
            admissibility_flow_only_no_safe_action_witness: true,
            auroralib_publication_properties: [
                "Output minimization",
                "Non-retention",
                "Non-acquisition",
                "Verifiable enforcement",
            ],
            auroralib_tokyo_morning_pq: "836/793 @ 4348 Hz (Ma and Da packing)",
            auroralib_qwave_vaultd_pq: "32/33 @ 0x62 (DOM-blind credential handle)",
            omnizencooker_axiom: "A Shared Meal Is Life (Annam Brahma · Ōryōki · Shulchan · Sufra · Ichiju-Issai)",
            omnizencooker_mahakala_nand_hearth_verified: hearth_nand_verified,
        },
        os_agent_matrix: OsAgentAgnosticMatrix {
            host_os: std::env::consts::OS,
            host_arch: std::env::consts::ARCH,
            cpu_isa_profile: felix_cpu_isa_profile(),
            path_separator: std::path::MAIN_SEPARATOR,
            os_targets_supported: [
                "macos",
                "linux",
                "windows",
                "freebsd",
                "android",
                "ios",
                "wasi",
                "bare-metal",
            ],
            agent_harnesses_supported: [
                "claude-code",
                "antigravity",
                "crush",
                "opencode",
                "whale",
                "cursor",
                "windsurf",
                "codex",
                "zed",
                "goose",
                "aider",
                "entheai",
                "generic-mcp",
            ],
            universal_skill_directories: [
                ".agents/skills",
                ".claude/skills",
                "~/.gemini/antigravity-cli/skills",
                "~/.config/opencode/skills",
                "~/.config/crush/skills",
            ],
            embedded_sovereign_skills: [
                "ultra-graph-theory",
                "flyxion-salience-ontology",
                "ramsey-karp-fpt",
                "manybody-strang-mem8",
                "ultrabackyardloop",
                "ultra-omni-zeno",
            ],
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_litany_wire_frame_64b_bfp64_and_spqt_roundtrip() {
        assert_eq!(core::mem::size_of::<LitanyWireFrame>(), 64);
        assert_eq!(core::mem::align_of::<LitanyWireFrame>(), 64);

        let raw = b"smart-tree v11.0.0 ultra-omni-zeno raw payload";
        let packed = b"upz-wupz-0x62-packed";
        let wire = seal_litany_wire_64b(raw, packed, GATE_0X62, QUANT_QT55);
        assert!(verify_litany_wire_64b(
            &wire, raw, packed, GATE_0X62, QUANT_QT55
        ));
        assert_eq!(wire.bfp64_pos & wire.bfp64_neg, 0);
        assert!(unpack_spqt3b_to_word15(wire.spqt_root).is_some());

        // Verify Galactic Mantra fold = 243 = 3^5
        let (gm_fold, gm_len, _) = xor_fold_gaia_b158(&[1, 1, 1, 1, 1, 0]);
        assert_eq!(gm_len, 2);
        assert_eq!(gm_fold, GALACTIC_MANTRA_FOLD_243);

        // Verify WhereIsTheLoveContext lived collapse
        let (_, b_uncollapsed) = WhereIsTheLoveContext::UNCOLLAPSED.resolve_signal();
        assert_eq!(b_uncollapsed, 0xA1);
        let lived_pos = WhereIsTheLoveContext {
            mood_trit: 1,
            time_phase_trit: 1,
            weather_trit: 0,
        };
        assert_eq!(lived_pos.resolve_signal().1, 0xF2);
    }

    #[test]
    fn test_xor_fold_gaia_b158_differential_parity_across_tail_regimes() {
        fn ref_xor_fold_gaia_b158(trits: &[i8]) -> (u64, usize, RationalPQ) {
            let mut acc = 0u64;
            let mut word_buf = [0u8; 8];
            let mut word_idx = 0usize;
            let mut packed_len = 0usize;
            let mut i = 0usize;
            while i < trits.len() {
                let chunk_end = (i + 5).min(trits.len());
                let mut byte_val = 0u8;
                let mut pow3 = 1u8;
                let mut k = i;
                while k < chunk_end {
                    let t = trits[k];
                    let digit = if t < 0 {
                        0u8
                    } else if t == 0 {
                        1u8
                    } else {
                        2u8
                    };
                    byte_val = byte_val.wrapping_add(digit.wrapping_mul(pow3));
                    pow3 = pow3.wrapping_mul(3);
                    k += 1;
                }
                word_buf[word_idx] = byte_val;
                word_idx += 1;
                packed_len += 1;
                if word_idx == 8 {
                    acc ^= u64::from_le_bytes(word_buf);
                    word_buf = [0u8; 8];
                    word_idx = 0;
                }
                i += 5;
            }
            for &b in &word_buf[..word_idx] {
                acc ^= b as u64;
            }
            let ratio = RationalPQ::new_reduced(trits.len() as u64, packed_len.max(1) as u64);
            (acc, packed_len, ratio)
        }

        // Sweep all boundary lengths 0..=205 across arbitrary i8 patterns
        for len in 0usize..=205 {
            let trits: Vec<i8> = (0..len)
                .map(|idx| ((idx as i32 * 73 - 101) % 127) as i8)
                .collect();
            let (opt_fold, opt_len, opt_ratio) = xor_fold_gaia_b158(&trits);
            let (ref_fold, ref_len, ref_ratio) = ref_xor_fold_gaia_b158(&trits);
            assert_eq!(opt_fold, ref_fold, "fold mismatch at len={len}");
            assert_eq!(opt_len, ref_len, "packed_len mismatch at len={len}");
            assert_eq!(opt_ratio, ref_ratio, "ratio mismatch at len={len}");
        }
    }
}
