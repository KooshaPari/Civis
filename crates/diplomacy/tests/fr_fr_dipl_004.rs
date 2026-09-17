//! FR-DIPL-004 tests — Treaty breach detection.
//!
//! Treaty breach SHALL emit `diplomacy.treaty.broken.v1` and apply
//! reputation penalty.

use civ_diplomacy::{
    detect_breach, Treaty, TreatyBreachEvent, TreatyStatus, TreatyTerm, BREACH_REPUTATION_PENALTY,
    PolityId,
};

fn p(id: u32) -> PolityId {
    PolityId::new(id)
}

/// FR-DIPL-004: Active treaty with non-aggression term breached by war emits event.
#[test]
fn treaty_breach_emits_event_and_penalty() {
    let treaty = Treaty {
        id: 1,
        parties: (p(1), p(2)),
        treaty_type: civ_diplomacy::TreatyType::NonAggression,
        terms: vec![TreatyTerm {
            key: "non_aggression".to_string(),
            value: "true".to_string(),
        }],
        expiration_tick: None,
        status: TreatyStatus::Active,
    };

    let event = detect_breach(&treaty, true, p(1), 100).expect("breach detected");
    if let TreatyBreachEvent::TreatyBroken {
        treaty_id,
        breacher,
        victim,
        reputation_penalty,
        tick,
    } = event
    {
        assert_eq!(treaty_id, 1);
        assert_eq!(breacher, p(1));
        assert_eq!(victim, p(2));
        assert_eq!(reputation_penalty, BREACH_REPUTATION_PENALTY);
        assert_eq!(reputation_penalty, -500);
        assert_eq!(tick, 100);
    }
}

/// FR-DIPL-004: No breach if no war declared.
#[test]
fn treaty_no_breach_without_war() {
    let treaty = Treaty {
        id: 1,
        parties: (p(1), p(2)),
        treaty_type: civ_diplomacy::TreatyType::Alliance,
        terms: vec![TreatyTerm {
            key: "alliance".to_string(),
            value: "true".to_string(),
        }],
        expiration_tick: None,
        status: TreatyStatus::Active,
    };

    let event = detect_breach(&treaty, false, p(1), 100);
    assert!(event.is_none(), "no breach without war");
}

/// FR-DIPL-004: No breach for non-binding treaty.
#[test]
fn treaty_no_breach_for_trade_only() {
    let treaty = Treaty {
        id: 1,
        parties: (p(1), p(2)),
        treaty_type: civ_diplomacy::TreatyType::Trade,
        terms: vec![TreatyTerm {
            key: "grain_share".to_string(),
            value: "5".to_string(),
        }],
        expiration_tick: None,
        status: TreatyStatus::Active,
    };

    let event = detect_breach(&treaty, true, p(1), 100);
    assert!(event.is_none(), "trade-only treaty not binding");
}

/// FR-DIPL-004: Breach from either party is detected.
#[test]
fn treaty_breach_from_defender() {
    let treaty = Treaty {
        id: 1,
        parties: (p(1), p(2)),
        treaty_type: civ_diplomacy::TreatyType::DefensivePact,
        terms: vec![TreatyTerm {
            key: "mutual_defense".to_string(),
            value: "true".to_string(),
        }],
        expiration_tick: None,
        status: TreatyStatus::Active,
    };

    let event = detect_breach(&treaty, true, p(2), 50).expect("breach");
    if let TreatyBreachEvent::TreatyBroken { breacher, victim, .. } = event {
        assert_eq!(breacher, p(2));
        assert_eq!(victim, p(1));
    }
}

/// FR-DIPL-004: Breach penalty constant is -500.
#[test]
fn treaty_reputation_penalty_value() {
    assert_eq!(BREACH_REPUTATION_PENALTY, -500);
}

/// FR-DIPL-004: No breach for expired treaty.
#[test]
fn treaty_no_breach_for_expired_treaty() {
    let treaty = Treaty {
        id: 1,
        parties: (p(1), p(2)),
        treaty_type: civ_diplomacy::TreatyType::NonAggression,
        terms: vec![TreatyTerm {
            key: "non_aggression".to_string(),
            value: "true".to_string(),
        }],
        expiration_tick: Some(50),
        status: TreatyStatus::Expired,
    };

    let event = detect_breach(&treaty, true, p(1), 100);
    assert!(event.is_none(), "expired treaty cannot be breached");
}
