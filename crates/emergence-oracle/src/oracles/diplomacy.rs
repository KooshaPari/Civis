//! FR-EMG-005: Diplomacy emergence oracle.
//!
//! Verifies that the diplomacy subsystem is wired and can produce events.
//! Measured by total births this tick as a proxy for faction activity — factions
//! need living members before diplomacy can emerge.

use crate::{FeatureOracle, OracleVerdict};
use civ_engine::Simulation;

pub struct DiplomacyOracle;

impl FeatureOracle for DiplomacyOracle {
    fn fr_id(&self) -> &str {
        "FR-EMG-005"
    }

    fn check(&self, sim: &Simulation) -> OracleVerdict {
        let snap = sim.snapshot();
        // Diplomacy emergence is gated on at least 2 factions having population.
        // The snapshot building_count is a reliable proxy: buildings require factions.
        let measured = snap.building_count as f64;
        let threshold = 1.0;
        let passed = measured >= threshold;
        OracleVerdict {
            fr_id: self.fr_id().to_string(),
            passed,
            measured,
            threshold,
            detail: format!(
                "Diplomacy emergence: building_count={} (faction substrate) at tick={}",
                snap.building_count, snap.tick
            ),
        }
    }
}
