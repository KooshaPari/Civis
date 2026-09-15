//! Determinism regression tests for the era/ideology/aggression live state.
//!
//! These fields (`cluster_cultures`, `faction_ideologies`, `faction_aggression`)
//! live on `Simulation` only and are not part of the persisted WorldState
//! directly. The determinism contract is: same seed -> same live state.

use civ_engine::Simulation;

const SEED: u64 = 0xCAFE_BEEF;

/// Same-seed determinism: identical seed + identical tick advancement ->
/// identical live-state clusters.
#[test]
fn same_seed_yields_identical_culture_ideology_aggression() {
    let mut a = Simulation::with_seed(SEED);
    let mut b = Simulation::with_seed(SEED);
    a.advance_ticks(4);
    b.advance_ticks(4);
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
}

/// Live-only fields are populated identically across same-seed sims at the
/// same tick index, so replay from a deterministic seed reproduces them.
#[test]
fn same_seed_replay_reproduces_live_state_at_same_tick() {
    let advance_count = 8;
    let mut a = Simulation::with_seed(SEED);
    let mut b = Simulation::with_seed(SEED);
    a.advance_ticks(advance_count);
    b.advance_ticks(advance_count);
    assert_eq!(
        a.faction_aggression, b.faction_aggression,
        "aggression must match across same-seed replays at the same tick"
    );
    assert_eq!(
        a.cluster_cultures, b.cluster_cultures,
        "cluster_cultures must match across same-seed replays at the same tick"
    );
    assert_eq!(
        a.faction_ideologies, b.faction_ideologies,
        "faction_ideologies must match across same-seed replays at the same tick"
    );
}
