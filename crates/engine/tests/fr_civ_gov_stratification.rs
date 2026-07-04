//! FR-CIV-GOV-020 TDD red step: phase_stratification
//!
//! Red step of TDD for civ-007-diplomacy-laws-government (stratification
//! sub-epic). Stratification tracks per-household wealth + power scores,
//! computes quantile bands (poor/middle/rich/elite), and emits per-tick
//! mobility events when a household crosses a band boundary.
//!
//! Public API the green step must provide:
//!
//! ```ignore
//! civ_engine::Simulation
//!     .set_household_wealth(household_id, units)   // i64
//!     .set_household_power(household_id, units)    // i64
//!     .register_household(household_id)            // optional, defaults to wealth=0
//!     .last_tick_stratification() -> &[StratificationEvent]
//! civ_engine::StratificationEvent { household_id, kind, band, score, score_delta }
//! civ_engine::StratificationEventKind::Promoted | Demoted | Unchanged
//! civ_engine::StratBand::Poor | Middle | Rich | Elite
//! civ_engine::StratificationReport { settlement_id, quantiles, gini }
//! civ_engine::last_tick_stratification_report(settlement_id) -> Option<StratificationReport>
//! ```
//!
//! The green step must also call `phase_stratification` from the `run_phase()`
//! match arm at the right cadence (every 16 ticks per ADR-020).
//!
//! Tests pinned by this file:
//!
//! - FR-CIV-GOV-020.base      per-tick stratification events are emitted
//! - FR-CIV-GOV-020.quantiles per-settlement quantile bands are computed
//! - FR-CIV-GOV-020.gini      Gini coefficient is in [0, 1]
//! - FR-CIV-GOV-020.mobility  Promoted event fires when a household crosses
//!                            a band boundary upward; Demoted downward
//! - FR-CIV-GOV-020.determinism identical seeds produce identical reports
//!
//! Spec: agileplus-specs/civ-007-diplomacy-laws-government/spec.md

#![cfg(test)]

use civ_engine::{Sim, SimSeed, StratBand, StratificationEvent, StratificationEventKind, StratificationReport};

const STRAT_SEED: u64 = 0xC1_07_C1_07;

fn empty_sim() -> Sim {
    Sim::with_seed(SimSeed::from_u64(STRAT_SEED))
}

#[test]
fn fr_civ_gov_020_base_per_tick_stratification_events_emitted() {
    let mut sim = empty_sim();
    sim.register_household(101);
    sim.register_household(202);
    sim.register_household(303);

    sim.tick();
    let events_after_first = sim.last_tick_stratification().len();

    // tick 1: no comparison baseline yet → no Promoted/Demoted events.
    assert_eq!(
        events_after_first, 0,
        "no stratification events should fire on the very first tick (no baseline)"
    );

    // bump household 202's wealth dramatically and tick again
    sim.set_household_wealth(202, 10_000);
    sim.tick();
    let events_after_bump = sim.last_tick_stratification().len();
    assert!(
        events_after_bump >= 1,
        "expected at least one stratification event after a wealth bump (got {events_after_bump})"
    );

    // event points at the right household
    let promoted = sim
        .last_tick_stratification()
        .iter()
        .find(|e| e.household_id == 202 && matches!(e.kind, StratificationEventKind::Promoted));
    assert!(
        promoted.is_some(),
        "expected a Promoted event for household 202 after a 10k wealth bump"
    );
}

#[test]
fn fr_civ_gov_020_quantiles_per_settlement_bands_computed() {
    let mut sim = empty_sim();
    let settlement_id = 7;

    // 4 households: one in each band per the canonical scoring
    //   Poor:    wealth < 50
    //   Middle:  50 <= wealth < 500
    //   Rich:    500 <= wealth < 5_000
    //   Elite:   wealth >= 5_000
    sim.register_household_in_settlement(settlement_id, 1);
    sim.register_household_in_settlement(settlement_id, 2);
    sim.register_household_in_settlement(settlement_id, 3);
    sim.register_household_in_settlement(settlement_id, 4);

    sim.set_household_wealth(1, 10); // Poor
    sim.set_household_wealth(2, 250); // Middle
    sim.set_household_wealth(3, 2_500); // Rich
    sim.set_household_wealth(4, 50_000); // Elite

    // let the simulation tick to compute the bands
    for _ in 0..2 {
        sim.tick();
    }

    let report = sim
        .last_tick_stratification_report(settlement_id)
        .expect("expected a stratification report for settlement 7");

    // Each band must have at least one household.
    assert!(
        report.quantiles.poor >= 1,
        "expected at least 1 Poor household (got {})",
        report.quantiles.poor
    );
    assert!(
        report.quantiles.middle >= 1,
        "expected at least 1 Middle household (got {})",
        report.quantiles.middle
    );
    assert!(
        report.quantiles.rich >= 1,
        "expected at least 1 Rich household (got {})",
        report.quantiles.rich
    );
    assert!(
        report.quantiles.elite >= 1,
        "expected at least 1 Elite household (got {})",
        report.quantiles.elite
    );

    // spot-check band membership against the public API
    assert_eq!(sim.household_band(1, settlement_id), Some(StratBand::Poor));
    assert_eq!(sim.household_band(2, settlement_id), Some(StratBand::Middle));
    assert_eq!(sim.household_band(3, settlement_id), Some(StratBand::Rich));
    assert_eq!(sim.household_band(4, settlement_id), Some(StratBand::Elite));
}

#[test]
fn fr_civ_gov_020_gini_in_unit_interval() {
    let mut sim = empty_sim();
    let settlement_id = 9;

    // extreme: all wealth to one household -> Gini should be ~1
    for i in 0..10 {
        sim.register_household_in_settlement(settlement_id, i);
    }
    for i in 1..10 {
        sim.set_household_wealth(i, 0);
    }
    sim.set_household_wealth(0, 1_000_000);
    for _ in 0..2 {
        sim.tick();
    }

    let report_unequal = sim
        .last_tick_stratification_report(settlement_id)
        .expect("report for settlement 9");
    assert!(
        (0.0..=1.0).contains(&report_unequal.gini),
        "Gini for unequal distribution must be in [0, 1] (got {})",
        report_unequal.gini
    );
    assert!(
        report_unequal.gini > 0.5,
        "expected Gini > 0.5 for highly unequal distribution (got {})",
        report_unequal.gini
    );

    // equal: each household has 100 -> Gini should be ~0
    let settlement_id_equal = 10;
    for i in 0..10 {
        sim.register_household_in_settlement(settlement_id_equal, i);
        sim.set_household_wealth(i, 100);
    }
    for _ in 0..2 {
        sim.tick();
    }

    let report_equal = sim
        .last_tick_stratification_report(settlement_id_equal)
        .expect("report for settlement 10");
    assert!(
        report_equal.gini < 0.01,
        "expected Gini ~ 0 for equal distribution (got {})",
        report_equal.gini
    );
}

#[test]
fn fr_civ_gov_020_mobility_promote_and_demote_events() {
    let mut sim = empty_sim();
    let settlement_id = 12;

    sim.register_household_in_settlement(settlement_id, 42);
    sim.set_household_wealth(42, 100); // Middle

    // baseline
    sim.tick();

    // jump to Elite -> Promoted event
    sim.set_household_wealth(42, 100_000);
    sim.tick();
    let promoted: Vec<&StratificationEvent> = sim
        .last_tick_stratification()
        .iter()
        .filter(|e| {
            e.household_id == 42 && matches!(e.kind, StratificationEventKind::Promoted)
        })
        .collect();
    assert!(
        !promoted.is_empty(),
        "expected a Promoted event when household 42 jumped from Middle to Elite"
    );

    // crash back to Poor -> Demoted event
    sim.set_household_wealth(42, 0);
    sim.tick();
    let demoted: Vec<&StratificationEvent> = sim
        .last_tick_stratification()
        .iter()
        .filter(|e| {
            e.household_id == 42 && matches!(e.kind, StratificationEventKind::Demoted)
        })
        .collect();
    assert!(
        !demoted.is_empty(),
        "expected a Demoted event when household 42 crashed from Elite to Poor"
    );
}

#[test]
fn fr_civ_gov_020_determinism_identical_seeds_identical_reports() {
    fn run_to(seed: u64, settlement_id: u32, wealths: &[(u64, i64)]) -> StratificationReport {
        let mut sim = Sim::with_seed(SimSeed::from_u64(seed));
        for (id, wealth) in wealths {
            sim.register_household_in_settlement(settlement_id, *id);
            sim.set_household_wealth(*id, *wealth);
        }
        for _ in 0..10 {
            sim.tick();
        }
        sim.last_tick_stratification_report(settlement_id)
            .expect("report")
    }

    let wealths = vec![(1, 100), (2, 250), (3, 1_000), (4, 50_000), (5, 0)];

    let report_a = run_to(STRAT_SEED, 99, &wealths);
    let report_b = run_to(STRAT_SEED, 99, &wealths);

    assert_eq!(
        report_a.quantiles.poor, report_b.quantiles.poor,
        "Poor count diverged across identical seeds"
    );
    assert_eq!(
        report_a.quantiles.middle, report_b.quantiles.middle,
        "Middle count diverged across identical seeds"
    );
    assert_eq!(
        report_a.quantiles.rich, report_b.quantiles.rich,
        "Rich count diverged across identical seeds"
    );
    assert_eq!(
        report_a.quantiles.elite, report_b.quantiles.elite,
        "Elite count diverged across identical seeds"
    );
    assert!(
        (report_a.gini - report_b.gini).abs() < 1e-9,
        "Gini diverged across identical seeds (a={}, b={})",
        report_a.gini,
        report_b.gini
    );
}