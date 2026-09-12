//! Building palette RPCs must mutate the authoritative ECS and publish world-space markers.
use civ_engine::{Building, BuildingType, Simulation};
use civ_protocol_3d::{decode_frame3d_binary, BuildingKind3d, Frame3d};
use civ_server::{spawn_ws_bridge_with_config, TickBroadcastFormat, WsBridgeConfig};
use futures::{SinkExt, StreamExt};
use serde_json::{json, Value};
use std::{sync::Arc, time::Duration};
use tokio::net::TcpStream;
use tokio_tungstenite::{connect_async, tungstenite::Message, MaybeTlsStream, WebSocketStream};

type Socket = WebSocketStream<MaybeTlsStream<TcpStream>>;

async fn rpc(
    socket: &mut Socket,
    frames: &mut Vec<Frame3d>,
    id: u64,
    method: &str,
    params: Value,
) -> Value {
    socket
        .send(Message::Text(
            json!({"jsonrpc":"2.0","id":id,"method":method,"params":params}).to_string(),
        ))
        .await
        .unwrap();
    tokio::time::timeout(Duration::from_secs(10), async {
        while let Some(message) = socket.next().await {
            match message.unwrap() {
                Message::Text(text) => {
                    let value: Value = serde_json::from_str(&text).unwrap();
                    if value.get("id") == Some(&json!(id)) {
                        return value;
                    }
                }
                Message::Binary(bytes) => frames.push(decode_frame3d_binary(&bytes).unwrap()),
                _ => {}
            }
        }
        panic!("socket closed before RPC response");
    })
    .await
    .expect("RPC deadline")
}

#[tokio::test]
async fn palette_aliases_spawn_authoritative_buildings_and_publish_world_coordinates() {
    let storage = tempfile::tempdir().unwrap();
    let sim = Arc::new(tokio::sync::Mutex::new(Simulation::with_seed(919)));
    let address = spawn_ws_bridge_with_config(
        sim.clone(),
        WsBridgeConfig {
            tick_broadcast_format: TickBroadcastFormat::Binary,
            saves_dir: storage.path().join("saves"),
            replays_dir: storage.path().join("replays"),
            ..Default::default()
        },
    )
    .await;
    let (mut socket, _) = connect_async(format!("ws://{address}/ws?tick_format=binary"))
        .await
        .unwrap();
    let mut frames = Vec::new();
    let paused = rpc(
        &mut socket,
        &mut frames,
        1,
        "sim.set_speed",
        json!({"multiplier":0,"role":"operator"}),
    )
    .await;
    assert!(paused.get("error").is_none(), "{paused}");
    let mut expected = Vec::new();
    for (index, (alias, engine_kind, wire_kind, x, z)) in [
        (
            "airport",
            BuildingType::CityCenter,
            BuildingKind3d::CityCenter,
            27.797_f32,
            -4.042_f32,
        ),
        (
            "port",
            BuildingType::Market,
            BuildingKind3d::Market,
            -64.0,
            64.0,
        ),
        (
            "hangar",
            BuildingType::Barracks,
            BuildingKind3d::Barracks,
            64.0,
            -64.0,
        ),
    ]
    .into_iter()
    .enumerate()
    {
        let before = sim.lock().await.world.query::<&Building>().iter().count();
        let nx = (x + 128.0) / 256.0;
        let nz = (z + 128.0) / 256.0;
        let response = rpc(
            &mut socket,
            &mut frames,
            index as u64 + 2,
            "sim.spawn_entity",
            json!({"kind":alias,"x":nx,"y":nz,"role":"operator"}),
        )
        .await;
        assert!(response.get("error").is_none(), "{response}");
        assert_eq!(response.pointer("/result/accepted"), Some(&json!(true)));
        assert_eq!(response.pointer("/result/ok"), Some(&json!(true)));
        assert_eq!(response.pointer("/result/kind"), Some(&json!(alias)));
        assert_eq!(
            response.pointer("/result/authoritative"),
            Some(&json!(true)),
            "building RPC must identify its authoritative publication"
        );
        assert_eq!(
            response.pointer("/result/scene_generation"),
            Some(&json!(0)),
            "building RPC must report the scene generation"
        );
        assert!(
            response
                .pointer("/result/tick")
                .and_then(Value::as_u64)
                .is_some(),
            "building RPC must report the publication tick"
        );
        assert!(
            response
                .pointer("/result/building_graph_version")
                .and_then(Value::as_u64)
                .is_some(),
            "building RPC must report the graph revision paired with its batch"
        );
        let receipt_id = response
            .pointer("/result/entity_id")
            .and_then(Value::as_u64)
            .expect("numeric entity receipt");
        let authoritative = sim.lock().await;
        assert_eq!(
            authoritative.world.query::<&Building>().iter().count(),
            before + 1
        );
        let (entity, building) = authoritative
            .world
            .query::<&Building>()
            .iter()
            .find(|(entity, _)| u64::from(entity.id()) == receipt_id)
            .map(|(entity, building)| (entity, *building))
            .expect("receipt identifies authoritative building");
        assert_eq!(building.building_type, engine_kind);
        assert_eq!(building.position.x, (nx * 127.0).round() as i32 - 64);
        assert_eq!(building.position.y, (nz * 127.0).round() as i32 - 64);
        expected.push((entity.to_bits().get(), wire_kind, x, z));
    }
    let tick = rpc(
        &mut socket,
        &mut frames,
        10,
        "sim.command",
        json!({"action":"tick","role":"operator"}),
    )
    .await;
    assert!(tick.get("error").is_none(), "{tick}");
    tokio::time::timeout(Duration::from_secs(10), async {
        loop {
            if let Some(frame) = frames.iter().find_map(|frame| match frame {
                Frame3d::BuildingDiff(frame)
                    if expected.iter().all(|(id, _, _, _)| {
                        frame.buildings.iter().any(|entry| entry.id == *id)
                    }) =>
                {
                    Some(frame)
                }
                _ => None,
            }) {
                for (id, kind, x, z) in &expected {
                    let entry = frame
                        .buildings
                        .iter()
                        .find(|entry| entry.id == *id)
                        .unwrap();
                    assert_eq!(&entry.kind, kind);
                    assert!((entry.position.x - x).abs() <= 128.0 / 127.0 + 0.0001);
                    assert!((entry.position.z - z).abs() <= 128.0 / 127.0 + 0.0001);
                }
                break;
            }
            if let Message::Binary(bytes) = socket.next().await.expect("open socket").unwrap() {
                frames.push(decode_frame3d_binary(&bytes).unwrap());
            }
        }
    })
    .await
    .expect("authoritative building broadcast deadline");

    // Invalid aliases and the legacy command must not create phantom buildings.
    let before = sim.lock().await.world.query::<&Building>().iter().count();
    for (id, method, params, code) in [
        (
            11,
            "sim.spawn_entity",
            json!({"kind":"city_center","x":0.5,"y":0.5,"role":"operator"}),
            -32602,
        ),
        (
            12,
            "sim.command",
            json!({"action":"spawn","kind":"city_center","role":"operator"}),
            -32601,
        ),
    ] {
        let response = rpc(&mut socket, &mut frames, id, method, params).await;
        assert_eq!(
            response.pointer("/error/code"),
            Some(&json!(code)),
            "{response}"
        );
        assert_eq!(
            sim.lock().await.world.query::<&Building>().iter().count(),
            before
        );
    }
    let (mut viewer, _) = connect_async(format!("ws://{address}/ws?tick_format=binary"))
        .await
        .unwrap();
    let forbidden = rpc(
        &mut viewer,
        &mut Vec::new(),
        20,
        "sim.spawn_entity",
        json!({"kind":"airport","x":0.5,"y":0.5}),
    )
    .await;
    assert_eq!(
        forbidden.pointer("/error/code"),
        Some(&json!(-32003)),
        "{forbidden}"
    );
    assert_eq!(
        sim.lock().await.world.query::<&Building>().iter().count(),
        before
    );
}

async fn receive_building(
    socket: &mut Socket,
    frames: &mut Vec<Frame3d>,
    id: u64,
) -> civ_protocol_3d::BuildingDiffFrame {
    tokio::time::timeout(Duration::from_secs(3), async {
        loop {
            if let Some(frame) = frames.iter().find_map(|frame| match frame {
                Frame3d::BuildingDiff(frame)
                    if frame.buildings.iter().any(|entry| entry.id == id) =>
                {
                    Some(frame.clone())
                }
                _ => None,
            }) {
                return frame;
            }
            match socket.next().await.expect("open socket").expect("WS frame") {
                Message::Binary(bytes) => frames.push(decode_frame3d_binary(&bytes).unwrap()),
                _ => {}
            }
        }
    })
    .await
    .expect("paused building must publish without advancing a tick")
}

#[tokio::test]
async fn paused_building_spawn_publishes_to_both_clients_without_ticking() {
    let storage = tempfile::tempdir().unwrap();
    // A nonzero fixture tick makes the observer cadence reject ordinary broadcasts.
    let mut initial = Simulation::with_seed(177);
    initial.tick();
    let sim = Arc::new(tokio::sync::Mutex::new(initial));
    let address = spawn_ws_bridge_with_config(
        sim.clone(),
        WsBridgeConfig {
            tick_broadcast_format: TickBroadcastFormat::Binary,
            saves_dir: storage.path().join("saves"),
            replays_dir: storage.path().join("replays"),
            ..Default::default()
        },
    )
    .await;
    let url = format!("ws://{address}/ws?tick_format=binary");
    let (mut operator, _) = connect_async(&url).await.unwrap();
    let mut operator_frames = Vec::new();
    let pause = rpc(
        &mut operator,
        &mut operator_frames,
        100,
        "sim.set_speed",
        json!({"multiplier":0,"role":"operator"}),
    )
    .await;
    assert_eq!(
        pause.pointer("/result/multiplier"),
        Some(&json!(0)),
        "{pause}"
    );
    let observer_url = format!("{url}&sub_filter=building_diff&tick_stride=1000");
    let (mut observer, _) = connect_async(&observer_url).await.unwrap();
    let mut observer_frames = Vec::new();
    let speed = rpc(
        &mut observer,
        &mut observer_frames,
        101,
        "sim.get_speed",
        json!({}),
    )
    .await;
    assert_eq!(
        speed.pointer("/result/multiplier"),
        Some(&json!(0)),
        "{speed}"
    );
    let paused_tick = sim.lock().await.state.tick;
    assert_ne!(
        paused_tick % 1000,
        0,
        "fixture must exercise cadence bypass"
    );
    let before = sim.lock().await.world.query::<&Building>().iter().count();
    operator_frames.clear();
    observer_frames.clear();

    let invalid = rpc(
        &mut operator,
        &mut operator_frames,
        102,
        "sim.spawn_entity",
        json!({"kind":"city_center","x":0.5,"y":0.5,"role":"operator"}),
    )
    .await;
    assert_eq!(
        invalid.pointer("/error/code"),
        Some(&json!(-32602)),
        "{invalid}"
    );
    let forbidden = rpc(
        &mut observer,
        &mut observer_frames,
        103,
        "sim.spawn_entity",
        json!({"kind":"airport","x":0.5,"y":0.5}),
    )
    .await;
    assert_eq!(
        forbidden.pointer("/error/code"),
        Some(&json!(-32003)),
        "{forbidden}"
    );
    assert_eq!(
        sim.lock().await.world.query::<&Building>().iter().count(),
        before
    );
    assert!(
        operator_frames
            .iter()
            .chain(&observer_frames)
            .all(|frame| !matches!(frame, Frame3d::BuildingDiff(_))),
        "rejected commands must not publish building changes"
    );

    // Observe a bounded quiet window on both sockets before a valid command.
    // This also catches a rejected command queued behind its RPC response.
    async fn assert_no_building_publication(socket: &mut Socket) {
        let observed = tokio::time::timeout(Duration::from_millis(200), async {
            loop {
                if let Message::Binary(bytes) = socket.next().await.expect("open socket").unwrap() {
                    assert!(
                        !matches!(
                            decode_frame3d_binary(&bytes).unwrap(),
                            Frame3d::BuildingDiff(_)
                        ),
                        "rejected spawn published a building frame"
                    );
                }
            }
        })
        .await;
        assert!(
            observed.is_err(),
            "quiet window must expire with an open socket"
        );
    }
    tokio::join!(
        assert_no_building_publication(&mut operator),
        assert_no_building_publication(&mut observer)
    );
    let spawn = rpc(
        &mut operator,
        &mut operator_frames,
        104,
        "sim.spawn_entity",
        json!({"kind":"airport","x":0.75,"y":0.25,"role":"operator"}),
    )
    .await;
    assert_eq!(
        spawn.pointer("/result/accepted"),
        Some(&json!(true)),
        "{spawn}"
    );
    assert_eq!(spawn.pointer("/result/ok"), Some(&json!(true)), "{spawn}");
    let receipt_id = spawn
        .pointer("/result/entity_id")
        .and_then(Value::as_u64)
        .unwrap();
    let stable_id = {
        let authoritative = sim.lock().await;
        assert_eq!(authoritative.state.tick, paused_tick);
        assert_eq!(
            authoritative.world.query::<&Building>().iter().count(),
            before + 1
        );
        let stable_id = authoritative
            .world
            .query::<&Building>()
            .iter()
            .find(|(entity, _)| u64::from(entity.id()) == receipt_id)
            .map(|(entity, building)| {
                assert_eq!(building.building_type, BuildingType::CityCenter);
                entity.to_bits().get()
            })
            .expect("authoritative receipt entity");
        stable_id
    };
    let (requester_frame, observer_frame) = tokio::join!(
        receive_building(&mut operator, &mut operator_frames, stable_id),
        receive_building(&mut observer, &mut observer_frames, stable_id),
    );
    for frame in [requester_frame, observer_frame] {
        assert_eq!(
            frame.tick, paused_tick,
            "publication must retain the paused tick"
        );
        let entry = frame
            .buildings
            .iter()
            .find(|entry| entry.id == stable_id)
            .unwrap();
        assert_eq!(entry.kind, BuildingKind3d::CityCenter);
        assert!((entry.position.x - 64.0).abs() <= 128.0 / 127.0 + 0.0001);
        assert!((entry.position.z + 64.0).abs() <= 128.0 / 127.0 + 0.0001);
    }
    let speed = rpc(
        &mut operator,
        &mut operator_frames,
        105,
        "sim.get_speed",
        json!({}),
    )
    .await;
    assert_eq!(
        speed.pointer("/result/multiplier"),
        Some(&json!(0)),
        "placement must not resume: {speed}"
    );
    assert_eq!(
        sim.lock().await.state.tick,
        paused_tick,
        "placement must not tick"
    );
}

fn building_state(sim: &Simulation) -> Vec<(u64, Building)> {
    let mut buildings: Vec<_> = sim
        .world
        .query::<&Building>()
        .iter()
        .map(|(entity, building)| (entity.to_bits().get(), *building))
        .collect();
    buildings.sort_by_key(|(id, _)| *id);
    buildings
}
#[tokio::test]
async fn building_palette_and_interleaved_terrain_survive_replay_without_duplication() {
    use civ_engine::{decode_civreplay, encode_civreplay};
    use civ_voxel::{
        material::{STONE, WOOD},
        WorldCoord,
    };

    let storage = tempfile::tempdir().unwrap();
    let sim = Arc::new(tokio::sync::Mutex::new(Simulation::with_seed(419)));
    let address = spawn_ws_bridge_with_config(
        sim.clone(),
        WsBridgeConfig {
            tick_broadcast_format: TickBroadcastFormat::Binary,
            saves_dir: storage.path().join("saves"),
            replays_dir: storage.path().join("replays"),
            ..Default::default()
        },
    )
    .await;
    let (mut socket, _) = connect_async(format!("ws://{address}/ws?tick_format=binary"))
        .await
        .unwrap();
    let mut frames = Vec::new();
    let paused = rpc(
        &mut socket,
        &mut frames,
        200,
        "sim.set_speed",
        json!({"multiplier":0,"role":"operator"}),
    )
    .await;
    assert_eq!(
        paused.pointer("/result/multiplier"),
        Some(&json!(0)),
        "{paused}"
    );
    let baseline_count = building_state(&*sim.lock().await).len();
    let initial_event_count = sim.lock().await.replay_log().events.len();
    let point = WorldCoord { x: 0, y: 0, z: 0 };
    for (index, (alias, x, y, material)) in [
        ("airport", 0.75, 0.25, WOOD),
        ("port", 0.25, 0.75, STONE),
        ("hangar", 0.625, 0.375, WOOD),
    ]
    .into_iter()
    .enumerate()
    {
        let terrain = rpc(
            &mut socket,
            &mut frames,
            201 + index as u64 * 2,
            "sim.place_voxel",
            json!({"x":point.x,"y":point.y,"z":point.z,"material":material.0,"role":"operator"}),
        )
        .await;
        assert_eq!(
            terrain.pointer("/result/ok"),
            Some(&json!(true)),
            "{terrain}"
        );
        let spawn = rpc(
            &mut socket,
            &mut frames,
            202 + index as u64 * 2,
            "sim.spawn_entity",
            json!({"kind":alias,"x":x,"y":y,"role":"operator"}),
        )
        .await;
        assert_eq!(spawn.pointer("/result/ok"), Some(&json!(true)), "{spawn}");
    }
    let (expected, expected_tick, source_log) = {
        let authoritative = sim.lock().await;
        assert_eq!(authoritative.voxel().read(point), WOOD);
        (
            building_state(&authoritative),
            authoritative.state.tick,
            authoritative.replay_log().clone(),
        )
    };
    assert_eq!(expected.len(), baseline_count + 3);
    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(10))
        .build()
        .unwrap();
    let response = client
        .get(format!("http://{address}/replay/export"))
        .send()
        .await
        .unwrap();
    assert!(response.status().is_success());
    let bytes = response.bytes().await.unwrap();
    let log = decode_civreplay(&bytes).unwrap();
    assert_eq!(
        log, source_log,
        "export preserves the authoritative event order"
    );
    assert_eq!(encode_civreplay(&log).unwrap(), bytes.as_ref());

    assert_eq!(log.schema_version, 2);
    let order: Vec<_> = log.events[initial_event_count..]
        .iter()
        .map(|event| match event {
            civ_engine::replay::ReplayEvent::VoxelWrite { .. } => "terrain",
            civ_engine::replay::ReplayEvent::BuildingSpawn { .. } => "building",
            _ => "unexpected",
        })
        .collect();
    assert_eq!(
        order,
        ["terrain", "building", "terrain", "building", "terrain", "building"]
    );
    let mut replayed = Simulation::with_seed(log.seed);
    let initial_events = replayed.replay_log().events.len();
    log.replay(&mut replayed).unwrap();
    assert_eq!(
        building_state(&replayed),
        expected,
        "replay must retain each building type, grid position, hp and max_hp"
    );
    assert_eq!(
        replayed.voxel().read(point),
        WOOD,
        "interleaved voxel writes retain their order"
    );
    assert_eq!(replayed.state.tick, expected_tick);
    assert_eq!(
        replayed.replay_log().events.len(),
        initial_events,
        "playback must not record the authoring events again"
    );

    let first_path = storage.path().join("palette.civreplay");
    std::fs::write(&first_path, &bytes).unwrap();
    let loaded = Simulation::load_replay_from_file(&first_path).unwrap();
    assert_eq!(building_state(&loaded), expected);
    assert_eq!(loaded.voxel().read(point), WOOD);
    assert_eq!(
        loaded.replay_log(),
        &log,
        "file loading retains one copy of each event"
    );
    let second_path = storage.path().join("palette-resaved.civreplay");
    loaded.save_replay(&second_path).unwrap();
    let reloaded = Simulation::load_replay_from_file(&second_path).unwrap();
    assert_eq!(
        building_state(&reloaded),
        expected,
        "save/reload must not duplicate authored buildings"
    );
    assert_eq!(reloaded.voxel().read(point), WOOD);
    assert_eq!(reloaded.replay_log(), &log);

    // Exercise the active production archive path, then author after restoring.
    let save = rpc(
        &mut socket,
        &mut frames,
        220,
        "save.slot",
        json!({"slot_name":"slot-1","role":"operator"}),
    )
    .await;
    assert!(save.get("error").is_none(), "{save}");
    let reset = rpc(
        &mut socket,
        &mut frames,
        221,
        "sim.reset",
        json!({"seed":999,"role":"operator"}),
    )
    .await;
    assert!(reset.get("error").is_none(), "{reset}");
    let load = rpc(
        &mut socket,
        &mut frames,
        222,
        "save.load",
        json!({"slot_name":"slot-1","role":"operator"}),
    )
    .await;
    assert!(load.get("error").is_none(), "{load}");
    assert_eq!(
        building_state(&*sim.lock().await),
        expected,
        "active archive must retain full entity bits and building state"
    );
    assert_eq!(sim.lock().await.voxel().read(point), WOOD);
    let next = rpc(
        &mut socket,
        &mut frames,
        223,
        "sim.spawn_entity",
        json!({"kind":"airport","x":0.125,"y":0.875,"role":"operator"}),
    )
    .await;
    assert_eq!(next.pointer("/result/ok"), Some(&json!(true)), "{next}");
    let continued = building_state(&*sim.lock().await);
    assert_eq!(
        continued.len(),
        expected.len() + 1,
        "post-load authoring adds exactly one building"
    );
    let save = rpc(
        &mut socket,
        &mut frames,
        224,
        "save.slot",
        json!({"slot_name":"slot-2","role":"operator"}),
    )
    .await;
    assert!(save.get("error").is_none(), "{save}");
    let load = rpc(
        &mut socket,
        &mut frames,
        225,
        "save.load",
        json!({"slot_name":"slot-2","role":"operator"}),
    )
    .await;
    assert!(load.get("error").is_none(), "{load}");
    assert_eq!(
        building_state(&*sim.lock().await),
        continued,
        "resaving after new authoring must not collide or duplicate IDs"
    );
    let response = client
        .get(format!("http://{address}/replay/export"))
        .send()
        .await
        .unwrap();
    assert!(response.status().is_success());
    let continued_log = decode_civreplay(&response.bytes().await.unwrap()).unwrap();
    assert_eq!(
        continued_log
            .events
            .iter()
            .filter(|event| matches!(event, civ_engine::replay::ReplayEvent::BuildingSpawn { .. }))
            .count(),
        4
    );
    let mut continued_replay = Simulation::with_seed(continued_log.seed);
    continued_log.replay(&mut continued_replay).unwrap();
    assert_eq!(building_state(&continued_replay), continued);
    assert_eq!(continued_replay.voxel().read(point), WOOD);
}
