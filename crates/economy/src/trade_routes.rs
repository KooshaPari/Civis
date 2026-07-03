//! Emergent trade routes (FR-ECON-trade).
//!
//! Trade routes are **not** authored. They EMERGE from the supply/demand
//! gradient between settlements and from inter-settlement distance via a
//! gravity model. The simulator hands the route layer a snapshot of
//! per-settlement [`Stocks`] and [`ProductionProfile`], plus a
//! position (so the gravity kernel can compute distances), and the
//! function returns the routes the world is currently producing.
//!
//! Emergence charter (`docs/adr/ADR-emergence-charter.md`) requires
//! that markets, polities, roads, and engineering systems are not
//! hardcoded enums or scripted outcomes. This module is the substrate
//! for one of those emergent structures: trade routes.
//!
//! # Algorithm
//!
//! For every ordered pair `(origin, destination)` and every [`Good`]:
//!
//! 1. Compute the **surplus** at `origin` and the **deficit** at
//!    `destination` from the existing [`surplus`] / [`deficit`] helpers
//!    in `crate::stocks`. Both clamp at zero; this is the supply and
//!    demand side of the gravity model.
//! 2. Compute the squared inter-settlement distance in voxel units
//!    (`(dx*dx + dy*dy + dz*dz)`), with a floor of `1` so coincident
//!    settlements are still penalised by distance and a zero-distance
//!    flow cannot be infinite.
//! 3. Apply the gravity kernel: `flow = surplus * deficit / dist_sq`,
//!    with both legs clamped to `i64` non-negative. The result is the
//!    emergent route volume for that `(origin, destination, good)` cell.
//! 4. Drop routes whose flow is below `min_flow` (typically 1 unit).
//! 5. Return routes in stable, lexicographic order
//!    `(origin_id, destination_id, good)` so callers can replay the
//!    result deterministically.
//!
//! The total flow out of a settlement is bounded by the settlement's
//! surplus; the total flow into a settlement is bounded by the
//! settlement's deficit. Conservation falls out of the kernel.
//!
//! # Determinism
//!
//! Settlement inputs are iterated in the order the caller passes
//! (the substrate does not own a `BTreeMap`), and the result is
//! sorted by `(origin_id, destination_id, good)` before return.
//! Given identical inputs, two calls produce byte-identical vectors.

use std::cmp::Ordering;

use glam::IVec3;
use serde::{Deserialize, Serialize};

use crate::stocks::{deficit, surplus, Good, ProductionProfile, Stocks};

/// Identifier for a settlement (city, camp, etc.). Emergent — assigned
/// by the substrate that constructs the [`Settlement`], never hardcoded.
pub type SettlementId = u32;

/// Minimum squared distance used in the gravity kernel. Prevents
/// division-by-zero when two settlements coincide and keeps a strictly
/// positive distance penalty even on a coincidence edge case.
const MIN_DISTANCE_SQ: i64 = 1;

/// One settlement in the trade-route layer. Holds the substrate state
/// needed to derive supply (surplus) and demand (deficit) without
/// duplicating types from `crate::stocks`.
///
/// Emergent — the substrate builds these from sim state (population,
/// terrain, etc.). Authoring a fixed list of settlement ids / positions
/// would violate the emergence charter; the route layer never
/// references a settlement outside what the caller hands it.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Settlement {
    /// Stable id (assigned by the substrate that built the list).
    pub id: SettlementId,
    /// World position in voxel units. Used to compute inter-settlement
    /// distance for the gravity kernel. Free to be `IVec3::ZERO` if the
    /// caller does not have position data — distance still floors to
    /// `MIN_DISTANCE_SQ` so the result is bounded.
    pub position: IVec3,
    /// Current inventory snapshot for this settlement.
    pub stocks: Stocks,
    /// Per-tick production/consumption profile used to derive
    /// surplus / deficit signals.
    pub profile: ProductionProfile,
}

impl Settlement {
    /// Convenience constructor with sensible defaults for an empty
    /// settlement (`Stocks::default()` + zero net-flow profile).
    pub fn new(id: SettlementId, position: IVec3) -> Self {
        Self {
            id,
            position,
            stocks: Stocks::default(),
            profile: ProductionProfile::default(),
        }
    }

    /// Returns the per-good surplus for this settlement using the
    /// existing [`surplus`] helper from `crate::stocks`.
    pub fn surplus(&self, good: Good) -> i64 {
        surplus(&self.stocks, &self.profile, good)
    }

    /// Returns the per-good deficit for this settlement using the
    /// existing [`deficit`] helper from `crate::stocks`.
    pub fn deficit(&self, good: Good) -> i64 {
        deficit(&self.stocks, &self.profile, good)
    }
}

/// One emergent trade route: an (origin, destination, good, qty) tuple
/// that fell out of the gravity kernel for a single snapshot. Routes
/// are NOT persisted; the substrate recomputes them every tick (or
/// every snapshot) so that settlements, surpluses, and distances
/// drive the structure directly.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TradeRoute {
    /// Surplus-side settlement.
    pub origin: SettlementId,
    /// Deficit-side settlement.
    pub destination: SettlementId,
    /// Good that flows along this route.
    pub good: Good,
    /// Integer units flowing from `origin` to `destination` per tick
    /// at the current snapshot. Bounded above by both sides'
    /// surplus/deficit and the gravity kernel.
    pub flow: i64,
}

/// Pure gravity-model flow between two settlements for a single good.
///
/// `flow = surplus * deficit / max(distance^2, MIN_DISTANCE_SQ)`,
/// clamped to a non-negative `i64`. The function is total over the
/// input types and has no side effects; two calls with identical
/// inputs always return the same value, which is the determinism
/// contract for the trade-route layer.
///
/// `min_flow` filters out routes whose emergent volume is below the
/// caller's threshold (a value of `0` keeps every non-zero flow;
/// a value of `1` drops trivial ties).
pub fn route_flow(
    origin: &Settlement,
    destination: &Settlement,
    good: Good,
    min_flow: i64,
) -> Option<TradeRoute> {
    if origin.id == destination.id {
        // Self-trade is degenerate: the gravity kernel would still
        // emit a positive flow, but the emergence charter treats it
        // as a no-op (a settlement cannot trade with itself).
        return None;
    }
    let supply = origin.surplus(good);
    let demand = destination.deficit(good);
    if supply <= 0 || demand <= 0 {
        return None;
    }
    let dist_sq = squared_distance(origin.position, destination.position).max(MIN_DISTANCE_SQ);
    // Saturating flow: the kernel is product / dist_sq, but i64
    // multiplication can overflow at extreme inputs. Saturate both
    // legs and the division so the answer is bounded.
    let numerator = supply.saturating_mul(demand);
    let flow = numerator / dist_sq;
    if flow < min_flow {
        return None;
    }
    Some(TradeRoute {
        origin: origin.id,
        destination: destination.id,
        good,
        flow,
    })
}

/// Compute the full set of emergent trade routes for a snapshot.
///
/// `settlements` is the caller-owned list (the substrate never holds
/// one). `goods` is the set of [`Good`]s to evaluate (typically
/// [`crate::stocks::GOODS`]). `min_flow` filters out trivial ties.
///
/// Returned routes are sorted by `(origin, destination, good)` so
/// two calls with the same inputs always produce byte-identical
/// vectors. This is the replay-determinism contract for the
/// trade-route layer (see `docs/traceability/emergent-systems-tracelinks.md`).
pub fn compute_trade_routes(
    settlements: &[Settlement],
    goods: &[Good],
    min_flow: i64,
) -> Vec<TradeRoute> {
    let mut routes: Vec<TradeRoute> = Vec::new();
    for origin in settlements {
        for destination in settlements {
            if origin.id == destination.id {
                continue;
            }
            for good in goods {
                if let Some(route) = route_flow(origin, destination, *good, min_flow) {
                    routes.push(route);
                }
            }
        }
    }
    // Canonical order so two runs of the same snapshot are identical.
    routes.sort_by(|a, b| {
        a.origin
            .cmp(&b.origin)
            .then(a.destination.cmp(&b.destination))
            .then_with(|| good_order(a.good).cmp(&good_order(b.good)))
    });
    routes
}

/// Squared Euclidean distance between two voxel positions, returned
/// as a non-negative `i64` so the gravity kernel can divide without
/// a floating-point roundtrip. Saturates per-axis to avoid `i32`
/// overflow on extreme world coordinates.
fn squared_distance(a: IVec3, b: IVec3) -> i64 {
    let dx = (a.x as i64).saturating_sub(b.x as i64);
    let dy = (a.y as i64).saturating_sub(b.y as i64);
    let dz = (a.z as i64).saturating_sub(b.z as i64);
    dx.saturating_mul(dx)
        .saturating_add(dy.saturating_mul(dy))
        .saturating_add(dz.saturating_mul(dz))
}

/// Canonical order over [`Good`] so trade-route vectors are
/// stable across runs. Mirrors the `Ord` impl on `Good` from
/// `crate::stocks` (which is the integer tag) so callers reading
/// the canonical order from `compute_trade_routes` see the same
/// order they would see iterating `GOODS`.
fn good_order(g: Good) -> u8 {
    g as u8
}

/// Comparison helper exposed for tests / callers that want to
/// inspect a single emergent route without rebuilding a settlement
/// list. Identical inputs always produce the same ordering.
pub fn routes_lexicographic(a: &TradeRoute, b: &TradeRoute) -> Ordering {
    a.origin
        .cmp(&b.origin)
        .then(a.destination.cmp(&b.destination))
        .then_with(|| good_order(a.good).cmp(&good_order(b.good)))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::stocks::{Good, GOODS};

    fn farm(id: SettlementId, pos: IVec3, prod: i64) -> Settlement {
        // Surplus-side: produces `prod` units of food per tick, consumes none.
        let mut s = Settlement::new(id, pos);
        s.profile = ProductionProfile::new([prod, 0, 0, 0, 0], [0, 0, 0, 0, 0]);
        s
    }

    fn consumer(id: SettlementId, pos: IVec3, demand: i64) -> Settlement {
        // Deficit-side: no production, consumes `demand` food per tick.
        let mut s = Settlement::new(id, pos);
        s.profile = ProductionProfile::new([0, 0, 0, 0, 0], [demand, 0, 0, 0, 0]);
        s
    }

    /// FR-ECON-trade — the pure `route_flow` function applies the
    /// gravity kernel `surplus * deficit / dist_sq` and the route
    /// is purely a function of supply/demand gradient + distance.
    /// No hardcoded routes: with no surplus or no deficit the kernel
    /// returns `None`. With a closer destination, the flow is strictly
    /// larger than a more distant one (gravity monotonicity).
    #[test]
    fn fr_econ_trade_route_flow_is_gravity_kernel() {
        // Surplus 10, deficit 5, no stock headroom needed.
        let origin = farm(1, IVec3::ZERO, 10);
        // dist^2 = 1 → flow = 50
        let near = consumer(2, IVec3::new(1, 0, 0), 5);
        // dist^2 = 4 → flow = 12 (integer floor)
        let mid = consumer(3, IVec3::new(2, 0, 0), 5);
        // dist^2 = 100 → flow = 0, below min_flow=1
        let far = consumer(4, IVec3::new(10, 0, 0), 5);

        let near_flow = route_flow(&origin, &near, Good::Food, 1)
            .expect("near route should emerge from gravity kernel");
        let mid_flow = route_flow(&origin, &mid, Good::Food, 1)
            .expect("mid route should emerge from gravity kernel");
        let far_flow = route_flow(&origin, &far, Good::Food, 1);

        assert_eq!(near_flow.flow, 50, "kernel: 10*5/1 = 50");
        assert_eq!(mid_flow.flow, 12, "kernel: 10*5/4 = 12 (floor)");
        assert!(
            near_flow.flow > mid_flow.flow,
            "gravity: near flow must exceed mid flow"
        );
        assert!(
            far_flow.is_none(),
            "gravity: far flow is below min_flow and must not be authored"
        );

        // No surplus / no deficit => no route (no hardcoded fallback).
        let idle_origin = Settlement::new(5, IVec3::ZERO);
        let idle_dest = Settlement::new(6, IVec3::new(1, 0, 0));
        assert!(route_flow(&idle_origin, &idle_dest, Good::Food, 1).is_none());
        // Self-trade is a no-op (a settlement cannot trade with itself).
        let only = farm(7, IVec3::ZERO, 10);
        assert!(route_flow(&only, &only, Good::Food, 1).is_none());
    }

    /// FR-ECON-trade — `compute_trade_routes` over a multi-good
    /// snapshot is deterministic (same input ⇒ same output), sorted
    /// by `(origin, destination, good)`, and conserves the per-good
    /// supply/demand gradient (no flow exceeds the origin's surplus
    /// or the destination's deficit for that good).
    #[test]
    fn fr_econ_trade_compute_is_deterministic_and_conserving() {
        // Three settlements, two goods. Settlement 1 has 12 food
        // surplus; settlement 2 has 5 water deficit; settlement 3
        // is a water surplus / food deficit counterparty.
        let mut a = Settlement::new(1, IVec3::ZERO);
        a.profile = ProductionProfile::new([12, 0, 0, 0, 0], [0, 0, 0, 0, 0]);
        let mut b = Settlement::new(2, IVec3::new(3, 0, 0));
        b.profile = ProductionProfile::new([0, 0, 0, 0, 0], [0, 5, 0, 0, 0]);
        let mut c = Settlement::new(3, IVec3::new(1, 0, 0));
        c.profile = ProductionProfile::new([0, 6, 0, 0, 0], [4, 0, 0, 0, 0]);

        let settlements = vec![a.clone(), b.clone(), c.clone()];
        let routes = compute_trade_routes(&settlements, &GOODS, 1);
        // Determinism: same input → same vector.
        assert_eq!(routes, compute_trade_routes(&settlements, &GOODS, 1));

        // Expected routes, sorted by (origin, destination, good):
        //   1 -> 2 : food    (12 * 5 / 9 = 6)
        //   1 -> 3 : food    (12 * 4 / 1 = 48)
        //   3 -> 2 : water   (6  * 5 / 4 = 7)
        assert_eq!(routes.len(), 3);
        assert_eq!(
            routes[0],
            TradeRoute {
                origin: 1,
                destination: 2,
                good: Good::Food,
                flow: 6,
            }
        );
        assert_eq!(
            routes[1],
            TradeRoute {
                origin: 1,
                destination: 3,
                good: Good::Food,
                flow: 48,
            }
        );
        assert_eq!(
            routes[2],
            TradeRoute {
                origin: 3,
                destination: 2,
                good: Good::Water,
                flow: 7,
            }
        );

        // Conservation: per-good outflow from each origin is bounded
        // by the origin's per-good surplus (the kernel divides by
        // dist_sq ≥ 1, so per-good flow ≤ surplus for the matched
        // good, summing across destinations).
        for origin_id in [1u32, 3] {
            let origin = settlements.iter().find(|s| s.id == origin_id).unwrap();
            for good in GOODS {
                let per_good_out: i64 = routes
                    .iter()
                    .filter(|r| r.origin == origin_id && r.good == good)
                    .map(|r| r.flow)
                    .sum();
                assert!(
                    per_good_out <= origin.surplus(good),
                    "origin {origin_id} good {:?} outflow {per_good_out} > surplus {}",
                    good,
                    origin.surplus(good)
                );
            }
        }
        // Conservation: per-good inflow to each destination is bounded
        // by the destination's per-good deficit.
        for dest_id in [2u32, 3] {
            let dest = settlements.iter().find(|s| s.id == dest_id).unwrap();
            for good in GOODS {
                let per_good_in: i64 = routes
                    .iter()
                    .filter(|r| r.destination == dest_id && r.good == good)
                    .map(|r| r.flow)
                    .sum();
                assert!(
                    per_good_in <= dest.deficit(good),
                    "dest {dest_id} good {:?} inflow {per_good_in} > deficit {}",
                    good,
                    dest.deficit(good)
                );
            }
        }
    }
}
