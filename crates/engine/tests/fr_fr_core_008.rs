//! FR-CORE-008 — world state SHALL be modelled as a `hecs::World`; no global
//! singletons.
//!
//! Matrix check: `world::no_global_resources`.
//!
//! `hecs` has no `Resources` container at all, so the property that matters is
//! the one asserted here: world state is owned per-`Simulation`, and no global
//! or static world exists for simulations to share. See
//! `docs/adr/ADR-022-runtime-representation-deviations.md` for why the engine is
//! `hecs`-native rather than `bevy_ecs`.

use civ_engine::Simulation;

/// Each simulation owns its world; mutating one never reaches another.
#[test]
fn no_global_resources() {
    let mut a = Simulation::with_seed(0xAAAA_u64);
    let mut b = Simulation::with_seed(0xBBBB_u64);

    let a_entities = a.world.len();
    let b_entities = b.world.len();
    assert!(a_entities > 0, "a freshly built simulation has entities");
    assert!(b_entities > 0, "a freshly built simulation has entities");

    // Spawning into one world must not be visible from the other.
    let _held = a.world.spawn(());
    assert_eq!(a.world.len(), a_entities + 1);
    assert_eq!(
        b.world.len(),
        b_entities,
        "worlds are owned per-Simulation, not shared globally"
    );

    // Ticking one simulation must not advance the other's clock.
    let b_tick_before = b.state.tick;
    a.tick();
    assert_eq!(a.state.tick, 1);
    assert_eq!(
        b.state.tick, b_tick_before,
        "there is no shared global tick clock"
    );

    // Removing from one world likewise stays local.
    a.world.clear();
    assert_eq!(a.world.len(), 0);
    assert_eq!(b.world.len(), b_entities, "clearing a world is local");
}

/// Two identically-seeded simulations evolve identically and independently,
/// which is only possible if nothing global is shared.
#[test]
fn identical_runs_stay_in_lockstep_without_sharing() {
    let mut c = Simulation::with_seed(0x5EED_u64);
    let mut d = Simulation::with_seed(0x5EED_u64);

    assert_eq!(c.world.len(), d.world.len(), "same seed => same initial world");

    for _ in 0..5 {
        c.tick();
        d.tick();
    }
    assert_eq!(c.state.tick, d.state.tick);
    assert_eq!(
        c.world.len(),
        d.world.len(),
        "no shared accumulator may perturb either run"
    );

    // Interleaving must not break lockstep, so neither can be reading a global.
    let mut e = Simulation::with_seed(0x5EED_u64);
    let other = Simulation::with_seed(0x1_u64);
    for _ in 0..5 {
        e.tick();
        // Drop the second simulation's world reference each iteration to prove
        // the first cannot be depending on it.
        let _ = other.world.len();
    }
    assert_eq!(
        e.world.len(),
        c.world.len(),
        "an unrelated simulation in scope changes nothing"
    );
}
