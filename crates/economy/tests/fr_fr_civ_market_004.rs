//! Tests for FR-CIV-MARKET-004
//!
//! Epic: FR-CIV-MARKET
//! Upgraded from stub to real assertions.
//!
//! FR-CIV-MARKET-004: Tâtonnement is the default law.
//! Every priced locale runs damped tâtonnement as the baseline price-discovery.

#[cfg(test)]
mod fr_fr_civ_market_004 {
    use civ_economy::MarketState;

    /// Tâtonnement: demand above supply raises price.
    #[test]
    fn excess_demand_raises_price() {
        let mut m = MarketState::default();
        let before = m.prices().get("food").copied().unwrap();
        let after = m.apply_pressure("food", 50, 100); // demand > supply
        assert!(
            after > before,
            "Excess demand should raise price: before={}, after={}",
            before,
            after
        );
    }

    /// Tâtonnement: supply above demand lowers price.
    #[test]
    fn excess_supply_lowers_price() {
        let mut m = MarketState::default();
        let before = m.prices().get("food").copied().unwrap();
        let after = m.apply_pressure("food", 100, 50); // supply > demand
        assert!(
            after < before,
            "Excess supply should lower price: before={}, after={}",
            before,
            after
        );
    }

    /// Tâtonnement: equal supply and demand keeps price stable.
    #[test]
    fn equilibrium_maintains_price() {
        let mut m = MarketState::default();
        let before = m.prices().get("food").copied().unwrap();
        let after = m.apply_pressure("food", 100, 100); // balanced
        assert_eq!(
            after, before,
            "Equal supply/demand should maintain price"
        );
    }

    /// Tâtonnement is deterministic: same inputs -> same price.
    #[test]
    fn tatonnement_deterministic() {
        let make_market = || {
            let mut m = MarketState::default();
            m.apply_pressure("food", 80, 120);
            m
        };
        let m1 = make_market();
        let m2 = make_market();
        assert_eq!(m1, m2, "Tâtonnement must be deterministic");
    }

    /// Damped smoothing: repeated small imbalances don't cause price explosions.
    #[test]
    fn damping_prevents_explosion() {
        let mut m = MarketState::default();
        let initial = m.prices().get("food").copied().unwrap();
        for _ in 0..100 {
            m.apply_pressure("food", 90, 110);
        }
        let final_price = m.prices().get("food").copied().unwrap();
        let total_change = (final_price - initial).abs();
        // With smoothing, 100 ticks of moderate imbalance shouldn't cause
        // more than a bounded total change.
        assert!(
            total_change < 10_000,
            "Price change {} over 100 ticks is too large (explosion?)",
            total_change
        );
    }
}
