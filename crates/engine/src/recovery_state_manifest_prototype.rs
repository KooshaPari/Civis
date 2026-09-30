//! Architecture experiment for a semantic save-state manifest.
//! Test-only: this does not alter CivSaveBundle or production save format.

use crate::{policy_from_kind, PolicyInput, Simulation};
use crate::engine::ResearchCache;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
struct RecoveryEconomyPolicy {
    base_consumption_joules: f64,
    scarcity_multiplier: f64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
struct RecoveryModIdentity {
    id: String,
    version: String,
    api_version: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
struct RecoveryStateManifest {
    schema_version: u32,
    economy_policy: RecoveryEconomyPolicy,
    control_policy_kind: String,
    research: ResearchCache,
    market_prices: BTreeMap<String, i64>,
    active_mods: Vec<RecoveryModIdentity>,
}

impl RecoveryStateManifest {
    fn capture(sim: &Simulation) -> Self {
        let mut active_mods: Vec<_> = sim
            .mod_host()
            .mods()
            .iter()
            .map(|loaded| RecoveryModIdentity {
                id: loaded.manifest.meta.id.clone(),
                version: loaded.manifest.meta.version.clone(),
                api_version: loaded.manifest.meta.api_version.clone(),
            })
            .collect();
        active_mods.sort_by(|a, b| a.id.cmp(&b.id));

        Self {
            schema_version: 1,
            economy_policy: RecoveryEconomyPolicy {
                base_consumption_joules: sim.economy_policy.base_consumption_joules,
                scarcity_multiplier: sim.economy_policy.scarcity_multiplier,
            },
            control_policy_kind: sim.policy().name().to_string(),
            research: sim.research_cache().clone(),
            market_prices: sim.market_state.prices.clone(),
            active_mods,
        }
    }

    fn apply_semantic_state(&self, sim: &mut Simulation) {
        sim.economy_policy = PolicyInput {
            base_consumption_joules: self.economy_policy.base_consumption_joules,
            scarcity_multiplier: self.economy_policy.scarcity_multiplier,
        };
        *sim.research_cache_mut() = self.research.clone();
        sim.set_policy(policy_from_kind(&self.control_policy_kind));
        sim.market_state.prices = self.market_prices.clone();
    }

    fn incompatible_guest_memory_ids(&self, sim: &Simulation) -> Vec<String> {
        let declared: BTreeSet<_> = self.active_mods.iter().map(|m| m.id.as_str()).collect();
        let guest = sim.export_mod_guest_state();
        let mut orphan: Vec<_> = guest
            .memories
            .iter()
            .filter(|m| !declared.contains(m.mod_id.as_str()))
            .map(|m| m.mod_id.clone())
            .collect();
        orphan.sort();
        orphan
    }
}

#[test]
fn recovery_prototype_manifest_roundtrip_restores_policy_and_research() {
    let mut source = Simulation::with_seed(7);
    source.economy_policy = PolicyInput {
        base_consumption_joules: 123_456.0,
        scarcity_multiplier: 2.75,
    };
    source.research_cache_mut().researched = vec!["pottery".into(), "masonry".into()];
    source.research_cache_mut().queued.push_back("writing".into());
    source.set_policy(policy_from_kind("capitalist"));
    source.market_state.prices.insert("grain".into(), 777);

    let manifest = RecoveryStateManifest::capture(&source);
    let bytes = serde_json::to_vec(&manifest).expect("serialize prototype manifest");
    let decoded: RecoveryStateManifest =
        serde_json::from_slice(&bytes).expect("deserialize prototype manifest");

    let mut restored = Simulation::with_seed(7);
    decoded.apply_semantic_state(&mut restored);

    assert_eq!(
        restored.economy_policy.base_consumption_joules,
        source.economy_policy.base_consumption_joules
    );
    assert_eq!(
        restored.economy_policy.scarcity_multiplier,
        source.economy_policy.scarcity_multiplier
    );
    assert_eq!(restored.policy().name(), source.policy().name());
    assert_eq!(restored.research_cache(), source.research_cache());
    assert_eq!(restored.policy().name(), "capitalist");
    assert_eq!(restored.market_state.prices, source.market_state.prices);
}

#[test]
fn recovery_prototype_manifest_detects_orphan_guest_memory() {
    let mut sim = Simulation::with_seed(7);
    sim.mod_host_mut()
        .restore_guest_memory("orphan-mod", vec![0xCA, 0xFE]);

    let manifest = RecoveryStateManifest::capture(&sim);
    assert!(manifest.active_mods.is_empty());
    assert_eq!(
        manifest.incompatible_guest_memory_ids(&sim),
        vec!["orphan-mod".to_string()]
    );
}


#[derive(Debug, Clone, PartialEq, Eq)]
enum RecoveryFormatClass {
    Explicit(u32),
    LegacyCandidate,
    SuspiciousMissingMetadata,
}

fn classify_save_dir_for_recovery(dir: &std::path::Path) -> RecoveryFormatClass {
    let metadata = dir.join("metadata.json");
    if metadata.exists() {
        let value: serde_json::Value =
            serde_json::from_slice(&std::fs::read(&metadata).expect("read metadata"))
                .expect("parse metadata");
        let version = value
            .get("format_version")
            .and_then(|v| v.as_u64())
            .expect("metadata format_version") as u32;
        return RecoveryFormatClass::Explicit(version);
    }

    // Presence of components introduced by modern componentized saves means
    // this directory is not safely classifiable as legacy merely because
    // metadata disappeared.
    const MODERN_MARKERS: &[&str] = &[
        "environment.json",
        "cluster_stocks.json",
        "institutions.json",
        "integrity.json",
    ];
    if MODERN_MARKERS.iter().any(|name| dir.join(name).exists()) {
        return RecoveryFormatClass::SuspiciousMissingMetadata;
    }

    RecoveryFormatClass::LegacyCandidate
}

#[test]
fn recovery_prototype_classifier_rejects_metadata_deleted_modern_bundle() {
    let sim = Simulation::with_seed(11);
    let dir = tempfile::tempdir().expect("tempdir");
    let save = dir.path().join("modern.civsave");
    crate::CivSaveBundle::save_dir(&save, &sim).expect("save");
    let before = classify_save_dir_for_recovery(&save);
    assert!(matches!(before, RecoveryFormatClass::Explicit(_)));

    std::fs::remove_file(save.join("metadata.json")).expect("remove metadata");
    assert_eq!(
        classify_save_dir_for_recovery(&save),
        RecoveryFormatClass::SuspiciousMissingMetadata
    );
}

#[test]
fn recovery_prototype_classifier_preserves_explicit_legacy_candidate_state() {
    let dir = tempfile::tempdir().expect("tempdir");
    std::fs::write(dir.path().join("replay.civreplay"), b"legacy-placeholder")
        .expect("legacy marker");
    assert_eq!(
        classify_save_dir_for_recovery(dir.path()),
        RecoveryFormatClass::LegacyCandidate
    );
}
