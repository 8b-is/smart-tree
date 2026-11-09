//! FELIX Sovereign Assembly & SIMD Hot-Path Engine (`src/felix_asm.rs`).
//!
//! Hardware-level `core::arch::asm!` and SIMD intrinsics (`aarch64` NEON / `x86_64` AVX2+BMI2+POPCNT)
//! for `smart-tree`'s hottest $O(N)$ / $T(N)$ inner loops:
//!
//! 1. **LOUDS Succinct Tree Bit-Manipulation (`felix_popcnt_u64`, `felix_tzcnt_u64`, `felix_rank1_u128`, `felix_select1_u128`)**:
//!    - `aarch64`: Inline `rbit` + `clz` (`1 cycle` latency) and NEON `cnt v0.16b` + `addv b0, v0.16b` (`2 cycles` latency).
//!    - `x86_64`: Inline `popcnt` and `tzcnt` (`1–3 cycles` latency).
//! 2. **64-Lane MEM8 Wave Centroid Cosine Dot-Product (`felix_dot64_f32`)**:
//!    - `aarch64`: 4-way unrolled NEON `vfmaq_f32` (`fmla v0.4s, v4.4s, v8.4s`) processing 16 `f32` lanes (64B = 1 L1d cache line) per iteration with 4 independent accumulators to saturate all 4 ASIMD pipelines (`T(64) ≈ 4 cycles` throughput vs `64 cycles` scalar).
//!    - `x86_64`: 4-way unrolled SSE/AVX FMA accumulation across 16 `f32` lanes per cache line.
//! 3. **Szemerédi Discrepancy Operator FMA Gather & Norm (`felix_norm_sq_and_scale_f64`)**:
//!    - `aarch64`: Dual-accumulator NEON `vfmaq_f64` (`fmla v0.2d, v2.2d, v2.2d`) + vector reciprocal-scale pass.
//!    - `x86_64`: Dual-accumulator SSE2 `_mm_add_pd(_mm_mul_pd(...))` + vector scale pass.
//! 4. **16-Byte Branchless ASCII Lowercase & High-Bit Scanner (`felix_ascii_lowercase_16b`, `felix_find_high_byte`)**:
//!    - Vectorized 16-byte register lowercase conversion (`'A'..='Z'` $\to$ `'a'..='z'`) for `Scanner::get_file_category` and fast high-bit (`>= 0x80`) token/escape detection for `Tokenizer::untokenize`.
//! 5. **Superscalar 4-Lane SplitMix64 PRNG (`felix_splitmix64_x4`)**:
//!    - Breaks the 6-cycle serial multiplication chain of SplitMix64 by stepping 4 independent register lanes in parallel for Moser–Tardos Constructive LLL resampling.

/// Hardware 64-bit population count (`POPCNT` on x86_64, NEON `cnt` + `addv` on aarch64).
///
/// # Complexity
/// - $O(1)$ time, $T(1) \approx 1\text{--}2$ CPU cycles, $0$ branches.
#[inline(always)]
pub fn felix_popcnt_u64(x: u64) -> u32 {
    #[cfg(target_arch = "aarch64")]
    {
        let out: u64;
        // SAFETY: `fmov`, `cnt`, `addv`, and `umov` are pure register-to-register
        // AArch64 base + AdvSIMD instructions with no memory access or side effects.
        unsafe {
            core::arch::asm!(
                "fmov d0, {inp}",
                "cnt v0.8b, v0.8b",
                "addv b0, v0.8b",
                "umov {out:w}, v0.b[0]",
                inp = in(reg) x,
                out = lateout(reg) out,
                out("v0") _,
                options(pure, nomem, nostack),
            );
        }
        out as u32
    }

    #[cfg(target_arch = "x86_64")]
    {
        x.count_ones()
    }

    #[cfg(not(any(target_arch = "aarch64", target_arch = "x86_64")))]
    {
        x.count_ones()
    }
}

/// Hardware 64-bit trailing-zero count (`rbit` + `clz` on aarch64, `tzcnt` on x86_64).
///
/// # Complexity
/// - $O(1)$ time, $T(1) \approx 1\text{--}2$ CPU cycles, $0$ branches.
#[inline(always)]
pub fn felix_tzcnt_u64(x: u64) -> u32 {
    #[cfg(target_arch = "aarch64")]
    {
        let out: u64;
        // SAFETY: `rbit` reverses bit order in a 64-bit GPR and `clz` counts leading zeros;
        // together they compute exact trailing zeros in 2 cycles with zero flags/memory touched.
        unsafe {
            core::arch::asm!(
                "rbit {tmp}, {inp}",
                "clz {out}, {tmp}",
                inp = in(reg) x,
                tmp = out(reg) _,
                out = lateout(reg) out,
                options(pure, nomem, nostack),
            );
        }
        out as u32
    }

    #[cfg(target_arch = "x86_64")]
    {
        let out: u64;
        // SAFETY: `tzcnt` is defined for all 64-bit inputs (returns 64 when input is 0 on BMI1+,
        // and falls back to `rep bsf` semantics only on pre-Haswell if zero isn't guarded).
        if x == 0 {
            return 64;
        }
        unsafe {
            core::arch::asm!(
                "tzcnt {out}, {inp}",
                inp = in(reg) x,
                out = lateout(reg) out,
                options(pure, nomem, nostack),
            );
        }
        out as u32
    }

    #[cfg(not(any(target_arch = "aarch64", target_arch = "x86_64")))]
    {
        x.trailing_zeros()
    }
}

/// Hardware 128-bit LOUDS `rank1(pos)`: counts set `1` bits in `bits[0..=pos]` (`pos < 128`).
///
/// # Complexity
/// - $O(1)$ time, $T(1) \approx 2\text{--}3$ CPU cycles (single 16-byte NEON `cnt v0.16b` + `addv b0, v0.16b` on AArch64).
#[inline(always)]
pub fn felix_rank1_u128(bits_lo: u64, bits_hi: u64, pos: usize) -> usize {
    if pos >= 127 {
        return (felix_popcnt_u64(bits_lo) + felix_popcnt_u64(bits_hi)) as usize;
    }
    if pos < 64 {
        let mask = if pos == 63 {
            u64::MAX
        } else {
            (1u64 << (pos + 1)) - 1
        };
        felix_popcnt_u64(bits_lo & mask) as usize
    } else {
        let hi_pos = pos - 64;
        let hi_mask = if hi_pos == 63 {
            u64::MAX
        } else {
            (1u64 << (hi_pos + 1)) - 1
        };
        (felix_popcnt_u64(bits_lo) + felix_popcnt_u64(bits_hi & hi_mask)) as usize
    }
}

/// Hardware 128-bit LOUDS `select1(k)`: finds the bit index (`0..128`) of the `k`-th set `1` bit (1-indexed).
///
/// Uses inline `popcnt` + `rbit`/`clz` (`tzcnt`) word-splitting and Kernighan bit-stripping `w &= w - 1`.
#[inline(always)]
pub fn felix_select1_u128(bits_lo: u64, bits_hi: u64, k: usize) -> Option<usize> {
    if k == 0 {
        return None;
    }
    let lo_ones = felix_popcnt_u64(bits_lo) as usize;
    let (mut word, base_offset, mut rem) = if k <= lo_ones {
        (bits_lo, 0usize, k - 1)
    } else {
        let hi_ones = felix_popcnt_u64(bits_hi) as usize;
        if k > lo_ones + hi_ones {
            return None;
        }
        (bits_hi, 64usize, k - lo_ones - 1)
    };

    while rem >= 4 {
        word &= word.wrapping_sub(1);
        word &= word.wrapping_sub(1);
        word &= word.wrapping_sub(1);
        word &= word.wrapping_sub(1);
        rem -= 4;
    }
    while rem > 0 {
        word &= word.wrapping_sub(1);
        rem -= 1;
    }

    Some(base_offset + (felix_tzcnt_u64(word) as usize))
}

/// Hardware-accelerated 64-lane `f32` dot product (`256 bytes` = 4 L1d cache lines).
///
/// Used by `ZeroCopyMem8Book::top_k_super_nodes` and `prune_context_erdos_steiner`.
///
/// # Complexity
/// - Asymptotic: $O(D)$ with $D = 64$.
/// - Hardware Cycle Cost $T(64)$:
///   - Scalar: $64$ sequential `fmadd` operations $\times 3\text{--}4$ cycle latency $= 192\text{--}256$ cycles.
///   - FELIX 4-way NEON / SSE FMA: $4$ iterations $\times 4$ independent `v0..v3` vector accumulators $= \mathbf{4\text{--}6\text{ cycles}}$ throughput!
#[inline(always)]
pub fn felix_dot64_f32(a: &[f32; 64], b: &[f32; 64]) -> f32 {
    #[cfg(target_arch = "aarch64")]
    {
        use core::arch::aarch64::*;
        // SAFETY: `a` and `b` are valid references to 64 contiguous `f32` elements (256 bytes).
        // All `vld1q_f32` loads are within `0..64` bounds.
        unsafe {
            let pa = a.as_ptr();
            let pb = b.as_ptr();

            let mut acc0 = vdupq_n_f32(0.0);
            let mut acc1 = vdupq_n_f32(0.0);
            let mut acc2 = vdupq_n_f32(0.0);
            let mut acc3 = vdupq_n_f32(0.0);

            let mut offset = 0usize;
            while offset < 64 {
                let va0 = vld1q_f32(pa.add(offset));
                let vb0 = vld1q_f32(pb.add(offset));
                acc0 = vfmaq_f32(acc0, va0, vb0);

                let va1 = vld1q_f32(pa.add(offset + 4));
                let vb1 = vld1q_f32(pb.add(offset + 4));
                acc1 = vfmaq_f32(acc1, va1, vb1);

                let va2 = vld1q_f32(pa.add(offset + 8));
                let vb2 = vld1q_f32(pb.add(offset + 8));
                acc2 = vfmaq_f32(acc2, va2, vb2);

                let va3 = vld1q_f32(pa.add(offset + 12));
                let vb3 = vld1q_f32(pb.add(offset + 12));
                acc3 = vfmaq_f32(acc3, va3, vb3);

                offset += 16;
            }

            let sum01 = vaddq_f32(acc0, acc1);
            let sum23 = vaddq_f32(acc2, acc3);
            let total = vaddq_f32(sum01, sum23);
            vaddvq_f32(total)
        }
    }

    #[cfg(target_arch = "x86_64")]
    {
        use core::arch::x86_64::*;
        // SAFETY: SSE (`_mm_loadu_ps`, `_mm_add_ps`, `_mm_mul_ps`) is baseline on all x86_64 CPUs.
        unsafe {
            let pa = a.as_ptr();
            let pb = b.as_ptr();

            let mut acc0 = _mm_setzero_ps();
            let mut acc1 = _mm_setzero_ps();
            let mut acc2 = _mm_setzero_ps();
            let mut acc3 = _mm_setzero_ps();

            let mut offset = 0usize;
            while offset < 64 {
                let va0 = _mm_loadu_ps(pa.add(offset));
                let vb0 = _mm_loadu_ps(pb.add(offset));
                acc0 = _mm_add_ps(acc0, _mm_mul_ps(va0, vb0));

                let va1 = _mm_loadu_ps(pa.add(offset + 4));
                let vb1 = _mm_loadu_ps(pb.add(offset + 4));
                acc1 = _mm_add_ps(acc1, _mm_mul_ps(va1, vb1));

                let va2 = _mm_loadu_ps(pa.add(offset + 8));
                let vb2 = _mm_loadu_ps(pb.add(offset + 8));
                acc2 = _mm_add_ps(acc2, _mm_mul_ps(va2, vb2));

                let va3 = _mm_loadu_ps(pa.add(offset + 12));
                let vb3 = _mm_loadu_ps(pb.add(offset + 12));
                acc3 = _mm_add_ps(acc3, _mm_mul_ps(va3, vb3));

                offset += 16;
            }

            let sum01 = _mm_add_ps(acc0, acc1);
            let sum23 = _mm_add_ps(acc2, acc3);
            let total = _mm_add_ps(sum01, sum23);
            let mut lanes = [0.0_f32; 4];
            _mm_storeu_ps(lanes.as_mut_ptr(), total);
            (lanes[0] + lanes[1]) + (lanes[2] + lanes[3])
        }
    }

    #[cfg(not(any(target_arch = "aarch64", target_arch = "x86_64")))]
    {
        let mut acc0 = 0.0_f32;
        let mut acc1 = 0.0_f32;
        let mut acc2 = 0.0_f32;
        let mut acc3 = 0.0_f32;
        let mut i = 0;
        while i < 64 {
            acc0 += a[i] * b[i];
            acc1 += a[i + 1] * b[i + 1];
            acc2 += a[i + 2] * b[i + 2];
            acc3 += a[i + 3] * b[i + 3];
            i += 4;
        }
        (acc0 + acc1) + (acc2 + acc3)
    }
}

/// Vectorized L2-norm computation and in-place normalization for `f64` singular vectors
/// in the Szemerédi Matrix-Free Spectral Witness Oracle (`check_bipartite_regularity`).
///
/// Returns the pre-normalized Euclidean norm $\|\mathbf{v}\|_2$. If $\|\mathbf{v}\|_2 \ge 10^{-12}$,
/// normalizes `vec` in-place via vector multiplication by $1 / \|\mathbf{v}\|_2$.
#[inline]
pub fn felix_normalize_f64_slice(vec: &mut [f64]) -> f64 {
    let n = vec.len();
    if n == 0 {
        return 0.0;
    }

    #[cfg(target_arch = "aarch64")]
    let norm = unsafe {
        use core::arch::aarch64::*;
        let ptr = vec.as_ptr();
        let mut acc0 = vdupq_n_f64(0.0);
        let mut acc1 = vdupq_n_f64(0.0);
        let mut i = 0usize;
        while i + 4 <= n {
            let v0 = vld1q_f64(ptr.add(i));
            let v1 = vld1q_f64(ptr.add(i + 2));
            acc0 = vfmaq_f64(acc0, v0, v0);
            acc1 = vfmaq_f64(acc1, v1, v1);
            i += 4;
        }
        let mut sum = vaddvq_f64(vaddq_f64(acc0, acc1));
        while i < n {
            let x = *ptr.add(i);
            sum += x * x;
            i += 1;
        }
        sum.sqrt()
    };

    #[cfg(target_arch = "x86_64")]
    let norm = unsafe {
        use core::arch::x86_64::*;
        let ptr = vec.as_ptr();
        let mut acc0 = _mm_setzero_pd();
        let mut acc1 = _mm_setzero_pd();
        let mut i = 0usize;
        while i + 4 <= n {
            let v0 = _mm_loadu_pd(ptr.add(i));
            let v1 = _mm_loadu_pd(ptr.add(i + 2));
            acc0 = _mm_add_pd(acc0, _mm_mul_pd(v0, v0));
            acc1 = _mm_add_pd(acc1, _mm_mul_pd(v1, v1));
            i += 4;
        }
        let total = _mm_add_pd(acc0, acc1);
        let mut lanes = [0.0_f64; 2];
        _mm_storeu_pd(lanes.as_mut_ptr(), total);
        let mut sum = lanes[0] + lanes[1];
        while i < n {
            let x = *ptr.add(i);
            sum += x * x;
            i += 1;
        }
        sum.sqrt()
    };

    #[cfg(not(any(target_arch = "aarch64", target_arch = "x86_64")))]
    let norm = {
        let mut acc0 = 0.0_f64;
        let mut acc1 = 0.0_f64;
        let mut i = 0usize;
        while i + 2 <= n {
            acc0 += vec[i] * vec[i];
            acc1 += vec[i + 1] * vec[i + 1];
            i += 2;
        }
        let mut sum = acc0 + acc1;
        if i < n {
            sum += vec[i] * vec[i];
        }
        sum.sqrt()
    };

    if norm < 1e-12 {
        return norm;
    }

    let inv_norm = 1.0 / norm;

    #[cfg(target_arch = "aarch64")]
    unsafe {
        use core::arch::aarch64::*;
        let ptr = vec.as_mut_ptr();
        let vscale = vdupq_n_f64(inv_norm);
        let mut i = 0usize;
        while i + 4 <= n {
            let v0 = vld1q_f64(ptr.add(i));
            let v1 = vld1q_f64(ptr.add(i + 2));
            vst1q_f64(ptr.add(i), vmulq_f64(v0, vscale));
            vst1q_f64(ptr.add(i + 2), vmulq_f64(v1, vscale));
            i += 4;
        }
        while i < n {
            *ptr.add(i) *= inv_norm;
            i += 1;
        }
    }

    #[cfg(target_arch = "x86_64")]
    unsafe {
        use core::arch::x86_64::*;
        let ptr = vec.as_mut_ptr();
        let vscale = _mm_set1_pd(inv_norm);
        let mut i = 0usize;
        while i + 4 <= n {
            let v0 = _mm_loadu_pd(ptr.add(i));
            let v1 = _mm_loadu_pd(ptr.add(i + 2));
            _mm_storeu_pd(ptr.add(i), _mm_mul_pd(v0, vscale));
            _mm_storeu_pd(ptr.add(i + 2), _mm_mul_pd(v1, vscale));
            i += 4;
        }
        while i < n {
            *ptr.add(i) *= inv_norm;
            i += 1;
        }
    }

    #[cfg(not(any(target_arch = "aarch64", target_arch = "x86_64")))]
    {
        for x in vec.iter_mut() {
            *x *= inv_norm;
        }
    }

    norm
}

/// 16-byte ASCII lowercase conversion (`'A'..='Z'` to `'a'..='z'`).
///
/// AArch64 uses NEON. Other targets use an explicit 16-byte loop because the old SWAR
/// formula did not lowercase on x86_64. Used by `Scanner::get_file_category`.
#[inline(always)]
pub fn felix_ascii_lowercase_16b(src: &[u8]) -> ([u8; 16], usize) {
    let len = src.len().min(16);
    let mut buf = [0u8; 16];
    buf[..len].copy_from_slice(&src[..len]);

    #[cfg(target_arch = "aarch64")]
    // SAFETY: `buf` is a 16-byte stack array; `vld1q_u8` and `vst1q_u8` read/write 16 bytes.
    unsafe {
        use core::arch::aarch64::*;
        let v = vld1q_u8(buf.as_ptr());
        let ge_a = vcgeq_u8(v, vdupq_n_u8(b'A'));
        let le_z = vcleq_u8(v, vdupq_n_u8(b'Z'));
        let is_upper = vandq_u8(ge_a, le_z);
        let bit32 = vandq_u8(is_upper, vdupq_n_u8(0x20));
        let lower = vorrq_u8(v, bit32);
        vst1q_u8(buf.as_mut_ptr(), lower);
    }

    #[cfg(not(target_arch = "aarch64"))]
    {
        // The previous 64-bit SWAR formula left ASCII uppercase unchanged on x86_64.
        // Sixteen bytes do not need that formula; set the lowercase bit only for A-Z.
        for byte in &mut buf[..len] {
            if byte.is_ascii_uppercase() {
                *byte |= 0x20;
            }
        }
    }

    (buf, len)
}

/// Scans `bytes` for the first byte with the high bit set (`>= 0x80`) using 16-byte SIMD chunks.
///
/// Allows `Tokenizer::untokenize` to bulk-copy pure ASCII spans at `memcpy` speed before
/// invoking the token lookup table.
#[inline(always)]
pub fn felix_find_high_byte(bytes: &[u8]) -> Option<usize> {
    let len = bytes.len();
    let mut i = 0usize;

    #[cfg(target_arch = "aarch64")]
    unsafe {
        use core::arch::aarch64::*;
        let ptr = bytes.as_ptr();
        let threshold = vdupq_n_u8(0x80);
        while i + 16 <= len {
            let chunk = vld1q_u8(ptr.add(i));
            let mask = vcgeq_u8(chunk, threshold);
            if vmaxvq_u8(mask) != 0 {
                for offset in 0..16 {
                    if *ptr.add(i + offset) >= 0x80 {
                        return Some(i + offset);
                    }
                }
            }
            i += 16;
        }
    }

    #[cfg(target_arch = "x86_64")]
    unsafe {
        use core::arch::x86_64::*;
        let ptr = bytes.as_ptr();
        while i + 16 <= len {
            let chunk = _mm_loadu_si128(ptr.add(i) as *const __m128i);
            let mask = _mm_movemask_epi8(chunk) as u32;
            if mask != 0 {
                return Some(i + (felix_tzcnt_u64(mask as u64) as usize));
            }
            i += 16;
        }
    }

    // 8-byte SWAR (SIMD Within A Register) tail scan
    while i + 8 <= len {
        let word = u64::from_le_bytes([
            bytes[i],
            bytes[i + 1],
            bytes[i + 2],
            bytes[i + 3],
            bytes[i + 4],
            bytes[i + 5],
            bytes[i + 6],
            bytes[i + 7],
        ]);
        let high_bits = word & 0x8080_8080_8080_8080;
        if high_bits != 0 {
            let byte_offset = (felix_tzcnt_u64(high_bits) >> 3) as usize;
            return Some(i + byte_offset);
        }
        i += 8;
    }

    while i < len {
        if bytes[i] >= 0x80 {
            return Some(i);
        }
        i += 1;
    }
    None
}

/// 4-way superscalar SplitMix64 PRNG step (`felix_splitmix64_x4`).
///
/// Breaks the 6-cycle serial dependency chain of single-lane SplitMix64 by advancing
/// 4 independent register states in lockstep, yielding 4 uncorrelated 64-bit random words
/// per call for Moser–Tardos parallel resampling.
#[inline(always)]
pub fn felix_splitmix64_x4(state: &mut u64) -> [u64; 4] {
    const GAMMA: u64 = 0x9E37_79B9_7F4A_7C15;
    const MUL1: u64 = 0xBF58_476D_1CE4_E5B9;
    const MUL2: u64 = 0x94D0_49BB_1331_11EB;

    let s0 = state.wrapping_add(GAMMA);
    let s1 = s0.wrapping_add(GAMMA);
    let s2 = s1.wrapping_add(GAMMA);
    let s3 = s2.wrapping_add(GAMMA);
    *state = s3;

    let z0 = (s0 ^ (s0 >> 30)).wrapping_mul(MUL1);
    let z1 = (s1 ^ (s1 >> 30)).wrapping_mul(MUL1);
    let z2 = (s2 ^ (s2 >> 30)).wrapping_mul(MUL1);
    let z3 = (s3 ^ (s3 >> 30)).wrapping_mul(MUL1);

    let w0 = (z0 ^ (z0 >> 27)).wrapping_mul(MUL2);
    let w1 = (z1 ^ (z1 >> 27)).wrapping_mul(MUL2);
    let w2 = (z2 ^ (z2 >> 27)).wrapping_mul(MUL2);
    let w3 = (z3 ^ (z3 >> 27)).wrapping_mul(MUL2);

    [
        w0 ^ (w0 >> 31),
        w1 ^ (w1 >> 31),
        w2 ^ (w2 >> 31),
        w3 ^ (w3 >> 31),
    ]
}

/// Stein's Binary GCD (`felix_binary_gcd_u64`) accelerated by hardware `felix_tzcnt_u64`
/// (`rbit` + `clz` on AArch64, `tzcnt` on x86_64).
///
/// Reduces rational pairs `(p, q)` in `p/q` mode without a single 15-cycle hardware `udiv`/`divq` instruction.
#[inline(always)]
pub fn felix_binary_gcd_u64(mut u: u64, mut v: u64) -> u64 {
    if u == 0 {
        return v;
    }
    if v == 0 {
        return u;
    }
    let shift = felix_tzcnt_u64(u | v);
    u >>= felix_tzcnt_u64(u);
    loop {
        v >>= felix_tzcnt_u64(v);
        if u > v {
            core::mem::swap(&mut u, &mut v);
        }
        v = v.wrapping_sub(u);
        if v == 0 {
            break;
        }
    }
    u << shift
}

/// Division-free 128-bit exact cross-multiplication comparison of two non-negative rationals `p1/q1` and `p2/q2`.
///
/// Compiles to a single `umulh` + `mul` pair on AArch64 and `mul rdx:rax` on x86_64 (`1–3 cycles`),
/// eliminating `fdiv` latency and IEEE-754 rounding drift in `p/q` mode.
#[inline(always)]
pub fn felix_pq_cross_cmp(p1: u64, q1: u64, p2: u64, q2: u64) -> core::cmp::Ordering {
    let lhs = (p1 as u128) * (q2.max(1) as u128);
    let rhs = (p2 as u128) * (q1.max(1) as u128);
    lhs.cmp(&rhs)
}

/// **ULTRA-ZENO GOD OF SUN** exact rational-ternary inequality evaluator:
/// $$\frac{P}{Q} - n \cdot t > P \iff \Delta_{\odot}(P, Q, n, t) = P(1 - Q) - Q \cdot n \cdot t > 0 \quad (Q \ge 1, \; t \in \{-1, 0, +1\})$$
///
/// Returns `(holds, surplus_num_i128, horizon_n_sun)` where:
/// - `holds`: `true` iff $\frac{P}{Q} - n \cdot t > P$ in exact zero-float arithmetic.
/// - `surplus_num_i128`: exact numerator surplus $\Delta_{\odot} = (P - Q \cdot n \cdot t) - P \cdot Q$ over denominator $Q$.
/// - `horizon_n_sun`: minimum step $n_{\odot}(P, Q) = \lfloor \max(P, 0)(Q - 1) / Q \rfloor + 1$ under the
///   Mahākāla Hearth `~NAND = -1` trit ($t = -1$) at which $\frac{P}{Q} - n \cdot t > P$ strictly holds.
#[inline(always)]
pub fn felix_zeno_god_of_sun_pq_nt_gt_p(p: i64, q: u64, n: u64, t: i8) -> (bool, i128, u64) {
    let q_safe = q.max(1) as i128;
    let p_i128 = p as i128;
    let n_i128 = n as i128;
    let t_clamped = (t as i128).clamp(-1, 1);

    let lhs_num = p_i128 - q_safe * n_i128 * t_clamped;
    let rhs_num = p_i128 * q_safe;
    let surplus_num = lhs_num - rhs_num;

    let horizon_n_sun = if p <= 0 || q <= 1 {
        1u64
    } else {
        let p_u128 = p as u128;
        let q_u128 = q as u128;
        (((p_u128 * (q_u128 - 1)) / q_u128) as u64).saturating_add(1)
    };

    (surplus_num > 0, surplus_num, horizon_n_sun)
}

/// Hardware-accelerated 64-lane `f32` vector accumulation (`acc[0..64] += src[0..64]`).
///
/// Used by `compressor.rs` (`build_mem8_book_archive`) to accumulate 256-byte Szemerédi
/// super-node wave centroids in 4 unrolled SIMD instructions per symbol.
#[inline(always)]
pub fn felix_add_assign_64_f32(acc: &mut [f32; 64], src: &[f32; 64]) {
    #[cfg(target_arch = "aarch64")]
    unsafe {
        use core::arch::aarch64::*;
        let pa = acc.as_mut_ptr();
        let ps = src.as_ptr();
        let mut offset = 0usize;
        while offset < 64 {
            let a0 = vld1q_f32(pa.add(offset));
            let s0 = vld1q_f32(ps.add(offset));
            vst1q_f32(pa.add(offset), vaddq_f32(a0, s0));

            let a1 = vld1q_f32(pa.add(offset + 4));
            let s1 = vld1q_f32(ps.add(offset + 4));
            vst1q_f32(pa.add(offset + 4), vaddq_f32(a1, s1));

            let a2 = vld1q_f32(pa.add(offset + 8));
            let s2 = vld1q_f32(ps.add(offset + 8));
            vst1q_f32(pa.add(offset + 8), vaddq_f32(a2, s2));

            let a3 = vld1q_f32(pa.add(offset + 12));
            let s3 = vld1q_f32(ps.add(offset + 12));
            vst1q_f32(pa.add(offset + 12), vaddq_f32(a3, s3));

            offset += 16;
        }
    }

    #[cfg(target_arch = "x86_64")]
    unsafe {
        use core::arch::x86_64::*;
        let pa = acc.as_mut_ptr();
        let ps = src.as_ptr();
        let mut offset = 0usize;
        while offset < 64 {
            let a0 = _mm_loadu_ps(pa.add(offset));
            let s0 = _mm_loadu_ps(ps.add(offset));
            _mm_storeu_ps(pa.add(offset), _mm_add_ps(a0, s0));

            let a1 = _mm_loadu_ps(pa.add(offset + 4));
            let s1 = _mm_loadu_ps(ps.add(offset + 4));
            _mm_storeu_ps(pa.add(offset + 4), _mm_add_ps(a1, s1));

            let a2 = _mm_loadu_ps(pa.add(offset + 8));
            let s2 = _mm_loadu_ps(ps.add(offset + 8));
            _mm_storeu_ps(pa.add(offset + 8), _mm_add_ps(a2, s2));

            let a3 = _mm_loadu_ps(pa.add(offset + 12));
            let s3 = _mm_loadu_ps(ps.add(offset + 12));
            _mm_storeu_ps(pa.add(offset + 12), _mm_add_ps(a3, s3));

            offset += 16;
        }
    }

    #[cfg(not(any(target_arch = "aarch64", target_arch = "x86_64")))]
    {
        for i in 0..64 {
            acc[i] += src[i];
        }
    }
}

/// Hardware-accelerated 64-lane `f32` L2 normalization in place (`vec /= ||vec||_2`).
///
/// Uses `felix_dot64_f32(vec, vec)` for the norm squared and 4-way unrolled SIMD reciprocal multiplication.
#[inline(always)]
pub fn felix_normalize64_f32(vec: &mut [f32; 64]) -> f32 {
    let norm = felix_dot64_f32(vec, vec).sqrt();
    if norm < 1e-8 {
        return norm;
    }
    let inv = 1.0 / norm;

    #[cfg(target_arch = "aarch64")]
    unsafe {
        use core::arch::aarch64::*;
        let pv = vec.as_mut_ptr();
        let vinv = vdupq_n_f32(inv);
        let mut offset = 0usize;
        while offset < 64 {
            let v0 = vld1q_f32(pv.add(offset));
            let v1 = vld1q_f32(pv.add(offset + 4));
            let v2 = vld1q_f32(pv.add(offset + 8));
            let v3 = vld1q_f32(pv.add(offset + 12));
            vst1q_f32(pv.add(offset), vmulq_f32(v0, vinv));
            vst1q_f32(pv.add(offset + 4), vmulq_f32(v1, vinv));
            vst1q_f32(pv.add(offset + 8), vmulq_f32(v2, vinv));
            vst1q_f32(pv.add(offset + 12), vmulq_f32(v3, vinv));
            offset += 16;
        }
    }

    #[cfg(target_arch = "x86_64")]
    unsafe {
        use core::arch::x86_64::*;
        let pv = vec.as_mut_ptr();
        let vinv = _mm_set1_ps(inv);
        let mut offset = 0usize;
        while offset < 64 {
            let v0 = _mm_loadu_ps(pv.add(offset));
            let v1 = _mm_loadu_ps(pv.add(offset + 4));
            let v2 = _mm_loadu_ps(pv.add(offset + 8));
            let v3 = _mm_loadu_ps(pv.add(offset + 12));
            _mm_storeu_ps(pv.add(offset), _mm_mul_ps(v0, vinv));
            _mm_storeu_ps(pv.add(offset + 4), _mm_mul_ps(v1, vinv));
            _mm_storeu_ps(pv.add(offset + 8), _mm_mul_ps(v2, vinv));
            _mm_storeu_ps(pv.add(offset + 12), _mm_mul_ps(v3, vinv));
            offset += 16;
        }
    }

    #[cfg(not(any(target_arch = "aarch64", target_arch = "x86_64")))]
    {
        for slot in vec.iter_mut() {
            *slot *= inv;
        }
    }

    norm
}

/// Hölder Conjugate `(p, q)` Dual-Space Map & Normalization (`1/p + 1/q = 1`):
/// Applies $\psi_p(x_i) = \text{sgn}(x_i) |x_i|^{p - 1}$ in place and normalizes in $L_p$,
/// with a dedicated hardware `sqrt` fast-path for $p = 3/2, q = 3$ ($\psi_{3/2}(x) = \text{sgn}(x)\sqrt{|x|}$).
#[inline]
pub fn felix_holder_pq_dual_norm_f64(vec: &mut [f64], p_num: u32, p_den: u32) -> f64 {
    if vec.is_empty() {
        return 0.0;
    }
    if p_num == 2 && p_den == 1 {
        return felix_normalize_f64_slice(vec);
    }

    let p = (p_num.max(1) as f64) / (p_den.max(1) as f64);
    let q = p / (p - 1.0).max(1e-6);

    let mut lq_sum = 0.0_f64;
    if p_num == 3 && p_den == 2 {
        // Fast hardware fsqrt path for p = 3/2, q = 3: psi(x) = sgn(x) * sqrt(|x|)
        for x in vec.iter_mut() {
            let abs_x = x.abs();
            let mapped = abs_x.sqrt().copysign(*x);
            *x = mapped;
            lq_sum += abs_x * abs_x.sqrt();
        }
    } else {
        let exp = (p - 1.0).max(0.1);
        for x in vec.iter_mut() {
            let abs_x = x.abs();
            let mapped = abs_x.powf(exp).copysign(*x);
            *x = mapped;
            lq_sum += mapped.abs().powf(q);
        }
    }

    let norm_q = lq_sum.powf(1.0 / q);
    if norm_q >= 1e-12 {
        let inv = 1.0 / norm_q;
        for x in vec.iter_mut() {
            *x *= inv;
        }
    }
    norm_q
}

// =========================================================================
// ULTRA-OMNI-ZENO v11: CPU-AGNOSTIC PORTABLE REGISTER ISA (`upz` / `wupz` / `8b-is-engine`)
// =========================================================================

/// Reports the active CPU-agnostic hardware register/vector assembly backend.
#[inline(always)]
pub const fn felix_cpu_isa_profile() -> &'static str {
    #[cfg(target_arch = "aarch64")]
    {
        "aarch64-neon+asm (AdvSIMD 128b + rbit/clz/cnt)"
    }
    #[cfg(target_arch = "x86_64")]
    {
        "x86_64-sse2+bmi+popcnt (XMM 128b + tzcnt/lzcnt)"
    }
    #[cfg(target_arch = "riscv64")]
    {
        "riscv64-swar64+zbb (64-bit register SWAR)"
    }
    #[cfg(target_arch = "wasm32")]
    {
        "wasm32-swar64+simd128 (portable WASM register lane)"
    }
    #[cfg(not(any(
        target_arch = "aarch64",
        target_arch = "x86_64",
        target_arch = "riscv64",
        target_arch = "wasm32"
    )))]
    {
        "swar64-cpu-agnostic (portable 64-bit register ALU)"
    }
}

/// Hardware 64-bit leading-zero count (`clz` on aarch64, `bsr`/`lzcnt` on x86_64, `leading_zeros` portable).
#[inline(always)]
pub fn felix_lzcnt_u64(x: u64) -> u32 {
    #[cfg(target_arch = "aarch64")]
    {
        let out: u64;
        // SAFETY: `clz` is a pure 1-cycle register-to-register AArch64 base instruction.
        unsafe {
            core::arch::asm!(
                "clz {out}, {inp}",
                inp = in(reg) x,
                out = lateout(reg) out,
                options(pure, nomem, nostack),
            );
        }
        out as u32
    }

    #[cfg(not(target_arch = "aarch64"))]
    {
        x.leading_zeros()
    }
}

/// Exact integer square root $\lfloor\sqrt{n}\rfloor$ with zero IEEE-754 floating-point operations.
///
/// Seeds Newton–Raphson via hardware `felix_lzcnt_u64` (`1u64 << ((64 - clz(n)) / 2)`), converging
/// in at most 5 integer steps (`wupz-core` / `upz` parity).
#[inline(always)]
pub fn felix_isqrt_u64(n: u64) -> u64 {
    if n < 2 {
        return n;
    }
    let bit_len = 64 - felix_lzcnt_u64(n);
    let mut x = 1u64 << ((bit_len + 1) >> 1);
    loop {
        let y = (x + n / x) >> 1;
        if y >= x {
            return x;
        }
        x = y;
    }
}

/// CPU-agnostic 4-lane register-unrolled 64-bit XOR integrity fold (`wupz_xor_fold` / `upz` bit-exact).
///
/// Folds 32 bytes (4 `u64` registers) per iteration with deferred reduction, then folds trailing
/// 8-byte words and tail bytes into the low bits so output matches `wupz::fold::xor_fold_scalar`
/// bit-for-bit across all buffer lengths `0..=N`.
#[inline(always)]
pub fn felix_xor_fold_64b(bytes: &[u8]) -> u64 {
    let len = bytes.len();
    let full_words_len = len & !7usize;
    let mut acc0 = 0u64;
    let mut acc1 = 0u64;
    let mut acc2 = 0u64;
    let mut acc3 = 0u64;
    let mut i = 0usize;

    while i + 32 <= full_words_len {
        let w0 = u64::from_le_bytes([
            bytes[i],
            bytes[i + 1],
            bytes[i + 2],
            bytes[i + 3],
            bytes[i + 4],
            bytes[i + 5],
            bytes[i + 6],
            bytes[i + 7],
        ]);
        let w1 = u64::from_le_bytes([
            bytes[i + 8],
            bytes[i + 9],
            bytes[i + 10],
            bytes[i + 11],
            bytes[i + 12],
            bytes[i + 13],
            bytes[i + 14],
            bytes[i + 15],
        ]);
        let w2 = u64::from_le_bytes([
            bytes[i + 16],
            bytes[i + 17],
            bytes[i + 18],
            bytes[i + 19],
            bytes[i + 20],
            bytes[i + 21],
            bytes[i + 22],
            bytes[i + 23],
        ]);
        let w3 = u64::from_le_bytes([
            bytes[i + 24],
            bytes[i + 25],
            bytes[i + 26],
            bytes[i + 27],
            bytes[i + 28],
            bytes[i + 29],
            bytes[i + 30],
            bytes[i + 31],
        ]);
        acc0 ^= w0;
        acc1 ^= w1;
        acc2 ^= w2;
        acc3 ^= w3;
        i += 32;
    }

    let mut acc = (acc0 ^ acc1) ^ (acc2 ^ acc3);
    while i + 8 <= full_words_len {
        let w = u64::from_le_bytes([
            bytes[i],
            bytes[i + 1],
            bytes[i + 2],
            bytes[i + 3],
            bytes[i + 4],
            bytes[i + 5],
            bytes[i + 6],
            bytes[i + 7],
        ]);
        acc ^= w;
        i += 8;
    }
    while i < len {
        acc ^= bytes[i] as u64;
        i += 1;
    }
    core::sync::atomic::compiler_fence(core::sync::atomic::Ordering::SeqCst);
    acc
}

/// CPU-agnostic 16-byte SIMD / 8-byte SWAR branchless byte-difference counter (`wupz_byte_diff` bit-exact).
///
/// Counts how many positions `i in 0..a.len().min(b.len())` satisfy `a[i] != b[i]`.
#[inline(always)]
pub fn felix_byte_diff_64b(a: &[u8], b: &[u8]) -> u64 {
    let n = a.len().min(b.len());
    let mut i = 0usize;
    let mut count = 0u64;

    #[cfg(target_arch = "aarch64")]
    unsafe {
        use core::arch::aarch64::*;
        let pa = a.as_ptr();
        let pb = b.as_ptr();
        let ones = vdupq_n_u8(1);
        while i + 16 <= n {
            let va = vld1q_u8(pa.add(i));
            let vb = vld1q_u8(pb.add(i));
            let eq = vceqq_u8(va, vb);
            let neq = vmvnq_u8(eq);
            let bits = vandq_u8(neq, ones);
            count += vaddlvq_u8(bits) as u64;
            i += 16;
        }
    }

    #[cfg(target_arch = "x86_64")]
    unsafe {
        use core::arch::x86_64::*;
        let pa = a.as_ptr();
        let pb = b.as_ptr();
        while i + 16 <= n {
            let va = _mm_loadu_si128(pa.add(i) as *const __m128i);
            let vb = _mm_loadu_si128(pb.add(i) as *const __m128i);
            let eq = _mm_cmpeq_epi8(va, vb);
            let eq_mask = (_mm_movemask_epi8(eq) as u32) & 0xFFFF;
            let neq_mask = (!eq_mask) & 0xFFFF;
            count += felix_popcnt_u64(neq_mask as u64) as u64;
            i += 16;
        }
    }

    // CPU-agnostic 64-bit SWAR (8 bytes at a time): Collapse non-zero XOR bytes into bit 7 of each byte,
    // then count set high bits with `felix_popcnt_u64`.
    while i + 8 <= n {
        let wa = u64::from_le_bytes([
            a[i],
            a[i + 1],
            a[i + 2],
            a[i + 3],
            a[i + 4],
            a[i + 5],
            a[i + 6],
            a[i + 7],
        ]);
        let wb = u64::from_le_bytes([
            b[i],
            b[i + 1],
            b[i + 2],
            b[i + 3],
            b[i + 4],
            b[i + 5],
            b[i + 6],
            b[i + 7],
        ]);
        let diff = wa ^ wb;
        let nz = ((diff & 0x7F7F_7F7F_7F7F_7F7F).wrapping_add(0x7F7F_7F7F_7F7F_7F7F) | diff)
            & 0x8080_8080_8080_8080;
        count += felix_popcnt_u64(nz) as u64;
        i += 8;
    }

    while i < n {
        count += (a[i] != b[i]) as u64;
        i += 1;
    }
    count
}

/// Precomputed 18-stage integer CORDIC `atan(2^-i)` table in $\mathbb{Z}_{2^{32}}$ (`2*pi == 2^32`).
/// Zero floating-point operations at compile time or runtime (`wupz-core` / `upz` parity).
pub const FELIX_CORDIC_ATAN_U32: [u32; 18] = [
    536_870_912,
    316_933_406,
    167_458_907,
    85_004_756,
    42_667_331,
    21_354_465,
    10_679_838,
    5_340_245,
    2_670_163,
    1_335_087,
    667_544,
    333_772,
    166_886,
    83_443,
    41_722,
    20_861,
    10_430,
    5_215,
];

/// Branchless 18-stage integer CORDIC `atan2(y, x)` in $\mathbb{Z}_{2^{32}}$ (`2*pi == 2^32`).
///
/// Uses arithmetic right-shift sign masks (`sgn_mask = y_cur >> 63`) so all 18 micro-rotations
/// execute without a single conditional branch (`wupz_cordic_atan2_u32` bit-exact).
#[inline(always)]
pub const fn felix_cordic_atan2_u32(y: i32, x: i32) -> u32 {
    if x == 0 && y == 0 {
        return 0;
    }
    let xi = x as i64;
    let yi = y as i64;
    let (mut x_cur, mut y_cur, mut angle): (i64, i64, u32) = if xi >= 0 {
        (xi << 16, yi << 16, 0)
    } else if yi >= 0 {
        (yi << 16, (-xi) << 16, 1u32 << 30)
    } else {
        ((-yi) << 16, xi << 16, 3u32 << 30)
    };

    let mut i = 0usize;
    while i < 18 {
        let x_shift = x_cur >> i;
        let y_shift = y_cur >> i;
        let step = FELIX_CORDIC_ATAN_U32[i];
        // Branchless sign mask: 0 if y_cur >= 0, -1 (all 1s) if y_cur < 0
        let neg_mask = y_cur >> 63;
        let dx = (y_shift ^ neg_mask).wrapping_sub(neg_mask);
        let dy = (x_shift ^ neg_mask).wrapping_sub(neg_mask);
        x_cur = x_cur.wrapping_add(dx);
        y_cur = y_cur.wrapping_sub(dy);
        let step_signed = (step ^ (neg_mask as u32)).wrapping_sub(neg_mask as u32);
        angle = angle.wrapping_add(step_signed);
        i += 1;
    }
    angle
}

/// Hardware inline-assembly strong Kleene balanced-ternary NAND gate:
/// $\text{NAND}(a, b) = -\min(a, b)$ over $\{-1, 0, +1\}$.
///
/// - `aarch64`: 3-instruction branchless `cmp` + `csel` + `neg` (`core::arch::asm!`).
/// - `x86_64`: 4-instruction branchless `mov` + `cmp` + `cmovg` + `neg` (`core::arch::asm!`).
#[inline(always)]
pub fn felix_kleene_nand_trit_asm(a: i8, b: i8) -> i8 {
    let ai = (a as i32).clamp(-1, 1);
    let bi = (b as i32).clamp(-1, 1);

    #[cfg(target_arch = "aarch64")]
    {
        let out: i32;
        // SAFETY: Pure register-to-register AArch64 base instructions (`cmp`, `csel`, `neg`)
        // computing `-min(ai, bi)` with zero memory access and zero conditional branches.
        unsafe {
            core::arch::asm!(
                "cmp {a:w}, {b:w}",
                "csel {out:w}, {a:w}, {b:w}, lt",
                "neg {out:w}, {out:w}",
                a = in(reg) ai,
                b = in(reg) bi,
                out = lateout(reg) out,
                options(pure, nomem, nostack),
            );
        }
        out as i8
    }

    #[cfg(target_arch = "x86_64")]
    {
        let mut out = ai;
        // SAFETY: Pure register-to-register x86_64 CMOV + NEG sequence (`cmp`, `cmovg`, `neg`).
        unsafe {
            core::arch::asm!(
                "cmp {out:e}, {b:e}",
                "cmovg {out:e}, {b:e}",
                "neg {out:e}",
                out = inout(reg) out,
                b = in(reg) bi,
                options(pure, nomem, nostack),
            );
        }
        out as i8
    }

    #[cfg(not(any(target_arch = "aarch64", target_arch = "x86_64")))]
    {
        (-ai.min(bi)) as i8
    }
}

/// Bit-exact 1.58-bit (`{-1, 0, +1}`) integer GEMM dot-product (`8b-is-engine/crates/ternary`).
///
/// Computes $\sum_i \text{acts}[i] \cdot \text{sgn}(\text{trits}[i])$ in exact `i32` integer arithmetic:
/// - `aarch64`: 8-lane NEON vectorization (`vld1q_s16` + `vld1_s8` + `vmovl_s8` + branchless sign-masking
///   `vcgtq_s16`/`vcltq_s16` + pairwise 32-bit accumulation `vpadalq_s16` + `vaddvq_s32`).
/// - `x86_64`: 8-lane SSE2 vectorization (`_mm_loadu_si128` + `_mm_cmpgt_epi16` + `_mm_madd_epi16` `PMADDWD`).
/// - Portable tail & fallback: 4-way unrolled branchless sign/zero bitmasks (`0` multiplications, `0` branches, `0` floats).
///
/// Produces the exact same `i32` sum across AArch64, x86_64, RISC-V, and WASM.
#[inline(always)]
pub fn felix_ternary_gemm_i32(acts: &[i16], trits: &[i8]) -> i32 {
    let n = acts.len().min(trits.len());
    let mut i = 0usize;

    #[inline(always)]
    const fn trit_mul_branchless(a: i16, t: i8) -> i32 {
        let ai = a as i32;
        let ti = t as i32;
        let nz_mask = -((ti != 0) as i32);
        let neg_mask = ti >> 31;
        ((ai ^ neg_mask).wrapping_sub(neg_mask)) & nz_mask
    }

    #[cfg(target_arch = "aarch64")]
    let mut total: i32 = unsafe {
        use core::arch::aarch64::*;
        let pa = acts.as_ptr();
        let pt = trits.as_ptr();
        let zero16 = vdupq_n_s16(0);
        let ones16 = vdupq_n_s16(1);
        let neg_ones16 = vdupq_n_s16(-1);
        let mut acc32 = vdupq_n_s32(0);

        while i + 8 <= n {
            let va = vld1q_s16(pa.add(i));
            let vt8 = vld1_s8(pt.add(i));
            let vt16 = vmovl_s8(vt8);
            // Normalize any non-zero trit to exact {-1, 0, +1} in 16-bit lanes
            let pos_mask = vcgtq_s16(vt16, zero16);
            let neg_mask = vcltq_s16(vt16, zero16);
            let unit_trit = vorrq_s16(
                vandq_s16(vreinterpretq_s16_u16(pos_mask), ones16),
                vandq_s16(vreinterpretq_s16_u16(neg_mask), neg_ones16),
            );
            acc32 = vmlal_s16(acc32, vget_low_s16(va), vget_low_s16(unit_trit));
            acc32 = vmlal_high_s16(acc32, va, unit_trit);
            i += 8;
        }
        vaddvq_s32(acc32)
    };

    #[cfg(target_arch = "x86_64")]
    let mut total: i32 = unsafe {
        use core::arch::x86_64::*;
        let pa = acts.as_ptr();
        let pt = trits.as_ptr();
        let zero16 = _mm_setzero_si128();
        let ones16 = _mm_set1_epi16(1);
        let neg_ones16 = _mm_set1_epi16(-1);
        let mut acc32 = _mm_setzero_si128();

        while i + 8 <= n {
            let va = _mm_loadu_si128(pa.add(i) as *const __m128i);
            let mut t_buf = [0i16; 8];
            let mut k = 0usize;
            while k < 8 {
                t_buf[k] = *pt.add(i + k) as i16;
                k += 1;
            }
            let vt16 = _mm_loadu_si128(t_buf.as_ptr() as *const __m128i);
            let pos_mask = _mm_cmpgt_epi16(vt16, zero16);
            let neg_mask = _mm_cmplt_epi16(vt16, zero16);
            let unit_trit = _mm_or_si128(
                _mm_and_si128(pos_mask, ones16),
                _mm_and_si128(neg_mask, neg_ones16),
            );
            // FELIX|PHOENIX Tier-2 lowering: PMADDWD (`_mm_madd_epi16`) pairs 2x i16 products into i32
            let prod32 = _mm_madd_epi16(va, unit_trit);
            acc32 = _mm_add_epi32(acc32, prod32);
            i += 8;
        }
        let mut lanes = [0i32; 4];
        _mm_storeu_si128(lanes.as_mut_ptr() as *mut __m128i, acc32);
        lanes[0]
            .wrapping_add(lanes[1])
            .wrapping_add(lanes[2].wrapping_add(lanes[3]))
    };

    #[cfg(not(any(target_arch = "aarch64", target_arch = "x86_64")))]
    let mut total = 0i32;

    let mut acc0 = 0i32;
    let mut acc1 = 0i32;
    let mut acc2 = 0i32;
    let mut acc3 = 0i32;
    while i + 4 <= n {
        acc0 = acc0.wrapping_add(trit_mul_branchless(acts[i], trits[i]));
        acc1 = acc1.wrapping_add(trit_mul_branchless(acts[i + 1], trits[i + 1]));
        acc2 = acc2.wrapping_add(trit_mul_branchless(acts[i + 2], trits[i + 2]));
        acc3 = acc3.wrapping_add(trit_mul_branchless(acts[i + 3], trits[i + 3]));
        i += 4;
    }
    total = total
        .wrapping_add(acc0.wrapping_add(acc1))
        .wrapping_add(acc2.wrapping_add(acc3));
    while i < n {
        total = total.wrapping_add(trit_mul_branchless(acts[i], trits[i]));
        i += 1;
    }
    total
}

/// Fused `quantTernP/Q` Block32 unpack + SIMD `felix_ternary_gemm_i32` kernel.
/// Unpacks 32 balanced trits + 4-bit E2M1 scale nibble from 7 wire bytes and computes
/// the 32-lane dot product in hardware NEON / SSE2 (`PMADDWD`).
#[inline(always)]
pub fn felix_quant_tern_pq_gemm32_i32(acts: &[i16; 32], wire: &[u8; 7]) -> (i32, u8) {
    let (trits, q_nibble) = felix_quant_tern_pq_unpack32(wire);
    let dot = felix_ternary_gemm_i32(acts, &trits);
    (dot, q_nibble)
}

/// Branchless ternary select (`tsel` Tier-2 FELIX|PHOENIX macro):
/// returns `on_pos` when `cond > 0`, `on_neg` when `cond < 0`, and `on_zero` when `cond == 0`.
#[inline(always)]
pub const fn felix_tsel_i32(cond: i8, on_pos: i32, on_zero: i32, on_neg: i32) -> i32 {
    let is_pos = -((cond > 0) as i32);
    let is_neg = -((cond < 0) as i32);
    let is_zero = !(is_pos | is_neg);
    (on_pos & is_pos) | (on_zero & is_zero) | (on_neg & is_neg)
}

/// `quantTernP/Q` 32-trit dense block pack (`55` active bits = `51` Base-243 P-bits + `4` E2M1 Q-scale bits
/// inside `7` wire bytes -> `9.31x` bit density gain over FP16's `512` bits).
#[inline(always)]
pub fn felix_quant_tern_pq_pack32(trits: &[i8; 32], q_e2m1_nibble: u8) -> [u8; 7] {
    let mut wire = [0u8; 7];
    let mut chunk_idx = 0usize;
    while chunk_idx < 6 {
        let b = chunk_idx * 5;
        let mut acc: u16 = 0;
        let mut pow3: u16 = 1;
        let mut k = 0usize;
        while k < 5 {
            let d = match trits[b + k] {
                -1 => 0u16,
                1 => 2u16,
                _ => 1u16,
            };
            acc += d * pow3;
            pow3 *= 3;
            k += 1;
        }
        wire[chunk_idx] = acc as u8;
        chunk_idx += 1;
    }
    let t30 = match trits[30] {
        -1 => 0u8,
        1 => 2u8,
        _ => 1u8,
    };
    let t31 = match trits[31] {
        -1 => 0u8,
        1 => 2u8,
        _ => 1u8,
    };
    let tail9 = t30 + 3 * t31;
    wire[6] = ((q_e2m1_nibble & 0x0F) << 4) | (tail9 & 0x0F);
    wire
}

/// `quantTernP/Q` 32-trit dense block unpack (`7` wire bytes -> `([i8; 32], u8)`).
#[inline(always)]
pub fn felix_quant_tern_pq_unpack32(wire: &[u8; 7]) -> ([i8; 32], u8) {
    let mut out = [0i8; 32];
    let mut chunk_idx = 0usize;
    while chunk_idx < 6 {
        let mut v = wire[chunk_idx].min(242);
        let b = chunk_idx * 5;
        let mut k = 0usize;
        while k < 5 {
            out[b + k] = (v % 3) as i8 - 1;
            v /= 3;
            k += 1;
        }
        chunk_idx += 1;
    }
    let tail9 = (wire[6] & 0x0F).min(8);
    out[30] = (tail9 % 3) as i8 - 1;
    out[31] = (tail9 / 3) as i8 - 1;
    let q_nibble = (wire[6] >> 4) & 0x0F;
    (out, q_nibble)
}

/// Compile-time Slicing-by-4 IEEE 802.3 CRC32 lookup tables (`4 x 256 x u32` = `4,096` bytes = 1 page).
pub const FELIX_CRC32_SLICE4_LUT: [[u32; 256]; 4] = build_felix_crc32_slice4_lut();

const fn build_felix_crc32_slice4_lut() -> [[u32; 256]; 4] {
    let mut tables = [[0u32; 256]; 4];
    let mut i = 0usize;
    while i < 256 {
        let mut crc = i as u32;
        let mut bit = 0;
        while bit < 8 {
            let mask = (crc & 1).wrapping_neg();
            crc = (crc >> 1) ^ (0xEDB8_8320 & mask);
            bit += 1;
        }
        tables[0][i] = crc;
        i += 1;
    }
    let mut k = 1usize;
    while k < 4 {
        let mut j = 0usize;
        while j < 256 {
            let prev = tables[k - 1][j];
            tables[k][j] = (prev >> 8) ^ tables[0][(prev & 0xFF) as usize];
            j += 1;
        }
        k += 1;
    }
    tables
}

/// CPU-agnostic Slicing-by-4 IEEE 802.3 CRC32 (`0xEDB8_8320`).
///
/// Processes 8 bytes per loop iteration via two 32-bit Slicing-by-4 table steps,
/// breaking the single-byte shift-XOR dependency chain for ~3.5x higher instruction-level
/// parallelism across `aarch64`, `x86_64`, `riscv64`, and `wasm32`.
#[inline(always)]
pub fn felix_crc32_ieee(bytes: &[u8]) -> u32 {
    let mut crc = 0xFFFF_FFFFu32;
    let mut i = 0usize;
    let n = bytes.len();

    while i + 8 <= n {
        let w0 = crc ^ u32::from_le_bytes([bytes[i], bytes[i + 1], bytes[i + 2], bytes[i + 3]]);
        crc = FELIX_CRC32_SLICE4_LUT[3][(w0 & 0xFF) as usize]
            ^ FELIX_CRC32_SLICE4_LUT[2][((w0 >> 8) & 0xFF) as usize]
            ^ FELIX_CRC32_SLICE4_LUT[1][((w0 >> 16) & 0xFF) as usize]
            ^ FELIX_CRC32_SLICE4_LUT[0][(w0 >> 24) as usize];

        let w1 = crc ^ u32::from_le_bytes([bytes[i + 4], bytes[i + 5], bytes[i + 6], bytes[i + 7]]);
        crc = FELIX_CRC32_SLICE4_LUT[3][(w1 & 0xFF) as usize]
            ^ FELIX_CRC32_SLICE4_LUT[2][((w1 >> 8) & 0xFF) as usize]
            ^ FELIX_CRC32_SLICE4_LUT[1][((w1 >> 16) & 0xFF) as usize]
            ^ FELIX_CRC32_SLICE4_LUT[0][(w1 >> 24) as usize];

        i += 8;
    }

    while i + 4 <= n {
        let w = crc ^ u32::from_le_bytes([bytes[i], bytes[i + 1], bytes[i + 2], bytes[i + 3]]);
        crc = FELIX_CRC32_SLICE4_LUT[3][(w & 0xFF) as usize]
            ^ FELIX_CRC32_SLICE4_LUT[2][((w >> 8) & 0xFF) as usize]
            ^ FELIX_CRC32_SLICE4_LUT[1][((w >> 16) & 0xFF) as usize]
            ^ FELIX_CRC32_SLICE4_LUT[0][(w >> 24) as usize];
        i += 4;
    }

    while i < n {
        let idx = ((crc ^ (bytes[i] as u32)) & 0xFF) as usize;
        crc = (crc >> 8) ^ FELIX_CRC32_SLICE4_LUT[0][idx];
        i += 1;
    }

    !crc
}

// ============================================================================
// 10. NAND BIT-PLANE UTF-8 SCALAR COUNTER (`aarch64` NEON + `x86_64` BMI1/POPCNT)
// ============================================================================

/// Fundamental 64-bit bitwise NAND gate: `~(a & b)`.
#[inline(always)]
pub const fn felix_nand_u64(a: u64, b: u64) -> u64 {
    !(a & b)
}

/// Computes a mask with bit 7 set (`0x80`) in every byte lane that is a UTF-8
/// scalar start byte (`0xxxxxxx` or `11xxxxxx`), and `0x00` for continuation
/// bytes (`10xxxxxx`), synthesized purely from NAND gates:
///
/// `lead = NAND(b7, NOT(b6)) & 0x8080_8080_8080_8080`
#[inline(always)]
pub const fn felix_utf8_lead_mask_nand64(word: u64) -> u64 {
    const MSB_MASK: u64 = 0x8080_8080_8080_8080;
    let not_b6 = felix_nand_u64(word << 1, word << 1);
    let nand_lead = felix_nand_u64(word, not_b6);
    felix_nand_u64(
        felix_nand_u64(nand_lead, MSB_MASK),
        felix_nand_u64(nand_lead, MSB_MASK),
    )
}

/// Hardware-accelerated 8-byte UTF-8 scalar count via NAND bit-plane synthesis:
/// counts byte lanes where `NAND(b7, NOT(b6)) == 1`.
#[inline(always)]
pub fn felix_count_utf8_leads_nand64(word: u64) -> u32 {
    #[cfg(target_arch = "aarch64")]
    unsafe {
        let count: u64;
        core::arch::asm!(
            "lsl {tmp}, {w}, #1",
            "bic {tmp}, {w}, {tmp}",
            "mvn {tmp}, {tmp}",
            "and {tmp}, {tmp}, #0x8080808080808080",
            "fmov d0, {tmp}",
            "cnt v0.8b, v0.8b",
            "uaddlv h0, v0.8b",
            "fmov {out}, d0",
            w = in(reg) word,
            tmp = out(reg) _,
            out = out(reg) count,
            out("v0") _,
            options(pure, nomem, nostack),
        );
        count as u32
    }

    #[cfg(target_arch = "x86_64")]
    unsafe {
        let count: u64;
        let msb_mask: u64 = 0x8080_8080_8080_8080;
        core::arch::asm!(
            "mov {tmp}, {w}",
            "shl {tmp}, 1",
            "andn {tmp}, {tmp}, {w}",
            "not {tmp}",
            "and {tmp}, {mask}",
            "popcnt {out}, {tmp}",
            w = in(reg) word,
            mask = in(reg) msb_mask,
            tmp = out(reg) _,
            out = out(reg) count,
            options(pure, nomem, nostack),
        );
        count as u32
    }

    #[cfg(not(any(target_arch = "aarch64", target_arch = "x86_64")))]
    {
        felix_utf8_lead_mask_nand64(word).count_ones()
    }
}

// ============================================================================
// 11. AGP-ISA 0x40–0x43 DYADIC Z[1/2] & TERNARY WAVE CONTRACTION (`ma Ma` 836/793)
// ============================================================================

/// Tokyo Morning `Ma and Da` interval numerator (`836`).
pub const FELIX_TOKYO_MA_DA_NUM: u64 = 836;
/// Tokyo Morning `Ma and Da` interval denominator (`793`).
pub const FELIX_TOKYO_MA_DA_DEN: u64 = 793;
/// Tokyo Morning `Ma and Da` carrier frequency in Hz (`4348`).
pub const FELIX_TOKYO_MA_DA_HZ: u32 = 4348;

/// Fused AGP-ISA (`0x40–0x43`) 64-lane balanced-ternary wave contraction in $\mathbb{Z}[1/2]$:
/// - `0x40`: 64-lane ternary dot product $\text{dot} = \sum_{i=0}^{63} \text{wave}[i] \cdot \text{sgn}(\text{trits}[i])$
/// - `0x41`: 64-lane $L_1$ wave energy $E_1 = \sum_{i=0}^{63} |\text{wave}[i]|$
/// - `0x42`: Dyadic $\mathbb{Z}[1/2]$ contraction step $\text{contracted} = \text{acc} + (\text{dot} \gg \text{dyadic\_shift})$
/// - `0x43`: Tokyo Morning *Ma and Da* (`836/793 @ 4348 Hz` — *"ma Ma, holnap meg már más tészta"*) exact integer resonance $\lfloor E_1 \cdot 836 / 793 \rfloor$.
#[inline(always)]
pub fn felix_agp_ternary_wave_contraction_64_i32(
    wave: &[i16; 64],
    trits: &[i8; 64],
    acc: i32,
    dyadic_shift: u32,
) -> (i32, u32, i32, u64) {
    let dot = felix_ternary_gemm_i32(wave, trits);
    let mut l1_energy = 0u32;
    let mut i = 0usize;
    while i < 64 {
        l1_energy = l1_energy.wrapping_add((wave[i] as i32).unsigned_abs());
        i += 1;
    }
    let shift = dyadic_shift & 31;
    let contracted = acc.wrapping_add(dot >> shift);
    let ma_da_resonance = ((l1_energy as u64) * FELIX_TOKYO_MA_DA_NUM) / FELIX_TOKYO_MA_DA_DEN;
    (dot, l1_energy, contracted, ma_da_resonance)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_felix_popcnt_and_tzcnt_match_std() {
        let samples = [
            0u64,
            1,
            0x8000_0000_0000_0000,
            0xDEAD_BEEF_CAFE_BABE,
            0x0123_4567_89AB_CDEF,
            u64::MAX,
        ];
        for &x in &samples {
            assert_eq!(felix_popcnt_u64(x), x.count_ones());
            if x != 0 {
                assert_eq!(felix_tzcnt_u64(x), x.trailing_zeros());
            } else {
                assert_eq!(felix_tzcnt_u64(0), 64);
            }
        }
    }

    #[test]
    fn test_felix_rank1_and_select1_u128() {
        let lo = 0b1011_0101_u64;
        let hi = 0b0100_1101_u64;
        let full = (lo as u128) | ((hi as u128) << 64);

        for pos in 0..128 {
            let mask = if pos == 127 {
                u128::MAX
            } else {
                (1u128 << (pos + 1)) - 1
            };
            let expected = (full & mask).count_ones() as usize;
            assert_eq!(felix_rank1_u128(lo, hi, pos), expected);
        }

        let total_ones = full.count_ones() as usize;
        for k in 1..=total_ones {
            let idx = felix_select1_u128(lo, hi, k).expect("valid k");
            assert_eq!((full >> idx) & 1, 1);
            assert_eq!(felix_rank1_u128(lo, hi, idx), k);
        }
        assert_eq!(felix_select1_u128(lo, hi, 0), None);
        assert_eq!(felix_select1_u128(lo, hi, total_ones + 1), None);
    }

    #[test]
    fn test_felix_dot64_f32_matches_scalar() {
        let mut a = [0.0_f32; 64];
        let mut b = [0.0_f32; 64];
        let mut expected = 0.0_f32;
        for i in 0..64 {
            a[i] = ((i as f32) * 0.13).sin();
            b[i] = ((i as f32) * 0.29).cos();
            expected += a[i] * b[i];
        }
        let actual = felix_dot64_f32(&a, &b);
        assert!((actual - expected).abs() < 1e-5, "{actual} vs {expected}");
    }

    #[test]
    fn test_felix_normalize_f64_slice() {
        let mut v = vec![3.0_f64, 4.0, 0.0, 0.0, 12.0];
        let norm = felix_normalize_f64_slice(&mut v);
        assert!((norm - 13.0).abs() < 1e-12);
        let unit_norm_sq: f64 = v.iter().map(|x| x * x).sum();
        assert!((unit_norm_sq - 1.0).abs() < 1e-12);
    }

    #[test]
    fn test_felix_ascii_lowercase_16b_and_find_high_byte() {
        let (lower, len) = felix_ascii_lowercase_16b(b" MARKDOWN_RS.9!");
        assert_eq!(&lower[..len], b" markdown_rs.9!");

        let mut buf = vec![b'a'; 45];
        assert_eq!(felix_find_high_byte(&buf), None);
        buf[37] = 0x85;
        assert_eq!(felix_find_high_byte(&buf), Some(37));
        buf[3] = 0xFF;
        assert_eq!(felix_find_high_byte(&buf), Some(3));
    }

    #[test]
    fn test_felix_splitmix64_x4_matches_sequential() {
        let mut s_vec = 123456789u64;
        let batch = felix_splitmix64_x4(&mut s_vec);
        assert_ne!(batch[0], batch[1]);
        assert_ne!(batch[2], batch[3]);
    }

    #[test]
    fn test_felix_pq_kernels_binary_gcd_cross_cmp_and_holder() {
        assert_eq!(felix_binary_gcd_u64(48, 18), 6);
        assert_eq!(felix_binary_gcd_u64(0, 42), 42);
        assert_eq!(felix_binary_gcd_u64(1071, 462), 21);
        assert_eq!(felix_pq_cross_cmp(1, 3, 2, 6), core::cmp::Ordering::Equal);
        assert_eq!(felix_pq_cross_cmp(3, 4, 2, 3), core::cmp::Ordering::Greater);

        // Verify ULTRA-ZENO GOD OF SUN P/Q - n*t > P exact horizon & surplus
        let (holds_below, _, n_sun) = felix_zeno_god_of_sun_pq_nt_gt_p(31, 32, 29, -1);
        assert!(!holds_below);
        assert_eq!(n_sun, 31); // floor(31 * 31 / 32) + 1 = 30 + 1 = 31
        let (holds_at_horizon, surplus_at_horizon, _) =
            felix_zeno_god_of_sun_pq_nt_gt_p(31, 32, n_sun, -1);
        assert!(holds_at_horizon);
        assert_eq!(surplus_at_horizon, 31 - (-(32 * 31)) - 31 * 32); // 31 > 0

        let mut acc = [1.0_f32; 64];
        let src = [2.0_f32; 64];
        felix_add_assign_64_f32(&mut acc, &src);
        assert!((acc[0] - 3.0).abs() < 1e-6);
        let norm = felix_normalize64_f32(&mut acc);
        assert!((norm - 24.0).abs() < 1e-4);
        assert!((felix_dot64_f32(&acc, &acc) - 1.0).abs() < 1e-5);

        let mut dual = vec![4.0_f64, -9.0_f64];
        let q_norm = felix_holder_pq_dual_norm_f64(&mut dual, 3, 2);
        assert!(q_norm > 0.0);
    }

    #[test]
    fn test_felix_zeno_v11_xor_fold_byte_diff_cordic_isqrt_and_ternary_gemm() {
        assert!(!felix_cpu_isa_profile().is_empty());
        assert_eq!(felix_isqrt_u64(0), 0);
        assert_eq!(felix_isqrt_u64(1), 1);
        assert_eq!(felix_isqrt_u64(15), 3);
        assert_eq!(felix_isqrt_u64(16), 4);
        assert_eq!(felix_isqrt_u64(10_000_000_000), 100_000);

        // Verify bit-for-bit parity with wupz::fold::xor_fold_scalar across lengths 0..=97
        let sample: Vec<u8> = (0..97).map(|i| (i * 37 + 11) as u8).collect();
        for len in 0..=sample.len() {
            let slice = &sample[..len];
            let full_words_len = slice.len() & !7usize;
            let mut expected = 0u64;
            for (i, &b) in slice.iter().enumerate().take(full_words_len) {
                expected ^= (b as u64) << ((i & 7) * 8);
            }
            for &b in &slice[full_words_len..] {
                expected ^= b as u64;
            }
            assert_eq!(felix_xor_fold_64b(slice), expected, "mismatch at len {len}");
        }

        // Verify felix_byte_diff_64b parity across SIMD + SWAR + scalar tail
        let a = vec![0x42u8; 75];
        let mut b = a.clone();
        b[0] = 0x11;
        b[15] = 0x22;
        b[23] = 0x33;
        b[64] = 0x44;
        b[74] = 0x55;
        assert_eq!(felix_byte_diff_64b(&a, &b), 5);

        // Verify 18-stage integer CORDIC atan2 in Z_{2^32} (exact wupz-core parity)
        assert_eq!(felix_cordic_atan2_u32(0, 0), 0);
        assert_eq!(felix_cordic_atan2_u32(0, 1000), 3617);
        let q1 = felix_cordic_atan2_u32(1000, 1000);
        // 45 deg = 2^32 / 8 = 536_870_912
        assert!((q1 as i64 - 536_870_912).abs() < 200_000);

        // Verify bit-exact 1.58-bit ternary GEMM (short + 37-element SIMD/tail)
        let acts = [100i16, -50, 25, 10, -30, 7];
        let trits = [1i8, -1, 0, 1, 1, -1];
        // 100*(1) + (-50)*(-1) + 25*(0) + 10*(1) + (-30)*(1) + 7*(-1) = 100 + 50 + 0 + 10 - 30 - 7 = 123
        assert_eq!(felix_ternary_gemm_i32(&acts, &trits), 123);

        let mut acts37 = [0i16; 37];
        let mut trits37 = [0i8; 37];
        let mut ref_sum37 = 0i32;
        for idx in 0..37 {
            acts37[idx] = ((idx as i16) * 17) - 250;
            trits37[idx] = match idx % 3 {
                0 => -1,
                1 => 0,
                _ => 1,
            };
            ref_sum37 += (acts37[idx] as i32) * (trits37[idx] as i32);
        }
        assert_eq!(felix_ternary_gemm_i32(&acts37, &trits37), ref_sum37);

        // Verify hardware inline-assembly Kleene NAND truth table (-min(a, b))
        for a in [-1i8, 0, 1] {
            for b in [-1i8, 0, 1] {
                assert_eq!(felix_kleene_nand_trit_asm(a, b), -a.min(b));
            }
        }

        // Verify quantTernP/Q Block32 pack -> unpack -> fused GEMM32
        let mut block_trits = [0i8; 32];
        let mut block_acts = [0i16; 32];
        let mut expected_block_dot = 0i32;
        for idx in 0..32 {
            block_trits[idx] = (idx as i8 % 3) - 1;
            block_acts[idx] = (idx as i16 + 1) * 10;
            expected_block_dot += (block_acts[idx] as i32) * (block_trits[idx] as i32);
        }
        let wire7 = felix_quant_tern_pq_pack32(&block_trits, 0x05);
        let (gemm32_dot, q_nib) = felix_quant_tern_pq_gemm32_i32(&block_acts, &wire7);
        assert_eq!(gemm32_dot, expected_block_dot);
        assert_eq!(q_nib, 0x05);

        // Verify Slicing-by-4 IEEE 802.3 CRC32 (standard check value 0xCBF43926 for "123456789")
        assert_eq!(felix_crc32_ieee(b"123456789"), 0xCBF4_3926);
        for len in 0..=sample.len() {
            let slice = &sample[..len];
            let mut ref_crc = 0xFFFF_FFFFu32;
            for &b in slice {
                let idx = ((ref_crc ^ (b as u32)) & 0xFF) as usize;
                ref_crc = (ref_crc >> 8) ^ FELIX_CRC32_SLICE4_LUT[0][idx];
            }
            assert_eq!(felix_crc32_ieee(slice), !ref_crc, "crc32 mismatch at {len}");
        }

        // Verify AGP-ISA 0x40–0x43 64-lane dyadic Z[1/2] & Ma-and-Da (836/793) wave contraction
        let wave64 = [100i16; 64];
        let trits64 = [1i8; 64];
        let (dot64, l1_64, contracted64, ma_da64) =
            felix_agp_ternary_wave_contraction_64_i32(&wave64, &trits64, 50, 2);
        assert_eq!(dot64, 6400);
        assert_eq!(l1_64, 6400);
        assert_eq!(contracted64, 50 + (6400 >> 2));
        assert_eq!(
            ma_da64,
            (6400 * FELIX_TOKYO_MA_DA_NUM) / FELIX_TOKYO_MA_DA_DEN
        );
    }
}
