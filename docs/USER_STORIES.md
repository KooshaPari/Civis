# CIV User Stories

**Project:** CIV - Deterministic Civilization Simulation Engine

---

## As a Game Developer

### Story 1: Deterministic Replay
**As a** game developer  
**I want** to be able to replay any simulation from a seed  
**So that** I can reproduce bugs and analyze game balance

**Acceptance Criteria:**
- [ ] Given a seed, simulation produces identical results across runs
- [ ] Can export full state snapshot to JSON
- [ ] Can restore from snapshot and continue simulation
- [ ] Determinism verified via automated tests

### Story 2: Large-Scale Population
**As a** game developer  
**I want** to simulate 10,000+ citizens  
**So that** I can create massive civilization scenarios

**Acceptance Criteria:**
- [ ] ECS handles 10K+ entities without degradation
- [ ] Tick rate maintains 60 ticks/second
- [ ] Memory usage stays under 100MB for 10K entities

### Story 3: Custom Components
**As a** game developer  
**I want** to add custom ECS components  
**So that** I can extend simulation without modifying core

**Acceptance Criteria:**
- [ ] Can define new components via Rust structs
- [ ] Can query entities by custom component combinations
- [ ] Documentation for extension patterns

---

## As a Game Player

### Story 4: Economy Management
**As a** player  
**I want** to manage resources (food, wood, metal, energy)  
**So that** my civilization can grow

**Acceptance Criteria:**
- [ ] Citizens consume food based on job
- [ ] Buildings produce/consume resources
- [ ] Can view resource flow in real-time
- [ ] Energy (joules) is the limiting factor

### Story 5: Citizen Assignment
**As a** player  
**I want** to assign citizens to jobs  
**So that** I can optimize production

**Acceptance Criteria:**
- [ ] Can assign citizens to: Farmer, Warrior, Scholar, Trader, Priest, Admin
- [ ] Each job has different resource production/consumption
- [ ] Unemployed citizens consume resources without producing
- [ ] Job satisfaction affects citizen happiness

### Story 6: Policy Effects
**As a** player  
**I want** policies that affect consumption  
**So that** I can balance tyranny vs efficiency

**Acceptance Criteria:**
- [ ] Can set base consumption rate
- [ ] Scarcity multiplier affects all consumption
- [ ] High tyranny reduces citizen happiness
- [ ] Low legitimacy causes unrest events

---

## As an AI Researcher

### Story 7: Agent Simulation
**As an** AI researcher  
**I want** to run autonomous agent simulations  
**So that** I can study emergent civilization behavior

**Acceptance Criteria:**
- [ ] API to inject agent decisions each tick
- [ ] Can record full simulation history
- [ ] Metrics export for analysis
- [ ] Support for reinforcement learning feedback

### Story 8: Reproducible Research
**As an** AI researcher  
**I want** deterministic, reproducible simulations  
**So that** I can compare agent strategies fairly

**Acceptance Criteria:**
- [ ] Same seed = same simulation result
- [ ] Determinism verified empirically
- [ ] Snapshot/restore for checkpointing
- [ ] Export metrics to standard formats

---

## Technical Constraints

- All calculations use fixed-point arithmetic (no floating point)
- Simulation must be deterministic across platforms (Rust, WASM)
- State snapshots must be backward compatible
- Network protocol uses NATS message bus
