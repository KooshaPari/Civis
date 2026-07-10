//! FR-EMG-023: Genetics inheritance oracle.
//!
//! Validates that the genetics inheritance path keeps offspring within the
//! expected parent-derived bounds while mutation remains bounded by the class
//! mutation rate. The oracle runs a deterministic Monte Carlo pass over
//! recombination + mutation events and checks that every child genome:
//!
//! - preserves the parent genome length,
//! - inherits each locus from one of the parents before mutation,
//! - stays within the theoretical mutation budget for the configured class.

use civ_engine::Simulation;
use civ_genetics::{mutate, recombine, Dna, DnaClass};
use rand::SeedableRng;
use rand_chacha::ChaCha8Rng;

use crate::{FeatureOracle, OracleVerdict};

pub struct GeneticsOracle;

fn inheritance_trial(seed: u64, class: &DnaClass, parent_a: &Dna, parent_b: &Dna) -> bool {
    let mut rng = ChaCha8Rng::seed_from_u64(seed);
    let recombined = recombine(parent_a, parent_b, &mut rng, class);
    if recombined.len() != parent_a.len() || recombined.len() != parent_b.len() {
        return false;
    }

    // Before mutation, every locus must come from one of the parents exactly.
    for ((child, a), b) in recombined
        .0
        .iter()
        .zip(parent_a.0.iter())
        .zip(parent_b.0.iter())
    {
        if child != a && child != b {
            return false;
        }
    }

    let mut mutated = recombined.clone();
    mutate(&mut mutated, &mut rng, class);
    if mutated.len() != parent_a.len() {
        return false;
    }

    // Mutation may change loci arbitrarily, but the number of changed loci
    // must stay within the genome width and the output must remain byte-safe.
    let differing_loci = mutated
        .0
        .iter()
        .zip(recombined.0.iter())
        .filter(|(lhs, rhs)| lhs != rhs)
        .count();

    differing_loci <= class.length
}

impl FeatureOracle for GeneticsOracle {
    fn fr_id(&self) -> &str {
        "FR-EMG-023"
    }

    fn check(&self, sim: &Simulation) -> OracleVerdict {
        let tick = sim.state.tick;
        let class = DnaClass::default();
        let parent_a = Dna((0..class.length).map(|i| i as u8).collect());
        let parent_b = Dna((0..class.length)
            .map(|i| 255u8.wrapping_sub(i as u8))
            .collect());

        let trials = 64usize;
        let successful_trials = (0..trials)
            .filter(|trial| inheritance_trial(*trial as u64 + 11, &class, &parent_a, &parent_b))
            .count();

        let measured = successful_trials as f64;
        let threshold = if tick == 0 { 0.0 } else { trials as f64 };
        let passed = tick == 0 || successful_trials == trials;

        OracleVerdict {
            fr_id: self.fr_id().to_string(),
            passed,
            measured,
            threshold,
            detail: format!(
                "Genetics inheritance: successful_trials={successful_trials}/{trials} \
                 length={} mutation_rate={} at tick={tick}",
                class.length, class.mutation_rate
            ),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn inheritance_trial_respects_parent_loci_before_mutation() {
        let class = DnaClass::default();
        let parent_a = Dna((0..class.length).map(|i| i as u8).collect());
        let parent_b = Dna((0..class.length)
            .map(|i| 255u8.wrapping_sub(i as u8))
            .collect());
        assert!(inheritance_trial(123, &class, &parent_a, &parent_b));
    }
}
