//! Cultural Assimilation — structured cultural exchange and merging system.
//!
//! Provides cultural exchange initiation, resistance computation, and
//! assimilation application. Cultural vectors use fixed-point arithmetic
//! (i32, scaled x100 where 100 = 1.0, range 0–10000).
//!
//! # Determinism
//!
//! All computation is integer-only. Given the same inputs, the same functions
//! produce identical outputs. No RNG, no floating-point, no wall-clock.

use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::PolityId;

/// Cultural vectors representing three axes of a faction's culture.
///
/// Values range from 0 to 10000 (0.0 to 100.0 in real units, scaled x100).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct CulturalVectors {
    /// Language influence level.
    pub language: i32,
    /// Religious influence level.
    pub religion: i32,
    /// Customs and traditions level.
    pub customs: i32,
}

impl CulturalVectors {
    /// Create a new cultural vector with all axes set to `value`.
    pub fn uniform(value: i32) -> Self {
        Self {
            language: value,
            religion: value,
            customs: value,
        }
    }

    /// Clamp all axes to the valid range [0, 10000].
    pub fn clamped(self) -> Self {
        Self {
            language: self.language.clamp(0, 10_000),
            religion: self.religion.clamp(0, 10_000),
            customs: self.customs.clamp(0, 10_000),
        }
    }
}

/// A cultural exchange between two factions.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CulturalExchange {
    /// Source faction (culture flowing outward).
    pub source_faction: PolityId,
    /// Target faction (culture flowing inward).
    pub target_faction: PolityId,
    /// Intensity of the exchange (0–10000, higher = stronger).
    pub intensity: i32,
    /// Cultural vectors being exchanged.
    pub vectors: CulturalVectors,
    /// Tick when the exchange started.
    pub started_at_tick: u64,
}

/// Result of a cultural assimilation attempt.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct AssimilationResult {
    /// Number of cultural practices successfully adopted.
    pub adopted_practices: u32,
    /// Resistance level encountered (0–10000).
    pub resistance_level: i32,
    /// Net cultural shift applied (signed, -10000 to 10000).
    pub net_effect: i32,
}

/// Errors during assimilation operations.
#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum AssimilationError {
    /// Cannot assimilate with self.
    #[error("cannot assimilate with self")]
    SelfAssimilation,
    /// Intensity must be in range [0, 10000].
    #[error("intensity {0} out of range [0, 10000]")]
    IntensityOutOfRange(i32),
    /// Factions are identical — no assimilation needed.
    #[error("identical cultural profiles")]
    IdenticalProfiles,
}

impl CulturalExchange {
    /// Create a new cultural exchange with the given intensity.
    pub fn new(
        source: PolityId,
        target: PolityId,
        intensity: i32,
        vectors: CulturalVectors,
        tick: u64,
    ) -> Result<Self, AssimilationError> {
        if source == target {
            return Err(AssimilationError::SelfAssimilation);
        }
        if !(0..=10_000).contains(&intensity) {
            return Err(AssimilationError::IntensityOutOfRange(intensity));
        }
        Ok(Self {
            source_faction: source,
            target_faction: target,
            intensity,
            vectors,
            started_at_tick: tick,
        })
    }

    /// Compute resistance to cultural assimilation.
    ///
    /// Resistance is higher when:
    /// - Cultural distance between source and target is large
    /// - Nationalism level is high
    /// - Religious difference is large
    ///
    /// Returns a value 0–10000 (higher = more resistance).
    ///
    /// Formula: `min(10000, distance * 10 + nationalism * 4 + religious_diff * 5) / 100`
    pub fn compute_resistance(
        source: &CulturalVectors,
        target: &CulturalVectors,
        nationalism_level: i32,
        religious_difference: i32,
    ) -> i32 {
        let dist = cultural_distance(source, target);
        let raw = dist * 10 + nationalism_level.clamp(0, 10_000) * 4
            + religious_difference.clamp(0, 10_000) * 5;
        // Scale down to 0–10000 range.
        (raw / 100).clamp(0, 10_000)
    }

    /// Apply assimilation: merge source cultural vectors into target.
    ///
    /// Returns the new target vectors and an [`AssimilationResult`].
    /// The net shift is `(intensity * (source - target)) / 10000`, meaning
    /// higher intensity causes faster convergence.
    pub fn apply_assimilation(
        &self,
        source_profile: &CulturalVectors,
        target_profile: &CulturalVectors,
    ) -> (CulturalVectors, AssimilationResult) {
        let intensity = self.intensity.clamp(0, 10_000);

        // Compute per-axis shift: intensity% of the gap.
        let shift_lang = ((source_profile.language - target_profile.language) * intensity) / 10_000;
        let shift_relig = ((source_profile.religion - target_profile.religion) * intensity) / 10_000;
        let shift_cust = ((source_profile.customs - target_profile.customs) * intensity) / 10_000;

        let new_target = CulturalVectors {
            language: target_profile.language + shift_lang,
            religion: target_profile.religion + shift_relig,
            customs: target_profile.customs + shift_cust,
        }
        .clamped();

        // Count adopted practices: each axis that moved closer to source.
        let mut adopted = 0u32;
        if (new_target.language - source_profile.language).abs()
            < (target_profile.language - source_profile.language).abs()
        {
            adopted += 1;
        }
        if (new_target.religion - source_profile.religion).abs()
            < (target_profile.religion - source_profile.religion).abs()
        {
            adopted += 1;
        }
        if (new_target.customs - source_profile.customs).abs()
            < (target_profile.customs - source_profile.customs).abs()
        {
            adopted += 1;
        }

        let net_effect = shift_lang + shift_relig + shift_cust;

        (new_target, AssimilationResult {
            adopted_practices: adopted,
            resistance_level: 0, // caller should compute separately
            net_effect,
        })
    }
}

/// Compute the cultural distance (Manhattan) between two cultural profiles.
///
/// Returns a value 0–30000 (each axis contributes 0–10000).
pub fn cultural_distance(a: &CulturalVectors, b: &CulturalVectors) -> i32 {
    (a.language - b.language).abs()
        + (a.religion - b.religion).abs()
        + (a.customs - b.customs).abs()
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    fn p(id: u32) -> PolityId {
        PolityId::new(id)
    }

    #[test]
    fn cultural_vectors_uniform_creates_equal_axes() {
        let v = CulturalVectors::uniform(5000);
        assert_eq!(v.language, 5000);
        assert_eq!(v.religion, 5000);
        assert_eq!(v.customs, 5000);
    }

    #[test]
    fn cultural_vectors_clamped_enforces_bounds() {
        let v = CulturalVectors {
            language: -500,
            religion: 15_000,
            customs: 5000,
        };
        let c = v.clamped();
        assert_eq!(c.language, 0);
        assert_eq!(c.religion, 10_000);
        assert_eq!(c.customs, 5000);
    }

    #[test]
    fn cultural_distance_zero_for_identical() {
        let a = CulturalVectors::uniform(4200);
        let b = CulturalVectors::uniform(4200);
        assert_eq!(cultural_distance(&a, &b), 0);
    }

    #[test]
    fn cultural_distance_symmetric() {
        let a = CulturalVectors {
            language: 1000,
            religion: 2000,
            customs: 3000,
        };
        let b = CulturalVectors {
            language: 4000,
            religion: 5000,
            customs: 6000,
        };
        assert_eq!(cultural_distance(&a, &b), cultural_distance(&b, &a));
    }

    #[test]
    fn cultural_distance_manhattan_sum() {
        let a = CulturalVectors {
            language: 1000,
            religion: 0,
            customs: 5000,
        };
        let b = CulturalVectors {
            language: 4000,
            religion: 3000,
            customs: 5000,
        };
        // |1000-4000| + |0-3000| + |5000-5000| = 3000+3000+0 = 6000
        assert_eq!(cultural_distance(&a, &b), 6000);
    }

    #[test]
    fn new_exchange_rejects_self_assimilation() {
        let result = CulturalExchange::new(
            p(1),
            p(1),
            5000,
            CulturalVectors::uniform(5000),
            100,
        );
        assert!(matches!(result, Err(AssimilationError::SelfAssimilation)));
    }

    #[test]
    fn new_exchange_rejects_out_of_range_intensity() {
        let result = CulturalExchange::new(
            p(1),
            p(2),
            15_000,
            CulturalVectors::uniform(5000),
            100,
        );
        assert!(matches!(
            result,
            Err(AssimilationError::IntensityOutOfRange(15_000))
        ));
    }

    #[test]
    fn new_exchange_accepts_valid_parameters() {
        let ex = CulturalExchange::new(
            p(1),
            p(2),
            3000,
            CulturalVectors::uniform(5000),
            100,
        )
        .expect("valid");
        assert_eq!(ex.source_faction, p(1));
        assert_eq!(ex.target_faction, p(2));
        assert_eq!(ex.intensity, 3000);
    }

    #[test]
    fn compute_resistance_high_distance_high_resistance() {
        let source = CulturalVectors::uniform(10_000);
        let target = CulturalVectors::uniform(0);
        let resistance =
            CulturalExchange::compute_resistance(&source, &target, 5000, 5000);
        // distance = 30000, raw = 30000*10 + 5000*4 + 5000*5 = 300000+20000+25000 = 345000
        // / 100 = 3450, clamped to 10000
        assert_eq!(resistance, 3450);
    }

    #[test]
    fn compute_resistance_low_distance_low_resistance() {
        let source = CulturalVectors::uniform(5000);
        let target = CulturalVectors::uniform(4900);
        let resistance =
            CulturalExchange::compute_resistance(&source, &target, 100, 100);
        // distance = 300, raw = 300*10 + 100*4 + 100*5 = 3000+400+500 = 3900
        // / 100 = 39
        assert_eq!(resistance, 39);
    }

    #[test]
    fn apply_assimilation_converges_profiles() {
        let source = CulturalVectors {
            language: 8000,
            religion: 8000,
            customs: 8000,
        };
        let target = CulturalVectors {
            language: 2000,
            religion: 2000,
            customs: 2000,
        };
        let ex = CulturalExchange::new(p(1), p(2), 5000, CulturalVectors::uniform(0), 1)
            .expect("valid");
        let (new_target, result) = ex.apply_assimilation(&source, &target);

        // shift per axis = (8000-2000)*5000/10000 = 3000
        assert_eq!(new_target.language, 5000);
        assert_eq!(new_target.religion, 5000);
        assert_eq!(new_target.customs, 5000);
        assert_eq!(result.adopted_practices, 3);
    }

    #[test]
    fn apply_assimilation_full_intensity_converges_completely() {
        let source = CulturalVectors::uniform(10_000);
        let target = CulturalVectors::uniform(0);
        let ex = CulturalExchange::new(p(1), p(2), 10_000, CulturalVectors::uniform(0), 1)
            .expect("valid");
        let (new_target, _) = ex.apply_assimilation(&source, &target);
        // 100% intensity → target becomes source.
        assert_eq!(new_target, source);
    }

    #[test]
    fn apply_assimilation_zero_intensity_no_change() {
        let source = CulturalVectors::uniform(10_000);
        let target = CulturalVectors::uniform(0);
        let ex = CulturalExchange::new(p(1), p(2), 0, CulturalVectors::uniform(0), 1)
            .expect("valid");
        let (new_target, _) = ex.apply_assimilation(&source, &target);
        assert_eq!(new_target, target);
    }

    #[test]
    fn assimilation_result_serialization_roundtrips() {
        let result = AssimilationResult {
            adopted_practices: 2,
            resistance_level: 3500,
            net_effect: 1500,
        };
        let json = serde_json::to_string(&result).expect("serialize");
        let decoded: AssimilationResult = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(result, decoded);
    }
}
