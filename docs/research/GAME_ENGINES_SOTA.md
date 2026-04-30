# Game Engines, Simulation Systems & ECS Architectures — State of the Art Research

**Document ID:** PHENOTYPE_CIVIS_SOTA_GAME_ENGINES  
**Status:** Active Research  
**Last Updated:** 2026-04-03  
**Author:** Phenotype Architecture Team

---

## Table of Contents

1. [Executive Summary](#1-executive-summary)
2. [Research Methodology](#2-research-methodology)
3. [Game Engine Landscape](#3-game-engine-landscape)
4. [Entity-Component-System (ECS) Deep Dive](#4-entity-component-system-ecs-deep-dive)
5. [Bevy Engine Analysis](#5-bevy-engine-analysis)
6. [Fyrox Engine Analysis](#6-fyrox-engine-analysis)
7. [Macroquad & Minimal Rust Engines](#7-macroquad--minimal-rust-engines)
8. [Godot 4 & GDExtension](#8-godot-4--gdextension)
9. [Unity DOTS / ECS](#9-unity-dots--ecs)
10. [Unreal Engine MassEntity](#10-unreal-engine-massentity)
11. [Game Loop Architectures](#11-game-loop-architectures)
12. [Rendering Pipelines](#12-rendering-pipelines)
13. [Networking & Multiplayer](#13-networking--multiplayer)
14. [Deterministic Lockstep](#14-deterministic-lockstep)
15. [Physics Engines](#15-physics-engines)
16. [Audio Systems](#16-audio-systems)
17. [Asset Pipelines](#17-asset-pipelines)
18. [Scripting & Modding](#18-scripting--modding)
19. [Comparison Matrices](#19-comparison-matrices)
20. [Rust Ecosystem for Simulation](#20-rust-ecosystem-for-simulation)
21. [Recommendations for Civis](#21-recommendations-for-civis)
22. [References](#22-references)

---

## 1. Executive Summary

This document presents a comprehensive state-of-the-art analysis of game engines, simulation systems, and ECS (Entity-Component-System) architectures as of 2026. The research specifically targets the design requirements of **Civis** — a deterministic civilization simulation engine built as a Rust workspace with `crates/engine` and `crates/server`.

### 1.1 Key Findings

| Finding | Impact on Civis |
|---------|-----------------|
| **ECS is the dominant architecture for large-scale simulation** | Confirms ADR-002; hecs/Bevy ECS are optimal choices |
| **Rust game engines have matured significantly** | Bevy 0.16+ is production-ready for 2D/3D simulation |
| **Deterministic lockstep is proven in production games** | Factorio, Age of Empires II DE validate the approach |
| **WebGPU is becoming the rendering standard** | Future-proof rendering via wgpu/Bevy |
| **WASM deployment enables browser-based simulation** | Critical for accessibility and education use cases |
| **Headless server pattern is industry standard** | Confirms crates/server architecture |

### 1.2 Technology Recommendations

| Layer | Recommended Technology | Rationale |
|-------|----------------------|-----------|
| ECS Core | Bevy ECS (standalone) or hecs | Proven, ergonomic, deterministic iteration |
| Rendering | wgpu (via Bevy) | WebGPU, cross-platform, headless capable |
| Networking | tokio + tungstenite | Async, production-grade, WebSocket support |
| Physics | rapier (optional) | Pure Rust, deterministic mode available |
| Audio | kira | Rust-native, spatial audio support |
| Asset Loading | bevy_asset or custom | Hot-reloading, async loading |
| Serialization | bincode + serde | Fast, compact, deterministic |
| WASM Target | wasm32-unknown-unknown | Browser deployment, headless server |

---

## 2. Research Methodology

### 2.1 Selection Criteria

Engines and architectures were evaluated against:

1. **Determinism Support**: Reproducible execution, seed-based randomness
2. **ECS Maturity**: Component queries, system scheduling, parallel execution
3. **Rust Native**: First-class Rust support vs. bindings
4. **Headless Capability**: Server-mode without rendering
5. **Performance**: Agent count, tick rate, memory efficiency
6. **Ecosystem**: Crates, documentation, community size
7. **Licensing**: Permissive licenses (MIT, Apache-2.0)

### 2.2 Analysis Dimensions

| Dimension | Weight | Metrics |
|-----------|--------|---------|
| ECS Architecture | 25% | Query ergonomics, system scheduling, parallel safety |
| Determinism | 20% | Reproducibility, seed support, cross-platform consistency |
| Performance | 15% | Benchmarks, memory layout, cache efficiency |
| Ecosystem | 15% | Crates, docs, community, examples |
| Rendering | 10% | Pipeline flexibility, shader support, headless mode |
| Networking | 10% | Multiplayer support, lockstep, latency handling |
| Tooling | 5% | Editor, profiler, debugger, hot-reload |

### 2.3 Scope

This research covers:
- **Rust-native engines**: Bevy, Fyrox, Macroquad, ggez
- **Traditional engines with Rust support**: Godot (GDExtension), Unity (DOTS)
- **ECS libraries**: hecs, legion, shipyard, specs, Bevy ECS
- **Simulation patterns**: Lockstep, rollback, event sourcing
- **Rendering**: Vulkan, WebGPU, OpenGL, Metal abstractions
- **Networking**: ENet, WebSockets, WebRTC, custom UDP

---

## 3. Game Engine Landscape

### 3.1 Engine Classification

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                        GAME ENGINE TAXONOMY                                  │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                             │
│  ┌─────────────────────┐    ┌─────────────────────┐                        │
│  │  BY LANGUAGE        │    │  BY ARCHITECTURE     │                        │
│  ├─────────────────────┤    ├─────────────────────┤                        │
│  │ • Rust-native       │    │ • ECS-based          │                        │
│  │ • C++ with bindings │    │ • OOP-based          │                        │
│  │ • Web-based         │    │ • Data-oriented      │                        │
│  │ • Script-first      │    │ • Component-based    │                        │
│  └─────────────────────┘    └─────────────────────┘                        │
│                                                                             │
│  ┌─────────────────────┐    ┌─────────────────────┐                        │
│  │  BY RENDERING       │    │  BY DEPLOYMENT       │                        │
│  ├─────────────────────┤    ├─────────────────────┤                        │
│  │ • Vulkan/Metal/DX12 │    │ • Desktop (native)   │                        │
│  │ • WebGPU            │    │ • Web (WASM)         │                        │
│  │ • OpenGL (legacy)   │    │ • Mobile             │                        │
│  │ • Headless          │    │ • Server (headless)  │                        │
│  └─────────────────────┘    └─────────────────────┘                        │
│                                                                             │
└─────────────────────────────────────────────────────────────────────────────┘
```

### 3.2 Engine Survey

| Engine | Language | ECS | Rendering | Headless | WASM | License |
|--------|----------|-----|-----------|----------|------|---------|
| Bevy | Rust | ✓ (built-in) | wgpu | ✓ | ✓ | MIT/Apache |
| Fyrox | Rust | ✗ (OOP) | OpenGL/DX11 | Partial | ✗ | MIT |
| Macroquad | Rust | ✗ | OpenGL | ✓ | ✓ | Zlib |
| ggez | Rust | ✗ | wgpu | Partial | ✗ | MIT |
| Godot 4 | C++/GDScript | ✗ | Vulkan/OpenGL | ✓ | ✓ | MIT |
| Unity | C# | DOTS (opt-in) | Custom | ✓ | ✗ | Proprietary |
| Unreal | C++ | MassEntity (new) | Custom | ✓ | ✗ | Proprietary |
| O3DE | C++ | ✗ | Vulkan/DX12 | ✓ | ✗ | Apache-2.0 |

### 3.3 Rust Engine Maturity Timeline

```
2019 ── specs ECS matures, Amethyst active
2020 ── Bevy 0.1 released, Amethyst pauses development
2021 ── Bevy 0.5, PBR rendering, scene system
2022 ── Bevy 0.8, ECS v2, async system scheduling
2023 ── Bevy 0.11, UI overhaul, WebGPU support
2024 ── Bevy 0.14, Bevy UI 2.0, 3D PBR improvements
2025 ── Bevy 0.15, WGSL shaders, improved WASM
2026 ── Bevy 0.16, production-ready, mature ecosystem
```

---

## 4. Entity-Component-System (ECS) Deep Dive

### 4.1 ECS Fundamentals

The Entity-Component-System architecture inverts traditional OOP design:

```
Traditional OOP:
  Entity (class) ──> has methods + data together
  └─ Inheritance hierarchies
  └─ Virtual dispatch overhead
  └─ Cache-unfriendly memory layout

ECS Architecture:
  Entity ──> Just an ID (u32 or u64)
  Component ──> Pure data (struct, no methods)
  System ──> Logic that operates on component queries
  └─ Data-oriented design
  └─ Cache-friendly (Structure of Arrays)
  └─ Implicit parallelism via borrow checking
```

### 4.2 ECS Library Comparison

| Library | Query Syntax | Parallel | Archetype | Ergonomics | Maturity |
|---------|-------------|----------|-----------|------------|----------|
| **Bevy ECS** | `Query<&mut T, With<U>>` | ✓ (system-level) | Archetype | Excellent | Production |
| **hecs** | `world.query::<(&T, &mut U)>()` | ✓ (manual) | Archetype | Good | Stable |
| **legion** | `<&T, &mut U>::query()` | ✓ (automatic) | Archetype | Good | Mature |
| **shipyard** | `world.borrow::<&T>()` | ✓ | Archetype | Good | Mature |
| **specs** | `Join` trait | ✓ (system-level) | Storage-based | Verbose | Declining |
| **flecs** | C/C++ API | ✓ | Archetype | Moderate | Active |

### 4.3 Bevy ECS — Deep Analysis

Bevy ECS is the most mature Rust ECS implementation, featuring:

#### 4.3.1 Archetype Storage

```rust
// Components are stored in archetype tables
// Each unique component combination = one archetype
// Query iteration is cache-friendly within archetypes

// Archetype: [Position, Velocity, Health]
// Archetype: [Position, Velocity]
// Archetype: [Position, Health]

// Querying (&Position, &mut Velocity) only touches
// archetypes that have both components
```

#### 4.3.2 System Scheduling

```rust
// Bevy automatically parallelizes systems based on
// component access patterns (Rust borrow checking)

fn movement_system(mut query: Query<(&Position, &mut Velocity)>) {
    // Exclusive access to Velocity, shared access to Position
    // Bevy knows this conflicts with systems that write Position
}

fn render_system(query: Query<(&Position, &Sprite)>) {
    // Shared access to Position — can run parallel to movement_system
}

// Bevy's scheduler builds a DAG of system dependencies
// Systems with no conflicts run in parallel
```

#### 4.3.3 System Sets and Ordering

```rust
#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
enum CivisSimulationSet {
    Policy,
    Economy,
    Social,
    Climate,
    Movement,
    Conflict,
    Metrics,
}

// Systems execute in this order for determinism:
app.configure_sets(
    Update,
    (
        CivisSimulationSet::Policy,
        CivisSimulationSet::Economy,
        CivisSimulationSet::Social,
        CivisSimulationSet::Climate,
        CivisSimulationSet::Movement,
        CivisSimulationSet::Conflict,
        CivisSimulationSet::Metrics,
    ).chain(),
);
```

#### 4.3.4 Determinism Considerations

```rust
// Bevy ECS iteration order is deterministic within archetypes
// BUT: system execution order must be explicitly configured

// GOOD: Explicit ordering
app.add_systems(Update, (
    policy_system,
    economy_system.after(policy_system),
    social_system.after(economy_system),
));

// BAD: Implicit ordering (non-deterministic)
app.add_systems(Update, (policy_system, economy_system, social_system));
// These may execute in any order!

// For Civis: Use SystemSets with explicit chaining
// to guarantee deterministic system execution order
```

### 4.4 hecs — Deep Analysis

hecs is a lightweight, minimal ECS library:

#### 4.4.1 Core API

```rust
use hecs::World;

let mut world = World::new();

// Spawn entities with components
let entity = world.spawn((
    Position { x: 0.0, y: 0.0 },
    Velocity { dx: 1.0, dy: 0.0 },
    Health { current: 100.0, max: 100.0 },
));

// Query with pattern matching
for (id, (pos, vel)) in world.query_mut::<(&mut Position, &Velocity)>() {
    pos.x += vel.dx;
    pos.y += vel.dy;
}

// Efficient batch operations
let mut query = world.query::<(&Position, &Velocity)>();
for (id, (pos, vel)) in query.iter() {
    // Read-only access — can be parallelized
}
```

#### 4.4.2 Determinism in hecs

```rust
// hecs iteration is deterministic within archetypes
// Archetypes are created in insertion order
// Entity IDs are deterministic (sequential allocation)

// For Civis determinism:
// 1. Spawn entities in deterministic order
// 2. Use stable entity IDs (not recycled)
// 3. Process queries in entity ID order
// 4. Avoid dynamic component addition during iteration

// Example: deterministic query processing
fn process_entities_deterministic(world: &mut World) {
    let mut entities: Vec<_> = world.query_mut::<(&Entity, &mut Position)>()
        .collect();
    entities.sort_by_key(|(e, _)| e.id()); // Ensure deterministic order
    for (entity, pos) in entities {
        // Process in ID order
    }
}
```

### 4.5 legion — Deep Analysis

legion offers automatic parallelism:

```rust
use legion::*;
use legion::systems::CommandBuffer;

#[system]
#[read_component(Position)]
#[write_component(Velocity)]
fn movement(pos: &Position, vel: &mut Velocity) {
    // legion automatically parallelizes this system
    // based on the read/write annotations
}

// Subqueries for filtering
#[system]
fn only_citizens(
    #[read_component] citizen: &Citizen,
    #[read_component] pos: &Position,
) {
    // Only runs on entities with Citizen component
}

// legion's scheduler handles parallelism automatically
let mut schedule = Schedule::builder()
    .add_system(movement_system())
    .add_system(render_system())
    .build();

schedule.execute(&mut world, &mut resources);
```

### 4.6 ECS Architecture Patterns for Simulation

#### 4.6.1 Component Design

```rust
// GOOD: Small, focused components
#[derive(Component, Clone, Copy, Debug)]
pub struct Position {
    pub x: f64,
    pub y: f64,
}

#[derive(Component, Clone, Copy, Debug)]
pub struct Velocity {
    pub dx: f64,
    pub dy: f64,
}

#[derive(Component, Clone, Debug)]
pub struct Citizen {
    pub id: CitizenId,
    pub birth_tick: u64,
    pub generation: u32,
}

// BAD: God components (anti-pattern)
#[derive(Component)]
pub struct Citizen {
    pub position: Position,
    pub velocity: Velocity,
    pub health: Health,
    pub inventory: Inventory,
    pub ideology: Ideology,
    pub social_network: SocialNetwork,
    // ... 50 more fields
}
```

#### 4.6.2 System Design

```rust
// GOOD: Single-responsibility systems
fn movement_system(query: Query<(&Position, &mut Velocity)>) {
    for (pos, mut vel) in query.iter() {
        // Only handles movement
    }
}

fn economy_system(
    market: Res<Market>,
    query: Query<(&Citizen, &mut Wealth)>,
) {
    // Only handles economy
}

// BAD: Multi-purpose systems
fn update_everything(query: Query<&mut Everything>) {
    // Handles movement, economy, social, climate...
    // Impossible to parallelize or reason about
}
```

#### 4.6.3 Event-Driven ECS

```rust
// Event component pattern for decoupled systems
#[derive(Component)]
pub struct PendingEvent {
    pub event_type: EventType,
    pub data: EventData,
    pub priority: u8,
}

// Event processing system
fn event_system(
    mut commands: Commands,
    query: Query<(Entity, &PendingEvent)>,
    mut events: EventWriter<SimulationEvent>,
) {
    for (entity, event) in query.iter() {
        events.send(SimulationEvent {
            source: entity,
            event_type: event.event_type,
            data: event.data.clone(),
        });
        commands.entity(entity).remove::<PendingEvent>();
    }
}
```

---

## 5. Bevy Engine Analysis

### 5.1 Architecture Overview

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                              Bevy Engine                                    │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                             │
│  ┌─────────────────────────────────────────────────────────────────────┐   │
│  │                         App / Schedule                               │   │
│  │  ┌──────────┐ ┌──────────┐ ┌──────────┐ ┌──────────┐ ┌──────────┐  │   │
│  │  │ First    │ │ PreUpdate│ │ Update   │ │ PostUpdate│ │ Last    │  │   │
│  │  │ (setup)  │ │          │ │ (game)   │ │          │ │ (cleanup)│  │   │
│  │  └──────────┘ └──────────┘ └──────────┘ └──────────┘ └──────────┘  │   │
│  └─────────────────────────────────────────────────────────────────────┘   │
│                                   │                                         │
│  ┌───────────────────────────────┴─────────────────────────────────────┐   │
│  │                          ECS Core                                    │   │
│  │  ┌──────────┐ ┌──────────┐ ┌──────────┐ ┌──────────┐               │   │
│  │  │ World    │ │ Query    │ │ System   │ │ Resource │               │   │
│  │  │          │ │          │ │          │ │          │               │   │
│  │  │ Entities │ │ Filters  │ │ Sets     │ │ Global   │               │   │
│  │  │ Archetypes│ │ Iteration│ │ Ordering │ │ State    │               │   │
│  │  └──────────┘ └──────────┘ └──────────┘ └──────────┘               │   │
│  └─────────────────────────────────────────────────────────────────────┘   │
│                                   │                                         │
│  ┌───────────────────────────────┴─────────────────────────────────────┐   │
│  │                        Rendering (wgpu)                              │   │
│  │  ┌──────────┐ ┌──────────┐ ┌──────────┐ ┌──────────┐               │   │
│  │  │ Pipeline │ │ Mesh     │ │ Material │ │ Camera   │               │   │
│  │  │          │ │          │ │          │ │          │               │   │
│  │  │ WGSL     │ │ GPU buf  │ │ PBR      │ │ View     │               │   │
│  │  │ Shaders  │ │ Vertex   │ │ Custom   │ │ Matrix   │               │   │
│  │  └──────────┘ └──────────┘ └──────────┘ └──────────┘               │   │
│  └─────────────────────────────────────────────────────────────────────┘   │
│                                   │                                         │
│  ┌───────────────────────────────┴─────────────────────────────────────┐   │
│  │                      Input / Audio / Asset                           │   │
│  │  ┌──────────┐ ┌──────────┐ ┌──────────┐ ┌──────────┐               │   │
│  │  │ Input    │ │ Audio    │ │ Asset    │ │ Window   │               │   │
│  │  │ (events) │ │ (kira)   │ │ (async)  │ │ (winit)  │               │   │
│  │  └──────────┘ └──────────┘ └──────────┘ └──────────┘               │   │
│  └─────────────────────────────────────────────────────────────────────┘   │
│                                                                             │
└─────────────────────────────────────────────────────────────────────────────┘
```

### 5.2 Bevy for Headless Simulation

```rust
// Civis can use Bevy ECS without rendering
fn main() {
    App::new()
        // No DefaultPlugins — headless mode
        .insert_resource(SimulationConfig {
            seed: 42,
            tick_limit: None,
        })
        .insert_resource(SimRng::new(42))
        .add_systems(Update, (
            policy_system,
            economy_system.after(policy_system),
            social_system.after(economy_system),
            climate_system.after(social_system),
            movement_system.after(climate_system),
            metric_system.after(movement_system),
        ).chain())
        .run();
}

// This runs the ECS scheduler without any rendering
// Perfect for crates/engine — pure simulation logic
```

### 5.3 Bevy WASM Support

```rust
// Bevy compiles to WASM with minimal configuration
// wasm32-unknown-unknown target

// Cargo.toml
[target.wasm32-unknown-unknown.dependencies]
bevy = { version = "0.16", default-features = false, features = [
    "bevy_winit",
    "bevy_render",
    "webgl2",  # or "webgpu" for modern browsers
] }

// Benefits for Civis:
// 1. Browser-based simulation viewing
// 2. No server rendering cost
// 3. Client-side prediction possible
// 4. Educational deployment
```

### 5.4 Bevy Plugin Architecture

```rust
// Civis can structure each domain as a Bevy plugin
pub struct CivisEnginePlugin;

impl Plugin for CivisEnginePlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(EconomyPlugin)
            .add_plugins(PolicyPlugin)
            .add_plugins(SocialPlugin)
            .add_plugins(ClimatePlugin)
            .add_plugins(SpatialPlugin)
            .add_plugins(MetricsPlugin);
    }
}

// Each plugin encapsulates its components, systems, and resources
pub struct EconomyPlugin;

impl Plugin for EconomyPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<Market>()
            .init_resource::<JouleAllocator>()
            .add_systems(
                Update,
                (
                    trade_system,
                    production_system.after(trade_system),
                    market_clearing_system.after(production_system),
                ).in_set(CivisSimulationSet::Economy),
            );
    }
}
```

---

## 6. Fyrox Engine Analysis

### 6.1 Overview

Fyrox (formerly rg3d) is a Rust game engine with a scene graph architecture:

```rust
// Fyrox uses scene graph + script component pattern
// NOT ECS — more traditional game engine architecture

impl Script for Game {
    fn on_init(&mut self, scene: &mut Scene, _: &mut GameEngine) {
        // Scene-based initialization
    }

    fn on_update(&mut self, scene: &mut Scene, engine: &mut GameEngine) {
        // Per-frame update
    }
}
```

### 6.2 Assessment for Civis

| Aspect | Rating | Notes |
|--------|--------|-------|
| ECS Support | ✗ | Scene graph, not ECS |
| Headless | Partial | Requires window context |
| WASM | ✗ | Not supported |
| Determinism | ✗ | No deterministic guarantees |
| Performance | Good | Native Rust performance |
| 3D Rendering | Good | PBR, shadows, post-processing |
| 2D Rendering | Good | Sprite support |

**Verdict**: Fyrox is unsuitable for Civis due to lack of ECS, no WASM support, and no headless mode. Its scene graph architecture conflicts with Civis's ECS-first design (ADR-002).

---

## 7. Macroquad & Minimal Rust Engines

### 7.1 Macroquad

```rust
// Macroquad is a minimal, immediate-mode game library
use macroquad::prelude::*;

#[macroquad::main("Game")]
async fn main() {
    loop {
        clear_background(BLACK);

        // Immediate mode rendering
        draw_circle(screen_width() / 2.0, screen_height() / 2.0, 100.0, RED);

        next_frame().await;
    }
}
```

### 7.2 Assessment for Civis

| Aspect | Rating | Notes |
|--------|--------|-------|
| ECS Support | ✗ | Manual implementation needed |
| Headless | ✓ | No window required |
| WASM | ✓ | Excellent support |
| Determinism | ✓ | Simple enough to control |
| Performance | Good | Minimal overhead |
| Complexity | Low | ~2000 LOC engine |

**Verdict**: Macroquad is suitable for simple 2D visualization of Civis simulation results but lacks the ECS architecture needed for the core simulation engine. Could be used for a lightweight client viewer.

---

## 8. Godot 4 & GDExtension

### 8.1 Architecture

Godot 4 uses a node-based scene graph with GDExtension for Rust:

```rust
// Godot 4 + gdext (Rust bindings)
use godot::prelude::*;

#[derive(GodotClass)]
#[class(init, base=Node2D)]
struct CivisSimulation {
    base: Base<Node2D>,
    world: World, // Civis ECS world
}

#[godot_api]
impl INode2D for CivisSimulation {
    fn process(&mut self, _delta: f64) {
        // Run one simulation tick
        self.world.step();
    }
}
```

### 8.2 Assessment for Civis

| Aspect | Rating | Notes |
|--------|--------|-------|
| ECS Support | ✗ | Node-based, not ECS |
| Headless | ✓ | `--headless` flag |
| WASM | ✓ | Godot 4.3+ supports WASM |
| Determinism | ✗ | Godot is not deterministic |
| Editor | Excellent | Best-in-class game editor |
| Rust Support | Good | gdext is mature |

**Verdict**: Godot could serve as a visualization frontend for Civis but should not be used for the core simulation. The node-based architecture conflicts with ECS, and Godot's non-deterministic execution would compromise Civis's core guarantee.

---

## 9. Unity DOTS / ECS

### 9.1 Architecture

Unity's Data-Oriented Technology Stack (DOTS):

```csharp
// Unity DOTS uses C# with Burst compiler
using Unity.Entities;
using Unity.Mathematics;

public struct Position : IComponentData {
    public float2 Value;
}

public struct Velocity : IComponentData {
    public float2 Value;
}

// System runs on all entities with Position + Velocity
[UpdateInGroup(typeof(SimulationSystemGroup))]
public partial struct MovementSystem : ISystem {
    public void OnUpdate(ref SystemState state) {
        foreach (var (pos, vel) in SystemAPI.Query<RefRW<Position>, RefRO<Velocity>>()) {
            pos.ValueRW += vel.ValueRO * SystemAPI.Time.DeltaTime;
        }
    }
}
```

### 9.2 Assessment for Civis

| Aspect | Rating | Notes |
|--------|--------|-------|
| ECS Support | ✓ | DOTS is mature ECS |
| Headless | ✓ | Dedicated server mode |
| WASM | ✗ | Not supported |
| Determinism | Partial | DOTS can be deterministic |
| Language | C# | Not Rust |
| Licensing | Proprietary | Revenue share required |

**Verdict**: Unity DOTS demonstrates that ECS can work at scale (used in production games) but is not suitable for Civis due to the C# requirement and proprietary licensing. However, DOTS's system grouping and scheduling patterns are valuable reference designs.

---

## 10. Unreal Engine MassEntity

### 10.1 Architecture

Unreal's MassEntity is a newer ECS framework:

```cpp
// Unreal MassEntity (C++)
// Fragment = Component
// Trait = Component bundle definition

USTRUCT()
struct FMassPositionFragment {
    GENERATED_BODY()
    FVector Location;
};

// Processors = Systems
class UMassMovementProcessor : public UMassProcessor {
    virtual void Execute(FMassEntityManager& EntityManager, FMassExecutionContext& Context) override {
        // Process entities with position + velocity
    }
};
```

### 10.2 Assessment for Civis

| Aspect | Rating | Notes |
|--------|--------|-------|
| ECS Support | ✓ | MassEntity (new, evolving) |
| Headless | ✓ | Dedicated server |
| WASM | ✗ | Not supported |
| Determinism | ✗ | Not guaranteed |
| Language | C++ | Not Rust |
| Complexity | Very High | Massive engine |

**Verdict**: Unreal is overkill for Civis and uses C++ instead of Rust. MassEntity's design patterns (fragment traits, processor execution) are informative but not directly applicable.

---

## 11. Game Loop Architectures

### 11.1 Fixed Timestep Loop

```rust
// Fixed timestep — essential for deterministic simulation
pub struct GameLoop {
    fixed_dt: Duration,
    accumulator: Duration,
    last_time: Instant,
}

impl GameLoop {
    pub fn new(ticks_per_second: u64) -> Self {
        Self {
            fixed_dt: Duration::from_secs_f64(1.0 / ticks_per_second as f64),
            accumulator: Duration::ZERO,
            last_time: Instant::now(),
        }
    }

    pub fn step<F>(&mut self, mut update: F)
    where
        F: FnMut(),
    {
        let now = Instant::now();
        let frame_time = now.duration_since(self.last_time);
        self.last_time = now;
        self.accumulator += frame_time;

        while self.accumulator >= self.fixed_dt {
            update(); // Fixed timestep update
            self.accumulator -= self.fixed_dt;
        }
    }
}

// For Civis: Use tick count, not wall clock time
pub struct SimulationLoop {
    current_tick: u64,
    tick_limit: Option<u64>,
}

impl SimulationLoop {
    pub fn run<F>(&mut self, mut tick_fn: F)
    where
        F: FnMut(u64) -> bool, // returns false to stop
    {
        while self.tick_limit.map_or(true, |limit| self.current_tick < limit) {
            if !tick_fn(self.current_tick) {
                break;
            }
            self.current_tick += 1;
        }
    }
}
```

### 11.2 Variable Timestep with Interpolation

```rust
// For rendering: interpolate between simulation states
pub struct InterpolatedState {
    previous: WorldState,
    current: WorldState,
    alpha: f64, // 0.0 to 1.0
}

impl InterpolatedState {
    pub fn interpolate_position(&self, entity: Entity) -> Position {
        let prev = self.previous.get_position(entity);
        let curr = self.current.get_position(entity);
        Position {
            x: prev.x + (curr.x - prev.x) * self.alpha,
            y: prev.y + (curr.y - prev.y) * self.alpha,
        }
    }
}

// Rendering runs at display refresh rate (60/120/144 Hz)
// Simulation runs at fixed tick rate (e.g., 60 ticks/sec)
// Interpolation provides smooth visual movement
```

### 11.3 Server Loop (Headless)

```rust
// Headless server loop for crates/server
pub struct ServerLoop {
    tick_rate: u64,
    max_ticks_per_frame: u64,
}

impl ServerLoop {
    pub async fn run(&self, sim: &mut Simulation, ws: &mut WebSocketServer) {
        let interval = Duration::from_secs_f64(1.0 / self.tick_rate as f64);
        let mut ticker = tokio::time::interval(interval);

        loop {
            ticker.tick().await;

            // Run simulation tick
            let result = sim.step();

            // Broadcast to connected clients
            ws.broadcast(TickUpdate {
                tick: sim.current_tick(),
                snapshot: result.snapshot,
                events: result.events,
            }).await;
        }
    }
}
```

### 11.4 Game Loop Comparison

| Pattern | Determinism | Smooth Rendering | Server Use | Complexity |
|---------|-------------|------------------|------------|------------|
| Fixed Timestep | ✓ | Requires interpolation | ✓ | Low |
| Variable Timestep | ✗ | ✓ | ✗ | Low |
| Semi-Fixed | Partial | ✓ | Partial | Medium |
| Tick-Based | ✓ | Separate render loop | ✓ | Medium |
| Event-Driven | ✓ | On-demand | ✓ | High |

---

## 12. Rendering Pipelines

### 12.1 wgpu / WebGPU

```rust
// wgpu is the Rust implementation of WebGPU
// Used by Bevy for rendering

use wgpu::*;

// Pipeline setup
let pipeline = device.create_render_pipeline(&RenderPipelineDescriptor {
    vertex: VertexState {
        module: &shader_module,
        entry_point: "vs_main",
        buffers: &[VertexBufferLayout {
            array_stride: std::mem::size_of::<Vertex>() as u64,
            attributes: &[
                VertexAttribute { format: VertexFormat::Float32x3, offset: 0, shader_location: 0 },
                VertexAttribute { format: VertexFormat::Float32x2, offset: 12, shader_location: 1 },
            ],
        }],
    },
    fragment: Some(FragmentState {
        module: &shader_module,
        entry_point: "fs_main",
        targets: &[Some(ColorTargetState {
            format: TextureFormat::Bgra8UnormSrgb,
            blend: Some(BlendState::REPLACE),
            write_mask: ColorWrites::ALL,
        })],
    }),
    primitive: PrimitiveState::default(),
    ..Default::default()
});
```

### 12.2 2D Rendering for Simulation

```rust
// For Civis 2D map rendering
// Approach: GPU instancing for tiles

#[derive(Clone, Copy, Pod, Zeroable)]
#[repr(C)]
struct TileInstance {
    position: [f32; 2],
    tile_type: u32,
    _padding: u32,
}

// Instance buffer — one entry per tile
let instance_buffer = device.create_buffer_init(&BufferInitDescriptor {
    label: Some("Tile Instances"),
    contents: bytemuck::cast_slice(&instances),
    usage: BufferUsages::VERTEX | BufferUsages::COPY_DST,
});

// Draw call renders all tiles in one pass
render_pass.draw_instanced(0..6, 0..num_tiles);
```

### 12.3 Headless Rendering

```rust
// For screenshot generation or map export
// wgpu can render without a window

use wgpu::util::DeviceExt;

// Create offscreen texture
let texture = device.create_texture(&TextureDescriptor {
    size: Extent3d { width: 2048, height: 2048, depth_or_array_layers: 1 },
    mip_level_count: 1,
    sample_count: 1,
    dimension: TextureDimension::D2,
    format: TextureFormat::Rgba8UnormSrgb,
    usage: TextureUsages::RENDER_ATTACHMENT | TextureUsages::COPY_SRC,
    label: Some("Offscreen"),
});

// Render to texture, then read back
let buffer = device.create_buffer(&BufferDescriptor {
    size: (2048 * 2048 * 4) as u64,
    usage: BufferUsages::COPY_DST | BufferUsages::MAP_READ,
    ..Default::default()
});

// Can be used for:
// - Map image export
// - Scenario visualization
// - Automated testing screenshots
```

### 12.4 Rendering Comparison

| Technology | API | Headless | WASM | Performance | Maturity |
|------------|-----|----------|------|-------------|----------|
| wgpu | WebGPU | ✓ | ✓ | Excellent | Mature |
| vulkano | Vulkan | ✓ | ✗ | Excellent | Stable |
| glow | OpenGL | ✓ | ✓ | Good | Mature |
| metal | Metal | ✗ | ✗ | Excellent | Apple only |
| dx12 | DirectX 12 | ✗ | ✗ | Excellent | Windows only |

---

## 13. Networking & Multiplayer

### 13.1 Architecture Patterns

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                     NETWORKING ARCHITECTURES                                 │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                             │
│  Client-Server (Authoritative):                                             │
│  ┌────────┐    Commands    ┌────────────────────────────────────────────┐  │
│  │ Client │ ──────────────> │              Server (Authoritative)        │  │
│  │        │ <────────────── │  ┌──────────┐  ┌──────────┐  ┌─────────┐  │  │
│  │ Render │    State        │  │ Sim      │  │ Validate │  │ Broad-  │  │  │
│  │ Only   │    Updates      │  │ Engine   │  │ Commands │  │ cast    │  │  │
│  └────────┘                 │  └──────────┘  └──────────┘  └─────────┘  │  │
│                             └────────────────────────────────────────────┘  │
│                                                                             │
│  Deterministic Lockstep:                                                    │
│  ┌────────┐    Inputs      ┌────────────────────────────────────────────┐  │
│  │ Client │ ──────────────> │              Input Server                  │  │
│  │  Sim   │ <────────────── │  ┌──────────┐  ┌──────────┐  ┌─────────┐  │  │
│  │  +     │    All Inputs   │  │ Collect  │  │ Broadcast│  │ Verify  │  │  │
│  │ Render │                 │  │ Inputs   │  │ to All   │  │ State   │  │  │
│  └────────┘                 │  └──────────┘  └──────────┘  └─────────┘  │  │
│                             └────────────────────────────────────────────┘  │
│                                                                             │
│  State Synchronization (Rollback):                                          │
│  ┌────────┐    Inputs      ┌────────────────────────────────────────────┐  │
│  │ Client │ ──────────────> │              Server                        │  │
│  │  Sim   │ <────────────── │  ┌──────────┐  ┌──────────┐  ┌─────────┐  │  │
│  │  +     │    State        │  │ Authori- │  │ Snapshot │  │ Roll-   │  │  │
│  │ Predict│    Corrections  │  │ tative   │  │ History  │  │ back    │  │  │
│  └────────┘                 │  └──────────┘  └──────────┘  └─────────┘  │  │
│                             └────────────────────────────────────────────┘  │
│                                                                             │
└─────────────────────────────────────────────────────────────────────────────┘
```

### 13.2 Deterministic Lockstep Implementation

```rust
// Lockstep is the pattern used by Factorio, Age of Empires, etc.
// All clients run the same simulation with the same inputs

pub struct LockstepNetwork {
    input_buffer: HashMap<u64, Vec<PlayerInput>>, // tick -> inputs
    confirmed_tick: u64,
    current_tick: u64,
    input_delay: u64, // Frames of input delay
}

impl LockstepNetwork {
    pub fn send_input(&mut self, tick: u64, input: PlayerInput) {
        self.input_buffer
            .entry(tick)
            .or_default()
            .push(input);
    }

    pub fn can_advance(&self) -> bool {
        // Can only advance when we have all inputs for next tick
        self.input_buffer.contains_key(&(self.current_tick + 1))
    }

    pub fn advance(&mut self, sim: &mut Simulation) {
        self.current_tick += 1;
        let inputs = self.input_buffer.remove(&self.current_tick).unwrap();

        // Apply inputs deterministically
        for input in inputs {
            sim.apply_input(input);
        }

        // Run simulation tick
        sim.step();
    }
}

// Client sends inputs every frame
// Server collects inputs from all clients
// Server broadcasts complete input set for each tick
// All clients simulate locally with same inputs
// Result: identical game state on all clients
```

### 13.3 WebSocket Server (crates/server)

```rust
// crates/server uses tokio + tungstenite for WebSocket
use tokio::net::TcpListener;
use tokio_tungstenite::accept_async;

pub struct CivisServer {
    simulations: HashMap<String, Simulation>,
    clients: HashMap<String, Vec<WebSocketStream<TcpStream>>>,
}

impl CivisServer {
    pub async fn run(&mut self, addr: &str) -> Result<(), Box<dyn Error>> {
        let listener = TcpListener::bind(addr).await?;

        while let Ok((stream, _)) = listener.accept().await {
            let ws_stream = accept_async(stream).await?;
            tokio::spawn(self.handle_client(ws_stream));
        }
        Ok(())
    }

    async fn handle_client(&self, mut ws: WebSocketStream<TcpStream>) {
        while let Some(msg) = ws.next().await {
            match msg {
                Ok(Message::Text(text)) => {
                    let cmd: ClientCommand = serde_json::from_str(&text)?;
                    let response = self.process_command(cmd).await?;
                    ws.send(Message::Text(serde_json::to_string(&response)?)).await?;
                }
                Ok(Message::Close(_)) => break,
                _ => continue,
            }
        }
    }
}
```

### 13.4 Networking Comparison

| Protocol | Latency | Reliability | Determinism | Best For |
|----------|---------|-------------|-------------|----------|
| WebSocket | Medium | ✓ (TCP) | ✓ | Civis server |
| UDP (raw) | Low | ✗ | ✓ | Real-time games |
| ENet | Low | Configurable | ✓ | Multiplayer games |
| WebRTC | Low | ✓ (SCTP) | ✓ | P2P games |
| HTTP/REST | High | ✓ | N/A | API endpoints |
| gRPC | Medium | ✓ | N/A | Microservices |

---

## 14. Deterministic Lockstep

### 14.1 Core Principles

Deterministic lockstep requires:

1. **Identical Initial State**: All clients start with the same world state
2. **Deterministic Simulation**: Same inputs always produce same outputs
3. **Synchronized Input**: All clients receive the same input for each tick
4. **No External State**: No system time, network state, or thread-dependent behavior in simulation

### 14.2 Implementation Checklist

```rust
// Deterministic lockstep requirements for Civis

// 1. Seeded RNG
let rng = ChaCha8Rng::seed_from_u64(seed);

// 2. Fixed-point arithmetic (not floating-point)
type Currency = fixed::types::I64F64;

// 3. Ordered collections
use indexmap::IndexMap; // Not HashMap

// 4. Deterministic iteration order
fn process_entities(world: &World) {
    let mut entities: Vec<_> = world.iter().collect();
    entities.sort_by_key(|e| e.id());
    for entity in entities {
        // Process in ID order
    }
}

// 5. No system time in simulation
// BAD: let now = Instant::now();
// GOOD: let tick = simulation.current_tick();

// 6. No thread-dependent behavior
// BAD: Rayon parallel iteration (non-deterministic order)
// GOOD: Sequential iteration or explicitly ordered parallel

// 7. Checksum verification
fn verify_state(sim: &Simulation) -> u64 {
    let state = sim.serialize();
    xxhash_rust::xxh3::xxh3_64(&state)
}
```

### 14.3 Proven Implementations

| Game | Engine | Determinism Method | Player Count |
|------|--------|-------------------|--------------|
| Factorio | Custom C++ | Lockstep + input delay | 4-65 |
| Age of Empires II DE | Custom C++ | Lockstep + rollback | 8 |
| StarCraft II | Custom C++ | Lockstep | 8 |
| Worms W.M.D | Custom C++ | Lockstep | 6 |
| Supreme Commander | Custom C++ | Lockstep | 8 |

### 14.4 Input Delay Calculation

```rust
// Input delay ensures all clients have received inputs before simulating
// Trade-off: higher delay = more tolerant of lag, worse responsiveness

pub fn calculate_input_delay(players: &[Player], target_latency_ms: u64) -> u64 {
    let max_latency = players.iter()
        .map(|p| p.latency_ms)
        .max()
        .unwrap_or(0);

    // Add buffer for jitter
    let buffer = 50; // ms
    let tick_duration_ms = 1000 / TICKS_PER_SECOND;

    ((max_latency + buffer) / tick_duration_ms) + 1
}

// For Civis: Server is authoritative, clients are viewers
// Input delay is less critical — server controls the tick rate
```

---

## 15. Physics Engines

### 15.1 rapier

```rust
// rapier is a pure Rust 2D/3D physics engine
use rapier2d::prelude::*;

let mut physics = PhysicsPipeline::new();
let mut island_manager = IslandManager::new();
let mut broad_phase = DefaultBroadPhase::new();
let mut narrow_phase = NarrowPhase::new();
let mut impulse_joint_set = ImpulseJointSet::new();
let mut multibody_joint_set = MultiBodyJointSet::new();
let mut ccd_solver = CCDSolver::new();

// Create rigid body
let rigid_body = RigidBodyBuilder::dynamic()
    .translation(vector![0.0, 5.0])
    .build();
let handle = rigid_body_set.insert(rigid_body);

// Create collider
let collider = ColliderBuilder::ball(0.5).build();
collider_set.insert_with_parent(collider, handle, &mut rigid_body_set);

// Step simulation
physics.step(
    &integration_params,
    &mut island_manager,
    &mut broad_phase,
    &mut narrow_phase,
    &mut rigid_body_set,
    &mut collider_set,
    &mut impulse_joint_set,
    &mut multibody_joint_set,
    &mut ccd_solver,
    None,
    &(),
);
```

### 15.2 Determinism in Physics

```rust
// rapier has a "deterministic" feature flag
// Uses fixed-point arithmetic internally

// For Civis: Physics is likely NOT needed for civilization simulation
// Movement is grid-based, not physics-based
// Combat is abstract, not collision-based

// If physics IS needed:
// 1. Enable deterministic feature
// 2. Use fixed timestep
// 3. Avoid floating-point comparisons
// 4. Verify cross-platform determinism
```

### 15.3 Physics Engine Comparison

| Engine | Language | Deterministic | 2D | 3D | WASM | Performance |
|--------|----------|---------------|-----|-----|------|-------------|
| rapier | Rust | ✓ (feature) | ✓ | ✓ | ✓ | Excellent |
| nphysics | Rust | ✗ | ✓ | ✓ | ✓ | Good |
| PhysX | C++ | ✗ | ✗ | ✓ | ✗ | Excellent |
| Box2D | C++ | ✓ | ✓ | ✗ | ✓ | Good |
| Jolt | C++ | Partial | ✗ | ✓ | ✗ | Excellent |

---

## 16. Audio Systems

### 16.1 kira

```rust
// kira is a Rust-native audio library
use kira::{
    manager::{AudioManager, AudioManagerSettings},
    sound::static_sound::{StaticSoundData, StaticSoundSettings},
};

let mut manager = AudioManager::new(AudioManagerSettings::default())?;

let sound = StaticSoundData::from_file("ambient.ogg", StaticSoundSettings::default())?;
manager.play(sound)?;

// Spatial audio for 3D positioning
use kira::sound::static_sound::SpatialSettings;

let sound = StaticSoundData::from_file("footstep.ogg", SpatialSettings {
    position: [x, y, z],
    ..Default::default()
})?;
```

### 16.2 Audio for Simulation

```rust
// For Civis: Audio is likely minimal
// - Ambient sounds for visualization mode
// - Event sounds (trade, conflict, policy enacted)
// - UI feedback sounds

// Headless server: No audio
// Client viewer: Optional ambient audio
```

---

## 17. Asset Pipelines

### 17.1 Asset Loading

```rust
// Bevy asset system with async loading
use bevy::asset::{Asset, AssetServer, Handle, Assets};

#[derive(Asset, TypePath)]
pub struct TerrainTile {
    pub texture: Handle<Image>,
    pub properties: TileProperties,
}

// Load assets asynchronously
fn load_assets(asset_server: Res<AssetServer>) {
    let tile_handle: Handle<TerrainTile> = asset_server.load("tiles/grass.tile");
}

// Track loading progress
fn check_loading(
    asset_server: Res<AssetServer>,
    tiles: Res<Assets<TerrainTile>>,
) {
    let load_state = asset_server.get_load_state(&tile_handle);
    match load_state {
        Some(LoadState::Loaded) => println!("Tile loaded!"),
        Some(LoadState::Failed) => eprintln!("Tile failed to load!"),
        _ => println!("Still loading..."),
    }
}
```

### 17.2 Asset Pipeline for Civis

```
Source Assets ──> Processing ──> Runtime Format ──> GPU Memory
     │                │                  │               │
  .png/.svg      Resize/Compress    .ktx2/.dds     Texture
  .glb/.gltf     Optimize meshes    .bin           Mesh buffer
  .json/.yaml    Validate schema    .bincode       Data
  .wav/.ogg      Convert format     .wav           Audio buffer
```

---

## 18. Scripting & Modding

### 18.1 WASM Plugin System

```rust
// Use WASM for modding — safe, sandboxed, cross-platform
use wasmtime::*;

pub struct ModLoader {
    engine: Engine,
    linker: Linker<ModState>,
}

impl ModLoader {
    pub fn load_mod(&self, wasm_bytes: &[u8]) -> Result<ModInstance> {
        let module = Module::from_binary(&self.engine, wasm_bytes)?;
        let mut store = Store::new(&self.engine, ModState::default());

        // Link host functions
        self.linker.define(&mut store, "host", "log", |mut caller: Caller<'_, ModState>, ptr: i32, len: i32| {
            // Host logging function
        })?;

        let instance = self.linker.instantiate(&mut store, &module)?;
        Ok(ModInstance { instance, store })
    }
}

// Mod authors write in any language that compiles to WASM:
// - Rust (recommended)
// - TypeScript (via AssemblyScript)
// - C/C++ (via Emscripten)
// - Go (via TinyGo)
```

### 18.2 Lua Scripting (Alternative)

```rust
// mlua for Lua scripting
use mlua::Lua;

let lua = Lua::new();
lua.load(r#"
    function on_tick(simulation)
        if simulation.tick % 365 == 0 then
            simulation:apply_policy("annual_tax")
        end
    end
"#).exec()?;

let on_tick: mlua::Function = lua.globals().get("on_tick")?;
on_tick.call(simulation)?;
```

### 18.3 Scripting Comparison

| Approach | Safety | Performance | Ecosystem | Hot Reload | Best For |
|----------|--------|-------------|-----------|------------|----------|
| WASM | ✓✓✓ | ✓✓ | Growing | ✓ | Civis mods |
| Lua | ✓✓ | ✓ | Mature | ✓ | Game scripting |
| Rhai (Rust) | ✓✓✓ | ✓✓ | Small | ✓ | Embedded Rust |
| Python (PyO3) | ✓ | ✓ | Huge | Partial | Data analysis |
| JavaScript (deno_core) | ✓✓ | ✓ | Huge | ✓ | Web integration |

---

## 19. Comparison Matrices

### 19.1 ECS Library Comparison (Detailed)

| Feature | Bevy ECS | hecs | legion | shipyard | specs |
|---------|----------|------|--------|----------|-------|
| Archetype Storage | ✓ | ✓ | ✓ | ✓ | ✗ |
| System Scheduling | ✓✓ | ✗ | ✓ | ✓ | ✓ |
| Automatic Parallelism | ✓ | ✗ | ✓ | ✓ | ✓ |
| Query Ergonomics | ✓✓ | ✓ | ✓ | ✓ | ✗ |
| Resource System | ✓ | ✗ | ✓ | ✓ | ✓ |
| Change Detection | ✓ | ✗ | ✓ | ✓ | ✗ |
| Bundle Support | ✓ | ✓ | ✓ | ✓ | ✓ |
| WASM Support | ✓ | ✓ | ✓ | ✓ | ✓ |
| Documentation | ✓✓ | ✓ | ✓ | ✓ | ✗ |
| Active Development | ✓✓ | ✓ | ✗ | ✓ | ✗ |

### 19.2 Engine Comparison for Simulation

| Criterion | Bevy | Fyrox | Macroquad | Godot 4 | Unity DOTS |
|-----------|------|-------|-----------|---------|------------|
| ECS Native | ✓✓ | ✗ | ✗ | ✗ | ✓ |
| Determinism | ✓ (configurable) | ✗ | ✓ | ✗ | Partial |
| Headless | ✓✓ | Partial | ✓ | ✓ | ✓ |
| WASM | ✓✓ | ✗ | ✓✓ | ✓ | ✗ |
| Rust Native | ✓✓ | ✓ | ✓ | ✗ | ✗ |
| 2D Rendering | ✓✓ | ✓ | ✓ | ✓✓ | ✓ |
| 3D Rendering | ✓ | ✓✓ | ✗ | ✓✓ | ✓✓ |
| Performance | ✓✓ | ✓ | ✓ | ✓ | ✓✓ |
| Ecosystem | ✓✓ | ✓ | ✓ | ✓✓ | ✓✓ |
| License | MIT/Apache | MIT | Zlib | MIT | Proprietary |

### 19.3 Networking Comparison for Deterministic Simulation

| Feature | WebSocket | UDP | ENet | WebRTC |
|---------|-----------|-----|------|--------|
| Reliability | ✓ (TCP) | ✗ | Configurable | ✓ (SCTP) |
| Latency | Medium | Low | Low | Low |
| Ordering | ✓ | ✗ | ✓ | ✓ |
| NAT Traversal | ✗ | ✗ | ✗ | ✓ (STUN/TURN) |
| Browser Support | ✓ | ✗ | ✗ | ✓ |
| Server Complexity | Low | High | Medium | High |
| Determinism | ✓ | ✓ | ✓ | ✓ |
| Best Use | Civis server | Fast-paced games | Multiplayer games | P2P games |

### 19.4 Rendering API Comparison

| API | Platform | Headless | WASM | Performance | Maturity |
|-----|----------|----------|------|-------------|----------|
| WebGPU (wgpu) | All | ✓ | ✓ | Excellent | Growing |
| Vulkan | Desktop | ✓ | ✗ | Excellent | Mature |
| OpenGL | All | ✓ | ✓ | Good | Legacy |
| Metal | Apple | ✗ | ✗ | Excellent | Mature |
| DirectX 12 | Windows | ✗ | ✗ | Excellent | Mature |

---

## 20. Rust Ecosystem for Simulation

### 20.1 Core Crates

| Crate | Purpose | Version | Deterministic | Notes |
|-------|---------|---------|---------------|-------|
| `hecs` | ECS | 0.10+ | ✓ | Lightweight, minimal |
| `bevy_ecs` | ECS (standalone) | 0.16+ | ✓ | Full-featured, can use without rendering |
| `rand` + `rand_chacha` | RNG | 0.8+ | ✓ | ChaCha8Rng for determinism |
| `fixed` | Fixed-point math | 2.0+ | ✓ | I64F64 for currency |
| `indexmap` | Ordered collections | 2.0+ | ✓ | Deterministic iteration |
| `bincode` | Serialization | 1.3+ | ✓ | Fast, compact |
| `serde` | Serialization framework | 1.0+ | ✓ | Required for bincode |
| `xxhash-rust` | Checksums | 0.8+ | ✓ | Fast state verification |
| `tokio` | Async runtime | 1.x | N/A | Server, networking |
| `tungstenite` | WebSocket | 0.20+ | N/A | WebSocket protocol |
| `axum` | HTTP server | 0.7+ | N/A | REST API |
| `kira` | Audio | 0.9+ | N/A | Rust-native audio |
| `rapier2d` | Physics (optional) | 0.17+ | ✓ (feature) | 2D physics |
| `rapier3d` | Physics (optional) | 0.17+ | ✓ (feature) | 3D physics |
| `wgpu` | Rendering | 0.20+ | N/A | WebGPU implementation |
| `wasmtime` | WASM runtime | 15+ | ✓ | Modding support |
| `mlua` | Lua scripting | 0.9+ | N/A | Alternative modding |

### 20.2 Bevy Ecosystem

| Crate | Purpose | Integration |
|-------|---------|-------------|
| `bevy` | Full engine | Core |
| `bevy_ecs` | ECS only | Standalone |
| `bevy_app` | App framework | Standalone |
| `bevy_time` | Time management | Standalone |
| `bevy_asset` | Asset loading | With rendering |
| `bevy_render` | Rendering | With wgpu |
| `bevy_window` | Window management | With winit |
| `bevy_input` | Input handling | With window |
| `bevy_ui` | UI framework | With rendering |
| `bevy_text` | Text rendering | With UI |
| `bevy_sprite` | 2D rendering | With rendering |
| `bevy_pbr` | 3D PBR | With rendering |
| `bevy_gltf` | GLTF loading | With rendering |
| `bevy_winit` | Window backend | Platform-specific |

### 20.3 Code Example: Standalone Bevy ECS

```rust
// Using Bevy ECS without rendering — ideal for crates/engine
use bevy_ecs::prelude::*;

// Components
#[derive(Component)]
struct Position { x: f64, y: f64 }

#[derive(Component)]
struct Velocity { dx: f64, dy: f64 }

#[derive(Component)]
struct Citizen { id: u64 }

#[derive(Resource)]
struct SimRng { rng: ChaCha8Rng }

#[derive(Resource)]
struct Market { /* ... */ }

// Systems
fn movement_system(mut query: Query<(&mut Position, &Velocity)>) {
    for (mut pos, vel) in query.iter_mut() {
        pos.x += vel.dx;
        pos.y += vel.dy;
    }
}

fn economy_system(
    mut market: ResMut<Market>,
    query: Query<(&Citizen, &Position)>,
) {
    for (citizen, pos) in query.iter() {
        // Economic logic
    }
}

fn metric_system(
    query: Query<&Citizen>,
    // Record metrics
) {
    let population = query.iter().count();
    // Record to time-series
}

// App setup
fn main() {
    let mut world = World::new();

    // Insert resources
    world.insert_resource(SimRng::new(42));
    world.insert_resource(Market::new());

    // Spawn entities
    for i in 0..10000 {
        world.spawn((
            Citizen { id: i },
            Position { x: 0.0, y: 0.0 },
            Velocity { dx: 0.1, dy: 0.1 },
        ));
    }

    // Create schedule
    let mut schedule = Schedule::default();
    schedule.add_systems((
        movement_system,
        economy_system.after(movement_system),
        metric_system.after(economy_system),
    ));

    // Run simulation
    for _tick in 0..1000 {
        schedule.run(&mut world);
    }
}
```

---

## 21. Recommendations for Civis

### 21.1 Architecture Decision Summary

Based on this research, the following recommendations are made for Civis:

| Decision | Recommendation | Rationale |
|----------|---------------|-----------|
| ECS Library | Bevy ECS (standalone) or hecs | Bevy ECS for full features, hecs for minimalism |
| Rendering | wgpu (via Bevy or standalone) | WebGPU standard, headless capable |
| Networking | tokio + tungstenite | Production-grade async WebSocket |
| Physics | None (grid-based movement) | Civilization sim doesn't need physics |
| Audio | Optional (kira) | Only for visualization mode |
| Scripting | WASM (wasmtime) | Safe, sandboxed, multi-language |
| Serialization | bincode + serde | Fast, compact, deterministic |
| RNG | ChaCha8Rng | Deterministic, parallel streams |
| Math | fixed (I64F64) | Cross-platform determinism |
| Collections | indexmap | Deterministic iteration |

### 21.2 Crate Structure Recommendation

```
Civis/
├── crates/
│   ├── engine/              # Core simulation (Bevy ECS standalone)
│   │   ├── src/
│   │   │   ├── lib.rs       # Public API
│   │   │   ├── simulation.rs # Simulation state
│   │   │   ├── tick.rs      # Tick loop
│   │   │   ├── rng.rs       # Deterministic RNG
│   │   │   ├── components/  # ECS components
│   │   │   ├── systems/     # ECS systems
│   │   │   ├── resources/   # ECS resources
│   │   │   └── replay.rs    # Replay system
│   │   └── Cargo.toml
│   │
│   └── server/              # Headless server (tokio + WebSocket)
│       ├── src/
│       │   ├── lib.rs       # Public API
│       │   ├── main.rs      # Entry point
│       │   ├── websocket.rs # WebSocket handler
│       │   ├── http.rs      # REST API (axum)
│       │   ├── protocol.rs  # Message protocol
│       │   └── sessions.rs  # Client sessions
│       └── Cargo.toml
```

### 21.3 Implementation Priorities

| Priority | Task | Dependencies | Timeline |
|----------|------|-------------|----------|
| P0 | ECS foundation (Bevy ECS or hecs) | None | Week 1-2 |
| P0 | Deterministic RNG setup | ECS | Week 1 |
| P0 | Basic tick loop | ECS, RNG | Week 2 |
| P0 | Component definitions | ECS | Week 2-3 |
| P1 | System implementations | Components | Week 3-4 |
| P1 | Serialization (bincode) | Components | Week 4 |
| P1 | Replay system | Tick loop, serialization | Week 4-5 |
| P2 | WebSocket server | Engine | Week 5-6 |
| P2 | REST API | Engine | Week 6 |
| P2 | Scenario loading | Engine, serialization | Week 6-7 |
| P3 | WASM modding | Engine | Week 8+ |
| P3 | Visualization client | Engine, wgpu | Week 8+ |

### 21.4 Risk Mitigation

| Risk | Likelihood | Impact | Mitigation |
|------|------------|--------|------------|
| Bevy API instability | Medium | Medium | Pin version, abstract ECS interface |
| Cross-platform determinism failure | Low | High | CI testing on multiple platforms |
| Performance bottleneck | Medium | High | Benchmark early, profile hot paths |
| WASM size too large | Medium | Medium | Tree-shaking, feature flags |
| WebSocket scalability | Low | Medium | Connection pooling, backpressure |

---

## 22. References

### 22.1 ECS Libraries

- [Bevy ECS](https://docs.rs/bevy_ecs/latest/bevy_ecs/) — Full-featured ECS with scheduling
- [hecs](https://docs.rs/hecs/latest/hecs/) — Lightweight, minimal ECS
- [legion](https://docs.rs/legion/latest/legion/) — Automatic parallelism
- [shipyard](https://docs.rs/shipyard/latest/shipyard/) — Type-safe ECS
- [specs](https://docs.rs/specs/latest/specs/) — Original Rust ECS (declining)

### 22.2 Game Engines

- [Bevy](https://bevyengine.org/) — Rust-native data-driven engine
- [Fyrox](https://fyrox.rs/) — Rust game engine with editor
- [Macroquad](https://github.com/not-fl3/macroquad) — Minimal Rust game library
- [ggez](https://ggez.rs/) — Rust game framework
- [Godot 4](https://godotengine.org/) — Open-source game engine
- [Unity DOTS](https://unity.com/dots) — Data-Oriented Technology Stack

### 22.3 Rendering

- [wgpu](https://wgpu.rs/) — WebGPU implementation in Rust
- [vulkano](https://vulkano.rs/) — Vulkan wrapper
- [glow](https://github.com/grovesNL/glow) — OpenGL abstraction

### 22.4 Networking

- [tokio](https://tokio.rs/) — Async runtime
- [tungstenite](https://docs.rs/tungstenite/latest/tungstenite/) — WebSocket
- [axum](https://docs.rs/axum/latest/axum/) — HTTP framework
- [ENet](https://github.com/zakarumych/enet) — Reliable UDP

### 22.5 Deterministic Simulation

- [Factorio Multiplayer](https://wiki.factorio.com/Multiplayer) — Lockstep implementation
- [Age of Empires II DE Netcode](https://www.ageofempires.com/) — Lockstep + rollback
- [Gaffer on Games](https://gafferongames.com/) — Networking articles
- [Deterministic Lockstep](https://www.gamedeveloper.com/programming/deterministic-lockstep) — Implementation guide

### 22.6 Rust Crates

- [rand_chacha](https://docs.rs/rand_chacha/latest/rand_chacha/) — ChaCha RNG
- [fixed](https://docs.rs/fixed/latest/fixed/) — Fixed-point arithmetic
- [indexmap](https://docs.rs/indexmap/latest/indexmap/) — Ordered hash map
- [bincode](https://docs.rs/bincode/latest/bincode/) — Binary serialization
- [xxhash-rust](https://docs.rs/xxhash-rust/latest/xxhash_rust/) — Fast hashing
- [wasmtime](https://docs.rs/wasmtime/latest/wasmtime/) — WASM runtime
- [kira](https://docs.rs/kira/latest/kira/) — Audio library
- [rapier](https://rapier.rs/) — Physics engine

### 22.7 Academic References

- Muratori, Casey. "Handmade Hero" — Data-oriented design
- Mitton, Randy. "Data-Oriented Design" — Book on ECS patterns
- Nystrom, Robert. "Game Programming Patterns" — Component pattern
- Coulton, et al. "ECS Performance Analysis" — Benchmarking study
- Factorio Blog. "Multiplayer Implementation" — Lockstep details

---

*This research document informs the technical architecture of Civis, a deterministic civilization simulation engine. Last updated: 2026-04-03.*
