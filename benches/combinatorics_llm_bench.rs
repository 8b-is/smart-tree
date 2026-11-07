//! Weight-Space SIMD Pack/GEMV Benchmarks
//!
//! Measures throughput of the bent 5-trit packer and fused unpack-unbend-GEMV
//! for ERNIE 5.5 LatentMoE dimensions (4096→2048→4096).

use criterion::{criterion_group, criterion_main, Criterion, Throughput};
use rand::Rng;
use st::mcp_combinatorics_llm::latent_moe_simd::{
    fused_unpack_unbend_gemv_row_simd, latent_moe_gemv_par_simd, pack_latent_moe_matrix_par_simd,
    quantize_bend_pack_row_simd, unpack_and_unbend_row, ERNIE55_HIDDEN_DIM, ERNIE55_LATENT_DIM,
    PACKED_LATENT_ROW_BYTES,
};
use std::hint::black_box;

fn bench_packer(c: &mut Criterion) {
    let mut group = c.benchmark_group("latent_moe_packer");
    group.throughput(Throughput::Bytes(ERNIE55_LATENT_DIM as u64));

    // Generate random weights once
    let mut rng = rand::thread_rng();
    let weights: Vec<u8> = (0..ERNIE55_LATENT_DIM).map(|_| rng.gen()).collect();
    let thresholds = (85u8, 170u8);
    let centroids = [0u8, 128u8, 255u8];
    let bend_k = 1u8;

    group.bench_function("pack_2048_row_scalar", |b| {
        let mut packed = vec![0u8; PACKED_LATENT_ROW_BYTES];
        b.iter(|| {
            quantize_bend_pack_row_simd(
                black_box(&weights),
                thresholds,
                centroids,
                bend_k,
                black_box(&mut packed),
            )
        });
    });

    group.bench_function("pack_2048_row_par", |b| {
        b.iter(|| {
            let mut packed = vec![0u8; PACKED_LATENT_ROW_BYTES];
            quantize_bend_pack_row_simd(
                black_box(&weights),
                thresholds,
                centroids,
                bend_k,
                black_box(&mut packed),
            )
        });
    });

    // Full matrix pack
    let matrix_weights: Vec<u8> = (0..ERNIE55_HIDDEN_DIM * ERNIE55_LATENT_DIM)
        .map(|_| rng.gen())
        .collect();
    group.throughput(Throughput::Bytes(
        (ERNIE55_HIDDEN_DIM * ERNIE55_LATENT_DIM) as u64,
    ));
    group.bench_function("pack_4096x2048_matrix_par", |b| {
        b.iter(|| {
            pack_latent_moe_matrix_par_simd(
                black_box(&matrix_weights),
                ERNIE55_HIDDEN_DIM,
                ERNIE55_LATENT_DIM,
                thresholds,
                centroids,
                bend_k,
                1.0f32,
            )
        });
    });

    group.finish();
}

fn bench_unpacker(c: &mut Criterion) {
    let mut group = c.benchmark_group("latent_moe_unpacker");
    group.throughput(Throughput::Bytes(ERNIE55_LATENT_DIM as u64));

    // Pre-pack a row
    let mut rng = rand::thread_rng();
    let weights: Vec<u8> = (0..ERNIE55_LATENT_DIM).map(|_| rng.gen()).collect();
    let thresholds = (85u8, 170u8);
    let centroids = [0u8, 128u8, 255u8];
    let bend_k = 1u8;
    let mut packed = vec![0u8; PACKED_LATENT_ROW_BYTES];
    quantize_bend_pack_row_simd(&weights, thresholds, centroids, bend_k, &mut packed);

    group.bench_function("unpack_2048_row", |b| {
        let mut out = vec![0i8; ERNIE55_LATENT_DIM];
        b.iter(|| unpack_and_unbend_row(black_box(&packed), bend_k, black_box(&mut out)));
    });

    group.finish();
}

fn bench_gemv(c: &mut Criterion) {
    let mut group = c.benchmark_group("latent_moe_gemv");
    group.throughput(Throughput::Elements(ERNIE55_LATENT_DIM as u64));

    // Pre-pack a matrix
    let mut rng = rand::thread_rng();
    let weights: Vec<u8> = (0..ERNIE55_HIDDEN_DIM * ERNIE55_LATENT_DIM)
        .map(|_| rng.gen())
        .collect();
    let thresholds = (85u8, 170u8);
    let centroids = [0u8, 128u8, 255u8];
    let bend_k = 1u8;

    let packed_matrix = pack_latent_moe_matrix_par_simd(
        &weights,
        ERNIE55_HIDDEN_DIM,
        ERNIE55_LATENT_DIM,
        thresholds,
        centroids,
        bend_k,
        1.0f32,
    );

    // Random activations
    let activations: Vec<i8> = (0..ERNIE55_LATENT_DIM)
        .map(|_| rng.gen_range(-128..127))
        .collect();

    group.bench_function("gemv_4096x2048_par", |b| {
        b.iter(|| {
            latent_moe_gemv_par_simd(
                black_box(&packed_matrix),
                black_box(&activations),
                black_box(1.0f32),
            )
        });
    });

    // Single row GEMV
    let row_packed = &packed_matrix.packed_data[0..PACKED_LATENT_ROW_BYTES];
    group.throughput(Throughput::Elements(ERNIE55_LATENT_DIM as u64));
    group.bench_function("gemv_single_row_2048", |b| {
        b.iter(|| {
            fused_unpack_unbend_gemv_row_simd(
                black_box(row_packed),
                black_box(&activations),
                bend_k,
            )
        });
    });

    group.finish();
}

fn bench_fused_pack_gemv(c: &mut Criterion) {
    let mut group = c.benchmark_group("latent_moe_fused_pack_gemv");

    let mut rng = rand::thread_rng();
    let weights: Vec<u8> = (0..ERNIE55_LATENT_DIM).map(|_| rng.gen()).collect();
    let thresholds = (85u8, 170u8);
    let centroids = [0u8, 128u8, 255u8];
    let bend_k = 1u8;
    let activations: Vec<i8> = (0..ERNIE55_LATENT_DIM)
        .map(|_| rng.gen_range(-128..127))
        .collect();

    group.bench_function("pack_then_gemv_2048", |b| {
        let mut packed = vec![0u8; PACKED_LATENT_ROW_BYTES];
        b.iter(|| {
            quantize_bend_pack_row_simd(
                black_box(&weights),
                thresholds,
                centroids,
                bend_k,
                black_box(&mut packed),
            );
            fused_unpack_unbend_gemv_row_simd(black_box(&packed), black_box(&activations), bend_k)
        });
    });

    group.finish();
}

fn bench_ternary_simd_kernels(c: &mut Criterion) {
    use st::mcp_combinatorics_llm::ternary_deck::{
        compute_ternary_sse_scalar, compute_ternary_sse_simd, pack_4trits_row_scalar,
        pack_4trits_row_simd, pack_5trits_row_scalar, pack_5trits_row_simd,
    };

    let mut group = c.benchmark_group("ternary_4k_simd_vs_scalar");
    let width = 3840usize;
    let row_trits: Vec<u8> = (0..width).map(|i| (i % 3) as u8).collect();

    let mut out_4trit = vec![0u8; width / 4];
    group.bench_function("pack_4trits_3840_scalar", |b| {
        b.iter(|| {
            black_box(pack_4trits_row_scalar(
                black_box(&row_trits),
                &mut out_4trit,
            ))
        })
    });
    group.bench_function("pack_4trits_3840_simd", |b| {
        b.iter(|| black_box(pack_4trits_row_simd(black_box(&row_trits), &mut out_4trit)))
    });

    let mut out_5trit = vec![0u8; width / 5];
    group.bench_function("pack_5trits_3840_scalar", |b| {
        b.iter(|| {
            black_box(pack_5trits_row_scalar(
                black_box(&row_trits),
                &mut out_5trit,
            ))
        })
    });
    group.bench_function("pack_5trits_3840_simd", |b| {
        b.iter(|| black_box(pack_5trits_row_simd(black_box(&row_trits), &mut out_5trit)))
    });

    // 64 rows of a 4K frame: enough to be honest about the u64 accumulator
    // without making the bench run for minutes.
    let pixels: Vec<u8> = (0..(3840 * 64)).map(|i| (i * 37 % 256) as u8).collect();
    let thresholds = (64u8, 180u8);
    let centroids = [16u8, 122u8, 235u8];

    group.bench_function("quantizer_sse_u64_scalar", |b| {
        b.iter(|| {
            black_box(compute_ternary_sse_scalar(
                black_box(&pixels),
                thresholds,
                centroids,
            ))
        })
    });
    group.bench_function("quantizer_sse_u64_simd", |b| {
        b.iter(|| {
            black_box(compute_ternary_sse_simd(
                black_box(&pixels),
                thresholds,
                centroids,
            ))
        })
    });

    group.finish();
}

criterion_group!(
    benches,
    bench_packer,
    bench_unpacker,
    bench_gemv,
    bench_fused_pack_gemv,
    bench_ramsey_stack_vs_heap,
    bench_szemeredi_rayon_bitset,
    bench_moser_tardos_luby_mis,
    bench_xor_fold_gaia_b158_1mib,
    bench_ternary_simd_kernels
);
criterion_main!(benches);
