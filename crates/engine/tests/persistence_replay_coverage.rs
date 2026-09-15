//! Regression coverage for the cross-lane persistence replay lane.
//!
//! Closes the parent-scorecard gap where the .civsave.zst archive round-trip
//! only verified byte-for-byte state preservation *at the moment of save*.
//! Long-running sessions need to confirm that after a load, ticking the
//! simulation further keeps the loaded state coherent and that two
//! simulations started from the same point but ticked to different
//! further ticks actually diverge.
//!
//! Coverage scope:
//!
//! * `institutions` + `institution_levels_emitted` (FR-CIV-INSTITUTIONS-001)
//! * `build_sites` (FR-CIV-CONSTRUCTION-001)
//! * `econ_focus` (FR-CIV-ECON-FOCUS-001)
//! * `deep_diplomacy.faction_resources` (FR-CIV-DIPLOMACY-001)
//!
//! Each test: build a sim, mutate all the durable fields, save, load, then
//! drive the loaded sim further and assert that the post-tick state is
//! *internally coherent* AND that divergent ticks produce distinct
//! hash-chain roots.

use civ_engine::{CivSaveBundle, Simulation};

/// Helper: write a sim to a temp archive and read it back.
fn roundtrip(sim: &mut Simulation) -> Simulation {
    let dir = tempfile::tempdir().expect("tempdir");
    let archive_path = dir.path().join("replay.civsave.zst");
    CivSaveBundle::save_archive(&archive_path, sim).expect("save_archive");
    CivSaveBundle::load_archive(&archive_path).expect("load_archive")
}

/// Replay determinism: a load followed by N additional ticks must equal a
/// fresh-from-same-seed run that ticked the same total number of ticks.
#[test]
fn replay_after_load_deterministic_to_target_tick() {
    let seed = 0xC1B15_u64;

    let mut pre_save = Simulation::with_seed(seed);
    for _ in 0..16 {
        pre_save.advance_ticks(1);
    }
    let pre_save_hash = pre_save.hash_chain_root();
    let pre_save_tick = pre_save.state.tick;
    assert!(pre_save_tick > 0, "sim must have ticked at least once");

    let mut loaded = roundtrip(&mut pre_save);
    assert_eq!(
        loaded.state.tick, pre_save_tick,
        "loaded tick matches pre-save"
    );

    let extra_ticks: u64 = 8;
    for _ in 0..extra_ticks {
        loaded.advance_ticks(1);
    }
    let post_load_hash = loaded.hash_chain_root();
    let post_load_tick = loaded.state.tick;
    assert_eq!(
        post_load_tick,
        pre_save_tick + extra_ticks,
        "post-load tick = pre-save tick + extra_ticks"
    );
    assert_ne!(
        pre_save_hash, post_load_hash,
        "hash-chain root must change after additional ticks"
    );

    // Fresh sim from same seed, ticked to the same total ticks, must match
    // the loaded+continued hash — proves the save/load is lossless.
    let mut fresh = Simulation::with_seed(seed);
    for _ in 0..post_load_tick {
        fresh.advance_ticks(1);
    }
    let fresh_hash = fresh.hash_chain_root();
    assert_eq!(
        fresh_hash, post_load_hash,
        "fresh-from-seed simulation must match loaded+continued hash at the same tick"
    );
}

/// Institutions + build_sites + econ_focus mutations must roundtrip through
/// save/load, and post-load ticks must not "lose" those mutations to the
/// mirror-side overwrite.
#[test]
fn replay_after_load_preserves_institutions_buildsites_econfocus_mutations() {
    let mut sim = Simulation::with_seed(20260914);
    for _ in 0..4 {
        sim.advance_ticks(1);
    }

    // Seed distinguishable values into each persisted field via the public
    // mutation entry points.
    sim.institutions.insert(11, vec![]);
    sim.build_sites.push(Default::default());
    sim.econ_focus.insert(11, Default::default());

    // The save-side mirror fires inside tick(). Tick once so the manual
    // mutations above are captured into the world-state mirror.
    sim.advance_ticks(1);

    let dir = tempfile::tempdir().expect("tempdir");
    let archive_path = dir.path().join("persist.civsave.zst");
    CivSaveBundle::save_archive(&archive_path, &mut sim).expect("save_archive");

    let mut loaded = CivSaveBundle::load_archive(&archive_path).expect("load_archive");

    // Post-load the persisted side must equal pre-save live.
    assert_eq!(loaded.state.institutions, sim.institutions);
    assert_eq!(loaded.state.build_sites.len(), sim.build_sites.len());
    assert_eq!(loaded.state.econ_focus.len(), sim.econ_focus.len());

    // Drive the loaded sim further and assert the mirror side stays in
    // sync — this catches the bug where a tick's live-to-mirror overwrite
    // would silently reset fields the user just set.
    for _ in 0..8 {
        loaded.advance_ticks(1);
    }
    assert_eq!(
        loaded.state.institutions, loaded.institutions,
        "post-load mirror stays in sync for institutions"
    );
    assert_eq!(
        loaded.state.build_sites, loaded.build_sites,
        "post-load mirror stays in sync for build_sites"
    );
    assert_eq!(
        loaded.state.econ_focus, loaded.econ_focus,
        "post-load mirror stays in sync for econ_focus"
    );
}

/// `state.deep_diplomacy.faction_resources` must remain coherent after a
/// load + N additional ticks.
#[test]
fn replay_after_load_preserves_deep_diplomacy_faction_resources() {
    let mut sim = Simulation::with_seed(0xDEED_u64);
    sim.advance_ticks(1);
    sim.deep_diplomacy.faction_resources.insert(13, 0x1234);
    sim.deep_diplomacy.faction_resources.insert(17, 0x5678);

    // Tick once so the save-side mirror captures the manual faction_resources
    // mutation into state.deep_diplomacy.
    sim.advance_ticks(1);

    let dir = tempfile::tempdir().expect("tempdir");
    let archive_path = dir.path().join("diplo.civsave.zst");
    CivSaveBundle::save_archive(&archive_path, &mut sim).expect("save_archive");
    let mut loaded = CivSaveBundle::load_archive(&archive_path).expect("load_archive");

    // Right after load: live == mirror (load-side mirror already ran).
    assert_eq!(
        loaded.deep_diplomacy.faction_resources, loaded.state.deep_diplomacy.faction_resources,
        "post-load live == mirror for deep_diplomacy"
    );

    // Add a new entry after load and tick; the mirror must adopt it.
    loaded.deep_diplomacy.faction_resources.insert(29, 0xABCD);
    loaded.advance_ticks(1);
    assert_eq!(
        loaded.deep_diplomacy.faction_resources, loaded.state.deep_diplomacy.faction_resources,
        "post-load+tick live == mirror"
    );
    assert!(
        loaded.state.deep_diplomacy.faction_resources.contains_key(&29),
        "post-load+tick insert reaches mirror"
    );

    // Continue ticking without manual mutation; the deep_diplomacy state
    // must remain coherent across multiple ticks.
    let frozen_hash = loaded.hash_chain_root();
    for _ in 0..4 {
        loaded.advance_ticks(1);
    }
    assert_eq!(
        loaded.deep_diplomacy.faction_resources, loaded.state.deep_diplomacy.faction_resources,
        "mirror stays in sync across multiple post-load ticks"
    );
    assert_eq!(
        loaded.deep_diplomacy.faction_resources.get(&29).copied(),
        Some(0xABCD),
        "manually-inserted entry survives multiple ticks"
    );
    assert_ne!(
        loaded.hash_chain_root(),
        frozen_hash,
        "hash-chain root advances with each tick even with no manual mutation"
    );
}

/// Two simulations started from the same seed and ticked to different
/// further points must produce different hash-chain roots — i.e. the
/// replay is *sensitive* to the number of ticks, not a frozen snapshot.
#[test]
fn replay_sensitivity_to_ticks_produces_distinct_hashes() {
    let seed = 0xBEEF_u64;

    let mut a = Simulation::with_seed(seed);
    for _ in 0..4 {
        a.advance_ticks(1);
    }
    let h_a = a.hash_chain_root();

    let mut b = roundtrip(&mut a);
    for _ in 0..2 {
        b.advance_ticks(1);
    }
    let h_b = b.hash_chain_root();

    let mut c = roundtrip(&mut a);
    for _ in 0..20 {
        c.advance_ticks(1);
    }
    let h_c = c.hash_chain_root();

    assert_ne!(h_a, h_b, "save-time hash must differ from 2 ticks past");
    assert_ne!(h_b, h_c, "2 ticks past must differ from 20 ticks past");
    assert_ne!(h_a, h_c, "save-time must differ from 20 ticks past");
}
