//! Economy engine subsystem for the simulation engine.
//!
//! This module contains helpers and types related to economy state management,
//! resource allocation, and market mechanics.

use crate::engine::{Fixed, ResourceType, WorldState};
use crate::SCALE;
use civ_economy::EconomyState;

/// Derive an [`EconomyState`] from the current [`WorldState`].
///
/// Syncs the energy budget and tick counter so the economy phase starts from
/// a consistent snapshot.
pub(crate) fn economy_state_from_world(world: &WorldState) -> EconomyState {
    let energy_budget_joules = i64::from(world.energy_budget_joules.to_bits()) / SCALE;
    let mut state = EconomyState::with_energy_budget(energy_budget_joules);
    state.tick = world.tick;
    state
}

/// Compute a baseline market price from supply/demand balance.
///
/// Returns a price in milliunits centered on `1_000` with a \u00b1250 band
/// driven by the demand surplus or deficit.
pub(crate) fn market_price_from_balance(supply: i64, demand: i64) -> i64 {
    let supply = supply.max(0);
    let demand = demand.max(0);
    let balance = demand.saturating_sub(supply);
    1_000i64.saturating_add(balance.clamp(-250, 250))
}

// ---- Simulation economy methods (extracted from engine.rs) ----

use crate::engine::Simulation;
use crate::engine::FOOD_SCARCITY_BASELINE;
use crate::engine::TECH_STORAGE;
use crate::engine::tech_unlocks_for_tier;
use crate::engine::{resource_market_key, route_resource, resource_amount, adjust_resource};
use civ_economy::{settlement_trade_flow_from_supply_demand, Good};
use civ_economy::AllocationEngine;
use civ_economy::LaborCapacityAllocator;

struct SettlementMarketSetup {
    id: u32,
    supply: i64,
    demand: i64,
    price: i64,
}

impl Simulation {
    pub(crate) fn phase_economy(&mut self) {
        let tick = self.state.tick;
        let policy_lines = self.mod_host.tick(tick);
        self.ingest_mod_phase_lines(policy_lines, tick, "policy");
        let economy_lines = self.mod_host.economy_tick(tick);
        self.ingest_mod_phase_lines(economy_lines, tick, "economy");

        self.economy_state.energy_budget_joules =
            i64::from(self.state.energy_budget_joules.to_bits()) / crate::SCALE;

        let demand = crate::policy::effective_consumption(self.economy_policy) as i64;
        let budget = self.economy_state.energy_budget_joules;

        // FR-CIV-LIFE P4-A: lifecycle-weighted allocation. The aggregate labor
        // fraction is derived from the per-tick lifecycle rollup computed in
        // phase_life (Adult count + 0.5 * Elder count, divided by total living
        // civilians). Children and the dead contribute 0; adults contribute 1.0;
        // elders contribute 0.5 (semi-retired, still productive).
        let metrics = &self.last_tick_lifecycle_metrics;
        let living = (metrics.children + metrics.adults + metrics.elders) as f64;
        let labor_fraction = if living > 0.0 {
            let productive = metrics.adults as f64 + 0.5 * metrics.elders as f64;
            (productive / living).clamp(0.0, 1.0)
        } else {
            0.0
        };
        let labor_allocator = LaborCapacityAllocator::new(labor_fraction);
        let allocated = labor_allocator.allocate(budget, demand);
        civ_economy::drain_energy_budget(&mut self.economy_state, allocated);
        civ_economy::step(&mut self.economy_state);

        self.state.energy_budget_joules = Fixed::from_num(self.economy_state.energy_budget_joules);
        let food_price_before = self
            .market_state
            .prices()
            .get("food")
            .copied()
            .unwrap_or(FOOD_SCARCITY_BASELINE);
        self.tick_settlement_trade_flows();
        self.tick_trade_routes();
        self.market_state.step(self.state.tick);
        if tech_unlocks_for_tier(self.research_tier()) & TECH_STORAGE != 0 {
            if let Some(price) = self.market_state.prices.get_mut("food") {
                let delta = *price - food_price_before;
                *price = food_price_before + delta / 2;
            }
        }
    }

    pub(crate) fn tick_settlement_trade_flows(&mut self) {
        self.last_tick_settlement_trade_flows.clear();

        let mut settlements: Vec<SettlementMarketSetup> = self
            .settlements
            .iter()
            .map(|(&settlement_id, &population)| {
                let supply = self
                    .settlement_food_stocked
                    .get(&settlement_id)
                    .copied()
                    .unwrap_or(0)
                    .max(0);
                let demand = i64::from(population);
                let price = market_price_from_balance(supply, demand);
                SettlementMarketSetup {
                    id: settlement_id,
                    supply,
                    demand,
                    price,
                }
            })
            .collect();
        settlements.sort_by_key(|entry| entry.id);

        for settlement in &settlements {
            self.market_state
                .apply_pressure("food", settlement.supply, settlement.demand);
        }

        for low_idx in 0..settlements.len() {
            for high_idx in (low_idx + 1)..settlements.len() {
                let low = &settlements[low_idx];
                let high = &settlements[high_idx];
                let Some(flow) = settlement_trade_flow_from_supply_demand(
                    u64::from(low.id),
                    u64::from(high.id),
                    Good::Food,
                    low.supply,
                    high.demand,
                    low.price,
                    high.price,
                    civ_economy::DEFAULT_SMOOTHING_FACTOR,
                ) else {
                    continue;
                };
                self.apply_settlement_flow(low.id, high.id, flow.qty);
                self.last_tick_settlement_trade_flows.push(flow);
            }
        }
    }

    pub(crate) fn apply_settlement_flow(&mut self, from_settlement: u32, to_settlement: u32, qty: i64) {
        let from_stock = self
            .settlement_food_stocked
            .entry(from_settlement)
            .or_insert(0);
        *from_stock = (*from_stock - qty).max(0);
        let to_stock = self
            .settlement_food_stocked
            .entry(to_settlement)
            .or_insert(0);
        *to_stock = to_stock.saturating_add(qty);
    }

    pub(crate) fn tick_trade_routes(&mut self) {
        for route in &self.state.trade_routes {
            if route.volume <= Fixed::ZERO || route.from_faction == route.to_faction {
                continue;
            }

            let resource = route_resource(&route.goods);
            let available = {
                let Some(from_resources) = self.state.faction_resources.get(&route.from_faction)
                else {
                    continue;
                };
                resource_amount(from_resources, resource)
            };
            if available <= Fixed::ZERO {
                continue;
            }

            let quantity = route.volume.min(available);
            {
                let from_resources = self
                    .state
                    .faction_resources
                    .entry(route.from_faction)
                    .or_default();
                adjust_resource(from_resources, resource, Fixed::ZERO - quantity);
            }
            {
                let to_resources = self
                    .state
                    .faction_resources
                    .entry(route.to_faction)
                    .or_default();
                adjust_resource(to_resources, resource, quantity);
            }

            let supply = {
                let Some(from_resources) = self.state.faction_resources.get(&route.from_faction)
                else {
                    continue;
                };
                resource_amount(from_resources, resource)
            };
            let demand = self
                .state
                .faction_resources
                .get(&route.to_faction)
                .map_or(Fixed::ZERO, |to_resources| {
                    resource_amount(to_resources, resource)
                });
            let supply_units = i64::from(supply.max(Fixed::ZERO).to_bits()) / crate::SCALE;
            let demand_units = i64::from(demand.max(Fixed::ZERO).to_bits()) / crate::SCALE;
            self.market_state.apply_pressure(
                resource_market_key(resource, 0),
                supply_units,
                demand_units,
            );
            let margin = (demand - supply).max(Fixed::ZERO);
            let profit = quantity * (Fixed::from_num(1) + margin / Fixed::from_num(100));

            if let Some(from_treasury) = self.state.faction_treasury.get_mut(&route.from_faction) {
                *from_treasury += profit;
            }
            if let Some(to_treasury) = self.state.faction_treasury.get_mut(&route.to_faction) {
                *to_treasury -= profit;
            }
        }
    }
}
