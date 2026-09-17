//! FR-CLIM-005 — Integration test: tipping-point cascade modelling.
//!
//! Verifies that tipping points trigger above critical temperatures and
//! amplify warming through cascading feedback.

use civ_climate::tipping::{CascadeTracker, TippingPoint};

/// FR-CLIM-005: Tipping-point cascade is triggered above critical temperature.
#[test]
fn cascade_triggered() {
    let mut tracker = CascadeTracker::new();

    // Below all thresholds — no triggering.
    let r1 = tracker.evaluate(1.0, 10);
    assert!(r1.newly_triggered.is_empty());
    assert_eq!(r1.amplification_c, 0.0);

    // Ice-albedo triggers at +2.5 °C.
    let r2 = tracker.evaluate(2.6, 11);
    assert_eq!(r2.newly_triggered.len(), 1);
    assert!(r2.newly_triggered.contains(&TippingPoint::IceAlbedo));
    assert!(tracker.is_triggered(TippingPoint::IceAlbedo));
}

/// FR-CLIM-005: All three tipping points activate at high anomaly.
#[test]
fn full_cascade_at_high_anomaly() {
    let mut tracker = CascadeTracker::new();
    let result = tracker.evaluate(5.0, 100);

    assert_eq!(result.newly_triggered.len(), 3, "All three tipping points should trigger");
    assert!(result.newly_triggered.contains(&TippingPoint::IceAlbedo));
    assert!(result.newly_triggered.contains(&TippingPoint::Permafrost));
    assert!(result.newly_triggered.contains(&TippingPoint::AmazonDieback));
    assert_eq!(tracker.triggered_count(), 3);
}

/// FR-CLIM-005: Tipping points amplify warming cumulatively.
#[test]
fn cascading_amplification() {
    let mut tracker = CascadeTracker::new();

    // Only ice-albedo
    let r1 = tracker.evaluate(2.6, 10);
    let amp1 = r1.amplification_c;
    assert!(amp1 > 0.0);

    // Ice-albedo + permafrost
    let r2 = tracker.evaluate(3.1, 11);
    let amp2 = r2.amplification_c;
    assert!(amp2 > amp1, "More tipping points should produce more amplification");
}

/// FR-CLIM-005: Once triggered, a tipping point stays active permanently.
#[test]
fn tipping_points_are_irreversible() {
    let mut tracker = CascadeTracker::new();
    tracker.evaluate(3.0, 10);
    assert!(tracker.is_triggered(TippingPoint::IceAlbedo));

    // Anomaly drops but ice-albedo stays triggered
    let result = tracker.evaluate(1.0, 11);
    assert!(tracker.is_triggered(TippingPoint::IceAlbedo));
    // No new triggers, but amplification still active
    assert!(result.amplification_c > 0.0);
}

/// FR-CLIM-005: Sequential crossing across ticks.
#[test]
fn sequential_tipping_across_ticks() {
    let mut tracker = CascadeTracker::new();

    let r0 = tracker.evaluate(2.0, 1);
    assert!(r0.newly_triggered.is_empty());

    let r1 = tracker.evaluate(2.6, 2);
    assert_eq!(r1.newly_triggered.len(), 1); // ice-albedo

    let r2 = tracker.evaluate(3.1, 3);
    assert_eq!(r2.newly_triggered.len(), 1); // permafrost

    let r3 = tracker.evaluate(3.6, 4);
    assert_eq!(r3.newly_triggered.len(), 1); // amazon dieback

    assert_eq!(tracker.triggered_count(), 3);
}

/// FR-CLIM-005: CO₂ release from permafrost and amazon dieback.
#[test]
fn co2_release_from_tipping_points() {
    let mut tracker = CascadeTracker::new();
    let result = tracker.evaluate(5.0, 100);

    // Permafrost releases 0.5 ppm, Amazon releases 0.3 ppm, ice-albedo releases 0.0
    assert!(result.co2_release_ppm > 0.0, "Active tipping points should release CO₂");
}
