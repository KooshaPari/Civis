//! Route viability: connectivity and price-gap profitability (FR-TRADE).
//!
//! Routes are not authored. A candidate edge exists only when settlements are
//! within travel range **and** the destination's local price exceeds the
//! origin's by more than transport cost.

use glam::IVec3;

use crate::settlement::SettlementNode;

/// Maximum squared distance (voxel units) for a viable trade path.
pub const MAX_ROUTE_DISTANCE_SQ: i64 = 10_000;

/// Transport cost in cents per squared-distance unit.
pub const TRANSPORT_COST_PER_DIST_SQ: i64 = 1;

/// Minimum route flow (food units) to surface an active route.
pub const MIN_ROUTE_FLOW: i64 = 1;

/// Squared Euclidean distance between two voxel positions.
#[must_use]
pub fn squared_distance(a: IVec3, b: IVec3) -> i64 {
    let dx = (a.x as i64).saturating_sub(b.x as i64);
    let dy = (a.y as i64).saturating_sub(b.y as i64);
    let dz = (a.z as i64).saturating_sub(b.z as i64);
    dx.saturating_mul(dx)
        .saturating_add(dy.saturating_mul(dy))
        .saturating_add(dz.saturating_mul(dz))
}

/// Returns `true` when settlements are close enough to trade (connectivity).
#[must_use]
pub fn path_viable(origin: IVec3, destination: IVec3) -> bool {
    squared_distance(origin, destination) <= MAX_ROUTE_DISTANCE_SQ
}

/// Transport cost in cents for a given squared distance (floored at 1 cent).
#[must_use]
pub fn transport_cost_cents(dist_sq: i64) -> i64 {
    dist_sq.saturating_mul(TRANSPORT_COST_PER_DIST_SQ).max(1)
}

/// Arbitrage margin in cents: `dest_price - origin_price - transport`.
#[must_use]
pub fn arbitrage_margin_cents(origin_price: i64, dest_price: i64, dist_sq: i64) -> i64 {
    let transport = transport_cost_cents(dist_sq);
    dest_price
        .saturating_sub(origin_price)
        .saturating_sub(transport)
}

/// Compute integer food flow along a profitable edge, bounded by surplus/deficit.
#[must_use]
pub fn route_flow_units(
    origin: &SettlementNode,
    destination: &SettlementNode,
    margin_cents: i64,
    dist_sq: i64,
) -> i64 {
    if margin_cents <= 0 {
        return 0;
    }
    let surplus = origin.food_surplus();
    let deficit = destination.food_deficit();
    if surplus <= 0 || deficit <= 0 {
        return 0;
    }
    let dist_sq = dist_sq.max(1);
    let kernel = surplus
        .saturating_mul(deficit)
        .saturating_div(dist_sq);
    let margin_flow = margin_cents.saturating_div(10);
    kernel.min(margin_flow).min(surplus).min(deficit)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::settlement::SettlementNode;

    fn node(id: u32, pos: IVec3, pop: u32, stock: i64, price: i64) -> SettlementNode {
        SettlementNode {
            id,
            position: pos,
            population: pop,
            food_stock: stock,
            local_food_price_cents: price,
        }
    }

    #[test]
    fn path_viable_within_range_only() {
        assert!(path_viable(IVec3::ZERO, IVec3::new(50, 0, 0)));
        assert!(!path_viable(IVec3::ZERO, IVec3::new(200, 0, 0)));
    }

    #[test]
    fn profitable_margin_produces_positive_flow() {
        let origin = node(1, IVec3::ZERO, 100, 5_000, 960);
        let dest = node(2, IVec3::new(5, 0, 0), 100, 10, 1_200);
        let dist_sq = squared_distance(origin.position, dest.position);
        let margin = arbitrage_margin_cents(origin.local_food_price_cents, dest.local_food_price_cents, dist_sq);
        let flow = route_flow_units(&origin, &dest, margin, dist_sq);
        assert!(margin > 0);
        assert!(flow >= MIN_ROUTE_FLOW);
    }
}
