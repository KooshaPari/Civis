//! WebSocket payload decode for civ-server attach (F3D0 + JSON-RPC fallback).

use crate::f3d0_mesh::{mesh_chunk_from_material_ids, CHUNK_VOXELS};
use civ_protocol_3d::{decode_frame3d_binary, is_frame3d_binary, Frame3d, FRAME3D_BINARY_MAGIC};
use godot::prelude::*;
use serde_json::Value;

/// Decoded WS packet for GDScript (`CivisWsFrame.decode_ws_packet`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DecodedWsPacket {
    /// `VoxelDelta` | `BuildingDiff` | `AgentAppearance` | `RpcJson`
    pub kind: String,
    pub tick: u64,
    /// JSON text for `JSON.parse_string` in GDScript.
    pub json: String,
}

fn frame_kind_tick_json(frame: &Frame3d) -> DecodedWsPacket {
    // Frame3d gained four additional variants (CivilianState, FactionState,
    // EventFeed, Climate) on the wave-1 branch. The Godot client only renders
    // the three voxel/building/agent variants as ECS entities, so for the
    // GDScript surface we report any of the others as a generic "Other" kind
    // and let GDScript ignore them — keeping the DecodedWsPacket contract
    // stable while the protocol grows.
    let kind = match frame {
        Frame3d::VoxelDelta(_) => "VoxelDelta",
        Frame3d::BuildingDiff(_) => "BuildingDiff",
        Frame3d::AgentAppearance(_) => "AgentAppearance",
        Frame3d::CivilianState(_)
        | Frame3d::FactionState(_)
        | Frame3d::EventFeed(_)
        | Frame3d::Climate(_) => "Other",
    };
    let tick = frame.tick();
    let json = serde_json::to_string(frame).unwrap_or_default();
    DecodedWsPacket {
        kind: kind.into(),
        tick,
        json,
    }
}

fn tick_from_jsonrpc(value: &Value) -> u64 {
    value
        .get("result")
        .and_then(|r| r.get("tick"))
        .and_then(|t| t.as_u64())
        .unwrap_or(0)
}

fn try_parse_jsonrpc(bytes: &[u8]) -> Option<DecodedWsPacket> {
    let text = std::str::from_utf8(bytes).ok()?;
    let value: Value = serde_json::from_str(text).ok()?;
    if value.get("jsonrpc").is_none() {
        return None;
    }
    Some(DecodedWsPacket {
        kind: "RpcJson".into(),
        tick: tick_from_jsonrpc(&value),
        json: text.to_string(),
    })
}

fn try_parse_frame3d_json(bytes: &[u8]) -> Result<DecodedWsPacket, String> {
    let text = std::str::from_utf8(bytes).map_err(|err| err.to_string())?;
    let frame: Frame3d = serde_json::from_str(text).map_err(|err| err.to_string())?;
    Ok(frame_kind_tick_json(&frame))
}

/// Decode a WebSocket payload: F3D0 binary first, then UTF-8 JSON (JSON-RPC or `Frame3d`).
pub fn decode_ws_packet_bytes(bytes: &[u8]) -> Result<DecodedWsPacket, String> {
    if is_frame3d_binary(bytes) {
        let frame = decode_frame3d_binary(bytes).map_err(|err| format!("{err:?}"))?;
        return Ok(frame_kind_tick_json(&frame));
    }
    if let Some(rpc) = try_parse_jsonrpc(bytes) {
        return Ok(rpc);
    }
    try_parse_frame3d_json(bytes)
}

pub fn decoded_to_dictionary(packet: DecodedWsPacket) -> VarDictionary {
    let mut dict = VarDictionary::new();
    dict.set("ok", true);
    dict.set("kind", GString::from(&packet.kind));
    dict.set("tick", packet.tick as i64);
    dict.set("json", GString::from(&packet.json));
    dict
}

pub fn error_to_dictionary(err: String) -> VarDictionary {
    let mut dict = VarDictionary::new();
    dict.set("ok", false);
    dict.set("error", GString::from(&err));
    dict
}

#[derive(GodotClass)]
#[class(base = RefCounted)]
pub struct CivisWsFrame {
    base: Base<RefCounted>,
}

#[godot_api]
impl CivisWsFrame {
    /// Decode civ-server WebSocket bytes: `F3D0` binary `Frame3d`, else JSON-RPC or text `Frame3d`.
    ///
    /// Returns `{ ok, kind, tick, json }` on success; `{ ok: false, error }` on failure.
    #[func]
    fn decode_ws_packet(bytes: PackedByteArray) -> VarDictionary {
        let data = bytes.to_vec();
        match decode_ws_packet_bytes(&data) {
            Ok(packet) => decoded_to_dictionary(packet),
            Err(err) => error_to_dictionary(err),
        }
    }

    #[func]
    fn mesh_voxel_chunk(material_ids: PackedInt32Array) -> VarDictionary {
        if material_ids.len() as usize != CHUNK_VOXELS {
            return error_to_dictionary(format!(
                "expected {CHUNK_VOXELS} material ids, got {}",
                material_ids.len()
            ));
        }
        let raw: Vec<u32> = (0..material_ids.len())
            .map(|i| material_ids.get(i).unwrap_or(0).max(0) as u32)
            .collect();
        let mesh = mesh_chunk_from_material_ids(&raw);
        if mesh.is_empty() {
            return error_to_dictionary("empty chunk mesh".into());
        }
        let mut dict = VarDictionary::new();
        dict.set("ok", true);
        dict.set("vertices", packed_vec3_from_flat(&mesh.vertices));
        dict.set("normals", packed_vec3_from_flat(&mesh.normals));
        let mut indices = PackedInt32Array::new();
        for index in mesh.indices {
            indices.push(index);
        }
        dict.set("indices", indices);
        dict
    }
}

fn packed_vec3_from_flat(flat: &[f32]) -> PackedVector3Array {
    let mut array = PackedVector3Array::new();
    let mut i = 0usize;
    while i + 2 < flat.len() {
        array.push(Vector3::new(flat[i], flat[i + 1], flat[i + 2]));
        i += 3;
    }
    array
}

#[godot_api]
impl IRefCounted for CivisWsFrame {
    fn init(base: Base<RefCounted>) -> Self {
        Self { base }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use civ_protocol_3d::{
        encode_frame3d_binary, BuildingDiffFrame, BuildingProvenance, Frame3d,
    };

    /// FR-CIV-GODOT-F3D0 — minimal `F3D0` fixture round-trips through decode (mirrors protocol-3d tests).
    #[test]
    fn decode_ws_packet_f3d0_building_diff_fixture() {
        let frame = Frame3d::BuildingDiff(BuildingDiffFrame {
            tick: 42,
            provenance: BuildingProvenance::Procedural,
            buildings: Vec::new(),
            graph: None,
        });
        let bytes = encode_frame3d_binary(&frame).expect("encode fixture");
        assert!(bytes.starts_with(FRAME3D_BINARY_MAGIC));

        let decoded = decode_ws_packet_bytes(&bytes).expect("decode F3D0");
        assert_eq!(decoded.kind, "BuildingDiff");
        assert_eq!(decoded.tick, 42);
        assert!(decoded.json.contains("BuildingDiff"));
        assert!(decoded.json.contains("Procedural"));
    }

    #[test]
    fn decode_ws_packet_jsonrpc_response() {
        let text = r#"{"jsonrpc":"2.0","id":1,"result":{"tick":12,"population":4}}"#;
        let decoded = decode_ws_packet_bytes(text.as_bytes()).expect("rpc");
        assert_eq!(decoded.kind, "RpcJson");
        assert_eq!(decoded.tick, 12);
        assert_eq!(decoded.json, text);
    }

    #[test]
    fn decode_ws_packet_text_frame3d_voxel_delta() {
        let json = r#"{"VoxelDelta":{"tick":7,"deltas":[]}}"#;
        let decoded = decode_ws_packet_bytes(json.as_bytes()).expect("text frame");
        assert_eq!(decoded.kind, "VoxelDelta");
        assert_eq!(decoded.tick, 7);
    }
}
