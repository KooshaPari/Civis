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
pub const SCHEMA_VERSION: &str = "0.1.0-stub";

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

/// Subspeciation trait signature: when two populations speciate, their
/// expressed phenotypes diverge in observable ways. This function checks
/// whether two phenotypes are "distinct enough" to represent separate
/// subspecies based on trait distance.
///
/// Trait distance is computed as the sum of squared differences in all
/// normalized fields (morphology scalars as u8/255, behavior weights as-is).
/// A higher threshold means tighter subspecies clustering.
///
/// Returns true if phenotypes represent distinct subspecies (distance > threshold).
#[must_use]
pub fn phenotypes_diverged(
    pheno_a: &Phenotype,
    pheno_b: &Phenotype,
    subspeciation_threshold: f32,
) -> bool {
    // Compute trait distance: sum of squared differences.
    let morph_dist = {
        let dh = (f32::from(pheno_a.morphology.height_cm)
            - f32::from(pheno_b.morphology.height_cm))
            / 255.0;
        let dc = (f32::from(pheno_a.morphology.body_color_hue)
            - f32::from(pheno_b.morphology.body_color_hue))
            / 255.0;
        let dl = (f32::from(pheno_a.morphology.leg_count)
            - f32::from(pheno_b.morphology.leg_count))
            / 255.0;
        let da = (f32::from(pheno_a.morphology.arm_count)
            - f32::from(pheno_b.morphology.arm_count))
            / 255.0;
        let de = (f32::from(pheno_a.morphology.eye_count)
            - f32::from(pheno_b.morphology.eye_count))
            / 255.0;
        dh * dh + dc * dc + dl * dl + da * da + de * de
    };

    let behav_dist = {
        let dag = pheno_a.behavior.aggression - pheno_b.behavior.aggression;
        let dc = pheno_a.behavior.curiosity - pheno_b.behavior.curiosity;
        let ds = pheno_a.behavior.sociability - pheno_b.behavior.sociability;
        let di = pheno_a.behavior.intelligence - pheno_b.behavior.intelligence;
        dag * dag + dc * dc + ds * ds + di * di
    };

    let total_dist = (morph_dist + behav_dist).sqrt();
    total_dist > subspeciation_threshold
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Covers FR-CIV-SPECIES-000 — exposes a semver-like schema version stub.
    #[test]
    fn schema_version_stub() {
        assert!(!SCHEMA_VERSION.is_empty());
        let core = SCHEMA_VERSION.split('-').next().unwrap();
        let segments: Vec<&str> = core.split('.').collect();
        assert_eq!(segments.len(), 3);
        assert!(segments.iter().all(|part| !part.is_empty()));
    }

    /// Covers FR-CIV-SPECIES-001 — identical DNA produces identical phenotype.
    #[test]
    fn identical_dna_produces_identical_phenotype() {
        let dna = Dna(vec![10, 20, 4, 2, 2, 200, 50, 128, 240, 99, 77, 55]);
        let p1 = express(&dna);
        let p2 = express(&dna);
        assert_eq!(p1, p2);
    }

    /// Covers FR-CIV-SPECIES-002 — expression is total: short DNAs zero-fill cleanly.
    #[test]
    fn expression_is_total_over_short_dnas() {
        let dna = Dna(vec![5]);
        let p = express(&dna);
        assert_eq!(p.morphology.height_cm, 5);
        assert_eq!(p.morphology.leg_count, 0);
        assert!((p.behavior.aggression).abs() < 1e-6);
    }

    /// Covers FR-CIV-SPECIES-003 — behaviour weights stay in `[0, 1]`.
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

    // -----------------------------------------------------------------------
    // Mutation boundary conditions
    // -----------------------------------------------------------------------

    /// Covers FR-CIV-SPECIES-004 — minimum trait values: all-zero DNA yields all-zero
    /// morphology fields and all-zero behaviour weights (the lower boundary).
    #[test]
    fn min_trait_values_from_zero_dna() {
        let dna = Dna(vec![0u8; 16]);
        let p = express(&dna);
        assert_eq!(p.morphology.height_cm, 0, "min height");
        assert_eq!(p.morphology.body_color_hue, 0, "min hue");
        assert_eq!(p.morphology.leg_count, 0, "min legs");
        assert_eq!(p.morphology.arm_count, 0, "min arms");
        assert_eq!(p.morphology.eye_count, 0, "min eyes");
        assert!(p.behavior.aggression.abs() < 1e-6, "min aggression");
        assert!(p.behavior.curiosity.abs() < 1e-6, "min curiosity");
        assert!(p.behavior.sociability.abs() < 1e-6, "min sociability");
        assert!(p.behavior.intelligence.abs() < 1e-6, "min intelligence");
    }

    /// Covers FR-CIV-SPECIES-005 — maximum trait values: all-255 DNA yields all-255
    /// morphology and behaviour weights at 1.0 (the upper boundary).
    #[test]
    fn max_trait_values_from_saturated_dna() {
        let dna = Dna(vec![255u8; 16]);
        let p = express(&dna);
        assert_eq!(p.morphology.height_cm, 255, "max height");
        assert_eq!(p.morphology.body_color_hue, 255, "max hue");
        assert_eq!(p.morphology.leg_count, 255, "max legs");
        assert_eq!(p.morphology.arm_count, 255, "max arms");
        assert_eq!(p.morphology.eye_count, 255, "max eyes");
        assert!((p.behavior.aggression - 1.0).abs() < 1e-4, "max aggression");
        assert!((p.behavior.curiosity - 1.0).abs() < 1e-4, "max curiosity");
        assert!(
            (p.behavior.sociability - 1.0).abs() < 1e-4,
            "max sociability"
        );
        assert!(
            (p.behavior.intelligence - 1.0).abs() < 1e-4,
            "max intelligence"
        );
    }

    /// Covers FR-CIV-SPECIES-006 — boundary byte 127 maps to a behaviour weight of
    /// approximately 0.498 (127/255), checking mid-range precision.
    #[test]
    fn midpoint_byte_produces_correct_weight() {
        // bytes 5..=8 are the behaviour weights; fill the rest with 0.
        let mut bytes = vec![0u8; 9];
        bytes[5] = 127;
        let p = express(&Dna(bytes));
        let expected = 127.0_f32 / 255.0;
        assert!(
            (p.behavior.aggression - expected).abs() < 1e-5,
            "byte 127 should map to {expected}, got {}",
            p.behavior.aggression
        );
    }

    // -----------------------------------------------------------------------
    // Selection pressure with known inputs
    // -----------------------------------------------------------------------

    /// Covers FR-CIV-SPECIES-007 — a more-aggressive DNA outscores a less-aggressive one
    /// in an environment that rewards high aggression (cosine similarity increases
    /// when the aggression byte is higher and the environment vector is all-255).
    #[test]
    fn selection_pressure_rewards_higher_aggression() {
        // Two DNAs identical except byte 5 (aggression).
        let low_aggression = Dna(vec![128, 128, 4, 2, 2, 50, 128, 128, 128]);
        let high_aggression = Dna(vec![128, 128, 4, 2, 2, 220, 128, 128, 128]);
        let environment = vec![255u8; 9];
        // Import fitness from the genetics crate which civ-species re-exports Dna from.
        let fit_low = civ_genetics::fitness(&low_aggression, &environment);
        let fit_high = civ_genetics::fitness(&high_aggression, &environment);
        assert!(
            fit_high > fit_low,
            "high-aggression DNA ({fit_high}) should be fitter than low-aggression ({fit_low}) in an aggression-favoring environment"
        );
    }

    /// Covers FR-CIV-SPECIES-008 — two DNAs that differ only in intelligence byte produce
    /// phenotypes that differ only in the intelligence weight, everything else equal.
    #[test]
    fn single_byte_change_affects_only_its_field() {
        let base = vec![10u8, 20, 4, 2, 2, 100, 100, 100, 50];
        let mut variant = base.clone();
        variant[8] = 200; // byte 8 → intelligence

        let p_base = express(&Dna(base));
        let p_variant = express(&Dna(variant));

        // Morphology must be identical.
        assert_eq!(
            p_base.morphology, p_variant.morphology,
            "morphology changed"
        );
        // All behaviour weights except intelligence must be identical.
        assert_eq!(
            p_base.behavior.aggression, p_variant.behavior.aggression,
            "aggression changed"
        );
        assert_eq!(
            p_base.behavior.curiosity, p_variant.behavior.curiosity,
            "curiosity changed"
        );
        assert_eq!(
            p_base.behavior.sociability, p_variant.behavior.sociability,
            "sociability changed"
        );
        // Intelligence must differ.
        assert_ne!(
            p_base.behavior.intelligence, p_variant.behavior.intelligence,
            "intelligence should have changed"
        );
        assert!(
            p_variant.behavior.intelligence > p_base.behavior.intelligence,
            "higher byte should produce higher weight"
        );
    }

    // -----------------------------------------------------------------------
    // Species creation and default state
    // -----------------------------------------------------------------------

    /// Covers FR-CIV-SPECIES-009 — empty DNA (length 0) is expressible: all fields
    /// collapse to their zero-filled defaults without panicking.
    #[test]
    fn empty_dna_expresses_to_all_zero_phenotype() {
        let p = express(&Dna(vec![]));
        assert_eq!(p.morphology.height_cm, 0);
        assert_eq!(p.morphology.leg_count, 0);
        assert!(p.behavior.aggression.abs() < 1e-6);
        assert!(p.behavior.intelligence.abs() < 1e-6);
    }

    /// Covers FR-CIV-SPECIES-010 — species with default DNA class produce phenotypes
    /// that respect the byte-layout contract: each byte maps to exactly one field,
    /// verified for all nine occupied positions.
    #[test]
    fn byte_layout_contract_is_exact() {
        let dna_bytes: [u8; 9] = [11, 22, 3, 4, 5, 51, 102, 153, 204];
        let p = express(&Dna(dna_bytes.to_vec()));
        assert_eq!(p.morphology.height_cm, 11);
        assert_eq!(p.morphology.body_color_hue, 22);
        assert_eq!(p.morphology.leg_count, 3);
        assert_eq!(p.morphology.arm_count, 4);
        assert_eq!(p.morphology.eye_count, 5);
        assert!((p.behavior.aggression - 51.0 / 255.0).abs() < 1e-5);
        assert!((p.behavior.curiosity - 102.0 / 255.0).abs() < 1e-5);
        assert!((p.behavior.sociability - 153.0 / 255.0).abs() < 1e-5);
        assert!((p.behavior.intelligence - 204.0 / 255.0).abs() < 1e-5);
    }

    /// Covers FR-CIV-SPECIES-011 — Phenotype is Clone + Copy: a copied phenotype equals
    /// the original and mutations to derived DNA do not affect already-expressed
    /// phenotypes (pure value semantics, no shared state).
    #[test]
    fn phenotype_value_semantics_no_aliasing() {
        let dna = Dna(vec![42u8; 16]);
        let p_original = express(&dna);
        // Express a different DNA and copy the original separately.
        let p_copy: Phenotype = p_original;
        let dna2 = Dna(vec![1u8; 16]);
        let p_other = express(&dna2);
        // Copies are equal to their source.
        assert_eq!(p_original, p_copy);
        // Unrelated phenotypes differ.
        assert_ne!(p_original, p_other);
    }

    // ──────────────────────────────────────────────────────────────────────
    // Subspeciation: expressed trait divergence (FR-CIV-SPECIES emergent)
    // ──────────────────────────────────────────────────────────────────────

    /// Covers FR-CIV-SPECIES (subspeciation): two populations with distinct
    /// expressed phenotypes are recognized as separate subspecies when their
    /// trait distance exceeds the subspeciation threshold.
    #[test]
    fn subspeciation_recognizes_diverged_phenotypes() {
        // Two very different phenotypes: one short/blue/peaceful, one tall/red/aggressive.
        let pheno_a = Phenotype {
            morphology: Morphology {
                height_cm: 50,
                body_color_hue: 100,  // Blue-ish
                leg_count: 4,
                arm_count: 2,
                eye_count: 2,
            },
            behavior: BehaviorWeights {
                aggression: 0.1,
                curiosity: 0.3,
                sociability: 0.8,
                intelligence: 0.6,
            },
        };

        let pheno_b = Phenotype {
            morphology: Morphology {
                height_cm: 200,       // Much taller
                body_color_hue: 10,   // Red-ish
                leg_count: 4,
                arm_count: 2,
                eye_count: 2,
            },
            behavior: BehaviorWeights {
                aggression: 0.9,      // Much more aggressive
                curiosity: 0.2,
                sociability: 0.1,
                intelligence: 0.3,
            },
        };

        // With a low subspeciation threshold, they should be recognized as diverged.
        let threshold = 0.5;
        assert!(
            phenotypes_diverged(&pheno_a, &pheno_b, threshold),
            "significantly different phenotypes should trigger subspeciation"
        );
    }

    /// Covers FR-CIV-SPECIES (subspeciation): phenotypes within the subspeciation
    /// threshold are NOT distinct subspecies (single population).
    #[test]
    fn subspeciation_recognizes_similar_phenotypes_as_same_species() {
        // Two nearly identical phenotypes (small random variance).
        let pheno_a = Phenotype {
            morphology: Morphology {
                height_cm: 100,
                body_color_hue: 128,
                leg_count: 4,
                arm_count: 2,
                eye_count: 2,
            },
            behavior: BehaviorWeights {
                aggression: 0.5,
                curiosity: 0.5,
                sociability: 0.5,
                intelligence: 0.5,
            },
        };

        let pheno_b = Phenotype {
            morphology: Morphology {
                height_cm: 101,       // 1 cm different
                body_color_hue: 129,  // 1 hue degree different
                leg_count: 4,
                arm_count: 2,
                eye_count: 2,
            },
            behavior: BehaviorWeights {
                aggression: 0.501,    // 0.001 different
                curiosity: 0.502,
                sociability: 0.501,
                intelligence: 0.50,
            },
        };

        // With a high subspeciation threshold, they should NOT trigger subspeciation.
        let threshold = 1.0; // Very high threshold (no speciation for small differences).
        assert!(
            !phenotypes_diverged(&pheno_a, &pheno_b, threshold),
            "similar phenotypes should not trigger subspeciation with high threshold"
        );
    }

    /// Covers FR-CIV-SPECIES (subspeciation): threshold is symmetric — distance
    /// from A to B equals distance from B to A.
    #[test]
    fn subspeciation_distance_is_symmetric() {
        let pheno_a = Phenotype {
            morphology: Morphology {
                height_cm: 50,
                body_color_hue: 100,
                leg_count: 4,
                arm_count: 2,
                eye_count: 2,
            },
            behavior: BehaviorWeights {
                aggression: 0.2,
                curiosity: 0.4,
                sociability: 0.6,
                intelligence: 0.5,
            },
        };

        let pheno_b = Phenotype {
            morphology: Morphology {
                height_cm: 150,
                body_color_hue: 200,
                leg_count: 6,
                arm_count: 3,
                eye_count: 3,
            },
            behavior: BehaviorWeights {
                aggression: 0.8,
                curiosity: 0.2,
                sociability: 0.1,
                intelligence: 0.9,
            },
        };

        let threshold = 0.3;
        let a_to_b = phenotypes_diverged(&pheno_a, &pheno_b, threshold);
        let b_to_a = phenotypes_diverged(&pheno_b, &pheno_a, threshold);

        assert_eq!(
            a_to_b, b_to_a,
            "subspeciation check must be symmetric"
        );
    }

    /// Covers FR-CIV-SPECIES (subspeciation + DNA): when two DNAs express
    /// into diverged phenotypes, subspeciation should be recognized.
    /// This test ties together genetics (DNA divergence) and phenotypic
    /// expression (trait differences).
    #[test]
    fn subspeciation_emerges_from_dna_divergence() {
        // Start with an ancestral DNA sequence.
        let ancestral_dna = Dna(vec![
            100, 100, 4, 2, 2, 100, 100, 100, 100, // morphology + baseline behavior
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, // padding
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        ]);

        // Simulate two isolated populations diverging.
        let mut pop_a_dna = ancestral_dna.clone();
        let mut pop_b_dna = ancestral_dna.clone();

        // Flip significant bytes in each population.
        // Population A: smaller, bluer, more peaceful.
        pop_a_dna.0[0] = 80;  // height down
        pop_a_dna.0[1] = 120; // hue shift toward blue
        pop_a_dna.0[5] = 50;  // lower aggression

        // Population B: taller, redder, more aggressive.
        pop_b_dna.0[0] = 180; // height up
        pop_b_dna.0[1] = 50;  // hue shift toward red
        pop_b_dna.0[5] = 200; // higher aggression

        let pheno_a = express(&pop_a_dna);
        let pheno_b = express(&pop_b_dna);

        // They should express distinct phenotypes.
        assert_ne!(pheno_a, pheno_b);
        assert_ne!(pheno_a.morphology.height_cm, pheno_b.morphology.height_cm);
        assert_ne!(pheno_a.behavior.aggression, pheno_b.behavior.aggression);

        // With a reasonable subspeciation threshold, they should be
        // recognized as distinct subspecies.
        let threshold = 0.5;
        assert!(
            phenotypes_diverged(&pheno_a, &pheno_b, threshold),
            "diverged DNA should express into diverged phenotypes that trigger subspeciation"
        );
    }

    /// Covers FR-CIV-SPECIES (subspeciation): mixing via recombination produces
    /// intermediate phenotypes that may or may not speciate depending on
    /// the threshold. This tests that the phenotypic expression of mixed
    /// offspring sits between the parents.
    #[test]
    fn subspeciation_mixing_produces_intermediate_phenotypes() {
        // Parent phenotypes (as if from diverged populations).
        let parent_a_dna = Dna(vec![
            50, 50, 4, 2, 2, 50, 50, 50, 50,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        ]);
        let parent_b_dna = Dna(vec![
            200, 200, 6, 3, 3, 200, 200, 200, 200,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        ]);

        let pheno_a = express(&parent_a_dna);
        let pheno_b = express(&parent_b_dna);

        // Create offspring by averaging the genomes (simplified recombination).
        // In reality, recombination would sample from parents; here we just
        // interpolate for clarity of the test.
        let mut mixed_dna = Vec::new();
        for i in 0..parent_a_dna.0.len() {
            let a = parent_a_dna.0[i] as u16;
            let b = parent_b_dna.0[i] as u16;
            mixed_dna.push(((a + b) / 2) as u8);
        }
        let mixed_pheno = express(&Dna(mixed_dna));

        // The mixed phenotype should be intermediate: height between parents.
        assert!(
            mixed_pheno.morphology.height_cm > pheno_a.morphology.height_cm
                && mixed_pheno.morphology.height_cm < pheno_b.morphology.height_cm,
            "mixed offspring height should be intermediate (got {}, between {} and {})",
            mixed_pheno.morphology.height_cm,
            pheno_a.morphology.height_cm,
            pheno_b.morphology.height_cm
        );

        // Aggression should also be intermediate.
        assert!(
            mixed_pheno.behavior.aggression > pheno_a.behavior.aggression
                && mixed_pheno.behavior.aggression < pheno_b.behavior.aggression,
            "mixed offspring aggression should be intermediate"
        );

        // The mixed phenotype may or may not trigger subspeciation,
        // depending on its distance to the parents. The point is that
        // it sits in the middle, not at the extreme.
        let threshold = 0.5;
        let diverged_from_a = phenotypes_diverged(&mixed_pheno, &pheno_a, threshold);
        let diverged_from_b = phenotypes_diverged(&mixed_pheno, &pheno_b, threshold);

        // At least one of these should be false (the mixed phenotype is
        // closer to at least one parent than a highly diverged phenotype would be).
        assert!(
            !diverged_from_a || !diverged_from_b,
            "mixed phenotype should be closer to at least one parent"
        );
    }
}
