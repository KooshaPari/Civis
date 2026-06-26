//! FR-EMG-006: Psyche emergence oracle.
//!
//! Validates that cluster belief centroids have diverged from the default
//! genome baseline. The `phase_emergence` psyche step updates per-cluster
//! belief centroids via `emergence_accrue_cluster_beliefs`; at least one
//! cluster must have a non-zero centroid (any component ≠ 0.0) to confirm
//! that OCEAN trait states are being written and accumulated.
//!
//! Measurement: number of clusters with at least one non-zero belief component.
//! Threshold: ≥ 1 such cluster after tick > 0.

use crate::{FeatureOracle, OracleVerdict};
use civ_engine::Simulation;

pub struct PsycheOracle;

impl FeatureOracle for PsycheOracle {
    fn fr_id(&self) -> &str {
        "FR-EMG-006"
    }

    fn check(&self, sim: &Simulation) -> OracleVerdict {
        let tick = sim.state.tick;
        let beliefs = sim.cluster_beliefs();

        // A cluster whose belief centroid has at least one non-zero component
        // has had psyche states written into it.
        let active_clusters = beliefs
            .values()
            .filter(|centroid| (*centroid).iter().any(|&v| v != 0.0))
            .count();

        let measured = active_clusters as f64;
        let threshold = if tick == 0 { 0.0 } else { 1.0 };
        let passed = tick == 0 || active_clusters >= 1;

        OracleVerdict {
            fr_id: self.fr_id().to_string(),
            passed,
            measured,
            threshold,
            detail: format!(
                "Psyche emergence: clusters_with_belief_state={active_clusters} \
                 total_belief_clusters={} at tick={tick}",
                beliefs.len()
            ),
        }
    }
}
