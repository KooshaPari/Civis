//! Trade route computation and settlement definitions.
//!
//! Provides the core types [`Settlement`], [`SettlementId`], and
//! [`TradeRoute`] along with deterministic route-computation functions.
//! The gravity-kernel [`compute_trade_routes`] produces routes from
//! settlement surplus/deficit gradients; [`routes_lexicographic`] sorts
//! results into a canonical order.

use crate::stocks::{deficit, surplus, ProductionProfile, Stocks, GOODS};

/// Settlement identifier — a stable 64-bit handle.
pub type SettlementId = u64;

/// A settlement participating in trade.
///
/// Carries its inventory ([`Stocks`]), production profile, and spatial
/// position on the voxel grid for distance-weighted route computation.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct Settlement {
    /// Unique settlement identifier.
    pub id: SettlementId,
    /// Human-readable settlement name.
    pub name: String,
    /// Position in the voxel world `(x, y, z)`.
    pub position: (i32, i32, i32),
    /// Current inventory stock snapshot.
    pub stocks: Stocks,
    /// Per-tick production and consumption profile.
    pub profile: ProductionProfile,
}

/// A trade route between two settlements.
///
/// Records the origin (`from`), destination (`to`), and total volume
/// flowing along this route during the current tick.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct TradeRoute {
    /// Unique route identifier.
    pub id: u64,
    /// Origin settlement id.
    pub from: u64,
    /// Destination settlement id.
    pub to: u64,
    /// Trade volume flowing on this route.
    pub volume: f64,
}

/// Squared Euclidean distance between two 3D positions.
///
/// Returns `f64` for gravity-kernel division. Positions are integer
/// voxel coordinates so the result is always exact.
fn distance_squared(a: (i32, i32, i32), b: (i32, i32, i32)) -> f64 {
    let dx = (a.0 - b.0) as f64;
    let dy = (a.1 - b.1) as f64;
    let dz = (a.2 - b.2) as f64;
    dx * dx + dy * dy + dz * dz
}

/// Minimum squared distance to prevent division-by-zero and to cap
/// maximum flow for co-located settlements.
const MIN_DIST_SQ: f64 = 1.0;

/// Compute trade routes between settlements using a gravity-kernel model.
///
/// Routes are formed when one settlement has a surplus of a good and
/// another has a deficit. Flow volume scales with the supply–demand
/// gradient damped by squared distance (`surplus * deficit / dist²`).
///
/// Each viable (origin, destination, good) triple produces one route.
/// Routes are returned in deterministic iteration order; callers should
/// use [`routes_lexicographic`] for a canonical sort.
pub fn compute_trade_routes(settlements: &[Settlement]) -> Vec<TradeRoute> {
    let mut routes = Vec::new();
    let mut next_id: u64 = 1;

    for origin in settlements {
        for dest in settlements {
            if origin.id == dest.id {
                continue;
            }

            let dist_sq = distance_squared(origin.position, dest.position).max(MIN_DIST_SQ);

            for good in GOODS {
                let sup = surplus(&origin.stocks, &origin.profile, good);
                let dem = deficit(&dest.stocks, &dest.profile, good);

                if sup > 0 && dem > 0 {
                    let volume = (sup as f64) * (dem as f64) / dist_sq;
                    if volume > 0.0 {
                        routes.push(TradeRoute {
                            id: next_id,
                            from: origin.id,
                            to: dest.id,
                            volume,
                        });
                        next_id += 1;
                    }
                }
            }
        }
    }

    routes
}

/// Sort trade routes into a deterministic lexicographic order.
///
/// Primary key: `from` (origin settlement id).
/// Secondary key: `to` (destination settlement id).
/// Tertiary key: `id` (route identifier).
pub fn routes_lexicographic(routes: &[TradeRoute]) -> Vec<TradeRoute> {
    let mut result = routes.to_vec();
    result.sort_by(|a, b| {
        a.from
            .cmp(&b.from)
            .then_with(|| a.to.cmp(&b.to))
            .then_with(|| a.id.cmp(&b.id))
    });
    result
}

/// Compute the flow for a single route between two settlements.
///
/// Returns a [`TradeRoute`] when the origin has surplus and the
/// destination has deficit for at least one good, with flow equal
/// to `min(surplus, deficit)`. Returns `None` when no complementary
/// trade exists.
pub fn route_flow(origin: &Settlement, destination: &Settlement) -> Option<TradeRoute> {
    let mut best_flow: i64 = 0;

    for good in GOODS {
        let sup = surplus(&origin.stocks, &origin.profile, good);
        let dem = deficit(&destination.stocks, &destination.profile, good);

        if sup > 0 && dem > 0 {
            let flow = sup.min(dem);
            if flow > best_flow {
                best_flow = flow;
            }
        }
    }

    if best_flow > 0 {
        Some(TradeRoute {
            id: 0,
            from: origin.id,
            to: destination.id,
            volume: best_flow as f64,
        })
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::stocks::{Good, ProductionProfile, Stocks};

    fn make_settlement(
        id: u64,
        pos: (i32, i32, i32),
        stocks: Stocks,
        profile: ProductionProfile,
    ) -> Settlement {
        Settlement {
            id,
            name: format!("Settlement {id}"),
            position: pos,
            stocks,
            profile,
        }
    }

    #[test]
    fn compute_trade_routes_empty_for_no_surplus_deficit() {
        let a = make_settlement(1, (0, 0, 0), Stocks::default(), ProductionProfile::default());
        let b = make_settlement(2, (10, 0, 0), Stocks::default(), ProductionProfile::default());
        let routes = compute_trade_routes(&[a, b]);
        assert!(routes.is_empty());
    }

    #[test]
    fn compute_trade_routes_gravity_kernel() {
        let mut origin_stocks = Stocks::default();
        origin_stocks.add(Good::Food, 5);
        let origin_profile = ProductionProfile::new([10, 0, 0, 0, 0], [0, 0, 0, 0, 0]);
        let origin = make_settlement(1, (0, 0, 0), origin_stocks, origin_profile);

        let dest_stocks = Stocks::default();
        let dest_profile = ProductionProfile::new([0, 0, 0, 0, 0], [5, 0, 0, 0, 0]);
        let dest = make_settlement(2, (10, 0, 0), dest_stocks, dest_profile);

        let routes = compute_trade_routes(&[origin, dest]);
        assert_eq!(routes.len(), 1);
        let r = &routes[0];
        assert_eq!(r.from, 1);
        assert_eq!(r.to, 2);
        // gravity kernel: surplus(10) * deficit(5) / dist²(100) = 1.0
        assert!((r.volume - 1.0).abs() < 1e-10);
    }

    #[test]
    fn compute_trade_routes_skips_same_settlement() {
        let mut s =
            make_settlement(1, (0, 0, 0), Stocks::default(), ProductionProfile::default());
        s.stocks.add(Good::Food, 50);
        s.profile = ProductionProfile::new([10, 0, 0, 0, 0], [0, 0, 0, 0, 0]);
        let routes = compute_trade_routes(&[s]);
        assert!(routes.is_empty());
    }

    #[test]
    fn compute_trade_routes_distance_damping() {
        let mut origin_stocks = Stocks::default();
        origin_stocks.add(Good::Food, 5);
        let origin_profile = ProductionProfile::new([10, 0, 0, 0, 0], [0, 0, 0, 0, 0]);
        let origin = make_settlement(1, (0, 0, 0), origin_stocks, origin_profile);

        let dest_stocks = Stocks::default();
        let dest_profile = ProductionProfile::new([0, 0, 0, 0, 0], [5, 0, 0, 0, 0]);

        let close_dest =
            make_settlement(2, (2, 0, 0), dest_stocks.clone(), dest_profile.clone());
        let far_dest = make_settlement(3, (100, 0, 0), dest_stocks, dest_profile);

        let routes = compute_trade_routes(&[origin, close_dest, far_dest]);
        let close_route = routes.iter().find(|r| r.to == 2).unwrap();
        let far_route = routes.iter().find(|r| r.to == 3).unwrap();
        assert!(close_route.volume > far_route.volume);
    }

    #[test]
    fn routes_lexicographic_sorts_correctly() {
        let routes = vec![
            TradeRoute { id: 3, from: 2, to: 1, volume: 5.0 },
            TradeRoute { id: 1, from: 1, to: 2, volume: 10.0 },
            TradeRoute { id: 2, from: 1, to: 1, volume: 3.0 },
        ];
        let sorted = routes_lexicographic(&routes);
        assert_eq!(sorted[0].from, 1);
        assert_eq!(sorted[0].to, 1);
        assert_eq!(sorted[1].from, 1);
        assert_eq!(sorted[1].to, 2);
        assert_eq!(sorted[2].from, 2);
        assert_eq!(sorted[2].to, 1);
    }

    #[test]
    fn route_flow_returns_none_when_no_complement() {
        let origin =
            make_settlement(1, (0, 0, 0), Stocks::default(), ProductionProfile::default());
        let dest =
            make_settlement(2, (5, 0, 0), Stocks::default(), ProductionProfile::default());
        assert_eq!(route_flow(&origin, &dest), None);
    }

    #[test]
    fn route_flow_returns_flow_for_surplus_deficit() {
        let mut origin_stocks = Stocks::default();
        origin_stocks.add(Good::Food, 5);
        let origin_profile = ProductionProfile::new([10, 0, 0, 0, 0], [0, 0, 0, 0, 0]);
        let origin = make_settlement(1, (0, 0, 0), origin_stocks, origin_profile);

        let dest_stocks = Stocks::default();
        let dest_profile = ProductionProfile::new([0, 0, 0, 0, 0], [5, 0, 0, 0, 0]);
        let dest = make_settlement(2, (5, 0, 0), dest_stocks, dest_profile);

        let flow = route_flow(&origin, &dest);
        assert!(flow.is_some());
        let flow = flow.unwrap();
        assert_eq!(flow.from, 1);
        assert_eq!(flow.to, 2);
        assert!((flow.volume - 5.0).abs() < 1e-10);
    }

    #[test]
    fn route_flow_picks_best_good() {
        let mut origin_stocks = Stocks::default();
        origin_stocks.add(Good::Food, 5);
        origin_stocks.add(Good::Water, 10);
        let origin_profile = ProductionProfile::new([10, 20, 0, 0, 0], [0, 0, 0, 0, 0]);
        let origin = make_settlement(1, (0, 0, 0), origin_stocks, origin_profile);

        let dest_stocks = Stocks::default();
        let dest_profile = ProductionProfile::new([0, 0, 0, 0, 0], [8, 3, 0, 0, 0]);
        let dest = make_settlement(2, (5, 0, 0), dest_stocks, dest_profile);

        let flow = route_flow(&origin, &dest).unwrap();
        assert!((flow.volume - 8.0).abs() < 1e-10);
    }
}
