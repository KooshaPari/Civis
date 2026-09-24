//! NFR-P-05 — p50 tick time at 100k citizens (nightly-scale performance smoke).

use civ_agents::{count_civilians, spawn_many};
use civ_engine::Simulation;
use std::time::{Duration, Instant};

// NFR-P-05 — build a genuine 100k-citizen world, tick it, and assert the p50
// tick time stays under a generous ceiling. This is the always-on guard behind
// the nightly-only `bench_tick_100k_citizens` criterion harness referenced by
// `crates/engine/benches/tick_bench.rs`: it catches gross tick-loop scaling
// regressions long before the stored criterion baseline comparison runs.
#[test]
#[ignore = "NFR-P-05: 100k-citizen p50 smoke is nightly-only (matches the bench policy)"]
fn nfr_p_05_p50_tick_at_100k_citizens_stays_under_ceiling() {
    let mut sim = Simulation::with_seed(42);
    spawn_many(&mut sim.world, 100_000, 10_000_000, 1);
    let citizens = count_civilians(&sim.world);
    assert!(
        citizens >= 100_000,
        "world must hold at least 100k citizens, has {citizens}"
    );

    let mut samples = Vec::new();
    for _ in 0..5 {
        let start = Instant::now();
        sim.tick();
        samples.push(start.elapsed());
    }
    assert_eq!(sim.state.tick, 5, "five ticks must advance the clock");
    samples.sort();
    let p50 = samples[samples.len() / 2];
    assert!(
        p50 < Duration::from_secs(30),
        "p50 tick at 100k citizens was {p50:?}; gross tick-loop scaling regression (NFR-P-05)"
    );
}
