# CivLab Architecture Decision Records (ADR Master Index)

**Version:** 1.0
**Date:** 2026-02-21
**Status:** APPROVED
**Scope:** All architectural decisions for headless Rust civilization simulation engine

---

## Overview

This document is the master index for all ADRs (Architecture Decision Records) for CivLab. Each ADR documents a significant architectural decision with context, rationale, alternatives, and consequences. ADRs are permanent records of architectural choices and must be updated only when re-evaluating a decision.

**Organization:** ADRs grouped by domain (engine, economy, rendering, integration, etc.). Full ADR details are in linked documents in `docs/adr/`.

---

## ADR Index

### ADR-001: Rust Workspace Crate Structure

**File:** `docs/adr/ADR-001-rust-crate-structure.md`
**Date:** 2026-02-21
**Status:** ACCEPTED
**Review Date:** 2026-05-21

**Summary:**
CivLab is organized as a Rust workspace with 9 focused crates:
- `engine`: Core simulation loop, tick advancement, event ordering
- `economy`: Ledger, market clearing, allocators (market, plan, joule)
- `spatial`: District/region representation, LOD transitions, neighbor queries
- `climate`: Energy accounting, conservation, weather events
- `actors`: Institutions, citizen lifecycle, state machines
- `policy`: Policy evaluation framework, constraint solver
- `geo`: Geopolitics (war, diplomacy, shadow networks)
- `social`: Ideology, health, insurgency, cohesion metrics
- `metrics`: Observability, event recording, time-series storage
- `server`: HTTP/gRPC server, EventEnvelopeV1 emission, artifact export

**Key Points:**
- Dependency graph enforced via `tach.toml`
- Clear separation of concerns
- Enables parallel development across teams
- Single `server/` integration point with Venture platform
- Workspace build supports incremental compilation

**Rationale:**
- Monolithic crate would couple domains unnecessarily and grow unbounded complexity
- Separate repos would lose determinism (cross-module tick synchronization)
- Workspace-based approach balances autonomy with synchronization

**Alternatives Considered:**
- A1: Monolithic crate (rejected: tight coupling, cyclomatic complexity)
- A2: Separate repos (rejected: version coordination nightmare)
- A3: Flat workspace (rejected: no boundary enforcement)

**Impact:**
- **Positive:** Reduced cyclomatic complexity, clear ownership, easier onboarding, boundary violations caught at compile-time
- **Negative:** Complex workspace management, cross-crate API changes need coordination, slower initial workspace build

**Validation:**
```bash
cargo build --workspace
cargo test --workspace
tach check  # Verify boundary compliance
```

**Related FRs:** All FRs (each mapped to specific crate)

---

### ADR-002: Joule Economy as Pluggable Resource Allocator

**File:** `docs/adr/ADR-002-joule-economy-as-allocator.md`
**Date:** 2026-02-21
**Status:** ACCEPTED
**Review Date:** 2026-05-21 (after P0 + Joule implementation in P1)

**Summary:**
CivLab's economy module implements a pluggable `Allocator` trait with three implementations:
1. **MarketAllocator:** Supply/demand price discovery, competitive bidding, exchange
2. **PlanAllocator:** Central planner model, pre-determined allocations, quotas
3. **JouleAllocator:** Joule-economy model, agent-centric work capacity allocation

**Key Points:**
- All allocators must satisfy conservation invariant: `supply_in + reserves_in - losses - consumption - reserves_out = delta_stock`
- Allocators selected at runtime via spec bundle (P2 feature)
- MarketAllocator implemented in P0; Plan and Joule in P1
- Venture spend-quota (TRACK_B) is orthogonal to CivLab allocation strategy

**Rationale:**
- **CIV Design Freedom:** Allows experimentation with different allocation strategies; A/B testing within same seed
- **Venture Integration:** Venture controls budget; CivLab controls allocation strategy (clear separation)
- **Conservation & Determinism:** All allocators enforced through validation interface
- **Code Reuse:** Shared infrastructure (ledger, price indexing, quota tracking) across allocators

**Alternatives Considered:**
- A1: Single market allocator (rejected: reduces expressiveness, no experimentation)
- A2: Joule only (rejected: loses market baseline for comparison)
- A3: Allocator config in spec bundle (deferred to P2: adds versioning complexity)

**Impact:**
- **Positive:** CivLab economy can evolve without breaking Venture; experimental allocators don't risk production; clear allocation strategy choices
- **Negative:** More complex codebase (trait + multiple impls); allocator changes must preserve conservation; trait dispatch overhead; 3x test burden (all allocators must pass tests)

**Validation:**
```bash
cargo test --package economy allocator::tests
cargo bench --package economy
# Run determinism tests with each allocator
cargo test --package economy determinism -- --nocapture
```

**Related FRs:**
- FR-CIV-ECON-002: Market Clearing Algorithm
- FR-CIV-ECON-003: Joule Economy Allocator Implementation
- FR-CIV-ECON-009: Hybrid Allocator (Market + Joule)

---

### ADR-003: Deterministic Replay & Bit-Reproducibility

**File:** `docs/adr/ADR-003-deterministic-replay.md`
**Date:** 2026-02-21
**Status:** ACCEPTED
**Review Date:** 2026-06-21 (after first full integration test)

**Summary:**
CivLab guarantees deterministic replay: given identical (scenario_spec, random_seed, policy_bundle), the full simulation trajectory (tick-by-tick state) is byte-for-byte identical across runs. This enables:
1. Reproducible research (publish seed + config, others verify results)
2. Replay debugging (save state at tick N, replay forward with variations)
3. Causality tracing (same seed = same exogenous events → enables causal analysis)

**Key Points:**
- **RNG Seeding:** Every random value (birth, death, migration, market clearing, climate events) is drawn from deterministic PRNG seeded by simulation seed
- **Fixed-Point Arithmetic:** All financial/economic calculations use fixed-point (u128 with 18 decimals) NOT floating-point
- **Order Stability:** Transfer lists, market orders, institutional transitions sorted by stable key (e.g., actor_id) not arbitrary iteration order
- **Policy Determinism:** Policy evaluation with same policy_bundle + state → identical outputs (required by ADR-002)
- **Event Ordering:** Events emitted in stable order per tick (enables replay with identical event sequence)

**Rationale:**
- **Research Credibility:** Reproducibility is fundamental to scientific computing; without it, results are suspect
- **Debugging:** Replay enables stepping through simulation, comparing divergence points
- **Testing:** Determinism tests catch subtle non-determinism bugs (floating-point, unordered iteration)

**Alternatives Considered:**
- A1: Non-deterministic (cheaper, faster, not reproducible) — rejected: ruins research value
- A2: Approximate reproducibility (within ±1%) — rejected: insufficient for causal analysis
- A3: Replay via event stream only (no state snapshots) — deferred to P2: trades disk space for computation

**Impact:**
- **Positive:** Reproducible research, easier debugging, strong testing foundation
- **Negative:** Fixed-point arithmetic slower than float (but accurate), stricter testing burden, potential performance penalty from ordering constraints

**Validation:**
```bash
# Property test: same seed produces identical state
cargo test --package engine determinism_test

# Replay test: export metrics run A, replay run A with same seed, verify metrics match
civ-sim run replay --run-id <id> --verify-metrics

# Floating-point detection: cargo check for f32/f64 in hot paths (should use fixed-point)
```

**Related FRs:**
- FR-CIV-RES-007: Replay File & Serialization
- FR-CIV-RES-010: Reproducibility Package & Citation

---

### ADR-004: Headless-First, Client-Agnostic Design

**File:** `docs/adr/ADR-004-headless-rendering.md`
**Status:** ACCEPTED (in principle; full ADR document to be written in P0)
**Review Date:** 2026-04-21

**Summary:**
CivLab core is headless (no rendering). The simulation engine emits:
1. **State snapshots:** tick-by-tick LOD data (strategic view, tactical view, sim view for research)
2. **Events:** material state changes (policy applied, market cleared, citizen migrated, conflict, etc.)
3. **Metrics:** aggregate time-series (population, welfare, Gini, legitimacy, military, etc.)

Client renderers (Bevy, Unreal, Unity, Godot, Web) consume these outputs and render as needed. Why NOT embed rendering in core:

**Key Points:**
- **Separation of Concerns:** Simulation logic (determinism, conservation, causal) is orthogonal to rendering (performance, UX, platform)
- **Platform Agnostic:** Core works with ANY renderer; no dependency on Bevy/Unreal/etc.
- **Test & Debug:** Headless core can be tested in pure Rust without rendering overhead; CI runs quickly
- **Research:** Researchers analyze simulation without graphics cost; can run 10x faster headless
- **Iteration:** Renderer can change without touching core; API versioning at rendering interface (LOD snapshots, events)

**Rendering Boundary:**
- **Server emits:** State snapshots (JSON/binary), events (JSONL), metrics (JSONL), replay files (binary)
- **Client receives:** Over WebSocket (JSON-RPC), HTTP (REST/gRPC), or local IPC (FFI)
- **Client renders:** Discretized into zoom levels (strategic/tactical/sim) with local prediction (smoothness, latency masking)

**Rationale:**
- **Long-term Maintainability:** Core logic isolated from graphics API churn (Bevy releases, Unreal versions, etc.)
- **Open Ecosystem:** Third-party renderers can integrate without modifying core
- **Performance & Cost:** Researchers can run thousands of simulations on CPU-only; gaming clients render as needed

**Alternatives Considered:**
- A1: Embed Bevy renderer in core (rejected: couples core to graphics framework)
- A2: Custom in-engine renderer (rejected: wasted effort, not competitive with Unreal/Unity)
- A3: Server-side rendering + client streaming (rejected: bandwidth, latency worse than client-side)

**Impact:**
- **Positive:** Core remains pure simulation, test-friendly, platform-agnostic; open ecosystem for renderers
- **Negative:** Client-server protocol must be carefully versioned; client prediction required for real-time responsiveness; renderer integration work shifts to client developers

**Validation:**
- [ ] Server can be compiled and run headless (no graphics dependencies)
- [ ] Godot client successfully integrates core (P0 proof-of-concept)
- [ ] Web renderer loads and visualizes state snapshots (P1 stretch goal)

**Related FRs:**
- FR-CIV-RES-001: Scenario Configuration & Loading (headless scenario API)
- FR-CIV-RES-002: Run Execution & Tick Advancement (headless execution)

---

### ADR-005: WebSocket JSON-RPC Client Protocol (Chosen over gRPC)

**File:** `docs/adr/ADR-005-client-protocol-websocket-jsonrpc.md`
**Status:** ACCEPTED (draft, full ADR to be written in P0)
**Review Date:** 2026-04-21

**Summary:**
Client-server communication uses WebSocket + JSON-RPC (not gRPC, not HTTP REST). Rationale:

**Key Points:**
- **WebSocket:** Bidirectional, persistent connection (server can push state updates without client polling); lower latency than HTTP request-response
- **JSON-RPC:** Human-readable, lightweight protocol; `{"jsonrpc": "2.0", "method": "execute_command", "params": {...}, "id": 1}`
- **Real-time Responsiveness:** Command latency < 100 ms (p95) achievable over WiFi with local prediction
- **Debugging:** JSON is text-based, easy to inspect logs; gRPC binary is opaque

**Protocol Design:**
1. **Handshake:** Client connects, server sends `{ready, scenario_metadata, lod_version, schema_version}`
2. **State updates:** Server broadcasts `state_changed` notifications (100 ms interval or on major event)
3. **Commands:** Client sends `execute_command` JSON-RPC; server responds with ACK + result
4. **Events:** Server emits `event.v1` notifications (async, no ACK needed)

**Rationale:**
- **gRPC Rejected:** Requires protobuf definitions, more complex tooling, harder for research integrations (Python researchers use JSON natively)
- **REST HTTP Rejected:** Request-response latency too high for real-time (each command = 50-100 ms round-trip); would need polling for state updates (inefficient)
- **WebSocket Chosen:** Enables push notifications (server sends updates asynchronously); full-duplex (simultaneous send/receive)

**Alternatives Considered:**
- A1: gRPC (rejected: protobuf friction, binary opaque, overkill for JSON data)
- A2: HTTP REST (rejected: request-response latency, inefficient polling for updates)
- A3: Raw TCP with binary protocol (rejected: harder to debug, no standard tools)

**Impact:**
- **Positive:** Low latency, human-readable, standard tooling (curl, postman, client libraries in all languages)
- **Negative:** JSON serialization slightly slower than binary; large state updates (100 KB+) may need compression

**Validation:**
- [ ] Client latency < 100 ms (p95) on real network conditions
- [ ] Server can handle 100+ concurrent clients without blocking
- [ ] Godot client successfully executes commands over WebSocket

**Related FRs:**
- FR-CIV-RTS-001 through FR-CIV-RTS-015: RTS commands over WebSocket JSON-RPC

---

### ADR-006: ECS vs Custom Entity Model vs bevy_ecs

**File:** `docs/adr/ADR-006-entity-model-selection.md`
**Status:** TENTATIVE (decision pending P0 implementation sprint)
**Review Date:** 2026-03-21

**Summary:**
Entity-Component-System (ECS) pattern is strongly considered for managing 10k+ citizens, districts, military units, institutions. Options:

1. **bevy_ecs:** Mature ECS from Bevy engine (can use without graphics)
2. **hecs:** Minimal ECS library (smaller dependency, lower overhead)
3. **Custom:** Hand-rolled entity model (full control, more complexity)

**Decision Pending:** Will be finalized after P0 kickoff based on performance profiling of prototype.

**Key Considerations:**
- **Population Scale:** 10k+ citizens per city means efficient iteration, caching, and indexing
- **Systems Architecture:** Update citizens → update jobs → update economy → resolve transfers (pipeline-friendly for ECS)
- **Serialization:** ECS state must be serializable for replay (determinism requirement)
- **Testing:** Custom model is easiest to test; ECS requires careful ordering

**Rationale (Preliminary):**
- **bevy_ecs:** Mature, proven, can be decoupled from Bevy engine; good for parallel systems
- **hecs:** Lighter weight, may be faster for CivLab's specific access patterns
- **Custom:** Most control, but higher maintenance burden

**Validation:** (Deferred to P0)
```bash
# Create three prototype branches
git branch feature/ecs-bevy
git branch feature/ecs-hecs
git branch feature/entity-custom

# Benchmark: 10k citizen update performance on each
# Measure: CPU time per tick, memory footprint, serialization overhead
```

**Related FRs:**
- FR-CIV-ACT-001 through FR-CIV-ACT-015: Citizen lifecycle (high-volume entity updates)

---

### ADR-007: Fixed-Point Arithmetic for Economic Calculations

**File:** `docs/adr/ADR-007-fixed-point-arithmetic.md`
**Status:** ACCEPTED
**Review Date:** 2026-06-21 (after P0 economic module implementation)

**Summary:**
All economic calculations (prices, ledger amounts, wealth, etc.) use fixed-point arithmetic, NOT floating-point. Specifically:
- **Type:** `i128` or `u128` with scaling factor (18 decimals)
- **Example:** 1.5 units represented as 1_500_000_000_000_000_000i128 (1.5 × 10^18)
- **Operations:** Add/sub are native; mul/div require rescaling; no precision loss

**Rationale:**
- **Determinism:** Floating-point arithmetic is non-deterministic across platforms (x86 vs ARM vs WASM)
- **Exactness:** Financial calculations require exact arithmetic; 0.1 + 0.2 = 0.3 exactly (not 0.30000000000000004 in float)
- **Audit Trail:** Fixed-point ensures ledger balances are exact; no rounding errors accumulate

**Example:**
```rust
// Fixed-point: exact
let price: i128 = 1_500_000_000_000_000_000; // 1.5 units
let amount: i128 = 2_000_000_000_000_000_000; // 2.0 units
let total = price * amount / 1_000_000_000_000_000_000; // = 3.0 units (exact)

// Floating-point: not exact
let price_f64 = 1.5_f64;
let amount_f64 = 2.0_f64;
let total_f64 = price_f64 * amount_f64; // = 3.0 (happens to be exact, but not guaranteed)
```

**Alternatives Considered:**
- A1: Floating-point f64 (rejected: non-deterministic, precision loss)
- A2: Decimal library (decimal128) (rejected: overkill, slower than fixed-point)
- A3: Rational arithmetic (rejected: complexity, slower)

**Impact:**
- **Positive:** Determinism guaranteed, exact ledger balances, reproducibility
- **Negative:** Fixed-point slower than float; requires careful rescaling in mul/div; more verbose code

**Validation:**
```bash
# Property test: fixed-point determinism
cargo test --package economy fixed_point_determinism

# Benchmark: fixed-point vs float performance
cargo bench --package economy fixed_point
```

**Related FRs:**
- FR-CIV-ECON-001: Ledger Double-Entry Accounting
- FR-CIV-ECON-005: Inflation & Price Index Tracking

---

### ADR-008: Event-Sourcing for Audit Trail & Compliance

**File:** `docs/adr/ADR-008-event-sourcing.md`
**Status:** ACCEPTED
**Review Date:** 2026-06-21 (after metrics module implementation)

**Summary:**
All material state changes are captured as immutable events in append-only log. Events enable:
1. **Audit Trail:** Complete record of why state changed (traced to policy/actor decision)
2. **Replay:** Rebuild state from event stream (redundant with snapshots, but useful for analysis)
3. **Causality:** Events have correlation_id for linking causal chains
4. **Compliance:** Immutable log satisfies regulatory audit requirements

**Key Points:**
- **Event Types:** 50+ domain-specific types (policy.applied.v1, economy.market_cleared.v1, citizen.migrated.v1, etc.)
- **Schema Versioning:** Events are versioned (.v1, .v2) to support format changes
- **Correlation:** Each event has correlation_id linking to originating decision (e.g., drought event → supply_shock → migration)
- **Immutability:** Event log is write-once; never modified (prevents audit trail tampering)

**Rationale:**
- **Transparency:** All state changes are traceable to specific events/decisions; no hidden state mutations
- **Debugging:** Researcher can reconstruct exact sequence of events leading to outcome
- **Compliance:** Audit trail satisfies governance and reproducibility requirements

**Alternatives Considered:**
- A1: Implicit state (no events, just snapshots) (rejected: loses causality, harder to debug)
- A2: Events + full state rebuild on replay (rejected: redundant, slower than snapshots)
- A3: Centralized event bus (rejected: adds complexity, requires careful ordering)

**Impact:**
- **Positive:** Complete audit trail, easier debugging, compliance-friendly
- **Negative:** Disk I/O for event log (~30 MB per 10k-tick run); requires schema versioning discipline

**Validation:**
```bash
# Verify no silent state changes
cargo test --package metrics event_completeness

# Correlation chain test
cargo test --package metrics correlation_tracing
```

**Related FRs:**
- FR-CIV-RES-004: Event Log & Audit Trail

---

### ADR-009: LOD (Level of Detail) Zoom Architecture

**File:** `docs/adr/ADR-009-lod-zoom-architecture.md`
**Status:** ACCEPTED
**Review Date:** 2026-05-21 (after spatial module implementation)

**Summary:**
Simulation supports three zoom levels with different data granularity:
1. **Zoom Level 1 (Strategic):** Region-level aggregates (total population, GDP, military strength); for strategic RTS view and diplomacy
2. **Zoom Level 2 (Tactical):** District-level details (individual structures, unit positions, resources); for city management and RTS
3. **Zoom Level 3 (Simulation):** Citizen-level details (job, welfare, ideology, stress); for research and deep analysis only

**Key Points:**
- **LOD Mapping:** Region aggregates are deterministic functions of district data at same tick
- **Data Contracts:** Each zoom level has explicit data schema (JSONL/Parquet); versions tracked separately
- **Drill-Down:** Client can click to zoom in; server sends more detailed data for zoomed region
- **Bandwidth Optimization:** Client receives only zoom-appropriate data (strategic view is ~0.5 KB per region; tactical ~2 KB per district)
- **Determinism:** Aggregation is pure function (no randomness); identical districts → identical regional aggregate

**Rationale:**
- **Scalability:** Rendering 10k citizens is expensive; aggregates reduce data transfer and client rendering burden
- **Game UX:** Strategic zoom enables high-level decision making (diplomacy, trade); tactical zoom enables detailed city management
- **Research:** Zoom 3 enables citizen-level analysis; researchers can drill down to understand aggregate behavior

**Alternatives Considered:**
- A1: Single unified data model (rejected: too much data at strategic level, too little at tactical)
- A2: Client-side aggregation (rejected: requires client to have full simulation state; defeats bandwidth optimization)
- A3: Fixed-resolution snapshots (rejected: no drill-down capability)

**Impact:**
- **Positive:** Bandwidth-efficient, enables drill-down exploration, supports strategic decision-making
- **Negative:** Requires careful schema versioning, aggregation correctness must be tested

**Validation:**
```bash
# Macro-micro consistency test
cargo test --package spatial lod_aggregation_accuracy

# Schema version test
cargo test --package metrics lod_schema_versioning
```

**Related FRs:**
- FR-CIV-GEO-003: District & Region Subdivision
- FR-CIV-GEO-010: LOD Rendering Contract & Data Schema

---

### ADR-010: Three-Tier Policy Evaluation (Baseline → Constrained → Optimized)

**File:** `docs/adr/ADR-010-policy-evaluation-tiers.md`
**Status:** DRAFT (to be finalized in P1)
**Review Date:** 2026-05-21

**Summary:**
Policy evaluation follows three-tier process:
1. **Baseline:** Admin specifies raw policy (tax_rate, subsidy amounts, etc.) without constraint checking
2. **Constrained:** System checks if policy violates hard invariants (conservation, bounds); rejects or corrects
3. **Optimized:** (P2) System suggests policy improvements (e.g., "increase food subsidy by 5 units to prevent migration spike")

**Rationale:**
- **Flexibility:** Admins can set arbitrary policies; system ensures they're valid (not impossible)
- **Safety:** Conservation invariant enforced at policy boundary (prevents impossible transfers)
- **Transparency:** Rejected policies logged with reasons; admin knows why change was blocked

**Related FRs:**
- FR-CIV-ECON-004: Policy-Driven Fiscal Control
- ADR-002 (via allocator validation)

---

## ADR Cross-References

| Domain | Key ADRs | Related FRs |
|--------|----------|------------|
| **Architecture** | ADR-001 (Workspace) | All FRs |
| **Economy** | ADR-002 (Allocators), ADR-007 (Fixed-Point) | FR-CIV-ECON-* |
| **Determinism** | ADR-003 (Replay), ADR-007 (Fixed-Point), ADR-008 (Events) | FR-CIV-RES-007, 010 |
| **Rendering** | ADR-004 (Headless), ADR-005 (WebSocket) | FR-CIV-RTS-*, FR-CIV-GEO-010 |
| **Entity Model** | ADR-006 (ECS Selection) | FR-CIV-ACT-*, FR-CIV-GEO-* |
| **Data** | ADR-008 (Events), ADR-009 (LOD) | FR-CIV-RES-* |

---

## Decision Impact Summary

| Decision | Impact | Timeline |
|----------|--------|----------|
| ADR-001: Workspace crates | Enables parallel development, increases complexity | P0 (weeks 1-4) |
| ADR-002: Pluggable allocators | Enables experimentation, 3x test burden | P0 (MarketAllocator), P1 (Plan, Joule) |
| ADR-003: Deterministic replay | Reproducibility guaranteed, stricter testing | P0 (design), ongoing validation |
| ADR-004: Headless design | Open ecosystem, client-server protocol needed | P0 (core), P0 (Godot proof-of-concept) |
| ADR-005: WebSocket JSON-RPC | Low-latency, human-readable, less efficient than binary | P0 (server), P0 (Godot client) |
| ADR-006: ECS selection | TBD; impacts entity update performance | P0 (prototyping) |
| ADR-007: Fixed-point arithmetic | Deterministic but slower, exact ledger balances | P0 (econ module) |
| ADR-008: Event-sourcing | Complete audit trail, ~30 MB event log per run | P0 (metrics module) |
| ADR-009: LOD architecture | Bandwidth-efficient, enables drill-down | P0 (spatial module), P1 (drill-down UI) |
| ADR-010: Three-tier policy | Flexible yet safe, adds validation layer | P1 (optimization tier) |

---

## ADR Maintenance & Evolution

**Updates:** ADRs should be updated only when re-evaluating a decision. Minor clarifications do not require new versions.

**Deprecation:** If a decision is reversed, mark old ADR as SUPERSEDED and create new ADR with rationale for change.

**Review Cycle:** All ADRs reviewed annually (or on request after major related project completion).

**Governance:** ADR changes require approval from architecture committee (2+ domain leads). Rationale and consequences must be documented.

