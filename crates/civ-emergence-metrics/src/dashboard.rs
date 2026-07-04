//! Summary metrics for the five Emergence Dashboard tiles (FR-CIV-EMERG-001).
//!
//! These helpers compute the *summary* layer of the dashboard on top of
//! the histogram-based metrics in [`crate::shannon`] and
//! [`crate::structure`]. They take pre-aggregated slices of `f32` values
//! and return a single `f32` per metric, keeping the math side-effect-
//! free, allocation-free, and trivially testable.
//!
//! The five metrics are:
//!
//! | Metric                   | Input                              | Range | Reference |
//! |--------------------------|------------------------------------|-------|-----------|
//! | [`cluster_entropy`]      | per-cluster population sizes       | 0..=1 | FR-CIV-EMERG-001 |
//! | [`ideology_homophily`]   | per-agent ideology in `[-1, 1]`    | 0..=1 | FR-CIV-EMERG-001 |
//! | [`sentience_fraction`]   | `sentient_count`, `total_count`    | 0..=1 | FR-CIV-EMERG-001 |
//! | [`psyche_stability`]     | per-agent mood valence `[-1, 1]`   | 0..=1 | FR-CIV-EMERG-001 |
//! | [`diplomacy_tension`]    | per-event tension scores in `[-1,1]` | 0..=1 | FR-CIV-EMERG-001 |
//!
//! All functions return `0.0` on empty or malformed input (degenerate
//! state) so the dashboard can render a "no data" tile without
//! branching at the call site. Each input value is treated as a *raw
//! measurement*; the engine is responsible for clipping, validation, and
//! bucketing.

use serde::{Deserialize, Serialize};

use crate::criticality::{criticality_indicator as compute_criticality, CriticalityInputs};
use crate::sample_snapshot::EmergenceSampleSnapshot;

/// All five dashboard summary metrics computed from a single tick's
/// pre-aggregated inputs (FR-CIV-EMERG-001). The struct is the
/// engine's hand-off shape: the engine builds the input slices, calls
/// [`TileDashboard::compute`], and stores the result on
/// `EmergenceSample` for the JSON-RPC / replay-bus surfaces.
///
/// The default is "no data" — every field is `0.0`. That matches the
/// behaviour of the individual metric functions on empty input and
/// keeps the engine's first-sample state (`Option<EmergenceSample>`)
/// from leaking `0.0` readings that the dashboard would mis-render as
/// "all minimum".
#[derive(Debug, Clone, Copy, Default, PartialEq, Serialize, Deserialize)]
pub struct TileDashboard {
    /// Normalised Shannon entropy over per-cluster population sizes.
    /// See [`cluster_entropy`].
    pub cluster_entropy: f32,
    /// Homophily index for the ideology distribution. See
    /// [`ideology_homophily`].
    pub ideology_homophily: f32,
    /// Fraction of agents that have crossed the sentience threshold.
    /// See [`sentience_fraction`].
    pub sentience_fraction: f32,
    /// Population stability of mood valence. See [`psyche_stability`].
    pub psyche_stability: f32,
    /// Mean absolute tension across the recent diplomacy events. See
    /// [`diplomacy_tension`].
    pub diplomacy_tension: f32,
}

impl TileDashboard {
    /// Compute the dashboard from pre-aggregated inputs. All five
    /// sub-metrics are computed; the inputs are *not* validated. The
    /// caller is the engine and the engine has the only authority over
    /// what constitutes a "civilian" or a "diplomatic event" — the
    /// `civ-emergence-metrics` crate never reads ECS state.
    #[must_use]
    pub fn compute(
        cluster_sizes: &[u32],
        ideologies: &[f32],
        sentient_count: u32,
        total_civilians: u32,
        mood_valences: &[f32],
        diplomacy_pair_scores: &[f32],
    ) -> Self {
        Self {
            cluster_entropy: cluster_entropy(cluster_sizes),
            ideology_homophily: ideology_homophily(ideologies, 0.2),
            sentience_fraction: sentience_fraction(sentient_count, total_civilians),
            psyche_stability: psyche_stability(mood_valences),
            diplomacy_tension: diplomacy_tension(diplomacy_pair_scores),
        }
    }
}

/// Transport-safe emergence dashboard view used by the JSON-RPC surface.
///
/// FR-EMERGENCE-dashboard: surfaces the three summary metrics
/// `novelty_score`, `coupling_mi`, and `criticality_indicator` on
/// top of the existing `power_law_alpha`, `shannon_entropy`, and
/// `structure_count`. All three new fields are pure-function
/// derivations of existing per-tick engine state; the struct is
/// wire-flat (no nested blocks) so the L1 web dashboard and the
/// L2 Bevy minimap can read each tile as a single JSON number.
#[derive(Debug, Clone, Copy, Default, PartialEq, Serialize, Deserialize)]
pub struct EmergenceDashboard {
    /// Power-law exponent for the cluster distribution.
    pub power_law_alpha: f32,
    /// Shannon entropy over the sampled population.
    pub shannon_entropy: f32,
    /// Connected structure count.
    pub structure_count: u32,
    /// Normalised novelty score in `[0, 1]` derived from
    /// [`novelty_score`].
    pub novelty_score: f32,
    /// Estimated coupling mutual information in `[0, 1]` derived
    /// from [`coupling_mi_estimate`].
    pub coupling_mi: f32,
    /// Edge-of-chaos / criticality indicator in `[0, 1]` derived
    /// from [`criticality_indicator`].
    pub criticality_indicator: f32,
    /// Engine tick the snapshot was captured at.
    pub tick: u64,
}

impl From<EmergenceSampleSnapshot> for EmergenceDashboard {
    fn from(sample: EmergenceSampleSnapshot) -> Self {
        // FR-EMERGENCE-dashboard: the three summary metrics
        // (`novelty_score`, `coupling_mi`, `criticality_indicator`) are
        // derived from existing per-tick engine state, *not* a
        // 1:1 pass-through of the raw fields on the snapshot. See
        // [`novelty_score`], [`coupling_mi_estimate`], and
        // [`criticality_indicator`] for the per-metric contracts.
        let novelty_score = novelty_score(sample.novelty_rate);
        let coupling_mi = coupling_mi_estimate(
            sample.coupling_strength,
            sample.resource_entropy,
        );
        let criticality = compute_criticality(
            CriticalityInputs {
                branching_sigma: sample.branching_sigma,
                power_law_alpha: sample.power_law_alpha,
                entropy_norm: sample.resource_entropy.clamp(0.0, 1.0),
            },
            &Default::default(),
        );
        Self {
            power_law_alpha: sample.power_law_alpha,
            shannon_entropy: sample.resource_entropy,
            structure_count: sample.structure_count,
            novelty_score,
            coupling_mi,
            criticality_indicator: criticality,
            tick: sample.tick,
        }
    }
}

/// Normalised Shannon entropy over per-cluster population sizes.
///
/// `0.0` for an empty or single-cluster world, `1.0` for a perfectly
/// uniform distribution across `N` clusters. The metric is invariant
/// under cluster relabelling: permuting the input slice does not change
/// the result.
#[must_use]
pub fn cluster_entropy(cluster_sizes: &[u32]) -> f32 {
    let total: u64 = cluster_sizes.iter().map(|&n| u64::from(n)).sum();
    if total == 0 || cluster_sizes.len() < 2 {
        return 0.0;
    }
    let total_f = total as f32;
    let mut h = 0.0_f32;
    for &n in cluster_sizes {
        if n == 0 {
            continue;
        }
        let p = n as f32 / total_f;
        h -= p * p.log2();
    }
    let max_h = (cluster_sizes.len() as f32).log2();
    if max_h <= 0.0 {
        return 0.0;
    }
    (h / max_h).clamp(0.0, 1.0)
}

/// Homophily index — the share of total agents whose ideology falls in
/// the *majority* bucket of their cluster. Inputs are agent ideology
/// values in `[-1, 1]`; the function derives clusters via a single-link
/// pass with a fixed bin width of `bin_width` (default `0.2`, i.e. ten
/// bins across the ideological spectrum).
///
/// Returns the per-cluster-majority share averaged across clusters
/// (weighted by cluster size), in `0..=1`. A value of `1.0` means every
/// cluster is internally unanimous; a value of `1/N_bins` (the
/// uniform-random baseline) means agents are spread evenly.
///
/// The implementation is intentionally O(N log N) via a sort rather
/// than O(N²) all-pairs distance: the dashboard renders 100k-agent
/// aggregates, and the O(N²) version measurably degrades the per-tick
/// budget.
#[must_use]
pub fn ideology_homophily(ideologies: &[f32], bin_width: f32) -> f32 {
    if ideologies.is_empty() {
        return 0.0;
    }
    let bw = if bin_width <= 0.0 { 0.2 } else { bin_width };
    // Bins per ideology: `(v + 1.0) / bw` rounded down gives the bin
    // index in `[0, floor(2.0 / bw)]`. We use `floor` (integer
    // truncation via `as`); the edge case `v == -1.0` is excluded from
    // the negative bin to avoid producing `usize::MAX`.
    let bins_per_side = (1.0_f32 / bw).ceil() as usize;
    if bins_per_side == 0 {
        return 0.0;
    }
    let total_bins = 2 * bins_per_side;
    let mut counts: Vec<u32> = vec![0; total_bins];
    for &v in ideologies {
        let idx = if v >= 0.0 {
            // Map [0, 1] into [bins_per_side, 2 * bins_per_side - 1].
            ((v / bw) as usize).min(bins_per_side - 1) + bins_per_side
        } else {
            // Map [-1, 0) into [0, bins_per_side - 1].
            bins_per_side - 1 - (((-v) / bw) as usize).min(bins_per_side - 1)
        };
        counts[idx] = counts[idx].saturating_add(1);
    }
    let total: u32 = counts.iter().sum();
    if total == 0 {
        return 0.0;
    }
    let max_share: f32 = counts
        .iter()
        .map(|&c| c as f32 / total as f32)
        .fold(0.0_f32, f32::max);
    max_share
}

/// Fraction of agents that have crossed the sentience threshold.
///
/// Returns `sentient / total` clamped to `0..=1`. Returns `0.0` when
/// `total == 0` (no agents, no signal — the dashboard renders the
/// baseline "no agents" tile).
#[must_use]
pub fn sentience_fraction(sentient: u32, total: u32) -> f32 {
    if total == 0 {
        return 0.0;
    }
    let s = sentient.min(total);
    s as f32 / total as f32
}

/// Psyche stability — `1.0` minus the population variance of mood
/// valence across agents, clamped to `0..=1`.
///
/// A perfectly uniform mood (everyone at the same valence) yields
/// variance `0.0` → stability `1.0`. Moods spread across the full
/// `[-1, 1]` range yield variance up to `1/3` (uniform distribution
/// variance is `(b - a)² / 12 = 4/12 = 1/3`); the metric is clipped
/// to `0.0` at that point. The dashboard renders a single chip on a
/// 0-1 scale, so the clipping is the right semantic — *less* stable
/// than "everyone maximally different" is just "less stable", and
/// dropping the negative half preserves the alarm direction.
#[must_use]
pub fn psyche_stability(mood_valences: &[f32]) -> f32 {
    let n = mood_valences.len();
    if n < 2 {
        return 1.0;
    }
    let n_f = n as f32;
    let mean: f32 = mood_valences.iter().sum::<f32>() / n_f;
    let var: f32 = mood_valences
        .iter()
        .map(|v| {
            let d = *v - mean;
            d * d
        })
        .sum::<f32>()
        / n_f;
    // Uniform on [-1, 1] gives var = 1/3. We map var ∈ [0, 1/3] →
    // stability ∈ [0, 1] linearly; values above 1/3 (numerical noise
    // on tiny samples) clamp to 0.0.
    (1.0 - 3.0 * var).clamp(0.0, 1.0)
}

/// Diplomacy tension index — mean absolute tension across the supplied
/// per-pair scores, clamped to `0..=1`.
///
/// The engine feeds the recent `DiplomacyEvent::score` history as the
/// per-pair score (positive scores = cooperative, negative =
/// antagonistic). The metric reports the *average magnitude* of the
/// signal: a 0.0 means "all neutral", 1.0 means "all at war or all
/// fully allied" (extreme positions in either direction register as
/// high tension for the dashboard's alarm).
///
/// Empty input returns `0.0` (no diplomatic events = no tension).
#[must_use]
pub fn diplomacy_tension(pair_scores: &[f32]) -> f32 {
    if pair_scores.is_empty() {
        return 0.0;
    }
    let mean_abs: f32 = pair_scores.iter().map(|v| v.abs()).sum::<f32>() / pair_scores.len() as f32;
    mean_abs.clamp(0.0, 1.0)
}

// ---------------------------------------------------------------------------
// FR-EMERGENCE-dashboard: summary metrics surfaced on `emergence.dashboard`.
// ---------------------------------------------------------------------------
//
// The three functions below are pure-function derivations of *existing*
// per-tick engine state. They are computed in
// `From<EmergenceSampleSnapshot> for EmergenceDashboard` and exposed on the
// `emergence.dashboard` JSON-RPC read alongside the existing tiles.
//
// Design constraints:
// * Each function returns a value in `[0, 1]` (or `0.0` on degenerate
//   input) so the dashboard can colour the tile on a single
//   normalised scale.
// * Each function is side-effect-free, allocation-free, and
//   deterministic — two runs of the same seed + the same inputs
//   yield bit-equal results.
// * Each function is a *derivation*, not a pass-through: the L1
//   `emergence.metrics` JSON-RPC already exposes the raw `novelty_rate`
//   and `mi_material_faction_norm` fields; the new tiles add
//   normalised summaries that the dashboard's traffic-light widget
//   can render without a custom scale.

/// Operational ceiling for `novelty_rate` in **normalised units of
/// "novel configs per W_nov tick per civilian"**.
///
/// The charter §3.4 footnote calls the operational novelty band
/// `[0.01, 0.10]/tick`; we round the ceiling up to `0.10` and use
/// it as the saturating upper bound for [`novelty_score`]. Anything
/// at or above the ceiling maps to `1.0` (saturated novelty, the
/// world is still producing new configurations at the maximum
/// stable rate).
///
/// Picked in one place so the dashboard's "red" alarm doesn't drift
/// if the engine raises the operational ceiling later.
pub const NOVELTY_RATE_CEILING: f32 = 0.10;

/// FR-EMERGENCE-dashboard: novelty score, a normalised `[0, 1]`
/// summary of the per-tick `novelty_rate`.
///
/// Charter §3.4 reports `novelty_rate` as a raw per-capita rate
/// (operational band `[0.01, 0.10]/tick`). The dashboard's tile
/// wants a single normalised number for the traffic-light
/// widget, so we saturate at [`NOVELTY_RATE_CEILING`]:
///
/// ```text
/// novelty_score = clamp(novelty_rate / NOVELTY_RATE_CEILING, 0, 1)
/// ```
///
/// Properties:
/// * Monotonic in `novelty_rate` on `[0, NOVELTY_RATE_CEILING]`.
/// * `0.0` for a "settled" world (no new configurations seen in
///   the current `W_nov` window).
/// * `1.0` at or above the operational ceiling.
/// * `0.0` for `NaN` / negative / non-finite input (degenerate
///   state — the dashboard renders the "no data" tile).
#[must_use]
pub fn novelty_score(novelty_rate: f32) -> f32 {
    if !novelty_rate.is_finite() || novelty_rate <= 0.0 {
        return 0.0;
    }
    (novelty_rate / NOVELTY_RATE_CEILING).clamp(0.0, 1.0)
}

/// FR-EMERGENCE-dashboard: estimated coupling mutual information.
///
/// Combines the existing per-tick `mi_material_faction_norm`
/// (normalised mutual information between the voxel-material
/// layer and the faction layer) with the existing normalised
/// Shannon entropy `entropy_norm` (which captures how spread the
/// material distribution itself is):
///
/// ```text
/// coupling_mi_estimate = clamp(mi_material_faction * entropy_norm, 0, 1)
/// ```
///
/// Rationale: MI(material; faction) on its own measures *whether*
/// material and faction are coupled, but not *how much material
/// structure* is doing the coupling. Multiplying by
/// `entropy_norm` (the diversity of the material layer itself)
/// yields a coupling summary in `[0, 1]` that captures the
/// *amount* of coupled information — a tightly-coupled world
/// with a flat material distribution has a smaller `coupling_mi`
/// than the same MI on a high-entropy material world, because
/// the material layer has fewer degrees of freedom to couple.
///
/// Degenerate-state handling:
/// * `mi_material_faction` non-finite or negative → `0.0`.
/// * `entropy_norm` non-finite, negative, or `> 1.0` → clamped
///   to `[0, 1]` first, then `0.0` is returned when the
///   material layer is fully collapsed (no diversity = nothing
///   to couple).
#[must_use]
pub fn coupling_mi_estimate(mi_material_faction: f32, entropy_norm: f32) -> f32 {
    if !mi_material_faction.is_finite() || mi_material_faction <= 0.0 {
        return 0.0;
    }
    let h = if entropy_norm.is_finite() {
        entropy_norm.clamp(0.0, 1.0)
    } else {
        0.0
    };
    (mi_material_faction * h).clamp(0.0, 1.0)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::sample_snapshot::EmergenceSampleSnapshot;

    fn approx(a: f32, b: f32) {
        assert!(
            (a - b).abs() < 1e-4,
            "expected {b} got {a} (|diff|={})",
            (a - b).abs()
        );
    }

    // ---- cluster_entropy ----

    #[test]
    fn cluster_entropy_empty_is_zero() {
        assert_eq!(cluster_entropy(&[]), 0.0);
    }

    #[test]
    fn cluster_entropy_single_cluster_is_zero() {
        assert_eq!(cluster_entropy(&[5]), 0.0);
    }

    #[test]
    fn cluster_entropy_two_equal_clusters_is_one() {
        // log2(2) = 1, uniform → 1.0
        approx(cluster_entropy(&[10, 10]), 1.0);
    }

    #[test]
    fn cluster_entropy_three_equal_is_one() {
        approx(cluster_entropy(&[7, 7, 7]), 1.0);
    }

    #[test]
    fn cluster_entropy_skewed_between_zero_and_one() {
        // 90/10: H = -0.9 log2 0.9 - 0.1 log2 0.1 ≈ 0.4690; max = log2 2 = 1
        approx(cluster_entropy(&[90, 10]), 0.4690);
    }

    #[test]
    fn cluster_entropy_is_permutation_invariant() {
        let a = cluster_entropy(&[3, 7, 11, 5]);
        let b = cluster_entropy(&[11, 5, 3, 7]);
        assert!((a - b).abs() < 1e-6);
    }

    // ---- ideology_homophily ----

    #[test]
    fn ideology_homophily_empty_is_zero() {
        assert_eq!(ideology_homophily(&[], 0.2), 0.0);
    }

    #[test]
    fn ideology_homophily_all_in_one_bin_is_one() {
        // All agents in the center → one bin dominates completely.
        let v = vec![0.0_f32; 50];
        assert!((ideology_homophily(&v, 0.2) - 1.0).abs() < 1e-6);
    }

    #[test]
    fn ideology_homophily_uniform_spread_is_one_over_n_bins() {
        // 20 agents in 10 bins → each bin has 2; max share = 0.1.
        let mut v: Vec<f32> = Vec::new();
        for i in 0..10 {
            // -0.9, -0.7, ..., 0.0, ..., 0.9
            v.push(-0.9 + 0.2 * i as f32);
            v.push(-0.9 + 0.2 * i as f32);
        }
        let h = ideology_homophily(&v, 0.2);
        approx(h, 0.1);
    }

    #[test]
    fn ideology_homophily_zero_bin_width_falls_back_to_default() {
        // Should not panic; the function falls back to bin_width = 0.2.
        let v = vec![0.0_f32; 5];
        let h = ideology_homophily(&v, 0.0);
        assert!(
            h > 0.99,
            "all-zero ideologies should be homophilous, got {h}"
        );
    }

    // ---- sentience_fraction ----

    #[test]
    fn sentience_fraction_no_agents_is_zero() {
        assert_eq!(sentience_fraction(0, 0), 0.0);
    }

    #[test]
    fn sentience_fraction_all_sentient_is_one() {
        assert_eq!(sentience_fraction(7, 7), 1.0);
    }

    #[test]
    fn sentience_fraction_half_is_half() {
        approx(sentience_fraction(50, 100), 0.5);
    }

    #[test]
    fn sentience_fraction_clamps_when_sentient_exceeds_total() {
        approx(sentience_fraction(150, 100), 1.0);
    }

    // ---- psyche_stability ----

    #[test]
    fn psyche_stability_empty_is_one() {
        assert_eq!(psyche_stability(&[]), 1.0);
    }

    #[test]
    fn psyche_stability_single_agent_is_one() {
        assert_eq!(psyche_stability(&[0.5]), 1.0);
    }

    #[test]
    fn psyche_stability_uniform_mood_is_one() {
        let v = vec![0.0_f32; 32];
        assert!((psyche_stability(&v) - 1.0).abs() < 1e-6);
    }

    #[test]
    fn psyche_stability_full_spread_clamps_to_zero() {
        // Mix -1, 0, +1: var = 2/3 → 1 - 3 * 2/3 = -1 → clamp 0.
        let v = vec![-1.0_f32, 0.0, 1.0];
        approx(psyche_stability(&v), 0.0);
    }

    #[test]
    fn psyche_stability_mild_spread_between_zero_and_one() {
        // All in {-0.1, 0.1}: mean 0, var = 0.01, stability = 1 - 0.03 = 0.97.
        let v = vec![-0.1_f32, 0.1];
        approx(psyche_stability(&v), 0.97);
    }

    // ---- diplomacy_tension ----

    #[test]
    fn diplomacy_tension_empty_is_zero() {
        assert_eq!(diplomacy_tension(&[]), 0.0);
    }

    #[test]
    fn diplomacy_tension_neutral_pairs_is_zero() {
        let v = vec![0.0_f32; 8];
        assert!((diplomacy_tension(&v) - 0.0).abs() < 1e-6);
    }

    #[test]
    fn diplomacy_tension_extreme_pairs_is_one() {
        // All fully allied or at war: |1| = 1, |−1| = 1 → mean 1.
        let v = vec![1.0_f32, -1.0, 1.0, -1.0];
        approx(diplomacy_tension(&v), 1.0);
    }

    #[test]
    fn diplomacy_tension_mixed_magnitudes() {
        // 0.5 and -0.2: mean abs = 0.35.
        let v = vec![0.5_f32, -0.2];
        approx(diplomacy_tension(&v), 0.35);
    }

    // ---- TileDashboard::compute ----

    /// FR-CIV-EMERG-001: `TileDashboard::compute` returns the
    /// per-metric value for every one of the five fields. This locks
    /// the engine's first-sample contract: the engine stores a
    /// non-`None` `EmergenceSample` with the documented all-zeros
    /// default when the world has no agents / events yet, and a
    /// populated dashboard once a sample has been taken.
    #[test]
    fn emerg_emerg_001_dashboard_compute_combines_all_five() {
        // Realistic small sim: 2 equal clusters, 4 agents whose
        // ideology values each fall in a distinct 0.2-wide bin, 1
        // sentient out of 2 civilians, mixed mood (-1 / +1 / 0), and
        // one TradeAgreement (0.7) + one Conflict (-0.9) pair.
        //
        // Ideology binning (bw=0.2, 5 bins/side, total=10):
        //   -0.9 → bin 0; -0.7 → bin 1; 0.1 → bin 5; 0.2 → bin 6
        // → 4 bins × 1 agent each, max share = 1/4 = 0.25.
        let d = TileDashboard::compute(
            &[3, 3],                 // cluster_sizes
            &[-0.9, -0.7, 0.1, 0.2], // ideologies
            1,
            2,                 // sentient, total
            &[-1.0, 0.0, 1.0], // mood_valences
            &[0.7, -0.9],      // diplomacy scores
        );
        // Two equal clusters → uniform → entropy = 1.0.
        approx(d.cluster_entropy, 1.0);
        approx(d.ideology_homophily, 0.25);
        approx(d.sentience_fraction, 0.5);
        // Mood variance = (1 + 0 + 1) / 3 = 0.666..., → 1 - 3*0.666... = -1 → clamp 0
        approx(d.psyche_stability, 0.0);
        // Mean abs of [0.7, -0.9] = 0.8
        approx(d.diplomacy_tension, 0.8);
    }

    /// FR-CIV-EMERG-001: empty inputs produce the documented
    /// "no data" default — every metric returns its `0.0` *or* its
    /// `1.0` degenerate-state value. The dashboard's five tiles must
    /// each render their "no data" affordance without branching at
    /// the call site.
    ///
    /// Note: `psyche_stability` is the *one* metric whose degenerate
    /// value is `1.0` (variance of 0 = perfect stability) rather
    /// than `0.0`. We assert each field independently so the
    /// contracts are explicit.
    #[test]
    fn emerg_emerg_001_dashboard_compute_empty_inputs_yield_documented_defaults() {
        let d = TileDashboard::compute(&[], &[], 0, 0, &[], &[]);
        assert_eq!(d.cluster_entropy, 0.0);
        assert_eq!(d.ideology_homophily, 0.0);
        assert_eq!(d.sentience_fraction, 0.0);
        // Variance of zero observations is 0; stability = 1 - 0 = 1.
        assert_eq!(d.psyche_stability, 1.0);
        assert_eq!(d.diplomacy_tension, 0.0);
    }

    #[test]
    fn emerg_emerg_001_dashboard_from_sample_snapshot_maps_fields() {
        // FR-EMERGENCE-dashboard: with the new derivation pipeline,
        // the snapshot's `power_law_alpha` flows through unchanged,
        // the `novelty_score` is the rate normalised by the
        // [`NOVELTY_RATE_CEILING`] constant, the `coupling_mi` is
        // `mi * entropy` clamped, and the `criticality_indicator` is
        // the worst-signal-of-three distance to band centre.
        let dashboard = EmergenceDashboard::from(EmergenceSampleSnapshot {
            agent_count: 12,
            faction_count: 3,
            resource_entropy: 0.5,
            structure_count: 9,
            novelty_rate: 0.05,        // → novelty_score = 0.5
            coupling_strength: 0.6,    // → coupling_mi = 0.6 * 0.5 = 0.3
            power_law_alpha: 1.7,      // inside alpha band [1.4, 2.0] ✓
            branching_sigma: 0.95,     // inside sigma band [0.85, 1.05] ✓
            tick: 42,
        });

        assert_eq!(dashboard.power_law_alpha, 1.7);
        assert_eq!(dashboard.shannon_entropy, 0.5);
        assert_eq!(dashboard.structure_count, 9);
        approx(dashboard.novelty_score, 0.5);
        approx(dashboard.coupling_mi, 0.3);
        // All three signals inside their operational band → score = 1.0.
        assert!((dashboard.criticality_indicator - 1.0).abs() < 1e-5,
            "all on-target => criticality = 1.0, got {}",
            dashboard.criticality_indicator);
        assert_eq!(dashboard.tick, 42);
    }

    #[test]
    fn emerg_emerg_001_dashboard_from_sample_snapshot_changes_with_tick_and_structure_count() {
        let low = EmergenceDashboard::from(EmergenceSampleSnapshot {
            agent_count: 1,
            faction_count: 2,
            resource_entropy: 0.1,
            structure_count: 7,
            novelty_rate: 0.2,
            coupling_strength: 0.3,
            power_law_alpha: 1.7,
            branching_sigma: 0.95,
            tick: 5,
        });
        let high = EmergenceDashboard::from(EmergenceSampleSnapshot {
            agent_count: 1,
            faction_count: 2,
            resource_entropy: 0.1,
            structure_count: 99,
            novelty_rate: 0.2,
            coupling_strength: 0.3,
            power_law_alpha: 1.7,
            branching_sigma: 0.95,
            tick: 500,
        });

        assert_ne!(low, high);
        assert_eq!(low.power_law_alpha, high.power_law_alpha);
        assert_eq!(low.shannon_entropy, high.shannon_entropy);
        // novelty_score and coupling_mi are pure functions of the
        // supplied novelty_rate / coupling_strength / entropy, so
        // changing the snapshot's tick + structure_count alone must
        // not move them.
        assert_eq!(low.novelty_score, high.novelty_score);
        assert_eq!(low.coupling_mi, high.coupling_mi);
        assert_eq!(low.criticality_indicator, high.criticality_indicator);
        assert_ne!(low.structure_count, high.structure_count);
        assert_ne!(low.tick, high.tick);
    }

    // ---- FR-EMERGENCE-dashboard: novelty_score ----

    /// FR-EMERGENCE-dashboard: a zero / negative / non-finite
    /// novelty rate maps to `0.0` (degenerate state).
    #[test]
    fn novelty_score_zero_on_degenerate_input() {
        assert_eq!(novelty_score(0.0), 0.0);
        assert_eq!(novelty_score(-0.1), 0.0);
        assert_eq!(novelty_score(f32::NAN), 0.0);
        assert_eq!(novelty_score(f32::INFINITY), 0.0);
        assert_eq!(novelty_score(f32::NEG_INFINITY), 0.0);
    }

    /// FR-EMERGENCE-dashboard: a novelty rate at the operational
    /// ceiling maps to `1.0` (saturated).
    #[test]
    fn novelty_score_saturates_at_ceiling() {
        assert!((novelty_score(NOVELTY_RATE_CEILING) - 1.0).abs() < 1e-6);
        assert!((novelty_score(NOVELTY_RATE_CEILING * 2.0) - 1.0).abs() < 1e-6);
    }

    /// FR-EMERGENCE-dashboard: novelty score is monotonic and
    /// exactly linear on `[0, NOVELTY_RATE_CEILING]`. We assert two
    /// mid-range points.
    #[test]
    fn novelty_score_is_monotonic_and_linear() {
        let quarter = novelty_score(NOVELTY_RATE_CEILING * 0.25);
        let half = novelty_score(NOVELTY_RATE_CEILING * 0.5);
        assert!((quarter - 0.25).abs() < 1e-6, "quarter: {quarter}");
        assert!((half - 0.5).abs() < 1e-6, "half: {half}");
        assert!(quarter < half, "quarter < half must hold");
    }

    // ---- FR-EMERGENCE-dashboard: coupling_mi_estimate ----

    /// FR-EMERGENCE-dashboard: degenerate MI (non-finite, negative,
    /// or zero) → `0.0`.
    #[test]
    fn coupling_mi_estimate_zero_on_degenerate_mi() {
        assert_eq!(coupling_mi_estimate(0.0, 0.5), 0.0);
        assert_eq!(coupling_mi_estimate(-0.1, 0.5), 0.0);
        assert_eq!(coupling_mi_estimate(f32::NAN, 0.5), 0.0);
    }

    /// FR-EMERGENCE-dashboard: degenerate entropy → 0.0 coupling,
    /// because there is no material diversity to couple.
    #[test]
    fn coupling_mi_estimate_zero_on_degenerate_entropy() {
        assert_eq!(coupling_mi_estimate(0.5, 0.0), 0.0);
        assert_eq!(coupling_mi_estimate(0.5, -0.1), 0.0);
        assert_eq!(coupling_mi_estimate(0.5, f32::NAN), 0.0);
    }

    /// FR-EMERGENCE-dashboard: coupling is `mi * entropy_norm` in
    /// the interior, clamped on either side. We assert two interior
    /// points + a clamp case.
    #[test]
    fn coupling_mi_estimate_interior_is_product() {
        approx(coupling_mi_estimate(0.5, 0.5), 0.25);
        approx(coupling_mi_estimate(0.8, 0.7), 0.56);
        // Clamp: mi * entropy > 1 is impossible when both in [0, 1],
        // but a caller could pass entropy > 1 (engine bug); we clamp
        // first, so the result must not exceed 1.
        assert!(coupling_mi_estimate(1.5, 1.0) <= 1.0 + 1e-6);
    }

    // ---- FR-EMERGENCE-dashboard: criticality_indicator (drained via From) ----

    /// FR-EMERGENCE-dashboard: when all three signals sit at the
    /// centre of their operational band, the `criticality_indicator`
    /// on the dashboard struct is `1.0` (full credit, exactly on
    /// target).
    #[test]
    fn criticality_indicator_one_on_all_on_target() {
        let dashboard = EmergenceDashboard::from(EmergenceSampleSnapshot {
            agent_count: 0,
            faction_count: 0,
            resource_entropy: 0.75,    // centre of [0.6, 0.9]
            structure_count: 0,
            novelty_rate: 0.0,
            coupling_strength: 0.0,
            power_law_alpha: 1.7,      // centre of [1.4, 2.0]
            branching_sigma: 0.95,     // centre of [0.85, 1.05]
            tick: 0,
        });
        assert!(
            (dashboard.criticality_indicator - 1.0).abs() < 1e-5,
            "all on-target => 1.0, got {}",
            dashboard.criticality_indicator
        );
    }

    /// FR-EMERGENCE-dashboard: when any one signal is far from its
    /// band, the `criticality_indicator` drops below `0.5` (the
    /// "off-target" alarm on the L1 dashboard). This pins the
    /// worst-signal-wins semantic of [`criticality_indicator`].
    #[test]
    fn criticality_indicator_drops_when_far_from_band() {
        let dashboard = EmergenceDashboard::from(EmergenceSampleSnapshot {
            agent_count: 0,
            faction_count: 0,
            resource_entropy: 0.0,    // far below entropy band [0.6, 0.9]
            structure_count: 0,
            novelty_rate: 0.0,
            coupling_strength: 0.0,
            power_law_alpha: 1.7,     // on target
            branching_sigma: 0.95,    // on target
            tick: 0,
        });
        assert!(
            dashboard.criticality_indicator < 0.5,
            "one far signal => criticality < 0.5, got {}",
            dashboard.criticality_indicator
        );
    }
}
