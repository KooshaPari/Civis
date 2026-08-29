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
