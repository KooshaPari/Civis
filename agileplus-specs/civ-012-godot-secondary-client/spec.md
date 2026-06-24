---
spec_id: civ-012
state: ACTIVE
plan_status: PLANNED
last_audit: 2026-05-29
---

# Specification: Godot Secondary Client

**Slug**: civ-012-godot-secondary-client | **Epic**: E8 | **Date**: 2026-05-29 | **State**: ACTIVE

## Problem Statement

Godot is the designated secondary client target. While Bevy is the primary and mandatory desktop experience, Godot provides an alternative renderer for modders and contributors who prefer GDScript or GDExtension. The Godot client must consume the same wire protocol (JSON-RPC + binary frames) as the Bevy client, render world state via scene tree sync, and serve as a validation that the protocol is engine-agnostic.

Note: Web client and TypeScript client are deprecated as of 2026-05-28 pivot. Godot supersedes the web tier as the secondary validation target.

## Target Users

- Godot GDScript/GDExtension developers implementing the client
- Protocol validation engineers verifying engine-agnostic protocol design
- Modders preferring Godot as a modding entry point

## Functional Requirements

- [ ] **FR-CIV-CLIENT-GODOT-001**: Godot 4.x GDScript plugin connects to `civ-server` via WebSocket; completes handshake in < 2 s; subscribes to binary tick frames; syncs scene tree entities each tick from tick delta
- [ ] **FR-CIV-CLIENT-GODOT-002**: Godot client renders deterministic snapshot at strategic zoom level (Zoom 1: nation scale); entities represented as 3D instanced meshes; camera supports pan and zoom; minimal HUD (resource bar + event feed)

## Non-Functional Requirements

- Godot version: 4.x (latest stable at time of implementation)
- Client lives in `clients/godot-ref/` (to be created)
- Consumes same binary frame format as Bevy client (FR-PROTO-004)
- No separate protocol extension — protocol must be 100% compatible
- Metal rendering on M1 MacBook; Vulkan on Linux fallback

## Constraints and Dependencies

- Depends on: FR-PROTO-004 (binary frames) — same wire format as Bevy client
- Depends on: FR-PROTO-003 (client handshake) — same handshake protocol
- Depends on: civ-011 Bevy client for protocol conformance baseline
- Target release v2; not required for MVP or v1

## Acceptance Criteria

- [ ] Godot client connects to `civ-server` and completes handshake < 2 s
- [ ] Scene tree entities sync from each tick delta without drift
- [ ] Strategic zoom renders nation-scale entities as instanced meshes at 60 FPS
- [ ] Camera pan and zoom functional
- [ ] Resource bar displays correct values from server state
- [ ] Event feed populates from tick event stream
- [ ] Protocol conformance test: Bevy and Godot clients see identical snapshots simultaneously

## Status

| Story | Status |
|-------|--------|
| E6.5 Godot GDScript plugin | Planned |
| E6.6 Godot playable demo | Planned |
| Protocol conformance test | Planned |
