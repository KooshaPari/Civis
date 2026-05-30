//! Sentience thresholding for emergent lineages.
//!
//! Sentience is not assigned directly. A lineage accumulates measurable DNA
//! traits, those traits are reduced to a cognition score, and a threshold
//! crossing records the transition into a sentient form.

use serde::{Deserialize, Serialize};

use crate::Dna;

/// Trait weights used to map DNA into a cognition score.
///
/// The model is intentionally simple and data-driven: each trait is a DNA byte
/// slot with a non-negative weight, and the final score is a weighted average
/// normalized to `[0, 1]`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CognitionTraitProfile {
    /// Human-readable name for the lineage profile.
    pub name: String,
    /// DNA byte indices and their weights.
    pub trait_weights: Vec<(usize, f32)>,
}

impl CognitionTraitProfile {
    /// Construct a new trait profile.
    #[must_use]
    pub fn new(name: impl Into<String>, trait_weights: Vec<(usize, f32)>) -> Self {
        Self {
            name: name.into(),
            trait_weights,
        }
    }
}

/// Threshold for emergent sentience.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct SentienceThreshold {
    /// Minimum cognition score required to be sentient.
    pub minimum_cognition: f32,
}

impl SentienceThreshold {
    /// Construct a threshold.
    #[must_use]
    pub fn new(minimum_cognition: f32) -> Self {
        Self {
            minimum_cognition,
        }
    }
}

/// Result of evaluating a lineage against the sentience threshold.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SentienceEvent {
    /// Optional lineage identifier from the simulation.
    pub lineage_id: Option<u64>,
    /// The measured cognition score.
    pub cognition_score: f32,
    /// The threshold used for the evaluation.
    pub threshold: SentienceThreshold,
    /// True when the score crosses the threshold.
    pub crossed: bool,
}

/// Compute a normalized cognition score from DNA and a trait profile.
///
/// Each selected byte contributes `byte / 255.0 * weight`. The sum is divided
/// by the total weight so the result stays in `[0, 1]` for non-negative
/// weights.
#[must_use]
pub fn cognition_score(dna: &Dna, profile: &CognitionTraitProfile) -> f32 {
    let mut total_weight = 0.0_f32;
    let mut weighted_sum = 0.0_f32;

    for (index, weight) in &profile.trait_weights {
        if *weight <= 0.0 {
            continue;
        }
        if let Some(byte) = dna.0.get(*index) {
            total_weight += *weight;
            weighted_sum += (*byte as f32 / 255.0) * *weight;
        }
    }

    if total_weight == 0.0 {
        0.0
    } else {
        (weighted_sum / total_weight).clamp(0.0, 1.0)
    }
}

/// Evaluate whether a lineage has crossed the sentience threshold.
///
/// This returns the measured score and a threshold-crossing flag; the caller
/// can use the event to emit a canonical transition record.
#[must_use]
pub fn evaluate_sentience(
    lineage_id: Option<u64>,
    dna: &Dna,
    profile: &CognitionTraitProfile,
    threshold: SentienceThreshold,
) -> SentienceEvent {
    let cognition_score = cognition_score(dna, profile);
    SentienceEvent {
        lineage_id,
        cognition_score,
        threshold,
        crossed: cognition_score >= threshold.minimum_cognition,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cognition_score_is_normalized() {
        let dna = Dna(vec![0, 128, 255]);
        let profile = CognitionTraitProfile::new("test", vec![(0, 1.0), (1, 2.0), (2, 3.0)]);
        let score = cognition_score(&dna, &profile);
        assert!(score >= 0.0);
        assert!(score <= 1.0);
    }

    #[test]
    fn threshold_crossing_event_reflects_score() {
        let dna = Dna(vec![255, 255, 255, 0]);
        let profile = CognitionTraitProfile::new("sapient-lineage", vec![(0, 0.5), (1, 0.5), (2, 0.5)]);
        let threshold = SentienceThreshold::new(0.8);
        let event = evaluate_sentience(Some(7), &dna, &profile, threshold);

        assert_eq!(event.lineage_id, Some(7));
        assert!(event.crossed);
        assert!(event.cognition_score >= threshold.minimum_cognition);
    }

    #[test]
    fn below_threshold_remains_unsentient() {
        let dna = Dna(vec![0, 0, 0, 255]);
        let profile = CognitionTraitProfile::new("proto-lineage", vec![(0, 1.0), (1, 1.0), (2, 1.0)]);
        let threshold = SentienceThreshold::new(0.4);
        let event = evaluate_sentience(None, &dna, &profile, threshold);

        assert!(!event.crossed);
        assert!(event.cognition_score < threshold.minimum_cognition);
    }
}
