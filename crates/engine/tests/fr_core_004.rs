//! FR-CORE-004 — Seeded stochastic RNG-draw replay logging.
//!
//! STATUS: SUPERSEDED. The original acceptance criteria required a seeded,
//! bit-deterministic RNG with per-draw `ReplayEvent::RngDraw` logging
//! (`Simulation::draw_rng_bool`) so that `same seed -> identical event
//! sequence`. That determinism mandate was intentionally dropped (real,
//! non-seeded randomness is now welcome), and the `draw_rng_bool` method +
//! `ReplayEvent::RngDraw` variant were removed accordingly.
//!
//! These tests previously asserted the removed behaviour and therefore could
//! not compile, which blocked the entire `civ-engine` test target from
//! building. They are retired here pending a product decision on whether to
//! reinstate RNG-draw *logging* (compatible with non-deterministic RNG) atop
//! the surviving `ReplayLog`. Reinstating logging-only would restore the first
//! acceptance criterion without the dropped determinism guarantee.
//!
//! Tracking: FR-CORE-004 (superseded by the no-determinism decision). When a
//! replacement logging API lands, re-add a test that asserts a draw is recorded
//! in `Simulation::replay_log()` without asserting seed-identical sequences.

/// Minimal sentinel so the re-enabled test target keeps compiling while
/// FR-CORE-004's logging API is re-decided.
#[test]
fn fr_core_004_superseded_by_no_determinism_decision() {
    // No assertion: the determinism-replay contract this file covered was
    // retired. See module docs for the reinstatement path.
}

/// FR-CORE-004 — Each tick SHALL complete within 100 ms wall-clock.
/// Runs 100 ticks and asserts total wall-clock time < 10 s
/// (generous 10x headroom over the 100 ms/tick requirement).
#[test]
fn tick_perf_100_ticks_under_10s() {
    use std::time::Instant;

    let mut sim = civ_engine::Simulation::with_seed(2024);
    let start = Instant::now();
    for _ in 0..100 {
        sim.tick();
    }
    let elapsed = start.elapsed();
    assert!(
        elapsed.as_secs_f64() < 10.0,
        "100 ticks took {:.2}s, expected < 10.0s",
        elapsed.as_secs_f64()
    );
}
