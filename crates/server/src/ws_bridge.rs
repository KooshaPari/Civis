use std::{
    net::SocketAddr,
    path::PathBuf,
    sync::{
        atomic::{AtomicU32, AtomicU64, Ordering},
        Arc,
    },
};

use axum::{
    body::Bytes,
    extract::{
        ws::{Message, WebSocket, WebSocketUpgrade},
        State,
    },
    http::{header, HeaderMap, StatusCode},
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};
use civ_agents::{Tools, Wardrobe};
use civ_engine::{decode_civreplay, encode_civreplay, Citizen, CivSaveBundle, Simulation};
use civ_protocol_3d::{
    encode_frame3d_binary, encode_frame3d_binary_from_json, AgentAppearanceFrame,
    AgentAppearanceUpdate, BuildingDiffFrame, BuildingProvenance, Frame3d,
};
use civ_save_db::SaveDb;
use futures::{SinkExt, StreamExt};
use tokio::{
    net::TcpListener,
    sync::{mpsc, Mutex},
    time::{interval, Duration},
};

use crate::{
    jsonrpc::{
        dispatch_request, encode_response, error_code, parse_error_response, parse_request,
        parse_role_param, set_sim_command_tick, set_spawn_civilian_result, DispatchContext,
        DispatchEffect, JsonRpcError, JsonRpcMethod, JsonRpcResponse,
    },
    saves::save_archive_path,
    voxel_frame_builder::build_voxel_delta_frame,
};

/// Which wire encodings the 10 Hz tick loop broadcasts to connected clients.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum TickBroadcastFormat {
    /// JSON text WebSocket frames only (legacy clients).
    Text,
    /// `F3D0`-prefixed binary frames only.
    Binary,
    /// JSON text frames followed by matching binary frames (default).
    #[default]
    Both,
}

impl TickBroadcastFormat {
    /// Whether tick broadcast includes JSON text WebSocket frames.
    #[must_use]
    pub fn sends_text(self) -> bool {
        matches!(self, Self::Text | Self::Both)
    }

    /// Whether tick broadcast includes `F3D0` binary WebSocket frames.
    #[must_use]
    pub fn sends_binary(self) -> bool {
        matches!(self, Self::Binary | Self::Both)
    }

    /// WebSocket frames emitted per simulation tick (three `Frame3d` values).
    #[must_use]
    pub fn messages_per_tick(self) -> usize {
        let kinds = 3;
        kinds * usize::from(self.sends_text()) + kinds * usize::from(self.sends_binary())
    }

    /// Parse `CIVIS_TICK_BROADCAST` values: `text`, `binary`, or `both` (case-insensitive).
    #[must_use]
    pub fn parse_env(value: &str) -> Option<Self> {
        match value.trim().to_ascii_lowercase().as_str() {
            "text" => Some(Self::Text),
            "binary" => Some(Self::Binary),
            "both" => Some(Self::Both),
            _ => None,
        }
    }

    /// Read [`TickBroadcastFormat`] from `CIVIS_TICK_BROADCAST`, defaulting to [`Self::Both`].
    #[must_use]
    pub fn from_env() -> Self {
        std::env::var("CIVIS_TICK_BROADCAST")
            .ok()
            .and_then(|value| Self::parse_env(&value))
            .unwrap_or_default()
    }
}

/// WebSocket bridge configuration.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WsBridgeConfig {
    /// Socket address to bind the HTTP/WebSocket server to.
    pub addr: SocketAddr,
    /// Maximum number of concurrent WebSocket clients.
    pub max_clients: usize,
    /// When true, `sim.command` tick requires role `operator` in params or connect header.
    pub require_role: bool,
    /// Tick broadcast wire encoding(s) for connected clients.
    ///
    /// Use [`TickBroadcastFormat::Binary`] to skip redundant JSON text frames and
    /// serialize each `Frame3d` once (inside the `F3D0` envelope only).
    pub tick_broadcast_format: TickBroadcastFormat,
    /// Directory for `.civsave.zst` slot files (created on bridge start).
    pub saves_dir: PathBuf,
}

impl Default for WsBridgeConfig {
    fn default() -> Self {
        Self {
            addr: SocketAddr::from(([127, 0, 0, 1], 3000)),
            max_clients: 16,
            require_role: false,
            tick_broadcast_format: TickBroadcastFormat::default(),
            saves_dir: PathBuf::from("saves"),
        }
    }
}

type TickBroadcastTx = mpsc::UnboundedSender<Arc<[Message]>>;

fn resolve_session_id() -> String {
    std::env::var("CIVIS_SESSION_ID")
        .ok()
        .filter(|value| !value.trim().is_empty())
        .unwrap_or_else(|| uuid::Uuid::new_v4().to_string())
}

fn save_db_path_for_saves_dir(saves_dir: &std::path::Path) -> PathBuf {
    saves_dir
        .parent()
        .map(std::path::Path::to_path_buf)
        .unwrap_or_else(|| PathBuf::from("."))
        .join("saves.db")
}

#[derive(Clone)]
struct AppState {
    sim: Arc<Mutex<Simulation>>,
    tick: Arc<AtomicU64>,
    speed_multiplier: Arc<AtomicU32>,
    clients: Arc<Mutex<Vec<TickBroadcastTx>>>,
    max_clients: usize,
    require_role: bool,
    tick_broadcast_format: TickBroadcastFormat,
    saves_dir: PathBuf,
    save_db: Arc<SaveDb>,
    session_id: String,
}

/// Run the WebSocket bridge and 10 Hz tick loop.
pub async fn run_ws_bridge(config: WsBridgeConfig, sim: Arc<Mutex<Simulation>>) {
    let listener = TcpListener::bind(config.addr)
        .await
        .expect("ws bridge bind");
    serve_ws_bridge(listener, config, sim).await;
}

/// Bind an ephemeral port, spawn the bridge, and return the listening address.
pub async fn spawn_ws_bridge(sim: Arc<Mutex<Simulation>>, max_clients: usize) -> SocketAddr {
    spawn_ws_bridge_with_config(
        sim,
        WsBridgeConfig {
            addr: SocketAddr::from(([127, 0, 0, 1], 0)),
            max_clients,
            require_role: false,
            tick_broadcast_format: TickBroadcastFormat::Both,
            ..Default::default()
        },
    )
    .await
}

/// Bind an ephemeral port with full bridge config (except `addr`, which is ignored).
pub async fn spawn_ws_bridge_with_config(
    sim: Arc<Mutex<Simulation>>,
    config: WsBridgeConfig,
) -> SocketAddr {
    let listener = TcpListener::bind("127.0.0.1:0")
        .await
        .expect("ws bridge bind");
    let addr = listener.local_addr().expect("ws bridge local addr");
    tokio::spawn(serve_ws_bridge(listener, config, sim));
    addr
}

async fn serve_ws_bridge(
    listener: TcpListener,
    config: WsBridgeConfig,
    sim: Arc<Mutex<Simulation>>,
) {
    std::fs::create_dir_all(&config.saves_dir).expect("create saves directory");
    let save_db_path = save_db_path_for_saves_dir(&config.saves_dir);
    let save_db = Arc::new(
        SaveDb::open(&save_db_path)
            .unwrap_or_else(|err| panic!("open save db at {save_db_path:?}: {err}")),
    );
    let session_id = resolve_session_id();
    tracing::info!(%session_id, ?save_db_path, "session-scoped save metadata db ready");
    let state = AppState {
        sim,
        tick: Arc::new(AtomicU64::new(0)),
        speed_multiplier: Arc::new(AtomicU32::new(1)),
        clients: Arc::new(Mutex::new(Vec::new())),
        max_clients: config.max_clients,
        require_role: config.require_role,
        tick_broadcast_format: config.tick_broadcast_format,
        saves_dir: config.saves_dir,
        save_db,
        session_id,
    };

    let app = Router::new()
        .route("/healthz", get(healthz))
        .route("/replay/export", get(replay_export))
        .route("/replay/import", post(replay_import))
        .route("/ws", get(ws_handler))
        .with_state(state.clone());

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

/// Load a `.civreplay` byte buffer into the bridge simulation.
async fn replay_import(
    State(state): State<AppState>,
    body: Bytes,
) -> Result<impl IntoResponse, StatusCode> {
    let log = decode_civreplay(&body).map_err(|_| StatusCode::BAD_REQUEST)?;
    let mut loaded = Simulation::with_seed(log.seed);
    log.replay(&mut loaded)
        .map_err(|_| StatusCode::BAD_REQUEST)?;
    let tick = loaded.state.tick;
    *state.sim.lock().await = loaded;
    state.tick.store(tick, Ordering::SeqCst);
    Ok((
        StatusCode::OK,
        Json(serde_json::json!({ "tick": tick, "ok": true })),
    ))
}

/// Export the current in-memory replay as `.civreplay` bytes (no filesystem path).
async fn replay_export(State(state): State<AppState>) -> Result<impl IntoResponse, StatusCode> {
    let bytes = {
        let sim = state.sim.lock().await;
        encode_civreplay(sim.replay_log()).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
    };
    Ok((
        StatusCode::OK,
        [
            (header::CONTENT_TYPE, "application/octet-stream"),
            (
                header::CONTENT_DISPOSITION,
                "attachment; filename=\"replay.civreplay\"",
            ),
        ],
        bytes,
    ))
}

async fn ws_handler(
    ws: WebSocketUpgrade,
    State(state): State<AppState>,
    headers: HeaderMap,
) -> impl IntoResponse {
    let header_role = headers
        .get("x-civis-role")
        .and_then(|value| value.to_str().ok())
        .filter(|role| !role.is_empty())
        .map(str::to_owned);
    ws.on_upgrade(move |socket| handle_socket(socket, state, header_role))
}

async fn handle_socket(socket: WebSocket, state: AppState, mut connection_role: Option<String>) {
    let (mut sender, mut receiver) = socket.split();
    let (tx, mut rx) = mpsc::unbounded_channel::<Arc<[Message]>>();

    {
        let mut clients = state.clients.lock().await;
        if clients.len() >= state.max_clients {
            let _ = sender.send(Message::Close(None)).await;
            return;
        }
        clients.push(tx.clone());
    }

    let forward = tokio::spawn(async move {
        while let Some(batch) = rx.recv().await {
            for msg in batch.iter() {
                if sender.send(msg.clone()).await.is_err() {
                    return;
                }
            }
        }
    });

    while let Some(msg) = receiver.next().await {
        match msg {
            Ok(Message::Text(text)) => {
                let response = handle_jsonrpc_text(&text, &state, &mut connection_role).await;
                let batch: Arc<[Message]> = Arc::from([Message::Text(response)]);
                if tx.send(batch).is_err() {
                    break;
                }
            }
            Ok(Message::Close(_)) => break,
            Ok(_) => {}
            Err(_) => break,
        }
    }
    forward.abort();
}

async fn handle_jsonrpc_text(
    text: &str,
    state: &AppState,
    connection_role: &mut Option<String>,
) -> String {
    match parse_request(text) {
        Ok(req) => {
            if connection_role.is_none() {
                if let Some(role) = parse_role_param(req.params.as_ref()) {
                    *connection_role = Some(role);
                }
            }
            let tick = state.tick.load(Ordering::SeqCst);
            let (population, snapshot) = match req.method {
                JsonRpcMethod::SimStatus => {
                    let sim = state.sim.lock().await;
                    let snap = sim.snapshot();
                    (Some(snap.population), None)
                }
                JsonRpcMethod::SimSnapshot => {
                    let sim = state.sim.lock().await;
                    let speed_multiplier = state.speed_multiplier.load(Ordering::Relaxed);
                    (
                        None,
                        Some(crate::jsonrpc::snapshot_fields_from_sim(
                            &sim,
                            speed_multiplier,
                        )),
                    )
                }
                _ => (None, None),
            };
            let mut plan = dispatch_request(
                req,
                DispatchContext {
                    tick,
                    population,
                    snapshot,
                    require_role: state.require_role,
                    speed_multiplier: state.speed_multiplier.load(Ordering::Relaxed),
                    connection_role: connection_role.clone(),
                    saves_dir: Some(state.saves_dir.clone()),
                },
            );
            apply_dispatch_effect(&mut plan.response, plan.effect, state).await;
            encode_response(&plan.response)
        }
        Err(err) => encode_response(&parse_error_response(text, err)),
    }
}

fn build_agent_appearance_frame(sim: &Simulation, tick: u64) -> AgentAppearanceFrame {
    let updates = sim
        .world
        .query::<(&Citizen, &Wardrobe, &Tools)>()
        .iter()
        .map(
            |(entity, (_citizen, wardrobe, tools))| AgentAppearanceUpdate {
                agent_id: u64::from(entity.id()),
                era: wardrobe.era,
                wardrobe: wardrobe.material,
                tools: tools.material,
                scale: 1.0,
            },
        )
        .collect();
    AgentAppearanceFrame { tick, updates }
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
    let agents = build_agent_appearance_frame(sim, tick);
    Ok([
        Frame3d::VoxelDelta(voxel),
        Frame3d::BuildingDiff(building),
        Frame3d::AgentAppearance(agents),
    ])
}

async fn apply_dispatch_effect(
    response: &mut JsonRpcResponse,
    effect: DispatchEffect,
    state: &AppState,
) {
    match effect {
        DispatchEffect::None => {}
        DispatchEffect::AdvanceTick => {
            if let Err(err) = advance_one_tick(state).await {
                tracing::error!("sim.command tick failed: {err}");
                set_replay_io_error(response, err);
            } else {
                let tick_after = state.tick.load(Ordering::SeqCst);
                set_sim_command_tick(response, tick_after);
            }
        }
        DispatchEffect::SaveReplay { path } => {
            let save_result = {
                let sim = state.sim.lock().await;
                sim.save_replay(&path)
            };
            if let Err(err) = save_result {
                tracing::error!("sim.save_replay failed: {err}");
                set_replay_io_error(response, err.to_string());
            }
        }
        DispatchEffect::LoadReplay { path } => match Simulation::load_replay_from_file(&path) {
            Ok(loaded) => {
                let tick = loaded.state.tick;
                *state.sim.lock().await = loaded;
                state.tick.store(tick, Ordering::SeqCst);
                if let Some(result) = response.result.as_mut() {
                    if let Some(obj) = result.as_object_mut() {
                        obj.insert("tick".to_owned(), serde_json::json!(tick));
                    }
                }
            }
            Err(err) => {
                tracing::error!("sim.load_replay failed: {err}");
                set_replay_io_error(response, err.to_string());
            }
        },
        DispatchEffect::ResetSimulation { seed } => {
            *state.sim.lock().await = Simulation::with_seed(seed);
            state.tick.store(0, Ordering::SeqCst);
        }
        DispatchEffect::SetPolicy {
            scarcity_multiplier,
            base_consumption_joules,
        } => {
            let mut sim = state.sim.lock().await;
            sim.economy_policy.scarcity_multiplier = scarcity_multiplier;
            if let Some(base) = base_consumption_joules {
                sim.economy_policy.base_consumption_joules = base as f64;
            }
            if let Some(result) = response.result.as_mut() {
                if let Some(obj) = result.as_object_mut() {
                    obj.insert(
                        "base_consumption_joules".to_owned(),
                        serde_json::json!(sim.economy_policy.base_consumption_joules),
                    );
                }
            }
        }
        DispatchEffect::SetSpeed { multiplier } => {
            state.speed_multiplier.store(multiplier, Ordering::Relaxed);
        }
        DispatchEffect::SpawnCivilian {
            x,
            y,
            faction,
            entity_seq,
        } => {
            let mut sim = state.sim.lock().await;
            let mut rng = sim.rng_mut().clone();
            let entity =
                civ_agents::spawn_civilian_at(&mut sim.world, entity_seq, faction, x, y, &mut rng);
            *sim.rng_mut() = rng;
            set_spawn_civilian_result(response, entity.id());
        }
        DispatchEffect::SpawnEntity {
            kind,
            x,
            y,
            faction,
            entity_seq,
        } => {
            use crate::jsonrpc::SpawnEntityKind;
            use civ_engine::{
                spawn_airport_at, spawn_hangar_at, spawn_military_at, spawn_port_at, UnitType,
            };

            let mut sim = state.sim.lock().await;
            let entity = match kind {
                SpawnEntityKind::Civilian => {
                    let mut rng = sim.rng_mut().clone();
                    let entity = civ_agents::spawn_civilian_at(
                        &mut sim.world,
                        entity_seq,
                        faction,
                        x,
                        y,
                        &mut rng,
                    );
                    *sim.rng_mut() = rng;
                    entity
                }
                SpawnEntityKind::Vehicle => {
                    spawn_military_at(&mut sim.world, faction, x, y, UnitType::Knight)
                }
                SpawnEntityKind::Airport => spawn_airport_at(&mut sim.world, x, y),
                SpawnEntityKind::Port => spawn_port_at(&mut sim.world, x, y),
                SpawnEntityKind::Hangar => spawn_hangar_at(&mut sim.world, x, y),
            };
            set_spawn_civilian_result(response, entity.id());
        }
        DispatchEffect::PlaceVoxel { x, y, z, material } => {
            let mut sim = state.sim.lock().await;
            sim.voxel_mut().write(
                civ_voxel::WorldCoord { x, y, z },
                civ_voxel::MaterialId(material),
            );
            if let Some(result) = response.result.as_mut() {
                if let Some(obj) = result.as_object_mut() {
                    obj.insert("ok".to_owned(), serde_json::json!(true));
                }
            }
        }
        DispatchEffect::ApplyDamage { event } => {
            let mut sim = state.sim.lock().await;
            sim.push_damage(event);
            if let Some(result) = response.result.as_mut() {
                if let Some(obj) = result.as_object_mut() {
                    obj.insert("ok".to_owned(), serde_json::json!(true));
                    obj.insert("queued".to_owned(), serde_json::json!(true));
                }
            }
        }
        DispatchEffect::SaveSlot { slot_name } => {
            let path = match save_archive_path(&state.saves_dir, &slot_name) {
                Ok(path) => path,
                Err(message) => {
                    set_replay_io_error(response, message);
                    return;
                }
            };
            let (save_result, tick) = {
                let sim = state.sim.lock().await;
                let tick = sim.state.tick;
                (CivSaveBundle::save_archive(&path, &sim), tick)
            };
            match save_result {
                Ok(()) => {
                    let byte_size = std::fs::metadata(&path).map(|meta| meta.len()).unwrap_or(0);
                    let file_path = path.display().to_string();
                    match state.save_db.record_slot_save(
                        &state.session_id,
                        &slot_name,
                        tick,
                        &file_path,
                        byte_size,
                    ) {
                        Ok(save_id) => {
                            let mut sim = state.sim.lock().await;
                            sim.record_session_saved(
                                &state.session_id,
                                &save_id,
                                &slot_name,
                                byte_size,
                            );
                        }
                        Err(err) => {
                            tracing::warn!(?err, "failed to record save metadata in save db");
                        }
                    }
                    if let Some(result) = response.result.as_mut() {
                        if let Some(obj) = result.as_object_mut() {
                            obj.insert("tick".to_owned(), serde_json::json!(tick));
                            obj.insert(
                                "path".to_owned(),
                                serde_json::json!(path.display().to_string()),
                            );
                        }
                    }
                }
                Err(err) => {
                    tracing::error!("save.slot failed: {err}");
                    set_replay_io_error(response, err.to_string());
                }
            }
        }
        DispatchEffect::LoadSlot { slot_name } => {
            let path = match save_archive_path(&state.saves_dir, &slot_name) {
                Ok(path) => path,
                Err(message) => {
                    set_replay_io_error(response, message);
                    return;
                }
            };
            match CivSaveBundle::load(&path) {
                Ok(loaded) => {
                    let tick = loaded.state.tick;
                    *state.sim.lock().await = loaded;
                    state.tick.store(tick, Ordering::SeqCst);
                    if let Some(result) = response.result.as_mut() {
                        if let Some(obj) = result.as_object_mut() {
                            obj.insert("tick".to_owned(), serde_json::json!(tick));
                        }
                    }
                }
                Err(err) => {
                    tracing::error!("save.load failed: {err}");
                    set_replay_io_error(response, err.to_string());
                }
            }
        }
    }
}

fn set_replay_io_error(response: &mut JsonRpcResponse, message: String) {
    let id = response.id.clone();
    *response = JsonRpcResponse::failure(
        id,
        JsonRpcError {
            code: error_code::INTERNAL_ERROR,
            message,
            data: None,
        },
    );
}

fn encode_tick_broadcast_messages(
    frames: [Frame3d; 3],
    format: TickBroadcastFormat,
) -> Result<Vec<Message>, String> {
    let mut payloads = Vec::with_capacity(format.messages_per_tick());
    let send_text = format.sends_text();
    let send_binary = format.sends_binary();

    if send_text && send_binary {
        let mut json_by_frame = Vec::with_capacity(frames.len());
        for frame in &frames {
            json_by_frame.push(serde_json::to_vec(frame).map_err(|e| e.to_string())?);
        }
        for json in &json_by_frame {
            let text = String::from_utf8(json.clone()).map_err(|e| e.to_string())?;
            payloads.push(Message::Text(text));
        }
        for (frame, json) in frames.iter().zip(json_by_frame.iter()) {
            let bytes =
                encode_frame3d_binary_from_json(frame, json).map_err(|e| format!("{e:?}"))?;
            payloads.push(Message::Binary(bytes));
        }
        return Ok(payloads);
    }

    for frame in &frames {
        if send_text {
            let text = serde_json::to_string(frame).map_err(|e| e.to_string())?;
            payloads.push(Message::Text(text));
        } else if send_binary {
            let bytes = encode_frame3d_binary(frame).map_err(|e| format!("{e:?}"))?;
            payloads.push(Message::Binary(bytes));
        }
    }

    Ok(payloads)
}

async fn advance_one_tick(state: &AppState) -> Result<(), String> {
    let batch = {
        let mut sim = state.sim.lock().await;
        sim.tick();
        let tick = sim.state.tick;
        state.tick.store(tick, Ordering::SeqCst);
        let frames = build_frame_triple(&sim)?;
        Arc::from(
            encode_tick_broadcast_messages(frames, state.tick_broadcast_format)?.into_boxed_slice(),
        )
    };

    let mut clients = state.clients.lock().await;
    clients.retain(|tx| tx.send(Arc::clone(&batch)).is_ok());
    Ok(())
}

async fn tick_once(state: &AppState) -> Result<(), String> {
    let multiplier = state.speed_multiplier.load(Ordering::Relaxed);
    if multiplier == 0 {
        return Ok(());
    }
    for _ in 0..multiplier {
        advance_one_tick(state).await?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use civ_save_db::SessionSaveRecord;
    use civ_voxel::{MaterialId, WorldCoord};

    fn test_app_state(
        sim: Arc<Mutex<Simulation>>,
        tick: u64,
        speed_multiplier: u32,
        require_role: bool,
    ) -> (tempfile::TempDir, AppState) {
        let dir = tempfile::tempdir().expect("tempdir");
        let saves_dir = dir.path().join("saves");
        std::fs::create_dir_all(&saves_dir).expect("saves dir");
        let save_db_path = save_db_path_for_saves_dir(&saves_dir);
        let save_db = Arc::new(SaveDb::open(&save_db_path).expect("open save db"));
        let state = AppState {
            sim,
            tick: Arc::new(AtomicU64::new(tick)),
            speed_multiplier: Arc::new(AtomicU32::new(speed_multiplier)),
            clients: Arc::new(Mutex::new(Vec::new())),
            max_clients: 1,
            require_role,
            tick_broadcast_format: TickBroadcastFormat::Both,
            saves_dir,
            save_db,
            session_id: "test-session".to_string(),
        };
        (dir, state)
    }

    #[test]
    fn tick_broadcast_both_sends_text_then_binary() {
        let frames = sample_frame_triple();
        let messages =
            encode_tick_broadcast_messages(frames, TickBroadcastFormat::Both).expect("encode");
        assert_eq!(
            messages.len(),
            TickBroadcastFormat::Both.messages_per_tick()
        );
        assert!(matches!(&messages[0], Message::Text(_)));
        assert!(matches!(&messages[1], Message::Text(_)));
        assert!(matches!(&messages[2], Message::Text(_)));
        assert!(matches!(&messages[3], Message::Binary(_)));
        if let Message::Binary(bytes) = &messages[3] {
            assert!(bytes.starts_with(civ_protocol_3d::FRAME3D_BINARY_MAGIC));
        }
    }

    #[test]
    fn tick_broadcast_format_parse_env() {
        assert_eq!(
            TickBroadcastFormat::parse_env("binary"),
            Some(TickBroadcastFormat::Binary)
        );
        assert_eq!(
            TickBroadcastFormat::parse_env("TEXT"),
            Some(TickBroadcastFormat::Text)
        );
        assert_eq!(
            TickBroadcastFormat::parse_env(" both "),
            Some(TickBroadcastFormat::Both)
        );
        assert_eq!(TickBroadcastFormat::parse_env("invalid"), None);
        assert_eq!(TickBroadcastFormat::default(), TickBroadcastFormat::Both);
    }

    #[test]
    fn tick_broadcast_message_count_per_format() {
        let frames = sample_frame_triple();
        for format in [
            TickBroadcastFormat::Text,
            TickBroadcastFormat::Binary,
            TickBroadcastFormat::Both,
        ] {
            let messages = encode_tick_broadcast_messages(frames.clone(), format).expect("encode");
            assert_eq!(
                messages.len(),
                format.messages_per_tick(),
                "{format:?} message count"
            );
        }
        assert_eq!(TickBroadcastFormat::Text.messages_per_tick(), 3);
        assert_eq!(TickBroadcastFormat::Binary.messages_per_tick(), 3);
        assert_eq!(TickBroadcastFormat::Both.messages_per_tick(), 6);
    }

    #[test]
    fn tick_broadcast_binary_only_skips_text_frames() {
        let frames = sample_frame_triple();
        let messages =
            encode_tick_broadcast_messages(frames, TickBroadcastFormat::Binary).expect("encode");
        assert_eq!(messages.len(), 3);
        assert!(messages.iter().all(|msg| matches!(msg, Message::Binary(_))));
    }

    #[test]
    fn tick_broadcast_both_binary_payload_matches_text_json() {
        let frames = sample_frame_triple();
        let messages =
            encode_tick_broadcast_messages(frames, TickBroadcastFormat::Both).expect("encode");
        let half = messages.len() / 2;
        for i in 0..half {
            let Message::Text(text) = &messages[i] else {
                panic!("expected text frame in first half");
            };
            let Message::Binary(bytes) = &messages[half + i] else {
                panic!("expected binary twin in second half");
            };
            let payload_len = u32::from_be_bytes([bytes[5], bytes[6], bytes[7], bytes[8]]) as usize;
            let payload = &bytes[9..9 + payload_len];
            assert_eq!(text.as_bytes(), payload);
        }
    }

    fn sample_frame_triple() -> [Frame3d; 3] {
        [
            Frame3d::BuildingDiff(BuildingDiffFrame {
                tick: 1,
                provenance: BuildingProvenance::Procedural,
            }),
            Frame3d::BuildingDiff(BuildingDiffFrame {
                tick: 1,
                provenance: BuildingProvenance::Freehand,
            }),
            Frame3d::AgentAppearance(AgentAppearanceFrame {
                tick: 1,
                updates: Vec::new(),
            }),
        ]
    }

    /// Manual probe: `cargo test -p civ-server tick_broadcast_encode_bench --release -- --ignored --nocapture`
    #[test]
    #[ignore = "manual perf probe"]
    fn tick_broadcast_encode_bench() {
        use std::time::Instant;

        let frames = sample_frame_triple();
        let iterations = 20_000u32;
        for format in [TickBroadcastFormat::Binary, TickBroadcastFormat::Both] {
            let start = Instant::now();
            for _ in 0..iterations {
                let _ = encode_tick_broadcast_messages(frames.clone(), format).expect("encode");
            }
            let elapsed = start.elapsed();
            eprintln!(
                "{format:?}: {iterations} ticks in {elapsed:?} ({:.0} encodes/sec, {} ws frames/tick)",
                iterations as f64 / elapsed.as_secs_f64(),
                format.messages_per_tick()
            );
        }
    }

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
            let (_dir, state) = test_app_state(sim, 0, 1, false);
            tick_once(&state).await.expect("tick");
            state.tick.load(Ordering::SeqCst)
        };
        let a = make().await;
        let b = make().await;
        assert_eq!(a, b);
    }

    #[tokio::test]
    async fn jsonrpc_invalid_json_returns_parse_error() {
        let sim = Arc::new(Mutex::new(Simulation::with_seed(9)));
        let (_dir, state) = test_app_state(sim, 0, 1, false);
        let mut connection_role = None;
        let text = handle_jsonrpc_text("{not json", &state, &mut connection_role).await;
        let value: serde_json::Value = serde_json::from_str(&text).expect("error response json");
        assert_eq!(value.get("jsonrpc").and_then(|v| v.as_str()), Some("2.0"));
        assert_eq!(value.get("id"), Some(&serde_json::Value::Null));
        assert_eq!(
            value.pointer("/error/code").and_then(|v| v.as_i64()),
            Some(-32_700)
        );
    }

    #[tokio::test]
    async fn jsonrpc_sim_status_reads_snapshot_population() {
        let sim = Arc::new(Mutex::new(Simulation::with_seed(13)));
        let population = {
            let guard = sim.lock().await;
            guard.snapshot().population
        };
        let (_dir, state) = test_app_state(sim, 5, 1, false);
        let mut connection_role = None;
        let text = handle_jsonrpc_text(
            r#"{"jsonrpc":"2.0","id":7,"method":"sim.status","params":{}}"#,
            &state,
            &mut connection_role,
        )
        .await;
        let value: serde_json::Value = serde_json::from_str(&text).expect("sim.status json");
        assert_eq!(value.get("id"), Some(&serde_json::json!(7)));
        assert_eq!(
            value.pointer("/result/tick").and_then(|v| v.as_u64()),
            Some(5)
        );
        assert_eq!(
            value.pointer("/result/population").and_then(|v| v.as_u64()),
            Some(population)
        );
    }

    #[tokio::test]
    async fn jsonrpc_sim_snapshot_reads_snapshot_fields() {
        let sim = Arc::new(Mutex::new(Simulation::with_seed(19)));
        {
            let mut guard = sim.lock().await;
            guard.tick();
        }
        let expected = {
            let guard = sim.lock().await;
            let snap = guard.snapshot();
            (
                snap.tick,
                snap.population,
                snap.building_count,
                snap.energy_budget.to_f64(),
                snap.market_prices.clone(),
                guard
                    .hash_chain_root()
                    .map(|root| civ_engine::hash_hex(&root))
                    .expect("hash chain root after tick"),
            )
        };
        let (_dir, state) = test_app_state(sim, 5, 4, false);
        let mut connection_role = None;
        let text = handle_jsonrpc_text(
            r#"{"jsonrpc":"2.0","id":8,"method":"sim.snapshot","params":{}}"#,
            &state,
            &mut connection_role,
        )
        .await;
        let value: serde_json::Value = serde_json::from_str(&text).expect("sim.snapshot json");
        assert_eq!(value.get("id"), Some(&serde_json::json!(8)));
        assert_eq!(
            value.pointer("/result/tick").and_then(|v| v.as_u64()),
            Some(expected.0)
        );
        assert_eq!(
            value.pointer("/result/population").and_then(|v| v.as_u64()),
            Some(expected.1)
        );
        assert_eq!(
            value
                .pointer("/result/building_count")
                .and_then(|v| v.as_u64()),
            Some(expected.2 as u64)
        );
        assert_eq!(
            value
                .pointer("/result/energy_budget")
                .and_then(|v| v.as_f64()),
            Some(expected.3)
        );
        assert_eq!(
            value.pointer("/result/market_prices").cloned(),
            serde_json::to_value(&expected.4).ok()
        );
        assert_eq!(
            value
                .pointer("/result/hash_chain_root")
                .and_then(|v| v.as_str()),
            Some(expected.5.as_str())
        );
        assert_eq!(
            value
                .pointer("/result/speed_multiplier")
                .and_then(|v| v.as_u64()),
            Some(4)
        );
    }

    #[tokio::test]
    async fn healthz_exposes_latest_tick() {
        let sim = Arc::new(Mutex::new(Simulation::with_seed(3)));
        let (_dir, state) = test_app_state(sim, 123, 1, false);
        let response = healthz(State(state)).await.into_response();
        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn replay_import_replaces_bridge_simulation() {
        let mut source = Simulation::with_seed(42);
        for _ in 0..5 {
            source.tick();
        }
        let bytes = encode_civreplay(source.replay_log()).expect("encode replay");
        let expected_tick = source.state.tick;

        let sim = Arc::new(Mutex::new(Simulation::with_seed(99)));
        let (_dir, state) = test_app_state(sim, 0, 1, false);
        let response = replay_import(State(state.clone()), bytes.into())
            .await
            .expect("replay import")
            .into_response();
        assert_eq!(response.status(), StatusCode::OK);

        let body = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .expect("response body");
        let value: serde_json::Value = serde_json::from_slice(&body).expect("json body");
        assert_eq!(value.get("ok"), Some(&serde_json::json!(true)));
        assert_eq!(
            value.get("tick").and_then(|v| v.as_u64()),
            Some(expected_tick)
        );
        assert_eq!(state.tick.load(Ordering::SeqCst), expected_tick);
        assert_eq!(state.sim.lock().await.state.tick, expected_tick);
    }

    #[tokio::test]
    async fn replay_export_sets_octet_stream_and_attachment_headers() {
        let sim = Arc::new(Mutex::new(Simulation::with_seed(31)));
        let (_dir, state) = test_app_state(sim, 0, 1, false);
        let response = replay_export(State(state))
            .await
            .expect("replay export")
            .into_response();
        assert_eq!(response.status(), StatusCode::OK);
        assert_eq!(
            response
                .headers()
                .get(header::CONTENT_TYPE)
                .and_then(|v| v.to_str().ok()),
            Some("application/octet-stream")
        );
        assert_eq!(
            response
                .headers()
                .get(header::CONTENT_DISPOSITION)
                .and_then(|v| v.to_str().ok()),
            Some("attachment; filename=\"replay.civreplay\"")
        );
    }

    #[tokio::test]
    async fn jsonrpc_sim_set_policy_updates_simulation_policy() {
        let sim = Arc::new(Mutex::new(Simulation::with_seed(5)));
        let (_dir, state) = test_app_state(sim.clone(), 0, 1, false);
        let mut connection_role = None;
        let text = handle_jsonrpc_text(
            r#"{"jsonrpc":"2.0","id":1,"method":"sim.set_policy","params":{"scarcity_multiplier":3.0,"base_consumption_joules":500}}"#,
            &state,
            &mut connection_role,
        )
        .await;
        let value: serde_json::Value = serde_json::from_str(&text).expect("set_policy json");
        assert_eq!(
            value.pointer("/result/updated"),
            Some(&serde_json::json!(true))
        );
        assert_eq!(
            value
                .pointer("/result/scarcity_multiplier")
                .and_then(|v| v.as_f64()),
            Some(3.0)
        );
        assert_eq!(
            value
                .pointer("/result/base_consumption_joules")
                .and_then(|v| v.as_f64()),
            Some(500.0)
        );
        let guard = sim.lock().await;
        assert_eq!(guard.economy_policy.scarcity_multiplier, 3.0);
        assert_eq!(guard.economy_policy.base_consumption_joules, 500.0);
    }

    #[tokio::test]
    async fn jsonrpc_sim_set_speed_stores_multiplier() {
        let sim = Arc::new(Mutex::new(Simulation::with_seed(5)));
        let (_dir, state) = test_app_state(sim, 0, 1, false);
        let mut connection_role = None;
        let text = handle_jsonrpc_text(
            r#"{"jsonrpc":"2.0","id":3,"method":"sim.set_speed","params":{"multiplier":4}}"#,
            &state,
            &mut connection_role,
        )
        .await;
        let value: serde_json::Value = serde_json::from_str(&text).expect("set_speed json");
        assert_eq!(
            value.pointer("/result/accepted"),
            Some(&serde_json::json!(true))
        );
        assert_eq!(
            value.pointer("/result/multiplier").and_then(|v| v.as_u64()),
            Some(4)
        );
        assert_eq!(state.speed_multiplier.load(Ordering::Relaxed), 4);
    }

    #[tokio::test]
    async fn jsonrpc_sim_get_speed_returns_stored_multiplier() {
        let sim = Arc::new(Mutex::new(Simulation::with_seed(5)));
        let (_dir, state) = test_app_state(sim, 0, 1, false);
        let mut connection_role = None;
        let set_text = handle_jsonrpc_text(
            r#"{"jsonrpc":"2.0","id":4,"method":"sim.set_speed","params":{"multiplier":8}}"#,
            &state,
            &mut connection_role,
        )
        .await;
        let set_value: serde_json::Value = serde_json::from_str(&set_text).expect("set_speed json");
        assert_eq!(
            set_value.pointer("/result/accepted"),
            Some(&serde_json::json!(true))
        );
        let get_text = handle_jsonrpc_text(
            r#"{"jsonrpc":"2.0","id":5,"method":"sim.get_speed"}"#,
            &state,
            &mut connection_role,
        )
        .await;
        let get_value: serde_json::Value = serde_json::from_str(&get_text).expect("get_speed json");
        assert_eq!(
            get_value
                .pointer("/result/multiplier")
                .and_then(|v| v.as_u64()),
            Some(8)
        );
    }

    #[tokio::test]
    async fn jsonrpc_sim_command_tick_rejects_wrong_role_when_enforced() {
        let sim = Arc::new(Mutex::new(Simulation::with_seed(9)));
        let (_dir, state) = test_app_state(sim, 0, 1, true);
        let mut connection_role = None;
        let text = handle_jsonrpc_text(
            r#"{"jsonrpc":"2.0","id":2,"method":"sim.command","params":{"action":"tick","role":"viewer"}}"#,
            &state,
            &mut connection_role,
        )
        .await;
        let value: serde_json::Value = serde_json::from_str(&text).expect("forbidden json");
        assert_eq!(
            value.pointer("/error/code").and_then(|v| v.as_i64()),
            Some(i64::from(error_code::FORBIDDEN))
        );
        assert_eq!(
            value
                .pointer("/error/data/required_role")
                .and_then(|v| v.as_str()),
            Some("operator")
        );
    }

    #[tokio::test]
    async fn jsonrpc_save_slot_records_save_db_and_replay_bus() {
        let sim = Arc::new(Mutex::new(Simulation::with_seed(7)));
        {
            let mut guard = sim.lock().await;
            guard.tick();
        }
        let saved_tick = sim.lock().await.state.tick;
        let (_dir, state) = test_app_state(sim.clone(), saved_tick, 1, false);
        let mut connection_role = None;
        let text = handle_jsonrpc_text(
            r#"{"jsonrpc":"2.0","id":70,"method":"save.slot","params":{"slot_name":"slot-1"}}"#,
            &state,
            &mut connection_role,
        )
        .await;
        let value: serde_json::Value = serde_json::from_str(&text).expect("save.slot json");
        assert_eq!(value.get("id"), Some(&serde_json::json!(70)));
        assert_eq!(
            value.pointer("/result/tick").and_then(|v| v.as_u64()),
            Some(saved_tick)
        );
        assert!(
            state.saves_dir.join("slot-1.civsave.zst").is_file(),
            "expected slot archive on disk"
        );

        let records = state
            .save_db
            .list_for_session("test-session")
            .expect("list save db");
        assert_eq!(records.len(), 1);
        let SessionSaveRecord::Slot(slot) = &records[0] else {
            panic!("expected slot record");
        };
        assert_eq!(slot.slot_name, "slot-1");
        assert_eq!(slot.tick, i64::try_from(saved_tick).unwrap_or(i64::MAX));
        assert!(slot.byte_size > 0);

        let guard = sim.lock().await;
        assert_eq!(
            guard
                .replay_log()
                .session_saved_bus_at_tick(saved_tick)
                .len(),
            1
        );
    }
}
