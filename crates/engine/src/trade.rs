//! Emergent settlement trade routes from price gaps + connectivity (FR-TRADE).
//!
//! Each tick the layer:
//! 1. Derives local food prices from global clearing + settlement stocks.
//! 2. Discovers profitable edges (price differential − transport > 0, path viable).
//! 3. Moves food along active routes and nudges global market pressure.

use std::collections::BTreeMap;

use civ_economy::MarketState;
use glam::IVec3;
use serde::{Deserialize, Serialize};

use crate::route::{
    arbitrage_margin_cents, path_viable, route_flow_units, squared_distance, MIN_ROUTE_FLOW,
};
use crate::settlement::{build_nodes, SettlementId, SettlementNode};

/// One active settlement trade route surfaced on the simulation snapshot.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SettlementTradeRoute {
    pub origin: SettlementId,
    pub destination: SettlementId,
    pub good: String,
    pub flow: i64,
    /// Arbitrage margin in cents that justified this route this tick.
    pub margin_cents: i64,
}

/// Mutable settlement-trade substrate owned by [`crate::engine::Simulation`].
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct SettlementTradeState {
    /// World positions for registered settlements (tests + scenario loaders).
    pub positions: BTreeMap<SettlementId, IVec3>,
    /// Routes that flowed this tick (recomputed every tick; not scripted).
    pub active_routes: Vec<SettlementTradeRoute>,
}

impl SettlementTradeState {
    /// Register or update a settlement's world position for connectivity checks.
    pub fn set_position(&mut self, settlement_id: SettlementId, position: IVec3) {
        self.positions.insert(settlement_id, position);
    }

    /// Discover routes, execute flows, apply market pressure. Returns active routes.
    pub fn tick(
        &mut self,
        settlements: &BTreeMap<SettlementId, u32>,
        food_stocked: &mut BTreeMap<SettlementId, i64>,
        market_state: &mut MarketState,
    ) -> &[SettlementTradeRoute] {
        self.active_routes.clear();
        if settlements.len() < 2 {
            return &self.active_routes;
        }

        let global_food = market_state.ensure_good("food");

        let nodes = build_nodes(settlements, &self.positions, food_stocked, global_food);
        let discovered = discover_routes(&nodes);
        execute_routes(&discovered, food_stocked);
        apply_market_pressure(&nodes, food_stocked, market_state);

        self.active_routes = discovered;
        &self.active_routes
    }
}

fn discover_routes(nodes: &[SettlementNode]) -> Vec<SettlementTradeRoute> {
    let mut routes = Vec::new();
    for origin in nodes {
        for destination in nodes {
            if origin.id == destination.id {
                continue;
            }
            if !path_viable(origin.position, destination.position) {
                continue;
            }
            let dist_sq = squared_distance(origin.position, destination.position);
            let margin = arbitrage_margin_cents(
                origin.local_food_price_cents,
                destination.local_food_price_cents,
                dist_sq,
            );
            if margin <= 0 {
                continue;
            }
            let flow = route_flow_units(origin, destination, margin, dist_sq);
            if flow < MIN_ROUTE_FLOW {
                continue;
            }
            routes.push(SettlementTradeRoute {
                origin: origin.id,
                destination: destination.id,
                good: "food".to_string(),
                flow,
                margin_cents: margin,
            });
        }
    }
    routes.sort_by(|a, b| {
        a.origin
            .cmp(&b.origin)
            .then(a.destination.cmp(&b.destination))
            .then(a.good.cmp(&b.good))
    });
    routes
}

fn execute_routes(
    routes: &[SettlementTradeRoute],
    food_stocked: &mut BTreeMap<SettlementId, i64>,
) {
    for route in routes {
        if route.flow <= 0 {
            continue;
        }
        if let Some(origin_stock) = food_stocked.get_mut(&route.origin) {
            *origin_stock = origin_stock.saturating_sub(route.flow);
        }
        if let Some(dest_stock) = food_stocked.get_mut(&route.destination) {
            *dest_stock = dest_stock.saturating_add(route.flow);
        }
    }
}

fn apply_market_pressure(
    nodes: &[SettlementNode],
    food_stocked: &BTreeMap<SettlementId, i64>,
    market_state: &mut MarketState,
) {
    let total_supply: i64 = nodes
        .iter()
        .map(|n| food_stocked.get(&n.id).copied().unwrap_or(0))
        .sum();
    let total_demand: i64 = nodes.iter().map(|n| crate::settlement::food_demand(n.population)).sum();
    market_state.apply_pressure("food", total_supply, total_demand);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::settlement::FOOD_DEMAND_PER_CAPITA;
    use civ_economy::MarketState;

    /// FR-TRADE — a profitable price gap plus viable path births a route from
    /// surplus to deficit settlement; goods flow and surface on active routes.
    #[test]
    fn fr_trade_route_emerges_between_surplus_and_deficit_settlement() {
        let mut state = SettlementTradeState::default();
        state.set_position(1, IVec3::ZERO);
        state.set_position(2, IVec3::new(8, 0, 0));

        let mut settlements = BTreeMap::from([(1, 100_u32), (2, 100_u32)]);
        let demand = 100_i64 * FOOD_DEMAND_PER_CAPITA;
        let mut food_stocked = BTreeMap::from([(1, demand + 4_000), (2, 5_i64)]);
        let mut market = MarketState::default();

        let routes = state
            .tick(&settlements, &mut food_stocked, &mut market)
            .to_vec();

        assert!(
            !routes.is_empty(),
            "expected emergent route between surplus (1) and deficit (2)"
        );
        let route = routes
            .iter()
            .find(|r| r.origin == 1 && r.destination == 2)
            .expect("route 1→2 must emerge from price gap + connectivity");
        assert_eq!(route.good, "food");
        assert!(route.flow >= MIN_ROUTE_FLOW);
        assert!(route.margin_cents > 0);
        assert!(
            food_stocked[&1] < demand + 4_000,
            "exporter stock must decrease after flow"
        );
        assert!(
            food_stocked[&2] > 5,
            "importer stock must increase after flow"
        );

        // Idempotent discovery on unchanged snapshot.
        let again = state
            .tick(&settlements, &mut food_stocked, &mut market)
            .to_vec();
        assert_eq!(routes, again);

        drop(settlements);
    }
}
