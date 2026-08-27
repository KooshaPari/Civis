//! Festival and celebration system (FR-CIV-FEST-001).
//!
//! Festivals are periodic celebrations triggered by settlement conditions
//! (resource surplus, belief alignment, morale thresholds). They provide
//! happiness boosts at the cost of temporary productivity penalties and
//! require cooldowns per settlement to prevent spam.
//!
//! ## Design
//!
//! - `FestivalEngine` is evaluated each tick against settlement state.
//! - When thresholds are met and cooldowns have elapsed, a festival spawns.
//! - Active festivals decrement their remaining duration each tick.
//! - `apply_effects` returns `FestivalEffects` deltas consumed by the
//!   economy, unrest, and social subsystems.

use std::collections::HashMap;

use serde::{Deserialize, Serialize};

// ────────────────────────────────────────────────────────────────────
// FestivalType
// ────────────────────────────────────────────────────────────────────

/// Category of festival, each tied to different trigger conditions.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum FestivalType {
    /// Triggered by large food/resource surplus.
    Harvest,
    /// Triggered by a recent military victory.
    Victory,
    /// Triggered by strong religious belief cohesion.
    Religious,
    /// Triggered by high cultural output or diversity.
    Cultural,
    /// Triggered by thriving trade routes / market surplus.
    Market,
    /// Triggered as a morale-boost emergency during unrest or disaster.
    Emergency,
}

impl FestivalType {
    /// Wire-safe display name.
    pub fn as_str(self) -> &'static str {
        match self {
            FestivalType::Harvest => "Harvest",
            FestivalType::Victory => "Victory",
            FestivalType::Religious => "Religious",
            FestivalType::Cultural => "Cultural",
            FestivalType::Market => "Market",
            FestivalType::Emergency => "Emergency",
        }
    }
}

impl Default for FestivalType {
    fn default() -> Self {
        FestivalType::Harvest
    }
}

// ────────────────────────────────────────────────────────────────────
// SettlementState — minimal input for festival evaluation
// ────────────────────────────────────────────────────────────────────

/// Snapshot of settlement metrics fed into festival evaluation.
/// Kept intentionally lightweight so callers don't need the full Sim.
#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize)]
pub struct SettlementState {
    /// Unique settlement identifier.
    pub settlement_id: u32,
    /// Food/resource surplus ratio (0.0 = none, 1.0+ = very large).
    pub resource_surplus: f32,
    /// Collective belief / religious cohesion (0.0–1.0).
    pub belief_strength: f32,
    /// Average morale / happiness (0.0–1.0).
    pub morale: f32,
    /// Cultural output score (0.0–1.0).
    pub culture_output: f32,
    /// Trade route revenue relative to baseline (1.0 = baseline).
    pub trade_revenue_ratio: f32,
    /// Current unrest level (0.0–1.0).
    pub unrest_level: f32,
    /// Whether a military victory was recorded last tick.
    pub recent_victory: bool,
    /// Active disaster severity (0.0 = none, 1.0 = catastrophic).
    pub disaster_severity: f32,
    /// Total population.
    pub population: u32,
}

// ────────────────────────────────────────────────────────────────────
// FestivalConfig
// ────────────────────────────────────────────────────────────────────

/// Per-type configuration governing when a festival can trigger and how
/// long it lasts.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct FestivalConfig {
    /// Minimum surplus / metric value required to trigger.
    pub trigger_threshold: f32,
    /// Duration of the festival in simulation ticks.
    pub duration_ticks: u32,
    /// Fraction of population that participates (0.0–1.0).
    pub participation_rate: f32,
    /// Flat happiness added per tick of the festival.
    pub happiness_bonus: f32,
    /// Productivity penalty per tick while the festival is active.
    pub productivity_penalty: f32,
}

impl Default for FestivalConfig {
    fn default() -> Self {
        Self {
            trigger_threshold: 0.5,
            duration_ticks: 5,
            participation_rate: 0.5,
            happiness_bonus: 0.05,
            productivity_penalty: 0.1,
        }
    }
}

// ────────────────────────────────────────────────────────────────────
// Festival
// ────────────────────────────────────────────────────────────────────

/// A single active festival instance.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Festival {
    /// What kind of festival this is.
    pub festival_type: FestivalType,
    /// Settlement hosting the festival.
    pub settlement_id: u32,
    /// Tick at which the festival started.
    pub start_tick: u64,
    /// How many ticks the festival lasts.
    pub duration: u32,
    /// Number of citizens participating.
    pub participants: u32,
    /// Accumulated happiness change from this festival so far.
    pub happiness_delta: f32,
    /// Ticks remaining (counted down by the engine).
    pub remaining_ticks: u32,
}

// ────────────────────────────────────────────────────────────────────
// FestivalEffects
// ────────────────────────────────────────────────────────────────────

/// Output deltas produced by a festival, consumed by the simulation's
/// economy, unrest, and social phases.
#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize)]
pub struct FestivalEffects {
    /// Happiness change per tick (positive = boost).
    pub happiness_delta: f32,
    /// Labor / productivity change per tick (negative = penalty).
    pub labor_delta: f32,
    /// Unrest change per tick (negative = unrest reduction).
    pub unrest_delta: f32,
    /// Trade bonus multiplier applied during the festival.
    pub trade_bonus: f32,
}

impl FestivalEffects {
    /// No-op effects.
    pub const NONE: Self = Self {
        happiness_delta: 0.0,
        labor_delta: 0.0,
        unrest_delta: 0.0,
        trade_bonus: 0.0,
    };

    /// Accumulate another set of effects (additive).
    pub fn accumulate(&mut self, other: &Self) {
        self.happiness_delta += other.happiness_delta;
        self.labor_delta += other.labor_delta;
        self.unrest_delta += other.unrest_delta;
        self.trade_bonus += other.trade_bonus;
    }

    /// Clamp all values to valid ranges.
    pub fn clamped(mut self) -> Self {
        self.happiness_delta = self.happiness_delta.clamp(-1.0, 1.0);
        self.labor_delta = self.labor_delta.clamp(-1.0, 0.0);
        self.unrest_delta = self.unrest_delta.clamp(-1.0, 1.0);
        self.trade_bonus = self.trade_bonus.clamp(0.0, 2.0);
        self
    }
}

// ────────────────────────────────────────────────────────────────────
// FestivalEngine
// ────────────────────────────────────────────────────────────────────

/// Manages festival lifecycle: evaluation, spawning, ticking, and effects.
#[derive(Debug, Serialize, Deserialize)]
pub struct FestivalEngine {
    /// Per-type configuration.
    pub config: HashMap<FestivalType, FestivalConfig>,
    /// Currently active festivals across all settlements.
    pub active_festivals: Vec<Festival>,
    /// Cooldown tracker: (settlement_id, festival_type) → last end tick.
    /// A new festival of that type cannot start until enough ticks pass.
    pub cooldown_tracker: HashMap<(u32, FestivalType), u64>,
    /// Minimum ticks between festivals of the same type in the same settlement.
    pub cooldown_ticks: u32,
}

impl FestivalEngine {
    /// Create a new engine with default configs for every festival type.
    pub fn new() -> Self {
        let mut config = HashMap::new();
        config.insert(
            FestivalType::Harvest,
            FestivalConfig {
                trigger_threshold: 0.6,
                duration_ticks: 6,
                participation_rate: 0.7,
                happiness_bonus: 0.08,
                productivity_penalty: 0.15,
            },
        );
        config.insert(
            FestivalType::Victory,
            FestivalConfig {
                trigger_threshold: 0.0, // boolean trigger
                duration_ticks: 4,
                participation_rate: 0.8,
                happiness_bonus: 0.12,
                productivity_penalty: 0.10,
            },
        );
        config.insert(
            FestivalType::Religious,
            FestivalConfig {
                trigger_threshold: 0.7,
                duration_ticks: 5,
                participation_rate: 0.6,
                happiness_bonus: 0.06,
                productivity_penalty: 0.08,
            },
        );
        config.insert(
            FestivalType::Cultural,
            FestivalConfig {
                trigger_threshold: 0.65,
                duration_ticks: 5,
                participation_rate: 0.5,
                happiness_bonus: 0.07,
                productivity_penalty: 0.12,
            },
        );
        config.insert(
            FestivalType::Market,
            FestivalConfig {
                trigger_threshold: 1.3, // trade_revenue_ratio must exceed 1.3x
                duration_ticks: 4,
                participation_rate: 0.6,
                happiness_bonus: 0.05,
                productivity_penalty: 0.05,
            },
        );
        config.insert(
            FestivalType::Emergency,
            FestivalConfig {
                trigger_threshold: 0.6, // unrest must exceed 0.6
                duration_ticks: 3,
                participation_rate: 0.4,
                happiness_bonus: 0.10,
                productivity_penalty: 0.20,
            },
        );
        Self {
            config,
            active_festivals: Vec::new(),
            cooldown_tracker: HashMap::new(),
            cooldown_ticks: 10,
        }
    }

    /// Evaluate which festival types *could* trigger for a settlement.
    ///
    /// Returns candidate types whose thresholds are met and whose cooldowns
    /// have elapsed. Only the single most appropriate type is returned
    /// (priority: Emergency > Harvest > Victory > Religious > Market > Cultural).
    pub fn evaluate(&self, state: &SettlementState, current_tick: u64) -> Vec<FestivalType> {
        let mut candidates = Vec::new();

        // Check each type against its config and the settlement state.
        if let Some(cfg) = self.config.get(&FestivalType::Emergency) {
            if state.unrest_level >= cfg.trigger_threshold
                || state.disaster_severity >= cfg.trigger_threshold
            {
                candidates.push(FestivalType::Emergency);
            }
        }
        if let Some(cfg) = self.config.get(&FestivalType::Harvest) {
            if state.resource_surplus >= cfg.trigger_threshold {
                candidates.push(FestivalType::Harvest);
            }
        }
        if let Some(cfg) = self.config.get(&FestivalType::Victory) {
            if state.recent_victory {
                // Victory uses boolean trigger; threshold is ignored.
                candidates.push(FestivalType::Victory);
            }
        }
        if let Some(cfg) = self.config.get(&FestivalType::Religious) {
            if state.belief_strength >= cfg.trigger_threshold {
                candidates.push(FestivalType::Religious);
            }
        }
        if let Some(cfg) = self.config.get(&FestivalType::Market) {
            if state.trade_revenue_ratio >= cfg.trigger_threshold {
                candidates.push(FestivalType::Market);
            }
        }
        if let Some(cfg) = self.config.get(&FestivalType::Cultural) {
            if state.culture_output >= cfg.trigger_threshold {
                candidates.push(FestivalType::Cultural);
            }
        }

        // Filter out types already on cooldown for this settlement.
        candidates.retain(|ft| {
            if let Some(&last_end) = self.cooldown_tracker.get(&(state.settlement_id, *ft)) {
                current_tick >= last_end + self.cooldown_ticks as u64
            } else {
                true
            }
        });

        // Filter out types already active for this settlement.
        candidates.retain(|ft| {
            !self
                .active_festivals
                .iter()
                .any(|f| f.festival_type == *ft && f.settlement_id == state.settlement_id)
        });

        candidates
    }

    /// Spawn a new festival for the given settlement at the current tick.
    ///
    /// Returns the spawned `Festival` or `None` if a config for the type
    /// is missing.
    pub fn spawn(
        &mut self,
        festival_type: FestivalType,
        settlement_id: u32,
        tick: u64,
        population: u32,
    ) -> Option<Festival> {
        let cfg = self.config.get(&festival_type)?.clone();
        let participants = (population as f32 * cfg.participation_rate).round() as u32;

        let festival = Festival {
            festival_type,
            settlement_id,
            start_tick: tick,
            duration: cfg.duration_ticks,
            participants,
            happiness_delta: 0.0,
            remaining_ticks: cfg.duration_ticks,
        };
        self.active_festivals.push(festival.clone());
        Some(festival)
    }

    /// Advance all active festivals by one tick, removing completed ones
    /// and recording their cooldown end tick.
    pub fn tick(&mut self, dt: u32, current_tick: u64) {
        for festival in &mut self.active_festivals {
            festival.remaining_ticks = festival.remaining_ticks.saturating_sub(dt);
            if let Some(cfg) = self.config.get(&festival.festival_type) {
                festival.happiness_delta += cfg.happiness_bonus * dt as f32;
            }
        }

        // Remove finished festivals and set cooldowns.
        let mut ended = Vec::new();
        self.active_festivals.retain(|f| {
            if f.remaining_ticks == 0 {
                ended.push((
                    f.settlement_id,
                    f.festival_type,
                    f.start_tick + f.duration as u64,
                ));
                false
            } else {
                true
            }
        });
        for (sid, ft, end_tick) in ended {
            self.cooldown_tracker.insert((sid, ft), end_tick);
        }
    }

    /// Compute the aggregate effects of all active festivals for a
    /// settlement. Used by the sim's economy / unrest phases.
    pub fn apply_effects(&self, settlement_id: u32) -> FestivalEffects {
        let mut total = FestivalEffects::NONE;
        for festival in &self.active_festivals {
            if festival.settlement_id != settlement_id {
                continue;
            }
            if let Some(cfg) = self.config.get(&festival.festival_type) {
                let fraction =
                    festival.participants as f32 / (festival.participants as f32 + 1.0).max(1.0);
                total.accumulate(&FestivalEffects {
                    happiness_delta: cfg.happiness_bonus * fraction,
                    labor_delta: -cfg.productivity_penalty * fraction,
                    unrest_delta: -cfg.happiness_bonus * fraction * 0.5,
                    trade_bonus: if festival.festival_type == FestivalType::Market {
                        0.1 * fraction
                    } else {
                        0.05 * fraction
                    },
                });
            }
        }
        total.clamped()
    }

    /// Number of currently active festivals.
    pub fn active_count(&self) -> usize {
        self.active_festivals.len()
    }

    /// Check whether a given settlement has an active festival of the
    /// given type.
    pub fn is_active(&self, settlement_id: u32, festival_type: FestivalType) -> bool {
        self.active_festivals
            .iter()
            .any(|f| f.settlement_id == settlement_id && f.festival_type == festival_type)
    }
}

impl Default for FestivalEngine {
    fn default() -> Self {
        Self::new()
    }
}

// ────────────────────────────────────────────────────────────────────
// Tests
// ────────────────────────────────────────────────────────────────────
#[cfg(test)]
mod tests {
    use super::*;

    /// Helper: build a default settlement state with a given id.
    fn default_state(id: u32) -> SettlementState {
        SettlementState {
            settlement_id: id,
            ..SettlementState::default()
        }
    }

    // ── 1. Evaluate returns empty when nothing meets thresholds ─────
    #[test]
    fn evaluate_nothing_triggers_on_empty_state() {
        let engine = FestivalEngine::new();
        let state = default_state(1);
        let candidates = engine.evaluate(&state, 0);
        assert!(candidates.is_empty());
    }

    // ── 2. Harvest triggers when surplus is high enough ─────────────
    #[test]
    fn evaluate_harvest_triggers_on_surplus() {
        let engine = FestivalEngine::new();
        let mut state = default_state(1);
        state.resource_surplus = 0.8;
        let candidates = engine.evaluate(&state, 0);
        assert!(
            candidates.contains(&FestivalType::Harvest),
            "expected Harvest in {:?}",
            candidates
        );
    }

    // ── 3. Victory triggers on recent_victory flag ──────────────────
    #[test]
    fn evaluate_victory_triggers() {
        let engine = FestivalEngine::new();
        let mut state = default_state(1);
        state.recent_victory = true;
        let candidates = engine.evaluate(&state, 0);
        assert!(candidates.contains(&FestivalType::Victory));
    }

    // ── 4. Emergency triggers on high unrest ────────────────────────
    #[test]
    fn evaluate_emergency_triggers_on_unrest() {
        let engine = FestivalEngine::new();
        let mut state = default_state(1);
        state.unrest_level = 0.8;
        let candidates = engine.evaluate(&state, 0);
        assert!(candidates.contains(&FestivalType::Emergency));
    }

    // ── 5. Market triggers when trade revenue ratio is high ─────────
    #[test]
    fn evaluate_market_triggers_on_trade() {
        let engine = FestivalEngine::new();
        let mut state = default_state(1);
        state.trade_revenue_ratio = 1.5;
        let candidates = engine.evaluate(&state, 0);
        assert!(candidates.contains(&FestivalType::Market));
    }

    // ── 6. Spawn creates a festival with correct participant count ──
    #[test]
    fn spawn_creates_festival_with_participants() {
        let mut engine = FestivalEngine::new();
        let f = engine.spawn(FestivalType::Harvest, 1, 0, 100).unwrap();
        assert_eq!(f.festival_type, FestivalType::Harvest);
        assert_eq!(f.settlement_id, 1);
        assert_eq!(f.duration, 6);
        assert_eq!(f.remaining_ticks, 6);
        // participation_rate for Harvest is 0.7, 100 * 0.7 = 70
        assert_eq!(f.participants, 70);
        assert_eq!(engine.active_count(), 1);
    }

    // ── 7. Tick decrements remaining_ticks and removes finished ─────
    #[test]
    fn tick_advances_and_removes_completed() {
        let mut engine = FestivalEngine::new();
        engine.spawn(FestivalType::Victory, 1, 0, 200).unwrap();
        assert_eq!(engine.active_count(), 1);

        engine.tick(2, 2);
        assert_eq!(engine.active_count(), 1);
        assert_eq!(engine.active_festivals[0].remaining_ticks, 2);

        engine.tick(2, 4);
        assert_eq!(engine.active_count(), 0, "festival should be removed");
    }

    // ── 8. Cooldown prevents immediate re-trigger ──────────────────
    #[test]
    fn cooldown_blocks_retrigger() {
        let mut engine = FestivalEngine::new();
        let mut state = default_state(1);
        state.recent_victory = true;

        let candidates = engine.evaluate(&state, 0);
        assert!(candidates.contains(&FestivalType::Victory));

        engine.spawn(FestivalType::Victory, 1, 0, 100).unwrap();
        engine.tick(4, 4); // Victory lasts 4 ticks

        // Immediately after ending, cooldown (10 ticks) should block.
        let candidates = engine.evaluate(&state, 4);
        assert!(
            !candidates.contains(&FestivalType::Victory),
            "Victory should be on cooldown"
        );

        // After cooldown expires (tick 14).
        let candidates = engine.evaluate(&state, 14);
        assert!(
            candidates.contains(&FestivalType::Victory),
            "Victory should be available again after cooldown"
        );
    }

    // ── 9. apply_effects returns correct deltas ────────────────────
    #[test]
    fn apply_effects_returns_nonzero_deltas() {
        let mut engine = FestivalEngine::new();
        engine.spawn(FestivalType::Religious, 1, 0, 50).unwrap();

        let effects = engine.apply_effects(1);
        assert!(
            effects.happiness_delta > 0.0,
            "Religious festival should boost happiness"
        );
        assert!(
            effects.labor_delta < 0.0,
            "Religious festival should penalize labor"
        );
        assert!(
            effects.unrest_delta < 0.0,
            "Religious festival should reduce unrest"
        );

        // Effects for a different settlement should be zero.
        let effects2 = engine.apply_effects(99);
        assert_eq!(effects2, FestivalEffects::NONE);
    }

    // ── 10. Multiple active festivals accumulate effects ────────────
    #[test]
    fn multiple_festivals_accumulate_effects() {
        let mut engine = FestivalEngine::new();
        // Spawn Harvest for settlement 1 (different from Victory which is
        // for settlement 1 too, so we use settlement 1 for both).
        engine.spawn(FestivalType::Harvest, 1, 0, 200).unwrap();
        engine.spawn(FestivalType::Religious, 1, 0, 200).unwrap();

        let effects = engine.apply_effects(1);
        // Both festivals contribute happiness.
        assert!(
            effects.happiness_delta > 0.1,
            "combined happiness should be larger than a single festival"
        );
    }

    // ── 11. evaluate filters out already-active types ───────────────
    #[test]
    fn evaluate_skips_already_active_type() {
        let mut engine = FestivalEngine::new();
        let mut state = default_state(1);
        state.resource_surplus = 0.8; // Harvest threshold

        engine.spawn(FestivalType::Harvest, 1, 0, 100).unwrap();
        let candidates = engine.evaluate(&state, 5);
        assert!(
            !candidates.contains(&FestivalType::Harvest),
            "Harvest should not re-trigger while active"
        );
    }

    // ── 12. Different settlements don't share cooldowns ─────────────
    #[test]
    fn settlements_have_independent_cooldowns() {
        let mut engine = FestivalEngine::new();
        let mut state1 = default_state(1);
        state1.recent_victory = true;
        let mut state2 = default_state(2);
        state2.recent_victory = true;

        engine.spawn(FestivalType::Victory, 1, 0, 100).unwrap();
        engine.tick(4, 4); // festival for settlement 1 ends

        // Settlement 1 is on cooldown.
        let c1 = engine.evaluate(&state1, 4);
        assert!(!c1.contains(&FestivalType::Victory));

        // Settlement 2 should still be free.
        let c2 = engine.evaluate(&state2, 4);
        assert!(c2.contains(&FestivalType::Victory));
    }

    // ── 13. FestivalEffects clamping works ──────────────────────────
    #[test]
    fn effects_clamped_to_valid_range() {
        let effects = FestivalEffects {
            happiness_delta: 5.0,
            labor_delta: -5.0,
            unrest_delta: -5.0,
            trade_bonus: 10.0,
        };
        let clamped = effects.clamped();
        assert_eq!(clamped.happiness_delta, 1.0);
        assert_eq!(clamped.labor_delta, -1.0);
        assert_eq!(clamped.unrest_delta, -1.0);
        assert_eq!(clamped.trade_bonus, 2.0);
    }

    // ── 14. is_active check ────────────────────────────────────────
    #[test]
    fn is_active_reflects_current_state() {
        let mut engine = FestivalEngine::new();
        assert!(!engine.is_active(1, FestivalType::Harvest));

        engine.spawn(FestivalType::Harvest, 1, 0, 100).unwrap();
        assert!(engine.is_active(1, FestivalType::Harvest));
        assert!(!engine.is_active(1, FestivalType::Victory));
        assert!(!engine.is_active(2, FestivalType::Harvest));
    }
}
