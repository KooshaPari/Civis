//! Victory and defeat condition checks (FR-CIV-GAME-001).

use crate::{DiplomacyKind, Simulation};

pub const PEACE_TICKS_THRESHOLD: u64 = 500;
pub const POPULATION_VICTORY: u64 = 10_000;
pub const TECH_VICTORY_COUNT: usize = 12;
const TYRANNY_POPULATION_SHARE: f64 = 0.95;
const TYRANNY_TICKS_THRESHOLD: u64 = 200;

/// Live progress toward the three victory conditions checked by [`check_outcome`].
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct OutcomeProgress {
    pub population: u64,
    pub population_target: u64,
    pub researched_techs: usize,
    pub researched_techs_target: usize,
    pub peace_ticks: u64,
    pub peace_ticks_target: u64,
}

/// Compute truthful progress from the same engine state used by [`check_outcome`].
#[must_use]
pub fn outcome_progress(sim: &Simulation) -> OutcomeProgress {
    let tick = sim.state.tick;
    let last_conflict_tick = sim
        .diplomacy_events()
        .iter()
        .filter(|event| event.kind == DiplomacyKind::Conflict)
        .map(|event| event.tick)
        .max();
    let peace_ticks = last_conflict_tick.map_or(tick, |last| tick.saturating_sub(last));

    OutcomeProgress {
        population: sim.state.population,
        population_target: POPULATION_VICTORY,
        researched_techs: sim.research_cache().researched.len(),
        researched_techs_target: TECH_VICTORY_COUNT,
        peace_ticks: peace_ticks.min(PEACE_TICKS_THRESHOLD),
        peace_ticks_target: PEACE_TICKS_THRESHOLD,
    }
}

/// Outcome of a game-state check.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum GameOutcome {
    /// Victory event carrying the victory-reason string (e.g. "Thriving
    /// Civilization (Player)"). Tuple-style so callers can build it with
    /// `GameOutcome::Victory(reason.into())` without naming fields.
    Victory(String),
    /// Defeat event carrying the defeat reason.
    Defeat(String),
    /// Game still in progress.
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
            Self::Victory(kind) => kind.as_str(),
            Self::Defeat(reason) => reason.as_str(),
            Self::Ongoing => "",
        }
    }

    pub fn faction(&self) -> Option<u32> {
        match self {
            Self::Victory(_) => None,
            Self::Defeat(_) | Self::Ongoing => None,
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
        .map(|v| v.to_f64().max(0.0))
        .sum();
    if total_treasury > 0.0 {
        for (_, wealth) in &state.faction_treasury {
            let share = wealth.to_f64().max(0.0) / total_treasury;
            if share >= TYRANNY_POPULATION_SHARE && tick >= TYRANNY_TICKS_THRESHOLD {
                return GameOutcome::Defeat("Tyranny".to_owned());
            }
        }
    }

    // ── Victory: all factions at peace for 500 ticks ────────────────────────
    // Count conflict events in the last PEACE_TICKS_THRESHOLD ticks.
    let recent_conflict = sim.diplomacy_events().iter().any(|e| {
        e.kind == DiplomacyKind::Conflict && tick.saturating_sub(e.tick) < PEACE_TICKS_THRESHOLD
    });
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
        let mut sim = Simulation::with_seed(42);
        sim.state.population = 1;
        assert_eq!(check_outcome(&sim), GameOutcome::Ongoing);
    }

    #[test]
    fn victory_population_threshold() {
        let mut sim = Simulation::with_seed(42);
        sim.state.population = POPULATION_VICTORY - 1;
        assert_eq!(check_outcome(&sim), GameOutcome::Ongoing);
        sim.state.population = POPULATION_VICTORY;
        assert!(matches!(
            check_outcome(&sim),
            GameOutcome::Victory(kind) if kind == "Thriving Civilization"
        ));
    }

    #[test]
    fn defeat_extinction() {
        let mut sim = Simulation::with_seed(42);
        sim.state.population = 0;
        assert!(matches!(
            check_outcome(&sim),
            GameOutcome::Defeat(reason) if reason == "Civilization Collapsed"
        ));
    }

    #[test]
    fn outcome_progress_reads_live_population_research_and_peace_state() {
        let mut sim = Simulation::with_seed(42);
        sim.state.population = 4_321;
        sim.state.tick = 123;
        sim.research_cache_mut().researched = vec![
            "pottery".to_owned(),
            "masonry".to_owned(),
            "writing".to_owned(),
        ];

        let progress = outcome_progress(&sim);
        assert_eq!(progress.population, 4_321);
        assert_eq!(progress.population_target, 10_000);
        assert_eq!(progress.researched_techs, 3);
        assert_eq!(progress.researched_techs_target, 12);
        assert_eq!(progress.peace_ticks, 123);
        assert_eq!(progress.peace_ticks_target, 500);
    }
}
