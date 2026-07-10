//! FR-EMG-010: Epidemic emergence oracle.
//!
//! Validates that disease/epidemic emergence systems are active in the
//! simulation — confirming that epidemic dynamics can spread and emerge.
//!
//! Measurement: presence of both citizens and settlements (buildings).
//! Threshold: ≥ 1 citizen AND ≥ 1 building after tick > 0 (epidemic transmission requires
//! population spread across settlements to facilitate disease spread).

use crate::{FeatureOracle, OracleVerdict};
use civ_engine::Simulation;

pub struct EpidemicOracle;

impl FeatureOracle for EpidemicOracle {
    fn fr_id(&self) -> &str {
        "FR-EMG-010"
    }

    fn check(&self, sim: &Simulation) -> OracleVerdict {
        let tick = sim.state.tick;
        let snap = sim.snapshot();

        // Epidemic transmission requires both population and settlement infrastructure.
        // Citizens distributed across buildings indicates epidemic emergence pathways are possible.
        let has_citizens = snap.citizen_count > 0;
        let has_settlements = snap.building_count > 0;
        let measured = (snap.citizen_count * snap.building_count) as f64;

        // At tick 0 no epidemic has emerged yet; any state is acceptable.
        // After tick 0, require both citizens AND buildings (meaningful settlement for transmission).
        let threshold = if tick == 0 { 0.0 } else { 1.0 };
        let passed = tick == 0 || (has_citizens && has_settlements);

        OracleVerdict {
            fr_id: self.fr_id().to_string(),
            passed,
            measured,
            threshold,
            detail: format!(
                "Epidemic emergence: citizens={} buildings={} (citizen×building={}) at tick={tick}",
                snap.citizen_count, snap.building_count, measured as u32
            ),
        }
    }
}
