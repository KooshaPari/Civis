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
