//! Architecture experiment for a semantic save-state manifest.
//! Test-only: this does not alter CivSaveBundle or production save format.

use crate::{PolicyInput, ResearchCache, Simulation};
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;

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
            active_mods,
        }
    }

    fn apply_semantic_state(&self, sim: &mut Simulation) {
        sim.economy_policy = PolicyInput {
            base_consumption_joules: self.economy_policy.base_consumption_joules,
            scarcity_multiplier: self.economy_policy.scarcity_multiplier,
        };
        sim.set_policy(crate::policy::policy_from_kind(&self.control_policy_kind));
        *sim.research_cache_mut() = self.research.clone();
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
    source.set_policy(crate::policy::policy_from_kind("capitalist"));
    source.research_cache_mut().researched = vec!["pottery".into(), "masonry".into()];
    source.research_cache_mut().queued.push_back("writing".into());

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
