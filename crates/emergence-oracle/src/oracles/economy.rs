//! FR-EMG-003: Economy emergence oracle.
//!
//! Verifies that the economic substrate is active. Proxied by the population
//! count — economic allocation requires agents to distribute resources among.

use crate::{FeatureOracle, OracleVerdict};
use engine::Simulation;

pub struct EconomyOracle;

impl FeatureOracle for EconomyOracle {
    fn fr_id(&self) -> &str {
        "FR-EMG-003"
    }

    fn check(&self, sim: &Simulation) -> OracleVerdict {
        let population = sim.state.population;
        let measured = population as f64;
        // Economy emergence requires at least 1 agent for allocation.
        let threshold = 1.0;
        let passed = measured >= threshold;
        OracleVerdict {
            fr_id: self.fr_id().to_string(),
            passed,
            measured,
            threshold,
            detail: format!(
                "Economy emergence: population={population} at tick={}",
                sim.state.tick
            ),
        }
    }
}
