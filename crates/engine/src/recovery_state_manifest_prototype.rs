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
    tick: u64,
    rng_seed: u64,
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
            tick: sim.state.tick,
            rng_seed: sim.state.rng_seed,
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

    fn validate_supported(&self) -> Result<(), String> {
        if self.schema_version != 1 {
            return Err(format!("unsupported recovery semantic schema {}", self.schema_version));
        }
        if self.control_policy_kind.is_empty() {
            return Err("missing control policy kind".to_string());
        }
        Ok(())
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

    decoded.validate_supported().expect("supported prototype schema");
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


#[test]
fn recovery_prototype_manifest_rejects_missing_required_semantic_component() {
    let sim = Simulation::with_seed(13);
    let manifest = RecoveryStateManifest::capture(&sim);
    let mut value = serde_json::to_value(&manifest).expect("manifest value");
    value
        .as_object_mut()
        .expect("object")
        .remove("research")
        .expect("research field present");

    let result = serde_json::from_value::<RecoveryStateManifest>(value);
    assert!(
        result.is_err(),
        "required semantic components must not silently default when absent"
    );
}

#[test]
fn recovery_prototype_manifest_rejects_unknown_future_schema() {
    let sim = Simulation::with_seed(13);
    let mut manifest = RecoveryStateManifest::capture(&sim);
    manifest.schema_version = 999;
    assert!(manifest.validate_supported().is_err());
}


struct RecoverySaveGenerationPublisher {
    root: std::path::PathBuf,
}

impl RecoverySaveGenerationPublisher {
    fn new(root: impl Into<std::path::PathBuf>) -> Self {
        Self { root: root.into() }
    }

    fn generation_dir(&self, id: &str) -> std::path::PathBuf {
        self.root.join("generations").join(id)
    }

    fn bundle_dir(&self, id: &str) -> std::path::PathBuf {
        self.generation_dir(id).join("bundle")
    }

    fn stage(&self, id: &str, sim: &Simulation) -> Result<(), String> {
        let generation = self.generation_dir(id);
        let bundle = self.bundle_dir(id);
        std::fs::create_dir_all(&generation).map_err(|e| e.to_string())?;
        crate::CivSaveBundle::save_dir(&bundle, sim).map_err(|e| e.to_string())?;
        let semantic = RecoveryStateManifest::capture(sim);
        semantic.validate_supported()?;
        std::fs::write(
            generation.join("semantic-state.json"),
            serde_json::to_vec_pretty(&semantic).map_err(|e| e.to_string())?,
        )
        .map_err(|e| e.to_string())?;
        self.validate(id)
    }

    fn validate(&self, id: &str) -> Result<(), String> {
        let generation = self.generation_dir(id);
        let bundle = self.bundle_dir(id);
        crate::CivSaveBundle::load_dir(&bundle).map_err(|e| e.to_string())?;
        let semantic_bytes =
            std::fs::read(generation.join("semantic-state.json")).map_err(|e| e.to_string())?;
        let semantic: RecoveryStateManifest =
            serde_json::from_slice(&semantic_bytes).map_err(|e| e.to_string())?;
        semantic.validate_supported()?;
        let loaded = crate::CivSaveBundle::load_dir(&bundle).map_err(|e| e.to_string())?;
        if loaded.state.tick != semantic.tick || loaded.state.rng_seed != semantic.rng_seed {
            return Err(format!(
                "semantic/bundle identity mismatch: semantic tick/seed={}/{}, bundle={}/{}",
                semantic.tick,
                semantic.rng_seed,
                loaded.state.tick,
                loaded.state.rng_seed
            ));
        }
        match classify_save_dir_for_recovery(&bundle) {
            RecoveryFormatClass::Explicit(_) => Ok(()),
            other => Err(format!("generation is not an explicit supported save: {other:?}")),
        }
    }

    fn commit(&self, id: &str) -> Result<(), String> {
        self.validate(id)?;
        std::fs::create_dir_all(&self.root).map_err(|e| e.to_string())?;
        let pending = self.root.join("CURRENT.pending");
        let current = self.root.join("CURRENT");
        std::fs::write(&pending, id.as_bytes()).map_err(|e| e.to_string())?;
        if current.exists() {
            std::fs::remove_file(&current).map_err(|e| e.to_string())?;
        }
        std::fs::rename(&pending, &current).map_err(|e| e.to_string())?;
        Ok(())
    }

    fn current(&self) -> Option<String> {
        std::fs::read_to_string(self.root.join("CURRENT")).ok()
    }
}

#[test]
fn recovery_prototype_failed_staged_save_does_not_replace_current_generation() {
    let root = tempfile::tempdir().expect("root");
    let publisher = RecoverySaveGenerationPublisher::new(root.path());

    let mut g1 = Simulation::with_seed(21);
    g1.economy_policy.base_consumption_joules = 111.0;
    publisher.stage("g1", &g1).expect("stage g1");
    publisher.commit("g1").expect("commit g1");
    assert_eq!(publisher.current().as_deref(), Some("g1"));

    let mut g2 = Simulation::with_seed(22);
    g2.economy_policy.base_consumption_joules = 222.0;
    publisher.stage("g2", &g2).expect("stage g2");
    std::fs::remove_file(publisher.generation_dir("g2").join("semantic-state.json"))
        .expect("damage required staged semantic manifest");

    assert!(publisher.commit("g2").is_err());
    assert_eq!(
        publisher.current().as_deref(),
        Some("g1"),
        "failed staged candidate must not replace the last accepted generation"
    );
    assert!(
        crate::CivSaveBundle::load_dir(&publisher.bundle_dir("g1")).is_ok(),
        "previous accepted save generation remains independently loadable"
    );
}

#[test]
fn recovery_prototype_successful_commit_switches_pointer_without_destroying_prior_generation() {
    let root = tempfile::tempdir().expect("root");
    let publisher = RecoverySaveGenerationPublisher::new(root.path());

    let g1 = Simulation::with_seed(31);
    publisher.stage("g1", &g1).expect("stage g1");
    publisher.commit("g1").expect("commit g1");

    let g2 = Simulation::with_seed(32);
    publisher.stage("g2", &g2).expect("stage g2");
    publisher.commit("g2").expect("commit g2");

    assert_eq!(publisher.current().as_deref(), Some("g2"));
    assert!(publisher.generation_dir("g1").exists());
    assert!(publisher.generation_dir("g2").exists());
}


#[test]
fn recovery_prototype_rejects_semantic_manifest_from_different_generation() {
    let root = tempfile::tempdir().expect("root");
    let publisher = RecoverySaveGenerationPublisher::new(root.path());

    let mut g1 = Simulation::with_seed(41);
    g1.state.tick = 10;
    publisher.stage("g1", &g1).expect("stage g1");

    let mut g2 = Simulation::with_seed(42);
    g2.state.tick = 20;
    publisher.stage("g2", &g2).expect("stage g2");

    std::fs::copy(
        publisher.generation_dir("g1").join("semantic-state.json"),
        publisher.generation_dir("g2").join("semantic-state.json"),
    )
    .expect("cross-wire semantic manifest");

    assert!(
        publisher.validate("g2").is_err(),
        "a valid semantic component from another save generation must not qualify this bundle"
    );
}
