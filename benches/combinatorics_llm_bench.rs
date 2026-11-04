use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion};
use st::mcp_combinatorics_llm::{
    kernels::{
        bron_kerbosch_degeneracy_heap, bron_kerbosch_degeneracy_root, run_moser_tardos_splitting,
        run_szemeredi_compression, BitMatrix, StackBitMatrix,
    },
    model_cache::McpProgressEmitter,
    MoserTardosParams, SzemerediParams,
};
use st::mem8::regularity::xor_fold_gaia_b158;
use std::hint::black_box;

fn generate_deterministic_edges(n: usize, density_permille: u64) -> Vec<(usize, usize)> {
    let mut edges = Vec::new();
    let mut state = 0x9E37_79B9_7F4A_7C15u64 ^ (n as u64);
    for u in 0..n {
        for v in (u + 1)..n {
            state = state.wrapping_add(0x9E37_79B9_7F4A_7C15);
            let mut z = state;
            z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
            z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
            z ^= z >> 31;
            if (z % 1000) < density_permille {
                edges.push((u, v));
            }
        }
    }
    edges
}

fn bench_ramsey_stack_vs_heap(c: &mut Criterion) {
    let mut group = c.benchmark_group("ramsey_bron_kerbosch_200v");
    let n = 200;
    let edges = generate_deterministic_edges(n, 250);

    let mut stack_adj = StackBitMatrix::<4>::new(n);
    let mut heap_adj = BitMatrix::new(n);
    for &(u, v) in &edges {
        stack_adj.add_undirected_edge(u, v);
        heap_adj.add_undirected_edge(u, v);
    }

    group.bench_function(BenchmarkId::new("StackBitSet_u64x4_ZeroAlloc", n), |b| {
        b.iter(|| {
            black_box(bron_kerbosch_degeneracy_root::<4>(
                black_box(&stack_adj),
                black_box(12),
            ))
        })
    });

    group.bench_function(BenchmarkId::new("HeapBitSet_Vec_Fallback", n), |b| {
        b.iter(|| {
            black_box(bron_kerbosch_degeneracy_heap(
                black_box(&heap_adj),
                black_box(12),
            ))
        })
    });

    group.finish();
}

fn bench_szemeredi_rayon_bitset(c: &mut Criterion) {
    let mut group = c.benchmark_group("szemeredi_regularity_rayon");
    let emitter = McpProgressEmitter::default();

    for &n in &[128usize, 256usize] {
        let edges = generate_deterministic_edges(n, 400);
        group.bench_with_input(BenchmarkId::from_parameter(n), &edges, |b, edge_list| {
            b.iter(|| {
                let params = SzemerediParams {
                    num_vertices: n,
                    edges: edge_list.clone(),
                    epsilon: 0.12,
                    min_parts: 4,
                };
                black_box(run_szemeredi_compression(params, &emitter).unwrap())
            })
        });
    }
    group.finish();
}

fn bench_moser_tardos_luby_mis(c: &mut Criterion) {
    let mut group = c.benchmark_group("moser_tardos_luby_mis");
    let emitter = McpProgressEmitter::default();

    let num_vars = 256;
    let mut bad_events = Vec::with_capacity(180);
    for i in 0..180 {
        let v1 = ((i * 3) % num_vars + 1) as i32;
        let v2 = ((i * 7 + 11) % num_vars + 1) as i32;
        let v3 = ((i * 13 + 29) % num_vars + 1) as i32;
        let v4 = ((i * 19 + 43) % num_vars + 1) as i32;
        bad_events.push(vec![v1, -v2, v3, -v4]);
    }

    group.bench_function("4cnf_256vars_180clauses", |b| {
        b.iter(|| {
            let params = MoserTardosParams {
                num_variables: num_vars,
                bad_events: bad_events.clone(),
                max_resamples: 2000,
            };
            black_box(run_moser_tardos_splitting(params, &emitter).unwrap())
        })
    });

    group.finish();
}

fn bench_xor_fold_gaia_b158_1mib(c: &mut Criterion) {
    let mut group = c.benchmark_group("xor_fold_gaia_b158");
    let trits: Vec<i8> = (0usize..(1024 * 1024))
        .map(|i| ((i as i32 * 73 - 101) % 3 - 1) as i8)
        .collect();

    group.bench_function("1MiB_8lane_pack5", |b| {
        b.iter(|| black_box(xor_fold_gaia_b158(black_box(&trits))))
    });

    group.finish();
}

criterion_group!(
    benches,
    bench_ramsey_stack_vs_heap,
    bench_szemeredi_rayon_bitset,
    bench_moser_tardos_luby_mis,
    bench_xor_fold_gaia_b158_1mib
);
criterion_main!(benches);
