//! Smoke coverage for the public WebSocket bridge after the retired build-frame API.

use std::sync::Arc;
use std::time::Duration;

use civ_engine::Simulation;
use civ_server::spawn_ws_bridge;
use futures::{SinkExt, StreamExt};
use tokio::time::timeout;
use tokio_tungstenite::{connect_async, tungstenite::Message};

#[tokio::test]
async fn fr_build_ws_bridge_serves_live_legends_status() {
    let sim = Arc::new(tokio::sync::Mutex::new(Simulation::with_seed(17)));
    let addr = spawn_ws_bridge(sim, 4).await;
    let (mut socket, _) = connect_async(format!("ws://{addr}/ws"))
        .await
        .expect("connect to websocket bridge");

    socket
        .send(Message::Text(
            r#"{"jsonrpc":"2.0","id":71,"method":"sim.legends","params":{"query":"status"}}"#
                .into(),
        ))
        .await
        .expect("send sim.legends request");

    let response = timeout(Duration::from_secs(2), async {
        while let Some(frame) = socket.next().await {
            let Message::Text(text) = frame.expect("websocket frame") else {
                continue;
            };
            let value: serde_json::Value = serde_json::from_str(&text).expect("JSON-RPC frame");
            if value.get("id") == Some(&serde_json::json!(71)) {
                return value;
            }
        }
        panic!("bridge closed before sim.legends response");
    })
    .await
    .expect("sim.legends response timeout");

    assert_eq!(
        response
            .pointer("/result/query_api_version")
            .and_then(|v| v.as_u64()),
        Some(1)
    );
    assert!(response
        .pointer("/result/tick")
        .and_then(|v| v.as_u64())
        .is_some());
}
