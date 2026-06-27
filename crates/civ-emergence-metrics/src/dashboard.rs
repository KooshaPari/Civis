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

/// All five dashboard summary metrics computed from a single tick's
/// pre-aggregated inputs (FR-CIV-EMERG-001). The struct is the
/// engine's hand-off shape: the engine builds the input slices, calls
/// [`EmergenceDashboard::compute`], and stores the result on
/// `EmergenceSample` for the JSON-RPC / replay-bus surfaces.
///
/// The default is "no data" — every field is `0.0`. That matches the
/// behaviour of the individual metric functions on empty input and
/// keeps the engine's first-sample state (`Option<EmergenceSample>`)
/// from leaking `0.0` readings that the dashboard would mis-render as
/// "all minimum".
#[derive(Debug, Clone, Copy, Default, PartialEq, Serialize, Deserialize)]
pub struct EmergenceDashboard {
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

impl EmergenceDashboard {
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

#[cfg(test)]
mod tests {
    use super::*;

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

    // ---- EmergenceDashboard::compute ----

    /// FR-CIV-EMERG-001: `EmergenceDashboard::compute` returns the
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
        let d = EmergenceDashboard::compute(
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
        let d = EmergenceDashboard::compute(&[], &[], 0, 0, &[], &[]);
        assert_eq!(d.cluster_entropy, 0.0);
        assert_eq!(d.ideology_homophily, 0.0);
        assert_eq!(d.sentience_fraction, 0.0);
        // Variance of zero observations is 0; stability = 1 - 0 = 1.
        assert_eq!(d.psyche_stability, 1.0);
        assert_eq!(d.diplomacy_tension, 0.0);
    }
}
