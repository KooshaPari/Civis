# Wrap-Over-Handroll Audit

Scope: `crates/` and `clients/`

Goal: find places where Civis is hand-rolling functionality that a mature crate already covers better, and classify whether the current code is acceptable, replaceable, or worth leaving alone.

## Summary

The strongest wrap-over-hand-roll candidates are:

1. Terrain noise generation in `crates/watch` and mirrored client terrain code.
2. Custom pathfinding in `crates/tactics`.
3. Custom replay and frame container formats in `crates/engine` and `crates/protocol-3d`.
4. HTTP/WebSocket/SSE glue in `crates/watch` and `crates/server` where the framework handles the transport but the app still hand-codes many protocol-level behaviors.

The weak or absent candidates are:

1. Audio synthesis: no custom audio engine found.
2. UI rendering: no custom UI renderer found; the repo already relies on engine/client frameworks, but not `bevy_egui`.
3. Math utilities: no clear custom math library to replace; most geometry is simple domain math, not a standalone math layer.
4. ECS duplication: some logic is ECS-shaped, but I did not find a separate ECS reimplementation.

## Findings

### 1. Custom noise functions

#### What is hand-rolled

- `crates/watch/src/terrain.rs` implements multi-octave value noise with a bespoke lattice hash and smoothstep interpolation.
- The same pattern also appears in client-side terrain helpers:
  - `clients/bevy-ref/src/bin/terrain.rs`
  - `clients/bevy-ref/src/bin/standalone.rs`
  - `clients/godot-ref/rust/src/lib.rs`

Key pieces:

- `value_noise`
- `smoothstep`
- `hash_to_unit`
- octave summation and island falloff

#### Better crate

- `noise`
- `fastnoise-lite`

#### Why this is a good replacement

- Both crates already provide seeded simplex/value/perlin-style noise with octave/fractal support.
- The current code is small, but duplicated across multiple targets.
- A library implementation would reduce drift between watch, Bevy ref, and Godot ref terrain behavior.

#### Migration effort

- **Low to medium**
- Likely a drop-in replacement for the inner noise sampler, but not for the custom island falloff and biome thresholds.
- Expect some tuning to preserve current terrain shape and the deterministic fingerprint tests.

#### Priority

- **High**

#### Notes

- If replay identity matters, keep the island/falloff layer and replace only the lattice noise core with a crate.
- If exact fingerprints are contractual, migration should be test-gated and done behind a feature flag or compatibility fixture.

### 2. Custom HTTP/SSE handling

#### What is hand-rolled

- `crates/watch/src/main.rs` defines a full HTTP API surface with Axum routes, SSE streaming, ETag handling, and remote fetch logic.
- `crates/server/src/ws_bridge.rs` implements a custom WebSocket bridge with JSON-RPC dispatch, tick broadcast coordination, and bespoke request/response plumbing.

Important details:

- `crates/watch/src/main.rs:3269` `sse_handler` builds SSE events from a broadcast stream manually.
- `crates/watch/src/main.rs:1893` `terrain_handler` manually implements `If-None-Match` / `ETag` handling.
- `crates/server/src/ws_bridge.rs` manually manages broadcast formatting and JSON-RPC response emission.

#### Better crate

- Transport/framework layer is already established:
  - `axum`
  - `tokio`
  - `tower-http`
  - `reqwest`
- For SSE, Axum’s SSE support is already the correct primitive.
- For WebSocket broadcast and JSON-RPC framing, there is not a universal single crate that removes the domain-specific logic, but `jsonrpc` helper crates could replace some request/response boilerplate if the protocol were less custom.

#### Why this is only a partial replacement target

- The HTTP layer itself is not the problem; the custom part is the application protocol on top of it.
- The custom handlers are mostly thin wrappers around app-specific state, not reinventions of the entire transport stack.

#### Migration effort

- **Low** for the transport layer itself
- **Medium to high** if trying to replace the custom JSON-RPC / broadcast protocol shaping

#### Priority

- **Medium**

#### Notes

- Keep Axum/tower-http/reqwest.
- Consider extracting repeated response/header patterns into shared helpers instead of trying to replace the whole stack.

### 3. Custom serialization formats

#### What is hand-rolled

- `crates/engine/src/replay_format.rs` defines a bespoke `.civreplay` container:
  - magic bytes
  - version
  - payload length
  - RON payload
  - SHA-256 footer
- `crates/protocol-3d/src/lib.rs` defines a bespoke `F3D0` binary envelope for `Frame3d`.
- `crates/server/src/saves.rs` and `crates/watch/src/main.rs` hand-code JSON response payloads for save metadata and control endpoints.

#### Better crate

- For human-readable structured serialization:
  - `serde`
  - `ron`
  - `serde_json`
  - `serde_yaml`
- For compact binary encoding:
  - `postcard`
  - `bincode`
  - `rmp-serde`
- For containerized archive payloads:
  - `zstd`
  - `tar`

#### Why this is nuanced

- The repo already uses the standard serialization crates correctly for the payloads.
- The custom part is the outer envelope, checksum, and magic/version framing.
- That envelope may be justified for replay integrity and forward compatibility, but it is still custom serialization infrastructure.

#### Migration effort

- **Low** to replace payload encoding
- **Medium to high** to replace the container semantics without breaking backward compatibility

#### Priority

- **High**

#### Notes

- `ReplayLog` already uses RON internally; the custom part is the `.civreplay` container around it.
- `Frame3d` is already `serde`-friendly; the custom envelope exists to support WebSocket binary frames, not because serde is missing.
- If the goal is to reduce hand-rolled code, the most realistic win is to keep the protocol shape but move more of the implementation onto standard encode/decode helpers.

### 4. Custom math utilities

#### What is hand-rolled

- There are small domain math helpers throughout the repo:
  - centroid/rotation logic in `crates/tactics/src/formation.rs`
  - Manhattan distance, sphere checks, and simple interpolation in tactical code
  - terrain falloff and normalization code in `crates/watch`

#### Better crate

- `glam`
- `nalgebra`

#### Assessment

- I did not find a standalone custom math library here.
- The repo is doing ordinary game-domain math inline, which is normal and not worth replacing wholesale.

#### Migration effort

- **High**

#### Priority

- **Low**

#### Notes

- `glam` is already part of the Bevy stack, but the current math is too domain-specific to justify a migration on its own.

### 5. Custom UI rendering

#### What is hand-rolled

- `clients/godot-ref/scripts/*.gd` and `crates/watch/src/main.rs` contain project-specific UI behavior and dashboard rendering logic.
- `clients/bevy-ref` has Bevy rendering helpers and scene setup code.

#### Better crate

- `bevy_egui`

#### Assessment

- I did not find a custom retained-mode UI system that would obviously be better served by `bevy_egui`.
- The existing UI surfaces are mostly:
  - Godot scene/UI code
  - web dashboard code
  - Bevy scene setup

#### Migration effort

- **High**

#### Priority

- **Low**

#### Notes

- This is not a clear wrap-over-hand-roll candidate unless the Bevy client is expected to gain a native debug UI.

### 6. Custom audio synthesis

#### What is hand-rolled

- No clear custom audio synthesis pipeline found in `crates/` or `clients/`.

#### Better crate

- `bevy_kira_audio`

#### Assessment

- Not applicable based on current code.

#### Priority

- **None**

### 7. Custom pathfinding

#### What is hand-rolled

- `crates/tactics/src/pathfinding.rs` implements:
  - deterministic BFS next-step selection
  - deterministic A* full-path search
  - custom tie-breaking and blocked-cell logic
- `crates/tactics/src/movement.rs` builds movement behavior on top of that custom pathfinder.

#### Better crate

- `oxidized_navigation`
- `petgraph`

#### Why this is a strong candidate

- The repo has already built general graph search logic by hand.
- The implementation is deterministic and grid-specific, which makes it a candidate for a maintained pathfinding library if the API can preserve ordering and blocking semantics.
- `petgraph` helps with graph algorithms but would not directly give the exact grid-specific movement semantics.
- `oxidized_navigation` is a better fit if the pathfinding surface is going to expand beyond simple grid search.

#### Migration effort

- **Medium**

#### Priority

- **High**

#### Notes

- The current code is correct-looking and test-covered, so this is not a bug report.
- It is still a hand-rolled algorithm implementation that could be delegated to a crate if the team wants to stop maintaining search logic itself.

### 8. Custom ECS patterns duplicating Bevy systems

#### What is hand-rolled

- I did not find a separate ECS implementation.
- The code uses Bevy ECS patterns where Bevy exists, and plain Rust data structures elsewhere.

#### Better crate

- Bevy ECS itself

#### Assessment

- No clear duplicate ECS system was found.

#### Priority

- **Low / none**

### 9. Any reimplementation of algorithms available in well-maintained crates

#### Strong candidates

- `crates/watch/src/terrain.rs` and client terrain helpers:
  - custom multi-octave noise, hashing, interpolation
  - replacement: `noise` or `fastnoise-lite`
- `crates/tactics/src/pathfinding.rs`:
  - custom BFS and A*
  - replacement: `oxidized_navigation` or `petgraph`
- `crates/engine/src/replay_format.rs`:
  - custom container framing around RON
  - replacement: standard serialization stack plus archive tooling, or keep as deliberate domain-specific envelope

#### Medium candidates

- `crates/server/src/ws_bridge.rs` and `crates/watch/src/main.rs`:
  - custom HTTP/WebSocket/SSE plumbing layered on Axum
  - replacement: keep the framework, extract and simplify protocol glue

#### Low candidates

- `crates/tactics/src/formation.rs`:
  - centroid/rotation/layout math
  - replacement: not compelling; this is small domain code
- `clients/bevy-ref/src/native_renderer.rs`:
  - GPU capability detection
  - replacement: not worth it; this is adapter inspection, not a reimplementation of a general algorithm

## Recommended priority order

1. Replace or isolate terrain noise logic behind `noise` or `fastnoise-lite`.
2. Replace or centralize custom pathfinding behind a maintained grid/path crate.
3. Revisit `.civreplay` and `F3D0` envelope ownership to decide whether the custom container is still justified.
4. Simplify repeated HTTP/WebSocket/SSE response plumbing, but keep Axum/reqwest/tower-http.
5. Leave math/UI/audio/ECS alone unless a new feature creates a clearer need.

## Practical migration guidance

- If exact output matters, add golden tests before swapping any implementation.
- For terrain noise, preserve the current biome/falloff layer and replace only the inner noise sampler first.
- For pathfinding, keep deterministic ordering as a non-negotiable contract.
- For serialization, distinguish between payload serialization and container framing. The payload layer is mostly already standard; the wrapper layer is what is custom.

