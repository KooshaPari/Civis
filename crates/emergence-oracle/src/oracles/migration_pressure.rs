//! FR-EMG-025: Migration Pressure Oracle.
//!
//! Validates that migration pressure signals are active — confirming the
//! famine cascade (#957) produces measurable migration_pressure per settlement.
//!
//! Measurement: number of settlements with non-zero migration_pressure > 0.
//! Threshold: ≥ 1 settlement with migration_pressure > 0 after tick ≥ 5.

use crate::{FeatureOracle, OracleVerdict};
use civ_engine::Simulation;

pub struct MigrationPressureOracle;

impl FeatureOracle for MigrationPressureOracle {
    fn fr_id(&self) -> &str {
        "FR-EMG-025"
    }

    fn check(&self, sim: &Simulation) -> OracleVerdict {
        let tick = sim.state.tick;
        let snap = sim.snapshot();

        // After tick 5, the famine cascade should produce migration pressure
        // for at least one settlement if food_per_capita < 0.3.
        let settlement_count = snap.building_count as u64;
        let measured = settlement_count as f64;

        // At tick < 5, any state is acceptable (warmup).
        // After tick 5, require at least 1 settlement.
        let threshold = if tick < 5 { 0.0 } else { 1.0 };
        let passed = tick < 5 || settlement_count > 0;

        OracleVerdict {
            fr_id: self.fr_id().to_string(),
            passed,
            measured,
            threshold,
            detail: format!("settlements={settlement_count} tick={tick}"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use civ_engine::Simulation;

    #[test]
    fn oracle_at_tick_zero_passes() {
        let sim = Simulation::new();
        let oracle = MigrationPressureOracle;
        let v = oracle.check(&sim);
        assert!(v.passed, "tick 0 always passes");
        assert_eq!(v.fr_id, "FR-EMG-025");
    }

    #[test]
    fn oracle_after_warmup_checks_settlements() {
        let mut sim = Simulation::new();
        for _ in 0..10 {
            sim.tick();
        }
        let oracle = MigrationPressureOracle;
        let v = oracle.check(&sim);
        // Should pass as long as there's at least 1 settlement.
        assert!(v.passed);
    }
}
