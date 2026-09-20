//! Stratification coverage for FR-CIV-GOV-200.
//!
//! Exercises `Simulation::phase_stratification` via the public household /
//! settlement seeding APIs and asserts on per-settlement reports, Gini, and
//! class-mobility (Promotion/Demotion) events.

use civ_engine::Simulation;
use civ_engine::social_types::{StratBand, StratificationEventKind};

/// Seed one settlement with 10 households of linearly increasing wealth.
fn seeded_sim() -> Simulation {
    let mut sim = Simulation::new();
    sim.set_settlement_population(1, 10);
    for i in 0..10u64 {
        sim.register_household_in_settlement(1, 100 + i);
        sim.set_household_wealth(100 + i, (i as i64 + 1) * 1_000);
    }
    sim
}

#[test]
// FR-CIV-GOV-200
fn stratification_report_computed_per_settlement() {
    let mut sim = seeded_sim();
    sim.phase_stratification();

    let report = sim
        .last_tick_stratification_report(1)
        .expect("settlement 1 must have a stratification report");
    assert_eq!(report.settlement_id, 1);
    assert!(report.gini > 0.0, "linear wealth spread must give gini > 0, got {}", report.gini);
    assert!(report.gini <= 1.0);

    // Quantiles must partition the 10 households across the 4 bands.
    let q = report.quantiles;
    assert_eq!(q.poor + q.middle + q.rich + q.elite, 10);
    assert!(q.poor > 0, "poorest quintile must be non-empty");
    assert!(q.elite > 0, "richest quintile must be non-empty");
}

#[test]
// FR-CIV-GOV-200
fn wealth_change_emits_promotion_or_demotion_event_once_per_household() {
    let mut sim = seeded_sim();
    sim.phase_stratification();

    // First pass: every (household, band) pair is emitted exactly once, with
    // kind derived from the score delta (Unchanged on the very first pass).
    let first = sim.last_tick_stratification();
    assert_eq!(first.len(), 10, "each household emits exactly one event");
    assert!(
        first
            .iter()
            .all(|e| e.kind == StratificationEventKind::Promoted),
        "first pass scores rise from 0 so every event is Promoted: {first:?}"
    );

    // Massively enrich household 100 (currently in the Poor band) and
    // impoverish household 109 (currently Elite).
    sim.set_household_wealth(100, 1_000_000);
    sim.set_household_wealth(109, 0);
    sim.phase_stratification();

    // phase_stratification appends to the event stream; scan from the end to
    // see the latest pass only.
    let events = sim.last_tick_stratification();
    let promo = events
        .iter()
        .rev()
        .find(|e| e.household_id == 100 && e.kind == StratificationEventKind::Promoted)
        .expect("household 100 must be promoted after wealth jump");
    assert_eq!(promo.band, StratBand::Elite);
    assert!(promo.score_delta > 0);

    assert!(
        events
            .iter()
            .rev()
            .any(|e| e.household_id == 109 && e.kind == StratificationEventKind::Demoted),
        "household 109 must be demoted: {events:?}"
    );

    // The band lookup must reflect the latest pass.
    assert_eq!(
        sim.household_band(100, 1),
        Some(StratBand::Elite),
        "band lookup must reflect the latest pass"
    );
}

#[test]
// FR-CIV-GOV-200
fn unchanged_wealth_produces_no_repeated_mobility_events() {
    let mut sim = seeded_sim();
    let first = {
        sim.phase_stratification();
        sim.last_tick_stratification().to_vec()
    };

    // Re-run with identical wealths: the (settlement, household, band) set
    // suppresses duplicate emissions, so the event list still shows only the
    // initial pass (same 10 events, nothing re-emitted for static wealth).
    sim.phase_stratification();
    let events = sim.last_tick_stratification();
    assert_eq!(
        events, first,
        "static wealth must not emit new events beyond the first pass"
    );
    assert_eq!(events.len(), 10);
}
