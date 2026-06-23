use std::{net::SocketAddr, sync::Arc};

use civ_engine::Simulation;
use civ_server::{run_ws_bridge, TickBroadcastFormat, WsBridgeConfig};
use tokio::sync::Mutex;

#[tokio::main]
async fn main() {
    let port = std::env::var("CIV_SERVER_PORT")
        .ok()
        .and_then(|value| value.parse().ok())
        .unwrap_or(3000);
    let addr: SocketAddr = SocketAddr::from(([127, 0, 0, 1], port));
    let max_clients = std::env::var("CIVIS_WS_MAX_CLIENTS")
        .ok()
        .and_then(|value| value.parse().ok())
        .unwrap_or(16);

    run_ws_bridge(
        WsBridgeConfig {
            addr,
            max_clients,
            require_role: false,
            tick_broadcast_format: TickBroadcastFormat::from_env(),
        },
        Arc::new(Mutex::new(Simulation::default())),
    )
    .await;
}
