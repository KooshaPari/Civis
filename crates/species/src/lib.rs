//! civ-species — deterministic DNA → phenotype expression + multi-species tagging.
//!
//! Per ADR-008, expression is pure deterministic algorithm: identical DNA
//! always produces an identical [`Phenotype`]. The mapping reads each byte
//! range of the DNA as a field in [`Morphology`] / [`BehaviorWeights`].
//!
//! See `docs/development-guide/fr-3d-additions.md` for `FR-CIV-SPECIES-*`.

#![forbid(unsafe_code)]
#![warn(missing_docs)]

use serde::{Deserialize, Serialize};

pub use civ_genetics::Dna;

/// Schema version. Bumped on breaking changes.
pub const SCHEMA_VERSION: u32 = 0;

/// Visible per-organism morphology. Drives the renderer (skin colour, height,
/// limb count, etc.). All fields are scalar so the diffusion / wardrobe layer
/// can interpolate visually as DNA drifts.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Morphology {
    /// Height in centimetres (8-bit precision; 0..255 cm).
    pub height_cm: u8,
    /// Body colour hue in `[0, 360)` degrees, quantised to 8 bits.
    pub body_color_hue: u8,
    /// Number of legs (0..=8 typical, but type allows any u8).
    pub leg_count: u8,
    /// Number of arms / forelimbs.
    pub arm_count: u8,
    /// Eye count (matters for funky multi-species genealogies).
    pub eye_count: u8,
}

/// Behaviour-driving scalar weights in `[0.0, 1.0]`. Consumed by the agent
/// utility-AI layer (`civ-agents`).
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct BehaviorWeights {
    /// Tendency to act aggressively in conflict.
    pub aggression: f32,
    /// Tendency to explore unknown terrain.
    pub curiosity: f32,
    /// Tendency to form social bonds.
    pub sociability: f32,
    /// Cognitive capacity proxy (affects research + tool use).
    pub intelligence: f32,
}

/// The full expressed phenotype for one organism.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Phenotype {
    /// Visible body plan.
    pub morphology: Morphology,
    /// Behavioural weights consumed by utility-AI / GOAP.
    pub behavior: BehaviorWeights,
}

/// Deterministic DNA → Phenotype mapping. Layout (first 9 bytes used; remaining
/// bytes are reserved for future fields and currently ignored):
///
/// ```text
/// byte 0 : Morphology.height_cm
/// byte 1 : Morphology.body_color_hue
/// byte 2 : Morphology.leg_count
/// byte 3 : Morphology.arm_count
/// byte 4 : Morphology.eye_count
/// byte 5 : BehaviorWeights.aggression    (byte/255.0)
/// byte 6 : BehaviorWeights.curiosity     (byte/255.0)
/// byte 7 : BehaviorWeights.sociability   (byte/255.0)
/// byte 8 : BehaviorWeights.intelligence  (byte/255.0)
/// ```
///
/// Shorter DNAs zero-fill the missing positions, so expression is total.
#[must_use]
pub fn express(dna: &Dna) -> Phenotype {
    let b = |i: usize| -> u8 { dna.0.get(i).copied().unwrap_or(0) };
    let f = |i: usize| -> f32 { f32::from(b(i)) / 255.0 };
    Phenotype {
        morphology: Morphology {
            height_cm: b(0),
            body_color_hue: b(1),
            leg_count: b(2),
            arm_count: b(3),
            eye_count: b(4),
        },
        behavior: BehaviorWeights {
            aggression: f(5),
            curiosity: f(6),
            sociability: f(7),
            intelligence: f(8),
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// FR-CIV-SPECIES-000 — schema version is exposed.
    #[test]
    fn schema_version_present() {
        assert_eq!(SCHEMA_VERSION, 0);
    }

    /// FR-CIV-SPECIES-001 — identical DNA produces identical phenotype.
    #[test]
    fn identical_dna_produces_identical_phenotype() {
        let dna = Dna(vec![10, 20, 4, 2, 2, 200, 50, 128, 240, 99, 77, 55]);
        let p1 = express(&dna);
        let p2 = express(&dna);
        assert_eq!(p1, p2);
    }

    /// FR-CIV-SPECIES-002 — expression is total: short DNAs zero-fill cleanly.
    #[test]
    fn expression_is_total_over_short_dnas() {
        let dna = Dna(vec![5]);
        let p = express(&dna);
        assert_eq!(p.morphology.height_cm, 5);
        assert_eq!(p.morphology.leg_count, 0);
        assert!((p.behavior.aggression).abs() < 1e-6);
    }

    /// FR-CIV-SPECIES-003 — behaviour weights stay in `[0, 1]`.
    #[test]
    fn behavior_weights_are_normalised() {
        let dna = Dna(vec![0; 9]);
        let p = express(&dna);
        for w in [
            p.behavior.aggression,
            p.behavior.curiosity,
            p.behavior.sociability,
            p.behavior.intelligence,
        ] {
            assert!((0.0..=1.0).contains(&w));
        }
        let dna = Dna(vec![255; 9]);
        let p = express(&dna);
        for w in [
            p.behavior.aggression,
            p.behavior.curiosity,
            p.behavior.sociability,
            p.behavior.intelligence,
        ] {
            assert!((0.0..=1.0).contains(&w));
        }
    }
}
