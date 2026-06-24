//! Victory and defeat condition checks (FR-CIV-GAME-001).

use crate::{DiplomacyKind, Simulation};

const PEACE_TICKS_THRESHOLD: u64 = 500;
const POPULATION_VICTORY: u64 = 10_000;
const TECH_VICTORY_COUNT: usize = 12;
const TYRANNY_POPULATION_SHARE: f64 = 0.95;
const TYRANNY_TICKS_THRESHOLD: u64 = 200;

/// Outcome of a game-state check.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum GameOutcome {
    Victory(String),
    Defeat(String),
    Ongoing,
}

impl GameOutcome {
    pub fn tag(&self) -> &'static str {
        match self {
            Self::Victory(_) => "victory",
            Self::Defeat(_) => "defeat",
            Self::Ongoing => "ongoing",
        }
    }

    pub fn reason(&self) -> &str {
        match self {
            Self::Victory(r) | Self::Defeat(r) => r.as_str(),
            Self::Ongoing => "",
        }
    }
}

/// Check all victory/defeat conditions against the current simulation state.
///
/// Called by the `sim.outcome` JSON-RPC handler; never mutates the simulation.
pub fn check_outcome(sim: &Simulation) -> GameOutcome {
    let state = &sim.state;
    let tick = state.tick;

    // ── Defeat: extinction ───────────────────────────────────────────────────
    if !state.factions.is_empty() && state.population == 0 {
        return GameOutcome::Defeat("Civilization Collapsed".to_owned());
    }

    // ── Defeat: tyranny (single faction > 95 % pop for 200 ticks) ───────────
    // We track this via the treasury share as a population proxy (cheapest
    // available per-faction scalar without ECS). For a real impl, track
    // faction_population once that field lands. ponytail: treasury-share proxy
    let total_treasury: f64 = state
        .faction_treasury
        .values()
        .map(|v| f64::from(v.to_num::<f32>().max(0.0)))
        .sum();
    if total_treasury > 0.0 {
        for (_, wealth) in &state.faction_treasury {
            let share = f64::from(wealth.to_num::<f32>().max(0.0)) / total_treasury;
            if share >= TYRANNY_POPULATION_SHARE && tick >= TYRANNY_TICKS_THRESHOLD {
                return GameOutcome::Defeat("Tyranny".to_owned());
            }
        }
    }

    // ── Victory: all factions at peace for 500 ticks ────────────────────────
    // Count conflict events in the last PEACE_TICKS_THRESHOLD ticks.
    let recent_conflict = sim
        .diplomacy_events()
        .iter()
        .any(|e| e.kind == DiplomacyKind::Conflict && tick.saturating_sub(e.tick) < PEACE_TICKS_THRESHOLD);
    if !recent_conflict && tick >= PEACE_TICKS_THRESHOLD {
        return GameOutcome::Victory("Age of Harmony".to_owned());
    }

    // ── Victory: population > 10 000 ────────────────────────────────────────
    if state.population >= POPULATION_VICTORY {
        return GameOutcome::Victory("Thriving Civilization".to_owned());
    }

    // ── Victory: all 12 techs researched ────────────────────────────────────
    if sim.research_cache().researched.len() >= TECH_VICTORY_COUNT {
        return GameOutcome::Victory("Age of Enlightenment".to_owned());
    }

    GameOutcome::Ongoing
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Simulation;

    #[test]
    fn ongoing_on_fresh_sim() {
        let sim = Simulation::new(42);
        assert_eq!(check_outcome(&sim), GameOutcome::Ongoing);
    }

    #[test]
    fn victory_population_threshold() {
        let mut sim = Simulation::new(42);
        sim.state.population = POPULATION_VICTORY;
        assert!(matches!(check_outcome(&sim), GameOutcome::Victory(_)));
    }

    #[test]
    fn defeat_extinction() {
        let mut sim = Simulation::new(42);
        sim.state.population = 0;
        assert!(matches!(check_outcome(&sim), GameOutcome::Defeat(_)));
    }
}
