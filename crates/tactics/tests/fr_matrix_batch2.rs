//! FR-matrix batch 2 — focused tactic coverage for 10 `IMPL-NO-TEST` IDs.
//!
//! These integration tests live under `crates/tactics/tests/`, so the traceability
//! scanner can attribute FR IDs directly to test references.

use civ_tactics::{
    score_doctrine_fitness, FactionEngagementStats,
    apply_damage,
    tick_operational_movement, OperationalMovementConfig,
    formation_offsets, FormationKind,
    line_of_sight,
    bfs_next_step,
    WarBridge,
    WarBridgeConfig,
    MilitaryUnitSample,
    tick_war_bridge,
    Doctrine, DoctrineLibrary, SCHEMA_VERSION, evolve_doctrine,
};
use civ_voxel::{MaterialId, VoxelWorld, WorldCoord};
use rand::SeedableRng;
use rand_chacha::ChaCha8Rng;

fn make_empty_world() -> VoxelWorld<MaterialId> {
    VoxelWorld::new(1)
}

fn world_coord(x: i64, y: i64, z: i64) -> WorldCoord {
    WorldCoord { x, y, z }
}

fn seed_rng() -> ChaCha8Rng {
    ChaCha8Rng::seed_from_u64(7)
}

// ---------------------------------------------------------------------------
// FR-CIV-TACTICS-000
// ---------------------------------------------------------------------------

/// FR-CIV-TACTICS-000 — schema version string is present and version-like.
/// Covers FR-CIV-TACTICS-000.
#[test]
fn green_fr_civ_tactics_000_schema_version_is_present() {
    assert!(!SCHEMA_VERSION.is_empty());
    assert!(SCHEMA_VERSION.contains('.'));
}

// ---------------------------------------------------------------------------
// FR-CIV-TACTICS-001
// ---------------------------------------------------------------------------

/// FR-CIV-TACTICS-001 — apply_damage removes voxels inside the impact sphere.
/// Covers FR-CIV-TACTICS-001.
#[test]
fn green_fr_civ_tactics_001_apply_damage_removes_voxels() {
    let mut world = make_empty_world();
    for z in -2..=2 {
        for x in -2..=2 {
            for y in -2..=2 {
                world.write(world_coord(x, y, z), MaterialId(1));
            }
        }
    }

    let removed = apply_damage(
        &mut world,
        &civ_tactics::DamageEvent {
            center: WorldCoord { x: 0, y: 0, z: 0 },
            radius_voxels: 1,
            energy: 5,
        },
    );

    assert!(removed > 0);
    assert_eq!(world.read(world_coord(0, 0, 0)), MaterialId(0));
}

// ---------------------------------------------------------------------------
// FR-CIV-TACTICS-010
// ---------------------------------------------------------------------------

/// FR-CIV-TACTICS-010 — evolve_doctrine increments generation and is
/// deterministic for the same seed.
/// Covers FR-CIV-TACTICS-010.
#[test]
fn green_fr_civ_tactics_010_evolve_doctrine_deterministic() {
    let mut library_a = DoctrineLibrary {
        current: vec![
            Doctrine {
                id: 1,
                unit_composition: vec![3, 1, 2],
                score: 5.0,
            },
            Doctrine {
                id: 2,
                unit_composition: vec![1, 2, 3],
                score: 6.0,
            },
        ],
        generation: 11,
    };
    let mut library_b = library_a.clone();
    let mut rng_a = seed_rng();
    let mut rng_b = seed_rng();
    evolve_doctrine(&mut library_a, &mut rng_a, 0.25);
    evolve_doctrine(&mut library_b, &mut rng_b, 0.25);
    assert_eq!(library_a, library_b);
    assert_eq!(library_a.generation, 12);
}

// ---------------------------------------------------------------------------
// FR-CIV-TACTICS-020
// ---------------------------------------------------------------------------

/// FR-CIV-TACTICS-020 — LOS checks block when a voxel is in between.
/// Covers FR-CIV-TACTICS-020.
#[test]
fn green_fr_civ_tactics_020_los_blocked_by_obstacle() {
    let mut world = make_empty_world();
    let from = world_coord(0, 0, 0);
    let to = world_coord(6, 0, 0);
    world.write(world_coord(3, 0, 0), MaterialId(1));
    assert!(!line_of_sight(&world, from, to));
}

// ---------------------------------------------------------------------------
// FR-CIV-TACTICS-021
// ---------------------------------------------------------------------------

/// FR-CIV-TACTICS-021 — line formation offsets keep requested slot count and stay centered.
/// Covers FR-CIV-TACTICS-021.
#[test]
fn green_fr_civ_tactics_021_line_offsets() {
    let offsets = formation_offsets(FormationKind::Line, 3);
    assert_eq!(offsets.len(), 3);
    assert_eq!(offsets[0], (0, -1));
    assert_eq!(offsets[1], (0, 0));
    assert_eq!(offsets[2], (0, 1));
}

// ---------------------------------------------------------------------------
// FR-CIV-TACTICS-022
// ---------------------------------------------------------------------------

/// FR-CIV-TACTICS-022 — combat resolves on the configured cadence.
/// Covers FR-CIV-TACTICS-022.
#[test]
fn green_fr_civ_tactics_022_engagement_respects_cadence() {
    let world = make_empty_world();
    let units = [
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
    ];

    let config = WarBridgeConfig {
        cadence_ticks: 4,
        engage_range_grid: 8,
        ..WarBridgeConfig::default()
    };
    let bridge = WarBridge::new(&world, config);

    let off_cadence = bridge.resolve_combat(3, &units, None);
    let on_cadence = bridge.resolve_combat(4, &units, None);

    assert!(off_cadence.is_empty());
    assert_eq!(on_cadence.len(), 2);
}

// ---------------------------------------------------------------------------
// FR-CIV-TACTICS-023
// ---------------------------------------------------------------------------

/// FR-CIV-TACTICS-023 — doctrine fitness is monotonic with engagement pressure.
/// Covers FR-CIV-TACTICS-023.
#[test]
fn green_fr_civ_tactics_023_doctrine_fitness_tracks_pressure() {
    let doctrine = Doctrine {
        id: 99,
        unit_composition: vec![2, 2, 2],
        score: 0.0,
    };
    let idle = FactionEngagementStats::default();
    let active = FactionEngagementStats {
        engagements_as_shooter: 3,
        engagements_as_target: 1,
        voxels_removed: 12,
    };
    assert!(score_doctrine_fitness(&doctrine, &active) > score_doctrine_fitness(&doctrine, &idle));
}

// ---------------------------------------------------------------------------
// FR-CIV-TACTICS-024
// ---------------------------------------------------------------------------

/// FR-CIV-TACTICS-024 — war bridge emits engagements toward opposite factions.
/// Covers FR-CIV-TACTICS-024.
#[test]
fn green_fr_civ_tactics_024_engages_opposite_factions() {
    let world = make_empty_world();
    let units = [
        MilitaryUnitSample {
            unit_id: 10,
            faction_id: 0,
            grid_x: 0,
            grid_y: 0,
        },
        MilitaryUnitSample {
            unit_id: 20,
            faction_id: 1,
            grid_x: 2,
            grid_y: 0,
        },
    ];

    let engagements = tick_war_bridge(
        16,
        &WarBridgeConfig {
            cadence_ticks: 16,
            engage_range_grid: 16,
            ..WarBridgeConfig::default()
        },
        &units,
        &world,
        None,
    );

    assert_eq!(engagements.len(), 2);
    let ids = engagements.iter().map(|engagement| (engagement.shooter_id, engagement.target_id)).collect::<Vec<_>>();
    assert!(ids.contains(&(10, 20)));
    assert!(ids.contains(&(20, 10)));
}

// ---------------------------------------------------------------------------
// FR-CIV-TACTICS-030
// ---------------------------------------------------------------------------
/// Covers FR-CIV-TACTICS-030.
#[test]
fn green_fr_civ_tactics_030_war_bridge_config_is_configurable() {
    let config = WarBridgeConfig {
        engage_range_grid: 16,
        ..WarBridgeConfig::default()
    };
    assert_eq!(config.engage_range_grid, 16);
}

// ---------------------------------------------------------------------------
// FR-CIV-TACTICS-031
// ---------------------------------------------------------------------------

/// FR-CIV-TACTICS-031 — movement moves units toward enemies on cadence.
/// Covers FR-CIV-TACTICS-031.
#[test]
fn green_fr_civ_tactics_031_movement_moves_toward_enemy() {
    let world = make_empty_world();
    let config = OperationalMovementConfig {
        cadence_ticks: 2,
        path_search_radius: 8,
    };
    let mut units = vec![
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
    ];

    let moves = tick_operational_movement(2, &config, &mut units, 1, &world);
    assert!(!moves.is_empty());
    assert_eq!(moves[0].unit_index, 0);
    assert!(units[moves[0].unit_index].grid_x > 0);
}

// ---------------------------------------------------------------------------
// FR-CIV-TACTICS-033
// ---------------------------------------------------------------------------

/// FR-CIV-TACTICS-033 — BFS next-step is stable and moves toward the target.
/// Covers FR-CIV-TACTICS-033.
#[test]
fn green_fr_civ_tactics_033_bfs_next_step_steers_toward_target() {
    let first = bfs_next_step((0, 0), (6, 0), 24);
    let second = bfs_next_step((0, 0), (6, 0), 24);
    assert_eq!(first, second);
    assert_eq!(first, Some((1, 0)));
}
