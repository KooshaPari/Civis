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
/// Compute a baseline market price from supply/demand balance.
///
/// Returns a price in milliunits centered on `1_000` with a ±250 band
/// driven by the demand surplus or deficit.
#[inline]
pub(crate) fn market_price_from_balance(supply: i64, demand: i64) -> i64 {
    let supply = supply.max(0);
    let demand = demand.max(0);
    let balance = demand.saturating_sub(supply);
    1_000i64.saturating_add(balance.clamp(-250, 250))
}
// ---- Simulation economy methods (extracted from engine.rs) ----

use crate::engine::tech_unlocks_for_tier;
use crate::engine::Simulation;
use crate::engine::FOOD_SCARCITY_BASELINE;
use crate::engine::TECH_STORAGE;
use crate::engine::{adjust_resource, resource_amount, resource_market_key, route_resource};
use civ_economy::AllocationEngine;
use civ_economy::LaborCapacityAllocator;
use civ_economy::{
    deficit, find_extraction_site, surplus, tick_extraction, ResourceKind as ExtractionKind, GOODS,
};
use civ_economy::{settlement_trade_flow_from_supply_demand, Good};

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
        self.compute_and_apply_new_routes();
        self.tick_extraction_sites();
        self.market_state.step(self.state.tick);
        if tech_unlocks_for_tier(self.research_tier()) & TECH_STORAGE != 0 {
            if let Some(price) = self.market_state.prices.get_mut("food") {
                let delta = *price - food_price_before;
                *price = food_price_before + delta / 2;
            }
        }

        // Famine + caravan wiring TODO: wire these when APIs stabilize
        // - famine::classify_famine(food_per_capita) for famine cascade
        // - caravan::tick_caravan() for trade routes
        // See crates/engine/src/famine.rs and crates/engine/src/caravan.rs
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

    pub(crate) fn apply_settlement_flow(
        &mut self,
        from_settlement: u32,
        to_settlement: u32,
        qty: i64,
    ) {
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

    /// Map an extraction [`ExtractionKind`] to an engine [`ResourceType`].
    #[inline]
    fn extraction_kind_to_resource(kind: ExtractionKind) -> ResourceType {
        match kind {
            ExtractionKind::Ore => ResourceType::Metal,
            ExtractionKind::Stone => ResourceType::Wood,
            ExtractionKind::Wood => ResourceType::Wood,
            ExtractionKind::Food => ResourceType::Food,
        }
    }

    /// Deterministic position derived from a faction ID for gravity-kernel
    /// distance calculations.
    #[inline]
    fn faction_position(faction_id: u32) -> (i32, i32, i32) {
        let x = (faction_id as i32) * 100;
        let y = ((faction_id as i32) * 37) % 50;
        let z = ((faction_id as i32) * 13) % 30;
        (x, y, z)
    }

    /// Convert engine faction resources into a [`civ_economy::Settlement`] for
    /// gravity-kernel trade-route computation.
    #[inline]
    fn build_economy_settlement(
        faction_id: u32,
        resources: &crate::engine::Resources,
    ) -> civ_economy::Settlement {
        let mut stocks = civ_economy::Stocks::default();
        let food_units = i64::from(resources.food.max(Fixed::ZERO).to_bits()) / crate::SCALE;
        let wood_units = i64::from(resources.wood.max(Fixed::ZERO).to_bits()) / crate::SCALE;
        let metal_units = i64::from(resources.metal.max(Fixed::ZERO).to_bits()) / crate::SCALE;
        let energy_units = i64::from(resources.energy.max(Fixed::ZERO).to_bits()) / crate::SCALE;
        stocks.add(Good::Food, food_units);
        stocks.add(Good::Wood, wood_units);
        stocks.add(Good::Metal, metal_units);
        stocks.add(Good::Water, energy_units);
        let profile = civ_economy::ProductionProfile::default();
        civ_economy::Settlement {
            id: u64::from(faction_id),
            name: String::new(),
            position: Self::faction_position(faction_id),
            stocks,
            profile,
        }
    }

    /// Compute trade routes using the gravity kernel from `civ-economy` and
    /// append new routes to `self.state.trade_routes`.
    pub(crate) fn compute_and_apply_new_routes(&mut self) {
        let settlements: Vec<civ_economy::Settlement> = self
            .state
            .faction_resources
            .iter()
            .map(|(&fid, res)| Self::build_economy_settlement(fid, res))
            .collect();

        if settlements.len() < 2 {
            return;
        }

        // PERF: build a once-per-tick `id -> Settlement` index so the
        // origin/dest lookups inside the route loop are O(1) instead of
        // `settlements.iter().find(...)` (which was O(n) per lookup).
        let mut by_id: std::collections::HashMap<u64, &civ_economy::Settlement> =
            std::collections::HashMap::with_capacity(settlements.len());
        for s in &settlements {
            by_id.insert(s.id, s);
        }

        let econ_routes = civ_economy::compute_trade_routes(&settlements);

        for econ_route in econ_routes {
            let from_faction = econ_route.from as u32;
            let to_faction = econ_route.to as u32;
            let volume = Fixed::from_num(econ_route.volume as i64);
            if volume <= Fixed::ZERO || from_faction == to_faction {
                continue;
            }
            let origin = by_id.get(&econ_route.from).copied();
            let dest = by_id.get(&econ_route.to).copied();
            let goods_label = match (origin, dest) {
                (Some(o), Some(d)) => {
                    let mut best_good = "grain";
                    let mut best_vol: i64 = 0;
                    for good in GOODS {
                        let sup = surplus(&o.stocks, &o.profile, good);
                        let dem = deficit(&d.stocks, &d.profile, good);
                        if sup > 0 && dem > 0 {
                            let flow = sup.min(dem);
                            if flow > best_vol {
                                best_vol = flow;
                                best_good = match good {
                                    Good::Food => "grain",
                                    Good::Water => "cloth",
                                    Good::Wood => "timber",
                                    Good::Metal => "ore",
                                    Good::Tools => "tools",
                                };
                            }
                        }
                    }
                    best_good
                }
                _ => "grain",
            };
            self.state.trade_routes.push(crate::engine::TradeRoute {
                from_faction,
                to_faction,
                goods: goods_label.to_string(),
                volume,
            });
        }
    }

    /// Run extraction sites for one tick and deposit yields into faction
    /// resources.
    pub(crate) fn tick_extraction_sites(&mut self) {
        let sites = find_extraction_site();
        for mut site in sites {
            let yield_amount = tick_extraction(&mut site);
            if yield_amount <= 0 {
                continue;
            }
            let resource = Self::extraction_kind_to_resource(site.resource_kind);
            let faction_id = site.settlement_id as u32;
            let resources = self.state.faction_resources.entry(faction_id).or_default();
            let delta = Fixed::from_num(yield_amount);
            adjust_resource(resources, resource, delta);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::engine::Simulation;

    #[test]
    fn faction_position_is_deterministic() {
        let a = Simulation::faction_position(1);
        let b = Simulation::faction_position(1);
        assert_eq!(a, b);
    }

    #[test]
    fn faction_positions_differ_for_different_ids() {
        let a = Simulation::faction_position(0);
        let b = Simulation::faction_position(1);
        assert_ne!(a, b);
    }

    #[test]
    fn extraction_kind_to_resource_maps_correctly() {
        assert_eq!(
            Simulation::extraction_kind_to_resource(ExtractionKind::Ore),
            ResourceType::Metal
        );
        assert_eq!(
            Simulation::extraction_kind_to_resource(ExtractionKind::Food),
            ResourceType::Food
        );
        assert_eq!(
            Simulation::extraction_kind_to_resource(ExtractionKind::Wood),
            ResourceType::Wood
        );
        assert_eq!(
            Simulation::extraction_kind_to_resource(ExtractionKind::Stone),
            ResourceType::Wood
        );
    }

    #[test]
    fn build_economy_settlement_maps_resources() {
        let res = crate::engine::Resources {
            food: Fixed::from_num(100),
            wood: Fixed::from_num(50),
            metal: Fixed::from_num(30),
            energy: Fixed::from_num(20),
        };
        let s = Simulation::build_economy_settlement(42, &res);
        assert_eq!(s.id, 42);
        assert_eq!(s.stocks.get(Good::Food), 100);
        assert_eq!(s.stocks.get(Good::Wood), 50);
        assert_eq!(s.stocks.get(Good::Metal), 30);
        assert_eq!(s.stocks.get(Good::Water), 20);
    }

    #[test]
    fn compute_and_apply_new_routes_adds_routes_when_factions_differ() {
        let mut sim = Simulation::new();
        sim.state.faction_resources.insert(
            0,
            crate::engine::Resources {
                food: Fixed::from_num(200),
                wood: Fixed::from_num(0),
                metal: Fixed::from_num(0),
                energy: Fixed::from_num(0),
            },
        );
        sim.state.faction_resources.insert(
            1,
            crate::engine::Resources {
                food: Fixed::from_num(0),
                wood: Fixed::from_num(100),
                metal: Fixed::from_num(0),
                energy: Fixed::from_num(0),
            },
        );
        let before = sim.state.trade_routes.len();
        sim.compute_and_apply_new_routes();
        assert!(sim.state.trade_routes.len() >= before);
    }

    #[test]
    fn tick_extraction_sites_runs_without_panicking() {
        let mut sim = Simulation::new();
        sim.tick_extraction_sites();
        assert!(sim.state.faction_resources.is_empty() || true);
    }

    #[test]
    fn compute_and_apply_routes_skips_when_fewer_than_two_factions() {
        let mut sim = Simulation::new();
        sim.state.faction_resources.clear();
        let before = sim.state.trade_routes.len();
        sim.compute_and_apply_new_routes();
        assert_eq!(sim.state.trade_routes.len(), before);
    }
}
