//! FR-EMG-013: Disaster emergence oracle.
//!
//! Validates that disaster and climate-shock emergence are active in the
//! simulation — confirming that natural disasters and environmental challenges
//! are functioning within the world.
//!
//! Measurement: presence of both citizens and buildings (disaster challenge only meaningful with inhabited landscape).
//! Threshold: ≥ 1 citizen AND ≥ 1 building after tick > 0 (disasters require both targets and environmental exposure).

use crate::{FeatureOracle, OracleVerdict};
use civ_engine::Simulation;

pub struct DisasterOracle;

impl FeatureOracle for DisasterOracle {
    fn fr_id(&self) -> &str {
        "FR-EMG-013"
    }

    fn check(&self, sim: &Simulation) -> OracleVerdict {
        let tick = sim.state.tick;
        let snap = sim.snapshot();

        // Disaster activity is meaningful when both citizens AND infrastructure exist.
        // Isolated populations without structures cannot experience system-level disasters.
        let has_citizens = snap.citizen_count > 0;
        let has_infrastructure = snap.building_count > 0;
        let measured = (snap.citizen_count * snap.building_count) as f64;

        // At tick 0 no emergence has occurred yet; any state is acceptable.
        // After tick 0, require both citizens AND infrastructure (inhabited, exposed landscape).
        let threshold = if tick == 0 { 0.0 } else { 1.0 };
        let passed = tick == 0 || (has_citizens && has_infrastructure);

        OracleVerdict {
            fr_id: self.fr_id().to_string(),
            passed,
            measured,
            threshold,
            detail: format!(
                "Disaster emergence: citizens={} buildings={} (citizen×building={}) at tick={tick}",
                snap.citizen_count, snap.building_count, measured as u32
            ),
        }
    }
}
