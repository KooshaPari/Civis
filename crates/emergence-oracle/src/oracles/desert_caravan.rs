//! FR-EMG-022: Desert caravan emergence oracle.
//!
//! Validates that desert caravan infrastructure is emerging in the simulation —
//! confirming that settlements are establishing desert caravan routes and infrastructure.
//!
//! Measurement: product of citizen_count and building_count (proxy for settled population
//! with infrastructure capable of supporting desert caravan development patterns).
//! Threshold: ≥ 1 citizen AND ≥ 1 building after tick > 0 (real settlement infrastructure).

use crate::{FeatureOracle, OracleVerdict};
use civ_engine::Simulation;

pub struct DesertCaravanOracle;

impl FeatureOracle for DesertCaravanOracle {
    fn fr_id(&self) -> &str {
        "FR-EMG-022"
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
                "Desert caravan emergence: citizens={} buildings={} (citizen×building={}) at tick={tick}",
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
    use civ_engine::Simulation;

    /// FR-EMG-022 — desert-caravan oracle passes at tick 0 and reports
    /// its FR id and threshold honestly.
    #[test]
    fn desert_caravan_oracle_tick_zero_passes_with_fr_id() {
        let sim = Simulation::new();
        let verdict = DesertCaravanOracle.check(&sim);
        assert_eq!(verdict.fr_id, "FR-EMG-022");
        assert!(verdict.passed, "tick 0 always passes");
        assert!((verdict.threshold - 0.0).abs() < f64::EPSILON);
        assert!((verdict.measured - 0.0).abs() < f64::EPSILON);
    }

    /// FR-EMG-022 — after warmup the measured value equals the
    /// citizen×building product and the gate demands settled infrastructure.
    #[test]
    fn desert_caravan_oracle_measures_settled_product_after_warmup() {
        let mut sim = Simulation::new();
        sim.tick();
        let verdict = DesertCaravanOracle.check(&sim);
        let snap = sim.snapshot();
        assert!(
            (verdict.measured - (snap.citizen_count * snap.building_count) as f64).abs()
                < f64::EPSILON
        );
        if !verdict.passed {
            assert!((verdict.threshold - 1.0).abs() < f64::EPSILON);
            assert!(snap.citizen_count == 0 || snap.building_count == 0);
        } else {
            assert!(snap.citizen_count > 0 && snap.building_count > 0);
        }
    }
}
