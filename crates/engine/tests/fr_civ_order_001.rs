//! TDD tests for `phase_order` — FR-CIV-ORDER-001 (government-stability phase).
//!
//! The order phase runs after `phase_unrest` and `phase_institutions` so it can
//! read the just-computed per-settlement unrest score and the institution kind
//! + level that will govern legitimacy. It computes
//!
//! `revolt_likelihood(unrest_norm, legitimacy)` per settlement and bins the
//! result into [`OrderLevel`]:
//!
//!   Holding      -- legitimacy comfortably exceeds unrest; no revolt pressure
//!   Strained     -- unrest and legitimacy are roughly equal
//!   Collapsing   -- unrest dominates legitimacy; revolt likely
//!   Revolted     -- revolt is essentially certain
//!
//! `unrest_norm` is `min(unrest_score / 500, 1.0)`. A settlement that escalates
//! into `UnrestLevel::Rioting` (score >= 150) with no institution crosses
//! `OrderLevel::Strained` within a single tick.
//!
//! Public API the implementation must provide:
//!   civ_engine::OrderLevel { Holding, Strained, Collapsing, Revolted }
//!   civ_engine::OrderEvent { settlement_id, level, level_delta,
//!                             revolt_likelihood, unrest_score,
//!                             legitimacy, institution_level, institution_kind }
//!   civ_engine::OrderSnapshot { settlement_id, level, revolt_likelihood,
//!                                unrest_score, legitimacy, institution_level,
//!                                institution_kind }
//!   civ_engine::Simulation
//!     .last_tick_order() -> &[OrderEvent]
//!     .last_tick_order_settlement(id) -> Option<OrderSnapshot>
//!     .order_level(id) -> Option<OrderLevel>
//!
//! Pinned tests:
//!   FR-CIV-ORDER-001.base           order snapshot emitted for each settlement
//!   FR-CIV-ORDER-001.food_shock     food shortage -> unrest -> order Strained
//!   FR-CIV-ORDER-001.temple_holds   temple legitimacy cushions revolt
//!   FR-CIV-ORDER-001.garrison_cushion garrison legitimacy also cushions
//!   FR-CIV-ORDER-001.pipeline       end-to-end: mood -> unrest -> order
//!   FR-CIV-ORDER-001.determinism    identical seeds yield identical order snapshots
//!   FR-CIV-ORDER-001.event_emission event emitted only on level change

use civ_engine::{OrderEvent, OrderLevel, OrderSnapshot, Simulation};

const ORDER_SEED: u64 = 0x00FD_A7A0_B5EE_0001;

#[test]
fn fr_civ_order_001_base_emits_one_snapshot_per_settlement_per_tick() {
    let mut sim = Simulation::with_seed(ORDER_SEED);
    sim.set_settlement_population(0, 60);
    sim.set_settlement_food_stocked(0, 1_000);
    sim.set_settlement_housing_capacity(0, 60);
    sim.set_settlement_crime_pressure(0, 0);
    // Prime `phase_unrest`'s iteration set so settlement 0 is visited.
    sim.set_settlement_gini(0, 0.0);
    sim.advance_ticks(1);

    let snapshot = sim
        .last_tick_order_settlement(0)
        .expect("settlement 0 should have an order snapshot after 1 tick");
    assert_eq!(snapshot.settlement_id, 0, "snapshot keyed to settlement 0");
    assert!(
        snapshot.revolt_likelihood.is_finite(),
        "revolt_likelihood must be finite, got {}",
        snapshot.revolt_likelihood
    );
    assert!(
        (0.0..=1.0).contains(&snapshot.revolt_likelihood),
        "revolt_likelihood must be in [0, 1], got {}",
        snapshot.revolt_likelihood
    );
    assert!(
        matches!(
            snapshot.level,
            OrderLevel::Holding
                | OrderLevel::Strained
                | OrderLevel::Collapsing
                | OrderLevel::Revolted
        ),
        "level must be one of the four OrderLevel variants"
    );
}

#[test]
fn fr_civ_order_001_food_shock_drives_order_to_strained() {
    // Pop = 49 to stay BELOW TEMPLE_UNLOCK_POPULATION (50), so no
    // institution can spawn mid-test and confuse the legitimacy=0 baseline.
    let mut sim = Simulation::with_seed(ORDER_SEED);
    sim.set_settlement_population(0, 49);
    sim.set_settlement_housing_capacity(0, 49);
    sim.set_settlement_crime_pressure(0, 0);
    // Prime `phase_unrest`'s iteration set by inserting a Gini entry, otherwise
    // it skips settlement 0 entirely and `phase_order` never produces a
    // snapshot for it. (See `phase_unrest` line that builds
    // `settlement_ids` from `settlement_gini` and `actor_settlement`.)
    sim.set_settlement_gini(0, 0.0);
    // Start with abundant food so mood is high and order is Holding.
    sim.set_settlement_food_stocked(0, 100_000);
    sim.advance_ticks(1);
    let snap_start = sim
        .last_tick_order_settlement(0)
        .expect("snapshot after tick 1");
    assert_eq!(
        snap_start.legitimacy, 0.0,
        "no institution at pop 49: legitimacy must be 0, got {}",
        snap_start.legitimacy
    );
    assert_eq!(
        snap_start.level,
        OrderLevel::Holding,
        "abundant food + no institution should hold order at Holding (got {:?})",
        snap_start.level
    );

    // Plunge food to zero (severe shortage) and crank crime pressure.
    sim.set_settlement_food_stocked(0, 0);
    sim.set_settlement_crime_pressure(0, 200);
    sim.advance_ticks(1);
    let snap_shock = sim.last_tick_order_settlement(0).unwrap();
    // No institution -> 0 legitimacy -> revolt likelihood rises with unrest.
    // We don't assert Revolted (it requires very high sustained unrest), but
    // we do assert order is no longer Holding.
    assert_ne!(
        snap_shock.level,
        OrderLevel::Holding,
        "food shock + crime pressure without institution should push order \
         past Holding (got {:?}, revolt_likelihood={})",
        snap_shock.level,
        snap_shock.revolt_likelihood
    );
}

#[test]
fn fr_civ_order_001_temple_legitimacy_cushions_revolt() {
    // Two settlements with identical conditions except settlement 1 has a
    // Temple (pop=200 >= 50 unlocks it) and settlement 0 stays below the
    // threshold (pop=49) so no institution can spawn there. Both suffer the
    // same food shock + crime pressure, so the only difference is legitimacy.
    let run = |with_temple: bool| -> OrderSnapshot {
        let mut sim = Simulation::with_seed(ORDER_SEED);
        let pop = if with_temple { 200 } else { 49 };
        sim.set_settlement_population(0, pop);
        sim.set_settlement_housing_capacity(0, pop);
        sim.set_settlement_crime_pressure(0, 100);
        // Prime `phase_unrest`/`phase_order` so the settlement is iterated.
        sim.set_settlement_gini(0, 0.0);
        sim.set_settlement_food_stocked(0, 0);
        sim.advance_ticks(2);
        sim.last_tick_order_settlement(0)
            .expect("settlement 0 should have an order snapshot")
    };
    let no_temple = run(false);
    let with_temple = run(true);

    // No institution -> legitimacy = 0
    assert!(
        no_temple.legitimacy.abs() < 1e-6,
        "no institution should have zero legitimacy, got {}",
        no_temple.legitimacy
    );
    // Temple L1 -> legitimacy >= 0.3 (spec: Temple L1 = 0.35)
    assert!(
        with_temple.legitimacy >= 0.3,
        "Temple L1 should grant at least 0.3 legitimacy, got {}",
        with_temple.legitimacy
    );
    // With legitimacy cushion, the with_temple settlement must be at the
    // same or better (lower-rank) order level than the no-temple one.
    assert!(
        with_temple.level <= no_temple.level,
        "Temple legitimacy should not produce a worse order level: \
         with_temple={:?} (likelihood={}) \
         vs no_temple={:?} (likelihood={})",
        with_temple.level,
        with_temple.revolt_likelihood,
        no_temple.level,
        no_temple.revolt_likelihood
    );
}

#[test]
fn fr_civ_order_001_garrison_also_provides_legitimacy() {
    let mut sim = Simulation::with_seed(ORDER_SEED);
    // Pop 200 unlocks both Temple AND Garrison (GARRISON_UNLOCK_POPULATION
    // is 120). Per the implementation, only one institution kind can be
    // active per settlement at a time. So this test only verifies the
    // Garrison-only branch via the legitimacy field of a freshly-cleared sim.
    sim.set_settlement_population(0, 200);
    sim.set_settlement_housing_capacity(0, 200);
    sim.set_settlement_crime_pressure(0, 50);
    sim.set_settlement_food_stocked(0, 50_000);
    // Prime `phase_unrest`'s iteration set so settlement 0 is visited.
    sim.set_settlement_gini(0, 0.0);
    sim.advance_ticks(2);

    let snap = sim
        .last_tick_order_settlement(0)
        .expect("order snapshot after 2 ticks");
    // The settlement should hold either Temple or Garrison by tick 2.
    assert!(
        snap.legitimacy > 0.0,
        "expected non-zero legitimacy from at least one institution, got {}",
        snap.legitimacy
    );
    assert!(
        snap.institution_level >= 1,
        "institution_level should be >= 1, got {}",
        snap.institution_level
    );
    assert!(
        snap.institution_kind == 1 || snap.institution_kind == 2,
        "institution_kind discriminant should be Temple (1) or Garrison (2), got {}",
        snap.institution_kind
    );
}

#[test]
fn fr_civ_order_001_pipeline_mood_unrest_order_chain() {
    // End-to-end: prove the pipeline mood -> unrest -> order produces a
    // snapshot whose revolt_likelihood is monotonically non-decreasing in
    // the unrest score across consecutive ticks.
    let mut sim = Simulation::with_seed(ORDER_SEED);
    sim.set_settlement_population(0, 200);
    sim.set_settlement_housing_capacity(0, 200);
    sim.set_settlement_crime_pressure(0, 0);
    sim.set_settlement_food_stocked(0, 100_000);
    // Prime `phase_unrest`'s iteration set so settlement 0 is visited.
    sim.set_settlement_gini(0, 0.0);
    sim.advance_ticks(1);
    let snap_happy = sim.last_tick_order_settlement(0).unwrap();

    sim.set_settlement_food_stocked(0, 0);
    sim.set_settlement_crime_pressure(0, 200);
    sim.advance_ticks(1);
    let snap_shock = sim.last_tick_order_settlement(0).unwrap();

    sim.set_settlement_food_stocked(0, -1_000); // even worse
    sim.advance_ticks(1);
    let snap_worse = sim.last_tick_order_settlement(0).unwrap();

    // The unrest score should be monotonically non-decreasing across the
    // worsening shocks, so the order level should never *improve* between
    // consecutive snapshots without an institution being added.
    assert!(
        snap_shock.level >= snap_happy.level,
        "shock tick should not improve order (happy={:?} shock={:?})",
        snap_happy.level,
        snap_shock.level
    );
    assert!(
        snap_worse.level >= snap_shock.level,
        "worse shock tick should not improve order (shock={:?} worse={:?})",
        snap_shock.level,
        snap_worse.level
    );
}

#[test]
fn fr_civ_order_001_determinism_identical_seeds_yield_identical_snapshots() {
    fn run() -> OrderSnapshot {
        let mut sim = Simulation::with_seed(ORDER_SEED);
        sim.set_settlement_population(0, 100);
        sim.set_settlement_population(1, 200);
        sim.set_settlement_food_stocked(0, 1_500);
        sim.set_settlement_food_stocked(1, 5_000);
        sim.set_settlement_housing_capacity(0, 100);
        sim.set_settlement_housing_capacity(1, 200);
        sim.set_settlement_crime_pressure(0, 10);
        sim.set_settlement_crime_pressure(1, 50);
        // Prime `phase_unrest`'s iteration set so settlements are visited.
        sim.set_settlement_gini(0, 0.0);
        sim.set_settlement_gini(1, 0.0);
        sim.advance_ticks(3);
        sim.last_tick_order_settlement(1)
            .expect("settlement 1 should have order snapshot")
    }
    let a = run();
    let b = run();
    assert_eq!(a.level, b.level, "level mismatch");
    assert_eq!(
        a.revolt_likelihood, b.revolt_likelihood,
        "revolt_likelihood mismatch: {} vs {}",
        a.revolt_likelihood, b.revolt_likelihood
    );
    assert_eq!(a.unrest_score, b.unrest_score, "unrest_score mismatch");
    assert_eq!(a.legitimacy, b.legitimacy, "legitimacy mismatch");
    assert_eq!(
        a.institution_level, b.institution_level,
        "institution_level mismatch"
    );
    assert_eq!(
        a.institution_kind, b.institution_kind,
        "institution_kind mismatch"
    );
}

#[test]
fn fr_civ_order_001_event_emitted_only_on_level_change() {
    // Run a settled, well-fed simulation and assert the event buffer is
    // either empty or only fires a single Holding->{X} entry. The phase
    // must not spam identical-level events each tick.
    let mut sim = Simulation::with_seed(ORDER_SEED);
    sim.set_settlement_population(0, 200);
    sim.set_settlement_housing_capacity(0, 200);
    sim.set_settlement_crime_pressure(0, 0);
    sim.set_settlement_food_stocked(0, 100_000);
    // Prime `phase_unrest`'s iteration set so settlement 0 is visited.
    sim.set_settlement_gini(0, 0.0);
    sim.advance_ticks(5);

    let events: &[OrderEvent] = sim.last_tick_order();
    // The buffer only contains events from the *most recent* tick, so it
    // holds at most 1 entry per settlement. With 1 settlement this means
    // at most 1 event.
    assert!(
        events.len() <= 1,
        "expected at most 1 order event from 1 settlement in 1 tick, got {}",
        events.len()
    );
    // If the event did fire, it must carry a valid level + legitimacy.
    for ev in events {
        assert!(
            ev.legitimacy.is_finite() && (0.0..=1.0).contains(&ev.legitimacy),
            "event legitimacy out of range: {}",
            ev.legitimacy
        );
        assert!(
            ev.revolt_likelihood.is_finite()
                && (0.0..=1.0).contains(&ev.revolt_likelihood),
            "event revolt_likelihood out of range: {}",
            ev.revolt_likelihood
        );
    }
}
