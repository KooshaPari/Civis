//! Experimental semantic save component.
//!
//! This is an implementation candidate derived from recovery evidence. It is
//! not wired into CivSaveBundle on this branch yet.

use crate::engine::ResearchCache;
use crate::{policy_from_kind, PolicyInput, Simulation};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

pub const SEMANTIC_STATE_SCHEMA_VERSION: u32 = 1;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SemanticEconomyPolicy {
    pub base_consumption_joules: f64,
    pub scarcity_multiplier: f64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SemanticModIdentity {
    pub id: String,
    pub version: String,
    pub api_version: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SemanticStateManifest {
    pub schema_version: u32,
    pub tick: u64,
    pub rng_seed: u64,
    pub economy_policy: SemanticEconomyPolicy,
    pub control_policy_kind: String,
    pub research: ResearchCache,
    pub market_prices: BTreeMap<String, i64>,
    pub tutorial_progress: crate::tutorial::TutorialProgress,
    pub religious_profiles: serde_json::Value,
    pub active_caravans: serde_json::Value,
    pub active_mods: Vec<SemanticModIdentity>,
    /// Mod namespaces that own persisted guest-memory blobs in this save.
    pub guest_memory_mod_ids: Vec<String>,
}

impl SemanticStateManifest {
    pub fn capture(sim: &Simulation) -> Result<Self, String> {
        let mut active_mods: Vec<_> = sim
            .mod_host()
            .mods()
            .iter()
            .map(|loaded| SemanticModIdentity {
                id: loaded.manifest.meta.id.clone(),
                version: loaded.manifest.meta.version.clone(),
                api_version: loaded.manifest.meta.api_version.clone(),
            })
            .collect();
        active_mods.sort_by(|a, b| a.id.cmp(&b.id));

        let guest = sim.export_mod_guest_state();
        let mut guest_memory_mod_ids: Vec<_> = guest.memories.iter().map(|m| m.mod_id.clone()).collect();
        guest_memory_mod_ids.sort();
        guest_memory_mod_ids.dedup();

        Ok(Self {
            schema_version: SEMANTIC_STATE_SCHEMA_VERSION,
            tick: sim.state.tick,
            rng_seed: sim.state.rng_seed,
            economy_policy: SemanticEconomyPolicy {
                base_consumption_joules: sim.economy_policy.base_consumption_joules,
                scarcity_multiplier: sim.economy_policy.scarcity_multiplier,
            },
            control_policy_kind: sim.policy().name().to_string(),
            research: sim.research_cache().clone(),
            market_prices: sim.market_state.prices.clone(),
            tutorial_progress: sim.tutorial_progress.clone(),
            religious_profiles: serde_json::to_value(&sim.religious_profiles)
                .map_err(|e| format!("serialize religious profiles: {e}"))?,
            active_caravans: serde_json::to_value(&sim.active_caravans)
                .map_err(|e| format!("serialize active caravans: {e}"))?,
            active_mods,
            guest_memory_mod_ids,
        })
    }

    pub fn validate_supported(&self) -> Result<(), String> {
        if self.schema_version != SEMANTIC_STATE_SCHEMA_VERSION {
            return Err(format!("unsupported semantic schema {}", self.schema_version));
        }
        if self.control_policy_kind.is_empty() {
            return Err("missing control policy kind".to_string());
        }
        Ok(())
    }

    pub fn apply_to(&self, sim: &mut Simulation) -> Result<(), String> {
        self.validate_supported()?;
        sim.economy_policy = PolicyInput {
            base_consumption_joules: self.economy_policy.base_consumption_joules,
            scarcity_multiplier: self.economy_policy.scarcity_multiplier,
        };
        *sim.research_cache_mut() = self.research.clone();
        sim.set_policy(policy_from_kind(&self.control_policy_kind));
        sim.market_state.prices = self.market_prices.clone();
        sim.tutorial_progress = self.tutorial_progress.clone();
        sim.religious_profiles = serde_json::from_value(self.religious_profiles.clone())
            .map_err(|e| format!("restore religious profiles: {e}"))?;
        sim.active_caravans = serde_json::from_value(self.active_caravans.clone())
            .map_err(|e| format!("restore active caravans: {e}"))?;
        Ok(())
    }

    pub fn orphan_guest_memory_ids_against_resolved_mods(&self, resolved: &Simulation) -> Vec<String> {
        let resolved_ids: BTreeSet<_> = resolved
            .mod_host()
            .mods()
            .iter()
            .map(|m| m.manifest.meta.id.as_str())
            .collect();
        let mut orphan: Vec<_> = self
            .guest_memory_mod_ids
            .iter()
            .filter(|id| !resolved_ids.contains(id.as_str()))
            .cloned()
            .collect();
        orphan.sort();
        orphan
    }

    pub fn orphan_guest_memory_ids(&self, _sim: &Simulation) -> Vec<String> {
        let declared: BTreeSet<_> = self.active_mods.iter().map(|m| m.id.as_str()).collect();
        let mut orphan: Vec<_> = self
            .guest_memory_mod_ids
            .iter()
            .filter(|id| !declared.contains(id.as_str()))
            .cloned()
            .collect();
        orphan.sort();
        orphan
    }
}
