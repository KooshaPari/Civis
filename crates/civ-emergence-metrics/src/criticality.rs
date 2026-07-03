//! Edge-of-chaos / criticality indicator for the emergence dashboard.
//!
//! Combines three existing per-tick signals into a single
//! "how close to the critical band are we?" indicator in `[0, 1]`:
//!
//! * [`branching_sigma`](CriticalityInputs::branching_sigma) — branching-process
//!   reproduction mean; the canonical critical band is `sigma == 1.0`
//!   (doob/galton-watson threshold). We treat `[0.85, 1.05]` as the
//!   operational "edge-of-chaos" band — this matches the dashboard charter
//!   §3.4 footnote: "operational criticality band `[0.85, 1.0]` for the
//!   loglog-power-law regime".
//! * [`power_law_alpha`](CriticalityInputs::power_law_alpha) — the
//!   power-law slope on event-size distribution; the operational band
//!   `[1.4, 2.0]` covers the "self-organised critical" 3/2-law
//!   (`1.5`) plus `1/f` noise envelope.
//! * [`entropy_norm`](CriticalityInputs::entropy_norm) — the Shannon
//!   entropy of the active material distribution normalised to its
//!   theoretical maximum for the active cell count; the critical band
//!   is `[0.6, 0.9]` — high enough to be diverse, low enough to still
//!   show structure.
//!
//! The output is `1.0` when all three signals sit exactly at the centre
//! of their bands and `0.0` when any one of them is "infinitely far"
//! from the band. The result is finite, bounded in `[0, 1]`, and
//! well-defined for any combination of finite inputs — including the
//! `0.0` defaults returned at tick `0` before any signal has stabilised.
//!
//! All three inputs already exist on [`crate::sample_snapshot::EmergenceSample`]
//! and on the JSON-RPC `emergence.metrics` read; this module is
//! **pure-function**, deterministic, and side-effect free. It does not
//! inspect any tick, ECS, or global state — see
//! [`docs/design/emergence-dashboard.md`] for the metric definitions.
//!
//! [`docs/design/emergence-dashboard.md`]: ../../../../docs/design/emergence-dashboard.md

/// Operational bands for each criticality input.
///
/// Defaults match the dashboard charter §3.4:
/// * branching sigma in `[0.85, 1.05]` (centred on the doob/galton-watson
///   threshold; this is the "edge-of-chaos" half-plane).
/// * power-law alpha in `[1.4, 2.0]` (centred on the self-organised
///   critical 3/2-law; the wider band covers 1/f noise envelope).
/// * entropy norm in `[0.6, 0.9]` (high diversity, but not so flat that
///   no structure remains).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CriticalityBands {
    /// Lower edge of the operational branching band.
    pub branching_lo: f32,
    /// Upper edge of the operational branching band.
    pub branching_hi: f32,
    /// Lower edge of the operational power-law slope band.
    pub alpha_lo: f32,
    /// Upper edge of the operational power-law slope band.
    pub alpha_hi: f32,
    /// Lower edge of the operational normalised-entropy band.
    pub entropy_lo: f32,
    /// Upper edge of the operational normalised-entropy band.
    pub entropy_hi: f32,
}

impl Default for CriticalityBands {
    fn default() -> Self {
        Self {
            branching_lo: 0.85,
            branching_hi: 1.05,
            alpha_lo: 1.4,
            alpha_hi: 2.0,
            entropy_lo: 0.6,
            entropy_hi: 0.9,
        }
    }
}

/// Inputs to [`criticality_indicator`].
///
/// All three fields already exist on the per-tick
/// [`crate::sample_snapshot::EmergenceSample`]. The struct is
/// `Copy` so the indicator can be called inside hot loops without
/// borrow-checker friction.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct CriticalityInputs {
    /// Branching-process reproduction mean (`sigma`) for this tick.
    /// Typically [`crate::branching::branching_mean`].
    pub branching_sigma: f32,
    /// Power-law slope (`alpha`) for the active event-size distribution.
    /// Typically [`crate::power_law::fit_power_law`].
    pub power_law_alpha: f32,
    /// Normalised Shannon entropy of the active material distribution.
    /// Typically [`crate::shannon::shannon_entropy_normalised`].
    pub entropy_norm: f32,
}

/// Distance to the centre of a half-open band `[lo, hi]`, expressed
/// as a *fraction of the band half-width*. Returns:
///
/// * `0.0` if `x` is inside the band,
/// * a positive number growing linearly with distance from the band
///   edge (in units of "half the band width"),
/// * `0.0` if the band is degenerate (`lo > hi`) — guards against
///   a downstream operator configuring a zero-width band.
fn distance_to_band(x: f32, lo: f32, hi: f32) -> f32 {
    if !(lo.is_finite() && hi.is_finite() && x.is_finite()) {
        // Non-finite input collapses to "infinitely far" -> score 0.
        return f32::INFINITY;
    }
    let half_width = (hi - lo) * 0.5;
    if half_width <= 0.0 || !half_width.is_finite() {
        return 0.0;
    }
    let centre = (lo + hi) * 0.5;
    let raw = (x - centre).abs() - half_width;
    if raw <= 0.0 {
        0.0
    } else {
        raw / half_width
    }
}

/// Edge-of-chaos / criticality indicator in `[0, 1]`.
///
/// Combines the three signals in [`CriticalityInputs`] into a single
/// summary score:
///
/// ```text
/// score = 1.0 / (1.0 + max(d_branching, d_alpha, d_entropy))
/// ```
///
/// where each `d_*` is the distance (in units of "band half-width")
/// from the centre of the corresponding operational band. With
/// this formulation:
///
/// * every signal inside its band => `score = 1.0`
/// * one signal one half-width outside its band => `score = 0.5`
/// * one signal many half-widths outside its band => `score ~= 0`
///
/// `NaN` / `±Inf` in any input collapse the score to `0.0` — the
/// tick is treated as "not measurable yet" and surfaced as a flat
/// trace. This matches the dashboard convention of never plotting
/// garbage and never asserting an `unreachable!` in the hot path.
///
/// # Examples
///
/// ```
/// use civ_emergence_metrics::{criticality_indicator, CriticalityInputs};
///
/// let inputs = CriticalityInputs {
///     branching_sigma: 1.0,    // exactly on the doob threshold
///     power_law_alpha: 1.5,    // the 3/2 self-organised-critical value
///     entropy_norm: 0.75,      // mid-band diversity
/// };
/// let score = criticality_indicator(inputs, &Default::default());
/// assert!((score - 1.0).abs() < 1e-6, "all on-target => 1.0, got {}", score);
/// ```
pub fn criticality_indicator(
    inputs: CriticalityInputs,
    bands: &CriticalityBands,
) -> f32 {
    let d_b = distance_to_band(inputs.branching_sigma, bands.branching_lo, bands.branching_hi);
    let d_a = distance_to_band(inputs.power_law_alpha, bands.alpha_lo, bands.alpha_hi);
    let d_e = distance_to_band(inputs.entropy_norm, bands.entropy_lo, bands.entropy_hi);

    if !d_b.is_finite() || !d_a.is_finite() || !d_e.is_finite() {
        return 0.0;
    }
    let worst = d_b.max(d_a).max(d_e);
    if worst <= 0.0 {
        return 1.0;
    }
    1.0 / (1.0 + worst)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn on_target() -> CriticalityInputs {
        CriticalityInputs {
            branching_sigma: 1.0,
            power_law_alpha: 1.5,
            entropy_norm: 0.75,
        }
    }

    #[test]
    fn on_target_inputs_yield_score_one() {
        let score = criticality_indicator(on_target(), &CriticalityBands::default());
        assert!(
            (score - 1.0).abs() < 1e-6,
            "all on-target => 1.0, got {}",
            score
        );
    }

    #[test]
    fn far_from_bands_yields_score_near_zero() {
        let inputs = CriticalityInputs {
            branching_sigma: 5.0,
            power_law_alpha: 10.0,
            entropy_norm: 0.0,
        };
        let score = criticality_indicator(inputs, &CriticalityBands::default());
        assert!(
            score < 0.1,
            "far-from-band => score near 0, got {}",
            score
        );
    }

    #[test]
    fn one_half_width_outside_band_halves_score() {
        // Move branching sigma one half-width past the band edge.
        // Band is [0.85, 1.05], half-width = 0.1, centre = 0.95.
        // One half-width out => 0.95 + 0.2 = 1.15.
        let mut inputs = on_target();
        inputs.branching_sigma = 1.15;
        let score = criticality_indicator(inputs, &CriticalityBands::default());
        assert!(
            (score - 0.5).abs() < 1e-4,
            "one half-width out => 0.5, got {}",
            score
        );
    }

    #[test]
    fn non_finite_inputs_collapse_to_zero() {
        let inputs = CriticalityInputs {
            branching_sigma: f32::NAN,
            power_law_alpha: 1.5,
            entropy_norm: 0.75,
        };
        assert_eq!(criticality_indicator(inputs, &CriticalityBands::default()), 0.0);

        let inputs = CriticalityInputs {
            branching_sigma: 1.0,
            power_law_alpha: f32::INFINITY,
            entropy_norm: 0.75,
        };
        assert_eq!(criticality_indicator(inputs, &CriticalityBands::default()), 0.0);
    }

    #[test]
    fn degenerate_band_treats_distance_as_zero() {
        // Operator configured a zero-width band; we don't punish them.
        let bands = CriticalityBands {
            branching_lo: 1.0,
            branching_hi: 1.0,
            ..Default::default()
        };
        let score = criticality_indicator(on_target(), &bands);
        assert!(
            (score - 1.0).abs() < 1e-6,
            "degenerate band shouldn't ruin the score, got {}",
            score
        );
    }

    #[test]
    fn worst_signal_dominates() {
        // Two signals on target, third far away => the worst one wins.
        let inputs = CriticalityInputs {
            branching_sigma: 1.0,
            power_law_alpha: 1.5,
            entropy_norm: 0.0, // way below the band
        };
        let score = criticality_indicator(inputs, &CriticalityBands::default());
        let expected = 1.0 / (1.0 + (0.6 - 0.0) / 0.15); // entropy band [0.6, 0.9]
        assert!(
            (score - expected).abs() < 1e-4,
            "worst-signal-wins, expected {} got {}",
            expected,
            score
        );
    }
}
