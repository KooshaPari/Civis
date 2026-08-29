---
title: Simulation
description: Tick loop, ECS components, world state, and determinism in the Civis simulation engine.
---

# Simulation

## Overview

The simulation engine is the deterministic heart of Civis. It runs a fixed-rate tick loop over an ECS world, applies per-phase updates, and emits snapshots that any client can replay given the same seed. The engine is pure: no I/O, no time source, no global state — every dependency is passed in.

The simulation is driven by `Simulation` (in `crates/engine/src/lib.rs`), which owns a `WorldState`, a `hecs::World`, and a `ChaCha8Rng`. `Simulation::new()` creates a default seed-42 simulation with starter entities; `Simulation::with_seed(seed)` accepts a caller-supplied deterministic seed for reproducible runs.

## World State

`WorldState` is the serializable global state used by both `step` and `Simulation`:

| Field | Type | Description |
|-------|------|-------------|
| `tick` | `u64` | Current simulation tick. |
| `population` | `u64` | Aggregate population count. |
| `energy_budget_joules` | `Fixed` | Remaining energy budget. |
| `rng_seed` | `u64` | Seed for deterministic random behavior. |
| `factions` | `HashMap<u32, String>` | Faction ID to display name. |
| `faction_treasury` | `HashMap<u32, Fixed>` | Faction ID to treasury balance. |

`WorldState::default()` starts at tick `0`, population `1_000_000`, energy budget `1_000_000_000_000`, RNG seed `42`, and three default factions.

## Tick Loop

The tick loop is a 60 Hz cycle (16.67 ms per tick). Each tick advances production, citizen lifecycle, military, and economy phases in order:

```rust
let mut sim = Simulation::new();
for _ in 0..N {
    sim.tick();
}
let snap = sim.snapshot();
```

`step(state, consumption_joules)` is the smallest deterministic state transition for callers that want raw state evolution without the full ECS world. It increments `state.tick`, subtracts consumption, and floors energy at zero without mutating the caller's original state.

## Components

The engine exports the following ECS component types:

| Component | Key fields |
|-----------|------------|
| `Position` | `x`, `y` hex-grid coordinates. |
| `Citizen` | `age`, `health`, `ideology`, `welfare`, `job`. |
| `Building` | `building_type`, `hp`, `max_hp`, `position`. |
| `Resources` | `food`, `wood`, `metal`, `energy`. |
| `Production` | `output_type`, `rate`. |
| `MilitaryUnit` | `unit_type`, `strength`, `morale`, `position`, `faction_id`. |

Enums:

- `JobType`: `Farmer`, `Warrior`, `Scholar`, `Trader`, `Priest`, `Admin`, `Unemployed`.
- `BuildingType`: `Farm`, `Mine`, `Barracks`, `Temple`, `Market`, `House`, `CityCenter`.
- `ResourceType`: `Food`, `Wood`, `Metal`, `Energy`.
- `UnitType`: `Soldier`, `Archer`, `Knight`, `Scout`.

Starter entities in `Simulation::new()`: 100 citizens, 1 city center, 5 farms, 10 soldier units.

## Fixed-Point Arithmetic

`Fixed` is the deterministic numeric type used by the simulation. Values are stored as scaled `i64` integers with `SCALE = 1_000_000`:

```rust
use civ_engine::Fixed;

let joules = Fixed::from_num(1_000_000); // 1.0 joule
let half   = joules / Fixed::from_num(2);
```

Available conversions and operations:

| API | Description |
|-----|-------------|
| `Fixed::ZERO` / `Fixed::ONE` | Constants. |
| `Fixed::from_num(value)` | Convert from an integer-like value. |
| `Fixed::from_raw(raw)` | Wrap an already-scaled raw value. |
| `to_f64()` | Convert to floating point for display. |
| `saturating_add(other)` / `saturating_sub(other)` | Overflow-safe arithmetic. |
| `clamp(min, max)` | Inclusive bounds. |

`Fixed` implements `Add`, `Sub`, `Mul`, `Div`, `AddAssign`, `SubAssign`, `Serialize`, and `Deserialize`.

## Query Patterns

The ECS query API uses `hecs`:

```rust
// Query all farmers.
let mut q = world.query::<(&Citizen, &Position)>();
for (_, (citizen, pos)) in q.iter() {
    if matches!(citizen.job, JobType::Farmer) {
        // ...
    }
}

// Mutable iteration for age updates.
for (_, mut citizen) in world.query::<&mut Citizen>().iter() {
    citizen.age += 1;
}
```

## Snapshots and Replay

`SimulationSnapshot` contains `tick`, `population`, `citizen_count`, `building_count`, `military_count`, and `energy_budget`. Snapshots serialize with `serde_json` and can be written to disk via `crates/save-db` for replay:

```bash
cargo run -p civis-cli -- replay save --path ./replays/run-001.json
cargo run -p civis-cli -- replay load --path ./replays/run-001.json
```

## Metrics

`metrics::compute(energy_budget_joules, consumption_joules)` returns:

| Field | Formula |
|-------|---------|
| `waste_joules` | `consumption_joules * 0.1`, floored at zero. |
| `surplus_joules` | `energy_budget_joules - consumption_joules`, floored at zero. |
| `tyranny_index` | `consumption_joules / (energy_budget_joules + 1.0)`, capped at `1.0`. |
| `legitimacy_index` | `1.0 - tyranny_index`, floored at zero. |

## See Also

- [Architecture](/architecture/) — workspace layout and engine boundary.
- [Economy](/economy/) — Joule budget and scarcity policy that drive simulation energy.
- [AI](/ai/) — citizen and faction behavior that runs inside the AI phase.
- [Diplomacy](/diplomacy/) — faction relations, treaties, war escalation.
- [API](/api/) — JSON-RPC methods that advance, reset, and inspect the simulation.
- [Deployment](/deployment/) — running the simulation server in production.