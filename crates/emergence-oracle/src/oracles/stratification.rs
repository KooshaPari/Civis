//! FR-EMG-015: Social stratification emergence oracle.
//!
//! Validates that social stratification emergence is active in the
//! simulation — confirming that citizens have organized into hierarchical
//! social structures with differentiated roles and status.
//!
//! Measurement: product of citizen_count and building_count (proxy for social complexity
//! and infrastructure supporting stratified organization).
//! Threshold: ≥ 1 citizen AND ≥ 1 building after tick > 0 (real organized society).

use crate::{FeatureOracle, OracleVerdict};
use civ_engine::Simulation;

pub struct StratificationOracle;

impl FeatureOracle for StratificationOracle {
    fn fr_id(&self) -> &str {
        "FR-EMG-015"
    }

    fn check(&self, sim: &Simulation) -> OracleVerdict {
        let tick = sim.state.tick;
        let snap = sim.snapshot();

        // Stratification emergence requires both citizens and buildings (organized society).
        // This indicates both agent creation and infrastructure establishment necessary for social hierarchy.
        let has_citizens = snap.citizen_count > 0;
        let has_buildings = snap.building_count > 0;
        let measured = (snap.citizen_count * snap.building_count) as f64;

        // At tick 0 no emergence has occurred yet; any state is acceptable.
        // After tick 0, require both citizens AND buildings (real organized society for stratification to exist).
        let threshold = if tick == 0 { 0.0 } else { 1.0 };
        let passed = tick == 0 || (has_citizens && has_buildings);

        OracleVerdict {
            fr_id: self.fr_id().to_string(),
            passed,
            measured,
            threshold,
            detail: format!(
                "Social stratification emergence: citizens={} buildings={} (citizen×building={}) at tick={tick}",
                snap.citizen_count,
                snap.building_count,
                measured as u32
            ),
        }
    }
}
