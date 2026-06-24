//! Civilization era evaluation (FR-CIV-GAME-003).
//!
//! Eras are derived from simulation state on demand — no persistent field needed.
//! Call [CivEra::evaluate] each tick; compare to previous to detect advances.

use serde::{Deserialize, Serialize};
use crate::engine::Simulation;

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
        let techs = sim.research_cache().researched.len();

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
            CivEra::Ancient     => "pop >= 2,000 or 5 techs researched",
            CivEra::Classical   => "pop >= 5,000 or 8 techs researched",
            CivEra::Medieval    => "pop >= 10,000 or 10 techs researched",
            CivEra::Renaissance => "all 12 techs researched",
            CivEra::Modern      => "(peak era reached)",
        }
    }
}