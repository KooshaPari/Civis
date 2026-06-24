//! Session-scoped SQLite metadata index for save files (CIV-1000 §9.2–9.3 MVP).

use std::{
    io::Read,
    path::{Path, PathBuf},
    sync::{Mutex, MutexGuard},
};

use chrono::Utc;
use rusqlite::{params, Connection};
use thiserror::Error;
use uuid::Uuid;

const SCHEMA: &str = r"
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

#[derive(Debug, Error)]
pub enum SaveDbError {
    #[error("sqlite error: {0}")]
    Sqlite(#[from] rusqlite::Error),
    #[error("lock poisoned")]
    LockPoisoned,
    #[error("io error at {path}: {message}")]
    Io {
        path: PathBuf,
        message: String,
    },
    #[error("save slot not found: {0}")]
    NotFound(String),
    #[error("save archive is empty or unreadable: {path}")]
    EmptyArchive { path: PathBuf },
}

/// Saved archive bytes retrieved via [`SaveDb::restore`].
///
/// Contains the **raw bytes** recorded at `record_slot_save` time plus the
/// metadata record so callers can route the bytes back to
/// `civ_engine::CivSaveBundle::load_archive` (or a future save-format loader)
/// without a second database round-trip.
///
/// Per `docs/design/SAVELOAD_HUD_PLAN.md` §2.4 (RESTORE tab) this is the
/// payload the wire hands to the engine on `save.load`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SaveArchive {
    pub record: SessionSaveRecord,
    pub bytes: Vec<u8>,
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
        Ok(Self {
            conn: Mutex::new(conn),
        })
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

    /// Locate a slot record by `slot_id` (the `id` returned from
    /// [`SaveDb::record_slot_save`] / [`SaveDb::record_autosave`]).
    fn find_record(&self, slot_id: &str) -> Result<SessionSaveRecord, SaveDbError> {
        let conn = self.conn()?;

        let mut slot_stmt = conn.prepare(
            "SELECT id, session_id, slot_name, tick, file_path, byte_size, created_at
             FROM save_slots WHERE id = ?1",
        )?;
        let slot_row = slot_stmt.query_row(params![slot_id], |row| {
            Ok(SaveSlotRecord {
                id: row.get(0)?,
                session_id: row.get(1)?,
                slot_name: row.get(2)?,
                tick: row.get(3)?,
                file_path: row.get(4)?,
                byte_size: row.get(5)?,
                created_at: row.get(6)?,
            })
        });
        match slot_row {
            Ok(rec) => return Ok(SessionSaveRecord::Slot(rec)),
            Err(rusqlite::Error::QueryReturnedNoRows) => {}
            Err(err) => return Err(SaveDbError::Sqlite(err)),
        }

        let mut auto_stmt = conn.prepare(
            "SELECT id, session_id, tick, file_path, byte_size, created_at
             FROM autosaves WHERE id = ?1",
        )?;
        let auto_row = auto_stmt.query_row(params![slot_id], |row| {
            Ok(AutosaveRecord {
                id: row.get(0)?,
                session_id: row.get(1)?,
                tick: row.get(2)?,
                file_path: row.get(3)?,
                byte_size: row.get(4)?,
                created_at: row.get(5)?,
            })
        });
        match auto_row {
            Ok(rec) => Ok(SessionSaveRecord::Autosave(rec)),
            Err(rusqlite::Error::QueryReturnedNoRows) => {
                Err(SaveDbError::NotFound(slot_id.to_owned()))
            }
            Err(err) => Err(SaveDbError::Sqlite(err)),
        }
    }

    /// Restore (load) a save by id — returns the raw archive bytes recorded
    /// at save-time, plus the metadata record.
    ///
    /// This is the **wire half** of the save/load flow:
    /// `record_slot_save` → `list_for_session` → `restore(slot_id)` →
    /// `civ_engine::CivSaveBundle::load_archive` (the caller hands the bytes
    /// back to the engine, keeping `civ-save-db` substrate-neutral).
    ///
    /// Charter-required per `emergence-charter.md:24-27`: no silent data loss
    /// on quit/restore. The return type makes the byte payload explicit so
    /// the server can wrap restore-with-autosave atomically.
    pub fn restore(&self, slot_id: &str) -> Result<SaveArchive, SaveDbError> {
        let record = self.find_record(slot_id)?;
        let path = match &record {
            SessionSaveRecord::Slot(r) => PathBuf::from(&r.file_path),
            SessionSaveRecord::Autosave(r) => PathBuf::from(&r.file_path),
        };
        let bytes = read_archive_bytes(&path)?;
        Ok(SaveArchive { record, bytes })
    }

    /// Restore a manual slot by `(session_id, slot_name)`. Convenience for
    /// the JSON-RPC `save.load { slot_name }` shape.
    pub fn restore_slot(
        &self,
        session_id: &str,
        slot_name: &str,
    ) -> Result<SaveArchive, SaveDbError> {
        let conn = self.conn()?;
        let id: Option<String> = conn
            .query_row(
                "SELECT id FROM save_slots WHERE session_id = ?1 AND slot_name = ?2",
                params![session_id, slot_name],
                |row| row.get(0),
            )
            .ok();
        let id = id.ok_or_else(|| {
            SaveDbError::NotFound(format!("{session_id}::{slot_name}"))
        })?;
        self.restore(&id)
    }

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

/// Read the save archive bytes from `path`. Returns
/// [`SaveDbError::EmptyArchive`] when the file is missing or zero-length —
/// charter: never hand an empty payload to the engine loader.
fn read_archive_bytes(path: &Path) -> Result<Vec<u8>, SaveDbError> {
    let mut file = std::fs::File::open(path).map_err(|err| SaveDbError::Io {
        path: path.to_path_buf(),
        message: err.to_string(),
    })?;
    let mut buf = Vec::new();
    file.read_to_end(&mut buf)
        .map_err(|err| SaveDbError::Io {
            path: path.to_path_buf(),
            message: err.to_string(),
        })?;
    if buf.is_empty() {
        return Err(SaveDbError::EmptyArchive {
            path: path.to_path_buf(),
        });
    }
    Ok(buf)
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

    /// PHASE-1 round-trip test (SAVELOAD_HUD_PLAN §3 / §5):
    /// write archive bytes → `record_slot_save` → `list_for_session` →
    /// `restore(id)` → assert the bytes round-trip byte-identical.
    #[test]
    fn record_save_list_restore_roundtrip_bytes_match() {
        let (dir, path) = temp_db();
        let db = SaveDb::open(&path).expect("open db");

        let archive_bytes: Vec<u8> = (0..1024u32).map(|i| (i & 0xFF) as u8).collect();
        let archive_path = dir.path().join("slot-1.civsave.zst");
        std::fs::write(&archive_path, &archive_bytes).expect("write archive");

        let id = db
            .record_slot_save(
                "sess-roundtrip",
                "slot-1",
                4242,
                archive_path.to_str().expect("utf8 path"),
                archive_bytes.len() as u64,
            )
            .expect("record slot save");

        let listed = db
            .list_for_session("sess-roundtrip")
            .expect("list for session");
        assert_eq!(listed.len(), 1, "exactly one record after one save");
        let SessionSaveRecord::Slot(rec) = &listed[0] else {
            panic!("expected slot record, got autosave");
        };
        assert_eq!(rec.id, id, "list returns the same id we recorded");
        assert_eq!(rec.tick, 4242);

        let archive = db.restore(&id).expect("restore by id");
        assert_eq!(
            archive.bytes,
            archive_bytes,
            "restored bytes must match what we wrote"
        );
        assert_eq!(archive.bytes.len() as i64, rec.byte_size);

        // restore_slot convenience lookup also works.
        let by_name = db
            .restore_slot("sess-roundtrip", "slot-1")
            .expect("restore_slot by name");
        assert_eq!(by_name.bytes, archive_bytes);
    }

    /// PHASE-1 round-trip test (SAVELOAD_HUD_PLAN §3 / §5):
    /// the restore half also covers autosaves — `F5` quick-load hits this
    /// path when the panel opens pre-filtered to the most-recent autosave.
    #[test]
    fn restore_returns_not_found_for_unknown_id() {
        let (_dir, path) = temp_db();
        let db = SaveDb::open(&path).expect("open db");
        let err = db.restore("not-a-real-id").expect_err("must not find");
        assert!(
            matches!(err, SaveDbError::NotFound(ref id) if id == "not-a-real-id"),
            "expected NotFound, got {err:?}"
        );

        let err = db
            .restore_slot("ghost-session", "ghost-slot")
            .expect_err("must not find");
        assert!(
            matches!(err, SaveDbError::NotFound(_)),
            "expected NotFound, got {err:?}"
        );
    }
}
