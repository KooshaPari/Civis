//! civ-tactics — Tactical voxel-destructible combat (per-soldier) + doctrine evolution genetic-algo
//!
//! Part of the Civis 3D extension (feat/civis-3d-foundation).
//! See `docs/roadmap/civis-3d-extension.md` for the full design context.
//!
//! Functional requirements: FR-CIV-TACTICS-*

#![forbid(unsafe_code)]
#![warn(missing_docs)]

mod formation;
mod los;
mod war_bridge;

pub use formation::{formation_offsets, FormationKind};
pub use los::line_of_sight;
pub use war_bridge::{
    grid_to_world_coord, tick_war_bridge, MilitaryUnitSample, WarBridgeConfig,
};

use civ_voxel::{MaterialId, VoxelWorld, WorldCoord};
use rand::Rng;
use rand_chacha::ChaCha8Rng;
use serde::{Deserialize, Serialize};
use std::cmp::Ordering;

/// Marker version of this crate's public schema.
pub const SCHEMA_VERSION: &str = "0.1.0-stub";

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
    #[test]
    fn formation_offsets_line_and_wedge() {
        let line = formation_offsets(FormationKind::Line, 3);
        assert_eq!(line, vec![(-1, 0), (0, 0), (1, 0)]);
        let wedge = formation_offsets(FormationKind::Wedge, 3);
        assert_eq!(wedge.len(), 3);
        assert_eq!(wedge[0], (0, 0));
    }

    /// FR-CIV-TACTICS-022 — war bridge queues damage when factions engage with LOS.
    #[test]
    fn war_bridge_queues_damage_on_cadence_with_los() {
        let world = VoxelWorld::new(1);
        let units = [
            MilitaryUnitSample {
                faction_id: 0,
                grid_x: 0,
                grid_y: 0,
            },
            MilitaryUnitSample {
                faction_id: 1,
                grid_x: 4,
                grid_y: 0,
            },
        ];
        let config = WarBridgeConfig::default();
        assert!(tick_war_bridge(31, &config, &units, &world).is_empty());
        let events = tick_war_bridge(32, &config, &units, &world);
        assert_eq!(events.len(), 2);
        assert!(events
            .iter()
            .all(|e| e.radius_voxels == config.damage_radius_voxels));
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
