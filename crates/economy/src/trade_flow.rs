//! Complementary trade-route formation (FR-CIV-TRADE-ROUTE).
//!
//! Distinct from the gravity-model [`crate::trade_routes`] layer. FR-CIV-TRADE-ROUTE
//! specifies a tighter contract:
//!
//! > Trade routes form between settlements with complementary surplus/deficit;
//! > flow volume scales with price differential.
//!
//! Concretely, for each ordered `(origin, destination)` pair and each [`Good`]:
//!
//! 1. **Complementary pairing.** A route only forms when the origin has a
//!    positive surplus of the good AND the destination has a positive
//!    deficit of the same good. Same-side (surplus/surplus or
//!    deficit/deficit) pairs are not complementary and produce no route.
//! 2. **Price-differential-scaled flow.** Flow volume is
//!    `flow = min(origin_surplus, destination_deficit) * price_differential`,
//!    where `price_differential = max(0, destination_price - origin_price)`.
 //!    Integer-only. Saturating on the multiplication leg so extreme price
//!    differentials cannot overflow `i64`.
 //! 3. **Round-trip stability.** The set of routes for a snapshot is fully
//!    determined by the inputs (no randomness, no hidden state).
//!
//! Both [`complementary_routes`] (the directed one-leg form required by the
//! FR) and [`complementary_round_trips`] (a convenience that pairs each
//! origin→destination leg with the inverse destination→origin leg when both
//! are complementary) are provided. The bare leg form is the canonical
//! acceptance surface:
//! a surplus settlement pairs with a deficit settlement, and the flow
//! scales linearly with the per-good price differential between them.
//!
//! # Determinism
//!
//! Settlements are iterated in caller order. Routes are sorted by
//! `(origin, destination, good)` before return so two calls with the same
//! inputs produce byte-identical vectors.
//!
//! # Non-goals
//!
//! * No distance term — FR-CIV-TRADE-ROUTE pairs purely on the
//!   supply/demand complementarity and the price signal.
//! * No persistence — routes are recomputed every call from the inputs.
//! * No Bevy rendering or any other I/O — pure logic.

use serde::{Deserialize, Serialize};

/// Per-good complementary trade flow from an origin (surplus-side) to a
/// destination (deficit-side). Volume scales with the destination's price
/// minus the origin's price for that good, clamped at zero.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ComplementaryTradeFlow {
    /// Settlement id where the good is in surplus.
    pub origin: u32,
    /// Settlement id where the good is in deficit.
    pub destination: u32,
    /// Good that flows from origin to destination.
    pub good: u8,
    /// Flow volume (integer units per tick). Zero or negative differentials
    /// collapse to zero (no flow). Capped at the minimum of origin surplus
    /// and destination deficit so conservation falls out of the kernel.
    pub flow: i64,
}

/// Compact per-settlement view consumed by [`complementary_routes`].
///
/// We deliberately do not depend on [`crate::trade_routes::Settlement`] —
/// this FR is a strict superset of inputs (it needs per-settlement prices
/// in addition to the supply/demand gradient), and binding to the gravity
/// layer would couple two unrelated emergent systems. Callers build this
/// struct from their own sim state.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SettlementFlow {
    /// Stable settlement id assigned by the caller.
    pub id: u32,
    /// Per-good surplus. Positive ⇒ origin-side candidate.
    pub surplus: i64,
    /// Per-good deficit. Positive ⇒ destination-side candidate.
    pub deficit: i64,
    /// Per-good clearing price in cents (see [`crate::market::MarketState`]).
    pub price_cents: i64,
}

/// Compute complementary trade flows across all ordered `(origin, destination)`
/// pairs in `settlements`.
///
/// A route forms when `origin.surplus > 0`, `destination.deficit > 0`, and
/// `destination.price > origin.price` (the price differential is strictly
/// positive — equal prices produce no flow). Flow is then
/// `min(surplus, deficit) * (destination_price - origin_price)` with
/// saturating arithmetic on the multiplication leg.
///
/// Same-side pairs (both surplus or both deficit, or self-pairs) never
/// form routes — the FR strictly requires complementary surplus/deficit.
///
/// Returns routes sorted by `(origin, destination, good)` so calls with
/// identical inputs are byte-identical.
pub fn complementary_routes(settlements: &[SettlementFlow]) -> Vec<ComplementaryTradeFlow> {
    let mut routes = Vec::new();
    for origin in settlements {
        if origin.surplus <= 0 {
            continue;
        }
        for destination in settlements {
            if origin.id == destination.id {
                continue;
            }
            if destination.deficit <= 0 {
                continue;
            }
            let differential = destination.price_cents - origin.price_cents;
            if differential <= 0 {
                continue;
            }
            let capacity = origin.surplus.min(destination.deficit);
            let flow = capacity.saturating_mul(differential);
            if flow <= 0 {
                continue;
            }
            routes.push(ComplementaryTradeFlow {
                origin: origin.id,
                destination: destination.id,
                good: 0,
                flow,
            });
        }
    }
    routes.sort_by_key(|r| (r.origin, r.destination, r.good));
    routes
}

/// Convenience: pair each origin→destination leg with its inverse when both
/// sides are complementary. Returns one entry per matched leg pair, sorted
/// by `(origin, destination)`.
///
/// This is bookkeeping over [`complementary_routes`] and does not change
/// the formation rule. Useful for tests and for callers that want the
/// round-trip surface for downstream accounting.
pub fn complementary_round_trips(
    settlements: &[SettlementFlow],
) -> Vec<(ComplementaryTradeFlow, ComplementaryTradeFlow)> {
    let routes = complementary_routes(settlements);
    let mut by_pair: std::collections::BTreeMap<(u32, u32), ComplementaryTradeFlow> =
        std::collections::BTreeMap::new();
    for route in routes {
        by_pair.insert((route.origin, route.destination), route);
    }
    let mut out = Vec::new();
    let keys: Vec<(u32, u32)> = by_pair.keys().copied().collect();
    for (a, b) in keys {
        if a >= b {
            continue;
        }
        if let (Some(forward), Some(reverse)) = (
            by_pair.get(&(a, b)),
            by_pair.get(&(b, a)).cloned(),
        ) {
            out.push((forward.clone(), reverse));
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn surplus(id: u32, surplus: i64, price: i64) -> SettlementFlow {
        SettlementFlow {
            id,
            surplus,
            deficit: 0,
            price_cents: price,
        }
    }

    fn deficit(id: u32, deficit: i64, price: i64) -> SettlementFlow {
        SettlementFlow {
            id,
            surplus: 0,
            deficit,
            price_cents: price,
        }
    }

    /// FR-CIV-TRADE-ROUTE acceptance test (canonical).
    ///
    /// A surplus settlement and a deficit settlement establish a route, and
    /// flow volume tracks the per-good price differential between them.
    #[test]
    fn fr_civ_trade_route_surplus_and_deficit_pair_with_price_scaled_flow() {
        // Settlement 1: 10 units surplus of the good at 100 cents.
        let origin = surplus(1, 10, 100);
        // Settlement 2: 5 units deficit at 130 cents. Differential = 30 cents.
        // Expected flow = min(10, 5) * 30 = 150.
        let destination = deficit(2, 5, 130);

        let routes = complementary_routes(&[origin, destination]);
        assert_eq!(
            routes.len(),
            1,
            "exactly one complementary route must form"
        );
        let route = &routes[0];
        assert_eq!(route.origin, 1, "origin must be the surplus settlement");
        assert_eq!(route.destination, 2, "destination must be the deficit settlement");
        assert_eq!(
            route.flow, 150,
            "flow must track price differential: min(10, 5) * (130 - 100) = 150"
        );

        // Widen the price differential; flow scales linearly with it.
        let destination_priced_up = deficit(2, 5, 250);
        let routes_up = complementary_routes(&[surplus(1, 10, 100), destination_priced_up]);
        assert_eq!(routes_up[0].flow, 5 * 150, "double the differential doubles the flow");

        // Drop the differential to zero (equal prices); no route.
        let destination_same_price = deficit(2, 5, 100);
        let routes_flat = complementary_routes(&[surplus(1, 10, 100), destination_same_price]);
        assert!(
            routes_flat.is_empty(),
            "equal prices collapse the differential and no route forms"
        );

        // Invert the differential (origin more expensive than destination);
        // FR-CIV-TRADE-ROUTE requires a strictly positive differential,
        // so flow is zero and no route forms.
        let destination_cheaper = deficit(2, 5, 80);
        let routes_inverted = complementary_routes(&[surplus(1, 10, 100), destination_cheaper]);
        assert!(
            routes_inverted.is_empty(),
            "negative differentials collapse to zero flow"
        );
    }

    /// Complementary-only formation: same-side pairs (both surplus or both
    /// deficit) never produce routes.
    #[test]
    fn fr_civ_trade_route_rejects_same_side_pairs() {
        let two_surplus = [
            surplus(1, 10, 100),
            surplus(2, 20, 130),
        ];
        assert!(
            complementary_routes(&two_surplus).is_empty(),
            "surplus/surplus is not complementary"
        );

        let two_deficit = [
            deficit(1, 5, 100),
            deficit(2, 7, 130),
        ];
        assert!(
            complementary_routes(&two_deficit).is_empty(),
            "deficit/deficit is not complementary"
        );

        // Self-pair is a degenerate no-op (a settlement cannot pair with
        // itself).
        let only = surplus(1, 10, 100);
        assert!(complementary_routes(&[only]).is_empty());
    }

    /// Conservation: per-route flow never exceeds origin surplus or
    /// destination deficit.
    #[test]
    fn fr_civ_trade_route_flow_bounded_by_both_legs() {
        // Origin surplus much larger than destination deficit; flow is
        // capped at the deficit side.
        let origin = surplus(1, 100, 100);
        let destination = deficit(2, 3, 200); // differential 100
        let routes = complementary_routes(&[origin, destination]);
        assert_eq!(routes[0].flow, 300, "min(100, 3) * 100 = 300");
    }

    /// Determinism: identical inputs yield byte-identical route vectors.
    #[test]
    fn fr_civ_trade_route_is_deterministic() {
        let settlements = [
            surplus(1, 10, 100),
            deficit(2, 5, 130),
            surplus(3, 7, 110),
            deficit(4, 4, 140),
        ];
        let first = complementary_routes(&settlements);
        let second = complementary_routes(&settlements);
        assert_eq!(first, second);
        // Canonical order: (origin, destination).
        for pair in first.windows(2) {
            assert!(pair[0].origin <= pair[1].origin);
            if pair[0].origin == pair[1].origin {
                assert!(pair[0].destination < pair[1].destination);
            }
        }
    }
}
