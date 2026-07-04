use std::sync::Arc;
use std::time::Duration;

use civ_engine::{decode_civreplay, encode_civreplay, Simulation, MAGIC};
use civ_protocol_3d::{decode_frame3d_binary, Frame3d, FRAME3D_BINARY_MAGIC};
use civ_server::{
    error_code, spawn_ws_bridge, spawn_ws_bridge_with_config, TickBroadcastFormat, WsBridgeConfig,
};
use futures::{SinkExt, StreamExt};
use http::HeaderValue;
use std::net::SocketAddr;
use tokio::time::timeout;
use tokio_tungstenite::{
    connect_async,
    tungstenite::{client::IntoClientRequest, Message},
};

#[tokio::test]
async fn replay_export_returns_civreplay_octet_stream() {
    let sim = Arc::new(tokio::sync::Mutex::new(Simulation::with_seed(17)));
    let addr = spawn_ws_bridge(sim, 4).await;
    let url = format!("http://{addr}/replay/export");

    let response = reqwest::get(&url).await.expect("replay export request");
    assert!(response.status().is_success());
    assert_eq!(
        response
            .headers()
            .get(reqwest::header::CONTENT_TYPE)
            .and_then(|v| v.to_str().ok()),
        Some("application/octet-stream")
    );
    assert_eq!(
        response
            .headers()
            .get(reqwest::header::CONTENT_DISPOSITION)
            .and_then(|v| v.to_str().ok()),
        Some("attachment; filename=\"replay.civreplay\"")
    );

    let bytes = response.bytes().await.expect("replay body");
    assert!(bytes.len() >= MAGIC.len() + 32);
    assert_eq!(&bytes[..MAGIC.len()], MAGIC.as_slice());

    let log = decode_civreplay(&bytes).expect("decode .civreplay");
    assert_eq!(
        encode_civreplay(&log).expect("re-encode .civreplay"),
        bytes.as_ref()
    );
    assert_eq!(log.seed, 17);
}

#[tokio::test]
async fn replay_import_matches_exported_tick_after_ticks() {
    let sim = Arc::new(tokio::sync::Mutex::new(Simulation::with_seed(17)));
    let export_addr = spawn_ws_bridge(sim, 4).await;
    let client = reqwest::Client::new();
    let healthz_url = format!("http://{export_addr}/healthz");
    let export_url = format!("http://{export_addr}/replay/export");

    timeout(Duration::from_secs(5), async {
        loop {
            let tick = reqwest::get(&healthz_url)
                .await
                .expect("healthz")
                .json::<serde_json::Value>()
                .await
                .expect("healthz json")
                .get("tick")
                .and_then(|v| v.as_u64())
                .unwrap_or(0);
            if tick >= 3 {
                return;
            }
            tokio::time::sleep(Duration::from_millis(50)).await;
        }
    })
    .await
    .expect("bridge should advance ticks");

    let bytes = client
        .get(&export_url)
        .send()
        .await
        .expect("replay export")
        .bytes()
        .await
        .expect("replay export body");

    let log = decode_civreplay(&bytes).expect("decode exported replay");
    let mut expected = Simulation::with_seed(log.seed);
    log.replay(&mut expected).expect("replay exported log");
    let expected_tick = expected.state.tick;
    assert!(expected_tick >= 3, "export should reflect advanced ticks");

    let import_sim = Arc::new(tokio::sync::Mutex::new(Simulation::with_seed(99)));
    let import_addr = spawn_ws_bridge(import_sim, 4).await;
    let import_url = format!("http://{import_addr}/replay/import");

    let response = client
        .post(&import_url)
        .header(reqwest::header::CONTENT_TYPE, "application/octet-stream")
        .body(bytes)
        .send()
        .await
        .expect("replay import");
    assert!(response.status().is_success());

    let body: serde_json::Value = response.json().await.expect("replay import json");
    assert_eq!(body.get("ok"), Some(&serde_json::json!(true)));
    assert_eq!(
        body.get("tick").and_then(|v| v.as_u64()),
        Some(expected_tick)
    );

    let imported_tick = reqwest::get(format!("http://{import_addr}/healthz"))
        .await
        .expect("healthz after import")
        .json::<serde_json::Value>()
        .await
        .expect("healthz json")
        .get("tick")
        .and_then(|v| v.as_u64())
        .expect("healthz tick");
    assert_eq!(imported_tick, expected_tick);
}

#[tokio::test]
async fn replay_import_loads_civreplay_bytes_into_bridge() {
    let mut source = Simulation::with_seed(42);
    for _ in 0..5 {
        source.tick();
    }
    let bytes = encode_civreplay(source.replay_log()).expect("encode replay");
    let expected_tick = source.state.tick;

    let sim = Arc::new(tokio::sync::Mutex::new(Simulation::with_seed(99)));
    let addr = spawn_ws_bridge(sim, 4).await;
    let url = format!("http://{addr}/replay/import");

    let client = reqwest::Client::new();
    let response = client
        .post(&url)
        .header(reqwest::header::CONTENT_TYPE, "application/octet-stream")
        .body(bytes)
        .send()
        .await
        .expect("replay import request");
    assert!(response.status().is_success());

    let body: serde_json::Value = response.json().await.expect("replay import json");
    assert_eq!(body.get("ok"), Some(&serde_json::json!(true)));
    assert_eq!(
        body.get("tick").and_then(|v| v.as_u64()),
        Some(expected_tick)
    );

    let health = reqwest::get(format!("http://{addr}/healthz"))
        .await
        .expect("healthz request")
        .json::<serde_json::Value>()
        .await
        .expect("healthz json");
    assert_eq!(
        health.get("tick").and_then(|v| v.as_u64()),
        Some(expected_tick)
    );
}

#[tokio::test]
async fn healthz_returns_ok_with_tick() {
    let sim = Arc::new(tokio::sync::Mutex::new(Simulation::with_seed(1)));
    let addr = spawn_ws_bridge(sim, 4).await;
    let url = format!("http://{addr}/healthz");

    let response = reqwest::get(&url).await.expect("healthz request");
    assert!(response.status().is_success());

    let body: serde_json::Value = response.json().await.expect("healthz json");
    assert!(body.get("tick").and_then(|v| v.as_u64()).is_some());
}

#[tokio::test]
async fn healthz_reports_ws_delivery_summary_after_tick() {
    let sim = Arc::new(tokio::sync::Mutex::new(Simulation::with_seed(1)));
    let addr = spawn_ws_bridge(sim, 4).await;
    let ws_url = format!("ws://{addr}/ws");
    let healthz_url = format!("http://{addr}/healthz");

    let (_socket, _) = connect_async(&ws_url).await.expect("ws connect");
    tokio::time::sleep(Duration::from_millis(250)).await;

    let response = reqwest::get(&healthz_url).await.expect("healthz request");
    assert!(response.status().is_success());

    let body: serde_json::Value = response.json().await.expect("healthz json");
    assert!(body
        .get("tick_batches_sent")
        .and_then(|v| v.as_u64())
        .is_some_and(|v| v > 0));
    assert!(body
        .get("tick_messages_sent")
        .and_then(|v| v.as_u64())
        .is_some_and(|v| v > 0));
    assert_eq!(
        body.get("ws_client_disconnects").and_then(|v| v.as_u64()),
        Some(0)
    );
}

#[tokio::test]
async fn ws_jsonrpc_health_returns_tick() {
    let sim = Arc::new(tokio::sync::Mutex::new(Simulation::with_seed(3)));
    let addr = spawn_ws_bridge(sim, 4).await;
    let url = format!("ws://{addr}/ws");

    let (mut socket, _) = connect_async(&url).await.expect("ws connect");
    socket
        .send(Message::Text(
            r#"{"jsonrpc":"2.0","id":1,"method":"health","params":{}}"#.into(),
        ))
        .await
        .expect("send health");

    let response = timeout(Duration::from_secs(2), async {
        while let Some(frame) = socket.next().await {
            let Message::Text(text) = frame.expect("ws frame") else {
                continue;
            };
            let value: serde_json::Value = serde_json::from_str(&text).expect("json text frame");
            if value.get("jsonrpc").and_then(|v| v.as_str()) == Some("2.0")
                && value.get("result").is_some()
            {
                return value;
            }
        }
        panic!("ws closed before jsonrpc response");
    })
    .await
    .expect("jsonrpc health timeout");

    assert_eq!(response.get("id"), Some(&serde_json::json!(1)));
    assert!(response
        .pointer("/result/tick")
        .and_then(|v| v.as_u64())
        .is_some());
}

#[tokio::test]
async fn ws_jsonrpc_invalid_json_returns_parse_error() {
    let sim = Arc::new(tokio::sync::Mutex::new(Simulation::with_seed(8)));
    let addr = spawn_ws_bridge(sim, 4).await;
    let url = format!("ws://{addr}/ws");

    let (mut socket, _) = connect_async(&url).await.expect("ws connect");
    socket
        .send(Message::Text("{not json".into()))
        .await
        .expect("send invalid json");

    let response = timeout(Duration::from_secs(2), async {
        while let Some(frame) = socket.next().await {
            let Message::Text(text) = frame.expect("ws frame") else {
                continue;
            };
            let value: serde_json::Value = serde_json::from_str(&text).expect("json text frame");
            if value.get("error").is_some() {
                return value;
            }
        }
        panic!("ws closed before jsonrpc error response");
    })
    .await
    .expect("jsonrpc parse error timeout");

    assert_eq!(
        response.get("jsonrpc").and_then(|v| v.as_str()),
        Some("2.0")
    );
    assert_eq!(response.get("id"), Some(&serde_json::Value::Null));
    assert_eq!(
        response.pointer("/error/code").and_then(|v| v.as_i64()),
        Some(i64::from(error_code::PARSE_ERROR))
    );
}

#[tokio::test]
async fn ws_jsonrpc_sim_status_returns_tick_and_population() {
    let sim = Arc::new(tokio::sync::Mutex::new(Simulation::with_seed(6)));
    let expected_population = {
        let guard = sim.lock().await;
        guard.snapshot().population
    };
    let addr = spawn_ws_bridge(sim, 4).await;
    let url = format!("ws://{addr}/ws");

    let (mut socket, _) = connect_async(&url).await.expect("ws connect");
    socket
        .send(Message::Text(
            r#"{"jsonrpc":"2.0","id":9,"method":"sim.status","params":{}}"#.into(),
        ))
        .await
        .expect("send sim.status");

    let response = timeout(Duration::from_secs(2), async {
        while let Some(frame) = socket.next().await {
            let Message::Text(text) = frame.expect("ws frame") else {
                continue;
            };
            let value: serde_json::Value = serde_json::from_str(&text).expect("json text frame");
            if value.get("id") == Some(&serde_json::json!(9)) {
                return value;
            }
        }
        panic!("ws closed before sim.status response");
    })
    .await
    .expect("sim.status timeout");

    assert!(response
        .pointer("/result/tick")
        .and_then(|v| v.as_u64())
        .is_some());
    assert_eq!(
        response
            .pointer("/result/population")
            .and_then(|v| v.as_u64()),
        Some(expected_population)
    );
}

#[tokio::test]
async fn ws_jsonrpc_sim_snapshot_returns_snapshot_fields() {
    let sim = Arc::new(tokio::sync::Mutex::new(Simulation::with_seed(21)));
    let addr = spawn_ws_bridge(sim.clone(), 4).await;
    let url = format!("ws://{addr}/ws");

    let (mut socket, _) = connect_async(&url).await.expect("ws connect");

    socket
        .send(Message::Text(
            r#"{"jsonrpc":"2.0","id":9,"method":"sim.set_speed","params":{"multiplier":4}}"#.into(),
        ))
        .await
        .expect("send sim.set_speed");

    let set_speed_response = timeout(Duration::from_secs(2), async {
        while let Some(frame) = socket.next().await {
            let Message::Text(text) = frame.expect("ws frame") else {
                continue;
            };
            let value: serde_json::Value = serde_json::from_str(&text).expect("json text frame");
            if value.get("id") == Some(&serde_json::json!(9)) {
                return value;
            }
        }
        panic!("ws closed before sim.set_speed response");
    })
    .await
    .expect("sim.set_speed timeout");

    assert_eq!(
        set_speed_response
            .pointer("/result/multiplier")
            .and_then(|v| v.as_u64()),
        Some(4)
    );

    socket
        .send(Message::Text(
            r#"{"jsonrpc":"2.0","id":10,"method":"sim.snapshot","params":{}}"#.into(),
        ))
        .await
        .expect("send sim.snapshot");

    let response = timeout(Duration::from_secs(2), async {
        while let Some(frame) = socket.next().await {
            let Message::Text(text) = frame.expect("ws frame") else {
                continue;
            };
            let value: serde_json::Value = serde_json::from_str(&text).expect("json text frame");
            if value.get("id") == Some(&serde_json::json!(10)) {
                return value;
            }
        }
        panic!("ws closed before sim.snapshot response");
    })
    .await
    .expect("sim.snapshot timeout");

    assert!(response.get("error").is_none());
    assert!(
        response
            .pointer("/result/tick")
            .and_then(|v| v.as_u64())
            .is_some(),
        "expected tick in snapshot result"
    );
    assert!(
        response
            .pointer("/result/population")
            .and_then(|v| v.as_u64())
            .is_some(),
        "expected population in snapshot result"
    );
    assert!(
        response
            .pointer("/result/building_count")
            .and_then(|v| v.as_u64())
            .is_some(),
        "expected building_count in snapshot result"
    );
    assert!(
        response
            .pointer("/result/energy_budget")
            .and_then(|v| v.as_f64())
            .is_some(),
        "expected energy_budget in snapshot result"
    );
    assert_eq!(
        response
            .pointer("/result/speed_multiplier")
            .and_then(|v| v.as_u64()),
        Some(4),
        "expected speed_multiplier in snapshot result after sim.set_speed"
    );
    let snap = sim.lock().await.snapshot();
    // The food price EMERGES from live supply/demand pressure (faction wealth +
    // population vs carrying capacity) and so moves every tick. The WS response
    // is captured at one tick and this snapshot at a later one, so an exact
    // cross-tick equality is racy by construction. Assert the field is present
    // and a valid clearing price (>= 1) in both reads instead.
    let response_food = response
        .pointer("/result/market_prices/food")
        .and_then(|v| v.as_i64());
    assert!(
        response_food.is_some_and(|p| p >= 1),
        "expected a positive food price in market_prices, got {response_food:?}"
    );
    assert!(
        snap.market_prices
            .get("food")
            .copied()
            .is_some_and(|p| p >= 1),
        "expected a positive food price in sim snapshot"
    );
    // Energy, like food, emerges from live supply/demand pressure and moves
    // every tick, so the cross-tick equality is racy. Assert present + positive.
    let response_energy = response
        .pointer("/result/market_prices/energy")
        .and_then(|v| v.as_i64());
    assert!(
        response_energy.is_some_and(|p| p >= 1),
        "expected a positive energy price in market_prices, got {response_energy:?}"
    );
    assert!(
        snap.market_prices
            .get("energy")
            .copied()
            .is_some_and(|p| p >= 1),
        "expected a positive energy price in sim snapshot"
    );

    let civ_pins = response
        .pointer("/result/civ_pins")
        .and_then(|v| v.as_array())
        .expect("civ_pins array in snapshot");
    assert!(
        !civ_pins.is_empty(),
        "expected civ_pins when startup civilians exist"
    );
    assert!(
        civ_pins
            .iter()
            .any(|pin| { pin.get("job").map(|j| !j.is_null()).unwrap_or(false) }),
        "expected at least one civ_pin with non-null job (UX-01 Citizen wire), got {civ_pins:?}"
    );
    assert!(
        civ_pins
            .iter()
            .any(|pin| { pin.get("job").and_then(|j| j.as_str()) == Some("farmer") }),
        "expected lowercase job label in civ_pins, got {civ_pins:?}"
    );
}

/// Three parallel clients each receive an `F3D0` binary tick broadcast within 3s
/// when the bridge is configured for binary-only tick encoding.
#[tokio::test]
async fn ws_smoke() {
    const PARALLEL_CLIENTS: usize = 3;
    let sim = Arc::new(tokio::sync::Mutex::new(Simulation::with_seed(42)));
    let addr = spawn_ws_bridge_with_config(
        sim,
        WsBridgeConfig {
            addr: SocketAddr::from(([127, 0, 0, 1], 0)),
            max_clients: PARALLEL_CLIENTS,
            require_role: false,
            tick_broadcast_format: TickBroadcastFormat::Binary,
            ..Default::default()
        },
    )
    .await;
    let url = format!("ws://{addr}/ws");

    let mut handles = Vec::with_capacity(PARALLEL_CLIENTS);
    for client_idx in 0..PARALLEL_CLIENTS {
        let url = url.clone();
        handles.push(tokio::spawn(async move {
            let (mut socket, _) = connect_async(&url).await.expect("ws connect");
            timeout(Duration::from_secs(3), async {
                while let Some(frame) = socket.next().await {
                    match frame.expect("ws frame") {
                        Message::Binary(bytes) => {
                            if bytes.starts_with(FRAME3D_BINARY_MAGIC) {
                                decode_frame3d_binary(&bytes).expect("F3D0 binary frame");
                                return;
                            }
                        }
                        Message::Text(text) => {
                            panic!("binary-only bridge emitted text frame: {text}");
                        }
                        _ => {}
                    }
                }
                panic!("ws closed before F3D0 binary frame (client {client_idx})");
            })
            .await
            .unwrap_or_else(|_| panic!("F3D0 binary frame timeout (client {client_idx})"));
        }));
    }

    for handle in handles {
        handle.await.expect("client task join");
    }
}

#[tokio::test]
async fn ws_jsonrpc_sim_command_tick_rejects_missing_role_when_required() {
    let sim = Arc::new(tokio::sync::Mutex::new(Simulation::with_seed(12)));
    let addr = spawn_ws_bridge_with_config(
        sim,
        WsBridgeConfig {
            addr: SocketAddr::from(([127, 0, 0, 1], 0)),
            max_clients: 4,
            require_role: true,
            tick_broadcast_format: TickBroadcastFormat::Both,
            ..Default::default()
        },
    )
    .await;
    let url = format!("ws://{addr}/ws");

    let (mut socket, _) = connect_async(&url).await.expect("ws connect");
    socket
        .send(Message::Text(
            r#"{"jsonrpc":"2.0","id":3,"method":"sim.command","params":{"action":"tick"}}"#.into(),
        ))
        .await
        .expect("send sim.command tick without role");

    let response = timeout(Duration::from_secs(2), async {
        while let Some(frame) = socket.next().await {
            let Message::Text(text) = frame.expect("ws frame") else {
                continue;
            };
            let value: serde_json::Value = serde_json::from_str(&text).expect("json text frame");
            if value.get("id") == Some(&serde_json::json!(3)) {
                return value;
            }
        }
        panic!("ws closed before sim.command error response");
    })
    .await
    .expect("sim.command forbidden timeout");

    assert_eq!(
        response.pointer("/error/code").and_then(|v| v.as_i64()),
        Some(i64::from(error_code::FORBIDDEN))
    );
    assert_eq!(
        response
            .pointer("/error/data/required_role")
            .and_then(|v| v.as_str()),
        Some("operator")
    );
}

#[tokio::test]
async fn ws_jsonrpc_sim_command_tick_accepts_x_civis_role_header() {
    let sim = Arc::new(tokio::sync::Mutex::new(Simulation::with_seed(18)));
    let addr = spawn_ws_bridge_with_config(
        sim,
        WsBridgeConfig {
            addr: SocketAddr::from(([127, 0, 0, 1], 0)),
            max_clients: 4,
            require_role: true,
            tick_broadcast_format: TickBroadcastFormat::Both,
            ..Default::default()
        },
    )
    .await;
    let url = format!("ws://{addr}/ws");

    let mut request = url.as_str().into_client_request().expect("ws request");
    request
        .headers_mut()
        .insert("x-civis-role", HeaderValue::from_static("operator"));
    let (mut socket, _) = connect_async(request)
        .await
        .expect("ws connect with role header");

    socket
        .send(Message::Text(
            r#"{"jsonrpc":"2.0","id":4,"method":"sim.command","params":{"action":"tick"}}"#.into(),
        ))
        .await
        .expect("send sim.command tick with header role");

    let response = timeout(Duration::from_secs(2), async {
        while let Some(frame) = socket.next().await {
            let Message::Text(text) = frame.expect("ws frame") else {
                continue;
            };
            let value: serde_json::Value = serde_json::from_str(&text).expect("json text frame");
            if value.get("id") == Some(&serde_json::json!(4)) {
                return value;
            }
        }
        panic!("ws closed before sim.command response");
    })
    .await
    .expect("sim.command header role timeout");

    assert_eq!(
        response.pointer("/result/accepted"),
        Some(&serde_json::json!(true))
    );
    assert!(response
        .pointer("/result/tick")
        .and_then(|v| v.as_u64())
        .is_some());
}

#[tokio::test]
async fn ws_jsonrpc_sim_set_policy_rejects_nan_scarcity() {
    let sim = Arc::new(tokio::sync::Mutex::new(Simulation::with_seed(44)));
    let addr = spawn_ws_bridge(sim, 4).await;
    let url = format!("ws://{addr}/ws");

    let (mut socket, _) = connect_async(&url).await.expect("ws connect");
    let set_policy_req = serde_json::json!({
        "jsonrpc": "2.0",
        "id": 50,
        "method": "sim.set_policy",
        "params": { "scarcity_multiplier": f64::NAN }
    });
    socket
        .send(Message::Text(set_policy_req.to_string()))
        .await
        .expect("send sim.set_policy nan");

    let response = timeout(Duration::from_secs(2), async {
        while let Some(frame) = socket.next().await {
            let Message::Text(text) = frame.expect("ws frame") else {
                continue;
            };
            let value: serde_json::Value = serde_json::from_str(&text).expect("json text frame");
            if value.get("id") == Some(&serde_json::json!(50)) {
                return value;
            }
        }
        panic!("ws closed before sim.set_policy error response");
    })
    .await
    .expect("sim.set_policy nan timeout");

    assert_eq!(
        response.pointer("/error/code").and_then(|v| v.as_i64()),
        Some(i64::from(error_code::INVALID_PARAMS))
    );
}

#[tokio::test]
async fn ws_jsonrpc_sim_command_tick_advances_tick() {
    let sim = Arc::new(tokio::sync::Mutex::new(Simulation::with_seed(4)));
    let addr = spawn_ws_bridge(sim, 4).await;
    let url = format!("ws://{addr}/ws");

    let (mut socket, _) = connect_async(&url).await.expect("ws connect");

    let tick_before = timeout(Duration::from_secs(2), async {
        socket
            .send(Message::Text(
                r#"{"jsonrpc":"2.0","id":1,"method":"health","params":{}}"#.into(),
            ))
            .await
            .expect("send health");
        while let Some(frame) = socket.next().await {
            let Message::Text(text) = frame.expect("ws frame") else {
                continue;
            };
            let value: serde_json::Value = serde_json::from_str(&text).expect("json text frame");
            if value.get("id") == Some(&serde_json::json!(1)) {
                return value
                    .pointer("/result/tick")
                    .and_then(|v| v.as_u64())
                    .expect("health tick");
            }
        }
        panic!("ws closed before health response");
    })
    .await
    .expect("health timeout");

    socket
        .send(Message::Text(
            r#"{"jsonrpc":"2.0","id":2,"method":"sim.command","params":{"action":"tick"}}"#.into(),
        ))
        .await
        .expect("send sim.command tick");

    let tick_after = timeout(Duration::from_secs(2), async {
        while let Some(frame) = socket.next().await {
            let Message::Text(text) = frame.expect("ws frame") else {
                continue;
            };
            let value: serde_json::Value = serde_json::from_str(&text).expect("json text frame");
            if value.get("id") == Some(&serde_json::json!(2)) {
                assert_eq!(
                    value.pointer("/result/accepted"),
                    Some(&serde_json::json!(true))
                );
                return value
                    .pointer("/result/tick")
                    .and_then(|v| v.as_u64())
                    .expect("sim.command tick");
            }
        }
        panic!("ws closed before sim.command response");
    })
    .await
    .expect("sim.command timeout");

    assert!(
        tick_after > tick_before,
        "tick should advance: {tick_before} -> {tick_after}"
    );
}

#[tokio::test]
async fn ws_jsonrpc_sim_reset_replaces_simulation_and_zeroes_tick() {
    let sim = Arc::new(tokio::sync::Mutex::new(Simulation::with_seed(17)));
    let expected_population = Simulation::with_seed(4242).snapshot().population;
    let addr = spawn_ws_bridge(sim, 4).await;
    let url = format!("ws://{addr}/ws");

    let (mut socket, _) = connect_async(&url).await.expect("ws connect");

    // Drain first tick broadcast so bridge tick is > 0 before reset.
    timeout(Duration::from_secs(2), async {
        while let Some(frame) = socket.next().await {
            let Message::Text(_) = frame.expect("ws frame") else {
                continue;
            };
            return;
        }
        panic!("ws closed before tick broadcast");
    })
    .await
    .expect("tick broadcast timeout");

    let tick_before_reset = timeout(Duration::from_secs(2), async {
        socket
            .send(Message::Text(
                r#"{"jsonrpc":"2.0","id":29,"method":"health","params":{}}"#.into(),
            ))
            .await
            .expect("send health");
        while let Some(frame) = socket.next().await {
            let Message::Text(text) = frame.expect("ws frame") else {
                continue;
            };
            let value: serde_json::Value = serde_json::from_str(&text).expect("json text frame");
            if value.get("id") == Some(&serde_json::json!(29)) {
                return value
                    .pointer("/result/tick")
                    .and_then(|v| v.as_u64())
                    .expect("health tick");
            }
        }
        panic!("ws closed before health response");
    })
    .await
    .expect("health timeout");

    let reset_req = serde_json::json!({
        "jsonrpc": "2.0",
        "id": 30,
        "method": "sim.reset",
        "params": { "seed": 4242 }
    });
    socket
        .send(Message::Text(reset_req.to_string()))
        .await
        .expect("send sim.reset");

    let reset_response = timeout(Duration::from_secs(2), async {
        while let Some(frame) = socket.next().await {
            let Message::Text(text) = frame.expect("ws frame") else {
                continue;
            };
            let value: serde_json::Value = serde_json::from_str(&text).expect("json text frame");
            if value.get("id") == Some(&serde_json::json!(30)) {
                return value;
            }
        }
        panic!("ws closed before sim.reset response");
    })
    .await
    .expect("sim.reset timeout");

    assert_eq!(
        reset_response
            .pointer("/result/seed")
            .and_then(|v| v.as_u64()),
        Some(4242)
    );
    assert_eq!(
        reset_response
            .pointer("/result/tick")
            .and_then(|v| v.as_u64()),
        Some(0)
    );

    socket
        .send(Message::Text(
            r#"{"jsonrpc":"2.0","id":31,"method":"sim.status","params":{}}"#.into(),
        ))
        .await
        .expect("send sim.status");

    let status_response = timeout(Duration::from_secs(2), async {
        while let Some(frame) = socket.next().await {
            let Message::Text(text) = frame.expect("ws frame") else {
                continue;
            };
            let value: serde_json::Value = serde_json::from_str(&text).expect("json text frame");
            if value.get("id") == Some(&serde_json::json!(31)) {
                return value;
            }
        }
        panic!("ws closed before sim.status response");
    })
    .await
    .expect("sim.status timeout");

    let tick_after_reset = status_response
        .pointer("/result/tick")
        .and_then(|v| v.as_u64())
        .expect("status tick");
    assert!(
        tick_after_reset < tick_before_reset,
        "reset should drop tick: {tick_before_reset} -> {tick_after_reset}"
    );
    assert_eq!(
        status_response
            .pointer("/result/population")
            .and_then(|v| v.as_u64()),
        Some(expected_population)
    );
}

#[tokio::test]
async fn ws_jsonrpc_sim_save_and_load_replay_roundtrip() {
    let sim = Arc::new(tokio::sync::Mutex::new(Simulation::with_seed(10)));
    // The bridge resolves `path` under its `replays_dir` (canonicalize +
    // containment). Use a dedicated temp `replays_dir` and a simple
    // relative path so the saved file is deterministically locatable for
    // the read-back assertion.
    let replays_dir = tempfile::tempdir().expect("temp replays dir");
    let replay_rel = "replay-roundtrip.civreplay";
    let replay_path = replays_dir.path().join(replay_rel);

    let addr = spawn_ws_bridge_with_config(
        sim,
        WsBridgeConfig {
            addr: SocketAddr::from(([127, 0, 0, 1], 0)),
            max_clients: 4,
            require_role: false,
            tick_broadcast_format: TickBroadcastFormat::Both,
            saves_dir: tempfile::tempdir()
                .expect("temp saves dir")
                .keep(),
            replays_dir: replays_dir.path().to_path_buf(),
        },
    )
    .await;
    let url = format!("ws://{addr}/ws");
    let (mut socket, _) = connect_async(&url).await.expect("ws connect");

    let save_req = serde_json::json!({
        "jsonrpc": "2.0",
        "id": 20,
        "method": "sim.save_replay",
        "params": { "path": replay_rel }
    });
    socket
        .send(Message::Text(save_req.to_string()))
        .await
        .expect("send sim.save_replay");

    let save_response = timeout(Duration::from_secs(2), async {
        while let Some(frame) = socket.next().await {
            let Message::Text(text) = frame.expect("ws frame") else {
                continue;
            };
            let value: serde_json::Value = serde_json::from_str(&text).expect("json text frame");
            if value.get("id") == Some(&serde_json::json!(20)) {
                return value;
            }
        }
        panic!("ws closed before sim.save_replay response");
    })
    .await
    .expect("sim.save_replay timeout");

    assert_eq!(
        save_response.pointer("/result/saved"),
        Some(&serde_json::json!(true))
    );
    assert!(std::path::Path::new(&replay_path).is_file());

    let expected_tick = Simulation::load_replay_from_file(&replay_path)
        .expect("reload saved replay")
        .state
        .tick;

    let load_req = serde_json::json!({
        "jsonrpc": "2.0",
        "id": 21,
        "method": "sim.load_replay",
        "params": { "path": replay_rel }
    });
    socket
        .send(Message::Text(load_req.to_string()))
        .await
        .expect("send sim.load_replay");

    let load_response = timeout(Duration::from_secs(2), async {
        while let Some(frame) = socket.next().await {
            let Message::Text(text) = frame.expect("ws frame") else {
                continue;
            };
            let value: serde_json::Value = serde_json::from_str(&text).expect("json text frame");
            if value.get("id") == Some(&serde_json::json!(21)) {
                return value;
            }
        }
        panic!("ws closed before sim.load_replay response");
    })
    .await
    .expect("sim.load_replay timeout");

    assert_eq!(
        load_response.pointer("/result/loaded"),
        Some(&serde_json::json!(true))
    );
    assert_eq!(
        load_response
            .pointer("/result/tick")
            .and_then(|v| v.as_u64()),
        Some(expected_tick)
    );

    socket
        .send(Message::Text(
            r#"{"jsonrpc":"2.0","id":22,"method":"sim.status","params":{}}"#.into(),
        ))
        .await
        .expect("send sim.status");

    let status_response = timeout(Duration::from_secs(2), async {
        while let Some(frame) = socket.next().await {
            let Message::Text(text) = frame.expect("ws frame") else {
                continue;
            };
            let value: serde_json::Value = serde_json::from_str(&text).expect("json text frame");
            if value.get("id") == Some(&serde_json::json!(22)) {
                return value;
            }
        }
        panic!("ws closed before sim.status response");
    })
    .await
    .expect("sim.status timeout");

    // The background 10 Hz ticker may advance the simulation between
    // `sim.load_replay` and the `sim.status` response. Under CI/load this can
    // be more than one tick, so accept a small forward-only window.
    let actual_tick = status_response
        .pointer("/result/tick")
        .and_then(|v| v.as_u64())
        .expect("sim.status result.tick missing");
    assert!(
        (expected_tick..=expected_tick + 3).contains(&actual_tick),
        "expected tick in [{expected_tick}, {}] after load_replay, got {actual_tick}",
        expected_tick + 3
    );
}

#[tokio::test]
async fn ws_jsonrpc_sim_set_policy_zero_scarcity_tick_preserves_energy_budget() {
    let sim = Arc::new(tokio::sync::Mutex::new(Simulation::with_seed(33)));
    let addr = spawn_ws_bridge(sim.clone(), 4).await;
    let url = format!("ws://{addr}/ws");

    let (mut socket, _) = connect_async(&url).await.expect("ws connect");

    let set_policy_req = serde_json::json!({
        "jsonrpc": "2.0",
        "id": 40,
        "method": "sim.set_policy",
        "params": { "scarcity_multiplier": 0.0 }
    });
    socket
        .send(Message::Text(set_policy_req.to_string()))
        .await
        .expect("send sim.set_policy");

    let set_policy_response = timeout(Duration::from_secs(2), async {
        while let Some(frame) = socket.next().await {
            let Message::Text(text) = frame.expect("ws frame") else {
                continue;
            };
            let value: serde_json::Value = serde_json::from_str(&text).expect("json text frame");
            if value.get("id") == Some(&serde_json::json!(40)) {
                return value;
            }
        }
        panic!("ws closed before sim.set_policy response");
    })
    .await
    .expect("sim.set_policy timeout");

    assert_eq!(
        set_policy_response.pointer("/result/updated"),
        Some(&serde_json::json!(true))
    );
    assert_eq!(
        set_policy_response
            .pointer("/result/scarcity_multiplier")
            .and_then(|v| v.as_f64()),
        Some(0.0)
    );

    socket
        .send(Message::Text(
            r#"{"jsonrpc":"2.0","id":41,"method":"sim.snapshot","params":{}}"#.into(),
        ))
        .await
        .expect("send sim.snapshot");

    let snapshot_before = timeout(Duration::from_secs(2), async {
        while let Some(frame) = socket.next().await {
            let Message::Text(text) = frame.expect("ws frame") else {
                continue;
            };
            let value: serde_json::Value = serde_json::from_str(&text).expect("json text frame");
            if value.get("id") == Some(&serde_json::json!(41)) {
                return value;
            }
        }
        panic!("ws closed before sim.snapshot response");
    })
    .await
    .expect("sim.snapshot timeout");

    let budget_before = snapshot_before
        .pointer("/result/energy_budget")
        .and_then(|v| v.as_f64())
        .expect("energy_budget before tick");

    socket
        .send(Message::Text(
            r#"{"jsonrpc":"2.0","id":42,"method":"sim.command","params":{"action":"tick"}}"#.into(),
        ))
        .await
        .expect("send sim.command tick");

    let tick_response = timeout(Duration::from_secs(2), async {
        while let Some(frame) = socket.next().await {
            let Message::Text(text) = frame.expect("ws frame") else {
                continue;
            };
            let value: serde_json::Value = serde_json::from_str(&text).expect("json text frame");
            if value.get("id") == Some(&serde_json::json!(42)) {
                return value;
            }
        }
        panic!("ws closed before sim.command response");
    })
    .await
    .expect("sim.command timeout");

    assert_eq!(
        tick_response.pointer("/result/accepted"),
        Some(&serde_json::json!(true))
    );

    socket
        .send(Message::Text(
            r#"{"jsonrpc":"2.0","id":43,"method":"sim.snapshot","params":{}}"#.into(),
        ))
        .await
        .expect("send sim.snapshot after tick");

    let snapshot_after = timeout(Duration::from_secs(2), async {
        while let Some(frame) = socket.next().await {
            let Message::Text(text) = frame.expect("ws frame") else {
                continue;
            };
            let value: serde_json::Value = serde_json::from_str(&text).expect("json text frame");
            if value.get("id") == Some(&serde_json::json!(43)) {
                return value;
            }
        }
        panic!("ws closed before sim.snapshot response");
    })
    .await
    .expect("sim.snapshot after tick timeout");

    let budget_after = snapshot_after
        .pointer("/result/energy_budget")
        .and_then(|v| v.as_f64())
        .expect("energy_budget after tick");

    assert_eq!(
        budget_after, budget_before,
        "zero scarcity should prevent policy drain on tick"
    );

    let hash_chain_root = snapshot_after
        .pointer("/result/hash_chain_root")
        .and_then(|v| v.as_str())
        .expect("hash_chain_root after tick");
    assert_eq!(hash_chain_root.len(), 64);
    assert!(hash_chain_root.chars().all(|ch| ch.is_ascii_hexdigit()));
}

#[tokio::test]
async fn ws_jsonrpc_sim_set_speed_accepts_valid_multiplier() {
    let sim = Arc::new(tokio::sync::Mutex::new(Simulation::with_seed(44)));
    let addr = spawn_ws_bridge(sim, 4).await;
    let url = format!("ws://{addr}/ws");

    let (mut socket, _) = connect_async(&url).await.expect("ws connect");

    socket
        .send(Message::Text(
            r#"{"jsonrpc":"2.0","id":50,"method":"sim.set_speed","params":{"multiplier":2}}"#
                .into(),
        ))
        .await
        .expect("send sim.set_speed");

    let response = timeout(Duration::from_secs(2), async {
        while let Some(frame) = socket.next().await {
            let Message::Text(text) = frame.expect("ws frame") else {
                continue;
            };
            let value: serde_json::Value = serde_json::from_str(&text).expect("json text frame");
            if value.get("id") == Some(&serde_json::json!(50)) {
                return value;
            }
        }
        panic!("ws closed before sim.set_speed response");
    })
    .await
    .expect("sim.set_speed timeout");

    assert_eq!(
        response.pointer("/result/accepted"),
        Some(&serde_json::json!(true))
    );
    assert_eq!(
        response
            .pointer("/result/multiplier")
            .and_then(|v| v.as_u64()),
        Some(2)
    );

    socket
        .send(Message::Text(
            r#"{"jsonrpc":"2.0","id":52,"method":"sim.get_speed"}"#.into(),
        ))
        .await
        .expect("send sim.get_speed");

    let get_response = timeout(Duration::from_secs(2), async {
        while let Some(frame) = socket.next().await {
            let Message::Text(text) = frame.expect("ws frame") else {
                continue;
            };
            let value: serde_json::Value = serde_json::from_str(&text).expect("json text frame");
            if value.get("id") == Some(&serde_json::json!(52)) {
                return value;
            }
        }
        panic!("ws closed before sim.get_speed response");
    })
    .await
    .expect("sim.get_speed timeout");

    assert_eq!(
        get_response
            .pointer("/result/multiplier")
            .and_then(|v| v.as_u64()),
        Some(2)
    );
}

#[tokio::test]
async fn ws_jsonrpc_sim_set_speed_rejects_invalid_multiplier() {
    let sim = Arc::new(tokio::sync::Mutex::new(Simulation::with_seed(45)));
    let addr = spawn_ws_bridge(sim, 4).await;
    let url = format!("ws://{addr}/ws");

    let (mut socket, _) = connect_async(&url).await.expect("ws connect");

    socket
        .send(Message::Text(
            r#"{"jsonrpc":"2.0","id":51,"method":"sim.set_speed","params":{"multiplier":3}}"#
                .into(),
        ))
        .await
        .expect("send sim.set_speed invalid");

    let response = timeout(Duration::from_secs(2), async {
        while let Some(frame) = socket.next().await {
            let Message::Text(text) = frame.expect("ws frame") else {
                continue;
            };
            let value: serde_json::Value = serde_json::from_str(&text).expect("json text frame");
            if value.get("id") == Some(&serde_json::json!(51)) {
                return value;
            }
        }
        panic!("ws closed before sim.set_speed error response");
    })
    .await
    .expect("sim.set_speed invalid timeout");

    assert_eq!(
        response.pointer("/error/code").and_then(|v| v.as_i64()),
        Some(-32_602)
    );
}

fn assert_six_valid_frame3d_kinds(frames: &[Frame3d], expected_tick: u64) {
    assert_eq!(
        frames.len(),
        civ_server::ws_bridge::FRAME_BUNDLE_LEN,
        "expected full F3D0 bundle for tick {expected_tick}"
    );
    let mut has_voxel = false;
    let mut has_building = false;
    let mut has_agent = false;
    let mut has_civilian = false;
    let mut has_faction = false;
    let mut has_event = false;
    for frame in frames {
        assert_eq!(frame.tick(), expected_tick);
        match frame {
            Frame3d::VoxelDelta(_) => has_voxel = true,
            Frame3d::BuildingDiff(_) => has_building = true,
            Frame3d::AgentAppearance(_) => has_agent = true,
            Frame3d::CivilianState(_) => has_civilian = true,
            Frame3d::FactionState(_) => has_faction = true,
            Frame3d::EventFeed(_) => has_event = true,
            Frame3d::Climate(_) => {}
        }
    }
    assert!(has_voxel && has_building && has_agent && has_civilian && has_faction && has_event);
}

async fn collect_f3d0_frames_after_sim_command_tick(
    format: TickBroadcastFormat,
) -> (u64, Vec<Frame3d>) {
    let sim = Arc::new(tokio::sync::Mutex::new(Simulation::with_seed(14)));
    let addr = spawn_ws_bridge_with_config(
        sim,
        WsBridgeConfig {
            addr: SocketAddr::from(([127, 0, 0, 1], 0)),
            max_clients: 4,
            require_role: false,
            tick_broadcast_format: format,
            ..Default::default()
        },
    )
    .await;
    let url = format!("ws://{addr}/ws");

    let (mut socket, _) = connect_async(&url).await.expect("ws connect");
    socket
        .send(Message::Text(
            r#"{"jsonrpc":"2.0","id":1,"method":"sim.command","params":{"action":"tick"}}"#.into(),
        ))
        .await
        .expect("send sim.command tick");

    let mut tick_after = None;
    let mut pending_binaries: Vec<Frame3d> = Vec::new();
    let mut frames_for_tick = Vec::new();

    timeout(Duration::from_secs(3), async {
        while tick_after.is_none()
            || frames_for_tick.len() < civ_server::ws_bridge::FRAME_BUNDLE_LEN
        {
            let frame = socket
                .next()
                .await
                .expect("ws stream open")
                .expect("ws frame");
            match frame {
                Message::Text(text) => {
                    let value: serde_json::Value =
                        serde_json::from_str(&text).expect("json text frame");
                    if value.get("id") == Some(&serde_json::json!(1)) {
                        assert_eq!(
                            value.pointer("/result/accepted"),
                            Some(&serde_json::json!(true))
                        );
                        tick_after = value.pointer("/result/tick").and_then(|v| v.as_u64());
                        if let Some(tick) = tick_after {
                            for decoded in pending_binaries.drain(..) {
                                if decoded.tick() == tick {
                                    frames_for_tick.push(decoded);
                                }
                            }
                        }
                    }
                }
                Message::Binary(bytes) => {
                    assert!(
                        bytes.starts_with(FRAME3D_BINARY_MAGIC),
                        "binary tick broadcast must be F3D0-framed"
                    );
                    let decoded =
                        decode_frame3d_binary(&bytes).expect("valid F3D0 frame after sim.command");
                    if let Some(tick) = tick_after {
                        if decoded.tick() == tick {
                            frames_for_tick.push(decoded);
                        }
                    } else {
                        pending_binaries.push(decoded);
                    }
                }
                _ => {}
            }
        }
    })
    .await
    .expect("sim.command tick broadcast timeout");

    let tick = tick_after.expect("sim.command tick result");
    (tick, frames_for_tick)
}

/// CIV-0200 / FR-CIV-BEVY-028: `sim.command` tick broadcasts decodable six-frame F3D0 bundle when format is `Both`.
#[tokio::test]
async fn ws_sim_command_tick_broadcasts_f3d0_when_both() {
    let (tick, frames) =
        collect_f3d0_frames_after_sim_command_tick(TickBroadcastFormat::Both).await;
    assert!(tick > 0, "sim.command should advance tick");
    assert_six_valid_frame3d_kinds(&frames, tick);
}

/// CIV-0200 / FR-CIV-BEVY-028: `sim.command` tick broadcasts decodable six-frame F3D0 bundle when format is `Binary`.
#[tokio::test]
async fn ws_sim_command_tick_broadcasts_f3d0_when_binary() {
    let (tick, frames) =
        collect_f3d0_frames_after_sim_command_tick(TickBroadcastFormat::Binary).await;
    assert!(tick > 0, "sim.command should advance tick");
    assert_six_valid_frame3d_kinds(&frames, tick);
}

/// Tick push after `sim.command` emits the configured number of WebSocket frames.
#[tokio::test]
async fn ws_sim_command_tick_broadcast_message_count_binary_vs_both() {
    for format in [TickBroadcastFormat::Binary, TickBroadcastFormat::Both] {
        let count = count_tick_broadcast_ws_frames_after_sim_command(format).await;
        assert_eq!(
            count,
            format.messages_per_tick(),
            "{format:?} ws frame count"
        );
    }
}

async fn count_tick_broadcast_ws_frames_after_sim_command(format: TickBroadcastFormat) -> usize {
    let sim = Arc::new(tokio::sync::Mutex::new(Simulation::with_seed(21)));
    let addr = spawn_ws_bridge_with_config(
        sim,
        WsBridgeConfig {
            addr: SocketAddr::from(([127, 0, 0, 1], 0)),
            max_clients: 4,
            require_role: false,
            tick_broadcast_format: format,
            ..Default::default()
        },
    )
    .await;
    let url = format!("ws://{addr}/ws");

    let (mut socket, _) = connect_async(&url).await.expect("ws connect");

    // Halt the 10 Hz loop so only the `sim.command` tick produces broadcast frames.
    socket
        .send(Message::Text(
            r#"{"jsonrpc":"2.0","id":1,"method":"sim.set_speed","params":{"multiplier":0}}"#.into(),
        ))
        .await
        .expect("send sim.set_speed 0");
    wait_for_jsonrpc_id(&mut socket, 1).await;

    socket
        .send(Message::Text(
            r#"{"jsonrpc":"2.0","id":2,"method":"sim.command","params":{"action":"tick"}}"#.into(),
        ))
        .await
        .expect("send sim.command tick");

    let expected = format.messages_per_tick();
    let mut frame_count = 0usize;
    let mut command_done = false;

    timeout(Duration::from_secs(3), async {
        while !command_done || frame_count < expected {
            let frame = socket
                .next()
                .await
                .expect("ws stream open")
                .expect("ws frame");
            match frame {
                Message::Text(text) => {
                    if let Ok(value) = serde_json::from_str::<serde_json::Value>(&text) {
                        if value.get("id") == Some(&serde_json::json!(2)) {
                            command_done = true;
                            continue;
                        }
                        if value.get("jsonrpc").is_some() {
                            continue;
                        }
                    }
                    if format.sends_text() {
                        let _: Frame3d =
                            serde_json::from_str(&text).expect("Frame3d text broadcast");
                        frame_count += 1;
                    }
                }
                Message::Binary(bytes)
                    if bytes.starts_with(FRAME3D_BINARY_MAGIC) && format.sends_binary() =>
                {
                    decode_frame3d_binary(&bytes).expect("F3D0 frame");
                    frame_count += 1;
                }
                _ => {}
            }
        }
    })
    .await
    .expect("sim.command tick broadcast timeout");

    assert!(command_done, "sim.command should return jsonrpc id 2");
    frame_count
}

async fn wait_for_jsonrpc_id(
    socket: &mut tokio_tungstenite::WebSocketStream<
        tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>,
    >,
    id: u64,
) {
    timeout(Duration::from_secs(2), async {
        while let Some(frame) = socket.next().await {
            let Message::Text(text) = frame.expect("ws frame") else {
                continue;
            };
            let value: serde_json::Value = serde_json::from_str(&text).expect("json text frame");
            if value.get("id") == Some(&serde_json::json!(id)) {
                return;
            }
        }
        panic!("ws closed before jsonrpc id {id}");
    })
    .await
    .unwrap_or_else(|_| panic!("jsonrpc id {id} timeout"));
}

#[tokio::test]
async fn ws_client_receives_binary_frame3d_after_tick() {
    let sim = Arc::new(tokio::sync::Mutex::new(Simulation::with_seed(14)));
    let addr = spawn_ws_bridge(sim, 4).await;
    let url = format!("ws://{addr}/ws");

    let (mut socket, _) = connect_async(&url).await.expect("ws connect");
    let binary = timeout(Duration::from_secs(2), async {
        while let Some(frame) = socket.next().await {
            let Message::Binary(bytes) = frame.expect("ws frame") else {
                continue;
            };
            if bytes.starts_with(FRAME3D_BINARY_MAGIC) {
                return bytes;
            }
        }
        panic!("ws closed before F3D0 binary frame");
    })
    .await
    .expect("binary frame timeout");

    decode_frame3d_binary(&binary).expect("10 Hz tick binary must decode as F3D0");
}

#[tokio::test]
async fn ws_client_receives_text_frames_after_tick() {
    let sim = Arc::new(tokio::sync::Mutex::new(Simulation::with_seed(2)));
    let addr = spawn_ws_bridge(sim, 4).await;
    let url = format!("ws://{addr}/ws");

    let (mut socket, _) = connect_async(&url).await.expect("ws connect");
    let frame = timeout(Duration::from_secs(2), socket.next())
        .await
        .expect("ws frame timeout")
        .expect("ws stream open")
        .expect("ws frame");

    match frame {
        Message::Text(text) => assert!(!text.is_empty()),
        other => panic!("expected text frame, got {other:?}"),
    }
}

/// FR-PROTO-001: ten concurrent clients each receive tick-broadcast text frames.
/// Covers FR-PROTO-001.
#[tokio::test]
async fn ws_ten_clients_each_receive_text_frame() {
    const MAX_CLIENTS: usize = 10;
    let sim = Arc::new(tokio::sync::Mutex::new(Simulation::with_seed(5)));
    let addr = spawn_ws_bridge(sim, MAX_CLIENTS).await;
    let url = format!("ws://{addr}/ws");

    let hold_open = Arc::new(std::sync::atomic::AtomicBool::new(true));
    let (ready_tx, mut ready_rx) = tokio::sync::mpsc::channel(MAX_CLIENTS);

    let mut handles = Vec::with_capacity(MAX_CLIENTS);
    for _client_idx in 0..MAX_CLIENTS {
        let url = url.clone();
        let hold_open = Arc::clone(&hold_open);
        let ready_tx = ready_tx.clone();
        handles.push(tokio::spawn(async move {
            let (mut socket, _) = connect_async(&url).await.expect("ws connect");
            timeout(Duration::from_secs(3), async {
                while let Some(frame) = socket.next().await {
                    let Message::Text(text) = frame.expect("ws frame") else {
                        continue;
                    };
                    if !text.is_empty() {
                        ready_tx
                            .send(())
                            .await
                            .expect("ready signal (client {client_idx})");
                        break;
                    }
                }
                while hold_open.load(std::sync::atomic::Ordering::Relaxed) {
                    tokio::time::sleep(Duration::from_millis(20)).await;
                }
            })
            .await
            .expect("ws frame timeout (client {client_idx})");
        }));
    }
    drop(ready_tx);

    for _ in 0..MAX_CLIENTS {
        timeout(Duration::from_secs(3), ready_rx.recv())
            .await
            .expect("ready timeout")
            .expect("ready channel");
    }

    // 11th client: upgrade may succeed but bridge rejects with Close while slots are full.
    let (mut socket, _) = connect_async(&url).await.expect("11th ws connect");
    let rejected = timeout(Duration::from_secs(1), async {
        while let Some(frame) = socket.next().await {
            match frame.expect("ws frame") {
                Message::Close(_) => return true,
                Message::Text(_) => return false,
                _ => {}
            }
        }
        true
    })
    .await
    .expect("11th client timeout");

    hold_open.store(false, std::sync::atomic::Ordering::Relaxed);
    for handle in handles {
        handle.await.expect("client task join");
    }

    assert!(
        rejected,
        "11th client should be closed or receive Close frame"
    );
}

#[tokio::test]
async fn ws_jsonrpc_sim_spawn_civilian_returns_entity_id() {
    let sim = Arc::new(tokio::sync::Mutex::new(Simulation::with_seed(4)));
    let addr = spawn_ws_bridge(sim, 4).await;
    let url = format!("ws://{addr}/ws");

    let (mut socket, _) = connect_async(&url).await.expect("ws connect");

    socket
        .send(Message::Text(
            r#"{"jsonrpc":"2.0","id":7,"method":"sim.spawn_civilian","params":{"x":0.4,"y":0.6,"faction":1}}"#
                .into(),
        ))
        .await
        .expect("send sim.spawn_civilian");

    let entity_id = timeout(Duration::from_secs(2), async {
        while let Some(frame) = socket.next().await {
            let Message::Text(text) = frame.expect("ws frame") else {
                continue;
            };
            let value: serde_json::Value = serde_json::from_str(&text).expect("json");
            if value.get("id") == Some(&serde_json::json!(7)) {
                assert_eq!(value.pointer("/result/ok"), Some(&serde_json::json!(true)));
                return value
                    .pointer("/result/entity_id")
                    .and_then(|v| v.as_u64())
                    .expect("entity_id");
            }
        }
        panic!("ws closed before spawn response");
    })
    .await
    .expect("spawn timeout");

    assert!(entity_id > 0, "entity_id should be non-zero");
}

#[tokio::test]
async fn ws_jsonrpc_sim_spawn_entity_vehicle_returns_entity_id() {
    let sim = Arc::new(tokio::sync::Mutex::new(Simulation::with_seed(4)));
    let addr = spawn_ws_bridge(sim, 4).await;
    let url = format!("ws://{addr}/ws");

    let (mut socket, _) = connect_async(&url).await.expect("ws connect");

    socket
        .send(Message::Text(
            r#"{"jsonrpc":"2.0","id":71,"method":"sim.spawn_entity","params":{"kind":"vehicle","x":0.3,"y":0.7,"faction":0}}"#
                .into(),
        ))
        .await
        .expect("send sim.spawn_entity");

    timeout(Duration::from_secs(2), async {
        while let Some(frame) = socket.next().await {
            let Message::Text(text) = frame.expect("ws frame") else {
                continue;
            };
            let value: serde_json::Value = serde_json::from_str(&text).expect("json");
            if value.get("id") == Some(&serde_json::json!(71)) {
                assert_eq!(value.pointer("/result/ok"), Some(&serde_json::json!(true)));
                assert!(
                    value
                        .pointer("/result/entity_id")
                        .and_then(|v| v.as_u64())
                        .unwrap_or(0)
                        > 0
                );
                return;
            }
        }
        panic!("ws closed before spawn_entity response");
    })
    .await
    .expect("spawn_entity timeout");
}

#[tokio::test]
async fn ws_jsonrpc_spawn_civilian_pin_appears_in_snapshot() {
    let sim = Arc::new(tokio::sync::Mutex::new(Simulation::with_seed(4)));
    let addr = spawn_ws_bridge(sim, 4).await;
    let url = format!("ws://{addr}/ws");

    let (mut socket, _) = connect_async(&url).await.expect("ws connect");

    socket
        .send(Message::Text(
            r#"{"jsonrpc":"2.0","id":72,"method":"sim.spawn_civilian","params":{"x":0.42,"y":0.58,"faction":2}}"#
                .into(),
        ))
        .await
        .expect("send spawn");

    timeout(Duration::from_secs(3), async {
        let mut snapshot_sent = false;
        while let Some(frame) = socket.next().await {
            let Message::Text(text) = frame.expect("ws frame") else {
                continue;
            };
            let value: serde_json::Value = serde_json::from_str(&text).expect("json");
            if value.get("id") == Some(&serde_json::json!(72)) {
                assert_eq!(value.pointer("/result/ok"), Some(&serde_json::json!(true)));
                socket
                    .send(Message::Text(
                        r#"{"jsonrpc":"2.0","id":73,"method":"sim.snapshot","params":{}}"#.into(),
                    ))
                    .await
                    .expect("snapshot");
                snapshot_sent = true;
                continue;
            }
            if snapshot_sent && value.get("id") == Some(&serde_json::json!(73)) {
                let pins = value
                    .pointer("/result/civ_pins")
                    .and_then(|v| v.as_array())
                    .expect("civ_pins");
                let found = pins.iter().any(|pin| {
                    let x = pin.get("x").and_then(|v| v.as_f64()).unwrap_or(0.0);
                    let y = pin.get("y").and_then(|v| v.as_f64()).unwrap_or(0.0);
                    (x - 0.42).abs() < 0.03 && (y - 0.58).abs() < 0.03
                });
                assert!(found, "spawn pin missing from snapshot: {pins:?}");
                return;
            }
        }
        panic!("ws closed before snapshot pin check");
    })
    .await
    .expect("spawn+snapshot pin timeout");
}

#[tokio::test]
async fn ws_jsonrpc_sim_damage_accepts_event() {
    let sim = Arc::new(tokio::sync::Mutex::new(Simulation::with_seed(4)));
    let addr = spawn_ws_bridge(sim, 4).await;
    let url = format!("ws://{addr}/ws");

    let (mut socket, _) = connect_async(&url).await.expect("ws connect");

    socket
        .send(Message::Text(
            r#"{"jsonrpc":"2.0","id":8,"method":"sim.damage","params":{"x":1000000,"y":0,"z":1000000,"radius":4}}"#
                .into(),
        ))
        .await
        .expect("send sim.damage");

    timeout(Duration::from_secs(2), async {
        while let Some(frame) = socket.next().await {
            let Message::Text(text) = frame.expect("ws frame") else {
                continue;
            };
            let value: serde_json::Value = serde_json::from_str(&text).expect("json");
            if value.get("id") == Some(&serde_json::json!(8)) {
                assert_eq!(value.pointer("/result/ok"), Some(&serde_json::json!(true)));
                assert_eq!(
                    value.pointer("/result/queued"),
                    Some(&serde_json::json!(true))
                );
                return;
            }
        }
        panic!("ws closed before damage response");
    })
    .await
    .expect("damage timeout");
}

/// FR-CIV-UX-006 — spawn palette kinds accepted over WS JSON-RPC.
/// Covers FR-CIV-UX-006.
#[tokio::test]
async fn ws_jsonrpc_spawn_palette_all_kinds_accepted() {
    let kinds = ["civilian", "vehicle", "airport", "port", "hangar"];
    for (idx, kind) in kinds.iter().enumerate() {
        let sim = Arc::new(tokio::sync::Mutex::new(Simulation::with_seed(
            40 + idx as u64,
        )));
        let addr = spawn_ws_bridge(sim, 4).await;
        let url = format!("ws://{addr}/ws");
        let (mut socket, _) = connect_async(&url).await.expect("ws connect");
        let id = 200 + idx as u64;
        let req = format!(
            r#"{{"jsonrpc":"2.0","id":{id},"method":"sim.spawn_entity","params":{{"kind":"{kind}","x":0.2,"y":0.3,"faction":0}}}}"#
        );
        socket
            .send(Message::Text(req))
            .await
            .expect("send spawn_entity");

        timeout(Duration::from_secs(2), async {
            while let Some(frame) = socket.next().await {
                let Message::Text(text) = frame.expect("ws frame") else {
                    continue;
                };
                let value: serde_json::Value = serde_json::from_str(&text).expect("json");
                if value.get("id") == Some(&serde_json::json!(id)) {
                    assert_eq!(
                        value.pointer("/result/ok"),
                        Some(&serde_json::json!(true)),
                        "spawn_entity kind={kind}"
                    );
                    assert_eq!(
                        value.pointer("/result/kind"),
                        Some(&serde_json::json!(kind)),
                        "spawn_entity kind echo for {kind}"
                    );
                    assert!(
                        value
                            .pointer("/result/entity_id")
                            .and_then(|v| v.as_u64())
                            .unwrap_or(0)
                            > 0,
                        "entity_id for kind={kind}"
                    );
                    return;
                }
            }
            panic!("ws closed before spawn_entity response for kind={kind}");
        })
        .await
        .unwrap_or_else(|_| panic!("spawn_entity timeout for kind={kind}"));
    }
}

#[tokio::test]
async fn ws_jsonrpc_save_slot_roundtrip() {
    let saves_dir = tempfile::tempdir().expect("temp saves dir");
    let replays_dir = tempfile::tempdir().expect("temp replays dir");
    let sim = Arc::new(tokio::sync::Mutex::new(Simulation::with_seed(42)));
    {
        let mut guard = sim.lock().await;
        for _ in 0..3 {
            guard.tick();
        }
    }

    let addr = spawn_ws_bridge_with_config(
        sim.clone(),
        WsBridgeConfig {
            addr: SocketAddr::from(([127, 0, 0, 1], 0)),
            max_clients: 4,
            require_role: false,
            tick_broadcast_format: TickBroadcastFormat::Both,
            saves_dir: saves_dir.path().to_path_buf(),
            replays_dir: replays_dir.path().to_path_buf(),
        },
    )
    .await;
    let url = format!("ws://{addr}/ws");
    let (mut socket, _) = connect_async(&url).await.expect("ws connect");

    // Pause background ticks so save/load assertions stay deterministic.
    socket
        .send(Message::Text(
            r#"{"jsonrpc":"2.0","id":59,"method":"sim.set_speed","params":{"multiplier":0}}"#
                .into(),
        ))
        .await
        .expect("send sim.set_speed");
    timeout(Duration::from_secs(2), async {
        while let Some(frame) = socket.next().await {
            let Message::Text(text) = frame.expect("ws frame") else {
                continue;
            };
            let value: serde_json::Value = serde_json::from_str(&text).expect("json");
            if value.get("id") == Some(&serde_json::json!(59)) {
                return;
            }
        }
        panic!("ws closed before sim.set_speed response");
    })
    .await
    .expect("sim.set_speed timeout");

    let save_req = serde_json::json!({
        "jsonrpc": "2.0",
        "id": 60,
        "method": "save.slot",
        "params": { "slot_name": "slot-1" }
    });
    socket
        .send(Message::Text(save_req.to_string()))
        .await
        .expect("send save.slot");

    let save_response = timeout(Duration::from_secs(2), async {
        while let Some(frame) = socket.next().await {
            let Message::Text(text) = frame.expect("ws frame") else {
                continue;
            };
            let value: serde_json::Value = serde_json::from_str(&text).expect("json");
            if value.get("id") == Some(&serde_json::json!(60)) {
                return value;
            }
        }
        panic!("ws closed before save.slot response");
    })
    .await
    .expect("save.slot timeout");

    let saved_tick = save_response
        .pointer("/result/tick")
        .and_then(|v| v.as_u64())
        .expect("save tick");
    assert_eq!(
        save_response.pointer("/result/saved"),
        Some(&serde_json::json!(true))
    );
    assert_eq!(
        save_response.pointer("/result/slot_name"),
        Some(&serde_json::json!("slot-1"))
    );
    assert!(
        saves_dir.path().join("slot-1.civsave.zst").is_file(),
        "expected slot-1.civsave.zst on disk"
    );

    {
        let mut guard = sim.lock().await;
        for _ in 0..5 {
            guard.tick();
        }
    }
    assert!(sim.lock().await.state.tick > saved_tick);

    let load_req = serde_json::json!({
        "jsonrpc": "2.0",
        "id": 61,
        "method": "save.load",
        "params": { "slot_name": "slot-1" }
    });
    socket
        .send(Message::Text(load_req.to_string()))
        .await
        .expect("send save.load");

    let load_response = timeout(Duration::from_secs(2), async {
        while let Some(frame) = socket.next().await {
            let Message::Text(text) = frame.expect("ws frame") else {
                continue;
            };
            let value: serde_json::Value = serde_json::from_str(&text).expect("json");
            if value.get("id") == Some(&serde_json::json!(61)) {
                return value;
            }
        }
        panic!("ws closed before save.load response");
    })
    .await
    .expect("save.load timeout");

    assert_eq!(
        load_response.pointer("/result/loaded"),
        Some(&serde_json::json!(true))
    );
    assert_eq!(
        load_response
            .pointer("/result/tick")
            .and_then(|v| v.as_u64()),
        Some(saved_tick)
    );
    assert_eq!(sim.lock().await.state.tick, saved_tick);

    let list_req = serde_json::json!({
        "jsonrpc": "2.0",
        "id": 62,
        "method": "save.list",
        "params": {}
    });
    socket
        .send(Message::Text(list_req.to_string()))
        .await
        .expect("send save.list");

    let list_response = timeout(Duration::from_secs(2), async {
        while let Some(frame) = socket.next().await {
            let Message::Text(text) = frame.expect("ws frame") else {
                continue;
            };
            let value: serde_json::Value = serde_json::from_str(&text).expect("json");
            if value.get("id") == Some(&serde_json::json!(62)) {
                return value;
            }
        }
        panic!("ws closed before save.list response");
    })
    .await
    .expect("save.list timeout");

    let entries = list_response
        .pointer("/result")
        .and_then(|v| v.as_array())
        .expect("save.list array");
    assert_eq!(entries.len(), 1);
    assert_eq!(entries[0].get("name"), Some(&serde_json::json!("slot-1")));
    assert_eq!(
        entries[0].get("tick").and_then(|v| v.as_u64()),
        Some(saved_tick)
    );
    assert_eq!(
        entries[0].get("save_type"),
        Some(&serde_json::json!("slot"))
    );
}

/// Opt-in `sub_filter` query limits tick broadcasts to requested `Frame3d` kinds.
#[tokio::test]
async fn ws_sub_filter_query_limits_tick_broadcast_frames() {
    let sim = Arc::new(tokio::sync::Mutex::new(Simulation::with_seed(31)));
    let addr = spawn_ws_bridge_with_config(
        sim,
        WsBridgeConfig {
            addr: SocketAddr::from(([127, 0, 0, 1], 0)),
            max_clients: 2,
            require_role: false,
            tick_broadcast_format: TickBroadcastFormat::Binary,
            ..Default::default()
        },
    )
    .await;
    let url = format!("ws://{addr}/ws?sub_filter=climate");

    let (mut socket, _) = connect_async(&url).await.expect("ws connect");

    socket
        .send(Message::Text(
            r#"{"jsonrpc":"2.0","id":1,"method":"sim.set_speed","params":{"multiplier":0}}"#.into(),
        ))
        .await
        .expect("pause ticks");
    wait_for_jsonrpc_id(&mut socket, 1).await;

    socket
        .send(Message::Text(
            r#"{"jsonrpc":"2.0","id":2,"method":"sim.command","params":{"action":"tick"}}"#.into(),
        ))
        .await
        .expect("manual tick");

    let mut command_done = false;
    let mut climate_frames = 0usize;
    let mut other_frames = 0usize;

    timeout(Duration::from_secs(3), async {
        while !command_done || climate_frames == 0 {
            let frame = socket
                .next()
                .await
                .expect("ws stream open")
                .expect("ws frame");
            match frame {
                Message::Text(text) => {
                    if let Ok(value) = serde_json::from_str::<serde_json::Value>(&text) {
                        if value.get("id") == Some(&serde_json::json!(2)) {
                            command_done = true;
                        }
                    }
                }
                Message::Binary(bytes) if bytes.starts_with(FRAME3D_BINARY_MAGIC) => {
                    let decoded = decode_frame3d_binary(&bytes).expect("F3D0 frame");
                    if matches!(decoded, Frame3d::Climate(_)) {
                        climate_frames += 1;
                    } else {
                        other_frames += 1;
                    }
                }
                _ => {}
            }
        }
    })
    .await
    .expect("filtered tick broadcast timeout");

    assert!(command_done, "sim.command should complete");
    assert_eq!(climate_frames, 1, "expected one climate frame");
    assert_eq!(
        other_frames, 0,
        "filtered client should not receive other kinds"
    );
}

/// `sim.subscribe` over JSON-RPC applies the same per-connection frame filter.
#[tokio::test]
async fn ws_sim_subscribe_limits_tick_broadcast_frames() {
    let sim = Arc::new(tokio::sync::Mutex::new(Simulation::with_seed(32)));
    let addr = spawn_ws_bridge_with_config(
        sim,
        WsBridgeConfig {
            addr: SocketAddr::from(([127, 0, 0, 1], 0)),
            max_clients: 2,
            require_role: false,
            tick_broadcast_format: TickBroadcastFormat::Binary,
            ..Default::default()
        },
    )
    .await;
    let url = format!("ws://{addr}/ws");

    let (mut socket, _) = connect_async(&url).await.expect("ws connect");

    socket
        .send(Message::Text(
            r#"{"jsonrpc":"2.0","id":1,"method":"sim.subscribe","params":{"frame_kinds":["event_feed"]}}"#
                .into(),
        ))
        .await
        .expect("subscribe");
    wait_for_jsonrpc_id(&mut socket, 1).await;

    socket
        .send(Message::Text(
            r#"{"jsonrpc":"2.0","id":2,"method":"sim.set_speed","params":{"multiplier":0}}"#.into(),
        ))
        .await
        .expect("pause ticks");
    wait_for_jsonrpc_id(&mut socket, 2).await;

    socket
        .send(Message::Text(
            r#"{"jsonrpc":"2.0","id":3,"method":"sim.command","params":{"action":"tick"}}"#.into(),
        ))
        .await
        .expect("manual tick");

    let mut command_done = false;
    let mut event_frames = 0usize;

    timeout(Duration::from_secs(3), async {
        while !command_done || event_frames == 0 {
            let frame = socket
                .next()
                .await
                .expect("ws stream open")
                .expect("ws frame");
            match frame {
                Message::Text(text) => {
                    if let Ok(value) = serde_json::from_str::<serde_json::Value>(&text) {
                        if value.get("id") == Some(&serde_json::json!(3)) {
                            command_done = true;
                        }
                    }
                }
                Message::Binary(bytes) if bytes.starts_with(FRAME3D_BINARY_MAGIC) => {
                    let decoded = decode_frame3d_binary(&bytes).expect("F3D0 frame");
                    if matches!(decoded, Frame3d::EventFeed(_)) {
                        event_frames += 1;
                    } else {
                        panic!("unexpected filtered frame: {decoded:?}");
                    }
                }
                _ => {}
            }
        }
    })
    .await
    .expect("subscribe-filtered tick broadcast timeout");

    assert!(command_done, "sim.command should complete");
    assert_eq!(event_frames, 1);
}
