# ADR-001: Rust Crate Structure — Workspace with Focused Modules

**Date:** 2026-02-21

**Status:** ACCEPTED

**Author:** CIV Architecture Team

---

## Context

The CIV city simulation engine requires modular architecture supporting multiple distinct simulation domains:
- Economy (ledger, market clearing)
- Spatial representation (LOD, districts, actors)
- Climate (energy accounting)
- Institutions & actors (lifecycle, state machines)
- Geopolitics (conflict, diplomacy)
- Social dynamics (ideology, health, insurgency)

A monolithic crate would couple these domains unnecessarily. Separate crates enable:
- Independent testing and iteration
- Clear boundary enforcement (via `tach.toml`)
- Easier integration with Venture platform (each crate has well-defined API)
- Parallel development across teams

## Decision

Organize CIV as a **Rust workspace** with the following focused crates:

```
civ/
  Cargo.toml (workspace root)

  crates/
    engine/          # Core simulation loop (CIV-0001)
                     # - Tick-based state machine
                     # - Event ordering logic
                     # - Determinism/replay contract

    economy/         # Economy module (CIV-0100)
                     # - Ledger, market clearing, transfers
                     # - Depends on: engine

    spatial/         # Spatial & LOD (CIV-0101)
                     # - District/agent representation
                     # - LOD transitions
                     # - Neighbor queries
                     # - Depends on: engine

    climate/         # Climate module (CIV-0102)
                     # - Energy accounting, conservation equation
                     # - Weather events
                     # - Depends on: engine, economy (energy supply)

    actors/          # Institutions & citizen lifecycle (CIV-0103)
                     # - Actor state machines
                     # - Institution formation/dissolution
                     # - Time-series metrics
                     # - Depends on: engine, economy, spatial

    policy/          # Policy evaluation framework (CIV-0100, CIV-0104)
                     # - Policy.evaluate() interface
                     # - Constraint solver (minimal constraint set theorem)
                     # - Determinism validator
                     # - Depends on: engine, economy, actors

    geo/             # Geopolitics (CIV-0105)
                     # - War, diplomacy, shadow networks
                     # - Conflict resolution
                     # - Depends on: engine, actors, spatial

    social/          # Social dynamics (CIV-0106)
                     # - Ideology, health, insurgency
                     # - Cohesion metrics
                     # - Depends on: engine, actors, spatial

    metrics/         # Observability (Venture integration)
                     # - Event recording
                     # - Time-series storage
                     # - Audit trail hooks
                     # - Depends on: engine, economy, actors, climate

    server/          # HTTP/gRPC server (Venture integration)
                     # - EventEnvelopeV1 emission
                     # - Policy.evaluate tool endpoint
                     # - Artifact export pipeline
                     # - Depends on: (all modules above)
```

## Dependency Graph (Enforced via `tach.toml`)

```
server  ─────┬─ metrics
             ├─ economy  ─────┬─ engine
             ├─ climate  ─────┤─ economy
             ├─ actors   ─────┼─ spatial ─── engine
             ├─ geo      ─────┤─ policy
             ├─ social   ──────┴─ engine
             └─ policy

            (All non-root crates depend on engine)
```

## Rationale

1. **Clear Separation of Concerns**
   - Each crate has a single, well-defined responsibility
   - Easier to test and reason about in isolation
   - Matches spec organization (CIV-0100, CIV-0101, etc.)

2. **Boundary Enforcement**
   - Use `tach.toml` to enforce dependency graph
   - Prevents accidental coupling (e.g., economy should not depend on geo)
   - CI rejects violations

3. **Parallel Development**
   - Teams can work on crates independently
   - Reduced merge conflicts on shared files
   - Clear handoff points (crate APIs)

4. **Venture Integration**
   - `server/` crate provides single integration point
   - EventEnvelopeV1, tool endpoints, artifact export all in one place
   - Easier to version and test Venture contracts

5. **Performance & Build Time**
   - Only build changed crates in incremental builds
   - Team members can work on one domain without pulling all code

## Consequences

### Positive
- Reduced cyclomatic complexity per crate
- Clearer ownership and accountability
- Easier to onboard new team members to specific domains
- Boundary violations caught at compile time

### Negative
- More complex workspace management (multiple Cargo.tomls)
- Cross-crate API changes require coordination
- Workspace build is slower than monolithic initially (improves with incremental builds)
- More CI/CD configuration

## Alternatives Considered

### A1: Monolithic Crate
**Pros:** Simpler build, shared internal state
**Cons:** Tight coupling, hard to parallelize, cyclomatic complexity grows unbounded
**Rejected:** Does not scale with 8 spec modules

### A2: Separate Repos
**Pros:** Maximum autonomy
**Cons:** Version coordination nightmare, harder to ensure determinism across modules
**Rejected:** Cross-module determinism requires synchronized ticks

### A3: Flat Workspace (No Sub-Modules)
**Pros:** Simple structure
**Cons:** No boundary enforcement, all crates at same level
**Rejected:** Violates dependency ordering (e.g., climate should not depend on geo)

## Implementation

### Crate Structure Template

Each crate should follow:
```
crates/{name}/
  Cargo.toml
  src/
    lib.rs         # Public API
    module1.rs     # Implementation
    ...
  tests/
    integration_tests.rs
  README.md        # Crate-specific docs
```

### Workspace Configuration

**Root Cargo.toml:**
```toml
[workspace]
members = [
  "crates/engine",
  "crates/economy",
  "crates/spatial",
  # ... etc
]
```

### Boundary Enforcement

**tach.toml** (via [Tach](https://www.notion.so/Tach-Python-42da5cfc6bda4a098e39b88e2a34b86f)):
```toml
[boundaries]
"engine" = []  # No dependencies
"economy" = ["engine"]
"spatial" = ["engine"]
"climate" = ["engine", "economy"]
# ... full graph
```

Run: `tach check` (pre-commit hook)

## Validation Commands

```bash
# Build all crates
cargo build --workspace

# Test all crates
cargo test --workspace

# Check dependency boundaries
tach check

# Lint
cargo clippy --workspace --all-targets -- -D warnings

# Format
cargo fmt --all
```

## References

- **CIV-0001:** Core Simulation Loop spec
- **CIV-0100–0106:** Domain-specific specs
- **CLAUDE.md:** Project governance (cross-module determinism)
- **Tach:** Boundary enforcement tool (https://github.com/gauge-sh/tach)

---

**Decision Delta:**
- Workspace-based organization with 9 focused crates
- Dependency graph enforced via `tach.toml`
- Single `server/` crate for Venture integration

**Review Date:** 2026-05-21 (after P0 implementation)
