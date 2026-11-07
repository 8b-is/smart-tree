//! A timing check, not a criterion bench: proves the vector path is actually
//! faster on this machine rather than merely correct. Run with
//! `cargo test --release --test ternary_deck_speed -- --nocapture`.
use st::mcp_combinatorics_llm::ternary_deck::{
    compute_ternary_sse_scalar, compute_ternary_sse_simd, pack_4trits_row_scalar,
    pack_4trits_row_simd, pack_5trits_row_scalar, pack_5trits_row_simd,
};
use std::hint::black_box;
use std::time::Instant;

#[test]
fn the_vector_paths_are_faster_than_the_scalar_ones() {
    let width = 3840usize;
    let row: Vec<u8> = (0..width).map(|i| (i % 3) as u8).collect();
    let mut out4 = vec![0u8; width / 4];
    let mut out5 = vec![0u8; width / 5];
    let iterations = if cfg!(debug_assertions) { 200 } else { 20_000 };

    let time = |f: &mut dyn FnMut()| {
        let start = Instant::now();
        for _ in 0..iterations {
            f();
        }
        start.elapsed()
    };

    let scalar4 = time(&mut || {
        black_box(pack_4trits_row_scalar(black_box(&row), &mut out4));
    });
    let vector4 = time(&mut || {
        black_box(pack_4trits_row_simd(black_box(&row), &mut out4));
    });
    let scalar5 = time(&mut || {
        black_box(pack_5trits_row_scalar(black_box(&row), &mut out5));
    });
    let vector5 = time(&mut || {
        black_box(pack_5trits_row_simd(black_box(&row), &mut out5));
    });

    let pixels: Vec<u8> = (0..(3840 * 64)).map(|i| (i * 37 % 256) as u8).collect();
    let scalar_sse = time(&mut || {
        black_box(compute_ternary_sse_scalar(
            black_box(&pixels),
            (64, 180),
            [16, 122, 235],
        ));
    });
    let vector_sse = time(&mut || {
        black_box(compute_ternary_sse_simd(
            black_box(&pixels),
            (64, 180),
            [16, 122, 235],
        ));
    });

    println!(
        "\n  4-trit row (3840):  scalar {:?}  vector {:?}",
        scalar4, vector4
    );
    println!(
        "  5-trit row (3840):  scalar {:?}  vector {:?}",
        scalar5, vector5
    );
    println!(
        "  sse (245,760 px):   scalar {:?}  vector {:?}",
        scalar_sse, vector_sse
    );

    if cfg!(debug_assertions) {
        println!("  [NOTE] Running in unoptimized debug mode. Run with --release for true SIMD benchmark speed.");
    } else {
        assert!(vector4 < scalar4, "4-trit vector path is not faster");
        assert!(vector5 < scalar5, "5-trit vector path is not faster");
        assert!(vector_sse < scalar_sse, "sse vector path is not faster");
    }
}
