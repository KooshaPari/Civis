//! Integration-level coverage tests for uncovered pub fns in
//! `civ-emergence-metrics` (FR-CIV-TEST-008).
//!
//! Tests here deliberately avoid duplicating the unit tests already
//! co-located with the source modules.  They focus on two surfaces that
//! had zero test coverage before this PR:
//!
//! * `BranchingRegime::label()` — all five enum arms, stable wire labels.
//! * `BranchingLedger::closed_total()` — monotonic diagnostic counter.
//! * `JointHistogram::rows()` and `cols()` — dimension accessors used by
//!   the dashboard renderer.

use civ_emergence_metrics::branching::{
    BranchingLedger, BranchingRegime,
    SIGMA_SUBCRITICAL, SIGMA_EDGE_LOW, SIGMA_EDGE_HIGH, SIGMA_SUPERCRITICAL,
    classify_regime,
};
use civ_emergence_metrics::mutual_information::JointHistogram;

/// `BranchingRegime::label()` must return a stable, non-empty string for
/// every variant.  The dashboard uses these strings as JSON tag values on
/// `sim.snapshot`; a rename causes a protocol-level regression that CI
/// should catch.
#[test]
fn branching_regime_label_covers_all_five_arms() {
    let cases = [
        (BranchingRegime::HeatDeath,            "Subcritical (heat-death risk)"),
        (BranchingRegime::SubcriticalTransition, "Subcritical → critical transition"),
        (BranchingRegime::EdgeOfChaos,           "Edge of chaos (target)"),
        (BranchingRegime::NearSupercritical,     "Near-supercritical"),
        (BranchingRegime::Supercritical,         "Supercritical (explosion risk)"),
    ];
    for (regime, expected) in cases {
        let got = regime.label();
        assert_eq!(
            got, expected,
            "BranchingRegime::{regime:?} has unexpected label"
        );
        assert!(!got.is_empty(), "label must be non-empty");
    }
}

/// `classify_regime` round-trips back to expected labels at the boundary
/// values defined by the charter constants (AC-002, AC-005).
#[test]
fn classify_regime_boundary_values_agree_with_label() {
    // Below subcritical threshold.
    assert_eq!(
        classify_regime(SIGMA_SUBCRITICAL - 0.01).label(),
        BranchingRegime::HeatDeath.label(),
    );
    // Exactly at the SOC lower edge.
    assert_eq!(
        classify_regime(SIGMA_EDGE_LOW).label(),
        BranchingRegime::EdgeOfChaos.label(),
    );
    // Exactly at the SOC upper edge.
    assert_eq!(
        classify_regime(SIGMA_EDGE_HIGH).label(),
        BranchingRegime::EdgeOfChaos.label(),
    );
    // Above supercritical threshold.
    assert_eq!(
        classify_regime(SIGMA_SUPERCRITICAL + 0.01).label(),
        BranchingRegime::Supercritical.label(),
    );
}

/// `BranchingLedger::closed_total()` must be a monotonically increasing
/// counter that never saturates on the normal test-scale push counts.
#[test]
fn branching_ledger_closed_total_is_monotonic() {
    let mut ledger = BranchingLedger::with_capacity(3);
    assert_eq!(ledger.closed_total(), 0, "fresh ledger starts at 0");

    for i in 1..=5u64 {
        ledger.push_closed(0.9, i, i);
        assert_eq!(
            ledger.closed_total(), i,
            "closed_total must equal push count after {i} pushes"
        );
    }
    // Ring buffer capacity is 3; the last `len` is still 3 but total is 5.
    assert_eq!(ledger.len(), 3);
    assert_eq!(ledger.closed_total(), 5);
}

/// `JointHistogram::rows()` and `cols()` must return the dimensions passed
/// at construction and remain correct after observations are recorded.
#[test]
fn joint_histogram_rows_and_cols_match_construction_dimensions() {
    let mut jh = JointHistogram::new(4, 7);
    assert_eq!(jh.rows(), 4, "rows() must equal construction `rows` arg");
    assert_eq!(jh.cols(), 7, "cols() must equal construction `cols` arg");

    // Dimensions must be stable after observations.
    jh.observe(0, 0);
    jh.observe(3, 6);
    assert_eq!(jh.rows(), 4);
    assert_eq!(jh.cols(), 7);

    // A 1×1 histogram is degenerate but valid.
    let tiny = JointHistogram::new(1, 1);
    assert_eq!(tiny.rows(), 1);
    assert_eq!(tiny.cols(), 1);
}