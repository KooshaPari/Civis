use std::{net::SocketAddr, sync::Arc, time::Duration};

use civ_engine::Simulation;
use civ_server::{spawn_ws_bridge_with_config, TickBroadcastFormat, WsBridgeConfig};
use futures::{SinkExt, StreamExt};
use serde_json::{json, Value};
use tokio::{net::TcpStream, time::timeout};
use tokio_tungstenite::{connect_async, tungstenite::Message, MaybeTlsStream, WebSocketStream};

async fn rpc(
    socket: &mut WebSocketStream<MaybeTlsStream<TcpStream>>,
    id: u64,
    method: &str,
    params: Value,
) -> Value {
    socket
        .send(Message::Text(
            json!({"jsonrpc":"2.0", "id":id, "method":method, "params":params})
                .to_string()
                .into(),
        ))
        .await
        .unwrap();
    timeout(Duration::from_secs(10), async {
        while let Some(frame) = socket.next().await {
            if let Message::Text(text) = frame.unwrap() {
                let value: Value = serde_json::from_str(&text).unwrap();
                if value.get("id") == Some(&json!(id)) {
                    return value;
                }
            }
        }
        panic!("connection closed before response {id}");
    })
    .await
    .expect("RPC timeout")
}

#[tokio::test]
async fn corrupt_slot_load_rejects_and_keeps_live_world() {
    let saves = tempfile::tempdir().unwrap();
    let replays = tempfile::tempdir().unwrap();
    let sim = Arc::new(tokio::sync::Mutex::new(Simulation::with_seed(42)));
    let addr = spawn_ws_bridge_with_config(
        sim.clone(),
        WsBridgeConfig {
            addr: SocketAddr::from(([127, 0, 0, 1], 0)),
            max_clients: 4,
            require_role: false,
            tick_broadcast_format: TickBroadcastFormat::Both,
            saves_dir: saves.path().to_path_buf(),
            replays_dir: replays.path().to_path_buf(),
        },
    )
    .await;
    let (mut socket, _) = connect_async(format!("ws://{addr}/ws")).await.unwrap();
    assert_eq!(
        rpc(&mut socket, 1, "sim.set_speed", json!({"multiplier":0}))
            .await
            .pointer("/result/accepted"),
        Some(&json!(true))
    );
    let saved = rpc(&mut socket, 2, "save.slot", json!({"slot_name":"slot-1"})).await;
    assert_eq!(saved.pointer("/result/saved"), Some(&json!(true)));
    let saved_tick = saved
        .pointer("/result/tick")
        .and_then(Value::as_u64)
        .unwrap();
    let advanced = rpc(&mut socket, 6, "sim.command", json!({"action":"tick"})).await;
    assert_eq!(advanced.pointer("/result/accepted"), Some(&json!(true)));
    let before = rpc(&mut socket, 3, "sim.snapshot", json!({})).await;
    assert_eq!(before.pointer("/result/tick"), Some(&json!(saved_tick + 1)));
    let path = saves.path().join("slot-1.civsave.zst");
    assert!(path.is_file());
    std::fs::write(&path, b"not a valid save archive").unwrap();
    assert_eq!(before.pointer("/result/speed_multiplier"), Some(&json!(0)));
    assert!(before
        .pointer("/result/tick")
        .and_then(Value::as_u64)
        .is_some());
    assert!(before
        .pointer("/result/population")
        .and_then(Value::as_u64)
        .is_some());
    assert!(before
        .pointer("/result/civ_pins")
        .and_then(Value::as_array)
        .is_some());
    let rejected = rpc(&mut socket, 4, "save.load", json!({"slot_name":"slot-1"})).await;
    assert!(
        rejected.get("error").is_some(),
        "corrupt slot must fail: {rejected}"
    );
    assert_eq!(rejected.pointer("/error/code"), Some(&json!(-32603)));
    assert!(rejected.get("result").is_none());
    let after = rpc(&mut socket, 5, "sim.snapshot", json!({})).await;
    for field in ["tick", "population", "civ_pins", "speed_multiplier"] {
        assert_eq!(
            before["result"][field], after["result"][field],
            "{field} changed after rejected load"
        );
    }
}
