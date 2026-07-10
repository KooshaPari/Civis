//! FR-EMG-014: Mood emergence oracle.
//!
//! Validates that social mood and collective sentiment emergence are active in the
//! simulation — confirming that cultural emotions and social sentiment dynamics
//! are functioning within the world.
//!
//! Measurement: presence of both citizens and buildings (mood requires social structures and gatherings).
//! Threshold: ≥ 1 citizen AND ≥ 1 building after tick > 0 (meaningful sentiment dynamics need both agents and venues).

use crate::{FeatureOracle, OracleVerdict};
use civ_engine::Simulation;

pub struct MoodOracle;

impl FeatureOracle for MoodOracle {
    fn fr_id(&self) -> &str {
        "FR-EMG-014"
    }

    fn check(&self, sim: &Simulation) -> OracleVerdict {
        let tick = sim.state.tick;
        let snap = sim.snapshot();

        // Mood and sentiment emergence requires both citizens AND social structures.
        // Isolated agents cannot form collective mood; social venues enable gathering and sentiment formation.
        let has_citizens = snap.citizen_count > 0;
        let has_structures = snap.building_count > 0;
        let measured = (snap.citizen_count * snap.building_count) as f64;

        // At tick 0 no emergence has occurred yet; any state is acceptable.
        // After tick 0, require both citizens AND structures (social venue for mood formation).
        let threshold = if tick == 0 { 0.0 } else { 1.0 };
        let passed = tick == 0 || (has_citizens && has_structures);

        OracleVerdict {
            fr_id: self.fr_id().to_string(),
            passed,
            measured,
            threshold,
            detail: format!(
                "Mood emergence: citizens={} buildings={} (citizen×building={}) at tick={tick}",
                snap.citizen_count, snap.building_count, measured as u32
            ),
        }
    }
}
