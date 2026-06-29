//! Trait inheritance helpers for emergent lineages.
//!
//! Offspring traits are produced by blending parent traits and then applying a
//! bounded mutation so the result stays close to the parental range.

use rand::Rng;
use rand_chacha::ChaCha8Rng;
use serde::{Deserialize, Serialize};

/// A bounded trait vector in normalized `[0.0, 1.0]` space.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TraitVector(pub Vec<f32>);

impl TraitVector {
    /// Construct a trait vector from raw values.
    #[must_use]
    pub fn new(values: Vec<f32>) -> Self {
        Self(values)
    }

    /// Length of the vector.
    #[must_use]
    pub fn len(&self) -> usize {
        self.0.len()
    }

    /// True when there are no traits.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}

/// Parameters controlling offspring trait inheritance.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct TraitInheritance {
    /// Maximum absolute mutation applied after blending.
    pub mutation_bound: f32,
}

impl TraitInheritance {
    /// Construct inheritance parameters.
    #[must_use]
    pub fn new(mutation_bound: f32) -> Self {
        Self {
            mutation_bound: mutation_bound.max(0.0),
        }
    }
}

/// Blend parent trait vectors and apply bounded mutation.
///
/// Each child component starts as the arithmetic mean of the two parents and
/// then receives a random delta in `[-mutation_bound, mutation_bound]`.
/// The final value is clamped to `[0.0, 1.0]`.
#[must_use]
pub fn inherit_trait_vector(
    parent_a: &TraitVector,
    parent_b: &TraitVector,
    rng: &mut ChaCha8Rng,
    inheritance: TraitInheritance,
) -> TraitVector {
    assert_eq!(
        parent_a.0.len(),
        parent_b.0.len(),
        "inherit_trait_vector: parent length mismatch"
    );

    let mut child = Vec::with_capacity(parent_a.0.len());
    let bound = inheritance.mutation_bound;
    for (a, b) in parent_a.0.iter().zip(parent_b.0.iter()) {
        let blended = (a + b) * 0.5;
        let delta = if bound == 0.0 {
            0.0
        } else {
            rng.gen_range(-bound..=bound)
        };
        child.push((blended + delta).clamp(0.0, 1.0));
    }

    TraitVector(child)
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::SeedableRng;

    fn rng(seed: u64) -> ChaCha8Rng {
        ChaCha8Rng::seed_from_u64(seed)
    }

    #[test]
    fn child_trait_stays_within_parent_blend_and_mutation_bound() {
        let parent_a = TraitVector::new(vec![0.1, 0.2, 0.3, 0.4]);
        let parent_b = TraitVector::new(vec![0.9, 0.8, 0.7, 0.6]);
        let inheritance = TraitInheritance::new(0.05);
        let mut rng = rng(1234);

        let child = inherit_trait_vector(&parent_a, &parent_b, &mut rng, inheritance);

        assert_eq!(child.len(), parent_a.len());
        for ((a, b), c) in parent_a.0.iter().zip(parent_b.0.iter()).zip(child.0.iter()) {
            let a = *a;
            let b = *b;
            let c = *c;
            let blended = (a + b) * 0.5;
            assert!(
                (c - blended).abs() <= inheritance.mutation_bound + f32::EPSILON,
                "child trait {c} must stay within mutation bound of blended parent value {blended}"
            );
            let min_parent = a.min(b);
            let max_parent = a.max(b);
            assert!(
                c >= (min_parent - inheritance.mutation_bound).max(0.0)
                    && c <= (max_parent + inheritance.mutation_bound).min(1.0),
                "child trait {c} must stay between parents within mutation bounds"
            );
        }
    }
}
