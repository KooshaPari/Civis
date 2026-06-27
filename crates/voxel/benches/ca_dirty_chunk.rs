use criterion::{black_box, criterion_group, criterion_main, BatchSize, Criterion};
use civ_voxel::fluid_ca::{step_with_config, CaGrid};
use civ_voxel::material::{MaterialRegistry, AIR, WATER};
use civ_voxel::BoundaryConfig;

fn dirty_chunk_fixture() -> (CaGrid, MaterialRegistry) {
    let mut grid = CaGrid::new([32, 16, 16]);
    // Two dirty chunks with a simple fluid surface so the CA has real work to do.
    for x in 0..4 {
        grid.set_with_temp(x, 8, 8, WATER, 25);
        grid.set_with_temp(x, 9, 8, AIR, 20);
    }
    for x in 16..20 {
        grid.set_with_temp(x, 8, 8, WATER, 25);
        grid.set_with_temp(x, 9, 8, AIR, 20);
    }
    (grid, MaterialRegistry::standard())
}

fn reference_grid_fixture() -> (CaGrid, MaterialRegistry) {
    let mut grid = CaGrid::new([64 * 16, 16, 64 * 16]);
    let mut chunk_index = 0usize;
    for cz in 0..64 {
        for cx in 0..64 {
            // Keep the reference workload at ~1% dirty chunks so the bench
            // mirrors the spec's 64×64 / 1% writes shape.
            if chunk_index % 100 != 0 {
                chunk_index += 1;
                continue;
            }
            let x = cx * 16 + 1;
            let z = cz * 16 + 1;
            grid.set_with_temp(x, 8, z, WATER, 25);
            grid.set_with_temp(x, 9, z, AIR, 20);
            chunk_index += 1;
        }
    }
    (grid, MaterialRegistry::standard())
}

fn bench_ca_dirty_chunk(c: &mut Criterion) {
    c.bench_function("ca_dirty_chunk::dirty_cell_indices", |b| {
        b.iter_batched(
            dirty_chunk_fixture,
            |(grid, _reg)| black_box(grid.dirty_cell_indices().len()),
            BatchSize::SmallInput,
        );
    });

    c.bench_function("ca_dirty_chunk::scratch_round_trip", |b| {
        b.iter_batched(
            dirty_chunk_fixture,
            |(mut grid, _reg)| {
                grid.refresh_scratch();
                let view = grid.scratch_view();
                grid.restore_scratch(view);
                black_box(grid.dirty_cell_indices().len())
            },
            BatchSize::SmallInput,
        );
    });

    c.bench_function("ca_dirty_chunk::step_with_config", |b| {
        b.iter_batched(
            dirty_chunk_fixture,
            |(mut grid, reg)| {
                let outcome = step_with_config(
                    black_box(&mut grid),
                    reg,
                    BoundaryConfig::closed(),
                    0,
                );
                black_box(outcome.changed_chunks.len())
            },
            BatchSize::SmallInput,
        );
    });

    c.bench_function("ca_dirty_chunk::phase_scan", |b| {
        b.iter_batched(
            dirty_chunk_fixture,
            |(grid, _reg)| black_box(grid.dirty_cell_indices().len()),
            BatchSize::SmallInput,
        );
    });

    c.bench_function("ca_dirty_chunk::phase_simulate", |b| {
        b.iter_batched(
            dirty_chunk_fixture,
            |(mut grid, reg)| {
                let outcome = step_with_config(
                    black_box(&mut grid),
                    reg,
                    BoundaryConfig::closed(),
                    0,
                );
                black_box((outcome.changed, outcome.changed_chunks.len()))
            },
            BatchSize::SmallInput,
        );
    });

    c.bench_function("ca_dirty_chunk::phase_dirty", |b| {
        b.iter_batched(
            dirty_chunk_fixture,
            |(mut grid, _reg)| {
                grid.refresh_scratch();
                let view = grid.scratch_view();
                grid.restore_scratch(view);
                black_box(grid.dirty_owned_cell_indices().len())
            },
            BatchSize::SmallInput,
        );
    });

    c.bench_function("ca_dirty_chunk::phase_propagate", |b| {
        b.iter_batched(
            dirty_chunk_fixture,
            |(mut grid, reg)| {
                let outcome = step_with_config(
                    black_box(&mut grid),
                    reg,
                    BoundaryConfig::closed(),
                    0,
                );
                black_box(outcome.changed_chunks.len())
            },
            BatchSize::SmallInput,
        );
    });

    c.bench_function("ca_dirty_chunk::determinism_probe", |b| {
        b.iter_batched(
            dirty_chunk_fixture,
            |(mut grid_a, reg)| {
                let mut grid_b = grid_a.clone();
                let out_a = step_with_config(black_box(&mut grid_a), reg, BoundaryConfig::closed(), 0);
                let out_b = step_with_config(
                    black_box(&mut grid_b),
                    MaterialRegistry::standard(),
                    BoundaryConfig::closed(),
                    0,
                );
                assert_eq!(out_a.changed_chunks, out_b.changed_chunks);
                assert_eq!(grid_a.cells, grid_b.cells);
                assert_eq!(grid_a.temperatures, grid_b.temperatures);
                assert_eq!(grid_a.saturation, grid_b.saturation);
                black_box(out_a.changed_chunks.len())
            },
            BatchSize::SmallInput,
        );
    });

    c.bench_function("ca_dirty_chunk::reference_grid_step_with_config", |b| {
        b.iter_batched(
            reference_grid_fixture,
            |(mut grid, reg)| {
                let outcome = step_with_config(
                    black_box(&mut grid),
                    reg,
                    BoundaryConfig::closed(),
                    0,
                );
                black_box((outcome.changed, outcome.changed_chunks.len()))
            },
            BatchSize::SmallInput,
        );
    });
}

criterion_group!(benches, bench_ca_dirty_chunk);
criterion_main!(benches);
