use std::{
    thread,
    time::{Duration, Instant},
};

use civ_protocol_3d::Frame3d;

use crate::{
    parse_jsonrpc_snapshot_meta, parse_ws_payload, ws_prefer_binary_from_env, WsSpectatorMeta,
};
use crossbeam_channel::{Receiver, Sender};
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
        thread::spawn(move || run_client(url, config, frame_tx, meta_tx));
        Self { frame_rx, meta_rx }
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
}

const SNAPSHOT_RPC: &str = r#"{"jsonrpc":"2.0","id":9001,"method":"sim.snapshot","params":{}}"#;
const SNAPSHOT_POLL_SECS: u64 = 2;

fn run_client(
    url: String,
    config: WsClientConfig,
    frame_tx: Sender<Frame3d>,
    meta_tx: Sender<WsSpectatorMeta>,
) {
    let runtime = Builder::new_multi_thread()
        .enable_all()
        .build()
        .expect("tokio runtime");
    runtime.block_on(async move {
        loop {
            if let Err(err) = connect_and_stream(&url, config, &frame_tx, &meta_tx).await {
                eprintln!("bevy ws client disconnected: {err}");
                thread::sleep(Duration::from_secs(1));
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
) -> Result<(), String> {
    write
        .send(Message::Text(SNAPSHOT_RPC.into()))
        .await
        .map_err(|err| err.to_string())
}

async fn connect_and_stream(
    url: &str,
    config: WsClientConfig,
    frame_tx: &Sender<Frame3d>,
    meta_tx: &Sender<WsSpectatorMeta>,
) -> Result<(), String> {
    let (ws, _) = tokio_tungstenite::connect_async(url)
        .await
        .map_err(|err| err.to_string())?;
    let (mut write, mut read) = ws.split();

    request_snapshot(&mut write).await?;

    let mut last_snapshot = Instant::now();

    while let Some(msg) = read.next().await {
        if last_snapshot.elapsed() >= Duration::from_secs(SNAPSHOT_POLL_SECS) {
            request_snapshot(&mut write).await?;
            last_snapshot = Instant::now();
        }

        let msg = msg.map_err(|err| err.to_string())?;
        match msg {
            Message::Text(text) => {
                if let Some(meta) = parse_jsonrpc_snapshot_meta(&text) {
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
