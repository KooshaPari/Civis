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
}
