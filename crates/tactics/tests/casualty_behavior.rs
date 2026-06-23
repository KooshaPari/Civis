//! BDD tests for combat casualty behaviour (FR-CIV-WAR-003).
//!
//! These tests assert that engagements produced by the war bridge yield
//! `DamageEvent`s whose `estimated_casualties()` is non-zero and can be used
//! to reduce a synthetic unit population, modelling the first increment of
//! population backpropagation.

use civ_tactics::{
    apply_damage, tick_war_bridge, DamageEvent, MilitaryUnitSample, WarBridge, WarBridgeConfig,
};
use civ_voxel::{MaterialId, VoxelWorld, WorldCoord};

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn empty_world() -> VoxelWorld<MaterialId> {
    VoxelWorld::new(1)
}

fn immediate_config() -> WarBridgeConfig {
    WarBridgeConfig {
        cadence_ticks: 1,
        engage_range_grid: 16,
        damage_radius_voxels: 2,
        damage_energy: 250,
        ..WarBridgeConfig::default()
    }
}

fn two_enemies() -> [MilitaryUnitSample; 2] {
    [
        MilitaryUnitSample {
            unit_id: 1,
            faction_id: 0,
            grid_x: 0,
            grid_y: 0,
        },
        MilitaryUnitSample {
            unit_id: 2,
            faction_id: 1,
            grid_x: 4,
            grid_y: 0,
        },
    ]
}

fn reduce_population(population: &mut u32, casualties: u32) {
    *population = population.saturating_sub(casualties);
}

// ---------------------------------------------------------------------------
// BDD — Given / When / Then
// ---------------------------------------------------------------------------

/// **Scenario:** A successful engagement produces a damage event with
/// estimated casualties.
///
/// *Given* two enemy units within engagement range with clear line-of-sight  
/// *When* the war bridge resolves combat on a cadence tick  
/// *Then* each engagement contains a `DamageEvent` whose `estimated_casualties()`
///        is greater than zero.
#[test]
fn engagements_produce_estimated_casualties() {
    let world = empty_world();
    let units = two_enemies();
    let config = immediate_config();

    let engagements = tick_war_bridge(1, &config, &units, &world, None);
    assert!(
        !engagements.is_empty(),
        "expected at least one engagement in clear LOS"
    );

    for engagement in &engagements {
        let casualties = engagement.damage.estimated_casualties();
        assert!(
            casualties > 0,
            "engagement {}→{} should produce non-zero casualties, got {}",
            engagement.shooter_id,
            engagement.target_id,
            casualties
        );
    }
}

/// **Scenario:** Applying estimated casualties to a unit population reduces it.
///
/// *Given* a unit with a population of 100  
/// *When* an engagement damage event estimates 5 casualties  
/// *Then* the unit population is reduced to 95.
#[test]
fn estimated_casualties_reduce_population() {
    let center = WorldCoord { x: 0, y: 0, z: 0 };
    let damage = DamageEvent {
        center,
        radius_voxels: 2,
        energy: 250,
    };

    let casualties = damage.estimated_casualties();
    assert!(casualties > 0, "damage should yield non-zero casualties");

    let mut population = 100_u32;
    reduce_population(&mut population, casualties);
    assert_eq!(
        population,
        100 - casualties,
        "population should be reduced by estimated casualties"
    );
    assert!(population < 100, "population must be strictly lower after casualties");
}

/// **Scenario:** Large damage can completely wipe out a small unit.
///
/// *Given* a unit with a population of 10  
/// *When* a massive damage event estimates 50 casualties  
/// *Then* the unit population is reduced to 0 (saturated).
#[test]
fn casualties_saturate_at_zero_population() {
    let center = WorldCoord { x: 0, y: 0, z: 0 };
    let huge_damage = DamageEvent {
        center,
        radius_voxels: 10,
        energy: 10_000,
    };

    let casualties = huge_damage.estimated_casualties();
    assert!(casualties > 0, "huge damage should yield non-zero casualties");

    let mut population = 10_u32;
    reduce_population(&mut population, casualties);
    assert_eq!(
        population, 0,
        "small population should be fully wiped out by large casualty estimate"
    );
}

/// **Scenario:** Zero-radius or zero-energy damage yields zero casualties and
/// leaves population untouched.
///
/// *Given* a unit with a population of 50  
/// *When* a damage event has zero radius or zero energy  
/// *Then* estimated casualties are zero and population remains 50.
#[test]
fn zero_damage_yields_zero_casualties_and_no_population_loss() {
    let center = WorldCoord { x: 0, y: 0, z: 0 };

    let no_radius = DamageEvent {
        center,
        radius_voxels: 0,
        energy: 500,
    };
    let no_energy = DamageEvent {
        center,
        radius_voxels: 5,
        energy: 0,
    };

    assert_eq!(no_radius.estimated_casualties(), 0);
    assert_eq!(no_energy.estimated_casualties(), 0);

    let mut population = 50_u32;
    reduce_population(&mut population, no_radius.estimated_casualties());
    reduce_population(&mut population, no_energy.estimated_casualties());
    assert_eq!(population, 50, "population should remain unchanged after zero casualties");
}

/// **Scenario:** The same damage event always produces the same casualty
/// estimate (determinism).
///
/// *Given* a fixed damage event  
/// *When* estimated casualties are computed twice  
/// *Then* both results are identical.
#[test]
fn estimated_casualties_are_deterministic() {
    let damage = DamageEvent {
        center: WorldCoord { x: 1, y: 2, z: 3 },
        radius_voxels: 3,
        energy: 200,
    };

    assert_eq!(
        damage.estimated_casualties(),
        damage.estimated_casualties(),
        "casualty estimate must be deterministic"
    );
}

/// **Scenario:** Engagements from `WarBridge::resolve_combat` can be replayed
/// to reduce a tracked population map.
///
/// *Given* a population map with two units at 100 each  
/// *When* engagements are resolved and casualties applied  
/// *Then* each targeted unit's population is reduced by its engagement's
///        estimated casualties.
#[test]
fn resolve_combat_reduces_tracked_populations() {
    let world = empty_world();
    let units = two_enemies();
    let config = immediate_config();
    let bridge = WarBridge::new(&world, config);

    let mut populations: std::collections::HashMap<u64, u32> =
        units.iter().map(|u| (u.unit_id, 100)).collect();

    let engagements = bridge.resolve_combat(1, &units, None);
    assert!(
        !engagements.is_empty(),
        "expected engagements between two enemy units"
    );

    for engagement in &engagements {
        let casualties = engagement.damage.estimated_casualties();
        if let Some(pop) = populations.get_mut(&engagement.target_id) {
            reduce_population(pop, casualties);
        }
    }

    for (unit_id, pop) in &populations {
        assert!(
            *pop <= 100,
            "unit {} population {} should not exceed initial 100",
            unit_id,
            pop
        );
    }

    // Every unit that was targeted at least once must have a strictly lower
    // population (because the default damage config yields > 0 casualties).
    let targeted: std::collections::HashSet<u64> =
        engagements.iter().map(|e| e.target_id).collect();
    for unit_id in &targeted {
        let pop = populations.get(unit_id).copied().unwrap_or(0);
        assert!(
            pop < 100,
            "targeted unit {} should have lost population, got {}",
            unit_id,
            pop
        );
    }
}

/// **Scenario:** Damage events from engagements can also carve voxels.
///
/// *Given* a world with solid voxels at the target location  
/// *When* the engagement's damage event is applied to the world  
/// *Then* at least one voxel is removed.
#[test]
fn engagement_damage_removes_voxels() {
    let mut world = empty_world();
    let units = two_enemies();
    let config = immediate_config();

    // Place solid voxels at the target location before borrowing the world.
    let target = &units[1];
    let target_wc = civ_tactics::grid_to_world_coord(target.grid_x, target.grid_y);
    world.write(target_wc, MaterialId(1));

    let bridge = WarBridge::new(&world, config);

    let engagements = bridge.resolve_combat(1, &units, None);
    assert!(!engagements.is_empty(), "expected an engagement to hit the target");

    let mut total_removed = 0usize;
    for engagement in &engagements {
        total_removed += apply_damage(&mut world, &engagement.damage);
    }

    assert!(
        total_removed > 0,
        "expected at least one voxel removed by engagement damage"
    );
}

/// **Scenario:** Casualty estimate is monotonic with respect to energy.
///
/// *Given* a fixed radius and two energy levels  
/// *When* casualty estimates are computed  
/// *Then* the higher energy yields greater or equal casualties.
#[test]
fn casualty_estimate_scales_with_energy() {
    let center = WorldCoord { x: 0, y: 0, z: 0 };
    let base = DamageEvent {
        center,
        radius_voxels: 2,
        energy: 100,
    };
    let stronger = DamageEvent {
        center,
        radius_voxels: 2,
        energy: 400,
    };

    assert!(
        stronger.estimated_casualties() >= base.estimated_casualties(),
        "higher energy should yield greater or equal casualties"
    );
    assert!(
        stronger.estimated_casualties() > base.estimated_casualties(),
        "strictly greater casualties for strictly higher energy at same radius"
    );
}

/// **Scenario:** Casualty estimate is monotonic with respect to blast radius.
///
/// *Given* a fixed energy and two radii  
/// *When* casualty estimates are computed  
/// *Then* the larger radius yields greater or equal casualties.
#[test]
fn casualty_estimate_scales_with_radius() {
    let center = WorldCoord { x: 0, y: 0, z: 0 };
    let base = DamageEvent {
        center,
        radius_voxels: 1,
        energy: 200,
    };
    let wider = DamageEvent {
        center,
        radius_voxels: 4,
        energy: 200,
    };

    assert!(
        wider.estimated_casualties() > base.estimated_casualties(),
        "larger radius should yield strictly greater casualties at same energy"
    );
}

/// **Scenario:** Off-cadence ticks produce no engagements, therefore no
/// casualties and no population loss.
///
/// *Given* a war bridge with cadence 8  
/// *When* combat is resolved on tick 1  
/// *Then* there are no engagements and a tracked population stays at 100.
#[test]
fn off_cadence_produces_no_casualties() {
    let world = empty_world();
    let units = two_enemies();
    let config = WarBridgeConfig {
        cadence_ticks: 8,
        engage_range_grid: 16,
        ..WarBridgeConfig::default()
    };
    let bridge = WarBridge::new(&world, config);

    let mut population = 100_u32;
    let engagements = bridge.resolve_combat(1, &units, None);
    assert!(
        engagements.is_empty(),
        "off-cadence tick should not produce engagements"
    );

    for engagement in &engagements {
        reduce_population(&mut population, engagement.damage.estimated_casualties());
    }

    assert_eq!(population, 100, "no engagements => no population loss");
}
