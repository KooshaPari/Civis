//! FR traceability tests for the server crate.
//!
//! Covers: FR-CIV-SERVER-001, FR-CIV-SERVER-001-WS, FR-CIV-SERVER-002,
//! FR-CIV-SERVER-002-PROTO, FR-SESSION-014

use civ_server::{SessionSnapshot, SharedSession, SESSION_HISTORY_CAP};

/// FR-CIV-SERVER-001 — Session history cap is defined and positive.
#[test]
fn fr_civ_server_001_session_history_cap() {
    assert!(SESSION_HISTORY_CAP > 0, "SESSION_HISTORY_CAP must be positive");
    assert!(
        SESSION_HISTORY_CAP >= 10,
        "SESSION_HISTORY_CAP should be at least 10"
    );
}

/// FR-CIV-SERVER-001-WS — SharedSession can be constructed with connection id.
#[test]
fn fr_civ_server_001_ws_shared_session_construction() {
    let session = SharedSession::new("test-conn-001");
    assert_eq!(session.connection_id, "test-conn-001");
    assert_eq!(session.last_acked_tick, 0);
    assert!(!session.closed);
}

/// FR-CIV-SERVER-002 — SessionSnapshot can be built from a session.
#[test]
fn fr_civ_server_002_session_snapshot_from_session() {
    let session = SharedSession::new("snap-test");
    let snap = SessionSnapshot::from_session(&session, None);
    assert_eq!(snap.connection_id, "snap-test");
    assert_eq!(snap.last_acked_tick, 0);
    assert_eq!(snap.tick_broadcasts_received, 0);
    assert!(snap.snapshot.is_none());
}

/// FR-CIV-SERVER-002-PROTO — Session tracks tick delivery.
#[test]
fn fr_civ_server_002_proto_tick_tracking() {
    let mut session = SharedSession::new("tick-test");
    session.record_tick_delivery(10);
    assert_eq!(session.last_acked_tick, 10);
    assert_eq!(session.tick_broadcasts_received, 1);

    session.record_tick_delivery(20);
    assert_eq!(session.last_acked_tick, 20);
    assert_eq!(session.tick_broadcasts_received, 2);
}

/// FR-SESSION-014 — Session can be marked closed.
#[test]
fn fr_session_014_session_mark_closed() {
    let mut session = SharedSession::new("close-test");
    assert!(!session.closed);
    session.mark_closed();
    assert!(session.closed);
}
