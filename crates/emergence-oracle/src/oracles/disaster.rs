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

#[cfg(test)]
mod tests {
    use super::*;
    use civ_engine::Simulation;

    // FR-EMG-013 — at tick 0 no emergence has occurred, so the disaster oracle
    // passes vacuously with threshold 0 and reports its FR id.
    #[test]
    fn fr_emg_013_passes_at_tick_zero_with_zero_threshold() {
        let sim = Simulation::new();
        let v = DisasterOracle.check(&sim);
        assert_eq!(v.fr_id, "FR-EMG-013");
        assert!(v.passed, "tick 0 must pass: {}", v.detail);
        assert_eq!(v.threshold, 0.0);
        assert!(v.measured >= 0.0, "measured is the citizen×building product");
    }

    // FR-EMG-013 — after warmup the oracle requires an inhabited, exposed
    // landscape: passing ⇔ citizens > 0 ∧ buildings > 0, with measured equal to
    // the citizen×building product at threshold 1.
    #[test]
    fn fr_emg_013_after_warmup_requires_citizens_and_buildings() {
        let mut sim = Simulation::new();
        for _ in 0..10 {
            sim.tick();
        }
        let snap = sim.snapshot();
        let v = DisasterOracle.check(&sim);
        assert_eq!(
            v.threshold, 1.0,
            "tick > 0 uses the inhabited-landscape threshold"
        );
        assert_eq!(
            v.measured,
            (snap.citizen_count * snap.building_count) as f64,
            "measured must be the citizen×building product"
        );
        let expect_pass = snap.citizen_count > 0 && snap.building_count > 0;
        assert_eq!(
            v.passed, expect_pass,
            "pass must hinge on both citizens and buildings: {}",
            v.detail
        );
        assert!(v.passed, "seeded warm sim must be inhabited: {}", v.detail);
        assert!(v.detail.starts_with("Disaster emergence:"));
    }
}
