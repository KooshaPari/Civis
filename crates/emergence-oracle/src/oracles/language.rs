//! FR-EMG-002: Language emergence oracle.
//!
//! Verifies that the population substrate capable of driving lexicon evolution is
//! present. Proxied by citizen count: language drift requires living speakers.

use crate::{FeatureOracle, OracleVerdict};
use engine::Simulation;

pub struct LanguageOracle;

impl FeatureOracle for LanguageOracle {
    fn fr_id(&self) -> &str {
        "FR-EMG-002"
    }

    fn check(&self, sim: &Simulation) -> OracleVerdict {
        let snap = sim.snapshot();
        let measured = snap.citizen_count as f64;
        // Language emergence requires at least 1 living citizen.
        let threshold = 1.0;
        let passed = measured >= threshold;
        OracleVerdict {
            fr_id: self.fr_id().to_string(),
            passed,
            measured,
            threshold,
            detail: format!(
                "Language emergence: citizen_count={} at tick={}",
                snap.citizen_count, snap.tick
            ),
        }
    }
}
