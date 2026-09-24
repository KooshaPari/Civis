//! NFR-P-02 — p99 tick time at 1k citizens (always-on integration gate).
//!
//! The criterion bench in `benches/tick_bench.rs` is the precise release
//! measurement, but that target is `harness = false`, so any `#[test]` inside
//! it is compiled out and never runs under `cargo test`. This integration
//! test carries the same scenario — 1k spawned citizens, 100 sampled ticks,
//! nearest-rank p99 — with a generous debug-build ceiling so gross tick-loop
//! regressions fail CI while the stored criterion baseline remains the
//! exact 14 ms release gate.

use civ_agents::{count_civilians, spawn_civilian_at, ActorVisualKind, Alignment};
use civ_engine::Simulation;
use std::time::Instant;

// NFR-P-02 — p99 single-tick time at 1k citizens stays under a generous
// debug ceiling (the criterion baseline in benches/tick_bench.rs is the
// precise < 14 ms release budget).
#[test]
fn nfr_p_02_p99_tick_time_at_1k_citizens_within_generous_ceiling() {
    let mut sim = Simulation::with_seed(42);
    let mut rng = sim.rng_mut().clone();
    for i in 0..1000u64 {
        let x = (i % 100) as f32 / 100.0 * 0.8 + 0.1;
        let z = (i / 100) as f32 / 10.0 * 0.8 + 0.1;
        let _ = spawn_civilian_at(
            &mut sim.world,
            60_000 + i,
            Alignment::Faction(0),
            x,
            z,
            ActorVisualKind::Humanoid,
            &mut rng,
        );
    }
    *sim.rng_mut() = rng;
    let citizens = count_civilians(&sim.world);
    assert!(
        citizens >= 1000,
        "scenario must hold at least 1k citizens, got {citizens}"
    );

    // Warm caches, then sample 100 single-tick durations.
    for _ in 0..10 {
        sim.tick();
    }
    let mut samples = Vec::with_capacity(100);
    for _ in 0..100 {
        let start = Instant::now();
        sim.tick();
        samples.push(start.elapsed());
    }
    samples.sort();
    let p99 = samples[98]; // nearest-rank p99 over 100 samples
    assert!(
        p99.as_millis() < 5_000,
        "NFR-P-02 p99 at 1k citizens was {p99:?} (14 ms release budget; 5 s debug regression ceiling)"
    );
    assert_eq!(sim.state.tick, 110, "warmup + 100 sampled ticks completed");
}
