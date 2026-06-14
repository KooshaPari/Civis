//! FR-CORE-004 — Seeded stochastic phase with RNG draw replay logging.
//!
//! Covers the acceptance criteria from `FUNCTIONAL_REQUIREMENTS.md`:
//!   * RNG draw events are recorded in the replay log
//!   * Same seed → identical stochastic event sequences

use civ_engine::{ReplayEvent, Simulation};

/// FR-CORE-004 — a recorded boolean RNG draw emits a `RngDraw` event.
#[test]
fn fr_core_004_rng_draw_event_recorded() {
    let mut sim = Simulation::with_seed(42);
    let result = sim.draw_rng_bool(0.5);
    let draws: Vec<_> = sim
        .replay_log()
        .events
        .iter()
        .filter(|e| matches!(e, ReplayEvent::RngDraw { .. }))
        .collect();
    assert_eq!(draws.len(), 1, "exactly one RngDraw event recorded");
    if let ReplayEvent::RngDraw { tick, probability, result: r } = draws[0] {
        assert_eq!(*tick, 0, "draw recorded at tick 0");
        assert_eq!(*probability, 0.5, "probability preserved");
        assert_eq!(*r, result, "result matches returned value");
    } else {
        panic!("expected RngDraw variant");
    }
}

/// FR-CORE-004 — property test: same seed + same tick count → identical
/// stochastic event sequences in the replay log.
#[test]
fn fr_core_004_same_seed_same_stochastic_sequence() {
    let mut a = Simulation::with_seed(2024);
    let mut b = Simulation::with_seed(2024);

    // Run enough ticks to hit both birth_window (every 200) and diplomacy (every 500).
    for _ in 0..600 {
        a.tick();
        b.tick();
    }

    let draws_a: Vec<_> = a
        .replay_log()
        .events
        .iter()
        .filter(|e| matches!(e, ReplayEvent::RngDraw { .. }))
        .cloned()
        .collect();
    let draws_b: Vec<_> = b
        .replay_log()
        .events
        .iter()
        .filter(|e| matches!(e, ReplayEvent::RngDraw { .. }))
        .cloned()
        .collect();

    assert!(
        !draws_a.is_empty(),
        "at least one RngDraw should be recorded after 600 ticks"
    );
    assert_eq!(
        draws_a, draws_b,
        "same seed must produce identical rng_draw sequences"
    );
}
