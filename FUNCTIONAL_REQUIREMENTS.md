# Functional Requirements — CivLab

**Version:** 1.0
**Status:** Draft
**Date:** 2026-03-25
**Traces to:** PRD.md v1.0 (CivLab epics E1–E6)

---

## Categories

| Code | Domain |
|------|--------|
| CORE | Core simulation engine and tick loop |
| ECON | Economy, markets, and Joule accounting |
| PROTO | Multi-client protocol (WebSocket, binary frames) |
| REPLAY | Deterministic replay and audit trail |
| API | Research API and scenario system |
| CLIENT | Client implementations and integration |

---

## FR-CORE-001: Fixed-Timestep Tick Loop

**Priority**: SHALL
**Description**: The simulation engine SHALL execute a fixed-timestep tick loop at 100 ms per tick with no jitter under nominal load.
**Acceptance Criteria**:
- [ ] 10,000 ticks execute in under 2 minutes headless on commodity hardware
- [ ] Tick interval jitter measured < 1 ms (P99) under no-client load
- [ ] Tick counter monotonically increases; no tick is skipped or duplicated
**Traces to**: E1.1
**Status**: Partial

---

## FR-CORE-002: ECS Entity Model

**Priority**: SHALL
**Description**: The engine SHALL use an ECS (Entity Component System) architecture with defined entity types: cells, buildings, agents, and institutions.
**Acceptance Criteria**:
- [ ] All four entity types creatable and addressable by stable entity ID
- [ ] Component queries complete in O(n) over entity count with cache-friendly memory layout
- [ ] Entity IDs survive serialization/deserialization round-trips unchanged
**Traces to**: E1.2
**Status**: Partial

---

## FR-CORE-003: Deterministic Transition Phase

**Priority**: SHALL
**Description**: The deterministic transition phase SHALL produce bit-identical output for identical inputs, using fixed-point arithmetic with no floating-point state mutations.
**Acceptance Criteria**:
- [ ] Replay of any recorded run from event log produces identical state hashes at every tick
- [ ] No f32/f64 types used in state-mutating paths; fixed-point types enforced by type system
- [ ] Determinism test suite runs on every CI commit and blocks merge on failure
**Traces to**: E1.4
**Status**: Partial

---

## FR-CORE-004: Stochastic Event Phase with Seeded RNG

**Priority**: SHALL
**Description**: The stochastic event phase SHALL use ChaCha20Rng seeded per run, with all RNG draws logged in the event stream to guarantee reproducibility.
**Acceptance Criteria**:
- [ ] RNG seed stored in run header and recoverable from the .civreplay file
- [ ] Every RNG draw emits a `rng_draw` event with seed state and result
- [ ] Property test: two runs from same seed produce identical stochastic event sequences
**Traces to**: E1.5
**Status**: Planned

---

## FR-CORE-005: Policy Evaluation Phase

**Priority**: SHALL
**Description**: The engine SHALL execute a policy evaluation phase each tick that transforms current state into control signals governing production, allocation, and taxation.
**Acceptance Criteria**:
- [ ] Policy evaluation runs before production and allocation in tick ordering
- [ ] Policy results are pure functions of current state (no side effects)
- [ ] Policy override via Scenario API injects custom policy functions without engine modification
**Traces to**: E1.3
**Status**: Planned

---

## FR-CORE-006: Multi-Client Command Queue

**Priority**: SHALL
**Description**: The engine SHALL maintain a priority-ordered command queue that serializes commands from multiple clients before each tick boundary.
**Acceptance Criteria**:
- [ ] Admin commands have priority over player commands; player priority over research
- [ ] FIFO ordering enforced within the same priority tier
- [ ] Commands submitted after tick cutoff are deferred to the next tick
**Traces to**: E1.7
**Status**: Planned

---

## FR-CORE-007: Tick Budget Enforcement

**Priority**: SHALL
**Description**: The engine SHALL target a tick processing budget of 14 ms on commodity hardware and emit a warning log entry when any tick exceeds 16 ms.
**Acceptance Criteria**:
- [ ] P99 tick duration < 14 ms on reference hardware (4-core, 16 GB RAM)
- [ ] Ticks exceeding 16 ms logged with profiling breakdown by phase
- [ ] Performance CI gate blocks merge when P99 regresses beyond 16 ms baseline
**Traces to**: E1.10
**Status**: Planned

---

## FR-ECON-001: Production System

**Priority**: SHALL
**Description**: Buildings SHALL produce goods each tick according to their production rate and input resource availability, writing outputs to per-entity inventory.
**Acceptance Criteria**:
- [ ] Production halts when required inputs are unavailable (no phantom goods)
- [ ] Production rates configurable per building type in scenario YAML
- [ ] Production events emitted to event log with entity ID, good type, and quantity
**Traces to**: E2.1
**Status**: Partial

---

## FR-ECON-002: Joule Energy Conservation

**Priority**: SHALL
**Description**: The Joule energy unit SHALL serve as the universal economic numeraire with the total quantity conserved across all entities each tick.
**Acceptance Criteria**:
- [ ] Property test: sum of all Joules in simulation is invariant across ticks (conservation law)
- [ ] Joule balance tracked per entity and in a global ledger
- [ ] Any tick that violates conservation produces a fatal simulation error with ledger diff
**Traces to**: E2.4
**Status**: Planned

---

## FR-ECON-003: Market Clearing

**Priority**: SHALL
**Description**: The market system SHALL clear bid/ask orders each tick using price discovery to reach supply/demand equilibrium for each traded good.
**Acceptance Criteria**:
- [ ] Grain market clears in MVP; multi-good markets operational in v1
- [ ] Clearing algorithm is deterministic given identical order books
- [ ] Uncleared orders expire after configurable TTL (default: 1 tick)
**Traces to**: E2.3
**Status**: Partial

---

## FR-ECON-004: Taxation and Budget System

**Priority**: SHALL
**Description**: The fiscal system SHALL implement configurable tax policies that transfer Joules from entities to an institutional treasury each tick and impact legitimacy.
**Acceptance Criteria**:
- [ ] Tax rate configurable per institution and good type
- [ ] Tax revenue credited to institution treasury in same tick as collection
- [ ] Legitimacy model decrements citizen satisfaction as a function of effective tax rate
**Traces to**: E2.6, E2.7, E2.8
**Status**: Planned

---

## FR-ECON-005: Allocation Algorithm

**Priority**: SHALL
**Description**: The allocation system SHALL distribute available goods to consumer entities each tick according to priority and need, running after market clearing.
**Acceptance Criteria**:
- [ ] Allocation priority: subsistence goods before luxury goods
- [ ] Unmet allocation needs increment deprivation counters on affected entities
- [ ] Allocation phase completes in O(n log n) over entity count
**Traces to**: E2.5
**Status**: Planned

---

## FR-PROTO-001: RFC 6455 WebSocket Server

**Priority**: SHALL
**Description**: The simulation server SHALL expose a WebSocket server compliant with RFC 6455 on a configurable port accepting at least 10 simultaneous client connections.
**Acceptance Criteria**:
- [ ] Server accepts >= 10 simultaneous WebSocket connections without degradation
- [ ] TLS support via configurable certificate paths required for non-localhost deployments
- [ ] Graceful close handshake sent to all clients on server shutdown
**Traces to**: E3.1
**Status**: Partial

---

## FR-PROTO-002: JSON-RPC 2.0 Message Dispatcher

**Priority**: SHALL
**Description**: The server SHALL dispatch all client messages using the JSON-RPC 2.0 protocol, returning structured results and errors for every request.
**Acceptance Criteria**:
- [ ] All RPC methods return `result` or `error` as specified by JSON-RPC 2.0
- [ ] Unknown method names return error code -32601
- [ ] Batch requests supported per JSON-RPC 2.0 specification
**Traces to**: E3.2
**Status**: Planned

---

## FR-PROTO-003: Client Handshake and Bootstrap

**Priority**: SHALL
**Description**: Connecting clients SHALL complete a handshake establishing identity, role, and initial world snapshot within 2 seconds on local network before receiving tick deltas.
**Acceptance Criteria**:
- [ ] Handshake completes within 2 seconds of connection on local network
- [ ] Bootstrap snapshot includes all entity states at the current tick
- [ ] Client role (admin/player/research) assigned during handshake and enforced on all subsequent commands
**Traces to**: E3.3
**Status**: Planned

---

## FR-PROTO-004: Binary Frame Protocol

**Priority**: SHALL
**Description**: High-frequency game clients SHALL receive tick deltas as zstd-compressed binary frames with a defined frame header.
**Acceptance Criteria**:
- [ ] Binary frame header includes: tick number, frame type, uncompressed size, checksum
- [ ] zstd compression ratio >= 3:1 on typical delta frames
- [ ] 10 game clients at 60 FPS consume <= 10 Mbps aggregate bandwidth
**Traces to**: E3.6
**Status**: Planned

---

## FR-PROTO-005: Snapshot Filtering by Region and Type

**Priority**: SHALL
**Description**: Clients SHALL be able to subscribe to filtered snapshot streams specifying entity type and/or geographic region bounds to reduce per-client bandwidth.
**Acceptance Criteria**:
- [ ] Filter spec transmitted during handshake and updatable via subscription command
- [ ] Server excludes filtered-out entities from delta frames
- [ ] Region filter uses bounding-box spatial query; out-of-bounds entities fully excluded
**Traces to**: E3.8
**Status**: Planned

---

## FR-REPLAY-001: Civreplay Export Format

**Priority**: SHALL
**Description**: The engine SHALL export complete simulation runs to a .civreplay file containing a header, full event log, and SHA-256 checksum for integrity verification.
**Acceptance Criteria**:
- [ ] Header includes: seed, scenario name, engine version, start timestamp
- [ ] Event log is append-only during a run; no events deleted or modified post-write
- [ ] SHA-256 checksum of event log included in footer and verified on load
**Traces to**: E1.8
**Status**: Partial

---

## FR-REPLAY-002: Bit-Identical Determinism Verification

**Priority**: SHALL
**Description**: Loading and replaying a .civreplay file SHALL produce bit-identical state at every tick compared to the original run, verified by state hash comparison.
**Acceptance Criteria**:
- [ ] Replay mode re-executes all events from log without re-sampling RNG
- [ ] State hash compared at each tick; first divergence reported with tick number and state diff
- [ ] Determinism CI gate runs replay verification on every commit and blocks on failure
**Traces to**: E1.9
**Status**: Planned

---

## FR-API-001: Scenario YAML Format and Validation

**Priority**: SHALL
**Description**: Scenarios SHALL be defined in a versioned YAML format specifying map dimensions, initial entity placement, starting conditions, and policy parameters.
**Acceptance Criteria**:
- [ ] YAML schema documented and versioned in the repo
- [ ] Schema validation runs at scenario load; invalid YAML produces descriptive error with field path
- [ ] Example scenario `data/scenarios/starting_settlement.yaml` included and validated in CI
**Traces to**: E5.1
**Status**: Partial

---

## FR-API-002: Python Scenario Runner

**Priority**: SHALL
**Description**: A Python API SHALL allow researchers to load a scenario YAML, run a headless simulation, and inspect state programmatically via a pip-installable package.
**Acceptance Criteria**:
- [ ] `civlab.run_scenario(path, ticks=50)` completes a 50-tick run in under 5 seconds
- [ ] Package installable via `pip install civlab` with all dependencies declared
- [ ] All public API methods have type annotations and docstrings
**Traces to**: E5.2
**Status**: Planned

---

## FR-API-003: Policy Parameter Override

**Priority**: SHALL
**Description**: The research API SHALL support overriding scenario policy parameters (tax rates, production multipliers, allocation weights) without modifying the scenario YAML file.
**Acceptance Criteria**:
- [ ] Override dict passed to `run_scenario` merges with scenario defaults
- [ ] Invalid parameter names raise `ValueError` listing allowed parameters
- [ ] Override values validated against parameter type constraints before simulation start
**Traces to**: E5.3
**Status**: Planned

---

## FR-API-004: Data Export for Analysis

**Priority**: SHALL
**Description**: The research API SHALL export simulation metrics (agent counts, market prices, ledger balances, events) to CSV and JSON formats compatible with Jupyter/matplotlib.
**Acceptance Criteria**:
- [ ] `civlab.export(run_id, format="csv")` writes per-tick metric table
- [ ] JSON export includes full event log with tick timestamps
- [ ] Export completes in under 30 seconds for a 100,000-tick run
**Traces to**: E5.6
**Status**: Planned

---

## FR-CLIENT-001: Bevy Reference Client

**Priority**: SHALL
**Description**: A Bevy-based reference client SHALL connect to the simulation server, render agents and buildings, and support click-to-build interactions at 60 FPS.
**Acceptance Criteria**:
- [ ] Client renders 60 FPS on reference hardware with 1,000 visible entities
- [ ] Click-to-build sends a `build_command` via JSON-RPC and reflects the result within one tick
- [ ] Client reconnects automatically after server restart within 5 seconds
**Traces to**: E6.1
**Status**: Partial

---

## FR-CLIENT-002: Web TypeScript Client

**Priority**: SHALL
**Description**: A TypeScript/React web client SHALL connect to the server via WebSocket and render a strategic map view of simulation state in a browser.
**Acceptance Criteria**:
- [ ] Client renders in Chrome, Firefox, and Safari (latest stable versions)
- [ ] WebSocket transport handles reconnect with exponential backoff
- [ ] React hooks `useSimulationState()` and `useEntityQuery()` documented and covered by tests
**Traces to**: E6.4
**Status**: Planned

---

## FR-CLIENT-003: Client Role Authorization Enforcement

**Priority**: SHALL
**Description**: The server SHALL enforce client role permissions (admin > player > research) and reject out-of-scope commands with a structured error response.
**Acceptance Criteria**:
- [ ] Research clients cannot submit build or policy commands
- [ ] Unauthorized command attempts return JSON-RPC error code -32603 with role information
- [ ] Role enforcement verified by integration tests covering all three role tiers
**Traces to**: E3.7
**Status**: Planned

---

## FR-METRICS: Simulation Metrics

### FR-METRICS-001: Metrics Struct
**Priority**: SHALL
**Description**: The `Metrics` struct SHALL define four f64 fields: `waste_joules` (10% of consumption), `surplus_joules` (energy budget minus consumption, floored at 0), `tyranny_index` (consumption / (budget+1), capped at 1.0), `legitimacy_index` (1.0 - tyranny_index, floored at 0).
**Acceptance Criteria**:
- [ ] `compute(1000.0, 500.0)` returns `waste_joules=50.0`, `surplus_joules=500.0`
- [ ] When `consumption >= budget`, `tyranny_index > 0.9` and `legitimacy_index < 0.1`
- [ ] All fields are `f64`; struct is `Debug`, `Clone`, `Copy`, `Default`
**Traces to:** E5 (Research Sandbox / Policy Analysis)
**Code:** `crates/engine/src/metrics.rs`

### FR-METRICS-002: Metrics Computation
**Priority**: SHALL
**Description**: The `compute(energy_budget_joules: f64, consumption_joules: f64) -> Metrics` function SHALL compute all four metrics in constant time O(1) using only arithmetic operations with no I/O or allocations.
**Acceptance Criteria**:
- [ ] Function has no side effects; identical inputs always produce identical outputs
- [ ] Function is callable from the ECS tick loop without performance regression (P99 < 1 µs)
**Traces to:** FR-CORE-003 (Deterministic Transition Phase)
**Code:** `crates/engine/src/metrics.rs`

### FR-METRICS-003: Fixed-Point Determinism for Metrics
**Priority**: SHALL
**Description**: Tyranny and legitimacy indices SHALL be computed using the fixed-point `Fixed` type (i64 scaled by 10^6) when deterministic cross-platform reproduction is required; float variants are provided for research export only.
**Acceptance Criteria**:
- [ ] Fixed-point and float results agree to within 6 decimal places for identical inputs
- [ ] Replay verification uses fixed-point metrics exclusively
**Traces to:** FR-REPLAY-002 (Bit-Identical Determinism Verification)
**Code:** `crates/engine/src/lib.rs` — `Fixed` type
