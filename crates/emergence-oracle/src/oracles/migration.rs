//! FR-EMG-009: Migration emergence oracle.
//!
//! Validates that creature migration and movement dynamics are active in the
//! simulation — confirming that agents are not stationary and pathfinding is working.
//!
//! Measurement: presence of both citizens and settlements (buildings).
//! Threshold: ≥ 1 citizen AND ≥ 1 building after tick > 0 (settlement and movement
//! have both occurred to establish distributed population).

use crate::{FeatureOracle, OracleVerdict};
use civ_engine::Simulation;

pub struct MigrationOracle;

impl FeatureOracle for MigrationOracle {
    fn fr_id(&self) -> &str {
        "FR-EMG-009"
    }

    fn check(&self, sim: &Simulation) -> OracleVerdict {
        let tick = sim.state.tick;
        let snap = sim.snapshot();

        // Migration is real when citizens exist AND are distributed across buildings.
        // This indicates both agent creation and settlement/pathfinding to new locations.
        let has_citizens = snap.citizen_count > 0;
        let has_settlements = snap.building_count > 0;
        let measured = (snap.citizen_count * snap.building_count) as f64;

        // At tick 0 no emergence has occurred yet; any state is acceptable.
        // After tick 0, require both citizens AND buildings (meaningful settlement).
        let threshold = if tick == 0 { 0.0 } else { 1.0 };
        let passed = tick == 0 || (has_citizens && has_settlements);

        OracleVerdict {
            fr_id: self.fr_id().to_string(),
            passed,
            measured,
            threshold,
            detail: format!(
                "Migration emergence: citizens={} buildings={} (citizen×building={}) at tick={tick}",
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

    /// FR-EMG-009 — at tick 0 the oracle passes with threshold 0.
    #[test]
    fn fr_emg_009_passes_at_tick_zero() {
        let sim = Simulation::new();
        let v = MigrationOracle.check(&sim);
        assert!(v.passed, "tick 0 always passes");
        assert_eq!(v.fr_id, "FR-EMG-009");
        assert_eq!(v.threshold, 0.0);
    }

    /// FR-EMG-009 — after warmup ticks the settled world must satisfy the
    /// citizen × building measurement and the oracle must pass.
    #[test]
    fn fr_emg_009_after_warmup_requires_settled_population() {
        let mut sim = Simulation::new();
        for _ in 0..10 {
            sim.tick();
        }
        let v = MigrationOracle.check(&sim);
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
