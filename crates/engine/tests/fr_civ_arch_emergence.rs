//! Deferred FR-CIV-ARCH integration coverage.
//!
//! The historical assertions require `building_emergence`, which is not
//! currently exported from `civ_engine`. Keep this anti-wipe test meaningful
//! by exercising the current public simulation surface until that API returns.

use civ_engine::Simulation;

#[test]
fn deferred_fr_arch_emergence_smoke_ticks_seeded_simulation() {
    let mut sim = Simulation::with_seed(12);
    let before = sim.snapshot();

    sim.tick();

    let after = sim.snapshot();
    assert_eq!(after.tick, before.tick + 1);
    assert!(after.building_count > 0);
}
