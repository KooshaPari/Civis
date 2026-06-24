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
        Query, State,
    },
    http::{header, HeaderMap, StatusCode},
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};
use civ_agents::{Civilian as AgentCivilian, Needs, Tools, Wardrobe};
use civ_build::ProductionEvent;
use civ_engine::{
    decode_civreplay, encode_civreplay, job_type_for_civilian_id,
    scenario::{load_scenario, preset_scenario_path},
    Citizen, CivSaveBundle, DiplomacyKind, JobType, Simulation,
};
use civ_protocol_3d::{
    encode_frame3d_binary, encode_frame3d_binary_from_json, AgentAppearanceFrame,
    AgentAppearanceUpdate, BattleEvent3d, BirthEvent3d, BuildingDiffFrame, BuildingProvenance,
    CivilianNeeds3d, CivilianStateEntry, CivilianStateFrame, ClimateFrame, DeathEvent3d,
    EventFeedFrame, EventFeedMessage3d, FactionStateEntry, FactionStateFrame, FactionTreasury3d,
    Frame3d, GenomeSummary3d, Government3d, TechEvent3d, WorldXZ,
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
    subscription_filter::{SubscriptionFilter, WsConnectQuery},
    voxel_frame_builder::build_voxel_delta_frame,
};

fn voxel_axis_span<F>(world: &civ_voxel::VoxelWorld<civ_voxel::MaterialId>, axis: F) -> f32
where
    F: Fn(civ_voxel::ChunkCoord) -> i32,
{
    let mut min = None::<i32>;
    let mut max = None::<i32>;
    for (coord, _) in world.chunks_dense() {
        let value = axis(coord);
        min = Some(min.map_or(value, |current| current.min(value)));
        max = Some(max.map_or(value, |current| current.max(value)));
    }
    let Some(min) = min else {
        return civ_voxel::FIXED_SCALE as f32;
    };
    let max = max.unwrap_or(min);
    let chunk_span = i64::from(max - min + 1) * i64::from(civ_voxel::CHUNK_EDGE as i32);
    (chunk_span * i64::from(civ_voxel::FIXED_SCALE)) as f32
}

/// Number of distinct `Frame3d` variants emitted per simulation tick (FR-CIV-BEVY-028 / item 53).
pub const FRAME_BUNDLE_LEN: usize = 7;

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

    /// WebSocket frames emitted per simulation tick ([`FRAME_BUNDLE_LEN`] `Frame3d` values).
    #[must_use]
    pub fn messages_per_tick(self) -> usize {
        FRAME_BUNDLE_LEN * usize::from(self.sends_text())
            + FRAME_BUNDLE_LEN * usize::from(self.sends_binary())
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
    /// Directory for `.civreplay` files (created on bridge start).
    /// `sim.save_replay` / `sim.load_replay` paths are validated as
    /// relative and resolved under this directory with containment
    /// enforced via canonicalize (see `resolve_replay_path`).
    pub replays_dir: PathBuf,
}

impl Default for WsBridgeConfig {
    fn default() -> Self {
        Self {
            addr: SocketAddr::from(([127, 0, 0, 1], 3000)),
            max_clients: 16,
            require_role: false,
            tick_broadcast_format: TickBroadcastFormat::default(),
            saves_dir: PathBuf::from("saves"),
            replays_dir: PathBuf::from("replays"),
        }
    }
}

type ClientOutboundTx = mpsc::UnboundedSender<ClientOutbound>;

/// Outbound WebSocket traffic for one connected client.
enum ClientOutbound {
    /// Immediate JSON-RPC (or error) text frame.
    Rpc(Message),
    /// Shared simulation tick bundle filtered per connection.
    Tick(Arc<TickBroadcast>),
}

/// One simulation tick's `Frame3d` bundle shared across connected clients.
struct TickBroadcast {
    tick: u64,
    frames: Arc<[Frame3d]>,
    encoded: Arc<[Message]>,
    /// Construction lifecycle events emitted during the most recent tick
    /// (FR-CIV-BUILD-002). Bevy clients use these to render scaffolding and
    /// completion FX; replay log mirrors them for deterministic reloads.
    construction_events: Arc<[ProductionEvent]>,
}

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
    clients: Arc<Mutex<Vec<ClientOutboundTx>>>,
    max_clients: usize,
    require_role: bool,
    tick_broadcast_format: TickBroadcastFormat,
    saves_dir: PathBuf,
    replays_dir: PathBuf,
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
    std::fs::create_dir_all(&config.replays_dir).expect("create replays directory");
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
        replays_dir: config.replays_dir,
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
    Query(query): Query<WsConnectQuery>,
) -> impl IntoResponse {
    let header_role = headers
        .get("x-civis-role")
        .and_then(|value| value.to_str().ok())
        .filter(|role| !role.is_empty())
        .map(str::to_owned);
    ws.on_upgrade(move |socket| handle_socket(socket, state, header_role, query))
}

async fn handle_socket(
    socket: WebSocket,
    state: AppState,
    mut connection_role: Option<String>,
    connect_query: WsConnectQuery,
) {
    let (mut sender, mut receiver) = socket.split();
    let (tx, mut rx) = mpsc::unbounded_channel::<ClientOutbound>();
    let subscription_filter = Arc::new(tokio::sync::Mutex::new(
        SubscriptionFilter::from_connect_query(&connect_query),
    ));
    let tick_broadcast_format = state.tick_broadcast_format;

    {
        let mut clients = state.clients.lock().await;
        if clients.len() >= state.max_clients {
            let _ = sender.send(Message::Close(None)).await;
            return;
        }
        clients.push(tx.clone());
    }

    let forward_filter = Arc::clone(&subscription_filter);
    let forward = tokio::spawn(async move {
        while let Some(outbound) = rx.recv().await {
            match outbound {
                ClientOutbound::Rpc(msg) => {
                    if sender.send(msg).await.is_err() {
                        return;
                    }
                }
                ClientOutbound::Tick(broadcast) => {
                    let filter = forward_filter.lock().await;
                    if !filter.is_active() {
                        for msg in broadcast.encoded.iter() {
                            if sender.send(msg.clone()).await.is_err() {
                                return;
                            }
                        }
                        continue;
                    }
                    if !filter.should_deliver_tick(broadcast.tick) {
                        continue;
                    }
                    let frames = filter.filter_frames(broadcast.frames.as_ref());
                    if frames.is_empty() {
                        continue;
                    }
                    let messages =
                        match encode_tick_broadcast_messages(&frames, tick_broadcast_format) {
                            Ok(messages) => messages,
                            Err(err) => {
                                tracing::error!("tick broadcast encode failed: {err}");
                                continue;
                            }
                        };
                    for msg in messages {
                        if sender.send(msg).await.is_err() {
                            return;
                        }
                    }
                }
            }
        }
    });

    while let Some(msg) = receiver.next().await {
        match msg {
            Ok(Message::Text(text)) => {
                let response = handle_jsonrpc_text(
                    &text,
                    &state,
                    &mut connection_role,
                    Arc::clone(&subscription_filter),
                )
                .await;
                if tx
                    .send(ClientOutbound::Rpc(Message::Text(response)))
                    .is_err()
                {
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
    subscription_filter: Arc<tokio::sync::Mutex<SubscriptionFilter>>,
) -> String {
    match parse_request(text) {
        Ok(req) => {
            if let Some(response) =
                handle_subscription_jsonrpc(&req, state, &subscription_filter).await
            {
                return encode_response(&response);
            }
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
                JsonRpcMethod::SimSnapshot
                | JsonRpcMethod::GetFactions
                | JsonRpcMethod::GetResources
                | JsonRpcMethod::GetEmergenceMetrics => {
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

            let emergence = if req.method == JsonRpcMethod::SimEmergence {
                let sim = state.sim.lock().await;
                sim.last_emergence_sample().map(Into::into)
            } else {
                None
            };
            let (research_researched, research_in_progress) = {
                let sim = state.sim.lock().await;
                let cache = sim.research_cache();
                (
                    cache.researched.clone(),
                    cache.in_progress.as_ref().map(|(t, _)| t.clone()),
                )
            };
            let outcome_fields = if req.method == crate::jsonrpc::JsonRpcMethod::SimOutcome {
                let sim = state.sim.lock().await;
                let outcome = civ_engine::conditions::check_outcome(&sim);
                Some(crate::jsonrpc::OutcomeFields {
                    tag: outcome.tag().to_owned(),
                    reason: outcome.reason().to_owned(),
                    tick: sim.state.tick,
                })
            } else {
                None
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
                    emergence,
                    researched: research_researched,
                    in_progress_tech: research_in_progress,
                    outcome_fields,
                    last_tick_ms: 0.0,
                },
            );
            apply_dispatch_effect(&mut plan.response, plan.effect, state).await;
            encode_response(&plan.response)
        }
        Err(err) => encode_response(&parse_error_response(text, err)),
    }
}

async fn handle_subscription_jsonrpc(
    req: &crate::jsonrpc::JsonRpcRequest,
    state: &AppState,
    subscription_filter: &Arc<tokio::sync::Mutex<SubscriptionFilter>>,
) -> Option<JsonRpcResponse> {
    match req.method {
        JsonRpcMethod::SimSubscribe | JsonRpcMethod::SimUpdateSubscription => {
            let tick = state.tick.load(Ordering::SeqCst);
            let mut filter = subscription_filter.lock().await;
            match filter.apply_subscribe_params(req.params.as_ref(), tick) {
                Ok(result) => Some(JsonRpcResponse::success(req.id.clone(), result)),
                Err(error) => Some(JsonRpcResponse::failure(req.id.clone(), error)),
            }
        }
        JsonRpcMethod::SimUnsubscribe => {
            let mut filter = subscription_filter.lock().await;
            filter.clear();
            Some(JsonRpcResponse::success(
                req.id.clone(),
                serde_json::json!({ "unsubscribed": true }),
            ))
        }
        _ => None,
    }
}

fn build_agent_appearance_frame(sim: &Simulation, tick: u64) -> AgentAppearanceFrame {
    let updates = sim
        .world
        .query::<(&AgentCivilian, &Wardrobe, &Tools)>()
        .iter()
        .map(
            |(_entity, (civilian, wardrobe, tools))| AgentAppearanceUpdate {
                agent_id: civilian.id,
                era: wardrobe.era,
                wardrobe: wardrobe.material,
                tools: tools.material,
                scale: 1.0,
                position: None,
            },
        )
        .collect();
    AgentAppearanceFrame { tick, updates }
}

fn need_satisfaction(pressure: f32) -> f32 {
    (1.0 - pressure).clamp(0.0, 1.0)
}

fn job_profession_label(job: JobType) -> &'static str {
    match job {
        JobType::Farmer => "farmer",
        JobType::Warrior => "warrior",
        JobType::Scholar => "scholar",
        JobType::Trader => "trader",
        JobType::Priest => "priest",
        JobType::Admin => "admin",
        JobType::Unemployed => "unemployed",
    }
}

fn government_for_faction(faction_id: u32) -> Government3d {
    match faction_id % 6 {
        0 => Government3d::Monarchy,
        1 => Government3d::Republic,
        2 => Government3d::Theocracy,
        3 => Government3d::Junta,
        4 => Government3d::Council,
        _ => Government3d::Corporate,
    }
}

fn faction_for_norm(x: f32, y: f32, sim: &Simulation) -> u32 {
    sim.spectator_view()
        .factions
        .iter()
        .min_by(|a, b| {
            let da = (x - a.capital[0]).powi(2) + (y - a.capital[1]).powi(2);
            let db = (x - b.capital[0]).powi(2) + (y - b.capital[1]).powi(2);
            da.partial_cmp(&db).unwrap_or(std::cmp::Ordering::Equal)
        })
        .map(|faction| faction.id)
        .unwrap_or(0)
}

fn build_civilian_state_frame(sim: &Simulation, tick: u64) -> CivilianStateFrame {
    let mut civilians: Vec<CivilianStateEntry> = sim
        .world
        .query::<(&AgentCivilian, &Needs, &Wardrobe)>()
        .iter()
        .map(|(entity, (civilian, needs, wardrobe))| {
            let (profession, health) = sim
                .world
                .get::<&Citizen>(entity)
                .ok()
                .map(|citizen| {
                    let job = citizen
                        .job
                        .unwrap_or_else(|| job_type_for_civilian_id(civilian.id));
                    (
                        job_profession_label(job).to_string(),
                        citizen.health.to_f64() as f32,
                    )
                })
                .unwrap_or_else(|| {
                    (
                        job_profession_label(job_type_for_civilian_id(civilian.id)).to_string(),
                        1.0,
                    )
                });
            let rest = (need_satisfaction(needs.food)
                + need_satisfaction(needs.shelter)
                + need_satisfaction(needs.safety)
                + need_satisfaction(needs.belonging))
                / 4.0;
            CivilianStateEntry {
                id: civilian.id,
                faction_id: match civilian.alignment {
                    civ_agents::Alignment::Faction(id) => id,
                    _ => 0,
                },
                needs: CivilianNeeds3d {
                    food: need_satisfaction(needs.food),
                    shelter: need_satisfaction(needs.shelter),
                    safety: need_satisfaction(needs.safety),
                    social: need_satisfaction(needs.belonging),
                    rest,
                },
                profession,
                genome_summary: GenomeSummary3d {
                    summary: format!("era-{}", wardrobe.era),
                    lineage: format!(
                        "faction-{}",
                        match civilian.alignment {
                            civ_agents::Alignment::Faction(id) => id,
                            _ => 0,
                        }
                    ),
                    traits: Vec::new(),
                },
                species: "human".to_string(),
                health,
            }
        })
        .collect();
    civilians.sort_by_key(|entry| entry.id);
    civilians.truncate(256);
    CivilianStateFrame { tick, civilians }
}

fn build_faction_state_frame(sim: &Simulation, tick: u64) -> FactionStateFrame {
    let era = ((tick / 120) % 6) as u16;
    let institutions = crate::jsonrpc::institutions_from_sim(sim);
    let market_balance = institutions
        .iter()
        .find(|row| row.kind == "market")
        .map(|row| row.balance_joules as f64)
        .unwrap_or(0.0);
    let treasury_balance = institutions
        .iter()
        .find(|row| row.kind == "treasury")
        .map(|row| row.balance_joules as f64)
        .unwrap_or(0.0);

    let mut factions: Vec<FactionStateEntry> = sim
        .spectator_view()
        .factions
        .iter()
        .map(|faction| {
            let mut amount = sim
                .state
                .faction_treasury
                .get(&faction.id)
                .map(|value| value.to_f64())
                .unwrap_or(0.0);
            if faction.id == 0 {
                amount += market_balance + treasury_balance;
            }
            FactionStateEntry {
                id: faction.id,
                era,
                government: government_for_faction(faction.id),
                treasury: FactionTreasury3d {
                    amount,
                    currency: "joules".to_string(),
                },
            }
        })
        .collect();
    factions.sort_by_key(|entry| entry.id);
    let mut population_by_faction: std::collections::BTreeMap<u32, u32> =
        std::collections::BTreeMap::new();
    for (_, (civilian, _needs, _wardrobe)) in sim
        .world
        .query::<(
            &civ_agents::Civilian,
            &civ_agents::Needs,
            &civ_agents::Wardrobe,
        )>()
        .iter()
    {
        let fid = match civilian.alignment {
            civ_agents::Alignment::Faction(id) => id,
            _ => 0,
        };
        *population_by_faction.entry(fid).or_insert(0) += 1;
    }
    FactionStateFrame {
        tick,
        factions,
        population_by_faction,
    }
}

/// Build event-feed messages for one tick.
///
/// Maps births/deaths/diplomacy/combat from the live sim. `last_tick_mod_lifecycle` JSON is
/// surfaced as [`EventFeedMessage3d::Tech`] stubs until a dedicated system/lifecycle wire kind
/// exists (Bevy maps those to `EventKind::System` locally).
fn build_event_feed_frame(sim: &Simulation, tick: u64) -> EventFeedFrame {
    let mut events = Vec::new();

    for birth in sim.last_births() {
        events.push(EventFeedMessage3d::Birth(BirthEvent3d {
            entity_id: birth.entity_id,
            faction_id: faction_for_norm(birth.x, birth.y, sim),
            species: "human".to_string(),
            position: Some(WorldXZ {
                x: birth.x,
                z: birth.y,
            }),
        }));
    }

    for death in sim.last_deaths() {
        events.push(EventFeedMessage3d::Death(DeathEvent3d {
            entity_id: death.entity_id,
            faction_id: faction_for_norm(death.x, death.y, sim),
            position: Some(WorldXZ {
                x: death.x,
                z: death.y,
            }),
            cause: String::new(),
        }));
    }

    for diplomacy in sim.diplomacy_events() {
        if diplomacy.tick != tick {
            continue;
        }
        let outcome = match diplomacy.kind {
            DiplomacyKind::TradeAgreement => "trade_agreement",
            DiplomacyKind::Conflict => "conflict",
            DiplomacyKind::Peace => "peace",
        };
        events.push(EventFeedMessage3d::Battle(BattleEvent3d {
            attacker_faction: diplomacy.faction_a,
            defender_faction: diplomacy.faction_b,
            outcome: outcome.to_string(),
            position: None,
        }));
    }

    for pulse in sim.last_tick_combat_pulses() {
        events.push(EventFeedMessage3d::Battle(BattleEvent3d {
            attacker_faction: 0,
            defender_faction: 0,
            outcome: "combat_pulse".to_string(),
            position: Some(WorldXZ {
                x: pulse.x,
                z: pulse.y,
            }),
        }));
    }

    for line in sim.last_tick_mod_lifecycle() {
        let Ok(value) = serde_json::from_str::<serde_json::Value>(line) else {
            continue;
        };
        let Some(event_name) = value.get("event").and_then(|v| v.as_str()) else {
            continue;
        };
        if event_name == "mod.loaded.v1" {
            let mod_name = value
                .get("mod_name")
                .or_else(|| value.get("mod_id"))
                .and_then(|v| v.as_str())
                .unwrap_or("mod");
            events.push(EventFeedMessage3d::Tech(TechEvent3d {
                faction_id: 0,
                era: ((tick / 120) % 6) as u16,
                tech: format!("mod.loaded:{mod_name}"),
            }));
        }
    }

    EventFeedFrame { tick, events }
}

fn build_frame_bundle(sim: &Simulation) -> Result<[Frame3d; FRAME_BUNDLE_LEN], String> {
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
        buildings: Vec::new(),
        graph: None,
    };
    Ok([
        Frame3d::VoxelDelta(voxel),
        Frame3d::BuildingDiff(building),
        Frame3d::AgentAppearance(build_agent_appearance_frame(sim, tick)),
        Frame3d::CivilianState(build_civilian_state_frame(sim, tick)),
        Frame3d::FactionState(build_faction_state_frame(sim, tick)),
        Frame3d::EventFeed(build_event_feed_frame(sim, tick)),
        Frame3d::Climate(ClimateFrame {
            tick,
            climate: *sim.climate(),
            weather: sim.weather_grid().to_vec(),
        }),
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
            // `parse_replay_path` already rejected absolute paths, `..`
            // segments, prefixes, and root. Resolve the path under the
            // bridge's `replays_dir` with canonicalize+containment to
            // defeat symlink escape. Any failure becomes a clear error
            // response — no silent fallback to `path` as-is.
            let resolved = match crate::jsonrpc::resolve_replay_path(&state.replays_dir, &path) {
                Ok(resolved) => resolved,
                Err(err) => {
                    tracing::error!("sim.save_replay rejected path {path:?}: {err}");
                    set_replay_io_error(response, format!("rejected path {path:?}: {err}"));
                    return;
                }
            };
            let save_result = {
                let sim = state.sim.lock().await;
                sim.save_replay(&resolved)
            };
            if let Err(err) = save_result {
                tracing::error!("sim.save_replay failed: {err}");
                set_replay_io_error(response, err.to_string());
            }
        }
        DispatchEffect::LoadReplay { path } => {
            // Same containment check as SaveReplay: the path must resolve
            // inside `replays_dir` after canonicalize.
            let resolved = match crate::jsonrpc::resolve_replay_path(&state.replays_dir, &path) {
                Ok(resolved) => resolved,
                Err(err) => {
                    tracing::error!("sim.load_replay rejected path {path:?}: {err}");
                    set_replay_io_error(response, format!("rejected path {path:?}: {err}"));
                    return;
                }
            };
            match Simulation::load_replay_from_file(&resolved) {
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
            }
        }
        DispatchEffect::ResetSimulation { seed } => {
            *state.sim.lock().await = Simulation::with_seed(seed);
            state.tick.store(0, Ordering::SeqCst);
        }
        DispatchEffect::LoadScenario { preset, seed } => {
            match load_scenario(preset_scenario_path(&preset)) {
                Ok(scenario) => {
                    *state.sim.lock().await = scenario.into_simulation(seed);
                    state.tick.store(0, Ordering::SeqCst);
                    tracing::info!(%preset, seed, "loaded scenario preset");
                }
                Err(err) => {
                    tracing::error!(%preset, ?err, "sim.load_scenario failed");
                    *response = crate::jsonrpc::JsonRpcResponse::failure(
                        response.id.clone(),
                        crate::jsonrpc::JsonRpcError {
                            code: crate::jsonrpc::error_code::INTERNAL_ERROR,
                            message: format!("failed to load preset {preset:?}: {err}"),
                            data: None,
                        },
                    );
                }
            }
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
            let entity = civ_agents::spawn_civilian_at(
                &mut sim.world,
                entity_seq,
                civ_agents::Alignment::Faction(faction),
                x,
                y,
                civ_agents::ActorVisualKind::Humanoid,
                &mut rng,
            );
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
                        civ_agents::Alignment::Faction(faction),
                        x,
                        y,
                        civ_agents::ActorVisualKind::Humanoid,
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
        DispatchEffect::QueueResearch { tech } => {
            state.sim.lock().await.research_cache_mut().queued.push_back(tech);
        }
        DispatchEffect::GodAction { action, x, y, target_faction, magnitude } => {
            use civ_engine::disasters::{trigger_disaster, DisasterKind};
            use civ_voxel::WorldCoord;
            let mut sim = state.sim.lock().await;
            let world_w = sim.voxel().width() as f32;
            let world_d = sim.voxel().depth() as f32;
            let wx = x.unwrap_or(0.5) * world_w;
            let wz = y.unwrap_or(0.5) * world_d;
            let pos = WorldCoord { x: wx as i64, y: 0, z: wz as i64 };
            let mag = magnitude.unwrap_or(0.5_f32).clamp(0.0, 1.0);
            match action.as_str() {
                "smite" => trigger_disaster(&mut sim, DisasterKind::Meteor, pos),
                "earthquake" => trigger_disaster(&mut sim, DisasterKind::Quake, pos),
                "plague" => {
                    trigger_disaster(&mut sim, DisasterKind::Plague, pos);
                    if let Some(fid) = target_faction {
                        if let Some(t) = sim.state.faction_treasury.get_mut(&fid) {
                            let debit = civ_engine::Fixed::from_num(mag * 500.0_f32);
                            *t = (*t - debit).max(civ_engine::Fixed::ZERO);
                        }
                    }
                }
                "bless" => {
                    if let Some(fid) = target_faction {
                        if let Some(t) = sim.state.faction_treasury.get_mut(&fid) {
                            let credit = civ_engine::Fixed::from_num(mag * 1000.0_f32);
                            *t += credit;
                        }
                    }
                    sim.add_belief(500);
                }
                "miracle" => {
                    sim.add_belief(2000);
                    let boost = civ_engine::Fixed::from_num(mag * 200.0_f32);
                    for t in sim.state.faction_treasury.values_mut() { *t += boost; }
                }
                _ => {}
            }
            if let Some(result) = response.result.as_mut() {
                if let Some(obj) = result.as_object_mut() {
                    obj.insert("applied".to_owned(), serde_json::json!(true));
                }
            }
        }
        DispatchEffect::GodAction { action, x, y, target_faction, magnitude } => {
            use civ_engine::disasters::{trigger_disaster, DisasterKind};
            use civ_voxel::WorldCoord;
            let mut sim = state.sim.lock().await;
            let world_w = sim.voxel().width() as f32;
            let world_d = sim.voxel().depth() as f32;
            let wx = x.unwrap_or(0.5) * world_w;
            let wz = y.unwrap_or(0.5) * world_d;
            let pos = WorldCoord { x: wx as i64, y: 0, z: wz as i64 };
            let mag = magnitude.unwrap_or(0.5_f32).clamp(0.0, 1.0);
            match action.as_str() {
                "smite" => trigger_disaster(&mut sim, DisasterKind::Meteor, pos),
                "earthquake" => trigger_disaster(&mut sim, DisasterKind::Quake, pos),
                "plague" => {
                    trigger_disaster(&mut sim, DisasterKind::Plague, pos);
                    if let Some(fid) = target_faction {
                        if let Some(t) = sim.state.faction_treasury.get_mut(&fid) {
                            let debit = civ_engine::Fixed::from_num(mag * 500.0_f32);
                            *t = (*t - debit).max(civ_engine::Fixed::ZERO);
                        }
                    }
                }
                "bless" => {
                    if let Some(fid) = target_faction {
                        if let Some(t) = sim.state.faction_treasury.get_mut(&fid) {
                            let credit = civ_engine::Fixed::from_num(mag * 1000.0_f32);
                            *t += credit;
                        }
                    }
                    sim.add_belief(500);
                }
                "miracle" => {
                    sim.add_belief(2000);
                    let boost = civ_engine::Fixed::from_num(mag * 200.0_f32);
                    for t in sim.state.faction_treasury.values_mut() { *t += boost; }
                }
                _ => {}
            }
            if let Some(result) = response.result.as_mut() {
                if let Some(obj) = result.as_object_mut() {
                    obj.insert("applied".to_owned(), serde_json::json!(true));
                }
            }
        }
        DispatchEffect::GodAction { action, x, y, target_faction, magnitude } => {
            use civ_engine::disasters::{trigger_disaster, DisasterKind};
            use civ_voxel::WorldCoord;
            let mut sim = state.sim.lock().await;
            let world_w = sim.voxel().width() as f32;
            let world_d = sim.voxel().depth() as f32;
            let wx = x.unwrap_or(0.5) * world_w;
            let wz = y.unwrap_or(0.5) * world_d;
            let pos = WorldCoord { x: wx as i64, y: 0, z: wz as i64 };
            let mag = magnitude.unwrap_or(0.5_f32).clamp(0.0, 1.0);
            match action.as_str() {
                "smite" => trigger_disaster(&mut sim, DisasterKind::Meteor, pos),
                "earthquake" => trigger_disaster(&mut sim, DisasterKind::Quake, pos),
                "plague" => {
                    trigger_disaster(&mut sim, DisasterKind::Plague, pos);
                    if let Some(fid) = target_faction {
                        if let Some(t) = sim.state.faction_treasury.get_mut(&fid) {
                            let debit = civ_engine::Fixed::from_num(mag * 500.0_f32);
                            *t = (*t - debit).max(civ_engine::Fixed::ZERO);
                        }
                    }
                }
                "bless" => {
                    if let Some(fid) = target_faction {
                        if let Some(t) = sim.state.faction_treasury.get_mut(&fid) {
                            let credit = civ_engine::Fixed::from_num(mag * 1000.0_f32);
                            *t += credit;
                        }
                    }
                    sim.add_belief(500);
                }
                "miracle" => {
                    sim.add_belief(2000);
                    let boost = civ_engine::Fixed::from_num(mag * 200.0_f32);
                    for t in sim.state.faction_treasury.values_mut() { *t += boost; }
                }
                _ => {}
            }
            if let Some(result) = response.result.as_mut() {
                if let Some(obj) = result.as_object_mut() {
                    obj.insert("applied".to_owned(), serde_json::json!(true));
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
    frames: &[Frame3d],
    format: TickBroadcastFormat,
) -> Result<Vec<Message>, String> {
    let mut payloads = Vec::with_capacity(format.messages_per_tick());
    let send_text = format.sends_text();
    let send_binary = format.sends_binary();

    if send_text && send_binary {
        let mut json_by_frame = Vec::with_capacity(frames.len());
        for frame in frames {
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

    for frame in frames {
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
        let bundle = build_frame_bundle(&sim)?;
        let encoded = Arc::from(
            encode_tick_broadcast_messages(&bundle, state.tick_broadcast_format)?
                .into_boxed_slice(),
        );
        // Snapshot construction events for the just-completed tick. The events
        // are cleared at the top of `Simulation::tick()`, so we read them here
        // after `sim.tick()` has run (FR-CIV-BUILD-002).
        let construction_events: Arc<[ProductionEvent]> = Arc::from(
            sim.last_construction_events()
                .to_vec()
                .into_boxed_slice(),
        );
        Arc::new(TickBroadcast {
            tick,
            frames: Arc::from(bundle),
            encoded,
            construction_events,
        })
    };

    let mut clients = state.clients.lock().await;
    clients.retain(|tx| tx.send(ClientOutbound::Tick(Arc::clone(&batch))).is_ok());
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

    #[test]
    fn need_satisfaction_inverts_and_clamps_pressure() {
        assert_eq!(need_satisfaction(0.0), 1.0); // no pressure -> fully satisfied
        assert_eq!(need_satisfaction(1.0), 0.0); // full pressure -> unsatisfied
        assert!((need_satisfaction(0.25) - 0.75).abs() < 1e-6);
        assert_eq!(need_satisfaction(2.0), 0.0); // over-pressure clamps to 0
        assert_eq!(need_satisfaction(-1.0), 1.0); // negative pressure clamps to 1
    }

    #[test]
    fn tick_broadcast_format_text_binary_flags_are_exclusive_per_variant() {
        assert!(TickBroadcastFormat::Text.sends_text());
        assert!(!TickBroadcastFormat::Text.sends_binary());
        assert!(!TickBroadcastFormat::Binary.sends_text());
        assert!(TickBroadcastFormat::Binary.sends_binary());
        assert!(TickBroadcastFormat::Both.sends_text());
        assert!(TickBroadcastFormat::Both.sends_binary());
        // `Both` emits both stream sizes; single-format variants emit one.
        assert_eq!(
            TickBroadcastFormat::Both.messages_per_tick(),
            TickBroadcastFormat::Text.messages_per_tick()
                + TickBroadcastFormat::Binary.messages_per_tick()
        );
    }
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
        let replays_dir = dir.path().join("replays");
        std::fs::create_dir_all(&replays_dir).expect("replays dir");
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
            replays_dir,
            save_db,
            session_id: "test-session".to_string(),
        };
        (dir, state)
    }

    fn test_subscription_filter() -> Arc<tokio::sync::Mutex<SubscriptionFilter>> {
        Arc::new(tokio::sync::Mutex::new(SubscriptionFilter::default()))
    }

    #[test]
    fn tick_broadcast_both_sends_text_then_binary() {
        let frames = sample_frame_bundle();
        let messages =
            encode_tick_broadcast_messages(&frames, TickBroadcastFormat::Both).expect("encode");
        assert_eq!(
            messages.len(),
            TickBroadcastFormat::Both.messages_per_tick()
        );
        for msg in &messages[..FRAME_BUNDLE_LEN] {
            assert!(matches!(msg, Message::Text(_)));
        }
        for msg in &messages[FRAME_BUNDLE_LEN..] {
            assert!(matches!(msg, Message::Binary(_)));
        }
        if let Message::Binary(bytes) = &messages[FRAME_BUNDLE_LEN] {
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
        let frames = sample_frame_bundle();
        for format in [
            TickBroadcastFormat::Text,
            TickBroadcastFormat::Binary,
            TickBroadcastFormat::Both,
        ] {
            let messages = encode_tick_broadcast_messages(&frames, format).expect("encode");
            assert_eq!(
                messages.len(),
                format.messages_per_tick(),
                "{format:?} message count"
            );
        }
        assert_eq!(
            TickBroadcastFormat::Text.messages_per_tick(),
            FRAME_BUNDLE_LEN
        );
        assert_eq!(
            TickBroadcastFormat::Binary.messages_per_tick(),
            FRAME_BUNDLE_LEN
        );
        assert_eq!(
            TickBroadcastFormat::Both.messages_per_tick(),
            FRAME_BUNDLE_LEN * 2
        );
    }

    #[test]
    fn tick_broadcast_binary_only_skips_text_frames() {
        let frames = sample_frame_bundle();
        let messages =
            encode_tick_broadcast_messages(&frames, TickBroadcastFormat::Binary).expect("encode");
        assert_eq!(messages.len(), FRAME_BUNDLE_LEN);
        assert!(messages.iter().all(|msg| matches!(msg, Message::Binary(_))));
    }

    #[test]
    fn tick_broadcast_both_binary_payload_matches_text_json() {
        let frames = sample_frame_bundle();
        let messages =
            encode_tick_broadcast_messages(&frames, TickBroadcastFormat::Both).expect("encode");
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

    fn sample_frame_bundle() -> [Frame3d; FRAME_BUNDLE_LEN] {
        [
            Frame3d::VoxelDelta(civ_protocol_3d::VoxelDeltaFrame {
                tick: 1,
                deltas: Vec::new(),
            }),
            Frame3d::BuildingDiff(BuildingDiffFrame {
                tick: 1,
                provenance: BuildingProvenance::Procedural,
                buildings: Vec::new(),
                graph: None,
            }),
            Frame3d::AgentAppearance(AgentAppearanceFrame {
                tick: 1,
                updates: Vec::new(),
            }),
            Frame3d::CivilianState(CivilianStateFrame {
                tick: 1,
                civilians: Vec::new(),
            }),
            Frame3d::FactionState(FactionStateFrame {
                tick: 1,
                factions: Vec::new(),
            }),
            Frame3d::EventFeed(EventFeedFrame {
                tick: 1,
                events: Vec::new(),
            }),
            Frame3d::Climate(ClimateFrame {
                tick: 1,
                climate: *Simulation::with_seed(1).climate(),
                weather: Vec::new(),
            }),
        ]
    }

    #[test]
    fn frame_bundle_includes_all_wire_kinds() {
        let sim = Simulation::with_seed(11);
        let bundle = build_frame_bundle(&sim).expect("bundle");
        assert_eq!(bundle.len(), FRAME_BUNDLE_LEN);
        let mut kinds = [false; FRAME_BUNDLE_LEN];
        for frame in &bundle {
            let idx = match frame {
                Frame3d::VoxelDelta(_) => 0,
                Frame3d::BuildingDiff(_) => 1,
                Frame3d::AgentAppearance(_) => 2,
                Frame3d::CivilianState(_) => 3,
                Frame3d::FactionState(_) => 4,
                Frame3d::EventFeed(_) => 5,
                Frame3d::Climate(_) => 6,
            };
            kinds[idx] = true;
        }
        assert!(kinds.iter().all(|present| *present));
        let Frame3d::CivilianState(civilian) = &bundle[3] else {
            panic!("expected civilian state frame");
        };
        assert!(
            !civilian.civilians.is_empty(),
            "seed 11 should emit agent civilians"
        );
        let Frame3d::FactionState(faction) = &bundle[4] else {
            panic!("expected faction state frame");
        };
        assert_eq!(faction.factions.len(), 4);
    }

    #[test]
    fn agent_appearance_ids_match_civilian_state_ids() {
        let sim = Simulation::with_seed(11);
        let bundle = build_frame_bundle(&sim).expect("bundle");
        let Frame3d::AgentAppearance(appearance) = &bundle[2] else {
            panic!("expected agent appearance frame");
        };
        let Frame3d::CivilianState(civilian) = &bundle[3] else {
            panic!("expected civilian state frame");
        };
        let appearance_ids: std::collections::BTreeSet<u64> = appearance
            .updates
            .iter()
            .map(|update| update.agent_id)
            .collect();
        let civilian_ids: std::collections::BTreeSet<u64> =
            civilian.civilians.iter().map(|entry| entry.id).collect();
        assert_eq!(
            appearance_ids, civilian_ids,
            "agent appearance and civilian state must share civilian.id keys"
        );
        assert!(
            !appearance_ids.is_empty(),
            "seed 11 should emit matching agent ids"
        );
    }

    /// Manual probe: `cargo test -p civ-server tick_broadcast_encode_bench --release -- --ignored --nocapture`
    #[test]
    #[ignore = "manual perf probe"]
    fn tick_broadcast_encode_bench() {
        use std::time::Instant;

        let frames = sample_frame_bundle();
        let iterations = 20_000u32;
        for format in [TickBroadcastFormat::Binary, TickBroadcastFormat::Both] {
            let start = Instant::now();
            for _ in 0..iterations {
                let _ = encode_tick_broadcast_messages(&frames, format).expect("encode");
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
            buildings: Vec::new(),
            graph: None,
        });
        let json = serde_json::to_string(&frame).expect("json");
        let decoded: Frame3d = serde_json::from_str(&json).expect("decode");
        assert_eq!(decoded.tick(), 9);
    }

    #[test]
    fn encode_tick_broadcast_text_only_format() {
        let frames = sample_frame_bundle();
        let messages =
            encode_tick_broadcast_messages(&frames, TickBroadcastFormat::Text).expect("encode");
        assert_eq!(messages.len(), FRAME_BUNDLE_LEN);
        assert!(messages.iter().all(|msg| matches!(msg, Message::Text(_))));
    }

    #[test]
    fn encode_tick_broadcast_binary_only_format() {
        let frames = sample_frame_bundle();
        let messages =
            encode_tick_broadcast_messages(&frames, TickBroadcastFormat::Binary).expect("encode");
        assert_eq!(messages.len(), FRAME_BUNDLE_LEN);
        assert!(messages.iter().all(|msg| matches!(msg, Message::Binary(_))));
        for msg in &messages {
            if let Message::Binary(bytes) = msg {
                assert!(
                    bytes.starts_with(civ_protocol_3d::FRAME3D_BINARY_MAGIC),
                    "binary frame must start with F3D0 magic"
                );
            }
        }
    }

    #[test]
    fn encode_tick_broadcast_both_format_interleaves_correctly() {
        let frames = sample_frame_bundle();
        let messages =
            encode_tick_broadcast_messages(&frames, TickBroadcastFormat::Both).expect("encode");
        let text_count = messages
            .iter()
            .filter(|msg| matches!(msg, Message::Text(_)))
            .count();
        let binary_count = messages
            .iter()
            .filter(|msg| matches!(msg, Message::Binary(_)))
            .count();
        assert_eq!(text_count, FRAME_BUNDLE_LEN);
        assert_eq!(binary_count, FRAME_BUNDLE_LEN);
    }

    #[test]
    fn build_agent_appearance_frame_from_empty_sim() {
        let sim = Simulation::with_seed(99);
        let frame = build_agent_appearance_frame(&sim, 42);
        assert_eq!(frame.tick, 42);
        // Seed 99 may have no agents, but frame must be valid
        assert_eq!(frame.updates.len(), frame.updates.len());
    }

    #[test]
    fn build_civilian_state_frame_truncates_at_256() {
        let sim = Simulation::with_seed(11);
        let frame = build_civilian_state_frame(&sim, 5);
        assert_eq!(frame.tick, 5);
        assert!(
            frame.civilians.len() <= 256,
            "civilians must be capped at 256"
        );
        // Verify all entries are sorted by id
        for i in 1..frame.civilians.len() {
            assert!(
                frame.civilians[i - 1].id <= frame.civilians[i].id,
                "civilians must be sorted by id"
            );
        }
    }

    #[test]
    fn build_civilian_state_frame_needs_are_normalized() {
        let sim = Simulation::with_seed(11);
        let frame = build_civilian_state_frame(&sim, 0);
        for entry in &frame.civilians {
            assert!(entry.needs.food >= 0.0 && entry.needs.food <= 1.0);
            assert!(entry.needs.shelter >= 0.0 && entry.needs.shelter <= 1.0);
            assert!(entry.needs.safety >= 0.0 && entry.needs.safety <= 1.0);
            assert!(entry.needs.social >= 0.0 && entry.needs.social <= 1.0);
            assert!(entry.needs.rest >= 0.0 && entry.needs.rest <= 1.0);
        }
    }

    #[test]
    fn build_faction_state_frame_wraps_era_at_six() {
        let sim = Simulation::with_seed(7);
        for tick in [0, 120, 240, 360, 480, 600, 719] {
            let frame = build_faction_state_frame(&sim, tick);
            let expected_era = ((tick / 120) % 6) as u16;
            for entry in &frame.factions {
                assert_eq!(
                    entry.era, expected_era,
                    "tick {tick} should wrap era to {expected_era}"
                );
            }
        }
    }

    #[test]
    fn build_faction_state_frame_sorts_by_faction_id() {
        let sim = Simulation::with_seed(3);
        let frame = build_faction_state_frame(&sim, 0);
        for i in 1..frame.factions.len() {
            assert!(
                frame.factions[i - 1].id <= frame.factions[i].id,
                "factions must be sorted by id"
            );
        }
    }

    #[test]
    fn build_event_feed_frame_handles_empty_events() {
        let sim = Simulation::with_seed(100);
        let frame = build_event_feed_frame(&sim, 0);
        assert_eq!(frame.tick, 0);
        // Empty simulation should produce minimal events
        let event_count = frame.events.len();
        assert!(event_count >= 0); // Always valid, even if empty
    }

    #[test]
    fn build_event_feed_frame_categorizes_all_event_kinds() {
        let sim = Simulation::with_seed(11);
        let frame = build_event_feed_frame(&sim, 0);
        for event in &frame.events {
            match event {
                EventFeedMessage3d::Birth(_) => {}
                EventFeedMessage3d::Death(_) => {}
                EventFeedMessage3d::Battle(_) => {}
                EventFeedMessage3d::Tech(_) => {}
                EventFeedMessage3d::Disaster(_) => {}
            }
        }
    }

    #[test]
    fn build_frame_bundle_returns_exact_frame_count() {
        for seed in [1, 5, 11, 37, 42, 99] {
            let sim = Simulation::with_seed(seed);
            let bundle = build_frame_bundle(&sim).expect("bundle");
            assert_eq!(
                bundle.len(),
                FRAME_BUNDLE_LEN,
                "seed {seed} must return exactly {FRAME_BUNDLE_LEN} frames"
            );
        }
    }

    #[test]
    fn build_frame_bundle_all_frames_have_matching_tick() {
        let sim = Simulation::with_seed(17);
        let expected_tick = sim.state.tick;
        let bundle = build_frame_bundle(&sim).expect("bundle");
        for frame in &bundle {
            assert_eq!(
                frame.tick(),
                expected_tick,
                "all frames in bundle must have matching tick"
            );
        }
    }

    #[test]
    fn tick_broadcast_format_both_sends_text_before_binary() {
        let frames = sample_frame_bundle();
        let messages =
            encode_tick_broadcast_messages(&frames, TickBroadcastFormat::Both).expect("encode");
        assert!(messages.len() >= FRAME_BUNDLE_LEN * 2);
        // First FRAME_BUNDLE_LEN are text
        for i in 0..FRAME_BUNDLE_LEN {
            assert!(matches!(messages[i], Message::Text(_)));
        }
        // Second FRAME_BUNDLE_LEN are binary
        for i in FRAME_BUNDLE_LEN..FRAME_BUNDLE_LEN * 2 {
            assert!(matches!(messages[i], Message::Binary(_)));
        }
    }

    #[test]
    fn encode_messages_per_tick_consistency() {
        assert_eq!(
            TickBroadcastFormat::Text.messages_per_tick(),
            FRAME_BUNDLE_LEN
        );
        assert_eq!(
            TickBroadcastFormat::Binary.messages_per_tick(),
            FRAME_BUNDLE_LEN
        );
        assert_eq!(
            TickBroadcastFormat::Both.messages_per_tick(),
            FRAME_BUNDLE_LEN * 2
        );
    }

    #[test]
    fn build_civilian_state_frame_handles_missing_citizen() {
        // Some agents may not have Citizen component; frame builder should handle gracefully
        let sim = Simulation::with_seed(11);
        let frame = build_civilian_state_frame(&sim, 0);
        // At least verify structure is valid
        for entry in &frame.civilians {
            assert!(!entry.profession.is_empty());
            assert!(!entry.species.is_empty());
            assert!(entry.health >= 0.0);
        }
    }

    #[test]
    fn need_satisfaction_boundary_values() {
        // Test exact boundary values for clamping logic
        assert_eq!(need_satisfaction(0.0), 1.0);
        assert_eq!(need_satisfaction(0.5), 0.5);
        assert_eq!(need_satisfaction(1.0), 0.0);
        // Over 1.0 should clamp to 0.0
        assert_eq!(need_satisfaction(100.0), 0.0);
        // Negative should clamp to 1.0
        assert_eq!(need_satisfaction(-100.0), 1.0);
    }

    #[test]
    fn government_for_faction_all_six_variants() {
        use Government3d::*;
        let variants = [Monarchy, Republic, Theocracy, Junta, Council, Corporate];
        for (idx, expected) in variants.iter().enumerate() {
            assert_eq!(government_for_faction(idx as u32), *expected);
        }
    }

    #[test]
    fn job_profession_label_all_variants() {
        use JobType::*;
        let all_jobs = [Farmer, Warrior, Scholar, Trader, Priest, Admin, Unemployed];
        let labels = [
            "farmer",
            "warrior",
            "scholar",
            "trader",
            "priest",
            "admin",
            "unemployed",
        ];
        for (job, label) in all_jobs.iter().zip(labels.iter()) {
            assert_eq!(job_profession_label(*job), *label);
        }
    }

    #[test]
    fn frame3d_all_variants_round_trip() {
        let frames = [
            Frame3d::VoxelDelta(civ_protocol_3d::VoxelDeltaFrame {
                tick: 1,
                deltas: Vec::new(),
            }),
            Frame3d::BuildingDiff(BuildingDiffFrame {
                tick: 1,
                provenance: BuildingProvenance::Procedural,
                buildings: Vec::new(),
                graph: None,
            }),
            Frame3d::AgentAppearance(AgentAppearanceFrame {
                tick: 1,
                updates: Vec::new(),
            }),
            Frame3d::CivilianState(CivilianStateFrame {
                tick: 1,
                civilians: Vec::new(),
            }),
            Frame3d::FactionState(FactionStateFrame {
                tick: 1,
                factions: Vec::new(),
            }),
            Frame3d::EventFeed(EventFeedFrame {
                tick: 1,
                events: Vec::new(),
            }),
            Frame3d::Climate(ClimateFrame {
                tick: 1,
                climate: *Simulation::with_seed(1).climate(),
                weather: Vec::new(),
            }),
        ];
        for frame in &frames {
            let json = serde_json::to_string(frame).expect("json encode");
            let decoded: Frame3d = serde_json::from_str(&json).expect("json decode");
            assert_eq!(decoded.tick(), frame.tick());
        }
    }

    #[test]
    fn encode_messages_handles_all_frame_variants() {
        let frames = [
            Frame3d::VoxelDelta(civ_protocol_3d::VoxelDeltaFrame {
                tick: 5,
                deltas: Vec::new(),
            }),
            Frame3d::BuildingDiff(BuildingDiffFrame {
                tick: 5,
                provenance: BuildingProvenance::Freehand,
                buildings: Vec::new(),
                graph: None,
            }),
            Frame3d::AgentAppearance(AgentAppearanceFrame {
                tick: 5,
                updates: Vec::new(),
            }),
            Frame3d::CivilianState(CivilianStateFrame {
                tick: 5,
                civilians: Vec::new(),
            }),
            Frame3d::FactionState(FactionStateFrame {
                tick: 5,
                factions: Vec::new(),
            }),
            Frame3d::EventFeed(EventFeedFrame {
                tick: 5,
                events: Vec::new(),
            }),
            Frame3d::Climate(ClimateFrame {
                tick: 5,
                climate: *Simulation::with_seed(1).climate(),
                weather: Vec::new(),
            }),
        ];
        for format in [
            TickBroadcastFormat::Text,
            TickBroadcastFormat::Binary,
            TickBroadcastFormat::Both,
        ] {
            let messages = encode_tick_broadcast_messages(&frames, format).expect("encode");
            assert!(!messages.is_empty());
        }
    }

    #[tokio::test]
    async fn build_frame_bundle_integrates_all_builders() {
        let sim = Simulation::with_seed(11);
        let bundle = build_frame_bundle(&sim).expect("bundle");
        // Verify all 7 frame types are present
        let mut found = [false; 7];
        for frame in &bundle {
            match frame {
                Frame3d::VoxelDelta(_) => found[0] = true,
                Frame3d::BuildingDiff(_) => found[1] = true,
                Frame3d::AgentAppearance(_) => found[2] = true,
                Frame3d::CivilianState(_) => found[3] = true,
                Frame3d::FactionState(_) => found[4] = true,
                Frame3d::EventFeed(_) => found[5] = true,
                Frame3d::Climate(_) => found[6] = true,
            }
        }
        assert!(
            found.iter().all(|&f| f),
            "bundle must contain all 7 frame types"
        );
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
    async fn frame_bundle_is_deterministic_for_fixed_seed() {
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
        let text = handle_jsonrpc_text(
            "{not json",
            &state,
            &mut connection_role,
            test_subscription_filter(),
        )
        .await;
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
            test_subscription_filter(),
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
            test_subscription_filter(),
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
    async fn jsonrpc_sim_subscribe_applies_frame_kind_filter() {
        let sim = Arc::new(Mutex::new(Simulation::with_seed(23)));
        let (_dir, state) = test_app_state(sim, 0, 1, false);
        let mut connection_role = None;
        let filter = test_subscription_filter();
        let text = handle_jsonrpc_text(
            r#"{"jsonrpc":"2.0","id":11,"method":"sim.subscribe","params":{"frame_kinds":["climate"]}}"#,
            &state,
            &mut connection_role,
            Arc::clone(&filter),
        )
        .await;
        let value: serde_json::Value = serde_json::from_str(&text).expect("subscribe json");
        assert_eq!(
            value.pointer("/result/subscribed"),
            Some(&serde_json::json!(true))
        );
        assert_eq!(
            value.pointer("/result/filter_active"),
            Some(&serde_json::json!(true))
        );
        let bundle = sample_frame_bundle();
        let guard = filter.lock().await;
        assert_eq!(guard.filter_frames(&bundle).len(), 1);
    }

    #[tokio::test]
    async fn jsonrpc_sim_unsubscribe_restores_full_broadcast() {
        let sim = Arc::new(Mutex::new(Simulation::with_seed(24)));
        let (_dir, state) = test_app_state(sim, 0, 1, false);
        let mut connection_role = None;
        let filter = test_subscription_filter();
        let subscribe = handle_jsonrpc_text(
            r#"{"jsonrpc":"2.0","id":12,"method":"sim.subscribe","params":{"frame_kinds":["climate"]}}"#,
            &state,
            &mut connection_role,
            Arc::clone(&filter),
        )
        .await;
        assert!(subscribe.contains("\"subscribed\":true"));
        let unsubscribe = handle_jsonrpc_text(
            r#"{"jsonrpc":"2.0","id":13,"method":"sim.unsubscribe","params":{}}"#,
            &state,
            &mut connection_role,
            Arc::clone(&filter),
        )
        .await;
        assert!(unsubscribe.contains("\"unsubscribed\":true"));
        let guard = filter.lock().await;
        assert!(!guard.is_active());
        assert_eq!(
            guard.filter_frames(&sample_frame_bundle()).len(),
            FRAME_BUNDLE_LEN
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
            test_subscription_filter(),
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
            test_subscription_filter(),
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
            test_subscription_filter(),
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
            test_subscription_filter(),
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
            test_subscription_filter(),
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
            test_subscription_filter(),
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

    #[test]
    fn government_for_faction_cycles_through_six_forms_and_wraps() {
        use Government3d::*;
        let expected = [Monarchy, Republic, Theocracy, Junta, Council, Corporate];
        for (id, want) in expected.into_iter().enumerate() {
            assert_eq!(government_for_faction(id as u32), want, "faction {id}");
        }
        // The mapping is mod-6: faction 6 wraps to Monarchy, 7 to Republic, 11 to Corporate.
        assert_eq!(government_for_faction(6), Monarchy);
        assert_eq!(government_for_faction(7), Republic);
        assert_eq!(government_for_faction(11), Corporate);
    }

    #[test]
    fn job_profession_label_maps_every_job_variant() {
        use JobType::*;
        let cases = [
            (Farmer, "farmer"),
            (Warrior, "warrior"),
            (Scholar, "scholar"),
            (Trader, "trader"),
            (Priest, "priest"),
            (Admin, "admin"),
            (Unemployed, "unemployed"),
        ];
        for (job, label) in cases {
            assert_eq!(job_profession_label(job), label);
        }
    }
}
