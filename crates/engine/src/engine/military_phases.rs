//! War/combat resolution and military phase methods extracted from engine.rs
//! (Pass 8 — Civis Engine Decomposition).

use crate::engine::{CombatDamagePulse, Simulation};
use crate::fixed_math::Fixed;
use civ_tactics::{
    apply_damage, evolve_doctrine, score_doctrine_fitness, tick_operational_movement,
    tick_war_bridge, Doctrine, DoctrineLibrary, FactionEngagementStats, MilitaryUnitSample,
    OperationalLayer,
};
use civ_voxel::FIXED_SCALE;
use hecs::Entity;
use rand::Rng;
use rand::SeedableRng;
use rand_chacha::ChaCha8Rng;

/// Default doctrine population for three factions (deterministic seed layout).
#[inline]
pub(crate) fn default_faction_doctrines() -> Vec<DoctrineLibrary> {
    (0..3)
        .map(|faction| DoctrineLibrary {
            generation: 0,
            current: vec![
                Doctrine {
                    id: faction as u64 * 10 + 1,
                    unit_composition: vec![10, 5, 2],
                    score: 0.5,
                },
                Doctrine {
                    id: faction as u64 * 10 + 2,
                    unit_composition: vec![8, 8, 4],
                    score: 0.8,
                },
            ],
        })
        .collect()
}

impl Simulation {
    /// Tactics phase - evolve faction doctrines and apply queued voxel damage.
    pub(crate) fn phase_tactics(&mut self) {
        self.last_tick_voxel_damage_count = 0;
        let scale = FIXED_SCALE as f32;
        for event in self.pending_damage.drain(..) {
            let x = (event.center.x as f32 / scale).clamp(0.0, 1.0);
            let y = (event.center.z as f32 / scale).clamp(0.0, 1.0);
            let has_pulse = self.last_tick_combat_pulses.iter().any(|pulse| {
                (pulse.x - x).abs() < f32::EPSILON && (pulse.y - y).abs() < f32::EPSILON
            });
            if !has_pulse {
                self.last_tick_combat_pulses.push(CombatDamagePulse {
                    x,
                    y,
                    unit_a: None,
                    unit_b: None,
                });
            }
            self.last_tick_voxel_damage_count += apply_damage(&mut self.voxel, &event);
        }

        const DOCTRINE_EVOLVE_MODULO: u64 = 64;
        if self.state.tick % DOCTRINE_EVOLVE_MODULO == 0 {
            let mut faction_stats =
                vec![FactionEngagementStats::default(); self.faction_doctrines.len()];
            for engagement in &self.last_tick_engagements {
                let shooter = engagement.shooter_faction as usize;
                let target = engagement.target_faction as usize;
                if shooter < faction_stats.len() {
                    faction_stats[shooter].engagements_as_shooter = faction_stats[shooter]
                        .engagements_as_shooter
                        .saturating_add(1);
                }
                if target < faction_stats.len() {
                    faction_stats[target].engagements_as_target = faction_stats[target]
                        .engagements_as_target
                        .saturating_add(1);
                }
            }
            if self.last_tick_voxel_damage_count > 0 && !self.last_tick_engagements.is_empty() {
                let per_shooter = (self.last_tick_voxel_damage_count as u32)
                    .saturating_div(self.last_tick_engagements.len() as u32)
                    .max(1);
                for engagement in &self.last_tick_engagements {
                    let shooter = engagement.shooter_faction as usize;
                    if shooter < faction_stats.len() {
                        faction_stats[shooter].voxels_removed = faction_stats[shooter]
                            .voxels_removed
                            .saturating_add(per_shooter);
                    }
                }
            }
            for (faction, library) in self.faction_doctrines.iter_mut().enumerate() {
                let stats = faction_stats.get(faction).copied().unwrap_or_default();
                for doctrine in &mut library.current {
                    doctrine.score = score_doctrine_fitness(doctrine, &stats);
                }
                let mut rng = ChaCha8Rng::seed_from_u64(
                    self.state.rng_seed ^ self.state.tick ^ u64::from(faction as u32),
                );
                evolve_doctrine(library, &mut rng, 0.2);
            }
        }
    }

    // Moved to species_lifecycle.rs
    /// Military phase — morale recovery and Phase-4 war → tactics bridge.
    pub(crate) fn phase_military(&mut self) {
        use crate::spawn::military_pin_id;

        let tick = self.state.tick;
        let lines = self.mod_host.military_tick(tick);
        self.ingest_mod_phase_lines(lines, tick, "military");

        let phase_cfg = self.military_phase;

        // PERF: snapshot every `MilitaryUnit` once. The previous code
        // called `world.query::<&MilitaryUnit>().iter()` five times per
        // tick (morale / samples / movement-update / damage-update / dead),
        // and two of those were nested inside the engagement/movement
        // loops — turning what should be O(n) into O(n*m). Holding the
        // snapshot lets us do all mutation against `units` (Vec indexed
        // by the same position as `entities`) and write back in one
        // final pass. Movement/damage updates become O(1) via the
        // already-computed index.
        let (mut entities, mut units): (Vec<Entity>, Vec<crate::engine::MilitaryUnit>) = self
            .world
            .query::<&crate::engine::MilitaryUnit>()
            .iter()
            .map(|(e, u)| (e, u.clone()))
            .unzip();

        // 1. Morale recovery.
        for unit in &mut units {
            if unit.morale < Fixed::from_num(1) {
                unit.morale = (unit.morale + Fixed::from_num(1) / Fixed::from_num(100))
                    .min(Fixed::from_num(1));
            }
        }

        // 2. Build per-unit samples for the war/tactics bridge.
        let mut samples: Vec<MilitaryUnitSample> = entities
            .iter()
            .zip(units.iter())
            .enumerate()
            .map(|(idx, (&entity, unit))| MilitaryUnitSample {
                unit_id: military_pin_id(entity, idx),
                faction_id: unit.faction_id,
                grid_x: unit.position.x,
                grid_y: unit.position.y,
            })
            .collect();

        // 3. Operational movement — mutate both the sample (for the bridge
        // downstream) and the unit snapshot (for write-back at the end).
        for grid_move in tick_operational_movement(
            self.state.tick,
            &phase_cfg.movement,
            &mut samples,
            phase_cfg.movement_pulses_per_cadence,
            &self.voxel,
        ) {
            if let Some(sample) = samples.get_mut(grid_move.unit_index) {
                sample.grid_x = grid_move.new_grid_x;
                sample.grid_y = grid_move.new_grid_y;
            }
            if let Some(unit) = units.get_mut(grid_move.unit_index) {
                unit.position.x = grid_move.new_grid_x;
                unit.position.y = grid_move.new_grid_y;
            }
        }

        // 4. War bridge / engagements.
        let config = phase_cfg.war;
        let fog = civ_tactics::build_fog_for_units(&config, &samples, &self.voxel);
        let engagements = tick_war_bridge(
            self.state.tick,
            &config,
            &samples,
            &self.voxel,
            fog.as_ref(),
        );
        self.operational
            .on_combat_engagements(self.state.tick, &engagements);
        self.last_tick_engagements = engagements.clone();

        let hp_loss = Fixed::from_num(config.strength_damage_fixed);
        let scale = FIXED_SCALE as f32;
        for engagement in &engagements {
            self.replay_log.record_combat(
                self.state.tick,
                engagement.shooter_id,
                engagement.target_id,
                engagement.damage,
            );
            // Apply damage to the snapshot directly (O(1) by index, no
            // re-query of the world).
            if let Some(unit) = units.get_mut(engagement.target_index) {
                unit.hp = (unit.hp - hp_loss).max(Fixed::from_num(0));
                unit.strength = unit.hp;
            }
            self.last_tick_combat_pulses.push(CombatDamagePulse {
                x: (engagement.damage.center.x as f32 / scale).clamp(0.0, 1.0),
                y: (engagement.damage.center.z as f32 / scale).clamp(0.0, 1.0),
                unit_a: Some(engagement.shooter_id),
                unit_b: Some(engagement.target_id),
            });
            self.pending_damage.push(engagement.damage);
        }

        // 5. Single write-back pass: despawn dead units and re-insert
        // survivors. `swap_remove` keeps the snapshot and entity list
        // aligned so the engagement indices from step 4 still line up.
        let mut i = 0;
        while i < entities.len() {
            if units[i].hp <= Fixed::from_num(0) {
                let _ = self.world.despawn(entities[i]);
                entities.swap_remove(i);
                units.swap_remove(i);
            } else {
                let _ = self.world.insert(entities[i], (units[i].clone(),));
                i += 1;
            }
        }
    }

    /// Configure fog-of-war parameters for the military phase.
    pub fn configure_military_fog(&mut self, vision_radius: Option<u32>, grid_size: u32) {
        if let Some(radius) = vision_radius {
            self.military_phase.war.fog_vision_radius = Some(radius);
            self.military_phase.war.fog_grid_size = grid_size.max(16);
        }
    }

    /// Apply scenario military cadence/combat overrides (FR-CIV-TACTICS-050).
    pub fn apply_scenario_military(&mut self, military: &crate::scenario::ScenarioMilitary) {
        if let Some(v) = military.movement_cadence_ticks {
            self.military_phase.movement.cadence_ticks = v;
        }
        if let Some(v) = military.movement_pulses_per_cadence {
            self.military_phase.movement_pulses_per_cadence = v;
        }
        if let Some(v) = military.war_cadence_ticks {
            self.military_phase.war.cadence_ticks = v;
        }
        if let Some(v) = military.engage_range_grid {
            self.military_phase.war.engage_range_grid = v.max(1);
        }
    }

    /// Store scenario taxation settings for later economy-phase wiring.
    pub fn apply_scenario_taxation(&mut self, taxation: &crate::scenario::ScenarioTaxation) {
        // Translate the scenario representation into the engine's
        // `civ_economy::Taxation` field. The scenario struct is a
        // wire-friendly shape; the engine keeps a `Taxation` that the
        // economy phase consumes directly.
        let mut resolved = civ_economy::Taxation::default();
        for (institution_id, rate_bp) in &taxation.rates_bp {
            if let Ok(id) = (*institution_id).try_into() {
                resolved.rates_bp.insert(id, *rate_bp);
            }
        }
        resolved.per_institution_cap = taxation
            .per_institution_cap
            .and_then(|cap| (cap >= 0).then_some(cap));
        self.scenario_taxation = resolved;
    }

    /// Military phase configuration (tests and tooling).
    #[must_use]
    pub fn military_phase_config(&self) -> &civ_tactics::MilitaryPhaseConfig {
        &self.military_phase
    }
}
