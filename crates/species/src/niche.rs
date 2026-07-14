//! FR-CIV-NICHE-ADAPT — species adapt to environmental niches.
//!
//! Pure-logic helpers that model how well a species' expressed
//! [`Phenotype`](crate::Phenotype) fits a given environmental [`Niche`].
//!
//! The model is intentionally simple and substrate-agnostic:
//!
//! - A niche declares *optima* for each phenotypic axis (morphology height
//!   and hue, behaviour weights, ...). Callers populate the optima from the
//!   environment layer (climate, terrain, biome, ...).
//! - [`niche_match`] returns a scalar in `[0.0, 1.0]` — `1.0` when the
//!   phenotype exactly hits every optimum, `0.0` when it is the worst
//!   possible fit along every axis.
//! - [`niche_growth_rate`] and [`niche_mortality_rate`] convert the
//!   match into per-tick rates in `[0.0, 1.0]`. A perfect match yields
//!   positive growth; a zero match yields positive mortality; in
//!   between the two can cancel out (or the growth-side dominates when
//!   the niche is favourable).
//! - [`tick_niche_adaptation`] composes the rates with a baseline
//!   mortality into a deterministic per-tick population update.
//!
//! The functions are total over their inputs and side-effect-free.

use crate::{BehaviorWeights, Morphology, Phenotype};

/// Per-axis weights used to compute how close a phenotype's traits are to
/// the niche optima. All weights are non-negative and summed into a
/// normalisation denominator at runtime; passing all-zero weights is
/// allowed and yields an axis-mean of zero (i.e. neutral), which is the
/// only pathological input we special-case below.
#[derive(Debug, Clone, Copy, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct NicheWeights {
    /// Importance of `Morphology.height_cm` distance to optimum.
    pub morphology_height: f32,
    /// Importance of `Morphology.body_color_hue` distance to optimum.
    pub morphology_hue: f32,
    /// Importance of `Morphology.leg_count` distance to optimum.
    pub morphology_legs: f32,
    /// Importance of `Morphology.arm_count` distance to optimum.
    pub morphology_arms: f32,
    /// Importance of `Morphology.eye_count` distance to optimum.
    pub morphology_eyes: f32,
    /// Importance of `BehaviorWeights.aggression` distance to optimum.
    pub behavior_aggression: f32,
    /// Importance of `BehaviorWeights.curiosity` distance to optimum.
    pub behavior_curiosity: f32,
    /// Importance of `BehaviorWeights.sociability` distance to optimum.
    pub behavior_sociability: f32,
    /// Importance of `BehaviorWeights.intelligence` distance to optimum.
    pub behavior_intelligence: f32,
}

impl NicheWeights {
    /// Equal weights on every axis (one over the axis count each).
    ///
    /// Convenient for tests and for callers that want a flat trait
    /// distance without manual tuning.
    #[must_use]
    pub const fn uniform() -> Self {
        Self {
            morphology_height: 1.0,
            morphology_hue: 1.0,
            morphology_legs: 1.0,
            morphology_arms: 1.0,
            morphology_eyes: 1.0,
            behavior_aggression: 1.0,
            behavior_curiosity: 1.0,
            behavior_sociability: 1.0,
            behavior_intelligence: 1.0,
        }
    }

    /// Sum of all per-axis weights. Used as the normalisation denominator
    /// when computing the weighted niche match.
    #[must_use]
    pub fn total(&self) -> f32 {
        self.morphology_height
            + self.morphology_hue
            + self.morphology_legs
            + self.morphology_arms
            + self.morphology_eyes
            + self.behavior_aggression
            + self.behavior_curiosity
            + self.behavior_sociability
            + self.behavior_intelligence
    }
}

impl Default for NicheWeights {
    fn default() -> Self {
        Self::uniform()
    }
}

/// An environmental niche: optimal phenotype axes plus sensitivity.
#[derive(Debug, Clone, Copy, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct Niche {
    /// Optimal `Morphology.height_cm` value (0..=255).
    pub opt_height_cm: u8,
    /// Optimal `Morphology.body_color_hue` value (0..=255).
    pub opt_body_color_hue: u8,
    /// Optimal `Morphology.leg_count` value (0..=255 — type allows it).
    pub opt_leg_count: u8,
    /// Optimal `Morphology.arm_count` value (0..=255).
    pub opt_arm_count: u8,
    /// Optimal `Morphology.eye_count` value (0..=255).
    pub opt_eye_count: u8,
    /// Optimal aggression weight in `[0.0, 1.0]`.
    pub opt_aggression: f32,
    /// Optimal curiosity weight in `[0.0, 1.0]`.
    pub opt_curiosity: f32,
    /// Optimal sociability weight in `[0.0, 1.0]`.
    pub opt_sociability: f32,
    /// Optimal intelligence weight in `[0.0, 1.0]`.
    pub opt_intelligence: f32,
    /// Per-axis importance weights.
    pub weights: NicheWeights,
}

impl Niche {
    /// Construct a niche that has the supplied morphology + behaviour
    /// optima and a chosen set of [`NicheWeights`]. Convenience for tests
    /// and for callers building a niche from climate/biome data.
    #[must_use]
    pub fn new(morphology: Morphology, behavior: BehaviorWeights, weights: NicheWeights) -> Self {
        Self {
            opt_height_cm: morphology.height_cm,
            opt_body_color_hue: morphology.body_color_hue,
            opt_leg_count: morphology.leg_count,
            opt_arm_count: morphology.arm_count,
            opt_eye_count: morphology.eye_count,
            opt_aggression: behavior.aggression,
            opt_curiosity: behavior.curiosity,
            opt_sociability: behavior.sociability,
            opt_intelligence: behavior.intelligence,
            weights,
        }
    }
}

/// Niche match score in `[0.0, 1.0]`.
///
/// `1.0` means the supplied phenotype exactly hits the niche optima on
/// every axis; `0.0` means it is as far as possible from the optima on
/// every axis. The score is a weighted mean of per-axis closeness
/// scores, so a deviation on an important axis hurts the score more
/// than a deviation on a lightly-weighted one.
///
/// Per-axis closeness is `1 - distance/max_distance`, where the max
/// distance is `1.0` (since all axes are normalised to `[0, 1]`). With
/// all weights at `uniform()` this reduces to the simple mean of
/// per-axis closeness scores.
///
/// When all weights are zero, the function returns `0.5` (neutral) to
/// avoid a division-by-zero. This matches the "weights say nothing
/// matters" interpretation: the niche is effectively uninformative.
#[must_use]
pub fn niche_match(phenotype: &Phenotype, niche: &Niche) -> f32 {
    let total_w = niche.weights.total();
    if total_w <= 0.0 {
        return 0.5;
    }

    // Per-axis closeness: 1.0 at distance 0, 0.0 at distance 1.
    let morph_height =
        1.0 - (f32::from(phenotype.morphology.height_cm.abs_diff(niche.opt_height_cm)) / 255.0);
    let morph_hue = 1.0
        - (f32::from(
            phenotype
                .morphology
                .body_color_hue
                .abs_diff(niche.opt_body_color_hue),
        ) / 255.0);
    let morph_legs =
        1.0 - (f32::from(phenotype.morphology.leg_count.abs_diff(niche.opt_leg_count)) / 255.0);
    let morph_arms =
        1.0 - (f32::from(phenotype.morphology.arm_count.abs_diff(niche.opt_arm_count)) / 255.0);
    let morph_eyes =
        1.0 - (f32::from(phenotype.morphology.eye_count.abs_diff(niche.opt_eye_count)) / 255.0);

    let behav_aggr = 1.0 - (phenotype.behavior.aggression - niche.opt_aggression).abs();
    let behav_curio = 1.0 - (phenotype.behavior.curiosity - niche.opt_curiosity).abs();
    let behav_soc = 1.0 - (phenotype.behavior.sociability - niche.opt_sociability).abs();
    let behav_int = 1.0 - (phenotype.behavior.intelligence - niche.opt_intelligence).abs();

    let weighted = morph_height * niche.weights.morphology_height
        + morph_hue * niche.weights.morphology_hue
        + morph_legs * niche.weights.morphology_legs
        + morph_arms * niche.weights.morphology_arms
        + morph_eyes * niche.weights.morphology_eyes
        + behav_aggr * niche.weights.behavior_aggression
        + behav_curio * niche.weights.behavior_curiosity
        + behav_soc * niche.weights.behavior_sociability
        + behav_int * niche.weights.behavior_intelligence;

    // Closeness scores are already in [0, 1]; clamp defensively in case
    // a caller passes out-of-range behaviour weights.
    (weighted / total_w).clamp(0.0, 1.0)
}

/// Per-tick growth rate in `[0.0, 1.0]` driven by a niche match.
///
/// `match == 1.0` → `max_growth`; `match == 0.0` → `0.0`; the
/// relationship is linear in `match`. `max_growth` is clamped to
/// `[0.0, 1.0]` defensively.
#[must_use]
pub fn niche_growth_rate(match_score: f32, max_growth: f32) -> f32 {
    let m = match_score.clamp(0.0, 1.0);
    let g = max_growth.clamp(0.0, 1.0);
    m * g
}

/// Per-tick mortality rate in `[0.0, 1.0]` driven by niche mismatch.
///
/// `match == 0.0` → `max_mortality`; `match == 1.0` → `0.0`; the
/// relationship is linear in `(1 - match)`. `max_mortality` is clamped
/// to `[0.0, 1.0]` defensively.
#[must_use]
pub fn niche_mortality_rate(match_score: f32, max_mortality: f32) -> f32 {
    let m = match_score.clamp(0.0, 1.0);
    let mm = max_mortality.clamp(0.0, 1.0);
    (1.0 - m) * mm
}

/// Result of one niche-adaptation tick.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct NicheTick {
    /// Niche match score that drove the rate calculation.
    pub match_score: f32,
    /// Per-tick growth (births / population) in `[0.0, 1.0]`.
    pub growth_rate: f32,
    /// Per-tick niche-induced mortality in `[0.0, 1.0]`.
    pub niche_mortality: f32,
    /// Total deaths (niche + baseline) for this tick.
    pub deaths: u64,
    /// Number of births for this tick.
    pub births: u64,
    /// Population after the tick (saturating at `0`).
    pub population_after: u64,
}

/// Apply niche-driven growth and mortality for one tick.
///
/// The model is intentionally simple and mirrors [`tick_pressure_loss`]:
///
/// - Pre-tick population `N` produces `floor(growth_rate * N)` births.
/// - Total mortality `mortality = clamp(baseline + niche_mortality, 0, 1)`
///   produces `floor(mortality * N)` deaths.
/// - Population after tick is `N + births - deaths`, saturated at 0.
///
/// `baseline_mortality` is the existing per-tick death rate (predation,
/// disease, age, …) and is composable with the niche term, just like
/// [`crate::apply_pressure_loss`] composes pressure on top of baseline.
#[must_use]
pub fn tick_niche_adaptation(
    population: u64,
    phenotype: &Phenotype,
    niche: &Niche,
    baseline_mortality: f32,
    max_growth: f32,
    max_mortality: f32,
) -> NicheTick {
    let match_score = niche_match(phenotype, niche);
    let growth = niche_growth_rate(match_score, max_growth);
    let niche_mort = niche_mortality_rate(match_score, max_mortality);
    let baseline = baseline_mortality.clamp(0.0, 1.0);
    let total_mortality = (baseline + niche_mort).clamp(0.0, 1.0);

    let pop_f = population as f64;
    let births = (growth as f64 * pop_f).floor().max(0.0) as u64;
    let deaths = (total_mortality as f64 * pop_f).floor().max(0.0) as u64;

    let population_after = population.saturating_add(births).saturating_sub(deaths);

    NicheTick {
        match_score,
        growth_rate: growth,
        niche_mortality: niche_mort,
        deaths,
        births,
        population_after,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn niche_exact_for(phenotype: &Phenotype) -> Niche {
        Niche::new(
            phenotype.morphology,
            phenotype.behavior,
            NicheWeights::uniform(),
        )
    }

    fn phenotype_with(morphology: Morphology, behavior: BehaviorWeights) -> Phenotype {
        Phenotype {
            morphology,
            behavior,
        }
    }

    fn zero_phenotype() -> Phenotype {
        phenotype_with(
            Morphology {
                height_cm: 0,
                body_color_hue: 0,
                leg_count: 0,
                arm_count: 0,
                eye_count: 0,
            },
            BehaviorWeights {
                aggression: 0.0,
                curiosity: 0.0,
                sociability: 0.0,
                intelligence: 0.0,
            },
        )
    }

    // -----------------------------------------------------------------------
    // Acceptance: matched niche grows population; mismatched shrinks it
    // (FR-CIV-NICHE-ADAPT).
    // -----------------------------------------------------------------------

    /// Covers FR-CIV-NICHE-ADAPT — when a species' phenotype hits the
    /// niche optima on every axis, the per-tick growth rate is positive
    /// and the per-tick mortality from mismatch is zero, so the
    /// population grows.
    #[test]
    fn matched_niche_grows_population() {
        let pheno = phenotype_with(
            Morphology {
                height_cm: 120,
                body_color_hue: 80,
                leg_count: 4,
                arm_count: 2,
                eye_count: 2,
            },
            BehaviorWeights {
                aggression: 0.5,
                curiosity: 0.6,
                sociability: 0.7,
                intelligence: 0.4,
            },
        );
        let niche = niche_exact_for(&pheno);
        let start = 1_000_u64;
        let tick = tick_niche_adaptation(
            start, &pheno, &niche, 0.0, 0.1, // max_growth
            0.1, // max_mortality
        );
        // Perfect match → 0 mortality from niche, growth = 0.1 × 1000 = 100.
        assert!(
            (tick.match_score - 1.0).abs() < 1e-6,
            "perfect phenotype match should yield match_score = 1.0, got {}",
            tick.match_score,
        );
        assert_eq!(tick.niche_mortality, 0.0);
        assert_eq!(tick.deaths, 0);
        assert_eq!(tick.births, 100);
        assert_eq!(tick.population_after, start + 100);
        assert!(
            tick.population_after > start,
            "matched niche must grow population ({} -> {})",
            start,
            tick.population_after,
        );
    }

    /// Covers FR-CIV-NICHE-ADAPT — when a species' phenotype is the
    /// worst possible fit on every axis, the per-tick mortality from
    /// mismatch is at its maximum and the growth is zero, so the
    /// population shrinks.
    #[test]
    fn mismatched_niche_shrinks_population() {
        let pheno = zero_phenotype();
        // Niche optima are at the opposite extreme of every axis.
        let niche_morph = Morphology {
            height_cm: 255,
            body_color_hue: 255,
            leg_count: 255,
            arm_count: 255,
            eye_count: 255,
        };
        let niche_behav = BehaviorWeights {
            aggression: 1.0,
            curiosity: 1.0,
            sociability: 1.0,
            intelligence: 1.0,
        };
        let niche = Niche::new(niche_morph, niche_behav, NicheWeights::uniform());

        let start = 1_000_u64;
        let tick = tick_niche_adaptation(
            start, &pheno, &niche, 0.0,  // no baseline; only niche-driven mortality
            0.05, // max_growth
            0.2,  // max_mortality
        );
        assert!(
            tick.match_score < 1e-6,
            "opposite-endpoints phenotype must match the niche on no axes, got {}",
            tick.match_score,
        );
        assert!(
            tick.niche_mortality > 0.0,
            "mismatch must drive positive niche mortality, got {}",
            tick.niche_mortality,
        );
        assert!(
            tick.population_after < start,
            "mismatched niche must shrink population ({} -> {})",
            start,
            tick.population_after,
        );
        // Growth should be zero (or near-zero): match_score = 0 → 0 births.
        assert_eq!(tick.births, 0);
        // Total deaths > 0; saturated deaths ≤ total_mortality × pop.
        assert!(tick.deaths > 0);
    }

    /// Covers FR-CIV-NICHE-ADAPT — the same population over the same
    /// niche but with two different phenotypes: the matched one grows,
    /// the mismatched one shrinks. This is the acceptance test from the
    /// FR: matched → grows, mismatched → shrinks.
    #[test]
    fn matched_grows_mismatched_shrinks_same_niche() {
        let niche_morph = Morphology {
            height_cm: 150,
            body_color_hue: 100,
            leg_count: 4,
            arm_count: 2,
            eye_count: 2,
        };
        let niche_behav = BehaviorWeights {
            aggression: 0.4,
            curiosity: 0.7,
            sociability: 0.5,
            intelligence: 0.6,
        };
        let niche = Niche::new(niche_morph, niche_behav, NicheWeights::uniform());

        let matched = Phenotype {
            morphology: niche_morph,
            behavior: niche_behav,
        };
        let mismatched = phenotype_with(
            Morphology {
                height_cm: 0,
                body_color_hue: 255,
                leg_count: 255,
                arm_count: 255,
                eye_count: 255,
            },
            BehaviorWeights {
                aggression: 1.0,
                curiosity: 0.0,
                sociability: 1.0,
                intelligence: 0.0,
            },
        );

        let pop = 500_u64;
        let matched_tick = tick_niche_adaptation(pop, &matched, &niche, 0.0, 0.1, 0.1);
        let mismatched_tick = tick_niche_adaptation(pop, &mismatched, &niche, 0.0, 0.1, 0.1);

        assert!(
            matched_tick.population_after > pop,
            "matched population must grow ({} -> {})",
            pop,
            matched_tick.population_after,
        );
        assert!(
            mismatched_tick.population_after < pop,
            "mismatched population must shrink ({} -> {})",
            pop,
            mismatched_tick.population_after,
        );
    }

    // -----------------------------------------------------------------------
    // Additional boundary / invariant tests (pure-logic, additive).
    // -----------------------------------------------------------------------

    /// Covers FR-CIV-NICHE-ADAPT — `niche_match` returns 1.0 for an
    /// exact phenotype/optimum match and 0.0 for the maximally distant
    /// one on every axis.
    #[test]
    fn niche_match_extremes_are_zero_and_one() {
        let exact = phenotype_with(
            Morphology {
                height_cm: 100,
                body_color_hue: 50,
                leg_count: 4,
                arm_count: 2,
                eye_count: 2,
            },
            BehaviorWeights {
                aggression: 0.3,
                curiosity: 0.3,
                sociability: 0.3,
                intelligence: 0.3,
            },
        );
        let niche = niche_exact_for(&exact);
        let m_exact = niche_match(&exact, &niche);
        assert!(
            (m_exact - 1.0).abs() < 1e-6,
            "exact match should yield 1.0, got {m_exact}",
        );

        // Opposite extreme on every axis: closeness per axis is 0.0.
        let opposite_morph = Morphology {
            height_cm: if exact.morphology.height_cm == 0 {
                255
            } else {
                0
            },
            body_color_hue: if exact.morphology.body_color_hue == 0 {
                255
            } else {
                0
            },
            leg_count: if exact.morphology.leg_count == 0 {
                255
            } else {
                0
            },
            arm_count: if exact.morphology.arm_count == 0 {
                255
            } else {
                0
            },
            eye_count: if exact.morphology.eye_count == 0 {
                255
            } else {
                0
            },
        };
        let opposite_behav = BehaviorWeights {
            // Pick the opposite extreme in [0, 1] — for 0.3 → 1.0.
            aggression: 1.0 - exact.behavior.aggression,
            curiosity: 1.0 - exact.behavior.curiosity,
            sociability: 1.0 - exact.behavior.sociability,
            intelligence: 1.0 - exact.behavior.intelligence,
        };
        let opposite_pheno = Phenotype {
            morphology: opposite_morph,
            behavior: opposite_behav,
        };
        let opp_niche = Niche::new(opposite_morph, opposite_behav, NicheWeights::uniform());
        let m_opp = niche_match(&opposite_pheno, &opp_niche);
        assert!(
            (m_opp - 1.0).abs() < 1e-6,
            "self-match of opposite-niche should yield 1.0, got {m_opp}",
        );

        // Direct phenotype vs. niche at opposite ends (both extremes on
        // every axis → each per-axis closeness is 0.0).
        let zero_morph = Morphology {
            height_cm: 0,
            body_color_hue: 0,
            leg_count: 0,
            arm_count: 0,
            eye_count: 0,
        };
        let zero_behav = BehaviorWeights {
            aggression: 0.0,
            curiosity: 0.0,
            sociability: 0.0,
            intelligence: 0.0,
        };
        let zero_pheno = Phenotype {
            morphology: zero_morph,
            behavior: zero_behav,
        };
        let max_niche = Niche::new(
            Morphology {
                height_cm: 255,
                body_color_hue: 255,
                leg_count: 255,
                arm_count: 255,
                eye_count: 255,
            },
            BehaviorWeights {
                aggression: 1.0,
                curiosity: 1.0,
                sociability: 1.0,
                intelligence: 1.0,
            },
            NicheWeights::uniform(),
        );
        let m_zero = niche_match(&zero_pheno, &max_niche);
        assert!(
            m_zero < 1e-6,
            "maximally distant phenotype should match ~0, got {m_zero}",
        );
    }

    /// Covers FR-CIV-NICHE-ADAPT — `niche_growth_rate` and
    /// `niche_mortality_rate` are linear in `(match, 1 - match)` and
    /// both clamp their arguments.
    #[test]
    fn growth_and_mortality_rates_are_linear_and_clamped() {
        // Perfect match: full growth, zero mortality.
        assert!((niche_growth_rate(1.0, 0.2) - 0.2).abs() < 1e-6);
        assert!((niche_mortality_rate(1.0, 0.5) - 0.0).abs() < 1e-6);

        // Zero match: zero growth, full mortality.
        assert!((niche_growth_rate(0.0, 0.2) - 0.0).abs() < 1e-6);
        assert!((niche_mortality_rate(0.0, 0.5) - 0.5).abs() < 1e-6);

        // Half match: half growth, half mortality.
        assert!((niche_growth_rate(0.5, 0.2) - 0.1).abs() < 1e-6);
        assert!((niche_mortality_rate(0.5, 0.5) - 0.25).abs() < 1e-6);

        // Out-of-range match scores are clamped.
        assert!((niche_growth_rate(2.0, 0.2) - 0.2).abs() < 1e-6);
        assert!((niche_mortality_rate(-1.0, 0.5) - 0.5).abs() < 1e-6);

        // Out-of-range caps are clamped.
        assert!((niche_growth_rate(1.0, 5.0) - 1.0).abs() < 1e-6);
        assert!((niche_mortality_rate(0.0, 5.0) - 1.0).abs() < 1e-6);
    }

    /// Covers FR-CIV-NICHE-ADAPT — when all weights are zero the
    /// niche is effectively a "no information" signal, and the helper
    /// resolves that as a neutral 0.5 match instead of dividing by zero.
    #[test]
    fn zero_weights_yield_neutral_half_match() {
        let pheno = zero_phenotype();
        let weights = NicheWeights {
            morphology_height: 0.0,
            morphology_hue: 0.0,
            morphology_legs: 0.0,
            morphology_arms: 0.0,
            morphology_eyes: 0.0,
            behavior_aggression: 0.0,
            behavior_curiosity: 0.0,
            behavior_sociability: 0.0,
            behavior_intelligence: 0.0,
        };
        let niche = Niche::new(
            Morphology {
                height_cm: 0,
                body_color_hue: 0,
                leg_count: 0,
                arm_count: 0,
                eye_count: 0,
            },
            BehaviorWeights {
                aggression: 0.0,
                curiosity: 0.0,
                sociability: 0.0,
                intelligence: 0.0,
            },
            weights,
        );
        let m = niche_match(&pheno, &niche);
        assert!(
            (m - 0.5).abs() < 1e-6,
            "zero-weight niche should be neutral (0.5), got {m}",
        );
    }

    /// Covers FR-CIV-NICHE-ADAPT — `tick_niche_adaptation` composes the
    /// niche mortality additively on top of the baseline, mirrors the
    /// behavior of `apply_pressure_loss`, and clamps the combined rate
    /// to `[0.0, 1.0]`. Pure-logic sanity check; no external state.
    #[test]
    fn tick_adds_niche_mortality_on_baseline_and_clamps() {
        let pheno = Phenotype {
            morphology: Morphology {
                height_cm: 0,
                body_color_hue: 0,
                leg_count: 0,
                arm_count: 0,
                eye_count: 0,
            },
            behavior: BehaviorWeights {
                aggression: 0.0,
                curiosity: 0.0,
                sociability: 0.0,
                intelligence: 0.0,
            },
        };
        // Niche optima at the opposite end → match_score ≈ 0.
        let niche = Niche::new(
            Morphology {
                height_cm: 255,
                body_color_hue: 255,
                leg_count: 255,
                arm_count: 255,
                eye_count: 255,
            },
            BehaviorWeights {
                aggression: 1.0,
                curiosity: 1.0,
                sociability: 1.0,
                intelligence: 1.0,
            },
            NicheWeights::uniform(),
        );

        let start = 1_000_u64;
        let tick = tick_niche_adaptation(
            start, &pheno, &niche, 0.0, 0.1, 0.1, // niche_mortality = 0.1 × 1.0
        );
        assert_eq!(tick.niche_mortality, 0.1);
        // baseline=0 + niche_mortality=0.1 → total_mortality = 0.1.
        // deaths = floor(0.1 × 1000) = 100.
        assert_eq!(tick.deaths, 100);
        // births = 0 (match_score = 0 → 0 growth).
        assert_eq!(tick.births, 0);
        assert_eq!(tick.population_after, 900);

        // Saturation: very high baseline + very high niche mortality
        // still clamps to total_mortality = 1.0.
        let tick2 = tick_niche_adaptation(start, &pheno, &niche, 0.95, 0.05, 0.5);
        // deaths = floor(1.0 × 1000) = 1000.
        assert_eq!(tick2.deaths, 1_000);
        assert_eq!(tick2.population_after, 0);

        // Edge case: empty population ⇒ no births, no deaths.
        let empty = tick_niche_adaptation(0, &pheno, &niche, 0.5, 0.1, 0.5);
        assert_eq!(empty.population_after, 0);
    }

    /// Covers FR-CIV-NICHE-ADAPT — `tick_niche_adaptation` is
    /// deterministic: identical inputs yield identical breakdowns. Two
    /// runs from the same state must produce the same `NicheTick`.
    #[test]
    fn tick_is_deterministic_for_same_inputs() {
        let pheno = phenotype_with(
            Morphology {
                height_cm: 80,
                body_color_hue: 120,
                leg_count: 4,
                arm_count: 2,
                eye_count: 2,
            },
            BehaviorWeights {
                aggression: 0.4,
                curiosity: 0.5,
                sociability: 0.6,
                intelligence: 0.7,
            },
        );
        let niche = Niche::new(
            Morphology {
                height_cm: 100,
                body_color_hue: 100,
                leg_count: 4,
                arm_count: 2,
                eye_count: 2,
            },
            BehaviorWeights {
                aggression: 0.5,
                curiosity: 0.5,
                sociability: 0.5,
                intelligence: 0.5,
            },
            NicheWeights::uniform(),
        );
        let a = tick_niche_adaptation(2_000, &pheno, &niche, 0.05, 0.1, 0.1);
        let b = tick_niche_adaptation(2_000, &pheno, &niche, 0.05, 0.1, 0.1);
        assert_eq!(a, b);
    }
}
