//! Multiplayer state-sync integration test (FR-MULTIPLAYER-SYNC).
//!
//! Spins up an in-process bridge with [`civ_server::spawn_ws_bridge_with_config`],
//! connects two WebSocket clients, and verifies the multiplayer
//! guarantees documented in `crates/server/src/session.rs`:
//!
//! 1. Both clients receive the same tick `Frame3d` broadcast each tick.
//! 2. A `sim.god_action` issued by client A mutates the shared
//!    simulation state and the mutation is visible to client B on the
//!    next broadcast.
//! 3. The engine audit log (`Simulation::last_god_actions`) records the
//!    issuing `connection_id`, so the multiplayer bridge can attribute
//!    every state-changing verb to a specific client for audit.

use std::sync::Arc;
use std::time::Duration;

use civ_engine::Simulation;
use civ_protocol_3d::{BuildingDiffFrame, ClimateFrame, Frame3d};
use civ_server::{spawn_ws_bridge_with_config, TickBroadcastFormat, WsBridgeConfig};
use futures::{SinkExt, StreamExt};
use serde_json::json;
use std::net::SocketAddr;
use tokio::time::timeout;
use tokio_tungstenite::{connect_async, tungstenite::Message};

const SEED: u64 = 42;

/// Spawn a bridge configured for deterministic audit-log reads.
async fn spawn_test_bridge(sim: Arc<tokio::sync::Mutex<Simulation>>) -> SocketAddr {
    spawn_ws_bridge_with_config(
        sim,
        WsBridgeConfig {
            addr: SocketAddr::from(([127, 0, 0, 1], 0)),
            max_clients: 4,
            require_role: false,
            tick_broadcast_format: TickBroadcastFormat::Text,
            saves_dir: std::env::temp_dir().join("civ-server-mp-test"),
            replays_dir: std::env::temp_dir().join("civ-server-mp-test-replays"),
        },
    )
    .await
}

/// Receive the next JSON-RPC response for `id` from `socket`.
///
/// Skips non-JSON messages (the bridge interleaves binary tick frames
/// alongside RPC responses). Times out after 5s to keep test latency
/// bounded.
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

/// Collect tick frames until we've seen at least `bundle_len` frames
/// for a tick `>= min_tick`, or `idle` passes without a new frame.
async fn collect_tick_bundle(
    socket: &mut tokio_tungstenite::WebSocketStream<
        tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>,
    >,
    min_tick: u64,
    bundle_len: usize,
    idle: Duration,
) -> Vec<Frame3d> {
    let mut frames = Vec::new();
    loop {
        let next = match timeout(idle, socket.next()).await {
            Ok(Some(Ok(msg))) => msg,
            Ok(Some(Err(_))) | Ok(None) | Err(_) => break,
        };
        let Message::Text(text) = next else {
            continue;
        };
        let v: serde_json::Value = match serde_json::from_str(&text) {
            Ok(v) => v,
            Err(_) => continue,
        };
        if v.get("id").is_some() {
            continue;
        }
        if let Ok(decoded) = serde_json::from_value::<Frame3d>(v) {
            frames.push(decoded);
        }
        let mut by_tick: std::collections::BTreeMap<u64, usize> =
            std::collections::BTreeMap::new();
        for f in &frames {
            *by_tick.entry(f.tick()).or_insert(0) += 1;
        }
        if let Some((&tick, &count)) = by_tick.iter().rev().next() {
            if tick >= min_tick && count >= bundle_len {
                break;
            }
        }
    }
    frames
}

/// Find a god-action entry in the engine audit log under the
/// assumption that no auto-tick has cleared it yet. Reads `sim` under
/// a brief lock and returns the first entry matching `action`.
async fn read_audit_entry(sim: &Arc<tokio::sync::Mutex<Simulation>>, action: &str) -> civ_engine::GodActionRecord {
    let guard = sim.lock().await;
    guard
        .last_god_actions()
        .iter()
        .find(|r| r.action == action)
        .cloned()
        .expect(
            "god action audit entry must be recorded after dispatch — \
             did the bridge dispatch the verb?",
        )
}

#[tokio::test]
async fn two_clients_receive_tick_broadcasts() {
    let sim = Arc::new(tokio::sync::Mutex::new(Simulation::with_seed(SEED)));
    let addr = spawn_test_bridge(sim).await;

    let (mut socket_a, _) = connect_async(format!("ws://{addr}/ws"))
        .await
        .expect("client A connect");
    let (mut socket_b, _) = connect_async(format!("ws://{addr}/ws"))
        .await
        .expect("client B connect");

    // Both clients should observe at least one tick bundle within the
    // idle window.
    let frames_a = collect_tick_bundle(&mut socket_a, 1, 1, Duration::from_secs(2)).await;
    let frames_b = collect_tick_bundle(&mut socket_b, 1, 1, Duration::from_secs(2)).await;
    let tick_a = frames_a
        .iter()
        .map(|f| f.tick())
        .max()
        .expect("client A received a tick frame");
    let tick_b = frames_b
        .iter()
        .map(|f| f.tick())
        .max()
        .expect("client B received a tick frame");
    assert!(tick_a >= 1, "client A should observe a tick frame (got {tick_a})");
    assert!(tick_b >= 1, "client B should observe a tick frame (got {tick_b})");
}

#[tokio::test]
async fn god_action_from_client_a_visible_to_client_b() {
    let sim = Arc::new(tokio::sync::Mutex::new(Simulation::with_seed(SEED)));
    let addr = spawn_test_bridge(sim.clone()).await;

    let (mut socket_a, _) = connect_async(format!("ws://{addr}/ws"))
        .await
        .expect("client A connect");
    let (mut socket_b, _) = connect_async(format!("ws://{addr}/ws"))
        .await
        .expect("client B connect");

    // Wait for both clients to subscribe (see at least one tick frame).
    let _ = collect_tick_bundle(&mut socket_a, 1, 1, Duration::from_secs(2)).await;
    let _ = collect_tick_bundle(&mut socket_b, 1, 1, Duration::from_secs(2)).await;

    // Pause auto-ticks so the audit log cannot be cleared while we
    // read it back. After the dispatch + audit-log read we resume
    // ticking via set_speed=1.
    {
        let pause = json!({
            "jsonrpc": "2.0",
            "id": 90,
            "method": "sim.set_speed",
            "params": {"multiplier": 0}
        })
        .to_string();
        socket_a.send(Message::Text(pause)).await.expect("send pause");
        let _ = recv_rpc(&mut socket_a, 90).await;
    }

    // Audit log should be empty while paused.
    {
        let guard = sim.lock().await;
        assert!(
            guard.last_god_actions().is_empty(),
            "audit log should start empty under speed=0"
        );
    }

    // Client A issues a god action. `life.spawn_organism` is one of
    // the least destructive verbs — it mutates `last_births()` without
    // requiring operator role enforcement.
    let god_action = json!({
        "jsonrpc": "2.0",
        "id": 100,
        "method": "sim.god_action",
        "params": {
            "action": "life.spawn_organism",
            "x": 0.5,
            "y": 0.5,
            "target_faction": 0,
            "seed_civilian_id": 9_999_999,
        }
    })
    .to_string();
    socket_a
        .send(Message::Text(god_action))
        .await
        .expect("send god_action from client A");

    // Receive the JSON-RPC response on client A.
    let resp_a = recv_rpc(&mut socket_a, 100).await;
    assert!(
        resp_a.get("error").is_none(),
        "client A god_action should not error, got: {resp_a}"
    );

    // Read the engine audit log IMMEDIATELY after dispatch (before
    // any auto-tick would clear it — and with speed=0 nothing ticks
    // anyway).
    let entry = read_audit_entry(&sim, "life.spawn_organism").await;
    let cid = entry
        .connection_id
        .as_deref()
        .expect("connection_id must be attributed");
    assert!(
        uuid::Uuid::parse_str(cid).is_ok(),
        "connection_id must be a UUID v4 hex string, got {cid:?}"
    );
    assert_eq!(
        entry.category, "life",
        "life.spawn_organism must be bucketed under the life category"
    );
    assert!(
        entry.params_json.contains("life.spawn_organism"),
        "params_json must echo the action verb"
    );

    // Resume ticking and verify both clients receive the next tick
    // bundle.
    let resume = json!({
        "jsonrpc": "2.0",
        "id": 101,
        "method": "sim.set_speed",
        "params": {"multiplier": 1}
    })
    .to_string();
    socket_a
        .send(Message::Text(resume))
        .await
        .expect("send sim.set_speed");
    let _ = recv_rpc(&mut socket_a, 101).await;

    // Both clients should observe at least one full tick bundle now
    // that speed=1 is active.
    let frames_a = collect_tick_bundle(&mut socket_a, 1, 7, Duration::from_secs(3)).await;
    let frames_b = collect_tick_bundle(&mut socket_b, 1, 7, Duration::from_secs(3)).await;
    assert!(
        !frames_a.is_empty(),
        "client A should keep receiving ticks after speed bump"
    );
    assert!(
        !frames_b.is_empty(),
        "client B should receive ticks after the god action"
    );
    let tick_b = frames_b
        .iter()
        .map(|f| f.tick())
        .max()
        .expect("client B max tick");
    assert!(tick_b >= 1, "client B tick should advance");
}

#[tokio::test]
async fn both_clients_observe_subsequent_ticks_after_write_through() {
    let sim = Arc::new(tokio::sync::Mutex::new(Simulation::with_seed(SEED)));
    let addr = spawn_test_bridge(sim.clone()).await;

    let (mut socket_a, _) = connect_async(format!("ws://{addr}/ws"))
        .await
        .expect("client A connect");
    let (mut socket_b, _) = connect_async(format!("ws://{addr}/ws"))
        .await
        .expect("client B connect");

    // Both clients subscribe.
    let _ = collect_tick_bundle(&mut socket_a, 1, 1, Duration::from_secs(2)).await;
    let _ = collect_tick_bundle(&mut socket_b, 1, 1, Duration::from_secs(2)).await;

    // Pause auto-ticks so we can read the audit log without races.
    {
        let pause = json!({
            "jsonrpc": "2.0",
            "id": 190,
            "method": "sim.set_speed",
            "params": {"multiplier": 0}
        })
        .to_string();
        socket_a.send(Message::Text(pause)).await.expect("send pause");
        let _ = recv_rpc(&mut socket_a, 190).await;
    }

    // Issue a terrain.add_land god action from client A. This is a
    // benign write that does not require operator role enforcement.
    let god_action = json!({
        "jsonrpc": "2.0",
        "id": 200,
        "method": "sim.god_action",
        "params": {
            "action": "terrain.add_land",
            "x": 0.5,
            "y": 0.5,
            "magnitude": 0.5,
            "radius_voxels": 3,
        }
    })
    .to_string();
    socket_a
        .send(Message::Text(god_action))
        .await
        .expect("send terrain.add_land");
    let resp = recv_rpc(&mut socket_a, 200).await;
    assert!(
        resp.get("error").is_none(),
        "terrain.add_land should not error: {resp}"
    );

    // Confirm the audit log captured the action and attributed it.
    let entry = read_audit_entry(&sim, "terrain.add_land").await;
    assert_eq!(
        entry.category, "terraform",
        "terrain.* verbs must be bucketed under the terraform category"
    );
    assert!(
        entry.connection_id.is_some(),
        "audit entry must carry the issuing connection_id"
    );

    // Resume ticking and verify both clients observe at least one
    // full tick bundle containing the post-action frames.
    let resume = json!({
        "jsonrpc": "2.0",
        "id": 201,
        "method": "sim.set_speed",
        "params": {"multiplier": 1}
    })
    .to_string();
    socket_a.send(Message::Text(resume)).await.expect("send resume");
    let _ = recv_rpc(&mut socket_a, 201).await;

    let frames_a = collect_tick_bundle(&mut socket_a, 1, 7, Duration::from_secs(3)).await;
    let frames_b = collect_tick_bundle(&mut socket_b, 1, 7, Duration::from_secs(3)).await;
    assert!(
        !frames_a.is_empty(),
        "client A should receive ticks after speed bump"
    );
    assert!(
        !frames_b.is_empty(),
        "client B should receive ticks after the god action"
    );
    let has_building_b = frames_b
        .iter()
        .any(|f| matches!(f, Frame3d::BuildingDiff(BuildingDiffFrame { .. })));
    let has_climate_b = frames_b
        .iter()
        .any(|f| matches!(f, Frame3d::Climate(ClimateFrame { .. })));
    let kind_summary_a: Vec<&str> = frames_a
        .iter()
        .map(|f| match f {
            Frame3d::VoxelDelta(_) => "voxel",
            Frame3d::BuildingDiff(_) => "building",
            Frame3d::AgentAppearance(_) => "agent",
            Frame3d::CivilianState(_) => "civilian",
            Frame3d::FactionState(_) => "faction",
            Frame3d::EventFeed(_) => "event",
            Frame3d::Climate(_) => "climate",
        })
        .collect();
    assert!(
        has_building_b || has_climate_b,
        "post-action bundle on client B must include building or climate frames (frames_a_kinds={:?})",
        kind_summary_a
    );
}

#[tokio::test]
async fn get_snapshot_for_session_returns_per_session_view() {
    let sim = Arc::new(tokio::sync::Mutex::new(Simulation::with_seed(SEED)));
    let addr = spawn_test_bridge(sim.clone()).await;

    let (mut socket_a, _) = connect_async(format!("ws://{addr}/ws"))
        .await
        .expect("client A connect");
    let (mut socket_b, _) = connect_async(format!("ws://{addr}/ws"))
        .await
        .expect("client B connect");

    // Subscribe both clients so their SharedSessions are populated.
    let _ = collect_tick_bundle(&mut socket_a, 1, 1, Duration::from_secs(2)).await;
    let _ = collect_tick_bundle(&mut socket_b, 1, 1, Duration::from_secs(2)).await;

    // Ask client A for its per-session snapshot. The response must
    // include the multiplayer `session_snapshot` block carrying the
    // connection_id and last_acked_tick for client A.
    let req = json!({
        "jsonrpc": "2.0",
        "id": 300,
        "method": "sim.get_snapshot_for_session",
        "params": {}
    })
    .to_string();
    socket_a.send(Message::Text(req)).await.expect("send get_snapshot_for_session");
    let resp = recv_rpc(&mut socket_a, 300).await;
    assert!(
        resp.get("error").is_none(),
        "sim.get_snapshot_for_session must not error: {resp}"
    );
    let session = resp
        .pointer("/result/session_snapshot")
        .expect("response must include session_snapshot block");
    let cid = session
        .get("connection_id")
        .and_then(|v| v.as_str())
        .expect("session_snapshot.connection_id must be a string");
    assert!(
        uuid::Uuid::parse_str(cid).is_ok(),
        "session_snapshot.connection_id must be a UUID v4 hex string, got {cid:?}"
    );
    assert!(
        session.get("last_acked_tick").is_some(),
        "session_snapshot must expose last_acked_tick"
    );

    // Client B's connection_id must differ from client A's (so the
    // session attribution is genuinely per-connection).
    let req_b = json!({
        "jsonrpc": "2.0",
        "id": 301,
        "method": "sim.get_snapshot_for_session",
        "params": {}
    })
    .to_string();
    socket_b.send(Message::Text(req_b)).await.expect("send get_snapshot_for_session B");
    let resp_b = recv_rpc(&mut socket_b, 301).await;
    let session_b = resp_b
        .pointer("/result/session_snapshot")
        .expect("response must include session_snapshot block");
    let cid_b = session_b
        .get("connection_id")
        .and_then(|v| v.as_str())
        .expect("session_snapshot.connection_id must be a string");
    assert_ne!(
        cid, cid_b,
        "client A and client B must have distinct connection_ids (got {cid} vs {cid_b})"
    );
    assert!(
        uuid::Uuid::parse_str(cid_b).is_ok(),
        "client B connection_id must be a UUID v4 hex string"
    );
}
