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
    spawn_civilian_at, tick_movement, Civilian as AgentCivilian, Position3d, Velocity,
};
use civ_engine::{Citizen, DiplomacyKind, JobType, Simulation};
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
}

#[derive(Debug, Clone, Serialize)]
struct Faction {
    id: u32,
    color: [u8; 3],
    capital: [f32; 2],
    radius: f32,
}

#[derive(Debug, Clone, Serialize)]
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
}

#[derive(Debug, Clone, Serialize)]
struct ProductionRates {
    food_per_tick: f64,
    wood_per_tick: f64,
    metal_per_tick: f64,
    energy_per_tick: f64,
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
    roads: Vec<Road>,
    trade_routes: Vec<TradeRoute>,
    economy: EconomySnapshot,
    births_this_tick: u32,
    deaths_this_tick: u32,
    diplomacy_events: Vec<DiplomacyPulse>,
    military_units: Vec<MilitaryPin>,
    damage_events: Vec<DamagePulse>,
    birth_events: Vec<PopulationPulse>,
    death_events: Vec<PopulationPulse>,
    tech_tree: Vec<TechNode>,
    is_day: bool,
    speed: u8,
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
        .route("/control/damage", post(damage_handler))
        .route("/control/speed", post(speed_handler))
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
            for _ in 0..speed {
                sim.tick();
                if sim.state.tick > 0 && sim.state.tick % 600 == 0 {
                    state
                        .target_era
                        .store(((sim.state.tick / 600).min(5)) as u16, Ordering::Relaxed);
                }
                let terrain = state.terrain.clone();
                let mut rng = sim.rng_mut().clone();
                tick_movement(&mut sim.world, 128, &mut rng, |x, y| {
                    terrain.is_walkable(x, y)
                });
                *sim.rng_mut() = rng;
                damage_events = tick_military(&mut sim, &terrain, &mut military);
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
    speed: u8,
    laws: &LawDb,
    current_era: u16,
) -> Snapshot {
    let events = sim.last_tick_voxel_events();
    let sample_civilians = sample_civilians(sim);
    let civ_pins = civ_pins(sim);
    let factions = factions(sim.state.tick);
    let buildings = buildings(&factions, sim.state.tick);
    let roads = roads(&buildings);
    let trade_routes = trade_routes(&factions, sim.state.tick);
    let economy = economy_snapshot(sim, &factions);
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
    let _ = build_voxel_delta_frame(sim.state.tick, events, sim.voxel()).map_err(|err| {
        warn!(?err, "voxel frame build failed for current tick");
    });
    let climate = sim.climate();
    let is_day = climate.day_phase >= 0.25 && climate.day_phase < 0.75;
    let tick_dt_ms = 100u32 / u32::from(speed.max(1));
    let tech_tree = tech_tree(laws, current_era);

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
        roads,
        trade_routes,
        economy,
        births_this_tick: birth_events.len() as u32,
        deaths_this_tick: death_events.len() as u32,
        diplomacy_events,
        military_units: military.to_vec(),
        damage_events: damage_events.to_vec(),
        birth_events,
        death_events,
        tech_tree,
        is_day,
        speed,
    }
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

fn economy_snapshot(sim: &Simulation, factions: &[Faction]) -> EconomySnapshot {
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
    sim.world
        .query::<(&AgentCivilian, &Position3d, &Velocity)>()
        .iter()
        .take(256)
        .enumerate()
        .map(|(idx, (_, (_citizen, pos, vel)))| {
            let x = pos.coord.x as f32 / civ_voxel::FIXED_SCALE as f32;
            let y = pos.coord.z as f32 / civ_voxel::FIXED_SCALE as f32;
            CivPin {
                idx: idx as u32,
                x,
                y,
                dx: vel.dx,
                dy: vel.dy,
                job: None,
            }
        })
        .collect()
}

fn factions(tick: u64) -> Vec<Faction> {
    let territory_radius_t = 18.0 + (tick as f32 * 0.018);
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
            radius: territory_radius_t + idx as f32 * 2.75,
        })
        .collect()
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
            });
        }
    }
    buildings
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

async fn damage_handler(
    State(state): State<AppState>,
    Json(req): Json<DamageReq>,
) -> Json<ControlOk> {
    let mut sim = state.sim.lock().await;
    sim.push_damage(DamageEvent {
        center: WorldCoord {
            x: req.x,
            y: req.y,
            z: req.z,
        },
        radius_voxels: req.radius,
        energy: req.energy,
    });
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
    use tower::ServiceExt;

    fn test_state() -> AppState {
        let (tx, _) = broadcast::channel::<Snapshot>(64);
        let sim = Arc::new(Mutex::new(Simulation::with_seed(42)));
        let terrain = Terrain::generate(42);
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
