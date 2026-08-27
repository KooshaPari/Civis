//! Criterion benchmarks for the Civis simulation engine tick loop and
//! voxel mesh generation.
//!
//! These benchmarks catch performance regressions on every PR by running
//! `cargo bench --workspace -- --test` in CI.

use civ_engine::Simulation;
use civ_voxel::{
    ChunkId, ChunkView, CubicMesher, LodLevel, MaterialId, VoxelWorld, WorldCoord, FIXED_SCALE,
};
use criterion::{black_box, criterion_group, criterion_main, BatchSize, Criterion};

// ---------------------------------------------------------------------------
// Tick loop benchmarks
// ---------------------------------------------------------------------------

/// Create a seeded simulation ready for benchmarking.
fn tick_sim_fixture() -> Simulation {
    Simulation::with_seed(42)
}

/// Benchmark a single `Simulation::tick()` call — the core hot path.
fn bench_tick_single(c: &mut Criterion) {
    c.bench_function("tick_loop::single_tick", |b| {
        b.iter_batched(
            tick_sim_fixture,
            |mut sim| {
                sim.tick();
                black_box(sim.state.tick)
            },
            BatchSize::SmallInput,
        );
    });
}

/// Benchmark 100 consecutive ticks — catches amortised overhead and cache
/// effects across repeated phase executions.
fn bench_tick_100(c: &mut Criterion) {
    c.bench_function("tick_loop::100_ticks", |b| {
        b.iter_batched(
            tick_sim_fixture,
            |mut sim| {
                for _ in 0..100 {
                    sim.tick();
                }
                black_box(sim.state.tick)
            },
            BatchSize::SmallInput,
        );
    });
}

/// Benchmark tick with a populated voxel substrate — exercises the
/// `phase_voxel` dirty-event drain path.
fn bench_tick_with_voxels(c: &mut Criterion) {
    c.bench_function("tick_loop::tick_with_voxels", |b| {
        b.iter_batched(
            || {
                let mut sim = tick_sim_fixture();
                // Seed a few voxel writes so phase_voxel has dirty events to drain.
                let scale = FIXED_SCALE;
                for x in 0..4 {
                    sim.push_voxel_write(
                        WorldCoord {
                            x: i64::from(x) * scale,
                            y: 0,
                            z: 0,
                        },
                        MaterialId(1),
                    );
                }
                sim
            },
            |mut sim| {
                sim.tick();
                black_box(sim.state.tick)
            },
            BatchSize::SmallInput,
        );
    });
}

// ---------------------------------------------------------------------------
// Voxel mesh generation benchmarks
// ---------------------------------------------------------------------------

/// Build a small 3×3×3 block inside a 16³ chunk for mesh benchmarking.
fn small_block_voxels() -> Vec<MaterialId> {
    let mut v = vec![MaterialId(0); 16 * 16 * 16];
    for ix in 0..3 {
        for iy in 0..3 {
            for iz in 0..3 {
                v[ix + iy * 16 + iz * 16 * 16] = MaterialId(1);
            }
        }
    }
    v
}

/// Build a densely-filled 16³ chunk for mesh benchmarking.
fn dense_chunk_voxels() -> Vec<MaterialId> {
    vec![MaterialId(1); 16 * 16 * 16]
}

/// Build a sparse 16³ chunk (~2% fill) for mesh benchmarking.
fn sparse_chunk_voxels() -> Vec<MaterialId> {
    let mut v = vec![MaterialId(0); 16 * 16 * 16];
    // Place voxels at 4-cell intervals along one diagonal + a few scattered.
    for i in (0..16).step_by(4) {
        v[i + i * 16 + i * 16 * 16] = MaterialId(1);
    }
    v[1 + 5 * 16 + 10 * 16 * 16] = MaterialId(2);
    v[14 + 3 * 16 + 7 * 16 * 16] = MaterialId(3);
    v
}

fn bench_mesh_small_block(c: &mut Criterion) {
    c.bench_function("voxel_mesh::small_block_3x3x3", |b| {
        let voxels = small_block_voxels();
        let view = ChunkView {
            id: ChunkId(0),
            voxels: &voxels,
        };
        b.iter(|| {
            let mesh = CubicMesher::mesh_cubic(black_box(view), LodLevel(0)).expect("mesh");
            black_box((mesh.vertices.len(), mesh.indices.len()))
        });
    });
}

fn bench_mesh_dense_chunk(c: &mut Criterion) {
    c.bench_function("voxel_mesh::dense_chunk_16x16x16", |b| {
        let voxels = dense_chunk_voxels();
        let view = ChunkView {
            id: ChunkId(0),
            voxels: &voxels,
        };
        b.iter(|| {
            let mesh = CubicMesher::mesh_cubic(black_box(view), LodLevel(0)).expect("mesh");
            black_box((mesh.vertices.len(), mesh.indices.len()))
        });
    });
}

fn bench_mesh_sparse_chunk(c: &mut Criterion) {
    c.bench_function("voxel_mesh::sparse_chunk_2pct_fill", |b| {
        let voxels = sparse_chunk_voxels();
        let view = ChunkView {
            id: ChunkId(0),
            voxels: &voxels,
        };
        b.iter(|| {
            let mesh = CubicMesher::mesh_cubic(black_box(view), LodLevel(0)).expect("mesh");
            black_box((mesh.vertices.len(), mesh.indices.len()))
        });
    });
}

/// End-to-end: write voxels into a `VoxelWorld`, then mesh every dense
/// chunk using the `chunks_dense()` iterator. This catches per-chunk
/// overhead the isolated mesh benchmarks miss.
fn bench_mesh_from_world(c: &mut Criterion) {
    c.bench_function("voxel_mesh::world_to_mesh_4_chunks", |b| {
        b.iter_batched(
            || {
                let mut w: VoxelWorld<MaterialId> = VoxelWorld::new(FIXED_SCALE);
                let scale = FIXED_SCALE;
                // Write a 4×4×4 block in each of 4 chunks along X.
                for chunk_i in 0..4 {
                    let bx = chunk_i * 16;
                    for dx in 0..4 {
                        for dy in 0..4 {
                            for dz in 0..4 {
                                w.write(
                                    WorldCoord {
                                        x: i64::from(bx + dx) * scale,
                                        y: i64::from(dy) * scale,
                                        z: i64::from(dz) * scale,
                                    },
                                    MaterialId(1),
                                );
                            }
                        }
                    }
                }
                w
            },
            |w| {
                let mut total_verts = 0usize;
                let mut total_inds = 0usize;
                for (_coord, chunk) in w.chunks_dense() {
                    let view = ChunkView {
                        id: ChunkId(0),
                        voxels: &chunk.voxels,
                    };
                    let mesh = CubicMesher::mesh_cubic(view, LodLevel(0)).expect("mesh");
                    total_verts += mesh.vertices.len();
                    total_inds += mesh.indices.len();
                }
                black_box((total_verts, total_inds))
            },
            BatchSize::SmallInput,
        );
    });
}

criterion_group!(
    benches,
    bench_tick_single,
    bench_tick_100,
    bench_tick_with_voxels,
    bench_mesh_small_block,
    bench_mesh_dense_chunk,
    bench_mesh_sparse_chunk,
    bench_mesh_from_world,
);
criterion_main!(benches);
