//! FR-INST-001 — Each civilization SHALL have an institutional type
//! (democracy, autocracy, technocracy, etc.).

use civ_institutions::{CivilizationState, GovernanceType};

/// Governance type is assigned at initialization.
#[test]
fn governance_type_assigned_at_init() {
    let state = CivilizationState::new(1, GovernanceType::Democracy);
    assert_eq!(state.governance_type, GovernanceType::Democracy);
}

/// All governance types are representable.
#[test]
fn governance_all_types_representable() {
    let types = [
        GovernanceType::Autocracy,
        GovernanceType::Oligarchy,
        GovernanceType::Democracy,
        GovernanceType::Technocracy,
        GovernanceType::Theocracy,
        GovernanceType::Anarchy,
    ];
    assert_eq!(types.len(), GovernanceType::COUNT);
    // All distinct.
    for (i, a) in types.iter().enumerate() {
        for (j, b) in types.iter().enumerate() {
            if i != j {
                assert_ne!(a, b, "governance types must be distinct");
            }
        }
    }
}

/// Default governance type is Democracy.
#[test]
fn governance_default_is_democracy() {
    assert_eq!(GovernanceType::default(), GovernanceType::Democracy);
}

/// Governance type persists across clone.
#[test]
fn governance_type_clone_preserves() {
    let original = GovernanceType::Technocracy;
    let cloned = original;
    assert_eq!(original, cloned);
}

/// Each governance type has a unique stable index.
#[test]
fn governance_type_indices_unique() {
    let mut indices: Vec<usize> = (0..GovernanceType::COUNT)
        .map(|i| match i {
            0 => GovernanceType::Autocracy.index(),
            1 => GovernanceType::Oligarchy.index(),
            2 => GovernanceType::Democracy.index(),
            3 => GovernanceType::Technocracy.index(),
            4 => GovernanceType::Theocracy.index(),
            5 => GovernanceType::Anarchy.index(),
            _ => unreachable!(),
        })
        .collect();
    indices.sort();
    indices.dedup();
    assert_eq!(indices.len(), GovernanceType::COUNT);
}
