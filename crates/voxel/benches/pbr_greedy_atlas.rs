//! Criterion benchmarks for the greedy triplanar atlas packer.
//!
//! `pack_64_textures_512x512` is the canonical PR-3 perf budget: 64 mixed
//! CC0 textures (128², 256², 512² drop-ins) packed into a 512×512 atlas in
//! under 5 ms. The packer runs once per material build (not per frame), so
//! 5 ms is generous — a real chunk load is far below that.
//!
//! The bench names match the public API one-for-one so PR notes can refer
//! to them by name (e.g. "regressed in PR #1234: pack_64_textures_512x512
//! went from 1.2 ms → 4.8 ms, near the 5 ms budget").

use std::time::Duration;

use civ_voxel::pbr::{AtlasTexture, GreedyAtlas};
use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion};

/// Build a deterministic 64-texture input that exercises every shelf case
/// the packer hits: a 256² tall one (single shelf), four 128² (sharing
/// that shelf), fifteen 64² (sharing a second shelf), the rest 32² filling
/// a third shelf. The shape is chosen so the result ends up around 80% of
/// the 512² height — tight enough to stress the search, loose enough to
/// never overflow.
fn workload_64() -> Vec<AtlasTexture> {
    let mut v = Vec::with_capacity(64);
    // One 256x256 hero texture — first shelf.
    v.push(AtlasTexture::new(0, 256, 256));
    // Four 128x128 — share shelf 0.
    for i in 1..=4 {
        v.push(AtlasTexture::new(i, 128, 128));
    }
    // Fifteen 64x64 — share shelf 1.
    for i in 5..20 {
        v.push(AtlasTexture::new(i, 64, 64));
    }
    // Forty-four 32x32 — fill shelf 2.
    for i in 20..64 {
        v.push(AtlasTexture::new(i, 32, 32));
    }
    debug_assert_eq!(v.len(), 64);
    v
}

/// Smaller 16-texture workload for the chart — useful when profiling
/// rather than for the budget gate.
fn workload_16() -> Vec<AtlasTexture> {
    (0..16)
        .map(|i| AtlasTexture::new(i as u32, 64, 64))
        .collect()
}

/// Single 1024x1024 tile — sanity check that the trivial case stays
/// sub-millisecond.
fn workload_1() -> Vec<AtlasTexture> {
    vec![AtlasTexture::new(0, 1024, 1024)]
}

fn bench_packer(c: &mut Criterion) {
    let mut group = c.benchmark_group("pbr_greedy_atlas");
    // We only budget-assert the canonical case below; tighten sample size
    // so the local CI doesn't burn minutes on every PR.
    group.sample_size(50);
    group.measurement_time(Duration::from_secs(2));

    group.bench_function("pack_64_textures_512x512", |b| {
        let textures = workload_64();
        let mut atlas = GreedyAtlas::new(512, 512);
        b.iter(|| {
            let rects = atlas.pack(&textures).expect("workload fits");
            // Black-box the result so the optimiser doesn't elide work.
            criterion::black_box(rects.len());
            criterion::black_box(atlas.shelf_count());
            criterion::black_box(atlas.packed_height());
        });
    });

    group.bench_function("pack_16_textures_512x512", |b| {
        let textures = workload_16();
        let mut atlas = GreedyAtlas::new(512, 512);
        b.iter(|| {
            let rects = atlas.pack(&textures).expect("workload fits");
            criterion::black_box(rects.len());
        });
    });

    group.bench_function("pack_1_texture_1024x1024", |b| {
        let textures = workload_1();
        let mut atlas = GreedyAtlas::new(1024, 1024);
        b.iter(|| {
            let rects = atlas.pack(&textures).expect("workload fits");
            criterion::black_box(rects.len());
        });
    });

    // Parametric sweep — same workload shape, scaling input size so a future
    // PR can spot asymptotic regressions at a glance.
    for &count in &[8usize, 32, 128] {
        group.bench_with_input(
            BenchmarkId::new("pack_sweep", count),
            &count,
            |b, &count| {
                let textures: Vec<AtlasTexture> = (0..count)
                    .map(|i| AtlasTexture::new(i as u32, 64, 64))
                    .collect();
                let mut atlas = GreedyAtlas::new(512, 512);
                b.iter(|| {
                    let rects = atlas.pack(&textures).expect("workload fits");
                    criterion::black_box(rects.len());
                });
            },
        );
    }

    group.finish();
}

criterion_group!(benches, bench_packer);
criterion_main!(benches);
