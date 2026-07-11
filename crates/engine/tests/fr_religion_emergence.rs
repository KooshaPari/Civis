//! FR-CIV-RELIGION emergent-belief acceptance contracts.

use std::collections::BTreeMap;

use civ_agents::PSYCHE_DIM;
use civ_engine::{
    cohesion_delta, diplomacy_conflict_threshold, diplomacy_peace_threshold,
    institution_belief_signal, institution_divergence_boost, Simulation,
};

#[test]
fn religion_diplomacy_coupling_raises_the_peace_threshold() {
    let low = diplomacy_peace_threshold(0, 0, 0, false);
    let high = diplomacy_peace_threshold(500_000, 100_000, 0, true);

    assert!(
        high > low,
        "belief, cohesion, and patronage must raise peace"
    );
}

#[test]
fn diplomacy_belief_and_unrest_oppose_each_other() {
    let base = diplomacy_conflict_threshold(0, 0);
    let faithful = diplomacy_conflict_threshold(500_000, 0);
    let restless = diplomacy_conflict_threshold(0, 500_000);

    assert!(faithful > base);
    assert!(restless < base);
}

#[test]
fn cohesion_delta_balances_belief_against_unrest() {
    assert!(cohesion_delta(10_000, 0) > 0);
    assert!(cohesion_delta(0, 500) < 0);
    assert!(cohesion_delta(10_000, 500) < cohesion_delta(10_000, 0));
}

#[test]
fn institution_belief_signal_includes_cluster_doctrine() {
    let mut clusters = BTreeMap::new();
    clusters.insert(1_u64, [0.9; PSYCHE_DIM]);

    assert!(institution_belief_signal(1_000, &clusters) > 1_000);
}

#[test]
fn institution_divergence_boost_is_monotonic() {
    let base = 10_000_u64;

    assert_eq!(institution_divergence_boost(base, 0.0), base);
    assert!(institution_divergence_boost(base, 0.2) > base);
    assert!(institution_divergence_boost(base, 0.8) > institution_divergence_boost(base, 0.2));
}

#[test]
fn seeded_religion_inputs_are_deterministic_across_ticks() {
    let mut first = Simulation::with_seed(77_777);
    let mut second = Simulation::with_seed(77_777);
    first.advance_ticks(32);
    second.advance_ticks(32);

    assert_eq!(first.snapshot().tick, second.snapshot().tick);
    assert_eq!(first.belief(), second.belief());
    assert_eq!(first.has_religious_patron(), second.has_religious_patron());
}
