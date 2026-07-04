use criterion::{black_box, criterion_group, criterion_main, BatchSize, Criterion};
use civ_agents::{spawn_many, tick_movement};
use civ_economy::{step as economy_step, EconomyState, InstitutionLedger};
use civ_voxel::fluid_ca::{step_with_config, CaGrid};
use civ_voxel::{material::AIR, material::WATER, BoundaryConfig, MaterialRegistry};
use rand::SeedableRng;
use rand_chacha::ChaCha8Rng;

fn ca_fixture() -> (CaGrid, MaterialRegistry) {
    let mut grid = CaGrid::new([8 * 16, 16, 8 * 16]);
    for cz in 0..8 {
        for cx in 0..8 {
            let x = cx * 16 + 1;
            let z = cz * 16 + 1;
            grid.set_with_temp(x, 8, z, WATER, 25);
            grid.set_with_temp(x, 9, z, AIR, 20);
        }
    }
    (grid, MaterialRegistry::standard())
}

fn bench_ca_step(c: &mut Criterion) {
    c.bench_function("ca_step_64_dirty", |b| {
        b.iter_batched(
            ca_fixture,
            |(mut grid, registry)| {
                let outcome = step_with_config(
                    black_box(&mut grid),
                    registry,
                    BoundaryConfig::closed(),
                    0,
                );
                black_box((outcome.changed, outcome.changed_chunks.len()))
            },
            BatchSize::SmallInput,
        );
    });
}

fn bench_agent_update(c: &mut Criterion) {
    c.bench_function("agent_update_64_civilians", |b| {
        b.iter_batched(
            || {
                let mut world = hecs::World::new();
                spawn_many(&mut world, 64, 1, 1);
                let rng = ChaCha8Rng::seed_from_u64(7);
                (world, rng)
            },
            |(mut world, mut rng)| {
                tick_movement(&mut world, 128, &mut rng, |_, _| true);
                black_box(world.query::<&civ_agents::Position3d>().iter().count())
            },
            BatchSize::SmallInput,
        );
    });
}

fn bench_economy_tick(c: &mut Criterion) {
    c.bench_function("economy_tick", |b| {
        b.iter_batched(
            || {
                let mut state = EconomyState::with_energy_budget(100_000);
                state.institutions = InstitutionLedger::with_defaults();
                state
            },
            |mut state| {
                economy_step(&mut state);
                black_box((state.tick, state.energy_budget_joules, state.ledger.len()))
            },
            BatchSize::SmallInput,
        );
    });
}

criterion_group!(benches, bench_ca_step, bench_agent_update, bench_economy_tick);
criterion_main!(benches);
