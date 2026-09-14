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

/// Simulation ticks are daily; chronological age advances once per in-game
/// year rather than once per frame.
const LIFECYCLE_YEAR_TICKS: u64 = 365;

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
#[inline]
pub(crate) fn civilian_to_health(needs: &Needs) -> civ_needs::Health {
    let integrity =
        ((needs.food + needs.shelter + needs.safety + needs.belonging) * 0.25).clamp(0.0, 1.0);
    civ_needs::Health {
        integrity,
        ..civ_needs::Health::default()
    }
}

#[inline]
pub(crate) fn lifecycle_distance(ax: f32, ay: f32, bx: f32, by: f32) -> f32 {
    let dx = ax - bx;
    let dy = ay - by;
    (dx * dx + dy * dy).sqrt()
}

#[inline]
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

#[inline]
pub(crate) fn migration_pressure(needs: &Needs, resource_pressure: f32) -> f32 {
    let deprivation =
        1.0 - ((needs.food + needs.rest + needs.safety + needs.belonging) * 0.25).clamp(0.0, 1.0);
    (0.7 * deprivation + 0.3 * resource_pressure).clamp(0.0, 1.0)
}

/// Cadence-sensitive stage effects applied once per in-game year (every
/// [`LIFECYCLE_YEAR_TICKS`] ticks): children accrue belonging/safety at a slow
/// social-development rate and working-age adults get a small food bump.
/// Mature/elderly physiological decay intentionally lives in
/// [`apply_elder_decay_per_tick`] so it remains on the daily-cadence timeline
/// independent of age-increment cadence.
#[inline]
pub(crate) fn apply_age_stage_effects(age: u16, needs: &mut Needs) {
    if age < 18 {
        needs.belonging = (needs.belonging + 0.01).min(1.0);
        needs.safety = (needs.safety + 0.01).min(1.0);
    } else if age < 50 {
        needs.food = (needs.food + 0.005).min(1.0);
    }
    // Elder (age >= 50) decay moved to apply_elder_decay_per_tick.
}

/// Daily-cadence elder physiological decay: rest and health slowly decline on
/// every tick for mature adults (age >= 50). This is split out from
/// [`apply_age_stage_effects`] so the slowing rest/health of elders stays
/// proportional to the daily tick loop instead of being throttled to
/// once-per-year alongside chronological age increments (see
/// [`LIFECYCLE_YEAR_TICKS`]).
#[inline]
pub(crate) fn apply_elder_decay_per_tick(age: u16, needs: &mut Needs) {
    if age >= 50 {
        needs.rest = (needs.rest - 0.01).max(0.0);
        needs.health = (needs.health - 0.01).max(0.0);
    }
}

#[inline]
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

#[inline]
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

#[inline]
pub(crate) fn settlement_anchor_for(settlement_id: u32, x: f32, y: f32) -> (f32, f32) {
    let seed = settlement_id as f32 * 0.137_503_2;
    let nx = (x + seed.sin() * 0.08).clamp(0.05, 0.95);
    let ny = (y + seed.cos() * 0.08).clamp(0.05, 0.95);
    (nx, ny)
}

/// Deterministic job assignment for agent civilians (stable across seeds).
#[inline]
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
#[inline]
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
    #[inline]
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
    #[inline]
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
    #[inline]
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

        // PERF: hoist `resource_pressure()` out of the per-civilian
        // collection loop. The pressure only depends on the global food
        // stock, not on the civilian being inspected, so caching the
        // value once saves N virtual method calls per tick.
        let resource_pressure_cached = self.resource_pressure();
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
                        migration_pressure: migration_pressure(needs, resource_pressure_cached),
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

        let ages_this_tick = self.state.tick.is_multiple_of(LIFECYCLE_YEAR_TICKS);
        for (entity, id, sample) in records.iter() {
            let next_age = sample.age;
            let Ok(mut needs) = self.world.get::<&mut Needs>(*entity) else {
                continue;
            };
            // Elder rest/health decay runs every tick (daily-cadence): the
            // physiological decay of mature adults must not be throttled to
            // once-per-year alongside age increments.
            apply_elder_decay_per_tick(next_age, &mut needs);
            if ages_this_tick {
                apply_age_stage_effects(next_age, &mut needs);
            }

            if sample.alignment == Alignment::None {
                continue;
            }

            if sample.age >= 65 && needs.health <= 0.15 {
                dead.push((*entity, *id, sample.x, sample.y));
            }
        }

        // Evaluate reproduction on the existing 200-tick cadence. Pairing on
        // every frame lets the same adults reproduce indefinitely, exhausting
        // food and destabilizing normal emergence runs.
        let birth_window = self.state.tick == 0 || self.state.tick.is_multiple_of(200);
        if birth_window {
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
        //
        // PERF: read the dominant maturity once (was previously a per-civilian
        // `world.query::<&Psyche>().iter().next()` call that turned the metrics
        // pass into O(n²) over the civilian population).
        let maturity: f32 = self
            .world
            .query::<&civ_agents::Psyche>()
            .iter()
            .next()
            .map(|(_, p)| p.maturity)
            .unwrap_or(0.0);
        let labor_cap_zero_dna = civ_genetics::Dna::zero(0);
        let lifecycle_params_default = civ_needs::LifecycleParams::default();
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
            let labor_cap = civ_needs::labor_capacity(
                sample.age,
                &health,
                &labor_cap_zero_dna,
                &lifecycle_params_default,
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
        let population = civ_agents::count_civilians(&self.world) as u64;
        // Each civilian consumes one food unit below. Snapshot the complete
        // daily rations before consumption, independently of population accounting.
        // Replenishing food raises this capacity on the next tick.
        let daily_food_capacity = self.state.resources.food.max(Fixed::ZERO).to_num::<u64>();
        let overcrowding_factor = if daily_food_capacity == 0 {
            1.0
        } else {
            (population as f32 / daily_food_capacity as f32).clamp(0.0, 1.0)
        };
        let over_capacity = population > daily_food_capacity;
        // FR-CIV-LIFE-003: birth probability is now derived per-civilian from
        // `civ_needs::should_reproduce`, which consults the lifecycle label
        // (Adult only), the food/safety thresholds, and the configurable
        // `LifecycleParams` fertility curves.
        let lifecycle_params = civ_needs::LifecycleParams::default();
        let birth_window = self.state.tick.is_multiple_of(200);
        let mut dead = Vec::new();
        let mut births = Vec::new();

        for (entity, (civilian, pos, needs)) in
            self.world
                .query_mut::<(&mut AgentCivilian, &Position3d, &mut Needs)>()
        {
            if self.state.tick.is_multiple_of(LIFECYCLE_YEAR_TICKS) {
                civilian.age = civilian.age.saturating_add(1);
            }
            if self.state.resources.food >= Fixed::ONE {
                needs.food = (needs.food + 0.008).min(1.0);
                self.state.resources.food -= Fixed::ONE;
            } else {
                needs.food = (needs.food - 0.03).max(0.0);
            }
            // A cohort above authoritative capacity competes for the same
            // stockpile; apply bounded pressure before declaring famine.
            if over_capacity {
                needs.food = (needs.food - 0.012).max(0.0);
            }
            if needs.food < 0.05 && (self.state.resources.food.to_bits() <= 0 || over_capacity) {
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
                    overcrowding_factor,
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

#[cfg(test)]
mod age_stage_cadence_tests {
    use super::{apply_age_stage_effects, apply_elder_decay_per_tick};
    use civ_agents::Needs;

    fn fresh_needs() -> Needs {
        Needs {
            food: 0.5,
            shelter: 0.5,
            safety: 0.5,
            belonging: 0.5,
            rest: 0.5,
            health: 0.5,
        }
    }

    /// FR-CIV-LIFE / engine-cadence — `apply_age_stage_effects` no longer
    /// mutates elder `rest` / `health`. The age-tiered bumps (child
    /// belonging/safety, adult food) remain on the annual cadence; the
    /// elder physiological decay moved to [`apply_elder_decay_per_tick`]
    /// so it tracks the daily tick loop instead of being throttled to
    /// once-per-year.
    #[test]
    fn apply_age_stage_effects_is_now_purely_annual_and_does_not_decay_elder_needs() {
        let mut elder = fresh_needs();
        apply_age_stage_effects(70, &mut elder);
        // No decay in the annual pass for an elder: rest/health must be
        // untouched (the decay was relocated to the daily helper).
        assert!(
            (elder.rest - 0.5).abs() < f32::EPSILON,
            "annual stage pass must not decay elder rest, got {}",
            elder.rest
        );
        assert!(
            (elder.health - 0.5).abs() < f32::EPSILON,
            "annual stage pass must not decay elder health, got {}",
            elder.health
        );
    }

    /// FR-CIV-LIFE / engine-cadence — `apply_elder_decay_per_tick` runs every
    /// tick and drains elder rest/health at the documented rate until they
    /// hit 0.0 (the clamp we already had in the original `apply_age_stage_effects`
    /// else-branch). Adult needs untouched.
    #[test]
    fn apply_elder_decay_per_tick_drains_rest_and_health_each_tick() {
        let mut elder = fresh_needs();
        // Single call: rest + health -0.01, others untouched.
        apply_elder_decay_per_tick(70, &mut elder);
        assert!(
            (elder.rest - 0.49).abs() < 1e-6,
            "elder rest must drop 0.01 per tick, got {}",
            elder.rest
        );
        assert!(
            (elder.health - 0.49).abs() < 1e-6,
            "elder health must drop 0.01 per tick, got {}",
            elder.health
        );
        assert!(
            (elder.food - 0.5).abs() < f32::EPSILON,
            "elder food must not move, got {}",
            elder.food
        );

        // Drive rest to clamp: 100 calls × 0.01 = 1.00 of decay from 0.5,
        // rest floors at 0.0.
        let mut drained = fresh_needs();
        for _ in 0..100 {
            apply_elder_decay_per_tick(70, &mut drained);
        }
        assert!(
            drained.rest <= 0.0,
            "elder rest must clamp at 0.0, got {}",
            drained.rest
        );
        assert!(
            drained.health <= 0.0,
            "elder health must clamp at 0.0, got {}",
            drained.health
        );
    }

    /// FR-CIV-LIFE / engine-cadence — adult needs (18-49) untouched by the
    /// daily decay helper; only elder needs move. Guards against accidental
    /// pan-handling of the lower age branch.
    #[test]
    fn apply_elder_decay_per_tick_skips_adults_and_children() {
        let mut adult = fresh_needs();
        apply_elder_decay_per_tick(40, &mut adult);
        assert!(
            (adult.rest - 0.5).abs() < f32::EPSILON,
            "adult rest must not decay, got {}",
            adult.rest
        );
        assert!(
            (adult.health - 0.5).abs() < f32::EPSILON,
            "adult health must not decay, got {}",
            adult.health
        );

        let mut child = fresh_needs();
        apply_elder_decay_per_tick(10, &mut child);
        assert!(
            (child.rest - 0.5).abs() < f32::EPSILON,
            "child rest must not decay, got {}",
            child.rest
        );
        assert!(
            (child.health - 0.5).abs() < f32::EPSILON,
            "child health must not decay, got {}",
            child.health
        );
    }
}

#[cfg(test)]
mod daily_food_capacity_tests {
    use super::*;
    use civ_voxel::WorldCoord;

    fn cohort(food: Fixed, recorded_population: u64) -> Simulation {
        let mut sim = Simulation::with_seed(171);
        sim.world = World::new();
        sim.state.tick = 1; // Exclude birth and annual-age cadence.
        sim.state.population = recorded_population;
        sim.state.resources.food = food;
        for id in [1, 2] {
            sim.world.spawn((
                AgentCivilian {
                    id,
                    age: 25,
                    alignment: Alignment::Faction(1),
                },
                Position3d {
                    coord: WorldCoord {
                        x: id as i64,
                        y: 0,
                        z: 0,
                    },
                },
                Needs {
                    food: 0.5,
                    shelter: 0.5,
                    safety: 0.5,
                    belonging: 0.5,
                    rest: 0.5,
                    health: 0.5,
                },
            ));
        }
        sim
    }

    fn food_needs(sim: &Simulation) -> BTreeMap<u64, f32> {
        sim.world
            .query::<(&AgentCivilian, &Needs)>()
            .iter()
            .map(|(_, (agent, needs))| (agent.id, needs.food))
            .collect()
    }

    #[test]
    fn life_phase_decays_elder_health_on_non_annual_ticks() {
        let mut sim = cohort(Fixed::from_num(100), 2);
        for (_, agent) in sim.world.query_mut::<&mut AgentCivilian>() {
            agent.age = 70;
        }
        for tick in [1, 2] {
            sim.state.tick = tick;
            sim.phase_life();
        }
        for (_, (agent, needs)) in sim.world.query::<(&AgentCivilian, &Needs)>().iter() {
            assert_eq!(agent.age, 70);
            assert!((needs.health - 0.48).abs() < 1e-6);
            assert!((needs.rest - 0.48).abs() < 1e-6);
        }
        assert_eq!(civ_agents::count_civilians(&sim.world), 2);
    }

    #[test]
    fn ration_capacity_is_independent_of_recorded_population_and_uses_whole_units() {
        for food in [
            Fixed::from_num(1),
            Fixed::from_bits(1999),
            Fixed::from_num(2),
        ] {
            let mut exact_count = cohort(food, 2);
            let mut stale_count = cohort(food, 2000);
            exact_count.phase_citizen_lifecycle();
            stale_count.phase_citizen_lifecycle();
            assert_eq!(food_needs(&exact_count), food_needs(&stale_count));
            assert_eq!(exact_count.last_deaths(), stale_count.last_deaths());
            let maximum = food_needs(&exact_count)
                .values()
                .copied()
                .fold(0.0_f32, f32::max);
            let expected = if food < Fixed::from_num(2) {
                0.496
            } else {
                0.508
            };
            assert!(
                (maximum - expected).abs() < 1e-6,
                "daily pressure must start below two complete rations"
            );
            assert_eq!(exact_count.state.population, 2);
            assert_eq!(stale_count.state.population, 2000);
        }
    }

    #[test]
    fn feeding_requires_whole_rations_and_preserves_fractional_stock() {
        for (bits, fed, remaining) in [
            (0, 0, 0),
            (1, 0, 1),
            (999, 0, 999),
            (1000, 1, 0),
            (1999, 1, 999),
            (2000, 2, 0),
        ] {
            let mut sim = cohort(Fixed::from_bits(bits), 2);
            sim.phase_citizen_lifecycle();
            let mut actual: Vec<_> = food_needs(&sim).values().copied().collect();
            actual.sort_by(f32::total_cmp);
            let pressure = if fed < 2 { 0.012 } else { 0.0 };
            let mut expected = vec![0.5 - 0.03 - pressure; 2 - fed];
            expected.extend(vec![0.5 + 0.008 - pressure; fed]);
            for (actual, expected) in actual.iter().zip(&expected) {
                assert!(
                    (actual - expected).abs() < 1e-6,
                    "food bits {bits}: {actual} != {expected}"
                );
            }
            assert_eq!(actual.len(), 2);
            assert_eq!(sim.state.resources.food, Fixed::from_bits(remaining));
        }
    }

    #[test]
    fn replenishing_food_removes_capacity_pressure_next_tick() {
        let mut sim = cohort(Fixed::from_num(1), 2);
        sim.phase_citizen_lifecycle();
        let before = food_needs(&sim);
        sim.state.resources.food = Fixed::from_num(4);
        sim.phase_citizen_lifecycle();
        for (id, food) in food_needs(&sim) {
            assert!((food - before[&id] - 0.008).abs() < 1e-6);
        }
    }

    #[test]
    fn empty_rations_cause_famine_and_update_population_by_death_events() {
        let mut sim = cohort(Fixed::ZERO, 2);
        for (_, needs) in sim.world.query_mut::<&mut Needs>() {
            needs.food = 0.06;
        }
        sim.phase_citizen_lifecycle();
        assert_eq!(sim.last_deaths().len(), 2);
        assert_eq!(sim.state.population, 0);
        assert_eq!(civ_agents::count_civilians(&sim.world), 0);
    }
}
