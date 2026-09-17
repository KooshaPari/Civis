//! Traceability test for FR-CIV-GODOT-ATTACH-000 — default attach to
//! `civ-server` over WebSocket JSON-RPC.
//!
//! Requirement (`docs/development-guide/fr-godot-attach.md:8`):
//!
//! > FR-CIV-GODOT-ATTACH-000 — Default attach `civ-server` WebSocket JSON-RPC.
//! > Acceptance: `attach_mode=server`, `CivisWsClient` connects, `health` +
//! > `sim.snapshot` succeed.
//!
//! The GDScript `CivisWsClient` (`clients/godot-ref/scripts/civis_ws_client.gd`)
//! sends `health` and `sim.snapshot` on open and routes every incoming packet
//! through the Rust GDExtension decoder `CivisWsFrame.decode_ws_packet`, backed
//! by [`civis_godot_rust::ws_frame`] — "WebSocket payload decode for civ-server
//! attach (F3D0 + JSON-RPC fallback)". This test carries the FR id in its name
//! (the audit could not see the generically named inline `ws_frame` tests) and
//! asserts that the decoder the live attach path depends on handles the
//! JSON-RPC envelope used by `health` + `sim.snapshot`, the binary `tick_format`
//! frames, and malformed input without panicking.

use civis_godot_rust::ws_frame::{decode_ws_packet_bytes, DecodedWsPacket};
use civ_protocol_3d::{encode_frame3d_binary, BuildingDiffFrame, BuildingProvenance, Frame3d};

/// FR-CIV-GODOT-ATTACH-000 — a `health` JSON-RPC response from `civ-server`
/// decodes as `RpcJson` and is preserved verbatim for GDScript `JSON.parse_string`.
#[test]
fn fr_civ_godot_attach_000_health_jsonrpc_response_decodes() {
    // Shape emitted by civ-server on `{"jsonrpc":"2.0","id":1,"method":"health"}`.
    let health = r#"{"jsonrpc":"2.0","id":1,"result":{"status":"ok","version":"0.1.0"}}"#;

    let DecodedWsPacket { kind, json, .. } =
        decode_ws_packet_bytes(health.as_bytes()).expect("health response must decode");

    assert_eq!(kind, "RpcJson", "JSON-RPC envelope must be classified RpcJson");
    assert_eq!(json, health, "payload must be forwarded verbatim to GDScript");
    // `health` carries no tick; the decoder must default it rather than fail.
    let tick = decode_ws_packet_bytes(health.as_bytes()).unwrap().tick;
    assert_eq!(tick, 0);
}

/// FR-CIV-GODOT-ATTACH-000 — a `sim.snapshot` JSON-RPC response decodes and its
/// `result.tick` is surfaced so the client can order snapshots.
#[test]
fn fr_civ_godot_attach_000_sim_snapshot_jsonrpc_response_decodes() {
    let snapshot = r#"{"jsonrpc":"2.0","id":2,"result":{"tick":144,"population":37,"is_day":true}}"#;

    let decoded = decode_ws_packet_bytes(snapshot.as_bytes()).expect("snapshot must decode");
    assert_eq!(decoded.kind, "RpcJson");
    assert_eq!(decoded.tick, 144, "snapshot tick must come from result.tick");
    assert_eq!(decoded.json, snapshot);
}

/// FR-CIV-GODOT-ATTACH-000 — the default `tick_format=binary` attach delivers
/// `F3D0` frames; the decoder must recognise the binary magic and report kind/tick.
#[test]
fn fr_civ_godot_attach_000_binary_f3d0_frame_decodes() {
    let frame = Frame3d::BuildingDiff(BuildingDiffFrame {
        tick: 7_777,
        provenance: BuildingProvenance::Procedural,
        buildings: vec![],
        graph: None,
    });
    let bytes = encode_frame3d_binary(&frame).expect("encode F3D0 fixture");

    let decoded = decode_ws_packet_bytes(&bytes).expect("F3D0 frame must decode");
    assert_eq!(decoded.kind, "BuildingDiff");
    assert_eq!(decoded.tick, 7_777);
}

/// FR-CIV-GODOT-ATTACH-000 — a versioned text `Frame3d` payload (non-binary
/// `tick_format`) also decodes, so attach works with either wire format.
#[test]
fn fr_civ_godot_attach_000_text_frame3d_fallback_decodes() {
    let json = r#"{"VoxelDelta":{"tick":9,"deltas":[]}}"#;
    let decoded = decode_ws_packet_bytes(json.as_bytes()).expect("text Frame3d must decode");
    assert_eq!(decoded.kind, "VoxelDelta");
    assert_eq!(decoded.tick, 9);
}

/// FR-CIV-GODOT-ATTACH-000 — the decoder is total: a frame `CivisWsClient`
/// cannot interpret (e.g. a truncated/garbage packet) yields `Err`, which the
/// GDScript side turns into `{ok: false}` rather than crashing the client.
#[test]
fn fr_civ_godot_attach_000_undecodable_packet_errors_without_panic() {
    assert!(decode_ws_packet_bytes(b"").is_err(), "empty packet must not decode");
    assert!(
        decode_ws_packet_bytes(b"not-json-not-f3d0").is_err(),
        "garbage packet must not decode"
    );
    // Binary magic present but the body is truncated -> decode error, no panic.
    assert!(decode_ws_packet_bytes(b"F3D0").is_err());
}
