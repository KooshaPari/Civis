//! Emergent technology and research (FR-TECH).
//!
//! Knowledge accumulates from population scale, resource surplus, and neighbor
//! pressure — not from a fixed prerequisite tree. Factions cross emergent
//! thresholds to unlock capability tiers surfaced on [`SimulationSnapshot`].

use std::collections::{BTreeMap, BTreeSet};

use civ_agents::{Alignment, Civilian as AgentCivilian};
use hecs::World;
use serde::{Deserialize, Serialize};

use crate::engine::{
    ResearchCache, Resources, TradeRoute, WorldState, TECH_GUNPOWDER, TECH_IRRIGATION,
    TECH_METALLURGY, TECH_SANITATION, TECH_STORAGE, TECH_WRITING,
};

/// Per-tick pressure signals that drive emergent research rates and thresholds.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct EmergentSignals {
    /// Faction-aligned civilian headcount (population pressure).
    pub population: u32,
    /// Resource headroom per capita; >1.0 means surplus, <1.0 scarcity.
    pub surplus_index: f32,
    /// Cross-border contact / rivalry urgency in `[0, 1]`.
    pub neighbor_pressure: f32,
}

/// Per-faction accumulated knowledge and unlocked emergent capabilities.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct FactionResearchProfile {
    pub knowledge: u64,
    pub tech_level: u32,
    pub tech_unlocks: u64,
    pub capabilities: Vec<String>,
}

/// All faction research profiles keyed by faction id.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct EmergentResearchState {
    profiles: BTreeMap<u32, FactionResearchProfile>,
    /// Cached signals from the most recent research phase (unlock threshold tuning).
    #[serde(skip)]
    last_signals: BTreeMap<u32, EmergentSignals>,
}

/// Emergent capability unlocked at each tier (no DAG — thresholds only).
const TIER_CAPABILITIES: &[(&str, u64)] = &[
    ("emergent-irrigation", TECH_IRRIGATION),
    ("emergent-storage", TECH_STORAGE),
    ("emergent-metallurgy", TECH_METALLURGY),
    ("emergent-writing", TECH_WRITING),
    ("emergent-sanitation", TECH_SANITATION),
    ("emergent-gunpowder", TECH_GUNPOWDER),
];

const BASE_KNOWLEDGE_THRESHOLD: u64 = 800;
const TICKS_PROSPERITY_TEST: u32 = 480;

impl EmergentResearchState {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    #[must_use]
    pub fn tech_level(&self, faction_id: u32) -> u32 {
        self.profiles
            .get(&faction_id)
            .map_or(0, |profile| profile.tech_level)
    }

    #[must_use]
    pub fn knowledge(&self, faction_id: u32) -> u64 {
        self.profiles
            .get(&faction_id)
            .map_or(0, |profile| profile.knowledge)
    }

    #[must_use]
    pub fn profiles(&self) -> &BTreeMap<u32, FactionResearchProfile> {
        &self.profiles
    }

    #[must_use]
    pub fn faction_tech_levels(&self) -> BTreeMap<u32, u32> {
        self.profiles
            .iter()
            .map(|(&id, profile)| (id, profile.tech_level))
            .collect()
    }

    #[must_use]
    pub fn max_tech_level(&self) -> u32 {
        self.profiles
            .values()
            .map(|profile| profile.tech_level)
            .max()
            .unwrap_or(0)
    }

    /// Accumulate knowledge for one faction from emergent pressure signals.
    pub fn accumulate_faction(&mut self, faction_id: u32, signals: &EmergentSignals) {
        let points = research_points_per_tick(signals);
        let profile = self.profiles.entry(faction_id).or_default();
        profile.knowledge = profile.knowledge.saturating_add(points);
    }

    /// Attempt tier unlocks for every faction whose knowledge crossed a threshold.
    pub fn try_unlock_all(&mut self) {
        let faction_ids: Vec<u32> = self.profiles.keys().copied().collect();
        for faction_id in faction_ids {
            self.try_unlock_faction(faction_id);
        }
    }

    fn try_unlock_faction(&mut self, faction_id: u32) {
        let Some(signals) = self.last_signals.get(&faction_id).copied() else {
            return;
        };
        loop {
            let profile = match self.profiles.get(&faction_id) {
                Some(profile) => profile,
                None => break,
            };
            let next_tier = profile.tech_level.saturating_add(1);
            if next_tier as usize > TIER_CAPABILITIES.len() {
                break;
            }
            let threshold = knowledge_threshold_for_tier(next_tier, &signals);
            if profile.knowledge < threshold {
                break;
            }
            let (name, bit) = TIER_CAPABILITIES[next_tier as usize - 1];
            let profile = self.profiles.get_mut(&faction_id).expect("profile");
            profile.tech_level = next_tier;
            profile.tech_unlocks |= bit;
            if !profile.capabilities.iter().any(|cap| cap == name) {
                profile.capabilities.push(name.to_string());
            }
        }
    }

    /// Run the research accumulation pass for every known faction.
    pub fn tick_research(
        &mut self,
        world_state: &WorldState,
        world: &World,
        trade_routes: &[TradeRoute],
        faction_aggression: &BTreeMap<u32, f32>,
    ) {
        self.last_signals.clear();
        for (&faction_id, _) in &world_state.factions {
            let population = faction_population(world, faction_id);
            let resources = world_state
                .faction_resources
                .get(&faction_id)
                .cloned()
                .unwrap_or_default();
            let treasury = world_state
                .faction_treasury
                .get(&faction_id)
                .copied()
                .unwrap_or_default();
            let signals = emergent_signals_for_faction(
                faction_id,
                population,
                &resources,
                treasury,
                trade_routes,
                faction_aggression,
            );
            self.last_signals.insert(faction_id, signals);
            self.accumulate_faction(faction_id, &signals);
        }
    }

    /// Apply emergent unlocks and mirror capability names into the global cache.
    pub fn tick_tech(&mut self, research_cache: &mut ResearchCache) {
        self.try_unlock_all();
        sync_global_research_cache(self, research_cache);
    }
}

/// Knowledge gained per tick from population, surplus, and neighbor-pressure signals.
#[must_use]
pub fn research_points_per_tick(signals: &EmergentSignals) -> u64 {
    let pop_term = u64::from(signals.population.max(1)) / 24;
    let surplus_term = (signals.surplus_index.max(0.0) * 12.0) as u64;
    let pressure_term = (signals.neighbor_pressure.clamp(0.0, 1.0) * 18.0) as u64;
    1 + pop_term + surplus_term + pressure_term
}

/// Emergent threshold for the next tier — surplus and population ease discovery;
/// neighbor pressure accelerates unlock urgency (need-driven innovation).
#[must_use]
pub fn knowledge_threshold_for_tier(tier: u32, signals: &EmergentSignals) -> u64 {
    let tier_scale = u64::from(tier.max(1));
    let base = BASE_KNOWLEDGE_THRESHOLD.saturating_mul(tier_scale);
    let pop_ease = 1.0 + (f32::from(signals.population.min(256)) / 128.0);
    let surplus_ease = 1.0 / signals.surplus_index.clamp(0.15, 8.0);
    let need_accel = 1.0 / (1.0 + signals.neighbor_pressure.clamp(0.0, 1.0) * 0.65);
    let scaled = (f64::from(base as u32) * f64::from(surplus_ease) * f64::from(need_accel)
        / f64::from(pop_ease)) as u64;
    scaled.max(120)
}

#[must_use]
pub fn emergent_signals_for_faction(
    faction_id: u32,
    population: u32,
    resources: &Resources,
    treasury: fixed::types::I16F16,
    trade_routes: &[TradeRoute],
    faction_aggression: &BTreeMap<u32, f32>,
) -> EmergentSignals {
    let pop = population.max(1);
    let stock = resources.food.to_num::<f64>()
        + resources.wood.to_num::<f64>()
        + resources.metal.to_num::<f64>()
        + resources.energy.to_num::<f64>()
        + treasury.to_num::<f64>();
    let per_capita = stock / f64::from(pop);
    let surplus_index = (per_capita / 2.5).clamp(0.05, 12.0) as f32;

    let route_count = trade_routes
        .iter()
        .filter(|route| route.from_faction == faction_id || route.to_faction == faction_id)
        .count() as f32;
    let aggression = faction_aggression
        .get(&faction_id)
        .copied()
        .unwrap_or(0.0)
        .clamp(0.0, 1.0);
    let neighbor_pressure = ((route_count / 4.0) + aggression * 0.5).clamp(0.0, 1.0);

    EmergentSignals {
        population: pop,
        surplus_index,
        neighbor_pressure,
    }
}

#[must_use]
pub fn faction_population(world: &World, faction_id: u32) -> u32 {
    world
        .query::<&AgentCivilian>()
        .iter()
        .filter(|(_, civ)| matches!(civ.alignment, Alignment::Faction(id) if id == faction_id))
        .count() as u32
}

fn sync_global_research_cache(state: &EmergentResearchState, cache: &mut ResearchCache) {
    let mut names = BTreeSet::new();
    for profile in state.profiles.values() {
        for cap in &profile.capabilities {
            names.insert(cap.clone());
        }
    }
    cache.researched = names.into_iter().collect();
}

#[cfg(test)]
mod tests {
    use super::*;
    use fixed::types::I16F16 as Fixed;

    fn prosperous_signals() -> EmergentSignals {
        EmergentSignals {
            population: 128,
            surplus_index: 4.5,
            neighbor_pressure: 0.2,
        }
    }

    fn struggling_signals() -> EmergentSignals {
        EmergentSignals {
            population: 20,
            surplus_index: 0.25,
            neighbor_pressure: 0.05,
        }
    }

    #[test]
    fn research_points_favor_prosperous_signals() {
        let rich = research_points_per_tick(&prosperous_signals());
        let poor = research_points_per_tick(&struggling_signals());
        assert!(rich > poor, "prosperous={rich} struggling={poor}");
    }

    #[test]
    fn surplus_lowers_emergent_threshold() {
        let rich = knowledge_threshold_for_tier(2, &prosperous_signals());
        let poor = knowledge_threshold_for_tier(2, &struggling_signals());
        assert!(rich < poor, "rich threshold={rich} poor threshold={poor}");
    }

    /// FR-TECH — prosperous faction out-researches a struggling peer over N ticks.
    #[test]
    fn prosperous_faction_outresearches_struggling_over_n_ticks() {
        let mut state = EmergentResearchState::new();
        let prosperous = prosperous_signals();
        let struggling = struggling_signals();

        for _ in 0..TICKS_PROSPERITY_TEST {
            state.accumulate_faction(0, &prosperous);
            state.accumulate_faction(1, &struggling);
            state.last_signals.insert(0, prosperous);
            state.last_signals.insert(1, struggling);
            state.try_unlock_all();
        }

        let prosperous_level = state.tech_level(0);
        let struggling_level = state.tech_level(1);
        assert!(
            prosperous_level > struggling_level,
            "prosperous tier {prosperous_level} must exceed struggling tier {struggling_level}"
        );
        assert!(
            state.knowledge(0) > state.knowledge(1),
            "prosperous knowledge {} > struggling {}",
            state.knowledge(0),
            state.knowledge(1)
        );
    }

    #[test]
    fn emergent_signals_scale_with_resources_and_trade() {
        let resources = Resources {
            food: Fixed::from_num(200),
            wood: Fixed::from_num(150),
            metal: Fixed::from_num(80),
            energy: Fixed::from_num(60),
        };
        let routes = vec![TradeRoute {
            from_faction: 0,
            to_faction: 1,
            goods: "grain".into(),
            volume: Fixed::from_num(10),
        }];
        let aggression = BTreeMap::from([(0, 0.4)]);
        let signals = emergent_signals_for_faction(
            0,
            64,
            &resources,
            Fixed::from_num(5_000),
            &routes,
            &aggression,
        );
        assert!(signals.surplus_index > 1.0);
        assert!(signals.neighbor_pressure > 0.0);
    }
}
