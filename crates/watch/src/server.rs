//! Axum router construction and HTTP server bootstrap.

use std::{
    net::SocketAddr,
    path::PathBuf,
    sync::{
        atomic::{AtomicU16, AtomicU8},
        Arc,
    },
};

use axum::{
    routing::{get, post},
    Router,
};
use civ_engine::Simulation;
use civ_save_db::SaveDb;
use tokio::sync::{broadcast, Mutex, RwLock};
use tower_http::{cors::CorsLayer, services::ServeDir};
use tracing::info;

use crate::app::{
    env_u16, load_law_db, resolve_data_dir, resolve_session_id, AppState, Snapshot,
    TerrainCache, REMOTE_FETCH_TIMEOUT,
};
use crate::control_routes::{
    damage_handler, place_voxel_handler, spawn_civilian_handler, spawn_entity_handler,
    speed_handler,
};
use crate::mods_api::{
    fetch_mod_handler, install_mod_handler, list_mod_catalog_handler, list_published_mods_handler,
    list_remote_mods_handler, publish_mod_handler, reload_mod_handler, unload_mod_handler,
    upload_mod_handler,
};
use crate::saves_api::{
    list_saves_handler, load_handler, load_slot_handler, save_handler, save_slot_handler,
};
use crate::sim_worker::{seed_civilians, seed_military, seed_voxels, simulation_worker};
use crate::sse::{snapshot_handler, sse_handler, terrain_handler};
use crate::terrain::Terrain;

pub(crate) fn build_api_router() -> Router<AppState> {
    Router::new()
        .route("/events", get(sse_handler))
        .route("/snapshot", get(snapshot_handler))
        .route("/terrain", get(terrain_handler))
        .route("/control/place_voxel", post(place_voxel_handler))
        .route("/control/spawn_civilian", post(spawn_civilian_handler))
        .route("/control/spawn_entity", post(spawn_entity_handler))
        .route("/control/damage", post(damage_handler))
        .route("/control/speed", post(speed_handler))
        .route("/control/save", post(save_handler))
        .route("/control/save/slot", post(save_slot_handler))
        .route("/control/load", post(load_handler))
        .route("/control/load/slot", post(load_slot_handler))
        .route("/control/saves", get(list_saves_handler))
        .route("/control/mods/catalog", get(list_mod_catalog_handler))
        .route("/control/mods/upload", post(upload_mod_handler))
        .route("/control/mods/publish", post(publish_mod_handler))
        .route("/control/mods/published", get(list_published_mods_handler))
        .route("/control/mods/install", post(install_mod_handler))
        .route("/control/mods/unload", post(unload_mod_handler))
        .route("/control/mods/reload", post(reload_mod_handler))
        .route("/control/mods/fetch", post(fetch_mod_handler))
        .route("/control/mods/remote", get(list_remote_mods_handler))
}

fn build_app(state: AppState) -> Router {
    build_api_router()
        .fallback_service(
            ServeDir::new("web/dashboard/dist").append_index_html_on_directories(true),
        )
        .with_state(state)
        .layer(CorsLayer::permissive())
}

pub async fn run() {
    let (tx, _) = broadcast::channel::<Snapshot>(64);
    let terrain = Terrain::generate(42);
    let terrain_cache = TerrainCache::from_terrain(&terrain);
    let data_dir = resolve_data_dir();
    let saves_dir = Arc::new(data_dir.join("saves"));
    std::fs::create_dir_all(&*saves_dir).expect("create saves dir");
    let save_db_path = data_dir.join("saves.db");
    let save_db = Arc::new(
        SaveDb::open(&save_db_path)
            .unwrap_or_else(|err| panic!("open save db at {save_db_path:?}: {err}")),
    );
    let session_id = resolve_session_id();
    info!(%session_id, ?save_db_path, "session-scoped save metadata db ready");
    let mods_dir = Arc::new(
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../mods")
            .canonicalize()
            .unwrap_or_else(|_| PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../mods")),
    );
    std::fs::create_dir_all(&*mods_dir).ok();
    std::fs::create_dir_all(mods_dir.join("uploads")).ok();
    std::fs::create_dir_all(mods_dir.join("publish")).ok();
    std::fs::create_dir_all(mods_dir.join("remote")).ok();
    let laws = Arc::new(load_law_db(mods_dir.as_path()));
    let http = reqwest::Client::builder()
        .timeout(REMOTE_FETCH_TIMEOUT)
        .redirect(reqwest::redirect::Policy::limited(5))
        .build()
        .expect("reqwest client");
    info!(
        "terrain: {0}x{0} = {1} cells generated",
        terrain.size,
        terrain.heights.len()
    );

    let sim = Arc::new(Mutex::new(Simulation::with_seed(42)));
    let military = Arc::new(Mutex::new(Vec::new()));
    {
        let mut s = sim.lock().await;
        s.register_mod_stubs(&[
            "mods/example-policy".to_owned(),
            "mods/example-economic".to_owned(),
        ]);
        seed_voxels(&mut s);
        seed_civilians(&mut s, &terrain);
    }
    {
        let mut s = sim.lock().await;
        let mut units = military.lock().await;
        seed_military(&mut s, &terrain, &mut units);
    }

    let state = AppState {
        latest: Arc::new(RwLock::new(None)),
        tx: tx.clone(),
        terrain: Arc::new(terrain),
        terrain_cache,
        laws,
        sim,
        military,
        target_era: Arc::new(AtomicU16::new(0)),
        speed: Arc::new(AtomicU8::new(1)),
        saves_dir,
        mods_dir,
        session_id,
        save_db,
        http,
    };

    tokio::spawn(simulation_worker(state.clone()));

    let app = build_app(state);

    let port = env_u16("CIV_WATCH_PORT", 9090);
    let addr: SocketAddr = format!("0.0.0.0:{port}")
        .parse()
        .expect("valid listen address");
    info!("civ-watch listening on http://{addr}");
    info!("dashboard: http://localhost:{port}");

    let listener = tokio::net::TcpListener::bind(addr)
        .await
        .unwrap_or_else(|e| panic!("bind {port}: {e}"));
    axum::serve(listener, app).await.expect("axum server");
}
