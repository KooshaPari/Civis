//! FR-DIPL-001 tests — War declarations.
//!
//! Civilizations SHALL be able to declare war, producing
//! `diplomacy.war.declared.v1` events.

use civ_diplomacy::{PolityId, War, WarDeclarationManager, WarError, WarEvent};

fn p(id: u32) -> PolityId {
    PolityId::new(id)
}

/// FR-DIPL-001: Declaring war emits a WarDeclared event.
#[test]
fn war::declare_emits_event() {
    let mut mgr = WarDeclarationManager::new();
    let event = mgr.declare_war(p(1), p(2), 10).expect("declare war");
    assert!(matches!(event, WarEvent::WarDeclared { .. }));
    if let WarEvent::WarDeclared { aggressor, defender, tick } = event {
        assert_eq!(aggressor, p(1));
        assert_eq!(defender, p(2));
        assert_eq!(tick, 10);
    }
}

/// FR-DIPL-001: Cannot declare war on yourself.
#[test]
fn war::cannot_declare_on_self() {
    let mut mgr = WarDeclarationManager::new();
    assert!(matches!(
        mgr.declare_war(p(1), p(1), 1),
        Err(WarError::CannotDeclareWarOnSelf)
    ));
}

/// FR-DIPL-001: Cannot declare war twice between same polities.
#[test]
fn war::cannot_declare_twice() {
    let mut mgr = WarDeclarationManager::new();
    mgr.declare_war(p(1), p(2), 1).expect("first");
    assert!(matches!(
        mgr.declare_war(p(1), p(2), 2),
        Err(WarError::WarAlreadyActive(_))
    ));
}

/// FR-DIPL-001: War declared event appears in the pending event buffer.
#[test]
fn war::event_in_buffer() {
    let mut mgr = WarDeclarationManager::new();
    mgr.declare_war(p(1), p(2), 5).expect("declare");
    let events = mgr.drain_events();
    assert_eq!(events.len(), 1);
    assert!(matches!(events[0], WarEvent::WarDeclared { .. }));
}

/// FR-DIPL-001: Ending a war emits WarEnded event.
#[test]
fn war::end_war_emits_event() {
    let mut mgr = WarDeclarationManager::new();
    mgr.declare_war(p(1), p(2), 1).expect("declare");
    let pair = civ_diplomacy::Pair::new(p(1), p(2));
    let event = mgr.end_war(pair, true, 100).expect("end war");
    assert!(matches!(event, WarEvent::WarEnded { via_treaty: true, .. }));
}

/// FR-DIPL-001: War duration is computed correctly.
#[test]
fn war::duration_computed() {
    let mut mgr = WarDeclarationManager::new();
    mgr.declare_war(p(1), p(2), 10).expect("declare");
    let pair = civ_diplomacy::Pair::new(p(1), p(2));
    let war = mgr.get_war(p(1), p(2)).unwrap();
    assert_eq!(war.duration_ticks(50), 40);
    assert_eq!(war.total_casualties(), 0);
}

/// FR-DIPL-001: is_at_war reflects active war state.
#[test]
fn war::is_at_war_check() {
    let mut mgr = WarDeclarationManager::new();
    assert!(!mgr.is_at_war(p(1), p(2)));
    mgr.declare_war(p(1), p(2), 1).expect("declare");
    assert!(mgr.is_at_war(p(1), p(2)));
    assert!(mgr.is_at_war(p(2), p(1))); // symmetric
}

/// FR-DIPL-001: Multiple simultaneous wars with different polities.
#[test]
fn war::multiple_simultaneous_wars() {
    let mut mgr = WarDeclarationManager::new();
    mgr.declare_war(p(1), p(2), 1).expect("w1");
    mgr.declare_war(p(1), p(3), 2).expect("w2");
    assert_eq!(mgr.active_war_count(), 2);
    assert!(mgr.is_at_war(p(1), p(2)));
    assert!(mgr.is_at_war(p(1), p(3)));
    assert!(!mgr.is_at_war(p(2), p(3)));
}
