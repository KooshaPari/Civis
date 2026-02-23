# CIV Comprehensive Technical Specification

**Project:** CIV - Deterministic Civilization Simulation Engine  
**Version:** 1.0  
**Status:** Draft  
**Last Updated:** 2026-02-23

---

## Table of Contents

1. [Architecture Overview](#architecture-overview)
2. [Core Systems](#core-systems)
3. [Entity Component System](#entity-component-system)
4. [Economy System](#economy-system)
5. [AI & Behavior](#ai--behavior)
6. [Network Protocol](#network-protocol)
7. [Performance Requirements](#performance-requirements)
8. [API Reference](#api-reference)

---

## Architecture Overview

```
┌─────────────────────────────────────────────────────────────────────┐
│                        CIV Simulation Engine                         │
├─────────────────────────────────────────────────────────────────────┤
│                                                                      │
│  ┌──────────────┐   ┌──────────────┐   ┌──────────────┐         │
│  │   Physics    │   │   Economy    │   │    AI        │         │
│  │   Engine     │   │   Engine     │   │   Engine     │         │
│  └──────┬───────┘   └──────┬───────┘   └──────┬───────┘         │
│         │                   │                   │                   │
│         └───────────────────┼───────────────────┘                   │
│                             │                                       │
│                    ┌────────▼────────┐                              │
│                    │  Tick Scheduler │                              │
│                    │  (60 Hz tick)   │                              │
│                    └────────┬────────┘                              │
│                             │                                       │
│         ┌───────────────────┼───────────────────┐                   │
│         │                   │                   │                   │
│  ┌──────▼──────┐   ┌──────▼──────┐   ┌──────▼──────┐          │
│  │   Input     │   │   State     │   │   Output    │          │
│  │   System    │◄─►│   (ECS)     │◄─►│   System     │          │
│  └─────────────┘   └──────────────┘   └─────────────┘          │
│                                                                      │
└─────────────────────────────────────────────────────────────────────┘
```

### Design Principles

1. **Deterministic**: Same seed → Same result (always)
2. **Fixed-Point**: No floating-point arithmetic (i64 with 10^6 scale)
3. **ECS-Based**: Data-oriented design for cache efficiency
4. **Reproducible**: Full state snapshots for replay/debugging
5. **Extensible**: Plugin architecture for custom components

---

## Core Systems

### 1.1 Tick Scheduler

The simulation runs at a fixed 60 Hz tick rate (16.67ms per tick).

```
Tick Structure:
├── Pre-Update Phase
│   ├── Input Processing (100μs budget)
│   └── AI Decision Making (1ms budget)
├── Update Phase
│   ├── Physics (2ms budget)
│   │   ├── Movement
│   │   ├── Collision
│   │   └── Pathfinding
│   ├── Economy (1ms budget)
│   │   ├── Production
│   │   ├── Consumption
│   │   └── Trade
│   ├── AI (2ms budget)
│   │   ├── Behavior Trees
│   │   ├── Goal Planning
│   │   └── Event Processing
│   └── Governance (1ms budget)
│       ├── Policy Application
│       ├── Metrics Calculation
│       └── Event Generation
├── Post-Update Phase
│   ├── State Validation
│   ├── Snapshot (every N ticks)
│   └── Network Sync
```

### 1.2 Fixed-Point Arithmetic

All numerical operations use fixed-point arithmetic for determinism:

```rust
pub struct Fixed {
    pub raw: i64,  // Scaled by 10^6
}

pub const SCALE: i64 = 1_000_000;

// Example: 1.5 joules = Fixed { raw: 1_500_000 }
```

**Precision Table:**

| Value Type | Range | Precision |
|------------|-------|-----------|
| Energy (Joules) | ±9.2 × 10^15 | 1 μJ |
| Resources | ±9.2 × 10^15 | 1 unit |
| Percentages | 0-100% | 0.0001% |
| Coordinates | ±2^31 | 1 unit |

---

## Entity Component System

### Components

| Component | Fields | Size |
|-----------|--------|------|
| Position | x: i32, y: i32 | 8 bytes |
| Citizen | age, health, ideology, welfare, job | 40 bytes |
| Building | type, hp, max_hp, position | 48 bytes |
| Resources | food, wood, metal, energy | 32 bytes |
| Production | output_type, rate | 16 bytes |
| MilitaryUnit | type, strength, morale, position, faction_id | 48 bytes |
| Faction | name, ideology, treasury | Variable |

### Component Flags

Components can be marked for:
- `Persistent`: Saved in snapshots
- `Networked`: Replicated to clients
- `Indexed`: Indexed for queries

### Query Patterns

```rust
// Query all citizens of a specific job
for (_, (citizen, position)) in world.query::<(&Citizen, &Position)>()
    .with(job: JobType::Farmer)
    .iter() { ... }

// Query all buildings in a region
for (_, (building, position)) in world.query::<(&Building, &Position)>()
    .intersecting(bounds)
    .iter() { ... }
```

---

## Economy System

### 1.3 Resource Flow

```
                    ┌─────────────────┐
                    │   Production    │
                    │  (buildings,   │
                    │   citizens)     │
                    └────────┬────────┘
                             │
              ┌──────────────┼──────────────┐
              │              │              │
              ▼              ▼              ▼
       ┌───────────┐ ┌───────────┐ ┌───────────┐
       │   Food    │ │   Wood    │ │  Metal    │
       │  Storage  │ │  Storage  │ │  Storage  │
       └─────┬─────┘ └─────┬─────┘ └─────┬─────┘
             │             │             │
             │    Consumption (citizens, buildings) 
             │             │             │
             ▼             ▼             ▼
       ┌─────────────────────────────────────────┐
       │           Joule Economy                │
       │   Energy = limit for all production   │
       └─────────────────┬───────────────────────┘
                         │
                         ▼
              ┌─────────────────────┐
              │  Production Rate     │
              │  = min(resource,    │
              │       energy/10)    │
              └─────────────────────┘
```

### 1.4 Job System

| Job | Food Production | Energy Cost | Special |
|-----|-----------------|-------------|--------|
| Farmer | +10 food/tick | -1/tick | |
| Warrior | 0 | -2/tick | Combat |
| Scholar | 0 | -1/tick | Research |
| Trader | 0 | -1/tick | Trade routes |
| Priest | 0 | -1/tick | Happiness |
| Admin | 0 | -2/tick | Governance |
| Unemployed | -1 food | -0.5/tick | Unrest risk |

### 1.5 Building Types

| Building | Cost (Joules) | Production | Maintenance |
|----------|---------------|------------|-------------|
| Farm | 1,000 | +10 food/tick | -1/tick |
| Mine | 2,000 | +5 metal/tick | -2/tick |
| Barracks | 3,000 | Unit training | -3/tick |
| Temple | 1,500 | +5 happiness | -1/tick |
| Market | 1,500 | Trade bonus | -1/tick |
| House | 500 | +10 pop cap | -0.5/tick |
| CityCenter | 10,000 | Governance | -5/tick |

---

## AI & Behavior

### 1.6 Citizen AI

Each citizen has:
- **Needs**: hunger, happiness, safety
- **Ideology**: -1.0 (libertarian) to +1.0 (authoritarian)
- **Skills**: combat, farming, research, trade

**Behavior Decision Tree:**
```
Citizen Decision Priority:
1. Survival (hunger > 0.8) → Find food
2. Safety (threat nearby) → Flee/Combat
3. Job Satisfaction → Continue/Change job
4. Ideological → Support faction goals
5. Social → Interact with others
```

### 1.7 Faction AI

Each faction has:
- **Goals**: expansion, wealth, dominance, survival
- **Resources**: treasury, population, military
- **Memory**: historical decisions

**Goal Planning:**
```rust
enum Goal {
    Expand { target: Position },
    Trade { partner: FactionId },
    War { target: FactionId },
    Research { technology: TechId },
    Build { building: BuildingType },
}

struct GoalPlanner {
    // BFS/A* through goal tree
    // Evaluate utility: expected_joules / cost
    // Select highest utility path
}
```

---

## Network Protocol

### 1.8 Event Types

| Event | Direction | Priority | Payload Size |
|-------|-----------|----------|--------------|
| TickSync | Server→Client | High | 100 bytes |
| EntityUpdate | Server→Client | High | Variable |
| ActionRequest | Client→Server | Medium | 50 bytes |
| ActionResult | Server→Client | Medium | 50 bytes |
| Chat | Bidirectional | Low | Variable |

### 1.9 State Synchronization

```
Full State Sync (on connect):
├── WorldState (1KB)
├── Entities (10KB per 1K entities)
└── History buffer (100 ticks, 100KB)

Delta Sync (per tick):
├── Changed entities (variable)
├── Events (variable)
└── Metrics delta (100 bytes)
```

---

## Performance Requirements

### 1.10 Benchmarks

| Metric | Target | Must Not Exceed |
|--------|--------|------------------|
| Tick time | 10ms | 16ms |
| Memory (10K entities) | 50MB | 100MB |
| Memory (50K entities) | 200MB | 500MB |
| Save time | 500ms | 1s |
| Load time | 500ms | 1s |
| Query latency | 1ms | 10ms |

### 1.11 Profiling Targets

- ECS query: < 0.1ms for 10K entities
- Fixed-point math: < 1μs per operation
- State snapshot: < 50ms for 10K entities

---

## API Reference

### Simulation API

```rust
pub struct Simulation {
    pub state: WorldState,
    pub world: World,
}

impl Simulation {
    pub fn new() -> Self;
    pub fn with_seed(seed: u64) -> Self;
    pub fn tick(&mut self);
    pub fn snapshot(&self) -> SimulationSnapshot;
    pub fn restore(&mut self, snapshot: &SimulationSnapshot);
}

pub struct WorldState {
    pub tick: u64,
    pub population: u64,
    pub energy_budget_joules: Fixed,
    pub rng_seed: u64,
    pub factions: HashMap<u32, String>,
    pub faction_treasury: HashMap<u32, Fixed>,
}
```

### Query API

```rust
// Create query
let mut query = world.query::<(&Citizen, &Position)>();

// With filter
query.with(job: JobType::Farmer);

// Iterate
for (entity, (citizen, position)) in query.iter() {
    // ...
}

// Mutations
for (entity, mut citizen) in world.query::<&mut Citizen>().iter() {
    citizen.age += 1;
}
```

### Serialization

```rust
// Full state serialization
let json = serde_json::to_string(&simulation.state).unwrap();

// Snapshot
let snapshot = simulation.snapshot();
let json = serde_json::to_string(&snapshot).unwrap();

// Restore
let snapshot: SimulationSnapshot = serde_json::from_str(&json).unwrap();
simulation.restore(&snapshot);
```

---

## Implementation Status

| Module | Status | LOC |
|--------|--------|-----|
| Fixed-point math | ✅ Done | 100 |
| ECS core | ✅ Done | 200 |
| World state | ✅ Done | 100 |
| Basic tick loop | ✅ Done | 150 |
| Production phase | ✅ Done | 50 |
| Citizen lifecycle | ✅ Done | 50 |
| Military phase | ✅ Done | 30 |
| Economy phase | ✅ Done | 30 |
| Policy module | ✅ Done | 20 |
| Metrics module | ✅ Done | 30 |
| I/O module | ✅ Done | 10 |
| **Total** | | **~770** |

---

## Future Extensions

### Planned Features

1. **Multiplayer**: Turn-based synchronization
2. **Modding API**: Scriptable behavior trees
3. **Visual Editor**: Entity placement tool
4. **AI Library**: Pre-built faction behaviors
5. **Replay System**: Full game playback
6. **Save Games**: Compressed state + metadata

### Research Directions

1. **WASM Compilation**: Browser-based simulation
2. **ML Integration**: Neural network agents
3. **Procedural Generation**: Infinite worlds
4. **Cloud Gaming**: Streaming simulation
