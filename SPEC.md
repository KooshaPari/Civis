# Civis — Comprehensive System Specification

**Document ID:** PHENOTYPE_CIVIS_SPEC  
**Status:** Active Research  
**Last Updated:** 2026-04-03  
**Author:** Phenotype Architecture Team

---

## Table of Contents

1. [Project Overview](#1-project-overview)
2. [Architecture](#2-architecture)
3. [Functionality Specification](#3-functionality-specification)
4. [Technical Architecture](#4-technical-architecture)
5. [API Reference](#5-api-reference)
6. [Error Handling](#6-error-handling)
7. [Security](#7-security)
8. [Data Models](#8-data-models)
9. [ECS Component Registry](#9-ecs-component-registry)
10. [System Execution Pipeline](#10-system-execution-pipeline)
11. [Determinism Architecture](#11-determinism-architecture)
12. [Networking Protocol](#12-networking-protocol)
13. [Serialization & Persistence](#13-serialization--persistence)
14. [Performance Targets](#14-performance-targets)
15. [Testing Strategy](#15-testing-strategy)
16. [Deployment Architecture](#16-deployment-architecture)
17. [Configuration Reference](#17-configuration-reference)
18. [Scenario Format Specification](#18-scenario-format-specification)
19. [Metrics & Observability](#19-metrics--observability)
20. [Modding & Extensibility](#20-modding--extensibility)
21. [Glossary](#21-glossary)
22. [References](#22-references)

---

## 1. Project Overview

### 1.1 Mission Statement

Civis is a **deterministic, policy-driven civilization simulation platform** built in Rust. It provides a modular workspace for simulating complex social, economic, political, and environmental systems with guaranteed reproducibility. Unlike existing agent-based modeling platforms that prioritize ease of use over rigor, Civis guarantees that the same initial conditions always produce identical results — a requirement for policy analysis, scientific research, and validation.

### 1.2 Core Principles

| Principle | Description | Enforcement |
|-----------|-------------|-------------|
| **Determinism First** | Same inputs always produce identical outputs across platforms | Seeded RNG, fixed-point math, ordered collections, CI validation |
| **Policy Native** | Institutions, governance, and policy instruments are first-class citizens | Dedicated policy crate, institution components, legislative systems |
| **Headless Server** | WebSocket + REST API for automation, scale, and integration | crates/server with tokio + tungstenite |
| **Modular Architecture** | ECS-based workspace with separable crates | Workspace Cargo.toml, plugin architecture |
| **Scientific Rigor** | Replay verification, metrics export, reproducibility | Replay engine, checksum chain, export formats |
| **Fail Loud** | No silent compatibility fallbacks; explicit failures only | ThisError enums, panic on invariant violation |

### 1.3 Target Use Cases

| Use Case | Description | Primary User |
|----------|-------------|--------------|
| **Policy Analysis** | Test "what if" scenarios for governance decisions | Policy researchers, think tanks |
| **Academic Research** | Reproducible ABM studies with verification | University researchers, PhD students |
| **Education** | Interactive civilization simulation for classrooms | Educators, students |
| **Game AI** | Deterministic NPC behavior and world simulation | Game developers |
| **Forecasting** | Scenario planning for complex adaptive systems | Organizations, forecasters |
| **AI Training** | Reinforcement learning environment for agents | ML researchers |

### 1.4 Differentiation

| Feature | Civis | NetLogo | MASON | Paradox Games | Unity DOTS |
|---------|-------|---------|-------|---------------|------------|
| Deterministic | Yes | No | Partial | Partial | Partial |
| Replay System | Yes | No | No | No | No |
| Policy First | Yes | No | No | Partial | No |
| Rust Performance | Yes | No | No | No | No |
| Headless Server | Yes | No | No | No | Yes |
| Open Source | Yes | Yes | Yes | No | No |
| Cross-Platform | Yes | Yes | Yes | Partial | Partial |
| WASM Support | Yes | Partial | No | No | No |
| ECS Architecture | Yes | No | No | No | Yes |
| Fixed-Point Math | Yes | No | No | Partial | No |

### 1.5 Workspace Structure

```
Civis/
├── Cargo.toml                    # Workspace manifest (resolver = "2")
├── SPEC.md                       # This specification
├── README.md                     # Quick start guide
├── AGENTS.md                     # Development guidelines
├── CLAUDE.md                     # AI assistant context
├── ADR.md                        # Architecture decision index
├── PRD.md                        # Product requirements
├── PLAN.md                       # Implementation plan
├── FUNCTIONAL_REQUIREMENTS.md    # Functional requirements
├── USER_JOURNEYS.md              # User journey maps
├── SECURITY.md                   # Security policy
│
├── crates/                       # Workspace crates
│   ├── engine/                   # Core simulation engine
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs            # Public API
│   │       ├── simulation.rs     # Simulation state & lifecycle
│   │       ├── tick.rs           # Tick loop implementation
│   │       ├── rng.rs            # Deterministic RNG wrapper
│   │       ├── world.rs          # ECS world management
│   │       ├── replay.rs         # Replay recording & verification
│   │       ├── components/       # ECS component definitions
│   │       ├── systems/          # ECS system implementations
│   │       └── resources/        # ECS resource definitions
│   │
│   └── server/                   # Headless simulation server
│       ├── Cargo.toml
│       └── src/
│           ├── lib.rs            # Public API
│           ├── main.rs           # Entry point
│           ├── websocket.rs      # WebSocket handler
│           ├── http.rs           # REST API (axum)
│           ├── protocol.rs       # Message protocol definitions
│           └── sessions.rs       # Client session management
│
├── docs/                         # Documentation
│   ├── adr/                      # Architecture Decision Records
│   │   ├── ADR-001-deterministic-simulation.md
│   │   ├── ADR-001-rust-crate-structure.md
│   │   ├── ADR-002-ecs-architecture.md
│   │   ├── ADR-002-joule-economy-as-allocator.md
│   │   ├── ADR-003-deterministic-replay.md
│   │   ├── ADR-003-policy-institution-modeling.md
│   │   └── README.md
│   ├── research/                 # State of the Art research
│   │   ├── GAME_ENGINES_SOTA.md
│   │   └── RESEARCH_INDEX.md
│   ├── specs/                    # Feature specifications
│   │   ├── CIV-0001-core-simulation-loop.md
│   │   ├── CIV-0100-economy-v1.md
│   │   └── ...
│   └── ...
│
├── scenarios/                    # YAML scenario files
├── scripts/                      # Build and utility scripts
├── assets/                       # Game assets (textures, audio)
├── web/                          # Web client (if applicable)
└── infra/                        # Infrastructure configuration
```

### 1.6 Technology Stack

| Category | Technology | Version | Justification |
|----------|------------|---------|---------------|
| Language | Rust | Edition 2021 | Performance, safety, determinism |
| ECS | hecs / Bevy ECS | Latest | Lightweight, proven, deterministic iteration |
| RNG | rand_chacha | 0.3+ | ChaCha8Rng, deterministic, parallel streams |
| Fixed-point | fixed | 2.0+ | Deterministic arithmetic for economics |
| Serialization | bincode + serde | 1.3+ / 1.0+ | Fast, compact, deterministic |
| Collections | indexmap | 2.0+ | Deterministic iteration order |
| Async Runtime | tokio | 1.x | Server, networking, async I/O |
| WebSocket | tokio-tungstenite | 0.20+ | WebSocket protocol support |
| HTTP | axum | 0.7+ | REST API framework |
| Testing | proptest | 1.x | Property-based testing |
| Benchmarking | criterion | 0.5+ | Performance benchmarks |
| Hashing | xxhash-rust | 0.8+ | Fast state checksums |
| Error Handling | thiserror | 1.x+ | Derive-based error enums |
| Logging | tracing | 0.1+ | Structured logging |

---

## 2. Architecture

### 2.1 High-Level Architecture

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                           Civis Platform                                     │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                             │
│  ┌──────────────────────────────────────────────────────────────────────┐  │
│  │                      Simulation Server (crates/server)                │  │
│  │                                                                        │  │
│  │   ┌──────────────┐  ┌──────────────┐  ┌──────────────┐             │  │
│  │   │ REST API      │  │ WebSocket    │  │ Scenario     │             │  │
│  │   │ (axum)        │  │ Server       │  │ Runner       │             │  │
│  │   │               │  │ (tungstenite)│  │              │             │  │
│  │   │ • State       │  │              │  │ • YAML       │             │  │
│  │   │ • Metrics     │  │ • Tick       │  │   loader     │             │  │
│  │   │ • Scenarios   │  │   streaming  │  │ • Validation │             │  │
│  │   │ • Replay      │  │ • Events     │  │ • Execution  │             │  │
│  │   └──────────────┘  └──────────────┘  └──────────────┘             │  │
│  └──────────────────────────────────────────────────────────────────────┘  │
│                                   │                                         │
│  ┌───────────────────────────────┴──────────────────────────────────────┐  │
│  │                       CivEngine (crates/engine)                        │  │
│  │                                                                        │  │
│  │   ┌────────────┐  ┌────────────┐  ┌────────────┐  ┌────────────┐    │  │
│  │   │ Tick       │  │ Event      │  │ State      │  │ Replay     │    │  │
│  │   │ Loop       │  │ Queue      │  │ Manager    │  │ Engine     │    │  │
│  │   │            │  │            │  │            │  │            │    │  │
│  │   │ • Time     │  │ • Schedule │  │ • Entities │  │ • Record   │    │  │
│  │   │ • Systems  │  │ • Trigger  │  │ • Snapshots│  │ • Verify   │    │  │
│  │   │ • RNG      │  │ • Callbacks│  │ • Checksums│  │ • Debug    │    │  │
│  │   └────────────┘  └────────────┘  └────────────┘  └────────────┘    │  │
│  └──────────────────────────────────────────────────────────────────────┘  │
│                                   │                                         │
│  ┌───────────────────────────────┴──────────────────────────────────────┐  │
│  │                    Civilization Systems (ECS)                          │  │
│  │                                                                        │  │
│  │   ┌────────┐ ┌────────┐ ┌────────┐ ┌────────┐ ┌────────┐             │  │
│  │   │Economy │ │Actors  │ │Policy  │ │Climate │ │Metrics │             │  │
│  │   │        │ │        │ │        │ │        │ │        │             │  │
│  │   │• Market│ │• Citizen│ │• Diplo │ │• Weather│ │• Time  │             │  │
│  │   │• Joule │ │• Social│ │• War   │ │• Season │ │• Series│             │  │
│  │   │• Trade │ │• Unit  │ │• Govern│ │• Disaster│ │• Export│             │  │
│  │   └────────┘ └────────┘ └────────┘ └────────┘ └────────┘             │  │
│  │                                                                        │  │
│  │   ┌────────┐ ┌────────┐ ┌────────┐ ┌────────┐                       │  │
│  │   │Spatial │ │Geo     │ │Social  │ │Server  │                       │  │
│  │   │        │ │        │ │        │ │        │                       │  │
│  │   │• Map   │ │• Region│ │• Ideo  │ │• API   │                       │  │
│  │   │• Path  │ │• Resource│ │• Network│ │• Socket│                       │  │
│  │   │• Terrain│ │        │ │• Instit │ │        │                       │  │
│  │   └────────┘ └────────┘ └────────┘ └────────┘                       │  │
│  └──────────────────────────────────────────────────────────────────────┘  │
│                                   │                                         │
│  ┌───────────────────────────────┴──────────────────────────────────────┐  │
│  │                        Infrastructure                                  │  │
│  │                                                                        │  │
│  │   ECS (hecs/Bevy) │ RNG (ChaCha8) │ Serialize (bincode) │ Checksums   │  │
│  └──────────────────────────────────────────────────────────────────────┘  │
│                                                                             │
└─────────────────────────────────────────────────────────────────────────────┘
```

### 2.2 Crate Dependency Graph

```
crates/server
    │
    ├── crates/engine
    │       │
    │       ├── hecs (or bevy_ecs)
    │       ├── rand_chacha
    │       ├── fixed
    │       ├── indexmap
    │       ├── bincode
    │       ├── serde
    │       ├── xxhash-rust
    │       └── thiserror
    │
    ├── tokio
    ├── tokio-tungstenite
    ├── axum
    ├── serde_json
    └── tracing
```

### 2.3 Design Philosophy

| Principle | Implementation | Verification |
|-----------|---------------|--------------|
| **Determinism First** | Seeded RNG, fixed-point arithmetic, ordered collections | Cross-platform CI tests, replay verification |
| **Policy Native** | First-class institutions, governance, policy instruments | Policy enactment tests, legitimacy tracking |
| **Headless Server** | WebSocket + REST API for automation and scale | API integration tests, load testing |
| **Modular Architecture** | ECS-based workspace with separable crates | Crate boundary tests, dependency analysis |
| **Scientific Rigor** | Replay verification, metrics export, reproducibility | Determinism benchmarks, checksum validation |
| **Fail Loud** | Explicit failures, no silent fallbacks | Error path tests, panic on invariant violation |

### 2.4 Architecture Decision Records

| ADR | Title | Status | Impact |
|-----|-------|--------|--------|
| ADR-001 | Deterministic Simulation Architecture | Accepted | Core design constraint |
| ADR-001 | Rust Crate Structure | Accepted | Workspace organization |
| ADR-002 | Entity-Component-System Architecture | Accepted | Core architecture pattern |
| ADR-002 | Joule Economy as Allocator | Accepted | Economic system design |
| ADR-003 | Deterministic Replay | Accepted | Verification mechanism |
| ADR-003 | Policy and Institution Modeling | Accepted | Governance system design |

---

## 3. Functionality Specification

### 3.1 Core Simulation Functions

#### 3.1.1 Simulation Lifecycle

```
┌─────────────┐     ┌─────────────┐     ┌─────────────┐     ┌─────────────┐
│  Created    │────>│  Running    │────>│  Paused     │────>│  Completed  │
│             │     │             │     │             │     │             │
│  • Seed set │     │  • Ticking  │     │  • Frozen   │     │  • Final    │
│  • Config   │     │  • Events   │     │  • State    │     │  • Metrics  │
│  • World    │     │  • Metrics  │     │  • Resume   │     │  • Replay   │
│    ready    │     │  • Stream   │     │    ready    │     │    ready    │
└─────────────┘     └─────────────┘     └─────────────┘     └─────────────┘
       │                   │                   │
       │                   ▼                   │
       │            ┌─────────────┐            │
       └───────────>│   Error     │<───────────┘
                    │             │
                    │  • Invalid  │
                    │  • Panic    │
                    │  • Timeout  │
                    └─────────────┘
```

#### 3.1.2 Tick Processing

Each simulation tick executes the following pipeline:

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                          TICK PIPELINE                                       │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                             │
│  Step 1: Pre-Tick Validation                                                │
│  ├── Verify RNG state integrity                                            │
│  ├── Validate world state checksum                                         │
│  └── Check tick limit (if configured)                                      │
│                                                                             │
│  Step 2: System Execution (in order)                                        │
│  ├── PolicySystem      — Update governance, apply policies                 │
│  ├── EconomySystem     — Trade, production, market clearing                │
│  ├── HealthSystem      — Aging, disease, death                             │
│  ├── SocialSystem      — Opinion formation, network updates                │
│  ├── ClimateSystem     — Weather, season effects                           │
│  ├── MovementSystem    — Location changes, migration                       │
│  ├── ConflictSystem    — Combat, war simulation                            │
│  ├── DiplomacySystem   — Treaty updates, war/peace                         │
│  └── MetricSystem      — Snapshot, time-series recording                   │
│                                                                             │
│  Step 3: Post-Tick Processing                                               │
│  ├── Record tick metrics to time-series                                    │
│  ├── Update world state checksum                                           │
│  ├── Record events to replay log (if enabled)                              │
│  ├── Broadcast tick update to connected clients                            │
│  └── Increment tick counter                                                │
│                                                                             │
│  Step 4: Completion Check                                                   │
│  ├── Check tick limit                                                      │
│  ├── Check termination conditions (extinction, victory)                    │
│  └── Return TickResult or continue                                         │
│                                                                             │
└─────────────────────────────────────────────────────────────────────────────┘
```

### 3.2 Economic System Functions

#### 3.2.1 Market Operations

| Operation | Input | Output | Deterministic |
|-----------|-------|--------|---------------|
| `place_order` | Agent, Good, Quantity, Price, Side | Order ID | Yes |
| `execute_trade` | Buy Order, Sell Order | Trade Record | Yes |
| `clear_market` | Order Book | Executed Trades, Remaining Orders | Yes |
| `update_prices` | Supply, Demand, History | New Prices | Yes |
| `calculate_gdp` | All Agent Wealth | GDP Value | Yes |
| `calculate_gini` | All Agent Wealth | Gini Coefficient | Yes |

#### 3.2.2 Joule Allocation

The Joule allocator distributes energy/resources based on priority and demand. Energy conservation is enforced: sum(allocations) must be less than or equal to total_energy.

```rust
pub struct JouleAllocator {
    pub total_energy: f64,
    pub allocations: Vec<(Entity, f64)>,
}

impl JouleAllocator {
    pub fn allocate(&self, agents: &[(Entity, f64)]) -> Vec<(Entity, f64)> {
        let total_demand: f64 = agents.iter().map(|(_, d)| d).sum();
        if total_demand == 0.0 {
            return Vec::new();
        }
        let ratio = self.total_energy / total_demand;
        agents.iter()
            .map(|(entity, demand)| (*entity, demand * ratio))
            .collect()
    }
}
```

### 3.3 Policy System Functions

#### 3.3.1 Policy Lifecycle

```
┌─────────────┐     ┌─────────────┐     ┌─────────────┐     ┌─────────────┐
│  Proposed   │────>│  Debated    │────>│  Enacted    │────>│  Expired    │
│             │     │             │     │             │     │             │
│  • Drafted  │     │  • Voting   │     │  • Active   │     │  • Term     │
│  • Sponsor  │     │  • Lobbying │     │  • Enforced │     │    ended    │
│  • Impact   │     │  • Amend    │     │  • Monitor  │     │  • Repealed │
│    analysis │     │  • Delay    │     │  • Compliance│    │  • Replaced │
└─────────────┘     └─────────────┘     └─────────────┘     └─────────────┘
```

#### 3.3.2 Policy Instruments

| Instrument | Domain | Parameters | Effect |
|------------|--------|------------|--------|
| Taxation | Economic | Rate, Base, Exemptions | Reduces wealth, funds institutions |
| Subsidy | Economic | Amount, Target, Duration | Increases target wealth |
| Regulation | Social | Standard, Penalty | Constrains behavior |
| Trade Agreement | Diplomatic | Tariff, Quota, Partners | Modifies trade flows |
| Military Draft | Conflict | Rate, Age Range, Duration | Creates military units |
| Rationing | Economic | Good, Quota, Priority | Limits consumption |
| Propaganda | Social | Message, Target, Intensity | Shifts ideology |
| Sanction | Diplomatic | Target, Severity, Duration | Reduces target economy |

### 3.4 Social System Functions

#### 3.4.1 Ideology Dynamics

Multi-dimensional ideology space with five axes: economic, social, foreign, authority, and tradition. Each axis ranges from -1.0 to 1.0.

Ideology updates follow the bounded confidence model (Deffuant model): agents only influence each other if their ideological distance is below a confidence threshold. This naturally produces polarization and echo chambers.

#### 3.4.2 Social Network Operations

| Operation | Input | Output | Complexity |
|-----------|-------|--------|------------|
| `add_tie` | Agent A, Agent B, Tie Type | Updated Network | O(1) |
| `remove_tie` | Agent A, Agent B | Updated Network | O(n) |
| `get_neighbors` | Agent | Vec<Entity> | O(1) |
| `propagate_idea` | Idea, Source, Rate | Affected Agents | O(n + e) |
| `calculate_clustering` | Network | Coefficient | O(n * d^2) |
| `find_communities` | Network | Community Map | O(n + e) |

### 3.5 Climate System Functions

#### 3.5.1 Climate Model

Regional climate simulation with global temperature, precipitation patterns, seasonal cycles, and stochastic disaster events. Each region has its own climate zone with base temperature and precipitation patterns.

### 3.6 Spatial System Functions

#### 3.6.1 Terrain & Movement

| Operation | Input | Output | Algorithm |
|-----------|-------|--------|-----------|
| `generate_map` | Width, Height, Seed | TerrainMap | Perlin noise |
| `find_path` | Start, Goal, Map | Vec<Position> | A* |
| `query_radius` | Center, Radius, Map | Vec<Entity> | Spatial hash |
| `calculate_cost` | From, To, Terrain | f32 | Terrain modifier |
| `get_neighbors` | Position, Map | Vec<Position> | Grid adjacency |
| `line_of_sight` | From, To, Map | bool | Bresenham |

---

## 4. Technical Architecture

### 4.1 ECS Architecture

Following ADR-002 (Entity-Component-System Architecture), Civis uses ECS architecture where:

- **Entity**: Lightweight ID (u32 or u64) with no data
- **Component**: Pure data structs with no methods
- **System**: Logic that operates on component queries

#### 4.1.1 Component Design Principles

Components should be small, focused, and cache-friendly. Avoid "god components" that bundle unrelated data. Each component represents a single aspect of an entity.

```rust
// GOOD: Small, focused components
#[derive(Clone, Copy, Debug)]
pub struct Position { pub x: i32, pub y: i32 }

#[derive(Clone, Copy, Debug)]
pub struct Velocity { pub dx: f32, pub dy: f32 }

#[derive(Clone, Copy, Debug)]
pub struct Health { pub current: f32, pub max: f32, pub age: u32 }

// BAD: God component (anti-pattern)
#[derive(Clone, Debug)]
pub struct Citizen {
    pub position: Position,
    pub velocity: Velocity,
    pub health: Health,
    pub inventory: Inventory,
    pub ideology: Ideology,
    // ... 50 more fields
}
```

#### 4.1.2 System Design Principles

Systems should have single responsibility. Each system handles one domain of logic. Systems declare their dependencies to enable correct ordering.

```rust
pub trait System {
    fn name(&self) -> &'static str;
    fn execute(&self, world: &mut World, rng: &mut ChaCha8Rng, tick: Tick);
    fn dependencies(&self) -> Vec<&'static str>;
}
```

#### 4.1.3 Resource Design

Resources are global state accessible to all systems. They include the market, RNG, simulation config, and time-series data.

```rust
pub struct Market {
    pub prices: IndexMap<GoodId, Currency>,
    pub supply: IndexMap<GoodId, f64>,
    pub demand: IndexMap<GoodId, f64>,
    pub trade_routes: Vec<TradeRoute>,
    pub order_book: Vec<Order>,
}

pub struct SimRng {
    pub rng: ChaCha8Rng,
    pub seed: u64,
}
```

### 4.2 System Execution Order

Systems execute in fixed order for determinism. The order is:

| Order | System | Components Read | Components Written | Dependencies |
|-------|--------|----------------|-------------------|--------------|
| 1 | PolicySystem | Institution, Policy | Policy, Institution | None |
| 2 | EconomySystem | Wealth, Market, Position | Wealth, Market | PolicySystem |
| 3 | HealthSystem | Health, Age, Wealth | Health | EconomySystem |
| 4 | SocialSystem | Ideology, SocialNetwork | Ideology, SocialNetwork | HealthSystem |
| 5 | ClimateSystem | Climate, Position | Climate, WeatherEvent | SocialSystem |
| 6 | MovementSystem | Position, Velocity | Position | ClimateSystem |
| 7 | ConflictSystem | MilitaryUnit, Position | MilitaryUnit, Health | MovementSystem |
| 8 | DiplomacySystem | Faction, DiplomaticRelation | DiplomaticRelation | ConflictSystem |
| 9 | MetricSystem | All | TimeSeries | None (read-only) |

### 4.3 Memory Architecture

ECS uses archetype-based storage (Structure of Arrays layout). Each unique component combination forms an archetype. Components of the same type are stored contiguously in memory, enabling cache-friendly iteration.

```
Archetype Table: [Position, Velocity, Health]
┌─────────┬─────────┬─────────┐
│ Position│ Velocity│ Health  │  ← SoA layout
├─────────┼─────────┼─────────┤
│ [x,y]   │ [dx,dy] │ [cur,mx]│  ← Contiguous per component
│ [x,y]   │ [dx,dy] │ [cur,mx]│
│ [x,y]   │ [dx,dy] │ [cur,mx]│
└─────────┴─────────┴─────────┘
```

### 4.4 Concurrency Model

System parallelism is determined by component access patterns. Systems that do not share mutable components can run in parallel. However, for Civis determinism guarantees, systems MUST execute in explicit order even if they could theoretically run in parallel.

### 4.5 Hot Path Design

Hot path code (executed every tick for every entity) must be optimized for cache efficiency and minimal allocation. Use stack allocation, avoid heap allocation, and prefer inline functions.

```rust
// GOOD: Stack-allocated, no heap allocation
#[inline]
fn process_agent(pos: &mut Position, vel: &Velocity) {
    pos.x += vel.dx as i32;
    pos.y += vel.dy as i32;
}

// BAD: Heap allocation in hot path
fn process_agent_bad(world: &mut World) {
    let agents: Vec<_> = world.query::<&Position>().iter()
        .map(|(e, p)| (e, p.clone()))
        .collect();
    // Heap allocation every tick
}
```

---

## 5. API Reference

### 5.1 REST API Endpoints

#### 5.1.1 Health & Status

| Endpoint | Method | Auth | Request | Response | Status Codes |
|----------|--------|------|---------|----------|--------------|
| `/api/v1/health` | GET | None | - | `{"status": "ok", "uptime_ms": 12345}` | 200 |
| `/api/v1/version` | GET | None | - | `{"version": "0.1.0", "commit": "abc123"}` | 200 |
| `/api/v1/metrics` | GET | Optional | - | `MetricsSnapshot` | 200, 500 |

#### 5.1.2 Scenario Management

| Endpoint | Method | Auth | Request | Response | Status Codes |
|----------|--------|------|---------|----------|--------------|
| `/api/v1/scenarios` | GET | None | - | `ScenarioList` | 200 |
| `/api/v1/scenarios/:id` | GET | None | - | `Scenario` | 200, 404 |
| `/api/v1/scenarios/:id/start` | POST | Optional | `{"seed": 12345}` | `{"simulation_id": "sim_abc"}` | 201, 400, 404 |
| `/api/v1/scenarios/:id/validate` | POST | None | Scenario YAML | `{"valid": true, "errors": []}` | 200, 400 |

#### 5.1.3 Simulation Control

| Endpoint | Method | Auth | Request | Response | Status Codes |
|----------|--------|------|---------|----------|--------------|
| `/api/v1/simulations` | GET | Optional | - | `SimulationList` | 200 |
| `/api/v1/simulations/:id` | GET | Optional | - | `SimulationState` | 200, 404 |
| `/api/v1/simulations/:id` | DELETE | Optional | - | `{"success": true}` | 200, 404 |
| `/api/v1/simulations/:id/pause` | POST | Optional | - | `{"status": "paused"}` | 200, 404 |
| `/api/v1/simulations/:id/resume` | POST | Optional | - | `{"status": "running"}` | 200, 404 |
| `/api/v1/simulations/:id/speed` | POST | Optional | `{"speed": 10}` | `{"speed": 10}` | 200, 400 |
| `/api/v1/simulations/:id/step` | POST | Optional | - | `TickResult` | 200, 404 |

#### 5.1.4 Metrics & Data

| Endpoint | Method | Auth | Request | Response | Status Codes |
|----------|--------|------|---------|----------|--------------|
| `/api/v1/simulations/:id/metrics` | GET | Optional | `?start=0&end=1000` | `MetricsSnapshot[]` | 200, 404 |
| `/api/v1/simulations/:id/metrics/export` | GET | Optional | `?format=csv` | CSV/JSON/Parquet | 200, 400, 404 |
| `/api/v1/simulations/:id/metrics/:metric` | GET | Optional | `?start=0&end=1000` | `TimeSeries<T>` | 200, 404 |

#### 5.1.5 Replay & Verification

| Endpoint | Method | Auth | Request | Response | Status Codes |
|----------|--------|------|---------|----------|--------------|
| `/api/v1/simulations/:id/replay` | POST | Optional | `{"from_tick": 0}` | Replay stream | 200, 404 |
| `/api/v1/simulations/:id/verify` | POST | Optional | - | `{"verified": true}` | 200, 404, 409 |
| `/api/v1/simulations/:id/checkpoint` | POST | Optional | - | `{"checkpoint_id": "chk_abc"}` | 201, 404 |
| `/api/v1/simulations/:id/restore` | POST | Optional | `{"checkpoint_id": "chk_abc"}` | `{"success": true}` | 200, 404 |

#### 5.1.6 Entity Queries

| Endpoint | Method | Auth | Request | Response | Status Codes |
|----------|--------|------|---------|----------|--------------|
| `/api/v1/simulations/:id/agents/:agent_id` | GET | Optional | - | `AgentDetails` | 200, 404 |
| `/api/v1/simulations/:id/factions` | GET | Optional | - | `FactionList` | 200, 404 |
| `/api/v1/simulations/:id/factions/:id` | GET | Optional | - | `Faction` | 200, 404 |
| `/api/v1/simulations/:id/policies` | GET | Optional | - | `PolicyList` | 200, 404 |
| `/api/v1/simulations/:id/events` | GET | Optional | `?type=Trade&start=0` | `EventList` | 200, 404 |

### 5.2 WebSocket Protocol

#### 5.2.1 Connection Lifecycle

1. Client opens WebSocket connection to server
2. Server accepts connection
3. Client sends subscribe command with room (simulation ID)
4. Server confirms subscription
5. Server streams tick updates to client
6. Client can send control commands (pause, resume, step, set_speed)
7. Client unsubscribes and closes connection

#### 5.2.2 Client Message Types

```json
// Subscribe to simulation
{"command": "subscribe", "room": "sim_abc123", "auth_token": "optional"}

// Unsubscribe
{"command": "unsubscribe", "room": "sim_abc123"}

// Control simulation
{"command": "control", "action": "pause", "simulation_id": "sim_abc123"}
{"command": "control", "action": "resume", "simulation_id": "sim_abc123"}
{"command": "control", "action": "step", "simulation_id": "sim_abc123"}
{"command": "control", "action": "set_speed", "simulation_id": "sim_abc123", "speed": 10}

// Query state
{"command": "query", "type": "agent", "id": "agent_123", "simulation_id": "sim_abc123"}
```

#### 5.2.3 Server Message Types

```json
// Tick update (streamed every tick)
{
  "type": "tick",
  "tick": 12345,
  "timestamp": 1712234567890,
  "snapshot": {
    "population": 10000,
    "gdp": 5000000.0,
    "gini_coefficient": 0.35,
    "avg_ideology_economic": 0.2,
    "health_index": 0.85,
    "conflict_score": 0.12,
    "carbon_emissions": 1500.5
  },
  "events": [
    {"type": "Trade", "data": {"buyer": "citizen_1234", "seller": "citizen_5678", "good": "food", "quantity": 10.5, "price": 50.0}}
  ],
  "checksum": "a1b2c3d4e5f6"
}

// Control response
{"type": "control_response", "action": "pause", "status": "success", "simulation_id": "sim_abc123"}

// Error
{"type": "error", "code": "SIMULATION_NOT_FOUND", "message": "Simulation sim_abc123 does not exist"}

// Simulation completed
{"type": "simulation_completed", "simulation_id": "sim_abc123", "final_tick": 100000, "final_metrics": {}, "checksum": "final"}
```

### 5.3 Internal API (crates/engine)

```rust
pub struct Simulation {
    pub tick: u64,
    pub seed: u64,
    pub world: World,
    pub market: Market,
    pub climate: ClimateSystem,
    pub systems: Vec<Box<dyn System>>,
    pub replay: Option<ReplayRecorder>,
}

impl Simulation {
    pub fn new(seed: u64, config: SimulationConfig) -> Self;
    pub fn step(&mut self) -> Result<TickResult, SimulationError>;
    pub fn run(&mut self) -> Result<RunResult, SimulationError>;
    pub fn pause(&mut self);
    pub fn resume(&mut self);
    pub fn snapshot(&self) -> SimulationSnapshot;
    pub fn checkpoint(&self) -> Result<Vec<u8>, SerializationError>;
    pub fn restore(&mut self, data: &[u8]) -> Result<(), SerializationError>;
    pub fn verify_replay(&self, replay: &Replay) -> Result<VerificationResult, ReplayError>;
    pub fn metrics(&self) -> &TimeSeries<MetricsSnapshot>;
    pub fn current_tick(&self) -> u64;
    pub fn status(&self) -> SimulationStatus;
}

pub struct TickResult {
    pub tick: u64,
    pub events: Vec<SimulationEvent>,
    pub metrics: MetricsSnapshot,
    pub checksum: u64,
}

pub struct RunResult {
    pub total_ticks: u64,
    pub final_metrics: MetricsSnapshot,
    pub final_checksum: u64,
    pub events: Vec<SimulationEvent>,
    pub duration_ms: u64,
}
```

---

## 6. Error Handling

### 6.1 Error Taxonomy

```rust
#[derive(Debug, thiserror::Error)]
pub enum CivisError {
    #[error("Simulation error: {0}")]
    Simulation(String),

    #[error("Determinism violation at tick {tick}: {reason}")]
    DeterminismViolation { tick: u64, reason: String },

    #[error("Simulation not found: {id}")]
    SimulationNotFound { id: String },

    #[error("Simulation already {status}: {id}")]
    InvalidState { status: String, id: String },

    #[error("Replay error: {0}")]
    Replay(#[from] ReplayError),

    #[error("Checksum mismatch at tick {tick}: expected {expected}, got {actual}")]
    ChecksumMismatch { tick: u64, expected: u64, actual: u64 },

    #[error("Serialization error: {0}")]
    Serialization(#[from] bincode::Error),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Invalid scenario: {0}")]
    InvalidScenario(String),

    #[error("Scenario validation failed: {errors:?}")]
    ScenarioValidation { errors: Vec<ValidationError> },

    #[error("API error: {status} - {message}")]
    ApiError { status: u16, message: String },

    #[error("Rate limit exceeded: retry after {retry_after}s")]
    RateLimitExceeded { retry_after: u64 },

    #[error("Economic overflow: {good} exceeded maximum at tick {tick}")]
    EconomicOverflow { good: String, tick: u64 },

    #[error("Market clearing failed: {reason}")]
    MarketClearingFailed { reason: String },

    #[error("Policy error: {0}")]
    Policy(String),

    #[error("Invalid policy instrument: {instrument}")]
    InvalidPolicyInstrument { instrument: String },

    #[error("Climate error: {0}")]
    Climate(String),

    #[error("Position out of bounds: ({x}, {y})")]
    PositionOutOfBounds { x: i32, y: i32 },

    #[error("Path not found: ({start_x}, {start_y}) to ({goal_x}, {goal_y})")]
    PathNotFound { start_x: i32, start_y: i32, goal_x: i32, goal_y: i32 },
}

#[derive(Debug, thiserror::Error)]
pub enum ReplayError {
    #[error("Replay not enabled for this simulation")]
    NotEnabled,

    #[error("Replay data corrupted at tick {tick}")]
    Corrupted { tick: u64 },

    #[error("Replay verification failed: {mismatches} checksum mismatches")]
    VerificationFailed { mismatches: u64 },

    #[error("Replay incomplete: expected {expected} ticks, got {actual}")]
    Incomplete { expected: u64, actual: u64 },
}

#[derive(Debug, thiserror::Error)]
pub enum ValidationError {
    #[error("Missing required field: {field}")]
    MissingField { field: String },

    #[error("Invalid value for {field}: {value}")]
    InvalidValue { field: String, value: String },

    #[error("Value out of range for {field}: {value} (min: {min}, max: {max})")]
    OutOfRange { field: String, value: String, min: String, max: String },

    #[error("Unseeded randomness detected in {location}")]
    UnseededRandomness { location: String },

    #[error("External dependency detected: {dependency}")]
    ExternalDependency { dependency: String },
}
```

### 6.2 Error Response Format

All API errors follow a consistent JSON format:

```json
{
  "error": {
    "code": "CHECKSUM_MISMATCH",
    "message": "Checksum mismatch at tick 1234: expected 1234567890, got 9876543210",
    "tick": 1234,
    "expected_checksum": "1234567890",
    "actual_checksum": "9876543210",
    "simulation_id": "sim_abc123",
    "timestamp": "2026-04-03T12:00:00Z"
  }
}
```

### 6.3 Error Handling Strategy

| Error Type | Handling | Recovery | Logging |
|------------|----------|----------|---------|
| DeterminismViolation | Panic | None | CRITICAL + alert |
| ChecksumMismatch | Return error | Restore checkpoint | ERROR |
| InvalidScenario | Return error | Fix scenario | WARN |
| EconomicOverflow | Return error | Clamp values | ERROR |
| PathNotFound | Return error | Use fallback path | WARN |
| RateLimitExceeded | Return 429 | Retry after delay | INFO |
| IoError | Return error | Retry with backoff | ERROR |
| SimulationNotFound | Return 404 | N/A | WARN |

### 6.4 Invariant Checking

Core invariants must hold at all times. Violations are caught after each tick:

- **Population non-negative**: Population count must never be negative
- **Energy conservation**: Total energy cannot exceed initial energy
- **Currency non-negative**: All agent wealth values must be non-negative
- **RNG state integrity**: RNG state must remain valid and deterministic
- **World state checksum**: Checksum must match expected value

```rust
pub struct Invariants {
    pub checks: Vec<InvariantCheck>,
}

pub enum InvariantSeverity {
    Warn,   // Log warning, continue
    Error,  // Return error, stop tick
    Panic,  // Panic immediately
}
```

---

## 7. Security

### 7.1 Threat Model

Attack surfaces include:

- **External to API (REST/WebSocket)**: Injection attacks via malicious scenario YAML, denial of service via resource exhaustion, replay manipulation with tampered event logs, unauthorized access without auth tokens
- **External to Engine (direct library use)**: Memory safety (mitigated by Rust), determinism attacks via non-deterministic inputs, resource exhaustion via unbounded scenarios
- **Internal to Engine (modding/WASM)**: Sandbox escape via WASM breakout, resource exhaustion via infinite loops, state corruption via invalid memory access

Assets to protect: simulation state integrity, determinism guarantees, replay authenticity, API availability, client data privacy.

### 7.2 Security Controls

| Control | Implementation | Coverage |
|---------|---------------|----------|
| **Input Validation** | Scenario YAML schema validation, type checking | All external inputs |
| **Rate Limiting** | Token bucket algorithm per IP/client | REST + WebSocket APIs |
| **Authentication** | Optional JWT tokens for API access | Protected endpoints |
| **Sandboxing** | WASM runtime with resource limits | Mod execution |
| **Checksum Chain** | XXH3 checksum every N ticks | State integrity |
| **Memory Safety** | Rust ownership + borrow checker | All Rust code |
| **Resource Limits** | Max tick limit, max entity count, max memory | Simulation config |
| **Audit Logging** | Structured tracing for all operations | Server operations |

### 7.3 Scenario Validation

Scenarios must be validated before execution. Validation checks:

- Required fields are present (seed, tick_limit)
- Seed is non-zero for determinism
- Tick limit is positive and within bounds
- No unseeded randomness in events
- No external API dependencies
- All policy instruments are valid types
- Faction population shares sum to 1.0
- Territory bounds are within map dimensions
- Climate zones are from predefined list
- Government types are from GovernmentType enum

### 7.4 WASM Sandbox Security

Mods execute in a WASM sandbox with strict resource limits:

- Memory limit: 100MB maximum
- Fuel limit: 1,000,000 instructions maximum
- Single memory (no multi-memory)
- Host function memory access validation
- No direct file system access
- No network access from within WASM

```rust
pub struct ModSandbox {
    engine: Engine,
    store_limits: StoreLimits,
}

impl ModSandbox {
    pub fn execute_mod(&self, wasm_bytes: &[u8]) -> Result<(), ModError> {
        // Compile, instantiate, and execute with resource limits
    }
}
```

### 7.5 API Security

Rate limiting uses a token bucket algorithm per client. Authentication is optional via JWT tokens. All API responses include appropriate HTTP status codes. Protected endpoints require valid auth tokens.

### 7.6 Determinism Attack Prevention

A DeterminismGuard validates that scenarios do not use banned functions (SystemTime, Instant, thread_rng, random) and that all RNG sources are approved and seeded.

---

## 8. Data Models

### 8.1 Core Simulation State

```rust
pub struct Simulation {
    pub tick: u64,
    pub seed: u64,
    pub rng: ChaCha8Rng,
    pub world: World,
    pub market: Market,
    pub climate: ClimateSystem,
    pub systems: Vec<Box<dyn System>>,
    pub replay: Option<ReplayRecorder>,
    pub metrics: TimeSeries<MetricsSnapshot>,
    pub status: SimulationStatus,
    pub invariants: Invariants,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum SimulationStatus {
    Created,
    Running,
    Paused,
    Completed,
    Error(String),
}

pub type Tick = u64;
pub type Currency = fixed::types::I64F64;
pub type RegionId = u32;
pub type CitizenId = u64;
pub type UnitId = u64;
pub type FactionId = u64;
pub type InstitutionId = u64;
pub type PolicyId = u64;
```

### 8.2 Economy Data Models

```rust
pub struct Market {
    pub prices: IndexMap<GoodId, Currency>,
    pub supply: IndexMap<GoodId, f64>,
    pub demand: IndexMap<GoodId, f64>,
    pub trade_routes: Vec<TradeRoute>,
    pub history: VecDeque<PriceRecord>,
    pub order_book: Vec<Order>,
}

pub struct TradeRoute {
    pub from: RegionId,
    pub to: RegionId,
    pub goods: Vec<GoodId>,
    pub capacity: f64,
    pub cost: Currency,
}

pub struct Order {
    pub agent: Entity,
    pub good: GoodId,
    pub quantity: f64,
    pub price_limit: Currency,
    pub side: OrderSide,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum OrderSide { Buy, Sell }

pub struct TradeRecord {
    pub tick: Tick,
    pub buyer: Entity,
    pub seller: Entity,
    pub good: GoodId,
    pub quantity: f64,
    pub price: Currency,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum GoodId {
    Food, Wood, Stone, Metal, Tools, Weapons, Luxury, Energy, Currency,
}
```

### 8.3 Actor Data Models

```rust
#[derive(Clone, Debug)]
pub struct Citizen {
    pub id: CitizenId,
    pub birth_tick: Tick,
    pub generation: u32,
    pub family_id: Option<u64>,
}

#[derive(Clone, Debug)]
pub struct Needs {
    pub hunger: f32,
    pub health: f32,
    pub safety: f32,
    pub belonging: f32,
    pub esteem: f32,
    pub actualization: f32,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum EmploymentStatus {
    Unemployed,
    Employed { employer: Entity, wage: Currency },
    SelfEmployed { business: Entity },
    Retired,
    Student,
}

#[derive(Clone, Debug)]
pub struct MilitaryUnit {
    pub id: UnitId,
    pub unit_type: UnitType,
    pub strength: u32,
    pub morale: f32,
    pub fatigue: f32,
    pub experience: f32,
    pub faction: Entity,
    pub commander: Option<Entity>,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum UnitType { Infantry, Cavalry, Archers, Siege, Navy }

#[derive(Clone, Debug)]
pub struct Faction {
    pub id: FactionId,
    pub name: String,
    pub government: GovernmentType,
    pub territory: Vec<RegionId>,
    pub population: u32,
    pub treasury: Currency,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum GovernmentType {
    Tribal, Monarchy, Republic, Democracy, Theocracy, Oligarchy, Anarchy,
}
```

### 8.4 Policy Data Models

```rust
#[derive(Clone, Debug)]
pub struct Institution {
    pub id: InstitutionId,
    pub name: String,
    pub level: GovernanceLevel,
    pub institution_type: InstitutionType,
    pub jurisdiction: Jurisdiction,
    pub budget: Currency,
    pub legitimacy: f32,
    pub corruption: f32,
    pub members: Vec<Entity>,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum GovernanceLevel { Local, Regional, National, Global }

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum InstitutionType {
    Government, Legislature, Judiciary, Military, CentralBank,
    RegulatoryAgency, InternationalOrg,
}

#[derive(Clone, Debug)]
pub enum Jurisdiction {
    Global,
    Faction(FactionId),
    Region(RegionId),
    Tile(Position),
}

#[derive(Clone, Debug)]
pub struct Policy {
    pub id: PolicyId,
    pub name: String,
    pub domain: PolicyDomain,
    pub instruments: Vec<PolicyInstrument>,
    pub enacting_institution: Entity,
    pub effective_date: Tick,
    pub expiration_date: Option<Tick>,
    pub compliance_rate: f32,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum PolicyDomain {
    Economic, Social, Environmental, Military, Diplomatic, Cultural,
}

#[derive(Clone, Debug)]
pub struct PolicyInstrument {
    pub name: String,
    pub instrument_type: InstrumentType,
    pub parameters: IndexMap<String, f64>,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum InstrumentType {
    Taxation, Subsidy, Regulation, TradeAgreement, MilitaryDraft,
    Rationing, Propaganda, Sanction,
}

#[derive(Clone, Debug)]
pub struct DiplomaticRelation {
    pub faction_a: Entity,
    pub faction_b: Entity,
    pub sentiment: f32,
    pub trust: f32,
    pub trade_volume: f64,
    pub pact_type: Option<PactType>,
    pub war_status: WarStatus,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum PactType {
    NonAggression, Defense, MutualDefense, TradeAgreement,
    ResearchAgreement, OpenBorders,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum WarStatus { Peace, Tension, War, Ceasefire }

#[derive(Clone, Debug)]
pub struct ShadowNetwork {
    pub members: Vec<Entity>,
    pub influence: f32,
    pub detection_risk: f32,
    pub covert_actions: Vec<CovertAction>,
}
```

### 8.5 Climate Data Models

```rust
pub struct ClimateSystem {
    pub global_temperature: f32,
    pub precipitation: f32,
    pub season: Season,
    pub year: u32,
    pub events: Vec<ClimateEvent>,
    pub regional_climates: IndexMap<RegionId, RegionalClimate>,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Season { Spring, Summer, Autumn, Winter }

#[derive(Clone, Debug)]
pub struct ClimateEvent {
    pub event_type: DisasterType,
    pub affected_regions: Vec<RegionId>,
    pub start_tick: Tick,
    pub duration: u32,
    pub severity: f32,
    pub casualties: u32,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum DisasterType {
    Earthquake, Tsunami, Volcano, Hurricane, Tornado,
    Wildfire, Plague, Famine, Drought, Flood,
}

#[derive(Clone, Debug)]
pub struct RegionalClimate {
    pub region_id: RegionId,
    pub base_temperature: f32,
    pub precipitation_pattern: Vec<f32>,
    pub climate_zone: ClimateZone,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum ClimateZone {
    Polar, Tundra, Taiga, Temperate, Mediterranean, Arid, Tropical,
}
```

### 8.6 Spatial Data Models

```rust
pub struct TerrainMap {
    pub width: u32,
    pub height: u32,
    pub tiles: Vec<Tile>,
    pub spatial_index: SpatialGrid<Entity>,
}

#[derive(Clone, Debug)]
pub struct Tile {
    pub terrain: TerrainType,
    pub elevation: i16,
    pub water_level: f32,
    pub fertility: f32,
    pub resources: Vec<ResourceDeposit>,
    pub improvement: Option<TileImprovement>,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum TerrainType {
    Ocean, Coast, Plains, Grassland, Forest, Jungle,
    Hills, Mountains, Desert, Tundra, Ice,
}

#[derive(Clone, Debug)]
pub struct TileImprovement {
    pub improvement_type: ImprovementType,
    pub owner: Entity,
    pub build_tick: Tick,
    pub health: f32,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum ImprovementType { Farm, Mine, Road, Fort, City, Port, Dam }

#[derive(Clone, Debug)]
pub struct ResourceDeposit {
    pub resource_type: ResourceType,
    pub quantity: f64,
    pub extraction_rate: f64,
    pub difficulty: f32,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum ResourceType {
    Food, Wood, Stone, Ore, Coal, Oil, Gas, Uranium, Water, RareEarth,
}

#[derive(Clone, Debug)]
pub struct Region {
    pub id: RegionId,
    pub name: String,
    pub bounds: BoundingBox,
    pub tiles: Vec<Position>,
    pub climate: RegionalClimate,
    pub population: u32,
    pub controlling_faction: Option<Entity>,
}

#[derive(Clone, Copy, Debug)]
pub struct BoundingBox {
    pub min_x: i32, pub min_y: i32, pub max_x: i32, pub max_y: i32,
}
```

### 8.7 Social Data Models

```rust
#[derive(Clone, Copy, Debug)]
pub struct Ideology {
    pub economic: f32,
    pub social: f32,
    pub foreign: f32,
    pub authority: f32,
    pub tradition: f32,
}

#[derive(Clone, Debug, Default)]
pub struct SocialNetwork {
    pub ties: Vec<SocialTie>,
    pub last_updated: Tick,
}

#[derive(Clone, Debug)]
pub struct SocialTie {
    pub target: Entity,
    pub strength: f32,
    pub tie_type: TieType,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum TieType { Family, Friend, Colleague, Acquaintance, Enemy, Rival }

#[derive(Clone, Debug)]
pub struct Culture {
    pub culture_id: u64,
    pub traits: Vec<CulturalTrait>,
    pub language: u64,
    pub religion: Option<u64>,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum CulturalTrait {
    Individualistic, Collectivist, Egalitarian, Hierarchical,
    Universalist, Particularist, UncertaintyAvoidant,
    LongTermOriented, Indulgent,
}
```

### 8.8 Metrics Data Models

```rust
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct MetricsSnapshot {
    pub tick: Tick,
    pub timestamp: u64,
    pub population: u32,
    pub birth_count: u32,
    pub death_count: u32,
    pub average_age: f32,
    pub gdp: f64,
    pub gini_coefficient: f32,
    pub unemployment_rate: f32,
    pub inflation_rate: f32,
    pub avg_ideology_economic: f32,
    pub avg_ideology_social: f32,
    pub social_mobility: f32,
    pub health_index: f32,
    pub life_expectancy: f32,
    pub disease_prevalence: f32,
    pub conflict_score: f32,
    pub war_count: u32,
    pub refugee_count: u32,
    pub carbon_emissions: f64,
    pub forest_coverage: f32,
    pub biodiversity_index: f32,
    pub avg_legitimacy: f32,
    pub avg_corruption: f32,
    pub policy_count: u32,
}

pub struct TimeSeries<T> {
    pub data: Vec<(Tick, T)>,
    pub compression: CompressionConfig,
}

impl<T: Copy> TimeSeries<T> {
    pub fn query_range(&self, start: Tick, end: Tick) -> &[(Tick, T)] {
        let start_idx = self.data.partition_point(|(t, _)| *t < start);
        let end_idx = self.data.partition_point(|(t, _)| *t <= end);
        &self.data[start_idx..end_idx]
    }

    pub fn moving_average(&self, window: usize) -> Vec<f64> {
        let mut result = Vec::with_capacity(self.data.len());
        let mut sum = 0.0;
        let mut count = 0;
        for (i, (_, value)) in self.data.iter().enumerate() {
            sum += *value as f64;
            count += 1;
            if i >= window {
                sum -= self.data[i - window].1 as f64;
                count -= 1;
            }
            result.push(sum / count as f64);
        }
        result
    }

    pub fn exponential_smooth(&self, alpha: f32) -> Vec<f64> {
        let mut result = Vec::with_capacity(self.data.len());
        let mut ema = self.data[0].1 as f64;
        result.push(ema);
        for (_, value) in self.data.iter().skip(1) {
            ema = alpha as f64 * (*value as f64) + (1.0 - alpha as f64) * ema;
            result.push(ema);
        }
        result
    }
}

#[derive(Clone, Copy, Debug)]
pub struct CompressionConfig {
    pub enabled: bool,
    pub interval: u64,
    pub method: CompressionMethod,
}

#[derive(Clone, Copy, Debug)]
pub enum CompressionMethod { None, Downsampling, DeltaEncoding }

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum ExportFormat { Csv, Json, Parquet, Arrow }
```

---

## 9. ECS Component Registry

### 9.1 Complete Component List

| Component | Fields | Size | Archetype Group |
|-----------|--------|------|-----------------|
| `Citizen` | (marker) | 0 bytes | Actors |
| `Position` | x: i32, y: i32 | 8 bytes | Spatial |
| `Velocity` | dx: f32, dy: f32 | 8 bytes | Spatial |
| `Health` | current: f32, max: f32, age: u32 | 12 bytes | Actors |
| `Wealth` | amount, income, expenses (Currency) | 24 bytes | Economy |
| `Needs` | hunger, health, safety, belonging, esteem, actualization | 24 bytes | Actors |
| `Ideology` | economic, social, foreign, authority, tradition | 20 bytes | Social |
| `SocialNetwork` | ties: Vec, last_updated: Tick | variable | Social |
| `MilitaryUnit` | id, type, strength, morale, fatigue, experience, faction, commander | 36 bytes | Conflict |
| `Faction` | id, name, government, territory, population, treasury | variable | Policy |
| `Institution` | id, name, level, type, jurisdiction, budget, legitimacy, corruption, members | variable | Policy |
| `Policy` | id, name, domain, instruments, enacting_institution, dates, compliance | variable | Policy |
| `DiplomaticRelation` | faction_a, faction_b, sentiment, trust, trade_volume, pact_type, war_status | 28 bytes | Policy |
| `ShadowNetwork` | members, influence, detection_risk, covert_actions | variable | Policy |
| `Climate` | temperature, precipitation, season, year | 16 bytes | Environment |
| `WeatherEvent` | type, location, start_tick, duration, severity | 20 bytes | Environment |
| `Tile` | terrain, elevation, water_level, fertility, resources, improvement | variable | Spatial |
| `Region` | id, name, bounds, tiles, climate, population, controlling_faction | variable | Spatial |
| `Culture` | culture_id, traits, language, religion | variable | Social |
| `EmploymentStatus` | enum variant | 16 bytes | Economy |
| `PendingEvent` | event_type, data, priority | variable | Core |

### 9.2 Component Registration

All components must be registered with the ECS world before use. Registration enables the ECS to track component storage and enable efficient queries.

```rust
pub fn register_components(world: &mut World) {
    // Actor components
    world.register_component::<Citizen>();
    world.register_component::<Position>();
    world.register_component::<Velocity>();
    world.register_component::<Health>();
    world.register_component::<Needs>();
    world.register_component::<EmploymentStatus>();

    // Economy components
    world.register_component::<Wealth>();

    // Social components
    world.register_component::<Ideology>();
    world.register_component::<SocialNetwork>();
    world.register_component::<Culture>();

    // Policy components
    world.register_component::<Faction>();
    world.register_component::<Institution>();
    world.register_component::<Policy>();
    world.register_component::<DiplomaticRelation>();
    world.register_component::<ShadowNetwork>();

    // Conflict components
    world.register_component::<MilitaryUnit>();

    // Environment components
    world.register_component::<Climate>();
    world.register_component::<WeatherEvent>();

    // Spatial components
    world.register_component::<Tile>();
    world.register_component::<Region>();

    // Core components
    world.register_component::<PendingEvent>();
}
```

---

## 10. System Execution Pipeline

### 10.1 System Registry

| System | Order | Reads | Writes | Parallelizable |
|--------|-------|-------|--------|----------------|
| `PolicySystem` | 1 | Institution, Policy | Policy, Institution | No (first) |
| `EconomySystem` | 2 | Wealth, Market, Position | Wealth, Market | No (after Policy) |
| `HealthSystem` | 3 | Health, Age, Wealth | Health | No (after Economy) |
| `SocialSystem` | 4 | Ideology, SocialNetwork | Ideology, SocialNetwork | No (after Health) |
| `ClimateSystem` | 5 | Climate, Position | Climate, WeatherEvent | No (after Social) |
| `MovementSystem` | 6 | Position, Velocity | Position | No (after Climate) |
| `ConflictSystem` | 7 | MilitaryUnit, Position | MilitaryUnit, Health | No (after Movement) |
| `DiplomacySystem` | 8 | Faction, DiplomaticRelation | DiplomaticRelation | No (after Conflict) |
| `MetricSystem` | 9 | All | TimeSeries | Yes (read-only) |

### 10.2 System Implementation Pattern

```rust
pub trait System {
    fn name(&self) -> &'static str;
    fn execute(&self, world: &mut World, rng: &mut ChaCha8Rng, tick: Tick);
    fn dependencies(&self) -> Vec<&'static str>;
}

pub struct SystemRegistry {
    systems: Vec<Box<dyn System>>,
    order: Vec<usize>,
}

impl SystemRegistry {
    pub fn new() -> Self {
        Self { systems: Vec::new(), order: Vec::new() }
    }

    pub fn register(&mut self, system: Box<dyn System>) {
        let idx = self.systems.len();
        self.systems.push(system);
        self.order.push(idx);
    }

    pub fn execute_all(&mut self, world: &mut World, rng: &mut ChaCha8Rng, tick: Tick) {
        for &idx in &self.order {
            self.systems[idx].execute(world, rng, tick);
        }
    }

    pub fn sort_by_dependencies(&mut self) {
        // Topological sort based on dependencies
    }
}
```

### 10.3 Tick Execution Flow

```rust
impl Simulation {
    pub fn step(&mut self) -> Result<TickResult, CivisError> {
        // Pre-tick validation
        self.validate_tick()?;

        // Record pre-tick checksum
        let pre_checksum = self.compute_checksum();

        // Execute systems in order
        self.systems.execute_all(&mut self.world, &mut self.rng, self.tick);

        // Post-tick processing
        let post_checksum = self.compute_checksum();
        let metrics = self.collect_metrics();
        let events = self.collect_events();

        // Record to replay
        if let Some(replay) = &mut self.replay {
            replay.record(self.tick, &events, post_checksum);
        }

        // Record metrics
        self.metrics.push(self.tick, metrics.clone());

        // Check invariants
        self.check_invariants()?;

        // Increment tick
        self.tick += 1;

        Ok(TickResult {
            tick: self.tick - 1,
            events,
            metrics,
            checksum: post_checksum,
        })
    }
}
```

---

## 11. Determinism Architecture

### 11.1 Determinism Stack

Following ADR-001 (Deterministic Simulation Architecture):

```
Layer 5: Replay System
  -- Event log, checksums, verification

Layer 4: Simulation Logic
  -- No system time, no thread dependence, no external state

Layer 3: Data Structures
  -- IndexMap, BTreeMap, stable sort, deterministic iteration

Layer 2: Arithmetic
  -- Fixed-point for economics (I64F64), strict IEEE-754

Layer 1: Randomness
  -- ChaCha8Rng with seed, parallel streams via set_stream()
```

### 11.2 RNG Configuration

```rust
use rand::SeedableRng;
use rand_chacha::ChaCha8Rng;

pub struct SimRng {
    rng: ChaCha8Rng,
    seed: u64,
}

impl SimRng {
    pub fn new(seed: u64) -> Self {
        Self {
            rng: ChaCha8Rng::seed_from_u64(seed),
            seed,
        }
    }

    pub fn jump(&mut self, stream_id: u64) -> ChaCha8Rng {
        let mut stream = self.rng.clone();
        stream.set_stream(stream_id);
        stream
    }

    pub fn checkpoint(&self) -> Vec<u8> {
        bincode::serialize(&self.rng).unwrap()
    }

    pub fn restore(&mut self, data: &[u8]) -> Result<(), bincode::Error> {
        self.rng = bincode::deserialize(data)?;
        Ok(())
    }
}
```

### 11.3 Fixed-Point Arithmetic

```rust
use fixed::types::I64F64;

pub type Currency = I64F64;

impl Currency {
    pub const ZERO: Self = I64F64::ZERO;
    pub const ONE: Self = I64F64::ONE;
    pub const MAX: Self = I64F64::MAX;
    pub const MIN: Self = I64F64::MIN;

    pub fn from_f64(v: f64) -> Self { I64F64::from_num(v) }
    pub fn to_f64(self) -> f64 { self.to_num() }
    pub fn from_i64(v: i64) -> Self { I64F64::from_num(v) }
    pub fn to_i64(self) -> i64 { self.to_num() }
}

pub fn currency_eq(a: Currency, b: Currency) -> bool { a == b }
pub fn currency_cmp(a: Currency, b: Currency) -> std::cmp::Ordering { a.cmp(&b) }
```

### 11.4 Ordered Collections

```rust
use indexmap::IndexMap;

pub struct AgentStorage<T> {
    agents: IndexMap<Entity, T>,
}

impl<T> AgentStorage<T> {
    pub fn new() -> Self { Self { agents: IndexMap::new() } }
    pub fn insert(&mut self, entity: Entity, agent: T) {
        self.agents.insert(entity, agent);
    }
    pub fn iter(&self) -> impl Iterator<Item = (&Entity, &T)> {
        self.agents.iter()
    }
    pub fn iter_sorted(&self) -> Vec<(&Entity, &T)> {
        let mut items: Vec<_> = self.agents.iter().collect();
        items.sort_by_key(|(e, _)| e.id());
        items
    }
    pub fn get(&self, entity: &Entity) -> Option<&T> { self.agents.get(entity) }
    pub fn get_mut(&mut self, entity: &Entity) -> Option<&mut T> {
        self.agents.get_mut(entity)
    }
    pub fn len(&self) -> usize { self.agents.len() }
    pub fn is_empty(&self) -> bool { self.agents.is_empty() }
}
```

### 11.5 Replay System

```rust
pub struct Replay {
    pub seed: u64,
    pub initial_state: Vec<u8>,
    pub events: Vec<LoggedEvent>,
    pub checksums: Vec<(Tick, u64)>,
}

pub struct LoggedEvent {
    pub tick: Tick,
    pub event_type: u64,
    pub data: Vec<u8>,
}

impl Replay {
    pub fn record(&mut self, tick: Tick, event: &dyn SimulationEvent, checksum: u64) {
        self.events.push(LoggedEvent {
            tick,
            event_type: event.type_id(),
            data: event.serialize(),
        });
        self.checksums.push((tick, checksum));
    }

    pub fn compute_checksum(world: &World) -> u64 {
        let serialized = bincode::serialize(world).unwrap();
        xxhash_rust::xxh3::xxh3_64(&serialized)
    }

    pub fn verify(&self, sim: &mut Simulation) -> Result<VerificationResult, ReplayError> {
        sim.restore(&self.initial_state)?;
        let mut mismatches = 0;

        for event in &self.events {
            while sim.tick < event.tick {
                sim.step()?;
            }
            sim.apply_logged_event(event)?;

            let actual = Self::compute_checksum(&sim.world);
            if let Some((_, expected)) = self.checksums.iter().find(|(t, _)| *t == event.tick) {
                if actual != *expected {
                    mismatches += 1;
                }
            }
        }

        if mismatches > 0 {
            Err(ReplayError::VerificationFailed { mismatches })
        } else {
            Ok(VerificationResult {
                ticks_verified: self.events.len() as u64,
                checksums_matched: self.checksums.len() as u64 - mismatches,
                mismatches,
            })
        }
    }
}

pub struct VerificationResult {
    pub ticks_verified: u64,
    pub checksums_matched: u64,
    pub mismatches: u64,
}
```

---

## 12. Networking Protocol

### 12.1 WebSocket Message Protocol

```rust
#[derive(serde::Serialize, serde::Deserialize, Debug)]
#[serde(tag = "command")]
pub enum ClientCommand {
    Subscribe { room: String, auth_token: Option<String> },
    Unsubscribe { room: String },
    Control {
        action: ControlAction,
        simulation_id: String,
        speed: Option<u64>,
    },
    Query {
        query_type: QueryType,
        id: Option<String>,
        simulation_id: String,
    },
}

#[derive(serde::Serialize, serde::Deserialize, Debug)]
pub enum ControlAction { Pause, Resume, Step, SetSpeed }

#[derive(serde::Serialize, serde::Deserialize, Debug)]
pub enum QueryType { Agent, Faction, Policy, Metrics, Events }

#[derive(serde::Serialize, serde::Deserialize, Debug)]
#[serde(tag = "type")]
pub enum ServerMessage {
    Subscribed { room: String },
    Unsubscribed { room: String },
    Tick {
        tick: u64,
        timestamp: u64,
        snapshot: MetricsSnapshot,
        events: Vec<SimulationEvent>,
        checksum: String,
    },
    ControlResponse {
        action: String,
        status: String,
        simulation_id: String,
    },
    QueryResponse {
        query_type: QueryType,
        data: serde_json::Value,
    },
    Error {
        code: String,
        message: String,
        simulation_id: Option<String>,
    },
    SimulationCompleted {
        simulation_id: String,
        final_tick: u64,
        final_metrics: MetricsSnapshot,
        checksum: String,
    },
}
```

### 12.2 REST API Request/Response Models

```rust
#[derive(serde::Serialize, serde::Deserialize, Debug)]
pub struct StartScenarioRequest {
    pub seed: u64,
    pub config: Option<SimulationConfig>,
}

#[derive(serde::Serialize, serde::Deserialize, Debug)]
pub struct StartScenarioResponse {
    pub simulation_id: String,
    pub status: String,
    pub tick: u64,
    pub seed: u64,
}

#[derive(serde::Serialize, serde::Deserialize, Debug)]
pub struct ControlRequest {
    pub action: String,
    pub speed: Option<u64>,
}

#[derive(serde::Serialize, serde::Deserialize, Debug)]
pub struct MetricsQuery {
    pub start: Option<u64>,
    pub end: Option<u64>,
    pub format: Option<String>,
}

#[derive(serde::Serialize, serde::Deserialize, Debug)]
pub struct CheckpointResponse {
    pub checkpoint_id: String,
    pub tick: u64,
    pub checksum: String,
}

#[derive(serde::Serialize, serde::Deserialize, Debug)]
pub struct VerificationResponse {
    pub verified: bool,
    pub checksums_match: u64,
    pub ticks_replayed: u64,
    pub time_ms: u64,
}
```

---

## 13. Serialization & Persistence

### 13.1 Checkpoint Format

```rust
#[derive(serde::Serialize, serde::Deserialize)]
pub struct Checkpoint {
    pub version: u32,
    pub tick: u64,
    pub seed: u64,
    pub world_state: Vec<u8>,
    pub market_state: Vec<u8>,
    pub climate_state: Vec<u8>,
    pub metrics: Vec<u8>,
    pub rng_state: Vec<u8>,
    pub checksum: u64,
}

impl Checkpoint {
    pub fn create(sim: &Simulation) -> Result<Self, CivisError> {
        let world_state = bincode::serialize(&sim.world)?;
        let market_state = bincode::serialize(&sim.market)?;
        let climate_state = bincode::serialize(&sim.climate)?;
        let metrics = bincode::serialize(&sim.metrics)?;
        let rng_state = bincode::serialize(&sim.rng)?;

        let mut hasher = xxhash_rust::xxh3::Xxh3::new();
        hasher.update(&world_state);
        hasher.update(&market_state);
        hasher.update(&climate_state);
        hasher.update(&metrics);
        hasher.update(&rng_state);
        let checksum = hasher.digest();

        Ok(Self {
            version: 1,
            tick: sim.tick,
            seed: sim.seed,
            world_state,
            market_state,
            climate_state,
            metrics,
            rng_state,
            checksum,
        })
    }

    pub fn verify(&self) -> Result<(), CivisError> {
        let mut hasher = xxhash_rust::xxh3::Xxh3::new();
        hasher.update(&self.world_state);
        hasher.update(&self.market_state);
        hasher.update(&self.climate_state);
        hasher.update(&self.metrics);
        hasher.update(&self.rng_state);
        let computed = hasher.digest();

        if computed != self.checksum {
            return Err(CivisError::ChecksumMismatch {
                tick: self.tick,
                expected: self.checksum,
                actual: computed,
            });
        }
        Ok(())
    }
}
```

### 13.2 Save/Load API

```rust
impl Simulation {
    pub fn save(&self, path: &std::path::Path) -> Result<(), CivisError> {
        let checkpoint = Checkpoint::create(self)?;
        let data = bincode::serialize(&checkpoint)?;
        std::fs::write(path, data)?;
        Ok(())
    }

    pub fn load(path: &std::path::Path) -> Result<Self, CivisError> {
        let data = std::fs::read(path)?;
        let checkpoint: Checkpoint = bincode::deserialize(&data)?;
        checkpoint.verify()?;

        let world = bincode::deserialize(&checkpoint.world_state)?;
        let market = bincode::deserialize(&checkpoint.market_state)?;
        let climate = bincode::deserialize(&checkpoint.climate_state)?;
        let metrics = bincode::deserialize(&checkpoint.metrics)?;
        let rng = bincode::deserialize(&checkpoint.rng_state)?;

        Ok(Self {
            tick: checkpoint.tick,
            seed: checkpoint.seed,
            world,
            market,
            climate,
            rng,
            metrics,
            systems: Vec::new(),
            replay: None,
            status: SimulationStatus::Paused,
            invariants: Invariants::default(),
        })
    }
}
```

---

## 14. Performance Targets

### 14.1 Target Metrics

| Metric | Target | Method | Verification |
|--------|--------|--------|--------------|
| Agents | 100,000 | ECS + parallel systems | Benchmark suite |
| Ticks per second | 1,000 | Deterministic RNG, optimized systems | cargo bench |
| Startup time | <1 second | Binary snapshots | Integration test |
| Memory per simulation | <500 MB | Compact components, SoA layout | Memory profiling |
| Replay speed | 10x real-time | Event log only | Replay benchmark |
| Save/Load | <100 ms | Bincode serialization | I/O benchmark |
| Concurrent simulations | 10+ | Multi-process isolation | Load test |

### 14.2 Optimization Techniques

| Technique | Expected Impact | When Applied | Complexity |
|-----------|-----------------|--------------|------------|
| Structure of Arrays (SoA) | 2-5x | All component storage | Low |
| Parallel system execution | 2-8x | >10K agents | Medium |
| Spatial indexing | 10x | Movement, trade queries | Medium |
| Fixed-point arithmetic | 1.5x | Economic calculations | Low |
| Object pools | 2x | Frequent spawn/despawn | Low |
| Incremental checksum | 1.3x | Replay recording | Medium |
| Component bitsets | 5x | Query filtering | Low |
| Archetype iteration | 3x | Multi-component queries | Low |

### 14.3 Memory Budget

| Component | Per-Entity Size | 100K Entities | Notes |
|-----------|----------------|---------------|-------|
| Position | 8 bytes | 800 KB | i32 x, y |
| Velocity | 8 bytes | 800 KB | f32 dx, dy |
| Health | 12 bytes | 1.2 MB | f32 current, max + u32 age |
| Wealth | 24 bytes | 2.4 MB | 3x I64F64 |
| Needs | 24 bytes | 2.4 MB | 6x f32 |
| Ideology | 20 bytes | 2.0 MB | 5x f32 |
| Citizen (marker) | 0 bytes | 0 bytes | Marker only |
| **Total per entity** | **~96 bytes** | **~9.6 MB** | Core components |
| ECS overhead | ~32 bytes | ~3.2 MB | Archetype tables |
| **Total ECS** | **~128 bytes** | **~12.8 MB** | Well under 500 MB target |

---

## 15. Testing Strategy

### 15.1 Test Categories

| Category | Tool | Coverage Target | Examples |
|----------|------|-----------------|----------|
| Unit Tests | cargo test | 80%+ | Component creation, system logic |
| Integration Tests | cargo test --test | All APIs | Scenario execution, API endpoints |
| Property Tests | proptest | Key invariants | Energy conservation, population non-negative |
| Determinism Tests | Custom | Cross-platform | Same seed = same result on all platforms |
| Performance Tests | criterion | Baseline + regression | Tick rate, memory usage |
| Replay Tests | Custom | All scenarios | Record then verify = match |

### 15.2 Determinism Testing

```rust
#[test]
fn cross_platform_determinism() {
    let seed = 12345u64;
    let mut sim = Simulation::new(seed, SimulationConfig::default());

    for _ in 0..1000 {
        sim.step().unwrap();
    }

    let checksum = sim.compute_checksum();
    assert_eq!(checksum, EXPECTED_CHECKSUM_1000,
        "Checksum mismatch -- determinism violated!");
}

#[test]
fn replay_verification() {
    let seed = 12345u64;
    let mut sim = Simulation::new(seed, SimulationConfig {
        enable_replay: true,
        ..Default::default()
    });

    for _ in 0..1000 {
        sim.step().unwrap();
    }

    let replay = sim.replay.as_ref().unwrap();
    let mut verify_sim = Simulation::new(seed, SimulationConfig::default());
    let result = replay.verify(&mut verify_sim);
    assert!(result.is_ok(), "Replay verification failed: {:?}", result);
}
```

### 15.3 Property-Based Testing

```rust
use proptest::prelude::*;

proptest! {
    #[test]
    fn energy_conservation(seed: u64, ticks in 1..1000u32) {
        let mut sim = Simulation::new(seed, SimulationConfig::default());
        let initial_energy = sim.total_energy();

        for _ in 0..ticks {
            sim.step().unwrap();
        }

        let final_energy = sim.total_energy();
        prop_assert!(final_energy <= initial_energy,
            "Energy increased: {} > {}", final_energy, initial_energy);
    }

    #[test]
    fn population_non_negative(seed: u64, ticks in 1..1000u32) {
        let mut sim = Simulation::new(seed, SimulationConfig::default());

        for _ in 0..ticks {
            sim.step().unwrap();
            prop_assert!(sim.population_count() >= 0,
                "Population went negative at tick {}", sim.tick);
        }
    }

    #[test]
    fn currency_non_negative(seed: u64, ticks in 1..1000u32) {
        let mut sim = Simulation::new(seed, SimulationConfig::default());

        for _ in 0..ticks {
            sim.step().unwrap();
            prop_assert!(sim.all_wealth_non_negative(),
                "Negative wealth detected at tick {}", sim.tick);
        }
    }
}
```

### 15.4 Scenario Validation Tests

| Scenario | Expected Outcome | Tolerance | Test |
|----------|-----------------|-----------|------|
| Isolated economy | Stable equilibrium | +/-5% | test_isolated_economy |
| War between factions | Population decline | >10% | test_war_population |
| Universal basic income | Reduced poverty | >20% | test_ubi_poverty |
| Carbon tax | Reduced emissions | >15% | test_carbon_tax |
| Plague outbreak | Population crash | >30% | test_plague |

---

## 16. Deployment Architecture

### 16.1 Docker Deployment

```dockerfile
FROM rust:1.75 as builder
WORKDIR /app
COPY . .
RUN cargo build --release --bin civis-server

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y ca-certificates && rm -rf /var/lib/apt/lists/*
COPY --from=builder /app/target/release/civis-server /usr/local/bin/
EXPOSE 8080
CMD ["civis-server"]
```

### 16.2 Kubernetes Deployment

```yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: civis-server
spec:
  replicas: 3
  selector:
    matchLabels:
      app: civis
  template:
    metadata:
      labels:
        app: civis
    spec:
      containers:
      - name: civis-server
        image: civis/civis-server:latest
        ports:
        - containerPort: 8080
        resources:
          requests:
            memory: "512Mi"
            cpu: "500m"
          limits:
            memory: "1Gi"
            cpu: "1000m"
        env:
        - name: RUST_LOG
          value: "info"
        - name: CIVIS_MAX_SIMULATIONS
          value: "10"
        - name: CIVIS_TICK_RATE
          value: "60"
---
apiVersion: v1
kind: Service
metadata:
  name: civis-service
spec:
  selector:
    app: civis
  ports:
  - port: 80
    targetPort: 8080
  type: LoadBalancer
```

### 16.3 Environment Variables

| Variable | Default | Description |
|----------|---------|-------------|
| `RUST_LOG` | `info` | Log level (trace, debug, info, warn, error) |
| `CIVIS_BIND_ADDR` | `0.0.0.0:8080` | Server bind address |
| `CIVIS_MAX_SIMULATIONS` | `10` | Maximum concurrent simulations |
| `CIVIS_TICK_RATE` | `60` | Simulation ticks per second |
| `CIVIS_WS_MAX_CONNECTIONS` | `100` | Maximum WebSocket connections |
| `CIVIS_RATE_LIMIT` | `60` | API requests per minute per client |
| `CIVIS_CHECKPOINT_DIR` | `/tmp/civis/checkpoints` | Checkpoint storage directory |
| `CIVIS_AUTH_TOKEN` | (none) | Optional JWT secret for API auth |

---

## 17. Configuration Reference

### 17.1 Simulation Configuration

```rust
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct SimulationConfig {
    pub seed: u64,
    pub tick_limit: Option<u64>,
    pub checkpoint_interval: u64,
    pub enable_replay: bool,
    pub parallel_threads: usize,
    pub metrics_interval: u64,
    pub enable_invariants: bool,
    pub log_level: String,
}

impl Default for SimulationConfig {
    fn default() -> Self {
        Self {
            seed: 0,
            tick_limit: None,
            checkpoint_interval: 1000,
            enable_replay: true,
            parallel_threads: 1,
            metrics_interval: 1,
            enable_invariants: true,
            log_level: "info".to_string(),
        }
    }
}
```

### 17.2 Economy Configuration

```rust
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct EconomyConfig {
    pub good_count: usize,
    pub clearing_algorithm: ClearingAlgorithm,
    pub initial_currency: Currency,
    pub enable_trade: bool,
    pub base_tariff: f32,
    pub max_price_change: f32,
}

#[derive(Clone, Copy, Debug, serde::Serialize, serde::Deserialize)]
pub enum ClearingAlgorithm { Equilibrium, OrderBook, Auction }
```

### 17.3 Policy Configuration

```rust
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct PolicyConfig {
    pub enable_legislation: bool,
    pub proposal_rate: f32,
    pub enforcement_efficiency: f32,
    pub base_corruption: f32,
    pub min_legitimacy: f32,
}
```

---

## 18. Scenario Format Specification

### 18.1 YAML Schema

```yaml
scenario:
  id: string                    # Unique scenario identifier
  name: string                  # Human-readable name
  description: string           # Detailed description
  version: string               # Scenario version (semver)
  author: string                # Scenario author

  config:
    seed: integer               # Random seed (required for determinism)
    tick_limit: integer         # Maximum ticks (optional)
    checkpoint_interval: integer
    enable_replay: boolean

  world:
    map:
      width: integer
      height: integer
      seed: integer
      climate_zones: [string]

    population:
      initial_size: integer
      distribution: string      # "clustered", "uniform", "random"
      age_distribution: string  # "realistic", "uniform"

    factions:
      - id: string
        name: string
        government: string
        territory: [integer]    # [min_x, min_y, max_x, max_y]
        population_share: float

  interventions:
    - tick: integer
      type: string              # "policy_enact", "climate_event"
      policy:
        name: string
        domain: string
        instruments:
          - type: string
            parameters:
              rate: float

  metrics:
    - population
    - gdp
    - gini_coefficient
    - carbon_emissions
    - avg_ideology_economic
    - health_index
    - conflict_score

  success_criteria:
    - metric: string
      target: string
      deadline_tick: integer
```

### 18.2 Scenario Validation Rules

| Rule | Description | Error |
|------|-------------|-------|
| Seed required | seed must be present and non-zero | MissingField: seed |
| Tick limit positive | tick_limit must be > 0 | OutOfRange: tick_limit |
| Valid climate zones | Must be from predefined list | InvalidValue: climate_zones |
| Valid government | Must be from GovernmentType enum | InvalidValue: government |
| Territory bounds | Must be within map dimensions | OutOfRange: territory |
| Population share sum | All faction shares must sum to 1.0 | InvalidValue: population_share |
| Valid instrument types | Must be from InstrumentType enum | InvalidValue: instrument |
| No unseeded randomness | All random sources must be seeded | UnseededRandomness |
| No external dependencies | No external API calls allowed | ExternalDependency |

---

## 19. Metrics & Observability

### 19.1 Metrics Categories

| Category | Metrics | Collection Frequency | Export Format |
|----------|---------|---------------------|---------------|
| Demographics | Population, births, deaths, avg age | Every tick | CSV, JSON, Parquet |
| Economy | GDP, Gini, unemployment, inflation | Every tick | CSV, JSON, Parquet |
| Social | Ideology distribution, social mobility | Every tick | CSV, JSON |
| Health | Health index, life expectancy, disease | Every tick | CSV, JSON |
| Conflict | Conflict score, war count, refugees | Every tick | CSV, JSON |
| Environment | Carbon emissions, forest coverage, biodiversity | Every tick | CSV, JSON, NetCDF |
| Governance | Legitimacy, corruption, policy count | Every tick | CSV, JSON |
| Performance | Tick duration, memory usage, entity count | Every 100 ticks | Prometheus |

### 19.2 Tracing Integration

Structured logging via the tracing crate provides observability into simulation execution. All systems emit trace spans with tick context. Key events (policy enactment, trade execution, conflict) emit trace events with full context.

```rust
use tracing::{info, warn, error, debug, instrument};

#[instrument(skip(world, rng), fields(tick = tick))]
fn execute_system(world: &mut World, rng: &mut ChaCha8Rng, tick: Tick) {
    info!("Executing system");
    // System logic
    debug!("System completed");
}
```

### 19.3 Prometheus Metrics

The server exposes a `/metrics` endpoint compatible with Prometheus scraping:

- `civis_simulations_active`: Number of running simulations
- `civis_ticks_total`: Total ticks executed across all simulations
- `civis_tick_duration_seconds`: Histogram of tick execution times
- `civis_entities_total`: Total entities across all simulations
- `civis_clients_connected`: Number of active WebSocket connections
- `civis_api_requests_total`: Counter of API requests by endpoint
- `civis_api_request_duration_seconds`: Histogram of API request times
- `civis_memory_bytes`: Current memory usage
- `civis_checkpoint_errors_total`: Counter of checkpoint failures

---

## 20. Modding & Extensibility

### 20.1 WASM Modding System

Mods compile to WASM and execute in a sandboxed runtime. Mod authors can write in any language that targets WASM (Rust recommended, TypeScript via AssemblyScript, C/C++ via Emscripten, Go via TinyGo).

```rust
use wasmtime::*;

pub struct ModLoader {
    engine: Engine,
    linker: Linker<ModState>,
}

impl ModLoader {
    pub fn load_mod(&self, wasm_bytes: &[u8]) -> Result<ModInstance> {
        let module = Module::from_binary(&self.engine, wasm_bytes)?;
        let mut store = Store::new(&self.engine, ModState::default());

        self.linker.define(&mut store, "host", "log", |mut caller: Caller<'_, ModState>, ptr: i32, len: i32| {
            // Host logging function with bounds checking
        })?;

        let instance = self.linker.instantiate(&mut store, &module)?;
        Ok(ModInstance { instance, store })
    }
}
```

### 20.2 Plugin Architecture

The engine uses a plugin architecture where each domain (economy, policy, social, climate, spatial) is a separate plugin. Plugins register their components, systems, and resources with the engine.

```rust
pub trait Plugin {
    fn name(&self) -> &'static str;
    fn register_components(&self, world: &mut World);
    fn register_systems(&self, registry: &mut SystemRegistry);
    fn register_resources(&self, world: &mut World);
}

pub struct EconomyPlugin;

impl Plugin for EconomyPlugin {
    fn name(&self) -> &'static str { "economy" }

    fn register_components(&self, world: &mut World) {
        world.register_component::<Wealth>();
    }

    fn register_systems(&self, registry: &mut SystemRegistry) {
        registry.register(Box::new(EconomySystem));
    }

    fn register_resources(&self, world: &mut World) {
        world.insert_resource(Market::new());
    }
}
```

### 20.3 Custom Scenario Types

Users can define custom scenario types by extending the YAML schema. Custom types must implement the Scenario trait:

```rust
pub trait Scenario {
    fn validate(&self) -> Result<(), Vec<ValidationError>>;
    fn apply(&self, sim: &mut Simulation) -> Result<(), CivisError>;
    fn tick(&self) -> u64;
}
```

---

## 21. Glossary

| Term | Definition |
|------|------------|
| **ABM** | Agent-Based Modeling -- simulation paradigm with autonomous agents |
| **ACE** | Agent-Based Computational Economics |
| **ADR** | Architecture Decision Record |
| **BDI** | Belief-Desire-Intention -- agent architecture |
| **CGE** | Computable General Equilibrium -- economic modeling approach |
| **ChaCha8Rng** | Cryptographic RNG algorithm used for deterministic randomness |
| **DSGE** | Dynamic Stochastic General Equilibrium -- macroeconomic models |
| **ECS** | Entity-Component-System -- data-oriented architecture |
| **Fixed-point** | Integer representation of decimals for deterministic arithmetic |
| **Gini** | Gini coefficient -- measure of wealth inequality (0 = equal, 1 = unequal) |
| **IAM** | Integrated Assessment Model -- climate-economy coupling |
| **Joule** | Energy unit in simulation economy |
| **ODD** | Overview, Design concepts, Details -- ABM description protocol |
| **RNG** | Random Number Generator |
| **SoA** | Structure of Arrays -- memory layout for cache efficiency |
| **Tick** | Discrete time step in simulation |
| **Seed** | Initial value for deterministic RNG |
| **Replay** | Recording and re-execution of simulation |
| **Determinism** | Same inputs always produce same outputs |
| **WASM** | WebAssembly -- sandboxed execution environment |
| **WASI** | WebAssembly System Interface |

---

## 22. References

### 22.1 Architecture Decision Records

- [ADR-001: Deterministic Simulation Architecture](docs/adr/ADR-001-deterministic-simulation.md)
- [ADR-001: Rust Crate Structure](docs/adr/ADR-001-rust-crate-structure.md)
- [ADR-002: Entity-Component-System Architecture](docs/adr/ADR-002-ecs-architecture.md)
- [ADR-002: Joule Economy as Allocator](docs/adr/ADR-002-joule-economy-as-allocator.md)
- [ADR-003: Deterministic Replay](docs/adr/ADR-003-deterministic-replay.md)
- [ADR-003: Policy and Institution Modeling](docs/adr/ADR-003-policy-institution-modeling.md)

### 22.2 Research Documents

- [Game Engines, Simulation Systems & ECS Architectures SOTA](docs/research/GAME_ENGINES_SOTA.md)
- [Civilization Simulation SOTA](SOTA-CIVILIZATION-SIMULATION.md)

### 22.3 Academic References

- Epstein, J. M., & Axtell, R. (1996). *Growing Artificial Societies*. MIT Press.
- Tesfatsion, L. (2006). Agent-based computational economics. *Journal of Economic Perspectives*.
- Grimm, V., et al. (2006). A standard protocol for describing agent-based models. *Ecological Modelling*.
- Axelrod, R. (1997). The dissemination of culture. *Journal of Conflict Resolution*.
- Schelling, T. C. (1971). Dynamic models of segregation. *Journal of Mathematical Sociology*.

### 22.4 Software References

- [hecs documentation](https://docs.rs/hecs/latest/hecs/)
- [rand_chacha documentation](https://docs.rs/rand_chacha/latest/rand_chacha/)
- [bincode documentation](https://docs.rs/bincode/latest/bincode/)
- [fixed documentation](https://docs.rs/fixed/latest/fixed/)
- [tokio documentation](https://tokio.rs/)
- [axum documentation](https://docs.rs/axum/latest/axum/)

### 22.5 Related Projects

- NetLogo: https://ccl.northwestern.edu/netlogo/
- MASON: https://cs.gmu.edu/~eclab/projects/mason/
- Repast: https://repast.github.io/
- UrbanSim: https://github.com/UDST/urbansim
- FlameGPU: https://flamegpu.com/
- Factorio: https://factorio.com/

---

*This specification defines the Civis civilization simulation platform. For implementation details, see the source code and ADRs in docs/adr/. Last updated: 2026-04-03.*
