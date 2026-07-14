//! Civilization era evaluation (FR-CIV-GAME-003).
//!
//! Eras are derived from simulation state on demand — no persistent field needed.
//! Call [CivEra::evaluate] each tick; compare to previous to detect advances.

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use crate::engine::Simulation;
use crate::tech::{gather_faction_inputs, tick_research, tick_tech, FactionTechState};

/// The six civilization ages, ordered by advancement.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum CivAge {
    Stone,
    Bronze,
    Iron,
    Classical,
    Medieval,
    Industrial,
}

impl CivAge {
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            CivAge::Stone => "Stone",
            CivAge::Bronze => "Bronze",
            CivAge::Iron => "Iron",
            CivAge::Classical => "Classical",
            CivAge::Medieval => "Medieval",
            CivAge::Industrial => "Industrial",
        }
    }

    #[must_use]
    pub fn evaluate(population: u32, techs: u32, surplus: i64) -> Self {
        if techs >= 12 || surplus >= 250_000 {
            CivAge::Industrial
        } else if techs >= 8 || population >= 5_000 {
            CivAge::Medieval
        } else if techs >= 5 || population >= 2_000 {
            CivAge::Classical
        } else if techs >= 2 || population >= 500 {
            CivAge::Bronze
        } else {
            CivAge::Stone
        }
    }
}

/// The six civilization eras, ordered by advancement.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum CivEra {
    Prehistoric,
    Ancient,
    Classical,
    Medieval,
    Renaissance,
    Modern,
}

impl CivEra {
    /// Evaluate the current era from live simulation state.
    /// Conditions are first-match from most-advanced downward.
    pub fn evaluate(sim: &Simulation) -> Self {
        let pop = sim.state.population;
        let techs = sim.researched_tech_count();

        if techs >= 12 {
            CivEra::Modern
        } else if pop >= 10_000 || techs >= 10 {
            CivEra::Renaissance
        } else if pop >= 5_000 || techs >= 8 {
            CivEra::Medieval
        } else if pop >= 2_000 || techs >= 5 {
            CivEra::Classical
        } else if pop >= 500 || techs >= 2 {
            CivEra::Ancient
        } else {
            CivEra::Prehistoric
        }
    }

    /// Wire-safe name for JSON-RPC / HUD display.
    pub fn as_str(self) -> &'static str {
        match self {
            CivEra::Prehistoric => "Prehistoric",
            CivEra::Ancient => "Ancient",
            CivEra::Classical => "Classical",
            CivEra::Medieval => "Medieval",
            CivEra::Renaissance => "Renaissance",
            CivEra::Modern => "Modern",
        }
    }

    /// One-line description of what unlocks the next era.
    pub fn next_conditions(self) -> &'static str {
        match self {
            CivEra::Prehistoric => "pop >= 500 or 2 techs researched",
            CivEra::Ancient => "pop >= 2,000 or 5 techs researched",
            CivEra::Classical => "pop >= 5,000 or 8 techs researched",
            CivEra::Medieval => "pop >= 10,000 or 10 techs researched",
            CivEra::Renaissance => "all 12 techs researched",
            CivEra::Modern => "(peak era reached)",
        }
    }

    /// Normalized position in the ordered era sequence for HUD progress bars.
    #[must_use]
    pub const fn era_progress_fraction(self) -> f32 {
        match self {
            CivEra::Prehistoric => 0.0,
            CivEra::Ancient => 0.2,
            CivEra::Classical => 0.4,
            CivEra::Medieval => 0.6,
            CivEra::Renaissance => 0.8,
            CivEra::Modern => 1.0,
        }
    }
}

/// Per-faction era surfaced on [`crate::engine::SimulationSnapshot`].
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FactionEraSnapshot {
    pub faction_id: u32,
    pub age: CivAge,
    pub age_label: String,
    pub tech_level: u32,
    pub population: u32,
    pub resource_surplus: i64,
}

/// Gameplay gates derived from emergent tech.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct TechGate {
    pub min_tech_level: u32,
    pub production_multiplier_permille: u32,
    pub military_tier: u32,
}

impl TechGate {
    /// Baseline gates increase gradually with technology.
    #[must_use]
    pub fn for_tech_level(tech_level: u32) -> Self {
        let production_multiplier_permille = 1_000 + tech_level.saturating_mul(125).min(1_000);
        let military_tier = match tech_level {
            0 => 0,
            1..=2 => 1,
            3..=4 => 2,
            5..=7 => 3,
            8..=11 => 4,
            _ => 5,
        };
        Self {
            min_tech_level: tech_level,
            production_multiplier_permille,
            military_tier,
        }
    }
}

/// Append-only record of era advances for HUD/replay diagnostics.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct EraHistory {
    pub advances: Vec<(u64, u32, CivAge, CivAge)>,
}

impl EraHistory {
    pub fn record_advance(&mut self, tick: u64, faction_id: u32, previous: CivAge, next: CivAge) {
        self.advances.push((tick, faction_id, previous, next));
    }
}

/// Mutable emergent era/tech state carried on [`Simulation`].
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct EraProgressionState {
    pub faction_ages: BTreeMap<u32, CivAge>,
    pub faction_tech: BTreeMap<u32, FactionTechState>,
    pub history: EraHistory,
}

impl EraProgressionState {
    /// Evaluate faction ages from current tech + economy signals.
    pub fn evaluate_eras(&mut self, sim: &Simulation) {
        let inputs = gather_faction_inputs(sim);
        let tick = sim.state.tick;
        for (faction_id, faction_inputs) in inputs {
            let tech_level = self
                .faction_tech
                .get(&faction_id)
                .map(|t| t.tech_level)
                .unwrap_or(0);
            let next = CivAge::evaluate(
                faction_inputs.population,
                tech_level,
                faction_inputs.surplus,
            );
            let previous = self
                .faction_ages
                .get(&faction_id)
                .copied()
                .unwrap_or(CivAge::Stone);
            if next > previous {
                self.history
                    .record_advance(tick, faction_id, previous, next);
            }
            self.faction_ages.insert(faction_id, next);
        }
    }

    /// Run research, tech, and era evaluation for the current tick.
    pub fn tick(&mut self, sim: &Simulation) {
        tick_research(sim, &mut self.faction_tech);
        tick_tech(&mut self.faction_tech);
        self.evaluate_eras(sim);
    }

    /// Build per-faction snapshot rows for the engine snapshot wire.
    #[must_use]
    pub fn faction_era_snapshots(&self, sim: &Simulation) -> BTreeMap<u32, FactionEraSnapshot> {
        let inputs = gather_faction_inputs(sim);
        let mut rows = BTreeMap::new();
        for (faction_id, faction_inputs) in inputs {
            let age = self
                .faction_ages
                .get(&faction_id)
                .copied()
                .unwrap_or(CivAge::Stone);
            let tech_level = self
                .faction_tech
                .get(&faction_id)
                .map(|t| t.tech_level)
                .unwrap_or(0);
            rows.insert(
                faction_id,
                FactionEraSnapshot {
                    faction_id,
                    age,
                    age_label: age.as_str().to_string(),
                    tech_level,
                    population: faction_inputs.population,
                    resource_surplus: faction_inputs.surplus,
                },
            );
        }
        rows
    }

    /// Deterministic gameplay gates for a faction's current tech state.
    #[must_use]
    pub fn tech_gate_for_faction(&self, faction_id: u32) -> TechGate {
        let tech_level = self
            .faction_tech
            .get(&faction_id)
            .map(|t| t.tech_level)
            .unwrap_or(0);
        TechGate::for_tech_level(tech_level)
    }
}

/// Research phase hook (FR-ERA): emergent progress from economy + population.
pub fn phase_research(sim: &mut Simulation) {
    let mut faction_tech = std::mem::take(&mut sim.era_progression.faction_tech);
    tick_research(sim, &mut faction_tech);
    sim.era_progression.faction_tech = faction_tech;
}

/// Tech + era phase hook (FR-ERA): unlock levels and evaluate ages.
pub fn phase_tech(sim: &mut Simulation) {
    let inputs = gather_faction_inputs(sim);
    let tick = sim.state.tick;
    tick_tech(&mut sim.era_progression.faction_tech);
    for (faction_id, faction_inputs) in inputs {
        let tech_level = sim
            .era_progression
            .faction_tech
            .get(&faction_id)
            .map(|t| t.tech_level)
            .unwrap_or(0);
        let next = CivAge::evaluate(
            faction_inputs.population,
            tech_level,
            faction_inputs.surplus,
        );
        let previous = sim
            .era_progression
            .faction_ages
            .get(&faction_id)
            .copied()
            .unwrap_or(CivAge::Stone);
        if next > previous {
            sim.era_progression
                .history
                .record_advance(tick, faction_id, previous, next);
        }
        sim.era_progression.faction_ages.insert(faction_id, next);
    }
}

#[cfg(test)]
mod tests {
    use super::{CivAge, CivEra};
    use civ_agents::{spawn_civilian_at, ActorVisualKind, Alignment};

    use crate::engine::{Resources, Simulation};

    fn thriving_stagnant_sim() -> Simulation {
        let mut sim = Simulation::with_seed(7);
        sim.state.resources = Resources {
            food: crate::Fixed::from_num(2_000),
            wood: crate::Fixed::from_num(1_000),
            metal: crate::Fixed::from_num(1_000),
            energy: crate::Fixed::from_num(1_000),
        };

        let mut rng = sim.rng_mut().clone();
        for id in 0..8 {
            spawn_civilian_at(
                &mut sim.world,
                1_000 + id,
                Alignment::Faction(0),
                0.2,
                0.2,
                ActorVisualKind::Humanoid,
                &mut rng,
            );
        }
        for id in 0..4 {
            spawn_civilian_at(
                &mut sim.world,
                2_000 + id,
                Alignment::Faction(1),
                0.8,
                0.8,
                ActorVisualKind::Humanoid,
                &mut rng,
            );
        }
        *sim.rng_mut() = rng;
        sim
    }

    #[test]
    fn era_progress_fraction_is_earliest_latest_and_monotonic() {
        let eras = [
            CivEra::Prehistoric,
            CivEra::Ancient,
            CivEra::Classical,
            CivEra::Medieval,
            CivEra::Renaissance,
            CivEra::Modern,
        ];

        assert_eq!(eras[0].era_progress_fraction(), 0.0);
        assert_eq!(eras[eras.len() - 1].era_progress_fraction(), 1.0);

        for window in eras.windows(2) {
            assert!(window[0].era_progress_fraction() < window[1].era_progress_fraction());
        }
    }

    /// FR-CIV-TECH: sim ticks accumulate research and unlock tech levels.
    #[test]
    fn fr_civ_tech_ticks_unlock_faction_tech() {
        let mut sim = thriving_stagnant_sim();
        let start_level = sim
            .era_progression()
            .faction_tech
            .get(&0)
            .cloned()
            .unwrap_or_default()
            .tech_level;

        sim.advance_ticks(1);
        let after_one = sim
            .era_progression()
            .faction_tech
            .get(&0)
            .cloned()
            .unwrap_or_default();
        assert!(
            after_one.research_points > 0,
            "FR-CIV-TECH: research should accumulate during the sim tick"
        );

        sim.advance_ticks(8);
        let after_n = sim
            .era_progression()
            .faction_tech
            .get(&0)
            .cloned()
            .unwrap_or_default();

        assert!(
            after_n.tech_level > start_level,
            "FR-CIV-TECH: N ticks should unlock a tech level (start={}, end={})",
            start_level,
            after_n.tech_level
        );
        assert_eq!(
            sim.research_tier(),
            u64::from(after_n.tech_level),
            "FR-CIV-TECH: public research tier should reflect unlocked faction tech"
        );
    }

    /// FR-TECH-gating: can_unlock predicate gates tech level advancement.
    #[test]
    fn can_unlock_gates_tech_advancement() {
        use crate::tech::can_unlock;
        assert!(can_unlock(0, 0), "level 0 can unlock tech 0");
        assert!(can_unlock(3, 2), "level 3 can unlock tech 2");
        assert!(can_unlock(5, 5), "level 5 can unlock tech 5");
        assert!(!can_unlock(2, 3), "level 2 cannot unlock tech 3");
        assert!(!can_unlock(0, 1), "level 0 cannot unlock tech 1");
    }

    /// FR-TECH-gating: a prosperous faction out-researches, then diffusion lifts a neighbor later.
    #[test]
    fn prosperous_faction_out_researches_and_diffuses_to_neighbor() {
        let mut sim = thriving_stagnant_sim();
        let mut rng = sim.rng_mut().clone();

        for id in 0..12 {
            let _ = spawn_civilian_at(
                &mut sim.world,
                10_000 + id,
                Alignment::Faction(0),
                0.20,
                0.20,
                ActorVisualKind::Humanoid,
                &mut rng,
            );
        }
        for id in 0..2 {
            let _ = spawn_civilian_at(
                &mut sim.world,
                20_000 + id,
                Alignment::Faction(1),
                0.80,
                0.80,
                ActorVisualKind::Humanoid,
                &mut rng,
            );
        }
        *sim.rng_mut() = rng;

        let start_0 = sim
            .era_progression()
            .faction_tech
            .get(&0)
            .cloned()
            .unwrap_or_default();
        let start_1 = sim
            .era_progression()
            .faction_tech
            .get(&1)
            .cloned()
            .unwrap_or_default();

        let mut advanced_0_tick = None;
        for _ in 0..40 {
            sim.advance_ticks(1);
            let tick = sim.state.tick;
            let tech_0 = sim
                .era_progression()
                .faction_tech
                .get(&0)
                .cloned()
                .unwrap_or_default();
            let tech_1 = sim
                .era_progression()
                .faction_tech
                .get(&1)
                .cloned()
                .unwrap_or_default();
            if advanced_0_tick.is_none() && tech_0.tech_level > start_0.tech_level {
                advanced_0_tick = Some(tick);
            }
            if let Some(leader_tick) = advanced_0_tick {
                if tech_1.tech_level > start_1.tech_level {
                    assert!(
                        tick > leader_tick,
                        "neighbor should gain tech via diffusion after the leader advances"
                    );
                    assert!(
                        tech_1.tech_level >= 1,
                        "neighbor should eventually gain at least one tech level"
                    );
                    return;
                }
            }
        }

        panic!("neighbor never gained tech via diffusion");
    }
}
