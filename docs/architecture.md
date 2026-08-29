---
title: Architecture
description: Crate layout, engine boundaries, and module topology for the Civis simulation stack.
---

# Architecture

## Overview

Civis is a Rust workspace composed of a deterministic simulation engine, a WebSocket/JSON-RPC server, a watcher service, and a Bevy-based 3D client. The architecture isolates the engine (`crates/engine`) from any rendering, networking, or persistence concern by enforcing tight crate boundaries and the single source of truth (SSOT) for world state at `crates/engine/src/lib.rs`.

The engine runs at a fixed 60 Hz tick, advances an ECS-backed world, and emits deterministic snapshots that any client can replay given the same seed. All transports (WebSocket frame broadcast, JSON-RPC, mod HTTP control plane) are layers on top of the engine API; they never mutate world state directly.

The workspace is organized so that:

- The **engine** owns the ECS world, fixed-point math, and the tick loop.
- The **server** (`crates/server`) exposes the engine over WebSocket JSON-RPC and broadcasts `Frame3d` per tick.
- The **watch** service (`crates/watch`) hosts the mod HTTP control API and remote-cache proxy.
- The **client** lives in `clients/bevy-ref` and consumes the server's frame broadcast.
- **Mods** (`crates/mod-host`) are dynamically loaded `.civmod` archives that can hook into engine phases without recompiling.

## Workspace Layout

| Crate | Role |
|-------|------|
| `crates/engine` | ECS, fixed-point math, `Simulation`, `step`, `metrics::compute`, `WorldState`. |
| `crates/server` | WebSocket bridge, JSON-RPC dispatch (`JsonRpcMethod`, `dispatch_request`), `Frame3d` broadcast. |
| `crates/watch` | HTTP control routes on `127.0.0.1:9090`, mod catalog, uploads, publish, remote cache. |
| `crates/diplomacy` | Faction relations, treaties, alliance / war state transitions. |
| `crates/economy` | Joule budget, production, consumption, scarcity policy multipliers. |
| `crates/ai` | Citizen needs, ideology, faction goal planner, behavior-tree scaffolding. |
| `crates/legends` | Saga-graph storage for emergent narrative events. |
| `crates/emergence-oracle` | Emergence sample collection and per-tick analytics. |
| `crates/observability` | Tracing, OTLP export, structured logging. |
| `crates/asset-pipeline` | Build-time asset packaging for the Bevy client. |
| `crates/protocol-3d` | Wire format definitions for `Frame3d` and tick broadcasts. |
| `crates/save-db` | Save/replay persistence and replay loaders. |
| `crates/mod-host` | Mod ABI, hot-reload, sandbox boundaries. |
| `crates/civis-cli` | Operator CLI for running server, watch, replay, and scenario commands. |
| `crates/civis-mcp` | MCP server exposing simulation state to external agents. |
| `crates/civlab-sdk` | Public SDK for embedding Civis into other products. |

## Engine Boundary

The engine boundary is the strict rule that `crates/engine` exposes only data types and pure functions. No I/O, no networking, no time source, no randomness other than the deterministic `ChaCha8Rng` seeded at construction. Every other crate depends on the engine; the engine depends on no other Civis crate.

```text
                 ┌──────────────────────────┐
                 │     clients/bevy-ref     │   consumes Frame3d
                 └────────────┬─────────────┘
                              │ ws://
                 ┌────────────▼─────────────┐
                 │      crates/server       │   JSON-RPC + broadcasts
                 └────────────┬─────────────┘
                              │
                 ┌────────────▼─────────────┐
                 │      crates/watch        │   HTTP control + mods
                 └────────────┬─────────────┘
                              │
                 ┌────────────▼─────────────┐
                 │      crates/engine       │   ECS + step + metrics
                 └──────────────────────────┘
```

## Tick Phases

Each tick runs the following phases in order. Per-phase timing budgets are soft targets for profiling; the tick loop guarantees deterministic ordering but not a fixed wall-clock budget.

| Phase | Responsibility |
|-------|----------------|
| Pre-update | Input ingestion, AI decision snapshots. |
| Physics | Movement, collision, pathfinding on the hex grid. |
| Economy | Production, consumption, treasury updates, policy multipliers. |
| AI | Behavior-tree tick, goal planner, faction decisions. |
| Governance | Policy application, metrics calculation, event emission. |
| Diplomacy | Treaty decay, alliance formation, war escalation checks. |
| Post-update | State validation, snapshot if stride matched, network sync. |

## Determinism Guarantees

The engine is required to be deterministic from the same seed:

- All numeric work uses `Fixed` (i64 with `SCALE = 1_000_000`).
- The RNG is a `ChaCha8Rng` seeded at `Simulation::with_seed`.
- `HashMap` iteration uses a stable insertion-ordered variant where ordering matters.
- No thread-local time, environment, or filesystem reads inside the engine.

`WorldState` is the serializable global state used by both `step` and `Simulation`. It is the only object that crosses the transport boundary in a snapshot.

## See Also

- [Simulation](/simulation/) — tick loop, components, ECS query patterns.
- [Economy](/economy/) — Joule budget, scarcity policy, treasury model.
- [AI](/ai/) — citizen needs, ideology, faction goal planner.
- [Diplomacy](/diplomacy/) — faction relations, treaties, war state.
- [API](/api/) — JSON-RPC surface and `Frame3d` wire format.
- [Development](/development/) — workspace tooling, build, and test loop.
- [Deployment](/deployment/) — server, watch, and Grafana deployment.