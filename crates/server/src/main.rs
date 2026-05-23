use std::{net::SocketAddr, sync::Arc};

use civ_engine::Simulation;
use civ_server::{run_ws_bridge, WsBridgeConfig};
use tokio::sync::Mutex;

#[tokio::main]
async fn main() {
    let addr: SocketAddr = std::env::var("CIVIS_WS_ADDR")
        .ok()
        .and_then(|value| value.parse().ok())
        .unwrap_or_else(|| SocketAddr::from(([127, 0, 0, 1], 3000)));
    let max_clients = std::env::var("CIVIS_WS_MAX_CLIENTS")
        .ok()
        .and_then(|value| value.parse().ok())
        .unwrap_or(16);

    run_ws_bridge(
        WsBridgeConfig { addr, max_clients },
        Arc::new(Mutex::new(Simulation::default())),
    )
    .await;
}
