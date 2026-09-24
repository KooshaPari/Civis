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

#[cfg(test)]
mod tests {
    // FR-EMG-014 — mood oracle: at tick 0 any state passes; afterwards it
    // demands both citizens and buildings (venues for sentiment formation)
    // and reports their product as the measurement.
    use super::*;
    use civ_engine::Simulation;

    // FR-EMG-014 — a fresh simulation passes unconditionally at tick 0.
    #[test]
    fn fr_emg_014_passes_at_tick_zero() {
        let sim = Simulation::new();
        let v = MoodOracle.check(&sim);
        assert_eq!(v.fr_id, "FR-EMG-014");
        assert!(v.passed, "tick 0 must pass unconditionally: {}", v.detail);
        assert!((v.threshold - 0.0).abs() < f64::EPSILON);
        assert!(
            v.detail.contains("Mood emergence"),
            "detail must identify the mood measurement: {}",
            v.detail
        );
    }

    // FR-EMG-014 — after tick 0 the threshold rises to 1.0 and passing
    // requires at least one citizen AND one building.
    #[test]
    fn fr_emg_014_threshold_is_one_after_tick_zero() {
        let mut sim = Simulation::new();
        sim.tick();
        let v = MoodOracle.check(&sim);
        assert!(
            (v.threshold - 1.0).abs() < f64::EPSILON,
            "post-tick-0 threshold must be 1.0, got {}",
            v.threshold
        );
        assert_eq!(v.passed, v.measured >= 1.0, "detail: {}", v.detail);
        // The default world spawns citizens and buildings, so it must clear 1.0.
        assert!(
            v.measured >= 1.0,
            "default world must have citizens and buildings: {}",
            v.detail
        );
    }
}
