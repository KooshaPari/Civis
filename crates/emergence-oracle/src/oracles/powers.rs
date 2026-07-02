//! FR-EMG-024: Powers registry oracle.
//!
//! Validates that the default god-tools power registry is structurally
//! consistent: every registered power must have a unique id, non-empty display
//! fields, and an internally consistent request/mask pairing.

use std::collections::HashSet;

use civ_engine::Simulation;
use civ_powers::{default_registry, PowerRequestKind, PowerTargetMask};

use crate::{FeatureOracle, OracleVerdict};

pub struct PowersOracle;

fn request_matches_target_mask(request: PowerRequestKind, applies_to: PowerTargetMask) -> bool {
    match request {
        PowerRequestKind::TerraformEdit | PowerRequestKind::MaterialEdit => {
            applies_to.contains(PowerTargetMask::VOXEL)
        }
        PowerRequestKind::ActorSpawn | PowerRequestKind::ActorEffect => {
            applies_to.contains(PowerTargetMask::AGENT)
        }
        PowerRequestKind::Disaster => {
            applies_to.contains(PowerTargetMask::SETTLEMENT)
                || applies_to.contains(PowerTargetMask::VOXEL)
        }
        PowerRequestKind::Law => applies_to.contains(PowerTargetMask::SETTLEMENT),
        PowerRequestKind::Time => applies_to.contains(PowerTargetMask::TIME),
        PowerRequestKind::NoOp => true,
    }
}

impl FeatureOracle for PowersOracle {
    fn fr_id(&self) -> &str {
        "FR-EMG-024"
    }

    fn check(&self, sim: &Simulation) -> OracleVerdict {
        let _ = sim;
        let registry_result = default_registry();
        let mut seen = HashSet::new();
        let mut unique_ids = true;
        let mut coherent_masks = true;
        let mut complete_fields = true;

        let mut measured = 0.0;
        let mut threshold = 0.0;
        let mut detail = String::from("power registry is empty");
        let mut passed = false;

        if let Ok(registry) = registry_result {
            let powers = registry.powers();
            measured = powers.len() as f64;
            threshold = powers.len() as f64;
            passed = !powers.is_empty();
            detail = format!("checked {} registered powers", powers.len());

            for power in powers {
                unique_ids &= seen.insert(power.id);
                complete_fields &= !power.label.is_empty() && !power.glyph.is_empty();
                coherent_masks &= request_matches_target_mask(power.request, power.applies_to);
            }

            passed &= unique_ids && coherent_masks && complete_fields;
            detail = format!(
                "{detail}; unique_ids={unique_ids}; coherent_masks={coherent_masks}; complete_fields={complete_fields}"
            );
        } else if let Err(err) = registry_result {
            detail = format!("failed to build default power registry: {err}");
        }

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
    fn request_target_pairing_is_consistent_for_default_power() {
        assert!(request_matches_target_mask(
            PowerRequestKind::TerraformEdit,
            PowerTargetMask::VOXEL
        ));
    }
}
