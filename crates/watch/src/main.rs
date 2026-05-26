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
use civ_engine::{Citizen, DiplomacyKind, JobType, ModBrowserEntry, Simulation};
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

#[derive(Debug, Deserialize)]
struct SaveReq {
    filename: String,
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
    let saves_dir = Arc::new(PathBuf::from("saves"));
    std::fs::create_dir_all(&*saves_dir).expect("create saves dir");
    info!(
        "terrain: {0}x{0} = {1} cells generated",
        terrain.size,
        terrain.heights.len()
    );

    let sim = Arc::new(Mutex::new(Simulation::with_seed(42)));
    let military = Arc::new(Mutex::new(Vec::new()));
    {
        let mut s = sim.lock().await;
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
        .route("/control/load", post(load_handler))
        .route("/control/saves", get(list_saves_handler))
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
    Ok(trimmed.trim_end_matches(".civreplay").to_string())
}

fn save_path(dir: &Path, filename: &str) -> Result<PathBuf, String> {
    let name = sanitize_save_filename(filename)?;
    Ok(dir.join(format!("{name}.civreplay")))
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
    let sim = state.sim.lock().await;
    sim.save_replay(&path).map_err(|err| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ControlOk {
                ok: false,
                message: Some(err.to_string()),
            }),
        )
    })?;
    let tick = sim.state.tick;
    Ok(Json(SaveResponse {
        ok: true,
        path: path.display().to_string(),
        tick,
    }))
}

async fn load_handler(
    State(state): State<AppState>,
    Json(req): Json<SaveReq>,
) -> Result<Json<LoadResponse>, (StatusCode, Json<ControlOk>)> {
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
    let loaded = Simulation::load_replay_from_file(&path).map_err(|err| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ControlOk {
                ok: false,
                message: Some(err.to_string()),
            }),
        )
    })?;
    *sim = loaded;
    let tick = sim.state.tick;
    Ok(Json(LoadResponse { ok: true, tick }))
}

async fn list_saves_handler(
    State(state): State<AppState>,
) -> Result<Json<Vec<SaveListEntry>>, (StatusCode, Json<ControlOk>)> {
    let mut entries = Vec::new();
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
        if path.extension().and_then(|s| s.to_str()) != Some("civreplay") {
            continue;
        }
        let meta = match entry.metadata() {
            Ok(meta) => meta,
            Err(_) => continue,
        };
        let modified = meta
            .modified()
            .ok()
            .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
            .map(|d| d.as_secs());
        let name = path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("")
            .to_string();
        entries.push(SaveListEntry {
            name,
            size_bytes: meta.len(),
            modified,
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
}
