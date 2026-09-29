//! Behavioural traceability test for `FR-CIV-PSYCHE-010`.
//!
//! The ID is tagged on `civ_agents::psyche::nudge_temperament`, the bounded
//! plasticity update that pulls a `Temperament` toward recent experience.
//! These tests assert the real contract: movement is toward the target, never
//! past it; learning rate shrinks with maturity; and both axes stay in `[0, 1]`.

use civ_agents::psyche::nudge_temperament;
use civ_agents::Temperament;

const EPS: f32 = 1e-6;

fn temper(reactivity: f32, sociability: f32) -> Temperament {
    Temperament {
        reactivity,
        sociability,
        risk_tol: 0.5,
        impulsivity: 0.5,
    }
}

/// FR-CIV-PSYCHE-010 — a recent high-variance / high-satisfaction period pulls
/// both temperament axes upward, and the move is toward (not past) the target.
#[test]
fn psyche_010_nudges_toward_recent_experience() {
    let mut t = temper(0.2, 0.2);
    nudge_temperament(&mut t, 1.0, 1.0, 0.0);

    assert!(t.reactivity > 0.2, "reactivity should rise toward 1.0");
    assert!(t.reactivity < 1.0, "reactivity must not overshoot the target");
    assert!(t.sociability > 0.2, "sociability should rise toward 1.0");
    assert!(t.sociability < 1.0, "sociability must not overshoot the target");
}

/// FR-CIV-PSYCHE-010 — the pull is symmetric: a low-variance period lowers
/// reactivity instead of raising it.
#[test]
fn psyche_010_low_variance_lowers_reactivity() {
    let mut t = temper(0.8, 0.5);
    nudge_temperament(&mut t, 0.0, 0.5, 0.0);

    assert!(t.reactivity < 0.8, "stable period should reduce reactivity");
    assert!(t.reactivity > 0.0, "reactivity must not undershoot the target");
    // An exactly-on-target social signal leaves that axis untouched.
    assert!((t.sociability - 0.5).abs() < EPS);
}

/// FR-CIV-PSYCHE-010 — plasticity decays with maturity: the same experience
/// moves a juvenile further than a mature adult.
#[test]
fn psyche_010_plasticity_decays_with_maturity() {
    let mut juvenile = temper(0.5, 0.5);
    let mut adult = temper(0.5, 0.5);
    nudge_temperament(&mut juvenile, 1.0, 1.0, 0.0);
    nudge_temperament(&mut adult, 1.0, 1.0, 1.0);

    assert!(juvenile.reactivity > adult.reactivity);
    assert!(juvenile.sociability > adult.sociability);
    // A fully mature adult has 20% plasticity, so lr = 0.0004, and the step is
    // lr * (target - current) = 0.0004 * 0.5.
    let adult_lr = 0.002 * 0.2;
    assert!((adult.reactivity - (0.5 + adult_lr * 0.5)).abs() < EPS);
    // A juvenile has full plasticity, so lr = 0.002 and the step is 5x larger.
    let juvenile_step = juvenile.reactivity - 0.5;
    assert!((juvenile_step - 0.002 * 0.5).abs() < EPS);
    assert!((juvenile_step / (adult.reactivity - 0.5) - 5.0).abs() < 1e-3);
}

/// FR-CIV-PSYCHE-010 — an already-saturated axis is pinned at the boundary;
/// repeated nudges cannot push a value outside `[0, 1]`.
#[test]
fn psyche_010_saturated_axes_stay_clamped() {
    let mut t = temper(1.0, 1.0);
    for _ in 0..50 {
        nudge_temperament(&mut t, 1.0, 1.0, 0.0);
    }
    assert_eq!(t.reactivity, 1.0);
    assert_eq!(t.sociability, 1.0);

    let mut low = temper(0.0, 0.0);
    for _ in 0..50 {
        nudge_temperament(&mut low, 0.0, 0.0, 0.0);
    }
    assert_eq!(low.reactivity, 0.0);
    assert_eq!(low.sociability, 0.0);
}

/// FR-CIV-PSYCHE-010 — an out-of-range maturity is clamped rather than
/// producing a negative learning rate that would move traits the wrong way.
#[test]
fn psyche_010_out_of_range_maturity_is_clamped() {
    let mut t = temper(0.5, 0.5);
    // maturity > 1.25 drives the raw plasticity negative; it must clamp to zero.
    nudge_temperament(&mut t, 0.0, 0.0, 10.0);
    assert!((t.reactivity - 0.5).abs() < EPS);
    assert!((t.sociability - 0.5).abs() < EPS);
}

/// FR-CIV-PSYCHE-010 — only reactivity and sociability are plastic; the other
/// two temperament axes are left untouched.
#[test]
fn psyche_010_leaves_other_axes_untouched() {
    let mut t = temper(0.3, 0.3);
    nudge_temperament(&mut t, 0.9, 0.9, 0.5);
    assert_eq!(t.risk_tol, 0.5);
    assert_eq!(t.impulsivity, 0.5);
}
