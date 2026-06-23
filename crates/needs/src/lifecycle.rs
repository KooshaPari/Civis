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
}
