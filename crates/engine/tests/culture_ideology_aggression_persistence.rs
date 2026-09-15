//! Persistence round-trip tests for the 4 cultural/ideology/aggression/unrest
//! fields added to `WorldState` (FR-CIV-CULTURE-001 + FR-CIV-IDEOLOGY-001 +
//! FR-CIV-AGGRESSION-001 + FR-CIV-UNREST-001).
//!
//! These close the determinism gap by exercising the save-side and load-side
//! mirrors at the byte-for-byte level: insert distinct values, advance a tick
//! to fire `save_state_mirror`, save, load, and assert all 4 fields round-trip
//! exactly.

use civ_engine::{CivSaveBundle, FactionIdeologyState, Simulation};
use tempfile::tempdir;

const SEED: u64 = 0xDEED_BEEF_CAFE;

fn fresh_ideology(values: [f32; 4]) -> FactionIdeologyState {
    FactionIdeologyState {
        values,
        norms: [0.5; 4],
        cooperation: 0.5,
        aggression: 0.0,
        openness: 0.5,
        tradition: 0.5,
    }
}

/// Insert distinguishable values for cluster_cultures / faction_ideologies /
/// faction_aggression / unrest_settlement_gini, advance one tick to fire
/// `save_state_mirror`, archive + reload, and assert all 4 fields round-trip
/// exactly at three layers:
///   (a) `WorldState` side round-trips byte-for-byte
///   (b) live `Simulation` reflects deserialized `WorldState` (load-side mirror)
///   (c) live and state fields are in lockstep after load
#[test]
fn cluster_cultures_ideologies_aggression_unrest_all_persist_through_archive() {
    let mut sim = Simulation::with_seed(SEED);
    sim.advance_ticks(1);

    // 1. cluster_cultures — 2 entries with distinguishable CultureProfile values
    sim.cluster_cultures.insert(
        100,
        civ_agents::culture::CultureProfile::new([0.10, 0.20, 0.30, 0.40]),
    );
    sim.cluster_cultures.insert(
        101,
        civ_agents::culture::CultureProfile::new([0.91, 0.84, 0.77, 0.50]),
    );

    // 2. faction_ideologies — 2 entries (Copy type)
    sim.faction_ideologies
        .insert(1, fresh_ideology([0.30, 0.40, 0.50, 0.60]));
    sim.faction_ideologies
        .insert(2, fresh_ideology([0.70, 0.80, 0.90, 0.10]));

    // 3. faction_aggression — 2 entries
    sim.faction_aggression.insert(7, 0.42);
    sim.faction_aggression.insert(11, 0.89);

    // 4. unrest_settlement_gini — 2 entries
    sim.unrest_settlement_gini.insert(100, 0.31);
    sim.unrest_settlement_gini.insert(101, 0.63);

    // Fire the save-side mirror, then capture the truth.
    sim.advance_ticks(1);
    let expected_live = (
        sim.cluster_cultures.clone(),
        sim.faction_ideologies.clone(),
        sim.faction_aggression.clone(),
        sim.unrest_settlement_gini.clone(),
    );

    let dir = tempdir().expect("tempdir");
    let archive = dir.path().join("culture_ideology_aggression.civsave.zst");
    CivSaveBundle::save_archive(&archive, &sim).expect("save_archive");
    let loaded = CivSaveBundle::load_archive(&archive).expect("load_archive");

    // (a) WorldState side round-trips byte-for-byte
    assert_eq!(
        loaded.state.cluster_cultures, expected_live.0,
        "cluster_cultures WorldState must match pre-save"
    );
    assert_eq!(
        loaded.state.faction_ideologies, expected_live.1,
        "faction_ideologies WorldState must match pre-save"
    );
    assert_eq!(
        loaded.state.faction_aggression, expected_live.2,
        "faction_aggression WorldState must match pre-save"
    );
    assert_eq!(
        loaded.state.unrest_settlement_gini, expected_live.3,
        "unrest_settlement_gini WorldState must match pre-save"
    );

    // (b) live Simulation reflects deserialized WorldState (load-side mirror)
    assert_eq!(
        loaded.cluster_cultures, expected_live.0,
        "live cluster_cultures must equal pre-save after load"
    );
    assert_eq!(
        loaded.faction_ideologies, expected_live.1,
        "live faction_ideologies must equal pre-save after load"
    );
    assert_eq!(
        loaded.faction_aggression, expected_live.2,
        "live faction_aggression must equal pre-save after load"
    );
    assert_eq!(
        loaded.unrest_settlement_gini, expected_live.3,
        "live unrest_settlement_gini must equal pre-save after load"
    );

    // (c) Lockstep: live and state match each other after load
    assert_eq!(
        loaded.cluster_cultures, loaded.state.cluster_cultures,
        "load-side lockstep: cluster_cultures live == state"
    );
    assert_eq!(
        loaded.faction_ideologies, loaded.state.faction_ideologies,
        "load-side lockstep: faction_ideologies live == state"
    );
    assert_eq!(
        loaded.faction_aggression, loaded.state.faction_aggression,
        "load-side lockstep: faction_aggression live == state"
    );
    assert_eq!(
        loaded.unrest_settlement_gini, loaded.state.unrest_settlement_gini,
        "load-side lockstep: unrest_settlement_gini live == state"
    );
}

/// Same-seed determinism: the 4 fields are deterministic given the same seed
/// and the same number of ticks.
#[test]
fn same_seed_yields_identical_culture_ideology_aggression_state() {
    let mut a = Simulation::with_seed(SEED);
    let mut b = Simulation::with_seed(SEED);
    a.advance_ticks(1);
    b.advance_ticks(1);
    assert_eq!(
        a.cluster_cultures, b.cluster_cultures,
        "same-seed sims must share cluster_cultures"
    );
    assert_eq!(
        a.faction_ideologies, b.faction_ideologies,
        "same-seed sims must share faction_ideologies"
    );
    assert_eq!(
        a.faction_aggression, b.faction_aggression,
        "same-seed sims must share faction_aggression"
    );
    assert_eq!(
        a.unrest_settlement_gini, b.unrest_settlement_gini,
        "same-seed sims must share unrest_settlement_gini"
    );
}