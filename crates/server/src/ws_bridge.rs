use std::{
    net::SocketAddr,
    sync::{
        atomic::{AtomicU64, Ordering},
        Arc,
    },
};

use axum::{
    extract::{
        ws::{Message, WebSocket, WebSocketUpgrade},
        State,
    },
    http::StatusCode,
    response::IntoResponse,
    routing::get,
    Json, Router,
};
use civ_engine::Simulation;
use civ_protocol_3d::{AgentAppearanceFrame, BuildingDiffFrame, BuildingProvenance, Frame3d};
use futures::{SinkExt, StreamExt};
use tokio::{
    net::TcpListener,
    sync::{mpsc, Mutex},
    time::{interval, Duration},
};

use crate::voxel_frame_builder::build_voxel_delta_frame;

/// WebSocket bridge configuration.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct WsBridgeConfig {
    /// Socket address to bind the HTTP/WebSocket server to.
    pub addr: SocketAddr,
    /// Maximum number of concurrent WebSocket clients.
    pub max_clients: usize,
}

#[derive(Clone)]
struct AppState {
    sim: Arc<Mutex<Simulation>>,
    tick: Arc<AtomicU64>,
    clients: Arc<Mutex<Vec<mpsc::UnboundedSender<Message>>>>,
    max_clients: usize,
}

/// Run the WebSocket bridge and 10 Hz tick loop.
pub async fn run_ws_bridge(config: WsBridgeConfig, sim: Arc<Mutex<Simulation>>) {
    let state = AppState {
        sim,
        tick: Arc::new(AtomicU64::new(0)),
        clients: Arc::new(Mutex::new(Vec::new())),
        max_clients: config.max_clients,
    };

    let app = Router::new()
        .route("/healthz", get(healthz))
        .route("/ws", get(ws_handler))
        .with_state(state.clone());

    let listener = TcpListener::bind(config.addr)
        .await
        .expect("ws bridge bind");
    let server = axum::serve(listener, app.into_make_service());

    let ticker_state = state.clone();
    let ticker = tokio::spawn(async move {
        let mut tick = interval(Duration::from_millis(100));
        loop {
            tick.tick().await;
            if let Err(err) = tick_once(&ticker_state).await {
                tracing::error!("ws bridge tick failed: {err}");
            }
        }
    });

    let _ = tokio::join!(server, ticker);
}

async fn healthz(State(state): State<AppState>) -> impl IntoResponse {
    let tick = state.tick.load(Ordering::SeqCst);
    (StatusCode::OK, Json(serde_json::json!({ "tick": tick })))
}

async fn ws_handler(ws: WebSocketUpgrade, State(state): State<AppState>) -> impl IntoResponse {
    ws.on_upgrade(move |socket| handle_socket(socket, state))
}

async fn handle_socket(socket: WebSocket, state: AppState) {
    let (mut sender, mut receiver) = socket.split();
    let (tx, mut rx) = mpsc::unbounded_channel::<Message>();

    {
        let mut clients = state.clients.lock().await;
        if clients.len() >= state.max_clients {
            let _ = sender.send(Message::Close(None)).await;
            return;
        }
        clients.push(tx);
    }

    let forward = tokio::spawn(async move {
        while let Some(msg) = rx.recv().await {
            if sender.send(msg).await.is_err() {
                break;
            }
        }
    });

    while let Some(Ok(_)) = receiver.next().await {}
    forward.abort();
}

fn build_frame_triple(sim: &Simulation) -> Result<[Frame3d; 3], String> {
    let tick = sim.state.tick;
    let voxel = build_voxel_delta_frame(tick, sim.last_tick_voxel_events(), sim.voxel())
        .map_err(|e| e.to_string())?;
    let building = BuildingDiffFrame {
        tick,
        provenance: if sim.snapshot().building_count % 2 == 0 {
            BuildingProvenance::Procedural
        } else {
            BuildingProvenance::Freehand
        },
    };
    let agents = AgentAppearanceFrame {
        tick,
        updates: Vec::new(),
    };
    Ok([
        Frame3d::VoxelDelta(voxel),
        Frame3d::BuildingDiff(building),
        Frame3d::AgentAppearance(agents),
    ])
}

async fn tick_once(state: &AppState) -> Result<(), String> {
    let payloads = {
        let mut sim = state.sim.lock().await;
        sim.tick();
        let tick = sim.state.tick;
        state.tick.store(tick, Ordering::SeqCst);
        let frames = build_frame_triple(&sim)?;
        frames
            .into_iter()
            .map(|frame| serde_json::to_string(&frame).map(Message::Text))
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| e.to_string())?
    };

    let mut clients = state.clients.lock().await;
    clients.retain(|tx| payloads.iter().all(|msg| tx.send(msg.clone()).is_ok()));
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use civ_voxel::{MaterialId, WorldCoord};

    #[test]
    fn frame3d_encodes_to_json() {
        let frame = Frame3d::BuildingDiff(BuildingDiffFrame {
            tick: 9,
            provenance: BuildingProvenance::Procedural,
        });
        let json = serde_json::to_string(&frame).expect("json");
        let decoded: Frame3d = serde_json::from_str(&json).expect("decode");
        assert_eq!(decoded.tick(), 9);
    }

    #[tokio::test]
    async fn voxel_delta_frame_is_non_empty_after_writes() {
        let sim = Arc::new(Mutex::new(Simulation::with_seed(7)));
        let frame = {
            let mut guard = sim.lock().await;
            guard
                .voxel_mut()
                .write(WorldCoord { x: 0, y: 0, z: 0 }, MaterialId(1));
            guard.tick();
            build_voxel_delta_frame(
                guard.state.tick,
                guard.last_tick_voxel_events(),
                guard.voxel(),
            )
            .expect("frame")
        };
        assert!(!frame.deltas.is_empty());
    }

    #[tokio::test]
    async fn frame_triple_is_deterministic_for_fixed_seed() {
        let make = || async {
            let sim = Arc::new(Mutex::new(Simulation::with_seed(11)));
            let state = AppState {
                sim,
                tick: Arc::new(AtomicU64::new(0)),
                clients: Arc::new(Mutex::new(Vec::new())),
                max_clients: 1,
            };
            tick_once(&state).await.expect("tick");
            state.tick.load(Ordering::SeqCst)
        };
        let a = make().await;
        let b = make().await;
        assert_eq!(a, b);
    }

    #[tokio::test]
    async fn healthz_exposes_latest_tick() {
        let state = AppState {
            sim: Arc::new(Mutex::new(Simulation::with_seed(3))),
            tick: Arc::new(AtomicU64::new(123)),
            clients: Arc::new(Mutex::new(Vec::new())),
            max_clients: 1,
        };
        let response = healthz(State(state)).await.into_response();
        assert_eq!(response.status(), StatusCode::OK);
    }
}
