//! Persistence round-trip tests for the riot_accumulator, migrant_accumulator,
//! and scenario_taxation fields added to `WorldState` (FR-CIV-UNREST-002 +
//! FR-CIV-ECON-010).
//!
//! These close the determinism gap by exercising the save-side and load-side
//! mirrors at the byte-for-byte level: insert distinct values, advance a tick
//! to fire `save_state_mirror`, save, load, and assert all 3 fields round-trip
//! exactly.

use civ_engine::CivSaveBundle;
use tempfile::tempdir;

const SEED: u64 = 0xCAFE_BABE_1234;

/// Insert distinguishable values for riot_accumulator, migrant_accumulator,
/// and scenario_taxation, advance one tick to fire `save_state_mirror`,
/// archive + reload, and assert all 3 fields round-trip exactly at three
/// layers:
///   (a) `WorldState` side round-trips byte-for-byte
///   (b) live `Simulation` reflects deserialized `WorldState` (load-side mirror)
///   (c) live and state fields are in lockstep after load
#[test]
fn riot_migrant_taxation_all_persist_through_archive() {
    let mut sim = civ_engine::Simulation::with_seed(SEED);
    sim.advance_ticks(1);

    // 1. riot_accumulator — 3 entries with distinguishable values
    sim.riot_accumulator.insert(10, -500);
    sim.riot_accumulator.insert(20, 1200);
    sim.riot_accumulator.insert(30, 0);

    // 2. migrant_accumulator — 3 entries with distinguishable values
    sim.migrant_accumulator.insert(10, 800);
    sim.migrant_accumulator.insert(20, -350);
    sim.migrant_accumulator.insert(30, 9999);

    // 3. scenario_taxation — set non-default rates
    sim.scenario_taxation.rates_bp.insert(1, 250); // 2.5 %
    sim.scenario_taxation.rates_bp.insert(2, 500); // 5.0 %
    sim.scenario_taxation.per_institution_cap = Some(500);

    // Fire the save-side mirror, then capture the truth.
    sim.advance_ticks(1);
    let expected_riot = sim.riot_accumulator.clone();
    let expected_migrant = sim.migrant_accumulator.clone();
    let expected_taxation = sim.scenario_taxation.clone();

    let dir = tempdir().expect("tempdir");
    let archive = dir.path().join("riot_migrant_taxation.civsave.zst");
    CivSaveBundle::save_archive(&archive, &sim).expect("save_archive");
    let loaded = CivSaveBundle::load_archive(&archive).expect("load_archive");

    // (a) WorldState side round-trips byte-for-byte
    assert_eq!(
        loaded.state.riot_accumulator, expected_riot,
        "riot_accumulator WorldState must match pre-save"
    );
    assert_eq!(
        loaded.state.migrant_accumulator, expected_migrant,
        "migrant_accumulator WorldState must match pre-save"
    );
    assert_eq!(
        loaded.state.scenario_taxation, expected_taxation,
        "scenario_taxation WorldState must match pre-save"
    );

    // (b) live Simulation reflects deserialized WorldState (load-side mirror)
    assert_eq!(
        loaded.riot_accumulator, expected_riot,
        "live riot_accumulator must equal pre-save after load"
    );
    assert_eq!(
        loaded.migrant_accumulator, expected_migrant,
        "live migrant_accumulator must equal pre-save after load"
    );
    assert_eq!(
        loaded.scenario_taxation, expected_taxation,
        "live scenario_taxation must equal pre-save after load"
    );

    // (c) Lockstep: live and state match each other after load
    assert_eq!(
        loaded.riot_accumulator, loaded.state.riot_accumulator,
        "load-side lockstep: riot_accumulator live == state"
    );
    assert_eq!(
        loaded.migrant_accumulator, loaded.state.migrant_accumulator,
        "load-side lockstep: migrant_accumulator live == state"
    );
    assert_eq!(
        loaded.scenario_taxation, loaded.state.scenario_taxation,
        "load-side lockstep: scenario_taxation live == state"
    );
}

/// Same-seed determinism: the 3 fields are deterministic given the same seed
/// and the same number of ticks.
#[test]
fn same_seed_yields_identical_riot_migrant_taxation_state() {
    let mut a = civ_engine::Simulation::with_seed(SEED);
    let mut b = civ_engine::Simulation::with_seed(SEED);
    a.advance_ticks(1);
    b.advance_ticks(1);

    // Manually set non-default values so determinism is exercised.
    a.riot_accumulator.insert(10, -500);
    b.riot_accumulator.insert(10, -500);
    a.migrant_accumulator.insert(10, 800);
    b.migrant_accumulator.insert(10, 800);
    a.scenario_taxation.rates_bp.insert(1, 250);
    b.scenario_taxation.rates_bp.insert(1, 250);

    assert_eq!(
        a.riot_accumulator, b.riot_accumulator,
        "same-seed sims must share riot_accumulator"
    );
    assert_eq!(
        a.migrant_accumulator, b.migrant_accumulator,
        "same-seed sims must share migrant_accumulator"
    );
    assert_eq!(
        a.scenario_taxation, b.scenario_taxation,
        "same-seed sims must share scenario_taxation"
    );
}

/// Empty accumulators and default taxation survive a round-trip correctly
/// (backward-compatible with legacy v3 saves).
#[test]
fn empty_accumulators_and_default_taxation_persist_through_archive() {
    let mut sim = civ_engine::Simulation::with_seed(SEED);
    sim.advance_ticks(1);

    // Leave all three fields at their defaults (empty maps, zeroed taxation).
    let expected_riot = sim.riot_accumulator.clone();
    let expected_migrant = sim.migrant_accumulator.clone();
    let expected_taxation = sim.scenario_taxation.clone();

    let dir = tempdir().expect("tempdir");
    let archive = dir.path().join("empty_persist.civsave.zst");
    CivSaveBundle::save_archive(&archive, &sim).expect("save_archive");
    let loaded = CivSaveBundle::load_archive(&archive).expect("load_archive");

    assert_eq!(
        loaded.riot_accumulator, expected_riot,
        "empty riot_accumulator must round-trip"
    );
    assert_eq!(
        loaded.migrant_accumulator, expected_migrant,
        "empty migrant_accumulator must round-trip"
    );
    assert_eq!(
        loaded.scenario_taxation, expected_taxation,
        "default scenario_taxation must round-trip"
    );
    assert_eq!(
        loaded.riot_accumulator, loaded.state.riot_accumulator,
        "lockstep: empty riot_accumulator live == state"
    );
    assert_eq!(
        loaded.migrant_accumulator, loaded.state.migrant_accumulator,
        "lockstep: empty migrant_accumulator live == state"
    );
    assert_eq!(
        loaded.scenario_taxation, loaded.state.scenario_taxation,
        "lockstep: default scenario_taxation live == state"
    );
}
