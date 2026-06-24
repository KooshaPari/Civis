# CivLab Status Report

**Report Type:** QA Matrix & Health Atlas
**Generated:** 2026-02-21
**Auditor:** AI Agent (Claude Opus 4.6)

---

## Executive Summary

| Metric | Score | Status |
|--------|-------|--------|
| **Overall Health** | 4/10 | 🟡 EARLY STAGE |
| **Documentation Completeness** | 9/10 | ✅ Excellent |
| **Implementation Progress** | 1/10 | 🔴 Minimal |
| **Architecture Quality** | 8/10 | ✅ Well-designed |
| **Test Coverage** | 0/10 | 🔴 None |
| **FR Coverage (Designed)** | 95% | ✅ Excellent |
| **FR Coverage (Implemented)** | 1% | 🔴 Not Started |

**Verdict:** Project is in early scaffolding phase with excellent documentation and architecture but minimal implementation. Core engine has only a stub WorldState struct.

---

## Project Overview

| Aspect | Value |
|--------|-------|
| **Name** | CivLab |
| **Purpose** | Headless deterministic civilization simulation engine |
| **Stack** | Rust (workspace, 8 crates planned) |
| **Current Crates** | 5 scaffolds: engine, policy, io, server, metrics |
| **Planned Crates** | 8: engine, economy, spatial, climate, actors, policy, geo, social |
| **Target Users** | Game developers, researchers, strategy gamers |

---

## Current Codebase State

### Implemented Crates

| Crate | Files | LOC | Tests | Status |
|-------|-------|-----|-------|--------|
| `civ-engine` | 1 | ~30 | 1 | Scaffold only |
| `civ-policy` | 1 | ~10 | 0 | Empty |
| `civ-io` | 1 | ~10 | 0 | Empty |
| `civ-server` | 1 | ~10 | 0 | Empty |
| `civ-metrics` | 1 | ~10 | 0 | Empty |

### Actual Implementation

```rust
// civ-engine/src/lib.rs (only real code)
pub struct WorldState {
    pub tick: u64,
    pub population: u64,
    pub energy_budget_joules: f64,
}

pub fn step(mut state: WorldState, consumption_joules: f64) -> WorldState {
    state.tick += 1;
    state.energy_budget_joules = (state.energy_budget_joules - consumption_joules).max(0.0);
    state
}
```

**Note:** Using `f64` violates determinism requirements (see FR-CIV-CORE-002).

---

## Functional Requirements Audit

### By Domain

| Domain | FR Count | Designed | Implemented | Tested | Coverage |
|--------|----------|----------|-------------|--------|----------|
| **ECON** (Economy) | 20 | 20 | 0 | 0 | 0% |
| **RTS** (Command Interface) | 15 | 15 | 0 | 0 | 0% |
| **GEO** (Geography/Terrain) | 10 | 10 | 0 | 0 | 0% |
| **ACT** (Actor Lifecycle) | 12 | 12 | 0 | 0 | 0% |
| **WAR** (War/Diplomacy) | 8 | 8 | 0 | 0 | 0% |
| **RES** (Research/Sandbox) | 8 | 8 | 0 | 0 | 0% |
| **TOTAL** | ~73 | 73 | 0 | 0 | **0%** |

### Critical P0 Requirements Status

| FR ID | Description | Priority | Status |
|-------|-------------|----------|--------|
| FR-CIV-ECON-001 | Ledger Double-Entry Accounting | P0 | ❌ Not Started |
| FR-CIV-ECON-002 | Market Clearing Algorithm | P0 | ❌ Not Started |
| FR-CIV-ECON-003 | Joule Economy Allocator | P0 | ❌ Not Started |
| FR-CIV-ECON-004 | Policy-Driven Fiscal Control | P0 | ❌ Not Started |
| FR-CIV-RTS-001 | Unit Movement Command | P0 | ❌ Not Started |
| FR-CIV-RTS-002 | Unit Combat & Attack Orders | P0 | ❌ Not Started |
| FR-CIV-GEO-001 | Terrain Types & Properties | P0 | ❌ Not Started |
| FR-CIV-GEO-004 | Neighbor Queries & Pathfinding | P0 | ❌ Not Started |
| FR-CIV-ACT-001 | Citizen Birth & Initialization | P0 | ❌ Not Started |
| FR-CIV-ACT-005 | Citizen Migration | P0 | ❌ Not Started |

---

## User Journey Readiness

| Journey | Description | Readiness | Blockers |
|---------|-------------|-----------|----------|
| **UJ-1** | Researcher scenario/parameters/replay | 5% | No scenario system, no replay, no CLI |
| **UJ-2** | Game dev integrating into Godot | 2% | No WebSocket server, no protocol |
| **UJ-3** | Player experiencing emergent collapse | 0% | No game loop, no actors, no events |
| **UJ-4** | RTS player military commands | 0% | No units, no combat, no AI |
| **UJ-5** | Policy experimenter A/B testing | 3% | No allocators, no metrics export |

---

## Architecture Assessment

### Strengths ✅

1. **Deterministic Design**: ChaCha20Rng, fixed-point, BTreeMap ordered iteration
2. **ECS Architecture**: Cache-friendly, modular entity model
3. **Multi-Client Protocol**: WebSocket JSON-RPC + binary frames
4. **Event Sourcing**: Full audit trail, replay capability
5. **LOD System**: Two-zoom strategic/tactical aggregation
6. **Clear Separation**: Headless core vs. rendering clients

### Issues 🔴

1. **Float Usage**: Current code uses `f64` - violates determinism
2. **Missing Crates**: 3 planned crates not scaffolded (economy, climate, geo)
3. **No ECS Library**: No hecs/bevy_ecs dependency yet
4. **No Fixed-Point**: No rust_decimal or fixed crate
5. **No RNG Seed**: No ChaCha20Rng implementation
6. **No Event System**: No event logging infrastructure

### Recommendations

| Issue | Fix | Priority |
|-------|-----|----------|
| Float determinism | Use `fixed` or `rust_decimal` crate | P0 |
| ECS missing | Add `bevy_ecs` or `hecs` dependency | P0 |
| RNG missing | Add `rand_chacha` with seed tracking | P0 |
| Event system | Design event envelope + NATS/Redis | P1 |

---

## Phase Progress (PLAN.md)

| Phase | Description | Tasks | Progress | ETA |
|-------|-------------|-------|----------|-----|
| **Phase 0** | Foundation | 5 | 10% | 2 weeks |
| **Phase 1** | Core Engine | 6 | 0% | 4 weeks |
| **Phase 2** | Economy | 8 | 0% | 6 weeks |
| **Phase 3** | Multi-Client | 5 | 0% | 4 weeks |
| **Phase 4** | RTS/War | 6 | 0% | 4 weeks |
| **Phase 5** | Research API | 4 | 0% | 3 weeks |
| **Phase 6** | Polish | 4 | 0% | 2 weeks |

---

## Critical Gaps (Top 10)

| # | Gap | Impact | Recommendation |
|---|-----|--------|----------------|
| 1 | No tick loop | Blocking all work | Implement fixed-timestep loop |
| 2 | Float vs fixed-point | Determinism violation | Replace f64 with fixed-point |
| 3 | No event logging | No audit/replay | Add event envelope + storage |
| 4 | No ECS entities | No actors/buildings | Add bevy_ecs + basic components |
| 5 | No scenario system | No UJ-1 | Implement YAML loader |
| 6 | No WebSocket server | No UJ-2 | Add tokio-tungstenite |
| 7 | No market allocator | No economy | Implement clearing algorithm |
| 8 | No hex grid | No geography | Add hexx or custom grid |
| 9 | No test framework | Quality risk | Add property tests + proptest |
| 10 | No CI/test gates | Regression risk | Add GitHub Actions |

---

## Recommended Next Steps

### Immediate (Week 1-2)

1. **Fix determinism**: Replace `f64` with `FixedI64\<Scale18\>` from `fixed` crate
2. **Add ECS**: Integrate `bevy_ecs` with basic components (Cell, Building, Citizen)
3. **Add tick loop**: Fixed-timestep 100ms with ChaCha20Rng seed
4. **Add event system**: Define EventEnvelopeV1 and emit to stdout/file
5. **Add property tests**: Use `proptest` for conservation invariants

### Short-term (Week 3-6)

1. **Implement ledger**: Double-entry transfers with sum-zero invariant
2. **Implement market clearing**: Basic bid/ask matching
3. **Add hex grid**: Terrain types and pathfinding
4. **Add WebSocket server**: JSON-RPC handshake + snapshot broadcast
5. **Create first scenario**: `temperate-city.yaml`

### Medium-term (Week 7-12)

1. **Implement joule economy**: Energy allocator with retirement
2. **Add citizen lifecycle**: Birth, aging, death, migration
3. **Add RTS commands**: Move, attack, formation
4. **Create Bevy demo**: 2D visualization
5. **Achieve UJ-1**: Researcher can run scenario, export replay

---

## Metrics Summary

| Category | Current | Target (MVP) | Gap |
|----------|---------|--------------|-----|
| Crates implemented | 5/8 | 8/8 | 3 |
| Test coverage | 0% | 80% | 80% |
| FR implemented | 0% | 40% | 40% |
| User journeys | 0/5 | 1/5 | 1 |
| Tick performance | N/A | <16ms | N/A |
| Memory footprint | N/A | <2GB | N/A |

---

## Conclusion

CivLab has **excellent documentation and architecture** but is in the **very early implementation phase**. The core engine has only a 30-line scaffold with a critical determinism bug (f64 usage).

**Key Recommendation:** Focus on Phase 0 foundation tasks with strict test-first approach. Fix determinism issues immediately before any further implementation.

**Estimated Time to MVP:** 12-16 weeks with focused effort on core engine + economy.

---

## Next Focus

- Fill project-specific architecture and API content.
- Keep trackers synchronized with implementation progress.
