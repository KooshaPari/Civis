//! Multiplayer session tracking for the WebSocket bridge.
//!
//! Each connected WebSocket client owns a [`SharedSession`] that the bridge
//! tracks in `AppState::sessions`. The session records the stable
//! [`SharedSession::connection_id`] (a UUID v4 minted on connect), the
//! client's role (when the operator role is enforced), the kinds of tick
//! frames the client has subscribed to, and the latest tick the client has
//! acknowledged receiving.
//!
//! The session abstraction is the unit of attribution for write-through
//! JSON-RPC actions: when a client issues a `sim.god_action` (or any other
//! state-mutating RPC), the bridge passes the session's `connection_id`
//! into [`civ_engine::Simulation::record_god_action`] so the engine keeps
//! an audit log of which client triggered which god action at which tick.
//!
//! Tick broadcasts remain engine-wide: every connected session receives the
//! same `Frame3d` bundle each tick (modulo the per-session subscription
//! filter), so a write-through from client A is observable by client B on
//! the next broadcast without any session-aware routing on the read path.

use std::time::Instant;

use serde::{Deserialize, Serialize};

use crate::jsonrpc::SnapshotFields;

// The following 33 requirement tags were removed from pub const SESSION_HISTORY_CAP
// declaration. They are not implemented at this symbol, and leaving them
// here claimed coverage that no code in this repository provides.
// `SESSION_HISTORY_CAP` is a ring-buffer size for the audit log. The PvE session
// requirements below need turn tokens, hot-seat, observers, challenge HTTP routes,
// UUIDv7 ids, and autosave timers; none exist in this file, and the crate has no
// database layer. `SESSION_HISTORY_CAP` is 32, a tuning constant, not a session model.
//
// Removed, with the reason each cannot be discharged here:
// FR-SESSION-001: needs a pve session type with one human plus AI nations; no session-type field exists
// FR-SESSION-002: needs a per-AI-nation ChaCha20Rng sub-stream from the session seed; no RNG field exists
// FR-SESSION-003: needs human permanent input authority; SharedSession.role is an operator role
// FR-SESSION-004: needs a NationAction queue; that type does not exist anywhere in crates/
// FR-SESSION-005: needs rejection of non-NationAction AI submissions; depends on that absent type
// FR-SESSION-006: needs a hot_seat multi-human shared WebSocket; no session-type field exists
// FR-SESSION-007: needs turn-token enforcement with error -32001; no turn token, and -32001 has 0 hits in the crate
// FR-SESSION-008: needs a session.turn.end RPC advancing and validating rotation; the method enum has no turn methods
// FR-SESSION-009: needs turn-timeout auto-advance at expires_at_tick; no such field exists
// FR-SESSION-010: needs simultaneous-turn action collection and deterministic resolution; not implemented
// FR-SESSION-011: needs observers that receive broadcasts without injection ability; no observer flag exists
// FR-SESSION-012: needs observer RPCs rejected with a specific error code; neither exists
// FR-SESSION-013: needs omniscient observer mode with tick_stride; no mode field exists
// FR-SESSION-014: needs server-side visibility filtering; get_snapshot_for_session returns the full snapshot
// FR-SESSION-015: needs an ENDED-session replay observer with seek; no session status or seek handler exists
// FR-SESSION-016: needs POST /api/v1/challenges with challenge_id and queue position; no HTTP route exists
// FR-SESSION-017: needs fully headless challenge sessions at max tick rate; no such mode exists
// FR-SESSION-018: needs a baseline score from an AI-only session; no scoring code exists
// FR-SESSION-019: needs weighted normalized metric deltas in fixed point; no score computation exists
// FR-SESSION-020: needs GET /api/v1/challenges/{id}/replay storing .civreplay; no route or storage exists
// FR-SESSION-021: needs a session.pause RPC halting the tick loop; no such method or field exists
// FR-SESSION-022: needs a session.resume RPC restoring the loop and BLAKE3 chain; not implemented
// FR-SESSION-023: needs session.set_speed accepting 1..=100 applied at a boundary; no handler exists
// FR-SESSION-024: needs session.fast_forward suppressing broadcasts then sending a final snapshot; not implemented
// FR-SESSION-025: needs session.paused.v1 / resumed.v1 / speed_changed.v1 events; no such event types exist
// FR-SESSION-026: needs a session.save RPC writing a named slot with a BLAKE3 hash; no save RPC exists here
// FR-SESSION-027: needs a session.load RPC verifying the BLAKE3 hash before restore; no load RPC exists here
// FR-SESSION-028: needs autosave to the autosave slot every autosave_interval_ticks; no timer exists
// FR-SESSION-029: needs loading from an ENDED session to branch a new session_id; no status or branch logic
// FR-SESSION-030: needs a UUIDv7 at session.create; SharedSession::new mints a UUID v4 and no create RPC exists
// FR-SESSION-031: needs full SessionConfig validation at create; no SessionConfig type exists
// FR-SESSION-032: needs persisting session state to a sessions table; the crate has no database layer
// FR-SESSION-033: needs reloading incomplete sessions on restart; there is no persistence to reload from
/// Maximum number of recent frames a session will remember for ack tracking.
///
/// Kept small: the only consumer that walks this is the audit log + tests.
pub const SESSION_HISTORY_CAP: usize = 32;

/// Per-client session state for the multiplayer WebSocket bridge.
///
/// `SharedSession` is the identity used by:
/// 1. The bridge tick loop to attribute broadcast sends to a connection.
/// 2. The JSON-RPC handler to attach `connection_id` to mutating
///    dispatches so the engine can audit actions.
/// 3. The `sim.get_snapshot_for_session` JSON-RPC handler to return a
///    per-client view (connection_id + last_acked_tick + standard snapshot).
// The following 1 requirement tags were removed from SharedSession.
// They are not discharged by this symbol. The tag named a requirement whose
// behavior lives elsewhere, or a requirement with no implementation at all, so
// leaving the tag here asserted coverage that this declaration does not provide.
// The test file for the live id is itself a stub and should not be counted as evidence: crates/server/tests/fr_fr_civ_server_001_ws.rs asserts only `SESSION_HISTORY_CAP > 0`, which is the constant that this crate already had 33 unrelated ids unbound from (session.rs:27-68).
//
// Removed, with the reason each cannot be discharged here:
// [unbound] FR-CIV-SERVER-001: IMPLEMENTED-BY-BEHAVIOR, against the wrong id. agileplus-specs/civ-021-recovered-requirements/spec.md:221-222 is a rename table: `FR-CIV-SERVER-001` -> `FR-CIV-SERVER-001-WS`, with the note that the real WebSocket server is the hyphenated form. So the id on this declaration is a STALE-ID that the repo's own recovery spec retired; the live id is FR-CIV-SERVER-001-WS. And the tagged type is a per-connection record, not a server: SharedSession holds connection_id, connected_at, role, subscribed_frame_kinds, last_acked_tick, tick_broadcasts_received, closed (crates/server/src/session.rs:84-128). It cannot accept a connection. True implementing symbol for the WebSocket server: the bridge in crates/server/src/ws_bridge.rs, which binds a TcpListener (:588) at 127.0.0.1:3800 (:150, default also in crates/server/src/main.rs:86) and services the upgrade. One caveat on that symbol, recorded so it is not lost: the spec text this id traces to names port 9876 (docs/specs/CIV-0001-core-simulation-loop.md:373, "WebSocket connect to ws://localhost:9876/sim"), and `git grep -rn "9876" -- crates/` returns zero hits - the bridge is on 3800, which crates/protocol-3d/src/lib.rs:67 already records for the sibling FR-CIV-PROTO-002. So the requirement is implemented under the wrong port; that is a spec/implementation drift to raise separately, not a reason to keep a stale id on a session record.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SharedSession {
    /// Stable, opaque connection id (UUID v4 hex string).
    ///
    /// Minted on `ws_handler` upgrade via [`SharedSession::new`] and used as
    /// the audit key on every engine mutation the bridge attributes.
    pub connection_id: String,

    /// When the WebSocket completed its upgrade handshake.
    #[serde(skip, default = "Instant::now")]
    pub connected_at: Instant,

    /// Operator role bound to this session. `None` until the client
    /// supplies `role` either in `sim.command` params (legacy) or via the
    /// dedicated auth header accepted by the bridge.
    #[serde(default)]
    pub role: Option<String>,

    /// Frame kinds this session has subscribed to via `sim.subscribe`.
    ///
    /// Empty means "no filter — receive the full bundle" (the default for
    /// new sessions). The bridge's existing [`SubscriptionFilter`] still
    /// owns the actual filter logic; this field is the per-session mirror
    /// used for audit + the `sim.get_snapshot_for_session` response.
    #[serde(default)]
    pub subscribed_frame_kinds: Vec<String>,

    /// Tick the session last acknowledged.
    ///
    /// Initialised to `0` on connect (clients that want the full history
    /// ack higher values via `sim.subscribe` / `sim.update_subscription`).
    /// The bridge advances this whenever a tick broadcast is delivered to
    /// the client without a back-pressure drop.
    #[serde(default)]
    pub last_acked_tick: u64,

    /// Monotonic counter of tick broadcasts delivered to this session.
    /// Useful for tests + audit (reception log even when no client replies).
    #[serde(default)]
    pub tick_broadcasts_received: u64,

    /// Whether the session has been closed (graceful close or back-pressure
    /// disconnect). Once true, the session is purged on the next sweep.
    #[serde(default)]
    pub closed: bool,
}

impl SharedSession {
    /// Mint a new session for a fresh WebSocket connection.
    ///
    /// `connection_id` is supplied by the caller so test harnesses can
    /// inject deterministic ids; production code should use
    /// [`SharedSession::with_new_connection_id`].
    #[must_use]
    pub fn new(connection_id: impl Into<String>) -> Self {
        Self {
            connection_id: connection_id.into(),
            connected_at: Instant::now(),
            role: None,
            subscribed_frame_kinds: Vec::new(),
            last_acked_tick: 0,
            tick_broadcasts_received: 0,
            closed: false,
        }
    }

    /// Mint a new session whose `connection_id` is a freshly-generated
    /// UUID v4 string. Used by the production `ws_handler`.
    #[must_use]
    pub fn with_new_connection_id() -> Self {
        Self::new(uuid::Uuid::new_v4().to_string())
    }

    /// Set the operator role for this session.
    pub fn set_role(&mut self, role: Option<String>) {
        self.role = role;
    }

    /// Record that this session just received a tick broadcast for
    /// `tick`. Idempotent: the same tick value updates
    /// `tick_broadcasts_received` without regressing `last_acked_tick`.
    // FR-CIV-SERVER-002-PROTO
    pub fn record_tick_delivery(&mut self, tick: u64) {
        if tick >= self.last_acked_tick {
            self.last_acked_tick = tick;
        }
        self.tick_broadcasts_received = self.tick_broadcasts_received.saturating_add(1);
    }

    /// Replace the subscription kind filter. An empty `kinds` clears the
    /// filter (full broadcast).
    pub fn set_subscribed_frame_kinds(&mut self, kinds: Vec<String>) {
        self.subscribed_frame_kinds = kinds;
    }

    /// Mark the session as closed so the next sweep purges it.
    pub fn mark_closed(&mut self) {
        self.closed = true;
    }
}

/// Per-client snapshot view returned by the `sim.get_snapshot_for_session`
/// JSON-RPC handler (and used by the bridge as a typed response shape).
///
/// Wraps the standard [`SnapshotFields`] (the same payload the engine emits
/// for `sim.snapshot`) with session-specific context (connection_id,
/// last_acked_tick) so a multiplayer client can confirm it is reading the
/// right session's state.
// The following 1 requirement tags were removed from SessionSnapshot.
// They are not discharged by this symbol. The tag named a requirement whose
// behavior lives elsewhere, or a requirement with no implementation at all, so
// leaving the tag here asserted coverage that this declaration does not provide.
// Same stub-test caveat: crates/server/tests/fr_fr_civ_server_002_proto.rs also asserts only `SESSION_HISTORY_CAP > 0`.
// Both SERVER tags fail the same way, so both come off together: the recovery spec at civ-021 retired these exact ids, and each is parked on a per-connection data struct rather than on the transport or protocol that would discharge it.
//
// Removed, with the reason each cannot be discharged here:
// [unbound] FR-CIV-SERVER-002: IMPLEMENTED-BY-BEHAVIOR, against the wrong id and the wrong scope. agileplus-specs/civ-021-recovered-requirements/spec.md:223-224 is the same rename table: `FR-CIV-SERVER-002` -> `FR-CIV-SERVER-002-PROTO`, noting that the real protocol is the hyphenated form, so the id here is a retired STALE-ID. Even taking the un-retired reading, the requirement is a client/server message protocol, and `git grep -rn "ClientMessage\|ServerMessage" -- crates/server/src/` returns zero hits: there are no such types. What the crate actually has is a JSON-RPC surface in crates/server/src/jsonrpc.rs, and this struct is a response envelope for one method. SessionSnapshot is a four-field record - connection_id, last_acked_tick, tick_broadcasts_received, snapshot: Option<SnapshotFields> (crates/server/src/session.rs:193-203) - with two constructors that copy session fields and wrap an engine payload (:208, :224). It defines no method, no variant, and no framing; it cannot be a protocol. True implementing symbols: the JSON-RPC request/response types in crates/server/src/jsonrpc.rs, surfaced over the ws_bridge transport. A stale id on a response DTO is strictly worse than no tag: it inflates the coverage count for a protocol that is implemented under a different id, and the id it advertises has no definition left in the repo.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SessionSnapshot {
    /// Connection id of the session the snapshot is scoped to.
    pub connection_id: String,
    /// Tick the session last acknowledged.
    pub last_acked_tick: u64,
    /// Monotonic counter of tick broadcasts delivered to the session.
    pub tick_broadcasts_received: u64,
    /// Standard `sim.snapshot` payload. `None` when the engine lock could
    /// not be acquired or the simulation is mid-replay-import.
    pub snapshot: Option<SnapshotFields>,
}

impl SessionSnapshot {
    /// Build a session snapshot from an optional engine payload.
    #[must_use]
    pub fn from_session(session: &SharedSession, snapshot: Option<SnapshotFields>) -> Self {
        Self {
            connection_id: session.connection_id.clone(),
            last_acked_tick: session.last_acked_tick,
            tick_broadcasts_received: session.tick_broadcasts_received,
            snapshot,
        }
    }

    /// Build a per-session snapshot view from a live simulation.
    ///
    /// Convenience constructor used by the `sim.get_snapshot_for_session`
    /// JSON-RPC handler. The handler already holds the engine lock and
    /// the session's `last_acked_tick`, so we accept those directly
    /// instead of re-locking the session map.
    #[must_use]
    pub fn new(
        connection_id: &str,
        last_acked_tick: u64,
        subscribed_frame_kinds: Vec<String>,
        sim: &civ_engine::Simulation,
    ) -> Self {
        let speed_multiplier = 1; // Bridge-side multiplier is request-time; leave to JSON-RPC layer.
        let snapshot = crate::jsonrpc::snapshot_fields_from_sim(sim, speed_multiplier);
        let view = sim.get_snapshot_for_session(
            connection_id,
            last_acked_tick,
            &subscribed_frame_kinds,
        );
        // Embed the raw engine view into the snapshot metadata for
        // dashboards that already speak the snapshot JSON shape.
        let mut snapshot_value = serde_json::to_value(&snapshot).unwrap_or(serde_json::Value::Null);
        if let Some(obj) = snapshot_value.as_object_mut() {
            obj.insert(
                "connection_id".to_owned(),
                serde_json::Value::String(connection_id.to_owned()),
            );
            obj.insert(
                "last_acked_tick".to_owned(),
                serde_json::json!(last_acked_tick),
            );
            obj.insert(
                "subscribed_frame_kinds".to_owned(),
                serde_json::json!(subscribed_frame_kinds),
            );
            obj.insert("session_view".to_owned(), view);
        }
        Self::from_raw(connection_id, last_acked_tick, snapshot_value)
    }

    /// Construct from a pre-built JSON payload. Used by
    /// [`SessionSnapshot::new`] when the bridge wants to embed
    /// additional fields (connection_id, last_acked_tick) onto the
    /// snapshot response.
    #[must_use]
    pub fn from_raw(
        connection_id: &str,
        last_acked_tick: u64,
        snapshot: serde_json::Value,
    ) -> Self {
        Self {
            connection_id: connection_id.to_owned(),
            last_acked_tick,
            tick_broadcasts_received: 0,
            snapshot: serde_json::from_value(snapshot).ok(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_session_has_uuid_v4_connection_id_when_using_with_new() {
        let session = SharedSession::with_new_connection_id();
        assert!(
            uuid::Uuid::parse_str(&session.connection_id).is_ok(),
            "connection_id must be a UUID v4 hex string, got {:?}",
            session.connection_id
        );
    }

    #[test]
    fn record_tick_delivery_advances_last_acked_tick() {
        let mut session = SharedSession::new("test-connection");
        assert_eq!(session.last_acked_tick, 0);
        assert_eq!(session.tick_broadcasts_received, 0);

        session.record_tick_delivery(5);
        assert_eq!(session.last_acked_tick, 5);
        assert_eq!(session.tick_broadcasts_received, 1);

        // Stale ticks must not regress last_acked_tick.
        session.record_tick_delivery(3);
        assert_eq!(session.last_acked_tick, 5);
        assert_eq!(session.tick_broadcasts_received, 2);
    }

    #[test]
    fn set_subscribed_frame_kinds_replaces_filter() {
        let mut session = SharedSession::new("conn");
        assert!(session.subscribed_frame_kinds.is_empty());

        session.set_subscribed_frame_kinds(vec!["voxel_delta".to_string(), "civilian_state".to_string()]);
        assert_eq!(session.subscribed_frame_kinds.len(), 2);

        session.set_subscribed_frame_kinds(Vec::new());
        assert!(session.subscribed_frame_kinds.is_empty());
    }

    #[test]
    fn set_role_round_trips() {
        let mut session = SharedSession::new("conn");
        assert!(session.role.is_none());
        session.set_role(Some("operator".to_string()));
        assert_eq!(session.role.as_deref(), Some("operator"));
    }

    #[test]
    fn mark_closed_sets_flag() {
        let mut session = SharedSession::new("conn");
        assert!(!session.closed);
        session.mark_closed();
        assert!(session.closed);
    }

    #[test]
    fn session_snapshot_from_session_copies_context() {
        let mut session = SharedSession::new("conn-snap");
        session.record_tick_delivery(42);
        let snap = SessionSnapshot::from_session(&session, None);
        assert_eq!(snap.connection_id, "conn-snap");
        assert_eq!(snap.last_acked_tick, 42);
        assert_eq!(snap.tick_broadcasts_received, 1);
        assert!(snap.snapshot.is_none());
    }
}
