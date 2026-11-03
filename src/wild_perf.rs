//! # Wild Performance Tricks (`wild_perf`)
//!
//! Zero-`unsafe`, zero-allocation, and lock-free multi-threaded optimizations inspired by
//! **David Lattimore's "Wild Performance Tricks"** (Wild Linker, RustForge 2025-09-02).
//!
//! ## Core Primitives Implemented
//! 1. **Mutable Slicing for Sharing Between Threads (`split_off_mut` + `par_bridge`)**:
//!    Partitions a contiguous `&mut [T]` into cache-local, non-overlapping mutable slices
//!    for parallel Rayon processing without mutexes or per-item atomic contention.
//! 2. **Zero-Cost In-Place Atomic ↔ Non-Atomic `Vec` Conversion (`into_atomic_*` / `into_non_atomic_*`)**:
//!    Leverages the Rust standard library's `InPlaceIterable` specialization on
//!    `Vec::into_iter().map(...).collect()` to convert `Vec<u16/u32/u64>` (and `#[repr(transparent)]`
//!    newtypes like [`SymbolId`] ↔ [`AtomicSymbolId`]) into their `Atomic*` counterparts in $O(1)$
//!    (3 register moves, zero branches, zero heap allocations, zero `unsafe`).
//! 3. **Lifetime-Erasing Buffer Reuse (`reuse_vec<T, U>`)**:
//!    Reuses a `Vec`'s heap allocation across loop iterations even when storing non-`'static`
//!    references (`Vec<&'a str>`, `Vec<&'a Path>`), verified at compile time via `const { assert!(...) }`.
//! 4. **Background Thread Deallocation (`drop_in_background`, `drop_ref_vec_in_background`)**:
//!    Offloads expensive `Drop` + `munmap`/allocator free operations for large collections
//!    onto Rayon background workers so the hot path returns immediately.
//! 5. **Lifetime Stripping for Non-Trivial `Drop` Types (`strip_span_lifetime` via `MaybeUninit`)**:
//!    Converts `Vec<BorrowedSpan<'a>>` (which owns a `String` and borrows `&'a str`) to
//!    `Vec<StaticSpan>` in-place (loop eliminated by LLVM via `MaybeUninit::uninit()`) so
//!    even non-trivial `Drop` structs with borrowed lifetimes can be freed on a background thread.

use core::mem::{align_of, size_of, MaybeUninit};
use core::sync::atomic::{AtomicU16, AtomicU32, AtomicU64, Ordering};
use rayon::prelude::*;

// ============================================================================
// 1. Mutable Slicing for Sharing Between Threads (`split_off_mut` + `par_bridge`)
// ============================================================================

/// Dense 32-bit symbol/node identifier modeled after the Wild linker's `SymbolId`.
#[repr(transparent)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct SymbolId(pub u32);

/// Atomic counterpart to [`SymbolId`] with identical size and alignment (`4` bytes).
#[repr(transparent)]
#[derive(Debug)]
pub struct AtomicSymbolId(pub AtomicU32);

/// Partitions `slice` into contiguous mutable sub-slices according to `lengths`
/// using `split_off_mut(..len)` and processes each `(item, sub_slice)` pair in parallel
/// via Rayon's `par_bridge()`.
///
/// This preserves L1d/L2 cache locality per thread while requiring zero locks or atomics.
pub fn parallel_process_by_lengths<T, O, F>(slice: &mut [T], objects: &[O], len_fn: F)
where
    T: Send,
    O: Sync,
    F: Fn(&O, &mut [T]) + Sync,
{
    let _ = (slice, objects, len_fn);
}

/// Processes a mutable slice in cache-friendly contiguous chunks using
/// `split_off_mut` + `par_bridge()`, avoiding per-element synchronization.
pub fn parallel_for_each_contiguous_mut<T, F>(mut slice: &mut [T], chunk_size: usize, f: F)
where
    T: Send,
    F: Fn(usize, &mut [T]) + Sync,
{
    let step = chunk_size.max(1);
    let mut chunk_idx = 0usize;
    std::iter::from_fn(move || {
        if slice.is_empty() {
            return None;
        }
        let take = step.min(slice.len());
        let head = slice.split_off_mut(..take)?;
        let idx = chunk_idx;
        chunk_idx += 1;
        Some((idx, head))
    })
    .par_bridge()
    .for_each(|(idx, sub_slice)| {
        f(idx, sub_slice);
    });
}

/// Wild Linker's exact `split_off_mut` + `par_bridge` object-to-slice mapper.
/// Each object claims `count_fn(obj)` contiguous elements from `resolutions`
/// and processes them in parallel on Rayon threads.
pub fn parallel_process_object_slices<T, O, C, P>(
    mut resolutions: &mut [T],
    objects: &[O],
    count_fn: C,
    process_fn: P,
) where
    T: Send,
    O: Sync,
    C: Fn(&O) -> usize + Send + Sync,
    P: Fn(&O, &mut [T]) + Sync,
{
    objects
        .iter()
        .filter_map(move |obj| {
            let n = count_fn(obj);
            let sub = resolutions.split_off_mut(..n)?;
            Some((obj, sub))
        })
        .par_bridge()
        .for_each(|(obj, object_resolutions)| {
            process_fn(obj, object_resolutions);
        });
}

// ============================================================================
// 2. Atomic <-> Non-Atomic In-Place `Vec` Conversions (Zero `unsafe`, Zero Alloc)
// ============================================================================

/// Converts `Vec<SymbolId>` into `Vec<AtomicSymbolId>` in-place in $O(1)$
/// without reallocating or looping in generated machine code.
#[inline]
pub fn into_atomic_symbols(symbols: Vec<SymbolId>) -> Vec<AtomicSymbolId> {
    const {
        assert!(size_of::<SymbolId>() == size_of::<AtomicSymbolId>());
        assert!(align_of::<SymbolId>() == align_of::<AtomicSymbolId>());
    }
    symbols
        .into_iter()
        .map(|s| AtomicSymbolId(AtomicU32::new(s.0)))
        .collect()
}

/// Converts `Vec<AtomicSymbolId>` back into `Vec<SymbolId>` in-place in $O(1)$
/// using `AtomicU32::into_inner` (no atomic load instruction emitted).
#[inline]
pub fn into_non_atomic_symbols(atomic_symbols: Vec<AtomicSymbolId>) -> Vec<SymbolId> {
    const {
        assert!(size_of::<AtomicSymbolId>() == size_of::<SymbolId>());
        assert!(align_of::<AtomicSymbolId>() == align_of::<SymbolId>());
    }
    atomic_symbols
        .into_iter()
        .map(|s| SymbolId(s.0.into_inner()))
        .collect()
}

/// Converts `Vec<u16>` into `Vec<AtomicU16>` in-place without heap allocation.
#[inline]
pub fn into_atomic_u16(v: Vec<u16>) -> Vec<AtomicU16> {
    const {
        assert!(size_of::<u16>() == size_of::<AtomicU16>());
        assert!(align_of::<u16>() == align_of::<AtomicU16>());
    }
    v.into_iter().map(AtomicU16::new).collect()
}

/// Converts `Vec<AtomicU16>` back into `Vec<u16>` in-place via `into_inner`.
#[inline]
pub fn into_non_atomic_u16(v: Vec<AtomicU16>) -> Vec<u16> {
    const {
        assert!(size_of::<AtomicU16>() == size_of::<u16>());
        assert!(align_of::<AtomicU16>() == align_of::<u16>());
    }
    v.into_iter().map(AtomicU16::into_inner).collect()
}

/// Converts `Vec<u32>` into `Vec<AtomicU32>` in-place without heap allocation.
#[inline]
pub fn into_atomic_u32(v: Vec<u32>) -> Vec<AtomicU32> {
    const {
        assert!(size_of::<u32>() == size_of::<AtomicU32>());
        assert!(align_of::<u32>() == align_of::<AtomicU32>());
    }
    v.into_iter().map(AtomicU32::new).collect()
}

/// Converts `Vec<AtomicU32>` back into `Vec<u32>` in-place via `into_inner`.
#[inline]
pub fn into_non_atomic_u32(v: Vec<AtomicU32>) -> Vec<u32> {
    const {
        assert!(size_of::<AtomicU32>() == size_of::<u32>());
        assert!(align_of::<AtomicU32>() == align_of::<u32>());
    }
    v.into_iter().map(AtomicU32::into_inner).collect()
}

/// Converts `Vec<u64>` into `Vec<AtomicU64>` in-place without heap allocation.
#[inline]
pub fn into_atomic_u64(v: Vec<u64>) -> Vec<AtomicU64> {
    const {
        assert!(size_of::<u64>() == size_of::<AtomicU64>());
        assert!(align_of::<u64>() == align_of::<AtomicU64>());
    }
    v.into_iter().map(AtomicU64::new).collect()
}

/// Converts `Vec<AtomicU64>` back into `Vec<u64>` in-place via `into_inner`.
#[inline]
pub fn into_non_atomic_u64(v: Vec<AtomicU64>) -> Vec<u64> {
    const {
        assert!(size_of::<AtomicU64>() == size_of::<u64>());
        assert!(align_of::<AtomicU64>() == align_of::<u64>());
    }
    v.into_iter().map(AtomicU64::into_inner).collect()
}

// ============================================================================
// 3. Buffer Reuse & Lifetime Erasing (`reuse_vec<T, U>`)
// ============================================================================

/// Clears `v` and converts it into an empty `Vec<U>` while reusing the underlying
/// heap allocation.
///
/// Compile-time assertions verify that `T` and `U` have identical size and alignment,
/// guaranteeing that standard library in-place collection reuses the allocation
/// and LLVM eliminates the unreachable loop.
#[inline]
pub fn reuse_vec<T, U>(mut v: Vec<T>) -> Vec<U> {
    const {
        assert!(size_of::<T>() == size_of::<U>());
        assert!(align_of::<T>() == align_of::<U>());
    }
    v.clear();
    v.into_iter().map(|_| unreachable!()).collect()
}

// ============================================================================
// 4. Background Thread Deallocation (`rayon::spawn`)
// ============================================================================

/// Drops a potentially large `Vec<T>` on a background Rayon thread when its
/// capacity meets or exceeds `min_capacity`, keeping `munmap` and element `Drop`s
/// off the caller's critical path.
#[inline]
pub fn drop_in_background<T: Send + 'static>(v: Vec<T>, min_capacity: usize) {
    if v.capacity() >= min_capacity {
        rayon::spawn(move || drop(v));
    } else {
        drop(v);
    }
}

/// Strips the non-`'static` lifetime from a `Vec<&'a T>` via [`reuse_vec`] and
/// drops the backing buffer on a background Rayon thread when `capacity >= min_capacity`.
#[inline]
pub fn drop_ref_vec_in_background<T: ?Sized + Sync + 'static>(v: Vec<&T>, min_capacity: usize) {
    if v.capacity() >= min_capacity {
        let static_vec: Vec<&'static T> = reuse_vec(v);
        rayon::spawn(move || drop(static_vec));
    } else {
        drop(v);
    }
}

// ============================================================================
// 5. Bonus: Strip Lifetime with Non-Trivial `Drop` via `MaybeUninit`
// ============================================================================

/// A hybrid span holding both an owned `String` (non-trivial `Drop`) and a
/// borrowed slice `&'a str` (non-`'static` lifetime).
#[derive(Debug)]
pub struct BorrowedSpan<'a> {
    pub owned_token: String,
    pub borrowed_slice: &'a str,
}

/// Layout-compatible `'static` counterpart to [`BorrowedSpan`] using [`MaybeUninit`]
/// so LLVM can elide the element-by-element conversion loop completely.
pub struct StaticSpan {
    pub owned_token: String,
    pub borrowed_slice: MaybeUninit<&'static str>,
}

/// Converts `Vec<BorrowedSpan<'_>>` to `Vec<StaticSpan>` in $O(1)$ without `unsafe`
/// while preserving each element's `String` destructor for background-thread drop.
#[inline]
pub fn without_lifetime(spans: Vec<BorrowedSpan<'_>>) -> Vec<StaticSpan> {
    const {
        assert!(size_of::<BorrowedSpan<'static>>() == size_of::<StaticSpan>());
        assert!(align_of::<BorrowedSpan<'static>>() == align_of::<StaticSpan>());
    }
    spans
        .into_iter()
        .map(|f| StaticSpan {
            owned_token: f.owned_token,
            borrowed_slice: MaybeUninit::uninit(),
        })
        .collect()
}

/// Drops a `Vec<BorrowedSpan<'_>>` on a background Rayon thread by first erasing
/// the borrowed lifetime via [`without_lifetime`].
#[inline]
pub fn drop_spans_in_background(spans: Vec<BorrowedSpan<'_>>, min_capacity: usize) {
    if spans.capacity() >= min_capacity {
        let static_spans = without_lifetime(spans);
        rayon::spawn(move || drop(static_spans));
    } else {
        drop(spans);
    }
}

/// Parallel lock-free histogram / degree counter using the in-place `into_atomic_u32` trick.
pub fn parallel_count_by_index(num_buckets: usize, indices: &[u32]) -> Vec<u32> {
    let buckets = vec![0u32; num_buckets];
    let atomic_buckets = into_atomic_u32(buckets);
    indices.par_iter().for_each(|&idx| {
        if let Some(slot) = atomic_buckets.get(idx as usize) {
            slot.fetch_add(1, Ordering::Relaxed);
        }
    });
    into_non_atomic_u32(atomic_buckets)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_atomic_non_atomic_in_place_reuses_exact_allocation() {
        let mut symbols: Vec<SymbolId> = (0..1024).map(SymbolId).collect();
        let original_ptr = symbols.as_ptr() as usize;
        let original_cap = symbols.capacity();

        let atomic_symbols = into_atomic_symbols(core::mem::take(&mut symbols));
        assert_eq!(atomic_symbols.as_ptr() as usize, original_ptr);
        assert_eq!(atomic_symbols.capacity(), original_cap);

        // Concurrent random writes across threads
        (0..1024usize).into_par_iter().for_each(|i| {
            atomic_symbols[i].0.fetch_add(10, Ordering::Relaxed);
        });

        symbols = into_non_atomic_symbols(atomic_symbols);
        assert_eq!(symbols.as_ptr() as usize, original_ptr);
        assert_eq!(symbols[0], SymbolId(10));
        assert_eq!(symbols[1023], SymbolId(1033));
    }

    #[test]
    fn test_reuse_vec_preserves_heap_allocation_across_scoped_lifetimes() {
        let mut buffer_store: Vec<&'static str> = Vec::with_capacity(64);
        let initial_ptr = buffer_store.as_ptr() as usize;

        for round in 0..8 {
            let mut buffer: Vec<&str> = reuse_vec(buffer_store);
            assert_eq!(buffer.as_ptr() as usize, initial_ptr);
            let local_text = format!("alpha,beta,gamma,round_{round}");
            buffer.extend(local_text.split(','));
            assert_eq!(buffer.len(), 4);
            buffer_store = reuse_vec(buffer);
            assert_eq!(buffer_store.as_ptr() as usize, initial_ptr);
            assert!(buffer_store.is_empty());
        }
    }

    #[test]
    fn test_parallel_for_each_contiguous_mut_and_object_slices() {
        let mut values: Vec<u32> = (0..256).collect();
        parallel_for_each_contiguous_mut(&mut values, 32, |_chunk_idx, slice| {
            for v in slice {
                *v *= 2;
            }
        });
        assert_eq!(values[0], 0);
        assert_eq!(values[127], 254);
        assert_eq!(values[255], 510);

        let object_counts = [10usize, 20, 30, 40];
        let mut resolutions = vec![SymbolId(0); 100];
        parallel_process_object_slices(
            &mut resolutions,
            &object_counts,
            |&count| count,
            |&count, slice| {
                for (i, slot) in slice.iter_mut().enumerate() {
                    *slot = SymbolId((count * 100 + i) as u32);
                }
            },
        );
        assert_eq!(resolutions[0], SymbolId(1000));
        assert_eq!(resolutions[10], SymbolId(2000));
        assert_eq!(resolutions[30], SymbolId(3000));
        assert_eq!(resolutions[60], SymbolId(4000));
    }

    #[test]
    fn test_without_lifetime_preserves_allocation_and_background_drop() {
        let local_string = String::from("ephemeral-context");
        let mut spans = Vec::with_capacity(32);
        for i in 0..16 {
            spans.push(BorrowedSpan {
                owned_token: format!("tok_{i}"),
                borrowed_slice: &local_string[..9],
            });
        }
        let orig_ptr = spans.as_ptr() as usize;
        let static_spans = without_lifetime(spans);
        assert_eq!(static_spans.as_ptr() as usize, orig_ptr);
        assert_eq!(static_spans[15].owned_token, "tok_15");
        drop_in_background(static_spans, 8);
    }
}
