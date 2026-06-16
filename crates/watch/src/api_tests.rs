//! Integration tests for civ-watch HTTP routes.

use std::path::PathBuf;
use std::sync::{
    atomic::{AtomicU16, AtomicU8},
    Arc,
};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use axum::{
    body::Body,
    http::{header, Request, StatusCode},
    routing::get,
    Router,
};
use civ_engine::{CivSaveBundle, Simulation};
use civ_mod_host::CIVMOD_MANIFEST_NAME;
use civ_save_db::SaveDb;
use tokio::sync::{broadcast, Mutex, RwLock};
use tower::ServiceExt;
use tower_http::cors::CorsLayer;

use crate::app::{
    default_law_db, AppState, RemoteModRegistry, RemoteModRegistryEntry, Snapshot, TerrainCache,
    AUTOSAVE_RING_MAX, REMOTE_FETCH_TIMEOUT, REMOTE_MOD_ARCHIVE_NAME,
};
use crate::mods_api::{
    format_remote_mod_validation_error, persist_remote_mod_cache, read_remote_mod_meta,
    remote_mod_cache_dir, repo_root, resolve_remote_cache_id, scan_mod_catalog,
    scan_remote_mod_cache, validate_remote_fetch_against_registry, validate_remote_fetch_url,
    validate_remote_mod_bytes,
};
use crate::server::build_api_router;
use crate::sim_worker::simulation_worker;
use crate::snapshot::make_snapshot;
use crate::terrain::{self, Terrain};

fn test_state() -> AppState {
    let (tx, _) = broadcast::channel::<Snapshot>(64);
    let sim = Arc::new(Mutex::new(Simulation::with_seed(42)));
    let terrain = Terrain::generate(42);
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("time")
        .as_nanos();
    let saves_dir = std::env::temp_dir().join(format!("civis-watch-test-{nanos}"));
    std::fs::create_dir_all(&saves_dir).expect("saves dir");
    let save_db_path = saves_dir.join("saves.db");
    let save_db = Arc::new(SaveDb::open(&save_db_path).expect("open save db"));
    let mods_dir = repo_root().join("mods");
    AppState {
        latest: Arc::new(RwLock::new(None)),
        tx,
        terrain: Arc::new(terrain.clone()),
        terrain_cache: TerrainCache::from_terrain(&terrain),
        laws: Arc::new(default_law_db()),
        sim,
        military: Arc::new(Mutex::new(Vec::new())),
        target_era: Arc::new(AtomicU16::new(0)),
        speed: Arc::new(AtomicU8::new(1)),
        saves_dir: Arc::new(saves_dir),
        mods_dir: Arc::new(mods_dir),
        session_id: "test-session".to_string(),
        save_db,
        http: reqwest::Client::builder()
            .timeout(REMOTE_FETCH_TIMEOUT)
            .redirect(reqwest::redirect::Policy::limited(5))
            .build()
            .expect("reqwest client"),
    }
}

fn test_app() -> Router {
    test_app_with_state(test_state())
}

fn test_app_with_state(state: AppState) -> Router {
    build_api_router()
        .with_state(state)
        .layer(CorsLayer::permissive())
}

async fn body_json(response: axum::response::Response) -> serde_json::Value {
    let bytes = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .expect("response body");
    serde_json::from_slice(&bytes).expect("json body")
}

#[tokio::test]
async fn get_terrain_returns_heightmap_json() {
    let app = test_app();
    let response = app
        .oneshot(
            Request::builder()
                .uri("/terrain")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    assert_eq!(
        response
            .headers()
            .get(header::ETAG)
            .expect("etag")
            .to_str()
            .unwrap(),
        format!("\"{:016x}\"", Terrain::generate(42).heights_fingerprint())
    );
    let json = body_json(response).await;
    assert_eq!(json["size"], terrain::SIZE);
    assert_eq!(
        json["heights"].as_array().expect("heights array").len(),
        terrain::SIZE * terrain::SIZE,
    );
    assert_eq!(
        json["biomes"].as_array().expect("biomes array").len(),
        terrain::SIZE * terrain::SIZE,
    );
}

#[tokio::test]
async fn get_terrain_returns_304_when_etag_matches() {
    let app = test_app();
    let first = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/terrain")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    let etag = first
        .headers()
        .get(header::ETAG)
        .expect("etag")
        .to_str()
        .unwrap()
        .to_owned();

    let second = app
        .oneshot(
            Request::builder()
                .uri("/terrain")
                .header(header::IF_NONE_MATCH, etag)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(second.status(), StatusCode::NOT_MODIFIED);
    assert!(axum::body::to_bytes(second.into_body(), usize::MAX)
        .await
        .unwrap()
        .is_empty());
}

#[tokio::test]
async fn get_snapshot_returns_null_before_first_tick() {
    let app = test_app();
    let response = app
        .oneshot(
            Request::builder()
                .uri("/snapshot")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    let json = body_json(response).await;
    assert!(json.is_null());
}

#[tokio::test]
async fn make_snapshot_includes_life_sim_state_parity() {
    let state = test_state();
    {
        let mut sim = state.sim.lock().await;
        sim.tick();
    }

    let sim = state.sim.lock().await;
    let snapshot = make_snapshot(
        &sim,
        &[],
        &[],
        &crate::app::TradeTickSummary::default(),
        state.speed.load(std::sync::atomic::Ordering::Relaxed),
        &state.laws,
        state.target_era.load(std::sync::atomic::Ordering::Relaxed),
    );

    assert_eq!(snapshot.population, sim.state.population);
    assert_eq!(snapshot.births_this_tick, sim.last_births().len() as u32);
    assert_eq!(snapshot.deaths_this_tick, sim.last_deaths().len() as u32);
    assert_eq!(snapshot.settlement_count, sim.settlement_count());
    assert_eq!(snapshot.cluster_stocks, sim.cluster_stocks().clone());
}

#[tokio::test]
async fn post_control_speed_rejects_invalid_value() {
    let app = test_app();
    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/control/speed")
                .header("content-type", "application/json")
                .body(Body::from(r#"{"speed":3}"#))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    let json = body_json(response).await;
    assert_eq!(json["ok"], false);
}

#[tokio::test]
async fn post_control_speed_accepts_valid_value() {
    let app = test_app();
    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/control/speed")
                .header("content-type", "application/json")
                .body(Body::from(r#"{"speed":2}"#))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    let json = body_json(response).await;
    assert_eq!(json["ok"], true);
}

#[tokio::test]
async fn post_control_place_voxel_returns_ok() {
    let app = test_app();
    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/control/place_voxel")
                .header("content-type", "application/json")
                .body(Body::from(r#"{"x":0,"y":0,"z":0,"material":2}"#))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    let json = body_json(response).await;
    assert_eq!(json["ok"], true);
}

#[tokio::test]
async fn fr_save_004_post_control_save_and_load_round_trip() {
    let app = test_app();
    let save_name = "unit-test-save";

    let save_response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/control/save")
                .header("content-type", "application/json")
                .body(Body::from(format!(r#"{{"filename":"{save_name}"}}"#)))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(save_response.status(), StatusCode::OK);
    let save_json = body_json(save_response).await;
    assert_eq!(save_json["ok"], true);

    let list_response = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/control/saves")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(list_response.status(), StatusCode::OK);
    let list_json = body_json(list_response).await;
    assert!(list_json
        .as_array()
        .expect("save list array")
        .iter()
        .any(|entry| entry["name"] == save_name));

    let load_response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/control/load")
                .header("content-type", "application/json")
                .body(Body::from(format!(r#"{{"filename":"{save_name}"}}"#)))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(load_response.status(), StatusCode::OK);
    let load_json = body_json(load_response).await;
    assert_eq!(load_json["ok"], true);
}

fn touch_mtime(path: &std::path::Path, secs: u64) {
    use std::time::{Duration, UNIX_EPOCH};
    let time = UNIX_EPOCH + Duration::from_secs(secs);
    std::fs::OpenOptions::new()
        .write(true)
        .open(path)
        .expect("open for mtime")
        .set_modified(time)
        .expect("set mtime");
}

fn autosave_archive_count(dir: &std::path::Path) -> usize {
    std::fs::read_dir(dir)
        .into_iter()
        .flatten()
        .flatten()
        .filter(|entry| {
            let path = entry.path();
            CivSaveBundle::is_save_archive(&path)
                && path
                    .file_name()
                    .and_then(|s| s.to_str())
                    .is_some_and(|s| s.starts_with("autosave") && s.ends_with(".civsave.zst"))
        })
        .count()
}

#[tokio::test]
async fn fr_save_002_post_save_slot_round_trip() {
    let state = test_state();
    let app = test_app_with_state(state.clone());

    let save_response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/control/save/slot")
                .header("content-type", "application/json")
                .body(Body::from(r#"{"slot":"slot-1"}"#))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(save_response.status(), StatusCode::OK);
    let save_json = body_json(save_response).await;
    assert_eq!(save_json["ok"], true);
    assert!(save_json["path"]
        .as_str()
        .unwrap_or("")
        .ends_with("slot-1.civsave.zst"));

    {
        let sim = state.sim.lock().await;
        let buses = sim.replay_log().session_saved_bus_at_tick(sim.state.tick);
        assert_eq!(buses.len(), 1);
        let value: serde_json::Value =
            serde_json::from_str(&buses[0]).expect("session.saved bus json");
        assert_eq!(value["event_type"], "session.saved.v1");
        assert_eq!(value["slot"], "slot-1");
    }

    let list_response = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/control/saves")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(list_response.status(), StatusCode::OK);
    let list_json = body_json(list_response).await;
    let slot_entry = list_json
        .as_array()
        .expect("save list array")
        .iter()
        .find(|entry| entry["name"] == "slot-1")
        .expect("slot-1 listed");
    assert_eq!(slot_entry["save_type"], "slot");
    assert_eq!(slot_entry["session_id"], "test-session");
    assert!(slot_entry["save_id"]
        .as_str()
        .is_some_and(|id| !id.is_empty()));
    assert!(slot_entry["tick"].as_u64().is_some());

    let load_response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/control/load/slot")
                .header("content-type", "application/json")
                .body(Body::from(r#"{"slot":"slot-1"}"#))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(load_response.status(), StatusCode::OK);
    let load_json = body_json(load_response).await;
    assert_eq!(load_json["ok"], true);
}

#[tokio::test]
async fn fr_save_003_autosave_ring_evicts_oldest_beyond_max() {
    let state = test_state();
    let saves_dir = state.saves_dir.clone();
    let app = test_app_with_state(state.clone());

    {
        let sim = state.sim.lock().await;
        for index in 0..=AUTOSAVE_RING_MAX {
            let name = format!("autosave-ring-{index:02}");
            let path = saves_dir.join(format!("{name}.civsave.zst"));
            CivSaveBundle::save_archive(&path, &sim).expect("seed autosave");
            touch_mtime(
                &path,
                1_700_000_000 + u64::try_from(index).expect("index fits u64"),
            );
        }
    }

    assert_eq!(autosave_archive_count(&saves_dir), AUTOSAVE_RING_MAX + 1);

    let trigger = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/control/save")
                .header("content-type", "application/json")
                .body(Body::from(r#"{"filename":"autosave-ring-trigger"}"#))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(trigger.status(), StatusCode::OK);

    assert_eq!(autosave_archive_count(&saves_dir), AUTOSAVE_RING_MAX);
    assert!(!saves_dir.join("autosave-ring-00.civsave.zst").is_file());
    assert!(saves_dir
        .join("autosave-ring-trigger.civsave.zst")
        .is_file());
}

#[tokio::test]
async fn get_events_streams_snapshot_within_timeout() {
    use http_body_util::BodyExt;

    let state = test_state();
    tokio::spawn(simulation_worker(state.clone()));
    let app = test_app_with_state(state);

    let response = app
        .oneshot(
            Request::builder()
                .uri("/events")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    assert!(response
        .headers()
        .get(header::CONTENT_TYPE)
        .and_then(|v| v.to_str().ok())
        .is_some_and(|ct| ct.starts_with("text/event-stream")));

    let mut body = response.into_body();
    let mut buf = String::new();
    let deadline = tokio::time::Instant::now() + Duration::from_secs(2);
    while !buf.contains("event: snapshot") {
        let remaining = deadline.saturating_duration_since(tokio::time::Instant::now());
        let frame = tokio::time::timeout(remaining, body.frame())
            .await
            .expect("timed out waiting for SSE snapshot event")
            .expect("body frame")
            .expect("frame data");
        if let Ok(chunk) = frame.into_data() {
            buf.push_str(&String::from_utf8_lossy(&chunk));
        }
    }
    assert!(buf.contains("\"tick\""));
}

#[tokio::test]
async fn post_control_spawn_civilian_returns_ok() {
    let app = test_app();
    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/control/spawn_civilian")
                .header("content-type", "application/json")
                .body(Body::from(r#"{"x":0.5,"y":0.5,"faction":0}"#))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    let json = body_json(response).await;
    assert_eq!(json["ok"], true);
}

#[tokio::test]
async fn post_control_spawn_entity_hangar_returns_ok() {
    let app = test_app();
    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/control/spawn_entity")
                .header("content-type", "application/json")
                .body(Body::from(
                    r#"{"kind":"hangar","x":0.55,"y":0.45,"faction":0}"#,
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    let json = body_json(response).await;
    assert_eq!(json["ok"], true);
}

#[tokio::test]
async fn post_control_spawn_entity_vehicle_returns_ok() {
    let app = test_app();
    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/control/spawn_entity")
                .header("content-type", "application/json")
                .body(Body::from(
                    r#"{"kind":"vehicle","x":0.4,"y":0.6,"faction":1}"#,
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    let json = body_json(response).await;
    assert_eq!(json["ok"], true);
}

#[tokio::test]
async fn post_control_damage_returns_ok() {
    let app = test_app();
    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/control/damage")
                .header("content-type", "application/json")
                .body(Body::from(r#"{"x":0,"y":0,"z":0,"radius":2,"energy":100}"#))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    let json = body_json(response).await;
    assert_eq!(json["ok"], true);
}

#[tokio::test]
async fn get_mods_catalog_lists_example_mods() {
    let app = test_app();
    let response = app
        .oneshot(
            Request::builder()
                .uri("/control/mods/catalog")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    let json = body_json(response).await;
    let entries = json.as_array().expect("catalog array");
    assert!(entries
        .iter()
        .any(|entry| entry["source"] == "mods/example-policy"));
    assert!(entries
        .iter()
        .any(|entry| entry["source"] == "mods/example-economic"));
}

#[tokio::test]
async fn post_mods_install_rejects_unknown_source() {
    let app = test_app();
    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/control/mods/install")
                .header("content-type", "application/json")
                .body(Body::from(r#"{"source":"not-a-real-mod"}"#))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn post_mods_unload_removes_installed_mod() {
    let state = test_state();
    let app = test_app_with_state(state.clone());

    let install_response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/control/mods/install")
                .header("content-type", "application/json")
                .body(Body::from(r#"{"source":"mods/example-policy"}"#))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(install_response.status(), StatusCode::OK);

    let unload_response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/control/mods/unload")
                .header("content-type", "application/json")
                .body(Body::from(r#"{"mod_id":"example-policy"}"#))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(unload_response.status(), StatusCode::OK);

    let sim = state.sim.lock().await;
    assert!(sim
        .mod_browser_entries()
        .iter()
        .all(|entry| entry.id != "example-policy"));
}

#[tokio::test]
async fn post_mods_unload_rejects_unknown_mod() {
    let app = test_app();
    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/control/mods/unload")
                .header("content-type", "application/json")
                .body(Body::from(r#"{"mod_id":"not-loaded"}"#))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn post_mods_reload() {
    let state = test_state();
    let app = test_app_with_state(state.clone());

    let install_response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/control/mods/install")
                .header("content-type", "application/json")
                .body(Body::from(r#"{"source":"mods/example-policy"}"#))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(install_response.status(), StatusCode::OK);

    let reload_response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/control/mods/reload")
                .header("content-type", "application/json")
                .body(Body::from(r#"{"mod_id":"example-policy"}"#))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(reload_response.status(), StatusCode::OK);

    let sim = state.sim.lock().await;
    assert!(sim
        .mod_browser_entries()
        .iter()
        .any(|entry| entry.id == "example-policy"));
}

fn minimal_upload_civmod_bytes(mod_id: &str) -> Vec<u8> {
    use std::io::Write;
    use zip::write::SimpleFileOptions;
    use zip::ZipWriter;

    const WAT: &str = r#"
            (module
              (func (export "civlab_policy_tick") (result i32)
                i32.const 3)
            )
        "#;
    let wasm = wat::parse_str(WAT).expect("wat");
    let manifest = format!(
        r#"
[mod]
id = "{mod_id}"
name = "Upload Test"
version = "0.0.1"
api_version = "1"
mod_type = "policy"
author = "t"
description = "d"

[dependencies]
civlab-api = ">=1.0.0, <2.0.0"

[permissions]
write_policy = true
"#
    );
    let mut buffer = Vec::new();
    {
        let mut zip = ZipWriter::new(std::io::Cursor::new(&mut buffer));
        let options = SimpleFileOptions::default();
        zip.start_file(CIVMOD_MANIFEST_NAME, options)
            .expect("manifest");
        zip.write_all(manifest.as_bytes()).expect("write manifest");
        zip.start_file("mod.wasm", options).expect("wasm");
        zip.write_all(&wasm).expect("write wasm");
        zip.finish().expect("finish");
    }
    buffer
}

fn signed_upload_civmod_bytes(mod_id: &str) -> (Vec<u8>, String) {
    use ed25519_dalek::Signer;
    use rand::rngs::OsRng;
    use std::io::Write;
    use zip::write::SimpleFileOptions;
    use zip::ZipWriter;

    const WAT: &str = r#"
            (module
              (func (export "civlab_policy_tick") (result i32)
                i32.const 3)
            )
        "#;
    let wasm = wat::parse_str(WAT).expect("wat");
    let signing_key = ed25519_dalek::SigningKey::generate(&mut OsRng);
    let signature = signing_key.sign(&wasm);
    let pk_hex: String = signing_key
        .verifying_key()
        .as_bytes()
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect();
    let manifest = format!(
        r#"
[mod]
id = "{mod_id}"
name = "Signed Upload Test"
version = "0.0.1"
api_version = "1"
mod_type = "policy"
author = "t"
description = "d"
author_pubkey_hex = "{pk_hex}"

[dependencies]
civlab-api = ">=1.0.0, <2.0.0"

[permissions]
write_policy = true
"#
    );
    let mut buffer = Vec::new();
    {
        let mut zip = ZipWriter::new(std::io::Cursor::new(&mut buffer));
        let options = SimpleFileOptions::default();
        zip.start_file(CIVMOD_MANIFEST_NAME, options)
            .expect("manifest");
        zip.write_all(manifest.as_bytes()).expect("write manifest");
        zip.start_file("mod.wasm", options).expect("wasm");
        zip.write_all(&wasm).expect("write wasm");
        zip.start_file("mod.wasm.sig", options).expect("sig");
        zip.write_all(signature.to_bytes().as_slice())
            .expect("write sig");
        zip.finish().expect("finish");
    }
    (buffer, pk_hex)
}

#[test]
fn format_remote_mod_validation_error_prefixes_signature_failures() {
    let err = civ_mod_host::ManifestError::Validation {
        path: PathBuf::from("mod.civmod"),
        message: "missing mod.wasm.sig for signed mod".to_owned(),
    };
    let formatted = format_remote_mod_validation_error(err);
    assert!(
        formatted.contains("signature verification failed"),
        "unexpected: {formatted}"
    );
}

#[test]
fn validate_remote_mod_bytes_rejects_unsigned_wasm_with_pubkey() {
    use std::io::Write;
    use zip::write::SimpleFileOptions;
    use zip::ZipWriter;

    let dir = tempfile::tempdir().expect("tempdir");
    let scratch = dir.path().join("mod.civmod");
    const WAT: &str = r#"
            (module
              (func (export "civlab_policy_tick") (result i32)
                i32.const 3)
            )
        "#;
    let wasm = wat::parse_str(WAT).expect("wat");
    let manifest = r#"
[mod]
id = "unsigned-with-pk"
name = "Upload Test"
version = "0.0.1"
api_version = "1"
mod_type = "policy"
author = "t"
description = "d"
author_pubkey_hex = "00112233445566778899aabbccddeeff00112233445566778899aabbccddeeff"

[dependencies]
civlab-api = ">=1.0.0, <2.0.0"

[permissions]
write_policy = true
"#;
    let mut buffer = Vec::new();
    {
        let mut zip = ZipWriter::new(std::io::Cursor::new(&mut buffer));
        let options = SimpleFileOptions::default();
        zip.start_file(CIVMOD_MANIFEST_NAME, options)
            .expect("manifest");
        zip.write_all(manifest.as_bytes()).expect("write manifest");
        zip.start_file("mod.wasm", options).expect("wasm");
        zip.write_all(&wasm).expect("write wasm");
        zip.finish().expect("finish");
    }
    let err = validate_remote_mod_bytes(&buffer, &scratch, None).expect_err("must fail");
    assert!(
        err.contains("signature"),
        "expected signature error, got: {err}"
    );
}

#[tokio::test]
async fn persist_remote_mod_cache_marks_signed_mods() {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("time")
        .as_nanos();
    let cache_id = format!("remote-signed-test-{nanos}");
    let mods_dir = repo_root().join("mods");
    let cache_dir = remote_mod_cache_dir(&mods_dir, &cache_id);
    let (bytes, pk_hex) = signed_upload_civmod_bytes(&cache_id);
    let url = "https://example.com/signed.civmod";
    let (archive_path, source) =
        persist_remote_mod_cache(&mods_dir, &cache_id, url, &bytes, None).expect("persist");
    assert!(archive_path.is_file());

    let meta = read_remote_mod_meta(&cache_dir).expect("meta");
    assert!(meta.signed);
    assert_eq!(meta.author_pubkey_hex.as_deref(), Some(pk_hex.as_str()));

    let remote_list = scan_remote_mod_cache(&mods_dir);
    assert!(remote_list.iter().any(|entry| {
        entry.id == cache_id
            && entry.signed
            && entry.author_pubkey_hex.as_deref() == Some(pk_hex.as_str())
    }));

    let catalog = scan_mod_catalog(&mods_dir, &std::collections::HashSet::new());
    assert!(catalog.iter().any(|entry| {
        entry.source == source
            && entry.signed
            && entry.author_pubkey_hex.as_deref() == Some(pk_hex.as_str())
    }));

    let _ = std::fs::remove_dir_all(cache_dir);
}

#[tokio::test]
async fn post_mods_upload_round_trip_catalog_and_install() {
    use base64::Engine as _;
    use std::time::{SystemTime, UNIX_EPOCH};

    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("time")
        .as_nanos();
    let mod_id = format!("upload-test-{nanos}");
    let filename = format!("upload-{nanos}.civmod");
    let civmod_bytes = minimal_upload_civmod_bytes(&mod_id);
    let data_base64 = base64::engine::general_purpose::STANDARD.encode(&civmod_bytes);
    let body = serde_json::json!({
        "filename": filename,
        "data_base64": data_base64,
    })
    .to_string();

    let app = test_app();
    let upload_response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/control/mods/upload")
                .header("content-type", "application/json")
                .body(Body::from(body))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(upload_response.status(), StatusCode::OK);
    let upload_json = body_json(upload_response).await;
    assert_eq!(upload_json["ok"], true);
    let source = upload_json["source"]
        .as_str()
        .expect("upload source")
        .to_owned();

    let catalog_response = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/control/mods/catalog")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(catalog_response.status(), StatusCode::OK);
    let catalog_json = body_json(catalog_response).await;
    assert!(catalog_json
        .as_array()
        .expect("catalog array")
        .iter()
        .any(|entry| entry["source"] == source && entry["id"] == mod_id));

    let install_response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/control/mods/install")
                .header("content-type", "application/json")
                .body(Body::from(format!(r#"{{"source":"{source}"}}"#)))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(install_response.status(), StatusCode::OK);
    let install_json = body_json(install_response).await;
    assert_eq!(install_json["ok"], true);

    let uploaded_path = repo_root().join(source.replace('/', std::path::MAIN_SEPARATOR_STR));
    let _ = std::fs::remove_file(uploaded_path);
}

#[tokio::test]
async fn post_mods_publish_round_trip() {
    use base64::Engine as _;
    use std::time::{SystemTime, UNIX_EPOCH};

    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("time")
        .as_nanos();
    let mod_id = format!("publish-test-{nanos}");
    let filename = format!("publish-{nanos}.civmod");
    let civmod_bytes = minimal_upload_civmod_bytes(&mod_id);
    let data_base64 = base64::engine::general_purpose::STANDARD.encode(&civmod_bytes);
    let upload_body = serde_json::json!({
        "filename": filename,
        "data_base64": data_base64,
    })
    .to_string();

    let app = test_app();
    let upload_response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/control/mods/upload")
                .header("content-type", "application/json")
                .body(Body::from(upload_body))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(upload_response.status(), StatusCode::OK);
    let upload_json = body_json(upload_response).await;
    let upload_source = upload_json["source"]
        .as_str()
        .expect("upload source")
        .to_owned();

    let publish_response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/control/mods/publish")
                .header("content-type", "application/json")
                .body(Body::from(format!(r#"{{"source":"{upload_source}"}}"#)))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(publish_response.status(), StatusCode::OK);
    let publish_json = body_json(publish_response).await;
    assert_eq!(publish_json["ok"], true);
    let published_source = publish_json["published_source"]
        .as_str()
        .expect("published source")
        .to_owned();
    assert_eq!(published_source, format!("mods/publish/{mod_id}.civmod"));

    let published_response = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/control/mods/published")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(published_response.status(), StatusCode::OK);
    let published_json = body_json(published_response).await;
    assert!(published_json
        .as_array()
        .expect("published array")
        .iter()
        .any(|entry| {
            entry["id"] == mod_id
                && entry["source"] == published_source
                && entry["name"] == "Upload Test"
                && entry["version"] == "0.0.1"
        }));

    let catalog_response = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/control/mods/catalog")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(catalog_response.status(), StatusCode::OK);
    let catalog_json = body_json(catalog_response).await;
    assert!(catalog_json
        .as_array()
        .expect("catalog array")
        .iter()
        .any(|entry| entry["source"] == published_source && entry["id"] == mod_id));

    let install_response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/control/mods/install")
                .header("content-type", "application/json")
                .body(Body::from(format!(r#"{{"source":"{published_source}"}}"#)))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(install_response.status(), StatusCode::OK);
    let install_json = body_json(install_response).await;
    assert_eq!(install_json["ok"], true);

    let uploaded_path = repo_root().join(upload_source.replace('/', std::path::MAIN_SEPARATOR_STR));
    let published_path =
        repo_root().join(published_source.replace('/', std::path::MAIN_SEPARATOR_STR));
    let _ = std::fs::remove_file(uploaded_path);
    let _ = std::fs::remove_file(published_path);
}

#[test]
fn validate_remote_fetch_url_rejects_empty_and_non_http() {
    assert!(validate_remote_fetch_url("").is_err());
    assert!(validate_remote_fetch_url("   ").is_err());
    assert!(validate_remote_fetch_url("file:///tmp/mod.civmod").is_err());
    assert!(validate_remote_fetch_url("ftp://example.com/mod.civmod").is_err());
    assert!(validate_remote_fetch_url("https://example.com/mod.civmod").is_ok());
    assert!(validate_remote_fetch_url("http://127.0.0.1/mod.civmod").is_ok());
}

#[test]
fn remote_registry_rejects_url_when_required() {
    let registry = RemoteModRegistry {
        require_registry: true,
        entries: vec![RemoteModRegistryEntry {
            url_prefix: "https://mods.example.com/".to_string(),
            mod_id: Some("demo-mod".to_string()),
            require_signature: false,
            allowed_pubkeys: vec![],
        }],
    };
    assert!(validate_remote_fetch_against_registry(
        &registry,
        "https://evil.example/mod.civmod",
        None
    )
    .is_err());
    assert!(validate_remote_fetch_against_registry(
        &registry,
        "https://mods.example.com/demo.civmod",
        Some("demo-mod")
    )
    .is_ok());
}

#[test]
fn resolve_remote_cache_id_uses_mod_id_or_url_hash() {
    let url = "https://example.com/mods/demo.civmod";
    assert_eq!(
        resolve_remote_cache_id(url, Some("demo-mod")).expect("mod id"),
        "demo-mod"
    );
    let hashed = resolve_remote_cache_id(url, None).expect("hash id");
    assert!(hashed.starts_with("url-"));
    assert_eq!(hashed.len(), "url-".len() + 16);
    assert!(resolve_remote_cache_id(url, Some("../escape")).is_err());
}

#[test]
fn remote_mod_cache_dir_is_under_remote_root() {
    let mods_dir = repo_root().join("mods");
    let path = remote_mod_cache_dir(&mods_dir, "demo-mod");
    assert!(path.ends_with("mods/remote/demo-mod") || path.ends_with("mods\\remote\\demo-mod"));
}

#[tokio::test]
async fn persist_remote_mod_cache_writes_meta_and_catalog_lists_it() {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("time")
        .as_nanos();
    let cache_id = format!("remote-cache-test-{nanos}");
    let mods_dir = repo_root().join("mods");
    let cache_dir = remote_mod_cache_dir(&mods_dir, &cache_id);
    let bytes = minimal_upload_civmod_bytes(&cache_id);
    let url = "https://example.com/test.civmod";
    let (archive_path, source) =
        persist_remote_mod_cache(&mods_dir, &cache_id, url, &bytes, None).expect("persist");
    assert!(archive_path.is_file());
    assert!(source.contains("mods/remote"));
    assert!(source.ends_with(REMOTE_MOD_ARCHIVE_NAME));

    let meta = read_remote_mod_meta(&cache_dir).expect("meta");
    assert_eq!(meta.id, cache_id);
    assert_eq!(meta.url, url);
    assert!(!meta.signed);
    assert!(meta.author_pubkey_hex.is_none());

    let remote_list = scan_remote_mod_cache(&mods_dir);
    assert!(remote_list
        .iter()
        .any(|entry| { entry.id == cache_id && entry.url == url && !entry.signed }));

    let catalog = scan_mod_catalog(&mods_dir, &std::collections::HashSet::new());
    assert!(catalog
        .iter()
        .any(|entry| entry.source == source && entry.id == cache_id));

    let _ = std::fs::remove_dir_all(cache_dir);
}

#[tokio::test]
async fn post_mods_fetch_and_remote_list_round_trip() {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("time")
        .as_nanos();
    let cache_id = format!("remote-fetch-{nanos}");
    let bytes = minimal_upload_civmod_bytes(&cache_id);
    let fixture = Router::new().route(
        "/mod.civmod",
        get(move || {
            let bytes = bytes.clone();
            async move { bytes }
        }),
    );
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
        .await
        .expect("bind fixture");
    let addr = listener.local_addr().expect("fixture addr");
    tokio::spawn(async move {
        axum::serve(listener, fixture)
            .await
            .expect("fixture server");
    });

    let url = format!("http://{addr}/mod.civmod");
    let app = test_app();
    let fetch_response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/control/mods/fetch")
                .header("content-type", "application/json")
                .body(Body::from(format!(
                    r#"{{"url":"{url}","mod_id":"{cache_id}"}}"#
                )))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(fetch_response.status(), StatusCode::OK);
    let fetch_json = body_json(fetch_response).await;
    assert_eq!(fetch_json["ok"], true);
    let source = fetch_json["source"].as_str().expect("source").to_owned();

    let remote_response = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/control/mods/remote")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(remote_response.status(), StatusCode::OK);
    let remote_json = body_json(remote_response).await;
    assert!(remote_json
        .as_array()
        .expect("remote array")
        .iter()
        .any(|entry| entry["id"] == cache_id && entry["url"] == url));

    let catalog_response = app
        .oneshot(
            Request::builder()
                .uri("/control/mods/catalog")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(catalog_response.status(), StatusCode::OK);
    let catalog_json = body_json(catalog_response).await;
    assert!(catalog_json
        .as_array()
        .expect("catalog array")
        .iter()
        .any(|entry| entry["source"] == source && entry["id"] == cache_id));

    let cache_dir = remote_mod_cache_dir(&repo_root().join("mods"), &cache_id);
    let _ = std::fs::remove_dir_all(cache_dir);
}

/// FR-SAVE-002 (error branch) — `POST /control/save/slot` with a slot id that is
/// not one of the canonical production slots is rejected with `400 Bad Request`
/// and an `ok: false` body, before any save is attempted.
#[tokio::test]
async fn fr_save_002_post_save_slot_rejects_invalid_slot() {
    let app = test_app();
    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/control/save/slot")
                .header("content-type", "application/json")
                .body(Body::from(r#"{"slot":"not-a-real-slot"}"#))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    let json = body_json(response).await;
    assert_eq!(json["ok"], false);
}

/// FR-SAVE-002 (error branch) — `POST /control/load/slot` validates the slot id
/// the same way: an unknown slot yields `400 Bad Request` with `ok: false` and
/// never touches the filesystem.
#[tokio::test]
async fn fr_save_002_post_load_slot_rejects_invalid_slot() {
    let app = test_app();
    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/control/load/slot")
                .header("content-type", "application/json")
                .body(Body::from(r#"{"slot":"bogus"}"#))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    let json = body_json(response).await;
    assert_eq!(json["ok"], false);
}

/// FR-SAVE load (error branch) — `POST /control/load` rejects traversal-style
/// filenames before touching the filesystem: `save_path` / `sanitize_save_filename`
/// disallow `/`, `\`, and `..`.
#[tokio::test]
async fn fr_save_load_rejects_traversal_filename() {
    let app = test_app();
    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/control/load")
                .header("content-type", "application/json")
                .body(Body::from(r#"{"filename":"../escape"}"#))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    let json = body_json(response).await;
    assert_eq!(json["ok"], false);
}

/// FR-SAVE load (error branch) — `POST /control/load` with a valid but missing
/// name falls through to `legacy_replay_path` and `Simulation::load_replay_from_file`,
/// which fails with a non-success status when no save archive, folder, or replay exists.
#[tokio::test]
async fn fr_save_load_missing_save_is_error() {
    let app = test_app();
    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/control/load")
                .header("content-type", "application/json")
                .body(Body::from(r#"{"filename":"definitely-does-not-exist-xyz"}"#))
                .unwrap(),
        )
        .await
        .unwrap();

    assert!(!response.status().is_success());
}

#[tokio::test]
async fn post_control_spawn_entity_unknown_kind_is_rejected() {
    let app = test_app();
    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/control/spawn_entity")
                .header("content-type", "application/json")
                .body(Body::from(
                    r#"{"kind":"bogus-kind","x":0.5,"y":0.5,"faction":0}"#,
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    let json = body_json(response).await;
    assert_eq!(json["ok"], false);
    assert!(json["message"]
        .as_str()
        .unwrap_or("")
        .contains("civilian"));
}

#[tokio::test]
async fn post_control_spawn_entity_herd_returns_ok() {
    let app = test_app();
    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/control/spawn_entity")
                .header("content-type", "application/json")
                .body(Body::from(
                    r#"{"kind":"herd","x":0.3,"y":0.7,"faction":2}"#,
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    let json = body_json(response).await;
    assert_eq!(json["ok"], true);
}

#[tokio::test]
async fn post_control_spawn_entity_airport_and_port_return_ok() {
    for kind in ["airport", "port"] {
        let app = test_app();
        let response = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/control/spawn_entity")
                    .header("content-type", "application/json")
                    .body(Body::from(format!(
                        r#"{{"kind":"{kind}","x":0.5,"y":0.5,"faction":0}}"#
                    )))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::OK);
        let json = body_json(response).await;
        assert_eq!(json["ok"], true);
    }
}

#[tokio::test]
async fn post_control_mods_fetch_rejects_unsupported_scheme() {
    let app = test_app();
    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/control/mods/fetch")
                .header("content-type", "application/json")
                .body(Body::from(r#"{"url":"ftp://example.com/mod.zip"}"#))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    let json = body_json(response).await;
    assert_eq!(json["ok"], false);
}

#[tokio::test]
async fn post_control_mods_fetch_rejects_empty_url() {
    let app = test_app();
    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/control/mods/fetch")
                .header("content-type", "application/json")
                .body(Body::from(r#"{"url":""}"#))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    let json = body_json(response).await;
    assert_eq!(json["ok"], false);
}

#[tokio::test]
async fn post_control_mods_upload_rejects_traversal_filename() {
    let body = serde_json::json!({
        "filename": "../evil.civmod",
        "data_base64": "",
    })
    .to_string();
    let app = test_app();
    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/control/mods/upload")
                .header("content-type", "application/json")
                .body(Body::from(body))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    let json = body_json(response).await;
    assert_eq!(json["ok"], false);
}

#[tokio::test]
async fn post_control_mods_upload_rejects_empty_filename() {
    let body = serde_json::json!({
        "filename": "",
        "data_base64": "",
    })
    .to_string();
    let app = test_app();
    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/control/mods/upload")
                .header("content-type", "application/json")
                .body(Body::from(body))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    let json = body_json(response).await;
    assert_eq!(json["ok"], false);
}

#[tokio::test]
async fn post_control_mods_publish_rejects_non_mods_source() {
    let app = test_app();
    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/control/mods/publish")
                .header("content-type", "application/json")
                .body(Body::from(
                    serde_json::json!({ "source": "notmods/whatever.civmod" }).to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    let json = body_json(response).await;
    assert_eq!(json["ok"], false);

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/control/mods/publish")
                .header("content-type", "application/json")
                .body(Body::from(serde_json::json!({ "source": "../escape" }).to_string()))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    let json = body_json(response).await;
    assert_eq!(json["ok"], false);
}
