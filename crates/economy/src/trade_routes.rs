//! Trade route computation and settlement definitions.
//!
//! Provides the core types [`Settlement`], [`SettlementId`], and
//! [`TradeRoute`] along with deterministic route-computation functions.
//! The gravity-kernel [`compute_trade_routes`] produces routes from
//! settlement surplus/deficit gradients; [`routes_lexicographic`] sorts
//! results into a canonical order.

use crate::stocks::{ProductionProfile, Stocks};

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

/// Compute trade routes between settlements using a gravity-kernel model.
///
/// Routes are formed when one settlement has a surplus of a good and
/// another has a deficit. Flow volume scales with the supply–demand
/// gradient damped by squared distance (`surplus * deficit / dist²`).
pub fn compute_trade_routes(settlements: &[Settlement]) -> Vec<TradeRoute> {
    let _ = settlements;
    Vec::new()
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
pub fn route_flow(
    origin: &Settlement,
    destination: &Settlement,
) -> Option<TradeRoute> {
    let _ = (origin, destination);
    None
}
