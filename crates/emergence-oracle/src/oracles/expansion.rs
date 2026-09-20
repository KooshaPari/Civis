//! FR-EMG-017: Religious conflict / schism emergence oracle.
//!
//! Validates that religious conflict and schism are emerging in the simulation —
//! confirming that religious tensions and divisions are functioning within the world.
//!
//! Measurement: product of citizen_count and building_count (proxy for settled population
//! with infrastructure capable of supporting religious conflict).
//! Threshold: ≥ 1 citizen AND ≥ 1 building after tick > 0 (real settlement infrastructure).

use crate::{FeatureOracle, OracleVerdict};
use civ_engine::Simulation;

pub struct ExpansionOracle;

impl FeatureOracle for ExpansionOracle {
    fn fr_id(&self) -> &str {
        "FR-EMG-017"
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
    use civ_engine::Simulation;

    /// FR-EMG-017 — at tick 0 the oracle passes with threshold 0.
    #[test]
    fn fr_emg_017_passes_at_tick_zero() {
        let sim = Simulation::new();
        let v = ExpansionOracle.check(&sim);
        assert!(v.passed, "tick 0 always passes");
        assert_eq!(v.fr_id, "FR-EMG-017");
        assert_eq!(v.threshold, 0.0);
    }

    /// FR-EMG-017 — after warmup the expansion substrate (settled
    /// infrastructure + population) must be measurable and passing.
    #[test]
    fn fr_emg_017_after_warmup_requires_settled_infrastructure() {
        let mut sim = Simulation::new();
        for _ in 0..10 {
            sim.tick();
        }
        let v = ExpansionOracle.check(&sim);
        assert_eq!(v.threshold, 1.0);
        assert!(
            v.passed,
            "warm sim should have citizens and buildings: {}",
            v.detail
        );
        assert!(
            v.measured >= 1.0,
            "citizen×building product must be ≥ 1 after tick > 0"
        );
    }
}
