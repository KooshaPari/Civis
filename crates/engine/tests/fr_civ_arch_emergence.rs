//! Deferred FR-CIV-ARCH integration coverage.
//!
//! The historical assertions require `building_emergence`, which is not
//! currently exported from `civ_engine`. Keep this anti-wipe test meaningful
//! by exercising the current public building-emergence hook and simulation
//! invariants until that API returns.

use civ_engine::Simulation;

#[test]
fn building_emergence_hook_preserves_seeded_building_invariants() {
    let mut sim = Simulation::with_seed(12);
    let before = sim.snapshot();

    sim.run_building_emergence_tick();

    let after_emergence = sim.snapshot();
    assert!(
        after_emergence.building_count >= before.building_count,
        "building emergence must not remove seeded ECS buildings"
    );

    sim.tick();

    let after = sim.snapshot();
    assert_eq!(after.tick, before.tick + 1);
    assert!(
        after.building_count >= after_emergence.building_count,
        "a normal tick must preserve buildings added by emergence"
    );
    assert!(
        after.building_count > 0,
        "seeded simulations contain buildings"
    );
}
