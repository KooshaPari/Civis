//! FR-EMG-007: Architecture emergence oracle.
//!
//! Verifies that the building/construction substrate is active. Measured directly
//! by building count from the simulation snapshot.

use crate::{FeatureOracle, OracleVerdict};
use civ_engine::Simulation;

pub struct ArchitectureOracle;

impl FeatureOracle for ArchitectureOracle {
    fn fr_id(&self) -> &str {
        "FR-EMG-007"
    }

    fn check(&self, sim: &Simulation) -> OracleVerdict {
        let snap = sim.snapshot();
        let measured = snap.building_count as f64;
        // Architecture emergence requires at least 1 standing structure.
        let threshold = 1.0;
        let passed = measured >= threshold;
        OracleVerdict {
            fr_id: self.fr_id().to_string(),
            passed,
            measured,
            threshold,
            detail: format!(
                "Architecture emergence: building_count={} at tick={}",
                snap.building_count, snap.tick
            ),
        }
    }
}
