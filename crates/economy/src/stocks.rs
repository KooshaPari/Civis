//! Resource stocks and mercantile trade for living actors and collectives.
//!
//! This module models integer-only inventories, per-tick production/consumption
//! profiles, and deterministic exchange rules that support specialization and
//! trade between individuals, settlements, and factions.

use serde::{Deserialize, Serialize};
use tracing::debug;

/// Traded good categories supported by the stock system.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[repr(u8)]
pub enum Good {
    /// Edible resources.
    Food = 0,
    /// Potable resources.
    Water = 1,
    /// Raw construction material.
    Wood = 2,
    /// Refined structural material.
    Metal = 3,
    /// Durable manufactured equipment.
    Tools = 4,
    /// Refined energetic output of advanced industrial production chains.
    Energy = 5,
}

/// All built-in goods in deterministic iteration order.
pub const GOODS: [Good; 6] = [
    Good::Food,
    Good::Water,
    Good::Wood,
    Good::Metal,
    Good::Tools,
    Good::Energy,
];

fn good_index(good: Good) -> usize {
    good as usize
}

/// Integer stock inventory for an actor or collective.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Stocks {
    quantities: [i64; GOODS.len()],
}

impl Default for Stocks {
    fn default() -> Self {
        Self {
            quantities: [0; GOODS.len()],
        }
    }
}

impl Stocks {
    /// Returns the current quantity of `good`.
    pub fn get(&self, good: Good) -> i64 {
        self.quantities[good_index(good)]
    }

    /// Sets the current quantity of `good` to `qty`, clamping negatives to 0.
    pub fn set(&mut self, good: Good, qty: i64) {
        self.quantities[good_index(good)] = qty.max(0);
    }

    /// Returns the [`Good::Food`] stock quantity.
    pub fn food(&self) -> i64 {
        self.get(Good::Food)
    }

    /// Returns the [`Good::Water`] stock quantity.
    pub fn water(&self) -> i64 {
        self.get(Good::Water)
    }

    /// Returns the [`Good::Wood`] stock quantity.
    pub fn wood(&self) -> i64 {
        self.get(Good::Wood)
    }

    /// Returns the [`Good::Metal`] stock quantity.
    pub fn metal(&self) -> i64 {
        self.get(Good::Metal)
    }

    /// Returns the [`Good::Tools`] stock quantity.
    pub fn tools(&self) -> i64 {
        self.get(Good::Tools)
    }

    /// Returns the [`Good::Energy`] stock quantity.
    pub fn energy(&self) -> i64 {
        self.get(Good::Energy)
    }

    /// Applies a signed quantity change to `good`.
    ///
    /// Positive deltas add stock. Negative deltas withdraw stock and clamp at
    /// zero. The returned value is the actual signed delta applied.
    pub fn add(&mut self, good: Good, delta: i64) -> i64 {
        let slot = &mut self.quantities[good_index(good)];
        if delta >= 0 {
            let before = *slot;
            *slot = slot.saturating_add(delta);
            *slot - before
        } else {
            let requested = delta.saturating_abs();
            let removed = requested.min(*slot);
            *slot -= removed;
            -removed
        }
    }

    /// Returns the total stock quantity across all goods.
    pub fn total(&self) -> i64 {
        self.quantities
            .iter()
            .copied()
            .fold(0i64, |acc, qty| acc.saturating_add(qty))
    }
}

/// Per-good production and consumption rates per tick.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProductionProfile {
    production: [i64; GOODS.len()],
    consumption: [i64; GOODS.len()],
}

impl Default for ProductionProfile {
    fn default() -> Self {
        Self {
            production: [0; GOODS.len()],
            consumption: [0; GOODS.len()],
        }
    }
}

impl ProductionProfile {
    /// Creates a profile from explicit production and consumption arrays.
    pub fn new(production: [i64; GOODS.len()], consumption: [i64; GOODS.len()]) -> Self {
        Self {
            production,
            consumption,
        }
    }

    /// Returns the production rate for `good`.
    pub fn production(&self, good: Good) -> i64 {
        self.production[good_index(good)]
    }

    /// Returns the consumption rate for `good`.
    pub fn consumption(&self, good: Good) -> i64 {
        self.consumption[good_index(good)]
    }

    /// Returns `production - consumption` for `good`.
    pub fn net_flow(&self, good: Good) -> i64 {
        self.production(good) - self.consumption(good)
    }
}

/// Trade proposal between two stock-holding actors.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TradeOffer {
    /// Good transferred from `a` to `b`.
    pub good_a_to_b: Good,
    /// Quantity transferred from `a` to `b`.
    pub qty_a_to_b: i64,
    /// Good transferred from `b` to `a`.
    pub good_b_to_a: Good,
    /// Quantity transferred from `b` to `a`.
    pub qty_b_to_a: i64,
}

/// True when a consumption deficit was clamped at zero: the request was a draw
/// (`flow < 0`) but the applied delta is *less* negative than requested because
/// stock ran out (e.g. wanted -10, only -3 existed). The previous inline check
/// compared the wrong direction and so never fired.
fn deficit_was_clamped(flow: i64, applied: i64) -> bool {
    flow < 0 && applied > flow
}

/// Applies one production/consumption tick to `stocks`.
///
/// Each good is advanced independently by its net flow. Deficits are clamped
/// at zero and reported through tracing for observability.
pub fn step_stocks(stocks: &mut Stocks, profile: &ProductionProfile) {
    for good in GOODS {
        let flow = profile.net_flow(good);
        let before = stocks.get(good);
        let applied = stocks.add(good, flow);
        if deficit_was_clamped(flow, applied) {
            debug!(
                good = ?good,
                requested_deficit = flow,
                applied_delta = applied,
                before,
                after = stocks.get(good),
                "stock deficit clamped at zero"
            );
        }
    }
}

/// Returns the surplus quantity for `good` after one tick, or zero if the
/// profile is in deficit for that good.
pub fn surplus(stocks: &Stocks, profile: &ProductionProfile, good: Good) -> i64 {
    let available_after_flow = stocks.get(good).saturating_add(profile.net_flow(good));
    available_after_flow.max(0)
}

/// Returns the unmet deficit quantity for `good` after one tick, or zero if the
/// profile is in surplus for that good.
pub fn deficit(stocks: &Stocks, profile: &ProductionProfile, good: Good) -> i64 {
    let available_after_flow = stocks.get(good).saturating_add(profile.net_flow(good));
    (-available_after_flow).max(0)
}

/// Returns the good with the highest net surplus rate.
pub fn comparative_advantage(profile: &ProductionProfile) -> Good {
    let mut best = GOODS[0];
    let mut best_flow = profile.net_flow(best);
    for good in GOODS.iter().copied().skip(1) {
        let flow = profile.net_flow(good);
        if flow > best_flow {
            best = good;
            best_flow = flow;
        }
    }
    best
}

/// Estimates gains from trade when two profiles specialize in their strongest
/// comparative-advantage goods.
pub fn trade_gain(a: &ProductionProfile, b: &ProductionProfile) -> i64 {
    let a_good = comparative_advantage(a);
    let b_good = comparative_advantage(b);
    if a_good == b_good {
        return 0;
    }

    let a_gain = a.net_flow(a_good).saturating_sub(a.net_flow(b_good)).max(0);
    let b_gain = b.net_flow(b_good).saturating_sub(b.net_flow(a_good)).max(0);
    a_gain.saturating_add(b_gain).max(0)
}

/// Proposes a mutually beneficial exchange when each side has surplus in what
/// the other side lacks.
pub fn propose_trade(
    a_stocks: &Stocks,
    a_profile: &ProductionProfile,
    b_stocks: &Stocks,
    b_profile: &ProductionProfile,
) -> Option<TradeOffer> {
    let mut best: Option<TradeOffer> = None;

    for good_a_to_b in GOODS {
        if surplus(a_stocks, a_profile, good_a_to_b) <= 0 {
            continue;
        }
        if deficit(b_stocks, b_profile, good_a_to_b) <= 0 {
            continue;
        }

        for good_b_to_a in GOODS {
            if good_b_to_a == good_a_to_b {
                continue;
            }
            if surplus(b_stocks, b_profile, good_b_to_a) <= 0 {
                continue;
            }
            if deficit(a_stocks, a_profile, good_b_to_a) <= 0 {
                continue;
            }

            let qty_a_to_b = surplus(a_stocks, a_profile, good_a_to_b)
                .min(deficit(b_stocks, b_profile, good_a_to_b))
                .min(a_stocks.get(good_a_to_b));
            let qty_b_to_a = surplus(b_stocks, b_profile, good_b_to_a)
                .min(deficit(a_stocks, a_profile, good_b_to_a))
                .min(b_stocks.get(good_b_to_a));

            if qty_a_to_b <= 0 || qty_b_to_a <= 0 {
                continue;
            }

            let offer = TradeOffer {
                good_a_to_b,
                qty_a_to_b,
                good_b_to_a,
                qty_b_to_a,
            };

            if best
                .as_ref()
                .map(|current| offer_better(&offer, current))
                .unwrap_or(true)
            {
                best = Some(offer);
            }
        }
    }

    best
}

fn offer_better(candidate: &TradeOffer, current: &TradeOffer) -> bool {
    let candidate_score = candidate.qty_a_to_b.saturating_add(candidate.qty_b_to_a);
    let current_score = current.qty_a_to_b.saturating_add(current.qty_b_to_a);
    candidate_score > current_score
}

/// Applies a trade offer while conserving the combined stock total.
pub fn apply_trade(a: &mut Stocks, b: &mut Stocks, offer: &TradeOffer) {
    let removed_a = a.add(offer.good_a_to_b, -offer.qty_a_to_b);
    let added_b = b.add(offer.good_a_to_b, offer.qty_a_to_b);
    let removed_b = b.add(offer.good_b_to_a, -offer.qty_b_to_a);
    let added_a = a.add(offer.good_b_to_a, offer.qty_b_to_a);
    debug_assert_eq!(removed_a.abs(), added_b.abs());
    debug_assert_eq!(removed_b.abs(), added_a.abs());
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;

    /// Covers FR-CIV-LIFE-020.
    /// Stock stepping conserves totals and never drives any good below zero.
    #[test]
    fn deficit_was_clamped_detects_only_real_clamps() {
        // Full deficit applied (stock covered it): not clamped.
        assert!(!deficit_was_clamped(-10, -10));
        // Deficit clamped (wanted -10, only -3 of stock): clamped.
        assert!(deficit_was_clamped(-10, -3));
        // Fully clamped to nothing (wanted -10, 0 stock): clamped.
        assert!(deficit_was_clamped(-10, 0));
        // Production (positive flow): never a deficit clamp.
        assert!(!deficit_was_clamped(10, 10));
        // No flow: not a clamp.
        assert!(!deficit_was_clamped(0, 0));
    }

    #[test]
    fn step_conserves_and_clamps_to_zero() {
        let mut stocks = Stocks::default();
        stocks.add(Good::Food, 10);
        stocks.add(Good::Water, 4);
        let profile = ProductionProfile::new([3, 0, 0, 0, 0], [15, 7, 0, 0, 0]);

        step_stocks(&mut stocks, &profile);

        assert!(stocks.get(Good::Food) >= 0);
        assert!(stocks.get(Good::Water) >= 0);
        assert!(stocks.get(Good::Wood) >= 0);
        assert!(stocks.get(Good::Metal) >= 0);
        assert!(stocks.get(Good::Tools) >= 0);
        assert_eq!(stocks.total(), 0);
        assert_eq!(stocks.get(Good::Food), 0);
        assert_eq!(stocks.get(Good::Water), 0);
    }

    #[test]
    fn production_and_consumption_getters_return_profile_rates() {
        let production = [10i64, 2, 3, 4, 5];
        let consumption = [1i64, 6, 0, 0, 2];
        let profile = ProductionProfile::new(production, consumption);

        for (i, good) in GOODS.iter().enumerate() {
            assert_eq!(profile.production(*good), production[i]);
            assert_eq!(profile.consumption(*good), consumption[i]);
        }
    }

    /// FR-CIV-LIFE-021: surplus and deficit signs reflect net flow against stock.
    #[test]
    fn surplus_and_deficit_signs_are_correct() {
        let stocks = Stocks::default();
        let profile = ProductionProfile::new([12, 0, 0, 0, 0], [2, 8, 0, 0, 0]);

        assert_eq!(surplus(&stocks, &profile, Good::Food), 10);
        assert_eq!(deficit(&stocks, &profile, Good::Food), 0);
        assert_eq!(surplus(&stocks, &profile, Good::Water), 0);
        assert_eq!(deficit(&stocks, &profile, Good::Water), 8);
    }

    /// FR-CIV-LIFE-022: comparative advantage is the highest net-surplus good.
    #[test]
    fn comparative_advantage_picks_max_surplus_good() {
        let profile = ProductionProfile::new([1, 7, 2, 0, 3], [0, 2, 1, 0, 1]);
        assert_eq!(comparative_advantage(&profile), Good::Water);
    }

    /// FR-CIV-LIFE-023: trade gain is positive when advantages differ and zero when identical.
    #[test]
    fn trade_gain_reflects_specialization_difference() {
        let a = ProductionProfile::new([10, 1, 0, 0, 0], [1, 0, 0, 0, 0]);
        let b = ProductionProfile::new([0, 9, 0, 0, 0], [0, 1, 0, 0, 0]);
        let c = ProductionProfile::new([10, 1, 0, 0, 0], [1, 0, 0, 0, 0]);

        assert!(trade_gain(&a, &b) > 0);
        assert_eq!(trade_gain(&a, &c), 0);
    }

    /// FR-CIV-LIFE-024: applying a trade conserves total stock across both actors.
    #[test]
    fn apply_trade_conserves_total_stock() {
        let mut a = Stocks::default();
        let mut b = Stocks::default();
        a.add(Good::Food, 10);
        a.add(Good::Wood, 2);
        b.add(Good::Water, 8);
        b.add(Good::Metal, 5);

        let before = a.total() + b.total();
        let offer = TradeOffer {
            good_a_to_b: Good::Food,
            qty_a_to_b: 4,
            good_b_to_a: Good::Water,
            qty_b_to_a: 3,
        };
        apply_trade(&mut a, &mut b, &offer);

        assert_eq!(a.total() + b.total(), before);
        assert_eq!(a.get(Good::Food), 6);
        assert_eq!(b.get(Good::Food), 4);
        assert_eq!(b.get(Good::Water), 5);
        assert_eq!(a.get(Good::Water), 3);
    }

    /// FR-CIV-LIFE-025: trade proposals are rejected when there is no mutual benefit.
    #[test]
    fn propose_trade_returns_none_without_mutual_benefit() {
        let a_stocks = Stocks::default();
        let b_stocks = Stocks::default();
        let a_profile = ProductionProfile::new([5, 0, 0, 0, 0], [0, 0, 0, 0, 0]);
        let b_profile = ProductionProfile::new([0, 5, 0, 0, 0], [0, 0, 0, 0, 0]);

        assert_eq!(
            propose_trade(&a_stocks, &a_profile, &b_stocks, &b_profile),
            None
        );
    }

    proptest! {
        /// FR-CIV-LIFE-024 — valid trades conserve the combined stock total.
        #[test]
        fn apply_trade_conserves_total_for_valid_offers(
            a_food in 0i64..200,
            a_water in 0i64..200,
            a_wood in 0i64..200,
            a_metal in 0i64..200,
            a_tools in 0i64..200,
            b_food in 0i64..200,
            b_water in 0i64..200,
            b_wood in 0i64..200,
            b_metal in 0i64..200,
            b_tools in 0i64..200,
            qty_a_to_b in 0i64..200,
            qty_b_to_a in 0i64..200,
        ) {
            let mut a = Stocks::default();
            let mut b = Stocks::default();
            for (good, qty) in [
                (Good::Food, a_food),
                (Good::Water, a_water),
                (Good::Wood, a_wood),
                (Good::Metal, a_metal),
                (Good::Tools, a_tools),
            ] {
                a.add(good, qty);
            }
            for (good, qty) in [
                (Good::Food, b_food),
                (Good::Water, b_water),
                (Good::Wood, b_wood),
                (Good::Metal, b_metal),
                (Good::Tools, b_tools),
            ] {
                b.add(good, qty);
            }

            let good_a_to_b = GOODS[(a_food as usize + b_water as usize) % GOODS.len()];
            let mut good_b_to_a = GOODS[(a_wood as usize + b_tools as usize + 1) % GOODS.len()];
            if good_b_to_a == good_a_to_b {
                good_b_to_a = GOODS[(good_b_to_a as usize + 1) % GOODS.len()];
            }

            let offer = TradeOffer {
                good_a_to_b,
                qty_a_to_b: qty_a_to_b.min(a.get(good_a_to_b)).min(b.get(good_a_to_b).saturating_add(50)),
                good_b_to_a,
                qty_b_to_a: qty_b_to_a.min(b.get(good_b_to_a)).min(a.get(good_b_to_a).saturating_add(50)),
            };

            let before = a.total() + b.total();
            apply_trade(&mut a, &mut b, &offer);

            prop_assert_eq!(a.total() + b.total(), before);
            for good in GOODS {
                prop_assert!(a.get(good) >= 0);
                prop_assert!(b.get(good) >= 0);
            }
        }

        /// FR-CIV-LIFE-022 — comparative advantage must select a maximum net-flow good.
        #[test]
        fn comparative_advantage_is_global_maximum(
            production in prop::array::uniform5(-100i64..100),
            consumption in prop::array::uniform5(0i64..100),
        ) {
            let profile = ProductionProfile::new(production, consumption);
            let chosen = comparative_advantage(&profile);
            let chosen_flow = profile.net_flow(chosen);

            for good in GOODS {
                prop_assert!(chosen_flow >= profile.net_flow(good));
            }
        }

        /// FR-CIV-LIFE-020 — arbitrary stock updates never make inventories negative.
        #[test]
        fn add_and_step_never_produce_negative_stock(
            initial in prop::array::uniform5(0i64..500),
            production in prop::array::uniform5(-100i64..100),
            consumption in prop::array::uniform5(0i64..150),
        ) {
            let mut stocks = Stocks::default();
            for (good, qty) in GOODS.into_iter().zip(initial) {
                stocks.add(good, qty);
            }
            let profile = ProductionProfile::new(production, consumption);
            step_stocks(&mut stocks, &profile);

            for good in GOODS {
                prop_assert!(stocks.get(good) >= 0);
            }
        }
    }
}
