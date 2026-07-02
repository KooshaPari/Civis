//! Trait inheritance for emergent offspring phenotypes.
//!
//! Covers FR-CIV-TRAIT-INHERIT: an offspring trait is the blend of parent
//! traits plus bounded mutation.

use rand::Rng;

/// Configuration for scalar trait inheritance.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TraitInheritanceConfig {
    /// Inclusive lower bound for the inherited trait value.
    pub min_value: f32,
    /// Inclusive upper bound for the inherited trait value.
    pub max_value: f32,
    /// Maximum absolute mutation delta applied after parental blending.
    pub mutation_bound: f32,
}

impl TraitInheritanceConfig {
    /// Construct a trait inheritance configuration.
    #[must_use]
    pub fn new(min_value: f32, max_value: f32, mutation_bound: f32) -> Self {
        Self {
            min_value,
            max_value,
            mutation_bound,
        }
    }

    /// Return the finite, non-negative mutation bound used at inheritance time.
    #[must_use]
    pub fn bounded_mutation(self) -> f32 {
        if self.mutation_bound.is_finite() {
            self.mutation_bound.max(0.0)
        } else {
            0.0
        }
    }
}

impl Default for TraitInheritanceConfig {
    fn default() -> Self {
        Self {
            min_value: 0.0,
            max_value: 1.0,
            mutation_bound: 0.05,
        }
    }
}

/// Blend two parent trait values and apply bounded mutation.
///
/// The parental blend is the arithmetic mean. Mutation is sampled uniformly
/// from `[-mutation_bound, mutation_bound]`, then the result is clamped to the
/// configured trait range.
#[must_use]
pub fn inherit_trait<R: Rng + ?Sized>(
    parent_a: f32,
    parent_b: f32,
    rng: &mut R,
    config: TraitInheritanceConfig,
) -> f32 {
    let min_value = config.min_value.min(config.max_value);
    let max_value = config.min_value.max(config.max_value);
    let blend = (parent_a + parent_b) * 0.5;
    let bound = config.bounded_mutation();
    let mutation = if bound > 0.0 {
        rng.gen_range(-bound..=bound)
    } else {
        0.0
    };

    (blend + mutation).clamp(min_value, max_value)
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::SeedableRng;
    use rand_chacha::ChaCha8Rng;

    /// Covers FR-CIV-TRAIT-INHERIT: child trait stays between parent extrema
    /// extended by the configured mutation bound.
    #[test]
    fn child_trait_lies_between_parents_within_mutation_bound() {
        let parent_a = 0.25_f32;
        let parent_b = 0.75_f32;
        let config = TraitInheritanceConfig::new(0.0, 1.0, 0.1);
        let mut rng = ChaCha8Rng::seed_from_u64(0xC1A0_71A1_u64);

        let child = inherit_trait(parent_a, parent_b, &mut rng, config);
        let lower = parent_a.min(parent_b) - config.mutation_bound;
        let upper = parent_a.max(parent_b) + config.mutation_bound;

        assert!(
            child >= lower && child <= upper,
            "child trait {child} must lie within parent range plus mutation bound [{lower}, {upper}]"
        );
    }
}
