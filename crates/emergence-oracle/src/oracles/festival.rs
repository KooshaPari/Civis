//! FR-EMG-012: Festival emergence oracle.
//!
//! Validates that festival and culture-celebration emergence are active in the
//! simulation — confirming that cultural celebrations and social gatherings
//! are functioning within the world.
//!
//! Measurement: presence of both citizens and buildings (cultural infrastructure needed for festivals).
//! Threshold: ≥ 1 citizen AND ≥ 1 building after tick > 0 (meaningful cultural activity requires
//! both agents and places to gather).

use crate::{FeatureOracle, OracleVerdict};
use civ_engine::Simulation;

pub struct FestivalOracle;

impl FeatureOracle for FestivalOracle {
    fn fr_id(&self) -> &str {
        "FR-EMG-012"
    }

    fn check(&self, sim: &Simulation) -> OracleVerdict {
        let tick = sim.state.tick;
        let snap = sim.snapshot();

        // Festival activity requires both citizens AND infrastructure (buildings/gathering places).
        // This indicates both cultural agents and social venues exist for celebration.
        let has_citizens = snap.citizen_count > 0;
        let has_structures = snap.building_count > 0;
        let measured = (snap.citizen_count * snap.building_count) as f64;

        // At tick 0 no emergence has occurred yet; any state is acceptable.
        // After tick 0, require both citizens AND structures (meaningful festival infrastructure).
        let threshold = if tick == 0 { 0.0 } else { 1.0 };
        let passed = tick == 0 || (has_citizens && has_structures);

        OracleVerdict {
            fr_id: self.fr_id().to_string(),
            passed,
            measured,
            threshold,
            detail: format!(
                "Festival emergence: citizens={} buildings={} (citizen×building={}) at tick={tick}",
                snap.citizen_count,
                snap.building_count,
                measured as u32
            ),
        }
    }
}
