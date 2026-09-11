//! Authoritative terrain edits must survive the server's binary chunk payload.
use std::sync::Arc;
use std::time::Duration;

use civ_engine::Simulation;
use civ_protocol_3d::{decode_frame3d_binary, encode_frame3d_binary, Frame3d};
use civ_server::{spawn_ws_bridge, voxel_frame_builder::build_voxel_delta_frame};
use civ_voxel::{
    material::{STONE, WOOD},
    MaterialId, VoxelWorld, WorldCoord, FIXED_SCALE,
};
use futures::{SinkExt, StreamExt};
use serde_json::{json, Value};
use tokio::net::TcpStream;
use tokio_tungstenite::{connect_async, tungstenite::Message, MaybeTlsStream, WebSocketStream};

fn assert_material_roundtrip(frame: Frame3d, index: usize, expected: MaterialId) {
    let encoded = encode_frame3d_binary(&frame).expect("encode authoritative frame");
    let Frame3d::VoxelDelta(decoded) = decode_frame3d_binary(&encoded).expect("decode frame")
    else {
        panic!("expected voxel frame");
    };
    let chunk = decoded
        .deltas
        .iter()
        .find(|delta| delta.event.chunk_id == civ_voxel::ChunkId(0))
        .expect("edited origin chunk");
    assert_eq!(chunk.voxels.len(), 16 * 16 * 16);
    assert_eq!(chunk.voxels[index], expected);
}

#[test]
fn dense_chunk_binary_roundtrip_preserves_material_and_axis_order() {
    let mut world: VoxelWorld<MaterialId> = VoxelWorld::new(FIXED_SCALE);
    world.write(
        WorldCoord {
            x: 2 * FIXED_SCALE,
            y: 4 * FIXED_SCALE,
            z: 6 * FIXED_SCALE,
        },
        STONE,
    );
    let events = world.drain_dirty();
    let frame = build_voxel_delta_frame(7, &events, &world).expect("dense frame");
    assert_material_roundtrip(Frame3d::VoxelDelta(frame), 2 + 4 * 16 + 6 * 256, STONE);
}

async fn rpc(
    socket: &mut WebSocketStream<MaybeTlsStream<TcpStream>>,
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
        .expect("send RPC");
    tokio::time::timeout(Duration::from_secs(5), async {
        while let Some(message) = socket.next().await {
            match message.expect("read RPC frame") {
                Message::Text(text) => {
                    let value: Value = serde_json::from_str(&text).expect("JSON frame");
                    if value.get("id") == Some(&json!(id)) {
                        assert!(value.get("error").is_none(), "RPC failed: {value}");
                        return value;
                    }
                }
                Message::Binary(bytes) => {
                    frames.push(decode_frame3d_binary(&bytes).expect("decode server binary frame"))
                }
                _ => {}
            }
        }
        panic!("socket closed before RPC response");
    })
    .await
    .expect("RPC response deadline")
}

#[tokio::test]
async fn ws_terraform_writes_material_visible_in_binary_chunk() {
    let sim = Arc::new(tokio::sync::Mutex::new(Simulation::with_seed(919)));
    let address = spawn_ws_bridge(sim.clone(), 4).await;
    let (mut socket, _) = connect_async(format!("ws://{address}/ws?tick_format=binary"))
        .await
        .expect("connect");
    let mut frames = Vec::new();
    rpc(
        &mut socket,
        &mut frames,
        101,
        "sim.set_speed",
        json!({"multiplier":0}),
    )
    .await;
    let pos = WorldCoord {
        x: 5 * FIXED_SCALE,
        y: 4 * FIXED_SCALE,
        z: 6 * FIXED_SCALE,
    };
    let edge = WorldCoord {
        x: pos.x + 2 * FIXED_SCALE,
        ..pos
    };
    let outside = WorldCoord {
        x: pos.x + 3 * FIXED_SCALE,
        ..pos
    };
    {
        let mut authoritative = sim.lock().await;
        for point in [pos, edge, outside] {
            authoritative.voxel_mut().write(point, MaterialId(0));
            assert_eq!(authoritative.voxel().read(point), MaterialId(0));
        }
    }
    let response = rpc(
        &mut socket,
        &mut frames,
        102,
        "sim.terraform_extent",
        json!({
            "x":pos.x,"y":pos.y,"z":pos.z,"op":"raise","radius":2,"material":WOOD.0
        }),
    )
    .await;
    assert_eq!(response.pointer("/result/ok"), Some(&json!(true)));
    assert_eq!(response.pointer("/result/writes"), Some(&json!(13)));
    let wire_frame = tokio::time::timeout(Duration::from_secs(5), async {
        loop {
            if let Some(frame) = frames.iter().find_map(|frame| match frame {
                Frame3d::VoxelDelta(frame)
                    if frame.deltas.iter().any(|delta| {
                        delta.event.chunk_id == civ_voxel::ChunkId(0)
                            && delta.voxels.get(5 + 4 * 16 + 6 * 256) == Some(&WOOD)
                    }) =>
                {
                    Some(frame.clone())
                }
                _ => None,
            }) {
                return frame;
            }
            if let Message::Binary(bytes) =
                socket.next().await.expect("open socket").expect("WS frame")
            {
                frames.push(
                    decode_frame3d_binary(&bytes)
                        .expect("decode authoritative server binary frame"),
                );
            }
        }
    })
    .await
    .expect("authoritative voxel delta deadline");
    assert_material_roundtrip(Frame3d::VoxelDelta(wire_frame), 5 + 4 * 16 + 6 * 256, WOOD);
    let mut authoritative = sim.lock().await;
    assert_eq!(authoritative.voxel().read(pos), WOOD);
    assert_eq!(authoritative.voxel().read(edge), WOOD);
    assert_eq!(authoritative.voxel().read(outside), MaterialId(0));
    authoritative.tick();
    let frame = build_voxel_delta_frame(
        authoritative.state.tick,
        authoritative.last_tick_voxel_events(),
        authoritative.voxel(),
    )
    .expect("post-edit frame");
    assert_material_roundtrip(
        Frame3d::VoxelDelta(frame.clone()),
        5 + 4 * 16 + 6 * 256,
        WOOD,
    );
    assert_material_roundtrip(
        Frame3d::VoxelDelta(frame.clone()),
        7 + 4 * 16 + 6 * 256,
        WOOD,
    );
    assert_material_roundtrip(
        Frame3d::VoxelDelta(frame),
        8 + 4 * 16 + 6 * 256,
        MaterialId(0),
    );
}
