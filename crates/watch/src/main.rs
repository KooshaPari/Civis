//! `civ-watch` - local hot-reload dashboard harness for Civis 3D.
//!
//! A background `Simulation` ticks at ~10 Hz and publishes JSON snapshots over
//! SSE at `GET /events` plus a polling endpoint at `GET /snapshot`. The static
//! dashboard build under `web/dashboard/dist` is served from `GET /`.

use std::{convert::Infallible, net::SocketAddr, sync::Arc, time::Duration};

use axum::{
    extract::State,
    response::{
        sse::{Event, KeepAlive, Sse},
        Json,
    },
    routing::get,
    Router,
};
use civ_engine::{Citizen, JobType, Simulation};
use civ_server::build_voxel_delta_frame;
use civ_voxel::MaterialId;
use futures::{stream::Stream, StreamExt};
use serde::Serialize;
use tokio::sync::{broadcast, RwLock};
use tokio_stream::wrappers::BroadcastStream;
use tower_http::{cors::CorsLayer, services::ServeDir};
use tracing::{info, warn};

#[derive(Debug, Clone, Serialize)]
struct SampleCivilian {
    age: u32,
    health: f64,
    ideology: f64,
    welfare: f64,
    job: Option<JobLabel>,
}

#[derive(Debug, Clone, Serialize)]
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
struct Snapshot {
    tick: u64,
    population: u64,
    voxel_dirty_count: usize,
    voxel_chunk_count: usize,
    sample_civilians: Vec<SampleCivilian>,
}

#[derive(Clone)]
struct AppState {
    latest: Arc<RwLock<Option<Snapshot>>>,
    tx: broadcast::Sender<Snapshot>,
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
    let state = AppState {
        latest: Arc::new(RwLock::new(None)),
        tx: tx.clone(),
    };

    tokio::spawn(simulation_worker(state.clone()));

    let app = Router::new()
        .route("/events", get(sse_handler))
        .route("/snapshot", get(snapshot_handler))
        .fallback_service(
            ServeDir::new("web/dashboard/dist").append_index_html_on_directories(true),
        )
        .with_state(state)
        .layer(CorsLayer::permissive());

    // Bindable port from CIV_WATCH_PORT (default 9090 — 8080 is reserved by
    // Windows dynamic port range on many systems).
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
    let mut sim = Simulation::with_seed(42);
    seed_voxels(&mut sim);

    let mut interval = tokio::time::interval(Duration::from_millis(100));
    loop {
        interval.tick().await;
        sim.tick();

        let snapshot = make_snapshot(&sim);
        *state.latest.write().await = Some(snapshot.clone());
        let _ = state.tx.send(snapshot);
    }
}

fn seed_voxels(sim: &mut Simulation) {
    for x in 0..8 {
        sim.voxel_mut().write(
            civ_voxel::WorldCoord {
                x: i64::from(x) * 1_000_000,
                y: 0,
                z: 0,
            },
            MaterialId(1),
        );
    }
}

fn make_snapshot(sim: &Simulation) -> Snapshot {
    let events = sim.last_tick_voxel_events();
    let sample_civilians = sample_civilians(sim);
    let _ = build_voxel_delta_frame(sim.state.tick, events, sim.voxel()).map_err(|err| {
        warn!(?err, "voxel frame build failed for current tick");
    });

    Snapshot {
        tick: sim.state.tick,
        population: sim.state.population,
        voxel_dirty_count: events.len(),
        voxel_chunk_count: sim.voxel().chunk_count(),
        sample_civilians,
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

async fn snapshot_handler(State(state): State<AppState>) -> Json<Option<Snapshot>> {
    Json(state.latest.read().await.clone())
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
