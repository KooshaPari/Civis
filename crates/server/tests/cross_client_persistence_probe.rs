use std::{net::SocketAddr, sync::Arc, time::Duration};

use civ_engine::Simulation;
use civ_server::{spawn_ws_bridge_with_config, TickBroadcastFormat, WsBridgeConfig};
use futures::{SinkExt, StreamExt};
use serde_json::{json, Value};
use tokio::{net::TcpStream, time::timeout};
use tokio_tungstenite::{connect_async, tungstenite::Message, MaybeTlsStream, WebSocketStream};

type Socket = WebSocketStream<MaybeTlsStream<TcpStream>>;

async fn rpc(socket: &mut Socket, id: u64, method: &str, params: Value) -> Value {
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

fn matching_pin(pin: &Value) -> bool {
    let x = pin.get("x").and_then(Value::as_f64).unwrap();
    let y = pin.get("y").and_then(Value::as_f64).unwrap();
    (x - 0.42).abs() < 0.0001 && (y - 0.58).abs() < 0.0001
}

fn projection(response: &Value) -> Value {
    let state = response.get("result").expect("full snapshot result");
    assert_eq!(state.get("speed_multiplier"), Some(&json!(0)));
    assert!(state.get("tick").and_then(Value::as_u64).is_some());
    assert!(state.get("population").and_then(Value::as_u64).is_some());
    assert!(state.get("civ_pins").and_then(Value::as_array).is_some());
    json!({"tick":state["tick"], "population":state["population"], "civ_pins":state["civ_pins"]})
}

#[tokio::test]
async fn second_client_observes_spawn_after_reconnect() {
    let saves = tempfile::tempdir().unwrap();
    let replays = tempfile::tempdir().unwrap();
    let sim = Arc::new(tokio::sync::Mutex::new(Simulation::with_seed(42)));
    let addr = spawn_ws_bridge_with_config(
        sim,
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
    let url = format!("ws://{addr}/ws");
    let (mut actor, _) = connect_async(&url).await.unwrap();
    let (mut observer, _) = connect_async(&url).await.unwrap();
    assert_eq!(
        rpc(&mut actor, 1, "sim.set_speed", json!({"multiplier":0}))
            .await
            .pointer("/result/accepted"),
        Some(&json!(true))
    );
    let before = projection(&rpc(&mut observer, 2, "sim.snapshot", json!({})).await);
    let pins = before["civ_pins"].as_array().unwrap();
    assert!(
        !pins.iter().any(matching_pin),
        "fixture coordinate already occupied"
    );
    let spawn = rpc(
        &mut actor,
        3,
        "sim.spawn_civilian",
        json!({"x":0.42,"y":0.58,"faction":2}),
    )
    .await;
    assert_eq!(spawn.pointer("/result/ok"), Some(&json!(true)));
    assert!(spawn
        .pointer("/result/entity_id")
        .and_then(Value::as_u64)
        .is_some());
    let after = projection(&rpc(&mut observer, 4, "sim.snapshot", json!({})).await);
    let new_pins = after["civ_pins"].as_array().unwrap();
    assert_eq!(new_pins.len(), pins.len() + 1);
    assert_eq!(new_pins.iter().filter(|pin| matching_pin(pin)).count(), 1);
    observer.close(None).await.unwrap();
    let (mut reconnected, _) = connect_async(&url).await.unwrap();
    let recovered = projection(&rpc(&mut reconnected, 5, "sim.snapshot", json!({})).await);
    assert_eq!(recovered, after);
    let saved = rpc(&mut actor, 6, "save.slot", json!({"slot_name":"slot-1"})).await;
    assert_eq!(saved.pointer("/result/saved"), Some(&json!(true)));
    let saved_tick = saved
        .pointer("/result/tick")
        .and_then(Value::as_u64)
        .unwrap();
    let tick = rpc(&mut actor, 7, "sim.command", json!({"action":"tick"})).await;
    assert_eq!(tick.pointer("/result/accepted"), Some(&json!(true)));
    let diverged = projection(&rpc(&mut reconnected, 8, "sim.snapshot", json!({})).await);
    assert_eq!(diverged["tick"], json!(saved_tick + 1));
    let loaded = rpc(&mut actor, 9, "save.load", json!({"slot_name":"slot-1"})).await;
    assert_eq!(loaded.pointer("/result/loaded"), Some(&json!(true)));
    assert_eq!(loaded.pointer("/result/tick"), Some(&json!(saved_tick)));
    assert!(loaded
        .pointer("/result/scene_generation")
        .and_then(Value::as_u64)
        .is_some());
    let restored = projection(&rpc(&mut reconnected, 10, "sim.snapshot", json!({})).await);
    assert_eq!(restored["tick"], after["tick"]);
    let restored_pins = restored["civ_pins"].as_array().unwrap();
    assert_eq!(
        restored_pins.iter().filter(|pin| matching_pin(pin)).count(),
        1,
        "spawned pin must survive reload"
    );
    assert_eq!(restored, after, "saved projection must survive reload");
}
