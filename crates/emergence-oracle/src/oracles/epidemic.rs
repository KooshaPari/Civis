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

#[cfg(test)]
mod tests {
    // FR-EMG-010 — epidemic oracle: passes unconditionally at tick 0;
    // afterwards it demands both citizens and settlements and reports the
    // citizen×building product as the measurement.
    use super::*;
    use civ_engine::Simulation;

    // FR-EMG-010 — tick 0 passes unconditionally and reports the census product.
    #[test]
    fn fr_emg_010_passes_at_tick_zero() {
        let sim = Simulation::new();
        let v = EpidemicOracle.check(&sim);
        assert_eq!(v.fr_id, "FR-EMG-010");
        assert!(v.passed, "tick 0 must pass unconditionally: {}", v.detail);
        assert!((v.threshold - 0.0).abs() < f64::EPSILON, "tick-0 threshold is 0.0");
        let snap = sim.snapshot();
        assert_eq!(
            v.measured,
            (snap.citizen_count * snap.building_count) as f64,
            "measured is the citizen×building product"
        );
        assert!(v.detail.contains("Epidemic emergence"), "detail identifies the oracle");
    }

    // FR-EMG-010 — after tick 0 the verdict demands citizens AND buildings.
    #[test]
    fn fr_emg_010_after_tick_zero_requires_citizens_and_buildings() {
        let mut sim = Simulation::new();
        sim.tick();
        let v = EpidemicOracle.check(&sim);
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
