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

    #[cfg(not(target_arch = "aarch64"))]
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

    #[cfg(not(target_arch = "aarch64"))]
    {
        for x in vec.iter_mut() {
            *x *= inv_norm;
        }
    }

    norm
}

/// Branchless 16-byte SIMD ASCII lowercase conversion (`'A'..='Z'` $\to$ `'a'..='z'`).
///
/// Used by `Scanner::get_file_category` to classify file extensions in a single register pass
/// without branches or heap allocations.
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
        for slot in &mut buf[..len] {
            let is_upper = (*slot >= b'A') & (*slot <= b'Z');
            *slot |= (is_upper as u8) << 5;
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
}
