//! Save/load API handlers and helpers.

use std::path::{Path, PathBuf};

use axum::{extract::State, http::StatusCode, response::Json};
use civ_engine::{CivSaveBundle, Simulation};
use civ_save_db::format_session_saved_event_json;
use tracing::{info, warn};

use crate::app::{
    AppState, ControlOk, LoadResponse, SaveListEntry, SaveReq, SaveResponse, SlotReq,
    AUTOSAVE_RING_MAX, PRODUCTION_SLOTS,
};

pub(crate) fn sanitize_save_filename(filename: &str) -> Result<String, String> {
    let trimmed = filename.trim();
    if trimmed.is_empty() {
        return Err("filename cannot be empty".into());
    }
    if trimmed.contains('/') || trimmed.contains('\\') || trimmed.contains("..") {
        return Err("filename must be a simple name".into());
    }
    Ok(trimmed
        .trim_end_matches(".civreplay")
        .trim_end_matches(".civsave.zst")
        .trim_end_matches(".civsave")
        .to_string())
}

pub(crate) fn save_path(dir: &Path, filename: &str) -> Result<PathBuf, String> {
    let name = sanitize_save_filename(filename)?;
    Ok(dir.join(format!("{name}.civsave.zst")))
}

pub(crate) fn record_save_metadata(
    state: &AppState,
    sim: &mut Simulation,
    filename: &str,
    path: &Path,
    tick: u64,
) {
    let byte_size = std::fs::metadata(path).map(|meta| meta.len()).unwrap_or(0);
    let file_path = path.display().to_string();
    let result = if is_autosave_name(filename) {
        state
            .save_db
            .record_autosave(&state.session_id, tick, &file_path, byte_size)
            .map(|save_id| (save_id, filename.to_string()))
    } else if PRODUCTION_SLOTS.contains(&filename) {
        state
            .save_db
            .record_slot_save(&state.session_id, filename, tick, &file_path, byte_size)
            .map(|save_id| (save_id, filename.to_string()))
    } else {
        return;
    };
    match result {
        Ok((save_id, slot)) => {
            sim.record_session_saved(&state.session_id, &save_id, &slot, byte_size);
            let event_json = format_session_saved_event_json(
                &state.session_id,
                &save_id,
                &slot,
                tick,
                byte_size,
            );
            info!(%event_json, "session.saved.v1 on replay bus");
            if is_autosave_name(filename) {
                match state.save_db.evict_autosaves(
                    &state.session_id,
                    u32::try_from(AUTOSAVE_RING_MAX).unwrap_or(u32::MAX),
                ) {
                    Ok(evicted_paths) => {
                        for evicted in evicted_paths {
                            if let Err(err) = std::fs::remove_file(&evicted) {
                                warn!(path = %evicted, ?err, "failed to remove evicted autosave file");
                            }
                        }
                    }
                    Err(err) => warn!(?err, "failed to evict autosaves from save db"),
                }
            }
        }
        Err(err) => warn!(?err, filename, "failed to record save metadata"),
    }
}

pub(crate) fn dir_size_bytes(dir: &Path) -> u64 {
    let mut total = 0u64;
    if let Ok(read) = std::fs::read_dir(dir) {
        for entry in read.flatten() {
            let path = entry.path();
            if path.is_dir() {
                total = total.saturating_add(dir_size_bytes(&path));
            } else if let Ok(meta) = entry.metadata() {
                total = total.saturating_add(meta.len());
            }
        }
    }
    total
}

pub(crate) fn legacy_replay_path(dir: &Path, filename: &str) -> Result<PathBuf, String> {
    let name = sanitize_save_filename(filename)?;
    Ok(dir.join(format!("{name}.civreplay")))
}

pub(crate) fn validate_production_slot(slot: &str) -> Result<(), String> {
    if PRODUCTION_SLOTS.contains(&slot) {
        Ok(())
    } else {
        Err(format!(
            "invalid slot {slot:?}; expected one of {}",
            PRODUCTION_SLOTS.join(", ")
        ))
    }
}

pub(crate) fn save_type_for_name(name: &str) -> &'static str {
    if PRODUCTION_SLOTS.contains(&name) {
        "slot"
    } else if name == "autosave" || name.starts_with("autosave-") {
        "auto"
    } else {
        "manual"
    }
}

pub(crate) fn is_autosave_name(name: &str) -> bool {
    name == "autosave" || name.starts_with("autosave-")
}

pub(crate) fn enforce_autosave_ring(dir: &Path) {
    let mut autosaves = Vec::new();
    let Ok(read_dir) = std::fs::read_dir(dir) else {
        return;
    };
    for entry in read_dir.flatten() {
        let path = entry.path();
        if !CivSaveBundle::is_save_archive(&path) {
            continue;
        }
        let Some(stem) = path
            .file_name()
            .and_then(|s| s.to_str())
            .map(|s| s.trim_end_matches(".civsave.zst"))
        else {
            continue;
        };
        if !stem.starts_with("autosave") {
            continue;
        }
        let mtime = entry
            .metadata()
            .ok()
            .and_then(|meta| meta.modified().ok())
            .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
            .map(|d| d.as_secs())
            .unwrap_or(0);
        autosaves.push((path, mtime));
    }
    if autosaves.len() <= AUTOSAVE_RING_MAX {
        return;
    }
    autosaves.sort_by(|a, b| b.1.cmp(&a.1).then_with(|| b.0.cmp(&a.0)));
    for (path, _) in autosaves.into_iter().skip(AUTOSAVE_RING_MAX) {
        if let Err(err) = std::fs::remove_file(&path) {
            warn!(?path, ?err, "failed to evict autosave from ring");
        }
    }
}

pub(crate) async fn save_handler(
    State(state): State<AppState>,
    Json(req): Json<SaveReq>,
) -> Result<Json<SaveResponse>, (StatusCode, Json<ControlOk>)> {
    let path = save_path(&state.saves_dir, &req.filename).map_err(|message| {
        (
            StatusCode::BAD_REQUEST,
            Json(ControlOk {
                ok: false,
                message: Some(message),
            }),
        )
    })?;
    let mut sim = state.sim.lock().await;
    CivSaveBundle::save_archive(&path, &sim).map_err(|err| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ControlOk {
                ok: false,
                message: Some(err.to_string()),
            }),
        )
    })?;
    let tick = sim.state.tick;
    let filename = sanitize_save_filename(&req.filename).map_err(|message| {
        (
            StatusCode::BAD_REQUEST,
            Json(ControlOk {
                ok: false,
                message: Some(message),
            }),
        )
    })?;
    if is_autosave_name(&filename) {
        enforce_autosave_ring(state.saves_dir.as_ref());
    }
    record_save_metadata(&state, &mut sim, &filename, &path, tick);
    Ok(Json(SaveResponse {
        ok: true,
        path: path.display().to_string(),
        tick,
    }))
}

pub(crate) async fn save_slot_handler(
    State(state): State<AppState>,
    Json(req): Json<SlotReq>,
) -> Result<Json<SaveResponse>, (StatusCode, Json<ControlOk>)> {
    validate_production_slot(&req.slot).map_err(|message| {
        (
            StatusCode::BAD_REQUEST,
            Json(ControlOk {
                ok: false,
                message: Some(message),
            }),
        )
    })?;
    save_handler(State(state), Json(SaveReq { filename: req.slot })).await
}

pub(crate) async fn load_slot_handler(
    State(state): State<AppState>,
    Json(req): Json<SlotReq>,
) -> Result<Json<LoadResponse>, (StatusCode, Json<ControlOk>)> {
    validate_production_slot(&req.slot).map_err(|message| {
        (
            StatusCode::BAD_REQUEST,
            Json(ControlOk {
                ok: false,
                message: Some(message),
            }),
        )
    })?;
    load_handler(State(state), Json(SaveReq { filename: req.slot })).await
}

pub(crate) async fn load_handler(
    State(state): State<AppState>,
    Json(req): Json<SaveReq>,
) -> Result<Json<LoadResponse>, (StatusCode, Json<ControlOk>)> {
    let archive_path = save_path(&state.saves_dir, &req.filename).map_err(|message| {
        (
            StatusCode::BAD_REQUEST,
            Json(ControlOk {
                ok: false,
                message: Some(message),
            }),
        )
    })?;
    let folder_path = state.saves_dir.join(format!(
        "{}.civsave",
        sanitize_save_filename(&req.filename).map_err(|message| {
            (
                StatusCode::BAD_REQUEST,
                Json(ControlOk {
                    ok: false,
                    message: Some(message),
                }),
            )
        })?
    ));
    let path = if CivSaveBundle::is_save_archive(&archive_path) {
        archive_path
    } else if CivSaveBundle::is_save_dir(&folder_path) {
        folder_path
    } else {
        archive_path
    };
    let mut sim = state.sim.lock().await;
    let loaded = if CivSaveBundle::is_save_archive(&path) || CivSaveBundle::is_save_dir(&path) {
        CivSaveBundle::load(&path).map_err(|err| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ControlOk {
                    ok: false,
                    message: Some(err.to_string()),
                }),
            )
        })?
    } else {
        let replay_path =
            legacy_replay_path(state.saves_dir.as_ref(), &req.filename).map_err(|message| {
                (
                    StatusCode::BAD_REQUEST,
                    Json(ControlOk {
                        ok: false,
                        message: Some(message),
                    }),
                )
            })?;
        Simulation::load_replay_from_file(&replay_path).map_err(|err| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ControlOk {
                    ok: false,
                    message: Some(err.to_string()),
                }),
            )
        })?
    };
    *sim = loaded;
    let tick = sim.state.tick;
    Ok(Json(LoadResponse { ok: true, tick }))
}

pub(crate) async fn list_saves_handler(
    State(state): State<AppState>,
) -> Result<Json<Vec<SaveListEntry>>, (StatusCode, Json<ControlOk>)> {
    let mut entries = Vec::new();
    let db_by_name = match state.save_db.list_for_session(&state.session_id) {
        Ok(records) => {
            let mut map = std::collections::HashMap::new();
            for record in records {
                match record {
                    civ_save_db::SessionSaveRecord::Slot(slot) => {
                        map.insert(
                            slot.slot_name.clone(),
                            (Some(slot.id), Some(slot.tick as u64), Some(slot.created_at)),
                        );
                    }
                    civ_save_db::SessionSaveRecord::Autosave(autosave) => {
                        let name = Path::new(&autosave.file_path)
                            .file_name()
                            .and_then(|s| s.to_str())
                            .map(|s| s.trim_end_matches(".civsave.zst").to_string())
                            .unwrap_or_else(|| format!("autosave-{}", autosave.tick));
                        map.insert(
                            name,
                            (
                                Some(autosave.id),
                                Some(autosave.tick as u64),
                                Some(autosave.created_at),
                            ),
                        );
                    }
                }
            }
            Some(map)
        }
        Err(err) => {
            warn!(?err, "failed to list save metadata from db");
            None
        }
    };
    let dir = state.saves_dir.as_ref();
    let read_dir = std::fs::read_dir(dir).map_err(|err| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ControlOk {
                ok: false,
                message: Some(err.to_string()),
            }),
        )
    })?;
    for entry in read_dir.flatten() {
        let path = entry.path();
        let name = if CivSaveBundle::is_save_archive(&path) {
            path.file_name()
                .and_then(|s| s.to_str())
                .map(|s| s.trim_end_matches(".civsave.zst").to_string())
        } else if CivSaveBundle::is_save_dir(&path) {
            path.file_stem()
                .and_then(|s| s.to_str())
                .map(|s| s.trim_end_matches(".civsave").to_string())
        } else if path.extension().and_then(|s| s.to_str()) == Some("civreplay") {
            path.file_stem()
                .and_then(|s| s.to_str())
                .map(|s| s.to_string())
        } else {
            continue;
        };
        let Some(name) = name else {
            continue;
        };
        let meta = match entry.metadata() {
            Ok(meta) => meta,
            Err(_) => continue,
        };
        let modified = meta
            .modified()
            .ok()
            .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
            .map(|d| d.as_secs());
        let size_bytes = if path.is_dir() {
            dir_size_bytes(&path)
        } else {
            meta.len()
        };
        entries.push(SaveListEntry {
            name: name.clone(),
            size_bytes,
            modified,
            save_type: save_type_for_name(&name),
            session_id: db_by_name
                .as_ref()
                .and_then(|map| map.get(&name))
                .map(|_| state.session_id.clone()),
            save_id: db_by_name
                .as_ref()
                .and_then(|map| map.get(&name))
                .and_then(|(id, _, _)| id.clone()),
            tick: db_by_name
                .as_ref()
                .and_then(|map| map.get(&name))
                .and_then(|(_, tick, _)| *tick),
            created_at: db_by_name
                .as_ref()
                .and_then(|map| map.get(&name))
                .and_then(|(_, _, created_at)| created_at.clone()),
        });
    }
    entries.sort_by(|a, b| a.name.cmp(&b.name));
    Ok(Json(entries))
}
