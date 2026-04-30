# ADR-002: Entity-Component-System Architecture

**Date**: 2026-04-04  
**Status**: Accepted  
**Deciders**: Civis Architecture Team  
**Related**: ADR-001, SPEC.md  

## Context

Civis must simulate civilizations with 100,000+ agents (citizens, institutions, military units) while maintaining deterministic behavior and high performance. Traditional object-oriented approaches have failed to meet these requirements in existing ABM platforms.

### Problem Analysis

| Issue | OO Approach | ECS Solution |
|-------|-------------|--------------|
| **Cache misses** | Pointer chasing through object graph | Contiguous memory (SoA) |
| **Flexibility** | Inheritance hierarchies are rigid | Composition via components |
| **Determinism** | Virtual dispatch non-deterministic | Sequential system execution |
| **Parallelism** | Shared mutable state | Data-parallel system processing |
| **Memory overhead** | Object headers, vtables | Compact component storage |

### Existing Platform Analysis

| Platform | Architecture | Max Agents | Performance |
|----------|--------------|------------|-------------|
| NetLogo | OOP (Logo) | 10K | Poor |
| MASON | OOP (Java) | 1M | Moderate |
| Repast | OOP (Java) | 100K | Moderate |
| AnyLogic | OOP (Java) | 100K | Moderate |
| FlameGPU | ECS (GPU) | 10M | Excellent |
| Bevy (game engine) | ECS (Rust) | 100K+ | Excellent |

**Conclusion**: ECS architecture enables the scale and performance required by Civis.

## Decision

**Adopt Entity-Component-System (ECS) architecture using the hecs library with the following principles:**

1. **Entities are IDs only** — Lightweight, no data
2. **Components are pure data** — No logic, cache-friendly layout
3. **Systems contain logic** — Stateless, deterministic functions
4. **World is the registry** — Owns entities, components, and systems
5. **Deterministic iteration order** — Required by ADR-001

### Architecture Overview

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                      Civis ECS Architecture                                  │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                             │
│  ┌──────────────────────────────────────────────────────────────────────┐  │
│  │                         Entity IDs                                      │  │
│  │  0, 1, 2, 3, 4, 5, ... (sparse index)                                  │  │
│  └──────────────────────────────────────────────────────────────────────┘  │
│                                                                             │
│  ┌──────────────────────────────────────────────────────────────────────┐  │
│  │                     Component Storage (SoA)                             │  │
│  │                                                                         │  │
│  │  Position:    [(0,0), (1,5), (3,2), (7,1), ...]  Vec<Position>          │  │
│  │  Wealth:     [100.0, 50.5, 200.0, 10.0, ...]    Vec<Wealth>            │  │
│  │  Health:     [1.0, 0.8, 1.0, 0.3, ...]           Vec<Health>           │  │
│  │  Ideology:   [-0.5, 0.2, 0.8, -0.1, ...]         Vec<Ideology>         │  │
│  │  Citizen:    [(), (), (), (), ...]               Vec<Citizen>          │  │
│  │                                                                         │  │
│  └──────────────────────────────────────────────────────────────────────┘  │
│                                                                             │
│  ┌──────────────────────────────────────────────────────────────────────┐  │
│  │                         Systems                                         │  │
│  │                                                                         │  │
│  │  EconomySystem:    (Wealth, Position) → trade, market clearing       │  │
│  │  SocialSystem:     (Ideology, Citizen) → opinion dynamics              │  │
│  │  HealthSystem:     (Health, Citizen) → aging, disease                │  │
│  │  MovementSystem:   (Position, Citizen) → migration                     │  │
│  │  PolicySystem:     (Citizen, Institution) → governance                │  │
│  │                                                                         │  │
│  └──────────────────────────────────────────────────────────────────────┘  │
│                                                                             │
│  ┌──────────────────────────────────────────────────────────────────────┐  │
│  │                     System Execution Order                              │  │
│  │                                                                         │  │
│  │  1. PolicySystem      → Update institutions, laws                     │  │
│  │  2. EconomySystem     → Trade, production, market clearing             │  │
│  │  3. HealthSystem      → Age, disease, death                           │  │
│  │  4. SocialSystem      → Opinion formation, network updates             │  │
│  │  5. MovementSystem    → Migration, location updates                    │  │
│  │                                                                         │  │
│  │  (Fixed order ensures determinism)                                     │  │
│  └──────────────────────────────────────────────────────────────────────┘  │
│                                                                             │
└─────────────────────────────────────────────────────────────────────────────┘
```

## Detailed Design

### Component Definitions

Components are plain data structures with no logic:

```rust
// Core identity component
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Citizen;

// Spatial component
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Position {
    pub x: i32,
    pub y: i32,
}

// Economic component
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Wealth {
    pub amount: Currency, // Fixed-point from ADR-001
}

// Health component
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Health {
    pub current: f32,
    pub max: f32,
    pub age: u32,
}

// Social component
#[derive(Clone, Debug, PartialEq)]
pub struct Ideology {
    pub economic: f32,   // -1 (communist) to 1 (capitalist)
    pub social: f32,     // -1 (liberal) to 1 (conservative)
    pub foreign: f32,    // -1 (isolationist) to 1 (interventionist)
}

// Network component
#[derive(Clone, Debug, PartialEq, Default)]
pub struct SocialNetwork {
    pub ties: Vec<Entity>, // Social connections
}

// Employment component
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Employment {
    pub employer: Option<Entity>,
    pub wage: Currency,
    pub satisfaction: f32,
}
```

### System Definitions

Systems are stateless functions that operate on component queries:

```rust
use hecs::World;

/// Economy system: handles trading, market clearing, production
pub struct EconomySystem;

impl EconomySystem {
    pub fn run(world: &mut World, rng: &mut ChaCha8Rng, market: &mut Market) {
        // Query all citizens with wealth and position
        let mut query = world.query_mut::<(&mut Wealth, &Position, &Citizen)>();
        
        // Collect trade intentions (deterministic order)
        let mut trade_intentions: Vec<(Entity, TradeIntention)> = Vec::new();
        for (entity, (wealth, position, _)) in &mut query {
            if let Some(intention) = Self::generate_intention(wealth, position, rng) {
                trade_intentions.push((entity, intention));
            }
        }
        
        // Sort by entity ID for determinism
        trade_intentions.sort_by_key(|(e, _)| e.id());
        
        // Execute trades
        for (entity, intention) in trade_intentions {
            market.execute(entity, intention);
        }
        
        // Clear market
        market.clear(rng);
    }
    
    fn generate_intention(
        wealth: &Wealth, 
        position: &Position, 
        rng: &mut ChaCha8Rng
    ) -> Option<TradeIntention> {
        // Deterministic decision logic
        if rng.gen::<f32>() < 0.3 { // 30% chance to trade
            Some(TradeIntention {
                good: rng.gen_range(0..NUM_GOODS),
                quantity: rng.gen_range(1..10),
                max_price: wealth.amount * 0.1,
            })
        } else {
            None
        }
    }
}

/// Social system: handles opinion dynamics, network formation
pub struct SocialSystem;

impl SocialSystem {
    pub fn run(world: &mut World, rng: &mut ChaCha8Rng) {
        // Update social networks
        let mut query = world.query_mut::<(&Position, &mut SocialNetwork)>();
        
        // Build spatial index for efficient neighbor finding
        let spatial_index = SpatialGrid::from_query(&query);
        
        // Process network updates (deterministic order)
        for (entity, (position, network)) in &mut query {
            // Find nearby agents
            let neighbors = spatial_index.query_radius(position, 10.0);
            
            // Form new ties based on proximity and RNG
            for neighbor in neighbors {
                if rng.gen::<f32>() < TIE_FORMATION_PROBABILITY {
                    network.ties.push(neighbor);
                }
            }
        }
    }
}
```

### World and Tick Loop

```rust
pub struct Simulation {
    world: World,
    rng: ChaCha8Rng,
    tick: Tick,
    market: Market,
    systems: Vec<Box<dyn System>>,
}

pub trait System {
    fn run(&self, world: &mut World, rng: &mut ChaCha8Rng, market: &mut Market);
}

impl Simulation {
    pub fn new(seed: u64) -> Self {
        let mut world = World::new();
        let rng = ChaCha8Rng::seed_from_u64(seed);
        let market = Market::new();
        
        // Systems in deterministic execution order
        let systems: Vec<Box<dyn System>> = vec![
            Box::new(PolicySystem),
            Box::new(EconomySystem),
            Box::new(HealthSystem),
            Box::new(SocialSystem),
            Box::new(MovementSystem),
        ];
        
        Self {
            world,
            rng,
            tick: 0,
            market,
            systems,
        }
    }
    
    pub fn step(&mut self) {
        // Execute systems in fixed order (deterministic)
        for system in &self.systems {
            system.run(&mut self.world, &mut self.rng, &mut self.market);
        }
        
        self.tick += 1;
    }
}
```

### Deterministic Parallel Execution

For performance at scale, systems can execute in parallel when there are no data dependencies:

```rust
use rayon::prelude::*;

pub struct ParallelSystems;

impl ParallelSystems {
    /// Systems that can run in parallel (no overlapping component writes)
    pub fn run_parallel(world: &mut World, rng: &mut ChaCha8Rng) {
        // Split RNG for parallel streams
        let mut rngs: Vec<ChaCha8Rng> = (0..4)
            .map(|i| {
                let mut stream_rng = rng.clone();
                stream_rng.set_stream(i);
                stream_rng
            })
            .collect();
        
        // HealthSystem and MovementSystem can run in parallel
        // (they write different components)
        rayon::join(
            || HealthSystem::run(world, &mut rngs[0]),
            || MovementSystem::run(world, &mut rngs[1]),
        );
    }
}
```

## Component Design Guidelines

### DO

- Keep components small and focused (< 64 bytes ideally)
- Use `Copy` types where possible (avoids allocation)
- Store IDs (Entity) rather than references
- Use fixed-point for economic values
- Document invariants

### DON'T

- Put logic in components
- Use heap-allocated types in hot paths
- Store complex nested structures
- Use references to other entities (use Entity IDs)
- Use floating-point for equality

### Example Component Sizes

| Component | Size | Notes |
|-----------|------|-------|
| Citizen | 0 bytes | ZST marker component |
| Position | 8 bytes | Two i32s |
| Wealth | 16 bytes | Fixed-point currency |
| Health | 12 bytes | Three f32s (acceptable) |
| Ideology | 12 bytes | Three f32s |
| Employment | 24 bytes | Option<Entity> + Currency + f32 |
| SocialNetwork | 24 bytes | Vec header (heap allocated) |

## System Design Guidelines

### DO

- Keep systems stateless
- Query only needed components
- Sort results by entity ID for determinism
- Use jump-ahead RNG for parallel execution
- Document side effects

### DON'T

- Mutate components not in query
- Spawn/despawn during iteration (queue for next tick)
- Use system time or randomness outside RNG
- Depend on iteration order (sort explicitly)
- Use locks or shared mutable state

## Consequences

### Positive

1. **Performance**: Cache-friendly memory layout enables 10x speedup over OOP
2. **Determinism**: Sequential system execution is naturally deterministic
3. **Composability**: Components combine flexibly without inheritance
4. **Parallelism**: Data-parallel system execution scales with cores
5. **Testability**: Systems are pure functions (with RNG parameter)
6. **Modularity**: Systems can be added/removed without affecting others

### Negative

1. **Learning Curve**: ECS is unfamiliar to many developers
2. **Verbosity**: More code than equivalent OOP for simple cases
3. **Refactoring Cost**: Changing component structure requires migration
4. **Debugging**: Entity IDs less intuitive than object references
5. **Library Dependency**: hecs is additional dependency

### Mitigations

| Concern | Mitigation |
|---------|------------|
| Learning | Documentation and examples; pair programming |
| Verbosity | Macros for common patterns; code generation |
| Refactoring | Migration utilities; versioned schemas |
| Debugging | Entity inspection tools; visualization |
| Dependency | hecs is small, well-maintained, MIT licensed |

## Alternatives Considered

### Alternative 1: Object-Oriented with Generational Indices (Rejected)

**Approach**: Use generational indices instead of pointers; Vec storage

**Rejection Reason**: Still has cache locality issues; complex lifetime management; less ecosystem support than ECS

### Alternative 2: Actor Model (Rejected)

**Approach**: Each agent is an actor with inbox; message passing

**Rejection Reason**: Message queues non-deterministic; overhead too high for 100K agents; debugging difficult

### Alternative 3: Data-Oriented Design without ECS (Rejected)

**Approach**: Custom SoA without hecs; manual entity management

**Rejection Reason**: Reimplementation of hecs features; more code to maintain; less tested than library

### Alternative 4: legion ECS (Considered)

**Approach**: Use legion instead of hecs

**Comparison**:

| Feature | hecs | legion |
|---------|------|--------|
| Maintenance | Active | Maintenance mode |
| API Stability | Stable | Changed frequently |
| Performance | Good | Better (batching) |
| Complexity | Lower | Higher |
| Determinism | Easy | Requires care |

**Decision**: hecs for predictability; may migrate to bevy_ecs in future

## Implementation Status

### Completed

- [x] hecs integration
- [x] Core component definitions
- [x] System trait and dispatcher
- [x] Deterministic system ordering

### In Progress

- [ ] Parallel system execution
- [ ] Component serialization for save/load
- [ ] System scheduling optimization

### Planned

- [ ] Visual entity inspector
- [ ] Component migration utilities
- [ ] System profiling and optimization

## References

- [hecs documentation](https://docs.rs/hecs/latest/hecs/) — Rust ECS library
- [ECS FAQ](https://github.com/SanderMertens/ecs-faq) — ECS patterns and best practices
- [Data-Oriented Design](https://www.dataorienteddesign.com/dodbook/) — Richard Fabian
- [Bevy ECS](https://bevyengine.org/learn/book/getting-started/ecs/) — Bevy engine ECS
- [Unity ECS](https://docs.unity3d.com/Packages/com.unity.entities@0.17/manual/index.html) — Unity DOTS

---

*This ADR establishes ECS as the foundational architecture for Civis simulation code.*
