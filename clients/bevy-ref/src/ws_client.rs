use std::{
    sync::atomic::{AtomicU32, Ordering},
    thread,
    time::Duration,
};

use civ_protocol_3d::Frame3d;

use crate::{
    parse_jsonrpc_snapshot_meta, parse_ws_payload, ws_prefer_binary_from_env, WsSpectatorMeta,
};
use crossbeam_channel::{Receiver, Sender};
use futures_util::{SinkExt, StreamExt};
use tokio::runtime::Builder;
use tokio_tungstenite::tungstenite::Message;

/// WebSocket session state exposed to the Bevy main thread.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum WsConnectionState {
    /// Active stream to `civ-server`.
    Connected,
    /// Backing off after a disconnect; will retry.
    Reconnecting,
    /// No successful connection yet (initial boot).
    #[default]
    Disconnected,
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

/// WebSocket client that bridges the tokio network task to Bevy systems.
pub struct WsClient {
    frame_rx: Receiver<Frame3d>,
    meta_rx: Receiver<WsSpectatorMeta>,
    rtt_rx: Receiver<f32>,
    state_rx: Receiver<WsConnectionState>,
    latest_state: AtomicU32,
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
        thread::spawn(move || run_client(url, config, frame_tx, meta_tx, rtt_tx, state_tx));
        Self {
            frame_rx,
            meta_rx,
            rtt_rx,
            state_rx,
            latest_state: AtomicU32::new(state_to_atomic(WsConnectionState::Disconnected)),
        }
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
}

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
            match connect_and_stream(&url, config, &frame_tx, &meta_tx, &rtt_tx, &state_tx).await
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

async fn connect_and_stream(
    url: &str,
    config: WsClientConfig,
    frame_tx: &Sender<Frame3d>,
    meta_tx: &Sender<WsSpectatorMeta>,
    rtt_tx: &Sender<f32>,
    state_tx: &Sender<WsConnectionState>,
) -> Result<(), String> {
    let (ws, _) = tokio_tungstenite::connect_async(url)
        .await
        .map_err(|err| err.to_string())?;
    publish_state(state_tx, WsConnectionState::Connected);

    let (mut write, mut read) = ws.split();

    let mut snapshot_ping = None;
    request_snapshot(&mut write, &mut snapshot_ping).await?;

    let mut last_snapshot = std::time::Instant::now();

    while let Some(msg) = read.next().await {
        if last_snapshot.elapsed() >= Duration::from_secs(SNAPSHOT_POLL_SECS) {
            request_snapshot(&mut write, &mut snapshot_ping).await?;
            last_snapshot = std::time::Instant::now();
        }

        let msg = msg.map_err(|err| err.to_string())?;
        match msg {
            Message::Text(text) => {
                if let Some(meta) = parse_jsonrpc_snapshot_meta(&text) {
                    record_snapshot_rtt(&mut snapshot_ping, rtt_tx);
                    if meta_tx.send(meta).is_err() {
                        return Err("bevy meta receiver dropped".into());
                    }
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
