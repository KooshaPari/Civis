//! FR-EMG-008: Creature emergence oracle.
//!
//! Verifies that the military unit / creature substrate is active. Measured by
//! military_count from the simulation snapshot — military units are the ECS
//! creature-class entities present in the initial world.

use crate::{FeatureOracle, OracleVerdict};
use engine::Simulation;

pub struct CreatureOracle;

impl FeatureOracle for CreatureOracle {
    fn fr_id(&self) -> &str {
        "FR-EMG-008"
    }

    fn check(&self, sim: &Simulation) -> OracleVerdict {
        let snap = sim.snapshot();
        let measured = snap.military_count as f64;
        // Creature emergence requires at least 1 unit-class entity.
        let threshold = 1.0;
        let passed = measured >= threshold;
        OracleVerdict {
            fr_id: self.fr_id().to_string(),
            passed,
            measured,
            threshold,
            detail: format!(
                "Creature emergence: military_count={} at tick={}",
                snap.military_count, snap.tick
            ),
        }
    }
}
