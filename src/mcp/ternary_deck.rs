//! ternary_deck — vectorised ternary row encoders and a 64-bit error accumulator.
//!
//! Three kernels, each written three times: a scalar reference, an AVX2 path, and
//! a NEON path, chosen at runtime. The scalar reference is the specification; the
//! vector paths must be **bit-identical** to it at every width, including widths
//! that are not a multiple of the tile.
//!
//! | kernel | tile | output |
//! |---|---|---|
//! | `pack_4trits_row_*` | 32 trits (AVX2), 64 (NEON) | 4 trits/byte, 2 bits each |
//! | `pack_5trits_row_*` | 20 trits (AVX2), 40 (NEON) | 5 trits/byte, base-3 |
//! | `compute_ternary_sse_*` | 32 px (AVX2), 16 (NEON) | exact `u64` sum of squared error |
//!
//! ## why the accumulator is `u64`
//!
//! `E = Σ (xᵢ − c_q(xᵢ))²` with `xᵢ` in `0..=255` peaks at `65_025` per pixel.
//! A 4K frame is `8_294_400` pixels, so `E` can reach `5.4e11` — a `u32` would
//! wrap silently after `4.29e9`, which is only ~66 thousand worst-case pixels.
//! The vector paths widen to 64 bits before accumulating, and the test asserts a
//! total above `u32::MAX`.
//!
//! ## the layouts
//!
//! **4 trits/byte** — `t0 | t1<<2 | t2<<4 | t3<<6`, least significant field first,
//! matching `ternary_lane::pack_trits`. `0b11` is unreachable because every trit
//! is masked to `0..=2`.
//!
//! **5 trits/byte** — base-3: `b = t0 + 3·t1 + 9·t2 + 27·t3 + 81·t4`, the same
//! `3^5 = 243 <= 256` packing the constellation uses everywhere. Max value 242,
//! so the whole base-3 byte is a valid index and unpacking is a division walk.

#![allow(unknown_lints, clippy::chunks_exact_to_as_chunks)]

/// Pack four trits (`0..=2`) into one byte, least significant field first.
#[inline]
pub fn pack_4trits_byte(trits: [u8; 4]) -> u8 {
    let t0 = trits[0].min(2);
    let t1 = trits[1].min(2);
    let t2 = trits[2].min(2);
    let t3 = trits[3].min(2);
    t0 | (t1 << 2) | (t2 << 4) | (t3 << 6)
}

/// Unpack four trits from one byte.
#[inline]
pub fn unpack_4trits_byte(byte: u8) -> [u8; 4] {
    [
        byte & 0x03,
        (byte >> 2) & 0x03,
        (byte >> 4) & 0x03,
        (byte >> 6) & 0x03,
    ]
}

/// Pack five trits into one byte in base 3: `Σ tᵢ · 3ⁱ`.
#[inline]
pub fn pack_5trits_byte(trits: [u8; 5]) -> u8 {
    let mut value = 0u16;
    let mut place = 1u16;
    for trit in trits {
        value += trit.min(2) as u16 * place;
        place *= 3;
    }
    value as u8 // <= 242, so it always fits
}

/// Unpack five trits from one base-3 byte.
#[inline]
pub fn unpack_5trits_byte(byte: u8) -> [u8; 5] {
    let mut value = byte as u32;
    let mut out = [0u8; 5];
    for slot in out.iter_mut() {
        *slot = (value % 3) as u8;
        value /= 3;
    }
    out
}

// ── 4 trits per byte ────────────────────────────────────────────────────────

/// Pack a row of trits four to a byte. Returns the number of bytes written.
pub fn pack_4trits_row_simd(trits: &[u8], out: &mut [u8]) -> usize {
    let expected = trits.len().div_ceil(4);
    assert!(
        out.len() >= expected,
        "output buffer too small: need {expected} bytes, got {}",
        out.len()
    );

    #[cfg(target_arch = "x86_64")]
    {
        if std::arch::is_x86_feature_detected!("avx2") {
            // SAFETY: AVX2 verified at runtime; `out` is at least `expected` long.
            unsafe {
                return pack_4trits_row_avx2(trits, out);
            }
        }
    }
    #[cfg(target_arch = "aarch64")]
    {
        if std::arch::is_aarch64_feature_detected!("neon") {
            // SAFETY: NEON is baseline on AArch64; `out` is at least `expected`.
            unsafe {
                return pack_4trits_row_neon(trits, out);
            }
        }
    }
    pack_4trits_row_scalar(trits, out)
}

/// The specification: one byte at a time, with a padded tail.
pub fn pack_4trits_row_scalar(trits: &[u8], out: &mut [u8]) -> usize {
    let mut written = 0usize;
    for chunk in trits.chunks_exact(4) {
        out[written] = pack_4trits_byte([chunk[0], chunk[1], chunk[2], chunk[3]]);
        written += 1;
    }
    let tail = trits.chunks_exact(4).remainder();
    if !tail.is_empty() {
        let mut padded = [0u8; 4];
        padded[..tail.len()].copy_from_slice(tail);
        out[written] = pack_4trits_byte(padded);
        written += 1;
    }
    written
}

#[cfg(target_arch = "aarch64")]
#[target_feature(enable = "neon")]
unsafe fn pack_4trits_row_neon(trits: &[u8], out: &mut [u8]) -> usize {
    use std::arch::aarch64::*;

    let n = trits.len();
    let mut input = 0usize;
    let mut written = 0usize;
    let mask = vdupq_n_u8(0x03);

    // vld4q_u8 de-interleaves 64 trits into four vectors of every fourth trit,
    // which is exactly the field layout: lane k holds t[4k..4k+4].
    while input + 64 <= n {
        let deinterleaved = vld4q_u8(trits.as_ptr().add(input));
        let f0 = vandq_u8(deinterleaved.0, mask);
        let f1 = vshlq_n_u8::<2>(vandq_u8(deinterleaved.1, mask));
        let f2 = vshlq_n_u8::<4>(vandq_u8(deinterleaved.2, mask));
        let f3 = vshlq_n_u8::<6>(vandq_u8(deinterleaved.3, mask));
        let packed = vorrq_u8(vorrq_u8(f0, f1), vorrq_u8(f2, f3));
        vst1q_u8(out.as_mut_ptr().add(written), packed);
        input += 64;
        written += 16;
    }

    if input < n {
        written += pack_4trits_row_scalar(&trits[input..], &mut out[written..]);
    }
    written
}

#[cfg(target_arch = "x86_64")]
#[target_feature(enable = "avx2")]
unsafe fn pack_4trits_row_avx2(trits: &[u8], out: &mut [u8]) -> usize {
    use std::arch::x86_64::*;

    let n = trits.len();
    let mut input = 0usize;
    let mut written = 0usize;

    let field = _mm256_set1_epi8(0x03);
    // maddubs pairs: (t0·1 + t1·4) per 16-bit lane
    let pair_weights = _mm256_set1_epi16(0x0401);
    // madd pairs those again: (p0·1 + p1·16) per 32-bit lane
    let quad_weights = _mm256_set1_epi32(0x0010_0001);
    // gather the low byte of each 32-bit lane
    let compact = _mm256_setr_epi8(
        0, 4, 8, 12, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, 0, 4, 8, 12, -1, -1, -1, -1,
        -1, -1, -1, -1, -1, -1, -1, -1,
    );

    // 32 trits -> 8 bytes per iteration
    while input + 32 <= n {
        let raw = _mm256_loadu_si256(trits.as_ptr().add(input) as *const __m256i);
        let clamped = _mm256_and_si256(raw, field);
        let pairs = _mm256_maddubs_epi16(clamped, pair_weights);
        let quads = _mm256_madd_epi16(pairs, quad_weights);
        let compacted = _mm256_shuffle_epi8(quads, compact);

        let low = _mm256_extract_epi32::<0>(compacted) as u32;
        let high = _mm256_extract_epi32::<4>(compacted) as u32;
        let word = (low as u64) | ((high as u64) << 32);
        std::ptr::copy_nonoverlapping(
            word.to_le_bytes().as_ptr(),
            out.as_mut_ptr().add(written),
            8,
        );

        input += 32;
        written += 8;
    }

    if input < n {
        written += pack_4trits_row_scalar(&trits[input..], &mut out[written..]);
    }
    written
}

// ── 5 trits per byte, base 3 ────────────────────────────────────────────────

/// Pack a row of trits five to a byte in base 3. Returns bytes written.
pub fn pack_5trits_row_simd(trits: &[u8], out: &mut [u8]) -> usize {
    let expected = trits.len().div_ceil(5);
    assert!(
        out.len() >= expected,
        "output buffer too small: need {expected} bytes, got {}",
        out.len()
    );

    #[cfg(target_arch = "x86_64")]
    {
        if std::arch::is_x86_feature_detected!("avx2") {
            // SAFETY: AVX2 verified at runtime; buffer sized above.
            unsafe {
                return pack_5trits_row_avx2(trits, out);
            }
        }
    }
    #[cfg(target_arch = "aarch64")]
    {
        if std::arch::is_aarch64_feature_detected!("neon") {
            // SAFETY: NEON is baseline on AArch64; buffer sized above.
            unsafe {
                return pack_5trits_row_neon(trits, out);
            }
        }
    }
    pack_5trits_row_scalar(trits, out)
}

/// The specification for the base-3 packing.
pub fn pack_5trits_row_scalar(trits: &[u8], out: &mut [u8]) -> usize {
    let mut written = 0usize;
    for chunk in trits.chunks_exact(5) {
        out[written] = pack_5trits_byte([chunk[0], chunk[1], chunk[2], chunk[3], chunk[4]]);
        written += 1;
    }
    let tail = trits.chunks_exact(5).remainder();
    if !tail.is_empty() {
        let mut padded = [0u8; 5];
        padded[..tail.len()].copy_from_slice(tail);
        out[written] = pack_5trits_byte(padded);
        written += 1;
    }
    written
}

#[cfg(target_arch = "aarch64")]
#[target_feature(enable = "neon")]
unsafe fn pack_5trits_row_neon(trits: &[u8], out: &mut [u8]) -> usize {
    use std::arch::aarch64::*;

    let n = trits.len();
    let mut input = 0usize;
    let mut written = 0usize;

    let max_trit = vdupq_n_u8(2);
    // 8 groups of 5 trits live inside 48 bytes, gathered through a 3-register table
    let gather: [[u8; 8]; 5] = [
        [0, 5, 10, 15, 20, 25, 30, 35],
        [1, 6, 11, 16, 21, 26, 31, 36],
        [2, 7, 12, 17, 22, 27, 32, 37],
        [3, 8, 13, 18, 23, 28, 33, 38],
        [4, 9, 14, 19, 24, 29, 34, 39],
    ];
    let w3 = vdup_n_u8(3);
    let w9 = vdup_n_u8(9);
    let w27 = vdup_n_u8(27);
    let w81 = vdup_n_u8(81);

    while input + 48 <= n {
        let table = vld1q_u8_x3(trits.as_ptr().add(input));
        let clamped = uint8x16x3_t(
            vminq_u8(table.0, max_trit),
            vminq_u8(table.1, max_trit),
            vminq_u8(table.2, max_trit),
        );

        let t0 = vqtbl3_u8(clamped, vld1_u8(gather[0].as_ptr()));
        let t1 = vqtbl3_u8(clamped, vld1_u8(gather[1].as_ptr()));
        let t2 = vqtbl3_u8(clamped, vld1_u8(gather[2].as_ptr()));
        let t3 = vqtbl3_u8(clamped, vld1_u8(gather[3].as_ptr()));
        let t4 = vqtbl3_u8(clamped, vld1_u8(gather[4].as_ptr()));

        // accumulators stay in u8: the maximum is 2 + 6 + 18 + 54 + 162 = 242
        let mut acc = vmla_u8(t0, t1, w3);
        acc = vmla_u8(acc, t2, w9);
        acc = vmla_u8(acc, t3, w27);
        acc = vmla_u8(acc, t4, w81);

        vst1_u8(out.as_mut_ptr().add(written), acc);

        input += 40;
        written += 8;
    }

    if input < n {
        written += pack_5trits_row_scalar(&trits[input..], &mut out[written..]);
    }
    written
}

#[cfg(target_arch = "x86_64")]
#[target_feature(enable = "avx2")]
unsafe fn pack_5trits_row_avx2(trits: &[u8], out: &mut [u8]) -> usize {
    use std::arch::x86_64::*;

    let n = trits.len();
    let mut input = 0usize;
    let mut written = 0usize;

    let max_trit = _mm256_set1_epi8(2);
    // two groups of five trits per 128-bit lane, each padded to eight bytes
    let gather = _mm256_setr_epi8(
        0, 1, 2, 3, 4, -1, -1, -1, 5, 6, 7, 8, 9, -1, -1, -1, 0, 1, 2, 3, 4, -1, -1, -1, 5, 6, 7,
        8, 9, -1, -1, -1,
    );
    let powers = _mm256_setr_epi8(
        1, 3, 9, 27, 81, 0, 0, 0, 1, 3, 9, 27, 81, 0, 0, 0, 1, 3, 9, 27, 81, 0, 0, 0, 1, 3, 9, 27,
        81, 0, 0, 0,
    );
    let ones = _mm256_set1_epi16(1);

    // 20 trits -> 4 bytes. The second 16-byte load starts at +10, so the guard
    // is +26: enough to cover it without reading past the row.
    while input + 26 <= n {
        let base = trits.as_ptr().add(input);
        let low = _mm_loadu_si128(base as *const __m128i);
        let high = _mm_loadu_si128(base.add(10) as *const __m128i);
        let combined = _mm256_set_m128i(high, low);

        let clamped = _mm256_min_epu8(combined, max_trit);
        let aligned = _mm256_shuffle_epi8(clamped, gather);

        // (t0 + 3t1), (9t2 + 27t3), (81t4), then the pair sums
        let products = _mm256_maddubs_epi16(aligned, powers);
        let quads = _mm256_madd_epi16(products, ones);
        let summed = _mm256_hadd_epi32(quads, quads);

        let b0 = _mm256_extract_epi32::<0>(summed) as u8;
        let b1 = _mm256_extract_epi32::<1>(summed) as u8;
        let b2 = _mm256_extract_epi32::<4>(summed) as u8;
        let b3 = _mm256_extract_epi32::<5>(summed) as u8;

        let target = out.as_mut_ptr().add(written);
        *target = b0;
        *target.add(1) = b1;
        *target.add(2) = b2;
        *target.add(3) = b3;

        input += 20;
        written += 4;
    }

    if input < n {
        written += pack_5trits_row_scalar(&trits[input..], &mut out[written..]);
    }
    written
}

// ── the 64-bit reconstruction error accumulator ─────────────────────────────

/// `E = Σ (xᵢ − c_q(xᵢ))²` over the whole slice, exactly, in `u64`.
pub fn compute_ternary_sse_simd(pixels: &[u8], thresholds: (u8, u8), centroids: [u8; 3]) -> u64 {
    #[cfg(target_arch = "x86_64")]
    {
        if std::arch::is_x86_feature_detected!("avx2") {
            // SAFETY: AVX2 verified at runtime.
            unsafe {
                return compute_ternary_sse_avx2(pixels, thresholds, centroids);
            }
        }
    }
    #[cfg(target_arch = "aarch64")]
    {
        if std::arch::is_aarch64_feature_detected!("neon") {
            // SAFETY: NEON is baseline on AArch64.
            unsafe {
                return compute_ternary_sse_neon(pixels, thresholds, centroids);
            }
        }
    }
    compute_ternary_sse_scalar(pixels, thresholds, centroids)
}

/// The specification for `E`.
#[inline]
pub fn compute_ternary_sse_scalar(pixels: &[u8], thresholds: (u8, u8), centroids: [u8; 3]) -> u64 {
    let (low, high) = thresholds;
    let [c0, c1, c2] = centroids;
    let mut total = 0u64;
    for &x in pixels {
        let centroid = if x < low {
            c0
        } else if x < high {
            c1
        } else {
            c2
        };
        let difference = x as i32 - centroid as i32;
        total += (difference * difference) as u64;
    }
    total
}

#[cfg(target_arch = "aarch64")]
#[target_feature(enable = "neon")]
unsafe fn compute_ternary_sse_neon(pixels: &[u8], thresholds: (u8, u8), centroids: [u8; 3]) -> u64 {
    use std::arch::aarch64::*;

    let n = pixels.len();
    let mut offset = 0usize;

    let low = vdupq_n_u8(thresholds.0);
    let high = vdupq_n_u8(thresholds.1);
    let c0 = vdupq_n_u8(centroids[0]);
    let c1 = vdupq_n_u8(centroids[1]);
    let c2 = vdupq_n_u8(centroids[2]);
    let mut acc = vdupq_n_u64(0);

    while offset + 16 <= n {
        let x = vld1q_u8(pixels.as_ptr().add(offset));
        // vbslq_u8 takes the first vector where the mask bit is 1
        let chosen_low = vbslq_u8(vcgeq_u8(x, low), c1, c0);
        let chosen = vbslq_u8(vcgeq_u8(x, high), c2, chosen_low);

        let difference = vabdq_u8(x, chosen);
        let squares_low = vmull_u8(vget_low_u8(difference), vget_low_u8(difference));
        let squares_high = vmull_u8(vget_high_u8(difference), vget_high_u8(difference));

        // pairwise widen u16 -> u32 (max 130_050), then accumulate into u64
        let mut sums = vpaddlq_u16(squares_low);
        sums = vpadalq_u16(sums, squares_high);
        acc = vpadalq_u32(acc, sums);

        offset += 16;
    }

    let mut total = vaddvq_u64(acc);
    if offset < n {
        total += compute_ternary_sse_scalar(&pixels[offset..], thresholds, centroids);
    }
    total
}

#[cfg(target_arch = "x86_64")]
#[target_feature(enable = "avx2")]
unsafe fn compute_ternary_sse_avx2(pixels: &[u8], thresholds: (u8, u8), centroids: [u8; 3]) -> u64 {
    use std::arch::x86_64::*;

    let n = pixels.len();
    let mut offset = 0usize;

    let low = _mm256_set1_epi8(thresholds.0 as i8);
    let high = _mm256_set1_epi8(thresholds.1 as i8);
    let c0 = _mm256_set1_epi8(centroids[0] as i8);
    let c1 = _mm256_set1_epi8(centroids[1] as i8);
    let c2 = _mm256_set1_epi8(centroids[2] as i8);
    let zero = _mm256_setzero_si256();

    let mut acc_low = _mm256_setzero_si256();
    let mut acc_high = _mm256_setzero_si256();

    while offset + 32 <= n {
        let x = _mm256_loadu_si256(pixels.as_ptr().add(offset) as *const __m256i);

        // unsigned >= : max(x, t) == x
        let ge_low = _mm256_cmpeq_epi8(_mm256_max_epu8(x, low), x);
        let ge_high = _mm256_cmpeq_epi8(_mm256_max_epu8(x, high), x);

        let selected = _mm256_blendv_epi8(c0, c1, ge_low);
        let chosen = _mm256_blendv_epi8(selected, c2, ge_high);

        // branchless |x - c| in u8
        let difference = _mm256_or_si256(_mm256_subs_epu8(x, chosen), _mm256_subs_epu8(chosen, x));

        let widened_low = _mm256_unpacklo_epi8(difference, zero);
        let widened_high = _mm256_unpackhi_epi8(difference, zero);
        let squares = _mm256_add_epi32(
            _mm256_madd_epi16(widened_low, widened_low),
            _mm256_madd_epi16(widened_high, widened_high),
        );

        // widen the 32-bit pair sums into 64-bit accumulators before adding
        acc_low = _mm256_add_epi64(acc_low, _mm256_unpacklo_epi32(squares, zero));
        acc_high = _mm256_add_epi64(acc_high, _mm256_unpackhi_epi32(squares, zero));

        offset += 32;
    }

    let total_vector = _mm256_add_epi64(acc_low, acc_high);
    let mut lanes = [0u64; 4];
    _mm256_storeu_si256(lanes.as_mut_ptr() as *mut __m256i, total_vector);
    let mut total = lanes.iter().sum::<u64>();

    if offset < n {
        total += compute_ternary_sse_scalar(&pixels[offset..], thresholds, centroids);
    }
    total
}

#[cfg(test)]
mod tests {
    use super::*;

    fn trits(width: usize) -> Vec<u8> {
        (0..width).map(|i| ((i * 7 + 3) % 3) as u8).collect()
    }

    #[test]
    fn the_byte_packings_round_trip() {
        // every reachable 4-trit combination (3^4 = 81) survives pack -> unpack
        for packed in 0u32..81 {
            let mut value = packed;
            let mut trits = [0u8; 4];
            for slot in trits.iter_mut() {
                *slot = (value % 3) as u8;
                value /= 3;
            }
            assert_eq!(unpack_4trits_byte(pack_4trits_byte(trits)), trits);
        }
        for value in 0u8..=242 {
            let five = unpack_5trits_byte(value);
            assert_eq!(pack_5trits_byte(five), value);
        }
        // 243..=255 cannot be produced by five base-3 digits
        assert_eq!(pack_5trits_byte([2, 2, 2, 2, 2]), 242);
    }

    #[test]
    fn the_four_trit_encoder_matches_the_scalar_reference_at_every_width() {
        for width in [0usize, 1, 3, 4, 5, 31, 32, 33, 53, 3840, 3847] {
            let row = trits(width);
            let expected_bytes = width.div_ceil(4);
            let mut scalar = vec![0u8; expected_bytes];
            let mut vector = vec![0u8; expected_bytes];

            assert_eq!(pack_4trits_row_scalar(&row, &mut scalar), expected_bytes);
            assert_eq!(pack_4trits_row_simd(&row, &mut vector), expected_bytes);
            assert_eq!(vector, scalar, "4-trit encoder diverged at width {width}");
        }
    }

    #[test]
    fn the_four_trit_encoder_round_trips_a_4k_row() {
        let row = trits(3840);
        let mut packed = vec![0u8; 3840 / 4];
        assert_eq!(pack_4trits_row_simd(&row, &mut packed), 960);
        for (index, byte) in packed.iter().enumerate() {
            assert_eq!(
                unpack_4trits_byte(*byte),
                [
                    row[index * 4],
                    row[index * 4 + 1],
                    row[index * 4 + 2],
                    row[index * 4 + 3]
                ]
            );
        }
    }

    #[test]
    fn the_five_trit_encoder_matches_the_scalar_reference_at_every_width() {
        for width in [
            0usize, 1, 4, 5, 6, 19, 20, 21, 25, 26, 40, 41, 53, 3840, 3847,
        ] {
            let row = trits(width);
            let expected_bytes = width.div_ceil(5);
            let mut scalar = vec![0u8; expected_bytes];
            let mut vector = vec![0u8; expected_bytes];

            assert_eq!(pack_5trits_row_scalar(&row, &mut scalar), expected_bytes);
            assert_eq!(pack_5trits_row_simd(&row, &mut vector), expected_bytes);
            assert_eq!(vector, scalar, "5-trit encoder diverged at width {width}");
        }
    }

    #[test]
    fn the_five_trit_encoder_round_trips_a_4k_row() {
        let row = trits(3840);
        let mut packed = vec![0u8; 3840 / 5];
        assert_eq!(pack_5trits_row_simd(&row, &mut packed), 768);
        for (index, byte) in packed.iter().enumerate() {
            let unpacked = unpack_5trits_byte(*byte);
            for slot in 0..5 {
                assert_eq!(
                    unpacked[slot],
                    row[index * 5 + slot],
                    "base-3 round trip broke at group {index} slot {slot}"
                );
            }
        }
    }

    #[test]
    fn the_error_accumulator_is_exact_and_survives_u32() {
        let pixels: Vec<u8> = (0..100_003).map(|i| ((i * 97 + 13) & 0xFF) as u8).collect();
        let thresholds = (55u8, 195u8);
        let centroids = [12u8, 128u8, 244u8];

        let scalar = compute_ternary_sse_scalar(&pixels, thresholds, centroids);
        let vector = compute_ternary_sse_simd(&pixels, thresholds, centroids);
        assert_eq!(vector, scalar, "the vector accumulator diverged");

        // 100_000 pixels at the worst-case 255² each = 6_502_500_000 > u32::MAX
        let extreme = vec![255u8; 100_000];
        let total = compute_ternary_sse_simd(&extreme, (85, 170), [0, 0, 0]);
        assert_eq!(total, 6_502_500_000u64);
        assert!(
            total > u32::MAX as u64,
            "the test must exceed u32 to mean anything"
        );
    }

    #[test]
    fn the_error_accumulator_matches_at_every_tail_length() {
        // 16 (NEON) and 32 (AVX2) both vectorise; the neighbouring lengths are
        // the ones that exercise the scalar tail.
        for len in [0usize, 1, 15, 16, 17, 31, 32, 33, 63, 64, 65, 1000] {
            let pixels: Vec<u8> = (0..len).map(|i| (i * 37 % 256) as u8).collect();
            assert_eq!(
                compute_ternary_sse_simd(&pixels, (64, 180), [16, 122, 235]),
                compute_ternary_sse_scalar(&pixels, (64, 180), [16, 122, 235]),
                "tail handling diverged at length {len}"
            );
        }
    }
}
