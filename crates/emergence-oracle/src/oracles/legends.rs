//! FR-EMG-004: Legends emergence oracle.
//!
//! Validates that at least one legend entity has been promoted and recorded in
//! the saga graph. Promotion requires the `legends` ingest phase to have
//! processed a real in-world event (birth, death, sentience, battle, founding)
//! — confirming that the loop is live.
//!
//! Measurement: `legends_query("status")` node_count — the number of legend
//! entities in the saga graph. Threshold: ≥ 1 node after tick > 0.

use crate::{FeatureOracle, OracleVerdict};
use civ_engine::Simulation;

pub struct LegendsOracle;

impl FeatureOracle for LegendsOracle {
    fn fr_id(&self) -> &str {
        "FR-EMG-004"
    }

    fn check(&self, sim: &Simulation) -> OracleVerdict {
        let tick = sim.state.tick;
        let result = sim.legends_query("status", None, None, None);
        let node_count = result.node_count;
        let measured = node_count as f64;

        // At tick 0 no events have been ingested; any node count (including 0) passes.
        let threshold = if tick == 0 { 0.0 } else { 1.0 };
        let passed = tick == 0 || node_count >= 1;

        OracleVerdict {
            fr_id: self.fr_id().to_string(),
            passed,
            measured,
            threshold,
            detail: format!(
                "Legends emergence: saga_graph_nodes={node_count} at tick={tick}"
            ),
        }
    }
}
