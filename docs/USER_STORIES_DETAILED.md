# CIV User Stories - Complete

**Project:** CIV - Deterministic Civilization Simulation Engine  
**Version:** 2.0

---

## Epic 1: Core Simulation

### Story 1.1: Deterministic Simulation
**As a** simulation developer  
**I want** reproducible results from a seed  
**So that** I can debug issues and verify fixes

**Acceptance Criteria:**
- [ ] Given seed `12345`, simulation produces identical state at tick N
- [ ] Identical results on: macOS, Linux, Windows, WASM
- [ ] CI runs determinism test on every commit
- [ ] Determinism verified for 10,000 ticks minimum

**Technical Notes:**
- Use ChaCha8RNG seeded from u64
- No system time in simulation logic
- No random floating-point operations

---

### Story 1.2: Tick Execution
**As a** game developer  
**I want** 60 Hz tick rate  
**So that** simulation feels smooth

**Acceptance Criteria:**
- [ ] Each tick completes in <16ms
- [ ] Tick runs physics, economy, AI, governance phases
- [ ] Time budget enforced (hard fail if exceeded)
- [ ] Benchmark reports tick time p50/p95/p99

---

### Story 1.3: Entity Management
**As a** game developer  
**I want** to create and query entities  
**So that** I can build game systems

**Acceptance Criteria:**
- [ ] Can spawn entities with multiple components
- [ ] Can query by component type
- [ ] Can filter queries by component values
- [ ] Can modify components in place

**Example:**
```rust
// Spawn citizen
world.spawn((
    Citizen { age: 25, job: Some(JobType::Farmer), ... },
    Position { x: 10, y: 20 },
));

// Query farmers
for (_, (citizen, position)) in world.query::<(&Citizen, &Position)>()
    .with(job: JobType::Farmer)
    .iter() { ... }
```

---

### Story 1.4: State Snapshots
**As a** game developer  
**I want** to save and restore simulation state  
**So that** I can implement save games

**Acceptance Criteria:**
- [ ] `snapshot()` returns complete state as JSON
- [ ] `restore(json)` restores state exactly
- [ ] Snapshot includes: tick, population, entities, factions
- [ ] Can continue simulation after restore

---

## Epic 2: Economy

### Story 2.1: Energy Economy
**As a** player  
**I want** joule-based energy management  
**So that** resources are limited and meaningful

**Acceptance Criteria:**
- [ ] Buildings produce energy per tick
- [ ] Citizens consume energy per tick
- [ ] Energy storage has maximum capacity
- [ ] Scarcity multiplier affects all consumption

---

### Story 2.2: Resource Production
**As a** player  
**I want** buildings to produce resources  
**So that** my civilization grows

**Acceptance Criteria:**
- [ ] Farms produce food
- [ ] Mines produce metal
- [ ] Production rate configurable per building
- [ ] Resources accumulate in storage

---

### Story 2.3: Citizen Jobs
**As a** player  
**I want** to assign citizens to jobs  
**So that** I can optimize production

**Acceptance Criteria:**
- [ ] Can assign job to individual citizen
- [ ] Can mass-assign by filter
- [ ] Each job has different production/consumption
- [ ] Job changes take effect immediately

---

### Story 2.4: Trade Routes
**As a** player  
**I want** to establish trade between settlements  
**So that** I can profit from specialization

**Acceptance Criteria:**
- [ ] Can create trade route between two points
- [ ] Trade generates income (joules)
- [ ] Trade routes can be disrupted (war, disaster)
- [ ] Trade efficiency visible in UI

---

## Epic 3: Governance

### Story 3.1: Tyranny Metrics
**As a** player  
**I want** to see tyranny/legitimacy metrics  
**So that** I know my governance status

**Acceptance Criteria:**
- [ ] Tyranny index calculated: consumption/budget
- [ ] Legitimacy = 1 - tyranny
- [ ] Thresholds: <0.3 stable, 0.3-0.7 unrest, >0.7 rebellion
- [ ] Visual indicator in UI

---

### Story 3.2: Policy Effects
**As a** player  
**I want** policies that affect simulation  
**So that** I can make meaningful choices

**Acceptance Criteria:**
- [ ] Can set base consumption rate
- [ ] Can set scarcity multiplier
- [ ] Policies have immediate effect
- [ ] Policy history tracked

---

### Story 3.3: Faction Tracking
**As a** player  
**I want** to track multiple factions  
**So that** I can manage diplomacy

**Acceptance Criteria:**
- [ ] Each faction has: name, treasury, population
- [ ] Can view faction relationship status
- [ ] Faction metrics update each tick
- [ ] Faction UI shows all relevant data

---

## Epic 4: AI

### Story 4.1: Citizen Behavior
**As an** AI researcher  
**I want** citizens to make decisions  
**So that** emergent behavior arises

**Acceptance Criteria:**
- [ ] Citizens prioritize survival needs
- [ ] Citizens change jobs based on satisfaction
- [ ] Ideology affects behavior
- [ ] Behavior is deterministic

---

### Story 4.2: Faction AI
**As an** AI researcher  
**I want** autonomous faction AI  
**So that** I can study strategy emergence

**Acceptance Criteria:**
- [ ] Factions evaluate goals each tick
- [ ] Goals ranked by utility
- [ ] Actions executed to pursue goals
- [ ] AI is deterministic

---

### Story 4.3: Agent Integration
**As an** AI researcher  
**I want** to inject agent decisions  
**So that** I can study human-AI interaction

**Acceptance Criteria:**
- [ ] API to submit agent actions each tick
- [ ] Actions validated before execution
- [ ] Action results returned to agent
- [ ] Full simulation history exported

---

## Epic 5: Multiplayer

### Story 5.1: Turn-Based Play
**As a** player  
**I want** to play turns with friends  
**So that** I can compete

**Acceptance Criteria:**
- [ ] Server manages turn order
- [ ] Players submit actions within time limit
- [ ] Server executes all actions
- [ ] Results broadcast to all players

---

### Story 5.2: Real-Time Sync
**As a** player  
**I want** real-time state synchronization  
**So that** I can see other players' actions

**Acceptance Criteria:**
- [ ] Delta compression for bandwidth
- [ ] <100ms latency for local networks
- [ ] Reconnection handling
- [ ] Rollback on conflict

---

### Story 5.3: Spectator Mode
**As a** observer  
**I want** to watch games  
**So that** I can learn strategies

**Acceptance Criteria:**
- [ ] Can spectate ongoing game
- [ ] Can pause/resume
- [ ] Can view from any faction
- [ ] Can view statistics

---

## Epic 6: Development

### Story 6.1: Custom Components
**As a** mod developer  
**I want** to add custom components  
**So that** I can extend the simulation

**Acceptance Criteria:**
- [ ] Can define new component types
- [ ] Can register custom queries
- [ ] Components serializable
- [ ] Performance impact documented

---

### Story 6.2: Custom Systems
**As a** mod developer  
**I want** to add custom game systems  
**So that** I can create new mechanics

**Acceptance Criteria:**
- [ ] Can register tick-phase callbacks
- [ ] Can access all game state
- [ ] Can emit events
- [ ] System ordering configurable

---

### Story 6.3: WASM Export
**As a** web developer  
**I want** to run simulation in browser  
**So that** I can create web games

**Acceptance Criteria:**
- [ ] Compiles to WASM without modification
- [ ] WASM size < 2MB
- [ ] JavaScript bindings provided
- [ ] Performance acceptable (30+ FPS)

---

## Epic 7: Testing

### Story 7.1: Property-Based Tests
**As a** QA engineer  
**I want** property-based testing  
**So that** I find edge cases

**Acceptance Criteria:**
- [ ] Proptest integration working
- [ ] 1000+ iterations per property
- [ ] Shrinking finds minimal counterexample
- [ ] CI runs property tests

---

### Story 7.2: Benchmarking
**As a** performance engineer  
**I want** performance benchmarks  
**So that** I can detect regressions

**Acceptance Criteria:**
- [ ] Tick time benchmark
- [ ] Query time benchmark
- [ ] Memory usage benchmark
- [ ] CI fails on >10% regression

---

## Priority Matrix

| Story | Points | Epic | Sprint |
|-------|--------|------|--------|
| 1.1 Determinism | 3 | Core | 1 |
| 1.2 Tick Execution | 5 | Core | 1 |
| 1.3 Entities | 5 | Core | 1 |
| 1.4 Snapshots | 3 | Core | 2 |
| 2.1 Energy | 3 | Economy | 2 |
| 2.2 Resources | 5 | Economy | 2 |
| 2.3 Jobs | 3 | Economy | 3 |
| 2.4 Trade | 5 | Economy | 4 |
| 3.1 Tyranny | 2 | Governance | 3 |
| 3.2 Policies | 3 | Governance | 3 |
| 3.3 Factions | 3 | Governance | 3 |
| 4.1 Citizens | 5 | AI | 4 |
| 4.2 Faction AI | 8 | AI | 5 |
| 4.3 Agents | 5 | AI | 5 |
| 5.1 Turns | 8 | Multiplayer | 6 |
| 5.2 Sync | 8 | Multiplayer | 6 |
| 5.3 Spectator | 3 | Multiplayer | 7 |
| 6.1 Components | 5 | Development | 7 |
| 6.2 Systems | 8 | Development | 8 |
| 6.3 WASM | 8 | Development | 8 |
| 7.1 Property Tests | 5 | Testing | 1 |
| 7.2 Benchmarks | 3 | Testing | 1 |

**Total Story Points: 107**

---

## Definition of Done

Each story is complete when:
1. All acceptance criteria checked
2. Unit tests added
3. Documentation updated
4. Code reviewed
5. CI passes
