//! FR-DIPL-003 tests — Treaty structured terms.
//!
//! Treaties SHALL encode terms (trade ratios, non-aggression, alliance)
//! as structured data.

use civ_diplomacy::{TreatyTerms, TreatyType, TreatyManager, PolityId};

fn p(id: u32) -> PolityId {
    PolityId::new(id)
}

/// FR-DIPL-003: TreatyTerms has structured fields for trade ratios.
#[test]
fn treaty_terms_structured() {
    let terms = TreatyTerms::trade(2_500, 7_500);
    assert_eq!(terms.trade_ratio_a, 2_500);
    assert_eq!(terms.trade_ratio_b, 7_500);
    assert!(!terms.non_aggression);
    assert!(!terms.alliance);
}

/// FR-DIPL-003: Non-aggression pact terms are structured.
#[test]
fn treaty_non_aggression_terms() {
    let terms = TreatyTerms::non_aggression();
    assert!(terms.non_aggression);
    assert!(!terms.alliance);
    assert_eq!(terms.trade_ratio_a, 10_000); // default 100%
}

/// FR-DIPL-003: Alliance terms include non-aggression.
#[test]
fn treaty_alliance_terms() {
    let terms = TreatyTerms::alliance();
    assert!(terms.alliance);
    assert!(terms.non_aggression);
}

/// FR-DIPL-003: Default terms are neutral (equal trade, no commitments).
#[test]
fn treaty_default_terms() {
    let terms = TreatyTerms::default();
    assert_eq!(terms.trade_ratio_a, 10_000);
    assert_eq!(terms.trade_ratio_b, 10_000);
    assert!(!terms.non_aggression);
    assert!(!terms.alliance);
    assert!(!terms.mutual_defense);
}

/// FR-DIPL-003: Trade ratios are clamped to [0, 10_000].
#[test]
fn treaty_trade_ratios_clamped() {
    let terms = TreatyTerms::trade(20_000, -500);
    assert_eq!(terms.trade_ratio_a, 10_000); // clamped to max
    assert_eq!(terms.trade_ratio_b, 0);      // clamped to min
}

/// FR-DIPL-003: TreatyManager can create treaties with TreatyType.
#[test]
fn treaty_manager_treaty_types() {
    let mut mgr = TreatyManager::new();
    let id = mgr
        .propose_treaty(p(1), (p(1), p(2)), TreatyType::Trade, vec![], None)
        .expect("propose");
    mgr.accept_treaty(p(2), id).expect("accept");
    let treaty = mgr.get_treaty(id).unwrap();
    assert_eq!(treaty.treaty_type, TreatyType::Trade);
    assert_eq!(treaty.status, civ_diplomacy::TreatyStatus::Active);
}

/// FR-DIPL-003: Alliance treaty type is available.
#[test]
fn treaty_alliance_type() {
    let mut mgr = TreatyManager::new();
    let id = mgr
        .propose_treaty(p(1), (p(1), p(2)), TreatyType::Alliance, vec![], None)
        .expect("propose");
    mgr.accept_treaty(p(1), id).expect("accept");
    let treaty = mgr.get_treaty(id).unwrap();
    assert_eq!(treaty.treaty_type, TreatyType::Alliance);
}

/// FR-DIPL-003: NonAggression treaty type is available.
#[test]
fn treaty_non_aggression_type() {
    let mut mgr = TreatyManager::new();
    let id = mgr
        .propose_treaty(p(1), (p(1), p(2)), TreatyType::NonAggression, vec![], None)
        .expect("propose");
    mgr.accept_treaty(p(1), id).expect("accept");
    let treaty = mgr.get_treaty(id).unwrap();
    assert_eq!(treaty.treaty_type, TreatyType::NonAggression);
}
