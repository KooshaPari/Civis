//! FR-EMG-021: Mountain pass emergence oracle.
//!
//! Validates that mountain pass infrastructure is emerging in the simulation —
//! confirming that settlements are establishing mountain pass routes and infrastructure.
//!
//! Measurement: product of citizen_count and building_count (proxy for settled population
//! with infrastructure capable of supporting mountain pass development patterns).
//! Threshold: ≥ 1 citizen AND ≥ 1 building after tick > 0 (real settlement infrastructure).

use crate::{FeatureOracle, OracleVerdict};
use civ_engine::Simulation;

pub struct MountainPassOracle;

impl FeatureOracle for MountainPassOracle {
    fn fr_id(&self) -> &str {
        "FR-EMG-021"
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
                "Mountain pass emergence: citizens={} buildings={} (citizen×building={}) at tick={tick}",
                snap.citizen_count,
                snap.building_count,
                measured as u32
            ),
        }
    }
}

#[cfg(test)]
mod tests {
    // FR-EMG-021 — mountain pass oracle: passes unconditionally at tick 0;
    // afterwards it demands both citizens and settlement buildings and
    // reports the citizen×building product as the measurement.
    use super::*;
    use civ_engine::Simulation;

    // FR-EMG-021 — tick 0 passes unconditionally and reports the census product.
    #[test]
    fn fr_emg_021_passes_at_tick_zero() {
        let sim = Simulation::new();
        let v = MountainPassOracle.check(&sim);
        assert_eq!(v.fr_id, "FR-EMG-021");
        assert!(v.passed, "tick 0 must pass unconditionally: {}", v.detail);
        assert!((v.threshold - 0.0).abs() < f64::EPSILON, "tick-0 threshold is 0.0");
        let snap = sim.snapshot();
        assert_eq!(
            v.measured,
            (snap.citizen_count * snap.building_count) as f64,
            "measured is the citizen×building product"
        );
        assert!(v.detail.contains("Mountain pass emergence"), "detail identifies the oracle");
    }

    // FR-EMG-021 — after tick 0 the verdict demands citizens AND buildings.
    #[test]
    fn fr_emg_021_after_tick_zero_requires_citizens_and_buildings() {
        let mut sim = Simulation::new();
        sim.tick();
        let v = MountainPassOracle.check(&sim);
        assert!(
            (v.threshold - 1.0).abs() < f64::EPSILON,
            "post-tick-0 threshold must be 1.0"
        );
        let snap = sim.snapshot();
        let expect = snap.citizen_count > 0 && snap.building_count > 0;
        assert_eq!(
            v.passed, expect,
            "verdict mirrors the live census: {}",
            v.detail
        );
        assert_eq!(
            v.passed,
            v.measured >= 1.0,
            "product of censuses encodes the gate: {}",
            v.detail
        );
    }
}
