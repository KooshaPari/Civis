//! Settlement nodes for emergent inter-settlement trade (FR-TRADE).
//!
//! Local prices derive from the global clearing price plus a stock-driven
//! scarcity offset so surplus settlements quote lower and deficit settlements
//! quote higher — the price differential that trade routes exploit.

use std::collections::BTreeMap;

use glam::IVec3;
use serde::{Deserialize, Serialize};

/// Stable settlement identifier (assigned by scenario/tests, never hardcoded).
pub type SettlementId = u32;

/// Food units each resident consumes per tick when deriving surplus/deficit.
pub const FOOD_DEMAND_PER_CAPITA: i64 = 10;

/// Maximum absolute cents added to the global food price from local scarcity.
pub const LOCAL_PRICE_DELTA_CAP_CENTS: i64 = 200;

/// One settlement participating in the trade-route layer.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SettlementNode {
    pub id: SettlementId,
    pub position: IVec3,
    pub population: u32,
    pub food_stock: i64,
    /// Locally-adjusted food price in cents for this tick.
    pub local_food_price_cents: i64,
}

impl SettlementNode {
    /// Food surplus available for export (non-negative).
    #[must_use]
    pub fn food_surplus(&self) -> i64 {
        let demand = food_demand(self.population);
        self.food_stock.saturating_sub(demand)
    }

    /// Food deficit that imports can satisfy (non-negative).
    #[must_use]
    pub fn food_deficit(&self) -> i64 {
        let demand = food_demand(self.population);
        demand.saturating_sub(self.food_stock)
    }
}

/// Per-capita food demand for a settlement population.
#[must_use]
pub fn food_demand(population: u32) -> i64 {
    (population as i64).saturating_mul(FOOD_DEMAND_PER_CAPITA)
}

/// Derive a settlement-local food price from the global baseline and stock.
///
/// Higher stock relative to demand lowers the local quote; deficit raises it.
#[must_use]
pub fn local_food_price_cents(global_food_price: i64, food_stock: i64, population: u32) -> i64 {
    let demand = food_demand(population).max(1);
    let supply = food_stock.max(0);
    let pressure = demand.saturating_sub(supply);
    let delta = pressure
        .saturating_mul(50)
        .saturating_div(supply.max(1))
        .clamp(-LOCAL_PRICE_DELTA_CAP_CENTS, LOCAL_PRICE_DELTA_CAP_CENTS);
    global_food_price.saturating_add(delta).max(1)
}

/// Build settlement nodes from engine maps. Missing position defaults to origin.
#[must_use]
pub fn build_nodes(
    settlements: &BTreeMap<SettlementId, u32>,
    positions: &BTreeMap<SettlementId, IVec3>,
    food_stocked: &BTreeMap<SettlementId, i64>,
    global_food_price: i64,
) -> Vec<SettlementNode> {
    let mut nodes: Vec<SettlementNode> = settlements
        .iter()
        .map(|(&id, &population)| {
            let food_stock = food_stocked.get(&id).copied().unwrap_or(0);
            let position = positions.get(&id).copied().unwrap_or(IVec3::ZERO);
            let local_food_price_cents =
                local_food_price_cents(global_food_price, food_stock, population);
            SettlementNode {
                id,
                position,
                population,
                food_stock,
                local_food_price_cents,
            }
        })
        .collect();
    nodes.sort_by_key(|n| n.id);
    nodes
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn surplus_lowers_local_price_deficit_raises_it() {
        let global = 1_000;
        let surplus_price = local_food_price_cents(global, 5_000, 100);
        let deficit_price = local_food_price_cents(global, 10, 100);
        assert!(
            surplus_price < global,
            "surplus settlement should quote below global: {surplus_price}"
        );
        assert!(
            deficit_price > global,
            "deficit settlement should quote above global: {deficit_price}"
        );
        assert!(surplus_price < deficit_price);
    }
}
