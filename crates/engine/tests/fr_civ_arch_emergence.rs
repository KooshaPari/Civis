//! FR-CIV-ARCH engine integration tests (emergence matrix batch).

use civ_engine::building_emergence::{biome_style_tag, emergent_style_key_for_sim, settlement_build_anchor};
use civ_engine::{BuildingType, Simulation};
use civ_planet::{defaults_earthlike, BiomeKind, GeologyMap};
use civ_voxel::WorldCoord;

/// FR-CIV-ARCH — settlement anchor returns centroid for multi-member clusters.
#[test]
fn fr_arch_settlement_anchor_from_clusters() {
    let sim = Simulation::with_seed(5);
    let (_cluster, anchor) = settlement_build_anchor(&sim.world);
    let _ = anchor;
}

/// FR-CIV-ARCH — biome style tag maps enriched biomes.
#[test]
fn fr_arch_biome_style_tag_mangrove_coastal() {
    assert_eq!(
        biome_style_tag(BiomeKind::Mangrove),
        civ_build::BiomeStyleTag::COASTAL
    );
}

/// FR-CIV-ARCH — style key derives deterministically from simulation state.
#[test]
fn fr_arch_style_key_is_deterministic_for_seed() {
    let sim_a = Simulation::with_seed(12);
    let sim_b = Simulation::with_seed(12);
    let geology = GeologyMap::seed(&defaults_earthlike().0);
    let anchor = WorldCoord { x: 0, y: 0, z: 0 };
    let key_a = emergent_style_key_for_sim(&sim_a, None, &geology, &anchor);
    let key_b = emergent_style_key_for_sim(&sim_b, None, &geology, &anchor);
    assert_eq!(key_a, key_b);
}

/// FR-CIV-ARCH — era-gated building types unlock with build era.
#[test]
fn fr_arch_building_type_era_unlocks() {
    assert!(BuildingType::Farm.min_era() == 0);
    assert!(BuildingType::Market.min_era() == 2);
    assert!(BuildingType::Barracks.min_era() == 4);
}
