//! Gameplay loop: objectives, victory/defeat conditions, scenario goals, and
//! per-faction scoring (FR-CIV-GAME-002).
//!
//! This module extends the base `check_outcome` / `GameOutcome` system with
//! structured victory types, defeat conditions, per-faction progress tracking,
//! and a composite score suitable for leaderboard / client display.

use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::engine::{Simulation, WorldState};
use crate::conditions::GameOutcome;

// ── Thresholds ───────────────────────────────────────────────────────────────

/// Fraction of total faction treasury required for Domination victory.
pub const DOMINATION_TERRITORY_THRESHOLD: f32 = 0.75;
/// Fraction of global belief required for Cultural victory.
pub const CULTURAL_BELIEF_THRESHOLD: f32 = 0.80;
/// Fraction of total resources required for Economic victory.
pub const ECONOMIC_RESOURCE_THRESHOLD: f32 = 0.60;
/// Research tier required for Scientific victory.
pub const SCIENTIFIC_TECH_TIER: u64 = 5;

// ── Victory / Defeat types ───────────────────────────────────────────────────

/// The kind of victory a faction can achieve.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum VictoryType {
    /// Controls ≥75% of simulated territory (proxied by treasury share).
    Domination,
    /// Faction belief spread ≥80% of global belief total.
    Cultural,
    /// Controls ≥60% of total resource stocks.
    Economic,
    /// Reaches research tier 5 or above.
    Scientific,
}

impl VictoryType {
    /// Human-readable label for client display.
    pub fn label(self) -> &'static str {
        match self {
            Self::Domination => "Domination",
            Self::Cultural => "Cultural",
            Self::Economic => "Economic",
            Self::Scientific => "Scientific",
        }
    }
}

/// The kind of defeat a faction can suffer.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DefeatCondition {
    /// Global population reaches zero.
    Extinction,
    /// The faction loses all treasury / territory (share → 0 while others remain).
    Collapse,
}

impl DefeatCondition {
    /// Human-readable label for client display.
    pub fn label(self) -> &'static str {
        match self {
            Self::Extinction => "Extinction",
            Self::Collapse => "Collapse",
        }
    }
}

// ── Victory condition descriptor ─────────────────────────────────────────────

/// A single named victory condition bound to a faction.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct VictoryCondition {
    /// Which type of victory this condition checks.
    pub victory_type: VictoryType,
    /// The faction ID this condition tracks.
    pub faction_id: u32,
    /// Threshold override. `None` = use the module-level constant for the type.
    pub threshold: Option<f32>,
}

impl VictoryCondition {
    /// Effective threshold for this condition.
    pub fn effective_threshold(&self) -> f32 {
        match self.threshold {
            Some(t) => t,
            None => match self.victory_type {
                VictoryType::Domination => DOMINATION_TERRITORY_THRESHOLD,
                VictoryType::Cultural => CULTURAL_BELIEF_THRESHOLD,
                VictoryType::Economic => ECONOMIC_RESOURCE_THRESHOLD,
                VictoryType::Scientific => SCIENTIFIC_TECH_TIER as f32,
            },
        }
    }

    /// Evaluate this condition against the current world state and simulation.
    ///
    /// Returns `Some(GameOutcome::Victory(_))` when the condition is satisfied,
    /// `None` otherwise.  Never returns `Defeat` or `Ongoing`.
    pub fn evaluate(&self, sim: &Simulation) -> Option<GameOutcome> {
        let state = &sim.state;
        let met = match self.victory_type {
            VictoryType::Domination => check_domination(state, self.faction_id, self.effective_threshold()),
            VictoryType::Cultural => check_cultural(state, self.faction_id, self.effective_threshold()),
            VictoryType::Economic => check_economic(state, self.faction_id, self.effective_threshold()),
            VictoryType::Scientific => check_scientific(sim, self.effective_threshold() as u64),
        };
        if met {
            let faction_name = state
                .factions
                .get(&self.faction_id)
                .cloned()
                .unwrap_or_else(|| format!("Faction {}", self.faction_id));
            Some(GameOutcome::Victory(format!(
                "{} Victory ({})",
                self.victory_type.label(),
                faction_name
            )))
        } else {
            None
        }
    }
}

// ── Per-condition checks ─────────────────────────────────────────────────────

/// Returns `true` when `faction_id` holds ≥`threshold` share of total treasury.
pub fn check_domination(state: &WorldState, faction_id: u32, threshold: f32) -> bool {
    let total: f64 = state
        .faction_treasury
        .values()
        .map(|v| v.to_f64().max(0.0))
        .sum();
    if total <= 0.0 {
        return false;
    }
    let faction = state
        .faction_treasury
        .get(&faction_id)
        .map(|v| v.to_f64().max(0.0))
        .unwrap_or(0.0);
    (faction / total) as f32 >= threshold
}

/// Returns `true` when `faction_id` belief share ≥ `threshold`.
///
/// Belief is a global scalar; per-faction belief is proxied by treasury share
/// until per-faction belief tracking is wired (FR-CIV-REL-003 downstream).
/// When `state.belief == 0` the check always returns `false`.
pub fn check_cultural(state: &WorldState, faction_id: u32, threshold: f32) -> bool {
    if state.belief == 0 {
        return false;
    }
    // Proxy: belief distributed proportionally to treasury share.
    let total_treasury: f64 = state
        .faction_treasury
        .values()
        .map(|v| v.to_f64().max(0.0))
        .sum();
    if total_treasury <= 0.0 {
        return false;
    }
    let faction_treasury = state
        .faction_treasury
        .get(&faction_id)
        .map(|v| v.to_f64().max(0.0))
        .unwrap_or(0.0);
    let belief_share = (faction_treasury / total_treasury) as f32;
    belief_share >= threshold
}

/// Returns `true` when `faction_id` holds ≥`threshold` of total resource units
/// (food + wood + metal + energy summed across all factions).
pub fn check_economic(state: &WorldState, faction_id: u32, threshold: f32) -> bool {
    let mut total = 0.0f64;
    let mut faction_total = 0.0f64;
    for (fid, res) in &state.faction_resources {
        let sum = res.food.to_f64().max(0.0)
            + res.wood.to_f64().max(0.0)
            + res.metal.to_f64().max(0.0)
            + res.energy.to_f64().max(0.0);
        total += sum;
        if *fid == faction_id {
            faction_total = sum;
        }
    }
    if total <= 0.0 {
        return false;
    }
    (faction_total / total) as f32 >= threshold
}

/// Returns `true` when the simulation's research tier ≥ `required_tier`.
pub fn check_scientific(sim: &Simulation, required_tier: u64) -> bool {
    sim.research_tier() >= required_tier
}

/// Returns `true` when `faction_id` has fully collapsed (treasury ≤ 0 while
/// at least one other faction has positive treasury).
pub fn check_collapse(state: &WorldState, faction_id: u32) -> bool {
    let faction_val = state
        .faction_treasury
        .get(&faction_id)
        .map(|v| v.to_f64())
        .unwrap_or(0.0);
    if faction_val > 0.0 {
        return false;
    }
    // At least one other faction must still exist with positive treasury.
    state
        .faction_treasury
        .iter()
        .any(|(fid, v)| *fid != faction_id && v.to_f64() > 0.0)
}

// ── Scenario objectives ──────────────────────────────────────────────────────

/// A single objective attached to a scenario.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ScenarioObjective {
    /// The victory condition this objective checks.
    pub condition: VictoryCondition,
    /// Optional tick deadline. When `Some(n)` and the condition is not met by
    /// tick `n`, the objective is treated as failed (returns a `Defeat`).
    pub tick_limit: Option<u64>,
}

impl ScenarioObjective {
    /// Evaluate the objective. Returns:
    /// - `Some(Victory(_))` when the condition is met in time.
    /// - `Some(Defeat(_))` when `tick_limit` has expired without success.
    /// - `None` while still in progress.
    pub fn evaluate(&self, sim: &Simulation) -> Option<GameOutcome> {
        if let Some(outcome) = self.condition.evaluate(sim) {
            return Some(outcome);
        }
        if let Some(limit) = self.tick_limit {
            if sim.state.tick >= limit {
                let faction_name = sim
                    .state
                    .factions
                    .get(&self.condition.faction_id)
                    .cloned()
                    .unwrap_or_else(|| format!("Faction {}", self.condition.faction_id));
                return Some(GameOutcome::Defeat(format!(
                    "{} objective expired for {} at tick {}",
                    self.condition.victory_type.label(),
                    faction_name,
                    limit,
                )));
            }
        }
        None
    }
}

/// Canonical example scenario: survive 1 000 ticks without extinction.
pub fn scenario_survive_1000(faction_id: u32) -> ScenarioObjective {
    ScenarioObjective {
        // Survival is proxied as "domination threshold = 0 (any positive share is OK)".
        // We reuse Domination at 0% threshold — any non-zero share means alive.
        condition: VictoryCondition {
            victory_type: VictoryType::Domination,
            faction_id,
            threshold: Some(0.0),
        },
        tick_limit: Some(1_000),
    }
}

/// Canonical example scenario: achieve Cultural victory within 2 000 ticks.
pub fn scenario_cultural_dominance(faction_id: u32) -> ScenarioObjective {
    ScenarioObjective {
        condition: VictoryCondition {
            victory_type: VictoryType::Cultural,
            faction_id,
            threshold: None, // uses CULTURAL_BELIEF_THRESHOLD (0.80)
        },
        tick_limit: Some(2_000),
    }
}

// ── GameplayState resource ───────────────────────────────────────────────────

/// Per-faction progress snapshot for each victory dimension.
///
/// Updated by [`compute_gameplay_state`]; consumed by the client to render
/// progress bars and by the server's `/sim/outcome` handler.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct FactionProgress {
    /// Fraction of total treasury held by this faction (0..1).
    pub territory_share: f32,
    /// Faction belief share proxy (0..1).
    pub belief_share: f32,
    /// Faction resource share (0..1).
    pub resource_share: f32,
    /// Whether the faction has reached Scientific victory tier.
    pub scientific_reached: bool,
}

/// Simulation-wide gameplay state resource.
///
/// Tracks per-faction progress toward each victory type.
/// Call [`compute_gameplay_state`] each tick to refresh.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct GameplayState {
    /// Per-faction progress snapshots, keyed by faction ID.
    pub faction_progress: HashMap<u32, FactionProgress>,
    /// Current tick (mirrors `WorldState::tick`).
    pub tick: u64,
    /// Resolved outcome when any condition fires; `None` while `Ongoing`.
    pub resolved_outcome: Option<GameOutcome>,
}

/// Recompute `GameplayState` from the current simulation.
///
/// Checks all four victory types and both defeat conditions.  Returns a fresh
/// `GameplayState` with `resolved_outcome` set if any condition fired.
pub fn compute_gameplay_state(sim: &Simulation) -> GameplayState {
    let state = &sim.state;
    let tick = state.tick;

    // ── Defeat: extinction ────────────────────────────────────────────────────
    if !state.factions.is_empty() && state.population == 0 {
        return GameplayState {
            faction_progress: Default::default(),
            tick,
            resolved_outcome: Some(GameOutcome::Defeat(
                DefeatCondition::Extinction.label().to_owned(),
            )),
        };
    }

    // ── Shared denominators ────────────────────────────────────────────────────
    let total_treasury: f64 = state
        .faction_treasury
        .values()
        .map(|v| v.to_f64().max(0.0))
        .sum();

    let total_resources: f64 = state
        .faction_resources
        .values()
        .map(|r| {
            r.food.to_f64().max(0.0)
                + r.wood.to_f64().max(0.0)
                + r.metal.to_f64().max(0.0)
                + r.energy.to_f64().max(0.0)
        })
        .sum();

    let sci_reached = check_scientific(sim, SCIENTIFIC_TECH_TIER);

    // ── Per-faction progress & victory/collapse checks ────────────────────────
    let mut faction_progress: HashMap<u32, FactionProgress> = HashMap::new();
    let mut resolved_outcome: Option<GameOutcome> = None;

    for (&fid, fname) in &state.factions {
        let treasury = state
            .faction_treasury
            .get(&fid)
            .map(|v| v.to_f64().max(0.0))
            .unwrap_or(0.0);

        let resources = state
            .faction_resources
            .get(&fid)
            .map(|r| {
                r.food.to_f64().max(0.0)
                    + r.wood.to_f64().max(0.0)
                    + r.metal.to_f64().max(0.0)
                    + r.energy.to_f64().max(0.0)
            })
            .unwrap_or(0.0);

        let territory_share = if total_treasury > 0.0 {
            (treasury / total_treasury) as f32
        } else {
            0.0
        };
        let belief_share = territory_share; // proxy until FR-CIV-REL-003 lands
        let resource_share = if total_resources > 0.0 {
            (resources / total_resources) as f32
        } else {
            0.0
        };

        faction_progress.insert(
            fid,
            FactionProgress {
                territory_share,
                belief_share,
                resource_share,
                scientific_reached: sci_reached,
            },
        );

        // Victory checks (first match wins)
        if resolved_outcome.is_none() {
            if territory_share >= DOMINATION_TERRITORY_THRESHOLD {
                resolved_outcome = Some(GameOutcome::Victory(format!(
                    "Domination Victory ({})", fname
                )));
            } else if belief_share >= CULTURAL_BELIEF_THRESHOLD {
                resolved_outcome = Some(GameOutcome::Victory(format!(
                    "Cultural Victory ({})", fname
                )));
            } else if resource_share >= ECONOMIC_RESOURCE_THRESHOLD {
                resolved_outcome = Some(GameOutcome::Victory(format!(
                    "Economic Victory ({})", fname
                )));
            } else if sci_reached {
                resolved_outcome = Some(GameOutcome::Victory(format!(
                    "Scientific Victory ({})", fname
                )));
            }
        }

        // Collapse defeat
        if resolved_outcome.is_none() && check_collapse(state, fid) {
            resolved_outcome = Some(GameOutcome::Defeat(format!(
                "{} ({})",
                DefeatCondition::Collapse.label(),
                fname
            )));
        }
    }

    GameplayState {
        faction_progress,
        tick,
        resolved_outcome,
    }
}

// ── Faction scoring ──────────────────────────────────────────────────────────

/// Composite score for a single faction.
///
/// Each sub-score is a non-negative integer; the total is their sum.
/// Inputs are all derived from `WorldState` so the function is pure and
/// deterministic.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FactionScore {
    /// Faction identifier.
    pub faction_id: u32,
    /// Population-derived score (global population ÷ faction count, capped).
    pub pop_score: u64,
    /// Research tier × 1 000.
    pub tech_score: u64,
    /// Belief share × 10 000 (scaled to integer).
    pub belief_score: u64,
    /// Treasury share × 10 000 (territory proxy).
    pub territory_score: u64,
    /// Trade-route volume involving this faction × 100.
    pub legends_score: u64,
    /// Total composite score.
    pub total: u64,
}

impl FactionScore {
    /// Sum all sub-scores into `total`.
    fn compute_total(
        pop_score: u64,
        tech_score: u64,
        belief_score: u64,
        territory_score: u64,
        legends_score: u64,
    ) -> u64 {
        pop_score
            .saturating_add(tech_score)
            .saturating_add(belief_score)
            .saturating_add(territory_score)
            .saturating_add(legends_score)
    }
}

/// Compute per-faction scores from the current simulation state.
///
/// The function is pure: same inputs always produce the same output.
/// Scores are returned sorted descending by `total` so the leaderboard is
/// ready to display.
pub fn compute_scores(sim: &Simulation) -> Vec<FactionScore> {
    let state = &sim.state;
    let faction_count = state.factions.len().max(1) as u64;

    let pop_score_base = state.population.saturating_div(faction_count);

    let total_treasury: f64 = state
        .faction_treasury
        .values()
        .map(|v| v.to_f64().max(0.0))
        .sum();

    let tech_score = sim.research_tier().saturating_mul(1_000);
    let belief_global = state.belief;

    let mut scores: Vec<FactionScore> = state
        .factions
        .keys()
        .map(|&fid| {
            let treasury = state
                .faction_treasury
                .get(&fid)
                .map(|v| v.to_f64().max(0.0))
                .unwrap_or(0.0);

            let territory_score = if total_treasury > 0.0 {
                ((treasury / total_treasury) * 10_000.0) as u64
            } else {
                0
            };

            // Belief score: proportional proxy via treasury share × global belief.
            let belief_score = if total_treasury > 0.0 && belief_global > 0 {
                let share = treasury / total_treasury;
                (share * belief_global as f64) as u64
            } else {
                0
            };

            // Legends score: sum of trade route volumes involving this faction.
            let legends_score: u64 = state
                .trade_routes
                .iter()
                .filter(|r| r.from_faction == fid || r.to_faction == fid)
                .map(|r| (r.volume.to_f64().max(0.0) * 100.0) as u64)
                .sum();

            let total = FactionScore::compute_total(
                pop_score_base,
                tech_score,
                belief_score,
                territory_score,
                legends_score,
            );

            FactionScore {
                faction_id: fid,
                pop_score: pop_score_base,
                tech_score,
                belief_score,
                territory_score,
                legends_score,
                total,
            }
        })
        .collect();

    // Deterministic sort: descending total, tie-break by faction_id ascending.
    scores.sort_by(|a, b| b.total.cmp(&a.total).then(a.faction_id.cmp(&b.faction_id)));
    scores
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::engine::Simulation;
    use crate::Fixed;

    // Helper: a fresh sim with three default factions
    fn fresh_sim() -> Simulation {
        Simulation::with_seed(42)
    }

    // ── Victory condition tests ───────────────────────────────────────────────

    #[test]
    fn domination_victory_triggers_at_threshold() {
        let mut sim = fresh_sim();
        // Give faction 0 the entire treasury — 100 % share ≥ 75 %
        sim.state.faction_treasury.insert(0, Fixed::from_num(10_000i64));
        sim.state.faction_treasury.insert(1, Fixed::from_num(0i64));
        sim.state.faction_treasury.insert(2, Fixed::from_num(0i64));

        let cond = VictoryCondition {
            victory_type: VictoryType::Domination,
            faction_id: 0,
            threshold: None,
        };
        assert!(cond.evaluate(&sim).is_some(), "should fire domination victory");
    }

    #[test]
    fn domination_victory_not_triggered_below_threshold() {
        let sim = fresh_sim(); // balanced treasury ~ 38 % each
        let cond = VictoryCondition {
            victory_type: VictoryType::Domination,
            faction_id: 0,
            threshold: None,
        };
        assert!(cond.evaluate(&sim).is_none(), "balanced sim should not trigger domination");
    }

    #[test]
    fn cultural_victory_triggers_at_threshold() {
        let mut sim = fresh_sim();
        sim.state.belief = 1_000; // non-zero so belief check is active
        // Give faction 0 dominant treasury share → belief_share proxy fires
        sim.state.faction_treasury.insert(0, Fixed::from_num(90_000i64));
        sim.state.faction_treasury.insert(1, Fixed::from_num(1i64));
        sim.state.faction_treasury.insert(2, Fixed::from_num(1i64));

        let cond = VictoryCondition {
            victory_type: VictoryType::Cultural,
            faction_id: 0,
            threshold: None,
        };
        assert!(cond.evaluate(&sim).is_some(), "should fire cultural victory");
    }

    #[test]
    fn cultural_victory_requires_nonzero_belief() {
        let mut sim = fresh_sim();
        sim.state.belief = 0;
        sim.state.faction_treasury.insert(0, Fixed::from_num(90_000i64));
        sim.state.faction_treasury.insert(1, Fixed::from_num(1i64));
        sim.state.faction_treasury.insert(2, Fixed::from_num(1i64));

        let cond = VictoryCondition {
            victory_type: VictoryType::Cultural,
            faction_id: 0,
            threshold: None,
        };
        assert!(cond.evaluate(&sim).is_none(), "zero belief should not trigger cultural victory");
    }

    #[test]
    fn economic_victory_triggers_at_threshold() {
        let mut sim = fresh_sim();
        // Give faction 0 enormous resources
        sim.state.faction_resources.insert(
            0,
            crate::engine::Resources {
                food: Fixed::from_num(1_000_000i64),
                wood: Fixed::from_num(1_000_000i64),
                metal: Fixed::from_num(1_000_000i64),
                energy: Fixed::from_num(1_000_000i64),
            },
        );
        sim.state.faction_resources.insert(
            1,
            crate::engine::Resources {
                food: Fixed::from_num(1i64),
                wood: Fixed::from_num(1i64),
                metal: Fixed::from_num(1i64),
                energy: Fixed::from_num(1i64),
            },
        );
        sim.state.faction_resources.insert(
            2,
            crate::engine::Resources {
                food: Fixed::from_num(1i64),
                wood: Fixed::from_num(1i64),
                metal: Fixed::from_num(1i64),
                energy: Fixed::from_num(1i64),
            },
        );

        let cond = VictoryCondition {
            victory_type: VictoryType::Economic,
            faction_id: 0,
            threshold: None,
        };
        assert!(cond.evaluate(&sim).is_some(), "should fire economic victory");
    }

    // ── Defeat condition tests ────────────────────────────────────────────────

    #[test]
    fn defeat_on_extinction() {
        let mut sim = fresh_sim();
        sim.state.population = 0;
        let gs = compute_gameplay_state(&sim);
        assert!(
            matches!(gs.resolved_outcome, Some(GameOutcome::Defeat(_))),
            "should detect extinction defeat"
        );
        let reason = gs.resolved_outcome.unwrap();
        assert!(
            reason.reason().contains("Extinction"),
            "reason should mention Extinction, got: {}",
            reason.reason()
        );
    }

    #[test]
    fn collapse_detected_when_faction_loses_all_treasury() {
        let mut sim = fresh_sim();
        sim.state.faction_treasury.insert(0, Fixed::from_num(0i64));
        sim.state.faction_treasury.insert(1, Fixed::from_num(1_000i64));
        sim.state.faction_treasury.insert(2, Fixed::from_num(1_000i64));

        assert!(check_collapse(&sim.state, 0), "faction 0 should be collapsed");
        assert!(!check_collapse(&sim.state, 1), "faction 1 should not be collapsed");
    }

    // ── Scenario objective tests ──────────────────────────────────────────────

    #[test]
    fn scenario_survive_1000_parses_correctly() {
        let obj = scenario_survive_1000(0);
        assert_eq!(obj.condition.victory_type, VictoryType::Domination);
        assert_eq!(obj.condition.faction_id, 0);
        assert_eq!(obj.tick_limit, Some(1_000));
        assert_eq!(obj.condition.effective_threshold(), 0.0);
    }

    #[test]
    fn scenario_cultural_dominance_parses_correctly() {
        let obj = scenario_cultural_dominance(1);
        assert_eq!(obj.condition.victory_type, VictoryType::Cultural);
        assert_eq!(obj.condition.faction_id, 1);
        assert_eq!(obj.tick_limit, Some(2_000));
        assert_eq!(obj.condition.effective_threshold(), CULTURAL_BELIEF_THRESHOLD);
    }

    #[test]
    fn scenario_objective_expires_at_tick_limit() {
        let mut sim = fresh_sim();
        // Tick past the limit without satisfying the condition.
        sim.state.tick = 2_001;
        sim.state.belief = 0; // cultural condition will never fire

        let obj = scenario_cultural_dominance(0);
        let result = obj.evaluate(&sim);
        assert!(
            matches!(result, Some(GameOutcome::Defeat(_))),
            "expired objective should return Defeat"
        );
    }

    #[test]
    fn scenario_objective_pending_before_tick_limit() {
        let sim = fresh_sim(); // tick = 0, belief = 0
        let obj = scenario_cultural_dominance(0);
        let result = obj.evaluate(&sim);
        assert!(result.is_none(), "should be None while still in progress");
    }

    // ── Scoring tests ─────────────────────────────────────────────────────────

    #[test]
    fn scores_compute_without_panic() {
        let sim = fresh_sim();
        let scores = compute_scores(&sim);
        assert!(!scores.is_empty(), "should produce at least one score");
        for s in &scores {
            assert_eq!(
                s.total,
                s.pop_score
                    .saturating_add(s.tech_score)
                    .saturating_add(s.belief_score)
                    .saturating_add(s.territory_score)
                    .saturating_add(s.legends_score),
                "total must equal sum of sub-scores"
            );
        }
    }

    #[test]
    fn score_determinism_same_inputs_same_output() {
        let sim_a = Simulation::with_seed(7);
        let sim_b = Simulation::with_seed(7);
        let scores_a = compute_scores(&sim_a);
        let scores_b = compute_scores(&sim_b);
        assert_eq!(scores_a, scores_b, "scores must be deterministic for identical seeds");
    }

    #[test]
    fn higher_treasury_yields_higher_territory_score() {
        let mut sim = fresh_sim();
        sim.state.faction_treasury.insert(0, Fixed::from_num(50_000i64));
        sim.state.faction_treasury.insert(1, Fixed::from_num(10i64));
        sim.state.faction_treasury.insert(2, Fixed::from_num(10i64));

        let scores = compute_scores(&sim);
        let s0 = scores.iter().find(|s| s.faction_id == 0).unwrap();
        let s1 = scores.iter().find(|s| s.faction_id == 1).unwrap();
        assert!(
            s0.territory_score > s1.territory_score,
            "dominant faction should have higher territory score"
        );
    }

    #[test]
    fn scores_sorted_descending_by_total() {
        let mut sim = fresh_sim();
        sim.state.faction_treasury.insert(0, Fixed::from_num(90_000i64));
        sim.state.faction_treasury.insert(1, Fixed::from_num(10i64));
        sim.state.faction_treasury.insert(2, Fixed::from_num(10i64));

        let scores = compute_scores(&sim);
        for window in scores.windows(2) {
            assert!(
                window[0].total >= window[1].total,
                "scores must be sorted descending"
            );
        }
    }

    // ── GameplayState integration ─────────────────────────────────────────────

    #[test]
    fn gameplay_state_ongoing_on_balanced_sim() {
        let sim = fresh_sim();
        let gs = compute_gameplay_state(&sim);
        assert!(
            gs.resolved_outcome.is_none(),
            "balanced sim should have no resolved outcome"
        );
    }

    #[test]
    fn gameplay_state_detects_domination_victory() {
        let mut sim = fresh_sim();
        sim.state.faction_treasury.insert(0, Fixed::from_num(100_000i64));
        sim.state.faction_treasury.insert(1, Fixed::from_num(1i64));
        sim.state.faction_treasury.insert(2, Fixed::from_num(1i64));

        let gs = compute_gameplay_state(&sim);
        assert!(
            matches!(gs.resolved_outcome, Some(GameOutcome::Victory(_))),
            "should detect domination victory in gameplay state"
        );
    }
}
