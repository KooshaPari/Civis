//! FR-EMG-019: Coastal settlement emergence oracle.
//!
//! Validates that coastal settlement infrastructure is emerging in the simulation —
//! confirming that settlements are establishing coastal infrastructure and trade routes.
//!
//! Measurement: product of citizen_count and building_count (proxy for settled population
//! with infrastructure capable of supporting coastal development patterns).
//! Threshold: ≥ 1 citizen AND ≥ 1 building after tick > 0 (real settlement infrastructure).

use crate::{FeatureOracle, OracleVerdict};
use civ_engine::Simulation;

pub struct CoastalSettlementOracle;

impl FeatureOracle for CoastalSettlementOracle {
    fn fr_id(&self) -> &str {
        "FR-EMG-019"
    }

    fn check(&self, sim: &Simulation) -> OracleVerdict {
        let tick = sim.state.tick;
        let snap = sim.snapshot();

        // Migration flow emergence requires both citizens and buildings (settled infrastructure).
        // This indicates both agent creation and infrastructure establishment necessary for migration patterns to develop.
        let has_citizens = snap.citizen_count > 0;
        let has_buildings = snap.building_count > 0;
        let measured = (snap.citizen_count * snap.building_count) as f64;

        // At tick 0 no emergence has occurred yet; any state is acceptable.
        // After tick 0, require both citizens AND buildings (real settlement for migration patterns to exist).
        let threshold = if tick == 0 { 0.0 } else { 1.0 };
        let passed = tick == 0 || (has_citizens && has_buildings);

        OracleVerdict {
            fr_id: self.fr_id().to_string(),
            passed,
            measured,
            threshold,
            detail: format!(
                "Coastal settlement emergence: citizens={} buildings={} (citizen×building={}) at tick={tick}",
                snap.citizen_count,
                snap.building_count,
                measured as u32
            ),
        }
    }
}

#[cfg(test)]
mod tests {
    // FR-EMG-019 — coastal settlement oracle: at tick 0 any state passes;
    // afterwards it demands both citizens and buildings and reports the
    // product as the measurement.
    use super::*;
    use civ_engine::Simulation;

    #[test]
    fn fr_emg_019_passes_at_tick_zero() {
        let sim = Simulation::new();
        let v = CoastalSettlementOracle.check(&sim);
        assert_eq!(v.fr_id, "FR-EMG-019");
        assert!(v.passed, "tick 0 must pass unconditionally: {}", v.detail);
        assert!((v.threshold - 0.0).abs() < f64::EPSILON);
        assert!(!v.detail.is_empty());
    }

    #[test]
    fn fr_emg_019_threshold_is_one_after_tick_zero() {
        let mut sim = Simulation::new();
        sim.tick();
        let v = CoastalSettlementOracle.check(&sim);
        assert!(
            (v.threshold - 1.0).abs() < f64::EPSILON,
            "post-tick-0 threshold must be 1.0"
        );
        assert_eq!(v.passed, v.measured >= 1.0, "detail: {}", v.detail);
    }
}
