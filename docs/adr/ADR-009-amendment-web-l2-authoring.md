# ADR-009 Amendment: Web L2 Authoring (Not Spectator-Only)

**Date:** 2026-05-24  
**Status:** ACCEPTED (amends [ADR-009](./ADR-009-web-client-strategy.md) default)  
**Supersedes:** “Web never mutates the sim” as a hard rule

## Decision

The browser dashboard **may mutate** the live simulation via the **same RPC/HTTP surface**
as Godot, within explicit **web limits**:

| Allowed on web | Backend |
|----------------|---------|
| Spawn civilian | `sim.spawn_civilian` / `POST /control/spawn_civilian` |
| Place voxel | `sim.place_voxel` / `POST /control/place_voxel` |
| Tactical damage | `POST /control/damage` (watch attach only) |
| Inspect, speed, replay | unchanged |

**Default:** authoring **enabled**. Use `?spectator=1` for metrics/replay-only demos.

## What stays out of scope (unchanged from ADR-009)

- Full P-U1 spawn palette, vehicles, airports, drag-build UX (Godot)
- GOAP / economy authoring / institution editing UI
- Manor Lords–class visuals (Unreal + art pipeline)
- Godot HTML5 export, Bevy WASM full game

## Rationale

Operators and contributors need a **zero-install** attach path that shares the civ-server
timeline with Godot. Read-only-only forced duplicate workflows (watch attach for edits).

## Limits of web (L2 cap)

Documented in [product-quality-ladder.md](../roadmap/product-quality-ladder.md): web targets
**L2 sandbox**, not L3–L5 product tier.
