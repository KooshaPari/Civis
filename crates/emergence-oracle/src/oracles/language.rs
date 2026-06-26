//! FR-EMG-002: Language emergence oracle.
//!
//! Validates that lexical divergence is occurring across isolated population
//! clusters. Two or more clusters possessing distinct evolved lexicons confirms
//! that the phoneme-drift + co-location isolation loop is active.
//!
//! Measurement: number of clusters that have grown their lexicons to at least
//! one coined lexeme (non-empty `EvolvedLexicon`). Threshold: ≥ 2 such
//! clusters after tick > 0, demonstrating independent divergence paths.

use crate::{FeatureOracle, OracleVerdict};
use civ_engine::Simulation;

pub struct LanguageOracle;

impl FeatureOracle for LanguageOracle {
    fn fr_id(&self) -> &str {
        "FR-EMG-002"
    }

    fn check(&self, sim: &Simulation) -> OracleVerdict {
        let tick = sim.state.tick;
        let lexicons = sim.cluster_lexicons();

        // Count clusters that have at least one coined lexeme.
        let active_clusters = lexicons.values().filter(|lex| !(*lex).is_empty()).count();
        let total_clusters = lexicons.len();
        let measured = active_clusters as f64;

        // Two or more independent lexicons confirms divergence has happened.
        // At tick 0 we only require the substrate to exist (threshold 0).
        let threshold = if tick == 0 { 0.0 } else { 2.0 };
        let passed = tick == 0 || active_clusters >= 2;

        OracleVerdict {
            fr_id: self.fr_id().to_string(),
            passed,
            measured,
            threshold,
            detail: format!(
                "Language emergence: active_lexicon_clusters={active_clusters} \
                 total_clusters={total_clusters} at tick={tick}"
            ),
        }
    }
}
