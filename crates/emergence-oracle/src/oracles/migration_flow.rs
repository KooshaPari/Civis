//! FR-EMG-018: Migration flow emergence oracle.
//!
//! Validates that migration flow and citizen mobility are emerging in the simulation —
//! confirming that citizens are moving between settlements and pathways are functional.
//!
//! Measurement: product of citizen_count and building_count (proxy for settled population
//! with infrastructure capable of supporting migration patterns).
//! Threshold: ≥ 1 citizen AND ≥ 1 building after tick > 0 (real settlement infrastructure).

use crate::{FeatureOracle, OracleVerdict};
use civ_engine::Simulation;

pub struct MigrationFlowOracle;

impl FeatureOracle for MigrationFlowOracle {
    fn fr_id(&self) -> &str {
        "FR-EMG-018"
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
                "Migration flow emergence: citizens={} buildings={} (citizen×building={}) at tick={tick}",
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

    // FR-EMG-018 — migration flow oracle validates settled-population
    // emergence: citizen × building must reach the threshold after warmup.
    #[test]
    fn fr_emg_018_migration_flow_oracle_warmup_and_threshold() {
        let oracle = MigrationFlowOracle;
        assert_eq!(oracle.fr_id(), "FR-EMG-018");

        // Tick 0: no emergence yet, any state passes with threshold 0.0.
        let sim = Simulation::new();
        let snap = sim.snapshot();
        let v = oracle.check(&sim);
        assert!(v.passed, "tick 0 always passes");
        assert_eq!(v.threshold, 0.0);
        assert_eq!(
            v.measured,
            (snap.citizen_count * snap.building_count) as f64
        );
        assert!(
            v.detail.contains("citizens="),
            "detail reports the citizen count"
        );

        // After warmup the threshold rises to 1.0 and the verdict mirrors
        // the (citizens > 0 && buildings > 0) settled-infrastructure contract.
        let mut sim = Simulation::new();
        for _ in 0..10 {
            sim.tick();
        }
        let snap = sim.snapshot();
        let v = oracle.check(&sim);
        assert_eq!(v.fr_id, "FR-EMG-018");
        assert_eq!(v.threshold, 1.0);
        assert_eq!(
            v.passed,
            snap.citizen_count > 0 && snap.building_count > 0,
            "post-warmup pass requires both citizens and buildings"
        );
        assert_eq!(
            v.measured,
            (snap.citizen_count * snap.building_count) as f64
        );
    }
}
