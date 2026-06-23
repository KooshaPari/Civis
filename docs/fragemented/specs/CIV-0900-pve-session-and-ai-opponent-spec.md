# CIV-0900: PvE Session and AI Opponent Model

**Spec ID:** CIV-0900
**Version:** 1.0
**Status:** SPECIFICATION
**Date:** 2026-02-21
**Authors:** CIV Architecture & Engine Team

**Related Specs:**
- CIV-0001: Core Simulation Loop (deterministic tick, ChaCha20Rng, BLAKE3 hash chain)
- CIV-0200: Client Protocol (JSON-RPC 2.0 over WebSocket, binary frame format)
- CIV-0400: AI / NPC Behavior Specification (MCTS, utility scoring, personality archetypes)
- CIV-1000: Save / Load System (state serialization, slot management)

---

## Table of Contents

1. [Overview and Scope](#1-overview-and-scope)
2. [Session Model](#2-session-model)
3. [Human Turn Model (Hot-Seat)](#3-human-turn-model-hot-seat)
4. [AI Nation Controller Integration](#4-ai-nation-controller-integration)
5. [Observer / Spectator Mode](#5-observer--spectator-mode)
6. [Async Challenge Mode](#6-async-challenge-mode)
7. [Pause / Resume / Speed Control](#7-pause--resume--speed-control)
8. [Save / Load Integration](#8-save--load-integration)
9. [Event Schema](#9-event-schema)
10. [JSON-RPC Methods](#10-json-rpc-methods)
11. [DB Schema](#11-db-schema)
12. [Rust Data Structures](#12-rust-data-structures)
13. [Acceptance Criteria](#13-acceptance-criteria)
14. [Integration Points](#14-integration-points)

---

## 1. Overview and Scope

### 1.1 What CIV-0900 Covers

CivLab is a **headless deterministic civilization simulation engine**. This spec defines the **session and opponent model** for all supported play configurations. A "session" is one bounded execution of the simulation with a defined set of participants, a scenario, and a lifecycle from creation to completion.

The session layer sits above the raw tick engine (CIV-0001) and below the client protocol (CIV-0200). It is responsible for:

- Organizing simulation participants (human clients, AI controllers, observers)
- Managing session lifecycle state (created, running, paused, ended)
- Coordinating turn authority in hot-seat configurations
- Hosting all AI nation controllers inside the engine process
- Exposing session-level JSON-RPC methods and emitting session-layer events
- Persisting session state to a database for resumption and replay

### 1.2 Supported Play Configurations

| Configuration | Human Clients | AI Nations | Description |
|---|---|---|---|
| **PvE** | 1 | 1..N | One human player competes against N AI-controlled nations in a live simulation |
| **Observer** | 0 | 2..N | No human control; engine runs AI vs AI; clients watch in real time |
| **Hot-Seat** | 2..M | 0..N | Multiple humans share one machine; each controls a nation; AI fills remaining slots |
| **Async Challenge** | 0 (headless) | 1..N | Human submits a strategy bundle offline; engine runs it; returns scored replay |

### 1.3 Explicit Out of Scope

The following are **permanently out of scope** for this specification and the CivLab engine:

- **Real-time online multiplayer** — no two human clients on separate machines compete in the same live session
- **Peer-to-peer synchronization** — no rollback netcode, no lockstep P2P protocol, no GGPO-style logic
- **Client-server lag compensation** — there is only one authoritative engine; clients are always read-only renderers
- **Matchmaking** — no matchmaking server, queue, or player ranking for live sessions
- **Remote human-vs-human** — human players are always either on the same machine (hot-seat) or playing offline (async challenge)

This scope boundary is **permanent and architectural**. The engine is not designed for and will never support real-time online PvP. Any future competitive mode is always async (submit strategy → engine scores it).

### 1.4 Session Topology

```
┌─────────────────────────────────────────────────────────────────────────┐
│                        CivLab Engine Process                            │
│                                                                         │
│  ┌─────────────────────────────────────────────────────────────────┐   │
│  │                      SessionManager                              │   │
│  │  session_id: UUIDv7  |  type: PvE  |  tick: u64                │   │
│  └───────────────────────────┬─────────────────────────────────────┘   │
│                              │ owns                                     │
│              ┌───────────────┼───────────────┐                         │
│              ▼               ▼               ▼                         │
│  ┌─────────────────┐ ┌─────────────┐ ┌─────────────────┐              │
│  │   SimCore       │ │ AiController│ │  AiController   │              │
│  │ (ChaCha20Rng)   │ │  Nation #2  │ │   Nation #3     │              │
│  │ (BLAKE3 chain)  │ │ (MCTS/util) │ │  (heuristic)    │              │
│  │ (tick loop)     │ └─────────────┘ └─────────────────┘              │
│  └────────┬────────┘        │                 │                        │
│           │                 └────────┬─────────┘                       │
│           │         NationActions    │                                  │
│           ◄─────────────────────────┘                                  │
│           │ tick_broadcast                                              │
│           ▼                                                             │
│  ┌─────────────────────────────────────────────────────────────────┐   │
│  │              WebSocket Broadcast Hub                             │   │
│  └────────────────────────┬────────────────────────────────────────┘   │
│                           │                                             │
└───────────────────────────┼─────────────────────────────────────────────┘
                            │  JSON-RPC 2.0 / WebSocket
                 ┌──────────┴──────────┐
                 ▼                     ▼
     ┌─────────────────────┐  ┌─────────────────────┐
     │   Human Client      │  │  Observer Client     │
     │  (Pixi.js + React)  │  │  (read-only view)   │
     │  Nation #1 control  │  │  full or FoW stream │
     └─────────────────────┘  └─────────────────────┘

  Note: AI Controllers run INSIDE the engine process.
  They are not external processes or remote clients.
  Human client is the ONLY WebSocket participant with write authority.
  Observer clients are read-only.
```

---

## 2. Session Model

### 2.1 Session Types

```rust
/// Top-level discriminant for session configuration.
/// Determines which subsystems are activated and how turn authority is managed.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SessionType {
    /// One human nation client + 1..N AI nations.
    /// Human client has permanent input authority for their nation.
    Pve,

    /// Zero human clients. Engine runs AI vs AI.
    /// Connected clients are all observers (read-only).
    Observer,

    /// 2..M human nations sharing one machine.
    /// Turn token rotates; only the holder has input authority.
    HotSeat,

    /// No live clients. Human submits PolicyBundle; engine runs headless.
    /// Results returned as scored replay.
    AsyncChallenge,
}
```

### 2.2 Session Lifecycle State Machine

```
                    ┌─────────────────┐
              ┌────►│    CREATED      │◄──────── session.create RPC
              │     │  (config held)  │
              │     └────────┬────────┘
              │              │ session.start RPC
              │              │ guard: config valid, scenario loaded
              │              ▼
              │     ┌─────────────────┐
              │     │   CONFIGURING   │
              │     │ (loading assets,│
              │     │  seeding RNG,   │
              │     │  init AI ctrls) │
              │     └────────┬────────┘
              │              │ engine emits session.started.v1
              │              ▼
   session    │     ┌─────────────────┐◄──── session.resume RPC
   create ────┘     │    RUNNING      │
   (error)          │  (tick loop     │
                    │   active)       │
                    └──┬──────────┬───┘
                       │          │
          session.pause│          │ session.end RPC
                       ▼          │ OR: all human nations eliminated
              ┌─────────────────┐ │ OR: victory condition reached
              │    PAUSED       │ │ OR: max_ticks exceeded
              │  (tick loop     │ │
              │   suspended)    │ │
              └────────┬────────┘ │
                       │          │
           session.resume│        │
                       │          ▼
                       │ ┌─────────────────┐
                       └►│     ENDED       │
                         │ (immutable;     │
                         │  replay avail)  │
                         └─────────────────┘

  State Transitions:
  CREATED     → CONFIGURING  : session.start RPC received, config valid
  CONFIGURING → RUNNING      : all AI controllers initialized, RNG seeded
  CONFIGURING → CREATED      : initialization failure (engine emits error)
  RUNNING     → PAUSED       : session.pause RPC (human client or admin)
  RUNNING     → ENDED        : end condition met or session.end RPC
  PAUSED      → RUNNING      : session.resume RPC
  PAUSED      → ENDED        : session.end RPC while paused

  Guards:
  - CREATED → CONFIGURING: scenario_ref must resolve; seed must be valid u64
  - RUNNING → PAUSED: only human client or ADMIN role may pause (not observer)
  - * → ENDED: final BLAKE3 hash recorded; state frozen; no further mutations
```

### 2.3 Session Config Schema (TOML)

```toml
# CivLab Session Configuration — canonical format
# All fields required unless marked optional.

[session]
# UUIDv7 assigned by engine on creation. Client sends empty string on create.
session_id = ""

# One of: "pve" | "observer" | "hot_seat" | "async_challenge"
session_type = "pve"

# Reference to a versioned scenario file (TOML or JSON).
# Engine resolves this from the scenario registry.
# Format: "<scenario_name>@<version>"  e.g. "mediterranean_start@1.2"
scenario_ref = "mediterranean_start@1.2"

# Deterministic seed for ChaCha20Rng. Must be a valid u64.
# If zero, engine generates a random seed and returns it in session.created.v1.
seed = 0

# Number of ticks per second during RUNNING state.
# Range: 1..=100. Default: 10 (100ms/tick = real-time).
# Can be changed at runtime via session.set_speed.
tick_speed_ms = 100

# Maximum ticks before session auto-ends. 0 = no limit.
# 1 tick = 100ms real time. 36000 ticks = 1 hour real time.
max_ticks = 0

# Autosave interval in ticks. 0 = no autosave. Default: 100.
autosave_interval_ticks = 100

[session.human_nations]
# Array of human-controlled nation IDs.
# For PvE: exactly one entry.
# For hot_seat: 2..M entries.
# For observer/async_challenge: empty array.
nation_ids = ["nation_player_1"]

# Hot-seat specific: turn mode
# "sequential" = one human acts, then next (default)
# "simultaneous" = all humans submit in parallel, engine resolves at tick end
turn_mode = "sequential"

# Ticks before auto-pass if human doesn't act (hot-seat only).
# 0 = no timeout. Default: 50 (5 seconds at 100ms/tick).
turn_timeout_ticks = 50

[[session.ai_nations]]
# Nation ID must match a nation defined in the scenario.
nation_id = "nation_rome"

# Difficulty level 1..=5. Maps to DifficultyConfig in CIV-0400.
# 1 = Novice, 2 = Standard, 3 = Veteran, 4 = Expert, 5 = Legendary
difficulty = 3

# Personality archetype from CIV-0400 Section 4.
# One of: "expansionist" | "militarist" | "trader" | "scientist" | "diplomat" | "random"
# "random" = engine selects deterministically from seed.
personality = "militarist"

# Optional per-nation RNG sub-seed offset. Added to session seed.
# Allows different AI nations to have divergent random sequences.
# Default: nation index in config array (0-indexed).
rng_offset = 0

[[session.ai_nations]]
nation_id = "nation_carthage"
difficulty = 2
personality = "trader"
rng_offset = 1

[session.observer]
# Whether observer clients see all nations' full state (no fog of war).
# false = observers must specify a fog_nation_id in session.observe RPC.
omniscient_by_default = true
```

### 2.4 Session ID

All sessions use **UUIDv7** as their primary identifier. UUIDv7 encodes a millisecond-precision Unix timestamp in the most-significant bits, enabling chronological sorting by session ID without a separate `created_at` index.

```
Format: xxxxxxxx-xxxx-7xxx-yxxx-xxxxxxxxxxxx
        ^^^^^^^^^^^^^^^^                    = 48-bit millisecond timestamp
                        ^^^^                = 12-bit random sequence
                             ^^^^^^^^^^^^   = 62-bit random node
```

The engine generates the session ID at `session.create` RPC time. The ID is included in **every** event and **every** RPC response for that session. Clients MUST NOT generate or guess session IDs.

### 2.5 Session Registry

The engine maintains an in-memory `SessionRegistry` mapping `SessionId -> Arc \< Mutex\<SessionState\>>`. On engine startup, incomplete sessions (state != ENDED) are reloaded from the `sessions` database table and offered to clients for resumption.

---

## 3. Human Turn Model (Hot-Seat)

### 3.1 Overview

Hot-seat mode allows multiple human nations to be controlled from a single WebSocket client connection, with a turn token that rotates between nations. Only the nation currently holding the turn token may submit actions. The engine enforces this: actions submitted by a nation not holding the token are rejected with error code `-32001` (not your turn).

### 3.2 Turn Token

```rust
/// Identifies which human nation currently holds input authority.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TurnToken {
    /// The nation that currently has input authority.
    pub nation_id: NationId,

    /// Tick on which this token was issued.
    pub issued_at_tick: u64,

    /// Tick after which the engine auto-passes if no actions received.
    /// None = no timeout enforced.
    pub expires_at_tick: Option<u64>,

    /// Sequential counter. Increments on every handoff.
    /// Used by client to detect missed handoffs.
    pub sequence: u64,
}
```

### 3.3 Sequential Turn Mode

In sequential mode, human nations take turns in the order specified by `session.human_nations.nation_ids`. AI nations act every tick regardless of whose human turn it is.

```
Tick N:   TurnToken { nation_id: "player_1", sequence: 0 }
          Human player_1 submits actions.
          AI nations submit actions (every tick, unaffected by turn token).
          Engine resolves all actions in NationAction queue order.

Tick N+1: player_1 calls session.turn.end RPC.
          Engine validates: player_1 holds token, tick matches.
          Engine issues TurnToken { nation_id: "player_2", sequence: 1 }.
          Engine emits session.turn.start.v1 { nation_id: "player_2", ... }.
          Client switches UI to player_2 controls.

Tick N+K: If expires_at_tick reached with no session.turn.end from player_2:
          Engine auto-passes: emits session.turn.auto_passed.v1.
          Advances token to next human nation.
```

### 3.4 Simultaneous Action Mode

In simultaneous mode, all human nations submit actions during the same ticks. The engine collects all human submissions and resolves them together at the tick boundary.

```
Each tick:
  - All human nations may submit actions in parallel.
  - Engine waits for all humans to call session.turn.end (or timeout).
  - Engine resolves all actions (human + AI) in deterministic order.
  - Engine emits tick_broadcast with updated state.
  - Next tick begins.

Submission order for simultaneous mode:
  - Human actions are sorted by (nation_id lexicographic order) within a tick.
  - This ensures determinism even if clients submit at different wall-clock times.
```

### 3.5 Turn Handoff Protocol (Sequential Mode)

**Client calls `session.turn.end`:**

```json
{
  "jsonrpc": "2.0",
  "id": "client-req-42",
  "method": "session.turn.end",
  "params": {
    "session_id": "01945b2a-3f8c-7e4d-a123-456789abcdef",
    "nation_id": "player_1",
    "token_sequence": 0
  }
}
```

**Engine response (success):**

```json
{
  "jsonrpc": "2.0",
  "id": "client-req-42",
  "result": {
    "accepted": true,
    "next_nation_id": "player_2",
    "token_sequence": 1
  }
}
```

**Engine emits `session.turn.start.v1` to client:**

```json
{
  "jsonrpc": "2.0",
  "method": "session.turn.start.v1",
  "params": {
    "session_id": "01945b2a-3f8c-7e4d-a123-456789abcdef",
    "nation_id": "player_2",
    "issued_at_tick": 47,
    "expires_at_tick": 97,
    "token_sequence": 1
  }
}
```

**Engine response (error — wrong nation):**

```json
{
  "jsonrpc": "2.0",
  "id": "client-req-43",
  "error": {
    "code": -32001,
    "message": "Not your turn",
    "data": {
      "submitted_by": "player_2",
      "current_token_holder": "player_1",
      "token_sequence": 0
    }
  }
}
```

### 3.6 Auto-Pass on Timeout

If a human nation holds the turn token and `expires_at_tick` is reached without a `session.turn.end` call:

1. Engine records the auto-pass event internally.
2. Engine emits `session.turn.auto_passed.v1` to all connected clients.
3. Engine advances the turn token to the next human nation in rotation.
4. Engine emits `session.turn.start.v1` for the new token holder.

No human action is inserted for the auto-passed tick. The nation state carries forward unchanged from the prior tick, subject to AI-applied game events (e.g. ongoing combat, resource decay).

---

## 4. AI Nation Controller Integration

### 4.1 Overview

All AI nations are controlled by `AiNationController` instances that live **inside the engine process**. They are not external processes, not remote clients, and do not communicate over WebSocket. They execute synchronously within the tick loop on the engine's thread pool.

This design provides:
- **Determinism**: AI controllers share the same `ChaCha20Rng` instance as the simulation, sub-seeded per nation
- **Fair play**: AI submits actions through the same `NationAction` queue as human clients
- **Performance**: No serialization overhead; direct in-memory state access (read-only)
- **Debuggability**: AI decision events are emitted alongside simulation events

### 4.2 `AiNationController` Trait

```rust
use crate::sim::{SimState, NationId, NationAction};
use rand_chacha::ChaCha20Rng;

/// Core trait that all AI nation implementations must satisfy.
///
/// Implementations must be:
/// - Deterministic: identical (state, rng) → identical output
/// - Pure: no side effects, no system time, no global mutable state
/// - Bounded: must complete within AI_DECISION_BUDGET_MS (configurable, default 8ms)
///
/// Trait is object-safe; engine stores Box<dyn AiNationController>.
pub trait AiNationController: Send + Sync {
    /// Return the nation ID this controller governs.
    fn nation_id(&self) -> NationId;

    /// Produce the set of actions this nation will take this tick.
    ///
    /// `state`:  current SimState (read-only snapshot, pre-tick)
    /// `rng`:    per-nation ChaCha20Rng sub-stream (mutable; advance freely)
    ///
    /// Returned actions are enqueued into the NationAction queue and processed
    /// identically to human-submitted actions. Order within the returned Vec
    /// is preserved; the engine applies them in declared order.
    ///
    /// MUST NOT:
    ///   - access any global/thread-local state
    ///   - use std::time or any clock
    ///   - use f64 arithmetic (use i32 fixed-point scaled by 1000)
    ///   - panic (errors must be logged and empty Vec returned)
    fn decide_actions(
        &mut self,
        state: &SimState,
        rng: &mut ChaCha20Rng,
    ) -> Vec<NationAction>;

    /// Called once after session config is applied and before RUNNING state.
    /// Use to pre-allocate planning buffers, warm MCTS tree, etc.
    fn initialize(&mut self, config: &AiNationConfig, state: &SimState);

    /// Human-readable name for logging and debug events.
    fn controller_name(&self) -> &'static str;
}
```

### 4.3 Controller Implementations

| Difficulty | Implementation | Strategy |
|---|---|---|
| 1 (Novice) | `NoviceController` | Fixed heuristics, no lookahead, high randomness |
| 2 (Standard) | `StandardController` | Utility scoring, shallow 2-ply lookahead |
| 3 (Veteran) | `VeteranController` | Full utility scoring, memory, reactive diplomacy |
| 4 (Expert) | `ExpertController` | PUCT MCTS, 500 simulations/tick budget |
| 5 (Legendary) | `LegendaryController` | Paranoid MCTS, 2000 sims/tick, full CIV-0400 pipeline |

All implementations satisfy `AiNationController`. The engine instantiates the correct variant from `DifficultyConfig` per CIV-0400 Section 10.

### 4.4 Per-Nation RNG Sub-Stream

Each AI nation receives a deterministically derived `ChaCha20Rng` sub-stream. The engine initializes these at session start using a key-derivation scheme:

```rust
/// Derive a per-nation RNG from the session seed.
///
/// Uses BLAKE3 keyed hash to derive a 32-byte seed for each nation.
/// The session seed and nation's rng_offset are both folded in,
/// ensuring each nation has a divergent random stream even at offset=0.
fn derive_nation_rng(session_seed: u64, nation_id: &NationId, rng_offset: u64) -> ChaCha20Rng {
    use blake3::Hasher;
    let mut hasher = Hasher::new_keyed(b"civlab-nation-rng-v1\0\0\0\0\0\0\0\0\0\0\0\0");
    hasher.update(&session_seed.to_le_bytes());
    hasher.update(nation_id.as_bytes());
    hasher.update(&rng_offset.to_le_bytes());
    let hash = hasher.finalize();
    let seed: [u8; 32] = hash.into();
    ChaCha20Rng::from_seed(seed)
}
```

### 4.5 Tick-Loop Integration

Within the tick loop (defined in CIV-0001), AI controllers execute in **Phase 2: AI Decision** which runs after human actions are collected but before action resolution:

```
Tick N execution sequence:
  Phase 1: Collect pending NationActions from human client WebSocket buffer
  Phase 2: AI Decision Phase — for each AiNationController in parallel:
             actions = controller.decide_actions(&state_snapshot, &mut nation_rng)
             enqueue actions into NationAction queue
  Phase 3: Resolve all NationActions (human + AI) in deterministic order
  Phase 4: Apply state transitions (economy, combat, diplomacy, etc.)
  Phase 5: Compute BLAKE3 hash of resulting state
  Phase 6: Serialize snapshot, broadcast to all WebSocket clients
  Phase 7: Emit tick events
```

AI controllers in Phase 2 run on the Rayon thread pool, one task per nation. Because each nation's RNG and state are independent inputs, there are no data races. The `SimState` passed to `decide_actions` is a read-only snapshot cloned before Phase 2 begins.

### 4.6 Personality Assignment

Personality archetypes from CIV-0400 Section 4 are assigned via session config. If `personality = "random"`, the engine selects deterministically:

```rust
fn resolve_personality(
    config_personality: &str,
    nation_id: &NationId,
    session_seed: u64,
) -> PersonalityArchetype {
    if config_personality != "random" {
        return PersonalityArchetype::from_str(config_personality)
            .expect("validated at session creation");
    }
    // Deterministic selection: hash (seed, nation_id) mod num_archetypes
    let mut hasher = blake3::Hasher::new();
    hasher.update(&session_seed.to_le_bytes());
    hasher.update(nation_id.as_bytes());
    let hash_bytes: [u8; 32] = hasher.finalize().into();
    let idx = u64::from_le_bytes(hash_bytes[0..8].try_into().unwrap())
        % PersonalityArchetype::COUNT as u64;
    PersonalityArchetype::from_index(idx as usize)
}
```

### 4.7 AI Decision Budget Enforcement

To keep the tick loop deterministic in wall-clock time, each AI controller is given a budget:

```rust
pub struct AiBudgetConfig {
    /// Hard limit per controller per tick.
    /// Default: 8ms. At 100ms/tick, 8ms leaves margin for 12 controllers.
    pub max_decision_ms: u64,

    /// If controller exceeds budget: log warning, return partial actions so far.
    /// The partial actions already enqueued are valid and applied.
    /// This is NOT an error state; the simulation continues deterministically.
    pub on_budget_exceeded: BudgetPolicy,
}

pub enum BudgetPolicy {
    /// Return whatever actions were computed before the deadline.
    ReturnPartial,
    /// Return empty Vec (no actions this tick). Safer but weaker AI behavior.
    ReturnEmpty,
}
```

Budget overrun is logged as a non-fatal warning. The tick hash is computed on the actual resulting state, so budget overruns do not affect determinism of the simulation — only AI decision quality for that tick.

---

## 5. Observer / Spectator Mode

### 5.1 Overview

Observer clients connect via the same WebSocket + JSON-RPC 2.0 protocol as human clients. They receive tick broadcasts and session events but **cannot inject any NationActions**. The engine enforces observer read-only status: any action submission from an observer connection is rejected with error code `-32002` (observer clients are read-only).

### 5.2 Fog of War Options

```
Option A: Omniscient Observer
  - Receives full SimState snapshot every tick.
  - All nations' territory, resources, unit positions visible.
  - Used for: debugging, research, AI-vs-AI spectating, replays.

Option B: Fog-of-War Observer (per-nation perspective)
  - Receives the SimState snapshot filtered through nation X's visibility.
  - Sees what nation X sees: explored tiles, visible units, known diplomacy.
  - Used for: watching a specific nation's game experience.
  - fog_nation_id specified in session.observe RPC.
```

The fog-of-war filtering is applied server-side before serialization. The observer never receives hidden state even transiently.

### 5.3 `session.observe` RPC

```json
// Request: subscribe as observer
{
  "jsonrpc": "2.0",
  "id": "obs-req-1",
  "method": "session.observe",
  "params": {
    "session_id": "01945b2a-3f8c-7e4d-a123-456789abcdef",
    // null = omniscient (full state).
    // NationId = fog-of-war filtered to that nation's perspective.
    "fog_nation_id": null,
    // Optional: only send every Nth tick (reduces bandwidth for fast simulations).
    // Default: 1 (every tick). Range: 1..=100.
    "tick_stride": 1
  }
}

// Response: acknowledged
{
  "jsonrpc": "2.0",
  "id": "obs-req-1",
  "result": {
    "subscribed": true,
    "session_id": "01945b2a-3f8c-7e4d-a123-456789abcdef",
    "fog_mode": "omniscient",
    "current_tick": 1042
  }
}
```

### 5.4 Replay Observer

Observers may connect to a **completed session** and scrub the recorded tick timeline:

```json
// Subscribe to replay at a specific starting tick
{
  "jsonrpc": "2.0",
  "id": "obs-req-2",
  "method": "session.observe",
  "params": {
    "session_id": "01945b2a-3f8c-7e4d-a123-456789abcdef",
    "fog_nation_id": null,
    "replay": {
      // Start playback from this tick.
      "start_tick": 500,
      // Replay speed multiplier. 1.0 = real time. 10.0 = 10x speed.
      "speed_multiplier": 5.0
    }
  }
}

// Seek to a specific tick in replay
{
  "jsonrpc": "2.0",
  "id": "obs-req-3",
  "method": "session.replay.seek",
  "params": {
    "session_id": "01945b2a-3f8c-7e4d-a123-456789abcdef",
    "target_tick": 2000
  }
}
```

Replay is served from the `.civreplay` file (CIV-0001 format). The engine reconstructs state at any tick by replaying from the nearest save checkpoint. Seek operations with no checkpoint within 500 ticks replay from the beginning (this is a cold seek; warm seeks from a nearby checkpoint complete in \< 100ms).

---

## 6. Async Challenge Mode

### 6.1 Overview

Async Challenge mode allows users to compete without a live session. The user prepares a `ChallengeBundle` containing their strategy (a `PolicyBundle` defining economic priorities, military posture, diplomatic stance, etc.) and submits it via REST API. The engine runs the challenge headless on the server and returns a scored replay.

This enables:
- Offline competition (no scheduling required)
- Deterministic fairness (same seed, same AI opponents, same scenario for all challengers)
- Leaderboard ranking by objective score
- Academic study of strategy effectiveness

### 6.2 ChallengeBundle Schema

```json
{
  "challenge_bundle_version": "1",
  "scenario_ref": "mediterranean_start@1.2",

  "human_strategy": {
    "policy_bundle_version": "1",
    "nation_id": "nation_player_1",

    "economic_policy": {
      "tax_rate_pct": 35,
      "infrastructure_spend_pct": 40,
      "military_spend_pct": 25,
      "research_spend_pct": 0
    },

    "military_posture": "defensive",

    "diplomatic_stances": {
      "nation_rome": "neutral",
      "nation_carthage": "friendly"
    },

    "tech_priorities": ["iron_working", "sailing", "masonry"],

    "city_build_orders": {
      "city_capital": ["granary", "barracks", "library"]
    },

    // Conditional rules evaluated by a simple rule engine.
    // Allows dynamic strategy without Turing-complete scripting.
    "conditional_rules": [
      {
        "condition": "treasury < 50",
        "action": "set_tax_rate_pct(45)"
      },
      {
        "condition": "any_neighbor_at_war AND military_strength_ratio < 0.8",
        "action": "set_military_posture(aggressive)"
      }
    ]
  },

  "ai_opponents": [
    { "nation_id": "nation_rome", "difficulty": 3, "personality": "militarist" },
    { "nation_id": "nation_carthage", "difficulty": 3, "personality": "trader" }
  ],

  // Seed is fixed per challenge definition (not per submission).
  // All submissions to the same challenge use the same seed.
  "seed": 7392847561029384756,

  "max_ticks": 36000,

  "scoring": {
    "weights": {
      "final_score": 0.4,
      "survival_ticks": 0.2,
      "territory_at_end": 0.2,
      "tech_level_at_end": 0.1,
      "treasury_at_end": 0.1
    }
  }
}
```

### 6.3 Async Challenge Lifecycle

```
User                    REST API              Engine                   DB
 │                          │                    │                      │
 │  POST /challenges         │                   │                      │
 │  (ChallengeBundle)        │                   │                      │
 ├──────────────────────────►│                   │                      │
 │                          │  validate bundle   │                      │
 │                          │  generate job_id   │                      │
 │                          │  INSERT challenge  │                      │
 │                          ├───────────────────────────────────────────►
 │  202 Accepted            │                   │                      │
 │  { challenge_id, status} │                   │                      │
 │◄──────────────────────────┤                   │                      │
 │                          │  enqueue job       │                      │
 │                          ├──────────────────►│                      │
 │                          │                   │  run headless        │
 │                          │                   │  simulation          │
 │                          │                   │  (no WS client)      │
 │                          │                   │  emit events to log  │
 │                          │                   │  compute score        │
 │                          │                   ├─────────────────────►│
 │                          │                   │  INSERT result       │
 │                          │                   │  store replay blob   │
 │                          │                   │                      │
 │  GET /challenges/{id}     │                   │                      │
 ├──────────────────────────►│                   │                      │
 │                          ├───────────────────────────────────────────►
 │                          │                   │  SELECT result        │
 │                          │◄──────────────────────────────────────────┤
 │  200 OK                  │                   │                      │
 │  { status: completed,    │                   │                      │
 │    score, metrics }       │                   │                      │
 │◄──────────────────────────┤                   │                      │
 │                          │                   │                      │
 │  GET /challenges/{id}/replay                  │                      │
 ├──────────────────────────►│                   │                      │
 │  200 OK (replay blob)     │                   │                      │
 │◄──────────────────────────┤                   │                      │
```

### 6.4 Scoring Formula

The challenge score is a weighted sum of final metrics normalized against a baseline AI-only run (same scenario, same seed, no human player — replaced by a Standard difficulty AI):

```
score = Σ_i [ weight_i × normalize(metric_i, baseline_i) ]

normalize(value, baseline) = (value - baseline) / max(baseline, 1)
  → clamped to [-1.0, 1.0] in fixed-point i32 (×1000)

Default weights:
  final_score:        0.40
  survival_ticks:     0.20
  territory_at_end:   0.20
  tech_level_at_end:  0.10
  treasury_at_end:    0.10

Score range: [-1.0, 1.0] → displayed as percentage vs baseline.
  0.0 = matched baseline AI performance
  0.5 = 50% better than baseline
 -0.5 = 50% worse than baseline
```

The baseline run is computed once per challenge definition and cached. All submissions to the same challenge use the same cached baseline.

### 6.5 REST Endpoints

```
POST   /api/v1/challenges
       Body: ChallengeBundle (JSON)
       Response: 202 { challenge_id: UUIDv7, status: "queued" }

GET    /api/v1/challenges/{challenge_id}
       Response: 200 {
         challenge_id, status, submitted_at, completed_at?,
         score?, final_metrics?, error_message?
       }

GET    /api/v1/challenges/{challenge_id}/replay
       Response: 200 application/octet-stream (.civreplay file)
       Or: 404 if not completed yet

GET    /api/v1/challenges/leaderboard?scenario_ref=mediterranean_start@1.2
       Response: 200 { entries: [{ rank, user_id, score, submitted_at }] }

GET    /api/v1/challenges/{challenge_id}/events
       Response: 200 { events: [...] }  (full event log for the run)
```

### 6.6 Headless Engine Execution

The async challenge engine runs without any WebSocket client. Key differences from a live session:

- `tick_speed_ms` is set to minimum (1ms between ticks) — headless runs as fast as the CPU allows
- No client broadcast — events are written to a memory-mapped log only
- No pause/resume — runs until `max_ticks` or victory condition
- Memory usage is bounded: state snapshots are written to disk every 500 ticks and evicted from RAM

---

## 7. Pause / Resume / Speed Control

### 7.1 Pause

`session.pause` halts the tick loop. The engine finishes the current tick in progress (if any), then enters `PAUSED` state. No new ticks are generated while paused. Human client actions submitted while paused are queued and applied when the session resumes.

**Authorization:** Only a human client (not an observer) may call `session.pause`. The engine checks the calling client's role.

```
Guard: calling client must have role "human" or "admin"
Guard: session must be in RUNNING state
Effect: tick loop suspends after current tick completes
Effect: emits session.paused.v1
```

### 7.2 Resume

`session.resume` resumes the tick loop from the exact state at pause time. The BLAKE3 hash chain continues from where it left off; no ticks are skipped.

```
Guard: session must be in PAUSED state
Guard: calling client must have role "human" or "admin"
Effect: tick loop resumes
Effect: emits session.resumed.v1
```

### 7.3 Speed Control

The tick speed is the real-wall-clock time between tick executions. At `tick_speed_ms = 100` (the default), the engine runs at 1:1 real time (10 ticks per second). At `tick_speed_ms = 10`, the engine runs at 10x speed (100 ticks per second).

```
session.set_speed params:
  ticks_per_second: u32    Range: 1..=100  Default: 10
    → tick_speed_ms = 1000 / ticks_per_second

Effect: next tick uses new interval
Effect: emits session.speed_changed.v1 with new ticks_per_second
```

Changing speed does not affect simulation determinism. The BLAKE3 hash chain is over simulation state, not wall-clock timestamps.

### 7.4 Fast-Forward

`session.fast_forward` runs the simulation headless (no broadcast) from the current tick to `target_tick`, then resumes normal operation.

```
session.fast_forward params:
  target_tick: u64     Must be > current_tick

Guard: target_tick > current_tick
Guard: session must be in RUNNING or PAUSED state

Execution:
  1. Pause client broadcast (clients receive no intermediate tick snapshots).
  2. Run simulation at maximum speed (tick_speed_ms = 1) until target_tick.
  3. Re-enable broadcast.
  4. Send full state snapshot at target_tick to all clients.
  5. Resume at previously configured speed.
  6. Emit session.fast_forward_completed.v1 { from_tick, to_tick, elapsed_ms }.
```

Fast-forward is non-interruptible: `session.pause` calls during a fast-forward are queued and applied after completion.

---

## 8. Save / Load Integration

### 8.1 Save

`session.save` serializes the full `SimState` at the current tick into a named slot. Full format is defined in CIV-1000; this section defines only the session-layer interface.

```
session.save params:
  slot_name: String    Max 64 chars, alphanumeric + hyphens
  description: Option<String>   Max 256 chars, free text

Effect: serializes SimState to binary blob (CIV-1000 format)
Effect: computes BLAKE3 hash of the blob
Effect: writes to session_saves table
Effect: emits session.save.completed.v1
```

### 8.2 Load

`session.load` restores a session from a saved slot. The session must be in PAUSED or ENDED state to load. Loading from an ENDED session creates a new session branching from that save point.

```
session.load params:
  slot_name: String    Must refer to an existing slot for this session_id

Guard: slot exists and BLAKE3 hash matches stored hash (integrity check)
Effect: replaces current SimState with deserialized state
Effect: resets BLAKE3 chain to value at save tick
Effect: AI controllers re-initialize from loaded state
Effect: emits session.loaded.v1 { slot_name, restored_tick }
Effect: transitions session to PAUSED (human must resume)
```

### 8.3 Autosave

When `autosave_interval_ticks > 0`, the engine automatically saves every N ticks to the slot named `"autosave"`. Autosave overwrites the previous autosave slot (only one autosave slot is maintained per session).

```
Autosave trigger: current_tick % autosave_interval_ticks == 0
Slot name: "autosave" (fixed, non-user-configurable)
Behavior: overwrite previous autosave silently
          user-created slots are never overwritten by autosave
```

### 8.4 Save Slot Schema Reference

The `session_saves` table is defined in Section 11. State blob format is defined in CIV-1000. This spec does not duplicate CIV-1000 content.

---

## 9. Event Schema

All session-layer events are emitted as JSON-RPC 2.0 notifications (no `id` field, no response expected). Events use versioned method names (`session.xxx.v1`) to allow non-breaking additions in future versions.

### 9.1 `session.created.v1`

```json
{
  "jsonrpc": "2.0",
  "method": "session.created.v1",
  "params": {
    "session_id": "01945b2a-3f8c-7e4d-a123-456789abcdef",
    "session_type": "pve",
    "scenario_ref": "mediterranean_start@1.2",
    "seed": 7392847561029384756,
    "tick_speed_ms": 100,
    "human_nations": ["nation_player_1"],
    "ai_nations": [
      { "nation_id": "nation_rome", "difficulty": 3, "personality": "militarist" },
      { "nation_id": "nation_carthage", "difficulty": 2, "personality": "trader" }
    ],
    "created_at": "2026-02-21T14:32:00.000Z"
  }
}
```

### 9.2 `session.started.v1`

```json
{
  "jsonrpc": "2.0",
  "method": "session.started.v1",
  "params": {
    "session_id": "01945b2a-3f8c-7e4d-a123-456789abcdef",
    "initial_tick": 0,
    "initial_blake3_hash": "a1b2c3d4e5f6...",
    "tick_speed_ms": 100,
    "started_at": "2026-02-21T14:32:01.500Z"
  }
}
```

### 9.3 `session.paused.v1`

```json
{
  "jsonrpc": "2.0",
  "method": "session.paused.v1",
  "params": {
    "session_id": "01945b2a-3f8c-7e4d-a123-456789abcdef",
    "paused_at_tick": 1042,
    "blake3_hash_at_pause": "d4e5f6a7b8c9...",
    "paused_by": "human_client",
    "paused_at": "2026-02-21T14:47:23.100Z"
  }
}
```

### 9.4 `session.resumed.v1`

```json
{
  "jsonrpc": "2.0",
  "method": "session.resumed.v1",
  "params": {
    "session_id": "01945b2a-3f8c-7e4d-a123-456789abcdef",
    "resumed_at_tick": 1042,
    "tick_speed_ms": 100,
    "resumed_at": "2026-02-21T14:50:00.000Z"
  }
}
```

### 9.5 `session.ended.v1`

```json
{
  "jsonrpc": "2.0",
  "method": "session.ended.v1",
  "params": {
    "session_id": "01945b2a-3f8c-7e4d-a123-456789abcdef",
    "final_tick": 18430,
    "final_blake3_hash": "f7a8b9c0d1e2...",
    "end_reason": "victory_condition",
    // One of: "victory_condition" | "all_eliminated" | "max_ticks" | "client_request" | "error"
    "winner_nation_id": "nation_player_1",
    "final_scores": {
      "nation_player_1": { "score": 4872, "territory": 142, "tech_level": 8 },
      "nation_rome": { "score": 3241, "territory": 98, "tech_level": 7 }
    },
    "replay_id": "01945b40-0000-7000-0000-000000000001",
    "ended_at": "2026-02-21T16:22:15.000Z"
  }
}
```

### 9.6 `session.turn.start.v1`

Emitted in hot-seat mode when a human nation receives the turn token.

```json
{
  "jsonrpc": "2.0",
  "method": "session.turn.start.v1",
  "params": {
    "session_id": "01945b2a-3f8c-7e4d-a123-456789abcdef",
    "nation_id": "nation_player_2",
    "issued_at_tick": 47,
    "expires_at_tick": 97,
    "token_sequence": 1,
    "turn_mode": "sequential"
  }
}
```

### 9.7 `session.turn.end.v1`

Emitted in hot-seat mode when a human nation releases the turn token.

```json
{
  "jsonrpc": "2.0",
  "method": "session.turn.end.v1",
  "params": {
    "session_id": "01945b2a-3f8c-7e4d-a123-456789abcdef",
    "nation_id": "nation_player_1",
    "ended_at_tick": 47,
    "token_sequence": 0,
    "action_count_this_turn": 3,
    "auto_passed": false
  }
}
```

### 9.8 `session.speed_changed.v1`

```json
{
  "jsonrpc": "2.0",
  "method": "session.speed_changed.v1",
  "params": {
    "session_id": "01945b2a-3f8c-7e4d-a123-456789abcdef",
    "previous_ticks_per_second": 10,
    "new_ticks_per_second": 50,
    "new_tick_speed_ms": 20,
    "effective_at_tick": 1043,
    "changed_at": "2026-02-21T14:48:00.000Z"
  }
}
```

### 9.9 `session.save.completed.v1`

```json
{
  "jsonrpc": "2.0",
  "method": "session.save.completed.v1",
  "params": {
    "session_id": "01945b2a-3f8c-7e4d-a123-456789abcdef",
    "slot_name": "my-checkpoint-1",
    "saved_at_tick": 1050,
    "blake3_hash": "c3d4e5f6a7b8...",
    "blob_size_bytes": 2097152,
    "is_autosave": false,
    "saved_at": "2026-02-21T14:48:30.000Z"
  }
}
```

### 9.10 `challenge.submitted.v1`

```json
{
  "jsonrpc": "2.0",
  "method": "challenge.submitted.v1",
  "params": {
    "challenge_id": "01945b50-0000-7000-0000-000000000001",
    "scenario_ref": "mediterranean_start@1.2",
    "human_nation_id": "nation_player_1",
    "seed": 7392847561029384756,
    "max_ticks": 36000,
    "submitted_at": "2026-02-21T15:00:00.000Z",
    "queue_position": 3,
    "estimated_completion_seconds": 45
  }
}
```

### 9.11 `challenge.completed.v1`

```json
{
  "jsonrpc": "2.0",
  "method": "challenge.completed.v1",
  "params": {
    "challenge_id": "01945b50-0000-7000-0000-000000000001",
    "session_id": "01945b51-0000-7000-0000-000000000001",
    "replay_id": "01945b52-0000-7000-0000-000000000001",
    "score": 0.342,
    "score_pct_vs_baseline": 34.2,
    "final_metrics": {
      "final_score": 4872,
      "survival_ticks": 36000,
      "territory_at_end": 142,
      "tech_level_at_end": 8,
      "treasury_at_end": 3421
    },
    "baseline_metrics": {
      "final_score": 3200,
      "survival_ticks": 28000,
      "territory_at_end": 95,
      "tech_level_at_end": 6,
      "treasury_at_end": 2100
    },
    "completed_at": "2026-02-21T15:00:52.000Z",
    "wall_clock_runtime_ms": 52340
  }
}
```

---

## 10. JSON-RPC Methods

All session-layer methods follow the JSON-RPC 2.0 specification. Error codes in the range `-32001` to `-32099` are CivLab-specific (general JSON-RPC reserved range: `-32700` to `-32000`).

### CivLab Error Codes

| Code | Name | Description |
|---|---|---|
| -32001 | NOT_YOUR_TURN | Action submitted by nation not holding turn token |
| -32002 | OBSERVER_READ_ONLY | Observer client attempted to submit action |
| -32003 | SESSION_NOT_FOUND | session_id does not exist |
| -32004 | INVALID_SESSION_STATE | Method not valid in current session state |
| -32005 | INVALID_CONFIG | Session config failed validation |
| -32006 | SCENARIO_NOT_FOUND | scenario_ref does not resolve |
| -32007 | SAVE_SLOT_NOT_FOUND | Named save slot does not exist |
| -32008 | SAVE_INTEGRITY_FAILURE | BLAKE3 hash mismatch on load |
| -32009 | UNAUTHORIZED | Client role insufficient for this method |
| -32010 | FAST_FORWARD_IN_PROGRESS | Cannot perform operation during fast-forward |

### 10.1 `session.create`

```json
// Request
{
  "jsonrpc": "2.0",
  "id": "req-1",
  "method": "session.create",
  "params": {
    "session_type": "pve",
    "scenario_ref": "mediterranean_start@1.2",
    "seed": 0,
    "tick_speed_ms": 100,
    "max_ticks": 0,
    "autosave_interval_ticks": 100,
    "human_nations": {
      "nation_ids": ["nation_player_1"],
      "turn_mode": "sequential",
      "turn_timeout_ticks": 50
    },
    "ai_nations": [
      { "nation_id": "nation_rome", "difficulty": 3, "personality": "militarist", "rng_offset": 0 },
      { "nation_id": "nation_carthage", "difficulty": 2, "personality": "trader", "rng_offset": 1 }
    ],
    "observer_config": {
      "omniscient_by_default": true
    }
  }
}

// Response (success)
{
  "jsonrpc": "2.0",
  "id": "req-1",
  "result": {
    "session_id": "01945b2a-3f8c-7e4d-a123-456789abcdef",
    "status": "created",
    "resolved_seed": 7392847561029384756,
    "scenario_version": "1.2.0"
  }
}
```

### 10.2 `session.start`

```json
// Request
{
  "jsonrpc": "2.0",
  "id": "req-2",
  "method": "session.start",
  "params": {
    "session_id": "01945b2a-3f8c-7e4d-a123-456789abcdef"
  }
}

// Response (success)
{
  "jsonrpc": "2.0",
  "id": "req-2",
  "result": {
    "session_id": "01945b2a-3f8c-7e4d-a123-456789abcdef",
    "status": "running",
    "initial_tick": 0,
    "initial_blake3_hash": "a1b2c3d4e5f6..."
  }
}
```

### 10.3 `session.pause` and `session.resume`

```json
// Pause request
{
  "jsonrpc": "2.0",
  "id": "req-3",
  "method": "session.pause",
  "params": { "session_id": "01945b2a-3f8c-7e4d-a123-456789abcdef" }
}

// Pause response
{
  "jsonrpc": "2.0",
  "id": "req-3",
  "result": { "status": "paused", "paused_at_tick": 1042 }
}

// Resume request
{
  "jsonrpc": "2.0",
  "id": "req-4",
  "method": "session.resume",
  "params": { "session_id": "01945b2a-3f8c-7e4d-a123-456789abcdef" }
}

// Resume response
{
  "jsonrpc": "2.0",
  "id": "req-4",
  "result": { "status": "running", "resumed_at_tick": 1042 }
}
```

### 10.4 `session.set_speed` and `session.fast_forward`

```json
// Set speed request
{
  "jsonrpc": "2.0",
  "id": "req-5",
  "method": "session.set_speed",
  "params": {
    "session_id": "01945b2a-3f8c-7e4d-a123-456789abcdef",
    "ticks_per_second": 50
  }
}

// Set speed response
{
  "jsonrpc": "2.0",
  "id": "req-5",
  "result": {
    "previous_ticks_per_second": 10,
    "new_ticks_per_second": 50,
    "effective_at_tick": 1043
  }
}

// Fast-forward request
{
  "jsonrpc": "2.0",
  "id": "req-6",
  "method": "session.fast_forward",
  "params": {
    "session_id": "01945b2a-3f8c-7e4d-a123-456789abcdef",
    "target_tick": 5000
  }
}

// Fast-forward response (immediate acknowledge; completes asynchronously)
{
  "jsonrpc": "2.0",
  "id": "req-6",
  "result": {
    "from_tick": 1042,
    "target_tick": 5000,
    "status": "fast_forwarding"
  }
}
// Engine emits session.fast_forward_completed.v1 when done
```

### 10.5 `session.save` and `session.load`

```json
// Save request
{
  "jsonrpc": "2.0",
  "id": "req-7",
  "method": "session.save",
  "params": {
    "session_id": "01945b2a-3f8c-7e4d-a123-456789abcdef",
    "slot_name": "my-checkpoint-1",
    "description": "Before the war with Rome"
  }
}

// Save response
{
  "jsonrpc": "2.0",
  "id": "req-7",
  "result": {
    "slot_name": "my-checkpoint-1",
    "saved_at_tick": 1050,
    "blake3_hash": "c3d4e5f6a7b8...",
    "blob_size_bytes": 2097152
  }
}

// Load request
{
  "jsonrpc": "2.0",
  "id": "req-8",
  "method": "session.load",
  "params": {
    "session_id": "01945b2a-3f8c-7e4d-a123-456789abcdef",
    "slot_name": "my-checkpoint-1"
  }
}

// Load response
{
  "jsonrpc": "2.0",
  "id": "req-8",
  "result": {
    "slot_name": "my-checkpoint-1",
    "restored_tick": 1050,
    "status": "paused"
  }
}
```

### 10.6 `session.observe`

```json
// Observe request (omniscient)
{
  "jsonrpc": "2.0",
  "id": "req-9",
  "method": "session.observe",
  "params": {
    "session_id": "01945b2a-3f8c-7e4d-a123-456789abcdef",
    "fog_nation_id": null,
    "tick_stride": 1
  }
}

// Observe request (fog-of-war for specific nation)
{
  "jsonrpc": "2.0",
  "id": "req-10",
  "method": "session.observe",
  "params": {
    "session_id": "01945b2a-3f8c-7e4d-a123-456789abcdef",
    "fog_nation_id": "nation_rome",
    "tick_stride": 5
  }
}

// Response
{
  "jsonrpc": "2.0",
  "id": "req-9",
  "result": {
    "subscribed": true,
    "fog_mode": "omniscient",
    "current_tick": 1042,
    "tick_stride": 1
  }
}
```

### 10.7 `session.end`

```json
// End request
{
  "jsonrpc": "2.0",
  "id": "req-11",
  "method": "session.end",
  "params": {
    "session_id": "01945b2a-3f8c-7e4d-a123-456789abcdef",
    "reason": "client_request"
  }
}

// Response
{
  "jsonrpc": "2.0",
  "id": "req-11",
  "result": {
    "session_id": "01945b2a-3f8c-7e4d-a123-456789abcdef",
    "final_tick": 18430,
    "final_blake3_hash": "f7a8b9c0d1e2...",
    "replay_id": "01945b40-0000-7000-0000-000000000001",
    "status": "ended"
  }
}
```

---

## 11. DB Schema

All tables use PostgreSQL syntax. UUIDs are stored as `UUID` type (16 bytes). Timestamps are `TIMESTAMPTZ` (UTC). Large binary blobs are stored in the `session_saves` table; replay files are stored on object storage with a reference URL in the `sessions` table.

### 11.1 `sessions` Table

```sql
-- Primary record for every session created.
-- One row per session regardless of session_type.
CREATE TABLE sessions (
    id                      UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    -- UUIDv7, sortable by creation time
    session_id              UUID NOT NULL UNIQUE,
    session_type            TEXT NOT NULL CHECK (session_type IN (
                                'pve', 'observer', 'hot_seat', 'async_challenge'
                            )),
    status                  TEXT NOT NULL DEFAULT 'created' CHECK (status IN (
                                'created', 'configuring', 'running', 'paused', 'ended'
                            )),
    scenario_ref            TEXT NOT NULL,
    scenario_version        TEXT NOT NULL,
    seed                    BIGINT NOT NULL,           -- u64 stored as BIGINT
    tick_speed_ms           INT NOT NULL DEFAULT 100 CHECK (tick_speed_ms BETWEEN 10 AND 1000),
    max_ticks               BIGINT NOT NULL DEFAULT 0,  -- 0 = unlimited
    autosave_interval_ticks INT NOT NULL DEFAULT 100,
    current_tick            BIGINT NOT NULL DEFAULT 0,
    final_tick              BIGINT,                    -- NULL until ended
    final_blake3_hash       TEXT,                      -- hex string, NULL until ended
    end_reason              TEXT CHECK (end_reason IN (
                                'victory_condition', 'all_eliminated',
                                'max_ticks', 'client_request', 'error'
                            )),
    winner_nation_id        TEXT,                      -- NULL if no single winner
    -- JSON blob of full SessionConfig (for reconstruction)
    config_json             JSONB NOT NULL,
    -- URL to .civreplay file on object storage. NULL until ended.
    replay_url              TEXT,
    replay_id               UUID,
    created_at              TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    started_at              TIMESTAMPTZ,
    paused_at               TIMESTAMPTZ,
    ended_at                TIMESTAMPTZ,
    -- Metadata for querying
    human_nation_ids        TEXT[] NOT NULL DEFAULT '{}',
    ai_nation_ids           TEXT[] NOT NULL DEFAULT '{}'
);

CREATE INDEX idx_sessions_status ON sessions (status);
CREATE INDEX idx_sessions_created_at ON sessions (created_at DESC);
CREATE INDEX idx_sessions_session_type ON sessions (session_type);
```

### 11.2 `session_saves` Table

```sql
-- One row per save slot per session.
-- Slot "autosave" is overwritten on each autosave trigger.
-- User-created slots are never overwritten automatically.
CREATE TABLE session_saves (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    session_id      UUID NOT NULL REFERENCES sessions (session_id) ON DELETE CASCADE,
    slot_name       TEXT NOT NULL CHECK (
                        length(slot_name) BETWEEN 1 AND 64 AND
                        slot_name ~ '^[a-zA-Z0-9\-]+$'
                    ),
    tick            BIGINT NOT NULL,
    -- Full serialized SimState (CIV-1000 format), compressed with zstd.
    state_blob      BYTEA NOT NULL,
    -- BLAKE3 hash of state_blob for integrity verification on load.
    blake3_hash     TEXT NOT NULL,
    blob_size_bytes INT NOT NULL,
    description     TEXT CHECK (length(description) <= 256),
    is_autosave     BOOLEAN NOT NULL DEFAULT FALSE,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    -- Only one slot with a given name per session (autosave replaces itself).
    CONSTRAINT uq_session_slot UNIQUE (session_id, slot_name)
);

CREATE INDEX idx_session_saves_session_id ON session_saves (session_id);
CREATE INDEX idx_session_saves_tick ON session_saves (session_id, tick DESC);
```

### 11.3 `challenge_submissions` Table

```sql
-- One row per challenge submission (ChallengeBundle).
CREATE TABLE challenge_submissions (
    id                  UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    challenge_id        UUID NOT NULL UNIQUE,          -- UUIDv7
    -- Optional user association. NULL for anonymous submissions.
    user_id             UUID REFERENCES users (id) ON DELETE SET NULL,
    scenario_ref        TEXT NOT NULL,
    seed                BIGINT NOT NULL,
    max_ticks           BIGINT NOT NULL,
    -- Full ChallengeBundle JSON for replay and audit.
    bundle_json         JSONB NOT NULL,
    -- Serialized PolicyBundle for the human nation.
    policy_bundle_json  JSONB NOT NULL,
    human_nation_id     TEXT NOT NULL,
    status              TEXT NOT NULL DEFAULT 'queued' CHECK (status IN (
                            'queued', 'running', 'completed', 'failed', 'cancelled'
                        )),
    queue_position      INT,
    submitted_at        TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    started_at          TIMESTAMPTZ,
    completed_at        TIMESTAMPTZ,
    error_message       TEXT,
    -- FK to sessions when engine creates a headless session for this challenge.
    session_id          UUID REFERENCES sessions (session_id)
);

CREATE INDEX idx_challenge_submissions_status ON challenge_submissions (status);
CREATE INDEX idx_challenge_submissions_user_id ON challenge_submissions (user_id);
CREATE INDEX idx_challenge_submissions_scenario ON challenge_submissions (scenario_ref);
CREATE INDEX idx_challenge_submissions_submitted_at
    ON challenge_submissions (submitted_at DESC);
```

### 11.4 `challenge_results` Table

```sql
-- One row per completed challenge submission.
CREATE TABLE challenge_results (
    id                      UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    challenge_id            UUID NOT NULL UNIQUE REFERENCES challenge_submissions (challenge_id),
    session_id              UUID REFERENCES sessions (session_id),
    -- Final weighted score. Range: -1.0 to 1.0 stored as integer ×1000000 for precision.
    score_millionths        INT NOT NULL,
    -- Individual metric values at session end.
    final_score             BIGINT NOT NULL,
    survival_ticks          BIGINT NOT NULL,
    territory_at_end        INT NOT NULL,
    tech_level_at_end       INT NOT NULL,
    treasury_at_end         BIGINT NOT NULL,
    -- Baseline metric values (AI-only run, same scenario+seed).
    baseline_score          BIGINT NOT NULL,
    baseline_survival_ticks BIGINT NOT NULL,
    baseline_territory      INT NOT NULL,
    baseline_tech_level     INT NOT NULL,
    baseline_treasury       BIGINT NOT NULL,
    -- URL to .civreplay file for this challenge run.
    replay_url              TEXT,
    replay_id               UUID,
    wall_clock_runtime_ms   BIGINT NOT NULL,
    completed_at            TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Leaderboard query index: scenario + score DESC
CREATE INDEX idx_challenge_results_leaderboard
    ON challenge_results (
        (SELECT scenario_ref FROM challenge_submissions cs WHERE cs.challenge_id = challenge_results.challenge_id),
        score_millionths DESC
    );

-- Materialized view for leaderboard (refreshed on each challenge completion).
CREATE MATERIALIZED VIEW challenge_leaderboard AS
SELECT
    ROW_NUMBER() OVER (
        PARTITION BY cs.scenario_ref
        ORDER BY cr.score_millionths DESC
    ) AS rank,
    cs.scenario_ref,
    cs.user_id,
    cs.challenge_id,
    cr.score_millionths,
    cr.score_millionths::FLOAT / 1000000.0 AS score,
    cs.submitted_at,
    cr.completed_at
FROM challenge_results cr
JOIN challenge_submissions cs ON cs.challenge_id = cr.challenge_id
WHERE cs.status = 'completed'
WITH DATA;

CREATE UNIQUE INDEX ON challenge_leaderboard (scenario_ref, rank);
```

---

## 12. Rust Data Structures

All types are defined in `crate::session`. They derive `Serialize`, `Deserialize`, `Debug`, and `Clone` unless noted. Types marked `// non-Clone` contain non-cloneable internals (e.g. mutex guards).

### 12.1 `SessionConfig`

```rust
use std::collections::HashMap;
use uuid::Uuid;
use serde::{Deserialize, Serialize};

/// Complete configuration for a session.
/// Validated at session.create time and stored verbatim in sessions.config_json.
/// Immutable after session transitions to RUNNING.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SessionConfig {
    pub session_type: SessionType,
    pub scenario_ref: ScenarioRef,
    /// If zero, engine generates a random seed. Resolved seed returned in session.created.v1.
    pub seed: u64,
    /// Range: 10..=1000 (ms). Default: 100.
    pub tick_speed_ms: u32,
    /// 0 = unlimited.
    pub max_ticks: u64,
    /// 0 = disabled. Default: 100.
    pub autosave_interval_ticks: u64,
    pub human_nations: HumanNationConfig,
    pub ai_nations: Vec<AiNationConfig>,
    pub observer_config: ObserverConfig,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ScenarioRef {
    pub name: String,
    pub version: String,
}

impl std::fmt::Display for ScenarioRef {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}@{}", self.name, self.version)
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct HumanNationConfig {
    pub nation_ids: Vec<NationId>,
    pub turn_mode: TurnMode,
    /// 0 = no timeout.
    pub turn_timeout_ticks: u64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AiNationConfig {
    pub nation_id: NationId,
    /// 1..=5
    pub difficulty: u8,
    pub personality: PersonalitySpec,
    pub rng_offset: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PersonalitySpec {
    Fixed(PersonalityArchetype),
    Random,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ObserverConfig {
    pub omniscient_by_default: bool,
}
```

### 12.2 `SessionType` and `TurnMode`

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SessionType {
    Pve,
    Observer,
    HotSeat,
    AsyncChallenge,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TurnMode {
    Sequential,
    Simultaneous,
}
```

### 12.3 `SessionState`

```rust
use std::sync::Arc;
use tokio::sync::Mutex;
use crate::sim::SimState;

/// Runtime state of an active session.
/// Protected by a Mutex for concurrent RPC handler access.
/// The tick loop holds the lock for the duration of each tick.
#[derive(Debug)] // non-Clone: contains Arc<Mutex<...>>
pub struct SessionState {
    pub session_id: SessionId,
    pub config: SessionConfig,
    pub lifecycle: SessionLifecycle,
    pub current_tick: u64,
    pub blake3_chain_tip: [u8; 32],
    pub hot_seat: Option<HotSeatState>,
    pub sim_state: Arc<Mutex<SimState>>,
    /// One controller per AI nation. Indexed by NationId.
    pub ai_controllers: HashMap<NationId, Box<dyn AiNationController>>,
    /// Per-nation RNG sub-streams. Indexed by NationId.
    pub nation_rngs: HashMap<NationId, ChaCha20Rng>,
    pub observer_subscriptions: Vec<ObserverSubscription>,
    /// Pending speed change (applied at next tick boundary).
    pub pending_tick_speed_ms: Option<u32>,
    /// If Some, session is fast-forwarding to this tick.
    pub fast_forward_target: Option<u64>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SessionLifecycle {
    Created,
    Configuring,
    Running,
    Paused,
    Ended { reason: EndReason, final_tick: u64 },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EndReason {
    VictoryCondition,
    AllEliminated,
    MaxTicks,
    ClientRequest,
    Error,
}
```

### 12.4 `HotSeatState` and `TurnToken`

```rust
/// Active hot-seat session state. None for PvE, Observer, AsyncChallenge sessions.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HotSeatState {
    /// Ordered list of human nations in turn rotation order.
    pub rotation: Vec<NationId>,
    /// Index into rotation of the current token holder.
    pub current_index: usize,
    pub current_token: TurnToken,
    pub mode: TurnMode,
    /// In Simultaneous mode: set of nations that have submitted this tick.
    pub submitted_this_tick: std::collections::HashSet<NationId>,
}

impl HotSeatState {
    pub fn current_nation(&self) -> &NationId {
        &self.rotation[self.current_index]
    }

    pub fn advance_token(&mut self, issued_at_tick: u64, timeout_ticks: u64) -> &TurnToken {
        self.current_index = (self.current_index + 1) % self.rotation.len();
        let nation_id = self.rotation[self.current_index].clone();
        let expires_at_tick = if timeout_ticks > 0 {
            Some(issued_at_tick + timeout_ticks)
        } else {
            None
        };
        self.current_token = TurnToken {
            nation_id,
            issued_at_tick,
            expires_at_tick,
            sequence: self.current_token.sequence + 1,
        };
        self.submitted_this_tick.clear();
        &self.current_token
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TurnToken {
    pub nation_id: NationId,
    pub issued_at_tick: u64,
    pub expires_at_tick: Option<u64>,
    pub sequence: u64,
}
```

### 12.5 `ChallengeBundle` and `ChallengeResult`

```rust
/// User-submitted challenge bundle.
/// Fully self-contained: engine runs this without any other configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChallengeBundle {
    pub challenge_bundle_version: String,
    pub scenario_ref: ScenarioRef,
    pub human_strategy: PolicyBundle,
    pub ai_opponents: Vec<AiNationConfig>,
    pub seed: u64,
    pub max_ticks: u64,
    pub scoring: ScoringConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PolicyBundle {
    pub policy_bundle_version: String,
    pub nation_id: NationId,
    pub economic_policy: EconomicPolicy,
    pub military_posture: MilitaryPosture,
    pub diplomatic_stances: HashMap<NationId, DiplomaticStance>,
    pub tech_priorities: Vec<String>,
    pub city_build_orders: HashMap<CityId, Vec<String>>,
    pub conditional_rules: Vec<ConditionalRule>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EconomicPolicy {
    /// All values in percent (0..=100). Sum must equal 100.
    pub tax_rate_pct: u8,
    pub infrastructure_spend_pct: u8,
    pub military_spend_pct: u8,
    pub research_spend_pct: u8,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MilitaryPosture {
    Defensive,
    Neutral,
    Aggressive,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DiplomaticStance {
    Hostile,
    Neutral,
    Friendly,
    Allied,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConditionalRule {
    /// Simple predicate string. Evaluated by a deterministic rule interpreter.
    /// No Turing-complete scripting; expressions are a fixed grammar.
    pub condition: String,
    /// Action string. Validated at bundle submission time.
    pub action: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScoringConfig {
    pub weights: ScoringWeights,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScoringWeights {
    /// All weights sum to 1.0. Stored as i32 ×1000 (fixed-point).
    pub final_score: i32,
    pub survival_ticks: i32,
    pub territory_at_end: i32,
    pub tech_level_at_end: i32,
    pub treasury_at_end: i32,
}

/// Completed challenge result returned to user and stored in challenge_results.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChallengeResult {
    pub challenge_id: Uuid,
    pub session_id: Uuid,
    pub replay_id: Uuid,
    /// Range: -1000 to +1000 (fixed-point ×1000 of the -1.0 to 1.0 score).
    pub score_millionths: i32,
    pub final_metrics: ChallengeMetrics,
    pub baseline_metrics: ChallengeMetrics,
    pub wall_clock_runtime_ms: u64,
    pub completed_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChallengeMetrics {
    pub final_score: i64,
    pub survival_ticks: u64,
    pub territory_at_end: i32,
    pub tech_level_at_end: i32,
    pub treasury_at_end: i64,
}
```

### 12.6 `ObserverSubscription`

```rust
/// Tracks an observer client's subscription parameters.
/// One per connected observer WebSocket.
#[derive(Debug, Clone)]
pub struct ObserverSubscription {
    /// Unique identifier for this connection.
    pub connection_id: Uuid,
    /// None = omniscient (full state). Some(id) = filtered to that nation's FoW.
    pub fog_nation_id: Option<NationId>,
    /// Send one tick out of every N. 1 = every tick.
    pub tick_stride: u32,
    /// For replay mode: the session is ENDED and client is scrubbing the timeline.
    pub replay_mode: Option<ReplayMode>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReplayMode {
    pub start_tick: u64,
    /// Fixed-point ×1000. 1000 = real time. 5000 = 5× speed.
    pub speed_multiplier_millipct: i32,
    pub current_replay_tick: u64,
}
```

---

## 13. Acceptance Criteria

Acceptance criteria use `FR-SESSION-NNN` identifiers and the SHALL / SHALL NOT convention.

### PvE Session

**FR-SESSION-001:** The engine SHALL support a session type `pve` with exactly one human-controlled nation and one or more AI-controlled nations in the same simulation.

**FR-SESSION-002:** The engine SHALL assign each AI nation a `ChaCha20Rng` sub-stream derived deterministically from the session seed and the nation's `rng_offset`. Given identical (seed, rng_offset) inputs, the derived sub-stream SHALL be bit-for-bit identical across all runs and platforms.

**FR-SESSION-003:** In a PvE session, the human client SHALL have permanent input authority for their nation for the full duration of the session. There SHALL be no turn token in PvE mode.

**FR-SESSION-004:** AI nations in a PvE session SHALL submit actions through the same `NationAction` queue as the human client. AI nations SHALL NOT access simulation state through any privileged mechanism not available to the human client's action model.

**FR-SESSION-005:** The engine SHALL reject any action submitted by an AI controller that is not a valid `NationAction` as defined by the current game rules, returning a validation error to the internal audit log.

### Hot-Seat Session

**FR-SESSION-006:** The engine SHALL support a session type `hot_seat` with two or more human-controlled nations sharing a single WebSocket client connection, with a turn token that rotates between nations.

**FR-SESSION-007:** The engine SHALL reject any `NationAction` submitted by a nation that does not currently hold the turn token with error code `-32001` (NOT_YOUR_TURN).

**FR-SESSION-008:** The engine SHALL advance the turn token to the next nation in rotation when the current holder calls `session.turn.end`, validating that the `token_sequence` in the request matches the engine's current sequence before accepting the handoff.

**FR-SESSION-009:** If `turn_timeout_ticks > 0` and the current token holder has not called `session.turn.end` by `expires_at_tick`, the engine SHALL automatically advance the token and emit `session.turn.auto_passed.v1`.

**FR-SESSION-010:** In simultaneous turn mode, the engine SHALL collect actions from all human nations within a tick and resolve them together in deterministic order (lexicographic by NationId) before advancing the simulation.

### Observer Mode

**FR-SESSION-011:** The engine SHALL support observer client connections that receive tick broadcasts without any ability to inject NationActions.

**FR-SESSION-012:** The engine SHALL reject any `NationAction` or session-mutating RPC submitted by a client registered as an observer with error code `-32002` (OBSERVER_READ_ONLY).

**FR-SESSION-013:** In omniscient observer mode, the engine SHALL transmit the full unfiltered SimState snapshot to the observer client every `tick_stride` ticks.

**FR-SESSION-014:** In fog-of-war observer mode, the engine SHALL apply server-side visibility filtering to produce a state snapshot containing only information visible to the specified `fog_nation_id` before transmission. The observer SHALL never receive hidden state data.

**FR-SESSION-015:** The engine SHALL support a replay observer mode for ENDED sessions, allowing clients to subscribe at any starting tick and request seek operations.

### Async Challenge

**FR-SESSION-016:** The engine SHALL accept a `ChallengeBundle` submission via `POST /api/v1/challenges` and return a `challenge_id` and queue position within 500ms of receiving a valid submission.

**FR-SESSION-017:** The engine SHALL run challenge sessions fully headless: no WebSocket client, no tick broadcast, no pause/resume — at maximum available tick speed.

**FR-SESSION-018:** The engine SHALL compute a baseline score by running an AI-only session with the same scenario, seed, and opponent configuration, replacing the human nation with a Standard (difficulty 2) AI controller. The baseline SHALL be computed once per unique (scenario_ref, seed, opponent_config) tuple and cached.

**FR-SESSION-019:** The challenge score SHALL be computed as a weighted sum of normalized metric deltas against the baseline, using fixed-point arithmetic (i32 ×1000). No floating-point arithmetic SHALL appear in the scoring computation.

**FR-SESSION-020:** On challenge completion, the engine SHALL store the full `.civreplay` file and make it accessible at `GET /api/v1/challenges/{id}/replay`.

### Pause / Resume / Speed

**FR-SESSION-021:** `session.pause` SHALL halt the tick loop after completing the current tick in progress. Partial ticks SHALL NOT be possible; the simulation SHALL always be in a fully-resolved tick state when paused.

**FR-SESSION-022:** `session.resume` SHALL resume the tick loop from the exact state at pause time. The BLAKE3 chain SHALL continue from the hash at the pause tick without any gaps or synthetic ticks.

**FR-SESSION-023:** `session.set_speed` SHALL accept `ticks_per_second` in the range `1..=100` and apply the new rate at the next tick boundary. Values outside this range SHALL be rejected with a JSON-RPC error.

**FR-SESSION-024:** `session.fast_forward` SHALL suppress client tick broadcasts for all intermediate ticks and send a single full-state snapshot when the target tick is reached.

**FR-SESSION-025:** All pause, resume, and speed change operations SHALL emit the corresponding session-layer events (`session.paused.v1`, `session.resumed.v1`, `session.speed_changed.v1`) to all connected clients (human and observer).

### Save / Load

**FR-SESSION-026:** `session.save` SHALL serialize the complete SimState (as defined in CIV-1000) to the named slot, compute a BLAKE3 hash of the blob, and store both in the `session_saves` table.

**FR-SESSION-027:** `session.load` SHALL verify the BLAKE3 hash of the stored blob before restoring state. A hash mismatch SHALL cause load to fail with error code `-32008` (SAVE_INTEGRITY_FAILURE) without applying any state changes.

**FR-SESSION-028:** The autosave mechanism SHALL write to the `"autosave"` slot every `autosave_interval_ticks` ticks, overwriting the previous autosave. User-created slots SHALL never be overwritten by autosave.

**FR-SESSION-029:** Loading a save slot from an ENDED session SHALL create a new session branching from the save point, with a new `session_id`. The original ENDED session SHALL remain immutable.

### Session Lifecycle

**FR-SESSION-030:** The engine SHALL assign each session a UUIDv7 at `session.create` time. The session ID SHALL be included in every event and RPC response associated with that session.

**FR-SESSION-031:** The engine SHALL validate the complete `SessionConfig` at `session.create` time, including scenario resolution, seed validity, and AI nation config validation, before returning a response. Invalid configs SHALL be rejected with error code `-32005`.

**FR-SESSION-032:** Session state SHALL be persisted to the `sessions` table before the engine returns a success response for any lifecycle-changing RPC (create, start, pause, resume, end).

**FR-SESSION-033:** On engine restart, incomplete sessions (status not `ended`) SHALL be reloaded from the database and made available for resumption by the original client.

---

## 14. Integration Points

### 14.1 CIV-0001: Core Simulation Loop

The session layer is a **wrapper** around the core tick loop defined in CIV-0001. The session layer:

- Owns the `ChaCha20Rng` initialization from `session.seed`
- Controls the tick loop start/stop (pause/resume/speed)
- Injects AI controller decisions into the `NationAction` queue at Phase 2 of each tick
- Receives the `BLAKE3` hash computed at Phase 5 of each tick and stores it in `SessionState.blake3_chain_tip`
- Wraps the tick broadcast (Phase 6) to apply observer fog-of-war filtering

The session layer does NOT modify the deterministic logic of CIV-0001 tick phases. AI actions and human actions are indistinguishable to the core simulation; the session layer is responsible for ensuring they are submitted in the correct order.

**Tick loop ownership:** The session's `RunningState` owns the `tokio::task::JoinHandle` for the tick loop task. Pause is implemented by sending a `SessionCommand::Pause` through a `tokio::sync::watch` channel watched by the tick task.

```rust
/// Commands sent from RPC handlers to the tick loop task.
pub enum SessionCommand {
    Pause,
    Resume,
    SetSpeed { tick_speed_ms: u32 },
    FastForward { target_tick: u64 },
    End { reason: EndReason },
}
```

### 14.2 CIV-0200: Client Protocol

The session layer adds the following method namespaces to the CIV-0200 JSON-RPC protocol:

| Namespace | Methods Added |
|---|---|
| `session.*` | create, start, pause, resume, end, observe, set_speed, fast_forward, save, load |
| `session.turn.*` | end (client → engine), start.v1 (engine → client, notification) |
| `session.replay.*` | seek |
| `challenge.*` | (REST endpoints, not WebSocket) |

All methods added by CIV-0900 follow the CIV-0200 conventions: JSON-RPC 2.0, versioned event names (`*.v1`), and the error code scheme defined in CIV-0200 Section X extended with the CivLab-specific codes defined in Section 10 of this spec.

Observer subscriptions are managed by registering the client connection in `SessionState.observer_subscriptions`. The CIV-0200 WebSocket broadcast hub is extended to support per-connection filtering (fog-of-war) and `tick_stride` decimation.

### 14.3 CIV-0400: AI / NPC Behavior

The session layer instantiates AI controllers using the factory pattern defined in CIV-0400:

```rust
/// Called at session CONFIGURING → RUNNING transition.
fn build_ai_controllers(
    config: &SessionConfig,
    sim_state: &SimState,
    session_seed: u64,
) -> HashMap<NationId, Box<dyn AiNationController>> {
    config.ai_nations.iter().map(|ai_cfg| {
        let personality = resolve_personality(
            &ai_cfg.personality, &ai_cfg.nation_id, session_seed
        );
        let difficulty_config = DifficultyConfig::for_level(ai_cfg.difficulty);
        let controller: Box<dyn AiNationController> = match ai_cfg.difficulty {
            1 => Box::new(NoviceController::new(ai_cfg.nation_id.clone(), personality, difficulty_config)),
            2 => Box::new(StandardController::new(ai_cfg.nation_id.clone(), personality, difficulty_config)),
            3 => Box::new(VeteranController::new(ai_cfg.nation_id.clone(), personality, difficulty_config)),
            4 => Box::new(ExpertController::new(ai_cfg.nation_id.clone(), personality, difficulty_config)),
            5 => Box::new(LegendaryController::new(ai_cfg.nation_id.clone(), personality, difficulty_config)),
            _ => unreachable!("difficulty validated at session creation"),
        };
        // Initialize from current sim state (pre-game state).
        // controller.initialize() is called before RUNNING state.
        (ai_cfg.nation_id.clone(), controller)
    }).collect()
}
```

`DifficultyConfig` from CIV-0400 Section 10 governs all AI tuning parameters (MCTS simulation count, utility weight ranges, memory depth, etc.). The session layer passes `AiNationConfig.difficulty` directly to `DifficultyConfig::for_level()`; it does not duplicate difficulty logic.

### 14.4 CIV-1000: Save / Load

CIV-0900 delegates all state serialization to CIV-1000. The session layer provides:

- The `SimState` value to serialize (at the current tick)
- The slot name and session ID for storage
- The BLAKE3 hash verification on load

CIV-1000 defines:
- Binary encoding format (likely `bincode` + zstd compression)
- Versioning of the state schema (forward/backward compatibility)
- Migration of saved states across engine versions

The `session_saves.state_blob` column stores the exact byte sequence produced by CIV-1000's serializer. The `session_saves.blake3_hash` column stores the BLAKE3 hash of that byte sequence, computed by the session layer (not by CIV-1000's serializer, to ensure the hash covers exactly what is stored).

---

*End of CIV-0900: PvE Session and AI Opponent Model v1.0*
