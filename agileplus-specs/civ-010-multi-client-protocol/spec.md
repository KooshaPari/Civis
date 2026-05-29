---
spec_id: civ-010
state: ACTIVE
plan_status: IN_PROGRESS
last_audit: 2026-05-29
---

# Specification: Multi-Client Protocol

**Slug**: civ-010-multi-client-protocol | **Epic**: E3 | **Date**: 2026-05-29 | **State**: ACTIVE

## Problem Statement

The headless simulation core must serve multiple simultaneous clients (game, research, spectator) over a standard WebSocket protocol. Clients receive tick deltas as zstd-compressed binary frames for high-frequency game use, or JSON-RPC for research tools. Snapshot filtering reduces per-client bandwidth. Role-based authorization prevents research clients from issuing build commands. The protocol must be engine-agnostic: Bevy, Godot, and research scripts all speak the same wire format.

## Target Users

- Server crate developers implementing `civ-server` and `civ-protocol-3d`
- Bevy client developers consuming binary tick frames
- Godot client developers implementing GDScript WebSocket bridge
- Research API developers using JSON-RPC scenario runner

## Functional Requirements

- [ ] **FR-PROTO-001**: RFC 6455 WebSocket server on configurable port; accepts >= 10 simultaneous connections; TLS support for non-localhost; graceful close on shutdown
- [ ] **FR-PROTO-002**: JSON-RPC 2.0 dispatcher; all methods return `result` or `error`; unknown method → error -32601; batch requests supported
- [ ] **FR-PROTO-003**: Client handshake completes in < 2 s on local network; bootstrap snapshot includes all entity states at current tick; role (admin/player/research) assigned and enforced on all subsequent commands
- [ ] **FR-PROTO-004**: Binary frame format — header: tick number, frame type, uncompressed size, checksum; zstd compression ratio >= 3:1 on typical delta; 10 clients at 60 FPS <= 10 Mbps aggregate
- [ ] **FR-PROTO-005**: Snapshot filtering — clients subscribe with entity-type and/or region bounding-box filter; server excludes filtered entities from delta frames; filter updatable via subscription command
- [ ] **FR-CLIENT-003**: Role authorization — research clients cannot issue build or policy commands; unauthorized → JSON-RPC error -32603 with role information; enforced by integration tests covering all three role tiers

## Non-Functional Requirements

- Crate: `civ-server` (WebSocket + REST) + `civ-protocol-3d` (binary frame format)
- `ws_bridge` in `civ-server` handles WS connection lifecycle
- WebSocket smoke tests in `server/tests/ws_smoke.rs`
- Snapshot serialization must complete in < 1 ms P99 to not delay tick broadcast

## Constraints and Dependencies

- Depends on: FR-CORE-006 (command queue) for multi-client command ingestion
- Depends on: FR-REPLAY-001 (.civreplay) for tick event sourcing
- Bevy primary client consumes binary frames; Godot secondary client consumes JSON-RPC or binary frames
- Web client deprecated (per 2026-05-28 pivot); TypeScript client spec archived

## Acceptance Criteria

- [ ] Server accepts >= 10 simultaneous WebSocket connections without degradation
- [ ] Client handshake completes < 2 s on local network
- [ ] Bootstrap snapshot correctly serializes all entity states at current tick
- [ ] Binary frames: zstd ratio >= 3:1; 10 clients at 60 FPS <= 10 Mbps
- [ ] Snapshot filter excludes out-of-bounds entities from delta frames
- [ ] Research client receives -32603 on unauthorized build command
- [ ] `server/tests/ws_smoke.rs` passes on every CI commit

## Implementation Notes

- `civ-server`: `SimServer`, `ClientHandler`, `ws_bridge`
- `civ-protocol-3d`: binary frame protocol
- JSON-RPC smoke test: `server/tests/ws_smoke.rs` partially exists
- TLS: configurable cert paths; localhost connections exempt

## Status

| Story | Status |
|-------|--------|
| E3.1 WebSocket server | Partial (`civ-server` + `ws_bridge` present; TLS + >=10 clients not tested) |
| E3.2 JSON-RPC 2.0 dispatcher | Partial (basic RPC; batch request support unverified) |
| E3.3 Client handshake + bootstrap | Planned |
| E3.4 Command protocol | Planned |
| E3.5 Snapshot subscription | Planned |
| E3.6 Binary frames | Planned (`civ-protocol-3d` stub present) |
| E3.7 Role authorization | Planned |
| E3.8 Snapshot filtering | Planned |
| E3.9 Query API | Planned |
| E3.10 Performance test | Planned |
