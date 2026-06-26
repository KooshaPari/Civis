//! Emergent citizen lifecycle classifiers and capacity curves.
//!
//! This module exposes read-only classifiers and continuous drivers. It does
//! not store lifecycle stages on entities.

use civ_genetics::{
    sentience::{cognition_score, CognitionTraitProfile},
    Dna,
};
use serde::{Deserialize, Serialize};

use crate::Health;

/// Read-only lifecycle label derived from continuous state.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum LifecycleLabel {
    /// Still developing.
    Child,
    /// Fully participating adult.
    Adult,
    /// Aging or physically degraded.
    Elder,
    /// No longer alive.
    Dead,
}

/// Tunable lifecycle thresholds and curves.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LifecycleParams {
    /// Base maturity gain per tick under no stress.
    pub base_maturity_rate: f32,
    /// Fraction of the maturity gain removed per critical need.
    pub maturity_stress_penalty: f32,
    /// Weight given to DNA longevity when deriving age thresholds.
    pub longevity_dna_weight: f32,
    /// Minimum food satisfaction for reproduction.
    pub fertility_food_threshold: f32,
    /// Minimum safety satisfaction for reproduction.
    pub fertility_safety_threshold: f32,
    /// Placeholder social support coupling rate.
    pub care_share_rate: f32,
    /// Floor on elder labor capacity.
    pub elder_labor_floor: f32,
    /// Trait profile used when measuring cognition from DNA.
    pub cognition_profile: CognitionTraitProfile,
}

impl Default for LifecycleParams {
    fn default() -> Self {
        Self {
            base_maturity_rate: 0.0008,
            maturity_stress_penalty: 0.5,
            longevity_dna_weight: 0.3,
            fertility_food_threshold: 0.5,
            fertility_safety_threshold: 0.4,
            care_share_rate: 0.1,
            elder_labor_floor: 0.2,
            cognition_profile: CognitionTraitProfile::new(
                "lifecycle-cognition",
                vec![(0, 0.5), (1, 0.5), (2, 0.5), (8, 0.25)],
            ),
        }
    }
}

fn clamp01(value: f32) -> f32 {
    value.clamp(0.0, 1.0)
}

/// Derived threshold at which a lineage transitions into elderhood.
#[must_use]
pub fn age_threshold(dna: &Dna, params: &LifecycleParams) -> u16 {
    let longevity_profile = CognitionTraitProfile::new(
        "lifecycle-longevity",
        vec![(20, 1.0), (21, 0.5), (22, 0.25)],
    );
    let longevity_score = cognition_score(dna, &longevity_profile);
    let dna_span = 50.0 + 70.0 * longevity_score;
    let blended = 90.0 * (1.0 - params.longevity_dna_weight)
        + dna_span * params.longevity_dna_weight;
    blended.round().clamp(50.0, 120.0) as u16
}

/// Continuous labor capacity derived from cognition, age, and integrity.
#[must_use]
pub fn labor_capacity(age: u16, health: &Health, dna: &Dna, params: &LifecycleParams) -> f32 {
    let cognition = cognition_score(dna, &params.cognition_profile);
    let age_peak = age_threshold(dna, params).max(20) as f32;
    let age = age as f32;
    let age_factor = if age <= age_peak {
        (age / age_peak).clamp(0.0, 1.0)
    } else {
        let decline = 1.0 / (1.0 + ((age - age_peak) / age_peak).max(0.0));
        decline.max(params.elder_labor_floor)
    };
    let integrity = clamp01(health.integrity);
    let capacity = 0.55 * cognition + 0.25 * age_factor + 0.20 * integrity;
    let elder_floor = if age > age_peak {
        params.elder_labor_floor * cognition.max(0.1)
    } else {
        0.0
    };
    capacity.max(elder_floor).clamp(0.0, 1.0)
}

/// Classify a citizen from read-only continuous state.
///
/// The labels are a measurement vocabulary, not a control flow. The order of
/// the predicates mirrors the spec's §1.2 table: dead first, then the
/// age/integrity Elder trigger (so a senior citizen with full capacity is
/// still classified as Elder, not Adult), then the maturity-integrity-capacity
/// Adult gate, with Child as the residual so that a young agent with low
/// labor capacity is still classified as Child rather than Elder.
#[must_use]
pub fn classify_lifecycle(
    age: u16,
    health: &Health,
    maturity: f32,
    labor_capacity: f32,
) -> LifecycleLabel {
    if health.is_dead() {
        LifecycleLabel::Dead
    } else if age >= 65 || health.integrity < 0.3 {
        LifecycleLabel::Elder
    } else if maturity >= 0.65 && health.integrity > 0.3 && labor_capacity > 0.4 {
        LifecycleLabel::Adult
    } else {
        LifecycleLabel::Child
    }
}

/// Parameters governing active lifecycle events (birth and migration).
///
/// Birth and migration are threshold-driven, emergent transitions that respond
/// to needs satisfaction and population stress. These are *not* scripted but
/// deterministic, driven by continuous state.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ActiveLifecycleParams {
    /// Minimum food satisfaction for a paired adult to consider reproduction.
    pub birth_food_threshold: f32,
    /// Minimum safety satisfaction for a paired adult to consider reproduction.
    pub birth_safety_threshold: f32,
    /// Minimum social satisfaction for a paired adult to consider reproduction.
    pub birth_social_threshold: f32,
    /// Base probability per tick that a viable pair produces a newborn.
    pub birth_probability: f32,
    /// Food satisfaction below which migration pressure activates.
    pub migration_food_threshold: f32,
    /// Safety satisfaction below which migration pressure activates.
    pub migration_safety_threshold: f32,
    /// Unrest level (aggregate need deprivation) above which agents migrate.
    pub migration_unrest_threshold: f32,
    /// Base probability per tick that a pressured agent migrates.
    pub migration_probability: f32,
}

impl Default for ActiveLifecycleParams {
    fn default() -> Self {
        Self {
            birth_food_threshold: 0.6,
            birth_safety_threshold: 0.5,
            birth_social_threshold: 0.4,
            birth_probability: 0.02,
            migration_food_threshold: 0.3,
            migration_safety_threshold: 0.3,
            migration_unrest_threshold: 0.5,
            migration_probability: 0.05,
        }
    }
}

/// Evaluate whether a paired, fertile adult may reproduce under given needs state.
///
/// Birth is viable when all reproductive thresholds are met: food, safety, and
/// social support are above configured levels. This is deterministic on needs
/// state only; the caller supplies a seeded RNG for the stochastic roll.
#[must_use]
pub fn can_birth(
    needs: &crate::Needs,
    params: &ActiveLifecycleParams,
) -> bool {
    needs.food >= params.birth_food_threshold
        && needs.safety >= params.birth_safety_threshold
        && needs.social >= params.birth_social_threshold
}

/// Compute aggregate unrest as a weighted measure of need deprivation.
///
/// Unrest is the inverse of satisfaction; higher unrest means more critical
/// needs. This drives migration pressure. The metric is the average
/// deprivation across non-health needs (food, water, rest, safety, social).
#[must_use]
pub fn compute_unrest(needs: &crate::Needs) -> f32 {
    let food_dep = (1.0 - needs.food).max(0.0);
    let water_dep = (1.0 - needs.water).max(0.0);
    let rest_dep = (1.0 - needs.rest).max(0.0);
    let safety_dep = (1.0 - needs.safety).max(0.0);
    let social_dep = (1.0 - needs.social).max(0.0);
    (food_dep + water_dep + rest_dep + safety_dep + social_dep) / 5.0
}

/// Evaluate whether an agent is under migration pressure.
///
/// Migration pressure is activated when either food or safety falls below
/// critical thresholds, or when aggregate unrest (deprivation) exceeds a limit.
/// This is threshold-driven and deterministic; the caller supplies a seeded RNG
/// for the stochastic migration roll.
#[must_use]
pub fn migration_pressure(
    needs: &crate::Needs,
    params: &ActiveLifecycleParams,
) -> bool {
    let unrest = compute_unrest(needs);
    needs.food < params.migration_food_threshold
        || needs.safety < params.migration_safety_threshold
        || unrest > params.migration_unrest_threshold

/// Per-tick probability that a given civilian reproduces, derived from
/// lifecycle state. Returns 0.0 for civilians that cannot reproduce
/// (children, elders, dead, or any civilian whose needs fall below the
/// configured food/safety thresholds).
///
/// Returns values in `[0.0, 1.0]`. Callers should still apply their own
/// external birth-window gating (e.g., a 200-tick birth window) on top of
/// this probability.
///
/// FR-CIV-LIFE-003 (reproduction gating).
#[must_use]
pub fn should_reproduce(
    age: u16,
    health: &Health,
    needs_food: f32,
    needs_safety: f32,
    overcrowding_factor: f32,
    params: &LifecycleParams,
) -> f32 {
    // Gate 1: lifecycle label. Only Adult-classified civilians can reproduce.
    // Elders are explicitly excluded — the spec treats the elder phase as a
    // post-reproductive window. Children are also excluded.
    if health.is_dead() {
        return 0.0;
    }
    if age < 18 || age >= 65 {
        return 0.0;
    }
    if health.integrity < 0.3 {
        return 0.0;
    }

    // Gate 2: needs thresholds from LifecycleParams. Both food and safety
    // must clear their respective thresholds.
    if needs_food < params.fertility_food_threshold {
        return 0.0;
    }
    if needs_safety < params.fertility_safety_threshold {
        return 0.0;
    }

    // Gate 3: fertility score from age curve (matches the existing
    // `fertility_score` helper in the engine, but uses the lifecycle params
    // so the curve is configurable).
    let age_factor = if age <= 30 {
        (age as f32 / 30.0).clamp(0.0, 1.0)
    } else if age <= 42 {
        1.0
    } else {
        (1.0 - ((age as f32 - 42.0) / 23.0)).clamp(0.0, 1.0)
    };

    // Gate 4: need-quality factor. Mean of food + safety, mapped to [0, 1].
    let need_factor = ((needs_food + needs_safety) * 0.5).clamp(0.0, 1.0);

    // Combine into a base probability and dampen by overcrowding.
    let base = 0.55 * age_factor + 0.45 * need_factor;
    (base * (1.0 - overcrowding_factor.clamp(0.0, 1.0))).clamp(0.0, 1.0)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn dna(seed: u8) -> Dna {
        let mut bytes = vec![0u8; 32];
        for (idx, byte) in bytes.iter_mut().enumerate() {
            *byte = seed.wrapping_add(idx as u8);
        }
        Dna(bytes)
    }

    #[test]
    fn lifecycle_label_boundaries_are_read_only() {
        let health = Health::default();
        assert_eq!(
            classify_lifecycle(2, &health, 0.1, 0.0),
            LifecycleLabel::Child
        );
        assert_eq!(
            classify_lifecycle(24, &health, 0.8, 0.8),
            LifecycleLabel::Adult
        );
        assert_eq!(
            classify_lifecycle(70, &Health { integrity: 0.4, ..health }, 0.8, 0.8),
            LifecycleLabel::Elder
        );
        assert_eq!(
            classify_lifecycle(24, &Health { integrity: 0.0, ..health }, 0.8, 0.8),
            LifecycleLabel::Dead
        );
    }

    #[test]
    fn labor_capacity_drops_with_integrity() {
        let params = LifecycleParams::default();
        let dna = dna(7);
        let healthy = Health {
            integrity: 1.0,
            ..Health::default()
        };
        let degraded = Health {
            integrity: 0.2,
            ..Health::default()
        };

        let high = labor_capacity(32, &healthy, &dna, &params);
        let low = labor_capacity(32, &degraded, &dna, &params);

        assert!(high >= low);
        assert!(low < high);
    }

    #[test]
    fn age_threshold_stays_in_expected_band() {
        let params = LifecycleParams::default();
        let low = age_threshold(&dna(0), &params);
        let high = age_threshold(&dna(200), &params);
        assert!((50..=120).contains(&low));
        assert!((50..=120).contains(&high));
    }

    // FR-CIV-LIFE active lifecycle tests

    #[test]
    fn birth_viable_when_all_thresholds_met() {
        let params = ActiveLifecycleParams::default();
        let sated = crate::Needs::sated();
        assert!(can_birth(&sated, &params));
    }

    #[test]
    fn birth_blocked_when_food_low() {
        let params = ActiveLifecycleParams::default();
        let mut needs = crate::Needs::sated();
        needs.food = params.birth_food_threshold - 0.1;
        assert!(!can_birth(&needs, &params));
    }

    #[test]
    fn birth_blocked_when_safety_low() {
        let params = ActiveLifecycleParams::default();
        let mut needs = crate::Needs::sated();
        needs.safety = params.birth_safety_threshold - 0.1;
        assert!(!can_birth(&needs, &params));
    }

    #[test]
    fn birth_blocked_when_social_low() {
        let params = ActiveLifecycleParams::default();
        let mut needs = crate::Needs::sated();
        needs.social = params.birth_social_threshold - 0.1;
        assert!(!can_birth(&needs, &params));
    }

    #[test]
    fn birth_viable_at_exact_thresholds() {
        let params = ActiveLifecycleParams::default();
        let needs = crate::Needs {
            food: params.birth_food_threshold,
            water: 1.0,
            rest: 1.0,
            safety: params.birth_safety_threshold,
            social: params.birth_social_threshold,
            health: 1.0,
        };
        assert!(can_birth(&needs, &params));
    }

    #[test]
    fn unrest_is_zero_when_all_needs_sated() {
        let sated = crate::Needs::sated();
        let unrest = compute_unrest(&sated);
        assert!(unrest < 1e-6);
    }

    #[test]
    fn unrest_is_one_when_all_needs_critical() {
        let critical = crate::Needs {
            food: 0.0,
            water: 0.0,
            rest: 0.0,
            safety: 0.0,
            social: 0.0,
            health: 1.0,
        };
        let unrest = compute_unrest(&critical);
        assert!(unrest > 0.99);
    }

    #[test]
    fn unrest_is_half_when_all_needs_half() {
        let half = crate::Needs {
            food: 0.5,
            water: 0.5,
            rest: 0.5,
            safety: 0.5,
            social: 0.5,
            health: 1.0,
        };
        let unrest = compute_unrest(&half);
        assert!((unrest - 0.5).abs() < 1e-6);
    }

    #[test]
    fn migration_pressure_off_when_sated() {
        let params = ActiveLifecycleParams::default();
        let sated = crate::Needs::sated();
        assert!(!migration_pressure(&sated, &params));
    }

    #[test]
    fn migration_pressure_on_when_food_critical() {
        let params = ActiveLifecycleParams::default();
        let mut needs = crate::Needs::sated();
        needs.food = params.migration_food_threshold - 0.1;
        assert!(migration_pressure(&needs, &params));
    }

    #[test]
    fn migration_pressure_on_when_safety_critical() {
        let params = ActiveLifecycleParams::default();
        let mut needs = crate::Needs::sated();
        needs.safety = params.migration_safety_threshold - 0.1;
        assert!(migration_pressure(&needs, &params));
    }

    #[test]
    fn migration_pressure_on_when_unrest_high() {
        let params = ActiveLifecycleParams::default();
        let high_unrest = crate::Needs {
            food: 0.1,
            water: 0.1,
            rest: 0.1,
            safety: 0.1,
            social: 0.1,
            health: 1.0,
        };
        assert!(migration_pressure(&high_unrest, &params));
    }

    #[test]
    fn migration_pressure_at_exact_thresholds() {
        let params = ActiveLifecycleParams::default();
        let at_boundary = crate::Needs {
            food: params.migration_food_threshold,
            water: 1.0,
            rest: 1.0,
            safety: params.migration_safety_threshold,
            social: 1.0,
            health: 1.0,
        };
        // Exactly at thresholds should NOT trigger (thresholds are exclusive lower bounds).
        assert!(!migration_pressure(&at_boundary, &params));
    }

    #[test]
    fn active_lifecycle_params_have_sensible_defaults() {
        let params = ActiveLifecycleParams::default();
        assert!(params.birth_food_threshold > 0.0 && params.birth_food_threshold <= 1.0);
        assert!(params.birth_safety_threshold > 0.0 && params.birth_safety_threshold <= 1.0);
        assert!(params.birth_social_threshold > 0.0 && params.birth_social_threshold <= 1.0);
        assert!(params.birth_probability > 0.0 && params.birth_probability <= 0.5);
        assert!(params.migration_food_threshold > 0.0 && params.migration_food_threshold <= 1.0);
        assert!(params.migration_safety_threshold > 0.0 && params.migration_safety_threshold <= 1.0);
        assert!(params.migration_unrest_threshold > 0.0 && params.migration_unrest_threshold <= 1.0);
        assert!(params.migration_probability > 0.0 && params.migration_probability <= 0.5);
    }

    #[test]
    fn fed_stable_population_can_reproduce() {
        // Scenario: well-fed, safe population with high social cohesion.
        let params = ActiveLifecycleParams::default();
        let well_fed = crate::Needs {
            food: 0.8,
            water: 0.8,
            rest: 0.8,
            safety: 0.8,
            social: 0.8,
            health: 1.0,
        };
        assert!(can_birth(&well_fed, &params));
        assert!(!migration_pressure(&well_fed, &params));
    }

    #[test]
    fn starving_population_migrates() {
        // Scenario: food-stressed population with moderate resource pressure.
        let params = ActiveLifecycleParams::default();
        let starving = crate::Needs {
            food: 0.2,
            water: 0.5,
            rest: 0.3,
            safety: 0.5,
            social: 0.3,
            health: 0.7,
        };
        assert!(!can_birth(&starving, &params));
        assert!(migration_pressure(&starving, &params));

    fn healthy() -> Health {
        Health {
            integrity: 1.0,
            ..Health::default()
        }
    }

    #[test]
    fn should_reproduce_blocks_children_elders_and_dead() {
        let params = LifecycleParams::default();
        let h = healthy();
        // Children
        assert_eq!(should_reproduce(10, &h, 1.0, 1.0, 0.0, &params), 0.0);
        // Elders (post-reproductive window)
        assert_eq!(should_reproduce(70, &h, 1.0, 1.0, 0.0, &params), 0.0);
        // Dead
        let dead = Health {
            integrity: 0.0,
            ..h
        };
        assert_eq!(should_reproduce(30, &dead, 1.0, 1.0, 0.0, &params), 0.0);
        // Critically degraded integrity
        let crit = Health {
            integrity: 0.2,
            ..h
        };
        assert_eq!(should_reproduce(30, &crit, 1.0, 1.0, 0.0, &params), 0.0);
    }

    #[test]
    fn should_reproduce_blocks_when_needs_below_thresholds() {
        let params = LifecycleParams::default();
        let h = healthy();
        // Food below threshold
        assert!(
            should_reproduce(30, &h, params.fertility_food_threshold - 0.1, 1.0, 0.0, &params)
                == 0.0
        );
        // Safety below threshold
        assert!(
            should_reproduce(30, &h, 1.0, params.fertility_safety_threshold - 0.1, 0.0, &params)
                == 0.0
        );
    }

    #[test]
    fn should_reproduce_peaks_in_adult_fertile_band() {
        let params = LifecycleParams::default();
        let h = healthy();
        let prime = should_reproduce(30, &h, 1.0, 1.0, 0.0, &params);
        let edge_young = should_reproduce(19, &h, 1.0, 1.0, 0.0, &params);
        let edge_old = should_reproduce(60, &h, 1.0, 1.0, 0.0, &params);
        assert!(prime > edge_young);
        assert!(prime > edge_old);
        // All must be valid probabilities.
        assert!((0.0..=1.0).contains(&prime));
        assert!((0.0..=1.0).contains(&edge_young));
        assert!((0.0..=1.0).contains(&edge_old));
    }

    #[test]
    fn should_reproduce_dampens_with_overcrowding() {
        let params = LifecycleParams::default();
        let h = healthy();
        let open = should_reproduce(30, &h, 1.0, 1.0, 0.0, &params);
        let crowded = should_reproduce(30, &h, 1.0, 1.0, 0.5, &params);
        let packed = should_reproduce(30, &h, 1.0, 1.0, 1.0, &params);
        assert!(open > crowded);
        assert!(crowded > packed);
        assert_eq!(packed, 0.0, "full overcrowding must drive probability to zero");
    }
}
