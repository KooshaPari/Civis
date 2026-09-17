//! FR traceability tests for the research crate.
//!
//! Covers: FR-CIV-RESEARCH-001-SCENARIO, FR-CIV-RESEARCH-002-SNAPSHOT, FR-CIV-RESEARCH-003-EXPORT

use civ_research::{TechCard, ValidationOutcome, SCHEMA_VERSION};

/// FR-CIV-RESEARCH-001-SCENARIO — TechCard can be constructed with all required fields.
#[test]
fn fr_civ_research_001_scenario_tech_card_construction() {
    let card = TechCard {
        id: "fire-making".into(),
        era: 1,
        inputs: vec!["flint".into(), "iron_pyrite".into()],
        energy_cost: 50,
        byproducts: vec!["ash".into()],
        dependencies: vec!["stone-tools".into()],
    };
    assert_eq!(card.id, "fire-making");
    assert_eq!(card.era, 1);
    assert_eq!(card.inputs.len(), 2);
    assert_eq!(card.energy_cost, 50);
    assert_eq!(card.byproducts.len(), 1);
    assert_eq!(card.dependencies.len(), 1);
}

/// FR-CIV-RESEARCH-002-SNAPSHOT — ValidationOutcome has both Accept and Reject variants.
#[test]
fn fr_civ_research_002_snapshot_validation_variants() {
    let accept = ValidationOutcome::Accept;
    let reject = ValidationOutcome::Reject(civ_research::RejectReason::UnknownDependency("missing-law".into()));
    assert_ne!(format!("{accept:?}"), format!("{reject:?}"));
}

/// FR-CIV-RESEARCH-003-EXPORT — TechCard can be serialised to JSON.
#[test]
fn fr_civ_research_003_export_tech_card_serde() {
    let card = TechCard {
        id: "wheel".into(),
        era: 2,
        inputs: vec!["wood".into()],
        energy_cost: 100,
        byproducts: vec![],
        dependencies: vec!["fire-making".into()],
    };
    let json = serde_json::to_string(&card).expect("serialize");
    let decoded: TechCard = serde_json::from_str(&json).expect("deserialize");
    assert_eq!(card, decoded);
}

/// FR-CIV-RESEARCH-001-SCENARIO — Empty TechCard is constructible.
#[test]
fn fr_civ_research_001_scenario_empty_card() {
    let card = TechCard {
        id: String::new(),
        era: 0,
        inputs: vec![],
        energy_cost: 0,
        byproducts: vec![],
        dependencies: vec![],
    };
    assert!(card.id.is_empty());
    assert!(card.inputs.is_empty());
}
