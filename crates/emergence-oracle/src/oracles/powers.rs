//! FR-EMG-024: Powers registry oracle.
//!
//! Validates that the default god-tools power registry is structurally
//! consistent: every registered power must have a unique id and non-empty
//! display / coupling fields.

use std::collections::HashSet;

use civ_engine::Simulation;
use civ_powers::{default_powers, PowerRegistry};

use crate::{FeatureOracle, OracleVerdict};

pub struct PowersOracle;

impl FeatureOracle for PowersOracle {
    fn fr_id(&self) -> &str {
        "FR-EMG-024"
    }

    fn check(&self, sim: &Simulation) -> OracleVerdict {
        let _ = sim;
        let registry = PowerRegistry::new(default_powers());
        let powers = registry.defs();
        let mut seen = HashSet::new();
        let mut unique_ids = true;
        let mut complete_fields = true;

        for power in powers {
            unique_ids &= seen.insert(power.id.as_str());
            complete_fields &= !power.label.is_empty() && !power.coupling_note.is_empty();
        }

        let measured = powers.len() as f64;
        let threshold = powers.len() as f64;
        let passed = !powers.is_empty() && unique_ids && complete_fields;
        let detail = format!(
            "checked {} registered powers; unique_ids={unique_ids}; complete_fields={complete_fields}",
            powers.len()
        );

        OracleVerdict {
            fr_id: self.fr_id().to_string(),
            passed,
            measured,
            threshold,
            detail,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_catalog_is_non_empty() {
        let registry = PowerRegistry::new(default_powers());
        assert!(!registry.is_empty());
    }
}
