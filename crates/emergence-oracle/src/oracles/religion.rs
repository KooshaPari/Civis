//! FR-EMG-001: Religion emergence oracle.
//!
//! Validates that the disaster → faith → divine-intervention belief loop is
//! active. After the sim has ticked, at least one of these must be true:
//!
//! * `sim.belief() > 0` — raw belief currency has accumulated.
//! * `sim.state.temple_level > 0` — the belief → institution feedback
//!   upgraded at least one temple tier (confirming a full loop iteration).
//! * `sim.has_religious_patron()` — shared veneration crystallised from
//!   saga promotions (FR-CIV-RELIGION-002 patron gate).
//!
//! A threshold of 1 belief point is used for ticks ≥ 1; tick 0 always passes
//! (the loop has not had time to run).

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
        let temple_level = sim.state.temple_level;
        let has_patron = sim.has_religious_patron();

        // Composite measured value: belief + 10k per temple tier + 100k for patron.
        let measured = belief as f64
            + f64::from(temple_level) * 10_000.0
            + if has_patron { 100_000.0 } else { 0.0 };

        // At tick 0 nothing has had time to accumulate; any value passes.
        let threshold = if tick == 0 { 0.0 } else { 1.0 };
        let passed = tick == 0 || belief > 0 || temple_level > 0 || has_patron;

        OracleVerdict {
            fr_id: self.fr_id().to_string(),
            passed,
            measured,
            threshold,
            detail: format!(
                "Religion emergence: belief={belief} temple_level={temple_level} \
                 has_patron={has_patron} at tick={tick}"
            ),
        }
    }
}
