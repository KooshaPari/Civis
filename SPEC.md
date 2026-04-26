# Civis — SPEC.md

## Overview

Civis (civ) is a deterministic simulation and policy-driven architecture workspace for headless civilization simulation. It provides a modular Rust workspace with separate crates for engine, economy, spatial systems, climate modeling, actors, policy, and metrics.

---

## Architecture

```
┌─────────────────────────────────────────────────────────────────┐
│                         Civis (civ)                              │
├─────────────────────────────────────────────────────────────────┤
│  ┌──────────────────────────────────────────────────────────┐  │
│  │                    Simulation Server                      │  │
│  │  ┌─────────────┐  ┌─────────────┐  ┌──────────────────┐ │  │
│  │  │   WebSocket │ │   REST API  │ │   Scenario       │ │  │
│  │  │   Server    │ │   (HTTP)    │ │   Runner         │ │  │
│  │  │             │ │             │ │                  │ │  │
│  │  │ • Tick      │ │ • State     │ │ • YAML loader    │ │  │
│  │  │ • Broadcast │ │ • Metrics   │ │ • Deterministic  │ │  │
│  │  └─────────────┘  └─────────────┘  └──────────────────┘ │  │
│  └──────────────────────────────────────────────────────────┘  │
│                              │                                   │
│  ┌───────────────────────────┴──────────────────────────────┐  │
│  │                   CivEngine (Core)                        │  │
│  │  ┌────────────┐ ┌────────────┐ ┌────────────┐           │  │
│  │  │    Tick    │ │   State    │ │   Replay   │           │  │
│  │  │   Loop     │ │   Manager  │ │   Engine   │           │  │
│  │  │            │ │            │ │            │           │  │
│  │  │ • Time     │ │ • World    │ │ • Record   │           │  │
│  │  │ • Events   │ │ • Entities │ │ • Verify   │           │  │
│  │  └────────────┘ └────────────┘ └────────────┘           │  │
│  └──────────────────────────────────────────────────────────┘  │
│                              │                                   │
│  ┌───────────────────────────┴──────────────────────────────┐  │
│  │              Civilization Systems (Crates)                  │  │
│  │  ┌────────┐ ┌────────┐ ┌────────┐ ┌────────┐ ┌────────┐ │  │
│  │  │Economy │ │Actors  │ │Policy  │ │Climate │ │Metrics │ │  │
│  │  │        │ │        │ │        │ │        │ │        │ │  │
│  │  │• Market│ │• Citizen│ │• Diplo │ │• Weather│ │• Time  │ │  │
│  │  │• Joule │ │• Social│ │• War   │ │• Season │ │• Series│ │  │
│  │  └────────┘ └────────┘ └────────┘ └────────┘ └────────┘ │  │
│  └──────────────────────────────────────────────────────────┘  │
└─────────────────────────────────────────────────────────────────┘
```

---

## Components

### Workspace Crates

| Crate | Responsibility | Key Types |
|-------|----------------|-----------|
| `civ-engine` | Core simulation loop | `Simulation`, `TickResult` |
| `civ-economy` | Market, resources, trade | `Market`, `JouleAllocator` |
| `civ-actors` | Citizens, military units | `Citizen`, `MilitaryUnit` |
| `civ-policy` | Diplomacy, institutions | `DiplomaticRelation`, `ShadowNetwork` |
| `civ-climate` | Weather, seasons, disasters | `ClimateSystem`, `WeatherEvent` |
| `civ-metrics` | Time-series, exports | `TimeSeries`, `MetricsSnapshot` |
| `civ-server` | WebSocket + REST server | `SimServer`, `ClientHandler` |
| `civ-spatial` | Maps, terrain, movement | `TerrainMap`, `Pathfinder` |
| `civ-geo` | Geographic features | `Region`, `ResourceDeposit` |
| `civ-social` | Ideology, institutions | `Institution`, `Ideology` |

### Simulation Core

| Component | Responsibility | Determinism |
|-----------|----------------|-------------|
| `TickLoop` | Advance simulation time | Seeded RNG |
| `EventQueue` | Process scheduled events | Ordered by tick |
| `StateManager` | World state snapshots | Full serialization |
| `ReplayEngine` | Record + verify runs | Byte-perfect replay |

---

## Data Models

### Core Simulation

```rust
// crates/engine/src/lib.rs
pub struct Simulation {
    pub tick: u64,
    pub seed: u64,
    pub rng: ChaCha8Rng,
    pub world: WorldState,
    pub systems: Vec<Box<dyn System>>,
}

pub struct WorldState {
    pub citizens: Vec<Citizen>,
    pub institutions: Vec<Institution>,
    pub market: Market,
    pub climate: ClimateState,
    pub terrain: TerrainMap,
}

pub trait System {
    fn tick(&mut self, world: &mut WorldState, rng: &mut dyn Rng);
}
```

### Economy

```rust
// crates/economy/src/lib.rs
pub struct Market {
    pub prices: HashMap<GoodId, Price>,
    pub supply: HashMap<GoodId, Quantity>,
    pub demand: HashMap<GoodId, Quantity>,
    pub trade_routes: Vec<TradeRoute>,
}

pub struct JouleAllocator {
    pub total_energy: Joules,
    pub allocations: Vec<(ActorId, Joules)>,
}

impl JouleAllocator {
    pub fn allocate(&self, actors: &[Actor]) -> Vec<(ActorId, Joules)> {
        // Energy conservation: sum(allocations) <= total_energy
    }
}
```

### Actors

```rust
// crates/actors/src/lib.rs
pub struct Citizen {
    pub id: CitizenId,
    pub age: u32,
    pub health: f32,
    pub wealth: f32,
    pub employment: EmploymentStatus,
    pub ideology: f32, // -1.0 to 1.0 (libertarian to authoritarian)
    pub social_ties: Vec<CitizenId>,
}

pub enum EmploymentStatus {
    Unemployed,
    Employed(OrganizationId),
    SelfEmployed,
    Retired,
}

pub struct MilitaryUnit {
    pub id: UnitId,
    pub unit_type: UnitType,
    pub strength: u32,
    pub morale: f32,
    pub fatigue: f32,
    pub faction: FactionId,
    pub position: Coordinate,
}
```

### Policy & Diplomacy

```rust
// crates/policy/src/lib.rs
pub struct DiplomaticRelation {
    pub faction_a: FactionId,
    pub faction_b: FactionId,
    pub sentiment: f32, // -1.0 to 1.0
    pub pact_type: Option<PactType>,
    pub trade_agreement: bool,
}

pub struct ShadowNetwork {
    pub members: Vec<ActorId>,
    pub influence: f32,
    pub detection_risk: f32,
    pub covert_actions: Vec<CovertAction>,
}

pub struct Institution {
    pub id: InstitutionId,
    pub name: String,
    pub policies: Vec<Policy>,
    pub members: Vec<ActorId>,
    pub budget: f32,
    pub approval_rating: f32,
}
```

### Metrics

```rust
// crates/metrics/src/lib.rs
pub struct MetricsSnapshot {
    pub tick: u64,
    pub population: u32,
    pub gdp: f32,
    pub avg_ideology: f32,
    pub health_index: f32,
    pub conflict_score: f32,
}

pub struct TimeSeries<T> {
    pub data: Vec<(u64, T)>,
}

impl<T> TimeSeries<T> {
    pub fn query_range(&self, start: u64, end: u64) -> &[(u64, T)] {
        // Binary search for range
    }
}
```

---

## Stack

| Category | Technology | Version |
|----------|------------|---------|
| Language | Rust | Edition 2021 |
| Async Runtime | Tokio | 1.x |
| Serialization | Serde | 1.x |
| RNG | rand + ChaCha8Rng | 0.8 |
| Testing | proptest | 1.x |
| Coverage | cargo-tarpaulin | Latest |
| Benchmarks | Criterion | 0.5 |

---

## API Contract

### WebSocket Protocol

```
Client → Server
{
  "command": "subscribe",
  "room": "simulation_main"
}

Server → Client (tick update)
{
  "tick": 12345,
  "snapshot": {
    "population": 10000,
    "gdp": 5000000.0,
    "avg_ideology": 0.2
  },
  "events": [
    {"type": "Trade", "data": {...}},
    {"type": "Birth", "data": {...}}
  ]
}
```

### REST Endpoints

| Endpoint | Method | Description |
|----------|--------|-------------|
| `/api/v1/scenarios` | GET | List available scenarios |
| `/api/v1/scenarios/:id` | POST | Start scenario |
| `/api/v1/simulations/:id` | GET | Get simulation state |
| `/api/v1/simulations/:id/metrics` | GET | Export metrics (CSV/JSON) |
| `/api/v1/simulations/:id/replay` | POST | Replay validation |

---

## Determinism

| Aspect | Implementation |
|--------|----------------|
| RNG | ChaCha8Rng with seed per simulation |
| Ordering | Fixed iteration order for all collections |
| Floating point | No cross-platform determinism guarantee |
| Serialization | Bincode for exact state capture |

### Replay Format

```rust
pub struct SimulationReplay {
    pub seed: u64,
    pub tick_count: u64,
    pub initial_state: WorldState,
    pub events: Vec<(u64, EventType, EventData)>,
}
```

---

## Performance

| Metric | Target |
|--------|--------|
| Ticks per second | 1000+ (simple scenario) |
| Concurrent clients | 100+ WebSocket connections |
| Memory per simulation | <500MB |
| Replay verification | 10x speed |
| State snapshot | <100ms |

---

## Project Structure

```
Civis/
├── crates/
│   ├── engine/               # Core simulation
│   │   ├── src/
│   │   └── tests/
│   ├── economy/              # Market, resources
│   ├── actors/               # Citizens, military
│   ├── policy/               # Diplomacy, institutions
│   ├── climate/              # Weather, seasons
│   ├── metrics/              # Time-series, exports
│   ├── spatial/              # Maps, terrain
│   ├── geo/                  # Geographic features
│   ├── social/               # Ideology, networks
│   └── server/               # WebSocket + REST
├── docs/
│   ├── wiki/                 # Knowledge base
│   ├── development-guide/    # Contributor docs
│   ├── api/                  # API documentation
│   └── roadmap/              # Planning artifacts
├── scenarios/                # YAML scenario files
└── Cargo.toml               # Workspace manifest
```

---

## References

- [Rust Book](https://doc.rust-lang.org/book/)
- [Tokio Documentation](https://tokio.rs/)
- [Serde Documentation](https://serde.rs/)
- [PLAN.md](./PLAN.md) — Implementation phases
- [AGENTS.md](./AGENTS.md) — Development guidelines
