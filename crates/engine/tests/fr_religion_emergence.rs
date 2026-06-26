//! FR-CIV-RELIGION emergent belief — acceptance contracts from `fr-emergence-matrix.md`.

use std::collections::BTreeMap;

use civ_agents::{max_cluster_belief_divergence, PSYCHE_DIM};
use civ_engine::{
    cohesion_delta, diplomacy_conflict_threshold, diplomacy_peace_threshold,
    institution_belief_signal, Simulation,
};

fn run_ticks(sim: &mut Simulation, n: u64) {
    for _ in 0..n {
        sim.tick();
    }
}

/// FR-CIV-RELIGION-002 / emergent-systems-tracelinks coupling 20.
#[test]
fn religion_diplomacy_coupling() {
    let low = diplomacy_peace_threshold(0, 0, 0, false);
    let high = diplomacy_peace_threshold(500_000, 100_000, 0, true);
    assert!(
        high > low,
        "high belief + cohesion + patron must raise peace threshold (low={low}, high={high})"
    );
}

#[test]
fn diplomacy_threshold_belief_raises_peace_cap() {
    let base = diplomacy_conflict_threshold(0, 0);
    let faithful = diplomacy_conflict_threshold(500_000, 0);
    assert!(faithful > base);
}

#[test]
fn diplomacy_belief_and_unrest_oppose() {
    let calm = diplomacy_peace_threshold(100_000, 0, 0, false);
    let restless = diplomacy_peace_threshold(100_000, 0, 100_000, false);
    assert!(calm > restless);
}

#[test]
fn cohesion_delta_balances_belief_against_unrest() {
    assert!(cohesion_delta(10_000, 0) > 0, "belief binds cohesion");
    assert!(cohesion_delta(0, 500) < 0, "unrest frays cohesion");
    assert!(
        cohesion_delta(10_000, 500) < cohesion_delta(10_000, 0),
        "unrest must damp belief binding"
    );
}

#[test]
fn institution_belief_signal_includes_cluster_doctrine() {
    let mut clusters = BTreeMap::new();
    clusters.insert(1_u64, [0.9; PSYCHE_DIM]);
    let boosted = institution_belief_signal(1_000, &clusters);
    assert!(boosted > 1_000, "cluster centroids must amplify temple signal");
}

#[test]
fn cluster_belief_divergence_emerges_from_ticks() {
    let mut sim = Simulation::with_seed(88);
    run_ticks(&mut sim, 250);
    let centroids: Vec<_> = sim.cluster_beliefs().values().copied().collect();
    if centroids.len() >= 2 {
        assert!(
            max_cluster_belief_divergence(&centroids) > 0.0,
            "isolated clusters must develop distinct belief centroids"
        );
    }
}

#[test]
fn legend_and_events_feed_macro_belief_over_warmup() {
    let mut sim = Simulation::with_seed(17);
    let before = sim.belief();
    run_ticks(&mut sim, 400);
    assert!(
        sim.belief() > before || sim.has_religious_patron(),
        "events/legends path must accrue faith or crystallise patron veneration"
    );
}
