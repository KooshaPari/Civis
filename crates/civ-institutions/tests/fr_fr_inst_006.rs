//! FR-INST-006 — Citizen lifecycle (birth, migration, death) SHALL be driven
//! by institutional and economic state.

use civ_institutions::*;

/// Lifecycle driven by institutional and economic state.
#[test]
fn lifecycle_driven_by_inst_economy() {
    // Good conditions: birth.
    let inst = InstitutionalCondition {
        governance_type: GovernanceType::Democracy,
        legitimacy_x1000: 800,
        collapsed: false,
    };
    let econ = EconomicCondition {
        joule_surplus_bp: 600,
        treasury_healthy: true,
    };
    let decision = evaluate_lifecycle(&inst, &econ);
    assert_eq!(decision.event, Some(LifecycleEvent::Birth));
    assert!(!decision.productivity_reduced);
}

/// Death when both institutions and economy collapse.
#[test]
fn death_on_dual_collapse() {
    let inst = InstitutionalCondition {
        governance_type: GovernanceType::Anarchy,
        legitimacy_x1000: 100,
        collapsed: true,
    };
    let econ = EconomicCondition {
        joule_surplus_bp: -500,
        treasury_healthy: false,
    };
    let decision = evaluate_lifecycle(&inst, &econ);
    assert_eq!(decision.event, Some(LifecycleEvent::Death));
    assert!(decision.productivity_reduced);
}

/// Migration when institutions collapse but economy is healthy.
#[test]
fn migration_on_institutional_collapse() {
    let inst = InstitutionalCondition {
        governance_type: GovernanceType::Anarchy,
        legitimacy_x1000: 100,
        collapsed: true,
    };
    let econ = EconomicCondition {
        joule_surplus_bp: 1_000,
        treasury_healthy: true,
    };
    let decision = evaluate_lifecycle(&inst, &econ);
    assert_eq!(decision.event, Some(LifecycleEvent::Migration));
}

/// Migration on severe economic deficit.
#[test]
fn migration_on_economic_deficit() {
    let inst = InstitutionalCondition {
        governance_type: GovernanceType::Democracy,
        legitimacy_x1000: 700,
        collapsed: false,
    };
    let econ = EconomicCondition {
        joule_surplus_bp: -2_000,
        treasury_healthy: true,
    };
    let decision = evaluate_lifecycle(&inst, &econ);
    assert_eq!(decision.event, Some(LifecycleEvent::Migration));
}

/// No event with neutral conditions.
#[test]
fn no_event_neutral_conditions() {
    let inst = InstitutionalCondition {
        governance_type: GovernanceType::Democracy,
        legitimacy_x1000: 500,
        collapsed: false,
    };
    let econ = EconomicCondition {
        joule_surplus_bp: 0,
        treasury_healthy: true,
    };
    let decision = evaluate_lifecycle(&inst, &econ);
    assert_eq!(decision.event, None);
    assert!(!decision.productivity_reduced);
}

/// Low legitimacy reduces productivity.
#[test]
fn low_legitimacy_reduces_productivity() {
    let inst = InstitutionalCondition {
        governance_type: GovernanceType::Autocracy,
        legitimacy_x1000: 200,
        collapsed: false,
    };
    let econ = EconomicCondition {
        joule_surplus_bp: 0,
        treasury_healthy: true,
    };
    let decision = evaluate_lifecycle(&inst, &econ);
    assert!(decision.productivity_reduced);
}

/// All three lifecycle events are represented.
#[test]
fn lifecycle_events_distinct() {
    assert_ne!(LifecycleEvent::Birth, LifecycleEvent::Migration);
    assert_ne!(LifecycleEvent::Birth, LifecycleEvent::Death);
    assert_ne!(LifecycleEvent::Migration, LifecycleEvent::Death);
}
