//! FR traceability tests for the observability crate.
//!
//! Covers: FR-CIV-METRICS-001, FR-CIV-METRICS-001-TIMESERIES

use civ_observability::{SimMetricSnapshot, SimMetrics};
use prometheus::Registry;

/// FR-CIV-METRICS-001 — SimMetrics can be created on a Prometheus registry.
#[test]
fn fr_civ_metrics_001_sim_metrics_creation() {
    let registry = Registry::new();
    let metrics = SimMetrics::new(&registry).expect("SimMetrics::new should succeed");
    // Verify the metrics were registered
    let gathered = registry.gather();
    assert!(!gathered.is_empty(), "registry should have metric families after SimMetrics::new");
}

/// FR-CIV-METRICS-001 — SimMetricSnapshot can be constructed with all fields.
#[test]
fn fr_civ_metrics_001_snapshot_construction() {
    let snapshot = SimMetricSnapshot {
        tick_duration_secs: 0.016,
        entity_count: 1000,
        faction_count: 5,
        building_count: 200,
        economy_treasury: 50000.0,
        diplomacy_treaties: 3,
        emergence_entropy: 0.75,
    };
    assert_eq!(snapshot.entity_count, 1000);
    assert_eq!(snapshot.faction_count, 5);
    assert!((snapshot.emergence_entropy - 0.75).abs() < 1e-6);
}

/// FR-CIV-METRICS-001-TIMESERIES — SimMetrics can record a snapshot.
#[test]
fn fr_civ_metrics_001_timeseries_record_snapshot() {
    let registry = Registry::new();
    let metrics = SimMetrics::new(&registry).expect("SimMetrics::new");
    let snapshot = SimMetricSnapshot {
        tick_duration_secs: 0.025,
        entity_count: 500,
        faction_count: 3,
        building_count: 100,
        economy_treasury: 25000.0,
        diplomacy_treaties: 2,
        emergence_entropy: 0.5,
    };
    metrics.record(&snapshot);

    // Verify tick duration was recorded
    let tick_hist = metrics.tick_duration.get_sample_count();
    assert_eq!(tick_hist, 1, "should have exactly 1 sample after record");
}

/// FR-CIV-METRICS-001 — Metric name constants are non-empty.
#[test]
fn fr_civ_metrics_001_metric_name_constants() {
    assert!(!civ_observability::perf::TICK_DURATION.is_empty());
    assert!(!civ_observability::perf::ENTITY_COUNT.is_empty());
    assert!(!civ_observability::perf::FACTION_COUNT.is_empty());
    assert!(!civ_observability::perf::BUILDING_COUNT.is_empty());
    assert!(!civ_observability::perf::ECONOMY_TREASURY.is_empty());
    assert!(!civ_observability::perf::DIPLOMACY_TREATIES.is_empty());
    assert!(!civ_observability::perf::EMERGENCE_ENTROPY.is_empty());
}
