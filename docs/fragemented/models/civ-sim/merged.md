# Merged Fragmented Markdown

## Source: models/civ-sim/API_EVENTS_SPEC.md

---
title: CivLab API & Events Specification
date: 2026-02-21
status: SPECIFICATION
version: 1.0.0
owner: CIV Protocol & Integration Team
tags: [api, events, websocket, json-rpc, spec]
---

# CivLab API & Events Specification

**Spec ID:** CIV-API-001
**Version:** 1.0.0
**Status:** SPECIFICATION
**Date:** 2026-02-21
**Authors:** CIV Protocol & Integration Team
**Related Specs:** CIV-0001 (Core Simulation Loop), CIV-0200 (Multi-Client Protocol)

---

## Table of Contents

1. [Overview](#1-overview)
2. [Transport & Framing](#2-transport--framing)
3. [EventEnvelopeV1 Schema](#3-eventenvelopev1-schema)
4. [State Hash Chain](#4-state-hash-chain)
5. [JSON-RPC Methods — Simulation Control](#5-json-rpc-methods--simulation-control)
6. [JSON-RPC Methods — Research API](#6-json-rpc-methods--research-api)
7. [Broadcast Frame Format](#7-broadcast-frame-format)
8. [Event Types — Demographics](#8-event-types--demographics)
9. [Event Types — Economy](#9-event-types--economy)
10. [Event Types — Climate](#10-event-types--climate)
11. [Event Types — Political & Institutional](#11-event-types--political--institutional)
12. [Event Types — War & Diplomacy](#12-event-types--war--diplomacy)
13. [Event Types — Research & Lifecycle](#13-event-types--research--lifecycle)
14. [Command Types](#14-command-types)
15. [Rate Limiting & Backpressure](#15-rate-limiting--backpressure)
16. [Error Codes](#16-error-codes)
17. [Performance Targets & SLOs](#17-performance-targets--slos)
18. [Client SDK Examples](#18-client-sdk-examples)
19. [Versioning Policy](#19-versioning-policy)

---

## 1. Overview

CivLab is a deterministic civilization simulation engine. The runtime executes a fixed-timestep tick loop (100 ms per tick, configurable speed multiplier) and broadcasts world-state events after each tick. Multiple heterogeneous clients — game engines (Bevy, Unreal, Unity, Godot), web browsers, research scripts — attach to a single headless simulation core.

### Communication Model

```
┌──────────────────────────────────────────────────────────────┐
│                    CIVLAB SIM CORE                           │
│                                                              │
│  Tick Engine ──► Event Emitter ──► Broadcast Hub            │
│       ▲                                   │                  │
│       │                                   ▼                  │
│  Command Queue              ┌─────────────────────┐         │
│       ▲                     │  WebSocket Server   │         │
│       │                     │  JSON-RPC 2.0       │         │
└───────│─────────────────────┼─────────────────────┘         │
        │                     │                                │
        │              Text frames (JSON)                      │
        │             Binary frames (MsgPack)                  │
        │                     │                                │
   ┌────┴────┐    ┌────────────┴──────────────────────┐        │
   │ Game    │    │ Research Client  │  Web Client     │        │
   │ Engine  │    │ (Python/Rust)    │  (TypeScript)   │        │
   └─────────┘    └──────────────────┴─────────────────┘        │
```

### Protocol Summary

| Channel | Purpose | Format |
|---------|---------|--------|
| JSON-RPC over WebSocket | Command/control, subscriptions, queries | JSON text frames |
| Tick broadcast (text) | Per-tick event stream, <= 30 Hz | JSON-RPC notification |
| Tick broadcast (binary) | High-frequency game engine clients | MessagePack binary frames |
| Snapshot endpoint | On-demand state dump | HTTP(S) GET, gzipped JSON |
| Replay endpoint | Historical event replay | HTTP(S) GET, `.civreplay` (zstd compressed) |

### Key Invariants

1. **Determinism.** Given identical `seed` and identical command sequence, the simulation produces byte-for-byte identical event streams and state hashes.
2. **Immutable tick history.** Ticks are never revised. Once `tick.completed.v1` is broadcast, that tick's events are final.
3. **BLAKE3 hash chain.** Every snapshot hash is derived from the prior snapshot hash XOR'd with all events emitted in that tick, providing tamper-evidence.
4. **UUIDv7 event IDs.** Event IDs encode millisecond-precision wall-clock time plus random bits, enabling time-ordered sorting without a centralized counter.
5. **Monotonic sequence numbers.** Within a tick, each emitted event carries a `seq` counter starting at 0. The `(tick, seq)` pair is globally unique.

---

## 2. Transport & Framing

### 2.1 WebSocket Connection Parameters

| Parameter | Value |
|-----------|-------|
| Default port | `9876` |
| WebSocket path | `/sim` |
| Subprotocol | `civlab-v1` |
| TLS | Optional (required in production) |
| Max message size | 64 MB (configurable) |
| Ping interval | 15 s (client must respond to pong within 5 s) |
| Idle timeout | 60 s without any message |

### 2.2 JSON-RPC 2.0 Text Frame

All command/control messages use JSON-RPC 2.0 framing over WebSocket text frames.

**Request (client → server):**
```json
{
  "jsonrpc": "2.0",
  "id": "<string | integer>",
  "method": "<method_name>",
  "params": { ... }
}
```

**Response (server → client):**
```json
{
  "jsonrpc": "2.0",
  "id": "<same as request>",
  "result": { ... }
}
```

**Error response:**
```json
{
  "jsonrpc": "2.0",
  "id": "<same as request>",
  "error": {
    "code": -32001,
    "message": "Run not found",
    "data": { "run_id": "abc123" }
  }
}
```

**Notification (server → client, no id):**
```json
{
  "jsonrpc": "2.0",
  "method": "sim.tick_broadcast",
  "params": { ... }
}
```

### 2.3 Binary Frame Format (MessagePack)

High-frequency game engine clients may negotiate binary frames by sending `"binary": true` in `sim.subscribe`. Binary frames carry the same logical structure as JSON tick broadcasts but encoded as MessagePack for ~40% size reduction.

Binary frame header (8 bytes, little-endian):

```
Offset  Size  Field
0       4     magic = 0x43495642 ("CIVB")
4       2     version = 1
6       1     frame_type (1=tick_broadcast, 2=snapshot_delta)
7       1     flags (bit 0 = compressed with zstd)
```

Followed immediately by MessagePack-encoded payload (same structure as JSON tick broadcast params).

### 2.4 Snapshot HTTP Endpoint

Snapshots are too large for WebSocket. The server exposes an HTTP endpoint:

```
GET /snapshots/{run_id}/{tick}
  Query params:
    scope = "full" | "delta" | "entities_only"
    format = "json" | "msgpack"
  Response headers:
    Content-Type: application/json (or application/msgpack)
    X-State-Hash: <hex-encoded BLAKE3 hash>
    X-Tick: <integer>
    X-Run-Id: <string>
  Response body: gzipped JSON or MessagePack state snapshot
```

### 2.5 Replay Endpoint

```
GET /replays/{run_id}
  Query params:
    from_tick = <integer>
    to_tick   = <integer>
    filter    = <comma-separated event_type globs>
  Response headers:
    Content-Type: application/octet-stream
    Content-Disposition: attachment; filename="<run_id>-<from>-<to>.civreplay"
  Response body: zstd-compressed .civreplay file
```

`.civreplay` file format:
- 16-byte magic header: `CIVREPLAY\x00\x01\x00\x00\x00\x00\x00`
- 4-byte tick count (uint32 LE)
- 4-byte event count (uint32 LE)
- JSON-encoded run metadata (null-terminated)
- Concatenated `EventEnvelopeV1` records (newline-delimited JSON)

---

## 3. EventEnvelopeV1 Schema

Every event emitted by the simulation is wrapped in `EventEnvelopeV1`. This envelope provides identity, ordering, hash-chain integrity, and routing metadata.

### 3.1 JSON Schema

```json
{
  "$schema": "https://json-schema.org/draft/2020-12/schema",
  "$id": "https://civlab.dev/schemas/EventEnvelopeV1.json",
  "title": "EventEnvelopeV1",
  "description": "Universal wrapper for all CivLab simulation events",
  "type": "object",
  "required": [
    "event_id",
    "event_type",
    "schema_version",
    "tick",
    "seq",
    "run_id",
    "seed",
    "prev_hash",
    "emitted_at_ms",
    "payload"
  ],
  "properties": {
    "event_id": {
      "type": "string",
      "format": "uuid",
      "description": "UUIDv7 — encodes wall-clock ms at emission; globally unique"
    },
    "event_type": {
      "type": "string",
      "pattern": "^[a-z][a-z0-9_]*(\\.[a-z][a-z0-9_]*)+\\.v[0-9]+$",
      "description": "Hierarchical dotted type, always versioned (e.g. economy.market_cleared.v1)"
    },
    "schema_version": {
      "type": "integer",
      "minimum": 1,
      "description": "Payload schema version; matches suffix in event_type"
    },
    "tick": {
      "type": "integer",
      "minimum": 0,
      "description": "Simulation tick at which this event occurred"
    },
    "seq": {
      "type": "integer",
      "minimum": 0,
      "description": "Monotonic sequence within the tick; (tick, seq) is globally unique"
    },
    "run_id": {
      "type": "string",
      "minLength": 1,
      "description": "Opaque identifier for the simulation run"
    },
    "seed": {
      "type": "integer",
      "description": "64-bit integer seed for this run's RNG"
    },
    "prev_hash": {
      "type": "string",
      "pattern": "^[0-9a-f]{64}$",
      "description": "BLAKE3 hex-encoded hash of the prior tick's accumulated hash; all zeros at tick 0"
    },
    "emitted_at_ms": {
      "type": "integer",
      "description": "Wall-clock Unix milliseconds when event was emitted"
    },
    "tags": {
      "type": "object",
      "additionalProperties": { "type": "string" },
      "description": "Optional key-value metadata (scenario_id, experiment_label, etc.)"
    },
    "payload": {
      "type": "object",
      "description": "Event-specific payload; schema determined by event_type"
    }
  },
  "additionalProperties": false
}
```

### 3.2 Example Envelope

```json
{
  "event_id": "018e6b3a-f1c2-7000-8000-000000000042",
  "event_type": "economy.market_cleared.v1",
  "schema_version": 1,
  "tick": 1440,
  "seq": 7,
  "run_id": "run_2026_02_21_001",
  "seed": 9876543210,
  "prev_hash": "a3f2c1d4e5b6789012345678901234567890abcdef1234567890abcdef123456",
  "emitted_at_ms": 1740134400000,
  "tags": {
    "scenario_id": "baseline_2050",
    "experiment_label": "carbon_tax_sweep_03"
  },
  "payload": {
    "good": "FOOD",
    "city_id": "city_lagos_001",
    "clearing_price": 142,
    "bid_volume": 50000,
    "ask_volume": 48500,
    "unmet_demand": 1500
  }
}
```

---

## 4. State Hash Chain

The simulation maintains a cryptographically chained hash over all tick snapshots. This enables any observer to verify the complete history has not been tampered with, and enables efficient incremental verification.

### 4.1 Chain Algorithm

```
// Initialization
snapshot[0].state_hash = BLAKE3(
    u64_le(seed) ||
    u64_le(0)    // tick 0
)

// Per-tick update
events_sorted = sort_by_seq(all_events_in_tick[t])
events_bytes  = concat(
    BLAKE3(canonical_json(e))
    for e in events_sorted
)

snapshot[t+1].state_hash = BLAKE3(
    snapshot[t].state_hash ||
    u64_le(t)  ||
    events_bytes
)
```

Where `canonical_json` is RFC 8785 (JSON Canonicalization Scheme) applied to the full `EventEnvelopeV1` object.

### 4.2 Verification

Clients can verify the chain at any tick by:

1. Requesting `snapshot[t-1].state_hash` (included in `sim.snapshot` result).
2. Fetching all events for tick `t` from the replay endpoint.
3. Recomputing `snapshot[t].state_hash` using the algorithm above.
4. Comparing to the hash in `tick.completed.v1` for tick `t`.

A mismatch indicates replay log corruption or tampering.

### 4.3 Hash in Events

Every `EventEnvelopeV1` carries `prev_hash`, which is the `state_hash` of the snapshot at the end of tick `t-1`. This allows event-level chain verification independent of the snapshot store.

---

## 5. JSON-RPC Methods — Simulation Control

### 5.1 `sim.handshake`

Establishes the client session and returns current simulation state. Must be the first RPC call after WebSocket upgrade. The server will not process any other method until handshake succeeds.

**Request:**
```json
{
  "jsonrpc": "2.0",
  "id": 1,
  "method": "sim.handshake",
  "params": {
    "client_id": "research-client-001",
    "protocol_version": "1.0",
    "client_type": "research",
    "capabilities": ["binary_frames", "compressed_snapshots"],
    "auth_token": "Bearer eyJ..."
  }
}
```

**Params Schema:**
| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `client_id` | string | yes | Caller-assigned stable identifier (max 64 chars) |
| `protocol_version` | string | yes | Must be `"1.0"` for this spec version |
| `client_type` | enum | yes | `"game_engine"` \| `"research"` \| `"web"` \| `"debug"` |
| `capabilities` | string[] | no | Optional feature negotiation |
| `auth_token` | string | no | Bearer token for authenticated deployments |

**Result:**
```json
{
  "jsonrpc": "2.0",
  "id": 1,
  "result": {
    "session_id": "sess_01HXR3BQZY2VKPWN5EMVD",
    "server_version": "0.9.0",
    "tick": 1440,
    "seed": 9876543210,
    "run_id": "run_2026_02_21_001",
    "scenario_id": "baseline_2050",
    "sim_status": "running",
    "snapshot_url": "http://localhost:9876/snapshots/run_2026_02_21_001/1440",
    "state_hash": "a3f2c1d4e5b6789012345678901234567890abcdef1234567890abcdef123456",
    "tick_rate_hz": 10,
    "speed_multiplier": 1.0,
    "negotiated_capabilities": ["compressed_snapshots"]
  }
}
```

**Result Schema:**
| Field | Type | Description |
|-------|------|-------------|
| `session_id` | string | Server-assigned session identifier |
| `server_version` | string | Server semver |
| `tick` | integer | Current simulation tick at time of handshake |
| `seed` | integer | Run RNG seed |
| `run_id` | string | Active run identifier |
| `scenario_id` | string | Loaded scenario |
| `sim_status` | enum | `"running"` \| `"paused"` \| `"completed"` \| `"idle"` |
| `snapshot_url` | string | URL to fetch full state snapshot |
| `state_hash` | string | BLAKE3 hex hash of state at `tick` |
| `tick_rate_hz` | integer | Configured ticks per second |
| `speed_multiplier` | number | Current speed (1.0 = real time, 0 = paused) |
| `negotiated_capabilities` | string[] | Server-confirmed capabilities |

**Errors:**
| Code | Meaning |
|------|---------|
| `-32010` | Unsupported protocol version |
| `-32011` | Authentication failed |
| `-32012` | Server at max client capacity |

---

### 5.2 `sim.subscribe`

Subscribe to per-tick event broadcast. After subscribing, the server pushes `sim.tick_broadcast` notifications after each tick completes.

**Request:**
```json
{
  "jsonrpc": "2.0",
  "id": 2,
  "method": "sim.subscribe",
  "params": {
    "filter_types": ["economy.*", "citizen.*", "climate.co2_threshold_crossed.v1"],
    "max_framerate_hz": 30,
    "binary": false,
    "include_snapshot_delta": false,
    "start_at_tick": null
  }
}
```

**Params Schema:**
| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `filter_types` | string[] | no | Glob patterns for event_type; `null` or `["*"]` = all events |
| `max_framerate_hz` | integer | no | Throttle broadcast rate (1-60); default 10 |
| `binary` | boolean | no | Use MessagePack binary frames; default false |
| `include_snapshot_delta` | boolean | no | Append state delta to each broadcast; default false |
| `start_at_tick` | integer\|null | no | Replay from this tick before switching to live; null = live only |

**Result:**
```json
{
  "jsonrpc": "2.0",
  "id": 2,
  "result": {
    "subscription_id": "sub_01HXR3BQZY2VKPWN5EMVD",
    "current_tick": 1440,
    "filter_applied": ["economy.*", "citizen.*", "climate.co2_threshold_crossed.v1"],
    "effective_framerate_hz": 10
  }
}
```

**Notes:**
- `filter_types` glob matching uses standard Unix glob rules: `*` matches any sequence of non-dot characters; `**` matches across dots.
- `max_framerate_hz` is bounded by the simulation's actual tick rate. If the sim runs at 10 Hz, requesting 30 Hz delivers at 10 Hz.
- If `start_at_tick` is provided, the server replays historical events (filtered) before switching to live. This may cause a burst of messages.

---

### 5.3 `sim.command`

Submit a player or agent command to the simulation. Commands are queued and applied at the start of the next tick. The response indicates whether the command was accepted into the queue, not yet whether it was executed successfully.

**Request:**
```json
{
  "jsonrpc": "2.0",
  "id": 3,
  "method": "sim.command",
  "params": {
    "command_type": "POLICY_SET",
    "actor_id": "nation_usa",
    "target_id": "policy_carbon_tax_v2",
    "params": {
      "rate_per_tonne_co2": 85,
      "effective_tick": 1441,
      "duration_ticks": 3600
    },
    "idempotency_key": "cmd_01HXR3C1A2B3C4D5E6F7G8H"
  }
}
```

**Params Schema:**
| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `command_type` | string | yes | One of the command types defined in Section 14 |
| `actor_id` | string | yes | Entity issuing the command (nation_id, city_id, etc.) |
| `target_id` | string | no | Target entity (may be same as actor) |
| `params` | object | yes | Command-type-specific parameters |
| `idempotency_key` | string | no | Dedup key; same key within 100 ticks = no-op |

**Result:**
```json
{
  "jsonrpc": "2.0",
  "id": 3,
  "result": {
    "accepted": true,
    "command_id": "cmd_01HXR3C1A2B3C4D5E6F7G8H",
    "tick_queued": 1440,
    "tick_applied": 1441,
    "reason": null,
    "validation_warnings": []
  }
}
```

**Result Schema:**
| Field | Type | Description |
|-------|------|-------------|
| `accepted` | boolean | Whether the command was accepted into the queue |
| `command_id` | string | Server-assigned command identifier |
| `tick_queued` | integer | Tick at which the command was received |
| `tick_applied` | integer | Tick at which the command will execute |
| `reason` | string\|null | Rejection reason if `accepted: false` |
| `validation_warnings` | string[] | Non-fatal warnings (command accepted but may have side effects) |

**Errors:**
| Code | Meaning |
|------|---------|
| `-32003` | Command rejected (see reason field) |
| `-32004` | Rate limit exceeded (max 10 commands per tick per client) |
| `-32005` | Simulation paused (use sim.resume first) |
| `-32006` | Actor not authorized for this command |

---

### 5.4 `sim.snapshot`

Request an on-demand state snapshot. For snapshots larger than 1 MB, the server returns a URL; for smaller snapshots, the data is inlined.

**Request:**
```json
{
  "jsonrpc": "2.0",
  "id": 4,
  "method": "sim.snapshot",
  "params": {
    "tick": 1440,
    "scope": "full",
    "format": "json",
    "inline_threshold_bytes": 524288
  }
}
```

**Params Schema:**
| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `tick` | integer\|null | no | Tick to snapshot; null = current tick |
| `scope` | enum | no | `"full"` \| `"entities_only"` \| `"economy_only"` \| `"delta"` |
| `format` | enum | no | `"json"` \| `"msgpack"` |
| `inline_threshold_bytes` | integer | no | Return inline if <= this; else return URL; default 1048576 |

**Result (URL-referenced):**
```json
{
  "jsonrpc": "2.0",
  "id": 4,
  "result": {
    "tick": 1440,
    "run_id": "run_2026_02_21_001",
    "scope": "full",
    "state_hash": "a3f2c1d4e5b6789012345678901234567890abcdef1234567890abcdef123456",
    "size_bytes": 4718592,
    "snapshot_url": "http://localhost:9876/snapshots/run_2026_02_21_001/1440?scope=full",
    "expires_at_ms": 1740138000000,
    "inline_data": null
  }
}
```

**Result (inlined):**
```json
{
  "jsonrpc": "2.0",
  "id": 4,
  "result": {
    "tick": 1440,
    "run_id": "run_2026_02_21_001",
    "scope": "entities_only",
    "state_hash": "...",
    "size_bytes": 131072,
    "snapshot_url": null,
    "expires_at_ms": null,
    "inline_data": { "nations": [...], "cities": [...], "citizens_count": 7200000 }
  }
}
```

---

### 5.5 `sim.replay`

Request a replay file for a historical tick range.

**Request:**
```json
{
  "jsonrpc": "2.0",
  "id": 5,
  "method": "sim.replay",
  "params": {
    "from_tick": 0,
    "to_tick": 1440,
    "filter_types": ["war.*", "policy.*"],
    "format": "civreplay"
  }
}
```

**Params Schema:**
| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `from_tick` | integer | yes | Start of replay range (inclusive) |
| `to_tick` | integer | yes | End of replay range (inclusive) |
| `filter_types` | string[] | no | Event type globs to include; null = all |
| `format` | enum | no | `"civreplay"` \| `"ndjson"` |

**Result:**
```json
{
  "jsonrpc": "2.0",
  "id": 5,
  "result": {
    "replay_url": "http://localhost:9876/replays/run_2026_02_21_001?from_tick=0&to_tick=1440",
    "event_count": 182340,
    "size_bytes": 8912384,
    "tick_range": [0, 1440],
    "expires_at_ms": 1740138000000
  }
}
```

---

### 5.6 `sim.pause`

Pause the simulation. No ticks advance; no events are emitted. Commands in the queue are preserved.

**Request:**
```json
{
  "jsonrpc": "2.0",
  "id": 6,
  "method": "sim.pause",
  "params": {}
}
```

**Result:**
```json
{
  "jsonrpc": "2.0",
  "id": 6,
  "result": {
    "paused_at_tick": 1440,
    "previous_status": "running"
  }
}
```

---

### 5.7 `sim.resume`

Resume a paused simulation.

**Request:**
```json
{
  "jsonrpc": "2.0",
  "id": 7,
  "method": "sim.resume",
  "params": {
    "speed_multiplier": 2.0
  }
}
```

**Params Schema:**
| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `speed_multiplier` | number | no | Set speed on resume; 0.1 to 1000; default = prior speed |

**Result:**
```json
{
  "jsonrpc": "2.0",
  "id": 7,
  "result": {
    "resumed_at_tick": 1440,
    "speed_multiplier": 2.0
  }
}
```

---

### 5.8 `sim.fast_forward`

Advance the simulation by a specified number of ticks as fast as possible (ignoring wall-clock pacing). Suspends broadcast during fast-forward; emits one synthetic `sim.fast_forward.completed` notification when done.

**Request:**
```json
{
  "jsonrpc": "2.0",
  "id": 8,
  "method": "sim.fast_forward",
  "params": {
    "ticks": 3600,
    "emit_milestones_every": 360,
    "resume_speed_after": 1.0
  }
}
```

**Params Schema:**
| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `ticks` | integer | yes | Number of ticks to advance |
| `emit_milestones_every` | integer | no | Emit progress notification every N ticks; null = no milestones |
| `resume_speed_after` | number | no | Speed multiplier after fast-forward completes |

**Result:**
```json
{
  "jsonrpc": "2.0",
  "id": 8,
  "result": {
    "accepted": true,
    "from_tick": 1440,
    "target_tick": 5040,
    "estimated_wall_ms": 3600
  }
}
```

---

### 5.9 `sim.scenario.load`

Load a scenario, replacing the current simulation state. Only callable when the simulation is paused or idle. Resets tick to 0.

**Request:**
```json
{
  "jsonrpc": "2.0",
  "id": 9,
  "method": "sim.scenario.load",
  "params": {
    "scenario_id": "climate_crisis_2075",
    "scenario_json": null,
    "seed": 42,
    "parameter_overrides": {
      "initial_co2_ppm": 440,
      "global_population": 9200000000
    }
  }
}
```

**Params Schema:**
| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `scenario_id` | string | one of | Load scenario from server-side registry by ID |
| `scenario_json` | object | one of | Inline scenario definition (takes precedence over `scenario_id`) |
| `seed` | integer | no | RNG seed; null = random |
| `parameter_overrides` | object | no | Override scenario default parameters |

**Result:**
```json
{
  "jsonrpc": "2.0",
  "id": 9,
  "result": {
    "run_id": "run_2026_02_21_002",
    "scenario_id": "climate_crisis_2075",
    "seed": 42,
    "tick": 0,
    "snapshot_url": "http://localhost:9876/snapshots/run_2026_02_21_002/0"
  }
}
```

---

### 5.10 `sim.metrics.query`

Query aggregated time-series metrics for any entity or the global simulation.

**Request:**
```json
{
  "jsonrpc": "2.0",
  "id": 10,
  "method": "sim.metrics.query",
  "params": {
    "metric_names": ["gdp_total", "co2_ppm", "happiness_mean", "population_total"],
    "from_tick": 0,
    "to_tick": 1440,
    "entity_id": null,
    "aggregate": "mean",
    "bucket_size_ticks": 10
  }
}
```

**Params Schema:**
| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `metric_names` | string[] | yes | Metric identifiers (see metric registry below) |
| `from_tick` | integer | yes | Query start tick (inclusive) |
| `to_tick` | integer | yes | Query end tick (inclusive) |
| `entity_id` | string\|null | no | Scope to a single entity; null = global |
| `aggregate` | enum | no | `"mean"` \| `"sum"` \| `"min"` \| `"max"` \| `"last"` |
| `bucket_size_ticks` | integer | no | Aggregation window; 1 = per-tick; default 10 |

**Result:**
```json
{
  "jsonrpc": "2.0",
  "id": 10,
  "result": {
    "run_id": "run_2026_02_21_001",
    "from_tick": 0,
    "to_tick": 1440,
    "bucket_size_ticks": 10,
    "series": {
      "gdp_total": {
        "unit": "millijoules",
        "values": [1240000000, 1241500000, 1243000000]
      },
      "co2_ppm": {
        "unit": "ppm",
        "values": [415.2, 415.4, 415.6]
      },
      "happiness_mean": {
        "unit": "dimensionless_0_1",
        "values": [0.61, 0.612, 0.609]
      },
      "population_total": {
        "unit": "persons",
        "values": [7200000000, 7200050000, 7200100000]
      }
    },
    "tick_labels": [0, 10, 20]
  }
}
```

**Supported Metric Names:**
| Metric | Unit | Description |
|--------|------|-------------|
| `gdp_total` | millijoules | Global GDP in energy units |
| `co2_ppm` | ppm | Atmospheric CO2 concentration |
| `temperature_delta_c` | celsius | Global mean temperature delta from baseline |
| `population_total` | persons | Total world population |
| `happiness_mean` | 0–1 | Population-weighted mean happiness |
| `gini_coefficient` | 0–1 | Global inequality (Gini) |
| `war_ongoing_count` | integer | Active wars |
| `food_price_mean` | millijoules/unit | Global mean food price |
| `energy_deficit_total` | millijoules | Total unmet energy demand |
| `legitimacy_mean` | 0–1 | Mean institutional legitimacy |

---

## 6. JSON-RPC Methods — Research API

### 6.1 `research.run.create`

Create a new simulation run with specified parameters. The new run starts paused at tick 0.

**Request:**
```json
{
  "jsonrpc": "2.0",
  "id": 20,
  "method": "research.run.create",
  "params": {
    "scenario_id": "baseline_2050",
    "label": "carbon_tax_sweep_rate_85",
    "description": "Carbon tax at $85/tonne from tick 0",
    "seed": 9876543210,
    "parameter_set": {
      "carbon_tax_rate": 85,
      "renewable_subsidy_pct": 0.15,
      "global_governance_strength": 0.4
    },
    "tags": {
      "experiment": "carbon_tax_sweep",
      "batch": "03"
    },
    "auto_start": true,
    "max_ticks": 36000
  }
}
```

**Params Schema:**
| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `scenario_id` | string | yes | Scenario to load |
| `label` | string | no | Human-readable label |
| `description` | string | no | Free-text description |
| `seed` | integer | no | RNG seed; null = server-generated |
| `parameter_set` | object | no | Scenario parameter overrides |
| `tags` | object | no | Arbitrary string key-value metadata |
| `auto_start` | boolean | no | Start immediately after creation; default false |
| `max_ticks` | integer | no | Auto-stop after this many ticks; null = unlimited |

**Result:**
```json
{
  "jsonrpc": "2.0",
  "id": 20,
  "result": {
    "run_id": "run_2026_02_21_003",
    "status": "running",
    "seed": 9876543210,
    "created_at_ms": 1740134400000,
    "scenario_id": "baseline_2050",
    "snapshot_url": "http://localhost:9876/snapshots/run_2026_02_21_003/0"
  }
}
```

---

### 6.2 `research.run.list`

List simulation runs with optional filters.

**Request:**
```json
{
  "jsonrpc": "2.0",
  "id": 21,
  "method": "research.run.list",
  "params": {
    "status_filter": ["completed", "running"],
    "scenario_id_filter": "baseline_2050",
    "tag_filter": { "experiment": "carbon_tax_sweep" },
    "limit": 50,
    "offset": 0,
    "order_by": "created_at_ms",
    "order_dir": "desc"
  }
}
```

**Result:**
```json
{
  "jsonrpc": "2.0",
  "id": 21,
  "result": {
    "total": 127,
    "runs": [
      {
        "run_id": "run_2026_02_21_003",
        "label": "carbon_tax_sweep_rate_85",
        "status": "running",
        "scenario_id": "baseline_2050",
        "seed": 9876543210,
        "current_tick": 1440,
        "max_ticks": 36000,
        "created_at_ms": 1740134400000,
        "tags": { "experiment": "carbon_tax_sweep", "batch": "03" }
      }
    ]
  }
}
```

---

### 6.3 `research.run.get`

Retrieve full details for a single run including final metrics summary if completed.

**Request:**
```json
{
  "jsonrpc": "2.0",
  "id": 22,
  "method": "research.run.get",
  "params": {
    "run_id": "run_2026_02_21_001"
  }
}
```

**Result:**
```json
{
  "jsonrpc": "2.0",
  "id": 22,
  "result": {
    "run_id": "run_2026_02_21_001",
    "label": "baseline_no_intervention",
    "status": "completed",
    "scenario_id": "baseline_2050",
    "seed": 9876543210,
    "parameter_set": {},
    "start_tick": 0,
    "end_tick": 36000,
    "duration_ticks": 36000,
    "created_at_ms": 1740048000000,
    "completed_at_ms": 1740134400000,
    "outcome": "completed_normally",
    "final_metrics": {
      "co2_ppm": 558.3,
      "temperature_delta_c": 2.7,
      "population_total": 8940000000,
      "happiness_mean": 0.48,
      "gini_coefficient": 0.62
    },
    "tags": {},
    "event_count": 4821330,
    "final_state_hash": "c9d8e7f6a5b4c3d2e1f0a9b8c7d6e5f4a3b2c1d0e9f8a7b6c5d4e3f2a1b0c9"
  }
}
```

---

### 6.4 `research.export.csv`

Export a metrics time series or event table as CSV. Returns a URL to the generated file.

**Request:**
```json
{
  "jsonrpc": "2.0",
  "id": 23,
  "method": "research.export.csv",
  "params": {
    "run_id": "run_2026_02_21_001",
    "table": "metrics",
    "metric_names": ["co2_ppm", "population_total", "happiness_mean"],
    "from_tick": 0,
    "to_tick": 36000,
    "bucket_size_ticks": 100,
    "include_header": true
  }
}
```

**Params Schema:**
| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `run_id` | string | yes | Run to export |
| `table` | enum | yes | `"metrics"` \| `"events"` \| `"citizens"` \| `"nations"` |
| `metric_names` | string[] | no | Metrics to include (for table=metrics) |
| `event_types` | string[] | no | Event type filter (for table=events) |
| `from_tick` | integer | no | Range start |
| `to_tick` | integer | no | Range end |
| `bucket_size_ticks` | integer | no | Aggregation window |
| `include_header` | boolean | no | Include CSV header row; default true |

**Result:**
```json
{
  "jsonrpc": "2.0",
  "id": 23,
  "result": {
    "export_id": "export_01HXR3D1A2B3C4D5",
    "download_url": "http://localhost:9876/exports/export_01HXR3D1A2B3C4D5.csv",
    "row_count": 361,
    "size_bytes": 28672,
    "expires_at_ms": 1740220800000
  }
}
```

---

### 6.5 `research.export.parquet`

Export as Apache Parquet for efficient downstream analysis with pandas, Polars, DuckDB, etc.

**Request:**
```json
{
  "jsonrpc": "2.0",
  "id": 24,
  "method": "research.export.parquet",
  "params": {
    "run_id": "run_2026_02_21_001",
    "table": "events",
    "event_types": ["economy.market_cleared.v1", "economy.energy_shortage.v1"],
    "from_tick": 0,
    "to_tick": 36000,
    "compression": "snappy",
    "row_group_size": 131072
  }
}
```

**Result:**
```json
{
  "jsonrpc": "2.0",
  "id": 24,
  "result": {
    "export_id": "export_01HXR3E1A2B3C4D5",
    "download_url": "http://localhost:9876/exports/export_01HXR3E1A2B3C4D5.parquet",
    "row_count": 248921,
    "size_bytes": 4194304,
    "schema": {
      "event_id": "string",
      "event_type": "string",
      "tick": "int64",
      "seq": "int32",
      "payload_json": "string"
    },
    "expires_at_ms": 1740220800000
  }
}
```

---

## 7. Broadcast Frame Format

### 7.1 JSON Tick Broadcast (text frame)

After each simulation tick completes, the server sends a JSON-RPC notification to all subscribed clients. The notification carries the complete set of events emitted during that tick, filtered per client subscription.

```json
{
  "jsonrpc": "2.0",
  "method": "sim.tick_broadcast",
  "params": {
    "tick": 1441,
    "run_id": "run_2026_02_21_001",
    "tick_duration_ms": 8,
    "event_count_total": 142,
    "event_count_delivered": 37,
    "state_hash": "b4c5d6e7f8a9b0c1d2e3f4a5b6c7d8e9f0a1b2c3d4e5f6a7b8c9d0e1f2a3b4",
    "events": [
      {
        "event_id": "018e6b3b-1234-7000-8000-000000000001",
        "event_type": "economy.market_cleared.v1",
        "schema_version": 1,
        "tick": 1441,
        "seq": 0,
        "run_id": "run_2026_02_21_001",
        "seed": 9876543210,
        "prev_hash": "a3f2c1d4e5b6789012345678901234567890abcdef1234567890abcdef123456",
        "emitted_at_ms": 1740134408000,
        "payload": { "..." : "..." }
      }
    ],
    "snapshot_delta": null
  }
}
```

**Broadcast Frame Fields:**
| Field | Type | Description |
|-------|------|-------------|
| `tick` | integer | Tick that just completed |
| `run_id` | string | Active run |
| `tick_duration_ms` | integer | Wall-clock ms taken to compute this tick |
| `event_count_total` | integer | Total events emitted this tick (unfiltered) |
| `event_count_delivered` | integer | Events in this frame (after client filter) |
| `state_hash` | string | BLAKE3 hash of state after this tick |
| `events` | EventEnvelopeV1[] | Filtered events for this client |
| `snapshot_delta` | object\|null | State delta (only if client subscribed with `include_snapshot_delta: true`) |

### 7.2 Binary Tick Broadcast (binary frame)

For game engine clients that requested `binary: true` in `sim.subscribe`, the broadcast is a binary WebSocket frame with the 8-byte header (see Section 2.3) followed by a MessagePack-encoded object with identical structure to the JSON broadcast.

MessagePack encoding rules:
- Integers use the shortest representation.
- Strings are UTF-8.
- The `events` array contains full `EventEnvelopeV1` objects (same fields).
- `state_hash` is a 32-byte binary fixext (MessagePack ext type 0x01).

### 7.3 Subscription Filtering

Clients specify filter patterns in `sim.subscribe`. The server evaluates each event's `event_type` against the client's filter list using glob matching:

| Pattern | Matches | Does Not Match |
|---------|---------|----------------|
| `"*"` | All events | — |
| `"economy.*"` | `economy.market_cleared.v1`, `economy.ledger_transfer.v1` | `citizen.born.v1` |
| `"economy.market_cleared.v1"` | Exact match only | `economy.market_cleared.v2` |
| `"war.*"`, `"battle.*"` | Any war or battle event | `treaty.signed.v1` |
| `"**.tipping_point.**"` | `climate.tipping_point.activated.v1` | — |

Events not matching any pattern are silently dropped for that client.

### 7.4 Fast-Forward Progress Notification

During `sim.fast_forward`, if `emit_milestones_every` is set:

```json
{
  "jsonrpc": "2.0",
  "method": "sim.fast_forward.progress",
  "params": {
    "run_id": "run_2026_02_21_001",
    "current_tick": 1800,
    "target_tick": 5040,
    "elapsed_wall_ms": 1200,
    "estimated_remaining_ms": 2400,
    "pct_complete": 10.0
  }
}
```

Completion notification:

```json
{
  "jsonrpc": "2.0",
  "method": "sim.fast_forward.completed",
  "params": {
    "run_id": "run_2026_02_21_001",
    "from_tick": 1440,
    "to_tick": 5040,
    "elapsed_wall_ms": 3600,
    "state_hash": "..."
  }
}
```

---

## 8. Event Types — Demographics

Demographics events track all changes to individual citizens and aggregate population statistics. The citizen model is agent-based: each citizen has a unique ID, class, happiness score, city of residence, and relationship graph.

### 8.1 `citizen.born.v1`

Emitted when a new citizen entity is created in the simulation.

**Trigger:** Population growth tick (runs every 10 simulation ticks).

**Payload JSON Schema:**
```json
{
  "$id": "https://civlab.dev/schemas/events/citizen.born.v1.json",
  "title": "CitizenBornV1Payload",
  "type": "object",
  "required": ["citizen_id", "city_id", "parent_ids", "class", "initial_happiness", "birth_year"],
  "properties": {
    "citizen_id": {
      "type": "string",
      "description": "Stable opaque identifier for this citizen"
    },
    "city_id": {
      "type": "string",
      "description": "City in which citizen is born"
    },
    "parent_ids": {
      "type": "array",
      "items": { "type": "string" },
      "maxItems": 2,
      "description": "Parent citizen IDs; empty for initial population seeding"
    },
    "class": {
      "type": "string",
      "enum": ["SUBSISTENCE", "WORKING", "MIDDLE", "UPPER", "ELITE"],
      "description": "Social class inherited from parents or assigned at birth"
    },
    "initial_happiness": {
      "type": "number",
      "minimum": 0.0,
      "maximum": 1.0,
      "description": "Starting happiness score [0,1]"
    },
    "birth_year": {
      "type": "integer",
      "description": "Simulated calendar year of birth"
    },
    "nation_id": {
      "type": "string",
      "description": "Nation governing the birth city"
    },
    "traits": {
      "type": "array",
      "items": { "type": "string" },
      "description": "Optional behavioral trait flags (e.g. ENTREPRENEURIAL, RISK_AVERSE)"
    }
  },
  "additionalProperties": false
}
```

**Example:**
```json
{
  "event_type": "citizen.born.v1",
  "tick": 100,
  "seq": 3,
  "payload": {
    "citizen_id": "cit_00000000000001a7",
    "city_id": "city_nairobi_001",
    "parent_ids": ["cit_000000000000004a", "cit_000000000000008c"],
    "class": "WORKING",
    "initial_happiness": 0.55,
    "birth_year": 2052,
    "nation_id": "nation_kenya",
    "traits": ["ENTREPRENEURIAL"]
  }
}
```

---

### 8.2 `citizen.died.v1`

Emitted when a citizen entity is removed from the simulation.

**Payload JSON Schema:**
```json
{
  "$id": "https://civlab.dev/schemas/events/citizen.died.v1.json",
  "title": "CitizenDiedV1Payload",
  "type": "object",
  "required": ["citizen_id", "city_id", "cause", "age_ticks", "class_at_death"],
  "properties": {
    "citizen_id": { "type": "string" },
    "city_id": { "type": "string" },
    "nation_id": { "type": "string" },
    "cause": {
      "type": "string",
      "enum": ["HUNGER", "DISEASE", "COMBAT", "OLD_AGE", "CLIMATE_EVENT", "POLLUTION"],
      "description": "Primary cause of death"
    },
    "age_ticks": {
      "type": "integer",
      "minimum": 0,
      "description": "Age of citizen in simulation ticks at death"
    },
    "class_at_death": {
      "type": "string",
      "enum": ["SUBSISTENCE", "WORKING", "MIDDLE", "UPPER", "ELITE"]
    },
    "happiness_at_death": {
      "type": "number",
      "minimum": 0.0,
      "maximum": 1.0
    },
    "assets_redistributed_to": {
      "type": "string",
      "description": "Heir citizen_id or 'STATE' if no heirs"
    }
  },
  "additionalProperties": false
}
```

**Example:**
```json
{
  "event_type": "citizen.died.v1",
  "tick": 1440,
  "seq": 22,
  "payload": {
    "citizen_id": "cit_000000000000004a",
    "city_id": "city_nairobi_001",
    "nation_id": "nation_kenya",
    "cause": "OLD_AGE",
    "age_ticks": 8700,
    "class_at_death": "MIDDLE",
    "happiness_at_death": 0.63,
    "assets_redistributed_to": "cit_00000000000001a7"
  }
}
```

---

### 8.3 `citizen.migrated.v1`

Emitted when a citizen changes city of residence.

**Payload JSON Schema:**
```json
{
  "$id": "https://civlab.dev/schemas/events/citizen.migrated.v1.json",
  "title": "CitizenMigratedV1Payload",
  "type": "object",
  "required": ["citizen_id", "from_city_id", "to_city_id", "from_nation_id", "to_nation_id", "reason"],
  "properties": {
    "citizen_id": { "type": "string" },
    "from_city_id": { "type": "string" },
    "to_city_id": { "type": "string" },
    "from_nation_id": { "type": "string" },
    "to_nation_id": { "type": "string" },
    "reason": {
      "type": "string",
      "enum": ["HAPPINESS", "JOB_OPPORTUNITY", "SAFETY", "CLIMATE_DISPLACEMENT", "POLICY_INCENTIVE", "FAMILY"],
      "description": "Primary migration motivation"
    },
    "happiness_at_origin": { "type": "number", "minimum": 0.0, "maximum": 1.0 },
    "happiness_at_destination": { "type": "number", "minimum": 0.0, "maximum": 1.0 },
    "journey_cost_millijoules": {
      "type": "integer",
      "minimum": 0,
      "description": "Economic cost of migration (deducted from citizen assets)"
    }
  },
  "additionalProperties": false
}
```

---

### 8.4 `citizen.class_changed.v1`

Emitted when a citizen's social class changes (upward or downward mobility).

**Payload JSON Schema:**
```json
{
  "$id": "https://civlab.dev/schemas/events/citizen.class_changed.v1.json",
  "title": "CitizenClassChangedV1Payload",
  "type": "object",
  "required": ["citizen_id", "city_id", "old_class", "new_class", "trigger", "direction"],
  "properties": {
    "citizen_id": { "type": "string" },
    "city_id": { "type": "string" },
    "nation_id": { "type": "string" },
    "old_class": {
      "type": "string",
      "enum": ["SUBSISTENCE", "WORKING", "MIDDLE", "UPPER", "ELITE"]
    },
    "new_class": {
      "type": "string",
      "enum": ["SUBSISTENCE", "WORKING", "MIDDLE", "UPPER", "ELITE"]
    },
    "direction": {
      "type": "string",
      "enum": ["UPWARD", "DOWNWARD"],
      "description": "Direction of class mobility"
    },
    "trigger": {
      "type": "string",
      "enum": [
        "INCOME_CHANGE", "UNEMPLOYMENT", "INHERITANCE", "POLICY", "DISASTER",
        "EDUCATION_COMPLETION", "BUSINESS_SUCCESS", "BUSINESS_FAILURE"
      ]
    },
    "income_before_millijoules": { "type": "integer" },
    "income_after_millijoules": { "type": "integer" }
  },
  "additionalProperties": false
}
```

---

### 8.5 `citizen.happiness_updated.v1`

Emitted when a citizen's happiness score changes by more than the configured threshold (default: ±0.05).

**Payload JSON Schema:**
```json
{
  "$id": "https://civlab.dev/schemas/events/citizen.happiness_updated.v1.json",
  "title": "CitizenHappinessUpdatedV1Payload",
  "type": "object",
  "required": ["citizen_id", "city_id", "old_value", "new_value", "delta", "reasons"],
  "properties": {
    "citizen_id": { "type": "string" },
    "city_id": { "type": "string" },
    "old_value": { "type": "number", "minimum": 0.0, "maximum": 1.0 },
    "new_value": { "type": "number", "minimum": 0.0, "maximum": 1.0 },
    "delta": {
      "type": "number",
      "description": "Signed delta (new_value - old_value)"
    },
    "reasons": {
      "type": "array",
      "items": {
        "type": "object",
        "required": ["factor", "contribution"],
        "properties": {
          "factor": {
            "type": "string",
            "enum": [
              "FOOD_SECURITY", "SAFETY", "INCOME", "HEALTHCARE",
              "ENVIRONMENT", "FREEDOM", "COMMUNITY", "CLIMATE_STRESS",
              "WAR", "POLICY_CHANGE"
            ]
          },
          "contribution": {
            "type": "number",
            "description": "Signed contribution of this factor to the delta"
          }
        }
      },
      "description": "Decomposition of the happiness change"
    }
  },
  "additionalProperties": false
}
```

**Example:**
```json
{
  "event_type": "citizen.happiness_updated.v1",
  "tick": 1441,
  "seq": 15,
  "payload": {
    "citizen_id": "cit_00000000000001a7",
    "city_id": "city_nairobi_001",
    "old_value": 0.55,
    "new_value": 0.49,
    "delta": -0.06,
    "reasons": [
      { "factor": "FOOD_SECURITY", "contribution": -0.04 },
      { "factor": "INCOME", "contribution": -0.03 },
      { "factor": "COMMUNITY", "contribution": 0.01 }
    ]
  }
}
```

---

## 9. Event Types — Economy

Economy events track all market activity, financial flows, and resource allocation decisions. All monetary values are denominated in **millijoules** — a unified energy-equivalent currency used throughout the simulation.

### 9.1 `economy.market_cleared.v1`

Emitted once per market per tick when the goods market clears (Walrasian auction).

**Payload JSON Schema:**
```json
{
  "$id": "https://civlab.dev/schemas/events/economy.market_cleared.v1.json",
  "title": "EconomyMarketClearedV1Payload",
  "type": "object",
  "required": ["good", "city_id", "clearing_price", "bid_volume", "ask_volume", "unmet_demand", "num_buyers", "num_sellers"],
  "properties": {
    "good": {
      "type": "string",
      "enum": ["FOOD", "ENERGY", "MANUFACTURED_GOODS", "LUXURY_GOODS", "RAW_MATERIALS", "SERVICES", "INFORMATION"],
      "description": "Good that was traded"
    },
    "city_id": {
      "type": "string",
      "description": "City market where clearing occurred"
    },
    "nation_id": { "type": "string" },
    "clearing_price": {
      "type": "integer",
      "minimum": 0,
      "description": "Equilibrium price in millijoules per unit"
    },
    "bid_volume": {
      "type": "integer",
      "minimum": 0,
      "description": "Total quantity demanded (buyer bids)"
    },
    "ask_volume": {
      "type": "integer",
      "minimum": 0,
      "description": "Total quantity supplied (seller asks)"
    },
    "traded_volume": {
      "type": "integer",
      "minimum": 0,
      "description": "Quantity actually transacted = min(bid_volume, ask_volume)"
    },
    "unmet_demand": {
      "type": "integer",
      "minimum": 0,
      "description": "Demand that could not be met (bid_volume - traded_volume)"
    },
    "prev_clearing_price": {
      "type": "integer",
      "minimum": 0,
      "description": "Clearing price from the previous tick"
    },
    "price_change_pct": {
      "type": "number",
      "description": "Percentage change from prev_clearing_price"
    },
    "num_buyers": { "type": "integer", "minimum": 0 },
    "num_sellers": { "type": "integer", "minimum": 0 }
  },
  "additionalProperties": false
}
```

**Example:**
```json
{
  "event_type": "economy.market_cleared.v1",
  "tick": 1441,
  "seq": 0,
  "payload": {
    "good": "FOOD",
    "city_id": "city_lagos_001",
    "nation_id": "nation_nigeria",
    "clearing_price": 142,
    "bid_volume": 50000,
    "ask_volume": 48500,
    "traded_volume": 48500,
    "unmet_demand": 1500,
    "prev_clearing_price": 138,
    "price_change_pct": 2.9,
    "num_buyers": 12340,
    "num_sellers": 890
  }
}
```

---

### 9.2 `economy.ledger_transfer.v1`

Emitted for every monetary transfer between actors (citizens, nations, firms, international bodies).

**Payload JSON Schema:**
```json
{
  "$id": "https://civlab.dev/schemas/events/economy.ledger_transfer.v1.json",
  "title": "EconomyLedgerTransferV1Payload",
  "type": "object",
  "required": ["transfer_id", "from_actor", "to_actor", "amount_millijoules", "transfer_type"],
  "properties": {
    "transfer_id": {
      "type": "string",
      "description": "Unique identifier for this transfer"
    },
    "from_actor": {
      "type": "string",
      "description": "Sending entity ID (citizen, nation, firm)"
    },
    "from_actor_type": {
      "type": "string",
      "enum": ["CITIZEN", "NATION", "FIRM", "INTERNATIONAL_ORG", "SIMULATION"]
    },
    "to_actor": {
      "type": "string",
      "description": "Receiving entity ID"
    },
    "to_actor_type": {
      "type": "string",
      "enum": ["CITIZEN", "NATION", "FIRM", "INTERNATIONAL_ORG", "SIMULATION"]
    },
    "amount_millijoules": {
      "type": "integer",
      "minimum": 1,
      "description": "Transfer amount in millijoules (always positive)"
    },
    "transfer_type": {
      "type": "string",
      "enum": [
        "TRADE", "TAX", "SUBSIDY", "AID", "WAGE", "REPARATION",
        "INVESTMENT", "DEBT_REPAYMENT", "FINE", "CONFISCATION"
      ]
    },
    "city_id": {
      "type": ["string", "null"],
      "description": "City context if applicable"
    },
    "policy_id": {
      "type": ["string", "null"],
      "description": "Policy that triggered this transfer"
    }
  },
  "additionalProperties": false
}
```

---

### 9.3 `economy.energy_shortage.v1`

Emitted when a city or nation cannot meet its energy demand. Triggers cascade effects on manufacturing, food production, and citizen happiness.

**Payload JSON Schema:**
```json
{
  "$id": "https://civlab.dev/schemas/events/economy.energy_shortage.v1.json",
  "title": "EconomyEnergyShortageV1Payload",
  "type": "object",
  "required": ["scope_id", "scope_type", "deficit_millijoules", "demand_millijoules", "supply_millijoules", "affected_cities", "penalty_pct"],
  "properties": {
    "scope_id": {
      "type": "string",
      "description": "City or nation ID experiencing shortage"
    },
    "scope_type": {
      "type": "string",
      "enum": ["CITY", "NATION", "REGION"]
    },
    "deficit_millijoules": {
      "type": "integer",
      "minimum": 0,
      "description": "Unmet energy demand in millijoules"
    },
    "demand_millijoules": { "type": "integer", "minimum": 0 },
    "supply_millijoules": { "type": "integer", "minimum": 0 },
    "affected_cities": {
      "type": "array",
      "items": { "type": "string" },
      "description": "List of city IDs affected by this shortage"
    },
    "penalty_pct": {
      "type": "number",
      "minimum": 0,
      "maximum": 100,
      "description": "Productivity penalty applied to affected cities (% reduction)"
    },
    "cause": {
      "type": "string",
      "enum": ["GRID_FAILURE", "SUPPLY_DISRUPTION", "DEMAND_SPIKE", "WEATHER", "WAR_DAMAGE"],
      "description": "Primary cause of shortage"
    },
    "shortage_duration_ticks": {
      "type": "integer",
      "minimum": 1,
      "description": "Projected duration based on current projections"
    }
  },
  "additionalProperties": false
}
```

---

### 9.4 `economy.price_spike.v1`

Emitted when a good's price in a city changes by more than the configured spike threshold (default: ±20% in one tick).

**Payload JSON Schema:**
```json
{
  "$id": "https://civlab.dev/schemas/events/economy.price_spike.v1.json",
  "title": "EconomyPriceSpikeV1Payload",
  "type": "object",
  "required": ["good", "city_id", "old_price", "new_price", "pct_change", "spike_type"],
  "properties": {
    "good": {
      "type": "string",
      "enum": ["FOOD", "ENERGY", "MANUFACTURED_GOODS", "LUXURY_GOODS", "RAW_MATERIALS", "SERVICES", "INFORMATION"]
    },
    "city_id": { "type": "string" },
    "nation_id": { "type": "string" },
    "old_price": { "type": "integer", "minimum": 0 },
    "new_price": { "type": "integer", "minimum": 0 },
    "pct_change": {
      "type": "number",
      "description": "Signed percentage change ((new - old) / old * 100)"
    },
    "spike_type": {
      "type": "string",
      "enum": ["SUPPLY_SHOCK", "DEMAND_SHOCK", "SPECULATION", "TRADE_DISRUPTION", "CLIMATE_IMPACT"]
    },
    "contributing_factors": {
      "type": "array",
      "items": { "type": "string" },
      "description": "List of causal factor codes"
    }
  },
  "additionalProperties": false
}
```

---

### 9.5 `economy.trade_route.established.v1`

Emitted when a new trade route is opened between two cities or nations.

**Payload JSON Schema:**
```json
{
  "$id": "https://civlab.dev/schemas/events/economy.trade_route.established.v1.json",
  "title": "EconomyTradeRouteEstablishedV1Payload",
  "type": "object",
  "required": ["route_id", "from_city_id", "to_city_id", "from_nation_id", "to_nation_id", "goods", "initial_volume_millijoules"],
  "properties": {
    "route_id": { "type": "string" },
    "from_city_id": { "type": "string" },
    "to_city_id": { "type": "string" },
    "from_nation_id": { "type": "string" },
    "to_nation_id": { "type": "string" },
    "goods": {
      "type": "array",
      "items": {
        "type": "string",
        "enum": ["FOOD", "ENERGY", "MANUFACTURED_GOODS", "LUXURY_GOODS", "RAW_MATERIALS", "SERVICES", "INFORMATION"]
      },
      "description": "Goods traded on this route"
    },
    "initial_volume_millijoules": {
      "type": "integer",
      "minimum": 0,
      "description": "Projected monthly trade volume"
    },
    "transport_type": {
      "type": "string",
      "enum": ["SEA", "RAIL", "AIR", "PIPELINE", "DIGITAL"]
    },
    "establishment_trigger": {
      "type": "string",
      "enum": ["TREATY", "MARKET_INCENTIVE", "POLICY", "PLAYER_COMMAND"]
    }
  },
  "additionalProperties": false
}
```

---

### 9.6 `economy.trade_route.broken.v1`

Emitted when an existing trade route is disrupted or terminated.

**Payload JSON Schema:**
```json
{
  "$id": "https://civlab.dev/schemas/events/economy.trade_route.broken.v1.json",
  "title": "EconomyTradeRouteBrokenV1Payload",
  "type": "object",
  "required": ["route_id", "from_city_id", "to_city_id", "reason", "lost_volume_millijoules"],
  "properties": {
    "route_id": { "type": "string" },
    "from_city_id": { "type": "string" },
    "to_city_id": { "type": "string" },
    "from_nation_id": { "type": "string" },
    "to_nation_id": { "type": "string" },
    "reason": {
      "type": "string",
      "enum": ["WAR", "SANCTIONS", "INFRASTRUCTURE_DAMAGE", "POLITICAL_DISPUTE", "CLIMATE_BARRIER", "BANKRUPTCY"]
    },
    "lost_volume_millijoules": { "type": "integer", "minimum": 0 },
    "expected_restoration_tick": {
      "type": ["integer", "null"],
      "description": "Estimated tick of restoration; null if indeterminate"
    }
  },
  "additionalProperties": false
}
```

---

## 10. Event Types — Climate

Climate events capture changes to the planetary climate system, including CO2 concentration, temperature, tipping points, and extreme weather.

### 10.1 `climate.co2_threshold_crossed.v1`

Emitted when atmospheric CO2 crosses a significant milestone (e.g., 450 ppm, 500 ppm).

**Payload JSON Schema:**
```json
{
  "$id": "https://civlab.dev/schemas/events/climate.co2_threshold_crossed.v1.json",
  "title": "ClimateCo2ThresholdCrossedV1Payload",
  "type": "object",
  "required": ["threshold_ppm", "current_ppm", "prev_ppm", "temp_delta_c", "crossing_direction", "predefined_thresholds_remaining"],
  "properties": {
    "threshold_ppm": {
      "type": "number",
      "description": "The milestone CO2 concentration crossed"
    },
    "current_ppm": {
      "type": "number",
      "description": "Actual current CO2 concentration"
    },
    "prev_ppm": {
      "type": "number",
      "description": "CO2 concentration at previous tick"
    },
    "temp_delta_c": {
      "type": "number",
      "description": "Current global mean temperature delta from pre-industrial baseline"
    },
    "crossing_direction": {
      "type": "string",
      "enum": ["UPWARD", "DOWNWARD"],
      "description": "Whether concentration is rising or falling through the threshold"
    },
    "ticks_to_reach": {
      "type": "integer",
      "minimum": 0,
      "description": "Number of ticks since sim start to reach this threshold"
    },
    "predefined_thresholds_remaining": {
      "type": "array",
      "items": { "type": "number" },
      "description": "Upcoming thresholds in crossing direction"
    },
    "annual_emission_rate_gtco2": {
      "type": "number",
      "description": "Current global CO2 emissions in Gt CO2/year"
    }
  },
  "additionalProperties": false
}
```

**Example:**
```json
{
  "event_type": "climate.co2_threshold_crossed.v1",
  "tick": 1800,
  "seq": 0,
  "payload": {
    "threshold_ppm": 450,
    "current_ppm": 450.3,
    "prev_ppm": 449.8,
    "temp_delta_c": 1.8,
    "crossing_direction": "UPWARD",
    "ticks_to_reach": 1800,
    "predefined_thresholds_remaining": [500, 550, 600],
    "annual_emission_rate_gtco2": 38.2
  }
}
```

---

### 10.2 `climate.tipping_point.activated.v1`

Emitted when a climate tipping point activates. Tipping points cause irreversible cascade effects that may activate additional tipping points.

**Payload JSON Schema:**
```json
{
  "$id": "https://civlab.dev/schemas/events/climate.tipping_point.activated.v1.json",
  "title": "ClimateTippingPointActivatedV1Payload",
  "type": "object",
  "required": ["point_id", "point_type", "activation_trigger", "cascade_risk", "co2_at_activation", "temp_at_activation"],
  "properties": {
    "point_id": {
      "type": "string",
      "description": "Unique identifier for this tipping point instance"
    },
    "point_type": {
      "type": "string",
      "enum": [
        "ARCTIC_SEA_ICE", "AMAZON_DIEBACK", "PERMAFROST_METHANE",
        "WEST_ANTARCTIC_ICE", "GREENLAND_ICE", "ATLANTIC_CIRCULATION",
        "SAHEL_GREENING", "BOREAL_FOREST", "CORAL_REEFS"
      ]
    },
    "activation_trigger": {
      "type": "string",
      "enum": ["TEMPERATURE_THRESHOLD", "CO2_THRESHOLD", "CASCADE_FROM_OTHER_TIPPING_POINT"]
    },
    "cascade_risk": {
      "type": "number",
      "minimum": 0.0,
      "maximum": 1.0,
      "description": "Probability that this activation triggers another tipping point within 1000 ticks"
    },
    "co2_at_activation": { "type": "number", "description": "CO2 ppm when activated" },
    "temp_at_activation": { "type": "number", "description": "Temperature delta C when activated" },
    "expected_additional_warming_c": {
      "type": "number",
      "description": "Projected additional warming from this tipping point alone"
    },
    "irreversible": {
      "type": "boolean",
      "description": "Whether this tipping point can be reversed"
    },
    "triggered_by_point_id": {
      "type": ["string", "null"],
      "description": "ID of tipping point that cascaded into this one"
    }
  },
  "additionalProperties": false
}
```

---

### 10.3 `climate.extreme_event.v1`

Emitted when an extreme weather event occurs.

**Payload JSON Schema:**
```json
{
  "$id": "https://civlab.dev/schemas/events/climate.extreme_event.v1.json",
  "title": "ClimateExtremeEventV1Payload",
  "type": "object",
  "required": ["event_id", "event_type", "severity", "affected_cells", "duration_ticks", "economic_damage_millijoules"],
  "properties": {
    "event_id": {
      "type": "string",
      "description": "Unique identifier for this extreme event instance"
    },
    "event_type": {
      "type": "string",
      "enum": ["FLOOD", "DROUGHT", "HEATWAVE", "HURRICANE", "WILDFIRE", "BLIZZARD", "STORM_SURGE", "LANDSLIDE"]
    },
    "severity": {
      "type": "integer",
      "minimum": 1,
      "maximum": 5,
      "description": "1 (minor) to 5 (catastrophic)"
    },
    "affected_cells": {
      "type": "array",
      "items": {
        "type": "object",
        "required": ["cell_id", "lat", "lon"],
        "properties": {
          "cell_id": { "type": "string" },
          "lat": { "type": "number" },
          "lon": { "type": "number" },
          "impact_severity": { "type": "integer", "minimum": 1, "maximum": 5 }
        }
      },
      "description": "Grid cells affected by this event"
    },
    "affected_cities": {
      "type": "array",
      "items": { "type": "string" },
      "description": "City IDs impacted"
    },
    "duration_ticks": {
      "type": "integer",
      "minimum": 1,
      "description": "Expected duration of event"
    },
    "displaced_citizens": { "type": "integer", "minimum": 0 },
    "casualties": { "type": "integer", "minimum": 0 },
    "economic_damage_millijoules": { "type": "integer", "minimum": 0 },
    "infrastructure_damage_pct": {
      "type": "number",
      "minimum": 0.0,
      "maximum": 100.0,
      "description": "Percentage of infrastructure in affected area damaged"
    }
  },
  "additionalProperties": false
}
```

---

### 10.4 `climate.sea_level.rise.v1`

Emitted on each simulation year tick when sea level changes measurably.

**Payload JSON Schema:**
```json
{
  "$id": "https://civlab.dev/schemas/events/climate.sea_level.rise.v1.json",
  "title": "ClimateSeaLevelRiseV1Payload",
  "type": "object",
  "required": ["cumulative_rise_cm", "rise_this_period_cm", "submerged_cells", "at_risk_cells", "displaced_citizens"],
  "properties": {
    "cumulative_rise_cm": {
      "type": "number",
      "minimum": 0,
      "description": "Total sea level rise since simulation start"
    },
    "rise_this_period_cm": {
      "type": "number",
      "description": "Rise since last emission of this event"
    },
    "submerged_cells": {
      "type": "array",
      "items": {
        "type": "object",
        "required": ["cell_id", "lat", "lon"],
        "properties": {
          "cell_id": { "type": "string" },
          "lat": { "type": "number" },
          "lon": { "type": "number" },
          "former_land_use": {
            "type": "string",
            "enum": ["RESIDENTIAL", "AGRICULTURAL", "INDUSTRIAL", "NATURAL", "INFRASTRUCTURE"]
          }
        }
      },
      "description": "Cells newly submerged in this period"
    },
    "at_risk_cells": {
      "type": "array",
      "items": { "type": "string" },
      "description": "Cell IDs projected to submerge within 100 ticks"
    },
    "displaced_citizens": {
      "type": "integer",
      "minimum": 0,
      "description": "Citizens displaced by new submergence"
    },
    "coastal_cities_at_risk": {
      "type": "array",
      "items": { "type": "string" },
      "description": "City IDs with > 10% land area now submerged"
    }
  },
  "additionalProperties": false
}
```

---

## 11. Event Types — Political & Institutional

### 11.1 `institution.legitimacy_changed.v1`

Emitted when an institution's legitimacy score changes by more than the configured threshold (default: ±0.05).

**Payload JSON Schema:**
```json
{
  "$id": "https://civlab.dev/schemas/events/institution.legitimacy_changed.v1.json",
  "title": "InstitutionLegitimacyChangedV1Payload",
  "type": "object",
  "required": ["inst_id", "inst_type", "nation_id", "old_value", "new_value", "delta", "cause"],
  "properties": {
    "inst_id": { "type": "string" },
    "inst_type": {
      "type": "string",
      "enum": ["GOVERNMENT", "JUDICIARY", "MILITARY", "POLICE", "MEDIA", "RELIGIOUS", "FINANCIAL", "INTERNATIONAL"]
    },
    "nation_id": { "type": "string" },
    "old_value": { "type": "number", "minimum": 0.0, "maximum": 1.0 },
    "new_value": { "type": "number", "minimum": 0.0, "maximum": 1.0 },
    "delta": { "type": "number" },
    "cause": {
      "type": "string",
      "enum": [
        "CORRUPTION_EXPOSED", "POLICY_SUCCESS", "POLICY_FAILURE",
        "ELECTORAL_OUTCOME", "ECONOMIC_PERFORMANCE", "MILITARY_OUTCOME",
        "DISASTER_RESPONSE", "MEDIA_COVERAGE", "FOREIGN_PRESSURE"
      ]
    },
    "affected_population_pct": {
      "type": "number",
      "minimum": 0.0,
      "maximum": 100.0,
      "description": "Percentage of national population that holds this legitimacy view"
    }
  },
  "additionalProperties": false
}
```

---

### 11.2 `institution.formed.v1`

Emitted when a new institution is created.

**Payload JSON Schema:**
```json
{
  "$id": "https://civlab.dev/schemas/events/institution.formed.v1.json",
  "title": "InstitutionFormedV1Payload",
  "type": "object",
  "required": ["inst_id", "inst_type", "nation_id", "initial_legitimacy", "founders", "formation_trigger"],
  "properties": {
    "inst_id": { "type": "string" },
    "inst_type": {
      "type": "string",
      "enum": ["GOVERNMENT", "JUDICIARY", "MILITARY", "POLICE", "MEDIA", "RELIGIOUS", "FINANCIAL", "INTERNATIONAL"]
    },
    "nation_id": { "type": "string" },
    "initial_legitimacy": { "type": "number", "minimum": 0.0, "maximum": 1.0 },
    "founders": {
      "type": "array",
      "items": { "type": "string" },
      "description": "Citizen or faction IDs who founded the institution"
    },
    "formation_trigger": {
      "type": "string",
      "enum": ["REVOLUTION", "TREATY", "ELECTION", "COUP", "PLAYER_COMMAND", "ORGANIC_GROWTH"]
    },
    "replaces_inst_id": {
      "type": ["string", "null"],
      "description": "ID of institution this replaces (if applicable)"
    }
  },
  "additionalProperties": false
}
```

---

### 11.3 `institution.dissolved.v1`

Emitted when an institution ceases to function.

**Payload JSON Schema:**
```json
{
  "$id": "https://civlab.dev/schemas/events/institution.dissolved.v1.json",
  "title": "InstitutionDissolvedV1Payload",
  "type": "object",
  "required": ["inst_id", "inst_type", "nation_id", "reason", "legitimacy_at_dissolution"],
  "properties": {
    "inst_id": { "type": "string" },
    "inst_type": { "type": "string" },
    "nation_id": { "type": "string" },
    "reason": {
      "type": "string",
      "enum": [
        "LEGITIMACY_COLLAPSE", "REVOLUTION", "MILITARY_DEFEAT",
        "TREATY", "BANKRUPTCY", "PLAYER_COMMAND", "MERGER"
      ]
    },
    "legitimacy_at_dissolution": { "type": "number", "minimum": 0.0, "maximum": 1.0 },
    "assets_redistributed_to": {
      "type": ["string", "null"],
      "description": "Entity that received dissolved institution's assets"
    },
    "successor_inst_id": {
      "type": ["string", "null"],
      "description": "ID of replacement institution if applicable"
    }
  },
  "additionalProperties": false
}
```

---

### 11.4 `policy.applied.v1`

Emitted when a policy is executed in the simulation (each policy is applied exactly once at its effective tick).

**Payload JSON Schema:**
```json
{
  "$id": "https://civlab.dev/schemas/events/policy.applied.v1.json",
  "title": "PolicyAppliedV1Payload",
  "type": "object",
  "required": ["policy_id", "policy_type", "nation_id", "issuing_entity", "effects"],
  "properties": {
    "policy_id": { "type": "string" },
    "policy_type": {
      "type": "string",
      "enum": [
        "CARBON_TAX", "INCOME_TAX", "TRADE_TARIFF", "SUBSIDY",
        "CONSCRIPTION", "NATIONALIZATION", "PRIVATIZATION",
        "PROPAGANDA", "CENSORSHIP", "AID_PROGRAM", "SANCTIONS",
        "INFRASTRUCTURE_INVESTMENT", "EDUCATION_REFORM", "HEALTHCARE_REFORM"
      ]
    },
    "nation_id": { "type": "string" },
    "issuing_entity": {
      "type": "string",
      "description": "Institution or player that issued this policy"
    },
    "scope": {
      "type": "string",
      "enum": ["NATIONAL", "CITY", "SECTOR", "CLASS"],
      "description": "Geographic or social scope of the policy"
    },
    "scope_ids": {
      "type": "array",
      "items": { "type": "string" },
      "description": "City IDs, sector codes, or class names in scope"
    },
    "effects": {
      "type": "array",
      "items": {
        "type": "object",
        "required": ["effect_type", "magnitude"],
        "properties": {
          "effect_type": { "type": "string" },
          "magnitude": { "type": "number" },
          "unit": { "type": "string" }
        }
      }
    },
    "duration_ticks": {
      "type": ["integer", "null"],
      "description": "Duration of policy effect; null = permanent"
    },
    "command_id": {
      "type": ["string", "null"],
      "description": "Originating sim.command ID if player-issued"
    }
  },
  "additionalProperties": false
}
```

---

### 11.5 `election.occurred.v1`

Emitted when a national election completes and a new government forms.

**Payload JSON Schema:**
```json
{
  "$id": "https://civlab.dev/schemas/events/election.occurred.v1.json",
  "title": "ElectionOccurredV1Payload",
  "type": "object",
  "required": ["nation_id", "election_type", "winner_faction", "vote_shares", "turnout_pct", "outgoing_faction"],
  "properties": {
    "nation_id": { "type": "string" },
    "election_type": {
      "type": "string",
      "enum": ["GENERAL", "MIDTERM", "SNAP", "RUNOFF", "REFERENDUM"]
    },
    "winner_faction": {
      "type": "string",
      "description": "Faction ID of winning party"
    },
    "vote_shares": {
      "type": "object",
      "additionalProperties": { "type": "number", "minimum": 0.0, "maximum": 100.0 },
      "description": "Map of faction_id -> vote share percentage"
    },
    "turnout_pct": { "type": "number", "minimum": 0.0, "maximum": 100.0 },
    "outgoing_faction": { "type": ["string", "null"] },
    "coalition": {
      "type": "array",
      "items": { "type": "string" },
      "description": "Factions in governing coalition"
    },
    "legitimacy_impact": { "type": "number" },
    "contested": { "type": "boolean" }
  },
  "additionalProperties": false
}
```

---

## 12. Event Types — War & Diplomacy

### 12.1 `war.declared.v1`

Emitted immediately when one nation declares war on another.

**Payload JSON Schema:**
```json
{
  "$id": "https://civlab.dev/schemas/events/war.declared.v1.json",
  "title": "WarDeclaredV1Payload",
  "type": "object",
  "required": ["war_id", "attacker_nation", "defender_nation", "casus_belli", "attacker_strength", "defender_strength"],
  "properties": {
    "war_id": {
      "type": "string",
      "description": "Unique identifier for this war instance"
    },
    "attacker_nation": { "type": "string" },
    "defender_nation": { "type": "string" },
    "casus_belli": {
      "type": "string",
      "enum": [
        "TERRITORIAL_DISPUTE", "RESOURCE_CONTROL", "IDEOLOGICAL",
        "REVENGE", "PREEMPTIVE", "ALLY_DEFENSE", "RELIGIOUS",
        "ECONOMIC_COERCION", "SHADOW_NETWORK"
      ],
      "description": "Stated justification for war"
    },
    "attacker_strength": {
      "type": "number",
      "minimum": 0,
      "description": "Relative military strength index of attacker"
    },
    "defender_strength": {
      "type": "number",
      "minimum": 0,
      "description": "Relative military strength index of defender"
    },
    "initial_strength_ratio": {
      "type": "number",
      "description": "attacker_strength / defender_strength"
    },
    "allied_attackers": {
      "type": "array",
      "items": { "type": "string" },
      "description": "Nation IDs allied with attacker"
    },
    "allied_defenders": {
      "type": "array",
      "items": { "type": "string" },
      "description": "Nation IDs allied with defender"
    },
    "contested_territory": {
      "type": "array",
      "items": { "type": "string" },
      "description": "Cell IDs that are the primary war objective"
    },
    "expected_duration_ticks": {
      "type": "integer",
      "description": "Model's estimated war duration"
    }
  },
  "additionalProperties": false
}
```

---

### 12.2 `war.ended.v1`

Emitted when a war concludes, whether by victory, negotiated peace, or mutual exhaustion.

**Payload JSON Schema:**
```json
{
  "$id": "https://civlab.dev/schemas/events/war.ended.v1.json",
  "title": "WarEndedV1Payload",
  "type": "object",
  "required": ["war_id", "outcome", "winner", "loser", "duration_ticks", "attacker_casualties", "defender_casualties"],
  "properties": {
    "war_id": { "type": "string" },
    "outcome": {
      "type": "string",
      "enum": ["ATTACKER_VICTORY", "DEFENDER_VICTORY", "STALEMATE", "NEGOTIATED_PEACE", "MUTUAL_EXHAUSTION"]
    },
    "winner": {
      "type": ["string", "null"],
      "description": "Winning nation ID; null for stalemate/exhaustion"
    },
    "loser": {
      "type": ["string", "null"],
      "description": "Losing nation ID; null for stalemate/exhaustion"
    },
    "duration_ticks": { "type": "integer", "minimum": 0 },
    "attacker_casualties": {
      "type": "integer",
      "minimum": 0,
      "description": "Citizen deaths on attacker side"
    },
    "defender_casualties": {
      "type": "integer",
      "minimum": 0,
      "description": "Citizen deaths on defender side"
    },
    "economic_damage_attacker_millijoules": { "type": "integer", "minimum": 0 },
    "economic_damage_defender_millijoules": { "type": "integer", "minimum": 0 },
    "terms": {
      "type": "array",
      "items": {
        "type": "object",
        "required": ["term_type", "value"],
        "properties": {
          "term_type": {
            "type": "string",
            "enum": ["TERRITORY_TRANSFER", "REPARATIONS", "TRADE_AGREEMENT", "MILITARY_LIMITATION", "ALLIANCE"]
          },
          "value": {},
          "from_nation": { "type": "string" },
          "to_nation": { "type": "string" }
        }
      },
      "description": "Peace terms agreed or imposed"
    },
    "total_battles": { "type": "integer", "minimum": 0 }
  },
  "additionalProperties": false
}
```

---

### 12.3 `battle.resolved.v1`

Emitted after each individual battle within a war resolves.

**Payload JSON Schema:**
```json
{
  "$id": "https://civlab.dev/schemas/events/battle.resolved.v1.json",
  "title": "BattleResolvedV1Payload",
  "type": "object",
  "required": ["battle_id", "war_id", "location_cell_id", "attacker_nation", "defender_nation", "attacker_force", "defender_force", "outcome"],
  "properties": {
    "battle_id": { "type": "string" },
    "war_id": { "type": "string" },
    "location_cell_id": {
      "type": "string",
      "description": "Grid cell where battle occurred"
    },
    "location_city_id": {
      "type": ["string", "null"],
      "description": "City ID if battle was over a city"
    },
    "attacker_nation": { "type": "string" },
    "defender_nation": { "type": "string" },
    "attacker_force": {
      "type": "object",
      "required": ["unit_count", "strength_before"],
      "properties": {
        "unit_count": { "type": "integer", "minimum": 0 },
        "strength_before": { "type": "number" },
        "strength_after": { "type": "number" },
        "casualties": { "type": "integer", "minimum": 0 }
      }
    },
    "defender_force": {
      "type": "object",
      "required": ["unit_count", "strength_before"],
      "properties": {
        "unit_count": { "type": "integer", "minimum": 0 },
        "strength_before": { "type": "number" },
        "strength_after": { "type": "number" },
        "casualties": { "type": "integer", "minimum": 0 }
      }
    },
    "outcome": {
      "type": "string",
      "enum": ["ATTACKER_VICTORY", "DEFENDER_VICTORY", "DRAW", "ATTACKER_RETREAT", "DEFENDER_RETREAT"]
    },
    "territory_changed": {
      "type": "boolean",
      "description": "Whether the battle resulted in territory control change"
    },
    "infrastructure_damage_pct": { "type": "number", "minimum": 0.0, "maximum": 100.0 },
    "duration_ticks": { "type": "integer", "minimum": 1 }
  },
  "additionalProperties": false
}
```

---

### 12.4 `treaty.signed.v1`

Emitted when nations enter a diplomatic agreement.

**Payload JSON Schema:**
```json
{
  "$id": "https://civlab.dev/schemas/events/treaty.signed.v1.json",
  "title": "TreatySignedV1Payload",
  "type": "object",
  "required": ["treaty_id", "treaty_type", "nation_ids", "terms", "initiating_nation"],
  "properties": {
    "treaty_id": { "type": "string" },
    "treaty_type": {
      "type": "string",
      "enum": [
        "PEACE", "ALLIANCE", "TRADE_AGREEMENT", "MUTUAL_DEFENSE",
        "NON_AGGRESSION", "CLIMATE_ACCORD", "NUCLEAR_LIMITATION",
        "AID_AGREEMENT", "TECHNOLOGY_SHARING"
      ]
    },
    "nation_ids": {
      "type": "array",
      "items": { "type": "string" },
      "minItems": 2,
      "description": "All signatory nation IDs"
    },
    "initiating_nation": { "type": "string" },
    "terms": {
      "type": "array",
      "items": {
        "type": "object",
        "required": ["term_type", "obligor_nation"],
        "properties": {
          "term_type": { "type": "string" },
          "obligor_nation": { "type": "string" },
          "value": {}
        }
      }
    },
    "duration_ticks": {
      "type": ["integer", "null"],
      "description": "Treaty duration; null = permanent"
    },
    "expiry_tick": {
      "type": ["integer", "null"]
    },
    "negotiation_duration_ticks": { "type": "integer", "minimum": 0 }
  },
  "additionalProperties": false
}
```

---

### 12.5 `shadow.network.detected.v1`

Emitted when a state's internal security apparatus uncovers a corruption or influence network.

**Payload JSON Schema:**
```json
{
  "$id": "https://civlab.dev/schemas/events/shadow.network.detected.v1.json",
  "title": "ShadowNetworkDetectedV1Payload",
  "type": "object",
  "required": ["network_id", "nation_id", "network_size", "capture_level", "detection_method"],
  "properties": {
    "network_id": { "type": "string" },
    "nation_id": { "type": "string" },
    "network_size": {
      "type": "integer",
      "minimum": 1,
      "description": "Number of citizens implicated"
    },
    "capture_level": {
      "type": "number",
      "minimum": 0.0,
      "maximum": 1.0,
      "description": "Fraction of government institutions infiltrated [0,1]"
    },
    "detection_method": {
      "type": "string",
      "enum": ["INTELLIGENCE", "WHISTLEBLOWER", "FOREIGN_PRESSURE", "FINANCIAL_AUDIT", "ACCIDENTAL"]
    },
    "estimated_drained_millijoules": {
      "type": "integer",
      "minimum": 0,
      "description": "Estimated resources extracted by network"
    },
    "foreign_sponsor": {
      "type": ["string", "null"],
      "description": "Nation sponsoring the network (if identified)"
    },
    "legitimacy_impact": {
      "type": "number",
      "description": "Immediate hit to government legitimacy"
    },
    "leaders_captured": {
      "type": "integer",
      "minimum": 0,
      "description": "Number of network leaders apprehended"
    }
  },
  "additionalProperties": false
}
```

---

## 13. Event Types — Research & Lifecycle

These events track the simulation run lifecycle and provide heartbeat/audit telemetry.

### 13.1 `scenario.started.v1`

Emitted at tick 0 when a simulation run begins.

**Payload JSON Schema:**
```json
{
  "$id": "https://civlab.dev/schemas/events/scenario.started.v1.json",
  "title": "ScenarioStartedV1Payload",
  "type": "object",
  "required": ["run_id", "scenario_id", "seed", "parameter_set", "initial_state_hash"],
  "properties": {
    "run_id": { "type": "string" },
    "scenario_id": { "type": "string" },
    "scenario_version": { "type": "string" },
    "seed": { "type": "integer" },
    "parameter_set": {
      "type": "object",
      "additionalProperties": {},
      "description": "Complete parameter set for reproducibility"
    },
    "initial_state_hash": {
      "type": "string",
      "pattern": "^[0-9a-f]{64}$",
      "description": "BLAKE3 hash of the tick-0 state"
    },
    "server_version": { "type": "string" },
    "started_at_ms": { "type": "integer" },
    "label": { "type": ["string", "null"] },
    "tags": {
      "type": "object",
      "additionalProperties": { "type": "string" }
    },
    "max_ticks": {
      "type": ["integer", "null"],
      "description": "Auto-stop tick; null = unlimited"
    }
  },
  "additionalProperties": false
}
```

---

### 13.2 `scenario.ended.v1`

Emitted when a simulation run finishes (normally or due to termination condition).

**Payload JSON Schema:**
```json
{
  "$id": "https://civlab.dev/schemas/events/scenario.ended.v1.json",
  "title": "ScenarioEndedV1Payload",
  "type": "object",
  "required": ["run_id", "scenario_id", "seed", "duration_ticks", "outcome", "final_metrics", "final_state_hash"],
  "properties": {
    "run_id": { "type": "string" },
    "scenario_id": { "type": "string" },
    "seed": { "type": "integer" },
    "duration_ticks": { "type": "integer", "minimum": 0 },
    "outcome": {
      "type": "string",
      "enum": [
        "COMPLETED_NORMALLY", "MAX_TICKS_REACHED", "CIVILIZATIONAL_COLLAPSE",
        "CLIMATE_RUNAWAY", "NUCLEAR_WAR", "PLAYER_TERMINATED", "ERROR"
      ]
    },
    "final_metrics": {
      "type": "object",
      "properties": {
        "co2_ppm": { "type": "number" },
        "temperature_delta_c": { "type": "number" },
        "population_total": { "type": "integer" },
        "happiness_mean": { "type": "number" },
        "gini_coefficient": { "type": "number" },
        "gdp_total_millijoules": { "type": "integer" },
        "wars_total": { "type": "integer" },
        "tipping_points_activated": { "type": "integer" }
      }
    },
    "final_state_hash": {
      "type": "string",
      "pattern": "^[0-9a-f]{64}$"
    },
    "total_events_emitted": { "type": "integer", "minimum": 0 },
    "ended_at_ms": { "type": "integer" }
  },
  "additionalProperties": false
}
```

---

### 13.3 `tick.completed.v1`

Emitted as the final event of every tick. Serves as an audit heartbeat and provides the authoritative state hash for the tick.

**Payload JSON Schema:**
```json
{
  "$id": "https://civlab.dev/schemas/events/tick.completed.v1.json",
  "title": "TickCompletedV1Payload",
  "type": "object",
  "required": ["tick", "run_id", "duration_ms", "event_count", "state_hash"],
  "properties": {
    "tick": { "type": "integer", "minimum": 0 },
    "run_id": { "type": "string" },
    "duration_ms": {
      "type": "integer",
      "minimum": 0,
      "description": "Wall-clock milliseconds to compute this tick"
    },
    "event_count": {
      "type": "integer",
      "minimum": 0,
      "description": "Total events emitted this tick (excluding tick.completed itself)"
    },
    "state_hash": {
      "type": "string",
      "pattern": "^[0-9a-f]{64}$",
      "description": "BLAKE3 hash of state after applying all events in this tick"
    },
    "phase_durations_ms": {
      "type": "object",
      "description": "Per-phase compute time breakdown",
      "properties": {
        "demographics": { "type": "integer" },
        "economy": { "type": "integer" },
        "climate": { "type": "integer" },
        "politics": { "type": "integer" },
        "military": { "type": "integer" },
        "snapshot": { "type": "integer" }
      }
    },
    "memory_usage_bytes": { "type": "integer" },
    "connected_clients": { "type": "integer" }
  },
  "additionalProperties": false
}
```

**Example:**
```json
{
  "event_type": "tick.completed.v1",
  "tick": 1441,
  "seq": 999,
  "payload": {
    "tick": 1441,
    "run_id": "run_2026_02_21_001",
    "duration_ms": 8,
    "event_count": 999,
    "state_hash": "b4c5d6e7f8a9b0c1d2e3f4a5b6c7d8e9f0a1b2c3d4e5f6a7b8c9d0e1f2a3b4",
    "phase_durations_ms": {
      "demographics": 1,
      "economy": 3,
      "climate": 1,
      "politics": 1,
      "military": 1,
      "snapshot": 1
    },
    "memory_usage_bytes": 134217728,
    "connected_clients": 3
  }
}
```

---

## 14. Command Types

Commands are submitted via `sim.command` (Section 5.3). Each command type has a specific `params` schema. All commands are applied deterministically at the start of `tick_applied`.

### 14.1 `POLICY_SET`

Set or replace a policy for a nation.

**`params` Schema:**
```json
{
  "type": "object",
  "required": ["policy_type", "nation_id"],
  "properties": {
    "policy_type": {
      "type": "string",
      "enum": [
        "CARBON_TAX", "INCOME_TAX", "TRADE_TARIFF", "SUBSIDY",
        "CONSCRIPTION", "NATIONALIZATION", "PRIVATIZATION",
        "PROPAGANDA", "CENSORSHIP", "AID_PROGRAM", "SANCTIONS",
        "INFRASTRUCTURE_INVESTMENT", "EDUCATION_REFORM", "HEALTHCARE_REFORM"
      ]
    },
    "nation_id": { "type": "string" },
    "parameters": {
      "type": "object",
      "description": "Policy-specific parameters (e.g. rate_per_tonne_co2 for CARBON_TAX)"
    },
    "effective_tick": {
      "type": ["integer", "null"],
      "description": "Override effective tick; null = next tick"
    },
    "duration_ticks": {
      "type": ["integer", "null"],
      "description": "Duration; null = permanent until replaced"
    },
    "scope": {
      "type": "string",
      "enum": ["NATIONAL", "CITY", "SECTOR", "CLASS"]
    },
    "scope_ids": {
      "type": "array",
      "items": { "type": "string" }
    }
  }
}
```

**Example:**
```json
{
  "command_type": "POLICY_SET",
  "actor_id": "nation_usa",
  "params": {
    "policy_type": "CARBON_TAX",
    "nation_id": "nation_usa",
    "parameters": { "rate_per_tonne_co2": 85, "revenue_recycling": "dividend" },
    "duration_ticks": 3600,
    "scope": "NATIONAL"
  }
}
```

---

### 14.2 `RESEARCH_INVEST`

Allocate research budget to a technology track.

**`params` Schema:**
```json
{
  "type": "object",
  "required": ["nation_id", "tech_track", "budget_millijoules_per_tick"],
  "properties": {
    "nation_id": { "type": "string" },
    "tech_track": {
      "type": "string",
      "enum": [
        "RENEWABLE_ENERGY", "NUCLEAR_FUSION", "CARBON_CAPTURE",
        "PRECISION_AGRICULTURE", "GEOENGINEERING", "AI_GOVERNANCE",
        "MILITARY_TECH", "HEALTHCARE", "MATERIALS_SCIENCE"
      ]
    },
    "budget_millijoules_per_tick": {
      "type": "integer",
      "minimum": 0,
      "description": "Resources allocated per tick"
    },
    "duration_ticks": {
      "type": ["integer", "null"],
      "description": "How long to maintain this investment; null = indefinite"
    },
    "collaborative_nations": {
      "type": "array",
      "items": { "type": "string" },
      "description": "Other nations sharing research costs and benefits"
    }
  }
}
```

---

### 14.3 `MILITARY_ORDER`

Issue a military action order.

**`params` Schema:**
```json
{
  "type": "object",
  "required": ["nation_id", "order_type", "target"],
  "properties": {
    "nation_id": { "type": "string" },
    "order_type": {
      "type": "string",
      "enum": [
        "MOBILIZE", "DEMOBILIZE", "INVADE", "DEFEND", "BLOCKADE",
        "WITHDRAW", "CEASEFIRE_REQUEST", "NUCLEAR_THREAT"
      ]
    },
    "target": {
      "type": "object",
      "description": "Target of order",
      "properties": {
        "target_type": {
          "type": "string",
          "enum": ["NATION", "CITY", "CELL", "WAR"]
        },
        "target_id": { "type": "string" }
      },
      "required": ["target_type", "target_id"]
    },
    "force_size": {
      "type": ["integer", "null"],
      "description": "Number of units to commit; null = maximum available"
    },
    "war_id": {
      "type": ["string", "null"],
      "description": "War ID if ordering within existing war"
    }
  }
}
```

---

### 14.4 `TRADE_NEGOTIATE`

Initiate a trade route or diplomatic trade negotiation.

**`params` Schema:**
```json
{
  "type": "object",
  "required": ["initiating_nation", "partner_nation", "goods", "proposed_terms"],
  "properties": {
    "initiating_nation": { "type": "string" },
    "partner_nation": { "type": "string" },
    "goods": {
      "type": "array",
      "items": {
        "type": "string",
        "enum": ["FOOD", "ENERGY", "MANUFACTURED_GOODS", "LUXURY_GOODS", "RAW_MATERIALS", "SERVICES", "INFORMATION"]
      }
    },
    "proposed_terms": {
      "type": "object",
      "properties": {
        "tariff_rate_pct": { "type": "number", "minimum": 0 },
        "volume_cap_millijoules": { "type": ["integer", "null"] },
        "duration_ticks": { "type": ["integer", "null"] },
        "reciprocal": { "type": "boolean" }
      }
    },
    "transport_type": {
      "type": "string",
      "enum": ["SEA", "RAIL", "AIR", "PIPELINE", "DIGITAL"]
    }
  }
}
```

---

### 14.5 `INFRASTRUCTURE_BUILD`

Order construction of infrastructure in a city.

**`params` Schema:**
```json
{
  "type": "object",
  "required": ["nation_id", "city_id", "infrastructure_type", "budget_millijoules"],
  "properties": {
    "nation_id": { "type": "string" },
    "city_id": { "type": "string" },
    "infrastructure_type": {
      "type": "string",
      "enum": [
        "SOLAR_FARM", "WIND_FARM", "NUCLEAR_PLANT", "COAL_PLANT",
        "RAIL_LINE", "PORT", "HOSPITAL", "SCHOOL", "DESALINATION",
        "SEAWALL", "CARBON_CAPTURE_PLANT", "SMART_GRID"
      ]
    },
    "budget_millijoules": {
      "type": "integer",
      "minimum": 0,
      "description": "Total budget allocated for this project"
    },
    "priority": {
      "type": "string",
      "enum": ["LOW", "NORMAL", "HIGH", "EMERGENCY"],
      "description": "Construction priority (affects cost and speed)"
    }
  }
}
```

---

### 14.6 `EMERGENCY_LEVY`

Impose an emergency tax on a nation's population.

**`params` Schema:**
```json
{
  "type": "object",
  "required": ["nation_id", "levy_rate_pct", "target_class", "purpose"],
  "properties": {
    "nation_id": { "type": "string" },
    "levy_rate_pct": {
      "type": "number",
      "minimum": 0,
      "maximum": 100,
      "description": "Tax rate as percentage of class income"
    },
    "target_class": {
      "type": "string",
      "enum": ["ALL", "SUBSISTENCE", "WORKING", "MIDDLE", "UPPER", "ELITE"]
    },
    "purpose": {
      "type": "string",
      "enum": ["WAR_FUNDING", "DISASTER_RELIEF", "DEBT_PAYMENT", "INFRASTRUCTURE", "CLIMATE_ADAPTATION"]
    },
    "duration_ticks": {
      "type": "integer",
      "minimum": 1
    }
  }
}
```

---

### 14.7 `SCENARIO_FORK`

Fork the current simulation run at a specified tick, creating a new run from that state.

**`params` Schema:**
```json
{
  "type": "object",
  "required": ["fork_at_tick"],
  "properties": {
    "fork_at_tick": {
      "type": "integer",
      "minimum": 0,
      "description": "Tick to fork from; must be <= current tick"
    },
    "new_seed": {
      "type": ["integer", "null"],
      "description": "New RNG seed for the forked run; null = same seed"
    },
    "label": { "type": ["string", "null"] },
    "parameter_overrides": {
      "type": "object",
      "description": "Modified parameters for the forked run"
    },
    "auto_start": {
      "type": "boolean",
      "description": "Start the fork immediately; default false"
    }
  }
}
```

**Result includes `new_run_id` of the forked run.**

---

## 15. Rate Limiting & Backpressure

### 15.1 Per-Client Command Limits

| Limit | Value | Window | Error Code |
|-------|-------|--------|------------|
| Max commands per tick | 10 | 1 tick (100 ms) | `-32004` |
| Max commands per session per minute | 300 | 60 s rolling | `-32004` |
| Max subscriptions per client | 1 active subscription | — | `-32020` |
| Max fast-forward requests in flight | 1 | — | `-32021` |

Commands that exceed rate limits return an error with `error.code = -32004`. The rejected command is not queued.

### 15.2 Per-Simulation Client Limits

| Limit | Value | Notes |
|-------|-------|-------|
| Max simultaneous clients | 5 | Configurable at server startup |
| Max binary frame clients | 2 | High-bandwidth clients counted separately |
| Handshake timeout | 5 s | Client must complete handshake within 5 s of WebSocket upgrade |

When at max capacity, new `sim.handshake` calls return error `-32012`.

### 15.3 Backpressure Protocol

If a client's outbound WebSocket buffer exceeds 10 MB, the server applies backpressure:

1. **Warning notification** sent to client:
```json
{
  "jsonrpc": "2.0",
  "method": "sim.backpressure.warning",
  "params": {
    "buffer_size_bytes": 11534336,
    "threshold_bytes": 10485760,
    "action": "dropping_non_critical"
  }
}
```

2. **Event dropping policy** (non-critical events dropped first):
   - Critical (never dropped): `tick.completed.v1`, `scenario.ended.v1`, `war.declared.v1`, `climate.tipping_point.activated.v1`
   - High priority: `climate.extreme_event.v1`, `economy.energy_shortage.v1`, `election.occurred.v1`
   - Normal: All economy market events, citizen events
   - Low (dropped first): `citizen.happiness_updated.v1`, `economy.ledger_transfer.v1` (high-volume)

3. If buffer exceeds **50 MB**, the server forcibly disconnects the client with close code `1008` (Policy Violation).

### 15.4 Subscription Throttling

If `max_framerate_hz` is set below the simulation tick rate, the server batches multiple ticks' events into a single broadcast frame. The frame's `tick` field reports the most recent tick included.

---

## 16. Error Codes

### 16.1 Standard JSON-RPC Error Codes

| Code | Name | Meaning |
|------|------|---------|
| `-32700` | Parse Error | Invalid JSON received |
| `-32600` | Invalid Request | JSON-RPC structure is malformed |
| `-32601` | Method Not Found | Method does not exist |
| `-32602` | Invalid Params | Parameters fail schema validation |
| `-32603` | Internal Error | Server-side exception (bug) |

### 16.2 Application Error Codes

| Code | Name | Meaning | Retryable |
|------|------|---------|-----------|
| `-32001` | Invalid Tick Range | `from_tick > to_tick` or tick out of range | No |
| `-32002` | Run Not Found | Specified `run_id` does not exist | No |
| `-32003` | Command Rejected | Command invalid for current sim state | No (fix params) |
| `-32004` | Rate Limit Exceeded | Too many commands | Yes (back off) |
| `-32005` | Simulation Paused | Cannot accept commands when paused | Yes (after resume) |
| `-32006` | Unauthorized Actor | Actor not permitted for this command | No |
| `-32007` | Scenario Not Found | `scenario_id` not in registry | No |
| `-32008` | Snapshot Unavailable | Tick snapshot was not retained | No |
| `-32009` | Export Failed | Export generation error | Yes (retry once) |
| `-32010` | Unsupported Protocol Version | Client version not supported | No |
| `-32011` | Authentication Failed | Bad or expired auth token | No |
| `-32012` | Server At Capacity | Max clients reached | Yes (wait) |
| `-32013` | Replay Unavailable | Replay log missing for tick range | No |
| `-32014` | Scenario Load Failed | Scenario JSON failed validation | No |
| `-32015` | Fast-Forward In Progress | Another fast-forward is running | Yes (wait) |
| `-32016` | Invalid Fork Tick | Fork tick > current tick | No |
| `-32017` | Metric Not Found | Unknown metric name | No |
| `-32018` | Subscription Not Found | `subscription_id` unknown | No |
| `-32019` | Simulation Not Running | No active run to interact with | No |
| `-32020` | Subscription Limit | Client already has max subscriptions | No |
| `-32021` | Fast-Forward Limit | Only one fast-forward allowed at a time | Yes (wait) |

### 16.3 Error Response Format

All errors include a `data` object with structured detail:

```json
{
  "jsonrpc": "2.0",
  "id": 42,
  "error": {
    "code": -32004,
    "message": "Rate limit exceeded: max 10 commands per tick",
    "data": {
      "limit_type": "commands_per_tick",
      "limit_value": 10,
      "current_count": 10,
      "reset_at_tick": 1442,
      "retry_after_ms": 87
    }
  }
}
```

---

## 17. Performance Targets & SLOs

### 17.1 Latency SLOs

| Operation | P50 Target | P99 Target | Notes |
|-----------|-----------|-----------|-------|
| `sim.handshake` | < 20 ms | < 100 ms | Cold start may be slower |
| `sim.subscribe` | < 5 ms | < 20 ms | — |
| `sim.command` (acceptance) | < 2 ms | < 16 ms | Within one tick budget |
| Tick compute (target) | < 8 ms | < 16 ms | For 60 FPS client viability |
| Broadcast delivery after tick | < 10 ms | < 50 ms | From tick completion to client receipt |
| `sim.snapshot` (< 1 MB) | < 100 ms | < 500 ms | Inline or URL |
| `sim.snapshot` (> 1 MB) | < 500 ms | < 2 s | URL returned; data streamed |
| `research.export.csv` (< 10k rows) | < 500 ms | < 2 s | — |
| `research.export.parquet` (any) | < 1 s | < 5 s | Async generation |
| `sim.metrics.query` (full run) | < 200 ms | < 1 s | Indexed metrics store |

### 17.2 Throughput Targets

| Metric | Target |
|--------|--------|
| Ticks per second (fast-forward) | >= 500 ticks/s on reference hardware |
| Events per second (broadcast) | >= 10,000 events/s per client |
| Concurrent subscribed clients | 5 (JSON) + 2 (binary) |
| Max event log throughput | 50 MB/s write |

### 17.3 Reference Hardware

Targets are specified for:
- CPU: 8-core x86-64 (AMD Ryzen 9 5900X or equivalent)
- RAM: 16 GB
- Storage: NVMe SSD (sequential write >= 2 GB/s)
- Network: localhost or < 1 ms RTT LAN

### 17.4 Monitoring Metrics

The server exposes a Prometheus-compatible `/metrics` HTTP endpoint:

| Metric | Type | Labels |
|--------|------|--------|
| `civlab_tick_duration_ms` | Histogram | `run_id`, `phase` |
| `civlab_events_emitted_total` | Counter | `run_id`, `event_type` |
| `civlab_connected_clients` | Gauge | `client_type` |
| `civlab_command_queue_depth` | Gauge | `run_id` |
| `civlab_broadcast_latency_ms` | Histogram | `client_id` |
| `civlab_state_hash_verified_total` | Counter | `run_id`, `result` |
| `civlab_backpressure_events_dropped_total` | Counter | `client_id`, `event_type` |

---

## 18. Client SDK Examples

### 18.1 Rust Research Client

Full connection lifecycle using the `civlab-client` crate:

```rust
use civlab_client::{CivLabClient, SubscribeOptions, EventFilter};
use tokio_stream::StreamExt;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Connect to local simulation server
    let client = CivLabClient::connect("ws://localhost:9876/sim").await?;

    // Handshake — establishes session
    let handshake = client
        .handshake("research-client-001", "1.0", civlab_client::ClientType::Research)
        .await?;

    println!(
        "Connected: run_id={}, tick={}, seed={}",
        handshake.run_id, handshake.tick, handshake.seed
    );

    // Subscribe to economy and citizen events at 10 Hz
    let mut sub = client
        .subscribe(SubscribeOptions {
            filter: EventFilter::globs(&["economy.*", "citizen.*", "tick.completed.v1"]),
            max_framerate_hz: 10,
            binary: false,
            include_snapshot_delta: false,
            start_at_tick: None,
        })
        .await?;

    println!("Subscribed: {}", sub.subscription_id);

    // Receive tick broadcasts
    while let Some(frame) = sub.next().await {
        let frame = frame?;
        println!(
            "Tick {}: {} events, state_hash={}...",
            frame.tick,
            frame.events.len(),
            &frame.state_hash[..16]
        );

        for event in &frame.events {
            match event.event_type.as_str() {
                "economy.market_cleared.v1" => {
                    let payload: EconomyMarketCleared = event.parse_payload()?;
                    if payload.unmet_demand > 10_000 {
                        eprintln!(
                            "WARNING: High unmet demand for {} in {}: {}",
                            payload.good, payload.city_id, payload.unmet_demand
                        );
                    }
                }
                "tick.completed.v1" => {
                    let payload: TickCompleted = event.parse_payload()?;
                    if payload.duration_ms > 16 {
                        eprintln!("WARN: Tick {} took {}ms (> 16ms budget)", payload.tick, payload.duration_ms);
                    }
                    // Stop after 100 ticks for this example
                    if payload.tick >= 100 {
                        break;
                    }
                }
                _ => {}
            }
        }
    }

    // Export results as Parquet
    let export = client
        .export_parquet("run_2026_02_21_001", &["economy.market_cleared.v1"], 0, 100)
        .await?;
    println!("Export ready: {}", export.download_url);

    Ok(())
}
```

### 18.2 Rust: Issuing Commands

```rust
use civlab_client::{CivLabClient, Command, PolicySetParams};

async fn apply_carbon_tax(client: &CivLabClient) -> anyhow::Result<()> {
    let result = client
        .command(Command {
            command_type: "POLICY_SET".into(),
            actor_id: "nation_usa".into(),
            target_id: None,
            params: serde_json::to_value(PolicySetParams {
                policy_type: "CARBON_TAX".into(),
                nation_id: "nation_usa".into(),
                parameters: serde_json::json!({
                    "rate_per_tonne_co2": 85,
                    "revenue_recycling": "dividend"
                }),
                duration_ticks: Some(3600),
                scope: "NATIONAL".into(),
                scope_ids: vec![],
                effective_tick: None,
            })?,
            idempotency_key: Some("carbon-tax-rate-85-run-001".into()),
        })
        .await?;

    if result.accepted {
        println!("Policy accepted, applying at tick {}", result.tick_applied);
    } else {
        eprintln!("Policy rejected: {}", result.reason.unwrap_or_default());
    }

    Ok(())
}
```

### 18.3 TypeScript Web Client

Full connection and event handling using the `@civlab/client` npm package:

```typescript
import { CivLabClient, EventEnvelopeV1, TickBroadcastFrame } from "@civlab/client";

async function main() {
  const client = new CivLabClient("ws://localhost:9876/sim");

  // Handshake
  const handshake = await client.handshake({
    clientId: "web-client-001",
    protocolVersion: "1.0",
    clientType: "web",
  });

  console.log(`Connected: run=${handshake.runId} tick=${handshake.tick}`);

  // Subscribe to economy and climate events
  const sub = await client.subscribe({
    filterTypes: ["economy.*", "climate.*", "tick.completed.v1"],
    maxFramerateHz: 30,
  });

  // Register typed event handlers
  client.on("economy.market_cleared.v1", (event: EventEnvelopeV1) => {
    const p = event.payload as {
      good: string;
      city_id: string;
      clearing_price: number;
      unmet_demand: number;
    };
    if (p.unmet_demand > 5000) {
      console.warn(`Market stress: ${p.good} in ${p.city_id}, unmet=${p.unmet_demand}`);
    }
  });

  client.on("climate.tipping_point.activated.v1", (event: EventEnvelopeV1) => {
    const p = event.payload as {
      point_type: string;
      cascade_risk: number;
      temp_at_activation: number;
    };
    console.error(
      `TIPPING POINT: ${p.point_type} at ${p.temp_at_activation.toFixed(2)}°C, cascade_risk=${p.cascade_risk}`
    );
  });

  client.on("tick.completed.v1", (event: EventEnvelopeV1) => {
    const p = event.payload as { tick: number; duration_ms: number; state_hash: string };
    document.getElementById("tick-counter")!.textContent = String(p.tick);
    document.getElementById("state-hash")!.textContent = p.state_hash.slice(0, 16) + "...";
  });

  // Issue a command
  const cmdResult = await client.command({
    commandType: "POLICY_SET",
    actorId: "nation_eu",
    params: {
      policy_type: "CARBON_TAX",
      nation_id: "nation_eu",
      parameters: { rate_per_tonne_co2: 120, revenue_recycling: "green_investment" },
      duration_ticks: 7200,
      scope: "NATIONAL",
    },
  });

  if (cmdResult.accepted) {
    console.log(`Command queued for tick ${cmdResult.tickApplied}`);
  }

  // Start receiving frames
  for await (const frame of sub) {
    // Frame-level processing (e.g. update render state)
    updateRenderState(frame);
  }
}

function updateRenderState(frame: TickBroadcastFrame): void {
  // Dispatch events to UI components, ECS world, etc.
  for (const event of frame.events) {
    window.dispatchEvent(new CustomEvent(`civlab:${event.event_type}`, { detail: event }));
  }
}

main().catch(console.error);
```

### 18.4 Python Research Script

Using the `civlab` Python client for automated research runs:

```python
import asyncio
import json
import polars as pl
from civlab import CivLabClient, SubscribeFilter

async def run_experiment(seed: int, carbon_tax_rate: int) -> dict:
    async with CivLabClient("ws://localhost:9876/sim") as client:
        # Load scenario and start run
        await client.scenario_load(
            scenario_id="baseline_2050",
            seed=seed,
            parameter_overrides={"initial_co2_ppm": 415}
        )

        # Apply carbon tax immediately
        await client.command(
            command_type="POLICY_SET",
            actor_id="nation_global",
            params={
                "policy_type": "CARBON_TAX",
                "nation_id": "nation_global",
                "parameters": {"rate_per_tonne_co2": carbon_tax_rate},
                "scope": "NATIONAL",
            }
        )

        # Fast-forward 10 years (36,000 ticks at 10 ticks/sec)
        await client.fast_forward(ticks=36_000, resume_speed_after=0)

        # Query final metrics
        metrics = await client.metrics_query(
            metric_names=["co2_ppm", "temperature_delta_c", "population_total", "happiness_mean"],
            from_tick=35_900,
            to_tick=36_000,
            aggregate="last",
            bucket_size_ticks=100
        )

        # Export for analysis
        export = await client.export_parquet(
            table="events",
            event_types=["climate.co2_threshold_crossed.v1", "climate.tipping_point.activated.v1"],
            from_tick=0,
            to_tick=36_000,
        )

        df = pl.read_parquet(export.download_url)

        return {
            "seed": seed,
            "carbon_tax_rate": carbon_tax_rate,
            "final_co2_ppm": metrics.series["co2_ppm"]["values"][-1],
            "final_temp_delta_c": metrics.series["temperature_delta_c"]["values"][-1],
            "tipping_points_activated": len(df.filter(pl.col("event_type") == "climate.tipping_point.activated.v1")),
        }

async def sweep():
    seeds = [42, 123, 999, 7777, 31415]
    rates = [0, 50, 85, 120, 200]

    results = []
    for seed in seeds:
        for rate in rates:
            result = await run_experiment(seed, rate)
            results.append(result)
            print(f"seed={seed} rate={rate} -> co2={result['final_co2_ppm']:.1f} ppm")

    df = pl.DataFrame(results)
    df.write_csv("carbon_tax_sweep_results.csv")
    print(df)

asyncio.run(sweep())
```

---

## 19. Versioning Policy

### 19.1 Protocol Versioning

The CivLab protocol follows semantic versioning:

| Version | Compatibility |
|---------|--------------|
| **Major** (e.g. 1.x → 2.0) | Breaking: clients must update |
| **Minor** (e.g. 1.0 → 1.1) | Additive: new methods/fields; clients safe to ignore |
| **Patch** (e.g. 1.0.0 → 1.0.1) | Bug fixes; no schema changes |

The server accepts `protocol_version: "1.x"` for any minor version within major 1.

### 19.2 Event Schema Versioning

Event type names encode their schema version: `economy.market_cleared.v1`. The version suffix increments on any breaking payload schema change.

**Breaking changes** (require new version suffix):
- Removing or renaming a required field
- Changing a field's type
- Changing an enum's values

**Non-breaking changes** (no version increment needed):
- Adding optional fields
- Expanding an enum (adding new values)
- Adding new description text

The server may simultaneously emit multiple versions of the same logical event type (e.g. during transition periods). Clients should subscribe to specific versions they support.

### 19.3 Deprecation Protocol

1. A new event version is introduced (e.g. `economy.market_cleared.v2`).
2. Both v1 and v2 are emitted for 3 simulation releases.
3. v1 is marked deprecated in the `sim.handshake` result via `deprecated_event_types: ["economy.market_cleared.v1"]`.
4. v1 is removed in the next major protocol version.

### 19.4 Schema Registry

All schemas are published at:

```
https://civlab.dev/schemas/{event_type}.json
```

The server also serves schemas over the WebSocket via:

```json
{
  "jsonrpc": "2.0",
  "id": 99,
  "method": "sim.schema.get",
  "params": { "event_type": "economy.market_cleared.v1" }
}
```

Result includes the full JSON Schema document for that event type's payload.

---

## Appendix A: Event Type Registry

Complete list of all event types, their section, and emit frequency:

| Event Type | Section | Typical Frequency |
|-----------|---------|------------------|
| `citizen.born.v1` | 8.1 | Every 10 ticks (population growth tick) |
| `citizen.died.v1` | 8.2 | Every 10 ticks |
| `citizen.migrated.v1` | 8.3 | Every tick (when migration occurs) |
| `citizen.class_changed.v1` | 8.4 | Every tick (when mobility occurs) |
| `citizen.happiness_updated.v1` | 8.5 | Every tick (per-citizen threshold) |
| `economy.market_cleared.v1` | 9.1 | Every tick (per good per city) |
| `economy.ledger_transfer.v1` | 9.2 | Every tick (per transfer) |
| `economy.energy_shortage.v1` | 9.3 | Irregular (when shortage occurs) |
| `economy.price_spike.v1` | 9.4 | Irregular (when spike threshold crossed) |
| `economy.trade_route.established.v1` | 9.5 | Irregular |
| `economy.trade_route.broken.v1` | 9.6 | Irregular |
| `climate.co2_threshold_crossed.v1` | 10.1 | Irregular (milestone events) |
| `climate.tipping_point.activated.v1` | 10.2 | Irregular (rare, high-impact) |
| `climate.extreme_event.v1` | 10.3 | Irregular (weather events) |
| `climate.sea_level.rise.v1` | 10.4 | Every 100 ticks (annual) |
| `institution.legitimacy_changed.v1` | 11.1 | Every tick (threshold-gated) |
| `institution.formed.v1` | 11.2 | Irregular |
| `institution.dissolved.v1` | 11.3 | Irregular |
| `policy.applied.v1` | 11.4 | Per policy application |
| `election.occurred.v1` | 11.5 | Irregular (per election cycle) |
| `war.declared.v1` | 12.1 | Irregular |
| `war.ended.v1` | 12.2 | Irregular |
| `battle.resolved.v1` | 12.3 | Every tick (during wars) |
| `treaty.signed.v1` | 12.4 | Irregular |
| `shadow.network.detected.v1` | 12.5 | Irregular |
| `scenario.started.v1` | 13.1 | Once per run (tick 0) |
| `scenario.ended.v1` | 13.2 | Once per run (final tick) |
| `tick.completed.v1` | 13.3 | Every tick (final event) |

---

## Appendix B: Glossary

| Term | Definition |
|------|-----------|
| **Tick** | Atomic unit of simulation time; 100 ms wall-clock by default |
| **Run** | A single execution of a scenario from tick 0 to termination |
| **Scenario** | Named configuration defining world state, parameters, and rules |
| **Seed** | 64-bit integer initializing the run's RNG |
| **Event Envelope** | Universal wrapper for all simulation events (`EventEnvelopeV1`) |
| **State Hash** | BLAKE3 hash of accumulated simulation state; used for chain integrity |
| **Hash Chain** | Sequence of state hashes linked by each tick's events |
| **Fast-Forward** | Advancing simulation ticks as fast as possible, suppressing broadcast |
| **Fork** | Creating a new run from an existing run's state at a given tick |
| **Millijoules** | Unified energy-equivalent monetary unit used throughout simulation |
| **Backpressure** | Mechanism to protect server when client cannot consume events fast enough |
| **UUIDv7** | UUID variant encoding creation time in high bits; sortable by time |
| **Canonical JSON** | RFC 8785 deterministic JSON serialization used for hashing |
| **MessagePack** | Binary serialization format used for high-frequency binary frames |
| **Glob Pattern** | Shell-style wildcard matching (e.g. `economy.*`) for event filtering |
| **Tipping Point** | Climate system threshold beyond which feedback loops are self-sustaining |
| **Casus Belli** | Stated justification for war declaration |
| **Shadow Network** | Hidden corruption/influence network within a nation's institutions |

---

*End of CivLab API & Events Specification v1.0.0*
*Generated: 2026-02-21 | Status: SPECIFICATION | Owner: CIV Protocol & Integration Team*


---

## Source: models/civ-sim/DATA_MODEL_DB_SPEC.md

# CivLab Data Model and Database Specification

**Spec ID:** SPEC-DATA-MODEL-CIV-001
**Version:** 1.0.0
**Status:** ACTIVE
**Date:** 2026-02-21
**Owner:** CIV Architecture & Engine Team

**Related Specs:**
- `CIV-0001-core-simulation-loop.md` — Deterministic tick architecture, ECS World struct, .civreplay format
- `CIV-0100-economy-v1.md` — Economy module, double-entry ledger, market clearing, conservation invariants
- `CIV-0102-climate-resource-dynamics.md` — Climate state, tipping points, energy scarcity coupling
- `CIV-0103-institutions-governance.md` — Institution state, legitimacy, capture mechanics
- `CIV-0105-war-diplomacy.md` — War records, mobilization, sanctions
- `CIV-0107-joule-economy-system.md` — Citizen joule ledger, quota mechanics

---

## Table of Contents

1. Data Architecture Overview
   - 1.1 Storage Tier Responsibilities
   - 1.2 Storage Hierarchy and Data Flow
   - 1.3 In-Memory ECS World
   - 1.4 SQLite Embedded Store
   - 1.5 PostgreSQL Multi-User Research Mode
   - 1.6 .civreplay Binary Format
2. SQLite Schema DDL — Full Table Definitions
   - 2.1 `schema_versions` — Migration tracking
   - 2.2 `runs` — Simulation run metadata
   - 2.3 `snapshots` — Per-tick state snapshots
   - 2.4 `events` — Event log
   - 2.5 `nations` — Nation state per tick
   - 2.6 `cities` — City state per tick
   - 2.7 `citizens` — Citizen records per tick
   - 2.8 `ledger_transfers` — Double-entry bookkeeping
   - 2.9 `markets` — Market clearing per good per tick
   - 2.10 `climate_state` — Climate state per tick
   - 2.11 `institutions` — Institution state per tick
   - 2.12 `wars` — Conflict records
   - 2.13 `research_runs` — Research and scenario metadata
   - 2.14 `replay_events` — Compressed event stream
   - 2.15 `metrics_timeseries` — Aggregated metrics
   - 2.16 `rng_seeds` — RNG seed log for replay auditability
3. Indexes and Performance
   - 3.1 SQLite Index DDL
   - 3.2 SQLite PRAGMA Settings
   - 3.3 PostgreSQL Partitioning Strategy
   - 3.4 Query Plan Annotations
4. Rust Type Definitions and DDL Mapping
   - 4.1 Core Simulation Structs
   - 4.2 SQLx Type Mappings
   - 4.3 Custom Type Encodings
   - 4.4 Diesel Schema Macros (alternative)
5. Data Lifecycle and Retention
   - 5.1 Hot Window Policy
   - 5.2 Snapshot Policy
   - 5.3 Pruning and Archival
   - 5.4 Export Formats
6. Scenario and Parameter Schema
   - 6.1 JSON Schema Definition
   - 6.2 Validation Rules
   - 6.3 Scenario Registry
7. Conservation and Integrity Invariants
   - 7.1 Ledger Conservation Trigger
   - 7.2 BLAKE3 State Hash Chain
   - 7.3 Unique and Not-Null Constraints Summary
   - 7.4 Foreign Key Cascade Behavior
8. Migration Strategy
   - 8.1 `schema_versions` Table
   - 8.2 Migration File Conventions
   - 8.3 Backward-Compatible Migration Rules
9. Research Query Patterns
   - 9.1 Average Happiness by Class Over Time
   - 9.2 GDP Trajectory per Nation
   - 9.3 Market Price Volatility
   - 9.4 War Frequency Distribution
   - 9.5 Energy Balance vs. Stability Correlation
   - 9.6 Citizen Migration Flows
   - 9.7 Institution Legitimacy Decay
   - 9.8 Climate Shock Correlation with Economic Disruption
   - 9.9 Gini Coefficient Trajectory
   - 9.10 Tipping Point Activation Timeline
10. Test Harness
    - 10.1 Property Tests
    - 10.2 Round-Trip Tests
    - 10.3 Performance Benchmarks
    - 10.4 Test Fixtures

---

## 1. Data Architecture Overview

### 1.1 Storage Tier Responsibilities

CivLab uses three distinct storage tiers with strict separation of concerns. No tier substitutes for another. No silent fallback from one tier to another is permitted.

| Tier | Technology | Scope | Access Pattern | Notes |
|------|-----------|-------|---------------|-------|
| Hot simulation state | In-memory ECS `World` struct | Current tick + rollback buffer (64 ticks) | Direct struct field access, O(1) | Zero I/O; deterministic; dropped on process exit |
| Run results and history | SQLite (embedded) | Per-run database file | SQL reads/writes via SQLx | One file per run; WAL mode; portable |
| Multi-user research | PostgreSQL (optional) | Shared research database | SQL via SQLx; partitioned tables | Partitioned by tick range; RLS per user |
| Replay archive | `.civreplay` binary | Full event stream + seed | Sequential read/write; mmap | BLAKE3-chained; compressed with zstd |
| Scenario library | JSON files on disk | Scenario definitions | File read at load time; validated | Immutable after scenario is started |

### 1.2 Storage Hierarchy and Data Flow

```
Simulation Engine (Rust, in-process)
        │
        ├── ECS World (in-memory)
        │     ├── NationComponent map  (BTreeMap<NationId, NationState>)
        │     ├── CityComponent map    (BTreeMap<CityId, CityState>)
        │     ├── CitizenComponent map (BTreeMap<CitizenId, CitizenRecord>)
        │     ├── LedgerState          (BTreeMap<Currency, i64> per actor)
        │     ├── MarketState          (BTreeMap<Good, MarketClearing>)
        │     ├── ClimateState         (single struct, updated per tick)
        │     └── InstitutionMap       (BTreeMap<InstId, InstitutionState>)
        │
        ├── Tick boundary → SQLite write path
        │     ├── events INSERT (every event emitted during tick)
        │     ├── nations INSERT (full nation state snapshot per tick)
        │     ├── cities INSERT (full city state snapshot per tick)
        │     ├── ledger_transfers INSERT (all transfers per tick)
        │     ├── markets INSERT (market clearing per good per tick)
        │     ├── climate_state INSERT (climate snapshot per tick)
        │     ├── institutions INSERT (institution state per tick)
        │     └── snapshots INSERT (state hash + msgpack blob every 100 ticks)
        │
        ├── Replay event append → .civreplay file
        │     └── All raw events, compressed, BLAKE3-chained
        │
        └── Pruning job (background)
              ├── Citizens older than 500 ticks → compressed to .civreplay
              └── SQLite VACUUM after large prune
```

### 1.3 In-Memory ECS World

The simulation engine maintains a single `World` struct that is the authoritative source of truth for the current tick. This is not persisted between process restarts. The `World` is a pure in-memory ECS (Entity-Component-System) store with the following properties:

- All collections use `BTreeMap` for deterministic iteration order (required for conservation invariants and BLAKE3 hash computation).
- No `HashMap` is used in simulation-critical paths. Hash maps produce non-deterministic iteration order across platforms and seeds, which would break replay.
- All numeric quantities are `i64` fixed-point integers. No `f32` or `f64` is used for economic values. The unit is millijoules (mJ) for energy/wealth, and milliunits for population-derived quantities.
- The `World` carries a rollback buffer of the last 64 tick states to support client-side rewinding without database reads.

```rust
// In-memory ECS World — canonical definition (see src/sim/world.rs)
pub struct World {
    pub tick: u64,
    pub run_id: Uuid,
    pub rng: ChaCha20Rng,
    pub nations: BTreeMap<NationId, NationState>,
    pub cities: BTreeMap<CityId, CityState>,
    pub citizens: BTreeMap<CitizenId, CitizenRecord>,
    pub ledger: LedgerState,
    pub markets: BTreeMap<Good, MarketClearing>,
    pub climate: ClimateState,
    pub institutions: BTreeMap<InstId, InstitutionState>,
    pub wars: BTreeMap<WarId, WarRecord>,
    pub event_log: Vec<SimEvent>,        // flushed to SQLite at tick boundary
    pub rollback_buf: RingBuf<WorldSnap, 64>,
}
```

### 1.4 SQLite Embedded Store

Each simulation run produces exactly one SQLite database file: `runs/{run_id}.db`. This file is the durable record of everything that occurred in that run. It is written at tick boundaries, not mid-tick.

Key SQLite configuration (applied at connection open time, enforced by the `DbConn::open` wrapper — see Section 3.2):

```sql
PRAGMA journal_mode = WAL;
PRAGMA synchronous = NORMAL;
PRAGMA foreign_keys = ON;
PRAGMA cache_size = -65536;      -- 64 MiB (negative = KiB units)
PRAGMA temp_store = MEMORY;
PRAGMA mmap_size = 8589934592;   -- 8 GiB
PRAGMA wal_autocheckpoint = 1000;
```

SQLite is the default storage backend and requires no external services. It is appropriate for single-user simulation, research batch runs, and scenario development.

### 1.5 PostgreSQL Multi-User Research Mode

When multiple researchers run scenarios concurrently against a shared dataset, the system can be configured to use PostgreSQL. This is controlled by the `CIVLAB_DB_URL` environment variable. If it starts with `postgres://`, the PostgreSQL path is used. If it starts with `sqlite://`, the SQLite path is used.

PostgreSQL-specific features used:
- `RANGE` partitioning on `tick` for the `snapshots`, `events`, `citizens`, and `metrics_timeseries` tables.
- Row-Level Security (RLS) keyed on `user_id` for `research_runs`.
- `pg_partman` for automatic partition creation.
- `BRIN` indexes on `tick` columns within each partition.

The schema DDL in Section 2 is written in a SQLite-compatible dialect. PostgreSQL-specific extensions are called out in `[POSTGRES ONLY]` annotations.

### 1.6 .civreplay Binary Format

The `.civreplay` file is a binary append-only log of all raw simulation events. It is the primary archival format and the authoritative source for full replay.

**File layout:**

```
+------------------+
|  Header (256 B)  |
|  magic: b"CIVR"  |
|  version: u16    |
|  run_id: [u8;16] |  -- UUID bytes
|  seed: u64       |
|  tick_count: u64 |
|  reserved: [u8]  |
+------------------+
|  Frame 0         |
|  frame_len: u32  |  -- LE
|  tick: u64       |  -- LE
|  seq: u64        |  -- LE
|  hash_prev: [u8;32] | -- BLAKE3 of previous frame bytes
|  payload_len: u32|
|  payload: [u8]   |  -- zstd-compressed MessagePack event bytes
|  hash_self: [u8;32] | -- BLAKE3 of (frame_len..payload end)
+------------------+
|  Frame 1         |
|  ...             |
+------------------+
```

The hash chain ensures that any tampering with the replay file is detectable: `frame[n].hash_prev` must equal `frame[n-1].hash_self`. The hash of the empty bytes is used for `frame[0].hash_prev`.

---

## 2. SQLite Schema DDL — Full Table Definitions

All DDL is valid SQLite 3.42+. PostgreSQL-specific additions are annotated. Foreign key enforcement requires `PRAGMA foreign_keys = ON` at connection time.

### 2.1 `schema_versions` — Migration Tracking

```sql
CREATE TABLE IF NOT EXISTS schema_versions (
    version         INTEGER     NOT NULL PRIMARY KEY,
    description     TEXT        NOT NULL,
    applied_at      TEXT        NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ', 'now')),
    checksum        TEXT        NOT NULL  -- SHA-256 of the migration file content
);

-- Seed with initial migration
INSERT OR IGNORE INTO schema_versions (version, description, checksum)
VALUES (1, 'initial schema', 'placeholder-replaced-by-migration-tooling');
```

### 2.2 `runs` — Simulation Run Metadata

The `runs` table is the root entity. Every other table references a `run_id`. A run is immutable once its `status` reaches `completed` or `failed`.

```sql
CREATE TABLE IF NOT EXISTS runs (
    run_id          TEXT        NOT NULL PRIMARY KEY,  -- UUID v4, stored as TEXT
    scenario_id     TEXT        NOT NULL,              -- references scenario JSON (not FK; scenarios live on disk)
    seed            INTEGER     NOT NULL,              -- u64 ChaCha20Rng seed
    start_tick      INTEGER     NOT NULL DEFAULT 0,
    end_tick        INTEGER,                           -- NULL while running
    status          TEXT        NOT NULL DEFAULT 'running'
                                CHECK (status IN ('running', 'completed', 'failed', 'paused', 'archived')),
    created_at      TEXT        NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ', 'now')),
    updated_at      TEXT        NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ', 'now')),
    params          TEXT        NOT NULL DEFAULT '{}', -- JSON: scenario parameter overrides
    tick_duration_ms INTEGER    NOT NULL DEFAULT 100,  -- wall-clock ms per simulation tick
    version         TEXT        NOT NULL DEFAULT '1.0.0', -- engine version that produced this run
    notes           TEXT                               -- free-text researcher annotation
);

CREATE INDEX IF NOT EXISTS idx_runs_scenario_id  ON runs (scenario_id);
CREATE INDEX IF NOT EXISTS idx_runs_status       ON runs (status);
CREATE INDEX IF NOT EXISTS idx_runs_created_at   ON runs (created_at);
```

**Rust mapping:**

```rust
#[derive(Debug, Clone, sqlx::FromRow)]
pub struct SimRun {
    pub run_id: String,           // Uuid serialized as hyphenated string
    pub scenario_id: String,
    pub seed: i64,                // u64 stored as i64 (SQLite INTEGER is signed 64-bit)
    pub start_tick: i64,
    pub end_tick: Option<i64>,
    pub status: RunStatus,
    pub created_at: String,       // ISO-8601 UTC
    pub updated_at: String,
    pub params: serde_json::Value,
    pub tick_duration_ms: i64,
    pub version: String,
    pub notes: Option<String>,
}

#[derive(Debug, Clone, PartialEq, sqlx::Type)]
#[sqlx(type_name = "TEXT", rename_all = "snake_case")]
pub enum RunStatus {
    Running,
    Completed,
    Failed,
    Paused,
    Archived,
}
```

### 2.3 `snapshots` — Per-Tick State Snapshots

Full world snapshots are written every 100 ticks. Delta snapshots are written every tick (containing only changed components). The `is_full` flag distinguishes them.

```sql
CREATE TABLE IF NOT EXISTS snapshots (
    run_id          TEXT        NOT NULL REFERENCES runs(run_id) ON DELETE CASCADE,
    tick            INTEGER     NOT NULL,
    is_full         INTEGER     NOT NULL DEFAULT 0 CHECK (is_full IN (0, 1)),
    state_hash      BLOB        NOT NULL,  -- BLAKE3 32-byte digest of canonical world bytes
    snapshot_bytes  BLOB        NOT NULL,  -- zstd-compressed MessagePack of full/delta world
    size_bytes      INTEGER     NOT NULL,  -- uncompressed size in bytes
    compressed_size INTEGER     NOT NULL,  -- compressed size in bytes
    created_at      TEXT        NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ', 'now')),

    PRIMARY KEY (run_id, tick)
);

-- [POSTGRES ONLY] Partition by tick range (1000-tick windows):
-- CREATE TABLE snapshots (...) PARTITION BY RANGE (tick);
-- CREATE TABLE snapshots_tick_0    PARTITION OF snapshots FOR VALUES FROM (0)    TO (1000);
-- CREATE TABLE snapshots_tick_1000 PARTITION OF snapshots FOR VALUES FROM (1000) TO (2000);
-- ... managed by pg_partman

CREATE INDEX IF NOT EXISTS idx_snapshots_run_tick_full
    ON snapshots (run_id, tick)
    WHERE is_full = 1;
```

**Rust mapping:**

```rust
#[derive(Debug, Clone, sqlx::FromRow)]
pub struct Snapshot {
    pub run_id: String,
    pub tick: i64,
    pub is_full: bool,
    pub state_hash: Vec<u8>,       // 32 bytes, BLAKE3 digest
    pub snapshot_bytes: Vec<u8>,   // zstd-compressed msgpack
    pub size_bytes: i64,
    pub compressed_size: i64,
    pub created_at: String,
}
```

**Hash computation contract:**

The `state_hash` is computed as `blake3::hash(&canonical_bytes)` where `canonical_bytes` is the deterministic MessagePack serialization of the `World` struct with all maps sorted by key. The hash of tick `n+1` must be derivable from the hash of tick `n` plus the events applied during tick `n+1`. This chain is verified by the integrity checker (see Section 7.2).

### 2.4 `events` — Event Log

Every simulation event emitted during tick execution is appended to this table at the tick boundary. This is the primary audit trail and replay source for incremental replay (without reading full snapshots).

```sql
CREATE TABLE IF NOT EXISTS events (
    run_id          TEXT        NOT NULL REFERENCES runs(run_id) ON DELETE CASCADE,
    tick            INTEGER     NOT NULL,
    event_id        TEXT        NOT NULL,   -- UUID v4
    event_type      TEXT        NOT NULL,   -- e.g. 'economy.ledger_transfer', 'war.declaration'
    payload         TEXT        NOT NULL,   -- JSON payload; schema defined per event_type
    seq             INTEGER     NOT NULL,   -- monotonically increasing within (run_id, tick)
    parent_event_id TEXT,                   -- NULL for root events; UUID for derived events
    phase           TEXT        NOT NULL DEFAULT 'unknown',
                                            -- tick phase: 'policy','production','market',
                                            --             'ledger','stochastic','climate'
    actor_id        TEXT,                   -- UUID of primary actor (nation, city, citizen, inst)
    created_at      TEXT        NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ', 'now')),

    PRIMARY KEY (run_id, tick, event_id),
    UNIQUE (run_id, seq)  -- global sequence uniqueness per run
);

CREATE INDEX IF NOT EXISTS idx_events_run_tick
    ON events (run_id, tick);
CREATE INDEX IF NOT EXISTS idx_events_run_type
    ON events (run_id, event_type);
CREATE INDEX IF NOT EXISTS idx_events_actor
    ON events (run_id, actor_id)
    WHERE actor_id IS NOT NULL;
CREATE INDEX IF NOT EXISTS idx_events_parent
    ON events (run_id, parent_event_id)
    WHERE parent_event_id IS NOT NULL;
```

**Event type taxonomy:**

| Prefix | Domain | Examples |
|--------|--------|---------|
| `economy.*` | Economy module | `economy.ledger_transfer`, `economy.market_cleared`, `economy.conservation_verified` |
| `climate.*` | Climate module | `climate.temp_updated`, `climate.tipping_point_activated`, `climate.sea_level_updated` |
| `war.*` | War module | `war.declaration`, `war.battle_resolved`, `war.peace_treaty` |
| `institution.*` | Institutions | `institution.policy_changed`, `institution.capture_level_changed`, `institution.legitimacy_updated` |
| `nation.*` | Nation | `nation.ideology_drift`, `nation.stability_updated`, `nation.population_updated` |
| `city.*` | City | `city.migration_flow`, `city.energy_balance_updated`, `city.food_balance_updated` |
| `citizen.*` | Citizen | `citizen.class_transition`, `citizen.employment_changed`, `citizen.happiness_updated` |
| `research.*` | Research events | `research.scenario_started`, `research.parameter_sweep_completed` |
| `sim.*` | Simulation control | `sim.tick_completed`, `sim.run_started`, `sim.run_completed`, `sim.snapshot_written` |

**Rust mapping:**

```rust
#[derive(Debug, Clone, sqlx::FromRow)]
pub struct SimEvent {
    pub run_id: String,
    pub tick: i64,
    pub event_id: String,
    pub event_type: String,
    pub payload: serde_json::Value,
    pub seq: i64,
    pub parent_event_id: Option<String>,
    pub phase: String,
    pub actor_id: Option<String>,
    pub created_at: String,
}
```

### 2.5 `nations` — Nation State Per Tick

One row per nation per tick. This is a full state snapshot of every nation at every persisted tick. Only ticks in the hot window (last 1000 ticks) have full rows; older ticks are pruned unless the run is a research run.

```sql
CREATE TABLE IF NOT EXISTS nations (
    run_id              TEXT        NOT NULL REFERENCES runs(run_id) ON DELETE CASCADE,
    tick                INTEGER     NOT NULL,
    nation_id           TEXT        NOT NULL,   -- UUID v4
    name                TEXT        NOT NULL,
    ideology_vector     TEXT        NOT NULL,   -- JSON array of 8 REAL values, each in [-1.0, 1.0]
                                                -- [planned_vs_market, authoritarian_vs_liberal,
                                                --  isolationist_vs_globalist, secular_vs_theocratic,
                                                --  militarist_vs_pacifist, ecoconservative_vs_extractivist,
                                                --  centralist_vs_federalist, technocratic_vs_traditionalist]
    stability           INTEGER     NOT NULL,   -- 0..10000 (fixed-point, divide by 100 for 0.00..100.00)
    legitimacy          INTEGER     NOT NULL,   -- 0..10000
    population_total    INTEGER     NOT NULL,   -- total population (integer count)
    population_growth   INTEGER     NOT NULL DEFAULT 0,  -- net change this tick (may be negative)
    gdp_millijoules     INTEGER     NOT NULL DEFAULT 0,  -- total economic output in mJ this tick
    energy_surplus_mj   INTEGER     NOT NULL DEFAULT 0,  -- net energy balance (positive = surplus)
    food_surplus_mu     INTEGER     NOT NULL DEFAULT 0,  -- net food balance in milliunits
    gini_coefficient    INTEGER     NOT NULL DEFAULT 0,  -- 0..10000 (x100 for 2 decimal places)
    at_war              INTEGER     NOT NULL DEFAULT 0 CHECK (at_war IN (0, 1)),
    capital_city_id     TEXT,                            -- UUID of capital city (NULL if no cities)

    PRIMARY KEY (run_id, tick, nation_id)
);

CREATE INDEX IF NOT EXISTS idx_nations_run_tick
    ON nations (run_id, tick);
CREATE INDEX IF NOT EXISTS idx_nations_run_nation
    ON nations (run_id, nation_id);

-- [POSTGRES ONLY] RANGE partition on tick
```

**Rust mapping:**

```rust
#[derive(Debug, Clone, sqlx::FromRow)]
pub struct NationState {
    pub run_id: String,
    pub tick: i64,
    pub nation_id: String,
    pub name: String,
    pub ideology_vector: Vec<f64>,   // deserialized from JSON; length always 8
    pub stability: i64,              // 0..10000 fixed-point
    pub legitimacy: i64,
    pub population_total: i64,
    pub population_growth: i64,
    pub gdp_millijoules: i64,
    pub energy_surplus_mj: i64,
    pub food_surplus_mu: i64,
    pub gini_coefficient: i64,
    pub at_war: bool,
    pub capital_city_id: Option<String>,
}
```

### 2.6 `cities` — City State Per Tick

One row per city per tick. Cities are sub-entities of nations. All city-level economic accounting rolls up to the nation.

```sql
CREATE TABLE IF NOT EXISTS cities (
    run_id              TEXT        NOT NULL REFERENCES runs(run_id) ON DELETE CASCADE,
    tick                INTEGER     NOT NULL,
    city_id             TEXT        NOT NULL,   -- UUID v4
    nation_id           TEXT        NOT NULL,   -- UUID v4; references nations.nation_id
    name                TEXT        NOT NULL,
    position_x          INTEGER     NOT NULL,   -- grid x coordinate (integer tile)
    position_y          INTEGER     NOT NULL,   -- grid y coordinate (integer tile)
    population          INTEGER     NOT NULL,
    energy_balance_mj   INTEGER     NOT NULL,   -- net energy (positive = surplus, negative = deficit)
    food_balance_mu     INTEGER     NOT NULL,   -- net food in milliunits
    housing_capacity    INTEGER     NOT NULL,   -- maximum population this city can support
    employed_count      INTEGER     NOT NULL DEFAULT 0,
    unemployed_count    INTEGER     NOT NULL DEFAULT 0,
    happiness_avg       INTEGER     NOT NULL DEFAULT 5000,  -- 0..10000
    infrastructure_level INTEGER   NOT NULL DEFAULT 0,     -- 0..100 (integer percent)
    is_capital          INTEGER     NOT NULL DEFAULT 0 CHECK (is_capital IN (0, 1)),
    under_siege         INTEGER     NOT NULL DEFAULT 0 CHECK (under_siege IN (0, 1)),

    PRIMARY KEY (run_id, tick, city_id)
);

CREATE INDEX IF NOT EXISTS idx_cities_run_tick
    ON cities (run_id, tick);
CREATE INDEX IF NOT EXISTS idx_cities_run_nation
    ON cities (run_id, nation_id, tick);
CREATE INDEX IF NOT EXISTS idx_cities_run_city
    ON cities (run_id, city_id);
```

### 2.7 `citizens` — Citizen Records Per Tick

The highest-volume table. One row per citizen per tick. For large simulations (>100k citizens), only every 10th tick is persisted unless the run is tagged as a research run. Rows older than 500 ticks are pruned to .civreplay unless the run is a research run (see Section 5.3).

```sql
CREATE TABLE IF NOT EXISTS citizens (
    run_id              TEXT        NOT NULL REFERENCES runs(run_id) ON DELETE CASCADE,
    tick                INTEGER     NOT NULL,
    citizen_id          TEXT        NOT NULL,   -- UUID v4
    city_id             TEXT        NOT NULL,   -- UUID v4
    nation_id           TEXT        NOT NULL,   -- UUID v4; denormalized for query efficiency
    happiness           INTEGER     NOT NULL,   -- 0..10000
    wealth_mj           INTEGER     NOT NULL,   -- net wealth in millijoules
    class_enum          TEXT        NOT NULL
                        CHECK (class_enum IN (
                            'subsistence', 'working', 'middle', 'professional',
                            'capitalist', 'elite', 'lumpenproletariat'
                        )),
    employment_status   TEXT        NOT NULL
                        CHECK (employment_status IN (
                            'employed', 'unemployed', 'self_employed',
                            'retired', 'student', 'disabled'
                        )),
    age_ticks           INTEGER     NOT NULL,   -- age in simulation ticks
    joule_quota_mj      INTEGER     NOT NULL DEFAULT 0,  -- current joule quota balance (CIV-0107)
    dissatisfaction     INTEGER     NOT NULL DEFAULT 0,  -- 0..10000; input to political instability
    migration_intent    INTEGER     NOT NULL DEFAULT 0 CHECK (migration_intent IN (0, 1)),

    PRIMARY KEY (run_id, tick, citizen_id)
);

CREATE INDEX IF NOT EXISTS idx_citizens_run_tick
    ON citizens (run_id, tick);
CREATE INDEX IF NOT EXISTS idx_citizens_run_city_tick
    ON citizens (run_id, city_id, tick);
CREATE INDEX IF NOT EXISTS idx_citizens_run_nation_tick
    ON citizens (run_id, nation_id, tick);
CREATE INDEX IF NOT EXISTS idx_citizens_class_tick
    ON citizens (run_id, tick, class_enum);

-- Partial index for citizens with migration intent (common filter)
CREATE INDEX IF NOT EXISTS idx_citizens_migration_intent
    ON citizens (run_id, tick, city_id)
    WHERE migration_intent = 1;

-- [POSTGRES ONLY] Partition by tick, 1000-tick windows
```

**Rust mapping:**

```rust
#[derive(Debug, Clone, sqlx::FromRow)]
pub struct CitizenRecord {
    pub run_id: String,
    pub tick: i64,
    pub citizen_id: String,
    pub city_id: String,
    pub nation_id: String,
    pub happiness: i64,
    pub wealth_mj: i64,
    pub class_enum: CitizenClass,
    pub employment_status: EmploymentStatus,
    pub age_ticks: i64,
    pub joule_quota_mj: i64,
    pub dissatisfaction: i64,
    pub migration_intent: bool,
}

#[derive(Debug, Clone, PartialEq, sqlx::Type)]
#[sqlx(type_name = "TEXT", rename_all = "snake_case")]
pub enum CitizenClass {
    Subsistence,
    Working,
    Middle,
    Professional,
    Capitalist,
    Elite,
    Lumpenproletariat,
}

#[derive(Debug, Clone, PartialEq, sqlx::Type)]
#[sqlx(type_name = "TEXT", rename_all = "snake_case")]
pub enum EmploymentStatus {
    Employed,
    Unemployed,
    SelfEmployed,
    Retired,
    Student,
    Disabled,
}
```

### 2.8 `ledger_transfers` — Double-Entry Bookkeeping

Every resource transfer is recorded as a pair of ledger entries: one debit and one credit. The conservation invariant (Section 7.1) enforces that debits and credits net to zero for every currency per tick. This table is append-only; no updates or deletions are permitted.

```sql
CREATE TABLE IF NOT EXISTS ledger_transfers (
    run_id              TEXT        NOT NULL REFERENCES runs(run_id) ON DELETE CASCADE,
    tick                INTEGER     NOT NULL,
    transfer_id         TEXT        NOT NULL,   -- UUID v4; unique per transfer
    from_actor_id       TEXT        NOT NULL,   -- UUID of debited actor (nation, city, institution)
    from_actor_type     TEXT        NOT NULL
                        CHECK (from_actor_type IN ('nation', 'city', 'citizen', 'institution', 'market', 'void')),
    to_actor_id         TEXT        NOT NULL,   -- UUID of credited actor
    to_actor_type       TEXT        NOT NULL
                        CHECK (to_actor_type IN ('nation', 'city', 'citizen', 'institution', 'market', 'void')),
    amount_mj           INTEGER     NOT NULL CHECK (amount_mj >= 0),  -- always positive; direction implied by from/to
    currency_enum       TEXT        NOT NULL
                        CHECK (currency_enum IN ('joule', 'fiat', 'quota', 'labor_credit', 'carbon_credit')),
    transfer_type       TEXT        NOT NULL,   -- e.g. 'wage', 'tax', 'trade', 'subsidy', 'war_reparation'
    event_id            TEXT,                  -- references events.event_id that triggered this transfer
    created_at          TEXT        NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ', 'now')),

    PRIMARY KEY (run_id, tick, transfer_id)
);

CREATE INDEX IF NOT EXISTS idx_ledger_run_tick
    ON ledger_transfers (run_id, tick);
CREATE INDEX IF NOT EXISTS idx_ledger_run_from_actor
    ON ledger_transfers (run_id, from_actor_id, tick);
CREATE INDEX IF NOT EXISTS idx_ledger_run_to_actor
    ON ledger_transfers (run_id, to_actor_id, tick);
CREATE INDEX IF NOT EXISTS idx_ledger_currency_type
    ON ledger_transfers (run_id, tick, currency_enum, transfer_type);
```

**Rust mapping:**

```rust
#[derive(Debug, Clone, sqlx::FromRow)]
pub struct LedgerTransfer {
    pub run_id: String,
    pub tick: i64,
    pub transfer_id: String,
    pub from_actor_id: String,
    pub from_actor_type: ActorType,
    pub to_actor_id: String,
    pub to_actor_type: ActorType,
    pub amount_mj: i64,         // always >= 0; enforced by DB CHECK and Rust type invariant
    pub currency_enum: Currency,
    pub transfer_type: String,
    pub event_id: Option<String>,
    pub created_at: String,
}

#[derive(Debug, Clone, PartialEq, sqlx::Type)]
#[sqlx(type_name = "TEXT", rename_all = "snake_case")]
pub enum Currency {
    Joule,
    Fiat,
    Quota,
    LaborCredit,
    CarbonCredit,
}

#[derive(Debug, Clone, PartialEq, sqlx::Type)]
#[sqlx(type_name = "TEXT", rename_all = "snake_case")]
pub enum ActorType {
    Nation,
    City,
    Citizen,
    Institution,
    Market,
    Void,   // used for creation events (from Void) and destruction events (to Void)
}
```

### 2.9 `markets` — Market Clearing Per Good Per Tick

One row per (good, city) pair per tick. Records the outcome of the market clearing algorithm for that good in that city at that tick. This table is the primary source for price signal analysis.

```sql
CREATE TABLE IF NOT EXISTS markets (
    run_id              TEXT        NOT NULL REFERENCES runs(run_id) ON DELETE CASCADE,
    tick                INTEGER     NOT NULL,
    good_enum           TEXT        NOT NULL
                        CHECK (good_enum IN (
                            'energy', 'food', 'housing', 'medicine',
                            'capital_goods', 'consumer_goods', 'labor', 'carbon_credit'
                        )),
    city_id             TEXT        NOT NULL,   -- UUID v4; market is always city-scoped
    clearing_price_mj   INTEGER     NOT NULL,   -- price in millijoules per unit
    bid_volume          INTEGER     NOT NULL,   -- total demanded quantity
    ask_volume          INTEGER     NOT NULL,   -- total supplied quantity
    cleared_volume      INTEGER     NOT NULL,   -- quantity actually exchanged
    unmet_demand        INTEGER     NOT NULL DEFAULT 0,  -- bid_volume - cleared_volume (>=0)
    unmet_supply        INTEGER     NOT NULL DEFAULT 0,  -- ask_volume - cleared_volume (>=0)
    price_floor_active  INTEGER     NOT NULL DEFAULT 0 CHECK (price_floor_active IN (0, 1)),
    price_ceiling_active INTEGER   NOT NULL DEFAULT 0 CHECK (price_ceiling_active IN (0, 1)),
    regime              TEXT        NOT NULL DEFAULT 'market'
                        CHECK (regime IN ('market', 'planned', 'joule', 'hybrid')),

    PRIMARY KEY (run_id, tick, good_enum, city_id)
);

CREATE INDEX IF NOT EXISTS idx_markets_run_tick
    ON markets (run_id, tick);
CREATE INDEX IF NOT EXISTS idx_markets_run_good
    ON markets (run_id, good_enum, tick);
CREATE INDEX IF NOT EXISTS idx_markets_run_city
    ON markets (run_id, city_id, tick);

-- Partial index for ticks with unmet demand (supply stress indicator)
CREATE INDEX IF NOT EXISTS idx_markets_unmet_demand
    ON markets (run_id, tick, good_enum)
    WHERE unmet_demand > 0;
```

### 2.10 `climate_state` — Climate State Per Tick

One row per tick for the global climate. Climate is not city-scoped or nation-scoped; it is a single global state. Local effects (sea level rise affecting coastal cities, drought affecting specific biomes) are derived from this table in the query layer.

```sql
CREATE TABLE IF NOT EXISTS climate_state (
    run_id                  TEXT    NOT NULL REFERENCES runs(run_id) ON DELETE CASCADE,
    tick                    INTEGER NOT NULL,
    global_temp_offset_mc   INTEGER NOT NULL,  -- millicelsius above pre-industrial baseline
    co2_ppm_mc              INTEGER NOT NULL,  -- CO2 in milli-ppm (divide by 1000 for ppm)
    sea_level_rise_mm       INTEGER NOT NULL,  -- sea level rise in millimeters
    ocean_acidification_mu  INTEGER NOT NULL,  -- pH drop * 1000000 (microunits)
    arctic_ice_pct          INTEGER NOT NULL,  -- % of baseline ice coverage * 100
    active_tipping_points   TEXT    NOT NULL DEFAULT '[]',  -- JSON array of TippingPoint enum strings
    extreme_weather_count   INTEGER NOT NULL DEFAULT 0,     -- number of extreme weather events this tick
    renewable_capacity_pct  INTEGER NOT NULL DEFAULT 0,     -- % of global energy from renewables * 100

    PRIMARY KEY (run_id, tick)
);

CREATE INDEX IF NOT EXISTS idx_climate_run_tick
    ON climate_state (run_id, tick);
```

**Tipping point enum values (used in `active_tipping_points` JSON array):**

```
'west_antarctic_ice_sheet_collapse'
'greenland_ice_sheet_collapse'
'amazon_dieback'
'permafrost_methane_release'
'atlantic_circulation_collapse'
'coral_reef_die_off'
'boreal_forest_dieback'
'monsoon_disruption'
```

**Rust mapping:**

```rust
#[derive(Debug, Clone, sqlx::FromRow)]
pub struct ClimateStateRow {
    pub run_id: String,
    pub tick: i64,
    pub global_temp_offset_mc: i64,   // millicelsius; divide by 1000 for °C
    pub co2_ppm_mc: i64,              // milli-ppm; divide by 1000 for ppm
    pub sea_level_rise_mm: i64,
    pub ocean_acidification_mu: i64,
    pub arctic_ice_pct: i64,          // * 100 fixed-point
    pub active_tipping_points: Vec<TippingPoint>,  // deserialized from JSON
    pub extreme_weather_count: i64,
    pub renewable_capacity_pct: i64,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TippingPoint {
    WestAntarcticIceSheetCollapse,
    GreenlandIceSheetCollapse,
    AmazonDieback,
    PermafrostMethaneRelease,
    AtlanticCirculationCollapse,
    CoralReefDieOff,
    BorealForestDieback,
    MonsoonDisruption,
}
```

### 2.11 `institutions` — Institution State Per Tick

Institutions are formal organizations within nations: central banks, planning bureaus, regulatory agencies, courts, military commands, etc. They have budgets, legitimacy, and capture levels.

```sql
CREATE TABLE IF NOT EXISTS institutions (
    run_id              TEXT        NOT NULL REFERENCES runs(run_id) ON DELETE CASCADE,
    tick                INTEGER     NOT NULL,
    inst_id             TEXT        NOT NULL,   -- UUID v4
    inst_type           TEXT        NOT NULL
                        CHECK (inst_type IN (
                            'central_bank', 'planning_bureau', 'regulatory_agency',
                            'court', 'military_command', 'trade_union', 'religious_body',
                            'media_organization', 'environmental_agency', 'taxation_authority'
                        )),
    nation_id           TEXT        NOT NULL,   -- UUID v4; owning nation
    name                TEXT        NOT NULL,
    capture_level       INTEGER     NOT NULL DEFAULT 0,  -- 0..10000; 0=not captured, 10000=fully captured
    legitimacy          INTEGER     NOT NULL DEFAULT 5000,  -- 0..10000
    budget_mj           INTEGER     NOT NULL DEFAULT 0,    -- operating budget in millijoules
    budget_spent_mj     INTEGER     NOT NULL DEFAULT 0,    -- amount spent this tick
    policy_vector       TEXT        NOT NULL DEFAULT '{}', -- JSON: active policy settings
    autonomy_level      INTEGER     NOT NULL DEFAULT 5000, -- 0..10000; 0=captured, 10000=fully autonomous
    effectiveness       INTEGER     NOT NULL DEFAULT 5000, -- 0..10000

    PRIMARY KEY (run_id, tick, inst_id)
);

CREATE INDEX IF NOT EXISTS idx_institutions_run_tick
    ON institutions (run_id, tick);
CREATE INDEX IF NOT EXISTS idx_institutions_run_nation
    ON institutions (run_id, nation_id, tick);
CREATE INDEX IF NOT EXISTS idx_institutions_type
    ON institutions (run_id, inst_type, tick);
```

### 2.12 `wars` — Conflict Records

Wars span multiple ticks. One row per war (not per tick). The war record is created when a war declaration event fires and updated when it ends. Battles within a war are tracked via the events table (`war.battle_resolved`).

```sql
CREATE TABLE IF NOT EXISTS wars (
    run_id              TEXT        NOT NULL REFERENCES runs(run_id) ON DELETE CASCADE,
    war_id              TEXT        NOT NULL,   -- UUID v4
    attacker_nation_id  TEXT        NOT NULL,   -- UUID v4
    defender_nation_id  TEXT        NOT NULL,   -- UUID v4
    start_tick          INTEGER     NOT NULL,
    end_tick            INTEGER,               -- NULL while ongoing
    outcome             TEXT                   -- NULL while ongoing
                        CHECK (outcome IS NULL OR outcome IN (
                            'attacker_victory', 'defender_victory', 'stalemate',
                            'peace_treaty', 'white_peace', 'annexation'
                        )),
    casualties_attacker INTEGER     NOT NULL DEFAULT 0,
    casualties_defender INTEGER     NOT NULL DEFAULT 0,
    territory_exchanged TEXT        NOT NULL DEFAULT '[]',  -- JSON array of city_ids that changed hands
    war_score_attacker  INTEGER     NOT NULL DEFAULT 0,     -- 0..10000
    war_score_defender  INTEGER     NOT NULL DEFAULT 0,

    PRIMARY KEY (run_id, war_id)
);

CREATE INDEX IF NOT EXISTS idx_wars_run_nations
    ON wars (run_id, attacker_nation_id, defender_nation_id);
CREATE INDEX IF NOT EXISTS idx_wars_active
    ON wars (run_id, start_tick)
    WHERE end_tick IS NULL;
```

### 2.13 `research_runs` — Research and Scenario Metadata

Supplements `runs` with research-specific metadata. Created when a run is tagged as a research run. Not all runs have a research_runs row.

```sql
CREATE TABLE IF NOT EXISTS research_runs (
    run_id              TEXT        NOT NULL PRIMARY KEY REFERENCES runs(run_id) ON DELETE CASCADE,
    scenario_json       TEXT        NOT NULL,   -- full scenario JSON at time of run
    parameter_set_json  TEXT        NOT NULL,   -- full parameter sweep entry (if part of sweep)
    user_id             TEXT        NOT NULL,   -- researcher identifier (not authenticated in SQLite mode)
    tags                TEXT        NOT NULL DEFAULT '[]',  -- JSON array of string tags
    sweep_id            TEXT,                  -- UUID of parameter sweep batch this run belongs to
    notes               TEXT,
    is_canonical        INTEGER     NOT NULL DEFAULT 0 CHECK (is_canonical IN (0, 1)),
    created_at          TEXT        NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ', 'now'))
);

CREATE INDEX IF NOT EXISTS idx_research_runs_user
    ON research_runs (user_id);
CREATE INDEX IF NOT EXISTS idx_research_runs_sweep
    ON research_runs (sweep_id)
    WHERE sweep_id IS NOT NULL;
CREATE INDEX IF NOT EXISTS idx_research_runs_canonical
    ON research_runs (is_canonical)
    WHERE is_canonical = 1;
```

### 2.14 `replay_events` — Compressed Event Stream for Inline Storage

This table holds a compressed replica of the event stream for runs where the full .civreplay file has been archived but fast seek access is still needed. It is not written during active runs; it is populated by the archival process.

```sql
CREATE TABLE IF NOT EXISTS replay_events (
    run_id              TEXT        NOT NULL REFERENCES runs(run_id) ON DELETE CASCADE,
    seq                 INTEGER     NOT NULL,   -- global sequence number within run
    tick                INTEGER     NOT NULL,
    event_bytes         BLOB        NOT NULL,   -- zstd-compressed MessagePack of SimEvent
    hash_prev           BLOB        NOT NULL,   -- BLAKE3 of previous frame (32 bytes)
    hash_self           BLOB        NOT NULL,   -- BLAKE3 of this frame (32 bytes)

    PRIMARY KEY (run_id, seq)
);

CREATE INDEX IF NOT EXISTS idx_replay_run_tick
    ON replay_events (run_id, tick);
```

### 2.15 `metrics_timeseries` — Aggregated Metrics

Pre-aggregated metrics written at the end of each tick. Redundant with data in nations/cities/citizens but cached here for fast time-series queries without expensive per-tick joins. The `entity_scope` identifies what the metric applies to.

```sql
CREATE TABLE IF NOT EXISTS metrics_timeseries (
    run_id              TEXT        NOT NULL REFERENCES runs(run_id) ON DELETE CASCADE,
    tick                INTEGER     NOT NULL,
    metric_name         TEXT        NOT NULL,
    metric_value        INTEGER     NOT NULL,   -- always integer; fixed-point where needed
    entity_scope        TEXT        NOT NULL,   -- 'global', UUID of nation/city/institution, or 'class:working'
    unit                TEXT        NOT NULL DEFAULT 'raw',  -- 'raw', 'millijoules', 'permille', 'count'

    PRIMARY KEY (run_id, tick, metric_name, entity_scope)
);

CREATE INDEX IF NOT EXISTS idx_metrics_run_name
    ON metrics_timeseries (run_id, metric_name, tick);
CREATE INDEX IF NOT EXISTS idx_metrics_run_scope
    ON metrics_timeseries (run_id, entity_scope, metric_name, tick);
```

**Standard metric names:**

| Metric Name | Unit | Entity Scope | Description |
|------------|------|-------------|-------------|
| `gdp` | `millijoules` | nation UUID | Total economic output |
| `happiness_avg` | `permille` | nation UUID / `class:X` | Population average happiness |
| `gini` | `permille` | nation UUID | Gini coefficient |
| `stability` | `permille` | nation UUID | Political stability |
| `legitimacy` | `permille` | nation UUID / institution UUID | Legitimacy score |
| `energy_balance` | `millijoules` | nation UUID / city UUID | Net energy balance |
| `co2_ppm` | `raw` | `global` | CO2 concentration |
| `temp_offset` | `millicelsius` | `global` | Temperature offset |
| `unemployment_rate` | `permille` | nation UUID / city UUID | Unemployment rate |
| `war_casualties` | `count` | `global` / nation UUID | Cumulative war casualties |
| `market_stress` | `permille` | `global` / city UUID / `good:X` | Market supply stress |
| `institution_capture` | `permille` | institution UUID | Institutional capture level |

### 2.16 `rng_seeds` — RNG Seed Log for Replay Auditability

Every use of the random number generator is logged with its seed, call index, and the simulation phase in which it occurred. This enables full deterministic replay verification: any external verifier can recompute the RNG sequence and confirm it matches the recorded seed log.

```sql
CREATE TABLE IF NOT EXISTS rng_seeds (
    run_id              TEXT        NOT NULL REFERENCES runs(run_id) ON DELETE CASCADE,
    tick                INTEGER     NOT NULL,
    phase_enum          TEXT        NOT NULL
                        CHECK (phase_enum IN (
                            'stochastic_events', 'migration', 'war_resolution',
                            'climate_perturbation', 'citizen_behavior', 'institution_drift'
                        )),
    seed_u64            INTEGER     NOT NULL,   -- u64 stored as i64 (bit-cast)
    call_index          INTEGER     NOT NULL,   -- monotonically increasing within (run_id, tick, phase_enum)
    call_site           TEXT        NOT NULL,   -- source location: "module::function:line"
    output_u64          INTEGER     NOT NULL,   -- the value returned by the RNG at this call

    PRIMARY KEY (run_id, tick, phase_enum, call_index)
);

CREATE INDEX IF NOT EXISTS idx_rng_run_tick
    ON rng_seeds (run_id, tick);
```

---

## 3. Indexes and Performance

### 3.1 SQLite Index DDL Summary

All indexes are created with `CREATE INDEX IF NOT EXISTS` in the migration scripts. The following table summarizes the rationale for each index group.

| Index | Table | Columns | Purpose |
|-------|-------|---------|---------|
| `idx_runs_scenario_id` | runs | scenario_id | Filter runs by scenario |
| `idx_snapshots_run_tick_full` | snapshots | (run_id, tick) WHERE is_full=1 | Fast lookup of full snapshots for rollback |
| `idx_events_run_tick` | events | (run_id, tick) | Per-tick event fetch for replay |
| `idx_events_run_type` | events | (run_id, event_type) | Filter events by type across all ticks |
| `idx_events_actor` | events | (run_id, actor_id) WHERE NOT NULL | All events for a specific actor |
| `idx_nations_run_tick` | nations | (run_id, tick) | Per-tick nation state fetch |
| `idx_nations_run_nation` | nations | (run_id, nation_id) | Full history of one nation |
| `idx_cities_run_nation` | cities | (run_id, nation_id, tick) | All cities of a nation at tick |
| `idx_citizens_run_city_tick` | citizens | (run_id, city_id, tick) | Citizens in a city at tick |
| `idx_citizens_class_tick` | citizens | (run_id, tick, class_enum) | Class distribution per tick |
| `idx_citizens_migration_intent` | citizens | (run_id, tick, city_id) WHERE intent=1 | Migration flow queries |
| `idx_ledger_run_from_actor` | ledger_transfers | (run_id, from_actor_id, tick) | Debits by actor |
| `idx_ledger_currency_type` | ledger_transfers | (run_id, tick, currency_enum, transfer_type) | Transfers by currency and type |
| `idx_markets_run_good` | markets | (run_id, good_enum, tick) | Price history for a good |
| `idx_markets_unmet_demand` | markets | (run_id, tick, good_enum) WHERE >0 | Supply stress events |
| `idx_institutions_run_nation` | institutions | (run_id, nation_id, tick) | Institutions in a nation |
| `idx_metrics_run_name` | metrics_timeseries | (run_id, metric_name, tick) | Time series for a metric |

### 3.2 SQLite PRAGMA Settings

These PRAGMAs are applied by the `DbConn::open` wrapper every time a connection is opened. They are not stored in the database; they must be re-applied on each connection.

```rust
// src/db/conn.rs
pub async fn open(path: &Path) -> Result<SqlitePool, DbError> {
    let options = SqliteConnectOptions::new()
        .filename(path)
        .create_if_missing(true)
        .journal_mode(SqliteJournalMode::Wal)
        .synchronous(SqliteSynchronous::Normal)
        .foreign_keys(true)
        .pragma("cache_size", "-65536")    // 64 MiB
        .pragma("temp_store", "MEMORY")
        .pragma("mmap_size", "8589934592") // 8 GiB
        .pragma("wal_autocheckpoint", "1000")
        .pragma("optimize", "0x10002");    // analyze on close

    SqlitePool::connect_with(options).await.map_err(DbError::Connection)
}
```

**PRAGMA rationale:**

| PRAGMA | Value | Rationale |
|--------|-------|-----------|
| `journal_mode=WAL` | WAL | Concurrent readers during simulation write; no journal contention |
| `synchronous=NORMAL` | NORMAL | Durable enough (survives OS crash); faster than FULL |
| `foreign_keys=ON` | ON | Enforce referential integrity; off by default in SQLite |
| `cache_size=-65536` | 64 MiB | Large page cache reduces I/O on repeated queries |
| `temp_store=MEMORY` | MEMORY | Sorting and grouping operations use RAM, not temp files |
| `mmap_size=8GiB` | 8589934592 | Memory-mapped I/O for read-heavy analytical queries |
| `wal_autocheckpoint=1000` | 1000 | Checkpoint WAL after 1000 pages to prevent unbounded growth |

### 3.3 PostgreSQL Partitioning Strategy

When PostgreSQL is the backend, the following tables use `RANGE` partitioning on `tick`:

```sql
-- snapshots: 1000-tick partitions
CREATE TABLE snapshots (
    -- ... same columns ...
) PARTITION BY RANGE (tick);

-- Managed by pg_partman; template:
CREATE TABLE snapshots_p0000 PARTITION OF snapshots
    FOR VALUES FROM (0) TO (1000);
CREATE TABLE snapshots_p1000 PARTITION OF snapshots
    FOR VALUES FROM (1000) TO (2000);
-- ... created dynamically by pg_partman as simulation progresses

-- BRIN index within each partition (tick is nearly monotonic within partition)
CREATE INDEX snapshots_p0000_tick_brin ON snapshots_p0000 USING BRIN (tick);

-- Same pattern for: events, citizens, metrics_timeseries
-- nations and cities are smaller; no partitioning needed
-- ledger_transfers: partitioned by tick if run has >10M transfers
```

**Partition maintenance:**

```sql
-- pg_partman configuration (managed table)
SELECT partman.create_parent(
    p_parent_table => 'public.snapshots',
    p_control => 'tick',
    p_type => 'range',
    p_interval => '1000',
    p_premake => 4
);
```

### 3.4 Query Plan Annotations

Key queries and their expected query plans:

**Q1: Fetch all nation states at tick T for run R**
```sql
EXPLAIN QUERY PLAN
SELECT * FROM nations WHERE run_id = ? AND tick = ?;
-- Expected: SEARCH nations USING INDEX idx_nations_run_tick (run_id=? AND tick=?)
-- Cardinality: O(num_nations), typically 4-20 rows
-- Expected wall time: <1ms
```

**Q2: Time series of GDP for a specific nation**
```sql
EXPLAIN QUERY PLAN
SELECT tick, gdp_millijoules FROM nations
WHERE run_id = ? AND nation_id = ?
ORDER BY tick ASC;
-- Expected: SEARCH nations USING INDEX idx_nations_run_nation (run_id=? AND nation_id=?)
-- Full index scan for one nation; cardinality = num_ticks
-- Expected wall time: <10ms for 10k ticks
```

**Q3: All events in tick range [T1, T2] for run R**
```sql
EXPLAIN QUERY PLAN
SELECT * FROM events WHERE run_id = ? AND tick BETWEEN ? AND ?
ORDER BY seq ASC;
-- Expected: SEARCH events USING INDEX idx_events_run_tick (run_id=? AND tick>? AND tick<?)
-- Expected wall time: <5ms for 100-tick window
```

**Q4: Market price history for energy in city C**
```sql
EXPLAIN QUERY PLAN
SELECT tick, clearing_price_mj FROM markets
WHERE run_id = ? AND good_enum = 'energy' AND city_id = ?
ORDER BY tick ASC;
-- Expected: SEARCH markets USING INDEX PRIMARY KEY (run_id=? AND tick=? AND good_enum=? AND city_id=?)
-- or: SEARCH markets USING INDEX idx_markets_run_good for range scans
```

---

## 4. Rust Type Definitions and DDL Mapping

### 4.1 Core Simulation Structs

The following Rust structs are the canonical definitions. All SQL DDL is derived from these structs; the structs are the source of truth.

```rust
// src/sim/types.rs — canonical type definitions

use std::collections::BTreeMap;
use uuid::Uuid;
use serde::{Serialize, Deserialize};

/// Fixed-point millijoule value. Never f64 in simulation-critical paths.
pub type Mj = i64;

/// Fixed-point permille value (0..10000 = 0.00%..100.00%).
pub type Permille = i64;

/// Simulation tick counter.
pub type Tick = u64;

/// Nation identifier.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct NationId(pub Uuid);

/// City identifier.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct CityId(pub Uuid);

/// Citizen identifier.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct CitizenId(pub Uuid);

/// Institution identifier.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct InstId(pub Uuid);

/// War identifier.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct WarId(pub Uuid);

/// The complete in-memory world state for one tick.
/// Serialized to msgpack for snapshots. All maps use BTreeMap for determinism.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorldSnap {
    pub tick: Tick,
    pub run_id: Uuid,
    pub nations: BTreeMap<NationId, NationState>,
    pub cities: BTreeMap<CityId, CityState>,
    pub citizens: BTreeMap<CitizenId, CitizenRecord>,
    pub ledger: LedgerState,
    pub markets: BTreeMap<(Good, CityId), MarketClearing>,
    pub climate: ClimateState,
    pub institutions: BTreeMap<InstId, InstitutionState>,
    pub wars: BTreeMap<WarId, WarRecord>,
}

/// Nation state at a single tick.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NationState {
    pub nation_id: NationId,
    pub name: String,
    /// Ideology vector: 8 dimensions, each in [-1.0, 1.0].
    /// Dimensions: [planned_vs_market, authoritarian_vs_liberal,
    ///   isolationist_vs_globalist, secular_vs_theocratic,
    ///   militarist_vs_pacifist, ecoconservative_vs_extractivist,
    ///   centralist_vs_federalist, technocratic_vs_traditionalist]
    pub ideology_vector: [f64; 8],
    pub stability: Permille,
    pub legitimacy: Permille,
    pub population_total: i64,
    pub population_growth: i64,
    pub gdp_millijoules: Mj,
    pub energy_surplus_mj: Mj,
    pub food_surplus_mu: i64,
    pub gini_coefficient: Permille,
    pub at_war: bool,
    pub capital_city_id: Option<CityId>,
}

/// City state at a single tick.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CityState {
    pub city_id: CityId,
    pub nation_id: NationId,
    pub name: String,
    pub position: (i32, i32),
    pub population: i64,
    pub energy_balance_mj: Mj,
    pub food_balance_mu: i64,
    pub housing_capacity: i64,
    pub employed_count: i64,
    pub unemployed_count: i64,
    pub happiness_avg: Permille,
    pub infrastructure_level: i64,
    pub is_capital: bool,
    pub under_siege: bool,
}

/// Individual citizen record at a single tick.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CitizenRecord {
    pub citizen_id: CitizenId,
    pub city_id: CityId,
    pub nation_id: NationId,
    pub happiness: Permille,
    pub wealth_mj: Mj,
    pub class: CitizenClass,
    pub employment_status: EmploymentStatus,
    pub age_ticks: u64,
    pub joule_quota_mj: Mj,
    pub dissatisfaction: Permille,
    pub migration_intent: bool,
}

/// Double-entry ledger state.
/// Invariant: for each currency, sum of all balances across all actors is conserved.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LedgerState {
    pub balances: BTreeMap<(Uuid, Currency), Mj>,
    pub transfers: Vec<LedgerTransfer>,
}

/// Single ledger transfer (double-entry: one debit, one credit per transfer).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LedgerTransfer {
    pub transfer_id: Uuid,
    pub from_actor_id: Uuid,
    pub from_actor_type: ActorType,
    pub to_actor_id: Uuid,
    pub to_actor_type: ActorType,
    pub amount_mj: Mj,
    pub currency: Currency,
    pub transfer_type: String,
    pub event_id: Option<Uuid>,
}

/// Market clearing result for one good in one city at one tick.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarketClearing {
    pub good: Good,
    pub city_id: CityId,
    pub clearing_price_mj: Mj,
    pub bid_volume: i64,
    pub ask_volume: i64,
    pub cleared_volume: i64,
    pub unmet_demand: i64,
    pub unmet_supply: i64,
    pub price_floor_active: bool,
    pub price_ceiling_active: bool,
    pub regime: AllocationRegime,
}

/// Climate state at a single tick. Global (not city-scoped).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClimateState {
    pub global_temp_offset_mc: i64,
    pub co2_ppm_mc: i64,
    pub sea_level_rise_mm: i64,
    pub ocean_acidification_mu: i64,
    pub arctic_ice_pct: Permille,
    pub active_tipping_points: Vec<TippingPoint>,
    pub extreme_weather_count: i64,
    pub renewable_capacity_pct: Permille,
}

/// Institution state at a single tick.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InstitutionState {
    pub inst_id: InstId,
    pub inst_type: InstitutionType,
    pub nation_id: NationId,
    pub name: String,
    pub capture_level: Permille,
    pub legitimacy: Permille,
    pub budget_mj: Mj,
    pub budget_spent_mj: Mj,
    pub policy_vector: serde_json::Value,
    pub autonomy_level: Permille,
    pub effectiveness: Permille,
}

/// War record. Spans multiple ticks; not tick-scoped.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WarRecord {
    pub war_id: WarId,
    pub attacker_nation_id: NationId,
    pub defender_nation_id: NationId,
    pub start_tick: Tick,
    pub end_tick: Option<Tick>,
    pub outcome: Option<WarOutcome>,
    pub casualties_attacker: i64,
    pub casualties_defender: i64,
    pub territory_exchanged: Vec<CityId>,
    pub war_score_attacker: Permille,
    pub war_score_defender: Permille,
}
```

### 4.2 SQLx Type Mappings

The project uses `sqlx` 0.8 with the `sqlite` feature flag. All database operations use typed queries with compile-time SQL verification via `sqlx::query!` and `sqlx::query_as!`.

```rust
// Cargo.toml
// sqlx = { version = "0.8", features = ["sqlite", "runtime-tokio", "uuid", "json", "macros"] }
// uuid = { version = "1", features = ["v4"] }
// rmp-serde = "1"      -- MessagePack serialization
// blake3 = "1"
// zstd = "0.13"

// src/db/queries.rs — example typed queries

pub async fn insert_nation_state(
    pool: &SqlitePool,
    run_id: &str,
    tick: i64,
    state: &NationState,
) -> Result<(), sqlx::Error> {
    let ideology_json = serde_json::to_string(&state.ideology_vector)
        .expect("ideology_vector serialization infallible for [f64; 8]");
    sqlx::query!(
        r#"
        INSERT INTO nations (
            run_id, tick, nation_id, name, ideology_vector,
            stability, legitimacy, population_total, population_growth,
            gdp_millijoules, energy_surplus_mj, food_surplus_mu,
            gini_coefficient, at_war, capital_city_id
        ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
        "#,
        run_id,
        tick,
        state.nation_id.0.to_string(),
        state.name,
        ideology_json,
        state.stability,
        state.legitimacy,
        state.population_total,
        state.population_growth,
        state.gdp_millijoules,
        state.energy_surplus_mj,
        state.food_surplus_mu,
        state.gini_coefficient,
        state.at_war as i64,
        state.capital_city_id.map(|id| id.0.to_string()),
    )
    .execute(pool)
    .await?;
    Ok(())
}

/// Bulk insert citizen records using a transaction for performance.
/// 1M rows target: < 5 seconds (see Section 10.3).
pub async fn bulk_insert_citizens(
    pool: &SqlitePool,
    run_id: &str,
    tick: i64,
    citizens: &[CitizenRecord],
) -> Result<usize, sqlx::Error> {
    let mut tx = pool.begin().await?;
    for citizen in citizens {
        sqlx::query!(
            r#"
            INSERT INTO citizens (
                run_id, tick, citizen_id, city_id, nation_id,
                happiness, wealth_mj, class_enum, employment_status,
                age_ticks, joule_quota_mj, dissatisfaction, migration_intent
            ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
            "#,
            run_id,
            tick,
            citizen.citizen_id.0.to_string(),
            citizen.city_id.0.to_string(),
            citizen.nation_id.0.to_string(),
            citizen.happiness,
            citizen.wealth_mj,
            citizen.class.as_str(),
            citizen.employment_status.as_str(),
            citizen.age_ticks as i64,
            citizen.joule_quota_mj,
            citizen.dissatisfaction,
            citizen.migration_intent as i64,
        )
        .execute(&mut *tx)
        .await?;
    }
    tx.commit().await?;
    Ok(citizens.len())
}
```

### 4.3 Custom Type Encodings

**BLAKE3 hash (32 bytes) ↔ BLOB(32):**

```rust
pub fn blake3_to_blob(hash: &blake3::Hash) -> Vec<u8> {
    hash.as_bytes().to_vec()
}

pub fn blake3_from_blob(blob: &[u8]) -> Result<blake3::Hash, DbError> {
    let bytes: [u8; 32] = blob.try_into()
        .map_err(|_| DbError::InvalidHash(format!("expected 32 bytes, got {}", blob.len())))?;
    Ok(blake3::Hash::from_bytes(bytes))
}
```

**ChaCha20Rng seed (u64) ↔ INTEGER (i64 bit-cast):**

```rust
// u64 → i64 via bit-cast. Round-trip safe. SQLite INTEGER is signed 64-bit.
pub fn seed_to_db(seed: u64) -> i64 { i64::from_ne_bytes(seed.to_ne_bytes()) }
pub fn seed_from_db(stored: i64) -> u64 { u64::from_ne_bytes(stored.to_ne_bytes()) }
```

**Ideology vector ([f64; 8]) ↔ JSON TEXT:**

```rust
pub fn ideology_to_json(v: &[f64; 8]) -> String {
    serde_json::to_string(v).expect("f64 array serialization is infallible")
}
pub fn ideology_from_json(s: &str) -> Result<[f64; 8], DbError> {
    let vec: Vec<f64> = serde_json::from_str(s)
        .map_err(|e| DbError::ParseError(format!("ideology JSON: {e}")))?;
    vec.try_into()
        .map_err(|_| DbError::ParseError("ideology_vector must have exactly 8 elements".into()))
}
```

**Permille (i64, 0..10000) ↔ INTEGER:** Stored directly. Display layer divides by 100 for percent display.

**Millijoule (i64) ↔ INTEGER:** Stored directly. Display layer divides by 1000 for joule display.

**UUID ↔ TEXT:** `uuid::Uuid::to_string()` → hyphenated lowercase. Parse with `uuid::Uuid::parse_str()`.

**Boolean ↔ INTEGER:** `true` = 1, `false` = 0. SQLite has no native BOOLEAN type.

**Vec<TippingPoint> ↔ JSON TEXT:** Serialized as JSON array of snake_case strings via serde.

**serde_json::Value ↔ TEXT:** Stored as compact JSON string. Parsed on read.

### 4.4 Diesel Schema Macros (Reference)

Provided for projects using Diesel as an alternative to SQLx. Not the active backend.

```rust
// src/db/diesel_schema.rs (reference only)

diesel::table! {
    runs (run_id) {
        run_id -> Text,
        scenario_id -> Text,
        seed -> BigInt,
        start_tick -> BigInt,
        end_tick -> Nullable<BigInt>,
        status -> Text,
        created_at -> Text,
        updated_at -> Text,
        params -> Text,
        tick_duration_ms -> BigInt,
        version -> Text,
        notes -> Nullable<Text>,
    }
}

diesel::table! {
    nations (run_id, tick, nation_id) {
        run_id -> Text,
        tick -> BigInt,
        nation_id -> Text,
        name -> Text,
        ideology_vector -> Text,
        stability -> BigInt,
        legitimacy -> BigInt,
        population_total -> BigInt,
        population_growth -> BigInt,
        gdp_millijoules -> BigInt,
        energy_surplus_mj -> BigInt,
        food_surplus_mu -> BigInt,
        gini_coefficient -> BigInt,
        at_war -> Integer,
        capital_city_id -> Nullable<Text>,
    }
}

diesel::table! {
    citizens (run_id, tick, citizen_id) {
        run_id -> Text,
        tick -> BigInt,
        citizen_id -> Text,
        city_id -> Text,
        nation_id -> Text,
        happiness -> BigInt,
        wealth_mj -> BigInt,
        class_enum -> Text,
        employment_status -> Text,
        age_ticks -> BigInt,
        joule_quota_mj -> BigInt,
        dissatisfaction -> BigInt,
        migration_intent -> Integer,
    }
}
```

---

## 5. Data Lifecycle and Retention

### 5.1 Hot Window Policy

The "hot window" is the set of ticks with full per-entity rows in SQLite and available for fast query without replay.

| Layer | Retention Window | Eviction Policy |
|-------|-----------------|----------------|
| In-memory rollback buffer | Last 64 ticks | Ring buffer; oldest evicted on push |
| SQLite citizens rows | Last 500 ticks | Pruning job after each 100-tick batch |
| SQLite nations/cities/institutions/markets | Last 1000 ticks | Pruning job |
| SQLite full snapshots (every 100 ticks) | Indefinite | Never pruned; zstd compressed |
| SQLite delta snapshots (every tick) | Last 200 ticks | Pruned when full snapshot covers range |
| .civreplay archive | All ticks, all events | Never pruned; zstd per-frame |

**The hot window is transparent to queries.** All queries target SQLite. If a tick is outside the hot window, the query layer returns an error directing the caller to use the replay/export path instead. No silent fallback to replay occurs.

### 5.2 Snapshot Policy

**Full snapshots (every 100 ticks):**

```rust
// Written when tick % 100 == 0
pub const FULL_SNAPSHOT_INTERVAL: u64 = 100;
pub const FULL_SNAPSHOT_COMPRESSION_LEVEL: i32 = 6;
```

**Delta snapshots (every tick, configurable):**

```rust
// Written every tick by default; configurable via scenario.persist_every_n_ticks
pub const DELTA_SNAPSHOT_COMPRESSION_LEVEL: i32 = 3;
```

Delta snapshot encoding: a `WorldDelta` struct containing only the entities whose state hash changed since the last snapshot. Components are keyed by entity ID; values are full new state (not diffs within the struct). This keeps deserialization simple (no patch-apply logic) at the cost of slightly larger delta blobs.

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorldDelta {
    pub tick: Tick,
    pub run_id: Uuid,
    pub changed_nations: BTreeMap<NationId, NationState>,
    pub changed_cities: BTreeMap<CityId, CityState>,
    pub changed_citizens: BTreeMap<CitizenId, CitizenRecord>,
    pub removed_citizens: Vec<CitizenId>,   // died this tick
    pub ledger_delta: LedgerDelta,
    pub changed_markets: BTreeMap<(Good, CityId), MarketClearing>,
    pub climate: ClimateState,              // always included (small struct)
    pub changed_institutions: BTreeMap<InstId, InstitutionState>,
    pub war_updates: Vec<WarUpdate>,
}
```

**Snapshot policy table:**

| Tick | Full Snapshot | Delta Snapshot | Notes |
|------|-------------|---------------|-------|
| 0 | YES | NO | Initial state; always full |
| 1..99 | NO | YES | Delta only |
| 100 | YES | NO | Full snapshot replaces delta at 100-tick boundary |
| 101..199 | NO | YES | Delta only |
| 200 | YES | NO | Full snapshot |
| ... | ... | ... | Pattern repeats |

### 5.3 Pruning and Archival

Pruning is a background `tokio::task` scheduled after each batch of 100 ticks completes. It does not block tick execution.

**Citizen pruning:**

```rust
pub const CITIZEN_RETENTION_TICKS: i64 = 500;

pub async fn prune_old_citizen_rows(
    pool: &SqlitePool,
    run_id: &str,
    current_tick: i64,
    is_research_run: bool,
) -> Result<PruneStats, DbError> {
    if is_research_run {
        return Ok(PruneStats::skipped("research_run_retention_override"));
    }
    let cutoff = current_tick - CITIZEN_RETENTION_TICKS;
    if cutoff <= 0 { return Ok(PruneStats::skipped("no_rows_old_enough")); }

    // Archive to .civreplay before deleting
    archive_citizens_to_civreplay(pool, run_id, cutoff).await?;

    let deleted = sqlx::query!(
        "DELETE FROM citizens WHERE run_id = ? AND tick < ?",
        run_id, cutoff
    )
    .execute(pool)
    .await
    .map_err(DbError::Sqlx)?
    .rows_affected();

    // VACUUM only for large prune operations (>100k rows)
    if deleted > 100_000 {
        sqlx::query("VACUUM").execute(pool).await.map_err(DbError::Sqlx)?;
    }

    Ok(PruneStats { rows_deleted: deleted, cutoff_tick: cutoff, skipped_reason: None })
}
```

**Nation/city/institution/market pruning (1000-tick retention):**

```rust
pub const WORLD_STATE_RETENTION_TICKS: i64 = 1000;

pub async fn prune_world_state_rows(
    pool: &SqlitePool,
    run_id: &str,
    current_tick: i64,
    is_research_run: bool,
) -> Result<PruneStats, DbError> {
    if is_research_run { return Ok(PruneStats::skipped("research_run")); }
    let cutoff = current_tick - WORLD_STATE_RETENTION_TICKS;
    if cutoff <= 0 { return Ok(PruneStats::skipped("no_rows_old_enough")); }

    // Prune in a single transaction
    let mut tx = pool.begin().await.map_err(DbError::Sqlx)?;
    let mut total_deleted = 0u64;

    for table in &["nations", "cities", "institutions"] {
        let rows = sqlx::query(&format!(
            "DELETE FROM {} WHERE run_id = ? AND tick < ?", table
        ))
        .bind(run_id)
        .bind(cutoff)
        .execute(&mut *tx)
        .await
        .map_err(DbError::Sqlx)?
        .rows_affected();
        total_deleted += rows;
    }

    // Markets: prune but keep unmet demand rows for research reference
    let market_rows = sqlx::query!(
        "DELETE FROM markets WHERE run_id = ? AND tick < ? AND unmet_demand = 0",
        run_id, cutoff
    )
    .execute(&mut *tx)
    .await
    .map_err(DbError::Sqlx)?
    .rows_affected();
    total_deleted += market_rows;

    tx.commit().await.map_err(DbError::Sqlx)?;
    Ok(PruneStats { rows_deleted: total_deleted, cutoff_tick: cutoff, skipped_reason: None })
}
```

### 5.4 Export Formats

**CSV export:**

One `.csv` file per table. Column names match SQL column names exactly. Produced by `civ export csv --run-id <UUID> --output-dir <path>`.

**Parquet export (via arrow2):**

One `.parquet` file per table. Schema mirrors SQL schema. Column types: Int64 for all INTEGER columns, Utf8 for TEXT, Binary for BLOB. Compression: ZSTD level 4. Row group size: 65536. Produced by `civ export parquet --run-id <UUID> --output-dir <path>`.

Parquet files are the recommended format for research analysis in Python (pandas/polars) or DuckDB:

```python
# Research analysis example (Python/DuckDB)
import duckdb

conn = duckdb.connect()
conn.execute("CREATE VIEW nations AS SELECT * FROM read_parquet('nations.parquet')")
conn.execute("CREATE VIEW citizens AS SELECT * FROM read_parquet('citizens.parquet')")

result = conn.execute("""
    SELECT n.tick, n.nation_id, n.name,
           AVG(c.happiness) AS avg_happiness,
           COUNT(*) AS citizen_count
    FROM citizens c
    JOIN nations n ON c.nation_id = n.nation_id AND c.tick = n.tick
    GROUP BY n.tick, n.nation_id, n.name
    ORDER BY n.tick, n.nation_id
""").fetchdf()
```

**.civreplay export:**

The `.civreplay` file is produced continuously during simulation (see Section 1.6). It can also be produced post-hoc from the SQLite event log via `civ export civreplay --run-id <UUID> --output <path>`.

---

## 6. Scenario and Parameter Schema

### 6.1 JSON Schema Definition

```json
{
  "$schema": "https://civlab.dev/schemas/scenario/v1.json",
  "scenario_id": "3fa85f64-5717-4562-b3fc-2c963f66afa6",
  "name": "Two Superpowers: Joule vs. Market",
  "description": "Comparative study of joule-technocracy and market-capitalism under climate stress.",
  "version": "1.0.0",
  "initial_nations": [
    {
      "nation_id": "11111111-1111-1111-1111-111111111111",
      "name": "Joule Republic",
      "ideology_vector": [-0.8, 0.2, 0.0, -0.5, -0.3, 0.6, 0.4, 0.9],
      "initial_stability": 7500,
      "initial_legitimacy": 8000,
      "initial_population": 50000000,
      "capital_position": {"x": 10, "y": 15},
      "allocation_regime": "joule"
    },
    {
      "nation_id": "22222222-2222-2222-2222-222222222222",
      "name": "Free Market Federation",
      "ideology_vector": [0.9, 0.7, 0.5, -0.2, 0.1, -0.4, -0.3, -0.2],
      "initial_stability": 6500,
      "initial_legitimacy": 7000,
      "initial_population": 60000000,
      "capital_position": {"x": 30, "y": 15},
      "allocation_regime": "market"
    }
  ],
  "initial_cities": [
    {
      "city_id": "aaaa0001-0000-0000-0000-000000000001",
      "nation_id": "11111111-1111-1111-1111-111111111111",
      "name": "Joulesburg",
      "position": {"x": 10, "y": 15},
      "initial_population": 5000000,
      "is_capital": true,
      "initial_infrastructure_level": 75
    }
  ],
  "initial_institutions": [
    {
      "inst_id": "bbbb0001-0000-0000-0000-000000000001",
      "inst_type": "planning_bureau",
      "nation_id": "11111111-1111-1111-1111-111111111111",
      "name": "Joule Allocation Bureau",
      "initial_legitimacy": 8500,
      "initial_budget_mj": 100000000000
    }
  ],
  "climate_config": {
    "initial_temp_offset_mc": 1200,
    "initial_co2_ppm_mc": 420000,
    "initial_sea_level_rise_mm": 200,
    "warming_rate_mc_per_1000_ticks": 50,
    "tipping_points_enabled": true,
    "tipping_point_thresholds": {
      "west_antarctic_ice_sheet_collapse": 15000,
      "amazon_dieback": 20000
    }
  },
  "economy_config": {
    "global_energy_supply_mj_per_tick": 500000000000,
    "food_base_production_mu_per_capita": 3500,
    "market_clearing_algorithm": "walrasian_tatonnement",
    "max_price_adjustment_per_tick_permille": 100,
    "joule_quota_base_mj_per_citizen": 1000000
  },
  "world_config": {
    "grid_width": 50,
    "grid_height": 40,
    "max_cities_per_nation": 20
  },
  "rng_seed": 4242424242424242,
  "tick_limit": 10000,
  "persist_every_n_ticks": 10,
  "research_run": true,
  "win_conditions": [
    {
      "type": "metric_threshold",
      "metric": "happiness_avg",
      "entity_scope": "global",
      "threshold": 8000,
      "sustained_ticks": 500,
      "winner": "highest"
    },
    {
      "type": "tick_limit",
      "tick": 10000,
      "winner": "evaluate_metrics"
    }
  ],
  "tags": ["comparative", "climate_stress", "two_nations", "joule_vs_market"]
}
```

### 6.2 Validation Rules

All scenario files are validated at load time. Failures are hard errors.

```rust
// src/scenario/validator.rs

pub fn validate(scenario: &Scenario) -> Result<(), Vec<ValidationError>> {
    let mut errors = Vec::new();

    // R1: scenario_id must be a valid UUID v4
    if Uuid::parse_str(&scenario.scenario_id).is_err() {
        errors.push(ValidationError::InvalidUuid("scenario_id".into()));
    }

    // R2: ideology_vector must have exactly 8 elements, each in [-1.0, 1.0]
    for nation in &scenario.initial_nations {
        if nation.ideology_vector.len() != 8 {
            errors.push(ValidationError::IdeologyVectorLength(nation.nation_id.clone()));
        }
        for &v in &nation.ideology_vector {
            if !(-1.0f64..=1.0).contains(&v) {
                errors.push(ValidationError::IdeologyVectorRange(nation.nation_id.clone(), v));
            }
        }
    }

    // R3: all city nation_ids must reference a declared nation
    let nation_ids: std::collections::HashSet<_> =
        scenario.initial_nations.iter().map(|n| &n.nation_id).collect();
    for city in &scenario.initial_cities {
        if !nation_ids.contains(&city.nation_id) {
            errors.push(ValidationError::OrphanCity(city.city_id.clone()));
        }
    }

    // R4: all institution nation_ids must reference a declared nation
    for inst in &scenario.initial_institutions {
        if !nation_ids.contains(&inst.nation_id) {
            errors.push(ValidationError::OrphanInstitution(inst.inst_id.clone()));
        }
    }

    // R5: rng_seed must be non-zero
    if scenario.rng_seed == 0 {
        errors.push(ValidationError::ZeroSeed);
    }

    // R6: tick_limit must be >= 100
    if scenario.tick_limit < 100 {
        errors.push(ValidationError::TickLimitTooSmall(scenario.tick_limit));
    }

    // R7: at least one nation and one city required
    if scenario.initial_nations.is_empty() { errors.push(ValidationError::NoNations); }
    if scenario.initial_cities.is_empty() { errors.push(ValidationError::NoCities); }

    // R8: each capital_position must map to an initial_city in that nation
    for nation in &scenario.initial_nations {
        let has_capital = scenario.initial_cities.iter().any(|c|
            c.nation_id == nation.nation_id && c.is_capital
        );
        if !has_capital {
            errors.push(ValidationError::MissingCapitalCity(nation.nation_id.clone()));
        }
    }

    // R9: win_conditions must have at least one tick_limit or metric_threshold entry
    if scenario.win_conditions.is_empty() {
        errors.push(ValidationError::NoWinConditions);
    }

    // R10: persist_every_n_ticks must be >= 1
    if scenario.persist_every_n_ticks < 1 {
        errors.push(ValidationError::InvalidPersistInterval);
    }

    if errors.is_empty() { Ok(()) } else { Err(errors) }
}
```

### 6.3 Scenario Registry

The scenario registry file at `scenarios/index.json` is updated by the CLI when scenarios are added or archived.

```json
{
  "version": "1",
  "updated_at": "2026-02-21T00:00:00Z",
  "scenarios": [
    {
      "scenario_id": "3fa85f64-5717-4562-b3fc-2c963f66afa6",
      "name": "Two Superpowers: Joule vs. Market",
      "file": "two-superpowers-joule-market.json",
      "tags": ["comparative", "climate_stress", "two_nations"],
      "created_at": "2026-02-21T00:00:00Z",
      "status": "active"
    },
    {
      "scenario_id": "00000000-0000-0000-0000-000000000001",
      "name": "Minimal Smoke Test",
      "file": "minimal.json",
      "tags": ["test", "ci"],
      "created_at": "2026-02-21T00:00:00Z",
      "status": "active"
    }
  ]
}
```

---

## 7. Conservation and Integrity Invariants

### 7.1 Ledger Conservation

The primary invariant: for every currency, the net sum of all Void-originated creation events minus all Void-destined destruction events per tick is zero. Peer-to-peer transfers are zero-sum by construction.

**In-process verification (runs before DB write):**

```rust
// src/sim/conservation.rs

pub fn verify_tick_conservation(transfers: &[LedgerTransfer]) -> Result<(), ConservationError> {
    let mut net: BTreeMap<Currency, i64> = BTreeMap::new();
    for t in transfers {
        match (t.from_actor_type, t.to_actor_type) {
            (ActorType::Void, _) => { *net.entry(t.currency).or_default() += t.amount_mj; }
            (_, ActorType::Void) => { *net.entry(t.currency).or_default() -= t.amount_mj; }
            _ => { /* peer transfer: zero net effect on system totals */ }
        }
    }
    for (currency, net_flow) in &net {
        if *net_flow != 0 {
            return Err(ConservationError::NonZeroNetFlow {
                currency: *currency,
                net_flow: *net_flow,
            });
        }
    }
    Ok(())
}
```

**SQL trigger (defense-in-depth, SQLite):**

```sql
CREATE TRIGGER IF NOT EXISTS trg_ledger_conservation_check
AFTER INSERT ON ledger_transfers
BEGIN
    SELECT RAISE(ABORT, 'CONSERVATION_VIOLATION: non-zero net Void flow for this currency/tick')
    WHERE EXISTS (
        SELECT 1
        FROM (
            SELECT
                SUM(CASE
                    WHEN from_actor_type = 'void' THEN  amount_mj
                    WHEN to_actor_type   = 'void' THEN -amount_mj
                    ELSE 0
                END) AS net_flow
            FROM ledger_transfers
            WHERE run_id = NEW.run_id
              AND tick   = NEW.tick
              AND currency_enum = NEW.currency_enum
        ) sub
        WHERE sub.net_flow <> 0
    );
END;
```

### 7.2 BLAKE3 State Hash Chain

Every full snapshot carries a `state_hash` = BLAKE3(canonical_msgpack(WorldSnap)). The canonical bytes are produced by serializing the `WorldSnap` with all BTreeMap keys sorted (guaranteed by BTreeMap ordering), all fields in declaration order, via `rmp_serde::to_vec_named`.

The integrity checker (see `src/sim/integrity.rs`) verifies that for each pair of consecutive full snapshots (tick A, tick B):

1. Load full snapshot at tick A.
2. Fetch all events in (A, B] from the `events` table.
3. Apply events tick-by-tick using the pure state transition function.
4. Compute BLAKE3 of the resulting WorldSnap at tick B.
5. Compare to stored `state_hash` at tick B.
6. If they differ, report a `HashViolation` for tick B.

This check is O(n_events * tick_span). For a 10k-tick run with 100 events/tick, verifying the full chain takes ~100k event applications. Expected wall time: < 60s for a typical research run on modern hardware.

```rust
// Integrity check entry point
pub async fn verify_full_chain(
    pool: &SqlitePool,
    run_id: &str,
) -> Result<HashChainReport, IntegrityError> {
    let full_snapshots = fetch_full_snapshots_ordered(pool, run_id).await?;
    let mut violations = Vec::new();

    for window in full_snapshots.windows(2) {
        let (tick_a, _hash_a) = &window[0];
        let (tick_b, hash_b_stored) = &window[1];

        let world_a = load_and_decompress_snapshot(pool, run_id, *tick_a).await?;
        let events_ab = fetch_events_in_range(pool, run_id, *tick_a, *tick_b).await?;
        let world_b = apply_events_deterministic(world_a, &events_ab)?;

        let bytes_b = rmp_serde::to_vec_named(&world_b)
            .map_err(|e| IntegrityError::Serialize(e.to_string()))?;
        let computed = blake3::hash(&bytes_b);

        if computed.as_bytes() != hash_b_stored.as_slice() {
            violations.push(HashViolation { tick: *tick_b });
        }
    }

    Ok(HashChainReport {
        run_id: run_id.to_string(),
        snapshots_checked: full_snapshots.len(),
        violations,
    })
}
```

### 7.3 Unique and Not-Null Constraints Summary

| Table | Primary Key | Additional UNIQUE | Critical NOT NULL |
|-------|------------|-------------------|------------------|
| `runs` | `(run_id)` | — | run_id, scenario_id, seed, status |
| `snapshots` | `(run_id, tick)` | — | state_hash, snapshot_bytes |
| `events` | `(run_id, tick, event_id)` | `(run_id, seq)` | event_type, payload, seq, phase |
| `nations` | `(run_id, tick, nation_id)` | — | name, ideology_vector, stability, legitimacy, population_total |
| `cities` | `(run_id, tick, city_id)` | — | nation_id, name, position_x, position_y, population |
| `citizens` | `(run_id, tick, citizen_id)` | — | city_id, nation_id, happiness, wealth_mj, class_enum, employment_status, age_ticks |
| `ledger_transfers` | `(run_id, tick, transfer_id)` | — | from_actor_id, to_actor_id, amount_mj, currency_enum, transfer_type |
| `markets` | `(run_id, tick, good_enum, city_id)` | — | clearing_price_mj, bid_volume, ask_volume, cleared_volume |
| `climate_state` | `(run_id, tick)` | — | global_temp_offset_mc, co2_ppm_mc, sea_level_rise_mm |
| `institutions` | `(run_id, tick, inst_id)` | — | inst_type, nation_id, name |
| `wars` | `(run_id, war_id)` | — | attacker_nation_id, defender_nation_id, start_tick |
| `rng_seeds` | `(run_id, tick, phase_enum, call_index)` | — | seed_u64, call_site, output_u64 |

### 7.4 Foreign Key Cascade Behavior

All child tables reference `runs(run_id) ON DELETE CASCADE`. No other cascade relationships. Cross-entity references (nation_id in cities, city_id in citizens) are soft references validated in-process, not enforced by the database.

```sql
-- Safe run deletion (removes all child data atomically):
PRAGMA foreign_keys = ON;
DELETE FROM runs WHERE run_id = ? AND status IN ('archived', 'failed');
-- Cascades automatically to all 15 child tables.
```

---

## 8. Migration Strategy

### 8.1 `schema_versions` Table

Migration state is tracked in `schema_versions`. The migration runner reads this on startup and runs any unapplied migrations in version order.

```sql
CREATE TABLE IF NOT EXISTS schema_versions (
    version         INTEGER     NOT NULL PRIMARY KEY,
    description     TEXT        NOT NULL,
    applied_at      TEXT        NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ', 'now')),
    checksum        TEXT        NOT NULL  -- hex(SHA-256) of migration SQL file content
);
```

### 8.2 Migration Runner

```rust
// src/db/migrations.rs

const MIGRATIONS: &[Migration] = &[
    Migration { version: 1, description: "initial schema: runs, snapshots, events",
        sql: include_str!("../../migrations/001_initial.sql") },
    Migration { version: 2, description: "nations, cities tables",
        sql: include_str!("../../migrations/002_nation_city.sql") },
    Migration { version: 3, description: "citizens, ledger_transfers, markets",
        sql: include_str!("../../migrations/003_economy.sql") },
    Migration { version: 4, description: "climate_state, institutions, wars",
        sql: include_str!("../../migrations/004_world.sql") },
    Migration { version: 5, description: "research_runs, replay_events, metrics_timeseries, rng_seeds",
        sql: include_str!("../../migrations/005_research.sql") },
    Migration { version: 6, description: "schema_versions self-bootstrap + all indexes",
        sql: include_str!("../../migrations/006_indexes.sql") },
];

pub async fn run_pending(pool: &SqlitePool) -> Result<Vec<u32>, MigrationError> {
    // Bootstrap schema_versions if it doesn't exist yet
    sqlx::query(
        "CREATE TABLE IF NOT EXISTS schema_versions (
            version INTEGER NOT NULL PRIMARY KEY,
            description TEXT NOT NULL,
            applied_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ', 'now')),
            checksum TEXT NOT NULL
        )"
    )
    .execute(pool)
    .await
    .map_err(MigrationError::Db)?;

    let applied: Vec<i64> = sqlx::query_scalar!(
        "SELECT version FROM schema_versions ORDER BY version ASC"
    )
    .fetch_all(pool)
    .await
    .unwrap_or_default();

    let applied_set: std::collections::HashSet<i64> = applied.into_iter().collect();
    let mut ran = Vec::new();

    for migration in MIGRATIONS {
        if applied_set.contains(&(migration.version as i64)) { continue; }

        let checksum = format!("{:x}", sha256_of(migration.sql.as_bytes()));
        let mut tx = pool.begin().await.map_err(MigrationError::Db)?;

        // Execute each statement in the migration file separately
        for statement in migration.sql.split(';').map(|s| s.trim()).filter(|s| !s.is_empty()) {
            sqlx::query(statement).execute(&mut *tx).await
                .map_err(|e| MigrationError::Execute(migration.version, e))?;
        }

        sqlx::query!(
            "INSERT INTO schema_versions (version, description, checksum) VALUES (?, ?, ?)",
            migration.version as i64, migration.description, checksum
        )
        .execute(&mut *tx)
        .await
        .map_err(MigrationError::Db)?;

        tx.commit().await.map_err(MigrationError::Db)?;
        ran.push(migration.version);
    }

    Ok(ran)
}
```

### 8.3 Migration File Conventions

Files are at `migrations/NNN_description.sql`. Embedded in binary via `include_str!`. Naming: zero-padded 3-digit version, snake_case description.

```
migrations/
  001_initial.sql               -- runs, snapshots, events
  002_nation_city.sql           -- nations, cities
  003_economy.sql               -- citizens, ledger_transfers, markets
  004_world.sql                 -- climate_state, institutions, wars
  005_research.sql              -- research_runs, replay_events, metrics_timeseries, rng_seeds
  006_indexes.sql               -- all indexes (separated for clarity)
  007_add_notes_column.sql      -- example additive migration
```

**Permitted migration operations:**

| Operation | Permitted |
|-----------|-----------|
| `ADD COLUMN ... DEFAULT ...` | YES |
| `CREATE TABLE IF NOT EXISTS` | YES |
| `CREATE INDEX IF NOT EXISTS` | YES |
| `INSERT OR IGNORE` | YES |
| `DROP COLUMN` | NO — mark deprecated in comment |
| `DROP TABLE` | NO — retain for at least one major version |
| `ALTER COLUMN` | NO — create new column, migrate data, deprecate old |
| Tightening CHECK constraint | NO — breaks existing data |
| Removing CHECK constraint | YES with a new table copy migration |

---

## 9. Research Query Patterns

All queries target SQLite. PostgreSQL variants use the same SQL plus window function extensions.

### 9.1 Average Happiness by Class Over Time

```sql
SELECT
    c.tick,
    c.class_enum,
    COUNT(*)                    AS citizen_count,
    AVG(c.happiness)            AS avg_happiness,
    AVG(c.wealth_mj)            AS avg_wealth_mj,
    AVG(c.dissatisfaction)      AS avg_dissatisfaction
FROM citizens c
WHERE c.run_id = :run_id
  AND c.nation_id = :nation_id
  AND c.tick BETWEEN :tick_start AND :tick_end
GROUP BY c.tick, c.class_enum
ORDER BY c.tick ASC, c.class_enum ASC;
```

Index used: `idx_citizens_run_nation_tick` + `idx_citizens_class_tick`. Expected wall time: <1s for 100-tick window.

### 9.2 GDP Trajectory per Nation

```sql
SELECT
    n.tick,
    n.nation_id,
    n.name                                              AS nation_name,
    n.gdp_millijoules                                   AS gdp_mj,
    n.gdp_millijoules / NULLIF(n.population_total, 0)  AS gdp_per_capita_mj,
    n.gini_coefficient,
    n.energy_surplus_mj,
    n.stability,
    n.legitimacy
FROM nations n
WHERE n.run_id = :run_id
  AND n.tick BETWEEN :tick_start AND :tick_end
ORDER BY n.tick ASC, n.nation_id ASC;
```

### 9.3 Market Price Volatility

```sql
SELECT
    m.good_enum,
    m.city_id,
    COUNT(*)                                                AS observations,
    MIN(m.clearing_price_mj)                               AS price_min,
    MAX(m.clearing_price_mj)                               AS price_max,
    AVG(m.clearing_price_mj)                               AS price_avg,
    AVG(m.clearing_price_mj * m.clearing_price_mj)
        - AVG(m.clearing_price_mj) * AVG(m.clearing_price_mj)  AS price_variance,
    AVG(m.unmet_demand)                                    AS avg_unmet_demand,
    SUM(CASE WHEN m.price_floor_active = 1 THEN 1 ELSE 0 END)   AS ticks_floor_active,
    SUM(CASE WHEN m.price_ceiling_active = 1 THEN 1 ELSE 0 END) AS ticks_ceiling_active
FROM markets m
WHERE m.run_id = :run_id
  AND m.tick BETWEEN :tick_start AND :tick_end
GROUP BY m.good_enum, m.city_id
ORDER BY price_variance DESC;
```

### 9.4 War Frequency Distribution

```sql
SELECT
    w.outcome,
    COUNT(*)                                                AS war_count,
    AVG(COALESCE(w.end_tick, :current_tick) - w.start_tick) AS avg_duration_ticks,
    SUM(w.casualties_attacker + w.casualties_defender)     AS total_casualties
FROM wars w
WHERE w.run_id = :run_id
GROUP BY w.outcome
ORDER BY war_count DESC;
```

### 9.5 Energy Balance vs. Stability Correlation (Pearson r)

```sql
SELECT
    n.nation_id,
    n.name,
    COUNT(*) AS sample_size,
    (COUNT(*) * SUM(n.energy_surplus_mj * n.stability)
     - SUM(n.energy_surplus_mj) * SUM(n.stability))
    / NULLIF(SQRT(
        (COUNT(*) * SUM(n.energy_surplus_mj * n.energy_surplus_mj)
         - SUM(n.energy_surplus_mj) * SUM(n.energy_surplus_mj))
        * (COUNT(*) * SUM(n.stability * n.stability)
           - SUM(n.stability) * SUM(n.stability))
    ), 0) AS pearson_r
FROM nations n
WHERE n.run_id = :run_id
  AND n.tick BETWEEN :tick_start AND :tick_end
GROUP BY n.nation_id, n.name
ORDER BY ABS(pearson_r) DESC;
```

### 9.6 Citizen Migration Flows

```sql
SELECT
    c.tick,
    c.city_id,
    COUNT(*)                                    AS total_citizens,
    SUM(c.migration_intent)                     AS migration_intent_count,
    CAST(SUM(c.migration_intent) AS REAL)
        / NULLIF(COUNT(*), 0) * 100.0           AS migration_intent_pct
FROM citizens c
WHERE c.run_id = :run_id
  AND c.tick BETWEEN :tick_start AND :tick_end
GROUP BY c.tick, c.city_id
ORDER BY c.tick ASC, migration_intent_pct DESC;
```

### 9.7 Institution Legitimacy Decay

```sql
SELECT
    i.tick,
    i.inst_id,
    i.name,
    i.inst_type,
    i.legitimacy,
    i.capture_level,
    i.autonomy_level,
    i.legitimacy - LAG(i.legitimacy, 1) OVER (
        PARTITION BY i.inst_id ORDER BY i.tick
    ) AS legitimacy_delta_per_tick
FROM institutions i
WHERE i.run_id = :run_id
  AND i.nation_id = :nation_id
  AND i.tick BETWEEN :tick_start AND :tick_end
ORDER BY i.inst_id, i.tick ASC;

-- Note: requires SQLite 3.25+ for LAG() window function.
```

### 9.8 Climate Shock vs. Economic Disruption

```sql
WITH climate_shocks AS (
    SELECT e.tick, json_extract(e.payload, '$.tipping_point') AS tipping_point
    FROM events e
    WHERE e.run_id = :run_id AND e.event_type = 'climate.tipping_point_activated'
)
SELECT
    cs.tipping_point,
    cs.tick AS shock_tick,
    AVG(n_pre.gdp_millijoules)  AS avg_gdp_pre_50ticks,
    AVG(n_post.gdp_millijoules) AS avg_gdp_post_50ticks,
    AVG(n_post.gdp_millijoules) - AVG(n_pre.gdp_millijoules) AS gdp_delta
FROM climate_shocks cs
JOIN nations n_pre  ON n_pre.run_id  = :run_id AND n_pre.tick  BETWEEN cs.tick - 50 AND cs.tick - 1
JOIN nations n_post ON n_post.run_id = :run_id AND n_post.tick BETWEEN cs.tick + 1  AND cs.tick + 50
GROUP BY cs.tipping_point, cs.tick
ORDER BY ABS(gdp_delta) DESC;
```

### 9.9 Gini Coefficient Trajectory with Moving Average

```sql
SELECT
    n.tick,
    n.nation_id,
    n.name,
    n.gini_coefficient,
    AVG(n.gini_coefficient) OVER (
        PARTITION BY n.nation_id
        ORDER BY n.tick
        ROWS BETWEEN 99 PRECEDING AND CURRENT ROW
    ) AS gini_ma_100ticks,
    n.stability,
    n.gdp_millijoules
FROM nations n
WHERE n.run_id = :run_id
  AND n.tick BETWEEN :tick_start AND :tick_end
ORDER BY n.nation_id, n.tick ASC;
```

### 9.10 Tipping Point Activation Timeline

```sql
SELECT
    json_extract(e.payload, '$.tipping_point') AS tipping_point,
    MIN(e.tick)                                AS first_activation_tick,
    cl.global_temp_offset_mc / 1000.0         AS temp_c_at_activation,
    cl.co2_ppm_mc / 1000.0                    AS co2_ppm_at_activation
FROM events e
JOIN climate_state cl ON cl.run_id = e.run_id AND cl.tick = e.tick
WHERE e.run_id = :run_id
  AND e.event_type = 'climate.tipping_point_activated'
GROUP BY json_extract(e.payload, '$.tipping_point')
ORDER BY first_activation_tick ASC;
```

---

## 10. Test Harness

### 10.1 Property Tests

Property tests verify the conservation and integrity invariants across arbitrary simulation states. They use `proptest` for property-based testing.

```toml
# Cargo.toml (dev dependencies)
[dev-dependencies]
proptest = "1"
tokio-test = "0.4"
tempfile = "3"
```

**Conservation invariant property test:**

```rust
// tests/property/conservation.rs

use proptest::prelude::*;
use civ_sim::sim::conservation::verify_tick_conservation;
use civ_sim::sim::types::{LedgerTransfer, ActorType, Currency};
use uuid::Uuid;

proptest! {
    /// Property: Any set of balanced creation/destruction pairs satisfies conservation.
    #[test]
    fn prop_balanced_void_transfers_conserve(
        amounts in prop::collection::vec(1i64..1_000_000_000i64, 0..100),
        currencies in prop::collection::vec(0u8..5, 0..100),
    ) {
        let transfers: Vec<LedgerTransfer> = amounts.iter().zip(currencies.iter())
            .flat_map(|(&amount, &cur_idx)| {
                let currency = match cur_idx % 5 {
                    0 => Currency::Joule,
                    1 => Currency::Fiat,
                    2 => Currency::Quota,
                    3 => Currency::LaborCredit,
                    _ => Currency::CarbonCredit,
                };
                let actor = Uuid::new_v4();
                // Create a matched creation + destruction pair
                vec![
                    LedgerTransfer {
                        transfer_id: Uuid::new_v4(),
                        from_actor_id: Uuid::nil(),
                        from_actor_type: ActorType::Void,
                        to_actor_id: actor,
                        to_actor_type: ActorType::Nation,
                        amount_mj: amount,
                        currency,
                        transfer_type: "creation".into(),
                        event_id: None,
                    },
                    LedgerTransfer {
                        transfer_id: Uuid::new_v4(),
                        from_actor_id: actor,
                        from_actor_type: ActorType::Nation,
                        to_actor_id: Uuid::nil(),
                        to_actor_type: ActorType::Void,
                        amount_mj: amount,
                        currency,
                        transfer_type: "destruction".into(),
                        event_id: None,
                    },
                ]
            })
            .collect();

        prop_assert!(verify_tick_conservation(&transfers).is_ok(),
            "Balanced creation/destruction pairs must satisfy conservation");
    }

    /// Property: Unbalanced Void transfers always violate conservation.
    #[test]
    fn prop_unbalanced_void_transfers_violate(
        amount in 1i64..1_000_000_000i64,
    ) {
        let transfer = LedgerTransfer {
            transfer_id: Uuid::new_v4(),
            from_actor_id: Uuid::nil(),
            from_actor_type: ActorType::Void,
            to_actor_id: Uuid::new_v4(),
            to_actor_type: ActorType::Nation,
            amount_mj: amount,
            currency: Currency::Joule,
            transfer_type: "creation_unmatched".into(),
            event_id: None,
        };
        prop_assert!(verify_tick_conservation(&[transfer]).is_err(),
            "Unmatched creation must violate conservation");
    }

    /// Property: Peer-to-peer transfers never affect conservation.
    #[test]
    fn prop_peer_transfers_do_not_affect_conservation(
        amounts in prop::collection::vec(1i64..1_000_000_000i64, 0..200),
    ) {
        let transfers: Vec<LedgerTransfer> = amounts.iter().map(|&amount| {
            LedgerTransfer {
                transfer_id: Uuid::new_v4(),
                from_actor_id: Uuid::new_v4(),
                from_actor_type: ActorType::Nation,
                to_actor_id: Uuid::new_v4(),
                to_actor_type: ActorType::City,
                amount_mj: amount,
                currency: Currency::Joule,
                transfer_type: "transfer".into(),
                event_id: None,
            }
        }).collect();
        prop_assert!(verify_tick_conservation(&transfers).is_ok(),
            "Peer transfers must always satisfy conservation (zero net effect)");
    }
}
```

**Ideology vector property test:**

```rust
proptest! {
    /// Property: ideology_to_json → ideology_from_json round-trips for all valid vectors.
    #[test]
    fn prop_ideology_roundtrip(
        values in prop::array::uniform8(-1.0f64..=1.0f64)
    ) {
        use civ_sim::db::encoding::{ideology_to_json, ideology_from_json};
        let json = ideology_to_json(&values);
        let recovered = ideology_from_json(&json).expect("roundtrip should succeed");
        for (a, b) in values.iter().zip(recovered.iter()) {
            prop_assert!((a - b).abs() < 1e-12,
                "ideology_vector roundtrip must be lossless for valid values");
        }
    }

    /// Property: seed_to_db → seed_from_db round-trips for all u64 values.
    #[test]
    fn prop_seed_roundtrip(seed: u64) {
        use civ_sim::db::encoding::{seed_to_db, seed_from_db};
        prop_assert_eq!(seed_from_db(seed_to_db(seed)), seed,
            "u64 seed bit-cast to i64 and back must be lossless");
    }
}
```

### 10.2 Round-Trip Tests

Round-trip tests verify that serialization → storage → retrieval → deserialization produces an identical struct. These use a real in-memory SQLite database.

```rust
// tests/integration/roundtrip.rs

use civ_sim::db::{conn::open_in_memory, migrations::run_pending};
use civ_sim::db::queries::*;
use civ_sim::sim::types::*;
use uuid::Uuid;

async fn setup_db() -> sqlx::SqlitePool {
    let pool = open_in_memory().await.expect("in-memory SQLite must open");
    run_pending(&pool).await.expect("migrations must succeed");
    pool
}

#[tokio::test]
async fn test_run_roundtrip() {
    let pool = setup_db().await;
    let run_id = Uuid::new_v4().to_string();

    // Insert
    sqlx::query!(
        "INSERT INTO runs (run_id, scenario_id, seed, status, params, version)
         VALUES (?, 'test-scenario', 12345, 'running', '{}', '1.0.0')",
        run_id
    )
    .execute(&pool).await.unwrap();

    // Retrieve
    let row = sqlx::query!(
        "SELECT run_id, scenario_id, seed, status FROM runs WHERE run_id = ?",
        run_id
    )
    .fetch_one(&pool).await.unwrap();

    assert_eq!(row.run_id, run_id);
    assert_eq!(row.seed, 12345i64);
    assert_eq!(row.status, "running");
}

#[tokio::test]
async fn test_nation_state_roundtrip() {
    let pool = setup_db().await;
    let run_id = Uuid::new_v4().to_string();
    let nation_id = Uuid::new_v4();

    // Setup run
    sqlx::query!(
        "INSERT INTO runs (run_id, scenario_id, seed, status, params, version)
         VALUES (?, 'test', 1, 'running', '{}', '1.0.0')",
        run_id
    )
    .execute(&pool).await.unwrap();

    let original = NationState {
        nation_id: NationId(nation_id),
        name: "Test Nation".to_string(),
        ideology_vector: [0.1, -0.2, 0.3, -0.4, 0.5, -0.6, 0.7, -0.8],
        stability: 7500,
        legitimacy: 6000,
        population_total: 10_000_000,
        population_growth: 1234,
        gdp_millijoules: 999_000_000,
        energy_surplus_mj: -50_000,
        food_surplus_mu: 100_000,
        gini_coefficient: 3500,
        at_war: false,
        capital_city_id: None,
    };

    insert_nation_state(&pool, &run_id, 42, &original).await.unwrap();

    let fetched = fetch_nations_at_tick(&pool, &run_id, 42).await.unwrap();
    assert_eq!(fetched.len(), 1);
    let n = &fetched[0];

    assert_eq!(n.nation_id, original.nation_id);
    assert_eq!(n.name, original.name);
    assert_eq!(n.stability, original.stability);
    assert_eq!(n.gdp_millijoules, original.gdp_millijoules);
    assert_eq!(n.at_war, original.at_war);

    for (a, b) in n.ideology_vector.iter().zip(original.ideology_vector.iter()) {
        assert!((a - b).abs() < 1e-12, "ideology_vector must round-trip losslessly");
    }
}

#[tokio::test]
async fn test_snapshot_roundtrip() {
    let pool = setup_db().await;
    let run_id = Uuid::new_v4().to_string();

    sqlx::query!(
        "INSERT INTO runs (run_id, scenario_id, seed, status, params, version)
         VALUES (?, 'test', 1, 'running', '{}', '1.0.0')",
        run_id
    )
    .execute(&pool).await.unwrap();

    let world = WorldSnap {
        tick: 100,
        run_id: Uuid::parse_str(&run_id).unwrap(),
        nations: Default::default(),
        cities: Default::default(),
        citizens: Default::default(),
        ledger: LedgerState { balances: Default::default(), transfers: vec![] },
        markets: Default::default(),
        climate: ClimateState {
            global_temp_offset_mc: 1500,
            co2_ppm_mc: 450_000,
            sea_level_rise_mm: 300,
            ocean_acidification_mu: 150_000,
            arctic_ice_pct: 6500,
            active_tipping_points: vec![],
            extreme_weather_count: 0,
            renewable_capacity_pct: 2500,
        },
        institutions: Default::default(),
        wars: Default::default(),
    };

    use civ_sim::sim::persistence::write_snapshot;
    write_snapshot(&pool, &world, true, &run_id).await.unwrap();

    let row = sqlx::query!(
        "SELECT tick, is_full, size_bytes, compressed_size FROM snapshots WHERE run_id = ? AND tick = 100",
        run_id
    )
    .fetch_one(&pool).await.unwrap();

    assert_eq!(row.tick, 100i64);
    assert_eq!(row.is_full, 1i64);
    assert!(row.size_bytes > 0);
    assert!(row.compressed_size > 0);
    assert!(row.compressed_size <= row.size_bytes);
}

#[tokio::test]
async fn test_conservation_trigger_fires() {
    let pool = setup_db().await;
    let run_id = Uuid::new_v4().to_string();

    sqlx::query!(
        "INSERT INTO runs (run_id, scenario_id, seed, status, params, version)
         VALUES (?, 'test', 1, 'running', '{}', '1.0.0')",
        run_id
    )
    .execute(&pool).await.unwrap();

    // Insert an unmatched Void-origin transfer (conservation violation)
    // Expect the trigger to raise ABORT
    let result = sqlx::query!(
        r#"
        INSERT INTO ledger_transfers
            (run_id, tick, transfer_id, from_actor_id, from_actor_type,
             to_actor_id, to_actor_type, amount_mj, currency_enum, transfer_type)
        VALUES (?, 1, ?, '00000000-0000-0000-0000-000000000000', 'void',
                ?, 'nation', 1000000, 'joule', 'unmatched_creation')
        "#,
        run_id,
        Uuid::new_v4().to_string(),
        Uuid::new_v4().to_string()
    )
    .execute(&pool).await;

    // The trigger should have rejected this insert
    assert!(result.is_err(), "Unmatched Void transfer should trigger conservation violation");
    let err_msg = result.unwrap_err().to_string();
    assert!(err_msg.contains("CONSERVATION_VIOLATION") || err_msg.contains("constraint"),
        "Error should mention conservation: {}", err_msg);
}
```

### 10.3 Performance Benchmarks

Performance benchmarks use Criterion.rs for statistical rigor. They run against a real SQLite database file (not in-memory) to measure realistic I/O performance.

```toml
# Cargo.toml
[[bench]]
name = "db_performance"
harness = false

[dev-dependencies]
criterion = { version = "0.5", features = ["async_tokio"] }
```

```rust
// benches/db_performance.rs

use criterion::{criterion_group, criterion_main, Criterion, BenchmarkId, Throughput};
use tokio::runtime::Runtime;
use civ_sim::db::{conn::open, migrations::run_pending};
use civ_sim::db::queries::bulk_insert_citizens;
use civ_sim::sim::types::*;
use uuid::Uuid;
use std::path::PathBuf;
use tempfile::tempdir;

fn make_citizen(run_id_str: &str, city_id: Uuid, nation_id: Uuid, tick: i64) -> CitizenRecord {
    CitizenRecord {
        citizen_id: CitizenId(Uuid::new_v4()),
        city_id: CityId(city_id),
        nation_id: NationId(nation_id),
        happiness: 6000,
        wealth_mj: 500_000,
        class: CitizenClass::Working,
        employment_status: EmploymentStatus::Employed,
        age_ticks: 500,
        joule_quota_mj: 1_000_000,
        dissatisfaction: 2000,
        migration_intent: false,
    }
}

/// Benchmark: Insert 1M citizen records.
/// Target: < 5 seconds total (200k rows/sec sustained write rate).
fn bench_bulk_citizen_insert(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    let dir = tempdir().unwrap();
    let db_path = dir.path().join("bench.db");

    let pool = rt.block_on(async {
        let pool = open(&db_path).await.unwrap();
        run_pending(&pool).await.unwrap();

        // Setup a run row
        sqlx::query!(
            "INSERT INTO runs (run_id, scenario_id, seed, status, params, version)
             VALUES ('bench-run', 'bench', 1, 'running', '{}', '1.0.0')"
        )
        .execute(&pool).await.unwrap();

        pool
    });

    let nation_id = Uuid::new_v4();
    let city_id = Uuid::new_v4();

    let mut group = c.benchmark_group("citizen_insert");
    group.throughput(Throughput::Elements(1_000_000));
    group.sample_size(3);  // large benchmark; 3 samples sufficient

    group.bench_function("1M_citizens", |b| {
        let mut tick = 0i64;
        b.iter(|| {
            tick += 1;
            let citizens: Vec<CitizenRecord> = (0..1_000_000)
                .map(|_| make_citizen("bench-run", city_id, nation_id, tick))
                .collect();
            rt.block_on(bulk_insert_citizens(&pool, "bench-run", tick, &citizens))
                .expect("bulk insert must succeed");
        });
    });

    group.finish();
}

/// Benchmark: Query last 100 ticks of nation state.
/// Target: < 100ms for run with 10k ticks, 10 nations.
fn bench_query_last_100_ticks(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    let dir = tempdir().unwrap();
    let db_path = dir.path().join("bench_query.db");

    let (pool, max_tick) = rt.block_on(async {
        let pool = open(&db_path).await.unwrap();
        run_pending(&pool).await.unwrap();

        sqlx::query!(
            "INSERT INTO runs (run_id, scenario_id, seed, status, params, version)
             VALUES ('qbench', 'bench', 1, 'running', '{}', '1.0.0')"
        )
        .execute(&pool).await.unwrap();

        // Insert 10k ticks × 10 nations
        let nation_ids: Vec<Uuid> = (0..10).map(|_| Uuid::new_v4()).collect();
        let ideology_json = "[0.1,-0.2,0.3,-0.4,0.5,-0.6,0.7,-0.8]";
        for tick in 0i64..10_000 {
            for nation_id in &nation_ids {
                sqlx::query!(
                    r#"INSERT INTO nations
                       (run_id, tick, nation_id, name, ideology_vector,
                        stability, legitimacy, population_total, population_growth,
                        gdp_millijoules, energy_surplus_mj, food_surplus_mu, gini_coefficient, at_war)
                       VALUES ('qbench', ?, ?, 'Nation', ?, 7000, 6000, 5000000, 100, 1000000, 50000, 30000, 3500, 0)"#,
                    tick, nation_id.to_string(), ideology_json
                )
                .execute(&pool).await.unwrap();
            }
        }
        (pool, 10_000i64)
    });

    let mut group = c.benchmark_group("nation_query");

    group.bench_function("last_100_ticks_10_nations", |b| {
        b.iter(|| {
            rt.block_on(async {
                let _rows = sqlx::query!(
                    "SELECT tick, nation_id, gdp_millijoules, stability FROM nations
                     WHERE run_id = 'qbench' AND tick BETWEEN ? AND ?
                     ORDER BY tick ASC",
                    max_tick - 100,
                    max_tick
                )
                .fetch_all(&pool).await.unwrap();
            });
        });
    });

    group.finish();
}

/// Benchmark: Market clearing query across 1000 ticks × 8 goods × 20 cities.
/// Target: < 500ms.
fn bench_market_price_query(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    let dir = tempdir().unwrap();
    let db_path = dir.path().join("bench_market.db");

    let pool = rt.block_on(async {
        let pool = open(&db_path).await.unwrap();
        run_pending(&pool).await.unwrap();

        sqlx::query!(
            "INSERT INTO runs (run_id, scenario_id, seed, status, params, version)
             VALUES ('mbench', 'bench', 1, 'running', '{}', '1.0.0')"
        )
        .execute(&pool).await.unwrap();

        let goods = ["energy", "food", "housing", "medicine",
                     "capital_goods", "consumer_goods", "labor", "carbon_credit"];
        let city_ids: Vec<Uuid> = (0..20).map(|_| Uuid::new_v4()).collect();

        for tick in 0i64..1_000 {
            for good in &goods {
                for city_id in &city_ids {
                    sqlx::query!(
                        r#"INSERT INTO markets
                           (run_id, tick, good_enum, city_id,
                            clearing_price_mj, bid_volume, ask_volume, cleared_volume,
                            unmet_demand, unmet_supply, regime)
                           VALUES ('mbench', ?, ?, ?, 500000, 10000, 9500, 9500, 500, 0, 'market')"#,
                        tick, good, city_id.to_string()
                    )
                    .execute(&pool).await.unwrap();
                }
            }
        }
        pool
    });

    let mut group = c.benchmark_group("market_query");

    group.bench_function("price_history_energy_all_cities_1000ticks", |b| {
        b.iter(|| {
            rt.block_on(async {
                let _rows = sqlx::query!(
                    "SELECT tick, city_id, clearing_price_mj, unmet_demand
                     FROM markets
                     WHERE run_id = 'mbench' AND good_enum = 'energy'
                     ORDER BY tick ASC"
                )
                .fetch_all(&pool).await.unwrap();
            });
        });
    });

    group.finish();
}

criterion_group!(
    benches,
    bench_bulk_citizen_insert,
    bench_query_last_100_ticks,
    bench_market_price_query
);
criterion_main!(benches);
```

**Performance targets and current baselines:**

| Benchmark | Target | Expected at |
|-----------|--------|-------------|
| Bulk insert 1M citizen rows | < 5s | NVMe SSD, SQLite WAL |
| Query last 100 ticks (10 nations) | < 100ms | After index warm-up |
| Market price history (8 goods × 20 cities × 1000 ticks) | < 500ms | With idx_markets_run_good |
| Full hash chain verification (10k-tick run) | < 60s | Single-threaded replay |
| Parquet export (full run, 10k ticks) | < 30s | NVMe, arrow2 zstd |
| Snapshot write (full, compressed) | < 50ms | Per tick boundary |

### 10.4 Test Fixtures

Three SQL fixture files provide standard test datasets. They are loaded by integration tests and can be loaded manually for development.

**`tests/fixtures/minimal_run.sql`**

A minimal valid run with 2 nations, 2 cities, 10 citizens, 10 ticks. Used for unit tests and fast CI.

```sql
-- tests/fixtures/minimal_run.sql
-- Minimal run fixture: 2 nations, 2 cities, 10 citizens, 10 ticks
-- Run ID: 00000000-0000-0000-0000-000000000001

INSERT INTO runs (run_id, scenario_id, seed, start_tick, end_tick, status, params, version)
VALUES ('00000000-0000-0000-0000-000000000001', 'minimal-scenario', 42, 0, 9, 'completed', '{}', '1.0.0');

INSERT INTO nations (run_id, tick, nation_id, name, ideology_vector, stability, legitimacy,
                     population_total, population_growth, gdp_millijoules, energy_surplus_mj,
                     food_surplus_mu, gini_coefficient, at_war)
VALUES
  ('00000000-0000-0000-0000-000000000001', 0,
   'aaaaaaaa-0000-0000-0000-000000000001', 'Nation Alpha',
   '[0.5,-0.3,0.1,-0.2,0.0,0.4,-0.1,0.2]',
   7000, 6500, 1000000, 500, 500000000, 10000, 50000, 3200, 0),

  ('00000000-0000-0000-0000-000000000001', 0,
   'bbbbbbbb-0000-0000-0000-000000000001', 'Nation Beta',
   '[-0.4,0.6,-0.2,0.1,-0.3,-0.5,0.3,-0.1]',
   6000, 7000, 800000, 300, 400000000, -5000, 40000, 4100, 0);

INSERT INTO cities (run_id, tick, city_id, nation_id, name, position_x, position_y,
                    population, energy_balance_mj, food_balance_mu, housing_capacity,
                    employed_count, unemployed_count, happiness_avg, infrastructure_level, is_capital)
VALUES
  ('00000000-0000-0000-0000-000000000001', 0,
   'cccccccc-0000-0000-0000-000000000001', 'aaaaaaaa-0000-0000-0000-000000000001',
   'Alpha Capital', 10, 10, 800000, 10000, 50000, 1000000, 600000, 50000, 6500, 70, 1),

  ('00000000-0000-0000-0000-000000000001', 0,
   'dddddddd-0000-0000-0000-000000000001', 'bbbbbbbb-0000-0000-0000-000000000001',
   'Beta Capital', 30, 10, 650000, -5000, 40000, 900000, 480000, 60000, 5800, 55, 1);

INSERT INTO climate_state (run_id, tick, global_temp_offset_mc, co2_ppm_mc, sea_level_rise_mm,
                            ocean_acidification_mu, arctic_ice_pct, active_tipping_points,
                            extreme_weather_count, renewable_capacity_pct)
VALUES ('00000000-0000-0000-0000-000000000001', 0,
        1200, 420000, 200, 150000, 7500, '[]', 0, 2000);

-- Additional ticks 1-9 follow same pattern (omitted for brevity in this header comment)
-- Full fixture file contains all 10 ticks.
```

**`tests/fixtures/full_scenario.sql`**

A full research scenario with 5 nations, 20 cities, 50k citizens, 1000 ticks, including war and climate events. Used for integration and performance tests. File size: ~50 MB. Generated by `civ fixtures generate --preset full`.

**`tests/fixtures/stress_test_run.sql`**

Extreme-scale fixture: 10 nations, 100 cities, 1M citizens at tick 0 only. Used for write-path performance tests. Generated by `civ fixtures generate --preset stress`.

**Fixture loading in tests:**

```rust
// tests/common/fixtures.rs

pub async fn load_fixture(pool: &SqlitePool, fixture_name: &str) {
    let fixture_sql = match fixture_name {
        "minimal_run" => include_str!("../fixtures/minimal_run.sql"),
        "full_scenario" => include_str!("../fixtures/full_scenario.sql"),
        "stress_test_run" => include_str!("../fixtures/stress_test_run.sql"),
        _ => panic!("Unknown fixture: {}", fixture_name),
    };

    // Execute each statement
    for stmt in fixture_sql.split(';').map(|s| s.trim()).filter(|s| !s.is_empty()) {
        sqlx::query(stmt).execute(pool).await
            .unwrap_or_else(|e| panic!("Fixture statement failed: {e}\nSQL: {stmt}"));
    }
}

// Usage in tests:
// let pool = setup_db().await;
// load_fixture(&pool, "minimal_run").await;
// let nations = fetch_nations_at_tick(&pool, "00000000-...", 0).await.unwrap();
// assert_eq!(nations.len(), 2);
```

---

## Appendix A — Enum Value Sets

### A.1 `runs.status`

| Value | Meaning |
|-------|---------|
| `running` | Simulation is actively executing |
| `completed` | Simulation reached tick_limit or a win condition |
| `failed` | Simulation aborted due to an error (conservation violation, OOM, etc.) |
| `paused` | Simulation is suspended; can be resumed |
| `archived` | Run data has been archived to .civreplay; SQLite rows pruned |

### A.2 `citizens.class_enum`

| Value | Description |
|-------|-------------|
| `subsistence` | At or below basic survival threshold; no discretionary resources |
| `working` | Basic needs met; employed in primary/secondary sector |
| `middle` | Comfortable; discretionary income; stable employment |
| `professional` | Knowledge work; high income; low dissatisfaction typically |
| `capitalist` | Owns means of production; income from capital |
| `elite` | Ruling class; high wealth; high political influence |
| `lumpenproletariat` | Chronically unemployed; disconnected from productive economy |

### A.3 `citizens.employment_status`

| Value | Description |
|-------|-------------|
| `employed` | Working for a wage or salary |
| `unemployed` | Seeking work; not employed |
| `self_employed` | Operating own business or farm |
| `retired` | No longer in labor force due to age |
| `student` | In education; not in labor force |
| `disabled` | Unable to work |

### A.4 `ledger_transfers.currency_enum`

| Value | Description | Unit |
|-------|-------------|------|
| `joule` | Physical energy currency (joule economy regimes) | Millijoules (mJ) |
| `fiat` | Government-issued currency (market regimes) | Millicredits |
| `quota` | Planned allocation quota (planned economy regimes) | Quota units |
| `labor_credit` | Labor-time certificates | Milliminutes of labor |
| `carbon_credit` | Carbon emission permits | Millitons CO2e |

### A.5 `markets.good_enum`

| Value | Description | Physical nature |
|-------|-------------|----------------|
| `energy` | Electrical/thermal energy | Physical (joules) |
| `food` | Agricultural and processed food | Physical (calories) |
| `housing` | Residential units | Physical (unit-months) |
| `medicine` | Healthcare goods and services | Physical/service |
| `capital_goods` | Industrial equipment | Physical |
| `consumer_goods` | Non-essential manufactured goods | Physical |
| `labor` | Work-hours | Service |
| `carbon_credit` | Emission permits | Financial |

### A.6 `institutions.inst_type`

| Value | Description |
|-------|-------------|
| `central_bank` | Monetary policy, money supply control |
| `planning_bureau` | Resource allocation planning in planned/joule economies |
| `regulatory_agency` | Market regulation, standards enforcement |
| `court` | Legal adjudication, contract enforcement |
| `military_command` | Armed forces coordination |
| `trade_union` | Labor collective bargaining |
| `religious_body` | Cultural/moral authority; legitimacy source |
| `media_organization` | Information production; narrative control |
| `environmental_agency` | Environmental regulation; climate policy |
| `taxation_authority` | Revenue collection; fiscal enforcement |

### A.7 `climate_state.active_tipping_points` (JSON array values)

| Value | Description | Activation threshold (approx.) |
|-------|-------------|-------------------------------|
| `west_antarctic_ice_sheet_collapse` | WAIS destabilization; multi-meter SLR | +1.5°C |
| `greenland_ice_sheet_collapse` | GIS melt; 7m SLR over centuries | +1.5°C |
| `amazon_dieback` | Rainforest transition to savanna | +2.0°C |
| `permafrost_methane_release` | Siberian/arctic CH4 emissions | +1.5°C |
| `atlantic_circulation_collapse` | AMOC shutdown; European cooling | +2.0°C |
| `coral_reef_die_off` | Mass bleaching; fishery collapse | +1.5°C |
| `boreal_forest_dieback` | Taiga to grassland transition | +3.0°C |
| `monsoon_disruption` | ITCZ shift; Asian monsoon failure | +2.5°C |

### A.8 `rng_seeds.phase_enum`

| Value | Description | Module |
|-------|-------------|--------|
| `stochastic_events` | Random event selection and outcome | `sim::events` |
| `migration` | Citizen migration destination sampling | `sim::city` |
| `war_resolution` | Battle outcome dice rolls | `sim::war` |
| `climate_perturbation` | Weather variability and tipping point rolls | `sim::climate` |
| `citizen_behavior` | Individual citizen decision sampling | `sim::citizen` |
| `institution_drift` | Capture and legitimacy random walk | `sim::institution` |

---

## Appendix B — Cross-Reference Table

| Entity | SQL Table | Rust Struct | In-memory ECS Component | .civreplay presence |
|--------|-----------|-------------|------------------------|---------------------|
| Simulation Run | `runs` | `SimRun` | — (metadata only) | Header |
| State Snapshot | `snapshots` | `Snapshot` | `WorldSnap` / `WorldDelta` | Per-frame |
| Simulation Event | `events` | `SimEvent` | `Vec<SimEvent>` in `World.event_log` | Every event |
| Nation | `nations` | `NationState` | `BTreeMap<NationId, NationState>` | Via events |
| City | `cities` | `CityState` | `BTreeMap<CityId, CityState>` | Via events |
| Citizen | `citizens` | `CitizenRecord` | `BTreeMap<CitizenId, CitizenRecord>` | Via events (sampled) |
| Ledger Transfer | `ledger_transfers` | `LedgerTransfer` | `LedgerState.transfers` | Via events |
| Market Clearing | `markets` | `MarketClearing` | `BTreeMap<(Good, CityId), MarketClearing>` | Via events |
| Climate | `climate_state` | `ClimateStateRow` / `ClimateState` | `World.climate` | Via events |
| Institution | `institutions` | `InstitutionState` | `BTreeMap<InstId, InstitutionState>` | Via events |
| War | `wars` | `WarRecord` | `BTreeMap<WarId, WarRecord>` | Via events |
| Research Run | `research_runs` | `ResearchRun` | — (metadata) | Header |
| Replay Event | `replay_events` | `ReplayEvent` | — (archive) | Primary |
| Metric | `metrics_timeseries` | `MetricRow` | — (derived) | No |
| RNG Seed | `rng_seeds` | `RngSeedRow` | `World.rng` (live state) | No |
| Schema Version | `schema_versions` | — (migration tooling) | — | No |

---

## Appendix C — SQLite File Size Estimates

Reference estimates for typical simulation runs at various scales. All estimates assume zstd compression for snapshot_bytes.

| Scale | Nations | Cities | Citizens | Ticks | SQLite Size | .civreplay Size |
|-------|---------|--------|---------|-------|-------------|----------------|
| Minimal | 2 | 4 | 10k | 1k | ~50 MB | ~5 MB |
| Small | 4 | 16 | 100k | 5k | ~2 GB | ~200 MB |
| Medium | 8 | 40 | 500k | 10k | ~15 GB | ~1 GB |
| Large | 16 | 100 | 2M | 20k | ~80 GB | ~5 GB |
| Research batch (10 runs) | 4 | 20 | 200k | 10k each | ~60 GB | ~5 GB total |

**Notes:**
- Citizens table dominates storage at scale. Enable pruning (500-tick retention) for runs beyond "Small".
- .civreplay is ~10x smaller than SQLite due to event-only format (no per-tick full rows).
- For research batches with parameter sweeps, use the parquet export path for analysis. SQLite files grow proportionally; consider PostgreSQL for concurrent multi-user research.

---

## Appendix D — Connection Pool Configuration

### D.1 SQLite Connection Pool (SQLx)

```rust
// src/db/conn.rs

use sqlx::sqlite::{SqliteConnectOptions, SqlitePoolOptions, SqliteJournalMode, SqliteSynchronous};
use std::path::Path;

pub async fn open(path: &Path) -> Result<SqlitePool, DbError> {
    let options = SqliteConnectOptions::new()
        .filename(path)
        .create_if_missing(true)
        .journal_mode(SqliteJournalMode::Wal)
        .synchronous(SqliteSynchronous::Normal)
        .foreign_keys(true)
        .pragma("cache_size", "-65536")
        .pragma("temp_store", "MEMORY")
        .pragma("mmap_size", "8589934592")
        .pragma("wal_autocheckpoint", "1000");

    // SQLite with WAL supports 1 writer + N readers concurrently.
    // max_connections = 1 writer + up to 7 readers for typical research workload.
    SqlitePoolOptions::new()
        .max_connections(8)
        .min_connections(1)
        .acquire_timeout(std::time::Duration::from_secs(30))
        .idle_timeout(std::time::Duration::from_secs(600))
        .connect_with(options)
        .await
        .map_err(DbError::Connection)
}

pub async fn open_in_memory() -> Result<SqlitePool, DbError> {
    let options = SqliteConnectOptions::new()
        .filename(":memory:")
        .journal_mode(SqliteJournalMode::Memory)
        .foreign_keys(true);

    SqlitePoolOptions::new()
        .max_connections(1)   // in-memory DB is connection-local in SQLite
        .connect_with(options)
        .await
        .map_err(DbError::Connection)
}
```

### D.2 PostgreSQL Connection Pool (SQLx, multi-user mode)

```rust
// src/db/pg_conn.rs (multi-user research mode)

use sqlx::postgres::{PgConnectOptions, PgPoolOptions, PgSslMode};

pub async fn open_pg(database_url: &str) -> Result<PgPool, DbError> {
    PgPoolOptions::new()
        .max_connections(20)
        .min_connections(2)
        .acquire_timeout(std::time::Duration::from_secs(10))
        .idle_timeout(std::time::Duration::from_secs(600))
        .max_lifetime(std::time::Duration::from_secs(1800))
        .connect(database_url)
        .await
        .map_err(DbError::Connection)
}
```

---

*End of CivLab Data Model and Database Specification*

**Spec ID:** SPEC-DATA-MODEL-CIV-001 | **Version:** 1.0.0 | **Date:** 2026-02-21


---

## Source: models/civ-sim/OPS_GOVERNANCE_SPEC.md

# Civ-Sim Ops/Governance Specification

**Status:** Active
**Version:** 1.0.0
**Owner:** CivLab Platform Team
**Last Updated:** 2026-02-21
**Applies To:** civlab-sim crate, civlab-server, civlab-clients (web, mobile, desktop)

---

## Table of Contents

1. [Overview and Scope](#1-overview-and-scope)
2. [Quality Gate System](#2-quality-gate-system)
3. [CI/CD Pipeline](#3-cicd-pipeline)
4. [Spec-First Governance](#4-spec-first-governance)
5. [Versioned Policy and Metric Definitions](#5-versioned-policy-and-metric-definitions)
6. [Runtime Governance and Guardrails](#6-runtime-governance-and-guardrails)
7. [Monitoring and Observability](#7-monitoring-and-observability)
8. [Artifact Integrity](#8-artifact-integrity)
9. [Storage Governance](#9-storage-governance)
10. [Dependency Governance](#10-dependency-governance)
11. [Risk Controls](#11-risk-controls)
12. [Compliance and Auditability](#12-compliance-and-auditability)

---

## 1. Overview and Scope

### 1.1 Operational Governance Philosophy

CivLab's operational governance is built on four axioms:

1. **Determinism is a hard contract, not a best-effort property.** A single non-deterministic output anywhere in the simulation loop is a P0 incident. All governance controls flow from this axiom.
2. **Spec precedes code.** No production code is written without a corresponding functional requirement and test. Governance documents are authoritative; code is a downstream artifact.
3. **Fail loud, never silently.** All error conditions surface as structured log entries, metric counter increments, and where applicable freeze-mode triggers. No fallback paths that hide defects.
4. **Artifacts are signed and auditable.** Every simulation output, scenario config change, and runtime intervention is captured in an immutable audit log with cryptographic integrity guarantees.

### 1.2 Scope

This document governs:

| Component | Scope Included |
|---|---|
| `civlab-sim` | Rust simulation engine crate: tick loop, ECS world, RNG, determinism rules |
| `civlab-server` | JSON-RPC WebSocket server, scenario orchestration, storage layer |
| `civlab-clients` | Web (Pixi.js v8 + React 19), Mobile (TBD), Desktop (Bevy 3D): client-side governance |
| `civlab-mods` | WASM mod sandbox (wasmtime 26.x): sandboxing, resource limits |
| CI/CD pipeline | GitHub Actions workflows, artifact signing, regression suites |
| Storage | SQLite (embedded), PostgreSQL (server), backup and lifecycle policy |
| Monitoring | Prometheus metrics, Grafana dashboards, alerting rules |

Out of scope: billing infrastructure, end-user authentication flows (delegated to WorkOS/AuthKit governance).

### 1.3 Ownership Map

| Domain | Primary Owner | Secondary Owner | Escalation |
|---|---|---|---|
| Simulation engine correctness | Sim Team Lead | Rust Guild | Platform CTO |
| CI/CD pipeline | Platform Team | DevOps | Platform CTO |
| Schema governance | Sim Architect | Spec Team | Platform CTO |
| Storage and backup | Infra Team | Platform Team | Platform CTO |
| Security and dependency audit | Security Guild | Platform Team | CISO |
| Monitoring and alerting | Platform Team | Sim Team Lead | On-call engineer |
| Mod sandbox policy | Security Guild | Sim Architect | CISO |

### 1.4 Governance Layers

Governance flows through four ordered layers. A violation at any layer is a blocking defect:

```
+---------------------------------------------------------+
|  Layer 1: SPEC LAYER                                    |
|  PRD.md, FUNCTIONAL_REQUIREMENTS.md, ADR.md, this doc  |
|  Authoritative. All downstream layers must conform.     |
+---------------------------------------------------------+
|  Layer 2: CODE LAYER                                    |
|  Rust impl, JSON Schema, TOML configs, Taskfile.yml     |
|  Must satisfy all FR SHALL statements.                  |
|  Gate: clippy -D warnings, rustfmt, schema validators   |
+---------------------------------------------------------+
|  Layer 3: RUNTIME LAYER                                 |
|  Tick loop, ECS world, guardrails, freeze mode          |
|  Must satisfy determinism rules D1-D7.                  |
|  Gate: BLAKE3 hash chain, double-run, seed-sweep tests  |
+---------------------------------------------------------+
|  Layer 4: ARTIFACT LAYER                                |
|  Signed reports, replay bundles, audit logs             |
|  Must be tamper-evident, reproducible, versioned.       |
|  Gate: Ed25519 signatures, retention policy compliance  |
+---------------------------------------------------------+
```

Changes travel top-to-bottom: spec change triggers code change triggers runtime re-validation triggers re-signed artifacts. A code change without a corresponding spec change is a governance violation and will be flagged by pre-commit hooks.

### 1.5 Determinism Rules Reference (D1-D7)

The following rules are referenced throughout this document. Each quality gate maps to one or more of these rules:

| Rule ID | Name | Description |
|---|---|---|
| D1 | Pure Functions | All tick-advancing systems are pure functions of (World, Tick, Seed). No hidden state. |
| D2 | No System Time | `std::time::SystemTime`, `std::time::Instant`, and all wall-clock reads are forbidden in sim code. |
| D3 | No Float Comparison | Floating-point equality or ordering in game logic is forbidden. All rates use `FixedI32<U16>`; all energy/GDP use `i64` newtypes. |
| D4 | No Global Mutable State | No `static mut`, no `OnceLock` that mutates after initialization, no thread-local state in sim code. |
| D5 | Deterministic Ordering | ECS system ordering is declared explicit and total. No reliance on hash map iteration order in output-affecting code. |
| D6 | Seeded RNG Only | Only `ChaCha20Rng` seeded from the scenario config is permitted. No `rand::thread_rng()` or OS entropy sources in sim code. |
| D7 | No I/O in Tick Loop | No filesystem, network, or database access inside the 100ms tick loop. All I/O is pre-loaded or post-processed. |

---

## 2. Quality Gate System

### 2.1 Gate Overview

Quality gates are enforced at three checkpoints: pre-commit (local), CI (pull request), and nightly (regression). A gate failure at any checkpoint is a hard blocker with no bypass without a documented exception approved by the primary owner.

| Gate ID | Name | Checkpoint | Blocks |
|---|---|---|---|
| QG-01 | Schema Validation | pre-commit + CI | PR merge |
| QG-02 | Determinism Double-Run | CI | PR merge |
| QG-03 | Seed-Sweep Determinism | CI nightly | Nightly green |
| QG-04 | Cross-Platform Hash Parity | CI nightly | Nightly green |
| QG-05 | Integration Test Matrix | CI | PR merge |
| QG-06 | Replay Consistency | CI | PR merge |
| QG-07 | Tick Latency Gate | CI | PR merge |
| QG-08 | Memory Ceiling Gate | CI | PR merge |
| QG-09 | Lint and Static Analysis | pre-commit + CI | PR merge |
| QG-10 | Dependency Audit | CI weekly | Weekly green |

### 2.2 Schema Validation (QG-01)

#### 2.2.1 JSON Schema Coverage

Every external input to the simulation is validated against a versioned JSON Schema before entering the simulation layer. Schema files live in `schemas/` at the repository root:

```
schemas/
  scenario/
    v1/
      scenario.schema.json       # Top-level scenario config schema
      terrain.schema.json        # Hex grid terrain descriptor
      civilization.schema.json   # Starting civ configuration
      policy.schema.json         # Policy bundle schema
      mod-manifest.schema.json   # WASM mod manifest
  rpc/
    v1/
      request.schema.json        # JSON-RPC request envelope
      response.schema.json       # JSON-RPC response envelope
      notifications.schema.json  # Server-push notification shapes
  metrics/
    v1/
      metric-definition.schema.json  # Metric definition format
```

All schemas include `$schema`, `$id`, `title`, `description`, and `version` fields. Breaking changes to any schema require a version directory bump (`v1` to `v2`) and a corresponding ADR.

#### 2.2.2 Custom Rust Validators

JSON Schema handles structural validation. Domain invariants are enforced by a Rust `ScenarioValidator` that runs after JSON Schema passes:

```rust
// crates/civlab-sim/src/validation/scenario.rs

pub struct ScenarioValidator;

impl ScenarioValidator {
    /// Validates all domain invariants that JSON Schema cannot express.
    /// Returns a Vec<ValidationError>; empty means valid.
    /// @trace FR-VAL-001
    pub fn validate(scenario: &ScenarioConfig) -> Vec<ValidationError> {
        let mut errors = Vec::new();
        Self::validate_seed_range(scenario, &mut errors);
        Self::validate_hex_grid_bounds(scenario, &mut errors);
        Self::validate_fixed_point_budgets(scenario, &mut errors);
        Self::validate_policy_balance(scenario, &mut errors);
        Self::validate_mod_manifest_hashes(scenario, &mut errors);
        errors
    }

    fn validate_seed_range(s: &ScenarioConfig, errors: &mut Vec<ValidationError>) {
        // Seed 0 is reserved for internal testing; production scenarios must use seed > 0
        if s.rng_seed == 0 {
            errors.push(ValidationError::new(
                "rng_seed",
                "seed 0 is reserved for testing; use seed >= 1 in production",
            ));
        }
    }

    fn validate_hex_grid_bounds(s: &ScenarioConfig, errors: &mut Vec<ValidationError>) {
        const MAX_RADIUS: u32 = 512;
        if s.hex_grid.radius > MAX_RADIUS {
            errors.push(ValidationError::new(
                "hex_grid.radius",
                format!("radius {} exceeds maximum {}", s.hex_grid.radius, MAX_RADIUS),
            ));
        }
    }

    fn validate_fixed_point_budgets(s: &ScenarioConfig, errors: &mut Vec<ValidationError>) {
        // Starting energy must be representable as KiloJoules (i64, non-negative)
        if s.starting_energy_kj < 0 {
            errors.push(ValidationError::new(
                "starting_energy_kj",
                "must be non-negative",
            ));
        }
        // Starting GDP must be representable as MilliCredits (i64, non-negative)
        if s.starting_gdp_mc < 0 {
            errors.push(ValidationError::new(
                "starting_gdp_mc",
                "must be non-negative",
            ));
        }
    }

    fn validate_policy_balance(s: &ScenarioConfig, errors: &mut Vec<ValidationError>) {
        // Sum of all policy allocation weights must equal exactly 1_000_000 ppm
        // (parts per million = fixed-point 1.0)
        let weight_sum: i64 = s.policies.iter().map(|p| p.weight_ppm).sum();
        if weight_sum != 1_000_000 {
            errors.push(ValidationError::new(
                "policies",
                format!(
                    "policy weights sum to {} ppm; must equal exactly 1_000_000",
                    weight_sum
                ),
            ));
        }
    }

    fn validate_mod_manifest_hashes(s: &ScenarioConfig, errors: &mut Vec<ValidationError>) {
        for m in &s.mods {
            if m.blake3_hash.is_empty() {
                errors.push(ValidationError::new(
                    "mods[].blake3_hash",
                    format!("mod '{}' missing required BLAKE3 hash", m.id),
                ));
            }
        }
    }
}
```

The validator is called from both the server's scenario-load path and from the `validate-scenario` Taskfile target.

#### 2.2.3 Taskfile Targets for QG-01

```yaml
# Taskfile.yml (excerpt: schema validation targets)

version: '3'

tasks:
  validate-scenario:
    desc: "Validate a scenario config file against JSON Schema and Rust domain validators"
    cmds:
      - cargo run -p civlab-cli -- validate-scenario --file {{.FILE}}
    requires:
      vars: [FILE]

  validate-schemas:
    desc: "Validate all schema files in schemas/ for JSON Schema correctness"
    cmds:
      - npx ajv-cli validate --allow-union-types -s "schemas/**/*.schema.json"
    preconditions:
      - sh: "command -v npx"
        msg: "npx must be installed"

  validate-all:
    desc: "Run all schema validation checks (schemas + sample fixtures)"
    deps: [validate-schemas]
    cmds:
      - |
        for fixture in tests/fixtures/scenarios/*.toml; do
          echo "Validating $fixture..."
          cargo run -p civlab-cli -- validate-scenario --file "$fixture"
        done
```

### 2.3 Determinism Test Harness (QG-02, QG-03, QG-04)

#### 2.3.1 Double-Run Check (QG-02)

The double-run check runs every scenario in the test fixture set twice with identical seeds and asserts byte-identical output. This catches any non-determinism introduced by OS scheduling, allocator layout, or hidden shared state.

```rust
// crates/civlab-sim/tests/determinism_double_run.rs
// @trace FR-DET-002

use civlab_sim::{SimulationEngine, ScenarioConfig};
use std::path::PathBuf;

fn load_fixture(name: &str) -> ScenarioConfig {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../tests/fixtures/scenarios")
        .join(name);
    ScenarioConfig::from_toml_file(&path).expect("fixture must parse")
}

/// Runs a scenario for n ticks and returns the final BLAKE3 hash chain root.
fn run_n_ticks(config: &ScenarioConfig, n: u64) -> [u8; 32] {
    let mut engine = SimulationEngine::new(config.clone());
    for _ in 0..n {
        engine.tick();
    }
    engine.hash_chain_root()
}

#[test]
fn determinism_double_run_standard_scenario() {
    let config = load_fixture("standard_100hex.toml");
    let run_a = run_n_ticks(&config, 1_000);
    let run_b = run_n_ticks(&config, 1_000);
    assert_eq!(run_a, run_b, "double-run mismatch: non-determinism detected");
}

#[test]
fn determinism_double_run_large_scenario() {
    let config = load_fixture("large_512hex.toml");
    let run_a = run_n_ticks(&config, 500);
    let run_b = run_n_ticks(&config, 500);
    assert_eq!(run_a, run_b, "double-run mismatch on large scenario");
}

#[test]
fn determinism_double_run_with_mods() {
    let config = load_fixture("with_mods_standard.toml");
    let run_a = run_n_ticks(&config, 200);
    let run_b = run_n_ticks(&config, 200);
    assert_eq!(run_a, run_b, "double-run mismatch with active mods");
}
```

#### 2.3.2 Seed-Sweep Test (QG-03)

The seed-sweep test runs a reduced scenario (50 ticks) across 256 distinct seeds and asserts that each seed produces a unique but internally consistent hash chain. This detects seed-contamination bugs where one run's RNG state bleeds into another.

```rust
// crates/civlab-sim/tests/determinism_seed_sweep.rs
// @trace FR-DET-006

#[test]
fn seed_sweep_256_seeds_no_cross_contamination() {
    let base_config = load_fixture("minimal_seed_sweep.toml");
    let mut results: std::collections::HashMap<[u8; 32], u64> = Default::default();

    for seed in 1u64..=256 {
        let mut config = base_config.clone();
        config.rng_seed = seed;
        let hash = run_n_ticks(&config, 50);

        if let Some(prior_seed) = results.get(&hash) {
            panic!(
                "seed {} produced same hash as seed {} — cross-contamination detected",
                seed, prior_seed
            );
        }
        results.insert(hash, seed);
    }
    assert_eq!(results.len(), 256, "expected 256 unique hashes for 256 seeds");
}

#[test]
fn seed_sweep_reproducibility_spot_check() {
    // Pick 16 seeds, run twice each, verify identical hash
    let base_config = load_fixture("minimal_seed_sweep.toml");
    for seed in [
        1u64, 7, 42, 100, 128, 200, 255, 256,
        512, 1000, 9999, 65535, 100_000, 1_000_000, u64::MAX / 2, u64::MAX - 1,
    ] {
        let mut config = base_config.clone();
        config.rng_seed = seed;
        let run_a = run_n_ticks(&config, 50);
        let run_b = run_n_ticks(&config, 50);
        assert_eq!(run_a, run_b, "seed {} not reproducible", seed);
    }
}
```

#### 2.3.3 Cross-Platform Hash Parity (QG-04)

The nightly CI matrix runs the double-run test suite on Linux x86_64, macOS arm64, and Windows x86_64. A GitHub Actions artifact uploads the hash chain output from each platform. A post-matrix comparison job asserts all three outputs are byte-identical.

```yaml
# .github/workflows/nightly-cross-platform.yml (excerpt: comparison job)

  cross-platform-hash-compare:
    needs: [test-linux, test-macos, test-windows]
    runs-on: ubuntu-latest
    steps:
      - uses: actions/download-artifact@v4
        with:
          pattern: hash-chain-output-*
          merge-multiple: true

      - name: Compare hash chain outputs across platforms
        run: |
          LINUX=$(cat hash-chain-output-linux/standard_100hex_1000ticks.hex)
          MACOS=$(cat hash-chain-output-macos/standard_100hex_1000ticks.hex)
          WINDOWS=$(cat hash-chain-output-windows/standard_100hex_1000ticks.hex)
          if [ "$LINUX" != "$MACOS" ] || [ "$LINUX" != "$WINDOWS" ]; then
            echo "CROSS-PLATFORM DETERMINISM FAILURE"
            echo "Linux:   $LINUX"
            echo "macOS:   $MACOS"
            echo "Windows: $WINDOWS"
            exit 1
          fi
          echo "Cross-platform hash parity: PASS"
```

#### 2.3.4 Taskfile Targets for Determinism

```yaml
  det-double-run:
    desc: "Run determinism double-run tests (QG-02)"
    cmds:
      - cargo test -p civlab-sim --test determinism_double_run -- --nocapture

  det-seed-sweep:
    desc: "Run seed-sweep determinism tests — slow, nightly only (QG-03)"
    cmds:
      - cargo test -p civlab-sim --test determinism_seed_sweep -- --nocapture

  det-all:
    desc: "Run all local determinism tests"
    deps: [det-double-run]
    cmds:
      - echo "Seed sweep runs in CI nightly only; use det-seed-sweep to run locally"
```

### 2.4 Integration Test Matrix (QG-05)

The integration test matrix covers all D1-D7 rules with at least two targeted test cases per rule. Tests carry a `// @trace FR-DET-NNN` annotation for FR traceability.

| Rule | Test File | Test Count | FR Trace |
|---|---|---|---|
| D1 Pure Functions | `tests/integration/d1_pure_functions.rs` | 4 | FR-DET-001 |
| D2 No System Time | `tests/integration/d2_no_system_time.rs` | 3 | FR-DET-002 |
| D3 No Float Compare | `tests/integration/d3_no_float_compare.rs` | 5 | FR-DET-003 |
| D4 No Global Mut | `tests/integration/d4_no_global_mut.rs` | 3 | FR-DET-004 |
| D5 Deterministic Ordering | `tests/integration/d5_deterministic_ordering.rs` | 4 | FR-DET-005 |
| D6 Seeded RNG | `tests/integration/d6_seeded_rng.rs` | 6 | FR-DET-006 |
| D7 No I/O in Tick | `tests/integration/d7_no_io_in_tick.rs` | 3 | FR-DET-007 |

Example D7 integration test asserting the tick function does not invoke I/O syscalls:

```rust
// crates/civlab-sim/tests/integration/d7_no_io_in_tick.rs
// @trace FR-DET-007

#[cfg(target_os = "linux")]
#[test]
fn d7_tick_loop_no_io_syscalls() {
    // Install a seccomp filter that panics on any read/write/open syscall
    // during the tick execution window. This is Linux-only and CI-enforced.
    let config = load_fixture("minimal_seed_sweep.toml");
    let mut engine = SimulationEngine::new(config);

    civlab_test_utils::with_io_syscall_trap(|| {
        engine.tick(); // Must complete without triggering the trap
    });
}
```

```yaml
  test-integration-matrix:
    desc: "Run full D1-D7 integration test matrix (QG-05)"
    cmds:
      - cargo test -p civlab-sim --tests -- --nocapture 2>&1 | tee /tmp/integration-matrix.log
      - |
        if grep -q "FAILED" /tmp/integration-matrix.log; then
          echo "Integration test failures:"
          grep "FAILED" /tmp/integration-matrix.log
          exit 1
        fi
        echo "Integration matrix: PASS"
```

### 2.5 Replay Consistency (QG-06)

#### 2.5.1 BLAKE3 Hash Chain

Every tick produces a BLAKE3 hash of the serialized ECS world state. Each hash chains into the next by including the prior hash as input:

```rust
// crates/civlab-sim/src/hash_chain.rs
// @trace FR-REP-001

use blake3::Hasher;

pub struct HashChain {
    current: [u8; 32],
    tick: u64,
}

impl HashChain {
    pub fn new(seed: u64) -> Self {
        let mut hasher = Hasher::new();
        hasher.update(b"civlab-sim-v1");
        hasher.update(&seed.to_le_bytes());
        Self {
            current: hasher.finalize().into(),
            tick: 0,
        }
    }

    /// Advance the chain by hashing the current world state snapshot.
    /// `world_snapshot` is the canonical deterministic serialization of the ECS world.
    pub fn advance(&mut self, world_snapshot: &[u8]) -> [u8; 32] {
        let mut hasher = Hasher::new();
        hasher.update(&self.current);             // chain link
        hasher.update(&self.tick.to_le_bytes());  // tick counter prevents hash reuse
        hasher.update(world_snapshot);            // world state
        let next = hasher.finalize().into();
        self.current = next;
        self.tick += 1;
        next
    }

    pub fn root(&self) -> [u8; 32] {
        self.current
    }
}
```

#### 2.5.2 Replay Verification Test

```rust
// crates/civlab-sim/tests/replay_consistency.rs
// @trace FR-REP-001

#[test]
fn replay_produces_identical_hash_chain() {
    let config = load_fixture("standard_100hex.toml");
    let tick_count = 1_000usize;

    // Initial run: capture per-tick hashes
    let mut engine = SimulationEngine::new(config.clone());
    let mut initial_hashes: Vec<[u8; 32]> = Vec::with_capacity(tick_count);
    for _ in 0..tick_count {
        engine.tick();
        initial_hashes.push(engine.current_hash());
    }

    // Replay run: reconstruct from replay bundle (tick-0 state + seed) and re-execute
    let replay_bundle = engine.export_replay_bundle();
    let mut replay_engine = SimulationEngine::from_replay_bundle(&replay_bundle);
    let mut replay_hashes: Vec<[u8; 32]> = Vec::with_capacity(tick_count);
    for _ in 0..tick_count {
        replay_engine.tick();
        replay_hashes.push(replay_engine.current_hash());
    }

    assert_eq!(
        initial_hashes, replay_hashes,
        "replay hash chain diverged from original run"
    );
}
```

#### 2.5.3 Tick-by-Tick Comparison Tool

The `civlab-cli replay-diff` command performs tick-by-tick comparison between two replay bundles and reports the first divergence point:

```bash
civlab-cli replay-diff \
  --bundle-a replays/run-001.civreplay \
  --bundle-b replays/run-002.civreplay \
  --output divergence-report.json
```

Output format:

```json
{
  "status": "diverged",
  "first_divergent_tick": 847,
  "bundle_a_hash_at_tick": "a3f7c2b1e9d045f88c2a1b3d6e9f012c3a4b5c6d7e8f9a0b1c2d3e4f5a6b7c8",
  "bundle_b_hash_at_tick": "9d4e1f2a3b4c5d6e7f8a9b0c1d2e3f4a5b6c7d8e9f0a1b2c3d4e5f6a7b8c9da",
  "identical_ticks": 846,
  "total_ticks_compared": 1000
}
```

### 2.6 Performance Gates (QG-07, QG-08)

#### 2.6.1 Tick Latency Targets

| Scenario Size | p50 Target | p99 Target | Hard Ceiling |
|---|---|---|---|
| Small (radius <= 50) | <= 5ms | <= 15ms | 30ms |
| Standard (radius <= 150) | <= 20ms | <= 60ms | 100ms |
| Large (radius <= 300) | <= 60ms | <= 90ms | 100ms |
| Maximum (radius <= 512) | <= 80ms | <= 95ms | 100ms |

The hard ceiling of 100ms is the tick loop budget. Exceeding it triggers a `civlab_sim_tick_budget_exceeded_total` counter increment. Three consecutive violations trigger freeze mode.

#### 2.6.2 Memory Ceiling

| Resource | Per-Simulation Limit | Server-Wide Limit |
|---|---|---|
| ECS World heap | 512 MB | — |
| WASM mod memory | 64 MB per mod | 256 MB total per simulation |
| Replay buffer | 128 MB | — |
| Hash chain buffer | 4 MB | — |

Memory usage is sampled every 10 ticks and exported as `civlab_sim_memory_bytes{component="ecs_world"}`.

```yaml
  bench-tick-latency:
    desc: "Run tick latency benchmarks and fail if thresholds are missed (QG-07)"
    cmds:
      - cargo bench -p civlab-sim --bench tick_latency -- --output-format bencher | tee /tmp/bench-output.txt
      - cargo run -p civlab-cli -- check-bench-thresholds --input /tmp/bench-output.txt --thresholds bench-thresholds.toml
```

`bench-thresholds.toml`:

```toml
[small]
p50_ms = 5
p99_ms = 15
ceiling_ms = 30

[standard]
p50_ms = 20
p99_ms = 60
ceiling_ms = 100

[large]
p50_ms = 60
p99_ms = 90
ceiling_ms = 100

[maximum]
p50_ms = 80
p99_ms = 95
ceiling_ms = 100
```

---

## 3. CI/CD Pipeline

### 3.1 Pipeline Overview

```
On Pull Request (required to merge):
  fmt-check -> clippy -> test-unit -> test-integration
      -> schema-validate -> det-double-run -> replay-check
      -> bench-gate -> dep-audit -> artifact-sign-verify

Nightly (required for nightly green badge):
  full-matrix-build -> det-seed-sweep -> cross-platform-hash
      -> dep-audit -> cargo-deny -> security-scan

On Release Tag (required to publish):
  all-PR-checks -> full-matrix-build -> artifact-sign
      -> sbom-generate -> release-publish
```

### 3.2 GitHub Actions Workflow (PR)

```yaml
# .github/workflows/ci.yml

name: CI

on:
  pull_request:
    branches: [main, develop]
  push:
    branches: [main]

env:
  RUST_BACKTRACE: 1
  CARGO_TERM_COLOR: always
  RUSTFLAGS: "-D warnings"

jobs:
  fmt-check:
    name: Rustfmt
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
        with:
          components: rustfmt
      - run: cargo fmt --all -- --check

  clippy:
    name: Clippy
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
        with:
          components: clippy
      - uses: Swatinem/rust-cache@v2
      - run: cargo clippy --all-targets --all-features -- -D warnings

  test-unit:
    name: Unit Tests
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
      - uses: Swatinem/rust-cache@v2
      - run: cargo test -p civlab-sim --lib -- --nocapture
      - run: cargo test -p civlab-server --lib -- --nocapture

  test-integration:
    name: Integration Tests (D1-D7)
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
      - uses: Swatinem/rust-cache@v2
      - run: cargo test -p civlab-sim --tests -- --nocapture 2>&1 | tee integration-results.txt
      - name: Assert no D1-D7 test failures
        run: |
          if grep -q "FAILED" integration-results.txt; then
            echo "Integration test failures detected:"
            grep "FAILED" integration-results.txt
            exit 1
          fi

  schema-validate:
    name: Schema Validation (QG-01)
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
      - uses: Swatinem/rust-cache@v2
      - uses: actions/setup-node@v4
        with:
          node-version: '22'
      - run: npm ci
      - run: task validate-schemas
      - run: task validate-all

  det-double-run:
    name: Determinism Double-Run (QG-02)
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
      - uses: Swatinem/rust-cache@v2
      - run: task det-double-run

  replay-check:
    name: Replay Consistency (QG-06)
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
      - uses: Swatinem/rust-cache@v2
      - run: cargo test -p civlab-sim --test replay_consistency -- --nocapture

  bench-gate:
    name: Performance Gate (QG-07/08)
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
      - uses: Swatinem/rust-cache@v2
      - run: task bench-tick-latency

  dep-audit:
    name: Dependency Audit (QG-10)
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
      - run: cargo install cargo-audit --locked
      - run: cargo audit
      - run: npm audit --audit-level=high
        working-directory: clients/web

  artifact-sign-verify:
    name: Artifact Signing Self-Test
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
      - uses: Swatinem/rust-cache@v2
      - run: cargo test -p civlab-server --test artifact_signing -- --nocapture
```

### 3.3 Nightly Regression Suite

```yaml
# .github/workflows/nightly.yml

name: Nightly Regression

on:
  schedule:
    - cron: '0 2 * * *'   # 02:00 UTC daily
  workflow_dispatch:

jobs:
  test-linux:
    name: Full Suite Linux x86_64
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
      - uses: Swatinem/rust-cache@v2
      - run: cargo test --all -- --nocapture
      - run: task det-seed-sweep
      - name: Export hash chain output
        run: |
          cargo run -p civlab-cli -- export-hash-chain \
            --fixture tests/fixtures/scenarios/standard_100hex.toml \
            --ticks 1000 \
            --output hash-chain-output-linux/standard_100hex_1000ticks.hex
      - uses: actions/upload-artifact@v4
        with:
          name: hash-chain-output-linux
          path: hash-chain-output-linux/

  test-macos:
    name: Full Suite macOS arm64
    runs-on: macos-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
      - uses: Swatinem/rust-cache@v2
      - run: cargo test --all -- --nocapture
      - name: Export hash chain output
        run: |
          cargo run -p civlab-cli -- export-hash-chain \
            --fixture tests/fixtures/scenarios/standard_100hex.toml \
            --ticks 1000 \
            --output hash-chain-output-macos/standard_100hex_1000ticks.hex
      - uses: actions/upload-artifact@v4
        with:
          name: hash-chain-output-macos
          path: hash-chain-output-macos/

  test-windows:
    name: Full Suite Windows x86_64
    runs-on: windows-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
      - uses: Swatinem/rust-cache@v2
      - run: cargo test --all -- --nocapture
      - name: Export hash chain output
        shell: bash
        run: |
          cargo run -p civlab-cli -- export-hash-chain \
            --fixture tests/fixtures/scenarios/standard_100hex.toml \
            --ticks 1000 \
            --output hash-chain-output-windows/standard_100hex_1000ticks.hex
      - uses: actions/upload-artifact@v4
        with:
          name: hash-chain-output-windows
          path: hash-chain-output-windows/

  cross-platform-hash-compare:
    name: Cross-Platform Hash Parity (QG-04)
    needs: [test-linux, test-macos, test-windows]
    runs-on: ubuntu-latest
    steps:
      - uses: actions/download-artifact@v4
        with:
          pattern: hash-chain-output-*
          merge-multiple: true
      - name: Compare outputs
        run: |
          LINUX=$(cat hash-chain-output-linux/standard_100hex_1000ticks.hex)
          MACOS=$(cat hash-chain-output-macos/standard_100hex_1000ticks.hex)
          WINDOWS=$(cat hash-chain-output-windows/standard_100hex_1000ticks.hex)
          echo "Linux:   $LINUX"
          echo "macOS:   $MACOS"
          echo "Windows: $WINDOWS"
          if [ "$LINUX" != "$MACOS" ] || [ "$LINUX" != "$WINDOWS" ]; then
            echo "FATAL: Cross-platform determinism failure"
            exit 1
          fi
          echo "PASS: All platforms produce identical hash chain"

  cargo-deny:
    name: License and Advisory Check
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: EmbarkStudios/cargo-deny-action@v1
        with:
          command: check all

  security-scan:
    name: SAST Security Scan
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - name: Run semgrep
        uses: semgrep/semgrep-action@v1
        with:
          config: p/rust p/secrets p/owasp-top-ten
```

### 3.4 Required Checks Before Merge

The following GitHub branch protection rules are enforced on `main` and `develop`:

| Check | Required | Dismiss Stale Reviews |
|---|---|---|
| fmt-check | yes | yes |
| clippy | yes | yes |
| test-unit | yes | yes |
| test-integration | yes | yes |
| schema-validate | yes | yes |
| det-double-run | yes | yes |
| replay-check | yes | yes |
| bench-gate | yes | yes |
| dep-audit | yes | yes |
| artifact-sign-verify | yes | yes |
| PR review (1 approver minimum) | yes | yes |

Direct pushes to `main` are forbidden. Force-push is disabled on `main` and `develop`. All changes via pull request.

### 3.5 Artifact Signing Pipeline (Release)

```yaml
# .github/workflows/release.yml (excerpt: signing)

  sign-artifacts:
    name: Sign Release Artifacts
    needs: [full-matrix-build]
    runs-on: ubuntu-latest
    permissions:
      id-token: write
      contents: write
    steps:
      - uses: actions/checkout@v4
      - uses: actions/download-artifact@v4
        with:
          pattern: build-*
          merge-multiple: true

      - name: Install cosign (keyless OIDC signing)
        uses: sigstore/cosign-installer@v3

      - name: Sign all artifacts
        run: |
          for artifact in dist/*; do
            cosign sign-blob \
              --yes \
              --bundle "${artifact}.cosign.bundle" \
              "$artifact"
            echo "Signed: $artifact"
          done

      - name: Generate SBOM with syft
        run: |
          curl -sSfL https://raw.githubusercontent.com/anchore/syft/main/install.sh \
            | sh -s -- -b /usr/local/bin
          syft packages . -o spdx-json > dist/civlab-sim.sbom.spdx.json

      - uses: actions/upload-artifact@v4
        with:
          name: signed-release-artifacts
          path: dist/
```

---

## 4. Spec-First Governance

### 4.1 Spec-First Development Process

All production code changes follow this mandatory sequence. Skipping any step is a governance violation caught by pre-commit hooks:

```
Step 1: SPEC UPDATE
  Update or create spec document (PRD, FR, ADR as applicable).
  Spec must be merged to main before code begins.

Step 2: FUNCTIONAL REQUIREMENT
  FR SHALL statement added to FUNCTIONAL_REQUIREMENTS.md.
  FR ID assigned: FR-{CAT}-{NNN}
  Acceptance criteria written: testable, unambiguous.

Step 3: TEST FIRST
  Test file created with // @trace FR-{CAT}-{NNN} annotation.
  Test is failing (red) before any impl code is written.
  Test name matches FR acceptance criterion.

Step 4: IMPLEMENTATION
  Code written to make test pass.
  Code references FR ID in doc comment.
  No code beyond what is required to pass the test.

Step 5: REVIEW
  PR created with spec link and FR ID in description.
  Reviewer checks spec-to-code traceability.
  All CI checks pass.

Step 6: MERGE
  Squash merge to main (preserves atomic FR-to-code history).
  Spec tracker updated (FR status -> IMPLEMENTED).
```

### 4.2 ADR Requirement Triggers

An Architecture Decision Record is required (blocking merge) for any change meeting one or more of the following criteria:

| Trigger | Example | ADR Template |
|---|---|---|
| New external crate dependency | Adding `serde_arrow` to Cargo.toml | `docs/adr/templates/new-dependency.md` |
| Change to determinism rules D1-D7 | Relaxing D3 for a specific subsystem | `docs/adr/templates/determinism-change.md` |
| Schema breaking change | Renaming a field in `scenario.schema.json` | `docs/adr/templates/schema-change.md` |
| Storage engine change | Migrating from SQLite to DuckDB for analytics | `docs/adr/templates/storage-change.md` |
| New client platform | Adding iOS native client | `docs/adr/templates/new-client.md` |
| RPC protocol change | Adding binary framing to JSON-RPC | `docs/adr/templates/protocol-change.md` |
| Tick loop architecture change | Moving from Bevy ECS to a custom scheduler | `docs/adr/templates/arch-change.md` |
| Fixed-point type change | Switching KiloJoules from i64 to i128 | `docs/adr/templates/type-change.md` |
| Mod sandbox policy change | Increasing WASM memory limit from 64 MB | `docs/adr/templates/mod-policy-change.md` |
| New cryptographic primitive | Replacing BLAKE3 with SHA3-512 | `docs/adr/templates/crypto-change.md` |

ADR format:

```markdown
# ADR-{NNN}: {Title}

**Status:** Proposed | Accepted | Deprecated | Superseded by ADR-{NNN}
**Date:** YYYY-MM-DD
**Owner:** {Name}
**Deciders:** {Names}

## Context
What is the situation? What problem are we solving?

## Decision
What is the decision? State it concisely.

## Consequences

### Positive

### Negative

### Neutral

## Alternatives Considered

| Alternative | Reason Rejected |
|---|---|

## Implementation Notes
Key implementation constraints or gotchas.

## Traceability
- FR IDs: FR-{CAT}-{NNN}
- Related ADRs: ADR-{NNN}
```

### 4.3 Spec Versioning

#### 4.3.1 Frontmatter Fields

All spec documents include standard frontmatter:

```yaml
---
title: "Civ-Sim Ops/Governance Specification"
version: "1.0.0"           # semver: MAJOR.MINOR.PATCH
status: "active"            # draft | active | deprecated | superseded
owner: "CivLab Platform Team"
last_updated: "2026-02-21"
breaking_change: false      # true if this version breaks downstream consumers
supersedes: null            # version this replaces, if any
review_date: "2026-08-21"   # scheduled review date (6-month default)
---
```

#### 4.3.2 Semver for Breaking Changes

| Change Type | Version Bump | ADR Required | Migration Guide Required |
|---|---|---|---|
| Additive (new section, new optional field) | MINOR | no | no |
| Clarification (no behavior change) | PATCH | no | no |
| Breaking (renamed field, removed section, changed semantics) | MAJOR | yes | yes |
| Deprecation (marks for removal in next MAJOR) | MINOR | no | yes |

#### 4.3.3 Change Review Checklist

PR template for spec changes enforces the following reviewer checklist:

- [ ] Version bump matches the nature of the change
- [ ] All frontmatter fields present and valid
- [ ] ADR linked if required by Section 4.2
- [ ] Downstream impact documented in PR description
- [ ] FR tracker updated if this change closes or modifies an FR
- [ ] Migration guide written if MAJOR bump

---

## 5. Versioned Policy and Metric Definitions

### 5.1 Simulation Policy Schema

Scenario configs are versioned TOML files. The canonical schema is `schemas/scenario/v1/scenario.schema.json`. The TOML structure:

```toml
# tests/fixtures/scenarios/standard_100hex.toml

[meta]
schema_version = "1.0.0"
scenario_id = "standard-100hex-v1"
name = "Standard 100-Hex Scenario"
description = "Baseline governance fixture for CI determinism tests"
created_at = "2026-02-21"
author = "civlab-platform-team"

[simulation]
rng_seed = 42
max_ticks = 10_000
hex_grid_radius = 100
tick_budget_ms = 100

[starting_state]
civilizations = 4
starting_energy_kj = 1_000_000       # 1 GJ in KiloJoules (i64)
starting_gdp_mc = 500_000_000         # 500 kCredits in MilliCredits (i64)

[policies]
# All weights must sum to exactly 1_000_000 ppm (fixed-point 1.0)

[[policies.allocations]]
id = "military"
weight_ppm = 200_000    # 20%

[[policies.allocations]]
id = "research"
weight_ppm = 300_000    # 30%

[[policies.allocations]]
id = "infrastructure"
weight_ppm = 300_000    # 30%

[[policies.allocations]]
id = "welfare"
weight_ppm = 200_000    # 20%

[terrain]
base_seed = 42
mountain_density_ppm = 150_000    # 15%
water_density_ppm = 250_000       # 25%
forest_density_ppm = 200_000      # 20%

[mods]
enabled = false

[output]
report_every_n_ticks = 100
include_hash_chain = true
sign_output = true
```

### 5.2 Metric Definition Format

Each simulation metric is defined in `metrics/definitions/` as a TOML file:

```toml
# metrics/definitions/energy_consumption_rate.toml

[metric]
id = "FR-MET-001"
name = "energy_consumption_rate"
display_name = "Energy Consumption Rate"
description = "Rate of energy consumption across all civilizations, in KiloJoules per tick"
version = "1.0.0"
status = "active"    # active | deprecated | experimental

[formula]
expression = "sum(civ.energy_consumed_kj) / tick_count"
unit = "KiloJoules/tick"
precision = "exact"    # exact (fixed-point) | approximate (float display only)

[source]
tick_field = "world.civilizations[*].energy_consumed_kj_this_tick"
aggregation = "sum"
window = "per_tick"    # per_tick | rolling_100 | cumulative

[thresholds]
warning_low  = 1_000          # Below this: possible stall
warning_high = 10_000_000     # Above this: possible runaway consumption
critical_high = 50_000_000    # Above this: freeze-mode candidate

[prometheus]
metric_name = "civlab_sim_energy_consumption_kj_per_tick"
type = "gauge"
labels = ["scenario_id", "civilization_id", "run_id"]
help = "Energy consumed this tick by a civilization, in KiloJoules"

[traceability]
fr_id = "FR-MET-001"
prd_epic = "E2.3"
```

### 5.3 Policy Bundle Versioning

#### 5.3.1 Bundle Version Bump Rules

| Change | Bundle Version Bump | Backward Compatible |
|---|---|---|
| Adding a new optional policy allocation | MINOR | yes |
| Changing a weight without renaming IDs | MINOR | yes |
| Renaming a policy ID | MAJOR | no |
| Removing a policy ID | MAJOR | no |
| Changing weight_ppm type or unit | MAJOR | no |

#### 5.3.2 Bundle Migration Procedure

When a MAJOR policy bundle version is released:

1. Tag the old bundle in `policies/archive/v{N}/`.
2. Write a migration script in `scripts/migrate_policy_bundle_vN_to_vN+1.py`.
3. Run migration against all saved scenarios in the test fixture set and verify schema validation passes.
4. Update `schemas/scenario/v1/policy.schema.json` and bump its version field.
5. Write ADR documenting the breaking change.
6. Write migration guide at `docs/migrations/policy-bundle-vN-to-vN+1.md`.

#### 5.3.3 Backward Compatibility Rules for Saved Scenarios

- The server MUST refuse to load a scenario config whose `schema_version` major version exceeds the server's supported schema major version. Return a structured error with expected and actual versions.
- The server MUST load any scenario config whose `schema_version` major version equals the server's version and minor version is less than or equal.
- The server MUST refuse to load any scenario config with `schema_version` major version lower than `(current_major - 1)`. One previous major version is supported for one release cycle only.
- No silent at-load-time migration. If out of range, return error. Client must run the migration tool explicitly before retry.

---

## 6. Runtime Governance and Guardrails

### 6.1 Intervention Authority

Runtime interventions are tiered by authority level:

| Tier | Actor | Interventions Allowed |
|---|---|---|
| T0 Emergency | Any server process (automated) | Freeze mode trigger, emergency stop |
| T1 Operator | Server admin via `civlab-cli` | Pause/resume, tick rate throttle, entity limit adjustment |
| T2 Game Master | Authenticated GM session | Scenario parameter hot-swap within policy bundle, event injection |
| T3 Observer | Authenticated client session | Read-only: subscribe to tick events, query world state |

All interventions are written to the audit log with: actor identity, intervention type, parameters, monotonic timestamp, wall-clock timestamp, and outcome. Audit log entries are immutable.

### 6.2 Runtime Guardrails

Hard limits enforced on every tick by the `SimulationGuardrails` component:

```rust
// crates/civlab-sim/src/guardrails.rs
// @trace FR-GUARD-001

#[derive(Debug, Clone)]
pub struct SimulationGuardrails {
    /// Maximum ticks before forced termination.
    pub max_tick_count: u64,
    /// Maximum ECS entities across all archetypes.
    pub max_entity_count: u64,
    /// Maximum ECS world heap usage in bytes.
    pub max_memory_bytes: usize,
    /// Tick wall-clock budget. 3 consecutive violations trigger freeze.
    pub tick_budget_ms: u64,
    /// Maximum active WASM mod instances.
    pub max_wasm_instances: u32,
    /// Maximum total WASM memory across all instances, in bytes.
    pub max_wasm_total_memory_bytes: usize,
    /// Consecutive limit violations before freeze mode activates.
    pub freeze_threshold: u32,
}

impl SimulationGuardrails {
    pub fn production() -> Self {
        Self {
            max_tick_count: 1_000_000,
            max_entity_count: 1_000_000,
            max_memory_bytes: 512 * 1024 * 1024,        // 512 MB
            tick_budget_ms: 100,
            max_wasm_instances: 16,
            max_wasm_total_memory_bytes: 256 * 1024 * 1024,  // 256 MB
            freeze_threshold: 3,
        }
    }

    pub fn ci_test() -> Self {
        Self {
            max_tick_count: 10_000,
            max_entity_count: 100_000,
            max_memory_bytes: 256 * 1024 * 1024,
            tick_budget_ms: 500,    // Relaxed for CI debug builds
            max_wasm_instances: 4,
            max_wasm_total_memory_bytes: 64 * 1024 * 1024,
            freeze_threshold: 5,
        }
    }
}
```

### 6.3 Freeze Mode

#### 6.3.1 Triggers

| Trigger | Condition | Severity |
|---|---|---|
| Tick budget exceeded | 3 consecutive ticks over `tick_budget_ms` | P1 |
| Determinism violation | BLAKE3 hash mismatch detected in online monitoring | P0 |
| Memory ceiling exceeded | ECS heap over `max_memory_bytes` for 2 consecutive samples | P1 |
| Entity count exceeded | Entity count over `max_entity_count` | P1 |
| WASM sandbox escape | wasmtime host-call policy violation | P0 |
| Tick count limit reached | `current_tick >= max_tick_count` | P2 expected termination |
| Manual operator trigger | `civlab-cli freeze <run-id>` | P1 |
| Critical metric threshold | Any metric with `critical_high` breach for 5 consecutive ticks | P1 |

#### 6.3.2 Freeze Mode Behavior

When freeze mode activates:

1. The tick loop halts after the current tick completes (no mid-tick halt).
2. Current world state is serialized to `snapshots/{run-id}/freeze-{tick}.civsnap`.
3. The snapshot is BLAKE3-hashed and Ed25519-signed.
4. A `FreezeEvent` notification is broadcast to all connected clients via JSON-RPC.
5. `civlab_sim_freeze_mode_active{run_id}` gauge is set to 1.
6. `civlab_sim_freeze_total{reason}` counter is incremented.
7. An audit log entry is written with: trigger reason, tick number, guardrail values, actor.
8. The server does NOT auto-restart the run. Recovery requires operator action.

```rust
// crates/civlab-server/src/freeze.rs
// @trace FR-GUARD-002

pub async fn activate_freeze_mode(
    run_id: RunId,
    trigger: FreezeTrigger,
    world_snapshot: WorldSnapshot,
    audit_log: &AuditLog,
    metrics: &SimMetrics,
    client_notifier: &ClientNotifier,
) -> Result<FreezeRecord, FreezeError> {
    let snap_bytes = world_snapshot.to_canonical_bytes();
    let snap_hash = blake3::hash(&snap_bytes);
    let snap_sig = sign_with_server_key(&snap_bytes);

    let snap_path = freeze_snapshot_path(run_id, world_snapshot.tick);
    tokio::fs::write(&snap_path, &snap_bytes).await?;

    audit_log.record(AuditEntry {
        event: AuditEvent::FreezeActivated,
        run_id,
        tick: world_snapshot.tick,
        actor: Actor::Automated(trigger.clone()),
        details: serde_json::json!({
            "trigger": trigger,
            "snapshot_hash": hex::encode(snap_hash.as_bytes()),
            "snapshot_path": snap_path.display().to_string(),
        }),
        timestamp: MonotonicTimestamp::now(),
    })?;

    metrics
        .freeze_mode_active
        .with_label_values(&[&run_id.to_string()])
        .set(1);
    metrics
        .freeze_total
        .with_label_values(&[trigger.label()])
        .inc();

    client_notifier
        .broadcast(FreezeNotification {
            run_id,
            tick: world_snapshot.tick,
            reason: trigger.user_facing_message(),
            snapshot_hash: hex::encode(snap_hash.as_bytes()),
        })
        .await;

    Ok(FreezeRecord {
        snap_path,
        snap_hash: snap_hash.into(),
        signature: snap_sig,
    })
}
```

#### 6.3.3 Freeze Recovery Procedure

```bash
# Step 1: Inspect freeze record
civlab-cli freeze-status --run-id <run-id>

# Step 2: Verify snapshot integrity
civlab-cli verify-snapshot --run-id <run-id> --tick <freeze-tick>

# Step 3: Diagnose divergence if needed
civlab-cli replay-diff \
  --bundle-a "snapshots/<run-id>/freeze-<tick>.civsnap" \
  --bundle-b "snapshots/<run-id>/pre-freeze-<tick-5>.civsnap"

# Step 4a: If safe to resume (e.g., transient tick budget spike)
civlab-cli resume \
  --run-id <run-id> \
  --justification "Transient tick budget spike; entity batch resolved"

# Step 4b: If not safe to resume (determinism violation or WASM escape)
civlab-cli abort \
  --run-id <run-id> \
  --reason "Determinism violation: BLAKE3 mismatch at tick <N>"

# Step 5: File incident (mandatory for P0 and P1 triggers)
civlab-cli incident create \
  --run-id <run-id> \
  --severity P1 \
  --trigger "tick_budget_exceeded" \
  --summary "Spike in entity creation caused 3 consecutive tick budget violations"
```

Resuming from freeze after a P0 determinism violation requires CTO sign-off in production. CI and staging environments may be resumed by the Sim Team Lead.

### 6.4 Emergency Stop

Emergency stop halts all active simulation runs immediately. Used when the server process itself must stop safely (infrastructure incident, security event).

```bash
# Emergency stop all runs
civlab-cli emergency-stop --reason "Infrastructure incident: database unreachable"

# Emergency stop a specific run
civlab-cli emergency-stop --run-id <run-id> --reason "P0: WASM sandbox escape detected"
```

Emergency stop behavior:

1. All tick loops receive cancellation signal; halt after current tick.
2. All in-progress world states serialized to emergency snapshots and signed.
3. All clients receive `EmergencyStopNotification` via JSON-RPC.
4. Server enters read-only mode; no new runs can start until operator clears the state.
5. All events recorded in the audit log.
6. `civlab_sim_emergency_stop_total` counter incremented.

To clear emergency state:

```bash
civlab-cli emergency-clear \
  --reason "Infrastructure incident resolved: database connection restored" \
  --operator-id <operator-id>
```

---

## 7. Monitoring and Observability

### 7.1 Prometheus Metrics Schema

All metrics are exported at `http://civlab-server:9090/metrics` in Prometheus text format.

#### 7.1.1 Simulation Run Metrics

| Metric Name | Type | Labels | Help |
|---|---|---|---|
| `civlab_sim_runs_started_total` | Counter | `scenario_id`, `scenario_version` | Total runs started since server start |
| `civlab_sim_runs_completed_total` | Counter | `scenario_id`, `scenario_version`, `result` | Total runs completed; result=success,aborted,frozen |
| `civlab_sim_runs_active` | Gauge | `scenario_id` | Currently active simulation runs |
| `civlab_sim_run_duration_seconds` | Histogram | `scenario_id`, `scenario_version` | Wall-clock duration of completed runs |
| `civlab_sim_tick_total` | Counter | `run_id`, `scenario_id` | Total ticks executed |
| `civlab_sim_tick_duration_seconds` | Histogram | `run_id`, `scenario_id` | Per-tick execution time; buckets: 0.001,0.005,0.01,0.02,0.05,0.1,0.2,0.5 |
| `civlab_sim_tick_budget_exceeded_total` | Counter | `run_id`, `scenario_id` | Ticks exceeding the 100ms budget |

#### 7.1.2 Determinism Metrics

| Metric Name | Type | Labels | Help |
|---|---|---|---|
| `civlab_sim_determinism_check_total` | Counter | `run_id`, `result` | Determinism checks; result=pass,fail |
| `civlab_sim_determinism_violations_total` | Counter | `run_id`, `scenario_id`, `d_rule` | Violations by D-rule; d_rule=D1..D7 |
| `civlab_sim_hash_chain_mismatches_total` | Counter | `run_id` | BLAKE3 hash chain mismatches in online monitoring |
| `civlab_sim_replay_consistency_failures_total` | Counter | `run_id`, `scenario_id` | Replay consistency test failures |

#### 7.1.3 Freeze Mode Metrics

| Metric Name | Type | Labels | Help |
|---|---|---|---|
| `civlab_sim_freeze_mode_active` | Gauge | `run_id` | 1 if run is in freeze mode, 0 otherwise |
| `civlab_sim_freeze_total` | Counter | `run_id`, `reason` | Freeze activations by trigger reason |
| `civlab_sim_emergency_stop_total` | Counter | — | Emergency stops since server start |

#### 7.1.4 Resource Metrics

| Metric Name | Type | Labels | Help |
|---|---|---|---|
| `civlab_sim_memory_bytes` | Gauge | `run_id`, `component` | Memory in bytes; component=ecs_world,wasm_total,replay_buffer,hash_chain |
| `civlab_sim_entity_count` | Gauge | `run_id`, `archetype` | ECS entity count by archetype |
| `civlab_sim_wasm_instances_active` | Gauge | `run_id` | Active WASM mod instances |
| `civlab_sim_wasm_memory_bytes` | Gauge | `run_id`, `mod_id` | WASM memory per mod instance |

#### 7.1.5 Game-Layer Metrics

| Metric Name | Type | Labels | Help |
|---|---|---|---|
| `civlab_sim_energy_consumption_kj_per_tick` | Gauge | `run_id`, `civilization_id` | Energy consumed this tick per civilization |
| `civlab_sim_gdp_mc_total` | Gauge | `run_id`, `civilization_id` | GDP of a civilization in MilliCredits |
| `civlab_sim_population_total` | Gauge | `run_id`, `civilization_id` | Population per civilization |
| `civlab_sim_territory_hexes` | Gauge | `run_id`, `civilization_id` | Hex tiles controlled |
| `civlab_sim_policy_weight_ppm` | Gauge | `run_id`, `civilization_id`, `policy_id` | Current policy allocation weight in ppm |

#### 7.1.6 Storage and I/O Metrics

| Metric Name | Type | Labels | Help |
|---|---|---|---|
| `civlab_storage_sqlite_query_duration_seconds` | Histogram | `operation`, `table` | SQLite query duration |
| `civlab_storage_postgres_query_duration_seconds` | Histogram | `operation`, `table` | PostgreSQL query duration |
| `civlab_storage_sqlite_wal_size_bytes` | Gauge | `db_path` | SQLite WAL file size |
| `civlab_storage_backup_last_success_timestamp` | Gauge | `backend` | Unix timestamp of last successful backup |
| `civlab_storage_artifact_sign_duration_seconds` | Histogram | — | Ed25519 artifact signing duration |

#### 7.1.7 Server and RPC Metrics

| Metric Name | Type | Labels | Help |
|---|---|---|---|
| `civlab_rpc_requests_total` | Counter | `method`, `status` | JSON-RPC requests; status=ok,error |
| `civlab_rpc_request_duration_seconds` | Histogram | `method` | JSON-RPC request handling duration |
| `civlab_rpc_active_connections` | Gauge | `client_type` | Active WebSocket connections; client_type=web,mobile,desktop |
| `civlab_rpc_messages_sent_total` | Counter | `notification_type` | Server-push notifications sent |

### 7.2 Grafana Dashboard Layout

The canonical dashboard is exported to `monitoring/grafana/dashboards/civlab-sim.json` and provisioned automatically. Dashboard UID: `civlab-sim-ops`.

| Row | Panel Name | Columns | Visualization | Key Query |
|---|---|---|---|---|
| 1 | Active Runs | 4 | Stat | `civlab_sim_runs_active` |
| 1 | Runs Completed (1h) | 4 | Stat | `increase(civlab_sim_runs_completed_total[1h])` |
| 1 | Freeze Mode Active | 4 | Stat (red if > 0) | `sum(civlab_sim_freeze_mode_active)` |
| 1 | Determinism Violations (24h) | 4 | Stat (red if > 0) | `increase(civlab_sim_determinism_violations_total[24h])` |
| 1 | Tick Duration p99 | 4 | Stat | `histogram_quantile(0.99, civlab_sim_tick_duration_seconds)` |
| 1 | Active WebSocket Clients | 4 | Stat | `sum(civlab_rpc_active_connections)` |
| 2 | Tick Duration Distribution | 12 | Heatmap | `civlab_sim_tick_duration_seconds_bucket` |
| 2 | Memory by Component | 12 | Time series | `civlab_sim_memory_bytes` by component |
| 3 | Runs by Result | 8 | Bar chart | `civlab_sim_runs_completed_total` by result |
| 3 | Determinism Check Pass Rate | 8 | Time series % | `rate(civlab_sim_determinism_check_total{result="pass"})` |
| 3 | RPC Request Rate | 8 | Time series | `rate(civlab_rpc_requests_total[5m])` by method |
| 4 | Energy Consumption by Civ | 12 | Time series | `civlab_sim_energy_consumption_kj_per_tick` |
| 4 | GDP by Civilization | 12 | Time series | `civlab_sim_gdp_mc_total` |
| 5 | SQLite Query Latency p99 | 12 | Time series | `histogram_quantile(0.99, civlab_storage_sqlite_query_duration_seconds)` |
| 5 | WAL Size Trend | 12 | Time series | `civlab_storage_sqlite_wal_size_bytes` |

### 7.3 Alerting Rules

```yaml
# monitoring/prometheus/alerts/civlab-sim.yml

groups:
  - name: civlab_sim_determinism
    rules:
      - alert: DeterminismViolationDetected
        expr: increase(civlab_sim_determinism_violations_total[5m]) > 0
        for: 0m
        labels:
          severity: critical
          team: sim
        annotations:
          summary: "Determinism violation in run {{ $labels.run_id }}"
          description: "D-rule {{ $labels.d_rule }} violated. Immediate investigation required."
          runbook_url: "https://docs.civlab.internal/runbooks/determinism-violation"

      - alert: HashChainMismatch
        expr: increase(civlab_sim_hash_chain_mismatches_total[5m]) > 0
        for: 0m
        labels:
          severity: critical
          team: sim
        annotations:
          summary: "BLAKE3 hash chain mismatch in run {{ $labels.run_id }}"
          description: "Online hash chain monitoring detected a mismatch. Freeze mode should have activated."
          runbook_url: "https://docs.civlab.internal/runbooks/hash-chain-mismatch"

  - name: civlab_sim_performance
    rules:
      - alert: TickBudgetExceededFrequent
        expr: rate(civlab_sim_tick_budget_exceeded_total[5m]) > 0.1
        for: 2m
        labels:
          severity: warning
          team: sim
        annotations:
          summary: "Tick budget frequently exceeded in run {{ $labels.run_id }}"
          description: "Over 10% of ticks are exceeding the 100ms budget. Freeze mode may activate."
          runbook_url: "https://docs.civlab.internal/runbooks/tick-budget-exceeded"

      - alert: TickP99HighLatency
        expr: histogram_quantile(0.99, rate(civlab_sim_tick_duration_seconds_bucket[5m])) > 0.09
        for: 5m
        labels:
          severity: warning
          team: sim
        annotations:
          summary: "Tick p99 latency above 90ms"
          description: "p99 tick duration is {{ $value | humanizeDuration }}. Budget is 100ms."
          runbook_url: "https://docs.civlab.internal/runbooks/high-tick-latency"

      - alert: SimMemoryCeilingApproaching
        expr: civlab_sim_memory_bytes{component="ecs_world"} > (450 * 1024 * 1024)
        for: 2m
        labels:
          severity: warning
          team: sim
        annotations:
          summary: "ECS world memory approaching ceiling in run {{ $labels.run_id }}"
          description: "ECS world using {{ $value | humanizeBytes }} of 512 MB limit."
          runbook_url: "https://docs.civlab.internal/runbooks/memory-ceiling"

  - name: civlab_sim_freeze
    rules:
      - alert: FreezeModeActive
        expr: sum(civlab_sim_freeze_mode_active) > 0
        for: 0m
        labels:
          severity: critical
          team: sim
        annotations:
          summary: "Simulation freeze mode is active"
          description: "{{ $value }} run(s) in freeze mode. Operator action required."
          runbook_url: "https://docs.civlab.internal/runbooks/freeze-mode"

      - alert: EmergencyStopOccurred
        expr: increase(civlab_sim_emergency_stop_total[5m]) > 0
        for: 0m
        labels:
          severity: critical
          team: platform
        annotations:
          summary: "Emergency stop triggered on civlab-server"
          description: "Server has entered emergency stop state. All runs halted."
          runbook_url: "https://docs.civlab.internal/runbooks/emergency-stop"

  - name: civlab_storage
    rules:
      - alert: BackupStaleness
        expr: time() - civlab_storage_backup_last_success_timestamp{backend="sqlite"} > 7200
        for: 10m
        labels:
          severity: warning
          team: infra
        annotations:
          summary: "SQLite backup has not succeeded in over 2 hours"
          description: "Last successful backup was {{ $value | humanizeDuration }} ago."
          runbook_url: "https://docs.civlab.internal/runbooks/backup-staleness"

      - alert: WALSizeLarge
        expr: civlab_storage_sqlite_wal_size_bytes > (512 * 1024 * 1024)
        for: 5m
        labels:
          severity: warning
          team: infra
        annotations:
          summary: "SQLite WAL file exceeds 512 MB"
          description: "WAL size is {{ $value | humanizeBytes }}. Consider checkpoint."
          runbook_url: "https://docs.civlab.internal/runbooks/wal-size"
```

### 7.4 Structured Logging

All log output uses `tracing` crate with `tracing-subscriber` JSON format. No `println!` or `eprintln!` in production code.

#### 7.4.1 Required Log Fields

| Field | Type | Description |
|---|---|---|
| `timestamp` | RFC3339 | Log time (not simulation time) |
| `level` | string | ERROR, WARN, INFO, DEBUG, or TRACE |
| `target` | string | Rust module path (e.g., `civlab_sim::tick_loop`) |
| `span.run_id` | string | Run ID if inside a simulation span |
| `span.tick` | u64 | Current tick if inside a tick span |
| `message` | string | Human-readable message |

#### 7.4.2 Log Levels and Sampling

| Level | Use | Sampling in Production |
|---|---|---|
| ERROR | Unrecoverable errors, freeze triggers, determinism violations | 100% always |
| WARN | Recoverable issues, threshold warnings | 100% |
| INFO | Run start/stop, freeze events, config changes, operator actions | 100% |
| DEBUG | Per-tick summaries every 100 ticks, RPC connections | 10% sampled |
| TRACE | Per-entity state changes, per-system timing | Off in production |

TRACE logs are never enabled in production. Enabling TRACE requires a feature flag change and a server restart, and must be approved by the Sim Team Lead.

#### 7.4.3 Log Retention

| Level | Retention | Storage Tier |
|---|---|---|
| ERROR | 2 years | Hot: 90 days; cold archive: remainder |
| WARN | 1 year | Hot: 30 days; cold archive: remainder |
| INFO | 90 days | Hot storage |
| DEBUG | 7 days | Hot storage rolling |
| TRACE | Not persisted | stdout only; never written to disk in production |

---

## 8. Artifact Integrity

### 8.1 Ed25519 Signing

All artifact outputs are signed with Ed25519 using a server-held signing key. The signing key is stored in an environment-specific secrets manager (HashiCorp Vault in production, environment variable in CI). Private keys are never written to disk.

#### 8.1.1 Signing Key Management

| Environment | Key Storage | Rotation Period | Rotation Procedure |
|---|---|---|---|
| Production | HashiCorp Vault (transit secrets engine) | 90 days | ADR-triggered rotation with 7-day overlap period |
| Staging | HashiCorp Vault | 180 days | Same as production |
| CI | GitHub Actions secret `CIVLAB_SIGNING_KEY_B64` | Per-release | Rotated with each major release |
| Development | Local file `~/.civlab/dev-signing-key.pem` (gitignored) | Developer discretion | N/A |

The public key corresponding to the production signing key is published in `public-keys/production.ed25519.pub` at the repository root. This file is included in the signed SBOM.

#### 8.1.2 What Gets Signed

| Artifact | Signed | Signature Format |
|---|---|---|
| Simulation reports (`*.civreport`) | yes | Detached `.sig` file, Ed25519 |
| Replay bundles (`*.civreplay`) | yes | Detached `.sig` file, Ed25519 |
| Freeze snapshots (`*.civsnap`) | yes | Embedded signature field |
| Audit log chunks | yes | Merkle tree root, Ed25519 |
| Release binaries | yes | cosign bundle (keyless OIDC) |
| SBOM | yes | cosign bundle (keyless OIDC) |
| Scenario config hashes | yes | Embedded in scenario metadata |

#### 8.1.3 Signing API

```rust
// crates/civlab-server/src/signing.rs
// @trace FR-INT-001

use ed25519_dalek::{Signature, Signer, SigningKey, VerifyingKey};

pub struct ArtifactSigner {
    signing_key: SigningKey,
}

impl ArtifactSigner {
    /// Sign arbitrary bytes. Returns a 64-byte Ed25519 signature.
    pub fn sign(&self, data: &[u8]) -> Signature {
        self.signing_key.sign(data)
    }

    /// Sign a file at `path` and write the detached signature to `path + ".sig"`.
    pub async fn sign_file(&self, path: &std::path::Path) -> Result<(), SigningError> {
        let data = tokio::fs::read(path).await?;
        let sig = self.sign(&data);
        let sig_path = path.with_extension("sig");
        tokio::fs::write(&sig_path, sig.to_bytes()).await?;
        tracing::info!(
            artifact = %path.display(),
            signature = %sig_path.display(),
            "artifact signed"
        );
        Ok(())
    }
}

pub struct ArtifactVerifier {
    verifying_key: VerifyingKey,
}

impl ArtifactVerifier {
    /// Verify a detached Ed25519 signature.
    pub fn verify(&self, data: &[u8], sig_bytes: &[u8; 64]) -> Result<(), VerificationError> {
        let sig = Signature::from_bytes(sig_bytes);
        self.verifying_key
            .verify_strict(data, &sig)
            .map_err(|_| VerificationError::InvalidSignature)
    }
}
```

#### 8.1.4 Verification CLI

```bash
# Verify a simulation report
civlab-cli verify-artifact \
  --artifact reports/run-abc123.civreport \
  --pubkey public-keys/production.ed25519.pub

# Verify a replay bundle
civlab-cli verify-artifact \
  --artifact replays/run-abc123.civreplay \
  --pubkey public-keys/production.ed25519.pub

# Batch verify all artifacts in a directory
civlab-cli verify-artifacts \
  --dir reports/ \
  --pubkey public-keys/production.ed25519.pub \
  --fail-fast
```

### 8.2 Report Package Format

A simulation report package (`*.civreport`) is a directory-format ZIP with the following structure:

```
run-abc123.civreport/
  manifest.json          # Report metadata and integrity hashes
  summary.json           # High-level run summary (tick count, outcome, final metrics)
  metrics/
    tick-metrics.csv     # Per-tick metrics (tick, energy, gdp, population per civ)
    aggregate.json       # Aggregated metric statistics (min, max, mean, p99 per metric)
  hash-chain/
    hashes.bin           # All per-tick BLAKE3 hashes (binary, 32 bytes * tick_count)
    root.hex             # Final hash chain root as hex string
  policies/
    initial-policy.toml  # Policy bundle at scenario start
    policy-changes.jsonl # Log of all policy changes during the run (one JSON per line)
  audit/
    interventions.jsonl  # All T1/T2 interventions during the run
  signature/
    manifest.sig         # Ed25519 signature over manifest.json
    report.sig           # Ed25519 signature over the entire report (all files, sorted)
```

`manifest.json` structure:

```json
{
  "format_version": "1.0.0",
  "run_id": "abc123",
  "scenario_id": "standard-100hex-v1",
  "scenario_schema_version": "1.0.0",
  "started_at": "2026-02-21T14:00:00Z",
  "completed_at": "2026-02-21T14:05:32Z",
  "tick_count": 10000,
  "outcome": "success",
  "rng_seed": 42,
  "hash_chain_root": "a3f7c2b1e9d045f8...",
  "signing_key_fingerprint": "SHA256:abc123...",
  "file_hashes": {
    "summary.json": "blake3:9d4e1f2a...",
    "metrics/tick-metrics.csv": "blake3:7c3b2a1d...",
    "metrics/aggregate.json": "blake3:5e6f7a8b...",
    "hash-chain/hashes.bin": "blake3:1a2b3c4d...",
    "hash-chain/root.hex": "blake3:2c3d4e5f...",
    "policies/initial-policy.toml": "blake3:3d4e5f6a...",
    "policies/policy-changes.jsonl": "blake3:4e5f6a7b...",
    "audit/interventions.jsonl": "blake3:5f6a7b8c..."
  }
}
```

### 8.3 Replay Bundle Signing

Replay bundles (`*.civreplay`) include:

```
run-abc123.civreplay/
  seed.bin              # 8-byte little-endian u64 seed
  tick-0-state.bin      # Canonical serialization of ECS world at tick 0
  scenario.toml         # The exact scenario config used
  hash-chain/
    root.hex            # Hash chain root after last tick
    tick-count.txt      # Number of ticks in the original run
  signature/
    bundle.sig          # Ed25519 signature over (seed.bin + tick-0-state.bin + root.hex)
    seed-proof.json     # JSON blob: {seed, scenario_id, scenario_hash, server_version}
```

The `seed-proof.json` is the tamper-detection anchor. Any replay that produces a different `hash-chain/root.hex` than the original run's root is invalid.

### 8.4 Audit Log

The audit log is an append-only JSONL file, one JSON object per line. Log chunks are signed with Ed25519 every 1000 entries. The audit log captures every event that changes simulation state or server configuration.

#### 8.4.1 Audit Event Types

| Event Type | Trigger | Mandatory Fields |
|---|---|---|
| `RunStarted` | Scenario starts | run_id, scenario_id, rng_seed, actor |
| `RunCompleted` | Scenario ends normally | run_id, tick_count, outcome, hash_chain_root |
| `RunAborted` | Operator abort | run_id, reason, tick_at_abort, actor |
| `FreezeActivated` | Freeze mode trigger | run_id, tick, trigger, snapshot_hash, actor |
| `FreezeResumed` | Operator resume after freeze | run_id, tick, justification, actor |
| `EmergencyStop` | Emergency stop | reason, active_run_ids, actor |
| `EmergencyCleared` | Emergency state cleared | reason, operator_id |
| `PolicyChanged` | T2 policy hot-swap | run_id, tick, old_policy_hash, new_policy_hash, actor |
| `EventInjected` | T2 event injection | run_id, tick, event_type, event_payload_hash, actor |
| `ConfigChanged` | Server config change | config_key, old_value_hash, new_value_hash, actor |
| `SigningKeyRotated` | Signing key rotation | new_key_fingerprint, old_key_fingerprint, actor |
| `AuditChunkSigned` | Chunk signature | chunk_start_entry, chunk_end_entry, chunk_hash, signature |

#### 8.4.2 Audit Log Entry Structure

```json
{
  "entry_id": 12345,
  "timestamp_wall": "2026-02-21T14:03:47.123456789Z",
  "timestamp_monotonic_ns": 987654321000,
  "event": "FreezeActivated",
  "run_id": "abc123",
  "tick": 847,
  "actor": {
    "type": "automated",
    "trigger": "tick_budget_exceeded",
    "consecutive_violations": 3
  },
  "details": {
    "trigger": "tick_budget_exceeded",
    "snapshot_hash": "a3f7c2b1e9d045f88c2a1b3d6e9f012c",
    "snapshot_path": "snapshots/abc123/freeze-847.civsnap"
  },
  "prev_entry_hash": "blake3:9d4e1f2a3b4c5d6e..."
}
```

The `prev_entry_hash` field chains entries together; any tampering with a prior entry invalidates all subsequent entries.

---

## 9. Storage Governance

### 9.1 SQLite Policy

SQLite is used for embedded single-node deployments and local development. All SQLite databases are opened with the following pragma configuration, applied at connection time:

```sql
-- Applied via civlab-server's SQLite connection initializer
-- @trace FR-STOR-001

PRAGMA journal_mode = WAL;          -- Write-Ahead Logging for concurrent reads
PRAGMA synchronous = NORMAL;        -- Durable without fsync on every write
PRAGMA page_size = 4096;            -- 4 KB pages (set only at DB creation time)
PRAGMA cache_size = -65536;         -- 64 MB page cache (negative = KiB)
PRAGMA foreign_keys = ON;           -- Enforce FK constraints
PRAGMA auto_vacuum = INCREMENTAL;   -- Incremental vacuuming to reclaim space
PRAGMA wal_autocheckpoint = 1000;   -- Checkpoint every 1000 WAL pages
PRAGMA mmap_size = 536870912;       -- 512 MB mmap (OS-backed memory-mapped I/O)
PRAGMA temp_store = MEMORY;         -- Temp tables in RAM
PRAGMA busy_timeout = 5000;         -- 5-second busy timeout before SQLITE_BUSY
```

Any change to pragma configuration requires an ADR and is tested against the determinism test suite (pragmas must not affect simulation output).

#### 9.1.1 Schema Migration Policy

SQLite schema migrations are managed by `sqlx` migrate with versioned SQL files in `migrations/sqlite/`:

```
migrations/sqlite/
  0001_initial_schema.sql
  0002_add_audit_log.sql
  0003_add_replay_bundles.sql
  ...
```

Rules:
- Migration files are numbered sequentially and immutable once merged to main.
- No `DROP TABLE` or `DROP COLUMN` without a corresponding data migration and a 2-week deprecation window.
- All migrations run inside a transaction. If any statement fails, the transaction rolls back and the server refuses to start.
- `sqlx migrate run` is called on server startup; the server will not start with pending migrations.
- Migrations are tested in CI against a clean database before merging.

#### 9.1.2 WAL Management

WAL checkpointing is automatic (every 1000 pages). For manual checkpointing:

```bash
# Full checkpoint and WAL reset
civlab-cli db checkpoint --backend sqlite --mode full

# Passive checkpoint (does not block readers)
civlab-cli db checkpoint --backend sqlite --mode passive
```

If the WAL exceeds 512 MB (alert threshold), a `PRAGMA wal_checkpoint(TRUNCATE)` is automatically triggered by the server and logged as an INFO event.

### 9.2 PostgreSQL Policy

PostgreSQL is used for multi-node server deployments. The `civlab-server` connects to PostgreSQL via `sqlx` with a connection pool of min 5, max 25 connections.

#### 9.2.1 Schema Migration Policy

PostgreSQL migrations use the same `sqlx` migrate tooling:

```
migrations/postgres/
  0001_initial_schema.sql
  0002_add_audit_log.sql
  ...
```

Additional PostgreSQL-specific rules:
- All tables have a `created_at TIMESTAMPTZ DEFAULT NOW()` column.
- Partitioning is used for `audit_log` and `tick_metrics` tables, partitioned by month.
- New partitions must be created at least 7 days before the month boundary. A cron job (`scripts/create-next-partition.sh`) runs daily and creates the next month's partition if it does not exist.
- Index creation must use `CREATE INDEX CONCURRENTLY` to avoid table locks.

#### 9.2.2 Partition Management

```sql
-- Partition creation template for audit_log
CREATE TABLE audit_log_2026_03
PARTITION OF audit_log
FOR VALUES FROM ('2026-03-01') TO ('2026-04-01');

-- Create index on new partition concurrently
CREATE INDEX CONCURRENTLY idx_audit_log_2026_03_run_id
ON audit_log_2026_03 (run_id);
```

Old partitions (older than the retention period in Section 9.4) are detached and dropped:

```bash
civlab-cli db drop-partition \
  --table audit_log \
  --partition audit_log_2024_01 \
  --confirm
```

### 9.3 Backup Policy

| Backend | Backup Type | Frequency | Retention | Restore Test Frequency |
|---|---|---|---|---|
| SQLite | Online backup (`.backup` API) | Every 1 hour | 30 days rolling | Weekly automated restore test |
| PostgreSQL | `pg_dump` (custom format) | Every 1 hour | 30 days rolling | Weekly automated restore test |
| PostgreSQL | WAL archiving (continuous) | Continuous | 7 days WAL | Monthly point-in-time restore test |
| Audit log | Append-only export | Every 6 hours | 2 years (cold) | Quarterly integrity check |
| Replay bundles | Cold object storage sync | Daily | Scenario TTL + 90 days | Manual on demand |

#### 9.3.1 Backup Taskfile Targets

```yaml
  backup-sqlite:
    desc: "Take an online SQLite backup to the configured backup directory"
    cmds:
      - cargo run -p civlab-cli -- db backup --backend sqlite --output {{.BACKUP_DIR}}/sqlite-{{.NOW}}.db
    vars:
      NOW:
        sh: date -u +%Y%m%dT%H%M%SZ
    requires:
      vars: [BACKUP_DIR]

  backup-postgres:
    desc: "Take a PostgreSQL backup using pg_dump"
    cmds:
      - pg_dump --format=custom --compress=9 --file={{.BACKUP_DIR}}/postgres-{{.NOW}}.pgdump {{.CIVLAB_DB_URL}}
    vars:
      NOW:
        sh: date -u +%Y%m%dT%H%M%SZ
    requires:
      vars: [BACKUP_DIR, CIVLAB_DB_URL]

  restore-test-sqlite:
    desc: "Restore latest SQLite backup to a temp DB and run validation queries"
    cmds:
      - cargo run -p civlab-cli -- db restore-test --backend sqlite --backup {{.LATEST_BACKUP}} --temp-db /tmp/civlab-restore-test.db
    requires:
      vars: [LATEST_BACKUP]

  restore-test-postgres:
    desc: "Restore latest PostgreSQL backup to a temp DB and run validation queries"
    cmds:
      - cargo run -p civlab-cli -- db restore-test --backend postgres --backup {{.LATEST_BACKUP}} --temp-db civlab_restore_test
    requires:
      vars: [LATEST_BACKUP]
```

### 9.4 Data Lifecycle and Retention

| Data Type | Active State | Archive After | Delete After | Archive Location |
|---|---|---|---|---|
| Active scenario run data | In database | Run completion | 90 days from completion | Cold object storage |
| Completed scenario reports | In database | 30 days | 2 years | Cold object storage |
| Replay bundles | On disk | 7 days after scenario end | Scenario TTL + 90 days | Cold object storage |
| Freeze snapshots | On disk | Incident resolution | 90 days after incident closed | Cold object storage |
| Audit log entries | In database | 90 days | 2 years | Cold archive (append-only export) |
| Structured logs ERROR/WARN | Hot storage | See Section 7.4.3 | Per Section 7.4.3 | Cold archive |
| Mod WASM binaries | On disk | Mod disabled | Mod removed from all scenarios | N/A (delete only) |

Deletion is performed by the `civlab-cli db gc` command, which runs the lifecycle policy and produces a deletion report before committing any deletions. Deletions are logged to the audit trail.

```bash
# Dry-run data GC (shows what would be deleted)
civlab-cli db gc --dry-run --report /tmp/gc-report.json

# Execute GC after review
civlab-cli db gc --confirm --report /tmp/gc-report.json
```

---

## 10. Dependency Governance

### 10.1 Rust Dependency Policy

#### 10.1.1 Adding a New Crate

Adding a new crate to any `Cargo.toml` in the workspace requires:

1. **ADR**: An Architecture Decision Record per Section 4.2 (trigger: "New external crate dependency").
2. **License check**: The crate's license must be on the allowed list in Section 10.3.
3. **Security audit**: `cargo audit` must pass with the new crate included.
4. **`cargo-deny` check**: `cargo deny check all` must pass.
5. **Justification**: PR description must include: crate name, version, why no existing crate suffices, license, and last release date.

The ADR is linked in the PR. The PR will not be merged without the ADR merged first.

#### 10.1.2 Pinned Crate Versions

The following crates are pinned to exact versions because minor or patch updates have historically introduced breaking behavioral changes:

| Crate | Pinned Version | Reason | Review Date |
|---|---|---|---|
| `bevy_ecs` | `=0.18.x` | ECS scheduling API surface | Per Bevy release cycle |
| `hexx` | `=0.21.x` | Hex math API; patch versions have changed coordinate conventions | Per hexx release |
| `wasmtime` | `=26.x` | Host API changes affect WASM sandbox policy | Per wasmtime major release |
| `blake3` | `>=1.5, <2` | Hash output must be stable for replay compatibility | Annual review |
| `chacha20` (via `rand_chacha`) | `>=0.3, <0.4` | RNG output stability is a determinism contract | Annual review |

Bumping any pinned crate version requires an ADR and full determinism test suite pass.

#### 10.1.3 Yanked Crate Response SLA

When `cargo audit` reports a yanked or advisory-flagged crate:

| Advisory Type | Response SLA | Action |
|---|---|---|
| RUSTSEC with CVSS >= 9.0 (Critical) | 4 hours | Immediate patching; freeze deploy pipeline until resolved |
| RUSTSEC with CVSS 7.0-8.9 (High) | 24 hours | PR within 24 hours; deploy within 48 hours |
| RUSTSEC with CVSS 4.0-6.9 (Medium) | 7 days | PR within 7 days |
| RUSTSEC with CVSS < 4.0 (Low) | 30 days | Track and patch in next scheduled maintenance |
| Yanked crate (no advisory) | 14 days | Replace with non-yanked version |

The on-call engineer is paged for Critical advisories. High advisories create a GitHub issue assigned to the Security Guild. Medium and Low create GitHub issues labeled `security` and `dependency`.

### 10.2 JavaScript/Node Dependency Policy

The `clients/web` package uses `npm` (or `pnpm` if `pnpm-lock.yaml` is present). Rules:

- `npm audit --audit-level=high` must pass in CI on every PR.
- `package-lock.json` is committed and must be up to date.
- No `*` or `latest` version pins in `package.json`; all dependencies must be semver-pinned.
- Major version bumps of `pixi.js` or `react` require an ADR.

### 10.3 License Policy

Allowed SPDX license identifiers for Rust crates:

```
MIT
Apache-2.0
Apache-2.0 WITH LLVM-exception
BSD-2-Clause
BSD-3-Clause
ISC
MPL-2.0
Unicode-DFS-2016
CC0-1.0
Unlicense
```

Explicitly forbidden licenses:

```
GPL-2.0          (copyleft; incompatible with proprietary distribution)
GPL-3.0          (copyleft)
LGPL-2.0         (copyleft; must be reviewed case-by-case; default: forbidden)
LGPL-2.1         (same)
LGPL-3.0         (same)
AGPL-3.0         (copyleft; absolutely forbidden)
SSPL-1.0         (source-available; not open source)
Commons Clause   (source-available restriction)
```

The `deny.toml` for `cargo-deny` encodes these rules and runs in nightly CI:

```toml
# deny.toml

[licenses]
allow = [
  "MIT",
  "Apache-2.0",
  "Apache-2.0 WITH LLVM-exception",
  "BSD-2-Clause",
  "BSD-3-Clause",
  "ISC",
  "MPL-2.0",
  "Unicode-DFS-2016",
  "CC0-1.0",
  "Unlicense",
]
deny = [
  "GPL-2.0",
  "GPL-3.0",
  "AGPL-3.0",
  "SSPL-1.0",
]
copyleft = "deny"
allow-osi-fsf-free = "neither"
confidence-threshold = 0.8

[advisories]
ignore = []   # No exemptions without ADR

[bans]
multiple-versions = "warn"    # Warn on duplicate crate versions; block if > 3 duplicates
wildcards = "deny"            # No wildcard version requirements
```

### 10.4 Security Audit Schedule

| Audit Type | Tool | Frequency | CI Integration |
|---|---|---|---|
| Rust advisory DB | `cargo audit` | Every PR + weekly nightly | Yes (blocks PR) |
| Rust license + ban | `cargo deny` | Every PR + weekly nightly | Yes (blocks PR) |
| Node.js advisory DB | `npm audit` | Every PR + weekly nightly | Yes (blocks PR) |
| SAST | `semgrep` | Every PR + nightly | Yes (blocks PR on high) |
| SBOM generation | `syft` | Every release | Yes |
| OSV database scan | `osv-scanner` | Weekly nightly | Yes (nightly) |
| Secrets detection | `gitleaks` | Pre-commit + every PR | Yes (blocks PR) |

---

## 11. Risk Controls

### 11.1 Risk Register

| Risk ID | Risk | Likelihood | Impact | Severity | Mitigation |
|---|---|---|---|---|---|
| R-01 | Determinism violation (D1-D7 breach) | Low | Critical | P0 | Double-run CI, seed-sweep, cross-platform parity, freeze mode |
| R-02 | Data loss (SQLite or PostgreSQL corruption) | Very Low | High | P1 | WAL mode, hourly backup, weekly restore test, signed artifacts |
| R-03 | Client desync (client state diverges from server) | Medium | High | P1 | Hash chain broadcast per N ticks, client-side verification |
| R-04 | WASM mod sandbox escape | Very Low | Critical | P0 | wasmtime 26.x fuel limits, memory cap, host-call allowlist, seccomp |
| R-05 | Replay bundle tampering | Very Low | High | P1 | Ed25519 signature on every bundle, verification API |
| R-06 | RNG seed leakage (seed exposed to clients) | Low | High | P1 | Seed never transmitted in RPC responses; only hash chain root |
| R-07 | Tick budget runaway (cascading slow ticks) | Low | Medium | P2 | Guardrail with 3-violation freeze threshold |
| R-08 | Dependency supply chain attack | Very Low | Critical | P0 | cargo audit, cargo deny, osv-scanner, SBOM, pinned versions |
| R-09 | Audit log tampering | Very Low | High | P1 | Per-entry hash chain, chunk Ed25519 signatures, append-only storage |
| R-10 | Schema migration failure (corrupt DB state) | Very Low | High | P1 | Transactional migrations, pre-migration backup, CI migration test |

### 11.2 Freeze Mode Runbook (Step-by-Step)

This runbook is executed whenever `FreezeModeActive` alert fires or `civlab_sim_freeze_mode_active > 0`.

```
RUNBOOK: Freeze Mode Response
Owner: On-call Engineer
Escalation: Sim Team Lead (P0/P1), Platform CTO (P0)
SLA: Acknowledge within 5 minutes; begin investigation within 15 minutes

STEP 1: ACKNOWLEDGE
  - Acknowledge the PagerDuty alert.
  - Post in #civlab-incidents Slack channel: "Investigating freeze mode on <run-id>"

STEP 2: IDENTIFY TRIGGER
  Run:
    civlab-cli freeze-status --run-id <run-id>

  Output includes: trigger type, tick number, guardrail values at freeze time.

  Trigger mapping:
    tick_budget_exceeded    -> Section 11.2.A: Tick Budget Runbook
    determinism_violation   -> Section 11.2.B: Determinism Violation Runbook (P0 ESCALATE NOW)
    memory_ceiling_exceeded -> Section 11.2.C: Memory Runbook
    entity_count_exceeded   -> Section 11.2.C: Memory Runbook
    wasm_sandbox_escape     -> Section 11.2.D: Security Incident Runbook (P0 ESCALATE NOW)
    manual_operator         -> Find the operator; ask why they froze it
    critical_metric_breach  -> Section 11.2.E: Metric Breach Runbook

STEP 3: VERIFY SNAPSHOT INTEGRITY
  Run:
    civlab-cli verify-snapshot --run-id <run-id> --tick <freeze-tick>

  If verification FAILS: this is a secondary incident (tampered snapshot).
  Escalate to Security Guild immediately.
  Do not resume the run.

STEP 4: INVESTIGATE
  See trigger-specific runbook section below.

STEP 5: DOCUMENT
  Before taking any action (resume or abort):
    civlab-cli incident create \
      --run-id <run-id> \
      --severity <P0|P1|P2> \
      --trigger <trigger-type> \
      --summary "<one-sentence description>"

STEP 6: RESOLVE
  If safe to resume:
    civlab-cli resume --run-id <run-id> --justification "<text>"

  If not safe to resume:
    civlab-cli abort --run-id <run-id> --reason "<text>"

STEP 7: POST-MORTEM
  For P0 and P1: a post-mortem is MANDATORY within 72 hours.
  Template: Section 11.4.


SECTION 11.2.A: Tick Budget Runbook
  1. Check recent tick latency metrics in Grafana.
  2. Look for entity count spikes: civlab_sim_entity_count{run_id=<run-id>}
  3. Check WASM mod memory usage: civlab_sim_wasm_memory_bytes{run_id=<run-id>}
  4. If spike is transient (< 10 ticks elevated): resume is likely safe.
  5. If sustained spike: do not resume; abort the run and investigate the scenario config.

SECTION 11.2.B: Determinism Violation Runbook (P0)
  1. ESCALATE TO SIM TEAM LEAD AND PLATFORM CTO IMMEDIATELY.
  2. Do NOT resume the run under any circumstances without CTO sign-off.
  3. Collect the freeze snapshot and the preceding snapshot (tick - 5 if available).
  4. Run: civlab-cli replay-diff \
       --bundle-a snapshots/<run-id>/freeze-<tick>.civsnap \
       --bundle-b snapshots/<run-id>/pre-freeze-<tick-5>.civsnap
  5. Identify the first divergent tick from the diff output.
  6. Check recent commits for changes to sim code that might affect D1-D7.
  7. File a GitHub issue labeled P0, determinism, with full diff output attached.
  8. Post-mortem is mandatory (Section 11.4).

SECTION 11.2.C: Memory/Entity Runbook
  1. Check entity count by archetype: civlab_sim_entity_count{run_id=<run-id>}
  2. Identify which archetype is growing unboundedly.
  3. Check the scenario config for uncapped entity-generating events.
  4. If the scenario config is the cause: abort the run; fix the config; re-run.
  5. If code is the cause: abort; file a bug; fix and release before re-running.

SECTION 11.2.D: Security Incident Runbook (P0)
  1. ESCALATE TO SECURITY GUILD AND CISO IMMEDIATELY.
  2. Emergency stop the server: civlab-cli emergency-stop --reason "WASM sandbox escape"
  3. Preserve all logs and snapshots.
  4. Do not restart the server until Security Guild clears it.
  5. Treat as a P0 security incident per the incident response process.

SECTION 11.2.E: Metric Breach Runbook
  1. Identify the metric that breached critical_high threshold.
  2. Check the metric definition in metrics/definitions/ for the expected range.
  3. Determine if the breach represents real simulation state or a metric computation bug.
  4. If real state: assess whether the scenario design is at fault; abort or resume.
  5. If metric bug: resume the run; file a bug for the metric computation.
```

### 11.3 Incident Severity Levels and Response Times

| Severity | Definition | Acknowledge | Begin Investigation | Resolve | Post-Mortem |
|---|---|---|---|---|---|
| P0 Critical | Determinism violation, WASM escape, data loss, security breach | 5 minutes | 15 minutes | Best effort / ASAP | Mandatory within 48 hours |
| P1 High | Freeze mode (non-P0 trigger), backup failure, metric breach | 15 minutes | 30 minutes | 4 hours | Mandatory within 72 hours |
| P2 Medium | Performance degradation, repeated tick budget violations | 1 hour | 2 hours | 24 hours | Optional; required if recurrence |
| P3 Low | Advisory-level issues, non-urgent dependency updates | 1 business day | 2 business days | 30 days | Not required |

On-call rotation is 24x7 for P0 and P1. P2 and P3 are handled during business hours.

### 11.4 Post-Mortem Template

Post-mortems are written as Markdown files in `docs/post-mortems/YYYY-MM-DD-<short-title>.md`:

```markdown
# Post-Mortem: <Short Title>

**Date:** YYYY-MM-DD
**Severity:** P0 | P1
**Author:** <Name>
**Reviewers:** <Names>
**Status:** Draft | Final

## Summary
One or two sentences describing what happened, impact, and duration.

## Timeline (UTC)
| Time | Event |
|---|---|
| HH:MM | Alert fired |
| HH:MM | On-call acknowledged |
| HH:MM | Root cause identified |
| HH:MM | Mitigation applied |
| HH:MM | Incident resolved |

## Root Cause
What was the fundamental cause? Be specific: which code, config, or infrastructure change caused the issue?

## Contributing Factors
What other factors made this incident possible or worse?

## Impact
- Runs affected:
- Clients affected:
- Data integrity impact:
- Duration:

## Detection
How was this detected? If detection was slow, why?

## Resolution
What steps were taken to resolve the incident?

## Action Items
| Action | Owner | Due Date | Status |
|---|---|---|---|
| Fix root cause | | | |
| Add test to prevent recurrence | | | |
| Improve detection/alerting | | | |

## Lessons Learned
What did we learn? What would we do differently?
```

---

## 12. Compliance and Auditability

### 12.1 FR Traceability

Every functional requirement must be traceable from spec to test to code. The traceability chain is:

```
FUNCTIONAL_REQUIREMENTS.md (FR-{CAT}-{NNN}: SHALL statement)
  -> Test file (// @trace FR-{CAT}-{NNN} comment)
    -> Implementation (/// @trace FR-{CAT}-{NNN} doc comment)
      -> PR (FR ID in description)
        -> Merge commit (squash: message includes FR ID)
```

The traceability check is automated in CI:

```yaml
  traceability-check:
    name: FR Traceability Check
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - name: Check all FR IDs have at least one test trace
        run: |
          cargo run -p civlab-cli -- check-traceability \
            --fr-file FUNCTIONAL_REQUIREMENTS.md \
            --test-dir crates/
```

The `check-traceability` command:
1. Parses all `FR-{CAT}-{NNN}` IDs from `FUNCTIONAL_REQUIREMENTS.md`.
2. Scans all test files under `crates/` for `// @trace FR-{CAT}-{NNN}` annotations.
3. Reports any FR IDs with zero test traces as errors (blocks PR merge).
4. Reports any test traces pointing to non-existent FR IDs as warnings.

### 12.2 Audit Trail Requirements

The following events MUST appear in the audit log. Missing any of these is a compliance violation:

| Event Category | Events | Required Fields |
|---|---|---|
| Simulation lifecycle | RunStarted, RunCompleted, RunAborted | run_id, scenario_id, rng_seed, tick_count, actor |
| Freeze and recovery | FreezeActivated, FreezeResumed, EmergencyStop, EmergencyCleared | run_id, tick, trigger, actor, justification |
| Runtime interventions | PolicyChanged, EventInjected | run_id, tick, actor, payload hash |
| Configuration changes | ConfigChanged, SigningKeyRotated | config_key, old_value_hash, new_value_hash, actor |
| Storage operations | DataDeleted (GC), PartitionDropped, BackupCompleted | affected_records, backup_path, operator |
| Security events | AuditChunkSigned, ArtifactVerificationFailed | chunk details, artifact path, reason |

The audit log must be queryable by `run_id`, `event_type`, `actor`, and time range:

```bash
# Query audit log for a specific run
civlab-cli audit query --run-id <run-id>

# Query all freeze events in the last 7 days
civlab-cli audit query --event FreezeActivated --since 7d

# Query all operator actions by a specific actor
civlab-cli audit query --actor <actor-id> --since 30d

# Verify audit log integrity (check hash chain and signatures)
civlab-cli audit verify --since 30d
```

### 12.3 Retention Policy for Logs and Artifacts

The following table is the single authoritative source for retention. It supersedes any conflicting statements elsewhere in this document.

| Data Category | Hot Retention | Cold Archive Retention | Total Retention | Legal Hold Override |
|---|---|---|---|---|
| Audit log entries | 90 days (database) | 2 years (cold export) | 2 years | Indefinite if legal hold flag set |
| Simulation reports | 30 days (database) | 2 years (cold storage) | 2 years | Indefinite |
| Replay bundles | 7 days (disk) | Scenario TTL + 90 days | Scenario-dependent | Indefinite |
| Freeze snapshots | Until incident resolved | 90 days after close | ~90-120 days | Indefinite |
| Structured logs ERROR | 90 days | 2 years cold | 2 years | Indefinite |
| Structured logs WARN | 30 days | 1 year cold | 1 year | Indefinite |
| Structured logs INFO | 90 days | None | 90 days | N/A |
| Signed release artifacts | 5 years (artifact store) | N/A | 5 years | Indefinite |
| SBOM files | 5 years (artifact store) | N/A | 5 years | Indefinite |

Retention automation:

```bash
# Check retention compliance (report only)
civlab-cli compliance retention-report --since 90d

# Apply retention policy (requires confirmation)
civlab-cli db gc --confirm

# Set legal hold on a run (prevents deletion)
civlab-cli compliance set-legal-hold --run-id <run-id> --reason "Active litigation"

# List all legal holds
civlab-cli compliance list-legal-holds
```

### 12.4 Governance Checkpoint Procedure

At the end of any significant implementation task, the implementing engineer runs a governance checkpoint before marking the task complete:

```bash
# Full governance checkpoint (run before closing any substantial PR)
task governance-check
```

```yaml
# Taskfile.yml: governance-check target

  governance-check:
    desc: "Run full governance checkpoint before marking a task complete"
    cmds:
      - echo "=== QG-01: Schema Validation ==="
      - task validate-all
      - echo "=== QG-02: Determinism Double-Run ==="
      - task det-double-run
      - echo "=== QG-05: Integration Matrix ==="
      - task test-integration-matrix
      - echo "=== QG-06: Replay Consistency ==="
      - cargo test -p civlab-sim --test replay_consistency -- --nocapture
      - echo "=== QG-09: Lint ==="
      - cargo fmt --all -- --check
      - cargo clippy --all-targets --all-features -- -D warnings
      - echo "=== QG-10: Dependency Audit ==="
      - cargo audit
      - echo "=== Traceability Check ==="
      - cargo run -p civlab-cli -- check-traceability --fr-file FUNCTIONAL_REQUIREMENTS.md --test-dir crates/
      - echo "=== Governance Check Complete ==="
```

The output of `task governance-check` is pasted into the PR description as evidence of compliance before requesting review.

### 12.5 Governance Review Schedule

This document and the following governance artifacts are reviewed on a scheduled basis:

| Document | Review Frequency | Reviewer | Next Review |
|---|---|---|---|
| OPS_GOVERNANCE_SPEC.md (this file) | Every 6 months | Platform CTO + Sim Team Lead | 2026-08-21 |
| FUNCTIONAL_REQUIREMENTS.md | Per release cycle | Sim Architect | Per release |
| ADR.md | Per ADR addition | Deciders named in ADR | Per ADR |
| `deny.toml` | Quarterly | Security Guild | 2026-05-21 |
| Grafana dashboards | Quarterly | Platform Team | 2026-05-21 |
| Alerting rules | Quarterly | Platform Team + On-call | 2026-05-21 |
| Backup and restore test results | Monthly | Infra Team | Monthly |
| Post-mortems action items | Monthly | Sim Team Lead | Monthly |

When a review is completed, the reviewer updates the `last_updated` frontmatter field on the document and creates a GitHub issue to schedule the next review. Reviews that surface deficiencies create follow-up issues labeled `governance`.

---

*End of Civ-Sim Ops/Governance Specification v1.0.0*



---

## Source: models/civ-sim/PRODUCT_MODEL.md

# CivLab Civ-Sim Product Model

**Version:** 0.2.0
**Status:** Active
**Owner:** CivLab Product / Engineering
**Last Updated:** 2026-02-21
**Audience:** Product managers, engineers, researchers, integration partners

---

## Table of Contents

1. [Executive Summary](#1-executive-summary)
2. [Problem Statement](#2-problem-statement)
3. [Product Vision and Strategy](#3-product-vision-and-strategy)
4. [Jobs To Be Done (JTBD)](#4-jobs-to-be-done-jtbd)
5. [Product Surfaces and Feature Map](#5-product-surfaces-and-feature-map)
6. [Simulation Domain Coverage](#6-simulation-domain-coverage)
7. [Technical Product Constraints](#7-technical-product-constraints)
8. [Competitive Analysis](#8-competitive-analysis)
9. [Business Model](#9-business-model)
10. [Success Metrics and KPIs](#10-success-metrics-and-kpis)
11. [Risks and Mitigations](#11-risks-and-mitigations)
12. [Roadmap and Milestones](#12-roadmap-and-milestones)
13. [Integration with Parpour/Venture](#13-integration-with-parpourventure)
14. [Governance and Decision Framework](#14-governance-and-decision-framework)

---

## 1. Executive Summary

### 1.1 Product Vision (Expanded)

CivLab is a **headless, deterministic civilization simulation engine** written in Rust. It models the full lifecycle of a society — energy production, resource distribution, institutional legitimacy, demographic change, climate feedback, and armed conflict — as a tick-driven state machine with byte-for-byte reproducible output.

The engine serves two distinct but deeply connected purposes:

1. **Standalone simulation platform** — enabling scenario designers, systems researchers, and policy analysts to define, run, compare, and inspect long-horizon "what if" experiments about governance, economics, and social stability.

2. **AI economic backend** — powering Parpour/Venture, an autonomous AI economic platform where AI agents run CivLab scenarios to validate, stress-test, and iteratively improve economic and governance policies before proposing them in real-world contexts.

CivLab is not a game engine. It is a scientific-grade simulation substrate. The simulation surface is domain-complete: every major lever of civilizational stability (energy, climate, institutions, demography, diplomacy, insurgency) is modeled with enough fidelity that meaningful policy experiments can produce unexpected and calibrated results.

### 1.2 Market Positioning

CivLab occupies a white space between:

- **Commercial strategy games** (Victoria 3, Stellaris) that simulate politics but are not programmable, not deterministic, and not API-accessible.
- **Agent-based modeling frameworks** (NetLogo, Mesa) that are programmable but lack deep civilizational domain models — researchers must build everything from scratch.
- **Academic simulation tools** (POLARIS, ASPATIAL) that are domain-specific, non-modular, and inaccessible to non-academics.

CivLab's positioning: **an open-source, embeddable, API-first civilization simulation engine that is rigorous enough for research and ergonomic enough for designers.**

```
                    Open Source / Embeddable
                              |
                              |  [CivLab]
            Broad Domain -----+------------- Narrow Domain
            Coverage          |               Coverage
                              |
                    Closed Source / Black Box
```

### 1.3 Core Value Propositions

| # | Value Proposition | Target | Differentiation |
|---|-------------------|--------|-----------------|
| 1 | **Byte-perfect determinism** | Researchers, AI agents | Seed → identical tick sequence guaranteed; D1-D7 ruleset enforced |
| 2 | **Domain completeness** | Policy analysts, designers | Energy, climate, institutions, demography, diplomacy, insurgency — all coupled |
| 3 | **Headless-first, embeddable** | Engineers, AI platforms | No GUI required; Rust crate + WASM target; integrate into any runtime |
| 4 | **Open-source core** | Community, academia | MIT/Apache-2.0 dual license; no vendor lock-in; forkable |
| 5 | **Modding platform** | Designers, researchers | WASM sandbox; four mod types; civlab-sdk; asset pipeline |
| 6 | **Parpour/Venture integration** | AI economic platforms | First-class API contract for AI agent scenario dispatch and result ingestion |

### 1.4 North Star Metric

> **Scenarios executed per day** across all deployment modes (local, cloud, Parpour/Venture) with 100% deterministic replay consistency.

This metric captures both adoption (volume of use) and core product health (determinism never regresses). A simulation platform that produces unreproducible results has zero scientific value. Growth in scenario volume with zero determinism regressions is the single best signal that CivLab is succeeding.

---

## 2. Problem Statement

### 2.1 Problems by Persona

#### Persona 1: Scenario Designer

The scenario designer builds simulation scenarios — defining initial conditions, governance structures, resource endowments, and policy regimes — to explore how civilizations evolve. They may be a game designer building a strategy title, a narrative designer constructing plausible historical counterfactuals, or an educator building teaching simulations.

**Core problems:**

- **Existing games (Victoria 3, Crusader Kings) are black boxes.** The designer can tweak surface parameters but cannot inspect the underlying model. When a society collapses, the cause is opaque. Designers cannot learn from the simulation.
- **Game engines are not simulation substrates.** Unity or Godot can render but do not provide energy/resource accounting, institutional modeling, or coupled social dynamics. Designers must build all domain logic from scratch.
- **Scenario authoring is code-heavy.** No domain-specific authoring language or structured UI for defining constitutions, resource endowments, and initial demographics exists in open tools.
- **Replay and comparison are absent.** Designers cannot branch a scenario at tick T, try two policy interventions, and compare the resulting divergence over 1000 ticks.

**Impact:** High iteration cost, low experimentation velocity, shallow scenario depth.

#### Persona 2: Policy Analyst / Systems Researcher

The policy analyst models real-world governance and economic systems to test hypotheses: "Does universal basic energy access reduce insurgency rates?" "How does climate shock timing interact with institutional resilience?" They may be academic researchers, think-tank analysts, or government modeling teams.

**Core problems:**

- **Agent-based frameworks (NetLogo, Mesa) require building everything from scratch.** A researcher must implement energy economics, climate feedback, institutional legitimacy, and social dynamics — a multi-year engineering effort — before running a single policy test.
- **Existing simulation tools are domain-siloed.** Climate models don't couple to governance. Economic models don't couple to demography. Real-world systems are deeply coupled; siloed models produce systematically misleading results.
- **Reproducibility is a crisis.** Published simulation results cannot be reproduced when tools are commercial, stochastic without controlled seeding, or dependent on deprecated environments.
- **Batch sweep infrastructure is DIY.** Running 10,000 parameter variations requires writing custom HPC scripts; there is no standard sweep API.

**Impact:** Slow research cycles, irreproducible findings, high infrastructure burden.

#### Persona 3: Research Operator (Parpour/Venture AI Agent)

The research operator is an autonomous AI agent (or the human orchestrating one) that uses CivLab as a simulation backend for policy testing. The agent proposes an economic or governance policy, dispatches a simulation scenario, evaluates the outcome metrics, and iterates. This is the Parpour/Venture integration persona.

**Core problems:**

- **No simulation engine exposes a clean programmatic API** for scenario definition, execution, and structured result retrieval. AI agents cannot interface with game UIs.
- **Stochastic simulations defeat AI learning loops.** If the same policy produces different outcomes each run (due to non-determinism), the AI agent cannot distinguish policy quality from random variance.
- **Result schemas are unstructured.** Game save files, NetLogo output CSVs, and academic tool outputs are not designed for machine consumption. AI agents need typed, versioned, schema-validated result payloads.
- **Long-horizon scenarios are computationally expensive.** Without performance guarantees (target: 100ms/tick), AI agent iteration loops are too slow for practical policy search.

**Impact:** AI policy agents cannot use existing simulation tools. CivLab is purpose-built to fill this gap.

### 2.2 Market Gap: Why Existing Tools Fail

| Tool | Strengths | Failure Mode for CivLab Use Cases |
|------|-----------|-----------------------------------|
| Dwarf Fortress | Deep simulation depth, citizen modeling | Closed source, not embeddable, non-deterministic, no API |
| Victoria 3 | Political economy, trade, demographics | Closed source, black box, not programmable, not headless |
| Factorio | Production chains, logistics, performance | No governance, no social dynamics, no demography |
| OpenTTD | Open source, moddable | Transport only; no political/social/economic governance |
| NetLogo / Mesa | Programmable, academic | No built-in domain models; build everything from scratch |
| AnyLogic | Hybrid simulation | Commercial, expensive, no civilization domain models |
| MASON / Repast | ABM frameworks | Research-grade, steep learning curve, no civilization models |

**The gap:** No tool simultaneously offers (a) deep civilizational domain models, (b) byte-perfect determinism, (c) headless embeddability, (d) open-source licensing, and (e) a clean API for programmatic scenario dispatch.

### 2.3 Opportunity Sizing

The addressable market for CivLab is segmented across three vectors:

**Open-source simulation community:** The intersection of strategy game developers, systems thinkers, and complexity researchers represents an estimated 50,000–200,000 technically proficient users globally. The success of Dwarf Fortress (2 million copies sold), OpenTTD (millions of downloads), and Factorio ($30M+ revenue) demonstrates strong market pull for deep, moddable simulation.

**Policy research and academic modeling:** University departments, government think-tanks, and international organizations (UN, World Bank) increasingly use computational simulation for policy analysis. Budget for simulation infrastructure at a single research institution can reach $100K–$500K/year.

**AI economic platforms (Parpour/Venture):** As AI agents are deployed for economic and policy analysis, demand for programmatic simulation backends is nascent but fast-growing. CivLab's Parpour integration targets this emerging segment directly, where the competitive set is essentially empty.

---

## 3. Product Vision and Strategy

### 3.1 Vision Statement

> CivLab is the open, deterministic substrate for civilizational simulation — a headless Rust engine that any researcher, designer, or AI agent can embed, extend, and trust to produce the same result from the same seed, every time, forever.

### 3.2 Strategic Pillars

#### Pillar 1: Determinism as Covenant

Determinism is not a feature — it is CivLab's foundational promise. Every product and engineering decision must preserve the D1-D7 determinism ruleset. Determinism erosion is an existential risk. The product will never ship a feature that compromises the seed-to-output guarantee, even at the cost of performance or expressiveness.

**Implications:**
- All randomness flows through ChaCha20Rng seeded from a user-supplied or system-derived seed.
- No wall-clock time, no float comparison, no hash map iteration order in simulation code.
- BLAKE3 state hash computed and exposed every tick for snapshot verification.
- CI pipeline enforces replay consistency as a hard gate.

#### Pillar 2: Domain Completeness Before Depth

The simulation must cover all major civilizational subsystems before deepening any single subsystem. A model that has a deep energy economy but no climate, no institutions, and no social dynamics produces fundamentally misleading results — it cannot capture real civilizational dynamics.

**Priority order:**
1. Energy economy (Joule Economy — the master constraint)
2. Climate (the long-horizon shock system)
3. Institutions (the governance and legitimacy layer)
4. Citizens/Demography (the social foundation)
5. Social/Insurgency (the legitimacy feedback)
6. War/Diplomacy (the external pressure)

Depth enhancements to any subsystem are deferred until all subsystems are present at MVP coverage.

#### Pillar 3: Headless-First, GUI as Optional Layer

CivLab's primary interface is its Rust API and WASM module. The Web RTS client (Pixi.js v8 + React 19) is a visualization layer, not the product. The Desktop client (Bevy 3D) is a future enhancement. All features must be exercisable without a GUI. GUI features that require GUI state unavailable to the headless API are not permitted.

**Implications:**
- Every simulation control available in the GUI must have a CLI/API equivalent.
- Scenarios are defined as structured data (TOML/JSON/MessagePack), not GUI-only workflows.
- Batch sweep, replay, and branch comparison are first-class headless operations.

#### Pillar 4: Open Ecosystem as Competitive Moat

CivLab's long-term defensibility comes not from the core engine alone but from the ecosystem built on top of it: community-created scenarios, WASM mods, research datasets, and integration adapters. The engine must be easy to embed, easy to mod, and easy to extend. The civlab-sdk and WASM registry are strategic investments in ecosystem lock-in through community, not through proprietary technology.

### 3.3 18-Month Roadmap Summary

| Phase | Timeline | Theme | Key Deliverables |
|-------|----------|-------|-----------------|
| Phase 0 | Months 0–2 | Core tick loop | Rust crate, ChaCha20Rng, BLAKE3 hash, D1-D7 harness, CI determinism gate |
| Phase 1 | Months 2–5 | Economy + Climate | Joule Economy (CIV-0100), Climate System (CIV-0102), metrics API |
| Phase 2 | Months 5–9 | Institutions + Citizens + Social | Institutions (CIV-0103), Demography (CIV-0104), Social/Insurgency (CIV-0106) |
| Phase 3 | Months 9–13 | War/Diplomacy + Mod Platform | War (CIV-0105), WASM mod sandbox (CIV-0700), civlab-sdk v0.1 |
| Phase 4 | Months 13–17 | Web client + Asset pipeline | Pixi.js Web RTS (CIV-0300), SDXL asset gen (CIV-0600), scenario authoring UI |
| Phase 5 | Months 17–24 | 3D + AI/NPC + Parpour GA | Bevy Desktop (CIV-0400), AI NPC (CIV-0601), Parpour/Venture GA integration |

### 3.4 Success Horizons

**At 12 months (Phase 3 complete):**
- All six simulation domains modeled at MVP coverage
- Headless API stable and versioned
- WASM mod sandbox operational
- Parpour/Venture beta integration running
- 10+ community-contributed scenarios in registry
- Zero determinism regressions in CI

**At 24 months (Phase 5 complete):**
- Web RTS client in public beta
- Bevy 3D desktop client in alpha
- Modding marketplace live with revenue share
- Cloud simulation credits platform in production
- 100+ community mods
- Parpour/Venture GA with SLA
- Academic publications citing CivLab reproducibility

---

## 4. Jobs To Be Done (JTBD)

The JTBD framework captures what users are trying to accomplish, at a level that transcends specific features. Jobs are expressed as: "When [situation], I want to [motivation], so I can [outcome]."

### 4.1 Scenario Designer JTBD

#### Functional Jobs

| Job ID | Job Statement | Priority |
|--------|---------------|----------|
| SD-F1 | When designing a new scenario, I want to define initial energy endowments, climate parameters, and institutional structures in a structured format, so I can start a simulation from a specific, reproducible initial state. | Critical |
| SD-F2 | When a simulation produces an unexpected collapse, I want to scrub back through the tick timeline and inspect the exact state at any tick, so I can identify the root cause of the failure. | Critical |
| SD-F3 | When testing a policy change, I want to branch the simulation at a specific tick and run two variants forward, so I can compare the counterfactual divergence over N ticks. | High |
| SD-F4 | When building a scenario library, I want to save, version, and share scenario definitions as portable artifacts, so collaborators can run the same scenario on their own machines. | High |
| SD-F5 | When the simulation produces interesting behavior, I want to see a visual timeline of key metrics (legitimacy, Joule stock, insurgency rate) so I can communicate the dynamic to stakeholders. | Medium |
| SD-F6 | When I want to extend the simulation beyond its built-in behaviors, I want to write a WASM mod (Policy, Economic, Event, or Scenario type) and load it into a running simulation. | Medium |

#### Social Jobs

| Job ID | Job Statement |
|--------|---------------|
| SD-S1 | Be recognized by the simulation design community as someone who builds rigorous, reproducible scenarios (not just "vibes-based" game balancing). |
| SD-S2 | Demonstrate to employers or collaborators that designed scenarios are based on a scientifically credible simulation substrate. |

#### Emotional Jobs

| Job ID | Job Statement |
|--------|---------------|
| SD-E1 | Feel confident that when I share a scenario, others will get exactly the same results I got. |
| SD-E2 | Feel in control of the simulation — understanding why things happen, not just watching them happen. |
| SD-E3 | Feel productive: rapid iteration without fighting tooling infrastructure. |

#### Pain Points and Gain Creators

| Pain Point | Severity | CivLab Gain Creator |
|------------|----------|---------------------|
| Black-box collapses with no traceable cause | High | Tick-level state inspection, BLAKE3 snapshot trail |
| No branching/counterfactual support | High | Branch API: fork at tick T, run N variants |
| Scenario sharing breaks due to non-determinism | High | Seed + initial-state artifact = portable, reproducible |
| Authoring requires writing raw code | Medium | TOML/JSON scenario definition + future authoring UI |
| Mod API is undocumented or unstable | Medium | civlab-sdk with versioned WASM API, typed mod types |

### 4.2 Policy Analyst / Systems Researcher JTBD

#### Functional Jobs

| Job ID | Job Statement | Priority |
|--------|---------------|----------|
| PA-F1 | When testing a policy hypothesis, I want to run 1,000+ parameter variations (sweep) from the CLI and collect structured result payloads, so I can do statistical analysis on outcome distributions. | Critical |
| PA-F2 | When writing a research paper, I want to publish a scenario artifact (seed + initial state + mod set) that any reader can run to reproduce my exact results, so my findings are independently verifiable. | Critical |
| PA-F3 | When studying climate-governance coupling, I want to see how climate shock timing interacts with institutional resilience across a parameter sweep, so I can identify non-linear thresholds. | High |
| PA-F4 | When analyzing instability events, I want the simulation to surface a structured causal trace (event chain: energy scarcity → legitimacy drop → insurgency threshold breach → state collapse), so I can validate or falsify theoretical claims. | High |
| PA-F5 | When comparing governance regimes, I want to run the same scenario under two constitutional configurations and see side-by-side metric divergence over 10,000 ticks. | High |
| PA-F6 | When I need to extend the model for a specific research question, I want to write a custom economic or event mod without forking the core engine. | Medium |

#### Social Jobs

| Job ID | Job Statement |
|--------|---------------|
| PA-S1 | Publish reproducible simulation results that survive peer review. |
| PA-S2 | Build a reputation for methodological rigor in computational social science. |
| PA-S3 | Collaborate with other researchers by sharing runnable, versioned simulation artifacts. |

#### Emotional Jobs

| Job ID | Job Statement |
|--------|---------------|
| PA-E1 | Trust that the simulation model is honest about its assumptions and limitations. |
| PA-E2 | Feel that the model is credible enough to cite in academic and policy contexts. |
| PA-E3 | Avoid the anxiety of discovering mid-paper that results cannot be reproduced. |

#### Pain Points and Gain Creators

| Pain Point | Severity | CivLab Gain Creator |
|------------|----------|---------------------|
| Building domain models from scratch (NetLogo/Mesa) | Critical | Pre-built coupled domain models; researcher focuses on policy levers |
| Irreproducible results block publication | Critical | Seed + BLAKE3 hash trail = cryptographic reproducibility |
| No batch sweep infrastructure | High | CLI sweep API: `civlab sweep --params params.toml --output results/` |
| Domain coupling is absent (siloed models) | High | All six domains modeled and coupled by design |
| Causal inspection is manual / impossible | High | Causal trace API: structured event chain for each instability |

### 4.3 Research Operator (Parpour/Venture AI Agent) JTBD

#### Functional Jobs

| Job ID | Job Statement | Priority |
|--------|---------------|----------|
| RO-F1 | When proposing a new economic policy, I want to dispatch a CivLab scenario via a typed API call, receive a structured result payload, and parse the outcome metrics programmatically, so I can score the policy without human intervention. | Critical |
| RO-F2 | When iterating on policy parameters, I want to run 100 scenario variants per hour with guaranteed ≤100ms/tick latency, so my policy search loop is fast enough to be practical. | Critical |
| RO-F3 | When a policy produces unexpected outcomes, I want to retrieve the full tick-level state trace for causal analysis, so I can identify which policy parameter caused which outcome. | High |
| RO-F4 | When comparing two policy variants, I want to branch from a shared initial state, run both variants, and receive a structured diff of outcome metrics, so I can rank policies by objective function. | High |
| RO-F5 | When deploying in production Parpour/Venture, I want the CivLab API contract to be versioned and stable, so my integration does not break when CivLab is updated. | High |

#### Social Jobs (of the AI agent's operator)

| Job ID | Job Statement |
|--------|---------------|
| RO-S1 | Demonstrate to Parpour/Venture stakeholders that AI policy proposals are grounded in simulation evidence, not heuristics. |
| RO-S2 | Build trust in AI-driven policy recommendations by making the simulation backend auditable and reproducible. |

#### Emotional Jobs (of the AI agent's operator)

| Job ID | Job Statement |
|--------|---------------|
| RO-E1 | Feel confident that simulation results are not contaminated by non-determinism, enabling fair policy comparison. |
| RO-E2 | Feel that the AI policy loop is scientifically credible, not a black box. |

### 4.4 JTBD to Feature Mapping

| Job ID | Primary Feature | Secondary Feature |
|--------|-----------------|-------------------|
| SD-F1 | Scenario authoring (TOML/JSON schema) | Scenario library / versioning |
| SD-F2 | Replay inspector / tick scrubbing | BLAKE3 snapshot trail |
| SD-F3 | Branch API | Divergence comparison view |
| SD-F4 | Portable scenario artifacts | Scenario registry |
| SD-F5 | Metrics dashboard | Timeline visualization |
| SD-F6 | WASM mod sandbox | civlab-sdk |
| PA-F1 | CLI sweep API | Result export (Parquet/JSON) |
| PA-F2 | Reproducibility artifact export | BLAKE3 verification tool |
| PA-F3 | Cross-domain metric correlation view | Parameter sweep heatmap |
| PA-F4 | Causal trace API | Instability event log |
| PA-F5 | Side-by-side metric comparison | Regime diff view |
| PA-F6 | WASM mod: Economic / Event types | civlab-sdk documentation |
| RO-F1 | Programmatic scenario dispatch API | Structured result schema |
| RO-F2 | Headless batch runner | Performance SLO (≤100ms/tick) |
| RO-F3 | Tick-level state trace API | Causal trace export |
| RO-F4 | Branch API (headless) | Metric diff API |
| RO-F5 | Versioned API contract | Changelog + deprecation policy |

---

## 5. Product Surfaces and Feature Map

### 5.1 Product Surface Overview

```
+------------------------------------------------------------------+
|                        CivLab Platform                           |
+------------------------------------------------------------------+
|                                                                  |
|  +------------------+   +------------------+   +--------------+ |
|  | Scenario         |   | Simulation       |   | Replay       | |
|  | Authoring UI     |   | Runner           |   | Inspector    | |
|  | (Web RTS)        |   | (Headless/GUI)   |   | (Web/CLI)    | |
|  +------------------+   +------------------+   +--------------+ |
|           |                     |                     |          |
|  +------------------+   +------------------+   +--------------+ |
|  | Metrics          |   | Policy           |   | CLI / API    | |
|  | Dashboard        |   | Intervention     |   | Surface      | |
|  | (Web RTS)        |   | Controls         |   | (Headless)   | |
|  +------------------+   +------------------+   +--------------+ |
|           |                     |                     |          |
|  +----------------------------------------------------------+    |
|  |                    civlab-core (Rust)                    |    |
|  |  Tick Loop | ChaCha20Rng | BLAKE3 | Domain Systems       |    |
|  +----------------------------------------------------------+    |
|                              |                                   |
|  +----------------------------------------------------------+    |
|  |              WASM Mod Sandbox + civlab-sdk               |    |
|  |  Policy | Economic | Event | Scenario mod types          |    |
|  +----------------------------------------------------------+    |
|                                                                  |
+------------------------------------------------------------------+
         |                    |                    |
+--------+--------+ +---------+--------+ +---------+--------+
| Web RTS Client  | | Desktop (Bevy 3D)| | Parpour/Venture  |
| (Pixi.js + R19) | | [Future]         | | AI Agent API     |
+-----------------+ +------------------+ +------------------+
```

### 5.2 Scenario Authoring Interface

#### Description

The scenario authoring interface is the entry point for defining the initial conditions of a simulation. It allows designers and researchers to specify all starting parameters of a civilization: its energy endowments, climate configuration, institutional structure, citizen demographics, governance constitution, and diplomatic relations.

The interface exists in two forms:
1. **Structured data format (primary):** TOML/JSON scenario definition files, fully functional without any GUI. This is the authoritative representation.
2. **Web UI (secondary):** A visual authoring layer in the Web RTS client (Pixi.js v8 + React 19) that generates and validates the structured data format. The UI is a convenience — not the source of truth.

#### Key Features

| Feature | Description | Acceptance Criteria |
|---------|-------------|---------------------|
| Constitution editor | Define governance type, election cycle, enforcement power, judicial independence | Validates against governance schema; invalid constitutions rejected at load time with error message |
| Resource endowment configurator | Set initial Joule stocks, production capacity (kJ/tick), distribution infrastructure rating | All values typed as KiloJoules (i64); range validation; negative stocks rejected |
| Climate profile selector | Set base temperature, precipitation, volatility, and initial climate shock schedule | Climate config validates against climate schema; out-of-range parameters rejected |
| Citizen demographics editor | Set population size, age distribution, skill distribution, faction composition | Faction percentages must sum to 100%; population ≥ 1 |
| Diplomatic relations matrix | Set initial alliance, trade, and hostility values between simulated entities | Symmetric validation (if A is allied with B, B must be allied with A) |
| Seed configurator | Set simulation seed (u64) or generate random seed | Seed displayed prominently; copied to clipboard on demand |
| Scenario validation | Pre-flight check: validate all fields, surface errors with path and message | No scenario dispatched with validation errors; errors listed with TOML/JSON key paths |
| Scenario export | Export scenario as portable artifact (TOML + metadata JSON + mod manifest) | Exported artifact is self-contained; can be loaded on a different machine and produce identical results |
| Scenario library | Save, tag, search, and load scenarios from local or remote registry | Scenarios addressable by name+version; BLAKE3 hash of scenario artifact for integrity |

#### User Actions (Primary Flows)

1. **New scenario:** User creates a blank scenario from template, fills in fields, validates, saves, dispatches.
2. **Fork scenario:** User loads existing scenario, modifies one parameter, saves as new version, dispatches in parallel.
3. **Import scenario:** User receives a scenario artifact from a collaborator, imports it, verifies BLAKE3 hash, dispatches.
4. **Export for publication:** User runs scenario, captures result, exports scenario artifact + result bundle for reproducibility.

#### Performance Targets

- Scenario validation: < 50ms for any scenario definition
- Scenario load from file: < 100ms
- Scenario export: < 200ms including BLAKE3 computation

### 5.3 Simulation Runner

#### Description

The simulation runner is the core execution engine. It accepts a scenario definition, initializes the simulation state, and advances the state by one tick per invocation of the tick function. The runner exists in three modes:

1. **Headless mode:** CLI or API invocation; no rendering; maximum throughput.
2. **Interactive mode:** Web RTS or Desktop GUI; renders state at configurable FPS; allows pause, step, fast-forward, intervention.
3. **Batch/sweep mode:** CLI or API; runs N scenarios in parallel; collects structured results.

#### Key Features

| Feature | Description | Acceptance Criteria |
|---------|-------------|---------------------|
| Deterministic tick loop | Advance simulation by one tick; all randomness from ChaCha20Rng; no side effects | Same seed + scenario → identical tick sequence; verified by CI replay test |
| BLAKE3 state hash | Compute BLAKE3 hash of full simulation state after each tick | Hash changes IFF state changes; stored in tick log; used for snapshot verification |
| Pause / resume | Stop tick advancement; resume from exact state | State is identical before and after pause/resume cycle |
| Step mode | Advance exactly one tick at a time | Available in all runner modes; useful for debugging |
| Fast-forward | Advance N ticks as fast as possible (no frame cap) | Headless: no frame cap; GUI: configurable multiplier (1x, 5x, 20x, 100x, max) |
| Intervention injection | Apply a policy intervention at the current tick without branching | Intervention is applied atomically; state hash reflects intervention |
| Branch at tick T | Snapshot state at tick T; spawn two runner instances from that snapshot | Both instances produce identical tick T state; diverge from tick T+1 forward |
| Scenario abort | Terminate simulation on abort condition (e.g., population → 0, Joule stock → 0) | Abort condition logged with tick, cause, and last state hash |
| Headless batch runner | Run N scenarios (same or different seeds/params) in parallel | Thread pool sizing configurable; results collected to structured output directory |
| Parameter sweep | Run Cartesian product of parameter ranges; collect result metrics | Sweep config: TOML file specifying parameter ranges; output: Parquet or JSON Lines |

#### Performance Targets

| Metric | Target | Notes |
|--------|--------|-------|
| Tick latency (single simulation) | ≤ 100ms per tick | Measured at p99 on reference hardware |
| Headless throughput | ≥ 600 ticks/minute per simulation instance | With all six domains active |
| Batch sweep throughput | ≥ 10,000 tick-scenarios/hour | On 8-core reference machine |
| Memory per simulation instance | ≤ 256MB | Full state in memory; no disk swapping |
| State hash computation | ≤ 5ms per tick | BLAKE3 is fast; must not dominate tick budget |

#### Tick Budget Allocation (Target, 100ms)

```
Tick Budget: 100ms
  ├── Joule Economy:       ~20ms
  ├── Climate System:      ~15ms
  ├── Institutions:        ~15ms
  ├── Citizens/Demography: ~20ms
  ├── Social/Insurgency:   ~10ms
  ├── War/Diplomacy:       ~10ms
  ├── BLAKE3 state hash:   ~5ms
  └── Overhead/serialize:  ~5ms
```

### 5.4 Replay Inspector and Timeline Viewer

#### Description

The replay inspector allows users to navigate the full tick history of a completed or paused simulation. It is the primary tool for post-hoc causal analysis: understanding why a civilization collapsed, why a legitimacy crisis emerged, or why an insurgency surged.

The inspector operates on a replay artifact: the complete sequence of state snapshots (or state diffs + full snapshots at interval) produced during a simulation run.

#### Key Features

| Feature | Description | Acceptance Criteria |
|---------|-------------|---------------------|
| Tick scrubbing | Navigate to any tick in the simulation history via slider or direct input | Seek to any tick in O(log N) time using snapshot index; state rendered within 200ms of seek |
| State inspection panel | View full simulation state at current tick: all domain metrics, entity states, event log | All domain state fields exposed; no hidden state |
| Event log | Chronological list of significant events (legitimacy threshold breach, insurgency outbreak, climate shock, war declaration, etc.) | Each event annotated with tick, type, causal chain summary, affected entities |
| Causal trace | For selected instability event, display structured causal chain: which domain transitions led to this event | Causal chain expressed as DAG: node = state transition, edge = causal dependency |
| Metric timeline | Time-series chart of any metric (Joule stock, legitimacy, insurgency rate, etc.) over the full simulation | Multiple metrics overlaid; cursor shows exact value at each tick; zoom to sub-range |
| Branch comparison | Load two replay artifacts from the same branch point; display side-by-side or overlay metric divergence | Tick alignment at branch point; divergence highlighted from tick T+1 |
| Annotation | Add user annotations to specific ticks or events for documentation or collaboration | Annotations stored in separate file; merged with replay artifact for sharing |
| Export | Export tick range as sub-replay or as CSV/Parquet for external analysis | Sub-replay is valid, self-contained replay artifact; CSV includes all metrics |
| BLAKE3 verification | Verify that replay artifact hashes match reference hashes for reproducibility | Verification result: pass/fail per tick; failures indicate determinism breach |

#### Replay Artifact Format

```
replay_artifact/
  metadata.json         # scenario seed, initial state hash, CivLab version, tick count
  snapshots/
    0000000000.snap     # full state snapshot at tick 0 (MessagePack)
    0001000000.snap     # full state snapshot at tick 1,000,000 (every N ticks)
  diffs/
    0000000001.diff     # state diff from tick 0→1 (MessagePack)
    ...
  events/
    events.json         # all significant events with tick, type, causal chain
  hashes/
    hashes.bin          # BLAKE3 hash per tick (binary, 32 bytes * N ticks)
```

### 5.5 Metrics Dashboard

#### Description

The metrics dashboard provides real-time and historical visualization of all tracked simulation metrics. It is the primary situational awareness surface for a running or completed simulation.

#### Tracked Metrics

**Joule Economy Metrics:**

| Metric | Type | Description | Unit |
|--------|------|-------------|------|
| joule_stock | Scalar | Total Joule reserves in system | kJ (i64) |
| joule_production_rate | Scalar | kJ produced per tick | kJ/tick |
| joule_consumption_rate | Scalar | kJ consumed per tick | kJ/tick |
| joule_distribution_efficiency | Ratio | Fraction of produced kJ reaching consumption nodes | 0.0–1.0 |
| joule_waste_rate | Scalar | kJ wasted per tick (not consumed, not stored) | kJ/tick |
| joule_surplus_rate | Scalar | kJ surplus per tick (produced - consumed - waste) | kJ/tick |
| joule_scarcity_index | Ratio | Population-weighted fraction experiencing energy deficit | 0.0–1.0 |
| per_capita_joule_access | Scalar | kJ per citizen per tick | kJ/citizen/tick |

**Governance and Institutional Metrics:**

| Metric | Type | Description | Unit |
|--------|------|-------------|------|
| institutional_legitimacy | Ratio | Population-weighted legitimacy score | 0.0–1.0 |
| tyranny_index | Ratio | Composite: enforcement overreach + political capture | 0.0–1.0 |
| institutional_resilience | Ratio | Capacity to absorb shocks without structural change | 0.0–1.0 |
| elite_capture_index | Ratio | Fraction of institutional decisions serving elite vs. population | 0.0–1.0 |
| policy_effectiveness | Ratio | Fraction of enacted policies producing intended outcome | 0.0–1.0 |
| election_cycle_health | Ordinal | Status of electoral processes (HEALTHY / STRESSED / SUSPENDED / COLLAPSED) | enum |

**Social and Demographic Metrics:**

| Metric | Type | Description | Unit |
|--------|------|-------------|------|
| population | Scalar | Total citizen count | persons |
| population_growth_rate | Scalar | Net population change per tick | persons/tick |
| gini_coefficient | Ratio | Resource inequality measure | 0.0–1.0 |
| social_cohesion | Ratio | Aggregate measure of inter-faction trust and cooperation | 0.0–1.0 |
| insurgency_rate | Ratio | Fraction of population in active insurgent activity | 0.0–1.0 |
| insurgency_severity | Ordinal | Composite severity (LATENT / ACTIVE / INSURGENCY / CIVIL_WAR) | enum |
| displacement_rate | Scalar | Citizens displaced per tick | persons/tick |

**Climate Metrics:**

| Metric | Type | Description | Unit |
|--------|------|-------------|------|
| mean_temperature | Scalar | Simulation-world mean temperature | degrees C (integer) |
| precipitation_index | Ratio | Normalized precipitation level | 0.0–1.0 |
| climate_shock_active | Boolean | Whether a climate shock event is currently active | bool |
| agricultural_yield_modifier | Ratio | Climate-driven modifier on food/resource production | 0.0–2.0 |
| climate_stress_index | Ratio | Long-run climate deviation from baseline | 0.0–1.0 |

**War and Diplomacy Metrics:**

| Metric | Type | Description | Unit |
|--------|------|-------------|------|
| conflict_active | Boolean | Whether armed conflict is currently occurring | bool |
| conflict_intensity | Ordinal | (NONE / SKIRMISH / CONFLICT / WAR / TOTAL_WAR) | enum |
| military_expenditure_ratio | Ratio | Fraction of Joule budget allocated to military | 0.0–1.0 |
| diplomatic_relations | Matrix | Pairwise relation score between simulated entities | -1.0–1.0 |
| territory_control | Map | Territory ownership by entity per tick | spatial |

#### Visualization Types

| Visualization | Metrics | Notes |
|---------------|---------|-------|
| Time-series line chart | All scalar/ratio metrics | Multi-metric overlay; log scale option |
| Stacked area chart | joule_production, joule_consumption, joule_waste, joule_surplus | Energy flow decomposition |
| Heat map (spatial) | territory_control, per-region resource access | Requires spatial grid model |
| Gauge (current tick) | institutional_legitimacy, tyranny_index, insurgency_rate, joule_scarcity_index | At-a-glance current state |
| Ordinal state badge | election_cycle_health, insurgency_severity, conflict_intensity | Color-coded: green/yellow/red/black |
| Scatter / correlation | Any two metrics over time | Useful for coupling analysis |
| Bar chart (distribution) | gini_coefficient, per_capita_joule_access by faction | Inequality visualization |
| Event log timeline | All events | Tick-indexed, filterable by type |

### 5.6 Policy Intervention Controls

#### Description

The policy intervention interface allows users to apply changes to a running simulation's governance parameters, resource allocations, or institutional structures — either as experimental interventions or as modeled policy actions.

Every intervention is a first-class simulation event: it is logged, causally attributable, and affects the BLAKE3 state hash. Interventions can be applied in interactive mode (via GUI) or in headless mode (via API or mod).

#### Intervention Types

| Intervention Type | Description | Authority Required | Branch |
|-------------------|-------------|-------------------|--------|
| Energy reallocation | Shift Joule distribution from one sector to another | Economic authority | Optional |
| Tax rate change | Modify the extraction rate from citizen surplus | Fiscal authority | Optional |
| Governance reform | Change institutional structure (election cycle, enforcement power, etc.) | Constitutional authority | Optional |
| Climate intervention | Apply geoengineering event (solar dimming, precipitation seeding) | N/A (researcher override) | Optional |
| Military mobilization | Allocate additional Joule budget to military; adjust conflict stance | Military authority | Optional |
| Diplomatic action | Send treaty proposal, impose sanctions, form alliance | Diplomatic authority | Optional |
| Emergency decree | Override normal governance channels for immediate action | Emergency authority | Recommended |
| Mod-defined intervention | Custom intervention type defined by a WASM Policy mod | Mod authority | Depends on mod |

#### Authority Model

The simulation models institutional authority: not all interventions are available at all times. Constitutional authority may be required for governance reforms; if that authority has been captured or delegated, the intervention may be blocked, delayed, or have unintended consequences.

In **researcher override mode** (enabled via API or CLI flag), all authority checks are bypassed. This is the default for headless research runs. In **realistic mode** (default for designed scenarios), authority checks are enforced by the institutions subsystem.

#### Branch Creation on Intervention

When applying an intervention in interactive mode, the user can optionally create a branch: the current state is snapshotted, the intervention is applied to a new runner instance, and the original instance continues unchanged. This allows comparison of "with intervention" vs. "without intervention" trajectories.

### 5.7 CLI and API Surface (Research Operator)

#### CLI Commands

```bash
# Run a single scenario
civlab run --scenario scenario.toml --seed 42 --ticks 10000 --output results/

# Run a parameter sweep
civlab sweep --params sweep.toml --output results/ --workers 8

# Replay a simulation
civlab replay --artifact results/run_42/ --from-tick 5000 --to-tick 6000

# Verify replay determinism
civlab verify --artifact results/run_42/ --hashes results/run_42/hashes.bin

# Branch a simulation
civlab branch --artifact results/run_42/ --at-tick 5000 \
  --intervention intervention_a.toml --output results/branch_a/

# Export metrics as CSV
civlab export --artifact results/run_42/ --metrics joule_stock,legitimacy --format csv

# Validate a scenario definition
civlab validate --scenario scenario.toml

# List available mods
civlab mods list

# Install a mod from registry
civlab mods install policy/universal-energy-access@0.3.2
```

#### HTTP API (for Parpour/Venture and programmatic access)

```
POST   /v1/scenarios                Create and dispatch a scenario
GET    /v1/scenarios/{id}           Get scenario status and metadata
GET    /v1/scenarios/{id}/results   Get structured result payload
POST   /v1/scenarios/{id}/branch    Branch at a specific tick
DELETE /v1/scenarios/{id}           Abort and clean up a scenario

GET    /v1/replays/{id}/ticks/{n}   Get full state at tick N
GET    /v1/replays/{id}/events      Get event log
GET    /v1/replays/{id}/causal/{n}  Get causal trace for event N
GET    /v1/replays/{id}/metrics     Get metric time series

POST   /v1/sweeps                   Submit a parameter sweep
GET    /v1/sweeps/{id}              Get sweep status
GET    /v1/sweeps/{id}/results      Get sweep result collection

GET    /v1/mods                     List available mods
POST   /v1/mods/{id}/install        Install a mod to a scenario
```

#### Result Payload Schema (JSON)

```json
{
  "scenario_id": "uuid",
  "seed": 42,
  "ticks_completed": 10000,
  "abort_reason": null,
  "final_state_hash": "blake3:...",
  "metrics": {
    "final": {
      "joule_stock": 1234567,
      "institutional_legitimacy": 0.72,
      "tyranny_index": 0.21,
      "insurgency_rate": 0.04,
      "population": 1500000,
      "conflict_active": false
    },
    "time_series": {
      "joule_stock": [/* array of i64 per tick */],
      "institutional_legitimacy": [/* array of f64 per tick */]
    }
  },
  "events": [
    {
      "tick": 4231,
      "type": "LEGITIMACY_CRISIS",
      "severity": "HIGH",
      "causal_chain": ["energy_scarcity_index > 0.6", "institutional_legitimacy < 0.4"]
    }
  ],
  "civlab_version": "0.2.0",
  "schema_version": "1.0.0"
}
```

### 5.8 Modding Platform

#### Overview

The CivLab modding platform allows developers to extend the simulation with custom behaviors without modifying the core engine. All mods run in a WASM sandbox with a capability-limited API surface. Mods are distributed through the CivLab registry (hosted) or as standalone WASM packages.

#### Four Mod Types

| Mod Type | Description | API Access | Example |
|----------|-------------|------------|---------|
| **Policy mod** | Defines a governance policy with effects on institutional and economic state | Economy, Institutions | Universal Basic Energy: every citizen receives minimum kJ/tick regardless of market |
| **Economic mod** | Adds or modifies production, distribution, or consumption rules | Economy | Renewable transition: replaces fossil Joule sources with renewable at defined rate |
| **Event mod** | Defines triggered events with conditions and state effects | All domains (read), Economy + Social (write) | Pandemic: reduces population and productivity under defined conditions |
| **Scenario mod** | Defines a full scenario template including initial state, constitution, and event schedule | Scenario authoring | Historical analog: pre-configured scenario approximating a historical civilization |

#### WASM Sandbox Constraints

- **Memory limit:** 64MB per mod instance
- **CPU budget:** max 10ms per tick contribution (enforced by host)
- **Capability model:** Mods declare required capabilities at load time; host validates and grants; no capability escalation at runtime
- **No I/O:** Mods cannot access filesystem, network, or system time
- **Determinism required:** Mods must be deterministic; any non-determinism is a validation failure at load time

#### civlab-sdk

The civlab-sdk is a Rust crate (and future bindings for other languages) that provides:

- Type definitions for all mod API types
- Macro helpers for mod registration
- Test harness for mod determinism and capability validation
- Documentation generator
- Local mod registry for development

```toml
# Cargo.toml for a Policy mod
[dependencies]
civlab-sdk = "0.1"

[lib]
crate-type = ["cdylib"]
```

#### Registry and Distribution

- **Registry URL:** registry.civlab.io
- **Mod manifest:** name, version, type, required capabilities, BLAKE3 hash, author
- **Installation:** `civlab mods install <type>/<name>@<version>`
- **Revenue share:** Paid mods in marketplace split revenue 70/30 (author/platform)

---

## 6. Simulation Domain Coverage

### 6.1 Domain Overview

CivLab models six coupled simulation domains. Each domain is a discrete subsystem with its own state, update logic, and event emission. Domains interact through a well-defined coupling interface — no domain directly mutates another domain's state. Cross-domain effects are mediated through the coupling layer.

```
+------------------+         +------------------+
|  Joule Economy   |<------->|  Climate System  |
+------------------+         +------------------+
        |  ^                         |  ^
        v  |                         v  |
+------------------+         +------------------+
|  Institutions    |<------->| Citizens/Demog.  |
+------------------+         +------------------+
        |  ^                         |  ^
        v  |                         v  |
+------------------+         +------------------+
| Social/Insurgency|<------->| War/Diplomacy    |
+------------------+         +------------------+
```

### 6.2 Joule Economy Subsystem

#### Purpose

The Joule Economy is the master constraint system. In CivLab, energy (measured in KiloJoules, represented as i64) is the primary resource. All production, distribution, consumption, and waste is denominated in kJ. Societies that cannot sustain energy production face cascading failures across all other domains.

#### Key Concepts

- **KiloJoule (kJ):** i64 newtype; the atomic unit of all economic transactions in CivLab.
- **Production:** Conversion of raw resources (modeled implicitly as capital stocks) into kJ per tick.
- **Distribution:** Movement of kJ from production nodes to consumption nodes; subject to infrastructure efficiency.
- **Consumption:** kJ consumed by citizens, institutions, military, and infrastructure per tick.
- **Waste:** kJ produced but neither consumed nor stored; represents inefficiency, corruption, or capacity limits.
- **Surplus:** kJ produced in excess of consumption and waste; accumulates in Joule stock.
- **Joule stock:** Reserve of kJ available for future ticks; represents civilizational energy savings.

#### Production → Distribution → Consumption → Waste Cycle

```
Resources → [Production] → kJ produced
                              |
                        [Distribution]
                        (efficiency %)
                         /           \
                   [Consumed]      [Lost/Waste]
                       |
                [Citizen needs]
                [Institutional ops]
                [Military budget]
                [Infrastructure]
```

#### Coupling Out (effects on other domains)

| Effect | Target Domain | Mechanism |
|--------|--------------|-----------|
| Energy scarcity → legitimacy loss | Institutions | `scarcity_index > threshold` triggers legitimacy decay |
| Energy scarcity → insurgency pressure | Social/Insurgency | Unfed citizens join insurgent pool |
| Military Joule allocation | War/Diplomacy | Military capacity is a function of allocated kJ/tick |
| Infrastructure investment | Climate | Resilience to climate shocks requires Joule investment |
| Surplus → demographic growth | Citizens/Demography | Surplus energy supports population growth |

#### FR Coverage

- CIV-0100: Joule Economy MVP
- CIV-0101: Production chain extensions (planned Phase 2)

### 6.3 Climate System

#### Purpose

The climate system models long-horizon environmental dynamics: temperature drift, precipitation variability, and discrete climate shock events (droughts, floods, extreme heat). Climate interacts with agricultural production (affecting resource availability), infrastructure resilience (affecting distribution efficiency), and displacement (affecting demography).

#### Key Concepts

- **Climate baseline:** Initial temperature and precipitation parameters set in scenario definition.
- **Climate drift:** Slow monotonic or oscillatory change in mean climate over simulation ticks.
- **Climate shock:** Discrete, high-impact event (drought, flood, storm) with defined duration and severity.
- **Agricultural yield modifier:** Climate-driven multiplier on food and biomass production.

#### Coupling Out

| Effect | Target Domain | Mechanism |
|--------|--------------|-----------|
| Drought → production loss | Joule Economy | Agricultural yield modifier reduces production kJ/tick |
| Flood → infrastructure damage | Joule Economy | Distribution efficiency decreases after flood event |
| Climate shock → displacement | Citizens/Demography | Displaced persons per tick increases during/after shock |
| Persistent climate stress → legitimacy pressure | Institutions | Long-run climate stress increases institutional demand for response |

#### FR Coverage

- CIV-0102: Climate System MVP

### 6.4 Institutions Subsystem

#### Purpose

The institutions subsystem models the governance layer: the structures through which a society makes collective decisions, enforces rules, and distributes authority. This includes executive, legislative, judicial, and electoral institutions.

#### Key Concepts

- **Constitution:** The foundational set of institutional rules (governance type, election cycle, rights, enforcement powers).
- **Legitimacy:** The population's acceptance of institutional authority. Legitimacy decays under scarcity, injustice, and ineffectiveness; it regenerates through effective policy and procedural justice.
- **Elite capture:** The degree to which institutional decisions serve narrow elite interests rather than the general population.
- **Enforcement power:** The institutional capacity to enforce rules; too low → rule breakdown; too high → tyranny.

#### Governance Types (MVP)

| Type | Description | Default Legitimacy | Default Tyranny |
|------|-------------|-------------------|-----------------|
| DEMOCRATIC | Elected government with institutional checks | 0.7 | 0.1 |
| AUTHORITARIAN | Centralized control; high enforcement, low checks | 0.4 | 0.6 |
| OLIGARCHIC | Small elite with nominal democratic structures | 0.5 | 0.4 |
| TECHNOCRATIC | Expert-managed with low popular accountability | 0.5 | 0.3 |
| ANARCHY | Absent central authority; emergent local governance | 0.3 | 0.0 |

#### Coupling Out

| Effect | Target Domain | Mechanism |
|--------|--------------|-----------|
| Policy effectiveness → Joule distribution | Joule Economy | Effective institutions improve distribution efficiency |
| Legitimacy collapse → insurgency threshold | Social/Insurgency | Legitimacy < 0.3 triggers insurgency escalation risk |
| Elite capture → resource extraction | Joule Economy | Captured institutions extract surplus to elite, not public investment |
| Institutional collapse → diplomatic vulnerability | War/Diplomacy | Weak institutions invite external aggression |

#### FR Coverage

- CIV-0103: Institutions MVP

### 6.5 Citizens and Demography Subsystem

#### Purpose

The citizens/demography subsystem models the human population: its size, age structure, skill distribution, faction composition, and material conditions. Citizens are the primary source of legitimacy (or its withdrawal), the labor for production, and the participants in insurgency and war.

#### Key Concepts

- **Population:** Total citizen count (integer; cannot be negative).
- **Age structure:** Distribution of citizens across age cohorts; affects labor capacity, dependency ratio, and military conscription pool.
- **Factions:** Named sub-groups with shared interests, grievances, and loyalty profiles. Factions are the primary actors in social and political dynamics.
- **Grievance:** Per-faction accumulation of unmet needs (energy, security, political representation). High grievance increases insurgency risk.
- **Material conditions:** Per-citizen kJ access, housing, and security. Derived from Joule Economy outputs.

#### Coupling Out

| Effect | Target Domain | Mechanism |
|--------|--------------|-----------|
| Population size → energy demand | Joule Economy | Consumption kJ/tick scales with population |
| Labor force → production capacity | Joule Economy | Production kJ/tick is a function of skilled labor supply |
| Faction grievance → insurgency pool | Social/Insurgency | High-grievance factions contribute to insurgent recruitment |
| Population → conscription pool | War/Diplomacy | Military manpower is a function of eligible population |
| Demographic pressure → institutional demand | Institutions | Population growth increases governance complexity and institutional load |

#### FR Coverage

- CIV-0104: Citizens/Demography MVP

### 6.6 Social and Insurgency Subsystem

#### Purpose

The social/insurgency subsystem models the transition from latent social tension to organized insurgency, and the dynamics of insurgency once active. It captures the legitimacy → grievance → insurgency escalation pathway and the feedback between insurgency, institutional response, and further legitimacy change.

#### Escalation Model

```
LATENT_TENSION → ACTIVE_DISCONTENT → INSURGENCY → CIVIL_WAR → STATE_COLLAPSE
      |                  |                |              |
  (grievance         (faction         (organized      (state
   accumulates)      organizing)       violence)      failure)
```

#### Key Dynamics

- **Grievance accumulation:** Energy scarcity, inequality, and legitimacy loss accumulate faction grievance per tick.
- **Insurgent recruitment:** When grievance exceeds faction threshold, citizens move from general population to insurgent pool.
- **Insurgent capacity:** Insurgent kJ access (from shadow economy or capture) determines conflict effectiveness.
- **Counterinsurgency:** Institutional enforcement actions reduce insurgent pool but risk legitimacy loss if applied excessively (tyranny feedback).
- **Negotiation:** Institutions can trade legitimacy concessions for insurgency de-escalation.

#### Coupling Out

| Effect | Target Domain | Mechanism |
|--------|--------------|-----------|
| Insurgency → production disruption | Joule Economy | Active insurgency reduces infrastructure efficiency |
| Civil war → institutional stress | Institutions | Civil war damages institutional capacity |
| Insurgency → displacement | Citizens/Demography | Conflict drives population displacement |
| Insurgency → diplomatic signal | War/Diplomacy | Internal instability invites external opportunism |

#### FR Coverage

- CIV-0106: Social/Insurgency MVP

### 6.7 War and Diplomacy Subsystem

#### Purpose

The war/diplomacy subsystem models inter-entity relations: alliances, trade agreements, military posture, and armed conflict. In multi-entity scenarios, diplomacy determines the external environment within which each entity's internal dynamics unfold.

#### Key Concepts

- **Entity:** The primary actor in the simulation (state, faction, city-state, etc.). Multiple entities can exist in one simulation.
- **Diplomatic relation:** A pairwise score (-1.0 = total war, +1.0 = full alliance) between entities.
- **Military capacity:** A function of allocated Joule budget + conscripted population + institutional effectiveness.
- **Conflict escalation:** Diplomacy scores below threshold → skirmish → conflict → war → total war.

#### Coupling Out

| Effect | Target Domain | Mechanism |
|--------|--------------|-----------|
| War → Joule drain | Joule Economy | Military operations consume kJ/tick at high rate |
| War → production disruption | Joule Economy | Combat damages production infrastructure |
| War → casualties | Citizens/Demography | Combat deaths reduce population |
| War → legitimacy pressure | Institutions | Prolonged war strains institutional legitimacy |
| Peace dividend | Joule Economy | Demobilization frees military kJ budget for civilian use |

#### FR Coverage

- CIV-0105: War/Diplomacy MVP

### 6.8 Cross-Domain Coupling Map

The following table summarizes all first-order coupling effects between domains. "→" means "affects."

| From Domain | To Domain | Effect Summary |
|-------------|-----------|----------------|
| Joule Economy | Institutions | Scarcity → legitimacy decay |
| Joule Economy | Social/Insurgency | Scarcity → grievance accumulation |
| Joule Economy | Citizens/Demography | Surplus → growth; scarcity → mortality |
| Joule Economy | War/Diplomacy | Military budget is kJ allocation |
| Climate | Joule Economy | Shocks → production/distribution loss |
| Climate | Citizens/Demography | Shocks → displacement |
| Climate | Institutions | Stress → governance demand |
| Institutions | Joule Economy | Policy effectiveness → distribution efficiency |
| Institutions | Social/Insurgency | Legitimacy floor → insurgency threshold |
| Institutions | War/Diplomacy | Weakness → diplomatic vulnerability |
| Citizens/Demography | Joule Economy | Population → demand; labor → supply |
| Citizens/Demography | Social/Insurgency | Faction grievance → insurgent pool |
| Citizens/Demography | War/Diplomacy | Population → conscription pool |
| Social/Insurgency | Joule Economy | Active insurgency → infrastructure disruption |
| Social/Insurgency | Institutions | Civil war → institutional capacity damage |
| Social/Insurgency | Citizens/Demography | Conflict → displacement |
| Social/Insurgency | War/Diplomacy | Internal instability → external opportunism |
| War/Diplomacy | Joule Economy | War → kJ drain + production disruption |
| War/Diplomacy | Citizens/Demography | Casualties → population loss |
| War/Diplomacy | Institutions | Prolonged war → legitimacy stress |

---

## 7. Technical Product Constraints

### 7.1 Determinism as Non-Negotiable

Determinism is the foundational technical constraint. It is not a feature to be traded off against performance or expressiveness. The D1-D7 ruleset defines the complete contract:

| Rule ID | Rule | Rationale |
|---------|------|-----------|
| D1 | All randomness flows through ChaCha20Rng seeded from the scenario seed (u64). | Controlled randomness = reproducibility |
| D2 | No wall-clock time access in simulation code. | System time is non-deterministic |
| D3 | No floating-point comparison with `==` or `!=`. | Float comparison is platform-dependent |
| D4 | No HashMap iteration order dependencies. | HashMap iteration is unordered in Rust |
| D5 | No thread-local state in simulation code. | Thread-local state is execution-context-dependent |
| D6 | All domain update functions are pure: output depends only on input + RNG state. | Side effects break reproducibility |
| D7 | BLAKE3 state hash computed and stored after every tick. | Enables external verification of replay fidelity |

**Enforcement:** D1-D7 violations are caught by:
- Static analysis (custom Clippy lints for D2, D3, D4, D5)
- CI replay test: every PR runs the reference scenario twice with the same seed and asserts byte-identical output
- BLAKE3 hash comparison: hashes from two replay runs must match at every tick

**Consequence of violation:** A determinism violation is a P0 bug. Any release that ships a known determinism violation is blocked.

### 7.2 Performance Targets

| Metric | Target | Measurement Method |
|--------|--------|--------------------|
| Tick latency (p99, single simulation) | ≤ 100ms | Benchmark suite (criterion) on reference hardware |
| Headless throughput | ≥ 600 ticks/min/instance | Benchmark suite |
| Batch sweep throughput | ≥ 10,000 tick-scenarios/hour | Integration benchmark |
| State hash computation | ≤ 5ms per tick | Profiled separately |
| Scenario load time | ≤ 100ms | End-to-end benchmark |
| Replay seek (to any tick) | ≤ 200ms | Replay benchmark |
| Memory per instance | ≤ 256MB | Memory profiler (Valgrind / heaptrack) |

**Reference hardware:** 8-core x86-64 Linux machine, 32GB RAM, NVMe SSD.

### 7.3 Portability

| Target | Status | Notes |
|--------|--------|-------|
| Linux x86-64 | Primary | CI + release target |
| macOS ARM64 (Apple Silicon) | Supported | CI target; developer primary |
| macOS x86-64 | Supported | CI target |
| Windows x86-64 | Supported | CI target; release binary |
| WASM32 (browser) | Supported | Web RTS client target; subset of API |
| WASM32-WASI | Planned | Server-side WASM deployment |
| Linux ARM64 | Planned | Cloud and embedded deployment |

**WASM constraints:** The WASM build excludes multi-threading (SharedArrayBuffer limitations in some environments) and filesystem access. Batch sweep and replay seek use in-memory representations in WASM builds.

### 7.4 Open-Source License

CivLab core engine is dual-licensed under **MIT / Apache-2.0** (user's choice). This is the standard Rust ecosystem dual license.

- **MIT:** Maximum permissiveness; compatible with commercial use.
- **Apache-2.0:** Adds patent protection; preferred by some enterprises.

**What is NOT open-source:**
- Cloud simulation credit platform (closed SaaS)
- Enterprise private cloud deployment tooling
- Parpour/Venture integration adapter (separate commercial license)
- SDXL asset generation pipeline (separate tooling)

### 7.5 Accessibility Requirements

| Requirement | Target | Notes |
|-------------|--------|-------|
| Web RTS UI color contrast | WCAG 2.1 AA | All metric gauges and state badges meet contrast ratio ≥ 4.5:1 |
| Keyboard navigation | Full keyboard coverage | All scenario authoring and replay inspector actions accessible via keyboard |
| Screen reader compatibility | ARIA labels on all interactive elements | Metrics dashboard uses chart.js with aria-label per data point |
| Colorblind mode | Deuteranopia + protanopia palettes | Metric charts offer colorblind-safe palette option |
| Font size | Min 14px body text in Web RTS | Scalable with browser zoom up to 200% without layout break |

---

## 8. Competitive Analysis

### 8.1 Comparison Matrix

| Dimension | CivLab | Dwarf Fortress | Victoria 3 | Factorio | OpenTTD | NetLogo/Mesa |
|-----------|--------|----------------|------------|----------|---------|--------------|
| Open source | Yes (MIT/Apache) | Partial (classic free, premium paid) | No | No | Yes (GPL) | Yes |
| Headless / embeddable | Yes | No | No | No | Partial | Yes |
| Deterministic replay | Yes (D1-D7) | No | No | Yes (partial) | No | Manual |
| Programmatic API | Yes (HTTP + CLI) | No | No | Mod API only | Partial | Script only |
| Energy economy | Joule Economy | Yes (deep) | Yes (trade) | Yes (deep) | No | No built-in |
| Climate system | Yes | Yes | Yes | No | No | No built-in |
| Governance/institutions | Yes | Partial | Yes (deep) | No | No | No built-in |
| Demography/citizens | Yes | Yes (deep) | Yes | No | No | No built-in |
| Social/insurgency | Yes | Partial | Yes (partial) | No | No | No built-in |
| War/diplomacy | Yes | Yes | Yes (deep) | No | No | No built-in |
| Mod platform | WASM + SDK | Custom (DFHack) | Paradox mod | Lua API | Squirrel API | Custom |
| Batch sweep / research | Yes | No | No | No | No | Yes |
| Causal trace / explainability | Yes | No | No | No | No | Manual |
| Performance SLO | 100ms/tick | Varies | Varies | High | High | Varies |
| AI agent integration | First-class | None | None | None | None | Manual |

### 8.2 Dwarf Fortress

**Strengths:**
- Unmatched simulation depth: fluid dynamics, geology, individual citizen psychology, artifact histories
- Decade-long community of dedicated players and modders
- No other simulation comes close in emergent narrative richness

**Weaknesses:**
- Not open-source in full (classic version is free but not open)
- Not embeddable: desktop application with complex rendering dependency
- Non-deterministic: no seed-to-output guarantee; replays are not guaranteed identical
- No programmatic API: cannot be controlled by external agents or scripts
- Steep learning curve; notoriously inaccessible to new users
- No causal trace: understanding WHY a fortress failed requires extensive manual inspection

**CivLab differentiation:** CivLab is not trying to match Dwarf Fortress's depth in any single subsystem. CivLab prioritizes breadth (all six civilizational domains), determinism, and embeddability. The target user for CivLab is a researcher or designer who needs a reproducible substrate, not a player seeking emergent narrative.

### 8.3 Victoria 3

**Strengths:**
- Sophisticated political economy: trade routes, pops, interest groups, laws
- Deep diplomatic and great-power dynamics
- High production value; accessible GUI

**Weaknesses:**
- Closed source; non-embeddable; no external API
- Non-deterministic; replays diverge
- Moddable but within Paradox's proprietary system; no WASM sandbox
- Political and trade model is rich but energy/Joule economy is absent
- Cannot be used as a research or AI backend

**CivLab differentiation:** CivLab provides a comparable political economy model (governance, legitimacy, elite capture) but adds the Joule Economy as the master constraint (Victoria 3 has no energy system), full determinism, and API-first headless operation.

### 8.4 Factorio

**Strengths:**
- Exceptionally polished production chain simulation
- High performance; scales to massive factory networks
- Excellent modding ecosystem (Lua API)
- Partial determinism in multiplayer

**Weaknesses:**
- No governance, institutions, or legitimacy
- No demography or social dynamics
- No climate system
- No diplomacy or armed conflict between civilizations
- Not a civilization simulator; it's a factory/logistics game

**CivLab differentiation:** Factorio's production chain model is an inspiration for CivLab's Joule Economy subsystem (production → distribution → consumption pipeline), but Factorio has zero coverage of the governance, social, climate, and conflict domains that are CivLab's core value.

### 8.5 OpenTTD

**Strengths:**
- Fully open-source (GPL); highly moddable
- Active community; long-term maintained
- Transport/logistics model is well-designed

**Weaknesses:**
- Transport and logistics only; no political/social/governance model
- No energy economy beyond transport fuel
- No demography, climate, or social dynamics
- No API for external agent control

**CivLab differentiation:** OpenTTD demonstrates that an open-source, community-driven simulation can build a large and loyal user base. CivLab aims to be OpenTTD for civilizational dynamics: an open, extensible substrate that the community can build on.

### 8.6 Summary: CivLab Differentiation

CivLab's unique position in the market is the combination of:

1. **Deterministic, byte-for-byte reproducible simulation** — no other tool in this space offers a formal D1-D7 determinism guarantee.
2. **All six civilizational domains, coupled** — no other open-source tool models energy + climate + governance + demography + social + war in a single coupled simulation.
3. **Headless, embeddable, API-first** — designed from day one for programmatic control by external agents (AI or human).
4. **Open-source core with commercial cloud layer** — community builds on the MIT/Apache-2.0 core; commercial value captured in cloud credits and enterprise deployment.

---

## 9. Business Model

### 9.1 Revenue Streams

| Stream | Type | Description | Target Customer |
|--------|------|-------------|-----------------|
| Open-source core | No revenue (cost center) | MIT/Apache-2.0 engine; community development | All users |
| Cloud simulation credits | Usage-based SaaS | Pay-per-tick on managed cloud infrastructure | Research operators, Parpour/Venture |
| Enterprise license | Annual SaaS + support | Private cloud deployment, SLA, dedicated infra, priority support | Enterprise research, government |
| Modding marketplace | Revenue share (30%) | Platform fee on paid mods sold through civlab registry | Mod developers |
| Parpour/Venture integration | API licensing + revenue share | Commercial API contract for Venture AI agent integration | Parpour |
| Training and consulting | Professional services | Scenario design, research methodology, custom mod development | Research institutions |

### 9.2 Pricing Model: Cloud Simulation Credits

Cloud simulation credits are the primary commercial revenue mechanism. Pricing is based on tick-compute units:

| Unit | Definition | Price (indicative) |
|------|------------|-------------------|
| 1 tick-compute unit (TCU) | 1 simulation tick on reference compute (8-core, 32GB RAM) | $0.0001 |
| Scenario bundle | 10,000 TCU | $1.00 |
| Research pack | 1,000,000 TCU + priority queue | $80/month |
| Enterprise tier | Unlimited TCU + private infra + SLA | Custom contract |

**Free tier:** 10,000 TCU/month free for registered users. Sufficient for exploration and small research runs.

### 9.3 Cost Model

| Cost Component | Estimate | Notes |
|----------------|----------|-------|
| Compute per tick | ~$0.00003 | On cloud VM; 100ms/tick target |
| Storage per scenario (10,000 ticks) | ~$0.01 | Replay artifact + metric time series |
| Storage per state snapshot | ~$0.001 | Compressed MessagePack full state |
| Gross margin target | ~70% | At scale; cloud infra costs dominate at small scale |

### 9.4 Open-Source Strategy

The OSS core is a strategic asset, not a cost. It:
- Drives community adoption and contribution
- Creates ecosystem (mods, scenarios, integrations) that increases switching cost
- Establishes CivLab as the credible, auditable substrate for research (reproducibility requires open source)
- Enables academic citations and research partnerships
- Reduces marketing cost: organic growth through GitHub, academic papers, and community

**OSS governance:** The core engine is maintained by the CivLab team with community contributions accepted via RFC process (see Section 14). The cloud platform, enterprise tooling, and Parpour integration are proprietary.

---

## 10. Success Metrics and KPIs

### 10.1 North Star Metric

**Scenarios executed per day** across all deployment modes, with 100% deterministic replay consistency.

- **Volume component:** Total scenarios dispatched (local + cloud + Parpour/Venture)
- **Quality gate:** Zero determinism regressions; any regression resets the quality gate

### 10.2 Product Health KPIs

| KPI | Definition | Target (12mo) | Target (24mo) | Measurement |
|-----|------------|---------------|---------------|-------------|
| Deterministic replay consistency | % of replay runs producing byte-identical output to original | 100% | 100% | CI test; automated nightly replay |
| Tick latency p99 | 99th percentile tick latency (ms) | ≤ 100ms | ≤ 80ms | Criterion benchmark |
| Scenarios executed/day | Total scenarios dispatched across all modes | 1,000/day | 10,000/day | Platform telemetry |
| Explainability score | % of instability events with a structured causal trace | 80% | 95% | Test coverage of causal trace API |
| Domain coverage | Fraction of planned domains at MVP coverage (6 total) | 6/6 | 6/6 (+ depth) | FR tracker |
| API adoption | % of scenarios dispatched via headless API vs. GUI | 60% API | 75% API | Platform telemetry |

### 10.3 Ecosystem KPIs

| KPI | Definition | Target (12mo) | Target (24mo) |
|-----|------------|---------------|---------------|
| Community scenarios in registry | Number of community-contributed scenario artifacts | 25 | 200 |
| Published mods | Number of mods in civlab registry | 10 | 100 |
| GitHub stars | civlab-core repository stars | 1,000 | 5,000 |
| Academic citations | Published papers citing CivLab | 3 | 20 |
| Monthly active researchers | Unique users running batch sweeps | 50 | 500 |

### 10.4 Developer Experience KPIs

| KPI | Definition | Target |
|-----|------------|--------|
| Time-to-first-run | Time from `cargo install civlab` to first scenario execution | ≤ 5 minutes |
| Time-to-first-sweep | Time from first run to first batch sweep | ≤ 15 minutes |
| Time-to-first-mod | Time from civlab-sdk installation to first working WASM mod | ≤ 60 minutes |
| Scenario validation error clarity | % of users who self-resolve validation errors without docs | 80% |
| CI build time | Total CI time per PR | ≤ 10 minutes |

### 10.5 Parpour/Venture Integration KPIs

| KPI | Definition | Target |
|-----|------------|--------|
| Venture API uptime | % uptime of CivLab API serving Venture agents | 99.9% |
| Venture scenario throughput | Scenarios/hour dispatched by Venture agents | ≥ 500/hour |
| Venture result latency | p99 latency from scenario dispatch to result retrieval | ≤ 30 seconds |
| Venture determinism rate | % of Venture-dispatched scenarios with verified deterministic replay | 100% |

---

## 11. Risks and Mitigations

### 11.1 Risk Register

| Risk ID | Risk | Probability | Impact | Severity |
|---------|------|-------------|--------|----------|
| R-01 | Determinism erosion: a subsystem or mod introduces non-determinism silently | Medium | Critical | Critical |
| R-02 | Performance regression: tick latency exceeds 100ms target after adding new domains | Medium | High | High |
| R-03 | Complexity ceiling: scenario authoring becomes too complex for non-experts | High | High | High |
| R-04 | Competition: well-funded commercial simulation tool copies CivLab's positioning | Low | High | Medium |
| R-05 | WASM mod security: malicious mod escapes sandbox | Low | Critical | High |
| R-06 | Ecosystem fragmentation: community forks produce incompatible scenario formats | Medium | Medium | Medium |
| R-07 | Parpour/Venture dependency: CivLab becomes too tightly coupled to Venture's specific needs | Medium | Medium | Medium |
| R-08 | Adoption plateau: open-source community does not grow beyond early adopters | Medium | Medium | Medium |

### 11.2 Risk Mitigations

#### R-01: Determinism Erosion

**Mitigation strategy:**
- D1-D7 ruleset is the authoritative contract; violations are P0 bugs.
- Custom Clippy lints enforce D2 (no system time), D3 (no float `==`), D4 (no HashMap iteration order).
- CI replay test runs on every PR: same seed + scenario → must produce byte-identical output. Hard gate; PR blocked if replay diverges.
- BLAKE3 hash per tick enables external verification; any hash mismatch is a determinism breach.
- Mod validation: WASM mods are tested for determinism at installation time using the civlab-sdk test harness. Non-deterministic mods are rejected.
- Scheduled fuzz testing: nightly CI run replays random scenarios with random seeds; any divergence triggers alert.

#### R-02: Performance Regression

**Mitigation strategy:**
- Criterion benchmark suite runs on every PR; performance regressions ≥ 5% relative to baseline trigger a review gate (not hard block, but requires explicit sign-off).
- Tick budget allocation document (Section 5.3) defines per-domain budget. Any domain exceeding its budget triggers a profiling requirement.
- Performance benchmarks are tracked in a time-series dashboard; trends are reviewed weekly.
- Domain implementations use SIMD and cache-friendly data layouts where applicable.
- bevy_ecs is being evaluated for potential ECS-based parallelization of independent domain updates.

#### R-03: Complexity Ceiling

**Mitigation strategy:**
- Scenario authoring schema is designed with progressive disclosure: minimal required fields; optional fields with sensible defaults.
- Schema validation provides clear, path-specific error messages. "Expected kJ value in range [0, i64::MAX] for field `energy.joule_stock`, got: -1000" — not "invalid config".
- Template library: pre-built scenario templates cover common starting conditions (medieval agrarian, industrial transition, post-scarcity, resource-constrained).
- Web authoring UI (Phase 4) provides guided workflow for common scenario patterns.
- Researcher documentation: quick-start guide targets ≤ 30-minute time-to-first-sweep for a policy analyst with no prior CivLab experience.
- Community scenario registry provides example scenarios that users can inspect, fork, and modify.

#### R-04: Competition Risk

**Mitigation strategy:**
- CivLab's moat is ecosystem, not technology alone. A competitor can build a similar engine, but cannot replicate 5 years of community scenarios, mods, and academic citations.
- Open-source licensing makes CivLab the default reference implementation. Even if a commercial tool exists, researchers require open-source for reproducibility.
- Parpour/Venture integration provides a durable commercial relationship that a new entrant cannot easily replicate.
- First-mover advantage in the AI agent simulation backend space is significant; the Parpour integration establishes CivLab as the reference implementation before competitors exist.

#### R-05: WASM Mod Security

**Mitigation strategy:**
- WASM sandbox runs in a separate process from the simulation engine. Process isolation contains any sandbox escape.
- Capability model is deny-by-default: mods must declare required capabilities at load time; undeclared capability access is a hard error.
- Memory limit (64MB) and CPU budget (10ms/tick) enforced by host; violations terminate the mod instance.
- Mod registry requires BLAKE3 hash verification of all installed mods; tampered mods rejected.
- Security audit of WASM sandbox implementation before civlab-sdk v1.0 release.

#### R-06: Ecosystem Fragmentation

**Mitigation strategy:**
- Scenario format is versioned (semver); schema migrations are provided by the civlab-core library.
- BLAKE3 hash of scenario artifact ties scenario to specific civlab-core version; incompatible versions are flagged at load time.
- RFC process (Section 14) requires community review for any breaking change to the scenario format.
- Scenario registry enforces format version; incompatible submissions are rejected with clear migration instructions.

#### R-07: Parpour/Venture Coupling Risk

**Mitigation strategy:**
- CivLab API is designed as a general-purpose simulation API, not a Venture-specific API. Venture is a first-class integration, not the only integration.
- Venture-specific API extensions (if needed) are implemented as a separate adapter layer, not in civlab-core.
- API versioning and changelog policy (Section 5.7) ensures backward compatibility for Venture's integration.
- Architecture review required for any CivLab change requested exclusively by Venture without general applicability.

#### R-08: Adoption Plateau

**Mitigation strategy:**
- Academic publishing: prioritize features that enable reproducible research (batch sweep, scenario export, causal trace). Academic citations are organic growth.
- Scenario design contest: annual community contest for most interesting/surprising scenario outcomes.
- Direct outreach to policy research institutions and complexity science departments.
- Parpour/Venture integration creates a showcase use case (AI-backed policy analysis) that generates press and community interest.

---

## 12. Roadmap and Milestones

### 12.1 Phase Table

| Phase | Timeline | Theme | Key Features | FR IDs | Success Criteria | Est. Complexity |
|-------|----------|-------|-------------|--------|-----------------|-----------------|
| Phase 0 | M0–M2 | Core tick loop | Rust crate, ChaCha20Rng seeding, BLAKE3 hash per tick, D1-D7 harness, CI replay gate, basic scenario TOML loader | CIV-0001–0010 | Replay test passes; tick loop runs at ≥ 10 ticks/ms; zero determinism violations in fuzz test | Medium |
| Phase 1 | M2–M5 | Economy + Climate | Joule Economy (production/distribution/consumption/waste cycle, KiloJoule type), Climate System (baseline, drift, shocks, yield modifier), metrics API for both domains | CIV-0100, CIV-0102 | All Joule Economy metrics tracked; climate shock events trigger and resolve correctly; batch sweep runs 100 variants | High |
| Phase 2 | M5–M9 | Institutions + Citizens + Social | Institutions (constitution, legitimacy, tyranny, elite capture), Citizens/Demography (population, age, factions, grievance), Social/Insurgency (escalation model, insurgent pool, counterinsurgency) | CIV-0103, CIV-0104, CIV-0106 | Legitimacy → insurgency pathway produces expected escalation; faction grievance model calibrated against reference scenarios | Very High |
| Phase 3 | M9–M13 | War/Diplomacy + Mod Platform | War/Diplomacy (entity relations, military capacity, conflict escalation), WASM mod sandbox (four mod types, capability model, memory/CPU limits), civlab-sdk v0.1 | CIV-0105, CIV-0700 | Multi-entity diplomatic simulation runs correctly; first community mod published and validated | Very High |
| Phase 4 | M13–M17 | Web client + Asset pipeline | Pixi.js v8 + React 19 Web RTS client, scenario authoring UI, metrics dashboard, replay inspector, SDXL asset generation pipeline | CIV-0300, CIV-0600 | Time-to-first-run ≤ 5min via Web UI; metrics dashboard displays all six domain metrics; replay seek latency ≤ 200ms | High |
| Phase 5 | M17–M24 | 3D + AI/NPC + Parpour GA | Bevy 3D Desktop client (CIV-0400), AI NPC integration (CIV-0601), Parpour/Venture GA integration, cloud simulation credits platform | CIV-0400, CIV-0601 | Venture AI agents run 500+ scenarios/hour; Bevy client renders 10,000-citizen simulation at ≥ 30fps; cloud credits platform in production | Very High |

### 12.2 Phase 0: Core Tick Loop (Months 0–2)

**Goal:** A minimal, correct, deterministic tick loop with no domain logic. The foundation that all subsequent phases build on.

**Deliverables:**
- `civlab-core` Rust crate published to crates.io (v0.0.1, pre-release)
- `ChaCha20Rng` integration: seeded from u64 scenario seed
- `BLAKE3` state hash computed after every tick
- D1-D7 rules encoded as custom Clippy lints (D2, D3, D4, D5)
- CI replay test: two runs with same seed → byte-identical BLAKE3 hashes at every tick
- Basic scenario TOML loader: seed, tick count, domain stubs
- Basic CLI: `civlab run --scenario scenario.toml --ticks 1000`
- Unit test coverage ≥ 90% for tick loop and hash logic

**Acceptance criteria:**
- CI replay test passes on Linux x86-64, macOS ARM64, macOS x86-64, Windows x86-64
- Tick loop runs at ≥ 10 ticks/ms with empty domain stubs (performance baseline)
- D1-D7 lints catch known violation examples in lint tests

### 12.3 Phase 1: Economy + Climate (Months 2–5)

**Goal:** The two foundational domain systems: the Joule Economy (master constraint) and the Climate System (long-horizon shock driver).

**Deliverables:**
- Joule Economy: KiloJoule i64 newtype, production/distribution/consumption/waste tick logic, Joule stock accumulation, scarcity index, per-capita access metric
- Climate System: baseline parameters, drift model, shock event scheduler, agricultural yield modifier
- Cross-domain coupling: climate shock → production loss; climate → distribution efficiency
- Metrics API: all Joule Economy and Climate metrics accessible via `civlab metrics get`
- Batch sweep CLI: `civlab sweep --params sweep.toml --workers 8 --output results/`
- Result export: JSON Lines and CSV output formats
- Integration tests: 10 reference scenarios with expected metric ranges; CI validates on every PR

**Acceptance criteria:**
- All Joule Economy metrics tracked at every tick with correct accounting (production = consumption + waste + stock delta)
- Climate shock events trigger at scheduled ticks, affect yield modifier correctly, and resolve after specified duration
- Batch sweep runs 100 variants in < 60 seconds on reference hardware
- Zero determinism violations in 10,000-tick fuzz test with 100 random seeds

### 12.4 Phase 2: Institutions + Citizens + Social (Months 5–9)

**Goal:** The three human-systems domains: governance structures, demographic dynamics, and social conflict.

**Deliverables:**
- Institutions: governance type enum, constitution schema, legitimacy model, tyranny index, elite capture model, policy effectiveness
- Citizens/Demography: population integer, age structure cohorts, faction system, grievance accumulation, material conditions
- Social/Insurgency: escalation state machine (LATENT → CIVIL_WAR), insurgent pool, counterinsurgency mechanics, negotiation mechanic
- Cross-domain coupling: energy scarcity → legitimacy; legitimacy → insurgency; faction grievance → insurgent pool; insurgency → production disruption
- Causal trace API: structured event chain for instability events
- Reference scenario set: 5 designed scenarios demonstrating legitimacy crisis, insurgency outbreak, elite capture, and faction conflict
- Documentation: domain model documentation for all three subsystems

**Acceptance criteria:**
- Legitimacy → insurgency escalation pathway produces expected state machine transitions in reference scenarios
- Causal trace API returns structured chain for ≥ 80% of instability events
- Faction grievance model: all five reference scenarios produce expected faction behavior within 5% metric tolerance
- No regression in Phase 1 determinism or performance

### 12.5 Phase 3: War/Diplomacy + Mod Platform (Months 9–13)

**Goal:** External conflict dynamics and the modding platform that enables community extension.

**Deliverables:**
- War/Diplomacy: entity model (multi-entity scenario), diplomatic relation matrix, military capacity model, conflict escalation state machine, peace settlement mechanic
- WASM mod sandbox: wasmtime integration, capability model, memory/CPU limits, four mod types (Policy, Economic, Event, Scenario)
- civlab-sdk v0.1: Rust crate with type definitions, registration macros, test harness, documentation
- Mod registry: basic hosted registry at registry.civlab.io with hash verification
- Reference mods: one working example of each mod type
- Cross-domain coupling: war → Joule drain; war → casualties; war → legitimacy stress

**Acceptance criteria:**
- Multi-entity scenario with two civilizations in diplomatic conflict runs correctly with no determinism violations
- WASM sandbox correctly enforces memory limit (64MB), CPU budget (10ms/tick), and capability model
- First community-contributed mod validated and published to registry
- civlab-sdk example mod builds, validates, and runs without errors

### 12.6 Phase 4: Web Client + Asset Pipeline (Months 13–17)

**Goal:** The primary GUI surface: the Web RTS client built with Pixi.js v8 and React 19.

**Deliverables:**
- Pixi.js v8 + React 19 Web RTS client
- Scenario authoring UI: form-based scenario definition with validation
- Simulation runner UI: start/pause/step/fast-forward, tick counter, status panel
- Metrics dashboard: time-series charts for all domain metrics, event log, metric gauges
- Replay inspector: tick scrubber, state inspection panel, metric timeline, BLAKE3 verification
- Policy intervention controls: intervention type menu, authority display, branch creation
- SDXL asset generation pipeline: procedural terrain, unit, and building sprites
- Kira 0.12 audio integration: ambient and event audio

**Acceptance criteria:**
- Time-to-first-run ≤ 5 minutes from browser load for new user
- Metrics dashboard renders all six domain metrics for a running 10,000-citizen simulation at ≥ 30fps
- Replay seek to any tick in 10,000-tick simulation in ≤ 200ms
- Scenario authoring UI validates scenario and displays path-specific error messages
- WCAG 2.1 AA color contrast on all metric displays and state badges

### 12.7 Parpour/Venture Integration Milestone (Month 15 target)

**Goal:** Production-grade API integration between CivLab and Parpour/Venture AI agent platform.

**Deliverables:**
- Versioned HTTP API v1.0 (stable contract)
- Venture adapter: scenario dispatch, result retrieval, branch API
- API schema documentation published
- Integration test suite: 50 automated tests covering Venture agent workflow
- SLA definition: 99.9% uptime, ≤ 30-second scenario result latency
- Changelog and deprecation policy published

**Acceptance criteria:**
- Venture AI agents run ≥ 500 scenarios/hour sustained
- All dispatched scenarios verified deterministic (BLAKE3 hash match on replay)
- Zero API contract breaking changes without versioned migration path

---

## 13. Integration with Parpour/Venture

### 13.1 What Parpour/Venture Is

Parpour is an autonomous AI economic platform. Venture is its AI agent layer: agents that propose, test, and iterate on economic and governance policies. Venture agents use CivLab as their primary simulation backend: before recommending or enacting a policy in a real-world context, the agent runs the policy through a CivLab scenario to evaluate its outcomes across multiple dimensions.

This creates a tight integration requirement: CivLab must be reliable, fast, deterministic, and API-accessible enough to serve as the inner loop of an AI policy search.

### 13.2 Venture Agent Workflow

```
Venture Agent
     |
     v
[Policy Proposal]  →  Translate policy to CivLab scenario parameters
     |
     v
[Scenario Dispatch]  →  POST /v1/scenarios  (civlab HTTP API)
     |
     v
[Wait for result]  →  GET /v1/scenarios/{id}  (poll or webhook)
     |
     v
[Result retrieval]  →  GET /v1/scenarios/{id}/results  (structured JSON)
     |
     v
[Outcome evaluation]  →  Parse metrics; compute policy score
     |
     v
[Iterate]  →  Adjust parameters; dispatch next scenario variant
```

The agent runs this loop N times (N = 10–1,000 per policy search session), comparing outcome metrics across variants to identify the parameter configuration that optimizes the agent's objective function (e.g., maximize legitimacy at 10,000 ticks without entering civil war).

### 13.3 API Contract Between Venture and CivLab

The Venture-CivLab API contract is governed by the following principles:

| Principle | Description |
|-----------|-------------|
| **Versioned and stable** | API is semver-versioned; breaking changes require a new major version; prior version maintained for ≥ 6 months |
| **Typed and schema-validated** | All request/response payloads are schema-validated; schema published as OpenAPI 3.1 document |
| **Determinism guaranteed** | Every scenario dispatched via Venture API includes a seed; result payload includes BLAKE3 final state hash; Venture can verify replay determinism |
| **Idempotent dispatch** | Scenario dispatch is idempotent with client-provided idempotency key; duplicate dispatch with same key returns same result |
| **Structured errors** | All errors return structured JSON with error code, message, and remediation hint |
| **Async by default** | Scenarios are dispatched asynchronously; result is polled or pushed via webhook; no synchronous blocking on long-running simulations |

#### Core API Endpoints (Venture Integration)

```
POST /v1/scenarios
  Body: { scenario: ScenarioDefinition, seed: u64, ticks: u64, mods: [ModRef] }
  Returns: { scenario_id: uuid, status: "QUEUED" }

GET /v1/scenarios/{id}
  Returns: { scenario_id, status: "QUEUED"|"RUNNING"|"COMPLETE"|"ABORTED", progress: { tick, total_ticks } }

GET /v1/scenarios/{id}/results
  Returns: ScenarioResult (full result payload; see Section 5.7)

POST /v1/scenarios/{id}/branch
  Body: { at_tick: u64, intervention: InterventionDefinition }
  Returns: { branch_scenario_id: uuid }
```

### 13.4 Shared Artifact Determinism Requirements

When Venture stores a scenario result for audit, regulatory, or reproducibility purposes, the stored artifact must include sufficient information to reproduce the result independently:

| Artifact Component | Required | Purpose |
|-------------------|----------|---------|
| Scenario definition (TOML) | Yes | Defines initial conditions |
| Seed (u64) | Yes | Controls all randomness |
| CivLab version (semver) | Yes | Ties result to specific engine version |
| Mod manifest (name + version + BLAKE3 hash) | Yes if mods used | Reproducible mod set |
| BLAKE3 final state hash | Yes | Verification of replay fidelity |
| BLAKE3 hash at each tick (hashes.bin) | Optional | Full replay verification |
| Result JSON | Yes | Structured outcome metrics |

A Venture audit package contains all required components. Any third party with `civlab` installed can run `civlab verify --artifact <package>` to independently reproduce and verify the result.

### 13.5 Parpour Business Relationship

The CivLab-Parpour commercial relationship is structured as:

- **Integration API license:** Parpour pays a monthly API license fee for commercial use of the CivLab HTTP API (above free tier TCU allocation).
- **Revenue share on cloud credits:** Parpour/Venture consumes cloud simulation credits; CivLab charges at enterprise rate.
- **Co-development arrangement:** Feature requests from Venture that have general applicability to CivLab's broader user base are implemented in civlab-core (open source) and prioritized in the roadmap. Venture-specific adapter code is Parpour's responsibility.
- **Joint publication:** CivLab and Parpour co-author technical publications demonstrating AI-driven policy analysis on CivLab simulations.

---

## 14. Governance and Decision Framework

### 14.1 Product Priority Decision Authority

| Decision Type | Decision Authority | Process |
|--------------|-------------------|---------|
| Roadmap phase priorities | CivLab Product Lead | Annual planning; quarterly review; published in WORK_STREAM.md |
| Feature scope within phase | CivLab Engineering Lead | Sprint planning; FR tracker update |
| API breaking changes | CivLab Architecture Review | RFC required; ≥ 14-day community comment period |
| Scenario format changes | CivLab Architecture Review | RFC required; migration path required |
| D1-D7 ruleset amendments | CivLab Engineering Lead + Community RFC | Unanimous team sign-off + RFC process |
| Mod type additions | CivLab Product Lead | ADR required; civlab-sdk update |
| Open-source license changes | CivLab Legal + Community RFC | Board approval + community notice |
| Parpour API contract changes | CivLab Product Lead + Parpour | Joint review; semver versioning; deprecation policy |

### 14.2 Spec-First Requirement

All new features in CivLab must be specified before implementation begins. The spec-first requirement means:

1. A Functional Requirement (FR) entry is created in `FUNCTIONAL_REQUIREMENTS.md` with FR-SIM-NNN format.
2. An ADR entry is created for any architectural decision with ADR-NNN format.
3. Acceptance criteria are defined in the FR before any implementation PR is opened.
4. The FR is approved by the CivLab Product Lead before engineering begins.
5. Implementation PRs reference the FR ID in their description.
6. The FR is marked IMPLEMENTED when the feature passes all acceptance criteria tests.

This ensures that the roadmap, code, and tests are always traceable to a specification.

### 14.3 ADR Process

Architecture Decision Records (ADRs) document significant technical decisions, their context, alternatives considered, and rationale.

**When an ADR is required:**
- Choosing a new external dependency (crate, WASM runtime, database)
- Changing the tick loop architecture
- Adding a new domain coupling
- Modifying the scenario format in a potentially breaking way
- Choosing a custom implementation over an existing library (with justification)
- Any change to D1-D7 ruleset or BLAKE3 hash algorithm

**ADR format (ADR.md):**

```markdown
## ADR-NNN: [Short Title]
**Status:** PROPOSED | ACCEPTED | DEPRECATED | SUPERSEDED
**Date:** YYYY-MM-DD
**Deciders:** [names/roles]

### Context
[Why this decision is needed]

### Decision
[What was decided]

### Alternatives Considered
[What else was considered and why not chosen]

### Consequences
[Trade-offs; what becomes easier/harder]

### Validation
[How to verify the decision was correct]
```

### 14.4 Community RFC Process for Major Changes

For changes that affect the public API, scenario format, or D1-D7 ruleset, CivLab uses an RFC (Request for Comments) process:

**RFC Process:**
1. Author opens a GitHub Discussion in the `civlab-rfcs` repository with the RFC template.
2. RFC is open for community comment for a minimum of 14 days (or 30 days for breaking changes).
3. CivLab team synthesizes feedback and posts a disposition: ACCEPTED, REJECTED, or DEFERRED.
4. Accepted RFCs are converted to FR + ADR entries and added to the roadmap.
5. Implementation begins only after RFC acceptance.

**What requires an RFC:**
- Any change to the public CivLab HTTP API (v1+)
- Any change to the scenario TOML/JSON schema
- Any change to the civlab-sdk WASM mod API
- Any change to the D1-D7 determinism ruleset
- Any change to the replay artifact format
- Adding or removing a simulation domain at the top level

**What does NOT require an RFC:**
- Bug fixes that do not change API behavior
- Performance improvements with no API surface change
- Documentation improvements
- New default parameter values (backward compatible)
- Internal refactoring with no external behavior change

### 14.5 Quality Gates

All changes to civlab-core must pass the following gates before merge:

| Gate | Description | Failure Action |
|------|-------------|----------------|
| CI replay test | Two runs with same seed → byte-identical BLAKE3 hashes | Block merge |
| D1-D7 Clippy lints | No violations of D2, D3, D4, D5 rules | Block merge |
| Performance regression | Tick latency ≥ 5% above baseline | Require engineering lead sign-off |
| Test coverage | ≥ 90% unit test coverage for modified modules | Block merge |
| FR traceability | All new code references an FR ID | Block merge |
| API schema validation | All API changes update OpenAPI schema | Block merge |
| WASM build | WASM32 target builds without error | Block merge |

---

## Appendix A: Glossary

| Term | Definition |
|------|------------|
| KiloJoule (kJ) | The atomic unit of CivLab's energy economy; represented as i64 newtype |
| Tick | One discrete simulation step; all domain updates occur within a single tick |
| ChaCha20Rng | The deterministic random number generator used for all simulation randomness |
| BLAKE3 | The cryptographic hash function used to compute per-tick state hashes |
| D1-D7 | The seven determinism rules that govern all simulation code |
| Scenario | A complete description of a simulation's initial conditions (seed + parameters + mods) |
| Replay artifact | The complete recorded output of a simulation run (state snapshots + diffs + events + hashes) |
| Branch | A fork of a simulation at a specific tick, creating two independent forward trajectories |
| Sweep | A batch run of N scenario variants, typically varying one or more parameters |
| Causal trace | A structured DAG representing the event chain that led to an instability event |
| WASM mod | A WebAssembly module loaded into the CivLab simulation sandbox to extend behavior |
| civlab-sdk | The Rust crate providing types, macros, and test harness for WASM mod development |
| TCU (Tick-Compute Unit) | The pricing unit for cloud simulation credits; 1 TCU = 1 simulation tick on reference compute |
| RFC | Request for Comments; the community review process for major CivLab changes |
| ADR | Architecture Decision Record; documents significant technical decisions |
| FR | Functional Requirement; a SHALL statement specifying required system behavior |
| Venture | The AI agent layer of Parpour that uses CivLab as a policy simulation backend |
| Joule Economy | CivLab's energy-as-primary-resource economic subsystem |
| Legitimacy | The population-weighted measure of institutional acceptance; key governance metric |
| Tyranny index | Composite metric of enforcement overreach and political capture |
| Elite capture | The degree to which institutional decisions serve elite rather than general population interests |
| Insurgency | Organized armed resistance to institutional authority; modeled as a state machine |

---

## Appendix B: FR ID Reference

| FR Range | Domain |
|----------|--------|
| CIV-0001–CIV-0099 | Core tick loop, harness, and determinism |
| CIV-0100–CIV-0101 | Joule Economy |
| CIV-0102 | Climate System |
| CIV-0103 | Institutions |
| CIV-0104 | Citizens/Demography |
| CIV-0105 | War/Diplomacy |
| CIV-0106 | Social/Insurgency |
| CIV-0300 | Web RTS Client (Pixi.js + React) |
| CIV-0400 | Desktop Client (Bevy 3D) |
| CIV-0600 | Asset Generation (SDXL) |
| CIV-0601 | AI/NPC Integration |
| CIV-0700 | WASM Mod Platform + civlab-sdk |

---

*This document is the authoritative product model for CivLab's civ-sim platform. It is updated as part of the standard RFC and ADR process. Questions, corrections, and RFC proposals should be submitted via the civlab-rfcs GitHub repository.*


---

## Source: models/civ-sim/TECHNICAL_SPEC.md

# CivLab Civ-Sim Technical Specification

**Version:** 2.0
**Status:** SPECIFICATION
**Date:** 2026-02-21
**Authors:** CIV Architecture & Engine Team
**Replaces:** v1 Scaffold (32-line stub)

---

## Table of Contents

1. [System Architecture Overview](#1-system-architecture-overview)
2. [Crate Manifest — Full Library Decisions](#2-crate-manifest--full-library-decisions)
3. [Crate Structure — Full Workspace Layout](#3-crate-structure--full-workspace-layout)
4. [ECS World Design](#4-ecs-world-design)
5. [Performance Architecture](#5-performance-architecture)
6. [Determinism Architecture](#6-determinism-architecture)
7. [Server Architecture](#7-server-architecture)
8. [Build and CI Configuration](#8-build-and-ci-configuration)
9. [Python Research Bindings](#9-python-research-bindings)
10. [Non-Functional Requirements Table](#10-non-functional-requirements-table)

---

## 1. System Architecture Overview

### 1.1 Executive Summary

CivLab is a **headless, deterministic civilization simulation engine** written in Rust. The core simulation runs as an independent process with no rendering dependencies. Multiple heterogeneous clients — game engines, web browsers, research scripts — attach via WebSocket and receive tick broadcasts. The system is designed for:

- **Deterministic replay:** given `(seed, scenario, event_log)`, every run produces byte-identical output.
- **Research-first:** the Python bindings allow headless batch sweeps without any server infrastructure.
- **Multi-client real-time:** up to N simultaneous game clients observe and command the same running world.

### 1.2 Headless Core + Multi-Client Topology

```
                         ┌─────────────────────────────────────────────────┐
                         │            civ-server (axum + tokio)            │
                         │                                                  │
                         │  ┌──────────────────────────────────────────┐   │
                         │  │          Simulation Thread               │   │
                         │  │  ┌──────────────────────────────────┐   │   │
                         │  │  │        civ-engine                │   │   │
                         │  │  │  (tick loop, ECS World, phases)  │   │   │
                         │  │  └──────────┬───────────────────────┘   │   │
                         │  │             │ tick delta                 │   │
                         │  │  ┌──────────▼───────────────────────┐   │   │
                         │  │  │     Domain Crates                │   │   │
                         │  │  │  economy │ climate │ actors       │   │   │
                         │  │  │  policy  │ geo     │ social       │   │   │
                         │  │  │  war     │ metrics │ replay       │   │   │
                         │  │  └──────────────────────────────────┘   │   │
                         │  └──────────────────────────────────────────┘   │
                         │         │ mpsc::Sender<BroadcastFrame>           │
                         │  ┌──────▼──────────────────────────────────┐   │
                         │  │    Broadcast Hub                         │   │
                         │  │  tokio::broadcast::channel<Arc<Frame>>  │   │
                         │  └──────────┬──────────────────────────────┘   │
                         │             │                                    │
                         │    ┌────────┼────────────────────────────┐      │
                         │    │        │                            │      │
                         │  ┌─▼──┐  ┌─▼──┐  ┌─────┐  ┌──────┐   │      │
                         │  │WS/1│  │WS/2│  │WS/3 │  │WS/N  │   │      │
                         │  └────┘  └────┘  └─────┘  └──────┘   │      │
                         │                                         │      │
                         └─────────────────────────────────────────┘      │
                                          │                                │
           ┌────────────────┬─────────────┴──────────┬────────────────────┘
           │                │                         │
   ┌───────▼──────┐ ┌───────▼──────┐        ┌────────▼──────────┐
   │ Bevy Client  │ │ Unreal/Unity │        │ Python Research   │
   │ (Rust, WS)   │ │ (C++/C#, WS) │        │ (pyo3 FFI, direct)│
   └──────────────┘ └──────────────┘        └───────────────────┘

   ┌──────────────┐ ┌──────────────┐
   │  Web Browser │ │  Godot (GD)  │
   │  (TS, WS)    │ │  (GDScript)  │
   └──────────────┘ └──────────────┘
```

### 1.3 Crate Dependency Graph (DAG)

The inter-crate dependency graph is strictly acyclic. `tach.toml` enforces all boundary rules at compile time and in CI.

```
civ-protocol   (shared types: no simulation logic)
      │
      ├──────────────────────────────────────────────┐
      │                                               │
civ-engine  ←── civ-economy ←── civ-climate          │
      │               │              │               │
      │          civ-actors ◄────────┘               │
      │               │                              │
      │          civ-policy                          │
      │               │                              │
      │          civ-geo                             │
      │               │                              │
      │          civ-social                          │
      │               │                              │
      │          civ-war                             │
      │               │                              │
      └──────► civ-metrics ◄──────────────────────────
                    │
              civ-replay
                    │
              civ-server  (top-level integration crate)
```

**Strict rules enforced via `tach.toml`:**
- `civ-engine` depends on `civ-protocol` only. No upward domain imports.
- Domain crates (`economy`, `climate`, `actors`, `policy`, `geo`, `social`, `war`) depend on `civ-engine` and `civ-protocol`.
- `civ-metrics` aggregates from all domain crates but emits into `civ-protocol` types.
- `civ-replay` depends on `civ-metrics` and `civ-protocol`.
- `civ-server` is the only crate that depends on all others.
- **No cycles permitted.** `cargo deny` and `tach check` both enforce this.

### 1.4 Responsibility Boundaries

| Crate | Owns | Does NOT Own |
|---|---|---|
| `civ-engine` | Tick loop, phase scheduler, ECS world, determinism invariants | Any domain logic |
| `civ-economy` | Market clearing, Joule allocator, ledger, price index | Citizen behavior |
| `civ-climate` | CO2 model, weather events, Monte Carlo disaster sampling | Production math |
| `civ-actors` | Citizen lifecycle, demographics, birth/death rates | Ideology, health |
| `civ-policy` | Policy FSM, effect application, three-tier evaluation | Market clearing |
| `civ-geo` | Terrain, cell grid, pathfinding, LOD aggregation | Social dynamics |
| `civ-social` | Ideology vectors, insurgency, health model, cohesion | Economy |
| `civ-war` | Military units, combat resolution, diplomacy, shadow networks | Market prices |
| `civ-metrics` | Prometheus export, time-series storage, analytics aggregation | Simulation logic |
| `civ-server` | axum router, WebSocket upgrade, broadcast hub, HTTP API | Simulation state |
| `civ-protocol` | Shared types (Event, Command, Snapshot), schemas | Logic of any kind |
| `civ-replay` | `.civreplay` format, record, seek, verify determinism | Server I/O |

---

## 2. Crate Manifest — Full Library Decisions

This section documents every third-party dependency with version pins, rationale, and the alternatives that were rejected.

### 2.1 Cargo.toml — Workspace Dependencies Block

```toml
[workspace.dependencies]

# ---------- ECS ----------
legion = { version = "0.4.0", default-features = false }

# ---------- RNG ----------
rand_chacha = "0.3.1"
rand = { version = "0.8.5", default-features = false, features = ["std_rng"] }

# ---------- Fixed-Point Arithmetic ----------
fixed = "1.23.1"

# ---------- Serialization ----------
serde = { version = "1.0.197", features = ["derive"] }
serde_json = "1.0.114"
rmp-serde = "1.1.2"       # MessagePack for binary frames

# ---------- Async Runtime ----------
tokio = { version = "1.36.0", features = ["full"] }

# ---------- WebSocket / HTTP ----------
axum = { version = "0.7.4", features = ["ws"] }
tokio-tungstenite = "0.21.0"

# ---------- Database ----------
sqlx = { version = "0.7.4", features = ["runtime-tokio-rustls", "postgres", "uuid", "chrono", "json"] }

# ---------- Parallelism ----------
rayon = "1.9.0"

# ---------- Math / Geometry ----------
glam = { version = "0.27.0", features = ["bytemuck"] }

# ---------- Metrics / Observability ----------
prometheus = "0.13.3"
opentelemetry = { version = "0.22.0", features = ["metrics", "trace"] }
opentelemetry-otlp = "0.15.0"
tracing = "0.1.40"
tracing-subscriber = { version = "0.3.18", features = ["env-filter", "json"] }
tracing-opentelemetry = "0.23.0"

# ---------- Compression ----------
zstd = "0.13.0"

# ---------- Hashing ----------
blake3 = "1.5.1"

# ---------- CLI ----------
clap = { version = "4.5.3", features = ["derive"] }

# ---------- Config ----------
config = "0.14.0"
toml = "0.8.12"

# ---------- Testing ----------
proptest = "1.4.0"
criterion = { version = "0.5.1", features = ["html_reports"] }
insta = { version = "1.36.1", features = ["json", "yaml"] }
cargo-nextest = "0.9.68"   # parallel test executor (dev tool, not lib dep)

# ---------- Python Bindings ----------
pyo3 = { version = "0.21.0", features = ["extension-module", "abi3-py310"] }

# ---------- Utilities ----------
uuid = { version = "1.7.0", features = ["v4", "serde"] }
chrono = { version = "0.4.34", features = ["serde"] }
anyhow = "1.0.80"
thiserror = "1.0.57"
bytes = "1.5.0"
bitset-core = "0.1.0"
```

### 2.2 ECS: `legion` 0.4

**Decision:** `legion` (archetype-based ECS)

**Version pin:** `0.4.0`

**Rationale:**
- **Zero-copy archetype queries:** Components are stored in contiguous typed arrays per archetype. Iterating `(Position, Inventory, Mood)` touches exactly the memory for those three component types, in order. No indirection through `Arc<Mutex<>>` or pointer chasing.
- **No `Arc<Mutex<>>` in hot path:** `legion` worlds own component data directly. Parallel system dispatch uses safe Rust borrowing rules at compile time, not runtime locks. This is required by the determinism invariant (no non-deterministic lock ordering).
- **Cache-friendly:** Archetype layout ensures entities sharing the same component set are stored together. Iterating all Citizens touches citizen-only memory; no interleaving of unrelated component data.
- **Pure Rust:** No C dependencies. Compiles cleanly on all tier-1 targets.
- **Serialization support:** Components implement `serde::Serialize + Deserialize`. `legion` worlds can be serialized to canonical form for state hashing and snapshotting.

**Alternatives Rejected:**

| Alternative | Why Rejected |
|---|---|
| `bevy_ecs` (standalone) | Pulls in a large fraction of the Bevy dependency tree even when used headless. The `bevy_ecs` standalone crate is not officially supported as a standalone library — it is maintained as part of Bevy's monorepo and breakage is common when used outside that context. Heavier compile times. |
| `specs` | Uses `Arc<Mutex<MaskedStorage<T>>>` for component storage. Every parallel system that reads components acquires a read lock. Under high parallelism (rayon scope with 8 threads), this creates lock contention on the component storage. Also uses dynamic dispatch for system scheduling, adding runtime overhead. |
| `hecs` | Solid alternative, but lacks first-class support for parallel world access patterns. Schedules are manual; no built-in concept of system phases. Would require significant bespoke scheduling code that `legion` provides out of the box. |
| Custom entity model | Full control, but the correctness burden of a cache-friendly archetype layout is substantial. `legion` has been validated at scale; a custom solution would require the same level of validation. ADR-006 deferred this decision to the P0 prototype but `legion` was selected after benchmarking. |

### 2.3 RNG: `rand_chacha` 0.3 + `rand` 0.8

**Decision:** `rand_chacha::ChaCha20Rng` for all simulation randomness.

**Version pin:** `rand_chacha = "0.3.1"`, `rand = "0.8.5"`

**Rationale:**
- **ChaCha20 algorithm:** Stream cipher with well-studied statistical properties. Passes all PractRand and TestU01 tests. Not a cryptographic commitment — we do not need CSPRNG security — but ChaCha20's regularity ensures no unexpected period collapses or correlation issues at large tick counts.
- **Seedable, portable byte order:** `ChaCha20Rng::seed_from_u64(n)` produces identical output on x86, ARM, and WASM because ChaCha20 is defined in terms of 32-bit little-endian words. This is the critical property: the same seed on any platform produces the same byte stream. `SmallRng`, `StdRng`, and `ThreadRng` do NOT guarantee this.
- **Deterministic per-tick derivation:** Tick seed is derived as `simulation_seed XOR (tick_number * PRIME_A) XOR (phase_id * PRIME_B)`. This ensures each phase within each tick gets a unique but reproducible RNG stream, preventing phase order from affecting RNG state.
- **No `ThreadRng`:** `ThreadRng` is seeded from OS entropy. Any call to `rand::random()` or `ThreadRng::new()` in simulation code is a determinism violation caught by the custom clippy lint `clippy::float_arithmetic` combined with a `#[forbid(unsafe_code)]` policy.

**Seed Derivation:**
```rust
const PRIME_A: u64 = 0x9e37_79b9_7f4a_7c15;
const PRIME_B: u64 = 0x6c62_272e_07bb_0142;

pub fn tick_phase_seed(sim_seed: u64, tick: u64, phase: PhaseId) -> u64 {
    sim_seed
        .wrapping_mul(PRIME_A)
        .wrapping_add(tick.wrapping_mul(PRIME_A))
        .wrapping_add(phase as u64 * PRIME_B)
}

pub fn make_rng(sim_seed: u64, tick: u64, phase: PhaseId) -> ChaCha20Rng {
    ChaCha20Rng::seed_from_u64(tick_phase_seed(sim_seed, tick, phase))
}
```

### 2.4 Fixed-Point Arithmetic: `fixed` 1.23 + manual `i64 × SCALE`

**Decision:** Two-tier approach. Use the `fixed` crate (`I32F32`, `I16F16` types) for physical quantities needing fractional precision. Use manual `i64 × SCALE` (SCALE = 1_000, representing milli-units) for economic quantities where conservation must be exactly verifiable.

**Version pin:** `fixed = "1.23.1"`

**No-Float Rule:**

All simulation logic — production output, market prices, resource quantities, population counts, ideological drift — must use integer or fixed-point types. The `f32`/`f64` types are forbidden in all crates except `civ-metrics` (for Prometheus gauge export) and `civ-geo` (for SIMD spatial queries via `glam`, where only rendering-adjacent position interpolation uses float).

**Rationale:**
- **Platform determinism:** IEEE 754 floating-point arithmetic is not deterministic across platforms. The x87 FPU uses 80-bit extended precision internally; the SSE2 path uses 64-bit. ARM Cortex-A uses a different rounding mode for `fma`. A value computed as `f64` on x86 will differ at the last bit on ARM. Over thousands of ticks, these differences compound.
- **Exact conservation:** Joule economy requires `sum(allocated_joules) <= total_available_joules` exactly. With floats, rounding in the allocation loop can silently violate this invariant. With `i64` scaled integers, the invariant is `sum(allocated) <= total` checked with integer comparison.
- **Audit trail:** Fixed-point amounts can be logged, replayed, and compared exactly. A float audit trail has fuzzy equality; a fixed-point trail has exact equality.

**Type Mapping:**
```rust
// Economic quantities (milli-units, SCALE = 1_000)
type Grain   = i64;   // 1 unit = 1_000 grain-millis
type Joules  = i64;   // 1 joule = 1_000 joule-millis
type Credits = i64;   // 1 credit = 1_000 milli-credits (equivalent to "cents")

// Physical / ratio quantities (fixed-point via `fixed` crate)
use fixed::types::{I32F32, I16F16};
type FertilityRatio = I16F16;  // 0.0 to 1.0 land fertility
type IdeologyScore  = I16F16;  // -1.0 to +1.0

// Geospatial (integer grid only)
type CellX = i32;
type CellY = i32;
```

**Clippy Enforcement:**
```toml
# .cargo/config.toml in each simulation crate
[target.'cfg(all())'.rustflags]
rustflags = [
    "-D", "clippy::float_arithmetic",  # forbids f32/f64 arithmetic ops in sim crates
]
```

### 2.5 Serialization: `serde` 1.0 + `serde_json` 1.0 + `rmp-serde` 1.1

**Decision:** `serde` derive macros on all domain types. JSON for debug/research/WebSocket protocol. MessagePack (via `rmp-serde`) for binary WebSocket frames.

**Version pins:** `serde = "1.0.197"`, `serde_json = "1.0.114"`, `rmp-serde = "1.1.2"`

**Rationale:**
- **`serde`:** Zero-cost abstractions via proc-macro derive. No runtime reflection. Compile-time schema checked. The `#[serde(deny_unknown_fields)]` attribute on all protocol types ensures forward-compatibility breakage is explicit.
- **`serde_json`:** Human-readable, debuggable, compatible with every client language. Used for JSON-RPC 2.0 messages on the WebSocket transport. Not used for snapshot storage in production (MessagePack is ~3x smaller).
- **`rmp-serde`:** MessagePack is a compact binary encoding of the same serde data model. A `Snapshot` that serializes to 120 KB JSON serializes to ~35 KB MessagePack. For 60 FPS clients receiving snapshots at every tick, this is a 3.4x bandwidth reduction. `rmp-serde` shares the same `serde::Serialize` derive — no separate schema definition.
- **NOT protobuf/flatbuffers:** Protocol Buffers require a separate `.proto` schema and generated code. Changes to domain types require updating both the Rust struct and the proto definition. With `serde`, the Rust type IS the schema. FlatBuffers have zero-copy reads but require manual offset handling; the snapshot types in CivLab are read-and-process, not zero-copy accessed, so FlatBuffers' main advantage does not apply.

### 2.6 Async Runtime: `tokio` 1.36

**Decision:** `tokio` 1.x with the multi-threaded scheduler for server I/O; the simulation tick loop runs on a dedicated `std::thread`, not inside tokio.

**Version pin:** `tokio = "1.36.0"`

**Rationale:**
- **Multi-thread scheduler for server:** axum, tokio-tungstenite, and sqlx all require tokio. The multi-thread scheduler dispatches I/O tasks across all CPU cores. At 100 concurrent WebSocket clients, the I/O workload is highly concurrent; the single-thread scheduler would serialize frame dispatches and increase latency.
- **Dedicated thread for simulation:** The simulation tick loop is CPU-bound and must not be preempted by async executor task switching. Running `Simulation::tick()` inside a tokio task would make it a blocking task that stalls the executor during the 8 ms deterministic transition phase. The correct pattern is `std::thread::spawn` for the sim loop, with `tokio::sync::mpsc::channel` bridging the sim thread to the async broadcast hub. This is the standard tokio architecture for CPU-bound background workers.
- **`tokio::time::sleep`:** The server uses `tokio::time::interval` for the 100 ms tick pacing clock. Actual sleep is on the sim thread via `std::thread::sleep` (or spin-loop for sub-millisecond precision).

**Thread Architecture:**
```
Main thread (tokio runtime, multi-thread scheduler)
    │
    ├── axum HTTP/WebSocket handler tasks (async, tokio)
    ├── broadcast receiver tasks per client (async, tokio)
    └── mpsc::Receiver task (receives from sim thread)

Simulation thread (std::thread, blocking)
    └── sim_loop():
        loop {
            let frame = engine.tick();
            tx.send(frame)?;       // mpsc::Sender (non-blocking)
            sleep_until_next_tick();
        }
```

### 2.7 WebSocket / HTTP: `axum` 0.7 + `tokio-tungstenite` 0.21

**Decision:** `axum` for the HTTP server and WebSocket upgrade handler. `tokio-tungstenite` for the underlying WebSocket protocol implementation.

**Version pins:** `axum = "0.7.4"`, `tokio-tungstenite = "0.21.0"`

**Rationale:**
- **`axum`:** Tower-based, composable HTTP framework. Type-safe extractors mean malformed requests fail at the type level, not at runtime. First-class WebSocket upgrade support via `axum::extract::ws`. Middleware (tracing, metrics, auth) composes cleanly as Tower layers.
- **`tokio-tungstenite`:** The reference WebSocket implementation for tokio. Used by axum's WS handler internally. Direct use gives access to `Message::Binary` for sending MessagePack frames without re-encoding through axum's higher-level API.
- **NOT warp:** warp has a complex type-level filter system that produces confusing error messages. axum's extractor model is more ergonomic and better documented.
- **NOT actix-web:** actix-web uses a different actor-model threading approach that does not compose cleanly with tokio's multi-thread scheduler. Migration complexity is high.

### 2.8 Database: `sqlx` 0.7

**Decision:** `sqlx` with the PostgreSQL driver and compile-time query verification.

**Version pin:** `sqlx = "0.7.4"`

**Rationale:**
- **Compile-time query verification:** `sqlx::query!` macros connect to the database at compile time and verify SQL syntax, column names, and return types. Type mismatches between the query and the Rust struct are compile errors, not runtime panics. This is the primary reason for choosing `sqlx` over `diesel` (which uses a DSL) or `sea-orm` (which generates queries at runtime).
- **Async-native:** `sqlx` is built on tokio. No blocking database calls on the async executor.
- **PostgreSQL driver:** Production deployments use PostgreSQL. `sqlx`'s PostgreSQL driver supports `LISTEN/NOTIFY` for real-time event feeds, JSONB columns for schema-flexible snapshot storage, and `uuid` / `chrono` type mapping.
- **Used for:** Run metadata (scenario, seed, start time, status), snapshot archive (compressed binary blobs), event log archive, metrics time-series storage.

### 2.9 Parallelism: `rayon` 1.9

**Decision:** `rayon` for data-parallel tick phases.

**Version pin:** `rayon = "1.9.0"`

**Rationale:**
- **Work-stealing thread pool:** `rayon` maintains a pool of threads equal to the number of logical CPUs. `par_iter()` distributes work across all threads with work-stealing. For the demographics phase (iterate all citizens, apply birth/death/aging), `rayon` achieves near-linear scaling up to the CPU count.
- **Safe parallel iterators:** `rayon`'s `par_iter()` enforces Rust's borrow rules. Two parallel tasks cannot mutate the same component. This eliminates data races at compile time.
- **Scoped parallelism:** Within a tick phase, `rayon::scope` allows structured concurrency. The scope ends before the next phase begins, guaranteeing phase isolation.
- **Phases that use `rayon`:** Demographics (citizen birth/death/age), Production (per-building output), Climate event sampling (per-cell Monte Carlo), Ideology drift (per-citizen vector dot product). Phases that mutate shared market state (Trade, Command Intake) run single-threaded to preserve deterministic ordering.

**What `rayon` does NOT cover:** Cross-phase parallelism. Phases execute sequentially; within a phase, entities are processed in parallel. This is the correct tradeoff: parallel processing within a phase does not change the deterministic output because the output of each entity's computation depends only on immutable input from the previous phase.

### 2.10 Math / Geometry: `glam` 0.27

**Decision:** `glam` for spatial math (cell positions, unit vectors, pathfinding heuristics).

**Version pin:** `glam = "0.27.0"`

**Rationale:**
- **SIMD-accelerated:** `glam` uses SIMD intrinsics (SSE2/NEON) for `Vec2`, `Vec3`, `Vec4`, and `Mat4` operations. Spatial queries (nearest-neighbor, range queries) that iterate thousands of cell positions benefit from SIMD dot products and comparisons.
- **`bytemuck` feature:** The `bytemuck` feature enables zero-copy casting between `glam` types and raw byte slices. Used when serializing cell position arrays to the binary frame format.
- **Float usage in `civ-geo` only:** `glam` uses `f32`. This is acceptable in `civ-geo` because spatial computations (pathfinding, rendering-adjacent LOD) are not subject to the economic determinism requirement. Cell coordinates are integer (`i32, i32`); `glam` is used only for intermediate geometric calculations (e.g., distance heuristics in A* pathfinding) where exact integer results are read back after rounding. The clippy float-arithmetic deny is scoped to simulation-core crates, not `civ-geo`.

### 2.11 Metrics / Observability: `prometheus` 0.13 + `opentelemetry` 0.22 + `tracing` 0.1

**Decision:** Three-layer observability stack.

**Version pins:**
- `prometheus = "0.13.3"`
- `opentelemetry = "0.22.0"`
- `opentelemetry-otlp = "0.15.0"`
- `tracing = "0.1.40"`
- `tracing-subscriber = "0.3.18"`
- `tracing-opentelemetry = "0.23.0"`

**Layer Responsibilities:**

| Layer | Library | Responsibility |
|---|---|---|
| Structured logging | `tracing` + `tracing-subscriber` | Span-based event logging per tick phase; JSON output for log aggregation |
| Trace propagation | `tracing-opentelemetry` | Propagates trace context from HTTP request through sim phases |
| Metrics export | `prometheus` | Per-tick Prometheus counters and histograms; scraped by Prometheus server |
| Distributed tracing | `opentelemetry-otlp` | Exports spans to Jaeger/Tempo via OTLP gRPC; used for latency profiling |

**Key Prometheus Metrics:**
```rust
// Defined in civ-metrics/src/prometheus_registry.rs

pub static TICK_DURATION_HISTOGRAM: Lazy<Histogram> = Lazy::new(|| {
    register_histogram!(
        "civlab_tick_duration_seconds",
        "Wall-clock duration of one simulation tick",
        vec![0.001, 0.002, 0.005, 0.008, 0.010, 0.016, 0.020, 0.050, 0.100, 0.200]
    ).unwrap()
});

pub static TICK_PHASE_DURATION: Lazy<HistogramVec> = Lazy::new(|| {
    register_histogram_vec!(
        "civlab_tick_phase_duration_seconds",
        "Wall-clock duration per tick phase",
        &["phase"],
        vec![0.0001, 0.0005, 0.001, 0.002, 0.005, 0.008, 0.016]
    ).unwrap()
});

pub static CITIZEN_COUNT: Lazy<IntGauge> = Lazy::new(|| {
    register_int_gauge!("civlab_citizen_count", "Total live citizen count").unwrap()
});

pub static BROADCAST_LAG_HISTOGRAM: Lazy<Histogram> = Lazy::new(|| {
    register_histogram!(
        "civlab_broadcast_lag_seconds",
        "Lag from tick completion to last client frame delivery",
        vec![0.0001, 0.001, 0.005, 0.010, 0.020, 0.050]
    ).unwrap()
});
```

### 2.12 Compression: `zstd` 0.13

**Decision:** `zstd` for snapshot compression in binary frames and `.civreplay` file compression.

**Version pin:** `zstd = "0.13.0"`

**Rationale:**
- **Speed-to-ratio tradeoff:** At compression level 3 (default), `zstd` achieves ~3-5x compression ratio on JSON snapshot data at >500 MB/s throughput. `gzip` achieves similar ratios at ~100 MB/s. For a 120 KB JSON snapshot compressed per tick at 10 ticks/second, `zstd` at 500 MB/s adds ~0.24 ms per tick. `gzip` would add ~1.2 ms.
- **`.civreplay` format:** Each event in the event log is individually `zstd`-compressed at level 1 (fastest). The full file header and initial state are compressed at level 9 (best ratio) since compression happens once at export time.
- **Binary frame transport:** `zstd` level 1 is applied to snapshot payloads in binary WebSocket frames when the `use_binary_frames` flag is set and `compressed` flag bit is enabled.

### 2.13 Hashing: `blake3` 1.5

**Decision:** `blake3` for state hashing (determinism verification) and event state-hash contracts.

**Version pin:** `blake3 = "1.5.1"`

**Rationale:**
- **Speed:** `blake3` is the fastest general-purpose cryptographic hash function. On AVX-512 hardware it exceeds 10 GB/s. Hashing a 120 KB serialized world state at 10 ticks/second adds less than 0.12 ms per tick.
- **SIMD native:** `blake3` uses SIMD intrinsics automatically via the `blake3` crate's build script. No manual feature flags required.
- **Canonical form input:** The state hash is computed over the canonical CBOR serialization of the full ECS world. CBOR is used (not JSON) because CBOR has a defined canonical binary encoding (no whitespace variation, deterministic key ordering in maps). The `ciborium` crate provides canonical CBOR.
- **Why not SHA-256:** SHA-256 is ~3x slower than `blake3` at the same security level. For per-tick hashing, this matters. SHA-256 provides no correctness benefit over `blake3` for this use case.

### 2.14 CLI: `clap` 4.5

**Decision:** `clap` 4 with derive API for the `civ-server` binary arguments.

**Version pin:** `clap = "4.5.3"`

**Server Binary CLI:**
```
civ-server [OPTIONS]

Options:
  --port <PORT>          WebSocket port [default: 9876]
  --seed <SEED>          Simulation seed [default: random u64]
  --scenario <PATH>      Scenario TOML file path
  --tick-rate <RATE>     Ticks per second [default: 10]
  --max-clients <N>      Maximum concurrent clients [default: 100]
  --db-url <URL>         PostgreSQL connection string
  --metrics-port <PORT>  Prometheus scrape port [default: 9090]
  --log-level <LEVEL>    Log level (trace|debug|info|warn|error) [default: info]
  --headless             Disable WebSocket server (research mode)
```

### 2.15 Config: `config` 0.14 + TOML

**Decision:** `config` crate with TOML file format for simulation configuration.

**Version pins:** `config = "0.14.0"`, `toml = "0.8.12"`

**Config layering:**
1. Default values (compiled in)
2. `config/default.toml`
3. `config/{environment}.toml` (dev, staging, prod)
4. Environment variables (`CIV_PORT`, `CIV_SEED`, etc.)
5. CLI flags (highest priority)

### 2.16 Testing: `proptest` 1.4 + `criterion` 0.5 + `insta` 1.36

**Decision:** Three-library testing stack.

**Version pins:**
- `proptest = "1.4.0"`
- `criterion = "0.5.1"`
- `insta = "1.36.1"`

**Responsibilities:**

| Library | Use Case | Example |
|---|---|---|
| `proptest` | Property-based tests on invariants | "For any seed, `sum(allocated_joules) <= total_joules`" |
| `criterion` | Microbenchmarks per tick phase | `engine::tick()` with 1k / 10k / 100k citizens |
| `insta` | Snapshot tests of serialized output | Verify `Snapshot::from_state()` produces exact known JSON |

**`proptest` usage:**
```rust
use proptest::prelude::*;

proptest! {
    #[test]
    fn joule_conservation_holds(
        available in 0i64..=1_000_000_000,
        n_actors in 1usize..=10_000,
        seed in any::<u64>(),
    ) {
        let actors: Vec<_> = (0..n_actors).map(|i| actor_with_demand(i as i64)).collect();
        let allocations = JouleAllocator::allocate(available, &actors, seed);
        let total_allocated: i64 = allocations.iter().sum();
        prop_assert!(total_allocated <= available,
            "conservation violated: {} > {}", total_allocated, available);
    }
}
```

---

## 3. Crate Structure — Full Workspace Layout

### 3.1 Directory Tree

```
civ/
├── Cargo.toml                    (workspace manifest)
├── Cargo.lock                    (committed, deterministic builds)
├── tach.toml                     (boundary enforcement)
├── .cargo/
│   └── config.toml               (target-wide rustflags, clippy lints)
├── crates/
│   ├── civ-engine/               (tick loop, ECS world, determinism)
│   │   ├── Cargo.toml
│   │   ├── src/
│   │   │   ├── lib.rs
│   │   │   ├── simulation.rs     (Simulation struct, tick() entry point)
│   │   │   ├── phase.rs          (PhaseId enum, phase scheduler)
│   │   │   ├── world.rs          (ECS World wrapper, entity management)
│   │   │   ├── snapshot.rs       (world → Snapshot serialization)
│   │   │   ├── command.rs        (Command intake, priority queue)
│   │   │   └── hash.rs           (BLAKE3 state hash computation)
│   │   └── tests/
│   │       ├── fr_core_tick_loop.rs
│   │       ├── fr_determinism_replay.rs
│   │       ├── fr_rng_seeding.rs
│   │       └── fr_phase_schedule.rs
│   │
│   ├── civ-economy/              (market, Joule allocator, ledger)
│   │   ├── Cargo.toml
│   │   ├── src/
│   │   │   ├── lib.rs
│   │   │   ├── allocator.rs      (Allocator trait + three impls)
│   │   │   ├── market.rs         (MarketAllocator: price discovery)
│   │   │   ├── plan.rs           (PlanAllocator: central planner)
│   │   │   ├── joule.rs          (JouleAllocator: energy model)
│   │   │   ├── ledger.rs         (double-entry accounting, i64)
│   │   │   └── price_index.rs    (inflation tracking, BTreeMap)
│   │   └── tests/
│   │       ├── fr_econ_market.rs
│   │       ├── fr_econ_joule.rs
│   │       └── fr_econ_properties.rs  (proptest invariants)
│   │
│   ├── civ-climate/              (CO2 model, disasters, Monte Carlo)
│   │   ├── Cargo.toml
│   │   ├── src/
│   │   │   ├── lib.rs
│   │   │   ├── co2.rs            (CO2 concentration model, i64 ppm-millis)
│   │   │   ├── weather.rs        (temperature, rainfall per cell)
│   │   │   ├── disaster.rs       (flood, drought, storm sampling)
│   │   │   └── monte_carlo.rs    (per-cell seeded event sampling)
│   │   └── tests/
│   │
│   ├── civ-actors/               (citizen lifecycle, demographics)
│   │   ├── Cargo.toml
│   │   ├── src/
│   │   │   ├── lib.rs
│   │   │   ├── citizen.rs        (Citizen ECS components, lifecycle FSM)
│   │   │   ├── demographics.rs   (birth/death rates, migration)
│   │   │   ├── employment.rs     (job assignment, unemployment)
│   │   │   └── military.rs       (MilitaryUnit, combat_strength)
│   │   └── tests/
│   │
│   ├── civ-policy/               (policy FSM, effect application)
│   │   ├── Cargo.toml
│   │   ├── src/
│   │   │   ├── lib.rs
│   │   │   ├── policy.rs         (Policy trait, evaluation pipeline)
│   │   │   ├── tiers.rs          (baseline → constrained → optimized)
│   │   │   ├── diplomacy.rs      (DiplomaticRelation, sentiment)
│   │   │   └── shadow_networks.rs(ShadowNetwork, covert actions)
│   │   └── tests/
│   │
│   ├── civ-geo/                  (terrain, pathfinding, LOD)
│   │   ├── Cargo.toml
│   │   ├── src/
│   │   │   ├── lib.rs
│   │   │   ├── grid.rs           (CellGrid, i32 coordinates)
│   │   │   ├── terrain.rs        (TerrainType, fertility, elevation)
│   │   │   ├── pathfinding.rs    (A* with glam SIMD heuristics)
│   │   │   └── lod.rs            (LOD aggregation: cell → district → region)
│   │   └── tests/
│   │
│   ├── civ-social/               (ideology, insurgency, health)
│   │   ├── Cargo.toml
│   │   ├── src/
│   │   │   ├── lib.rs
│   │   │   ├── ideology.rs       (I16F16 ideology vectors, drift model)
│   │   │   ├── insurgency.rs     (grievance accumulation, rebellion risk)
│   │   │   ├── health.rs         (disease model, mortality)
│   │   │   └── cohesion.rs       (social cohesion metrics, Gini)
│   │   └── tests/
│   │
│   ├── civ-war/                  (military, diplomacy, shadow networks)
│   │   ├── Cargo.toml
│   │   ├── src/
│   │   │   ├── lib.rs
│   │   │   ├── combat.rs         (resolve_combat, BattleResult)
│   │   │   ├── attrition.rs      (supply lines, fatigue, morale)
│   │   │   └── occupation.rs     (territory control, resistance)
│   │   └── tests/
│   │
│   ├── civ-metrics/              (prometheus, analytics export)
│   │   ├── Cargo.toml
│   │   ├── src/
│   │   │   ├── lib.rs
│   │   │   ├── registry.rs       (Prometheus metric definitions)
│   │   │   ├── timeseries.rs     (TimeSeries<T>, append, query_range)
│   │   │   ├── snapshot.rs       (MetricsSnapshot per tick)
│   │   │   ├── export.rs         (CSV, Parquet, JSONL export)
│   │   │   └── aggregates.rs     (GDP, Gini, HDI derivation)
│   │   └── tests/
│   │
│   ├── civ-server/               (axum HTTP + WebSocket server)
│   │   ├── Cargo.toml
│   │   ├── src/
│   │   │   ├── main.rs           (binary entry point, clap CLI)
│   │   │   ├── router.rs         (axum Router, middleware stack)
│   │   │   ├── ws_handler.rs     (WebSocket upgrade, session lifecycle)
│   │   │   ├── broadcast.rs      (tokio::broadcast::channel hub)
│   │   │   ├── session.rs        (ClientSession, priority, filter)
│   │   │   └── sim_bridge.rs     (mpsc bridge: sim thread → async hub)
│   │   └── tests/
│   │       ├── fr_websocket_server.rs
│   │       └── fr_server_engine.rs
│   │
│   ├── civ-protocol/             (shared types: events, commands, schemas)
│   │   ├── Cargo.toml
│   │   ├── src/
│   │   │   ├── lib.rs
│   │   │   ├── types.rs          (EntityId, Cell, Tick, GoodId)
│   │   │   ├── events.rs         (SimEvent enum, 50+ variants)
│   │   │   ├── commands.rs       (Command enum, action types)
│   │   │   ├── snapshot.rs       (Snapshot, SnapshotHeader)
│   │   │   └── frame.rs          (BinaryFrame, TickBroadcast)
│   │   └── tests/
│   │
│   └── civ-replay/               (.civreplay format, seek, record)
│       ├── Cargo.toml
│       ├── src/
│       │   ├── lib.rs
│       │   ├── format.rs         (ReplayFile, magic bytes, header)
│       │   ├── writer.rs         (ReplayWriter, append events)
│       │   ├── reader.rs         (ReplayReader, seek by tick)
│       │   └── verifier.rs       (verify determinism via BLAKE3)
│       └── tests/
```

### 3.2 Per-Crate Public API Surfaces

#### `civ-engine`

```rust
// Public API surface of civ-engine

pub struct Simulation {
    pub tick: u64,
    pub seed: u64,
    world: legion::World,
    resources: legion::Resources,
    schedule: PhaseSchedule,
}

impl Simulation {
    pub fn new(seed: u64, scenario: &Scenario) -> Result<Self>;
    pub fn tick(&mut self) -> Result<TickOutput>;
    pub fn snapshot(&self) -> Result<Snapshot>;
    pub fn state_hash(&self) -> [u8; 32];
    pub fn apply_command(&mut self, cmd: Command) -> Result<CommandResult>;
}

pub struct TickOutput {
    pub tick: u64,
    pub snapshot: Snapshot,
    pub events: Vec<SimEvent>,
    pub metrics: MetricsSnapshot,
    pub state_hash: [u8; 32],
    pub phase_durations: BTreeMap<PhaseId, Duration>,
}

pub trait Phase: Send + Sync {
    fn id(&self) -> PhaseId;
    fn run(&self, world: &mut legion::World, resources: &mut legion::Resources) -> Result<Vec<SimEvent>>;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum PhaseId {
    CommandIntake   = 0,
    Policy          = 1,
    Demographics    = 2,
    Production      = 3,
    Trade           = 4,
    Climate         = 5,
    War             = 6,
    Social          = 7,
    Metrics         = 8,
    Snapshot        = 9,
}
```

#### `civ-economy`

```rust
pub trait Allocator: Send + Sync {
    fn allocate(
        &self,
        available: i64,
        demands: &[AllocationDemand],
        rng: &mut ChaCha20Rng,
    ) -> AllocationResult;

    fn verify_conservation(&self, result: &AllocationResult, available: i64) -> Result<()>;
}

pub struct MarketAllocator { /* price discovery via supply/demand */ }
pub struct PlanAllocator   { /* central quota assignment */ }
pub struct JouleAllocator  { /* work-capacity proportional allocation */ }

pub struct Ledger {
    entries: BTreeMap<EntityId, i64>,  // credit balances
}

impl Ledger {
    pub fn transfer(&mut self, from: EntityId, to: EntityId, amount: i64) -> Result<()>;
    pub fn balance(&self, entity: EntityId) -> i64;
    pub fn total_supply(&self) -> i64;  // conservation check
}
```

#### `civ-server`

```rust
pub struct SimServer {
    router: axum::Router,
    broadcast_tx: tokio::sync::broadcast::Sender<Arc<BroadcastFrame>>,
    command_tx: tokio::sync::mpsc::Sender<Command>,
}

impl SimServer {
    pub async fn serve(self, addr: SocketAddr) -> Result<()>;
}

// WebSocket handler (per-client)
async fn ws_handler(
    ws: WebSocketUpgrade,
    State(state): State<ServerState>,
) -> impl IntoResponse;

pub struct ClientSession {
    pub session_id: Uuid,
    pub client_type: ClientType,
    pub priority: u32,
    pub filter: SnapshotFilter,
    pub use_binary_frames: bool,
}
```

#### `civ-protocol`

```rust
// All types in this crate derive Serialize + Deserialize

pub type Tick = u64;
pub type EntityId = u64;
pub type CellX = i32;
pub type CellY = i32;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct Snapshot {
    pub header: SnapshotHeader,
    pub cells: Vec<CellSnapshot>,
    pub agents: Vec<AgentSnapshot>,
    pub institutions: Vec<InstitutionSnapshot>,
    pub markets: Vec<MarketSnapshot>,
    pub events: Vec<SimEvent>,
    pub metrics: MetricsSnapshot,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct BinaryFrame {
    pub tick: u32,
    pub event_count: u32,
    pub snapshot_size: u32,
    pub flags: u32,
    pub payload: Vec<u8>,  // zstd-compressed rmp-serde encoded Snapshot
}
```

### 3.3 Dependency Direction Summary (DAG)

```
civ-protocol      (no sim deps)
    ↑
civ-engine        (depends on: civ-protocol)
    ↑
civ-economy       (depends on: civ-engine, civ-protocol)
civ-climate       (depends on: civ-engine, civ-protocol)
civ-actors        (depends on: civ-engine, civ-economy, civ-protocol)
civ-policy        (depends on: civ-engine, civ-actors, civ-protocol)
civ-geo           (depends on: civ-engine, civ-protocol)
civ-social        (depends on: civ-engine, civ-actors, civ-protocol)
civ-war           (depends on: civ-engine, civ-actors, civ-policy, civ-protocol)
    ↑
civ-metrics       (depends on: all domain crates, civ-protocol)
    ↑
civ-replay        (depends on: civ-metrics, civ-protocol)
    ↑
civ-server        (depends on: all crates above)
```

**Tach enforcement (`tach.toml` excerpt):**
```toml
[[modules]]
path = "crates/civ-engine"
depends_on = ["crates/civ-protocol"]

[[modules]]
path = "crates/civ-economy"
depends_on = ["crates/civ-engine", "crates/civ-protocol"]

[[modules]]
path = "crates/civ-server"
depends_on = ["*"]  # server is the integration top
```

---

## 4. ECS World Design

### 4.1 Archetype Layout

`legion` organizes entities by **archetype** — the set of components they carry. All entities with the same component set are stored together in contiguous memory.

**Primary Archetypes:**

| Archetype Name | Components | Entity Count (typical) | Mutation Frequency |
|---|---|---|---|
| `Citizen` | Position, Inventory, Mood, Health, Employment, Age | 1k – 100k | Every tick (mood, health) |
| `Building` | Position, BuildingRole, Inventory, ProductionState | 100 – 10k | Every tick (production) |
| `Cell` | CellPos, TerrainType, Fertility, ClimateState | 10k – 1M | Per climate tick (~10 ticks) |
| `Institution` | InstRole, Treasury, PolicyBundle, Legitimacy | 1 – 1k | Per policy tick (~10 ticks) |
| `Market` | MarketKey, PriceHistory, OrderBook | 10 – 1k | Every tick (clearing) |
| `MilitaryUnit` | Position, UnitType, Strength, Morale, Fatigue, Faction | 0 – 10k | Per war tick |

**Component Definitions:**
```rust
// civ-protocol/src/types.rs

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Position { pub x: i32, pub y: i32 }

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Inventory {
    pub grain:  i64,  // milli-units (SCALE=1_000)
    pub labor:  i64,  // person-hour-millis
    pub energy: i64,  // joule-millis
    pub wood:   i64,
    pub metal:  i64,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Mood {
    pub happiness:   i16,  // -1000 to +1000 (fixed-point × 10)
    pub legitimacy:  i16,  // 0 to 1000
    pub grievance:   i16,  // 0 to 1000 (rebellion risk proxy)
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Health {
    pub vitality:  i16,  // 0 to 1000
    pub disease:   i16,  // 0 = healthy, 1000 = terminal
    pub stress:    i16,  // 0 to 1000 (chronic stress model)
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Age {
    pub ticks_lived: u64,
    pub life_stage: LifeStage,  // Child, Adult, Elder
}

// Ideology stored in civ-social (separate archetype component)
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Ideology {
    pub auth_lib:    i16,  // I16F16 stored as i16 (×32768 scale), -1.0 to +1.0
    pub collectivism: i16,
    pub tradition:   i16,
    pub militarism:  i16,
}
```

### 4.2 System Execution Order (Phase Schedule)

The simulation executes phases in a fixed sequential order. Within each phase, `rayon` may parallelize entity iteration, but phases do not overlap.

```
Tick N
│
├─ Phase 0: CommandIntake        [50 µs budget]
│    Systems: intake_commands, validate_commands, build_control_vector
│    Parallelism: none (single-threaded, ordered by client priority)
│    Output: Control struct (immutable for remaining phases)
│
├─ Phase 1: Policy               [2 ms budget]
│    Systems: evaluate_policies, constrain_policies, apply_fiscal_controls
│    Parallelism: none (policy FSM is stateful, sequential)
│    Output: PolicyEffect list
│
├─ Phase 2: Demographics         [1 ms budget]
│    Systems: age_citizens, apply_birth_rate, apply_death_rate, migrate_citizens
│    Parallelism: rayon par_iter over all Citizen entities
│    Output: new/dead citizen entity mutations queued (applied after phase)
│
├─ Phase 3: Production           [3 ms budget]
│    Systems: tick_buildings, compute_output, update_inventories
│    Parallelism: rayon par_iter over all Building entities (independent)
│    Output: inventory mutations queued
│
├─ Phase 4: Trade                [2 ms budget]
│    Systems: clear_markets, route_goods, apply_allocations
│    Parallelism: none (market clearing is globally stateful)
│    Output: trade events, price updates
│
├─ Phase 5: Climate              [1 ms budget]
│    Systems: tick_co2, compute_weather, sample_disasters
│    Parallelism: rayon par_iter over Cell entities for weather
│    Output: climate state mutations, disaster events
│
├─ Phase 6: War                  [2 ms budget]
│    Systems: resolve_combat, apply_casualties, update_occupation
│    Parallelism: rayon par_iter over independent battle instances
│    Output: combat result events, morale mutations
│
├─ Phase 7: Social               [1.5 ms budget]
│    Systems: drift_ideology, compute_cohesion, update_insurgency, tick_health
│    Parallelism: rayon par_iter over Citizen ideology and health
│    Output: ideology drift mutations, insurgency risk update
│
├─ Phase 8: Metrics              [0.8 ms budget]
│    Systems: aggregate_gdp, compute_gini, compute_hdi, emit_prometheus
│    Parallelism: none (aggregation is a fold, inherently sequential)
│    Output: MetricsSnapshot
│
└─ Phase 9: Snapshot + Broadcast [50 µs budget]
     Systems: serialize_snapshot, compute_state_hash, enqueue_broadcast
     Parallelism: none (serialization is a single-pass walk)
     Output: BroadcastFrame sent to mpsc channel → server broadcast hub

Total budget: ~14 ms at 1,000 citizens
```

### 4.3 Parallel System Scheduling (rayon Scope Per Phase)

Phases that use `rayon` create a scoped parallel region:

```rust
// civ-engine/src/phase.rs

pub fn run_demographics_phase(
    world: &mut legion::World,
    resources: &mut legion::Resources,
    rng_seed: u64,
    tick: u64,
) -> Vec<SimEvent> {
    use rayon::prelude::*;

    // Collect entity IDs to process (deterministic order: sorted)
    let mut citizen_ids: Vec<EntityId> = {
        let mut query = <(Entity, &Age, &Health)>::query();
        query.iter(world)
            .map(|(e, _, _)| e.id().into())
            .collect()
    };
    citizen_ids.sort_unstable();  // INVARIANT: deterministic order

    // Parallel computation (read-only, returns deltas)
    let deltas: Vec<CitizenDelta> = citizen_ids
        .par_iter()
        .map(|&id| {
            let local_seed = tick_phase_seed(rng_seed, tick, PhaseId::Demographics)
                .wrapping_add(id);
            let mut local_rng = ChaCha20Rng::seed_from_u64(local_seed);
            compute_citizen_delta(world, id, &mut local_rng, tick)
        })
        .collect();

    // Sequential application of deltas (deterministic: sorted entity order)
    let mut events = Vec::new();
    for delta in deltas {
        events.extend(apply_citizen_delta(world, delta));
    }
    events
}
```

**Key pattern:** Computation is parallel (read-only lambdas per entity); mutation is sequential (apply sorted deltas). This preserves determinism while gaining parallelism.

### 4.4 Query Types and Cache Characteristics

| Query Type | Cache Behavior | Used In |
|---|---|---|
| `<(&Position, &Inventory, &Mood)>::query()` | Sequential archetype scan, high cache hit rate on Citizen archetype | Production, Social |
| `<(&Position, &BuildingRole, &mut Inventory)>::query()` | Reads Position + BuildingRole (immutable), writes Inventory | Production phase |
| `<Entity, &Age, &Health>::query().filter(component::<Employment>())` | Filter by component presence, still contiguous | Demographics |
| `<(&MarketKey, &mut OrderBook)>::query()` | Small archetype (few markets), excellent cache locality | Trade |
| `<(&InstRole, &mut Treasury, &PolicyBundle)>::query()` | Very small archetype (few institutions), effectively L1-resident | Policy |

**SoA layout note:** `legion` uses archetype-based SoA layout. All `Position` components for Citizen entities are stored in one contiguous `Vec<Position>`. All `Mood` components are in another contiguous `Vec<Mood>`. Iterating both simultaneously is a single strided pass over two cache lines per entity pair. This is the primary performance advantage over an `AoS` layout where `struct Citizen { pos, mood, health, ... }` would interleave hot and cold fields.

**Cold data isolation:** Biography text, historical event logs, and birth-location metadata are stored outside the ECS in a `BTreeMap<EntityId, CitizenBiography>` in the engine resources. These are never accessed in hot-path phases. Keeping them out of the ECS prevents them from polluting archetype cache lines.

---

## 5. Performance Architecture

### 5.1 Tick Budget Targets

| Citizen Count | p50 Tick Time | p99 Tick Time | p999 Tick Time | Notes |
|---|---|---|---|---|
| 1,000 | ≤ 8 ms | ≤ 14 ms | ≤ 16 ms | Target for 60 FPS game clients |
| 10,000 | ≤ 30 ms | ≤ 45 ms | ≤ 50 ms | Acceptable for research mode |
| 100,000 | ≤ 150 ms | ≤ 180 ms | ≤ 200 ms | Research-only; tick rate dropped to 5/sec |

**Measurement methodology:**
- `TICK_DURATION_HISTOGRAM` Prometheus metric records wall-clock time per tick using `std::time::Instant` (wall clock only, not simulation time).
- `TICK_PHASE_DURATION` records per-phase latency.
- `criterion` benchmarks run in `crates/civ-engine/benches/tick_bench.rs` for offline regression testing.
- p50/p99/p999 percentiles exported via `opentelemetry-otlp` to Jaeger for production deployments.

### 5.2 Phase Budget Breakdown (at 1,000 Citizens)

| Phase | Budget | Parallelism | Bottleneck |
|---|---|---|---|
| CommandIntake | 50 µs | None | Command queue drain |
| Policy | 2 ms | None | Policy FSM traversal |
| Demographics | 1 ms | rayon | ChaCha20 RNG per citizen |
| Production | 3 ms | rayon | Per-building output formulas |
| Trade | 2 ms | None | Market clearing algorithm |
| Climate | 1 ms | rayon | Monte Carlo per cell |
| War | 2 ms | rayon | Combat resolution per battle |
| Social | 1.5 ms | rayon | Ideology dot products |
| Metrics | 0.8 ms | None | GDP fold, Gini sort |
| Snapshot | 50 µs | None | BLAKE3 hash + rmp-serde |
| **Total** | **~14 ms** | | |

### 5.3 SIMD Targets

**`glam` for spatial queries (`civ-geo`):**
`glam::Vec2` operations (distance, dot product) compile to SSE2 `MOVAPS/DPPS` instructions on x86. The A* pathfinding heuristic function (Euclidean distance squared) is called thousands of times per path query. SIMD reduces this from ~4 scalar multiplications + additions to a single `DPPS` instruction.

**Ideology vector dot products (`civ-social`):**
Each Citizen has a 4-component ideology vector `(auth_lib, collectivism, tradition, militarism)` stored as `[i16; 4]`. Social cohesion computation requires dot products between citizen ideology vectors and institutional ideology vectors. This is an 8×i16 → i32 sum pattern suitable for SIMD.

```rust
// civ-social/src/ideology.rs

/// Compute the alignment score between a citizen and an institution.
/// Uses a manual SIMD-friendly pattern that the compiler auto-vectorizes
/// to PMADDWD on x86 and SMLAL on ARM.
#[inline]
pub fn ideology_alignment(citizen: &Ideology, institution: &Ideology) -> i32 {
    // 4-element i16 dot product → i32
    // Compiler auto-vectorizes this to SIMD on any platform with -O2
    let a = [citizen.auth_lib, citizen.collectivism, citizen.tradition, citizen.militarism];
    let b = [institution.auth_lib, institution.collectivism, institution.tradition, institution.militarism];
    a.iter().zip(b.iter()).map(|(&x, &y)| (x as i32) * (y as i32)).sum()
}
```

Across 10,000 citizens, `rayon` dispatches this in parallel; with SIMD, per-citizen cost is ~1 ns instead of ~4 ns, reducing the social phase from ~4 ms to ~1 ms.

### 5.4 Memory Layout Strategy

**Hot path (ECS archetype arrays):** Citizen components (Position, Inventory, Mood, Health, Age) stored in contiguous `Vec<T>` per component type within each archetype. A sequential scan of 1,000 Citizen Mood components touches exactly 6 KB (1000 × 6 bytes), fitting in L1 cache (typically 32 KB).

**Cold data (off-ECS storage):** The following are stored outside the ECS in engine resources, accessed only on specific events:
- `BTreeMap<EntityId, CitizenBiography>` — name, birthplace, family history
- `BTreeMap<EntityId, Vec<HistoricalEvent>>` — per-citizen event log
- `BTreeMap<InstitutionId, PolicyHistory>` — past policy decisions

**Allocation strategy:** No per-tick heap allocations on hot paths. Entity deletion uses tombstone marking (set `alive = false` in `BitSet`) and deferred compaction every 100 ticks. Event vectors are pre-allocated with capacity `= expected_events_per_tick × 1.5` and reset each tick without deallocation.

### 5.5 Profiling Methodology

**Online profiling (`tracing` + OpenTelemetry):**
```rust
// civ-engine/src/phase.rs
use tracing::{instrument, span, Level};

#[instrument(skip_all, fields(tick = tick, phase = ?phase_id))]
pub fn run_phase(phase_id: PhaseId, tick: u64, world: &mut legion::World) {
    let _span = span!(Level::DEBUG, "phase", phase = ?phase_id).entered();
    // ... phase execution
}
```
Spans export to Jaeger via `opentelemetry-otlp`. Flamegraph-equivalent data is available in the Jaeger UI for each tick.

**Offline benchmarks (`criterion`):**
```bash
cargo bench --bench tick_bench -- --save-baseline main
# After changes:
cargo bench --bench tick_bench -- --baseline main
```
Criterion reports percent change with confidence intervals. A regression > 10% in the p99 tick time triggers a CI failure.

**Flamegraph:**
```bash
cargo flamegraph --bin civ-server -- --headless --ticks 1000 --citizens 10000
```
Uses `cargo-flamegraph` (wraps `perf record` on Linux, `dtrace` on macOS). Output is `flamegraph.svg` in the project root.

---

## 6. Determinism Architecture

### 6.1 Determinism Contract

**Formal invariant:** For all `(seed, scenario, commands)`, running the simulation twice produces:
1. Identical `state_hash` at every tick.
2. Identical `events` sequence at every tick.
3. Identical final `Snapshot` after N ticks.

This invariant is verified by the CI replay test on every PR merge.

### 6.2 Full Enumeration of Determinism Rules

#### Rule D1: No `f32`/`f64` in Simulation Logic

**Category:** Type system enforcement
**Enforced by:** Clippy lint `clippy::float_arithmetic`, scoped to simulation crates via `.cargo/config.toml`

```rust
// VIOLATION — caught at compile time:
let output: f64 = inventory.grain as f64 * 0.95;  // ERROR: float arithmetic

// CORRECT:
let output: i64 = inventory.grain * 95 / 100;  // exact, integer
```

**Clippy configuration (`.cargo/config.toml` for sim crates):**
```toml
[target.'cfg(all())'.rustflags]
rustflags = [
    "-W", "clippy::float_arithmetic",
    "-D", "clippy::float_arithmetic",  # deny = build error
]
```

**Exceptions:** `civ-geo` (SIMD spatial math with `glam`), `civ-metrics` (Prometheus gauge export). Both crates are excluded from the deny via `#[allow(clippy::float_arithmetic)]` at crate level with mandatory justification comment.

#### Rule D2: All RNG through `ChaCha20Rng` — Seeded, Not Defaulted

**Category:** API contract
**Enforced by:** CI lint scanning for `rand::random`, `rand::thread_rng`, `OsRng`, `SmallRng::from_entropy`

```rust
// VIOLATION:
let choice = rand::random::<usize>() % options.len();  // non-deterministic

// CORRECT:
let mut rng = make_rng(sim_seed, tick, PhaseId::Demographics);
let choice = rng.gen_range(0..options.len());
```

**Seed derivation for every RNG call site:**
```rust
pub fn make_rng(sim_seed: u64, tick: u64, phase: PhaseId) -> ChaCha20Rng {
    let seed = sim_seed
        .wrapping_mul(0x9e37_79b9_7f4a_7c15)
        .wrapping_add(tick.wrapping_mul(0x6c62_272e_07bb_0142))
        .wrapping_add(phase as u64 * 0xbf58_476d_1ce4_e5b9);
    ChaCha20Rng::seed_from_u64(seed)
}

// For per-entity stochastic decisions, additionally mix in entity ID:
pub fn make_entity_rng(sim_seed: u64, tick: u64, phase: PhaseId, entity_id: u64) -> ChaCha20Rng {
    let seed = make_rng(sim_seed, tick, phase)
        .next_u64()
        .wrapping_add(entity_id * 0x517c_c1b7_2722_0a95);
    ChaCha20Rng::seed_from_u64(seed)
}
```

#### Rule D3: `BTreeMap` Everywhere — Never `HashMap` in Simulation State

**Category:** Data structure discipline
**Enforced by:** Custom clippy lint `sim_hashmap_forbidden` (checked in `pre-commit` hook)

```rust
// VIOLATION:
let mut goods: HashMap<GoodId, Quantity> = HashMap::new();
for (id, qty) in &goods {
    emit_event(id, qty);  // iteration order: undefined
}

// CORRECT:
let mut goods: BTreeMap<GoodId, Quantity> = BTreeMap::new();
for (id, qty) in &goods {
    emit_event(id, qty);  // iteration order: ascending by GoodId (Ord)
}
```

**Where `HashMap` is permitted:** `civ-server` (session registry, not simulation state), `civ-metrics` (metric label indexes), build scripts.

**Where `BTreeMap` is mandatory:** All types in `civ-protocol`, all simulation state in domain crates, all command queues, all event buffers.

#### Rule D4: No `SystemTime` or `Instant` in Simulation State

**Category:** Clock discipline
**Enforced by:** Grep-based CI check for `SystemTime::now()` and `Instant::now()` in simulation crates

```rust
// VIOLATION:
let now = SystemTime::now();
let elapsed = now.elapsed().unwrap().as_secs();

// CORRECT:
let current_tick = simulation.tick;
let elapsed_ticks = current_tick - started_tick;
let sim_seconds = elapsed_ticks * 100;  // 100ms per tick
```

`Instant::now()` is allowed only in `civ-server` (for WebSocket ping/pong timing) and `civ-metrics` (for Prometheus timestamp export). Both usages are excluded from simulation state.

#### Rule D5: Sorted Entity Iteration Order

**Category:** Iteration discipline
**Enforced by:** Code review + the canonical `run_demographics_phase` pattern

All phases that iterate entities and produce events or mutations must sort entity IDs before iteration:

```rust
// The canonical parallel phase pattern (D5 + D2 combined)

let mut entity_ids: Vec<EntityId> = query.iter(world)
    .map(|(e, ..)| EntityId::from(e))
    .collect();
entity_ids.sort_unstable();  // REQUIRED: deterministic order

let deltas: Vec<Delta> = entity_ids
    .par_iter()
    .map(|&id| {
        let rng = make_entity_rng(seed, tick, phase, id);
        compute_delta(world, id, rng)
    })
    .collect();  // rayon preserves par_iter order in collect()

// Apply in sorted order
for delta in &deltas {
    apply_delta(world, *delta);
}
```

**Why `par_iter().collect()` is deterministic:** `rayon::par_iter()` followed by `.collect()` preserves input order in the output `Vec`. Tasks execute in parallel but the result vector is assembled in original order. This is a documented rayon guarantee.

#### Rule D6: Immutable Phase Input — Functional Data Flow

**Category:** Architecture discipline
**Enforced by:** Type system (`&State` vs `&mut State` parameters)

Each phase receives immutable access to prior-phase state and produces a list of mutations. Mutations are accumulated in a `Vec<Delta>` and applied sequentially after the phase completes:

```rust
// Phase signature contract
pub trait Phase {
    fn run(
        &self,
        world: &legion::World,           // IMMUTABLE read of current state
        resources: &legion::Resources,
        rng_seed: u64,
        tick: u64,
    ) -> Vec<WorldMutation>;             // OWNED output, no aliasing
}

// Engine applies mutations sequentially after each phase
fn apply_mutations(world: &mut legion::World, mutations: Vec<WorldMutation>) {
    let mut sorted = mutations;
    sorted.sort_by_key(|m| m.entity_id);  // D5: deterministic application order
    for mutation in sorted {
        mutation.apply(world);
    }
}
```

#### Rule D7: State Hash Verification

**Category:** Correctness verification
**Enforced by:** CI determinism replay test

Every tick produces a `state_hash: [u8; 32]` computed by BLAKE3 over the canonical CBOR serialization of the full ECS world state. This hash is:
- Included in every `TickOutput`
- Included in every emitted `SimEvent` (as `state_hash` field)
- Verified during `.civreplay` replay against the original run's hashes

```rust
// civ-engine/src/hash.rs

pub fn compute_state_hash(world: &legion::World, tick: u64) -> [u8; 32] {
    use ciborium::into_writer;
    use blake3::Hasher;

    // Serialize world to canonical CBOR
    let mut cbor_bytes: Vec<u8> = Vec::with_capacity(256 * 1024);
    let canonical_world = CanonicalWorldView::from(world);  // sorted, deterministic
    into_writer(&canonical_world, &mut cbor_bytes).expect("cbor serialization must not fail");

    // Include tick in hash to prevent cross-tick hash collisions
    let mut hasher = Hasher::new();
    hasher.update(&tick.to_le_bytes());
    hasher.update(&cbor_bytes);
    hasher.finalize().into()
}
```

### 6.3 CI Enforcement: Determinism Replay Test

```rust
// civ-engine/tests/fr_determinism_replay.rs

/// FR-CIV-CORE-002: Deterministic Transition
/// FR-CIV-CORE-011: Replay Determinism Verification
#[test]
fn determinism_replay_100_ticks() {
    let seed = 0xDEAD_BEEF_0BAD_CAFEu64;
    let scenario = Scenario::test_default();

    // Run 1
    let mut sim_a = Simulation::new(seed, &scenario).unwrap();
    let mut hashes_a: Vec<[u8; 32]> = Vec::new();
    for _ in 0..100 {
        let output = sim_a.tick().unwrap();
        hashes_a.push(output.state_hash);
    }

    // Run 2 (same inputs, fresh simulation)
    let mut sim_b = Simulation::new(seed, &scenario).unwrap();
    let mut hashes_b: Vec<[u8; 32]> = Vec::new();
    for _ in 0..100 {
        let output = sim_b.tick().unwrap();
        hashes_b.push(output.state_hash);
    }

    // Every tick must match
    for (i, (a, b)) in hashes_a.iter().zip(hashes_b.iter()).enumerate() {
        assert_eq!(a, b,
            "Determinism violation at tick {}: hash_a={} hash_b={}",
            i,
            hex::encode(a),
            hex::encode(b)
        );
    }
}

/// Test with multiple seeds
#[test]
fn determinism_multiple_seeds() {
    for seed in [1u64, 2, 3, 5, 7, 11, 13, 0xFFFF_FFFF_FFFF_FFFF] {
        let scenario = Scenario::test_default();
        let mut sim_a = Simulation::new(seed, &scenario).unwrap();
        let mut sim_b = Simulation::new(seed, &scenario).unwrap();
        for _ in 0..50 {
            let out_a = sim_a.tick().unwrap();
            let out_b = sim_b.tick().unwrap();
            assert_eq!(out_a.state_hash, out_b.state_hash,
                "seed={:#018x}", seed);
        }
    }
}
```

---

## 7. Server Architecture

### 7.1 axum Router Layout

```rust
// civ-server/src/router.rs

pub fn build_router(state: ServerState) -> axum::Router {
    axum::Router::new()
        // WebSocket endpoint (game clients, research clients)
        .route("/sim", get(ws_handler))

        // REST endpoints (admin, health, metrics)
        .route("/health",   get(health_handler))
        .route("/metrics",  get(metrics_handler))   // Prometheus scrape
        .route("/admin/snapshot", get(admin_snapshot_handler))
        .route("/admin/seed",     get(admin_seed_handler))

        // Tower middleware stack
        .layer(
            ServiceBuilder::new()
                .layer(TraceLayer::new_for_http()           // tracing spans per request
                    .make_span_with(DefaultMakeSpan::new()
                        .level(Level::DEBUG)))
                .layer(TimeoutLayer::new(Duration::from_secs(30)))
                .layer(ConcurrencyLimitLayer::new(200))     // max concurrent HTTP connections
        )
        .with_state(state)
}

#[derive(Clone)]
pub struct ServerState {
    pub broadcast_rx: BroadcastReceiver<Arc<BroadcastFrame>>,
    pub command_tx:   mpsc::Sender<Command>,
    pub session_registry: Arc<Mutex<SessionRegistry>>,  // OK: not hot path
    pub db_pool:      sqlx::PgPool,
}
```

### 7.2 WebSocket Upgrade Handler

```rust
// civ-server/src/ws_handler.rs

/// Handles WebSocket upgrade and manages the per-client session lifecycle.
/// This function is called once per client connection. It runs as a tokio task.
pub async fn ws_handler(
    ws: WebSocketUpgrade,
    State(state): State<ServerState>,
    Query(params): Query<HandshakeParams>,
) -> impl IntoResponse {
    ws.on_upgrade(move |socket| handle_client(socket, state, params))
}

async fn handle_client(
    socket: WebSocket,
    state: ServerState,
    params: HandshakeParams,
) {
    let session = ClientSession::new(params);
    let session_id = session.id;

    state.session_registry
        .lock().await
        .insert(session_id, session.clone());

    // Subscribe to the broadcast channel for this client
    let mut rx = state.broadcast_rx.resubscribe();

    let (mut ws_tx, mut ws_rx) = socket.split();

    // Send initial handshake response
    let initial_snapshot = fetch_current_snapshot(&state).await;
    let handshake_response = serde_json::to_string(&HandshakeResponse {
        session_id,
        tick: initial_snapshot.header.tick,
        snapshot: initial_snapshot,
    }).unwrap();
    ws_tx.send(Message::Text(handshake_response)).await.ok();

    // Concurrent loops: receive commands + send broadcasts
    loop {
        tokio::select! {
            // Incoming command from client
            msg = ws_rx.next() => {
                match msg {
                    Some(Ok(Message::Text(text))) => {
                        if let Ok(cmd) = serde_json::from_str::<Command>(&text) {
                            state.command_tx.send(cmd).await.ok();
                        }
                    }
                    None | Some(Err(_)) => break,  // client disconnected
                    _ => {}
                }
            }

            // Outgoing broadcast frame from simulation
            frame = rx.recv() => {
                match frame {
                    Ok(frame) => {
                        let filtered = frame.filter(&session.filter);
                        let msg = if session.use_binary_frames {
                            Message::Binary(filtered.to_msgpack_bytes())
                        } else {
                            Message::Text(filtered.to_json_rpc_string())
                        };
                        if ws_tx.send(msg).await.is_err() {
                            break;  // client disconnected
                        }
                    }
                    Err(tokio::sync::broadcast::error::RecvError::Lagged(n)) => {
                        tracing::warn!("Client {} lagged {} frames", session_id, n);
                        // Client is too slow; drop frames, do not block simulation
                    }
                    Err(tokio::sync::broadcast::error::RecvError::Closed) => break,
                }
            }
        }
    }

    state.session_registry.lock().await.remove(&session_id);
}
```

### 7.3 Multi-Client Broadcast via `tokio::broadcast::channel`

```rust
// civ-server/src/broadcast.rs

/// The broadcast channel has capacity = 256 frames (~25 seconds at 10 ticks/sec).
/// Slow clients lag but never block the simulation thread.
pub const BROADCAST_CAPACITY: usize = 256;

pub fn create_broadcast_channel() -> (
    tokio::sync::broadcast::Sender<Arc<BroadcastFrame>>,
    tokio::sync::broadcast::Receiver<Arc<BroadcastFrame>>,
) {
    tokio::sync::broadcast::channel(BROADCAST_CAPACITY)
}

/// The sim bridge task runs on the async executor and forwards frames
/// from the sim thread's mpsc channel to the broadcast channel.
pub async fn run_sim_bridge(
    mut sim_rx: mpsc::Receiver<TickOutput>,
    broadcast_tx: broadcast::Sender<Arc<BroadcastFrame>>,
) {
    while let Some(output) = sim_rx.recv().await {
        let frame = Arc::new(BroadcastFrame::from(output));
        // broadcast::send returns Err only if there are no receivers
        // (all clients disconnected). This is not an error.
        let _ = broadcast_tx.send(frame);
        BROADCAST_CLIENTS_GAUGE.set(broadcast_tx.receiver_count() as i64);
    }
}
```

**Design note:** `Arc<BroadcastFrame>` is used because `broadcast::channel` clones the value for each receiver. Cloning an `Arc` is `O(1)` (atomic increment); cloning a `BroadcastFrame` (potentially 120 KB) for each of 100 clients would be `O(n × frame_size)`. With `Arc`, all clients share the same heap allocation.

### 7.4 Command Priority Queue

Commands from clients are buffered in a `BinaryHeap` ordered by `(client_priority, tick_received)`. The `CommandIntake` phase drains this queue at the start of each tick.

```rust
// civ-server/src/sim_bridge.rs

#[derive(PartialEq, Eq)]
struct PrioritizedCommand {
    priority:      u32,       // lower = higher priority
    tick_received: u64,
    command:       Command,
}

// BinaryHeap is a max-heap; we want min-priority first,
// so we invert the Ord impl.
impl Ord for PrioritizedCommand {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        other.priority.cmp(&self.priority)
            .then_with(|| other.tick_received.cmp(&self.tick_received))
    }
}

impl PartialOrd for PrioritizedCommand {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

pub struct CommandQueue {
    heap: BinaryHeap<PrioritizedCommand>,
}

impl CommandQueue {
    /// Called by CommandIntake phase at start of each tick.
    /// Returns commands sorted by (priority asc, tick_received asc).
    pub fn drain_for_tick(&mut self) -> Vec<Command> {
        let mut cmds = Vec::new();
        while let Some(pc) = self.heap.pop() {
            cmds.push(pc.command);
        }
        cmds
    }
}
```

**Priority tier mapping:**
```rust
pub enum ClientType {
    Admin       = 0,  // override any command
    Game        = 1,  // player game clients
    AI          = 2,  // NPC / research agents
    ReadOnly    = 3,  // loggers, monitoring
}
```

### 7.5 Simulation Loop vs. Server Event Loop Separation

```
╔══════════════════════════════════════════════════════════════════════╗
║  SIMULATION THREAD (std::thread)                                     ║
║                                                                      ║
║  loop {                                                              ║
║      // 1. Drain command queue (mpsc::Receiver, try_recv)            ║
║      let commands = cmd_rx.try_recv_all();                           ║
║                                                                      ║
║      // 2. Execute tick (CPU-bound, ~14ms)                           ║
║      let output = engine.tick(commands)?;                            ║
║                                                                      ║
║      // 3. Send to async bridge (non-blocking)                       ║
║      sim_tx.try_send(output)?;                                       ║
║                                                                      ║
║      // 4. Sleep to next tick boundary (100ms)                       ║
║      sleep_until(next_tick_at);                                      ║
║  }                                                                   ║
╚════════════════════════════╤═════════════════════════════════════════╝
                             │ mpsc::channel  (sim → async)
╔════════════════════════════▼═════════════════════════════════════════╗
║  TOKIO ASYNC EXECUTOR (multi-thread scheduler)                       ║
║                                                                      ║
║  run_sim_bridge task:                                                ║
║      sim_rx.recv() → broadcast_tx.send(Arc<BroadcastFrame>)         ║
║                                                                      ║
║  Per-client ws_handler tasks:                                        ║
║      rx.recv() → ws_tx.send(filtered_frame)                         ║
║      ws_rx.next() → cmd_queue.push(command)                         ║
║                                                                      ║
║  HTTP handler tasks:                                                 ║
║      /health, /metrics, /admin/snapshot                              ║
╚══════════════════════════════════════════════════════════════════════╝
```

**Key invariant:** The simulation thread is never `await`ed. It never uses `tokio::spawn` or any async primitive. The only bridge is `std::sync::mpsc::channel` for commands (client → sim) and `tokio::sync::mpsc::channel` for tick outputs (sim → async). This design ensures that:

1. The simulation tick is not subject to tokio task scheduling jitter.
2. Slow clients (backpressure on `broadcast::Receiver`) do not slow down the simulation.
3. The simulation can advance at any configured tick rate independently of network I/O.

---

## 8. Build and CI Configuration

### 8.1 Workspace `Cargo.toml` Profiles

```toml
# Cargo.toml (workspace root)

[profile.release]
opt-level = 3
lto = "thin"               # thin LTO: ~20% binary size reduction, faster than "fat"
codegen-units = 1          # single codegen unit for maximum inlining across crates
strip = "debuginfo"        # strip debug symbols from release binary
panic = "abort"            # no unwinding in release; smaller binary, faster

[profile.dev]
opt-level = 1              # opt-level=1: faster test compilation than 0, slow enough to debug
debug = true
overflow-checks = true     # catch integer overflows in dev/test
incremental = true

[profile.bench]
inherits = "release"
debug = true               # keep debug info for flamegraph / perf

[profile.test]
opt-level = 1
debug = true
overflow-checks = true
```

### 8.2 Feature Flags

```toml
# civ-engine/Cargo.toml

[features]
default = []

# Enables the Python research API (pyo3 bindings)
research-api = ["pyo3"]

# Enables compile-time SQL verification (requires DB at compile time)
sqlx-offline = ["sqlx/offline"]

# Enables additional tracing instrumentation (performance overhead)
trace-verbose = []
```

**Usage:**
```bash
# Build server binary (no Python bindings)
cargo build --release --bin civ-server

# Build Python extension module
cargo build --release --features research-api --lib

# Build with offline sqlx (no DB connection at compile time)
SQLX_OFFLINE=true cargo build --release
```

### 8.3 CI Pipeline

```yaml
# .github/workflows/ci.yml (representative structure)

name: CI

on: [push, pull_request]

jobs:
  fmt:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - run: cargo fmt --all -- --check

  clippy:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - run: |
          cargo clippy --all-targets --all-features -- \
            -D warnings \
            -D clippy::float_arithmetic \
            -D clippy::unwrap_used \
            -D clippy::expect_used \
            -D clippy::panic \
            -W clippy::pedantic

  test:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - run: cargo nextest run --all --no-fail-fast
      - run: cargo test --doc --all   # doctest pass

  determinism:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - run: cargo nextest run -p civ-engine --test fr_determinism_replay

  bench-regression:
    runs-on: ubuntu-latest
    if: github.event_name == 'pull_request'
    steps:
      - uses: actions/checkout@v4
        with: { fetch-depth: 0 }
      - run: |
          cargo bench --bench tick_bench -- --save-baseline pr
          git checkout ${{ github.base_ref }}
          cargo bench --bench tick_bench -- --baseline pr --load-baseline main
          # criterion returns exit code 1 if regression > threshold

  deny:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - run: cargo deny check licenses bans advisories

  boundary:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - run: pip install tach && tach check
```

**CI gates that must pass before merge:**
1. `cargo fmt --check` — formatting
2. `cargo clippy -D warnings -D clippy::float_arithmetic` — lints + no-float enforcement
3. `cargo nextest run --all` — all tests including determinism replay
4. `cargo deny check` — license compliance + CVE scanning
5. `tach check` — crate boundary enforcement
6. Bench regression check on PRs (p99 tick time must not increase > 10%)

### 8.4 Cargo Deny Configuration

```toml
# deny.toml

[licenses]
allow = [
    "MIT",
    "Apache-2.0",
    "Apache-2.0 WITH LLVM-exception",
    "BSD-2-Clause",
    "BSD-3-Clause",
    "ISC",
    "Unicode-DFS-2016",
    "Zlib",
]
deny = ["GPL-2.0", "GPL-3.0", "LGPL-2.0", "LGPL-3.0", "AGPL-3.0"]
copyleft = "deny"

[advisories]
db-path = "~/.cargo/advisory-db"
db-urls = ["https://github.com/rustsec/advisory-db"]
vulnerability = "deny"
unmaintained = "warn"
unsound = "deny"
```

---

## 9. Python Research Bindings

### 9.1 Design Goals

The Python research API exposes the simulation engine as a Python extension module (`.so` / `.pyd`) without requiring the WebSocket server infrastructure. A researcher can run thousands of simulations with parameter sweeps using `multiprocessing` or `concurrent.futures`.

**Target usage:**
```python
from civlab import CivLab, Scenario, SimulationResult
import multiprocessing

def run_scenario(params):
    seed, tax_rate, subsidy_level = params
    lab = CivLab()
    result = lab.run(
        seed=seed,
        scenario=Scenario.from_toml("scenarios/base.toml").with_params(
            tax_rate=tax_rate,
            subsidy_level=subsidy_level,
        ),
        n_ticks=1000,
    )
    return result.metrics_array()

# Parameter sweep: 100 seeds × 10 tax rates × 5 subsidy levels = 5,000 runs
params = [
    (seed, tax, sub)
    for seed in range(100)
    for tax in [0.1, 0.15, 0.2, 0.25, 0.3, 0.35, 0.4, 0.45, 0.5, 0.55]
    for sub in [0.0, 0.1, 0.2, 0.3, 0.4]
]

with multiprocessing.Pool(processes=multiprocessing.cpu_count()) as pool:
    results = pool.map(run_scenario, params)
```

### 9.2 pyo3 FFI Layer

```rust
// civ-engine/src/python.rs  (compiled only with feature = "research-api")

use pyo3::prelude::*;
use pyo3::types::{PyDict, PyList};
use numpy::{PyArray1, PyArray2};

/// Python class: CivLab
/// Entry point for headless simulation runs.
#[pyclass(name = "CivLab")]
pub struct PyCivLab;

#[pymethods]
impl PyCivLab {
    #[new]
    pub fn new() -> Self {
        PyCivLab
    }

    /// Run a simulation for n_ticks and return a SimulationResult.
    pub fn run(
        &self,
        py: Python<'_>,
        seed: u64,
        scenario: &PyScenario,
        n_ticks: u64,
    ) -> PyResult<PySimulationResult> {
        // Release GIL during simulation (CPU-bound, no Python objects touched)
        py.allow_threads(|| {
            let mut sim = Simulation::new(seed, &scenario.inner)
                .map_err(|e| PyRuntimeError::new_err(e.to_string()))?;

            let mut snapshots = Vec::with_capacity(n_ticks as usize);
            let mut events = Vec::new();

            for _ in 0..n_ticks {
                let output = sim.tick()
                    .map_err(|e| PyRuntimeError::new_err(e.to_string()))?;
                snapshots.push(output.metrics);
                events.extend(output.events);
            }

            Ok(PySimulationResult {
                seed,
                n_ticks,
                snapshots,
                final_hash: sim.state_hash(),
            })
        })
    }
}

/// Python class: SimulationResult
#[pyclass(name = "SimulationResult")]
pub struct PySimulationResult {
    seed: u64,
    n_ticks: u64,
    snapshots: Vec<MetricsSnapshot>,
    final_hash: [u8; 32],
}

#[pymethods]
impl PySimulationResult {
    /// Return metrics as a numpy-compatible 2D array.
    /// Shape: (n_ticks, n_metrics)
    /// Columns: [tick, population, gdp, avg_happiness, gini, legitimacy, insurgency_risk, hdi]
    pub fn metrics_array<'py>(&self, py: Python<'py>) -> &'py PyArray2<f64> {
        let n_cols = 8usize;
        let mut data = vec![0.0f64; self.snapshots.len() * n_cols];
        for (i, snap) in self.snapshots.iter().enumerate() {
            let row = i * n_cols;
            data[row + 0] = snap.tick as f64;
            data[row + 1] = snap.population as f64;
            data[row + 2] = snap.gdp as f64 / 1_000.0;  // convert milli-credits
            data[row + 3] = snap.avg_happiness as f64 / 10.0;
            data[row + 4] = snap.gini as f64 / 1_000.0;
            data[row + 5] = snap.legitimacy as f64 / 10.0;
            data[row + 6] = snap.insurgency_risk as f64 / 10.0;
            data[row + 7] = snap.hdi as f64 / 1_000.0;
        }
        PyArray2::from_vec2(py, &data.chunks(n_cols)
            .map(|row| row.to_vec())
            .collect::<Vec<_>>())
            .unwrap()
    }

    /// Return the final state hash as a hex string (for reproducibility verification)
    #[getter]
    pub fn final_hash_hex(&self) -> String {
        hex::encode(self.final_hash)
    }
}

/// Python module registration
#[pymodule]
fn civlab(_py: Python<'_>, m: &PyModule) -> PyResult<()> {
    m.add_class::<PyCivLab>()?;
    m.add_class::<PyScenario>()?;
    m.add_class::<PySimulationResult>()?;
    Ok(())
}
```

### 9.3 GIL Release Strategy

The simulation tick loop is CPU-bound Rust code with no Python object access. `py.allow_threads()` releases the GIL for the entire `n_ticks` loop. This enables:
- True parallelism when `multiprocessing` spawns separate processes (each has its own GIL)
- Concurrent simulation runs in the same process using `concurrent.futures.ThreadPoolExecutor` (since GIL is released during simulation, threads actually run concurrently for the CPU-bound portion)

**Recommended Python usage for maximum throughput:**
```python
# Use ProcessPoolExecutor for maximum isolation + no GIL contention
from concurrent.futures import ProcessPoolExecutor

with ProcessPoolExecutor(max_workers=16) as executor:
    futures = [executor.submit(run_one, params) for params in param_list]
    results = [f.result() for f in futures]
```

### 9.4 Build and Distribution

**maturin** is used to build the Python extension module:

```bash
# Install maturin
pip install maturin

# Build the extension module (debug)
maturin develop --features research-api

# Build the wheel for distribution
maturin build --release --features research-api --strip
# Output: target/wheels/civlab-0.1.0-cp310-cp310-linux_x86_64.whl
```

**`pyproject.toml` (in civ-engine):**
```toml
[build-system]
requires = ["maturin>=1.4,<2.0"]
build-backend = "maturin"

[project]
name = "civlab"
requires-python = ">=3.10"
dependencies = ["numpy>=1.24"]

[tool.maturin]
features = ["research-api"]
python-source = "python"
module-name = "civlab._civlab"
```

---

## 10. Non-Functional Requirements Table

### 10.1 Correctness — Determinism

| # | Metric | Target | Measurement Method | Enforcement Mechanism |
|---|---|---|---|---|
| NFR-C-01 | Tick-by-tick state hash match | 100% of ticks match across two independent runs with same seed | `fr_determinism_replay` test: compare `state_hash` per tick | CI gate: test must pass on every PR |
| NFR-C-02 | Cross-platform hash match | Same hash on x86_64 Linux, ARM64 macOS, WASM | Matrix CI build + determinism test on all three targets | CI matrix: GitHub Actions with `ubuntu-latest`, `macos-latest`, `ubuntu-arm` runners |
| NFR-C-03 | Fixed-point arithmetic enforcement | Zero `f32`/`f64` operations in simulation crates | Clippy `float_arithmetic` lint | `cargo clippy -D clippy::float_arithmetic` in CI (build fails on violation) |
| NFR-C-04 | RNG seeding coverage | Zero calls to unseeded RNG in simulation crates | grep-based CI scan for `thread_rng`, `rand::random`, `OsRng` | Pre-commit hook + CI scan step |
| NFR-C-05 | BTreeMap enforcement | Zero `HashMap` in simulation state types | Custom clippy lint `sim_hashmap_forbidden` | Pre-commit hook scans `use std::collections::HashMap` in sim crates |
| NFR-C-06 | Event log completeness | Every state-mutating phase produces at least one event per tick | `fr_event_completeness` test: assert `events.len() > 0` per tick | Unit test in `civ-engine/tests/` |
| NFR-C-07 | State hash in every event | `SimEvent.state_hash` must match `state_hash` at tick of emission | Replay verifier: recompute hash, compare | `civ-replay` verifier runs on `.civreplay` files in CI |

### 10.2 Performance — Tick Latency

| # | Metric | Target | Measurement Method | Enforcement Mechanism |
|---|---|---|---|---|
| NFR-P-01 | p50 tick time (1k citizens) | ≤ 8 ms | `TICK_DURATION_HISTOGRAM` p50, scraped by Prometheus | CI bench regression: `criterion` baseline comparison on PR |
| NFR-P-02 | p99 tick time (1k citizens) | ≤ 14 ms | `TICK_DURATION_HISTOGRAM` p99 | CI bench regression gate: fail PR if p99 increases > 10% |
| NFR-P-03 | p999 tick time (1k citizens) | ≤ 16 ms | `TICK_DURATION_HISTOGRAM` p999 | Prometheus alert in production |
| NFR-P-04 | p50 tick time (10k citizens) | ≤ 30 ms | Same histogram, different scenario | Separate `bench_10k_citizens` criterion benchmark |
| NFR-P-05 | p50 tick time (100k citizens) | ≤ 150 ms | Same histogram, large scenario | Performance regression test in nightly CI only |
| NFR-P-06 | Broadcast lag | p99 ≤ 10 ms from tick completion to last client delivery | `BROADCAST_LAG_HISTOGRAM` | Prometheus alert: `civlab_broadcast_lag_seconds{quantile="0.99"} > 0.010` |
| NFR-P-07 | Snapshot serialization overhead | ≤ 1 ms per tick for 1k citizens | `TICK_PHASE_DURATION{phase="Snapshot"}` | Criterion benchmark `bench_snapshot_1k` |
| NFR-P-08 | Memory footprint | ≤ 256 MB RSS for 10k citizens | `/proc/self/status` VmRSS in health endpoint | Nightly memory regression test |

### 10.3 Scalability

| # | Metric | Target | Measurement Method | Enforcement Mechanism |
|---|---|---|---|---|
| NFR-S-01 | Max simultaneous WebSocket clients | ≥ 100 clients at 10 ticks/sec | Load test: 100 concurrent `tokio-tungstenite` clients | Load test in CI (`tests/load/100_clients.rs`) |
| NFR-S-02 | Client connection overhead | ≤ 5 ms per client connection (handshake + initial snapshot) | WebSocket upgrade + handshake response latency percentile | Integration test with timer |
| NFR-S-03 | Citizen count scaling | Tick time scales sub-linearly from 1k to 10k citizens | Ratio: `tick_time_10k / tick_time_1k ≤ 8` (expect ~5 with rayon) | Criterion comparison benchmark |
| NFR-S-04 | Command throughput | ≥ 1,000 commands/sec accepted without tick delay | Stress test: flood `command_tx` at 1k/sec, verify tick time unchanged | Load test with command flood |
| NFR-S-05 | Event log growth rate | ≤ 5 MB/minute at 1k citizens, 10 ticks/sec | Monitor `civlab_event_log_bytes_total` | Prometheus recording rule + alert |
| NFR-S-06 | WebSocket frame size | ≤ 20 KB average binary frame for 1k citizen snapshot | `FRAME_SIZE_HISTOGRAM` | Unit test on `BinaryFrame::to_msgpack_bytes()` with reference snapshot |

### 10.4 Reliability — Crash Recovery

| # | Metric | Target | Measurement Method | Enforcement Mechanism |
|---|---|---|---|---|
| NFR-R-01 | Snapshot persistence interval | Snapshot written to PostgreSQL every 100 ticks | `civlab_snapshots_written_total` counter | Integration test: run 100 ticks, verify DB has 1 snapshot row |
| NFR-R-02 | Recovery point objective (RPO) | On crash, resume from last persisted snapshot (≤ 100 ticks lost) | Kill server mid-run, restart, verify tick counter | Recovery integration test |
| NFR-R-03 | Recovery time objective (RTO) | Server restart + state load ≤ 30 seconds | Time from process start to first tick broadcast | Health check endpoint `/health` transitions from `starting` to `ready` |
| NFR-R-04 | Event log durability | Event log flushed to disk before acknowledgement | `fsync` on event log append (O_DSYNC) | Unit test: write event, kill process, verify log on restart |
| NFR-R-05 | Client reconnect | Client can reconnect and receive current snapshot within 2 seconds | Integration test: disconnect client, reconnect, measure time to first snapshot | WebSocket reconnect test |
| NFR-R-06 | Simulation panic isolation | Panic in one tick phase does not kill the server process | Inject panic via test endpoint, verify server continues | Integration test with panic injection |

### 10.5 Observability

| # | Metric | Target | Measurement Method | Enforcement Mechanism |
|---|---|---|---|---|
| NFR-O-01 | Prometheus metric coverage | 100% of tick phases have latency histograms | Count `HistogramVec` labels vs `PhaseId` enum variants | CI test: verify each `PhaseId` has a corresponding metric |
| NFR-O-02 | Structured log completeness | Every error has structured fields: `tick`, `phase`, `entity_id`, `error` | Log schema validation in CI | `tracing` instrumentation review checklist |
| NFR-O-03 | Trace propagation | Every WebSocket command is traceable from client receipt to tick application | `tracing::span` with `trace_id` propagated through command → phase | Manual trace inspection in Jaeger |
| NFR-O-04 | Metrics cardinality | Total Prometheus time series count ≤ 10,000 | Prometheus cardinality API: `count({__name__=~".+"})` | Prometheus alert: `prometheus_tsdb_head_series > 10000` |
| NFR-O-05 | Dashboard coverage | All NFR metrics visible in Grafana dashboard | Manual dashboard review | Dashboard JSON committed to repo at `ops/grafana/civ-sim.json` |
| NFR-O-06 | Alert coverage | Each p99 latency target has a Prometheus alerting rule | Count alert rules vs NFR-P-* count | CI: validate `ops/prometheus/alerts.yml` with `promtool check rules` |

---

## Appendix A: Acceptance Criteria Cross-Reference

Each section above maps to Functional Requirements. The canonical FR list is in the `docs/specs/` directory. This table provides the cross-reference for traceability.

| Spec Section | FR IDs | Source Spec |
|---|---|---|
| §4 ECS World Design | FR-CIV-CORE-019, FR-CIV-ACT-001 | CIV-0001 |
| §6 Determinism Architecture | FR-CIV-CORE-001 through 015 | CIV-0001 |
| §7 Server Architecture | FR-CIV-PROTO-001 through 015 | CIV-0200 |
| §9 Python Research Bindings | FR-CIV-RESEARCH-001 through 004 | PLAN.md P5 |
| §10 NFR Table | All NFR-* | ADR-003, ADR-004 |
| §2.2 ECS: legion | ADR-006 | ADR.md |
| §2.4 Fixed-point | ADR-007 | ADR.md |
| §2.7 WebSocket | ADR-005 | ADR.md |
| §2.1 Workspace layout | ADR-001 | ADR.md |

---

## Appendix B: Key Invariants Summary

For quick reference, the full set of hard invariants enforced at compile time or CI time:

| Invariant | Rule | Enforcement Level |
|---|---|---|
| No floats in simulation | `clippy::float_arithmetic` deny | Compile-time |
| No unseeded RNG | CI grep scan | CI gate |
| No HashMap in sim state | `sim_hashmap_forbidden` lint | Pre-commit + CI |
| No SystemTime in sim | CI grep scan | CI gate |
| No cycles in crate deps | `tach check` | CI gate |
| Sorted entity iteration | Code pattern + review | Code review |
| Joule conservation | proptest property | Test suite |
| Ledger balance non-negative | `Ledger::transfer` checks | Runtime assertion |
| Phase order fixed | `PhaseSchedule` ordering | Compile-time (enum) |
| State hash per tick | `compute_state_hash` always called | Test coverage |

---

## Appendix C: Version History

| Version | Date | Changes |
|---|---|---|
| 1.0 | (earlier) | 32-line stub scaffold |
| 2.0 | 2026-02-21 | Full engineering-grade expansion. ~2,700+ lines. All sections per spec brief. Derived from CIV-0001, CIV-0200, PLAN.md, ADR.md. |


---

## Source: models/civ-sim/USER_SPEC.md

# CivLab Civ-Sim User Specification

**Document Status:** Draft v0.1
**Last Updated:** 2026-02-21
**Owner:** CivLab Product Engineering
**Audience:** UX Designers, Frontend Engineers, Backend Engineers, QA

---

## Table of Contents

1. [Overview and User Philosophy](#1-overview-and-user-philosophy)
2. [User Personas](#2-user-personas)
3. [Core User Flows](#3-core-user-flows)
4. [UX Requirements (FR Format)](#4-ux-requirements-fr-format)
5. [Information Architecture](#5-information-architecture)
6. [Interaction Patterns](#6-interaction-patterns)
7. [CLI Interface](#7-cli-interface)
8. [API Interface](#8-api-interface)
9. [Accessibility and Localization](#9-accessibility-and-localization)
10. [Acceptance Criteria](#10-acceptance-criteria)

---

## 1. Overview and User Philosophy

### 1.1 What CivLab Is

CivLab is a headless deterministic civilization simulation engine. Its purpose is to serve as a rigorous analytical instrument — not an entertainment product. Users reason about civilizational dynamics: energy economics, institutional stability, climate feedback, social cohesion, conflict, and diplomacy. Every output the system produces must be traceable, reproducible, and explainable.

The engine simulates at 100ms per tick, uses ChaCha20Rng for all stochastic sampling, and generates a BLAKE3 hash of full simulation state on every tick. Determinism is not optional — it is the foundation on which user trust is built. A user who cannot replay a run and get the same result cannot trust any result.

### 1.2 Design Principles

**Principle 1: Provenance First**
Every number, chart, and table presented to a user must carry an unambiguous trail back to the scenario configuration, seed, and tick range that produced it. The system must never present outputs that cannot be reconstructed.

**Principle 2: Determinism Is a User Feature**
Users must be able to share a run ID and have any other user reproduce the exact simulation state. Run IDs encode the seed and config hash. Replay from any tick must be a first-class operation, not a debugging affordance.

**Principle 3: Fail Loudly**
If a simulation encounters an unstable state, a determinism violation, or a configuration error, it surfaces this immediately with specific detail. The system does not attempt graceful degradation. It halts and explains.

**Principle 4: Minimize Cognitive Overhead for Branching**
Counterfactual reasoning — "what if we had applied intervention X at tick 3000?" — is a primary analytical operation. The UI must make branching and comparing branches as direct as possible. No accidental destruction of branches. No hidden history.

**Principle 5: CLI and API Are First-Class**
Research Operators run hundreds of parameter sweeps. The CLI and JSON-RPC API must be as capable and ergonomic as the web UI. Nothing available in the web UI should be unavailable via CLI or API.

**Principle 6: Explicit Assumptions**
Scenario configurations encode explicit assumptions. The UI must surface these assumptions throughout the workflow — when creating, when viewing outputs, and when exporting. Outputs without visible assumptions are analytically worthless.

**Principle 7: Modular Extension Without Trust Violations**
Mods execute in a wasmtime 26.x WASM sandbox. Mod authors have a defined SDK surface. The system must make it clear to users when a mod is active in a run, and what that mod's claimed behavior is.

### 1.3 Who This System Serves

CivLab serves three primary user populations and one secondary population:

| Population | Primary Need | Primary Surface |
|---|---|---|
| Scenario Designer | Author and version simulation scenarios | Web UI (scenario editor) |
| Policy Analyst | Compare policy regimes; produce reports | Web UI (comparison panels, export) |
| Research Operator | Batch sweeps, statistical aggregation, automation | CLI + REST/WebSocket API |
| Modder / Extension Developer | Author WASM mods; test against engine | CLI + civlab-sdk crate + local test harness |

These populations have overlapping but distinct tool preferences, workflow patterns, and output requirements. The system must serve all four without forcing any of them into workflows optimized for another group.

### 1.4 Non-Goals

This specification does not cover:

- Entertainment gameplay features (no victory conditions, no player agency in the game-design sense)
- Real-time multiplayer simulation
- Cloud infrastructure provisioning
- Commercial licensing workflows
- Mobile client implementation details (future)
- Bevy 3D desktop client implementation details (future)

---

## 2. User Personas

### 2.1 Persona A: Scenario Designer

#### 2.1.1 Background

**Name (representative):** Maren
**Role:** Computational social scientist or policy simulation architect
**Technical level:** High. Comfortable with TOML/JSON config authoring, version control concepts, and command-line tools. May not be a software engineer but understands structured data.
**Context:** Works at a think tank, university research center, or government modeling office. Produces scenarios that other analysts then run and study.

#### 2.1.2 Goals

1. Author scenario configurations that encode explicit, peer-reviewable assumptions about initial civilization state, policy parameters, and environmental conditions.
2. Iterate rapidly on scenario variants (e.g., high-energy vs. low-energy starting conditions) without duplicating work.
3. Version and archive scenarios so that published analyses can reference a specific, immutable scenario version.
4. Validate scenario configurations before publishing them to ensure they are well-formed, deterministic-safe, and within engine parameter bounds.
5. Document the rationale for specific parameter choices directly within the scenario artifact.

#### 2.1.3 Pain Points (Without CivLab)

- Scenarios exist as ad hoc TOML files with no enforced schema, leading to silent misconfiguration.
- No versioning: a scenario file changed after analysis invalidates prior results with no audit trail.
- Validation is a manual trial-and-error process against the engine.
- Sharing scenarios requires sharing raw config files with no guarantee of reproducibility.
- Rationale for parameter choices lives in separate documents that drift out of sync with configs.

#### 2.1.4 Key Tasks

| Task | Frequency | Criticality |
|---|---|---|
| Create new scenario from template | 2-5x/week | High |
| Edit scenario parameters | Daily | High |
| Validate scenario config | Every edit session | Critical |
| Save scenario version (immutable snapshot) | Weekly or at milestone | Critical |
| Fork scenario to create variant | 1-3x/week | High |
| Add inline rationale/annotation to parameters | Per editing session | Medium |
| Browse scenario version history | 1-2x/week | Medium |
| Publish scenario to shared workspace | 1-2x/month | High |
| Diff two scenario versions | As needed | Medium |

#### 2.1.5 Workflow Patterns

**Pattern 1: Template-to-Scenario**
Maren starts from a curated engine template (e.g., "Pre-Industrial Island State") and modifies parameters section by section. She uses the schema-aware editor to catch out-of-range values immediately. She saves draft frequently and commits a named version when satisfied.

**Pattern 2: Variant Branching**
Maren takes a validated scenario and forks it to create a variant that differs in one policy cluster (e.g., trade liberalization on vs. off). She wants the diff between fork and parent to be explicit and reviewable.

**Pattern 3: Collaborative Review**
Maren shares a scenario version link with a colleague. The colleague must be able to load the exact configuration with no ambiguity.

#### 2.1.6 UI Surface Requirements

- Schema-aware TOML/JSON editor with inline validation and error messages referencing the parameter name and valid range.
- Scenario version history panel with diff view between any two versions.
- Inline annotation fields per parameter group (free-text, markdown-rendered).
- Template library browser with parameter preview and one-click fork.
- Scenario publish/share workflow producing a stable reference URL encoding the scenario hash.
- Fork origin display on all scenario views (shows parent version).

#### 2.1.7 Success Criteria

- Maren can create a valid, versioned scenario from a template in under 10 minutes.
- Any saved scenario version is immutable and permanently replayable.
- Validation errors are actionable (specific parameter name, current value, valid range, suggested fix).
- Diff between two scenario versions is readable without external tooling.
- Scenario reference URL is stable and can be cited in a published document.

---

### 2.2 Persona B: Policy Analyst

#### 2.2.1 Background

**Name (representative):** Tariq
**Role:** Policy analyst, economist, or political scientist
**Technical level:** Medium. Proficient with spreadsheets, data visualization tools, and can read JSON. Not a programmer.
**Context:** Works at a policy institute, legislative research office, or international organization. Uses CivLab to evaluate civilizational outcomes under different policy regimes and produce decision-support reports.

#### 2.2.2 Goals

1. Run pre-authored scenarios (produced by Scenario Designers) and inspect outcomes.
2. Compare two or more policy regimes side-by-side across multiple metrics.
3. Understand the causal chain behind a specific outcome (why did legitimacy collapse at tick 4200?).
4. Produce professional-grade export packages containing charts, data tables, assumption disclosures, and narrative annotations.
5. Annotate specific moments in a simulation timeline with interpretive commentary.
6. Share findings with non-technical stakeholders in formats that do not require CivLab access.

#### 2.2.3 Pain Points (Without CivLab)

- Comparing policy regimes requires manually aligning outputs from different simulation runs in a spreadsheet.
- Charts have no provenance — cannot verify what scenario or parameters produced them.
- Timeline inspection is not possible; only endpoint summaries are available.
- Export artifacts are not self-contained; they require the reader to have access to raw data files.
- Assumptions are not disclosed in exported reports.

#### 2.2.4 Key Tasks

| Task | Frequency | Criticality |
|---|---|---|
| Browse available scenarios | Daily | High |
| Run scenario (trigger engine) | Daily | Critical |
| Inspect simulation timeline | Daily | High |
| Navigate hex map at two LOD levels | Daily | Medium |
| Add annotation to timeline event | 3-5x/week | Medium |
| Open comparison panel (2+ runs) | 3-5x/week | High |
| Select metrics for comparison chart | Per comparison session | High |
| Export report package | 1-2x/week | Critical |
| Share run reference with colleague | 1-2x/week | High |
| Investigate metric anomaly (drill down) | 2-3x/week | High |

#### 2.2.5 Workflow Patterns

**Pattern 1: Baseline vs. Intervention**
Tariq runs the baseline scenario to completion, then runs the same scenario with an intervention applied at a specific tick. He opens the comparison panel, selects the metrics he cares about (tyranny, legitimacy, trade balance), and reviews the divergence curve.

**Pattern 2: Policy Regime Matrix**
Tariq compares four runs simultaneously: low tax + open trade, low tax + closed trade, high tax + open trade, high tax + closed trade. He produces a summary table showing endpoint metrics for each regime and exports it as a report package.

**Pattern 3: Causal Chain Investigation**
A metric spikes unexpectedly. Tariq scrubs the timeline to the spike point, inspects the event log at that tick, and follows the causal chain backward through prior ticks.

#### 2.2.6 Multi-Run Comparison Requirements

- Side-by-side metric charts with aligned time axes.
- Metric selector: choose any combination of the eight core metrics (waste, surplus, tyranny, legitimacy, resilience, Joule stock, trade balance, population).
- Delta chart mode: show difference between runs rather than absolute values.
- Divergence point detection: automatic annotation of the first tick where two runs diverge beyond a configurable threshold.
- Regime label assignment: user can name each run for display purposes (e.g., "Open Trade", "Closed Trade").

#### 2.2.7 Export Requirements

- Export formats: PDF report (human-readable), JSON bundle (machine-readable), CSV (tabular metrics), Parquet (analytical datasets).
- Report bundle must include: scenario config hash, run ID, seed, parameter assumptions, all charts as vector SVG, all data tables as embedded CSV, user annotations, generation timestamp, engine version.
- All exported charts must embed their provenance in SVG metadata.
- PDF report must include a cover page with assumption disclosure section.
- Export package must be cryptographically signed (BLAKE3 hash of bundle contents) for integrity verification.

#### 2.2.8 Annotation Requirements

- Annotations attach to a specific tick range and optionally to a specific metric.
- Annotations are free-text with Markdown rendering.
- Annotations appear in the timeline view as markers and in the exported report.
- Annotations are saved per-run, not per-scenario. A scenario run can have multiple annotation sets (one per analyst).
- Annotations must be exported with the report and attributed to the author.

#### 2.2.9 Success Criteria

- Tariq can produce a side-by-side comparison of two runs within 5 minutes of run completion.
- Exported report is self-contained: a reader with no CivLab access can verify what assumptions produced the charts.
- Timeline scrubbing to a specific tick is responsive (under 200ms from drag to render).
- Metric anomalies surface in the event log with causal attribution.
- Report PDF renders without external fonts or assets.

---

### 2.3 Persona C: Research Operator

#### 2.3.1 Background

**Name (representative):** Priya
**Role:** Quantitative researcher, computational economist, or simulation engineer
**Technical level:** Very high. Comfortable with shell scripting, Python/R data analysis, distributed compute, and structured output formats.
**Context:** Runs large-scale parameter sweeps to characterize simulation sensitivity, build confidence intervals over stochastic draws, and produce machine-readable datasets for downstream analysis pipelines.

#### 2.3.2 Goals

1. Execute parameter sweeps over a scenario config with a defined parameter grid.
2. Run multiple seeds per parameter combination to produce distributions over stochastic outcomes.
3. Aggregate outputs into machine-readable datasets (CSV, JSON, Parquet) without manual intervention.
4. Detect and flag parameter combinations that produce unstable or degenerate simulation states.
5. Integrate CivLab into automated pipelines (CI/CD, Jupyter notebooks, Python analysis scripts).
6. Control resource usage: max concurrent runs, memory limits, disk quotas.

#### 2.3.3 Pain Points (Without CivLab)

- No native sweep command; sweeps require custom shell scripts with manual job management.
- No structured output format; must parse engine logs manually.
- No automatic stability detection; degenerate runs silently corrupt aggregate statistics.
- No confidence interval tooling; statistical analysis happens entirely outside the engine.
- Parallelism is limited by manual process management.

#### 2.3.4 CLI-First Expectations

Priya expects:

- A single `civlab sweep` command that accepts a sweep manifest file.
- Progress reporting on stdout in a machine-parseable format (JSON lines or structured text).
- Run artifacts written to a configurable output directory with a predictable naming convention.
- Exit codes that distinguish success, partial failure (some runs failed), and total failure.
- A `civlab aggregate` command that ingests a sweep output directory and produces summary statistics.
- Man pages or `--help` output that is complete and accurate.

#### 2.3.5 Headless and API Usage Patterns

- Priya invokes CivLab from Python scripts via subprocess or the REST API.
- She expects JSON output from all CLI commands when `--format json` is specified.
- She submits batch run requests via the REST API and polls for completion.
- She streams tick-level data from running simulations via the WebSocket JSON-RPC API.
- She uses the API to inject interventions programmatically during a live run.

#### 2.3.6 Output Format Requirements

| Format | Use Case | Required Fields |
|---|---|---|
| JSON Lines | Per-tick metric stream | tick, seed, run_id, metric_name, value |
| CSV | Endpoint summary per run | run_id, seed, param_key, param_value, metric_name, endpoint_value |
| Parquet | Full time-series dataset | All CSV fields + tick column + all metric columns |
| JSON bundle | Full run artifact | run_id, scenario_hash, config, seed, tick_hashes[], metrics_by_tick{} |

#### 2.3.7 Multi-Run Aggregation and Statistical Analysis

- The `civlab aggregate` command computes: mean, median, p5, p25, p75, p95, std dev, and IQR per metric per parameter combination.
- Degenerate run detection: runs where any core metric reaches `NaN`, `Inf`, or a predefined instability threshold are flagged and excluded from aggregates with a warning.
- Confidence interval output: 95% CI reported alongside each aggregate.
- Sensitivity analysis output: Spearman rank correlation between each swept parameter and each endpoint metric.
- All aggregate outputs available as CSV, JSON, and Parquet.

#### 2.3.8 Success Criteria

- A sweep of 1000 runs executes without requiring any manual intervention.
- Degenerate runs are automatically excluded and logged; aggregate statistics are correct over the remaining runs.
- All outputs land in a predictable directory structure that can be committed to a data repository.
- Integration into a Python analysis script requires no more than 10 lines of subprocess/API calls.
- `civlab --help` is sufficient to complete a sweep without reading external documentation.

---

### 2.4 Persona D: Modder / Extension Developer

#### 2.4.1 Background

**Name (representative):** Ola
**Role:** Software engineer, researcher, or advanced user building domain-specific extensions
**Technical level:** Very high. Comfortable with Rust or AssemblyScript targeting WASM, the civlab-sdk crate, and the engine's hook system.
**Context:** Building a custom Policy mod, Economic model extension, Event generator, or Scenario template that extends engine behavior within the WASM sandbox.

#### 2.4.2 Goals

1. Author WASM mods using the civlab-sdk crate with a clear, stable API surface.
2. Test mods locally against the engine before publishing to the registry.
3. Understand the sandbox constraints (what the mod can and cannot do).
4. Distribute mods via the CivLab mod registry with a signed artifact.
5. Inspect which mods are active in a given run and what they affect.

#### 2.4.3 WASM Sandbox Constraints (visible to user)

| Constraint | Value |
|---|---|
| WASM runtime | wasmtime 26.x |
| Mod types | Policy, Economic, Event, Scenario |
| Max WASM memory | 256 MB per mod instance |
| Max execution time per hook | 10ms |
| Host imports allowed | civlab_sdk::host::* (enumerated in SDK docs) |
| Forbidden | File I/O, network, random (must use engine-provided RNG) |
| Determinism requirement | Mods must be pure functions of their inputs |

#### 2.4.4 SDK Workflow

1. Install civlab-sdk crate (`cargo add civlab-sdk`).
2. Implement the appropriate mod trait (e.g., `PolicyMod`, `EconomicMod`).
3. Build to WASM target (`cargo build --target wasm32-wasip2`).
4. Run local test harness (`civlab mod test ./my_mod.wasm --scenario examples/island_state.toml`).
5. Inspect test output: hook call counts, metric effects, determinism verification.
6. Package and sign (`civlab mod package ./my_mod.wasm --sign`).
7. Publish to registry (`civlab mod publish ./my_mod.civmod`).

#### 2.4.5 UI Surface Requirements

- Mod browser panel in web UI: browse registry, view mod metadata, enable/disable mods per run.
- Active mod list visible in run header (any run with mods active displays a mod badge).
- Mod hook call log in developer mode: shows which hooks fired, execution time, and claimed effects.
- Mod integrity verification status displayed per-mod (signature valid / signature invalid / unsigned).

#### 2.4.6 Success Criteria

- A competent Rust developer can author, test, and publish a minimal Policy mod in under one day.
- Mods that violate sandbox constraints fail at load time with a specific error message.
- Any run using mods is labeled in all exports and reports.
- Mod-active runs are still fully replayable, provided the same mod WASM artifact is available.

---

## 3. Core User Flows

### 3.1 Flow 1: Create Scenario → Validate → Save Version

#### 3.1.1 Description

The Scenario Designer authors a new scenario configuration, validates it against the engine schema and parameter bounds, and saves an immutable versioned snapshot.

#### 3.1.2 Preconditions

- User is authenticated and has write access to a workspace.
- Engine schema is loaded and available for validation.

#### 3.1.3 Step-by-Step

**Step 1: Initiate scenario creation**
User navigates to the Scenarios panel and selects "New Scenario". System presents two paths: (a) Start from blank, (b) Browse template library.

**Step 2: Select template (if applicable)**
User browses the template library. Templates are categorized by starting civilization type (Island State, Continental Empire, Archipelago Federation, etc.). Each template shows a parameter preview panel with key metrics. User selects a template and clicks "Fork as New Scenario".

**Step 3: Open scenario editor**
System opens the schema-aware scenario editor. The editor displays the TOML configuration with section panels: Initial State, Policy Parameters, Climate Parameters, Institutional Parameters, RNG Seed.

**Step 4: Edit parameters**
User modifies parameters. The editor validates each field against the schema on change (not on submit). Validation errors appear inline below the field with the constraint violated and the valid range. Out-of-range values are highlighted in amber. Schema-invalid values are highlighted in red.

**Step 5: Add annotations**
User opens the Annotations panel. Annotations can be added per parameter section. The user enters free-text rationale for parameter choices. Annotations are stored alongside the config.

**Step 6: Run validation**
User clicks "Validate". The system sends the full config to the engine validation endpoint. Validation checks: (a) schema conformance, (b) parameter range bounds, (c) cross-parameter constraint satisfaction, (d) determinism precondition checks (no floating-point dependencies on external state). Validation result is displayed as a structured report.

**Step 7: Address validation errors**
If validation fails, the error report lists each failed check with the parameter path, current value, and the violated constraint. User returns to editor to fix errors. Repeat from Step 4.

**Step 8: Save draft**
User saves the current state as a draft. Drafts are mutable. The system assigns a draft ID and displays a draft indicator in the scenario header.

**Step 9: Commit version**
When satisfied, user clicks "Commit Version". System requires a version label (e.g., "v1.0-baseline") and an optional commit message. System computes BLAKE3 hash of the config, stores an immutable snapshot, and assigns a version ID of the form `{scenario_id}@{config_hash_prefix}`. The committed version cannot be modified.

**Step 10: Confirm version saved**
System displays the version ID, config hash, and timestamp. A stable reference URL is generated. The version appears in the scenario version history panel.

#### 3.1.4 Error States and Recovery

| Error | System Response | User Action |
|---|---|---|
| Schema validation failure | Inline error on field, validation report | Fix parameter, re-validate |
| Parameter out of range | Amber highlight, range tooltip | Adjust to valid range |
| Cross-parameter constraint failure | Validation report with constraint description | Adjust dependent parameters |
| Version label collision | Error: "Version label already exists" | Choose a different label |
| Network error during commit | Error banner, draft preserved | Retry commit |

#### 3.1.5 ASCII Flow Diagram

```
User                      Web UI                     Engine Validation
 |                           |                               |
 |-- "New Scenario" -------> |                               |
 |                           |-- Show template browser ----> |
 |-- Select template ------> |                               |
 |                           |-- Fork config, open editor -> |
 |-- Edit parameters ------> |                               |
 |                           |-- Inline validate (schema) -> |
 |                           |<-- Validation result -------- |
 |<-- Inline errors -------- |                               |
 |-- Fix parameters -------> |                               |
 |-- Add annotations ------> |                               |
 |-- "Validate" -----------> |                               |
 |                           |-- Full validation request --> |
 |                           |<-- Validation report --------- |
 |<-- Validation report ----- |                               |
 |                           |                               |
 |   [If errors: fix and     |                               |
 |    repeat from Edit]      |                               |
 |                           |                               |
 |-- "Commit Version" ------> |                               |
 |   (label + message)       |-- Compute config BLAKE3 ----> |
 |                           |-- Store immutable snapshot -> |
 |                           |-- Assign version ID --------> |
 |<-- Version confirmed ------ |                               |
 |   (version_id, hash, URL) |                               |
```

---

### 3.2 Flow 2: Run Simulation → Inspect Timeline → Compare Against Baseline

#### 3.2.1 Description

The Policy Analyst runs a scenario version, inspects the resulting simulation timeline at two levels of detail (strategic hex view and city-level operational view), and compares it against a baseline run side-by-side.

#### 3.2.2 Preconditions

- At least one committed scenario version exists.
- A baseline run for the scenario exists (or the user designates one after running).

#### 3.2.3 Step-by-Step

**Step 1: Select scenario version**
User navigates to the Scenario panel, selects a committed scenario version, and clicks "Run". System presents the run configuration dialog: seed input (auto-generated or user-specified), optional mod selection, run label.

**Step 2: Submit run**
User confirms the run configuration and submits. System creates a run record with a run ID of the form `{scenario_hash}-{seed}-{timestamp}`. Engine begins executing. Run status indicator appears in the run list: Queued → Running → Complete.

**Step 3: Monitor run progress**
While running, the Progress panel shows: current tick, estimated completion, tick rate (ticks/second), current metric snapshots updated every 50 ticks. User can close the panel; the run continues in the background.

**Step 4: Open completed run**
User clicks on a completed run. System loads the timeline view. Default state: strategic hex map centered on the simulation world, timeline scrubber positioned at tick 0, metric panel showing all eight core metrics as time-series sparklines.

**Step 5: Navigate hex map (strategic view)**
User pans the hex map with click-drag. Zoom with scroll wheel or pinch. At strategic zoom, hex tiles show aggregated data: color-coded by dominant metric (e.g., legitimacy gradient from green to red). Hovering a hex tile shows a tooltip with aggregated metrics for that tile's region.

**Step 6: Descend to operational view**
User double-clicks a hex tile or uses the "Zoom to City" button. System transitions to operational (city-level) LOD. City view shows: individual citizen agents (aggregated by cohort), building footprints, resource flows, institution status indicators. A breadcrumb shows the current LOD level and location.

**Step 7: Scrub timeline**
User drags the timeline scrubber to a specific tick. System updates all views (hex map, metric charts, event log) to reflect state at that tick. Scrubbing is frame-synced: display updates within 200ms of scrubber position change.

**Step 8: Inspect event log**
The event log panel shows all events fired at the current tick and surrounding window (±5 ticks). Events include: policy changes, resource threshold crossings, institution stability changes, conflict onset, diplomatic events. Each event has a causal attribution field showing what triggered it.

**Step 9: Annotate a moment**
User right-clicks on the timeline at a specific tick and selects "Add Annotation". A text field appears. User enters commentary. Annotation marker appears on the timeline.

**Step 10: Open comparison panel**
User clicks "Compare" in the toolbar. The comparison panel opens. User selects the baseline run from the run list. System loads both runs into the comparison view.

**Step 11: Configure comparison**
User selects which metrics to compare (checkbox list). User selects display mode: Absolute (both time series on same chart) or Delta (difference between runs on a single chart). Divergence point detection runs automatically and marks the first tick of significant divergence.

**Step 12: Interpret comparison**
Both runs are displayed with aligned time axes. The baseline run is shown in a muted color; the comparison run in a prominent color. Regime labels assigned by user appear in the legend. The divergence point annotation displays the metric that diverged first and the magnitude.

#### 3.2.4 ASCII Flow Diagram

```
User                      Web UI                     Engine / API
 |                           |                               |
 |-- Select scenario ver. -> |                               |
 |-- Configure run --------> |                               |
 |   (seed, mods, label)     |-- Submit run request -------> |
 |                           |<-- run_id assigned ---------- |
 |                           |                               |
 |<-- Run status: Queued --- |   [Engine executing]          |
 |<-- Run status: Running -- |<-- Progress stream (WS) ----- |
 |<-- Run status: Complete - |<-- Completion event (WS) ---- |
 |                           |                               |
 |-- Open completed run ----> |                               |
 |                           |-- Load timeline data -------> |
 |<-- Timeline view loaded - |                               |
 |                           |                               |
 |-- Pan/zoom hex map ------> |                               |
 |                           |-- LOD switch (if needed) ---> |
 |<-- Updated map state ----- |                               |
 |                           |                               |
 |-- Scrub timeline --------> |                               |
 |                           |-- Fetch tick state ----------> |
 |<-- All views updated ----- |                               |
 |                           |                               |
 |-- "Compare" ------------- > |                               |
 |-- Select baseline run ---> |                               |
 |                           |-- Load baseline run data ---> |
 |<-- Comparison panel ------- |                               |
 |                           |                               |
 |-- Select metrics --------> |                               |
 |-- Select display mode ---> |                               |
 |<-- Comparison charts ------ |                               |
 |<-- Divergence annotation - |                               |
```

---

### 3.3 Flow 3: Trigger Intervention → Replay Branch → Evaluate Metric Deltas

#### 3.3.1 Description

The Policy Analyst (or Scenario Designer) pauses a simulation at a specific tick, applies a counterfactual intervention, runs a new branch from that tick forward, and evaluates how outcomes diverge from the original run.

#### 3.3.2 Preconditions

- An original run exists and is complete.
- The user has write access to the workspace.

#### 3.3.3 Step-by-Step

**Step 1: Navigate to intervention point**
User opens a completed run in the timeline view. User scrubs to the tick where they want to apply the intervention (e.g., tick 3000).

**Step 2: Open intervention panel**
User right-clicks on the timeline at tick 3000 and selects "Create Intervention Branch". Alternatively, user clicks the "Branch" button in the toolbar and enters a tick number. System opens the Intervention Panel.

**Step 3: Define intervention**
The Intervention Panel presents a structured form for the supported intervention types:
- Policy parameter override: select parameter, enter new value (validated against schema bounds).
- Institution shock: select institution, apply a legitimacy delta.
- Resource injection/removal: select resource pool, enter delta value.
- Climate event injection: select event type, duration, and severity.
- Diplomatic state change: select actor pair, change relation state.

User configures the intervention. The panel shows a preview of what changes will be applied at the branch tick.

**Step 4: Name the branch**
User enters a branch label (e.g., "Trade Liberalization at Tick 3000"). System appends a branch ID derived from the parent run ID and the intervention hash.

**Step 5: Submit branch run**
User clicks "Run Branch". System forks the engine state at tick 3000 (using the tick state hash to reconstruct state exactly), applies the intervention, and runs forward with the same RNG seed continuation.

**Step 6: Monitor branch run**
Branch run appears in the run list under the parent run, visually indented to show the branch relationship. Progress indicator shows the branch run status.

**Step 7: Open branch comparison**
Once complete, the system automatically opens the branch comparison view. The original run is shown as the "trunk" and the branch as the "fork". The divergence point (tick 3000) is marked on the timeline.

**Step 8: Evaluate metric deltas**
Delta chart mode is active by default: each metric chart shows (branch_value - trunk_value) over time. Positive delta is colored green; negative delta red. The summary panel shows the metric deltas at the final tick and at the user-annotated evaluation point.

**Step 9: Annotate findings**
User adds annotations to the branch explaining the intervention rationale and interpreting the delta patterns.

**Step 10: Export branch comparison**
User exports the branch comparison as a report package. The report includes both runs, the intervention specification, and the delta charts.

#### 3.3.4 Branching Model Details

- Branch point: any tick T where a tick state hash exists.
- Engine reconstructs state at tick T by replaying from seed to T (deterministic).
- Intervention is applied as a state patch at T+1.
- Branch run uses the same ChaCha20Rng stream advanced to the post-T position, then diverges based on the patched state.
- Branch ID: `{parent_run_id}@branch-T{tick}-{intervention_hash_prefix}`.
- Branches can themselves be branched (tree structure, not just fork).
- Maximum branch depth: 10 (engine limit, surfaced as a UI constraint).

#### 3.3.5 ASCII Flow Diagram

```
User                      Web UI                     Engine / API
 |                           |                               |
 |-- Open completed run ----> |                               |
 |-- Scrub to tick T -------> |                               |
 |-- "Create Intervention  -> |                               |
 |   Branch"                 |                               |
 |                           |                               |
 |-- Define intervention ---> |                               |
 |   (type, params, values)  |-- Validate intervention ----> |
 |                           |<-- Validation result --------- |
 |<-- Preview patch --------- |                               |
 |                           |                               |
 |-- Name branch -----------> |                               |
 |-- "Run Branch" ----------> |                               |
 |                           |-- Fork state at tick T ------> |
 |                           |   (reconstruct via replay)    |
 |                           |-- Apply intervention patch --> |
 |                           |-- Run branch forward -------> |
 |<-- Branch progress -------- |<-- Progress stream (WS) ---- |
 |<-- Branch complete -------- |<-- Completion event (WS) --- |
 |                           |                               |
 |                           |-- Load both runs -----------> |
 |<-- Branch comparison view- |                               |
 |   (trunk + fork aligned)  |                               |
 |                           |                               |
 |-- Review delta charts ---> |                               |
 |-- Add annotations -------> |                               |
 |-- Export report ----------> |                               |
 |                           |-- Generate signed bundle ---> |
 |<-- Download report -------- |                               |
```

---

### 3.4 Flow 4: Export Report Package

#### 3.4.1 Description

The Policy Analyst produces a self-contained, signed export package containing all assumptions, outputs, charts, and annotations from one or more runs.

#### 3.4.2 Step-by-Step

**Step 1: Initiate export**
User opens a run or comparison view and clicks "Export". The Export dialog opens.

**Step 2: Configure export**
Export dialog options:
- Export format: PDF Report, JSON Bundle, CSV Tables, Parquet Dataset, or All (ZIP containing all formats).
- Content scope: Current run only, Selected runs, All runs in comparison panel.
- Include charts: Yes/No (PDF and JSON only).
- Include raw tick data: Yes/No (adds tick-level metric time series).
- Include annotations: Yes/No.
- Include assumption disclosure: always included, cannot be disabled.
- Signing: always applied (BLAKE3 hash of bundle, displayed as artifact fingerprint).

**Step 3: Generate export**
User clicks "Generate". System assembles the bundle. Progress indicator shows assembly stages: Collecting run data, Rendering charts, Compiling tables, Signing artifact.

**Step 4: Download**
Download link appears. User downloads the bundle. The bundle filename encodes: `civlab-export-{run_id_prefix}-{timestamp}.{ext}`.

**Step 5: Verify integrity (optional)**
User can verify the bundle's BLAKE3 hash using `civlab verify {bundle_file}`. Output: PASS or FAIL with the expected and actual hashes.

#### 3.4.3 Report Bundle Format (JSON)

```json
{
  "civlab_export_version": "1.0",
  "generated_at": "2026-02-21T14:30:00Z",
  "engine_version": "0.9.0",
  "artifact_fingerprint": "<BLAKE3 hex>",
  "runs": [
    {
      "run_id": "abc123def456-seed42-20260221",
      "scenario_hash": "<BLAKE3 hex>",
      "scenario_label": "Pre-Industrial Island State v1.2",
      "seed": 42,
      "tick_count": 10000,
      "tick_hashes": ["<BLAKE3>", "..."],
      "config": { /* full scenario TOML as JSON */ },
      "assumptions": { /* annotated parameter rationale */ },
      "mods_active": [],
      "metrics_summary": { /* endpoint values per metric */ },
      "metrics_timeseries": { /* tick-indexed per metric */ },
      "annotations": [
        {
          "author": "Tariq",
          "tick_range": [4200, 4250],
          "metric": "legitimacy",
          "text": "Legitimacy collapse triggered by tax shock at tick 4195"
        }
      ]
    }
  ],
  "charts": [
    {
      "chart_id": "legitimacy-timeseries",
      "run_ids": ["abc123def456-seed42-20260221"],
      "metric": "legitimacy",
      "format": "svg",
      "data_uri": "data:image/svg+xml;base64,..."
    }
  ]
}
```

---

### 3.5 Flow 5: Parameter Sweep (Research Operator)

#### 3.5.1 Description

The Research Operator executes a large-scale parameter sweep over a scenario configuration, collects results, and produces an aggregated statistical dataset.

#### 3.5.2 Sweep Manifest Format

```toml
# sweep_manifest.toml
[sweep]
scenario = "scenarios/island_state_v1.2.toml"
seeds = [1, 2, 3, 4, 5]           # 5 seeds per parameter combination
max_concurrent = 8                  # parallel run limit
output_dir = "./sweep_output"
output_formats = ["csv", "parquet", "json"]
stability_check = true              # exclude degenerate runs

[[sweep.parameters]]
name = "policy.tax_rate"
type = "linspace"
min = 0.05
max = 0.40
steps = 8

[[sweep.parameters]]
name = "policy.trade_openness"
type = "values"
values = [0.0, 0.5, 1.0]

[sweep.metrics]
endpoint_ticks = [5000, 10000]
time_series = true
```

#### 3.5.3 CLI Execution

```bash
# Run the sweep
civlab sweep --manifest sweep_manifest.toml

# With progress output
civlab sweep --manifest sweep_manifest.toml --progress

# Dry run (validate manifest, count runs)
civlab sweep --manifest sweep_manifest.toml --dry-run

# Resume interrupted sweep
civlab sweep --manifest sweep_manifest.toml --resume ./sweep_output
```

#### 3.5.4 Output Directory Structure

```
sweep_output/
  manifest.toml                   # Copy of sweep manifest
  sweep_id.txt                    # Unique sweep ID
  runs/
    {run_id}/
      config.toml                 # Effective config for this run
      metrics_timeseries.parquet  # Tick-level metrics
      metrics_endpoint.json       # Endpoint summary
      tick_hashes.bin             # BLAKE3 hashes per tick
      status.json                 # Run status and metadata
  aggregate/
    summary.csv                   # Mean/CI per parameter combo
    sensitivity.csv               # Spearman correlations
    degenerate_runs.json          # Flagged unstable runs
    full_dataset.parquet          # All runs, all metrics, all ticks
```

#### 3.5.5 Aggregation Command

```bash
# Aggregate completed sweep
civlab aggregate --input ./sweep_output --output ./analysis

# Specific aggregation options
civlab aggregate \
  --input ./sweep_output \
  --output ./analysis \
  --metrics legitimacy,tyranny,resilience \
  --ci 0.95 \
  --exclude-degenerate \
  --format parquet,csv
```

#### 3.5.6 ASCII Flow Diagram

```
Research Operator           civlab CLI                  Engine Pool
 |                               |                           |
 |-- Write sweep_manifest.toml   |                           |
 |-- civlab sweep --manifest --> |                           |
 |                               |-- Validate manifest ----> |
 |                               |-- Compute run matrix ---> |
 |                               |   (params x seeds)        |
 |<-- Dry run count: 120 runs -- |                           |
 |-- (confirm, run for real) --> |                           |
 |                               |-- Spawn workers (N=8) --> |
 |                               |   Per worker:             |
 |                               |   - Submit run            |
 |                               |   - Poll completion       |
 |                               |   - Write artifacts       |
 |<-- Progress: [========  ] --> |<-- Run completions ------- |
 |   (runs done / total)         |                           |
 |                               |-- [All runs complete]     |
 |<-- Sweep complete ----------- |                           |
 |   (summary: N ok, M skipped)  |                           |
 |                               |                           |
 |-- civlab aggregate ----------> |                           |
 |   --input ./sweep_output      |-- Read run artifacts      |
 |                               |-- Compute statistics      |
 |                               |-- Detect degenerate runs  |
 |                               |-- Write aggregate outputs |
 |<-- Aggregation complete ------- |                           |
 |   (summary.csv, full.parquet) |                           |
```

---

### 3.6 Flow 6: Mod Authoring → Test → Publish

#### 3.6.1 Description

The Modder authors a WASM mod using the civlab-sdk crate, tests it locally, and publishes it to the mod registry.

#### 3.6.2 Step-by-Step

**Step 1: Initialize mod project**

```bash
civlab mod new --type policy --name my_tax_policy
# Creates:
#   my_tax_policy/
#     Cargo.toml        (civlab-sdk dep pre-configured)
#     src/lib.rs        (PolicyMod trait skeleton)
#     tests/            (test harness scaffold)
#     civmod.toml       (mod manifest)
```

**Step 2: Implement the mod trait**

Mod author implements the required trait in `src/lib.rs`. The SDK provides:
- `PolicyMod::apply(&self, state: &SimState, tick: u64) -> PolicyEffect`
- `EconomicMod::compute_production(&self, state: &SimState) -> ResourceDelta`
- `EventMod::should_fire(&self, state: &SimState, rng: &mut EngineRng) -> Option<Event>`

**Step 3: Build**

```bash
cargo build --target wasm32-wasip2 --release
# Output: target/wasm32-wasip2/release/my_tax_policy.wasm
```

**Step 4: Run local test harness**

```bash
civlab mod test \
  ./target/wasm32-wasip2/release/my_tax_policy.wasm \
  --scenario examples/island_state_v1.2.toml \
  --ticks 1000 \
  --verify-determinism \
  --verbose
```

Test harness output:
- Hook call counts per tick
- Metric effect summary
- Determinism verification (run twice, compare hashes)
- Sandbox constraint violations (if any)
- Execution time per hook call (flagged if > 10ms limit)

**Step 5: Package and sign**

```bash
civlab mod package \
  ./target/wasm32-wasip2/release/my_tax_policy.wasm \
  --manifest civmod.toml \
  --sign  # Uses local signing key from civlab keychain
# Output: my_tax_policy-0.1.0.civmod
```

**Step 6: Publish**

```bash
civlab mod publish ./my_tax_policy-0.1.0.civmod
# Submits to registry, awaits signature verification
# Returns: registry URL for mod
```

---

## 4. UX Requirements (FR Format)

### 4.1 Provenance Requirements

**FR-UX-001: Every chart must display its provenance.**
Every chart rendered in the web UI must display a provenance badge showing: run ID (truncated to 8 chars), scenario label and version, seed, and tick range. Clicking the provenance badge opens the full provenance panel with complete metadata. Provenance must be present in all exported chart SVGs as embedded metadata.

**FR-UX-002: Exported artifacts must include full assumption disclosure.**
Every export package (PDF, JSON, CSV, Parquet) must include the full scenario configuration used to produce it. This cannot be disabled by the user. The assumption disclosure section must appear on page 1 of the PDF report.

**FR-UX-003: Run IDs must be stable, unique, and human-readable.**
Run IDs must be deterministic from the inputs: `{scenario_hash_8char}-s{seed}-{ISO8601_compact}`. Format example: `a3f8b21c-s42-20260221T143000Z`. The same scenario + seed + timestamp always produces the same run ID. Run IDs must be displayed wherever a run is referenced.

**FR-UX-004: Replay references must be self-contained.**
A replay reference must encode everything needed to reproduce the run: scenario hash, seed, engine version. Format: `civlab://replay/{scenario_hash}/{seed}/{engine_version}`. The web UI must render replay references as copyable links. The CLI must accept replay references as arguments to `civlab replay`.

**FR-UX-005: Tick state hashes must be visible in timeline view.**
The timeline view must provide access to the BLAKE3 hash of each tick's state. Users must be able to copy a tick hash from the timeline. The tick hash must be included in exported artifacts.

### 4.2 Run ID and Replay Reference Display

**FR-UX-006: Run list must display run ID, scenario label, seed, status, and tick count.**
The run list panel must show these fields for each run in a scannable table format. Status is one of: Queued, Running, Complete, Failed, Cancelled. Completed runs show tick count and total wall clock time.

**FR-UX-007: Run header must be persistent during timeline inspection.**
When inspecting a run, a persistent header bar must display: run ID, scenario label, version, seed, and total tick count. The header must remain visible while scrolling the timeline or panning the hex map.

**FR-UX-008: Comparison panel must display regime labels prominently.**
When the user assigns regime labels to runs in a comparison, those labels must appear in chart legends, axis annotations, and the comparison summary panel. Labels must persist in exported reports.

### 4.3 Warning State System

**FR-UX-009: Low-confidence states must be surfaced with a visible warning indicator.**
If any metric enters a low-confidence state (defined as: the metric value is within the top or bottom 5% of the parameter's historical distribution across all completed runs, or the metric has exceeded a predefined instability threshold), the metric chart must display an amber warning icon and a tooltip explaining the cause. The timeline must show a low-confidence band overlay on the affected tick range.

**FR-UX-010: Determinism violations must trigger an immediate error state.**
If the engine detects a determinism violation (tick hash mismatch on replay), the affected run must be marked with a red "Determinism Violation" badge. All charts and tables from that run must display a prominent warning that data integrity is uncertain. The user must acknowledge the warning before proceeding. The violation must be logged with the specific tick, expected hash, and actual hash.

**FR-UX-011: Unstable simulation states must halt the run and surface an error.**
If the engine halts a run due to an unstable state (NaN, Inf, or degenerate metric value), the run status must show "Failed - Instability" with a detailed error panel showing: the tick of failure, the metric that triggered failure, the last known stable state hash, and a suggested diagnosis. The run must not be marked as Complete.

**FR-UX-012: Mod-active runs must display a persistent mod badge.**
Any run where mods are active must display a "Mods Active" badge in the run header. The badge must list the active mods with their registry names and versions. Clicking the badge opens the mod details panel. Mod-active status must appear in all exports.

**FR-UX-013: Configuration schema violations must block scenario commit.**
If the scenario config contains any schema violation, the "Commit Version" action must be disabled. The UI must display a count of blocking violations with a link to the validation report. Committing a schema-invalid scenario is not possible by any code path.

### 4.4 Cognitive Load Requirements for Branching and Rollback

**FR-UX-014: Branch origin must be persistently visible during branch run inspection.**
When inspecting a branch run, the timeline must display a vertical marker at the branch tick labeled with the branch label. The trunk run must be shown as a faded overlay in metric charts. The branch origin (parent run ID, branch tick) must be displayed in the branch run header.

**FR-UX-015: Branch tree must be navigable from a visual run tree panel.**
A run tree panel must show the parent-child relationships between runs as an expandable tree. Trunk runs are shown at the top level; branch runs are indented under their parent. Clicking any run in the tree opens that run's timeline view.

**FR-UX-016: Rollback to a prior branch point must be one action.**
The user must be able to create a new branch from any tick in any completed run without navigating away from the current view. The action "Branch from here" must be available via right-click on the timeline and via the toolbar.

**FR-UX-017: Destructive actions must require explicit confirmation with consequences stated.**
Any action that would delete or overwrite simulation data must present a confirmation dialog that states specifically what will be deleted and cannot be undone. The dialog must require the user to type a confirmation string (the run ID or scenario name) before the action proceeds.

### 4.5 Accessibility Requirements

**FR-UX-018: All interactive elements must meet WCAG 2.1 AA contrast requirements.**
Text and interactive elements must have a minimum contrast ratio of 4.5:1 against their background. Large text (18pt+ or 14pt bold+) must have a minimum contrast ratio of 3:1. Color must not be the only means of conveying information.

**FR-UX-019: All interactive elements must be keyboard-accessible.**
Every action available via mouse must be accessible via keyboard. Tab order must be logical and follow reading order. Focus indicators must be visible (2px solid outline, minimum). Custom keyboard shortcuts must be documented in a keyboard shortcut panel accessible via `?`.

**FR-UX-020: Screen reader support for simulation state panels.**
All metric panels, event log entries, and timeline annotations must have appropriate ARIA labels and roles. Dynamic content updates (run status changes, metric updates) must use `aria-live` regions with `polite` politeness. The hex map must expose a text-based alternative view (tabular grid of region metrics) accessible via screen reader.

**FR-UX-021: Color-blind safe palette required for all metric visualizations.**
All charts and hex map coloring must use a color-blind safe palette. Default palette must be distinguishable under deuteranopia, protanopia, and tritanopia. A palette selector must allow users to choose from: Default (color-blind safe), High Contrast, Monochrome. Palette preference must persist across sessions.

**FR-UX-022: Font sizes must be user-adjustable.**
Base font size must be adjustable from 12px to 24px in the application settings. All layout must accommodate the full range without horizontal scrollbars or overlapping elements at viewport widths >= 1024px.

### 4.6 Performance Requirements

**FR-UX-023: Timeline scrubber response must be under 200ms.**
From the moment the user releases the scrubber handle to the moment all visible views (hex map, metric charts, event log) reflect the selected tick, no more than 200ms must elapse. This must hold for runs up to 100,000 ticks.

**FR-UX-024: Hex map pan and zoom must achieve 60fps.**
Pan and zoom interactions on the hex map must render at 60 frames per second on target hardware (defined as: laptop with integrated GPU, 1080p display). Frame drops below 45fps must trigger an automatic LOD reduction.

**FR-UX-025: Run list must load in under 500ms for up to 500 runs.**
The run list panel must be populated within 500ms of navigation to the runs view, for workspaces containing up to 500 runs. Pagination kicks in above 500 runs.

**FR-UX-026: Export generation must not block the UI.**
Export bundle generation must run asynchronously. The UI must remain interactive during export. A progress indicator must show export stages. The user must be able to cancel an in-progress export.

**FR-UX-027: Comparison panel must handle up to 8 simultaneous runs.**
The comparison panel must support up to 8 runs simultaneously without exceeding 500ms render time for metric chart updates. Above 4 runs, the layout switches from side-by-side to overlay mode.

---

## 5. Information Architecture

### 5.1 Data Hierarchy

```
Workspace
  └── Scenario Collection
        └── Scenario
              ├── Scenario Versions (immutable snapshots)
              │     └── Version (scenario_hash, label, timestamp, config)
              └── Runs
                    ├── Run (run_id, seed, status, tick_count)
                    │     ├── Tick States (hash per tick)
                    │     ├── Metric Time Series (per metric, all ticks)
                    │     ├── Event Log (events per tick)
                    │     ├── Annotations (per tick range, per analyst)
                    │     └── Branch Runs (tree, max depth 10)
                    └── Sweep (sweep_id, manifest, aggregate outputs)
                          └── Sweep Runs (run_id per param combo x seed)
```

### 5.2 Screen / View Map

```
App Root
  ├── Workspace Dashboard
  │     ├── Scenario Collection Browser
  │     ├── Recent Runs Panel
  │     └── Workspace Settings
  ├── Scenarios
  │     ├── Scenario List
  │     ├── Scenario Detail
  │     │     ├── Version History
  │     │     ├── Version Diff
  │     │     └── Run List (for this scenario)
  │     └── Scenario Editor
  │           ├── Parameter Editor (schema-aware)
  │           ├── Annotation Panel
  │           ├── Validation Report
  │           └── Template Browser
  ├── Runs
  │     ├── Run List (workspace-level)
  │     ├── Run Detail
  │     │     ├── Timeline View
  │     │     │     ├── Hex Map (strategic LOD)
  │     │     │     ├── City View (operational LOD)
  │     │     │     ├── Timeline Scrubber
  │     │     │     ├── Metric Sparklines Panel
  │     │     │     └── Event Log Panel
  │     │     ├── Run Tree Panel
  │     │     └── Annotation Manager
  │     └── Comparison Panel
  │           ├── Run Selector
  │           ├── Metric Selector
  │           ├── Comparison Charts
  │           └── Divergence Annotation
  ├── Sweeps
  │     ├── Sweep List
  │     ├── Sweep Detail
  │     │     ├── Sweep Progress
  │     │     ├── Aggregate Summary
  │     │     └── Sensitivity Analysis View
  │     └── Sweep Export
  ├── Mods
  │     ├── Mod Registry Browser
  │     ├── Active Mods Panel
  │     └── Mod Detail
  ├── Export
  │     └── Export Configuration Dialog
  └── Settings
        ├── Workspace Settings
        ├── Accessibility (palette, font size)
        ├── Keyboard Shortcuts
        └── API Keys
```

### 5.3 Navigation Structure

**Primary navigation:** Left sidebar with icons + labels for: Workspace, Scenarios, Runs, Sweeps, Mods, Settings.

**Secondary navigation:** Breadcrumb trail within each section (e.g., Scenarios > Island State v1.2 > Run abc123).

**Contextual navigation:** Right-click context menus on timeline, hex map, and run list items for quick-access actions.

**Run tree navigation:** The run tree panel provides hierarchy navigation within a scenario's runs, showing parent-branch relationships.

**Keyboard navigation map:**

| Shortcut | Action |
|---|---|
| `G S` | Go to Scenarios |
| `G R` | Go to Runs |
| `G W` | Go to Sweeps |
| `G M` | Go to Mods |
| `?` | Open keyboard shortcuts panel |
| `Ctrl+K` | Open command palette |
| `[` | Scrub timeline backward 100 ticks |
| `]` | Scrub timeline forward 100 ticks |
| `Shift+[` | Scrub timeline backward 1000 ticks |
| `Shift+]` | Scrub timeline forward 1000 ticks |
| `C` | Open comparison panel |
| `B` | Create branch from current timeline position |
| `E` | Open export dialog |
| `A` | Add annotation at current timeline position |

---

## 6. Interaction Patterns

### 6.1 Timeline Scrubber Interaction Model

**Component:** Horizontal scrubber bar spanning the full width of the timeline view.

**Visual elements:**
- Track: full-width bar representing the run duration (tick 0 to max tick).
- Handle: draggable circular indicator showing current tick position.
- Tick labels: major tick markers at regular intervals (auto-scaled to run length).
- Annotation markers: vertical tick marks on the track at annotated ticks.
- Low-confidence bands: amber overlay regions on the track.
- Branch point marker: vertical line at branch tick (branch runs only).
- Event density heatmap: subtle color intensity on the track showing event frequency per tick range.

**Interactions:**
- Click anywhere on the track: jump to that tick position.
- Click and drag the handle: scrub continuously; views update at 60fps during drag.
- Release handle: final tick position committed; all views update to final state.
- Keyboard `[` / `]`: step backward/forward 100 ticks.
- Keyboard `Shift+[` / `Shift+]`: step backward/forward 1000 ticks.
- Keyboard `Home` / `End`: jump to tick 0 or max tick.
- Right-click on track: context menu with "Add Annotation", "Create Branch", "Copy Tick Hash".
- Scroll wheel over scrubber: fine adjustment ±1 tick per scroll click.

**Performance contract:** All view updates triggered by scrubber interaction must complete within 200ms. If data for a tick is not yet in the client cache, a skeleton loader appears immediately and is replaced when data loads. The scrubber handle moves immediately on drag; views may lag up to 200ms.

### 6.2 Hex Map Pan / Zoom Gestures

**Component:** Pixi.js v8 canvas rendering hex tile grid.

**Strategic LOD (default):**
- Hex tiles represent geographic regions (e.g., 50km per hex).
- Each tile shows: color coding by metric (legitimacy gradient by default), resource icon if a significant resource is present, institution status icon if an institution is active.
- Hovering a tile: tooltip with all eight core metric aggregates for that region at current tick.
- Clicking a tile: selects the region; metric charts filter to that region; event log filters to events in that region.
- Double-clicking a tile: transitions to operational LOD for that region.

**Operational LOD (city-level):**
- Shows city layout with building footprints, citizen cohort indicators, resource flow arrows.
- Breadcrumb displays: `Strategic View > [Region Name] > Operational View`.
- Clicking "Zoom Out" in breadcrumb or pressing `Escape`: returns to strategic LOD.

**Gesture model:**
- Pan: left-click drag.
- Zoom: scroll wheel (smooth, not stepped). Pinch-to-zoom on touchpad.
- Reset zoom: double-click on empty canvas area, or press `Home` while map is focused.
- LOD transition: automatic when zoom level crosses the LOD threshold. Manual via double-click on tile.

**Performance contract:** Pan and zoom at 60fps. LOD switch completes within 300ms including data fetch. Maximum map size supported: 1024x1024 hex grid (tiled rendering; only visible tiles rendered).

### 6.3 Intervention Panel Interaction

**Trigger:** Right-click on timeline at target tick > "Create Intervention Branch", or toolbar "Branch" button.

**Panel layout:**
- Header: "New Branch from Tick {T}"
- Branch label field: text input, required, max 64 chars.
- Intervention type selector: dropdown (Policy Override, Institution Shock, Resource Delta, Climate Event, Diplomatic Change).
- Intervention form: dynamically rendered based on type selection.
- Preview panel: shows the exact state delta that will be applied at tick T+1 as a JSON diff.
- Validation indicator: green checkmark or amber warning based on intervention validity.
- "Run Branch" button: disabled until intervention is valid and branch label is non-empty.
- "Cancel" button: closes panel without creating branch.

**Intervention form fields by type:**

| Type | Fields |
|---|---|
| Policy Override | Parameter selector (searchable), Current value (read-only), New value (validated input) |
| Institution Shock | Institution selector, Legitimacy delta (-1.0 to +1.0), Duration (ticks) |
| Resource Delta | Resource pool selector, Delta value, Unit (read-only) |
| Climate Event | Event type selector, Severity (0.0-1.0), Duration (ticks) |
| Diplomatic Change | Actor A selector, Actor B selector, Relation state selector |

### 6.4 Comparison Panel Layout

**Default layout (2 runs):**
- Left panel: Run A timeline and metric charts.
- Right panel: Run B timeline and metric charts.
- Both panels share a synchronized timeline scrubber.
- A center divider with a swap button allows swapping run positions.

**Overlay mode (3-8 runs):**
- Single chart area with all runs overlaid.
- Legend shows run ID truncated + regime label for each run.
- Line style differentiation: solid, dashed, dotted, dash-dot (cycles for runs 5-8).
- Color assignment: color-blind safe palette, one color per run.

**Display mode toggle:**
- "Absolute" mode: shows raw metric values. Y-axis scaled to accommodate all runs.
- "Delta" mode: shows (run_value - baseline_value) per tick. Baseline run is selectable.
- "Normalized" mode: shows each run normalized to its own tick-0 value (percentage change).

**Metric selector:** Checkbox grid with all eight core metrics plus any mod-contributed metrics. Selection persists within the session.

**Divergence annotation:** Automatically computed as the first tick where any selected metric diverges beyond 5% between any two runs (configurable threshold). Displayed as a vertical dashed line across all charts with a label showing the diverging metric and magnitude.

### 6.5 Notification and Alert System

**Alert levels:**

| Level | Color | Persistence | User Action Required |
|---|---|---|---|
| Info | Blue | Auto-dismiss 5s | None |
| Warning | Amber | Persistent until dismissed | Dismiss |
| Error | Red | Persistent until acknowledged | Acknowledge |
| Critical | Red + border | Blocks interaction | Acknowledge + confirm |

**Notification types and their level:**

| Event | Level |
|---|---|
| Run completed | Info |
| Run failed (instability) | Error |
| Determinism violation detected | Critical |
| Export complete | Info |
| Validation error count | Warning |
| Mod integrity verification failed | Error |
| Workspace approaching storage limit | Warning |
| Sweep completed with degenerate runs | Warning |

**Notification panel:** Accessible via bell icon in top nav. Shows last 50 notifications with timestamp, level, and description. Persistent notifications remain until the underlying issue is resolved (not just dismissed).

---

## 7. CLI Interface

### 7.1 Command Structure

```
civlab <command> [subcommand] [flags] [args]
```

Top-level commands:

| Command | Description |
|---|---|
| `civlab run` | Execute a single simulation run |
| `civlab sweep` | Execute a parameter sweep |
| `civlab replay` | Replay a run from a run ID or replay reference |
| `civlab export` | Export run data or report bundles |
| `civlab aggregate` | Aggregate sweep outputs into statistics |
| `civlab verify` | Verify integrity of an export bundle |
| `civlab mod` | Mod management subcommands |
| `civlab scenario` | Scenario management subcommands |
| `civlab config` | CivLab CLI configuration |
| `civlab version` | Print version information |

### 7.2 `civlab run`

```
USAGE:
    civlab run [FLAGS] <scenario>

ARGS:
    <scenario>    Path to scenario TOML/JSON file, or scenario ID

FLAGS:
    -s, --seed <seed>              RNG seed (default: randomly generated)
    -t, --ticks <ticks>            Number of ticks to simulate (default: scenario default)
    -o, --output <dir>             Output directory (default: ./civlab_runs/{run_id})
    -f, --format <format>          Output format: json, csv, parquet, all (default: json)
        --mods <mod1,mod2>         Comma-separated mod IDs to activate
        --label <label>            Human-readable run label
        --no-timeseries            Suppress tick-level time series output (endpoint only)
        --stream                   Stream tick data to stdout as JSON lines
        --max-ticks <n>            Hard stop at n ticks regardless of scenario config
    -q, --quiet                    Suppress progress output
    -v, --verbose                  Verbose logging (repeat for more: -vv, -vvv)
        --format <format>          Output format for CLI messages: text, json (default: text)
    -h, --help                     Print help

EXAMPLES:
    # Run with auto-generated seed, default output
    civlab run scenarios/island_state_v1.2.toml

    # Run with specific seed and label
    civlab run scenarios/island_state_v1.2.toml --seed 42 --label "Baseline Run"

    # Stream tick data to stdout
    civlab run scenarios/island_state_v1.2.toml --seed 42 --stream | jq '.metrics.legitimacy'

    # Run with mods, output as Parquet
    civlab run scenarios/island_state_v1.2.toml --mods progressive_tax_v1 --format parquet

EXIT CODES:
    0    Run completed successfully
    1    Run failed due to simulation instability
    2    Configuration error (invalid scenario file)
    3    Engine error (unexpected engine failure)
    4    Mod load error
    5    Output write error
```

### 7.3 `civlab sweep`

```
USAGE:
    civlab sweep [FLAGS] --manifest <manifest>

FLAGS:
    -m, --manifest <manifest>      Path to sweep manifest TOML file (required)
    -o, --output <dir>             Output directory (default: ./civlab_sweeps/{sweep_id})
        --resume <dir>             Resume interrupted sweep from output directory
        --dry-run                  Validate manifest and print run count without executing
        --max-concurrent <n>       Max parallel runs (default: CPU cores / 2)
        --format <format>          Output format: json, csv, parquet, all (default: parquet,csv)
        --no-timeseries            Suppress tick-level outputs; endpoint metrics only
    -p, --progress                 Show progress bar and ETA
    -q, --quiet                    Suppress all output except errors
        --format <format>          CLI message format: text, json (default: text)
    -h, --help                     Print help

EXAMPLES:
    # Run sweep from manifest
    civlab sweep --manifest sweep_manifest.toml --progress

    # Dry run to validate and count
    civlab sweep --manifest sweep_manifest.toml --dry-run
    # Output: Sweep manifest valid. 120 runs (8 param combos x 5 seeds x 3 configs).

    # Resume interrupted sweep
    civlab sweep --manifest sweep_manifest.toml --resume ./civlab_sweeps/sweep_abc123

    # JSON output for pipeline integration
    civlab sweep --manifest sweep_manifest.toml --format json --quiet

EXIT CODES:
    0    All runs completed successfully
    1    Partial failure (some runs failed; aggregate computed over successful runs)
    2    Manifest validation error
    3    All runs failed
    4    Resume directory not found or corrupt
    5    Output write error
```

### 7.4 `civlab replay`

```
USAGE:
    civlab replay [FLAGS] <run-ref>

ARGS:
    <run-ref>    Run ID, replay reference URI (civlab://replay/...), or path to run artifact dir

FLAGS:
    -o, --output <dir>             Output directory
    -t, --from-tick <tick>         Replay starting from this tick (requires full artifact)
        --verify                   Verify tick hashes during replay and report any mismatch
    -f, --format <format>          Output format: json, csv, parquet (default: json)
        --stream                   Stream replayed tick data to stdout as JSON lines
    -q, --quiet                    Suppress progress output
    -h, --help                     Print help

EXAMPLES:
    # Replay a run by ID (requires engine to have original scenario and seed)
    civlab replay a3f8b21c-s42-20260221T143000Z

    # Replay from a replay reference URI
    civlab replay "civlab://replay/a3f8b21c/42/0.9.0"

    # Replay and verify all tick hashes
    civlab replay a3f8b21c-s42-20260221T143000Z --verify

    # Replay from tick 3000 forward
    civlab replay ./civlab_runs/a3f8b21c-s42-20260221T143000Z --from-tick 3000

EXIT CODES:
    0    Replay completed; all hashes verified (if --verify)
    1    Determinism violation detected (hash mismatch)
    2    Run reference not found
    3    Engine version mismatch
    4    Incomplete artifact (cannot replay)
```

### 7.5 `civlab export`

```
USAGE:
    civlab export [FLAGS] <run-id-or-dir>

ARGS:
    <run-id-or-dir>    Run ID or path to run artifact directory

FLAGS:
    -o, --output <path>            Output path for export bundle
    -f, --format <format>          Export format: pdf, json, csv, parquet, all (default: json)
        --include-timeseries       Include full tick-level time series (default: endpoint only)
        --include-charts           Generate and include SVG charts (default: false for CLI)
        --metrics <list>           Comma-separated metric names to include (default: all)
    -h, --help                     Print help

EXAMPLES:
    # Export run as JSON bundle
    civlab export a3f8b21c-s42-20260221T143000Z --format json

    # Export as all formats
    civlab export ./civlab_runs/a3f8b21c-s42-20260221T143000Z --format all

    # Export specific metrics as CSV with time series
    civlab export a3f8b21c-s42-20260221T143000Z \
      --format csv \
      --metrics legitimacy,tyranny,resilience \
      --include-timeseries

EXIT CODES:
    0    Export completed and written to output path
    1    Run not found
    2    Format not supported
    3    Output write error
```

### 7.6 `civlab aggregate`

```
USAGE:
    civlab aggregate [FLAGS] --input <sweep-dir>

FLAGS:
    -i, --input <sweep-dir>        Path to sweep output directory (required)
    -o, --output <dir>             Output directory for aggregate files (default: {sweep-dir}/aggregate)
    -f, --format <format>          Output format: csv, json, parquet, all (default: csv,parquet)
        --metrics <list>           Comma-separated metric names to aggregate (default: all)
        --ci <level>               Confidence interval level: 0.90, 0.95, 0.99 (default: 0.95)
        --exclude-degenerate       Exclude flagged degenerate runs from aggregates
        --endpoint-ticks <list>    Comma-separated tick values for endpoint metrics (default: final tick)
        --sensitivity              Compute Spearman rank correlations (parameter sensitivity analysis)
    -h, --help                     Print help

EXAMPLES:
    # Basic aggregation
    civlab aggregate --input ./civlab_sweeps/sweep_abc123

    # Aggregation with sensitivity analysis, 95% CI, exclude degenerate
    civlab aggregate \
      --input ./civlab_sweeps/sweep_abc123 \
      --output ./analysis \
      --ci 0.95 \
      --exclude-degenerate \
      --sensitivity \
      --format parquet,csv

EXIT CODES:
    0    Aggregation completed successfully
    1    Input directory not found or invalid sweep structure
    2    No successful runs to aggregate (all degenerate or failed)
    3    Output write error
```

### 7.7 `civlab verify`

```
USAGE:
    civlab verify [FLAGS] <bundle-path>

ARGS:
    <bundle-path>    Path to export bundle file

FLAGS:
    -h, --help    Print help

OUTPUT:
    PASS  artifact_fingerprint: <BLAKE3 hex>
    FAIL  artifact_fingerprint: <BLAKE3 hex> (expected), <BLAKE3 hex> (actual)

EXIT CODES:
    0    Bundle integrity verified (PASS)
    1    Bundle integrity failed (FAIL)
    2    Bundle file not found or cannot be read
    3    Bundle format not recognized
```

### 7.8 `civlab mod` Subcommands

```
civlab mod new --type <type> --name <name>
    # Initialize a new mod project scaffold
    # Types: policy, economic, event, scenario

civlab mod test <wasm-path> [FLAGS]
    # Test a WASM mod against a scenario
    FLAGS:
        --scenario <path>          Scenario file to test against (required)
        --ticks <n>                Number of ticks to test (default: 1000)
        --verify-determinism       Run twice, compare hashes (verifies mod purity)
        --verbose                  Show per-hook call log

civlab mod package <wasm-path> [FLAGS]
    # Package and sign a mod artifact
    FLAGS:
        --manifest <civmod.toml>   Mod manifest file (default: ./civmod.toml)
        --sign                     Sign with local signing key

civlab mod publish <civmod-path>
    # Publish mod to the registry

civlab mod list
    # List installed/available mods

civlab mod inspect <mod-id>
    # Show mod metadata, hook declarations, and sandbox requirements
```

---

## 8. API Interface

### 8.1 REST Endpoints for Scenario Management

**Base URL:** `http://localhost:7777/api/v1` (local engine server)

All requests and responses use `Content-Type: application/json`.

**Authentication:** Bearer token in `Authorization` header. Token obtained from `civlab config auth`.

#### Scenarios

```
GET    /scenarios                    List all scenarios in workspace
POST   /scenarios                    Create new scenario (body: scenario config JSON)
GET    /scenarios/{id}               Get scenario metadata
GET    /scenarios/{id}/versions      List all versions of a scenario
POST   /scenarios/{id}/versions      Commit a new scenario version (body: config + label)
GET    /scenarios/{id}/versions/{v}  Get specific version (config + metadata)
POST   /scenarios/validate           Validate a scenario config (body: config JSON)
```

**POST /scenarios/validate response:**

```json
{
  "valid": false,
  "errors": [
    {
      "path": "policy.tax_rate",
      "message": "Value 0.95 exceeds maximum allowed value 0.80",
      "current_value": 0.95,
      "constraint": "max: 0.80"
    }
  ],
  "warnings": []
}
```

#### Runs

```
GET    /runs                         List all runs in workspace
POST   /runs                         Submit a new run (body: {scenario_id, version, seed, label, mods})
GET    /runs/{run_id}                Get run metadata and status
DELETE /runs/{run_id}                Delete run (requires confirmation token)
GET    /runs/{run_id}/metrics        Get endpoint metric summary
GET    /runs/{run_id}/metrics/tick/{t}  Get metric values at specific tick
GET    /runs/{run_id}/tick-hashes    Get all tick hashes (paginated)
GET    /runs/{run_id}/events         Get event log (paginated, filterable by tick range)
POST   /runs/{run_id}/annotations    Add annotation to run
GET    /runs/{run_id}/annotations    List annotations for run
POST   /runs/{run_id}/branch         Create branch run from tick T (body: {tick, intervention, label})
GET    /runs/{run_id}/branches       List all branch runs of a run
```

**POST /runs request body:**

```json
{
  "scenario_id": "scen_abc123",
  "version": "v1.2",
  "seed": 42,
  "label": "Baseline Run",
  "mods": [],
  "ticks": 10000
}
```

**GET /runs/{run_id} response:**

```json
{
  "run_id": "a3f8b21c-s42-20260221T143000Z",
  "scenario_id": "scen_abc123",
  "scenario_hash": "a3f8b21c...",
  "scenario_label": "Island State v1.2",
  "seed": 42,
  "label": "Baseline Run",
  "status": "complete",
  "tick_count": 10000,
  "created_at": "2026-02-21T14:30:00Z",
  "completed_at": "2026-02-21T14:32:45Z",
  "wall_clock_seconds": 165,
  "mods_active": [],
  "parent_run_id": null,
  "branch_tick": null
}
```

#### Sweeps

```
POST   /sweeps                       Submit a sweep job (body: sweep manifest JSON)
GET    /sweeps/{sweep_id}            Get sweep status and progress
GET    /sweeps/{sweep_id}/runs       List all runs in a sweep
GET    /sweeps/{sweep_id}/aggregate  Get aggregate statistics (if computed)
POST   /sweeps/{sweep_id}/aggregate  Trigger aggregation computation
DELETE /sweeps/{sweep_id}            Cancel and delete sweep
```

#### Export

```
POST   /export                       Request export bundle generation
    Body: { run_ids: [...], format: "json|pdf|csv|parquet|all", options: {...} }
    Response: { export_id: "...", status: "queued" }

GET    /export/{export_id}           Get export status
GET    /export/{export_id}/download  Download completed export bundle
```

### 8.2 WebSocket JSON-RPC for Live Run Data

**Endpoint:** `ws://localhost:7777/ws`

**Protocol:** JSON-RPC 2.0 over WebSocket.

#### Subscribe to Run Progress

```json
// Request
{
  "jsonrpc": "2.0",
  "id": 1,
  "method": "subscribe_run",
  "params": {
    "run_id": "a3f8b21c-s42-20260221T143000Z",
    "tick_interval": 50
  }
}

// Response (immediate)
{
  "jsonrpc": "2.0",
  "id": 1,
  "result": {
    "subscription_id": "sub_xyz",
    "run_id": "a3f8b21c-s42-20260221T143000Z"
  }
}

// Streaming events (push, no id)
{
  "jsonrpc": "2.0",
  "method": "run_tick",
  "params": {
    "subscription_id": "sub_xyz",
    "tick": 100,
    "tick_hash": "...",
    "metrics": {
      "waste": 0.12,
      "surplus": 0.45,
      "tyranny": 0.08,
      "legitimacy": 0.82,
      "resilience": 0.67,
      "joule_stock": 142500,
      "trade_balance": 3200,
      "population": 48000
    }
  }
}

// Completion event
{
  "jsonrpc": "2.0",
  "method": "run_complete",
  "params": {
    "subscription_id": "sub_xyz",
    "run_id": "a3f8b21c-s42-20260221T143000Z",
    "final_tick": 10000,
    "final_tick_hash": "..."
  }
}
```

#### Get Tick State

```json
// Request
{
  "jsonrpc": "2.0",
  "id": 2,
  "method": "get_tick_state",
  "params": {
    "run_id": "a3f8b21c-s42-20260221T143000Z",
    "tick": 4200
  }
}

// Response
{
  "jsonrpc": "2.0",
  "id": 2,
  "result": {
    "tick": 4200,
    "tick_hash": "...",
    "metrics": { ... },
    "events": [ ... ],
    "hex_state": { ... }
  }
}
```

#### Inject Intervention (live run)

```json
// Request
{
  "jsonrpc": "2.0",
  "id": 3,
  "method": "inject_intervention",
  "params": {
    "run_id": "a3f8b21c-s42-20260221T143000Z",
    "at_tick": 5000,
    "intervention": {
      "type": "policy_override",
      "parameter": "policy.tax_rate",
      "value": 0.25
    }
  }
}

// Response
{
  "jsonrpc": "2.0",
  "id": 3,
  "result": {
    "branch_run_id": "a3f8b21c-s42-20260221T143000Z@branch-T5000-b7c9d2",
    "status": "queued"
  }
}
```

### 8.3 Error Response Format

All API errors use a consistent structure:

```json
{
  "error": {
    "code": "RUN_NOT_FOUND",
    "message": "Run a3f8b21c-s42-20260221T143000Z does not exist in this workspace.",
    "details": {
      "run_id": "a3f8b21c-s42-20260221T143000Z"
    }
  }
}
```

**Standard error codes:**

| Code | HTTP Status | Description |
|---|---|---|
| `SCENARIO_NOT_FOUND` | 404 | Scenario ID does not exist |
| `VERSION_NOT_FOUND` | 404 | Scenario version does not exist |
| `RUN_NOT_FOUND` | 404 | Run ID does not exist |
| `VALIDATION_FAILED` | 422 | Scenario config failed validation |
| `SEED_REQUIRED` | 400 | Seed must be provided for this operation |
| `DETERMINISM_VIOLATION` | 409 | Replay produced mismatched tick hash |
| `ENGINE_INSTABILITY` | 500 | Engine halted due to unstable simulation state |
| `MOD_LOAD_FAILED` | 422 | WASM mod failed to load or sandbox verification failed |
| `BRANCH_DEPTH_EXCEEDED` | 422 | Branch tree exceeds maximum depth of 10 |
| `UNAUTHORIZED` | 401 | Invalid or missing Bearer token |
| `FORBIDDEN` | 403 | Token lacks required workspace permission |

---

## 9. Accessibility and Localization

### 9.1 WCAG 2.1 AA Compliance Requirements

CivLab web UI must achieve WCAG 2.1 Level AA compliance at launch. The following items are required:

**Perceivable:**
- All non-text content (charts, hex map, icons) must have text alternatives (alt text, ARIA labels, or adjacent text).
- Color is never the only means of conveying information. All color-coded information must have a secondary indicator (pattern, icon, or text label).
- All audio content must have captions or transcripts (not applicable at launch; no audio content planned).
- Text can be resized up to 200% without loss of functionality.
- No content flashes more than 3 times per second (no animations trigger photosensitive seizures).

**Operable:**
- All functionality available from a keyboard.
- No keyboard traps.
- Skip navigation link available at top of page (bypasses nav to reach main content).
- All interactive elements have visible focus indicators.
- Page titles are descriptive and unique per view.
- Link and button text is descriptive (no "Click here", no "More").

**Understandable:**
- Page language is set (`lang="en"` or appropriate locale).
- Unusual abbreviations (e.g., "FR-UX-001", "Joule stock") are expandable via definition/glossary.
- Error messages identify the field in error and describe how to fix it.
- Labels are associated with their form controls.

**Robust:**
- HTML is valid and well-formed.
- ARIA attributes used correctly.
- Custom components (timeline scrubber, hex map) expose accessible roles, states, and properties.

### 9.2 Screen Reader Support for Simulation State

The hex map and timeline scrubber are rich visual components that require special accessibility treatment.

**Hex Map Accessible Alternative:**
- A "Table View" toggle renders the hex map as a data table: rows = regions, columns = metrics.
- Table updates when the timeline scrubber position changes.
- Screen reader announcement: "Simulation state at tick {T}. {N} regions. Data table follows."

**Timeline Scrubber Accessible Alternative:**
- The scrubber is implemented as `role="slider"` with `aria-valuenow`, `aria-valuemin`, `aria-valuemax`, `aria-label="Simulation timeline, tick {T} of {max_tick}"`.
- Keyboard focus on the scrubber: `aria-live="polite"` region announces the current tick and the top changed metric on each tick change.

**Metric Charts Accessible Alternative:**
- Each chart has a "Data Table" toggle rendering the time series as a scrollable table.
- Chart SVGs include a `<title>` element with a human-readable description.
- Example: `<title>Legitimacy metric over 10000 ticks: starts at 0.82, peaks at 0.91 at tick 2300, declines to 0.34 at tick 4200</title>`.

**Event Log:**
- Each event log entry is a list item (`<li>`) with full text description.
- Causal attribution is included in the text: "Legitimacy threshold breach at tick 4195, caused by tax_rate increase at tick 4000."

**Run Status Changes:**
- Run status changes (Queued → Running → Complete) are announced via `aria-live="polite"` region.
- Run failures are announced via `aria-live="assertive"` region.

### 9.3 Color-Blind Safe Palette

The default palette must be distinguishable under deuteranopia (red-green), protanopia (red-green), and tritanopia (blue-yellow) color blindness.

**Default Metric Color Assignments:**

| Metric | Hex | Deuteranopia-safe |
|---|---|---|
| Legitimacy | `#1A85FF` (blue) | Yes |
| Tyranny | `#E66100` (orange) | Yes |
| Resilience | `#5DB0FA` (light blue) | Yes |
| Waste | `#994F00` (brown) | Yes |
| Surplus | `#40B0A6` (teal) | Yes |
| Joule Stock | `#FFC20A` (yellow) | Yes |
| Trade Balance | `#DC3977` (rose) | Yes |
| Population | `#785EF0` (purple) | Yes |

Source palette: IBM Color Blind Safe palette (2022). All colors verified against Coblis color blindness simulator.

**Hex Map Gradient:**
- Default gradient: white (low) to `#1A85FF` (high) — avoids red/green.
- Alternative gradient options in Accessibility settings: Viridis, Cividis, Greys.

**Chart Lines:**
- Up to 8 runs in comparison: line style is differentiated by both color AND line dash pattern (solid, dashed, dotted, dash-dot, long-dash, dash-dot-dot, short-dash, two-dash).

### 9.4 i18n Readiness

At launch, CivLab ships in English only. The codebase must be i18n-ready for future localization.

**Requirements:**
- All user-visible strings must be externalized to a strings file (`en.json`). No hardcoded user-facing strings in component code.
- Date formatting must use the `Intl.DateTimeFormat` API with locale awareness.
- Number formatting must use the `Intl.NumberFormat` API with locale awareness.
- Currency and unit symbols must be configurable, not hardcoded.
- RTL layout support must be considered in component design (no fixed left/right assumptions that break under `dir="rtl"`). Full RTL support is not required at launch but must not be architecturally precluded.
- Locale must be configurable in application settings.

**Strings file structure:**

```json
{
  "nav.scenarios": "Scenarios",
  "nav.runs": "Runs",
  "nav.sweeps": "Sweeps",
  "run.status.queued": "Queued",
  "run.status.running": "Running",
  "run.status.complete": "Complete",
  "run.status.failed": "Failed",
  "run.status.cancelled": "Cancelled",
  "metric.legitimacy": "Legitimacy",
  "metric.tyranny": "Tyranny",
  "metric.resilience": "Resilience",
  "metric.waste": "Waste",
  "metric.surplus": "Surplus",
  "metric.joule_stock": "Joule Stock",
  "metric.trade_balance": "Trade Balance",
  "metric.population": "Population"
}
```

---

## 10. Acceptance Criteria

### 10.1 Persona A: Scenario Designer Acceptance Tests

These tests can be performed by Maren (or a QA proxy) without access to engine internals.

**AT-SD-001: Create scenario from template**
- Navigate to Scenarios > New Scenario > Browse Templates.
- Select "Pre-Industrial Island State" template.
- Click "Fork as New Scenario".
- Verify: Editor opens with template parameters populated.
- Verify: Template origin shown in scenario header ("Forked from: Pre-Industrial Island State").
- Pass criterion: Complete in under 5 minutes.

**AT-SD-002: Inline validation error**
- In the scenario editor, set `policy.tax_rate` to `1.5` (above valid maximum of 0.80).
- Verify: Field is immediately highlighted amber.
- Verify: Error message appears below field: "tax_rate must be between 0.0 and 0.80. Current value: 1.50."
- Verify: "Commit Version" button is disabled.
- Pass criterion: Error appears within 500ms of field change without page reload.

**AT-SD-003: Full validation report**
- With a scenario containing at least one cross-parameter constraint violation, click "Validate".
- Verify: Validation report lists all violations with parameter path, current value, and constraint description.
- Verify: Report distinguishes errors (blocking) from warnings (non-blocking).
- Pass criterion: Validation report rendered within 2 seconds.

**AT-SD-004: Commit immutable version**
- With a valid scenario, click "Commit Version".
- Enter label "v1.0-test" and commit message "Test commit".
- Verify: System displays version ID, config hash (BLAKE3 hex), and stable reference URL.
- Attempt to edit any parameter of the committed version.
- Verify: Parameters are read-only; no edit controls present.
- Pass criterion: Version ID is stable and the reference URL resolves to the same config.

**AT-SD-005: Scenario version diff**
- Create two versions of the same scenario that differ in one parameter.
- Navigate to Version History and select both versions.
- Click "Diff".
- Verify: Diff view highlights the changed parameter with old and new values.
- Verify: Unchanged parameters are not shown in the diff by default (collapsed).
- Pass criterion: Diff renders within 1 second.

**AT-SD-006: Scenario reference URL stability**
- Commit a scenario version. Copy the reference URL.
- In a new browser session (or incognito), navigate to the reference URL.
- Verify: The same scenario version loads with the same config.
- Pass criterion: Reference URL is stable and human-shareable.

### 10.2 Persona B: Policy Analyst Acceptance Tests

**AT-PA-001: Run a scenario**
- Navigate to a committed scenario version.
- Click "Run". Accept default seed. Click "Submit".
- Verify: Run appears in run list with status "Queued" then "Running" then "Complete".
- Verify: Run ID is displayed in the run list entry.
- Pass criterion: Run status reflects accurate engine state within 2 seconds of change.

**AT-PA-002: Timeline scrubbing responsiveness**
- Open a completed run with at least 5000 ticks.
- Drag the timeline scrubber rapidly across the full range.
- Verify: All views (hex map, metric charts, event log) update to reflect the new tick position.
- Verify: Update completes within 200ms of releasing the scrubber.
- Pass criterion: No view shows stale tick data after scrubber release.

**AT-PA-003: Hex map LOD transition**
- In strategic LOD, double-click a hex tile.
- Verify: Transition to operational (city-level) LOD within 300ms.
- Verify: Breadcrumb shows "Strategic View > [Region Name] > Operational View".
- Click breadcrumb "Strategic View" link.
- Verify: Returns to strategic LOD with the same camera position.
- Pass criterion: LOD transition is smooth and reversible.

**AT-PA-004: Side-by-side comparison**
- Run the same scenario with two different seeds (or two different interventions).
- Click "Compare" and select both runs.
- Assign regime labels to each run.
- Select three metrics for comparison.
- Verify: Side-by-side charts render for each selected metric.
- Verify: Regime labels appear in chart legends.
- Verify: Divergence point annotation appears if runs diverge beyond threshold.
- Pass criterion: Comparison panel populated within 1 second of run selection.

**AT-PA-005: Export report package**
- From a comparison view with annotations, click "Export" > "All".
- Verify: Export dialog shows all format options.
- Verify: Assumption disclosure cannot be unchecked.
- Download the ZIP bundle.
- Verify: ZIP contains PDF, JSON, CSV, and Parquet files.
- Verify: JSON bundle contains `artifact_fingerprint` field (BLAKE3 hash).
- Run `civlab verify <bundle.json>` on the JSON bundle.
- Verify: Output is "PASS".
- Pass criterion: Export complete without UI blocking; bundle passes integrity check.

**AT-PA-006: Provenance badge on chart**
- In the timeline view, hover over the provenance badge on any metric chart.
- Verify: Badge displays run ID (8 chars), scenario label, seed, and tick range.
- Click the badge.
- Verify: Full provenance panel opens with complete metadata.
- Pass criterion: Provenance information matches the run's actual parameters.

### 10.3 Persona C: Research Operator Acceptance Tests

**AT-RO-001: Sweep dry run**
- Write a sweep manifest with 3 parameter combinations and 5 seeds (15 total runs).
- Run `civlab sweep --manifest manifest.toml --dry-run`.
- Verify: Output states "15 runs" without executing any simulation.
- Verify: Exit code 0.
- Pass criterion: Completes in under 5 seconds.

**AT-RO-002: Sweep execution and output structure**
- Execute the same sweep without `--dry-run`.
- After completion, verify the output directory contains:
  - `manifest.toml`, `sweep_id.txt`
  - `runs/{run_id}/` directories for all 15 runs
  - `aggregate/summary.csv`, `aggregate/full_dataset.parquet`
- Verify: Exit code 0 for all-success, exit code 1 if any runs failed.
- Pass criterion: Output structure matches specification exactly.

**AT-RO-003: Degenerate run detection**
- Construct a sweep manifest with at least one parameter combination known to produce instability (e.g., extreme parameter values).
- Run sweep.
- Verify: Degenerate runs appear in `aggregate/degenerate_runs.json` with instability reason.
- Verify: Aggregate statistics computed over remaining valid runs only.
- Verify: Warning in stdout: "N runs excluded as degenerate. Aggregates computed over M runs."
- Pass criterion: Degenerate runs do not corrupt aggregate statistics.

**AT-RO-004: JSON streaming output**
- Run `civlab run scenarios/island_state.toml --seed 1 --stream --ticks 100`.
- Verify: Stdout is a sequence of valid JSON lines, one per tick.
- Pipe to `jq '.metrics.legitimacy'` and verify numeric values are emitted.
- Verify: Exit code 0.
- Pass criterion: Every tick produces exactly one JSON line; all lines are valid JSON.

**AT-RO-005: REST API run submission and polling**
- Submit a run via `POST /api/v1/runs` with a valid scenario ID and seed.
- Verify: Response contains `run_id` and `status: "queued"`.
- Poll `GET /api/v1/runs/{run_id}` until status is `complete` or `failed`.
- Verify: On completion, `GET /api/v1/runs/{run_id}/metrics` returns all eight core metric values.
- Pass criterion: Full run cycle completable via REST API without web UI.

**AT-RO-006: WebSocket tick streaming**
- Connect to WebSocket endpoint and send `subscribe_run` request for a running simulation.
- Verify: `run_tick` events are received at the configured `tick_interval`.
- Verify: `run_complete` event is received when the run finishes.
- Verify: All `run_tick` events contain all eight core metrics.
- Pass criterion: No ticks missed; events arrive in tick order.

**AT-RO-007: Sweep resume**
- Start a sweep. Interrupt it after 50% completion (kill the process).
- Rerun with `--resume ./sweep_output`.
- Verify: Only the remaining runs execute (already-complete run artifacts are not re-run).
- Verify: Final aggregate includes all runs (both pre-interrupt and post-resume).
- Pass criterion: Resume produces identical aggregate to uninterrupted run.

### 10.4 Persona D: Modder Acceptance Tests

**AT-MD-001: Mod scaffold generation**
- Run `civlab mod new --type policy --name test_policy`.
- Verify: Directory `test_policy/` created with `Cargo.toml`, `src/lib.rs`, `civmod.toml`, `tests/` scaffold.
- Verify: `Cargo.toml` contains `civlab-sdk` dependency at the correct version.
- Pass criterion: `cargo build --target wasm32-wasip2` succeeds on the scaffold without modification.

**AT-MD-002: Mod test harness determinism check**
- Run `civlab mod test ./my_mod.wasm --scenario examples/island_state.toml --ticks 500 --verify-determinism`.
- Verify: Test runs the simulation twice with the same mod.
- Verify: Output states "Determinism check: PASS" if all tick hashes match between runs.
- Verify: Output states "Determinism check: FAIL — tick {T} hash mismatch" if hashes diverge.
- Pass criterion: A pure mod passes; a non-deterministic mod fails.

**AT-MD-003: Sandbox constraint violation**
- Compile a mod that attempts file I/O (violates sandbox constraints).
- Run `civlab mod test` against this mod.
- Verify: Output states "Sandbox violation: file_io attempted at hook PolicyMod::apply".
- Verify: Exit code is non-zero.
- Pass criterion: Violation detected at load time or first hook call; run does not proceed.

**AT-MD-004: Active mod badge in web UI**
- Run a scenario with a mod activated.
- In the web UI, open the completed run.
- Verify: "Mods Active" badge is visible in the run header.
- Click the badge.
- Verify: Panel shows mod name, version, registry URL, and signature status.
- Pass criterion: Mod badge is always present for mod-active runs, on every view.

### 10.5 Usability Benchmarks

These benchmarks define the minimum acceptable usability bar, measurable in usability testing with representative users.

| Benchmark | Target | Measurement Method |
|---|---|---|
| Time to create first scenario from template | < 10 minutes (median) | Task time in usability session |
| Time to produce side-by-side comparison | < 5 minutes from run completion (median) | Task time in usability session |
| Time to produce export report | < 3 minutes from comparison view (median) | Task time in usability session |
| Error rate on scenario commit | < 10% of attempts blocked by preventable errors | Count blocked commits in usability session |
| Timeline scrubber discoverability | > 90% of users find scrubber without instruction | Observation in usability session |
| CLI sweep command success without docs | > 70% of Research Operators complete a sweep using only `--help` | Task success rate in usability session |
| Provenance badge comprehension | > 80% of Policy Analysts correctly describe what produced a chart | Comprehension question in usability session |

---

*End of CivLab Civ-Sim User Specification v0.1*
*Document maintained by CivLab Product Engineering*
*Next review: 2026-03-21*


---
