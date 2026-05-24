use std::{thread, time::Duration};

use civ_protocol_3d::Frame3d;

use crate::{parse_ws_payload, ws_prefer_binary_from_env};
use crossbeam_channel::{Receiver, Sender};
use futures_util::StreamExt;
use tokio::runtime::Builder;

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
    rx: Receiver<Frame3d>,
}

impl WsClient {
    /// Spawn a reconnecting WebSocket client on a dedicated tokio runtime.
    pub fn spawn(url: String) -> Self {
        Self::spawn_with_config(url, WsClientConfig::default())
    }

    /// Spawn with explicit attach preferences (binary-first tick handling).
    pub fn spawn_with_config(url: String, config: WsClientConfig) -> Self {
        let (tx, rx) = crossbeam_channel::unbounded();
        thread::spawn(move || run_client(url, config, tx));
        Self { rx }
    }

    /// Drain all currently available frames without blocking the main thread.
    #[must_use]
    pub fn poll(&self) -> Vec<Frame3d> {
        let mut frames = Vec::new();
        while let Ok(frame) = self.rx.try_recv() {
            frames.push(frame);
        }
        frames
    }
}

fn run_client(url: String, config: WsClientConfig, tx: Sender<Frame3d>) {
    let runtime = Builder::new_multi_thread()
        .enable_all()
        .build()
        .expect("tokio runtime");
    runtime.block_on(async move {
        loop {
            if let Err(err) = connect_and_stream(&url, config, &tx).await {
                eprintln!("bevy ws client disconnected: {err}");
                thread::sleep(Duration::from_secs(1));
            }
        }
    });
}

async fn connect_and_stream(
    url: &str,
    config: WsClientConfig,
    tx: &Sender<Frame3d>,
) -> Result<(), String> {
    let (ws, _) = tokio_tungstenite::connect_async(url)
        .await
        .map_err(|err| err.to_string())?;
    let (_, mut read) = ws.split();

    while let Some(msg) = read.next().await {
        let msg = msg.map_err(|err| err.to_string())?;
        let frame = match &msg {
            tokio_tungstenite::tungstenite::Message::Text(text) if config.prefer_binary => {
                continue;
            }
            tokio_tungstenite::tungstenite::Message::Text(text) => {
                parse_ws_payload(text.as_bytes())?
            }
            tokio_tungstenite::tungstenite::Message::Binary(bytes) => parse_ws_payload(bytes)?,
            _ => continue,
        };
        if tx.send(frame).is_err() {
            return Err("bevy receiver dropped".into());
        }
    }

    Err("websocket closed".into())
}
