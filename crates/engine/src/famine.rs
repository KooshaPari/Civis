//! Food collapse cascade model (FR-CIV-FAMINE-001).
//!
//! Food shortage propagates through society via four cascade stages:
//! 1. **Hungry** — food_per_capita < 0.5: unrest +10%, labor -5%
//! 2. **Starving** — food_per_capita < 0.3: unrest +25%, labor -20%, migration pressure
//! 3. **Famine** — food_per_capita < 0.1: unrest +50%, labor -50%, emigration, trade desperation
//! 4. **Collapse** — food_per_capita == 0: settlement dissolution risk
//!
//! Each stage feeds into the existing subsystems:
//! - Unrest → faction_decisions (protests, regime change)
//! - Labor reduction → economy (lower production)
//! - Migration pressure → agents (emigration decisions)
//! - Trade desperation → diplomacy (forced trade agreements)

use serde::{Deserialize, Serialize};

/// Severity of food shortage in a settlement.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum FamineStage {
    /// Food per capita >= 0.5: no famine effects.
    None,
    /// Food per capita < 0.5: mild hunger, slight unrest and labor reduction.
    Hungry,
    /// Food per capita < 0.3: significant starvation, migration pressure.
    Starving,
    /// Food per capita < 0.1: full famine, mass emigration, trade desperation.
    Famine,
    /// Food per capita == 0: total collapse, settlement dissolution risk.
    Collapse,
}

/// Cascade effects at each famine stage.
#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize)]
pub struct FamineEffects {
    /// Additional unrest modifier (additive).
    pub unrest_delta: f32,
    /// Labor productivity multiplier (0.0–1.0).
    pub labor_multiplier: f32,
    /// Migration pressure per tick (0.0–1.0).
    pub migration_pressure: f32,
    /// Trade desperation modifier (affects diplomacy willingness).
    pub trade_desperation: f32,
    /// Population death risk per tick (probability).
    pub death_risk: f32,
}

impl FamineEffects {
    /// No famine — no effects.
    pub const NONE: Self = Self {
        unrest_delta: 0.0,
        labor_multiplier: 1.0,
        migration_pressure: 0.0,
        trade_desperation: 0.0,
        death_risk: 0.0,
    };

    /// Accumulate effects across multiple agents or ticks.
    pub fn accumulate(&mut self, other: &Self) {
        self.unrest_delta += other.unrest_delta;
        self.labor_multiplier *= other.labor_multiplier;
        self.migration_pressure = self.migration_pressure.max(other.migration_pressure);
        self.trade_desperation = self.trade_desperation.max(other.trade_desperation);
        self.death_risk += other.death_risk;
    }

    /// Clamp all values to valid ranges.
    pub fn clamped(mut self) -> Self {
        self.unrest_delta = self.unrest_delta.clamp(0.0, 1.0);
        self.labor_multiplier = self.labor_multiplier.clamp(0.0, 1.0);
        self.migration_pressure = self.migration_pressure.clamp(0.0, 1.0);
        self.trade_desperation = self.trade_desperation.clamp(0.0, 1.0);
        self.death_risk = self.death_risk.clamp(0.0, 0.1);
        self
    }
}

/// Determine the famine stage from food per capita (0.0–1.0+).
pub fn classify_famine(food_per_capita: f32) -> FamineStage {
    if food_per_capita <= 0.0 {
        FamineStage::Collapse
    } else if food_per_capita < 0.1 {
        FamineStage::Famine
    } else if food_per_capita < 0.3 {
        FamineStage::Starving
    } else if food_per_capita < 0.5 {
        FamineStage::Hungry
    } else {
        FamineStage::None
    }
}

/// Get cascade effects for a given famine stage.
pub fn famine_effects(stage: FamineStage) -> FamineEffects {
    match stage {
        FamineStage::None => FamineEffects::NONE,
        FamineStage::Hungry => FamineEffects {
            unrest_delta: 0.10,
            labor_multiplier: 0.95,
            migration_pressure: 0.0,
            trade_desperation: 0.0,
            death_risk: 0.0,
        },
        FamineStage::Starving => FamineEffects {
            unrest_delta: 0.25,
            labor_multiplier: 0.80,
            migration_pressure: 0.15,
            trade_desperation: 0.20,
            death_risk: 0.002,
        },
        FamineStage::Famine => FamineEffects {
            unrest_delta: 0.50,
            labor_multiplier: 0.50,
            migration_pressure: 0.40,
            trade_desperation: 0.60,
            death_risk: 0.01,
        },
        FamineStage::Collapse => FamineEffects {
            unrest_delta: 0.80,
            labor_multiplier: 0.20,
            migration_pressure: 0.80,
            trade_desperation: 0.90,
            death_risk: 0.05,
        },
    }
}

/// Compute food per capita for a settlement.
///
/// `food_stock` is the settlement's total food supply.
/// `population` is the number of agents in the settlement.
/// Returns a value in `[0.0, 2.0+]` where 1.0 means exactly enough food for everyone.
pub fn food_per_capita(food_stock: f32, population: u32) -> f32 {
    if population == 0 {
        // No population: no famine concern (or settlement is empty → collapse).
        if food_stock > 0.0 {
            f32::INFINITY // Surplus with no one to eat it
        } else {
            0.0 // Empty and destitute
        }
    } else {
        food_stock / population as f32
    }
}

/// Compute the aggregate famine stage for a settlement given food stock and population.
pub fn settlement_famine(food_stock: f32, population: u32) -> (FamineStage, FamineEffects) {
    let fpc = food_per_capita(food_stock, population);
    let stage = classify_famine(fpc);
    let effects = famine_effects(stage);
    (stage, effects)
}

/// Accumulate famine effects across multiple settlements.
///
/// Returns the aggregate effects (weighted by population) and the worst stage observed.
pub fn aggregate_famine(
    settlements: &[(f32, u32)], // (food_stock, population) per settlement
) -> (FamineStage, FamineEffects) {
    if settlements.is_empty() {
        return (FamineStage::None, FamineEffects::NONE);
    }

    let total_pop: u32 = settlements.iter().map(|(_, p)| *p).sum();
    if total_pop == 0 {
        return (FamineStage::Collapse, famine_effects(FamineStage::Collapse));
    }

    let total_food: f32 = settlements.iter().map(|(f, _)| *f).sum();
    let global_fpc = total_food / total_pop as f32;
    let global_stage = classify_famine(global_fpc);

    // Aggregate effects weighted by population fraction
    let mut effects = FamineEffects::NONE;
    for &(food, pop) in settlements {
        if pop == 0 {
            continue;
        }
        let (_stage, stage_effects) = settlement_famine(food, pop);
        let weight = pop as f32 / total_pop as f32;

        effects.unrest_delta += stage_effects.unrest_delta * weight;
        effects.labor_multiplier *= stage_effects.labor_multiplier.powf(weight);
        effects.migration_pressure = effects
            .migration_pressure
            .max(stage_effects.migration_pressure * weight);
        effects.trade_desperation = effects
            .trade_desperation
            .max(stage_effects.trade_desperation * weight);
        effects.death_risk += stage_effects.death_risk * weight;
    }

    (global_stage, effects.clamped())
}

/// Event generated when famine stage transitions occur.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FamineEvent {
    /// Settlement index (or aggregate).
    pub settlement_id: u32,
    /// Previous stage.
    pub from: FamineStage,
    /// New stage.
    pub to: FamineStage,
    /// Food per capita at time of transition.
    pub food_per_capita: f32,
}

impl FamineEvent {
    /// Returns true if this transition is an escalation (worsening).
    pub fn is_escalation(&self) -> bool {
        self.to > self.from
    }

    /// Returns true if this transition is a deescalation (improving).
    pub fn is_deescalation(&self) -> bool {
        self.to < self.from
    }
}

/// Detect famine stage transitions and emit events.
///
/// Compare old and new food-per-capita values to detect stage changes.
pub fn detect_transition(
    settlement_id: u32,
    old_food_per_capita: f32,
    new_food_per_capita: f32,
) -> Option<FamineEvent> {
    let old_stage = classify_famine(old_food_per_capita);
    let new_stage = classify_famine(new_food_per_capita);

    if old_stage != new_stage {
        Some(FamineEvent {
            settlement_id,
            from: old_stage,
            to: new_stage,
            food_per_capita: new_food_per_capita,
        })
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ── Classification ────────────────────────────────────────────────

    #[test]
    fn well_fed_is_none() {
        assert_eq!(classify_famine(1.0), FamineStage::None);
        assert_eq!(classify_famine(0.6), FamineStage::None);
    }

    #[test]
    fn hungry_below_half() {
        assert_eq!(classify_famine(0.49), FamineStage::Hungry);
        assert_eq!(classify_famine(0.31), FamineStage::Hungry);
    }

    #[test]
    fn starving_below_third() {
        assert_eq!(classify_famine(0.29), FamineStage::Starving);
        assert_eq!(classify_famine(0.11), FamineStage::Starving);
    }

    #[test]
    fn famine_below_tenth() {
        assert_eq!(classify_famine(0.09), FamineStage::Famine);
        assert_eq!(classify_famine(0.01), FamineStage::Famine);
    }

    #[test]
    fn collapse_at_zero() {
        assert_eq!(classify_famine(0.0), FamineStage::Collapse);
        assert_eq!(classify_famine(-0.1), FamineStage::Collapse);
    }

    // ── Effects ───────────────────────────────────────────────────────

    #[test]
    fn hungry_effects_are_mild() {
        let e = famine_effects(FamineStage::Hungry);
        assert!(e.unrest_delta < 0.2, "hungry unrest should be mild");
        assert!(e.labor_multiplier > 0.9, "hungry labor should be near full");
        assert_eq!(e.migration_pressure, 0.0, "no migration at hungry stage");
    }

    #[test]
    fn famine_effects_are_severe() {
        let e = famine_effects(FamineStage::Famine);
        assert!(e.unrest_delta >= 0.4, "famine unrest should be high");
        assert!(e.labor_multiplier <= 0.6, "famine labor should be halved");
        assert!(e.migration_pressure > 0.3, "famine should drive migration");
        assert!(e.death_risk > 0.0, "famine should have death risk");
    }

    #[test]
    fn collapse_is_worst() {
        let c = famine_effects(FamineStage::Collapse);
        assert!(c.unrest_delta >= 0.7);
        assert!(c.labor_multiplier <= 0.3);
        assert!(c.migration_pressure >= 0.7);
    }

    #[test]
    fn effects_monotonically_worsen() {
        let stages = [
            FamineStage::None,
            FamineStage::Hungry,
            FamineStage::Starving,
            FamineStage::Famine,
            FamineStage::Collapse,
        ];
        let mut prev_unrest = -1.0;
        let mut prev_labor = 2.0;
        for &s in &stages {
            let e = famine_effects(s);
            assert!(
                e.unrest_delta >= prev_unrest,
                "unrest should increase with stage: {s:?}"
            );
            assert!(
                e.labor_multiplier <= prev_labor,
                "labor should decrease with stage: {s:?}"
            );
            prev_unrest = e.unrest_delta;
            prev_labor = e.labor_multiplier;
        }
    }

    // ── Food per capita ───────────────────────────────────────────────

    #[test]
    fn food_per_capita_basic() {
        assert!((food_per_capita(10.0, 10) - 1.0).abs() < 0.001);
        assert!((food_per_capita(5.0, 10) - 0.5).abs() < 0.001);
        assert!((food_per_capita(0.0, 10) - 0.0).abs() < 0.001);
    }

    #[test]
    fn food_per_capita_empty_pop() {
        assert!(food_per_capita(5.0, 0).is_infinite());
        assert!((food_per_capita(0.0, 0) - 0.0).abs() < 0.001);
    }

    // ── Settlement famine ─────────────────────────────────────────────

    #[test]
    fn well_fed_settlement_is_none() {
        let (stage, _) = settlement_famine(10.0, 10);
        assert_eq!(stage, FamineStage::None);
    }

    #[test]
    fn starving_settlement_is_detected() {
        let (stage, effects) = settlement_famine(2.0, 10); // fpc = 0.2
        assert_eq!(stage, FamineStage::Starving);
        assert!(effects.migration_pressure > 0.0);
    }

    // ── Aggregate ─────────────────────────────────────────────────────

    #[test]
    fn aggregate_none_when_all_well_fed() {
        let settlements = vec![(10.0, 5), (10.0, 5)];
        let (stage, _) = aggregate_famine(&settlements);
        assert_eq!(stage, FamineStage::None);
    }

    #[test]
    fn aggregate_collapse_when_no_food() {
        let settlements = vec![(0.0, 10), (0.0, 10)];
        let (stage, _) = aggregate_famine(&settlements);
        assert_eq!(stage, FamineStage::Collapse);
    }

    #[test]
    fn aggregate_empty_settlements() {
        let (stage, _) = aggregate_famine(&[]);
        assert_eq!(stage, FamineStage::None);
    }

    // ── Transitions ───────────────────────────────────────────────────

    #[test]
    fn detect_no_transition_when_stable() {
        let event = detect_transition(0, 0.8, 0.7);
        assert!(event.is_none());
    }

    #[test]
    fn detect_escalation() {
        let event = detect_transition(0, 0.6, 0.2).unwrap();
        assert!(event.is_escalation());
        assert!(!event.is_deescalation());
        assert_eq!(event.from, FamineStage::None);
        assert_eq!(event.to, FamineStage::Starving);
    }

    #[test]
    fn detect_deescalation() {
        let event = detect_transition(0, 0.05, 0.4).unwrap();
        assert!(!event.is_escalation());
        assert!(event.is_deescalation());
        assert_eq!(event.from, FamineStage::Famine);
        assert_eq!(event.to, FamineStage::Hungry);
    }

    // ── Effects accumulation ──────────────────────────────────────────

    #[test]
    fn accumulate_merges_effects() {
        let mut a = famine_effects(FamineStage::Hungry);
        let b = famine_effects(FamineStage::Starving);
        a.accumulate(&b);
        assert!(a.unrest_delta > 0.10, "should accumulate unrest");
        assert!(
            a.migration_pressure > 0.0,
            "should pick up migration from b"
        );
    }
}
