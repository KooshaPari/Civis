use std::{thread, time::Duration};

use civ_protocol_3d::Frame3d;
use crossbeam_channel::{Receiver, Sender};
use futures_util::StreamExt;
use tokio::runtime::Builder;

/// WebSocket client that bridges the tokio network task to Bevy systems.
pub struct WsClient {
    rx: Receiver<Frame3d>,
}

impl WsClient {
    /// Spawn a reconnecting WebSocket client on a dedicated tokio runtime.
    pub fn spawn(url: String) -> Self {
        let (tx, rx) = crossbeam_channel::unbounded();
        thread::spawn(move || run_client(url, tx));
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

fn run_client(url: String, tx: Sender<Frame3d>) {
    let runtime = Builder::new_multi_thread()
        .enable_all()
        .build()
        .expect("tokio runtime");
    runtime.block_on(async move {
        loop {
            if let Err(err) = connect_and_stream(&url, &tx).await {
                eprintln!("bevy ws client disconnected: {err}");
                thread::sleep(Duration::from_secs(1));
            }
        }
    });
}

async fn connect_and_stream(url: &str, tx: &Sender<Frame3d>) -> Result<(), String> {
    let (ws, _) = tokio_tungstenite::connect_async(url)
        .await
        .map_err(|err| err.to_string())?;
    let (_, mut read) = ws.split();

    while let Some(msg) = read.next().await {
        let msg = msg.map_err(|err| err.to_string())?;
        if !msg.is_text() {
            continue;
        }
        let frame: Frame3d = serde_json::from_str(msg.to_text().map_err(|err| err.to_string())?)
            .map_err(|err| err.to_string())?;
        if tx.send(frame).is_err() {
            return Err("bevy receiver dropped".into());
        }
    }

    Err("websocket closed".into())
}
