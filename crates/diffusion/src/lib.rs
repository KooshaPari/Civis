//! civ-diffusion — Bass/Rogers S-curve tech-adoption engine.
//!
//! Pure deterministic math (no LLM, no RNG). Given an innovation coefficient
//! `p`, an imitation coefficient `q`, and the currently adopted fraction
//! `f(t)`, the Bass model predicts the per-tick increase:
//!
//! ```text
//! f'(t) = (p + q · f(t)) · (1 − f(t))
//! ```
//!
//! See the diffusion-of-innovations literature (Rogers 1962, Bass 1969). This
//! crate drives the per-civilian wardrobe + tools era propagation in
//! `civ-agents` so that visible technology spreads gradually across a
//! civilization rather than snap-upgrading.
//!
//! All math runs in `f32`; the simulation owner quantises into per-civilian
//! discrete state once the diffusion rate is known. Replay determinism is
//! preserved by `civ-engine`'s fixed-point seeding (this crate does not
//! consume RNG).
//!
//! See `docs/roadmap/civis-3d-extension.md`. FR coverage: `FR-CIV-DIFFUSION-*`.

#![forbid(unsafe_code)]
#![warn(missing_docs)]

use serde::{Deserialize, Serialize};

/// Schema version for `civ-diffusion`. Bumped on breaking changes.
pub const SCHEMA_VERSION: &str = "0.1.0-stub";

/// Parameters describing one tech-adoption curve. `p` is the innovation
/// coefficient (the rate at which non-adopters spontaneously try the tech),
/// `q` is the imitation coefficient (the rate at which non-adopters copy
/// adopters). Both should be in `[0.0, 1.0]`. The classic Bass-model
/// consumer-goods meta-analysis suggests `p ≈ 0.03`, `q ≈ 0.38`.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct DiffusionParams {
    /// Innovation coefficient.
    pub p: f32,
    /// Imitation coefficient.
    pub q: f32,
}

impl Default for DiffusionParams {
    fn default() -> Self {
        Self { p: 0.03, q: 0.38 }
    }
}

/// Per-tick adoption increase given current adopted fraction `f` and params.
#[must_use]
pub fn tick_increase(f: f32, params: DiffusionParams) -> f32 {
    let f = f.clamp(0.0, 1.0);
    let rate = (params.p + params.q * f) * (1.0 - f);
    rate.max(0.0)
}

/// Advance one tick: returns the new adopted fraction. Clamped to `[0, 1]`.
#[must_use]
pub fn advance(f: f32, params: DiffusionParams) -> f32 {
    (f + tick_increase(f, params)).clamp(0.0, 1.0)
}

/// Simulate `ticks` ticks forward from `f0`. Returns length `ticks + 1`
/// (including the initial value).
#[must_use]
pub fn trajectory(f0: f32, params: DiffusionParams, ticks: usize) -> Vec<f32> {
    let mut out = Vec::with_capacity(ticks + 1);
    let mut f = f0.clamp(0.0, 1.0);
    out.push(f);
    for _ in 0..ticks {
        f = advance(f, params);
        out.push(f);
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    /// FR-CIV-DIFFUSION-000 — exposes a semver-like schema version stub.
    #[test]
    fn schema_version_stub() {
        assert!(!SCHEMA_VERSION.is_empty());
        let core = SCHEMA_VERSION.split('-').next().unwrap();
        let segments: Vec<&str> = core.split('.').collect();
        assert_eq!(segments.len(), 3);
        assert!(segments.iter().all(|part| !part.is_empty()));
    }

    /// FR-CIV-DIFFUSION-001 — adoption matches the Bass closed-form for the
    /// per-tick increment.
    #[test]
    fn tick_increase_matches_closed_form() {
        let p = DiffusionParams { p: 0.03, q: 0.38 };
        for f in [0.0_f32, 0.1, 0.25, 0.5, 0.75, 0.9, 1.0] {
            let expected = ((p.p + p.q * f) * (1.0 - f)).max(0.0);
            let got = tick_increase(f, p);
            assert!(
                (got - expected).abs() < 1e-6,
                "f={f}: expected {expected}, got {got}"
            );
        }
    }

    /// FR-CIV-DIFFUSION-002 — fully-adopted population produces zero increase.
    #[test]
    fn saturation_produces_zero_increase() {
        let p = DiffusionParams::default();
        assert!(tick_increase(1.0, p).abs() < 1e-6);
    }

    /// FR-CIV-DIFFUSION-003 — trajectories are monotonically non-decreasing
    /// when both `p` and `q` are non-negative.
    #[test]
    fn trajectories_are_monotone_nondecreasing() {
        let p = DiffusionParams::default();
        let traj = trajectory(0.0, p, 200);
        for window in traj.windows(2) {
            assert!(
                window[0] <= window[1] + 1e-6,
                "non-monotone at {} → {}",
                window[0],
                window[1]
            );
        }
        assert!(*traj.last().unwrap() > 0.95);
    }

    /// FR-CIV-DIFFUSION-004 — clamping: out-of-range inputs are corrected.
    #[test]
    fn out_of_range_inputs_clamp() {
        let p = DiffusionParams::default();
        assert!(advance(-0.5, p) >= 0.0);
        assert!(advance(1.5, p) <= 1.0);
    }

    /// FR-CIV-DIFFUSION-005 — running the same trajectory twice produces
    /// bit-identical output (no hidden RNG, no time leakage).
    #[test]
    fn trajectory_is_deterministic() {
        let p = DiffusionParams { p: 0.02, q: 0.4 };
        let a = trajectory(0.01, p, 100);
        let b = trajectory(0.01, p, 100);
        assert_eq!(a, b);
    }

    // -----------------------------------------------------------------------
    // Conservation of "mass" (adoption total / trajectory integrity)
    // -----------------------------------------------------------------------

    /// FR-CIV-DIFFUSION-006 — the trajectory vector length is exactly `ticks + 1`
    /// (the initial state plus one entry per tick). No values are lost or doubled.
    #[test]
    fn trajectory_length_equals_ticks_plus_one() {
        let p = DiffusionParams::default();
        for ticks in [0, 1, 10, 100] {
            let traj = trajectory(0.1, p, ticks);
            assert_eq!(
                traj.len(),
                ticks + 1,
                "ticks={ticks}: expected {} entries, got {}",
                ticks + 1,
                traj.len()
            );
        }
    }

    /// FR-CIV-DIFFUSION-007 — the first element of the trajectory is always the
    /// (clamped) initial adoption fraction — no off-by-one in the output buffer.
    #[test]
    fn trajectory_first_element_is_initial_value() {
        let p = DiffusionParams::default();
        for f0 in [0.0_f32, 0.25, 0.5, 0.9, 1.0] {
            let traj = trajectory(f0, p, 5);
            assert!(
                (traj[0] - f0).abs() < 1e-6,
                "f0={f0}: first element should be {f0}, got {}",
                traj[0]
            );
        }
    }

    /// FR-CIV-DIFFUSION-008 — the cumulative adoption increase over N ticks
    /// equals the difference between the final and initial adoption fractions.
    /// This verifies that `tick_increase` and `advance` are consistent (no
    /// adoption is created or discarded between calls).
    #[test]
    fn cumulative_increase_equals_final_minus_initial() {
        let p = DiffusionParams { p: 0.03, q: 0.38 };
        let f0 = 0.05_f32;
        let ticks = 50;
        let traj = trajectory(f0, p, ticks);
        let sum_of_increases: f32 = traj.windows(2).map(|w| w[1] - w[0]).sum();
        let direct_delta = traj[ticks] - traj[0];
        assert!(
            (sum_of_increases - direct_delta).abs() < 1e-4,
            "sum of increments {sum_of_increases} != final-initial {direct_delta}"
        );
    }

    // -----------------------------------------------------------------------
    // Monotone spread (resources spread outward, never concentrate backward)
    // -----------------------------------------------------------------------

    /// FR-CIV-DIFFUSION-009 — adoption is strictly non-decreasing tick-by-tick
    /// even when both coefficients are very small (nearly no spread).
    #[test]
    fn monotone_spread_with_minimal_coefficients() {
        let p = DiffusionParams { p: 0.001, q: 0.001 };
        let traj = trajectory(0.01, p, 300);
        for window in traj.windows(2) {
            assert!(
                window[1] >= window[0] - 1e-7,
                "non-monotone: {} → {}",
                window[0],
                window[1]
            );
        }
    }

    /// FR-CIV-DIFFUSION-010 — when imitation dominates (q >> p), adoption
    /// still never decreases and reaches near-saturation faster than the
    /// innovation-only case.
    #[test]
    fn imitation_dominated_curve_is_faster_than_innovation_only() {
        let innovation_only = DiffusionParams { p: 0.03, q: 0.0 };
        let imitation_heavy = DiffusionParams { p: 0.03, q: 0.8 };
        let f0 = 0.01_f32;
        let ticks = 50;

        let traj_inno = trajectory(f0, innovation_only, ticks);
        let traj_imit = trajectory(f0, imitation_heavy, ticks);

        // Monotone for both.
        for w in traj_imit.windows(2) {
            assert!(w[1] >= w[0] - 1e-7, "imitation traj non-monotone");
        }
        for w in traj_inno.windows(2) {
            assert!(w[1] >= w[0] - 1e-7, "innovation traj non-monotone");
        }

        // Imitation-heavy reaches higher adoption after the same number of ticks.
        assert!(
            traj_imit[ticks] > traj_inno[ticks],
            "imitation-heavy ({}) should outpace innovation-only ({}) at tick {ticks}",
            traj_imit[ticks],
            traj_inno[ticks]
        );
    }

    /// FR-CIV-DIFFUSION-011 — tick_increase is always non-negative for any
    /// valid (non-negative p, q) parameter set, so adoption never reverses.
    #[test]
    fn tick_increase_is_always_nonnegative() {
        let param_pairs = [
            (0.0_f32, 0.0_f32),
            (0.03, 0.0),
            (0.0, 0.38),
            (0.03, 0.38),
            (1.0, 1.0),
        ];
        for (pv, qv) in param_pairs {
            let params = DiffusionParams { p: pv, q: qv };
            for f_tenth in 0..=10 {
                let f = f_tenth as f32 / 10.0;
                let inc = tick_increase(f, params);
                assert!(
                    inc >= 0.0,
                    "p={pv} q={qv} f={f}: tick_increase should be >= 0, got {inc}"
                );
            }
        }
    }

    // -----------------------------------------------------------------------
    // Edge cases: empty / degenerate inputs
    // -----------------------------------------------------------------------

    /// FR-CIV-DIFFUSION-012 — zero ticks: trajectory returns a single-element
    /// vector equal to the (clamped) initial value; no panic.
    #[test]
    fn zero_ticks_returns_single_element() {
        let p = DiffusionParams::default();
        let traj = trajectory(0.5, p, 0);
        assert_eq!(traj.len(), 1);
        assert!((traj[0] - 0.5).abs() < 1e-6);
    }

    /// FR-CIV-DIFFUSION-013 — zero-coefficient params (p=0, q=0): adoption
    /// cannot grow at all; advance() returns the same value forever.
    #[test]
    fn zero_coefficients_produce_no_growth() {
        let p = DiffusionParams { p: 0.0, q: 0.0 };
        let f0 = 0.3_f32;
        let traj = trajectory(f0, p, 50);
        for &v in &traj {
            assert!(
                (v - f0).abs() < 1e-6,
                "p=q=0 should hold adoption constant at {f0}, got {v}"
            );
        }
    }

    /// FR-CIV-DIFFUSION-014 — fully-saturated start (f=1.0): adoption cannot
    /// exceed 1.0 and remains at 1.0 for all ticks regardless of coefficients.
    #[test]
    fn fully_saturated_start_stays_at_one() {
        let p = DiffusionParams::default();
        let traj = trajectory(1.0, p, 20);
        for &v in &traj {
            assert!(
                (v - 1.0).abs() < 1e-6,
                "saturated trajectory should stay at 1.0, got {v}"
            );
        }
    }

    /// FR-CIV-DIFFUSION-015 — single-tick trajectory: advance() and
    /// trajectory()[1] must agree exactly.
    #[test]
    fn single_tick_trajectory_matches_advance() {
        let p = DiffusionParams { p: 0.05, q: 0.4 };
        let f0 = 0.2_f32;
        let traj = trajectory(f0, p, 1);
        let direct = advance(f0, p);
        assert!(
            (traj[1] - direct).abs() < 1e-7,
            "trajectory[1]={} != advance()={}",
            traj[1],
            direct
        );
    }
}
