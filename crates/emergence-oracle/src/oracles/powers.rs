//! FR-EMG-011: Power registry structural-consistency oracle.
//!
//! Validates the [`civ_powers::PowerRegistry`] provided by the God-Tools power
//! system for structural invariants that the runtime API cannot enforce on its
//! own:
//!
//! * `PowerId`s must be unique across the registry (defence-in-depth: the
//!   registry's `register` already rejects duplicates, so this oracle
//!   double-checks the public surface).
//! * Every `PowerDef` must carry a non-empty `label` and a non-empty `glyph` so
//!   the UI chrome (tab strip, palette) always has something to render.
//! * `request` (a [`PowerRequestKind`]) must agree with the power's
//!   `applies_to` [`PowerTargetMask`]: a terraform edit must address voxels, an
//!   actor effect must address agents, a law must address a settlement/agent/
//!   field, a time power must address time, etc. Mismatches indicate either a
//!   broken hand-written entry or a drift between coupling code and the
//!   declarative manifest.
//!
//! Re-added in this revision: the oracle was dropped during the squash-reapply
//! merge of PR #1246 that superseded the original PowersOracle PR (#1243).
//! The baseline is being raised from 24 to 25 oracles.

use civ_engine::Simulation;
use civ_powers::{
    default_registry, PowerDef, PowerRegistry, PowerRequestKind, PowerTargetMask,
};

use crate::{FeatureOracle, OracleVerdict};

pub struct PowersOracle;

/// Per-power coherence check between `request` and `applies_to`.
///
/// Returns `true` when the mask is consistent with the request kind. Any power
/// that fails this check will be reported individually in the verdict detail
/// so a regression is easy to bisect.
fn request_mask_is_coherent(power: &PowerDef) -> bool {
    let mask = power.applies_to;
    match power.request {
        PowerRequestKind::TerraformEdit => mask.contains(PowerTargetMask::VOXEL),
        PowerRequestKind::MaterialEdit => mask.contains(PowerTargetMask::VOXEL),
        PowerRequestKind::ActorSpawn | PowerRequestKind::ActorEffect => {
            mask.contains(PowerTargetMask::AGENT)
        }
        PowerRequestKind::Disaster => {
            mask.contains(PowerTargetMask::FIELD)
                || mask.contains(PowerTargetMask::SETTLEMENT)
                || mask.contains(PowerTargetMask::AGENT)
        }
        PowerRequestKind::Law => {
            mask.contains(PowerTargetMask::SETTLEMENT)
                || mask.contains(PowerTargetMask::AGENT)
                || mask.contains(PowerTargetMask::FIELD)
        }
        PowerRequestKind::Time => mask.contains(PowerTargetMask::TIME),
        // Read-only / inspection-style powers: no mask constraint.
        PowerRequestKind::NoOp => true,
    }
}

/// Build the registry snapshot used by the oracle. Factored out so unit tests
/// can exercise the validator directly without instantiating a `Simulation`.
fn load_registry() -> Result<PowerRegistry, String> {
    default_registry().map_err(|e| format!("default_registry() failed: {e}"))
}

/// Inspect a registry and return `(consistent_powers, total_powers, problems)`.
fn inspect_registry(reg: &PowerRegistry) -> (usize, usize, Vec<String>) {
    let total = reg.len();
    let mut problems: Vec<String> = Vec::new();
    let mut consistent = 0usize;
    let mut seen_ids: Vec<&str> = Vec::with_capacity(total);

    for power in reg.powers() {
        let id_str = power.id.as_str();
        let label_ok = !power.label.is_empty();
        let glyph_ok = !power.glyph.is_empty();
        let mask_ok = request_mask_is_coherent(power);
        let unique_ok = !seen_ids.contains(&id_str);
        seen_ids.push(id_str);

        if label_ok && glyph_ok && mask_ok && unique_ok {
            consistent += 1;
        } else {
            let mut reasons = Vec::new();
            if !unique_ok {
                reasons.push("duplicate-id");
            }
            if !label_ok {
                reasons.push("empty-label");
            }
            if !glyph_ok {
                reasons.push("empty-glyph");
            }
            if !mask_ok {
                reasons.push("request-mask-mismatch");
            }
            problems.push(format!("{id_str}({})", reasons.join(",")));
        }
    }

    (consistent, total, problems)
}

impl FeatureOracle for PowersOracle {
    fn fr_id(&self) -> &str {
        "FR-EMG-011"
    }

    fn check(&self, _sim: &Simulation) -> OracleVerdict {
        // Registry construction cannot depend on simulation state — it is a
        // static manifest — so this oracle is deterministic regardless of tick.
        match load_registry() {
            Ok(reg) => {
                let (consistent, total, problems) = inspect_registry(&reg);
                let measured = consistent as f64;
                let threshold = total as f64;
                let passed = total > 0 && consistent == total;

                let detail = if passed {
                    format!(
                        "Power registry structural consistency: consistent={consistent}/{total} \
                         (unique ids, non-empty labels/glyphs, request↔mask coherent)"
                    )
                } else if total == 0 {
                    format!(
                        "Power registry structural consistency: registry is empty \
                         (consistent={consistent}/{total})"
                    )
                } else {
                    format!(
                        "Power registry structural consistency: consistent={consistent}/{total} \
                         problems=[{}]",
                        problems.join(", ")
                    )
                };

                OracleVerdict {
                    fr_id: self.fr_id().to_string(),
                    passed,
                    measured,
                    threshold,
                    detail,
                }
            }
            Err(e) => OracleVerdict {
                fr_id: self.fr_id().to_string(),
                passed: false,
                measured: 0.0,
                threshold: 1.0,
                detail: format!("Power registry structural consistency: load failed: {e}"),
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_registry_is_structurally_consistent() {
        let reg = load_registry().expect("default_registry() should construct");
        let (consistent, total, problems) = inspect_registry(&reg);
        assert_eq!(
            consistent, total,
            "default power registry must be structurally consistent; problems={:?}",
            problems
        );
        assert!(total > 0, "default power registry must ship at least one power");
        for p in problems {
            // No problem should reference the only shipped power.
            assert!(
                !p.contains("terrain.raise"),
                "terrain.raise must not appear in problems: {p}"
            );
        }
    }

    #[test]
    fn request_mask_is_coherent_for_known_kinds() {
        // Sanity-check the coherence table against the kinds we exercise today.
        let terraform = PowerDef {
            id: civ_powers::PowerId::new("test.terraform"),
            tab: civ_powers::PowerTab::Terrain,
            category: civ_powers::PowerCategory::Mutating,
            label: "Test Terraform",
            glyph: "test_terraform",
            hotkey: None,
            request: PowerRequestKind::TerraformEdit,
            applies_to: PowerTargetMask::VOXEL,
            coupling_note: "ok",
            availability: civ_powers::PowerAvailability::Near,
        };
        assert!(request_mask_is_coherent(&terraform));

        let mismatch = PowerDef {
            applies_to: PowerTargetMask::AGENT,
            ..terraform
        };
        assert!(!request_mask_is_coherent(&mismatch));
    }
}