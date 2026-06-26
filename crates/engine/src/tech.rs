//! Emergent per-faction tech accumulation (FR-ERA).
//!
//! Technology is not a scripted tree: each faction accrues research progress
//! from population density, treasury, and resource surplus. Progress converts
//! into discrete tech levels that feed era evaluation.

use std::collections::BTreeMap;

use civ_agents::{Alignment, Civilian as AgentCivilian};
use serde::{Deserialize, Serialize};

use crate::engine::{Resources, Simulation};

/// Per-faction emergent tech state (not a fixed tech tree).
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct FactionTechState {
    /// Accumulated research points available to spend on tech growth.
    pub research_points: u64,
    /// Discrete tech levels unlocked through emergent accumulation.
    pub tech_level: u32,
    /// Deterministic diffusion pressure carried across ticks.
    pub diffusion_points: u64,
}

/// Inputs gathered from live simulation state for one faction.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FactionEmergenceInputs {
    pub population: u32,
    pub surplus: i64,
    pub treasury: i64,
}

/// Progress required to advance from `tech_level` to the next level.
#[must_use]
pub fn research_cost_for_level(tech_level: u32) -> u64 {
    let next = u64::from(tech_level.saturating_add(1));
    120 * next * next
}

/// Per-tick research income from thriving vs stagnant conditions.
#[must_use]
pub fn research_income(inputs: FactionEmergenceInputs) -> u64 {
    if inputs.population < 4 && inputs.surplus < 50 && inputs.treasury < 500 {
        return 0;
    }
    let pop = u64::from(inputs.population);
    let surplus = (inputs.surplus.max(0) as u64) / 4;
    let treasury = (inputs.treasury.max(0) as u64) / 100;
    pop.saturating_mul(2).saturating_add(surplus).saturating_add(treasury)
}

/// Bass-like diffusion weight for one faction observing a better neighbor.
#[must_use]
pub fn diffusion_delta(
    current_level: u32,
    neighbor_level: u32,
    neighbor_pressure: u64,
    research_points: u64,
) -> u64 {
    if neighbor_level <= current_level {
        return 0;
    }
    let gap = u64::from(neighbor_level - current_level);
    let adoption_base = 6_u64;
    let imitation = neighbor_pressure / 3 + research_points / 24;
    adoption_base
        .saturating_mul(gap)
        .saturating_add(imitation)
        .max(1)
}

/// Neighbor diffusion is anchored to faction-id adjacency in sorted order.
#[must_use]
pub fn neighboring_factions(faction_ids: &[u32], faction_id: u32) -> impl Iterator<Item = u32> + '_ {
    let idx = faction_ids.iter().position(|candidate| *candidate == faction_id);
    let left = idx.and_then(|i| i.checked_sub(1)).and_then(|i| faction_ids.get(i).copied());
    let right = idx
        .and_then(|i| i.checked_add(1))
        .and_then(|i| faction_ids.get(i).copied());
    left.into_iter().chain(right)
}

/// Resource surplus above a modest subsistence floor (food + wood + metal).
#[must_use]
pub fn resource_surplus(resources: &Resources) -> i64 {
    let food = resources.food.to_num::<i64>();
    let wood = resources.wood.to_num::<i64>();
    let metal = resources.metal.to_num::<i64>();
    let baseline = 40_i64;
    (food - baseline).max(0) + (wood - baseline).max(0) + (metal - baseline).max(0)
}

/// Count aligned civilians per faction from the ECS world.
#[must_use]
pub fn faction_populations(sim: &Simulation) -> BTreeMap<u32, u32> {
    let mut counts = BTreeMap::new();
    for (_, civ) in sim.world.query::<&AgentCivilian>().iter() {
        if let Alignment::Faction(faction_id) = civ.alignment {
            *counts.entry(faction_id).or_insert(0) += 1;
        }
    }
    counts
}

/// Gather emergence inputs for every known faction id in world state.
#[must_use]
pub fn gather_faction_inputs(sim: &Simulation) -> BTreeMap<u32, FactionEmergenceInputs> {
    let populations = faction_populations(sim);
    let mut inputs = BTreeMap::new();
    for (&faction_id, name) in &sim.state.factions {
        let _ = name;
        let population = populations.get(&faction_id).copied().unwrap_or(0);
        let resources = sim
            .state
            .faction_resources
            .get(&faction_id)
            .cloned()
            .unwrap_or_default();
        let surplus = resource_surplus(&resources);
        let treasury = sim
            .state
            .faction_treasury
            .get(&faction_id)
            .map(|v| v.to_num::<i64>())
            .unwrap_or(0);
        inputs.insert(
            faction_id,
            FactionEmergenceInputs {
                population,
                surplus,
                treasury,
            },
        );
    }
    inputs
}

/// Advance per-faction research progress from emergent pressures.
pub fn tick_research(sim: &mut Simulation, tech_by_faction: &mut BTreeMap<u32, FactionTechState>) {
    let inputs = gather_faction_inputs(sim);
    let faction_ids: Vec<u32> = inputs.keys().copied().collect();
    let snapshot = tech_by_faction.clone();
    for (faction_id, faction_inputs) in inputs {
        let state = tech_by_faction.entry(faction_id).or_default();
        let delta = research_income(faction_inputs);
        state.research_points = state.research_points.saturating_add(delta);

        let mut diffusion = 0_u64;
        for neighbor_id in neighboring_factions(&faction_ids, faction_id) {
            if let Some(neighbor) = snapshot.get(&neighbor_id) {
                diffusion = diffusion.saturating_add(diffusion_delta(
                    state.tech_level,
                    neighbor.tech_level,
                    neighbor.diffusion_points,
                    state.research_points,
                ));
            }
        }
        state.diffusion_points = state.diffusion_points.saturating_add(diffusion);
    }
}

/// Convert accumulated research and diffusion pressure into discrete tech levels.
pub fn tick_tech(tech_by_faction: &mut BTreeMap<u32, FactionTechState>) {
    for state in tech_by_faction.values_mut() {
        loop {
            let threshold = research_cost_for_level(state.tech_level);
            let available = state.research_points.saturating_add(state.diffusion_points);
            if available < threshold {
                break;
            }
            let spend_research = state.research_points.min(threshold);
            state.research_points -= spend_research;
            let remaining = threshold - spend_research;
            state.diffusion_points = state.diffusion_points.saturating_sub(remaining);
            state.tech_level = state.tech_level.saturating_add(1);
        }
    }
}
