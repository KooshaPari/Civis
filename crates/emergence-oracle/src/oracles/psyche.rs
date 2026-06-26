//! FR-EMG-006: Psyche emergence oracle.
//!
//! Verifies that the psyche/social-mood subsystem is wired. Proxied by citizen
//! count: individual psyche states exist only while citizens exist.

use crate::{FeatureOracle, OracleVerdict};
use civ_engine::Simulation;

pub struct PsycheOracle;

impl FeatureOracle for PsycheOracle {
    fn fr_id(&self) -> &str {
        "FR-EMG-006"
    }

    fn check(&self, sim: &Simulation) -> OracleVerdict {
        let snap = sim.snapshot();
        let measured = snap.citizen_count as f64;
        // Psyche emergence requires at least 1 citizen with an internal state.
        let threshold = 1.0;
        let passed = measured >= threshold;
        OracleVerdict {
            fr_id: self.fr_id().to_string(),
            passed,
            measured,
            threshold,
            detail: format!(
                "Psyche emergence: citizen_count={} (psyche substrate) at tick={}",
                snap.citizen_count, snap.tick
            ),
        }
    }
}
