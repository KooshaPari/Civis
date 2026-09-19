//! Tests for FR-CIV-MARKET-003
//!
//! Epic: FR-CIV-MARKET
//! Upgraded from stub to real assertions.
//!
//! FR-CIV-MARKET-003: Price smoothing algorithm.

#[cfg(test)]
mod fr_fr_civ_market_003 {
    use civ_economy::MarketState;

    // Constants from market.rs (module is not pub, so hardcode values).
    /// Maximum absolute price change per step (100 cents).
    const MAX_PRESSURE_DELTA: i64 = 100;
    /// Minimum price allowed (1 cent).
    const MIN_PRICE: i64 = 1;
    /// Default price for new goods (1000 cents).
    const DEFAULT_PRICE: i64 = 1_000;

    /// Single pressure application stays within bounds.
    #[test]
    fn pressure_within_bounds() {
        let mut market = MarketState::default();
        // Large surplus should lower price, but not below MIN_PRICE.
        for _ in 0..100 {
            market.apply_pressure("food", 0, 1000);
        }
        let price = *market.prices().get("food").unwrap();
        assert!(
            price >= MIN_PRICE,
            "Price {} must be at least MIN_PRICE ({})",
            price,
            MIN_PRICE
        );
    }

    /// Price changes are bounded per step.
    #[test]
    fn price_change_bounded_per_step() {
        let mut market = MarketState::default();
        let initial = *market.prices().get("food").unwrap();
        // Extreme demand should still limit the change.
        market.apply_pressure("food", 10_000, 0);
        let new_price = *market.prices().get("food").unwrap();
        let delta = (new_price - initial).abs();
        assert!(
            delta <= MAX_PRESSURE_DELTA,
            "Single-step delta {} exceeds max ({})",
            delta,
            MAX_PRESSURE_DELTA
        );
    }

    /// Prices never go negative or zero.
    #[test]
    fn prices_never_negative() {
        let mut market = MarketState::default();
        for _ in 0..200 {
            market.apply_pressure("food", 0, 10_000);
        }
        for (good, price) in market.prices() {
            assert!(
                *price >= MIN_PRICE,
                "Price of {} went below minimum: {}",
                good,
                price
            );
        }
    }

    /// Default price is seeded for unknown goods.
    #[test]
    fn unknown_good_gets_default_price() {
        let mut market = MarketState::default();
        let price = market.ensure_good("exotic_mineral");
        assert_eq!(
            price, DEFAULT_PRICE,
            "New good should get default price of {}",
            DEFAULT_PRICE
        );
    }
}
