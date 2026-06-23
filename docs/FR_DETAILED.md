# CIV Functional Requirements (Detailed)

**Project:** CIV - Deterministic Civilization Simulation Engine  
**Version:** 2.0  
**Status:** Draft  
**Last Updated:** 2026-02-23

---

## FR Numbering Convention

```
FR-{DOMAIN}-{SEQUENCE}
FR-CORE-XXX   = Core simulation mechanics
FR-ECON-XXX   = Economy and resources  
FR-METRICS-XXX = Metrics and governance
FR-AI-XXX     = Artificial intelligence
FR-NET-XXX    = Networking and multiplayer
FR-PERF-XXX   = Performance requirements
FR-API-XXX    = Public API surface
FR-TEST-XXX   = Testing requirements
FR-DOC-XXX    = Documentation
```

---

## CORE SIMULATION (FR-CORE)

### FR-CORE-001: Deterministic Tick Engine
**Priority:** P0 (Critical)  
**Status:** ✅ Implemented

The simulation engine MUST execute deterministic tick-based updates using fixed-point arithmetic to ensure reproducibility across platforms.

**Requirements:**
- Tick rate: 60 Hz (16.67ms per tick)
- All arithmetic: i64 with 10^6 scale (Fixed type)
- Same seed MUST produce identical simulation results
- Tick budget: <16ms including all phases

**Verification:**
```rust
#[test]
fn test_determinism() {
    let mut sim1 = Simulation::with_seed(12345);
    let mut sim2 = Simulation::with_seed(12345);
    for _ in 0..1000 { sim1.tick(); sim2.tick(); }
    assert_eq!(sim1.state, sim2.state);
}
```

---

### FR-CORE-002: Entity Component System
**Priority:** P0 (Critical)  
**Status:** ✅ Implemented

The engine MUST use an ECS architecture for efficient entity management and querying.

**Requirements:**
- Support 50,000+ simultaneous entities
- Component-based data model (not OOP)
- Query system for entity filtering
- Support component lifecycle hooks

**Components Implemented:**
- Position (x, y)
- Citizen (age, health, ideology, welfare, job)
- Building (type, hp, max_hp, position)
- Resources (food, wood, metal, energy)
- Production (output_type, rate)
- MilitaryUnit (type, strength, morale, position, faction_id)

---

### FR-CORE-003: World State Management
**Priority:** P0 (Critical)  
**Status:** ✅ Implemented

The engine MUST maintain a global world state accessible to all systems.

**WorldState Fields:**
| Field | Type | Description |
|-------|------|-------------|
| tick | u64 | Current tick number |
| population | u64 | Total citizen count |
| energy_budget_joules | Fixed | Available energy |
| rng_seed | u64 | For reproducibility |
| factions | HashMap | Faction ID → name |
| faction_treasury | HashMap | Faction → balance |

---

### FR-CORE-004: Simulation Snapshots
**Priority:** P1 (High)  
**Status:** ⬜ Pending

The engine MUST support saving and restoring full simulation state.

**Requirements:**
- Serialize entire WorldState to JSON
- Serialize all ECS entities
- Support compression for large states
- Restore from snapshot and continue

**Acceptance Criteria:**
- [ ] `simulation.snapshot() → JSON` works
- [ ] `simulation.restore(json)` works  
- [ ] Restored simulation produces same results
- [ ] Snapshot size \< 1MB for 10K entities

---

### FR-CORE-005: Tick Phases
**Priority:** P0 (Critical)  
**Status:** ✅ Implemented

Each tick MUST execute in defined phases in strict order.

**Phase Order:**
1. Pre-Update (input processing)
2. Update
   - Physics
   - Economy
   - AI
   - Governance
3. Post-Update (validation, sync)

**Implementation:**
```rust
pub fn tick(&mut self) {
    self.state.tick += 1;
    self.phase_production();
    self.phase_citizen_lifecycle();
    self.phase_military();
    self.phase_economy();
}
```

---

## ECONOMY SYSTEM (FR-ECON)

### FR-ECON-001: Joule-Based Energy
**Priority:** P0 (Critical)  
**Status:** ✅ Implemented

The economy MUST use joules as the primary energy currency.

**Requirements:**
- Energy production from buildings
- Energy consumption by citizens and buildings
- Energy storage (battery-like)
- Scarcity multiplier affects all consumption

**Formula:**
```
effective_consumption = base_consumption * max(scarcity_multiplier, 0)
```

---

### FR-ECON-002: Resource Management
**Priority:** P1 (High)  
**Status:** ⬜ Pending

The system MUST track food, wood, and metal resources.

**Resource Types:**
| Resource | Producer | Consumer | Storage |
|----------|----------|----------|---------|
| Food | Farm | Citizen | Granary |
| Wood | Forest | Building | Lumberyard |
| Metal | Mine | Military | Warehouse |

---

### FR-CORE-003: Citizen Job System
**Priority:** P1 (High)  
**Status:** ⬜ Pending

Citizens MUST be assignable to jobs that affect production.

**Jobs:**
| Job | Production | Consumption | Special |
|-----|-----------|-------------|---------|
| Farmer | +10 food | -1 energy | |
| Warrior | 0 | -2 energy | Combat |
| Scholar | 0 | -1 energy | Research |
| Trader | 0 | -1 energy | Trade |
| Priest | 0 | -1 energy | +Happiness |
| Admin | 0 | -2 energy | Governance |
| Unemployed | -1 food | -0.5 energy | Unrest |

---

## METRICS & GOVERNANCE (FR-METRICS)

### FR-METRICS-001: Tyranny Index
**Priority:** P0 (Critical)  
**Status:** ✅ Implemented

The system MUST calculate a tyranny index based on resource consumption vs budget.

**Formula:**
```
tyranny_index = min(consumption_joules / (energy_budget_joules + 1), 1.0)
legitimacy_index = 1.0 - tyranny_index
```

**Thresholds:**
- tyranny \< 0.3: Stable
- 0.3 &lt; tyranny \< 0.7: Unrest risk
- tyranny &gt; 0.7: Rebellion risk

---

### FR-METRICS-002: Faction Metrics
**Priority:** P1 (High)  
**Status:** ⬜ Pending

Each faction MUST have tracked metrics.

**Tracked Per Faction:**
- Population
- Treasury balance
- Military strength
- Territory size
- Happiness average

---

## AI & BEHAVIOR (FR-AI)

### FR-AI-001: Citizen Behavior
**Priority:** P1 (High)  
**Status:** ⬜ Pending

Citizens MUST make decisions based on needs and ideology.

**Decision Priority:**
1. Survival (hunger > 80%)
2. Safety (nearby threat)
3. Job satisfaction
4. Ideological alignment
5. Social interaction

---

### FR-AI-002: Faction AI
**Priority:** P1 (High)  
**Status:** ⬜ Pending

Factions MUST have goal-planning AI.

**Goal Types:**
- Expand (claim territory)
- Trade (establish routes)
- War (attack enemy)
- Research (develop tech)
- Build (construct buildings)

---

## NETWORKING (FR-NET)

### FR-NET-001: Turn Protocol
**Priority:** P2 (Medium)  
**Status:** ⬜ Pending

Multiplayer MUST use a turn-synchronization protocol.

**Protocol:**
1. Server broadcasts current state
2. Client submits actions
3. Server validates and executes
4. Server broadcasts result

---

### FR-NET-002: State Sync
**Priority:** P2 (Medium)  
**Status:** ⬜ Pending

The system MUST support real-time state synchronization.

**Sync Strategy:**
- Full sync on connect
- Delta sync per tick
- Event-based for rare changes

---

## PERFORMANCE (FR-PERF)

### FR-PERF-001: Tick Budget
**Priority:** P0 (Critical)  
**Status:** ✅ Implemented

Each tick MUST complete within 16ms budget.

**Current Implementation:**
- Target: 10ms typical
- Budget: 16ms maximum
- Verified by benchmarks

---

### FR-PERF-002: Entity Limits
**Priority:** P1 (High)  
**Status:** ⬜ Pending

The system SHOULD support 50,000 entities.

**Targets:**
- 10K entities: 60 FPS
- 50K entities: 30 FPS

---

## API (FR-API)

### FR-API-001: Public Simulation API
**Priority:** P0 (Critical)  
**Status:** ✅ Implemented

```rust
pub trait SimulationEngine {
    fn tick(&mut self);
    fn state(&self) -> &WorldState;
    fn snapshot(&self) -> Snapshot;
    fn restore(&mut self, snapshot: &Snapshot);
    fn query<Q: Query>(&self) -> QueryResult<Q>;
}
```

---

## TESTING (FR-TEST)

### FR-TEST-001: Determinism Tests
**Priority:** P0 (Critical)  
**Status:** ✅ Implemented

```rust
#[test]
fn test_determinism() { ... }
#[test]  
fn test_step_advances_tick() { ... }
#[test]
fn test_step_decreases_energy() { ... }
```

---

## DOCUMENTATION (FR-DOC)

### FR-DOC-001: API Documentation
**Priority:** P1 (High)  
**Status:** ⬜ Pending

All public APIs MUST have docstrings.

---

## Summary

| Category | Total | Done | Pending |
|----------|-------|------|---------|
| Core | 5 | 4 | 1 |
| Economy | 3 | 1 | 2 |
| Metrics | 2 | 1 | 1 |
| AI | 2 | 0 | 2 |
| Networking | 2 | 0 | 2 |
| Performance | 2 | 1 | 1 |
| API | 1 | 1 | 0 |
| Testing | 1 | 1 | 0 |
| Documentation | 1 | 0 | 1 |
| **TOTAL** | **19** | **9** | **10** |
