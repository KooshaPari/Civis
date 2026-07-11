//! Deferred FR-CIV-ARCH integration coverage.
//!
//! Covers the public building-emergence API and simulation hook.

use civ_engine::{
    biome_style_tag, building_type_unlocked_at_era, BiomeKind, BuildingType, Simulation,
};

#[test]
fn building_emergence_hook_preserves_seeded_building_invariants() {
    let arid = biome_style_tag(BiomeKind::Desert);
    assert_eq!(arid, biome_style_tag(BiomeKind::Savanna));
    assert_ne!(arid, biome_style_tag(BiomeKind::Forest));
    assert!(!building_type_unlocked_at_era(BuildingType::Barracks, 2));
    assert!(building_type_unlocked_at_era(BuildingType::Barracks, 4));

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
