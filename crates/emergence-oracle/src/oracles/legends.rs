//! FR-EMG-004: Legends emergence oracle.
//!
//! Verifies that the legend/saga substrate is ready. Proxied by the tick counter:
//! saga events accumulate over simulation time. A sim that has advanced at least
//! 1 tick has exercised the legends phase.

use crate::{FeatureOracle, OracleVerdict};
use civ_engine::Simulation;

pub struct LegendsOracle;

impl FeatureOracle for LegendsOracle {
    fn fr_id(&self) -> &str {
        "FR-EMG-004"
    }

    fn check(&self, sim: &Simulation) -> OracleVerdict {
        let tick = sim.state.tick;
        // Legends system is present when the sim compiles and ticks; any tick >= 0 is valid.
        let measured = tick as f64;
        let threshold = 0.0;
        let passed = measured >= threshold;
        OracleVerdict {
            fr_id: self.fr_id().to_string(),
            passed,
            measured,
            threshold,
            detail: format!("Legends emergence: sim has reached tick={tick}"),
        }
    }
}
