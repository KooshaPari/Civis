//! FR-EMG-001: Religion emergence oracle.
//!
//! Validates that the disaster → faith belief loop is active. After the sim
//! has ticked, at least one of these must be true:
//!
//! * `sim.belief() > 0` — raw belief currency has accumulated.
//! * `sim.has_religious_patron()` — shared veneration crystallised from
//!   saga promotions (FR-CIV-RELIGION-002 patron gate).

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
        let has_patron = sim.has_religious_patron();

        let measured = belief as f64 + if has_patron { 100_000.0 } else { 0.0 };
        let threshold = if tick == 0 { 0.0 } else { 1.0 };
        let passed = tick == 0 || belief > 0 || has_patron;

        OracleVerdict {
            fr_id: self.fr_id().to_string(),
            passed,
            measured,
            threshold,
            detail: format!(
                "Religion emergence: belief={belief} has_patron={has_patron} at tick={tick}"
            ),
        }
    }
}
