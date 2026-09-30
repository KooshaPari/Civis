use crate::semantic_state_manifest::{SemanticStateManifest, SEMANTIC_STATE_SCHEMA_VERSION};
use crate::{policy_from_kind, PolicyInput, Simulation};

#[test]
fn semantic_manifest_candidate_restores_qualified_domains() {
    let mut source = Simulation::with_seed(99);
    source.economy_policy = PolicyInput {
        base_consumption_joules: 123_456.0,
        scarcity_multiplier: 2.75,
    };
    source.set_policy(policy_from_kind("capitalist"));
    source.research_cache_mut().researched = vec!["pottery".into(), "masonry".into()];
    source.research_cache_mut().queued.push_back("writing".into());
    source.market_state.prices.insert("grain".into(), 777);
    source.tutorial_progress.current = crate::tutorial::TutorialMilestone::FirstReligion;
    source.tutorial_progress.faction_exists = true;

    let mut religion = crate::religion::ReligiousProfile::new(123, 42);
    religion.settlement_id = 7;
    source.religious_profiles.insert(7, religion);

    let mut cargo = std::collections::BTreeMap::new();
    cargo.insert(1, 500);
    source.active_caravans.push(crate::caravan::Caravan {
        id: 77,
        source: 1,
        target: 2,
        cargo,
        ticks_remaining: 9,
        travel_time: 12,
        raided: false,
    });

    let manifest = SemanticStateManifest::capture(&source).expect("capture");
    let bytes = serde_json::to_vec(&manifest).expect("encode");
    let decoded: SemanticStateManifest = serde_json::from_slice(&bytes).expect("decode");
    decoded.validate_supported().expect("supported");

    let mut restored = Simulation::with_seed(99);
    decoded.apply_to(&mut restored).expect("apply");

    assert_eq!(restored.economy_policy.base_consumption_joules, 123_456.0);
    assert_eq!(restored.policy().name(), "capitalist");
    assert_eq!(restored.research_cache().researched, vec!["pottery", "masonry"]);
    assert_eq!(restored.market_state.prices.get("grain"), Some(&777));
    assert_eq!(restored.tutorial_progress.current, crate::tutorial::TutorialMilestone::FirstReligion);
    assert!(restored.religious_profiles.contains_key(&7));
    assert_eq!(restored.active_caravans.len(), 1);
    assert_eq!(restored.active_caravans[0].id, 77);
}

#[test]
fn semantic_manifest_candidate_rejects_future_schema() {
    let sim = Simulation::with_seed(99);
    let mut manifest = SemanticStateManifest::capture(&sim).expect("capture");
    manifest.schema_version = SEMANTIC_STATE_SCHEMA_VERSION + 1;
    assert!(manifest.validate_supported().is_err());
}

#[test]
fn semantic_manifest_candidate_detects_orphan_guest_memory() {
    let mut sim = Simulation::with_seed(99);
    sim.mod_host_mut().restore_guest_memory("orphan-mod", vec![1, 2, 3]);
    let manifest = SemanticStateManifest::capture(&sim).expect("capture");
    assert_eq!(manifest.orphan_guest_memory_ids(&sim), vec!["orphan-mod"]);
}
