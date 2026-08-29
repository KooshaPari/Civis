---
title: API
description: Civis public APIs — Rust engine crate, JSON-RPC over WebSocket, civ-watch HTTP control, and civ-lab SDK.
---

# API

## Overview

Civis exposes four public API surfaces, each with a clearly separated role:

1. **Rust engine crate** (`civ_engine`) — deterministic simulation API used by all other Civis binaries and by external Rust embedders via `crates/civlab-sdk`.
2. **WebSocket JSON-RPC** (`civ-server`) — operational API for clients; tick broadcast of `Frame3d` over the same socket.
3. **HTTP control** (`civ-watch`) — mod lifecycle, uploads, publish, install, remote cache.
4. **civ-lab SDK** (`crates/civlab-sdk`) — high-level SDK wrapping the above for embedded use.

This page summarizes each surface and links to the detailed reference. For the full method catalog, see the linked subpages.

## Rust Engine Crate

```rust
use civ_engine::{Fixed, WorldState, Simulation, metrics, step};

let state = WorldState::default();
let next = step(state, Fixed::from_num(1_000));
let summary = metrics::compute(
    next.energy_budget_joules.to_f64(),
    Fixed::from_num(1_000).to_f64(),
);
```

| API | Description |
|-----|-------------|
| `Fixed::from_num(value)` | Convert integer-like value to scaled fixed point. |
| `Fixed::saturating_add(other)` | Overflow-safe addition. |
| `Fixed::clamp(min, max)` | Inclusive bounds. |
| `step(state, consumption_joules)` | Smallest deterministic state transition. |
| `Simulation::new()` | Default seed-42 simulation with starter entities. |
| `Simulation::with_seed(seed)` | Caller-supplied deterministic seed. |
| `Simulation::tick()` | Advance one tick (all phases). |
| `Simulation::snapshot()` | `SimulationSnapshot` aggregate view. |
| `metrics::compute(energy, consumption)` | Tyranny, legitimacy, waste, surplus. |

See [Simulation](/simulation/) for the full simulation API.

## WebSocket JSON-RPC (`civ-server`)

Connect to `ws://<bind>/ws` and send JSON-RPC 2.0 requests as text frames. Tick pushes (`Frame3d`) are broadcast separately, not as JSON-RPC responses.

Selected methods (full catalog in [JSON-RPC surface](api/jsonrpc-surface.md)):

| Method | Params | Result |
|--------|--------|--------|
| `health` | `{}` | `{ "tick": <u64> }` |
| `sim.status` | `{}` | `{ "tick", "population"? }` |
| `sim.snapshot` | `{}` | Full snapshot or `{ "tick", "speed_multiplier" }` |
| `sim.subscribe` | `{ "frame_kinds"? }` | `{ "subscribed": true, "subscription_id" }` |
| `sim.command` | `{ "action": "tick" \| "noop" }` | `{ "accepted": true }` |
| `sim.save_replay` | `{ "path" }` | `{ "saved": true, "path" }` |
| `sim.load_replay` | `{ "path" }` | `{ "loaded": true, "tick" }` |
| `sim.reset` | `{ "seed" }` | `{ "seed", "tick": 0 }` |
| `sim.set_policy` | `{ "scarcity_multiplier", "base_consumption_joules"? }` | `{ "updated": true }` |

Role gate: when `WsBridgeConfig::require_role` is true (env `CIVIS_REQUIRE_ROLE=1`), privileged methods require effective role `"operator"` from `params.role`. Error: `-32003` (`FORBIDDEN`) with `data.required_role: "operator"`.

## HTTP Control (`civ-watch`)

`civ-watch` listens on `http://127.0.0.1:9090` by default (`CIV_WATCH_PORT` overrides).

| Method | Path | Description |
|--------|------|-------------|
| `GET` | `/control/mods/catalog` | List installable mods (examples, uploads, publish, remote cache). |
| `POST` | `/control/mods/upload` | Upload a `.civmod` archive (base64 body). |
| `POST` | `/control/mods/publish` | Copy a validated mod into `mods/publish/`. |
| `GET` | `/control/mods/published` | List published mods. |
| `POST` | `/control/mods/install` | Load a mod from a catalog `source` path. |
| `POST` | `/control/mods/unload` | Unload a mod by stable `mod_id`. |
| `POST` | `/control/mods/reload` | Hot-reload a loaded mod by `mod_id`. |

Validation failures return HTTP `400` with `{ "ok": false, "message": "reason" }`. See [civ-watch control API](api/watch-control.md) for full detail.

## civ-lab SDK

`crates/civlab-sdk` is the public SDK for embedding Civis into other products. It re-exports `civ_engine` and provides high-level wrappers for connecting to `civ-server` and `civ-watch`:

```rust
use civlab_sdk::{EngineHandle, ServerClient, WatchClient};

let engine = EngineHandle::connect("civ-server://127.0.0.1:7777")?;
let watch  = WatchClient::connect("http://127.0.0.1:9090")?;
```

The SDK is API-stable across minor versions; breaking changes require a major version bump.

## See Also

- [Architecture](/architecture/) — API crate layout and engine boundary.
- [Simulation](/simulation/) — engine API in depth.
- [Development](/development/) — running the API locally for development.
- [Deployment](/deployment/) — running the API in production.
- [JSON-RPC surface](api/jsonrpc-surface.md) — full method catalog.
- [civ-watch control](api/watch-control.md) — full HTTP control surface.