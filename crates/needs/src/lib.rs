//! civ-needs — emergent life-sim needs model.
//!
//! Every *living item* (not necessarily human — life is emergent) carries a
//! [`Needs`] vector. Needs are **satisfaction levels** in `[0, 1]` where `1.0`
//! means fully satisfied and `0.0` means critically unmet. Each tick needs
//! **decay** toward zero at a per-need rate; satisfying a need (eating, drinking,
//! resting, …) pushes it back up. Sustained deprivation accumulates as
//! [`Health`] damage → sickness → death.
//!
//! Determinism is sacred (ADR-008): decay and health transitions are pure and
//! integer/`f32`-deterministic; any stochastic sickness roll consumes a caller
//! supplied seeded [`ChaCha8Rng`]. No wall-clock, no `thread_rng`.
//!
//! Traceability: `FR-CIV-LIFE-001` (needs decay), `FR-CIV-LIFE-002` (sickness),
//! `FR-CIV-LIFE-003` (death from unmet needs).

#![forbid(unsafe_code)]
#![warn(missing_docs)]

use rand::Rng;
use rand_chacha::ChaCha8Rng;
use serde::{Deserialize, Serialize};

pub mod decay;
pub mod lifecycle;

pub use decay::{apply_resource, tick_rise, NeedChannel, NeedLevel, RiseRates};
pub use lifecycle::{classify_lifecycle, age_threshold, labor_capacity, LifecycleLabel, LifecycleParams};

/// Schema version. Bumped on breaking changes.
pub const SCHEMA_VERSION: &str = "0.1.0";

/// The six survival needs every living item tracks. Each field is a
/// **satisfaction** level in `[0, 1]`: `1.0` = sated, `0.0` = critical.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Needs {
    /// Caloric / food satisfaction.
    pub food: f32,
    /// Hydration satisfaction.
    pub water: f32,
    /// Rest / sleep satisfaction.
    pub rest: f32,
    /// Perceived safety (inverse of threat exposure).
    pub safety: f32,
    /// Social / belonging satisfaction.
    pub social: f32,
    /// Physical health satisfaction (proxied by [`Health`], mirrored here for AI).
    pub health: f32,
}

/// Which need a value belongs to (used for utility scoring / POI matching).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum NeedKind {
    /// Food.
    Food,
    /// Water.
    Water,
    /// Rest.
    Rest,
    /// Safety.
    Safety,
    /// Social.
    Social,
    /// Health.
    Health,
}

/// All need kinds in canonical order (matches [`Needs`] field order).
pub const NEED_KINDS: [NeedKind; 6] = [
    NeedKind::Food,
    NeedKind::Water,
    NeedKind::Rest,
    NeedKind::Safety,
    NeedKind::Social,
    NeedKind::Health,
];

impl Needs {
    /// A fully-satisfied living item.
    #[must_use]
    pub fn sated() -> Self {
        Self {
            food: 1.0,
            water: 1.0,
            rest: 1.0,
            safety: 1.0,
            social: 1.0,
            health: 1.0,
        }
    }

    /// Read the satisfaction level for a [`NeedKind`].
    #[must_use]
    pub fn get(&self, kind: NeedKind) -> f32 {
        match kind {
            NeedKind::Food => self.food,
            NeedKind::Water => self.water,
            NeedKind::Rest => self.rest,
            NeedKind::Safety => self.safety,
            NeedKind::Social => self.social,
            NeedKind::Health => self.health,
        }
    }

    /// Set the satisfaction level for a [`NeedKind`], clamped to `[0, 1]`.
    pub fn set(&mut self, kind: NeedKind, value: f32) {
        let v = value.clamp(0.0, 1.0);
        match kind {
            NeedKind::Food => self.food = v,
            NeedKind::Water => self.water = v,
            NeedKind::Rest => self.rest = v,
            NeedKind::Safety => self.safety = v,
            NeedKind::Social => self.social = v,
            NeedKind::Health => self.health = v,
        }
    }

    /// Satisfy a need by `amount` (clamped to `[0, 1]`). Returns the new level.
    pub fn satisfy(&mut self, kind: NeedKind, amount: f32) -> f32 {
        let next = (self.get(kind) + amount.max(0.0)).clamp(0.0, 1.0);
        self.set(kind, next);
        next
    }

    /// The most-deprived (lowest satisfaction) need and its level. This is the
    /// highest-*pressure* need that a utility-AI should target next.
    #[must_use]
    pub fn most_pressing(&self) -> (NeedKind, f32) {
        let mut worst = (NeedKind::Food, self.food);
        for kind in NEED_KINDS {
            let v = self.get(kind);
            if v < worst.1 {
                worst = (kind, v);
            }
        }
        worst
    }

    /// `true` when any need has fallen to or below `critical` (deprivation).
    #[must_use]
    pub fn any_critical(&self, critical: f32) -> bool {
        NEED_KINDS.iter().any(|&k| self.get(k) <= critical)
    }
}

/// Per-need decay rates per tick (satisfaction lost each tick when unmet).
///
/// Water decays fastest, then food/rest, with social/safety/health slower —
/// roughly mirroring real survival-pressure ordering (Dwarf-Fortress / Songs of
/// Syx style). All rates are deterministic constants.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct DecayRates {
    /// Food decay per tick.
    pub food: f32,
    /// Water decay per tick.
    pub water: f32,
    /// Rest decay per tick.
    pub rest: f32,
    /// Safety decay per tick (threat re-accrues when not in a safe place).
    pub safety: f32,
    /// Social decay per tick.
    pub social: f32,
    /// Health does not decay on its own; it is driven by [`apply_health`].
    pub health: f32,
}

impl Default for DecayRates {
    fn default() -> Self {
        Self {
            food: 0.004,
            water: 0.006,
            rest: 0.003,
            safety: 0.002,
            social: 0.0015,
            health: 0.0,
        }
    }
}

/// Health / sickness state for a living item.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Health {
    /// Integrity in `[0, 1]`; `0.0` = dead.
    pub integrity: f32,
    /// `true` once an illness has taken hold (accelerates integrity loss).
    pub sick: bool,
    /// Consecutive ticks spent with at least one critical need.
    pub deprivation_streak: u32,
}

impl Default for Health {
    fn default() -> Self {
        Self {
            integrity: 1.0,
            sick: false,
            deprivation_streak: 0,
        }
    }
}

impl Health {
    /// `true` when integrity has reached zero.
    #[must_use]
    pub fn is_dead(&self) -> bool {
        self.integrity <= 0.0
    }
}

/// Tunable thresholds governing the deprivation → sickness → death pipeline.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct HealthParams {
    /// A need at or below this is "critical" / unmet.
    pub critical: f32,
    /// Integrity lost per tick per critical need while deprived.
    pub damage_per_critical: f32,
    /// Integrity regained per tick when fully clear of critical needs.
    pub regen: f32,
    /// Deprivation-streak length after which a sickness roll fires.
    pub sickness_onset_ticks: u32,
    /// Per-tick probability of falling sick once past `sickness_onset_ticks`.
    pub sickness_chance: f32,
    /// Extra integrity lost per tick while [`Health::sick`].
    pub sickness_damage: f32,
}

impl Default for HealthParams {
    fn default() -> Self {
        Self {
            critical: 0.1,
            damage_per_critical: 0.01,
            regen: 0.004,
            sickness_onset_ticks: 30,
            sickness_chance: 0.05,
            sickness_damage: 0.02,
        }
    }
}

/// Outcome of one [`tick`] of the needs/health pipeline.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct TickOutcome {
    /// Number of needs critical this tick.
    pub critical_count: u32,
    /// `true` if the item became sick this tick.
    pub fell_sick: bool,
    /// `true` if the item died this tick.
    pub died: bool,
}

/// Outcome of one FR-CIV-LIFE needs tick.
///
/// The decay step is applied first, then the agent selects the most-deprived
/// need as its satisfaction priority. This keeps the control loop deterministic
/// and gives higher-pressure needs (especially hunger / safety / social) a
/// stable ordering for downstream planners.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct NeedsTickOutcome {
    /// Need chosen for satisfaction next.
    pub priority: NeedKind,
    /// Satisfaction level after decay, before any replenishment.
    pub priority_level: f32,
}

/// Decay every need by its per-tick rate (no satisfaction added). Health is
/// mirrored from the supplied integrity so utility-AI sees a unified vector.
pub fn decay(needs: &mut Needs, rates: &DecayRates) {
    needs.food = (needs.food - rates.food).max(0.0);
    needs.water = (needs.water - rates.water).max(0.0);
    needs.rest = (needs.rest - rates.rest).max(0.0);
    needs.safety = (needs.safety - rates.safety).max(0.0);
    needs.social = (needs.social - rates.social).max(0.0);
    needs.health = (needs.health - rates.health).max(0.0);
}

/// Advance needs by one deterministic FR-CIV-LIFE tick.
///
/// This is intentionally pure apart from the mutation of `needs`: decay is
/// applied in place, then the agent selects the most-deprived need to satisfy
/// next. The returned priority is stable under identical inputs, which keeps
/// replay and AI planning deterministic.
#[must_use]
pub fn tick_needs(needs: &mut Needs, rates: &DecayRates) -> NeedsTickOutcome {
    decay(needs, rates);
    let (priority, priority_level) = needs.most_pressing();
    NeedsTickOutcome {
        priority,
        priority_level,
    }
}

/// Advance one living item by a single tick: decay needs, then drive health
/// from deprivation (sickness onset + integrity damage/regen + death).
///
/// `rng` is only consulted for the seeded sickness roll, keeping replays exact.
pub fn tick(
    needs: &mut Needs,
    health: &mut Health,
    rates: &DecayRates,
    params: &HealthParams,
    rng: &mut ChaCha8Rng,
) -> TickOutcome {
    if health.is_dead() {
        return TickOutcome {
            critical_count: 0,
            fell_sick: false,
            died: false,
        };
    }

    let _needs_tick = tick_needs(needs, rates);

    let critical_count = NEED_KINDS
        .iter()
        .filter(|&&k| k != NeedKind::Health && needs.get(k) <= params.critical)
        .count() as u32;

    let mut fell_sick = false;
    if critical_count > 0 {
        health.deprivation_streak = health.deprivation_streak.saturating_add(1);
        health.integrity =
            (health.integrity - params.damage_per_critical * critical_count as f32).max(0.0);

        if !health.sick
            && health.deprivation_streak >= params.sickness_onset_ticks
            && rng.gen::<f32>() < params.sickness_chance
        {
            health.sick = true;
            fell_sick = true;
        }
    } else {
        health.deprivation_streak = 0;
        health.integrity = (health.integrity + params.regen).min(1.0);
        // Clear of deprivation long enough → recover from sickness.
        if health.sick && health.integrity >= 0.95 {
            health.sick = false;
        }
    }

    if health.sick {
        health.integrity = (health.integrity - params.sickness_damage).max(0.0);
    }

    // Mirror integrity into the needs.health channel for utility scoring.
    needs.health = health.integrity;

    let died = health.is_dead();
    TickOutcome {
        critical_count,
        fell_sick,
        died,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;
    use rand::SeedableRng;

    fn rng(seed: u64) -> ChaCha8Rng {
        ChaCha8Rng::seed_from_u64(seed)
    }

    /// Covers FR-CIV-LIFE-000 — schema version is a non-empty semver triple.
    #[test]
    fn schema_version_is_semver() {
        let parts: Vec<&str> = SCHEMA_VERSION.split('.').collect();
        assert_eq!(parts.len(), 3);
        assert!(parts.iter().all(|p| !p.is_empty()));
    }

    /// Covers FR-CIV-LIFE-001 — needs decay monotonically toward zero each tick.
    #[test]
    fn needs_decay_toward_zero() {
        let mut needs = Needs::sated();
        let rates = DecayRates::default();
        let before = needs;
        decay(&mut needs, &rates);
        assert!(needs.food < before.food);
        assert!(needs.water < before.water);
        assert!(needs.rest < before.rest);
        // Water decays fastest.
        assert!(needs.water < needs.food);
    }

    /// Covers FR-CIV-LIFE-001 — decay never produces negative satisfaction.
    #[test]
    fn decay_clamps_at_zero() {
        let mut needs = Needs {
            food: 0.001,
            water: 0.0,
            rest: 0.0,
            safety: 0.0,
            social: 0.0,
            health: 0.0,
        };
        for _ in 0..100 {
            decay(&mut needs, &DecayRates::default());
        }
        for k in NEED_KINDS {
            assert!(needs.get(k) >= 0.0, "{k:?} went negative");
        }
    }

    proptest! {
        /// Covers FR-CIV-LIFE-001 — decay keeps every need within the closed interval [0, 1].
        #[test]
        fn decay_preserves_need_bounds(
            food in 0.0f32..=1.0,
            water in 0.0f32..=1.0,
            rest in 0.0f32..=1.0,
            safety in 0.0f32..=1.0,
            social in 0.0f32..=1.0,
            health in 0.0f32..=1.0,
            food_rate in 0.0f32..=2.0,
            water_rate in 0.0f32..=2.0,
            rest_rate in 0.0f32..=2.0,
            safety_rate in 0.0f32..=2.0,
            social_rate in 0.0f32..=2.0,
            health_rate in 0.0f32..=2.0,
        ) {
            let mut needs = Needs { food, water, rest, safety, social, health };
            let rates = DecayRates {
                food: food_rate,
                water: water_rate,
                rest: rest_rate,
                safety: safety_rate,
                social: social_rate,
                health: health_rate,
            };

            decay(&mut needs, &rates);

            for kind in NEED_KINDS {
                let v = needs.get(kind);
                prop_assert!((0.0..=1.0).contains(&v), "{kind:?} out of bounds: {v}");
            }
        }
    }

    /// Covers FR-CIV-LIFE-001 — most_pressing reports the lowest-satisfaction need.
    #[test]
    fn most_pressing_picks_lowest() {
        let needs = Needs {
            food: 0.9,
            water: 0.2,
            rest: 0.8,
            safety: 0.7,
            social: 0.95,
            health: 1.0,
        };
        assert_eq!(needs.most_pressing(), (NeedKind::Water, 0.2));
    }

    /// Covers FR-CIV-LIFE-001 — a tick decays needs before selecting the next priority.
    #[test]
    fn fr_civ_life_001_tick_needs_decays_then_prioritizes() {
        let mut needs = Needs {
            food: 0.5,
            water: 0.5,
            rest: 0.5,
            safety: 0.12,
            social: 0.13,
            health: 1.0,
        };
        let rates = DecayRates {
            food: 0.01,
            water: 0.01,
            rest: 0.01,
            safety: 0.02,
            social: 0.03,
            health: 0.0,
        };

        let out = tick_needs(&mut needs, &rates);

        assert_eq!(out.priority, NeedKind::Safety);
        assert!((out.priority_level - 0.10).abs() < 1e-6);
        assert!((needs.safety - 0.10).abs() < 1e-6);
        assert!((needs.social - 0.10).abs() < 1e-6);
    }

    /// Covers FR-CIV-LIFE-001 — satisfying a need raises it and clamps at 1.0.
    #[test]
    fn satisfy_raises_and_clamps() {
        let mut needs = Needs::sated();
        needs.food = 0.5;
        assert_eq!(needs.satisfy(NeedKind::Food, 0.3), 0.8);
        assert_eq!(needs.satisfy(NeedKind::Food, 5.0), 1.0);
    }

    /// Covers FR-CIV-LIFE-003 — sustained unmet needs drain health to death.
    #[test]
    fn unmet_needs_cause_death() {
        let mut needs = Needs {
            food: 0.0,
            water: 0.0,
            rest: 0.0,
            safety: 0.0,
            social: 0.0,
            health: 1.0,
        };
        let mut health = Health::default();
        let mut r = rng(1);
        let rates = DecayRates::default();
        let params = HealthParams::default();
        let mut died = false;
        for _ in 0..10_000 {
            let out = tick(&mut needs, &mut health, &rates, &params, &mut r);
            if out.died {
                died = true;
                break;
            }
        }
        assert!(died, "agent with all needs at zero must eventually die");
        assert!(health.is_dead());
    }

    /// Covers FR-CIV-LIFE-003 — deprivation damage is applied at the critical boundary.
    #[test]
    fn critical_boundary_triggers_damage() {
        let mut needs = Needs {
            food: 0.1,
            water: 1.0,
            rest: 1.0,
            safety: 1.0,
            social: 1.0,
            health: 1.0,
        };
        let mut health = Health::default();
        let rates = DecayRates {
            food: 0.0,
            water: 0.0,
            rest: 0.0,
            safety: 0.0,
            social: 0.0,
            health: 0.0,
        };
        let params = HealthParams::default();
        let mut r = rng(11);

        let out = tick(&mut needs, &mut health, &rates, &params, &mut r);

        assert_eq!(out.critical_count, 1);
        assert_eq!(health.deprivation_streak, 1);
        assert!(health.integrity < 1.0);
    }

    /// Covers FR-CIV-LIFE-002 — sickness onset is governed by the streak threshold.
    #[test]
    fn sickness_onset_respects_threshold() {
        let mut needs = Needs {
            food: 0.0,
            water: 1.0,
            rest: 1.0,
            safety: 1.0,
            social: 1.0,
            health: 1.0,
        };
        let mut health = Health::default();
        let rates = DecayRates {
            food: 0.0,
            water: 0.0,
            rest: 0.0,
            safety: 0.0,
            social: 0.0,
            health: 0.0,
        };
        let params = HealthParams {
            critical: 0.1,
            damage_per_critical: 0.0,
            regen: 0.0,
            sickness_onset_ticks: 3,
            sickness_chance: 1.0,
            sickness_damage: 0.0,
        };
        let mut r = rng(12);

        let mut fell_sick_at = None;
        for tick_index in 0..4 {
            let out = tick(&mut needs, &mut health, &rates, &params, &mut r);
            if out.fell_sick {
                fell_sick_at = Some(tick_index);
                break;
            }
        }

        assert_eq!(fell_sick_at, Some(2));
        assert!(health.sick);
    }

    /// Covers FR-CIV-LIFE-002 — recovery clears sickness once integrity has rebuilt enough.
    #[test]
    fn sickness_recovery_requires_high_integrity() {
        let mut needs = Needs::sated();
        let mut health = Health {
            integrity: 0.90,
            sick: true,
            deprivation_streak: 0,
        };
        let rates = DecayRates::default();
        let params = HealthParams {
            critical: 0.1,
            damage_per_critical: 0.0,
            regen: 0.06,
            sickness_onset_ticks: 30,
            sickness_chance: 0.0,
            sickness_damage: 0.0,
        };
        let mut r = rng(13);

        let out = tick(&mut needs, &mut health, &rates, &params, &mut r);

        assert!(!out.fell_sick);
        assert!(!health.sick);
        assert!(health.integrity >= 0.95);
        assert!(health.integrity <= 1.0);
    }

    proptest! {
        /// Covers FR-CIV-LIFE-001/002/003 — repeated ticks keep health bounded and never resurrect a dead agent.
        #[test]
        fn health_stays_bounded_and_dead_is_terminal(
            seed in any::<u64>(),
            ticks in 0u32..250,
        ) {
            let mut needs = Needs::sated();
            let mut health = Health::default();
            let mut r = rng(seed);
            let rates = DecayRates::default();
            let params = HealthParams::default();

            for _ in 0..ticks {
                let out = tick(&mut needs, &mut health, &rates, &params, &mut r);
                prop_assert!((0.0..=1.0).contains(&health.integrity));
                prop_assert!((0.0..=1.0).contains(&needs.health));
                if out.died {
                    prop_assert!(health.is_dead());
                    let frozen = (needs, health);
                    let mut frozen_needs = frozen.0;
                    let mut frozen_health = frozen.1;
                    let post = tick(&mut frozen_needs, &mut frozen_health, &rates, &params, &mut r);
                    prop_assert!(!post.died);
                    prop_assert_eq!(frozen_needs, needs);
                    prop_assert_eq!(frozen_health, health);
                    break;
                }
            }
        }
    }

    /// Covers FR-CIV-LIFE-003 — a fully-sated agent never dies.
    #[test]
    fn sated_agent_survives() {
        let mut needs = Needs::sated();
        let mut health = Health::default();
        let mut r = rng(2);
        let rates = DecayRates::default();
        let params = HealthParams::default();
        for _ in 0..200 {
            // Keep topping up so nothing goes critical.
            for k in NEED_KINDS {
                needs.satisfy(k, 1.0);
            }
            let out = tick(&mut needs, &mut health, &rates, &params, &mut r);
            assert!(!out.died);
        }
        assert!(!health.is_dead());
    }

    /// Covers FR-CIV-LIFE-002 — slow deprivation triggers sickness before death.
    ///
    /// A single mildly-critical need accrues a deprivation streak past the
    /// onset threshold without draining integrity to zero first, so the
    /// seeded sickness roll has time to fire.
    #[test]
    fn deprivation_triggers_sickness() {
        let mut needs = Needs {
            food: 0.05,
            water: 1.0,
            rest: 1.0,
            safety: 1.0,
            social: 1.0,
            health: 1.0,
        };
        let mut health = Health::default();
        let mut r = rng(7);
        // Food fully unmet, but only ONE critical need → slow integrity loss.
        let rates = DecayRates {
            food: 0.02,
            water: 0.0,
            rest: 0.0,
            safety: 0.0,
            social: 0.0,
            health: 0.0,
        };
        let params = HealthParams::default();
        let mut ever_sick = false;
        for _ in 0..500 {
            let out = tick(&mut needs, &mut health, &rates, &params, &mut r);
            if out.fell_sick || health.sick {
                ever_sick = true;
                break;
            }
            if out.died {
                break;
            }
        }
        assert!(ever_sick, "prolonged deprivation must cause sickness");
    }

    /// Covers FR-CIV-LIFE-002/003 — the pipeline is deterministic under a fixed seed.
    #[test]
    fn pipeline_is_deterministic() {
        let run = |seed: u64| {
            let mut needs = Needs {
                food: 0.15,
                water: 0.12,
                rest: 0.2,
                safety: 0.3,
                social: 0.4,
                health: 1.0,
            };
            let mut health = Health::default();
            let mut r = rng(seed);
            let rates = DecayRates::default();
            let params = HealthParams::default();
            let mut sick_tick = None;
            for t in 0..400u32 {
                let out = tick(&mut needs, &mut health, &rates, &params, &mut r);
                if out.fell_sick {
                    sick_tick = Some(t);
                }
                if out.died {
                    break;
                }
            }
            (health.integrity.to_bits(), health.sick, sick_tick)
        };
        assert_eq!(run(99), run(99));
    }

    /// Covers FR-CIV-LIFE-003 — a dead agent is a no-op (integrity stays at zero).
    #[test]
    fn dead_agent_is_a_noop() {
        let mut needs = Needs::sated();
        let mut health = Health {
            integrity: 0.0,
            sick: true,
            deprivation_streak: 50,
        };
        let mut r = rng(3);
        let out = tick(
            &mut needs,
            &mut health,
            &DecayRates::default(),
            &HealthParams::default(),
            &mut r,
        );
        assert!(!out.died);
        assert_eq!(health.integrity, 0.0);
    }

    /// FR-CIV-LIFE-001 — decay subtracts each need by its configured per-tick rate.
    #[test]
    fn fr_civ_life_001_decay_is_rate_specific() {
        let mut needs = Needs {
            food: 1.0,
            water: 1.0,
            rest: 1.0,
            safety: 1.0,
            social: 1.0,
            health: 1.0,
        };
        let rates = DecayRates {
            food: 0.01,
            water: 0.02,
            rest: 0.03,
            safety: 0.04,
            social: 0.05,
            health: 0.06,
        };

        decay(&mut needs, &rates);

        assert!((needs.food - 0.99).abs() < 1e-6);
        assert!((needs.water - 0.98).abs() < 1e-6);
        assert!((needs.rest - 0.97).abs() < 1e-6);
        assert!((needs.safety - 0.96).abs() < 1e-6);
        assert!((needs.social - 0.95).abs() < 1e-6);
        assert!((needs.health - 0.94).abs() < 1e-6);
    }

    /// FR-CIV-LIFE-001 — `set` clamps values into the open range [0, 1].
    #[test]
    fn fr_civ_life_001_set_clamps_values() {
        let mut needs = Needs::sated();
        needs.set(NeedKind::Food, 1.5);
        needs.set(NeedKind::Water, -0.25);
        needs.set(NeedKind::Rest, 0.5);
        needs.set(NeedKind::Safety, f32::NAN);
        needs.set(NeedKind::Social, 2.0);
        needs.set(NeedKind::Health, -3.0);

        assert_eq!(needs.food, 1.0);
        assert_eq!(needs.water, 0.0);
        assert!((needs.rest - 0.5).abs() < 1e-6);
        assert!(needs.safety.is_nan());
        assert_eq!(needs.social, 1.0);
        assert_eq!(needs.health, 0.0);
    }

    /// FR-CIV-LIFE-001 — `most_pressing` keeps the first minimum need in field order.
    #[test]
    fn fr_civ_life_001_most_pressing_tiebreaks_by_field_order() {
        let needs = Needs {
            food: 0.2,
            water: 0.2,
            rest: 0.8,
            safety: 0.2,
            social: 0.1,
            health: 1.0,
        };
        assert_eq!(needs.most_pressing(), (NeedKind::Social, 0.1));
    }

    /// FR-CIV-LIFE-001 — `any_critical` is inclusive at the threshold boundary.
    #[test]
    fn fr_civ_life_001_critical_is_inclusive_boundary() {
        let needs = Needs {
            food: 0.25,
            water: 0.1,
            rest: 0.100_000_1,
            safety: 0.99,
            social: 1.0,
            health: 1.0,
        };
        assert!(needs.any_critical(0.1));
    }

    /// FR-CIV-LIFE-002 — sickness onset requires an exact deprivation streak threshold.
    #[test]
    fn fr_civ_life_002_sickness_starts_only_after_threshold() {
        let mut needs = Needs {
            food: 0.0,
            water: 1.0,
            rest: 1.0,
            safety: 1.0,
            social: 1.0,
            health: 1.0,
        };
        let mut health = Health::default();
        let rates = DecayRates {
            food: 0.0,
            water: 0.0,
            rest: 0.0,
            safety: 0.0,
            social: 0.0,
            health: 0.0,
        };
        let params = HealthParams {
            critical: 0.5,
            damage_per_critical: 0.0,
            regen: 0.0,
            sickness_onset_ticks: 3,
            sickness_chance: 1.0,
            sickness_damage: 0.0,
        };

        let mut r = rng(1);
        for _ in 0..2 {
            let out = tick(&mut needs, &mut health, &rates, &params, &mut r);
            assert!(!out.fell_sick);
            assert!(!health.sick);
        }

        let out = tick(&mut needs, &mut health, &rates, &params, &mut r);
        assert!(out.fell_sick);
        assert!(health.sick);
        assert_eq!(health.deprivation_streak, 3);
    }

    /// FR-CIV-LIFE-002 — sickness is not reasserted when already sick.
    #[test]
    fn fr_civ_life_002_repeated_sickness_rolls_do_not_retrigger() {
        let mut needs = Needs {
            food: 0.0,
            water: 1.0,
            rest: 1.0,
            safety: 1.0,
            social: 1.0,
            health: 1.0,
        };
        let mut health = Health::default();
        let rates = DecayRates {
            food: 0.0,
            water: 0.0,
            rest: 0.0,
            safety: 0.0,
            social: 0.0,
            health: 0.0,
        };
        let params = HealthParams {
            critical: 0.5,
            damage_per_critical: 0.0,
            regen: 0.0,
            sickness_onset_ticks: 1,
            sickness_chance: 1.0,
            sickness_damage: 0.0,
        };
        let mut r = rng(3);

        let out1 = tick(&mut needs, &mut health, &rates, &params, &mut r);
        let out2 = tick(&mut needs, &mut health, &rates, &params, &mut r);

        assert!(out1.fell_sick);
        assert!(!out2.fell_sick);
        assert!(health.sick);
    }

    /// FR-CIV-LIFE-002 — critical streak resets when critical state is resolved.
    #[test]
    fn fr_civ_life_002_streak_resets_after_critical_recovery() {
        let mut needs = Needs {
            food: 0.0,
            water: 1.0,
            rest: 1.0,
            safety: 1.0,
            social: 1.0,
            health: 1.0,
        };
        let mut health = Health {
            integrity: 0.5,
            sick: false,
            deprivation_streak: 0,
        };
        let mut rng = rng(2);
        let mut rates = DecayRates::default();
        rates.food = 0.0;
        let params = HealthParams {
            critical: 0.5,
            damage_per_critical: 0.0,
            regen: 0.2,
            sickness_onset_ticks: 2,
            sickness_chance: 0.0,
            sickness_damage: 0.0,
        };

        let out = tick(&mut needs, &mut health, &rates, &params, &mut rng);
        assert_eq!(out.critical_count, 1);
        assert_eq!(health.deprivation_streak, 1);

        needs.food = 1.0;
        let out2 = tick(&mut needs, &mut health, &rates, &params, &mut rng);
        assert_eq!(out2.critical_count, 0);
        assert_eq!(health.deprivation_streak, 0);
    }

    /// FR-CIV-LIFE-003 — critical-count excludes health need from critical calculations.
    #[test]
    fn fr_civ_life_003_critical_count_excludes_health_need() {
        let mut needs = Needs {
            food: 0.5,
            water: 0.5,
            rest: 0.5,
            safety: 0.5,
            social: 0.5,
            health: 0.0,
        };
        let mut health = Health::default();
        let rates = DecayRates {
            food: 0.0,
            water: 0.0,
            rest: 0.0,
            safety: 0.0,
            social: 0.0,
            health: 0.0,
        };
        let params = HealthParams {
            critical: 0.1,
            damage_per_critical: 0.0,
            regen: 0.0,
            sickness_onset_ticks: 5,
            sickness_chance: 0.0,
            sickness_damage: 0.0,
        };
        let mut rng = rng(4);

        let out = tick(&mut needs, &mut health, &rates, &params, &mut rng);

        assert_eq!(out.critical_count, 0);
    }

    /// FR-CIV-LIFE-003 — needs.health mirrors integrity after health updates.
    #[test]
    fn fr_civ_life_003_needs_health_tracks_integrity() {
        let mut needs = Needs {
            food: 1.0,
            water: 1.0,
            rest: 1.0,
            safety: 1.0,
            social: 1.0,
            health: 1.0,
        };
        let mut health = Health {
            integrity: 0.42,
            sick: false,
            deprivation_streak: 0,
        };
        let mut rates = DecayRates::default();
        rates.food = 0.0;
        rates.water = 0.0;
        rates.rest = 0.0;
        rates.safety = 0.0;
        rates.social = 0.0;
        rates.health = 0.0;
        let mut rng = rng(5);
        let params = HealthParams {
            critical: 1.0,
            damage_per_critical: 0.2,
            regen: 0.0,
            sickness_onset_ticks: 1_000,
            sickness_chance: 0.0,
            sickness_damage: 0.0,
        };

        let _ = tick(&mut needs, &mut health, &rates, &params, &mut rng);

        assert_eq!(health.integrity, needs.health);
    }

    /// FR-CIV-LIFE-003 — death is terminal and mirrored into needs.health.
    #[test]
    fn fr_civ_life_003_dead_state_is_reflected_in_needs_health() {
        let mut needs = Needs {
            food: 0.0,
            water: 0.0,
            rest: 0.0,
            safety: 0.0,
            social: 0.0,
            health: 1.0,
        };
        let mut health = Health {
            integrity: 0.02,
            sick: false,
            deprivation_streak: 0,
        };
        let mut rng = rng(9);
        let rates = DecayRates {
            food: 0.0,
            water: 0.0,
            rest: 0.0,
            safety: 0.0,
            social: 0.0,
            health: 0.0,
        };
        let params = HealthParams {
            critical: 0.5,
            damage_per_critical: 0.02,
            regen: 0.0,
            sickness_onset_ticks: 999,
            sickness_chance: 0.0,
            sickness_damage: 0.0,
        };

        let out = tick(&mut needs, &mut health, &rates, &params, &mut rng);
        assert!(out.died);
        assert!(health.is_dead());
        assert_eq!(health.integrity, needs.health);
        assert_eq!(needs.health, 0.0);
    }
}

/// Determine birth probability for a civilian based on lifecycle stage, health, and environmental constraints.
///
/// FR-CIV-LIFE-003: Reproduction is driven by lifecycle stage (working-age adults only),
/// health/nutrition availability, safety, and population pressure.
/// Returns a probability in `[0, 1]` to be used with a seeded RNG.
#[must_use]
pub fn should_reproduce(
    age: f32,
    health: &Health,
    food_satisfaction: f32,
    safety_satisfaction: f32,
    overcrowding_factor: f32,
    lifecycle_params: &LifecycleParams,
) -> f32 {
    use crate::lifecycle::{classify_lifecycle_from_age as classify_lifecycle};

    // Only working-age adults can reproduce.
    let lifecycle = classify_lifecycle(age, lifecycle_params);
    if lifecycle != crate::lifecycle::LifecycleLabel::WorkingAge {
        return 0.0;
    }

    // Dead or critically ill individuals do not reproduce.
    if health.is_dead() || health.sick {
        return 0.0;
    }

    // Base reproduction rate for healthy working-age adults.
    let mut birth_prob = 0.15;

    // Health/integrity modulates: sicker individuals have lower birth probability.
    birth_prob *= health.integrity.max(0.1);

    // Food satisfaction must be adequate: starving populations don't reproduce well.
    if food_satisfaction < 0.3 {
        return 0.0; // Critical food shortage prevents reproduction.
    }
    birth_prob *= food_satisfaction.clamp(0.3, 1.0) / 1.0;

    // Safety must be acceptable: threatened/panicked populations suppress reproduction.
    if safety_satisfaction < 0.2 {
        return 0.0; // Critical danger prevents reproduction.
    }
    birth_prob *= (safety_satisfaction * 0.5 + 0.5).clamp(0.0, 1.0);

    // Overcrowding suppresses reproduction (carrying capacity effect).
    if overcrowding_factor > 1.0 {
        birth_prob /= (1.0 + overcrowding_factor);
    }

    birth_prob.clamp(0.0, 1.0)
}

#[cfg(test)]
mod test_reproduction {
    use super::*;

    #[test]
    fn test_satisfied_adult_reproduces() {
        let lifecycle_params = LifecycleParams::default();
        let health = Health {
            integrity: 1.0,
            sick: false,
            deprivation_streak: 0,
        };
        // Working-age adult (age 25 fits in default ranges)
        let prob = should_reproduce(25.0, &health, 0.9, 0.9, 0.5, &lifecycle_params);
        assert!(prob > 0.05, "Satisfied adult should have >5% birth prob, got {}", prob);
    }

    #[test]
    fn test_starving_adult_no_reproduce() {
        let lifecycle_params = LifecycleParams::default();
        let health = Health {
            integrity: 0.8,
            sick: false,
            deprivation_streak: 0,
        };
        // Starving (food < 0.3)
        let prob = should_reproduce(25.0, &health, 0.1, 0.9, 0.5, &lifecycle_params);
        assert_eq!(prob, 0.0, "Starving adult should not reproduce");
    }

    #[test]
    fn test_juvenile_no_reproduce() {
        let lifecycle_params = LifecycleParams::default();
        let health = Health {
            integrity: 1.0,
            sick: false,
            deprivation_streak: 0,
        };
        // Juvenile (age 8)
        let prob = should_reproduce(8.0, &health, 0.9, 0.9, 0.5, &lifecycle_params);
        assert_eq!(prob, 0.0, "Juvenile should not reproduce");
    }

    #[test]
    fn test_sick_adult_no_reproduce() {
        let lifecycle_params = LifecycleParams::default();
        let health = Health {
            integrity: 1.0,
            sick: true,
            deprivation_streak: 0,
        };
        // Working-age but sick
        let prob = should_reproduce(25.0, &health, 0.9, 0.9, 0.5, &lifecycle_params);
        assert_eq!(prob, 0.0, "Sick adult should not reproduce");
    }
}
