//! Institution strength helpers for governance (FR-LAW / FR-CIV-GOV).
//!
//! Wraps [`civ_institutions`] records owned by the simulation so the law
//! phase can measure how strongly a faction can codify customary norms.

use civ_institutions::Institution;
use std::collections::BTreeMap;

/// Sum institution levels across all settlements owned by `faction_id`.
///
/// `settlement_factions` maps settlement id → owning faction.
/// `institutions` maps settlement id → active institution record.
#[must_use]
pub fn faction_institution_strength(
    faction_id: u32,
    settlement_factions: &BTreeMap<u32, u32>,
    institutions: &BTreeMap<u32, Institution>,
) -> u32 {
    settlement_factions
        .iter()
        .filter(|(_, owner)| **owner == faction_id)
        .filter_map(|(sid, _)| institutions.get(sid))
        .map(|inst| u32::from(inst.level))
        .sum()
}

#[cfg(test)]
mod tests {
    use super::*;
    use civ_institutions::{Institution, InstitutionKind};

    #[test]
    fn sums_levels_for_faction_settlements_only() {
        let mut owners = BTreeMap::new();
        owners.insert(0, 1);
        owners.insert(1, 1);
        owners.insert(2, 2);

        let mut institutions = BTreeMap::new();
        institutions.insert(
            0,
            Institution {
                kind: InstitutionKind::Temple,
                level: 2,
            },
        );
        institutions.insert(
            1,
            Institution {
                kind: InstitutionKind::Garrison,
                level: 1,
            },
        );
        institutions.insert(
            2,
            Institution {
                kind: InstitutionKind::Temple,
                level: 2,
            },
        );

        assert_eq!(
            faction_institution_strength(1, &owners, &institutions),
            3
        );
    }
}
