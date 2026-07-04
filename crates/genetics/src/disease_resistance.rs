//! Disease resistance as a heritable trait under sustained disease pressure.
//!
//! Implements [`FR-CIV-DISEASE-RESIST`]. Resistance is a single-component
//! heritable trait in the normalized `[0.0, 1.0]` space; populations under
//! sustained disease pressure select for higher mean resistance across
//! generations.
//!
//! The module is pure logic — no Bevy rendering, no LLM, no I/O. All
//! randomness threads through a caller-provided [`ChaCha8Rng`] so the
//! selection loop is replay-deterministic under a fixed seed.
//!
//! See `docs/development-guide/fr-3d-additions.md` for `FR-CIV-DISEASE-RESIST`.

use rand::Rng;
use rand_chacha::ChaCha8Rng;
use serde::{Deserialize, Serialize};

/// Per-organism disease resistance in the normalized `[0.0, 1.0]` range.
///
/// `0.0` = fully susceptible, `1.0` = fully resistant. The value is
/// interpreted as the probability that an individual survives a single
/// disease-exposure event (Bernoulli with parameter `resistance`).
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct DiseaseResistance(pub f32);

impl DiseaseResistance {
    /// Construct a resistance value, clamped to `[0.0, 1.0]`.
    #[must_use]
    pub fn new(value: f32) -> Self {
        Self(value.clamp(0.0, 1.0))
    }

    /// The raw clamped value.
    #[must_use]
    pub fn value(self) -> f32 {
        self.0
    }

    /// Probability that this organism survives one exposure to disease,
    /// given a per-event pressure in `[0.0, 1.0]`. The effective survival
    /// probability is `resistance * pressure + (1 - pressure)`, so a
    /// `pressure` of `0.0` is no exposure at all and a `pressure` of `1.0`
    /// makes survival exactly equal to the resistance trait.
    #[must_use]
    pub fn survival_probability(self, pressure: f32) -> f32 {
        let p = pressure.clamp(0.0, 1.0);
        (self.0 * p + (1.0 - p)).clamp(0.0, 1.0)
    }
}

/// Parameters controlling how disease resistance is inherited and how
/// selection acts on a population.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct DiseaseSelection {
    /// Disease pressure in `[0.0, 1.0]`. `0.0` = no exposure, `1.0` =
    /// every organism is exposed every generation.
    pub pressure: f32,
    /// Maximum absolute mutation applied to a child's resistance after
    /// blending the parents. The final value is clamped to `[0.0, 1.0]`.
    pub mutation_bound: f32,
}

impl DiseaseSelection {
    /// Construct selection parameters, clamping `pressure` and `mutation_bound`
    /// to non-negative ranges with `pressure <= 1.0`.
    #[must_use]
    pub fn new(pressure: f32, mutation_bound: f32) -> Self {
        Self {
            pressure: pressure.clamp(0.0, 1.0),
            mutation_bound: mutation_bound.max(0.0),
        }
    }
}

/// Sample whether an organism with the given resistance survives one
/// disease-exposure event under the given selection pressure. Deterministic
/// given the supplied `rng` state.
#[must_use]
pub fn survives_exposure(
    resistance: DiseaseResistance,
    selection: DiseaseSelection,
    rng: &mut ChaCha8Rng,
) -> bool {
    let p = resistance.survival_probability(selection.pressure);
    rng.gen::<f32>() < p
}

/// Blend two parent resistance values and apply bounded mutation. This is
/// the heritable component: offspring resistance starts at the arithmetic
/// mean of the two parents and is then perturbed by a uniform delta in
/// `[-mutation_bound, mutation_bound]`, clamped to `[0.0, 1.0]`.
#[must_use]
pub fn inherit_disease_resistance(
    parent_a: DiseaseResistance,
    parent_b: DiseaseResistance,
    selection: DiseaseSelection,
    rng: &mut ChaCha8Rng,
) -> DiseaseResistance {
    let blended = (parent_a.0 + parent_b.0) * 0.5;
    let bound = selection.mutation_bound;
    let delta = if bound == 0.0 {
        0.0
    } else {
        rng.gen_range(-bound..=bound)
    };
    DiseaseResistance::new(blended + delta)
}

/// Mean resistance across a population, in `[0.0, 1.0]`. Returns `0.0` for
/// an empty population.
#[must_use]
pub fn mean_resistance(population: &[DiseaseResistance]) -> f32 {
    if population.is_empty() {
        return 0.0;
    }
    let sum: f32 = population.iter().map(|r| r.0).sum();
    sum / population.len() as f32
}

/// Drive a single generation of disease selection forward.
///
/// For each organism in `current`, a Bernoulli draw with parameter
/// `resistance.survival_probability(pressure)` decides whether it survives.
/// Survivors are sampled as the parent pool for the next generation. The
/// returned vector preserves the current population size, so sustained
/// pressure changes trait distribution through survivor-biased reproduction
/// instead of conflating selection with population collapse.
///
/// If no organisms survive but disease pressure is active and at least one
/// organism has non-zero resistance, the most resistant organism is retained
/// as a bottleneck survivor. This keeps sustained selection from collapsing a
/// viable lineage to extinction due to one unlucky deterministic draw.
#[must_use]
pub fn selection_step(
    current: &[DiseaseResistance],
    selection: DiseaseSelection,
    rng: &mut ChaCha8Rng,
) -> Vec<DiseaseResistance> {
    // Determine which organisms survive this generation's exposure.
    let mut survivors: Vec<DiseaseResistance> = current
        .iter()
        .filter(|r| survives_exposure(**r, selection, rng))
        .copied()
        .collect();

    if survivors.is_empty() {
        if selection.pressure > 0.0 {
            if let Some(best) = current
                .iter()
                .copied()
                .filter(|r| r.0 > 0.0)
                .max_by(|a, b| a.0.total_cmp(&b.0))
            {
                survivors.push(best);
            }
        }
        if survivors.is_empty() {
            return Vec::new();
        }
    }

    // Breed back to the current population size from the survivor pool.
    let mut next = Vec::with_capacity(current.len());
    let n = survivors.len();
    for i in 0..current.len() {
        let a = survivors[i % n];
        let b = if n == 1 {
            a
        } else {
            survivors[rng.gen_range(0..n)]
        };
        next.push(inherit_disease_resistance(a, b, selection, rng));
    }

    next
}

/// Drive `generations` of selection steps from `initial`, returning the
/// sequence of mean population resistance at each generation (length
/// `generations + 1`, with `result[0]` being the initial mean).
///
/// If the population goes extinct at any step the trailing entries are
/// filled with `0.0`.
#[must_use]
pub fn evolve_resistance(
    initial: &[DiseaseResistance],
    selection: DiseaseSelection,
    generations: usize,
    rng: &mut ChaCha8Rng,
) -> Vec<f32> {
    let mut means = Vec::with_capacity(generations + 1);
    means.push(mean_resistance(initial));

    let mut population: Vec<DiseaseResistance> = initial.to_vec();
    for _ in 0..generations {
        if population.is_empty() {
            means.push(0.0);
            continue;
        }
        population = selection_step(&population, selection, rng);
        means.push(mean_resistance(&population));
    }

    means
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::SeedableRng;

    fn rng(seed: u64) -> ChaCha8Rng {
        ChaCha8Rng::seed_from_u64(seed)
    }

    #[test]
    fn resistance_is_clamped_into_unit_range() {
        let low = DiseaseResistance::new(-0.5);
        let high = DiseaseResistance::new(2.0);
        assert_eq!(low.value(), 0.0);
        assert_eq!(high.value(), 1.0);
    }

    #[test]
    fn survival_probability_collapses_to_resistance_at_full_pressure() {
        let r = DiseaseResistance::new(0.3);
        let p_full = r.survival_probability(1.0);
        assert!(
            (p_full - 0.3).abs() < 1e-6,
            "full pressure must yield survival = resistance (got {p_full})"
        );
    }

    #[test]
    fn survival_probability_is_one_with_zero_pressure() {
        let r = DiseaseResistance::new(0.0);
        let p_zero = r.survival_probability(0.0);
        assert!(
            (p_zero - 1.0).abs() < 1e-6,
            "zero pressure must yield survival = 1.0 (got {p_zero})"
        );
    }

    #[test]
    fn inherit_disease_resistance_clamps_to_unit_range() {
        // Two parents at the extremes with a large mutation bound must
        // still produce a value within [0.0, 1.0].
        let parent_a = DiseaseResistance::new(1.0);
        let parent_b = DiseaseResistance::new(1.0);
        let selection = DiseaseSelection::new(1.0, 1.0);
        let mut r = rng(7);
        let child = inherit_disease_resistance(parent_a, parent_b, selection, &mut r);
        assert!((0.0..=1.0).contains(&child.value()));
    }

    #[test]
    fn mean_resistance_is_zero_for_empty_population() {
        assert_eq!(mean_resistance(&[]), 0.0);
    }

    /// Acceptance test for [`FR-CIV-DISEASE-RESIST`]: under sustained
    /// disease pressure, mean resistance rises across generations.
    ///
    /// Starts a population at a low baseline mean resistance and drives
    /// many generations of selection with `pressure = 1.0`. Asserts that
    /// the final mean is strictly greater than the initial mean.
    #[test]
    fn mean_resistance_rises_under_sustained_disease_pressure() {
        // Initial population: 64 organisms with uniformly low resistance.
        let initial: Vec<DiseaseResistance> = (0..64)
            .map(|i| DiseaseResistance::new(0.05 + (i as f32) * 0.001))
            .collect();
        let initial_mean = mean_resistance(&initial);

        // Sustained full pressure, small mutation bound to keep the
        // signal clear while still allowing heritable variation.
        let selection = DiseaseSelection::new(1.0, 0.02);
        let mut r = rng(2024);

        let history = evolve_resistance(&initial, selection, 60, &mut r);
        assert_eq!(
            history.len(),
            61,
            "should record initial mean + 60 generations"
        );
        assert!(
            (history[0] - initial_mean).abs() < 1e-6,
            "first entry must equal initial mean (got {} vs {})",
            history[0],
            initial_mean
        );

        let final_mean = *history.last().unwrap();
        assert!(
            final_mean > initial_mean,
            "FR-CIV-DISEASE-RESIST: mean resistance must rise under sustained \
             disease pressure (initial {initial_mean}, final {final_mean})"
        );
        // The rise should be material, not a rounding artifact.
        assert!(
            final_mean - initial_mean > 0.05,
            "expected a meaningful rise in mean resistance (initial \
             {initial_mean}, final {final_mean})"
        );
    }

    /// Companion property: with zero pressure, the mean resistance
    /// should drift only within the mutation bound — no systematic
    /// directional selection. This guards against the acceptance test
    /// passing for the wrong reason (e.g. unconditional upward drift).
    #[test]
    fn mean_resistance_does_not_rise_under_zero_pressure() {
        let initial: Vec<DiseaseResistance> = (0..64)
            .map(|i| DiseaseResistance::new(0.4 + (i as f32) * 0.001))
            .collect();
        let initial_mean = mean_resistance(&initial);

        let selection = DiseaseSelection::new(0.0, 0.02);
        let mut r = rng(2025);

        let history = evolve_resistance(&initial, selection, 60, &mut r);
        let final_mean = *history.last().unwrap();
        let drift = (final_mean - initial_mean).abs();
        assert!(
            drift < 0.05,
            "with zero pressure the population must not drift directionally \
             (initial {initial_mean}, final {final_mean}, |drift| {drift})"
        );
    }

    /// Determinism guard: identical seeds must produce identical selection
    /// trajectories. This is the replay-determinism contract for the
    /// module.
    #[test]
    fn selection_step_is_deterministic_under_fixed_seed() {
        let initial: Vec<DiseaseResistance> = (0..32)
            .map(|i| DiseaseResistance::new(0.1 + (i as f32) * 0.01))
            .collect();
        let selection = DiseaseSelection::new(1.0, 0.03);

        let mut r1 = rng(99);
        let history_1 = evolve_resistance(&initial, selection, 40, &mut r1);

        let mut r2 = rng(99);
        let history_2 = evolve_resistance(&initial, selection, 40, &mut r2);

        assert_eq!(
            history_1, history_2,
            "trajectory must be deterministic under fixed seed"
        );
    }
}
