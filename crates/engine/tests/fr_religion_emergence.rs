//! FR-CIV-RELIGION emergent belief — acceptance contracts from `fr-emergence-matrix.md`.

use std::collections::BTreeMap;

use civ_agents::{
    isolation_weighted_belief_divergence, max_cluster_belief_divergence, PSYCHE_DIM,
};
use civ_engine::{
    cohesion_delta, diplomacy_conflict_threshold, diplomacy_peace_threshold,
    institution_belief_signal, institution_divergence_boost, Simulation, WorldCoord,
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

// ── Acceptance contracts from fr-emergence-matrix FR-CIV-REL-001..003 ─────────

/// FR-CIV-REL-001 / FR-CIV-RELIGION belief-from-events contract:
/// After sufficient ticks belief must rise above zero (population accrual + event triggers).
#[test]
fn belief_rises_after_warmup_ticks() {
    let mut sim = Simulation::with_seed(42);
    // Start with zero belief.
    sim.state.belief = 0;
    sim.state.population = 10_000;
    run_ticks(&mut sim, 10);
    assert!(
        sim.belief() > 0,
        "belief must accrue from population and events within 10 ticks (got {})",
        sim.belief()
    );
}

/// FR-CIV-REL-001 / belief-from-events: even with no population,
/// disaster events accrue faith.
#[test]
fn belief_rises_after_disaster_event() {
    use civ_engine::disasters::{trigger_disaster, DisasterKind};

    let mut sim = Simulation::with_seed(42);
    sim.state.belief = 0;
    let before = sim.belief();
    trigger_disaster(
        &mut sim,
        DisasterKind::Wildfire,
        WorldCoord { x: 0, y: 0, z: 0 },
    );
    assert!(
        sim.belief() > before,
        "disaster must raise belief (fear breeds faith) (before={before}, after={})",
        sim.belief()
    );
}

/// FR-CIV-REL-003 / temple-tracks-belief contract:
/// At high belief, temple_level rises above zero.
#[test]
fn temple_level_tracks_belief() {
    let mut sim = Simulation::with_seed(42);
    // Prime with very high belief so temple_target > 0.
    sim.state.belief = 50_000;
    sim.state.temple_level = 0;
    // Multiple ticks so institution_step can ratchet up.
    run_ticks(&mut sim, 5);
    assert!(
        sim.state.temple_level > 0,
        "temple_level must rise when belief is high (level={})",
        sim.state.temple_level
    );
}

/// FR-CIV-REL-003 / temple-tracks-belief: higher belief produces higher temple level
/// than lower belief over the same tick count.
#[test]
fn higher_belief_produces_higher_temple_level() {
    let mut high = Simulation::with_seed(42);
    high.state.belief = 100_000;
    high.state.temple_level = 0;

    let mut low = Simulation::with_seed(42);
    low.state.belief = 5_000;
    low.state.temple_level = 0;

    run_ticks(&mut high, 10);
    run_ticks(&mut low, 10);

    assert!(
        high.state.temple_level >= low.state.temple_level,
        "higher belief must produce >= temple level (high={}, low={})",
        high.state.temple_level,
        low.state.temple_level
    );
}

/// Cluster divergence acceptance contract:
/// `isolation_weighted_belief_divergence` returns 0 when contact is full,
/// and returns `belief_distance` when fully isolated.
#[test]
fn isolation_weighted_divergence_zero_at_full_contact() {
    let mut centroids: BTreeMap<u64, [f32; PSYCHE_DIM]> = BTreeMap::new();
    centroids.insert(0, [0.1, 0.2, 0.3, 0.4]);
    centroids.insert(1, [0.9, 0.8, 0.7, 0.6]);

    let mut full_contact: BTreeMap<(u64, u64), f32> = BTreeMap::new();
    full_contact.insert((0, 1), 1.0);

    assert!(
        isolation_weighted_belief_divergence(&centroids, &full_contact).abs() < f32::EPSILON,
        "full contact must collapse divergence to 0"
    );
}

/// Cluster divergence: fully isolated clusters return the raw belief_distance.
#[test]
fn isolation_weighted_divergence_max_at_zero_contact() {
    let c0: [f32; PSYCHE_DIM] = [0.1, 0.2, 0.3, 0.4];
    let c1: [f32; PSYCHE_DIM] = [0.9, 0.8, 0.7, 0.6];

    let mut centroids: BTreeMap<u64, [f32; PSYCHE_DIM]> = BTreeMap::new();
    centroids.insert(0, c0);
    centroids.insert(1, c1);

    let no_contact: BTreeMap<(u64, u64), f32> = BTreeMap::new();

    let divergence = isolation_weighted_belief_divergence(&centroids, &no_contact);
    let raw = max_cluster_belief_divergence(&[c0, c1]);
    assert!(
        (divergence - raw).abs() < f32::EPSILON,
        "zero contact must return raw belief_distance (got {divergence}, expected {raw})"
    );
}

/// Cluster divergence: divergence rises as contact drops from full to zero.
#[test]
fn isolation_divergence_increases_with_less_contact() {
    let c0: [f32; PSYCHE_DIM] = [0.0, 0.0, 0.0, 0.0];
    let c1: [f32; PSYCHE_DIM] = [1.0, 1.0, 1.0, 1.0];

    let mut centroids: BTreeMap<u64, [f32; PSYCHE_DIM]> = BTreeMap::new();
    centroids.insert(0, c0);
    centroids.insert(1, c1);

    let mut high_contact: BTreeMap<(u64, u64), f32> = BTreeMap::new();
    high_contact.insert((0, 1), 0.8);

    let mut low_contact: BTreeMap<(u64, u64), f32> = BTreeMap::new();
    low_contact.insert((0, 1), 0.2);

    let div_high = isolation_weighted_belief_divergence(&centroids, &high_contact);
    let div_low = isolation_weighted_belief_divergence(&centroids, &low_contact);
    assert!(
        div_low > div_high,
        "lower contact must produce higher divergence (low={div_low}, high={div_high})"
    );
}

/// institution_divergence_boost monotonically amplifies signal with divergence.
#[test]
fn institution_divergence_boost_amplifies_signal() {
    let base = 10_000_u64;
    let boosted_high = institution_divergence_boost(base, 0.8);
    let boosted_low = institution_divergence_boost(base, 0.2);
    let no_boost = institution_divergence_boost(base, 0.0);

    assert_eq!(no_boost, base, "zero divergence should not change signal");
    assert!(
        boosted_low > no_boost,
        "low divergence should amplify signal (got {boosted_low})"
    );
    assert!(
        boosted_high > boosted_low,
        "high divergence should amplify more (high={boosted_high}, low={boosted_low})"
    );
}

/// Determinism per seed — events path.
/// Two sims with same seed diverge identically (FR-CIV-RELIGION acceptance contract).
#[test]
fn belief_and_temple_deterministic_per_seed() {
    let mut a = Simulation::with_seed(77_777);
    let mut b = Simulation::with_seed(77_777);
    run_ticks(&mut a, 60);
    run_ticks(&mut b, 60);
    assert_eq!(
        a.belief(),
        b.belief(),
        "belief must be deterministic for same seed"
    );
    assert_eq!(
        a.state.temple_level,
        b.state.temple_level,
        "temple_level must be deterministic for same seed"
    );
    assert_eq!(
        a.has_religious_patron(),
        b.has_religious_patron(),
        "patron veneration must be deterministic for same seed"
    );
}
