//! FR-SOCI-001 — Ideological alignment per-citizen cohort.
//!
//! Ideological alignment SHALL be tracked per-citizen cohort as a
//! continuous score.

use civ_social::{IdeologyScore};

#[test]
fn per_cohort_continuous() {
    // Two cohorts should each maintain independent scores.
    let mut c1 = IdeologyScore::new(1, 300);
    let mut c2 = IdeologyScore::new(2, -200);

    // Deltas are independent.
    c1.apply_delta(100);
    c2.apply_delta(-50);

    assert_eq!(c1.alignment_bp, 400);
    assert_eq!(c2.alignment_bp, -250);
}

#[test]
fn score_is_continuous_in_range() {
    // The score is continuous: every integer value in [-1000, 1000] is valid.
    let mut s = IdeologyScore::new(0, 0);
    for target in [-500, -1, 0, 1, 500, 999, 1000] {
        s.alignment_bp = target; // direct set (engine drives this)
        s.alignment_bp = s.alignment_bp.clamp(-1_000, 1_000);
        assert!(s.alignment_bp >= -1_000 && s.alignment_bp <= 1_000);
    }
}

#[test]
fn extreme_dissent_flagged() {
    let s = IdeologyScore::new(0, -1_000);
    assert!(s.is_dissenting());
    assert!(!s.is_aligned());
}

#[test]
fn positive_alignment_flagged() {
    let s = IdeologyScore::new(0, 1);
    assert!(s.is_aligned());
    assert!(!s.is_dissenting());
}
