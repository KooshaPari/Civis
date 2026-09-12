use std::{
    sync::atomic::{AtomicU32, AtomicU64, Ordering},
    thread,
    time::Duration,
};

use civ_protocol_3d::Frame3d;

use crate::{
    EmergenceHudData, OutcomeHudData, WsConnectionState, WsSpectatorMeta,
    parse_jsonrpc_snapshot_meta, parse_ws_payload, ws_prefer_binary_from_env,
};
use crossbeam_channel::{Receiver, Sender};
use futures_util::{SinkExt, StreamExt};
use serde_json;
use std::sync::{Arc, Mutex};
use tokio::runtime::Builder;
use tokio_tungstenite::tungstenite::Message;

/// Drain all available items from a crossbeam channel into a Vec without blocking.
/// Reuses the destination's existing capacity to avoid per-frame allocation.
#[allow(dead_code)]
fn drain_into<T>(rx: &Receiver<T>, dst: &mut Vec<T>) {
    dst.clear();
    while let Ok(item) = rx.try_recv() {
        dst.push(item);
    }
}

/// Live attach WebSocket client preferences.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct WsClientConfig {
    /// When true, skip JSON text tick frames and decode binary `F3D0` payloads only.
    /// Matches `civ-server` `TickBroadcastFormat::Both` without duplicate work.
    pub prefer_binary: bool,
}

impl Default for WsClientConfig {
    fn default() -> Self {
        Self {
            prefer_binary: ws_prefer_binary_from_env(),
        }
    }
}

/// Server-authoritative live-scene replacement notification.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SceneReset {
    /// Tick represented by the replacement scene.
    pub tick: u64,
}

/// Server-reported performance counters from `sim.perf`.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SimPerfData {
    /// Last simulation tick wall-clock duration in milliseconds.
    pub tick_ms: f64,
}

/// Aggregated last-tick sim events from `sim.sim.events` (id=9011).
///
/// Bundles every per-tick buffer the engine flushes (disaster pulses, audio
/// cues, construction events, mood snapshots, research progress, religion,
/// legends, emergence sample, climate snapshot) into one round-trip so the
/// Bevy client can reflect sim state in a single poll.
///
/// Mirrors the server's `SimSimEvents` payload shape — uses `serde_json::Value`
/// for nested collections so client and server stay decoupled on item shape.
#[derive(Debug, Clone, Default)]
pub struct SimSimEventsData {
    /// Server tick this snapshot represents.
    pub tick: u64,
    /// Voxel damage events emitted this tick.
    pub damage_events_count: u32,
    /// Voxel material removed by damage this tick.
    pub voxel_damage_removed: u32,
    /// Per-event damage records (kind, voxel, target). Empty array when no
    /// snapshot was available on the server.
    pub damage_events: Vec<serde_json::Value>,
    /// SFX / ambient audio events queued this tick.
    pub audio_events: Vec<serde_json::Value>,
    /// Music cues keyed by cue id (loops, stingers).
    pub music_cues: serde_json::Value,
    /// Climate snapshot (temperature, humidity, season, day_phase).
    pub climate: Option<serde_json::Value>,
    /// Emergence dashboard sample (entropy_bits, branching, mi_score...).
    pub emergence_sample: Option<serde_json::Value>,
    /// Optional religion state blob (sects, intensities, schisms).
    pub religion_state: Option<serde_json::Value>,
    /// Optional legends/saga blob (sagas, nodes, weights).
    pub legends: Option<serde_json::Value>,
    /// Array of tech ids that completed this tick (or recently).
    pub researched: Vec<serde_json::Value>,
    /// Currently researching tech name + progress percentage.
    pub in_progress_tech: serde_json::Value,
}

/// A pending JSON-RPC request with a stable correlation ID and connection origin.
///
/// Returned by [`WsClient::request_rpc`]; the caller polls `try_recv()` until a
/// reply arrives (in tests, via [`WsClient::test_complete_rpc`]).
#[derive(Debug)]
pub struct RpcTicket {
    /// Stable correlation ID assigned by the client (matches the JSON-RPC `id`).
    pub id: u64,
    /// Connection ID that originated this request — used by callers that need
    /// to match RPCs to a particular player/session scope.
    pub connection_id: String,
    reply_rx: Receiver<Result<serde_json::Value, String>>,
}

/// Server-authoritative pending world-generation operation, held on the client.
///
/// Stored via [`WsClient::install_world_generation`] so the client can prove that
/// a freshly ACKed scene is the *one it requested* (correlation), and to drain
/// the previous live-stream scene exactly once when the new terrain lands.
struct WorldGenState {
    generation: u64,
    #[allow(dead_code)]
    connection_id: String,
    clear_fn: Option<Box<dyn FnOnce() + Send>>,
}

/// WebSocket client that bridges the tokio network task to Bevy systems.
pub struct WsClient {
    frame_rx: Receiver<Frame3d>,
    meta_rx: Receiver<WsSpectatorMeta>,
    rtt_rx: Receiver<f32>,
    state_rx: Receiver<WsConnectionState>,
    latest_state: AtomicU32,
    cmd_tx: Sender<String>,
    /// Channel for outbound JSON-RPC text frames (fire-and-forget).
    send_tx: Sender<String>,
    /// Ticketed frames are separate so failed tickets cannot replay after reconnect.
    ticket_tx: Sender<(u64, String)>,
    /// Inbound parsed EmergenceHudData from id=2 sim.emergence responses.
    emergence_rx: crossbeam_channel::Receiver<EmergenceHudData>,
    /// Inbound parsed SimPerfData from id=3 sim.perf responses.
    perf_rx: crossbeam_channel::Receiver<SimPerfData>,
    /// Inbound aggregated sim.events from id=9011 sim.sim.events responses.
    sim_events_rx: crossbeam_channel::Receiver<SimSimEventsData>,
    outcome_rx: crossbeam_channel::Receiver<OutcomeHudData>,
    save_list_rx: crossbeam_channel::Receiver<Vec<SaveListEntry>>,
    scene_reset_rx: Receiver<SceneReset>,
    /// Correlated-request state: maps ticket ID -> (connection_id, reply_sender).
    pending_rpcs: Arc<
        Mutex<std::collections::HashMap<u64, (String, Sender<Result<serde_json::Value, String>>)>>,
    >,
    /// Active world-generation state (generation id, connection id, clear closure).
    world_generation_state: Arc<Mutex<Option<WorldGenState>>>,
    /// Atomic counter assigning stable ticket IDs for `request_rpc`.
    next_request_id: Arc<AtomicU64>,
    /// Flag used by `suspend_world_stream` to pause live-scene streaming.
    suspended: Arc<Mutex<bool>>,
    /// Optional sender for test-mode inbound JSON frames.
    inbound_json_tx: Option<Sender<String>>,
    /// Stable connection identifier originating this client (matches replies).
    connection_id: String,
}

impl WsClient {
    /// Create a client that stays disconnected without starting a network task.
    ///
    /// Standalone clients still expose the same polling and command channels as
    /// live clients, but must not reconnect to the server endpoint unless the
    /// user explicitly selects server attach mode.
    #[must_use]
    pub fn disconnected() -> Self {
        let (_frame_tx, frame_rx) = crossbeam_channel::unbounded();
        let (_meta_tx, meta_rx) = crossbeam_channel::unbounded();
        let (_rtt_tx, rtt_rx) = crossbeam_channel::unbounded();
        let (_state_tx, state_rx) = crossbeam_channel::unbounded();
        let (cmd_tx, _cmd_rx) = crossbeam_channel::unbounded::<String>();
        let (send_tx, _send_rx) = crossbeam_channel::unbounded::<String>();
        let (ticket_tx, _ticket_rx) = crossbeam_channel::unbounded::<(u64, String)>();
        let (_emergence_tx, emergence_rx) = crossbeam_channel::unbounded();
        let (_perf_tx, perf_rx) = crossbeam_channel::unbounded();
        let (_sim_events_tx, sim_events_rx) = crossbeam_channel::unbounded();
        let (_outcome_tx, outcome_rx) = crossbeam_channel::unbounded();
        let (_save_list_tx, save_list_rx) = crossbeam_channel::unbounded();
        let (_scene_reset_tx, scene_reset_rx) = crossbeam_channel::unbounded();

        Self {
            frame_rx,
            meta_rx,
            rtt_rx,
            state_rx,
            latest_state: AtomicU32::new(state_to_atomic(WsConnectionState::Disconnected)),
            cmd_tx,
            send_tx,
            ticket_tx,
            emergence_rx,
            perf_rx,
            sim_events_rx,
            outcome_rx,
            save_list_rx,
            scene_reset_rx,
            pending_rpcs: Arc::new(Mutex::new(std::collections::HashMap::new())),
            world_generation_state: Arc::new(Mutex::new(None)),
            next_request_id: Arc::new(AtomicU64::new(FIRST_TICKET_ID)),
            suspended: Arc::new(Mutex::new(false)),
            inbound_json_tx: None,
            connection_id: "civis://disconnected".to_string(),
        }
    }

    /// Spawn a reconnecting WebSocket client on a dedicated tokio runtime.
    pub fn spawn(url: String) -> Self {
        Self::spawn_with_config(url, WsClientConfig::default())
    }

    /// Spawn with explicit attach preferences (binary-first tick handling).
    pub fn spawn_with_config(url: String, config: WsClientConfig) -> Self {
        let (frame_tx, frame_rx) = crossbeam_channel::unbounded();
        let (meta_tx, meta_rx) = crossbeam_channel::unbounded();
        let (rtt_tx, rtt_rx) = crossbeam_channel::unbounded();
        let (state_tx, state_rx) = crossbeam_channel::unbounded();
        let (cmd_tx, cmd_rx) = crossbeam_channel::unbounded::<String>();
        let (send_tx, send_rx) = crossbeam_channel::unbounded::<String>();
        let (ticket_tx, ticket_rx) = crossbeam_channel::unbounded::<(u64, String)>();
        let (emergence_tx, emergence_rx) = crossbeam_channel::unbounded::<EmergenceHudData>();
        let (perf_tx, perf_rx) = crossbeam_channel::unbounded::<SimPerfData>();
        let (sim_events_tx, sim_events_rx) = crossbeam_channel::unbounded::<SimSimEventsData>();
        let (outcome_tx, outcome_rx) = crossbeam_channel::unbounded::<OutcomeHudData>();
        let (save_list_tx, save_list_rx) = crossbeam_channel::unbounded::<Vec<SaveListEntry>>();
        let (scene_reset_tx, scene_reset_rx) = crossbeam_channel::unbounded::<SceneReset>();
        let connection_id = format!("civis://{}", url);
        let pending_rpcs = Arc::new(Mutex::new(std::collections::HashMap::new()));
        let network_pending_rpcs = pending_rpcs.clone();

        thread::spawn(move || {
            run_client(
                url,
                config,
                frame_tx,
                meta_tx,
                rtt_tx,
                state_tx,
                cmd_rx,
                send_rx,
                ticket_rx,
                emergence_tx,
                perf_tx,
                sim_events_tx,
                outcome_tx,
                save_list_tx,
                scene_reset_tx,
                network_pending_rpcs,
            );
        });

        Self {
            frame_rx,
            meta_rx,
            rtt_rx,
            state_rx,
            latest_state: AtomicU32::new(state_to_atomic(WsConnectionState::Disconnected)),
            cmd_tx,
            send_tx,
            ticket_tx,
            emergence_rx,
            perf_rx,
            sim_events_rx,
            outcome_rx,
            save_list_rx,
            scene_reset_rx,
            pending_rpcs,
            world_generation_state: Arc::new(Mutex::new(None)),
            next_request_id: Arc::new(AtomicU64::new(FIRST_TICKET_ID)),
            suspended: Arc::new(Mutex::new(false)),
            inbound_json_tx: None,
            connection_id: connection_id.clone(),
        }
    }

    /// Clone the outbound RPC sender so other Bevy resources can enqueue frames
    /// without holding a reference to the full `WsClient`.
    #[must_use]
    pub fn rpc_sender(&self) -> crossbeam_channel::Sender<String> {
        self.send_tx.clone()
    }

    /// Drain any parsed `sim.emergence` responses (id=2) from the background thread.
    #[must_use]
    pub fn poll_emergence(&self) -> Vec<EmergenceHudData> {
        let mut out = Vec::new();
        while let Ok(em) = self.emergence_rx.try_recv() {
            out.push(em);
        }
        out
    }

    /// Drain parsed `sim.perf` responses, returning only the newest sample.
    #[must_use]
    pub fn poll_perf(&self) -> Option<SimPerfData> {
        let mut latest = None;
        while let Ok(perf) = self.perf_rx.try_recv() {
            latest = Some(perf);
        }
        latest
    }

    /// Drain parsed `sim/sim_events` responses, returning only the newest sample.
    /// This is the unified stream of last_tick_* event buffers (damage events,
    /// audio events, research state, religion, legends, emergence sample, climate).
    #[must_use]
    pub fn poll_sim_events(&self) -> Option<SimSimEventsData> {
        let mut latest = None;
        while let Ok(ev) = self.sim_events_rx.try_recv() {
            latest = Some(ev);
        }
        latest
    }

    #[must_use]
    pub fn poll_outcome(&self) -> Option<OutcomeHudData> {
        let mut latest = None;
        while let Ok(o) = self.outcome_rx.try_recv() {
            latest = Some(o);
        }
        latest
    }

    /// Discard outcomes received before a new player session begins.
    pub fn clear_outcomes(&self) {
        drain_outcomes(&self.outcome_rx);
    }

    /// Drain save-list responses from `save.list` (id=2099) RPC replies.
    #[must_use]
    pub fn poll_save_list(&self) -> Vec<SaveListEntry> {
        let mut entries = Vec::new();
        while let Ok(batch) = self.save_list_rx.try_recv() {
            entries.extend(batch);
        }
        entries
    }

    /// Drain server-authoritative scene replacement notifications.
    #[must_use]
    pub fn poll_scene_resets(&self) -> Vec<SceneReset> {
        let mut resets = Vec::new();
        while let Ok(reset) = self.scene_reset_rx.try_recv() {
            resets.push(reset);
        }
        resets
    }

    /// Drain all currently available frames without blocking the main thread.
    #[must_use]
    pub fn poll(&self) -> Vec<Frame3d> {
        let mut frames = Vec::with_capacity(self.frame_rx.len());
        self.poll_into(&mut frames);
        frames
    }

    /// Drain all currently available frames into caller-owned storage.
    ///
    /// Render loops should reuse the destination across updates to avoid a
    /// per-frame allocation while keeping the channel non-blocking.
    pub fn poll_into(&self, frames: &mut Vec<Frame3d>) {
        while let Ok(frame) = self.frame_rx.try_recv() {
            frames.push(frame);
        }
    }

    #[must_use]
    pub fn poll_meta(&self) -> Vec<WsSpectatorMeta> {
        let mut metas = Vec::new();
        while let Ok(meta) = self.meta_rx.try_recv() {
            metas.push(meta);
        }
        metas
    }

    /// Latest measured `sim.snapshot` round-trip time in milliseconds, if any.
    #[must_use]
    pub fn latest_rtt_ms(&self) -> Option<f32> {
        let mut latest = None;
        while let Ok(ms) = self.rtt_rx.try_recv() {
            latest = Some(ms);
        }
        latest
    }

    /// Latest connection state from the background reconnect loop.
    #[must_use]
    pub fn latest_connection_state(&self) -> WsConnectionState {
        while let Ok(state) = self.state_rx.try_recv() {
            self.latest_state
                .store(state_to_atomic(state), Ordering::Relaxed);
        }
        atomic_to_state(self.latest_state.load(Ordering::Relaxed))
    }

    /// Send a fire-and-forget pre-formatted JSON-RPC command string.
    /// Drops silently if the WebSocket background task has not connected yet.
    pub fn send_rpc_raw(&self, json: String) {
        let _ = self.cmd_tx.send(json.clone());
        // Mirror outbound frames to any test observer so `test_rpc_client` callers
        // can assert on the exact JSON-RPC request sent to the server.
        if let Some(tx) = &self.inbound_json_tx {
            let _ = tx.send(json);
        }
    }

    /// Send a JSON-RPC request over the live WebSocket connection.
    ///
    /// The message is queued; the background thread forwards it on the next
    /// write iteration. Silently drops if the background thread has exited.
    pub fn send_rpc(&self, method: &str, params: serde_json::Value) {
        let msg = serde_json::json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": method,
            "params": params,
        })
        .to_string();
        let _ = self.cmd_tx.send(msg.clone());
        if let Some(tx) = &self.inbound_json_tx {
            let _ = tx.send(msg);
        }
    }

    /// Create a new pending request ticket for correlated request/response.
    ///
    /// The caller can poll this ticket until `WsClient::test_complete_rpc` is called
    /// (in tests) or until the server replies in production. The `RpcTicket` provides
    /// both a stable ID for matching replies and a "connection_id" for the caller's
    /// `install_world_generation`.
    pub fn request_rpc(&self, method: &str, params: serde_json::Value) -> RpcTicket {
        let id = self.next_request_id.fetch_add(1, Ordering::Relaxed);
        let (tx, rx) = crossbeam_channel::bounded(1);
        let connection_id = self.connection_id.clone();
        // Store the pending request so the live socket receive path can deliver
        // the matching JSON-RPC response.
        self.pending_rpcs
            .lock()
            .unwrap()
            .insert(id, (connection_id.clone(), tx));
        // Send the JSON-RPC text frame to the network task.
        let payload =
            serde_json::json!({"jsonrpc": "2.0", "id": id, "method": method, "params": params});
        let json = serde_json::to_string(&payload).unwrap();
        let _ = self.ticket_tx.send((id, json.clone()));
        self.mirror_outbound(&json);
        RpcTicket {
            id,
            connection_id,
            reply_rx: rx,
        }
    }

    /// Install a live-world generation operation into the WebSocket client.
    ///
    /// The caller passes a `generation` ID and the `connection_id` that
    /// originates this request. The previous live-stream scene is drained by the
    /// caller (via `clear_live_stream_scene_in_world`) at the same point, so the
    /// client can prove the next ACKed scene is the one it requested before it is
    /// allowed to play.
    pub fn install_world_generation(&self, generation: u64, connection_id: String) {
        let mut state = self.world_generation_state.lock().unwrap();
        *state = Some(WorldGenState {
            generation,
            connection_id,
            clear_fn: None,
        });
    }

    /// Query whether a world generation is currently active for the given generation ID.
    ///
    /// Returns `true` when `install_world_generation` was called for this generation
    /// and the generation has not yet been completed (by `finish_world_load`).
    pub fn world_generation_is_active(&self, generation: u64) -> bool {
        let state = self.world_generation_state.lock().unwrap();
        match state.as_ref() {
            Some(s) => s.generation == generation,
            None => false,
        }
    }

    /// Signal that a live-world generation has arrived and should be finalized,
    /// resetting the pending generation state to `None` (profile clear hooks, if
    /// any were installed, are invoked as part of finalizing).
    pub fn finish_world_load(&self) {
        let mut state = self.world_generation_state.lock().unwrap();
        if let Some(s) = state.take() {
            if let Some(clear_fn) = s.clear_fn {
                clear_fn();
            }
        }
    }

    /// Suspend the live-scene streaming loop, useful for tests that need to freeze
    /// the world while injecting RPC replies.
    pub fn suspend_world_stream(&self) {
        let mut suspended = self.suspended.lock().unwrap();
        *suspended = true;
    }

    /// Test-only helper to complete a pending RPC ticket with a pre-computed result.
    ///
    /// This bypasses the live network task and directly delivers a reply to the
    /// ticket's receiver, which is essential for unit tests that inject server
    /// responses.
    pub fn test_complete_rpc(&self, id: u64, result: Result<serde_json::Value, String>) {
        let mut pending = self.pending_rpcs.lock().unwrap();
        if let Some((_, tx)) = pending.remove(&id) {
            let _ = tx.send(result);
        }
    }

    /// Test-only helper to spawn a client in a mode that pushes all received JSON
    /// into a channel instead of processing them as real network events.
    ///
    /// Returns `(client, inbound_json)` where `inbound_json` receives everything
    /// the client would normally parse as a server response.
    pub fn test_rpc_client() -> (Self, Receiver<String>) {
        let (json_tx, json_rx) = crossbeam_channel::unbounded();
        let mut client = Self::disconnected();
        client.inbound_json_tx = Some(json_tx);
        (client, json_rx)
    }

    /// Mirror an outbound JSON-RPC frame to the test observer channel if one is
    /// configured (set by [`WsClient::test_rpc_client`]). This lets unit tests
    /// assert on the exact request text sent to the server via the `requests`
    /// receiver while production clients remain unaffected (`inbound_json_tx` is
    /// `None` for real network clients).
    fn mirror_outbound(&self, json: &str) {
        if let Some(tx) = &self.inbound_json_tx {
            let _ = tx.send(json.to_owned());
        }
    }
}

impl RpcTicket {
    /// Attempt to receive the reply for this request, if available.
    ///
    /// Returns `None` when no reply has arrived yet (caller must keep polling).
    /// Returns `Some(Ok(...))` on successful server response, or `Some(Err(...))`
    /// for server errors.
    pub fn try_recv(&self) -> Option<Result<serde_json::Value, String>> {
        self.reply_rx.try_recv().ok()
    }

    /// The stable connection ID that originated this request.
    ///
    /// Useful for callers that need to match RPCs with a particular client
    /// connection so they can deliver results to the correct world generation
    /// state.
    pub fn connection_id(&self) -> String {
        self.connection_id.clone()
    }
}

fn drain_outcomes(rx: &crossbeam_channel::Receiver<OutcomeHudData>) {
    while rx.try_recv().is_ok() {}
}

impl Clone for WsClient {
    fn clone(&self) -> Self {
        Self {
            frame_rx: self.frame_rx.clone(),
            meta_rx: self.meta_rx.clone(),
            rtt_rx: self.rtt_rx.clone(),
            state_rx: self.state_rx.clone(),
            latest_state: AtomicU32::new(self.latest_state.load(Ordering::Relaxed)),
            cmd_tx: self.cmd_tx.clone(),
            send_tx: self.send_tx.clone(),
            ticket_tx: self.ticket_tx.clone(),
            emergence_rx: self.emergence_rx.clone(),
            perf_rx: self.perf_rx.clone(),
            outcome_rx: self.outcome_rx.clone(),
            save_list_rx: self.save_list_rx.clone(),
            scene_reset_rx: self.scene_reset_rx.clone(),
            sim_events_rx: self.sim_events_rx.clone(),
            pending_rpcs: self.pending_rpcs.clone(),
            world_generation_state: self.world_generation_state.clone(),
            next_request_id: self.next_request_id.clone(),
            suspended: self.suspended.clone(),
            inbound_json_tx: self.inbound_json_tx.clone(),
            connection_id: self.connection_id.clone(),
        }
    }
}

/// First client-generated JSON-RPC request ID. Fixed polling IDs stay in the
/// low range, so ticket replies cannot be mistaken for poll replies.
pub const FIRST_TICKET_ID: u64 = 1 << 32;

const OUTCOME_RPC: &str = r#"{"jsonrpc":"2.0","id":9003,"method":"sim.outcome","params":{}}"#;
const OUTCOME_POLL_SECS: u64 = 30;
const SIM_EVENTS_RPC: &str = r#"{"jsonrpc":"2.0","id":9011,"method":"sim.events","params":{}}"#;
/// Poll cadence for sim.events — fast (10 Hz) because it carries ephemeral
/// disaster pulses / audio cues that the Bevy client renders immediately.
const SIM_EVENTS_POLL_SECS: u64 = 2;
const SNAPSHOT_RPC: &str = r#"{"jsonrpc":"2.0","id":9001,"method":"sim.snapshot","params":{}}"#;
const SNAPSHOT_POLL_SECS: u64 = 2;
/// While the server is quiet, wake often enough to flush user RPCs without
/// adding meaningful idle traffic or making the Bevy thread wait on a frame.
const OUTBOUND_WAKE_MILLIS: u64 = 20;

/// First reconnect delay after a disconnect.
pub const RECONNECT_BACKOFF_INITIAL_SECS: u64 = 1;
/// Maximum reconnect delay (exponential backoff cap).
pub const RECONNECT_BACKOFF_MAX_SECS: u64 = 30;

struct ReconnectBackoff {
    attempt: u32,
}

impl ReconnectBackoff {
    fn new() -> Self {
        Self { attempt: 0 }
    }

    fn reset(&mut self) {
        self.attempt = 0;
    }

    fn next_delay(&mut self) -> Duration {
        let shift = self.attempt.min(5);
        let secs = RECONNECT_BACKOFF_INITIAL_SECS
            .saturating_mul(1u64 << shift)
            .min(RECONNECT_BACKOFF_MAX_SECS);
        self.attempt = self.attempt.saturating_add(1);
        Duration::from_secs(secs)
    }
}

fn state_to_atomic(state: WsConnectionState) -> u32 {
    match state {
        WsConnectionState::Connected => 0,
        WsConnectionState::Reconnecting => 1,
        WsConnectionState::Disconnected => 2,
    }
}

fn atomic_to_state(value: u32) -> WsConnectionState {
    match value {
        0 => WsConnectionState::Connected,
        1 => WsConnectionState::Reconnecting,
        _ => WsConnectionState::Disconnected,
    }
}

fn publish_state(state_tx: &Sender<WsConnectionState>, state: WsConnectionState) {
    let _ = state_tx.send(state);
}

fn run_client(
    url: String,
    config: WsClientConfig,
    frame_tx: Sender<Frame3d>,
    meta_tx: Sender<WsSpectatorMeta>,
    rtt_tx: Sender<f32>,
    state_tx: Sender<WsConnectionState>,
    cmd_rx: Receiver<String>,
    send_rx: crossbeam_channel::Receiver<String>,
    ticket_rx: Receiver<(u64, String)>,
    emergence_tx: Sender<EmergenceHudData>,
    perf_tx: Sender<SimPerfData>,
    sim_events_tx: Sender<SimSimEventsData>,
    outcome_tx: Sender<OutcomeHudData>,
    save_list_tx: Sender<Vec<SaveListEntry>>,
    scene_reset_tx: Sender<SceneReset>,
    pending_rpcs: Arc<
        Mutex<std::collections::HashMap<u64, (String, Sender<Result<serde_json::Value, String>>)>>,
    >,
) {
    let Ok(runtime) = Builder::new_multi_thread().enable_all().build() else {
        eprintln!("bevy ws client: failed to build tokio runtime — staying disconnected");
        publish_state(&state_tx, WsConnectionState::Disconnected);
        return;
    };
    runtime.block_on(async move {
        let mut backoff = ReconnectBackoff::new();
        publish_state(&state_tx, WsConnectionState::Disconnected);
        loop {
            publish_state(&state_tx, WsConnectionState::Reconnecting);
            match connect_and_stream(
                &url,
                config,
                &frame_tx,
                &meta_tx,
                &rtt_tx,
                &state_tx,
                &cmd_rx,
                &send_rx,
                &ticket_rx,
                &emergence_tx,
                &perf_tx,
                &sim_events_tx,
                &outcome_tx,
                &save_list_tx,
                &scene_reset_tx,
                &pending_rpcs,
            )
            .await
            {
                Ok(()) => {
                    backoff.reset();
                }
                Err(err) => {
                    fail_pending_rpcs(
                        &pending_rpcs,
                        "WebSocket connection closed before the server replied.",
                    );
                    eprintln!("bevy ws client disconnected: {err}");
                    let delay = backoff.next_delay();
                    thread::sleep(delay);
                }
            }
        }
    });
}

async fn request_snapshot(
    write: &mut futures_util::stream::SplitSink<
        tokio_tungstenite::WebSocketStream<
            tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>,
        >,
        Message,
    >,
    snapshot_ping: &mut Option<std::time::Instant>,
) -> Result<(), String> {
    *snapshot_ping = Some(std::time::Instant::now());
    write
        .send(Message::Text(SNAPSHOT_RPC.into()))
        .await
        .map_err(|err| err.to_string())
}

fn record_snapshot_rtt(snapshot_ping: &mut Option<std::time::Instant>, rtt_tx: &Sender<f32>) {
    if let Some(sent) = snapshot_ping.take() {
        let _ = rtt_tx.send(sent.elapsed().as_secs_f32() * 1000.0);
    }
}

/// Parse a sim.emergence (id=2) JSON-RPC response into `EmergenceHudData`.
fn parse_emergence_response(text: &str) -> Option<EmergenceHudData> {
    let v: serde_json::Value = serde_json::from_str(text).ok()?;
    if v.get("id").and_then(|i| i.as_i64()) != Some(2) {
        return None;
    }
    let result = v.get("result")?;
    Some(EmergenceHudData {
        entropy_bits: result
            .get("entropy_bits")
            .and_then(|v| v.as_f64())
            .unwrap_or(0.0) as f32,
        entropy_norm: result
            .get("entropy_norm")
            .and_then(|v| v.as_f64())
            .unwrap_or(0.0) as f32,
        power_law_alpha: result
            .get("power_law_alpha")
            .and_then(|v| v.as_f64())
            .unwrap_or(0.0) as f32,
        novelty_rate: result
            .get("novelty_rate")
            .and_then(|v| v.as_f64())
            .unwrap_or(0.0) as f32,
        mi_material_faction_norm: result
            .get("mi_material_faction_norm")
            .and_then(|v| v.as_f64())
            .map(|f| f as f32),
        structure_count: result
            .get("structure_count")
            .and_then(|v| v.as_u64())
            .map(|n| n as u32),
        branching_regime: result
            .get("branching_regime")
            .and_then(|v| v.as_str())
            .unwrap_or("SUBCRITICAL")
            .to_owned(),
    })
}

/// Parse a `sim.perf` (id=3) JSON-RPC response into [`SimPerfData`].
fn parse_perf_response(text: &str) -> Option<SimPerfData> {
    let value: serde_json::Value = serde_json::from_str(text).ok()?;
    if value.get("id").and_then(|id| id.as_i64()) != Some(3) {
        return None;
    }
    let tick_ms = value
        .get("result")?
        .get("last_tick_ms")
        .and_then(|value| value.as_f64())?;
    if !tick_ms.is_finite() || tick_ms < 0.0 {
        return None;
    }
    Some(SimPerfData { tick_ms })
}

/// Parse a `sim.sim.events` (id=9011) JSON-RPC response into [`SimSimEventsData`].
///
/// Tolerates missing / malformed nested fields by defaulting to empty collections
/// rather than dropping the whole payload — a partial sim events snapshot is more
/// useful to the Bevy client than no snapshot at all (silent render drops are
/// debug-hostile).
fn parse_sim_events_response(text: &str) -> Option<SimSimEventsData> {
    let value: serde_json::Value = serde_json::from_str(text).ok()?;
    if value.get("id").and_then(|id| id.as_i64()) != Some(9011) {
        return None;
    }
    let result = value.get("result")?;
    Some(SimSimEventsData {
        tick: result.get("tick").and_then(|t| t.as_u64()).unwrap_or(0),
        damage_events_count: result
            .get("damage_events_count")
            .and_then(|v| v.as_u64())
            .unwrap_or(0) as u32,
        voxel_damage_removed: result
            .get("voxel_damage_removed")
            .and_then(|v| v.as_u64())
            .unwrap_or(0) as u32,
        damage_events: result
            .get("damage_events")
            .and_then(|v| v.as_array())
            .cloned()
            .unwrap_or_default()
            .into_iter()
            .filter_map(|v| match v {
                serde_json::Value::Object(_) => Some(v),
                _ => None,
            })
            .collect(),
        audio_events: result
            .get("audio_events")
            .and_then(|v| v.as_array())
            .cloned()
            .unwrap_or_default()
            .into_iter()
            .filter_map(|v| match v {
                serde_json::Value::Object(_) => Some(v),
                _ => None,
            })
            .collect(),
        music_cues: result
            .get("music_cues")
            .cloned()
            .unwrap_or_else(|| serde_json::json!({})),
        climate: result.get("climate").cloned().filter(|v| v.is_object()),
        emergence_sample: result
            .get("emergence_sample")
            .cloned()
            .filter(|v| v.is_object()),
        religion_state: result
            .get("religion_state")
            .cloned()
            .filter(|v| v.is_object()),
        legends: result.get("legends").cloned().filter(|v| v.is_object()),
        researched: result
            .get("researched")
            .and_then(|v| v.as_array())
            .cloned()
            .unwrap_or_default()
            .into_iter()
            .filter_map(|v| match v {
                serde_json::Value::String(_) | serde_json::Value::Object(_) => Some(v),
                _ => None,
            })
            .collect(),
        in_progress_tech: result
            .get("in_progress_tech")
            .cloned()
            .unwrap_or_else(|| serde_json::json!({})),
    })
}

fn parse_outcome_response(text: &str) -> Option<OutcomeHudData> {
    let v: serde_json::Value = serde_json::from_str(text).ok()?;
    if v.get("id").and_then(|i| i.as_i64()) != Some(9003) {
        return None;
    }
    let result = v.get("result")?;
    Some(OutcomeHudData {
        tag: result
            .get("outcome")
            .and_then(|v| v.as_str())
            .unwrap_or("ongoing")
            .to_owned(),
        reason: result
            .get("reason")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_owned(),
        tick: result.get("tick").and_then(|v| v.as_u64()).unwrap_or(0),
        progress: result
            .get("progress")
            .and_then(|value| serde_json::from_value(value.clone()).ok()),
    })
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SaveListEntry {
    pub name: String,
    pub tick: u64,
    pub save_type: String,
}

fn parse_save_list_response(text: &str) -> Option<Vec<SaveListEntry>> {
    let v: serde_json::Value = serde_json::from_str(text).ok()?;
    let id = v.get("id")?;
    let id_ok = id.as_u64() == Some(2099) || id.as_i64() == Some(2099);
    if !id_ok {
        return None;
    }
    let entries = v.get("result")?.as_array()?;
    let mut out = Vec::new();
    for entry in entries {
        let Some(name) = entry.get("name").and_then(|v| v.as_str()) else {
            continue;
        };
        let Some(save_type) = entry.get("save_type").and_then(|v| v.as_str()) else {
            continue;
        };
        let tick = entry.get("tick").and_then(|v| v.as_u64()).unwrap_or(0);
        out.push(SaveListEntry {
            name: name.to_string(),
            tick,
            save_type: save_type.to_string(),
        });
    }
    Some(out)
}

fn parse_scene_reset_notification(text: &str) -> Option<SceneReset> {
    let value: serde_json::Value = serde_json::from_str(text).ok()?;
    if value.get("method").and_then(|method| method.as_str()) != Some("scene.reset") {
        return None;
    }
    Some(SceneReset {
        tick: value
            .get("params")?
            .get("tick")
            .and_then(|tick| tick.as_u64())?,
    })
}

/// Deliver a JSON-RPC response to its matching ticket, if the client owns it.
///
/// Ticket IDs begin above the fixed polling range; routing tickets first keeps
/// each response on its exact request/response path.
fn complete_pending_rpc(
    pending_rpcs: &Arc<
        Mutex<std::collections::HashMap<u64, (String, Sender<Result<serde_json::Value, String>>)>>,
    >,
    text: &str,
) -> bool {
    let Ok(value) = serde_json::from_str::<serde_json::Value>(text) else {
        return false;
    };
    let Some(id) = value.get("id").and_then(serde_json::Value::as_u64) else {
        return false;
    };
    if value.get("result").is_none() && value.get("error").is_none() {
        return false;
    }
    let Some((_, tx)) = pending_rpcs.lock().unwrap().remove(&id) else {
        return false;
    };
    let reply = match value.get("result") {
        Some(result) => Ok(result.clone()),
        None => Err(value
            .get("error")
            .and_then(|error| error.get("message"))
            .and_then(serde_json::Value::as_str)
            .map(str::to_owned)
            .unwrap_or_else(|| "JSON-RPC request failed".to_owned())),
    };
    let _ = tx.send(reply);
    true
}

/// Resolve every outstanding ticket after a connection ends.
///
/// Requests are scoped to a socket session: a later reconnect must never leave
/// UI operations waiting for a response that was lost with the old socket.
fn fail_pending_rpcs(
    pending_rpcs: &Arc<
        Mutex<std::collections::HashMap<u64, (String, Sender<Result<serde_json::Value, String>>)>>,
    >,
    message: &str,
) {
    let pending = std::mem::take(&mut *pending_rpcs.lock().unwrap());
    for (_, (_, tx)) in pending {
        let _ = tx.send(Err(message.to_owned()));
    }
}

async fn connect_and_stream(
    url: &str,
    config: WsClientConfig,
    frame_tx: &Sender<Frame3d>,
    meta_tx: &Sender<WsSpectatorMeta>,
    rtt_tx: &Sender<f32>,
    state_tx: &Sender<WsConnectionState>,
    cmd_rx: &Receiver<String>,
    send_rx: &crossbeam_channel::Receiver<String>,
    ticket_rx: &Receiver<(u64, String)>,
    emergence_tx: &Sender<EmergenceHudData>,
    perf_tx: &Sender<SimPerfData>,
    sim_events_tx: &Sender<SimSimEventsData>,
    outcome_tx: &Sender<OutcomeHudData>,
    save_list_tx: &Sender<Vec<SaveListEntry>>,
    scene_reset_tx: &Sender<SceneReset>,
    pending_rpcs: &Arc<
        Mutex<std::collections::HashMap<u64, (String, Sender<Result<serde_json::Value, String>>)>>,
    >,
) -> Result<(), String> {
    let (ws, _) = tokio_tungstenite::connect_async(url)
        .await
        .map_err(|err| err.to_string())?;
    publish_state(state_tx, WsConnectionState::Connected);

    let (mut write, mut read) = ws.split();

    let mut snapshot_ping = None;
    request_snapshot(&mut write, &mut snapshot_ping).await?;

    let mut last_snapshot = std::time::Instant::now();
    let mut last_outcome = std::time::Instant::now();
    let mut last_sim_events = std::time::Instant::now();
    let mut outbound_wake = tokio::time::interval(Duration::from_millis(OUTBOUND_WAKE_MILLIS));
    outbound_wake.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);

    loop {
        // Flush outbound commands (speed/pause RPCs) before blocking on next inbound frame.
        while let Ok(cmd) = cmd_rx.try_recv() {
            write
                .send(Message::Text(cmd.into()))
                .await
                .map_err(|e| e.to_string())?;
        }

        // Drain any outbound RPC frames queued by Bevy systems.
        while let Ok(json) = send_rx.try_recv() {
            write
                .send(Message::Text(json.into()))
                .await
                .map_err(|e| e.to_string())?;
        }

        // A ticket failed on an earlier socket must not be replayed after a
        // reconnect. Raw frames above retain their fire-and-forget semantics.
        while let Ok((id, json)) = ticket_rx.try_recv() {
            if !pending_rpcs.lock().unwrap().contains_key(&id) {
                continue;
            }
            write
                .send(Message::Text(json.into()))
                .await
                .map_err(|e| e.to_string())?;
        }

        if last_outcome.elapsed() >= Duration::from_secs(OUTCOME_POLL_SECS) {
            write
                .send(Message::Text(OUTCOME_RPC.into()))
                .await
                .map_err(|e| e.to_string())?;
            last_outcome = std::time::Instant::now();
        }
        if last_sim_events.elapsed() >= Duration::from_secs(SIM_EVENTS_POLL_SECS) {
            write
                .send(Message::Text(SIM_EVENTS_RPC.into()))
                .await
                .map_err(|e| e.to_string())?;
            last_sim_events = std::time::Instant::now();
        }
        if last_snapshot.elapsed() >= Duration::from_secs(SNAPSHOT_POLL_SECS) {
            request_snapshot(&mut write, &mut snapshot_ping).await?;
            last_snapshot = std::time::Instant::now();
        }

        let msg = tokio::select! {
            message = read.next() => match message {
                Some(Ok(message)) => message,
                Some(Err(error)) => return Err(format!("websocket receiver error: {error}")),
                None => return Err("websocket receiver reached EOF".into()),
            },
            _ = outbound_wake.tick() => continue,
        };
        match msg {
            Message::Text(text) => {
                if complete_pending_rpc(pending_rpcs, &text) {
                    continue;
                }
                if let Some(reset) = parse_scene_reset_notification(&text) {
                    let _ = scene_reset_tx.send(reset);
                    continue;
                }
                if let Some(meta) = parse_jsonrpc_snapshot_meta(&text) {
                    record_snapshot_rtt(&mut snapshot_ping, rtt_tx);
                    if meta_tx.send(meta).is_err() {
                        return Err("bevy meta receiver dropped".into());
                    }
                    continue;
                }
                if let Some(events) = parse_sim_events_response(&text) {
                    let _ = sim_events_tx.send(events);
                    continue;
                }
                if let Some(em) = parse_emergence_response(&text) {
                    let _ = emergence_tx.send(em);
                    continue;
                }
                if let Some(perf) = parse_perf_response(&text) {
                    let _ = perf_tx.send(perf);
                    continue;
                }
                if let Some(oc) = parse_outcome_response(&text) {
                    let _ = outcome_tx.send(oc);
                    continue;
                }
                if let Some(entries) = parse_save_list_response(&text) {
                    let _ = save_list_tx.send(entries);
                    continue;
                }
                if config.prefer_binary {
                    continue;
                }
                let frame = parse_ws_payload(text.as_bytes())?;
                if frame_tx.send(frame).is_err() {
                    return Err("bevy frame receiver dropped".into());
                }
            }
            Message::Binary(bytes) => {
                let frame = parse_ws_payload(&bytes)?;
                if frame_tx.send(frame).is_err() {
                    return Err("bevy frame receiver dropped".into());
                }
            }
            Message::Close(frame) => {
                return Err(format!("websocket close frame: {frame:?}"));
            }
            _ => {}
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reconnect_backoff_doubles_until_cap() {
        let mut backoff = ReconnectBackoff::new();
        assert_eq!(backoff.next_delay(), Duration::from_secs(1));
        assert_eq!(backoff.next_delay(), Duration::from_secs(2));
        assert_eq!(backoff.next_delay(), Duration::from_secs(4));
        assert_eq!(backoff.next_delay(), Duration::from_secs(8));
        assert_eq!(backoff.next_delay(), Duration::from_secs(16));
        assert_eq!(backoff.next_delay(), Duration::from_secs(30));
        assert_eq!(backoff.next_delay(), Duration::from_secs(30));
    }

    #[test]
    fn drain_into_reuses_capacity_across_bursts() {
        let (sender, receiver) = crossbeam_channel::unbounded();
        let mut items = Vec::with_capacity(8);
        let capacity = items.capacity();

        sender.send(1_u8).expect("first item");
        sender.send(2_u8).expect("second item");
        drain_into(&receiver, &mut items);
        assert_eq!(items, vec![1, 2]);
        assert_eq!(items.capacity(), capacity);

        drain_into(&receiver, &mut items);
        assert!(items.is_empty());
        assert_eq!(items.capacity(), capacity);
    }

    #[test]
    fn parse_outcome_response_reads_victory_payload() {
        let text = r#"{"jsonrpc":"2.0","id":9003,"result":{"outcome":"victory","reason":"population","tick":99,"progress":{"population":10000,"population_target":10000,"researched_techs":4,"researched_techs_target":12,"peace_ticks":10,"peace_ticks_target":500}}}"#;
        let outcome = parse_outcome_response(text).expect("outcome");
        assert_eq!(outcome.tag, "victory");
        assert_eq!(outcome.reason, "population");
        assert_eq!(outcome.tick, 99);
        let progress = outcome.progress.expect("progress");
        assert_eq!(progress.population, 10_000);
        assert_eq!(progress.peace_ticks, 10);
    }

    #[test]
    fn parse_perf_response_reads_tick_duration() {
        let text = r#"{"jsonrpc":"2.0","id":3,"result":{"last_tick_ms":8.25}}"#;
        assert_eq!(
            parse_perf_response(text),
            Some(SimPerfData { tick_ms: 8.25 })
        );
    }

    #[test]
    fn parse_perf_response_rejects_wrong_id_errors_and_invalid_values() {
        let wrong_id = r#"{"jsonrpc":"2.0","id":2,"result":{"last_tick_ms":8.25}}"#;
        let error = r#"{"jsonrpc":"2.0","id":3,"error":{"code":-32000,"message":"busy"}}"#;
        let negative = r#"{"jsonrpc":"2.0","id":3,"result":{"last_tick_ms":-1.0}}"#;
        let malformed = "not json";
        assert!(parse_perf_response(wrong_id).is_none());
        assert!(parse_perf_response(error).is_none());
        assert!(parse_perf_response(negative).is_none());
        assert!(parse_perf_response(malformed).is_none());
    }

    #[test]
    fn parse_outcome_response_ignores_other_rpc_ids() {
        let text = r#"{"jsonrpc":"2.0","id":3,"result":{"outcome":"victory"}}"#;
        assert!(parse_outcome_response(text).is_none());
    }

    #[test]
    fn drain_outcomes_removes_queued_session_results() {
        let (tx, rx) = crossbeam_channel::unbounded();
        tx.send(OutcomeHudData {
            tag: "victory".to_string(),
            ..Default::default()
        })
        .expect("queue outcome");
        drain_outcomes(&rx);
        assert!(rx.try_recv().is_err());
    }

    #[test]
    fn ticket_ids_are_disjoint_from_builtin_poll_ids() {
        assert!(FIRST_TICKET_ID > 1);
        assert!(FIRST_TICKET_ID > 2);
        assert!(FIRST_TICKET_ID > 3);
        assert!(FIRST_TICKET_ID > 9001);
        assert!(FIRST_TICKET_ID > 9003);
        assert!(FIRST_TICKET_ID > 9011);
        let client = WsClient::disconnected();
        assert_eq!(
            client.request_rpc("sim.test", serde_json::json!({})).id,
            FIRST_TICKET_ID
        );
    }

    #[test]
    fn live_socket_routes_ticket_results_errors_notifications_and_disconnects() {
        let listener = std::net::TcpListener::bind("127.0.0.1:0").expect("bind test websocket");
        listener.set_nonblocking(true).expect("set nonblocking");
        let address = listener.local_addr().expect("test websocket address");
        let (server_done_tx, server_done_rx) = std::sync::mpsc::channel();

        let server = thread::spawn(move || {
            Builder::new_current_thread()
                .enable_all()
                .build()
                .expect("server runtime")
                .block_on(async move {
                    let listener = tokio::net::TcpListener::from_std(listener).expect("tokio listener");
                    let (stream, _) = listener.accept().await.expect("accept client");
                    let mut socket = tokio_tungstenite::accept_async(stream)
                        .await
                        .expect("websocket handshake");
                    while let Some(Ok(Message::Text(text))) = socket.next().await {
                        let value: serde_json::Value = serde_json::from_str(&text).expect("json request");
                        match value.get("id").and_then(serde_json::Value::as_u64) {
                            Some(9001) => {
                                socket
                                    .send(Message::Text(
                                        r#"{"jsonrpc":"2.0","id":9001,"result":{"is_day":true,"tick":1}}"#.into(),
                                    ))
                                    .await
                                    .expect("snapshot response");
                                socket
                                    .send(Message::Text(
                                        r#"{"jsonrpc":"2.0","id":2,"result":{"entropy_bits":7.0,"entropy_norm":0.5,"power_law_alpha":1.2,"novelty_rate":0.1}}"#.into(),
                                    ))
                                    .await
                                    .expect("reserved poll response");
                            }
                            Some(id) if id == FIRST_TICKET_ID => {
                                socket
                                    .send(Message::Text(
                                        r#"{"jsonrpc":"2.0","method":"scene.reset","params":{"tick":2}}"#.into(),
                                    ))
                                    .await
                                    .expect("scene notification");
                                socket
                                    .send(Message::Text(
                                        format!(r#"{{"jsonrpc":"2.0","id":{id},"result":{{"accepted":true}}}}"#).into(),
                                    ))
                                    .await
                                    .expect("first ticket response");
                            }
                            Some(id) if id == FIRST_TICKET_ID + 1 => socket
                                .send(Message::Text(
                                    format!(r#"{{"jsonrpc":"2.0","id":{id},"error":{{"code":-32001,"message":"denied"}}}}"#).into(),
                                ))
                                .await
                                .expect("second ticket error"),
                            Some(id) if id == FIRST_TICKET_ID + 2 => break,
                            _ => {}
                        }
                    }
                });
            let _ = server_done_tx.send(());
        });

        let client = WsClient::spawn_with_config(
            format!("ws://{address}"),
            WsClientConfig {
                prefer_binary: true,
            },
        );
        let initial_deadline = std::time::Instant::now() + Duration::from_secs(1);
        let mut initial_emergence = Vec::new();
        while std::time::Instant::now() < initial_deadline && initial_emergence.is_empty() {
            initial_emergence = client.poll_emergence();
            thread::sleep(Duration::from_millis(10));
        }
        assert_eq!(
            initial_emergence.len(),
            1,
            "client consumed the reserved poll reply"
        );
        thread::sleep(Duration::from_millis(OUTBOUND_WAKE_MILLIS * 3));
        let accepted = client.request_rpc("sim.first", serde_json::json!({}));
        let denied = client.request_rpc("sim.second", serde_json::json!({}));
        let disconnected = client.request_rpc("sim.third", serde_json::json!({}));

        let deadline = std::time::Instant::now() + Duration::from_secs(3);
        let mut accepted_reply = None;
        let mut denied_reply = None;
        let mut disconnected_reply = None;
        while std::time::Instant::now() < deadline
            && (accepted_reply.is_none() || denied_reply.is_none() || disconnected_reply.is_none())
        {
            accepted_reply = accepted_reply.or_else(|| accepted.try_recv());
            denied_reply = denied_reply.or_else(|| denied.try_recv());
            disconnected_reply = disconnected_reply.or_else(|| disconnected.try_recv());
            thread::sleep(Duration::from_millis(10));
        }

        assert_eq!(
            accepted_reply,
            Some(Ok(serde_json::json!({"accepted": true})))
        );
        assert_eq!(denied_reply, Some(Err("denied".to_owned())));
        assert!(matches!(
            disconnected_reply,
            Some(Err(message)) if message.contains("connection closed")
        ));
        assert_eq!(client.poll_scene_resets(), vec![SceneReset { tick: 2 }]);
        server_done_rx
            .recv_timeout(Duration::from_secs(1))
            .expect("server completed ticket exchange");
        server.join().expect("server exited");
    }
}
