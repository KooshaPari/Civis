//! Persistence round-trip tests for the 3 era/emergence/significance fields
//! added to `WorldState` (FR-CIV-ERA-001 + FR-CIV-EMERGENCE-001 +
//! FR-CIV-LEGENDS-001).
//!
//! Covers: FR-CIV-ERA-001
//!
//! These close the determinism gap by exercising the save-side and load-side
//! mirrors at the byte-for-byte level: insert distinct values, advance a tick
//! to fire `save_state_mirror`, save, load, and assert all 3 fields round-trip
//! exactly.

use civ_engine::{CivSaveBundle, EmergenceSample, Simulation, SignificanceAccumulator, SignificanceConfig};
use civ_emergence_metrics::dashboard::EmergenceDashboard;
use civ_legends::ids::{Epoch, LegendEntityId};
use civ_legends::model::{EventKind, Role};
use tempfile::tempdir;

const SEED: u64 = 0xCAFE_BABE_0001;

/// Construct a non-default `EmergenceSample` with distinguishable values.
fn sample() -> EmergenceSample {
    EmergenceSample {
        tick: 42,
        entropy_bits: 3.17,
        entropy_norm: 0.75,
        structure_count: Some(7),
        structure_largest: Some(1024),
        structure_foreground: Some(4096),
        histogram_total: 100_000,
        histogram_populated_bins: 12,
        sample_dur_us: 500,
        dashboard: EmergenceDashboard {
            cluster_entropy: 0.65,
            ideology_homophily: 0.42,
            sentience_fraction: 0.13,
            psyche_stability: 0.88,
            diplomacy_tension: 0.31,
        },
        branching_sigma: 0.97,
        branching_sigma_score: 0.85,
        branching_window: 100,
        avalanches_closed: 5,
        branching_regime: civ_engine::BranchingRegime::EdgeOfChaos,
        power_law_alpha: 2.5,
        novelty_rate: 0.17,
        mi_material_faction_norm: Some(0.42),
    }
}

/// Populate the significance accumulator with a few events so the
/// round-trip exercises non-trivial serialized content.
fn populate_significance(acc: &mut SignificanceAccumulator) {
    let config = SignificanceConfig {
        decay_rate: 0.95,
        cluster_window: 5,
        cluster_bonus: 0.2,
        max_significance: 100.0,
    };
    acc.record_event(
        LegendEntityId(1),
        Epoch(10),
        &EventKind::Battle,
        &[Role::Leader],
        0.8,
        &config,
    );
    acc.record_event(
        LegendEntityId(2),
        Epoch(11),
        &EventKind::Discovery,
        &[Role::Founder, Role::Witness],
        0.5,
        &config,
    );
}

/// Insert distinguishable values for era_progression / emergence_sample /
/// significance, advance one tick to fire `save_state_mirror`, archive + reload,
/// and assert all 3 fields round-trip exactly at three layers:
///   (a) `WorldState` side round-trips byte-for-byte
///   (b) live `Simulation` reflects deserialized `WorldState` (load-side mirror)
///   (c) live and state fields are in lockstep after load
#[test]
fn era_emergence_significance_all_persist_through_archive() {
    let mut sim = Simulation::with_seed(SEED);
    sim.advance_ticks(1);

    // 1. era_progression — insert faction ages
    use civ_engine::era::CivAge;
    sim.era_progression.faction_ages.insert(1, CivAge::Bronze);
    sim.era_progression.faction_ages.insert(2, CivAge::Iron);
    sim.era_progression.faction_ages.insert(3, CivAge::Classical);

    // 2. emergence_sample — set a non-default sample
    sim.emergence_sample = Some(sample());

    // 3. significance — record events for 2 entities
    populate_significance(&mut sim.significance);

    // Fire the save-side mirror, then capture the truth.
    sim.advance_ticks(1);
    let expected_era = sim.era_progression.clone();
    let expected_sample = sim.emergence_sample;
    let expected_sig = sim.significance.clone();

    let dir = tempdir().expect("tempdir");
    let archive = dir.path().join("era_emergence_significance.civsave.zst");
    CivSaveBundle::save_archive(&archive, &sim).expect("save_archive");
    let loaded = CivSaveBundle::load_archive(&archive).expect("load_archive");

    // (a) WorldState side round-trips byte-for-byte
    assert_eq!(
        loaded.state.era_progression, expected_era,
        "era_progression WorldState must match pre-save"
    );
    assert_eq!(
        loaded.state.emergence_sample, expected_sample,
        "emergence_sample WorldState must match pre-save"
    );
    assert_eq!(
        loaded.state.significance, expected_sig,
        "significance WorldState must match pre-save"
    );

    // (b) live Simulation reflects deserialized WorldState (load-side mirror)
    assert_eq!(
        loaded.era_progression, expected_era,
        "live era_progression must equal pre-save after load"
    );
    assert_eq!(
        loaded.emergence_sample, expected_sample,
        "live emergence_sample must equal pre-save after load"
    );
    assert_eq!(
        loaded.significance, expected_sig,
        "live significance must equal pre-save after load"
    );

    // (c) Lockstep: live and state match each other after load
    assert_eq!(
        loaded.era_progression, loaded.state.era_progression,
        "load-side lockstep: era_progression live == state"
    );
    assert_eq!(
        loaded.emergence_sample, loaded.state.emergence_sample,
        "load-side lockstep: emergence_sample live == state"
    );
    assert_eq!(
        loaded.significance, loaded.state.significance,
        "load-side lockstep: significance live == state"
    );
}

/// Same-seed determinism: the 3 fields are deterministic given the same seed
/// and the same number of ticks.
#[test]
fn same_seed_yields_identical_era_emergence_significance_state() {
    let mut a = Simulation::with_seed(SEED);
    let mut b = Simulation::with_seed(SEED);
    a.advance_ticks(1);
    b.advance_ticks(1);

    // Apply identical mutations
    use civ_engine::era::CivAge;
    a.era_progression.faction_ages.insert(1, CivAge::Medieval);
    b.era_progression.faction_ages.insert(1, CivAge::Medieval);

    a.emergence_sample = Some(sample());
    b.emergence_sample = Some(sample());

    populate_significance(&mut a.significance);
    populate_significance(&mut b.significance);

    assert_eq!(
        a.era_progression, b.era_progression,
        "same-seed sims must share era_progression"
    );
    assert_eq!(
        a.emergence_sample, b.emergence_sample,
        "same-seed sims must share emergence_sample"
    );
    assert_eq!(
        a.significance, b.significance,
        "same-seed sims must share significance"
    );
}
