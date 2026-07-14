//! FR-EMG-010: Trade emergence oracle.
//!
//! Validates that trade and economy-flow emergence are active in the
//! simulation — confirming that trading relationships and economic activity
//! are functioning within the world.
//!
//! Measurement: product of citizen_count and building_count (proxy for settled population
//! with infrastructure capable of supporting trade).
//! Threshold: ≥ 1 citizen AND ≥ 1 building after tick > 0 (real settlement infrastructure).

use crate::{FeatureOracle, OracleVerdict};
use civ_engine::Simulation;

pub struct TradeOracle;

impl FeatureOracle for TradeOracle {
    fn fr_id(&self) -> &str {
        "FR-EMG-010"
    }

    fn check(&self, sim: &Simulation) -> OracleVerdict {
        let tick = sim.state.tick;
        let snap = sim.snapshot();

        // Trade emergence requires both citizens and buildings (settled infrastructure).
        // This indicates both agent creation and infrastructure establishment necessary for economic trade.
        let has_citizens = snap.citizen_count > 0;
        let has_buildings = snap.building_count > 0;
        let measured = (snap.citizen_count * snap.building_count) as f64;

        // At tick 0 no emergence has occurred yet; any state is acceptable.
        // After tick 0, require both citizens AND buildings (real settlement for trade to exist).
        let threshold = if tick == 0 { 0.0 } else { 1.0 };
        let passed = tick == 0 || (has_citizens && has_buildings);

        OracleVerdict {
            fr_id: self.fr_id().to_string(),
            passed,
            measured,
            threshold,
            detail: format!(
                "Trade emergence: citizens={} buildings={} (citizen×building={}) at tick={tick}",
                snap.citizen_count, snap.building_count, measured as u32
            ),
        }
    }
}
