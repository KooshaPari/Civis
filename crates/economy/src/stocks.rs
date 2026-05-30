//! Mercantile resource stocks for individual and collective (settlement /
//! faction) economics — the calculus that lets a living item *persist* and that
//! drives emergent social clustering.
//!
//! All conserved quantities are integer (`i64`) — no floating-point
//! accumulation, keeping replays bit-identical (ADR-008). Trade is Ricardian:
//! actors specialize in their comparative-advantage good and swap, and a
//! [`TradeOffer`] is only proposed when both sides net-benefit; [`apply_trade`]
//! conserves the total quantity of every good across the two actors.
//!
//! Traceability: `FR-CIV-LIFE-020..025`.

use serde::{Deserialize, Serialize};

/// A tradeable good. Quantities are integer units.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Good {
    /// Caloric food.
    Food,
    /// Drinkable water.
    Water,
    /// Timber / wood.
    Wood,
    /// Ore / metal.
    Metal,
    /// Manufactured tools.
    Tools,
}

/// All goods in canonical order (matches the [`Stocks`] backing-array order).
pub const GOODS: [Good; 5] = [Good::Food, Good::Water, Good::Wood, Good::Metal, Good::Tools];

impl Good {
    /// Index into the per-good arrays used by [`Stocks`] / [`ProductionProfile`].
    #[must_use]
    pub const fn index(self) -> usize {
        match self {
            Good::Food => 0,
            Good::Water => 1,
            Good::Wood => 2,
            Good::Metal => 3,
            Good::Tools => 4,
        }
    }
}

/// A per-actor or per-collective resource inventory. Integer quantities only.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct Stocks {
    qty: [i64; 5],
}

impl Stocks {
    /// Construct stocks from an explicit per-good quantity array (canonical order).
    #[must_use]
    pub fn from_array(qty: [i64; 5]) -> Self {
        Self {
            qty: qty.map(|q| q.max(0)),
        }
    }

    /// Current quantity of `good`.
    #[must_use]
    pub fn get(&self, good: Good) -> i64 {
        self.qty[good.index()]
    }

    /// Add `amount` of `good` (may be negative to withdraw). Quantity is clamped
    /// at zero on withdrawal; returns the amount actually applied (negative when
    /// a withdrawal was partially clamped).
    pub fn add(&mut self, good: Good, amount: i64) -> i64 {
        let i = good.index();
        let before = self.qty[i];
        let next = before.saturating_add(amount).max(0);
        self.qty[i] = next;
        next - before
    }

    /// Sum of all good quantities (used by conservation assertions / HUD).
    #[must_use]
    pub fn total(&self) -> i64 {
        self.qty.iter().sum()
    }
}

/// Per-good production and consumption rates per tick (integer units).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct ProductionProfile {
    production: [i64; 5],
    consumption: [i64; 5],
}

impl ProductionProfile {
    /// Build a profile from explicit production and consumption arrays.
    #[must_use]
    pub fn new(production: [i64; 5], consumption: [i64; 5]) -> Self {
        Self {
            production: production.map(|q| q.max(0)),
            consumption: consumption.map(|q| q.max(0)),
        }
    }

    /// Production rate for `good`.
    #[must_use]
    pub fn production(&self, good: Good) -> i64 {
        self.production[good.index()]
    }

    /// Consumption rate for `good`.
    #[must_use]
    pub fn consumption(&self, good: Good) -> i64 {
        self.consumption[good.index()]
    }

    /// Net per-tick flow for `good`: positive = surplus, negative = deficit.
    #[must_use]
    pub fn net_flow(&self, good: Good) -> i64 {
        self.production(good) - self.consumption(good)
    }
}

/// Apply one tick of a [`ProductionProfile`] to [`Stocks`]: production is added,
/// consumption withdrawn (clamped at zero — an actor cannot consume goods it
/// does not have). Conserving and never negative.
pub fn step_stocks(stocks: &mut Stocks, profile: &ProductionProfile) {
    for good in GOODS {
        stocks.add(good, profile.production(good));
        stocks.add(good, -profile.consumption(good));
    }
}

/// Per-tick surplus of `good` (`net_flow` clamped to be non-negative).
#[must_use]
pub fn surplus(profile: &ProductionProfile, good: Good) -> i64 {
    profile.net_flow(good).max(0)
}

/// Per-tick deficit of `good` (magnitude of negative `net_flow`, else 0).
#[must_use]
pub fn deficit(profile: &ProductionProfile, good: Good) -> i64 {
    (-profile.net_flow(good)).max(0)
}

/// The good in which a profile has the greatest net surplus rate — its
/// comparative advantage. Ties break deterministically toward [`GOODS`] order.
#[must_use]
pub fn comparative_advantage(profile: &ProductionProfile) -> Good {
    let mut best = Good::Food;
    let mut best_flow = profile.net_flow(Good::Food);
    for &good in &GOODS[1..] {
        let flow = profile.net_flow(good);
        if flow > best_flow {
            best_flow = flow;
            best = good;
        }
    }
    best
}

/// Ricardian gains-from-trade estimate when two collectives each specialize in
/// their comparative-advantage good and swap. Positive when their advantages
/// differ (each covers the other's deficit), zero when identical.
#[must_use]
pub fn trade_gain(a: &ProductionProfile, b: &ProductionProfile) -> i64 {
    let adv_a = comparative_advantage(a);
    let adv_b = comparative_advantage(b);
    if adv_a == adv_b {
        return 0;
    }
    // Each side ships its surplus to cover the other's deficit; the realizable
    // gain is the matched volume in both directions.
    let a_to_b = surplus(a, adv_a).min(deficit(b, adv_a));
    let b_to_a = surplus(b, adv_b).min(deficit(a, adv_b));
    a_to_b + b_to_a
}

/// A mutually-beneficial swap between two actors.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct TradeOffer {
    /// Good actor A ships to actor B.
    pub good_a_to_b: Good,
    /// Quantity A ships to B.
    pub qty_a_to_b: i64,
    /// Good actor B ships to actor A.
    pub good_b_to_a: Good,
    /// Quantity B ships to A.
    pub qty_b_to_a: i64,
}

/// Propose a trade where A exports its comparative-advantage good to cover B's
/// deficit and vice-versa. `Some` only when both directions carry a positive,
/// stock-backed volume (genuine mutual benefit); otherwise `None`.
#[must_use]
pub fn propose_trade(
    a_stocks: &Stocks,
    a_profile: &ProductionProfile,
    b_stocks: &Stocks,
    b_profile: &ProductionProfile,
) -> Option<TradeOffer> {
    let good_a_to_b = comparative_advantage(a_profile);
    let good_b_to_a = comparative_advantage(b_profile);
    if good_a_to_b == good_b_to_a {
        return None;
    }

    // A ships its surplus good to cover B's deficit, bounded by A's on-hand stock.
    let qty_a_to_b = surplus(a_profile, good_a_to_b)
        .min(deficit(b_profile, good_a_to_b))
        .min(a_stocks.get(good_a_to_b));
    let qty_b_to_a = surplus(b_profile, good_b_to_a)
        .min(deficit(a_profile, good_b_to_a))
        .min(b_stocks.get(good_b_to_a));

    if qty_a_to_b <= 0 || qty_b_to_a <= 0 {
        return None;
    }
    Some(TradeOffer {
        good_a_to_b,
        qty_a_to_b,
        good_b_to_a,
        qty_b_to_a,
    })
}

/// Execute a [`TradeOffer`], moving goods between two actors. Conserves the
/// total quantity of every good across the pair (asserted in tests).
pub fn apply_trade(a: &mut Stocks, b: &mut Stocks, offer: &TradeOffer) {
    let from_a = offer.qty_a_to_b.min(a.get(offer.good_a_to_b)).max(0);
    a.add(offer.good_a_to_b, -from_a);
    b.add(offer.good_a_to_b, from_a);

    let from_b = offer.qty_b_to_a.min(b.get(offer.good_b_to_a)).max(0);
    b.add(offer.good_b_to_a, -from_b);
    a.add(offer.good_b_to_a, from_b);
}

#[cfg(test)]
mod tests {
    use super::*;

    /// FR-CIV-LIFE-020 — step applies net flow, conserving and never negative.
    #[test]
    fn step_conserves_and_never_negative() {
        let mut s = Stocks::from_array([5, 0, 10, 0, 0]);
        // Consume more water than on hand -> clamps at zero, no panic.
        let profile = ProductionProfile::new([2, 0, 0, 1, 0], [0, 3, 4, 0, 0]);
        step_stocks(&mut s, &profile);
        assert_eq!(s.get(Good::Food), 7); // +2
        assert_eq!(s.get(Good::Water), 0); // -3 clamped at 0
        assert_eq!(s.get(Good::Wood), 6); // 10-4
        assert_eq!(s.get(Good::Metal), 1); // +1
        for g in GOODS {
            assert!(s.get(g) >= 0, "{g:?} negative");
        }
    }

    /// FR-CIV-LIFE-021 — surplus/deficit signs are correct.
    #[test]
    fn surplus_and_deficit_signs() {
        let profile = ProductionProfile::new([5, 1, 0, 0, 0], [2, 4, 0, 0, 0]);
        assert_eq!(surplus(&profile, Good::Food), 3);
        assert_eq!(deficit(&profile, Good::Food), 0);
        assert_eq!(surplus(&profile, Good::Water), 0);
        assert_eq!(deficit(&profile, Good::Water), 3);
    }

    /// FR-CIV-LIFE-022 — comparative_advantage picks the max-surplus good.
    #[test]
    fn comparative_advantage_picks_max_surplus() {
        let profile = ProductionProfile::new([3, 9, 1, 0, 0], [1, 1, 0, 0, 0]);
        // Water net = 8, the largest.
        assert_eq!(comparative_advantage(&profile), Good::Water);
    }

    /// FR-CIV-LIFE-023 — trade_gain > 0 for differing advantages, 0 when identical.
    #[test]
    fn trade_gain_positive_for_differing_advantages() {
        // A: food surplus, water deficit. B: water surplus, food deficit.
        let a = ProductionProfile::new([10, 0, 0, 0, 0], [0, 5, 0, 0, 0]);
        let b = ProductionProfile::new([0, 10, 0, 0, 0], [5, 0, 0, 0, 0]);
        assert!(trade_gain(&a, &b) > 0);
        // Identical advantage -> no gain.
        assert_eq!(trade_gain(&a, &a), 0);
    }

    /// FR-CIV-LIFE-024 — propose_trade None when no mutual benefit.
    #[test]
    fn propose_trade_none_without_mutual_benefit() {
        let a = ProductionProfile::new([10, 0, 0, 0, 0], [0, 5, 0, 0, 0]);
        // B has the SAME comparative advantage -> nothing to gain.
        let b = a;
        let sa = Stocks::from_array([100, 100, 0, 0, 0]);
        let sb = sa;
        assert!(propose_trade(&sa, &a, &sb, &b).is_none());
    }

    /// FR-CIV-LIFE-025 — apply_trade conserves total goods across both actors.
    #[test]
    fn apply_trade_conserves_total_goods() {
        let a_profile = ProductionProfile::new([10, 0, 0, 0, 0], [0, 5, 0, 0, 0]);
        let b_profile = ProductionProfile::new([0, 10, 0, 0, 0], [5, 0, 0, 0, 0]);
        let mut sa = Stocks::from_array([20, 0, 0, 0, 0]);
        let mut sb = Stocks::from_array([0, 20, 0, 0, 0]);

        let before_food = sa.get(Good::Food) + sb.get(Good::Food);
        let before_water = sa.get(Good::Water) + sb.get(Good::Water);

        let offer = propose_trade(&sa, &a_profile, &sb, &b_profile).expect("mutual benefit");
        assert!(offer.qty_a_to_b > 0 && offer.qty_b_to_a > 0);
        apply_trade(&mut sa, &mut sb, &offer);

        assert_eq!(sa.get(Good::Food) + sb.get(Good::Food), before_food);
        assert_eq!(sa.get(Good::Water) + sb.get(Good::Water), before_water);
        // Goods actually moved between actors.
        assert!(sb.get(Good::Food) > 0);
        assert!(sa.get(Good::Water) > 0);
    }
}
