//! FR-AI-006 — AI decision events SHALL be emitted for post-run analysis
//! and replay.

#[test]
fn decision_emitted() {
    use civ_ai::{DecisionEvent, DecisionLog};

    let mut log = DecisionLog::default();

    log.emit(DecisionEvent {
        tick: 42,
        leader_id: "leader_A".into(),
        chosen_action: "expand_territory".into(),
        score: 0.87,
        candidates_count: 5,
    });

    log.emit(DecisionEvent {
        tick: 42,
        leader_id: "leader_B".into(),
        chosen_action: "form_alliance".into(),
        score: 0.92,
        candidates_count: 3,
    });

    assert_eq!(log.len(), 2);
    assert!(!log.is_empty());

    // Verify event fields are preserved.
    assert_eq!(log.events[0].tick, 42);
    assert_eq!(log.events[0].leader_id, "leader_A");
    assert_eq!(log.events[0].chosen_action, "expand_territory");
    assert!((log.events[0].score - 0.87).abs() < f64::EPSILON);
    assert_eq!(log.events[0].candidates_count, 5);
}

#[test]
fn events_filterable_by_leader() {
    use civ_ai::{DecisionEvent, DecisionLog};

    let mut log = DecisionLog::default();
    log.emit(DecisionEvent { tick: 1, leader_id: "A".into(), chosen_action: "x".into(), score: 1.0, candidates_count: 1 });
    log.emit(DecisionEvent { tick: 2, leader_id: "B".into(), chosen_action: "y".into(), score: 0.5, candidates_count: 2 });
    log.emit(DecisionEvent { tick: 3, leader_id: "A".into(), chosen_action: "z".into(), score: 0.7, candidates_count: 3 });

    let a_events = log.for_leader("A");
    assert_eq!(a_events.len(), 2);
    assert_eq!(a_events[0].chosen_action, "x");
    assert_eq!(a_events[1].chosen_action, "z");

    let c_events = log.for_leader("C");
    assert!(c_events.is_empty());
}

#[test]
fn events_serializable() {
    use civ_ai::DecisionEvent;

    let event = DecisionEvent {
        tick: 100,
        leader_id: "test".into(),
        chosen_action: "build".into(),
        score: 0.5,
        candidates_count: 4,
    };
    let json = serde_json::to_string(&event).unwrap();
    let deserialized: DecisionEvent = serde_json::from_str(&json).unwrap();
    assert_eq!(event, deserialized);
}
