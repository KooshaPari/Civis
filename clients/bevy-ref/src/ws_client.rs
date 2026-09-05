use std::{
    collections::HashMap,
    sync::{
        atomic::{AtomicU32, AtomicU64, Ordering},
        Arc, Mutex, Weak,
    },
    thread,
    time::Duration,
};

use civ_protocol_3d::Frame3d;

use crate::{
    parse_jsonrpc_snapshot_meta, parse_ws_payload, ws_prefer_binary_from_env, EmergenceHudData,
    OutcomeHudData, WsConnectionState, WsSpectatorMeta,
};
use crossbeam_channel::{Receiver, Sender};
use futures_util::{SinkExt, StreamExt};
use serde_json;
use tokio::runtime::Builder;
use tokio_tungstenite::tungstenite::Message;

/// Drain all available items from a crossbeam channel into a Vec without blocking.
/// Reuses the destination's existing capacity to avoid per-frame allocation.
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

/// A correlated request. Polling never blocks the render thread.
#[derive(Debug)]
pub struct RpcTicket {
    pub id: u64,
    connection: u64,
    reply: Receiver<Result<serde_json::Value, String>>,
    state: Weak<Mutex<RpcState>>,
}

impl Drop for RpcTicket {
    fn drop(&mut self) {
        if let Some(state) = self.state.upgrade() {
            state
                .lock()
                .expect("RPC state lock")
                .pending
                .remove(&self.id);
        }
    }
}

impl RpcTicket {
    pub(crate) fn connection_id(&self) -> u64 {
        self.connection
    }

    pub fn try_recv(&self) -> Option<Result<serde_json::Value, String>> {
        if self
            .state
            .upgrade()
            .is_none_or(|state| state.lock().expect("RPC state lock").connection != self.connection)
        {
            return Some(Err(
                "Connection changed before the reply was applied. Retry the operation.".to_owned(),
            ));
        }
        match self.reply.try_recv() {
            Ok(reply) => Some(reply),
            Err(crossbeam_channel::TryRecvError::Empty) => None,
            Err(crossbeam_channel::TryRecvError::Disconnected) => Some(Err(
                "The server request was interrupted. Retry the operation.".to_owned(),
            )),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum StreamGate {
    Unrestricted,
    Suspended,
    Generation { connection: u64, generation: u64 },
}

#[derive(Debug)]
struct RpcState {
    pending: HashMap<u64, Sender<Result<serde_json::Value, String>>>,
    connection: u64,
    generation: Option<u64>,
    gate: StreamGate,
}

impl Default for RpcState {
    fn default() -> Self {
        Self {
            pending: HashMap::new(),
            connection: 0,
            generation: None,
            gate: StreamGate::Unrestricted,
        }
    }
}

type SharedRpcState = Arc<Mutex<RpcState>>;

struct ReceivedFrame {
    connection: u64,
    generation: Option<u64>,
    frame: Frame3d,
}

fn route_rpc_reply(text: &str, state: &SharedRpcState) -> bool {
    let Ok(value) = serde_json::from_str::<serde_json::Value>(text) else {
        return false;
    };
    let Some(id) = value.get("id").and_then(serde_json::Value::as_u64) else {
        return false;
    };
    let mut state = state.lock().expect("RPC state lock");
    let Some(reply) = state.pending.remove(&id) else {
        return id >= 1_000_000;
    };
    let result = if let Some(error) = value.get("error") {
        Err(error
            .get("message")
            .and_then(serde_json::Value::as_str)
            .unwrap_or("Server rejected the request")
            .to_owned())
    } else {
        value
            .get("result")
            .cloned()
            .ok_or_else(|| "Server reply has no result".to_owned())
    };
    let _ = reply.send(result);
    true
}

fn interrupt_requests(state: &SharedRpcState) {
    let mut state = state.lock().expect("RPC state lock");
    for (_, reply) in state.pending.drain() {
        let _ = reply.send(Err(
            "Connection interrupted. Reconnect and retry.".to_owned()
        ));
    }
    state.connection = state.connection.wrapping_add(1);
    state.generation = None;
    if state.gate != StreamGate::Unrestricted {
        state.gate = StreamGate::Suspended;
    }
}

fn enqueue_frame(
    frame: Frame3d,
    tx: &Sender<ReceivedFrame>,
    state: &SharedRpcState,
) -> Result<(), String> {
    let state = state.lock().expect("RPC state lock");
    tx.send(ReceivedFrame {
        connection: state.connection,
        generation: state.generation,
        frame,
    })
    .map_err(|_| "bevy frame receiver dropped".to_owned())
}

fn queued_request_is_active(text: &str, state: &SharedRpcState) -> bool {
    let id = serde_json::from_str::<serde_json::Value>(text)
        .ok()
        .and_then(|value| value.get("id").and_then(serde_json::Value::as_u64));
    !id.is_some_and(|id| {
        id >= 1_000_000
            && !state
                .lock()
                .expect("RPC state lock")
                .pending
                .contains_key(&id)
    })
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

/// WebSocket client that bridges the tokio network task to Bevy systems.
pub struct WsClient {
    frame_rx: Receiver<ReceivedFrame>,
    meta_rx: Receiver<WsSpectatorMeta>,
    rtt_rx: Receiver<f32>,
    state_rx: Receiver<WsConnectionState>,
    latest_state: AtomicU32,
    cmd_tx: Sender<String>,
    /// Channel for outbound JSON-RPC text frames (fire-and-forget).
    send_tx: Sender<String>,
    /// Inbound parsed EmergenceHudData from id=2 sim.emergence responses.
    emergence_rx: crossbeam_channel::Receiver<EmergenceHudData>,
    /// Inbound parsed SimPerfData from id=3 sim.perf responses.
    perf_rx: crossbeam_channel::Receiver<SimPerfData>,
    /// Inbound aggregated sim.events from id=9011 sim.sim.events responses.
    sim_events_rx: crossbeam_channel::Receiver<SimSimEventsData>,
    outcome_rx: crossbeam_channel::Receiver<OutcomeHudData>,
    save_list_rx: crossbeam_channel::Receiver<Vec<SaveListEntry>>,
    scene_reset_rx: Receiver<SceneReset>,
    rpc_state: SharedRpcState,
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
        let (_emergence_tx, emergence_rx) = crossbeam_channel::unbounded();
        let (_perf_tx, perf_rx) = crossbeam_channel::unbounded();
        let (_sim_events_tx, sim_events_rx) = crossbeam_channel::unbounded();
        let (_outcome_tx, outcome_rx) = crossbeam_channel::unbounded();
        let (_save_list_tx, save_list_rx) = crossbeam_channel::unbounded();
        let (_scene_reset_tx, scene_reset_rx) = crossbeam_channel::unbounded();
        let rpc_state = SharedRpcState::default();

        Self {
            frame_rx,
            meta_rx,
            rtt_rx,
            state_rx,
            latest_state: AtomicU32::new(state_to_atomic(WsConnectionState::Disconnected)),
            cmd_tx,
            send_tx,
            emergence_rx,
            perf_rx,
            sim_events_rx,
            outcome_rx,
            save_list_rx,
            scene_reset_rx,
            rpc_state,
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
        let (emergence_tx, emergence_rx) = crossbeam_channel::unbounded::<EmergenceHudData>();
        let (perf_tx, perf_rx) = crossbeam_channel::unbounded::<SimPerfData>();
        let (sim_events_tx, sim_events_rx) = crossbeam_channel::unbounded::<SimSimEventsData>();
        let (outcome_tx, outcome_rx) = crossbeam_channel::unbounded::<OutcomeHudData>();
        let (save_list_tx, save_list_rx) = crossbeam_channel::unbounded::<Vec<SaveListEntry>>();
        let (scene_reset_tx, scene_reset_rx) = crossbeam_channel::unbounded::<SceneReset>();

        let rpc_state = SharedRpcState::default();
        let network_rpc_state = Arc::clone(&rpc_state);
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
                emergence_tx,
                perf_tx,
                sim_events_tx,
                outcome_tx,
                save_list_tx,
                scene_reset_tx,
                network_rpc_state,
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
            emergence_rx,
            perf_rx,
            sim_events_rx,
            outcome_rx,
            save_list_rx,
            scene_reset_rx,
            rpc_state,
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
        if self.rpc_state.lock().expect("RPC state lock").gate != StreamGate::Unrestricted {
            // Tracked world boots clear the scene and admit a generation together.
            while self.scene_reset_rx.try_recv().is_ok() {}
            return resets;
        }
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
        let state = self.rpc_state.lock().expect("RPC state lock");
        if state.gate == StreamGate::Suspended {
            return;
        }
        while let Ok(frame) = self.frame_rx.try_recv() {
            let admitted = match state.gate {
                StreamGate::Unrestricted => {
                    frame.connection == state.connection && frame.generation == state.generation
                }
                StreamGate::Generation {
                    connection,
                    generation,
                } => frame.connection == connection && frame.generation == Some(generation),
                StreamGate::Suspended => false,
            };
            if admitted {
                frames.push(frame.frame);
            }
        }
    }

    /// Hold incoming world frames until an acknowledged replacement is installed.
    pub fn suspend_world_stream(&self) {
        self.rpc_state.lock().expect("RPC state lock").gate = StreamGate::Suspended;
    }

    /// Apply the scene clear and frame admission under one connection check.
    /// An ACK from a previous socket cannot admit a reused server generation.
    pub fn install_world_generation(
        &self,
        generation: u64,
        connection: u64,
        clear: impl FnOnce(),
    ) -> bool {
        let mut state = self.rpc_state.lock().expect("RPC state lock");
        if state.connection != connection {
            return false;
        }
        clear();
        // Replies received before the load ACK describe the previous world.
        while self.meta_rx.try_recv().is_ok() {}
        while self.outcome_rx.try_recv().is_ok() {}
        while self.sim_events_rx.try_recv().is_ok() {}
        while self.emergence_rx.try_recv().is_ok() {}
        state.gate = StreamGate::Generation {
            connection,
            generation,
        };
        true
    }

    /// A reconnect invalidates the generation even when the new server reuses its number.
    pub fn world_generation_is_active(&self, generation: u64) -> bool {
        let state = self.rpc_state.lock().expect("RPC state lock");
        matches!(state.gate, StreamGate::Generation { connection, generation: expected }
            if connection == state.connection && expected == generation)
            && state.generation.is_none_or(|seen| seen <= generation)
    }

    /// Release bootstrap admission once its terrain is installed, or after cancelling.
    pub fn finish_world_load(&self) {
        let mut state = self.rpc_state.lock().expect("RPC state lock");
        while self.scene_reset_rx.try_recv().is_ok() {}
        state.gate = StreamGate::Unrestricted;
    }

    /// Queue an RPC with an ID reserved independently of legacy polling requests.
    pub fn request_rpc(&self, method: &str, params: serde_json::Value) -> RpcTicket {
        static NEXT_REQUEST: AtomicU64 = AtomicU64::new(1_000_000);
        let id = NEXT_REQUEST.fetch_add(1, Ordering::Relaxed);
        let (reply, receiver) = crossbeam_channel::bounded(1);
        let connection = {
            let mut state = self.rpc_state.lock().expect("RPC state lock");
            state.pending.insert(id, reply);
            state.connection
        };
        let request =
            serde_json::json!({"jsonrpc":"2.0", "id":id, "method":method, "params":params});
        if self.cmd_tx.send(request.to_string()).is_err() {
            if let Some(reply) = self
                .rpc_state
                .lock()
                .expect("RPC state lock")
                .pending
                .remove(&id)
            {
                let _ = reply.send(Err(
                    "No server connection is available. Reconnect and retry.".to_owned(),
                ));
            }
        }
        RpcTicket {
            id,
            connection,
            reply: receiver,
            state: Arc::downgrade(&self.rpc_state),
        }
    }

    #[cfg(test)]
    pub(crate) fn test_rpc_client() -> (Self, Receiver<String>) {
        let mut client = Self::disconnected();
        let (tx, rx) = crossbeam_channel::unbounded();
        client.cmd_tx = tx;
        (client, rx)
    }

    #[cfg(test)]
    pub(crate) fn test_complete_rpc(&self, id: u64, result: Result<serde_json::Value, &str>) {
        let value = match result {
            Ok(result) => serde_json::json!({"id":id, "result":result}),
            Err(error) => serde_json::json!({"id":id, "error":{"message":error}}),
        };
        assert!(route_rpc_reply(&value.to_string(), &self.rpc_state));
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
        let _ = self.cmd_tx.send(json);
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
        let _ = self.cmd_tx.send(msg);
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
            emergence_rx: self.emergence_rx.clone(),
            perf_rx: self.perf_rx.clone(),
            outcome_rx: self.outcome_rx.clone(),
            save_list_rx: self.save_list_rx.clone(),
            scene_reset_rx: self.scene_reset_rx.clone(),
            rpc_state: Arc::clone(&self.rpc_state),
            sim_events_rx: self.sim_events_rx.clone(),
        }
    }
}

const OUTCOME_RPC: &str = r#"{"jsonrpc":"2.0","id":9003,"method":"sim.outcome","params":{}}"#;
const OUTCOME_POLL_SECS: u64 = 30;
const SIM_EVENTS_RPC: &str = r#"{"jsonrpc":"2.0","id":9011,"method":"sim.events","params":{}}"#;
/// Poll cadence for sim.events — fast (10 Hz) because it carries ephemeral
/// disaster pulses / audio cues that the Bevy client renders immediately.
const SIM_EVENTS_POLL_SECS: u64 = 2;
const SNAPSHOT_RPC: &str = r#"{"jsonrpc":"2.0","id":9001,"method":"sim.snapshot","params":{}}"#;
const SNAPSHOT_POLL_SECS: u64 = 2;

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
    frame_tx: Sender<ReceivedFrame>,
    meta_tx: Sender<WsSpectatorMeta>,
    rtt_tx: Sender<f32>,
    state_tx: Sender<WsConnectionState>,
    cmd_rx: Receiver<String>,
    send_rx: crossbeam_channel::Receiver<String>,
    emergence_tx: Sender<EmergenceHudData>,
    perf_tx: Sender<SimPerfData>,
    sim_events_tx: Sender<SimSimEventsData>,
    outcome_tx: Sender<OutcomeHudData>,
    save_list_tx: Sender<Vec<SaveListEntry>>,
    scene_reset_tx: Sender<SceneReset>,
    rpc_state: SharedRpcState,
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
                &emergence_tx,
                &perf_tx,
                &sim_events_tx,
                &outcome_tx,
                &save_list_tx,
                &scene_reset_tx,
                &rpc_state,
            )
            .await
            {
                Ok(()) => {
                    interrupt_requests(&rpc_state);
                    backoff.reset();
                }
                Err(err) => {
                    interrupt_requests(&rpc_state);
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

async fn connect_and_stream(
    url: &str,
    config: WsClientConfig,
    frame_tx: &Sender<ReceivedFrame>,
    meta_tx: &Sender<WsSpectatorMeta>,
    rtt_tx: &Sender<f32>,
    state_tx: &Sender<WsConnectionState>,
    cmd_rx: &Receiver<String>,
    send_rx: &crossbeam_channel::Receiver<String>,
    emergence_tx: &Sender<EmergenceHudData>,
    perf_tx: &Sender<SimPerfData>,
    sim_events_tx: &Sender<SimSimEventsData>,
    outcome_tx: &Sender<OutcomeHudData>,
    save_list_tx: &Sender<Vec<SaveListEntry>>,
    scene_reset_tx: &Sender<SceneReset>,
    rpc_state: &SharedRpcState,
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

    loop {
        // Flush outbound commands (speed/pause RPCs) before blocking on next inbound frame.
        while let Ok(cmd) = cmd_rx.try_recv() {
            if !queued_request_is_active(&cmd, rpc_state) {
                continue; // A cancelled/disconnected ticket must never replay on reconnect.
            }
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

        let msg = match tokio::time::timeout(Duration::from_millis(100), read.next()).await {
            Ok(Some(msg)) => msg.map_err(|err| err.to_string())?,
            Ok(None) => break,
            Err(_) => continue, // Keep flushing user commands even when a paused server emits no ticks.
        };
        match msg {
            Message::Text(text) => {
                if route_rpc_reply(&text, rpc_state) {
                    continue;
                }
                if let Some(reset) = parse_scene_reset_notification(&text) {
                    let value: serde_json::Value =
                        serde_json::from_str(&text).map_err(|e| e.to_string())?;
                    rpc_state.lock().expect("RPC state lock").generation =
                        value["params"]["scene_generation"].as_u64();
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
                enqueue_frame(frame, frame_tx, rpc_state)?;
            }
            Message::Binary(bytes) => {
                let frame = parse_ws_payload(&bytes)?;
                enqueue_frame(frame, frame_tx, rpc_state)?;
            }
            _ => {}
        }
    }

    Err("websocket closed".into())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rpc_ticket_correlates_errors_and_ignores_cancelled_replies() {
        let (client, queue) = WsClient::test_rpc_client();
        let first = client.request_rpc("sim.load_scenario", serde_json::json!({}));
        let second = client.request_rpc("sim.terraform_extent", serde_json::json!({}));
        assert_ne!(first.id, second.id);
        client.test_complete_rpc(second.id, Err("permission denied"));
        assert_eq!(second.try_recv(), Some(Err("permission denied".to_owned())));
        assert!(first.try_recv().is_none());
        let stale_id = first.id;
        drop(first);
        let cancelled = queue.recv().unwrap();
        assert!(!queued_request_is_active(&cancelled, &client.rpc_state));
        client.test_complete_rpc(stale_id, Ok(serde_json::json!({"scene_generation":1})));
    }

    #[test]
    fn rpc_ticket_disconnect_cancels_queued_mutations_but_keeps_explicit_retry() {
        let (client, queue) = WsClient::test_rpc_client();
        let first = client.request_rpc("sim.load_scenario", serde_json::json!({}));
        let queued = queue.recv().unwrap();
        interrupt_requests(&client.rpc_state);
        assert!(first.try_recv().unwrap().is_err());
        assert!(!queued_request_is_active(&queued, &client.rpc_state));
        let retry = client.request_rpc("sim.load_scenario", serde_json::json!({}));
        assert!(queued_request_is_active(
            &queue.recv().unwrap(),
            &client.rpc_state
        ));
        assert_ne!(retry.id, first.id);
    }

    #[test]
    fn rpc_ticket_queued_ack_and_deferred_install_reject_reconnected_socket() {
        let (client, _queue) = WsClient::test_rpc_client();
        client.suspend_world_stream();
        let queued = client.request_rpc("sim.load_scenario", serde_json::json!({}));
        client.test_complete_rpc(queued.id, Ok(serde_json::json!({"scene_generation":7})));
        interrupt_requests(&client.rpc_state);
        client.rpc_state.lock().unwrap().generation = Some(7);
        assert!(
            queued.try_recv().unwrap().is_err(),
            "a queued old-socket ACK must fail"
        );

        let retry = client.request_rpc("sim.load_scenario", serde_json::json!({}));
        client.test_complete_rpc(retry.id, Ok(serde_json::json!({"scene_generation":7})));
        let reply = retry.try_recv().unwrap().unwrap();
        let connection = retry.connection_id();
        interrupt_requests(&client.rpc_state); // Disconnect between reply poll and deferred scene clear.
        client.rpc_state.lock().unwrap().generation = Some(7);
        let mut cleared = false;
        assert!(!client.install_world_generation(
            reply["scene_generation"].as_u64().unwrap(),
            connection,
            || cleared = true
        ));
        assert!(
            !cleared,
            "old ACK must preserve existing scene on the replacement socket"
        );
        assert!(!client.world_generation_is_active(7));
    }

    #[test]
    fn world_boot_admits_only_matched_generation_in_both_ack_reset_orders() {
        for reset_first in [false, true] {
            let (mut client, _) = WsClient::test_rpc_client();
            let (tx, rx) = crossbeam_channel::unbounded();
            client.frame_rx = rx;
            let frame = || {
                Frame3d::VoxelDelta(civ_protocol_3d::VoxelDeltaFrame {
                    tick: 1,
                    deltas: vec![],
                })
            };
            client.rpc_state.lock().unwrap().generation = Some(1);
            enqueue_frame(frame(), &tx, &client.rpc_state).unwrap();
            client.suspend_world_stream();
            if reset_first {
                client.rpc_state.lock().unwrap().generation = Some(2);
                enqueue_frame(frame(), &tx, &client.rpc_state).unwrap();
            }
            assert!(client.poll().is_empty()); // No frames before scene clear/admission.
            assert!(client.install_world_generation(2, 0, || {}));
            if !reset_first {
                assert!(client.poll().is_empty()); // Older buffered scene cannot satisfy the ACK.
                client.rpc_state.lock().unwrap().generation = Some(2);
                enqueue_frame(frame(), &tx, &client.rpc_state).unwrap();
            }
            assert_eq!(client.poll().len(), 1);
            interrupt_requests(&client.rpc_state);
            client.rpc_state.lock().unwrap().generation = Some(2); // Restart reuses number.
            assert!(!client.world_generation_is_active(2));
        }
    }

    #[tokio::test]
    async fn rpc_ticket_sends_when_server_has_no_inbound_ticks() {
        use tokio::net::TcpListener;
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let url = format!("ws://{}", listener.local_addr().unwrap());
        let (client, cmd_rx) = WsClient::test_rpc_client();
        let (frame_tx, _frame_rx) = crossbeam_channel::unbounded();
        let (meta_tx, _meta_rx) = crossbeam_channel::unbounded();
        let (rtt_tx, _rtt_rx) = crossbeam_channel::unbounded();
        let (state_tx, _state_rx) = crossbeam_channel::unbounded();
        let (_send_tx, send_rx) = crossbeam_channel::unbounded();
        let (emergence_tx, _emergence_rx) = crossbeam_channel::unbounded();
        let (perf_tx, _perf_rx) = crossbeam_channel::unbounded();
        let (events_tx, _events_rx) = crossbeam_channel::unbounded();
        let (outcome_tx, _outcome_rx) = crossbeam_channel::unbounded();
        let (saves_tx, _saves_rx) = crossbeam_channel::unbounded();
        let (reset_tx, _reset_rx) = crossbeam_channel::unbounded();
        let server = async {
            let (socket, _) = listener.accept().await.unwrap();
            let mut ws = tokio_tungstenite::accept_async(socket).await.unwrap();
            ws.next().await.unwrap().unwrap(); // Initial snapshot request; deliberately send nothing.
            tokio::time::sleep(Duration::from_millis(150)).await;
            let ticket = client.request_rpc("sim.load_scenario", serde_json::json!({}));
            let request = tokio::time::timeout(Duration::from_secs(2), ws.next())
                .await
                .unwrap()
                .unwrap()
                .unwrap();
            let request: serde_json::Value =
                serde_json::from_str(request.to_text().unwrap()).unwrap();
            assert_eq!(request["id"].as_u64(), Some(ticket.id));
            ws.send(Message::Text(
                serde_json::json!({"id":ticket.id,"result":{"scene_generation":7}})
                    .to_string()
                    .into(),
            ))
            .await
            .unwrap();
            ws.close(None).await.unwrap();
            ticket
        };
        let network = connect_and_stream(
            &url,
            WsClientConfig::default(),
            &frame_tx,
            &meta_tx,
            &rtt_tx,
            &state_tx,
            &cmd_rx,
            &send_rx,
            &emergence_tx,
            &perf_tx,
            &events_tx,
            &outcome_tx,
            &saves_tx,
            &reset_tx,
            &client.rpc_state,
        );
        let (ticket, _) = tokio::join!(server, network);
        assert_eq!(ticket.try_recv().unwrap().unwrap()["scene_generation"], 7);
    }

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
}
