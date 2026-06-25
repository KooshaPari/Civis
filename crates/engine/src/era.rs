//! Civilization era evaluation (FR-ERA / FR-CIV-GAME-003).
//!
//! Eras **emerge** from accumulated tech, population density, and resource
//! surplus — not a scripted tech tree. Call [`EraProgressionState::tick`] each
//! simulation tick; compare faction ages over time to detect advances.

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use crate::engine::Simulation;
use crate::history::EraHistory;
use crate::tech::{gather_faction_inputs, tick_research, tick_tech, FactionTechState};

/// Named civilization ages (stone → bronze → iron → …).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum CivAge {
    Stone,
    Bronze,
    Iron,
    Classical,
    Medieval,
    Industrial,
}

impl CivAge {
    /// Wire-safe label for JSON-RPC / HUD display.
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

    /// Evaluate emergent age from live faction metrics (first-match from most advanced).
    #[must_use]
    pub fn evaluate(population: u32, tech_level: u32, surplus: i64) -> Self {
        if population >= 4_000 || tech_level >= 12 || surplus >= 25_000 {
            CivAge::Industrial
        } else if population >= 1_500 || tech_level >= 8 || surplus >= 10_000 {
            CivAge::Medieval
        } else if population >= 400 || tech_level >= 5 || surplus >= 2_500 {
            CivAge::Classical
        } else if population >= 150 || tech_level >= 3 || surplus >= 600 {
            CivAge::Iron
        } else if population >= 40 || tech_level >= 1 || surplus >= 150 {
            CivAge::Bronze
        } else {
            CivAge::Stone
        }
    }

    /// One-line description of emergent conditions for the next age.
    #[must_use]
    pub fn next_conditions(self) -> &'static str {
        match self {
            CivAge::Stone => "pop >= 40, tech >= 1, or surplus >= 150",
            CivAge::Bronze => "pop >= 150, tech >= 3, or surplus >= 600",
            CivAge::Iron => "pop >= 400, tech >= 5, or surplus >= 2500",
            CivAge::Classical => "pop >= 1500, tech >= 8, or surplus >= 10000",
            CivAge::Medieval => "pop >= 4000, tech >= 12, or surplus >= 25000",
            CivAge::Industrial => "(peak age reached)",
        }
    }
}

/// Legacy six-era enum kept for backward-compatible call sites.
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
    /// Map a [`CivAge`] into the legacy era enum.
    #[must_use]
    pub fn from_age(age: CivAge) -> Self {
        match age {
            CivAge::Stone => CivEra::Prehistoric,
            CivAge::Bronze => CivEra::Ancient,
            CivAge::Iron | CivAge::Classical => CivEra::Classical,
            CivAge::Medieval => CivEra::Medieval,
            CivAge::Industrial => CivEra::Modern,
        }
    }

    /// Evaluate the global era from live simulation state (max faction age).
    #[must_use]
    pub fn evaluate(sim: &Simulation) -> Self {
        let age = sim
            .era_progression()
            .faction_ages
            .values()
            .copied()
            .max()
            .unwrap_or(CivAge::Stone);
        Self::from_age(age)
    }

    #[must_use]
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

    #[must_use]
    pub fn next_conditions(self) -> &'static str {
        match self {
            CivEra::Prehistoric => CivAge::Stone.next_conditions(),
            CivEra::Ancient => CivAge::Bronze.next_conditions(),
            CivEra::Classical => CivAge::Iron.next_conditions(),
            CivEra::Medieval => CivAge::Classical.next_conditions(),
            CivEra::Renaissance => CivAge::Medieval.next_conditions(),
            CivEra::Modern => "(peak era reached)",
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
            let age = self.faction_ages.get(&faction_id).copied().unwrap_or(CivAge::Stone);
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
}

/// Research phase hook (FR-ERA): emergent progress from economy + population.
pub fn phase_research(sim: &mut Simulation) {
    tick_research(sim, &mut sim.era_progression_mut().faction_tech);
}

/// Tech + era phase hook (FR-ERA): unlock levels and evaluate ages.
pub fn phase_tech(sim: &mut Simulation) {
    let progression = sim.era_progression_mut();
    tick_tech(&mut progression.faction_tech);
    progression.evaluate_eras(sim);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::engine::Fixed;

    fn thriving_stagnant_sim() -> Simulation {
        let mut sim = Simulation::with_seed(42);
        let thriving = sim.state.faction_resources.entry(0).or_default();
        thriving.food = Fixed::from_num(8_000);
        thriving.wood = Fixed::from_num(6_000);
        thriving.metal = Fixed::from_num(4_000);
        sim.state
            .faction_treasury
            .insert(0, Fixed::from_num(50_000));

        let stagnant = sim.state.faction_resources.entry(1).or_default();
        stagnant.food = Fixed::from_num(5);
        stagnant.wood = Fixed::from_num(5);
        stagnant.metal = Fixed::from_num(5);
        sim.state
            .faction_treasury
            .insert(1, Fixed::from_num(10));
        sim
    }

    /// FR-ERA: a thriving faction advances emergent age over N ticks; stagnant does not.
    #[test]
    fn thriving_faction_advances_era_stagnant_does_not() {
        let mut sim = thriving_stagnant_sim();
        let start_thriving = sim
            .era_progression()
            .faction_ages
            .get(&0)
            .copied()
            .unwrap_or(CivAge::Stone);
        let start_stagnant = sim
            .era_progression()
            .faction_ages
            .get(&1)
            .copied()
            .unwrap_or(CivAge::Stone);

        sim.advance_ticks(320);

        let end_thriving = sim
            .era_progression()
            .faction_ages
            .get(&0)
            .copied()
            .unwrap_or(CivAge::Stone);
        let end_stagnant = sim
            .era_progression()
            .faction_ages
            .get(&1)
            .copied()
            .unwrap_or(CivAge::Stone);

        assert!(
            end_thriving > start_thriving,
            "thriving faction should advance from {start_thriving:?}, got {end_thriving:?}"
        );
        assert_eq!(
            end_stagnant, start_stagnant,
            "stagnant faction should remain at {start_stagnant:?}, got {end_stagnant:?}"
        );

        let snapshot = sim.snapshot();
        assert!(
            snapshot.faction_eras.get(&0).map(|s| s.age) > snapshot.faction_eras.get(&1).map(|s| s.age),
            "snapshot must surface higher era for thriving faction"
        );
    }
}
