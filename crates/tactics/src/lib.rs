//! civ-tactics — Tactical voxel-destructible combat (per-soldier) + doctrine evolution genetic-algo
//!
//! Part of the Civis 3D extension (feat/civis-3d-foundation).
//! See `docs/roadmap/civis-3d-extension.md` for the full design context.
//!
//! Functional requirements: FR-CIV-TACTICS-*

#![forbid(unsafe_code)]
#![warn(missing_docs)]

mod doctrine_fitness;
mod fog_of_war;
mod formation;
mod grid_obstacles;
mod los;
mod military_phase;
mod morale;
mod movement;
mod operational;
mod pathfinding;
mod war_bridge;

pub use doctrine_fitness::{score_doctrine_fitness, FactionEngagementStats};
pub use fog_of_war::FogOfWar;
pub use formation::{
    formation_offsets, formation_positions, rotate_offsets, Facing, FormationKind,
};
pub use grid_obstacles::{grid_cell_blocked, grid_cell_impassable, grid_cell_occupied};
pub use los::line_of_sight;
pub use military_phase::MilitaryPhaseConfig;
pub use morale::*;
pub use movement::{
    operational_movement_pulse, tick_operational_movement, GridMove, OperationalMovementConfig,
};
pub use operational::{NoopOperationalLayer, OperationalLayer};
pub use pathfinding::{
    astar_path, astar_path_with_blocked, bfs_next_step, bfs_next_step_with_blocked,
};
pub use war_bridge::{
    build_fog_for_units, grid_to_world_coord, tick_war_bridge, CombatEngagement,
    MilitaryUnitSample, WarBridge, WarBridgeConfig,
};

use civ_voxel::{MaterialId, VoxelWorld, WorldCoord};
use rand::Rng;
use rand_chacha::ChaCha8Rng;
use serde::{Deserialize, Serialize};
use std::cmp::Ordering;

/// Marker version of this crate's public schema.
pub const SCHEMA_VERSION: &str = "0.1.0";

/// A voxel damage application centered at a world coordinate.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct DamageEvent {
    /// Center of the damage sphere.
    pub center: WorldCoord,
    /// Radius of the damage sphere in voxels.
    pub radius_voxels: u8,
    /// Energy carried by the event.
    pub energy: u32,
}

/// Casualty conversion: how much blast-footprint·energy maps to one personnel
/// loss. Larger = fewer casualties per unit of damage. Tuned so a small
/// skirmish blast (r≈2, energy≈100) yields a handful of casualties.
const CASUALTY_DIVISOR: u64 = 256;

impl DamageEvent {
    /// Estimate personnel casualties produced by this voxel damage event
    /// (FR-CIV-WAR-003, first increment toward population backprop).
    ///
    /// Models lethal exposure as the blast footprint (disk area `≈ 3·r²`, an
    /// integer approximation of `π·r²`) scaled by the delivered `energy`, divided
    /// by [`CASUALTY_DIVISOR`]. Zero radius or zero energy yields zero casualties;
    /// casualties rise monotonically with both. Pure and deterministic, so it is
    /// replay-stable. The actual deduction from a unit's strength is a follow-up
    /// once units carry a personnel count.
    #[must_use]
    pub fn estimated_casualties(&self) -> u32 {
        let r = u64::from(self.radius_voxels);
        let footprint = 3 * r * r; // ≈ π r²
        let raw = footprint.saturating_mul(u64::from(self.energy)) / CASUALTY_DIVISOR;
        u32::try_from(raw).unwrap_or(u32::MAX)
    }
}

/// A doctrine candidate for the GA.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Doctrine {
    /// Stable doctrine identifier.
    pub id: u64,
    /// Unit composition counts by unit-type slot.
    pub unit_composition: Vec<u16>,
    /// Fitness score.
    pub score: f32,
}

/// Doctrine population and generation counter.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DoctrineLibrary {
    /// Current population.
    pub current: Vec<Doctrine>,
    /// Current generation number.
    pub generation: u32,
}

fn material_is_solid(material: MaterialId) -> bool {
    material.0 != 0
}

fn to_i64(coord: WorldCoord) -> (i64, i64, i64) {
    (coord.x, coord.y, coord.z)
}

fn within_sphere(center: (i64, i64, i64), pos: (i64, i64, i64), radius: i64) -> bool {
    let dx = pos.0 - center.0;
    let dy = pos.1 - center.1;
    let dz = pos.2 - center.2;
    dx * dx + dy * dy + dz * dz <= radius * radius
}

/// Carves a spherical region of voxels to `MaterialId(0)`.
///
/// Returns the number of voxels removed.
pub fn apply_damage(world: &mut VoxelWorld<MaterialId>, event: &DamageEvent) -> usize {
    let radius = i64::from(event.radius_voxels);
    let center = to_i64(event.center);
    let mut removed = 0usize;

    for dz in -radius..=radius {
        for dy in -radius..=radius {
            for dx in -radius..=radius {
                if !within_sphere(
                    center,
                    (center.0 + dx, center.1 + dy, center.2 + dz),
                    radius,
                ) {
                    continue;
                }
                let pos = WorldCoord {
                    x: center.0 + dx,
                    y: center.1 + dy,
                    z: center.2 + dz,
                };
                let material = world.read(pos);
                if material_is_solid(material) {
                    world.write(pos, MaterialId(0));
                    removed += 1;
                }
            }
        }
    }

    removed
}

fn tournament_index(rng: &mut ChaCha8Rng, len: usize) -> usize {
    let a = rng.gen_range(0..len);
    let b = rng.gen_range(0..len);
    a.max(b)
}

fn mutate_composition(composition: &mut [u16], mutation_rate: f32, rng: &mut ChaCha8Rng) {
    for slot in composition {
        if rng.gen::<f32>() < mutation_rate {
            let delta: i16 = rng.gen_range(-3..=3);
            let next = (*slot as i32 + i32::from(delta)).clamp(0, u16::MAX as i32) as u16;
            *slot = next;
        }
    }
}

fn compare_scores(a: &Doctrine, b: &Doctrine) -> Ordering {
    a.score
        .partial_cmp(&b.score)
        .unwrap_or(Ordering::Equal)
        .then_with(|| a.id.cmp(&b.id))
}

/// Evolves the doctrine library in place using tournament selection and mutation.
pub fn evolve_doctrine(library: &mut DoctrineLibrary, rng: &mut ChaCha8Rng, mutation_rate: f32) {
    if library.current.is_empty() {
        library.generation = library.generation.saturating_add(1);
        return;
    }

    let mut ranked = library.current.clone();
    ranked.sort_by(|a, b| compare_scores(b, a));
    let population_size = library.current.len();
    let mut next = Vec::with_capacity(population_size);

    for index in 0..population_size {
        let first = &ranked[tournament_index(rng, ranked.len())];
        let second = &ranked[tournament_index(rng, ranked.len())];
        let mut child = if compare_scores(first, second) != Ordering::Less {
            first.clone()
        } else {
            second.clone()
        };
        child.id = library.generation as u64 + index as u64 + 1;
        mutate_composition(&mut child.unit_composition, mutation_rate, rng);
        next.push(child);
    }

    library.current = next;
    library.generation = library.generation.saturating_add(1);
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::SeedableRng;

    fn rng(seed: u64) -> ChaCha8Rng {
        ChaCha8Rng::seed_from_u64(seed)
    }

    fn world_with_cross() -> VoxelWorld<MaterialId> {
        let mut world = VoxelWorld::new(1);
        for x in -4..=4 {
            for y in -4..=4 {
                for z in -4..=4 {
                    let pos = WorldCoord { x, y, z };
                    if x.abs() + y.abs() + z.abs() <= 4 {
                        world.write(pos, MaterialId(1));
                    }
                }
            }
        }
        world
    }

    /// FR-CIV-TACTICS-000 — exposes a semver-like schema version stub.
    #[test]
    fn schema_version_stub() {
        assert!(!SCHEMA_VERSION.is_empty());
        let core = SCHEMA_VERSION.split('-').next().unwrap();
        let segments: Vec<&str> = core.split('.').collect();
        assert_eq!(segments.len(), 3);
        assert!(segments.iter().all(|part| !part.is_empty()));
    }

    /// FR-CIV-TACTICS-001 — apply_damage removes voxels in a sphere.
    #[test]
    fn estimated_casualties_zero_when_no_energy_or_radius() {
        let center = WorldCoord { x: 0, y: 0, z: 0 };
        assert_eq!(
            DamageEvent { center, radius_voxels: 0, energy: 500 }.estimated_casualties(),
            0,
            "zero radius => no footprint => no casualties"
        );
        assert_eq!(
            DamageEvent { center, radius_voxels: 5, energy: 0 }.estimated_casualties(),
            0,
            "zero energy => no lethality => no casualties"
        );
    }

    #[test]
    fn estimated_casualties_monotonic_in_radius_and_energy() {
        let center = WorldCoord { x: 0, y: 0, z: 0 };
        let base = DamageEvent { center, radius_voxels: 3, energy: 200 };
        let bigger_r = DamageEvent { radius_voxels: 5, ..base };
        let bigger_e = DamageEvent { energy: 400, ..base };
        assert!(bigger_r.estimated_casualties() > base.estimated_casualties());
        assert!(bigger_e.estimated_casualties() > base.estimated_casualties());
        // 3·3²·200 / 256 = 5400/256 = 21
        assert_eq!(base.estimated_casualties(), 21);
    }

    #[test]
    fn estimated_casualties_saturates_without_overflow() {
        let center = WorldCoord { x: 0, y: 0, z: 0 };
        let huge = DamageEvent { center, radius_voxels: u8::MAX, energy: u32::MAX };
        // Must not panic; large but finite.
        let c = huge.estimated_casualties();
        assert!(c > 0);
    }

    #[test]
    fn apply_damage_removes_voxels_in_a_sphere() {
        let mut world = world_with_cross();
        let event = DamageEvent {
            center: WorldCoord { x: 0, y: 0, z: 0 },
            radius_voxels: 2,
            energy: 10,
        };
        let removed = apply_damage(&mut world, &event);
        assert!(removed > 0);
        assert_eq!(world.read(WorldCoord { x: 0, y: 0, z: 0 }), MaterialId(0));
    }

    /// FR-CIV-TACTICS-002 — apply_damage is deterministic.
    #[test]
    fn apply_damage_is_deterministic() {
        let event = DamageEvent {
            center: WorldCoord { x: 0, y: 0, z: 0 },
            radius_voxels: 3,
            energy: 99,
        };
        let mut w1 = world_with_cross();
        let mut w2 = world_with_cross();
        assert_eq!(apply_damage(&mut w1, &event), apply_damage(&mut w2, &event));
        assert_eq!(w1.drain_dirty(), w2.drain_dirty());
    }

    /// FR-CIV-TACTICS-003 — apply_damage outside any chunk is a no-op.
    #[test]
    fn apply_damage_outside_any_chunk_is_a_noop() {
        let mut world = VoxelWorld::new(1);
        let event = DamageEvent {
            center: WorldCoord {
                x: 10_000,
                y: 10_000,
                z: 10_000,
            },
            radius_voxels: 2,
            energy: 1,
        };
        assert_eq!(apply_damage(&mut world, &event), 0);
    }

    /// FR-CIV-TACTICS-010 — evolve_doctrine bumps generation and is deterministic
    /// under a fixed seed.
    #[test]
    fn evolve_doctrine_is_deterministic_under_fixed_seed() {
        let mut lib1 = DoctrineLibrary {
            current: vec![
                Doctrine {
                    id: 1,
                    unit_composition: vec![5, 1, 0],
                    score: 10.0,
                },
                Doctrine {
                    id: 2,
                    unit_composition: vec![1, 4, 2],
                    score: 12.0,
                },
            ],
            generation: 7,
        };
        let mut lib2 = lib1.clone();
        let mut r1 = rng(42);
        let mut r2 = rng(42);
        evolve_doctrine(&mut lib1, &mut r1, 0.5);
        evolve_doctrine(&mut lib2, &mut r2, 0.5);
        assert_eq!(lib1, lib2);
        assert_eq!(lib1.generation, 8);
    }

    /// FR-CIV-TACTICS-020 — line_of_sight blocks solid voxels between endpoints.
    #[test]
    fn line_of_sight_blocks_solid_voxels() {
        let mut world = VoxelWorld::new(1);
        let from = WorldCoord { x: 0, y: 0, z: 0 };
        let to = WorldCoord { x: 8, y: 0, z: 0 };
        assert!(line_of_sight(&world, from, to));
        for x in 1..8 {
            world.write(WorldCoord { x, y: 0, z: 0 }, MaterialId(1));
        }
        assert!(!line_of_sight(&world, from, to));
    }

    /// FR-CIV-TACTICS-021 — formation_offsets returns stable slot layouts.
    ///
    /// Canonical frame: +dx = forward (depth), +dy = right flank.
    /// Line spreads units along the dy (flank) axis, centred on the leader.
    #[test]
    fn formation_offsets_line_and_wedge() {
        let line = formation_offsets(FormationKind::Line, 3);
        // centre = (3-1)/2 = 1 → dy offsets: -1, 0, 1; dx always 0
        assert_eq!(line, vec![(0, -1), (0, 0), (0, 1)]);
        let wedge = formation_offsets(FormationKind::Wedge, 3);
        assert_eq!(wedge.len(), 3);
        assert_eq!(wedge[0], (0, 0));
    }

    /// FR-CIV-TACTICS-022/024 — war bridge returns per-soldier engagements on cadence.
    #[test]
    fn war_bridge_queues_damage_on_cadence_with_los() {
        let world = VoxelWorld::new(1);
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
                grid_x: 4,
                grid_y: 0,
            },
        ];
        let config = WarBridgeConfig {
            cadence_ticks: 4,
            ..WarBridgeConfig::default()
        };
        assert!(tick_war_bridge(3, &config, &units, &world, None).is_empty());
        let engagements = tick_war_bridge(4, &config, &units, &world, None);
        assert_eq!(engagements.len(), 2);
        assert!(engagements
            .iter()
            .all(|e| e.damage.radius_voxels == config.damage_radius_voxels));
        assert!(engagements
            .iter()
            .any(|e| e.shooter_id == 10 && e.target_id == 20));
    }

    /// FR-CIV-TACTICS-033 — BFS next step is stable toward a distant target.
    #[test]
    fn pathfinding_bfs_steps_toward_enemy() {
        assert_eq!(bfs_next_step((0, 0), (5, 0), 16), Some((1, 0)));
    }

    /// FR-CIV-TACTICS-039 — movement routes around cells occupied by other units.
    #[test]
    fn operational_movement_avoids_occupied_cell() {
        let mut units = [
            MilitaryUnitSample {
                unit_id: 1,
                faction_id: 0,
                grid_x: 0,
                grid_y: 0,
            },
            MilitaryUnitSample {
                unit_id: 2,
                faction_id: 0,
                grid_x: 1,
                grid_y: 0,
            },
            MilitaryUnitSample {
                unit_id: 3,
                faction_id: 1,
                grid_x: 4,
                grid_y: 0,
            },
        ];
        let config = OperationalMovementConfig::default();
        let world = VoxelWorld::new(1);
        let moves = tick_operational_movement(4, &config, &mut units, 1, &world);
        assert!(moves.iter().any(|m| m.unit_index == 0 && m.new_grid_y == 1));
    }

    /// FR-CIV-TACTICS-036 — movement routes around solid voxels on the grid plane.
    #[test]
    fn operational_movement_avoids_voxel_obstacle() {
        use crate::war_bridge::grid_to_world_coord;
        let mut world = VoxelWorld::new(1);
        world.write(grid_to_world_coord(1, 0), MaterialId(1));
        let mut units = [
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
        let config = OperationalMovementConfig::default();
        let moves = tick_operational_movement(4, &config, &mut units, 1, &world);
        assert!(moves.iter().any(|m| m.unit_index == 0 && m.new_grid_y == 1));
    }

    /// FR-CIV-TACTICS-031 — operational movement steps toward nearest enemy.
    #[test]
    fn operational_movement_steps_toward_enemy() {
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
        let mut units = units;
        let config = OperationalMovementConfig {
            cadence_ticks: 4,
            ..Default::default()
        };
        let world = VoxelWorld::new(1);
        assert!(tick_operational_movement(3, &config, &mut units, 1, &world).is_empty());
        let moves = tick_operational_movement(4, &config, &mut units, 2, &world);
        assert!(!moves.is_empty());
        assert!(moves
            .iter()
            .any(|mv| mv.unit_index == 0 && mv.new_grid_x >= 1));
    }

    /// FR-CIV-TACTICS-023 — doctrine fitness increases with engagement pressure.
    #[test]
    fn doctrine_fitness_rewards_engagement_stats() {
        let doctrine = Doctrine {
            id: 1,
            unit_composition: vec![4, 4, 4],
            score: 0.0,
        };
        let quiet = FactionEngagementStats::default();
        let active = FactionEngagementStats {
            engagements_as_shooter: 2,
            engagements_as_target: 0,
            voxels_removed: 8,
        };
        assert!(
            score_doctrine_fitness(&doctrine, &active) > score_doctrine_fitness(&doctrine, &quiet)
        );
    }

    /// FR-CIV-TACTICS-011 — evolve_doctrine selects fitter doctrines.
    #[test]
    fn evolve_doctrine_selects_fitter_doctrines() {
        let mut library = DoctrineLibrary {
            current: vec![
                Doctrine {
                    id: 11,
                    unit_composition: vec![0, 0, 1],
                    score: 1.0,
                },
                Doctrine {
                    id: 22,
                    unit_composition: vec![9, 9, 9],
                    score: 100.0,
                },
            ],
            generation: 1,
        };
        let mut rng = rng(1);
        evolve_doctrine(&mut library, &mut rng, 0.0);
        assert!(library.current.iter().all(|d| d.score >= 1.0));
        assert!(library.current.iter().any(|d| d.score == 100.0));
    }
}

/// A military unit in the world — personnel strength, position, and morale state.
///
/// FR-CIV-TACTICS: Represents a cohesive tactical unit that can move, engage in combat,
/// and track morale. Serializable for save/load.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MilitaryUnit {
    /// Remaining health points / strength (0 = defeated).
    pub hp: u32,
    /// Grid-aligned position (x, y).
    pub pos: (i32, i32),
    /// Morale level (0.0 = routed, 1.0 = elite).
    pub morale: f32,
}

impl MilitaryUnit {
    /// Create a new military unit with full strength and morale.
    #[must_use]
    pub fn new(hp: u32, x: i32, y: i32) -> Self {
        Self {
            hp,
            pos: (x, y),
            morale: 1.0,
        }
    }

    /// Check if this unit is still combat-effective (hp > 0).
    #[must_use]
    pub fn is_alive(&self) -> bool {
        self.hp > 0
    }
}

#[cfg(test)]
mod test_military_unit {
    use super::*;

    #[test]
    fn test_military_unit_construct_and_serde() {
        let unit = MilitaryUnit::new(50, 10, 20);
        assert_eq!(unit.hp, 50);
        assert_eq!(unit.pos, (10, 20));
        assert_eq!(unit.morale, 1.0);

        // Round-trip through serde (using bincode).
        let encoded = bincode_next::encode_to_vec(&unit, bincode_next::config::standard())
            .expect("encode");
        let decoded: MilitaryUnit =
            bincode_next::decode_from_slice(&encoded, bincode_next::config::standard())
                .expect("decode")
                .0;
        assert_eq!(decoded, unit);
    }

    #[test]
    fn test_military_unit_is_alive() {
        let mut unit = MilitaryUnit::new(50, 0, 0);
        assert!(unit.is_alive());
        unit.hp = 0;
        assert!(!unit.is_alive());
    }
}
