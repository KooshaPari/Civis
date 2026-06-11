//! FR-matrix batch 5 — integration tests for 10 IMPL-NO-TEST IDs in `civ-tactics`.
//!
//! Each `#[test]` function name contains the FR ID so the matrix scanner can
//! link the test back to the corresponding traceability row.
//!
//! Covered IDs:
//! - FR-CIV-TACTICS-000
//! - FR-CIV-TACTICS-001
//! - FR-CIV-TACTICS-010
//! - FR-CIV-TACTICS-020
//! - FR-CIV-TACTICS-021
//! - FR-CIV-TACTICS-022
//! - FR-CIV-TACTICS-023
//! - FR-CIV-TACTICS-024
//! - FR-CIV-TACTICS-033
//! - FR-CIV-TACTICS-037

use civ_tactics::{
    astar_path_with_blocked,
    bfs_next_step_with_blocked,
    evolve_doctrine,
    formation_offsets,
    line_of_sight,
    score_doctrine_fitness,
    tick_war_bridge,
    Doctrine,
    DoctrineLibrary,
    Facing,
    FactionEngagementStats,
    FormationKind,
    MilitaryUnitSample,
    WarBridge,
    WarBridgeConfig,
    SCHEMA_VERSION,
    DamageEvent,
};
use civ_voxel::{MaterialId, WorldCoord, VoxelWorld};
use rand::SeedableRng;
use rand_chacha::ChaCha8Rng;

fn world_for_damage() -> VoxelWorld<MaterialId> {
    let mut world = VoxelWorld::new(1);
    for x in -4_i32..=4 {
        for y in -4_i32..=4 {
            for z in -4_i32..=4 {
                let offset = x.abs() + y.abs() + z.abs();
                if offset <= 3 {
                    world.write(
                        WorldCoord {
                            x: i64::from(x),
                            y: i64::from(y),
                            z: i64::from(z),
                        },
                        MaterialId(1),
                    );
                }
            }
        }
    }
    world
}

fn unit(unit_id: u64, faction_id: u32, grid_x: i32, grid_y: i32) -> MilitaryUnitSample {
    MilitaryUnitSample {
        unit_id,
        faction_id,
        grid_x,
        grid_y,
    }
}

fn empty_world() -> VoxelWorld<MaterialId> {
    VoxelWorld::new(1)
}

// ---------------------------------------------------------------- FR-CIV-TACTICS-000
#[test]
fn fr_civ_tactics_000_schema_version_is_semver_like() {
    let root = SCHEMA_VERSION.split('-').next().unwrap_or_default();
    let parts: Vec<&str> = root.split('.').collect();
    assert_eq!(parts.len(), 3);
    assert!(parts.iter().all(|p| !p.is_empty()));
}

// ---------------------------------------------------------------- FR-CIV-TACTICS-001
#[test]
fn fr_civ_tactics_001_apply_damage_erosion() {
    let mut world = world_for_damage();
    let event = DamageEvent {
        center: WorldCoord {
            x: 0,
            y: 0,
            z: 0,
        },
        radius_voxels: 2,
        energy: 99,
    };
    let removed = civ_tactics::apply_damage(&mut world, &event);
    assert!(removed > 0);
    assert_eq!(world.read(WorldCoord { x: 0, y: 0, z: 0 }), MaterialId(0));
}

// ---------------------------------------------------------------- FR-CIV-TACTICS-010
#[test]
fn fr_civ_tactics_010_evolve_doctrine_is_deterministic() {
    let mut lib_a = DoctrineLibrary {
        current: vec![
            Doctrine {
                id: 1,
                unit_composition: vec![3, 3, 4],
                score: 6.5,
            },
            Doctrine {
                id: 2,
                unit_composition: vec![9, 1, 2],
                score: 2.1,
            },
        ],
        generation: 12,
    };
    let mut lib_b = lib_a.clone();
    let mut rng_a = ChaCha8Rng::seed_from_u64(11);
    let mut rng_b = ChaCha8Rng::seed_from_u64(11);
    evolve_doctrine(&mut lib_a, &mut rng_a, 0.5);
    evolve_doctrine(&mut lib_b, &mut rng_b, 0.5);
    assert_eq!(lib_a, lib_b);
    assert_eq!(lib_a.generation, 13);
}

// ---------------------------------------------------------------- FR-CIV-TACTICS-020
#[test]
fn fr_civ_tactics_020_los_respects_solid_blockers() {
    let mut world = empty_world();
    let from = WorldCoord {
        x: 0,
        y: 0,
        z: 0,
    };
    let to = WorldCoord {
        x: 8,
        y: 0,
        z: 0,
    };
    assert!(line_of_sight(&world, from, to));
    for x in 1..8 {
        world.write(
            WorldCoord {
                x: i64::from(x),
                y: 0,
                z: 0,
            },
            MaterialId(1),
        );
    }
    assert!(!line_of_sight(&world, from, to));
}

// ---------------------------------------------------------------- FR-CIV-TACTICS-021
#[test]
fn fr_civ_tactics_021_formation_offsets_are_centered_for_lines() {
    assert_eq!(formation_offsets(FormationKind::Line, 1), vec![(0, 0)]);
    assert_eq!(formation_offsets(FormationKind::Line, 4), vec![(0, -1), (0, 0), (0, 1), (0, 2)]);
}

// ---------------------------------------------------------------- FR-CIV-TACTICS-022
#[test]
fn fr_civ_tactics_022_war_bridge_resolves_engagements_by_cadence() {
    let world = empty_world();
    let units = vec![
        unit(101, 0, 0, 0),
        unit(102, 1, 3, 0),
    ];
    let config = WarBridgeConfig {
        cadence_ticks: 4,
        ..WarBridgeConfig::default()
    };
    assert!(tick_war_bridge(3, &config, &units, &world, None).is_empty());
    let engagements = tick_war_bridge(4, &config, &units, &world, None);
    assert_eq!(engagements.len(), 2);
}

// ---------------------------------------------------------------- FR-CIV-TACTICS-023
#[test]
fn fr_civ_tactics_023_drops_score_for_engagement_pressure() {
    let doctrine = Doctrine {
        id: 1,
        unit_composition: vec![2, 3, 5],
        score: 1.0,
    };
    let baseline = FactionEngagementStats::default();
    let engaged = FactionEngagementStats {
        engagements_as_shooter: 2,
        engagements_as_target: 1,
        voxels_removed: 3,
    };
    assert!(
        score_doctrine_fitness(&doctrine, &engaged) > score_doctrine_fitness(&doctrine, &baseline)
    );
}

// ---------------------------------------------------------------- FR-CIV-TACTICS-024
#[test]
fn fr_civ_tactics_024_war_bridge_formation_positions_track_unit_count() {
    let world = empty_world();
    let bridge = WarBridge::with_defaults(&world);
    let units = vec![unit(1, 0, 0, 0), unit(2, 0, 0, 1), unit(3, 0, 0, 2)];
    let positions = bridge.formation_move(&units, FormationKind::Line, Facing::East);
    assert_eq!(positions.len(), units.len());
    let mut deduped = positions.clone();
    deduped.sort_unstable();
    deduped.dedup();
    assert_eq!(deduped.len(), positions.len());
}

// ---------------------------------------------------------------- FR-CIV-TACTICS-033
#[test]
fn fr_civ_tactics_033_bfs_moves_toward_goal_with_blocked_cells() {
    let blocked = |x: i32, y: i32| x == 1 && y == 0;
    assert_eq!(
        bfs_next_step_with_blocked((0, 0), (3, 0), 16, &blocked),
        Some((0, 1))
    );
}

// ---------------------------------------------------------------- FR-CIV-TACTICS-037
#[test]
fn fr_civ_tactics_037_astar_finds_blocked_path() {
    let blocked = |x: i32, y: i32| x == 1 && y == 0;
    let path = astar_path_with_blocked((0, 0), (3, 0), 64, &blocked)
        .expect("path should route around the obstacle");
    assert_eq!(path.first().copied(), Some((0, 0)));
    assert_eq!(path.last().copied(), Some((3, 0)));
    assert!(path.iter().all(|&(x, y)| !blocked(x, y)));
}
