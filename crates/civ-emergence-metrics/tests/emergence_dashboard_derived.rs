//! FR-EMERGENCE-tests — derived dashboard metrics (novelty, coupling MI,
//! criticality) must classify synthetic inputs into non-trivial bands.
//!
//! Pure-math guardrails: a hardcoded stub that always returns `0.0` for
//! `novelty_score`, `coupling_mi_estimate`, or `criticality_indicator`
//! fails these tests.

use civ_emergence_metrics::criticality::{
    criticality_indicator, CriticalityBands, CriticalityInputs,
};
use civ_emergence_metrics::dashboard::{
    coupling_mi_estimate, novelty_score, NOVELTY_RATE_CEILING,
};

const EPS: f32 = 1e-5;

#[test]
fn fr_emergence_dashboard_novelty_score_maps_rate_to_nontrivial_band() {
    assert!((novelty_score(0.0) - 0.0).abs() < EPS);
    let mid = novelty_score(NOVELTY_RATE_CEILING * 0.5);
    assert!(
        mid > 0.25 && mid < 0.75,
        "mid-band novelty_score expected in (0.25, 0.75), got {mid}"
    );
    assert!((novelty_score(NOVELTY_RATE_CEILING) - 1.0).abs() < EPS);
}

#[test]
fn fr_emergence_dashboard_coupling_mi_estimate_is_nontrivial_when_coupled() {
    assert!((coupling_mi_estimate(0.0, 0.8) - 0.0).abs() < EPS);
    let coupled = coupling_mi_estimate(0.6, 0.7);
    assert!(
        coupled > 0.3 && coupled <= 1.0,
        "coupled MI estimate expected in (0.3, 1.0], got {coupled}"
    );
}

#[test]
fn fr_emergence_dashboard_criticality_indicator_peaks_in_operational_band() {
    let bands = CriticalityBands::default();
    let centre = CriticalityInputs {
        branching_sigma: (bands.branching_lo + bands.branching_hi) * 0.5,
        power_law_alpha: (bands.alpha_lo + bands.alpha_hi) * 0.5,
        entropy_norm: (bands.entropy_lo + bands.entropy_hi) * 0.5,
    };
    let peak = criticality_indicator(centre, &bands);
    assert!(
        peak > 0.5,
        "criticality at band centre should be > 0.5, got {peak}"
    );

    let far = CriticalityInputs {
        branching_sigma: 0.0,
        power_law_alpha: 0.0,
        entropy_norm: 0.0,
    };
    let low = criticality_indicator(far, &bands);
    assert!(
        low < peak,
        "off-band criticality ({low}) must score below in-band ({peak})"
    );
    assert!(low >= 0.0 && low <= 1.0);
}
