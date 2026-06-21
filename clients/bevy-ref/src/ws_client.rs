use std::{
    sync::atomic::{AtomicU32, Ordering},
    thread,
    time::Duration,
};

use civ_protocol_3d::Frame3d;

use crate::{
    parse_jsonrpc_snapshot_meta, parse_ws_payload, ws_prefer_binary_from_env, EmergenceHudData, OutcomeHudData, OutcomeHudData,
    parse_jsonrpc_snapshot_meta, parse_ws_payload, ws_prefer_binary_from_env, EmergenceHudData, OutcomeHudData,
    parse_jsonrpc_snapshot_meta, parse_ws_payload, ws_prefer_binary_from_env, EmergenceHudData,
    WsConnectionState, WsSpectatorMeta,
};
use crossbeam_channel::{Receiver, Sender};
use serde_json;
use futures_util::{SinkExt, StreamExt};
use tokio::runtime::Builder;
use tokio_tungstenite::tungstenite::Message;

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
    /// Inbound parsed EmergenceHudData from id=2 sim.emergence responses.
    emergence_rx: crossbeam_channel::Receiver<EmergenceHudData>,
    outcome_rx: crossbeam_channel::Receiver<OutcomeHudData>,
    outcome_rx: crossbeam_channel::Receiver<OutcomeHudData>,
}

impl WsClient {
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
        thread::spawn(move || {
            run_client(url, config, frame_tx, meta_tx, rtt_tx, state_tx, cmd_rx);
        });
        let (send_tx, send_rx) = crossbeam_channel::unbounded::<String>();
        let (emergence_tx, emergence_rx) = crossbeam_channel::unbounded::<EmergenceHudData>();
        let (outcome_tx, outcome_rx) = crossbeam_channel::unbounded::<OutcomeHudData>();
        let (outcome_tx, outcome_rx) = crossbeam_channel::unbounded::<OutcomeHudData>();
        thread::spawn(move || run_client(url, config, frame_tx, meta_tx, rtt_tx, state_tx, send_rx, emergence_tx, outcome_tx));
        let (send_tx, send_rx) = crossbeam_channel::unbounded::<String>();
        let (emergence_tx, emergence_rx) = crossbeam_channel::unbounded::<EmergenceHudData>();
        let (outcome_tx, outcome_rx) = crossbeam_channel::unbounded::<OutcomeHudData>();
        thread::spawn(move || run_client(url, config, frame_tx, meta_tx, rtt_tx, state_tx, send_rx, emergence_tx, outcome_tx));
        Self {
            frame_rx,
            meta_rx,
            rtt_rx,
            state_rx,
            latest_state: AtomicU32::new(state_to_atomic(WsConnectionState::Disconnected)),
            send_tx,
            emergence_rx,
        }
    }

            send_tx,
            emergence_rx,
            outcome_rx,
            outcome_rx,
    /// Enqueue an outbound JSON-RPC text frame (fire-and-forget; drops silently if disconnected).
    pub fn send_rpc(&self, json: String) {
        let _ = self.send_tx.send(json);
            send_tx,
            emergence_rx,
            outcome_rx,
        }
    }

    /// Enqueue an outbound JSON-RPC text frame (fire-and-forget; drops silently if disconnected).
    pub fn send_rpc(&self, json: String) {
        let _ = self.send_tx.send(json);
    }

    /// Clone the outbound RPC sender so other Bevy resources can enqueue frames
    /// without holding a reference to the full `WsClient.
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
        out
        }
        out
    }

    #[must_use]
    pub fn poll_outcome(&self) -> Option<OutcomeHudData> {
        let mut latest = None;
        while let Ok(o) = self.outcome_rx.try_recv() { latest = Some(o); }
        latest
    }

    /// Drain all currently available frames without blocking the main thread.
    #[must_use]
    pub fn poll(&self) -> Vec<Frame3d> {
        let mut frames = Vec::new();
        while let Ok(frame) = self.frame_rx.try_recv() {
            frames.push(frame);
        }
        frames
    }

    /// Drain `sim.snapshot` JSON-RPC metadata (day/night, tick).
    #[must_use]
    pub fn poll_outcome(&self) -> Option<OutcomeHudData> {
        let mut latest = None;
        while let Ok(o) = self.outcome_rx.try_recv() { latest = Some(o); }
        latest
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

const OUTCOME_RPC: &str = "{"jsonrpc":"2.0","id":9003,"method":"sim.outcome","params":{}}";
const OUTCOME_POLL_SECS: u64 = 30;
const OUTCOME_RPC: &str = "{"jsonrpc":"2.0","id":9003,"method":"sim.outcome","params":{}}";
const OUTCOME_POLL_SECS: u64 = 30;
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
    frame_tx: Sender<Frame3d>,
    meta_tx: Sender<WsSpectatorMeta>,
    rtt_tx: Sender<f32>,
    state_tx: Sender<WsConnectionState>,
    cmd_rx: Receiver<String>,
    send_rx: crossbeam_channel::Receiver<String>,
    emergence_tx: Sender<EmergenceHudData>,
    outcome_tx: Sender<OutcomeHudData>,
    send_rx: crossbeam_channel::Receiver<String>,
    emergence_tx: Sender<EmergenceHudData>,
    outcome_tx: Sender<OutcomeHudData>,
) {
    let runtime = Builder::new_multi_thread()
        .enable_all()
        .build()
        .expect("tokio runtime");
    runtime.block_on(async move {
        let mut backoff = ReconnectBackoff::new();
        publish_state(&state_tx, WsConnectionState::Disconnected);
        loop {
            publish_state(&state_tx, WsConnectionState::Reconnecting);
            match connect_and_stream(&url, config, &frame_tx, &meta_tx, &rtt_tx, &state_tx, &cmd_rx).await {
                Ok(()) => { backoff.reset(); }
            match connect_and_stream(&url, config, &frame_tx, &meta_tx, &rtt_tx, &state_tx, &send_rx, &emergence_tx, &outcome_tx).await {
            match connect_and_stream(&url, config, &frame_tx, &meta_tx, &rtt_tx, &state_tx, &send_rx, &emergence_tx, &outcome_tx).await {
                Ok(()) => {
                    backoff.reset();
                }
                Err(err) => {
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
        entropy_norm: result.get("entropy_norm").and_then(|v| v.as_f64()).unwrap_or(0.0) as f32,
        power_law_alpha: result.get("power_law_alpha").and_then(|v| v.as_f64()).unwrap_or(0.0) as f32,
        novelty_rate: result.get("novelty_rate").and_then(|v| v.as_f64()).unwrap_or(0.0) as f32,
        mi_material_faction_norm: result.get("mi_material_faction_norm").and_then(|v| v.as_f64()).map(|f| f as f32),
        structure_count: result.get("structure_count").and_then(|v| v.as_u64()).map(|n| n as u32),
        branching_regime: result.get("branching_regime").and_then(|v| v.as_str()).unwrap_or("SUBCRITICAL").to_owned(),
    })
}

fn parse_outcome_response(text: &str) -> Option<OutcomeHudData> {
    let v: serde_json::Value = serde_json::from_str(text).ok()?;
    if v.get("id").and_then(|i| i.as_i64()) != Some(9003) { return None; }
    let result = v.get("result")?;
    Some(OutcomeHudData {
        tag: result.get("outcome").and_then(|v| v.as_str()).unwrap_or("ongoing").to_owned(),
        reason: result.get("reason").and_then(|v| v.as_str()).unwrap_or("").to_owned(),
        tick: result.get("tick").and_then(|v| v.as_u64()).unwrap_or(0),
    })
}

fn parse_outcome_response(text: &str) -> Option<OutcomeHudData> {
    let v: serde_json::Value = serde_json::from_str(text).ok()?;
    if v.get("id").and_then(|i| i.as_i64()) != Some(9003) { return None; }
    let result = v.get("result")?;
    Some(OutcomeHudData {
        tag: result.get("outcome").and_then(|v| v.as_str()).unwrap_or("ongoing").to_owned(),
        reason: result.get("reason").and_then(|v| v.as_str()).unwrap_or("").to_owned(),
        tick: result.get("tick").and_then(|v| v.as_u64()).unwrap_or(0),
    })
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
    emergence_tx: &Sender<EmergenceHudData>,
    outcome_tx: &Sender<OutcomeHudData>,
    send_rx: &crossbeam_channel::Receiver<String>,
    emergence_tx: &Sender<EmergenceHudData>,
    outcome_tx: &Sender<OutcomeHudData>,
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
    let mut last_outcome = std::time::Instant::now();

    loop {
        // Flush outbound commands (speed/pause RPCs) before blocking on next inbound frame.
        while let Ok(cmd) = cmd_rx.try_recv() {
            write.send(Message::Text(cmd.into())).await.map_err(|e| e.to_string())?;
    while let Some(msg) = read.next().await {
        // Drain any outbound RPC frames queued by Bevy systems.
        while let Ok(json) = send_rx.try_recv() {
            write
                .send(Message::Text(json.into()))
                .await
                .map_err(|e| e.to_string())?;
        }

        if last_outcome.elapsed() >= Duration::from_secs(OUTCOME_POLL_SECS) {
            write.send(Message::Text(OUTCOME_RPC.into())).await.map_err(|e| e.to_string())?;
            last_outcome = std::time::Instant::now();
        }
        if last_outcome.elapsed() >= Duration::from_secs(OUTCOME_POLL_SECS) {
            write.send(Message::Text(OUTCOME_RPC.into())).await.map_err(|e| e.to_string())?;
            last_outcome = std::time::Instant::now();
        }
        if last_snapshot.elapsed() >= Duration::from_secs(SNAPSHOT_POLL_SECS) {
            request_snapshot(&mut write, &mut snapshot_ping).await?;
            last_snapshot = std::time::Instant::now();
        }

        let msg = msg.map_err(|err| err.to_string())?;
        match msg {
            Message::Text(text) => {
                if let Some(meta) = parse_jsonrpc_snapshot_meta(&text) {
                    record_snapshot_rtt(&mut snapshot_ping, rtt_tx);
                    if meta_tx.send(meta).is_err() { return Err("bevy meta receiver dropped".into()); }
                    if meta_tx.send(meta).is_err() {
                        return Err("bevy meta receiver dropped".into());
                    }
                    continue;
                }
                if let Some(em) = parse_emergence_response(&text) {
                    let _ = emergence_tx.send(em);
                    continue;
                }
                if let Some(oc) = parse_outcome_response(&text) {
                    let _ = outcome_tx.send(oc);
                    continue;
                }
                if let Some(oc) = parse_outcome_response(&text) {
                    let _ = outcome_tx.send(oc);
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
            _ => {}
        }
    }

    Err("websocket closed".into())
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
    fn reconnect_backoff_resets_after_success() {
        let mut backoff = ReconnectBackoff::new();
        let _ = backoff.next_delay();
        let _ = backoff.next_delay();
        backoff.reset();
        assert_eq!(backoff.next_delay(), Duration::from_secs(1));
    }
}
