//! FR-EMG-003: Economy emergence oracle.
//!
//! Validates that the market is clearing at non-trivial prices — confirming
//! that real supply/demand dynamics are at work, not a dormant allocator.
//!
//! Measurement: number of distinct goods with a positive clearing price in
//! `SimulationSnapshot::market_prices`. Threshold: ≥ 1 good with price > 0
//! after tick > 0 (the economy phase has had at least one opportunity to run).

use crate::{FeatureOracle, OracleVerdict};
use civ_engine::Simulation;

pub struct EconomyOracle;

impl FeatureOracle for EconomyOracle {
    fn fr_id(&self) -> &str {
        "FR-EMG-003"
    }

    fn check(&self, sim: &Simulation) -> OracleVerdict {
        let tick = sim.state.tick;
        let snap = sim.snapshot();

        // Count goods whose clearing price is strictly positive.
        let active_prices = snap.market_prices.values().filter(|&&p| p > 0).count();
        let measured = active_prices as f64;

        // At tick 0 the economy phase has not run; any state is acceptable.
        let threshold = if tick == 0 { 0.0 } else { 1.0 };
        let passed = tick == 0 || active_prices >= 1;

        OracleVerdict {
            fr_id: self.fr_id().to_string(),
            passed,
            measured,
            threshold,
            detail: format!(
                "Economy emergence: goods_with_positive_price={active_prices} \
                 total_tracked_goods={} population={} at tick={tick}",
                snap.market_prices.len(),
                snap.population
            ),
        }
    }
}
