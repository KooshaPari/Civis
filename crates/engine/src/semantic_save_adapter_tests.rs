use crate::semantic_save_adapter::{
    SemanticLoadDisposition, SemanticSaveAdapter, SEMANTIC_STATE_FILE,
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
