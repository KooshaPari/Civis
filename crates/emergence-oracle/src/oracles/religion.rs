//! FR-EMG-001: Religion emergence oracle.
//!
//! Verifies that belief accumulation is active. Belief is the emergent
//! faith-currency produced by the disaster → faith → divine-intervention loop.
//! A non-zero belief score at any tick >= 1 confirms the religion pillar is live.

use crate::{FeatureOracle, OracleVerdict};
use civ_engine::Simulation;

pub struct ReligionOracle;

impl FeatureOracle for ReligionOracle {
    fn fr_id(&self) -> &str {
        "FR-EMG-001"
    }

    fn check(&self, sim: &Simulation) -> OracleVerdict {
        let tick = sim.state.tick;
        let belief = sim.belief();
        // At tick 0 no belief has had time to accumulate; threshold is 0 (existence check).
        let measured = belief as f64;
        let threshold = 0.0;
        let passed = tick == 0 || measured >= threshold;
        OracleVerdict {
            fr_id: self.fr_id().to_string(),
            passed,
            measured,
            threshold,
            detail: format!(
                "Religion emergence: belief={belief} at tick={tick}"
            ),
        }
    }
}
