# Bevy Marker Ownership Policy (Task #8)

## Policy decision
- Server-attach mode owns **server-authored stream markers**: `LiveAgentTag`,
  `LiveBuildingTag`, `LiveGraphParcelTag`, and `LiveChunkTag`.
  These components are only for entities synchronized from `Frame3d` payloads via
  `WsClient` in `live_attach`/`live_scene`.
- In-process mode owns **local simulation markers**: `SimCivilianMarkerPublic` and
  `SimBuildingMarkerPublic`.
  These components are only for entities generated from `SimState` in
  `SimBridgePlugin`.

## Separation rules
1. Do not reuse stream markers in in-process mode, or local-sim markers in
   server-attach mode.
2. Keep marker lifecycles scoped to attach mode:
   - server-attach paths are gated by `if *attach == AttachMode::Server`.
   - in-process systems are gated by `!is_server_attach_mode(*mode)`.
3. `scene_dump.rs` should keep both marker queries because the dump remains valid
   in either mode.

## Enforcement
- This file’s mode boundary is now documented in marker definitions (`live_stream`
  and `sim_bridge`).
- `clients/bevy-ref/tests/requirements_bdd.rs` contains
  `requirement_marker_types_differentiate_server_attach_vs_in_process`, which
  asserts the marker component types remain distinct across attach modes.
