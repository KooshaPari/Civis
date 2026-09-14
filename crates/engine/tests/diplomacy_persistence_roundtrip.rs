//! Regression coverage for the diplomacy persistence lane.
//!
//! Closes the parent-scorecard gap where stance_engine, deep_diplomacy,
//! faction_relations, and grief_accumulator lived only on Simulation,
//! not on WorldState, so the .civsave.zst archive path silently reset
//! every faction's diplomacy stance, grief score, relation, and deep
//! diplomacy state on reload.

use civ_engine::{CivSaveBundle, Simulation};

#[test]
fn archive_roundtrips_diplomacy_state_after_save_load() {
    let mut sim = Simulation::with_seed(20260913);
    sim.advance_ticks(1);

    // Seed distinguishable deep_diplomacy state — we use
    // `faction_resources: BTreeMap<u32, i32>` which derives PartialEq,
    // so we can prove byte-for-byte roundtrip of this concrete field.
    sim.deep_diplomacy.faction_resources.insert(7, 4242);
    sim.deep_diplomacy.faction_resources.insert(11, 9001);

    // Tick once more so the save-side mirror at end of `tick()` writes
    // the freshly-seeded `sim.deep_diplomacy` into `sim.state.deep_diplomacy`.
    sim.advance_ticks(1);

    // Pre-save: live (Simulation) side and WorldState side must agree
    // (save-side mirror runs at end of every tick).
    assert_eq!(
        sim.deep_diplomacy.faction_resources, sim.state.deep_diplomacy.faction_resources,
        "pre-save deep_diplomacy.faction_resources must agree"
    );

    // Round-trip via the .civsave.zst archive path.
    let dir = tempfile::tempdir().expect("tempdir");
    let archive_path = dir.path().join("diplomacy.civsave.zst");
    CivSaveBundle::save_archive(&archive_path, &sim).expect("save_archive");
    let loaded = CivSaveBundle::load_archive(&archive_path).expect("load_archive");

    // Post-load WorldState side must equal pre-save live values byte-for-byte.
    assert_eq!(
        loaded.state.deep_diplomacy.faction_resources, sim.deep_diplomacy.faction_resources,
        "loaded WorldState.deep_diplomacy.faction_resources must equal pre-save live"
    );

    // Post-load Simulation side must equal WorldState side (load-side mirror).
    assert_eq!(
        loaded.deep_diplomacy.faction_resources, loaded.state.deep_diplomacy.faction_resources,
        "post-load Simulation.deep_diplomacy.faction_resources must match WorldState"
    );
}
