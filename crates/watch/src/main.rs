//! `civ-watch` — local hot-reload sandbox harness for Civis 3D.
//!
//! Background `Simulation` ticks at ~10 Hz; SSE snapshots at `GET /events`;
//! latest snapshot at `GET /snapshot`; procedural heightmap at `GET /terrain`;
//! sandbox controls under `POST /control/*` (place_voxel, spawn_civilian,
//! damage, speed). Dashboard static build under `web/dashboard/dist` is
//! served at `GET /`.

mod terrain;

use std::{
    convert::Infallible,
    net::SocketAddr,
    path::{Path, PathBuf},
    sync::{
        atomic::{AtomicU16, AtomicU8, Ordering},
        Arc,
    },
    time::Duration,
};

use axum::{
    body::{Body, Bytes},
    extract::State,
    http::{header, HeaderMap, HeaderValue, StatusCode},
    response::{
        sse::{Event, KeepAlive, Sse},
        IntoResponse, Json, Response,
    },
    routing::{get, post},
    Router,
};
use civ_agents::{
    drift_toward_home, spawn_civilian_at, tick_movement, Civilian as AgentCivilian, Needs,
    Position3d, Velocity,
};
use civ_engine::{Citizen, CivSaveBundle, DiplomacyKind, JobType, ModBrowserEntry, ModType, Simulation, load_manifest};
use base64::Engine as _;
use civ_mod_host::{read_civmod_archive, read_manifest_from_civmod, CIVMOD_MANIFEST_NAME};
use civ_save_db::{format_session_saved_event_json, SaveDb};
use sha2::{Digest, Sha256};
use civ_laws::{LawDb, LawKind};
use civ_server::build_voxel_delta_frame;
use civ_tactics::DamageEvent;
use civ_voxel::{MaterialId, WorldCoord};
use futures::{stream::Stream, StreamExt};
use serde::{Deserialize, Serialize};
use tokio::sync::{broadcast, Mutex, RwLock};
use tokio_stream::wrappers::BroadcastStream;
use tower_http::{cors::CorsLayer, services::ServeDir};
use tracing::{info, warn};

use crate::terrain::Terrain;

fn env_u16(name: &str, default: u16) -> u16 {
    std::env::var(name)
        .ok()
        .and_then(|value| value.parse().ok())
        .unwrap_or(default)
}

fn resolve_data_dir() -> PathBuf {
    std::env::var("CIVIS_DATA_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from("."))
}

fn resolve_session_id() -> String {
    std::env::var("CIVIS_SESSION_ID")
        .ok()
        .filter(|value| !value.trim().is_empty())
        .unwrap_or_else(|| uuid::Uuid::new_v4().to_string())
}

#[derive(Debug, Clone, Serialize)]
struct SampleCivilian {
    age: u32,
    health: f64,
    ideology: f64,
    welfare: f64,
    job: Option<JobLabel>,
}

#[derive(Debug, Clone, Copy, Serialize)]
#[serde(rename_all = "lowercase")]
enum JobLabel {
    Farmer,
    Warrior,
    Scholar,
    Trader,
    Priest,
    Admin,
    Unemployed,
}

impl From<JobType> for JobLabel {
    fn from(value: JobType) -> Self {
        match value {
            JobType::Farmer => Self::Farmer,
            JobType::Warrior => Self::Warrior,
            JobType::Scholar => Self::Scholar,
            JobType::Trader => Self::Trader,
            JobType::Priest => Self::Priest,
            JobType::Admin => Self::Admin,
            JobType::Unemployed => Self::Unemployed,
        }
    }
}

#[derive(Debug, Clone, Serialize)]
struct CivPin {
    idx: u32,
    x: f32,
    y: f32,
    dx: f32,
    dy: f32,
    job: Option<JobLabel>,
}

#[derive(Debug, Clone, Serialize)]
struct MilitaryPin {
    id: u64,
    x: f32,
    y: f32,
    unit_type: String,
    faction: u32,
    strength: f32,
}

#[derive(Debug, Clone, Serialize)]
struct DamagePulse {
    x: f32,
    y: f32,
    /// Engaging unit pin id when damage came from military contact.
    #[serde(skip_serializing_if = "Option::is_none")]
    unit_a: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    unit_b: Option<u64>,
}

#[derive(Debug, Clone, Serialize)]
struct DisasterEvent {
    tick: u64,
    kind: String,
    x: f32,
    y: f32,
    radius: f32,
    severity: f32,
}

#[derive(Debug, Clone, Serialize)]
struct Faction {
    id: u32,
    color: [u8; 3],
    capital: [f32; 2],
    radius: f32,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
#[serde(rename_all = "PascalCase")]
enum BuildingKind {
    Residential,
    Commercial,
    Industrial,
    Civic,
}

#[derive(Debug, Clone, Serialize)]
struct Building {
    id: u32,
    x: f32,
    y: f32,
    kind: BuildingKind,
    era: u8,
    faction_id: u32,
    occupants: u32,
    capacity: u32,
}

#[derive(Debug, Clone, Serialize)]
struct HousingStats {
    total_capacity: u32,
    occupied: u32,
    homeless: u32,
    vacancy_rate: f32,
}

#[derive(Debug, Clone, Serialize)]
struct Road {
    from: [f32; 2],
    to: [f32; 2],
    width: f32,
    kind: RoadKind,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "PascalCase")]
enum RoadKind {
    Trail,
    Dirt,
    Paved,
    Highway,
}

#[derive(Debug, Clone, Serialize)]
struct TradeRoute {
    from_faction: u32,
    to_faction: u32,
    goods: String,
    volume: f32,
}

#[derive(Debug, Clone, Serialize)]
struct GameEvent {
    tick: u64,
    kind: String,
    message: String,
    faction_id: Option<u32>,
}

#[derive(Debug, Clone, Serialize)]
struct TechNode {
    id: String,
    kind: String,
    era_min: u16,
    unlocked: bool,
}

#[derive(Debug, Clone, Serialize)]
struct InstitutionRow {
    id: u32,
    kind: String,
    balance_joules: i64,
}

#[derive(Debug, Clone, Serialize)]
struct EconomySnapshot {
    energy_budget: f64,
    faction_treasury: Vec<FactionTreasury>,
    production_rates: ProductionRates,
    institutions: Vec<InstitutionRow>,
    resources: ResourceSnapshot,
}

#[derive(Debug, Clone, Serialize)]
struct WeatherSnapshot {
    season: String,
    temperature: f32,
    wind_speed: f32,
    precipitation: String,
}

#[derive(Debug, Clone, Serialize)]
struct ResourceSnapshot {
    food: f64,
    wood: f64,
    metal: f64,
    energy: f64,
}

#[derive(Debug, Clone, Serialize)]
struct PopulationPulse {
    tick: u64,
    entity_id: u64,
    x: f32,
    y: f32,
}

#[derive(Debug, Clone, Serialize)]
struct DiplomacyPulse {
    tick: u64,
    faction_a: u32,
    faction_b: u32,
    kind: DiplomacyKind,
}

#[derive(Debug, Clone, Serialize)]
struct FactionTreasury {
    id: u32,
    name: String,
    balance: f64,
    trade_balance: f64,
}

#[derive(Debug, Clone, Serialize)]
struct ProductionRates {
    food_per_tick: f64,
    wood_per_tick: f64,
    metal_per_tick: f64,
    energy_per_tick: f64,
}

#[derive(Debug, Clone, Default)]
struct TradeTickSummary {
    balances: std::collections::HashMap<u32, f64>,
    volume: f64,
}

#[derive(Debug, Clone, Serialize)]
struct Snapshot {
    tick: u64,
    tick_dt_ms: u32,
    current_era: u16,
    population: u64,
    voxel_dirty_count: usize,
    voxel_chunk_count: usize,
    sample_civilians: Vec<SampleCivilian>,
    civ_pins: Vec<CivPin>,
    factions: Vec<Faction>,
    buildings: Vec<Building>,
    housing_stats: HousingStats,
    roads: Vec<Road>,
    trade_routes: Vec<TradeRoute>,
    economy: EconomySnapshot,
    trade_volume_this_tick: f64,
    births_this_tick: u32,
    deaths_this_tick: u32,
    diplomacy_events: Vec<DiplomacyPulse>,
    military_units: Vec<MilitaryPin>,
    damage_events: Vec<DamagePulse>,
    damage_events_count: u32,
    disaster_events: Vec<DisasterEvent>,
    birth_events: Vec<PopulationPulse>,
    death_events: Vec<PopulationPulse>,
    tech_tree: Vec<TechNode>,
    events: Vec<GameEvent>,
    is_day: bool,
    weather: WeatherSnapshot,
    speed: u8,
    /// Loaded CivLab mods for dashboard mod browser (FR-CIV-TACTICS-054).
    mods: Vec<ModBrowserEntry>,
}

/// Pre-serialized terrain JSON and a stable ETag for cheap repeat fetches.
#[derive(Clone)]
struct TerrainCache {
    body: Bytes,
    etag: HeaderValue,
}

impl TerrainCache {
    fn from_terrain(terrain: &Terrain) -> Self {
        let body = Bytes::from(serde_json::to_vec(terrain).expect("terrain serializes"));
        let etag = format!("\"{:016x}\"", terrain.heights_fingerprint());
        Self {
            body,
            etag: HeaderValue::from_str(&etag).expect("valid etag"),
        }
    }
}

#[derive(Clone)]
struct AppState {
    latest: Arc<RwLock<Option<Snapshot>>>,
    tx: broadcast::Sender<Snapshot>,
    terrain: Arc<Terrain>,
    terrain_cache: TerrainCache,
    laws: Arc<LawDb>,
    sim: Arc<Mutex<Simulation>>,
    military: Arc<Mutex<Vec<MilitaryPin>>>,
    target_era: Arc<AtomicU16>,
    speed: Arc<AtomicU8>,
    saves_dir: Arc<PathBuf>,
    mods_dir: Arc<PathBuf>,
    session_id: String,
    save_db: Arc<SaveDb>,
    http: reqwest::Client,
}

#[derive(Debug, Serialize)]
struct ControlOk {
    ok: bool,
    message: Option<String>,
}

#[derive(Debug, Deserialize)]
struct PlaceVoxelReq {
    x: i64,
    y: i64,
    z: i64,
    material: u16,
}

#[derive(Debug, Deserialize)]
struct SpawnCivilianReq {
    x: f32,
    y: f32,
    faction: u32,
}

#[derive(Debug, Deserialize)]
struct SpawnEntityReq {
    kind: String,
    x: f32,
    y: f32,
    faction: u32,
}

#[derive(Debug, Deserialize)]
struct DamageReq {
    x: i64,
    y: i64,
    z: i64,
    radius: u8,
    energy: u32,
}

#[derive(Debug, Deserialize)]
struct SpeedReq {
    speed: u8,
}

const PRODUCTION_SLOTS: [&str; 5] = ["slot-1", "slot-2", "slot-3", "slot-4", "slot-5"];
const AUTOSAVE_RING_MAX: usize = 10;

#[derive(Debug, Deserialize)]
struct SaveReq {
    filename: String,
}

#[derive(Debug, Deserialize)]
struct SlotReq {
    slot: String,
}

#[derive(Debug, Serialize)]
struct SaveResponse {
    ok: bool,
    path: String,
    tick: u64,
}

#[derive(Debug, Serialize)]
struct LoadResponse {
    ok: bool,
    tick: u64,
}

#[derive(Debug, Serialize)]
struct SaveListEntry {
    name: String,
    size_bytes: u64,
    modified: Option<u64>,
    save_type: &'static str,
    #[serde(skip_serializing_if = "Option::is_none")]
    session_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    save_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    tick: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    created_at: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
struct ModCatalogEntry {
    source: String,
    id: String,
    name: String,
    version: String,
    mod_type: String,
    kind: String,
    installed: bool,
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    signed: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    author_pubkey_hex: Option<String>,
}

#[derive(Debug, Deserialize)]
struct InstallModReq {
    source: String,
}

#[derive(Debug, Deserialize)]
struct UnloadModReq {
    mod_id: String,
}

#[derive(Debug, Deserialize)]
struct ReloadModReq {
    mod_id: String,
}

#[derive(Debug, Deserialize)]
struct PublishModReq {
    source: String,
}

#[derive(Debug, Serialize)]
struct PublishModResponse {
    ok: bool,
    published_source: String,
}

#[derive(Debug, Clone, Serialize)]
struct PublishedModEntry {
    id: String,
    name: String,
    version: String,
    source: String,
}

#[derive(Debug, Deserialize)]
struct UploadModReq {
    filename: String,
    data_base64: String,
}

#[derive(Debug, Serialize)]
struct UploadModResponse {
    ok: bool,
    source: String,
}

const REMOTE_MOD_MAX_BYTES: usize = 50 * 1024 * 1024;
const REMOTE_FETCH_TIMEOUT: Duration = Duration::from_secs(30);
const REMOTE_MOD_META_NAME: &str = "meta.json";
const REMOTE_MOD_ARCHIVE_NAME: &str = "mod.civmod";
const REMOTE_REGISTRY_NAME: &str = "remote-registry.json";

#[derive(Debug, Clone, Default, Deserialize)]
struct RemoteModRegistry {
    #[serde(default)]
    require_registry: bool,
    #[serde(default)]
    entries: Vec<RemoteModRegistryEntry>,
}

#[derive(Debug, Clone, Deserialize)]
struct RemoteModRegistryEntry {
    url_prefix: String,
    #[serde(default)]
    mod_id: Option<String>,
    #[serde(default)]
    require_signature: bool,
    #[serde(default)]
    allowed_pubkeys: Vec<String>,
}

fn load_remote_mod_registry(mods_dir: &Path) -> RemoteModRegistry {
    let path = mods_dir.join(REMOTE_REGISTRY_NAME);
    let Ok(contents) = std::fs::read_to_string(path) else {
        return RemoteModRegistry::default();
    };
    serde_json::from_str(&contents).unwrap_or_default()
}

fn match_registry_entry<'a>(
    registry: &'a RemoteModRegistry,
    url: &str,
) -> Option<&'a RemoteModRegistryEntry> {
    let trimmed = url.trim();
    registry
        .entries
        .iter()
        .find(|entry| trimmed.starts_with(entry.url_prefix.trim()))
}

fn validate_remote_fetch_against_registry<'a>(
    registry: &'a RemoteModRegistry,
    url: &str,
    mod_id: Option<&str>,
) -> Result<Option<&'a RemoteModRegistryEntry>, String> {
    let matched = match_registry_entry(registry, url);
    if registry.require_registry && matched.is_none() {
        return Err(format!("url not in signed remote mod registry: {}", url.trim()));
    }
    if let (Some(entry), Some(requested_id)) = (matched, mod_id) {
        if let Some(expected) = entry.mod_id.as_deref() {
            if expected != requested_id {
                return Err(format!(
                    "mod_id {requested_id} does not match registry entry ({expected})"
                ));
            }
        }
    }
    Ok(matched)
}

fn validate_remote_mod_against_registry(
    entry: Option<&RemoteModRegistryEntry>,
    manifest: &civ_mod_host::ModManifest,
) -> Result<(), String> {
    let Some(entry) = entry else {
        return Ok(());
    };
    if let Some(expected) = entry.mod_id.as_deref() {
        if manifest.meta.id != expected {
            return Err(format!(
                "archive mod id `{}` does not match registry (`{expected}`)",
                manifest.meta.id
            ));
        }
    }
    if entry.require_signature && manifest.meta.author_pubkey_hex.is_none() {
        return Err("remote mod must be signed (author_pubkey_hex missing)".into());
    }
    if let Some(pk) = manifest.meta.author_pubkey_hex.as_deref() {
        if !entry.allowed_pubkeys.is_empty()
            && !entry
                .allowed_pubkeys
                .iter()
                .any(|allowed| allowed.eq_ignore_ascii_case(pk))
        {
            return Err("author pubkey not in registry allowlist".into());
        }
    }
    Ok(())
}

#[derive(Debug, Deserialize)]
struct FetchModReq {
    url: String,
    #[serde(default)]
    mod_id: Option<String>,
}

#[derive(Debug, Serialize)]
struct FetchModResponse {
    ok: bool,
    id: String,
    source: String,
    path: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct RemoteModMeta {
    id: String,
    url: String,
    fetched_at: u64,
    #[serde(default)]
    signed: bool,
    #[serde(default)]
    author_pubkey_hex: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
struct RemoteModEntry {
    id: String,
    path: String,
    fetched_at: u64,
    url: String,
    signed: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    author_pubkey_hex: Option<String>,
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "civ_watch=info".into()),
        )
        .init();

    let (tx, _) = broadcast::channel::<Snapshot>(64);
    let terrain = Terrain::generate(42);
    let terrain_cache = TerrainCache::from_terrain(&terrain);
    let laws = Arc::new(default_law_db());
    let data_dir = resolve_data_dir();
    let saves_dir = Arc::new(data_dir.join("saves"));
    std::fs::create_dir_all(&*saves_dir).expect("create saves dir");
    let save_db_path = data_dir.join("saves.db");
    let save_db = Arc::new(
        SaveDb::open(&save_db_path).unwrap_or_else(|err| panic!("open save db at {save_db_path:?}: {err}")),
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

fn build_api_router() -> Router<AppState> {
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

async fn simulation_worker(state: AppState) {
    let mut interval = tokio::time::interval(Duration::from_millis(100));
    loop {
        interval.tick().await;
        let speed = state.speed.load(Ordering::Relaxed);
        if speed == 0 {
            continue;
        }
        let snapshot = {
            let mut sim = state.sim.lock().await;
            let mut military = state.military.lock().await;
            let mut damage_events = Vec::new();
            let mut trade = TradeTickSummary::default();
            for _ in 0..speed {
                sim.tick();
                if sim.state.tick > 0 && sim.state.tick % 600 == 0 {
                    state
                        .target_era
                        .store(((sim.state.tick / 600).min(5)) as u16, Ordering::Relaxed);
                }
                let terrain = state.terrain.clone();
                let factions = factions(sim.state.tick);
                let buildings = buildings(&factions, sim.state.tick);
                assign_and_drift_housing(&mut sim, &buildings);
                let mut rng = sim.rng_mut().clone();
                tick_movement(&mut sim.world, 128, &mut rng, |x, y| {
                    terrain.is_walkable(x, y)
                });
                *sim.rng_mut() = rng;
                damage_events = tick_military(&mut sim, &terrain, &mut military);
                let tick = sim.state.tick;
                let (trade_volume, trade_balances) = apply_trade_routes(&mut sim, &factions, tick);
                trade.volume += trade_volume;
                for (faction_id, balance) in trade_balances {
                    *trade.balances.entry(faction_id).or_insert(0.0) += balance;
                }
                for event in &damage_events {
                    sim.push_damage(DamageEvent {
                        center: WorldCoord {
                            x: (event.x * civ_voxel::FIXED_SCALE as f32) as i64,
                            y: 0,
                            z: (event.y * civ_voxel::FIXED_SCALE as f32) as i64,
                        },
                        radius_voxels: 1,
                        energy: 8,
                    });
                }
            }
            let current_era = state.target_era.load(Ordering::Relaxed);
            make_snapshot(
                &sim,
                &military,
                &damage_events,
                &trade,
                speed,
                &state.laws,
                current_era,
            )
        };
        *state.latest.write().await = Some(snapshot.clone());
        let _ = state.tx.send(snapshot);
    }
}

fn seed_voxels(sim: &mut Simulation) {
    // Seed a tiny block of voxels so the chunk store is non-empty before any
    // user interaction. Eventually the procedural terrain will be written into
    // the sim's voxel store too.
    for x in 0..8 {
        sim.voxel_mut().write(
            WorldCoord {
                x: i64::from(x) * 1_000_000,
                y: 0,
                z: 0,
            },
            MaterialId(1),
        );
    }
}

fn seed_civilians(sim: &mut Simulation, terrain: &Terrain) {
    let mut spawned = 0_u64;
    let mut x = 0.11_f32;
    let mut y = 0.19_f32;
    while spawned < 32 {
        if terrain.is_walkable(x, y) {
            let id = 10_000 + spawned;
            let mut rng = sim.rng_mut().clone();
            let _ = spawn_civilian_at(&mut sim.world, id, (spawned % 4) as u32, x, y, &mut rng);
            *sim.rng_mut() = rng;
            spawned += 1;
        }
        x = (x + 0.071).fract();
        y = (y + 0.113).fract();
    }
}

fn seed_military(sim: &mut Simulation, terrain: &Terrain, units: &mut Vec<MilitaryPin>) {
    let factions = factions(sim.state.tick);
    let mut next_id = 1_000_000_000_u64;
    for faction in factions {
        for _ in 0..5 {
            let seed = next_id ^ (u64::from(faction.id) << 32);
            units.push(MilitaryPin {
                id: next_id,
                x: (faction.capital[0] + noise_offset(seed, 0)).clamp(0.01, 0.99),
                y: (faction.capital[1] + noise_offset(seed, 1)).clamp(0.01, 0.99),
                unit_type: "Soldier".to_string(),
                faction: faction.id,
                strength: 1.0,
            });
            next_id += 1;
        }
    }
    let _ = terrain;
}

fn tick_military(
    sim: &mut Simulation,
    _terrain: &Terrain,
    units: &mut [MilitaryPin],
) -> Vec<DamagePulse> {
    let factions = factions(sim.state.tick);
    let conflict_factions: Vec<u32> = sim
        .diplomacy_events()
        .iter()
        .filter(|event| matches!(event.kind, DiplomacyKind::Conflict))
        .flat_map(|event| [event.faction_a, event.faction_b])
        .collect();
    if conflict_factions.is_empty() {
        return Vec::new();
    }

    let mut damage_events = Vec::new();
    for unit in units.iter_mut() {
        if !conflict_factions.contains(&unit.faction) {
            continue;
        }
        if let Some(target) = factions
            .iter()
            .filter(|faction| faction.id != unit.faction && conflict_factions.contains(&faction.id))
            .min_by(|a, b| {
                let da = (unit.x - a.capital[0]).powi(2) + (unit.y - a.capital[1]).powi(2);
                let db = (unit.x - b.capital[0]).powi(2) + (unit.y - b.capital[1]).powi(2);
                da.partial_cmp(&db).unwrap_or(std::cmp::Ordering::Equal)
            })
        {
            let dx = target.capital[0] - unit.x;
            let dy = target.capital[1] - unit.y;
            let dist = (dx * dx + dy * dy).sqrt().max(0.0001);
            let seed = unit.id ^ (u64::from(unit.faction) << 32) ^ sim.state.tick;
            unit.x = (unit.x + dx / dist * 0.01 + noise_offset(seed, 0) * 0.5).clamp(0.0, 1.0);
            unit.y = (unit.y + dy / dist * 0.01 + noise_offset(seed, 1) * 0.5).clamp(0.0, 1.0);
        }
    }

    for i in 0..units.len() {
        for j in (i + 1)..units.len() {
            if units[i].faction == units[j].faction {
                continue;
            }
            if !conflict_factions.contains(&units[i].faction)
                || !conflict_factions.contains(&units[j].faction)
            {
                continue;
            }
            let dx = units[i].x - units[j].x;
            let dy = units[i].y - units[j].y;
            if dx * dx + dy * dy <= 0.05 * 0.05 {
                damage_events.push(DamagePulse {
                    x: (units[i].x + units[j].x) * 0.5,
                    y: (units[i].y + units[j].y) * 0.5,
                    unit_a: Some(units[i].id),
                    unit_b: Some(units[j].id),
                });
                units[i].strength = (units[i].strength - 0.05).max(0.0);
                units[j].strength = (units[j].strength - 0.05).max(0.0);
            }
        }
    }
    damage_events
}

fn make_snapshot(
    sim: &Simulation,
    military: &[MilitaryPin],
    damage_events: &[DamagePulse],
    trade: &TradeTickSummary,
    speed: u8,
    laws: &LawDb,
    current_era: u16,
) -> Snapshot {
    let voxel_events = sim.last_tick_voxel_events();
    let sample_civilians = sample_civilians(sim);
    let civ_pins = civ_pins(sim);
    let factions = factions(sim.state.tick);
    let mut buildings = buildings(&factions, sim.state.tick);
    merge_authoring_buildings(&mut buildings, sim);
    let housing_stats = housing_snapshot(sim, &mut buildings);
    let roads = roads(&buildings);
    let trade_routes = trade_routes(&factions, sim.state.tick);
    let economy = economy_snapshot(sim, &factions, &trade.balances);
    let birth_events: Vec<PopulationPulse> = sim
        .last_births()
        .iter()
        .map(|event| PopulationPulse {
            tick: event.tick,
            entity_id: event.entity_id,
            x: event.x,
            y: event.y,
        })
        .collect::<Vec<PopulationPulse>>();
    let death_events: Vec<PopulationPulse> = sim
        .last_deaths()
        .iter()
        .map(|event| PopulationPulse {
            tick: event.tick,
            entity_id: event.entity_id,
            x: event.x,
            y: event.y,
        })
        .collect::<Vec<PopulationPulse>>();
    let diplomacy_events: Vec<DiplomacyPulse> = sim
        .diplomacy_events()
        .iter()
        .map(|event| DiplomacyPulse {
            tick: event.tick,
            faction_a: event.faction_a,
            faction_b: event.faction_b,
            kind: event.kind,
        })
        .collect();
    let tech_nodes = tech_tree(laws, current_era);
    let disaster_events = disaster_events(sim.state.tick, &factions, &buildings);
    let events = game_events(
        sim,
        &birth_events,
        &death_events,
        &diplomacy_events,
        &disaster_events,
        &buildings,
        &tech_nodes,
    );
    let _ = build_voxel_delta_frame(sim.state.tick, voxel_events, sim.voxel()).map_err(|err| {
        warn!(?err, "voxel frame build failed for current tick");
    });
    let climate = sim.climate();
    let is_day = climate.day_phase >= 0.25 && climate.day_phase < 0.75;
    let weather = weather_snapshot(sim.state.tick, climate.year_phase);
    let tick_dt_ms = 100u32 / u32::from(speed.max(1));

    Snapshot {
        tick: sim.state.tick,
        tick_dt_ms,
        current_era,
        population: sim.state.population,
        voxel_dirty_count: events.len(),
        voxel_chunk_count: sim.voxel().chunk_count(),
        sample_civilians,
        civ_pins,
        factions,
        buildings,
        housing_stats,
        roads,
        trade_routes,
        economy,
        trade_volume_this_tick: trade.volume,
        births_this_tick: birth_events.len() as u32,
        deaths_this_tick: death_events.len() as u32,
        diplomacy_events,
        military_units: military.to_vec(),
        damage_events: damage_events.to_vec(),
        damage_events_count: damage_events.len() as u32,
        disaster_events,
        birth_events,
        death_events,
        tech_tree: tech_nodes,
        events,
        is_day,
        weather,
        speed,
        mods: sim.mod_browser_entries(),
    }
}

fn weather_snapshot(tick: u64, year_phase: f32) -> WeatherSnapshot {
    let season = season_from_year_phase(year_phase);
    let temperature = temperature_from_year_phase(year_phase);
    let precipitation = precipitation_from_weather(&season, temperature);
    let wind_speed = 2.5
        + (year_phase * std::f32::consts::TAU).sin().abs() * 2.0
        + (tick as f32 * 0.000_01).sin().abs() * 0.5
        + season_wind_bias(&season);

    WeatherSnapshot {
        season,
        temperature,
        wind_speed,
        precipitation,
    }
}

fn season_from_year_phase(year_phase: f32) -> String {
    match year_phase {
        phase if phase < 0.25 => "Spring".to_string(),
        phase if phase < 0.5 => "Summer".to_string(),
        phase if phase < 0.75 => "Autumn".to_string(),
        _ => "Winter".to_string(),
    }
}

fn temperature_from_year_phase(year_phase: f32) -> f32 {
    11.0 + (std::f32::consts::TAU * (year_phase - 0.25)).sin() * 17.0
}

fn precipitation_from_weather(season: &str, temperature: f32) -> String {
    match season {
        "Winter" if temperature <= 0.0 => "snow".to_string(),
        "Winter" => "none".to_string(),
        "Spring" | "Autumn" if temperature < 12.0 => "rain".to_string(),
        "Summer" if temperature < 14.0 => "rain".to_string(),
        _ => "none".to_string(),
    }
}

fn season_wind_bias(season: &str) -> f32 {
    match season {
        "Spring" => 1.0,
        "Summer" => 0.4,
        "Autumn" => 1.2,
        "Winter" => 1.6,
        _ => 0.0,
    }
}

fn game_events(
    sim: &Simulation,
    births_this_tick: &[PopulationPulse],
    deaths_this_tick: &[PopulationPulse],
    diplomacy_events: &[DiplomacyPulse],
    disaster_events: &[DisasterEvent],
    buildings: &[Building],
    tech_tree: &[TechNode],
) -> Vec<GameEvent> {
    let mut events = Vec::new();
    let tick = sim.state.tick;

    for birth in births_this_tick {
        let faction_id = faction_for_point(birth.x, birth.y);
        events.push(GameEvent {
            tick: birth.tick,
            kind: "birth".to_string(),
            message: match faction_id {
                Some(id) => format!("A new citizen was born in Faction {id}"),
                None => "A new citizen was born".to_string(),
            },
            faction_id,
        });
    }

    for _death in deaths_this_tick {
        events.push(GameEvent {
            tick,
            kind: "death".to_string(),
            message: "A citizen died".to_string(),
            faction_id: None,
        });
    }

    for disaster in disaster_events {
        events.push(GameEvent {
            tick: disaster.tick,
            kind: "disaster".to_string(),
            message: format!(
                "{} at ({:.2}, {:.2})",
                disaster.kind, disaster.x, disaster.y
            ),
            faction_id: None,
        });
    }

    for diplomacy in diplomacy_events {
        let kind = match diplomacy.kind {
            DiplomacyKind::TradeAgreement => "trade",
            DiplomacyKind::Conflict => "conflict",
            DiplomacyKind::Peace => "peace",
        };
        let message = match diplomacy.kind {
            DiplomacyKind::TradeAgreement => format!(
                "Trade Agreement between Faction {} and Faction {}",
                diplomacy.faction_a, diplomacy.faction_b
            ),
            DiplomacyKind::Conflict => format!(
                "Conflict between Faction {} and Faction {}",
                diplomacy.faction_a, diplomacy.faction_b
            ),
            DiplomacyKind::Peace => format!(
                "Peace declared between Faction {} and Faction {}",
                diplomacy.faction_a, diplomacy.faction_b
            ),
        };
        events.push(GameEvent {
            tick: diplomacy.tick,
            kind: kind.to_string(),
            message,
            faction_id: Some(diplomacy.faction_a),
        });
    }

    for node in tech_tree
        .iter()
        .filter(|node| node.unlocked && node.era_min == (sim.state.tick / 600) as u16)
    {
        events.push(GameEvent {
            tick,
            kind: "tech".to_string(),
            message: format!(
                "Era {} reached: {} technology unlocked",
                node.era_min, node.id
            ),
            faction_id: None,
        });
    }

    let mut mod_buses = sim.replay_log().mod_loaded_bus_at_tick(tick);
    if tick <= 1 {
        for bus in sim.replay_log().mod_loaded_bus_at_tick(0) {
            if !mod_buses.iter().any(|existing| existing == &bus) {
                mod_buses.push(bus.clone());
            }
        }
    }
    for bus in &mod_buses {
        let message = serde_json::from_str::<serde_json::Value>(bus)
            .ok()
            .and_then(|value| {
                value
                    .get("mod_name")
                    .and_then(|name| name.as_str())
                    .map(|name| format!("Mod loaded: {name}"))
            })
            .unwrap_or_else(|| "Mod loaded".to_string());
        events.push(GameEvent {
            tick,
            kind: "mod.loaded".to_string(),
            message,
            faction_id: None,
        });
    }

    for bus in sim.replay_log().session_saved_bus_at_tick(tick) {
        let message = serde_json::from_str::<serde_json::Value>(&bus)
            .ok()
            .and_then(|value| {
                value
                    .get("slot")
                    .and_then(|slot| slot.as_str())
                    .map(|slot| format!("Game saved to {slot}"))
            })
            .unwrap_or_else(|| "Game saved".to_string());
        events.push(GameEvent {
            tick,
            kind: "session.saved".to_string(),
            message,
            faction_id: None,
        });
    }

    for bus in sim.replay_log().mod_permission_violation_bus_at_tick(tick) {
        let message = serde_json::from_str::<serde_json::Value>(&bus)
            .ok()
            .and_then(|value| {
                let mod_id = value.get("mod_id").and_then(|id| id.as_str())?;
                let call = value.get("call").and_then(|call| call.as_str())?;
                Some(format!("Mod {mod_id} denied: {call}"))
            })
            .unwrap_or_else(|| "Mod permission denied".to_string());
        events.push(GameEvent {
            tick,
            kind: "mod.permission_violation".to_string(),
            message,
            faction_id: None,
        });
    }

    for building in buildings {
        if matches!(building.kind, BuildingKind::Residential) {
            events.push(GameEvent {
                tick,
                kind: "building".to_string(),
                message: format!(
                    "New Residential building in Faction {}",
                    building.faction_id
                ),
                faction_id: Some(building.faction_id),
            });
        }
    }

    events.sort_by(|a, b| {
        a.tick
            .cmp(&b.tick)
            .then_with(|| a.kind.cmp(&b.kind))
            .then_with(|| a.message.cmp(&b.message))
    });
    events
        .into_iter()
        .rev()
        .take(20)
        .collect::<Vec<_>>()
        .into_iter()
        .rev()
        .collect()
}

fn disaster_events(tick: u64, factions: &[Faction], buildings: &[Building]) -> Vec<DisasterEvent> {
    if tick == 0 || tick % 1000 != 0 {
        return Vec::new();
    }
    let roll = hash01(tick as f32 * 0.017);
    if roll < 0.25 {
        return vec![DisasterEvent {
            tick,
            kind: "Earthquake".to_string(),
            x: hash01(tick as f32 * 0.11) * 0.8 + 0.1,
            y: hash01(tick as f32 * 0.19 + 3.0) * 0.8 + 0.1,
            radius: 0.18,
            severity: 0.55,
        }];
    }
    if roll < 0.5 {
        let (x, y) = buildings
            .iter()
            .find(|building| building.kind == BuildingKind::Residential)
            .map(|building| (building.x, building.y))
            .unwrap_or_else(|| {
                (
                    hash01(tick as f32 * 0.07) * 0.8 + 0.1,
                    hash01(tick as f32 * 0.13 + 9.0) * 0.8 + 0.1,
                )
            });
        return vec![DisasterEvent {
            tick,
            kind: "Wildfire".to_string(),
            x,
            y,
            radius: 0.12,
            severity: 0.7,
        }];
    }
    if roll < 0.75 {
        let center = factions
            .first()
            .map(|faction| faction.capital)
            .unwrap_or([0.5, 0.5]);
        return vec![DisasterEvent {
            tick,
            kind: "Flood".to_string(),
            x: center[0],
            y: center[1],
            radius: 0.22,
            severity: 0.6,
        }];
    }
    vec![DisasterEvent {
        tick,
        kind: "Plague".to_string(),
        x: 0.5,
        y: 0.5,
        radius: 0.26,
        severity: 0.1,
    }]
}

fn hash01(value: f32) -> f32 {
    let hashed = (value * 12.9898).sin() * 43_758.547;
    hashed - hashed.floor()
}

fn faction_for_point(x: f32, y: f32) -> Option<u32> {
    factions(0)
        .into_iter()
        .min_by(|a, b| {
            let da = (x - a.capital[0]).powi(2) + (y - a.capital[1]).powi(2);
            let db = (x - b.capital[0]).powi(2) + (y - b.capital[1]).powi(2);
            da.partial_cmp(&db).unwrap_or(std::cmp::Ordering::Equal)
        })
        .map(|faction| faction.id)
}

fn default_law_db() -> LawDb {
    LawDb::load_ron(
        r#"(
            version: 0,
            laws: [
                (
                    id: "mass_conservation",
                    kind: Conservation,
                    era_min: 0,
                    inputs: [],
                    outputs: [],
                    losses: [],
                    dependencies: [],
                ),
                (
                    id: "steel",
                    kind: Material,
                    era_min: 4,
                    inputs: ["iron_ore", "coal"],
                    outputs: ["steel_ingot"],
                    losses: ["slag"],
                    dependencies: ["mass_conservation"],
                ),
                (
                    id: "fusion_power",
                    kind: FictionalExtension,
                    era_min: 9,
                    inputs: ["deuterium"],
                    outputs: ["energy"],
                    losses: ["helium_4"],
                    dependencies: ["mass_conservation"],
                ),
            ],
        )"#,
    )
    .expect("sample law db")
}

fn tech_tree(db: &LawDb, current_era: u16) -> Vec<TechNode> {
    let mut nodes = db
        .laws
        .iter()
        .map(|law| TechNode {
            id: law.id.clone(),
            kind: match law.kind {
                LawKind::Conservation => "Conservation".to_string(),
                LawKind::Material => "Material".to_string(),
                LawKind::FictionalExtension => "FictionalExtension".to_string(),
            },
            era_min: law.era_min,
            unlocked: current_era >= law.era_min,
        })
        .collect::<Vec<_>>();
    nodes.sort_by(|a, b| a.era_min.cmp(&b.era_min).then_with(|| a.id.cmp(&b.id)));
    nodes
}

fn economy_snapshot(
    sim: &Simulation,
    factions: &[Faction],
    trade_balances_this_tick: &std::collections::HashMap<u32, f64>,
) -> EconomySnapshot {
    let energy_budget = sim.state.energy_budget_joules.to_f64();
    let resources = &sim.state.resources;
    let faction_treasury = factions
        .iter()
        .map(|faction| {
            let name = sim
                .state
                .factions
                .get(&faction.id)
                .cloned()
                .unwrap_or_else(|| format!("Faction {}", faction.id));
            let balance = sim
                .state
                .faction_treasury
                .get(&faction.id)
                .map(|value| value.to_f64())
                .unwrap_or(0.0);
            FactionTreasury {
                id: faction.id,
                name,
                balance,
                trade_balance: *trade_balances_this_tick.get(&faction.id).unwrap_or(&0.0),
            }
        })
        .collect();

    let mut food_per_tick = 0.0;
    let wood_per_tick = 0.0;
    let mut metal_per_tick = 0.0;
    for (_, building) in sim.world.query::<&civ_engine::Building>().iter() {
        match building.building_type {
            civ_engine::BuildingType::Farm => food_per_tick += 10.0,
            civ_engine::BuildingType::Mine => metal_per_tick += 5.0,
            _ => {}
        }
    }

    let institutions = civ_server::jsonrpc::institutions_from_sim(sim)
        .into_iter()
        .map(|row| InstitutionRow {
            id: row.id,
            kind: row.kind.to_string(),
            balance_joules: row.balance_joules,
        })
        .collect();

    EconomySnapshot {
        energy_budget,
        faction_treasury,
        production_rates: ProductionRates {
            food_per_tick,
            wood_per_tick,
            metal_per_tick,
            energy_per_tick: energy_budget / 1000.0,
        },
        institutions,
        resources: ResourceSnapshot {
            food: resources.food.to_f64(),
            wood: resources.wood.to_f64(),
            metal: resources.metal.to_f64(),
            energy: resources.energy.to_f64(),
        },
    }
}

fn sample_civilians(sim: &Simulation) -> Vec<SampleCivilian> {
    sim.world
        .query::<&Citizen>()
        .iter()
        .take(8)
        .map(|(_, citizen)| SampleCivilian {
            age: citizen.age,
            health: citizen.health.to_f64(),
            ideology: citizen.ideology.to_f64(),
            welfare: citizen.welfare.to_f64(),
            job: None,
        })
        .collect()
}

fn civ_pins(sim: &Simulation) -> Vec<CivPin> {
    let mut pins = Vec::new();
    for (idx, (_, (_civilian, pos, vel))) in sim
        .world
        .query::<(&AgentCivilian, &Position3d, &Velocity)>()
        .iter()
        .enumerate()
    {
        let x = normalize_world_coord(pos.coord.x);
        let y = normalize_world_coord(pos.coord.z);
        pins.push(CivPin {
            idx: idx as u32,
            x,
            y,
            dx: vel.dx,
            dy: vel.dy,
            job: None,
        });
    }
    pins.sort_by_key(|pin| pin.idx);
    pins
}

fn assign_and_drift_housing(sim: &mut Simulation, buildings: &[Building]) {
    let world = &mut sim.world;
    let homes: Vec<_> = buildings
        .iter()
        .filter(|building| {
            matches!(building.kind, BuildingKind::Residential) && building.capacity > 0
        })
        .collect();
    let mut occupancy: std::collections::BTreeMap<u32, u32> = std::collections::BTreeMap::new();
    let mut home_lookup = std::collections::BTreeMap::new();
    for building in &homes {
        home_lookup.insert(building.id, (building.x, building.y));
    }

    for (_, (_civilian, pos, vel, needs)) in
        world.query_mut::<(&AgentCivilian, &Position3d, &mut Velocity, &Needs)>()
    {
        if needs.shelter <= 0.5 {
            continue;
        }
        let mut selected = None;
        for building in homes.iter() {
            let used = occupancy.get(&building.id).copied().unwrap_or(0);
            if used < building.capacity {
                selected = Some(*building);
                break;
            }
        }
        if let Some(home) = selected {
            occupancy
                .entry(home.id)
                .and_modify(|count| *count += 1)
                .or_insert(1);
            let home_pos = Position3d {
                coord: WorldCoord {
                    x: (home.x * civ_voxel::FIXED_SCALE as f32) as i64,
                    y: 0,
                    z: (home.y * civ_voxel::FIXED_SCALE as f32) as i64,
                },
            };
            let drifted = drift_toward_home(pos, &home_pos, *vel, needs.shelter);
            vel.dx = drifted.dx;
            vel.dy = drifted.dy;
        }
    }
}

fn factions(tick: u64) -> Vec<Faction> {
    let base_radius = 0.05 + (tick as f32 * 0.000_02).min(0.12);
    let capitals = [
        (0.22, 0.24, [214, 174, 110]),
        (0.76, 0.27, [112, 176, 122]),
        (0.27, 0.73, [103, 151, 214]),
        (0.72, 0.74, [184, 118, 196]),
    ];

    capitals
        .iter()
        .enumerate()
        .map(|(idx, (x, y, color))| Faction {
            id: idx as u32,
            color: *color,
            capital: [*x, *y],
            radius: (base_radius + idx as f32 * 0.018).min(0.3),
        })
        .collect()
}

fn normalize_world_coord(coord: i64) -> f32 {
    (coord as f32 / civ_voxel::FIXED_SCALE as f32).clamp(0.0, 1.0)
}

fn buildings(factions: &[Faction], tick: u64) -> Vec<Building> {
    let kinds = [
        BuildingKind::Residential,
        BuildingKind::Commercial,
        BuildingKind::Industrial,
        BuildingKind::Civic,
    ];
    let mut buildings = Vec::new();
    for faction in factions {
        for i in 0..3 {
            let idx = faction.id * 3 + i;
            let seed = u64::from(idx)
                .wrapping_mul(1_103_515_245)
                .wrapping_add(tick / 120);
            let x = wrap01(faction.capital[0] + noise_offset(seed, 0));
            let y = wrap01(faction.capital[1] + noise_offset(seed, 1));
            buildings.push(Building {
                id: idx,
                x,
                y,
                kind: kinds[(idx as usize) % kinds.len()].clone(),
                era: ((tick / 600) % 6) as u8,
                faction_id: faction.id,
                occupants: 0,
                capacity: match kinds[(idx as usize) % kinds.len()] {
                    BuildingKind::Residential => 4,
                    _ => 0,
                },
            });
        }
    }
    buildings
}

/// Placed airports / ports / hangars from ECS authoring (FR-CIV-UX-006).
fn merge_authoring_buildings(buildings: &mut Vec<Building>, sim: &Simulation) {
    use civ_engine::{grid_to_norm, BuildingType};

    for (idx, (_, building)) in sim
        .world
        .query::<&civ_engine::Building>()
        .iter()
        .enumerate()
    {
        let (x, y) = grid_to_norm(building.position);
        let (kind, id_base) = match building.building_type {
            BuildingType::CityCenter => (BuildingKind::Civic, 9_000_u32),
            BuildingType::Market => (BuildingKind::Commercial, 9_100_u32),
            BuildingType::Barracks => (BuildingKind::Industrial, 9_200_u32),
            _ => continue,
        };
        buildings.push(Building {
            id: id_base + idx as u32,
            x,
            y,
            kind,
            era: ((sim.state.tick / 600) % 6) as u8,
            faction_id: 0,
            occupants: 0,
            capacity: 0,
        });
    }
}

fn housing_snapshot(sim: &Simulation, buildings: &mut [Building]) -> HousingStats {
    let needy_count = sim
        .world
        .query::<(&AgentCivilian, &Needs)>()
        .iter()
        .filter(|(_, (_, needs))| needs.shelter > 0.5)
        .count() as u32;
    let total_capacity = buildings.iter().map(|building| building.capacity).sum();
    let occupied = needy_count.min(total_capacity);
    let homeless = needy_count.saturating_sub(total_capacity);
    let mut remaining = occupied;
    for building in buildings.iter_mut() {
        if building.capacity == 0 {
            building.occupants = 0;
            continue;
        }
        let assigned = remaining.min(building.capacity);
        building.occupants = assigned;
        remaining = remaining.saturating_sub(assigned);
    }
    let vacancy_rate = if total_capacity == 0 {
        0.0
    } else {
        (total_capacity.saturating_sub(occupied)) as f32 / total_capacity as f32
    };

    HousingStats {
        total_capacity,
        occupied,
        homeless,
        vacancy_rate,
    }
}

fn roads(buildings: &[Building]) -> Vec<Road> {
    let mut roads = Vec::new();
    let mut by_faction: std::collections::BTreeMap<u32, Vec<&Building>> =
        std::collections::BTreeMap::new();
    for building in buildings {
        by_faction
            .entry(building.faction_id)
            .or_default()
            .push(building);
    }

    for faction_buildings in by_faction.values_mut() {
        faction_buildings.sort_by_key(|building| building.id);
        for pair in faction_buildings.windows(2) {
            let from = pair[0];
            let to = pair[1];
            let distance = ((to.x - from.x).powi(2) + (to.y - from.y).powi(2)).sqrt();
            let kind = if distance < 0.03 {
                RoadKind::Trail
            } else if distance < 0.06 {
                RoadKind::Dirt
            } else if distance < 0.10 {
                RoadKind::Paved
            } else {
                RoadKind::Highway
            };
            let width = match kind {
                RoadKind::Trail => 0.2,
                RoadKind::Dirt => 0.4,
                RoadKind::Paved => 0.6,
                RoadKind::Highway => 1.0,
            };
            roads.push(Road {
                from: [from.x, from.y],
                to: [to.x, to.y],
                width,
                kind,
            });
        }
    }

    roads
}

fn trade_routes(factions: &[Faction], tick: u64) -> Vec<TradeRoute> {
    let goods = ["grain", "timber", "ore", "cloth", "salt", "tools"];
    let mut routes = Vec::new();
    for (idx, from) in factions.iter().enumerate() {
        for to in factions.iter().skip(idx + 1) {
            let goods_idx = ((tick / 180) as usize + idx + to.id as usize) % goods.len();
            let volume = 8.0 + (((tick / 30) as f32 + from.id as f32 + to.id as f32) % 16.0);
            routes.push(TradeRoute {
                from_faction: from.id,
                to_faction: to.id,
                goods: goods[goods_idx].to_string(),
                volume,
            });
        }
    }
    routes
}

fn apply_trade_routes(
    sim: &mut Simulation,
    factions: &[Faction],
    tick: u64,
) -> (f64, std::collections::HashMap<u32, f64>) {
    let routes = trade_routes(factions, tick);
    let diplomacy = sim
        .diplomacy_events()
        .iter()
        .map(|event| {
            (
                (
                    event.faction_a.min(event.faction_b),
                    event.faction_a.max(event.faction_b),
                ),
                event.kind,
            )
        })
        .collect::<std::collections::HashMap<_, _>>();

    let mut trade_volume_this_tick = 0.0;
    let mut trade_balances = std::collections::HashMap::new();
    for route in routes {
        let key = (
            route.from_faction.min(route.to_faction),
            route.from_faction.max(route.to_faction),
        );
        let Some(kind) = diplomacy.get(&key).copied() else {
            continue;
        };
        if !matches!(kind, DiplomacyKind::Peace | DiplomacyKind::TradeAgreement) {
            continue;
        }

        let resource = route_resource(&route.goods);
        let supply = resource_amount(&sim.state.resources, resource);
        let demand = resource_demand(&sim.state.resources, resource);
        let trade_price = 1.0 + (demand - supply) * 0.1;
        let quantity = f64::from(route.volume) * 0.5;
        let treasury_delta = f64::from(route.volume) * trade_price;

        adjust_resource(&mut sim.state.resources, resource, -quantity);
        adjust_treasury(
            &mut sim.state.faction_treasury,
            route.from_faction,
            treasury_delta,
        );
        *trade_balances.entry(route.from_faction).or_insert(0.0) += treasury_delta;
        adjust_resource(&mut sim.state.resources, resource, quantity);
        adjust_treasury(
            &mut sim.state.faction_treasury,
            route.to_faction,
            -treasury_delta,
        );
        *trade_balances.entry(route.to_faction).or_insert(0.0) -= treasury_delta;
        trade_volume_this_tick += f64::from(route.volume);
    }

    (trade_volume_this_tick, trade_balances)
}

fn route_resource(goods: &str) -> civ_engine::ResourceType {
    match goods {
        "grain" => civ_engine::ResourceType::Food,
        "timber" => civ_engine::ResourceType::Wood,
        "ore" | "tools" => civ_engine::ResourceType::Metal,
        "cloth" | "salt" => civ_engine::ResourceType::Energy,
        _ => civ_engine::ResourceType::Food,
    }
}

fn resource_amount(resources: &civ_engine::Resources, resource: civ_engine::ResourceType) -> f64 {
    match resource {
        civ_engine::ResourceType::Food => resources.food.to_f64(),
        civ_engine::ResourceType::Wood => resources.wood.to_f64(),
        civ_engine::ResourceType::Metal => resources.metal.to_f64(),
        civ_engine::ResourceType::Energy => resources.energy.to_f64(),
    }
}

fn resource_demand(resources: &civ_engine::Resources, resource: civ_engine::ResourceType) -> f64 {
    (1000.0 - resource_amount(resources, resource)).max(0.0)
}

fn fixed_from_f64(value: f64) -> civ_engine::Fixed {
    civ_engine::Fixed::from_raw((value * civ_engine::SCALE as f64).round() as i64)
}

fn adjust_resource(
    resources: &mut civ_engine::Resources,
    resource: civ_engine::ResourceType,
    delta: f64,
) {
    let delta = fixed_from_f64(delta);
    match resource {
        civ_engine::ResourceType::Food => resources.food += delta,
        civ_engine::ResourceType::Wood => resources.wood += delta,
        civ_engine::ResourceType::Metal => resources.metal += delta,
        civ_engine::ResourceType::Energy => resources.energy += delta,
    }
}

fn adjust_treasury(
    treasury: &mut std::collections::HashMap<u32, civ_engine::Fixed>,
    faction_id: u32,
    delta: f64,
) {
    if let Some(balance) = treasury.get_mut(&faction_id) {
        *balance += fixed_from_f64(delta);
    }
}

fn noise_offset(seed: u64, lane: u64) -> f32 {
    let mixed = seed
        .wrapping_mul(0x9E37_79B9_7F4A_7C15)
        .wrapping_add(lane.wrapping_mul(0xBF58_476D_1CE4_E5B9));
    let unit = ((mixed >> 40) as f32) / ((1u64 << 24) as f32);
    (unit - 0.5) * 0.10
}

fn wrap01(value: f32) -> f32 {
    value.rem_euclid(1.0)
}

async fn snapshot_handler(State(state): State<AppState>) -> Json<Option<Snapshot>> {
    Json(state.latest.read().await.clone())
}

async fn terrain_handler(State(state): State<AppState>, headers: HeaderMap) -> Response {
    let cache = &state.terrain_cache;
    if headers
        .get(header::IF_NONE_MATCH)
        .is_some_and(|value| value == cache.etag)
    {
        return (
            StatusCode::NOT_MODIFIED,
            [(header::ETAG, cache.etag.clone())],
        )
            .into_response();
    }

    Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, "application/json")
        .header(header::ETAG, cache.etag.clone())
        .header(header::CACHE_CONTROL, "public, max-age=31536000, immutable")
        .body(Body::from(cache.body.clone()))
        .expect("terrain response")
}

async fn place_voxel_handler(
    State(state): State<AppState>,
    Json(req): Json<PlaceVoxelReq>,
) -> Json<ControlOk> {
    let mut sim = state.sim.lock().await;
    sim.voxel_mut().write(
        WorldCoord {
            x: req.x,
            y: req.y,
            z: req.z,
        },
        MaterialId(req.material),
    );
    Json(ControlOk {
        ok: true,
        message: None,
    })
}

async fn spawn_civilian_handler(
    State(state): State<AppState>,
    Json(req): Json<SpawnCivilianReq>,
) -> Json<ControlOk> {
    let mut sim = state.sim.lock().await;
    let id = sim.state.tick.wrapping_add(1) ^ 0x00c0_ffee;
    let mut rng = sim.rng_mut().clone();
    let _ = spawn_civilian_at(&mut sim.world, id, req.faction, req.x, req.y, &mut rng);
    *sim.rng_mut() = rng;
    Json(ControlOk {
        ok: true,
        message: None,
    })
}

async fn spawn_entity_handler(
    State(state): State<AppState>,
    Json(req): Json<SpawnEntityReq>,
) -> Json<ControlOk> {
    let mut sim = state.sim.lock().await;
    match req.kind.as_str() {
        "civilian" => {
            let id = sim.state.tick.wrapping_add(1) ^ 0x00c0_ffee;
            let mut rng = sim.rng_mut().clone();
            let _ = spawn_civilian_at(&mut sim.world, id, req.faction, req.x, req.y, &mut rng);
            *sim.rng_mut() = rng;
        }
        "vehicle" => {
            use civ_engine::{spawn_military_at, UnitType};
            let _ = spawn_military_at(&mut sim.world, req.faction, req.x, req.y, UnitType::Knight);
            let mut military = state.military.lock().await;
            let id = sim.state.tick.wrapping_add(1) ^ 0xdeadbee_u64;
            military.push(MilitaryPin {
                id,
                x: req.x.clamp(0.0, 1.0),
                y: req.y.clamp(0.0, 1.0),
                unit_type: "Vehicle".to_string(),
                faction: req.faction,
                strength: 1.0,
            });
        }
        "airport" => {
            use civ_engine::spawn_airport_at;
            let _ = spawn_airport_at(&mut sim.world, req.x, req.y);
        }
        "port" => {
            use civ_engine::spawn_port_at;
            let _ = spawn_port_at(&mut sim.world, req.x, req.y);
        }
        "hangar" => {
            use civ_engine::spawn_hangar_at;
            let _ = spawn_hangar_at(&mut sim.world, req.x, req.y);
        }
        _ => {
            return Json(ControlOk {
                ok: false,
                message: Some(
                    "kind must be civilian, vehicle, airport, port, or hangar".to_string(),
                ),
            });
        }
    }
    Json(ControlOk {
        ok: true,
        message: None,
    })
}

async fn damage_handler(
    State(state): State<AppState>,
    Json(req): Json<DamageReq>,
) -> Json<ControlOk> {
    let mut sim = state.sim.lock().await;
    let event = DamageEvent {
        center: WorldCoord {
            x: req.x,
            y: req.y,
            z: req.z,
        },
        radius_voxels: req.radius,
        energy: req.energy,
    };
    sim.push_damage(event);
    Json(ControlOk {
        ok: true,
        message: None,
    })
}

async fn speed_handler(
    State(state): State<AppState>,
    Json(req): Json<SpeedReq>,
) -> Json<ControlOk> {
    if ![0u8, 1, 2, 4, 8].contains(&req.speed) {
        return Json(ControlOk {
            ok: false,
            message: Some("speed must be 0, 1, 2, 4, or 8".into()),
        });
    }
    state.speed.store(req.speed, Ordering::Relaxed);
    Json(ControlOk {
        ok: true,
        message: None,
    })
}

fn sanitize_save_filename(filename: &str) -> Result<String, String> {
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

fn save_path(dir: &Path, filename: &str) -> Result<PathBuf, String> {
    let name = sanitize_save_filename(filename)?;
    Ok(dir.join(format!("{name}.civsave.zst")))
}

fn record_save_metadata(
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
                match state
                    .save_db
                    .evict_autosaves(&state.session_id, u32::try_from(AUTOSAVE_RING_MAX).unwrap_or(u32::MAX))
                {
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

fn dir_size_bytes(dir: &Path) -> u64 {
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

fn legacy_replay_path(dir: &Path, filename: &str) -> Result<PathBuf, String> {
    let name = sanitize_save_filename(filename)?;
    Ok(dir.join(format!("{name}.civreplay")))
}

fn validate_production_slot(slot: &str) -> Result<(), String> {
    if PRODUCTION_SLOTS.contains(&slot) {
        Ok(())
    } else {
        Err(format!(
            "invalid slot {slot:?}; expected one of {}",
            PRODUCTION_SLOTS.join(", ")
        ))
    }
}

fn save_type_for_name(name: &str) -> &'static str {
    if PRODUCTION_SLOTS.contains(&name) {
        "slot"
    } else if name == "autosave" || name.starts_with("autosave-") {
        "auto"
    } else {
        "manual"
    }
}

fn is_autosave_name(name: &str) -> bool {
    name == "autosave" || name.starts_with("autosave-")
}

fn enforce_autosave_ring(dir: &Path) {
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

async fn save_handler(
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

async fn save_slot_handler(
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
    save_handler(
        State(state),
        Json(SaveReq {
            filename: req.slot,
        }),
    )
    .await
}

async fn load_slot_handler(
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
    load_handler(
        State(state),
        Json(SaveReq {
            filename: req.slot,
        }),
    )
    .await
}

async fn load_handler(
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

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn mod_type_label(kind: ModType) -> &'static str {
    match kind {
        ModType::Policy => "policy",
        ModType::Economic => "economic",
        ModType::Event => "event",
        ModType::Scenario => "scenario",
    }
}

fn catalog_entry_from_manifest(
    source: String,
    kind: &str,
    manifest: &civ_mod_host::ModManifest,
    installed_ids: &std::collections::HashSet<String>,
    signed: bool,
    author_pubkey_hex: Option<String>,
) -> ModCatalogEntry {
    ModCatalogEntry {
        source,
        id: manifest.meta.id.clone(),
        name: manifest.meta.name.clone(),
        version: manifest.meta.version.clone(),
        mod_type: mod_type_label(manifest.meta.mod_type).to_owned(),
        kind: kind.to_owned(),
        installed: installed_ids.contains(&manifest.meta.id),
        signed,
        author_pubkey_hex,
    }
}

fn civmod_catalog_entries(
    repo: &Path,
    dir: &Path,
    installed_ids: &std::collections::HashSet<String>,
    seen: &mut std::collections::HashSet<String>,
) -> Vec<ModCatalogEntry> {
    let mut entries = Vec::new();
    let Ok(read) = std::fs::read_dir(dir) else {
        return entries;
    };
    for entry in read.flatten() {
        let path = entry.path();
        if path.extension().and_then(|e| e.to_str()) != Some("civmod") {
            continue;
        }
        let source = path
            .strip_prefix(repo)
            .map(|p| p.to_string_lossy().replace('\\', "/"))
            .unwrap_or_else(|_| path.display().to_string());
        if !seen.insert(source.clone()) {
            continue;
        }
        if let Ok(manifest) = read_manifest_from_civmod(&path) {
            entries.push(catalog_entry_from_manifest(
                source,
                "civmod",
                &manifest,
                installed_ids,
                false,
                None,
            ));
        }
    }
    entries
}

fn scan_mod_catalog(mods_dir: &Path, installed_ids: &std::collections::HashSet<String>) -> Vec<ModCatalogEntry> {
    let repo = repo_root();
    let mut entries = Vec::new();
    let mut seen = std::collections::HashSet::new();

    entries.extend(civmod_catalog_entries(
        &repo,
        mods_dir,
        installed_ids,
        &mut seen,
    ));
    entries.extend(civmod_catalog_entries(
        &repo,
        &mods_dir.join("uploads"),
        installed_ids,
        &mut seen,
    ));
    entries.extend(civmod_catalog_entries(
        &repo,
        &mods_dir.join("publish"),
        installed_ids,
        &mut seen,
    ));
    entries.extend(remote_civmod_catalog_entries(
        &repo,
        mods_dir,
        installed_ids,
        &mut seen,
    ));

    for name in ["example-policy", "example-economic"] {
        let dir = mods_dir.join(name);
        let manifest_path = dir.join(CIVMOD_MANIFEST_NAME);
        if !manifest_path.is_file() {
            continue;
        }
        let source = format!("mods/{name}");
        if !seen.insert(source.clone()) {
            continue;
        }
        if let Ok(manifest) = load_manifest(&manifest_path) {
            entries.push(catalog_entry_from_manifest(
                source,
                "dir",
                &manifest,
                installed_ids,
                false,
                None,
            ));
        }
    }

    entries.sort_by(|a, b| a.source.cmp(&b.source));
    entries
}

fn resolve_install_source(source: &str, mods_dir: &Path) -> Result<String, String> {
    let trimmed = source.trim();
    if trimmed.is_empty() {
        return Err("source cannot be empty".into());
    }
    if trimmed.contains("..") {
        return Err("source must not contain '..'".into());
    }

    let normalized = trimmed.replace('\\', "/");
    if normalized.starts_with("mods/") {
        let path = repo_root().join(&normalized);
        if path.is_file() || path.is_dir() {
            return Ok(normalized);
        }
        return Err(format!("mod source not found: {normalized}"));
    }

    if normalized.ends_with(".civmod") {
        let path = mods_dir.join(
            normalized
                .trim_start_matches("mods/")
                .trim_start_matches('/'),
        );
        if path.is_file() {
            return Ok(format!("mods/{}", path.file_name().unwrap().to_string_lossy()));
        }
        return Err(format!("mod archive not found: {normalized}"));
    }

    let dir_name = normalized.trim_start_matches("mods/").trim_start_matches('/');
    let dir = mods_dir.join(dir_name);
    if dir.is_dir() {
        return Ok(format!("mods/{dir_name}"));
    }

    Err(format!("unknown mod source: {trimmed}"))
}

fn sanitize_mod_filename(filename: &str) -> Result<String, String> {
    let trimmed = filename.trim();
    if trimmed.is_empty() {
        return Err("filename cannot be empty".into());
    }
    if trimmed.contains('/') || trimmed.contains('\\') || trimmed.contains("..") {
        return Err("filename must be a simple name".into());
    }
    let base = trimmed.trim_end_matches(".civmod");
    if base.is_empty() {
        return Err("filename must have a name".into());
    }
    Ok(base.to_string())
}

fn mod_source_relative(path: &Path) -> String {
    path.strip_prefix(repo_root())
        .map(|p| p.to_string_lossy().replace('\\', "/"))
        .unwrap_or_else(|_| path.display().to_string())
}

fn resolve_repo_mod_path(source: &str) -> Result<PathBuf, String> {
    let trimmed = source.trim();
    if trimmed.is_empty() {
        return Err("source cannot be empty".into());
    }
    if trimmed.contains("..") {
        return Err("source must not contain '..'".into());
    }
    let normalized = trimmed.replace('\\', "/");
    if !normalized.starts_with("mods/") {
        return Err("source must be under mods/".into());
    }
    let path = repo_root().join(&normalized);
    if !path.is_file() {
        return Err(format!("mod source not found: {normalized}"));
    }
    Ok(path)
}

fn scan_published_mods(mods_dir: &Path) -> Vec<PublishedModEntry> {
    let publish_dir = mods_dir.join("publish");
    let repo = repo_root();
    let mut entries = Vec::new();
    let Ok(read) = std::fs::read_dir(&publish_dir) else {
        return entries;
    };
    for entry in read.flatten() {
        let path = entry.path();
        if path.extension().and_then(|e| e.to_str()) != Some("civmod") {
            continue;
        }
        let source = path
            .strip_prefix(&repo)
            .map(|p| p.to_string_lossy().replace('\\', "/"))
            .unwrap_or_else(|_| path.display().to_string());
        if let Ok(manifest) = read_manifest_from_civmod(&path) {
            entries.push(PublishedModEntry {
                id: manifest.meta.id.clone(),
                name: manifest.meta.name.clone(),
                version: manifest.meta.version.clone(),
                source,
            });
        }
    }
    entries.sort_by(|a, b| a.id.cmp(&b.id));
    entries
}

fn validate_remote_fetch_url(url: &str) -> Result<(), String> {
    let trimmed = url.trim();
    if trimmed.is_empty() {
        return Err("url cannot be empty".into());
    }
    let parsed = reqwest::Url::parse(trimmed).map_err(|err| format!("invalid url: {err}"))?;
    match parsed.scheme() {
        "http" | "https" => Ok(()),
        scheme => Err(format!("unsupported url scheme: {scheme}")),
    }
}

fn sanitize_remote_mod_id(mod_id: &str) -> Result<String, String> {
    let trimmed = mod_id.trim();
    if trimmed.is_empty() {
        return Err("mod_id cannot be empty".into());
    }
    if trimmed.contains("..") || trimmed.contains('/') || trimmed.contains('\\') {
        return Err("mod_id must not contain path separators or '..'".into());
    }
    let valid_id = trimmed.as_bytes().first().is_some_and(|b| b.is_ascii_lowercase())
        && trimmed.len() <= 64
        && trimmed
            .bytes()
            .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'-');
    if !valid_id {
        return Err(format!(
            "mod_id `{trimmed}` must match [a-z][a-z0-9-]{{0,63}}"
        ));
    }
    Ok(trimmed.to_owned())
}

fn url_hash_cache_id(url: &str) -> String {
    let digest = Sha256::digest(url.trim().as_bytes());
    digest[..8]
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect()
}

fn resolve_remote_cache_id(url: &str, mod_id: Option<&str>) -> Result<String, String> {
    validate_remote_fetch_url(url)?;
    if let Some(id) = mod_id {
        return sanitize_remote_mod_id(id);
    }
    Ok(format!("url-{}", url_hash_cache_id(url)))
}

fn remote_mod_cache_dir(mods_dir: &Path, cache_id: &str) -> PathBuf {
    mods_dir.join("remote").join(cache_id)
}

fn remote_mod_source_path(repo: &Path, cache_dir: &Path) -> String {
    cache_dir
        .join(REMOTE_MOD_ARCHIVE_NAME)
        .strip_prefix(repo)
        .map(|p| p.to_string_lossy().replace('\\', "/"))
        .unwrap_or_else(|_| {
            format!(
                "mods/remote/{}/{}",
                cache_dir
                    .file_name()
                    .and_then(|s| s.to_str())
                    .unwrap_or("unknown"),
                REMOTE_MOD_ARCHIVE_NAME
            )
        })
}

fn is_zip_payload(bytes: &[u8]) -> bool {
    bytes.len() >= 4 && bytes.starts_with(b"PK\x03\x04")
}

fn format_remote_mod_validation_error(err: civ_mod_host::ManifestError) -> String {
    let msg = err.to_string();
    if msg.contains("signature") || msg.contains("mod.wasm.sig") || msg.contains("author_pubkey_hex") {
        format!("civmod signature verification failed: {msg}")
    } else {
        format!("invalid civmod archive: {msg}")
    }
}

fn validate_remote_mod_bytes(
    bytes: &[u8],
    scratch_path: &Path,
    registry_entry: Option<&RemoteModRegistryEntry>,
) -> Result<(civ_mod_host::ModManifest, bool), String> {
    if bytes.is_empty() {
        return Err("downloaded payload is empty".into());
    }
    if bytes.len() > REMOTE_MOD_MAX_BYTES {
        return Err(format!(
            "downloaded payload exceeds {} byte limit",
            REMOTE_MOD_MAX_BYTES
        ));
    }
    if !is_zip_payload(bytes) {
        return Err("downloaded payload is not a zip/civmod archive".into());
    }
    if let Some(parent) = scratch_path.parent() {
        std::fs::create_dir_all(parent).map_err(|err| err.to_string())?;
    }
    std::fs::write(scratch_path, bytes).map_err(|err| err.to_string())?;
    match read_civmod_archive(scratch_path) {
        Ok((manifest, wasm)) => {
            let signed =
                manifest.meta.author_pubkey_hex.is_some() && wasm.is_some();
            match validate_remote_mod_against_registry(registry_entry, &manifest) {
                Ok(()) => Ok((manifest, signed)),
                Err(err) => {
                    let _ = std::fs::remove_file(scratch_path);
                    Err(err)
                }
            }
        }
        Err(err) => {
            let _ = std::fs::remove_file(scratch_path);
            Err(format_remote_mod_validation_error(err))
        }
    }
}

fn write_remote_mod_meta(cache_dir: &Path, meta: &RemoteModMeta) -> Result<(), String> {
    std::fs::create_dir_all(cache_dir).map_err(|err| err.to_string())?;
    let json = serde_json::to_string_pretty(meta).map_err(|err| err.to_string())?;
    std::fs::write(cache_dir.join(REMOTE_MOD_META_NAME), json).map_err(|err| err.to_string())
}

fn read_remote_mod_meta(cache_dir: &Path) -> Option<RemoteModMeta> {
    let path = cache_dir.join(REMOTE_MOD_META_NAME);
    let contents = std::fs::read_to_string(path).ok()?;
    serde_json::from_str(&contents).ok()
}

fn unix_timestamp_secs() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|duration| duration.as_secs())
        .unwrap_or(0)
}

fn persist_remote_mod_cache(
    mods_dir: &Path,
    cache_id: &str,
    url: &str,
    bytes: &[u8],
    registry_entry: Option<&RemoteModRegistryEntry>,
) -> Result<(PathBuf, String), String> {
    let cache_dir = remote_mod_cache_dir(mods_dir, cache_id);
    let archive_path = cache_dir.join(REMOTE_MOD_ARCHIVE_NAME);
    let (manifest, signed) = validate_remote_mod_bytes(bytes, &archive_path, registry_entry)?;
    let author_pubkey_hex = manifest.meta.author_pubkey_hex.clone();
    let meta = RemoteModMeta {
        id: cache_id.to_owned(),
        url: url.trim().to_owned(),
        fetched_at: unix_timestamp_secs(),
        signed,
        author_pubkey_hex,
    };
    write_remote_mod_meta(&cache_dir, &meta)?;
    let source = remote_mod_source_path(&repo_root(), &cache_dir);
    Ok((archive_path, source))
}

async fn download_remote_mod_payload(
    http: &reqwest::Client,
    url: &str,
) -> Result<Vec<u8>, String> {
    validate_remote_fetch_url(url)?;
    let response = http
        .get(url.trim())
        .send()
        .await
        .map_err(|err| format!("fetch request failed: {err}"))?;
    if !response.status().is_success() {
        return Err(format!("fetch returned HTTP {}", response.status()));
    }
    let bytes = response
        .bytes()
        .await
        .map_err(|err| format!("fetch read failed: {err}"))?;
    if bytes.len() > REMOTE_MOD_MAX_BYTES {
        return Err(format!(
            "downloaded payload exceeds {} byte limit",
            REMOTE_MOD_MAX_BYTES
        ));
    }
    Ok(bytes.to_vec())
}

fn remote_civmod_catalog_entries(
    repo: &Path,
    mods_dir: &Path,
    installed_ids: &std::collections::HashSet<String>,
    seen: &mut std::collections::HashSet<String>,
) -> Vec<ModCatalogEntry> {
    let remote_root = mods_dir.join("remote");
    let Ok(read) = std::fs::read_dir(&remote_root) else {
        return Vec::new();
    };
    let mut entries = Vec::new();
    for entry in read.flatten() {
        let dir = entry.path();
        if !dir.is_dir() {
            continue;
        }
        let archive = dir.join(REMOTE_MOD_ARCHIVE_NAME);
        if !archive.is_file() {
            continue;
        }
        let source = archive
            .strip_prefix(repo)
            .map(|p| p.to_string_lossy().replace('\\', "/"))
            .unwrap_or_else(|_| archive.display().to_string());
        if !seen.insert(source.clone()) {
            continue;
        }
        if let Ok(manifest) = read_manifest_from_civmod(&archive) {
            let cache_meta = read_remote_mod_meta(&dir);
            let signed = cache_meta.as_ref().map(|m| m.signed).unwrap_or(false);
            let author_pubkey_hex = cache_meta.and_then(|m| m.author_pubkey_hex);
            entries.push(catalog_entry_from_manifest(
                source,
                "civmod",
                &manifest,
                installed_ids,
                signed,
                author_pubkey_hex,
            ));
        }
    }
    entries
}

fn scan_remote_mod_cache(mods_dir: &Path) -> Vec<RemoteModEntry> {
    let remote_root = mods_dir.join("remote");
    let repo = repo_root();
    let Ok(read) = std::fs::read_dir(&remote_root) else {
        return Vec::new();
    };
    let mut entries = Vec::new();
    for entry in read.flatten() {
        let dir = entry.path();
        if !dir.is_dir() {
            continue;
        }
        let archive = dir.join(REMOTE_MOD_ARCHIVE_NAME);
        if !archive.is_file() {
            continue;
        }
        let meta = read_remote_mod_meta(&dir).unwrap_or_else(|| RemoteModMeta {
            id: dir
                .file_name()
                .and_then(|s| s.to_str())
                .unwrap_or("unknown")
                .to_owned(),
            url: String::new(),
            fetched_at: archive
                .metadata()
                .ok()
                .and_then(|meta| meta.modified().ok())
                .and_then(|time| time.duration_since(std::time::UNIX_EPOCH).ok())
                .map(|duration| duration.as_secs())
                .unwrap_or(0),
            signed: false,
            author_pubkey_hex: None,
        });
        let path = archive
            .strip_prefix(&repo)
            .map(|p| p.to_string_lossy().replace('\\', "/"))
            .unwrap_or_else(|_| archive.display().to_string());
        entries.push(RemoteModEntry {
            id: meta.id,
            path,
            fetched_at: meta.fetched_at,
            url: meta.url,
            signed: meta.signed,
            author_pubkey_hex: meta.author_pubkey_hex,
        });
    }
    entries.sort_by(|a, b| a.id.cmp(&b.id));
    entries
}

async fn upload_mod_handler(
    State(state): State<AppState>,
    Json(req): Json<UploadModReq>,
) -> Result<Json<UploadModResponse>, (StatusCode, Json<ControlOk>)> {
    let name = sanitize_mod_filename(&req.filename).map_err(|message| {
        (
            StatusCode::BAD_REQUEST,
            Json(ControlOk {
                ok: false,
                message: Some(message),
            }),
        )
    })?;
    let bytes = base64::engine::general_purpose::STANDARD
        .decode(req.data_base64.trim())
        .map_err(|err| {
            (
                StatusCode::BAD_REQUEST,
                Json(ControlOk {
                    ok: false,
                    message: Some(format!("invalid base64: {err}")),
                }),
            )
        })?;
    if bytes.is_empty() {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ControlOk {
                ok: false,
                message: Some("upload data cannot be empty".into()),
            }),
        ));
    }

    let uploads_dir = state.mods_dir.join("uploads");
    std::fs::create_dir_all(&uploads_dir).map_err(|err| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ControlOk {
                ok: false,
                message: Some(err.to_string()),
            }),
        )
    })?;
    let path = uploads_dir.join(format!("{name}.civmod"));
    std::fs::write(&path, &bytes).map_err(|err| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ControlOk {
                ok: false,
                message: Some(err.to_string()),
            }),
        )
    })?;

    read_manifest_from_civmod(&path).map_err(|err| {
        let _ = std::fs::remove_file(&path);
        (
            StatusCode::BAD_REQUEST,
            Json(ControlOk {
                ok: false,
                message: Some(format!("invalid civmod archive: {err}")),
            }),
        )
    })?;

    Ok(Json(UploadModResponse {
        ok: true,
        source: mod_source_relative(&path),
    }))
}

async fn publish_mod_handler(
    State(state): State<AppState>,
    Json(req): Json<PublishModReq>,
) -> Result<Json<PublishModResponse>, (StatusCode, Json<ControlOk>)> {
    let source_path = resolve_repo_mod_path(&req.source).map_err(|message| {
        (
            StatusCode::BAD_REQUEST,
            Json(ControlOk {
                ok: false,
                message: Some(message),
            }),
        )
    })?;
    let manifest = read_manifest_from_civmod(&source_path).map_err(|err| {
        (
            StatusCode::BAD_REQUEST,
            Json(ControlOk {
                ok: false,
                message: Some(format!("invalid civmod archive: {err}")),
            }),
        )
    })?;
    let id = manifest.meta.id.trim();
    if id.is_empty()
        || id.contains('/')
        || id.contains('\\')
        || id.contains("..")
    {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ControlOk {
                ok: false,
                message: Some("manifest mod id must be a simple name".into()),
            }),
        ));
    }

    let publish_dir = state.mods_dir.join("publish");
    std::fs::create_dir_all(&publish_dir).map_err(|err| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ControlOk {
                ok: false,
                message: Some(err.to_string()),
            }),
        )
    })?;
    let dest = publish_dir.join(format!("{id}.civmod"));
    std::fs::copy(&source_path, &dest).map_err(|err| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ControlOk {
                ok: false,
                message: Some(err.to_string()),
            }),
        )
    })?;

    Ok(Json(PublishModResponse {
        ok: true,
        published_source: mod_source_relative(&dest),
    }))
}

async fn list_published_mods_handler(
    State(state): State<AppState>,
) -> Json<Vec<PublishedModEntry>> {
    Json(scan_published_mods(state.mods_dir.as_ref()))
}

async fn list_mod_catalog_handler(
    State(state): State<AppState>,
) -> Json<Vec<ModCatalogEntry>> {
    let sim = state.sim.lock().await;
    let installed: std::collections::HashSet<String> = sim
        .mod_browser_entries()
        .into_iter()
        .map(|entry| entry.id)
        .collect();
    Json(scan_mod_catalog(state.mods_dir.as_ref(), &installed))
}

async fn install_mod_handler(
    State(state): State<AppState>,
    Json(req): Json<InstallModReq>,
) -> Result<Json<ControlOk>, (StatusCode, Json<ControlOk>)> {
    let rel = resolve_install_source(&req.source, state.mods_dir.as_ref()).map_err(|message| {
        (
            StatusCode::BAD_REQUEST,
            Json(ControlOk {
                ok: false,
                message: Some(message),
            }),
        )
    })?;
    let mut sim = state.sim.lock().await;
    let record = sim.install_mod_path(&rel).map_err(|err| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ControlOk {
                ok: false,
                message: Some(err.to_string()),
            }),
        )
    })?;
    Ok(Json(ControlOk {
        ok: true,
        message: Some(format!("installed {} ({})", record.mod_name, record.mod_id)),
    }))
}

async fn unload_mod_handler(
    State(state): State<AppState>,
    Json(req): Json<UnloadModReq>,
) -> Result<Json<ControlOk>, (StatusCode, Json<ControlOk>)> {
    let mut sim = state.sim.lock().await;
    let record = sim.unload_mod_by_id(&req.mod_id, "user_request").map_err(|message| {
        (
            StatusCode::BAD_REQUEST,
            Json(ControlOk {
                ok: false,
                message: Some(message),
            }),
        )
    })?;
    Ok(Json(ControlOk {
        ok: true,
        message: Some(format!("unloaded {} ({})", record.mod_name, record.mod_id)),
    }))
}

async fn reload_mod_handler(
    State(state): State<AppState>,
    Json(req): Json<ReloadModReq>,
) -> Result<Json<ControlOk>, (StatusCode, Json<ControlOk>)> {
    let mut sim = state.sim.lock().await;
    let record = sim.reload_mod_by_id(&req.mod_id).map_err(|message| {
        (
            StatusCode::BAD_REQUEST,
            Json(ControlOk {
                ok: false,
                message: Some(message),
            }),
        )
    })?;
    Ok(Json(ControlOk {
        ok: true,
        message: Some(format!("reloaded {} ({})", record.mod_name, record.mod_id)),
    }))
}

async fn fetch_mod_handler(
    State(state): State<AppState>,
    Json(req): Json<FetchModReq>,
) -> Result<Json<FetchModResponse>, (StatusCode, Json<ControlOk>)> {
    let registry = load_remote_mod_registry(state.mods_dir.as_ref());
    let registry_entry = validate_remote_fetch_against_registry(
        &registry,
        &req.url,
        req.mod_id.as_deref(),
    )
    .map_err(|message| {
        (
            StatusCode::BAD_REQUEST,
            Json(ControlOk {
                ok: false,
                message: Some(message),
            }),
        )
    })?;
    let cache_id = resolve_remote_cache_id(&req.url, req.mod_id.as_deref()).map_err(|message| {
        (
            StatusCode::BAD_REQUEST,
            Json(ControlOk {
                ok: false,
                message: Some(message),
            }),
        )
    })?;
    let bytes = download_remote_mod_payload(&state.http, &req.url)
        .await
        .map_err(|message| {
            (
                StatusCode::BAD_GATEWAY,
                Json(ControlOk {
                    ok: false,
                    message: Some(message),
                }),
            )
        })?;
    let (path, source) = persist_remote_mod_cache(
        state.mods_dir.as_ref(),
        &cache_id,
        &req.url,
        &bytes,
        registry_entry,
    )
    .map_err(|message| {
        (
            StatusCode::BAD_REQUEST,
            Json(ControlOk {
                ok: false,
                message: Some(message),
            }),
        )
    })?;
    Ok(Json(FetchModResponse {
        ok: true,
        id: cache_id,
        source,
        path: path.display().to_string(),
    }))
}

async fn list_remote_mods_handler(
    State(state): State<AppState>,
) -> Json<Vec<RemoteModEntry>> {
    Json(scan_remote_mod_cache(state.mods_dir.as_ref()))
}

async fn list_saves_handler(
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
                            (Some(autosave.id), Some(autosave.tick as u64), Some(autosave.created_at)),
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

async fn sse_handler(
    State(state): State<AppState>,
) -> Sse<impl Stream<Item = Result<Event, Infallible>>> {
    let rx = state.tx.subscribe();
    let stream = BroadcastStream::new(rx).filter_map(|item| async move {
        match item {
            Ok(snapshot) => match serde_json::to_string(&snapshot) {
                Ok(json) => Some(Ok(Event::default().event("snapshot").data(json))),
                Err(err) => {
                    warn!(?err, "failed to serialize snapshot");
                    None
                }
            },
            Err(err) => {
                warn!(?err, "snapshot stream closed");
                None
            }
        }
    });
    Sse::new(stream).keep_alive(KeepAlive::default())
}

#[cfg(test)]
mod api_tests {
    use super::*;
    use axum::{
        body::Body,
        http::{Request, StatusCode},
    };
    use std::time::{SystemTime, UNIX_EPOCH};
    use tower::ServiceExt;

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
    async fn post_control_save_and_load_round_trip() {
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
    async fn post_save_slot_round_trip() {
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
        assert!(slot_entry["save_id"].as_str().is_some_and(|id| !id.is_empty()));
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
    async fn autosave_ring_evicts_oldest_beyond_max() {
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
        assert!(saves_dir.join("autosave-ring-trigger.civsave.zst").is_file());
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
        assert!(entries.iter().any(|entry| entry["source"] == "mods/example-policy"));
        assert!(entries.iter().any(|entry| entry["source"] == "mods/example-economic"));
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
        assert!(
            sim.mod_browser_entries()
                .iter()
                .all(|entry| entry.id != "example-policy")
        );
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
        assert!(
            sim.mod_browser_entries()
                .iter()
                .any(|entry| entry.id == "example-policy")
        );
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
            entry.id == cache_id && entry.signed && entry.author_pubkey_hex.as_deref() == Some(pk_hex.as_str())
        }));

        let catalog = scan_mod_catalog(&mods_dir, &std::collections::HashSet::new());
        assert!(catalog.iter().any(|entry| {
            entry.source == source && entry.signed && entry.author_pubkey_hex.as_deref() == Some(pk_hex.as_str())
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
        assert!(
            validate_remote_fetch_against_registry(&registry, "https://evil.example/mod.civmod", None)
                .is_err()
        );
        assert!(
            validate_remote_fetch_against_registry(
                &registry,
                "https://mods.example.com/demo.civmod",
                Some("demo-mod")
            )
            .is_ok()
        );
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
        assert!(remote_list.iter().any(|entry| {
            entry.id == cache_id && entry.url == url && !entry.signed
        }));

        let catalog = scan_mod_catalog(&mods_dir, &std::collections::HashSet::new());
        assert!(catalog.iter().any(|entry| entry.source == source && entry.id == cache_id));

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

}
