//! FR-INST-002 — Institutional capture score SHALL accumulate each tick
//! based on resource concentration.

use civ_institutions::{CaptureScore, CAPTURE_THRESHOLD_BP};

/// Capture accumulates with concentration.
#[test]
fn capture_accumulates_with_concentration() {
    let mut cs = CaptureScore::new();
    let before = cs.value_bp;
    cs.accumulate(5_000); // 50% concentration
    assert!(cs.value_bp > before, "capture must increase");
}

/// Zero concentration yields zero capture accumulation.
#[test]
fn capture_zero_concentration() {
    let mut cs = CaptureScore::new();
    cs.accumulate(0);
    assert_eq!(cs.value_bp, 0);
}

/// Full concentration accumulates at maximum rate.
#[test]
fn capture_full_concentration() {
    let mut cs = CaptureScore::new();
    // With rate=10bp and concentration=10000bp: delta = 10*10000/10000 = 10bp/tick
    cs.accumulate(10_000);
    assert_eq!(cs.value_bp, 10);
}

/// Capture score never exceeds maximum.
#[test]
fn capture_clamped_to_max() {
    let mut cs = CaptureScore::new();
    for _ in 0..2000 {
        cs.accumulate(10_000);
    }
    assert!(cs.value_bp <= 10_000);
}

/// Capture score is non-negative.
#[test]
fn capture_never_negative() {
    let mut cs = CaptureScore::new();
    cs.accumulate(-500);
    assert!(cs.value_bp >= 0);
}

/// Capture threshold detection works at 0.75.
#[test]
fn capture_threshold_at_075() {
    let cs = CaptureScore::with_value(CAPTURE_THRESHOLD_BP);
    assert!(cs.is_captured(), "score at threshold should be captured");
}

/// Multiple ticks accumulate capture progressively.
#[test]
fn capture_accumulates_over_ticks() {
    let mut cs = CaptureScore::new();
    for _ in 0..100 {
        cs.accumulate(5_000);
    }
    // After 100 ticks at 50% concentration: ~500 bp
    assert!(cs.value_bp >= 450, "should accumulate over time");
}
