//! Disaster effects for the simulation world.
//!
//! The API is intentionally thin: callers can trigger a named disaster at a
//! world coordinate, and the engine can expose a phase hook for future
//! scheduling without changing the core tick shape.

use civ_agents::Position3d;
use civ_needs::{Health as LifeHealth, Needs as LifeNeeds};
use civ_voxel::material::{AIR, GRAVEL, ICE, LAVA, STEAM, STONE, WATER};
use civ_voxel::WorldCoord;

#[cfg(test)]
use civ_voxel::MaterialId;
use hecs::Entity;
use serde::{Deserialize, Serialize};

use crate::engine::Simulation;

/// Supported disaster kinds.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum DisasterKind {
    /// Large impact crater, heat, and structural damage.
    Meteor,
    /// Water intrusion and flooding.
    Flood,
    /// Ground shock, rubble, and localized infrastructure damage.
    Quake,
    /// Hot spread that burns flammable areas.
    Wildfire,
    /// Wind-driven rain and safety loss.
    Storm,
    /// Disease pressure that mostly hits people rather than terrain.
    Plague,
}

/// Trigger a disaster immediately and apply its effects to terrain and agents.
pub fn trigger_disaster(sim: &mut Simulation, kind: DisasterKind, pos: WorldCoord) {
    apply_disaster(sim, kind, pos);
}

impl Simulation {
    /// Phase hook for disaster systems.
    pub fn phase_disasters(&mut self) {}
}

fn apply_disaster(sim: &mut Simulation, kind: DisasterKind, pos: WorldCoord) {
    let radius = radius_for(kind);
    let affected = positions_in_radius(pos, radius);
    match kind {
        DisasterKind::Meteor => {
            sim.push_voxel_write(pos, LAVA);
            for (i, cell) in affected.iter().enumerate() {
                if *cell == pos {
                    continue;
                }
                let material = match i {
                    0 => LAVA,
                    1..=6 => STONE,
                    7..=18 => GRAVEL,
                    _ => AIR,
                };
                sim.push_voxel_write(*cell, material);
            }
            hit_agents(sim, pos, radius, 0.28, 0.35, 0.25, 0.55, true);
        }
        DisasterKind::Flood => {
            for cell in affected {
                sim.push_voxel_write(cell, WATER);
            }
            hit_agents(sim, pos, radius, 0.10, 0.42, 0.20, 0.25, false);
        }
        DisasterKind::Quake => {
            for (i, cell) in affected.iter().enumerate() {
                let material = if i % 7 == 0 { STONE } else { GRAVEL };
                sim.push_voxel_write(*cell, material);
            }
            hit_agents(sim, pos, radius, 0.16, 0.30, 0.24, 0.20, false);
        }
        DisasterKind::Wildfire => {
            for (i, cell) in affected.iter().enumerate() {
                let material = if i % 3 == 0 { LAVA } else { STEAM };
                sim.push_voxel_write(*cell, material);
            }
            hit_agents(sim, pos, radius, 0.18, 0.46, 0.38, 0.20, true);
        }
        DisasterKind::Storm => {
            for (i, cell) in affected.iter().enumerate() {
                let material = if i % 4 == 0 { ICE } else { WATER };
                sim.push_voxel_write(*cell, material);
            }
            hit_agents(sim, pos, radius, 0.14, 0.20, 0.22, 0.12, false);
        }
        DisasterKind::Plague => {
            hit_agents(sim, pos, radius * 2, 0.05, 0.10, 0.18, 0.06, false);
        }
    }
}

fn radius_for(kind: DisasterKind) -> i64 {
    match kind {
        DisasterKind::Meteor => 3 * civ_voxel::FIXED_SCALE,
        DisasterKind::Flood => 5 * civ_voxel::FIXED_SCALE,
        DisasterKind::Quake => 4 * civ_voxel::FIXED_SCALE,
        DisasterKind::Wildfire => 4 * civ_voxel::FIXED_SCALE,
        DisasterKind::Storm => 6 * civ_voxel::FIXED_SCALE,
        DisasterKind::Plague => 2 * civ_voxel::FIXED_SCALE,
    }
}

fn positions_in_radius(center: WorldCoord, radius: i64) -> Vec<WorldCoord> {
    let mut out = Vec::new();
    let radius_cells = (radius / civ_voxel::FIXED_SCALE).max(1);
    for dx in -radius_cells..=radius_cells {
        for dy in -radius_cells..=radius_cells {
            for dz in -radius_cells..=radius_cells {
                if dx * dx + dy * dy + dz * dz <= radius_cells * radius_cells {
                    out.push(WorldCoord {
                        x: center.x + dx * civ_voxel::FIXED_SCALE,
                        y: center.y + dy * civ_voxel::FIXED_SCALE,
                        z: center.z + dz * civ_voxel::FIXED_SCALE,
                    });
                }
            }
        }
    }
    out.sort_unstable_by_key(|c| (c.x, c.y, c.z));
    out.dedup();
    out
}

fn hit_agents(
    sim: &mut Simulation,
    pos: WorldCoord,
    radius: i64,
    shelter_delta: f32,
    safety_delta: f32,
    food_delta: f32,
    health_delta: f32,
    heat_damage: bool,
) {
    let radius_sq = (radius as i128) * (radius as i128);
    let effects: Vec<(Entity, bool)> = {
        let entities: Vec<Entity> = sim
            .world
            .query::<(&civ_agents::Civilian, &Position3d)>()
            .iter()
            .filter_map(|(entity, (_, position))| {
                let dx = (position.coord.x - pos.x) as i128;
                let dy = (position.coord.y - pos.y) as i128;
                let dz = (position.coord.z - pos.z) as i128;
                (dx * dx + dy * dy + dz * dz <= radius_sq).then_some(entity)
            })
            .collect();

        entities
            .into_iter()
            .map(|entity| {
                let mut despawn = false;
                if let Ok(mut needs) = sim.world.get::<&mut LifeNeeds>(entity) {
                    needs.rest = (needs.rest - shelter_delta).max(0.0);
                    needs.safety = (needs.safety - safety_delta).max(0.0);
                    needs.food = (needs.food - food_delta).max(0.0);
                    needs.health = (needs.health - health_delta).max(0.0);
                }
                if let Ok(mut life_health) = sim.world.get::<&mut LifeHealth>(entity) {
                    let damage = if heat_damage {
                        health_delta * 0.5
                    } else {
                        health_delta * 0.25
                    };
                    life_health.integrity = (life_health.integrity - damage).max(0.0);
                    despawn = life_health.integrity <= 0.0;
                }
                (entity, despawn)
            })
            .collect()
    };

    for (entity, despawn) in effects {
        if despawn {
            let _ = sim.world.despawn(entity);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use civ_agents::{Alignment, Civilian, LodTier, Position3d, Tools, Velocity, Wardrobe};
    use civ_needs::{Health as LifeHealth, Needs as LifeNeeds};

    fn seeded_sim() -> Simulation {
        let sim = Simulation::with_seed(7);
        sim
    }

    #[test]
    fn meteor_changes_terrain_and_hits_agents() {
        let mut sim = seeded_sim();
        let target = WorldCoord { x: 0, y: 0, z: 0 };
        trigger_disaster(&mut sim, DisasterKind::Meteor, target);

        assert_eq!(sim.voxel().read(target), LAVA);
        // Spawn a fresh `civ_needs::Needs`-bearing agent at the impact point
        // (the simulation's default spawn path doesn't include the needs
        // component, so we can't just look up an existing entity). Drop the
        // spawn-into-world helper into `seeded_sim` once the world owns a
        // default needs bundle.
        let pos = Position3d { coord: target };
        let entity = sim.world.spawn((
            Civilian {
                id: 9_999,
                alignment: Alignment::Faction(1),
                age: 24,
            },
            pos,
            LodTier::Hot,
            LifeNeeds::sated(),
            LifeHealth::default(),
        ));
        // Re-run just the agent hit so the impact hits our entity.
        hit_agents(
            &mut sim,
            target,
            3 * civ_voxel::FIXED_SCALE,
            0.28,
            0.35,
            0.25,
            0.55,
            true,
        );
        let needs = sim.world.get::<&LifeNeeds>(entity).expect("life needs");
        assert!(needs.rest < 1.0, "rest should drop after meteor");
        assert!(needs.safety < 1.0, "safety should drop after meteor");
        assert!(needs.food < 1.0, "food should drop after meteor");
    }

    #[test]
    fn phase_hook_is_callable() {
        let mut sim = Simulation::with_seed(1);
        sim.phase_disasters();
    }
}
