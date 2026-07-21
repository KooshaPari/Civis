use std::{
    sync::{
        atomic::{AtomicU32, Ordering},
        Arc,
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
    save_list_rx: crossbeam_channel::Receiver<Vec<SaveListEntry>>,
    scene_reset_rx: Receiver<SceneReset>,
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
            emergence_rx,
            outcome_rx,
            save_list_rx,
            scene_reset_rx,
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
        let (outcome_tx, outcome_rx) = crossbeam_channel::unbounded::<OutcomeHudData>();
        let (save_list_tx, save_list_rx) = crossbeam_channel::unbounded::<Vec<SaveListEntry>>();
        let (scene_reset_tx, scene_reset_rx) = crossbeam_channel::unbounded::<SceneReset>();

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
                outcome_tx,
                save_list_tx,
                scene_reset_tx,
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
            outcome_rx,
            save_list_rx,
            scene_reset_rx,
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

    #[must_use]
    pub fn poll_outcome(&self) -> Option<OutcomeHudData> {
        let mut latest = None;
        while let Ok(o) = self.outcome_rx.try_recv() {
            latest = Some(o);
        }
        latest
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
        let mut frames = Vec::new();
        while let Ok(frame) = self.frame_rx.try_recv() {
            frames.push(frame);
        }
        frames
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
            outcome_rx: self.outcome_rx.clone(),
            save_list_rx: self.save_list_rx.clone(),
            scene_reset_rx: self.scene_reset_rx.clone(),
        }
    }
}

const OUTCOME_RPC: &str = r#"{"jsonrpc":"2.0","id":9003,"method":"sim.outcome","params":{}}"#;
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
    save_list_tx: Sender<Vec<SaveListEntry>>,
    scene_reset_tx: Sender<SceneReset>,
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
                &outcome_tx,
                &save_list_tx,
                &scene_reset_tx,
            )
            .await
            {
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
    frame_tx: &Sender<Frame3d>,
    meta_tx: &Sender<WsSpectatorMeta>,
    rtt_tx: &Sender<f32>,
    state_tx: &Sender<WsConnectionState>,
    cmd_rx: &Receiver<String>,
    send_rx: &crossbeam_channel::Receiver<String>,
    emergence_tx: &Sender<EmergenceHudData>,
    outcome_tx: &Sender<OutcomeHudData>,
    save_list_tx: &Sender<Vec<SaveListEntry>>,
    scene_reset_tx: &Sender<SceneReset>,
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

        if last_outcome.elapsed() >= Duration::from_secs(OUTCOME_POLL_SECS) {
            write
                .send(Message::Text(OUTCOME_RPC.into()))
                .await
                .map_err(|e| e.to_string())?;
            last_outcome = std::time::Instant::now();
        }
        if last_snapshot.elapsed() >= Duration::from_secs(SNAPSHOT_POLL_SECS) {
            request_snapshot(&mut write, &mut snapshot_ping).await?;
            last_snapshot = std::time::Instant::now();
        }

        let msg = match read.next().await {
            Some(msg) => msg.map_err(|err| err.to_string())?,
            None => break,
        };
        match msg {
            Message::Text(text) => {
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
                if let Some(em) = parse_emergence_response(&text) {
                    let _ = emergence_tx.send(em);
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
    fn parse_outcome_response_ignores_other_rpc_ids() {
        let text = r#"{"jsonrpc":"2.0","id":3,"result":{"outcome":"victory"}}"#;
        assert!(parse_outcome_response(text).is_none());
    }

    #[test]
    fn parse_scene_reset_notification_reads_tick() {
        let text = r#"{"jsonrpc":"2.0","method":"scene.reset","params":{"tick":42}}"#;
        assert_eq!(parse_scene_reset_notification(text), Some(SceneReset { tick: 42 }));
    }

    #[test]
    fn parse_scene_reset_notification_ignores_rpc_responses() {
        let text = r#"{"jsonrpc":"2.0","id":7,"result":{"tick":42}}"#;
        assert_eq!(parse_scene_reset_notification(text), None);
    }
}
