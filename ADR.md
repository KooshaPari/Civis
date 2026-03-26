# CivLab — Architecture Decision Records

**Project**: CivLab (Headless Civilization Simulation Engine)
**Last Updated**: 2026-03-25
**Owner**: Engineering Team

---

## ADR-001: Rust + ECS (Hecs) for Deterministic Simulation Core

**Date**: 2026-03-25
**Status**: Accepted
**Context**: CivLab requires deterministic simulation: identical input → identical output across platforms and runs. Floating-point arithmetic is non-deterministic (IEEE 754 is platform-dependent). Rust's type system prevents data races. Alternative languages (Go, Python, C++) either lack strong determinism guarantees or require manual synchronization.

**Decision**: Implement simulation core in Rust using the Hecs ECS (Entity Component System) library. All arithmetic uses fixed-point i64 (not floating-point) with a scale factor of 10^6. RNG is seeded once per run (ChaCha8Rng); all RNG calls are logged and reproducible. ECS ensures tight memory layout and cache-friendly iteration over entity components.

**Consequences**:
- Determinism guaranteed: bit-identical output on any platform (including replay from event log)
- Performance: fixed-point arithmetic is slower than float but acceptable (<16ms tick at target scale)
- Learning curve: Rust's borrow checker and ECS paradigm require discipline
- Memory efficiency: ECS layout enables SIMD and vectorization; data-oriented design
- No runtime GC: predictable latency (critical for <16ms tick budget)

**Alternatives Considered**:
- C++ with deterministic libraries: Good performance but no borrow checker; manual memory safety
- Go with careful floating-point control: Language not designed for SIMD; no ECS ecosystem
- Python with NumPy: Slow for simulation loops; GIL contention at scale

---

## ADR-002: Fixed-Point Arithmetic (i64 @ 10^6 Scale) for Determinism

**Date**: 2026-03-25
**Status**: Accepted
**Context**: Floating-point arithmetic (f64) produces platform-dependent results due to register allocation, compiler optimizations, and CPU differences. For a simulation where "replay from event log produces identical snapshot," floating-point is unacceptable. Fixed-point arithmetic is deterministic and portable.

**Decision**: Define a `Fixed` type: newtype i64 with scale 10^6. Multiply inputs by 10^6 on entry; divide outputs by 10^6 on exit. All intermediate calculations use i64 (scaled). Multiplication and division are careful to avoid overflow (use i128 intermediate).

**Consequences**:
- Determinism preserved: all simulation calculations are reproducible
- Precision: 10^6 scale gives 6 decimal places; sufficient for economy (prices, production rates)
- No floating-point rounding errors: no accumulation of small errors over 1M ticks
- Overhead: fixed-point multiplication is 10-20% slower than float; acceptable given determinism value
- Boundary: overflow impossible for reasonable values (i64 max ~9e18; 10^6 scale allows values up to ~9e12)

**Alternatives Considered**:
- Decimal type (BigDecimal): Arbitrary precision but slower; unnecessary overhead
- Floating-point with strict rounding (IEEE mode): Still not truly deterministic across platforms
- Fractional libraries (num-rational): Exact but much slower than fixed-point

---

## ADR-003: WebSocket JSON-RPC + Binary Frames Protocol for Multi-Client Communication

**Date**: 2026-03-25
**Status**: Accepted
**Context**: CivLab must support multiple simultaneous clients (Bevy, Unreal, Unity, Web, research dashboards) connected to the same deterministic simulation. Clients render at different rates (60 FPS for game clients, slower for dashboards) and request different data (game client wants full snapshot, research wants specific metrics). HTTP polling is inefficient; gRPC is not suitable for web browsers.

**Decision**: Dual-protocol approach:
1. **Handshake & Commands**: JSON-RPC 2.0 over WebSocket for initial connection, client registration, and command submission (player actions, policy changes). Request/response model ensures reliability.
2. **Snapshots & Events**: Binary frames (custom format with zstd compression) for high-frequency state broadcasts. Clients specify snapshot type (FULL, DELTA, FILTERED_BY_REGION). Frames sent after each tick; clients filter by subscription.

**Consequences**:
- Flexibility: JSON-RPC for reliability, binary for performance
- Bandwidth efficient: zstd compression reduces snapshot size by 60-80%
- Latency: <50ms round-trip for commands; <16ms for snapshot delivery (one tick latency)
- Complexity: must implement custom binary format and zstd integration
- Client implementation: each game engine must parse binary frames; standard protocol (not gRPC/Protobuf)

**Alternatives Considered**:
- gRPC streams: Excellent but not supported in browsers (requires grpc-web proxy); overkill for this use case
- GraphQL subscriptions: Over-engineered for snapshot updates
- Raw TCP binary: No framing; error-prone
- Protobuf + gRPC: Standard but adds overhead; harder to debug

---

## ADR-004: Event Log Replay for Full Determinism Verification

**Date**: 2026-03-25
**Status**: Accepted
**Context**: Claiming "determinism" means little without verification. The specification requires: "Replay any v1 run from event log; verify bit-identical." This mandates event-sourced design where every state mutation is recorded and replayable.

**Decision**: Implement append-only event log (events.jsonl format). Every tick writes events that caused state changes. At startup, load initial state snapshot + event log; replay all events → final state must match stored snapshot. CI includes mandatory "replay determinism test" that runs 100 ticks, serializes state, replays from log, compares hashes.

**Consequences**:
- Full auditability: can inspect why any entity state changed
- Replay capability: load scenario → load events → exact same state every time
- Storage overhead: event log grows ~100-500 bytes per event; manageable
- Performance testing enabled: can profile replay vs. live execution
- Risk: a single event log corruption (e.g., truncation) breaks entire run (mitigated by checksums per event)

**Alternatives Considered**:
- Snapshot-based replay: Only snapshots; cannot debug individual decisions
- State versioning (Git-style): More complex; overkill for this use case
- No verification: Contradicts determinism requirement

---

## ADR-005: Policy → Production → Allocation → Trade as Core Economic Loop

**Date**: 2026-03-25
**Status**: Accepted
**Context**: CivLab targets "economy as deep as Victoria 3." The economy pipeline must be clear, auditable, and fast (<16ms per tick). Each phase transforms state; clear separation enables testing and modification.

**Decision**: Strict phase ordering per tick:
1. **Policy Evaluation** (5ms budget): Process policies (tax rates, building quotas, subsidies) → population desires/supplies updated
2. **Production** (4ms budget): Buildings produce goods based on available workers + resources.
3. **Allocation** (3ms budget): Distribute produced goods to population/storage. Respects population priority and storage constraints.
4. **Trade** (2ms budget): Markets clear via bid/ask matching. Prices adjust based on supply/demand.
5. **Stochastic Events** (2ms budget): Apply seeded RNG (rebellion, plague, harvest variance). Record events to event log.

Total tick time: ~16ms. Phases are sequential; no concurrency (ensures determinism).

**Consequences**:
- Clear mental model: developers understand flow; easy to modify
- Auditability: can trace which phase caused state change
- Performance: budgets enforce latency discipline; encourage optimization
- Constraints: phases cannot feed back to earlier phases (breaks causality)
- Testing: each phase is independently testable

**Alternatives Considered**:
- Simultaneous resolution: Race conditions unless carefully serialized; harder to debug
- Event-driven (trigger production on demand): Difficult to enforce global consistency; budgets unpredictable

---

## ADR-006: Joule as Universal Energy Numeraire (Thermodynamic Model)

**Date**: 2026-03-25
**Status**: Accepted
**Context**: Victoria 3-style economies are complex (100+ goods, dynamic prices). CivLab wants simplicity without sacrificing depth. Using a single universal numeraire (energy/joules) enables simplified accounting, property testing, and intuitive pricing.

**Decision**: All goods produce and consume energy (measured in joules). Grain production requires labor (human energy) + land (solar potential). Building a settlement requires stone (structural energy) + wood (chemical energy). Markets operate on joule exchange: farmer trades 10,000 joules of grain for 5,000 joules of tools.

Energy conservation property:
```
total_produced_energy - total_consumed_energy - total_lost_to_inefficiency = inventory_change
```

Property test: for every tick, verify energy conservation within ±1% (rounding tolerance).

**Consequences**:
- Elegant model: thermodynamic principles apply to economy
- Easier balancing: designer can reason about energy flows instead of price curves
- Educational value: players/researchers understand economics as energy allocation
- Complexity: requires defining joule equivalents for 50+ goods (design burden)
- Risk: if conservation test fails, entire tick is invalid; must replay from last known good state

**Alternatives Considered**:
- Labor-based accounting (Adam Smith): Complex; 100+ goods each with labor coefficients
- Supply/demand curves (Victoria 3): Expensive to balance; many parameters
- Commodity basket: No unifying principle; arbitrary weighting

---

## ADR-007: Multi-Crate Workspace (engine, server) for Separation of Concerns

**Date**: 2026-03-25
**Status**: Accepted
**Context**: CivLab has distinct concerns: pure simulation (deterministic, testable, no I/O) vs. serving (WebSocket, networking, command handling). Mixing them in one crate couples testing to networking infrastructure. Cargo workspaces enable clean separation.

**Decision**: Use Cargo workspace with two crates:
- **civ-engine**: Core simulation (ECS, tick loop, event log). Pure Rust; zero external I/O. No async, no WebSocket—just deterministic state machines.
- **civ-server**: HTTP/WebSocket server, command parsing, snapshot serialization. Handles async, networking, client lifecycle.

civ-server depends on civ-engine; civ-engine has no dependencies on civ-server. Test civ-engine without server running.

**Consequences**:
- Clean separation: simulation logic is isolated and testable
- Reusable core: other projects can embed civ-engine (Python bindings, C FFI)
- Deployment flexibility: engine can run headless; server connects as client
- Build complexity: slight overhead from two crates
- Monorepo: shared Cargo.toml workspace; versions in sync

**Alternatives Considered**:
- Single crate: Simpler for small projects but couples concerns
- Separate repositories: Harder to keep in sync; duplicated infrastructure config

---

## ADR-008: Scenario Format as YAML + Python API for Scripting

**Date**: 2026-03-25
**Status**: Accepted
**Context**: Researchers want to define scenarios (starting conditions, policies, initial buildings) and run parameter sweeps. Operators want to distribute scenarios as replayable bundles. YAML is human-readable but limited. Python scripting enables complexity without custom languages.

**Decision**: Two-layer scenario system:
1. **YAML definition** (static): Initial state, map layout, policy constants
2. **Python API** (dynamic): Load YAML → instantiate scenario object → call methods to override parameters, run simulation, access results

Enables parameter sweeps: test 100 variations, compare metrics. Researchers can use Jupyter notebooks, matplotlib for analysis.

**Consequences**:
- Accessible to non-programmers (YAML) and power users (Python)
- Parameter sweeps enabled: test 100 variations, compare metrics
- Reproducibility: scenario + random seed = deterministic run
- Tooling: researchers can use Jupyter notebooks, matplotlib for analysis
- Complexity: must maintain Python bindings (civ-engine → Python)

**Alternatives Considered**:
- YAML-only: Limited expressiveness; no parameter sweeps
- Embedded scripting language (Lua, wasm): Overkill; Python ecosystem more familiar
- JSON: Less readable than YAML; same expressiveness

---

## ADR-009: Zstd Compression for Snapshot Serialization

**Date**: 2026-03-25
**Status**: Accepted
**Context**: Snapshots are large (~10-100 MB for 100K agents). Transmitting uncompressed over WebSocket to 10 clients @ 60 FPS consumes bandwidth. Standard compression (gzip) is slower. Zstd offers 60% compression ratio at near-zero CPU cost (microseconds).

**Decision**: Serialize snapshots to JSON, compress with zstd (compression level 3, balance speed/ratio). Binary frames include compressed data length; clients decompress on receipt. Snapshots cached in-memory; recompressed only on state change.

**Consequences**:
- Bandwidth reduced by 60-80%: 10 clients @ 60 FPS → manageable network load
- CPU cost: <1ms per snapshot (zstd level 3 is tuned for speed)
- Client implementation: must include zstd decompression (available in all major languages)
- Storage: archived scenarios compressed for distribution (10 MB → 4 MB per run)

**Alternatives Considered**:
- gzip: Slower compression (5-10ms per snapshot); same ratio
- Custom binary encoding (no JSON): Faster but less debuggable; JSON is human-readable
- Uncompressed: Unacceptable bandwidth at scale (100 Mbps for 10 clients)

---

## ADR-010: Cargo Test + Property Testing (proptest) for Determinism Verification

**Date**: 2026-03-25
**Status**: Accepted
**Context**: Determinism is the core promise. Without testing, bugs go undetected until production. Standard unit tests (example-based) miss edge cases. Property-based testing (generate random inputs, verify invariants) catches subtle bugs.

**Decision**: Implement test suite:
1. **Unit tests**: Test individual systems (production, allocation, trade) against known inputs
2. **Property tests** (proptest): Generate random initial states and event logs; verify energy conservation, no negative resources, population boundaries
3. **Determinism regression tests**: Run 100-tick scenario, replay from event log, assert final state == stored state
4. **CI requirement**: All tests pass; determinism regression tests mandatory on every commit

**Consequences**:
- High confidence: property tests catch rare bugs (e.g., integer overflow in edge cases)
- Test suite time: ~10-30s per run (acceptable for CI)
- Learning curve: property-based testing is different mindset; requires discipline
- Coverage: proptest can generate 1000s of examples; example-based tests cover maybe 5
- Debugging: failed property tests require minimization (shrink to simplest failing input)

**Alternatives Considered**:
- Example-based tests only: Miss edge cases; false confidence
- Fuzzing (libFuzzer): Excellent but needs integration with Rust build system
- Manual testing: Doesn't scale; human error

---

**Document History**:
- v1.0 (2026-03-25): Initial ADR set. 10 decisions covering determinism, language choice, protocol design, economy, and testing.

*Cross-ref: [PRD.md](./PRD.md) | [FUNCTIONAL_REQUIREMENTS.md](./FUNCTIONAL_REQUIREMENTS.md)*
