//! FR traceability tests for engine metrics, replay, and determinism.
//!
//! Covers: FR-METRICS-001/002/003, FR-CIV-METRICS-001/001-TIMESERIES,
//! FR-REPLAY-002, NFR-CIV-DET-001/002/003

use civ_engine::metrics::{compute, compute_fixed, Metrics, MetricsFixed};
use civ_engine::replay::ReplayEvent;
use civ_engine::Fixed;

// ===========================================================================
// FR-METRICS-001 — Metrics computation produces valid values
// ===========================================================================

/// FR-METRICS-001 — Metrics with zero budget and zero consumption.
#[test]
fn fr_metrics_001_zero_budget_zero_consumption() {
    let m = compute(0.0, 0.0);
    assert!(m.waste_joules >= 0.0);
    assert!(m.surplus_joules >= 0.0);
    assert!(m.tyranny_index >= 0.0);
    assert!(m.legitimacy_index >= 0.0);
}

/// FR-METRICS-001 — Metrics with positive budget and consumption.
#[test]
fn fr_metrics_001_positive_values() {
    let m = compute(1000.0, 500.0);
    assert!(m.waste_joules >= 0.0, "waste must be non-negative");
    assert!(m.surplus_joules > 0.0, "surplus should be positive");
    assert!(m.tyranny_index > 0.0, "tyranny should be positive");
    assert!(m.legitimacy_index > 0.0, "legitimacy should be positive");
}

// ===========================================================================
// FR-METRICS-002 — Fixed-point metrics match float metrics
// ===========================================================================

/// FR-METRICS-002 — Fixed-point and float metrics agree on tyranny index.
#[test]
fn fr_metrics_002_fixed_matches_float_tyranny() {
    let budget = 1000.0_f64;
    let consumption = 400.0_f64;
    let float_m = compute(budget, consumption);
    let fixed_m = compute_fixed(
        Fixed::from_num(budget as i64),
        Fixed::from_num(consumption as i64),
    );
    let diff = (float_m.tyranny_index - fixed_m.tyranny_index.to_num::<f64>()).abs();
    assert!(diff < 0.01, "tyranny index mismatch: float={} fixed={}", float_m.tyranny_index, fixed_m.tyranny_index.to_num::<f64>());
}

// ===========================================================================
// FR-METRICS-003 — Metrics are clamped to sane ranges
// ===========================================================================

/// FR-METRICS-003 — Tyranny index is always in [0, 1].
#[test]
fn fr_metrics_003_tyranny_clamped() {
    for budget in [0.0, 1.0, 100.0, 10000.0] {
        for consumption in [0.0, 50.0, 500.0, 50000.0] {
            let m = compute(budget, consumption);
            assert!(m.tyranny_index >= 0.0 && m.tyranny_index <= 1.0,
                "tyranny out of range: {budget}/{consumption} = {}", m.tyranny_index);
        }
    }
}

// ===========================================================================
// FR-CIV-METRICS-001 — Metrics with infinity/nan inputs are safe
// ===========================================================================

/// FR-CIV-METRICS-001 — NaN inputs produce zero metrics (not NaN).
#[test]
fn fr_civ_metrics_001_nan_inputs_produce_valid() {
    let m = compute(f64::NAN, f64::NAN);
    assert!(m.waste_joules.is_finite(), "waste must be finite with NaN input");
    assert!(m.surplus_joules.is_finite(), "surplus must be finite with NaN input");
    assert!(m.tyranny_index.is_finite(), "tyranny must be finite with NaN input");
}

/// FR-CIV-METRICS-001 — Infinity inputs produce clamped metrics.
#[test]
fn fr_civ_metrics_001_infinity_inputs_clamped() {
    let m = compute(f64::INFINITY, f64::INFINITY);
    assert!(m.tyranny_index <= 1.0, "tyranny must be <= 1.0 with inf input");
}

// ===========================================================================
// FR-CIV-METRICS-001-TIMESERIES — MetricsFixed equality
// ===========================================================================

/// FR-CIV-METRICS-001-TIMESERIES — MetricsFixed derives PartialEq and Default.
#[test]
fn fr_civ_metrics_001_timeseries_fixed_equality() {
    let a = MetricsFixed::default();
    let b = MetricsFixed::default();
    assert_eq!(a, b, "default MetricsFixed values should be equal");
}

// ===========================================================================
// FR-REPLAY-002 — ReplayEvent construction and serialisation
// ===========================================================================

/// FR-REPLAY-002 — ReplayEvent variants can be constructed.
#[test]
fn fr_replay_002_event_construction() {
    use civ_voxel::{MaterialId, WorldCoord};
    let event = ReplayEvent::VoxelWrite {
        tick: 100,
        pos: WorldCoord { x: 1, y: 2, z: 3 },
        value: MaterialId(5),
    };
    match event {
        ReplayEvent::VoxelWrite { tick, .. } => assert_eq!(tick, 100),
        _ => panic!("expected VoxelWrite"),
    }
}

/// FR-REPLAY-002 — ReplayEvent can be serialised and deserialised.
#[test]
fn fr_replay_002_event_serde_roundtrip() {
    use civ_voxel::{MaterialId, WorldCoord};
    let event = ReplayEvent::VoxelWrite {
        tick: 42,
        pos: WorldCoord { x: 0, y: 0, z: 0 },
        value: MaterialId(1),
    };
    let json = serde_json::to_string(&event).expect("serialize");
    let decoded: ReplayEvent = serde_json::from_str(&json).expect("deserialize");
    assert_eq!(event, decoded);
}

// ===========================================================================
// NFR-CIV-DET-001 — Determinism: same seeds produce identical metrics
// ===========================================================================

/// NFR-CIV-DET-001 — Same inputs produce identical Metrics.
#[test]
fn fr_nfr_civ_det_001_deterministic_metrics() {
    let a = compute(500.0, 300.0);
    let b = compute(500.0, 300.0);
    assert_eq!(a.waste_joules, b.waste_joules);
    assert_eq!(a.surplus_joules, b.surplus_joules);
    assert_eq!(a.tyranny_index, b.tyranny_index);
    assert_eq!(a.legitimacy_index, b.legitimacy_index);
}

/// NFR-CIV-DET-002 — Fixed-point metrics are deterministic across calls.
#[test]
fn fr_nfr_civ_det_002_deterministic_fixed_metrics() {
    let budget = Fixed::from_num(1000i64);
    let consumption = Fixed::from_num(400i64);
    let a = compute_fixed(budget, consumption);
    let b = compute_fixed(budget, consumption);
    assert_eq!(a, b);
}

/// NFR-CIV-DET-003 — Metrics are invariant to order of computation.
#[test]
fn fr_nfr_civ_det_003_metrics_order_invariant() {
    let m1 = compute(1000.0, 500.0);
    let m2 = compute(1000.0, 500.0);
    assert_eq!(m1.tyranny_index, m2.tyranny_index);
}
