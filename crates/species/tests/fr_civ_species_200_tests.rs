//! Behavioural traceability tests for `FR-CIV-SPECIES-200`.
//!
//! The ID is tagged on `civ_species::express`, the single function that
//! projects a `Dna` into an observable `Phenotype`. These tests assert real
//! behaviour: byte positions map to the documented traits, short DNAs
//! zero-fill instead of panicking, and distinct loci stay independent.

use civ_species::{express, Dna};

fn dna(bytes: &[u8]) -> Dna {
    Dna(bytes.to_vec())
}

/// FR-CIV-SPECIES-200 — each documented locus drives the matching trait:
/// bytes 0..5 are morphology, 5..9 are behaviour weights scaled to `[0, 1]`.
#[test]
fn species_200_locus_layout_drives_matching_traits() {
    // 255 in every slot: full-scale morphology and behaviour.
    let full = express(&dna(&[255; 9]));
    assert_eq!(full.morphology.height_cm, 255);
    assert_eq!(full.morphology.body_color_hue, 255);
    assert_eq!(full.morphology.leg_count, 255);
    assert_eq!(full.morphology.arm_count, 255);
    assert_eq!(full.morphology.eye_count, 255);
    assert_eq!(full.behavior.aggression, 1.0);
    assert_eq!(full.behavior.curiosity, 1.0);
    assert_eq!(full.behavior.sociability, 1.0);
    assert_eq!(full.behavior.intelligence, 1.0);

    // Zero in every slot: zero-scale everything, no negative or NaN weights.
    let zero = express(&dna(&[0u8; 9]));
    assert_eq!(zero.morphology.height_cm, 0);
    assert_eq!(zero.morphology.leg_count, 0);
    assert_eq!(zero.behavior.aggression, 0.0);
    assert_eq!(zero.behavior.intelligence, 0.0);

    // A mid-range value lands mid-range: the scaling is linear, not thresholded.
    let mid = express(&dna(&[128, 0, 0, 0, 0, 128, 0, 0, 0]));
    assert_eq!(mid.morphology.height_cm, 128);
    let expected = f32::from(128u8) / 255.0;
    assert!((mid.behavior.aggression - expected).abs() < 1e-6);
    assert!((mid.behavior.curiosity - 0.0).abs() < 1e-6);
}

/// FR-CIV-SPECIES-200 — expression is total: a DNA shorter than the layout
/// zero-fills the missing positions instead of panicking.
#[test]
fn species_200_short_dna_zero_fills() {
    let short = express(&dna(&[7]));
    assert_eq!(short.morphology.height_cm, 7);
    assert_eq!(short.morphology.body_color_hue, 0);
    assert_eq!(short.morphology.leg_count, 0);
    assert_eq!(short.behavior.aggression, 0.0);

    // An empty DNA expresses to the all-zero phenotype rather than panicking.
    let empty = express(&Dna(Vec::new()));
    assert_eq!(empty.morphology.height_cm, 0);
    assert_eq!(empty.behavior.sociability, 0.0);
}

/// FR-CIV-SPECIES-200 — loci are independent: changing one byte changes
/// exactly one trait and leaves the others untouched.
#[test]
fn species_200_loci_are_independent() {
    let base = dna(&[10, 20, 30, 40, 50, 60, 70, 80, 90]);
    let before = express(&base);

    // Mutate only the leg-count locus (index 2).
    let mut mutated_bytes = base.0.clone();
    mutated_bytes[2] = 31;
    let after = express(&Dna(mutated_bytes));

    assert_eq!(after.morphology.leg_count, 31);
    assert_eq!(before.morphology.leg_count, 30);
    // Everything else is byte-identical.
    assert_eq!(after.morphology.height_cm, before.morphology.height_cm);
    assert_eq!(after.morphology.body_color_hue, before.morphology.body_color_hue);
    assert_eq!(after.morphology.arm_count, before.morphology.arm_count);
    assert_eq!(after.morphology.eye_count, before.morphology.eye_count);
    assert_eq!(after.behavior, before.behavior);
}

/// FR-CIV-SPECIES-200 — extra trailing loci are ignored, so a longer DNA
/// expresses to the same phenotype as its prefix.
#[test]
fn species_200_extra_loci_are_ignored() {
    let nine = dna(&[1, 2, 3, 4, 5, 6, 7, 8, 9]);
    let longer = dna(&[1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12]);
    assert_eq!(express(&nine), express(&longer));
}
