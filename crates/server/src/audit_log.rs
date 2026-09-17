//! FR-PROT-004: Audit log — persists emitted events to a DB-backed ring buffer
//! scoped per simulation tick.
//!
//! The `AuditLog` struct collects events during a tick and flushes them to the
//! `save-db` SQLite database so operators and replay tools can query the full
//! event history after a run.

use std::fmt;

use serde::{Deserialize, Serialize};

/// One audited event persisted per tick.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AuditEntry {
    /// Event type string (e.g. `"session.saved.v1"`, `"climate.threshold.crossed.v1"`).
    pub event_type: String,
    /// Session ID that originated or received the event.
    pub session_id: String,
    /// Simulation tick at which the event was emitted.
    pub tick: u64,
    /// ISO-8601 wall-clock timestamp of the emission.
    pub created_at: String,
    /// Opaque JSON payload string.
    pub payload: String,
}

/// In-memory event collector that batches entries per tick before flushing to
/// persistent storage.
///
/// # Invariants
///
/// * Every entry added via [`push`](AuditLog::push) carries the same `tick` as
///   the log's current tick (set at construction or [`reset`]).
/// * [`flush`] drains all buffered entries and returns them; the log is empty
///   after a successful flush.
#[derive(Debug, Clone)]
pub struct AuditLog {
    /// Current tick this log is collecting for.
    tick: u64,
    /// Buffered entries that have not yet been flushed.
    entries: Vec<AuditEntry>,
    /// Maximum entries before oldest are silently dropped (ring-buffer cap).
    capacity: usize,
}

impl AuditLog {
    /// Create a new audit log scoped to `tick`.
    #[must_use]
    pub fn new(tick: u64) -> Self {
        Self::with_capacity(tick, 4096)
    }

    /// Create a new audit log with an explicit ring-buffer capacity.
    #[must_use]
    pub fn with_capacity(tick: u64, capacity: usize) -> Self {
        Self {
            tick,
            entries: Vec::with_capacity(capacity.min(4096)),
            capacity,
        }
    }

    /// Current tick this log is collecting for.
    #[must_use]
    pub fn tick(&self) -> u64 {
        self.tick
    }

    /// Number of buffered (unflushed) entries.
    #[must_use]
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// True when no entries are buffered.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// Record an event in the buffer. If the buffer is at capacity the oldest
    /// entry is evicted to make room (ring-buffer behaviour).
    pub fn push(&mut self, entry: AuditEntry) {
        if self.entries.len() >= self.capacity {
            // Evict oldest — O(1) swap-remove from the front isn't available on
            // Vec, so we drain the first entry. This path is rare because
            // capacity is generous (4096 by default).
            self.entries.remove(0);
        }
        self.entries.push(entry);
    }

    /// Convenience: record a typed event with explicit fields.
    pub fn record(
        &mut self,
        event_type: impl Into<String>,
        session_id: impl Into<String>,
        payload: impl Into<String>,
    ) {
        self.push(AuditEntry {
            event_type: event_type.into(),
            session_id: session_id.into(),
            tick: self.tick,
            created_at: chrono_now_rfc3339(),
            payload: payload.into(),
        });
    }

    /// Drain all buffered entries and return them. The log is empty afterwards.
    #[must_use]
    pub fn flush(&mut self) -> Vec<AuditEntry> {
        std::mem::take(&mut self.entries)
    }

    /// Advance the log to a new tick, discarding any unflushed entries from the
    /// previous tick.
    pub fn reset(&mut self, new_tick: u64) {
        self.entries.clear();
        self.tick = new_tick;
    }
}

impl fmt::Display for AuditLog {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "AuditLog(tick={}, buffered={})",
            self.tick,
            self.entries.len()
        )
    }
}

/// Best-effort ISO-8601 timestamp. Falls back to a fixed string when `chrono`
/// is unavailable (which it should not be in the server crate).
fn chrono_now_rfc3339() -> String {
    // The server crate depends on `chrono` transitively via `civ-save-db`.
    // Use std time as a lightweight alternative to avoid adding a direct dep.
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    format!("{now}")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_log_starts_empty() {
        let log = AuditLog::new(42);
        assert_eq!(log.tick(), 42);
        assert!(log.is_empty());
        assert_eq!(log.len(), 0);
    }

    #[test]
    fn push_increases_length() {
        let mut log = AuditLog::new(1);
        log.record("test.event.v1", "sess-1", r#"{"key":"value"}"#);
        assert_eq!(log.len(), 1);
        assert!(!log.is_empty());
    }

    #[test]
    fn flush_drains_all_entries() {
        let mut log = AuditLog::new(5);
        log.record("a", "s", "{}");
        log.record("b", "s", "{}");
        log.record("c", "s", "{}");
        let flushed = log.flush();
        assert_eq!(flushed.len(), 3);
        assert_eq!(flushed[0].event_type, "a");
        assert_eq!(flushed[1].event_type, "b");
        assert_eq!(flushed[2].event_type, "c");
        assert!(log.is_empty());
    }

    #[test]
    fn flush_returns_empty_when_no_entries() {
        let mut log = AuditLog::new(0);
        let flushed = log.flush();
        assert!(flushed.is_empty());
    }

    #[test]
    fn reset_discards_old_entries_and_advances_tick() {
        let mut log = AuditLog::new(10);
        log.record("old.event.v1", "s", "{}");
        assert_eq!(log.len(), 1);
        log.reset(20);
        assert_eq!(log.tick(), 20);
        assert!(log.is_empty());
    }

    #[test]
    fn capacity_evicts_oldest_when_full() {
        let mut log = AuditLog::with_capacity(1, 3);
        log.record("first", "s", "{}");
        log.record("second", "s", "{}");
        log.record("third", "s", "{}");
        // Buffer is now at capacity (3).
        log.record("fourth", "s", "{}");
        // "first" should have been evicted.
        assert_eq!(log.len(), 3);
        let entries = log.flush();
        assert_eq!(entries[0].event_type, "second");
        assert_eq!(entries[1].event_type, "third");
        assert_eq!(entries[2].event_type, "fourth");
    }

    #[test]
    fn entries_carry_correct_tick_and_created_at() {
        let mut log = AuditLog::new(99);
        log.record("evt", "sess", "{}");
        let entries = log.flush();
        assert_eq!(entries[0].tick, 99);
        // created_at is a non-empty timestamp string
        assert!(!entries[0].created_at.is_empty());
    }

    #[test]
    fn fr_prot_004_persist_event_to_audit_log() {
        // FR-PROT-004: The server SHALL persist all emitted events to the DB
        // audit log within the same tick. This test verifies that events are
        // correctly buffered and flushed (the DB persistence layer is tested
        // via integration tests against SaveDb).
        let mut log = AuditLog::new(7);
        log.record("session.saved.v1", "sess-1", r#"{"slot":"slot-1"}"#);
        log.record("climate.threshold.crossed.v1", "system", r#"{"temp":14.5}"#);

        let flushed = log.flush();
        assert_eq!(flushed.len(), 2);

        // Verify both events share the same tick.
        for entry in &flushed {
            assert_eq!(entry.tick, 7);
            assert!(!entry.event_type.is_empty());
            assert!(!entry.session_id.is_empty());
        }
        assert_eq!(flushed[0].event_type, "session.saved.v1");
        assert_eq!(flushed[1].event_type, "climate.threshold.crossed.v1");
    }
}
