//! Species lifecycle management: birth, death, aging, reproduction,
//! migration, and population rollups. Extracted from `engine.rs` in
//! decomposition pass 2 (Civis Engine Decomposition).

use civ_agents::{
    spawn_child_near, spawn_civilian_at, ActorVisualKind, Alignment, Civilian as AgentCivilian,
    Needs, Position3d,
};
use civ_voxel::FIXED_SCALE;
use hecs::{Entity, World};
use rand::Rng;
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;

use super::Simulation;
use crate::engine::{Fixed, SimRng};

// PopulationEvent and LifecycleCounters are defined in this file.

use std::collections::BTreeMap;

// ---------------------------------------------------------------------------
// Lifecycle types (moved from engine.rs)
// ---------------------------------------------------------------------------

/// Per-tick population event (birth or death).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PopulationEvent {
    pub tick: u64,
    pub entity_id: u64,
    pub x: f32,
    pub y: f32,
}

/// Per-tick lifecycle counters for children / adults / elders / dead.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct LifecycleCounters {
    pub children: u32,
    pub adults: u32,
    pub elders: u32,
    pub dead: u32,
}

impl LifecycleCounters {
    /// Total civilians observed across all labels.
    #[must_use]
    pub fn total(&self) -> u32 {
        self.children + self.adults + self.elders + self.dead
    }

    /// Total living civilians (children + adults + elders).
    #[must_use]
    pub fn total_living(&self) -> u32 {
        self.children + self.adults + self.elders
    }

    /// Working-age fraction (adults / total). Returns `0.0` when empty.
    #[must_use]
    pub fn adult_fraction(&self) -> f32 {
        let total = self.total();
        if total == 0 {
            0.0
        } else {
            self.adults as f32 / total as f32
        }
    }
}

// ---------------------------------------------------------------------------
// Lifecycle helper types (crate-private)
// ---------------------------------------------------------------------------

#[derive(Clone)]
pub(crate) struct CivilianLifecycleSample {
    pub age: u16,
    pub alignment: Alignment,
    pub x: f32,
    pub y: f32,
    pub fertility_score: f32,
    pub migration_pressure: f32,
}

// ---------------------------------------------------------------------------
// Free helper functions
// ---------------------------------------------------------------------------

/// Map an `AgentCivilian` age + `civ_agents::Needs` to a `civ_needs::Health`
/// value. `Health.integrity` is the mean of the four agent needs (`food`,
/// `shelter`, `safety`, `belonging`); the rest of `Health` is left at its
/// `Default::default()` so the lifecycle classifier uses the integrity axis
/// deterministically. Public so the test module can reuse the mapping without
/// duplicating the formula.
pub(crate) fn civilian_to_health(needs: &Needs) -> civ_needs::Health {
    let integrity =
        ((needs.food + needs.shelter + needs.safety + needs.belonging) * 0.25).clamp(0.0, 1.0);
    civ_needs::Health {
        integrity,
        ..civ_needs::Health::default()
    }
}

pub(crate) fn lifecycle_distance(ax: f32, ay: f32, bx: f32, by: f32) -> f32 {
    let dx = ax - bx;
    let dy = ay - by;
    (dx * dx + dy * dy).sqrt()
}

pub(crate) fn fertility_score(age: u16, needs: &Needs) -> f32 {
    let age_factor = if (18..=42).contains(&age) {
        1.0
    } else if age < 18 {
        (age as f32 / 18.0).clamp(0.0, 1.0)
    } else {
        (1.0 - ((age.saturating_sub(42) as f32) / 28.0)).clamp(0.0, 1.0)
    };
    let need_factor =
        ((needs.food + needs.rest + needs.safety + needs.belonging) * 0.25).clamp(0.0, 1.0);
    (0.55 * age_factor + 0.45 * need_factor).clamp(0.0, 1.0)
}

pub(crate) fn migration_pressure(needs: &Needs, resource_pressure: f32) -> f32 {
    let deprivation =
        1.0 - ((needs.food + needs.rest + needs.safety + needs.belonging) * 0.25).clamp(0.0, 1.0);
    (0.7 * deprivation + 0.3 * resource_pressure).clamp(0.0, 1.0)
}

pub(crate) fn apply_age_stage_effects(age: u16, needs: &mut Needs) {
    if age < 18 {
        needs.belonging = (needs.belonging + 0.01).min(1.0);
        needs.safety = (needs.safety + 0.01).min(1.0);
    } else if age < 50 {
        needs.food = (needs.food + 0.005).min(1.0);
    } else {
        needs.rest = (needs.rest - 0.01).max(0.0);
        needs.health = (needs.health - 0.01).max(0.0);
    }
}

pub(crate) fn is_fertile_adult(
    entity: Entity,
    world: &World,
    sample: &CivilianLifecycleSample,
) -> bool {
    let Ok(needs) = world.get::<&Needs>(entity) else {
        return false;
    };
    sample.age >= 18
        && sample.age <= 42
        && sample.fertility_score >= 0.72
        && needs.food >= 0.68
        && needs.rest >= 0.55
        && needs.safety >= 0.55
        && needs.belonging >= 0.5
}

pub(crate) fn is_migratory_adult(
    entity: Entity,
    world: &World,
    sample: &CivilianLifecycleSample,
) -> bool {
    let Ok(needs) = world.get::<&Needs>(entity) else {
        return false;
    };
    sample.age >= 18
        && sample.migration_pressure >= 0.68
        && needs.food <= 0.45
        && (needs.rest <= 0.75 || needs.safety <= 0.75 || needs.belonging <= 0.75)
}

pub(crate) fn settlement_anchor_for(settlement_id: u32, x: f32, y: f32) -> (f32, f32) {
    let seed = settlement_id as f32 * 0.137_503_2;
    let nx = (x + seed.sin() * 0.08).clamp(0.05, 0.95);
    let ny = (y + seed.cos() * 0.08).clamp(0.05, 0.95);
    (nx, ny)
}

/// Deterministic job assignment for agent civilians (stable across seeds).
pub fn job_type_for_civilian_id(id: u64) -> crate::engine::JobType {
    match id % 7 {
        0 => crate::engine::JobType::Farmer,
        1 => crate::engine::JobType::Warrior,
        2 => crate::engine::JobType::Scholar,
        3 => crate::engine::JobType::Trader,
        4 => crate::engine::JobType::Priest,
        5 => crate::engine::JobType::Admin,
        _ => crate::engine::JobType::Unemployed,
    }
}

/// Attach [`crate::engine::Citizen`] (with job) to agent entities that only
/// have [`AgentCivilian`].
pub fn attach_citizen_to_agents(world: &mut World) {
    let agents: Vec<(Entity, AgentCivilian)> = world
        .query::<&AgentCivilian>()
        .iter()
        .map(|(entity, civilian)| (entity, civilian.clone()))
        .collect();
    for (entity, civilian) in agents {
        if world.get::<&crate::engine::Citizen>(entity).is_ok() {
            continue;
        }
        let citizen = crate::engine::Citizen {
            age: civilian.age as u32,
            health: Fixed::from_num(1),
            ideology: Fixed::ZERO,
            welfare: Fixed::from_num(7) / Fixed::from_num(10),
            job: Some(job_type_for_civilian_id(civilian.id)),
        };
        let _ = world.insert(entity, (citizen,));
    }
}

pub(crate) fn spawn_faction_civilians(world: &mut World, rng: &mut SimRng) {
    spawn_faction_civilians_custom(world, rng, 32, 4, 2_500);
}

/// Spawn civilians for each faction with custom parameters.
pub(crate) fn spawn_faction_civilians_custom(
    world: &mut World,
    rng: &mut SimRng,
    civilians_per_faction: u32,
    faction_count: u32,
    quadrant_spread: i32,
) {
    let scale = FIXED_SCALE as f32;
    let mut next_civilian_id = 1u64;

    // Arrange faction capitals in a ring around the map center
    let faction_count_f32 = faction_count as f32;
    for faction in 0..faction_count {
        let angle = (faction as f32 / faction_count_f32) * std::f32::consts::TAU;
        let radius = 7_500.0;
        let center_x = (angle.cos() * radius) as i32;
        let center_y = (angle.sin() * radius) as i32;

        for _ in 0..civilians_per_faction {
            let grid_x = center_x + rng.gen_range(-quadrant_spread..=quadrant_spread);
            let grid_z = center_y + rng.gen_range(-quadrant_spread..=quadrant_spread);
            let norm_x = (grid_x as f32 / scale).clamp(0.0, 1.0);
            let norm_y = (grid_z as f32 / scale).clamp(0.0, 1.0);
            spawn_civilian_at(
                world,
                next_civilian_id,
                Alignment::Faction(faction),
                norm_x,
                norm_y,
                ActorVisualKind::Humanoid,
                rng,
            );
            next_civilian_id += 1;
        }
    }
}

// ---------------------------------------------------------------------------
// Simulation methods (impl block)
// ---------------------------------------------------------------------------

use crate::engine::{KinshipEdge, KinshipKind};

impl Simulation {
    /// Helper: compute resource pressure from food stockpiles.
    pub(crate) fn resource_pressure(&self) -> f32 {
        let food = self.state.resources.food.to_bits().max(0) as f32;
        let pressure = if food <= 0.0_f32 {
            1.0
        } else {
            (1.0_f32 / (1.0 + food / 250.0)).clamp(0.0, 1.0)
        };
        pressure
    }

    /// Helper: compute unrest pressure from the latest unrest snapshots.
    pub(crate) fn unrest_pressure(&self) -> f32 {
        let max_unrest = self
            .last_tick_unrest_snapshots
            .values()
            .map(|snapshot| snapshot.score.max(0))
            .max()
            .unwrap_or(0) as f32;
        (max_unrest / 500.0).clamp(0.0, 1.0)
    }

    /// Derive the next settlement id from the current settlement map.
    pub(crate) fn next_settlement_id(&self) -> u32 {
        self.settlements
            .keys()
            .copied()
            .max()
            .unwrap_or(0)
            .saturating_add(1)
    }

    /// Extended lifecycle phase: pairing, birth, death, migration,
    /// and lifecycle metric rollup (FR-CIV-LIFE P4-A).
    pub(crate) fn phase_life(&mut self) {
        attach_citizen_to_agents(&mut self.world);
        self.last_births.clear();
        self.last_deaths.clear();

        let mut records: Vec<(Entity, u64, CivilianLifecycleSample)> = self
            .world
            .query::<(&AgentCivilian, &Position3d, &Needs)>()
            .iter()
            .map(|(entity, (civilian, pos, needs))| {
                (
                    entity,
                    civilian.id,
                    CivilianLifecycleSample {
                        age: civilian.age,
                        alignment: civilian.alignment,
                        x: pos.coord.x as f32 / FIXED_SCALE as f32,
                        y: pos.coord.z as f32 / FIXED_SCALE as f32,
                        fertility_score: fertility_score(civilian.age, needs),
                        migration_pressure: migration_pressure(needs, self.resource_pressure()),
                    },
                )
            })
            .collect();
        records.sort_by_key(|(_, id, _)| *id);

        let mut dead = Vec::new();
        let mut births = Vec::new();
        let mut paired_adults: BTreeSet<u64> = BTreeSet::new();
        let mut found_new_settlements = Vec::new();
        let mut next_settlement_id = self.next_settlement_id();

        for (entity, id, sample) in records.iter() {
            let next_age = {
                let Ok(mut civilian) = self.world.get::<&mut AgentCivilian>(*entity) else {
                    continue;
                };
                civilian.age = civilian.age.saturating_add(1);
                civilian.age
            };
            let Ok(mut needs) = self.world.get::<&mut Needs>(*entity) else {
                continue;
            };
            apply_age_stage_effects(next_age, &mut needs);

            if sample.alignment == Alignment::None {
                continue;
            }

            if sample.age >= 65 && needs.health <= 0.15 {
                dead.push((*entity, *id, sample.x, sample.y));
            }
        }

        for (left_idx, left) in records.iter().enumerate() {
            if paired_adults.contains(&left.1) {
                continue;
            }
            if !is_fertile_adult(left.0, &self.world, &left.2) {
                continue;
            }

            let mut partner: Option<&(Entity, u64, CivilianLifecycleSample)> = None;
            for right in records.iter().skip(left_idx + 1) {
                if paired_adults.contains(&right.1) {
                    continue;
                }
                if !is_fertile_adult(right.0, &self.world, &right.2) {
                    continue;
                }
                if left.2.alignment != right.2.alignment {
                    continue;
                }
                if lifecycle_distance(left.2.x, left.2.y, right.2.x, right.2.y) > 0.04 {
                    continue;
                }
                partner = Some(right);
                break;
            }

            let Some(right) = partner else {
                continue;
            };

            let birth_pressure = ((left.2.fertility_score + right.2.fertility_score) * 0.5)
                * (1.0
                    - left
                        .2
                        .migration_pressure
                        .max(right.2.migration_pressure)
                        .clamp(0.0, 1.0));
            if birth_pressure < 0.68 {
                continue;
            }

            paired_adults.insert(left.1);
            paired_adults.insert(right.1);

            let child_id = self.next_civilian_id;
            self.next_civilian_id += 1;
            let x = ((left.2.x + right.2.x) * 0.5).clamp(0.01, 0.99);
            let y = ((left.2.y + right.2.y) * 0.5).clamp(0.01, 0.99);
            births.push((child_id, x, y, left.2.alignment, left.1, right.1));
        }

        for (entity, id, x, y) in dead.iter().copied() {
            let _ = self.world.despawn(entity);
            self.last_deaths.push(PopulationEvent {
                tick: self.state.tick,
                entity_id: id,
                x,
                y,
            });
        }

        let pressure = self.resource_pressure().max(self.unrest_pressure());
        if pressure >= 0.55 {
            let mut grouped: BTreeMap<u32, Vec<(Entity, u64, CivilianLifecycleSample)>> =
                BTreeMap::new();
            for (entity, id, sample) in &records {
                if paired_adults.contains(id) {
                    continue;
                }
                if !is_migratory_adult(*entity, &self.world, sample) {
                    continue;
                }
                let settlement_id = match sample.alignment {
                    Alignment::Faction(fid) => fid,
                    _ => 0,
                };
                grouped
                    .entry(settlement_id)
                    .or_default()
                    .push((*entity, *id, sample.clone()));
            }

            for (settlement_id, mut candidates) in grouped {
                candidates.sort_by_key(|(_, id, _)| *id);
                let source_population = self.settlements.get(&settlement_id).copied().unwrap_or(0);
                if candidates.len() < 2 || source_population < 2 {
                    continue;
                }

                let migration_count = if pressure >= 0.8 {
                    candidates.len().min(3)
                } else {
                    candidates.len().min(2)
                };
                let new_settlement_id = next_settlement_id;
                next_settlement_id = next_settlement_id.saturating_add(1);
                found_new_settlements.push((
                    settlement_id,
                    new_settlement_id,
                    migration_count as u32,
                ));

                for (entity, id, mut sample) in candidates.into_iter().take(migration_count) {
                    if let Ok(mut civilian) = self.world.get::<&mut AgentCivilian>(entity) {
                        civilian.alignment = Alignment::Faction(new_settlement_id);
                    }
                    if let Ok(mut pos) = self.world.get::<&mut Position3d>(entity) {
                        let (nx, ny) = settlement_anchor_for(new_settlement_id, sample.x, sample.y);
                        pos.coord.x = (nx * FIXED_SCALE as f32) as i64;
                        pos.coord.z = (ny * FIXED_SCALE as f32) as i64;
                        sample.x = nx;
                        sample.y = ny;
                    }
                }
            }
        }

        for (child_id, x, y, alignment, parent_a, parent_b) in births {
            let _ = spawn_child_near(&mut self.world, child_id, alignment, x, y, &mut self.rng);
            self.last_births.push(PopulationEvent {
                tick: self.state.tick,
                entity_id: child_id,
                x,
                y,
            });
            self.register_kinship(
                child_id,
                KinshipEdge {
                    kind: KinshipKind::Family,
                    target: parent_a,
                },
            );
            self.register_kinship(
                child_id,
                KinshipEdge {
                    kind: KinshipKind::Family,
                    target: parent_b,
                },
            );
            self.register_kinship(
                parent_a,
                KinshipEdge {
                    kind: KinshipKind::Family,
                    target: child_id,
                },
            );
            self.register_kinship(
                parent_b,
                KinshipEdge {
                    kind: KinshipKind::Family,
                    target: child_id,
                },
            );
            paired_adults.insert(parent_a);
            paired_adults.insert(parent_b);
        }

        for (source_settlement_id, new_settlement_id, count) in found_new_settlements {
            let source = self.settlements.entry(source_settlement_id).or_insert(0);
            *source = source.saturating_sub(count);
            self.settlements.insert(new_settlement_id, count);
        }

        let births_count = self.last_births.len() as u64;
        let deaths_count = self.last_deaths.len() as u64;
        self.state.population = self.state.population.saturating_add(births_count);
        self.state.population = self.state.population.saturating_sub(deaths_count);

        // FR-CIV-LIFE P4-A: compute per-tick lifecycle metrics (children /
        // adults / elders / dead) so phase_economy can derive aggregate
        // labor fraction. Uses LifecycleLabel from civ_needs. Children are
        // tagged by age; elders by age >= 65. Dead civilians come from the
        // `dead` despawn list captured earlier this tick.
        let mut metrics = LifecycleCounters::default();
        for (_entity, _id, sample) in records.iter() {
            // Use the existing fertility_score as a proxy for general
            // well-being (it is already a [0, 1] value derived from age and
            // needs). In CIV-003 P5-A this will be replaced with a
            // dedicated Health derivation; for now it gives deterministic
            // testable rollups.
            let integrity = sample.fertility_score.clamp(0.0, 1.0);
            let health = civ_needs::Health {
                integrity,
                ..civ_needs::Health::default()
            };
            // Maturity: read from the first civilian's Psyche if available,
            // otherwise default 0 (Child band).
            let maturity = self
                .world
                .query::<&civ_agents::Psyche>()
                .iter()
                .next()
                .map(|(_, p)| p.maturity)
                .unwrap_or(0.0);
            let labor_cap = civ_needs::labor_capacity(
                sample.age,
                &health,
                &civ_genetics::Dna::zero(0),
                &civ_needs::LifecycleParams::default(),
            );
            match civ_needs::classify_lifecycle(sample.age, &health, maturity, labor_cap) {
                civ_needs::LifecycleLabel::Child => metrics.children += 1,
                civ_needs::LifecycleLabel::Adult => metrics.adults += 1,
                civ_needs::LifecycleLabel::WorkingAge => metrics.adults += 1,
                civ_needs::LifecycleLabel::Elder => metrics.elders += 1,
                civ_needs::LifecycleLabel::Dead => metrics.dead += 1,
            }
        }
        // Dead tally from this tick's despawn list:
        metrics.dead = metrics.dead.saturating_add(dead.len() as u32);
        self.last_tick_lifecycle_metrics = metrics;

        // Emergent migration wiring TODO: wire when MigrationPlanner APIs stabilize
        // See crates/engine/src/emergent_migration.rs for migration_tick() API
        // Requires settlement_snapshots, agent_snapshots, migration_planner fields
        // on Simulation struct to be properly populated each tick.
    }

    /// Citizen lifecycle phase — aging, food consumption, birth gating,
    /// death, and population accounting.
    pub(crate) fn phase_citizen_lifecycle(&mut self) {
        attach_citizen_to_agents(&mut self.world);
        self.last_births.clear();
        self.last_deaths.clear();
        let population = civ_agents::count_civilians(&self.world) as f64;
        let max_pop = self.state.population.max(1) as f64;
        let overcrowding_factor = (population / max_pop).clamp(0.0, 1.0);
        // FR-CIV-LIFE-003: birth probability is now derived per-civilian from
        // `civ_needs::should_reproduce`, which consults the lifecycle label
        // (Adult only), the food/safety thresholds, and the configurable
        // `LifecycleParams` fertility curves.
        let lifecycle_params = civ_needs::LifecycleParams::default();
        let birth_window = self.state.tick % 200 == 0;
        let mut dead = Vec::new();
        let mut births = Vec::new();

        for (entity, (civilian, pos, needs)) in
            self.world
                .query_mut::<(&mut AgentCivilian, &Position3d, &mut Needs)>()
        {
            civilian.age = civilian.age.saturating_add(1);
            if self.state.resources.food.to_bits() > 0 {
                needs.food = (needs.food + 0.008).min(1.0);
                self.state.resources.food =
                    (self.state.resources.food - Fixed::from_num(1)).max(Fixed::ZERO);
            } else {
                needs.food = (needs.food - 0.03).max(0.0);
            }
            if needs.food < 0.05 && self.state.resources.food.to_bits() <= 0 {
                dead.push((entity, civilian.id, pos.coord));
                continue;
            }
            if birth_window && civilian.age > 18 {
                let health = civ_needs::Health {
                    integrity: ((needs.food + needs.shelter + needs.safety + needs.belonging)
                        * 0.25)
                        .clamp(0.0, 1.0),
                    ..civ_needs::Health::default()
                };
                let should_birth = civ_needs::should_reproduce(
                    civilian.age as f32,
                    &health,
                    needs.food,
                    needs.safety,
                    overcrowding_factor as f32,
                    &lifecycle_params,
                );
                if self.rng.gen_bool(should_birth.clamp(0.0, 1.0) as f64) {
                    let child_id = self.next_civilian_id;
                    self.next_civilian_id += 1;
                    let x = pos.coord.x as f32 / FIXED_SCALE as f32;
                    let y = pos.coord.z as f32 / FIXED_SCALE as f32;
                    births.push((child_id, x, y));
                }
            }
        }

        for (child_id, x, y) in births {
            let _ = spawn_child_near(
                &mut self.world,
                child_id,
                Alignment::None,
                x,
                y,
                &mut self.rng,
            );
            self.last_births.push(PopulationEvent {
                tick: self.state.tick,
                entity_id: child_id,
                x,
                y,
            });
        }

        for (entity, entity_id, coord) in dead {
            let _ = self.world.despawn(entity);
            self.last_deaths.push(PopulationEvent {
                tick: self.state.tick,
                entity_id,
                x: coord.x as f32 / FIXED_SCALE as f32,
                y: coord.z as f32 / FIXED_SCALE as f32,
            });
        }

        let births_count = self.last_births.len() as u64;
        let deaths_count = self.last_deaths.len() as u64;
        self.last_life_deaths = deaths_count as u32;
        self.state.population = self.state.population.saturating_add(births_count);
        self.state.population = self.state.population.saturating_sub(deaths_count);
    }
}
