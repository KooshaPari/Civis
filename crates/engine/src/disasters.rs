//! Disaster effects for the simulation world.
//!
//! The API is intentionally thin: callers can trigger a named disaster at a
//! world coordinate, and the engine can expose a phase hook for future
//! scheduling without changing the core tick shape.

use civ_agents::Position3d;
use civ_needs::{Health as LifeHealth, Needs as LifeNeeds};
use civ_voxel::material::{AIR, GRAVEL, ICE, LAVA, STEAM, STONE, WATER};
use civ_voxel::WorldCoord;

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
    // Fear breeds faith: a disaster drives the surviving population to worship,
    // raising belief (emergent disasters -> faith coupling, FR-CIV-EMERGENCE).
    const DISASTER_FAITH_GAIN: u64 = 50;
    sim.add_belief(DISASTER_FAITH_GAIN);
}

impl Simulation {
    /// Invoke a divine disaster: spend `cost` belief to call down `kind` at
    /// `pos`. Returns `true` and triggers the disaster when enough faith has
    /// accumulated; returns `false` and does nothing otherwise. This is the
    /// player-facing divine power that closes the
    /// disasters → belief → divine-intervention loop (FR-CIV-EMERGENCE).
    pub fn invoke_divine_disaster(
        &mut self,
        kind: DisasterKind,
        pos: WorldCoord,
        cost: u64,
    ) -> bool {
        if self.try_invoke_divine_power(cost) {
            trigger_disaster(self, kind, pos);
            true
        } else {
            false
        }
    }

    /// Phase hook for disaster systems (FR-CIV-0100 §2).
    ///
    /// Disasters EMERGE from environmental state rather than being scripted:
    /// each weather cell whose conditions cross a physical ignition/onset
    /// threshold spawns the corresponding disaster at its region. Wildfires
    /// ignite under sustained extreme heat + low moisture; further kinds extend
    /// the same threshold-driven pattern.
    pub fn phase_disasters(&mut self) {
        /// Wildfire ignites at/above this air temperature (fixed-point milli-°C).
        const WILDFIRE_TEMP_FP: i32 = 40_000; // 40 °C
        /// ...and at/below this precipitation (fixed-point mm) — dry fuel.
        const WILDFIRE_PRECIP_FP: i32 = 200;
        /// Quake onset: tidal stress above this magnitude (lunar tide offset)...
        const QUAKE_TIDE_THRESHOLD: f32 = 0.9;
        /// ...co-located with a tectonically-active latitude (fixed-point deg).
        const QUAKE_LATITUDE_FP: i32 = 40_000;

        // Collect onset sites first so the immutable weather borrow is released
        // before we mutate the simulation via trigger_disaster. Disasters emerge
        // from physical state: heat+drought -> wildfire; tidal stress at a
        // tectonic latitude -> quake.
        let tidal_stress = self.climate.tide_offset.abs();
        // Research mitigates nature: fire-suppression tech raises the ignition
        // threshold (research -> fewer disasters). Computed before the weather
        // borrow so the immutable grow iteration holds no `&self` method call.
        let wildfire_temp_threshold = wildfire_ignition_temp_fp(WILDFIRE_TEMP_FP, self.research_tier());
        let mut wildfires = Vec::new();
        let mut quakes = Vec::new();
        for cell in &self.weather_grid {
            let pos = WorldCoord {
                x: i64::from(cell.region_id),
                y: 0,
                z: 0,
            };
            if cell.temp_c_fp >= wildfire_temp_threshold && cell.precip_mm_fp <= WILDFIRE_PRECIP_FP {
                wildfires.push(pos);
            }
            if tidal_stress >= QUAKE_TIDE_THRESHOLD && cell.latitude_fp.abs() >= QUAKE_LATITUDE_FP {
                quakes.push(pos);
            }
        }

        for pos in wildfires {
            trigger_disaster(self, DisasterKind::Wildfire, pos);
        }
        for pos in quakes {
            trigger_disaster(self, DisasterKind::Quake, pos);
        }
    }
}

/// Fire-suppression technology raises the temperature required to ignite a
/// wildfire: each research tier adds this many fixed-point milli-°C to the
/// ignition threshold.
const WILDFIRE_RESEARCH_MITIGATION_FP: u64 = 2_000; // +2 °C per tier
/// Cap on research mitigation so even an advanced civilisation can still burn
/// under sufficiently extreme heat — disasters are damped, never abolished.
const WILDFIRE_RESEARCH_MITIGATION_CAP_FP: u64 = 20_000; // +20 °C max

/// Downward-causation policy (FR-CIV-0100 §3 emergence): research mitigates
/// nature. Returns the effective wildfire ignition temperature given the base
/// physical threshold and the civilisation's research tier. At tier 0 it is the
/// raw physical threshold; higher tiers raise it (bounded), so wildfires become
/// rarer as technology advances — never impossible.
fn wildfire_ignition_temp_fp(base_fp: i32, research_tier: u64) -> i32 {
    // Arithmetic in u64 (the tier's natural type) so a huge tier saturates to
    // the cap instead of wrapping negative via an `as i64` cast.
    let bonus = research_tier
        .saturating_mul(WILDFIRE_RESEARCH_MITIGATION_FP)
        .min(WILDFIRE_RESEARCH_MITIGATION_CAP_FP) as i64;
    (base_fp as i64).saturating_add(bonus) as i32
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
            hit_agents(
                sim,
                pos,
                radius,
                DisasterEffect::new(0.28, 0.35, 0.25, 0.55, true),
            );
        }
        DisasterKind::Flood => {
            for cell in affected {
                sim.push_voxel_write(cell, WATER);
            }
            hit_agents(
                sim,
                pos,
                radius,
                DisasterEffect::new(0.10, 0.42, 0.20, 0.25, false),
            );
        }
        DisasterKind::Quake => {
            for (i, cell) in affected.iter().enumerate() {
                let material = if i % 7 == 0 { STONE } else { GRAVEL };
                sim.push_voxel_write(*cell, material);
            }
            hit_agents(
                sim,
                pos,
                radius,
                DisasterEffect::new(0.16, 0.30, 0.24, 0.20, false),
            );
        }
        DisasterKind::Wildfire => {
            for (i, cell) in affected.iter().enumerate() {
                let material = if i % 3 == 0 { LAVA } else { STEAM };
                sim.push_voxel_write(*cell, material);
            }
            hit_agents(
                sim,
                pos,
                radius,
                DisasterEffect::new(0.18, 0.46, 0.38, 0.20, true),
            );
        }
        DisasterKind::Storm => {
            for (i, cell) in affected.iter().enumerate() {
                let material = if i % 4 == 0 { ICE } else { WATER };
                sim.push_voxel_write(*cell, material);
            }
            hit_agents(
                sim,
                pos,
                radius,
                DisasterEffect::new(0.14, 0.20, 0.22, 0.12, false),
            );
        }
        DisasterKind::Plague => {
            hit_agents(
                sim,
                pos,
                radius * 2,
                DisasterEffect::new(0.05, 0.10, 0.18, 0.06, false),
            );
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

#[derive(Clone, Copy)]
struct DisasterEffect {
    shelter_delta: f32,
    safety_delta: f32,
    food_delta: f32,
    health_delta: f32,
    heat_damage: bool,
}

impl DisasterEffect {
    const fn new(
        shelter_delta: f32,
        safety_delta: f32,
        food_delta: f32,
        health_delta: f32,
        heat_damage: bool,
    ) -> Self {
        Self {
            shelter_delta,
            safety_delta,
            food_delta,
            health_delta,
            heat_damage,
        }
    }
}

fn hit_agents(sim: &mut Simulation, pos: WorldCoord, radius: i64, effect: DisasterEffect) {
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
                    needs.rest = (needs.rest - effect.shelter_delta).max(0.0);
                    needs.safety = (needs.safety - effect.safety_delta).max(0.0);
                    needs.food = (needs.food - effect.food_delta).max(0.0);
                    needs.health = (needs.health - effect.health_delta).max(0.0);
                }
                if let Ok(mut life_health) = sim.world.get::<&mut LifeHealth>(entity) {
                    let damage = if effect.heat_damage {
                        effect.health_delta * 0.5
                    } else {
                        effect.health_delta * 0.25
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
    use civ_agents::{Alignment, Civilian, LodTier, Position3d};
    use civ_needs::{Health as LifeHealth, Needs as LifeNeeds};
    use civ_planet::{Climate, WeatherCell, WeatherKind};

    fn seeded_sim() -> Simulation {
        Simulation::with_seed(7)
    }

    /// FR-CIV-EMERGENCE — a disaster raises belief (fear breeds faith): the
    /// disasters system feeds the divine-powers economy (downward causation).
    #[test]
    fn disaster_raises_belief_fear_breeds_faith() {
        let mut sim = seeded_sim();
        let before = sim.belief();
        trigger_disaster(&mut sim, DisasterKind::Quake, WorldCoord { x: 0, y: 0, z: 0 });
        assert!(
            sim.belief() > before,
            "a disaster should raise belief (fear breeds faith)"
        );
    }

    /// FR-CIV-EMERGENCE — a divine disaster spends belief and smites the terrain.
    #[test]
    fn invoke_divine_disaster_spends_belief_and_smites() {
        let mut sim = seeded_sim();
        sim.add_belief(1_000);
        let target = WorldCoord { x: 0, y: 0, z: 0 };
        assert!(sim.invoke_divine_disaster(DisasterKind::Quake, target, 500));
        assert!(matches!(sim.voxel().read(target), GRAVEL | STONE));
    }

    /// FR-CIV-EMERGENCE — without enough faith, no smite and no belief is spent.
    #[test]
    fn invoke_divine_disaster_requires_faith() {
        let mut sim = seeded_sim();
        let before = sim.belief();
        let target = WorldCoord { x: 0, y: 0, z: 0 };
        assert!(!sim.invoke_divine_disaster(DisasterKind::Quake, target, 1_000_000));
        assert_eq!(sim.belief(), before, "no faith, no smite, no spend");
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
            DisasterEffect::new(0.28, 0.35, 0.25, 0.55, true),
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

    /// radius_for returns a positive radius per kind, scaled by severity (coverage).
    #[test]
    fn radius_for_is_positive_per_kind() {
        assert!(radius_for(DisasterKind::Meteor) > 0);
        assert!(radius_for(DisasterKind::Plague) > 0);
        assert!(radius_for(DisasterKind::Storm) > radius_for(DisasterKind::Plague));
    }

    /// FR-CIV-0100 §3 — at research tier 0 the wildfire ignition threshold is
    /// the raw physical value (no mitigation).
    #[test]
    fn wildfire_ignition_unmitigated_at_tier_zero() {
        assert_eq!(wildfire_ignition_temp_fp(40_000, 0), 40_000);
    }

    /// Research raises the ignition threshold, and the effect is monotonic
    /// non-decreasing in tier (technology makes wildfires rarer).
    #[test]
    fn wildfire_ignition_rises_with_research() {
        let base = wildfire_ignition_temp_fp(40_000, 0);
        let low = wildfire_ignition_temp_fp(40_000, 3);
        let high = wildfire_ignition_temp_fp(40_000, 100);
        assert!(low > base, "research must raise the ignition threshold");
        assert!(high >= low, "more research never lowers the threshold");
    }

    /// Mitigation is capped, so extreme heat can still ignite an advanced
    /// civilisation — disasters are damped, never abolished.
    #[test]
    fn wildfire_ignition_mitigation_is_capped() {
        let saturated = wildfire_ignition_temp_fp(40_000, u64::MAX);
        assert_eq!(
            saturated,
            40_000 + WILDFIRE_RESEARCH_MITIGATION_CAP_FP as i32
        );
    }

    /// Test that phase_disasters triggers wildfire when environmental conditions exceed thresholds
    /// (high temperature + low moisture + storm conditions)
    #[test]
    fn phase_disasters_triggers_wildfire_on_high_heat_low_moisture() {
        let mut sim = Simulation::with_seed(42);

        // Set up extreme environmental conditions that should trigger wildfire
        // High temperature, low moisture, stormy weather
        sim.climate = Climate {
            tick: 1000,
            day_phase: 0.5,  // midday heat
            year_phase: 0.3, // summer
            moon_phase: 0.0,
            tide_offset: 0.0,
        };

        // Create weather with extreme heat and storm conditions
        sim.weather_grid = vec![WeatherCell {
            region_id: 0,
            latitude_fp: 0, // equator
            season: civ_planet::SeasonKind::Summer,
            kind: WeatherKind::Storm,
            temp_c_fp: 45_000,        // 45°C - extreme heat
            precip_mm_fp: 50,         // low precipitation
            storm_intensity_fp: 3000, // high storm intensity
        }];

        // Advance tick so disaster can be triggered
        sim.state.tick = 1000;

        // Call phase_disasters - should trigger wildfire under these conditions
        sim.phase_disasters();

        // Verify that a disaster was triggered by checking if terrain was modified
        // Wildfires should create LAVA/STEAM patterns
        let origin = WorldCoord { x: 0, y: 0, z: 0 };
        let has_wildfire_effects =
            sim.voxel().read(origin) == LAVA || sim.voxel().read(origin) == STEAM;

        // This should eventually be true once implementation is complete
        assert!(
            has_wildfire_effects,
            "Wildfire should be triggered under extreme heat/storm conditions"
        );
    }

    /// Test that phase_disasters remains quiescent when conditions are normal
    #[test]
    fn phase_disasters_quiescent_under_normal_conditions() {
        let mut sim = Simulation::with_seed(42);

        // Set up normal environmental conditions
        sim.climate = Climate {
            tick: 1000,
            day_phase: 0.5,
            year_phase: 0.6, // autumn
            moon_phase: 0.0,
            tide_offset: 0.0,
        };

        // Normal weather conditions
        sim.weather_grid = vec![WeatherCell {
            region_id: 0,
            latitude_fp: 30_000, // temperate zone
            season: civ_planet::SeasonKind::Autumn,
            kind: WeatherKind::Clear,
            temp_c_fp: 18_000,       // 18°C - mild temperature
            precip_mm_fp: 500,       // moderate precipitation
            storm_intensity_fp: 200, // low storm intensity
        }];

        sim.state.tick = 1000;

        // Store original voxel state
        let origin = WorldCoord { x: 0, y: 0, z: 0 };
        let original_material = sim.voxel().read(origin);

        // Call phase_disasters - should NOT trigger disasters under normal conditions
        sim.phase_disasters();

        // Verify terrain remains unchanged (no disaster triggered)
        assert_eq!(
            sim.voxel().read(origin),
            original_material,
            "No disaster should be triggered under normal conditions"
        );
    }

    /// Test that phase_disasters triggers quake under tectonic stress conditions
    #[test]
    fn phase_disasters_triggers_quake_on_tectonic_stress() {
        let mut sim = Simulation::with_seed(123);

        // Simulate tectonic stress through extreme climate patterns that could indicate geological instability
        sim.climate = Climate {
            tick: 2000,
            day_phase: 0.0,   // transition period
            year_phase: 0.25, // spring equinox - potential stress period
            moon_phase: 0.5,  // full moon - maximum tidal forces
            tide_offset: 1.0, // extreme tide
        };

        // Create weather patterns that could correlate with geological stress
        sim.weather_grid = vec![WeatherCell {
            region_id: 0,
            latitude_fp: 45_000, // tectonically active zone
            season: civ_planet::SeasonKind::Spring,
            kind: WeatherKind::Clear,
            temp_c_fp: 12_000, // normal temp but with pressure changes
            precip_mm_fp: 300,
            storm_intensity_fp: 1500, // moderate but with pressure fronts
        }];

        sim.state.tick = 2000;

        // Call phase_disasters - should potentially trigger quake under these conditions
        sim.phase_disasters();

        // Quakes create GRAVEL/STONE patterns
        let origin = WorldCoord { x: 0, y: 0, z: 0 };
        let has_quake_effects =
            sim.voxel().read(origin) == GRAVEL || sim.voxel().read(origin) == STONE;

        // This should eventually be true once implementation is complete
        assert!(
            has_quake_effects,
            "Quake should be triggered under tectonic stress conditions"
        );
    }
}
