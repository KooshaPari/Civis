//! FR-DIPL-002 tests — Peace treaties.
//!
//! Peace SHALL be negotiated via signed treaty, producing
//! `diplomacy.peace.signed.v1` events.

use civ_diplomacy::{PeaceEvent, PeaceError, PeaceTreatyManager, PolityId};

fn p(id: u32) -> PolityId {
    PolityId::new(id)
}

/// FR-DIPL-002: Signing a peace treaty emits PeaceSigned event.
#[test]
fn peace_signed_emits_event() {
    let mut mgr = PeaceTreatyManager::new();
    let event = mgr.sign_treaty(p(1), p(2), 500, 100, false, 10).expect("sign");
    assert!(matches!(event, PeaceEvent::PeaceSigned { .. }));
    if let PeaceEvent::PeaceSigned { treaty, tick } = event {
        assert_eq!(treaty.parties, (p(1), p(2)));
        assert_eq!(treaty.ceasefire_duration, 500);
        assert_eq!(treaty.reparations_amount, 100);
        assert_eq!(tick, 10);
    }
}

/// FR-DIPL-002: Cannot sign treaty with yourself.
#[test]
fn peace_cannot_sign_with_self() {
    let mut mgr = PeaceTreatyManager::new();
    assert!(matches!(
        mgr.sign_treaty(p(1), p(1), 500, 0, false, 1),
        Err(PeaceError::SamePolity)
    ));
}

/// FR-DIPL-002: Peace treaty is detected as active.
#[test]
fn peace_has_peace_check() {
    let mut mgr = PeaceTreatyManager::new();
    assert!(!mgr.has_peace(p(1), p(2)));
    mgr.sign_treaty(p(1), p(2), 500, 0, false, 1).expect("sign");
    assert!(mgr.has_peace(p(1), p(2)));
    assert!(mgr.has_peace(p(2), p(1))); // symmetric
}

/// FR-DIPL-002: Peace treaty expires after duration.
#[test]
fn peace_treaty_expires() {
    let mut mgr = PeaceTreatyManager::new();
    mgr.sign_treaty(p(1), p(2), 100, 0, false, 10).expect("sign");
    assert!(mgr.has_peace(p(1), p(2)));

    // Before expiry
    let expired = mgr.check_expirations(50);
    assert!(expired.is_empty());

    // At expiry
    let expired = mgr.check_expirations(110); // 10 + 100 = 110
    assert_eq!(expired.len(), 1);
}

/// FR-DIPL-002: Events are drained properly.
#[test]
fn peace_events_drained() {
    let mut mgr = PeaceTreatyManager::new();
    mgr.sign_treaty(p(1), p(2), 100, 0, false, 1).expect("sign");
    let events = mgr.drain_events();
    assert_eq!(events.len(), 1);
    assert!(matches!(events[0], PeaceEvent::PeaceSigned { .. }));
    assert!(mgr.drain_events().is_empty());
}

/// FR-DIPL-002: Treaty stores territorial concessions flag.
#[test]
fn peace_territorial_concessions_recorded() {
    let mut mgr = PeaceTreatyManager::new();
    mgr.sign_treaty(p(1), p(2), 500, 200, true, 1).expect("sign");
    let treaties = mgr.treaties_for(p(1));
    assert_eq!(treaties.len(), 1);
    assert!(treaties[0].territorial_concessions);
}

/// FR-DIPL-002: Multiple treaties can exist for different pairs.
#[test]
fn peace_multiple_treaties() {
    let mut mgr = PeaceTreatyManager::new();
    mgr.sign_treaty(p(1), p(2), 100, 0, false, 1).expect("t1");
    mgr.sign_treaty(p(3), p(4), 200, 50, false, 2).expect("t2");
    assert!(mgr.has_peace(p(1), p(2)));
    assert!(!mgr.has_peace(p(1), p(3)));
    assert!(mgr.treaties_for(p(3)).len() == 1);
}
