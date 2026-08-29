//! Criterion benchmarks for the Civis simulation engine **tick loop**.
//!
//! These benchmarks measure end-to-end `Simulation::tick()` throughput at
//! three granularities so PR regressions show up quickly:
//!
//! - [`bench_single_tick`]: one tick — isolates per-call overhead.
//! - [`bench_100_ticks`]: amortised loop overhead across a short horizon.
//! - [`bench_1000_ticks`]: long-horizon throughput (ticks/sec), used as
//!   the primary metric for the tick-loop optimisation series.
//!
//! All benchmarks use `iter_batched` with a fresh seeded
//! [`Simulation`] per iteration so cached state from one run never leaks
//! into the next.

use civ_engine::Simulation;
use criterion::{black_box, criterion_group, criterion_main, BatchSize, Criterion, Throughput};

// ---------------------------------------------------------------------------
// Shared fixture
// ---------------------------------------------------------------------------

/// Create a deterministic seeded simulation ready for benchmarking.
#[inline]
fn tick_sim_fixture() -> Simulation {
    Simulation::with_seed(42)
}

// ---------------------------------------------------------------------------
// Single-tick benchmark
// ---------------------------------------------------------------------------

/// Benchmark one `Simulation::tick()` call — the core hot path.
fn bench_single_tick(c: &mut Criterion) {
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

// ---------------------------------------------------------------------------
// 100-tick benchmark
// ---------------------------------------------------------------------------

/// Benchmark 100 consecutive ticks — catches amortised overhead and
/// cache effects across repeated phase executions.
fn bench_100_ticks(c: &mut Criterion) {
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

// ---------------------------------------------------------------------------
// 1000-tick throughput benchmark
// ---------------------------------------------------------------------------

/// Benchmark 1000 consecutive ticks — primary throughput metric for the
/// tick-loop optimisation series (FR-CIV-PERF-tick).
///
/// Reports **ticks/second** via Criterion's `throughput(Throughput::Elements(1000))`
/// so regressions show up as drops in the per-iteration rate even when
/// wall-clock noise is present. One full sample is large enough to amortise
/// spawn cost (one-shot in `with_seed`) while staying well under Criterion's
/// default measurement window.
fn bench_1000_ticks(c: &mut Criterion) {
    let mut group = c.benchmark_group("tick_loop");
    group.throughput(Throughput::Elements(1000));
    group.bench_function("1000_ticks", |b| {
        b.iter_batched(
            tick_sim_fixture,
            |mut sim| {
                for _ in 0..1000 {
                    sim.tick();
                }
                black_box(sim.state.tick)
            },
            BatchSize::SmallInput,
        );
    });
    group.finish();
}

criterion_group!(
    benches,
    bench_single_tick,
    bench_100_ticks,
    bench_1000_ticks,
);
criterion_main!(benches);