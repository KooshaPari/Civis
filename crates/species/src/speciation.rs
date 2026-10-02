//! Deterministic population speciation helpers.
//!
//! This module keeps speciation pure and substrate-driven: when two population
//! centroids drift past the class threshold, they are minted as distinct species
//! records with deterministic identifiers.
//!
//! Traceability (`docs/design/species-sentience.md:120-124`):
//! - `FR-CIV-SPECIES-300` — a new species is issued *iff* a genome's Hamming
//!   distance to its species reference exceeds `speciation_threshold`; below the
//!   threshold the child stays in the parent species.
//! - `FR-CIV-SPECIES-301` — `speciation_distance` is symmetric, normalised to
//!   `[0,1]`. (Implemented in `civ-genetics`, which this module calls.)
//! - `FR-CIV-SPECIES-302` — each `SpeciesRecord` links to its parent species.
//! - `FR-CIV-SPECIES-303` — speciation never compares across `DnaClass`
//!   boundaries; archetypes form disjoint species forests.
//! - `FR-CIV-SPECIES-304` — species issuance is stable and idempotent per split:
//!   the caller's IDs are minted verbatim, never reallocated.
//!
//! This module previously tagged itself with a digitless alias naming
//! "speciation" that no spec defines. That alias hid all five requirements
//! above from the traceability matrix, which reported them SPEC-ONLY while
//! code and tests existed here. The alias is deliberately not written here so
//! the scanner cannot mistake it for a requirement ID.

use civ_genetics::{should_speciate, Dna, DnaClass};
use serde::{Deserialize, Serialize};

/// A stable species record minted from a population centroid.
///
/// FR-CIV-SPECIES-302 — the record links to the species it split from, so the
/// inspector and legends engine can render an evolutionary tree.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SpeciesRecord {
    /// Stable species ID. FR-CIV-SPECIES-304 — minted verbatim from the caller
    /// so re-evaluating a split cannot duplicate or renumber species.
    pub id: u64,
    /// The DNA class the species belongs to. FR-CIV-SPECIES-303 — comparisons
    /// are confined to one class, so classes form disjoint species forests.
    pub dna_class: String,
    /// Founder centroid that triggered the split.
    pub founder_centroid: Dna,
    /// Optional parent species ID when this species split from an ancestor.
    /// FR-CIV-SPECIES-302 — founders have no parent.
    pub parent_species_id: Option<u64>,
}

/// Result of splitting one population lineage into two species.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PopulationSplit {
    /// Species record minted for the first population.
    pub left: SpeciesRecord,
    /// Species record minted for the second population.
    pub right: SpeciesRecord,
}

/// Errors produced by speciation helpers.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SpeciationError {
    /// The two populations are still within the class threshold.
    NotDiverged,
    /// The two input genomes are incompatible with the class.
    LengthMismatch,
}

/// Split two diverged populations into distinct species records.
///
/// FR-CIV-SPECIES-300 — the split fires *iff* the class threshold is exceeded,
/// and FR-CIV-SPECIES-304 — the two caller-supplied IDs are minted verbatim, so
/// the result is deterministic and re-evaluating one birth cannot mint
/// duplicate species.
///
/// The function is deterministic:
/// - when the populations are still within the threshold, it returns `Err(NotDiverged)`;
/// - when they have diverged, it returns two unique species IDs in the order
///   requested by the caller.
pub fn split_diverged_populations(
    left_population: &Dna,
    right_population: &Dna,
    class: &DnaClass,
    left_species_id: u64,
    right_species_id: u64,
    parent_species_id: Option<u64>,
) -> Result<PopulationSplit, SpeciationError> {
    if left_population.len() != class.length || right_population.len() != class.length {
        return Err(SpeciationError::LengthMismatch);
    }

    if !should_speciate(left_population, right_population, class) {
        return Err(SpeciationError::NotDiverged);
    }

    Ok(PopulationSplit {
        left: SpeciesRecord {
            id: left_species_id,
            dna_class: class.name.clone(),
            founder_centroid: left_population.clone(),
            parent_species_id,
        },
        right: SpeciesRecord {
            id: right_species_id,
            dna_class: class.name.clone(),
            founder_centroid: right_population.clone(),
            parent_species_id,
        },
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn diverged_populations_split_into_distinct_species() {
        let class = DnaClass {
            name: "test-lineage".into(),
            length: 9,
            mutation_rate: 0.01,
            speciation_threshold: 0.25,
        };

        let left = Dna(vec![0, 0, 0, 0, 0, 0, 0, 0, 0]);
        let right = Dna(vec![255, 255, 255, 0, 0, 0, 0, 0, 0]);

        let split = split_diverged_populations(&left, &right, &class, 10, 11, Some(3))
            .expect("diverged populations should speciate");

        assert_eq!(split.left.id, 10);
        assert_eq!(split.right.id, 11);
        assert_ne!(split.left.id, split.right.id);
        assert_eq!(split.left.dna_class, "test-lineage");
        assert_eq!(split.right.dna_class, "test-lineage");
        assert_eq!(split.left.parent_species_id, Some(3));
        assert_eq!(split.right.parent_species_id, Some(3));
        assert_eq!(split.left.founder_centroid, left);
        assert_eq!(split.right.founder_centroid, right);
    }

    #[test]
    fn similar_populations_do_not_split() {
        let class = DnaClass {
            name: "test-lineage".into(),
            length: 9,
            mutation_rate: 0.01,
            speciation_threshold: 0.5,
        };

        let left = Dna(vec![10, 10, 10, 10, 10, 10, 10, 10, 10]);
        let right = Dna(vec![10, 10, 10, 10, 11, 10, 10, 10, 10]);

        let split = split_diverged_populations(&left, &right, &class, 1, 2, None);
        assert_eq!(split, Err(SpeciationError::NotDiverged));
    }

    // ──────────────────────────────────────────────────────────────────────
    // FR-CIV-SPECIES-300/302/303/304 — populations diverge into new species
    // past a genetic-distance threshold. Two diverged populations are
    // recognised as distinct species records (unique IDs, distinct founder
    // centroids, shared parent, confined to one DNA class).
    // ──────────────────────────────────────────────────────────────────────

    /// Covers FR-CIV-SPECIES-300 — two populations whose DNA Hamming-distance
    /// fraction exceeds the class's `speciation_threshold` are split into
    /// distinct `SpeciesRecord`s. The resulting records carry unique IDs,
    /// matching class names, the original centroids, and the shared parent.
    ///
    /// Also covers FR-CIV-SPECIES-302 (shared parent link), FR-CIV-SPECIES-303
    /// (both records stay inside one `DnaClass`) and FR-CIV-SPECIES-304 (the
    /// caller-supplied IDs 101/202 are minted verbatim and stay distinct).
    #[test]
    fn fr_civ_species_300_diverged_populations_become_distinct_species() {
        let class = DnaClass {
            name: "fr-civ-species-300-lineage".into(),
            length: 16,
            mutation_rate: 0.01,
            speciation_threshold: 0.25,
        };

        // Population A: bytes 0..8 zeroed, 8..16 saturated → 50% byte-level
        // divergence from population B, well above the 25% threshold.
        let left_pop = Dna(vec![
            0, 0, 0, 0, 0, 0, 0, 0, 255, 255, 255, 255, 255, 255, 255, 255,
        ]);
        // Population B: complement (zeroed where A is saturated, vice versa).
        let right_pop = Dna(vec![
            255, 255, 255, 255, 255, 255, 255, 255, 0, 0, 0, 0, 0, 0, 0, 0,
        ]);

        // Pre-flight: Hamming-distance fraction is 1.0 → strictly above the
        // 25% threshold, so speciation must fire.
        assert!(
            civ_genetics::speciation_distance(&left_pop, &right_pop) > class.speciation_threshold,
            "diverged populations must exceed the speciation threshold",
        );

        let split = split_diverged_populations(&left_pop, &right_pop, &class, 101, 202, Some(7))
            .expect("diverged populations must split into distinct species");

        // Distinct species records: unique IDs, distinct founder centroids.
        assert_ne!(
            split.left.id, split.right.id,
            "diverged populations must mint distinct species IDs",
        );
        assert_eq!(split.left.id, 101);
        assert_eq!(split.right.id, 202);

        assert_ne!(
            split.left.founder_centroid, split.right.founder_centroid,
            "founder centroids must reflect the diverged populations",
        );
        assert_eq!(split.left.founder_centroid, left_pop);
        assert_eq!(split.right.founder_centroid, right_pop);

        // Both records carry the class name and share the parent species ID.
        assert_eq!(split.left.dna_class, "fr-civ-species-300-lineage");
        assert_eq!(split.right.dna_class, "fr-civ-species-300-lineage");
        assert_eq!(split.left.parent_species_id, Some(7));
        assert_eq!(split.right.parent_species_id, Some(7));
    }

    /// Covers FR-CIV-SPECIES-300 — under-threshold populations stay as a single
    /// lineage (no speciation event is minted). This complements the divergence
    /// case above and proves the threshold gate is honoured.
    #[test]
    fn fr_civ_species_300_under_threshold_stays_one_species() {
        let class = DnaClass {
            name: "fr-civ-species-300-lineage".into(),
            length: 16,
            mutation_rate: 0.01,
            speciation_threshold: 0.5,
        };

        // Differ in only 1 of 16 bytes (~6%) → well below the 50% threshold.
        let left_pop = Dna(vec![10; 16]);
        let mut right_pop = left_pop.clone();
        right_pop.0[0] = 11;

        let split = split_diverged_populations(&left_pop, &right_pop, &class, 1, 2, None);
        assert_eq!(
            split,
            Err(SpeciationError::NotDiverged),
            "populations under the threshold must not speciate",
        );
    }
}
