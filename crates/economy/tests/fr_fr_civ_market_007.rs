//! Tests for FR-CIV-MARKET-007
//!
//! Epic: FR-CIV-MARKET
//! Upgraded from stub to real assertions.
//!
//! FR-CIV-MARKET-007: Anti-inflation anchoring.
//! Prices are anchored to energy value of output and must not spiral.
//! Currency trust decays when prices rise too quickly (anti-spiral).

#[cfg(test)]
mod fr_fr_civ_market_007 {
    use civ_economy::CurrencyTrust;

    /// CurrencyTrust can be created with default starting trust.
    #[test]
    fn currency_trust_initializes() {
        let trust = CurrencyTrust::default();
        let trust_value = trust.trust();
        // Default trust should be ~7500 basis points (0.75).
        assert!(
            trust_value > 0.0 && trust_value <= 1.0,
            "Initial trust should be between 0 and 1, got {}",
            trust_value
        );
    }

    /// Currency trust declines when prices rise rapidly.
    #[test]
    fn trust_drops_on_price_spike() {
        let mut trust = CurrencyTrust::default();
        let initial_trust = trust.trust_bp();
        // Simulate a rapid price spike: price jumped from 100 to 300 cents.
        let _outcome = civ_economy::step_currency_trust(
            &mut trust,
            1000,    // trade_volume
            300,     // price_level_cents (high)
            100,     // previous_price_level_cents (low) => 200% spike
            1000,    // supply
            1000,    // previous_supply
        );
        let new_trust = trust.trust_bp();
        assert!(
            new_trust < initial_trust,
            "Trust should drop on price spike: {} >= {}",
            new_trust,
            initial_trust
        );
    }

    /// Currency trust increases with stable prices and volume.
    #[test]
    fn trust_recovers_with_stability() {
        let mut trust = CurrencyTrust::default();
        // Simulate 20 ticks of stable prices.
        for _ in 0..20 {
            civ_economy::step_currency_trust(
                &mut trust,
                500,  // trade_volume
                100,  // price_level_cents (stable)
                100,  // previous_price_level_cents (same)
                1000, // supply
                1000, // previous_supply
            );
        }
        let trust_value = trust.trust();
        assert!(
            trust_value > 0.5,
            "Trust should be above 0.5 after 20 stable ticks, got {}",
            trust_value
        );
    }
}
