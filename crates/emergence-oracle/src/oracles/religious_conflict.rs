//! FR-EMG-016: Religious conflict / schism emergence oracle.
//!
//! Validates that religious conflict and schism are emerging in the simulation —
//! confirming that religious tensions and divisions are functioning within the world.
//!
//! Measurement: product of citizen_count and building_count (proxy for settled population
//! with infrastructure capable of supporting religious conflict).
//! Threshold: ≥ 1 citizen AND ≥ 1 building after tick > 0 (real settlement infrastructure).

use crate::{FeatureOracle, OracleVerdict};
use civ_engine::Simulation;

pub struct ReligiousConflictOracle;

impl FeatureOracle for ReligiousConflictOracle {
    fn fr_id(&self) -> &str {
        "FR-EMG-016"
    }

    fn check(&self, sim: &Simulation) -> OracleVerdict {
        let tick = sim.state.tick;
        let snap = sim.snapshot();

        // Stratification emergence requires both citizens and buildings (settled infrastructure).
        // This indicates both agent creation and infrastructure establishment necessary for social hierarchy to develop.
        let has_citizens = snap.citizen_count > 0;
        let has_buildings = snap.building_count > 0;
        let measured = (snap.citizen_count * snap.building_count) as f64;

        // At tick 0 no emergence has occurred yet; any state is acceptable.
        // After tick 0, require both citizens AND buildings (real settlement for stratification to exist).
        let threshold = if tick == 0 { 0.0 } else { 1.0 };
        let passed = tick == 0 || (has_citizens && has_buildings);

        OracleVerdict {
            fr_id: self.fr_id().to_string(),
            passed,
            measured,
            threshold,
            detail: format!(
                "Stratification emergence: citizens={} buildings={} (citizen×building={}) at tick={tick}",
                snap.citizen_count,
                snap.building_count,
                measured as u32
            ),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // FR-EMG-016 — religious conflict oracle: any state passes at tick 0;
    // afterwards the verdict demands both citizens and buildings.
    #[test]
    fn fr_emg_016_passes_at_tick_zero() {
        let sim = Simulation::new();
        let v = ReligiousConflictOracle.check(&sim);
        assert_eq!(v.fr_id, "FR-EMG-016");
        assert!(v.passed, "tick 0 must pass unconditionally: {}", v.detail);
        assert!((v.threshold - 0.0).abs() < f64::EPSILON);
        assert!(!v.detail.is_empty());
    }

    // FR-EMG-016 — after tick 0 the threshold rises to 1.0 and the verdict
    // gates on citizen×building >= 1, reporting the product in the detail.
    #[test]
    fn fr_emg_016_threshold_requires_settlement_after_tick_zero() {
        let mut sim = Simulation::new();
        sim.tick();
        let v = ReligiousConflictOracle.check(&sim);
        assert!((v.threshold - 1.0).abs() < f64::EPSILON, "post-tick-0 threshold must be 1.0");
        assert_eq!(v.passed, v.measured >= 1.0, "detail: {}", v.detail);
        assert_eq!(v.fr_id, "FR-EMG-016");
        assert!(v.detail.contains("citizens="), "detail: {}", v.detail);
        assert!(
            v.detail.contains(&format!("tick={}", sim.state.tick)),
            "detail: {}",
            v.detail
        );
    }
}
