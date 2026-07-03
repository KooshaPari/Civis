use std::sync::Arc;
use std::time::Duration;

use civ_engine::Simulation;
use civ_server::spawn_ws_bridge;
use futures::{SinkExt, StreamExt};
use tokio::time::timeout;
use tokio_tungstenite::{connect_async, tungstenite::Message};

const SEED: u64 = 42;

async fn recv_rpc(
    socket: &mut tokio_tungstenite::WebSocketStream<
        tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>,
    >,
    id: i64,
) -> serde_json::Value {
    timeout(Duration::from_secs(5), async {
        loop {
            let frame = socket.next().await.expect("ws closed").expect("ws error");
            let Message::Text(text) = frame else {
                continue;
            };
            let v: serde_json::Value = serde_json::from_str(&text).expect("json");
            if v.get("id").and_then(|i| i.as_i64()) == Some(id) {
                return v;
            }
        }
    })
    .await
    .unwrap_or_else(|_| panic!("timeout waiting for id={id}"))
}

#[tokio::test]
async fn test_sim_reset_clears_state() {
    let sim = Arc::new(tokio::sync::Mutex::new(Simulation::with_seed(SEED)));
    let addr = spawn_ws_bridge(sim, 4).await;
    let url = format!("ws://{addr}/ws");
    let (mut ws, _) = connect_async(&url).await.expect("connect");

    let msg = r#"{"jsonrpc":"2.0","id":1,"method":"sim.reset","params":{}}"#;
    ws.send(Message::Text(msg.to_owned())).await.expect("send");

    let resp = recv_rpc(&mut ws, 1).await;
    assert_eq!(
        resp["result"]["tick"].as_u64(),
        Some(0),
        "reset should return tick=0, got: {resp}"
    );
}

#[tokio::test]
async fn test_load_scenario_ardani() {
    let sim = Arc::new(tokio::sync::Mutex::new(Simulation::with_seed(SEED)));
    let addr = spawn_ws_bridge(sim, 4).await;
    let url = format!("ws://{addr}/ws");
    let (mut ws, _) = connect_async(&url).await.expect("connect");

    let load = r#"{"jsonrpc":"2.0","id":2,"method":"sim.load_scenario","params":{"preset":"Ardani","seed":42}}"#;
    ws.send(Message::Text(load.to_owned())).await.expect("send load");
    let _load_resp = recv_rpc(&mut ws, 2).await;

    let status = r#"{"jsonrpc":"2.0","id":3,"method":"sim.status","params":{}}"#;
    ws.send(Message::Text(status.to_owned()))
        .await
        .expect("send status");
    let resp = recv_rpc(&mut ws, 3).await;

    assert!(
        resp["result"]["tick"].as_u64().is_some(),
        "status after load_scenario should return a numeric tick, got: {resp}"
    );
}

#[tokio::test]
async fn test_set_speed_accepted() {
    let sim = Arc::new(tokio::sync::Mutex::new(Simulation::with_seed(SEED)));
    let addr = spawn_ws_bridge(sim, 4).await;
    let url = format!("ws://{addr}/ws");
    let (mut ws, _) = connect_async(&url).await.expect("connect");

    let msg =
        r#"{"jsonrpc":"2.0","id":4,"method":"sim.set_speed","params":{"multiplier":2}}"#;
    ws.send(Message::Text(msg.to_owned())).await.expect("send");

    let resp = recv_rpc(&mut ws, 4).await;
    assert!(
        resp.get("error").is_none(),
        "set_speed should not return an error, got: {resp}"
    );
    assert!(
        resp.get("result").is_some(),
        "set_speed should return a result field, got: {resp}"
    );
}

#[tokio::test]
async fn test_god_action_smite() {
    let sim = Arc::new(tokio::sync::Mutex::new(Simulation::with_seed(SEED)));
    let addr = spawn_ws_bridge(sim, 4).await;
    let url = format!("ws://{addr}/ws");
    let (mut ws, _) = connect_async(&url).await.expect("connect");

    let msg = r#"{"jsonrpc":"2.0","id":5,"method":"sim.god_action","params":{"action":"smite","x":0.5,"y":0.5,"magnitude":1.0}}"#;
    ws.send(Message::Text(msg.to_owned())).await.expect("send");

    let resp = recv_rpc(&mut ws, 5).await;
    assert!(
        resp.get("error").is_none(),
        "god_action smite should not return an error, got: {resp}"
    );
    assert!(
        resp.get("result").is_some(),
        "god_action smite should return a result field, got: {resp}"
    );
}

#[tokio::test]
async fn test_outcome_ongoing_at_start() {
    let sim = Arc::new(tokio::sync::Mutex::new(Simulation::with_seed(SEED)));
    let addr = spawn_ws_bridge(sim, 4).await;
    let url = format!("ws://{addr}/ws");
    let (mut ws, _) = connect_async(&url).await.expect("connect");

    let reset = r#"{"jsonrpc":"2.0","id":6,"method":"sim.reset","params":{}}"#;
    ws.send(Message::Text(reset.to_owned()))
        .await
        .expect("send reset");
    let _reset_resp = recv_rpc(&mut ws, 6).await;

    let outcome = r#"{"jsonrpc":"2.0","id":7,"method":"sim.outcome","params":{}}"#;
    ws.send(Message::Text(outcome.to_owned()))
        .await
        .expect("send outcome");
    let resp = recv_rpc(&mut ws, 7).await;

    assert_eq!(
        resp["result"]["outcome"].as_str(),
        Some("ongoing"),
        "outcome should be 'ongoing' at simulation start, got: {resp}"
    );
}

#[tokio::test]
async fn test_perf_returns_last_tick_ms() {
    let sim = Arc::new(tokio::sync::Mutex::new(Simulation::with_seed(SEED)));
    let addr = spawn_ws_bridge(sim, 4).await;
    let url = format!("ws://{addr}/ws");
    let (mut ws, _) = connect_async(&url).await.expect("connect");

    let msg = r#"{"jsonrpc":"2.0","id":8,"method":"sim.perf","params":{}}"#;
    ws.send(Message::Text(msg.to_owned())).await.expect("send");

    let resp = recv_rpc(&mut ws, 8).await;
    assert!(
        resp.get("error").is_none(),
        "sim.perf should not return an error, got: {resp}"
    );
    assert!(
        resp["result"]["last_tick_ms"].as_f64().is_some(),
        "sim.perf result should contain a numeric last_tick_ms, got: {resp}"
    );
}