//! FR-CORE-001 (partial) — headless tick throughput budget.
//!
//! Default `cargo test` skips this; run in release when measuring performance.

#[cfg(not(debug_assertions))]
use std::time::Duration;
use std::time::Instant;

use civ_engine::Simulation;

const TICK_COUNT: usize = 10_000;

#[cfg(not(debug_assertions))]
const BUDGET: Duration = Duration::from_secs(2);

/// FR-CORE-001 — 10k `Simulation::tick()` calls stay under 2s in release builds.
#[test]
#[ignore = "benchmark: run with --ignored --release"]
fn ten_thousand_ticks_under_budget() {
    let mut sim = Simulation::with_seed(42);
    let start = Instant::now();
    for _ in 0..TICK_COUNT {
        sim.tick();
    }
    let elapsed = start.elapsed();

    assert_eq!(sim.state.tick, TICK_COUNT as u64);

    #[cfg(not(debug_assertions))]
    assert!(
        elapsed < BUDGET,
        "10k ticks took {elapsed:?}, budget is {BUDGET:?} (release only)"
    );

    #[cfg(debug_assertions)]
    eprintln!(
        "tick_budget (debug): {TICK_COUNT} ticks in {elapsed:?} (budget enforced in release)"
    );
}
