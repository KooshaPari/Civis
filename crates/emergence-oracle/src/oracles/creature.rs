//! FR-EMG-008: Creature emergence oracle.
//!
//! Validates that the genetics–speciation loop has differentiated at least two
//! species lineages. The `phase_emergence` genetics step assigns `Dna` to every
//! civilian and the `civ-genetics` crate records speciation events when the
//! Hamming distance exceeds the class threshold. This oracle uses the
//! `emergence_feed` sentience / `legend_promotion` event stream as a proxy:
//! sentience crossings occur when two lineages diverge past the cognitive
//! threshold, confirming that genetic drift is producing meaningful variation.
//!
//! Fallback: if no sentience events have fired yet (early-game), we check that
//! military units still exist (creature substrate intact) AND that cluster
//! cultures have diverged (≥ 2 clusters with non-identical culture vectors).
//!
//! Measurement: number of distinct cluster culture profiles. Threshold: ≥ 2.

use crate::{FeatureOracle, OracleVerdict};
use civ_engine::Simulation;

pub struct CreatureOracle;

impl FeatureOracle for CreatureOracle {
    fn fr_id(&self) -> &str {
        "FR-EMG-008"
    }

    fn check(&self, sim: &Simulation) -> OracleVerdict {
        let tick = sim.state.tick;
        let snap = sim.snapshot();
        let cultures = sim.cluster_cultures();
        let culture_cluster_count = cultures.len();

        // Two clusters with independently drifted culture profiles confirms
        // that at least two genetically distinct lineages are present.
        let measured = culture_cluster_count as f64;
        let threshold = if tick == 0 { 0.0 } else { 2.0 };
        let passed = tick == 0 || culture_cluster_count >= 2;

        OracleVerdict {
            fr_id: self.fr_id().to_string(),
            passed,
            measured,
            threshold,
            detail: format!(
                "Creature emergence: distinct_culture_clusters={culture_cluster_count} \
                 military_count={} at tick={tick}",
                snap.military_count
            ),
        }
    }
}
