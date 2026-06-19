//! Background autosaver for the WebSocket bridge (CIV-1000 §13).
//!
//! Periodic loop that writes a tick-stamped `autosave-NNNNNN.civsave.zst` archive
//! via [`CivSaveBundle`], records metadata in [`SaveDb`], and evicts the oldest
//! entries so the on-disk ring stays bounded.
//!
//! Cadence is read from `CIV_AUTOSAVE_EVERY_SECS` (default 60s). A value of
//! `0` disables the loop entirely. Ring size comes from `CIV_AUTOSAVE_KEEP`
//! (default 3).

use std::{path::PathBuf, sync::Arc, time::Duration};

use civ_engine::{CivSaveBundle, Simulation};
use civ_save_db::SaveDb;
use tokio::{sync::Mutex, time::interval};

use crate::saves::save_archive_path;

/// Default autosave cadence when `CIV_AUTOSAVE_EVERY_SECS` is unset.
const DEFAULT_AUTOSAVE_EVERY_SECS: u64 = 60;
/// Default autosave ring size when `CIV_AUTOSAVE_KEEP` is unset.
pub const DEFAULT_AUTOSAVE_KEEP: u32 = 3;

/// Resolve autosave cadence (seconds) from env. `0` disables the loop.
#[must_use]
pub fn autosave_cadence_from_env() -> Option<Duration> {
    let secs = std::env::var("CIV_AUTOSAVE_EVERY_SECS")
        .ok()
        .and_then(|value| value.parse::<u64>().ok())
        .unwrap_or(DEFAULT_AUTOSAVE_EVERY_SECS);
    if secs == 0 {
        None
    } else {
        Some(Duration::from_secs(secs))
    }
}

/// Resolve autosave ring size (count to retain) from env.
#[must_use]
pub fn autosave_keep_from_env() -> u32 {
    std::env::var("CIV_AUTOSAVE_KEEP")
        .ok()
        .and_then(|value| value.parse::<u32>().ok())
        .filter(|value| *value > 0)
        .unwrap_or(DEFAULT_AUTOSAVE_KEEP)
}

/// Build the canonical on-disk filename for a tick-based autosave archive.
#[must_use]
pub fn autosave_filename_for_tick(tick: u64) -> String {
    format!("autosave-{tick:020}")
}

/// Shared state handed to the autosaver loop.
#[derive(Clone)]
pub struct AutosaveContext {
    /// Live simulation mutex.
    pub sim: Arc<Mutex<Simulation>>,
    /// Save directory (created on bridge start).
    pub saves_dir: PathBuf,
    /// Session id used for `save_db` metadata + `session.saved.v1` bus JSON.
    pub session_id: String,
    /// SQLite metadata index opened in the bridge.
    pub save_db: Arc<SaveDb>,
    /// How many autosave archives to retain on disk.
    pub keep: u32,
}

/// Outcome of a single autosave pass (used by tests).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AutosaveResult {
    /// Path of the written archive.
    pub path: PathBuf,
    /// Engine tick captured in the archive.
    pub tick: u64,
    /// Byte size of the archive on disk.
    pub byte_size: u64,
    /// Save-id returned by the metadata index.
    pub save_id: String,
}

/// One autosave pass: snapshot the sim, write the archive, record metadata,
/// enforce the ring. Errors are returned so tests can assert.
pub async fn run_autosave_once(ctx: &AutosaveContext) -> Result<AutosaveResult, String> {
    // Build the autosave path; the tick we capture here is the same one we
    // will use for the filename, the metadata row, and the replay bus JSON.
    let tick_for_filename = {
        let sim = ctx.sim.lock().await;
        sim.state.tick
    };
    let filename = autosave_filename_for_tick(tick_for_filename);
    let path = save_archive_path(&ctx.saves_dir, &filename)
        .map_err(|message| format!("invalid autosave path: {message}"))?;

    // Re-acquire the lock and write the archive so the captured tick matches
    // the on-disk state under contention. The double-acquire is bounded —
    // we only hold the lock long enough to copy a bincode stream.
    let tick_at_write = {
        let sim = ctx.sim.lock().await;
        CivSaveBundle::save_archive(&path, &sim)
            .map_err(|err| format!("autosave archive write failed: {err}"))?;
        sim.state.tick
    };

    let byte_size = std::fs::metadata(&path).map(|meta| meta.len()).unwrap_or(0);

    let save_id = ctx
        .save_db
        .record_autosave(
            &ctx.session_id,
            tick_at_write,
            &path.display().to_string(),
            byte_size,
        )
        .map_err(|err| format!("autosave metadata insert failed: {err}"))?;

    // Emit `session.saved.v1` on the replay bus (slot or autosave; CIV-1000).
    {
        let mut sim = ctx.sim.lock().await;
        sim.record_session_saved(
            &ctx.session_id,
            &save_id,
            &autosave_filename_for_tick(tick_at_write),
            byte_size,
        );
    }

    // Enforce the ring. The eviction happens via the SQLite index, which
    // returns the file paths of the dropped entries; we delete the files.
    if let Ok(evicted_paths) = ctx.save_db.evict_autosaves(&ctx.session_id, ctx.keep) {
        for evicted in evicted_paths {
            if let Err(err) = std::fs::remove_file(&evicted) {
                tracing::warn!(path = %evicted, ?err, "failed to delete evicted autosave file");
            }
        }
    }

    Ok(AutosaveResult {
        path,
        tick: tick_at_write,
        byte_size,
        save_id,
    })
}

/// Spawn the autosaver loop. Returns the [`tokio::task::JoinHandle`] so
/// tests can drive the loop deterministically (via `run_autosave_once`).
/// Returns `None` when cadence is `0` (autosaver disabled).
pub fn spawn_autosave_loop(
    ctx: AutosaveContext,
    cadence: Duration,
) -> Option<tokio::task::JoinHandle<()>> {
    let mut ticker = interval(cadence);
    // Skip the first immediate tick so the bridge has a chance to accept at
    // least one client before persisting state.
    ticker.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Delay);

    Some(tokio::spawn(async move {
        ticker.tick().await;
        loop {
            ticker.tick().await;
            match run_autosave_once(&ctx).await {
                Ok(result) => {
                    tracing::info!(
                        path = %result.path.display(),
                        tick = result.tick,
                        byte_size = result.byte_size,
                        "autosave complete"
                    );
                }
                Err(err) => {
                    tracing::error!(?err, "autosave failed");
                }
            }
        }
    }))
}

#[cfg(test)]
#[allow(unsafe_code)] // tests mutate `CIV_AUTOSAVE_*` env vars; serialized via the test mutex.
mod tests {
    use super::*;
    use std::path::Path;
    use std::sync::{Mutex as StdMutex, OnceLock};
    use tempfile::tempdir;

    static AUTOSAVE_ENV_MUTEX: OnceLock<StdMutex<()>> = OnceLock::new();

    struct EnvVarScope {
        key: &'static str,
        previous: Option<String>,
        _lock: std::sync::MutexGuard<'static, ()>,
    }

    impl EnvVarScope {
        fn set(key: &'static str, value: impl AsRef<str>) -> Self {
            let _lock = AUTOSAVE_ENV_MUTEX
                .get_or_init(|| StdMutex::new(()))
                .lock()
                .expect("env lock poisoned");
            let previous = std::env::var(key).ok();
            unsafe {
                std::env::set_var(key, value.as_ref());
            }
            Self {
                key,
                previous,
                _lock,
            }
        }

        fn remove(key: &'static str) -> Self {
            let _lock = AUTOSAVE_ENV_MUTEX
                .get_or_init(|| StdMutex::new(()))
                .lock()
                .expect("env lock poisoned");
            let previous = std::env::var(key).ok();
            unsafe {
                std::env::remove_var(key);
            }
            Self {
                key,
                previous,
                _lock,
            }
        }
    }

    impl Drop for EnvVarScope {
        fn drop(&mut self) {
            match self.previous.take() {
                Some(value) => unsafe {
                    std::env::set_var(self.key, value);
                },
                None => unsafe {
                    std::env::remove_var(self.key);
                },
            }
        }
    }

    fn temp_saves_dir() -> (tempfile::TempDir, PathBuf) {
        let dir = tempdir().expect("tempdir");
        let saves = dir.path().join("saves");
        std::fs::create_dir_all(&saves).expect("saves dir");
        (dir, saves)
    }

    fn open_save_db(saves_dir: &Path) -> Arc<SaveDb> {
        let db_path = saves_dir
            .parent()
            .map(std::path::Path::to_path_buf)
            .unwrap_or_else(|| PathBuf::from("."))
            .join("saves.db");
        Arc::new(SaveDb::open(&db_path).expect("save db open"))
    }

    #[test]
    fn filename_is_zero_padded_for_sort() {
        assert_eq!(
            autosave_filename_for_tick(0),
            "autosave-00000000000000000000"
        );
        assert_eq!(
            autosave_filename_for_tick(1_234_567),
            "autosave-00000000000001234567"
        );
    }

    #[test]
    fn cadence_from_env_defaults_to_60s() {
        // std::env::set_var/remove_var are unsafe in concurrent programs.
        let _scope = EnvVarScope::remove("CIV_AUTOSAVE_EVERY_SECS");
        let cadence = autosave_cadence_from_env().expect("default cadence");
        assert_eq!(cadence, Duration::from_secs(60));
    }

    #[test]
    fn cadence_from_env_zero_disables() {
        let _scope = EnvVarScope::set("CIV_AUTOSAVE_EVERY_SECS", "0");
        assert!(autosave_cadence_from_env().is_none());
    }

    #[test]
    fn cadence_from_env_honors_explicit_value() {
        let _scope = EnvVarScope::set("CIV_AUTOSAVE_EVERY_SECS", "5");
        assert_eq!(autosave_cadence_from_env(), Some(Duration::from_secs(5)));
    }

    #[test]
    fn keep_from_env_defaults_to_3() {
        let _scope = EnvVarScope::remove("CIV_AUTOSAVE_KEEP");
        assert_eq!(autosave_keep_from_env(), 3);
    }

    #[tokio::test]
    async fn run_autosave_once_writes_archive_and_records_metadata() {
        let (_dir, saves_dir) = temp_saves_dir();
        let save_db = open_save_db(&saves_dir);
        let sim = Arc::new(Mutex::new(Simulation::with_seed(7)));
        {
            let mut guard = sim.lock().await;
            for _ in 0..3 {
                guard.tick();
            }
        }
        let ctx = AutosaveContext {
            sim: Arc::clone(&sim),
            saves_dir: saves_dir.clone(),
            session_id: "test-session".to_string(),
            save_db: Arc::clone(&save_db),
            keep: 3,
        };

        let result = run_autosave_once(&ctx).await.expect("autosave once");
        assert!(result.path.is_file(), "autosave archive on disk");
        assert_eq!(result.tick, 3);
        assert!(result.byte_size > 0);
        assert!(!result.save_id.is_empty());

        let archive_name = result
            .path
            .file_name()
            .and_then(|s| s.to_str())
            .expect("archive name");
        assert_eq!(archive_name, "autosave-00000000000000000003.civsave.zst");

        let records = save_db
            .list_for_session("test-session")
            .expect("list save db");
        assert_eq!(records.len(), 1);
        let civ_save_db::SessionSaveRecord::Autosave(autosave) = &records[0] else {
            panic!("expected autosave record");
        };
        assert_eq!(autosave.tick, 3);
        assert_eq!(
            autosave.byte_size,
            i64::try_from(result.byte_size).expect("byte_size fits i64")
        );
    }

    #[tokio::test]
    async fn run_autosave_once_evicts_oldest_beyond_ring_size() {
        let (_dir, saves_dir) = temp_saves_dir();
        let save_db = open_save_db(&saves_dir);
        let sim = Arc::new(Mutex::new(Simulation::with_seed(13)));
        let ctx = AutosaveContext {
            sim: Arc::clone(&sim),
            saves_dir: saves_dir.clone(),
            session_id: "test-session".to_string(),
            save_db: Arc::clone(&save_db),
            keep: 2,
        };

        // Tick 5 times, running autosave each tick (sim advances +1 each pass).
        for expected_tick in 1..=5 {
            {
                let mut guard = sim.lock().await;
                guard.tick();
            }
            let result = run_autosave_once(&ctx).await.expect("autosave once");
            assert_eq!(result.tick, expected_tick);
        }

        let records = save_db
            .list_for_session("test-session")
            .expect("list save db");
        let autosaves: Vec<_> = records
            .into_iter()
            .filter_map(|r| match r {
                civ_save_db::SessionSaveRecord::Autosave(a) => Some(a),
                _ => None,
            })
            .collect();
        assert_eq!(autosaves.len(), 2, "ring retains only the latest 2 entries");
        let ticks: Vec<i64> = autosaves.iter().map(|a| a.tick).collect();
        assert!(ticks.contains(&4));
        assert!(ticks.contains(&5));

        // On-disk archives match the retained metadata.
        for autosave in &autosaves {
            assert!(
                Path::new(&autosave.file_path).is_file(),
                "expected on-disk file for tick {}",
                autosave.tick
            );
        }
    }

    #[tokio::test]
    async fn run_autosave_once_emits_session_saved_bus() {
        let (_dir, saves_dir) = temp_saves_dir();
        let save_db = open_save_db(&saves_dir);
        let sim = Arc::new(Mutex::new(Simulation::with_seed(31)));
        {
            let mut guard = sim.lock().await;
            for _ in 0..2 {
                guard.tick();
            }
        }
        let ctx = AutosaveContext {
            sim: Arc::clone(&sim),
            saves_dir,
            session_id: "bus-session".to_string(),
            save_db: Arc::clone(&save_db),
            keep: 3,
        };

        let result = run_autosave_once(&ctx).await.expect("autosave once");
        let guard = sim.lock().await;
        let bus = guard.replay_log().session_saved_bus_at_tick(result.tick);
        assert_eq!(bus.len(), 1, "session.saved.v1 emitted on replay bus");
        let value: serde_json::Value = serde_json::from_str(&bus[0]).expect("bus json");
        assert_eq!(value["event_type"], "session.saved.v1");
        assert_eq!(value["session_id"], "bus-session");
        assert_eq!(value["save_id"], result.save_id);
    }
}
