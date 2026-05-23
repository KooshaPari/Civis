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
        atomic::{AtomicU8, Ordering},
        Arc,
    },
    time::Duration,
};

use axum::{
    extract::State,
    response::{
        sse::{Event, KeepAlive, Sse},
        Json,
    },
    routing::{get, post},
    Router,
};
use civ_agents::{
    spawn_civilian as agents_spawn_civilian, Civilian as AgentCivilian, LodTier, Needs, Position3d,
    Tools, Wardrobe,
};
use civ_engine::{Citizen, JobType, Simulation};
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
struct Snapshot {
    tick: u64,
    tick_dt_ms: u32,
    population: u64,
    voxel_dirty_count: usize,
    voxel_chunk_count: usize,
    sample_civilians: Vec<SampleCivilian>,
    civ_pins: Vec<CivPin>,
    factions: Vec<Faction>,
    buildings: Vec<Building>,
    is_day: bool,
    speed: u8,
}

#[derive(Clone)]
struct AppState {
    latest: Arc<RwLock<Option<Snapshot>>>,
    tx: broadcast::Sender<Snapshot>,
    terrain: Arc<Terrain>,
    sim: Arc<Mutex<Simulation>>,
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
    let terrain = Arc::new(Terrain::generate(42));
    info!(
        "terrain: {0}x{0} = {1} cells generated",
        terrain.size,
        terrain.heights.len()
    );

    let sim = Arc::new(Mutex::new(Simulation::with_seed(42)));
    {
        let mut s = sim.lock().await;
        seed_voxels(&mut s);
    }

    let state = AppState {
        latest: Arc::new(RwLock::new(None)),
        tx: tx.clone(),
        terrain,
        sim,
        speed: Arc::new(AtomicU8::new(1)),
    };

    tokio::spawn(simulation_worker(state.clone()));

    let app = Router::new()
        .route("/events", get(sse_handler))
        .route("/snapshot", get(snapshot_handler))
        .route("/terrain", get(terrain_handler))
        .route("/control/place_voxel", post(place_voxel_handler))
        .route("/control/spawn_civilian", post(spawn_civilian_handler))
        .route("/control/damage", post(damage_handler))
        .route("/control/speed", post(speed_handler))
        .fallback_service(
            ServeDir::new("web/dashboard/dist").append_index_html_on_directories(true),
        )
        .with_state(state)
        .layer(CorsLayer::permissive());

    let port: u16 = std::env::var("CIV_WATCH_PORT")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(9090);
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
            for _ in 0..speed {
                sim.tick();
            }
            make_snapshot(&sim, speed)
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

fn make_snapshot(sim: &Simulation, speed: u8) -> Snapshot {
    let events = sim.last_tick_voxel_events();
    let sample_civilians = sample_civilians(sim);
    let civ_pins = civ_pins(sim);
    let factions = factions(sim.state.tick);
    let buildings = buildings(&factions, sim.state.tick);
    let _ = build_voxel_delta_frame(sim.state.tick, events, sim.voxel()).map_err(|err| {
        warn!(?err, "voxel frame build failed for current tick");
    });
    let climate = sim.climate();
    let is_day = climate.day_phase >= 0.25 && climate.day_phase < 0.75;
    let tick_dt_ms = 100u32 / u32::from(speed.max(1));

    Snapshot {
        tick: sim.state.tick,
        tick_dt_ms,
        population: sim.state.population,
        voxel_dirty_count: events.len(),
        voxel_chunk_count: sim.voxel().chunk_count(),
        sample_civilians,
        civ_pins,
        factions,
        buildings,
        is_day,
        speed,
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
            job: citizen.job.map(JobLabel::from),
        })
        .collect()
}

fn civ_pins(sim: &Simulation) -> Vec<CivPin> {
    sim.world
        .query::<&Citizen>()
        .iter()
        .take(256)
        .enumerate()
        .map(|(idx, (_, citizen))| {
            let seed = u64::from(idx as u32).wrapping_mul(2_654_435_761) ^ u64::from(citizen.age);
            let base_x = ((seed & 0xffff) as f32) / 65535.0;
            let base_y = (((seed >> 16) & 0xffff) as f32) / 65535.0;
            let angle = ((seed >> 32) as f32 / u32::MAX as f32) * std::f32::consts::TAU;
            let drift = 0.0015 + ((seed >> 48) as f32 / 65535.0) * 0.0025;
            let dx = angle.cos() * drift;
            let dy = angle.sin() * drift;
            let tick_phase = (sim.state.tick as f32) * 0.1;
            let x = wrap01(base_x + dx * tick_phase);
            let y = wrap01(base_y + dy * tick_phase);
            CivPin {
                idx: idx as u32,
                x,
                y,
                dx,
                dy,
                job: citizen.job.map(JobLabel::from),
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

async fn terrain_handler(State(state): State<AppState>) -> Json<Terrain> {
    Json((*state.terrain).clone())
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
    agents_spawn_civilian(
        &mut sim.world,
        AgentCivilian {
            id,
            faction: req.faction,
            age: 18,
        },
        Position3d {
            coord: WorldCoord {
                x: (req.x * 1_000_000.0) as i64,
                y: 0,
                z: (req.y * 1_000_000.0) as i64,
            },
        },
        Wardrobe {
            era: 0,
            material: MaterialId(0),
        },
        Tools {
            era: 0,
            material: MaterialId(0),
        },
        Needs {
            food: 0.25,
            shelter: 0.25,
            safety: 0.25,
            belonging: 0.25,
        },
        LodTier::Hot,
    );
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
