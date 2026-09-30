//! Recovery-oracle tests for the mature-first specification program.
//!
//! These tests are intentionally expected to expose current persistence gaps.
//! They MUST NOT be counted as product greens until executed against the bound
//! candidate and reviewed. Do not weaken assertions to make current code pass.

use crate::{CivSaveBundle, PolicyInput, Simulation};
use tempfile::tempdir;

#[test]
fn recovery_oracle_save_roundtrip_preserves_runtime_economy_policy() {
    let mut sim = Simulation::with_seed(0xC1A15);
    sim.economy_policy = PolicyInput {
        base_consumption_joules: 123_456.0,
        scarcity_multiplier: 2.75,
    };

    let dir = tempdir().expect("tempdir");
    let save = dir.path().join("policy.civsave");
    CivSaveBundle::save_dir(&save, &sim).expect("save");

    let loaded = CivSaveBundle::load_dir(&save).expect("load");
    assert_eq!(
        loaded.economy_policy.base_consumption_joules,
        sim.economy_policy.base_consumption_joules,
        "a world save must preserve its runtime economy policy or the product contract must explicitly replace this oracle with scenario/profile rebinding"
    );
    assert_eq!(
        loaded.economy_policy.scarcity_multiplier,
        sim.economy_policy.scarcity_multiplier,
        "scarcity policy changed across save/load"
    );
}

#[test]
fn recovery_oracle_save_roundtrip_preserves_research_cache_state() {
    let mut sim = Simulation::with_seed(0xC1A15);
    sim.research_cache_mut().researched = vec!["pottery".into(), "masonry".into()];
    sim.research_cache_mut().queued.push_back("writing".into());

    let dir = tempdir().expect("tempdir");
    let save = dir.path().join("research.civsave");
    CivSaveBundle::save_dir(&save, &sim).expect("save");

    let loaded = CivSaveBundle::load_dir(&save).expect("load");
    assert_eq!(
        loaded.research_cache().researched,
        sim.research_cache().researched,
        "researched technologies are user-visible/outcome-affecting state"
    );
    assert_eq!(
        loaded.research_cache().queued,
        sim.research_cache().queued,
        "queued/in-progress research must have an explicit persistence disposition"
    );
}

#[test]
fn recovery_oracle_guest_memory_does_not_prove_loaded_mod_restoration() {
    let mut sim = Simulation::with_seed(0xC1A15);
    let orphan_id = "recovery-orphan-mod";
    assert!(
        !sim.mod_host().mods().iter().any(|m| m.manifest.meta.id == orphan_id),
        "fixture requires the mod to be absent"
    );
    sim.mod_host_mut()
        .restore_guest_memory(orphan_id, vec![0xCA, 0xFE]);

    let dir = tempdir().expect("tempdir");
    let save = dir.path().join("orphan-mod.civsave");
    CivSaveBundle::save_dir(&save, &sim).expect("save");

    let loaded = CivSaveBundle::load_dir(&save).expect("load");
    assert_eq!(
        loaded.mod_host().guest_memory_snapshot(orphan_id),
        vec![0xCA, 0xFE],
        "fixture first proves orphan guest bytes survive"
    );
    assert!(
        loaded.mod_host().mods().iter().any(|m| m.manifest.meta.id == orphan_id),
        "EXPECTED CURRENT FAILURE / CONTRACT QUESTION: guest memory exists without a corresponding loaded mod; a mature load must resolve, migrate, reject, or explicitly degrade this state rather than call bytes alone a restored active mod"
    );
}


#[test]
fn recovery_oracle_save_roundtrip_preserves_control_policy_kind() {
    let mut sim = Simulation::with_seed(0xC1A15);
    sim.set_policy(crate::policy::policy_from_kind("capitalist"));
    assert_eq!(sim.policy().name(), "capitalist");

    let dir = tempdir().expect("tempdir");
    let save = dir.path().join("control-policy.civsave");
    CivSaveBundle::save_dir(&save, &sim).expect("save");

    let loaded = CivSaveBundle::load_dir(&save).expect("load");
    assert_eq!(
        loaded.policy().name(),
        "capitalist",
        "the high-level control policy is distinct from PolicyInput and needs an explicit persistence/rebinding contract"
    );
}


#[test]
fn recovery_oracle_save_roundtrip_preserves_market_state() {
    let mut sim = Simulation::with_seed(0xC1A15);
    sim.market_state.prices.insert("grain".into(), 777);
    assert_eq!(sim.market_state.prices.get("grain"), Some(&777));

    let dir = tempdir().expect("tempdir");
    let save = dir.path().join("market-state.civsave");
    CivSaveBundle::save_dir(&save, &sim).expect("save");

    let loaded = CivSaveBundle::load_dir(&save).expect("load");
    assert_eq!(
        loaded.market_state.prices.get("grain"),
        Some(&777),
        "market prices are exposed to clients and affect future economic behavior; reset/reconstruction requires an explicit contract"
    );
}


#[test]
fn recovery_oracle_v5_metadata_removal_cannot_silently_downgrade() {
    let sim = Simulation::with_seed(0xC1A15);
    let dir = tempdir().expect("tempdir");
    let save = dir.path().join("metadata-downgrade.civsave");
    CivSaveBundle::save_dir(&save, &sim).expect("save v5 fixture");

    std::fs::remove_file(save.join("metadata.json")).expect("remove metadata");
    let result = CivSaveBundle::load_dir(&save);
    assert!(
        result.is_err(),
        "a freshly-written current-format bundle with metadata removed must not be silently reclassified as legacy and bypass current-format integrity policy"
    );
}


#[test]
fn recovery_oracle_save_roundtrip_preserves_tutorial_progress() {
    let mut sim = Simulation::with_seed(0xC1A15);
    sim.tutorial_progress.current = crate::tutorial::TutorialMilestone::FirstReligion;
    sim.tutorial_progress.faction_exists = true;
    sim.tutorial_progress.tech_unlocked = true;
    sim.tutorial_progress.war_declared = true;
    sim.tutorial_progress.religion_emerged = true;

    let dir = tempdir().expect("tempdir");
    let save = dir.path().join("tutorial-progress.civsave");
    CivSaveBundle::save_dir(&save, &sim).expect("save");

    let loaded = CivSaveBundle::load_dir(&save).expect("load");
    assert_eq!(
        loaded.tutorial_progress,
        sim.tutorial_progress,
        "TutorialProgress is explicitly a persisted onboarding state surface; load must not silently restart onboarding"
    );
}

#[test]
fn recovery_oracle_save_roundtrip_preserves_religious_profiles() {
    let mut sim = Simulation::with_seed(0xC1A15);
    let mut profile = crate::religion::ReligiousProfile::new(123, 42);
    profile.settlement_id = 7;
    profile.monitoring = 0.25;
    profile.mythic_coherence = 0.75;
    profile.uncertainty_reduction = 0.5;
    sim.religious_profiles.insert(7, profile);

    let dir = tempdir().expect("tempdir");
    let save = dir.path().join("religious-profiles.civsave");
    CivSaveBundle::save_dir(&save, &sim).expect("save");

    let loaded = CivSaveBundle::load_dir(&save).expect("load");
    assert_eq!(
        loaded.religious_profiles,
        sim.religious_profiles,
        "religious profiles are dashboard-visible and feed later emergence/tutorial behavior"
    );
}

#[test]
fn recovery_oracle_save_roundtrip_preserves_active_caravans_in_transit() {
    let mut sim = Simulation::with_seed(0xC1A15);
    let mut cargo = std::collections::BTreeMap::new();
    cargo.insert(1, 500);
    sim.active_caravans.push(crate::caravan::Caravan {
        id: 77,
        source: 1,
        target: 2,
        cargo,
        ticks_remaining: 9,
        travel_time: 12,
        raided: false,
    });

    let dir = tempdir().expect("tempdir");
    let save = dir.path().join("active-caravans.civsave");
    CivSaveBundle::save_dir(&save, &sim).expect("save");

    let loaded = CivSaveBundle::load_dir(&save).expect("load");
    assert_eq!(
        loaded.active_caravans.len(),
        1,
        "an in-flight caravan changes future stocks/trust and needs an explicit durable or reconstruction contract"
    );
    let caravan = &loaded.active_caravans[0];
    assert_eq!(caravan.id, 77);
    assert_eq!(caravan.ticks_remaining, 9);
    assert_eq!(caravan.cargo.get(&1), Some(&500));
}
