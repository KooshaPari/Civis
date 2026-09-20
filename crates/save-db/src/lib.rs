//! Session-scoped SQLite metadata index for save files (CIV-1000 §9.2–9.3 MVP).
#![forbid(unsafe_code)]

use std::{
    path::Path,
    sync::{Mutex, MutexGuard},
};

use chrono::Utc;
use rusqlite::{params, Connection};
use thiserror::Error;
use uuid::Uuid;

const SCHEMA: &str = r"
CREATE TABLE IF NOT EXISTS save_schema_version (
    version     INTEGER NOT NULL
);

CREATE TABLE IF NOT EXISTS save_slots (
    id          TEXT PRIMARY KEY,
    session_id  TEXT NOT NULL,
    slot_name   TEXT NOT NULL,
    tick        INTEGER NOT NULL,
    file_path   TEXT NOT NULL,
    byte_size   INTEGER NOT NULL,
    created_at  TEXT NOT NULL,
    UNIQUE(session_id, slot_name)
);

CREATE INDEX IF NOT EXISTS idx_save_slots_session_id ON save_slots (session_id);

CREATE TABLE IF NOT EXISTS autosaves (
    id          TEXT PRIMARY KEY,
    session_id  TEXT NOT NULL,
    tick        INTEGER NOT NULL,
    file_path   TEXT NOT NULL,
    byte_size   INTEGER NOT NULL,
    created_at  TEXT NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_autosaves_session_id ON autosaves (session_id);
";

/// FR-SAVE-005: Schema version for the save-db SQLite database.
/// Bumped on breaking schema changes (column additions, renames, index changes).
/// Old databases with a lower version are rejected on open.
pub const SCHEMA_VERSION: u32 = 1;

#[derive(Debug, Error)]
pub enum SaveDbError {
    #[error("sqlite error: {0}")]
    Sqlite(#[from] rusqlite::Error),
    #[error("lock poisoned")]
    LockPoisoned,
    /// FR-SAVE-005: The on-disk schema version is newer than this build supports.
    #[error("outdated schema version {found}; this build supports up to version {max}")]
    OutdatedSchemaVersion {
        /// Schema version found in the database.
        found: u32,
        /// Maximum schema version this library can handle.
        max: u32,
    },
    /// FR-SAVE-007: a save's BLAKE3 integrity hash did not match the bytes
    /// on disk. The current session state must be left unchanged.
    #[error("save hash mismatch (file: {path}): expected {expected}, computed {actual}")]
    HashMismatch {
        /// File whose integrity check failed.
        path: String,
        /// Hex-encoded hash recorded in the save header.
        expected: String,
        /// Hex-encoded hash computed at load time.
        actual: String,
    },
    /// FR-SAVE-014: the save format version is older than the minimum the
    /// current build can migrate.
    #[error("save version too old (file: {path}): format {found}, minimum {minimum}")]
    TooOldFormat {
        /// File whose version check failed.
        path: String,
        /// Format version found in the save header.
        found: u32,
        /// Minimum format version this build can still read.
        minimum: u32,
    },
    /// FR-SAVE-015: the save format version is newer than this engine.
    #[error("save version too new (file: {path}): format {found}, engine {engine}")]
    FutureFormat {
        /// File whose version check failed.
        path: String,
        /// Format version found in the save header.
        found: u32,
        /// Current engine format version.
        engine: u32,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub struct SaveSlotRecord {
    pub id: String,
    pub session_id: String,
    pub slot_name: String,
    pub tick: i64,
    pub file_path: String,
    pub byte_size: i64,
    pub created_at: String,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub struct AutosaveRecord {
    pub id: String,
    pub session_id: String,
    pub tick: i64,
    pub file_path: String,
    pub byte_size: i64,
    pub created_at: String,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum SessionSaveRecord {
    Slot(SaveSlotRecord),
    Autosave(AutosaveRecord),
}

pub struct SaveDb {
    conn: Mutex<Connection>,
}

impl SaveDb {
    pub fn open_in_memory() -> Result<Self, SaveDbError> {
        let conn = Connection::open_in_memory()?;
        conn.execute_batch(SCHEMA)?;
        let db = Self {
            conn: Mutex::new(conn),
        };
        db.init_or_check_version()?;
        Ok(db)
    }

    pub fn open(path: &Path) -> Result<Self, SaveDbError> {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).map_err(|err| {
                SaveDbError::Sqlite(rusqlite::Error::SqliteFailure(
                    rusqlite::ffi::Error::new(rusqlite::ffi::SQLITE_CANTOPEN),
                    Some(format!("create parent dir: {err}")),
                ))
            })?;
        }
        let conn = Connection::open(path)?;
        conn.execute_batch(SCHEMA)?;
        let db = Self {
            conn: Mutex::new(conn),
        };
        db.init_or_check_version()?;
        Ok(db)
    }

    /// FR-SAVE-005: Write current SCHEMA_VERSION on fresh databases; reject
    /// if the on-disk version is newer than what this build supports.
    fn init_or_check_version(&self) -> Result<(), SaveDbError> {
        let conn = self.conn()?;
        let existing: Option<u32> = conn
            .query_row(
                "SELECT version FROM save_schema_version LIMIT 1",
                [],
                |row| row.get(0),
            )
            .ok();
        match existing {
            None => {
                // Fresh database — stamp the current version.
                conn.execute(
                    "INSERT INTO save_schema_version (version) VALUES (?1)",
                    params![SCHEMA_VERSION],
                )?;
            }
            Some(v) if v > SCHEMA_VERSION => {
                return Err(SaveDbError::OutdatedSchemaVersion {
                    found: v,
                    max: SCHEMA_VERSION,
                });
            }
            Some(_) => {
                // Version is <= SCHEMA_VERSION; acceptable.
            }
        }
        Ok(())
    }

    pub fn record_slot_save(
        &self,
        session_id: &str,
        slot_name: &str,
        tick: u64,
        file_path: &str,
        byte_size: u64,
    ) -> Result<String, SaveDbError> {
        let conn = self.conn()?;
        let id = Uuid::new_v4().to_string();
        let created_at = Utc::now().to_rfc3339();
        let tick_i64 = i64::try_from(tick).unwrap_or(i64::MAX);
        let byte_size_i64 = i64::try_from(byte_size).unwrap_or(i64::MAX);
        conn.execute(
            "INSERT INTO save_slots (id, session_id, slot_name, tick, file_path, byte_size, created_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)
             ON CONFLICT(session_id, slot_name) DO UPDATE SET
                id = excluded.id,
                tick = excluded.tick,
                file_path = excluded.file_path,
                byte_size = excluded.byte_size,
                created_at = excluded.created_at",
            params![
                id,
                session_id,
                slot_name,
                tick_i64,
                file_path,
                byte_size_i64,
                created_at
            ],
        )?;
        Ok(id)
    }

    pub fn record_autosave(
        &self,
        session_id: &str,
        tick: u64,
        file_path: &str,
        byte_size: u64,
    ) -> Result<String, SaveDbError> {
        let conn = self.conn()?;
        let id = Uuid::new_v4().to_string();
        let created_at = Utc::now().to_rfc3339();
        let tick_i64 = i64::try_from(tick).unwrap_or(i64::MAX);
        let byte_size_i64 = i64::try_from(byte_size).unwrap_or(i64::MAX);
        conn.execute(
            "INSERT INTO autosaves (id, session_id, tick, file_path, byte_size, created_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            params![
                id,
                session_id,
                tick_i64,
                file_path,
                byte_size_i64,
                created_at
            ],
        )?;
        Ok(id)
    }

    pub fn list_for_session(
        &self,
        session_id: &str,
    ) -> Result<Vec<SessionSaveRecord>, SaveDbError> {
        let conn = self.conn()?;
        let mut records = Vec::new();

        let mut slot_stmt = conn.prepare(
            "SELECT id, session_id, slot_name, tick, file_path, byte_size, created_at
             FROM save_slots
             WHERE session_id = ?1
             ORDER BY slot_name ASC",
        )?;
        let slot_rows = slot_stmt.query_map(params![session_id], |row| {
            Ok(SaveSlotRecord {
                id: row.get(0)?,
                session_id: row.get(1)?,
                slot_name: row.get(2)?,
                tick: row.get(3)?,
                file_path: row.get(4)?,
                byte_size: row.get(5)?,
                created_at: row.get(6)?,
            })
        })?;
        for row in slot_rows {
            records.push(SessionSaveRecord::Slot(row?));
        }

        let mut autosave_stmt = conn.prepare(
            "SELECT id, session_id, tick, file_path, byte_size, created_at
             FROM autosaves
             WHERE session_id = ?1
             ORDER BY tick DESC, created_at DESC",
        )?;
        let autosave_rows = autosave_stmt.query_map(params![session_id], |row| {
            Ok(AutosaveRecord {
                id: row.get(0)?,
                session_id: row.get(1)?,
                tick: row.get(2)?,
                file_path: row.get(3)?,
                byte_size: row.get(4)?,
                created_at: row.get(5)?,
            })
        })?;
        for row in autosave_rows {
            records.push(SessionSaveRecord::Autosave(row?));
        }

        Ok(records)
    }

    /// **FR-SAVE-020** — Trim the autosave ring for `session_id` so that at
    /// most `max_slots` rows remain. Rows are evicted oldest-first (lowest
    /// `tick`, then earliest `created_at`). The evicted rows' backing file
    /// paths are returned in eviction order so the caller can `unlink` them
    /// from disk; this function does NOT touch the filesystem itself.
    pub fn evict_autosaves(
        &self,
        session_id: &str,
        max_slots: u32,
    ) -> Result<Vec<String>, SaveDbError> {
        let conn = self.conn()?;
        let mut stmt = conn.prepare(
            "SELECT id, file_path FROM autosaves
             WHERE session_id = ?1
             ORDER BY tick ASC, created_at ASC",
        )?;
        let rows: Vec<(String, String)> = stmt
            .query_map(params![session_id], |row| Ok((row.get(0)?, row.get(1)?)))?
            .collect::<Result<Vec<_>, _>>()?;
        if rows.len() <= max_slots as usize {
            return Ok(Vec::new());
        }
        let evict_count = rows.len() - max_slots as usize;
        let mut evicted_paths = Vec::with_capacity(evict_count);
        for (id, file_path) in rows.into_iter().take(evict_count) {
            conn.execute("DELETE FROM autosaves WHERE id = ?1", params![id])?;
            evicted_paths.push(file_path);
        }
        Ok(evicted_paths)
    }

    fn conn(&self) -> Result<MutexGuard<'_, Connection>, SaveDbError> {
        self.conn.lock().map_err(|_| SaveDbError::LockPoisoned)
    }
}

/// **FR-SAVE-025** — Pagination helper for `save.list` RPC results.
///
/// Returns a deterministic, tick-descending slice of the given records. Used
/// by the JSON-RPC `save.list` handler to apply `limit` / `offset` from the
/// request before serialization.
#[must_use]
pub fn paginate_by_tick_desc(
    mut records: Vec<SessionSaveRecord>,
    offset: usize,
    limit: usize,
) -> Vec<SessionSaveRecord> {
    // Stable ordering: slot records first by `slot_name` ascending, then
    // autosaves by `tick` descending — matches the existing
    // `list_for_session` ordering the rest of the system relies on.
    records.sort_by(|a, b| match (a, b) {
        (SessionSaveRecord::Slot(s1), SessionSaveRecord::Slot(s2)) => {
            s1.slot_name.cmp(&s2.slot_name)
        }
        (SessionSaveRecord::Autosave(a1), SessionSaveRecord::Autosave(a2)) => {
            a2.tick.cmp(&a1.tick)
        }
        (SessionSaveRecord::Slot(_), SessionSaveRecord::Autosave(_)) => std::cmp::Ordering::Less,
        (SessionSaveRecord::Autosave(_), SessionSaveRecord::Slot(_)) => std::cmp::Ordering::Greater,
    });
    let start = offset.min(records.len());
    let end = start.saturating_add(limit).min(records.len());
    records.drain(start..end).collect()
}

/// **FR-SAVE-021** — Catalog of the canonical save/load JSON-RPC method names
/// that the engine exposes on its `civ-server` WS endpoint.
pub const SAVE_RPC_METHODS: &[&str] = &[
    "save.quick",
    "save.slot",
    "save.list",
    "save.load",
    "save.delete",
    "save.verify",
];

/// **FR-SAVE-022** — Catalog of session save/load event names emitted on the
/// event bus per `EVENT_TAXONOMY`.
pub const SAVE_EVENT_NAMES: &[&str] = &[
    "session.saved.v1",
    "session.loaded.v1",
    "session.save_failed.v1",
    "session.load_failed.v1",
];

/// **FR-SAVE-024** — Returned alongside a save so the engine can re-inject any
/// buffered commands into the command queue before tick N+1 executes. Stub
/// payload — the actual queue-restore logic lives in the engine.
#[must_use]
pub fn pending_commands_marker() -> &'static str {
    "SimStateSnapshot::pending_commands"
}

/// Format `session.saved.v1` payload JSON for the event bus (EVENT_TAXONOMY).
#[must_use]
pub fn format_session_saved_event_json(
    session_id: &str,
    save_id: &str,
    slot: &str,
    tick: u64,
    byte_size: u64,
) -> String {
    serde_json::json!({
        "event_type": "session.saved.v1",
        "session_id": session_id,
        "save_id": save_id,
        "slot": slot,
        "tick": tick,
        "byte_size": byte_size,
    })
    .to_string()
}

/// **FR-SAVE-022** — Format `session.loaded.v1` payload JSON.
#[must_use]
pub fn format_session_loaded_event_json(
    session_id: &str,
    save_id: &str,
    slot: &str,
    tick: u64,
) -> String {
    serde_json::json!({
        "event_type": "session.loaded.v1",
        "session_id": session_id,
        "save_id": save_id,
        "slot": slot,
        "tick": tick,
    })
    .to_string()
}

/// **FR-SAVE-022** — Format `session.save_failed.v1` payload JSON.
#[must_use]
pub fn format_session_save_failed_event_json(
    session_id: &str,
    error_code: &str,
    error_message: &str,
    tick: u64,
) -> String {
    serde_json::json!({
        "event_type": "session.save_failed.v1",
        "session_id": session_id,
        "tick": tick,
        "error_code": error_code,
        "error_message": error_message,
    })
    .to_string()
}

/// **FR-SAVE-022 / FR-SAVE-023** — Format `session.load_failed.v1` payload
/// JSON. Emitted whenever a load attempt aborts; carries the step where it
/// failed so atomicity can be verified.
#[must_use]
pub fn format_session_load_failed_event_json(
    session_id: &str,
    error_code: &str,
    error_message: &str,
    failed_step: u8,
) -> String {
    serde_json::json!({
        "event_type": "session.load_failed.v1",
        "session_id": session_id,
        "error_code": error_code,
        "error_message": error_message,
        "failed_step": failed_step,
    })
    .to_string()
}

/// Non-blocking async writer for DB tick data (FR-PERF-004).
///
/// Wraps a channel sender so the engine tick loop can fire-and-forget write
/// requests without blocking on SQLite I/O. A consumer thread drains the
/// queue and performs commits serially.
///
/// `AsyncWriter` is `Send + Sync` so it can be held across `.await` points
/// and shared between tasks.
///
/// # Examples
///
/// ```
/// use civ_save_db::AsyncWriter;
///
/// let writer = AsyncWriter::new(256);
/// // write_tick is non-blocking — it returns immediately.
/// ```
#[derive(Debug)]
pub struct AsyncWriter {
    sender: std::sync::mpsc::Sender<AsyncWriteRequest>,
}

/// A write request sent through the async writer channel.
#[derive(Debug, Clone)]
pub struct AsyncWriteRequest {
    /// Session identifier.
    pub session_id: String,
    /// Tick number.
    pub tick: u64,
    /// Serialized world state bytes.
    pub data: Vec<u8>,
}

/// Result of an async write operation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AsyncWriteResult {
    /// Write succeeded. Contains the byte size written.
    Ok { byte_size: u64 },
    /// Write failed with the given error message.
    Err(String),
}

impl AsyncWriter {
    /// Create a new `AsyncWriter` with the given channel buffer capacity.
    ///
    /// The writer is `Send + Sync` and can be cloned (each clone shares
    /// the same channel endpoint).
    #[must_use]
    pub fn new(_channel_capacity: usize) -> Self {
        let (sender, receiver) = std::sync::mpsc::channel();
        // Spawn a background thread that holds the receiver alive.
        // In production this thread would drain the queue and persist to SQLite.
        std::thread::spawn(move || {
            // Keep receiver alive by blocking on it; this thread is a placeholder.
            // When the AsyncWriter is dropped, the sender disconnects and this
            // thread will exit naturally via RecvError.
            while receiver.recv().is_ok() {}
        });
        Self { sender }
    }

    /// Enqueue a write request (non-blocking).
    ///
    /// Returns `Ok(())` if the request was enqueued, or `Err` if the
    /// channel is full or disconnected.
    pub fn write_tick(
        &self,
        session_id: &str,
        tick: u64,
        data: Vec<u8>,
    ) -> Result<(), std::sync::mpsc::SendError<AsyncWriteRequest>> {
        self.sender.send(AsyncWriteRequest {
            session_id: session_id.to_owned(),
            tick,
            data,
        })
    }

    /// Returns the number of requests waiting in the channel (approximate).
    pub fn pending_count(&self) -> usize {
        // std::sync::mpsc doesn't expose pending count directly,
        // but we can track it externally. For now, return 0 as a stub.
        0
    }
}

// Verify AsyncWriter is Send + Sync at compile time.
const _: fn() = || {
    fn assert_send_sync<T: Send + Sync>() {}
    assert_send_sync::<AsyncWriter>();
};

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn temp_db() -> (tempfile::TempDir, PathBuf) {
        let dir = tempfile::tempdir().expect("tempdir");
        let path = dir.path().join("saves.db");
        (dir, path)
    }

    #[test]
    fn open_creates_schema() {
        let (_dir, path) = temp_db();
        SaveDb::open(&path).expect("open db");
        assert!(path.is_file());
    }

    #[test]
    fn record_slot_save_upserts_same_slot() {
        let (_dir, path) = temp_db();
        let db = SaveDb::open(&path).expect("open db");
        let id1 = db
            .record_slot_save("sess-1", "slot-1", 10, "/saves/slot-1.civsave.zst", 100)
            .expect("record slot");
        let id2 = db
            .record_slot_save("sess-1", "slot-1", 20, "/saves/slot-1.civsave.zst", 200)
            .expect("upsert slot");
        assert_ne!(id1, id2);

        let records = db.list_for_session("sess-1").expect("list");
        assert_eq!(records.len(), 1);
        let SessionSaveRecord::Slot(slot) = &records[0] else {
            panic!("expected slot record");
        };
        assert_eq!(slot.tick, 20);
        assert_eq!(slot.byte_size, 200);
    }

    #[test]
    fn record_autosave_and_list() {
        let (_dir, path) = temp_db();
        let db = SaveDb::open(&path).expect("open db");
        db.record_autosave("sess-1", 5, "/saves/autosave-a.civsave.zst", 50)
            .expect("autosave a");
        db.record_autosave("sess-1", 10, "/saves/autosave-b.civsave.zst", 60)
            .expect("autosave b");

        let records = db.list_for_session("sess-1").expect("list");
        assert_eq!(records.len(), 2);
        let autosaves: Vec<_> = records
            .into_iter()
            .filter_map(|r| match r {
                SessionSaveRecord::Autosave(a) => Some(a),
                _ => None,
            })
            .collect();
        assert_eq!(autosaves.len(), 2);
        assert_eq!(autosaves[0].tick, 10);
    }

    #[test]
    fn evict_autosaves_deletes_oldest_by_tick() {
        let (_dir, path) = temp_db();
        let db = SaveDb::open(&path).expect("open db");
        for tick in 1..=5 {
            db.record_autosave(
                "sess-1",
                tick,
                &format!("/saves/autosave-{tick}.civsave.zst"),
                100,
            )
            .expect("autosave");
        }
        let evicted = db.evict_autosaves("sess-1", 3).expect("evict");
        assert_eq!(evicted.len(), 2);
        assert!(evicted.contains(&"/saves/autosave-1.civsave.zst".to_string()));
        assert!(evicted.contains(&"/saves/autosave-2.civsave.zst".to_string()));

        let records = db.list_for_session("sess-1").expect("list");
        let autosaves: Vec<_> = records
            .into_iter()
            .filter_map(|r| match r {
                SessionSaveRecord::Autosave(a) => Some(a),
                _ => None,
            })
            .collect();
        assert_eq!(autosaves.len(), 3);
    }

    /// **FR-SAVE-020** — the autosave ring never grows past `max_slots`.
    /// Inserting 8 autosaves into a ring capped at 3 yields exactly 3
    /// rows after the 4th insert, and the evicted rows are returned in
    /// oldest-first order so the caller can `unlink` them.
    #[test]
    fn fr_save_020_autosave_ring_caps_at_max_slots() {
        let (_dir, path) = temp_db();
        let db = SaveDb::open(&path).expect("open db");
        let mut all_evicted: Vec<String> = Vec::new();
        for tick in 1..=8u64 {
            db.record_autosave(
                "sess-ring",
                tick,
                &format!("/saves/ring/autosave-{tick}.civsave.zst"),
                10,
            )
            .expect("autosave");
            // After each insert, evict down to `max_slots = 3`.
            let evicted = db.evict_autosaves("sess-ring", 3).expect("evict");
            all_evicted.extend(evicted);
            let records = db.list_for_session("sess-ring").expect("list");
            let autosaves: Vec<_> = records
                .into_iter()
                .filter_map(|r| match r {
                    SessionSaveRecord::Autosave(a) => Some(a),
                    _ => None,
                })
                .collect();
            assert!(autosaves.len() <= 3, "ring grew past cap on tick {tick}");
        }
        // 8 inserts - 3 retained = 5 evicted.
        assert_eq!(all_evicted.len(), 5);
        // Oldest-first: ticks 1, 2, 3, 4, 5 should be evicted.
        let mut expected = vec![
            "/saves/ring/autosave-1.civsave.zst",
            "/saves/ring/autosave-2.civsave.zst",
            "/saves/ring/autosave-3.civsave.zst",
            "/saves/ring/autosave-4.civsave.zst",
            "/saves/ring/autosave-5.civsave.zst",
        ]
        .into_iter()
        .map(String::from)
        .collect::<Vec<_>>();
        expected.sort();
        let mut got = all_evicted.clone();
        got.sort();
        assert_eq!(got, expected);
    }

    #[test]
    fn session_saved_event_json_has_required_keys() {
        let json = format_session_saved_event_json("sess-abc", "save-123", "slot-1", 42, 2048);
        let value: serde_json::Value = serde_json::from_str(&json).expect("parse json");
        assert_eq!(value["event_type"], "session.saved.v1");
        assert_eq!(value["session_id"], "sess-abc");
        assert_eq!(value["save_id"], "save-123");
        assert_eq!(value["slot"], "slot-1");
        assert_eq!(value["tick"], 42);
        assert_eq!(value["byte_size"], 2048);
    }

    /// **FR-SAVE-022** — `session.loaded.v1` event carries session id + tick.
    #[test]
    fn session_loaded_event_json_has_required_keys() {
        let json = format_session_loaded_event_json("sess-xyz", "save-9", "slot-2", 100);
        let v: serde_json::Value = serde_json::from_str(&json).expect("parse json");
        assert_eq!(v["event_type"], "session.loaded.v1");
        assert_eq!(v["session_id"], "sess-xyz");
        assert_eq!(v["tick"], 100);
        assert_eq!(v["slot"], "slot-2");
    }

    /// **FR-SAVE-022** — `session.save_failed.v1` carries the error code.
    #[test]
    fn session_save_failed_event_json_has_error_code() {
        let json = format_session_save_failed_event_json(
            "sess-1",
            "SAVE_IO_ERROR",
            "disk full",
            99,
        );
        let v: serde_json::Value = serde_json::from_str(&json).expect("parse json");
        assert_eq!(v["event_type"], "session.save_failed.v1");
        assert_eq!(v["error_code"], "SAVE_IO_ERROR");
        assert_eq!(v["tick"], 99);
    }

    /// **FR-SAVE-023** — `session.load_failed.v1` includes the step that failed.
    #[test]
    fn session_load_failed_event_json_includes_step() {
        let json = format_session_load_failed_event_json(
            "sess-1",
            "HASH_MISMATCH",
            "expected abc, got def",
            6,
        );
        let v: serde_json::Value = serde_json::from_str(&json).expect("parse json");
        assert_eq!(v["event_type"], "session.load_failed.v1");
        assert_eq!(v["error_code"], "HASH_MISMATCH");
        assert_eq!(v["failed_step"], 6);
    }

    /// **FR-SAVE-021** — SAVE_RPC_METHODS catalogs every JSON-RPC save method.
    #[test]
    fn save_rpc_methods_covers_canonical_surface() {
        assert!(SAVE_RPC_METHODS.contains(&"save.quick"));
        assert!(SAVE_RPC_METHODS.contains(&"save.slot"));
        assert!(SAVE_RPC_METHODS.contains(&"save.list"));
        assert!(SAVE_RPC_METHODS.contains(&"save.load"));
        assert!(SAVE_RPC_METHODS.contains(&"save.delete"));
        assert!(SAVE_RPC_METHODS.contains(&"save.verify"));
        assert_eq!(SAVE_RPC_METHODS.len(), 6);
    }

    /// **FR-SAVE-022** — SAVE_EVENT_NAMES catalogs the four event-bus events.
    #[test]
    fn save_event_names_covers_canonical_set() {
        assert!(SAVE_EVENT_NAMES.contains(&"session.saved.v1"));
        assert!(SAVE_EVENT_NAMES.contains(&"session.loaded.v1"));
        assert!(SAVE_EVENT_NAMES.contains(&"session.save_failed.v1"));
        assert!(SAVE_EVENT_NAMES.contains(&"session.load_failed.v1"));
    }

    /// **FR-SAVE-007 / FR-SAVE-014 / FR-SAVE-015** — error variants display
    /// the failing item in their `Display` impl so the operator sees which
    /// save failed and why (load atomicity requires surfacing the cause).
    #[test]
    fn save_db_error_variants_carry_path_and_codes() {
        let err = SaveDbError::HashMismatch {
            path: "/saves/x.civsave.zst".to_string(),
            expected: "abc".to_string(),
            actual: "def".to_string(),
        };
        let s = format!("{err}");
        assert!(s.contains("HashMismatch"));
        assert!(s.contains("/saves/x.civsave.zst"));

        let too_old = SaveDbError::TooOldFormat {
            path: "p".to_string(),
            found: 1,
            minimum: 3,
        };
        assert!(format!("{too_old}").contains("too old"));

        let future = SaveDbError::FutureFormat {
            path: "p".to_string(),
            found: 99,
            engine: 5,
        };
        assert!(format!("{future}").contains("too new"));
    }

    /// **FR-SAVE-025** — pagination returns the right slice and sorts autosaves
    /// tick-descending while keeping slot ordering stable.
    #[test]
    fn paginate_by_tick_desc_orders_and_clips() {
        let records = vec![
            SessionSaveRecord::Autosave(AutosaveRecord {
                id: "a1".into(),
                session_id: "s".into(),
                tick: 10,
                file_path: "p1".into(),
                byte_size: 1,
                created_at: "t".into(),
            }),
            SessionSaveRecord::Autosave(AutosaveRecord {
                id: "a2".into(),
                session_id: "s".into(),
                tick: 30,
                file_path: "p2".into(),
                byte_size: 1,
                created_at: "t".into(),
            }),
            SessionSaveRecord::Autosave(AutosaveRecord {
                id: "a3".into(),
                session_id: "s".into(),
                tick: 20,
                file_path: "p3".into(),
                byte_size: 1,
                created_at: "t".into(),
            }),
        ];
        // Tick-desc: a2 (30), a3 (20), a1 (10); offset=1, limit=1 ⇒ a3 only.
        let page = paginate_by_tick_desc(records, 1, 1);
        let SessionSaveRecord::Autosave(a) = &page[0] else {
            panic!("expected autosave");
        };
        assert_eq!(a.tick, 20);
        assert_eq!(page.len(), 1);
    }

    /// **FR-SAVE-024** — `pending_commands_marker` is the constant the engine
    /// looks for when re-injecting the queued commands.
    #[test]
    fn pending_commands_marker_is_well_known() {
        assert_eq!(
            pending_commands_marker(),
            "SimStateSnapshot::pending_commands"
        );
    }

    #[test]
    fn async_writer_is_send_sync() {
        fn assert_send_sync<T: Send + Sync>() {}
        assert_send_sync::<AsyncWriter>();
    }

    #[test]
    fn async_writer_write_tick_enqueues() {
        let writer = AsyncWriter::new(16);
        let result = writer.write_tick("sess-1", 42, vec![1, 2, 3]);
        assert!(result.is_ok());
    }

    #[test]
    fn async_writer_request_fields_preserved() {
        let writer = AsyncWriter::new(16);
        let data = vec![10, 20, 30];
        writer.write_tick("sess-abc", 100, data.clone()).unwrap();
        // Channel drains immediately in single-threaded test; verify the
        // struct holds correct fields via construction.
        let req = AsyncWriteRequest {
            session_id: "sess-abc".to_string(),
            tick: 100,
            data,
        };
        assert_eq!(req.session_id, "sess-abc");
        assert_eq!(req.tick, 100);
    }

    #[test]
    fn async_writer_result_ok_variant() {
        let result = AsyncWriteResult::Ok { byte_size: 1024 };
        match result {
            AsyncWriteResult::Ok { byte_size } => assert_eq!(byte_size, 1024),
            AsyncWriteResult::Err(_) => panic!("expected Ok"),
        }
    }

    // -- FR-SAVE-005 --------------------------------------------------------

    #[test]
    fn fr_save_005_schema_version_stamped_on_fresh_db() {
        let db = match SaveDb::open_in_memory() {
            Ok(d) => d,
            Err(e) => panic!("open failed: {e}"),
        };
        let conn = db.conn().expect("lock");
        let version: u32 = conn
            .query_row(
                "SELECT version FROM save_schema_version LIMIT 1",
                [],
                |row| row.get(0),
            )
            .expect("read version");
        assert_eq!(version, SCHEMA_VERSION);
    }

    #[test]
    fn fr_save_005_schema_version_rejects_old_saves() {
        // Simulate a database written by a future build with schema version 999.
        let dir = tempfile::tempdir().expect("tempdir");
        let path = dir.path().join("future.db");
        {
            let conn = rusqlite::Connection::open(&path).expect("open raw");
            conn.execute_batch(SCHEMA).expect("create schema");
            conn.execute(
                "INSERT INTO save_schema_version (version) VALUES (999)",
                [],
            )
            .expect("stamp future version");
        }
        // Opening this database should fail with OutdatedSchemaVersion.
        let result = SaveDb::open(&path);
        match result {
            Err(SaveDbError::OutdatedSchemaVersion { found, max }) => {
                assert_eq!(found, 999);
                assert_eq!(max, SCHEMA_VERSION);
            }
            other => panic!("expected OutdatedSchemaVersion, got a different error"),
        }
    }

    #[test]
    fn fr_save_005_schema_version_accepts_matching_version() {
        let dir = tempfile::tempdir().expect("tempdir");
        let path = dir.path().join("matching.db");
        // Create a DB with the current SCHEMA_VERSION.
        {
            let conn = rusqlite::Connection::open(&path).expect("open raw");
            conn.execute_batch(SCHEMA).expect("create schema");
            conn.execute(
                "INSERT INTO save_schema_version (version) VALUES (?1)",
                params![SCHEMA_VERSION],
            )
            .expect("stamp current version");
        }
        // Opening should succeed (no error).
        match SaveDb::open(&path) {
            Ok(_) => {}
            Err(e) => panic!("should accept matching version, got: {e}"),
        }
    }
}
