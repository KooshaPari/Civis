# CIV-0001: Core Simulation Loop — Deterministic Tick Architecture

**Version:** 2.0
**Status:** SPECIFICATION
**Date:** 2026-02-21
**Authors:** CIV Architecture & Engine Team

---

## Executive Summary

CivLab's simulation core is a **deterministic, fixed-timestep tick engine** that runs independently of any client renderer. The engine executes a single timeline of world state, accepting commands from multiple clients, computing state transitions deterministically, and emitting snapshots and events for clients to render.

This spec defines:
- **Tick architecture** (fixed-timestep, lockstep, parallel phase scheduling)
- **Entity model** (ECS-style, Rust-native)
- **State snapshot protocol** (deterministic computation → immutable snapshots)
- **Multi-client attachment** (how clients connect to the headless core)
- **Determinism invariants** (all enforcement rules)
- **Replay format** (.civreplay: compressed event log + header)

---

## Context & Design Goals

### Problem Statement

Simulation engines often entangle:
1. **Game logic** (what happens) with **rendering** (how it looks)
2. **Deterministic core** with **client I/O** (networking, frame timing)
3. **Single-client state** with **multi-client synchronization**

CivLab solves this via:
- **Headless core:** Pure deterministic state machine, zero rendering
- **Protocol abstraction:** Clients communicate via JSON-RPC (WebSocket) or binary frames (game engines)
- **Pluggable renderers:** Bevy, Unreal, Unity, Godot, Web can all attach to the same running core

### Design Goals

| Goal | Rationale |
|------|-----------|
| **Determinism** | Replay from event log; auditability; testing |
| **Replayability** | Given seed + input events, reproduce exact state |
| **Horizontal scaling** | Multiple clients attach to single core; core logic is stateless per tick |
| **Sub-16ms tick** | 60 FPS client rendering requires <= 16ms tick compute time |
| **Research-friendly** | Scenarios, parameters, policies are first-class; easy to fork/modify |
| **Client-agnostic** | Core doesn't know if client is Bevy, Unreal, or Web |

---

## Tick Architecture

### Fixed-Timestep Model

**Tick Duration:** 100 ms (10 ticks per second)
**Simulation Speed:** Decoupled from wall-clock time (can run 100x faster or slower)

```
Wall Time       Tick 0           Tick 1           Tick 2
           |----100ms----|----100ms----|----100ms----|
Sim Time   | 0:00:00     | 0:01:40     | 0:03:20
           |  Year 0     | Year ~0.1   | Year ~0.2
```

**Invariant:** Ticks are **monotonic and gapless.** No frame skipping; no time jumps.

### Tick Execution Phase Schedule

Each tick follows a rigid phase sequence (no parallel mutation):

```
Tick N
├─ 1. Command Intake (50 µs)
│    Input: commands from all clients
│    Output: command buffer (ordered by client priority)
│
├─ 2. Policy Phase (2 ms)
│    Input: current state + commands
│    Process: evaluate all policies (tax, production, allocation)
│    Output: control signals to economy, diplomacy, war modules
│
├─ 3. Deterministic Transition (8 ms)
│    Input: state + controls
│    Process: Production → Trade → Allocation → Casualty Handling
│    Output: state'
│    Invariant: No RNG calls; all values deterministic
│
├─ 4. Stochastic Event Phase (3 ms)
│    Input: state'
│    Process: Roll random events (with seeded RNG)
│    Output: state'' + event list
│    Invariant: All RNG calls logged; same seed → same events
│
├─ 5. Metrics Compute (1 ms)
│    Input: state''
│    Process: Aggregate surplus, waste, legitimacy, GDP
│    Output: metrics snapshot
│
└─ 6. Client Broadcast (50 µs)
     Input: state'' + events + metrics
     Process: Emit snapshot to all subscribed clients
     Output: JSON/binary frames on all attached WebSocket streams
```

**Total Tick Time:** ~14 ms (well under 16 ms budget for 60 FPS clients)

### Deterministic Transition Guarantee

The deterministic transition phase MUST be:
- **No randomness:** All values computed from state + controls
- **No floats in core logic:** Use fixed-point (i64 cents, not f64 dollars)
- **Stable collection iteration:** BTreeMap, not HashMap
- **No system time leaks:** Simulation clock is abstract; no `SystemTime::now()`

**Enforcement:**
```rust
// Pseudo-code
fn deterministic_transition(state: &State, control: &Control) -> State {
    // All calls to rng() here are ERRORS (will fail linter)
    let new_state = state
        .apply_production(control.production)
        .apply_trade(control.prices)
        .apply_allocation(control.allocation)
        .apply_casualties(control.military);

    // Type signature forbids randomness:
    // fn apply_X(&self, control: X) -> Self
    // No &mut self; no rng parameter
    new_state
}
```

---

## Entity & Component Model (ECS)

CivLab uses a **hybrid ECS** (Entity-Component-System) architecture optimized for Rust:

### Entity Types

| Entity Type | ID | Examples | Mutation Frequency |
|---|---|---|---|
| **Cell** | Cell(x: i32, y: i32) | terrain, crops, roads | Per tick (climate, growth) |
| **Building** | Building(id: u64) | factories, warehouses, universities | Per tick (production) |
| **Agent** | Agent(id: u64) | citizens, merchants, generals | Per frame (movement, mood) |
| **Institution** | Inst(id: u64) | provinces, nations, factions | Per month (policy) |
| **Market** | Market(good: GoodId, location: Cell) | grain market @ (10,20) | Per tick (price, volume) |

### Component Structure

**Per-entity data stored in flat arrays (cache-friendly):**

```rust
pub struct World {
    // Entity metadata
    entity_gen: Vec<u32>,  // Generation counter (handle validity)
    entity_alive: BitSet,   // Dense alive set

    // Dense component arrays (parallel iteration)
    position: Vec<Position>,
    inventory: Vec<Inventory>,
    mood: Vec<Mood>,
    health: Vec<Health>,

    // Sparse components (only some entities have these)
    institution_role: SparseSet<InstRole>,
    military_command: SparseSet<MilCommand>,
}

pub struct Position {
    x: i32,
    y: i32,
}

pub struct Inventory {
    grain: i64,       // in units
    labor: i64,       // in person-hours
    energy: i64,      // in joules (Joule economy)
}

pub struct Mood {
    happiness: i16,   // -100 to +100
    legitimacy: i16,  // trust in regime
    grievance: i16,   // rebellion risk
}
```

### Query Pattern (Zero-Copy)

```rust
// Pseudo-code for a tick operation
fn tick_production(world: &mut World, tick: u64) {
    // Iterate all buildings with inventory
    for (entity, (pos, inv, role)) in world
        .iter_entities()
        .iter(Position, Inventory, BuildingRole)
    {
        let output = compute_production(*role, *inv, tick);
        world.inventory[entity].energy += output;
    }
}
```

**Benefit:** Single iteration over contiguous memory; no allocation; deterministic iteration order.

---

## State Snapshot Protocol

### Snapshot Structure

A **snapshot** is an immutable, serializable view of world state at tick N:

```json
{
  "header": {
    "tick": 12345,
    "seed": 999,
    "sim_time_hours": 12096,
    "version": "1.0"
  },
  "world": {
    "cells": [
      {
        "x": 0, "y": 0,
        "terrain": "grassland",
        "fertility": 95,
        "population": 450,
        "buildings": [{ "id": 1001, "type": "farm" }]
      },
      // ... thousands more
    ],
    "agents": [
      {
        "id": 10001,
        "position": {"x": 5, "y": 10},
        "health": 87,
        "mood": {"happiness": 35, "legitimacy": 62, "grievance": 8},
        "inventory": {"grain": 200, "labor": 8, "energy": 5000}
      },
      // ... thousands more
    ],
    "institutions": [
      {
        "id": 5001,
        "name": "Kingdom of Acme",
        "population": 50000,
        "treasury": 1000000,
        "ideology": "monarchy",
        "legitimacy": 75
      },
      // ... dozens more
    ]
  },
  "markets": {
    "grain": [
      {
        "location": {"x": 10, "y": 20},
        "price": 125,  // cents per unit
        "bid_volume": 1000,
        "ask_volume": 800,
        "clearing_volume": 800
      },
      // ... one per good per region
    ]
  },
  "metrics": {
    "global_gdp": 5000000,
    "average_happiness": 42,
    "gini_coefficient": 0.65,
    "total_casualties": 1200,
    "food_supply_months": 3.5,
    "energy_efficiency": 0.82
  },
  "events": [
    {
      "tick": 12345,
      "type": "production.completed",
      "entity": 1001,
      "data": {"output": 150, "good": "grain"}
    },
    {
      "tick": 12345,
      "type": "market.cleared",
      "market": "grain@(10,20)",
      "data": {"price": 125, "volume": 800}
    },
    // ... 50-200 events per tick
  ]
}
```

### Snapshot Computation

**Input:** `state[N] + control[N] + seed[N]`
**Output:** `snapshot[N] + state[N+1]`

```rust
fn compute_snapshot(state: &State, control: &Control, seed: u64) -> (Snapshot, State) {
    // 1. Deterministic transition (same as always)
    let state_det = deterministic_transition(&state, control);

    // 2. Stochastic events (using seeded RNG)
    let mut rng = ChaCha20Rng::seed_from_u64(seed);
    let (state_new, events) = stochastic_phase(&state_det, &mut rng);

    // 3. Metrics
    let metrics = compute_metrics(&state_new);

    // 4. Serialize snapshot
    let snapshot = Snapshot {
        header: SnapshotHeader {
            tick: state.tick + 1,
            seed,
            sim_time_hours: state.tick * 1.67,  // 100 ms per tick
        },
        world: &state_new,
        events,
        metrics,
    };

    (snapshot, state_new)
}
```

### Snapshot Broadcast

After snapshot is computed, emit to all subscribed clients:

```
Server → Client (WebSocket JSON-RPC)
{
  "jsonrpc": "2.0",
  "method": "sim.tick_broadcast",
  "params": {
    "snapshot": { ... },
    "sequence": 12345
  }
}
```

Or binary frame (game engines):
```
[4 bytes: tick number] [4 bytes: event count] [compressed snapshot bytes...]
```

---

## Multi-Client Architecture

### Client Attachment Model

Multiple clients connect to a **single shared simulation core:**

```
                    ┌─ Bevy Client (Linux)
                    │
┌──────────────────┤─ Unreal Client (Windows)
│ CivLab Headless  │
│ Simulation Core  ├─ Unity Client (Mac)
│ (Tick ~14ms)     │
└──────────────────┤─ Web Browser (TS + WebGL)
                    │
                    └─ Research API (Python script)
```

All clients see the **same deterministic world timeline.** If a command is accepted at tick 12345, all clients see its effects at tick 12346 (with some latency for client rendering).

### Client Lifecycle

```
1. CONNECT
   Client → Server: WebSocket connect to ws://localhost:9876/sim
   Server: Accept connection, allocate ClientSession

2. HANDSHAKE
   Client → Server: {"method": "sim.handshake", "params": {...}}
   Server → Client: {"result": {"tick": 100, "snapshot": {...}}}

3. COMMAND (repeating)
   Client → Server: {"method": "sim.command", "params": {"action": "build", ...}}
   Server → Client: {"result": {"accepted": true, "tick_applied": 101}}

4. SUBSCRIBE (repeating)
   Client → Server: {"method": "sim.subscribe", "params": {"filter": "entities.buildings"}}
   Server → Client: [Broadcast frames on every tick]

5. DISCONNECT
   Client → Server: WebSocket close
   Server: Cleanup ClientSession
```

### Client Priority & Command Ordering

**Problem:** Multiple clients may issue commands in the same tick. Which takes precedence?

**Solution:** **Command priority queue** per client, evaluated in FIFO order:

```rust
pub struct ClientPriority {
    client_id: u64,
    priority: u32,  // 0=highest, 1000=lowest (e.g., research client is low priority)
    tick_received: u64,
}

fn policy_phase(commands: Vec<Command>) -> Control {
    // Sort by (priority, tick_received) → execute in order
    // Later commands may override earlier commands (e.g., two clients build at same location)
    let mut sorted = commands;
    sorted.sort_by_key(|cmd| (cmd.client.priority, cmd.tick_received));

    let mut control = Control::default();
    for cmd in sorted {
        control.apply(cmd);  // May override previous command
    }
    control
}
```

**Priority Tiers:**
- **0 (Highest):** Admin/test clients (override any command)
- **1:** Player-controlled clients (game clients)
- **2:** AI agents (NPCs, research bots)
- **3-9:** Research APIs, logging clients (read-only)

### Snapshot Filtering

Clients can request partial snapshots (to reduce bandwidth):

```json
{
  "method": "sim.subscribe",
  "params": {
    "filter": "entities.agents,metrics.gdp,markets.grain",
    "region": {"x_min": 0, "y_min": 0, "x_max": 100, "y_max": 100}
  }
}
```

Server responds with filtered snapshot (only fields matching filter + region bounds).

---

## Determinism Invariants

### Mandatory Rules

Every simulation run MUST satisfy these invariants:

#### I1: No Floating-Point in Simulation Logic

**Rule:** Money, resources, energy are always `i64` (integer cents, units, joules).

**Violation Example (❌):**
```rust
let price: f64 = 12.34;  // Loses precision
```

**Correct (✅):**
```rust
let price_cents: i64 = 1234;  // Exact
```

#### I2: ChaCha20Rng Seeded, Not Unseeded

**Rule:** All randomness via seeded `ChaCha20Rng`. No `rand::random()`.

**Violation Example (❌):**
```rust
let choice = rand::random::<usize>() % options.len();  // Non-deterministic
```

**Correct (✅):**
```rust
let mut rng = ChaCha20Rng::seed_from_u64(seed);
let choice = rng.gen_range(0..options.len());  // Deterministic
```

#### I3: BTreeMap for Ordered Collections

**Rule:** Use `BTreeMap`, not `HashMap`, for any collection whose iteration order affects output.

**Violation Example (❌):**
```rust
let mut goods: HashMap<GoodId, Quantity> = /* ... */;
for (id, qty) in &goods {
    emit_event(id, qty);  // Order is undefined
}
```

**Correct (✅):**
```rust
let mut goods: BTreeMap<GoodId, Quantity> = /* ... */;
for (id, qty) in &goods {
    emit_event(id, qty);  // Order guaranteed by key
}
```

#### I4: No System Time in Simulation

**Rule:** Simulation clock is `tick: u64`. No `SystemTime::now()` or `Instant::now()`.

**Violation Example (❌):**
```rust
let now = SystemTime::now();  // Non-deterministic
let elapsed = now.elapsed();  // Non-deterministic
```

**Correct (✅):**
```rust
let current_tick = state.tick;
let elapsed_ticks = state.tick - started_tick;
```

#### I5: Deterministic Iteration Order

**Rule:** Any iteration that produces events or mutations must be in deterministic order.

**Violation Example (❌):**
```rust
for agent in agents.iter() {  // Order undefined if agents are in Vec
    if should_move(agent) {
        agent.move_to(new_pos);
    }
}
```

**Correct (✅):**
```rust
let mut agent_ids: Vec<_> = agents.keys().collect();
agent_ids.sort();
for agent_id in agent_ids {
    let agent = agents.get_mut(agent_id);
    if should_move(agent) {
        agent.move_to(new_pos);
    }
}
```

#### I6: No Mutable Shared State Across Phases

**Rule:** Each phase has immutable read from prior phase, produces new state for next phase.

**Violation Example (❌):**
```rust
// Bad: Direct mutation
let state = &mut shared_state;
policy_phase(state);
transition_phase(state);  // If policy didn't fully commit, this sees inconsistent state
```

**Correct (✅):**
```rust
let state1 = policy_phase(&state0);
let state2 = transition_phase(&state1);  // Clear data flow
```

### Enforcement Mechanisms

#### E1: Determinism Validator Test

**Every tick simulation must pass replay test:**

```rust
#[test]
fn test_tick_deterministic_replay() {
    let seed = 12345u64;
    let state0 = create_test_state();
    let control = create_test_control();

    // Run 1
    let (snapshot1, state1) = tick(&state0, &control, seed);

    // Run 2 (replay with same inputs)
    let (snapshot2, state2) = tick(&state0, &control, seed);

    // Must be identical
    assert_eq!(snapshot1, snapshot2);
    assert_eq!(state1, state2);
}
```

**CI Gate:** This test must pass on all PRs.

#### E2: Clippy Lint for Non-Determinism

**Custom linter rule (rustc plugin):**

```bash
cargo clippy --all -- \
    -W "random_without_seed" \
    -W "floating_point_in_sim" \
    -W "hashmap_in_critical_path" \
    -W "system_time_in_sim"
```

**Blocks merge if violations found.**

#### E3: Hash Contracts

Every event includes hash of producing state:

```json
{
  "event": {
    "tick": 12345,
    "type": "production.completed",
    "state_hash": "abc123...",
    "produced": 150
  }
}
```

On replay, recompute state hash and compare. If mismatch → determinism violation → error.

---

## Replay File Format (.civreplay)

### File Structure

```
[CIVREPLAY_V1_HEADER]
[4 bytes: magic] = 0xCAFEB0BA
[4 bytes: version] = 1
[8 bytes: seed]
[8 bytes: initial_tick]
[4 bytes: initial_state_size]
[N bytes: initial_state_json_gzipped]

[EVENTS_SECTION]
[4 bytes: event_count]
[For each event]:
  [4 bytes: event_size]
  [N bytes: event_json_gzipped]

[CHECKSUM]
[32 bytes: sha256 of entire file]
```

### Example .civreplay

```bash
# Create replay file
civlab export-replay \
  --run-id 9876 \
  --output scenario_1_run_9876.civreplay

# Inspect
civlab replay-info scenario_1_run_9876.civreplay
# Output:
#   Seed: 54321
#   Duration: 10000 ticks (16.7 minutes sim time)
#   Events: 487234
#   Size: 42 MB (compressed 8 MB)

# Replay from event log
civlab replay scenario_1_run_9876.civreplay \
  --output replayed_snapshot.json \
  --until-tick 5000

# Verify replay determinism
civlab replay-verify scenario_1_run_9876.civreplay
# Output: ✓ All 10000 ticks match original
```

### Replay Library API

```rust
pub struct ReplayFile {
    seed: u64,
    initial_state: State,
    events: Vec<Event>,
}

impl ReplayFile {
    pub fn load(path: &Path) -> Result<Self> { /* ... */ }

    pub fn verify(&self) -> Result<()> {
        // Replay entire run, verify checksum
        let final_snapshot = self.replay_to_end()?;
        self.verify_checksum(&final_snapshot)?;
        Ok(())
    }

    pub fn replay_to_tick(&self, tick: u64) -> Result<(Snapshot, State)> {
        let mut state = self.initial_state.clone();
        for event in &self.events {
            if event.tick > tick { break; }
            state = apply_event(&state, event)?;
        }
        Ok((state.to_snapshot(), state))
    }
}
```

---

## Client Protocol Specification

### WebSocket JSON-RPC 2.0 Messages

#### Method: `sim.handshake`

**Request:**
```json
{
  "jsonrpc": "2.0",
  "id": 1,
  "method": "sim.handshake",
  "params": {
    "client_name": "bevy_renderer_1",
    "client_type": "game",
    "version": "1.0"
  }
}
```

**Response:**
```json
{
  "jsonrpc": "2.0",
  "id": 1,
  "result": {
    "tick": 100,
    "seed": 54321,
    "snapshot": { /* full initial snapshot */ },
    "server_time_ms": 1234567890
  }
}
```

#### Method: `sim.command`

**Request:**
```json
{
  "jsonrpc": "2.0",
  "id": 2,
  "method": "sim.command",
  "params": {
    "action": "build",
    "entity": 1001,
    "building_type": "farm",
    "position": {"x": 10, "y": 20}
  }
}
```

**Response:**
```json
{
  "jsonrpc": "2.0",
  "id": 2,
  "result": {
    "accepted": true,
    "tick_applied": 101,
    "cost": {"grain": 50, "labor": 100}
  }
}
```

#### Method: `sim.snapshot`

**Request:**
```json
{
  "jsonrpc": "2.0",
  "id": 3,
  "method": "sim.snapshot",
  "params": {
    "tick": 105,
    "filter": ["entities.buildings", "metrics.gdp"]
  }
}
```

**Response:**
```json
{
  "jsonrpc": "2.0",
  "id": 3,
  "result": {
    "snapshot": { /* filtered snapshot */ }
  }
}
```

#### Method: `sim.subscribe`

**Request:**
```json
{
  "jsonrpc": "2.0",
  "id": 4,
  "method": "sim.subscribe",
  "params": {
    "filter": ["entities.agents", "events.all"],
    "framerate": 60
  }
}
```

**Response (Immediate):**
```json
{
  "jsonrpc": "2.0",
  "id": 4,
  "result": { "subscribed": true }
}
```

**Broadcast (Every tick or every N ticks):**
```json
{
  "jsonrpc": "2.0",
  "method": "sim.tick_broadcast",
  "params": {
    "tick": 101,
    "snapshot": { /* filtered snapshot */ },
    "events": [ /* events since last tick */ ]
  }
}
```

### Binary Frame Format (for Game Engines)

**Header (16 bytes):**
```
[4 bytes: tick number, big-endian]
[4 bytes: event count, big-endian]
[4 bytes: snapshot size, big-endian]
[4 bytes: flags (reserved)]
```

**Payload:**
```
[snapshot_size bytes: zstd-compressed snapshot JSON]
[event_size bytes: zstd-compressed event array JSON]
```

**Example (Unreal C++ unpacking):**
```cpp
struct TickFrame {
    uint32_t tick;
    uint32_t event_count;
    uint32_t snapshot_size;
    std::vector<uint8_t> snapshot_data;
    std::vector<uint8_t> event_data;
};

TickFrame unpack_frame(const std::vector<uint8_t>& raw) {
    TickFrame frame;
    frame.tick = read_u32_be(raw, 0);
    frame.event_count = read_u32_be(raw, 4);
    frame.snapshot_size = read_u32_be(raw, 8);
    frame.snapshot_data = zstd_decompress(raw.subspan(16, frame.snapshot_size));
    // ...
    return frame;
}
```

---

## Acceptance Criteria

### FR-CIV-CORE-001: Tick Monotonicity
**Spec:** Every tick increments exactly once per step.
**Test:** Assert `tick[n+1] == tick[n] + 1` for all iterations.
**Status:** Open

### FR-CIV-CORE-002: Deterministic Transition
**Spec:** Same state + control → identical state, no RNG.
**Test:** Run transition 10 times, verify all outputs identical.
**Status:** Open

### FR-CIV-CORE-003: Seeded RNG in Stochastic Phase
**Spec:** Stochastic events use ChaCha20Rng seeded with seed parameter.
**Test:** Same seed → identical events; different seed → different (but valid) events.
**Status:** Open

### FR-CIV-CORE-004: Sub-16ms Tick Time
**Spec:** Single tick completes in &lt; 16 ms wall time.
**Test:** Measure tick_compute_time on commodity hardware; assert \< 16 ms.
**Status:** Open

### FR-CIV-CORE-005: BTreeMap Ordered Iteration
**Spec:** All entities iterated in sorted (deterministic) order.
**Test:** Verify all collections are BTreeMap, not HashMap, in critical paths.
**Status:** Open

### FR-CIV-CORE-006: No System Time in Simulation
**Spec:** No `SystemTime::now()` calls in simulation crate.
**Test:** Clippy linter blocks `system_time_in_sim` pattern.
**Status:** Open

### FR-CIV-CORE-007: Snapshot Serialization
**Spec:** State can be serialized to JSON snapshot without loss.
**Test:** Snapshot round-trip: state → snapshot → state', verify equality.
**Status:** Open

### FR-CIV-CORE-008: Multi-Client Command Ordering
**Spec:** Commands from multiple clients applied in deterministic order (priority queue).
**Test:** Issue 10 commands from 3 clients, verify order matches priority + FIFO.
**Status:** Open

### FR-CIV-CORE-009: WebSocket JSON-RPC Protocol
**Spec:** Implement JSON-RPC 2.0 methods: handshake, command, snapshot, subscribe.
**Test:** Connect test client, issue handshake + command + subscribe, verify responses.
**Status:** Open

### FR-CIV-CORE-010: Replay File Format
**Spec:** Export runs to .civreplay format (header + event log + checksum).
**Test:** Export run, load .civreplay, verify checksum and event count.
**Status:** Open

### FR-CIV-CORE-011: Replay Determinism Verification
**Spec:** Can replay .civreplay file and verify determinism (state hash match).
**Test:** Load .civreplay, replay to end, compare state hash with original.
**Status:** Open

### FR-CIV-CORE-012: Fixed-Point Arithmetic
**Spec:** No floating-point in money, resources, or energy; all i64.
**Test:** Clippy linter detects floating-point in sim logic; compilation fails.
**Status:** Open

### FR-CIV-CORE-013: Phase Schedule Integrity
**Spec:** Ticks execute phases in order (Command → Policy → Transition → Stochastic → Metrics → Broadcast).
**Test:** Log phase entry/exit timestamps, verify order and timing.
**Status:** Open

### FR-CIV-CORE-014: Event Logging
**Spec:** Every state-mutating action emits event to log.
**Test:** Verify event count > 0 per tick; replay matches event log.
**Status:** Open

### FR-CIV-CORE-015: State Hash Contracts
**Spec:** Every event includes hash of state that produced it.
**Test:** Replay event, verify state hash matches; mismatch → error.
**Status:** Open

### FR-CIV-CORE-016: Client Priority Tiers
**Spec:** Commands prioritized by (client_priority, tick_received).
**Test:** Issue conflicting commands from different priority clients, verify higher priority wins.
**Status:** Open

### FR-CIV-CORE-017: Snapshot Filtering
**Spec:** Clients can request partial snapshots (filter by entity type, region).
**Test:** Subscribe with filter, verify returned snapshot matches filter.
**Status:** Open

### FR-CIV-CORE-018: Binary Frame Format
**Spec:** Support zstd-compressed binary frames in addition to JSON-RPC.
**Test:** Send binary frame, verify unpacking matches JSON equivalent.
**Status:** Open

### FR-CIV-CORE-019: ECS Entity Model
**Spec:** Entities modeled as dense arrays (cache-friendly, zero-copy queries).
**Test:** Verify all components are Vec\<T\> or SparseSet; no allocations per iteration.
**Status:** Open

### FR-CIV-CORE-020: Horizontal Multi-Client Scaling
**Spec:** Core logic is stateless per tick; multiple clients attach without interference.
**Test:** Connect 5 clients simultaneously, issue commands in parallel, verify no race conditions.
**Status:** Open

---

## Implementation Notes

### Rust Crate Structure

```
crates/
├── engine/          # Core tick loop, phases, ECS
├── policy/          # Policy evaluation, controls
├── economy/         # Production, trade, allocation
├── social/          # Citizens, mood, legitimacy
├── military/        # Units, combat, casualties
├── diplomacy/       # Treaties, alliances, war declaration
├── geography/       # Terrain, climate, migration
├── server/          # WebSocket server, protocol dispatcher
└── replay/          # Replay file I/O, verification
```

### Testing Strategy

**All new code must have:**
1. **Unit tests** (same file, `#[cfg(test)]`)
2. **Integration tests** (tests/ dir, multi-crate)
3. **Determinism tests** (replay verification)
4. **Scenario tests** (run full scenario, check metrics bounds)

**Example test structure:**

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_phase_schedule_order() { /* ... */ }

    #[test]
    fn test_tick_deterministic_replay() { /* ... */ }

    #[test]
    fn test_determinism_multiple_seeds() {
        for seed in [1, 2, 3, 5, 7, 11] {
            let (snap1, state1) = tick(&state0, control, seed);
            let (snap2, state2) = tick(&state0, control, seed);
            assert_eq!(snap1, snap2);
        }
    }
}
```

### Performance Budget

| Component | Budget | Typical | Notes |
|---|---|---|---|
| Command Intake | 50 µs | 10 µs | 1000 commands/tick OK |
| Policy Phase | 2 ms | 1.5 ms | Growing with actor count |
| Deterministic Transition | 8 ms | 6 ms | Production + trade + allocation |
| Stochastic Events | 3 ms | 2.5 ms | RNG + event generation |
| Metrics | 1 ms | 0.8 ms | Aggregation only |
| Broadcast | 50 µs | 30 µs | Network buffering |
| **Total** | **~14 ms** | **~11 ms** | **Target: 60 FPS clients** |

---

## References

- **ADR-003:** Deterministic Scenario Replay
- **ADR-002:** Joule Economy as Allocator
- **CIV-0100:** Economy v1 Spec
- **CIV-0107:** Joule Economy System v1
- **Rust ECS:** https://github.com/ivankabestwill/hecs (zero-copy queries)
- **ChaCha20Rng:** https://docs.rs/rand_chacha/
- **JSON-RPC 2.0:** https://www.jsonrpc.org/specification

---

**Version History:**
- v2.0 (2026-02-21): Full expansion from 19-line scaffold to 600+ line spec. Multi-client, protocol, determinism rules.
- v1.0 (earlier): Brief outline.
