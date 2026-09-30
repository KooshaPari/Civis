use crate::semantic_save_adapter::{
    SemanticBundleBridge, SemanticGenerationPublisher, SemanticLoadDisposition, SemanticSaveAdapter, SEMANTIC_STATE_FILE,
};
use crate::{PolicyInput, Simulation};
use tempfile::tempdir;

#[test]
fn semantic_save_adapter_requires_component_before_apply() {
    let dir = tempdir().expect("tempdir");
    assert_eq!(
        SemanticSaveAdapter::read_component(dir.path()).unwrap_err(),
        SemanticLoadDisposition::MissingRequiredSemanticState
    );
}

#[test]
fn semantic_save_adapter_roundtrips_component_before_production_wiring() {
    let mut sim = Simulation::with_seed(5);
    sim.economy_policy = PolicyInput {
        base_consumption_joules: 42.0,
        scarcity_multiplier: 3.0,
    };
    sim.research_cache_mut().researched.push("writing".into());

    let dir = tempdir().expect("tempdir");
    let path = SemanticSaveAdapter::write_component(dir.path(), &sim).expect("write");
    assert_eq!(path.file_name().unwrap(), SEMANTIC_STATE_FILE);

    let manifest = SemanticSaveAdapter::read_component(dir.path()).expect("read");
    let mut restored = Simulation::with_seed(5);
    SemanticSaveAdapter::apply_after_compatibility(&manifest, &mut restored).expect("apply");

    assert_eq!(restored.economy_policy.base_consumption_joules, 42.0);
    assert_eq!(restored.research_cache().researched, vec!["writing"]);
}

#[test]
fn semantic_save_adapter_surfaces_orphan_guest_memory_before_apply() {
    let mut source = Simulation::with_seed(5);
    source
        .mod_host_mut()
        .restore_guest_memory("missing-mod", vec![1, 2, 3]);

    let dir = tempdir().expect("tempdir");
    SemanticSaveAdapter::write_component(dir.path(), &source).expect("write");
    let manifest = SemanticSaveAdapter::read_component(dir.path()).expect("read");

    // A freshly constructed simulation represents the resolved active mod set:
    // missing-mod is not loaded.
    let resolved = Simulation::with_seed(5);
    assert_eq!(
        SemanticSaveAdapter::validate_against_resolved_mods(&manifest, &resolved),
        Err(SemanticLoadDisposition::OrphanGuestMemory(vec![
            "missing-mod".to_string()
        ]))
    );
}


#[test]
fn semantic_bundle_bridge_turns_reproduced_policy_research_loss_green() {
    let mut source = Simulation::with_seed(55);
    source.economy_policy = PolicyInput {
        base_consumption_joules: 123_456.0,
        scarcity_multiplier: 2.75,
    };
    source.research_cache_mut().researched = vec!["pottery".into(), "masonry".into()];
    source.research_cache_mut().queued.push_back("writing".into());

    let dir = tempdir().expect("tempdir");
    SemanticBundleBridge::save_opt_in(dir.path(), &source).expect("opt-in save");

    // No mods are required by this fixture, so an empty resolved environment is compatible.
    let resolved = Simulation::with_seed(55);
    let loaded = SemanticBundleBridge::load_opt_in(dir.path(), &resolved).expect("opt-in load");

    assert_eq!(loaded.economy_policy.base_consumption_joules, 123_456.0);
    assert_eq!(loaded.economy_policy.scarcity_multiplier, 2.75);
    assert_eq!(loaded.research_cache().researched, vec!["pottery", "masonry"]);
    assert_eq!(loaded.research_cache().queued.front().map(String::as_str), Some("writing"));
}

#[test]
fn semantic_bundle_bridge_missing_component_fails_without_changing_default_loader() {
    let sim = Simulation::with_seed(55);
    let dir = tempdir().expect("tempdir");
    crate::save_bundle::CivSaveBundle::save_dir(dir.path(), &sim).expect("legacy/current save");

    // Existing loader remains available as the comparison baseline.
    crate::save_bundle::CivSaveBundle::load_dir(dir.path()).expect("default load remains valid");

    let resolved = Simulation::with_seed(55);
    assert!(
        SemanticBundleBridge::load_opt_in(dir.path(), &resolved).is_err(),
        "opt-in vNext bridge must require semantic-state.json rather than silently downgrade"
    );
}


#[test]
fn semantic_bundle_bridge_preserves_default_v5_bytes_when_opt_in_component_is_added() {
    let sim = Simulation::with_seed(77);
    let dir = tempdir().expect("tempdir");

    crate::save_bundle::CivSaveBundle::save_dir(dir.path(), &sim).expect("baseline save");
    let before = std::fs::read(dir.path().join("world_state.json")).expect("baseline world bytes");

    SemanticSaveAdapter::write_component(dir.path(), &sim).expect("add semantic component");
    let after = std::fs::read(dir.path().join("world_state.json")).expect("world bytes after semantic add");

    assert_eq!(before, after, "adding the opt-in semantic component must not rewrite legacy/current world_state bytes");
    assert!(dir.path().join(SEMANTIC_STATE_FILE).is_file());
}

#[test]
fn semantic_bundle_bridge_future_schema_fails_before_default_load() {
    let sim = Simulation::with_seed(77);
    let dir = tempdir().expect("tempdir");
    SemanticBundleBridge::save_opt_in(dir.path(), &sim).expect("opt-in save");

    let path = dir.path().join(SEMANTIC_STATE_FILE);
    let mut json: serde_json::Value =
        serde_json::from_slice(&std::fs::read(&path).expect("read semantic")).expect("json");
    json["schema_version"] = serde_json::json!(9999);
    std::fs::write(&path, serde_json::to_vec_pretty(&json).expect("encode")).expect("write");

    let resolved = Simulation::with_seed(77);
    let err = SemanticBundleBridge::load_opt_in(dir.path(), &resolved).unwrap_err();
    assert!(err.contains("semantic component rejected"));
}


#[test]
fn semantic_generation_failed_stage_does_not_replace_current() {
    let root = tempdir().expect("tempdir");
    let publisher = SemanticGenerationPublisher::new(root.path());

    let g1 = Simulation::with_seed(1);
    publisher.stage("g1", &g1).expect("stage g1");
    publisher.commit("g1").expect("commit g1");
    assert_eq!(publisher.current().unwrap().as_deref(), Some("g1"));

    let g2 = Simulation::with_seed(2);
    let g2_dir = publisher.stage("g2", &g2).expect("stage g2");
    std::fs::remove_file(g2_dir.join(SEMANTIC_STATE_FILE)).expect("fault required semantic component");

    assert!(publisher.commit("g2").is_err());
    assert_eq!(
        publisher.current().unwrap().as_deref(),
        Some("g1"),
        "failed staged generation must not replace accepted CURRENT"
    );
}

#[test]
fn semantic_generation_success_switches_current_and_preserves_prior_generation() {
    let root = tempdir().expect("tempdir");
    let publisher = SemanticGenerationPublisher::new(root.path());

    publisher.stage("g1", &Simulation::with_seed(1)).expect("stage g1");
    publisher.commit("g1").expect("commit g1");
    publisher.stage("g2", &Simulation::with_seed(2)).expect("stage g2");
    publisher.commit("g2").expect("commit g2");

    assert_eq!(publisher.current().unwrap().as_deref(), Some("g2"));
    assert!(publisher.generation_dir("g1").is_dir());
    assert!(publisher.generation_dir("g2").is_dir());
}

#[test]
fn semantic_generation_identity_mismatch_fails_before_pointer_switch() {
    let root = tempdir().expect("tempdir");
    let publisher = SemanticGenerationPublisher::new(root.path());

    publisher.stage("g1", &Simulation::with_seed(1)).expect("stage g1");
    publisher.commit("g1").expect("commit g1");

    let g2_dir = publisher.stage("g2", &Simulation::with_seed(2)).expect("stage g2");
    std::fs::write(g2_dir.join("GENERATION"), b"other").expect("corrupt identity");

    assert!(publisher.commit("g2").is_err());
    assert_eq!(publisher.current().unwrap().as_deref(), Some("g1"));
}


#[test]
fn semantic_generation_reconcile_discards_orphan_next_without_promoting() {
    let root = tempdir().expect("tempdir");
    let publisher = SemanticGenerationPublisher::new(root.path());

    publisher.stage("g1", &Simulation::with_seed(1)).expect("stage g1");
    publisher.commit("g1").expect("commit g1");
    publisher.stage("g2", &Simulation::with_seed(2)).expect("stage g2");

    // Simulate interruption after writing publication intent but before the
    // rename that would make g2 accepted.
    std::fs::write(root.path().join("CURRENT.next"), b"g2").expect("write orphan intent");

    assert_eq!(publisher.reconcile().unwrap().as_deref(), Some("g1"));
    assert!(!root.path().join("CURRENT.next").exists());
    assert_eq!(publisher.current().unwrap().as_deref(), Some("g1"));
    assert!(publisher.generation_dir("g2").is_dir(), "staged candidate remains inspectable");
}

#[test]
fn semantic_generation_reconcile_rejects_current_pointing_to_invalid_generation() {
    let root = tempdir().expect("tempdir");
    let publisher = SemanticGenerationPublisher::new(root.path());

    let g1 = publisher.stage("g1", &Simulation::with_seed(1)).expect("stage g1");
    publisher.commit("g1").expect("commit g1");
    std::fs::remove_file(g1.join(SEMANTIC_STATE_FILE)).expect("damage accepted generation");

    assert!(publisher.reconcile().is_err(),
        "CURRENT pointing at an invalid generation must not be silently treated as healthy");
}


#[test]
fn semantic_bundle_bridge_rebinds_current_tick_to_restored_world_tick() {
    let root = tempdir().expect("tempdir");
    let mut source = Simulation::with_seed(41);
    source.advance_ticks(7);
    SemanticBundleBridge::save_opt_in(root.path(), &source).expect("save opt-in");

    let resolved = Simulation::with_seed(41);
    let loaded = SemanticBundleBridge::load_opt_in(root.path(), &resolved).expect("load opt-in");

    assert_eq!(loaded.state.tick, source.state.tick);
    assert_eq!(
        loaded.current_tick, loaded.state.tick,
        "live current_tick mirror must be rebound to authoritative restored WorldState before any post-load phase"
    );
}


#[test]
fn semantic_bundle_bridge_resynchronizes_live_tick_mirror() {
    let mut source = Simulation::with_seed(91);
    source.advance_ticks(7);
    assert_eq!(source.state.tick, 7);
    assert_eq!(source.current_tick, 7);

    let dir = tempdir().expect("tempdir");
    SemanticBundleBridge::save_opt_in(dir.path(), &source).expect("opt-in save");

    // Current production loader restores WorldState.tick but does not explicitly
    // resynchronize the live Simulation.current_tick mirror.
    let production = crate::save_bundle::CivSaveBundle::load_dir(dir.path()).expect("production load");
    assert_eq!(production.state.tick, 7);
    assert_ne!(
        production.current_tick, production.state.tick,
        "control must stay red until the default loader owns mirror restoration"
    );

    let resolved = Simulation::with_seed(91);
    let vnext = SemanticBundleBridge::load_opt_in(dir.path(), &resolved).expect("vNext load");
    assert_eq!(vnext.state.tick, 7);
    assert_eq!(vnext.current_tick, vnext.state.tick);
}

#[test]
fn semantic_bundle_bridge_rejects_cross_tick_semantic_component() {
    let mut source = Simulation::with_seed(92);
    source.advance_ticks(3);
    let dir = tempdir().expect("tempdir");
    SemanticBundleBridge::save_opt_in(dir.path(), &source).expect("opt-in save");

    let path = dir.path().join(SEMANTIC_STATE_FILE);
    let mut json: serde_json::Value =
        serde_json::from_slice(&std::fs::read(&path).expect("read semantic")).expect("json");
    json["tick"] = serde_json::json!(99);
    std::fs::write(&path, serde_json::to_vec_pretty(&json).expect("encode")).expect("write");

    let resolved = Simulation::with_seed(92);
    let err = SemanticBundleBridge::load_opt_in(dir.path(), &resolved).unwrap_err();
    assert!(err.contains("semantic tick 99 does not match loaded world tick 3"));
}
