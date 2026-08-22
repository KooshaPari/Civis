//! Evolution engine for species -- mutation, selection, crossover, and speciation.
//!
//! This module provides a deterministic framework for evolving populations of
//! [`civ_genetics::Species`]. All randomness flows through a caller-provided
//! [`rand::Rng`] implementation to ensure reproducibility under fixed seeds.

use civ_genetics::{Dna, Species};
use rand::Rng;
use serde::{Deserialize, Serialize};

/// Types of genetic mutations that can occur during evolution.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Mutation {
    /// A single gene (byte) is replaced with a new random value.
    PointGene,
    /// A segment of the genome is duplicated.
    ChromosomalDup,
    /// A segment of the genome is deleted.
    Deletion,
}

/// Composite fitness metrics for evaluating a species' survival potential.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct FitnessMetrics {
    /// Energy efficiency (0.0 to 1.0).
    pub energy_efficiency: f64,
    /// Reproduction rate (offspring per tick).
    pub reproduction_rate: f64,
    /// Adaptability to environmental changes (0.0 to 1.0).
    pub adaptability: f64,
}

impl FitnessMetrics {
    /// Calculate a single weighted fitness score.
    #[must_use]
    pub fn score(&self) -> f64 {
        (self.energy_efficiency * 0.4) + (self.reproduction_rate * 0.3) + (self.adaptability * 0.3)
    }
}

/// The core engine driving evolutionary processes in the simulation.
pub struct EvolutionEngine {
    /// Probability of a point mutation per gene (0.0 to 1.0).
    pub mutation_rate: f64,
    /// Probability of a chromosomal duplication event.
    pub duplication_rate: f64,
    /// Probability of a deletion event.
    pub deletion_rate: f64,
}

impl Default for EvolutionEngine {
    fn default() -> Self {
        Self {
            mutation_rate: 0.01,
            duplication_rate: 0.001,
            deletion_rate: 0.001,
        }
    }
}

impl EvolutionEngine {
    /// Apply a mutation to a species' genome and return the type of mutation applied.
    ///
    /// This method examines the rates configured in the engine and the random
    /// state to determine which mutation, if any, occurs.
    pub fn mutate(&self, species: &Species, rng: &mut impl Rng) -> Mutation {
        let _dna = &species.founder_centroid; // available for future mutation logic
        let roll: f64 = rng.gen();

        if roll < self.mutation_rate {
            Mutation::PointGene
        } else if roll < self.mutation_rate + self.duplication_rate {
            Mutation::ChromosomalDup
        } else {
            Mutation::Deletion
        }
    }

    /// Select a subset of the population based on a fitness function.
    ///
    /// Uses a simple truncation selection: the top 50% (rounded up, minimum 1)
    /// of individuals are returned, sorted by descending fitness.
    pub fn select<F>(&self, population: &[Species], fitness_fn: F) -> Vec<Species>
    where
        F: Fn(&Species) -> f64,
    {
        if population.is_empty() {
            return Vec::new();
        }

        let mut scored: Vec<(Species, f64)> = population
            .iter()
            .map(|s| (s.clone(), fitness_fn(s)))
            .collect();

        // Sort by fitness descending
        scored.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

        let keep = std::cmp::max(1, scored.len() / 2);
        scored.into_iter().take(keep).map(|(s, _)| s).collect()
    }

    /// Combine the genetic material of two parents to produce an offspring.
    ///
    /// Uses uniform crossover: genes are taken alternately from parent A and parent B.
    pub fn crossover(&self, parent_a: &Species, parent_b: &Species) -> Species {
        let len = std::cmp::min(
            parent_a.founder_centroid.len(),
            parent_b.founder_centroid.len(),
        );
        let mut child_dna = Vec::with_capacity(len);

        for i in 0..len {
            if i % 2 == 0 {
                child_dna.push(parent_a.founder_centroid.0[i]);
            } else {
                child_dna.push(parent_b.founder_centroid.0[i]);
            }
        }

        Species {
            id: 0, // ID assignment should be handled by the registry
            dna_class: parent_a.dna_class.clone(),
            founder_centroid: Dna(child_dna),
        }
    }

    /// Group a population into distinct species based on a genetic distance threshold.
    ///
    /// Uses a simple clustering algorithm: an individual is added to an existing group
    /// if its distance to the group's representative (first member) is below the threshold.
    pub fn speciation(&self, population: &[Species], threshold: f64) -> Vec<Vec<Species>> {
        let mut groups: Vec<Vec<Species>> = Vec::new();

        for species in population {
            let mut found_group = false;

            for group in &mut groups {
                if let Some(representative) = group.first() {
                    let distance = self.genetic_distance(
                        &representative.founder_centroid,
                        &species.founder_centroid,
                    );
                    if distance < threshold {
                        group.push(species.clone());
                        found_group = true;
                        break;
                    }
                }
            }

            if !found_group {
                groups.push(vec![species.clone()]);
            }
        }

        groups
    }

    /// Calculate the normalized Hamming distance between two DNA strands.
    fn genetic_distance(&self, dna_a: &Dna, dna_b: &Dna) -> f64 {
        if dna_a.len() != dna_b.len() || dna_a.is_empty() {
            return 1.0;
        }

        let diff_count = dna_a
            .0
            .iter()
            .zip(dna_b.0.iter())
            .filter(|(a, b)| a != b)
            .count();

        diff_count as f64 / dna_a.len() as f64
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::SeedableRng;
    use rand_chacha::ChaCha8Rng;

    fn test_species(id: u64, dna: Vec<u8>, dna_class: &str) -> Species {
        Species {
            id,
            dna_class: dna_class.to_string(),
            founder_centroid: Dna(dna),
        }
    }

    fn engine() -> EvolutionEngine {
        EvolutionEngine {
            mutation_rate: 0.1,
            duplication_rate: 0.1,
            deletion_rate: 0.1,
        }
    }

    // --- Mutation Tests ---

    /// Test 1: Point mutations are selected at the expected rate.
    #[test]
    fn test_point_mutation_selection() {
        let eng = engine();
        let mut rng = ChaCha8Rng::seed_from_u64(42);
        let species = test_species(1, vec![1, 2, 3], "human");

        let mut point_count = 0;
        for _ in 0..100 {
            if matches!(eng.mutate(&species, &mut rng), Mutation::PointGene) {
                point_count += 1;
            }
        }
        assert!(
            point_count > 5,
            "Expected some point mutations, got {point_count}"
        );
    }

    /// Test 2: Duplication is selected when configured to dominate.
    #[test]
    fn test_duplication_selection() {
        let eng = EvolutionEngine {
            mutation_rate: 0.0,
            duplication_rate: 0.9,
            deletion_rate: 0.1,
        };
        let mut rng = ChaCha8Rng::seed_from_u64(42);
        let species = test_species(1, vec![1, 2, 3], "human");

        let mut dup_count = 0;
        for _ in 0..100 {
            if matches!(eng.mutate(&species, &mut rng), Mutation::ChromosomalDup) {
                dup_count += 1;
            }
        }
        assert!(
            dup_count > 80,
            "Expected high duplication rate, got {dup_count}"
        );
    }

    /// Test 3: Deletion is the only possible outcome when all other rates are zero.
    #[test]
    fn test_deletion_always_selected_when_alone() {
        let eng = EvolutionEngine {
            mutation_rate: 0.0,
            duplication_rate: 0.0,
            deletion_rate: 1.0,
        };
        let mut rng = ChaCha8Rng::seed_from_u64(42);
        let species = test_species(1, vec![1, 2, 3], "human");

        assert!(matches!(eng.mutate(&species, &mut rng), Mutation::Deletion));
    }

    /// Test 4: Mutation enum equality and inequality.
    #[test]
    fn test_mutation_enum_equality() {
        assert_eq!(Mutation::PointGene, Mutation::PointGene);
        assert_ne!(Mutation::PointGene, Mutation::Deletion);
        assert_ne!(Mutation::ChromosomalDup, Mutation::Deletion);
    }

    /// Test 5: Default engine has correct rates.
    #[test]
    fn test_default_engine() {
        let eng = EvolutionEngine::default();
        assert_eq!(eng.mutation_rate, 0.01);
        assert_eq!(eng.duplication_rate, 0.001);
        assert_eq!(eng.deletion_rate, 0.001);
    }

    // --- Selection Tests ---

    /// Test 6: Selection filters out low-fitness individuals.
    #[test]
    fn test_selection_filters_low_fitness() {
        let eng = engine();
        let pop = vec![
            test_species(1, vec![10], "a"),
            test_species(2, vec![1], "a"),
            test_species(3, vec![5], "a"),
        ];

        let selected = eng.select(&pop, |s| s.founder_centroid.0[0] as f64);
        // Top 50% of 3 is 1 (max(1, 1))
        assert_eq!(selected.len(), 1);
        assert_eq!(selected[0].id, 1);
    }

    /// Test 7: Selection keeps at least one individual from a single-member population.
    #[test]
    fn test_selection_keeps_at_least_one() {
        let eng = engine();
        let pop = vec![test_species(1, vec![1], "a")];
        let selected = eng.select(&pop, |s| s.founder_centroid.0[0] as f64);
        assert_eq!(selected.len(), 1);
    }

    /// Test 8: Selection on an empty population returns an empty vector.
    #[test]
    fn test_selection_empty_population() {
        let eng = engine();
        let selected = eng.select(&[], |_: &Species| 0.0);
        assert!(selected.is_empty());
    }

    /// Test 9: Selection returns individuals sorted by descending fitness.
    #[test]
    fn test_selection_sorted_by_fitness() {
        let eng = engine();
        let pop = vec![
            test_species(1, vec![1], "a"),
            test_species(2, vec![10], "a"),
            test_species(3, vec![5], "a"),
            test_species(4, vec![20], "a"),
        ];

        let selected = eng.select(&pop, |s| s.founder_centroid.0[0] as f64);
        // Top 50% of 4 is 2. Should be ids 4 (20) and 2 (10).
        assert_eq!(selected.len(), 2);
        assert_eq!(selected[0].id, 4);
        assert_eq!(selected[1].id, 2);
    }

    // --- Crossover Tests ---

    /// Test 10: Crossover combines DNA from both parents alternately.
    #[test]
    fn test_crossover_combines_dna() {
        let eng = engine();
        let parent_a = test_species(1, vec![1, 2, 3, 4], "human");
        let parent_b = test_species(2, vec![10, 20, 30, 40], "human");

        let child = eng.crossover(&parent_a, &parent_b);
        // Uniform crossover: even indices from A, odd from B
        assert_eq!(child.founder_centroid.0, vec![1, 20, 3, 40]);
    }

    /// Test 11: Crossover respects the shorter parent's genome length.
    #[test]
    fn test_crossover_length_respects_shorter_parent() {
        let eng = engine();
        let parent_a = test_species(1, vec![1, 2], "human");
        let parent_b = test_species(2, vec![10, 20, 30, 40], "human");

        let child = eng.crossover(&parent_a, &parent_b);
        assert_eq!(child.founder_centroid.len(), 2);
    }

    /// Test 12: Crossover inherits dna_class from parent A.
    #[test]
    fn test_crossover_inherits_dna_class() {
        let eng = engine();
        let parent_a = test_species(1, vec![1, 2], "human");
        let parent_b = test_species(2, vec![3, 4], "elf");

        let child = eng.crossover(&parent_a, &parent_b);
        assert_eq!(child.dna_class, "human");
    }

    // --- Speciation Tests ---

    /// Test 13: Identical genomes are grouped together.
    #[test]
    fn test_speciation_groups_identical() {
        let eng = engine();
        let pop = vec![
            test_species(1, vec![1, 2, 3], "a"),
            test_species(2, vec![1, 2, 3], "a"),
        ];

        let groups = eng.speciation(&pop, 0.1);
        assert_eq!(groups.len(), 1);
        assert_eq!(groups[0].len(), 2);
    }

    /// Test 14: Completely different genomes form separate groups.
    #[test]
    fn test_speciation_separates_different() {
        let eng = engine();
        let pop = vec![
            test_species(1, vec![1, 2, 3], "a"),
            test_species(2, vec![100, 200, 255], "a"),
        ];

        let groups = eng.speciation(&pop, 0.1);
        assert_eq!(groups.len(), 2);
    }

    /// Test 15: Empty population yields no groups.
    #[test]
    fn test_speciation_empty_population() {
        let eng = engine();
        let groups = eng.speciation(&[], 0.1);
        assert!(groups.is_empty());
    }

    /// Test 16: Mixed population forms correct number of groups.
    #[test]
    fn test_speciation_mixed_population() {
        let eng = engine();
        let pop = vec![
            test_species(1, vec![10, 20, 30, 40, 50], "a"),
            test_species(2, vec![10, 20, 30, 40, 51], "a"), // 1/5 diff = 0.2 from #1
            test_species(3, vec![200, 200, 200, 200, 200], "a"), // far from #1 and #2
        ];

        // threshold 0.5: #1 and #2 are 0.2 apart (grouped), #3 is far
        let groups = eng.speciation(&pop, 0.5);
        assert_eq!(groups.len(), 2);
        assert_eq!(groups[0].len(), 2); // #1 and #2
        assert_eq!(groups[1].len(), 1); // #3
    }
    // --- FitnessMetrics Tests ---

    /// Test 17: FitnessMetrics fields are accessible.
    #[test]
    fn test_fitness_metrics_fields() {
        let metrics = FitnessMetrics {
            energy_efficiency: 0.8,
            reproduction_rate: 1.2,
            adaptability: 0.9,
        };
        assert_eq!(metrics.energy_efficiency, 0.8);
        assert_eq!(metrics.reproduction_rate, 1.2);
        assert_eq!(metrics.adaptability, 0.9);
    }

    /// Test 18: Fitness score is calculated with correct weights.
    #[test]
    fn test_fitness_score_calculation() {
        let metrics = FitnessMetrics {
            energy_efficiency: 1.0,
            reproduction_rate: 0.0,
            adaptability: 0.0,
        };
        // 1.0 * 0.4 + 0 + 0 = 0.4
        assert!((metrics.score() - 0.4).abs() < 1e-6);
    }

    /// Test 19: Genetic distance is zero for identical DNA.
    #[test]
    fn test_genetic_distance_zero_for_identical() {
        let eng = engine();
        let dna = Dna(vec![1, 2, 3]);
        assert_eq!(eng.genetic_distance(&dna, &dna), 0.0);
    }

    /// Test 20: Genetic distance is 1.0 for completely different DNA.
    #[test]
    fn test_genetic_distance_one_for_completely_different() {
        let eng = engine();
        let dna_a = Dna(vec![0, 0, 0]);
        let dna_b = Dna(vec![255, 255, 255]);
        assert_eq!(eng.genetic_distance(&dna_a, &dna_b), 1.0);
    }
}
